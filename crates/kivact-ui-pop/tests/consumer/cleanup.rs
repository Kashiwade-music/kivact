//! Role: Real-window checks for reactive kit presentation and mount-once slots.
use super::*;
use kivact::primitive::NodeExt;

#[test]
fn common_builders_follow_reactive_inputs_and_mount_content_once() {
    let (screen, (labels, notes, points, rows, icon, mounted)) =
        kivact::test::render_with_handle(crate::options(), || {
            let labels = signal(vec!["One".to_owned(), "Two".to_owned()]);
            let notes = signal(vec!["Before".to_owned()]);
            let points = signal(vec![0f32, 1., 2.]);
            let rows = signal(1usize);
            let icon = signal(kivact::style::Icon::named("user"));
            let mounted = signal(0usize);
            let empty = ui::empty_state("Empty")
                .description("No entries")
                .content(move || {
                    mounted.update(|n| *n += 1);
                    text("Action content")
                });
            assert_eq!(mounted.get_untracked(), 0);
            (
                (labels, notes, points, rows, icon, mounted),
                column((
                    ui::stepper(labels, 0usize).notes(notes),
                    ui::stat_card("Count", "12").icon(icon).points(points),
                    ui::list_item("Default row"),
                    ui::skeleton().rows(rows).test_id("skeleton"),
                    empty,
                ))
                .style(style().width(400).padding(16).gap(12)),
            )
        });
    assert_eq!(mounted.get_untracked(), 1);
    screen.get_by_text("Action content");
    let height = screen.get_by_test_id("skeleton").rect().unwrap().height;
    let pixels = screen.screenshot().hash();
    labels.set(vec![
        "Alpha".to_owned(),
        "Beta".to_owned(),
        "Gamma".to_owned(),
    ]);
    notes.set(vec!["After".to_owned(), "Ready".to_owned()]);
    points.set(vec![2., 0., 2.]);
    rows.set(3);
    icon.set(kivact::style::Icon::named("mail"));
    screen.run_until_idle();
    screen.get_by_text("Gamma");
    screen.get_by_text("After");
    assert!(screen.query_by_text("Before").is_none());
    assert!(screen.get_by_test_id("skeleton").rect().unwrap().height > height);
    assert_ne!(pixels, screen.screenshot().hash());
    assert_eq!(mounted.get_untracked(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn alert_semantic_tones_have_distinct_themed_appearance_and_defaults_mount() {
    let mut hashes = std::collections::HashSet::new();
    for tone in [
        kivact::ui::Tone::Neutral,
        kivact::ui::Tone::Info,
        kivact::ui::Tone::Success,
        kivact::ui::Tone::Warning,
        kivact::ui::Tone::Danger,
        kivact::ui::Tone::Accent,
    ] {
        let (screen, ()) = kivact::test::render_with_handle(crate::options(), move || {
            (
                (),
                column((
                    ui::alert("Notice").description("Details").tone(tone),
                    ui::stat_card("Default", "0"),
                    ui::thread("A", "Thread"),
                    ui::message("Message"),
                    ui::timeline_item("Event"),
                    ui::agenda_item("Now", "Agenda"),
                ))
                .style(style().width(400).padding(12)),
            )
        });
        screen.get_by_text("Notice");
        hashes.insert(screen.screenshot().hash());
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
    assert_eq!(hashes.len(), 6);
}

#[test]
fn standard_field_keeps_visible_name_hint_and_error_under_kit_theme() {
    let (screen, error) = kivact::test::render_with_handle(crate::options(), || {
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
fn caller_padding_and_semantics_win_over_builder_defaults() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            column((
                ui::stat_card("Default padding", "1").test_id("default"),
                ui::stat_card("Caller padding", "2")
                    .test_id("caller")
                    .style(style().padding(7)),
                ui::alert("Notice").role(Role::Status).name("Caller alert"),
            ))
            .style(style().width(400)),
        )
    });
    let inset = |id: &str, label: &str| {
        screen.get_by_text(label).rect().unwrap().x - screen.get_by_test_id(id).rect().unwrap().x
    };
    assert_eq!(inset("default", "Default padding"), 21.);
    assert_eq!(inset("caller", "Caller padding"), 28.);
    screen.get_by_role(Role::Status, "Caller alert");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
