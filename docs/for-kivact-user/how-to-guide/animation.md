# 値を時間で変化させる

画面上の目標値を signal に持ち、`animated` から途中の値を読みます。各 animation が別々の時計や timer を持つ構成にはしません。

```rust,no_run
use kivact::prelude::*;
use kivact::motion::{animated, Easing, Transition};

fn progress() -> impl IntoView {
    let target = signal(0.0_f32);
    let amount = animated(target, Transition::tween(250, Easing::Linear));
    column((
        button("進める").on_press(move || target.set(100.0)),
        text(move || format!("{:.0}%", amount.get())),
    ))
}
```

`animated` は現在の目標値から始めます。初回も別の開始値から動かすなら `Animated::from(start, target, transition)` を使います。途中の目標変更は表示中の値から続きます。`pause`・`resume` と `jump_to` は、それぞれ停止・再開・即時変更の操作です。

装飾的な transition は reduced motion の設定で終点へ進みます。処理中を伝える spinner のように、意味を持つ動き（essential な動き）は `Transition::decorative(false)` にすると reduced motion の下でも動き続けます。`frame_loop` は reduced motion で止まりません。標準の button の loading の表示も reduced motion の下で回ります。完了通知が必要なら setup 中に `on_complete` を登録します。自然に終わった場合と reduced motion による完了で呼ばれ、中断・`jump_to`・owner 破棄は完了に数えません。

背景色や transform などの状態間の見た目は style の transition を使えます。独自の時間計算が必要なときだけ `frame_loop` を選び、owner の lifetime と surface の停止に従います。無限 animation は `on_complete` による次の処理を前提にしないでください。

## 登場と退場

node が表示されたときと取り除かれるときの動きは、style の `enter` と `exit` で書きます。`enter` は mount 直後の見た目（CSS の `@starting-style`）、`exit` は取り除くときに向かう見た目で、どちらも同じ style の `transition(TransitionProp::Paint, ..)` で動きます。

```rust,no_run
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::TransitionProp;

fn notice(shown: Signal<bool>) -> impl IntoView {
    show(shown, || {
        text("保存しました").style(
            style()
                .padding(12)
                .background(ColorToken::SurfaceAlt)
                // 右から滑り込み、右へ消える。
                .enter(style().translate_x(40).opacity(0.0))
                .exit(style().translate_x(40).opacity(0.0))
                .transition(TransitionProp::Paint, Transition::tween(260, Easing::EaseOut)),
        )
    })
}
```

退場中の node は、動き終わるまで元の位置に描かれますが、取り除いた時点で押せず、focus も支援技術の木からも外れます。transition がない場合と、reduced motion の設定で装飾的な transition を使う場合は、すぐに表示され、すぐに消えます。標準部品の toast・dialog・popup・tooltip でも、テーマの recipe が返す style に同じように書けます。
