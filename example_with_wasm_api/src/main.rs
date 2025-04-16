use shopify_function::prelude::*;

#[typegen("schema.graphql")]
mod schema {
    #[query("input.graphql")]
    pub mod input {}
}

#[shopify_function]
fn run(input: schema::input::Input) -> shopify_function::Result<schema::FunctionRunResult> {
    let mut errors = vec![];

    if input.cart().lines().iter().any(|line| line.quantity() > &1) {
        errors.push(schema::FunctionError {
            localized_message: "Quantity must be 1".to_string(),
            target: "cart.lines.quantity".to_string(),
        });
    }

    Ok(schema::FunctionRunResult { errors })
}

fn main() {
    eprintln!("Please invoke a named export");
    std::process::exit(1);
}
