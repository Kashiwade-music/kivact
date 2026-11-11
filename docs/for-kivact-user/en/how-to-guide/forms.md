# Validate a form before saving

[日本語](../../jp/how-to-guide/forms.md)

Edit a draft, validate it with a `memo`, show the problem next to the control
with `field`, and write the committed value only when the user saves. This
guide builds that pattern for one input, then for a form with every standard
input type.

## The shortest working form

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    let name = signal(String::new()); // the draft
    let saved = signal(None::<String>); // the committed value

    let error = memo(move || {
        name.with(|n| n.trim().is_empty()).then(|| "Enter a name".to_owned())
    });
    let save = move || {
        if error.get().is_none() {
            saved.set(Some(name.get().trim().to_owned()));
        }
    };

    column((
        field("Name", text_input("Name", name).on_submit(save))
            .hint("Shown on your profile")
            .error(error),
        button("Save").disabled(move || error.get().is_some()).on_press(save),
        show_some(saved, |name| text(move || format!("Saved: {}", name.get()))),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Profile").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_is_disabled_until_the_name_is_valid() {
        let screen = kivact::test::render(app);
        screen.get_by_text("Enter a name");
        assert!(screen.get_by_role(Role::Button, "Save").is_disabled());

        screen.get_by_role(Role::TextBox, "Name").type_text("Ann");
        assert!(screen.query_by_text("Enter a name").is_none());
        screen.get_by_role(Role::Button, "Save").click();
        screen.get_by_text("Saved: Ann");
    }
}
```

Three pieces make this work:

1. **The signal you give an input is the draft.** `text_input("Name", name)`
   writes every committed edit to `name` immediately. Nothing is "saved" yet;
   `saved` changes only when `save` runs.
2. **The error is a derived value.** A `memo` returning `Option<String>` is
   `None` while the input is valid and `Some(message)` while it is not. It
   re-runs when `name` changes and notifies its readers only if the result
   differs.
3. **`field` connects the pieces.** The label is the control's accessible name,
   the hint describes it, and a `Some` error shows its message under the control,
   marks the control invalid and adds the message to its description. You never
   wire `invalid` or `described_by` yourself.

Why the closure is `move`: the button's handler outlives `app`, so it must own
what it uses. `name`, `saved` and `error` are small `Copy` handles, so the
closure is `Copy` too and the same `save` can go to `on_submit` and `on_press`.
You do not need to clone the handles.

## Submit on Enter

`text_input(..).on_submit(f)` runs `f` when the user presses Enter in the input.
Share one closure with the Save button, as above, so both paths run the same
validation.

- `on_submit` belongs to single-line inputs. In a `text_area`, Enter inserts a
  line break; submit with a button instead.
- Text an IME is still composing is not written to the binding; only the
  confirmed text is. The draft therefore never contains half-converted input.
- To observe composition, for example to hide a hint while it runs, use
  `.on_composition_change(|composing| ..)`.

## Keep a draft apart from the saved model

For more than one value, keep a whole struct as the draft and a second copy as
the saved value. `Signal::field` makes a `Binding` to one field of the struct
for each input.

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Plan {
    Free,
    Pro,
}

#[derive(Clone, PartialEq)]
struct Profile {
    name: String,
    bio: String,
    plan: Plan,
    volume: f64,
    public: bool,
    terms: bool,
}

fn app() -> impl IntoView {
    let saved = signal(Profile {
        name: String::new(),
        bio: String::new(),
        plan: Plan::Free,
        volume: 40.0,
        public: true,
        terms: false,
    });
    let draft = signal(saved.get_untracked());

    let name = draft.project(|p| &p.name, |p| &mut p.name);
    let bio = draft.project(|p| &p.bio, |p| &mut p.bio);
    let plan = draft.project(|p| &p.plan, |p| &mut p.plan).optional();
    let volume = draft.project(|p| &p.volume, |p| &mut p.volume);
    let public = draft.project(|p| &p.public, |p| &mut p.public);
    let terms = draft.project(|p| &p.terms, |p| &mut p.terms);

    let name_error = memo(move || {
        draft.with(|p| p.name.trim().is_empty()).then(|| "Enter a name".to_owned())
    });
    let terms_error = memo(move || {
        (!draft.with(|p| p.terms)).then(|| "Accept the terms to continue".to_owned())
    });
    let valid = memo(move || name_error.get().is_none() && terms_error.get().is_none());
    let changed = memo(move || draft.with(|d| saved.with(|s| d != s)));

    column((
        field("Name", text_input("Name", name)).error(name_error),
        field("Bio", text_area("Bio", bio).wrap(true).rows(2..=5)),
        field(
            "Plan",
            select(
                "Plan",
                vec![ListItem::new(Plan::Free, "Free"), ListItem::new(Plan::Pro, "Pro")],
                plan,
            ),
        ),
        field("Volume", slider("Volume", volume).step(5.0)),
        switch("Public profile", public),
        field("Terms", checkbox("I accept the terms", terms)).error(terms_error),
        row((
            button("Save")
                .disabled(move || !valid.get() || !changed.get())
                .on_press(move || saved.set(draft.get())),
            button("Reset")
                .disabled(move || !changed.get())
                .on_press(move || draft.set(saved.get())),
        ))
        .style(style().gap(8)),
    ))
    .style(style().gap(12).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Profile").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_needs_a_valid_changed_draft_and_reset_restores_the_saved_one() {
        let screen = kivact::test::render(app);
        let save = || screen.get_by_role(Role::Button, "Save");
        assert!(save().is_disabled());

        screen.get_by_role(Role::TextBox, "Name").type_text("Ann");
        assert!(save().is_disabled(), "the terms are not accepted yet");

        // `field` made its label the checkbox's accessible name.
        screen.get_by_role(Role::CheckBox, "Terms").click();
        assert!(!save().is_disabled());

        save().click();
        assert!(save().is_disabled(), "nothing differs from the saved profile");

        screen.get_by_role(Role::TextBox, "Name").type_text(" B");
        assert!(!save().is_disabled());
        screen.get_by_role(Role::Button, "Reset").click();
        assert_eq!(
            screen.get_by_role(Role::TextBox, "Name").value().as_deref(),
            Some("Ann")
        );
    }
}
```

What each piece does:

- `draft.project(get, get_mut)` returns a `Binding<U>` for one field. The first
  closure borrows the field for reading, the second borrows it for writing; you
  do not clone anything. Reading is a memo, so a control re-renders only when its
  own field changes.
- `select` and `radio_group` edit an `Option<K>` (nothing chosen yet is `None`).
  When the model holds a plain value, append `.optional()`: reading gives
  `Some(current)` and writing `None` is ignored. `K` needs `Clone + Eq + Hash`,
  which a small enum derives as above.
- `changed` compares the two copies with `with`, which borrows instead of
  cloning. That is why the struct derives `PartialEq`.
- A `field` replaces the control's accessible name with its own label, so the
  checkbox above is found as "Terms" by assistive technology and by tests.
- Save copies the draft into the saved value, Reset copies back. The saved
  value is what you would send to a server or write to disk.

The `Clone + PartialEq` bounds are what a signal needs to be read with `get`
and compared with `set`; derive both on a form model. Fields that are `String`
or `f64` already satisfy them.

## Pick the right control

| Value | Control | Binding type |
| --- | --- | --- |
| Short text | `text_input(name, Binding<String>)` | `Signal<String>` or a field |
| Long text | `text_area(name, Binding<String>)` | same, Enter inserts a new line |
| One of a few options | `radio_group(name, items, Binding<Option<K>>)` | all options visible |
| One of many options | `select(name, items, Binding<Option<K>>)` | a drop-down list |
| A number in a range | `slider(name, Binding<f64>)` | `.range(..)`, `.step(..)` |
| On or off, applied at once | `switch(label, Binding<bool>)` | |
| A yes or no choice | `checkbox(label, Binding<bool>)` | `.indeterminate(..)` for a mixed state |
| A number typed in | `number_input(..)` | see the `kivact::ui` rustdoc |

The first argument of `text_input`, `text_area`, `select`, `radio_group` and
`slider` is the accessible name and is **not drawn**. Wrap the control in
`field("Label", ..)` to draw a visible label that also becomes the name. The
first argument of `checkbox` and `switch` is their visible label.

## When to show an error

The example shows an error as soon as the draft is invalid, including before the
user has typed anything. To wait until the user tries to save, put the message
behind a flag:

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    let email = signal(String::new());
    let tried = signal(false);

    let problem = memo(move || {
        email.with(|e| !e.contains('@')).then(|| "Enter an email address".to_owned())
    });
    // Visible only after the first attempt.
    let shown = memo(move || if tried.get() { problem.get() } else { None });

    column((
        field("Email", text_input("Email", email)).error(shown),
        button("Subscribe").on_press(move || {
            tried.set(true);
            if problem.get().is_none() {
                println!("subscribed {}", email.get());
            }
        }),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Subscribe").run(app)
}
```

Other rules:

- A control that is invalid for its own reason (`text_input(..).invalid(..)`)
  stays invalid while the `field` has no error; the control is invalid when
  either says so.
- Rejecting a value in the binding's writer (`binding(read, write)` where
  `write` refuses) makes the input snap back to the binding's value. Use that
  for input filters such as "digits only", not for "required".
- `.max_length(n)` limits the input by characters (Unicode scalar values), and a
  composition in progress is never cut.
- `.read_only(true)` allows selecting and copying; `.disabled(..)` blocks input;
  `.purpose(InputPurpose::Password)` hides the text and blocks copy and cut.

## Commit a single field with Enter or blur

For an inline rename you may want the edit to apply only when the user confirms
it. `text_input(..).on_draft_commit(on_commit)` turns the field into a draft: Enter or
leaving the field calls `on_commit` with the text, Escape restores the text from
before the edit.

An inline rename also needs to know which item is being renamed, to open the
field with its text selected and to close it again. `use_inline_edit` keeps
that: `show(id)` starts a rename, `is_open(&id)` tells whether it runs, and
`field(..)` builds the field.

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::use_inline_edit;

fn names() -> impl IntoView {
    let items = signal(vec![(1u32, "Intro".to_owned()), (2, "Outro".to_owned())]);
    let rename = use_inline_edit(signal(None::<u32>));
    column(for_each(
        items,
        |(id, _)| *id,
        move |item| {
            let id = item.get_untracked().0;
            row((
                text(move || item.get().1),
                show(
                    move || !rename.is_open(&id),
                    move || button("Rename").on_press(move || rename.show(id)),
                ),
                show(
                    move || rename.is_open(&id),
                    move || {
                        let api = rename.field(id, "Name", signal(item.get_untracked().1)).on_draft_commit(move |name| {
                            items.update(|all| {
                                if let Some(row) = all.iter_mut().find(|row| row.0 == id) {
                                    row.1 = name;
                                }
                            });
                        });
                        node().part(api.control())
                    },
                ),
            ))
        },
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Rename").run(names)
}
```

The field takes the focus with its text selected when it appears. Enter or
leaving it changed calls the commit function with the text, Escape drops the
edit, and either one closes the field. The field also closes when its row is
removed, so a rename never outlives its item. An input marked `invalid` does
not commit. For a field that stays on screen, use
`text_input(..).on_draft_commit(on_commit)` alone: `draft` replaces `on_submit`, because
Enter belongs to the draft, and `.on_edit(..)` reports `Begin`, `Commit` and
`Cancel`.

## See also

- [Lists and tables](lists-and-tables.md): editing rows of a list, and writing
  a row's checkbox back to the list.
- [Overlays](overlays.md): confirm an unsaved change with a dialog.
- [Internationalization](i18n.md): translate labels, hints and error messages.
- [Test your app](testing.md): drive a form like the tests above.
- Reference: `cargo doc -p kivact-ui --open` for every input's options.

### Mobile input and restoration

Set `.purpose(InputPurpose::Email)` to describe the expected input.
The other purposes are Text, Number, Decimal, Url, Phone, Password and Search.
Password also masks the value and prevents clipboard and accessibility disclosure.
Desktop winit maps Password to its password hint and all other purposes to Normal.
`platform::keyboard_inset()` observes software keyboard occlusion in logical pixels.

Handle `platform::LifecycleEvent::SaveState` with `platform::on_lifecycle` and call
`platform::save_state("draft", bytes)`. Read the previous launch's bytes with
`platform::restored_state("draft")`; missing keys return `None`. Saving never changes
the launch snapshot. Check `platform::capabilities().get().state_restoration`; desktop
backends report false and ignore saves. Virtual tests can request a save with
`screen.script().save_state()` and pass `screen.script().saved_state()` into the next
launch's `RenderOptions::restored_state`.


## Calendar and date/time fields

```rust,no_run
use kivact::prelude::*;
use kivact::i18n::{CalendarDate, DateRange, DateStyle, TimeOfDay, DateTime, TimeStyle};
use kivact::ui::{self, DatePresentation};
fn dates() -> impl IntoView {
    let day = signal(None::<CalendarDate>);
    let range = signal(None::<DateRange>);
    let time = signal(None::<TimeOfDay>);
    let appointment = signal(None::<DateTime>);
    column((
        ui::calendar(day),
        ui::date_input("Date", day).format(DateStyle::Short),
        ui::date_range_input("Period", range),
        ui::time_input("Time", time).format(TimeStyle { seconds: false }),
        ui::date_time_input("Appointment", appointment)
            .time_format(TimeStyle { seconds: true })
            .presentation(DatePresentation::Adaptive),
    ))
}
```

`calendar` accepts single-date and ordered-range signals/bindings. `month` is the
paging binding; `today`, `min`, `max` and `week_start` accept reactive values.
`disabled_dates` rejects dates; `marks` decorates them. `range_span` limits the
inclusive day count. Use `caption(CalendarCaption::Dropdown)` for month/year menus.

Fields parse ISO and locale text through `Locale::parse_date`. Enter commits;
Arrow Down opens the date calendar. Calendar arrows, Home/End and Page Up/Down
navigate; Enter/Space selects. Month/weekday names follow the locale.
`DatePresentation::Adaptive` selects a bottom sheet using `Capabilities.touch`;
`Desktop` and `BottomSheet` select explicitly. Sheet wheels support spinbutton
keys and assistive value actions.

Style them through `UiTokens.calendar`, `UiTokens.date_input`, `UiTokens.time_input`
and the `calendar`, `date_input`, `date_range_input`, `time_input`, `date_time_input`
recipe tables of `UiRecipes`.
## Group actions and toggles

Use `toolbar` for three or more related controls. Its start, center and end
regions share one Tab stop. Arrow keys move focus along its orientation;
Home and End reach the first and last available controls. Give the toolbar
an accessible name even when no visible heading is needed.

```rust,no_run
use kivact::prelude::*;

fn editor_actions() -> impl IntoView {
    let formatting = signal(Vec::<&'static str>::new());
    column((
        toolbar("Document actions")
            .start((button("Undo"), button("Redo")))
            .end((button("Save"),)),
        toggle_group("Formatting", vec![
            ListItem::new("bold", "Bold").with_icon("bold"),
            ListItem::new("italic", "Italic").with_icon("italic"),
        ], formatting),
        button_group("Clipboard", (button("Copy"), button("Paste"))),
    ))
}
```

A toggle group edits `Signal<Option<K>>` for at most one pressed choice, or
`Signal<Vec<K>>` for independent choices; bindings and canonical `Selection`
work too. Arrows move focus without changing the selection. Space, Enter,
click and accessibility activation toggle an item. `ListItem::with_pressed_icon`
replaces its icon while pressed. `icon_only(true)` retains accessible labels.
For exclusive choices that always choose when navigating, use
`radio_group(..).variant(kivact::ui::RadioGroupVariant::Segmented)`.

`button_group` accepts arbitrary controls, separators and text cells through
`IntoChildren`. `button_group_separator()` and `button_group_text(..)` are
non-focusable cells. `.orientation(Orientation::Vertical)` changes both the
layout and arrow navigation of any of these groups. `.disabled(source)` is
the standard `Element` modifier. Themes customize the corresponding
`UiTokens` groups and `UiRecipes` tables; caller styles still override recipes.

`radio_group(...).item_node(|target, item, state| target.with_node(...))` exposes the complete canonical target for advanced attachments, such as a kit keyboard hint. Return that target with modifiers; do not create another native focus or selection owner. `.item(...)` continues to replace only inner content, and the two slots compose.

### Picker interaction and touch sheets

Date and time fields keep a text editor and a separate picker button. Use
`picker_label(..)` to name that button independently. Arrow Down or Alt+Arrow
Down opens the picker and moves focus into it. Selection or dismissal returns
focus to the actual invoker; clicking outside keeps the new outside focus.

A calendar range stages its first endpoint without publishing a partial range.
The second endpoint publishes the ordered range; Escape discards the stage.
In a bottom sheet, day choices and time-wheel steps publish immediately while
Save closes the sheet. A date-and-time sheet uses Save date to advance to Time,
then Save to close. The month heading opens Month/Year wheels; Done returns to
the calendar. Spinbutton Up/Increment increases, Down/Decrement decreases, and
Page Up/Down uses a larger step. A month swipe never chooses its contact date.

Custom interaction code can use `CalendarOptions::activation_guard` to refuse
an activation before either range-picking or single-selection transitions,
without marking the date disabled. `DialogApi::return_focus` accepts a constant
or reactive node reference and snapshots the return target for each opening.
