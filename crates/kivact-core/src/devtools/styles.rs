//! Role: Style overrides, forced states and theme tokens of the DevTools target.

use std::collections::{BTreeMap, HashSet};
use std::panic::Location;

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::driver::Runtime;
use kivact_runtime::{Owner, pure, untrack};
use kivact_style::Style;
use kivact_style::internal::{ForcedStates, describe_state_styles, describe_style, style_property};
use kivact_types::{NodeId, WindowId};

use super::convert;
use crate::devtools::apply;
use crate::devtools::state::{DevtoolsState, Inspector};
use crate::view::RecipeName;
use crate::view::devtools_hooks::NodeOverrides;
use crate::view::element_data::{LayerOrigin, StyleLayer};
use crate::view::state::ViewState;

/// The value of `property` on `id` without any DevTools override, in the form
/// an override takes.
pub(super) fn base_value(rt: &Runtime, id: NodeId, property: &str) -> Option<String> {
    let (layers, owner, _) = style_source(rt, id)?;
    let mut merged = Style::default();
    for layer in &layers {
        merged = merged.merge(&evaluate(owner, layer));
    }
    let definition = style_property(property)?;
    (definition.describe)(&merged)
}

/// The style layers of `id`, highest priority first.
pub(super) fn layers(rt: &Runtime, id: NodeId) -> Vec<p::StyleLayerInfo> {
    let Some((layers, owner, at)) = style_source(rt, id) else {
        return Vec::new();
    };
    let i = DevtoolsState::inspector(rt);
    let applied = apply::node_overrides(
        &i.hooks.overrides.get_untracked(),
        id,
        &apply::call_sites(rt, owner, at),
    );
    let forced = i
        .hooks
        .forced
        .get_untracked()
        .get(&id)
        .copied()
        .unwrap_or_default();
    let mut evaluated = Vec::with_capacity(layers.len());
    let mut merged = Style::default();
    for layer in &layers {
        let style = evaluate(owner, layer);
        merged = merged.merge(&style);
        evaluated.push((style, layer));
    }
    let mut out = Vec::new();
    if !applied.is_empty() {
        out.push(p::StyleLayerInfo {
            origin: p::StyleOrigin::Devtools,
            name: None,
            source: None,
            properties: properties(applied),
        });
    }
    out.extend(state_layers(&merged, forced));
    for (style, layer) in evaluated.iter().rev() {
        out.push(source_layer(owner, style, layer));
    }
    mark_overridden(&mut out);
    out
}

/// Remove every override and forced state.
pub(super) fn clear_all(rt: &Runtime) {
    let i = DevtoolsState::inspector(rt);
    i.hooks.overrides.set(Vec::new());
    rebuild_node_overrides(i);
    i.hooks.forced.set(BTreeMap::new());
}

/// Parse the override list once into the table the style hooks read.
fn rebuild_node_overrides(i: &Inspector) {
    let mut table = NodeOverrides::default();
    for entry in i.hooks.overrides.get_untracked() {
        let Ok(style) = apply::override_style(&entry.property, &entry.value) else {
            continue;
        };
        match entry.scope {
            p::OverrideScope::CallSite { location } => {
                table
                    .by_site
                    .push((location.file, location.line, location.column, style));
            }
            p::OverrideScope::Node { node } => {
                let id = NodeId(node);
                let merged = match table.by_node.remove(&id) {
                    Some(before) => before.merge(&style),
                    None => style,
                };
                table.by_node.insert(id, merged);
            }
            _ => {}
        }
    }
    *i.hooks.node_overrides.borrow_mut() = table;
    i.hooks.node_overrides_changed.notify();
}

pub(super) fn set_style_override(
    rt: &Runtime,
    params: p::SetStyleOverrideParams,
) -> Result<p::StyleOverride, p::RpcError> {
    match &params.scope {
        p::OverrideScope::Node { .. } | p::OverrideScope::CallSite { .. } => {
            apply::override_style(&params.property, &params.value).map_err(invalid_params)?;
        }
        p::OverrideScope::Token { token } => {
            validate_token(rt, token, &params.property, &params.value)?;
        }
    }
    let i = DevtoolsState::inspector(rt);
    let mut all = i.hooks.overrides.get_untracked();
    let entry = store_override(
        &mut all,
        params.scope,
        params.property,
        params.value,
        || {
            let id = i.session.next_override.get() + 1;
            i.session.next_override.set(id);
            id
        },
    );
    i.hooks.overrides.set(all);
    rebuild_node_overrides(i);
    Ok(entry)
}

/// Store an override in `all`. An override with the same scope and property
/// keeps its id and takes the new value; otherwise `next_id` gives the id of a
/// new override, which is appended.
fn store_override(
    all: &mut Vec<p::StyleOverride>,
    scope: p::OverrideScope,
    property: String,
    value: String,
    next_id: impl FnOnce() -> u64,
) -> p::StyleOverride {
    if let Some(slot) = all
        .iter_mut()
        .find(|slot| slot.scope == scope && slot.property == property)
    {
        slot.value = value;
        return slot.clone();
    }
    let entry = p::StyleOverride {
        id: next_id(),
        scope,
        property,
        value,
    };
    all.push(entry.clone());
    entry
}

pub(super) fn clear_style_overrides(
    rt: &Runtime,
    params: p::ClearStyleOverridesParams,
) -> Result<p::Empty, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    match params.id {
        Some(id) => i
            .hooks
            .overrides
            .update(|all| all.retain(|entry| entry.id != id)),
        None => i.hooks.overrides.set(Vec::new()),
    }
    rebuild_node_overrides(i);
    Ok(p::Empty {})
}

pub(super) fn list_style_overrides(
    rt: &Runtime,
    _: p::Empty,
) -> Result<p::StyleOverrideList, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    Ok(p::StyleOverrideList {
        overrides: i.hooks.overrides.get_untracked(),
    })
}

pub(super) fn force_state(
    rt: &Runtime,
    params: p::ForceStateParams,
) -> Result<p::Empty, p::RpcError> {
    let node = NodeId(params.node);
    if ViewState::tree(rt).get(node).is_none() {
        return Err(p::RpcError::new(
            p::code::NODE_NOT_FOUND,
            format!("node {} does not exist", params.node),
        ));
    }
    let mut states = ForcedStates::default();
    for state in params.states {
        states.insert(convert::forced_state(state));
    }
    let i = DevtoolsState::inspector(rt);
    i.hooks.forced.update(|all| {
        if states.is_empty() {
            all.remove(&node);
        } else {
            all.insert(node, states);
        }
    });
    Ok(p::Empty {})
}

pub(super) fn list_theme_tokens(
    rt: &Runtime,
    _: p::Empty,
) -> Result<p::ThemeTokenList, p::RpcError> {
    let tokens = main_theme(rt)
        .map(|theme| apply::token_list(&theme))
        .unwrap_or_default();
    Ok(p::ThemeTokenList { tokens })
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::SetStyleOverride(params) => {
            reply.send(set_style_override(rt, params).map(p::Reply::SetStyleOverride))
        }
        p::Call::ClearStyleOverrides(params) => {
            reply.send(clear_style_overrides(rt, params).map(p::Reply::ClearStyleOverrides))
        }
        p::Call::ListStyleOverrides(params) => {
            reply.send(list_style_overrides(rt, params).map(p::Reply::ListStyleOverrides))
        }
        p::Call::ForceState(params) => {
            reply.send(force_state(rt, params).map(p::Reply::ForceState))
        }
        p::Call::ListThemeTokens(params) => {
            reply.send(list_theme_tokens(rt, params).map(p::Reply::ListThemeTokens))
        }
        other => return Some((other, reply)),
    }
    None
}

/// The stored layers, owner and creation location of `id`.
fn style_source(
    rt: &Runtime,
    id: NodeId,
) -> Option<(Vec<StyleLayer>, Owner, Option<&'static Location<'static>>)> {
    let tree = ViewState::tree(rt);
    let node = tree.get(id)?;
    Some((node.style_layers(), node.owner, node.at))
}

/// Evaluate one style layer under its node's owner. The stored layers are
/// plain (`set_style_layers` fixed the recipe layers to their variants).
fn evaluate(owner: Owner, layer: &StyleLayer) -> Style {
    owner.run_handler(|| untrack(|| pure(|| layer.f.style(&crate::view::Variants::default()))))
}

/// The recipe name of a layer, called under the node's owner.
fn recipe_name(owner: Owner, name: &RecipeName) -> String {
    owner.run_handler(|| untrack(|| pure(|| name()))).to_owned()
}

fn source_layer(owner: Owner, style: &Style, layer: &StyleLayer) -> p::StyleLayerInfo {
    let (origin, name) = match &layer.origin {
        LayerOrigin::Base => (p::StyleOrigin::Base, None),
        LayerOrigin::BaseRecipe(recipe) => {
            (p::StyleOrigin::Recipe, Some(recipe_name(owner, recipe)))
        }
        LayerOrigin::Recipe(recipe) => (p::StyleOrigin::Recipe, Some(recipe_name(owner, recipe))),
        LayerOrigin::Caller => (p::StyleOrigin::Caller, None),
    };
    p::StyleLayerInfo {
        origin,
        name,
        source: layer.at.map(convert::location),
        properties: described(style),
    }
}

/// One layer per state delta, above the layers that declare them.
fn state_layers(style: &Style, forced: ForcedStates) -> Vec<p::StyleLayerInfo> {
    let mut deltas = describe_state_styles(style);
    let mut out = Vec::new();
    for (name, origin) in [
        ("pressed", p::StyleOrigin::State),
        ("hover", p::StyleOrigin::State),
        ("focus_visible", p::StyleOrigin::State),
        ("focus", p::StyleOrigin::State),
        ("disabled", p::StyleOrigin::Disabled),
        ("selected", p::StyleOrigin::Selected),
    ] {
        // The style method is `active` (`StyleState::name`); DevTools calls it `pressed`.
        let state_name = if name == "pressed" { "active" } else { name };
        let Some(index) = deltas.iter().position(|(state, _)| *state == state_name) else {
            continue;
        };
        let (_, items) = deltas.remove(index);
        let marked = forced_bit(name).is_some_and(|bit| forced.contains(bit));
        out.push(p::StyleLayerInfo {
            origin,
            name: Some(if marked {
                format!("{name} (forced)")
            } else {
                name.to_owned()
            }),
            source: None,
            properties: properties(
                items
                    .into_iter()
                    .filter(|(property, _)| style_property(property).is_some())
                    .map(|(property, value)| (property.to_owned(), value)),
            ),
        });
    }
    out
}

fn forced_bit(name: &str) -> Option<ForcedStates> {
    Some(match name {
        "hover" => ForcedStates::HOVER,
        "pressed" => ForcedStates::PRESSED,
        "focus" => ForcedStates::FOCUS,
        "focus_visible" => ForcedStates::FOCUS_VISIBLE,
        "disabled" => ForcedStates::DISABLED,
        "selected" => ForcedStates::SELECTED,
        _ => return None,
    })
}

fn properties(items: impl IntoIterator<Item = (String, String)>) -> Vec<p::StyleProperty> {
    items
        .into_iter()
        .map(|(name, value)| p::StyleProperty {
            name,
            value,
            overridden: false,
        })
        .collect()
}

/// The properties of `style` that the table names, each in its DevTools form.
fn described(style: &Style) -> Vec<p::StyleProperty> {
    properties(describe_style(style).into_iter().filter_map(|(name, _)| {
        let definition = style_property(name)?;
        Some((name.to_owned(), (definition.describe)(style)?))
    }))
}

/// Mark a property a higher layer also sets.
fn mark_overridden(layers: &mut [p::StyleLayerInfo]) {
    let mut seen = HashSet::new();
    for layer in layers.iter_mut() {
        for property in &mut layer.properties {
            property.overridden = seen.contains(&property.name);
        }
        for property in &layer.properties {
            seen.insert(property.name.clone());
        }
    }
}

/// The theme of the main window's root.
fn main_theme(rt: &Runtime) -> Option<kivact_style::Theme> {
    let owner = {
        let tree = ViewState::tree(rt);
        let root = tree.root(WindowId::MAIN)?;
        tree.get(root)?.owner
    };
    Some(kivact_style::internal::theme_of(owner)?.get_untracked())
}

/// Check one theme token override against the token's kind.
fn validate_token(
    rt: &Runtime,
    token: &str,
    property: &str,
    value: &str,
) -> Result<(), p::RpcError> {
    if property != token {
        return Err(invalid_params(format!(
            "a token override sets one token: property must be \"{token}\""
        )));
    }
    let Some(theme) = main_theme(rt) else {
        return Err(invalid_params(format!(
            "theme token \"{token}\" does not exist"
        )));
    };
    let tokens = apply::token_list(&theme);
    let Some(entry) = tokens.iter().find(|entry| entry.name == token) else {
        return Err(invalid_params(format!(
            "theme token \"{token}\" does not exist"
        )));
    };
    let accepted = match entry.kind {
        p::TokenKind::Color => {
            kivact_style::Color::css(value.trim()).is_ok() || is_color_token(&tokens, value.trim())
        }
        p::TokenKind::Length | p::TokenKind::Number => value.trim().parse::<f64>().is_ok(),
        p::TokenKind::Text => true,
    };
    if accepted {
        Ok(())
    } else {
        Err(invalid_params(format!(
            "theme token \"{token}\" takes {}; got \"{value}\"",
            kind_name(entry.kind)
        )))
    }
}

fn is_color_token(tokens: &[p::ThemeToken], value: &str) -> bool {
    let name = value.strip_prefix("colors.").unwrap_or(value);
    tokens.iter().any(|entry| {
        entry.kind == p::TokenKind::Color && entry.name.strip_prefix("colors.") == Some(name)
    })
}

fn kind_name(kind: p::TokenKind) -> &'static str {
    match kind {
        p::TokenKind::Color => "a CSS color or a color token name",
        p::TokenKind::Length | p::TokenKind::Number => "a number",
        p::TokenKind::Text => "text",
    }
}

fn invalid_params(message: impl Into<String>) -> p::RpcError {
    p::RpcError::new(p::code::INVALID_PARAMS, message)
}

#[cfg(test)]
mod tests {
    //! Role: Tests for the style override table.

    use super::store_override;
    use kivact_devtools_protocol as p;

    #[test]
    fn a_repeated_override_keeps_its_id_and_takes_the_new_value() {
        let scope = p::OverrideScope::Node { node: 7 };
        let mut all = Vec::new();
        let mut allocated = 0_u64;
        let first = store_override(
            &mut all,
            scope.clone(),
            "width".to_owned(),
            "120".to_owned(),
            || {
                allocated += 1;
                allocated
            },
        );
        let second = store_override(
            &mut all,
            scope,
            "width".to_owned(),
            "50%".to_owned(),
            || {
                allocated += 1;
                allocated
            },
        );
        assert_eq!(all.len(), 1);
        assert_eq!(second.id, first.id);
        assert_eq!(all[0].value, "50%");
        assert_eq!(allocated, 1);
    }
}
