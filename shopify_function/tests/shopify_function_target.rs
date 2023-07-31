use shopify_function::prelude::*;
use shopify_function::Result;

const A_INPUT: &str = r#"{
  "id": "gid://shopify/Order/1234567890",
  "num": 123,
  "name": "test"
}"#;
static mut A_OUTPUT: Vec<u8> = vec![];

#[test]
fn test_a_export() {
    let expected_result = r#"{"status":200}"#;
    a::export();
    let actual_result = std::str::from_utf8(unsafe { A_OUTPUT.as_slice() }).unwrap();
    assert_eq!(actual_result, expected_result);
}

#[shopify_function_target(
  query_path = "./tests/fixtures/input.graphql",
  schema_path = "./tests/fixtures/schema_with_targets.graphql",
  output_result_type = FunctionAResult,
  input_stream = std::io::Cursor::new(A_INPUT.as_bytes().to_vec()),
  output_stream = unsafe { &mut A_OUTPUT }
)]
fn a(_input: input::ResponseData) -> Result<output::FunctionAResult> {
    Ok(output::FunctionAResult {
        status: Some(200),
    })
}

const B_INPUT: &str = r#"{
  "id": "gid://shopify/Order/1234567890",
  "aResult": 200
}"#;
static mut B_OUTPUT: Vec<u8> = vec![];

#[test]
fn test_b_export() {
    let expected_result = r#"{"name":"new name: gid://shopify/Order/1234567890"}"#;
    b::export();
    let actual_result = std::str::from_utf8(unsafe { B_OUTPUT.as_slice() }).unwrap();
    assert_eq!(actual_result, expected_result);
}

#[shopify_function_target(
  query_path = "./tests/fixtures/b.graphql",
  schema_path = "./tests/fixtures/schema_with_targets.graphql",
  output_result_type = FunctionBResult,
  input_stream = std::io::Cursor::new(B_INPUT.as_bytes().to_vec()),
  output_stream = unsafe { &mut B_OUTPUT }
)]
fn b(input: input::ResponseData) -> Result<output::FunctionBResult> {
    Ok(output::FunctionBResult {
        name: Some(format!("new name: {}", input.id)),
    })
}
