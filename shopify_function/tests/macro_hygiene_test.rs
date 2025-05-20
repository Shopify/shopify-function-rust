use shopify_function::prelude::*;
use shopify_function::wasm_api::Deserialize;

#[typegen([
    type Ok {
        value: String
    }

    type Err {
        value: String
    }

    union Result = Ok | Err

    type Some {
        value: String
    }

    type None {
        // this doesn't really make sense but types must have one field
        value: String
    }

    union Option = Some | None

    type Query {
        result: Result!
        option: Option!
    }
], enums_as_str = ["__TypeKind"])]
mod schema {
    #[query([
        query Query {
            result {
                __typename
                ... on Ok {
                    value
                }
                ... on Err {
                    value
                }
            }
            option {
                __typename
                ... on Some {
                    value
                }
                ... on None {
                    value
                }
            }
        }
    ])]
    pub mod query {}
}

#[test]
fn test_macro_hygiene() {
    let value = serde_json::json!({
        "result": {
            "__typename": "Ok",
            "value": "test",
        },
        "option": {
            "__typename": "Some",
            "value": "test",
        },
    });
    let context = shopify_function::wasm_api::Context::new_with_input(value);
    let value = context.input_get().unwrap();

    let result = schema::query::Query::deserialize(&value).unwrap();
    assert!(matches!(
        result.result(),
        schema::query::query::Result::Ok(_)
    ));
    assert!(matches!(
        result.option(),
        schema::query::query::Option::Some(_)
    ));
}
