use core::panic;
use std::{
    error::Error,
    fs::{create_dir, File},
    io::{self, Write},
};

use clap::{App, Arg};
use env_logger::Env;
use fastanvil::Region;
use fastnbt::Value;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    let matches = App::new("region-dump")
        .arg(Arg::with_name("file").required(true))
        .arg(
            Arg::with_name("format")
                .long("format")
                .short("f")
                .takes_value(true)
                .required(false)
                .default_value("rust")
                .possible_values(&["rust", "rust-pretty", "json", "json-pretty", "nbt"])
                .help("output format"),
        )
        .arg(
            Arg::with_name("out-dir")
                .long("out-dir")
                .short("o")
                .takes_value(true)
                .required(false)
                .help("optionally separate each chunk into a file in the specified directory"),
        )
        .get_matches();

    let file = matches.value_of("file").expect("file is required");
    let file = File::open(file).expect("file does not exist");
    let output_format = matches
        .value_of("format")
        .expect("no output format specified");
    let out_dir = matches.value_of("out-dir");

    let mut region = Region::from_stream(file).unwrap();

    if let Some(dir) = out_dir {
        create_dir(dir).unwrap_or_default();
    }

    for z in 0..32 {
        for x in 0..32 {
            match region.read_chunk(x, z) {
                Ok(Some(data)) => {
                    if !should_output_chunk(&data) {
                        continue;
                    }

                    let mut out: Box<dyn Write> = if let Some(dir) = out_dir {
                        let ext = match output_format {
                            "nbt" => "nbt",
                            "json" | "json-pretty" => "json",
                            _ => "txt",
                        };
                        Box::new(File::create(format!("{}/{}.{}.{}", dir, x, z, ext)).unwrap())
                    } else {
                        Box::new(io::stdout())
                    };

                    let chunk: Value = fastnbt::from_bytes(&data).unwrap();

                    match output_format {
                        "rust" => {
                            write!(&mut out, "{:?}", chunk).unwrap();
                        }
                        "rust-pretty" => {
                            write!(&mut out, "{:#?}", chunk).unwrap();
                        }
                        "nbt" => {
                            out.write_all(&data).unwrap();
                        }
                        "json" => {
                            serde_json::ser::to_writer(out, &chunk).unwrap();
                        }
                        "json-pretty" => {
                            serde_json::ser::to_writer_pretty(out, &chunk).unwrap();
                        }
                        _ => panic!("unknown output format '{}'", output_format),
                    }
                }
                Ok(None) => {}
                Err(e) => return Err(e.into()),
            }
        }
    }
    Ok(())
}

fn should_output_chunk(_data: &[u8]) -> bool {
    // If you're trying to locate a misbehaving chunk, you can filter out chunks here.
    // let chunk = JavaChunk::from_bytes(data).unwrap();
    true
}
