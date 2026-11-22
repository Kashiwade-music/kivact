# Step 6: Style and accessibility

[日本語](../../jp/getting-started/06-style-and-accessibility.md)

In this step you give the app a look: a larger title, muted secondary text, a card around the list and button variants. You also give every control a name that assistive technology can read, and add a dark mode switch.

## The program so far

```rust,no_run
use kivact::prelude::*;

const LOGO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect width="24" height="24" rx="6" fill="#336699"/><path d="M6 12.5l4 4 8-9" stroke="white" stroke-width="2.5" fill="none"/></svg>"##;

#[derive(Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
    done: bool,
}

#[component]
fn header(dark: Signal<bool>) -> impl Element {
    row((
        image(LOGO).name("Tasks logo").style(style().size(32, 32)),
        column((
            text("Tasks").role(Role::Heading).level(1).style(style().font_size(28).font_weight(700)),
            text("Things to do today").style(style().color(color::TEXT_MUTED)),
        ))
        .style(style().gap(2).flex_grow(1)),
        switch("Dark mode", dark),
    ))
    .style(style().gap(12).align_items(Align::Center))
}

#[component]
fn task_row(tasks: Signal<Vec<Task>>, task: Signal<Task>) -> impl Element {
    let id = task.get().id;
    row((
        checkbox(move || task.get().title, bind!(task.done)),
        button("Remove")
            .name(move || format!("Remove {}", task.get().title))
            .variant(ButtonVariant::Ghost)
            .on_press(move || tasks.update(|list| list.retain(|task| task.id != id))),
    ))
    .role(Role::ListItem)
    .style(style().gap(8).align_items(Align::Center).justify_content(Justify::SpaceBetween))
}

#[component]
fn task_list(tasks: Signal<Vec<Task>>) -> impl Element {
    column(for_each(
        tasks,
        |task| task.id,
        move |task| task_row(tasks, task),
    ))
    .role(Role::List)
    .name("Task list")
    .style(
        style()
            .gap(8)
            .padding(space::MD)
            .border_radius(radius::MD)
            .background(color::SURFACE_ALT)
            .border(1, color::BORDER),
    )
}

#[component]
fn tasks_screen(dark: Signal<bool>) -> impl Element {
    let tasks = signal(vec![
        Task { id: 1, title: "Buy milk".to_owned(), done: false },
        Task { id: 2, title: "Write the tutorial".to_owned(), done: false },
        Task { id: 3, title: "Go for a walk".to_owned(), done: false },
    ]);
    let next_id = var(4);
    let draft = signal(String::new());
    let left = memo(move || tasks.with(|list| list.iter().filter(|task| !task.done).count()));

    let add = callback(move |_| {
        let title = draft.get().trim().to_owned();
        if !title.is_empty() {
            let id = next_id.get();
            next_id.set(id + 1);
            tasks.update(|list| list.push(Task { id, title, done: false }));
            draft.set(String::new());
        }
    });

    column((
        header(dark),
        row((
            text_input(draft)
                .label("New task")
                .placeholder("What needs doing?")
                .on_submit(add)
                .style(style().width(240)),
            button("Add")
                .disabled(move || draft.with(|text| text.trim().is_empty()))
                .on_press(add),
        ))
        .style(style().gap(8)),
        show(move || tasks.with(|list| !list.is_empty()), move || task_list(tasks))
            .fallback(|| text("Nothing to do. Add a task above.")),
        text(move || format!("{} left", left.get())).style(style().color(color::TEXT_MUTED)),
    ))
    .style(
        style()
            .padding(24)
            .gap(16)
            .width(pct(100))
            .height(pct(100))
            .background(color::SURFACE)
            .color(color::TEXT),
    )
}

fn app() -> impl IntoView {
    let dark = signal(false);
    provide_theme(
        move || if dark.get() { Theme::dark() } else { Theme::light() },
        move || tasks_screen(dark),
    )
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

## What is new

**Styles use CSS names.** `style()` has methods for text, color, size and borders, named like their CSS properties.

- `font_size(28)` and `font_weight(700)` set the size and weight of the text in a view and everything below it. `font_weight` is a CSS-style number, where 400 is regular and 700 is bold.
- `color(..)` sets the text color. `background(..)`, `border(1, ..)` and `border_radius(..)` style the box itself.
- `flex_grow(1)` lets a view take the free space in a `row`, which pushes the switch to the right. `align_items(Align::Center)` and `justify_content(Justify::SpaceBetween)` align children.

**Theme variables.** `color::SURFACE`, `color::TEXT`, `color::TEXT_MUTED`, `color::SURFACE_ALT`, `color::BORDER`, `color::ACCENT` and the rest are *theme variables*, like `var(--surface)` in CSS: the active theme decides what each one is. `space::MD` and `radius::MD` are the theme's medium spacing and corner radius. Put a variable anywhere a value goes; when the theme changes, every style that uses it follows, with no closure. Use theme variables for most colors, and the same code works in the light and the dark theme. When you need one fixed color, use `rgb(0x336699)`; plain numbers such as `padding(24)` are fixed lengths in logical pixels.

**Image and font bytes.** `image(LOGO)` uses the SVG bytes embedded in this
program. `include_bytes!("logo.svg").as_slice()` supplies a file embedded at
build time. For bytes produced at runtime, `kivact::view::AssetBytes::new(bytes)`
retains an owned `Vec<u8>`. Generated RGBA pixels use
`kivact::draw::Pixels::new(width, height, rgba)`, where `rgba` can also be an
owned vector containing four bytes per pixel. Clone an existing `AssetBytes`
when reusing unchanged bytes. Fonts belong to the theme: start the app with
`App::new("Tasks").theme(Theme::light().with_fonts([include_bytes!("Inter.ttf").as_slice()]))`.
`with_fonts` takes a list of fonts, each one static bytes, a `Vec<u8>` read at
run time or a path.

**Button variants.** A button is the main action by default (`Primary`, filled with the accent color). `.variant(ButtonVariant::Ghost)` gives Remove a quiet look with no fill until you hover it. The other variants are `Secondary`, `Outline`, `Danger` and `Link`.

**Accessible names.** A screen reader announces a control by its role and name, and keyboard and test tools find controls the same way. So every control needs a name:

- Controls with a visible label already have one: the button's label, the checkbox's label, the switch's label. The text field got its name from `.label(..)` in step 4.
- `.name(..)` sets or overrides the name. The Remove buttons would all read "Remove", so each one is named after its task: "Remove Buy milk". The name may be a closure, so it follows the title.
- Elements with no text, such as an image, need `.name(..)` because it is their alternative text. An image without a name is treated as decoration and hidden from assistive technology. The logo here is named "Tasks logo".
- `.role(..)` says what kind of thing a view is. `text(..)` with `Role::Heading` marks the title as a heading. `Role::List` and `Role::ListItem`, together with a name on the list, let a screen reader announce "Task list" and count its items.

**Dark mode.** `provide_theme(theme, || ..)` applies a theme to everything inside it. The theme can be a closure: here it reads the `dark` signal, so the theme changes when the "Dark mode" switch does. Kivact restyles the existing views and rebuilds nothing. `provide_theme` draws nothing by itself, so the top view gets `background(color::SURFACE)` and `color(color::TEXT)`, the way a web page sets them on `body`.

If you do not need a switch, a fixed theme is one line: `App::new("Tasks").theme(Theme::dark())`.

**Text decorations and visibility.** Add `text_decoration(TextDecorationLine::UNDERLINE)` and `text_decoration_style(TextDecorationStyle::Wavy)` to draw a wavy underline. The default pattern is `Solid`; `Dashed` and `Dotted` are also available. To change the pattern with state, make the whole style a closure: `.style(move || style().text_decoration_style(..))`. `visibility(Visibility::Hidden)` preserves the subtree's layout space while excluding it from painting, assistive technology, focus and pointer hits. `Visibility::Visible` restores it. These types live in `kivact::style`. Text color changes interpolate with `TransitionProp::All`.

## Run it

```text
cargo run
```

The window shows the logo, a bold title with a gray subtitle, and a "Dark mode" switch on the right. The tasks sit in a bordered card, each with a checkbox and a quiet Remove button. Turn on the switch: the whole window changes to dark colors at once. Press Tab to move focus through the controls and Space to check a task.

Next: [Step 7: Async](07-async.md)
