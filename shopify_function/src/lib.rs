//! Crate to write Shopify Functions in Rust.
//!
//! TBD...
//!
//! ```compile_fail
//! use shopify_function::{input_query, scalars::*};
//!
//! #[input_query(query_path = "./input.graphql", schema_path = "./schema.graphql")]
//!
//! #[shopify_function]
//! fn function(input: input_query::ResponseData) -> Result<output::FunctionResult> {
//!     /* ... */
//! }
//! ```

pub use serde;
pub use serde_json;
pub use shopify_function_macro::{input_query, shopify_function};

/// Only used for struct generation.
#[doc(hidden)]
pub mod scalars;

/// Convenience alias for `anyhow::Result`.
pub type Result<T> = anyhow::Result<T>;

/// Runs the given function `f` with the invocation payload, returning
/// the deserialized output.
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

#[cfg(test)]
mod tests {}
