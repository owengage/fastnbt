use super::builder::Builder;
use crate::stream::{ErrorKind, Name, Parser, Result, Value};
use crate::Tag;

fn name(n: &str) -> Name {
    Some(n.to_owned())
}

#[test]
fn empty_payload() {
    let payload = Builder::new().build();
    let mut parser = Parser::new(payload.as_slice());

    let value = parser.next();
    assert!(value.is_err());
}

#[test]
fn simple_byte() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::Byte)
        .name("abc")
        .byte_payload(123)
        .build();

    let mut parser = Parser::new(payload.as_slice());

    assert_eq!(parser.next()?, Value::Byte(name("abc"), 123));
    Ok(())
}

#[test]
fn simple_short() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::Short)
        .name("abc")
        .short_payload(1234)
        .build();

    let mut parser = Parser::new(payload.as_slice());

    assert_eq!(parser.next()?, Value::Short(name("abc"), 1234));
    Ok(())
}

#[test]
fn simple_int() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::Int)
        .name("abc")
        .int_payload(50345)
        .build();

    let mut parser = Parser::new(payload.as_slice());

    assert_eq!(parser.next()?, Value::Int(name("abc"), 50345));
    Ok(())
}

#[test]
fn simple_long() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::Long)
        .name("abc")
        .long_payload(std::i32::MAX as i64 + 1)
        .build();

    let mut parser = Parser::new(payload.as_slice());

    assert_eq!(
        parser.next()?,
        Value::Long(name("abc"), std::i32::MAX as i64 + 1)
    );
    Ok(())
}

#[test]
fn simple_float() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::Float)
        .name("float")
        .float_payload(1.23)
        .build();

    let mut parser = Parser::new(payload.as_slice());

    assert_eq!(parser.next()?, Value::Float(name("float"), 1.23));
    Ok(())
}

#[test]
fn simple_double() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::Double)
        .name("double")
        .double_payload(1.23456)
        .build();

    let mut parser = Parser::new(payload.as_slice());

    assert_eq!(parser.next()?, Value::Double(name("double"), 1.23456));
    Ok(())
}

#[test]
fn simple_string() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::String)
        .name("str")
        .string_payload("something")
        .build();

    let mut parser = Parser::new(payload.as_slice());

    assert_eq!(
        parser.next()?,
        Value::String(name("str"), "something".to_owned())
    );
    Ok(())
}

#[test]
fn cesu8_string_in_nbt() -> Result<()> {
    // In the builder we always convert to java cesu8 form for strings anyway,
    // but this test is more explicit and includes some unicode that actually
    // has a different representation in cesu8 and utf-8.
    let modified_unicode_str = cesu8::to_java_cesu8("😈");

    let payload = Builder::new()
        .tag(Tag::String)
        .name("cesu8")
        .raw_str_len(modified_unicode_str.len())
        .raw_bytes(&modified_unicode_str)
        .build();

    let mut parser = Parser::new(payload.as_slice());

    assert_eq!(
        parser.next()?,
        Value::String(name("cesu8"), "😈".to_owned())
    );

    Ok(())
}

#[test]
fn invalid_unicode_can_carry_on() -> Result<()> {
    let bs = [255, 255, 255];

    let payload = Builder::new()
        .tag(Tag::String)
        .name("invalid")
        .raw_str_len(bs.len())
        .raw_bytes(&bs)
        .string("available", "yes")
        .build();

    let mut parser = Parser::new(payload.as_slice());

    assert!(matches!(
        parser.next().unwrap_err().kind(),
        ErrorKind::Nonunicode(_)
    ));

    let s = parser.next()?;
    assert_eq!(
        s,
        Value::String(Some("available".to_owned()), "yes".to_string())
    );

    Ok(())
}

#[test]
fn simple_byte_array() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::ByteArray)
        .name("bytes")
        .int_payload(3)
        .byte_array_payload(&[1, 2, 3])
        .build();

    let mut parser = Parser::new(payload.as_slice());

    assert_eq!(
        parser.next()?,
        Value::ByteArray(name("bytes"), vec![1, 2, 3])
    );
    Ok(())
}

#[test]
fn simple_int_array() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::IntArray)
        .name("ints")
        .int_payload(3)
        .int_array_payload(&[1, 2, 3])
        .build();

    let mut parser = Parser::new(payload.as_slice());

    assert_eq!(parser.next()?, Value::IntArray(name("ints"), vec![1, 2, 3]));
    Ok(())
}

#[test]
fn simple_long_array() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::LongArray)
        .name("longs")
        .int_payload(5)
        .long_array_payload(&[1, 2, 3, i64::MIN, i64::MAX])
        .build();

    let mut parser = Parser::new(payload.as_slice());

    assert_eq!(
        parser.next()?,
        Value::LongArray(name("longs"), vec![1, 2, 3, i64::MIN, i64::MAX])
    );
    Ok(())
}

#[test]
fn multiple_primatives() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::Double)
        .name("double")
        .double_payload(1.23456)
        .tag(Tag::Int)
        .name("int")
        .int_payload(123456)
        .tag(Tag::Byte)
        .name("byte")
        .byte_payload(123)
        .build();

    let mut parser = Parser::new(payload.as_slice());

    assert_eq!(parser.next()?, Value::Double(name("double"), 1.23456));
    assert_eq!(parser.next()?, Value::Int(name("int"), 123456));
    assert_eq!(parser.next()?, Value::Byte(name("byte"), 123));
    Ok(())
}

#[test]
fn end_of_reader_is_signalled() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::Byte)
        .name("byte")
        .byte_payload(123)
        .build();

    let mut parser = Parser::new(payload.as_slice());

    assert!(parser.next().is_ok());
    assert!(matches!(parser.next(), Err(e) if e.is_eof()));
    Ok(())
}

#[test]
fn simple_compound() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("object")
        .tag(Tag::Byte)
        .name("some byte")
        .byte_payload(123)
        .tag(Tag::End)
        .build();

    let mut parser = Parser::new(payload.as_slice());
    assert_eq!(parser.next()?, Value::Compound(name("object")));
    assert_eq!(parser.next()?, Value::Byte(name("some byte"), 123));
    assert_eq!(parser.next()?, Value::CompoundEnd);
    Ok(())
}

#[test]
fn simple_list() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::List)
        .name("list")
        .tag(Tag::Byte) // type of each element
        .int_payload(3) // number of elements
        .byte_payload(1)
        .byte_payload(2)
        .byte_payload(3)
        .build();

    let mut parser = Parser::new(payload.as_slice());
    assert_eq!(parser.next()?, Value::List(name("list"), Tag::Byte, 3));
    assert_eq!(parser.next()?, Value::Byte(None, 1));
    assert_eq!(parser.next()?, Value::Byte(None, 2));
    assert_eq!(parser.next()?, Value::Byte(None, 3));
    assert_eq!(parser.next()?, Value::ListEnd);
    assert!(matches!(parser.next(), Err(e) if e.is_eof()));
    Ok(())
}

#[test]
fn simple_list_of_int() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::List)
        .name("list")
        .tag(Tag::Int) // type of each element
        .int_payload(3) // number of elements
        .int_payload(1)
        .int_payload(2)
        .int_payload(3)
        .build();

    let mut parser = Parser::new(payload.as_slice());
    assert_eq!(parser.next()?, Value::List(name("list"), Tag::Int, 3));
    assert_eq!(parser.next()?, Value::Int(None, 1));
    assert_eq!(parser.next()?, Value::Int(None, 2));
    assert_eq!(parser.next()?, Value::Int(None, 3));
    assert_eq!(parser.next()?, Value::ListEnd);
    assert!(matches!(parser.next(), Err(e) if e.is_eof()));
    Ok(())
}

#[test]
fn two_lists_one_after_another() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::List)
        .name("ints")
        .tag(Tag::Int) // type of each element
        .int_payload(3) // number of elements
        .int_payload(1)
        .int_payload(2)
        .int_payload(3)
        .tag(Tag::List)
        .name("bytes")
        .tag(Tag::Byte) // type of each element
        .int_payload(3) // number of elements
        .byte_payload(1)
        .byte_payload(2)
        .byte_payload(3)
        .build();

    let mut parser = Parser::new(payload.as_slice());
    assert_eq!(parser.next()?, Value::List(name("ints"), Tag::Int, 3));
    assert_eq!(parser.next()?, Value::Int(None, 1));
    assert_eq!(parser.next()?, Value::Int(None, 2));
    assert_eq!(parser.next()?, Value::Int(None, 3));
    assert_eq!(parser.next()?, Value::ListEnd);
    assert_eq!(parser.next()?, Value::List(name("bytes"), Tag::Byte, 3));
    assert_eq!(parser.next()?, Value::Byte(None, 1));
    assert_eq!(parser.next()?, Value::Byte(None, 2));
    assert_eq!(parser.next()?, Value::Byte(None, 3));
    assert_eq!(parser.next()?, Value::ListEnd);
    assert!(matches!(parser.next(), Err(e) if e.is_eof()));
    Ok(())
}

#[test]
fn compound_with_list_inside() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("object")
        .tag(Tag::List)
        .name("ints")
        .tag(Tag::Int) // type of each element
        .int_payload(3) // number of elements
        .int_payload(1)
        .int_payload(2)
        .int_payload(3)
        .tag(Tag::End)
        .build();

    let mut parser = Parser::new(payload.as_slice());
    assert_eq!(parser.next()?, Value::Compound(name("object")));
    assert_eq!(parser.next()?, Value::List(name("ints"), Tag::Int, 3));
    assert_eq!(parser.next()?, Value::Int(None, 1));
    assert_eq!(parser.next()?, Value::Int(None, 2));
    assert_eq!(parser.next()?, Value::Int(None, 3));
    assert_eq!(parser.next()?, Value::ListEnd);
    assert_eq!(parser.next()?, Value::CompoundEnd);
    assert!(matches!(parser.next(), Err(e) if e.is_eof()));
    Ok(())
}

#[test]
fn nested_compound() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("outer")
        .tag(Tag::Compound)
        .name("inner")
        .tag(Tag::Byte)
        .name("somebyte")
        .byte_payload(123)
        .tag(Tag::End)
        .tag(Tag::Byte)
        .name("extra")
        .byte_payload(3)
        .tag(Tag::End)
        .build();

    let mut parser = Parser::new(payload.as_slice());
    assert_eq!(parser.next()?, Value::Compound(name("outer")));
    assert_eq!(parser.next()?, Value::Compound(name("inner")));
    assert_eq!(parser.next()?, Value::Byte(name("somebyte"), 123));
    assert_eq!(parser.next()?, Value::CompoundEnd);
    assert_eq!(parser.next()?, Value::Byte(name("extra"), 3));
    assert_eq!(parser.next()?, Value::CompoundEnd);
    assert!(matches!(parser.next(), Err(e) if e.is_eof()));
    Ok(())
}

#[test]
fn list_of_compounds() -> Result<()> {
    let payload = Builder::new()
        .tag(Tag::List)
        .name("things")
        .tag(Tag::Compound)
        .int_payload(3)
        .tag(Tag::End)
        .tag(Tag::End)
        .tag(Tag::End)
        .build();

    let mut parser = Parser::new(payload.as_slice());
    assert_eq!(
        parser.next()?,
        Value::List(name("things"), Tag::Compound, 3)
    );
    assert_eq!(parser.next()?, Value::Compound(None));
    assert_eq!(parser.next()?, Value::CompoundEnd);
    assert_eq!(parser.next()?, Value::Compound(None));
    assert_eq!(parser.next()?, Value::CompoundEnd);
    assert_eq!(parser.next()?, Value::Compound(None));
    assert_eq!(parser.next()?, Value::CompoundEnd);
    assert_eq!(parser.next()?, Value::ListEnd);
    assert!(matches!(parser.next(), Err(e) if e.is_eof()));
    Ok(())
}
