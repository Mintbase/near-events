//! # THIS IS WIP!
//!
//! This is a WIP library to easily annotate rust data structures and
//! translating them into NEAR events
//!
//! ## Design goals
//!
//! - Avoid the overhead of creating event types. Instead use dynamic types
//!   from `serde_json` and immediately serialize
//!
//! ## TODO
//!
//! - [x] Create event string from an event data log
//! - [x] Create event string from an event data vector (needs to be wrapped in
//!       tuple struct)
//! - [x] Replace generics with serde_json::Value
//! - [] Support for deserialization for indexers
//!   - [] Deserialization code mustn't be wasm'ed for size reasons
//!        (use feature-gating for this)
//! - [] Doc comments
//! - [] Package documentation
//! - [] `emit_event` on the traits, but test for size bloat first
//! - [] remove panics/unwraps/expects that are not related to the standards
//! - [] use no_std
//! - [] use a lightweight serde clone
//!

// use near_sdk::serde::Serialize;
use serde_json::Value;

/// TODO: doc comment
pub use near_event_data_log_macro::near_event_data_log;
/// TODO: doc comment
pub use near_event_data_macro::near_event_data;

/// TODO: doc comment
pub fn serialize_from_value(
    standard: &str,
    version: &str,
    event: &str,
    data: Value,
) -> String {
    let json = serde_json::json!({
        "standard": standard,
        "version": version,
        "event": event,
        "data": data
    });
    format!("EVENT_JSON:{}", json)
}

/// TODO: doc comment
pub fn partial_deserialize_event(
    event_json: &str,
) -> (String, String, String, Value) {
    use std::str::FromStr;
    let event_json = event_json.strip_prefix("EVENT_JSON:").unwrap();
    let object = Value::from_str(event_json.trim_start()).unwrap();

    let standard = opt_value_to_string(object.get("standard"));
    let version = opt_value_to_string(object.get("version"));
    let event = opt_value_to_string(object.get("event"));
    let data = object.get("data").unwrap();

    (standard, version, event, data.clone())
}

fn opt_value_to_string(opt_v: Option<&Value>) -> String {
    opt_v
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
        .unwrap()
}

/// TODO: doc comment
pub trait NearEventDataLog {
    fn serialize_event(&self) -> String;
}

/// TODO: doc comment
pub trait NearEventData {
    fn serialize_event(self) -> String;
}

#[cfg(test)]
mod tests {
    use near_sdk::serde::{Deserialize, Serialize};

    use super::*;

    #[near_event_data_log(
        standard = "nepXXX",
        version = "1.0.0",
        event = "test_event"
    )]
    struct TestEventLogV1 {
        foo: String,
    }

    #[near_event_data(
        standard = "nepXXX",
        version = "1.0.0",
        event = "test_event"
    )]
    struct TestEventDataV1(Vec<TestEventLogV1>);
    #[near_event_data_log(
        standard = "nepXXX",
        version = "2.0.0",
        event = "test_event"
    )]
    struct TestEventLogV2 {
        bar: String,
    }

    #[near_event_data(
        standard = "nepXXX",
        version = "2.0.0",
        event = "test_event"
    )]
    struct TestEventDataV2(Vec<TestEventLogV2>);

    const EVENT_STR: &str = r#"EVENT_JSON:{"standard":"nepXXX","version":"1.0.0","event":"test_event","data":[{"foo":"bar"}]}"#;

    #[test]
    fn data_log_serializes() {
        let log = TestEventLogV1 {
            foo: "bar".to_string(),
        };
        assert_eq!(log.serialize_event(), EVENT_STR);
    }

    #[test]
    fn data_serializes() {
        let logs = TestEventDataV1(vec![TestEventLogV1 {
            foo: "bar".to_string(),
        }]);
        assert_eq!(logs.serialize_event(), EVENT_STR);
    }

    fn extract_inner_data(event_json: String) -> String {
        let (standard, version, event, data) =
            partial_deserialize_event(&event_json);

        println!("({}, {}, {})", standard, version, event);
        match (standard.as_str(), version.as_str(), event.as_str()) {
            ("nepXXX", "1.0.0", "test_event") => {
                // go from JSON dynamic typing to rust static typing
                let data_v1: TestEventDataV1 =
                    serde_json::value::from_value(data).unwrap();
                // extract string from first log in data
                data_v1.0[0].foo.clone()
            }
            ("nepXXX", "2.0.0", "test_event") => {
                // go from JSON dynamic typing to rust static typing
                let data_v2: TestEventDataV2 =
                    serde_json::value::from_value(data).unwrap();
                // extract string from first log in data
                data_v2.0[0].bar.clone()
            }
            (s, v, e) => {
                panic!("Got an unexpected event triplet: ({}, {}, {})", s, v, e)
            }
        }
    }

    #[test]
    fn deserialization() {
        let event_v1 = TestEventLogV1 {
            foo: "bar".to_string(),
        }
        .serialize_event();
        let event_v2 = TestEventLogV2 {
            bar: "baz".to_string(),
        }
        .serialize_event();

        assert_eq!(extract_inner_data(event_v1), "bar");
        assert_eq!(extract_inner_data(event_v2), "baz");
    }
}
