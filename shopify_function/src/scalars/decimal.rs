use std::ops::Deref;

/// Convenience wrapper for converting between Shopify's `Decimal` scalar, which
/// is serialized as a `String`, and Rust's `f64`.
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String"))]
#[cfg_attr(feature = "serde", serde(into = "String"))]
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

#[cfg(feature = "serde")]
impl TryFrom<String> for Decimal {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(value.as_str())
            .map(Self)
            .map_err(|_| "Error parsing decimal: invalid float literal")
    }
}

#[cfg(feature = "miniserde")]
impl TryFrom<String> for Decimal {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        miniserde::json::from_str(value.as_str())
            .map(Self)
            .map_err(|_| "Error parsing decimal: invalid float literal")
    }
}

#[cfg(feature = "serde")]
impl From<Decimal> for String {
    fn from(value: Decimal) -> Self {
        ryu::Buffer::new().format(value.0).to_string()
    }
}

#[cfg(feature = "miniserde")]
impl From<Decimal> for String {
    fn from(value: Decimal) -> Self {
        ryu::Buffer::new().format(value.0).to_string()
    }
}

#[cfg(feature = "miniserde")]
impl miniserde::Serialize for Decimal {
    fn begin(&self) -> miniserde::ser::Fragment<'_> {
        miniserde::ser::Fragment::Str(miniserde::json::to_string(&self.0).into())
    }
}

#[cfg(feature = "miniserde")]
miniserde::make_place!(Place);

#[cfg(feature = "miniserde")]
impl miniserde::de::Visitor for Place<Decimal> {
    fn string(&mut self, s: &str) -> miniserde::Result<()> {
        self.out = Some(Decimal(
            miniserde::json::from_str(s).map_err(|_| miniserde::Error)?,
        ));
        Ok(())
    }
}

#[cfg(feature = "miniserde")]
impl miniserde::Deserialize for Decimal {
    fn begin(out: &mut Option<Self>) -> &mut dyn miniserde::de::Visitor {
        Place::new(out)
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
    #[cfg(feature = "serde")]
    fn test_json_deserialization() {
        let decimal_value = serde_json::json!("123.4");
        let decimal: Decimal =
            serde_json::from_value(decimal_value).expect("Error deserializing from JSON");
        assert_eq!(123.4, decimal.as_f64());
    }

    #[test]
    #[cfg(feature = "serde")]
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
    #[cfg(feature = "serde")]
    fn test_json_serialization() {
        let decimal = Decimal(123.4);
        let json_value = serde_json::to_value(decimal).expect("Error serializing to JSON");
        assert_eq!(serde_json::json!("123.4"), json_value);
    }
}
