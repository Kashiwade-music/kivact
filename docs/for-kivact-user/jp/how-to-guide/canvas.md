# キャンバスで描く

[English](../../en/how-to-guide/canvas.md)

標準のレイアウト要素では表現しにくいもの、たとえばグラフ、図、ゲームの盤面、独自のメーター、画像エディターを表示したいときにキャンバスを使います。図形、パス、テキスト、画像を普通の関数呼び出しで描くと、読んでいるシグナルが変わったときに Kivact が再描画します。このガイドでは、描いた部分をクリック可能にしてスクリーンリーダーにも伝える方法、自前のレンダラーのフレームを表示する方法、ウィンドウなしで画像を書き出す方法も説明します。

## 図形を描く

`canvas` は `Draw` と、そのノードの現在の `Size` を受け取るクロージャを取ります。座標は論理ピクセルで、キャンバス自身の左上を原点にします。

```rust,no_run
use kivact::control::canvas;
use kivact::draw::{Point, Rect};
use kivact::prelude::*;

fn badge() -> impl IntoView {
    canvas(|draw, size| {
        draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), rgb(0x20242c));
        draw.circle(Point::new(48.0, 48.0), 20, rgb(0x6699ff));
        draw.rounded_rect(Rect::new(96.0, 28.0, 120.0, 40.0), 8.0, ColorToken::Accent);
        draw.line(
            Point::new(12.0, 100.0),
            Point::new(size.width - 12.0, 100.0),
            2,
            rgb(0xffffff),
        );
    })
    .style(style().size(240, 120))
    .name("暗い背景に青い円とアクセント色の帯")
}

fn main() -> kivact::Result<()> {
    App::new("Canvas").run(badge)
}
```

- キャンバスは自分の大きさを持ちません。`.style(style().size(w, h))` で大きさを与えるか、レイアウトに引き伸ばさせ、実際の大きさはクロージャの 2 番目の引数から読みます。最初の描画は大きさ 0 で行われ、レイアウト後にもう一度描かれるので、描画は常に `size` を基準にしてください。
- キャンバスには `.name(..)` を付けます。スクリーンリーダーがその絵として読み上げる文字列になります。
- 色は `rgb(0xrrggbb)`、`rgba(0xrrggbb, alpha)`、または `ColorToken::Accent` のようなテーマトークンで指定します。トークンはアクティブなテーマ（ライト、ダーク、独自テーマ）に従います。`Paint` に変換できるものは、ペイントを受け取る場所ならどこでも使えます。色、トークン、グラデーションのいずれも渡せます。
- 線は幅（`2` は 2 ピクセル幅の実線）か、端・接合・破線を指定できる `Stroke` で指定します。

## パスを描く

`Path` は、移動と線分をつないだものです。`fill_path` は塗りつぶし、`stroke_path` は輪郭線を描きます。

```rust,no_run
use kivact::control::canvas;
use kivact::draw::{LineCap, LineJoin, Path, Point, Stroke};
use kivact::prelude::*;

fn wave() -> impl IntoView {
    canvas(|draw, size| {
        let mid = size.height / 2.0;
        let wave = Path::new()
            .move_to(Point::new(8.0, mid))
            .cubic_to(
                Point::new(size.width * 0.3, mid - 40.0),
                Point::new(size.width * 0.6, mid + 40.0),
                Point::new(size.width - 8.0, mid),
            );
        let stroke = Stroke::new(4).cap(LineCap::Round).join(LineJoin::Round);
        draw.stroke_path(&wave, &stroke, ColorToken::Accent);

        let triangle = Path::new()
            .move_to(Point::new(20.0, 20.0))
            .line_to(Point::new(60.0, 20.0))
            .line_to(Point::new(40.0, 50.0))
            .close();
        draw.fill_path(&triangle, rgb(0xe5484d));
        draw.stroke_path(&triangle, &Stroke::new(2).dash([4.0, 3.0]), rgb(0xffffff));
    })
    .style(style().size(240, 120).background(rgb(0x20242c)))
    .name("波と三角形")
}
# fn main() -> kivact::Result<()> { App::new("Paths").run(wave) }
```

`Path` には `quad_to`・`arc`・`ellipse`・`arc_to`（SVG の `A`）・`rect`・`rounded_rect` もあります。角度は度で、0 は右向きです。パスは既定では非ゼロ規則で塗られます。穴を空けたいときは `path.with_fill_rule(FillRule::EvenOdd)` を使います。単純な図形なら、パスを組むより `draw.circle` や `draw.fill_shape(rect, &Shape, paint)` のほうが簡潔です。

## グラデーションで塗る

グラデーションには 2 種類あり、色の置き方が違います。

```rust,no_run
use kivact::control::canvas;
use kivact::draw::{CanvasGradient, Point, Rect, linear_gradient};
use kivact::prelude::*;

fn gradients() -> impl IntoView {
    canvas(|draw, size| {
        // 1. CSS と同じく、塗る図形を基準にします。
        let sky = linear_gradient(180).colors([rgb(0x1e3a8a), rgb(0x93c5fd)]);
        draw.fill_rect(Rect::new(0.0, 0.0, size.width, 60.0), sky);

        // 2. HTML の canvas と同じく、座標で配置します。同じグラデーションを
        //    使った 2 つの図形は、同じ位置で同じ色になります。
        let ramp = CanvasGradient::linear(Point::new(0.0, 0.0), Point::new(size.width, 0.0))
            .stop(0.0, rgb(0xe5484d))
            .stop(1.0, rgb(0x3e63dd));
        draw.fill_rect(Rect::new(0.0, 70.0, size.width, 20.0), ramp.clone());
        draw.circle(Point::new(size.width / 2.0, 110.0), 16, ramp);
    })
    .style(style().size(240, 130))
    .name("グラデーションの見本")
}
# fn main() -> kivact::Result<()> { App::new("Gradients").run(gradients) }
```

`linear_gradient`・`radial_gradient`・`conic_gradient` は、塗る図形に合わせて伸縮します。`CanvasGradient::{linear, radial, conic}` はキャンバス上の点に固定されます。どちらもペイントを受け取る場所（線を含む）で使え、座標で配置するほうは `.extend(GradientExtend::Repeat)` で両端の先まで繰り返せます。

## テキストを描き、測る

`draw.text(text, position, style)` は 1 行を描きます。`TextStyle` の `align` と `baseline` で基準点を変えない限り、位置はテキストの左上です。ラベルの周りの大きさを決めるには、描画と同じ組版で先に測ります。

```rust,no_run
use kivact::control::canvas;
use kivact::draw::{Draw, Point, Rect, TextBaseline, TextStyle};
use kivact::prelude::*;
use kivact::style::TextAlign;

/// ラベルの幅に左右 12 px の余白を足した幅のピル。
fn pill(draw: &mut Draw, label: &str, at: Point) {
    let style = TextStyle {
        size: 12.0,
        weight: 600,
        color: rgb(0xffffff),
        ..TextStyle::default()
    };
    let metrics = draw.measure_text(label, &style);
    let rect = Rect::new(at.x, at.y, metrics.size.width + 12.0, metrics.size.height + 8.0);
    draw.rounded_rect(rect, rect.height / 2.0, ColorToken::Accent);
    draw.text(label, Point::new(at.x + 6.0, at.y + 4.0), style);
}

fn labels() -> impl IntoView {
    canvas(|draw, size| {
        pill(draw, "新着", Point::new(12.0, 12.0));
        pill(draw, "進行中", Point::new(12.0, 44.0));

        // 点を中心に置くときは、測らずに基準点を使います。
        let centered = TextStyle {
            align: TextAlign::Center,
            baseline: TextBaseline::Middle,
            ..TextStyle::default()
        };
        draw.text("中央", Point::new(size.width / 2.0, 100.0), centered);
    })
    .style(style().size(240, 120))
    .name("ラベルの見本")
}
# fn main() -> kivact::Result<()> { App::new("Labels").run(labels) }
```

`TextMetrics` は `size`（`size.width`・`size.height`）と、`baseline` のオフセットを論理ピクセルで返します。折り返すテキストは `draw.layout_paragraph(text, &style, Some(max_width))` で組み、`draw.paragraph(&paragraph, origin, color)` で描きます。`TextStyle` の項目は要素のテキストと同じ語彙（`family`・`weight`・`italic`・`letter_spacing`・`decoration` など）です。

## グループを変形・クリップ・半透明にする

`draw.layer(Layer, |draw| { … })` は、クロージャの中で描いたもの全体に、変形・クリップ・不透明度・合成モード・フィルター・マスク・影を適用します。グループはクロージャが戻ると終わります。レイヤーの変形は、テキスト・画像・クリップにも効きます。

```rust,no_run
# use kivact::control::canvas;
# use kivact::draw::{Layer, Point, Rect, Transform};
# use kivact::prelude::*;
fn spinner() -> impl IntoView {
    canvas(|draw, size| {
        let center = Transform::new()
            .translate(size.width / 2.0, size.height / 2.0)
            .rotate(30);
        draw.layer(Layer::new().transform(center).opacity(0.8), |draw| {
            draw.fill_rect(Rect::new(-20.0, -4.0, 40.0, 8.0), ColorToken::Accent);
        });
    })
    .style(style().size(80, 80))
    .name("スピナー")
}
# fn main() -> kivact::Result<()> { App::new("Layer").run(spinner) }
```

## 状態が変わったら再描画する

クロージャの中でシグナルから読んだものは、すべて依存関係になります。シグナルが変わると、このキャンバスだけが再描画されます。「汚れた」フラグを立てたり、再描画の関数を呼んだりする必要はありません。

```rust,no_run
use kivact::control::canvas;
use kivact::draw::{LineJoin, Path, Point, Stroke};
use kivact::prelude::*;

fn sparkline(values: Signal<Vec<f64>>) -> impl IntoView {
    canvas(move |draw, size| {
        let values = values.get(); // 読むことでこのキャンバスが購読します
        let step = size.width / values.len().saturating_sub(1).max(1) as f64;
        let path = values.iter().enumerate().fold(Path::new(), |path, (i, v)| {
            let point = Point::new(i as f64 * step, size.height * (1.0 - v.clamp(0.0, 1.0)));
            if i == 0 { path.move_to(point) } else { path.line_to(point) }
        });
        draw.stroke_path(&path, &Stroke::new(2).join(LineJoin::Round), ColorToken::Accent);
    })
    .style(style().size(240, 60))
    .name("推移を示す折れ線")
}

fn app() -> impl IntoView {
    let values = signal(vec![0.5_f64]);
    column((
        sparkline(values),
        button("点を追加").on_press(move || {
            values.update(|v| v.push(((v.len() as f64) * 1.7).sin() * 0.5 + 0.5));
        }),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Sparkline").run(app)
}
```

クロージャは、入力だけで結果が決まる純粋な関数に保ってください。シグナルを読み、`draw` のメソッドを呼ぶだけにします。クロージャの中でシグナルに書き込んではいけません。画像はクロージャの外で一度だけ読み込みます（`let logo = Asset::from(bytes)` のあとで `draw.image(&logo, rect)`）。画像は `Asset` の同一性でキャッシュされるため、描画のたびに新しい `Asset` を作ると、描画のたびにデコードし直すことになります。

キャンバスをアニメーションさせるには、`frame_loop` や `animated` でシグナルを動かし、クロージャでそのシグナルを読みます。[値と要素をアニメーションさせる](animation.md) を参照してください。

## 描いた部分をクリック可能にし、アクセシブルにする

キャンバスは 1 つのノードです。描いたピクセルには、*リージョン*として宣言するまで、ポインター・キーボード・スクリーンリーダーにとっての意味がありません。リージョンは、形（当たり判定）、アクセシブルなロールと名前、ハンドラを持ちます。手順は 2 つです。

1. セットアップで、キャンバスに `.region(id, part)` を付けて振る舞いを宣言します。part は `kivact::primitive::part` で作り、通常の要素に part を付けるのと同じように、ロール・名前・状態・ハンドラを持たせます。
2. 描画のクロージャで、同じ `RegionId` に `draw.region(id, rect)` で形を与えます。

```rust,test_harness
use kivact::control::canvas;
use kivact::draw::{Rect, RegionId};
use kivact::prelude::*;
use kivact::primitive::part;

const SWATCHES: [(&str, u32); 3] = [("赤", 0xe5484d), ("緑", 0x30a46c), ("青", 0x3e63dd)];

fn swatches() -> impl IntoView {
    let chosen = signal(0usize);

    let mut board = canvas(move |draw, size| {
        for (i, (_, color)) in SWATCHES.iter().enumerate() {
            let rect = Rect::new(i as f64 * 56.0, 0.0, 48.0, size.height);
            draw.fill_rect(rect, rgb(*color));
            if chosen.get() == i {
                draw.stroke_rect(rect, 3, rgb(0xffffff));
            }
            draw.region(RegionId(i as u64), rect);
        }
    });
    for (i, (name, _)) in SWATCHES.iter().enumerate() {
        board = board.region(
            RegionId(i as u64),
            part(|state| state)
                .role(Role::Button)
                .name(*name)
                .selected(move || chosen.get() == i)
                .on_press(move || chosen.set(i)),
        );
    }

    column((
        board.style(style().size(168, 48)).name("色の見本"),
        text(move || format!("選択中: {}", SWATCHES[chosen.get()].0)),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Swatches").run(swatches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_region_is_a_button_for_tests_and_screen_readers() {
        let screen = kivact::test::render(swatches);
        screen.get_by_text("選択中: 赤");
        screen.get_by_role(Role::Button, "緑").click();
        screen.get_by_text("選択中: 緑");
        assert!(screen.get_by_role(Role::Button, "緑").is_selected());
    }
}
```

リージョンは、宣言した順にキーボードフォーカスを受け取り、`on_press` を通じて Enter と Space に反応し、アクセシビリティツリーではボタンとして現れます。そのため `kivact::test` からロールと名前で探してクリックできます。

知っておくこと:

- **宣言してから描きます。** 描いたのに宣言されていない `RegionId`、または同じレイヤーで 2 回描いた `RegionId` は、エラーとして報告され、無視されます。宣言したのに描かれないリージョンは、単に存在しないものとして扱われます。
- **当たり判定は描いたとおりになります。** リージョンは、描いたレイヤーの変形を保持します。回転や拡大縮小をした図形は、見えている位置で当たります。長方形以外の領域には `draw.region_shape(id, rect, &Shape::circle(r))` や `draw.region_path(id, rect, &path)` を使います。
- **重なりは描画順で決まり**、Tab の順は宣言順です。
- 項目数が変わる場合は、`canvas.regions(items, key, make)` でキー付きのコレクションを宣言できます。項目がリストに残っている間、その状態は保たれます。
- キャンバス全体でポインターに反応したい場合（ブラシ、ドラッグ）は `.on_pointer(|event| …)` を使います。`PointerEvent::pos` はキャンバスの座標で、`PointerEvent::region()` は当たったリージョン（あれば）を返します。
- 編集可能なテキストフィールドを、キャンバスのリージョンの中に置くことはできません。

## 自前のレンダラーのフレームを表示する

アプリが自分で絵を作る場合（エミュレーター、3D ビューポート、動画デコーダーなど）は、ワーカースレッドで描画し、各フレームを `Pixels`（RGBA8、ストレートアルファ）として送り、キャンバスで最新のものを描きます。フレームはチャンネルが UI スレッドへ運ぶので、ポーリングは要りません。

```rust,no_run
use std::sync::Arc;
use std::time::Duration;
use kivact::control::canvas;
use kivact::draw::{ImageOptions, Pixels, Rect};
use kivact::prelude::*;

fn renderer_preview() -> impl IntoView {
    let image = signal(None::<Pixels>);
    let (tx, rx) = async_channel::bounded(2);

    // フレームの大きさは物理ピクセル: 論理サイズに表示倍率を掛けます。
    let scale = kivact::platform::scale_factor();
    let width = (320.0 * scale).ceil().max(1.0) as u32;
    let height = (180.0 * scale).ceil().max(1.0) as u32;

    std::thread::spawn(move || {
        for red in (0..=255u8).cycle() {
            // この単色を、自前のレンダラーの出力に置き換えます。
            let rgba: Arc<[u8]> = [red, 64, 128, 255]
                .repeat(width as usize * height as usize)
                .into();
            if tx.send_blocking(Pixels::new(width, height, rgba)).is_err() {
                break; // ウィンドウが閉じられました
            }
            std::thread::sleep(Duration::from_millis(16));
        }
    });

    spawn(async move {
        while let Ok(mut newest) = rx.recv().await {
            // UI が忙しい間にたまったフレームは読み飛ばします。
            for _ in 0..rx.len() {
                if let Ok(frame) = rx.try_recv() {
                    newest = frame;
                }
            }
            image.set(Some(newest));
        }
    });

    canvas(move |draw, size| {
        if let Some(frame) = image.get() {
            let target = Rect::new(0.0, 0.0, size.width, size.height);
            draw.pixels(&frame, target, ImageOptions::default());
        }
    })
    .style(style().size(320, 180))
    .name("レンダラーのプレビュー")
}

fn main() -> kivact::Result<()> {
    App::new("Renderer").run(renderer_preview)
}
```

- `async-channel` は、自分の `Cargo.toml` に追加する依存です。受信側のタスクを起こすチャンネルなら何でも使えます。
- ウィンドウやコンポーネントがなくなると、受信タスクが破棄されてチャンネルが閉じ、`send_blocking` が `Err` を返します。ワーカーのループはそこで終わらせてください。
- 遅い UI でフレームがたまらないよう、チャンネルは小さくし、上のように古いフレームを捨てます。
- ウィンドウの大きさや表示倍率が変わりうる場合は、新しい大きさをワーカーに伝えて描き直させます。`ImageOptions` では、切り抜き（`source`）、不透明度（`opacity`）、ドット絵向けの最近傍補間（`pixelated`）を指定できます。
- 内容が変わらないフレームは、バッファをコピーし直さず同じ `Arc` を使い回します。
- GPU からの読み戻しが遅すぎる場合は、[キャンバスで GPU テクスチャを共有する](gpu-texture-sharing.md) を参照してください。

## ウィンドウなしで画像を書き出す

`Offscreen` は、同じ `Draw` の語彙で、指定した大きさの RGBA 画像に描画します。`App` もウィンドウも要らないので、作品の書き出し、サムネイル、連番フレーム、コマンドラインツールで使えます。絵を 1 つの関数として書けば、画面表示と書き出しの両方で使えます。

```rust
use kivact::draw::{Draw, Offscreen, Point, Rect, Size};
use kivact::prelude::*;

fn artwork(draw: &mut Draw, size: Size) {
    draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), rgb(0x101820));
    let center = Point::new(size.width / 2.0, size.height / 2.0);
    draw.circle(center, size.height / 3.0, rgb(0xf2aa4c));
}

fn main() -> kivact::Result<()> {
    let mut renderer = Offscreen::new();
    let image = renderer.render(640, 360, artwork)?;

    // 隙間なく詰めた RGBA で、上の行から順に、ストレート（乗算済みでない）アルファです。
    assert_eq!(image.rgba.len(), 640 * 360 * 4);
    assert_eq!(&image.rgba[..4], &[0x10, 0x18, 0x20, 0xff]);
    // `image.rgba` は、好みの PNG や image クレートで符号化してください。
    Ok(())
}
```

同じアプリで `canvas(artwork)` と書けば、書き出すのとまったく同じ関数を画面に表示できます。

- 画像は透明な状態から始まります。不透明にしたければ背景を塗ってください。
- 座標は、倍率 1 の出力ピクセルです。大きさが 0 の場合や、レンダラーが受け付ける範囲を超える場合はエラーになり、`paint` は呼ばれません。
- 描けないコマンド（有限でない座標、負の半径、バッファの長さが大きさと合わない `Pixels`）や、読み込みに失敗した画像・フォントは、キャンバスなら報告して残りを描きますが、`Offscreen` ではエラーとして返ります。保存する前に `Result` を確かめてください。
- テキストにはシステムのフォントが使われます。どのマシンでも同じ出力にするには、同梱するフォントのファイルを `renderer.font(include_bytes!("…"))` で登録し、`Offscreen::theme(theme)` でアプリのテーマを渡します。
- `Offscreen` は、作ったスレッドから出られません。バックグラウンドで書き出すには、`blocking` の中で作ります。

```rust,no_run
use kivact::draw::{Draw, Offscreen, Rect, Size};
use kivact::prelude::*;

fn artwork(draw: &mut Draw, size: Size) {
    draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), rgb(0x101820));
}

fn exporter() -> impl IntoView {
    let export = action(ActionPolicy::Latest, |(width, height): (u32, u32)| async move {
        let image = blocking(move || Offscreen::new().render(width, height, artwork)).await?;
        // ここで `image` をディスクに書き出します（UI スレッドの外のままです）。
        Ok(image.rgba.len())
    });
    column((
        button("1920 × 1080 で書き出す").on_press(move || export.run((1920, 1080))),
        dynamic(move || export.state(), |state| match state {
            AsyncState::Idle => text(""),
            AsyncState::Pending => text("書き出し中…"),
            AsyncState::Ready(bytes) => text(format!("{bytes} バイトを描画しました")),
            AsyncState::Failed(error) => text(format!("失敗: {error}")),
        }),
    ))
}
# fn main() -> kivact::Result<()> { App::new("Export").run(exporter) }
```

## 関連項目

- [値と要素をアニメーションさせる](animation.md): フレームループでキャンバスを動かします。
- [コンポーネントから非同期処理を実行する](async-and-tasks.md): チャンネル、`spawn`、`blocking`。
- [キャンバスで GPU テクスチャを共有する](gpu-texture-sharing.md): 読み戻しなしで GPU レンダラーのフレームを表示します。
- [アプリをテストする](testing.md): ロールでリージョンを探し、スクリーンショットを比べます。
- API リファレンス: `cargo doc -p kivact --open` を実行し、`kivact::draw` と `kivact::control::canvas` を開いてください。
