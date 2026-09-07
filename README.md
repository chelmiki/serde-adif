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

## Reading and writing ADIF files

```rust
use serde_derive::{Deserialize, Serialize};

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
let qsos: Vec<Qso> = serde_adif::from_str(&data).unwrap();

// Writing
let adif = serde_adif::to_string(&qsos).unwrap();
std::fs::write("log.adi", adif).unwrap();
```

## Architecture

`de.rs` uses a dedicated type per structural level of ADIF, rather than one
type reused recursively at every depth:

```
AdifDeserializer   (Deserializer) - whole file
 └─ RecordSeq         (SeqAccess) - iterates every record
     └─ RecordDeserializer (Deserializer) - one record, not yet parsed
         └─ RecordFields   (MapAccess)    - iterates that record's fields
             └─ ValueDeserializer (Deserializer) - one field's raw value
                 └─ CommaSeparated (SeqAccess)    - items of one list-valued field
```

This mirrors ADIF's own grammar, which has exactly these three fixed levels
(file -> records -> fields) and no way to nest a record inside a field's
value.

`ser.rs` follows the same reasoning but needs fewer types, since a
`Serializer` can implement its matching `Serialize*` trait (`SerializeSeq`,
`SerializeStruct`, ...) on itself instead of delegating to a separate
iterator type the way `SeqAccess`/`MapAccess` do on the deserialize side:

```
AdifSerializer      (Serializer + SerializeSeq) - whole file, iterates records
 └─ RecordSerializer (Serializer + SerializeStruct) - one record, iterates fields
     └─ ValueSerializer (Serializer + SerializeSeq/SerializeTuple/SerializeTupleStruct) - one field's value, iterates list items
```

## References

- ADIF Data Types https://adif.org/314/ADIF_314.htm
- SERDE https://serde.rs/

