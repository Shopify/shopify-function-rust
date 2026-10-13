use super::*;
use shopify_function::wasm_api::{Context, Serialize};
use shopify_function::{run_function_with_input, Result};

#[test]
fn test_a() -> Result<()> {
    let result = run_function_with_input(
        target_a,
        r#"
            {
                "id": "gid://shopify/Order/1234567890",
                "num": 123,
                "name": "test"
            }
        "#,
    )?;
    let expected = crate::schema::FunctionTargetAResult { status: Some(200) };
    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_target_b() -> Result<()> {
    let result = run_function_with_input(
        target_b,
        r#"
            {
                "id": "gid://shopify/Order/1234567890",
                "aResult": 200
            }
        "#,
    )?;
    let expected = crate::schema::FunctionTargetBResult {
        name: Some("new name: \"gid://shopify/Order/1234567890\"".to_string()),
        operations: vec![
            crate::schema::Operation::DoThis(crate::schema::This {
                this_field: "this field".to_string(),
            }),
            crate::schema::Operation::DoThat(crate::schema::That { that_field: 42 }),
        ],
    };

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_input_object_serialization_omits_none_fields_and_keeps_required_fields() -> Result<()> {
    let result = serialize_to_json(&crate::schema::SerializationProbe {
        optional_value: None,
        defaulted_value: None,
        required_value: 1,
    })?;

    assert_eq!(serde_json::json!({ "requiredValue": 1 }), result);
    Ok(())
}

#[test]
fn test_input_object_serialization_includes_some_fields() -> Result<()> {
    let result = serialize_to_json(&crate::schema::SerializationProbe {
        optional_value: Some(200),
        defaulted_value: Some(201),
        required_value: 1,
    })?;

    assert_eq!(
        serde_json::json!({ "optionalValue": 200, "defaultedValue": 201, "requiredValue": 1 }),
        result
    );
    Ok(())
}

#[test]
fn test_one_of_input_object_serialization_writes_active_variant() -> Result<()> {
    let result = serialize_to_json(&crate::schema::Operation::DoThis(crate::schema::This {
        this_field: "this field".to_string(),
    }))?;

    assert_eq!(
        serde_json::json!({ "doThis": { "thisField": "this field" } }),
        result
    );
    Ok(())
}

#[test]
fn test_target_prepare_returns_the_variables_run_declares() -> Result<()> {
    let result = run_function_with_input(
        target_prepare,
        r#"
            {
                "id": "gid://shopify/CartLine/1"
            }
        "#,
    )?;

    // `None` omits `country` and `minimumQuantity`.
    assert_eq!(
        serde_json::json!({
            "variables": {
                "selector": {
                    "filter": {
                        "titles": ["gid://shopify/CartLine/1"],
                        "match": "ANY",
                        "minimumPrice": "1.5"
                    }
                },
                "first": 10,
                "tags": ["sale", null]
            }
        }),
        serialize_to_json(&result)?
    );
    Ok(())
}

#[test]
fn test_variables_serialization_omits_none_variables() -> Result<()> {
    let variables = crate::schema::target_run::InputVariables {
        selector: crate::schema::LineSelector::Id("gid://shopify/CartLine/1".to_string()),
        first: None,
        country: Some("CA".to_string()),
        minimum_quantity: Some(2),
        tags: None,
    };

    assert_eq!(
        serde_json::json!({
            "selector": { "id": "gid://shopify/CartLine/1" },
            "country": "CA",
            "minimumQuantity": 2
        }),
        serialize_to_json(&variables)?
    );
    Ok(())
}

#[test]
fn test_target_run_counts_matching_lines() -> Result<()> {
    let result = run_function_with_input(
        target_run,
        r#"
            {
                "matchingLines": [
                    { "id": "gid://shopify/CartLine/1" },
                    { "id": "gid://shopify/CartLine/2" }
                ]
            }
        "#,
    )?;

    assert_eq!(
        crate::schema::FunctionTargetRunResult { line_count: 2 },
        result
    );
    Ok(())
}

fn serialize_to_json<T: Serialize + ?Sized>(value: &T) -> Result<serde_json::Value> {
    let mut context = Context::new_with_input(serde_json::json!({}));
    value.serialize(&mut context)?;
    Ok(context.finalize_output_and_return()?)
}
