# フィードバックを表示し、関連する入力をまとめる

[English](../../en/how-to-guide/feedback.md)

empty state、alert、card、input group は `kivact::ui` を使います。各部品の recipe は適用中の theme が提供します。root には `.name(..)` や `.style(..)` などの `Element` modifier も指定できます。

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

## フィードバックの動作を選ぶ

内容がないことを控えめに説明するには empty state、注意が必要なメッセージには alert を使います。画面を切り替えた直後に空の状態を一度だけ通知したいなら `.announce()`、同じ画面で検索結果などが変わるたびに通知したいなら `.announce_changes()` を選びます。メッセージは簡潔にします。

`empty_state(title).announce_changes()` は正規化したメッセージの変更を通知します。初回描画は読み上げず、遅いマウントや表示中の変更を通知し、非表示になった後は再通知を許可します。実際のライブ領域の祖先がある場合は重複を抑制します。同じメッセージを自分の effect から重ねて読み上げる必要はありません。

`.on_press(..)` を指定した card は button、`.selected(binding)` を指定した card は checkbox として動作します。クリック先や選択はアプリケーションの状態に置きます。装飾の画像は `.media(..)` に渡す view のままにし、通常の内容は `card(content)` の必須引数に渡します。

## 見た目を変える

色、余白、コンパクト表示、影、画像の枠は、選んだ kit の `.variant(..)` や通常の `.style(..)` で調整します。操作と読み上げには引き続き標準部品を使います。独自の操作も名前を持つ実際のコントロールにします。

Card の recipe には実際の button または checkbox の Part から `input: Option<PartState>` が渡されます。通常の card は `None` です。`overlay` の recipe は内容の上に操作しない装飾の node を描画します。範囲は配置済みの card の root に従い、クリッピングと角の形はテーマが定義します。

Alert の `header`、`message`、`controls` の recipe は、既存のアイコン、見出しと説明、操作・開閉・閉じるの各 slot を囲む実際の枠を描画します。既定は `Display::Contents` なので、recipe が指定しない限り箱を追加しません。見た目は同じ `AlertState` から決めます。

`AlertState.has_content` は、開閉できるかどうかとは独立に、詳細の slot が実際にあるかを表します。標準の開閉操作は実際の対象に標準の tooltip を付けます。名前とヒントは `alert.expand` と `alert.collapse` のメッセージで翻訳できます。
