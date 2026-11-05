//! Role: Present literal and typed shortcuts with platform command-key glyphs.
use crate::variants::{Variant, Variants};
use kivact_core::platform::{CommandKey, capabilities};
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::style::{Align, style};
use kivact_core::view::{
    IntoView, Key, Modifiers, Node, NodeBuilder, Role, View, for_index, row, text,
};
pub struct Kbd {
    root: Node,
    keys: Value<String>,
    variants: Variants,
}
pub fn kbd(keys: impl IntoText) -> Kbd {
    Kbd {
        root: row(()).role(Role::Image),
        keys: keys.into_text_value(),
        variants: Variants::default(),
    }
}
impl Kbd {
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for Kbd {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
fn token(key: &str, command: CommandKey) -> String {
    match key.trim().to_ascii_lowercase().as_str() {
        "mod" => if command == CommandKey::Command {
            "\u{2318}"
        } else {
            "Ctrl"
        }
        .to_owned(),
        "cmd" | "command" => "\u{2318}".into(),
        "ctrl" | "control" => "Ctrl".into(),
        "shift" => "Shift".into(),
        "alt" => "Alt".into(),
        _ => key.trim().to_owned(),
    }
}
impl IntoView for Kbd {
    fn into_view(self) -> View {
        let keys = self.keys;
        let platform = capabilities();
        let label = move || {
            keys.get()
                .split('+')
                .map(|key| token(key, platform.get().command_key))
                .collect::<Vec<_>>()
                .join(" + ")
        };
        crate::layer::with_base_recipe_style(
            self.root.name(label),
            |r| r.kbd_recipe(),
            &self.variants,
            || (),
        )
        .child(text(label))
        .into_view()
    }
}
pub struct KbdCombo {
    root: Node,
    keys: Value<Vec<Key>>,
    modifiers: Value<Modifiers>,
    variants: Variants,
}
/// Render typed keys. Use `.modifiers(..)` for a chord's physical modifiers;
/// `meta` displays the platform command key from its capabilities.
pub fn kbd_combo(keys: impl IntoValue<Vec<Key>>) -> KbdCombo {
    KbdCombo {
        root: row(()).role(Role::Image),
        keys: keys.into_value(),
        modifiers: (move || Modifiers::default()).into_value(),
        variants: Variants::default(),
    }
}
impl KbdCombo {
    pub fn modifiers(mut self, modifiers: impl IntoValue<Modifiers>) -> Self {
        self.modifiers = modifiers.into_value();
        self
    }
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for KbdCombo {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
fn key_name(key: Key) -> String {
    match key {
        Key::Char(c) => c.to_string(),
        Key::Enter => "Enter".into(),
        Key::Tab => "Tab".into(),
        Key::Backspace => "Backspace".into(),
        Key::ArrowUp => "Up".into(),
        Key::ArrowDown => "Down".into(),
        Key::ArrowLeft => "Left".into(),
        Key::ArrowRight => "Right".into(),
        Key::F(n) => format!("F{n}"),
        other => format!("{other:?}"),
    }
}
impl IntoView for KbdCombo {
    fn into_view(self) -> View {
        let keys = self.keys;
        let modifiers = self.modifiers;
        let platform = capabilities();
        let labels = (move || {
            let m = modifiers.get();
            let mut labels = Vec::new();
            if m.ctrl {
                labels.push("Ctrl".to_owned());
            }
            if m.meta {
                labels.push(token("mod", platform.get().command_key));
            }
            if m.alt {
                labels.push("Alt".into());
            }
            if m.shift {
                labels.push("Shift".into());
            }
            labels.extend(keys.get().into_iter().map(key_name));
            labels
        })
        .into_value();
        let variants = self.variants;
        self.root
            .name(move || labels.get().join(" + "))
            .base_style(style().align_items(Align::Center).gap(4))
            .child(for_index(
                move || labels.get(),
                move |_, label| {
                    crate::layer::with_base_recipe_style(
                        row(text(label)),
                        |r| r.kbd_recipe(),
                        &variants,
                        || (),
                    )
                },
            ))
            .into_view()
    }
}
