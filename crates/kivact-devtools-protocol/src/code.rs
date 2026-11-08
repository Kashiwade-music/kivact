//! Role: JSON-RPC error codes of the DevTools protocol (design 4.2.1).

/// The line is not JSON.
pub const PARSE_ERROR: i32 = -32700;
/// The line is not a JSON-RPC 2.0 request.
pub const INVALID_REQUEST: i32 = -32600;
/// The method is unknown.
pub const METHOD_NOT_FOUND: i32 = -32601;
/// The params are invalid.
pub const INVALID_PARAMS: i32 = -32602;
/// The agent failed internally.
pub const INTERNAL_ERROR: i32 = -32603;
/// The token is wrong.
pub const BAD_TOKEN: i32 = -32001;
/// The connection has not sent `initialize` yet.
pub const NOT_INITIALIZED: i32 = -32002;
/// The protocol version of the client differs from the target's.
pub const VERSION_MISMATCH: i32 = -32003;
/// The node does not exist (including an unmounted node).
pub const NODE_NOT_FOUND: i32 = -32010;
/// The window does not exist.
pub const WINDOW_NOT_FOUND: i32 = -32011;
/// A query found no node.
pub const NO_MATCH: i32 = -32012;
/// A query found more than one node.
pub const MULTIPLE_MATCHES: i32 = -32013;
/// The wait of a `find_*` call timed out.
pub const TIMED_OUT: i32 = -32014;
/// The target cannot do this (no multi-window, no GPU, embed input, ...).
pub const UNSUPPORTED: i32 = -32015;
