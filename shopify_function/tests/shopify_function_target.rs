use std::io::Write;

use shopify_function::prelude::*;
use shopify_function::Result;

#[shopify_function_target(
    query_path = "./tests/fixtures/input.graphql",
    schema_path = "./tests/fixtures/schema_with_targets.graphql"
)]
fn fetch(_input: fetch::input::ResponseData) -> Result<fetch::output::FunctionResult> {
    Ok(fetch::output::FunctionResult { errors: vec![] })
}

#[shopify_function_target(
    query_path = "./tests/fixtures/validate.graphql",
    schema_path = "./tests/fixtures/schema_with_targets.graphql"
)]
fn validate(input: validate::input::ResponseData) -> Result<validate::output::FunctionResult> {
    Ok(validate::output::FunctionResult { errors: vec![] })
}
