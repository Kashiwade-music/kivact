# Basic で設定画面を作る

shadcn/ui の「Lyra」preset（Neutral の配色、Inter、Lucide の icon、角丸 0）を写した公式キットです。標準部品 `kivact::ui` の button・toggle・checkbox・switch・text_input・text_area・select・dialog・menu・tooltip・tabs・radio_group・slider・splitter・split・progress・disclosure・toast_region に Lyra の見た目を載せ、card・field・badge・table・toast など 60 余りの部品を足します。この章では、名前と通知の設定を持つ小さな設定カードを作り、最後にダークモードへ切り替えます。

まずリポジトリのルートでカタログを起動し、見た目を確認してください。左の一覧から部品を選ぶと、説明、実行結果、そのコードが並びます。上のバーの月と太陽の button でテーマを切り替えられます。

```text
cargo run -p kivact-ui-basic-catalog
```

## アプリを作る

[雛形](../../../../templates/basic/README.md) に従ってアプリを作り、`kivact-ui-basic` の path 依存を追加します。この crate の path は checkout 内の `crates/kivact-ui-basic` です。依存は `default-features = false` で指定し、GPU はアプリ側の `kivact/gpu` feature で選びます。`src/main.rs` を次に置き換えます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn panel() -> impl IntoView {
    basic::card()
        .header(basic::card_header("Profile").description("Choose how others see you."))
        .content(move || text("設定はここに入ります。"))
        .style(style().width(360))
}

fn main() -> kivact::Result<()> {
    App::new("Basic で設定画面を作る")
        .theme(basic::theme(kivact::style::ThemeMode::Light))
        .run(|| column(panel()).style(style().padding(24)))
}
```

`cargo run` で白地の枠付きカードが表示されます。`basic::theme(kivact::style::ThemeMode::Light)` が配色、余白、標準部品の見た目（recipe）をまとめて `App` に渡します。`basic::card()` は見出し、内容、フッターを、`.header(..)`、`.content(..)`、`.footer(..)` のスロットで受けます。見出しの題名と説明は `card_header` に渡します。

## フォントを登録する

フォントはアプリのものです。キットは Inter を前提にした寸法にしてありますが、font の bytes は同梱しません。Inter と、コードに使う Geist Mono は OFL のライセンスで配布されています。自分の package の `assets/fonts/` に置き、`App::font` で登録してから、テーマの `text()` の `family` に名前を書きます（`Theme::with_text`）。カタログは `examples/kivact-ui-basic/assets/fonts/` に、出所とライセンスの記録つきで同じものを置いています。

```rust,no_run
use kivact::prelude::*;
use kivact::style::theme::Typography;
use kivact_ui_basic as basic;

fn app(inter: &'static [u8], geist_mono: &'static [u8]) -> App {
    let theme = basic::theme(kivact::style::ThemeMode::Light);
    let text = Typography {
        family: Some("Inter, sans-serif".into()),
        ..theme.text().clone()
    };
    let theme = theme.with_text(text);
    App::new("Basic").font(inter).font(geist_mono).theme(theme)
}
```

`main` では `app(include_bytes!("../assets/fonts/Inter[opsz,wght].ttf"), include_bytes!("../assets/fonts/GeistMono[wght].ttf")).run(..)` のように bytes を渡します。日本語を表示する場合は `"Inter, 'Noto Sans JP', sans-serif"` のように後ろへ足します。family 名は font ファイルの中に書かれた名前で、ファイル名ではありません。登録しない family は system UI のフォントで描かれます。

## 設定カードを組む

名前の入力、通知の switch、種類の select、保存の button を、card の中に field で並べます。

- `kivact::ui::text_input`、`switch`、`select`、`button` は core の標準部品です。操作と a11y は共通で、Lyra の見た目は theme の recipe が決めます。
- `kivact::ui::field(label, control)` は表示名、説明、エラーを入力部品とまとめます。`.hint(text)` と `.error(value)` を渡します。
- 編集中の値は、アプリの signal を部品に渡して持ちます。確定した値は別の signal に分けます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn settings() -> impl IntoView {
    let name = signal(String::from("Ada"));
    let digest = signal(true);
    let language = signal(Some("ja"));
    let saved = signal(String::new());
    let languages = vec![
        ListItem::new("ja", "日本語"),
        ListItem::new("en", "English"),
    ];
    basic::card()
        .header(basic::card_header("Profile").description("Choose how others see you."))
        .content(
            move || column((
                kivact::ui::field("Display name", text_input("Display name", name))
                    .hint("This is shown on your public page."),
                kivact::ui::field("Language", select("Language", languages, language)),
                kivact::ui::field("Weekly digest", switch("Weekly digest", digest)).style(style().direction(kivact::style::Direction::Row)),
                basic::muted(move || format!("Saved: {}", saved.get())),
            ))
            .style(style().gap(16)),
        )
        .footer(
            move || button("Save")
                .icon(icon("check"))
                .on_press(move || saved.set(name.get())),
        )
        .style(style().width(400))
}

fn main() -> kivact::Result<()> {
    App::new("Basic で設定画面を作る")
        .theme(basic::theme(kivact::style::ThemeMode::Light))
        .run(|| column(settings()).style(style().padding(24)))
}
```

起動して名前を書き換え、「Save」を押します。`Saved:` の表示が入力した名前になることを確認してください。入力するだけでは `saved` は変わりません。`text_input("Display name", name)` の `name` は編集中の状態、`on_press` が確定する操作をつなぎます。Tab で入力欄、select、switch、button へ移り、switch は Space、button は Enter/Space で操作します。`icon("check")`（`kivact::control::icon`）は basic の theme が持つ Lucide の icon で、名前を付けない限り装飾として扱われます。

## ダークモードへ切り替える

`ThemeMode::Dark` は Neutral のダークの配色を持ちます。実行中に切り替えるには、テーマを signal から作り、`provide_theme` に closure で渡します。部品の style は現在のテーマを読むので、切り替えても作り直されず、入力した値も残ります。root の背景と文字色は、アプリの側で `basic::Color` から塗ります。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn root() -> impl IntoView {
    let dark = signal(false);
    let digest = signal(true);
    let theme = move || {
        basic::theme(if dark.get() {
            kivact::style::ThemeMode::Dark
        } else {
            kivact::style::ThemeMode::Light
        })
    };
    kivact::view::provide_theme(theme, move || {
        column((
            button("Toggle theme")
                .variant(ButtonVariant::Outline)
                .icon(dynamic(move || dark.get(), |dark| {
                    icon(if dark { "sun" } else { "moon" })
                }))
                .on_press(move || dark.update(|dark| *dark = !*dark)),
            switch("Weekly digest", digest),
        ))
        .style(
            style()
                .size(pct(100), pct(100))
                .padding(24)
                .gap(16)
                .background(basic::Color::Background)
                .color(basic::Color::Foreground),
        )
    })
}

fn main() -> kivact::Result<()> {
    App::new("Basic").run(root)
}
```

button を押すと配色が切り替わり、switch の状態は保たれます。`basic::Color` は描くときのテーマから色を読むので、テーマが変わるたびに解決し直されます。

## 次に読むもの

- 部品を探すとき：カタログ（`cargo run -p kivact-ui-basic-catalog`）の一覧と検索で、部品ごとの例と、実行しているコードを見られます。
- 作業別の手順：[Basic の手順](../../how-to-guide/basic.md)に、テーマ、状態の持ち方、overlay、表とグラフ、フォーム、icon の使い方があります。
- reference：`cargo doc -p kivact-ui-basic --no-deps` で生成する rustdoc に、全部品の型とメソッドがあります。
