use std::process;

use shopify_function::prelude::*;
use shopify_function::Result;

#[derive(Deserialize)]
#[shopify_function(rename_all = "camelCase")]
struct Configuration {}

#[typegen("./schema.graphql", enums_as_str = ["CountryCode"])]
mod schema {
    #[query("./a.graphql")]
    pub mod target_a {}

    #[query("./b.graphql")]
    pub mod target_b {}
}

#[shopify_function]
fn target_a(_input: schema::target_a::Input) -> Result<schema::FunctionTargetAResult> {
    log!("In target_a");
    let var = 42;
    log!("With var: {var}");
    log!("With var: {}", var);
    Ok(schema::FunctionTargetAResult { status: Some(200) })
}

#[shopify_function]
fn target_b(input: schema::target_b::Input) -> Result<schema::FunctionTargetBResult> {
    log!("In target_b");
    Ok(schema::FunctionTargetBResult {
        name: Some(format!("new name: \"{}\"", input.id())),
        operations: vec![
            schema::Operation::DoThis(schema::This {
                this_field: "this field".to_string(),
            }),
            schema::Operation::DoThat(schema::That { that_field: 42 }),
        ],
    })
}

#[shopify_function]
fn target_panic(_input: schema::target_a::Input) -> Result<schema::FunctionTargetAResult> {
    panic!("Something went wrong");
}

fn main() {
    process::abort()
}

#[cfg(test)]
mod tests;
