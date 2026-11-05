# キー付きの候補を選ぶ

`kivact::ui::combobox(label, options, selection)` は `Option<K>` の単一選択または `Vec<K>` の複数選択を編集します。label は入力欄のアクセシブルな名前です。入力すると候補のラベルを絞り、矢印キーで有効な候補を移動し、Enter で選びます。Escape は選択値を変えずに閉じます。通常のテキスト編集キーは入力欄を編集します。

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

`.chips()` は選択済みのラベルと削除ボタンを表示します。入力欄が空のときの Backspace は最後の選択を削除します。`.exclude_selected()` は選択済みの候補を隠し、同じ選択値の上で tokenizer を作ります。選択中のキーに必要なラベルは候補の取得元が変わっても保持します。

`.query(binding)` は入力テキストを外部から制御し、`.free_text()` は候補を閉じたときも任意のテキストを保持します。キーによる選択と自由なテキストは別の値です。`.mode(ComboboxMode::Inline)` は候補を常に表示し、既定の `Popup` は入力欄に沿ったポップアップを使います。`.open(binding)` はポップアップの開閉を制御します。

`.search(|query| async move { ... })` は `kivact::Result<Vec<ListItem<K>>>` を返す非同期の候補を設定します。`.debounce(Duration)` は入力の待ち時間で、既定は 150 ms です。開いた直後は待たずに取得します。query の変更や閉じる操作で前の要求はキャンセルされます。`.filter(|query, label| ...)` はローカルな照合を置き換え、`.placeholder`、`.empty_text`、`.show_clear` は入力欄を設定します。

combobox と select の `.presentation(SelectPresentation::BottomSheet)` はタッチ操作で下からの sheet を開きます。`Adaptive` はホストのタッチ対応に従います。キーボードとマウスは入力欄に沿ったポップアップを使います。sheet も同じ選択値を編集し、閉じるとトリガーへフォーカスを戻します。

テーマには `UiTokens::combobox` と `UiRecipes::combobox` を設定します。recipe は `frame`、`input`、`surface`、`option`、`chip`、`remove`、`empty` です。テーマ固有の見た目は `.variant(...)` で指定します。

独自の選択内容には `selection_field(label, value_label).content(factory)` を使います。架空の選択キーを作らず select のトリガーと表面の recipe を再利用します。`.presentation(...)` はタッチ時の表面を選び、`.open(binding)` は表示状態を呼び出し側で管理します。検索と選択値の binding は content factory の外で作り、ポップアップとドロワーを開き直しても保持します。
