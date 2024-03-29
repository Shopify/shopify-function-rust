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
    let expected = crate::target_a::output::FunctionTargetAResult { status: Some(200) };
    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_function_b() -> Result<()> {
    let result = run_function_with_input(
        function_b,
        r#"
            {
                "id": "gid://shopify/Order/1234567890",
                "aResult": 200
            }
        "#,
    )?;
    let expected = crate::mod_b::output::FunctionTargetBResult {
        name: Some("new name: \"gid://shopify/Order/1234567890\"".to_string()),
    };

    assert_eq!(result, expected);
    Ok(())
}
