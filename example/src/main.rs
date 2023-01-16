use shopify_function::prelude::*;
use shopify_function::Result;

use serde::{Deserialize, Serialize};

generate_types!(
    query_path = "./input.graphql",
    schema_path = "./schema.graphql"
);

#[derive(Serialize, Deserialize, Default, PartialEq)]
struct Config {
    pub quantity: i64,
    pub percentage: f64,
}

#[shopify_function]
fn function(input: input::ResponseData) -> Result<output::FunctionResult> {
    let config: Config = input
        .discount_node
        .metafield
        .as_ref()
        .map(|m| serde_json::from_str::<Config>(m.value.as_str()))
        .transpose()?
        .unwrap_or_default();

    let cart_lines = input.cart.lines;

    if cart_lines.is_empty() || config.percentage == 0.0 {
        return Ok(output::FunctionResult {
            discount_application_strategy: output::DiscountApplicationStrategy::FIRST,
            discounts: vec![],
        });
    }

    let mut targets = vec![];
    for line in cart_lines {
        if line.quantity >= config.quantity {
            targets.push(output::Target {
                product_variant: Some(output::ProductVariantTarget {
                    id: match line.merchandise {
                        input::InputCartLinesMerchandise::ProductVariant(variant) => variant.id,
                        _ => continue,
                    },
                    quantity: None,
                }),
            });
        }
    }

    if targets.is_empty() {
        return Ok(output::FunctionResult {
            discount_application_strategy: output::DiscountApplicationStrategy::FIRST,
            discounts: vec![],
        });
    }

    Ok(output::FunctionResult {
        discounts: vec![output::Discount {
            message: None,
            targets,
            value: output::Value {
                percentage: Some(output::Percentage {
                    value: config.percentage.to_string(),
                }),
                fixed_amount: None,
            },
        }],
        discount_application_strategy: output::DiscountApplicationStrategy::FIRST,
    })
}

#[cfg(test)]
mod tests;
