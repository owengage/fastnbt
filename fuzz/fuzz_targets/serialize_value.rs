#![no_main]
use libfuzzer_sys::fuzz_target;

use serde::Serialize;
use std::collections::HashMap;

use fastnbt::error::Result;
use fastnbt::from_bytes;
use fastnbt::to_bytes;
use fastnbt::Value;

fuzz_target!(|v: Value| {
    let mut inner = HashMap::new();
    inner.insert("".to_string(), v);

    let v = Value::Compound(inner);
    let bs = to_bytes(&v);

    if let Ok(bs) = bs {
        let _: Result<Value> = from_bytes(&bs);
    }
});
