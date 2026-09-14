//! Serialize a Rust data structure into ADIF data

use crate::{
    error::{Error, Result},
    Adif, EOH, EOR,
};
use serde::ser::{self, Serialize};
use std::fmt::Write;

macro_rules! unsupported_serialize {
    ($($method:ident($($argty:ty),*) -> $ret:ty => $label:expr),* $(,)?) => {
        $(
            fn $method(self, $(_: $argty),*) -> Result<$ret> {
                Err(Error::Unsupported($label))
            }
        )*
    };
}

struct AdifSerializer {
    output: String,
}

impl AdifSerializer {
    fn new() -> Self {
        Self {
            output: String::new(),
        }
    }
}

/// Serializes an `Adif<H, R>` into an ADIF document.
///
/// The top level of an ADIF document always has the same shape: an
/// optional header followed by a sequence of records. `H` and `R` are
/// typically structs representing the header and each record
/// respectively. If `header` is always `None`, `()` works for `H`. It's
/// never actually serialized, since `H`'s `Serialize` impl is only
/// invoked when `header` is `Some(...)`.
pub fn to_string<H, R>(value: &Adif<H, R>) -> Result<String>
where
    H: Serialize,
    R: Serialize,
{
    let mut serializer = AdifSerializer::new();

    if let Some(header) = &value.header {
        let mut header_serializer = RecordSerializer::new_header();
        header.serialize(&mut header_serializer)?;

        // An ADIF file must start with something different than '<'
        // when there is a header
        serializer.output += "# ADIF Header\n";

        serializer.output += &header_serializer.finish();
        serializer.output += "\n";
    }
    value.records.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl ser::Serializer for &mut AdifSerializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Unsupported("newtype variant"))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Unsupported("some"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    unsupported_serialize! {
        serialize_bool(bool) -> Self::Ok => "bool",
        serialize_i64(i64) -> Self::Ok => "i64",
        serialize_i32(i32) -> Self::Ok => "i32",
        serialize_i16(i16) -> Self::Ok => "i16",
        serialize_i8(i8) -> Self::Ok => "i8",
        serialize_u64(u64) -> Self::Ok => "u64",
        serialize_u32(u32) -> Self::Ok => "u32",
        serialize_u16(u16) -> Self::Ok => "u16",
        serialize_u8(u8) -> Self::Ok => "u8",
        serialize_f64(f64) -> Self::Ok => "f64",
        serialize_f32(f32) -> Self::Ok => "f32",
        serialize_str(&str) -> Self::Ok => "str",
        serialize_char(char) -> Self::Ok => "char",
        serialize_bytes(&[u8]) -> Self::Ok => "bytes",
        serialize_none() -> Self::Ok => "none",
        serialize_unit() -> Self::Ok => "unit",
        serialize_unit_struct(&'static str) -> Self::Ok => "unit struct",
        serialize_unit_variant(&'static str, u32, &'static str) -> Self::Ok => "unit variant",
        serialize_struct(&'static str, usize) -> Self::SerializeStruct => "struct",
        serialize_struct_variant(&'static str, u32, &'static str, usize) -> Self::SerializeStructVariant => "struct variant",
        serialize_tuple(usize) -> Self::SerializeTuple => "tuple",
        serialize_tuple_struct(&'static str, usize) -> Self::SerializeTupleStruct => "tuple struct",
        serialize_tuple_variant(&'static str, u32, &'static str, usize) -> Self::SerializeTupleVariant => "tuple variant",
        serialize_map(Option<usize>) -> Self::SerializeMap => "map",
    }
}

impl ser::SerializeSeq for &mut AdifSerializer {
    type Ok = ();
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut serializer = RecordSerializer::new_record();
        value.serialize(&mut serializer)?;
        self.output += &serializer.finish();
        self.output += "\n";
        Ok(())
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        Ok(())
    }
}

struct RecordSerializer {
    output: String,
    terminator: &'static str,
}

impl RecordSerializer {
    fn new_record() -> Self {
        RecordSerializer {
            output: String::new(),
            terminator: EOR,
        }
    }

    fn new_header() -> Self {
        RecordSerializer {
            output: String::new(),
            terminator: EOH,
        }
    }

    fn finish(self) -> String {
        self.output
    }
}

impl ser::Serializer for &mut RecordSerializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Self;
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_some<T>(self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Unsupported("some"))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Unsupported("newtype struct"))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Unsupported("newtype variant"))
    }

    unsupported_serialize! {
        serialize_bool(bool) -> Self::Ok => "bool",
        serialize_i64(i64) -> Self::Ok => "i64",
        serialize_i32(i32) -> Self::Ok => "i32",
        serialize_i16(i16) -> Self::Ok => "i16",
        serialize_i8(i8) -> Self::Ok => "i8",
        serialize_u64(u64) -> Self::Ok => "u64",
        serialize_u32(u32) -> Self::Ok => "u32",
        serialize_u16(u16) -> Self::Ok => "u16",
        serialize_u8(u8) -> Self::Ok => "u8",
        serialize_f64(f64) -> Self::Ok => "f64",
        serialize_f32(f32) -> Self::Ok => "f32",
        serialize_str(&str) -> Self::Ok => "str",
        serialize_char(char) -> Self::Ok => "char",
        serialize_bytes(&[u8]) -> Self::Ok => "bytes",
        serialize_none() -> Self::Ok => "none",
        serialize_unit() -> Self::Ok => "unit",
        serialize_unit_struct(&'static str) -> Self::Ok => "unit struct",
        serialize_unit_variant(&'static str, u32, &'static str) -> Self::Ok => "unit variant",
        serialize_struct_variant(&'static str, u32, &'static str, usize) -> Self::SerializeStructVariant => "struct variant",
        serialize_tuple(usize) -> Self::SerializeTuple => "tuple",
        serialize_tuple_struct(&'static str, usize) -> Self::SerializeTupleStruct => "tuple struct",
        serialize_tuple_variant(&'static str, u32, &'static str, usize) -> Self::SerializeTupleVariant => "tuple variant",
        serialize_map(Option<usize>) -> Self::SerializeMap => "map",
        serialize_seq(Option<usize>) -> Self::SerializeSeq => "seq",
    }
}

impl ser::SerializeStruct for &mut RecordSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut serializer = ValueSerializer::new();
        value.serialize(&mut serializer)?;

        // A None field is omitted entirely, not written as an empty or
        // placeholder value
        if serializer.is_none {
            return Ok(());
        }

        let value_str = serializer.finish();
        let len = value_str.len();

        // Write in the format <key:len>value
        write!(&mut self.output, "<{}:{}>{}", key, len, value_str)
            .map_err(|err| serde::ser::Error::custom(err.to_string()))
    }

    fn end(self) -> Result<()> {
        self.output += self.terminator;
        Ok(())
    }
}

struct ValueSerializer {
    output: String,
    is_none: bool,
}

impl ValueSerializer {
    fn new() -> Self {
        ValueSerializer {
            output: String::new(),
            is_none: false,
        }
    }

    fn finish(self) -> String {
        self.output
    }
}

impl ser::Serializer for &mut ValueSerializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output += if v { "Y" } else { "N" };
        Ok(())
    }

    // ADIF does not distinguish between different sizes of integers, all
    // signed and unsigned integers will be serialized as text
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    // Serialize a char as a single-character string.
    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output += v;
        Ok(())
    }

    // ADIF has no representation for an absent value - a field is either
    // present with data, or it's omitted from the record entirely.
    fn serialize_none(self) -> Result<()> {
        self.is_none = true;
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.is_none = true;
        Ok(())
    }

    // Since ADIF is a human readable format we serialize enums using their name
    // rather than their index
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    // newtype structs are treated as wrappers and only their content is
    // serialized
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Unsupported("newtype variant"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    unsupported_serialize! {
        serialize_bytes(&[u8]) -> Self::Ok => "bytes",
        serialize_unit_struct(&'static str) -> Self::Ok => "unit struct",
        serialize_struct(&'static str, usize) -> Self::SerializeStruct => "struct",
        serialize_struct_variant(&'static str, u32, &'static str, usize) -> Self::SerializeStructVariant => "struct variant",
        serialize_tuple_variant(&'static str, u32, &'static str, usize) -> Self::SerializeTupleVariant => "tuple variant",
        serialize_map(Option<usize>) -> Self::SerializeMap => "map",
    }
}

impl ser::SerializeSeq for &mut ValueSerializer {
    type Ok = ();
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.is_empty() {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeTuple for &mut ValueSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for &mut ValueSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}
