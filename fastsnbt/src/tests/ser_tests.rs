use crate::to_string;
use serde::Serialize;
use fastnbt::{ByteArray, IntArray, LongArray};

#[test]
fn test_true() {
    let snbt = to_string(&true).unwrap();
    assert_eq!("true", snbt);
    let snbt = to_string(&false).unwrap();
    assert_eq!("false", snbt);
}

#[test]
fn test_string_escape() {
    let string = "this str \" contains \" quotes";
    let snbt = to_string(string).unwrap();
    assert_eq!("\"this str \\\" contains \\\" quotes\"", snbt);
    let string = "str \\\" with \" quotes \\ & backslashes";
    let snbt = to_string(string).unwrap();
    assert_eq!("\"str \\\\\\\" with \\\" quotes \\\\ & backslashes\"", snbt);
}

#[test]
fn test_byte() {
    let byte = 10u8;
    let snbt = to_string(&byte).unwrap();
    assert_eq!("10b", snbt);
}

#[test]
fn test_short() {
    let byte = 10u16;
    let snbt = to_string(&byte).unwrap();
    assert_eq!("10s", snbt);
}

#[test]
fn test_int() {
    let byte = -10;
    let snbt = to_string(&byte).unwrap();
    assert_eq!("-10", snbt);
}

#[test]
fn test_long() {
    let byte = 10u64;
    let snbt = to_string(&byte).unwrap();
    assert_eq!("10l", snbt);
}

#[test]
fn test_float() {
    let byte = 10.4f32;
    let snbt = to_string(&byte).unwrap();
    assert_eq!("10.4f", snbt);
}

#[test]
fn test_double() {
    let byte = 10.4f64;
    let snbt = to_string(&byte).unwrap();
    assert_eq!("10.4", snbt);
}

#[test]
fn test_char() {
    let char = '"';
    let snbt = to_string(&char).unwrap();
    assert_eq!("\"\\\"\"", snbt);
}

#[test]
fn test_struct() {
    #[derive(Serialize)]
    struct SimpleStruct<'a> {
        x: u8,
        y: &'a str,
    }

    let data = SimpleStruct { x: 10, y: "test" };
    let snbt = to_string(&data).unwrap();
    assert_eq!("{\"x\":10b,\"y\":\"test\"}", snbt);
}

#[test]
fn test_normal_array() {
    #[derive(Serialize)]
    struct ByteStruct {
        bytes: Vec<u8>,
    }

    let data = ByteStruct { bytes: vec![0, 1, 2, 3] };
    let snbt = to_string(&data).unwrap();
    assert_eq!("{\"bytes\":[0b,1b,2b,3b]}", snbt);
}

#[test]
fn test_bytearray() {
    let data = ByteArray::new(vec![-1, 2, -3, 4]);
    let snbt = to_string(&data).unwrap();
    assert_eq!("[B;-1b,2b,-3b,4b]", snbt);
}

#[test]
fn test_intarray() {
    let data = IntArray::new(vec![-1, 2, -3, 4]);
    let snbt = to_string(&data).unwrap();
    assert_eq!("[I;-1,2,-3,4]", snbt);
}

#[test]
fn test_longarray() {
    let data = LongArray::new(vec![-1, 2, -3, 4]);
    let snbt = to_string(&data).unwrap();
    assert_eq!("[L;-1l,2l,-3l,4l]", snbt);
}

#[test]
fn test_struct_arrays() {
    #[derive(Serialize)]
    struct ArrayStruct {
        bytes: ByteArray,
        longs: LongArray,
    }

    let data = ArrayStruct { bytes: ByteArray::new(vec![-20,10]), longs: LongArray::new(vec![-40, 10_000]) };
    let snbt = to_string(&data).unwrap();
    assert_eq!("{\"bytes\":[B;-20b,10b],\"longs\":[L;-40l,10000l]}", snbt);
}
