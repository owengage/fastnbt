use crate::nbt::test::builder::Builder;
use crate::nbt::tokenizer::{Token, Tokenizer};
use crate::nbt::Tag;

#[test]
fn can_tokenize_start_tag() {
    let payload = Builder::new().tag(Tag::Byte).build();
    let mut tok = Tokenizer::new(payload.as_slice());

    let payload = Builder::new().tag(Tag::Compound).build();
    let mut tok = Tokenizer::new(payload.as_slice());
    assert_eq!(tok.next().unwrap(), Token::Tag(Tag::Compound));
}

#[test]
fn tokenize_byte() {
    let payload = Builder::new()
        .tag(Tag::Byte)
        .name("some-name")
        .byte_payload(123)
        .build();

    let mut tok = Tokenizer::new(payload.as_slice());

    assert_eq!(tok.next().unwrap(), Token::Tag(Tag::Byte));
    assert_eq!(tok.next().unwrap(), Token::Name("some-name"));
    assert_eq!(tok.next().unwrap(), Token::Byte(123));
}

fn tokenize_short() {
    let payload = Builder::new()
        .tag(Tag::Byte)
        .name("another-name")
        .short_payload(256)
        .build();

    let mut tok = Tokenizer::new(payload.as_slice());

    assert_eq!(tok.next().unwrap(), Token::Tag(Tag::Byte));
    assert_eq!(tok.next().unwrap(), Token::Name("another-name"));
    assert_eq!(tok.next().unwrap(), Token::Short(256));
}

// Empty name
