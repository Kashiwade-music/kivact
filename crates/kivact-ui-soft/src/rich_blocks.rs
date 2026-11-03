//! Role: Edit source-preserving Markdown blocks using native wrapped text fields.
use crate::rich_block_doc::{Block, Document, Kind};
use crate::{markdown, tokens};
use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::i18n::{message, message_with};
use kivact::interaction::text_edit_policy;
use kivact::primitive::{
    EditContext, EditLineLayout, EditOutcome, EditRequest, EditState, NodeRefExt, TextEditing,
    TextHighlight, TextSession, part,
};
use kivact::style::{Align, Justify, Style, pct, style};
use kivact::ui::{ButtonVariant, ControlSize, button, checkbox};
use kivact::view::{Handled, IntoView, Key, NodeRef, PointerPhase, Role, View, for_each, node_ref};
use kivact::{Binding, Signal, Value, Var, binding, effect, on_cleanup, on_mount, signal, var};
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy)]
struct Editor {
    source: Binding<String>,
    document: Signal<Document>,
    references: Var<HashMap<usize, NodeRef>>,
    focus_after_mount: Var<Option<usize>>,
    target: Signal<Option<NodeRef>>,
    composing: Signal<bool>,
    whole: Signal<bool>,
}

pub(super) fn edit_blocks(
    label: Value<String>,
    source: Binding<String>,
    target: Signal<Option<NodeRef>>,
    composing: Signal<bool>,
) -> Node {
    let document = signal(Document::parse(&source.get_untracked(), 0));
    let editor = Editor {
        source,
        document,
        references: var(HashMap::new()),
        focus_after_mount: var(None),
        target,
        composing,
        whole: signal(false),
    };
    effect(move || {
        let markdown = source.get();
        if document.with(|doc| doc.markdown() != markdown) {
            let first = document.with_untracked(Document::next_id);
            document.set(Document::parse(&markdown, first));
            editor.whole.set(false);
        }
    });
    let blocks = for_each(
        move || document.with(|doc| doc.blocks.clone()),
        |block| block.id,
        move |id, block| editor.block(label, id, block),
    );
    column(blocks)
        .role(Role::Group)
        .name(label)
        .style(style().gap(8).padding(12).width(pct(100)).min_width(0))
}

#[derive(Clone, Copy)]
enum Command {
    Split,
    Merge,
    Move { forward: bool, x: f32 },
    SelectAll,
    CopyAll,
    Clear,
}

impl Editor {
    fn change<T>(self, change: impl FnOnce(&mut Document) -> T) -> T {
        let (result, source) = self.document.update(|document| {
            let result = change(document);
            (result, document.markdown())
        });
        if self.source.get_untracked() != source {
            self.source.set(source);
        }
        result
    }

    fn block(self, label: Value<String>, id: usize, block: Value<Block>) -> View {
        let kind = block.with(|block| block.kind.clone());
        if !kind.editable() {
            return node()
                .child(markdown(move || block.with(|block| block.body.clone())))
                .style(move || selection_style(self.whole.get()))
                .into_view();
        }
        let reference = node_ref();
        self.references
            .update(|references| references.insert(id, reference));
        on_cleanup(move || {
            self.references.update(|references| references.remove(&id));
            if self.target.get_untracked() == Some(reference) {
                self.target.set(None);
            }
        });
        let focused = reference.focused();
        effect(move || {
            if focused.get() {
                self.target.set(Some(reference));
                self.composing.set(false);
            }
        });
        on_mount(move || {
            if self.focus_after_mount.get() == Some(id) {
                self.focus_after_mount.set(None);
                reference.focus();
                reference.set_edit_session(TextSession::caret_at(0));
            }
        });
        let name = message_with(
            "soft.rich_text.block",
            "{label}, block {index}",
            move || {
                let index = self
                    .document
                    .with(|doc| doc.blocks.iter().position(|block| block.id == id))
                    .unwrap_or_default()
                    + 1;
                [("label", label.get()), ("index", index.to_string())]
            },
        );
        let command = Rc::new(Cell::new(None));
        let native_command = command.clone();
        let prose = kind.prose();
        let editing =
            TextEditing::new(move || block.with(|block| Rc::<str>::from(block.body.as_str())))
                .multiline(true)
                .line_layout(|| EditLineLayout {
                    wrap: true,
                    rows: Some((1, 24)),
                })
                .report_text(true)
                .behavior(move |state, request, context| {
                    let action =
                        key_command(state, &request, context, prose, self.whole.get_untracked());
                    native_command.set(action);
                    if action.is_some() {
                        EditOutcome::NotEditing
                    } else {
                        text_edit_policy(state, request, context)
                    }
                })
                .highlight(
                    move || tokens().colors.text_secondary,
                    move |color, line, &()| {
                        let marks = if prose {
                            line.char_indices()
                                .filter_map(|(at, character)| {
                                    matches!(character, '*' | '_' | '`' | '~' | '[' | ']')
                                        .then_some(TextHighlight {
                                            range: at..at + 1,
                                            color: *color,
                                        })
                                })
                                .collect()
                        } else {
                            Vec::new()
                        };
                        (marks, ())
                    },
                )
                .on_edit(move |event| {
                    let composing = event.composition().is_some();
                    self.composing.set(composing);
                    self.whole.set(false);
                    if !composing && let Some(text) = event.text {
                        self.change(|doc| doc.replace(id, text));
                    }
                })
                .on_selection(move |_| self.whole.set(false));
        let style_kind = kind.clone();
        let field = node()
            .node_ref(reference)
            .part(
                part(|input| input)
                    .role(Role::TextBox)
                    .focusable(true)
                    .name(name)
                    .text_editing(editing),
            )
            .style(move || editor_style(&style_kind, self.whole.get()))
            .on_pointer(move |event| {
                if matches!(event.phase, PointerPhase::Down(_)) {
                    self.whole.set(false);
                }
                Handled::No
            })
            .on_key(move |_| {
                let Some(command) = command.take() else {
                    return Handled::No;
                };
                self.run(id, reference, command);
                Handled::Yes
            });
        match kind {
            Kind::List { number, checked } => {
                let marker = if checked.is_some() {
                    let checked = binding(
                        move || {
                            block.with(|block| {
                                matches!(
                                    block.kind,
                                    Kind::List {
                                        checked: Some(true),
                                        ..
                                    }
                                )
                            })
                        },
                        move |checked| self.change(|doc| doc.task(id, checked)),
                    );
                    checkbox(message("soft.rich_text.task", "Task completed"), checked).into_view()
                } else {
                    text(number.map_or_else(|| "•".into(), |number| format!("{number}.")))
                        .style(style().width(24).shrink(0))
                        .into_view()
                };
                row((marker, field))
                    .style(style().gap(4).align_items(Align::Start))
                    .into_view()
            }
            Kind::Code(language) => {
                let copy = button(message("soft.rich_text.copy_code", "Copy code"))
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Small)
                    .on_press(move || copy_text(block.with(|block| block.body.clone())));
                column((
                    row((text(language), copy)).style(
                        style()
                            .align_items(Align::Center)
                            .justify_content(Justify::SpaceBetween),
                    ),
                    field,
                ))
                .style(|| {
                    style()
                        .padding(8)
                        .radius(6)
                        .background(tokens().colors.bg_muted)
                })
                .into_view()
            }
            _ => field.into_view(),
        }
    }

    fn run(self, id: usize, reference: NodeRef, command: Command) {
        match command {
            Command::Split => {
                if let Some(edit) = reference.edit_state()
                    && let Some(next) = self.change(|doc| doc.split(id, edit.selection()))
                {
                    self.focus_after_mount.set(Some(next));
                }
            }
            Command::Merge => {
                if let Some((target, caret)) = self.change(|doc| doc.merge(id))
                    && let Some(reference) = self.reference(target)
                {
                    reference.focus();
                    reference.set_edit_session(TextSession::caret_at(caret));
                }
            }
            Command::Move { forward, x } => {
                let next = self.document.with(|doc| doc.neighbor(id, forward));
                if let Some(reference) = next.and_then(|id| self.reference(id)) {
                    move_to(reference, forward, x);
                }
            }
            Command::SelectAll => self.whole.set(true),
            Command::CopyAll => copy_text(self.source.get_untracked()),
            Command::Clear => self.whole.set(false),
        }
    }

    fn reference(self, id: usize) -> Option<NodeRef> {
        self.references
            .with(|references| references.get(&id).copied())
    }
}

fn key_command(
    state: &EditState,
    request: &EditRequest<'_>,
    context: &EditContext<'_>,
    prose: bool,
    whole: bool,
) -> Option<Command> {
    let EditRequest::Key { key, modifiers } = request else {
        return None;
    };
    if state.is_composing() {
        return None;
    }
    if (modifiers.ctrl || modifiers.meta) && !modifiers.alt && !modifiers.shift {
        match key {
            Key::Char('a' | 'A') => return Some(Command::SelectAll),
            Key::Char('c' | 'C') if whole => return Some(Command::CopyAll),
            _ => {}
        }
    }
    if modifiers.ctrl || modifiers.alt || modifiers.meta || modifiers.shift {
        return None;
    }
    let caret = state.caret();
    match key {
        Key::Escape if whole => Some(Command::Clear),
        Key::Enter if prose => Some(Command::Split),
        Key::Backspace if prose && state.selection() == (0..0) => Some(Command::Merge),
        Key::ArrowUp | Key::ArrowDown if state.selection().is_empty() => {
            let forward = *key == Key::ArrowDown;
            let line = context.line_of(state, caret).unwrap_or(0);
            let count = context.line_count(state).unwrap_or(1);
            if (!forward && line == 0) || (forward && line + 1 >= count) {
                Some(Command::Move {
                    forward,
                    x: context.x_of(state, caret).unwrap_or(0.0),
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn move_to(reference: NodeRef, forward: bool, x: f32) {
    reference.focus();
    let Some(edit) = reference.edit_state() else {
        return;
    };
    let caret = reference
        .text_layout(0..1)
        .and_then(|first| {
            let line = if forward {
                0
            } else {
                first.line_count.checked_sub(1)?
            };
            let layout = if line == 0 {
                first
            } else {
                reference.text_layout(line..line + 1)?
            };
            let desired = layout.lines.first()?.rect.x + x;
            let offset = layout
                .carets
                .iter()
                .min_by(|a, b| {
                    (a.rect.x - desired)
                        .abs()
                        .total_cmp(&(b.rect.x - desired).abs())
                })?
                .offset as usize;
            Some(
                edit.value()
                    .char_indices()
                    .nth(offset)
                    .map_or(edit.value().len(), |(at, _)| at),
            )
        })
        .unwrap_or_else(|| if forward { 0 } else { edit.value().len() });
    reference.set_edit_session(TextSession::caret_at(caret));
}

fn selection_style(whole: bool) -> Style {
    if whole {
        style().background(tokens().colors.accent_muted)
    } else {
        style()
    }
}

fn editor_style(kind: &Kind, whole: bool) -> Style {
    let p = tokens();
    let base = selection_style(whole)
        .grow(1)
        .min_width(0)
        .padding(2)
        .focus_visible(crate::recipes_dialog::focus(&p));
    match kind {
        Kind::Heading(level) => {
            let heading = match level {
                1 => p.typography.heading1,
                2 => p.typography.heading2,
                3 => p.typography.heading3,
                4 => p.typography.heading4,
                5 => p.typography.heading5,
                _ => p.typography.heading6,
            };
            base.font_size(heading.size_px)
                .line_height(heading.line_height_px / heading.size_px)
                .weight(600)
        }
        Kind::Quote => base
            .padding_left(8)
            .border_left(2, p.colors.border)
            .color(p.colors.text_secondary),
        Kind::Code(_) => base.font_family(p.typography.family_code),
        _ => base,
    }
}

fn copy_text(text: String) {
    kivact::task::spawn(async move { kivact::platform::copy_text(text).await });
}
