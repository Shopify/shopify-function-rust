use super::*;
use shopify_function::{run_function_with_input, Result};

#[test]
fn test_a() -> Result<()> {
    let result = run_function_with_input(
        a::a,
        r#"
            {
                "id": "gid://shopify/Order/1234567890",
                "num": 123,
                "name": "test"
            }
        "#,
    )?;
    let expected = crate::a::output::FunctionAResult { status: Some(200) };
    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_b() -> Result<()> {
    let result = run_function_with_input(
        b::b,
        r#"
            {
                "id": "gid://shopify/Order/1234567890",
                "aResult": 200
            }
        "#,
    )?;
    let expected = crate::b::output::FunctionBResult {
        name: Some("new name: \"gid://shopify/Order/1234567890\"".to_string()),
    };

    assert_eq!(result, expected);
    Ok(())
}
