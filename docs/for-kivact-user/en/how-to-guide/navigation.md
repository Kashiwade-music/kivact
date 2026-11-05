# Navigation

[日本語](../../jp/how-to-guide/navigation.md)

Use the core navigation builders with your application's data. Installing a kit theme changes their recipes and tokens.

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{self, PaginationMode};

fn navigation() -> impl IntoView {
    let page = signal(0usize);
    let step = signal(1usize);
    column((
        ui::breadcrumb(vec![
            ListItem::new("home", "Home"),
            ListItem::new("orders", "Orders"),
        ]).on_activate(|key| println!("Open {key}")),
        ui::pagination(20usize, page).siblings(2usize),
        ui::pagination(20usize, page).mode(PaginationMode::Input),
        ui::stepper(vec![
            StepperItem::new("Account"),
            StepperItem::new("Address").description("Delivery address"),
            StepperItem::new("Confirm").status(StepStatus::Success),
        ], step),
        ui::navigation_menu("Main navigation")
            .link("guide", "Guide", || button("Read the guide"))
            .link("help", "Help", || text("Contact support")),
    ))
}
```

Pagination binds a zero-based page index; visible page numbers and the editable page field are one based. `.siblings` controls the neighborhood around the current page; `.first_last(true)` adds endpoint controls. `.step` changes the previous/next increment. `.item` replaces the visible contents of each `PageItem` while retaining activation, current-page semantics and focus behavior. `.label_format` receives a one-based page number.

The constructor's count is the page-count source. Calling `.total_items(total)` explicitly replaces that source with an item total, divided by `.page_size(binding)` (or one if omitted). `.page_size_options` accepts reactive options. Changing page size through the control returns to the first page. `.has_more(true)` allows next-page navigation when the total is still unknown. `PaginationMode` selects pages, dots, a count, compact controls, an editable input, or no page controls. Input commits on Enter or focus departure and clamps to the available pages.

Stepper current indices are zero based. A step's description, status, optional marker, enabled state, indicator and content belong to its `StepperItem`. `.on_step_press` reports activation; the application changes the current step. `.orientation` changes the layout. Kits express density, spacing and indicator styling through recipes, tokens and variants.

Breadcrumbs expose a navigation landmark and list. The last item has `CurrentItem::Page` and is not an activation target. Earlier items activate their keys with Enter, Space or a pointer. `.item` replaces item contents and `.separator` replaces decorative separators.

Navigation triggers open on hover. Clicking, Enter, Space or ArrowDown pins a panel; Escape dismisses it. Pinned panels also dismiss on outside press or focus departure. `.open_delay` and `.close_delay` accept `Duration`; `.placement` and `.align` position panels. Use `interaction::use_navigation_menu()` when composing a domain navigation control: `NavOpen` is its single visibility state, and `hover_after`, `leave_after`, `pin`, `toggle` and `close` send transitions to that owner.

`breadcrumb(items).current(current_key)` overrides the default final current page. The source accepts `Option<K>` reactively: `Some(key)` marks that key, while explicit `None` marks no page. Omitting the method retains the final-item default.

Stepper item recipes receive `index`, `count`, and actual `orientation` from the same responsive layout projection; visual variants do not need to copy the axis or item count.

Breadcrumb item recipes receive `BreadcrumbItemState { current, input }`; `input` comes from the actual link part and is absent for passive/current labels. Use this snapshot for state-dependent typography such as hover underline; paint-only pseudo-state styles cannot change text decoration.

## Choose a page-control layout

Use `PaginationMode::Input` when users know the page number, `Compact` for a
small previous/next control, or `Dots` for a short visual sequence. Dots include
every page; Left and Right move selection and focus, wrap at the ends, and follow
the reading direction. A large result set is better served by numbered pages or
an input than hundreds of dots.

`PaginationWindow::Adjacent` shows the nearest numbered pages.
`.window(PaginationWindow::Buffered)` keeps a fuller window near the start and
end. This is a construction option; page count and current page can still change.

To show item ranges, use `.summary(|facts| view)`. The callback receives a
`Value<PaginationSummary>` containing the current zero-based page, page count,
optional item total, and page size. Format the displayed range from those facts
inside a text closure so it follows page and size changes.

## Keep compact steps useful

`.collapsed_steps(CollapsedSteps::Current)` shows the current item in compact
layout; `All` keeps every item visible. This changes presentation, not which step
is current. Previous and next controls skip disabled steps. The current binding
and each `StepperItem` remain the source for selection and status.

A step's `.indicator(..)` chooses its marker: `StepIndicator::Auto` follows the
step status, `Number` always shows the step number, and `None` hides the marker.
Pass `StepIndicator::custom(|| view)` for any other contents, such as a letter
or an image.

## Navigate from inside a menu

Use `.action(key, label, press)` for a top-level entry that navigates directly
without a panel. Inside a panel, build entries with `navigation_menu_link(title)`:
it carries link semantics and closes the surrounding panel when activated. The
application performs the navigation; it does not copy opening or keyboard
handling.

```rust,no_run
use kivact::prelude::*;
use kivact::ui;

fn docs_menu() -> impl IntoView {
    ui::navigation_menu("Docs")
        .action("home", "Home", || println!("Go home"))
        .link("guides", "Guides", || {
            ui::navigation_menu_link("Getting started")
                .description("Build the first window")
                .on_press(|| println!("Open getting started"))
        })
}
```

For a custom menu trigger, connect visibility to an application binding with
`interaction::use_menu_with_open(entries, open_binding)`. `.trigger_action(callback)`
replaces the trigger's normal press action during setup; it is not an observer
of visibility changes. Opening from outside does not move focus into the menu,
while keyboard opening from the trigger keeps the standard initial focus, so do
not add a manual focus correction after showing the menu.

## Customize appearance

Install a kit theme for its navigation look. Use `.variant(..)` for kit-specific
choices and `.style(..)` for local layout. Theme authors can replace pagination
`controls`, `sizes` and `input` recipes, or stepper `leading`, `trailing`,
`progress`, `layout` and `track` recipes. These style the standard parts while
keeping their keyboard and accessibility behavior. See the
[theming guide](theming.md) for details.

For a custom navigation control, start with `use_navigation_menu` and its panel
keys rather than a separate visibility signal for every trigger. Its controller
handles delayed opening, pinned panels, dismissal and removal. See
[custom controls](custom-controls.md) before composing the lower-level Parts.
