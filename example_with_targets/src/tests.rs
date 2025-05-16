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
