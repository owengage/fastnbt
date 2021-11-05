use std::io::Write;

use fastanvil::RegionBuffer;
use fastnbt::Value;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let file = std::fs::File::open(args[0].clone()).unwrap();

    let mut region = RegionBuffer::new(file);

    region
        .for_each_chunk(|x, z, data| {
            let mut file = std::fs::File::create(format!("chunks/{}.{}.nbt", x, z)).unwrap();
            file.write_all(data).unwrap();

            let mut file = std::fs::File::create(format!("chunks/{}.{}.txt", x, z)).unwrap();
            let chunk: Value = fastnbt::de::from_bytes(data).unwrap();

            match &chunk {
                Value::Compound(c) => match &c["sections"] {
                    Value::List(sections) => {
                        for sec in sections {
                            match sec {
                                Value::Compound(sec) => match &sec["biomes"] {
                                    Value::Compound(biomes) => match &biomes["palette"] {
                                        Value::List(palette) => {
                                            if palette.len() > 2 {
                                                println!("Long palette! {}", palette.len());
                                                println!("{:?}", biomes);
                                            }
                                        }
                                        _ => panic!("no palette"),
                                    },
                                    _ => panic!("no biomes"),
                                },
                                _ => panic!("no section"),
                            }
                        }
                    }
                    _ => panic!("no sections"),
                },
                _ => panic!("no chunk"),
            }

            file.write_all(format!("{:#?}", chunk).as_bytes()).unwrap();
        })
        .unwrap();
}
