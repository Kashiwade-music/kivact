# フォームの編集と確定を分ける

入力欄には編集中の状態を渡し、保存する値は確定操作で書きます。検証は `memo` で計算し、`field` の `error` に渡します。

```rust,no_run
use kivact::prelude::*;

fn profile() -> impl IntoView {
    let draft = signal(String::new());
    let saved = signal(String::new());
    let error = memo(move || {
        draft.with(|s| s.trim().is_empty()).then(|| "名前が必要です".to_owned())
    });
    column((
        field("表示名", text_input("表示名", draft).placeholder("名前を入力").max_length(40))
            .hint("画面に表示される名前です")
            .error(error),
        button("保存")
            .disabled(move || error.get().is_some())
            .on_press(move || saved.set(draft.get())),
        text(move || format!("保存済み: {}", saved.get())),
    )).style(style().gap(8).padding(16))
}
```

`field(label, control)` は、画面に見える label と control を縦に並べます。label は control の accessible name になります（`labelled_by` で結び、control が `text_input("表示名", ..)` のように自分の名前を持っていても、見える label が名前として使われます。見えるものと読み上げられるものが同じになります）。`.hint(..)` は control の説明になります。`.error(値)` は `Option<String>` を受け、`Some` の間だけ message を表示して、control を `invalid` にし、message を control の説明に加えます。`None` に戻すと message も説明も消えます。`invalid` と `described_by` を自分で結ぶ必要はありません。control は `text_input`、`checkbox`、`switch`、`select` など、どの標準部品でも渡せます。

確定した編集は binding に書かれます。変化に合わせて別の処理をするときは `binding(読む, 書く)` を渡し、`書く` の中で値を書いてから処理します（変化を受ける口は binding だけです）。IME の preedit は binding に書かれず、`on_composition_change` で開始と終了を観測できます。一行の Enter に確定操作を割り当てる場合は `on_submit` を使います。複数行には `text_area(名前, binding)` を使い、Enter を改行にします。

`Binding<String>` の writer で入力を拒否すると、入力欄は binding の値へ戻ります。保存時に検証したいだけなら、編集中の値まで拒否せず、上の例のように確定操作を制約します。長さは Unicode scalar 数で、preedit は途中で切りません。read-only は選択とコピーを許し、disabled は入力を受けません。password は表示を隠し、コピー・切取りと a11y の text snapshot を抑止します。

保存を押したときにだけ error を出したい場合は、`error` に渡す `Option<String>` を確定操作で書く signal にします。入力のたびに検証する場合は上の `memo` のままにします。control が別の理由で invalid のとき（`text_input(..).invalid(..)`）、その値は `field` の error と合わせて扱われます。

行内の checkbox が持つ業務上の真偽値と、一覧の行選択は別の状態です。[一覧と表](lists-tables.md)の選択 binding に業務データを押し込めないでください。独自の外観が必要なら [headless 編集](../../for-component-author/interaction.md) を使います。
