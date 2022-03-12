#![no_main]
use libfuzzer_sys::fuzz_target;

use fastnbt::de::from_bytes;
use fastnbt::error::Result;
use fastnbt::ser::to_bytes;
use fastnbt::Value;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let value: Result<Value> = from_bytes(data);
    if let Ok(v) = value {
        let _bs = to_bytes(&v).unwrap();
    }
});
