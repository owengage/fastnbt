use core::panic;
use std::{
    error::Error,
    fs::{create_dir, File},
    io::{self, Write},
};

use clap::{App, Arg};
use env_logger::Env;
use fastanvil::RegionBuffer;
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

    let mut region = RegionBuffer::new(file);

    if let Some(dir) = out_dir {
        create_dir(dir).unwrap_or_default();
    }

    region
        .for_each_chunk(|x, z, data| {
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

            let chunk: Value = fastnbt::from_bytes(data).unwrap();

            match output_format {
                "rust" => {
                    write!(&mut out, "{:?}", chunk).unwrap();
                }
                "rust-pretty" => {
                    write!(&mut out, "{:#?}", chunk).unwrap();
                }
                "nbt" => {
                    out.write_all(data).unwrap();
                }
                "json" => {
                    serde_json::ser::to_writer(out, &chunk).unwrap();
                }
                "json-pretty" => {
                    serde_json::ser::to_writer_pretty(out, &chunk).unwrap();
                }
                _ => panic!("unknown output format '{}'", output_format),
            }
        })
        .map_err(|e| e.into())
}
