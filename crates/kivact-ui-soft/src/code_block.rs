//! Role: Render syntax-highlighted code with line and copy controls.
use crate::{IconSize, SyntaxColors, icon, tokens, visually_hidden};
use kivact::i18n::message;
use kivact::interaction::use_content_reveal;
use kivact::interaction::use_disclosure;
use kivact::motion::Transition;
use kivact::state::Callback;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ScrollAxes;
use kivact::view::Span;
use kivact::view::column;
use kivact::view::node;
use kivact::view::rich_text;
use kivact::view::row;
use kivact::view::scroll;
use kivact::view::span;
use kivact::view::text;

use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Signal;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::memo;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Length;
use kivact::style::TextAlign;
use kivact::style::TransitionProp;
use kivact::style::pct;
use kivact::style::style;
use kivact::task::sleep;
use kivact::task::spawn;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::button;
use kivact::ui::tooltip;
use kivact::view::IntoView;
use kivact::view::LiveRegion;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::dynamic;
use kivact::view::node_ref;
use kivact::view::show;
use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;
use std::time::Duration;

/// A syntax coloring category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Keyword,
    Type,
    String,
    Comment,
    Operator,
    Punctuation,
    Number,
    Constant,
    Function,
    Variable,
    Tag,
    Attribute,
    Property,
}

type Tokenizer = fn(&str) -> Vec<(Range<usize>, TokenKind)>;

/// A caller-supplied code language.
#[derive(Clone)]
pub struct CodeLanguage {
    pub(crate) name: String,
    pub(crate) identifiers: Vec<String>,
    tokenizer: Tokenizer,
}

impl CodeLanguage {
    /// Define a language with a tokenizer returning ordered UTF-8 byte ranges.
    pub fn custom(
        name: impl Into<String>,
        identifiers: impl IntoIterator<Item = impl Into<String>>,
        tokenizer: Tokenizer,
    ) -> Self {
        Self {
            name: name.into(),
            identifiers: identifiers.into_iter().map(Into::into).collect(),
            tokenizer,
        }
    }
}

/// The built-in identifier or custom language used for highlighting.
#[derive(Clone)]
pub enum CodeLanguageInput {
    Named(String),
    Custom(CodeLanguage),
}

impl From<&str> for CodeLanguageInput {
    fn from(value: &str) -> Self {
        Self::Named(value.to_ascii_lowercase())
    }
}
impl From<String> for CodeLanguageInput {
    fn from(value: String) -> Self {
        Self::Named(value.to_ascii_lowercase())
    }
}
impl From<CodeLanguage> for CodeLanguageInput {
    fn from(value: CodeLanguage) -> Self {
        Self::Custom(value)
    }
}

impl CodeLanguageInput {
    fn label(&self) -> &str {
        match self {
            Self::Named(name) if name == "plaintext" => "",
            Self::Named(name) => name,
            Self::Custom(language) => &language.name,
        }
    }
    fn tokenize(&self, line: &str) -> Vec<(Range<usize>, TokenKind)> {
        match self {
            Self::Named(name) => scan(line, name),
            Self::Custom(language) => (language.tokenizer)(line),
        }
    }
}

/// The size of the code text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CodeBlockSize {
    Sm,
    #[default]
    Md,
}
/// The outer width policy of a code block.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CodeBlockWidth {
    #[default]
    Fit,
    Fill,
    Px(f64),
}
/// Whether the code block has a card frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CodeBlockContainer {
    #[default]
    Card,
    Section,
}

/// A Neutral syntax palette override.
pub type SyntaxTheme = SyntaxColors;

/// A configurable source-code display.
pub struct CodeBlock {
    root: Node,
    code: Value<String>,
    language: CodeLanguageInput,
    title: Option<Value<String>>,
    language_label: Value<bool>,
    line_numbers: Value<bool>,
    highlight_lines: Value<Vec<u32>>,
    diff: Value<bool>,
    copy_button: Value<bool>,
    on_copy: Option<Rc<dyn Fn()>>,
    end_content: Option<View>,
    wrap: Value<bool>,
    max_height: Option<Length>,
    collapsible: Value<bool>,
    collapsible_threshold: usize,
    expanded: Signal<bool>,
    size: Value<CodeBlockSize>,
    width: Value<CodeBlockWidth>,
    container: Value<CodeBlockContainer>,
    syntax_theme: Option<SyntaxTheme>,
}

/// Render source code.
pub fn code_block(code: impl IntoValue<String>) -> CodeBlock {
    CodeBlock {
        root: node(),
        code: code.into_value(),
        language: "plaintext".into(),
        title: None,
        language_label: (true).into_value(),
        line_numbers: false.into_value(),
        highlight_lines: Vec::new().into_value(),
        diff: (false).into_value(),
        copy_button: (true).into_value(),
        on_copy: None,
        end_content: None,
        wrap: false.into_value(),
        max_height: None,
        collapsible: (false).into_value(),
        collapsible_threshold: 10,
        expanded: signal(true),
        size: (CodeBlockSize::Md).into_value(),
        width: (CodeBlockWidth::Fit).into_value(),
        container: (CodeBlockContainer::Card).into_value(),
        syntax_theme: None,
    }
}

impl CodeBlock {
    /// Select a language for syntax highlighting.
    pub fn language(mut self, value: impl Into<CodeLanguageInput>) -> Self {
        self.language = value.into();
        self
    }
    /// Show a title above the code.
    pub fn title(mut self, value: impl IntoText) -> Self {
        self.title = Some(value.into_text_value());
        self
    }
    /// Show or hide the language label.
    pub fn language_label(mut self, value: impl IntoValue<bool>) -> Self {
        self.language_label = value.into_value();
        self
    }
    /// Show the line-number gutter.
    pub fn line_numbers(mut self, value: impl IntoValue<bool>) -> Self {
        self.line_numbers = value.into_value();
        self
    }
    /// Highlight the given one-based line numbers.
    pub fn highlight_lines(mut self, value: impl IntoValue<Vec<u32>>) -> Self {
        self.highlight_lines = value.into_value();
        self
    }
    /// Render unified-diff markings.
    pub fn diff(mut self, value: impl IntoValue<bool>) -> Self {
        self.diff = value.into_value();
        self
    }
    /// Show or hide the copy button.
    pub fn copy_button(mut self, value: impl IntoValue<bool>) -> Self {
        self.copy_button = value.into_value();
        self
    }
    /// Run a callback after a successful copy.
    pub fn on_copy(mut self, value: impl Into<Callback<()>>) -> Self {
        let value: Callback<()> = value.into();
        self.on_copy = Some(Rc::new(move || value.call(())));
        self
    }
    /// Add caller controls before the copy button.
    pub fn end_content(mut self, value: impl IntoView) -> Self {
        self.end_content = Some(value.into_view());
        self
    }
    /// Wrap long code lines.
    pub fn wrap(mut self, value: impl IntoValue<bool>) -> Self {
        self.wrap = value.into_value();
        self
    }
    /// Limit the scrolling viewport's height.
    pub fn max_height(mut self, value: impl Into<Length>) -> Self {
        self.max_height = Some(value.into());
        self
    }
    /// Make a sufficiently long block collapsible.
    pub fn collapsible(mut self, value: impl IntoValue<bool>) -> Self {
        self.collapsible = value.into_value();
        self
    }
    /// Set the minimum number of lines for the collapse control.
    pub fn collapsible_threshold(mut self, value: usize) -> Self {
        self.collapsible_threshold = value;
        self
    }
    /// Supply the expansion state.
    pub fn expanded(mut self, value: Signal<bool>) -> Self {
        self.expanded = value;
        self
    }
    /// Set the size of the code text.
    pub fn size(mut self, value: impl IntoValue<CodeBlockSize>) -> Self {
        self.size = value.into_value();
        self
    }
    /// Set the width policy.
    pub fn width(mut self, value: impl IntoValue<CodeBlockWidth>) -> Self {
        self.width = value.into_value();
        self
    }
    /// Select the card or section frame.
    pub fn container(mut self, value: impl IntoValue<CodeBlockContainer>) -> Self {
        self.container = value.into_value();
        self
    }
    /// Override the syntax palette for this block.
    pub fn syntax_theme(mut self, value: SyntaxTheme) -> Self {
        self.syntax_theme = Some(value);
        self
    }
}

impl NodeBuilder for CodeBlock {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for CodeBlock {
    fn into_view(self) -> View {
        let Self {
            root,
            code,
            language,
            title,
            language_label,
            line_numbers,
            highlight_lines,
            diff,
            copy_button,
            on_copy,
            end_content,
            wrap,
            max_height,
            collapsible,
            collapsible_threshold,
            expanded,
            size,
            width,
            container,
            syntax_theme,
        } = self;
        let palette = memo(move || syntax_theme.unwrap_or_else(|| tokens().colors.syntax));
        let lang = match (language.label(), diff.get()) {
            ("", false) => String::new(),
            ("", true) => "diff.get()".to_owned(),
            (label, false) => label.to_owned(),
            (label, true) => format!("{label} diff.get()"),
        };
        let fallback = message("soft.code_block.code", "Code");
        let viewport_label = if lang.is_empty() {
            fallback
        } else {
            lang.clone().into_value()
        };
        let has_header = title.is_some() || (language_label.get() && !lang.is_empty());
        let heading = memo(move || {
            let title = title.map(|value| value.get()).unwrap_or_default();
            match (title.is_empty(), lang.is_empty() || !language_label.get()) {
                (true, true) => String::new(),
                (true, false) => lang.clone(),
                (false, true) => title,
                (false, false) => format!("{title} — {lang}"),
            }
        });
        let can_collapse = memo(move || {
            has_header && collapsible.get() && code.get().lines().count() >= collapsible_threshold
        });
        let disclosure = use_disclosure(binding(
            move || !can_collapse || expanded.get(),
            move |open| expanded.set(open),
        ));
        let motion = tokens().motion;
        let reveal = use_content_reveal(
            disclosure.is_open(),
            Transition::tween(motion.medium as f64, motion.ease_standard),
        );
        let disclosure = disclosure.leaving(move || reveal.leaving());
        let copied = signal(false);
        let copied_label = message("soft.code_block.copied", "Copied");
        let controls = row((
            end_content,
            copy_button
                .get()
                .then(|| copy_control(code, copied, copied_label, palette, on_copy)),
        ))
        .style(|| {
            style()
                .align_items(Align::Center)
                .gap(tokens().spacing.s1)
                .shrink(0)
        });
        let header_title = dynamic(can_collapse, move |can_collapse| {
            let label = text(heading).style(style().single_line());
            if can_collapse {
                let disclosure_label = move || {
                    let title = heading.get();
                    if title.is_empty() {
                        fallback.get()
                    } else {
                        title
                    }
                };
                row((
                    icon("chevron-right").size(IconSize::Xs).style(move || {
                        style()
                            .rotate(if expanded.get() { 90 } else { 0 })
                            .transition(
                                TransitionProp::Paint,
                                Transition::tween(motion.fast as f64, motion.ease_standard),
                            )
                    }),
                    label,
                ))
                .part(disclosure.trigger())
                .name(disclosure_label)
                .style(|| style().gap(tokens().spacing.s1).align_items(Align::Center))
            } else {
                label
            }
        });
        let lines = memo(move || {
            (
                code.get(),
                line_numbers.get(),
                highlight_lines.get(),
                wrap.get(),
            )
        });
        let content = move || {
            let language = language.clone();
            let body = dynamic(lines, move |(code, numbered, highlighted, wrapped)| {
                render_lines(
                    &code,
                    &language,
                    numbered,
                    &highlighted,
                    wrapped,
                    diff.get(),
                    size.get(),
                    palette,
                )
            });
            scroll(body)
                .axes(if wrap.get() {
                    ScrollAxes::Vertical
                } else {
                    ScrollAxes::Both
                })
                .style(move || match max_height {
                    Some(height) => style().max_height(height),
                    None => style(),
                })
        };
        let measured = node_ref();
        let body = column(show(
            move || reveal.present(),
            move || {
                let content = content.clone();
                column(dynamic(wrap, move |_| content()))
                    .node_ref(measured)
                    .style(style().shrink(0))
            },
        ))
        .role(Role::Group)
        .name(viewport_label)
        .part(disclosure.content())
        .style(style().clip(true))
        .style(reveal.container_style(measured));
        let inside = if has_header {
            column((
                row((header_title, controls)).style(move || {
                    let p = tokens();
                    let mut look = style()
                        .justify_content(Justify::SpaceBetween)
                        .align_items(Align::Center)
                        .padding_x(p.spacing.s4)
                        .padding_y(p.spacing.s2)
                        .gap(p.spacing.s2)
                        .font_family(p.typography.family_code)
                        .font_size(p.typography.supporting.size_px)
                        .weight(p.typography.weight_medium)
                        .color(palette.get().comment);
                    if line_numbers.get() {
                        look = look.border_bottom(p.sizes.border_width, p.colors.border);
                    }
                    look
                }),
                body,
            ))
        } else {
            row((
                node().style(style().grow(1).min_width(0)).child(body),
                controls.style(|| {
                    style()
                        .margin_x(tokens().spacing.s2)
                        .margin_top(tokens().spacing.s2)
                }),
            ))
        };
        root.base_style(move || {
            let p = tokens();
            let look = style()
                .direction(Direction::Column)
                .min_width(0)
                .max_width(pct(100));
            let look = match width.get() {
                CodeBlockWidth::Fit => look.align_self(Align::Start),
                CodeBlockWidth::Fill => look.width(pct(100)),
                CodeBlockWidth::Px(width) => look.width(width),
            };
            if container.get() == CodeBlockContainer::Card {
                look.background(palette.get().background)
                    .border(p.sizes.border_width, p.colors.border)
                    .radius(p.radius.element)
                    .clip(true)
            } else {
                look
            }
        })
        .child(inside)
        .child(
            visually_hidden(move || {
                if copied.get() {
                    copied_label.get()
                } else {
                    String::new()
                }
            })
            .live(LiveRegion::Polite),
        )
        .into_view()
    }
}

fn copy_control(
    code: Value<String>,
    copied: Signal<bool>,
    copied_label: Value<String>,
    palette: Value<SyntaxColors>,
    on_copy: Option<Rc<dyn Fn()>>,
) -> View {
    let copy = message("soft.code_block.copy_code", "Copy code");
    let generation = Rc::new(Cell::new(0u64));
    let control = button(move || {
        if copied.get() {
            copied_label.get()
        } else {
            copy.get()
        }
    })
    .variant(ButtonVariant::Ghost)
    .size(ControlSize::Small)
    .hide_label(true)
    .icon(
        icon(move || kivact::style::Icon::named(if copied.get() { "check" } else { "copy" }))
            .size(IconSize::Sm),
    )
    .style(move || style().color(palette.get().comment))
    .on_press(move || {
        let code = code.get_untracked();
        let on_copy = on_copy.clone();
        let generation = generation.clone();
        spawn(async move {
            kivact::platform::copy_text(code).await?;
            copied.set(true);
            let stamp = generation.get().wrapping_add(1);
            generation.set(stamp);
            if let Some(on_copy) = on_copy {
                on_copy();
            }
            sleep(Duration::from_secs(2)).await;
            if generation.get() == stamp {
                copied.set(false);
            }
            Ok(())
        });
    });
    tooltip(copy).attach(control)
}

#[allow(
    clippy::too_many_arguments,
    reason = "The renderer shares the resolved options for all code rows."
)]
fn render_lines(
    code: &str,
    language: &CodeLanguageInput,
    numbered: bool,
    highlighted: &[u32],
    wrapped: bool,
    diff: bool,
    size: CodeBlockSize,
    palette: Value<SyntaxColors>,
) -> Node {
    let lines = code.split_terminator('\n').collect::<Vec<_>>();
    let digits = lines.len().max(1).to_string().len();
    let rows = lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let line = line.trim_end_matches('\r');
            let file_header = (line.starts_with("--- ")
                && lines.get(index + 1).is_some_and(|s| s.starts_with("+++ "))
                && lines.get(index + 2).is_some_and(|s| s.starts_with("@@")))
                || (line.starts_with("+++ ")
                    && index > 0
                    && lines[index - 1].starts_with("--- ")
                    && lines.get(index + 1).is_some_and(|s| s.starts_with("@@")));
            let header = diff && (file_header || line.starts_with("@@"));
            let marker = if diff && !header {
                line.chars()
                    .next()
                    .filter(|ch| matches!(ch, '+' | '-' | ' '))
            } else {
                None
            };
            let source = if marker.is_some() { &line[1..] } else { line };
            let source = source.replace('\t', "  ");
            let highlighted = highlighted.contains(&((index + 1) as u32));
            let ranges = if header {
                Vec::new()
            } else {
                language.tokenize(&source)
            };
            let code = rich_text(move || render_spans(&source, &ranges, &palette.get())).style(
                move || {
                    style()
                        .text_wrap(wrapped)
                        .min_width(0)
                        .grow(1)
                        .color(if header {
                            palette.get().punctuation
                        } else {
                            palette.get().variable
                        })
                },
            );
            let gutter = numbered.then(|| {
                text(format!("{:>digits$}", index + 1)).style(move || {
                    let p = tokens();
                    style()
                        .shrink(0)
                        .padding_right(p.spacing.s3)
                        .margin_right(p.spacing.s4)
                        .text_align(TextAlign::End)
                        .text_wrap(false)
                        .border_right(p.sizes.border_width, p.colors.border)
                        .color(palette.get().punctuation)
                })
            });
            let marker_text = marker.map(|marker| {
                text(marker.to_string()).style(move || {
                    style()
                        .margin_right(tokens().spacing.s2)
                        .color(palette.get().punctuation)
                })
            });
            row((gutter, marker_text, code)).style(move || {
                let p = tokens();
                let c = palette.get();
                let look = style().align_items(Align::Start).padding_x(p.spacing.s4);
                if highlighted {
                    look.background(p.colors.accent_muted)
                } else if header {
                    look.background(c.diff_header)
                } else {
                    match marker {
                        Some('+') => look.background(c.added),
                        Some('-') => look.background(c.removed),
                        _ => look,
                    }
                }
            })
        })
        .collect::<Vec<_>>();
    column(rows).style(move || {
        let p = tokens();
        let metrics = p.typography.code;
        style()
            .padding_y(p.spacing.s3)
            .font_family(metrics.family)
            .font_size(if size == CodeBlockSize::Sm {
                p.typography.supporting.size_px
            } else {
                metrics.size_px
            })
            .line_height(metrics.line_height_px / metrics.size_px)
            .min_width(0)
    })
}

fn render_spans(
    source: &str,
    ranges: &[(Range<usize>, TokenKind)],
    palette: &SyntaxColors,
) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut offset = 0;
    for (range, kind) in ranges {
        if range.start < offset || range.is_empty() {
            continue;
        }
        let (Some(before), Some(token)) =
            (source.get(offset..range.start), source.get(range.clone()))
        else {
            continue;
        };
        if !before.is_empty() {
            spans.push(span(before));
        }
        let color = match kind {
            TokenKind::Keyword => palette.keyword,
            TokenKind::Type => palette.r#type,
            TokenKind::String => palette.string,
            TokenKind::Comment => palette.comment,
            TokenKind::Operator => palette.operator,
            TokenKind::Punctuation => palette.punctuation,
            TokenKind::Number => palette.number,
            TokenKind::Constant => palette.constant,
            TokenKind::Function => palette.function,
            TokenKind::Variable => palette.variable,
            TokenKind::Tag => palette.tag,
            TokenKind::Attribute => palette.attribute,
            TokenKind::Property => palette.property,
        };
        spans.push(span(token).color(color));
        offset = range.end;
    }
    if offset < source.len() {
        spans.push(span(&source[offset..]));
    }
    if spans.is_empty() {
        spans.push(span(" "));
    }
    spans
}

fn scan(line: &str, language: &str) -> Vec<(Range<usize>, TokenKind)> {
    const LANGUAGES: &str = "typescript javascript tsx jsx ts js json html xml svg css scss less python py bash sh zsh shell php hack yaml yml markdown md c h cpp c++ cc cxx hpp hh sql rust rs kotlin kt kts swift go golang";
    const KEYWORDS: &str = "as async await break case catch class const continue def default delete do else enum export extends fn for from func function if impl import in interface let match mod move mut new package pass private pub public raise ref return self static struct super switch throw trait try type use var void where while with yield SELECT FROM WHERE INSERT INTO UPDATE DELETE JOIN GROUP BY ORDER LIMIT CREATE TABLE";
    if !LANGUAGES.split_whitespace().any(|name| name == language) {
        return Vec::new();
    }
    let hash_comments = matches!(
        language,
        "python" | "py" | "bash" | "sh" | "zsh" | "shell" | "php" | "yaml" | "yml"
    );
    let mut tokens = Vec::new();
    let mut offset = 0;
    while offset < line.len() {
        let tail = &line[offset..];
        let Some(first) = tail.chars().next() else {
            break;
        };
        let (length, kind) = if tail.starts_with("//")
            || (hash_comments && first == '#')
            || (language == "sql" && tail.starts_with("--"))
        {
            (tail.len(), Some(TokenKind::Comment))
        } else if let Some(comment) = tail.strip_prefix("/*") {
            (
                comment.find("*/").map_or(tail.len(), |end| end + 4),
                Some(TokenKind::Comment),
            )
        } else if matches!(first, '"' | '\'' | '`') {
            let mut escaped = false;
            let mut end = tail.len();
            for (index, character) in tail.char_indices().skip(1) {
                if character == first && !escaped {
                    end = index + character.len_utf8();
                    break;
                }
                escaped = character == '\\' && !escaped;
            }
            let kind = if language == "json" && tail[end..].trim_start().starts_with(':') {
                TokenKind::Property
            } else {
                TokenKind::String
            };
            (end, Some(kind))
        } else if first.is_ascii_digit() {
            (
                tail.bytes()
                    .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.'))
                    .count(),
                Some(TokenKind::Number),
            )
        } else if first.is_alphabetic() || first == '_' {
            let end = tail
                .char_indices()
                .find(|(_, ch)| !ch.is_alphanumeric() && *ch != '_')
                .map_or(tail.len(), |(end, _)| end);
            let word = &tail[..end];
            let kind = if matches!(word, "true" | "false" | "null" | "None" | "nil") {
                Some(TokenKind::Constant)
            } else if KEYWORDS.split_whitespace().any(|keyword| keyword == word) {
                Some(TokenKind::Keyword)
            } else if tail[end..].trim_start().starts_with('(') {
                Some(TokenKind::Function)
            } else if first.is_uppercase() {
                Some(TokenKind::Type)
            } else {
                None
            };
            (end, kind)
        } else {
            let kind = if "+-*/%=!<>&|^~?:".contains(first) {
                Some(TokenKind::Operator)
            } else if "{}()[];,.@".contains(first) {
                Some(TokenKind::Punctuation)
            } else {
                None
            };
            (first.len_utf8(), kind)
        };
        if let Some(kind) = kind {
            tokens.push((offset..offset + length, kind));
        }
        offset += length;
    }
    tokens
}

kivact::state::constant_value!(CodeBlockContainer, CodeBlockSize, CodeBlockWidth);
