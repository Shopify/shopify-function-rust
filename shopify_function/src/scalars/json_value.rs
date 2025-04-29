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
            Ok(Self::Null)
        } else if let Some(b) = value.as_bool() {
            Ok(Self::Boolean(b))
        } else if let Some(n) = value.as_number() {
            Ok(Self::Number(n))
        } else if let Some(s) = value.as_string() {
            Ok(Self::String(s))
        } else if let Some(array_len) = value.array_len() {
            let mut array = Vec::with_capacity(array_len);
            for i in 0..array_len {
                let item = value.get_at_index(i);
                array.push(Self::deserialize(&item)?);
            }
            Ok(Self::Array(array))
        } else if let Some(object_len) = value.obj_len() {
            let mut object = BTreeMap::new();
            for i in 0..object_len {
                let key = value
                    .get_obj_key_at_index(i)
                    .ok_or(ReadError::InvalidType)?;
                let value = value.get_at_index(i);
                object.insert(key.to_string(), Self::deserialize(&value)?);
            }
            Ok(Self::Object(object))
        } else {
            Err(ReadError::InvalidType)
        }
    }
}

impl Serialize for JsonValue {
    fn serialize(&self, context: &mut Context) -> Result<(), WriteError> {
        match self {
            Self::Null => context.write_null(),
            Self::String(s) => context.write_utf8_str(s),
            Self::Number(n) => context.write_f64(*n),
            Self::Boolean(b) => context.write_bool(*b),
            Self::Object(o) => context.write_object(
                |ctx| {
                    for (key, value) in o {
                        ctx.write_utf8_str(key)?;
                        value.serialize(ctx)?;
                    }
                    Ok(())
                },
                o.len(),
            ),
            Self::Array(a) => context.write_array(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let json = serde_json::json!({
            "null": null,
            "string": "string",
            "number": 123,
            "boolean": true,
            "object": {
                "key": "value"
            },
            "array": [1, 2, 3]
        });
        let context = Context::new_with_input(json);
        let value = context.input_get().unwrap();
        let json_value = JsonValue::deserialize(&value).unwrap();
        assert_eq!(
            json_value,
            JsonValue::Object(BTreeMap::from([
                ("null".to_string(), JsonValue::Null),
                (
                    "string".to_string(),
                    JsonValue::String("string".to_string())
                ),
                ("number".to_string(), JsonValue::Number(123.0)),
                ("boolean".to_string(), JsonValue::Boolean(true)),
                (
                    "object".to_string(),
                    JsonValue::Object(BTreeMap::from([(
                        "key".to_string(),
                        JsonValue::String("value".to_string())
                    )]))
                ),
                (
                    "array".to_string(),
                    JsonValue::Array(vec![
                        JsonValue::Number(1.0),
                        JsonValue::Number(2.0),
                        JsonValue::Number(3.0)
                    ])
                )
            ]))
        );
    }
}
