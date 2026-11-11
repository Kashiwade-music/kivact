//! Role: Present literal and typed shortcuts with platform command-key glyphs.
use crate::variants::{Variant, Variants};
use kivact_core::platform::capabilities;
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::view::{
    Element, IntoView, Key, Modifiers, Node, Role, View, for_index, row, text,
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
    pub fn variant(mut self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl Element for Kbd {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Kbd {
    fn into_view(self) -> View {
        let names = super::kbd_labels::KeyNames::new();
        let platform = capabilities();
        let caps = (move || {
            self.keys
                .get()
                .split('+')
                .map(str::trim)
                .filter(|key| !key.is_empty())
                .map(|key| names.literal(key, platform.get().command_key))
                .collect::<Vec<_>>()
        })
        .into_value();
        render(self.root, caps, self.variants)
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
    pub fn variant(mut self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl Element for KbdCombo {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for KbdCombo {
    fn into_view(self) -> View {
        let names = super::kbd_labels::KeyNames::new();
        let platform = capabilities();
        let caps = (move || {
            let command = platform.get().command_key;
            let modifiers = self.modifiers.get();
            let mut caps = Vec::new();
            for (enabled, key) in [
                (modifiers.ctrl, "ctrl"),
                (modifiers.meta, "mod"),
                (modifiers.alt, "alt"),
                (modifiers.shift, "shift"),
            ] {
                if enabled {
                    caps.push(names.literal(key, command));
                }
            }
            caps.extend(
                self.keys
                    .get()
                    .into_iter()
                    .map(|key| names.typed(key, command)),
            );
            caps
        })
        .into_value();
        render(self.root, caps, self.variants)
    }
}
fn render(root: Node, caps: Value<Vec<crate::KbdKeyState>>, variants: Variants) -> View {
    let separator = kivact_core::i18n::message("kbd.separator", " + ");
    crate::layer::with_base_recipe_style(
        root.name(move || {
            caps.get()
                .into_iter()
                .map(|cap| cap.label)
                .collect::<Vec<_>>()
                .join(&separator.get())
        }),
        |r| r.kbd_recipe(),
        &variants,
        || (),
    )
    .child(for_index(
        move || caps.get(),
        move |_, cap| {
            crate::layer::with_base_recipe_style(
                row(text(move || cap.get().glyph)),
                |r| r.kbd_key_recipe(),
                &variants,
                move || cap.get(),
            )
        },
    ))
    .into_view()
}
