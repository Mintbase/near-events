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
//! - [] Create event string from event data
//! - [] Support for deserialization for indexers
//!   - [] Deserialization code mustn't be wasm'ed for size reasons
//!

use near_sdk::serde::Serialize;

pub use near_event_data_log_macro::near_event_data_log;

pub fn serialize<T: Serialize>(
    standard: &str,
    version: &str,
    event: &str,
    data: Vec<T>,
) -> String {
    let json = serde_json::json!({
        "standard": standard,
        "version": version,
        "event": event,
        "data": data
    });
    format!("EVENT_JSON:{}", json)
}

pub trait NearEventDataLog {
    fn serialize_event(&self) -> String;
}

pub struct NearEventData<T: NearEventDataLog>(Vec<T>);

#[cfg(test)]
mod tests {
    use super::*;

    #[near_event_data_log(
        standard = "nepXXX",
        version = "1.0.0",
        event = "test_event"
    )]
    struct TestEventLog {
        foo: String,
    }

    const EVENT_STR: &str = r#"EVENT_JSON:{"standard":"nepXXX","version":"1.0.0","event":"test_event","data":[{"foo":"bar"}]}"#;

    #[test]
    fn correct_serialization() {
        let log = TestEventLog {
            foo: "bar".to_string(),
        };
        assert_eq!(log.serialize_event(), EVENT_STR);
    }
}
