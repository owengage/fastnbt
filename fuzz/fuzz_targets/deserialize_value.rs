#![no_main]
use libfuzzer_sys::fuzz_target;

use fastnbt::error::Result;
use fastnbt::de::from_bytes;
use fastnbt::Value;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let _value: Result<Value> = from_bytes(data);
});
