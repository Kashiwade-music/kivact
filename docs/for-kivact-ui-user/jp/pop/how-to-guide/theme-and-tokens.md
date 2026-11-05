# Pop のテーマとトークンを設定する

[English](../../../en/pop/how-to-guide/theme-and-tokens.md)

Pop の見た目を導入し、実行中にライトとダークを切り替え、色やフォントを変え、自分のコンポーネントからキットの値を読むための手引きです。

## テーマを導入する

`pop::theme(mode)` は Kivact の `Theme` を返します。`App::theme` に渡すと、ウィンドウ全体の見た目が変わります。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_pop as pop;

fn app() -> impl IntoView {
    column((
        text("Settings"),
        button("Save").variant(kivact::ui::ButtonVariant::Primary),
    ))
    .style(style().gap(12).padding(24))
}

fn main() -> kivact::Result<()> {
    App::new("Settings")
        .theme(pop::theme(ThemeMode::Light))
        .run(app)
}
```

このテーマは、`kivact::ui` の標準コンポーネント（ボタン、入力欄、チェックボックス、タブ、ラジオグループ、スライダー、テーブル、ダイアログ、メニュー、トースト）と、`kivact-ui-pop` のすべてのコンポーネントを描き替えます。キットのアイコンも組み込まれるので、`icon("check")` で Pop のアイコンを描けます。OS の設定は読みません。`ThemeMode::Light` か `ThemeMode::Dark` はアプリが選びます。

## 実行中にライトとダークを切り替える

選択をシグナルに持たせ、ルートを `provide_theme` で包みます。テーマを作るクロージャを渡すと、シグナルが変わるたびにすべてのスタイルが解決し直されます。ビューの作り直しは起きないので、入力中の文字やフォーカスはそのまま残ります。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::style::provide_theme;
use kivact_ui_pop as pop;

fn app() -> impl IntoView {
    let dark = signal(false);
    let theme = move || {
        pop::theme(if dark.get() {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        })
    };
    provide_theme(theme, move || {
        column((
            checkbox("Dark mode", dark),
            text_input("Search", signal(String::new())),
        ))
        .style(
            style()
                .gap(12)
                .padding(24)
                .height(pct(100))
                .background(pop::ColorName::Background)
                .color(pop::ColorName::Text),
        )
    })
}

fn main() -> kivact::Result<()> {
    App::new("Theme switch").run(app)
}
```

`provide_theme` はノードを作らず、何も塗りません。そのため、サブツリーの最上位ノードが自分でページの色を指定します。`pop::ColorName` はキットの色の名前（`Background`、`Surface`、`Line`、`Text`、`TextMuted` など）です。スタイルにそのまま渡すと、ノードが描かれるテーマから値が引かれるので、クロージャを書かなくても切り替えに追従します。

## 色を変える

キットの色は `pop::Tokens` にあります。公開フィールドだけを持つ普通の構造体で、`blue`、`mint`、`yellow`、`pink`、`red`、`navy`、`surface`、`line`、`text` などを含みます。コンポーネントはテーマからこれを読むので、変更すればすべてに反映されます。トークンをコピーして、フィールドを書き換え、`Theme::with` で入れ直します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_pop as pop;

fn brand_theme() -> Theme {
    let theme = pop::theme(ThemeMode::Light);
    let mut tokens = (*theme.ext::<pop::Tokens>()).clone();
    tokens.blue = rgb(0x0f766e);
    let mut colors = theme.colors().clone();
    colors.accent = tokens.blue;
    theme.with(tokens).with_colors(colors)
}

fn main() -> kivact::Result<()> {
    App::new("Teal")
        .theme(brand_theme())
        .run(|| button("Teal").variant(kivact::ui::ButtonVariant::Primary))
}
```

例で 2 か所を変えているのは、2 つの表が関わるためです。`Tokens` はキット自身の色（主ボタンの青、バッジの色相）を持ちます。`theme.colors()` は Kivact 共通のパレットで、スライダーのトラックのアクセントなど、標準の部品が読みます。`pop::theme` はトークンからパレットを一度だけ作るので、あとからトークンを変えてもパレットは動きません。上のように両方を設定してください。

`Tokens::hue(hue)` は、6 つの `pop::Hue`（`Blue`、`Mint`、`Yellow`、`Pink`、`Red`、`Gray`）のいずれかについて、背景色と前景色を返します。バッジ、アバター、統計カードが使う色です。

## 自分のコンポーネントでトークンを読む

スタイルには上のように `pop::ColorName::..` を渡すのが基本です。`Tokens` 構造体そのものを読むのは、色相の 2 色のように値から何かを計算したいときです。スタイルのクロージャの中で読むと、結果がテーマの切り替えに追従します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::use_theme;
use kivact_ui_pop as pop;

fn highlight(content: impl IntoView) -> Node {
    column(content).style(move || {
        let theme = use_theme().get();
        let tokens = theme.ext::<pop::Tokens>();
        let (background, foreground) = tokens.hue(pop::Hue::Yellow);
        style()
            .padding(12)
            .radius(tokens.control_radius)
            .background(background)
            .color(foreground)
    })
}
# fn main() {}
```

単純な色だけなら、キットは不要です。`ColorToken::Surface`、`Text`、`TextMuted`、`Border`、`Accent`、`Danger` は共通のパレットから来て、どのテーマでも動きます。これだけを使うコンポーネントは、他のキットでもそのまま使えます。

## フォントと文字組み

フォントはアプリが持ちます。同梱するファイルを `App::font` で登録し、テーマの文字設定でファミリー名を指定します。キットのカタログは、本文に Inter、日本語に Noto Sans JP を使っています。Pop の 3 つのコンポーネント（`BadgeVariant::Slant` のバッジ、アバター、番号付きの `section_heading`）は `Barlow` というファミリーを指定し、登録がなければ `sans-serif` にフォールバックします。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::style::theme::Typography;
use kivact_ui_pop as pop;

fn themed() -> Theme {
    let theme = pop::theme(ThemeMode::Light);
    let text = Typography {
        family: Some("Inter, 'Noto Sans JP', sans-serif".into()),
        ..theme.text().clone()
    };
    theme.with_text(text)
}

/// `inter` and `noto` are font files you embed, for example
/// `include_bytes!("../assets/Inter.ttf")`.
fn application(inter: &'static [u8], noto: &'static [u8]) -> App {
    App::new("Pop").font(inter).font(noto).theme(themed())
}
# fn main() {}
```

ファミリー名には、ファイル名ではなく、フォントの内部に書かれた名前を使います。

## キットの文言を差し替える

コンポーネントが自分で描く文言（カレンダーの月ボタン、日付の形式エラー、ドロップゾーンの選択ボタンなど）は英語です。`App::messages` で差し替えます。標準コンポーネントは中核のキーを使い、たとえば `date.open` や `date.error.format` です。例は [日付と期間を選ばせる](dates-and-calendars.md) にあります。

## 関連

- [日付と期間を選ばせる](dates-and-calendars.md)
- [テーブルで行を表示する](tables.md)
- すべてのトークンとコンポーネント: `cargo doc -p kivact-ui-pop --open`

Pop テーマでは標準のドロップ先を使います。ファイル選択と複数ファイルを明示的に有効にし、装飾アイコンを一度だけ構築するスロットに渡します。

```rust,no_run
use kivact::prelude::*;

fn upload() -> impl IntoView {
    kivact::ui::dropzone("Upload")
        .multiple()
        .pick_on_press()
        .icon(icon("cloud-upload").style(style().size(24, 24)))
        .hint("PNG, PDF")
        .on_files(|files| {
            for file in files {
                println!("{}", file.path.display());
            }
        })
}
```
