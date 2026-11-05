//! Role: Display selectable source code with themed syntax, numbering and clipboard feedback.
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::variants::{Variant, Variants};
use kivact_core::custom::{Part, TextEditing, part};
use kivact_core::state::{Binding, Callback, IntoBinding, IntoText, IntoValue, Value, signal};
use kivact_core::style::{Length, StyledRange, style};
use kivact_core::view::{IntoView, Node, NodeBuilder, Role, ScrollAxes, View, node, scroll, text};
use kivact_interaction::{use_copy_feedback, use_disclosure};
use std::time::Duration;
/// Rendered source facts supplied to code-block recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeBlockState {
    pub numbered: bool,
    pub wrapped: bool,
    pub expanded: bool,
    pub copied: bool,
}
/// A read-only multiline editor and its header, gutter and footer.
pub struct CodeBlock {
    root: Node,
    code: Value<String>,
    editing: TextEditing,
    title: Option<Value<String>>,
    language: Option<Value<String>>,
    language_label: Value<bool>,
    line_numbers: Value<bool>,
    highlight_lines: Value<Vec<u32>>,
    diff: Value<bool>,
    copy_button: Value<bool>,
    on_copy: Option<Callback<()>>,
    end_content: Option<View>,
    footer: Option<View>,
    wrap: Value<bool>,
    max_height: Option<Value<Length>>,
    collapsible: Value<bool>,
    collapsible_threshold: Value<usize>,
    expanded: Option<Binding<bool>>,
    highlight: Option<Highlight>,
    variants: Variants,
}
type Highlight = Box<dyn FnOnce(TextEditing, Value<Vec<u32>>, Value<bool>) -> TextEditing>;
/// Display `code` with native selection and a copy button.
/// `line_numbers(true)` enables a reactive gutter; `.highlight(content, f)` uses
/// the same content and cross-line state contract as `TextEditing::line_styles`.
pub fn code_block(code: impl IntoValue<String>) -> CodeBlock {
    let code = code.into_value();
    CodeBlock {
        root: node(),
        code,
        editing: TextEditing::new(code)
            .multiline()
            .read_only(true)
            .behavior(crate::code_editing::code_keys),
        title: None,
        language: None,
        language_label: true.into_value(),
        line_numbers: false.into_value(),
        highlight_lines: Vec::new().into_value(),
        diff: false.into_value(),
        copy_button: true.into_value(),
        on_copy: None,
        end_content: None,
        footer: None,
        wrap: false.into_value(),
        max_height: None,
        collapsible: false.into_value(),
        collapsible_threshold: 10usize.into_value(),
        expanded: None,
        highlight: None,
        variants: Variants::default(),
    }
}
impl CodeBlock {
    /// Show a title in the header and use it as the editor's accessible name.
    pub fn title(mut self, title: impl IntoText) -> Self {
        self.title = Some(title.into_text_value());
        self
    }
    /// Set a display-only language label; syntax coloring is supplied by `highlight`.
    pub fn language(mut self, language: impl IntoText) -> Self {
        self.language = Some(language.into_text_value());
        self
    }
    /// Show or hide the language label without changing syntax coloring.
    pub fn language_label(mut self, on: impl IntoValue<bool>) -> Self {
        self.language_label = on.into_value();
        self
    }
    /// Number the source lines starting at one, constant or reactive.
    pub fn line_numbers(mut self, on: impl IntoValue<bool>) -> Self {
        self.line_numbers = on.into_value();
        self
    }
    /// Style each line with a content source and cross-line parser state.
    /// The callback is pure and has the `TextEditing::line_styles` contract.
    pub fn highlight<T, S, F>(mut self, content: impl IntoValue<T>, f: F) -> Self
    where
        T: Clone + PartialEq + 'static,
        S: PartialEq + Default + 'static,
        F: Fn(&T, &str, &S) -> (Vec<StyledRange>, S) + 'static,
    {
        let content = content.into_value();
        self.highlight = Some(Box::new(move |editor, lines, diff| {
            editor.line_styles(
                move || (content.get(), lines.get(), diff.get()),
                move |(content, lines, diff), line, state: &(S, usize)| {
                    let (mut ranges, next) = f(content, line, &state.0);
                    crate::code_editing::line_background(&mut ranges, line, state.1, lines, *diff);
                    (ranges, (next, state.1 + 1))
                },
            )
        }));
        self
    }
    /// Emphasize these one-based source line numbers.
    pub fn highlight_lines(mut self, lines: impl IntoValue<Vec<u32>>) -> Self {
        self.highlight_lines = lines.into_value();
        self
    }
    /// Color added and removed diff lines, keeping their original source text.
    pub fn diff(mut self, on: impl IntoValue<bool>) -> Self {
        self.diff = on.into_value();
        self
    }
    /// Show the whole-source copy action.
    pub fn copy_button(mut self, on: impl IntoValue<bool>) -> Self {
        self.copy_button = on.into_value();
        self
    }
    /// Observe copy requests; the component always writes the source to the clipboard.
    pub fn on_copy(mut self, f: impl Into<Callback<()>>) -> Self {
        self.on_copy = Some(f.into());
        self
    }
    /// Fill the trailing header slot.
    pub fn end_content(mut self, content: impl IntoView) -> Self {
        self.end_content = Some(content.into_view());
        self
    }
    /// Fill the footer slot.
    pub fn footer(mut self, content: impl IntoView) -> Self {
        self.footer = Some(content.into_view());
        self
    }
    /// Soft-wrap source at the available editor width.
    pub fn wrap(mut self, on: impl IntoValue<bool>) -> Self {
        self.wrap = on.into_value();
        self
    }
    /// Scroll source vertically beyond this height.
    pub fn max_height(mut self, height: impl IntoValue<Length>) -> Self {
        self.max_height = Some(height.into_value());
        self
    }
    /// Allow collapsing source with the header disclosure.
    pub fn collapsible(mut self, on: impl IntoValue<bool>) -> Self {
        self.collapsible = on.into_value();
        self
    }
    /// Only offer collapse when source has at least this many lines.
    pub fn collapsible_threshold(mut self, lines: impl IntoValue<usize>) -> Self {
        self.collapsible_threshold = lines.into_value();
        self
    }
    /// Bind expanded source to the application; defaults to expanded.
    pub fn expanded(mut self, expanded: impl IntoBinding<bool>) -> Self {
        self.expanded = Some(expanded.into_binding());
        self
    }
    /// Add an appearance read by all code recipes.
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for CodeBlock {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for CodeBlock {
    fn into_view(self) -> View {
        let code = self.code;
        let numbered = self.line_numbers;
        let wrapped = self.wrap;
        let expanded = self.expanded.unwrap_or_else(|| signal(true).into_binding());
        let can_collapse = move || {
            self.collapsible.get()
                && code.with(|s| s.split('\n').count() >= self.collapsible_threshold.get())
        };
        let disclosure = use_disclosure(kivact_core::state::binding(
            move || !can_collapse() || expanded.get(),
            move |on| expanded.set(on),
        ));
        let open = disclosure.is_open();
        let duration = kivact_core::style::use_tokens::<crate::UiTokens>()
            .code_block
            .copy_feedback_ms;
        let feedback = use_copy_feedback(Duration::from_millis(duration));
        let copied = feedback.is_copied();
        let state = move || CodeBlockState {
            numbered: numbered.get(),
            wrapped: wrapped.get(),
            expanded: open.get(),
            copied: copied.get(),
        };
        let name = self
            .title
            .unwrap_or_else(|| kivact_core::i18n::message("code_block.label", "Code"));
        let label = with_recipe_style(
            text(name),
            |r| r.code_block_recipes().title_recipe(),
            &self.variants,
            state,
        );
        let trigger = with_recipe_style(
            node()
                .part(disclosure.trigger())
                .name(name)
                .child(text(name))
                .hidden(move || !can_collapse()),
            |r| r.code_block_recipes().collapse_recipe(),
            &self.variants,
            state,
        );
        let language = self.language.map(|language| {
            with_recipe_style(
                text(language).hidden(move || !self.language_label.get()),
                |r| r.code_block_recipes().language_recipe(),
                &self.variants,
                state,
            )
        });
        let on_copy = self.on_copy;
        let copy = with_recipe_style(
            crate::button(move || {
                if copied.get() {
                    kivact_core::i18n::message("code_block.copied", "Copied").get()
                } else {
                    kivact_core::i18n::message("code_block.copy", "Copy").get()
                }
            })
            .hidden(move || !self.copy_button.get())
            .on_press(move || {
                feedback.copy(code.get_untracked());
                if let Some(f) = on_copy {
                    f.call(())
                }
            }),
            |r| r.code_block_recipes().copy_recipe(),
            &self.variants,
            state,
        );
        let end = self.end_content.map(|v| {
            with_recipe_style(
                node().child(v),
                |r| r.code_block_recipes().end_content_recipe(),
                &self.variants,
                state,
            )
        });
        let header = with_recipe_style(
            node().child((label, trigger, language, end, copy)),
            |r| r.code_block_recipes().header_recipe(),
            &self.variants,
            state,
        );
        let mut editing = self.editing;
        editing = match self.highlight {
            Some(highlight) => highlight(editing, self.highlight_lines, self.diff),
            None => crate::code_editing::plain_highlight(editing, self.highlight_lines, self.diff),
        };
        editing = editing.line_layout(move || kivact_core::custom::EditLineLayout {
            wrap: wrapped.get(),
            rows: Some((1, usize::MAX)),
        });
        let editor = Part::new(part(|s| s).state(), move |node| {
            node.role(Role::TextBox)
                .focusable(true)
                .name(name)
                .text_editing(editing)
        });
        let editor = with_recipe_style(
            node().part(editor),
            |r| r.code_block_recipes().code_recipe(),
            &self.variants,
            state,
        );
        let gutter = with_recipe_style(
            text(move || {
                code.with(|s| {
                    (1..=s.split('\n').count())
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                })
            })
            .hidden(move || !numbered.get()),
            |r| r.code_block_recipes().gutter_recipe(),
            &self.variants,
            state,
        );
        let body = with_recipe_style(
            node().child((gutter, editor)),
            |r| r.code_block_recipes().body_recipe(),
            &self.variants,
            state,
        );
        let max_height = self.max_height;
        let viewport = with_recipe_style(
            scroll(body)
                .axes(ScrollAxes::Both)
                .hidden(move || !open.get()),
            |r| r.code_block_recipes().viewport_recipe(),
            &self.variants,
            state,
        )
        .style(move || match max_height {
            Some(height) => style().max_height(height.get()),
            None => style(),
        });
        let footer = self.footer.map(|v| {
            with_recipe_style(
                node().child(v),
                |r| r.code_block_recipes().footer_recipe(),
                &self.variants,
                state,
            )
        });
        with_base_recipe_style(
            self.root,
            |r| r.code_block_recipes().root_recipe(),
            &self.variants,
            state,
        )
        .child((header, viewport, footer))
        .into_view()
    }
}
