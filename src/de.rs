//! Deserialize ADIF data to a Rust data structure.

use crate::error::{Error, Result};
use serde::de::{
    self, Deserialize, DeserializeSeed, Deserializer, IntoDeserializer, MapAccess, SeqAccess,
    Visitor,
};
use serde::forward_to_deserialize_any;

macro_rules! unsupported_deserialize {
    ($($method:ident($($argty:ty),*) => $label:expr),* $(,)?) => {
        $(
            fn $method<V>(self, $(_: $argty,)* visitor: V) -> Result<V::Value>
            where
                V: Visitor<'de>,
            {
                let _ = visitor;
                Err(Error::Unsupported($label))
            }
        )*
    };
}

#[derive(Debug)]
pub struct AdifDeserializer<'de> {
    input: &'de str,
}

/// Deserializes an ADIF document into `T`.
///
/// The top level of an ADIF document is always a sequence of records, so `T`
/// must be a sequence type (e.g. `Vec<Qso>`), not a single record struct -
/// deserializing directly into a bare struct returns an
/// [`Unsupported`](crate::Error::Unsupported) error.
pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut de = AdifDeserializer { input: s };
    T::deserialize(&mut de)
}

impl<'de> AdifDeserializer<'de> {
    fn parse_next(&mut self) -> Result<Option<(&'de str, &'de str)>> {
        // Trim leading whitespace
        self.input = self.input.trim_start();

        if self.input.is_empty() {
            return Ok(None);
        }

        // Skip header if first character is not '<'
        if !self.input.starts_with('<') {
            // Find <EOH> case-insensitively
            let eoh_pos = self
                .input
                .as_bytes()
                .windows(5) // "<EOH>" length
                .position(|w| w.eq_ignore_ascii_case(b"<eoh>"))
                .ok_or(Error::Eof)?;
            // Advance past <EOH>
            self.input = &self.input[eoh_pos + 5..];
            self.input = self.input.trim_start();
        }

        let Some(start) = self.input.find('<') else {
            return Ok(None);
        };
        let end = self.input[start..].find('>').ok_or_else(|| {
            // The rest of the input could be huge with no ">" ever
            // found (e.g. a truncated file), so only show a short
            // snippet of what follows the opening "<" rather than
            // the entire remainder.
            let snippet: String = self.input[start..].chars().take(20).collect();
            Error::ExpectedClosingTag(snippet)
        })? + start;
        let field_def = &self.input[start + 1..end];

        let mut parts = field_def.split(':');
        let key = parts
            .next()
            .ok_or_else(|| Error::MalformedTag(field_def.to_string()))?;
        let length_str = parts
            .next()
            .ok_or_else(|| Error::MalformedTag(field_def.to_string()))?;
        let length: usize = length_str
            .parse()
            .map_err(|_| Error::InvalidLength(length_str.to_string()))?;

        // The key may or may not contain a type hint
        // This will eventually be useful when we want to
        // implement a generic deserializer
        let _type_hint = parts.next();

        let value_start = end + 1;
        let value_end = value_start + length;

        if value_end > self.input.len() {
            return Err(Error::Eof);
        }

        let value = &self.input[value_start..value_end];

        self.input = &self.input[value_end..];
        Ok(Some((key, value)))
    }

    fn consume_eor(&mut self) -> bool {
        let trimmed = self.input.trim_start();
        if trimmed.to_uppercase().starts_with("<EOR>") {
            // advance past <EOR>
            self.input = &trimmed[5..];
            true
        } else {
            false
        }
    }

    fn deserialize_record<'a>(&'a mut self) -> RecordDeserializer<'a, 'de> {
        RecordDeserializer { de: self }
    }
}

impl<'de> Deserializer<'de> for &mut AdifDeserializer<'de> {
    type Error = crate::error::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // top-level is always a sequence of records separated by <EOR>
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(RecordSeq { de: self })
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(RecordFields {
            de: self,
            current: None,
        })
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    unsupported_deserialize! {
        deserialize_bool() => "bool",
        deserialize_i8() => "i8",
        deserialize_i16() => "i16",
        deserialize_i32() => "i32",
        deserialize_i64() => "i64",
        deserialize_u8() => "u8",
        deserialize_u16() => "u16",
        deserialize_u32() => "u32",
        deserialize_u64() => "u64",
        deserialize_f32() => "f32",
        deserialize_f64() => "f64",
        deserialize_char() => "char",
        deserialize_str() => "str",
        deserialize_string() => "string",
        deserialize_unit() => "unit",
        deserialize_bytes() => "bytes",
        deserialize_byte_buf() => "byte buf",
        deserialize_unit_struct(&'static str) => "unit struct",
        deserialize_tuple(usize) => "tuple",
        deserialize_tuple_struct(&'static str, usize) => "tuple struct",
        deserialize_struct(&'static str, &'static [&'static str]) => "struct",
        deserialize_enum(&'static str, &'static [&'static str]) => "enum",
    }

    forward_to_deserialize_any! {
        identifier ignored_any
    }
}

/// Sequence of ADIF records
struct RecordSeq<'a, 'de> {
    de: &'a mut AdifDeserializer<'de>,
}

impl<'de, 'a> SeqAccess<'de> for RecordSeq<'a, 'de> {
    type Error = crate::error::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.de.input.trim().is_empty() {
            return Ok(None);
        }

        let val = seed.deserialize(self.de.deserialize_record())?;

        // consume `<EOR>` if present
        self.de.consume_eor();
        Ok(Some(val))
    }
}

/// MapAccess implementation for ADIF record
struct RecordFields<'a, 'de> {
    de: &'a mut AdifDeserializer<'de>,
    current: Option<(String, &'de str)>,
}

impl<'de, 'a> MapAccess<'de> for RecordFields<'a, 'de> {
    type Error = crate::error::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self
            .de
            .input
            .trim_start()
            .to_uppercase()
            .starts_with("<EOR>")
        {
            return Ok(None);
        }
        if let Some((k, v)) = self.de.parse_next()? {
            let key = k.to_ascii_lowercase();
            self.current = Some((key.clone(), v));
            seed.deserialize(key.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some((_, v)) = self.current.take() {
            seed.deserialize(ValueDeserializer { value: v })
        } else {
            unreachable!("next_value_seed called without a preceding next_key_seed");
        }
    }
}

struct RecordDeserializer<'a, 'de> {
    de: &'a mut AdifDeserializer<'de>,
}

impl<'de, 'a> Deserializer<'de> for RecordDeserializer<'a, 'de> {
    type Error = crate::error::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(RecordFields {
            de: self.de,
            current: None,
        })
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    unsupported_deserialize! {
        deserialize_bool() => "bool",
        deserialize_i8() => "i8",
        deserialize_i16() => "i16",
        deserialize_i32() => "i32",
        deserialize_i64() => "i64",
        deserialize_u8() => "u8",
        deserialize_u16() => "u16",
        deserialize_u32() => "u32",
        deserialize_u64() => "u64",
        deserialize_f32() => "f32",
        deserialize_f64() => "f64",
        deserialize_char() => "char",
        deserialize_str() => "str",
        deserialize_string() => "string",
        deserialize_unit() => "unit",
        deserialize_bytes() => "bytes",
        deserialize_byte_buf() => "byte buf",
        deserialize_unit_struct(&'static str) => "unit struct",
        deserialize_newtype_struct(&'static str) => "newtype struct",
        deserialize_seq() => "seq",
        deserialize_tuple(usize) => "tuple",
        deserialize_tuple_struct(&'static str, usize) => "tuple struct",
        deserialize_enum(&'static str, &'static [&'static str]) => "enum",
    }

    forward_to_deserialize_any! {
        struct map identifier ignored_any
    }
}

struct ValueDeserializer<'de> {
    value: &'de str,
}

macro_rules! deserialize_number {
    ($($method:ident => $visit:ident : $ty:ty),* $(,)?) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value>
            where V: Visitor<'de>,
            {
                let parsed: $ty = self.value.trim().parse().map_err(|e| {
                    de::Error::custom(format!("cannot parse {:?} as {}: {e}", self.value, stringify!($ty)))
		})?;
                visitor.$visit(parsed)
            }
        )*
    };
}

struct CommaSeparated<'de, I: Iterator<Item = &'de str>> {
    items: I,
}

impl<'de, I: Iterator<Item = &'de str>> SeqAccess<'de> for CommaSeparated<'de, I> {
    type Error = crate::error::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.items.next() {
            Some(item) => seed
                .deserialize(ValueDeserializer { value: item })
                .map(Some),
            None => Ok(None),
        }
    }
}

impl<'de> Deserializer<'de> for ValueDeserializer<'de> {
    type Error = crate::error::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Try to parse number first
        if let Ok(i) = self.value.parse::<i64>() {
            return visitor.visit_i64(i);
        } else if let Ok(f) = self.value.parse::<f64>() {
            return visitor.visit_f64(f);
        }
        // fallback: treat as string
        visitor.visit_str(self.value)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        visitor.visit_string(self.value.to_string())
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.value)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        let mut chars = self.value.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => visitor.visit_char(c),
            _ => Err(de::Error::custom(format!(
                "expected a single character, found {:?}",
                self.value
            ))),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        let parsed: bool = match self.value.trim().to_lowercase().as_ref() {
            "y" => true,
            "n" => false,
            _ => {
                return Err(de::Error::custom(format!(
                    "expected \"Y\" or \"N\" (case-insensitive) for a boolean, found {:?}",
                    self.value
                )))
            }
        };
        visitor.visit_bool(parsed)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self.value.into_deserializer())
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let items = self
            .value
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        visitor.visit_seq(CommaSeparated { items })
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    deserialize_number! {
        deserialize_i8  => visit_i8  : i8,
        deserialize_i16 => visit_i16 : i16,
        deserialize_i32 => visit_i32 : i32,
        deserialize_i64 => visit_i64 : i64,
        deserialize_u8  => visit_u8  : u8,
        deserialize_u16 => visit_u16 : u16,
        deserialize_u32 => visit_u32 : u32,
        deserialize_u64 => visit_u64 : u64,
        deserialize_f32 => visit_f32 : f32,
        deserialize_f64 => visit_f64 : f64,
    }

    unsupported_deserialize! {
        deserialize_unit() => "unit",
        deserialize_bytes() => "bytes",
        deserialize_byte_buf() => "byte buf",
        deserialize_map() => "map",
        deserialize_unit_struct(&'static str) => "unit struct",
        deserialize_struct(&'static str, &'static [&'static str]) => "nested struct",
    }

    forward_to_deserialize_any! {
        identifier ignored_any
    }
}
