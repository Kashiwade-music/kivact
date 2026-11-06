//! Role: The sidebar panel, its rail, the page beside it and the provider.

use super::state::{SidebarCollapsible, SidebarCx, SidebarSide, SidebarVariant, WIDTH, WIDTH_ICON};
use crate::parts::pressable;

use kivact::draw::Shadow;
use kivact::geometry::Rect;
use kivact::i18n::message;
use kivact::view::{Node, NodeBuilder, column, node, row, scroll};

use kivact::state::Value;
use kivact::state::{IntoValue, callback};
use kivact::style::{Align, Color, Cursor, Direction, Position, pct, style};
use kivact::view::{Handled, IntoView, Key, Role, show};

// xtask-allow(state-fields): Independent construction-time placement, framing, collapse behavior, view slots and rail visibility; the provider binding owns opening.
/// The panel of a sidebar (sidebar.tsx). Created by [`sidebar`].
///
/// The root is the box that takes the sidebar's width in the layout, so
/// modifiers apply to it.
pub struct Sidebar {
    pub(super) root: Node,
    pub(super) side: SidebarSide,
    pub(super) variant: SidebarVariant,
    pub(super) collapsible: SidebarCollapsible,
    header: Option<kivact::view::View>,
    content: Option<kivact::view::View>,
    footer: Option<kivact::view::View>,
    rail: Value<bool>,
}

/// A sidebar on the left, framed as `Sidebar`, that slides out when closed.
///
/// Pass it to [`sidebar_provider`] together with the page it sits beside.
#[track_caller]
pub fn sidebar() -> Sidebar {
    Sidebar {
        root: node(),
        side: SidebarSide::Left,
        variant: SidebarVariant::Sidebar,
        collapsible: SidebarCollapsible::Offcanvas,
        header: None,
        content: None,
        footer: None,
        rail: false.into_value(),
    }
}

impl Sidebar {
    /// Choose the side of the window.
    pub fn side(mut self, side: SidebarSide) -> Self {
        self.side = side;
        self
    }

    /// Choose the frame.
    pub fn variant(mut self, variant: SidebarVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Choose what closing does.
    pub fn collapsible(mut self, collapsible: SidebarCollapsible) -> Self {
        self.collapsible = collapsible;
        self
    }

    /// The top section: a column with 8 px of padding and gap.
    pub fn header(mut self, header: impl IntoView) -> Self {
        self.header = Some(header.into_view());
        self
    }

    /// The scrolling middle section: groups, menus and separators.
    pub fn content(mut self, content: impl IntoView) -> Self {
        self.content = Some(content.into_view());
        self
    }

    /// The bottom section, like the header.
    pub fn footer(mut self, footer: impl IntoView) -> Self {
        self.footer = Some(footer.into_view());
        self
    }

    /// Draw the rail while true (constant or reactive): a thin strip on the
    /// inner edge that toggles the sidebar when pressed.
    #[track_caller]
    pub fn rail(mut self, rail: impl IntoValue<bool>) -> Self {
        self.rail = rail.into_value();
        self
    }
}

impl NodeBuilder for Sidebar {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// The room a sidebar takes in the layout for the actual state.
fn width_of(cx: SidebarCx) -> f64 {
    let framed = matches!(cx.variant, SidebarVariant::Floating | SidebarVariant::Inset);
    if cx.collapsible == SidebarCollapsible::None || cx.open.get() {
        return WIDTH;
    }
    match cx.collapsible {
        SidebarCollapsible::Icon if framed => WIDTH_ICON + 16.0,
        SidebarCollapsible::Icon => WIDTH_ICON,
        _ => 0.0,
    }
}

impl Sidebar {
    fn build(self, cx: SidebarCx) -> Node {
        let Sidebar {
            root,
            side,
            variant,
            collapsible,
            header,
            content,
            footer,
            rail,
        } = self;
        let offcanvas_closed =
            move || collapsible == SidebarCollapsible::Offcanvas && !cx.open.get();
        let section = |view: Option<kivact::view::View>| {
            view.map(|view| column(view).style(style().shrink(0).gap(8).padding(8)))
        };
        let middle = content.map(|view| {
            scroll(view).style(
                style()
                    .grow(1)
                    .shrink(1)
                    .min_height(0)
                    .gap(0)
                    .scrollbar_width(0),
            )
        });
        let inner = node()
            .hidden(offcanvas_closed)
            .style(move || {
                let p = kivact::style::use_tokens::<crate::Tokens>();
                let s = style()
                    .direction(Direction::Column)
                    .width(pct(100))
                    .height(pct(100))
                    .min_width(0)
                    .background(p.sidebar)
                    .color(p.sidebar_foreground)
                    .font_size(12);
                match (variant, side) {
                    (SidebarVariant::Sidebar, SidebarSide::Left) => {
                        s.border_right(1, p.sidebar_border)
                    }
                    (SidebarVariant::Sidebar, SidebarSide::Right) => {
                        s.border_left(1, p.sidebar_border)
                    }
                    (SidebarVariant::Floating, _) => s
                        .radius(0)
                        .ring(1, p.sidebar_border)
                        .shadow(Shadow::new(0, 1, 2, Color::BLACK.with_alpha(0.05))),
                    (SidebarVariant::Inset, _) => s,
                }
            })
            .child(section(header))
            .child(middle)
            .child(section(footer));
        let rail = show(
            move || rail.get() && collapsible != SidebarCollapsible::None,
            move || build_rail(cx),
        );
        let root = root.base_style(move || {
            let framed = matches!(cx.variant, SidebarVariant::Floating | SidebarVariant::Inset);
            let s = style()
                .position(Position::Relative)
                .direction(Direction::Column)
                .shrink(0)
                .z_index(10)
                .height(pct(100))
                .min_height(0)
                .color(kivact::style::use_tokens::<crate::Tokens>().sidebar_foreground)
                .width(width_of(cx));
            if framed { s.padding(8) } else { s }
        });
        root.child(inner).child(rail)
    }
}

/// The strip on the inner edge: a click toggles the sidebar, and a hover shows a thin line.
fn build_rail(cx: SidebarCx) -> Node {
    let (rail, state) = pressable(node(), Role::Button, Some(callback(move |()| cx.toggle())));
    let side = cx.side;
    rail.name(message("basic.sidebar.toggle", "Toggle Sidebar"))
        .focusable(false)
        .style(move || {
            let p = kivact::style::use_tokens::<crate::Tokens>();
            let closed = !cx.open.get();
            // The 2 px line is local paint, which a state style cannot
            // draw, so it follows the rail's hover.
            let line = if state.get().hovered {
                p.sidebar_border
            } else {
                Color::TRANSPARENT
            };
            let offcanvas = cx.collapsible == SidebarCollapsible::Offcanvas;
            let s = style()
                .position(Position::Absolute)
                .top(0)
                .bottom(0)
                .width(16)
                .z_index(20)
                .cursor(Cursor::EwResize);
            let s = if offcanvas && closed {
                s.hover(style().background(p.sidebar))
            } else {
                s
            };
            let s = match side {
                SidebarSide::Left => s.right(-8),
                SidebarSide::Right => s.left(-8),
            };
            s.paint(move |d, size| {
                d.fill_rect(
                    Rect::new((size.width - 2.0) / 2.0, 0.0, 2.0, size.height),
                    line,
                );
            })
        })
}

/// The page beside the sidebar (`SidebarInset`).
fn inset(cx: SidebarCx, page: kivact::view::View) -> Node {
    node()
        .style(move || {
            let p = kivact::style::use_tokens::<crate::Tokens>();
            let s = style()
                .position(Position::Relative)
                .direction(Direction::Column)
                .grow(1)
                .shrink(1)
                .min_width(0)
                .min_height(0)
                .align_items(Align::Stretch)
                .background(p.background)
                .color(p.foreground);
            if cx.variant == SidebarVariant::Inset {
                // `m-2` without the margin on the sidebar's side, which returns
                // while the sidebar is closed.
                let apart = if cx.open.get() { 0.0 } else { 8.0 };
                let s = s.radius(0).margin_y(8).shadow(Shadow::new(
                    0,
                    1,
                    2,
                    Color::BLACK.with_alpha(0.05),
                ));
                match cx.side {
                    SidebarSide::Left => s.margin_right(8).margin_left(apart),
                    SidebarSide::Right => s.margin_left(8).margin_right(apart),
                }
            } else {
                s
            }
        })
        .child(page)
}

/// A sidebar `sidebar` with the page `inset` beside it, opened and closed by `open`.
///
/// `open` is the application's state, so it can persist it. The trigger, the
/// rail and Ctrl+B (Cmd+B on a Mac) write it. The shortcut is a key handler
/// on the provider, and the public API offers window-level shortcuts only as
/// handlers on the App's view: make the provider the root view of the App and
/// it works wherever the focus is, also with nothing focused (shadcn listens
/// on the window); nested deeper, it works while the focus is inside the
/// provider. The provider fills its parent, so give the parent a size.
/// Narrow windows do not turn the sidebar into a sheet: it always keeps its
/// desktop layout.
///
/// ```no_run
/// use kivact::view::{icon};
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn app() -> impl IntoView {
///     let open = signal(true);
///     basic::sidebar_provider(
///         open,
///         basic::sidebar()
///             .collapsible(basic::SidebarCollapsible::Icon)
///             .header(text("Acme"))
///             .content({
///                 basic::sidebar_group("Application").content({
///                     basic::sidebar_menu((
///                         basic::sidebar_menu_button("Home").icon(icon("house")).active(true),
///                         basic::sidebar_menu_button("Inbox").icon(icon("inbox")).badge("12"),
///                     ))
///                 })
///             })
///             .rail(true),
///         column((basic::sidebar_trigger(), text("Page"))),
///     )
/// }
/// ```
#[track_caller]
pub fn sidebar_provider(
    open: impl kivact::state::IntoBinding<bool>,
    sidebar: Sidebar,
    page: impl IntoView,
) -> impl kivact::view::Component {
    let open = open.into_binding();
    let cx = SidebarCx {
        open,
        side: sidebar.side,
        variant: sidebar.variant,
        collapsible: sidebar.collapsible,
    };
    let page = page.into_view();
    let body = kivact::state::provide(cx, move || {
        let panel = sidebar.build(cx);
        let page = inset(cx, page);
        match cx.side {
            SidebarSide::Left => (panel, page),
            SidebarSide::Right => (page, panel),
        }
    });
    row(body)
        .style(move || {
            let s = style()
                .width(pct(100))
                .height(pct(100))
                .min_width(0)
                .min_height(0)
                .align_items(Align::Stretch);
            if cx.variant == SidebarVariant::Inset {
                s.background(kivact::style::use_tokens::<crate::Tokens>().sidebar)
            } else {
                s
            }
        })
        .on_key(move |event| {
            let m = event.modifiers();
            if (m.ctrl || m.meta) && !m.alt && !m.shift && event.key() == Key::Char('b') {
                cx.toggle();
                Handled::Yes
            } else {
                Handled::No
            }
        })
}
