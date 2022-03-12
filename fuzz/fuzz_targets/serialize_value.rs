#![no_main]
use libfuzzer_sys::fuzz_target;

use serde::Serialize;
use std::collections::HashMap;

use fastnbt::de::from_bytes;
use fastnbt::error::Result;
use fastnbt::ser::to_bytes;
use fastnbt::Value;

fuzz_target!(|v: Value| {
    // v == Float(1.4e-44)
    let mut inner = HashMap::new();
    inner.insert("".to_string(), v);

    let v = Value::Compound(inner);
    let bs = to_bytes(&v).unwrap();
    println!("{bs:?}");
    let roundtrip: Value = from_bytes(&bs).unwrap(); // anything that serializes should deserialize.

    assert_eq!(v, roundtrip);
});
