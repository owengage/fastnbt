use std::collections::HashMap;

use crate::{ByteArray, IntArray, LongArray, Value};

#[test]
fn nbt() {
    assert_eq!(nbt!(1_i8), Value::Byte(1));
    assert_eq!(nbt!(1_u8), Value::Byte(1));
    assert_eq!(nbt!(1_i16), Value::Short(1));
    assert_eq!(nbt!(1_u16), Value::Short(1));
    assert_eq!(nbt!(1), Value::Int(1));
    assert_eq!(nbt!(1_u32), Value::Int(1));
    assert_eq!(nbt!(1_i64), Value::Long(1));
    assert_eq!(nbt!(1_u64), Value::Long(1));
    assert_eq!(nbt!(1_f32), Value::Float(1.0));
    assert_eq!(nbt!(1.0), Value::Double(1.0));
    assert_eq!(nbt!(true), Value::Byte(1));
    assert_eq!(nbt!(false), Value::Byte(0));

    assert_eq!(nbt!("string"), Value::String("string".to_owned()));
    assert_eq!(
        nbt!("string".to_owned()),
        Value::String("string".to_owned())
    );

    assert_eq!(nbt!([]), Value::List(vec![]));
    assert_eq!(
        nbt!([1, 3]),
        Value::List(vec![Value::Int(1), Value::Int(3)])
    );
    assert_eq!(
        nbt!([
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
            "Duis mattis massa metus, vel consequat lacus tincidunt ut.",
            "Nam in lobortis quam, vel vehicula magna.",
            "Cras massa turpis, facilisis non volutpat vitae, elementum.",
        ]),
        Value::List(vec![
            Value::String("Lorem ipsum dolor sit amet, consectetur adipiscing elit.".to_owned()),
            Value::String("Duis mattis massa metus, vel consequat lacus tincidunt ut.".to_owned()),
            Value::String("Nam in lobortis quam, vel vehicula magna.".to_owned()),
            Value::String("Cras massa turpis, facilisis non volutpat vitae, elementum.".to_owned()),
        ])
    );

    assert_eq!(nbt!({}), Value::Compound(HashMap::new()));
    assert_eq!(
        nbt!({ "key": "value" }),
        Value::Compound(HashMap::from([(
            "key".to_owned(),
            Value::String("value".to_owned())
        ),]))
    );
    assert_eq!(
        nbt!({
            "key1": "value1",
            "key2": 42,
            "key3": [4, 2],
        }),
        Value::Compound(HashMap::from([
            ("key1".to_owned(), Value::String("value1".to_owned())),
            ("key2".to_owned(), Value::Int(42)),
            (
                "key3".to_owned(),
                Value::List(vec![Value::Int(4), Value::Int(2)])
            ),
        ]))
    );

    assert_eq!(nbt!([B;]), Value::ByteArray(ByteArray::new(vec![])));
    assert_eq!(nbt!([I;]), Value::IntArray(IntArray::new(vec![])));
    assert_eq!(nbt!([L;]), Value::LongArray(LongArray::new(vec![])));
    assert_eq!(
        nbt!([B; 1, 2, 3]),
        Value::ByteArray(ByteArray::new(vec![1, 2, 3]))
    );
    assert_eq!(
        nbt!([I;1,2,3]),
        Value::IntArray(IntArray::new(vec![1, 2, 3]))
    );
    assert_eq!(
        nbt!([L; 1, 2, 3,]),
        Value::LongArray(LongArray::new(vec![1, 2, 3]))
    );
}
