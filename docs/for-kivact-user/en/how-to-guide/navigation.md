# Navigation

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

Stepper exposes actual decorative `leading`, `trailing`, and `progress` recipe parts with zero-footprint defaults, plus `layout` and `track` containers. They receive the same `StepperItemState` including actual orientation/count. `.collapsed_steps(CollapsedSteps::Current)` retains the current item only (default); `All` retains every item in compact layout without changing selection or the responsive threshold.

Each step recipe also receives its actual `StepIndicatorKind` and independent `interactive` configuration, without copying metadata into a visual variant. Compact previous/next commands derive the nearest enabled item from the same item vector and current source.

`pagination(count, page).summary(|facts| view)` replaces summary content with the actual `Value<PaginationSummary>`. `page` is zero based; `count` is pages, `total_items` is an optional item total, and `page_size` is items per page. Derive displayed first/last items in the presentation callback instead of storing duplicate range endpoints. `has_more` preserves unfinished remote counts. `PaginationItemState.mode` projects the actual structural mode so dot and numbered recipes need no copied variant.

Dots render every actual page. Left/Right move selection and actual target focus with wrapping and reading-direction awareness; the selected dot alone is in the dot tab order. Dots use their own actual-target navigation, while other modes retain the generic root roving focus. Count changes recreate only the mounted dot scope, not a separate focus registry.

Pagination exposes actual `controls`, `sizes`, and `input` recipe wrappers with neutral `Display::Contents` defaults. The size selector precedes the controls; Count/Compact/Input place the existing summary or editor between previous and next controls. They share the actual summary projection and introduce no extra pagination or focus owner.

`PaginationWindow::Adjacent` keeps the standard nearest-page window. Construction-only `.window(PaginationWindow::Buffered)` fills the edge window, retaining five numbered pages plus sibling allowance before a gap; all short windows remain visible. `NavigationMenuApi::panel_key()` allocates a genuine panel identity during setup, including a dynamic mount factory. It is not an event epoch, registry, or cached visibility; standard navigation-menu panels use this same allocator. `interaction::use_menu_with_open(entries, open_binding)` lets a navigation owner supply the actual accepted popup source. External hover opening initializes menu levels without requesting focus; ArrowUp/Down still open and focus canonically. A refused setter leaves the actual open state and focus untouched. `.trigger_action(callback)` replaces the existing trigger press action during setup, rather than observing it; use `navigation.toggle(panel_key)` for hover-to-pinned activation. Other trigger keys and popup dismissal remain canonical.

A controlled menu distinguishes an actual external opening from a trigger opening in its existing opening-mode record. External showing does not run the content mount’s implicit first-item focus. After a replacement trigger action, an accepted open result uses the standard trigger opening mode and first-item request; a refused or closed result makes no focus request.

The same navigation machine privately records one visible `NavOpen` plus the identity of its actual pending opening or departure command. `panel_removed(key)` clears only that key’s active panel or scheduled command. Disposing panel A while B is pending preserves B’s request; disposing A before its own opening delay cannot later publish an orphan active key. Domain panel cleanup and eligibility changes send this actual event, with no registry, mirror, extra signal or epoch counter.
