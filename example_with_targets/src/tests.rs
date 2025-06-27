use super::*;
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
fn test_null_field_skipping() -> Result<()> {
    let test_function_none =
        |_input: crate::schema::target_a::Input| -> Result<crate::schema::FunctionTargetAResult> {
            Ok(crate::schema::FunctionTargetAResult {
                status: None, // This should not appear in serialized output
            })
        };

    let test_function_some =
        |_input: crate::schema::target_a::Input| -> Result<crate::schema::FunctionTargetAResult> {
            Ok(crate::schema::FunctionTargetAResult {
                status: Some(200), // This should appear in serialized output
            })
        };

    let test_input = r#"{
        "id": "gid://shopify/Order/1234567890",
        "num": 123,
        "name": "test"
    }"#;

    let result_none = run_function_with_input(test_function_none, test_input)?;
    let result_some = run_function_with_input(test_function_some, test_input)?;

    assert_eq!(result_none.status, None);
    assert_eq!(result_some.status, Some(200));

    Ok(())
}
