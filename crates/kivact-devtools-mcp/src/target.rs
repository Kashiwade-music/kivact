//! Role: Choosing the DevTools target of the MCP tools and sending calls to it.

use std::io::Write;
use std::path::{Path, PathBuf};

use kivact_devtools_client as client;
use kivact_devtools_protocol as p;
use serde_json::{Value, json};

/// The target the tools talk to.
pub(crate) enum Selection {
    /// No target is chosen yet.
    None,
    /// The chosen target.
    Selected {
        /// The discovery file the target was chosen by.
        path: PathBuf,
        /// The connection to the target.
        connection: client::Connection,
    },
}

/// Why a tool call failed: an answer of the agent, or a text for the AI.
pub(crate) enum ToolError {
    /// The agent answered with an error.
    Rpc(p::RpcError),
    /// The MCP server explains the failure as text.
    Text(String),
}

impl Selection {
    /// `select_target`: open `path` and keep it.
    pub(crate) fn select(&mut self, path: &Path) -> Result<(), ToolError> {
        let file = client::read_target(path)
            .map_err(|error| ToolError::Text(format!("cannot read {}: {error}", path.display())))?;
        let (connection, _initialized) = client::Connection::open(&file, client_info())
            .map_err(|error| ToolError::Text(error.to_string()))?;
        if let Selection::Selected { path: previous, .. } = self {
            let _ = writeln!(
                std::io::stderr().lock(),
                "kivact-devtools-mcp: target switched from {} to {}",
                previous.display(),
                path.display()
            );
        }
        *self = Selection::Selected {
            path: path.to_path_buf(),
            connection,
        };
        Ok(())
    }

    /// Any other tool: choose a target if none is selected (rule in 4.6), then `call_raw`.
    pub(crate) fn call(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ToolError> {
        if matches!(self, Selection::None) {
            self.connect()?;
        }
        let Selection::Selected { connection, .. } = self else {
            return Err(ToolError::Text("no target is selected".to_owned()));
        };
        match connection.call_raw(method, params) {
            Ok(value) => Ok(value),
            Err(client::ClientError::Rpc(error)) => Err(ToolError::Rpc(error)),
            Err(client::ClientError::Closed) => {
                *self = Selection::None;
                Err(ToolError::Text(
                    "target closed; call list_targets".to_owned(),
                ))
            }
            Err(error) => Err(ToolError::Text(error.to_string())),
        }
    }

    /// Choose the target from the environment, or the only running one.
    fn connect(&mut self) -> Result<(), ToolError> {
        if let Some(path) = client::target_from_env() {
            return self.select(&path);
        }
        let targets = client::list_targets();
        match targets.as_slice() {
            [] => Err(ToolError::Text(
                "no running target; start an app with DevTools enabled".to_owned(),
            )),
            [(path, _)] => self.select(path),
            many => Err(ToolError::Text(format!(
                "several targets are running; call select_target with one of: {}",
                target_paths(many)
            ))),
        }
    }
}

/// `list_targets`: the running targets, without their tokens.
pub(crate) fn list_targets() -> serde_json::Value {
    let targets: Vec<Value> = client::list_targets()
        .iter()
        .map(|(path, file)| {
            json!({
                "target": path.display().to_string(),
                "app_name": file.app_name,
                "pid": file.pid,
                "port": file.port,
                "kivact_version": file.kivact_version,
                "started_unix_ms": file.started_unix_ms,
                "workspace_root": file.workspace_root,
            })
        })
        .collect();
    json!({ "targets": targets })
}

fn client_info() -> p::ClientInfo {
    p::ClientInfo {
        name: "kivact-devtools-mcp".to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        kind: p::ClientKind::Mcp,
    }
}

fn target_paths(targets: &[(PathBuf, p::DiscoveryFile)]) -> String {
    targets
        .iter()
        .map(|(path, _)| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
