use shopify_function::prelude::*;
use shopify_function::Result;

const FUNCTION_INPUT: &str = r#"{
  "id": "gid://shopify/Order/1234567890",
  "num": 123,
  "name": "test"
}"#;
static mut FUNCTION_OUTPUT: Vec<u8> = vec![];

#[test]
fn test_function() {
    let expected_result = r#"{"name":"new name: gid://shopify/Order/1234567890"}"#;
    export_main().unwrap();
    let actual_result = std::str::from_utf8(unsafe { FUNCTION_OUTPUT.as_slice() }).unwrap();
    assert_eq!(actual_result, expected_result);
}

#[test]
fn test_json_deserialization() {
    let input = r#"{
        "id": "gid://shopify/Order/1234567890",
        "num": 123,
        "name": "test"
    }"#;

    let parsed: input::ResponseData = serde_json::from_str(input).unwrap();

    assert_eq!(parsed.id, "gid://shopify/Order/1234567890");
    assert_eq!(parsed.num, Some(123));
    assert_eq!(parsed.name, Some("test".to_string()));
}

#[shopify_function(
  export = main,
  query = Input,
  mutation = Output,
  query_path = "./tests/fixtures/input.graphql",
  schema_path = "./tests/fixtures/schema.graphql",
  input_stream = std::io::Cursor::new(FUNCTION_INPUT.as_bytes().to_vec()),
  output_stream = unsafe { &mut FUNCTION_OUTPUT }
)]
fn function(input: input::ResponseData) -> Result<output::FunctionResult> {
    Ok(output::FunctionResult {
        name: Some(format!("new name: {}", input.id)),
    })
}

#[shopify_function(
  export = foo,
  query = FooInput,
  mutation = FooOutput,
  query_path = "./tests/fixtures/input.graphql",
  schema_path = "./tests/fixtures/schema.graphql",
  input_stream = std::io::Cursor::new(FUNCTION_INPUT.as_bytes().to_vec()),
  output_stream = unsafe { &mut FUNCTION_OUTPUT }
)]
fn foo(input: input::ResponseData) -> Result<output::FunctionResult> {
    Ok(output::FunctionResult {
        name: Some(format!("new name: {}", input.id)),
    })
}
