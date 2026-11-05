//! Role: Display a parsed keyboard shortcut as a sequence of key caps.
use kivact::state::IntoValue;
use kivact::view::NodeBuilder;

use kivact::i18n::{message, message_with};
use kivact::state::Value;
use kivact::state::memo;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::style;
use kivact::view::Role;
use kivact::view::row;
use kivact::view::text;

fn key(token: &str) -> (Value<String>, Value<String>) {
    let (glyph, label) = match token.to_ascii_lowercase().as_str() {
        "mod" if cfg!(any(target_os = "macos", target_os = "ios")) => {
            ("⌘", message("soft.kbd.key.command", "Command"))
        }
        "mod" => {
            return (
                message("soft.kbd.key.control_short", "Ctrl"),
                message("soft.kbd.key.control", "Control"),
            );
        }
        "cmd" | "command" => ("⌘", message("soft.kbd.key.command", "Command")),
        "ctrl" | "control" => ("⌃", message("soft.kbd.key.control", "Control")),
        "alt" => ("⌥", message("soft.kbd.key.alt", "Alt")),
        "shift" => ("⇧", message("soft.kbd.key.shift", "Shift")),
        "enter" | "return" => ("↵", message("soft.kbd.key.enter", "Enter")),
        "backspace" => ("⌫", message("soft.kbd.key.backspace", "Backspace")),
        "tab" => ("⇥", message("soft.kbd.key.tab", "Tab")),
        "up" => ("↑", message("soft.kbd.key.up", "Up arrow")),
        "down" => ("↓", message("soft.kbd.key.down", "Down arrow")),
        "left" => ("←", message("soft.kbd.key.left", "Left arrow")),
        "right" => ("→", message("soft.kbd.key.right", "Right arrow")),
        "plus" => ("+", message("soft.kbd.key.plus", "Plus")),
        "escape" | "esc" => {
            return (
                message("soft.kbd.key.escape_short", "Esc"),
                message("soft.kbd.key.escape", "Escape"),
            );
        }
        _ => {
            let label = token.to_uppercase().into_value();
            return (label, label);
        }
    };
    (glyph.to_owned().into_value(), label)
}

/// Render a shortcut such as `mod+shift+p` as individual key caps.
#[track_caller]
pub fn kbd(shortcut: &str) -> impl kivact::view::Component {
    let (caps, labels): (Vec<_>, Vec<_>) = shortcut
        .split('+')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(key)
        .unzip();
    let name = labels
        .into_iter()
        .reduce(|shortcut, key| {
            message_with("soft.kbd.shortcut", "{shortcut} + {key}", move || {
                [("shortcut", shortcut.get()), ("key", key.get())]
            })
        })
        .unwrap_or_else(|| memo(String::new));
    row(caps
        .into_iter()
        .map(|glyph| {
            row(text(glyph)).style(|| {
                let t = kivact::style::use_tokens::<crate::Tokens>();
                style()
                    .height(20)
                    .min_width(20)
                    .padding_x(t.spacing.s1)
                    .radius(t.radius.inner)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .background(t.colors.neutral)
                    .border_bottom(2, t.colors.border_emphasized)
                    .color(t.colors.text_primary)
                    .font_size(12)
                    .line_height(20.0 / 12.0)
                    .weight(t.typography.weight_medium)
                    .text_wrap(false)
            })
        })
        .collect::<Vec<_>>())
    .with_node(move |node| node.role(Role::Image))
    .name(name)
    .style(style().align_items(Align::Center).gap(4).shrink(0))
}
