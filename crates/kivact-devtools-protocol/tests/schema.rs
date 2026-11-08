//! Role: The generated schema lists the same methods, version and error codes as the Rust protocol.

use kivact_devtools_protocol::{METHOD_NAMES, PROTOCOL_VERSION, SCHEMA_JSON, code};

fn schema() -> serde_json::Value {
    serde_json::from_str(SCHEMA_JSON).expect("invariant: the generated schema is JSON")
}

#[test]
fn methods_match_in_order() {
    let schema = schema();
    let names: Vec<&str> = schema["x-methods"]
        .as_array()
        .expect("invariant: x-methods is an array")
        .iter()
        .map(|method| {
            method["name"]
                .as_str()
                .expect("invariant: a method has a name")
        })
        .collect();
    assert_eq!(names, METHOD_NAMES);
}

#[test]
fn version_and_error_codes_match() {
    let schema = schema();
    assert_eq!(schema["x-protocol"], PROTOCOL_VERSION);
    let errors = schema["x-errors"]
        .as_array()
        .expect("invariant: x-errors is an array");
    let node_not_found = errors
        .iter()
        .find(|error| error["name"] == "NODE_NOT_FOUND");
    assert_eq!(
        node_not_found.map(|error| error["code"].clone()),
        Some(code::NODE_NOT_FOUND.into())
    );
}
