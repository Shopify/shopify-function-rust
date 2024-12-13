use shopify_function::prelude::*;
use shopify_function::Result;

#[typegen("schema.graphql", codec = "miniserde")]
mod schema {
    #[query("a.graphql")]
    pub mod a {}

    #[query("b.graphql")]
    pub mod b {}
}

#[shopify_function(codec = "miniserde")]
fn target_a(_input: schema::a::Input) -> Result<schema::FunctionTargetAResult> {
    Ok(schema::FunctionTargetAResult { status: Some(200) })
}

#[shopify_function(codec = "miniserde")]
fn function_b(input: schema::b::Input) -> Result<schema::FunctionTargetBResult> {
    Ok(schema::FunctionTargetBResult {
        name: Some(format!("new name: \"{}\"", input.id)),
    })
}

#[cfg(test)]
mod tests;
