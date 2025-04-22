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
    pub use shopify_function_macro::{shopify_function, typegen};
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

pub use serde;
pub use serde_json;

pub use shopify_function_wasm_api as wasm_api;

pub struct Iter<T: wasm_api::Deserialize> {
    value: wasm_api::Value,
    index: usize,
    len: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T: wasm_api::Deserialize> Clone for Iter<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            index: self.index,
            len: self.len,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: wasm_api::Deserialize> wasm_api::Deserialize for Iter<T> {
    fn deserialize(value: &wasm_api::Value) -> std::result::Result<Self, wasm_api::read::Error> {
        if let Some(len) = value.array_len() {
            Ok(Self {
                value: *value,
                index: 0,
                len,
                _marker: std::marker::PhantomData,
            })
        } else {
            Err(wasm_api::read::Error::InvalidType)
        }
    }
}

impl<T: wasm_api::Deserialize> Iterator for Iter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.index >= self.len {
            return None;
        }
        let value = self.value.get_at_index(self.index);
        self.index += 1;
        // need to unwrap here because we don't want programs to need
        // to handle errors here
        Some(T::deserialize(&value).unwrap())
    }
}

#[cfg(test)]
mod tests {}
