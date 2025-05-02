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
    Ok(schema::FunctionTargetAResult { status: Some(200) })
}

#[shopify_function]
fn target_b(input: schema::target_b::Input) -> Result<schema::FunctionTargetBResult> {
    Ok(schema::FunctionTargetBResult {
        name: Some(format!("new name: \"{}\"", input.id())),
    })
}

fn main() {
    eprintln!("Invoke a named import");
    std::process::exit(1);
}

#[cfg(test)]
mod tests;
