//! Role: Build the code block, its header, gutter, footer and copy behavior.

use crate::code_block::look::CodeBlockState;
use crate::code_block::look::CodeBlockTheme;
use crate::code_block::look::CodeCollapse;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::custom::{Caller, Part, TextEditing, part};
use kivact_core::state::IntoValue;
use kivact_core::state::{Callback, Signal, Value, memo, signal};
use kivact_core::style::{Length, StyledRange, style};
use kivact_core::view::Variants;
use kivact_core::view::{
    Element, IntoView, Node, Orientation, Role, ScrollAxes, View, node, node_ref, scroll, show,
    text,
};
use kivact_interaction::{CopyFeedbackApi, DisclosureApi, use_copy_feedback, use_disclosure};
use kivact_macros::component;
use std::time::Duration;

/// The line-styling function a [`Highlighter`] holds.
type HighlightRun = std::rc::Rc<dyn Fn(TextEditing, Value<Vec<u32>>, Value<bool>) -> TextEditing>;
/// A line highlighter for [`CodeBlock::highlight`], built by [`highlighter`].
pub struct Highlighter {
    run: HighlightRun,
}
/// Style each line with a content source and cross-line parser state.
/// The callback is pure and has the `TextEditing::line_styles` contract.
#[track_caller]
pub fn highlighter<T, S, F>(content: impl IntoValue<T>, f: F) -> Highlighter
where
    T: Clone + PartialEq + 'static,
    S: PartialEq + Default + 'static,
    F: Fn(&T, &str, &S) -> (Vec<StyledRange>, S) + 'static,
{
    let content = content.into_value();
    let f = std::rc::Rc::new(f);
    Highlighter {
        run: std::rc::Rc::new(
            move |editor: TextEditing, lines: Value<Vec<u32>>, diff: Value<bool>| {
                let f = f.clone();
                editor.line_styles(
                    move || (content.get(), lines.get(), diff.get()),
                    move |(content, lines, diff), line, state: &(S, usize)| {
                        let (mut ranges, next) = f(content, line, &state.0);
                        crate::code_block::editing::line_background(
                            &mut ranges,
                            line,
                            state.1,
                            lines,
                            *diff,
                        );
                        (ranges, (next, state.1 + 1))
                    },
                )
            },
        ),
    }
}

/// Display `code` with native selection and a copy button.
///
/// `title` shows in the header and names the editor. `language` is a display-only
/// label, shown while `language_label` is true; syntax coloring comes from `highlight`,
/// which takes a [`highlighter`] (content source and cross-line parser state).
/// `line_numbers` numbers the source lines from one, and `highlight_lines` emphasizes
/// one-based source lines. `diff` colors added and removed lines. `copy_button` shows
/// the whole-source copy action; `on_copy` observes copies, and the component always
/// writes the source to the clipboard. `end_content` fills the trailing header slot and
/// `footer` the footer. `wrap` soft-wraps at the editor width, and `max_height` scrolls
/// beyond that height. `collapsible` allows collapsing with the header disclosure once the
/// source has `collapsible_threshold` lines or more. `expanded` binds the expanded source
/// to the application and defaults to expanded.
#[component]
pub fn code_block(
    code: Value<String>,
    title: Option<Value<String>>,
    language: Option<Value<String>>,
    #[prop(default = true)] language_label: Value<bool>,
    #[prop(default = false)] line_numbers: Value<bool>,
    #[prop(default = Vec::<u32>::new())] highlight_lines: Value<Vec<u32>>,
    #[prop(default = false)] diff: Value<bool>,
    #[prop(default = true)] copy_button: Value<bool>,
    on_copy: Option<Callback<()>>,
    end_content: Option<View>,
    footer: Option<View>,
    #[prop(default = false)] wrap: Value<bool>,
    max_height: Option<Value<Length>>,
    #[prop(default = false)] collapsible: Value<bool>,
    #[prop(default = 10usize)] collapsible_threshold: Value<usize>,
    expanded: Option<Signal<bool>>,
    highlight: Option<Highlighter>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let expanded = expanded.unwrap_or_else(|| signal(true));
    let can_collapse = memo(move || {
        collapsible.get() && code.with(|s| s.split('\n').count() >= collapsible_threshold.get())
    });
    let disclosure = use_disclosure(kivact_core::state::binding(
        move || !can_collapse.get() || expanded.get(),
        move |on| expanded.set(on),
    ));
    let open = disclosure.is_open();
    let duration = kivact_core::style::use_theme()
        .with(|t| t.get(&crate::code_block::look::COPY_FEEDBACK_MS_VAR) as u64);
    let feedback = use_copy_feedback(Duration::from_millis(duration));
    let copied = feedback.is_copied();
    let has_title = title.is_some();
    let has_language = language.is_some();
    let has_end = end_content.is_some();
    let has_footer = footer.is_some();
    let state = memo(move || CodeBlockState {
        numbered: line_numbers.get(),
        wrapped: wrap.get(),
        collapse: match (can_collapse.get(), open.get()) {
            (false, _) => CodeCollapse::Fixed,
            (true, true) => CodeCollapse::Expanded,
            (true, false) => CodeCollapse::Collapsed,
        },
        copied: copied.get(),
        has_header_content: has_title
            || (has_language && language_label.get())
            || copy_button.get()
            || has_end,
        has_footer,
    });
    let name = title.unwrap_or_else(|| kivact_core::i18n::message("code_block.label", "Code"));
    let presentation = CodePresentation {
        name,
        state,
        disclosure,
        can_collapse,
        feedback,
    };
    let copy = copy_control(&presentation, &variants, code, on_copy, copy_button);
    let header_node = header(
        &presentation,
        &variants,
        language,
        language_label,
        copy,
        end_content,
    );
    let footer = footer.map(|v| {
        with_recipe_style(
            node().child(v),
            |r: &CodeBlockTheme| r.footer_recipe(),
            &variants,
            move || state.get(),
        )
    });
    let source = CodeSource {
        code,
        highlight,
        highlight_lines,
        diff,
        max_height,
    };
    let viewport = source.render(&presentation, &variants);
    with_base_recipe_style(
        node(),
        |r: &CodeBlockTheme| r.root_recipe(),
        &variants,
        move || state.get(),
    )
    .child((header_node, viewport, footer))
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
    highlight: Option<Highlighter>,
    highlight_lines: Value<Vec<u32>>,
    diff: Value<bool>,
    max_height: Option<Value<Length>>,
}
fn copy_control(
    presentation: &CodePresentation,
    variants: &Variants,
    code: Value<String>,
    on_copy: Option<Callback<()>>,
    shown: Value<bool>,
) -> crate::Button {
    let copied = presentation.feedback.is_copied();
    let feedback = presentation.feedback;
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
        |r: &CodeBlockTheme| r.copy_recipe(),
        variants,
        move || state.get(),
    )
}
fn header(
    presentation: &CodePresentation,
    variants: &Variants,
    language: Option<Value<String>>,
    language_label: Value<bool>,
    copy: crate::Button,
    end_content: Option<View>,
) -> Node {
    let name = presentation.name;
    let state = presentation.state;
    let can_collapse = presentation.can_collapse;
    let label = with_recipe_style(
        text(name),
        |r: &CodeBlockTheme| r.title_recipe(),
        variants,
        move || state.get(),
    );
    let trigger = with_recipe_style(
        node()
            .part(presentation.disclosure.trigger())
            .name(name)
            .child(text(name))
            .hidden(move || !can_collapse.get()),
        |r: &CodeBlockTheme| r.collapse_recipe(),
        variants,
        move || state.get(),
    );
    let language = language.map(|language| {
        with_recipe_style(
            text(language).hidden(move || !language_label.get()),
            |r: &CodeBlockTheme| r.language_recipe(),
            variants,
            move || state.get(),
        )
    });
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
    let end = end_content.map(|v| {
        with_recipe_style(
            node().child(v),
            |r: &CodeBlockTheme| r.end_content_recipe(),
            variants,
            move || state.get(),
        )
    });
    with_recipe_style(
        node()
            .child((label, trigger, language, end, copy, status))
            .hidden(move || !state.get().has_header()),
        |r: &CodeBlockTheme| r.header_recipe(),
        variants,
        move || state.get(),
    )
}
impl CodeSource {
    fn render(self, presentation: &CodePresentation, variants: &Variants) -> View {
        let state = presentation.state;
        let name = presentation.name;
        let disclosure = presentation.disclosure;
        let variants = variants.clone();
        let motion = kivact_core::style::use_theme().with(|t| {
            t.ext::<crate::disclosure::look::DisclosureTheme>()
                .motion_value()
                .clone()
        });
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
            .behavior(crate::code_block::editing::code_keys);
        let editing = match &self.highlight {
            Some(highlight) => (highlight.run)(editing, self.highlight_lines, self.diff),
            None => crate::code_block::editing::plain_highlight(
                editing,
                self.highlight_lines,
                self.diff,
            ),
        };
        let (editor, reference) = native_editor(editing, name, variants, state);
        let gutter = crate::code_block::gutter::gutter(reference, self.code, variants, state);
        let body = with_recipe_style(
            node().child((editor, gutter)),
            |r: &CodeBlockTheme| r.body_recipe(),
            variants,
            move || state.get(),
        );
        let max_height = self.max_height;
        with_recipe_style(
            scroll(body).axes(ScrollAxes::Both),
            |r: &CodeBlockTheme| r.viewport_recipe(),
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
            |r: &CodeBlockTheme| r.code_recipe(),
            variants,
            move || state.get(),
        ),
        reference,
    )
}
