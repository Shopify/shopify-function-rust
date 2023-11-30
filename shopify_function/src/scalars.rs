mod decimal;

pub type Boolean = bool;
pub type Float = f64;
pub type Int = i64;
pub type ID = String;
pub use decimal::Decimal;
pub type Void = ();
pub type URL = String;
pub type Handle = String;

pub type Date = String;
pub type DateTime = String;
pub type DateTimeWithoutTimezone = String;
pub type TimeWithoutTimezone = String;
