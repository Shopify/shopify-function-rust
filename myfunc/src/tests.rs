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

    let expected_handle_result: serde_json::Value = serde_json::from_str(expected_json).unwrap();
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

    let expected_handle_result: serde_json::Value = serde_json::from_str(expected_json).unwrap();
    assert_eq!(
        handle_result.to_string(),
        expected_handle_result.to_string()
    );
}
