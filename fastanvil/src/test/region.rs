use std::io::{Cursor, Read, Seek, Write};

use crate::{
    ChunkLocation, CompressionScheme::Uncompressed, Error, Region, CHUNK_HEADER_SIZE,
    REGION_HEADER_SIZE, SECTOR_SIZE,
};

fn new_empty() -> Region<Cursor<Vec<u8>>> {
    Region::new(Cursor::new(vec![])).unwrap()
}

fn assert_location<S>(r: &mut Region<S>, x: usize, z: usize, offset: u64, size: u64)
where
    S: Read + Write + Seek,
{
    let ChunkLocation {
        offset: found_offset,
        sectors: found_size,
    } = r.location(x, z).unwrap().unwrap();

    assert_eq!(offset, found_offset);
    assert_eq!(size, found_size);
}

fn n_sector_chunk(n: usize) -> Vec<u8> {
    assert!(n > 0);
    vec![0; (n * SECTOR_SIZE) - CHUNK_HEADER_SIZE]
}

#[test]
fn new_region_should_be_empty() {
    let mut r = new_empty();

    for x in 0..32 {
        for z in 0..32 {
            let chunk = r.read_chunk(x, z);
            assert!(matches!(chunk, Ok(None)))
        }
    }
}

#[test]
fn blank_write_chunk() {
    let mut r = new_empty();
    r.write_compressed_chunk(0, 0, Uncompressed, &[1, 2, 3])
        .unwrap();
    assert_location(&mut r, 0, 0, 2, 1);
}

#[test]
fn write_invalid_offset_errors() {
    let mut r = new_empty();
    assert!(matches!(
        r.write_compressed_chunk(32, 0, Uncompressed, &[1, 2, 3]),
        Err(Error::InvalidOffset(..))
    ));
    assert!(matches!(
        r.write_compressed_chunk(0, 32, Uncompressed, &[1, 2, 3]),
        Err(Error::InvalidOffset(..))
    ));
}

#[test]
fn exact_sector_size_chunk_takes_one_sector() {
    let mut r = new_empty();
    r.write_compressed_chunk(0, 0, Uncompressed, &n_sector_chunk(1))
        .unwrap();
    assert_location(&mut r, 0, 0, 2, 1);
}

#[test]
fn over_one_sector_size_chunk_takes_two_sectors() {
    let mut r = new_empty();
    r.write_compressed_chunk(
        0,
        0,
        Uncompressed,
        &[0; SECTOR_SIZE - CHUNK_HEADER_SIZE + 1],
    )
    .unwrap();
    assert_location(&mut r, 0, 0, 2, 2);
}

#[test]
fn several_sector_chunk_takes_correct_size() {
    let mut r = new_empty();
    r.write_compressed_chunk(0, 0, Uncompressed, &n_sector_chunk(5))
        .unwrap();
    assert_location(&mut r, 0, 0, 2, 5);
}

#[test]
fn oversized_chunk_fails() {
    let mut r = new_empty();
    let res = r.write_compressed_chunk(0, 0, Uncompressed, &n_sector_chunk(256));
    assert!(matches!(res, Err(Error::ChunkTooLarge)))
}

#[test]
fn write_several_chunks() {
    let mut r = new_empty();
    r.write_compressed_chunk(0, 0, Uncompressed, &n_sector_chunk(2))
        .unwrap();
    r.write_compressed_chunk(0, 1, Uncompressed, &n_sector_chunk(3))
        .unwrap();

    assert_location(&mut r, 0, 0, 2, 2);
    assert_location(&mut r, 0, 1, 4, 3);
}

#[test]
fn write_and_get_chunk() {
    let mut r = new_empty();
    r.write_compressed_chunk(0, 0, Uncompressed, &[1, 2, 3])
        .unwrap();
    let c = r.read_chunk(0, 0).unwrap().unwrap();
    assert_eq!(c, &[1, 2, 3]);
}

#[test]
fn getting_other_chunks_404s() {
    let mut r = new_empty();
    r.write_compressed_chunk(1, 1, Uncompressed, &[1, 2, 3])
        .unwrap();
    assert!(matches!(r.read_chunk(0, 0), Ok(None)));
    assert!(matches!(r.read_chunk(1, 0), Ok(None)));
    assert!(matches!(r.read_chunk(1, 1), Ok(Some(_))));
}

#[test]
fn overwrite_with_smaller_chunk() {
    let mut r = new_empty();
    r.write_compressed_chunk(0, 0, Uncompressed, &n_sector_chunk(2))
        .unwrap();
    r.write_compressed_chunk(0, 0, Uncompressed, &n_sector_chunk(1))
        .unwrap();

    assert_location(&mut r, 0, 0, 2, 1);
}

#[test]
fn overwrite_with_larger_chunk() {
    let mut r = new_empty();
    r.write_compressed_chunk(0, 0, Uncompressed, &n_sector_chunk(2))
        .unwrap();

    // this chunk will be offset 4 size 1.
    r.write_compressed_chunk(0, 1, Uncompressed, &n_sector_chunk(1))
        .unwrap();

    // overwrite chunk at offset 2 to be 3 large, which would overwrite the
    // above chunk if done in-place.
    r.write_compressed_chunk(0, 0, Uncompressed, &n_sector_chunk(3))
        .unwrap();

    // sectors now look like [H,H,??,??,01,00,00,00]
    assert_location(&mut r, 0, 0, 5, 3);
}

#[test]
fn chunk_can_fill_gap_left_by_moved_chunk_after_it() {
    let mut r = new_empty();
    // HH000111222---- - starting point, chunks 0,1,2 all 3 sectors
    // HH000---2221111 - chunk 1 grows beyond capcacity, moves to end.
    // HH0000002221111 - chunk 0 can grow to 6 sectors.

    r.write_compressed_chunk(0, 0, Uncompressed, &n_sector_chunk(3))
        .unwrap();
    r.write_compressed_chunk(0, 1, Uncompressed, &n_sector_chunk(3))
        .unwrap();
    r.write_compressed_chunk(0, 2, Uncompressed, &n_sector_chunk(3))
        .unwrap();

    // chunk 0,1 gets moved to the end
    r.write_compressed_chunk(0, 1, Uncompressed, &n_sector_chunk(4))
        .unwrap();

    // chunk 0,0 can grow to 6 without moving.
    r.write_compressed_chunk(0, 0, Uncompressed, &n_sector_chunk(6))
        .unwrap();

    // HH0000002221111
    assert_location(&mut r, 0, 0, 2, 6);
    assert_location(&mut r, 0, 1, 11, 4);
    assert_location(&mut r, 0, 2, 8, 3);
}

#[test]
fn load_from_existing_buffer() {
    let mut r = new_empty();
    r.write_compressed_chunk(0, 0, Uncompressed, &n_sector_chunk(1))
        .unwrap();
    r.write_compressed_chunk(0, 1, Uncompressed, &n_sector_chunk(2))
        .unwrap();

    let buf = r.into_inner().unwrap();

    // reload the region
    let mut r = Region::from_stream(buf).unwrap();
    assert_location(&mut r, 0, 0, 2, 1);
    assert_location(&mut r, 0, 1, 3, 2);
}

#[test]
fn deleted_chunk_doenst_exist() {
    let mut r = new_empty();

    r.write_compressed_chunk(0, 0, Uncompressed, &n_sector_chunk(3))
        .unwrap();
    r.write_compressed_chunk(0, 1, Uncompressed, &n_sector_chunk(3))
        .unwrap();
    r.write_compressed_chunk(0, 2, Uncompressed, &n_sector_chunk(3))
        .unwrap();

    r.remove_chunk(0, 1).unwrap();

    assert!(matches!(r.read_chunk(0, 0), Ok(Some(_))));
    assert!(matches!(r.read_chunk(0, 1), Ok(None)));
    assert!(matches!(r.read_chunk(0, 2), Ok(Some(_))));
}

#[test]
fn deleting_non_existing_chunk_works() {
    let mut r = new_empty();

    r.write_compressed_chunk(0, 0, Uncompressed, &n_sector_chunk(3))
        .unwrap();
    r.write_compressed_chunk(0, 2, Uncompressed, &n_sector_chunk(3))
        .unwrap();

    r.remove_chunk(0, 1).unwrap();

    assert!(matches!(r.read_chunk(0, 0), Ok(Some(_))));
    assert!(matches!(r.read_chunk(0, 1), Ok(None)));
    assert!(matches!(r.read_chunk(0, 2), Ok(Some(_))));
}

#[test]
fn into_inner_rewinds_to_correct_position() {
    let mut r = new_empty();

    r.write_compressed_chunk(0, 0, Uncompressed, &n_sector_chunk(3))
        .unwrap();
    r.write_compressed_chunk(0, 1, Uncompressed, &n_sector_chunk(3))
        .unwrap();
    r.write_compressed_chunk(0, 2, Uncompressed, &n_sector_chunk(3))
        .unwrap();

    let expected_position = REGION_HEADER_SIZE + SECTOR_SIZE * 3 * 3;

    let inner = r.into_inner().unwrap();
    assert_eq!(inner.position(), expected_position as u64);
}

#[test]
fn into_inner_rewinds_behind_header_if_empty_region() {
    let r = new_empty();

    let inner = r.into_inner().unwrap();
    assert_eq!(inner.position(), REGION_HEADER_SIZE as u64);
}

// TODO: Should we always zero out space? Would likely be good for compression.
// TODO: defrag?

// TODO: Worry about atomicity of underlying buffer? The Read+Write+Seek can't
// really provide us with atomicity, we'd probably need some highly level
// abstraction on top of this providing this. Something that copies a region and
// only write the to copy until done, then atomically moves the file over the
// old region.
