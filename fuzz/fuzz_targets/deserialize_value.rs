#![no_main]
use libfuzzer_sys::fuzz_target;

use fastnbt::error::Result;
use fastnbt::to_bytes;
use fastnbt::Value;
use fastnbt::{from_bytes_with_opts, DeOpts};

fuzz_target!(|data: &[u8]| {
    let value: Result<Value> = from_bytes_with_opts(data, DeOpts::new().max_seq_len(100));
    if let Ok(v) = value {
        let _bs = to_bytes(&v).unwrap();
    }
});
