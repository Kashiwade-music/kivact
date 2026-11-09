//! Role: End-to-end test of the MCP server with a fake DevTools agent.

use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::thread;

use kivact_devtools_protocol as p;
use serde_json::{Value, json};

/// The running MCP process and its pipes.
struct Mcp {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Mcp {
    fn start(discovery: &Path, base: &Path) -> Mcp {
        let mut child = Command::new(env!("CARGO_BIN_EXE_kivact-devtools-mcp"))
            .env("KIVACT_DEVTOOLS_TARGET", discovery)
            .env("LOCALAPPDATA", base)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("invariant: the MCP server starts");
        let stdin = child
            .stdin
            .take()
            .expect("invariant: the child has a stdin pipe");
        let stdout = BufReader::new(
            child
                .stdout
                .take()
                .expect("invariant: the child has a stdout pipe"),
        );
        Mcp {
            child,
            stdin,
            stdout,
        }
    }

    fn request(&mut self, id: u64, method: &str, params: Value) -> Value {
        let request = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        writeln!(self.stdin, "{request}").expect("invariant: the child reads stdin");
        self.stdin
            .flush()
            .expect("invariant: the child reads stdin");
        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .expect("invariant: the child answers");
        serde_json::from_str(&line).expect("invariant: the child answers in JSON")
    }

    fn notification(&mut self, method: &str) {
        let notification = json!({ "jsonrpc": "2.0", "method": method });
        writeln!(self.stdin, "{notification}").expect("invariant: the child reads stdin");
        self.stdin
            .flush()
            .expect("invariant: the child reads stdin");
    }
}

impl Drop for Mcp {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn tools_list_and_calls_reach_a_fake_agent() {
    let base = temp_base();
    let (discovery, listener, token) = write_discovery(&base);
    let agent_token = token.clone();
    let _agent = thread::spawn(move || serve_agent(listener, agent_token));
    {
        let mut mcp = Mcp::start(&discovery, &base);
        assert_initialize(&mut mcp);
        mcp.notification("notifications/initialized");
        assert_tools(&mut mcp);
        assert_app_info(&mut mcp);
        assert_targets(&mut mcp, &discovery, &token);
        assert_closed(&mut mcp);
    }
    let _ = fs::remove_dir_all(&base);
}

fn assert_initialize(mcp: &mut Mcp) {
    let response = mcp.request(
        1,
        "initialize",
        json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "0" },
        }),
    );
    let result = &response["result"];
    assert_eq!(result["protocolVersion"], "2025-06-18");
    assert!(result["capabilities"]["tools"].is_object());
    assert_eq!(result["serverInfo"]["name"], "kivact-devtools-mcp");
}

fn assert_tools(mcp: &mut Mcp) {
    let response = mcp.request(2, "tools/list", json!({}));
    let tools = response["result"]["tools"]
        .as_array()
        .expect("invariant: tools/list returns an array");
    let names: BTreeSet<&str> = tools
        .iter()
        .map(|tool| {
            tool["name"]
                .as_str()
                .expect("invariant: every tool has a name")
        })
        .collect();
    let mut expected: BTreeSet<&str> = p::METHOD_NAMES
        .iter()
        .copied()
        .filter(|name| !["initialize", "subscribe", "show_table"].contains(name))
        .collect();
    expected.insert("list_targets");
    expected.insert("select_target");
    assert_eq!(names, expected);
    for tool in tools {
        let schema = tool["inputSchema"].to_string();
        assert!(!schema.contains("\"$ref\""), "{} has a $ref", tool["name"]);
        assert_eq!(tool["inputSchema"]["type"], "object");
    }
}

fn assert_app_info(mcp: &mut Mcp) {
    let response = mcp.request(
        3,
        "tools/call",
        json!({ "name": "get_app_info", "arguments": {} }),
    );
    let result = &response["result"];
    assert!(result.get("isError").is_none());
    assert_eq!(result["structuredContent"]["name"], "mcp-test-app");
    assert_eq!(result["content"][0]["type"], "text");
}

fn assert_targets(mcp: &mut Mcp, discovery: &Path, token: &str) {
    let response = mcp.request(
        4,
        "tools/call",
        json!({ "name": "list_targets", "arguments": {} }),
    );
    let result = &response["result"];
    assert!(result["structuredContent"].is_object());
    let targets = result["structuredContent"]["targets"]
        .as_array()
        .expect("invariant: list_targets returns an array");
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0]["target"], discovery.display().to_string());
    assert!(targets[0].get("token").is_none());
    assert!(!response.to_string().contains("\"token\""));
    assert!(!response.to_string().contains(token));
}

fn assert_closed(mcp: &mut Mcp) {
    let response = mcp.request(
        5,
        "tools/call",
        json!({ "name": "get_app_info", "arguments": {} }),
    );
    let result = &response["result"];
    assert_eq!(result["isError"], true);
    let text = result["content"][0]["text"]
        .as_str()
        .expect("invariant: the error is text");
    assert!(text.contains("target closed"), "{text}");
}

fn temp_base() -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    std::env::temp_dir().join(format!(
        "kivact-devtools-mcp-{}-{stamp}",
        std::process::id()
    ))
}

/// Write a discovery file in the child's discovery directory and start its fake agent.
fn write_discovery(base: &Path) -> (PathBuf, TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("invariant: a loopback port is free");
    let port = listener
        .local_addr()
        .expect("invariant: the listener has an address")
        .port();
    let token = "mcp-test-token".to_owned();
    let pid = u64::from(std::process::id());
    let file = p::DiscoveryFile {
        protocol: p::PROTOCOL_VERSION,
        pid,
        app_name: "mcp-test-app".to_owned(),
        kivact_version: "0.1.0".to_owned(),
        port: u32::from(port),
        token: token.clone(),
        started_unix_ms: 0,
        workspace_root: base.display().to_string(),
    };
    let directory = base.join("kivact").join("devtools");
    fs::create_dir_all(&directory).expect("invariant: the temp directory can be made");
    let discovery = directory.join(p::discovery_file_name(pid, file.port));
    let text = serde_json::to_vec(&file).expect("invariant: the discovery file serializes");
    fs::write(&discovery, text).expect("invariant: the discovery file can be written");
    (discovery, listener, token)
}

fn serve_agent(listener: TcpListener, token: String) {
    for stream in listener.incoming() {
        let Ok(stream) = stream else {
            continue;
        };
        serve_connection(stream, &token);
    }
}

/// Answer `initialize` and `get_app_info`; the first answer to the latter closes the connection.
fn serve_connection(stream: TcpStream, token: &str) {
    let Ok(reader_stream) = stream.try_clone() else {
        return;
    };
    let mut reader = BufReader::new(reader_stream);
    let mut writer = stream;
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
        let Ok(request) = serde_json::from_str::<Value>(line.trim()) else {
            continue;
        };
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        let outcome = match method {
            "initialize" => initialize_result(&request, token),
            "get_app_info" => Ok(json!(sample_app())),
            other => Err(p::RpcError::new(
                p::code::METHOD_NOT_FOUND,
                format!("unknown method: {other}"),
            )),
        };
        if writeln!(writer, "{}", p::response_line(&p::response(id, outcome))).is_err() {
            return;
        }
        writer.flush().ok();
        if method == "get_app_info" {
            return;
        }
    }
}

fn initialize_result(request: &Value, token: &str) -> Result<Value, p::RpcError> {
    let sent = request
        .get("params")
        .and_then(|params| params.get("token"))
        .and_then(Value::as_str);
    if sent != Some(token) {
        return Err(p::RpcError::new(p::code::BAD_TOKEN, "bad token"));
    }
    let initialized = p::InitializeResult {
        protocol: p::PROTOCOL_VERSION,
        app: sample_app(),
    };
    serde_json::to_value(initialized)
        .map_err(|error| p::RpcError::new(p::code::INTERNAL_ERROR, error.to_string()))
}

fn sample_app() -> p::AppInfo {
    p::AppInfo {
        name: "mcp-test-app".to_owned(),
        pid: 4242,
        kivact_version: "0.1.0".to_owned(),
        build: p::BuildKind::Debug,
        exe: "kivact-test".to_owned(),
        cwd: "/tmp".to_owned(),
        workspace_root: "/tmp/workspace".to_owned(),
    }
}
