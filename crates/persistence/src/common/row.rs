use crate::Error;

pub fn row_string(row: &[serde_json::Value], index: usize) -> String {
    match row.get(index).unwrap_or(&serde_json::Value::Null) {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

pub fn row_i64(row: &[serde_json::Value], index: usize) -> i64 {
    row.get(index)
        .and_then(|value| value.as_i64())
        .or_else(|| {
            row.get(index)
                .and_then(|value| value.as_str()?.parse().ok())
        })
        .unwrap_or_default()
}

pub fn row_value(row: &[serde_json::Value], index: usize) -> &serde_json::Value {
    row.get(index).unwrap_or(&serde_json::Value::Null)
}

pub fn row_json(row: &[serde_json::Value], index: usize) -> serde_json::Value {
    row.get(index).cloned().unwrap_or(serde_json::Value::Null)
}

pub fn strict_row_value(
    row: &[serde_json::Value],
    index: usize,
) -> Result<&serde_json::Value, Error> {
    row.get(index)
        .ok_or_else(|| Error::Database(format!("fixed projection is missing column {index}")))
}

pub fn strict_row_string(row: &[serde_json::Value], index: usize) -> Result<String, Error> {
    match strict_row_value(row, index)? {
        serde_json::Value::Null => Ok(String::new()),
        serde_json::Value::String(value) => Ok(value.clone()),
        serde_json::Value::Bool(value) => Ok(value.to_string()),
        serde_json::Value::Number(value) => Ok(value.to_string()),
        other => Err(Error::Database(format!(
            "fixed projection column {index} expected scalar string value, got {other}"
        ))),
    }
}

pub fn strict_row_i64(row: &[serde_json::Value], index: usize) -> Result<i64, Error> {
    match strict_row_value(row, index)? {
        serde_json::Value::Null => Ok(0),
        serde_json::Value::Number(value) => value.as_i64().ok_or_else(|| {
            Error::Database(format!(
                "fixed projection column {index} expected i64-compatible number"
            ))
        }),
        serde_json::Value::String(value) => value.parse::<i64>().map_err(|error| {
            Error::Database(format!(
                "fixed projection column {index} expected i64-compatible string: {error}"
            ))
        }),
        other => Err(Error::Database(format!(
            "fixed projection column {index} expected i64 value, got {other}"
        ))),
    }
}

pub fn strict_row_optional_string(
    row: &[serde_json::Value],
    index: usize,
) -> Result<Option<String>, Error> {
    match strict_row_value(row, index)? {
        serde_json::Value::Null => Ok(None),
        _ => strict_row_string(row, index).map(Some),
    }
}

pub fn strict_row_optional_i64(
    row: &[serde_json::Value],
    index: usize,
) -> Result<Option<i64>, Error> {
    match strict_row_value(row, index)? {
        serde_json::Value::Null => Ok(None),
        _ => strict_row_i64(row, index).map(Some),
    }
}
