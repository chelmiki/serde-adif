use serde::de::{
    self, Deserialize, DeserializeSeed, Deserializer as SerdeDeserializer, IntoDeserializer,
    MapAccess, SeqAccess, Visitor,
};
use serde::forward_to_deserialize_any;

#[derive(Debug)]
pub struct Deserializer<'de> {
    input: &'de str,
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T, de::value::Error>
where
    T: Deserialize<'a>,
{
    let mut de = Deserializer { input: s };
    T::deserialize(&mut de)
}

impl<'de> Deserializer<'de> {
    fn parse_next(&mut self) -> Option<(&'de str, &'de str)> {
        // Trim leading whitespace
        self.input = self.input.trim_start();

        if self.input.is_empty() {
            return None;
        }

        // Skip header if first character is not '<'
        if !self.input.starts_with('<') {
            // Find <EOH> case-insensitively
            let eoh_pos = self
                .input
                .as_bytes()
                .windows(5) // "<EOH>" length
                .position(|w| w.eq_ignore_ascii_case(b"<eoh>"))?;
            // Advance past <EOH>
            self.input = &self.input[eoh_pos + 5..];
            self.input = self.input.trim_start();
        }

        let start = self.input.find('<')?;
        let end = self.input[start..].find('>')? + start;
        let field_def = &self.input[start + 1..end];

        let mut parts = field_def.split(':');
        let key = parts.next()?;
        let length: usize = parts.next()?.parse().ok()?;

        let value_start = end + 1;
        let value_end = value_start + length;

        if value_end > self.input.len() {
            return None;
        }

        let value = &self.input[value_start..value_end];

        self.input = &self.input[value_end..];
        Some((key, value))
    }

    fn consume_eor(&mut self) -> bool {
        let trimmed = self.input.trim_start();
        if trimmed.to_uppercase().starts_with("<EOR>") {
            // advance past "<EOR>"
            let pos = trimmed.find("<EOR>").unwrap();
            self.input = &trimmed[pos + 5..];
            true
        } else {
            false
        }
    }

    fn deserialize_record<'a>(&'a mut self) -> RecordDeserializer<'a, 'de> {
        RecordDeserializer { de: self }
    }
}

impl<'de, 'a> SerdeDeserializer<'de> for &'a mut Deserializer<'de> {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // top-level is always a sequence of records separated by <EOR>
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Records { de: self })
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(Map {
            de: self,
            current: None,
        })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string unit option enum
        bytes byte_buf struct identifier ignored_any unit_struct newtype_struct tuple tuple_struct
    }
}

/// Sequence of ADIF records
struct Records<'a, 'de> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> SeqAccess<'de> for Records<'a, 'de> {
    type Error = de::value::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
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
struct Map<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    current: Option<(String, &'de str)>,
}

impl<'de, 'a> MapAccess<'de> for Map<'a, 'de> {
    type Error = de::value::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((k, v)) = self.de.parse_next() {
            let key = k.to_ascii_lowercase();
            if key == "eor" {
                return Ok(None);
            }
            self.current = Some((key.clone(), v));
            seed.deserialize(key.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some((_, v)) = self.current.take() {
            seed.deserialize(ValueDeserializer { value: v })
        } else {
            Err(de::Error::custom("value without key"))
        }
    }
}

struct RecordDeserializer<'a, 'de> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> serde::de::Deserializer<'de> for RecordDeserializer<'a, 'de> {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(Map {
            de: self.de,
            current: None,
        })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        unit option map enum bytes byte_buf struct identifier ignored_any
        unit_struct newtype_struct seq tuple tuple_struct
    }
}

struct ValueDeserializer<'de> {
    value: &'de str,
}

impl<'de> SerdeDeserializer<'de> for ValueDeserializer<'de> {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Try to parse number first
        if let Ok(i) = self.value.parse::<i64>() {
            return visitor.visit_i64(i);
        } else if let Ok(f) = self.value.parse::<f64>() {
            return visitor.visit_f64(f);
        } else if self.value.eq_ignore_ascii_case("true") {
            return visitor.visit_bool(true);
        } else if self.value.eq_ignore_ascii_case("false") {
            return visitor.visit_bool(false);
        }

        // fallback: treat as string
        visitor.visit_str(self.value)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string unit option enum
        bytes map seq byte_buf struct identifier ignored_any unit_struct newtype_struct tuple tuple_struct
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Record {
        call: String,
        mode: String,
        freq: f32,
        cnt: u32,
        new: bool,
    }

    #[test]
    fn test_deserialize() {
        let input = "This is an Adif header\
                     <EOH>\
                     <CALL:6>EI4JKB<Mode:3>FT8<Freq:6>14.074<cnt:1>1<new:4>true<EOR>\
                     <call:4>EI4L<mode:2>CW<freq:4>3.46<cnt:1>2<new:5>false<EOR>";

        let records: Vec<Record> = from_str(input).unwrap();
        println!("{:?}", records);
    }
}
