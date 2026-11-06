//! Role: Keyboard routing contracts across focus changes.

use crate::support::mount;
use kivact::prelude::*;
use kivact::view::{Handled, KeyCode, Modifiers};

#[test]
fn key_up_goes_to_the_node_that_received_key_down() {
    let (screen, (down, up_a, up_b)) = mount(|| {
        let down = signal(0);
        let up_a = signal(0);
        let up_b = signal(0);
        let a = node()
            .role(Role::Button)
            .focusable(true)
            .test_id("a")
            .on_key(move |_| {
                down.set(down.get_untracked() + 1);
                Handled::Yes
            })
            .on_key_up(move |_| {
                up_a.set(up_a.get_untracked() + 1);
                Handled::Yes
            })
            .style(style().size(80, 40));
        let b = node()
            .role(Role::Button)
            .focusable(true)
            .test_id("b")
            .on_key_up(move |_| {
                up_b.set(up_b.get_untracked() + 1);
                Handled::Yes
            })
            .style(style().size(80, 40));
        ((down, up_a, up_b), column((a, b)))
    });
    screen.get_by_test_id("a").focus();
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    assert_eq!(down.get_untracked(), 1);
    screen.get_by_test_id("b").focus();
    assert!(screen.get_by_test_id("b").is_focused());
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    assert_eq!(up_a.get_untracked(), 1);
    assert_eq!(up_b.get_untracked(), 0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
