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

pub use shopify_function_macro::shopify_function;

pub mod scalars;

pub mod prelude {
    pub use crate::scalars::*;
    pub use shopify_function_macro::{shopify_function, typegen, Deserialize};
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Runs the given function `f` with the invocation payload, returning the
/// deserialized output. This function is provided as a helper when writing
/// tests.
#[cfg(not(target_family = "wasm"))]
pub fn run_function_with_input<F, P: wasm_api::Deserialize, O>(f: F, payload: &str) -> Result<O>
where
    F: Fn(P) -> Result<O>,
{
    let parsed_json: serde_json::Value = serde_json::from_str(payload)?;
    let context = wasm_api::Context::new_with_input(parsed_json);
    let input = wasm_api::Deserialize::deserialize(&context.input_get().unwrap()).unwrap();
    f(input)
}

pub use serde;
pub use serde_json;

pub use shopify_function_wasm_api as wasm_api;

#[cfg(test)]
mod tests {}
