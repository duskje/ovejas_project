use std::collections::{HashMap, HashSet};
use serde_json::Value;

#[derive(Debug)]
#[derive(PartialEq)]
pub struct StateDelta {
    pub resources_to_delete: Vec<Value>,
    pub resources_to_create: Vec<Value>,
    pub resources_to_update: Vec<Value>,
}

impl StateDelta {
    pub fn from_json(local_json: Value, remote_json: Value) -> Self {
        let mut delta: Vec<&str> = Vec::new();
        
        let mut local_resources: HashMap<&str, Value> = HashMap::new();

        for resource in local_json.as_array().unwrap() {
            let urn = resource.get("urn").expect("Resource has no urn").as_str().expect("Could not parse str");
            local_resources.insert(urn, resource.clone());
        };

        let mut remote_resources: HashMap<&str, Value> = HashMap::new();

        for resource in remote_json.as_array().unwrap() {
            let urn = resource.get("urn").expect("Resource has no urn").as_str().expect("Could not parse str");
            remote_resources.insert(urn, resource.clone());
        };

        let local_keys: HashSet<&str> = remote_resources.keys().copied().collect();
        let remote_keys: HashSet<&str> = local_resources.keys().copied().collect();

        println!("{local_keys:?}");
        println!("{remote_keys:?}");

        let resources_to_create: Vec<Value> = local_keys.difference(&remote_keys)
            .map(|local_key| remote_resources.get(local_key).unwrap())
            .cloned()
            .collect();

        let resources_to_delete: Vec<Value> = remote_keys.difference(&local_keys)
            .map(|remote_key| local_resources.get(remote_key).unwrap())
            .cloned()
            .collect();


        let resources_to_update: Vec<Value> = remote_keys.intersection(&local_keys)
            .map(|key| {
                let local_resource = local_resources.get(key).unwrap();
                let remote_resource = remote_resources.get(key).unwrap();

                if  local_resource != remote_resource {
                    Some(remote_resource)
                } else {
                    None
                }
            })
            .filter_map(|value| value) 
            .cloned()
            .collect();

        StateDelta {
            resources_to_delete,
            resources_to_create,
            resources_to_update,
        }
    }
}
