use super::*;
use shopify_function::{run_function_with_input, Result};

#[test]
fn test_a() -> Result<()> {
    let result = serde_json::to_string(
        &run_function_with_input(
            target_a,
            r#"
            {
                "id": "gid://shopify/Order/1234567890",
                "num": 123,
                "name": "test"
            }
        "#,
        )
        .unwrap(),
    )?;
    let expected = r#"{"status":200}"#;
    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_function_b() -> Result<()> {
    let result = serde_json::to_string(
        &run_function_with_input(
            function_b,
            r#"
            {
                "id": "gid://shopify/Order/1234567890",
                "aResult": 200
            }
        "#,
        )
        .unwrap(),
    )?;
    let expected = r#"{"name":"new name: \"gid://shopify/Order/1234567890\""}"#;
    assert_eq!(result, expected);
    Ok(())
}
