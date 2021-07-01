#![no_main]
use libfuzzer_sys::fuzz_target;

use fastanvil::RegionBuffer;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let reader = Cursor::new(data);
    // TODO: this function expects the input to be compressed, find a way to avoid that
    let r = RegionBuffer::new(reader);
    let _location = r.chunk_location(0, 0);
    let _chunk = r.load_chunk(0, 0);
});
