//! Support for [`serde`] deserialization of Shopify Function inputs.
//!
//! Deserialization of the input normally goes through
//! [`shopify_function_wasm_api::Deserialize`]. This module adapts [`serde`] to that trait, for
//! types that are easier to express with `serde`, such as a simple enum or the shape of a JSON
//! metafield.
//!
//! Add `#[shopify_function(serde)]` to a type that derives both [`serde::Deserialize`] and
//! [`macro@crate::Deserialize`]. The type keeps its own name, so it can be named in the
//! `custom_scalar_overrides` argument of a query:
//!
//! ```
//! use shopify_function::prelude::*;
//! use shopify_function::wasm_api::Deserialize as _;
//!
//! #[derive(serde::Deserialize, Deserialize, PartialEq, Debug)]
//! #[shopify_function(serde)]
//! #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
//! enum Status {
//!     Active,
//!     Archived,
//! }
//!
//! let context = shopify_function::wasm_api::Context::new_with_input(
//!     serde_json::json!("ARCHIVED"),
//! );
//! let value = context.input_get().unwrap();
//!
//! assert_eq!(Status::deserialize(&value).unwrap(), Status::Archived);
//! ```
//!
//! To write the implementation by hand, call [`from_value`].
//!
//! Because the input is read through the Wasm API, all strings are owned. Types that borrow from
//! the input, such as fields with `#[serde(borrow)]`, are not supported.

use crate::wasm_api::{read, Value};
use serde::de::{
    DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor,
};
use std::fmt;

#[doc(no_inline)]
pub use serde::de::DeserializeOwned;

/// Deserializes a value with [`serde`].
///
/// The code that `#[shopify_function(serde)]` generates calls this function, and converts the
/// error into [`shopify_function_wasm_api::read::Error`], which cannot keep the message. Call this
/// function directly to keep the message.
pub fn from_value<T: DeserializeOwned>(value: &Value) -> Result<T, Error> {
    T::deserialize(ValueDeserializer::new(*value))
}

/// An error that can occur when deserializing with [`serde`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    message: String,
}

impl Error {
    fn invalid_type(expected: &str, value: &Value) -> Self {
        Self {
            message: format!(
                "invalid type: expected {expected}, found {}",
                type_name(value)
            ),
        }
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message.as_str())
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl From<Error> for read::Error {
    fn from(_: Error) -> Self {
        // `read::Error` has no variant that can hold a message.
        read::Error::InvalidType
    }
}

fn type_name(value: &Value) -> &'static str {
    if value.is_null() {
        "null"
    } else if value.as_bool().is_some() {
        "boolean"
    } else if value.as_number().is_some() {
        "number"
    } else if value.is_array() {
        "array"
    } else if value.is_obj() {
        "object"
    } else if value.as_error().is_some() {
        "error"
    } else {
        "string"
    }
}

/// A [`serde::Deserializer`] for a value read from the Shopify Function input.
struct ValueDeserializer {
    value: Value,
}

impl ValueDeserializer {
    fn new(value: Value) -> Self {
        Self { value }
    }

    fn as_number(&self, expected: &str) -> Result<f64, Error> {
        self.value
            .as_number()
            .ok_or_else(|| Error::invalid_type(expected, &self.value))
    }

    fn as_string(&self, expected: &str) -> Result<String, Error> {
        self.value
            .as_string()
            .ok_or_else(|| Error::invalid_type(expected, &self.value))
    }

    fn seq_access(&self) -> Result<SeqDeserializer, Error> {
        let len = self
            .value
            .array_len()
            .ok_or_else(|| Error::invalid_type("array", &self.value))?;
        Ok(SeqDeserializer {
            value: self.value,
            len,
            index: 0,
        })
    }

    fn map_access(&self) -> Result<MapDeserializer, Error> {
        let len = self
            .value
            .obj_len()
            .ok_or_else(|| Error::invalid_type("object", &self.value))?;
        Ok(MapDeserializer {
            value: self.value,
            len,
            index: 0,
        })
    }
}

/// Deserializes an integer, which the input represents as a 64-bit float.
macro_rules! deserialize_int {
    ($method:ident, $ty:ty, $visit:ident) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            let number = self.as_number(stringify!($ty))?;
            if number.trunc() != number || number < <$ty>::MIN as f64 || number > <$ty>::MAX as f64
            {
                return Err(<Error as serde::de::Error>::custom(format!(
                    "number {number} is out of range for {}",
                    stringify!($ty)
                )));
            }
            visitor.$visit(number as $ty)
        }
    };
}

impl<'de> serde::Deserializer<'de> for ValueDeserializer {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if self.value.is_null() {
            visitor.visit_unit()
        } else if let Some(boolean) = self.value.as_bool() {
            visitor.visit_bool(boolean)
        } else if let Some(number) = self.value.as_number() {
            visitor.visit_f64(number)
        } else if self.value.is_array() {
            visitor.visit_seq(self.seq_access()?)
        } else if self.value.is_obj() {
            visitor.visit_map(self.map_access()?)
        } else if let Some(string) = self.value.as_string() {
            visitor.visit_string(string)
        } else {
            Err(Error::invalid_type("a supported type", &self.value))
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let boolean = self
            .value
            .as_bool()
            .ok_or_else(|| Error::invalid_type("boolean", &self.value))?;
        visitor.visit_bool(boolean)
    }

    deserialize_int!(deserialize_i8, i8, visit_i8);
    deserialize_int!(deserialize_i16, i16, visit_i16);
    deserialize_int!(deserialize_i32, i32, visit_i32);
    deserialize_int!(deserialize_i64, i64, visit_i64);
    deserialize_int!(deserialize_u8, u8, visit_u8);
    deserialize_int!(deserialize_u16, u16, visit_u16);
    deserialize_int!(deserialize_u32, u32, visit_u32);
    deserialize_int!(deserialize_u64, u64, visit_u64);

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_f32(self.as_number("f32")? as f32)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_f64(self.as_number("f64")?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let string = self.as_string("char")?;
        let mut chars = string.chars();
        match (chars.next(), chars.next()) {
            (Some(char), None) => visitor.visit_char(char),
            _ => Err(<Error as serde::de::Error>::custom(
                "expected a string with a single character",
            )),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_string(self.as_string("string")?)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Error> {
        Err(<Error as serde::de::Error>::custom(
            "bytes are not supported",
        ))
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if self.value.is_null() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if self.value.is_null() {
            visitor.visit_unit()
        } else {
            Err(Error::invalid_type("null", &self.value))
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_seq(self.seq_access()?)
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_map(self.map_access()?)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        if let Some(variant) = self.value.as_string() {
            // A unit variant, which the input represents as a string.
            visitor.visit_enum(variant.into_deserializer())
        } else if self.value.is_obj() {
            // Any other variant, which the input represents as an object with a single property.
            if self.value.obj_len() != Some(1) {
                return Err(<Error as serde::de::Error>::custom(
                    "expected an object with a single property",
                ));
            }
            let variant = self
                .value
                .get_obj_key_at_index(0)
                .ok_or_else(|| Error::invalid_type("object", &self.value))?;
            visitor.visit_enum(EnumDeserializer {
                variant,
                value: self.value.get_at_index(0),
            })
        } else {
            Err(Error::invalid_type("string or object", &self.value))
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }
}

struct SeqDeserializer {
    value: Value,
    len: usize,
    index: usize,
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        if self.index >= self.len {
            return Ok(None);
        }
        let element = self.value.get_at_index(self.index);
        self.index += 1;
        seed.deserialize(ValueDeserializer::new(element)).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len - self.index)
    }
}

struct MapDeserializer {
    value: Value,
    len: usize,
    index: usize,
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        if self.index >= self.len {
            return Ok(None);
        }
        let key = self
            .value
            .get_obj_key_at_index(self.index)
            .ok_or_else(|| Error::invalid_type("object", &self.value))?;
        seed.deserialize(key.into_deserializer()).map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
        let value = self.value.get_at_index(self.index);
        self.index += 1;
        seed.deserialize(ValueDeserializer::new(value))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len - self.index)
    }
}

struct EnumDeserializer {
    variant: String,
    value: Value,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = ValueDeserializer;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Error> {
        let variant = seed.deserialize(self.variant.into_deserializer())?;
        Ok((variant, ValueDeserializer::new(self.value)))
    }
}

impl<'de> VariantAccess<'de> for ValueDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        if self.value.is_null() {
            Ok(())
        } else {
            Err(Error::invalid_type("null", &self.value))
        }
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
        seed.deserialize(self)
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value, Error> {
        serde::Deserializer::deserialize_seq(self, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        serde::Deserializer::deserialize_map(self, visitor)
    }
}
