//! Role: The sidebar panel, its rail, the page beside it and the provider.

use super::look::{SidebarState, SidebarTheme};
use super::menu::pressable;
use super::recipes::themed;
use super::resize::{open_width, resize_handle};
use super::state::{
    SidebarCollapsible, SidebarCx, SidebarFrame, SidebarResize, SidebarResizeCx, SidebarSide,
    current, current_resize,
};
use super::vars;
use kivact_core::geometry::Rect;
use kivact_core::i18n::message;
use kivact_core::state::{Signal, Value, callback, memo, provide};
use kivact_core::style::{Color, style, use_theme};
use kivact_core::view::{Handled, Key, Node, Role, View, column, node, row, scroll, show};
use kivact_macros::component;

/// The panel of a sidebar: its header, scrolling content and footer. Created by [`sidebar`].
///
/// Outside a [`sidebar_provider`] it is an open sidebar with the defaults.
/// Its root is the box that takes the sidebar's width in the layout, so the
/// modifiers of the caller apply to it.
#[component]
pub fn sidebar(
    header: Option<View>,
    content: Option<View>,
    footer: Option<View>,
    #[prop(default = false)] rail: Value<bool>,
) -> impl Element {
    let cx = current().unwrap_or_else(SidebarCx::standalone);
    let state = memo(move || cx.state());
    let collapsible = cx.collapsible;
    let offcanvas_closed = move || collapsible == SidebarCollapsible::Offcanvas && !cx.open.get();
    let head = header.map(|view| themed(column(view), |r: &SidebarTheme| r.header_recipe(), state));
    let middle =
        content.map(|view| themed(scroll(view), |r: &SidebarTheme| r.content_recipe(), state));
    let foot = footer.map(|view| themed(column(view), |r: &SidebarTheme| r.footer_recipe(), state));
    let inner = themed(
        node().hidden(offcanvas_closed),
        |r: &SidebarTheme| r.panel_recipe(),
        state,
    )
    .child(head)
    .child(middle)
    .child(foot);
    let rail_view = show(
        move || rail.get() && collapsible != SidebarCollapsible::None,
        move || rail_node(cx, state),
    );
    let resize = current_resize();
    let root = themed(node(), |r: &SidebarTheme| r.sidebar_recipe(), state);
    let root = match resize {
        Some(resize) => root.style(move || open_width(cx, resize)),
        None => root,
    };
    root.child(inner)
        .child(rail_view)
        .child(resize.map(|resize| resize_handle(cx, resize, state)))
}

/// The strip on the inner edge: a click toggles the sidebar, and a hover shows a thin line.
fn rail_node(cx: SidebarCx, state: Value<SidebarState>) -> Node {
    let (rail, hover) = pressable(node(), Role::Button, Some(callback(move |()| cx.toggle())));
    let rail = rail
        .name(message("ui.sidebar.toggle", "Toggle Sidebar"))
        .focusable(false);
    themed(rail, |r: &SidebarTheme| r.rail_recipe(), state).style(move || {
        // The 2 px line is local paint, which a state style cannot draw, so
        // it follows the rail's hover.
        let line = if hover.get().is_hovered() {
            use_theme().get().get(&vars::BORDER_VAR)
        } else {
            Color::TRANSPARENT
        };
        style().paint(move |d, size| {
            d.fill_rect(
                Rect::new((size.width - 2.0) / 2.0, 0.0, 2.0, size.height),
                line,
            );
        })
    })
}

/// The page beside the sidebar (`SidebarInset`).
fn inset(state: Value<SidebarState>, page: View) -> Node {
    themed(
        node().child(page),
        |r: &SidebarTheme| r.inset_recipe(),
        state,
    )
}

/// A sidebar `sidebar` with the page beside it, opened and closed by `open`.
///
/// `open` is the application's state, so it can persist it. The trigger, the
/// rail and Ctrl+B (Cmd+B on a Mac) write it. The shortcut is a key handler
/// on the provider, so it works while the focus is inside the provider; make
/// the provider the root view of the App to have it wherever the focus is.
/// Narrow windows do not turn the sidebar into a sheet: it always keeps its
/// desktop layout. The provider fills its parent, so give the parent a size.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::ui::{SidebarCollapsible, sidebar, sidebar_group, sidebar_menu, sidebar_menu_button, sidebar_provider, sidebar_trigger};
///
/// fn app() -> impl IntoView {
///     let open = signal(true);
///     sidebar_provider(
///         open,
///         sidebar()
///             .header(text("Acme"))
///             .content(sidebar_group("Application").content(sidebar_menu(vec![
///                 sidebar_menu_button("Home").icon(icon("check")).active(true).into_view(),
///                 sidebar_menu_button("Inbox").icon(icon("copy")).badge("12").into_view(),
///             ])))
///             .rail(true),
///         column((sidebar_trigger(), text("Page"))),
///     )
///     .collapsible(SidebarCollapsible::Icon)
/// }
/// ```
#[component]
pub fn sidebar_provider(
    open: Signal<bool>,
    sidebar: View,
    page: View,
    #[prop(default = SidebarSide::Left)] side: SidebarSide,
    #[prop(default = SidebarFrame::Sidebar)] frame: SidebarFrame,
    #[prop(default = SidebarCollapsible::Offcanvas)] collapsible: SidebarCollapsible,
    resize: Option<SidebarResize>,
) -> impl Element {
    let cx = SidebarCx {
        open,
        side,
        frame,
        collapsible,
    };
    let state = memo(move || cx.state());
    let body = provide(cx, move || {
        provide(SidebarResizeCx(resize), move || {
            let page = inset(state, page).into_view();
            match side {
                SidebarSide::Left => (sidebar, page),
                SidebarSide::Right => (page, sidebar),
            }
        })
    });
    themed(row(body), |r: &SidebarTheme| r.root_recipe(), state).on_key(move |event| {
        let m = event.modifiers();
        if (m.ctrl || m.meta) && !m.alt && !m.shift && event.key() == Key::Char('b') {
            cx.toggle();
            Handled::Yes
        } else {
            Handled::No
        }
    })
}
