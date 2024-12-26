pub mod state;

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use serde_json::{json, Value};

    use crate::state::StateDelta;

    #[test]
    fn diff() {
        let local_file = File::open(format!("examples/resource_a.json"));

        let mut local_json = String::new();
        let _ = local_file.unwrap().read_to_string(&mut local_json);

        let local_json: Value = serde_json::from_str(local_json.as_str()).unwrap();

        let remote_file = File::open(format!("examples/resource_b.json"));

        let mut remote_json = String::new();
        let _ = remote_file.unwrap().read_to_string(&mut remote_json);

        let remote_json: Value = serde_json::from_str(remote_json.as_str()).unwrap();

        let delta = StateDelta::from_json(
            local_json.as_object().unwrap()["resources"].clone(),
            remote_json.as_object().unwrap()["resources"].clone()
        );

        let expected = StateDelta { 
            resources_to_delete: vec![],
            resources_to_create: vec![],
            resources_to_update: vec![
                json!(
                {
                    "urn": "ovejas.system::User::user_0",
                    "parameters": {
                        "name": "user0",
                        "uid": 110,
                        "gid": 111
                    }
                })
            ]
        };

        assert_eq!(delta, expected);
    }
}
