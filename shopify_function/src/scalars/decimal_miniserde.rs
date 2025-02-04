#![cfg(feature = "miniserde")]

use std::borrow::Cow;

use super::Decimal;
use miniserde::{de::Visitor, json, make_place, ser::Fragment};

make_place!(Place);

impl Visitor for Place<Decimal> {
    fn string(&mut self, s: &str) -> miniserde::Result<()> {
        self.out = Some(Decimal(json::from_str(s)?));
        Ok(())
    }
}

impl miniserde::de::Deserialize for Decimal {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        Place::new(out)
    }
}

impl miniserde::ser::Serialize for Decimal {
    fn begin(&self) -> Fragment {
        Fragment::Str(Cow::Owned(json::to_string(&self.0)))
    }
}

#[cfg(test)]
mod tests {
    use super::Decimal;
    use miniserde::json;

    #[test]
    fn test_json_deserialization() {
        let json_value = "\"123.4\"";
        let result: Decimal = json::from_str(json_value).expect("Error deserializing from JSON");
        assert_eq!(Decimal(123.4), result);
    }

    #[test]
    fn test_json_deserialization_error() {
        let json_value = "\"123.4.5\"";
        let error = json::from_str::<Decimal>(json_value).expect_err("Expected an error");
        assert_eq!("miniserde error", error.to_string());
    }

    #[test]
    fn test_json_serialization() {
        let decimal = Decimal(123.4);
        let result = json::to_string(&decimal);
        assert_eq!("\"123.4\"", result);
    }
}
