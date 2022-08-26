pub use serde;
pub use serde_json;
pub use shopify_rust_function_macro::{function_config, input_query, shopify_function};

pub mod discounts;

pub type Result<T> = anyhow::Result<T>;

pub fn parse_config<'a, T: Default + serde::Deserialize<'a> + PartialEq>(
    config: Option<&'a str>,
) -> T {
    config
        .and_then(|s| serde_json::from_str(s.as_ref()).ok())
        .unwrap_or_default()
}

pub fn run_function_with_input<'a, F, P: serde::Deserialize<'a>>(
    f: F,
    payload: &'a str,
) -> Result<discounts::Output>
where
    F: Fn(P) -> Result<discounts::Output>,
{
    let parsed_payload: P = serde_json::from_str(payload)?;
    f(parsed_payload)
}

#[cfg(test)]
mod tests {}
