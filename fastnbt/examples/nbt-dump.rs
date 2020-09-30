use fastnbt::nbt;
use flate2::read::GzDecoder;
use nbt::stream;
use std::io;

//
// This example uses the streaming parser to simply dump NBT from stdin with GZip compression.
//

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
