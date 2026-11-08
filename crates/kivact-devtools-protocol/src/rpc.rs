//! Role: JSON-RPC 2.0 envelopes and the reading and writing of one line.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::call::{Call, METHOD_NAMES};
use crate::code;
use crate::notify::Notification;

/// One JSON-RPC request as read from the wire.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RpcRequest {
    /// The JSON-RPC version (`"2.0"`).
    pub jsonrpc: String,
    /// The id of the request.
    pub id: Value,
    /// The method.
    pub method: String,
    /// The params.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// A JSON-RPC error object.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RpcError {
    /// The error code.
    pub code: i32,
    /// The message.
    pub message: String,
    /// The data of the error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// One JSON-RPC response as read from the wire.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RpcResponse {
    /// The JSON-RPC version (`"2.0"`).
    pub jsonrpc: String,
    /// The id of the request.
    pub id: Value,
    /// The result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// The error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

/// A line from the agent: a response to a call, or a notification.
#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    /// A response to a call.
    Response(RpcResponse),
    /// A notification.
    Notification(Notification),
}

impl RpcError {
    /// An error with this code (see `code`) and no `data`.
    pub fn new(code: i32, message: impl Into<String>) -> RpcError {
        RpcError {
            code,
            message: message.into(),
            data: None,
        }
    }
}

/// Read a request line. Not JSON → `PARSE_ERROR`; not a JSON-RPC 2.0 request → `INVALID_REQUEST`.
pub fn parse_request(line: &str) -> Result<RpcRequest, RpcError> {
    let value: Value = serde_json::from_str(line)
        .map_err(|error| RpcError::new(code::PARSE_ERROR, error.to_string()))?;
    let request: RpcRequest = serde_json::from_value(value)
        .map_err(|error| RpcError::new(code::INVALID_REQUEST, error.to_string()))?;
    if request.jsonrpc != "2.0" {
        return Err(RpcError::new(
            code::INVALID_REQUEST,
            "jsonrpc must be \"2.0\"",
        ));
    }
    Ok(request)
}

/// Turn a request into a `Call`. Unknown method → `METHOD_NOT_FOUND`; bad params → `INVALID_PARAMS`.
/// Missing or `null` params are read as `{}` (so `Empty` methods need no params).
pub fn parse_call(request: &RpcRequest) -> Result<Call, RpcError> {
    if !METHOD_NAMES.contains(&request.method.as_str()) {
        return Err(RpcError::new(
            code::METHOD_NOT_FOUND,
            format!("unknown method: {}", request.method),
        ));
    }
    let envelope =
        json!({ "method": request.method, "params": params_or_empty(request.params.as_ref()) });
    serde_json::from_value(envelope)
        .map_err(|error| RpcError::new(code::INVALID_PARAMS, error.to_string()))
}

/// Read one line from the agent (a response or a notification). A line this
/// client does not understand is an `Err`; clients ignore it (a newer target).
pub fn parse_message(line: &str) -> Result<Message, RpcError> {
    let value: Value = serde_json::from_str(line)
        .map_err(|error| RpcError::new(code::PARSE_ERROR, error.to_string()))?;
    if value.get("method").is_some() {
        let envelope =
            json!({ "method": value["method"], "params": params_or_empty(value.get("params")) });
        return serde_json::from_value(envelope)
            .map(Message::Notification)
            .map_err(|error| RpcError::new(code::INVALID_PARAMS, error.to_string()));
    }
    serde_json::from_value(value)
        .map(Message::Response)
        .map_err(|error| RpcError::new(code::INVALID_REQUEST, error.to_string()))
}

/// The response for `id`.
pub fn response(id: Value, result: Result<Value, RpcError>) -> RpcResponse {
    match result {
        Ok(value) => RpcResponse {
            jsonrpc: "2.0".to_owned(),
            id,
            result: Some(value),
            error: None,
        },
        Err(error) => RpcResponse {
            jsonrpc: "2.0".to_owned(),
            id,
            result: None,
            error: Some(error),
        },
    }
}

/// One response as a line (no trailing newline).
pub fn response_line(response: &RpcResponse) -> String {
    serde_json::to_string(response).expect("invariant: protocol values serialize")
}

/// One request as a line (no trailing newline).
pub fn request_line(id: u64, method: &str, params: Value) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }).to_string()
}

/// One notification as a line (no trailing newline): `{"jsonrpc":"2.0","method":..,"params":..}`.
pub fn notification_line(notification: &Notification) -> String {
    let mut value =
        serde_json::to_value(notification).expect("invariant: protocol values serialize");
    value["jsonrpc"] = json!("2.0");
    value.to_string()
}

fn params_or_empty(params: Option<&Value>) -> Value {
    match params {
        None | Some(Value::Null) => json!({}),
        Some(value) => value.clone(),
    }
}
