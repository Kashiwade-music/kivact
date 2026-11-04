# 値と要素をアニメーションさせる

[English](../../en/how-to-guide/animation.md)

このガイドでは、値を新しい目標へなめらかに動かす方法、フレームごとにコードを実行する方法、要素が現れるとき・消えるときにスライドやフェードをさせる方法を説明します。Kivact のアニメーションはすべて、ディスプレイに駆動される 1 つの時計を共有しています。そのため、アニメーション同士の足並みがそろい、ウィンドウが隠れている間は止まり、テストもしやすくなります。

何をアニメーションさせるかで、道具を選びます。

| やりたいこと | 使うもの |
|---|---|
| 数値・点・長方形を目標へなめらかに近づける | `animated` |
| 要素の見た目（ホバー、押下、色、不透明度、位置）をなめらかに変える | 要素のスタイルの `transition` |
| 要素が現れるときにフェードやスライドで入り、消えるときに出ていく | 要素のスタイルの `enter` と `exit` |
| 毎フレーム自分の計算を行う（物理、時計の文字盤、シミュレーション） | `frame_loop` |

アニメーション関連の名前は `kivact::motion` にあります。スタイルには `Transition` の値として渡します。

## 値を目標へ向けてアニメーションさせる

`animated(target, transition)` は、`target` に追従する値を返します。目標が変わると、値は今の位置から新しい目標へ動きます。シグナルを読むのと同じように `.get()` で読めるので、それを読んでいるものはすべて、ステップごとに更新されます。

```rust,no_run
use kivact::motion::{Easing, Transition, animated};
use kivact::prelude::*;

fn progress() -> impl IntoView {
    let target = signal(0.0_f64);
    let amount = animated(target, Transition::tween(250, Easing::EaseOut));

    column((
        button("満たす").on_press(move || target.set(100.0)),
        button("空にする").on_press(move || target.set(0.0)),
        text(move || format!("{:.0}%", amount.get())),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Progress").run(progress)
}
```

目標には、シグナル、クロージャ（`move || if open.get() { 240.0 } else { 0.0 }`）、定数のどれでも渡せます。満たす途中で「空にする」を押すと、値は今の位置から折り返します。途中で止められたアニメーションが飛ぶことはありません。

`Transition::tween(ミリ秒, easing)` は、決まった時間で動きます。イージングは `Linear`・`EaseIn`・`EaseOut`・`EaseInOut`・`CubicBezier(x1, y1, x2, y2)`（CSS と同じ数値）、または `Easing::steps_end(n)` / `steps_start(n)` です。`Transition::spring(Spring::DEFAULT)` は、物理的なバネのように動きます。時間は決まっておらず、収まったときに終わります。途中で止められたバネは速度を引き継ぎます。`Spring::GENTLE`・`WOBBLY`・`STIFF` はプリセットで、`stiffness`・`damping`・`mass` を自分で指定することもできます。どのトランジションにも `.delay(ms)`、`.repeat(Repeat::Times(3))`（`Repeat::Forever` なら目標が変わるまで繰り返し）、`.alternate()`（1 回おきに逆再生）を付けられます。バネの 1 回分は、収まったところで終わります。`Transition::keyframes([(0.0, 0.0), (0.6, 1.2), (1.0, 1.0)], 400)` は、0 から 1 までのオフセットで指定した値を通り、目標で終わります。

### 値をスタイルで使う

`transform` と `opacity` で要素を動かすのは軽い処理です。レイアウトが変わらないからです。スタイルのクロージャの中で、アニメーションする値を読みます。

```rust,no_run
use kivact::motion::{Spring, Transition, animated};
use kivact::prelude::*;

fn drawer() -> impl IntoView {
    let open = signal(false);
    let offset = animated(
        move || if open.get() { 0.0_f64 } else { -240.0 },
        Transition::spring(Spring::DEFAULT),
    );

    column((
        button("ドロワーを切り替える").on_press(move || open.update(|o| *o = !*o)),
        text("ドロワーの内容").style(move || {
            style()
                .padding(16)
                .background(ColorToken::SurfaceAlt)
                .translate_x(offset.get())
        }),
    ))
    .style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Drawer").run(drawer) }
```

`width`・`height`・余白をアニメーションさせることもできますが、フレームごとにレイアウト計算が走ります。選べるなら transform を使ってください。

### さらに細かく制御する

- `Animated::from(start, target, transition)` は、最初の表示も `start` からアニメーションさせます。通常の `animated` は目標の値から始まります。
- `jump_to(value)` は、値をすぐに設定し、実行中のアニメーションを取り消します。
- `pause()` と `resume()` は、停止と再開です。止まっていた時間は数えません。
- `is_running()` は、動いているかどうかを返す追跡付きの読み取りです。
- セットアップで登録する `.on_complete(|| …)` は、1 回の実行が自然に終わったときに一度だけ呼ばれます。目標が置き換えられたとき、`jump_to` したとき、無限に繰り返しているときは呼ばれません。現在の値と同じ目標では実行が始まらないので、そのときも呼ばれません。
- 値の型には、`f32`、`f64`、それらの最大 4 要素のタプル、`Point`・`Size`・`Rect` が使えます。独自の型は `Animatable` を実装するとアニメーションできます（いくつかの `f64` の成分との相互変換を書きます）。色はここではアニメーションできません。0 から 1 の数値をアニメーションさせて自分で色を混ぜるか、次のスタイルのトランジションを使ってください。

## 要素の見た目をアニメーションさせる

状態の間で見た目を変えたいとき（ホバー、押下、フォーカス、あるいは値が変わるスタイルのクロージャ）は、スタイルにトランジションを付けます。不透明度、単色の背景、枠線の色、transform、影、フィルターを補間します。

```rust,no_run
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::TransitionProp;

fn hover_card() -> impl IntoView {
    column(text("ポインターを重ねてください"))
        .style(
            style()
                .padding(24)
                .background(ColorToken::Surface)
                .hover(style().background(ColorToken::SurfaceAlt).translate_y(-2))
                .transition(TransitionProp::Paint, Transition::tween(150, Easing::EaseOut)),
        )
}
# fn main() -> kivact::Result<()> { App::new("Hover").run(hover_card) }
```

これらのトランジションはホスト側で動かします。コードが書くのは目標のスタイルだけで、フレームごとの処理はありません。

## 要素の登場と退場をアニメーションさせる

`enter` は要素が追加されたときの出発点の見た目、`exit` は取り除かれるときに向かう見た目です。どちらもスタイルの `Paint` トランジションを使います。`show`・`show_some`・リストなど、要素が出入りする仕組みと組み合わせます。

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
                // 右からスライドしてフェードインし、同じ向きに出ていきます。
                .enter(style().translate_x(40).opacity(0.0))
                .exit(style().translate_x(40).opacity(0.0))
                .transition(TransitionProp::Paint, Transition::tween(260, Easing::EaseOut)),
        )
    })
}

fn app() -> impl IntoView {
    let shown = signal(false);
    column((
        button("切り替える").on_press(move || shown.update(|s| *s = !*s)),
        notice(shown),
    ))
    .style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Presence").run(app) }
```

退場中の要素について知っておくこと:

- トランジションが終わるまで描かれ、その後に取り除かれます。取り除かれた時点から、クリックもフォーカスも受けず、アクセシビリティツリーにも現れません。そのため、2 回操作されたり、「消えた」後に読み上げられたりすることはありません。
- 終わるまではレイアウト上の場所を保つので、後ろの要素は最後に詰まります。これを避けるには、その要素に `style().position(Position::Absolute)`（`kivact::style::Position`）を付けます。
- `Paint` トランジションがなければ、要素はすぐに現れ、すぐに消えます。

標準の toast・dialog・popup・tooltip もこの仕組みを使っています。テーマのレシピが、`enter` と `exit` を含むスタイルを返せます。

## フレームごとにコードを実行する

自分の計算（シミュレーション、時計、ゲームループ）が必要なときは `frame_loop` を使います。コールバックは、ティックの時刻、前のティックからの経過時間（`delta`）、フレーム番号、表示時刻の推定値を持つ `FrameInfo` を受け取ります。レートを指定して開始します。

```rust,no_run
use kivact::view::canvas;
use kivact::draw::{Path, Stroke};
use kivact::geometry::{Point};
use kivact::motion::{FrameInfo, FrameRate, frame_loop};
use kivact::prelude::*;

fn clock_hand() -> impl IntoView {
    let angle = signal(0.0_f64);

    let frames = frame_loop(move |frame: FrameInfo| {
        // 10 秒で 1 周。ティックを数えず、アプリの時計から求めます。
        angle.set((frame.now.as_secs_f64() / 10.0 * 360.0) % 360.0);
    });
    on_mount(move || frames.start(FrameRate::Hz(30.0)));

    canvas(move |draw, size| {
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let radians = angle.get().to_radians();
        let tip = Point::new(
            center.x + radians.sin() * 40.0,
            center.y - radians.cos() * 40.0,
        );
        draw.stroke_path(&Path::new().move_to(center).line_to(tip), &Stroke::new(3), ColorToken::Accent);
    })
    .style(style().size(120, 120))
    .name("時計の針")
}
# fn main() -> kivact::Result<()> { App::new("Clock").run(clock_hand) }
```

- **正で有限なレートなら何でも使えます**: `FrameRate::Hz(24.0)`、`Hz(90.0)`、`Hz(144.0)` など。`FrameRate::Display` はディスプレイのリフレッシュごとにティックします。0・負・無限大・NaN のレートはエラーとして報告され、無視されます。
- ディスプレイのリフレッシュレートより高いレートを指定しても、ティックはリフレッシュレートで進みます（60 Hz のディスプレイでは 120 Hz のループも 60 で動きます）。フレームはディスプレイが求めたときにしか作られないためです。
- 動きの量は固定のステップを仮定せず `frame.delta` に掛けると、実際のレートに速さが左右されません。
- もう一度 `start(rate)` を呼ぶとレートが変わり、`stop()` で止まります。`rate()` は現在のレートを返すリアクティブな読み取りです（停止中は `None`）。
- ループはセットアップで作ります。開始と停止は、`on_mount`、イベントハンドラ、`effect` から行います。コンポーネントが破棄されると自動的に止まります。
- ウィンドウを描画できない間（最小化、非表示、アプリの一時停止）は、フレームが作られません。戻ってくると、ループは溜まった分のティックを一度に出さず、そのレートで再開し、アニメーションは止まった位置から続きます。
- 結果はシグナルに書き込み、ビューにそれを読ませます（上の例のとおり）。フレームのコールバックで多くの処理をすると、そのフレームが遅れます。軽く保ち、重い計算は `blocking` に移してください（[コンポーネントから非同期処理を実行する](async-and-tasks.md) を参照）。

## 動きを減らす設定に従う

システムに「動きを減らす」設定をしている人は多くいます。Kivact はその設定を報告し、ほとんどの場合は自動で従います。

- トランジションは、既定で*装飾的*です。動きを減らす設定がオンのときは、すぐに終わります。`animated` は目標へ即座に移り、`enter`/`exit` も即座に適用されます。
- 「処理中」を伝えるスピナーなど、意味を持つ動きには `Transition::tween(…).essential()` を付けます。このトランジションは動き続けます。
- `frame_loop` は、動きを減らす設定では**止まりません**。ゲームやシミュレーションかもしれないからです。装飾だけのループなら、`kivact::platform::is_motion_reduced().get()`（追跡付きの読み取り）を読んで、止めるか遅くしてください。

```rust,no_run
# use kivact::motion::{FrameInfo, FrameRate, frame_loop};
# use kivact::prelude::*;
fn drifting_background() -> impl IntoView {
    let shift = signal(0.0_f64);
    let frames = frame_loop(move |frame: FrameInfo| shift.set(frame.now.as_secs_f64() * 12.0));
    effect(move || {
        if kivact::platform::is_motion_reduced().get() {
            frames.stop();
        } else {
            frames.start(FrameRate::Hz(30.0));
        }
    });
    text(move || format!("{:.0}", shift.get()))
}
# fn main() -> kivact::Result<()> { App::new("Reduced").run(drifting_background) }
```

## アニメーションをテストする

テストの時計は仮想で、フレームとともに進み、止まります。そのため、テストは待ちません。`screen.advance(Duration)` は、すべてのフレームを通して時間を進めます。テストは、動きを減らす設定をオンにして始めることも、画面が動いている間に切り替えることもできます。

```rust,test_harness
use kivact::motion::{Easing, Transition, animated};
use kivact::prelude::*;

fn progress() -> impl IntoView {
    let target = signal(0.0_f64);
    let amount = animated(target, Transition::tween(250, Easing::Linear));
    column((
        button("満たす").on_press(move || target.set(100.0)),
        text(move || format!("{:.0}%", amount.get())),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Progress").run(progress)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use kivact::test::{RenderOptions, render, render_with};

    #[test]
    fn the_value_arrives_after_the_transition() {
        let screen = render(progress);
        screen.get_by_role(Role::Button, "満たす").click();
        assert!(screen.query_by_text("100%").is_none());
        screen.advance(Duration::from_millis(300));
        screen.get_by_text("100%");
    }

    #[test]
    fn reduced_motion_arrives_at_once() {
        let options = RenderOptions::default().reduced_motion();
        let screen = render_with(options, progress);
        screen.get_by_role(Role::Button, "満たす").click();
        screen.get_by_text("100%");
    }
}
```

## 関連項目

- [キャンバスで描く](canvas.md): アニメーションする値やフレームループの状態を描画で読みます。
- [コンポーネントから非同期処理を実行する](async-and-tasks.md): タイマーとバックグラウンド処理。
- [アプリをテストする](testing.md): 仮想時計と `RenderOptions`。
- API リファレンス: `cargo doc -p kivact --open` を実行し、`kivact::motion` と `kivact::style::Style` を開いてください。
