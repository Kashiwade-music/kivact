# テーマを適用、切り替え、カスタマイズする

[English](../../../en/basic/how-to-guide/theme-and-tokens.md)

このガイドでは、アプリに Basic の見た目を与える方法、実行中にライトとダークを切り替える方法、キットの色を読み書きする方法、標準コンポーネントの描き方を調整する方法、そして Lucide のアイコンの使い方を説明します。[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

## テーマを適用する

`basic::theme(mode)` を `App::theme` に渡します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_basic as basic;

fn main() -> kivact::Result<()> {
    App::new("Basic")
        .theme(basic::theme(ThemeMode::Light))
        .run(|| button("Save"))
}
```

テーマには、Neutral の配色、12 px の文字、角の丸みなし、32 px のコントロール、そして `kivact::ui` の標準コンポーネントすべて（`button`、`toggle`、`checkbox`、`switch`、`text_input`、`text_area`、`select`、`menu`、`dialog`、`tooltip`、`tabs`、`radio_group`、`slider`、`range_slider`、`toast_region`、`splitter`、`split`、`progress`、`disclosure`、`data_table` など）の描き方（recipe）が入っています。キットはこれらのコピーを持ちません。`kivact::ui::button` と `basic::card` が何の仲立ちもなく並べられるのも、テーマを変えると両方の見た目が変わるのも、そのためです。

見た目をウィンドウの一部に限りたいときは、その部分を `kivact::style::provide_theme(theme, || ..)` で包みます。外側の兄弟は、上から来たテーマのままです。

## 実行中にライトとダークを切り替える

固定のテーマではなく、signal を読むクロージャを渡します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_basic as basic;

fn root() -> impl IntoView {
    let dark = signal(false);
    kivact::style::provide_theme(
        move || {
            basic::theme(if dark.get() { ThemeMode::Dark } else { ThemeMode::Light })
        },
        move || {
            column((
                switch("Dark mode", dark),
                button("Save"),
                text_input("Name", signal(String::from("Ada"))),
            ))
            .style(
                style()
                    .size(pct(100), pct(100))
                    .padding(24)
                    .gap(12)
                    .background(basic::ColorName::Background)
                    .color(basic::ColorName::Foreground),
            )
        },
    )
}

fn main() -> kivact::Result<()> {
    App::new("Theme").run(root)
}
```

- マウント済みのコンポーネントは作り直されずにスタイルだけが更新されるので、入力した文字、フォーカス、開いている状態は切り替えの後も残ります。
- `provide_theme` は何も描きません。上のように、部分木のいちばん上のノードにページの色を自分で与えてください。色を持つノードの下にない文字は、`App` のテーマの文字色になります。
- モードの保存はアプリの仕事です。次回の起動でも覚えておきたいときは、`dark` の値を自分で保存し、signal を作るときに読み戻します。
- クロージャの中でテーマを作ってもコストはわずかです。キットはテーマの間で recipe の表とアイコンのセットを共有しているので、変わっていない表は再スタイルを起こしません。

## フォントを選ぶ

キットはフォントを同梱しません。Inter に合わせた寸法で、コードブロックとインラインコードでは `Geist Mono` を求めます。フォントファイルを `App::font` で登録し、テーマの文字の family に名前を書きます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::theme::Typography;
use kivact::style::{Theme, ThemeMode};
use kivact_ui_basic as basic;

fn theme(mode: ThemeMode) -> Theme {
    let theme = basic::theme(mode);
    let text = Typography {
        // 日本語、中国語、韓国語を表示するアプリでは、Inter の後ろに CJK 用の family を足します。
        family: Some("Inter, 'Noto Sans JP', sans-serif".into()),
        ..theme.text().clone()
    };
    theme.with_text(text)
}

fn app(fonts: &[&'static [u8]]) -> App {
    let mut app = App::new("Basic").theme(theme(ThemeMode::Light));
    for &font in fonts {
        app = app.font(font);
    }
    app
}
```

`main` では、`include_bytes!` で読み込んだバイト列を渡します（`app(&[include_bytes!("../assets/Inter[opsz,wght].ttf") as &[u8], include_bytes!("../assets/GeistMono[wght].ttf")])`）。family 名はフォントファイルの中に書かれた名前で、ファイル名ではありません。登録していない family は、システムのフォントに置き換わります。

コンポーネントが自分で表示する文言（Loading、Close、Copy、Next など）は英語です。`App::messages` で置き換えます。キーは `basic.` で始まり、キットの README に一覧があります。

```rust,no_run
use kivact::i18n::Messages;
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn main() -> kivact::Result<()> {
    let messages = Messages::new()
        .with("basic.pagination.next", "Weiter")
        .with("basic.pagination.previous", "Zurück");
    App::new("Basic")
        .theme(basic::theme(kivact::style::ThemeMode::Light))
        .messages(messages)
        .run(|| basic::pagination(signal(0usize), 5usize))
}
```

## 自分のスタイルでキットの色を使う

`basic::ColorName` はパレットの色の名前です（`Background`、`Foreground`、`Card`、`Primary`、`Secondary`、`Muted`、`MutedForeground`、`Accent`、`Destructive`、`Border`、`Input`、`Ring`、`Chart1` から `Chart5`、`Sidebar` 系の色、`Code` の構文色）。色を受け取るスタイルのメソッドにはすべてそのまま渡せます。ノードを描くときに有効なテーマから解決されるので、テーマの切り替えに追従します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn banner() -> impl IntoView {
    text("Heads up").style(
        style()
            .padding(12)
            .background(basic::ColorName::Muted)
            .color(basic::ColorName::MutedForeground)
            .border(1, basic::ColorName::Border),
    )
}
```

トークンから新しい色を計算したいとき（半透明の primary など）は、スタイルのクロージャの中で `basic::Tokens` を読みます。クロージャはテーマが変わるたびに再実行されます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::use_theme;
use kivact_ui_basic as basic;

fn tinted() -> impl IntoView {
    text("Tinted").style(|| {
        let tokens = use_theme().get().ext::<basic::Tokens>();
        style().padding(12).background(tokens.primary.with_alpha(0.1))
    })
}
```

## 色を変える

`basic::Tokens::new(mode)` は、パレットをふつうのフィールドとして返します。変えたいものだけを書き換え、`Theme::with` で載せます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{Theme, ThemeMode, rgb};
use kivact_ui_basic as basic;

fn blue_theme(mode: ThemeMode) -> Theme {
    let mut tokens = basic::Tokens::new(mode);
    tokens.primary = rgb(0x2563eb);
    tokens.primary_foreground = rgb(0xffffff);
    tokens.ring = rgb(0x93c5fd);
    basic::theme(mode).with(tokens)
}

fn main() -> kivact::Result<()> {
    App::new("Blue")
        .theme(blue_theme(ThemeMode::Light))
        .run(|| button("Save"))
}
```

トークンはモードごとにあります。アプリが切り替えを持つなら、両方のモードでカスタマイズしたパレットを作ってください。いくつか知っておくことがあります。

- フィールドは shadcn/ui の CSS 変数に対応します。`background`、`foreground`、`card`、`popover`、`primary`、`secondary`、`muted`、`accent`、`destructive`、`border`、`input`、`ring`、`chart`（5 色）、`sidebar_*` の色、`code`（コードブロックの構文色）です。
- 変えた色は標準コンポーネントにも届きます。recipe が有効なテーマのトークンを読むからです。
- `basic::theme` は、いくつかのトークンをコアのパレット（ページ、文字、accent、枠線、フォーカス、danger）にもコピーします。キット以外のコンポーネント（他のライブラリのものなど）はそのパレットを読みます。変えたトークンとそろえたいときは、パレットも `Theme::with_colors` に渡してください。

## 標準コンポーネントの描き方を調整する

recipe は、コンポーネントの状態をスタイルに変える関数です。すべてのボタンの見た目を変えるには、テーマからキットの `UiRecipes` を取り出し、現在の recipe を呼んで何かを足す関数で recipe を置き換え、表を入れ直します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{Theme, ThemeMode};
use kivact::ui::{ButtonState, Look, UiRecipes};
use kivact_ui_basic as basic;

fn rounded_buttons(mode: ThemeMode) -> Theme {
    let theme = basic::theme(mode);
    let recipes = (*theme.ext::<UiRecipes>()).clone().button(|current| {
        move |theme: &Theme, look: &Look<ButtonState>| current(theme, look).radius(8)
    });
    theme.with(recipes)
}
```

置き換え関数は現在の recipe を受け取るので、キットがするほかのこと（バリアント、サイズ、ホバーやフォーカスの見た目）はそのまま残ります。1 つのインスタンスだけなら、recipe は使わず `.style(..)` を渡してください。呼び出し側のスタイルは、標準コンポーネントでもキット独自のビルダーでも、常にキットの既定より優先されます。

## アイコンを使う

`basic::ICON_NAMES` の Lucide アイコン（`check`、`search`、`settings`、`trash-2` など約 125 個）はすべてテーマのアイコンセットに入っています。コアの `icon(name)` で描きます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn toolbar() -> impl IntoView {
    assert!(basic::ICON_NAMES.contains(&"settings"));
    row((
        button("Download").icon(icon("download")),
        button("Settings").icon(icon("settings")).hide_label(true),
        basic::badge("New").icon(icon("sparkles")),
    ))
    .style(style().gap(8))
}
```

- アイコンは 16 px で、現在の文字色で描かれます。小さなボタンのアイコンの枠のように、それより小さい箱の中では、収まるように縮みます。手で大きさを変えるときは、アイコンにサイズを指定します（`icon("info").style(style().size(24, 24))`）。
- コンポーネントが 1 つの view を受けるスロットを持つ場合（`alert(..).with_icon(..)`、`badge(..).with_icon(..)`、`button(..).with_icon(..)`）は `icon(name)` を渡します。項目がレコードの場合（`ListItem`、`MenuEntry`）は、名前を文字列で渡します（`MenuEntry::item("save", "Save").with_icon("save")`）。
- アイコンは飾りです。アイコンだけのボタンの accessible name はラベルから付くので、`button("Settings").with_icon(..).hide_label(true)` のようにラベルは残してください。
- セットにない名前は何も描きません。デバッグビルドでは、足りない名前が呼び出した場所つきで報告されるので、綴りの間違いに気づけます。
- 自分のアイコンは、SVG のバイト列から `with_icon` でテーマに足せます（ファイル内の線や塗りの色は、文字色に置き換えられます）。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_basic as basic;

static STAR: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#000' stroke-width='2'><path d='M12 3l2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1L3.2 9.5l6.1-.9z'/></svg>";

fn main() -> kivact::Result<()> {
    let theme = basic::theme(ThemeMode::Light).with_icon("star", STAR);
    App::new("Icons")
        .theme(theme)
        .run(|| button("Favorite").icon(icon("star")))
}
```

## 関連項目

- [オーバーレイ](overlays.md): このテーマでのダイアログ、シート、ポップオーバー、メニュー、トースト。
- [ナビゲーション](navigation.md)、[データの表示](data-display.md)、[フォーム](forms.md): キットのコンポーネントの系統。
- ソースつきのすべてのコンポーネント: リポジトリのクローンで `cargo run -p kivact-ui-basic-catalog`。
- API の全体: `cargo doc -p kivact-ui-basic --open`。
