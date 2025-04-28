use shopify_function_wasm_api::{
    read::Error as ReadError, write::Error as WriteError, Context, Deserialize, Serialize, Value,
};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone)]
pub enum JsonValue {
    Null,
    String(String),
    Number(f64),
    Boolean(bool),
    Object(BTreeMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

impl Deserialize for JsonValue {
    fn deserialize(value: &Value) -> Result<Self, ReadError> {
        if value.is_null() {
            Ok(JsonValue::Null)
        } else if let Some(b) = value.as_bool() {
            Ok(JsonValue::Boolean(b))
        } else if let Some(n) = value.as_number() {
            Ok(JsonValue::Number(n))
        } else if let Some(s) = value.as_string() {
            Ok(JsonValue::String(s))
        } else if let Some(array_len) = value.array_len() {
            let mut array = Vec::with_capacity(array_len);
            for i in 0..array_len {
                let item = value.get_at_index(i);
                array.push(JsonValue::deserialize(&item)?);
            }
            Ok(JsonValue::Array(array))
        } else if let Some(object_len) = value.obj_len() {
            let mut object = BTreeMap::new();
            for i in 0..object_len {
                let key = value
                    .get_obj_key_at_index(i)
                    .ok_or(ReadError::InvalidType)?;
                let value = value.get_at_index(i);
                object.insert(key.to_string(), JsonValue::deserialize(&value)?);
            }
            Ok(JsonValue::Object(object))
        } else {
            Err(ReadError::InvalidType)
        }
    }
}

impl Serialize for JsonValue {
    fn serialize(&self, context: &mut Context) -> Result<(), WriteError> {
        match self {
            JsonValue::Null => context.write_null(),
            JsonValue::String(s) => context.write_utf8_str(s),
            JsonValue::Number(n) => context.write_f64(*n),
            JsonValue::Boolean(b) => context.write_bool(*b),
            JsonValue::Object(o) => context.write_object(
                |ctx| {
                    for (key, value) in o {
                        ctx.write_utf8_str(key)?;
                        value.serialize(ctx)?;
                    }
                    Ok(())
                },
                o.len(),
            ),
            JsonValue::Array(a) => context.write_array(
                |ctx| {
                    for value in a {
                        value.serialize(ctx)?;
                    }
                    Ok(())
                },
                a.len(),
            ),
        }
    }
}
