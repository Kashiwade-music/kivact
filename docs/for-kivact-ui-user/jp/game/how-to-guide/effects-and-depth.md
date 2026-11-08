# エフェクトと奥行きを加える

[English](../../../en/game/how-to-guide/effects-and-depth.md)

このガイドは、Game の画面に動き、奥行き、光を加える方法を説明します。ポインターに反応する HUD、3D で傾くカード、回転するパネル、タイトルのグリッチ、シェーダーの背景、起動の手順、スクロールに連動するヘッダーです。以下の 7 つの見本は、カタログのエフェクトのページにあるものです。どれもアプリにそのまま貼れる普通の関数で、キットのテーマと密度に従うよう、キット自身の色と角を使っています。

これらのエフェクトの規則は、キットのパネルの規則と同じです。エフェクトは動きと光を与え、キットのトークンが色と角の直線を与えます。

## キットの色と角を保つ

- 色は、固定の値ではなく `game::ColorName`（`Cyan`、`Yellow`、`Navy`、`Canvas` など）から取ります。`ColorName` は、ノードが描かれるテーマから読まれるので、テーマを切り替えるとエフェクトも従います。
- Game のパネルは角が直線です。`.radius(0)` を保ってください。目印の角には、`.clip_path(game::cut_corner(12.0))` で、向かい合う 2 つの角を切ったパネルにできます。
- 発光や色の調子を作るには、`with_alpha` でトークンから新しい色を 1 つ計算します。例は `use_theme().get().ext::<game::Tokens>().cyan.with_alpha(0.4)` です。キットに無い色にだけ使ってください。

## ポインターの HUD: パララックス

`pointer_position()` は、ウィンドウの中のポインターの位置です。外にあるときは `None` です。これをウィンドウの中心からの差に変え、層ごとに奥行きに応じて動かします。`spring` で動きが滑らかに収まり、奥にある層ほど小さく動きます。

```rust,no_run
use kivact::motion::{Spring, Transition, animated};
use kivact::platform;
use kivact::prelude::*;
use kivact::view::pointer_position;
use kivact_ui_game as game;

fn parallax_hud() -> impl IntoView {
    let pointer = pointer_position();
    let window = platform::window_size();
    // ポインターを中心からの差として表す。各軸で -0.5 から 0.5。
    let offset = animated(
        move || match pointer.get() {
            Some(p) => {
                let size = window.get();
                (p.x / size.width - 0.5, p.y / size.height - 0.5)
            }
            None => (0.0, 0.0),
        },
        Transition::spring(Spring::GENTLE),
    );

    // 層は、奥行きに offset を掛けた分だけ動く。格子は最も小さく、表示は最も大きく動く。
    let layer = |depth: f64, content: Node| {
        content.style(move || {
            let (x, y) = offset.get();
            style().translate(x * depth, y * depth)
        })
    };

    stack((
        layer(
            4.0,
            node().style(style().size(260, 160).border(1, game::ColorName::Cyan)),
        ),
        layer(
            12.0,
            text("TARGET LOCK").style(style().color(game::ColorName::Yellow).padding(12)),
        ),
    ))
}
# fn main() {}
```

動きを減らす設定がオンのとき、バネは動きません。層は新しい位置へすぐに移り、HUD は読みやすいままです。

## 3D で傾くカード

カードはポインターの方へ傾きます。transform に perspective を含めるので、カードは一定の距離から見えます。`backface_visibility` は、直角を超えて回ったときにカードを隠します。カードは、キットの紺のパネル、水色の枠、角を切った形を保ちます。

```rust,no_run
use kivact::draw::Transform;
use kivact::motion::{Spring, Transition, animated};
use kivact::platform;
use kivact::prelude::*;
use kivact::style::BackfaceVisibility;
use kivact::view::pointer_position;
use kivact_ui_game as game;

fn tilt_card() -> impl IntoView {
    let pointer = pointer_position();
    let window = platform::window_size();
    // ポインターの位置から、x と y の軸まわりの回転を度数で表す。
    let tilt = animated(
        move || match pointer.get() {
            Some(p) => {
                let size = window.get();
                ((p.y / size.height - 0.5) * -20.0, (p.x / size.width - 0.5) * 20.0)
            }
            None => (0.0, 0.0),
        },
        Transition::spring(Spring::STIFF),
    );

    column((
        text("OPERATOR").style(style().color(game::ColorName::Text2)),
        text("Pilot 07").style(style().color(ColorToken::Text)),
    ))
    .style(move || {
        let (rx, ry) = tilt.get();
        style()
            .padding(20)
            .size(220, 120)
            .background(game::ColorName::Navy)
            .border(1, game::ColorName::Cyan)
            .radius(0)
            .clip_path(game::cut_corner(12.0))
            .transform(Transform::new().perspective(600.0).rotate_x(rx).rotate_y(ry))
            .backface_visibility(BackfaceVisibility::Hidden)
    })
}
# fn main() {}
```

## 回転するドローアブルのパネル

設定のパネルは、キャンバスの中に絵として描き、全体を回せます。`drawable()` でパネルを作り、`.drawable(panel, content)` で内容をノードにマウントし、ノードの `draw` の中で、transform を持つレイヤーの下に `draw.drawable(panel, 位置)` で描きます。回した絵は、ノードがマウントしているのと同じ view を見せます。仕組みは[ドローアブルのガイド](../../../../for-kivact-user/jp/how-to-guide/drawables.md)で説明します。

```rust,no_run
use kivact::draw::{Layer, Transform};
use kivact::geometry::Point;
use kivact::motion::{Spring, Transition, animated};
use kivact::platform;
use kivact::prelude::*;
use kivact::style::BackfaceVisibility;
use kivact::view::pointer_position;
use kivact_ui_game as game;

fn turning_panel() -> impl IntoView {
    let panel = drawable();
    let pointer = pointer_position();
    let window = platform::window_size();
    let turn = animated(
        move || match pointer.get() {
            Some(p) => (p.x / window.get().width - 0.5) * 60.0,
            None => 0.0,
        },
        Transition::spring(Spring::GENTLE),
    );

    node()
        .style(style().size(260, 180))
        .drawable(panel, || {
            column((
                text("GAIN").style(style().color(game::ColorName::Yellow)),
                text("CONTRAST").style(style().color(game::ColorName::Yellow)),
            ))
            .style(style().padding(16).background(game::ColorName::Navy))
        })
        .draw(move |draw, size| {
            let angle = turn.get();
            draw.layer(
                Layer::new()
                    .transform(
                        Transform::new()
                            .translate(size.width / 2.0, size.height / 2.0)
                            .perspective(700.0)
                            .rotate_y(angle),
                    )
                    .backface_visibility(BackfaceVisibility::Hidden),
                |draw| draw.drawable(panel, Point::new(-130.0, -90.0)),
            );
        })
}
# fn main() {}
```

## グリッチ: 乱数の揺れと変位

フィルターグラフは、その下のものを歪められます。ここでは、タービュレンスの乱数が、タイトルの画素を動かします。`displacement_map` は、乱数の赤と緑の成分で、各画素を `strength` に比例してずらします。強さを変えるとグリッチが出たり消えたりします。タイトルの色はキットの黄色のままです。

```rust,no_run
use kivact::draw::filter::Channel;
use kivact::draw::{Filter, FilterGraph, FilterInput};
use kivact::prelude::*;
use kivact_ui_game as game;

fn glitch(strength: f64) -> Filter {
    let mut graph = FilterGraph::new();
    let noise = graph.turbulence(0.04, 2, 7.0);
    graph.displacement_map(FilterInput::SOURCE, noise, strength, Channel::R, Channel::G);
    Filter::new().graph(graph)
}

fn title(strength: Signal<f64>) -> impl IntoView {
    text("SIGNAL LOST").style(move || {
        style()
            .padding(12)
            .color(game::ColorName::Yellow)
            .filter(glitch(strength.get()))
    })
}
# fn main() {}
```

強さは変位の画素数なので、小さい値は弱い揺れに、大きい値は文字を引き裂くように見えます。

## フォールバックのあるシェーダーの背景

paint のシェーダーが、内容の後ろに走査線の模様を描きます。色は、キットの `navy` のトークンを `draw` のクロージャの中でテーマから読むので、シェーダーもテーマに従います。GPU はシェーダーを実行します。CPU で描く窓は、代わりに `fallback`（単色の紺）を描くので、背景は常に同じに見えます。シェーダーの時計は 30 Hz で進み、動きを減らす設定がオンのときは止まります。

```rust,no_run
use kivact::draw::PaintShader;
use kivact::geometry::Rect;
use kivact::motion::FrameRate;
use kivact::prelude::*;
use kivact_ui_game as game;

const SCANLINES: &str = r#"
struct Params { speed: f32, tint: vec4<f32> }

fn paint(position: vec2<f32>, context: Context, params: Params) -> vec4<f32> {
    let scan = 0.85 + 0.15 * step(0.5, fract(position.y * 0.25 + context.time * params.speed));
    return vec4<f32>(params.tint.rgb * scan, params.tint.a);
}
"#;

fn scanline_background(content: impl IntoView) -> impl IntoView {
    let shader = PaintShader::wgsl(SCANLINES).expect("the scanline shader is valid WGSL");
    stack((
        node()
            .style(style().size(pct(100), pct(100)))
            .draw(move |draw, size| {
                let navy = draw.theme().ext::<game::Tokens>().navy;
                let paint = shader
                    .paint()
                    .param("speed", 1.5)
                    .param("tint", navy)
                    .fallback(navy)
                    .animate(FrameRate::Hz(30.0));
                draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), paint);
            }),
        content,
    ))
}
# fn main() {}
```

`.essential()` は、動きに意味があるときだけシェーダーの paint に付けてください。走査線は飾りなので、動きを減らす設定の中では時計が止まり、1 つのフレームのまま残ります。

## タイムラインと Stagger で起動ログを出す

起動ログの行が、順番に現れます。各行は 1 つの timeline の track を持ち、`Stagger` が各行の開始時刻を与えるので、行ごとに遅延を手で書く必要がありません。

```rust,no_run
use kivact::motion::{At, Easing, Stagger, Timeline, Transition, timeline};
use kivact::prelude::*;
use kivact_ui_game as game;

fn log_line(tl: Timeline, at_ms: f64, label: &'static str) -> impl IntoView {
    let shown = tl.track(0.0_f64);
    tl.to(shown, 1.0, Transition::tween(180, Easing::EaseOut))
        .at(At::Time(at_ms));
    text(label).style(move || {
        style()
            .opacity(shown.get())
            .color(game::ColorName::Cyan)
    })
}

fn boot_log() -> impl IntoView {
    let tl = timeline();
    let stagger = Stagger::new(150.0);
    on_mount(move || tl.play());
    column((
        log_line(tl, stagger.delay(0, 3), "> LINK ESTABLISHED"),
        log_line(tl, stagger.delay(1, 3), "> LOADING MODULES"),
        log_line(tl, stagger.delay(2, 3), "> READY"),
    ))
    .style(style().gap(4).padding(16).background(game::ColorName::Canvas))
}
# fn main() {}
```

## スクロールに連動するヘッダー

長い一覧のヘッダーは、一覧をスクロールすると小さくなるので、読んでいる途中の一覧に広い場所ができます。`scroll_progress` は、先頭で 0、末尾で 1 になるスクロール位置を返し、ヘッダーの高さがそれに従います。

```rust,no_run
use kivact::prelude::*;
use kivact::view::scroll_progress;
use kivact_ui_game as game;

fn log_list(entries: usize) -> impl IntoView {
    let scroller = node_ref();
    let progress = scroll_progress(scroller, Orientation::Vertical);

    let rows: Vec<Node> = (0..entries)
        .map(|i| text(format!("EVENT {i:03}")).style(style().padding(8)))
        .collect();

    column((
        row((game::mono("LOG"), text("ENTRIES"))).style(move || {
            // 一覧の先頭では 56 px、末尾では 32 px。
            style()
                .height(56.0 - progress.get() * 24.0)
                .padding(12)
                .background(game::ColorName::Navy)
                .radius(0)
                .clip_path(game::cut_corner(10.0))
        }),
        scroll(column(rows))
            .node_ref(scroller)
            .style(style().height(200).background(game::ColorName::Canvas)),
    ))
}
# fn main() {}
```

## 効果の選び方

- **利用者の動きに従う動き**（HUD、傾き）: ポインターの値に対するバネ。自分で収まり、フレームループは要りません。
- **順番どおりの動き**（起動ログ）: 開始をずらした timeline。
- **スクロールに従う動き**: タイマーではなく、スクロールの進み具合の値。
- **光と歪み**: キットのトークンを色として使う、フィルターかシェーダー。

## 関連項目

- [一覧とツリーでアイテムを見せる](item-lists-and-selection.md)
- [テーマと密度を設定する](theme-and-density.md): これらの効果が読むトークン。
- [タイムライン、スクロールに連動する動き、共有レイアウト](../../../../for-kivact-user/jp/how-to-guide/timelines-and-scroll.md)
- [キャンバスで描く](../../../../for-kivact-user/jp/how-to-guide/canvas.md)
- すべてのトークンとコンポーネント: `cargo doc -p kivact-ui-game --open`
