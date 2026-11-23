# テーマとコンポーネントの見た目を変える

[English](../../en/how-to-guide/theming.md)

`Theme` は、標準コンポーネントがすべて参照する色、余白、角丸、文字サイズ、アイコン、コンポーネントごとの recipe をまとめたものです。アプリのテーマは `App::theme` で選び、実行中に切り替えるには `provide_theme` を使い、自分のスタイルではテーマ変数を読み、一部分だけを変えるには recipe を使います。公式キットは完成したテーマを提供しているので、ゼロから作ることはほとんどありません。

## テーマを選ぶ

```rust,no_run
use kivact::prelude::*;

fn main() -> kivact::Result<()> {
    // `kivact-ui-basic` is in Cargo.toml; `Theme::dark()` needs no kit.
    let theme = kivact_ui_basic::theme(ThemeMode::Dark);

    App::new("Dark app")
        .theme(theme)
        .run(|| button("Hello").style(style().margin(16)))
}
```

- `Theme::light()` が既定で、`Theme::dark()` がその暗いテーマです。`theme.mode()` は `ThemeMode`(`Light` か `Dark`)を返します。
- どのキットにも `theme(mode: ThemeMode)` があります。`kivact_ui_basic::theme`、`kivact_ui_pop::theme`、`kivact_ui_game::theme`、`kivact_ui_creative::theme`、`kivact_ui_soft::theme` です。結果を `App::theme` に渡します。それぞれの見た目と、付属の追加コンポーネントは、各キットのガイドで説明しています([キット一覧](../../../for-kivact-ui-user/jp/README.md)を参照)。
- システム設定とユーザー設定のどちらでモードを決めるかは、アプリが決めることです。Kivact は代わりに選びません。

## アプリの実行中にテーマを切り替える

`App::theme` は起動時のテーマを固定します。後から変えるには、テーマをシグナルに入れ、画面を `provide_theme` で包みます。

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    let theme = signal(Theme::light());

    provide_theme(theme, move || {
        column((
            button("Toggle dark mode").on_press(move || {
                theme.update(|t| {
                    *t = match t.mode() {
                        ThemeMode::Light => Theme::dark(),
                        ThemeMode::Dark => Theme::light(),
                    }
                })
            }),
            text(move || format!("mode: {:?}", use_theme().get().mode())),
        ))
        // `provide_theme` draws nothing, so give its top node the theme's colors.
        .style(
            style()
                .height(pct(100))
                .padding(16)
                .gap(8)
                .background(color::SURFACE)
                .color(color::TEXT),
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
        screen.get_by_text("mode: Light");
        screen.get_by_role(Role::Button, "Toggle dark mode").click();
        screen.get_by_text("mode: Dark");
    }
}
```

`provide_theme` の下にあるスタイルは、シグナルが変わると色を解決し直します。作り直されるものも、失われる状態もありません。テーマにはクロージャや `memo` も渡せます。`provide_theme` は部分木にも使えます。明るいアプリの中で、設定パネルだけを暗くできます。兄弟の部分木は、上から引き継いだテーマのままです。

## スタイルでテーマ変数を使う

テーマ変数は、描画されるテーマに対して解決されるので、切り替えに自動で追従します。

```rust,no_run
use kivact::prelude::*;

#[component]
fn card(body: View) -> impl Element {
    column(body).style(
        style()
            .background(color::SURFACE_ALT)
            .border(1, color::BORDER)
            .border_radius(radius::MD)
            .padding(space::MD),
    )
}

#[component]
fn spaced(body: View) -> impl Element {
    // A theme variable needs no closure: the style follows a theme switch.
    column(body).style(style().padding(space::LG).gap(space::SM).width(pct(100) - space::XL))
}

fn main() -> kivact::Result<()> {
    App::new("Theme variables").run(|| card(spaced(text("Hello"))))
}
```

- 色のテーマ変数は `color` にあります。`SURFACE`、`SURFACE_ALT`、`TEXT`、`TEXT_MUTED`、`ACCENT`、`ACCENT_TEXT`、`BORDER`、`FOCUS`、`DANGER`、`SUCCESS`、`WARNING`、`OVERLAY`、`SHADOW`、`SELECTION`、`HOVER`、`PRESSED`、`RAISED` です。`space::XS` から `space::XL`、`radius::SM` から `radius::LG`、`font::SIZE`、`font::SIZE_SMALL`、`font::SIZE_LARGE` は長さです。固定の色には `rgb(0x2563eb)` や `rgba(..)` を使います。長さは CSS の `calc()` のように `+` と `-` で組み合わせられます。`pct(100) - space::XL` は割合とテーマ変数を混ぜたもので、テーマが変わると計算し直されます。
- そのほかの値は、アクセサーまたは変数で読みます。`theme.focus_ring()` と `theme.focus_ring_offset()` はアクセサーで読みます。コントロールの高さ、無効時の不透明度、オーバーレイの余白は、変数 `control::HEIGHT_VAR`、`control::DISABLED_OPACITY_VAR`、`control::OVERLAY_MARGIN_VAR`(`kivact::style::control` にあります)で、`theme.get(..)` で読みます。`theme.get(&color::ACCENT_VAR)` は変数の値を読みます。たとえばキャンバスに描くときに使います。
- `use_theme()` は有効なテーマを返します。スタイルが変数以上のものに依存するときは、スタイルのクロージャの中で読みます。

## テーマ変数を設定する

`set(&VAR, value)` は、1 つの変数だけを変えた新しいテーマを返し、残りはそのまま保ちます。値には別の変数も指定できます。ここでは `FOCUS` が `ACCENT` に従います。

```rust,no_run
use kivact::prelude::*;
use kivact::style::control;

fn brand() -> Theme {
    Theme::light()
        .with_focus_ring(3.0)
        .with_focus_ring_offset(2.0)
        .set(&color::ACCENT_VAR, rgb(0x7c3aed))
        .set(&color::FOCUS_VAR, &color::ACCENT_VAR)
        .set(&control::DISABLED_OPACITY_VAR, 0.4)
}

fn main() -> kivact::Result<()> {
    App::new("Brand").theme(brand()).run(|| button("Purple"))
}
```

テーマ変数はどれも、値として別の変数を指定できます。サイズも同じです。たとえばボタンの高さのようなコンポーネント自身の変数は、既定で中核の変数を参照するので、中核の値を変えると、それに従うコンポーネントすべてに反映されます。変数のループは 1 回だけ報告され、既定の値に戻ります。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeVar;

/// The gap of a toolbar: the theme's medium spacing until a theme sets it.
pub static TOOLBAR_GAP_VAR: ThemeVar<f64> = ThemeVar::alias("toolbar.gap", &space::MD_VAR);

fn roomy() -> Theme {
    Theme::light().set(&TOOLBAR_GAP_VAR, &space::LG_VAR)
}
# fn main() { let _ = roomy(); }
```

別名は 1 通りの書き方です。既定の値として `ThemeVar::alias(name, &OTHER_VAR)` を使い、テーマの中では `theme.set(&VAR, &OTHER_VAR)` を使います。

標準コンポーネントはすべて新しいアクセントを使います。`space`、`radius`、`font` の変数(`space::MD_VAR` など)も同じように使えます。`with_icons` はアイコンを設定します。コントロールの高さ、無効時の不透明度、オーバーレイの余白は、`control::HEIGHT_VAR`、`control::DISABLED_OPACITY_VAR`、`control::OVERLAY_MARGIN_VAR` で、同じように設定します。`Theme::light()` ではなくキットのテーマから始めると、キットの recipe とトークンが保たれます。

オーバーレイをウィンドウ端から 16 px 内側に配置するには、`theme.set(&control::OVERLAY_MARGIN_VAR, 16.0)` を使います。既定は余白 0 で、ウィンドウ内への fitting は常に有効です。アンカー付きのオーバーレイは、反転、整列範囲の拡張、スライドの順に試してから、範囲内に収めます。個別の余白は `Overlay::viewport_margin` で指定できます。

## 標準コンポーネントの見た目を 1 つ変える

テーマ変数は全体を一度に変えます。1 つのコンポーネントの 1 つの部位だけを変えるには、その **recipe** を包みます。recipe は、テーマと部位の状態、それまでのスタイルから `Style` を返す関数です。標準コンポーネントにはそれぞれ `kivact::ui::theme` に表があり(たとえば `ButtonTheme`)、`Theme::with` で載せます。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::theme::ButtonTheme;

fn square_buttons() -> Theme {
    // `base` is the style the theme's recipe made; return it changed.
    Theme::light().with(ButtonTheme::default().root(|_theme, _look, base| base.border_radius(0)))
}

fn main() -> kivact::Result<()> {
    App::new("Square").theme(square_buttons()).run(|| button("Save"))
}
```

- 部位の名前のメソッド(`root` と、表のほかの部位)は、その部位についてテーマが定めた recipe の後ろに、自分の関数を加えます。`base` はテーマが作ったスタイルで、`look` は部位の状態と variant を持ちます。
- テーマは**1 回だけ**(起動時に)作り、保持してください。`App::theme` に渡すか、`provide_theme` のためにシグナルに入れます。recipe の表は同一性で比較されるため、実行のたびに作り直すテーマは、その下のすべてを毎回スタイルし直します。
- テーマではなく 1 つのインスタンスだけ変えるなら、そこに `.style(..)` を付けます。コンポーネントに指定したスタイルは recipe の後に重なり、優先されます。

テーマでは、`CheckboxTheme` の `indicator` の内側にある `check` と `mixed` の部位で、チェックボックスの印を描けます。スイッチは `SwitchTheme` の `track` の内側にある `thumb` 部位で、つまみを描けます。これらを個別に描く場合は、ルートの recipe の描画も差し替えます。ラジオグループでは、`RadioGroupTheme` の `mark` 部位がドットアイコンの見た目を決めます。選択状態が変わっても追加した部品はマウントされたままなので、paint transition で透明度、移動、拡大縮小を補間できます。

### variant: 同じコンポーネントの別の見た目

`.variant(value)` は、テーマを変えずに recipe へ値を渡します。`ButtonVariant` が組み込みの variant です。

```rust,no_run
use kivact::prelude::*;

#[component]
fn buttons() -> impl Element {
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

variant のないボタンは `Primary` です。各 variant の見た目はテーマが決めます。キットは独自の variant の型を追加することもよくあります。自分で作るには、`kivact::ui::theme::Variant` を実装した型を定義し、recipe の中で `look.variant::<MyVariant>()` で読みます。`button(..).size(kivact::ui::ControlSize::Small)` などの `size` オプションも、同じようにメトリクスを変えます。

## hover、active、focus、disabled の見た目

状態ごとの見た目は、イベントを待ち受けるのではなく、返すスタイルの一部として書きます。

```rust,no_run
use kivact::prelude::*;

#[component]
fn tile() -> impl Element {
    node()
        .child(text("Open"))
        .role(Role::Button)
        .focusable(true)
        .style(
            style()
                .padding(12)
                .background(color::SURFACE_ALT)
                .hover(style().background(color::HOVER))
                .active(style().background(color::PRESSED))
                .focus_visible(style().outline(2, color::FOCUS))
                .disabled(style().opacity(0.5)),
        )
}

#[component]
fn row_with_actions() -> impl Element {
    // `group()` on the parent lets a child react to the parent's hover.
    row((
        text("Report.pdf"),
        spacer(),
        button("Delete").style(style().opacity(0.0).group_hover(style().opacity(1.0))),
    ))
    .style(style().group().padding(8))
}

fn main() -> kivact::Result<()> {
    App::new("States").run(|| column((tile(), row_with_actions())))
}
```

- `hover` はポインターが node かその子孫の上にある間、`active` は押している間、`focus` はフォーカスがある間、`focus_visible` はフォーカスを見せるべき間(キーボード操作時)、`disabled` は node が無効な間に適用されます。
- フォーカスできる要素に `focus` も `focus_visible` もなければ、テーマのフォーカスリングが自動で付きます。無効な要素はテーマの `disabled_opacity` で描かれます。消すには `style().focus_visible(style().outline_none())` や `style().disabled(style().opacity(1.0))` を使います。
- `group()` と `group_hover(..)`(`group_focus(..)` も)は、祖先の状態に反応します。ポインターのハンドラーを書く必要はありません。
- 状態スタイルでは、どのプロパティも指定できます。パディングや幅のようなレイアウトのプロパティも含みます。状態が変わると、node はレイアウトし直されます。
- recipe の中で hover 状態を読んで分岐することはしません。状態ごとにスタイルを書けば、エンジンが適用します。

## アイコン

アイコンは、現在のテーマが画像に解決する名前です。そのため、キットは同じ名前に自分のグリフを割り当てられます。標準の印(`check`、`dot`、`chevron-down`、`chevron-right`、`x`)はどのテーマにもあります。自分のアイコンは `Theme::with_icon` で加え、`icon(..)` で描きます。

```rust,test_harness
use kivact::prelude::*;

static STAR: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 2l3 7h7l-6 4 2 7-6-4-6 4 2-7-6-4h7z"/></svg>"#;

fn app() -> impl IntoView {
    row((
        icon("star").name("Favorite").style(style().size(24, 24).color(color::WARNING)),
        text("Starred"),
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
        let options = kivact::test::RenderOptions::from_env().theme(Theme::light().with_icon("star", STAR));
        let screen = kivact::test::render_with(options, app);
        screen.get_by_role(Role::Image, "Favorite");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
```

- `icon(name)` は、テキストの色で 16 px の画像を描きます。大きさと色はスタイルで決め、`.name(..)` を付けると支援技術に公開されます(名前がなければ装飾として扱われます)。
- テーマが知らない名前は何も描かれません。デバッグビルドでは 1 回だけ報告されます。
- アイコンを受け取るコンポーネント(`MenuEntry::icon`、`ListItem::icon`)も、同じ名前か、画像の `Asset` を受け取ります。ファイルのままのアイコンは[画像とフォント](images-and-fonts.md)を参照してください。

## 別の色空間で色を選ぶ

色は 16 進数や `rgb` だけでなく、CSS の色空間でも書けます。`oklch`、`oklab`、`lab`、`lch`、`hwb`、`hsl`、`rgba` の関数は `kivact::style` にあり、それぞれがその空間の数値を取ります。明るさの揃ったパレットを作るなら `oklch(l, c, h)` が適しています。`l` は 0 から 1、`c`(彩度)は 0 から、`h` は度数の色相です。`lab` と `lch` では `l` が 0 から 100 です。`hwb` は色相、白み、黒みの順で、白みと黒みは 0 から 1 です。

`Color::css(text)` は、CSS の書き方の色を解釈します。例は `oklch(70% 0.15 250)`、`hsl(210 60% 40%)`、`#2563eb` です。解釈できない文字列はエラーになるので、設定やファイルから来る値は、セットアップで一度確かめてください。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{hwb, oklch};

#[component]
fn swatches(brand_text: String) -> impl Element {
    // A color from a setting: a value that does not parse falls back to the default.
    let brand = Color::css(&brand_text).unwrap_or(oklch(0.62, 0.19, 255.0));
    row((
        text("Brand").style(
            style()
                .padding(8)
                .background(brand)
                .color(oklch(0.98, 0.0, 0.0)),
        ),
        text("Ready").style(
            style()
                .padding(8)
                .background(hwb(150.0, 0.12, 0.25))
                .color(oklch(0.98, 0.0, 0.0)),
        ),
    ))
    .style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Swatches").run(|| swatches("oklch(70% 0.15 250)".to_string())) }
```

これらの空間の色は、色を受け付ける場所ならどこでも使えます。スタイル、枠線、影、グラデーションの色止め、キャンバスの描画などです。画面が表示できない色は、描くときに画面の範囲へ写されます。

## グラデーションの補間と hint を決める

グラデーションは、既定では色止めを Oklab の中で混ぜます。`.interpolate(space)` は別の空間で混ぜます。CSS の `linear-gradient(in oklch, ..)` と同じです。極座標の空間(`Oklch`、`Lch`、`Hsl`、`Hwb`)では、`.hue(..)` で色相環の回り方を選びます。`HueInterpolation::Shorter`(既定)、`Longer`、`Increasing`、`Decreasing` があります。

`.hint(at)` は、色止めとその次の色止めのあいだの中点を動かします。色止めの直後に呼びます。2 つの色のちょうど中間になる位置が、中央ではなく `at` になります。CSS の色のヒントと同じです。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{ColorSpace, HueInterpolation, linear_gradient, oklch};

#[component]
fn dusk() -> impl Element {
    // Sweep through the hues in oklch, the long way round, with the
    // midpoint of the first pair moved towards the warm end.
    let sky = linear_gradient(180)
        .interpolate(ColorSpace::Oklch)
        .hue(HueInterpolation::Longer)
        .stop(oklch(0.45, 0.16, 280.0), pct(0))
        .hint(pct(70))
        .stop(oklch(0.78, 0.17, 45.0), pct(100));

    text("Dusk")
        .style(style().padding(24).height(120).background(sky).color(oklch(0.98, 0.0, 0.0)))
}
# fn main() -> kivact::Result<()> { App::new("Dusk").run(dusk) }
```

## 自分のテーマ変数を加える

自分の色やサイズをテーマ経由で共有するには、自分のテーマ変数を定義します。

```rust,test_harness
use kivact::prelude::*;
use kivact::style::{Length, ThemeVar};

static HIGHLIGHT_VAR: ThemeVar<Color> = ThemeVar::new("brand.highlight", rgb(0xff8800));
static GUTTER_VAR: ThemeVar<f64> = ThemeVar::new("brand.gutter", 12.0);
const HIGHLIGHT: Color = Color::var(&HIGHLIGHT_VAR);
const GUTTER: Length = Length::var(&GUTTER_VAR);

#[component]
fn banner() -> impl Element {
    text("New").style(style().color(HIGHLIGHT).padding(GUTTER))
}

fn main() -> kivact::Result<()> {
    let theme = Theme::light().set(&HIGHLIGHT_VAR, rgb(0x0088ff)).set(&GUTTER_VAR, 16.0);
    App::new("Brand variables").theme(theme).run(banner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_set_value_wins_and_an_unset_one_falls_back() {
        let themed = Theme::light().set(&GUTTER_VAR, 16.0);
        assert_eq!(themed.get(&GUTTER_VAR), 16.0);
        assert_eq!(Theme::dark().get(&GUTTER_VAR), 12.0);
    }
}
```

- 変数は、`static` の `ThemeVar`(名前と既定の値)と、それを指す `const` の `Color` や `Length` です。値を書く場所ならどこでも `const` を使えます。
- `theme.set(&VAR, value)` は 1 つのテーマに値を与えます。値を設定しないテーマは既定の値を使います。既定の値は別の変数でもかまいません。
- recipe のような関数の表は、引き続き `ThemeExtension` にし、`Theme::with` で載せます。[再利用できるコンポーネントを書く](custom-components.md)を参照してください。

## 関連項目

- [キット](../../../for-kivact-ui-user/jp/README.md): すぐ使えるテーマとコンポーネント。
- [レイアウト](layout.md): サイズ、余白、配置。
- [画像とフォント](images-and-fonts.md): カスタムフォントと SVG のアイコン。
- [翻訳とロケール](i18n.md): ロケール、文言、文字の向き。
- リファレンス: `Theme`、`Style`、`ThemeVar` は `cargo doc -p kivact-style --open`、`kivact::ui::theme` は `cargo doc -p kivact-ui --open`。
