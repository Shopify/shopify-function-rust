use super::*;
use shopify_rust_function::{run_function_with_input, Result};

#[test]
fn test_discount_with_no_configuration() -> Result<()> {
    let result = run_function_with_input(
        function,
        r#"
          {
              "cart": {
                  "lines": [
                      {
                          "quantity": 5,
                          "merchandise": {
                              "__typename": "ProductVariant",
                              "id": "gid://shopify/ProductVariant/0"
                          }
                      },
                      {
                          "quantity": 1,
                          "merchandise": {
                              "__typename": "ProductVariant",
                              "id": "gid://shopify/ProductVariant/1"
                          }
                      }
                  ]
              },
              "discountNode": { "metafield": null }
          }
        "#,
    )?;
    let expected = serde_json::from_str::<discounts::Output>(
        r#"
          {
              "discounts": [],
              "discountApplicationStrategy": "FIRST"
          }
        "#,
    )?;
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
                            "quantity": 5,
                            "merchandise": {
                                "__typename": "ProductVariant",
                                "id": "gid://shopify/ProductVariant/0"
                            }
                        },
                        {
                            "quantity": 1,
                            "merchandise": {
                                "__typename": "ProductVariant",
                                "id": "gid://shopify/ProductVariant/1"
                            }
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
    let expected = serde_json::from_str::<discounts::Output>(
        r#"
          {
            "discounts": [{
                "targets": [
                    { "productVariant": { "id": "gid://shopify/ProductVariant/0" } }
                ],
                "value": { "percentage": { "value": 10.0 } }
            }],
            "discountApplicationStrategy": "FIRST"
          }
        "#,
    )?;
    assert_eq!(result, expected);
    Ok(())
}
