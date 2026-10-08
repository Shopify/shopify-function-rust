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

    input Vec {
        value: String
    }

    enum Box {
        A
    }

    type Query {
        result: Result!
        option: Option!
        filtered(vec: Vec, list: [Vec!], box: Box!): String
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

    #[query([
        query Filtered($vec: Vec, $list: [Vec!], $box: Box!) {
            filtered(vec: $vec, list: $list, box: $box)
        }
    ])]
    pub mod filtered {}
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

#[test]
fn test_variables_macro_hygiene() {
    let variables = schema::filtered::FilteredVariables {
        vec: None,
        list: Some(vec![schema::Vec {
            value: Some("test".to_string()),
        }]),
        r#box: schema::Box::A,
    };

    let mut context = shopify_function::wasm_api::Context::new_with_input(serde_json::json!({}));
    shopify_function::wasm_api::Serialize::serialize(&variables, &mut context).unwrap();
    let output = context.finalize_output_and_return().unwrap();

    assert_eq!(
        serde_json::json!({
            "list": [{ "value": "test" }],
            "box": "A",
        }),
        output
    );

    let context = shopify_function::wasm_api::Context::new_with_input(output);
    let value = context.input_get().unwrap();
    assert_eq!(
        variables,
        schema::filtered::FilteredVariables::deserialize(&value).unwrap()
    );
}
