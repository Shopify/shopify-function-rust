pub use serde;
pub use serde_json;
pub use shopify_function_macro::{input_query, shopify_function};

pub mod scalars;

pub type Result<T> = anyhow::Result<T>;

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
