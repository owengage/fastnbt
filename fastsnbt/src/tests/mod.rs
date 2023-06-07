use fastnbt::IntArray;
use serde::{Serialize, Deserialize};

use crate::{to_string, from_str};

mod de_tests;
mod ser_tests;

#[test]
fn test_mixed() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct MixedStruct {
        name: String,
        ints: IntArray,
        f: f64,
        collection: Vec<bool>,
    }

    let data = MixedStruct {
        name: "Cool \"name\"".into(),
        ints: IntArray::new(vec![-1, 3, 2000]),
        f: -5.0e-40,
        collection: vec![true, false, true, true]
    };
    let serialized = to_string(&data).unwrap();
    assert_eq!("{\"name\":\"Cool \\\"name\\\"\",\"ints\":[I;-1,3,2000],\"f\":-5e-40,\"collection\":[true,false,true,true]}", serialized);

    let deserialized: MixedStruct = from_str(&serialized).unwrap();
    assert_eq!(deserialized, data);
}
