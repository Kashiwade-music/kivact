# Show conversations and source code

[日本語](../../jp/how-to-guide/chat-and-code.md)

Use `kivact::ui` for chat and code in every kit theme. The application owns the
messages, draft and response task. The components supply semantics, editing,
scroll following and clipboard feedback.

## Keep a conversation at its latest message

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{chat_composer, chat_message, chat_thread, ChatMessageStatus, ChatSender};

fn conversation() -> impl IntoView {
    let draft = signal(String::new());
    let messages = signal(vec!["Welcome!".to_owned()]);
    let history = for_each(messages, |message| message.clone(), |message| {
        chat_message("Ada")
            .sender(ChatSender::Assistant)
            .content(text(message))
            .time("10:42")
            .status(ChatMessageStatus::Read)
    });
    chat_thread(history)
        .name("Conversation")
        .composer(chat_composer(draft).on_submit(move |message| {
            messages.update(|items| items.push(message));
        }))
        .style(style().width(480).height(360))
}
```

`chat_thread` is a `Role::Log`. Give it a bounded height. It follows appended
content until the reader scrolls away, and its “Jump to latest” action resumes
following. `.viewport_ref(reference)` exposes the actual scroll viewport.
When prepending earlier history, pass `.history_revision(version)`: the value is
the history dataset's version, changed in the same update as that prepend. It
preserves the reader's visual anchor without forcing them to the end.

`chat_message(author)` displays the participant name. `.sender(..)` supplies the
participant role independently. `.content`, `.avatar`, `.header`, `.media`, `.icon`,
`.actions`, `.footer`, `.metadata` and `.reactions` are view slots. `.time` and
`.status` add metadata. `.align`, `.group` and `.reactions_side` control grouping;
`.on_press` makes the body a keyboard-accessible button.

Show `typing_indicator()` while awaiting a response. `chat_thread.streaming(..)`
shows the same indicator after the message history, and `.loading_top(..)` shows
earlier-history loading at the top.

## Submit a draft

The composer trims a nonempty draft, clears its binding and invokes `on_submit`.
Enter sends; Shift+Enter inserts a newline. IME composition consumes Enter before
submission. Up and Down at the draft's edges recall submitted text and restore
the unfinished draft; `.history(false)` turns recall off.

Use `.stop_shown(responding).on_stop(..)` while a response task is running. The
application cancels its task in the stop handler. `.disabled(..)` disables the
input and its actions. Header context, header actions, footer actions, send
actions and drawer slots accept arbitrary views. `.status(..)` accepts a
`ComposerStatus::Error` or `Warning`, placed by `.status_position(..)`.

## Display selectable source and copy it

```rust,no_run
use kivact::prelude::*;
use kivact::style::StyledRange;
use kivact::ui::code_block;

fn snippet() -> impl IntoView {
    let code = signal("fn main() {}\n".to_owned());
    code_block(code)
        .title("main.rs")
        .language("Rust")
        .line_numbers(true)
        .wrap(true)
        .highlight((), |_, line, inside_comment: &bool| {
            let comment = *inside_comment || line.starts_with("/*");
            let ranges = if comment {
                vec![StyledRange {
                    range: 0..line.len(),
                    color: Some(Color::token(ColorToken::TextMuted)),
                    background: None,
                    decoration: None,
                }]
            } else {
                Vec::new()
            };
            (ranges, comment && !line.ends_with("*/"))
        })
        .max_height(240)
}
```

The source remains one read-only native text box: readers can select, navigate
and copy text with ordinary editing shortcuts. The copy button writes the whole
current source and confirms success temporarily. `.on_copy(..)` observes the
request; the component performs the clipboard write.

`.highlight(content, f)` has the `TextEditing::line_styles` contract: the pure
callback gets content, one source line without its break, and the previous
line's parser state. It returns styled byte ranges and the next state. The first
state is `Default`. Theme-aware content can carry a kit's syntax palette; language
parsers and syntax palettes stay in the kits. `.language(..)` is a display label.

`.line_numbers(..)`, `.wrap(..)`, `.highlight_lines(..)`, `.diff(..)` and
`.copy_button(..)` accept reactive values. `.collapsible(true)`,
`.collapsible_threshold(..)` and `.expanded(binding)` configure disclosure.
`.end_content(..)` and `.footer(..)` fill the header's trailing and footer slots.

## Change appearance

Each component accepts `.variant(value)` and sends it to its recipes through
`Look`. `UiRecipes` has `chat_thread`, `chat_message`, `typing_indicator`,
`chat_composer` and `code_block` tables. Recipes return style; view slots supply
content. `UiTokens::chat` sets typing timing, and `UiTokens::code_block` sets the
code font size, line height and copy-feedback duration. Use the same code metrics
for the editor and gutter when replacing recipes.
