# Build navigation: sidebar, tabs, breadcrumb, menu bar and pagination

[日本語](../../../jp/basic/how-to-guide/navigation.md)

This guide shows how to let users move around an app: an app shell with a sidebar, tabs inside a page, a breadcrumb trail, a menu bar, a top navigation menu and page numbers. It assumes you have done the [tutorial](../getting-started/README.md).

None of these components keeps the current location. You hold it in a signal (`page`, `current`, ...), pass it to the component and show whatever the signal says. Pressing something writes the signal, and the rest of the screen follows.

## Build an app shell with a sidebar

`basic::sidebar_provider(open, sidebar, page)` puts a sidebar beside the page area and shares the `open` signal with them. A menu button marks the current page with `.active(..)` and changes it in `.on_press(..)`:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn shell() -> impl IntoView {
    let open = signal(true);
    let page = signal("home");
    let entry = move |key: &'static str, label: &'static str, name: &'static str| {
        basic::sidebar_menu_button(label)
            .icon(icon(name))
            .active(move || page.get() == key)
            .on_press(move || page.set(key))
    };
    let sidebar = basic::sidebar()
        .collapsible(basic::SidebarCollapsible::Icon)
        .rail(true)
        .header(basic::large("Acme Inc"))
        .content({
            basic::sidebar_group("Application").content({
                basic::sidebar_menu((
                    entry("home", "Home", "house"),
                    entry("inbox", "Inbox", "inbox"),
                    entry("settings", "Settings", "settings"),
                ))
            })
        });
    let content = column((
        row((
            basic::sidebar_trigger(),
            basic::separator()
                .orientation(Orientation::Vertical)
                .style(style().height(16)),
            basic::breadcrumb_content((
                basic::breadcrumb_link("Acme"),
                basic::breadcrumb_separator(),
                basic::breadcrumb_page(move || page.get().to_string()),
            )),
        ))
        .style(style().height(48).padding_x(12).gap(8).align_items(Align::Center)),
        basic::h4(move || page.get().to_string()).style(style().padding(12)),
    ))
    .style(style().grow(1));
    column(basic::sidebar_provider(open, sidebar, content)).style(
        style()
            .size(pct(100), pct(100))
            .background(basic::ColorName::Background)
            .color(basic::ColorName::Foreground),
    )
}
```

- The provider fills its parent. In a bounded frame, give the parent a width and a height.
- `.collapsible(..)` decides what closing does: `Offcanvas` (default) slides the sidebar out of view, `Icon` shrinks it to a 48 px column of icons whose labels become tooltips, and `None` cannot be closed. `.rail(true)` adds a thin strip at the edge that toggles it.
- `basic::sidebar_trigger()`, the rail and Ctrl+B (Cmd+B on macOS) all write `open`. To make the shortcut work anywhere in the window, make the provider the root view of the app.
- `.side(basic::SidebarEdge::Right)` moves the sidebar to the right, and `.variant(basic::SidebarVariant::Floating)` or `Inset` change how it is framed.
- `.header(..)`, `.content(..)` and `.footer(..)` take functions. Groups are `sidebar_group(label)` with `.content(..)` and an optional `.action(..)`; separate them with `basic::sidebar_separator()`.
- A button can show a count with `.badge("12")`, and a nested list with `.sub(|| (basic::sidebar_menu_sub_button("Overview").active(true), basic::sidebar_menu_sub_button("Roadmap")))`.
- Put a search field in the sidebar with `basic::sidebar_input(label, signal)`.

## Switch views with tabs

Tabs are the standard `tabs(label, options, current)`. `current` holds the key of the selected tab, which is always one of the options. `.content(|key| ..)` builds the panel for the selected key:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn account() -> impl IntoView {
    let current = signal("account");
    let options = vec![
        ListItem::new("account", "Account"),
        ListItem::new("password", "Password"),
        ListItem::new("billing", "Billing").disabled(true),
    ];
    tabs("Settings", options, current)
        .variant(basic::TabsVariant::Line)
        .content(|key| {
            let (title, body) = match key {
                "account" => ("Account", "Make changes to your account here."),
                _ => ("Password", "Change your password here."),
            };
            column((basic::h4(title), basic::muted(body))).style(style().gap(8).padding_top(16))
        })
}
```

- The boxed look is the default. `.variant(basic::TabsVariant::Line)` gives the underlined look.
- `.orientation(Orientation::Vertical)` stacks the tabs in a column beside their panel.
- The first argument is the accessible name of the tab list. Arrow keys move between tabs.

## Show where the user is

A breadcrumb is `basic::breadcrumb_content((..))` of links, separators and the current page. Links take `.on_press(..)`; the current page is not a link:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn trail() -> impl IntoView {
    let at = signal(String::from("Components"));
    basic::breadcrumb_content((
        basic::breadcrumb_link("Home").on_press(move || at.set("Home".into())),
        basic::breadcrumb_separator(),
        basic::breadcrumb_link("Components").on_press(move || at.set("Components".into())),
        basic::breadcrumb_separator(),
        basic::breadcrumb_page(at),
    ))
}
```

Put `basic::breadcrumb_ellipsis()` between separators to stand for hidden levels. If a plain list of names is enough, `basic::breadcrumb(vec!["Home".to_string(), "Docs".to_string()])` builds the whole trail from text.

## Add a menu bar

`basic::menubar(menus).on_activate(choose)` takes a list of `(title, entries)` pairs and shows the titles in a row. Each opens a standard menu. `choose` gets the key of the entry that was picked:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn bar() -> impl IntoView {
    let last = signal(String::from("Nothing chosen yet."));
    column((
        basic::menubar(vec![
                (
                    "File".to_string(),
                    vec![
                        MenuEntry::item("new", "New tab").with_shortcut("Ctrl+T"),
                        MenuEntry::separator(),
                        MenuEntry::item("print", "Print...").with_shortcut("Ctrl+P"),
                    ],
                ),
                (
                    "Edit".to_string(),
                    vec![
                        MenuEntry::item("undo", "Undo").with_shortcut("Ctrl+Z"),
                        MenuEntry::item("redo", "Redo").with_shortcut("Ctrl+Y"),
                    ],
                ),
            ]).on_activate(move |key: &'static str| last.set(format!("Chosen: {key}")))
        .style(style().width(240)),
        basic::muted(last),
    ))
    .style(style().gap(12))
}
```

The entries are the same `MenuEntry` values as for any menu (items, check and radio items, headings, submenus; see [Overlays](overlays.md#open-a-menu-from-a-button-or-a-right-click)). Pass a closure instead of a `Vec` to make the menus follow your signals. The bar does not take over the system menu: it is an ordinary row inside your window.

## Link to sections from the top of a page

`basic::navigation_menu(items)` is a row of `NavItem`s. A `NavItem::link(label, action)` runs `action` when pressed. A `NavItem::menu(label, content)` opens a panel under the item, and `content` builds that panel's view each time. Use `basic::navigation_menu_link(title)` for the rows inside a panel:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn top_nav() -> impl IntoView {
    let clicks = signal(0usize);
    let getting_started = move || {
        column((
            basic::navigation_menu_link("Introduction")
                .description("How the pieces fit together."),
            basic::navigation_menu_link("Installation")
                .description("Add the crates to your project.")
                .icon(icon("download")),
        ))
        .style(style().width(320))
    };
    column((
        basic::navigation_menu(vec![
            basic::NavItem::menu("Getting started", getting_started),
            basic::NavItem::link("Docs", move || clicks.update(|n| *n += 1)),
        ]),
        basic::muted(move || format!("Docs pressed: {}", clicks.get())),
    ))
    .style(style().gap(12))
}
```

## Page through a long list

`basic::pagination(page, count)` shows previous and next buttons and the page numbers, with ellipses when there are many. `page` is a zero-based signal and `count` the number of pages:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn pager() -> impl IntoView {
    let page = signal(0usize);
    column((
        basic::pagination(page, 12usize),
        basic::muted(move || format!("Page {} of 12", page.get() + 1)),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
```

- `.label_format(|number| format!("p.{number}"))` changes how a page number reads.
- For full control, compose `basic::pagination_content((basic::pagination_previous(), basic::pagination_link("1"), basic::pagination_link("2").active(true), basic::pagination_ellipsis(), basic::pagination_next()))` yourself.
- The words "Previous", "Next" and the accessible labels come from messages under `basic.pagination.*` that you can replace; see [Theme and tokens](theme-and-tokens.md#choose-the-fonts).
- To page through table rows, `data_table(..).page(page, size)` has pagination built in; see [Data display](data-display.md).

## See also

- [Overlays](overlays.md): menus, popovers and dialogs.
- [Data display](data-display.md): tables with sorting and paging.
- Rustdoc: `cargo doc -p kivact-ui-basic --open` for the sidebar parts and the navigation components.
