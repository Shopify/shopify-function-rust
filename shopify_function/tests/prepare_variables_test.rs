use shopify_function::prelude::*;
use shopify_function::wasm_api::{self, Context, Deserialize, Serialize};

/// The type the schema module aliases `Money` to.
#[derive(Debug, PartialEq, Clone)]
pub struct Money(pub String);

impl Serialize for Money {
    fn serialize(&self, context: &mut Context) -> Result<(), wasm_api::write::Error> {
        context.write_utf8_str(&self.0)
    }
}

impl Deserialize for Money {
    fn deserialize(value: &wasm_api::Value) -> Result<Self, wasm_api::read::Error> {
        Ok(Self(String::deserialize(value)?))
    }
}

#[typegen([
    scalar JSON
    scalar Money

    type Query {
        lines(first: Int!): [String!]!
        priced(range: PriceRange): [String!]!
        tree(node: Node): Boolean!
    }

    input PriceRange {
        min: Money
    }

    input Node {
        value: Int!
        children: [Node!]
    }

    input OptionalPrepareResult {
        variables: JSON
    }

    input PricedPrepareResult {
        variables: JSON!
    }

    input TreePrepareResult {
        variables: JSON!
    }
], enums_as_str = ["__TypeKind"], custom_scalar_overrides = {
    "OptionalPrepareResult.variables" => optional_run::InputVariables,
    "PricedPrepareResult.variables" => priced_run::InputVariables,
    "TreePrepareResult.variables" => tree_run::TreeVariables,
})]
mod schema {
    pub type Money = super::Money;

    #[query([
        query Input($first: Int! = 10) {
            lines(first: $first)
        }
    ])]
    pub mod optional_run {}

    #[query([
        query Input($range: PriceRange) {
            priced(range: $range)
        }
    ])]
    pub mod priced_run {}

    #[query([
        query Tree($node: Node) {
            tree(node: $node)
        }
    ])]
    pub mod tree_run {}
}

fn serialize_to_json<T: Serialize>(value: &T) -> serde_json::Value {
    let mut context = Context::new_with_input(serde_json::json!({}));
    value.serialize(&mut context).unwrap();
    context.finalize_output_and_return().unwrap()
}

fn deserialize_from_json<T: Deserialize>(json: serde_json::Value) -> T {
    let context = Context::new_with_input(json);
    T::deserialize(&context.input_get().unwrap()).unwrap()
}

#[test]
fn test_nullable_prepare_variables_are_optional() {
    assert_eq!(
        serde_json::json!({}),
        serialize_to_json(&schema::OptionalPrepareResult { variables: None })
    );
    // `None` omits `$first`, so the query's default applies.
    assert_eq!(
        serde_json::json!({ "variables": {} }),
        serialize_to_json(&schema::OptionalPrepareResult {
            variables: Some(schema::optional_run::InputVariables { first: None }),
        })
    );
    assert_eq!(
        serde_json::json!({ "variables": { "first": 1 } }),
        serialize_to_json(&schema::OptionalPrepareResult {
            variables: Some(schema::optional_run::InputVariables { first: Some(1) }),
        })
    );
}

#[test]
fn test_prepare_variables_serialize_aliased_custom_scalars_with_their_own_impl() {
    assert_eq!(
        serde_json::json!({ "variables": { "range": { "min": "1.5" } } }),
        serialize_to_json(&schema::PricedPrepareResult {
            variables: schema::priced_run::InputVariables {
                range: Some(schema::PriceRange {
                    min: Some(Money("1.5".to_string())),
                }),
            },
        })
    );
}

#[test]
fn test_prepare_variables_round_trip() {
    let result = schema::TreePrepareResult {
        variables: schema::tree_run::TreeVariables {
            node: Some(schema::Node {
                value: 1,
                children: Some(vec![schema::Node {
                    value: 2,
                    children: None,
                }]),
            }),
        },
    };
    let json = serde_json::json!({
        "variables": {
            "node": {
                "value": 1,
                "children": [{ "value": 2 }]
            }
        }
    });

    assert_eq!(json, serialize_to_json(&result));
    assert_eq!(
        result,
        deserialize_from_json::<schema::TreePrepareResult>(json)
    );
}
