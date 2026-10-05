use std::process;

use shopify_function::prelude::*;
use shopify_function::scalars::Decimal;
use shopify_function::Result;

#[allow(dead_code)]
#[derive(Deserialize)]
#[shopify_function(rename_all = "camelCase")]
struct Configuration {}

#[typegen("./schema.graphql", enums_as_str = ["CountryCode"], custom_scalar_overrides = {
    "FunctionTargetPrepareResult.variables" => target_run::InputVariables,
})]
mod schema {
    #[query("./a.graphql")]
    pub mod target_a {}

    #[query("./b.graphql")]
    pub mod target_b {}

    #[query("./cart.graphql")]
    pub mod target_cart {}

    #[query("./prepare.graphql")]
    pub mod target_prepare {}

    #[query("./run.graphql")]
    pub mod target_run {}
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

#[shopify_function]
fn target_cart(input: schema::target_cart::Input) -> Result<schema::FunctionTargetCartResult> {
    // Iterate over cart lines and sum quantities - this accesses the `quantity` property
    // multiple times, which would demonstrate any benefit from interning strings
    let total_quantity: i32 = input
        .cart_lines()
        .unwrap_or(&[])
        .iter()
        .map(|line| *line.quantity())
        .sum();

    Ok(schema::FunctionTargetCartResult { total_quantity })
}

#[shopify_function]
fn target_prepare(
    input: schema::target_prepare::Input,
) -> Result<schema::FunctionTargetPrepareResult> {
    let variables = schema::target_run::InputVariables {
        selector: schema::LineSelector::Filter(schema::LineFilter {
            titles: Some(vec![input.id().clone()]),
            r#match: Some(schema::LineMatch::Any),
            minimum_price: Some(Decimal(1.5)),
        }),
        first: Some(10),
        country: None,
        minimum_quantity: None,
        tags: Some(vec![Some("sale".to_string()), None]),
    };

    Ok(schema::FunctionTargetPrepareResult { variables })
}

#[shopify_function]
fn target_run(input: schema::target_run::Input) -> Result<schema::FunctionTargetRunResult> {
    Ok(schema::FunctionTargetRunResult {
        line_count: input.matching_lines().len() as i32,
    })
}

fn main() {
    log!("Invoke a named export");
    process::abort()
}

#[cfg(test)]
mod tests;
