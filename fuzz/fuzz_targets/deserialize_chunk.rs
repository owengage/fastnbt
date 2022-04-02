#![no_main]
use libfuzzer_sys::fuzz_target;

use fastanvil::JavaChunk;
use fastnbt::error::Result;
use fastnbt::from_bytes;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let _chunk: Result<JavaChunk> = from_bytes(data);
});
