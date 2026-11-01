//! Role: Tests for native editing attached to author parts.
//!
//! The `TextEditing` contract end to end on real windows: range edits and
//! sessions in `TextEditEvent`, a behavior closure, IME state, multiline
//! layout, programmatic state, read-only/disabled/limit rules, native leaf
//! diagnostics, carets of scrolled and empty editors, and editing attached
//! to text and image leaves.

use std::cell::RefCell;
use std::rc::Rc;

use kivact::control::canvas;
use kivact::draw::{Rect, RegionId};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::render;
use kivact::view::A11yAction;
use kivact::view::Modifiers;

/// `state`'s `(anchor, caret)` in Unicode scalar offsets of its text.
fn scalars(state: &kivact::primitive::EditState) -> (u32, u32) {
    (
        state.char_of_byte(state.anchor()),
        state.char_of_byte(state.caret()),
    )
}

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::default()
    }
}

#[test]
fn generic_part_editor_reports_native_edits_and_ime_state() {
    use kivact::primitive::{
        EditContext, EditOutcome, EditRequest, EditState, NodeRefExt, TextChange, TextEditEvent,
        TextEditing, part,
    };
    fn selection_policy(
        state: &mut EditState,
        request: EditRequest<'_>,
        _: &EditContext<'_>,
    ) -> EditOutcome {
        let EditRequest::Key {
            key: Key::ArrowLeft,
            modifiers,
        } = request
        else {
            return EditOutcome::NotEditing;
        };
        let selection = state.selection();
        let to = if !modifiers.shift && !selection.is_empty() {
            selection.start
        } else {
            state.prev_grapheme(state.caret())
        };
        state.move_to(to, modifiers.shift);
        EditOutcome::Done { changed: false }
    }
    let events: Rc<RefCell<Vec<TextEditEvent>>> = Rc::default();
    let seen = events.clone();
    let (screen, (reference, value)) = crate::support::mount(move || {
        // The editor's text is given once; edits keep a copy by range.
        let copy = RefCell::new(String::from("a🦀"));
        let value = signal(copy.borrow().clone());
        let editing = TextEditing::new("a🦀")
            .behavior(selection_policy)
            .on_edit(move |event| {
                event.apply(&mut copy.borrow_mut());
                if event.composition().is_none() {
                    value.set(copy.borrow().clone());
                }
                seen.borrow_mut().push(event);
            });
        let part = part(|state| state)
            .role(Role::TextBox)
            .name("Generic editor")
            .focusable(true)
            .text_editing(editing);
        let reference = part.node_ref();
        (
            (reference, value),
            node().part(part).style(style().size(220, 48)),
        )
    });
    assert_eq!(scalars(&reference.edit_state().unwrap()), (2, 2));
    let field = screen.get_by_role(Role::TextBox, "Generic editor");
    field.focus();
    assert_eq!(field.text_selection(), Some((2, 2)));
    assert_eq!(
        field.text_caret(),
        Some(reference.text_layout(0..0).unwrap().caret)
    );
    field.type_text("X");
    assert_eq!(value.get_untracked(), "a🦀X");
    let last = events.borrow().last().unwrap().clone();
    assert_eq!(
        last.changes,
        [TextChange {
            range: 5..5,
            text: "X".into()
        }]
    );
    assert_eq!(last.session.selection(), 6..6);
    assert_eq!(last.text, None);
    assert_eq!(reference.edit_session().unwrap().caret(), 6);
    field.compose("仮");
    assert_eq!(value.get_untracked(), "a🦀X");
    assert_eq!(events.borrow().last().unwrap().composition(), Some(6..9));
    assert_eq!(reference.edit_state().unwrap().marked(), Some(6..9));
    assert_eq!(screen.ime_text().unwrap().selection, 4..4);
    field.commit("仮名");
    assert_eq!(value.get_untracked(), "a🦀X仮名");
    assert_eq!(field.text_selection(), Some((5, 5)));
    assert_eq!(
        field.text_caret(),
        Some(reference.text_layout(0..0).unwrap().caret)
    );
    let last = events.borrow().last().unwrap().clone();
    assert_eq!(last.composition(), None);
    assert_eq!(
        last.changes,
        [TextChange {
            range: 6..9,
            text: "仮名".into()
        }]
    );
    field.a11y_with_value(A11yAction::SetValue, "AT🦀");
    assert_eq!(value.get_untracked(), "AT🦀");
    assert_eq!(scalars(&reference.edit_state().unwrap()), (3, 3));
    assert_eq!(field.value().as_deref(), Some("AT🦀"));
    field.press_key_with(Key::ArrowLeft, shift());
    assert_eq!(field.text_selection(), Some((3, 2)));
    assert_eq!(
        field.text_caret(),
        Some(reference.text_layout(0..0).unwrap().caret)
    );
    field.press_key(Key::ArrowLeft);
    assert_eq!(field.text_selection(), Some((2, 2)));
    let notifications = events.borrow().len();
    let mut state = reference.edit_state().unwrap();
    state.move_to(0, false);
    reference.set_edit_state(state);
    screen.run_until_idle();
    assert_eq!(scalars(&reference.edit_state().unwrap()), (0, 0));
    assert_eq!(events.borrow().len(), notifications);
    let mut state = reference.edit_state().unwrap();
    state.compose(Some(0..1), "A", Some(0..1));
    reference.set_edit_state(state);
    screen.run_until_idle();
    assert_eq!(events.borrow().len(), notifications + 1);
    assert_eq!(events.borrow().last().unwrap().composition(), Some(0..1));
    assert_eq!(
        events.borrow().last().unwrap().changes,
        [TextChange {
            range: 0..1,
            text: "A".into()
        }]
    );
    let mut state = reference.edit_state().unwrap();
    state.unmark();
    reference.set_edit_state(state);
    screen.run_until_idle();
    assert_eq!(events.borrow().len(), notifications + 2);
    assert_eq!(events.borrow().last().unwrap().composition(), None);
    assert!(events.borrow().last().unwrap().changes.is_empty());
    assert_eq!(value.get_untracked(), "AT🦀");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn generic_part_editor_exposes_multiline_layout_and_programmatic_selection() {
    use kivact::primitive::{NodeRefExt, TextEditing, part};
    let (screen, reference) = crate::support::mount(|| {
        let editor = part(|state| state)
            .role(Role::TextBox)
            .name("Lines")
            .focusable(true)
            .text_editing(TextEditing::new("a🦀\n日").multiline(true));
        let reference = editor.node_ref();
        (
            reference,
            node()
                .part(editor)
                .style(style().size(180, 80).padding(8).translate(12, 9).scale(1.2)),
        )
    });
    let field = screen.get_by_role(Role::TextBox, "Lines");
    field.focus();
    let layout = reference.text_layout(0..2).unwrap();
    assert_eq!(layout.line_count, 2);
    assert_eq!(
        layout
            .lines
            .iter()
            .map(|line| line.range.clone())
            .collect::<Vec<_>>(),
        [0..6, 6..9]
    );
    assert!(layout.lines[1].rect.y > layout.lines[0].rect.y);
    assert!(layout.lines.iter().all(|line| line.baseline >= line.rect.y));
    assert_eq!(layout.carets.last().unwrap().offset, 4);
    // A part of the lines: its carets keep the value's scalar offsets.
    let second = reference.text_layout(1..9).unwrap();
    assert_eq!(second.line_count, 2);
    assert_eq!(second.lines, layout.lines[1..]);
    assert_eq!(second.carets.first().unwrap().offset, 3);
    assert_eq!(second.carets.last(), layout.carets.last());
    assert_eq!(
        layout.caret,
        *screen.ime_text().unwrap().char_rects.last().unwrap()
    );
    let mut state = reference.edit_state().unwrap();
    state.move_to(0, false);
    state.move_to(5, true);
    reference.set_edit_state(state);
    screen.run_until_idle();
    assert_eq!(scalars(&reference.edit_state().unwrap()), (0, 2));
    assert_eq!(field.text_selection(), Some((0, 2)));
    assert_eq!(screen.ime_text().unwrap().selection, 0..2);
    let moved = reference.text_layout(0..0).unwrap();
    assert_eq!(
        moved.caret,
        *screen.ime_text().unwrap().char_rects.last().unwrap()
    );
    assert!(moved.caret.y < layout.caret.y);
    assert_eq!(field.text_caret(), Some(moved.caret));
    let mut state = reference.edit_state().unwrap();
    state.move_to(9, false);
    state.move_to(1, true);
    reference.set_edit_state(state);
    screen.run_until_idle();
    assert_eq!(field.text_selection(), Some((4, 1)));
    assert_eq!(
        field.text_caret(),
        Some(reference.text_layout(0..0).unwrap().caret)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn generic_part_editor_respects_read_only_disabled_and_reactive_settings() {
    use kivact::primitive::{EditState, NodeRefExt, TextEditing, part};
    let (screen, (reference, read_only, disabled, password)) = crate::support::mount(|| {
        let read_only = signal(true);
        let disabled = signal(false);
        let password = signal(false);
        let editing = TextEditing::new("abc")
            .read_only(read_only)
            .password(password)
            .max_length(4usize)
            .placeholder("Empty");
        let part = part(|state| state)
            .role(Role::TextBox)
            .name("Guarded")
            .focusable(true)
            .text_editing(editing);
        let reference = part.node_ref();
        (
            (reference, read_only, disabled, password),
            node()
                .part(part)
                .disabled(disabled)
                .style(style().size(180, 40)),
        )
    });
    reference.set_edit_state(EditState::new("rejected"));
    screen.run_until_idle();
    assert_eq!(reference.edit_state().unwrap().value(), "abc");
    read_only.set(false);
    disabled.set(true);
    screen.run_until_idle();
    reference.set_edit_state(EditState::new("disabled"));
    screen.run_until_idle();
    assert_eq!(reference.edit_state().unwrap().value(), "abc");
    disabled.set(false);
    screen.run_until_idle();
    screen.get_by_role(Role::TextBox, "Guarded").type_text("YZ");
    assert_eq!(reference.edit_state().unwrap().value(), "abcY");
    reference.set_edit_state(EditState::new("a🦀XYZ"));
    screen.run_until_idle();
    let state = reference.edit_state().unwrap();
    assert_eq!(state.value(), "a🦀XY");
    assert_eq!(scalars(&state), (4, 4));
    screen.get_by_role(Role::TextBox, "Guarded").type_text("Q");
    assert_eq!(reference.edit_state().unwrap().value(), "a🦀XY");
    let field = screen.get_by_role(Role::TextBox, "Guarded");
    assert_eq!(field.text_selection(), Some((4, 4)));
    password.set(true);
    screen.run_until_idle();
    assert_eq!(field.value(), None);
    assert_eq!(field.text_selection(), None);
    assert_eq!(field.text_caret(), None);
    password.set(false);
    screen.run_until_idle();
    assert_eq!(field.text_selection(), Some((4, 4)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn generic_part_editor_uses_native_leaf_diagnostics_and_ignores_region_editing() {
    use kivact::primitive::{TextEditing, part};
    let screen = render(|| {
        column((
            node()
                .part(
                    part(|state| state)
                        .role(Role::TextBox)
                        .name("Leaf")
                        .text_editing(TextEditing::new("native")),
                )
                .child(text("Dropped child"))
                .draw(|_, _| panic!("native editors drop custom drawing"))
                .style(style().size(180, 40)),
            canvas(|draw, _| {
                draw.region(RegionId(9), Rect::new(0., 0., 80., 30.));
            })
            .region(
                RegionId(9),
                part(|state| state)
                    .role(Role::Button)
                    .name("Region")
                    .text_editing(TextEditing::new("ignored")),
            )
            .style(style().size(100, 40)),
        ))
    });
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Leaf").value().as_deref(),
        Some("native")
    );
    assert!(screen.query_by_text("Dropped child").is_none());
    assert_eq!(screen.get_by_role(Role::Button, "Region").value(), None);
    let errors = screen.errors();
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(
        errors
            .iter()
            .all(|error| *error.kind() == kivact::diagnostics::ErrorKind::Unsupported)
    );
}

#[test]
fn generic_part_editor_publishes_scrolled_and_empty_carets() {
    use kivact::primitive::{NodeRefExt, TextEditing, part};
    let (screen, reference) = crate::support::mount(|| {
        let editor = part(|state| state)
            .role(Role::TextBox)
            .name("Scrolled editor")
            .focusable(true)
            .text_editing(TextEditing::new("日本語🦀abcdefghijklmnopqrstuvwxyz"));
        let reference = editor.node_ref();
        (
            reference,
            node()
                .part(editor)
                .style(style().size(90, 40).padding(8).translate(15, 12).scale(1.2)),
        )
    });
    let field = screen.get_by_role(Role::TextBox, "Scrolled editor");
    field.focus();
    assert_eq!(field.text_selection(), Some((30, 30)));
    assert_eq!(
        field.text_caret(),
        Some(reference.text_layout(0..0).unwrap().caret)
    );
    let bounds = field.rect().unwrap();
    let caret = field.text_caret().unwrap();
    assert!(caret.x >= bounds.x && caret.x <= bounds.right());
    field.a11y_with_value(A11yAction::SetValue, "");
    assert_eq!(field.text_selection(), Some((0, 0)));
    assert_eq!(
        field.text_caret(),
        Some(reference.text_layout(0..0).unwrap().caret)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn editing_attachment_overrides_text_and_image_leaves() {
    use kivact::primitive::{NodeRefExt, TextEditing, part};
    let (screen, (references, values)) = crate::support::mount(|| {
        let first = signal(String::from("a"));
        let second = signal(String::from("b"));
        let text_part = part(|input| input).focusable(true).text_editing(
            TextEditing::new(move || Rc::<str>::from(first.get()))
                .report_text(true)
                .on_edit(move |event| {
                    if event.composition().is_none() {
                        first.set(event.text.unwrap());
                    }
                }),
        );
        let image_part = part(|input| input).focusable(true).text_editing(
            TextEditing::new(move || Rc::<str>::from(second.get()))
                .report_text(true)
                .on_edit(move |event| {
                    if event.composition().is_none() {
                        second.set(event.text.unwrap());
                    }
                }),
        );
        let text_reference = text_part.node_ref();
        let image_reference = image_part.node_ref();
        (
            ([text_reference, image_reference], [first, second]),
            column((
                text("unused paragraph")
                    .part(text_part)
                    .role(Role::TextBox)
                    .name("Text leaf editor"),
                image(include_bytes!("../fixtures/rgba.png"))
                    .part(image_part)
                    .role(Role::TextBox)
                    .name("Image leaf editor"),
            ))
            .style(style().align_items(Align::Start).gap(8)),
        )
    });
    for ((reference, value), (name, prefix)) in references
        .iter()
        .zip(values)
        .zip([("Text leaf editor", "a"), ("Image leaf editor", "b")])
    {
        let field = screen.get_by_role(Role::TextBox, name);
        field.focus();
        field.type_text("🦀");
        assert_eq!(value.get_untracked(), format!("{prefix}🦀"));
        field.compose("仮");
        assert_eq!(value.get_untracked(), format!("{prefix}🦀"));
        field.commit("日");
        assert_eq!(value.get_untracked(), format!("{prefix}🦀日"));
        assert_eq!(scalars(&reference.edit_state().unwrap()), (3, 3));
        assert_eq!(field.text_selection(), Some((3, 3)));
        assert_eq!(
            field.text_caret(),
            Some(reference.text_layout(0..0).unwrap().caret)
        );
        assert!(field.text_caret().unwrap().height > 0.);
        assert!(field.rect().unwrap().width >= 120.);
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
