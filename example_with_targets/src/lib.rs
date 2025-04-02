use shopify_function::prelude::*;
use shopify_function::Result;

const TARGET_A_INPUT: &str = r#"{
    "id": "gid://shopify/Order/1234567890",
    "num": 123,
    "name": "test",
    "country": "CA"
  }"#;
  static mut TARGET_A_OUTPUT: Vec<u8> = vec![];

#[typegen("./schema.graphql", enums_as_str = ["CountryCode"])]
mod schema {

    #[query("./a.graphql")]
    pub mod target_a {}

    #[query("./b.graphql")]
    pub mod target_b {}
}

#[shopify_function(
    // Implicit target = "example.target-a"
    // Implicit generated module name = "target_a"
    input_stream = std::io::Cursor::new(TARGET_A_INPUT.as_bytes().to_vec()),
    output_stream = unsafe { &mut TARGET_A_OUTPUT }
)]
fn target_a(
    input: schema::target_a::Input,
) -> Result<schema::FunctionTargetAResult> {
    Ok(schema::FunctionTargetAResult { status: Some(200) })
}

const TARGET_B_INPUT: &str = r#"{
  "id": "gid://shopify/Order/1234567890",
  "targetAResult": 200
}"#;
static mut TARGET_B_OUTPUT: Vec<u8> = vec![];

#[shopify_function(
    // Explicit target if function name does not match target handle
    input_stream = std::io::Cursor::new(TARGET_B_INPUT.as_bytes().to_vec()),
    output_stream = unsafe { &mut TARGET_B_OUTPUT }
)]
fn function_b(input: schema::target_b::Input) -> Result<schema::FunctionTargetBResult> {
    Ok(schema::FunctionTargetBResult { name: Some(format!("new name: \"{}\"", input.id)) })
}

#[cfg(test)]
mod tests;
