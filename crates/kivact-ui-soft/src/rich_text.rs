//! Role: Source-preserving Markdown editing and a formatted read-only preview.
use crate::markdown::markdown;
pub(crate) use crate::rich_block_doc::EditTarget;
use kivact::custom::NodeRefExt;
use kivact::custom::TextSession;
use kivact::i18n::message;
use kivact::state::Binding;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::memo;
use kivact::state::on_cleanup;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::button;
use kivact::ui::text_area;
use kivact::view::Element;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeRef;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::column;
use kivact::view::dynamic;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::row;

/// The original Markdown source is authoritative, including line endings and whitespace.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RichDoc {
    markdown: String,
}

impl RichDoc {
    /// Store Markdown source as an application-owned rich document.
    pub fn from_markdown(markdown: impl Into<String>) -> Self {
        Self {
            markdown: markdown.into(),
        }
    }

    /// Borrow the document Markdown source.
    pub fn markdown(&self) -> &str {
        &self.markdown
    }

    /// Consume the document and return its Markdown source.
    pub fn into_markdown(self) -> String {
        self.markdown
    }
}

#[derive(Clone, Copy, PartialEq)]
enum EditorMode {
    Preview,
    Write {
        /// Whether the write text area is composing text.
        composing: bool,
    },
    Blocks,
}

impl EditorMode {
    fn surface(self) -> Self {
        match self {
            Self::Write { .. } => Self::Write { composing: false },
            Self::Preview => Self::Preview,
            Self::Blocks => Self::Blocks,
        }
    }
}

/// Render the supported Markdown blocks and inline marks.
pub fn rich_text_view(source: impl IntoText) -> impl kivact::view::Element {
    node().child(markdown(source.into_text_value()))
}

/// A toolbar for switching the Markdown editor between source and preview.
pub fn rich_text_toolbar(preview: impl IntoBinding<bool>) -> impl kivact::view::Element {
    let preview = preview.into_binding();
    let mode = binding(
        move || {
            if preview.get() {
                EditorMode::Preview
            } else {
                EditorMode::Write { composing: false }
            }
        },
        move |mode| preview.set(mode == EditorMode::Preview),
    );
    toolbar(mode, None, false)
}

fn toolbar(
    mode: Binding<EditorMode>,
    target: Option<Value<EditTarget>>,
    offers_blocks: bool,
) -> Node {
    let mode_button = |label, value: EditorMode| {
        button(label)
            .variant(ButtonVariant::Ghost)
            .size(ControlSize::Small)
            .with_node(move |node| node.selected(move || mode.get().surface() == value))
            .on_press(move || mode.set(value))
    };
    let commands = target.map(|target| {
        let command = |label, marker| {
            button(label)
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Small)
                .disabled(move || !matches!(target.get(), EditTarget::Ready(_)))
                .on_press(move || {
                    if let EditTarget::Ready(editor) = target.get_untracked() {
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
        offers_blocks.then(|| {
            mode_button(
                message("soft.rich_text.blocks", "Blocks"),
                EditorMode::Blocks,
            )
        }),
        mode_button(
            message("soft.rich_text.write", "Write"),
            EditorMode::Write { composing: false },
        ),
        mode_button(
            message("soft.rich_text.preview", "Preview"),
            EditorMode::Preview,
        ),
    ))
    .with_node(move |node| node.role(Role::Toolbar))
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

/// A bound Markdown editor with formatting controls and an optional block-editing surface.
pub struct RichTextEditor {
    root: Node,
    label: Value<String>,
    source: Binding<String>,
    read_only: Value<bool>,
    block_editing: Value<bool>,
}

/// Build a labelled editor that writes Markdown through the supplied binding.
pub fn rich_text_editor(label: impl IntoText, source: impl IntoBinding<String>) -> RichTextEditor {
    RichTextEditor {
        root: node(),
        label: label.into_text_value(),
        source: source.into_binding(),
        read_only: (false).into_value(),
        block_editing: (false).into_value(),
    }
}

impl RichTextEditor {
    /// Start in the wrapped block editor and offer Write and Preview modes.
    /// The Markdown binding remains authoritative in every mode.
    pub fn block_editing(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.block_editing = enabled.into_value();
        self
    }

    /// Prevent edits while retaining the document display and native selection.
    pub fn read_only(mut self, value: impl IntoValue<bool>) -> Self {
        self.read_only = value.into_value();
        self
    }
}

impl Element for RichTextEditor {
    #[doc(hidden)]
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
        let read_only = read_only.get();
        let offers_blocks = !read_only && block_editing.get();
        let mode = signal(if read_only {
            EditorMode::Preview
        } else if offers_blocks {
            EditorMode::Blocks
        } else {
            EditorMode::Write { composing: false }
        });
        let blocks = offers_blocks.then(|| crate::rich_blocks::block_editor(source));
        let write_ref = node_ref();
        let surface = dynamic(
            move || mode.get().surface(),
            move |current| match current {
                EditorMode::Preview => rich_text_view(move || source.get())
                    .style(style().padding(12))
                    .into_view(),
                EditorMode::Blocks => blocks
                    .map(|blocks| crate::rich_blocks::edit_blocks(label, blocks))
                    .into_view(),
                EditorMode::Write { .. } => {
                    on_cleanup(move || {
                        mode.update(|mode| {
                            if let EditorMode::Write { composing } = mode {
                                *composing = false;
                            }
                        })
                    });
                    text_area(label, source)
                        .node_ref(write_ref)
                        .on_composition_change(move |active| {
                            mode.update(|mode| {
                                if let EditorMode::Write { composing } = mode {
                                    *composing = active == kivact::view::Composition::Composing;
                                }
                            })
                        })
                        .wrap(true)
                        .rows(6..=16)
                        .into_view()
                }
            },
        );
        let target = memo(move || match mode.get() {
            EditorMode::Preview => EditTarget::Detached,
            EditorMode::Write { composing: true } => EditTarget::Composing(write_ref),
            EditorMode::Write { composing: false } => EditTarget::Ready(write_ref),
            EditorMode::Blocks => blocks.map_or(EditTarget::Detached, |blocks| blocks.target()),
        });
        let toolbar =
            (!read_only).then(|| toolbar(mode.into_binding(), Some(target), offers_blocks));
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
