# タイムライン、スクロールに連動する動き、共有レイアウト

[English](../../en/how-to-guide/timelines-and-scroll.md)

このガイドは、次のような場合に使います。1 つのアニメーションが順番に、または同時に進む複数の段階を持つとき。リストの項目を一つずつ少しずらして始めたいとき。ユーザーのスクロール量に動きを合わせたいとき。要素が古い場所から新しい場所へ、同じ要素のように滑って移動するとき。新しい内容が入る前に、古い内容の退場を終えさせたいとき。使うのは `timeline`、リストの `Stagger`、スクロールの進み具合を表す値、`PresenceMode`、`layout_id` です。

| やりたいこと | 使うもの |
|---|---|
| 順番どおりに、または決まった時刻に動く複数の値を 1 つの時計で動かす | track と segment を持つ `timeline()` |
| リストの項目を一定の間隔でずらして始める | `Stagger` |
| スクローラーがどれだけ動いたかに従う、0 から 1 の値 | `scroll_progress` |
| スクローラーの中を要素がどれだけ通過したかに従う、0 から 1 の値 | `view_progress` |
| スクロールの位置などの進み具合の値に従って動く timeline | `Timeline::drive` |
| 古い内容が退場を終えてから新しい内容を入れる | `PresenceMode::Wait` |
| 要素を、前の箱から新しい箱へ滑らせる | `layout_id` と layout transition |

## timeline で値を順番に動かす

timeline は独自の時計を持ちます。セットアップで作り、駆動する値ごとに **track** を、track を目標へ動かす **segment** を加え、`on_mount` またはイベントハンドラーから開始します。track は signal と同じように読めるので、再生中は view が更新されます。

```rust,no_run
use kivact::motion::{At, Easing, Stagger, Timeline, Transition, timeline};
use kivact::prelude::*;

fn boot_line(tl: Timeline, delay_ms: f64, label: &'static str) -> impl IntoView {
    let shown = tl.track(0.0_f64);
    tl.to(shown, 1.0, Transition::tween(200, Easing::EaseOut))
        .at(At::Time(delay_ms));
    text(label).style(move || style().opacity(shown.get()))
}

fn boot_sequence() -> impl IntoView {
    let tl = timeline();
    let stagger = Stagger::new(180.0);
    on_mount(move || tl.play());

    column((
        boot_line(tl, stagger.delay(0, 3), "Booting kernel"),
        boot_line(tl, stagger.delay(1, 3), "Mounting drives"),
        boot_line(tl, stagger.delay(2, 3), "Ready"),
    ))
    .style(style().gap(6).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Boot").run(boot_sequence)
}
```

いつ何が起きるかを決める規則は次のとおりです。

- segment は、動かさない限り**直前の segment のあと**に始まります。`.at(..)` は `At` を取ります。`At::AfterPrevious(ms)`（既定）、`At::WithPrevious(ms)`（直前の segment と同じ始まり）、`At::Time(ms)`（timeline の始まりから）、`At::Label(name, ms)` のいずれかです。`to` のすぐ後に `.at(..)` を呼びます。
- `tl.label("name", At::Time(ms))` は時刻に名前を付けます。`tl.seek_label("name")` でそこへ移動します。定義されていない名前は報告されて無視されます。
- ある時刻での track の値は、すでに始まった最後の segment の値です。そのため、同じ track の 2 つの segment が争うことはありません。
- `tl.call(At::Time(ms), || …)` は、再生がその時刻を通過したときに一度クロージャを実行します。`seek` で時刻をまたいだときは実行されません。
- `tl.play()`、`pause()`、`reverse()`、`restart()`、`finish()`、`seek(ms)`、`set_rate(r)` で、ハンドラーや effect から再生を操作します。`time()` と `progress()`（0 から 1）は追跡される読み取りで、`is_playing()` は再生中かどうかを、`duration()` は 1 回分の長さ（ミリ秒）を返します。
- `tl.repeat(Repeat::Times(3))` または `Repeat::Forever` で timeline 全体を繰り返し、`.alternate()` で 1 回おきに逆向きに再生します。1 つの segment は無限に繰り返せず、そうすると報告されて 1 回だけ再生されます。
- `tl.on_complete(|| …)` は、セットアップで登録すると、再生が自然に終わったときに一度実行されます。進み具合の値で駆動される timeline は終わりません。

動きを減らす設定がオンのとき、essential に指定されていない timeline は、再生されるとすぐに終わります。track はすぐに終点へ飛び、`on_complete` が実行されます。動きが意味を持つ場合、たとえば「処理中」を伝える進捗表示は `tl.essential()` で指定します。

### バネで時間の形を決める

`Transition::spring(..)` の segment は、静止するまでにかかる時間をそのまま使い、その後に `At::AfterPrevious` で続く segment はそれを待ちます。timeline を時間で計画したいときは `Spring::with_duration(ms, bounce)` を使います。これは約 `ms` ミリ秒で静止するバネです。`bounce` は -1 から 1 で、0 は行き過ぎずに止まり、正の値は行き過ぎます。

## リストを Stagger でずらす

`Stagger::new(each_ms)` は、リストの各項目に、前の項目より一段あとの遅延を与えます。`stagger.delay(index, count)` は、1 つの項目の遅延をミリ秒で返します。`.from(StaggerFrom::…)` で数える起点を選び、`.easing(..)` で段の刻み方を決めます。

- `StaggerFrom::Start`（既定）は最初の項目から数えます。
- `StaggerFrom::End` は最後の項目から数えます。
- `StaggerFrom::Center` は中央から外側へ向かいます。
- `StaggerFrom::Edges` は両端から内側へ向かいます。
- `StaggerFrom::Index(i)` は項目 `i` から数えます。

```rust,no_run
use kivact::motion::{Easing, Stagger, StaggerFrom, Transition};
use kivact::prelude::*;
use kivact::style::TransitionProp;

fn chips(labels: &'static [&'static str]) -> impl IntoView {
    let stagger = Stagger::new(60.0)
        .from(StaggerFrom::Center)
        .easing(Easing::EaseOut);
    let count = labels.len();

    let items: Vec<Node> = labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let delay = stagger.delay(i, count);
            text(*label).style(
                style()
                    .padding(8)
                    .background(ColorToken::SurfaceAlt)
                    .enter(style().opacity(0.0).translate_y(8))
                    .transition(
                        TransitionProp::All,
                        Transition::tween(240, Easing::EaseOut).delay(delay),
                    ),
            )
        })
        .collect();

    row(items).style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Chips").run(|| chips(&["Ore", "Fuel", "Food"])) }
```

Stagger は遅延だけを与えます。各項目がどう動くかは項目の transition が決めるので、同じ遅延を `enter`、スタイルの transition、timeline の track のどれとも組み合わせられます。

## スクロールの位置に従わせる

`scroll_progress(scroller, axis)` は、スクロール範囲の始まりで 0、終わりで 1 になる値を返します。スクロールするコンテナーには `.node_ref(..)` で `NodeRef` を付け、スタイルのクロージャでその値を読みます。その方向にスクロールできないコンテナーでは 0 になります。

`view_progress(target, scroller, axis, range)` は、要素がスクローラーの見えている部分をどれだけ通過したかを、0 から 1 の値で測ります。`ViewRange` が、測定の始まりと終わりを決めます。

| `ViewRange` | 0 のとき | 1 のとき |
|---|---|---|
| `Entry` | 要素が末端から入り始める | 要素が完全に中に入る |
| `Contain` | 要素が完全に中に入る | 要素が完全に外へ出る |
| `Cover` | 要素が末端から入り始める | 要素が始端から完全に出る |
| `Exit` | 要素が出始める | 要素が始端から完全に出る |

```rust,no_run
use kivact::prelude::*;
use kivact::view::{ViewRange, scroll_progress, view_progress};

fn release_notes(lines: usize) -> impl IntoView {
    let scroller = node_ref();
    let card = node_ref();
    let header = scroll_progress(scroller, Orientation::Vertical);
    let reveal = view_progress(card, scroller, Orientation::Vertical, ViewRange::Entry);

    let entries: Vec<Node> = (0..lines)
        .map(|i| {
            let entry = text(format!("Entry {i}")).style(style().padding(12));
            if i == 4 {
                // この項目は、ビューポートに入るあいだに、フェードしながら上がってくる。
                entry.node_ref(card).style(move || {
                    let p = reveal.get();
                    style().padding(12).opacity(p).translate_y((1.0 - p) * 24.0)
                })
            } else {
                entry
            }
        })
        .collect();

    column((
        // リストが上端を過ぎるほど、見出しは薄くなる。
        text("Release notes").style(move || {
            let p = header.get();
            style().opacity(1.0 - p * 0.6).translate_y(-p * 8.0)
        }),
        scroll(column(entries)).node_ref(scroller).style(style().height(240)),
    ))
    .style(style().gap(8).padding(16))
}
# fn main() -> kivact::Result<()> { App::new("Notes").run(|| release_notes(40)) }
```

スクロールの値は、ホストが新しいスクロール位置やレイアウトを報告するたびに更新されます。マウントされていない要素や、スクロールできないスクローラーの値は 0 です。

## スクロールで timeline を駆動する

timeline は、自分の時計ではなく進み具合の値から時間をもらうこともできます。`tl.drive(progress)` は `0..=1` を timeline 全体に対応づけます。segment を加えたあとで呼びます。駆動される timeline は再生・一時停止・再開ができず、終わることもないので、動かすのはスクロールだけになります。フレームループを使わずにスクロールに連動するアニメーションを書くにはこうします。

```rust,no_run
use kivact::motion::{Easing, Transition, timeline};
use kivact::prelude::*;
use kivact::view::scroll_progress;

fn reading_progress(scroller: NodeRef) -> impl IntoView {
    let tl = timeline();
    let width = tl.track(0.0_f64);
    tl.to(width, 100.0, Transition::tween(1000, Easing::Linear));
    tl.drive(scroll_progress(scroller, Orientation::Vertical));

    column((
        node().style(move || style().height(4).width(pct(width.get())).background(ColorToken::Accent)),
        scroll(text("Long article")).node_ref(scroller).style(style().height(240)),
    ))
}
# fn main() -> kivact::Result<()> { App::new("Reading").run(|| reading_progress(node_ref())) }
```

## 新しい内容が入る前に古い内容を退場させる

`show`、`show_some`、`dynamic` は、条件が変わると、ある分岐を別の分岐に差し替えます。既定では、古い内容が退場しながら新しい内容が入ります（`PresenceMode::Sync`）。`.presence(..)` で他の 2 つのモードを選べます。

- `PresenceMode::Sync`: 古い内容が退場しながら、新しい内容が入る。
- `PresenceMode::Wait`: 古い内容の退場が終わってから、新しい内容をマウントする。2 つが重なることはない。2 つの状態を同時に描くと衝突する、ダイアログの本文やページ、パネルに使います。
- `PresenceMode::PopLayout`: 古い内容は、すぐにレイアウトから外れ、新しい内容が入る。周りのものは退場を待たない。

退場を待たせるには、内容に `style().exit(..)` と `TransitionProp::All` の transition を付けて、待つべき退場があるようにします。どちらも無ければ、どのモードでも即座に取り外されます。

```rust,no_run
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact::view::PresenceMode;

fn fading(label: &'static str) -> Node {
    text(label).style(
        style()
            .padding(12)
            .background(ColorToken::SurfaceAlt)
            .exit(style().opacity(0.0))
            .transition(TransitionProp::All, Transition::tween(200, Easing::EaseOut)),
    )
}

fn play_pause(playing: Signal<bool>) -> impl IntoView {
    column((
        button("Toggle").on_press(move || playing.update(|p| *p = !*p)),
        show(playing, || fading("Playing"))
            .fallback(|| fading("Paused"))
            .presence(PresenceMode::Wait),
    ))
    .style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Player").run(|| play_pause(signal(true))) }
```

## 要素を新しい場所へ滑らせる

同じものを表す 2 つのノードに、同じ `layout_id` を付けます。一方が、もう一方のあった場所に現れると、Kivact は、同じ id を持っていた直前のノードの箱から、新しいノードを動かします。動きには新しいノードの**layout transition** が使われるので、無ければ何も動きません。id はノードの属性であり、スタイルのプロパティではありません。

```rust,no_run
use kivact::motion::{Spring, Transition};
use kivact::prelude::*;

fn tabs() -> impl IntoView {
    let selected = signal(0_usize);
    row((
        tab("Home", selected, 0),
        tab("Inbox", selected, 1),
        tab("Settings", selected, 2),
    ))
    .style(style().gap(16).padding(12))
}

fn tab(label: &'static str, selected: Signal<usize>, index: usize) -> impl IntoView {
    column((
        button(label).on_press(move || selected.set(index)),
        show(move || selected.get() == index, move || {
            // 1 本の目印が、タブのあいだを移動する。各タブは自分のボタンの下に
            // 目印をマウントし、目印は直前の箱から滑る。
            node().layout_id("tab-marker").style(
                style()
                    .height(3)
                    .background(ColorToken::Accent)
                    .layout_transition(Transition::spring(Spring::STIFF)),
            )
        }),
    ))
}
# fn main() -> kivact::Result<()> { App::new("Tabs").run(tabs) }
```

`layout_id` の規則は次のとおりです。

- id は、同時にマウントされているノードのあいだで一意でなければなりません。同じ id の 2 つ目のノードは、`duplicate layout_id` として一度報告されます。
- layout の動きは箱を新しい位置へ一度だけ動かすので、layout transition は `repeat` と `alternate` を無視します。それらを指定したスタイルは一度報告されます。
- 動きを減らす設定がオンのとき、layout transition は、`.essential()` で指定されていなければすぐに終わります。

## 関連項目

- [値と要素をアニメーションさせる](animation.md): 1 つの値、スタイルのトランジション、登場と退場、フレームループ。
- [画面をレイアウトする](layout.md): この例で使うスクロールコンテナーとサイズ。
- [React や SolidJS の知識を Kivact に対応づける](coming-from-react-or-solid.md): AnimatePresence、layoutId、useScroll、GSAP の timeline と stagger。
- [アプリをテストする](testing.md): テストで timeline を進める仮想の時計。
- API リファレンス: `cargo doc -p kivact --open` で `kivact::motion` と `kivact::view` を開きます。
