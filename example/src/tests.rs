use super::*;
use shopify_function::{run_function_with_input, Result};

#[test]
fn test_discount_with_no_configuration() -> Result<()> {
    let result = run_function_with_input(
        function,
        r#"
            {
                "cart": {
                    "lines": [
                        {
                            "cost": {
                                "totalAmount": {
                                    "amount": "0"
                                }
                            },
                            "merchandise": {
                                "__typename": "ProductVariant",
                                "id": "gid://shopify/ProductVariant/0"
                            },
                            "quantity": 5
                        },
                        {
                            "cost": {
                                "totalAmount": {
                                    "amount": "0"
                                }
                            },
                            "merchandise": {
                                "__typename": "ProductVariant",
                                "id": "gid://shopify/ProductVariant/1"
                            },
                            "quantity": 1
                        }
                    ]
                },
                "discountNode": {
                    "metafield": null
                }
            }
        "#,
    )?;
    let expected = crate::output::FunctionResult {
        discounts: vec![],
        discount_application_strategy: crate::output::DiscountApplicationStrategy::FIRST,
    };
    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_discount_with_configuration() -> Result<()> {
    let result = run_function_with_input(
        function,
        r#"
            {
                "cart": {
                    "lines": [
                        {
                            "cost": {
                                "totalAmount": {
                                    "amount": "0"
                                }
                            },
                            "merchandise": {
                                "__typename": "ProductVariant",
                                "id": "gid://shopify/ProductVariant/0"
                            },
                            "quantity": 5
                        },
                        {
                            "cost": {
                                "totalAmount": {
                                    "amount": "10"
                                }
                            },
                            "merchandise": {
                                "__typename": "ProductVariant",
                                "id": "gid://shopify/ProductVariant/1"
                            },
                            "quantity": 1
                        }
                    ]
                },
                "discountNode": {
                    "metafield": {
                        "value": "{\"quantity\": 5, \"percentage\": 10}"
                    }
                }
            }
        "#,
    )?;
    let expected = crate::output::FunctionResult {
        discounts: vec![crate::output::Discount {
            message: None,
            targets: vec![crate::output::Target::ProductVariant(
                output::ProductVariantTarget {
                    id: "gid://shopify/ProductVariant/0".to_string(),
                    quantity: None,
                },
            )],
            value: crate::output::Value::Percentage(output::Percentage {
                value: Decimal(10.0),
            }),
        }],
        discount_application_strategy: crate::output::DiscountApplicationStrategy::FIRST,
    };

    assert_eq!(result, expected);
    Ok(())
}
