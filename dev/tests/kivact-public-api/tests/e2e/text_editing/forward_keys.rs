//! Role: Native control-key opt-in preserves editing and IME defaults.

use super::*;

#[test]
fn explicit_forward_keys_preserve_native_defaults_modifiers_and_composition() {
    let (screen, (value, keys)) = crate::support::mount(|| {
        let value = signal("abcd".to_owned());
        let keys = signal(Vec::<Key>::new());
        (
            (value, keys),
            column((
                kivact::ui::text_input("Native defaults", signal("abcd".to_owned())),
                kivact::ui::text_input("Forwarded", value)
                    .forward_keys(vec![
                        Key::Tab,
                        Key::ArrowLeft,
                        Key::ArrowRight,
                        Key::Char('a'),
                        Key::Space,
                    ])
                    .on_key(move |event: KeyEvent| {
                        keys.update(|keys| keys.push(event.key));
                        kivact::view::Handled::No
                    }),
                kivact::ui::button("After"),
            )),
        )
    });
    let native = screen.get_by_role(Role::TextBox, "Native defaults");
    native.press_key(Key::End);
    native.press_key(Key::ArrowLeft);
    assert_eq!(native.text_selection(), Some((3, 3)));
    screen.press_tab(false);
    let field = screen.get_by_role(Role::TextBox, "Forwarded");
    assert!(field.is_focused(), "default Tab still moves focus");
    field.press_key(Key::End);
    keys.set(Vec::new());
    for key in [Key::ArrowLeft, Key::ArrowRight, Key::Tab] {
        screen.press_key(key);
    }
    screen.press_tab(true);
    assert_eq!(
        keys.get_untracked(),
        [Key::ArrowLeft, Key::ArrowRight, Key::Tab, Key::Tab]
    );
    assert!(field.is_focused());
    assert_eq!(
        field.text_selection(),
        Some((4, 4)),
        "Handled::No does not resume native editing"
    );
    keys.set(Vec::new());
    screen.press_key_with(Key::ArrowLeft, shift());
    assert_eq!(
        field.text_selection(),
        Some((4, 3)),
        "Shift+Left still extends selection"
    );
    screen.press_key_with(Key::ArrowLeft, ctrl());
    assert!(
        keys.get_untracked().is_empty(),
        "Ctrl keeps the native editing path"
    );
    field.compose("仮");
    for key in [Key::Tab, Key::ArrowLeft, Key::ArrowRight] {
        screen.press_key(key);
    }
    screen.press_tab(true);
    assert!(field.is_focused());
    assert!(
        keys.get_untracked().is_empty(),
        "the IME consumes opted-in keys first"
    );
    assert_eq!(value.get_untracked(), "abcd");
    field.compose("");
    screen.type_text("a ");
    assert!(
        value.get_untracked().contains("a "),
        "characters and Space cannot be forwarded"
    );
    assert!(keys.get_untracked().is_empty());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn forwarded_control_keys_follow_reactive_lists_in_both_builders() {
    for standard in [false, true] {
        let (screen, (forwarded, received)) = crate::support::mount(move || {
            let forwarded = signal(Vec::<Key>::new());
            let received = signal(Vec::<Key>::new());
            let value = signal("abcd".to_owned());
            let observe = move |event: KeyEvent| {
                received.update(|keys| keys.push(event.key));
                kivact::view::Handled::No
            };
            let field = if standard {
                kivact::ui::text_input("Reactive keys", value)
                    .forward_keys(forwarded)
                    .on_key(observe)
                    .into_view()
            } else {
                node()
                    .name("Reactive keys")
                    .style(style().size(240, 40))
                    .part(use_text_edit(value).forward_keys(forwarded).control())
                    .on_key(observe)
                    .into_view()
            };
            (
                (forwarded, received),
                column((field, kivact::ui::button("After"))),
            )
        });
        let field = screen.get_by_role(Role::TextBox, "Reactive keys");
        field.press_key(Key::End);
        received.set(Vec::new());
        screen.press_key(Key::ArrowLeft);
        assert_eq!(field.text_selection(), Some((3, 3)));
        assert!(received.get_untracked().is_empty());

        forwarded.set(vec![Key::ArrowLeft]);
        screen.run_until_idle();
        screen.press_key(Key::ArrowLeft);
        assert_eq!(field.text_selection(), Some((3, 3)));
        assert_eq!(received.get_untracked(), [Key::ArrowLeft]);

        forwarded.set(vec![Key::ArrowRight]);
        received.set(Vec::new());
        screen.run_until_idle();
        screen.press_key(Key::ArrowLeft);
        screen.press_key(Key::ArrowRight);
        assert_eq!(field.text_selection(), Some((2, 2)));
        assert_eq!(received.get_untracked(), [Key::ArrowRight]);

        forwarded.set(Vec::new());
        screen.run_until_idle();
        screen.press_tab(false);
        assert!(screen.get_by_role(Role::Button, "After").is_focused());
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn backspace_and_delete_reach_the_key_handlers_only_from_an_empty_field() {
    let (screen, (value, keys)) = crate::support::mount(|| {
        let value = signal("ab".to_owned());
        let keys = signal(Vec::<Key>::new());
        (
            (value, keys),
            kivact::ui::text_input("Tags", value).on_key(move |event: KeyEvent| {
                keys.update(|keys| keys.push(event.key));
                kivact::view::Handled::Yes
            }),
        )
    });
    let field = screen.get_by_role(Role::TextBox, "Tags");
    field.focus();
    field.press_key(Key::End);
    screen.press_key(Key::Backspace);
    screen.press_key(Key::ArrowLeft);
    keys.set(Vec::new());
    screen.press_key(Key::Delete);
    assert_eq!(value.get_untracked(), "", "with text they edit natively");
    assert!(
        keys.get_untracked().is_empty(),
        "{:?}",
        keys.get_untracked()
    );
    screen.press_key(Key::Backspace);
    screen.press_key(Key::Delete);
    // With nothing to delete native editing leaves them to the handlers,
    // without `forward_keys` (a multi-value input removes its last chip).
    assert_eq!(keys.get_untracked(), [Key::Backspace, Key::Delete]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
