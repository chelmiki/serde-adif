# Serde ADIF

ADIF (Amateur Data Interchange Format) is an open standard used to exchange ham radio log data between software and websites. It uses text-based records, for example a QLog export:

```text
### QLog ADIF Export
<ADIF_VER:5>3.1.4
<PROGRAMID:4>QLog
<PROGRAMVERSION:6>0.40.1
<CREATED_TIMESTAMP:15>20260417 170249
<EOH>

<call:7>EI0IRTS
<freq:7:N>14.0757
<mode:3>FT8
<tx_pwr:2>20
<qso_date:8:D>20240120
<eor>

<call:6>EI4JKB
<freq:6:N>7.0749
<mode:3>FT8
<tx_pwr:2>20
<qso_date:8:D>20240121
<eor>
```

`serde-adif` is a real `serde::Serialize`/`Deserialize` backend for ADIF, so it works directly with `#[derive(Serialize, Deserialize)]` on your own structs, the same way `serde_json` or `toml` do for their formats. It doesn't impose any particular shape for a QSO - you define whichever fields you care about, and `serde-adif` maps them to and from ADIF's `<FIELD:LENGTH:TYPE>` tags.

`serde-adif` does syntax checking but does not check or interpret ADIF semantics. It knows about basic types: strings, numbers, booleans, chars and sequences but doesn't know or care about higher level constructs like GridSquare (whether it is a valid 2, 4 or a 6 characters grid square) or dates; these are simply parsed as text and left up to the application to do the conversion and validation of those types. Similarly, `serde-adif` does not check whether a record's field names are recognized ADIF fields, or custom field names declared via USERDEFn header fields or prefixed with APP_. An application using `serde-adif` to serialize a struct containing custom fields that are not APP_-prefixed is responsible for adding its corresponding USERDEFn to the ADIF header.

## Reading and writing ADIF files

```rust
use serde_derive::{Deserialize, Serialize};
use serde_adif::Adif;

#[derive(Deserialize, Serialize)]
struct Header {}

#[derive(Deserialize, Serialize)]
struct Qso {
    call: String,
    freq: f32,
    mode: String,
    tx_pwr: Option<u16>,
    qso_date: String,
}

// Reading
let data = std::fs::read_to_string("log.adi").unwrap();
let adif = serde_adif::from_str::<Header, Qso>(&data).unwrap();

// Writing
let adif_text = serde_adif::to_string(&adif).unwrap();
std::fs::write("log.adi", adif_text).unwrap();
```

## Architecture

`de.rs` uses a dedicated type per structural level of ADIF, rather than one
type reused recursively at every depth:

```
AdifDeserializer   (Deserializer) - whole file: an optional header, then a sequence of records
 ├─ RecordDeserializer    (Deserializer) - the header, via deserialize_header()
 │   └─ RecordFields          (MapAccess)  - iterates header fields, stops at <EOH>
 └─ RecordSeq             (SeqAccess) - iterates every record
     └─ RecordDeserializer    (Deserializer) - one record, via deserialize_record()
         └─ RecordFields          (MapAccess)  - iterates record fields, stops at <EOR>
             └─ ValueDeserializer     (Deserializer) - one field's raw value
                 └─ CommaSeparated        (SeqAccess)  - items of one list-valued field
```

This mirrors ADIF's own grammar, which has exactly these three fixed levels
(file -> records -> fields) and no way to nest a record inside a field's
value. A header and a record are structurally identical at the field level,
a sequence of `<FIELD:LENGTH:TYPE>` tags terminated by a closing tag, so
`RecordDeserializer`/`RecordFields` are shared between both instead of
duplicated, differing only in which closing tag ends them (`<EOH>` vs
`<EOR>`).

`ser.rs` follows the same reasoning but needs fewer types, since a
`Serializer` can implement its matching `Serialize*` trait (`SerializeSeq`,
`SerializeStruct`, ...) on itself instead of delegating to a separate
iterator type the way `SeqAccess`/`MapAccess` do on the deserialize side:

```
AdifSerializer      (Serializer + SerializeSeq) - whole file, writes an optional header, then iterates records
 ├─ RecordSerializer (Serializer + SerializeStruct) - the header, via RecordSerializer::new_header()
 └─ RecordSerializer (Serializer + SerializeStruct) - one record, via RecordSerializer::new_record()
     └─ ValueSerializer (Serializer + SerializeSeq/SerializeTuple/SerializeTupleStruct) - one field's value, iterates list items
```

Same applies here, one `RecordSerializer` type serves both roles,
distinguished only by the terminator.

The public entry points, `from_str`/`to_string`, operate on `Adif<H, R>`, a
struct pairing an optional header (`H`) with a sequence of records (`R`).

## References

- ADIF Data Types https://adif.org/314/ADIF_314.htm
- SERDE https://serde.rs/

