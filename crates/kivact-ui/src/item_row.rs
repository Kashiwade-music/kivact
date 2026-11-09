//! Role: Attach canonical row input and render independent item content slots.
use crate::collection_values::{ItemListMarker, ItemPosition, ItemRowState};
use crate::layer::with_base_recipe_style;
use crate::variants::{Variant, Variants};
use kivact_core::custom::{Part, part};
use kivact_core::state::{Callback, IntoText, IntoValue, Value, callback};
use kivact_core::style::{Style, TextOverflow, style};
use kivact_core::view::{
    HandlerResult, IntoChildren, IntoView, Node, NodeBuilder, NodeRef, Role, View, node, node_ref,
    text,
};
/// A semantic item row; its description and activation share the actual root.
pub struct ItemRow {
    root: Node,
    contents: RowContents,
}
struct RowContents {
    title: Value<String>,
    // xtask-allow(state-fields): passive record - title content
    title_content: Option<View>,
    // xtask-allow(state-fields): passive record - description
    description: Option<Value<String>>,
    // xtask-allow(state-fields): passive record - description content
    description_content: Option<View>,
    // xtask-allow(state-fields): passive record - media
    media: Option<View>,
    // xtask-allow(state-fields): passive record - actions
    actions: Option<View>,
    // xtask-allow(state-fields): passive record - header
    header: Option<View>,
    // xtask-allow(state-fields): passive record - footer
    footer: Option<View>,
    // xtask-allow(state-fields): passive record - marker
    marker: Option<View>,
    press: Option<Callback<()>>,
    // xtask-allow(state-fields): passive record - delegate
    delegate: Option<NodeRef>,
    // xtask-allow(state-fields): passive record - position
    position: Option<(Value<usize>, Value<usize>)>,
    // xtask-allow(state-fields): passive record - title lines
    title_lines: Option<Value<Option<u32>>>,
    // xtask-allow(state-fields): passive record - description lines
    description_lines: Option<Value<Option<u32>>>,
    variants: Variants,
}
#[track_caller]
pub fn item_row(title: impl IntoText) -> ItemRow {
    let title = title.into_text_value();
    ItemRow {
        root: node().role(Role::ListItem).name(title),
        contents: RowContents {
            title,
            title_content: None,
            description: None,
            description_content: None,
            media: None,
            actions: None,
            header: None,
            footer: None,
            marker: None,
            press: None,
            delegate: None,
            position: None,
            title_lines: None,
            description_lines: None,
            variants: Variants::default(),
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
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.contents.description = Some(text.into_text_value());
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
    pub fn on_press<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.contents.press = Some(callback(move |()| f()));
        self
    }
    /// Activate and focus the actual eligible target instead of making the row a tab stop.
    pub fn delegate_to(mut self, target: NodeRef) -> Self {
        self.contents.delegate = Some(target);
        self
    }
    /// Actual zero-based index and count for keyed reactive rows; overrides fixed-child context.
    pub fn position(mut self, index: impl IntoValue<usize>, count: impl IntoValue<usize>) -> Self {
        self.contents.position = Some((index.into_value(), count.into_value()));
        self
    }
    /// Limit actual plain title text; explicit None is unlimited, omission follows theme defaults.
    pub fn title_lines(mut self, lines: impl IntoValue<Option<u32>>) -> Self {
        self.contents.title_lines = Some(lines.into_value());
        self
    }
    /// Limit actual plain description text; custom content keeps its own layout.
    pub fn description_lines(mut self, lines: impl IntoValue<Option<u32>>) -> Self {
        self.contents.description_lines = Some(lines.into_value());
        self
    }
    pub fn variant(mut self, value: impl Variant) -> Self {
        self.contents.variants.add(value);
        self
    }
}
impl NodeBuilder for ItemRow {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for ItemRow {
    fn into_view(self) -> View {
        let Self { root, contents } = self;
        root.part(Part::new(().into_value(), move |root| {
            contents.render(root)
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
        pick: fn(&crate::recipes::UiRecipes) -> &crate::recipes::Recipe<ItemRowState>,
    ) -> Node {
        self.wrap_children((view,), pick)
    }
    fn wrap_children(
        &self,
        children: impl IntoChildren,
        pick: fn(&crate::recipes::UiRecipes) -> &crate::Recipe<ItemRowState>,
    ) -> Node {
        let state = self.state;
        with_base_recipe_style(
            node().child(children.into_views()),
            pick,
            &self.variants,
            move || state.get(),
        )
    }
}
impl RowContents {
    fn render(mut self, mut root: Node) -> Node {
        let context = crate::item_context::current();
        let variants = self.variants.clone().with_defaults(&context.variants);
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
        let actionable = self.press.is_some() || self.delegate.is_some();
        if let Some(target) = self.delegate {
            root = root.on_press(move || target.activate());
        } else if let Some(press) = self.press {
            root = root.focusable(true).on_press(move || press.call(()));
        }
        let has_description = self.description.is_some() || self.description_content.is_some();
        let (title_lines, description_lines) = (self.title_lines, self.description_lines);
        let part = part(move |input| {
            let tokens = kivact_core::style::use_tokens::<crate::UiTokens>();
            ItemRowState {
                input,
                has_description,
                actionable,
                position: position(),
                marker: context.marker.get(),
                start: context.start.get(),
                title_lines: title_lines.map_or(tokens.item_list.title_lines, |lines| lines.get()),
                description_lines: description_lines
                    .map_or(tokens.item_list.description_lines, |lines| lines.get()),
            }
        });
        let render = RowRender {
            state: part.state(),
            variants,
        };
        let (content, description_ref) = self.content(&render);
        if let Some(reference) = description_ref {
            root = root.described_by(reference);
        }
        let marker = render_marker(self.marker, &render);
        let media = self
            .media
            .map(|v| render.wrap(v, |r| r.item_list_recipes().media_recipe()));
        let actions = self
            .actions
            .map(|v| render.wrap(v, |r| r.item_list_recipes().actions_recipe()));
        let header = self
            .header
            .map(|v| render.wrap(v, |r| r.item_list_recipes().header_recipe()));
        let footer = self
            .footer
            .map(|v| render.wrap(v, |r| r.item_list_recipes().footer_recipe()));
        let body = render.wrap_children((marker, media, content, actions), |r| {
            r.item_list_recipes().body_recipe()
        });
        let state = render.state;
        with_base_recipe_style(
            root.part(part),
            |r| r.item_list_recipes().item_recipe(),
            &render.variants,
            move || state.get(),
        )
        .child((header, body, footer))
    }
    fn content(&mut self, render: &RowRender) -> (Node, Option<NodeRef>) {
        let state = render.state;
        let title = self.title;
        let title = render.wrap(
            self.title_content.take().unwrap_or_else(|| {
                text(title)
                    .style(move || line_style(state.get().title_lines))
                    .into_view()
            }),
            |r| r.item_list_recipes().title_recipe(),
        );
        let reference = node_ref();
        let description = self
            .description_content
            .take()
            .or_else(|| {
                self.description.take().map(|value| {
                    text(value)
                        .style(move || line_style(state.get().description_lines))
                        .into_view()
                })
            })
            .map(|view| {
                render
                    .wrap(view, |r| r.item_list_recipes().description_recipe())
                    .node_ref(reference)
            });
        let reference = description.is_some().then_some(reference);
        (
            render.wrap_children((title, description), |r| {
                r.item_list_recipes().content_recipe()
            }),
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
        |r| r.item_list_recipes().marker_recipe(),
    )
}
