use fastnbt::stream::{Parser, Value};
use flate2::read::GzDecoder;
use std::io;

//
// This example uses the streaming parser to simply dump NBT from stdin with GZip compression.
//

fn main() {
    let stdin = io::stdin();
    let decoder = GzDecoder::new(stdin);

    let mut parser = Parser::new(decoder);
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
}
