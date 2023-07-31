use std::io::Write;

use shopify_function::prelude::*;
use shopify_function::Result;

const FETCH_INPUT: &str = r#"{
  "id": "gid://shopify/Order/1234567890",
  "num": 123,
  "name": "test"
}"#;
const VALIDATE_INPUT: &str = r#"{
  "fetchResult": "result"
}"#;
static mut FUNCTION_OUTPUT: Vec<u8> = vec![];

#[test]
fn test_fetch_export() {
    let expected_result = r#"{"errors":[]}"#;
    fetch::export();
    let actual_result = std::str::from_utf8(unsafe { FUNCTION_OUTPUT.as_slice() }).unwrap();
    assert_eq!(actual_result, expected_result);
}

#[shopify_function_target(
    query_path = "./tests/fixtures/input.graphql",
    schema_path = "./tests/fixtures/schema_with_targets.graphql",
    input_stream = std::io::Cursor::new(FETCH_INPUT.as_bytes().to_vec()),
    output_stream = unsafe { &mut FUNCTION_OUTPUT }
)]
fn fetch(_input: input::ResponseData) -> Result<output::FunctionResult> {
    Ok(output::FunctionResult { errors: vec![] })
}

#[test]
fn test_validate_export() {
    let expected_result = r#"{"errors":[]}"#;
    validate::export();
    let actual_result = std::str::from_utf8(unsafe { FUNCTION_OUTPUT.as_slice() }).unwrap();
    assert_eq!(actual_result, expected_result);
}

#[shopify_function_target(
    query_path = "./tests/fixtures/validate.graphql",
    schema_path = "./tests/fixtures/schema_with_targets.graphql",
    input_stream = std::io::Cursor::new(VALIDATE_INPUT.as_bytes().to_vec()),
    output_stream = unsafe { &mut FUNCTION_OUTPUT }
)]
fn validate(_input: input::ResponseData) -> Result<output::FunctionResult> {
    Ok(output::FunctionResult { errors: vec![] })
}
