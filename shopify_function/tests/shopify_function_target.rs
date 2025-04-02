use shopify_function::prelude::*;
use shopify_function::Result;

const TARGET_A_INPUT: &str = r#"{
  "id": "gid://shopify/Order/1234567890",
  "num": 123,
  "name": "test",
  "country": "CA"
}"#;
static mut TARGET_A_OUTPUT: Vec<u8> = vec![];

#[typegen("./tests/fixtures/schema_with_targets.graphql", enums_as_str = ["CountryCode"])]
mod schema_with_targets {
    #[query("./tests/fixtures/input.graphql")]
    pub mod target_a {}

    #[query("./tests/fixtures/b.graphql")]
    pub mod target_b {}
}

#[test]
fn test_target_a_export() {
    let expected_result = r#"{"status":200}"#;
    target_a_export();
    let actual_result = std::str::from_utf8(unsafe { TARGET_A_OUTPUT.as_slice() }).unwrap();
    assert_eq!(actual_result, expected_result);
}

#[shopify_function(
  // Implicit target = "test.target-a"
  input_stream = std::io::Cursor::new(TARGET_A_INPUT.as_bytes().to_vec()),
  output_stream = unsafe { &mut TARGET_A_OUTPUT }
)]
fn target_a(
    input: schema_with_targets::target_a::Input,
) -> Result<schema_with_targets::FunctionTargetAResult> {
    if input.country != Some("CA".to_string()) {
        panic!("Expected CountryCode to be the CA String")
    }
    Ok(schema_with_targets::FunctionTargetAResult { status: Some(200) })
}

const TARGET_B_INPUT: &str = r#"{
  "id": "gid://shopify/Order/1234567890",
  "targetAResult": 200
}"#;
static mut TARGET_B_OUTPUT: Vec<u8> = vec![];

#[test]
fn test_mod_b_export() {
    let expected_result = r#"{"name":"new name: gid://shopify/Order/1234567890","country":"CA"}"#;
    some_function_export();
    let actual_result = std::str::from_utf8(unsafe { TARGET_B_OUTPUT.as_slice() }).unwrap();
    assert_eq!(actual_result, expected_result);
}

#[shopify_function(
  input_stream = std::io::Cursor::new(TARGET_B_INPUT.as_bytes().to_vec()),
  output_stream = unsafe { &mut TARGET_B_OUTPUT },
)]
fn some_function(
    input: schema_with_targets::target_b::Input,
) -> Result<schema_with_targets::FunctionTargetBResult> {
    Ok(schema_with_targets::FunctionTargetBResult {
        name: Some(format!("new name: {}", input.id)),
        country: Some("CA".to_string()),
    })
}

#[typegen("./tests/fixtures/schema_with_targets.graphql", enums_as_str = ["CountryCode"])]
mod schema_with_country_enum {
    pub type Id = String;
    pub type Void = ();
}
// const _: schema_with_country_enum::CountryCode = schema_with_country_enum::CountryCode::Ca;
