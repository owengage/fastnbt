#![no_main]
use libfuzzer_sys::fuzz_target;

use std::collections::HashMap;

use fastnbt::error::Result;
use fastnbt::to_bytes;
use fastnbt::Value;
use fastnbt::{from_bytes_with_opts, DeOpts};

fuzz_target!(|data: &[u8]| {
    let value: Result<Value> = from_bytes_with_opts(data, DeOpts::new().max_seq_len(100));
    if let Ok(v) = value {
        let mut wrapper = HashMap::new();
        wrapper.insert("wrapper".to_string(), v);
        let _bs = to_bytes(&Value::Compound(wrapper)).unwrap();
    }
});
