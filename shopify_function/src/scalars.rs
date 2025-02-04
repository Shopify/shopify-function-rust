mod decimal;

#[cfg(feature = "miniserde")]
mod decimal_miniserde;

pub type Boolean = bool;
pub type Float = f64;
pub type Int = i64;
pub type ID = String;
pub type JSON = serde_json::Value;
pub use decimal::Decimal;
pub type Void = ();
pub type URL = String;
pub type Handle = String;

pub type Date = String;
pub type DateTime = String;
pub type DateTimeWithoutTimezone = String;
pub type TimeWithoutTimezone = String;
