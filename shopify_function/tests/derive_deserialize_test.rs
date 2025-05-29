use shopify_function::prelude::*;
use shopify_function::wasm_api::Deserialize;

#[derive(Deserialize, PartialEq, Debug)]
#[shopify_function(rename_all = "camelCase")]
struct TestStruct {
    field_a: String,
    field_b: i32,
}

#[test]
fn test_derive_deserialize() {
    let context = shopify_function::wasm_api::Context::new_with_input(serde_json::json!({
        "fieldA": "test",
        "fieldB": 1,
    }));
    let root_value = context.input_get().unwrap();

    let input = TestStruct::deserialize(&root_value).unwrap();
    assert_eq!(
        input,
        TestStruct {
            field_a: "test".to_string(),
            field_b: 1
        }
    );
}

#[test]
fn test_derive_deserialize_error() {
    let context = shopify_function::wasm_api::Context::new_with_input(serde_json::json!({}));
    let root_value = context.input_get().unwrap();

    TestStruct::deserialize(&root_value).unwrap_err();
}

#[derive(Deserialize, PartialEq, Debug)]
#[shopify_function(rename_all = "camelCase")]
struct TestStructWithRename {
    #[shopify_function(rename = "customFieldName")]
    field_one: String,
    field_two: i32,
    #[shopify_function(rename = "ANOTHER_CUSTOM_NAME")]
    field_three: bool,
}

#[test]
fn test_derive_deserialize_with_field_rename() {
    let context = shopify_function::wasm_api::Context::new_with_input(serde_json::json!({
        "customFieldName": "renamed field",
        "fieldTwo": 42,
        "ANOTHER_CUSTOM_NAME": true
    }));
    let root_value = context.input_get().unwrap();

    let input = TestStructWithRename::deserialize(&root_value).unwrap();
    assert_eq!(
        input,
        TestStructWithRename {
            field_one: "renamed field".to_string(),
            field_two: 42,
            field_three: true
        }
    );
}

#[test]
fn test_field_rename_takes_precedence_over_rename_all() {
    // Test that field-level rename overrides struct-level rename_all
    let context = shopify_function::wasm_api::Context::new_with_input(serde_json::json!({
        "customFieldName": "correct",
        "fieldOne": "incorrect", // This should be ignored
        "fieldTwo": 10,
        "ANOTHER_CUSTOM_NAME": false
    }));
    let root_value = context.input_get().unwrap();

    let input = TestStructWithRename::deserialize(&root_value).unwrap();
    assert_eq!(input.field_one, "correct");
    assert_eq!(input.field_two, 10);
    assert_eq!(input.field_three, false);
}

#[derive(Deserialize, PartialEq, Debug)]
struct TestStructNoRenameAll {
    #[shopify_function(rename = "different_name")]
    original_name: String,
    unchanged_field: i32,
}

#[test]
fn test_field_rename_without_rename_all() {
    let context = shopify_function::wasm_api::Context::new_with_input(serde_json::json!({
        "different_name": "works",
        "unchanged_field": 99
    }));
    let root_value = context.input_get().unwrap();

    let input = TestStructNoRenameAll::deserialize(&root_value).unwrap();
    assert_eq!(
        input,
        TestStructNoRenameAll {
            original_name: "works".to_string(),
            unchanged_field: 99
        }
    );
}

#[derive(Deserialize, PartialEq, Debug, Default)]
struct TestValidAttributes {
    #[shopify_function(rename = "custom")]
    renamed_field: String,

    #[shopify_function(default)]
    default_field: Option<i32>,

    // Multiple attributes on same field
    #[shopify_function(rename = "both", default)]
    renamed_and_default: String,
}

#[test]
fn test_valid_attributes_combination() {
    let context = shopify_function::wasm_api::Context::new_with_input(serde_json::json!({
        "custom": "renamed value",
        // default_field is missing - should use default
        "both": null // should use default for null
    }));
    let root_value = context.input_get().unwrap();

    let input = TestValidAttributes::deserialize(&root_value).unwrap();
    assert_eq!(input.renamed_field, "renamed value");
    assert_eq!(input.default_field, None);
    assert_eq!(input.renamed_and_default, String::default());
}
