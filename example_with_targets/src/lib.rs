use shopify_function::prelude::*;
use shopify_function::Result;

#[shopify_function_target(
  query_path = "./a.graphql",
  schema_path = "./schema.graphql",
  output_result_type = FunctionAResult
)]
fn a(_input: input::ResponseData) -> Result<output::FunctionAResult> {
    Ok(output::FunctionAResult {
        status: Some(200),
    })
}

#[shopify_function_target(
  query_path = "./b.graphql",
  schema_path = "./schema.graphql",
  output_result_type = FunctionBResult
)]
fn b(input: input::ResponseData) -> Result<output::FunctionBResult> {
    Ok(output::FunctionBResult {
        name: Some(format!("new name: \"{}\"", input.id)),
    })
}

#[cfg(test)]
mod tests;
