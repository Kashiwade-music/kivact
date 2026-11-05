//! Role: Render a named, wrapping Neutral breadcrumb trail.
use crate::stack::rtl;

use kivact::state::IntoValue;
use kivact::view::IntoViewFn;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ViewFn;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::row;
use kivact::view::text;

use kivact::interaction::MenuEntry;
use kivact::state::Callback;
use kivact::state::IntoText;
use kivact::state::Value;
use kivact::state::callback;
use kivact::state::provide;
use kivact::style::Align;
use kivact::style::Cursor;
use kivact::style::Direction;
use kivact::style::TextDecorationLine;
use kivact::style::style;
use kivact::ui::menu;
use kivact::view::HandlerResult;
use kivact::view::IntoView;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::node_ref;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BreadcrumbsVariant {
    #[default]
    Default,
    Supporting,
}

pub struct Breadcrumbs {
    root: Node,
    items: Vec<BreadcrumbItem>,
    variant: Value<BreadcrumbsVariant>,
    separator: Option<ViewFn>,
}

pub fn breadcrumbs(items: impl IntoIterator<Item = BreadcrumbItem>) -> Breadcrumbs {
    Breadcrumbs {
        root: node()
            .with_node(move |node| node.role(Role::Navigation))
            .name(kivact::i18n::message(
                "soft.breadcrumbs.label",
                "Breadcrumb",
            )),
        items: items.into_iter().collect(),
        variant: (BreadcrumbsVariant::Default).into_value(),
        separator: None,
    }
}

impl Breadcrumbs {
    pub fn variant(mut self, variant: impl IntoValue<BreadcrumbsVariant>) -> Self {
        self.variant = variant.into_value();
        self
    }
    pub fn separator<V: IntoView>(mut self, separator: impl IntoViewFn<V>) -> Self {
        self.separator = Some(separator.into_view_fn());
        self
    }
}

impl NodeBuilder for Breadcrumbs {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Breadcrumbs {
    fn into_view(self) -> View {
        let Self {
            root,
            items,
            variant,
            separator,
        } = self;
        provide((), move || {
            let explicit = items.iter().any(|item| item.current == Some(true));
            let count = items.len();
            let children = items
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    let current = item.current.unwrap_or(!explicit && index + 1 == count);
                    let separator = if index == 0 {
                        None
                    } else {
                        Some(if let Some(separator) = &separator {
                            separator.call(())
                        } else {
                            text("/")
                                .style(|| style().scale_xy(if rtl() { -1.0 } else { 1.0 }, 1.0))
                                .into_view()
                        })
                    };
                    row((separator, item.build(variant.get(), current)))
                        .with_node(move |node| node.role(Role::ListItem))
                        .style(|| {
                            let t = kivact::style::use_tokens::<crate::Tokens>();
                            style()
                                .direction(logical_row())
                                .align_items(Align::Center)
                                .gap(t.spacing.s1)
                        })
                })
                .collect::<Vec<_>>();
            root.child(
                row(children)
                    .with_node(move |node| node.role(Role::List))
                    .style(move || {
                        let t = kivact::style::use_tokens::<crate::Tokens>();
                        let text = if variant.get() == BreadcrumbsVariant::Default {
                            t.typography.body
                        } else {
                            t.typography.supporting
                        };
                        style()
                            .direction(logical_row())
                            .wrap(true)
                            .align_items(Align::Center)
                            .gap(t.spacing.s1)
                            .font_size(text.size_px)
                            .line_height(text.line_height_px / text.size_px)
                            .color(t.colors.text_secondary)
                    }),
            )
        })
    }
}

type ItemMenu = (Vec<MenuEntry<String>>, Callback<String>);

pub struct BreadcrumbItem {
    root: Node,
    label: Value<String>,
    current: Option<bool>,
    icon: Option<View>,
    action: Option<Callback<()>>,
    menu: Option<ItemMenu>,
}

pub fn breadcrumb_item(label: impl IntoText) -> BreadcrumbItem {
    let label = label.into_text_value();
    BreadcrumbItem {
        root: node(),
        label,
        current: None,
        icon: None,
        action: None,
        menu: None,
    }
}

impl BreadcrumbItem {
    pub fn current(mut self, current: impl Into<Option<bool>>) -> Self {
        self.current = current.into();
        self
    }
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    pub fn on_press(mut self, action: impl Into<Callback<()>>) -> Self {
        let action: Callback<()> = action.into();
        self.action = Some(callback(move |_| action.call(())));
        self
    }
    pub fn menu<R: HandlerResult>(
        mut self,
        entries: Vec<MenuEntry<String>>,
        select: impl Fn(String) -> R + 'static,
    ) -> Self {
        self.menu = Some((entries, callback(select)));
        self
    }

    fn build(self, variant: BreadcrumbsVariant, current: bool) -> View {
        let Self {
            root,
            label,
            icon: leading,
            action,
            menu: popup,
            ..
        } = self;
        let popup = if current { None } else { popup };
        let action = if current || popup.is_some() {
            None
        } else {
            action
        };
        let actionable = popup.is_some() || action.is_some();
        let hover_ref = node_ref();
        let hovered = hover_ref.hovered();
        let title = text(label);
        let title = if current {
            title.with_node(move |node| node.current_item(Some(kivact::view::CurrentItem::Page)))
        } else {
            title
        };
        let content = root
            .node_ref(hover_ref)
            .with_node(move |node| {
                node.base_style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    let base = style()
                        .direction(logical_row())
                        .align_items(Align::Center)
                        .gap(t.spacing.s1);
                    if actionable {
                        let base = base
                            .padding_y(t.spacing.s1)
                            .cursor(Cursor::Pointer)
                            .color(t.colors.text_secondary)
                            .focus_visible(crate::recipe_style::ring(&t, t.colors.accent));
                        if hovered.get() {
                            base.text_decoration(TextDecorationLine::UNDERLINE)
                        } else {
                            base
                        }
                    } else {
                        base.weight(t.typography.weight_semibold).color(
                            if variant == BreadcrumbsVariant::Default {
                                t.colors.text_primary
                            } else {
                                t.colors.text_secondary
                            },
                        )
                    }
                })
            })
            .child((
                leading,
                title,
                popup.as_ref().map(|_| {
                    icon("chevron-down").style(|| {
                        let size = kivact::style::use_tokens::<crate::Tokens>().sizes.icon_xsm;
                        style().size(size, size)
                    })
                }),
            ));
        if let Some((entries, select)) = popup {
            menu(label, entries)
                .on_activate(move |key| select.call(key))
                .attach(content)
                .into_view()
        } else if let Some(action) = action {
            content
                .with_node(move |node| node.role(Role::Link))
                .name(label)
                .focusable(true)
                .on_press(move || action.call(()))
                .into_view()
        } else {
            content.into_view()
        }
    }
}

impl NodeBuilder for BreadcrumbItem {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for BreadcrumbItem {
    fn into_view(self) -> View {
        provide((), move || {
            let current = self.current.unwrap_or(false);
            self.build(BreadcrumbsVariant::Default, current)
        })
    }
}

fn logical_row() -> Direction {
    if rtl() {
        Direction::RowReverse
    } else {
        Direction::Row
    }
}

kivact::state::constant_value!(BreadcrumbsVariant);
