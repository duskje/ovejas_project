use std::collections::HashSet;
use serde_json::Value;

pub struct StateDelta {
    pub not_in_remote: Vec<String>,
    pub not_in_local: Vec<String>,
    pub value_not_equal: Vec<String>,
}

impl StateDelta {
    pub fn from_json(local_json: Value, remote_json: Value) -> Self {

        let mut delta: Vec<String> = Vec::new();

        for (key, value) in local_json.as_object().unwrap() {
            let remote_object = remote_json.as_object().unwrap();

            if value != &remote_object[key] {
                delta.push(key.to_string());
            }
        };

        let local_keys: HashSet<String> = local_json.as_object().unwrap().keys().cloned().collect();
        let remote_keys: HashSet<String> = remote_json.as_object().unwrap().keys().cloned().collect();

        StateDelta {
            not_in_remote: local_keys.difference(&remote_keys).cloned().collect(),
            not_in_local: remote_keys.difference(&local_keys).cloned().collect(),
            value_not_equal: delta,
        }
    }
}
