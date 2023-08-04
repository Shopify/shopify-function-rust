use shopify_function::prelude::*;
use shopify_function::Result;

#[shopify_function_target(
    // Implicit target = "example.target-a"
    // Implicit generated module name = "target_a"
    query_path = "a.graphql",
    schema_path = "schema.graphql"
)]
fn target_a(
    _input: target_a::input::ResponseData,
) -> Result<target_a::output::FunctionTargetAResult> {
    Ok(target_a::output::FunctionTargetAResult { status: Some(200) })
}

#[shopify_function_target(
    // Explicit target if function name does not match target handle
    target = "example.target-b",
    // Override generated module name
    module_name = "mod_b",
    query_path = "b.graphql",
    schema_path = "schema.graphql"
)]
fn function_b(input: mod_b::input::ResponseData) -> Result<mod_b::output::FunctionTargetBResult> {
    Ok(mod_b::output::FunctionTargetBResult {
        name: Some(format!("new name: \"{}\"", input.id)),
    })
}

#[cfg(test)]
mod tests;
