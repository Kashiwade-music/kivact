//! Role: A one-line text editor built only from the three text primitives.
//!
//! The editor is a `primitive()` that draws its own text. It uses no
//! `text_input` and no `TextEditing`: everything a native editor does for
//! it, the editor does with
//!
//! - a **paragraph** (`Constraints::measure_paragraph` sizes the node,
//!   `Draw::layout_paragraph` and `Draw::paragraph` draw the text, and
//!   `TextParagraph::{caret, hit, rects}` place the caret, answer clicks
//!   and paint the selection),
//! - an **IME session** (`ime_session` gives the platform the committed
//!   text, the selection and the caret; `on_ime` receives the composition,
//!   the commit and the reconversion range), and
//! - **accessibility text** (`semantics_text` publishes the runs, the
//!   selection and the caret of the paragraph it drew).
//!
//! The state is the committed text, the selection (an anchor and a caret, in
//! bytes) and the composition being shown. The composition is drawn inside
//! the text at the caret, underlined, and replaced by a commit.
//!
//! The caret moves along grapheme and shaping-cluster boundaries. There is
//! no clipboard, undo or horizontal scrolling.

use std::ops::Range;

use kivact::draw::{Point, Rect, Size, TextStyle};
use kivact::prelude::*;
use kivact::primitive::{
    Constraints, ImeEvent, ImeSession, Measurement, NodeExt, Primitive, primitive, shape_paragraph,
};
use kivact::view::{Handled, Key, KeyEvent, MouseButton, PointerEvent, PointerPhase};

/// The space between the node's edge and its text.
const PAD: Point = Point { x: 6.0, y: 4.0 };
/// The narrowest the editor is.
const MIN_WIDTH: f32 = 160.0;

/// The composition the IME is showing: its text and where the IME's caret is
/// in it (a byte offset of `text`).
#[derive(Clone, Debug, PartialEq)]
struct Composition {
    text: String,
    cursor: usize,
}

/// A one-line editor of `text`, named `name` for assistive technology.
///
/// `text` is the committed text: the editor writes it as the user types and
/// commits, and shows what the app writes to it. The caret starts at the end.
pub fn line_editor(name: &'static str, text: Signal<String>) -> Primitive {
    let anchor = signal(text.get_untracked().len());
    let caret = signal(text.get_untracked().len());
    let composition: Signal<Option<Composition>> = signal(None);
    // What is on screen: the committed text with the composition at the
    // caret, as `(text, composition range in it, caret offset in it)`.
    let display = move || {
        let text = text.get();
        let at = floor(&text, caret.get());
        match composition.get() {
            Some(c) => {
                let range = at..at + c.text.len();
                let cursor = at + floor(&c.text, c.cursor);
                (
                    format!("{}{}{}", &text[..at], c.text, &text[at..]),
                    Some(range),
                    cursor,
                )
            }
            None => (text, None, at),
        }
    };
    // The selection's anchor in what is on screen (the caret while composing).
    let anchor_shown = move || {
        if composition.get().is_some() {
            display().2
        } else {
            floor(&text.get(), anchor.get())
        }
    };
    let selection = move || {
        let len = text.get().len();
        let (a, c) = (anchor.get().min(len), caret.get().min(len));
        a.min(c)..a.max(c)
    };

    let measured = move || display().0;

    primitive()
        .measure(measured, |text: &String, c: Constraints| {
            let paragraph = c.measure_paragraph(text, &style_of(), None);
            let Size { width, height } = paragraph.size();
            Measurement::new(Size::new(
                (width + 2.0 * PAD.x + 2.0).max(MIN_WIDTH),
                height + 2.0 * PAD.y,
            ))
            .baseline(paragraph.baseline() + PAD.y)
        })
        .role(Role::TextBox)
        .name(name)
        .focusable(true)
        .style(
            style()
                .border(1, ColorToken::Border)
                .radius(4)
                .background(ColorToken::Surface),
        )
        .draw(move |d, _| {
            let (shown_text, preedit, at) = display();
            let paragraph = d.layout_paragraph(&shown_text, &style_of(), None);
            if preedit.is_none() {
                for band in paragraph.rects(selection()) {
                    d.fill_rect(offset(band), ColorToken::Selection);
                }
            }
            d.paragraph(&paragraph, PAD, ColorToken::Text);
            for band in preedit.map(|r| paragraph.rects(r)).unwrap_or_default() {
                let band = offset(band);
                d.fill_rect(
                    Rect::new(band.x, band.y + band.height - 1.5, band.width, 1.5),
                    ColorToken::Text,
                );
            }
            let c = offset(paragraph.caret(at));
            d.fill_rect(Rect::new(c.x, c.y, 1.5, c.height), ColorToken::Text);
        })
        .ime_session(ImeSession::new(text, selection, move || {
            let (shown_text, _, at) = display();
            offset(shape_paragraph(&shown_text, &style_of(), None).caret(at))
        }))
        .on_ime(move |event| match event {
            ImeEvent::Preedit {
                text: preedit,
                selection,
            } => {
                if preedit.is_empty() {
                    composition.set(None);
                    return;
                }
                if composition.get().is_none() {
                    // A composition replaces the selection.
                    delete_selection(text, anchor, caret);
                }
                let cursor = selection.map_or(preedit.len(), |r| r.end);
                composition.set(Some(Composition {
                    text: preedit,
                    cursor,
                }));
            }
            ImeEvent::Commit(committed) => {
                composition.set(None);
                insert(text, anchor, caret, &committed);
            }
            // A reconversion: the IME took this range, which the next
            // composition replaces.
            ImeEvent::Selection(range) => {
                let len = text.get().len();
                anchor.set(range.start.min(len));
                caret.set(range.end.min(len));
            }
            ImeEvent::End => composition.set(None),
        })
        .semantics_text(move || {
            let (shown_text, _, at) = display();
            shape_paragraph(&shown_text, &style_of(), None).semantics_text(PAD, anchor_shown(), at)
        })
        .on_key(move |e: KeyEvent| {
            // The IME owns the keys while it composes.
            if composition.get().is_some() {
                return Handled::No;
            }
            edit_key(&e, text, anchor, caret)
        })
        .on_pointer(move |e: PointerEvent| {
            if composition.get().is_some() {
                return Handled::No;
            }
            let held = e.buttons.contains(MouseButton::Primary);
            let press = e.phase == PointerPhase::Down && e.button == Some(MouseButton::Primary);
            if !(press || (e.phase == PointerPhase::Move && held)) {
                return Handled::No;
            }
            let paragraph = shape_paragraph(&text.get(), &style_of(), None);
            let at = paragraph.hit(Point::new(e.pos.x - PAD.x, e.pos.y - PAD.y));
            caret.set(floor(&text.get(), at));
            if press && !e.modifiers.shift {
                anchor.set(caret.get());
            }
            Handled::Yes
        })
}

fn style_of() -> TextStyle {
    TextStyle {
        size: 14.0,
        ..TextStyle::default()
    }
}

/// `r` (in the paragraph's coordinates) in the node's.
fn offset(r: Rect) -> Rect {
    Rect {
        x: r.x + PAD.x,
        y: r.y + PAD.y,
        ..r
    }
}

/// `offset` cut to `text` and to the char boundary at or before it.
fn floor(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

/// The selection's bytes.
fn span(anchor: Signal<usize>, caret: Signal<usize>) -> Range<usize> {
    let (a, c) = (anchor.get(), caret.get());
    a.min(c)..a.max(c)
}

/// Remove the selected text, leaving the caret where it was.
fn delete_selection(text: Signal<String>, anchor: Signal<usize>, caret: Signal<usize>) -> bool {
    let range = span(anchor, caret);
    let current = text.get();
    let range = floor(&current, range.start)..floor(&current, range.end);
    if range.is_empty() {
        return false;
    }
    text.update(|t| t.replace_range(range.clone(), ""));
    anchor.set(range.start);
    caret.set(range.start);
    true
}

/// Replace the selection with `inserted`, the caret after it.
fn insert(text: Signal<String>, anchor: Signal<usize>, caret: Signal<usize>, inserted: &str) {
    delete_selection(text, anchor, caret);
    let at = floor(&text.get(), caret.get());
    text.update(|t| t.insert_str(at, inserted));
    anchor.set(at + inserted.len());
    caret.set(at + inserted.len());
}

/// What a key does to the text and the selection.
fn edit_key(
    e: &KeyEvent,
    text: Signal<String>,
    anchor: Signal<usize>,
    caret: Signal<usize>,
) -> Handled {
    let shift = e.modifiers.shift;
    let command = e.modifiers.ctrl || e.modifiers.meta || e.modifiers.alt;
    let current = text.get();
    let at = floor(&current, caret.get());
    let selected = span(anchor, caret);
    let paragraph = shape_paragraph(&current, &style_of(), None);
    let target = match e.key {
        Key::ArrowLeft if !shift && !selected.is_empty() => selected.start,
        Key::ArrowRight if !shift && !selected.is_empty() => selected.end,
        Key::ArrowLeft => paragraph.prev_boundary(at),
        Key::ArrowRight => paragraph.next_boundary(at),
        Key::Home => 0,
        Key::End => current.len(),
        Key::Char('a') if command => {
            anchor.set(0);
            caret.set(current.len());
            return Handled::Yes;
        }
        Key::Backspace => {
            if !delete_selection(text, anchor, caret) && at > 0 {
                let from = paragraph.prev_boundary(at);
                text.update(|t| t.replace_range(from..at, ""));
                anchor.set(from);
                caret.set(from);
            }
            return Handled::Yes;
        }
        Key::Delete => {
            if !delete_selection(text, anchor, caret) && at < current.len() {
                let to = paragraph.next_boundary(at);
                text.update(|t| t.replace_range(at..to, ""));
            }
            return Handled::Yes;
        }
        _ => {
            return match e.text.as_deref() {
                Some(typed) if !command && !typed.chars().any(char::is_control) => {
                    insert(text, anchor, caret, typed);
                    Handled::Yes
                }
                _ => Handled::No,
            };
        }
    };
    caret.set(target);
    if !shift {
        anchor.set(target);
    }
    Handled::Yes
}
