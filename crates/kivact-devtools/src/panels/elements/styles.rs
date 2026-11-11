//! Role: The Styles tab of the Elements panel.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::{Inspected, Session};
use crate::widgets::override_row::{Override, override_row};
use crate::widgets::property_row::RowAccess;

/// The properties a style override can set; every other row is read-only.
const EDITABLE: &[&str] = &[
    "width",
    "height",
    "min_width",
    "min_height",
    "max_width",
    "max_height",
    "padding",
    "padding_top",
    "padding_right",
    "padding_bottom",
    "padding_left",
    "margin",
    "margin_top",
    "margin_right",
    "margin_bottom",
    "margin_left",
    "gap",
    "row_gap",
    "column_gap",
    "grow",
    "shrink",
    "opacity",
    "background",
    "color",
    "border_color",
    "border_width",
    "radius",
    "font_size",
    "line_height",
    "weight",
    "text_align",
    "display",
];

/// The hint of a row whose value names no theme token while the token scope is selected.
const NOT_A_TOKEN: &str = "This property does not use a theme token";

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

/// One interactive state a row can force (the toggle group's key).
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

impl ForcedState {
    /// The state the protocol names.
    fn from_protocol(state: p::InteractiveState) -> ForcedState {
        match state {
            p::InteractiveState::Hovered => ForcedState::Hovered,
            p::InteractiveState::Pressed => ForcedState::Pressed,
            p::InteractiveState::Focused => ForcedState::Focused,
            p::InteractiveState::FocusVisible => ForcedState::FocusVisible,
            p::InteractiveState::Disabled => ForcedState::Disabled,
            p::InteractiveState::Selected => ForcedState::Selected,
        }
    }

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

/// The override a row edits: the scope it applies to and the property it sets.
#[derive(Clone, PartialEq)]
struct OverrideKey {
    /// What the override applies to.
    scope: p::OverrideScope,
    /// The property the override sets (the token name under the token scope).
    property: String,
}

/// The style layers and overrides of the selected node.
pub(crate) fn styles_tab(
    session: Session,
    detail: Resource<Option<p::NodeDetail>>,
) -> impl IntoView {
    let scope_signal = signal(Some(Scope::Node));
    let tokens = session.read::<p::ListThemeTokens>();
    let state = memo(move || {
        (
            detail.latest().flatten(),
            session
                .overrides
                .latest()
                .map(|list| list.overrides)
                .unwrap_or_default(),
            scope_signal.get(),
            tokens.latest().map(|list| list.tokens).unwrap_or_default(),
        )
    });
    dynamic(state, move |(detail, overrides, selected, tokens)| {
        let scope = selected.unwrap_or(Scope::Node);
        match detail {
            None => text("Select a node to edit its styles").into_view(),
            Some(detail) => {
                styles_body(session, detail, overrides, tokens, scope_signal, scope).into_view()
            }
        }
    })
}

/// The scope select, the forced states and every style layer.
fn styles_body(
    session: Session,
    detail: p::NodeDetail,
    overrides: Vec<p::StyleOverride>,
    tokens: Vec<p::ThemeToken>,
    scope_signal: Signal<Option<Scope>>,
    scope: Scope,
) -> impl IntoView {
    let node = detail.node.id;
    let call_site = call_site(&detail);
    let forced = signal(
        detail
            .forced
            .iter()
            .map(|state| ForcedState::from_protocol(*state))
            .collect::<Vec<_>>(),
    );
    effect_on(forced, move |states| force_states(session, node, states));
    let mut sections: Vec<View> = Vec::new();
    for layer in &detail.styles {
        sections.push(layer_section(
            session,
            node,
            &overrides,
            &tokens,
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
            button("Record change to review")
                .disabled(move || session.inspected.with(Inspected::selected).is_none())
                .on_press(move || record_change(session)),
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
    tokens: &[p::ThemeToken],
    scope: Scope,
    call_site: Option<p::SourceLocation>,
    layer: &p::StyleLayerInfo,
) -> View {
    let mut rows: Vec<View> = layer
        .properties
        .iter()
        .map(|property| {
            property_row_view(
                session,
                node,
                overrides,
                tokens,
                scope,
                call_site.clone(),
                property,
            )
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

/// One property row; color properties get the color input. A row no override can
/// set is read-only, and under the token scope a row whose value names no token is unavailable.
fn property_row_view(
    session: Session,
    node: p::NodeId,
    overrides: &[p::StyleOverride],
    tokens: &[p::ThemeToken],
    scope: Scope,
    call_site: Option<p::SourceLocation>,
    property: &p::StyleProperty,
) -> View {
    let name = property.name.clone();
    let editable = EDITABLE.contains(&name.as_str());
    let target = if editable {
        override_target(scope, node, call_site, property, tokens)
    } else {
        None
    };
    let access = match (editable, &target) {
        (false, _) => RowAccess::ReadOnly,
        (true, None) => RowAccess::Unavailable(NOT_A_TOKEN),
        (true, Some(_)) => RowAccess::Editable,
    };
    let (key, start) = match target {
        Some((key, start)) => (Some(key), start),
        None => (None, property.value.clone()),
    };
    let existing_id = key
        .as_ref()
        .and_then(|key| find_override_id(overrides, key));
    let initial = match existing_id {
        Some(id) => Override::On {
            id: Some(id),
            value: start,
        },
        None => Override::Off { value: start },
    };
    let commit_key = key;
    let color = is_color_property(&name);
    override_row(
        name,
        initial,
        color,
        access,
        move |value| {
            if let Some(key) = &commit_key {
                set_override(session, key.clone(), value);
            }
        },
        move |id| clear_override(session, id),
    )
}

/// The override a row of `property` edits under `scope`, with the value a new one starts with.
/// `None` when the scope cannot apply to the row (the token scope and a value that names no token).
fn override_target(
    scope: Scope,
    node: p::NodeId,
    call_site: Option<p::SourceLocation>,
    property: &p::StyleProperty,
    tokens: &[p::ThemeToken],
) -> Option<(OverrideKey, String)> {
    let name = property.name.clone();
    match scope {
        Scope::Node => Some((
            OverrideKey {
                scope: p::OverrideScope::Node { node },
                property: name,
            },
            property.value.clone(),
        )),
        Scope::CallSite => {
            let scope = match call_site {
                Some(location) => p::OverrideScope::CallSite { location },
                None => p::OverrideScope::Node { node },
            };
            Some((
                OverrideKey {
                    scope,
                    property: name,
                },
                property.value.clone(),
            ))
        }
        Scope::Token => token_named(&property.value, tokens).map(|token| {
            (
                OverrideKey {
                    scope: p::OverrideScope::Token {
                        token: token.name.clone(),
                    },
                    property: token.name.clone(),
                },
                token.value.clone(),
            )
        }),
    }
}

/// The theme token a row value names: the token of that exact name, or the color token `<name>`
/// whose name is `colors.<name>` (the form the override values use).
fn token_named<'a>(value: &str, tokens: &'a [p::ThemeToken]) -> Option<&'a p::ThemeToken> {
    let value = value.trim();
    tokens
        .iter()
        .find(|token| token.name == value || token.name.strip_prefix("colors.") == Some(value))
}

/// The id of the override that `key` edits, if the agent has one.
fn find_override_id(overrides: &[p::StyleOverride], key: &OverrideKey) -> Option<u64> {
    overrides
        .iter()
        .rev()
        .find(|entry| entry.scope == key.scope && entry.property == key.property)
        .map(|entry| entry.id)
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

/// Set one override (the agent replaces an override of the same scope and property) and refresh
/// the facts the panel shows.
fn set_override(session: Session, key: OverrideKey, value: String) {
    session.send::<p::SetStyleOverride>(
        p::SetStyleOverrideParams {
            scope: key.scope,
            property: key.property,
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

/// Add the selected node's change to the review, with a screenshot.
fn record_change(session: Session) {
    let Some(node) = session.inspected.with(Inspected::selected) else {
        return;
    };
    session.send::<p::AddComment>(
        p::AddCommentParams {
            window: session.window(),
            node: Some(node),
            text: "Style change".to_owned(),
            screenshot: true,
        },
        move |_| session.review.refetch(),
    );
}
