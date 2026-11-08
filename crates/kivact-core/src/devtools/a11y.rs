//! Role: Accessibility tree, audits, focus and layout lint of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::Runtime;

fn unsupported<T>() -> Result<T, p::RpcError> {
    Err(p::RpcError::new(
        p::code::UNSUPPORTED,
        "not implemented yet",
    ))
}

pub(super) fn get_a11y_tree(
    rt: &Runtime,
    params: p::WindowParams,
) -> Result<p::A11yTree, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn audit_a11y(
    rt: &Runtime,
    params: p::WindowParams,
) -> Result<p::AuditReport, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn get_focus(
    rt: &Runtime,
    params: p::WindowParams,
) -> Result<p::FocusInfo, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn get_tab_order(
    rt: &Runtime,
    params: p::WindowParams,
) -> Result<p::TabOrder, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn lint_layout(
    rt: &Runtime,
    params: p::WindowParams,
) -> Result<p::LayoutLint, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::GetA11yTree(params) => {
            reply.send(get_a11y_tree(rt, params).map(p::Reply::GetA11yTree))
        }
        p::Call::AuditA11y(params) => reply.send(audit_a11y(rt, params).map(p::Reply::AuditA11y)),
        p::Call::GetFocus(params) => reply.send(get_focus(rt, params).map(p::Reply::GetFocus)),
        p::Call::GetTabOrder(params) => {
            reply.send(get_tab_order(rt, params).map(p::Reply::GetTabOrder))
        }
        p::Call::LintLayout(params) => {
            reply.send(lint_layout(rt, params).map(p::Reply::LintLayout))
        }
        other => return Some((other, reply)),
    }
    None
}
