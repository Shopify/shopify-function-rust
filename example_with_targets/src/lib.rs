use shopify_function::prelude::*;
use shopify_function::Result;

// Implicit export = "target_a"
#[shopify_function_target(query_path = "a.graphql", schema_path = "schema.graphql")]
fn target_a(_input: target_a::input::ResponseData) -> Result<target_a::output::FunctionTargetAResult> {
    Ok(target_a::output::FunctionTargetAResult { status: Some(200) })
}

// Explicit export = "target_b"
#[shopify_function_target(export = "target_b", query_path = "b.graphql", schema_path = "schema.graphql")]
fn function_b(input: target_b::input::ResponseData) -> Result<target_b::output::FunctionTargetBResult> {
    Ok(target_b::output::FunctionTargetBResult {
        name: Some(format!("new name: \"{}\"", input.id)),
    })
}

#[cfg(test)]
mod tests;
