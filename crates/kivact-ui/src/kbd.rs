//! Role: Present literal and typed shortcuts with platform command-key glyphs.

pub(crate) mod labels;
pub(crate) mod look;

use crate::kbd::look::KbdTheme;
use crate::theme::KbdKeyState;
use kivact_core::custom::Caller;
use kivact_core::platform::capabilities;
use kivact_core::state::{IntoValue, Value};
use kivact_core::view::{Key, Modifiers, Node, Role, Variants, for_index, row, text};
use kivact_macros::component;

/// A shortcut written as text, such as `"Ctrl+S"`: each key is one key cap.
#[component]
pub fn kbd(keys: Value<String>, #[caller] caller: Caller) -> impl Element {
    let variants = caller.variants();
    let names = labels::KeyNames::new();
    let platform = capabilities();
    let caps = (move || {
        keys.get()
            .split('+')
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .map(|key| names.literal(key, platform.get().command_key))
            .collect::<Vec<_>>()
    })
    .into_value();
    render(row(()).role(Role::Image), caps, variants)
}

/// A chord of typed keys. `modifiers` are the chord's physical modifiers;
/// `meta` displays the platform command key from its capabilities.
#[component]
pub fn kbd_combo(
    keys: Value<Vec<Key>>,
    #[prop(default = Modifiers::default())] modifiers: Value<Modifiers>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let names = labels::KeyNames::new();
    let platform = capabilities();
    let caps = (move || {
        let command = platform.get().command_key;
        let modifiers = modifiers.get();
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
        caps.extend(keys.get().into_iter().map(|key| names.typed(key, command)));
        caps
    })
    .into_value();
    render(row(()).role(Role::Image), caps, variants)
}

fn render(root: Node, caps: Value<Vec<KbdKeyState>>, variants: Variants) -> Node {
    let separator = kivact_core::i18n::message("kbd.separator", " + ");
    crate::layer::with_base_recipe_style(
        root.name(move || {
            caps.get()
                .into_iter()
                .map(|cap| cap.label)
                .collect::<Vec<_>>()
                .join(&separator.get())
        }),
        |r: &KbdTheme| r.root_recipe(),
        &variants,
        || (),
    )
    .child(for_index(
        move || caps.get(),
        move |_, cap| {
            crate::layer::with_base_recipe_style(
                row(text(move || cap.get().glyph)),
                |r: &KbdTheme| r.key_recipe(),
                &variants,
                move || cap.get(),
            )
        },
    ))
}
