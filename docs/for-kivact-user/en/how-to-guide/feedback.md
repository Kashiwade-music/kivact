# Show feedback and group related controls

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

When migrating kit components, move empty-state `media` to `icon` and `content` to `actions`; card `image` becomes `media` and its content is the required argument; alert `action` and banner `end` become `actions`. `input_group_text` becomes `input_addon`. Replace `clickable_card` with `card(..).on_press(..)` and `selectable_card` with `card(..).selected(..)`. Kit color, compactness, padding, elevation, media-frame, and container looks belong to kit-defined `.variant(..)` values, recipes, or ordinary `.style(..)` modifiers. They are not separate core visual enums.
