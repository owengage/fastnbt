#![no_main]
use libfuzzer_sys::fuzz_target;

use fastnbt::de::{from_bytes_with_opts, DeOpts};
use fastnbt::error::Result;
use fastnbt::ser::to_bytes;
use fastnbt::Value;

fuzz_target!(|data: &[u8]| {
    let value: Result<Value> = from_bytes_with_opts(data, DeOpts { max_seq_len: 1024 });
    if let Ok(v) = value {
        let _bs = to_bytes(&v).unwrap();
    }
});
