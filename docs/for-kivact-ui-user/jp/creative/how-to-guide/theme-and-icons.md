# Creative キットのテーマ・フォント・アイコンを設定する

[English](../../../en/creative/how-to-guide/theme-and-icons.md)

目的: キットのテーマを適用し、アクセントの色と密度を選び、ライトとダークを実行中に切り替え、自分のフォントを使い、自分のアイコンを追加し、キットが自分で表示する文言を翻訳します。

キットの見た目は、Kivact の普通の `Theme` です。`kivact::ui` の標準部品（ボタン、テキスト入力、メニュー、ダイアログなど）も含めて、すべての見た目はそこから決まります。このガイドは、[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。テーマの仕組みそのものは、[テーマと部品の見た目を変える](../../../../for-kivact-user/jp/how-to-guide/theming.md)で説明しています。

名前付きのツールバーは `kivact::ui::toolbar(label).start(children).center(children).end(children)`、
連結したアクションは `button_group(label, children)`、単一選択や複数選択は
`toggle_group(label, options, selection)` を使います。Creative テーマが密な間隔、
フィールドの背景、押された状態の色を設定します。排他的なセグメントには
`kivact::ui::radio_group(label, options, selection).variant(kivact::ui::RadioGroupVariant::Segmented)`
を使います。キーボード操作と選択は標準部品が持ち、テーマが見た目を設定します。

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

`ThemeOptions` 構造体を `theme_with` に渡します。省略するフィールドには `..Default::default()` を使います。

```rust,no_run
use kivact::style::{Theme, ThemeMode};
use kivact_ui_creative as ui;

fn compact_lime() -> Theme {
    ui::theme_with(ThemeMode::Dark, ui::ThemeOptions {
        accent: ui::Accent::Lime,
        density: ui::Density::Compact,
    })
}
```

`Accent` には `Iris`・`Lime`・`Aqua`・`Coral`・`Amber` があります。アクセントは、選択、フォーカスリング、操作中の部品の色になります。`Density::Compact` は、コントロールの高さを 24 から 20 ピクセルにし、間隔を詰めます。ほかのテーマ設定は、返されたテーマに適用します。

## 実行中にテーマを切り替える

`App::theme` はアプリを通じて固定です。実行中にテーマを変えるには、変えたい UI の部分を `provide_theme` で包み、シグナルを読むクロージャを渡します。シグナルが変わると、クロージャが再評価されます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::style::provide_theme;
use kivact_ui_creative as ui;

fn themed() -> impl IntoView {
    let dark = signal(true);
    let accent = signal(ui::Accent::Iris);

    provide_theme(
        move || {
            let mode = if dark.get() { ThemeMode::Dark } else { ThemeMode::Light };
            ui::theme_with(mode, ui::ThemeOptions { accent: accent.get(), ..Default::default() })
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
                    .background(ui::ColorName::Canvas)
                    .color(ui::ColorName::Text),
            )
        },
    )
}
```

`ui::ColorName` は、キット独自のパレットをスタイルの値にしたものです（`Canvas`・`Island`・`Header`・`Field`・`Hair`・`Text`・`Muted`・`Accent` など）。スタイルが色を受け取る場所ならどこでも使え、テーマを切り替えると自動で追従します。テーマを読むクロージャは要りません。

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

名前は、キットの部品の引数にも現れます。`DockTab::icon("palette")`、`TreeItem::new(.., "box")`、`PickerItem::icon(..)`、`PieEntry`、`ToolWindowOptions::icon(..)` などです。どれも、画像のような `kivact::style::Icon` も受け取るので、登録せずに画像を直接渡すこともできます。求めた名前がないテーマでは、何も描かず、診断を報告します。テーマを切り替えるときは、キットのテーマを保つか、その名前を追加してください。

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

標準の `kivact::ui::badge`、`avatar`、`avatar_group`、`skeleton`、`spinner`、`kbd`、`kbd_combo` は Creative の recipe を使います。tone はキットの色、key cap は等幅フォントを使い、skeleton と spinner の動きは Core が所有します。Creative の `KeyGlyph` は引き続き keyframe marker を表します。画像、fallback、status、size、動きの全 option は [インジケーターとキーボードショートカット](../../../../for-kivact-user/jp/how-to-guide/indicators.md)を参照してください。

インライン通知は、標準の `kivact::ui::alert(title).description(body).tone(tone).variant(ui::NoticeVariant)` を使います。`.icon(icon("warning"))`、`.dismissible(true)`、`.dismiss_content(icon("x"))` を付けると、従来のアイコンと四角い閉じるボタンになります。閉じる状態と focus の復元は Core が所有します。標準の empty state、card、input group も Creative の recipe を使い、[フィードバックの全 option](../../../../for-kivact-user/jp/how-to-guide/feedback.md)を保持します。

表示専用の概要には `kivact::ui::progress(value).variant(ProgressVariant::Ring)` または `Gauge`、`stat(label, value)`、`heading(text, level)`、`description_list((description(term, detail), ...))` を使います。Creative は高さ 8 px のバー表示を保ち、密な指標用の文字スタイルを設定します。未確定の動き、百分率の意味、独自内容の slot は Core が持ちます。編集用の `ui::property_row` は domain 部品として残ります。[指標と説明リスト](../../../../for-kivact-user/jp/how-to-guide/metrics-and-description.md)を参照してください。

編集用の `ui::property_row` は標準の description 表示を組み立てます。行は `Role::ListItem` を公開するため、関連する行を `kivact::ui::description_list` または `Role::List` のレイアウトコンテナに置きます。ラベル幅、行の高さ、前後の操作、編集用の binding は変わりません。

静的な `kivact::ui::table` の各部品と `item_list` / `item_row` は、密な Creative の collection recipe を使います。内容とアクセシビリティの契約は [リストと表](../../../../for-kivact-user/jp/how-to-guide/lists-and-tables.md)を参照してください。domain の outliner は単一の `use_tree` model を標準の `tree_rows` で表示し、フィルター、固定幅の cell、可視性、restriction、drag の要求を保ちます。
