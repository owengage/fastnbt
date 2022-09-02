use fastnbt::{nbt, Value};

use crate::from_str;

use super::Builder;

#[test]
fn simple_byte() {
    let bs = Builder::new().byte("b", 123).build();
    let v: Value = from_str(std::str::from_utf8(&bs).unwrap()).unwrap();

    assert_eq!(
        nbt!({
            "b": 123u8
        }),
        v
    );
}
