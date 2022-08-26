pub use serde;
pub use serde_json;
pub use shopify_rust_function_macro::shopify_function;

pub mod discounts;

pub fn parse_config<'a, T: Default + serde::Deserialize<'a>>(config: Option<&'a str>) -> T {
    config
        .and_then(|s| serde_json::from_str(s.as_ref()).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {}
