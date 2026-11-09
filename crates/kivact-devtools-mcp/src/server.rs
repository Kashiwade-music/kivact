//! Role: The MCP stdio loop: one JSON-RPC line in, at most one line out.

use std::io::{BufRead, BufReader, Write};

use kivact_devtools_protocol as p;
use serde_json::{Value, json};

use crate::result;
use crate::target::{Selection, ToolError};

/// Run the MCP server on stdin and stdout until stdin ends.
pub(crate) fn run(schema: Value) {
    let tools = crate::tools::list(&schema);
    let mut selection = Selection::None;
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let stdout = std::io::stdout();
    let mut writer = stdout.lock();
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(error) => {
                log(&format!("cannot read stdin: {error}"));
                break;
            }
        }
        let request = line.trim();
        if request.is_empty() {
            continue;
        }
        let Some(response) = handle(request, &tools, &mut selection) else {
            continue;
        };
        if write_response(&mut writer, &response).is_err() {
            break;
        }
    }
}

/// One request line: its response, or `None` for a message without an `id` (a notification,
/// which JSON-RPC 2.0 does not answer).
fn handle(line: &str, tools: &[Value], selection: &mut Selection) -> Option<p::RpcResponse> {
    let request: Value = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(error) => {
            return Some(p::response(
                Value::Null,
                Err(p::RpcError::new(p::code::PARSE_ERROR, error.to_string())),
            ));
        }
    };
    let id = request.get("id").cloned()?;
    let Some(method) = request.get("method").and_then(Value::as_str) else {
        return Some(p::response(
            id,
            Err(p::RpcError::new(
                p::code::INVALID_REQUEST,
                "the line is not a JSON-RPC request",
            )),
        ));
    };
    let outcome = match method {
        "initialize" => Ok(initialize_result(&request)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tools })),
        "tools/call" => Ok(call_tool(request.get("params"), tools, selection)),
        _ => Err(p::RpcError::new(
            p::code::METHOD_NOT_FOUND,
            format!("unknown method: {method}"),
        )),
    };
    Some(p::response(id, outcome))
}

fn initialize_result(request: &Value) -> Value {
    let version = request
        .get("params")
        .and_then(|params| params.get("protocolVersion"))
        .cloned()
        .unwrap_or(Value::Null);
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {} },
        "serverInfo": {
            "name": "kivact-devtools-mcp",
            "version": env!("CARGO_PKG_VERSION"),
        },
    })
}

fn call_tool(params: Option<&Value>, tools: &[Value], selection: &mut Selection) -> Value {
    let params = params.cloned().unwrap_or(Value::Null);
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let advertised = tools
        .iter()
        .any(|tool| tool.get("name").and_then(Value::as_str) == Some(name));
    if !advertised {
        return result::failure(&ToolError::Text(format!("unknown tool: {name}")));
    }
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let outcome = match name {
        "list_targets" => Ok(crate::target::list_targets()),
        "select_target" => select_target(selection, &arguments),
        _ => selection.call(name, arguments),
    };
    match outcome {
        Ok(value) => result::success(name, value),
        Err(error) => result::failure(&error),
    }
}

fn select_target(selection: &mut Selection, arguments: &Value) -> Result<Value, ToolError> {
    let target = arguments
        .get("target")
        .and_then(Value::as_str)
        .ok_or_else(|| ToolError::Text("select_target needs a \"target\" string".to_owned()))?;
    selection.select(std::path::Path::new(target))?;
    Ok(json!({}))
}

fn write_response(writer: &mut impl Write, response: &p::RpcResponse) -> std::io::Result<()> {
    writer.write_all(p::response_line(response).as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()
}

fn log(message: &str) {
    let _ = writeln!(std::io::stderr().lock(), "kivact-devtools-mcp: {message}");
}
