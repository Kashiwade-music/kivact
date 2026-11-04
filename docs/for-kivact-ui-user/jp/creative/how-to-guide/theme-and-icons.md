# Creative キットのテーマ・フォント・アイコンを設定する

[English](../../../en/creative/how-to-guide/theme-and-icons.md)

目的: キットのテーマを適用し、アクセントの色と密度を選び、ライトとダークを実行中に切り替え、自分のフォントを使い、自分のアイコンを追加し、キットが自分で表示する文言を翻訳します。

キットの見た目は、Kivact の普通の `Theme` です。`kivact::ui` の標準部品（ボタン、テキスト入力、メニュー、ダイアログなど）も含めて、すべての見た目はそこから決まります。このガイドは、[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。テーマの仕組みそのものは、[テーマと部品の見た目を変える](../../../../for-kivact-user/jp/how-to-guide/theming.md)で説明しています。

## テーマを適用する

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_creative as ui;

fn root() -> impl IntoView {
    text("クリエイティブツールへようこそ")
}

fn main() -> kivact::Result<()> {
    App::new("Creative アプリ")
        .theme(ui::theme(ThemeMode::Dark))
        .run(root)
}
```

`ui::theme(mode)` は、*Iris* のアクセントとゆったりした間隔を持つ、ダーク（またはライト）のテーマです。キットのレシピも一緒に組み込まれるので、標準部品がキットの部品と見た目がそろいます。キットが描くすべてのアイコンを含むアイコンセットも入っています。フォントファイルは不要で、文字にはシステムの `sans-serif` と `monospace` を使います。

## アクセントと密度を選ぶ

`ThemeOptions` トレイトをインポートして（メソッドだけを使うので `_` を付けます）、メソッドをつなぎます。

```rust,no_run
use kivact::style::{Theme, ThemeMode};
use kivact_ui_creative::{self as ui, ThemeOptions as _};

fn compact_lime() -> Theme {
    ui::theme(ThemeMode::Dark)
        .accent(ui::Accent::Lime)
        .density(ui::Density::Compact)
}
```

`Accent` には `Iris`・`Lime`・`Aqua`・`Coral`・`Amber` があります。アクセントは、選択、フォーカスリング、操作中の部品の色になります。`Density::Compact` は、コントロールの高さを 24 から 20 ピクセルにし、間隔を詰めます。どちらのメソッドも、テーマがすでに持っているほかの設定は変えません。

## 実行中にテーマを切り替える

`App::theme` はアプリを通じて固定です。実行中にテーマを変えるには、変えたい UI の部分を `provide_theme` で包み、シグナルを読むクロージャを渡します。シグナルが変わると、クロージャが再評価されます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::style::provide_theme;
use kivact_ui_creative::{self as ui, ThemeOptions as _};

fn themed() -> impl IntoView {
    let dark = signal(true);
    let accent = signal(ui::Accent::Iris);

    provide_theme(
        move || {
            let mode = if dark.get() { ThemeMode::Dark } else { ThemeMode::Light };
            ui::theme(mode).accent(accent.get())
        },
        move || {
            column((
                kivact::ui::switch("ダークモード", dark),
                row((
                    button("Iris").on_press(move || accent.set(ui::Accent::Iris)),
                    button("Lime").on_press(move || accent.set(ui::Accent::Lime)),
                    button("Coral").on_press(move || accent.set(ui::Accent::Coral)),
                ))
                .style(style().gap(6)),
            ))
            // `provide_theme` 自身はノードを作らないので、最上位のノードに色を付ける。
            .style(
                style()
                    .gap(10)
                    .padding(16)
                    .background(ui::Color::Canvas)
                    .color(ui::Color::Text),
            )
        },
    )
}
```

`ui::Color` は、キット独自のパレットをスタイルの値にしたものです（`Canvas`・`Island`・`Header`・`Field`・`Hair`・`Text`・`Muted`・`Accent` など）。スタイルが色を受け取る場所ならどこでも使え、テーマを切り替えると自動で追従します。テーマを読むクロージャは要りません。

## 自分のフォントを使う

既定のフォントファミリは設定不要です。同梱したフォントを使うには、2 つのことをします。フォントのデータをアプリに登録することと、ファミリ名をテーマに伝えることです。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::style::theme::Typography;
use kivact_ui_creative as ui;

/// `font` はフォントファイルのバイト列。たとえば
/// `include_bytes!("../assets/Geist[wght].ttf")`。
fn app_with_font(font: &'static [u8]) -> App {
    let theme = ui::theme(ThemeMode::Dark);

    // キットのトークン: ファミリ名を変えて、テーマへ戻す。
    let mut tokens = (*theme.ext::<ui::Tokens>()).clone();
    tokens.body_font = "Geist".into();
    tokens.mono_font = "Geist Mono".into();

    // コアのテキストスタイルも、本文のファミリに合わせる。
    let text = Typography {
        family: Some(tokens.body_font.clone()),
        ..theme.text().clone()
    };
    let theme = theme.with_text(text).with(tokens);

    App::new("フォント").font(font).theme(theme)
}
```

`body_font` はラベルに、`mono_font` は数値やコードに使われます。ファミリ名は、フォントファイルの中に記録されている名前にしてください。使うフォントファイルごとに、`.font(..)` を 1 回ずつ呼びます。参考として、キットのカタログは Geist、Geist Mono、Noto Sans JP でまさにこれを行っています。

## アイコンを描く・追加する

アイコンは、テーマのアイコンセットにある名前です。キットのテーマには、`play`・`pause`・`eye`・`eye-off`・`lock`・`folder`・`stack`・`layout-columns`・`adjustments`・`keyframes`・`palette`・`color-picker`・`chevron-down`・`plus`・`trash`・`search`・`x` など、16 ピクセルの Tabler のアウトラインアイコンが入っています。`icon(name)` で描くと、周囲の文字色になります。

```rust,no_run
use kivact::view::Asset;
use kivact::view::icon;
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_creative as ui;

static NODE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3l9 9-9 9-9-9z"/></svg>"#;

fn main() -> kivact::Result<()> {
    // 自分の画像に、好きな名前を付けて登録する。
    let theme = ui::theme(ThemeMode::Dark).with_icon("node", Asset::from(NODE));
    App::new("アイコン").theme(theme).run(|| {
        row((icon("play"), icon("node"), text("ノードを再生")))
            .style(style().gap(8).padding(12))
    })
}
```

名前は、キットの部品の引数にも現れます。`DockTab::icon("palette")`、`TreeItem::new(.., "box")`、`PickerItem::icon(..)`、`PieEntry`、`WindowOptions::icon(..)` などです。どれも、画像のような `kivact::style::Icon` も受け取るので、登録せずに画像を直接渡すこともできます。求めた名前がないテーマでは、何も描かず、診断を報告します。テーマを切り替えるときは、キットのテーマを保つか、その名前を追加してください。

`ui::KeyGlyph`（`Diamond`・`Square`・`Circle`・`HalfDiamond`）は、キーフレームの印の形です。`Keyframe::glyph(..)` でキーごとに設定します。`ui::LabelTone`（`Normal`・`Muted`・`Accent`・`Warning`・`Error`・`Custom(color)`）は、テーマに追従する文字色の名前で、`TreeItem::tone(..)` が使います。

## キットが表示する文言を翻訳する

キットが自分で描く少数の文言（ドックのメニューの命令、トランスポートのボタン名、ダイアログの閉じるボタン、アウトライナーのアクセシビリティ名）は、既定では英語です。`App::messages` で、キーを自分の文言に置き換えます。指定しなかったキーは、英語の既定の文言のままです。

```rust,no_run
use kivact::i18n::Messages;
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn translated(app: App) -> App {
    app.messages(
        Messages::new()
            .with("creative.dock.close_panel", "パネルを閉じる")
            .with("creative.dock.maximize", "パネルを最大化")
            .with("creative.transport.play", "再生")
            .with("creative.outliner.label", "階層"),
    )
}
```

キーはすべて `creative.` で始まります。各部品の rustdoc に、読み取るキーが載っています（`cargo doc -p kivact-ui-creative --open`）。実行中に言語を切り替えるには、[国際化（翻訳とロケールの設定）](../../../../for-kivact-user/jp/how-to-guide/i18n.md)を参照してください。

## 関連項目

- [テーマと部品の見た目を変える](../../../../for-kivact-user/jp/how-to-guide/theming.md): コアのテーマの仕組み。
- [ドック・パネルグループ・アウトライナーを作る](panels-docks-and-outliners.md): アイコンと文言が現れる場所。
- 組み込みのアイコン名の一覧は、リポジトリの `crates/kivact-ui-creative/README.md` にあります。
- API の全体: `cargo doc -p kivact-ui-creative --open`
- キットの見た目ですべての部品を確認する: `cargo run -p kivact-ui-creative-catalog`
