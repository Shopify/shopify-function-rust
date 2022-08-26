use shopify_rust_function::{discounts, function_config, input_query, shopify_function, Result};

#[input_query(query_path = "./input.graphql", schema_path = "./schema.graphql")]
#[function_config]
#[derive(Default, PartialEq)]
struct Config {
    pub quantity: i64,
    pub percentage: f64,
}

#[shopify_function]
fn function(input: input_query::ResponseData) -> Result<discounts::Output> {
    let config: Config = shopify_rust_function::parse_config(
        input
            .discount_node
            .metafield
            .as_ref()
            .map(|m| m.value.as_str()),
    );
    let cart_lines = input.cart.lines;

    if cart_lines.is_empty() || config.percentage == 0.0 {
        return Ok(discounts::EMPTY_DISCOUNT.clone());
    }

    let mut targets = vec![];
    for line in cart_lines {
        if line.quantity >= config.quantity {
            targets.push(discounts::Target::ProductVariant {
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
        return Ok(discounts::EMPTY_DISCOUNT.clone());
    }

    Ok(discounts::Output {
        discounts: vec![discounts::Discount {
            message: None,
            conditions: None,
            targets,
            value: discounts::Value::Percentage(discounts::Percentage {
                value: config.percentage,
            }),
        }],
        discount_application_strategy: discounts::DiscountApplicationStrategy::First,
    })
}

#[cfg(test)]
mod tests;
