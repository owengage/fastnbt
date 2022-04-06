use std::io::Cursor;

use crate::{CCoord, Error, Region, RegionBuffer, RegionRead, RegionWrite};

fn new_buf() -> Cursor<Vec<u8>> {
    Cursor::new(vec![])
}

#[test]
fn new_region_should_be_empty() {
    let r = RegionBuffer::new_empty(new_buf()).unwrap();

    for x in 0..32 {
        for z in 0..32 {
            let chunk = r.read_chunk(CCoord(x), CCoord(z));
            assert!(matches!(chunk, Err(Error::ChunkNotFound)))
        }
    }
}

// #[test]
// fn write_and_get_chunk() {
//     let r = RegionBuffer::new_empty(new_buf()).unwrap();
//     r.write_chunk(CCoord(0), CCoord(0), &[1, 2, 3]).unwrap();
//     let c = r.read_chunk(CCoord(0), CCoord(0)).unwrap();
//     assert_eq!(c, &[1, 2, 3]);
// }
