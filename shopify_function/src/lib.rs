//! Crate to write Shopify Functions in Rust.
//!
//! This crate provides some convenience when building Shopify Functions using
//! Rust. The crate takes care of generating the required Rust structs to handle
//! the data types being passed between Shopify and the Function. The crate also
//! takes care of deserializing the input data and serializing the output data.
//!
//! ```ignore
//! use shopify_function::prelude::*
//!
//! generate_types!(query_path = "./input.graphql", schema_path = "./schema.graphql");
//!
//! #[shopify_function]
//! fn function(input: input::ResponseData) -> Result<output::FunctionResult> {
//!     /* ... */
//! }
//! ```

#[cfg(all(feature = "serde", feature = "miniserde"))]
compile_error!("Cannot use both serde and miniserde features");

pub use shopify_function_macro::shopify_function;

pub mod scalars;

pub mod prelude {
    pub use crate::scalars::*;
    pub use shopify_function_macro::{shopify_function, typegen};
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(feature = "serde")]
/// Runs the given function `f` with the invocation payload, returning the
/// deserialized output. This function is provided as a helper when writing
/// tests.
pub fn run_function_with_input<'a, F, P: serde::Deserialize<'a>, O>(
    f: F,
    payload: &'a str,
) -> Result<O>
where
    F: Fn(P) -> Result<O>,
{
    let parsed_payload: P = serde_json::from_str(payload)?;
    f(parsed_payload)
}

#[cfg(feature = "miniserde")]
/// Runs the given function `f` with the invocation payload, returning the
/// deserialized output. This function is provided as a helper when writing
/// tests.
pub fn run_function_with_input<F, P: miniserde::Deserialize, O>(f: F, payload: &str) -> Result<O>
where
    F: Fn(P) -> Result<O>,
{
    let parsed_payload: P = miniserde::json::from_str(payload)?;
    f(parsed_payload)
}

#[cfg(feature = "serde")]
pub use serde;
#[cfg(feature = "serde")]
pub use serde_json;

#[cfg(feature = "miniserde")]
pub use miniserde;

#[cfg(test)]
mod tests {}
