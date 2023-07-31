use std::io::Write;

use shopify_function::prelude::*;
use shopify_function::Result;

#[test]
fn test_fetch() {
    fetch::export();
}

#[shopify_function_target(
    query_path = "./tests/fixtures/input.graphql",
    schema_path = "./tests/fixtures/schema_with_targets.graphql"
)]
fn fetch(_input: input::ResponseData) -> Result<output::FunctionResult> {
    Ok(output::FunctionResult { errors: vec![] })
}

#[shopify_function_target(
    query_path = "./tests/fixtures/validate.graphql",
    schema_path = "./tests/fixtures/schema_with_targets.graphql"
)]
fn validate(input: input::ResponseData) -> Result<output::FunctionResult> {
    Ok(output::FunctionResult { errors: vec![] })
}
