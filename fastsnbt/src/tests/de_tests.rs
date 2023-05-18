use fastnbt::{ByteArray, IntArray, LongArray};
use serde::Deserialize;

use crate::from_str;


#[test]
fn test_num() {
    let input = "20b";
    let v: i32 = from_str(input).unwrap();
    assert_eq!(20, v);
    let input = "20s";
    let v: i32 = from_str(input).unwrap();
    assert_eq!(20, v);
    let input = "20l";
    let v: i32 = from_str(input).unwrap();
    assert_eq!(20, v);
    let input = "20l";
    let v: u8 = from_str(input).unwrap();
    assert_eq!(20, v);
}

#[test]
fn test_float() {
    let input = "50.";
    let f: f64 = from_str(input).unwrap();
    assert_eq!(50., f);
    let input = "-5000.e-2f";
    let f: f64 = from_str(input).unwrap();
    assert_eq!(-50., f);
    let input = "inf";
    let f: f64 = from_str(input).unwrap();
    assert_eq!(f64::INFINITY, f);
}

#[test]
fn test_str() {
    let input = "\"simple\"";
    let s: &str = from_str(input).unwrap();
    assert_eq!("simple", s);
    let input = "no+.quo0tes";
    let s: &str = from_str(input).unwrap();
    assert_eq!("no+.quo0tes", s);
    let input = "'this\\'is a string'";
    let s: String = from_str(input).unwrap();
    assert_eq!("this\'is a string", s);
    let input = "\"yet\\\"ano\\\\\\\"ther\"";
    let s: String = from_str(input).unwrap();
    assert_eq!("yet\"ano\\\"ther", s);
    assert!(from_str::<&str>("\"not closed").is_err());
    assert!(from_str::<&str>("test/").is_err());
}

#[test]
fn test_seq() {
    let input = "[1b,2b,3b]";
    let bytes: Vec<u8> = from_str(input).unwrap();
    assert_eq!(&[1, 2, 3], bytes.as_slice());
    let input = "[1B,2,5.0e1D]";
    let data: (i8,u64,f64) = from_str(input).unwrap();
    assert_eq!((1, 2, 50.), data);
}

#[test]
fn test_map() {
    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct SimpleStruct<'a> {
        s: &'a str,
        x: i16,
    }

    let input = "{x:-10,s:test}";
    let data: SimpleStruct = from_str(input).unwrap();
    assert_eq!(SimpleStruct { s: "test", x: -10 }, data);
}

#[test]
fn test_bytearray() {
    let input = "[B;1b,-2b,3B]";
    let data: ByteArray = from_str(input).unwrap();
    assert_eq!(ByteArray::new(vec![1,-2,3]), data);
}

#[test]
fn test_intarray() {
    let input = "[I;1,2,-3]";
    let data: IntArray = from_str(input).unwrap();
    assert_eq!(IntArray::new(vec![1,2,-3]), data);
}

#[test]
fn test_longarray() {
    let input = "[L;1l,2L,-3l]";
    let data: LongArray = from_str(input).unwrap();
    assert_eq!(LongArray::new(vec![1,2,-3]), data);
}
