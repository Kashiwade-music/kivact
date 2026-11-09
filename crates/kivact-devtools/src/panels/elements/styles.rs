//! Role: The Styles tab of the Elements panel.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;
use crate::widgets::property_row::{color_property_row, property_row};

/// What a new style override applies to.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Scope {
    /// One node.
    Node,
    /// One call site.
    CallSite,
    /// One theme token.
    Token,
}

/// One interactive state a row can force.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ForcedState {
    /// Hovered.
    Hovered,
    /// Pressed.
    Pressed,
    /// Focused.
    Focused,
    /// Focus visible.
    FocusVisible,
    /// Disabled.
    Disabled,
    /// Selected.
    Selected,
}

/// The style layers and overrides of the selected node.
pub(crate) fn styles_tab(
    session: Session,
    detail: Resource<Option<p::NodeDetail>>,
) -> impl IntoView {
    let scope_signal = signal(Some(Scope::Node));
    let state = memo(move || {
        (
            detail.latest().flatten(),
            session
                .overrides
                .latest()
                .map(|list| list.overrides)
                .unwrap_or_default(),
            scope_signal.get(),
        )
    });
    dynamic(state, move |(detail, overrides, selected)| {
        let scope = selected.unwrap_or(Scope::Node);
        match detail {
            None => text("Select a node to edit its styles").into_view(),
            Some(detail) => {
                styles_body(session, detail, overrides, scope_signal, scope).into_view()
            }
        }
    })
}

/// The scope select, the forced states and every style layer.
fn styles_body(
    session: Session,
    detail: p::NodeDetail,
    overrides: Vec<p::StyleOverride>,
    scope_signal: Signal<Option<Scope>>,
    scope: Scope,
) -> impl IntoView {
    let node = detail.node.id;
    let call_site = call_site(&detail);
    let forced = signal(forced_states(&detail));
    effect_on(forced, move |states| force_states(session, node, states));
    let mut sections: Vec<View> = Vec::new();
    for layer in &detail.styles {
        sections.push(layer_section(
            session,
            node,
            &overrides,
            scope,
            call_site.clone(),
            layer,
        ));
    }
    if sections.is_empty() {
        sections.push(text("No style layers").into_view());
    }
    column((
        row((
            text("Override scope").style(style().font_size(12.0)),
            select("Override scope", scope_items(), scope_signal).style(style().width(230)),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        toggle_group("Forced states", state_items(), forced).style(style().min_width(0)),
        scroll(column(sections)).style(style().grow(1).min_height(0)),
    ))
    .style(style().grow(1).min_height(0).gap(8).padding(8))
}

/// One style layer with a heading and one row per property.
fn layer_section(
    session: Session,
    node: p::NodeId,
    overrides: &[p::StyleOverride],
    scope: Scope,
    call_site: Option<p::SourceLocation>,
    layer: &p::StyleLayerInfo,
) -> View {
    let mut rows: Vec<View> = layer
        .properties
        .iter()
        .map(|property| {
            property_row_view(session, node, overrides, scope, call_site.clone(), property)
        })
        .collect();
    if rows.is_empty() {
        rows.push(
            text("No properties")
                .style(style().font_size(11.0))
                .into_view(),
        );
    }
    column((
        text(layer_title(layer)).style(style().font_size(12.0).weight(600).padding_y(2)),
        column(rows),
    ))
    .style(style().gap(4).padding_y(4))
    .into_view()
}

/// One property row; color properties get the color input.
fn property_row_view(
    session: Session,
    node: p::NodeId,
    overrides: &[p::StyleOverride],
    scope: Scope,
    call_site: Option<p::SourceLocation>,
    property: &p::StyleProperty,
) -> View {
    let name = property.name.clone();
    let existing = find_override(overrides, &name, scope, node, call_site.as_ref());
    let enabled = signal(existing.is_some());
    let value = signal(property.value.clone());
    let existing_id = existing.map(|entry| entry.id);
    let toggle_scope = override_scope(scope, node, call_site.clone(), &name);
    let toggle_property = name.clone();
    let toggle_value = value;
    effect_on(enabled, move |on| {
        if *on {
            set_override(
                session,
                toggle_scope.clone(),
                toggle_property.clone(),
                toggle_value.get_untracked(),
            );
        } else if let Some(id) = existing_id {
            clear_override(session, id);
        }
    });
    let commit_name = name.clone();
    let commit_scope = override_scope(scope, node, call_site, &commit_name);
    let commit = move |text: String| {
        set_override(session, commit_scope.clone(), commit_name.clone(), text);
    };
    if is_color_property(&name) {
        color_property_row(name, value, enabled, commit).into_view()
    } else {
        property_row(name, value, enabled, commit).into_view()
    }
}

/// The override for `property` under the current scope, if any.
fn find_override(
    overrides: &[p::StyleOverride],
    property: &str,
    scope: Scope,
    node: p::NodeId,
    call_site: Option<&p::SourceLocation>,
) -> Option<p::StyleOverride> {
    overrides
        .iter()
        .rev()
        .find(|entry| {
            entry.property == property
                && match (&entry.scope, scope) {
                    (p::OverrideScope::Node { node: target }, Scope::Node) => *target == node,
                    (p::OverrideScope::CallSite { location }, Scope::CallSite) => {
                        call_site == Some(location)
                    }
                    (p::OverrideScope::Token { token }, Scope::Token) => token == property,
                    _ => false,
                }
        })
        .cloned()
}

/// The scope a new override of `property` gets.
fn override_scope(
    scope: Scope,
    node: p::NodeId,
    call_site: Option<p::SourceLocation>,
    property: &str,
) -> p::OverrideScope {
    match scope {
        Scope::Node => p::OverrideScope::Node { node },
        Scope::CallSite => match call_site {
            Some(location) => p::OverrideScope::CallSite { location },
            None => p::OverrideScope::Node { node },
        },
        Scope::Token => p::OverrideScope::Token {
            token: property.to_owned(),
        },
    }
}

/// The first app-code frame of the call chain, else its first frame.
fn call_site(detail: &p::NodeDetail) -> Option<p::SourceLocation> {
    detail
        .call_chain
        .iter()
        .find(|frame| frame.app_code)
        .or_else(|| detail.call_chain.first())
        .map(|frame| frame.location.clone())
}

/// The states the style layers mark as forced.
fn forced_states(detail: &p::NodeDetail) -> Vec<ForcedState> {
    let mut states = Vec::new();
    for layer in &detail.styles {
        let Some(name) = layer
            .name
            .as_deref()
            .and_then(|name| name.strip_suffix(" (forced)"))
        else {
            continue;
        };
        let Some(state) = state_named(name) else {
            continue;
        };
        if !states.contains(&state) {
            states.push(state);
        }
    }
    states
}

/// The interactive state a style layer name stands for.
fn state_named(name: &str) -> Option<ForcedState> {
    Some(match name {
        "hover" => ForcedState::Hovered,
        "pressed" => ForcedState::Pressed,
        "focus" => ForcedState::Focused,
        "focus_visible" => ForcedState::FocusVisible,
        "disabled" => ForcedState::Disabled,
        "selected" => ForcedState::Selected,
        _ => return None,
    })
}

/// The toggle items of the forced states.
fn state_items() -> Vec<ListItem<ForcedState>> {
    [
        (ForcedState::Hovered, "Hovered"),
        (ForcedState::Pressed, "Pressed"),
        (ForcedState::Focused, "Focused"),
        (ForcedState::FocusVisible, "Focus visible"),
        (ForcedState::Disabled, "Disabled"),
        (ForcedState::Selected, "Selected"),
    ]
    .into_iter()
    .map(|(state, label)| ListItem::new(state, label))
    .collect()
}

/// The choices of the override scope.
fn scope_items() -> Vec<ListItem<Scope>> {
    vec![
        ListItem::new(Scope::Node, "This node"),
        ListItem::new(Scope::CallSite, "All nodes from this call site"),
        ListItem::new(Scope::Token, "Theme token"),
    ]
}

/// The heading of one layer.
fn layer_title(layer: &p::StyleLayerInfo) -> String {
    let named = |fallback: &str| {
        layer
            .name
            .clone()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| fallback.to_owned())
    };
    match layer.origin {
        p::StyleOrigin::Devtools => "DevTools overrides".to_owned(),
        p::StyleOrigin::State => named("State"),
        p::StyleOrigin::Selected => "Selected".to_owned(),
        p::StyleOrigin::Disabled => "Disabled".to_owned(),
        p::StyleOrigin::Caller => "Caller".to_owned(),
        p::StyleOrigin::Recipe => named("Recipe"),
        p::StyleOrigin::Base => "Base".to_owned(),
        p::StyleOrigin::ThemeDefault => "Theme default".to_owned(),
        p::StyleOrigin::Inherited => "Inherited".to_owned(),
    }
}

/// Whether a property is edited with a color input.
fn is_color_property(name: &str) -> bool {
    matches!(name, "background" | "color" | "border_color")
}

/// Add one override and refresh the facts the panel shows.
fn set_override(session: Session, scope: p::OverrideScope, property: String, value: String) {
    session.send::<p::SetStyleOverride>(
        p::SetStyleOverrideParams {
            scope,
            property,
            value,
        },
        move |_| {
            session.overrides.refetch();
            session.detail.refetch();
        },
    );
}

/// Remove one override and refresh the facts the panel shows.
fn clear_override(session: Session, id: u64) {
    session.send::<p::ClearStyleOverrides>(
        p::ClearStyleOverridesParams { id: Some(id) },
        move |_| {
            session.overrides.refetch();
            session.detail.refetch();
        },
    );
}

/// Force the interactive states of `node`.
fn force_states(session: Session, node: p::NodeId, states: &[ForcedState]) {
    session.send::<p::ForceState>(
        p::ForceStateParams {
            node,
            states: states.iter().map(|state| state.protocol()).collect(),
        },
        move |_| session.detail.refetch(),
    );
}

impl ForcedState {
    /// The protocol value of this state.
    fn protocol(self) -> p::InteractiveState {
        match self {
            ForcedState::Hovered => p::InteractiveState::Hovered,
            ForcedState::Pressed => p::InteractiveState::Pressed,
            ForcedState::Focused => p::InteractiveState::Focused,
            ForcedState::FocusVisible => p::InteractiveState::FocusVisible,
            ForcedState::Disabled => p::InteractiveState::Disabled,
            ForcedState::Selected => p::InteractiveState::Selected,
        }
    }
}
