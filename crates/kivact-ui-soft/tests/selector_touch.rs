//! Role: Verify touch and keyboard Selector presentations over one binding.

use kivact::control::NodeBuilder;
use kivact::signal;
use kivact::style::{ThemeMode, style};
use kivact::test::{RenderOptions, render_with_handle};
use kivact::ui::ListItem;
use kivact::view::{Key, PointerKind, PopupKind, Role};
use kivact_ui_soft::{SelectorPresentation, selector};
use std::time::Duration;

#[test]
fn compact_touch_uses_a_latched_dialog_and_keyboard_uses_the_popup() {
    let (screen, selected) = render_with_handle(
        RenderOptions {
            theme: kivact_ui_soft::theme(ThemeMode::Light),
            width: 500.0,
            height: 700.0,
            ..RenderOptions::from_env()
        },
        || {
            let selected = signal(Some("a"));
            let field = selector(
                "Team",
                vec![
                    ListItem::new("a", "Alpha"),
                    ListItem::new("skip", "Unavailable").disabled(true),
                    ListItem::new("b", "Beta"),
                ],
                selected,
            )
            .presentation(SelectorPresentation::Adaptive)
            .style(style().width(280));
            (selected, field)
        },
    );
    let combo = || screen.get_by_role(Role::ComboBox, "Team");
    assert_eq!(combo().popup_kind(), Some(PopupKind::ListBox));
    let rect = combo().rect().unwrap();
    let x = rect.x + rect.width / 2.0;
    let y = rect.y + rect.height / 2.0;
    let touch = screen.contact_down(PointerKind::Touch, x, y);
    assert!(screen.query_by_role(Role::Dialog, "Team").is_none());
    screen.contact_up(touch);
    screen.advance(Duration::from_millis(350));
    screen.get_by_role(Role::Dialog, "Team");
    assert_eq!(combo().popup_kind(), Some(PopupKind::Dialog));
    assert!(screen.get_by_role(Role::Option, "Alpha").is_focused());
    assert_eq!(selected.get(), Some("a"));

    screen.resize(900.0, 700.0);
    screen.get_by_role(Role::Dialog, "Team");
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Option, "Beta").is_focused());
    assert_eq!(selected.get(), Some("a"));
    screen.press_key(Key::Enter);
    assert_eq!(selected.get(), Some("b"));
    assert!(screen.query_by_role(Role::Dialog, "Team").is_none());
    assert!(combo().is_focused());

    screen.resize(500.0, 700.0);
    combo().press_key(Key::ArrowDown);
    screen.get_by_role(Role::ListBox, "Team");
    assert!(screen.query_by_role(Role::Dialog, "Team").is_none());
    assert_eq!(combo().popup_kind(), Some(PopupKind::ListBox));
    assert!(combo().is_focused());
    assert_eq!(selected.get(), Some("b"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
