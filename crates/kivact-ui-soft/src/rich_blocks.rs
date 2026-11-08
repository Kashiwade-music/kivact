//! Role: Edit source-preserving Markdown blocks using native wrapped text fields.
use crate::markdown;
use crate::rich_block_doc::{Block, Document, Kind};
use kivact::custom::EditContext;
use kivact::custom::EditLineLayout;
use kivact::custom::EditOutcome;
use kivact::custom::EditRequest;
use kivact::custom::EditState;
use kivact::custom::NodeRefExt;
use kivact::custom::TextEditing;
use kivact::custom::TextSession;
use kivact::i18n::{message, message_with};
use kivact::interaction::text_edit_policy;
use kivact::state::Binding;
use kivact::state::Machine;
use kivact::state::MachineHandle;
use kivact::state::Step;
use kivact::state::Value;
use kivact::state::Var;
use kivact::state::binding;
use kivact::state::machine;
use kivact::state::memo;
use kivact::state::on_cleanup;
use kivact::state::on_mount;
use kivact::state::var;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::Style;
use kivact::style::StyledRange;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::button;
use kivact::ui::checkbox;
use kivact::view::Handled;
use kivact::view::IntoView;
use kivact::view::Key;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::NodeRef;
use kivact::view::PointerPhase;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::column;
use kivact::view::for_each;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::text;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub(super) struct Editor {
    source: Binding<String>,
    document: Value<Document>,
    state: MachineHandle<BlockMachine>,
    references: Var<HashMap<usize, NodeRef>>,
}

/// Create the block-editing owner once per editor; it outlives switches to Write and Preview.
pub(super) fn block_editor(source: Binding<String>) -> Editor {
    let references = var(HashMap::<usize, NodeRef>::new());
    let state = machine(
        BlockSnapshot {
            document: Document::parse(&source.get_untracked(), 0),
            phase: BlockEditPhase::Idle,
        },
        move || BlockMachine {
            source: source.get_untracked(),
            references: references.with(Clone::clone),
        },
        move |_, markdown: String| {
            if source.get_untracked() != markdown {
                source.set(markdown);
            }
        },
    );
    state.feed(move || source.get(), |_| BlockEvent::Source);
    let document = memo(move || state.get().document);
    Editor {
        source,
        document,
        state,
        references,
    }
}

/// Render the editable blocks of an existing block-editing owner.
pub(super) fn edit_blocks(label: Value<String>, editor: Editor) -> Node {
    let document = editor.document;
    let blocks = for_each(
        move || document.with(|doc| doc.blocks.clone()),
        |block| block.id,
        move |block| editor.block(label, block.with_untracked(|block| block.id), block),
    );
    column(blocks)
        .with_node(move |node| node.role(Role::Group))
        .name(label)
        .style(style().gap(8).padding(12).width(pct(100)).min_width(0))
}

#[derive(Clone, Copy)]
enum Command {
    Split,
    Merge,
    Move { forward: bool, x: f64 },
    SelectAll,
    CopyAll,
    Clear,
}

impl Editor {
    /// The block field the toolbar formats, read from this owner's phase.
    pub(super) fn target(self) -> crate::rich_text::EditTarget {
        match self.state.get().phase {
            BlockEditPhase::Focused {
                target,
                composition: BlockComposition::Composing,
                ..
            } => crate::rich_text::EditTarget::Composing(target),
            BlockEditPhase::Focused { target, .. } => crate::rich_text::EditTarget::Ready(target),
            BlockEditPhase::Idle | BlockEditPhase::AwaitingMount { .. } => {
                crate::rich_text::EditTarget::Detached
            }
        }
    }

    fn whole(self) -> bool {
        matches!(
            self.state.get().phase,
            BlockEditPhase::Focused {
                selection: BlockSelection::Whole,
                ..
            }
        )
    }

    fn change<T>(self, change: impl FnOnce(&mut Document) -> T) -> T {
        self.state.send(BlockEvent::Source);
        let mut document = self.document.get_untracked();
        let result = change(&mut document);
        self.state.send(BlockEvent::Document {
            document,
            await_mount: None,
        });
        result
    }

    fn block(self, label: Value<String>, id: usize, block: Value<Block>) -> View {
        let kind = block.with(|block| block.kind.clone());
        if !kind.editable() {
            return node()
                .child(markdown(move || block.with(|block| block.body.clone())))
                .style(move || selection_style(self.whole()))
                .into_view();
        }
        let reference = self.block_reference(id);
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
        let editing = self.block_editing(id, block, kind.prose(), command.clone());
        let style_kind = kind.clone();
        let field = node()
            .node_ref(reference)
            .with_node(move |node| node.role(Role::TextBox))
            .focusable(true)
            .name(name)
            .text_editing(editing)
            .style(move || editor_style(&style_kind, self.whole()))
            .on_pointer(move |event| {
                if matches!(event.phase(), PointerPhase::Down(_)) {
                    self.state
                        .send(BlockEvent::Selection(BlockSelection::Caret));
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
        self.decorate_block(id, block, kind, field)
    }

    fn block_reference(self, id: usize) -> NodeRef {
        let reference = node_ref();
        self.references
            .update(|references| references.insert(id, reference));
        on_cleanup(move || {
            self.references.update(|references| references.remove(&id));
            self.state.send(BlockEvent::Detach(reference));
        });
        let focused = reference.focused();
        self.state.feed(focused, move |focused| {
            if focused {
                BlockEvent::Focus(reference)
            } else {
                BlockEvent::Blur
            }
        });
        on_mount(move || {
            if self.state.get().phase == (BlockEditPhase::AwaitingMount { block: id }) {
                self.state.send(BlockEvent::Focus(reference));
                reference.focus();
                reference.set_edit_session(TextSession::caret_at(0));
            }
        });
        reference
    }

    fn block_editing(
        self,
        id: usize,
        block: Value<Block>,
        prose: bool,
        command: Rc<Cell<Option<Command>>>,
    ) -> TextEditing {
        let native_command = command;
        TextEditing::new(move || block.with(|block| block.body.clone()))
            .multiline()
            .line_layout(|| EditLineLayout {
                wrap: true,
                rows: Some((1, 24)),
            })
            .report_text()
            .behavior(move |state, request, context| {
                let action = key_command(state, &request, context, prose, self.whole());
                native_command.set(action);
                if action.is_some() {
                    EditOutcome::NotEditing
                } else {
                    text_edit_policy(state, request, context)
                }
            })
            .line_styles(
                move || {
                    kivact::style::use_tokens::<crate::Tokens>()
                        .colors
                        .text_secondary
                },
                move |color, line, &()| {
                    let marks = if prose {
                        line.char_indices()
                            .filter_map(|(at, character)| {
                                matches!(character, '*' | '_' | '`' | '~' | '[' | ']').then_some(
                                    StyledRange {
                                        range: at..at + 1,
                                        color: Some(*color),
                                        background: None,
                                        decoration: None,
                                    },
                                )
                            })
                            .collect()
                    } else {
                        Vec::new()
                    };
                    (marks, ())
                },
            )
            .on_text_change(move |event| {
                let composing = event.composition().is_some();
                self.state.send(BlockEvent::Composition(if composing {
                    BlockComposition::Composing
                } else {
                    BlockComposition::Idle
                }));
                if !composing && let Some(text) = event.text {
                    self.change(|doc| doc.replace(id, text));
                }
            })
            .on_selection_change(move |_| {
                self.state
                    .send(BlockEvent::Selection(BlockSelection::Caret))
            })
    }

    fn decorate_block(self, id: usize, block: Value<Block>, kind: Kind, field: Node) -> View {
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
                        .background(kivact::style::use_tokens::<crate::Tokens>().colors.bg_muted)
                })
                .into_view()
            }
            _ => field.into_view(),
        }
    }

    fn run(self, id: usize, reference: NodeRef, command: Command) {
        match command {
            Command::Split => {
                if let Some(edit) = reference.edit_state() {
                    self.state.send(BlockEvent::Source);
                    let mut document = self.document.get_untracked();
                    let await_mount = document.split(id, edit.selection());
                    self.state.send(BlockEvent::Document {
                        document,
                        await_mount,
                    });
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
            Command::SelectAll => self
                .state
                .send(BlockEvent::Selection(BlockSelection::Whole)),
            Command::CopyAll => copy_text(self.source.get_untracked()),
            Command::Clear => self
                .state
                .send(BlockEvent::Selection(BlockSelection::Caret)),
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

fn move_to(reference: NodeRef, forward: bool, x: f64) {
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
        style().background(
            kivact::style::use_tokens::<crate::Tokens>()
                .colors
                .accent_muted,
        )
    } else {
        style()
    }
}

fn editor_style(kind: &Kind, whole: bool) -> Style {
    let p = kivact::style::use_tokens::<crate::Tokens>();
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

#[derive(Clone, PartialEq)]
enum BlockSelection {
    Caret,
    Whole,
}
#[derive(Clone, PartialEq)]
enum BlockComposition {
    Idle,
    Composing,
}
#[derive(Clone, PartialEq)]
enum BlockEditPhase {
    Idle,
    Focused {
        target: NodeRef,
        selection: BlockSelection,
        composition: BlockComposition,
    },
    AwaitingMount {
        block: usize,
    },
}
#[derive(Clone, PartialEq)]
struct BlockSnapshot {
    document: Document,
    phase: BlockEditPhase,
}
enum BlockEvent {
    Source,
    Document {
        document: Document,
        await_mount: Option<usize>,
    },
    Focus(NodeRef),
    Blur,
    Detach(NodeRef),
    Selection(BlockSelection),
    Composition(BlockComposition),
}
struct BlockMachine {
    source: String,
    references: HashMap<usize, NodeRef>,
}
impl Machine for BlockMachine {
    type State = BlockSnapshot;
    type Event = BlockEvent;
    type Effect = String;
    fn transition(
        &self,
        state: &BlockSnapshot,
        event: BlockEvent,
    ) -> Step<BlockSnapshot, BlockEvent, String> {
        let mut next = state.clone();
        match event {
            BlockEvent::Source => {
                if next.document.markdown() == self.source {
                    return Step::stay();
                }
                next.document = Document::parse(&self.source, next.document.next_id());
                next.phase = BlockEditPhase::Idle;
            }
            BlockEvent::Document {
                document,
                await_mount,
            } => {
                next.document = document;
                if let Some(block) = await_mount {
                    next.phase = BlockEditPhase::AwaitingMount { block };
                } else {
                    let removed = match &next.phase {
                        BlockEditPhase::Focused { target, .. } => self
                            .references
                            .iter()
                            .find_map(|(id, reference)| (reference == target).then_some(*id))
                            .is_some_and(|id| {
                                !next.document.blocks.iter().any(|block| block.id == id)
                            }),
                        BlockEditPhase::AwaitingMount { block } => !next
                            .document
                            .blocks
                            .iter()
                            .any(|candidate| candidate.id == *block),
                        BlockEditPhase::Idle => false,
                    };
                    if removed {
                        next.phase = BlockEditPhase::Idle;
                    }
                }
            }
            BlockEvent::Focus(target) => {
                let selection = match next.phase {
                    BlockEditPhase::Focused { selection, .. } => selection,
                    BlockEditPhase::Idle | BlockEditPhase::AwaitingMount { .. } => {
                        BlockSelection::Caret
                    }
                };
                next.phase = BlockEditPhase::Focused {
                    target,
                    selection,
                    composition: BlockComposition::Idle,
                };
            }
            BlockEvent::Blur => return Step::stay(),
            BlockEvent::Detach(target) => {
                if matches!(next.phase, BlockEditPhase::Focused { target: focused, .. } if focused == target)
                {
                    next.phase = BlockEditPhase::Idle;
                }
            }
            BlockEvent::Selection(selection) => {
                if let BlockEditPhase::Focused {
                    selection: current, ..
                } = &mut next.phase
                {
                    *current = selection;
                }
            }
            BlockEvent::Composition(composition) => {
                if let BlockEditPhase::Focused {
                    selection,
                    composition: current,
                    ..
                } = &mut next.phase
                {
                    *selection = BlockSelection::Caret;
                    *current = composition;
                }
            }
        }
        let markdown = next.document.markdown();
        Step::to(next).with(markdown)
    }
}
