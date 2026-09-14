use serde_derive::{Deserialize, Serialize};

/// Adif is the type needed to serialize to or deserialize
/// from an ADIF document. Unlike other formats (JSON) where
/// the outer type can be any of the supported JSON types,
/// serde-adif enforces Adif to be the outer type. Adif files
/// are composed of an optional ADIF header, which contains a
/// series of header key-value pairs, followed by a sequence of
/// records, each record containing a series of key-value pairs.
///
/// The user can decide on the structure of H and R. This gives
/// flexibility to define the shape of the header and the
/// records, that is, which fields are present and which are the
/// types of those fields (numbers, strings, characters, sequences,
/// boolean, newtype, etc).

// to_string/from_str entry points do not rely on the Serialize and
// Deserialize implementations of Adif; instead, they extract and
// (de)serialize the header and the sequence of records independently.
// However, deriving Serialize and Deserialize allows interoperating
// with other serde formats, e.g. serde_json::to_string(&adif)
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Adif<H, R> {
    pub header: Option<H>,
    pub records: Vec<R>,
}
