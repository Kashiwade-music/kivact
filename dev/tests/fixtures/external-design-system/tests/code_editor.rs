//! Role: Tests drive the public-API code editor through the native window driver.

use std::cell::RefCell;
use std::rc::Rc;

use kivact::prelude::*;
use kivact::primitive::{NodeRefExt, TextLayout};
use kivact::test::{RenderOptions, Screen, WindowMode, render_with};
use kivact::view::{A11yAction, Modifiers};
use kivact_external_design_system::{COMMENT, CodeEditor, KEYWORD, button, code_editor};

fn render(root: impl FnOnce() -> Node + 'static) -> Screen {
    render_with(
        RenderOptions {
            window: WindowMode::Real,
            gpu: cfg!(feature = "gpu"),
            ..Default::default()
        },
        root,
    )
}

/// The editor and its keyword list, handed out of the setup.
type Setup = Rc<RefCell<Option<(CodeEditor, Signal<Vec<String>>)>>>;

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

/// Whether the glyphs between scalar offsets `from` and `to` (on one line)
/// have pixels of the keyword color.
fn colored(screen: &Screen, layout: &TextLayout, from: u32, to: u32) -> bool {
    painted(screen, layout, from, to, KEYWORD)
}

/// Whether the glyphs between scalar offsets `from` and `to` (on one line)
/// have pixels of `color`.
fn painted(screen: &Screen, layout: &TextLayout, from: u32, to: u32, color: u32) -> bool {
    let caret = |offset: u32| {
        layout
            .carets
            .iter()
            .find(|c| c.offset == offset)
            .unwrap_or_else(|| panic!("no caret at {offset}"))
            .rect
    };
    let (a, b) = (caret(from), caret(to));
    assert!((a.y - b.y).abs() < 1.0, "{from}..{to} spans lines");
    let scale = screen.scale();
    let px = |v: f32| (v * scale).round().max(0.0) as u32;
    let shot = screen.screenshot();
    let shot = shot.crop(px(a.x + 1.0), px(a.y), px(b.x - a.x - 2.0), px(a.height));
    let (r, g, bl) = (color >> 16, (color >> 8) & 0xff, color & 0xff);
    shot.pixels.iter().any(|p| {
        let (pr, pg, pb) = (p >> 16, (p >> 8) & 0xff, p & 0xff);
        pr.abs_diff(r) < 48 && pg.abs_diff(g) < 48 && pb.abs_diff(bl) < 48
    })
}

#[test]
fn a_public_code_editor_indents_colors_undoes_and_counts_lines_from_range_edits() {
    let state: Setup = Rc::default();
    let setup = state.clone();
    let screen = render(move || {
        let keywords = signal(vec![String::from("fn")]);
        let (view, editor) = code_editor("Code", "fn main() {\n}", keywords);
        *setup.borrow_mut() = Some((editor, keywords));
        column((view, button("After"))).style(style().padding(24).gap(12))
    });
    let (editor, keywords) = state.borrow().clone().unwrap();
    let field = screen.get_by_role(Role::TextBox, "Code");
    // Every line of the short document.
    let layout = || editor.node_ref().text_layout(0..usize::MAX).unwrap();
    assert_eq!(editor.lines().get_untracked(), 2);
    assert_eq!(field.value().as_deref(), Some("fn main() {\n}"));
    assert!(colored(&screen, &layout(), 0, 2), "the keyword is colored");
    assert!(!colored(&screen, &layout(), 3, 7), "a name is not");

    // Enter keeps the indentation, Tab indents, typing goes through.
    field.focus();
    field.press_key(Key::Enter);
    assert_eq!(editor.lines().get_untracked(), 3);
    field.press_key(Key::Tab);
    assert!(field.is_focused(), "Tab indents instead of moving focus");
    field.type_text("let x = 1;");
    assert_eq!(editor.text(), "fn main() {\n}\n    let x = 1;");
    assert_eq!(field.value().as_deref(), Some(editor.text().as_str()));
    assert!(!colored(&screen, &layout(), 18, 21));

    // A new keyword list colors again; so does typing a keyword.
    keywords.set(vec![String::from("fn"), String::from("let")]);
    screen.run_until_idle();
    assert!(
        colored(&screen, &layout(), 18, 21),
        "the new keyword is colored"
    );
    assert!(!colored(&screen, &layout(), 22, 23));
    field.press_key(Key::Enter);
    assert_eq!(editor.text(), "fn main() {\n}\n    let x = 1;\n    ");
    assert_eq!(editor.lines().get_untracked(), 4);
    field.type_text("fn y");
    assert!(
        colored(&screen, &layout(), 33, 35),
        "a typed keyword is colored"
    );

    // Undo goes back one edit at a time, the line count with it.
    for _ in 0..5 {
        field.press_key_with(Key::Char('z'), ctrl());
    }
    assert_eq!(editor.text(), "fn main() {\n}\n    let x = 1;");
    assert_eq!(editor.lines().get_untracked(), 3);

    // A composition is one step: its preedit is in the text until it ends.
    field.compose("仮");
    assert!(editor.node_ref().edit_state().unwrap().is_composing());
    assert_eq!(editor.text(), "fn main() {\n}\n    let x = 1;仮");
    field.commit("名");
    assert_eq!(editor.text(), "fn main() {\n}\n    let x = 1;名");
    assert_eq!(field.value().as_deref(), Some(editor.text().as_str()));
    field.press_key_with(Key::Char('z'), ctrl());
    assert_eq!(editor.text(), "fn main() {\n}\n    let x = 1;");

    // Shift+Tab dedents the line; the selection and the line keys.
    field.press_key_with(Key::Tab, shift());
    assert_eq!(editor.text(), "fn main() {\n}\nlet x = 1;");
    field.press_key_with(Key::ArrowLeft, shift());
    field.press_key_with(Key::ArrowLeft, shift());
    assert_eq!(field.text_selection(), Some((24, 22)));
    assert_eq!(
        editor.node_ref().edit_session().unwrap().selection(),
        22..24
    );
    field.press_key(Key::Home);
    assert_eq!(field.text_selection(), Some((14, 14)));
    field.press_key(Key::ArrowUp);
    assert_eq!(field.text_selection(), Some((12, 12)));
    field.press_key(Key::ArrowDown);
    assert_eq!(field.text_selection(), Some((14, 14)));

    // Assistive technology sets the whole value like typing.
    field.a11y_with_value(A11yAction::SetValue, "fn a()\nfn b()\n");
    assert_eq!(editor.text(), "fn a()\nfn b()\n");
    assert_eq!(editor.lines().get_untracked(), 3);
    assert_eq!(field.value().as_deref(), Some("fn a()\nfn b()\n"));

    // Escape, then Tab, leaves the editor.
    field.press_key(Key::Escape);
    field.press_key(Key::Tab);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn block_comments_color_across_lines_and_the_caret_position_follows_the_session() {
    let state: Setup = Rc::default();
    let setup = state.clone();
    let text = "fn a() {}
/* one
two fn */ fn b
fn c";
    let screen = render(move || {
        let keywords = signal(vec![String::from("fn")]);
        let (view, editor) = code_editor("Code", text, keywords);
        *setup.borrow_mut() = Some((editor, keywords));
        column((view, button("After"))).style(style().padding(24).gap(12))
    });
    let (editor, _) = state.borrow().clone().unwrap();
    let field = screen.get_by_role(Role::TextBox, "Code");
    // Every line of the short document.
    let layout = || editor.node_ref().text_layout(0..usize::MAX).unwrap();
    // Line 1 opens a comment line 2 closes; a keyword inside it is a
    // comment, the one after it a keyword again.
    assert!(colored(&screen, &layout(), 0, 2));
    assert!(painted(&screen, &layout(), 10, 16, COMMENT));
    assert!(painted(&screen, &layout(), 17, 23, COMMENT));
    assert!(
        !colored(&screen, &layout(), 21, 23),
        "a keyword in a comment"
    );
    assert!(colored(&screen, &layout(), 27, 29), "after the comment");
    assert!(colored(&screen, &layout(), 32, 34));
    // The caret's line and column, from the session.
    assert_eq!(editor.caret().get_untracked(), (4, 5));
    field.focus();
    field.press_key(Key::Home);
    assert_eq!(editor.caret().get_untracked(), (4, 1));
    field.press_key(Key::ArrowUp);
    field.press_key(Key::ArrowUp);
    assert_eq!(editor.caret().get_untracked(), (2, 1));
    // Opening a comment on the first line colors it and the line before
    // the old opening as a comment; the lines after the close keep theirs.
    field.press_key(Key::ArrowUp);
    field.type_text("/*");
    assert_eq!(editor.caret().get_untracked(), (1, 3));
    assert!(
        painted(&screen, &layout(), 2, 4, COMMENT),
        "fn is in the comment"
    );
    assert!(painted(&screen, &layout(), 12, 18, COMMENT));
    assert!(colored(&screen, &layout(), 29, 31), "after the comment");
    // Undoing it (two typed characters) gives the first line its keyword
    // back.
    field.press_key_with(Key::Char('z'), ctrl());
    field.press_key_with(Key::Char('z'), ctrl());
    assert!(colored(&screen, &layout(), 0, 2));
    assert_eq!(editor.text(), text);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
