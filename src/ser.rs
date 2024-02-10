use encoding_rs::{self, WINDOWS_1252};
use serde::{ser, Serialize};

use crate::constants::*;
use crate::error::ErrorKind as Error;

type Result<T> = std::result::Result<T, Error>;

pub struct Serializer {
    output: Vec<u8>,
    writing_value: bool,
    writing_key: bool,
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: Vec::new(),
        writing_key: false,
        writing_value: false,
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}


impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i8(self, v: i8) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        if self.writing_key { return Err(Error::InvalidKeyType); }
        if self.writing_value {
            self.output.extend(TYPE_I64.to_le_bytes());
        }
        self.output.extend(v.to_le_bytes());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_u16(self, v: u16) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_u32(self, v: u32) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_u64(self, v: u64) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::try_from(v).or(Err(Error::NumericOverflow))?)
    }

    fn serialize_f32(self, v: f32) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        if self.writing_key { return Err(Error::InvalidKeyType); }
        if self.writing_value {
            self.output.extend(TYPE_F64.to_le_bytes());
        }
        self.output.extend(v.to_le_bytes());
        Ok(())
    }

    fn serialize_char(self, v: char) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        if self.writing_value {
            self.output.extend(TYPE_STRING.to_le_bytes());
        }
        let (bytes, _, encoding_errors) = WINDOWS_1252.encode(v);
        let len = u32::try_from(bytes.len() + 1).or(Err(Error::NumericOverflow))?;
        self.output.extend(len.to_le_bytes());
        if encoding_errors { return Err(Error::TextEncodingError); }
        self.output.extend(bytes.to_vec());
        self.output.push(0);
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        Err(Error::UnsupportedValue)
    }

    fn serialize_none(self) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        Err(Error::UnsupportedValue)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> std::prelude::v1::Result<Self::Ok, Self::Error>
    where
        T: Serialize
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        Err(Error::UnsupportedValue)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        Err(Error::UnsupportedValue)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        Err(Error::UnsupportedValue)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> std::prelude::v1::Result<Self::Ok, Self::Error>
    where
        T: Serialize
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> std::prelude::v1::Result<Self::Ok, Self::Error>
    where T: Serialize {
        Err(Error::UnsupportedValue)
    }

    fn serialize_seq(self, _len: Option<usize>) -> std::prelude::v1::Result<Self::SerializeSeq, Self::Error> {
        Err(Error::UnsupportedValue)
    }

    fn serialize_tuple(self, _len: usize) -> std::prelude::v1::Result<Self::SerializeTuple, Self::Error> {
        Err(Error::UnsupportedValue)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::prelude::v1::Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::UnsupportedValue)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> std::prelude::v1::Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::UnsupportedValue)
    }

    fn serialize_map(self, len: Option<usize>) -> std::prelude::v1::Result<Self::SerializeMap, Self::Error> {
        if self.writing_key { return Err(Error::InvalidKeyType); }
        if self.writing_value { return Err(Error::UnsupportedValue); }
        let Some(len) = len else {
            return Err(Error::LengthNotGiven);
        };
        self.output.extend([
            0x4D,
            0x41,
            0x50,
            0x31,
            0x2E,
            0x30,
        ]);
        let len = u32::try_from(len).or(Err(Error::NumericOverflow))?;
        self.output.extend(len.to_le_bytes());
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> std::prelude::v1::Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> std::prelude::v1::Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::UnsupportedValue)
    }
}


impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.writing_key = true;
        let result = key.serialize(&mut **self);
        self.writing_key = false;
        result
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.writing_value = true;
        let result = value.serialize(&mut **self);
        self.writing_value = false;
        result
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.writing_key = true;
        let result = key.serialize(&mut **self);
        self.writing_key = false;
        result?;
        self.writing_value = true;
        let result = value.serialize(&mut **self);
        self.writing_value = false;
        result
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}


impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> std::prelude::v1::Result<(), Self::Error>
    where T: Serialize {
        unimplemented!()
    }

    fn end(self) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}
impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> std::prelude::v1::Result<(), Self::Error>
    where T: Serialize {
        unimplemented!()
    }

    fn end(self) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}
impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> std::prelude::v1::Result<(), Self::Error>
    where T: Serialize {
        unimplemented!()
    }

    fn end(self) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}
impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> std::prelude::v1::Result<(), Self::Error>
    where T: Serialize {
        unimplemented!()
    }

    fn end(self) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}
impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> std::prelude::v1::Result<(), Self::Error>
    where T: Serialize {
        unimplemented!()
    }

    fn end(self) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}