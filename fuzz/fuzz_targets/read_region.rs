#![no_main]
use libfuzzer_sys::fuzz_target;

use fastanvil::Region;
use std::io::Cursor;

fuzz_target!(|data: Vec<u8>| {
    let reader = Cursor::new(data);
    let mut r = Region::create(reader);
    match r {
        Ok(mut r) => {
            r.read_chunk(0, 0);
        }
        Err(_) => {}
    };
});
