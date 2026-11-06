# 会話とソースコードを表示する

[English](../../en/how-to-guide/chat-and-code.md)

どのキットのテーマでも、チャットとコードには `kivact::ui` を使います。
メッセージ、下書き、応答タスクはアプリが所有します。部品は意味付け、編集、
末尾への追従、クリップボードのフィードバックを担当します。

## 会話の末尾を追う

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{chat_composer, chat_message, chat_thread, ChatMessageStatus, ChatSender};

fn conversation() -> impl IntoView {
    let draft = signal(String::new());
    let messages = signal(vec!["Welcome!".to_owned()]);
    let history = for_index(messages, |_, message| {
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

`chat_thread` は `Role::Log` です。高さに上限を設けてください。読者が上に
スクロールするまでは追加内容に追従し、「Jump to latest」で追従を再開します。
`.viewport_ref(reference)` で実際のスクロール領域を参照できます。
過去の履歴を先頭に追加するときは `.history_revision(version)` を渡します。
これは履歴データの版で、先頭への追加と同じ更新で変更します。読者を末尾へ
移動させず、見ている内容の位置を保ちます。

`chat_message(author)` は参加者の名前を表示し、`.sender(..)` はその名前と
独立した役割です。`.content`、`.avatar`、`.header`、`.media`、`.icon`、
`.actions`、`.footer`、`.metadata`、`.reactions` は View のスロットです。
`.time` と `.status` は補足情報を追加します。`.align`、`.group`、
`.reactions_side` は配置やまとまり方を決め、`.on_press` は本文をキーボードでも
操作できるボタンにします。

応答を待つ間は `typing_indicator()` を表示します。
`chat_thread.streaming(..)` でも履歴の後に同じ表示を置けます。
`.loading_top(..)` は過去の履歴の読み込み中を先頭に表示します。

## 下書きを送信する

Composer は空白を除去した空でない下書きを取り出し、Binding を空にしてから
`on_submit` を呼びます。Enter は送信、Shift+Enter は改行です。IME の変換中の
Enter は送信前に IME が処理します。下書きの端で Up・Down を押すと送信済みの
文章を呼び出し、作成中だった下書きへ戻れます。`.history(false)` で無効にできます。

応答中は `.stop_shown(responding).on_stop(..)` を設定し、停止ハンドラーでアプリの
タスクを取り消します。`.disabled(..)` は入力と操作を無効にします。
ヘッダーの文脈、ヘッダーの操作、フッターの操作、送信操作、Drawer のスロットには
任意の View を置けます。`.status(..)` は `ComposerStatus::Error` または `Warning`
を受け取り、`.status_position(..)` で位置を選べます。

## ソースを選択・コピーする

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

ソースは読み取り専用の一つのネイティブ TextBox です。通常の編集用ショートカットで
選択、移動、コピーできます。コピーのボタンは現在のソース全体を書き込み、成功を
一時的に表示します。`.on_copy(..)` は操作の通知で、コピー自体は部品が行います。

`.highlight(content, f)` は `TextEditing::line_styles` と同じ契約です。
純粋な関数に content、改行を含まない一行、前の行の解析状態を渡し、装飾する
バイト範囲と次の状態を返します。最初の状態は `Default` です。content にキットの
構文パレットを含めることもできます。言語の解析関数と構文パレットはキットに残り、
`.language(..)` は表示用のラベルです。

`.line_numbers(..)`、`.wrap(..)`、`.highlight_lines(..)`、`.diff(..)`、
`.copy_button(..)` はリアクティブな値を受け取ります。`.collapsible(true)`、
`.collapsible_threshold(..)`、`.expanded(binding)` で開閉を設定し、
`.end_content(..)` と `.footer(..)` でヘッダー末尾とフッターに内容を置けます。

## 見た目を変える

各部品の `.variant(value)` は `Look` を通してレシピへ渡されます。
`UiRecipes` には `chat_thread`、`chat_message`、`typing_indicator`、
`chat_composer`、`code_block` のテーブルがあります。レシピは Style を返し、
スロットが内容を供給します。`UiTokens::chat` は入力中のアニメーションの周期、
`UiTokens::code_block` はコードの文字サイズ、行高、コピー成功の表示時間です。
レシピを置き換える場合も、エディターと行番号には同じ文字の寸法を使います。
