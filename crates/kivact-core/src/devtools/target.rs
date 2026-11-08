//! Role: Route one DevTools call to the leaf module that owns its method.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::Runtime;

use super::a11y;
use super::console;
use super::emulation;
use super::input;
use super::motion;
use super::node;
use super::overlay;
use super::perf;
use super::query;
use super::review;
use super::scripts;
use super::styles;
use super::tree;
use super::updates;

type Route = fn(&Runtime, p::Call, Responder) -> Option<(p::Call, Responder)>;

const ROUTES: [Route; 14] = [
    tree::route,
    node::route,
    query::route,
    overlay::route,
    styles::route,
    updates::route,
    perf::route,
    a11y::route,
    input::route,
    motion::route,
    console::route,
    emulation::route,
    review::route,
    scripts::route,
];

/// Answer one call: the first leaf that owns its method takes it.
pub(crate) fn dispatch(rt: &Runtime, call: p::Call, reply: Responder) {
    let mut left = Some((call, reply));
    for route in ROUTES {
        let Some((call, reply)) = left.take() else {
            return;
        };
        left = route(rt, call, reply);
    }
    if let Some((_, reply)) = left {
        reply.send(Err(p::RpcError::new(
            p::code::INTERNAL_ERROR,
            "handled by the agent",
        )));
    }
}
