use serde::{Deserialize, Serialize};
use std::{ops::Deref, str::FromStr};

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

impl Deref for Decimal {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for Decimal {
    type Error = std::num::ParseFloatError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        f64::from_str(value.as_str()).map(Self)
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
    fn test_json_serialization() {
        let decimal = Decimal(123.4);
        let json_value = serde_json::to_value(decimal).expect("Error serializing to JSON");
        assert_eq!(serde_json::json!("123.4"), json_value);
    }
}
