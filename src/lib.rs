// Copyright (c) 2017 The Robigalia Project Developers Licensed under the Apache License, Version
// 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. All files in the project
// carrying such notice may not be copied, modified, or distributed except according to those
// terms.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate core;

extern crate serde;
extern crate encode_unicode;

use core::intrinsics::transmute;

use serde::{Serialize, Deserialize};
use serde::de::{EnumVisitor, Visitor};
use serde::de::value::ValueDeserializer;
use encode_unicode::CharExt;

#[derive(Debug)]
pub enum Error {
    EndOfStream,
    InvalidRepresentation,
    MoreElements,
    TooManyVariants,
    NotSupported,
    ApplicationError(&'static str),
    #[cfg(not(feature = "std"))]
    Custom(&'static str),
    #[cfg(feature = "std")]
    Custom(String),
}


impl core::fmt::Display for Error {
    #[inline] fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        #[cfg(feature = "std")]
        use std::error::Error;
        #[cfg(not(feature = "std"))]
        use serde::error::Error;

        match self {
            &::Error::ApplicationError(s) => write!(f, "application error: {}", s),
            _ => f.write_str(self.description()),
        }
    }
}

#[cfg(not(feature = "std"))]
impl serde::de::Error for Error {
    #[inline] fn custom<T: Into<&'static str>>(msg: T) -> Error {
        Error::Custom(msg.into())
    }

    #[inline] fn end_of_stream() -> Error {
        Error::EndOfStream
    }
}

#[cfg(not(feature = "std"))]
impl serde::ser::Error for Error {
    #[inline] fn custom<T: Into<&'static str>>(msg: T) -> Error {
        Error::Custom(msg.into())
    }
}

#[cfg(feature = "std")]
impl serde::de::Error for Error {
    #[inline] fn custom<T: Into<String>>(msg: T) -> Error {
        Error::Custom(msg.into())
    }

    #[inline] fn end_of_stream() -> Error {
        Error::EndOfStream
    }
}

#[cfg(feature = "std")]
impl serde::ser::Error for Error {
    #[inline] fn custom<T: Into<String>>(msg: T) -> Error {
        Error::Custom(msg.into())
    }
}

#[cfg(not(feature = "std"))]
impl serde::error::Error for Error {
    #[inline] fn description(&self) -> &str {
        match self {
            &Error::EndOfStream => "end of stream reached but more data was needed",
            &Error::InvalidRepresentation => "invalid representation for a value",
            &Error::MoreElements => "there are more elements of the sequence remaining",
            &Error::TooManyVariants => "too many variants, only up to 256 are supported",
            &Error::NotSupported => "feature not supported",
            &Error::ApplicationError(s) => s,
            &Error::Custom(s) => s
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    #[inline] fn description(&self) -> &str {
        match self {
            &Error::EndOfStream => "end of stream reached but more data was needed",
            &Error::InvalidRepresentation => "invalid representation for a value",
            &Error::MoreElements => "there are more elements of the sequence remaining",
            &Error::TooManyVariants => "too many variants, only up to 256 are supported",
            &Error::NotSupported => "feature not supported",
            &Error::ApplicationError(s) => s,
            &Error::Custom(ref s) => &s
        }
    }
}

/// Serialize a value into a buffer. Returns the number of bytes used.
pub fn serialize<T: Serialize>(buf: &mut [u8], val: &T) -> SerializeResult<usize> {
    let mut serializer = Serializer { buf: buf, idx: 0 };
    T::serialize(val, &mut serializer)?;
    Ok(serializer.idx)
}

/// Deserialize a value from a buffer. Returns the number of bytes used.
pub fn deserialize<T: Deserialize>(buf: &[u8]) -> SerializeResult<(T, usize)> {
    let mut deserializer = Deserializer { buf: buf, idx: 0 };
    let val = T::deserialize(&mut deserializer)?;
    Ok((val, deserializer.idx))
}

struct Serializer<'a> {
    buf: &'a mut [u8],
    idx: usize,
}

impl<'a> Serializer<'a> {
    #[inline] fn check_bounds(&self, len: usize) -> Result<(), Error> {
        if let Some(val) = self.idx.checked_add(len) {
            if val <= self.buf.len() {
                return Ok(());
            }
        }
        Err(Error::EndOfStream)
    }

    #[inline] fn write_u8(&mut self, val: u8) -> Result<(), Error> {
        self.check_bounds(1)?;
        unsafe {
            *self.buf.get_unchecked_mut(self.idx) = val;
        }
        self.idx += 1;
        Ok(())
    }

    #[inline] fn write_u16(&mut self, val: u16) -> Result<(), Error> {
        self.check_bounds(2)?;
        unsafe {
            *self.buf.get_unchecked_mut(self.idx) = (val & 0xFF) as u8;
            *self.buf.get_unchecked_mut(self.idx + 1) = (val >> 8 & 0xFF) as u8;
        }
        self.idx += 2;
        Ok(())
    }

    #[inline] fn write_u32(&mut self, val: u32) -> Result<(), Error> {
        self.check_bounds(4)?;
        unsafe {
            *self.buf.get_unchecked_mut(self.idx) = (val & 0xFF) as u8;
            *self.buf.get_unchecked_mut(self.idx + 1) = (val >> 8 & 0xFF) as u8;
            *self.buf.get_unchecked_mut(self.idx + 2) = (val >> 16 & 0xFF) as u8;
            *self.buf.get_unchecked_mut(self.idx + 3) = (val >> 24 & 0xFF) as u8;
        }
        self.idx += 4;
        Ok(())
    }

    #[inline] fn write_u64(&mut self, val: u64) -> Result<(), Error> {
        self.check_bounds(8)?;
        unsafe { 
            *self.buf.get_unchecked_mut(self.idx) = (val & 0xFF) as u8;
            *self.buf.get_unchecked_mut(self.idx + 1) = (val >> 8 & 0xFF) as u8;
            *self.buf.get_unchecked_mut(self.idx + 2) = (val >> 16 & 0xFF) as u8;
            *self.buf.get_unchecked_mut(self.idx + 3) = (val >> 24 & 0xFF) as u8;
            *self.buf.get_unchecked_mut(self.idx + 4) = (val >> 32 & 0xFF) as u8;
            *self.buf.get_unchecked_mut(self.idx + 5) = (val >> 40 & 0xFF) as u8;
            *self.buf.get_unchecked_mut(self.idx + 6) = (val >> 48 & 0xFF) as u8;
            *self.buf.get_unchecked_mut(self.idx + 7) = (val >> 56 & 0xFF) as u8;
        }
        self.idx += 8;
        Ok(())
    }

    #[inline] fn write_slice(&mut self, slice: &[u8]) -> Result<(), Error> {
        self.check_bounds(slice.len())?;
        self.buf[self.idx..self.idx + slice.len()].copy_from_slice(slice);
        self.idx += slice.len();
        Ok(())
    }

    #[inline] fn write_usize(&mut self, val: usize) -> Result<(), Error> {
        self.write_u64(val as u64)
    }
}

type SerializeResult<T> = Result<T, Error>;

impl<'a> serde::Serializer for Serializer<'a> {
    type Error = Error;
    type SeqState = ();
    type TupleState = ();
    type TupleStructState = ();
    type TupleVariantState = ();
    type MapState = ();
    type StructState = ();
    type StructVariantState = ();

    #[inline] fn serialize_unit(&mut self) -> SerializeResult<()> {
        Ok(())
    }

    #[inline] fn serialize_unit_struct(&mut self, _: &'static str) -> SerializeResult<()> {
        Ok(())
    }

    #[inline] fn serialize_bool(&mut self, v: bool) -> SerializeResult<()> {
        self.write_u8(if v { 1 } else { 0 })
    }

    #[inline] fn serialize_u8(&mut self, v: u8) -> SerializeResult<()> {
        self.write_u8(v)
    }

    #[inline] fn serialize_u16(&mut self, v: u16) -> SerializeResult<()> {
        self.write_u16(v)
    }

    #[inline] fn serialize_u32(&mut self, v: u32) -> SerializeResult<()> {
        self.write_u32(v)
    }

    #[inline] fn serialize_u64(&mut self, v: u64) -> SerializeResult<()> {
        self.write_u64(v)
    }

    #[inline] fn serialize_usize(&mut self, v: usize) -> SerializeResult<()> {
        self.serialize_u64(v as u64)
    }

    #[inline] fn serialize_i8(&mut self, v: i8) -> SerializeResult<()> {
        self.write_u8(v as u8)
    }

    #[inline] fn serialize_i16(&mut self, v: i16) -> SerializeResult<()> {
        self.write_u16(v as u16)
    }

    #[inline] fn serialize_i32(&mut self, v: i32) -> SerializeResult<()> {
        self.write_u32(v as u32)
    }

    #[inline] fn serialize_i64(&mut self, v: i64) -> SerializeResult<()> {
        self.write_u64(v as u64)
    }

    #[inline] fn serialize_isize(&mut self, v: isize) -> SerializeResult<()> {
        self.serialize_i64(v as i64)
    }

    #[inline] fn serialize_f32(&mut self, v: f32) -> SerializeResult<()> {
        self.write_u32(unsafe { transmute(v) })
    }

    #[inline] fn serialize_f64(&mut self, v: f64) -> SerializeResult<()> {
        self.write_u64(unsafe { transmute(v) })
    }

    #[inline] fn serialize_str(&mut self, v: &str) -> SerializeResult<()> {
        self.serialize_usize(v.len())?;
        self.write_slice(v.as_bytes())
    }

    #[inline] fn serialize_char(&mut self, c: char) -> SerializeResult<()> {
        self.check_bounds(c.len_utf8())?;
        self.idx += c.to_utf8_slice(&mut self.buf[self.idx..]);
        Ok(())
    }

    #[inline] fn serialize_bytes(&mut self, v: &[u8]) -> SerializeResult<()> {
        self.serialize_usize(v.len())?;
        self.write_slice(v)
    }

    #[inline] fn serialize_none(&mut self) -> SerializeResult<()> {
        self.write_u8(0)
    }

    #[inline] fn serialize_some<T: Serialize>(&mut self, v: T) -> SerializeResult<()> {
        self.write_u8(1)?;
        v.serialize(self)
    }

    #[inline] fn serialize_seq(&mut self, len: Option<usize>) -> SerializeResult<()> {
        match len {
            Some(l) => self.serialize_usize(l),
            None => Err(Error::ApplicationError("serialize_seq needs a size")),
        }
    }

    #[inline] fn serialize_seq_elt<V: Serialize>(&mut self, _: &mut (), value: V) -> SerializeResult<()> {
        value.serialize(self)
    }

    #[inline] fn serialize_seq_end(&mut self, _: ()) -> SerializeResult<()> {
        Ok(())
    }

    #[inline] fn serialize_seq_fixed_size(&mut self, _len: usize) -> SerializeResult<()> {
        Ok(())
    }

    #[inline] fn serialize_tuple(&mut self, _len: usize) -> SerializeResult<()> {
        Ok(())
    }

    #[inline] fn serialize_tuple_elt<V: Serialize>(&mut self, _: &mut (), value: V) -> SerializeResult<()> {
        value.serialize(self)
    }

    #[inline] fn serialize_tuple_end(&mut self, _: ()) -> SerializeResult<()> {
        Ok(())
    }

    #[inline] fn serialize_tuple_struct(&mut self, _name: &'static str, _len: usize) -> SerializeResult<()> {
        Ok(())
    }

    #[inline] fn serialize_tuple_struct_elt<V: Serialize>(&mut self, _: &mut (), value: V) -> SerializeResult<()> {
        value.serialize(self)
    }

    #[inline] fn serialize_tuple_struct_end(&mut self, _: ()) -> SerializeResult<()> {
        Ok(())
    }

    #[inline] fn serialize_tuple_variant(&mut self,
                               _name: &'static str,
                               variant_index: usize,
                               _variant: &'static str,
                               _len: usize)
                               -> SerializeResult<()> {
        // FIXME: once serde 0.9 is out
        if variant_index > 255 {
            panic!("Can't encode enum with more than 256 variants!");
        }
        self.write_u8(variant_index as u8)
    }

    #[inline] fn serialize_tuple_variant_elt<V: Serialize>(&mut self, _: &mut (), value: V) -> SerializeResult<()> {
        value.serialize(self)
    }

    #[inline] fn serialize_tuple_variant_end(&mut self, _: ()) -> SerializeResult<()> {
        Ok(())
    }

    #[inline] fn serialize_map(&mut self, _len: Option<usize>) -> SerializeResult<()> {
        Err(Error::NotSupported)
    }

    #[inline] fn serialize_map_key<K: Serialize>(&mut self, _: &mut (), _key: K) -> SerializeResult<()> {
        Err(Error::NotSupported)
    }

    #[inline] fn serialize_map_value<V: Serialize>(&mut self, _: &mut (), _value: V) -> SerializeResult<()> {
        Err(Error::NotSupported)
    }

    #[inline] fn serialize_map_end(&mut self, _: ()) -> SerializeResult<()> {
        Err(Error::NotSupported)
    }

    #[inline] fn serialize_struct(&mut self, _name: &'static str, _len: usize) -> SerializeResult<()> {
        Ok(())
    }

    #[inline] fn serialize_struct_elt<V: Serialize>(&mut self,
                               _: &mut (),
                               _key: &'static str,
                               value: V)
                               -> SerializeResult<()> {
        value.serialize(self)
    }

    #[inline] fn serialize_struct_end(&mut self, _: ()) -> SerializeResult<()> {
        Ok(())
    }

    #[inline] fn serialize_struct_variant(&mut self,
                                _name: &'static str,
                                variant_index: usize,
                                _variant: &'static str,
                                _len: usize)
                                -> SerializeResult<()> {
        // FIXME: once serde 0.9 is out
        if variant_index > 255 {
            return Err(Error::TooManyVariants);
        }
        self.write_u8(variant_index as u8)
    }

    #[inline] fn serialize_struct_variant_elt<V: Serialize>(&mut self,
                                       _: &mut (),
                                       _key: &'static str,
                                       value: V)
                                       -> SerializeResult<()> {
        value.serialize(self)
    }

    #[inline] fn serialize_struct_variant_end(&mut self, _: ()) -> SerializeResult<()> {
        Ok(())
    }

    #[inline] fn serialize_newtype_struct<T: Serialize>(&mut self, _name: &'static str, value: T) -> SerializeResult<()> {
        value.serialize(self)
    }

    #[inline] fn serialize_newtype_variant<T: Serialize>(&mut self,
                                    _name: &'static str,
                                    variant_index: usize,
                                    _variant: &'static str,
                                    value: T)
                                    -> SerializeResult<()> {
        // FIXME: once serde 0.9 is out
        if variant_index > 255 {
            return Err(Error::TooManyVariants);
        }
        self.write_u8(variant_index as u8)?;
        value.serialize(self)
    }

    #[inline] fn serialize_unit_variant(&mut self,
                              _name: &'static str,
                              variant_index: usize,
                              _variant: &'static str)
                              -> SerializeResult<()> {
        self.write_usize(variant_index)
    }
}

struct Deserializer<'a> {
    buf: &'a [u8],
    idx: usize,
}

impl<'a> Deserializer<'a> {
    #[inline] fn check_bounds(&self, len: usize) -> Result<(), Error> {
        if let Some(val) = self.idx.checked_add(len) {
            if val <= self.buf.len() {
                return Ok(());
            }
        }
        Err(Error::EndOfStream)
    }

    #[inline] fn read_u8(&mut self) -> Result<u8, Error> {
        self.check_bounds(1)?;
        let val = unsafe {
            *self.buf.get_unchecked(self.idx)
        };
        self.idx += 1;
        Ok(val)
    }

    #[inline] fn read_u16(&mut self) -> Result<u16, Error> {
        self.check_bounds(2)?;
        let mut val;
        unsafe {
            val = *self.buf.get_unchecked(self.idx) as u16;
            val |= (*self.buf.get_unchecked(self.idx + 1) as u16) << 8;
        }
        self.idx += 2;
        Ok(val)
    }

    #[inline] fn read_u32(&mut self) -> Result<u32, Error> {
        self.check_bounds(4)?;
        let mut val;
        unsafe {
            val = *self.buf.get_unchecked(self.idx) as u32;
            val |= (*self.buf.get_unchecked(self.idx + 1) as u32) << 8;
            val |= (*self.buf.get_unchecked(self.idx + 2) as u32) << 16;
            val |= (*self.buf.get_unchecked(self.idx + 3) as u32) << 24;
        }
        self.idx += 4;
        Ok(val)
    }

    #[inline] fn read_u64(&mut self) -> Result<u64, Error> {
        self.check_bounds(8)?;
        let mut val;
        unsafe {
            val = *self.buf.get_unchecked(self.idx) as u64;
            val |= (*self.buf.get_unchecked(self.idx + 1) as u64) << 8;
            val |= (*self.buf.get_unchecked(self.idx + 2) as u64) << 16;
            val |= (*self.buf.get_unchecked(self.idx + 3) as u64) << 24;
            val |= (*self.buf.get_unchecked(self.idx + 4) as u64) << 32;
            val |= (*self.buf.get_unchecked(self.idx + 5) as u64) << 40;
            val |= (*self.buf.get_unchecked(self.idx + 6) as u64) << 48;
            val |= (*self.buf.get_unchecked(self.idx + 7) as u64) << 56;
        }
        self.idx += 8;
        Ok(val)
    }
}


struct SeqVisitor<'a, 'b: 'a> {
    deserializer: &'a mut Deserializer<'b>,
    len: usize,
}

impl<'a, 'b: 'a> serde::de::SeqVisitor for SeqVisitor<'a, 'b> {
    type Error = Error;

    #[inline] fn visit<T: Deserialize>(&mut self) -> Result<Option<T>, Error> {
        if self.len > 0 {
            self.len -= 1;
            Ok(Some(Deserialize::deserialize(self.deserializer)?))
        } else {
            Ok(None)
        }
    }

    #[inline] fn end(&mut self) -> Result<(), Error> {
        if self.len == 0 {
            Ok(())
        } else {
            Err(Error::MoreElements)
        }
    }
}

type DeserializeResult<T> = Result<T, Error>;

impl<'a> serde::Deserializer for Deserializer<'a> {
    type Error = Error;

    #[inline] fn deserialize<V: Visitor>(&mut self, _visitor: V) -> DeserializeResult<V::Value> {
        Err(Error::NotSupported)
    }

    #[inline] fn deserialize_bool<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        let value: u8 = Deserialize::deserialize(self)?;
        match value {
            0 => visitor.visit_bool(false),
            1 => visitor.visit_bool(true),
            _ => Err(Error::InvalidRepresentation),
        }
    }

    #[inline] fn deserialize_u8<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        visitor.visit_u8(self.read_u8()?)
    }

    #[inline] fn deserialize_u16<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        visitor.visit_u16(self.read_u16()?)
    }

    #[inline] fn deserialize_u32<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        visitor.visit_u32(self.read_u32()?)
    }

    #[inline] fn deserialize_u64<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        visitor.visit_u64(self.read_u64()?)
    }

    #[inline] fn deserialize_usize<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        visitor.visit_usize(self.read_u64()? as usize)
    }

    #[inline] fn deserialize_i8<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        visitor.visit_i8(self.read_u8()? as i8)
    }

    #[inline] fn deserialize_i16<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        visitor.visit_i16(self.read_u16()? as i16)
    }

    #[inline] fn deserialize_i32<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        visitor.visit_i32(self.read_u32()? as i32)
    }

    #[inline] fn deserialize_i64<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        visitor.visit_i64(self.read_u64()? as i64)
    }

    #[inline] fn deserialize_f32<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        visitor.visit_f32(unsafe { transmute(self.read_u32()?) })
    }
    #[inline] fn deserialize_f64<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        visitor.visit_f64(unsafe { transmute(self.read_u64()?) })
    }

    #[inline] fn deserialize_isize<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        visitor.visit_isize(self.read_u64()? as isize)
    }

    #[inline] fn deserialize_unit<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        visitor.visit_unit()
    }

    #[inline] fn deserialize_char<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        match char::from_utf8_slice(&self.buf[self.idx..]) {
            Ok((c, count)) => {
                // this ought to be safe, if it weren't how did from_utf8_slice do its thing?
                self.idx = self.idx.wrapping_add(count); 
                visitor.visit_char(c)
            },
            Err(_) => Err(Error::InvalidRepresentation),
        }
    }

    #[inline] fn deserialize_str<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        // decode len
        let len: usize = Deserialize::deserialize(self)?;
        self.check_bounds(len)?;
        let slice = &self.buf[self.idx..self.idx + len];
        match core::str::from_utf8(slice) {
            Ok(s) => visitor.visit_str(s),
            Err(_) => Err(Error::InvalidRepresentation),
        }
    }

    #[inline] fn deserialize_string<V: Visitor>(&mut self, _visitor: V) -> DeserializeResult<V::Value> {
        Err(Error::NotSupported) // FIXME: depend on std/collections optionally?
    }

    #[inline] fn deserialize_bytes<V: Visitor>(&mut self, visitor: V) -> DeserializeResult<V::Value> {
        self.deserialize_seq(visitor)
    }

    #[inline] fn deserialize_enum<V: EnumVisitor>(&mut self,
                           _enum: &'static str,
                           _variants: &'static [&'static str],
                           mut visitor: V)
                           -> Result<V::Value, Error> {
        visitor.visit(self)
    }

    #[inline] fn deserialize_tuple<V: Visitor>(&mut self, _len: usize, mut visitor: V) -> DeserializeResult<V::Value> {
        struct TupleVisitor<'a, 'b: 'a>(&'a mut Deserializer<'b>);

        impl<'a, 'b: 'a> serde::de::SeqVisitor for TupleVisitor<'a, 'b> {
            type Error = Error;

            #[inline] fn visit<T: Deserialize>(&mut self) -> Result<Option<T>, Error> {
                Ok(Some(serde::Deserialize::deserialize(self.0)?))
            }

            #[inline] fn end(&mut self) -> Result<(), Error> {
                Ok(())
            }
        }

        visitor.visit_seq(TupleVisitor(self))
    }

    #[inline] fn deserialize_seq_fixed_size<V: Visitor>(&mut self, len: usize, mut visitor: V) -> DeserializeResult<V::Value> {
        visitor.visit_seq(SeqVisitor {
            deserializer: self,
            len: len
        })
    }

    #[inline] fn deserialize_option<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        let value: u8 = Deserialize::deserialize(self)?;
        match value {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            _ => Err(Error::InvalidRepresentation),
        }
    }

    #[inline] fn deserialize_seq<V: Visitor>(&mut self, mut visitor: V) -> DeserializeResult<V::Value> {
        let len = Deserialize::deserialize(self)?;

        visitor.visit_seq(SeqVisitor {
            deserializer: self,
            len: len,
        })
    }

    #[inline] fn deserialize_map<V: Visitor>(&mut self, _visitor: V) -> DeserializeResult<V::Value> {
        Err(Error::NotSupported) // FIXME: depend on std/collections optionally?
    }

    #[inline] fn deserialize_struct<V: Visitor>(&mut self,
                             _name: &str,
                             fields: &'static [&'static str],
                             visitor: V)
                             -> DeserializeResult<V::Value> {
        self.deserialize_tuple(fields.len(), visitor)
    }

    #[inline] fn deserialize_struct_field<V: Visitor>(&mut self, _visitor: V) -> DeserializeResult<V::Value> {
        Err(Error::NotSupported)
    }

    #[inline] fn deserialize_newtype_struct<V: Visitor>(&mut self,
                                     _name: &str,
                                     mut visitor: V)
                                     -> DeserializeResult<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    #[inline] fn deserialize_unit_struct<V: Visitor>(&mut self,
                                  _name: &'static str,
                                  mut visitor: V)
                                  -> DeserializeResult<V::Value> {
        visitor.visit_unit()
    }

    #[inline] fn deserialize_tuple_struct<V: Visitor>(&mut self,
                                   _name: &'static str,
                                   len: usize,
                                   visitor: V)
                                   -> DeserializeResult<V::Value> {
        self.deserialize_tuple(len, visitor)
    }

    #[inline] fn deserialize_ignored_any<V: Visitor>(&mut self, _visitor: V) -> DeserializeResult<V::Value> {
        Err(Error::NotSupported)
    }
}

impl<'a> serde::de::VariantVisitor for Deserializer<'a> {
    type Error = Error;

    #[inline] fn visit_variant<V: Deserialize>(&mut self) -> Result<V, Error> {
        let index: u8 = Deserialize::deserialize(self)?;
        let mut deserializer = (index as usize).into_deserializer();
        Ok(serde::Deserialize::deserialize(&mut deserializer)?)
    }

    #[inline] fn visit_unit(&mut self) -> Result<(), Error> {
        Ok(())
    }

    #[inline] fn visit_newtype<T: Deserialize>(&mut self) -> Result<T, Error> {
        serde::de::Deserialize::deserialize(self)
    }

    #[inline] fn visit_tuple<V: Visitor>(&mut self, len: usize, visitor: V) -> Result<V::Value, Error> {
        serde::de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    #[inline] fn visit_struct<V: Visitor>(&mut self,
                       fields: &'static [&'static str],
                       visitor: V)
                       -> Result<V::Value, Error> {
        serde::de::Deserializer::deserialize_tuple(self, fields.len(), visitor)
    }
}
