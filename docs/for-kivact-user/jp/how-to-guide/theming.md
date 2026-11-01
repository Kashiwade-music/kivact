# テーマとコンポーネントの見た目を変える

[English](../../en/how-to-guide/theming.md)

`Theme` は、標準コンポーネントがすべて参照する色、余白、角丸、文字サイズ、アイコン、コンポーネントごとの recipe をまとめたものです。アプリのテーマは `App::theme` で選び、実行中に切り替えるには `provide_theme` を使い、自分のスタイルからトークンを読み、一部分だけを変えるには recipe を使います。公式キットは完成したテーマを提供しているので、ゼロから作ることはほとんどありません。

## テーマを選ぶ

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;

fn main() -> kivact::Result<()> {
    // `kivact-ui-basic` は Cargo.toml にある。`Theme::dark()` ならキットは不要。
    let theme = kivact_ui_basic::theme(ThemeMode::Dark);

    App::new("Dark app")
        .theme(theme)
        .run(|| button("こんにちは").style(style().margin(16)))
}
```

- `Theme::light()` が既定で、`Theme::dark()` がその暗いテーマです。`theme.mode()` は `ThemeMode`(`Light` か `Dark`)を返します。
- どのキットにも `theme(mode: ThemeMode)` があります。`kivact_ui_basic::theme`、`kivact_ui_pop::theme`、`kivact_ui_game::theme`、`kivact_ui_creative::theme` です。結果を `App::theme` に渡します。それぞれの見た目と、付属の追加コンポーネントは、各キットのガイドで説明しています([キット一覧](../../../for-kivact-ui-user/jp/README.md)を参照)。
- システム設定とユーザー設定のどちらでモードを決めるかは、アプリが決めることです。Kivact は代わりに選びません。

## アプリの実行中にテーマを切り替える

`App::theme` は起動時のテーマを固定します。後から変えるには、テーマをシグナルに入れ、画面を `provide_theme` で包みます。

```rust,test_harness
use kivact::prelude::*;
use kivact::style::{ThemeMode, current_theme};
use kivact::view::provide_theme;

fn app() -> impl IntoView {
    let theme = signal(Theme::light());

    provide_theme(theme, move || {
        column((
            button("ダークモードを切り替え").on_press(move || {
                theme.update(|t| {
                    *t = match t.mode() {
                        ThemeMode::Light => Theme::dark(),
                        ThemeMode::Dark => Theme::light(),
                    }
                })
            }),
            text(move || format!("モード: {:?}", current_theme().mode())),
        ))
        // `provide_theme` は何も描かないので、最上位の node にテーマの色を与える。
        .style(
            style()
                .height(pct(100))
                .padding(16)
                .gap(8)
                .background(ColorToken::Surface)
                .color(ColorToken::Text),
        )
    })
}

fn main() -> kivact::Result<()> {
    App::new("Theme switch").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_toggle_switches_the_mode_without_remounting() {
        let screen = kivact::test::render(app);
        screen.get_by_text("モード: Light");
        screen.get_by_role(Role::Button, "ダークモードを切り替え").click();
        screen.get_by_text("モード: Dark");
    }
}
```

`provide_theme` の下にあるスタイルは、シグナルが変わると色を解決し直します。作り直されるものも、失われる状態もありません。テーマにはクロージャや `memo` も渡せます。`provide_theme` は部分木にも使えます。明るいアプリの中で、設定パネルだけを暗くできます。兄弟の部分木は、上から引き継いだテーマのままです。

## 自分のスタイルでトークンを読む

`ColorToken` で指定した色は、描画されるテーマに対して解決されるので、切り替えに自動で追従します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::current_theme;

fn card(body: impl IntoView) -> impl IntoView {
    column(body).style(
        style()
            .background(ColorToken::SurfaceAlt)
            .border(1, ColorToken::Border)
            .radius(8)
            .padding(12),
    )
}

fn spaced(body: impl IntoView) -> impl IntoView {
    // テーマはクロージャの中で読む。テーマが変わるとスタイルが再計算される。
    column(body).style(move || {
        let theme = current_theme();
        style().padding(theme.space().lg).gap(theme.space().sm)
    })
}

fn main() -> kivact::Result<()> {
    App::new("Tokens").run(|| card(spaced(text("こんにちは"))))
}
```

- 色のトークンは `Surface`、`SurfaceAlt`、`Text`、`TextMuted`、`Accent`、`AccentText`、`Border`、`Focus`、`Danger`、`Success`、`Warning`、`Overlay`、`Shadow`、`Selection`、`Hover`、`Pressed`、`Raised` です。固定の色には `rgb(0x2563eb)` や `rgba(..)` を使います。
- そのほかのトークンはアクセサーで読みます。`theme.space().md`、`theme.radius().lg`、`theme.text().size`、`theme.focus_ring()`、`theme.disabled_opacity()`、`theme.control_height()` です。
- `current_theme()` は、セットアップで 1 回読むのではなく、スタイルのクロージャの**中**で読みます。セットアップで取ったコピーは固定のスナップショットなので、テーマの切り替えに追従できません。

## トークンを変える

`with_*` のビルダーはどれも新しいテーマを返し、残りはそのまま保ちます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::rgb;
use kivact::style::theme::Palette;

fn brand() -> Theme {
    let base = Theme::light().with_focus_ring(3.0);
    let colors = Palette {
        accent: rgb(0x7c3aed),
        focus: rgb(0x7c3aed),
        ..base.colors().clone()
    };
    base.with_colors(colors).with_disabled_opacity(0.4)
}

fn main() -> kivact::Result<()> {
    App::new("Brand").theme(brand()).run(|| button("紫"))
}
```

標準コンポーネントはすべて新しいアクセントを使います。`with_space`、`with_radius`、`with_text`(文字サイズと既定のフォントファミリー)、`with_control_height`、`with_icons` も同じように使えます。`Theme::light()` ではなくキットのテーマから始めると、キットの recipe とトークンが保たれます。

## 標準コンポーネントの見た目を 1 つ変える

トークンは全体を一度に変えます。1 つのコンポーネントの 1 つの部位だけを変えるには、その **recipe** を包みます。recipe は、テーマと部位の状態から `Style` を返す関数です。recipe の表が `UiRecipes` で、`Theme::with` で載せます。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::UiRecipes;

fn square_buttons() -> Theme {
    let theme = Theme::light();
    // テーマが今持っている recipe(キットのテーマならキットのもの)を読み……
    let recipes = UiRecipes::clone(&theme.get::<UiRecipes>());
    // ……button の recipe を包む。古い recipe を呼び、1 つだけ上書きする。
    let recipes = recipes.button(|previous| move |theme, state| previous(theme, state).radius(0));
    theme.with(recipes)
}

fn main() -> kivact::Result<()> {
    App::new("Square").theme(square_buttons()).run(|| button("保存"))
}
```

- 部位の名前のメソッド(`button`、`text_input`、`checkbox`、...)がその recipe を置き換えます。現在の recipe を受け取るので、その上に積めます。ダイアログのように複数の部位をまとめた表は、`dialog_recipes()` で読み、`dialog(|table| table.surface(..))` で変えます。
- テーマは**1 回だけ**(起動時、または `static` の中で)作ります。recipe の表は同一性で比較されるため、実行のたびに作り直すテーマは、その下のすべてを毎回スタイルし直します。
- 振る舞い(キー、フォーカス、アクセシビリティ)は recipe に含まれないので、見た目を変えても動作は変わりません。
- テーマではなく 1 つのインスタンスだけ変えるなら、そこに `.style(..)` を付けます。コンポーネントに指定したスタイルは recipe の後に重なり、優先されます。

### variant: 同じコンポーネントの別の見た目

`.variant(value)` は、テーマを変えずに recipe へ値を渡します。`ButtonVariant` が組み込みの variant です。

```rust,no_run
use kivact::prelude::*;

fn buttons() -> impl IntoView {
    row((
        button("Primary"),
        button("Secondary").variant(ButtonVariant::Secondary),
        button("Outline").variant(ButtonVariant::Outline),
        button("Ghost").variant(ButtonVariant::Ghost),
        button("Danger").variant(ButtonVariant::Danger),
        button("Link").variant(ButtonVariant::Link),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Variants").run(buttons)
}
```

variant のないボタンは `Primary` です。各 variant の見た目はテーマが決めます。キットは独自の variant の型を追加することもよくあります。自分で作るには、`kivact::ui::Variant` を実装した型を定義し、recipe の中で `look.variant::<MyVariant>()` で読みます。`button(..).size(kivact::ui::ControlSize::Small)` などの `size` オプションも、同じようにメトリクスを変えます。

## hover、pressed、focus、disabled の見た目

状態ごとの見た目は、イベントを待ち受けるのではなく、返すスタイルの一部として書きます。

```rust,no_run
use kivact::prelude::*;

fn tile() -> impl IntoView {
    node()
        .child(text("開く"))
        .role(Role::Button)
        .focusable(true)
        .style(
            style()
                .padding(12)
                .background(ColorToken::SurfaceAlt)
                .hover(style().background(ColorToken::Hover))
                .pressed(style().background(ColorToken::Pressed))
                .focus_visible(style().ring(2, ColorToken::Focus))
                .disabled(style().opacity(0.5)),
        )
}

fn row_with_actions() -> impl IntoView {
    // 親に `group()` を付けると、子が親の hover に反応できる。
    row((
        text("レポート.pdf"),
        spacer(),
        button("削除").style(style().opacity(0.0).group_hover(style().opacity(1.0))),
    ))
    .style(style().group().padding(8))
}

fn main() -> kivact::Result<()> {
    App::new("States").run(|| column((tile(), row_with_actions())))
}
```

- `hover` はポインターが node かその子孫の上にある間、`pressed` は押している間、`focus` はフォーカスがある間、`focus_visible` はフォーカスを見せるべき間(キーボード操作時)、`disabled` は node が無効な間に適用されます。
- フォーカスできる要素に `focus` も `focus_visible` もなければ、テーマのフォーカスリングが自動で付きます。無効な要素はテーマの `disabled_opacity` で描かれます。消すには `style().focus_visible(style().ring_none())` や `style().disabled(style().opacity(1.0))` を使います。
- `group()` と `group_hover(..)`(`group_focus(..)` も)は、祖先の状態に反応します。ポインターのハンドラーを書く必要はありません。
- recipe の中で hover 状態を読んで分岐することはしません。状態ごとにスタイルを書けば、エンジンが適用します。

## アイコン

アイコンは、現在のテーマが画像に解決する名前です。そのため、キットは同じ名前に自分のグリフを割り当てられます。標準の印(`check`、`dot`、`chevron-down`、`chevron-right`、`x`)はどのテーマにもあります。自分のアイコンは `Theme::with_icon` で加え、`icon(..)` で描きます。

```rust,test_harness
use kivact::prelude::*;

static STAR: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 2l3 7h7l-6 4 2 7-6-4-6 4 2-7-6-4h7z"/></svg>"#;

fn app() -> impl IntoView {
    row((
        icon("star").name("お気に入り").style(style().size(24, 24).color(ColorToken::Warning)),
        text("スター付き"),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    let theme = Theme::light().with_icon("star", STAR);
    App::new("Icons").theme(theme).run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_theme_icon_draws_without_errors() {
        let options = kivact::test::RenderOptions {
            theme: Theme::light().with_icon("star", STAR),
            ..kivact::test::RenderOptions::from_env()
        };
        let screen = kivact::test::render_with(options, app);
        screen.get_by_role(Role::Image, "お気に入り");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
```

- `icon(name)` は、テキストの色で 16 px の画像を描きます。大きさと色はスタイルで決め、`.name(..)` を付けると支援技術に公開されます(名前がなければ装飾として扱われます)。
- テーマが知らない名前は何も描かれません。デバッグビルドでは 1 回だけ報告されます。
- アイコンを受け取るコンポーネント(`MenuEntry::icon`、`ListItem::icon`)も、同じ名前か、画像の `Asset` を受け取ります。ファイルのままのアイコンは[画像とフォント](images-and-fonts.md)を参照してください。

## 自分のトークンを加える

自分の色やサイズをテーマ経由で共有するには、`ThemeExtension` を実装した型を定義します。

```rust,test_harness
use kivact::prelude::*;
use kivact::style::{Color, ThemeExtension, current_theme, rgb};

#[derive(Debug, PartialEq)]
struct Brand {
    highlight: Color,
    gutter: f32,
}

impl ThemeExtension for Brand {
    // `Brand` を持たないテーマが読む値。たとえば、自分のコンポーネントが
    // 別のテーマの下で動くときに使われる。
    fn fallback(_theme: &Theme) -> Self {
        Brand { highlight: rgb(0xff8800), gutter: 12.0 }
    }
}

fn banner() -> impl IntoView {
    text("新着").style(move || {
        let brand = current_theme().get::<Brand>();
        style().color(brand.highlight).padding(brand.gutter)
    })
}

fn main() -> kivact::Result<()> {
    let theme = Theme::light().with(Brand { highlight: rgb(0x0088ff), gutter: 16.0 });
    App::new("Brand tokens").theme(theme).run(banner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_installed_value_wins_and_a_missing_one_falls_back() {
        let themed = Theme::light().with(Brand { highlight: rgb(0x0088ff), gutter: 16.0 });
        assert_eq!(themed.get::<Brand>().gutter, 16.0);
        assert_eq!(Theme::dark().get::<Brand>().gutter, 12.0);
    }
}
```

- 型には `Debug + PartialEq + Send + Sync + 'static` が必要です。2 つのテーマは、載せた値が等しければ等しいとみなされるため、変わっていないテーマでスタイルし直されることはありません。
- `Theme::get::<Brand>()` は、載せた値を共有する `Arc<Brand>` を返します。コピーはされません。
- トークンは(上のように)スタイルのクロージャの中で読みます。そうすれば、テーマの切り替えで、コンポーネントを作り直さずにスタイルし直されます。
- コンポーネントを配布するライブラリーは、自分の recipe の表にも同じ仕組みを使います。[再利用できるコンポーネントを書く](custom-components.md)を参照してください。

## 関連項目

- [キット](../../../for-kivact-ui-user/jp/README.md): すぐ使えるテーマとコンポーネント。
- [レイアウト](layout.md): サイズ、余白、配置。
- [画像とフォント](images-and-fonts.md): カスタムフォントと SVG のアイコン。
- [翻訳とロケール](i18n.md): ロケール、文言、文字の向き。
- リファレンス: `Theme`、`Style`、`ColorToken` は `cargo doc -p kivact-style --open`、`UiRecipes` は `cargo doc -p kivact-ui --open`。
