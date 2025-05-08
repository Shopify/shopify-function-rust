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
