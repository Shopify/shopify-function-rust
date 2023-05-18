use serde::Serialize;
use shopify_function::prelude::*;
use shopify_function::Result;

generate_types!(
    query_path = "./tests/fixtures/input.graphql",
    schema_path = "./tests/fixtures/schema.graphql"
);

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

const FUNCTION_INPUT: &str = r#"{
    "id": "gid://shopify/Order/1234567890",
    "num": 123,
    "name": "test"
}"#;

#[test]
fn test_function() {
    main().unwrap();
}

#[shopify_function(
    input_stream = std::io::Cursor::new(FUNCTION_INPUT.as_bytes().to_vec())
)]
fn my_function(input: input::ResponseData) -> Result<output::FunctionResult> {
    Ok(output::FunctionResult {
        name: Some(format!("new name: {}", input.id)),
    })
}
