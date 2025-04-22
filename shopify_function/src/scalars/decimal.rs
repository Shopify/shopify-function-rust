use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

/// Convenience wrapper for converting between Shopify's `Decimal` scalar, which
/// is serialized as a `String`, and Rust's `f64`.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct Decimal(pub f64);

impl Decimal {
    /// Access the value as an `f64`
    pub fn as_f64(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(ryu::Buffer::new().format(self.0))
    }
}

impl Deref for Decimal {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for Decimal {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(value.as_str())
            .map(Self)
            .map_err(|_| "Error parsing decimal: invalid float literal")
    }
}

impl From<Decimal> for String {
    fn from(value: Decimal) -> Self {
        ryu::Buffer::new().format(value.0).to_string()
    }
}

impl From<Decimal> for f64 {
    fn from(value: Decimal) -> Self {
        value.0
    }
}

impl From<f64> for Decimal {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl crate::wasm_api::Deserialize for Decimal {
    fn deserialize(value: &crate::wasm_api::Value) -> Result<Self, crate::wasm_api::read::Error> {
        let string_value: String = crate::wasm_api::Deserialize::deserialize(value)?;
        string_value
            .parse()
            .map(Self)
            .map_err(|_| crate::wasm_api::read::Error::InvalidType)
    }
}

impl crate::wasm_api::Serialize for Decimal {
    fn serialize(
        &self,
        context: &mut crate::wasm_api::Context,
    ) -> Result<(), crate::wasm_api::write::Error> {
        crate::wasm_api::Serialize::serialize(self.0.to_string().as_str(), context)
    }
}

#[cfg(test)]
mod tests {
    use super::Decimal;

    #[test]
    fn test_json_deserialization() {
        let decimal_value = serde_json::json!("123.4");
        let decimal: Decimal =
            serde_json::from_value(decimal_value).expect("Error deserializing from JSON");
        assert_eq!(123.4, decimal.as_f64());
    }

    #[test]
    fn test_json_deserialization_error() {
        let decimal_value = serde_json::json!("123.4.5");
        let error =
            serde_json::from_value::<Decimal>(decimal_value).expect_err("Expected an error");
        assert_eq!(
            "Error parsing decimal: invalid float literal",
            error.to_string()
        );
    }

    #[test]
    fn test_json_serialization() {
        let decimal = Decimal(123.4);
        let json_value = serde_json::to_value(decimal).expect("Error serializing to JSON");
        assert_eq!(serde_json::json!("123.4"), json_value);
    }

    #[test]
    fn test_display_formatting() {
        assert_eq!(Decimal(123.45).to_string(), "123.45");
        assert_eq!(Decimal(123.0).to_string(), "123.0");
        assert_eq!(Decimal(0.0).to_string(), "0.0");
        assert_eq!(Decimal(-5.678).to_string(), "-5.678");
    }
}
