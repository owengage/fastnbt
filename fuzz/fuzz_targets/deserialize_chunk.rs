#![no_main]
use libfuzzer_sys::fuzz_target;

use fastanvil::JavaChunk;
use fastnbt::error::Result;
use fastnbt::de::from_bytes;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let _chunk: Result<JavaChunk> = from_bytes(data);
});
