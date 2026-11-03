//! Role: Source-preserving Markdown editing and a formatted read-only preview.
use crate::markdown::markdown;
use kivact::control::{Node, NodeBuilder, column, node, row};
use kivact::i18n::message;
use kivact::primitive::{NodeRefExt, TextSession};
use kivact::style::{Align, style};
use kivact::ui::{ButtonVariant, ControlSize, button, text_area};
use kivact::view::{IntoText, IntoView, NodeRef, Role, View, dynamic, node_ref};
use kivact::{Binding, Signal, Value, on_mount, signal};

/// The original Markdown source is authoritative, including line endings and whitespace.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RichDoc {
    markdown: String,
}

impl RichDoc {
    pub fn from_markdown(markdown: impl Into<String>) -> Self {
        Self {
            markdown: markdown.into(),
        }
    }

    pub fn markdown(&self) -> &str {
        &self.markdown
    }

    pub fn into_markdown(self) -> String {
        self.markdown
    }
}

/// Render the supported Markdown blocks and inline marks.
pub fn rich_text_view(source: impl IntoText) -> Node {
    node().child(markdown(source.into_text_value()))
}

/// A toolbar for switching the Markdown editor between source and preview.
pub fn rich_text_toolbar(preview: impl Into<Binding<bool>>) -> Node {
    toolbar(preview.into(), None, None)
}

fn toolbar(
    preview: Binding<bool>,
    editor: Option<(Signal<Option<NodeRef>>, Signal<bool>)>,
    blocks: Option<Signal<bool>>,
) -> Node {
    let mode = |label, selected, block_mode| {
        button(label)
            .variant(ButtonVariant::Ghost)
            .size(ControlSize::Small)
            .selected(move || {
                preview.get() == selected
                    && (selected || blocks.is_none_or(|blocks| blocks.get() == block_mode))
            })
            .on_press(move || {
                let changed = preview.get_untracked() != selected
                    || (!selected
                        && blocks.is_some_and(|blocks| blocks.get_untracked() != block_mode));
                if changed && let Some((target, composing)) = editor {
                    composing.set(false);
                    target.set(None);
                }
                if !selected && let Some(blocks) = blocks {
                    blocks.set(block_mode);
                }
                preview.set(selected);
            })
    };
    let commands = editor.map(|(editor, composing)| {
        let command = |label, marker| {
            button(label)
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Small)
                .disabled(move || preview.get() || composing.get() || editor.get().is_none())
                .on_press(move || {
                    if let Some(editor) = editor.get_untracked() {
                        format_selection(editor, marker);
                    }
                })
        };
        row((
            command(message("soft.rich_text.bold", "Bold"), "**"),
            command(message("soft.rich_text.italic", "Italic"), "*"),
            command(
                message("soft.rich_text.strikethrough", "Strikethrough"),
                "~~",
            ),
            command(message("soft.rich_text.inline_code", "Inline code"), "`"),
        ))
        .style(style().align_items(Align::Center).gap(4))
    });
    row((
        commands,
        blocks.map(|_| mode(message("soft.rich_text.blocks", "Blocks"), false, true)),
        mode(message("soft.rich_text.write", "Write"), false, false),
        mode(message("soft.rich_text.preview", "Preview"), true, false),
    ))
    .role(Role::Toolbar)
    .roving_focus(kivact::view::Orientation::Horizontal)
    .name(message("soft.rich_text.toolbar", "Editor toolbar"))
    .style(style().align_items(Align::Center).gap(4))
}

fn format_selection(editor: NodeRef, marker: &'static str) {
    let Some(mut edit) = editor.edit_state() else {
        return;
    };
    if edit.is_composing() {
        return;
    }

    let reversed = edit.anchor() > edit.caret();
    let range = edit.selection();
    let selected = edit.selected_text();
    let length = marker.len();
    let before = &edit.value()[..range.start];
    let after = &edit.value()[range.end..];

    // Remove markers immediately surrounding the selection. Checking the whole
    // delimiter run keeps Italic from stripping one star from a Bold mark.
    let marker_byte = marker.as_bytes()[0];
    let left = before
        .bytes()
        .rev()
        .take_while(|&byte| byte == marker_byte)
        .count();
    let right = after
        .bytes()
        .take_while(|&byte| byte == marker_byte)
        .count();
    let (replace, replacement, start, end) =
        if left == length && right == length || marker_byte == b'*' && left == 3 && right == 3 {
            (
                range.start - length..range.end + length,
                selected.to_owned(),
                range.start - length,
                range.end - length,
            )
        } else if let Some(inner) = selected
            .strip_prefix(marker)
            .and_then(|text| text.strip_suffix(marker))
            && !inner.starts_with(marker_byte as char)
            && !inner.ends_with(marker_byte as char)
        {
            (
                range.clone(),
                inner.to_owned(),
                range.start,
                range.start + inner.len(),
            )
        } else {
            // Keep boundary whitespace outside emphasis delimiters. The adjusted
            // offsets are UTF-8 boundaries supplied by trim and the native editor.
            let inner = selected.trim();
            let (leading, trailing) = if inner.is_empty() {
                ("", selected)
            } else {
                let leading = selected.len() - selected.trim_start().len();
                (&selected[..leading], &selected[leading + inner.len()..])
            };
            let replacement = format!("{leading}{marker}{inner}{marker}{trailing}");
            let start = range.start + leading.len() + length;
            (range, replacement, start, start + inner.len())
        };
    edit.replace(replace, &replacement);
    edit.set_session(if reversed {
        TextSession::select(end, start)
    } else {
        TextSession::select(start, end)
    });
    editor.set_edit_state(edit);
    editor.focus();
}

pub struct RichTextEditor {
    root: Node,
    label: Value<String>,
    source: Binding<String>,
    read_only: bool,
    block_editing: bool,
}

pub fn rich_text_editor(
    label: impl IntoText,
    source: impl Into<Binding<String>>,
) -> RichTextEditor {
    RichTextEditor {
        root: node(),
        label: label.into_text_value(),
        source: source.into(),
        read_only: false,
        block_editing: false,
    }
}

impl RichTextEditor {
    /// Start in the wrapped block editor and offer Write and Preview modes.
    /// The Markdown binding remains authoritative in every mode.
    pub fn block_editing(mut self, enabled: bool) -> Self {
        self.block_editing = enabled;
        self
    }

    pub fn read_only(mut self, value: bool) -> Self {
        self.read_only = value;
        self
    }
}

impl NodeBuilder for RichTextEditor {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for RichTextEditor {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            source,
            read_only,
            block_editing,
        } = self;
        let previewing = signal(read_only);
        let target = signal(None::<NodeRef>);
        let blocks = block_editing.then(|| signal(true));
        let composing = signal(false);
        let surface = dynamic(
            move || (previewing.get(), blocks.is_some_and(|blocks| blocks.get())),
            move |(previewing, blocks)| {
                if previewing {
                    rich_text_view(move || source.get())
                        .style(style().padding(12))
                        .into_view()
                } else if blocks {
                    crate::rich_blocks::edit_blocks(label, source, target, composing).into_view()
                } else {
                    let editor = node_ref();
                    on_mount(move || target.set(Some(editor)));
                    text_area(label, source)
                        .node_ref(editor)
                        .on_composition_change(move |active| composing.set(active))
                        .wrap(true)
                        .rows(6..=16)
                        .into_view()
                }
            },
        );
        let toolbar =
            (!read_only).then(|| toolbar(previewing.into(), Some((target, composing)), blocks));
        root.child(column((toolbar, surface)).style(style().gap(4)))
            .into_view()
    }
}

#[cfg(test)]
mod tests {
    //! Role: Verify the editor preserves Markdown source.
    use super::RichDoc;

    #[test]
    fn preserves_source_bytes_even_for_unsupported_syntax() {
        let source = "## Title  \r\n\r\n| A | B |\r\n|---|---|\r\n";
        let document = RichDoc::from_markdown(source);
        assert_eq!(document.markdown(), source);
        assert_eq!(document.into_markdown(), source);
    }
}
