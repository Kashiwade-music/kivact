# Navigation

[日本語](../../../jp/soft/how-to-guide/navigation.md)

Keep the current view in application state. For a short set of related views, bind a segmented control to an optional key and use that same key to select the content.

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::ListItem;
use kivact::view::dynamic;
use kivact_ui_soft as soft;

#[component]
fn project_view() -> impl Element {
    let view = signal(Some("summary"));
    column((
        radio_group(
            "Project view",
            vec![ListItem::new("summary", "Summary"), ListItem::new("activity", "Activity")],
            view,
        ).variant(kivact::ui::RadioGroupVariant::Segmented),
        dynamic(view, |view| match view {
            Some("activity") => kivact::ui::heading("Activity", 2),
            _ => kivact::ui::heading("Summary", 2),
        }),
    ))
    .style(style().gap(soft::SPACE_S4))
}
```

Use a stable key such as `"summary"` so selection survives a change in the visible label. Standard tabs are a better fit when the selection switches among named panels within a larger screen.

Resizable panels use `kivact::ui::splitter` with an application size signal and a `ScalarRange`.
Use `.collapsible(fraction)` and `.snaps(fractions)` for collapse and detents, `.orientation(..)`
for stacked panes, and `.pointer_scale(..)` for reversed or proportional sizing. A reactive range
can read a container's `NodeRef::rect()` to resolve percentages and pixel bounds. Custom grip
content belongs in `.handle(..)`; its style controls the divider, visibility, position, and overlay.
Use `LayoutRegion::width(..)` when a layout panel follows the same size signal. A supplied end
resize slot controls its own reversed direction. Observe scroll boundaries with `NodeRef::scroll_state()`;
scrollable areas use the core keyboard scroll behavior.

Use standard `disclosure(title, open).content(factory)` and `.animated(true)` for expandable sections. The Soft theme supplies the original spring transition. `disclosure_group(signal(None::<K>))` gives single selection; a `Vec<K>` signal gives multiple selection. `.panel(key, title, factory)` replaces keyed kit sections. Set `.variant(soft::DisclosureVariant { dividers: true, ..Default::default() })` for separators; `chevron` accepts `ChevronPosition::Start` or `End`, and `density` accepts `Some(DisclosureDensity::Compact)`, `Balanced`, or `Spacious`. These visual variants apply to both standalone sections and groups and follow text direction. `.always_open()` replaces disabled collapsing; `.content(factory).attach(trigger)` supplies a custom trigger. Create `signal(default_open)` at the call site for an initial opening.

Use standard `toolbar(label).start(...).center(...).end(...)`; `.variant(soft::ToolbarVariant(soft::SectionVariant::Muted))` selects surface chrome and `.dividers(&[ToolbarEdge::Bottom])` draws logical-edge separators. Wrap the content factory in `soft::with_key_hints(axis, || view)` and attach `soft::keyboard_hint(button)` to actual keyboard targets for one first-focus hint. The hint axis accepts a reactive orientation; arrows, focus departure, or three seconds end the visit. Segmented controls use `radio_group(...).variant(RadioGroupVariant::Segmented)` with optional `.variant(soft::SegmentedLayout::Fill)`. For icon-only segments, combine `.variant(soft::SegmentedIconOnly(true))` with `.item(...)` returning an icon and no visible label; the standard radio retains its accessible name. Attach `tooltip(reason).enabled(disabled)` to explain a disabled control. Use the standard `.disabled(...)` modifier.

For segmented hints, apply the attachment to the actual radio with `.item_node(|target, _, _| soft::keyboard_hint(target))` inside `with_key_hints`; the content-only `.item` slot stays available for icon-only rendering.

A scrolling carousel is standard `carousel(index, slides).name(label).mode(CarouselMode::Scroll)`. The caller owns `signal(0usize)`; `Snap::Mandatory` retains snapping and `Snap::None` retains free scrolling. `gap` and `padding` take logical pixels; `buttons`, `edge_fade`, and `looping` remain standard options. Headless `interaction::use_carousel(index, count)` supplies custom navigation commands. Soft recipes supply the original rounded edge controls, shadow, spacing, and four-pixel fade. Standard control names are Previous slide and Next slide, and each group is named Slide N / Count.

Use standard `breadcrumb(items)` with keyed `ListItem` values. The last item is current by default; `.current(value)` selects an explicit key and `.current(None::<K>)` clears it. `.on_activate` handles non-current keys. `.item` composes icons, per-entry actions, or a standard menu; omit global activation when the slot owns its action. `.separator` replaces the slash. `soft::BreadcrumbsVariant::Supporting` retains compact typography; the default uses body text, wrapping, and logical direction.

Empty messages use standard `empty_state(title).description(text).icon(view).actions(view)`. Add `.announce_changes()` for the previous automatic change-announcement behavior; initial render is silent and actual live ancestors suppress duplication. `.heading_level(level)` clamps to 1–6. `.variant(soft::EmptyStateVariant::compact(compact))` accepts reactive density, stacks actions when compact, and retains centered text and illustration slots.

Cards use standard `card(content)`, `.name(label).on_press(action)` for a button, or `.name(label).selected(signal)` for a checkbox. `soft::CardVariant` retains Default/Transparent/Muted and every categorical color; `soft::CardVariant`, `soft::Elevation` and `soft::CardPadding(space)` set the color, elevation and logical `Space` padding. For Soft layout children that consume inset context, wrap them with `soft::with_card_insets(variant, padding, || content)` and give the card the same `variant`; `padding` is an `Option<soft::CardPadding>`. Set width/height with standard `.style(...)`; wrap contents in standard `scroll(...).axes(ScrollAxes::Vertical)` for a bounded scrolling card. Header/media/footer are standard slots, and actual pointer feedback is supplied by the decorative overlay recipe.

Avatars use standard `avatar(name)` with reactive `.src(Some(asset))`, `.fallback_src(Some(asset))`, canonical `AvatarShape`, `.on_press`, and `.status(Tone)` or `.status_content(view)`. `soft::AvatarSize` retains Xsm20/Sm24/Md36/Lg48/Xl128 and snapped `Px`; use `.variant(size)`. The Soft theme shows the avatar's name in a tooltip by default (the previous Name mode); `.name_tooltip(false)` turns it off, and `.name_tooltip(false)` with standard `tooltip(text, avatar(...))` gives custom tooltip text for the previous Text mode. `.name(alt)` overrides alternative text. A custom status label can be composed into reactive `.name(...)`; the status content remains actual labelled content without a label registry. `avatar_status_dot(...).avatar_size(size)` keeps custom presence colors/icons and diameter metrics. For anonymous people, supply `.fallback_content(soft::avatar_silhouette)`. Groups use canonical `avatar_group(children).name(label).total(total)`; configure each child size and `.variant(soft::AvatarGroupShape(shape))` for the shared frame, with `.overlap(move || side * 0.25)` when changing size. `.overflow_content(|count| button(...))` supports the former custom count text/action from the actual omitted count. The canonical group owns keyboard navigation.

Steps use standard `stepper(vec![StepperItem::new(label)], current)`. Item description/status/disabled/optional/indicator/content/end_content and group orientation and min_step_width remain canonical, and a step press writes the `current` signal; preserve reactive status by deriving the item vector. `StepIndicator::custom(factory)` retains arbitrary glyph content. `soft::StepDensity`, `soft::StepIndicatorPosition` and `soft::StepCollapsedVariant` set the density, indicator position and compact summary. These style the actual leading/trailing/progress parts using canonical index/count/orientation, including first/last connectors and filled segments. Sparse old indices can be mapped to vector positions with a custom number indicator and activation lookup. Compact WithLabelAndControls/WithLabel/HiddenLabel preserve their original visibility; the separate canonical `CollapsedSteps::{Current,All}` construction policy controls which items remain visible. Compact navigation skips disabled items.

Banners use standard `alert(title).tone(Tone::Info).icon(soft::banner_icon(Tone::Info))`; Warning/Success map directly and Error maps to `Tone::Danger`. `.actions(...)` replaces end content; details use `.content(factory).collapsible(true).open(signal)` (set collapsible explicitly for the previous banner default). Standard `.dismissible(...)`, `.dismiss_label(...)`, and `.on_dismiss(...)` retain dismissal and external-focus restoration. A localized title-specific dismiss name can be passed with `i18n::message_with`. `soft::BannerVariant` (`Card` or `Section`) and `soft::Elevation` set the surface and elevation. The theme styles actual header/message/controls/body wrappers, and the actual chevron target keeps its standard tooltip.

Pagination uses zero-based standard `pagination(count, page)`. Replace the old one-based `u32` signal with `usize` index state, or adapt the same caller signal with standard `binding` getters/setters; notify on accepted writes there instead of a kit callback. `.window(PaginationWindow::Buffered)` preserves the previous edge-filled window and all four boundary/gap cases. Pages/Count/Compact/Dots/Input map to `PaginationMode`; the old None-center appearance maps to Count with a hidden `pagination_status` summary, while canonical None removes controls. `soft::PaginationSize(size)` sets the control size. Keep first_last/step/siblings/disabled/page_label/has_more/page_size/options as canonical configuration; explicitly use page size10 to preserve the former implicit default when counting items. Add `.summary(soft::pagination_status)` for the original polite page notice. Count uses `.summary(|facts| (soft::pagination_count(facts), soft::pagination_status(facts)))`; Compact uses `pagination_page_summary`. These helpers read the actual summary source and own no page model. An Input presentation can show custom prefix/total text in the summary slot while the actual editor keeps its canonical draft and label. Dots show all pages and wrap selection with actual keyboard focus in either direction.

Soft top/side/mobile navigation remain domain layouts. Desktop top menus, workspace-heading menus, mega menus and collapsed-rail flyouts now share one `interaction::use_navigation_menu` owner per navigation root. Actual panel identities come from its setup-only `panel_key`; hover and pin are `NavOpen` states, and Popup/Dialog/Menu observe the same accepted binding. Pointer timers live in that controller, not a hover-open mirror. Menus retain canonical item keyboard navigation/typeahead, and trigger activation pins a hover visit. Rail eligibility changes send actual close/departure events so expanding then collapsing cannot resurrect a stale panel. Standard `navigation_menu` and rich `navigation_menu_link` also receive the full Neutral recipe table.
