use fastnbt::Value;
use flate2::read::GzDecoder;
use std::io;
use std::io::Read;

fn main() {
    let stdin = io::stdin();
    let mut decoder = GzDecoder::new(stdin);
    let mut buf = vec![];
    decoder.read_to_end(&mut buf).unwrap();

    let val: Value = fastnbt::from_bytes(buf.as_slice()).unwrap();

    println!("{:#?}", val);
}
