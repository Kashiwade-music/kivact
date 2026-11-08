# 3D の変換と奥行きを使う

[English](../../en/how-to-guide/3d-and-depth.md)

このガイドでは、要素を 3 次元で傾けたり、回したり、動かしたりする方法を説明します。裏返るカード、ポインターの方へ傾くパネル、奥行きの違う層などが例です。Kivact はこれらに CSS Transforms 2 を使います。同じ関数、`perspective`、`transform-style: preserve-3d`、`backface-visibility` を備えています。CPU と GPU の描画は同じ絵になり、ポインターのヒットテストも、変換の後に見えている位置で当たります。

## 空間の中で要素を回す

スタイルで使える変換のプロパティは、`translate_x`、`translate_y`、`translate_z`、`rotate`（平面、z 軸まわり）、`rotate_x`、`rotate_y`、`scale` です。角度は度で指定します。遠近がないと平面的に見えるので、x 軸や y 軸まわりの回転は潰れて見えるだけです。奥行きを見せるには、親に `perspective`（見る人の距離、論理ピクセル）を与えます。

```rust,no_run
use kivact::prelude::*;

fn tilted_card() -> impl IntoView {
    let angle = signal(0.0_f64);

    column((
        button("15 度回す").on_press(move || angle.update(|a| *a += 15.0)),
        column(text("カード"))
            .style(move || {
                style()
                    .size(200, 120)
                    .padding(16)
                    .radius(12)
                    .background(ColorToken::Surface)
                    .rotate_x(20.0)
                    .rotate_y(angle.get())
            }),
    ))
    // 遠近は親に付けます。子は 800 px の距離から見たように描かれます。
    .style(style().gap(12).padding(24).perspective(800.0))
}
# fn main() -> kivact::Result<()> { App::new("傾いたカード").run(tilted_card) }
```

- `perspective` は、子を立体に見せたい要素に付けます。800 は、少し後ろに下がったカメラのような見え方です。小さい値ほど傾きが強調されます。
- `perspective_origin(x, y)` は、見る人が向いている点を箱に対して動かします。既定は中央です。
- `transform_origin(x, y)` は、要素を回す中心の点を動かします。既定は中央です。`transform_origin_z` で奥行きを指定します。
- 変換を 1 つの値として書くには、`kivact::draw::Transform` を `style().transform(..)` に渡します（例: `Transform::new().perspective(800.0).rotate_y(angle)`）。関数は CSS と同じく、書いた順に適用されます。`matrix_3d` は 4×4 の行列を受け付けますが、最下行は `(0, 0, 0, 1)` でなければなりません。遠近は `perspective` プロパティか `perspective(..)` で表してください。一般の遠近の行はサポートしていません。

## 子を一つの 3D 空間に置く

既定では、各要素は親の平面に平らに描かれます。表と裏を持つカードでは、表と裏が一つの 3D 空間を共有する必要があるので、コンテナに `transform_style(TransformStyle::Preserve3d)` を使います。すると子は、コンテナに対する奥行きと回転を保ったまま置かれます。

```rust,no_run
use kivact::motion::{Spring, Transition, animated};
use kivact::prelude::*;
use kivact::style::{BackfaceVisibility, TransformStyle};

fn flip_card() -> impl IntoView {
    let flipped = signal(false);
    let turn = animated(
        move || if flipped.get() { 180.0_f64 } else { 0.0 },
        Transition::spring(Spring::GENTLE),
    );

    let front = column(text("表")).style(
        style()
            .size(200, 120)
            .padding(16)
            .background(ColorToken::Accent)
            .backface_visibility(BackfaceVisibility::Hidden),
    );
    let back = column(text("裏")).style(
        style()
            .size(200, 120)
            .padding(16)
            .background(ColorToken::Surface)
            .backface_visibility(BackfaceVisibility::Hidden)
            .rotate_y(180.0),
    );

    column((
        button("裏返す").on_press(move || flipped.update(|f| *f = !*f)),
        stack((front, back)).style(move || {
            style()
                .transform_style(TransformStyle::Preserve3d)
                .rotate_y(turn.get())
        }),
    ))
    .style(style().gap(12).padding(24).perspective(800.0))
}
# fn main() -> kivact::Result<()> { App::new("裏返るカード").run(flip_card) }
```

- `Preserve3d` は、表と裏を持つ要素に付けます。その子が 3D 空間に置かれるので、裏の `rotate_y(180)` によって裏が反対向きになります。
- `perspective` は、そのコンテナの親に付けます。よく使われる CSS の書き方と同じです。コンテナ自身の回転が遠近で見え、中の表と裏は同じ 3D 空間を共有します。
- グループは preserve-3d の空間を壊し、子を一つの平面に平らにします。次のものを持つ要素はグループになります: 1 未満の `opacity`、`filter`、`mask`、`clip_path`、`backdrop_filter`、`isolation(Isolation::Isolate)`。空間を共有させたい表と裏には、これらを付けないでください。

## 裏面を隠す

`backface_visibility(BackfaceVisibility::Hidden)` は、要素の裏が見る人の方を向いているときに、その要素を隠します。裏返るカードでは、表と裏がそれぞれ自分の裏を隠すので、見る人の側の面だけが描かれます。既定の `Visible` では、90 度を超えて回したカードは裏が鏡像として見えます。

## ポインターに追従させる

`pointer_position()` は、ポインターが最後にあった位置を、ウィンドウの論理座標で返します。その下に何があるかとは関係ありません。ポインターがウィンドウの外にあるときは `None` になります。`window_size()` と組み合わせると、-1 から 1 の値に変換できます。その値を `animated` とばねに通すと、傾きがなめらかにポインターに追従し、ポインターの向きが変わっても勢いを保ちます。

```rust,no_run
use kivact::motion::{Spring, Transition, animated};
use kivact::platform::window_size;
use kivact::prelude::*;
use kivact::view::pointer_position;

fn tilt_card() -> impl IntoView {
    let pointer = pointer_position();
    let window = window_size();

    // ポインターの位置を、ウィンドウの中で -1..1 にしたもの。外にあるときは中央とみなします。
    let aim = animated(
        move || {
            let size = window.get();
            let point = pointer.get().map_or(Point::new(size.width / 2.0, size.height / 2.0), |p| p);
            Point::new(
                (point.x / size.width.max(1.0)) * 2.0 - 1.0,
                (point.y / size.height.max(1.0)) * 2.0 - 1.0,
            )
        },
        Transition::spring(Spring::GENTLE),
    );

    column(
        column((text("ポインターを動かす"), text("カードがそちらへ傾きます。")))
            .style(move || {
                let a = aim.get();
                style()
                    .padding(24)
                    .radius(16)
                    .background(ColorToken::Surface)
                    .rotate_x(-a.y * 12.0)
                    .rotate_y(a.x * 12.0)
                    .translate_z(20.0)
            }),
    )
    .style(style().padding(40).perspective(600.0))
}
# fn main() -> kivact::Result<()> { App::new("傾くカード").run(tilt_card) }
```

- ポインターの位置はウィンドウの座標なので、例ではカードではなくウィンドウの大きさと比べています。傾きの上限（ここでは 12 度）は、レイアウトに合わせて決めてください。
- `translate_z` は要素を見る人の方へ持ち上げます。20 px は、親の平面より少し手前に出すという意味です。視差を出すには、奥の層に負の `translate_z` を与え、動きを大きくします。
- ポインターがウィンドウの外へ出ても、ばねは速度を保ったまま傾きを中央へ戻します。

## 注意点

- **ヒットテストは変換に従います。** 回転させたり持ち上げたりした要素は、描かれている位置で当たります。
- **混み合った 3D 空間は平らに描かれます。** preserve-3d の空間に 256 を超える参加要素があると、その空間全体が、文書の順による平らな重なりとして描かれ、ヒットテストもされます。大きな画面では、3D の面を少なく大きくするか、いくつかの空間に分けてください。
- **扱えるのは線形の行列と遠近だけです。** `matrix_3d` は最下行が `(0, 0, 0, 1)` の行列を受け付けます。ほかの最下行は報告され、置き換えられます。
- **変換はレイアウトを動かしません。** 要素を回したり持ち上げたりしても、レイアウトのエンジンは動きません。2D で同じ要素を動かすのとほぼ同じ費用です。
- **動いている層は少しぼけることがあります。** 3D の要素が動いている間は、小数のピクセル位置に置かれるため、輪郭がわずかにぼやけることがあります。止まると、もとの鮮明さに戻ります。

## 関連項目

- [値と要素をアニメーションさせる](animation.md): ばね、トランジション、`animated`。
- [キャンバスで描く](canvas.md): キャンバスの中の 3D の層。
- [エフェクトを使う](effects.md): 同じ要素へのフィルター、マスク、ブレンド。
- API リファレンス: `cargo doc -p kivact --open` を実行し、`kivact::style` と `kivact::draw::Transform` を見てください。
