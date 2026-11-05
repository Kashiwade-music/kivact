# Accordion panels and previews

Use `disclosure_group` with `signal(None::<K>)` for one expanded panel or
`signal(Vec::<K>::new())` for independently expanded panels. Each `.panel`
receives its key, title, and a factory rebuilt whenever the panel opens.
The selection binding is the only source of expanded state. Headers support
Enter, Space, and normal forward and backward Tab navigation, following the
[APG Accordion pattern](https://www.w3.org/WAI/ARIA/apg/patterns/accordion/).

```rust,no_run
use kivact::prelude::*;

fn sections() -> impl IntoView {
    kivact::ui::disclosure_group(signal(Some("overview")))
        .panel("overview", "Overview", || text("Project overview"))
        .panel("details", "Details", || text("Project details"))
}
```

`disclosure(title, open).always_open()` keeps its body expanded structurally.
Use `.content(factory).attach(trigger)` to supply your own trigger.

`hover_card(factory).attach(trigger)` previews content after the theme's
pointer entry delay; focus opens it immediately. Moving towards the preview
keeps it open through the shared pointer corridor. Escape or an outside press
dismisses it until the current engagement ends. Configure `placement`, `align`,
`open_delay`, and `close_delay`; `label` gives an interactive preview a dialog
name. `focus_trigger` controls focus opening, `enabled` gates the preview,
and `default_open` keeps an initial preview until its first completed visit.
An explicit `open` binding is the visibility authority: pointer and focus request
writes, rejected writes are respected, and an external close suppresses reopening
until the current engagement ends. `on_open_change`
observes visibility changes, excluding the initial value.

A `dialog(title, open).mode(DialogMode::Drawer(edge))` includes a drag handle.
Pull it towards its edge to dismiss after the theme's distance or flick
threshold. Cancellation restores its position. `InteractionPolicy.drawer`
sets gesture thresholds; `hover_card` sets preview timing.

`use_copy_feedback(duration)` combines clipboard writing with a confirmation:
call `copy.copy(text)` from a handler and read `copy.is_copied()` for display.
Each successful copy restarts the duration. Clipboard failures reach the owner
error handler without starting feedback; owner disposal cancels pending work.

Disclosure recipes read `DisclosureRecipeState`: actual input and open state plus an optional fixed `group` panel position (`index`, `count`). This permits separators without storing group lifecycle state. Group variants merge with panel variants, with a panel-specific type taking precedence. `UiTokens::disclosure.motion.transition` can supply a theme transition; an absent value retains the duration-based ease-out tween.
