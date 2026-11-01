//! Role: Real-window contracts for consistent kit inputs and presentation defaults.
use super::*;
use kivact::primitive::NodeExt;

#[test]
fn pagination_uses_zero_based_bounds_and_reactive_counts() {
    let (screen, (page, count)) = kivact::test::render_with_handle(crate::options(true), || {
        let page = signal(999usize);
        let count = signal(5usize);
        ((page, count), g::pagination(page, count))
    });
    assert_eq!(page.get_untracked(), 999);
    assert!(screen.get_by_role(Role::Button, "5").is_selected());
    assert!(screen.get_by_role(Role::Button, "Next").is_disabled());
    screen.get_by_role(Role::Button, "Previous").click();
    assert_eq!(page.get_untracked(), 3);
    count.set(2);
    screen.run_until_idle();
    assert_eq!(page.get_untracked(), 3);
    assert!(screen.get_by_role(Role::Button, "2").is_selected());
    screen.get_by_role(Role::Button, "Previous").click();
    assert_eq!(page.get_untracked(), 0);
    count.set(0);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Button, "1").is_none());
    assert!(screen.get_by_role(Role::Button, "Previous").is_disabled());
    assert!(screen.get_by_role(Role::Button, "Next").is_disabled());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn reactive_breadcrumbs_avatars_gauge_and_skeleton_follow_inputs() {
    let (screen, (items, avatars, total, rows)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let items = signal(vec!["Home".to_owned(), "Before".to_owned()]);
            let avatars = signal(vec![("A".to_owned(), rgb(0x123456))]);
            let total = signal(3usize);
            let rows = signal(1usize);
            (
                (items, avatars, total, rows),
                column((
                    g::breadcrumb(items),
                    g::avatar_stack(avatars),
                    g::gauge("Capacity", 2usize).total(total),
                    g::skeleton().rows(rows).test_id("skeleton"),
                    g::avatar("Default"),
                    g::dropzone("Upload"),
                ))
                .style(style().width(360).padding(16).gap(12)),
            )
        });
    assert_eq!(
        screen.get_by_role(Role::Meter, "Capacity").value_range(),
        (Some(0.), Some(3.), Some(2.))
    );
    let height = screen.get_by_test_id("skeleton").rect().unwrap().height;
    items.set(vec!["Home".to_owned(), "After".to_owned()]);
    avatars.set(vec![
        ("B".to_owned(), rgb(0x654321)),
        ("C".to_owned(), rgb(0x123456)),
    ]);
    total.set(5);
    rows.set(3);
    screen.run_until_idle();
    assert!(screen.query_by_text("Before").is_none());
    screen.get_by_text("After");
    assert!(screen.query_by_text("A").is_none());
    screen.get_by_text("B");
    screen.get_by_text("C");
    assert_eq!(
        screen.get_by_role(Role::Meter, "Capacity").value_range().1,
        Some(5.)
    );
    assert!(screen.get_by_test_id("skeleton").rect().unwrap().height > height);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn standard_field_keeps_visible_name_hint_and_error_under_kit_theme() {
    let (screen, error) = kivact::test::render_with_handle(crate::options(true), || {
        let error = signal(None::<String>);
        (
            error,
            kivact::ui::field(
                "Visible name",
                kivact::ui::text_input("Old name", signal(String::new())),
            )
            .hint("Helpful text")
            .error(error),
        )
    });
    let input = || screen.get_by_role(Role::TextBox, "Visible name");
    assert!(!input().invalid());
    assert_eq!(
        input()
            .described_by()
            .iter()
            .map(|e| e.text())
            .collect::<Vec<_>>(),
        ["Helpful text"]
    );
    error.set(Some("Required".to_owned()));
    screen.run_until_idle();
    assert!(input().invalid());
    assert_eq!(
        input()
            .described_by()
            .iter()
            .map(|e| e.text())
            .collect::<Vec<_>>(),
        ["Helpful text", "Required"]
    );
    screen.get_by_role(Role::Alert, "Required");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn segmented_default_slot_preserves_selection_disabled_focus_and_caller_width() {
    let (screen, (selected, disabled)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let selected = signal(Some("a"));
            let disabled = signal(false);
            (
                (selected, disabled),
                kivact::ui::radio_group(
                    "Mode",
                    vec![
                        ListItem::new("a", "Alpha").icon("check"),
                        ListItem::new("b", "Beta").disabled(true),
                    ],
                    selected,
                )
                .orientation(Orientation::Horizontal)
                .variant(g::Segmented)
                .disabled(disabled)
                .test_id("segments")
                .style(style().width(420)),
            )
        });
    assert_eq!(
        screen.get_by_test_id("segments").rect().unwrap().width,
        420.
    );
    let alpha = screen.get_by_role(Role::RadioButton, "Alpha");
    assert_eq!(alpha.is_checked(), Some(true));
    assert!(screen.get_by_role(Role::RadioButton, "Beta").is_disabled());
    let before = screen.screenshot().hash();
    alpha.press_key(Key::Tab);
    alpha.focus();
    assert!(alpha.is_focused());
    assert_ne!(before, screen.screenshot().hash());
    disabled.set(true);
    screen.run_until_idle();
    alpha.force_click();
    assert_eq!(selected.get_untracked(), Some("a"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn game_alert_tones_and_builder_defaults_mount_in_the_theme() {
    let mut hashes = std::collections::HashSet::new();
    for tone in [
        Tone::Neutral,
        Tone::Info,
        Tone::Success,
        Tone::Warning,
        Tone::Danger,
        Tone::Accent,
    ] {
        let (screen, ()) = kivact::test::render_with_handle(crate::options(true), move || {
            (
                (),
                column((
                    g::alert("Notice").description("Details").tone(tone),
                    g::kpi("Default", 12.),
                    g::gauge("Gauge", 25usize),
                    g::avatar("A"),
                    g::empty_state("Empty"),
                ))
                .style(style().width(400).padding(12)),
            )
        });
        screen.get_by_text("Notice");
        assert_eq!(
            screen.get_by_role(Role::Meter, "Gauge").value_range().1,
            Some(100.)
        );
        hashes.insert(screen.screenshot().hash());
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
    assert_eq!(hashes.len(), 6);
}

#[test]
fn caller_semantics_win_and_gauge_total_keeps_normal_call_order() {
    let (screen, total) = kivact::test::render_with_handle(crate::options(true), || {
        let total = signal(10usize);
        (
            total,
            column((
                g::gauge("Default", 2usize)
                    .total(total)
                    .name("Caller gauge")
                    .role(Role::Group)
                    .value_max(42.),
                g::gauge("Reactive", 2usize).value_max(42.).total(total),
                g::skeleton().name("Caller skeleton").role(Role::Group),
                g::empty_state("Empty")
                    .name("Caller empty")
                    .role(Role::Status),
                g::alert("Notice")
                    .tone(Tone::Danger)
                    .name("Caller alert")
                    .role(Role::Group),
            ))
            .style(style().width(400)),
        )
    });
    assert_eq!(
        screen
            .get_by_role(Role::Group, "Caller gauge")
            .value_range()
            .1,
        Some(42.)
    );
    screen.get_by_role(Role::Group, "Caller skeleton");
    screen.get_by_role(Role::Status, "Caller empty");
    screen.get_by_role(Role::Group, "Caller alert");
    total.set(15);
    screen.run_until_idle();
    assert_eq!(
        screen.get_by_role(Role::Meter, "Reactive").value_range().1,
        Some(15.)
    );
    assert_eq!(
        screen
            .get_by_role(Role::Group, "Caller gauge")
            .value_range()
            .1,
        Some(42.)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
