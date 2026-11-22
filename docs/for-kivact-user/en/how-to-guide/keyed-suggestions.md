# Choose keyed suggestions

[日本語](../../jp/how-to-guide/keyed-suggestions.md)

`kivact::ui::combobox(items, selected)` edits one `Option<K>` or several `Vec<K>` keys. `.label(..)` names the editor. Typing filters option labels; arrows highlight enabled suggestions, Enter chooses, and Escape dismisses without changing the choice. Native text editing keys continue to edit the text.

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{combobox, SelectPresentation};

#[component]
fn pickers() -> impl Element {
    let one = signal(None::<&str>);
    let many = signal(Vec::<&str>::new());
    let options = vec![ListItem::new("red", "Red"), ListItem::new("green", "Green")];
    column((
        combobox(options.clone(), many).label("Colors").chips().exclude_selected(),
        select(options, one).label("Color").presentation(SelectPresentation::Adaptive),
    ))
}
```

`.chips()` adds removable chosen labels; Backspace in an empty editor removes the last choice. `.exclude_selected()` hides chosen suggestions, providing a tokenizer over the same selection. Labels needed by selected keys survive changing suggestion sources.

Use `.query(binding)` to control the editor text and `.free_text()` to keep arbitrary text when suggestions close. Keyed selection and free text remain separate values. `.mode(ComboboxMode::Inline)` keeps the list visible, while the default `Popup` uses an anchored list. `.open(binding)` controls popup visibility.

`.search(|query| async move { ... })` supplies asynchronous suggestions returning `kivact::Result<Vec<ListItem<K>>>`. `.debounce(Duration)` defaults to 150 ms for typing; opening fetches immediately. A changed query or a closed popup cancels the previous request. `.filter(|query, label| ...)` replaces local matching; `.placeholder`, `.empty_text` and `.show_clear` configure the editor.

Both combobox and select accept `.presentation(SelectPresentation::BottomSheet)` for touch input, or `Adaptive` to follow host touch capabilities. Keyboard and mouse retain the anchored popup. The sheet uses the same selection and returns focus to the trigger after dismissal.

Themes supply the combobox theme variables and `ComboboxTheme`. Its recipes cover `frame`, `input`, `surface`, `option`, `chip`, `remove` and `empty`. Apply `.variant(...)` for a theme-specific look.

For a chooser with your own trigger content, give `select` the trigger view with `.trigger(view)`; it keeps the select's surface, keys and recipes. `.presentation(...)` selects the touch surface and `.open(binding)` makes visibility caller-owned. Keep search and chosen-value signals outside that view to preserve them across popup and drawer openings.

Multiple-choice UI transactions announce accepted additions, removals, and clearing in a polite live region. Rejected selection writes produce no announcement. Override `combobox.token_added`, `combobox.token_removed`, and `combobox.all_removed` through the application message catalog.
