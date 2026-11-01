//! Role: Tests for the advanced Creative controls through the public API.

use super::*;
use kivact::view::Handled;

#[test]
fn arrangement_loop_automation_track_flags_and_clip_tools_update_public_models() {
    let (screen, (clips, tracks, range, tool)) =
        kivact::test::render_with_handle(crate::options(), || {
            let clips = signal(vec![
                ui::Clip::new(1, 0, 0.0, 16.0, "Clip").color(rgb(0x34d399)),
            ]);
            let tracks = signal(vec![
                ui::TimelineTrack::new(0, "Vox", ui::TrackKind::Audio, rgb(0x34d399)).automation(
                    vec![
                        ui::AutomationPoint::new(1, 2.0, 0.25),
                        ui::AutomationPoint::new(2, 16.0, 0.75),
                    ],
                ),
            ]);
            let range = signal((0.0, 16.0));
            let tool = signal(ui::TimelineTool::Select);
            (
                (clips, tracks, range, tool),
                app_arrangement(
                    clips,
                    tracks,
                    ui::ArrangementState {
                        loop_range: range.into(),
                        tool: tool.into(),
                        ..ui::ArrangementState::new()
                    },
                )
                .test_id("arrangement"),
            )
        });
    let end = screen.get_by_role(Role::Slider, "Loop end");
    end.a11y_with_value(A11yAction::SetValue, "12");
    screen
        .get_by_role(Role::Slider, "Loop start")
        .a11y_with_value(A11yAction::SetValue, "4");
    assert_eq!(range.get_untracked(), (4.0, 12.0));
    end.a11y_with_value(A11yAction::SetValue, "NaN");
    assert_eq!(range.get_untracked(), (4.0, 12.0));
    let r = end.rect().expect("loop end");
    end.drag(r.center(), Point::new(r.center().x + 26.0, r.center().y));
    assert_eq!(range.get_untracked(), (4.0, 13.0));
    let auto = screen.get_by_role(Role::Slider, "Vox automation point 1");
    auto.a11y_with_value(A11yAction::SetValue, "0.5");
    assert_eq!(tracks.get_untracked()[0].automation[0].value, 0.5);
    let r = auto.rect().expect("automation");
    auto.drag(
        r.center(),
        Point::new(r.center().x + 26.0, r.center().y - 8.5),
    );
    assert_eq!(tracks.get_untracked()[0].automation[0].beat, 3.0);
    assert!((tracks.get_untracked()[0].automation[0].value - 0.75).abs() < 0.01);
    let mute = screen.get_by_role(Role::Button, "Vox Mute");
    mute.click();
    assert!(tracks.get_untracked()[0].muted);
    assert_eq!(mute.is_checked(), Some(true));
    screen
        .get_by_test_id("arrangement")
        .press_key(Key::Char('3'));
    assert_eq!(tool.get_untracked(), ui::TimelineTool::Split);
    screen.get_by_role(Role::Button, "Clip").click();
    assert_eq!(clips.get_untracked().len(), 2);
    tool.set(ui::TimelineTool::Mute);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Clip (2)").click();
    assert!(clips.get_untracked()[1].muted);
    tool.set(ui::TimelineTool::Erase);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Clip (2)").click();
    assert_eq!(clips.get_untracked().len(), 1);
    clean(&screen);
}

#[test]
fn timecode_accepts_absolute_relative_frames_and_rejects_invalid_fields() {
    let (screen, frames) = kivact::test::render_with_handle(crate::options(), || {
        let frames = signal(0.0);
        (
            frames,
            ui::number("Timecode", frames)
                .timecode(24)
                .range(0.0, 86400.0),
        )
    });
    let field = screen.get_by_role(Role::SpinButton, "Timecode");
    for (entry, expected) in [
        ("00:00:03:14", 86.0),
        ("+12", 98.0),
        ("-24", 74.0),
        ("00:60:00:00", 74.0),
        ("00:00:00:24", 74.0),
        ("1:12", 36.0),
        ("1:02:12", 1500.0),
    ] {
        field.press_key(Key::Enter);
        let edit = screen.get_by_role(Role::TextBox, "Timecode");
        edit.type_text(entry);
        screen.press_key(Key::Enter);
        assert_eq!(frames.get_untracked(), expected, "{entry}");
        if entry == "00:60:00:00" || entry == "00:00:00:24" {
            assert!(edit.is_focused());
            screen.press_key(Key::Escape);
        }
        assert!(field.is_focused());
    }
    frames.set(85.6);
    screen.run_until_idle();
    assert!(screen.get_by_text("00:00:03:13").exists());
    clean(&screen);
}

#[test]
fn table_details_and_movable_interval_keep_public_consumer_state() {
    let (screen, (expanded, span)) = kivact::test::render_with_handle(crate::options(), || {
        let expanded = signal(true);
        let span = signal((20.0, 60.0));
        (
            (expanded, span),
            column((
                lines(
                    &["Name"],
                    signal(vec![line(1, ["Layer"])]),
                    signal(Vec::new()),
                )
                .row_details(move |_| {
                    kivact::ui::disclosure("Transform", expanded, || text("Position"))
                        .keep_mounted(true)
                }),
                ui::range("Selection", span)
                    .limits(0.0, 100.0)
                    .interval(rgb(0x8b7cff))
                    .movable(true)
                    .test_id("interval"),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Transform").click();
    assert!(!expanded.get_untracked());
    let owner = screen.get_by_test_id("interval").rect().expect("owner");
    let block = screen.get_by_role(Role::Button, "Selection interval");
    let r = block.rect().expect("interval");
    let p = Point::new(r.center().x - owner.x, r.center().y - owner.y);
    block.drag(p, Point::new(p.x + owner.width * 0.1, p.y));
    let moved = f64::from(owner.width * 0.1 / (owner.width - 20.0)) * 100.0;
    let (low, high) = span.get_untracked();
    assert!((low - 20.0 - moved).abs() < 1e-6 && (high - 60.0 - moved).abs() < 1e-6);
    clean(&screen);
}

#[test]
fn floating_window_drags_collapses_and_restores_opener_focus() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            column((
                kivact::ui::button("Another opener").variant(kivact::ui::ButtonVariant::Secondary),
                ui::window_button("Render Settings", ui::WindowOptions::default(), |_| {
                    column((
                        text("Retained settings"),
                        kivact::ui::text_input("Window text", signal(String::new()))
                            .value_text("Settings"),
                    ))
                    .into_view()
                }),
            )),
        )
    });
    let opener = screen.get_by_role(Role::Button, "Render Settings");
    opener.focus();
    opener.click();
    let title = screen.get_by_role(Role::Button, "Render Settings title bar");
    let retained = screen.get_by_text("Retained settings");
    let original = title.rect().expect("title");
    title.drag(Point::new(20.0, 15.0), Point::new(50.0, 35.0));
    let moved = title.rect().expect("moved title");
    assert!((moved.x - original.x - 30.0).abs() < 0.5);
    assert!((moved.y - original.y - 20.0).abs() < 0.5);
    title.drag(Point::new(20.0, 15.0), Point::new(-5000.0, -5000.0));
    let clamped = screen
        .get_by_role(Role::Group, "Render Settings window")
        .rect()
        .expect("clamped window");
    assert!((clamped.y - 0.0).abs() < 0.5);
    assert!((clamped.x + clamped.width - 80.0).abs() < 0.5);
    // Recover through the visible title-bar segment, rather than bypassing hit testing.
    screen.mouse_move(5.0, 8.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(505.0, 108.0);
    screen.mouse_up(MouseButton::Primary);
    assert!(title.rect().expect("recovered title").x > -100.0);
    title.a11y(A11yAction::Toggle);
    assert_eq!(title.is_expanded(), Some(false));
    assert!(!retained.is_visible());
    title.a11y(A11yAction::Toggle);
    assert!(retained.is_visible());
    screen.get_by_role(Role::TextBox, "Window text").focus();
    screen.press_key(Key::Escape);
    assert!(opener.is_focused());
    opener.click();
    screen.get_by_role(Role::TextBox, "Window text").focus();
    screen.get_by_role(Role::Button, "Close window").click();
    assert!(opener.is_focused());
    opener.click();
    screen.get_by_role(Role::TextBox, "Window text").focus();
    let other = screen.get_by_role(Role::Button, "Another opener");
    other.click();
    assert!(other.is_focused());
    assert!(
        screen
            .query_by_role(Role::Dialog, "Render Settings")
            .is_none()
    );
    clean(&screen);
}

#[test]
fn configured_window_placement_chrome_and_content_close_use_public_api() {
    let (screen, accepted) = kivact::test::render_with_handle(crate::options(), || {
        let accepted = signal(false);
        (
            accepted,
            ui::window_button(
                "Open tool窶ｦ",
                ui::WindowOptions::default()
                    .title("Tool settings")
                    .width(340.0)
                    .position(ui::WindowPosition::TopRight {
                        right: 40.0,
                        top: 120.0,
                    })
                    .frame_padding(0.0)
                    .body_padding(14.0)
                    .gap(0.0)
                    .radius(16.0)
                    .titlebar_height(40.0)
                    .titlebar_padding(14.0, 8.0)
                    .icon("settings"),
                move |close| {
                    column((
                        kivact::ui::text_input("Retained option", signal(String::new()))
                            .value_text("Original"),
                        kivact::ui::button("Apply and close")
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .on_press(move || {
                                accepted.set(true);
                                close.close();
                            }),
                    ))
                    .into_view()
                },
            ),
        )
    });
    let opener = screen.get_by_role(Role::Button, "Open tool窶ｦ");
    opener.click();
    let window = screen.get_by_role(Role::Group, "Tool settings window");
    let original = window.rect().expect("window");
    assert!((original.x - 420.0).abs() < 0.5, "{original:?}");
    assert!((original.y - 120.0).abs() < 0.5, "{original:?}");
    assert!((original.width - 340.0).abs() < 0.5, "{original:?}");
    let title = window.get_by_role(Role::Button, "Tool settings title bar");
    assert_eq!(title.rect().map(|r| r.height), Some(40.0));
    let edit = window.get_by_role(Role::TextBox, "Retained option");
    edit.type_text("Changed");
    screen.get_by_role(Role::Button, "Minimize window").click();
    assert_eq!(window.rect().map(|r| r.y), Some(120.0));
    assert!(!edit.is_visible());
    title.a11y(A11yAction::Toggle);
    assert!(edit.is_visible());
    assert_eq!(edit.value().as_deref(), Some("Changed"));
    title.drag(Point::new(60.0, 20.0), Point::new(35.0, 40.0));
    let moved = window.rect().expect("moved window");
    assert!((moved.x - original.x + 25.0).abs() < 0.5);
    assert!((moved.y - original.y - 20.0).abs() < 0.5);
    window.get_by_role(Role::Button, "Apply and close").click();
    assert!(accepted.get_untracked());
    assert!(opener.is_focused());
    assert!(
        screen
            .query_by_role(Role::Dialog, "Tool settings")
            .is_none()
    );
    clean(&screen);
}

#[test]
fn modal_window_preserves_backdrop_containment_and_close_callback_focus() {
    let (screen, background) = kivact::test::render_with_handle(crate::options(), || {
        let background = signal(0);
        (
            background,
            column((
                kivact::ui::button("Background command")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .on_press(move || background.update(|v| *v += 1)),
                ui::window_button(
                    "Confirm窶ｦ",
                    ui::WindowOptions::default()
                        .title("Confirmation")
                        .width(440.0)
                        .modal(true),
                    |close| {
                        kivact::ui::button("Accept")
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .on_press(move || close.close())
                            .into_view()
                    },
                ),
            )),
        )
    });
    let opener = screen.get_by_role(Role::Button, "Confirm窶ｦ");
    opener.click();
    let modal = screen.get_by_role(Role::Dialog, "Confirmation");
    assert!(
        screen
            .query_by_role(Role::Button, "Minimize window")
            .is_none()
    );
    let other = screen
        .get_by_role(Role::Button, "Background command")
        .rect()
        .expect("background");
    screen.mouse_move(other.center().x, other.center().y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert!(modal.exists());
    assert_eq!(background.get_untracked(), 0);
    screen.get_by_role(Role::Button, "Accept").click();
    assert!(!modal.exists());
    assert!(opener.is_focused());
    opener.click();
    assert!(screen.get_by_role(Role::Dialog, "Confirmation").exists());
    screen.press_key(Key::Escape);
    assert!(!modal.exists());
    assert!(opener.is_focused());
    clean(&screen);
}

#[test]
fn table_rows_are_one_tab_stop_and_follow_the_list_keys() {
    let (screen, (selection, rows)) = kivact::test::render_with_handle(crate::options(), || {
        let selection = signal(vec![1_u64]);
        let rows = signal(vec![
            line(1, ["Apple"]),
            line(2, ["Banana"]),
            line(3, ["Cherry"]),
            line(4, ["Date"]),
        ]);
        (
            (selection, rows),
            column((
                kivact::ui::button("Before").variant(kivact::ui::ButtonVariant::Secondary),
                lines(&["Name"], rows, selection),
            )),
        )
    });
    assert!(
        screen
            .get_by_role(Role::Grid, "Data table")
            .is_multiselectable()
    );
    let row = |name: &str| screen.get_by_role(Role::Row, name);
    // The header, then one tab stop for the rows: the selected row.
    screen.get_by_role(Role::Button, "Before").click();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::ColumnHeader, "Name").is_focused());
    screen.press_tab(false);
    assert!(row("Apple").is_focused());
    screen.press_tab(false);
    assert!(
        !screen.get_by_role(Role::Row, "Banana").is_focused(),
        "the other rows are not tab stops"
    );
    screen.press_tab(true);
    assert!(row("Apple").is_focused());
    screen.press_key(Key::ArrowDown);
    assert_eq!(selection.get_untracked(), vec![2]);
    assert!(row("Banana").is_focused());
    screen.press_key(Key::End);
    assert_eq!(selection.get_untracked(), vec![4]);
    assert!(row("Date").is_focused());
    screen.press_key(Key::Home);
    assert_eq!(selection.get_untracked(), vec![1]);
    assert!(row("Apple").is_focused());
    screen.press_key(Key::ArrowUp);
    assert!(row("Apple").is_focused(), "stops at the first row");
    // Ctrl moves the focus alone and Space toggles; Shift extends.
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    screen.press_key_with(Key::ArrowDown, ctrl);
    screen.press_key_with(Key::ArrowDown, ctrl);
    assert!(row("Cherry").is_focused());
    assert_eq!(selection.get_untracked(), vec![1]);
    screen.press_key_with(Key::Space, ctrl);
    assert_eq!(selection.get_untracked(), vec![1, 3]);
    screen.press_key_with(Key::ArrowDown, shift);
    assert_eq!(selection.get_untracked(), vec![3, 4]);
    // Ctrl+A and Cmd+A select every row.
    selection.set(Vec::new());
    screen.press_key_with(Key::Char('a'), ctrl);
    assert_eq!(selection.get_untracked(), vec![1, 2, 3, 4]);
    selection.set(Vec::new());
    screen.press_key_with(
        Key::Char('a'),
        Modifiers {
            meta: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(selection.get_untracked(), vec![1, 2, 3, 4]);
    // Typing a row's name focuses it; the filter and removal move the stop.
    screen.press_key(Key::Char('b'));
    assert!(row("Banana").is_focused());
    assert_eq!(selection.get_untracked(), vec![2]);
    rows.update(|rows| rows.retain(|row| row.id != 2));
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Before").click();
    screen.press_tab(false);
    screen.press_tab(false);
    assert!(row("Cherry").is_focused(), "the row that took its place");
    clean(&screen);
}

#[test]
fn table_rows_still_select_and_reorder_by_dragging() {
    let (screen, (selection, rows)) = kivact::test::render_with_handle(crate::options(), || {
        let selection = signal(Vec::<u64>::new());
        let rows = signal(vec![line(1, ["A"]), line(2, ["B"]), line(3, ["C"])]);
        (
            (selection, rows),
            lines(&["Name"], rows, selection).on_reorder(move |(id, destination)| {
                rows.update(|rows| {
                    let source = rows
                        .iter()
                        .position(|row| row.id == id)
                        .expect("row exists");
                    let row = rows.remove(source);
                    rows.insert(destination, row);
                })
            }),
        )
    });
    let a = screen.get_by_role(Role::Row, "A");
    let rect = a.rect().expect("row A");
    a.drag(
        Point::new(rect.width / 2.0, rect.height / 2.0),
        Point::new(rect.width / 2.0, rect.height * 2.0),
    );
    assert_eq!(selection.get_untracked(), vec![1], "the press selects");
    assert!(screen.get_by_role(Role::Row, "A").is_focused());
    assert_eq!(
        rows.get_untracked()
            .iter()
            .map(|r| r.id)
            .collect::<Vec<_>>(),
        vec![2, 1, 3]
    );
    clean(&screen);
}

#[test]
fn arrangement_more_than_128_tracks_and_large_knot_ids_remain_operable() {
    let (screen, (tracks, pan)) = kivact::test::render_with_handle(crate::options(), || {
        let mut values = (0..129)
            .map(|index| {
                ui::TimelineTrack::new(
                    index as u64,
                    format!("Track {index}"),
                    ui::TrackKind::Audio,
                    rgb(0x34d399),
                )
            })
            .collect::<Vec<_>>();
        values[128].automation = vec![ui::AutomationPoint::new(u64::MAX, 4.0, 0.5)];
        let tracks = signal(values);
        let pan = signal(Point::new(0.0, -64.0 * 60.0));
        (
            (tracks, pan),
            app_arrangement(
                signal(Vec::new()),
                tracks,
                ui::ArrangementState {
                    pan: pan.into(),
                    ..ui::ArrangementState::new()
                },
            ),
        )
    });
    screen.get_by_role(Role::Button, "Track 64 Mute").click();
    assert!(tracks.get_untracked()[64].muted);
    pan.set(Point::new(0.0, -128.0 * 60.0));
    screen.run_until_idle();
    screen
        .get_by_role(Role::Button, "Track 128 Solo")
        .a11y(A11yAction::Activate);
    assert!(tracks.get_untracked()[128].solo);
    screen
        .get_by_role(
            Role::Slider,
            &format!("Track 128 automation point {}", u64::MAX),
        )
        .a11y_with_value(A11yAction::SetValue, "0.75");
    assert_eq!(tracks.get_untracked()[128].automation[0].value, 0.75);
    clean(&screen);
}

#[test]
fn individual_node_links_select_and_delete_without_removing_nodes() {
    let (screen, (items, links, selected)) =
        kivact::test::render_with_handle(crate::options(), || {
            let socket = ui::NodeSocket::new;
            let items = signal(vec![
                test_node(
                    1,
                    "Source",
                    20.0,
                    vec![],
                    vec![socket("Image", "Image", kivact::style::rgb(0xffd25a))],
                ),
                test_node(
                    2,
                    "Destination",
                    400.0,
                    vec![
                        socket("Image", "Image", kivact::style::rgb(0xffd25a)),
                        socket("Other", "Image", kivact::style::rgb(0xffd25a)),
                    ],
                    vec![],
                ),
            ]);
            let links = signal(vec![
                ui::NodeLink::new(1, 0, 2, 0),
                ui::NodeLink::new(1, 0, 2, 1),
            ]);
            let selected = signal(None::<ui::NodeLink>);
            (
                (items, links, selected),
                app_node_editor(
                    items,
                    links,
                    ui::NodeState {
                        link: selected.into(),
                        ..ui::NodeState::new()
                    },
                ),
            )
        });
    let link = screen.get_by_role(Role::Button, "Link Source output 0 to Destination input 0");
    link.click();
    assert_eq!(
        selected.get_untracked(),
        Some(ui::NodeLink::new(1, 0, 2, 0))
    );
    assert!(link.is_selected());
    screen.press_key(Key::Delete);
    assert_eq!(items.get_untracked().len(), 2);
    assert_eq!(links.get_untracked(), vec![ui::NodeLink::new(1, 0, 2, 1)]);
    clean(&screen);
}

#[test]
fn color_popup_escape_rolls_back_while_outside_and_accessibility_dismissal_commit() {
    let (screen, color) = kivact::test::render_with_handle(crate::options(), || {
        let color = signal(rgb(0x8b7cff));
        (
            color,
            column((
                ui::color_popup("Edit color", color),
                kivact::ui::button("Other").variant(kivact::ui::ButtonVariant::Secondary),
            )),
        )
    });
    let original = color.get_untracked();
    let opener = screen.get_by_role(Role::Button, "Edit color");
    opener.click();
    screen
        .get_by_role(Role::SpinButton, "Color R")
        .a11y_with_value(A11yAction::SetValue, "0.25");
    assert_ne!(color.get_untracked(), original);
    screen.press_key(Key::Escape);
    assert_eq!(color.get_untracked(), original);
    assert!(opener.is_focused());
    opener.click();
    screen
        .get_by_role(Role::SpinButton, "Color R")
        .a11y_with_value(A11yAction::SetValue, "0.25");
    let committed = color.get_untracked();
    screen.get_by_role(Role::Button, "Other").click();
    assert_eq!(color.get_untracked(), committed);
    opener.click();
    screen
        .get_by_role(Role::SpinButton, "Color G")
        .a11y_with_value(A11yAction::SetValue, "0.5");
    let committed = color.get_untracked();
    screen
        .get_by_role(Role::Dialog, "Edit color")
        .a11y(A11yAction::Dismiss);
    assert_eq!(color.get_untracked(), committed);
    let next = rgb(0x446688);
    color.set(next);
    screen.run_until_idle();
    opener.a11y(A11yAction::Expand);
    screen
        .get_by_role(Role::SpinButton, "Color R")
        .a11y_with_value(A11yAction::SetValue, "0.75");
    screen.press_key(Key::Escape);
    assert_eq!(
        color.get_untracked(),
        next,
        "Expand snapshots the current color transaction"
    );
    clean(&screen);
}

#[test]
fn pie_hold_direction_release_and_keyboard_commands_share_public_state() {
    let (screen, (state, chosen)) = kivact::test::render_with_handle(crate::options(), || {
        let state = signal(ui::PieState::default());
        let chosen = signal(None::<usize>);
        let entries = vec![
            ui::PieEntry::new("Wireframe", 180.0, "box"),
            ui::PieEntry::new("Solid", 0.0, "eye"),
        ];
        (
            (state, chosen),
            column((ui::pie_menu(entries, state, move |index| {
                chosen.set(Some(index))
            }),))
            .role(Role::Group)
            .name("Pie host")
            .focusable(true)
            .style(style().size(640, 480))
            .on_key(move |event| {
                if event.key == Key::Char('z') {
                    state.set(ui::PieState::open_at(Point::new(300.0, 200.0), true));
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
            .on_key_up(move |event| {
                if event.key == Key::Char('z') && state.get_untracked().held {
                    chosen.set(state.get_untracked().hot);
                    state.set(ui::PieState::default());
                    Handled::Yes
                } else {
                    Handled::No
                }
            }),
        )
    });
    screen.get_by_role(Role::Group, "Pie host").focus();
    screen.key_down(Key::Char('z'), KeyCode::KeyZ, Modifiers::default());
    assert!(state.get_untracked().open);
    screen.mouse_move(196.0, 200.0);
    assert_eq!(state.get_untracked().hot, Some(0));
    screen.key_up(Key::Char('z'), KeyCode::KeyZ, Modifiers::default());
    assert_eq!(chosen.get_untracked(), Some(0));
    assert!(!state.get_untracked().open);
    state.set(ui::PieState::open_at(Point::new(300.0, 200.0), false));
    screen.advance(Duration::ZERO);
    screen
        .get_by_role(Role::Menu, "Shading pie")
        .press_key(Key::Char('2'));
    assert_eq!(chosen.get_untracked(), Some(1));
    assert!(!state.get_untracked().open);
    state.set(ui::PieState::open_at(Point::new(300.0, 200.0), true));
    screen.run_until_idle();
    screen
        .get_by_role(Role::Menu, "Shading pie")
        .press_key(Key::Escape);
    assert!(!state.get_untracked().open);
    assert!(
        !state.get_untracked().held,
        "shared dismissal cancels the held transaction"
    );
    clean(&screen);
}
#[test]
fn property_group_exclusive_expansion_drag_and_accessibility_reorder_keep_content() {
    let (screen, (panels, value)) = kivact::test::render_with_handle(crate::options(), || {
        let panels = signal(vec![
            ui::InspectorPanel::new(1, "Transform"),
            ui::InspectorPanel::new(2, "Relations"),
        ]);
        let value = signal(4.0);
        (
            (panels, value),
            ui::panel_group(panels, move |_| {
                ui::number("Retained", value).range(0.0, 10.0)
            }),
        )
    });
    let transform = screen.get_by_role(Role::Button, "Transform");
    screen
        .get_by_role(Role::Group, "Transform panel")
        .get_by_role(Role::SpinButton, "Retained")
        .a11y_with_value(A11yAction::SetValue, "7");
    screen.key_down(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    screen.get_by_role(Role::Button, "Relations").click();
    screen.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
    assert!(!panels.get_untracked()[0].expanded);
    assert!(panels.get_untracked()[1].expanded);
    assert_eq!(value.get_untracked(), 7.0);
    let grip = screen.get_by_role(Role::Button, "Reorder Relations");
    let a = grip.rect().expect("grip");
    let destination = transform.rect().expect("transform");
    grip.drag(
        Point::new(a.width / 2.0, a.height / 2.0),
        Point::new(destination.center().x - a.x, destination.y - a.y),
    );
    assert_eq!(panels.get_untracked()[0].id, 2);
    assert!(
        panels.get_untracked()[0].expanded,
        "dragging the grip does not toggle its panel"
    );
    screen
        .get_by_role(Role::Button, "Reorder Relations")
        .click();
    assert!(
        panels.get_untracked()[0].expanded,
        "clicking the grip does not toggle its panel"
    );
    screen
        .get_by_role(Role::Button, "Reorder Relations")
        .a11y(A11yAction::Increment);
    assert_eq!(panels.get_untracked()[0].id, 1);
    transform.a11y(A11yAction::Toggle);
    assert_eq!(value.get_untracked(), 7.0);
    assert!(
        screen
            .get_by_role(Role::Group, "Transform panel")
            .get_by_role(Role::SpinButton, "Retained")
            .is_visible()
    );
    clean(&screen);
}

#[test]
fn disposing_an_open_color_popup_restores_its_opening_color() {
    let (screen, (color, shown)) = kivact::test::render_with_handle(crate::options(), || {
        let color = signal(rgb(0x8b7cff));
        let shown = signal(true);
        (
            (color, shown),
            show(
                move || shown.get(),
                move || ui::color_popup("Edit color", color),
            ),
        )
    });
    let original = color.get_untracked();
    screen.get_by_role(Role::Button, "Edit color").click();
    screen
        .get_by_role(Role::SpinButton, "Color R")
        .a11y_with_value(A11yAction::SetValue, "0.25");
    assert_ne!(color.get_untracked(), original);
    shown.set(false);
    screen.run_until_idle();
    assert_eq!(color.get_untracked(), original);
    clean(&screen);
}

#[test]
fn color_popup_commits_a_released_picker_drag_on_outside_dismissal() {
    let (screen, color) = kivact::test::render_with_handle(crate::options(), || {
        let color = signal(rgb(0x8b7cff));
        (
            color,
            column((
                ui::color_popup("Edit color", color),
                kivact::ui::button("Other").variant(kivact::ui::ButtonVariant::Secondary),
            )),
        )
    });
    let original = color.get_untracked();
    screen.get_by_role(Role::Button, "Edit color").click();
    let picker = screen
        .get_by_role(Role::Slider, "Saturation and value")
        .rect()
        .expect("picker");
    screen.mouse_move(
        picker.x + picker.width * 0.25,
        picker.y + picker.height * 0.25,
    );
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(
        picker.x + picker.width * 0.75,
        picker.y + picker.height * 0.75,
    );
    screen.mouse_up(MouseButton::Primary);
    let committed = color.get_untracked();
    assert_ne!(committed, original);
    screen.get_by_role(Role::Button, "Other").click();
    assert_eq!(color.get_untracked(), committed);
    clean(&screen);
}

#[test]
fn arrangement_full_u64_clip_track_and_knot_ids_survive_reorder_split_and_retire() {
    let (screen, (clips, tracks, tool)) =
        kivact::test::render_with_handle(crate::options(), || {
            let clips = signal(vec![ui::Clip::new(
                u64::MAX,
                u64::MAX,
                4.0,
                8.0,
                "Full ID clip",
            )]);
            let tracks = signal(vec![
                ui::TimelineTrack::new(u64::MAX, "First", ui::TrackKind::Audio, rgb(0x34d399))
                    .automation(vec![ui::AutomationPoint::new(u64::MAX, 2.0, 0.25)]),
                ui::TimelineTrack::new(u64::MAX - 1, "Second", ui::TrackKind::Audio, rgb(0x8b7cff)),
            ]);
            let tool = signal(ui::TimelineTool::Select);
            (
                (clips, tracks, tool),
                app_arrangement(
                    clips,
                    tracks,
                    ui::ArrangementState {
                        tool: tool.into(),
                        ..ui::ArrangementState::new()
                    },
                )
                .style(style().size(700, 360)),
            )
        });
    let clip = screen.get_by_role(Role::Button, "Full ID clip");
    let mute = screen.get_by_role(Role::Button, "First Mute");
    let knot = screen.get_by_role(
        Role::Slider,
        &format!("First automation point {}", u64::MAX),
    );
    tracks.update(|tracks| tracks.reverse());
    screen.run_until_idle();
    assert_eq!(clip.name(), "Full ID clip");
    mute.a11y(A11yAction::Activate);
    knot.a11y_with_value(A11yAction::SetValue, "0.75");
    assert!(tracks.get_untracked()[1].muted);
    assert_eq!(tracks.get_untracked()[1].automation[0].value, 0.75);
    tool.set(ui::TimelineTool::Split);
    screen.run_until_idle();
    clip.click();
    assert_eq!(clips.get_untracked().len(), 2);
    assert_ne!(clips.get_untracked()[0].id, clips.get_untracked()[1].id);
    clips.update(|clips| clips.retain(|clip| clip.id != u64::MAX));
    screen.run_until_idle();
    assert_eq!(clip.name(), "");
    clips.update(|clips| {
        clips.push(ui::Clip::new(
            u64::MAX,
            u64::MAX,
            4.0,
            4.0,
            "Regenerated clip",
        ))
    });
    screen.run_until_idle();
    assert_eq!(clip.name(), "");
    screen.get_by_role(Role::Button, "Regenerated clip");
    clean(&screen);
}

#[test]
fn table_reorder_can_be_rejected_and_missing_handler_disables_drag() {
    let (screen, (rows, requests)) = kivact::test::render_with_handle(crate::options(), || {
        let rows = signal(vec![line(1, ["First"]), line(2, ["Second"])]);
        let requests = signal(Vec::<(u64, usize)>::new());
        (
            (rows, requests),
            column((
                lines(&["Name"], rows, signal(Vec::new()))
                    .on_reorder(move |request| requests.update(|all| all.push(request))),
                lines(
                    &["Name"],
                    vec![line(3, ["Disabled A"]), line(4, ["Disabled B"])],
                    signal(Vec::new()),
                ),
            )),
        )
    });
    let first = screen.get_by_role(Role::Row, "First");
    let rect = first.rect().unwrap();
    first.drag(
        Point::new(40.0, rect.height / 2.0),
        Point::new(40.0, rect.height / 2.0 + 35.0),
    );
    assert_eq!(requests.get_untracked(), [(1, 1)]);
    assert_eq!(rows.get_untracked()[0].id, 1);
    assert!(
        screen.get_by_role(Role::Row, "First").rect().unwrap().y
            < screen.get_by_role(Role::Row, "Second").rect().unwrap().y
    );
    let disabled = screen.get_by_role(Role::Row, "Disabled A");
    let rect = disabled.rect().unwrap();
    disabled.drag(
        Point::new(40.0, rect.height / 2.0),
        Point::new(40.0, rect.height / 2.0 + 35.0),
    );
    assert!(
        screen
            .get_by_role(Role::Row, "Disabled A")
            .rect()
            .unwrap()
            .y
            < screen
                .get_by_role(Role::Row, "Disabled B")
                .rect()
                .unwrap()
                .y
    );
    assert_eq!(requests.get_untracked(), [(1, 1)]);
    clean(&screen);
}
