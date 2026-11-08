//! Role: The review of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::Runtime;

fn unsupported<T>() -> Result<T, p::RpcError> {
    Err(p::RpcError::new(
        p::code::UNSUPPORTED,
        "not implemented yet",
    ))
}

pub(super) fn get_review(rt: &Runtime, params: p::Empty) -> Result<p::ReviewDocument, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn add_comment(rt: &Runtime, params: p::AddCommentParams, reply: Responder) {
    let _ = (rt, params);
    reply.send(unsupported());
}

pub(super) fn edit_comment(
    rt: &Runtime,
    params: p::EditCommentParams,
) -> Result<p::ReviewItem, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn remove_comment(
    rt: &Runtime,
    params: p::RemoveCommentParams,
) -> Result<p::Empty, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn export_review(
    rt: &Runtime,
    params: p::Empty,
) -> Result<p::ExportedReview, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::GetReview(params) => reply.send(get_review(rt, params).map(p::Reply::GetReview)),
        p::Call::AddComment(params) => add_comment(rt, params, reply),
        p::Call::EditComment(params) => {
            reply.send(edit_comment(rt, params).map(p::Reply::EditComment))
        }
        p::Call::RemoveComment(params) => {
            reply.send(remove_comment(rt, params).map(p::Reply::RemoveComment))
        }
        p::Call::ExportReview(params) => {
            reply.send(export_review(rt, params).map(p::Reply::ExportReview))
        }
        other => return Some((other, reply)),
    }
    None
}
