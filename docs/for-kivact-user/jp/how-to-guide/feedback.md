# フィードバックを表示し、関連する入力をまとめる

empty state、alert、card、input group は `kivact::ui` を使います。各部品の recipe は適用中の theme が提供します。root には `.name(..)` や `.style(..)` などの `NodeBuilder` modifier も指定できます。

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

empty state の `.icon(..)` と `.actions(..)` は一度だけ構築する view を受け取ります。title は見出しです。`.heading_level(..)` は定数またはリアクティブな `u8` を受け取り、1～6 に制限します。`.announce()` を指定すると mount ごとに一度だけ polite に読み上げます。その後 title や description が変わっても表示だけを更新し、再度読み上げません。

alert は `Role::Alert` と assertive な live region を公開します。表示しても focus は移動せず、アプリが取り除くか利用者が閉じるまで残ります。`.tone(..)` は定数またはリアクティブな意味上の tone を受け取ります。`.icon(..)` と `.actions(..)` で内容を追加し、`.dismissible(..)` で閉じる操作の表示を切り替え、`.dismiss_label(..)` で操作の読み上げ名を指定します。`.on_dismiss(..)` で閉じたことを受け取れます。閉じると core の focus 復元を通じて直前の外側の focus を復元し、callback を呼びます。`AlertVariant::Banner` は幅全体を使う構造を選びます。

詳細を付ける場合は `.content(|| view)` を使います。開くたびに詳細を構築し直します。`.collapsible(..)` で開閉操作を有効にし、`.open(binding)` で開閉をアプリの状態に接続します。`.collapsible(true)` を指定しなければ詳細は表示したままです。

card は必須引数に本文を受け取り、`.header(..)`、`.media(..)`、`.footer(..)` で領域を追加します。`.on_press(..)` を指定するとキーボード操作可能な button になり、`.selected(binding)` を指定すると checkbox になります。操作可能な card には `.name(..)` で読み上げ名を付け、操作できない間は `.disabled(..)` を使います。

`input_group(label, children)` は構築済みの子を受け取ります。`input_group_with(label, || children)` は `.size(..)` を継承する scope 内で一度だけ子を構築します。子自身の size 指定が優先します。`.start(..)` と `.end(..)` は横並びの付加領域、`.block_start(..)` と `.block_end(..)` は上下の付加領域です。`input_addon(text)` は操作しないテキストを表示します。

kit から移すときは empty state の `media` を `icon`、`content` を `actions` にします。card の `image` は `media`、本文は必須引数になります。alert の `action` と banner の `end` は `actions` に、`input_group_text` は `input_addon` にします。`clickable_card` は `card(..).on_press(..)`、`selectable_card` は `card(..).selected(..)` に置き換えます。kit の色、compact 表示、余白、elevation、media の枠、container の見た目は kit 独自の `.variant(..)`、recipe、通常の `.style(..)` で指定します。core に kit ごとの見た目の enum は追加しません。
