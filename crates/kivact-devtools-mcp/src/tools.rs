//! Role: The MCP tools: protocol methods and local targets, with resolved input schemas.

use serde_json::{Map, Value, json};

/// The methods that are not tools: the connection handshake and the app-side table.
const EXCLUDED: [&str; 3] = ["initialize", "subscribe", "show_table"];

/// How deep a `$ref` may be followed before the input schema gives up.
const MAX_DEPTH: usize = 16;

/// Every tool, the two local ones first.
pub(crate) fn list(schema: &Value) -> Vec<Value> {
    let empty = Map::new();
    let defs = schema
        .get("$defs")
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    let mut tools = vec![list_targets_tool(), select_target_tool()];
    let methods = schema
        .get("x-methods")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    for method in methods {
        let Some(name) = method.get("name").and_then(Value::as_str) else {
            continue;
        };
        if EXCLUDED.contains(&name) {
            continue;
        }
        let params = method
            .get("params")
            .and_then(Value::as_str)
            .unwrap_or("Empty");
        let input = resolve_refs(&json!({ "$ref": format!("#/$defs/{params}") }), defs, 0);
        tools.push(json!({
            "name": name,
            "description": method.get("description").cloned().unwrap_or(Value::Null),
            "inputSchema": input,
        }));
    }
    tools
}

/// Expand every `$ref` of `value` from `defs`, at most `MAX_DEPTH` levels deep.
fn resolve_refs(value: &Value, defs: &Map<String, Value>, depth: usize) -> Value {
    if depth > MAX_DEPTH {
        return json!({});
    }
    match value {
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                let Some(name) = reference.strip_prefix("#/$defs/") else {
                    return json!({});
                };
                let Some(target) = defs.get(name) else {
                    return json!({});
                };
                let mut resolved = resolve_refs(target, defs, depth + 1);
                if let Value::Object(resolved_object) = &mut resolved {
                    for (key, value) in object {
                        if key != "$ref" {
                            resolved_object.insert(key.clone(), value.clone());
                        }
                    }
                }
                return resolved;
            }
            let mut resolved = Map::new();
            for (key, value) in object {
                resolved.insert(key.clone(), resolve_refs(value, defs, depth + 1));
            }
            Value::Object(resolved)
        }
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| resolve_refs(item, defs, depth + 1))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn list_targets_tool() -> Value {
    json!({
        "name": "list_targets",
        "description": "The running DevTools targets: discovery file paths and app details (no tokens).",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "additionalProperties": false,
        },
    })
}

fn select_target_tool() -> Value {
    json!({
        "name": "select_target",
        "description": "Choose the target of later tool calls by discovery file path from list_targets.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Discovery file path from list_targets.",
                },
            },
            "required": ["target"],
            "additionalProperties": false,
        },
    })
}
