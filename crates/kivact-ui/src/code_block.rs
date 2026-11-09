//! Role: Present selectable code with syntax, numbering and clipboard feedback.
use crate::code_values::CodeBlockState;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::variants::{Variant, Variants};
use kivact_core::custom::{Part, TextEditing, part};
use kivact_core::state::{
    Binding, Callback, IntoBinding, IntoText, IntoValue, Value, memo, signal,
};
use kivact_core::style::{Length, StyledRange, style};
use kivact_core::view::{
    Element, IntoView, Node, Orientation, Role, ScrollAxes, View, node, node_ref, scroll, show,
    text,
};
use kivact_interaction::{CopyFeedbackApi, DisclosureApi, use_copy_feedback, use_disclosure};
use std::time::Duration;
/// A read-only multiline editor and its header, gutter and footer.
pub struct CodeBlock {
    root: Node,
    code: Value<String>,
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
type Highlight = Box<dyn Fn(TextEditing, Value<Vec<u32>>, Value<bool>) -> TextEditing>;
/// Display `code` with native selection and a copy button.
/// `line_numbers(true)` enables a reactive gutter; `.highlight(content, f)` uses
/// the same content and cross-line state contract as `TextEditing::line_styles`.
pub fn code_block(code: impl IntoValue<String>) -> CodeBlock {
    let code = code.into_value();
    CodeBlock {
        root: node(),
        code,
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
        let f = std::rc::Rc::new(f);
        self.highlight = Some(Box::new(move |editor, lines, diff| {
            let f = f.clone();
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
    pub fn variant(mut self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl Element for CodeBlock {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// Setup handles and read-only recipe projections, without another interaction owner.
struct CodePresentation {
    name: Value<String>,
    state: Value<CodeBlockState>,
    disclosure: DisclosureApi,
    can_collapse: Value<bool>,
    feedback: CopyFeedbackApi,
}
/// Inputs consumed together when constructing the one native source editor.
struct CodeSource {
    code: Value<String>,
    // xtask-allow(state-fields): option: absent means the component's default
    highlight: Option<Highlight>,
    highlight_lines: Value<Vec<u32>>,
    diff: Value<bool>,
    // xtask-allow(state-fields): option: absent means the component's default
    max_height: Option<Value<Length>>,
}
impl CodeBlock {
    fn presentation(&mut self) -> CodePresentation {
        let code = self.code;
        let collapsible = self.collapsible;
        let threshold = self.collapsible_threshold;
        let expanded = self
            .expanded
            .take()
            .unwrap_or_else(|| signal(true).into_binding());
        let can_collapse = memo(move || {
            collapsible.get() && code.with(|s| s.split('\n').count() >= threshold.get())
        });
        let disclosure = use_disclosure(kivact_core::state::binding(
            move || !can_collapse.get() || expanded.get(),
            move |on| expanded.set(on),
        ));
        let open = disclosure.is_open();
        let duration = kivact_core::style::use_tokens::<crate::UiTokens>()
            .code_block
            .copy_feedback_ms;
        let feedback = use_copy_feedback(Duration::from_millis(duration));
        let copied = feedback.is_copied();
        let numbered = self.line_numbers;
        let wrapped = self.wrap;
        let has_title = self.title.is_some();
        let has_language = self.language.is_some();
        let language_label = self.language_label;
        let copy_button = self.copy_button;
        let has_end = self.end_content.is_some();
        let has_footer = self.footer.is_some();
        let state = memo(move || CodeBlockState {
            numbered: numbered.get(),
            wrapped: wrapped.get(),
            expanded: open.get(),
            copied: copied.get(),
            has_header: has_title
                || (has_language && language_label.get())
                || copy_button.get()
                || has_end
                || can_collapse.get(),
            has_footer,
        });
        let name = self
            .title
            .unwrap_or_else(|| kivact_core::i18n::message("code_block.label", "Code"));
        CodePresentation {
            name,
            state,
            disclosure,
            can_collapse,
            feedback,
        }
    }
    fn copy_control(&self, presentation: &CodePresentation) -> crate::Button {
        let copied = presentation.feedback.is_copied();
        let feedback = presentation.feedback;
        let code = self.code;
        let on_copy = self.on_copy;
        let shown = self.copy_button;
        let state = presentation.state;
        let copied_label = kivact_core::i18n::message("code_block.copied", "Copied");
        let copy_label = kivact_core::i18n::message("code_block.copy", "Copy");
        let control = crate::button(move || {
            if copied.get() {
                copied_label.get()
            } else {
                copy_label.get()
            }
        })
        .variant(crate::ButtonVariant::Ghost)
        .icon(kivact_core::view::icon(move || {
            kivact_core::style::Icon::named(if copied.get() { "check" } else { "copy" })
        }))
        .hide_label(true)
        .hidden(move || !shown.get())
        .on_press(move || {
            feedback.copy(code.get_untracked());
            if let Some(f) = on_copy {
                f.call(())
            }
        });
        with_recipe_style(
            control,
            |r| r.code_block_recipes().copy_recipe(),
            &self.variants,
            move || state.get(),
        )
    }
    fn header(&mut self, presentation: &CodePresentation) -> Node {
        let name = presentation.name;
        let state = presentation.state;
        let can_collapse = presentation.can_collapse;
        let label = with_recipe_style(
            text(name),
            |r| r.code_block_recipes().title_recipe(),
            &self.variants,
            move || state.get(),
        );
        let trigger = with_recipe_style(
            node()
                .part(presentation.disclosure.trigger())
                .name(name)
                .child(text(name))
                .hidden(move || !can_collapse.get()),
            |r| r.code_block_recipes().collapse_recipe(),
            &self.variants,
            move || state.get(),
        );
        let language_shown = self.language_label;
        let language = self.language.map(|language| {
            with_recipe_style(
                text(language).hidden(move || !language_shown.get()),
                |r| r.code_block_recipes().language_recipe(),
                &self.variants,
                move || state.get(),
            )
        });
        let copy = self.copy_control(presentation);
        let copied = presentation.feedback.is_copied();
        let copied_label = kivact_core::i18n::message("code_block.copied", "Copied");
        let status = node()
            .role(Role::Status)
            .live_region(kivact_core::view::LiveRegion::Polite)
            .name(move || {
                if copied.get() {
                    copied_label.get()
                } else {
                    String::new()
                }
            })
            .style(style().size(0, 0));
        let end = self.end_content.take().map(|v| {
            with_recipe_style(
                node().child(v),
                |r| r.code_block_recipes().end_content_recipe(),
                &self.variants,
                move || state.get(),
            )
        });
        with_recipe_style(
            node()
                .child((label, trigger, language, end, copy, status))
                .hidden(move || !state.get().has_header),
            |r| r.code_block_recipes().header_recipe(),
            &self.variants,
            move || state.get(),
        )
    }
}
impl CodeSource {
    fn render(self, presentation: &CodePresentation, variants: &Variants) -> View {
        let state = presentation.state;
        let name = presentation.name;
        let disclosure = presentation.disclosure;
        let variants = variants.clone();
        let motion = kivact_core::style::use_tokens::<crate::UiTokens>()
            .disclosure
            .motion
            .clone();
        show(disclosure.is_open(), move || {
            node()
                .part(disclosure.content())
                .role(Role::Group)
                .name(name)
                .child(self.viewport(name, state, &variants))
                .style(style().exit_size(Orientation::Vertical).transition(
                    kivact_core::style::TransitionProp::Size,
                    motion.transition.clone().unwrap_or_else(|| {
                        kivact_core::motion::Transition::tween(
                            motion.duration.as_secs_f64() * 1000.0,
                            kivact_core::motion::Easing::EaseOut,
                        )
                    }),
                ))
        })
        .into_view()
    }
    fn viewport(
        &self,
        name: Value<String>,
        state: Value<CodeBlockState>,
        variants: &Variants,
    ) -> View {
        let editing = TextEditing::new(self.code)
            .multiline()
            .read_only(true)
            .behavior(crate::code_editing::code_keys);
        let editing = match &self.highlight {
            Some(highlight) => highlight(editing, self.highlight_lines, self.diff),
            None => crate::code_editing::plain_highlight(editing, self.highlight_lines, self.diff),
        };
        let (editor, reference) = native_editor(editing, name, variants, state);
        let gutter = crate::code_gutter::gutter(reference, self.code, variants, state);
        let body = with_recipe_style(
            node().child((editor, gutter)),
            |r| r.code_block_recipes().body_recipe(),
            variants,
            move || state.get(),
        );
        let max_height = self.max_height;
        with_recipe_style(
            scroll(body).axes(ScrollAxes::Both),
            |r| r.code_block_recipes().viewport_recipe(),
            variants,
            move || state.get(),
        )
        .style(move || match max_height {
            Some(height) => style().max_height(height.get()),
            None => style(),
        })
        .into_view()
    }
}
fn native_editor(
    editing: TextEditing,
    name: Value<String>,
    variants: &Variants,
    state: Value<CodeBlockState>,
) -> (Node, kivact_core::view::NodeRef) {
    let editing = editing.line_layout(move || kivact_core::custom::EditLineLayout {
        wrap: state.get().wrapped,
        rows: Some((1, usize::MAX)),
    });
    let reference = node_ref();
    let editor = Part::new(part(|s| s).state(), move |node| {
        node.role(Role::TextBox)
            .focusable(true)
            .name(name)
            .text_editing(editing)
    });
    (
        with_recipe_style(
            node().part(editor).node_ref(reference),
            |r| r.code_block_recipes().code_recipe(),
            variants,
            move || state.get(),
        ),
        reference,
    )
}
impl IntoView for CodeBlock {
    fn into_view(mut self) -> View {
        let presentation = self.presentation();
        let header = self.header(&presentation);
        let state = presentation.state;
        let footer = self.footer.take().map(|v| {
            with_recipe_style(
                node().child(v),
                |r| r.code_block_recipes().footer_recipe(),
                &self.variants,
                move || state.get(),
            )
        });
        let source = CodeSource {
            code: self.code,
            highlight: self.highlight,
            highlight_lines: self.highlight_lines,
            diff: self.diff,
            max_height: self.max_height,
        };
        let viewport = source.render(&presentation, &self.variants);
        with_base_recipe_style(
            self.root,
            |r| r.code_block_recipes().root_recipe(),
            &self.variants,
            move || state.get(),
        )
        .child((header, viewport, footer))
        .into_view()
    }
}
