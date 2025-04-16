use shopify_function::prelude::*;

#[typegen("./tests/fixtures/schema_with_targets.graphql", enums_as_str = ["CountryCode"])]
mod schema_with_targets {
    #[query("./tests/fixtures/input.graphql")]
    pub mod target_a {}
}

#[test]
fn test_serialize_enum() {}
