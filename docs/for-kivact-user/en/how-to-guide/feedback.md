# Show feedback and group related controls

[日本語](../../jp/how-to-guide/feedback.md)

Use `kivact::ui` for empty states, alerts, cards, and input groups. The installed theme supplies each component's recipes. Root `NodeBuilder` modifiers, including `.name(..)` and `.style(..)`, remain available.

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{self, AlertVariant, ControlSize};

fn feedback() -> impl IntoView {
    let chosen = signal(false);
    column((
        ui::empty_state("No projects")
            .description("Create a project to get started.")
            .actions(ui::button("Create project"))
            .announce(),
        ui::alert("Connection interrupted")
            .description("Your changes are stored locally.")
            .variant(AlertVariant::Banner)
            .on_dismiss(|| {}),
        ui::card(text("Local workspace"))
            .header(text("Project"))
            .footer(text("Available offline"))
            .name("Local workspace")
            .selected(chosen),
        ui::input_group_with("Website", || (
            ui::input_addon("https://"),
            ui::text_input("Host", signal(String::new())),
        )).size(ControlSize::Large),
    ))
}
```

An empty state's `.icon(..)` and `.actions(..)` accept views built once. Its title is a heading; `.heading_level(..)` takes a constant or reactive `u8`, clamped to 1–6. `.announce()` opts into one polite announcement per mount. Later title or description updates remain visible without another announcement.

An alert exposes `Role::Alert` and an assertive live region. Showing one does not move focus, and it remains until the application removes it or the user dismisses it. `.tone(..)` takes a constant or reactive semantic tone. Use `.icon(..)` and `.actions(..)` for custom content, `.dismissible(..)` for a reactive dismiss control, `.dismiss_label(..)` for its accessible name, and `.on_dismiss(..)` to receive dismissal. Dismissal restores the previous outside focus through core focus restoration before invoking the callback. `AlertVariant::Banner` selects the full-width structural presentation.

For optional details, `.content(|| view)` rebuilds the details on each expansion. `.collapsible(..)` enables the disclosure control, and `.open(binding)` connects expansion to application state. Without `.collapsible(true)`, details stay visible.

Cards take content as their required argument, then optional `.header(..)`, `.media(..)`, and `.footer(..)` slots. `.on_press(..)` makes a card a keyboard-accessible button; `.selected(binding)` makes it a checkbox. Give interactive cards an accessible `.name(..)` and use `.disabled(..)` when activation is unavailable.

`input_group(label, children)` accepts already built children. `input_group_with(label, || children)` constructs children once in the inherited `.size(..)` scope, so explicit sizes on a child still win. `.start(..)` and `.end(..)` add inline adornments; `.block_start(..)` and `.block_end(..)` add regions above and below. `input_addon(text)` supplies noninteractive text.

## Choose the right feedback behavior

Use an empty state for a quiet explanation of missing content and an alert for a
message that needs attention. `.announce()` requests one polite empty-state
announcement after mounting. `.announce_changes()` instead announces a changed
visible message, including content that arrives after loading, and suppresses
repeated announcements of the same message. Keep the message concise.

A card with `.on_press(..)` acts as a button; a card with `.selected(binding)` acts
as a checkbox. Keep the click destination or selection in application state.
Decorative media remains a view passed to `.media(..)`; ordinary content is the
required `card(content)` argument.

## Customize the look

Use a kit's `.variant(..)` for its density, color or container choices, or apply
ordinary styles. Theme authors can style alert `header`, `message` and `controls`
wrappers, and a card's decorative `overlay`, without replacing the underlying
interaction. Default alert wrappers use `Display::Contents`, so they add no box
unless the recipe asks for one. Keep custom actions as real named controls.
