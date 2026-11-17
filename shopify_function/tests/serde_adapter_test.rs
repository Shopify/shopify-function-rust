use shopify_function::prelude::*;
use shopify_function::serde_adapter::from_value;
use shopify_function::wasm_api::Deserialize as _;
use std::collections::BTreeMap;

fn value_from(input: serde_json::Value) -> shopify_function::wasm_api::Value {
    let context = shopify_function::wasm_api::Context::new_with_input(input);
    context.input_get().unwrap()
}

#[derive(serde::Deserialize, Deserialize, PartialEq, Debug)]
#[shopify_function(serde)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Active,
    ArchivedByMerchant,
    #[serde(rename = "custom-name")]
    Custom,
    #[serde(other)]
    Unknown,
}

#[test]
fn test_simple_enum() {
    for (input, expected) in [
        ("ACTIVE", Status::Active),
        ("ARCHIVED_BY_MERCHANT", Status::ArchivedByMerchant),
        ("custom-name", Status::Custom),
        ("SOMETHING_ELSE", Status::Unknown),
    ] {
        let value = value_from(serde_json::json!(input));
        assert_eq!(Status::deserialize(&value).unwrap(), expected);
    }
}

#[derive(serde::Deserialize, Deserialize, PartialEq, Debug)]
#[shopify_function(serde)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Color {
    Red,
    Green,
}

#[test]
fn test_simple_enum_with_unknown_value() {
    let value = value_from(serde_json::json!("BLUE"));
    Color::deserialize(&value).unwrap_err();
}

#[test]
fn test_simple_enum_with_non_string_value() {
    let value = value_from(serde_json::json!(1));
    Color::deserialize(&value).unwrap_err();
}

#[test]
fn test_error_message_is_kept_by_from_value() {
    let value = value_from(serde_json::json!("BLUE"));
    let error = from_value::<Color>(&value).unwrap_err();
    assert!(
        error.message().contains("BLUE"),
        "unexpected message: {error}"
    );
}

/// A type with the shape of a JSON metafield, which is the main reason to use `serde`.
#[derive(serde::Deserialize, Deserialize, PartialEq, Debug)]
#[shopify_function(serde)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub name: String,
    pub quantity: i32,
    pub price: f64,
    pub enabled: bool,
    pub color: Option<Color>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub attributes: BTreeMap<String, String>,
    pub pair: (i32, String),
}

#[test]
fn test_nested_types() {
    let value = value_from(serde_json::json!({
        "name": "Discount",
        "quantity": 2,
        "price": 10.5,
        "enabled": true,
        "color": "RED",
        // `tags` is missing, so the default is used
        "attributes": { "a": "1", "b": "2" },
        "pair": [1, "one"],
    }));

    assert_eq!(
        Configuration::deserialize(&value).unwrap(),
        Configuration {
            name: "Discount".to_string(),
            quantity: 2,
            price: 10.5,
            enabled: true,
            color: Some(Color::Red),
            tags: vec![],
            attributes: BTreeMap::from([
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string()),
            ]),
            pair: (1, "one".to_string()),
        }
    );
}

#[test]
fn test_missing_field_is_an_error() {
    let value = value_from(serde_json::json!({ "name": "Discount" }));
    let error = from_value::<Configuration>(&value).unwrap_err();
    assert!(
        error.message().contains("quantity"),
        "unexpected message: {error}"
    );
}

#[test]
fn test_number_out_of_range_is_an_error() {
    let value = value_from(serde_json::json!({
        "name": "Discount",
        "quantity": 1.5,
        "price": 1.0,
        "enabled": true,
        "color": null,
        "attributes": {},
        "pair": [1, "one"],
    }));
    from_value::<Configuration>(&value).unwrap_err();
}

#[derive(serde::Deserialize, Deserialize, PartialEq, Debug)]
#[shopify_function(serde)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    Noop,
    Add(i32),
    Move { from: String, to: String },
}

#[test]
fn test_enum_with_fields() {
    let value = value_from(serde_json::json!("noop"));
    assert_eq!(Operation::deserialize(&value).unwrap(), Operation::Noop);

    let value = value_from(serde_json::json!({ "add": 2 }));
    assert_eq!(Operation::deserialize(&value).unwrap(), Operation::Add(2));

    let value = value_from(serde_json::json!({ "move": { "from": "a", "to": "b" } }));
    assert_eq!(
        Operation::deserialize(&value).unwrap(),
        Operation::Move {
            from: "a".to_string(),
            to: "b".to_string()
        }
    );
}

#[test]
fn test_untagged_enum_and_json_value() {
    #[derive(serde::Deserialize, Deserialize, PartialEq, Debug)]
    #[shopify_function(serde)]
    #[serde(untagged)]
    enum StringOrNumber {
        Number(f64),
        String(String),
    }

    let value = value_from(serde_json::json!(2.5));
    assert_eq!(
        StringOrNumber::deserialize(&value).unwrap(),
        StringOrNumber::Number(2.5)
    );

    let value = value_from(serde_json::json!("two"));
    assert_eq!(
        StringOrNumber::deserialize(&value).unwrap(),
        StringOrNumber::String("two".to_string())
    );

    // `deserialize_any` also lets a fully dynamic type work
    let value = value_from(serde_json::json!({ "a": [1, true, null, "x"] }));
    assert_eq!(
        from_value::<serde_json::Value>(&value).unwrap(),
        serde_json::json!({ "a": [1.0, true, null, "x"] })
    );
}

#[test]
fn test_generic_type() {
    #[derive(serde::Deserialize, Deserialize, PartialEq, Debug)]
    #[shopify_function(serde)]
    struct Wrapper<T> {
        value: T,
    }

    let value = value_from(serde_json::json!({ "value": "ACTIVE" }));
    assert_eq!(
        Wrapper::<Status>::deserialize(&value).unwrap(),
        Wrapper {
            value: Status::Active
        }
    );
}

#[test]
fn test_function_input_can_use_serde() {
    fn run(input: Configuration) -> shopify_function::Result<i32> {
        Ok(input.quantity)
    }

    let payload = r#"{
        "name": "Discount",
        "quantity": 3,
        "price": 1.0,
        "enabled": true,
        "color": null,
        "attributes": {},
        "pair": [1, "one"]
    }"#;
    let result: i32 = shopify_function::run_function_with_input(run, payload).unwrap();
    assert_eq!(result, 3);
}
