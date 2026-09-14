use serde_adif::Adif;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct NoHeader {}

#[test]
fn test_roundtrip_simple_types() {
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
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

    let record = Record {
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
    let adif = Adif {
        header: None,
        records: vec![record],
    };
    assert_eq!(
        serde_adif::from_str::<NoHeader, Record>(&serde_adif::to_string(&adif).unwrap()).unwrap(),
        adif
    );
}

#[test]
fn test_roundtrip_option() {
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Record {
        freq: Option<f32>,
        dbm: Option<i16>,
    }

    let missing_field = Record {
        freq: None,
        dbm: Some(-10),
    };

    let adif = Adif {
        header: None,
        records: vec![missing_field],
    };

    assert_eq!(
        serde_adif::from_str::<NoHeader, Record>(&serde_adif::to_string(&adif).unwrap()).unwrap(),
        adif
    );

    let empty_record = Record {
        freq: None,
        dbm: None,
    };

    let adif = Adif {
        header: None,
        records: vec![empty_record],
    };

    assert_eq!(
        serde_adif::from_str::<NoHeader, Record>(&serde_adif::to_string(&adif).unwrap()).unwrap(),
        adif
    );
}

#[test]
fn test_roundtrip_field_value_sequences() {
    // Field values that expand to a comma-separated sequence come in four
    // shapes: Vec<T>, a fixed-size array [T; N], a plain tuple (T, S), and a
    // tuple struct with more than one field (a single-field tuple struct is
    // a newtype, tested separately) - this test covers all four.

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Spot(String, f32);

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Record {
        index: u32,
        awards: Vec<u16>,
        rst: [u8; 3],
        exchange: (char, String),
        spot: Spot,
    }

    let record_1 = Record {
        index: 1,
        awards: vec![1, 2, 3],
        rst: [5, 9, 9],
        exchange: ('A', "DX".to_string()),
        spot: Spot("EI4JKB".to_string(), 144.0),
    };

    let adif = Adif {
        header: None,
        records: vec![record_1],
    };

    assert_eq!(
        serde_adif::from_str::<NoHeader, Record>(&serde_adif::to_string(&adif).unwrap()).unwrap(),
        adif
    );

    // Only Vec<T> can be empty - an array's size is fixed by its type, and a
    // tuple/tuple struct's too, so neither has an "empty" instance to test here.
    let record_2 = Record {
        index: 1,
        awards: vec![],
        rst: [5, 9, 9],
        exchange: ('A', "DX".to_string()),
        spot: Spot("EI4JKB".to_string(), 144.0),
    };

    let adif = Adif {
        header: None,
        records: vec![record_2],
    };

    assert_eq!(
        serde_adif::from_str::<NoHeader, Record>(&serde_adif::to_string(&adif).unwrap()).unwrap(),
        adif
    );
}

#[test]
fn test_roundtrip_newtype() {
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Gridsquare(String);

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Record {
        index: u32,
        gridsquare: Gridsquare,
    }

    let record = Record {
        index: 1,
        gridsquare: Gridsquare("IO63qh".to_string()),
    };

    let adif = Adif {
        header: None,
        records: vec![record],
    };

    assert_eq!(
        serde_adif::from_str::<NoHeader, Record>(&serde_adif::to_string(&adif).unwrap()).unwrap(),
        adif
    );
}

#[test]
fn test_roundtrip_top_level_sequences() {
    // The top level is a list of ADIF records, each record separated from
    // the next by its <EOR> tag followed by a newline.
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Record {
        index: u32,
        callsign: String,
    }

    let record_1 = Record {
        index: 1,
        callsign: "EI4JKB".to_string(),
    };
    let record_2 = Record {
        index: 2,
        callsign: "AAAA".to_string(),
    };
    let record_3 = Record {
        index: 3,
        callsign: "BBB".to_string(),
    };

    let adif = Adif {
        header: None,
        records: vec![record_1, record_2, record_3],
    };

    assert_eq!(
        serde_adif::from_str::<NoHeader, Record>(&serde_adif::to_string(&adif).unwrap()).unwrap(),
        adif
    );
}

#[test]
fn test_roundtrip_enum() {
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    enum Mode {
        Ssb,
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Record {
        index: u32,
        mode: Mode,
    }

    let record = Record {
        index: 1,
        mode: Mode::Ssb,
    };

    let adif = Adif {
        header: None,
        records: vec![record],
    };

    assert_eq!(
        serde_adif::from_str::<NoHeader, Record>(&serde_adif::to_string(&adif).unwrap()).unwrap(),
        adif
    );
}

#[test]
fn test_roundtrip_with_header() {
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Header {
        adif_ver: String,
        programid: String,
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct Record {
        index: u32,
        callsign: String,
    }

    let adif = Adif {
        header: Some(Header {
            adif_ver: "3.1.4".to_string(),
            programid: "eirlog".to_string(),
        }),
        records: vec![Record {
            index: 1,
            callsign: "EI4JKB".to_string(),
        }],
    };
    assert_eq!(
        serde_adif::from_str(&serde_adif::to_string(&adif).unwrap()).unwrap(),
        adif
    );
}
