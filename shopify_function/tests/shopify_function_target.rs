use shopify_function::prelude::*;
use shopify_function::Result;

const TARGET_A_INPUT: &str = r#"{
  "id": "gid://shopify/Order/1234567890",
  "num": 123,
  "name": "test"
}"#;
static mut TARGET_A_OUTPUT: Vec<u8> = vec![];

#[test]
fn test_target_a_export() {
    let expected_result = r#"{"status":200}"#;
    target_a::export();
    let actual_result = std::str::from_utf8(unsafe { TARGET_A_OUTPUT.as_slice() }).unwrap();
    assert_eq!(actual_result, expected_result);
}

#[shopify_function_target(
  // Implicit target = "test.target-a"
  query_path = "./tests/fixtures/input.graphql",
  schema_path = "./tests/fixtures/schema_with_targets.graphql",
  input_stream = std::io::Cursor::new(TARGET_A_INPUT.as_bytes().to_vec()),
  output_stream = unsafe { &mut TARGET_A_OUTPUT }
)]
fn target_a(
    _input: target_a::input::ResponseData,
) -> Result<target_a::output::FunctionTargetAResult> {
    Ok(target_a::output::FunctionTargetAResult { status: Some(200) })
}

const TARGET_B_INPUT: &str = r#"{
  "id": "gid://shopify/Order/1234567890",
  "targetAResult": 200
}"#;
static mut TARGET_B_OUTPUT: Vec<u8> = vec![];

#[test]
fn test_mod_b_export() {
    let expected_result = r#"{"name":"new name: gid://shopify/Order/1234567890"}"#;
    mod_b::export();
    let actual_result = std::str::from_utf8(unsafe { TARGET_B_OUTPUT.as_slice() }).unwrap();
    assert_eq!(actual_result, expected_result);
}

#[shopify_function_target(
  target = "test.target-b",
  module_name = "mod_b",
  query_path = "./tests/fixtures/b.graphql",
  schema_path = "./tests/fixtures/schema_with_targets.graphql",
  input_stream = std::io::Cursor::new(TARGET_B_INPUT.as_bytes().to_vec()),
  output_stream = unsafe { &mut TARGET_B_OUTPUT }
)]
fn some_function(
    input: mod_b::input::ResponseData,
) -> Result<mod_b::output::FunctionTargetBResult> {
    Ok(mod_b::output::FunctionTargetBResult {
        name: Some(format!("new name: {}", input.id)),
    })
}
