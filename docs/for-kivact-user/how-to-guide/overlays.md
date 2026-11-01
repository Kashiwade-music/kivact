# Dialog と重ねて表示する UI

標準の dialog は `Binding<bool>` を開閉の正本にします。body は `.content(..)`、footer は `.footer(..)` に、開くたびに実行できる factory で渡します。builder 自身が view です。

```rust,no_run
use kivact::prelude::*;

fn confirmation() -> impl IntoView {
    let open = signal(false);
    column((
        button("確認を開く").on_press(move || open.set(true)),
        dialog("変更を保存しますか", open)
            .content(|| text("保存後も編集を続けられます。"))
            .footer(move || button("閉じる").on_press(move || open.set(false))),
    ))
}
```

`dialog(title, open).width(640)` で dialog ごとの幅を指定できます。幅は theme の recipe より優先し、未指定なら既定 theme では 480 logical pixel です。`Length` の signal や closure も渡せるので、表示中の変更に追従します。theme の最大幅の制約は維持します。

modal は内部へ focus を移し、外へ出さず、閉じると既存の復帰規則を使います。factory の中に作った signal は閉じると破棄されます。再表示しても残す draft は factory の外に置きます。使い終えた view を保存して返す方法では再表示できません。

`DialogKind`（`kivact::ui::DialogKind`）で alert、drawer、popover を選べます。開く button は `dialog(..).attach(trigger)` で付けます。popover はその trigger に付いて開き、閉じると focus が trigger に戻ります。Escape・外側の press による dismiss は `Dismiss` で選びます。独自の枠を作る場合は `interaction::use_dialog` と content/title/description/close_trigger の Part を使い、同じ focus と lifetime の経路を保ちます。focus trap や復帰処理を二重に作らないでください。

短い補足には `tooltip(..).attach(trigger)`、選択操作には `menu(..)`（自前の trigger には `.attach(trigger)`、右クリックには `.attach_context(target)`。開く側と揃えは `.placement(..)`・`.align(..)`、trigger 以上の幅は `.at_least_trigger_width(true)`）や `select` を使います。名前と説明は支援技術にも届くものを付け、開閉・初期 focus・Escape・閉じた後の復帰を [画面のテスト](../getting-started/testing.md)で確認します。
