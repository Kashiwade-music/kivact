//! Role: What the agent hands the core: calls to answer and connection endings.

use serde_json::Value;

use kivact_devtools_protocol::{Call, Reply, RpcError, response, response_line};

use crate::session::{Writer, write_line};

/// Something that arrived since the last `Agent::take_incoming`.
#[allow(
    clippy::large_enum_variant,
    reason = "one short-lived value per call; boxing would only add noise"
)]
pub enum Incoming {
    /// A call to answer through its `Responder`.
    Call(Call, Responder),
    /// An initialized connection ended; `remaining` initialized connections are still open.
    Closed { remaining: usize },
}

/// Where the reply of one call goes. Answer every call exactly once (`send` consumes it);
/// a client gives up on an unanswered call after 30 s. There is no `Drop`: a call dropped
/// without an answer is simply never answered.
pub struct Responder {
    id: Value,
    writer: Writer,
}

impl Responder {
    pub(crate) fn new(id: Value, writer: Writer) -> Responder {
        Responder { id, writer }
    }

    /// Write the JSON-RPC response line of this call to its connection.
    pub fn send(self, result: Result<Reply, RpcError>) {
        let answer = response(self.id, result.map(Reply::into_value));
        write_line(&self.writer, &response_line(&answer));
    }
}
