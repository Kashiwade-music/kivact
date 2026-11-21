//! Role: Attach canonical row input and render independent item content slots.
use crate::item_list::ItemListMarker;
use crate::item_list::ItemPosition;
use crate::item_list::look::ItemListTheme;
use crate::item_list::look::ItemRowState;
use crate::layer::with_base_recipe_style;
use kivact_core::custom::{Part, part};
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::style::{Style, TextOverflow, style};
use kivact_core::view::Variants;
use kivact_core::view::{
    Element, IntoChildren, IntoView, Node, NodeRef, Role, View, node, node_ref, text,
};
/// A semantic item row; its description and activation share the actual root.
pub struct ItemRow {
    root: Node,
    contents: RowContents,
}
// xtask-allow(state): legacy: title_content, description, description_content, media, actions, header, footer, marker, delegate, position, title_lines, description_lines; absent
struct RowContents {
    title: Value<String>,
    title_content: Option<View>,
    description_content: Option<View>,
    media: Option<View>,
    actions: Option<View>,
    header: Option<View>,
    footer: Option<View>,
    marker: Option<View>,
    delegate: Option<NodeRef>,
    position: Option<(Value<usize>, Value<usize>)>,
    title_lines: Option<Value<Option<u32>>>,
    description_lines: Option<Value<Option<u32>>>,
}
#[track_caller]
pub fn item_row(title: impl IntoText) -> ItemRow {
    let title = title.into_text_value();
    ItemRow {
        root: node().role(Role::ListItem).name(title),
        contents: RowContents {
            title,
            title_content: None,
            description_content: None,
            media: None,
            actions: None,
            header: None,
            footer: None,
            marker: None,
            delegate: None,
            position: None,
            title_lines: None,
            description_lines: None,
        },
    }
}
impl ItemRow {
    pub fn media(mut self, view: impl IntoView) -> Self {
        self.contents.media = Some(view.into_view());
        self
    }
    pub fn actions(mut self, view: impl IntoView) -> Self {
        self.contents.actions = Some(view.into_view());
        self
    }
    pub fn title_content(mut self, view: impl IntoView) -> Self {
        self.contents.title_content = Some(view.into_view());
        self
    }
    pub fn description_content(mut self, view: impl IntoView) -> Self {
        self.contents.description_content = Some(view.into_view());
        self
    }
    pub fn header(mut self, view: impl IntoView) -> Self {
        self.contents.header = Some(view.into_view());
        self
    }
    pub fn footer(mut self, view: impl IntoView) -> Self {
        self.contents.footer = Some(view.into_view());
        self
    }
    /// Custom marker content, used when the enclosing list generates no marker.
    pub fn marker(mut self, view: impl IntoView) -> Self {
        self.contents.marker = Some(view.into_view());
        self
    }
    /// Activate and focus the actual eligible target instead of making the row a tab stop.
    pub fn delegate_to(mut self, target: NodeRef) -> Self {
        self.contents.delegate = Some(target);
        self
    }
    /// Actual zero-based index and count for keyed reactive rows; overrides fixed-child context.
    #[track_caller]
    pub fn position(mut self, index: impl IntoValue<usize>, count: impl IntoValue<usize>) -> Self {
        self.contents.position = Some((index.into_value(), count.into_value()));
        self
    }
    /// Limit actual plain title text; explicit None is unlimited, omission follows theme defaults.
    #[track_caller]
    pub fn title_lines(mut self, lines: impl IntoValue<Option<u32>>) -> Self {
        self.contents.title_lines = Some(lines.into_value());
        self
    }
    /// Limit actual plain description text; custom content keeps its own layout.
    #[track_caller]
    pub fn description_lines(mut self, lines: impl IntoValue<Option<u32>>) -> Self {
        self.contents.description_lines = Some(lines.into_value());
        self
    }
}
impl Element for ItemRow {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for ItemRow {
    fn into_view(self) -> View {
        let Self { root, contents } = self;
        // The press and the description are the outer root's: the part's host
        // is a node of its own, so they are read before the part is made.
        let pressable = root.authored().has_press();
        let description = root.authored().description();
        let own = root.authored().variants();
        root.part(Part::new(().into_value(), move |root| {
            contents.render(root, pressable, description, own)
        }))
        .into_view()
    }
}
struct RowRender {
    state: Value<ItemRowState>,
    variants: Variants,
}
impl RowRender {
    fn wrap(
        &self,
        view: impl IntoView,
        pick: fn(&ItemListTheme) -> &crate::recipe::Recipe<ItemRowState>,
    ) -> Node {
        self.wrap_children((view,), pick)
    }
    fn wrap_children(
        &self,
        children: impl IntoChildren,
        pick: fn(&ItemListTheme) -> &crate::theme::Recipe<ItemRowState>,
    ) -> Node {
        let variants = self.variants.clone();
        let state = self.state;
        with_base_recipe_style(
            node().child(children.into_views()),
            pick,
            &variants,
            move || state.get(),
        )
    }
}
impl RowContents {
    fn render(
        mut self,
        mut root: Node,
        pressable: bool,
        description: Option<Value<String>>,
        own: Variants,
    ) -> Node {
        let context = crate::item_list::context::current();
        let mut variants = context.variants.clone();
        variants.merge(&own);
        let position = self.position;
        let fallback = context.position;
        let position = move || {
            position
                .map(|(index, count)| ItemPosition {
                    index: index.get(),
                    count: count.get(),
                })
                .or(fallback)
        };
        if self.position.is_some() || fallback.is_some() {
            root = root
                .pos_in_set(move || {
                    position().map_or(0, |p| {
                        p.index.saturating_add(1).min(u32::MAX as usize) as u32
                    })
                })
                .set_size(move || position().map_or(0, |p| p.count.min(u32::MAX as usize) as u32));
        }
        // The delegate replaces the press.
        let actionable = pressable || self.delegate.is_some();
        if let Some(target) = self.delegate {
            root = root.on_press(move || target.activate());
        } else if pressable {
            root = root.focusable(true);
        }
        let has_description = description.is_some() || self.description_content.is_some();
        let (title_lines, description_lines) = (self.title_lines, self.description_lines);
        let part = part(move |input| {
            let tokens = kivact_core::style::use_theme()
                .with(|t| t.ext::<crate::item_list::look::ItemListTheme>().clone());
            ItemRowState {
                input,
                has_description,
                actionable,
                position: position(),
                marker: context.marker.get(),
                start: context.start.get(),
                title_lines: title_lines.map_or(*tokens.title_lines_value(), |lines| lines.get()),
                description_lines: description_lines
                    .map_or(*tokens.description_lines_value(), |lines| lines.get()),
            }
        });
        let render = RowRender {
            state: part.state(),
            variants,
        };
        let (content, description_ref) = self.content(&render, description);
        if let Some(reference) = description_ref {
            root = root.described_by(reference);
        }
        let marker = render_marker(self.marker, &render);
        let media = self
            .media
            .map(|v| render.wrap(v, |r: &ItemListTheme| r.media_recipe()));
        let actions = self
            .actions
            .map(|v| render.wrap(v, |r: &ItemListTheme| r.actions_recipe()));
        let header = self
            .header
            .map(|v| render.wrap(v, |r: &ItemListTheme| r.header_recipe()));
        let footer = self
            .footer
            .map(|v| render.wrap(v, |r: &ItemListTheme| r.footer_recipe()));
        let body = render.wrap_children((marker, media, content, actions), |r: &ItemListTheme| {
            r.body_recipe()
        });
        let state = render.state;
        with_base_recipe_style(
            root.part(part),
            |r: &ItemListTheme| r.item_recipe(),
            &render.variants,
            move || state.get(),
        )
        .child((header, body, footer))
    }
    fn content(
        &mut self,
        render: &RowRender,
        description: Option<Value<String>>,
    ) -> (Node, Option<NodeRef>) {
        let state = render.state;
        let title = self.title;
        let title = render.wrap(
            self.title_content.take().unwrap_or_else(|| {
                text(title)
                    .style(move || line_style(state.get().title_lines))
                    .into_view()
            }),
            |r: &ItemListTheme| r.title_recipe(),
        );
        let reference = node_ref();
        let description = self
            .description_content
            .take()
            .or_else(|| {
                description.map(|value| {
                    text(value)
                        .style(move || line_style(state.get().description_lines))
                        .into_view()
                })
            })
            .map(|view| {
                render
                    .wrap(view, |r: &ItemListTheme| r.description_recipe())
                    .node_ref(reference)
            });
        let reference = description.is_some().then_some(reference);
        (
            render.wrap_children((title, description), |r: &ItemListTheme| r.content_recipe()),
            reference,
        )
    }
}
fn line_style(lines: Option<u32>) -> Style {
    match lines {
        Some(lines) => style()
            .line_clamp(lines.max(1))
            .text_overflow(TextOverflow::Ellipsis),
        None => style(),
    }
}
fn render_marker(custom: Option<View>, render: &RowRender) -> Node {
    let state = render.state;
    let has_custom = custom.is_some();
    render.wrap(
        node()
            .hidden(move || !has_custom && state.get().marker == ItemListMarker::None)
            .child(
                node()
                    .hidden(move || state.get().marker != ItemListMarker::None)
                    .child(custom),
            )
            .child(
                text(move || {
                    let s = state.get();
                    match s.marker {
                        ItemListMarker::None => String::new(),
                        ItemListMarker::Disc => "•".into(),
                        ItemListMarker::Circle => "◦".into(),
                        ItemListMarker::Decimal => format!(
                            "{}.",
                            u64::from(s.start) + s.position.map_or(0, |p| p.index) as u64
                        ),
                    }
                })
                .hidden(move || state.get().marker == ItemListMarker::None),
            ),
        |r: &ItemListTheme| r.marker_recipe(),
    )
}
