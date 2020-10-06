use fastnbt::stream;
use flate2::read::GzDecoder;
use std::io;

fn main() {
    let stdin = io::stdin();
    let decoder = GzDecoder::new(stdin);

    let mut parser = stream::Parser::new(decoder);
    let mut indent = 0;

    loop {
        match parser.next() {
            Err(e) => {
                println!("{:?}", e);
                break;
            }
            Ok(value) => {
                match value {
                    stream::Value::CompoundEnd => indent -= 4,
                    stream::Value::ListEnd => indent -= 4,
                    _ => {}
                }

                println!("{:indent$}{:?}", "", value, indent = indent);

                match value {
                    stream::Value::Compound(_) => indent += 4,
                    stream::Value::List(_, _, _) => indent += 4,
                    _ => {}
                }
            }
        }
    }
}
