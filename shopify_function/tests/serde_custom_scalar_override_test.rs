//! Tests that a type deserialized with `serde` can be the target of a `custom_scalar_overrides`
//! entry, which is the main reason to support `serde`.

use shopify_function::prelude::*;
use shopify_function::wasm_api::Deserialize as _;

/// The shape of a JSON metafield.
#[derive(serde::Deserialize, Deserialize, PartialEq, Debug)]
#[shopify_function(serde)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub minimum_quantity: i32,
    pub strategy: Strategy,
}

#[derive(serde::Deserialize, Deserialize, PartialEq, Debug)]
#[shopify_function(serde)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Strategy {
    Cheapest,
    MostExpensive,
    #[serde(other)]
    Unknown,
}

#[typegen([
    scalar Json

    type Shop {
        name: String!
        configuration: Json!
        fallbackConfiguration: Json
    }

    type Query {
        shop: Shop!
    }
], enums_as_str = ["__TypeKind"])]
mod schema {
    #[query([
        query Query {
            shop {
                name
                configuration
                fallbackConfiguration
            }
        }
    ], custom_scalar_overrides = {
        // The path is relative to the schema module, so `super` is the crate root.
        "Query.shop.configuration" => super::Configuration,
        "Query.shop.fallbackConfiguration" => super::Configuration,
    })]
    pub mod query {}
}

#[test]
fn test_custom_scalar_override_with_serde() {
    let context = shopify_function::wasm_api::Context::new_with_input(serde_json::json!({
        "shop": {
            "name": "Test shop",
            "configuration": {
                "minimumQuantity": 3,
                "strategy": "MOST_EXPENSIVE",
            },
            "fallbackConfiguration": null,
        },
    }));
    let value = context.input_get().unwrap();

    let query = schema::query::Query::deserialize(&value).unwrap();
    let shop = query.shop();

    assert_eq!(shop.name(), "Test shop");
    assert_eq!(
        shop.configuration(),
        &Configuration {
            minimum_quantity: 3,
            strategy: Strategy::MostExpensive,
        }
    );
    assert_eq!(shop.fallback_configuration(), None);
}
