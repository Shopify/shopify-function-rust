use shopify_function::prelude::*;

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
