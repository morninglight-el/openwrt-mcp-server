// Structured Logging Module
// This module emits compact JSON log events without adding a runtime dependency.

use serde_json::{json, Map, Value};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn info(event: &str, fields: &[(&str, Value)]) {
    log("info", event, fields);
}

pub fn warn(event: &str, fields: &[(&str, Value)]) {
    log("warn", event, fields);
}

pub fn error(event: &str, fields: &[(&str, Value)]) {
    log("error", event, fields);
}

fn log(level: &str, event: &str, fields: &[(&str, Value)]) {
    let mut record = Map::new();
    record.insert("level".to_string(), json!(level));
    record.insert("event".to_string(), json!(event));
    record.insert("timestamp_unix_ms".to_string(), json!(timestamp_unix_ms()));

    for (key, value) in fields {
        record.insert((*key).to_string(), value.clone());
    }

    let line = Value::Object(record).to_string();
    if level == "error" {
        eprintln!("{line}");
    } else {
        println!("{line}");
    }
}

fn timestamp_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::timestamp_unix_ms;

    #[test]
    fn timestamp_is_available() {
        assert!(timestamp_unix_ms() > 0);
    }
}
