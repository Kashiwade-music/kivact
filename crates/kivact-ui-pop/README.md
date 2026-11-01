# kivact-ui-pop

Calm slate surfaces, narrow blue gradients and yellow accents as reusable Kivact
components for compact desktop work apps.

The only normal and development dependency is the public `kivact` crate;
there are no build dependencies.
There is no dependence on the renderer, engine, layout, protocol, reactive crates,
or another official UI component crate.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as ui;

fn settings() -> impl IntoView {
    let sync = signal(true);
    let name = signal(String::new());
    ui::panel(column((
        kivact::ui::field("Display name", kivact::ui::text_input("Display name",name)),
        kivact::ui::switch("Sync reports",sync),
        kivact::ui::button("Save").icon(kivact::control::icon("check")).on_press(move || {
            // Persist through the application's own model or service.
        }),
    )).style(style().gap(16)))
}
```

Standard buttons, checks, switches, fields, selects, menus, dialogs, tooltips and plain tabs come from `kivact::ui`. This kit installs their complete recipes and provides domain components.

Apply `ui::theme(kivact::style::ThemeMode::Light)` to `App` or a public `provide_theme`
boundary. `ThemeMode::Dark` carries the source's complete dark palette. An
application can instead install modified public `ui::Tokens` through
`Theme::with`; component theme styles react to the current theme. Fonts
are application-owned: the reference catalog registers Inter, Barlow and Noto
Sans JP through Kivact's public font API. The theme's `IconSet` holds the kit's
Tabler icons, so `kivact::control::icon("name")` draws them; the kit has no
icon function of its own.

The kit's theme gives the standard components of `kivact::ui` the Pop look:
buttons, checkboxes, switches, selects, menus, dialogs, tooltips, tabs, radio
groups (`kivact::ui::radio_group`), sliders (`kivact::ui::slider`) and the toast
region (`kivact::ui::toast_region`) are the standard ones. The toggle
button (`kivact::ui::toggle`, filled by the `ui::Hue` variant), the progress
bar (`kivact::ui::progress`, with `ui::Striped` and `ui::Segments`) and the
accordion (`kivact::ui::disclosure`) are standard ones in this look. The kit adds
the segmented look of a radio group
(`kivact::ui::radio_group(label, options, value).variant(ui::Segmented)`), inputs, date pickers, number inputs, tags and file
dropzones, all taking their values as constructor arguments. Tag and chip
collections accept read-only values and report `on_add` / `on_remove`
requests; the application applies or rejects them.

The standard `kivact::ui::data_table(rows, key, columns)` is drawn in this
kit's theme (its `DataTableRecipes`): a rounded surface, 40 px headers, 52 px
rows and 160 px columns that scroll sideways. What it does besides showing the
rows is the caller's state, given to the method that turns it on
(`.selection(..)`, `.filter(..)`, `.page(..)`, `.sort(..)`). Controls beside
the search, the count of checked rows and export buttons are built in
`.toolbar(|context| view)`, and the pager can be replaced with
`.footer(|context| view)`. Dialogs, drawers, menus,
popovers, tooltips,
command palettes (the standard `kivact::ui::command_palette` in Pop's look;
`Command::icon` gives a row its icon) and timed notifications use public overlays. Badges, avatars,
progress, loaders, empty states, timelines, chat bubbles and calendars complete the display
patterns. `thread(..).portrait(|| view)`, `stepper(..).notes(..)` and
`timeline_item(..).title_content(|| view)` replace the avatar, add the notes
and replace the title text. The landing hero and mission cards are application
examples in the catalog.
Removable chip collections, upload rows, typing bubbles and notification
history expose their data and state to the caller, including upload byte sizes
and progress, read flags and command grouping/search hints.

`kivact::ui::tabs(label, options, selected)` is drawn with the underline look;
`.content(|key| ..)` adds the `Role::TabPanel` of the selected tab under the
list, labelled by its tab. `.item(ui::tab_label)` shows each tab's icon, label
and its `ListItem::description` as a count badge. A segmented control is a
radio group, not tabs, and `` shows its segments the
same way.

`text_area` retains its fixed 80 px default. Opt into resizing its height with
`ui::resizable_text_area(kivact::ui::text_area("Memo",memo), 80.0, 72.0)` after
configuring the field. The lower-right grip changes only height and preserves
the mounted editor, its value and editing focus. Set the wrapper's width with
ordinary `.style(...)` modifiers. The grip carries a `kivact::interaction::use_splitter` handle part:
Up and Down change the height by 8 px, Home and End jump to the minimum and to
4000 px, and accessibility increment, decrement and finite value requests use
the same range.

Builder components implement `NodeBuilder` and `IntoView`, so ordinary
`NodeBuilder` modifiers such as `.style(...)`, `.test_id(...)`, `.disabled(...)`,
`.name(...)` and `.on_key(...)` remain available. Patterns that return a `Node`
accept those modifiers directly. State lives in caller-owned signals/bindings; data
types such as `kivact::i18n::CalendarDate`
are public. File pickers report paths. Transfer I/O, persistence, networking and
business rules remain application concerns; the reference catalog demonstrates
local metadata and simulated upload progress through those public bindings.

Run `cargo run -p kivact-ui-pop-catalog` from the repository root to view
the source shell and all thirteen sections. The sidebar scrolls the existing
page; theme switching preserves input and selection state. Ctrl+K toggles the
command palette (the catalog binds the key and gives it to the palette's
`close_on`, and adds the key hints as its `footer`).

`tests/consumer.rs` imports only `kivact` and `kivact_ui_pop`; it exercises
pointer, keyboard, native edits, mixed checks, selection, sorting, paging,
overlay dismissal/focus, notifications, calendar navigation and live themes.
The workspace dependency audit additionally checks Cargo metadata across all
dependency kinds. Validation results and image evidence are recorded by the
stage-three workspace report. Consumer screens require real windows by default;
`KIVACT_TEST_GPU=1` selects GPU rendering and asserts the reported renderer,
while the default asserts CPU rendering. `KIVACT_TEST_HEADLESS=1` selects an
explicit development-only headless run. GPU builds can use `--features kivact/gpu`.

The standard `kivact::ui::field(label, control)` takes `.hint(text)` and `.error(value)`. Applications supply
dropzone guidance with `.hint(...)` and native picker filters
with `.filters(...)`; both default to empty. Agenda rows accept `.period(...)` and render a details button only
with `.on_details(...)`. Calendar day tones are opt-in through `.weekend(...)`.
Configure the application `Toaster` before mounting
`kivact::ui::toast_region(toaster)`; `Toast::new(title).tone(Tone::Success)`
makes the core `Toast` with the tone's icon (the theme names the icons
`toast-info`, `toast-success`, ...), and the second line is `.body(text)`.
`ui::Hue` is the color of Pop's badges and cards; meaning (an alert, a toast)
is the standard `kivact::ui::Tone`. Mission data
and mission cards belong to the catalog application.

Collection inputs use `impl IntoValue<Vec<T>>`; passing a signal grants no write
access. Tag removals report indices, chip additions report `()`, and tag additions
report strings (`kivact::ui::tag_input(label, tags, on_add, on_remove)`). Table export is application-owned, including
the choice of columns, rows, TSV formatting and clipboard I/O.

Optional presentation is configured by builders: `stat_card(title, value)` has `trend`, `icon`, `points`, and `tone`; `list_item(title)` has `subtitle`, `icon`, `tone`, and mount-once `trailing`. Text and samples default empty, icons absent, and hues gray. `thread(initials, title)` has `preview`, `unread` (zero), and `selected` (false). `message(content)` and `message_group(content)` have `timestamp`, `mine` (false), and `initials`. `timeline_item(title)` has `detail`, `timestamp`, `icon`, and `tone`; `agenda_item(time, title)` has `description` and `tone`. Date range inputs group endpoint pairs: `date_range_input((start_label, end_label), (start, end), today)`. Component lists, step notes and file filters accept reactive values. The decorative Pop loader belongs to the private catalog.
