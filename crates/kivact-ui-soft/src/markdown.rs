//! Role: Render Markdown blocks and inline marks with Neutral components.
use crate::markdown_parser::{Alignment, Block, ListEntry, parse, reference};
use crate::{
    CitationSource, CitationVariant, CodeLanguage, CodeLanguageInput, blockquote, citation,
    code_block, heading, tokens,
};
use kivact::control::{
    Node, NodeBuilder, ScrollAxes, Span, column, node, rich_text, row, scroll, span, text,
};
use kivact::i18n::{message, message_with};
use kivact::primitive::{NodeExt, part};
use kivact::style::{Align, Direction, Justify, TextAlign, TextDecorationLine, pct, style};
use kivact::task::spawn;
use kivact::ui::{ButtonVariant, ControlSize, button};
use kivact::view::{IntoValue, IntoView, Role, View, dynamic};
use kivact::{Value, memo};
use std::collections::HashMap;
use std::rc::Rc;

/// Vertical spacing between Markdown blocks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkdownDensity {
    #[default]
    Default,
    Compact,
}
/// How the bounded content aligns within its available width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ContentAlign {
    #[default]
    Start,
    Center,
}
/// Enable copying permalinks from headings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HeadingLinks {
    pub id_prefix: String,
    pub base_url: String,
}

type LinkAction = Rc<dyn Fn(&str)>;
type RenderCode = Rc<dyn Fn(&str, Option<&str>) -> View>;
type RenderImage = Rc<dyn Fn(&str, &str) -> View>;
type RenderHeading = Rc<dyn Fn(u32, &str) -> View>;

/// A reactive Markdown document or inline fragment.
pub struct Markdown {
    root: Node,
    source: Value<String>,
    sources: Value<Vec<(String, CitationSource)>>,
    streaming: Value<bool>,
    options: Options,
}

struct Options {
    inline: bool,
    density: MarkdownDensity,
    level: u32,
    on_link: Option<LinkAction>,
    citation_style: CitationVariant,
    width: Option<f32>,
    align: ContentAlign,
    heading_links: Option<HeadingLinks>,
    soft_breaks: bool,
    autolink: bool,
    code_languages: Vec<CodeLanguage>,
    render_code: Option<RenderCode>,
    render_image: Option<RenderImage>,
    render_heading: Option<RenderHeading>,
}

/// Render Markdown source.
pub fn markdown(source: impl IntoValue<String>) -> Markdown {
    Markdown {
        root: node(),
        source: source.into_value(),
        sources: Vec::new().into_value(),
        streaming: false.into_value(),
        options: Options {
            inline: false,
            density: MarkdownDensity::Default,
            level: 1,
            on_link: None,
            citation_style: CitationVariant::Label,
            width: Some(680.0),
            align: ContentAlign::Start,
            heading_links: None,
            soft_breaks: false,
            autolink: false,
            code_languages: Vec::new(),
            render_code: None,
            render_image: None,
            render_heading: None,
        },
    }
}

impl Markdown {
    /// Render inline markup only.
    pub fn inline(mut self, value: bool) -> Self {
        self.options.inline = value;
        self
    }
    /// Set the vertical density.
    pub fn density(mut self, value: MarkdownDensity) -> Self {
        self.options.density = value;
        self
    }
    /// Remap the first heading level.
    pub fn heading_level_start(mut self, value: u32) -> Self {
        self.options.level = value.clamp(1, 6);
        self
    }
    /// Treat the source as an incomplete stream.
    pub fn streaming(mut self, value: impl IntoValue<bool>) -> Self {
        self.streaming = value.into_value();
        self
    }
    /// Notify the application when a safe link is activated.
    pub fn on_link(mut self, value: impl Fn(&str) + 'static) -> Self {
        self.options.on_link = Some(Rc::new(value));
        self
    }
    /// Supply source metadata for inline citation IDs.
    pub fn sources(mut self, value: impl IntoValue<Vec<(String, CitationSource)>>) -> Self {
        self.sources = value.into_value();
        self
    }
    /// Select how source references appear.
    pub fn citation_style(mut self, value: CitationVariant) -> Self {
        self.options.citation_style = value;
        self
    }
    /// Set the prose width limit.
    pub fn content_width(mut self, value: Option<f32>) -> Self {
        self.options.width = value;
        self
    }
    /// Align the bounded prose.
    pub fn content_align(mut self, value: ContentAlign) -> Self {
        self.options.align = value;
        self
    }
    /// Add permalink controls to headings.
    pub fn heading_links(mut self, value: HeadingLinks) -> Self {
        self.options.heading_links = Some(value);
        self
    }
    /// Preserve source soft breaks as newlines.
    pub fn soft_breaks(mut self, value: bool) -> Self {
        self.options.soft_breaks = value;
        self
    }
    /// Recognize bare web and email links.
    pub fn autolink(mut self, value: bool) -> Self {
        self.options.autolink = value;
        self
    }
    /// Supply custom fenced-code languages.
    pub fn code_languages(mut self, value: Vec<CodeLanguage>) -> Self {
        self.options.code_languages = value;
        self
    }
    /// Replace the fenced code renderer.
    pub fn render_code<V: IntoView>(
        mut self,
        value: impl Fn(&str, Option<&str>) -> V + 'static,
    ) -> Self {
        self.options.render_code = Some(Rc::new(move |source, language| {
            value(source, language).into_view()
        }));
        self
    }
    /// Replace the image renderer.
    pub fn render_image<V: IntoView>(mut self, value: impl Fn(&str, &str) -> V + 'static) -> Self {
        self.options.render_image =
            Some(Rc::new(move |source, alt| value(source, alt).into_view()));
        self
    }
    /// Replace the heading renderer.
    pub fn render_heading<V: IntoView>(mut self, value: impl Fn(u32, &str) -> V + 'static) -> Self {
        self.options.render_heading =
            Some(Rc::new(move |level, title| value(level, title).into_view()));
        self
    }
}

impl NodeBuilder for Markdown {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Markdown {
    fn into_view(self) -> View {
        let Self {
            root,
            source,
            sources,
            streaming,
            options,
        } = self;
        let inline = options.inline;
        let input = memo(move || (source.get(), sources.get(), streaming.get(), tokens()));
        root.base_style(move || {
            let look = style().direction(Direction::Column).min_width(0);
            if inline {
                return look;
            }
            let p = tokens();
            let body = p.typography.body;
            look.width(pct(100))
                .font_family(body.family)
                .font_size(body.size_px)
                .line_height(body.line_height_px / body.size_px)
                .color(p.colors.text_primary)
        })
        .child(dynamic(input, move |(source, sources, streaming, _)| {
            let source = if streaming {
                settled_tail(&source)
            } else {
                &source
            };
            let lines = source.lines().collect::<Vec<_>>();
            let references = lines
                .iter()
                .filter_map(|line| reference(line))
                .map(|(label, url)| (label.to_lowercase(), url.to_owned()))
                .collect();
            let mut renderer = Renderer {
                options: &options,
                sources: &sources,
                references,
                numbers: HashMap::new(),
                slugs: HashMap::new(),
            };
            if options.inline {
                renderer.inline(source, false, true)
            } else {
                renderer
                    .blocks(&parse(&lines, options.soft_breaks, 0), 0)
                    .into_view()
            }
        }))
        .into_view()
    }
}

struct Renderer<'a> {
    options: &'a Options,
    sources: &'a [(String, CitationSource)],
    references: HashMap<String, String>,
    numbers: HashMap<String, u32>,
    slugs: HashMap<String, usize>,
}

impl Renderer<'_> {
    fn blocks(&mut self, blocks: &[Block<'_>], depth: usize) -> Node {
        let mut previous = 0.0f32;
        let children = blocks
            .iter()
            .enumerate()
            .map(|(index, block)| {
                let (top, bottom) = self.spacing(block);
                let gap = if index == 0 { 0.0 } else { previous.max(top) };
                previous = bottom;
                let view = self.block(block, depth);
                let mut look = style().margin_top(gap).min_width(0);
                if depth == 0 && !matches!(block, Block::Table(..) | Block::Code(..) | Block::Rule)
                {
                    if let Some(width) = self.options.width {
                        look = look.max_width(width).width(pct(100));
                    }
                    look = look.align_self(match self.options.align {
                        ContentAlign::Start => Align::Start,
                        ContentAlign::Center => Align::Center,
                    });
                }
                node().style(look).child(view)
            })
            .collect::<Vec<_>>();
        column(children)
    }

    fn spacing(&self, block: &Block<'_>) -> (f32, f32) {
        let p = tokens();
        let s = p.spacing;
        let compact = self.options.density == MarkdownDensity::Compact;
        match block {
            Block::Heading(level, _) if level + self.options.level - 1 <= 3 => {
                if compact {
                    (s.s4, s.s2)
                } else {
                    (s.s6, s.s3)
                }
            }
            Block::Heading(..) => {
                if compact {
                    (s.s3, s.s1)
                } else {
                    (s.s4, s.s2)
                }
            }
            Block::Rule => {
                let gap = if compact { s.s3 } else { s.s6 };
                (gap, gap)
            }
            Block::Code(..) | Block::Table(..) | Block::Quote(..) => {
                let gap = if compact { s.s2 } else { s.s4 };
                (gap, gap)
            }
            _ => {
                let gap = if compact { s.s1 } else { s.s3 };
                (gap, gap)
            }
        }
    }

    fn block(&mut self, block: &Block<'_>, depth: usize) -> View {
        match block {
            Block::Heading(level, title) => {
                self.heading((*level + self.options.level - 1).min(6), title)
            }
            Block::Paragraph(source) => self.inline(source, false, true),
            Block::Quote(blocks) => {
                let content = self.blocks(blocks, depth + 1).into_view();
                blockquote(move || content).into_view()
            }
            Block::Code(language, source) => {
                let (language, diff) = if *language == "diff" {
                    ("plaintext", true)
                } else if let Some(language) = language.strip_prefix("diff-") {
                    (language, true)
                } else {
                    (*language, false)
                };
                if let Some(render) = &self.options.render_code {
                    return render(source, (!language.is_empty()).then_some(language));
                }
                let syntax = self
                    .options
                    .code_languages
                    .iter()
                    .find(|item| {
                        item.identifiers
                            .iter()
                            .any(|id| id.eq_ignore_ascii_case(language))
                    })
                    .map_or_else(
                        || {
                            CodeLanguageInput::from(if language.is_empty() {
                                "plaintext"
                            } else {
                                language
                            })
                        },
                        |item| item.clone().into(),
                    );
                code_block(source.to_owned())
                    .language(syntax)
                    .diff(diff)
                    .collapsible(true)
                    .wrap(true)
                    .into_view()
            }
            Block::Rule => node()
                .style(|| {
                    let p = tokens();
                    style().border_top(p.sizes.border_width, p.colors.border)
                })
                .into_view(),
            Block::List(entries) => self.list(entries, depth).into_view(),
            Block::Table(rows, alignment) => self.table(rows, alignment).into_view(),
        }
    }

    fn heading(&mut self, level: u32, title: &str) -> View {
        let view = if let Some(render) = &self.options.render_heading {
            render(level, title)
        } else {
            heading(level, title).into_view()
        };
        let Some(links) = &self.options.heading_links else {
            return view;
        };
        let slug = slug(title);
        let count = self.slugs.entry(slug.clone()).or_insert(0);
        let suffix = if *count == 0 {
            String::new()
        } else {
            format!("-{count}")
        };
        *count += 1;
        let url = format!("{}#{}{}{}", links.base_url, links.id_prefix, slug, suffix);
        let title = title.to_owned();
        let label = message_with(
            "soft.markdown_heading_links.copy",
            "Copy link to {heading}",
            move || [("heading", title.clone())],
        );
        let copy = button(label)
            .variant(ButtonVariant::Ghost)
            .size(ControlSize::Small)
            .hide_label(true)
            .icon(text("#"))
            .on_press(move || {
                let copy = kivact::platform::copy_text(url.clone());
                spawn(copy);
            })
            .style(
                style()
                    .opacity(0.0)
                    .group_hover(style().opacity(1.0))
                    .group_focus(style().opacity(1.0)),
            );
        row((view, copy))
            .style(|| {
                style()
                    .group()
                    .align_items(Align::Center)
                    .gap(tokens().spacing.s1_5)
            })
            .into_view()
    }

    fn list(&mut self, entries: &[ListEntry<'_>], depth: usize) -> Node {
        let all_tasks = entries.iter().all(|item| item.checked.is_some());
        let items = entries
            .iter()
            .map(|item| {
                let marker = match (item.checked, item.number) {
                    (Some(checked), _) => text(if checked { "☑" } else { "☐" })
                        .role(Role::CheckBox)
                        .checked(checked),
                    (None, Some(number)) => {
                        text(format!("{number}.")).style(style().tabular_nums())
                    }
                    _ => text(match depth % 3 {
                        1 => "◦",
                        2 => "▪",
                        _ => "•",
                    }),
                }
                .style(|| {
                    style()
                        .min_width(tokens().spacing.s4)
                        .text_align(TextAlign::End)
                });
                row((
                    marker,
                    self.blocks(&item.blocks, depth + 1)
                        .style(style().grow(1).min_width(0)),
                ))
                .role(Role::ListItem)
                .style(|| {
                    style()
                        .align_items(Align::Start)
                        .gap(tokens().spacing.s2)
                        .padding_y(tokens().spacing.s1)
                })
            })
            .collect::<Vec<_>>();
        let list = column(items)
            .role(Role::List)
            .style(|| style().gap(tokens().spacing.s0_5));
        if all_tasks {
            list.name(message("soft.markdown.task_list", "Task list"))
        } else {
            list
        }
    }

    fn table(&mut self, rows: &[Vec<&str>], align: &[Alignment]) -> impl IntoView + use<> {
        let mut widths = vec![4usize; align.len()];
        for (index, row) in rows.iter().enumerate() {
            for (column, cell) in row.iter().enumerate() {
                let length = cell.chars().count();
                let min = if index == 0 {
                    length.min(20)
                } else {
                    length.div_ceil(2)
                };
                widths[column] = widths[column].max(min).min(24);
            }
        }
        let rows = rows
            .iter()
            .enumerate()
            .map(|(index, cells)| {
                let cells = cells
                    .iter()
                    .enumerate()
                    .map(|(column, cell)| {
                        let content = self.inline(cell, index == 0, false);
                        let width = widths[column] as f32;
                        let justify = match align[column] {
                            Alignment::Start => Justify::Start,
                            Alignment::Center => Justify::Center,
                            Alignment::End => Justify::End,
                        };
                        node()
                            .style(move || {
                                let p = tokens();
                                style()
                                    .width(width * p.typography.body.size_px * 0.64 + p.spacing.s4)
                                    .direction(Direction::Row)
                                    .justify_content(justify)
                                    .padding(p.spacing.s2)
                                    .border_right(p.sizes.border_width, p.colors.border)
                                    .border_bottom(p.sizes.border_width, p.colors.border)
                            })
                            .child(content)
                    })
                    .collect::<Vec<_>>();
                row(cells)
            })
            .collect::<Vec<_>>();
        scroll(column(rows).style(|| {
            let p = tokens();
            style()
                .border_left(p.sizes.border_width, p.colors.border)
                .border_top(p.sizes.border_width, p.colors.border)
        }))
        .axes(ScrollAxes::Horizontal)
        .name(message("soft.table.label", "Table"))
    }

    fn inline(&mut self, source: &str, strong: bool, paragraph: bool) -> View {
        let mut parts = Vec::<Result<String, View>>::new();
        let mut plain = String::new();
        let mut rest = source;
        while !rest.is_empty() {
            if let Some((used, view)) = self.inline_object(rest) {
                if !plain.is_empty() {
                    parts.push(Ok(std::mem::take(&mut plain)));
                }
                parts.push(Err(view));
                rest = &rest[used..];
            } else {
                let Some(character) = rest.chars().next() else {
                    break;
                };
                plain.push(character);
                rest = &rest[character.len_utf8()..];
                if character == '\\'
                    && let Some(next) = rest.chars().next()
                {
                    plain.push(next);
                    rest = &rest[next.len_utf8()..];
                }
            }
        }
        if parts.is_empty() {
            return inline_spans(&plain, strong).into_view();
        }
        if !plain.is_empty() {
            parts.push(Ok(plain));
        }
        // Keep marked, escaped and entity-containing spans on the existing renderer.
        if paragraph
            && !strong
            && parts.iter().all(|part| {
                !part
                    .as_ref()
                    .is_ok_and(|text| text.contains(['*', '_', '~', '`', '\\', '&']))
            })
        {
            let p = tokens();
            let mut text = String::new();
            let mut objects = Vec::new();
            for part in parts {
                match part {
                    Ok(plain) => text.push_str(&plain),
                    Err(view) => objects.push((text.len(), view)),
                }
            }
            let typography = kivact::draw::TextStyle {
                family: Some(p.typography.family_body.into()),
                size: p.typography.body.size_px,
                ..Default::default()
            };
            return kivact::primitive::inline_view(text, typography, objects).into_view();
        }
        let nodes: Vec<_> = parts
            .into_iter()
            .map(|part| match part {
                Ok(plain) => inline_spans(&plain, strong).into_view(),
                Err(view) => view,
            })
            .collect();
        row(nodes)
            .style(style().wrap(true).align_items(Align::Baseline).min_width(0))
            .into_view()
    }

    fn inline_object(&mut self, rest: &str) -> Option<(usize, View)> {
        let image = rest.starts_with("![");
        if let Some(label) = rest.strip_prefix(if image { "![" } else { "[" }) {
            let (title, after) = label.split_once(']')?;
            let used = rest.len() - after.len();
            let (url, extra) = if let Some(destination) = after.strip_prefix('(') {
                let (url, remaining) = destination.split_once(')')?;
                (
                    url.split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_matches(['<', '>'])
                        .to_owned(),
                    after.len() - remaining.len(),
                )
            } else if let Some(reference) = after.strip_prefix('[') {
                let (label, remaining) = reference.split_once(']')?;
                let label = if label.is_empty() { title } else { label };
                (
                    self.references.get(&label.to_lowercase())?.clone(),
                    after.len() - remaining.len(),
                )
            } else if let Some(url) = self.references.get(&title.to_lowercase()) {
                (url.clone(), 0)
            } else {
                return (!image)
                    .then(|| self.citation(title))
                    .flatten()
                    .map(|view| (used, view));
            };
            if image {
                let fallback = || text(format!("[{title}]")).into_view();
                let rendered = if safe_url(&url) {
                    self.options
                        .render_image
                        .as_ref()
                        .map_or_else(fallback, |render| render(&url, title))
                } else {
                    fallback()
                };
                return Some((used + extra, rendered));
            }
            return Some((used + extra, self.link(title, url)));
        }
        if let Some(candidate) = rest.strip_prefix('【') {
            let (id, after) = candidate.split_once('】')?;
            return self
                .citation(id)
                .map(|view| (rest.len() - after.len(), view));
        }
        if self.options.autolink {
            let (candidate, used) = if let Some(candidate) = rest.strip_prefix('<') {
                let (candidate, after) = candidate.split_once('>')?;
                (candidate, rest.len() - after.len())
            } else {
                let candidate = rest
                    .split_whitespace()
                    .next()?
                    .trim_end_matches(['?', '!', '.', ',', ':', '*', '_', '~', ')']);
                (candidate, candidate.len())
            };
            let url = if candidate.starts_with("http://") || candidate.starts_with("https://") {
                candidate.to_owned()
            } else if candidate.starts_with("www.") {
                format!("https://{candidate}")
            } else if candidate.contains('@')
                && candidate
                    .split_once('@')
                    .is_some_and(|(user, domain)| !user.is_empty() && domain.contains('.'))
            {
                format!("mailto:{candidate}")
            } else {
                return None;
            };
            return Some((used, self.link(candidate, url)));
        }
        None
    }

    fn citation(&mut self, id: &str) -> Option<View> {
        let (_, source) = self.sources.iter().find(|(source, _)| source == id)?;
        let next = self.numbers.len() as u32 + 1;
        let number = *self.numbers.entry(id.to_owned()).or_insert(next);
        let cite = citation(number, source.clone()).variant(self.options.citation_style);
        Some(if let Some(action) = self.options.on_link.clone() {
            cite.on_open(move |url| {
                if safe_url(url) {
                    action(url);
                }
            })
            .into_view()
        } else {
            cite.into_view()
        })
    }

    fn link(&self, label: &str, url: String) -> View {
        if !safe_url(&url) {
            return inline_spans(label, false).into_view();
        }
        let mut link = inline_spans(label, false).style(|| {
            style()
                .color(tokens().colors.text_accent)
                .text_decoration(TextDecorationLine::UNDERLINE)
        });
        if let Some(action) = self.options.on_link.clone() {
            link = link.part(
                part(|input| input)
                    .role(Role::Link)
                    .focusable(true)
                    .on_press(move || action(&url)),
            );
        }
        link.into_view()
    }
}

fn inline_spans(source: &str, strong: bool) -> Node {
    let p = tokens();
    let mut spans = Vec::new();
    marked(
        source,
        &mut spans,
        Marks {
            strong,
            ..Marks::default()
        },
        0,
        &p,
    );
    rich_text(spans).style(style().min_width(0))
}

#[derive(Clone, Copy, Default)]
struct Marks {
    strong: bool,
    italic: bool,
    deleted: bool,
    code: bool,
}

fn marked(mut source: &str, spans: &mut Vec<Span>, marks: Marks, depth: usize, p: &crate::Tokens) {
    if depth >= 32 {
        spans.push(styled_span(source, marks, p));
        return;
    }
    let mut plain = String::new();
    while !source.is_empty() {
        if let Some(after) = source.strip_prefix('\\')
            && let Some(next) = after.chars().next()
            && next.is_ascii_punctuation()
        {
            plain.push(next);
            source = &after[next.len_utf8()..];
            continue;
        }
        let found = ["***", "___", "**", "__", "~~", "*", "_", "`"]
            .into_iter()
            .find_map(|marker| {
                let after = source.strip_prefix(marker)?;
                let (content, rest) = after.split_once(marker)?;
                (!content.is_empty()
                    && (marker == "`" || !content.starts_with(char::is_whitespace)))
                .then_some((marker, content, rest))
            });
        if let Some((marker, content, rest)) = found {
            if !plain.is_empty() {
                spans.push(styled_span(&plain, marks, p));
                plain.clear();
            }
            let mut inner = marks;
            match marker {
                "***" | "___" => {
                    inner.strong = true;
                    inner.italic = true;
                }
                "**" | "__" => inner.strong = true,
                "*" | "_" => inner.italic = true,
                "~~" => inner.deleted = true,
                _ => inner.code = true,
            }
            if inner.code {
                spans.push(styled_span(content.trim(), inner, p));
            } else {
                marked(content, spans, inner, depth + 1, p);
            }
            source = rest;
            continue;
        }
        if let Some(entity) = source.strip_prefix('&')
            && let Some((name, rest)) = entity.split_once(';')
            && let Some(character) = decode_entity(name)
        {
            plain.push(character);
            source = rest;
            continue;
        }
        let Some(character) = source.chars().next() else {
            break;
        };
        plain.push(character);
        source = &source[character.len_utf8()..];
    }
    if !plain.is_empty() {
        spans.push(styled_span(&plain, marks, p));
    }
}

fn styled_span(source: &str, marks: Marks, p: &crate::Tokens) -> Span {
    let mut span = span(source);
    if marks.strong {
        span = span.weight(p.typography.weight_semibold);
    }
    if marks.italic {
        span = span.italic(true);
    }
    if marks.deleted {
        span = span
            .text_decoration(TextDecorationLine::LINE_THROUGH)
            .color(p.colors.text_secondary);
    }
    if marks.code {
        span = span
            .font_family(p.typography.family_code)
            .background(p.colors.bg_muted);
    }
    span
}

fn decode_entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "copy" => Some('©'),
        "nbsp" => Some('\u{a0}'),
        _ => {
            let value = name.strip_prefix('#')?;
            let code = if let Some(hex) = value.strip_prefix(['x', 'X']) {
                u32::from_str_radix(hex, 16).ok()?
            } else {
                value.parse().ok()?
            };
            char::from_u32(code)
        }
    }
}

fn safe_url(url: &str) -> bool {
    if url.is_empty() || url.chars().any(char::is_control) {
        return false;
    }
    let url = url.trim().to_ascii_lowercase();
    let Some((scheme, _)) = url.split_once(':') else {
        return true;
    };
    scheme.contains(['/', '?', '#']) || matches!(scheme, "http" | "https" | "mailto" | "tel")
}

fn slug(title: &str) -> String {
    let mut output = String::new();
    for character in title.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
        } else if !matches!(character, '\'' | '"') && !output.is_empty() && !output.ends_with('-') {
            output.push('-');
        }
    }
    if output.ends_with('-') {
        output.pop();
    }
    if output.is_empty() {
        output.push_str("section");
    }
    output
}

fn settled_tail(source: &str) -> &str {
    let start = source.rfind('\n').map_or(0, |index| index + 1);
    let tail = &source[start..];
    let cut = tail
        .rfind('[')
        .filter(|index| !tail[*index..].contains(']'))
        .or_else(|| {
            tail.rfind('`')
                .filter(|_| tail.matches('`').count() % 2 == 1)
        })
        .or_else(|| {
            tail.rfind("~~")
                .filter(|_| tail.matches("~~").count() % 2 == 1)
        });
    if let Some(cut) = cut {
        return &source[..start + cut];
    }
    source.trim_end_matches(['*', '~'])
}
