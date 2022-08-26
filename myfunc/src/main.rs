use shopify_rust_function::{
    api::{input, DiscountApplicationStrategy},
    api::{Discount, Output, Percentage, Target, Value},
    shopify_function,
};

#[shopify_function]
fn function(input: input::Input) -> Result<Output, Box<dyn std::error::Error>> {
    let config: input::Configuration = input.configuration();
    let cart_lines = input.cart.lines;

    if cart_lines.is_empty() || config.percentage == 0.0 {
        return Ok(Output {
            discounts: vec![],
            discount_application_strategy: DiscountApplicationStrategy::First,
        });
    }

    let mut targets = vec![];
    for line in cart_lines {
        if line.quantity >= config.quantity {
            targets.push(Target::ProductVariant {
                id: line.merchandise.id.unwrap_or_default(),
                quantity: None,
            });
        }
    }

    if targets.is_empty() {
        return Ok(Output {
            discounts: vec![],
            discount_application_strategy: DiscountApplicationStrategy::First,
        });
    }

    Ok(Output {
        discounts: vec![Discount {
            message: None,
            conditions: None,
            targets,
            value: Value::Percentage(Percentage {
                value: config.percentage,
            }),
        }],
        discount_application_strategy: DiscountApplicationStrategy::First,
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
