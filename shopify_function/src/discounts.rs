#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::scalars::*;

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct Output {
    pub discount_application_strategy: DiscountApplicationStrategy,
    pub discounts: Vec<Discount>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
#[serde(rename_all(
    serialize = "SCREAMING_SNAKE_CASE",
    deserialize = "SCREAMING_SNAKE_CASE"
))]
pub enum DiscountApplicationStrategy {
    First,
    Maximum,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
pub struct Discount {
    pub value: Value,
    pub targets: Vec<Target>,
    pub message: Option<String>,
    pub conditions: Option<Vec<Condition>>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum Value {
    FixedAmount(FixedAmount),
    Percentage(Percentage),
}

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct FixedAmount {
    pub applies_to_each_item: Option<Boolean>,
    pub value: Float,
}

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
pub struct Percentage {
    pub value: Float,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum Target {
    ProductVariant { id: ID, quantity: Option<Int> },
}

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum Condition {
    #[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
    ProductMinimumQuantity {
        ids: Vec<ID>,
        minimum_quantity: Int,
        target_type: ConditionTargetType,
    },
    #[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
    ProductMinimumSubtotal {
        ids: Vec<ID>,
        minimum_amount: Float,
        target_type: ConditionTargetType,
    },
}

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
#[serde(rename_all(
    serialize = "SCREAMING_SNAKE_CASE",
    deserialize = "SCREAMING_SNAKE_CASE"
))]
pub enum ConditionTargetType {
    ProductVariant,
}

pub static EMPTY_DISCOUNT: Output = Output {
    discounts: vec![],
    discount_application_strategy: DiscountApplicationStrategy::First,
};
