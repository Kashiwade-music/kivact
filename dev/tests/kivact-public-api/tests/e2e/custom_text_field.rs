//! Role: Exercise a text editor composed only from public APIs.

use super::support::mount;
use kivact::custom::{NodeRefExt, TextEditing, TextSession, inline_view};
use kivact::draw::{TextBaseline, TextFormat};
use kivact::geometry::Point;
use kivact::interaction::text_edit_policy;
use kivact::prelude::*;
use kivact::style::{Color, Position, StyledRange};
use kivact::test::Screen;
use kivact::view::{EditPhase, Handled, KeyEvent, Modifiers, Node, NodeRef, node_ref};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct History {
    pending: Option<(String, TextSession)>,
    steps: Vec<(String, TextSession)>,
    restoring: bool,
}

/// Application-owned observations; none reach the engine or a private contract.
#[derive(Clone)]
struct Editor {
    committed: Signal<String>,
    session: Rc<RefCell<TextSession>>,
    phases: Rc<RefCell<Vec<(EditPhase, String)>>>,
    history: Rc<RefCell<History>>,
    numbers: Rc<RefCell<Vec<usize>>>,
    styled: Rc<RefCell<Vec<(String, bool)>>>,
    inline_presses: Signal<usize>,
    node: NodeRef,
}

/// The component supplies its own visuals, range storage and undo grouping;
/// interaction supplies just the standard key and clipboard policy.
fn code_editor(initial: &str) -> (Editor, Node) {
    let editor = Editor {
        committed: signal(initial.to_owned()),
        session: Rc::new(RefCell::new(TextSession::caret_at(initial.len()))),
        phases: Rc::default(),
        history: Rc::default(),
        numbers: Rc::default(),
        styled: Rc::default(),
        inline_presses: signal(0),
        node: node_ref(),
    };
    let shadow = Rc::new(RefCell::new(initial.to_owned()));
    let changes = editor.clone();
    let sessions = editor.session.clone();
    let committed_session = Rc::new(RefCell::new(TextSession::caret_at(initial.len())));
    let selection_before_edit = committed_session.clone();
    let phases = editor.clone();
    let syntax = editor.styled.clone();
    let editing = TextEditing::new(initial.to_owned())
        .multiline()
        .behavior(text_edit_policy)
        .on_text_change(move |event| {
            let mut text = shadow.borrow_mut();
            assert!(
                event.apply(&mut text),
                "range edits apply to the application's document"
            );
            if event.composition().is_none() {
                changes.committed.set(text.clone());
            }
        })
        .on_selection_change(move |session| {
            if !session.is_composing() {
                *committed_session.borrow_mut() = session.clone();
            }
            *sessions.borrow_mut() = session;
        })
        .on_edit(move |phase| {
            let text = phases.committed.get_untracked();
            phases.phases.borrow_mut().push((phase, text.clone()));
            let mut history = phases.history.borrow_mut();
            match phase {
                EditPhase::Begin if !history.restoring => {
                    history.pending = Some((text, selection_before_edit.borrow().clone()));
                }
                EditPhase::Commit if history.restoring => history.restoring = false,
                EditPhase::Commit => {
                    if let Some(before) = history.pending.take() {
                        if before.0 != text {
                            history.steps.push(before);
                        }
                    }
                }
                EditPhase::Cancel => history.pending = None,
                EditPhase::Begin => {}
            }
        })
        .line_styles((), move |_, line, inside: &bool| {
            syntax.borrow_mut().push((line.to_owned(), *inside));
            let comment = *inside || line.contains("/*");
            let ranges = if comment {
                vec![StyledRange {
                    range: 0..line.len(),
                    color: Some(rgb(0x16803a)),
                    background: None,
                    decoration: None,
                }]
            } else {
                line.char_indices()
                    .filter_map(|(offset, character)| {
                        character.is_ascii_digit().then_some(StyledRange {
                            range: offset..offset + 1,
                            color: Some(rgb(0x2040d0)),
                            background: None,
                            decoration: None,
                        })
                    })
                    .collect()
            };
            (ranges, comment && !line.contains("*/"))
        });
    let undo = editor.clone();
    let numbers = editor.numbers.clone();
    let inline_presses = editor.inline_presses;
    let inline = node()
        .role(Role::Button)
        .name("Inspect value")
        .focusable(true)
        .style(style().size(30, 22).background(rgb(0xe0e8ff)))
        .on_press(move || inline_presses.update(|count| *count += 1));
    let view = node()
        .node_ref(editor.node)
        .role(Role::TextBox)
        .name("Source code")
        .focusable(true)
        .style(
            style()
                .size(420, 180)
                .padding_left(36)
                .font_size(20)
                .font_family("monospace")
                .background(Color::WHITE),
        )
        .text_editing(editing)
        .on_key(move |event: KeyEvent| {
            let modifiers = event.modifiers();
            if event.key() != Key::Char('z') || !(modifiers.ctrl || modifiers.meta) {
                return Handled::No;
            }
            let previous = undo.history.borrow_mut().steps.pop();
            if let Some((text, session)) = previous {
                undo.history.borrow_mut().restoring = true;
                let mut state = undo.node.edit_state().expect("mounted native editor");
                state.replace(0..state.value().len(), &text);
                state.set_session(session);
                undo.node.set_edit_state(state);
            }
            Handled::Yes
        })
        .draw_with_layout(move |draw, layout| {
            let mut drawn = Vec::new();
            for (index, line) in layout.lines.iter().enumerate() {
                let number = index + 1;
                draw.text(
                    &number.to_string(),
                    Point::new(0., line.baseline),
                    TextFormat {
                        size: 16.,
                        color: rgb(0x606060),
                        baseline: TextBaseline::Alphabetic,
                        ..TextFormat::default()
                    },
                );
                drawn.push(number);
            }
            *numbers.borrow_mut() = drawn;
        })
        .child(
            node()
                .style(
                    style()
                        .position(Position::Absolute)
                        .left(280)
                        .top(0)
                        .width(40),
                )
                .child(inline_view(
                    " ",
                    TextFormat {
                        size: 20.,
                        ..TextFormat::default()
                    },
                    [(0, inline.into_view())],
                )),
        );
    (editor, view)
}

fn open(initial: &'static str) -> (Screen, Editor) {
    mount(move || code_editor(initial))
}

fn ctrl() -> Modifiers {
    Modifiers {
        ctrl: true,
        ..Modifiers::default()
    }
}
fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::default()
    }
}

#[test]
fn custom_text_field_text_entry_and_line_breaks() {
    let (screen, editor) = open("let x = 1;");
    let field = screen.get_by_role(Role::TextBox, "Source code");
    field.press_key(Key::End);
    field.type_text("日🦀");
    field.press_key(Key::Enter);
    field.type_text("next");
    assert_eq!(editor.committed.get_untracked(), "let x = 1;日🦀\nnext");
    assert_eq!(field.value().as_deref(), Some("let x = 1;日🦀\nnext"));
    assert_eq!(
        editor.node.edit_state().expect("editor state").value(),
        "let x = 1;日🦀\nnext"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn custom_text_field_ime_preedit_commit_and_cancel() {
    let (screen, editor) = open("let x = 1;");
    let field = screen.get_by_role(Role::TextBox, "Source code");
    field.press_key(Key::End);
    editor.phases.borrow_mut().clear();
    field.compose("にほ");
    assert_eq!(editor.committed.get_untracked(), "let x = 1;");
    assert_eq!(field.value().as_deref(), Some("let x = 1;にほ"));
    let ime = screen.ime_text().expect("platform IME session");
    assert_eq!(ime.composition, Some(10..12));
    assert!(ime.char_rects.iter().all(|rect| rect.height > 0.));
    assert!(
        editor.phases.borrow().is_empty(),
        "preedit creates no undo transaction"
    );
    field.commit("日本語");
    assert_eq!(editor.committed.get_untracked(), "let x = 1;日本語");
    assert_eq!(
        &*editor.phases.borrow(),
        &[
            (EditPhase::Begin, "let x = 1;".to_owned()),
            (EditPhase::Commit, "let x = 1;日本語".to_owned())
        ]
    );
    field.compose("仮");
    field.compose("");
    assert_eq!(editor.committed.get_untracked(), "let x = 1;日本語");
    assert_eq!(field.value().as_deref(), Some("let x = 1;日本語"));
    assert!(
        screen
            .ime_text()
            .expect("IME session")
            .composition
            .is_none()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn custom_text_field_selection_uses_graphemes_and_byte_ranges() {
    let (screen, editor) = open("a🦀日");
    let field = screen.get_by_role(Role::TextBox, "Source code");
    field.press_key(Key::End);
    field.press_key_with(Key::ArrowLeft, shift());
    field.press_key_with(Key::ArrowLeft, shift());
    assert_eq!(editor.session.borrow().selection(), 1..8);
    assert_eq!(
        field.text_selection(),
        Some((3, 1)),
        "a11y uses scalar offsets"
    );
    assert!(
        !editor
            .node
            .text_layout(0..1)
            .expect("shaped selection")
            .selection
            .is_empty()
    );
    field.type_text("X");
    assert_eq!(editor.committed.get_untracked(), "aX");
    field.press_key(Key::Backspace);
    assert_eq!(editor.committed.get_untracked(), "a");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn custom_text_field_clipboard_copy_cut_and_multiline_paste() {
    let (screen, editor) = open("let x = 1;");
    let field = screen.get_by_role(Role::TextBox, "Source code");
    field.press_key_with(Key::Char('a'), ctrl());
    field.press_key_with(Key::Char('c'), ctrl());
    assert_eq!(screen.clipboard().as_deref(), Some("let x = 1;"));
    assert_eq!(editor.committed.get_untracked(), "let x = 1;");
    field.press_key_with(Key::Char('x'), ctrl());
    assert_eq!(editor.committed.get_untracked(), "");
    assert_eq!(screen.clipboard().as_deref(), Some("let x = 1;"));
    screen.set_clipboard("let y = 2;\r\nreturn y;");
    field.press_key_with(Key::Char('v'), ctrl());
    assert_eq!(editor.committed.get_untracked(), "let y = 2;\nreturn y;");
    assert_eq!(field.text_selection(), Some((20, 20)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn custom_text_field_undo_groups_commits_with_edit_phase() {
    let (screen, editor) = open("let x = 1;");
    let field = screen.get_by_role(Role::TextBox, "Source code");
    field.press_key(Key::End);
    field.compose("に");
    field.compose("にほ");
    assert!(editor.history.borrow().steps.is_empty());
    field.commit("日本");
    assert_eq!(
        editor.history.borrow().steps.len(),
        1,
        "one IME commit is one undo step"
    );
    field.type_text("X");
    assert_eq!(editor.history.borrow().steps.len(), 2);
    field.press_key_with(Key::Char('z'), ctrl());
    assert_eq!(editor.committed.get_untracked(), "let x = 1;日本");
    assert_eq!(editor.history.borrow().steps.len(), 1);
    field.press_key_with(Key::Char('z'), ctrl());
    assert_eq!(editor.committed.get_untracked(), "let x = 1;");
    assert!(editor.history.borrow().steps.is_empty());
    assert_eq!(field.value().as_deref(), Some("let x = 1;"));
    assert!(
        editor
            .phases
            .borrow()
            .chunks_exact(2)
            .all(|pair| pair[0].0 == EditPhase::Begin && pair[1].0 == EditPhase::Commit)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn custom_text_field_accessibility_publishes_text_and_selection() {
    let (screen, editor) = open("a🦀日\nlet x = 1;");
    let field = screen.get_by_role(Role::TextBox, "Source code");
    assert_eq!(
        field.text_runs().expect("accessible editing runs").concat(),
        "a🦀日\nlet x = 1;"
    );
    field.reconvert(1..3);
    assert_eq!(field.text_selection(), Some((1, 3)));
    assert_eq!(editor.session.borrow().selection(), 1..8);
    assert!(field.text_caret().expect("accessible caret").height > 0.);
    field.a11y_set_value("日🦀\nchanged");
    assert_eq!(editor.committed.get_untracked(), "日🦀\nchanged");
    assert_eq!(
        field.text_runs().expect("updated accessible runs").concat(),
        "日🦀\nchanged"
    );
    assert_eq!(field.text_selection(), Some((10, 10)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn custom_text_field_draws_line_numbers_syntax_and_an_inline_control() {
    let (screen, editor) = open("/* open\ncomment */\nlet x = 2;");
    let _ = screen.screenshot();
    assert_eq!(&*editor.numbers.borrow(), &[1, 2, 3]);
    assert!(
        editor
            .styled
            .borrow()
            .iter()
            .any(|(line, state)| line == "comment */" && *state),
        "syntax state crosses line boundaries"
    );
    let field = screen.get_by_role(Role::TextBox, "Source code");
    let inline = field.get_by_role(Role::Button, "Inspect value");
    assert!(inline.rect().expect("inline control is laid out").height > 0.);
    inline.click();
    assert_eq!(editor.inline_presses.get_untracked(), 1);
    assert_eq!(
        editor.committed.get_untracked(),
        "/* open\ncomment */\nlet x = 2;"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
