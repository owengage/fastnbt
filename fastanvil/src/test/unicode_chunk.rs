use crate::JavaChunk;

const UNICODE_CHUNK: &[u8] = include_bytes!("../../resources/unicode.chunk");

#[test]
fn unicode_chunk() {
    // This chunk contains unicode that isn't on the basic multilingual plane.
    // Characters off this plane are encoded in a modified form of cesu8 which
    // is an encoding for unicode. Rust uses utf-8 for strings so there can be
    // conflicts if not deserialized properly.
    let c = JavaChunk::from_bytes(UNICODE_CHUNK);
    assert!(c.is_ok());
}
