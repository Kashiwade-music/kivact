//! Role: Markdown of a review document and the locator expression of an item.

use crate::common::{BuildKind, InteractiveState};
use crate::emulation::{Emulation, PseudoLocale, ThemeEmulation};
use crate::node::CallFrame;
use crate::review::{ReviewDocument, ReviewItem, ReviewTarget};
use crate::style::OverrideScope;

/// The id of the main window (`WindowId::MAIN` is 1).
const MAIN_WINDOW: u64 = 1;

/// The Markdown of a review document.
pub fn to_markdown(document: &ReviewDocument) -> String {
    let name = &document.app.name;
    let kivact_version = &document.app.kivact_version;
    let build = build_name(document.app.build);
    let session = &document.session;
    let updated = &document.updated;
    let emulation = emulation_summary(&document.environment);
    let mut text = format!("# DevTools review: {name}\n\n");
    text.push_str(&format!("- Session: {session} / Updated: {updated}\n"));
    text.push_str(&format!("- Build: {build} / Kivact {kivact_version}\n"));
    text.push_str(&format!("- Emulation: {emulation}\n"));
    for item in &document.items {
        text.push('\n');
        text.push_str(&item_markdown(item));
    }
    text
}

/// The locator of an item as a `kivact::test` Rust expression, if it points at a named node.
pub fn locator(item: &ReviewItem) -> Option<String> {
    let ReviewTarget::Node {
        role,
        name,
        test_id,
        ..
    } = &item.target
    else {
        return None;
    };
    let query = match (test_id, role) {
        (Some(id), _) => format!("get_by_test_id({id:?})"),
        (None, Some(role)) => {
            format!(
                "get_by_role(Role::{role:?}, {:?})",
                name.as_deref().unwrap_or("")
            )
        }
        (None, None) => return None,
    };
    let screen = if item.window == MAIN_WINDOW {
        "screen.".to_owned()
    } else {
        format!("screen.window({:?}).", item.window_title)
    };
    Some(format!("{screen}{query}"))
}

fn build_name(build: BuildKind) -> &'static str {
    match build {
        BuildKind::Debug => "debug",
        BuildKind::Release => "release",
    }
}

fn emulation_summary(emulation: &Emulation) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(theme) = emulation.theme {
        parts.push(
            match theme {
                ThemeEmulation::Light => "theme light",
                ThemeEmulation::Dark => "theme dark",
            }
            .to_owned(),
        );
    }
    if let Some(reduced_motion) = emulation.reduced_motion {
        parts.push(format!("reduced motion {}", on_off(reduced_motion)));
    }
    if let Some(locale) = &emulation.locale {
        parts.push(format!("locale {locale}"));
    }
    match emulation.pseudo_locale {
        PseudoLocale::Off => {}
        PseudoLocale::Accented => parts.push("pseudo-locale accented".to_owned()),
        PseudoLocale::Long => parts.push("pseudo-locale long".to_owned()),
        PseudoLocale::Rtl => parts.push("pseudo-locale rtl".to_owned()),
    }
    if let Some(scale) = emulation.scale {
        parts.push(format!("scale {scale}"));
    }
    if let Some(touch) = emulation.touch {
        parts.push(format!("touch {}", on_off(touch)));
    }
    if parts.is_empty() {
        "none".to_owned()
    } else {
        parts.join(", ")
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

fn item_markdown(item: &ReviewItem) -> String {
    let mut text = String::new();
    text.push_str(&heading(item));
    text.push_str("\n\n");
    if let ReviewTarget::Node {
        source,
        call_chain,
        bounds,
        states,
        ..
    } = &item.target
    {
        if let Some(source) = source {
            let (file, line, column) = (&source.file, source.line, source.column);
            text.push_str(&format!("- Source: {file}:{line}:{column}\n"));
        }
        if !call_chain.is_empty() {
            text.push_str(&format!("- Call chain: {}\n", call_chain_text(call_chain)));
        }
        if let Some(query) = locator(item) {
            text.push_str(&format!("- Query: {query}\n"));
        }
        let states = states_text(states);
        let (x, y, width, height) = (bounds.x, bounds.y, bounds.width, bounds.height);
        text.push_str(&format!(
            "- State: {states} / Bounds: x {x}, y {y}, {width}×{height}\n"
        ));
    }
    if let Some(image) = &item.image {
        text.push_str(&format!("- Image: {image}\n"));
    }
    if !item.style_changes.is_empty() {
        text.push_str("- Style changes:\n");
        for change in &item.style_changes {
            let property = &change.property;
            let before = change.before.as_deref().unwrap_or("(unset)");
            let after = &change.after;
            let scope = scope_text(&change.scope);
            text.push_str(&format!("  - {property}: {before} → {after}  ({scope})\n"));
        }
    }
    text.push_str("- Comments:\n");
    for (index, comment) in item.comments.iter().enumerate() {
        let number = index + 1;
        let (body, created) = (&comment.text, &comment.created);
        text.push_str(&format!("  {number}. {body}  ({created})\n"));
    }
    text
}

fn heading(item: &ReviewItem) -> String {
    let id = &item.id;
    let window_title = &item.window_title;
    match &item.target {
        ReviewTarget::Node { role, name, .. } => {
            let role = match role {
                Some(role) => format!("{role:?}"),
                None => "element".to_owned(),
            };
            let name = match name {
                Some(name) => format!(" \"{name}\""),
                None => String::new(),
            };
            format!("## {id} {role}{name}  (window \"{window_title}\")")
        }
        ReviewTarget::Window {} => format!("## {id} window \"{window_title}\""),
    }
}

fn call_chain_text(chain: &[CallFrame]) -> String {
    let mut frames: Vec<&CallFrame> = chain.iter().filter(|frame| frame.app_code).collect();
    frames.extend(chain.iter().filter(|frame| !frame.app_code));
    frames
        .iter()
        .map(|frame| format!("{}:{}", frame.location.file, frame.location.line))
        .collect::<Vec<_>>()
        .join(" ← ")
}

fn scope_text(scope: &OverrideScope) -> String {
    match scope {
        OverrideScope::Node { node } => format!("node #{node}"),
        OverrideScope::CallSite { location } => {
            format!("call site {}:{}", location.file, location.line)
        }
        OverrideScope::Token { token } => format!("token {token}"),
    }
}

fn states_text(states: &[InteractiveState]) -> String {
    if states.is_empty() {
        return "none".to_owned();
    }
    states
        .iter()
        .map(|state| match state {
            InteractiveState::Hovered => "hovered",
            InteractiveState::Pressed => "pressed",
            InteractiveState::Focused => "focused",
            InteractiveState::FocusVisible => "focus_visible",
            InteractiveState::Disabled => "disabled",
            InteractiveState::Selected => "selected",
        })
        .collect::<Vec<_>>()
        .join(", ")
}
