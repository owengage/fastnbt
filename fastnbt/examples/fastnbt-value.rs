use fastnbt::Value2;
use flate2::read::GzDecoder;
use std::io;
use std::io::Read;

fn main() {
    let stdin = io::stdin();
    let mut decoder = GzDecoder::new(stdin);
    let mut buf = vec![];
    decoder.read_to_end(&mut buf).unwrap();

    let val: Value2 = fastnbt::de::from_bytes(buf.as_slice()).unwrap();

    println!("{:#?}", val);
}
