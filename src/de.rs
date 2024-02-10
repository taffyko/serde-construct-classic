use serde::{de::{self, DeserializeSeed, MapAccess, Visitor}, Deserialize};
use encoding_rs::{self, WINDOWS_1252};

use crate::constants::*;

use crate::error::{ErrorKind as ErrKind, ErrorWithOffset};

type Result<T> = std::result::Result<T, ErrorWithOffset>;

pub struct Deserializer<'de> {
    input: &'de [u8],
    start_len: usize,
    reading_value: bool,
    reading_key: bool,
}

pub fn from_bytes<'a, T>(b: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(b);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        ErrKind::TrailingCharacters.with(deserializer.offset())
    }
}

struct KeyValueList<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> KeyValueList<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        KeyValueList { de }
    }
}

impl<'de, 'a> MapAccess<'de> for KeyValueList<'a, 'de> {
    type Error = ErrorWithOffset;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // Check if there are no more entries.
        if self.de.input.len() == 0 {
            return Ok(None);
        }
        self.de.reading_key = true;
        let result = seed.deserialize(&mut *self.de).map(Some);
        self.de.reading_key = false;
        result
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // Deserialize a map value.
        self.de.reading_value = true;
        let result = seed.deserialize(&mut *self.de);
        self.de.reading_value = false;
        result
    }
}

impl<'de> Deserializer<'de> {
    fn offset(&self) -> usize {
        return self.start_len - self.input.len();
    }

    fn peek_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.input[..4].try_into().unwrap())
    }

    fn read_u32(&mut self) -> u32 {
        let v = u32::from_le_bytes(self.input[..4].try_into().unwrap());
        self.input = &self.input[4..];
        v
    }

    fn read_i64(&mut self) -> Result<i64> {
        if self.reading_value {
            if self.read_u32() != TYPE_I64 { return ErrKind::TypeMismatch.with(self.offset()); }
        }
        let v = i64::from_le_bytes(self.input[..8].try_into().unwrap());
        self.input = &self.input[8..];
        Ok(v)
    }

    fn read_f64(&mut self) -> Result<f64> {
        if self.reading_value {
            if self.read_u32() != TYPE_F64 { return ErrKind::TypeMismatch.with(self.offset()); }
        }
        let v = f64::from_le_bytes(self.input[..8].try_into().unwrap());
        self.input = &self.input[8..];
        Ok(v)
    }

    fn read_integer<T>(&mut self) -> Result<T>
    where T: TryFrom<i64>
    {
        let result = T::try_from(self.read_i64()?);
        result.or(ErrKind::NumericOverflow.with(self.offset()))
    }

    pub fn read_string(&mut self) -> Result<String> {
        if self.reading_value {
            if self.read_u32() != TYPE_STRING { return ErrKind::TypeMismatch.with(self.offset()); }
        }
        let len = self.read_u32() as usize;
        if len > self.input.len() {
            return ErrKind::StringLengthError(len, self.input.len()).with(self.offset()-4);
        }
        if self.input[len-1] != 0 {
            // all strings end with a NUL
            return ErrKind::MissingStringTerminator.with(self.offset()+len-1);
        }
        let slice = &self.input[..len-1]; // exclude the terminating NUL byte
        let (s, _, _) = WINDOWS_1252.decode(slice);
        self.input = &self.input[len..];
        Ok(s.to_string())
    }

    pub fn from_bytes(input: &'de [u8]) -> Self {
        Self { start_len: input.len(), input, reading_value: false, reading_key: false }
    }
}


impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = ErrorWithOffset;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        if self.reading_key {
            self.deserialize_str(visitor)
        } else if self.reading_value {
            match self.peek_u32() {
                TYPE_I64 => { self.deserialize_i64(visitor) },
                TYPE_F64 => { self.deserialize_f64(visitor) },
                TYPE_STRING => { self.deserialize_str(visitor) },
                ty => { ErrKind::UnknownTypeId(ty).with(self.offset()) }
            }
        } else {
            self.deserialize_map(visitor)
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        visitor.visit_i64(self.read_integer()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        visitor.visit_f64(self.read_f64()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        visitor.visit_string(self.read_string()?)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        visitor.visit_bool(self.read_i64()? != 0)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        visitor.visit_i8(self.read_integer()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        visitor.visit_i16(self.read_integer()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        visitor.visit_i32(self.read_integer()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        visitor.visit_u8(self.read_integer()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        visitor.visit_u8(self.read_integer()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        visitor.visit_u32(self.read_integer()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        visitor.visit_u64(self.read_integer()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        visitor.visit_f32(self.read_f64()? as f32)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de> {
            self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        // magic number: MAP1.0
        if !(
            self.input[0] == 0x4D
            && self.input[1] == 0x41
            && self.input[2] == 0x50
            && self.input[3] == 0x31
            && self.input[4] == 0x2E
            && self.input[5] == 0x30
        ) {
            return ErrKind::InvalidHeader.with(self.offset());
        }
        self.input = &self.input[6..];
        let _key_count = self.read_u32();
        let value = visitor.visit_map(KeyValueList::new(self))?;
        Ok(value)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de> {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de> {
        self.deserialize_any(visitor)
    }
}