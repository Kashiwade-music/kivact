# Pop のテーマと変数を設定する

[English](../../../en/pop/how-to-guide/theme-and-tokens.md)

この手引きでは、Pop の見た目を導入し、実行中にライトとダークを切り替え、色とフォントを変え、自分のコンポーネントからキットの値を読む方法を説明します。

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

このテーマは、`kivact::ui` の標準コンポーネント（ボタン、入力欄、チェックボックス、タブ、ラジオグループ、スライダー、テーブル、ダイアログ、メニュー、トースト）と、`kivact-ui-pop` のすべてのコンポーネントを描きます。キットのアイコンも組み込むので、`icon("check")` は Pop のアイコンを描きます。OS の設定は読みません。`ThemeMode::Light` と `ThemeMode::Dark` のどちらを使うかは、アプリが選びます。

## 実行中にライトとダークを切り替える

選択をシグナルに持たせ、ルートを `provide_theme` で包みます。テーマを作るクロージャを渡すと、シグナルが変わるたびにすべてのスタイルが解決し直されます。何も作り直されないので、入力中の文字とフォーカスはそのまま残ります。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::style::{color, provide_theme};
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
            text_input(signal(String::new())).label("Search"),
        ))
        .style(
            style()
                .gap(12)
                .padding(24)
                .height(pct(100))
                .background(color::SURFACE)
                .color(color::TEXT),
        )
    })
}

fn main() -> kivact::Result<()> {
    App::new("Theme switch").run(app)
}
```

`provide_theme` はノードを作らず、何も描きません。そのため、サブツリーの最上位ノードが自分でページの色を指定します。`color::SURFACE` と `color::TEXT` は中核の色の変数です。Pop の見た目はこれらから描かれるので、どちらかを受け取るスタイルは、クロージャを書かなくても切り替えに追従します。

## 色を変える

Pop の色は、`kivact_ui_pop` から公開される、キット自身のテーマ変数です。それぞれに、テーマで設定する変数（`BLUE_VAR`）と、スタイルで使う定数（`BLUE`）があります。

- `BLUE`: キットの操作部品のアクセント（主ボタン、チェックされた箱、選ばれたタブとカレンダーの日）。
- `RED`: 危険の色。破壊的な操作とエラーに使います。
- `MINT` と `YELLOW`: 成功と警告の色。
- `SURFACE`: カードとパネルの面。`TEXT` と `TEXT_MUTED` は本文と補助の文字。`LINE` は細い線と枠線。

キットは、テーマを作るときにこれらを設定します。そこで、色を変えるには、テーマでその変数を設定します。コンポーネントは効いているテーマの変数を読むので、変えるとすべてが描き替わります。テーマは、これらから `kivact::style::color` の中核の変数（アクセント、危険、成功、警告、面、文字、枠線）も設定するので、`kivact::ui` のコンポーネントも同じ色に従います。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{Theme, ThemeMode, rgb};
use kivact_ui_pop as pop;

fn brand_theme() -> Theme {
    pop::theme(ThemeMode::Light).set(&pop::BLUE_VAR, rgb(0x0f766e))
}

fn main() -> kivact::Result<()> {
    App::new("Teal")
        .theme(brand_theme())
        .run(|| button("Teal").variant(kivact::ui::ButtonVariant::Primary))
}
```

アプリがライトとダークを切り替えられるなら、それぞれの値を別々に設定します。テーマには、渡されたモードに対して `set` を呼びます。そうすると、モードごとに値を持てます。

## 自分のコンポーネントで変数を読む

スタイルでは、上のように定数を使います。変数から何かを計算する必要があるときだけ、変数そのものを読みます。結果がテーマの切り替えに追従するよう、スタイルのクロージャの中で読みます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{color, radius, use_theme};

#[component]
fn highlight(content: View) -> impl IntoView {
    column(content).style(move || {
        let accent = use_theme().get().get(&color::ACCENT_VAR);
        style()
            .padding(12)
            .border_radius(radius::MD)
            .background(accent.with_alpha(0.15))
            .color(color::TEXT)
    })
}
```

## フォントと文字組み

フォントはアプリのものです。テーマで `font::FAMILY_VAR` にファミリー名を書き、配布するファイルを `Theme::with_fonts` で登録します。キットのカタログは本文に Inter、日本語に Noto Sans JP を使います。Pop の部品のうち 3 つ（`kivact::ui::Slant` のバッジ、アバター、番号付きの `SectionNumber`）は `Barlow` というファミリーを指定し、登録がなければ `sans-serif` に戻ります。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{FontFamily, Theme, ThemeMode, font};
use kivact_ui_pop as pop;

fn themed() -> Theme {
    // `Inter` and `Noto Sans JP` are the families the catalog uses.
    pop::theme(ThemeMode::Light)
        .set(&font::FAMILY_VAR, FontFamily::from("Inter, 'Noto Sans JP', sans-serif"))
}

/// `inter` and `noto` are font files you embed, for example
/// `include_bytes!("../assets/Inter.ttf")`.
fn application(inter: &'static [u8], noto: &'static [u8]) -> App {
    App::new("Pop").theme(themed().with_fonts([inter, noto]))
}
# fn main() {}
```

ファミリー名は、ファイル名ではなく、フォントファイルの中に書かれた名前です。

## キットの文言を差し替える

コンポーネントが自分で描く文言（カレンダーの月のボタン、日付の形式エラー、ドロップゾーンの選択ボタンなど）は英語です。`App::messages` で差し替えます。標準コンポーネントは正式なキーを使います。たとえば `date.open` と `date.error.format` です。例は [日付と期間を選ぶ](dates-and-calendars.md) にあります。

## 関連

- [日付と期間を選ぶ](dates-and-calendars.md)
- [テーブルで行を表示する](tables.md)
- コンポーネントの全一覧: `cargo doc -p kivact-ui-pop --open`

Pop のテーマで、標準のドロップ先を使います。ファイルの選択と複数ファイルを明示的に有効にします。装飾アイコンは、取り付けられるスロットです。

```rust,no_run
use kivact::prelude::*;

#[component]
fn upload() -> impl Element {
    kivact::ui::dropzone("Upload")
        .multiple(true)
        .pick_on_press(true)
        .icon(icon("cloud-upload").style(style().size(24, 24)))
        .hint("PNG, PDF")
        .on_files(|files| {
            for file in files {
                println!("{}", file.path.display());
            }
        })
}
```
