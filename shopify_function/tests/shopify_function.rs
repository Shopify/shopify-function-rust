use shopify_function::prelude::*;
use shopify_function::Result;

const FUNCTION_INPUT: &str = r#"{
  "id": "gid://shopify/Order/1234567890",
  "num": 123,
  "name": "test"
}"#;
static mut FUNCTION_OUTPUT: Vec<u8> = vec![];

generate_types!(
    query_path = "./tests/fixtures/input.graphql",
    schema_path = "./tests/fixtures/schema.graphql"
);

#[test]
fn test_function() {
    let expected_result = r#"{"name":"new name: gid://shopify/Order/1234567890"}"#;
    main().unwrap();
    let actual_result = std::str::from_utf8(unsafe { FUNCTION_OUTPUT.as_slice() }).unwrap();
    assert_eq!(actual_result, expected_result);
}

#[shopify_function(
  input_stream = std::io::Cursor::new(FUNCTION_INPUT.as_bytes().to_vec()),
  output_stream = unsafe { &mut FUNCTION_OUTPUT }
)]
fn my_function(input: input::ResponseData) -> Result<output::FunctionResult> {
    Ok(output::FunctionResult {
        name: Some(format!("new name: {}", input.id)),
    })
}
