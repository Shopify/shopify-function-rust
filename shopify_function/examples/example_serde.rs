use shopify_function::prelude::*;
use shopify_function::Result;

#[typegen("examples/schema.graphql")]
mod schema {
    #[query("examples/a.graphql")]
    pub mod a {}

    #[query("examples/b.graphql")]
    pub mod b {}
}

#[shopify_function]
fn target_a(_input: schema::a::Input) -> Result<schema::FunctionTargetAResult> {
    Ok(schema::FunctionTargetAResult { status: Some(200) })
}

#[shopify_function]
fn function_b(input: schema::b::Input) -> Result<schema::FunctionTargetBResult> {
    Ok(schema::FunctionTargetBResult {
        name: Some(format!("new name: \"{}\"", input.id)),
    })
}

#[cfg(test)]
mod tests {
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
        let expected = crate::schema::FunctionTargetBResult {
            name: Some("new name: \"gid://shopify/Order/1234567890\"".to_string()),
        };

        assert_eq!(result, expected);
        Ok(())
    }
}

fn main() {}
