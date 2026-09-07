use serde_derive::Serialize;

#[test]
fn test_serialize_simple_types() {
    #[derive(Serialize)]
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

    let test = Record {
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
    let records = vec![test];
    let expected =
        "<index:1>1<callsign:6>EI4JKB<freq:6>14.074<dbm:3>-10<bw:1>5<synced:1>Y<qsl_recv:1>N<class:1>A<mode:3>FT8<EOR>\n";
    assert_eq!(serde_adif::to_string(&records).unwrap(), expected);
}

#[test]
fn test_serialize_option() {
    #[derive(Serialize)]
    struct Record {
        freq: Option<f32>,
        dbm: Option<i16>,
    }

    let missing_field = Record {
        freq: None,
        dbm: Some(-10),
    };

    let records = vec![missing_field];
    let expected = "<dbm:3>-10<EOR>\n";
    assert_eq!(serde_adif::to_string(&records).unwrap(), expected);

    let empty_record = Record {
        freq: None,
        dbm: None,
    };

    let records = vec![empty_record];
    let expected = "<EOR>\n";
    assert_eq!(serde_adif::to_string(&records).unwrap(), expected);
}

#[test]
fn test_serialize_field_value_sequences() {
    // Field values that expand to a comma-separated sequence come in four
    // shapes: Vec<T>, a fixed-size array [T; N], a plain tuple (T, S), and a
    // tuple struct with more than one field (a single-field tuple struct is
    // a newtype, tested separately) - this test covers all four.

    #[derive(Serialize)]
    struct Spot(String, f32);

    #[derive(Serialize)]
    struct Record {
        index: u32,
        awards: Vec<u16>,
        rst: [u8; 3],
        exchange: (char, String),
        spot: Spot,
    }

    let test1 = Record {
        index: 1,
        awards: vec![1, 2, 3],
        rst: [5, 9, 9],
        exchange: ('A', "DX".to_string()),
        spot: Spot("EI4JKB".to_string(), 144.0),
    };

    let records = vec![test1];
    let expected =
        "<index:1>1<awards:5>1,2,3<rst:5>5,9,9<exchange:4>A,DX<spot:10>EI4JKB,144<EOR>\n";
    assert_eq!(serde_adif::to_string(&records).unwrap(), expected);

    // Only Vec<T> can be empty - an array's size is fixed by its type, and a
    // tuple/tuple struct's too, so neither has an "empty" instance to test here.
    let test2 = Record {
        index: 1,
        awards: vec![],
        rst: [5, 9, 9],
        exchange: ('A', "DX".to_string()),
        spot: Spot("EI4JKB".to_string(), 144.0),
    };

    let records = vec![test2];
    let expected = "<index:1>1<awards:0><rst:5>5,9,9<exchange:4>A,DX<spot:10>EI4JKB,144<EOR>\n";
    assert_eq!(serde_adif::to_string(&records).unwrap(), expected);
}

#[test]
fn test_serialize_newtype() {
    #[derive(Serialize)]
    struct Gridsquare(String);

    #[derive(Serialize)]
    struct Record {
        index: u32,
        gridsquare: Gridsquare,
    }

    let test = Record {
        index: 1,
        gridsquare: Gridsquare("IO63qh".to_string()),
    };

    let records = vec![test];
    let expected = "<index:1>1<gridsquare:6>IO63qh<EOR>\n";
    assert_eq!(serde_adif::to_string(&records).unwrap(), expected);
}

#[test]
fn test_serialize_top_level_newtype() {
    // A newtype struct wrapping the top-level Vec<Record> is a transparent
    // wrapper - it serializes identically to the Vec<Record> it contains.
    #[derive(Serialize)]
    struct Record {
        index: u32,
    }

    #[derive(Serialize)]
    struct Log(Vec<Record>);

    let log = Log(vec![Record { index: 1 }, Record { index: 2 }]);
    let expected = "<index:1>1<EOR>\n<index:1>2<EOR>\n";
    assert_eq!(serde_adif::to_string(&log).unwrap(), expected);
}

#[test]
fn test_serialize_top_level_sequences() {
    // The top level is a list of ADIF records, each record separated from
    // the next by its <EOR> tag followed by a newline.
    #[derive(Serialize)]
    struct Record {
        index: u32,
        callsign: String,
    }

    let test1 = Record {
        index: 1,
        callsign: "EI4JKB".to_string(),
    };
    let test2 = Record {
        index: 2,
        callsign: "AAAA".to_string(),
    };
    let test3 = Record {
        index: 3,
        callsign: "BBB".to_string(),
    };

    let records = vec![test1, test2, test3];
    let expected = "<index:1>1<callsign:6>EI4JKB<EOR>\n<index:1>2<callsign:4>AAAA<EOR>\n<index:1>3<callsign:3>BBB<EOR>\n";
    assert_eq!(serde_adif::to_string(&records).unwrap(), expected);
}

#[test]
fn test_serialize_enum() {
    #[derive(Serialize)]
    enum Mode {
        Ssb,
    }

    #[derive(Serialize)]
    struct Record {
        index: u32,
        mode: Mode,
    }

    let test = Record {
        index: 1,
        mode: Mode::Ssb,
    };

    let records = vec![test];
    let expected = "<index:1>1<mode:3>Ssb<EOR>\n";
    assert_eq!(serde_adif::to_string(&records).unwrap(), expected);
}

#[test]
fn test_serialize_invalid_field_type() {
    // ADIF field values do not support nested structs
    #[derive(Serialize)]
    struct Contest {
        name: String,
        date: String,
    }

    #[derive(Serialize)]
    struct Record {
        index: u32,
        contest: Contest,
    }

    let test = Record {
        index: 1,
        contest: Contest {
            name: "test".to_string(),
            date: "1970/01/01".to_string(),
        },
    };

    let records = vec![test];
    let error = serde_adif::to_string(&records).unwrap_err();
    assert_eq!(error.to_string(), "Unsupported type struct");
}

#[test]
fn test_serialize_invalid_top_level_type() {
    // ADIF is a sequence of Records, so at the top level
    // a sequence is the only valid type.

    #[derive(Serialize)]
    struct Record {
        index: u32,
    }

    let record = Record { index: 1 };

    let error = serde_adif::to_string(&record).unwrap_err();
    assert_eq!(error.to_string(), "Unsupported type struct");
}

#[test]
fn test_serialize_invalid_record_type() {
    // ADIF is a sequence of Records, each record is a struct/map.
    // The only valid type within the sequence is a struct.

    let error = serde_adif::to_string(&vec![1, 2]).unwrap_err();
    assert_eq!(error.to_string(), "Unsupported type i32");
}
