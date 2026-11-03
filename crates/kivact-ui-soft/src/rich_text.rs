//! Role: Source-preserving Markdown editing and a formatted read-only preview.
use crate::markdown::markdown;
use kivact::control::{Node, NodeBuilder, column, node, row};
use kivact::i18n::message;
use kivact::style::{Align, style};
use kivact::ui::{ButtonVariant, ControlSize, button, text_area};
use kivact::view::{IntoText, IntoView, Role, View, dynamic};
use kivact::{Binding, Value, signal};

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
    let preview = preview.into();
    let mode = |label, selected| {
        button(label)
            .variant(ButtonVariant::Ghost)
            .size(ControlSize::Small)
            .selected(move || preview.get() == selected)
            .on_press(move || preview.set(selected))
    };
    row((
        mode(message("soft.rich_text.write", "Write"), false),
        mode(message("soft.rich_text.preview", "Preview"), true),
    ))
    .role(Role::Toolbar)
    .name(message("soft.rich_text.toolbar", "Editor toolbar"))
    .style(style().align_items(Align::Center).gap(4))
}

pub struct RichTextEditor {
    root: Node,
    label: Value<String>,
    source: Binding<String>,
    read_only: bool,
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
    }
}

impl RichTextEditor {
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
        } = self;
        let previewing = signal(read_only);
        let surface = dynamic(
            move || previewing.get(),
            move |previewing| {
                if previewing {
                    rich_text_view(move || source.get())
                        .style(style().padding(12))
                        .into_view()
                } else {
                    text_area(label, source).wrap(true).rows(6..=16).into_view()
                }
            },
        );
        let toolbar = (!read_only).then(|| rich_text_toolbar(previewing));
        root.child(column((toolbar, surface)).style(style().gap(4)))
            .into_view()
    }
}

#[cfg(test)]
mod tests {
    use super::RichDoc;

    #[test]
    fn preserves_source_bytes_even_for_unsupported_syntax() {
        let source = "## Title  \r\n\r\n| A | B |\r\n|---|---|\r\n";
        let document = RichDoc::from_markdown(source);
        assert_eq!(document.markdown(), source);
        assert_eq!(document.into_markdown(), source);
    }
}
