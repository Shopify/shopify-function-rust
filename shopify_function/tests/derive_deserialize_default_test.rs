use shopify_function::prelude::*;
use shopify_function::wasm_api::Deserialize;

#[derive(Deserialize, PartialEq, Debug, Default)]
#[shopify_function(rename_all = "camelCase")]
struct TestStructWithDefault {
    // Field with default attribute - will use Default implementation when null
    #[shopify_function(default)]
    field_a: String,

    // Field with default attribute - will use Default implementation when null
    #[shopify_function(default)]
    field_b: i32,

    // Field without default attribute - will error when null
    field_c: bool,
}

// Define a struct with more complex default types
#[derive(Deserialize, PartialEq, Debug, Default)]
struct TestComplexDefaults {
    // Standard primitive types
    #[shopify_function(default)]
    integer: i32,

    #[shopify_function(default)]
    float: f64,

    #[shopify_function(default)]
    string: String,

    #[shopify_function(default)]
    boolean: bool,

    // Collection types
    #[shopify_function(default)]
    vector: Vec<i32>,

    #[shopify_function(default)]
    option: Option<String>,
}

#[test]
fn test_derive_deserialize_with_default() {
    // Test with all fields present
    let context = shopify_function::wasm_api::Context::new_with_input(serde_json::json!({
        "fieldA": "test",
        "fieldB": 1,
        "fieldC": true
    }));
    let root_value = context.input_get().unwrap();

    let input = TestStructWithDefault::deserialize(&root_value).unwrap();
    assert_eq!(
        input,
        TestStructWithDefault {
            field_a: "test".to_string(),
            field_b: 1,
            field_c: true
        }
    );

    // Test with default fields set to null
    let context = shopify_function::wasm_api::Context::new_with_input(serde_json::json!({
        "fieldA": null,
        "fieldB": null,
        "fieldC": true
    }));
    let root_value = context.input_get().unwrap();

    let input = TestStructWithDefault::deserialize(&root_value).unwrap();
    assert_eq!(
        input,
        TestStructWithDefault {
            field_a: String::default(), // Empty string
            field_b: i32::default(),    // 0
            field_c: true
        }
    );

    // Test with default fields missing
    let context = shopify_function::wasm_api::Context::new_with_input(serde_json::json!({
        "fieldC": true
    }));
    let root_value = context.input_get().unwrap();

    // Our implementation is handling missing fields correctly by treating them as null values
    let input = TestStructWithDefault::deserialize(&root_value).unwrap();
    assert_eq!(
        input,
        TestStructWithDefault {
            field_a: String::default(), // Empty string
            field_b: i32::default(),    // 0
            field_c: true
        }
    );
}

#[test]
fn test_derive_deserialize_complex_defaults() {
    // Test with all fields set to null
    let context = shopify_function::wasm_api::Context::new_with_input(serde_json::json!({
        "integer": null,
        "float": null,
        "string": null,
        "boolean": null,
        "vector": null,
        "option": null
    }));
    let root_value = context.input_get().unwrap();

    let input = TestComplexDefaults::deserialize(&root_value).unwrap();
    assert_eq!(
        input,
        TestComplexDefaults {
            integer: 0,
            float: 0.0,
            string: String::new(),
            boolean: false,
            vector: Vec::new(),
            option: None,
        }
    );

    // Test with values provided
    let context = shopify_function::wasm_api::Context::new_with_input(serde_json::json!({
        "integer": 42,
        "float": 3.19,
        "string": "hello",
        "boolean": true,
        "vector": [1, 2, 3],
        "option": "some value"
    }));
    let root_value = context.input_get().unwrap();

    let input = TestComplexDefaults::deserialize(&root_value).unwrap();
    assert_eq!(
        input,
        TestComplexDefaults {
            integer: 42,
            float: 3.19,
            string: "hello".to_string(),
            boolean: true,
            vector: vec![1, 2, 3],
            option: Some("some value".to_string()),
        }
    );
}

#[test]
fn test_missing_non_default_field() {
    // Missing a required field (field_c)
    let context = shopify_function::wasm_api::Context::new_with_input(serde_json::json!({
        "fieldA": "test",
        "fieldB": 1
    }));
    let root_value = context.input_get().unwrap();

    // Should fail because field_c is required
    TestStructWithDefault::deserialize(&root_value).unwrap_err();
}
