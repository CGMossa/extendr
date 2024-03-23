#![cfg(feature = "serde")]
use extendr_api::deserializer::from_robj;
use extendr_api::prelude::*;
use extendr_api::serializer::to_robj;
use serde::{Deserialize, Serialize};

// Enums are mapped to named lists of lists.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Enum {
    Unit,
    Newtype(u32),
    Tuple(u32, u32),
    Struct { a: u32 },
    AnOption(Option<i32>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Test {
    int: i32,
    // FIXME: something with serialization
    // seq: Vec<&'a str>,
    option: Option<i32>,
    option_rint: Option<Rint>,
}

///
#[test]
fn test_back_to_back() -> std::result::Result<(), Box<dyn std::error::Error>> {
    test! {
        // region: enum ser-de

        let expected = Enum::Unit;
        assert_eq!(expected, from_robj(&to_robj(&expected)?)?);

        let expected = Enum::Newtype(1);
        assert_eq!(expected, from_robj(&to_robj(&expected)?)?);

        let expected = Enum::Tuple(1, 2);
        assert_eq!(expected, from_robj(&to_robj(&expected)?)?);

        let expected = Enum::Struct { a: 1 };
        assert_eq!(expected, from_robj(&to_robj(&expected)?)?);

        let expected = Enum::AnOption(Some(1));
        assert_eq!(expected, from_robj(&to_robj(&expected)?)?);

        let expected = Enum::AnOption(None);
        assert_eq!(expected, from_robj(&to_robj(&expected)?)?);

        // endregion

        // region: struct ser-de

        let test01 = Test {
            int: 1,
            // seq: vec!["a", "b"],
            option: Some(42_i32),
            option_rint: Some(Rint::new(21)),
        };
        let test02 = Test {
            int: 1,
            // seq: vec!["a", "b"],
            option: None,
            option_rint: Some(Rint::na()),
        };
        let test03 = Test {
            int: 1,
            // seq: vec!["a", "b"],
            option: None,
            option_rint: None,
        };

        let expected = test01;
        assert_eq!(expected, from_robj(&to_robj(&expected)?)?);

        let expected = test02;
        assert_eq!(expected, from_robj(&to_robj(&expected)?)?);

        let expected = test03;
        assert_eq!(expected, from_robj(&to_robj(&expected)?)?);

        // endregion
        // Ok(())
    };
    Ok(())
}
