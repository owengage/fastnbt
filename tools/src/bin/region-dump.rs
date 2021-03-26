use fastanvil::RegionBuffer;
use fastnbt::stream::{Parser, Value};

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let file = std::fs::File::open(args[0].clone()).unwrap();

    let mut region = RegionBuffer::new(file);

    region
        .for_each_chunk(|_x, _z, data| {
            let mut parser = Parser::new(data.as_slice());
            let mut indent = 0;

            loop {
                match parser.next() {
                    Err(e) => {
                        println!("{:?}", e);
                        break;
                    }
                    Ok(value) => {
                        match value {
                            Value::CompoundEnd => indent -= 4,
                            Value::ListEnd => indent -= 4,
                            _ => {}
                        }

                        println!("{:indent$}{:?}", "", value, indent = indent);

                        match value {
                            Value::Compound(_) => indent += 4,
                            Value::List(_, _, _) => indent += 4,
                            _ => {}
                        }
                    }
                }
            }
        })
        .unwrap();
}
