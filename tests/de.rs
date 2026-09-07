use serde_derive::Deserialize;

#[test]
fn test_deserialize_simple_types() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Record<'a> {
        index: u32,
        callsign: String,
        freq: f32,
        dbm: i16,
        bw: f32,
        synced: bool,
        qsl_recv: bool,
        class: char,
        mode: &'a str,
    }

    let expected_record = Record {
        index: 1,
        callsign: "EI4JKB".to_string(),
        freq: 14.074,
        dbm: -10,
        bw: 5.0,
        synced: true,
        qsl_recv: false,
        class: 'A',
        mode: "FT8",
    };
    let expected = vec![expected_record];
    let test =
        "<index:1>1<callsign:6>EI4JKB<freq:6>14.074<dbm:3>-10<bw:1>5<synced:1>Y<qsl_recv:1>N<class:1>A<mode:3>FT8<EOR>\n";
    assert_eq!(serde_adif::from_str::<Vec<Record>>(test).unwrap(), expected);
}

#[test]
fn test_deserialize_option() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        freq: Option<f32>,
        dbm: Option<i16>,
    }

    let missing_field = Record {
        freq: None,
        dbm: Some(-10),
    };

    let expected = vec![missing_field];
    let test = "<dbm:3>-10<EOR>\n";
    assert_eq!(serde_adif::from_str::<Vec<Record>>(test).unwrap(), expected);

    let empty_record = Record {
        freq: None,
        dbm: None,
    };

    let expected = vec![empty_record];
    let test = "<EOR>\n";
    assert_eq!(serde_adif::from_str::<Vec<Record>>(test).unwrap(), expected);
}

#[test]
fn test_deserialize_field_value_sequences() {
    // Field values that expand to a comma-separated sequence come in four
    // shapes: Vec<T>, a fixed-size array [T; N], a plain tuple (T, S), and a
    // tuple struct with more than one field (a single-field tuple struct is
    // a newtype, tested separately) - this test covers all four.

    #[derive(Deserialize, Debug, PartialEq)]
    struct Spot(String, f32);

    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        index: u32,
        awards: Vec<u16>,
        rst: [u8; 3],
        exchange: (char, String),
        spot: Spot,
    }

    let expected_record_1 = Record {
        index: 1,
        awards: vec![1, 2, 3],
        rst: [5, 9, 9],
        exchange: ('A', "DX".to_string()),
        spot: Spot("EI4JKB".to_string(), 144.0),
    };

    let expected = vec![expected_record_1];
    let test = "<index:1>1<awards:5>1,2,3<rst:5>5,9,9<exchange:4>A,DX<spot:10>EI4JKB,144<EOR>\n";
    assert_eq!(serde_adif::from_str::<Vec<Record>>(test).unwrap(), expected);

    // Only Vec<T> can be empty - an array's size is fixed by its type, and a
    // tuple/tuple struct's too, so neither has an "empty" instance to test here.
    let expected_record_2 = Record {
        index: 1,
        awards: vec![],
        rst: [5, 9, 9],
        exchange: ('A', "DX".to_string()),
        spot: Spot("EI4JKB".to_string(), 144.0),
    };

    let expected = vec![expected_record_2];
    let test = "<index:1>1<awards:0><rst:5>5,9,9<exchange:4>A,DX<spot:10>EI4JKB,144<EOR>\n";
    assert_eq!(serde_adif::from_str::<Vec<Record>>(test).unwrap(), expected);
}

#[test]
fn test_deserialize_newtype() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Gridsquare(String);

    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        index: u32,
        gridsquare: Gridsquare,
    }

    let expected_record = Record {
        index: 1,
        gridsquare: Gridsquare("IO63qh".to_string()),
    };

    let expected = vec![expected_record];
    let test = "<index:1>1<gridsquare:6>IO63qh<EOR>\n";
    assert_eq!(serde_adif::from_str::<Vec<Record>>(test).unwrap(), expected);
}

#[test]
fn test_deserialize_top_level_newtype() {
    // A newtype struct wrapping the top-level Vec<Record> is a transparent
    // wrapper - it deserializes identically to a plain Vec<Record>.
    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        index: u32,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Log(Vec<Record>);

    let test = "<index:1>1<EOR>\n<index:1>2<EOR>\n";
    let expected = Log(vec![Record { index: 1 }, Record { index: 2 }]);
    assert_eq!(serde_adif::from_str::<Log>(test).unwrap(), expected);
}

#[test]
fn test_deserialize_top_level_sequences() {
    // The top level is a list of ADIF records, each record separated from
    // the next by its <EOR> tag followed by a newline.
    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        index: u32,
        callsign: String,
    }

    let expected_record_1 = Record {
        index: 1,
        callsign: "EI4JKB".to_string(),
    };
    let expected_record_2 = Record {
        index: 2,
        callsign: "AAAA".to_string(),
    };
    let expected_record_3 = Record {
        index: 3,
        callsign: "BBB".to_string(),
    };

    let expected = vec![expected_record_1, expected_record_2, expected_record_3];
    let test = "<index:1>1<callsign:6>EI4JKB<EOR>\n<index:1>2<callsign:4>AAAA<EOR>\n<index:1>3<callsign:3>BBB<EOR>\n";
    assert_eq!(serde_adif::from_str::<Vec<Record>>(test).unwrap(), expected);
}

#[test]
fn test_deserialize_enum() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Mode {
        Ssb,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        index: u32,
        mode: Mode,
    }

    let expected_record = Record {
        index: 1,
        mode: Mode::Ssb,
    };

    let expected = vec![expected_record];
    let test = "<index:1>1<mode:3>Ssb<EOR>\n";
    assert_eq!(serde_adif::from_str::<Vec<Record>>(test).unwrap(), expected);
}

#[test]
fn test_deserialize_sequence_whitespace() {
    // A comma-separated field value may have whitespace around each item.
    // Each item is trimmed before parsing.

    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        awards: Vec<u16>,
    }

    let expected = vec![Record {
        awards: vec![1, 2, 3],
    }];
    let test = "<awards:7>1, 2, 3<EOR>\n";
    assert_eq!(serde_adif::from_str::<Vec<Record>>(test).unwrap(), expected);
}

#[test]
fn test_deserialize_record_whitespace() {
    // Records may be separated by more than a single newline, e.g. blank
    // lines between records.
    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        index: u32,
    }

    let expected = vec![Record { index: 1 }, Record { index: 2 }];
    let test = "<index:1>1<EOR>\n\n\n<index:1>2<EOR>\n";
    assert_eq!(serde_adif::from_str::<Vec<Record>>(test).unwrap(), expected);
}

#[test]
fn test_deserialize_ignore_extra_fields() {
    // Extra ADIF fields not present on the target struct are ignored rather
    // than causing an error.

    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        index: u32,
    }

    let expected = vec![Record { index: 1 }];
    let test = "<ignore:3>abc<index:1>1<EOR>\n";
    assert_eq!(serde_adif::from_str::<Vec<Record>>(test).unwrap(), expected);
}

#[test]
fn test_deserialize_malformed_tag() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        index: u32,
    }

    let test = "<index>1<EOR>\n";
    let result = serde_adif::from_str::<Vec<Record>>(test);
    assert!(matches!(result, Err(serde_adif::Error::MalformedTag(_))))
}

#[test]
fn test_deserialize_invalid_length() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        index: u32,
    }

    let test = "<index:one>1<EOR>\n";
    let result = serde_adif::from_str::<Vec<Record>>(test);
    assert!(matches!(result, Err(serde_adif::Error::InvalidLength(_))));
}

#[test]
fn test_deserialize_invalid_number() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        freq: f64,
    }

    let test = "<freq:3>one<EOR>\n";
    let error = serde_adif::from_str::<Vec<Record>>(test).unwrap_err();
    assert!(error.to_string().contains("cannot parse \"one\" as f64"));
}

#[test]
fn test_deserialize_invalid_bool() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        qsl_recv: bool,
    }

    let test = "<qsl_recv:1>X<EOR>\n";
    let error = serde_adif::from_str::<Vec<Record>>(test).unwrap_err();
    assert_eq!(
        error.to_string(),
        "expected \"Y\" or \"N\" (case-insensitive) for a boolean, found \"X\""
    );
}

#[test]
fn test_deserialize_invalid_char() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        class: char,
    }

    let test = "<class:2>AA<EOR>\n";
    let error = serde_adif::from_str::<Vec<Record>>(test).unwrap_err();
    assert_eq!(
        error.to_string(),
        "expected a single character, found \"AA\""
    );
}

#[test]
fn test_deserialize_missing_closing_tag() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        index: u32,
    }

    let test = "<index:11\n";
    let result = serde_adif::from_str::<Vec<Record>>(test);
    assert!(matches!(
        result,
        Err(serde_adif::Error::ExpectedClosingTag(_))
    ));
}

#[test]
fn test_deserialize_missing_non_optional_fields() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Record {
        index: u32,
        callsign: String,
    }

    let test = "<index:1>1<EOR>\n";
    let error = serde_adif::from_str::<Vec<Record>>(test).unwrap_err();
    assert!(error.to_string().contains("missing field `callsign`"));
}
