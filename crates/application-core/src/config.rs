use serde_json::{json, Value};

pub fn normalize_config_string_array(parsed: Value) -> Vec<String> {
    let mut values = parsed
        .as_array()
        .map(|items| {
            items
                .iter()
                .map(config_value_to_string)
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    values.sort();
    values.dedup();
    values
}

pub fn config_string_array_value(values: &[String]) -> Value {
    json!(values)
}

fn config_value_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}
