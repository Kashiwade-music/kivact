//! Role: Accessibility tree, audits, focus and layout lint of the DevTools target.

use std::collections::{BTreeMap, HashMap};

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_engine::api::Reader;
use kivact_runtime::Runtime;
use kivact_types::{
    CheckedState, Fill, FillImage, FocusTarget, LayoutRule, LiveRegion, NodeId, Rgba, Role,
    Semantics, WindowId,
};

use super::convert;
use crate::view::devtools_state::DevtoolsState;
use crate::view::state::ViewState;
use crate::view::tree::ViewNodes;

fn window_not_found(raw: p::WindowId) -> p::RpcError {
    p::RpcError::new(
        p::code::WINDOW_NOT_FOUND,
        format!("window {raw} does not exist"),
    )
}

fn open_window(reader: &Reader, raw: p::WindowId) -> Result<WindowId, p::RpcError> {
    convert::window_from(raw)
        .filter(|window| reader.windows().contains(window))
        .ok_or_else(|| window_not_found(raw))
}

fn zero_rect() -> p::Rect {
    p::Rect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    }
}

fn wire_action(action: kivact_types::A11yAction) -> p::A11yAction {
    use kivact_types::A11yAction as A;
    match action {
        A::Activate => p::A11yAction::Activate,
        A::Focus => p::A11yAction::Focus,
        A::Increment => p::A11yAction::Increment,
        A::Decrement => p::A11yAction::Decrement,
        A::Expand => p::A11yAction::Expand,
        A::Collapse => p::A11yAction::Collapse,
        A::Toggle => p::A11yAction::Toggle,
        A::Select => p::A11yAction::Select,
        A::ScrollIntoView => p::A11yAction::ScrollIntoView,
        A::SetValue => p::A11yAction::SetValue,
        A::ShowMenu => p::A11yAction::ShowMenu,
        A::Dismiss => p::A11yAction::Dismiss,
    }
}

fn state_names(semantics: &Semantics<FocusTarget>) -> Vec<String> {
    let mut states = Vec::new();
    if semantics.disabled {
        states.push("disabled".to_owned());
    }
    if semantics.hidden {
        states.push("hidden".to_owned());
    }
    if semantics.focusable {
        states.push("focusable".to_owned());
    }
    if semantics.selected == Some(true) {
        states.push("selected".to_owned());
    }
    if let Some(checked) = semantics.checked {
        states.push(
            match checked {
                CheckedState::Unchecked => "unchecked",
                CheckedState::Checked => "checked",
                CheckedState::Indeterminate => "mixed",
            }
            .to_owned(),
        );
    }
    if let Some(expanded) = semantics.expanded {
        states.push(if expanded { "expanded" } else { "collapsed" }.to_owned());
    }
    if semantics.invalid == Some(true) {
        states.push("invalid".to_owned());
    }
    if semantics.required == Some(true) {
        states.push("required".to_owned());
    }
    if semantics.read_only == Some(true) {
        states.push("read-only".to_owned());
    }
    if semantics.busy == Some(true) {
        states.push("busy".to_owned());
    }
    if semantics.protected {
        states.push("protected".to_owned());
    }
    match semantics.live_region {
        Some(LiveRegion::Polite) => states.push("live-polite".to_owned()),
        Some(LiveRegion::Assertive) => states.push("live-assertive".to_owned()),
        Some(LiveRegion::Off) | None => {}
    }
    states
}

fn semantics_info(semantics: &Semantics<FocusTarget>) -> p::SemanticsInfo {
    p::SemanticsInfo {
        role: semantics.role.map(convert::role),
        name: semantics.name.clone(),
        description: semantics.description.clone(),
        value: semantics.value_text.clone(),
        states: state_names(semantics),
        actions: semantics
            .actions
            .as_ref()
            .map(|actions| actions.iter().copied().map(wire_action).collect())
            .unwrap_or_default(),
        labelled_by: semantics
            .labelled_by
            .iter()
            .map(|target| target.node.0)
            .collect(),
        described_by: semantics
            .described_by
            .iter()
            .map(|target| target.node.0)
            .collect(),
        controls: semantics
            .controls
            .iter()
            .map(|target| target.node.0)
            .collect(),
        focusable: semantics.focusable,
    }
}

fn bounds(reader: &Reader, id: NodeId) -> p::Rect {
    reader
        .box_model(id)
        .map(|model| convert::rect(model.border))
        .unwrap_or_else(zero_rect)
}

pub(super) fn get_a11y_tree(
    rt: &Runtime,
    params: p::WindowParams,
) -> Result<p::A11yTree, p::RpcError> {
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    let window = open_window(&reader, params.window)?;
    let root = reader
        .root(window)
        .ok_or_else(|| window_not_found(params.window))?;
    let mut nodes: Vec<p::A11yNode> = Vec::new();
    let mut index: HashMap<NodeId, usize> = HashMap::new();
    let mut stack = vec![(root, None::<NodeId>)];
    while let Some((id, parent)) = stack.pop() {
        let mut child_parent = parent;
        if let Some(semantics) = reader.semantics(id) {
            index.insert(id, nodes.len());
            nodes.push(p::A11yNode {
                node: id.0,
                parent: parent.map(|parent| parent.0),
                children: Vec::new(),
                semantics: semantics_info(&semantics),
                bounds: bounds(&reader, id),
            });
            child_parent = Some(id);
        }
        for child in reader.children(id).into_iter().rev() {
            stack.push((child, child_parent));
        }
    }
    for position in 0..nodes.len() {
        if let Some(parent) = nodes[position].parent
            && let Some(&parent_index) = index.get(&NodeId(parent))
        {
            let child = nodes[position].node;
            nodes[parent_index].children.push(child);
        }
    }
    Ok(p::A11yTree {
        window: params.window,
        nodes,
    })
}

fn needs_name(role: Role) -> bool {
    matches!(
        role,
        Role::Button
            | Role::Link
            | Role::CheckBox
            | Role::Switch
            | Role::RadioButton
            | Role::TextBox
            | Role::ComboBox
            | Role::Slider
            | Role::Tab
            | Role::MenuItem
            | Role::TreeItem
            | Role::Option
    )
}

fn has_name(semantics: &Semantics<FocusTarget>) -> bool {
    semantics
        .name
        .as_deref()
        .is_some_and(|name| !name.is_empty())
}

fn finding(
    rule: p::AuditRule,
    node: NodeId,
    message: String,
    value: Option<f64>,
    limit: Option<f64>,
) -> p::AuditFinding {
    p::AuditFinding {
        rule,
        node: node.0,
        message,
        value,
        limit,
    }
}

fn nearest_background(reader: &Reader, id: NodeId) -> Option<Rgba> {
    let mut current = reader.parent(id);
    while let Some(node) = current {
        let color = reader
            .effective_paint(node)
            .background
            .as_ref()
            .and_then(fill_color);
        if let Some(color) = color
            && color.a >= 0.999
        {
            return Some(color);
        }
        current = reader.parent(node);
    }
    None
}

fn fill_color(fill: &Fill) -> Option<Rgba> {
    match fill {
        Fill::Solid(color) => Some(*color),
        Fill::Layers(layers) => layers.first().and_then(|layer| match &layer.image {
            FillImage::Solid(color) => Some(*color),
            _ => None,
        }),
    }
}

fn linear(channel: f64) -> f64 {
    let channel = if channel.is_finite() {
        channel.clamp(0.0, 1.0)
    } else {
        0.0
    };
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn luminance(color: Rgba) -> f64 {
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

fn over(foreground: Rgba, background: Rgba) -> Rgba {
    let alpha = if foreground.a.is_finite() {
        foreground.a.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mix = |front: f64, back: f64| front * alpha + back * (1.0 - alpha);
    Rgba {
        r: mix(foreground.r, background.r),
        g: mix(foreground.g, background.g),
        b: mix(foreground.b, background.b),
        a: 1.0,
    }
}

fn contrast_ratio(first: Rgba, second: Rgba) -> f64 {
    let first = luminance(first);
    let second = luminance(second);
    let (lighter, darker) = if first >= second {
        (first, second)
    } else {
        (second, first)
    };
    (lighter + 0.05) / (darker + 0.05)
}

fn text_contrast(
    tree: &ViewNodes,
    reader: &Reader,
    id: NodeId,
    findings: &mut Vec<p::AuditFinding>,
) {
    if tree.content(id).is_none() {
        return;
    }
    let Some(background) = nearest_background(reader, id) else {
        return;
    };
    let text = reader.computed_text(id);
    let foreground = over(text.color, background);
    let ratio = contrast_ratio(foreground, background);
    let limit = if text.font_size >= 18.0 || (text.font_size >= 14.0 && text.weight >= 700) {
        3.0
    } else {
        4.5
    };
    if ratio < limit {
        findings.push(finding(
            p::AuditRule::LowContrast,
            id,
            format!("contrast {ratio:.2}:1 is below {limit:.1}:1"),
            Some(ratio),
            Some(limit),
        ));
    }
}

fn audit_node(
    tree: &ViewNodes,
    reader: &Reader,
    id: NodeId,
    semantics: Option<&Semantics<FocusTarget>>,
    touch: bool,
    findings: &mut Vec<p::AuditFinding>,
) {
    let role = semantics.and_then(|semantics| semantics.role);
    let named = semantics.is_some_and(has_name);
    if let Some(role) = role
        && needs_name(role)
        && !named
    {
        findings.push(finding(
            p::AuditRule::MissingName,
            id,
            format!("{role:?} has no accessible name"),
            None,
            None,
        ));
    }
    let focusable = tree.focusable(id).unwrap_or(false)
        || semantics.is_some_and(|semantics| semantics.focusable);
    if focusable && role.is_none() {
        findings.push(finding(
            p::AuditRule::FocusableWithoutRole,
            id,
            "focusable node has no role".to_owned(),
            None,
            None,
        ));
    }
    if role == Some(Role::Image) && !named {
        findings.push(finding(
            p::AuditRule::UnlabelledImage,
            id,
            "image has no accessible name".to_owned(),
            None,
            None,
        ));
    }
    let on_press = tree
        .get(id)
        .and_then(|node| node.handlers.as_deref())
        .is_some_and(|handlers| handlers.activate.is_some());
    if (focusable || on_press)
        && let Some(model) = reader.box_model(id)
    {
        let minimum = if touch { 44.0 } else { 24.0 };
        let short = model.border.width.min(model.border.height);
        if short < minimum {
            findings.push(finding(
                p::AuditRule::SmallTarget,
                id,
                format!(
                    "target is {:.0}x{:.0}, below {minimum:.0}x{minimum:.0}",
                    model.border.width, model.border.height
                ),
                Some(short),
                Some(minimum),
            ));
        }
    }
}

fn collect_ids(reader: &Reader, root: NodeId) -> Vec<NodeId> {
    let mut ids = Vec::new();
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        ids.push(id);
        stack.extend(reader.children(id));
    }
    ids
}

pub(super) fn audit_a11y(
    rt: &Runtime,
    params: p::WindowParams,
) -> Result<p::AuditReport, p::RpcError> {
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    let window = open_window(&reader, params.window)?;
    let root = reader
        .root(window)
        .ok_or_else(|| window_not_found(params.window))?;
    let touch = ViewState::of(rt).capabilities().touch;
    let mut findings = Vec::new();
    let mut test_ids: BTreeMap<String, Vec<NodeId>> = BTreeMap::new();
    for id in collect_ids(&reader, root) {
        let semantics = reader.semantics(id);
        if let Some(test_id) = semantics
            .as_ref()
            .and_then(|semantics| semantics.test_id.as_ref())
        {
            test_ids.entry(test_id.clone()).or_default().push(id);
        }
        audit_node(&tree, &reader, id, semantics.as_ref(), touch, &mut findings);
        text_contrast(&tree, &reader, id, &mut findings);
    }
    for (test_id, ids) in test_ids {
        if ids.len() >= 2 {
            findings.push(finding(
                p::AuditRule::DuplicateTestId,
                ids[0],
                format!("test id {test_id:?} is used by {} nodes", ids.len()),
                None,
                None,
            ));
        }
    }
    Ok(p::AuditReport {
        window: params.window,
        findings,
    })
}

/// Whether the focus ring of `window` is visible.
pub(super) fn focus_visible(rt: &Runtime, window: WindowId) -> bool {
    ViewState::of(rt)
        .focus_keys(window)
        .and_then(|keys| keys.focus.try_get_untracked())
        .flatten()
        .is_some_and(|key| key.visible)
}

pub(super) fn get_focus(
    rt: &Runtime,
    params: p::WindowParams,
) -> Result<p::FocusInfo, p::RpcError> {
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    let window = open_window(&reader, params.window)?;
    let i = DevtoolsState::inspector(rt);
    let history = i
        .hooks
        .focus_history
        .borrow()
        .get(&window)
        .map(|moves| moves.iter().cloned().collect())
        .unwrap_or_default();
    let visible = focus_visible(rt, window);
    Ok(p::FocusInfo {
        window: params.window,
        node: reader.focused(window).map(|target| target.node.0),
        visible,
        history,
    })
}

pub(super) fn get_tab_order(
    rt: &Runtime,
    params: p::WindowParams,
) -> Result<p::TabOrder, p::RpcError> {
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    let window = open_window(&reader, params.window)?;
    Ok(p::TabOrder {
        window: params.window,
        nodes: reader
            .tab_order(window)
            .into_iter()
            .map(|target| target.node.0)
            .collect(),
    })
}

fn layout_rule(rule: LayoutRule) -> p::LayoutRuleKind {
    match rule {
        LayoutRule::Overflow {
            into: kivact_types::LayoutLimit::OwnBox,
        } => p::LayoutRuleKind::OverflowOwnBox,
        LayoutRule::Overflow {
            into: kivact_types::LayoutLimit::VisualParent,
        } => p::LayoutRuleKind::OverflowParent,
        LayoutRule::Wrapped { .. } => p::LayoutRuleKind::Wrapped,
        LayoutRule::OffCenter => p::LayoutRuleKind::OffCenter,
        LayoutRule::ClippedWithoutEllipsis => p::LayoutRuleKind::ClippedWithoutEllipsis,
    }
}

pub(super) fn lint_layout(
    rt: &Runtime,
    params: p::WindowParams,
) -> Result<p::LayoutLint, p::RpcError> {
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    let window = open_window(&reader, params.window)?;
    let findings = reader
        .layout_findings(window, &kivact_types::LayoutCheck::default())
        .into_iter()
        .map(|found| p::LayoutFindingInfo {
            node: found.node.0,
            rule: layout_rule(found.rule),
            amount: found.amount,
            text_rect: convert::rect(found.text_rect),
            limit_rect: convert::rect(found.limit_rect),
        })
        .collect();
    Ok(p::LayoutLint {
        window: params.window,
        findings,
    })
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
