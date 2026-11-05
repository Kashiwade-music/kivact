# Choose keyed suggestions

`kivact::ui::combobox(label, options, selection)` edits one `Option<K>` or several `Vec<K>` keys. The label names the editor. Typing filters option labels; arrows highlight enabled suggestions, Enter chooses, and Escape dismisses without changing the choice. Native text editing keys continue to edit the text.

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{combobox, select, SelectPresentation};

fn pickers() -> impl IntoView {
    let one = signal(None::<&str>);
    let many = signal(Vec::<&str>::new());
    let options = vec![ListItem::new("red", "Red"), ListItem::new("green", "Green")];
    column((
        combobox("Colors", options.clone(), many).chips().exclude_selected(),
        select("Color", options, one).presentation(SelectPresentation::Adaptive),
    ))
}
```

`.chips()` adds removable chosen labels; Backspace in an empty editor removes the last choice. `.exclude_selected()` hides chosen suggestions, providing a tokenizer over the same selection. Labels needed by selected keys survive changing suggestion sources.

Use `.query(binding)` to control the editor text and `.free_text()` to keep arbitrary text when suggestions close. Keyed selection and free text remain separate values. `.mode(ComboboxMode::Inline)` keeps the list visible, while the default `Popup` uses an anchored list. `.open(binding)` controls popup visibility.

`.search(|query| async move { ... })` supplies asynchronous suggestions returning `kivact::Result<Vec<ListItem<K>>>`. `.debounce(Duration)` defaults to 150 ms for typing; opening fetches immediately. A changed query or a closed popup cancels the previous request. `.filter(|query, label| ...)` replaces local matching; `.placeholder`, `.empty_text` and `.show_clear` configure the editor.

Both combobox and select accept `.presentation(SelectPresentation::BottomSheet)` for touch input, or `Adaptive` to follow host touch capabilities. Keyboard and mouse retain the anchored popup. The sheet uses the same selection and returns focus to the trigger after dismissal.

Themes supply `UiTokens::combobox` and `UiRecipes::combobox`. Its recipes cover `frame`, `input`, `surface`, `option`, `chip`, `remove` and `empty`. Apply `.variant(...)` for a theme-specific look.
