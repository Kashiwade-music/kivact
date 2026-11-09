//! Role: MCP server that exposes a Kivact DevTools agent to AI tools.

mod result;
mod server;
mod target;
mod tools;

use kivact_devtools_protocol as p;

fn main() {
    let schema: serde_json::Value =
        serde_json::from_str(p::SCHEMA_JSON).expect("invariant: the protocol schema is valid JSON");
    server::run(schema);
}
