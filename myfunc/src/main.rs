use shopify_rust_function::{
    discount_schema, parse_config, serde, serde::Deserialize, shopify_function,
};

use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery, Clone, Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
#[graphql(query_path = "./input.graphql", schema_path = "./schema.graphql")]
struct InputQuery;

#[derive(Deserialize, Default)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Config {
    pub quantity: i64,
    pub percentage: f64,
}

static EMPTY_DISCOUNT: discount_schema::Output = discount_schema::Output {
    discounts: vec![],
    discount_application_strategy: discount_schema::DiscountApplicationStrategy::First,
};

#[shopify_function]
fn function(
    input: input_query::ResponseData,
) -> Result<discount_schema::Output, Box<dyn std::error::Error>> {
    let config: Config = parse_config(
        input
            .discount_node
            .metafield
            .as_ref()
            .map(|m| m.value.as_str()),
    );
    let cart_lines = input.cart.lines;

    if cart_lines.is_empty() || config.percentage == 0.0 {
        return Ok(EMPTY_DISCOUNT.clone());
    }

    let mut targets = vec![];
    for line in cart_lines {
        if line.quantity >= config.quantity {
            targets.push(discount_schema::Target::ProductVariant {
                id: match line.merchandise {
                    input_query::InputQueryCartLinesMerchandise::ProductVariant(variant) => {
                        variant.id
                    }
                    _ => continue,
                },
                quantity: None,
            });
        }
    }

    if targets.is_empty() {
        return Ok(EMPTY_DISCOUNT.clone());
    }

    Ok(discount_schema::Output {
        discounts: vec![discount_schema::Discount {
            message: None,
            conditions: None,
            targets,
            value: discount_schema::Value::Percentage(discount_schema::Percentage {
                value: config.percentage,
            }),
        }],
        discount_application_strategy: discount_schema::DiscountApplicationStrategy::First,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(configuration: Option<input::Configuration>) -> input::Input {
        let input = r#"
        {
            "cart": {
                "lines": [
                    {
                        "quantity": 5,
                        "merchandise": {
                            "id": "gid://shopify/ProductVariant/0"
                        }
                    },
                    {
                        "quantity": 1,
                        "merchandise": {
                            "id": "gid://shopify/ProductVariant/1"
                        }
                    }
                ]
            },
            "discountNode": { "metafield": null }
        }
        "#;
        let default_input: input::Input = serde_json::from_str(input).unwrap();
        let value = configuration.map(|x| serde_json::to_string(&x).unwrap());

        let discount_node = input::DiscountNode {
            metafield: Some(input::Metafield { value }),
        };

        input::Input {
            discount_node,
            ..default_input
        }
    }

    #[test]
    fn test_discount_with_no_configuration() {
        let input = input(None);
        let handle_result = serde_json::json!(function(input).unwrap());

        let expected_json = r#"
            {
                "discounts": [],
                "discountApplicationStrategy": "FIRST"
            }
        "#;

        let expected_handle_result: serde_json::Value =
            serde_json::from_str(expected_json).unwrap();
        assert_eq!(
            handle_result.to_string(),
            expected_handle_result.to_string()
        );
    }

    #[test]
    fn test_discount_with_configuration() {
        let input = input(Some(input::Configuration {
            quantity: 5,
            percentage: 10.0,
        }));
        let handle_result = serde_json::json!(function(input).unwrap());

        let expected_json = r#"
            {
                "discounts": [{
                    "targets": [
                        { "productVariant": { "id": "gid://shopify/ProductVariant/0" } }
                    ],
                    "value": { "percentage": { "value": 10.0 } }
                }],
                "discountApplicationStrategy": "FIRST"
            }
        "#;

        let expected_handle_result: serde_json::Value =
            serde_json::from_str(expected_json).unwrap();
        assert_eq!(
            handle_result.to_string(),
            expected_handle_result.to_string()
        );
    }
}
