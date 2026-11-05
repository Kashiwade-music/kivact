//! Role: Inputs, the tag editor and the dropzone commit and report publicly.

use kivact::geometry::Point;
use kivact::prelude::*;
use kivact::view::A11yAction;
use kivact_ui_pop as ui;

#[test]
fn inputs_and_tag_editor_commit_native_edits() {
    let (screen, (draft, tags, memo)) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_pop::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let draft = signal(String::new());
            let tags = signal(vec!["one".into()]);
            let memo = signal(String::new());
            (
                (draft, tags, memo),
                column((
                    kivact::ui::text_input("Name", draft).placeholder("Enter name"),
                    ui::resizable_text_area(
                        kivact::ui::text_area("Resizable note", memo),
                        80.0,
                        72.0,
                    ),
                    kivact::ui::text_area("Fixed note", kivact::state::signal(String::new())),
                    kivact::ui::tag_input("Add tag", tags)
                        .on_add(move |tag| tags.update(|items| items.push(tag)))
                        .on_remove(move |index| {
                            tags.update(|items| {
                                items.remove(index);
                            })
                        }),
                ))
                .style(style().width(360).gap(8)),
            )
        },
    );
    screen
        .get_by_role(Role::TextBox, "Name")
        .type_text("Minato");
    assert_eq!(draft.get_untracked(), "Minato");
    let note = screen.get_by_role(Role::TextBox, "Resizable note");
    note.type_text("Keep this draft");
    let initial = note.rect().expect("note laid out");
    assert_eq!(initial.height, 80.0);
    let resize = |delta: f64| {
        let grip = screen.get_by_role(Role::Splitter, "Resize text area");
        grip.hover();
        let rect = grip.rect().expect("resize grip laid out");
        let x = rect.x + rect.width / 2.0;
        let y = rect.y + rect.height / 2.0;
        screen.mouse_move(x, y);
        screen.mouse_down(kivact::view::MouseButton::Primary);
        screen.mouse_move(x, y + delta / 2.0);
        screen.mouse_move(x, y + delta);
        screen.mouse_up(kivact::view::MouseButton::Primary);
    };
    resize(48.0);
    assert_eq!(note.rect().unwrap().height, 128.0);
    assert_eq!(note.rect().unwrap().width, initial.width);
    assert_eq!(memo.get_untracked(), "Keep this draft");
    assert!(note.is_focused());
    screen.type_text(" after resizing");
    assert_eq!(memo.get_untracked(), "Keep this draft after resizing");
    resize(-200.0);
    assert_eq!(note.rect().unwrap().height, 72.0);
    assert_eq!(note.rect().unwrap().width, initial.width);
    assert!(note.is_focused());
    screen.type_text(".");
    assert_eq!(memo.get_untracked(), "Keep this draft after resizing.");
    let grip = screen.get_by_role(Role::Splitter, "Resize text area");
    grip.press_key(Key::ArrowDown);
    assert_eq!(note.rect().unwrap().height, 80.0);
    grip.press_key(Key::ArrowDown);
    assert_eq!(note.rect().unwrap().height, 88.0);
    grip.press_key(Key::ArrowUp);
    assert_eq!(note.rect().unwrap().height, 80.0);
    grip.press_key(Key::Home);
    assert_eq!(note.rect().unwrap().height, 72.0);
    grip.a11y(A11yAction::Increment);
    assert_eq!(note.rect().unwrap().height, 80.0);
    grip.a11y_set_value("-100");
    assert_eq!(note.rect().unwrap().height, 72.0);
    grip.a11y_set_value("NaN");
    assert_eq!(note.rect().unwrap().height, 72.0);
    assert_eq!(
        screen
            .get_by_role(Role::TextBox, "Fixed note")
            .rect()
            .unwrap()
            .height,
        80.0
    );
    let tag = screen.get_by_role(Role::TextBox, "Add tag");
    tag.type_text("two");
    tag.press_key(Key::Enter);
    assert_eq!(tags.get_untracked(), vec!["one", "two"]);
    tag.press_key(Key::Backspace);
    assert_eq!(tags.get_untracked(), vec!["one"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn dropzone_reports_public_file_events_and_platform_picker_results() {
    let (screen, paths) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_pop::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let paths = signal(Vec::<std::path::PathBuf>::new());
            (
                paths,
                kivact::ui::dropzone("Upload")
                    .multiple()
                    .pick_on_press()
                    .icon(icon("cloud-upload").style(style().size(24, 24)))
                    .hint("Select reports")
                    .filters(vec![("Reports".into(), vec!["pdf".into()])])
                    .on_files(move |files| {
                        paths.set(files.into_iter().map(|file| file.path).collect())
                    }),
            )
        },
    );
    screen.get_by_role(Role::Text, "Select reports");
    let dropzone = screen.get_by_role(Role::Button, "Upload");
    let rect = dropzone
        .rect()
        .expect("invariant: drop target was laid out");
    let point = Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.file_drag(
        kivact::view::FilePhase::Enter,
        vec!["report.pdf".into()],
        point,
    );
    screen.file_drag(
        kivact::view::FilePhase::Drop,
        vec!["report.pdf".into()],
        point,
    );
    assert_eq!(
        paths.get_untracked(),
        vec![std::path::PathBuf::from("report.pdf")]
    );
    screen.respond_files_dialog(Some(vec!["selected.png".into(), "調査.pdf".into()]));
    dropzone.press_key(Key::Enter);
    assert_eq!(
        paths.get_untracked(),
        vec![
            std::path::PathBuf::from("selected.png"),
            std::path::PathBuf::from("調査.pdf")
        ]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
