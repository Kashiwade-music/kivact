# テーマと recipe を差し替える

標準部品の見た目は `Theme` で決まります。小さい変更から順に、token、recipe、slot の三段で変えます。

1. **token。** 色の役割（`ColorToken::Accent` など）、余白、角丸、文字、focus ring の幅（`focus_ring`）、disabled の不透明度（`disabled_opacity`）、操作部品の高さ、icon の表（`icons`）です。値は `theme.space().md`、`theme.focus_ring()` のように accessor で読み、`Theme::light().with_focus_ring(3.0)`、`theme.with_colors(Palette { accent, ..theme.colors().clone() })` のように `with_*` の builder で変えます。値を変えたテーマを渡すと、すべての標準部品が変わります。色を受け取る method（`background`、`color`、`border`、`ring`、gradient の stop、`Draw` の fill）は `ThemeColor` を受けます。`rgb(..)` の色、`ColorToken::Accent` のようなテーマの色の役割、kit の `Color`（`basic::Color::Border`）のどれでも渡せ、描くときのテーマから解決されるので、テーマの切替に追従します。
2. **recipe。** 部位ごとに、テーマと部位の状態から `Style` を返す関数です。`UiRecipes` にまとめ、`Theme::with` で載せます。recipe が返すのは style だけです。
3. **slot。** 項目に何を表示するかです。`tabs(..).item(..)`、`radio_group(..).item(..)`、`menu(..).item(..)`、`select(..).option(..)` に view を作る関数を渡します。渡さなければ、項目の icon、label、description、check と radio の印を表示します。

## 既存の recipe を包んで一部だけ変える

`UiRecipes` の method は、部位の今の recipe を受けて新しい recipe を返す関数をとります。今の recipe を呼んでから上書きすれば、元の見た目を保ったまま一部だけを変えられます。kit のテーマにも同じ方法が使えます。

`UiRecipes` の field は private で、変える道はこの method だけです。今の recipe を読むだけなら `button_recipe()` のように `<部位>_recipe()` を、select や menu などの部品ごとの表は `select_recipes()` のように `<表>_recipes()` を使います（`recipes.tabs_recipes().tab_recipe()`）。上書きの method は読む method から `_recipe` と `_recipes` を除いた名前です。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::UiRecipes;

fn main() -> kivact::Result<()> {
    let theme = Theme::light();
    // テーマが持つ recipe を読み、button だけ角を落とす。`get` は kit の token も
    // 読める generic な method なので、読む型（UiRecipes）を書きます。
    let recipes = UiRecipes::clone(&theme.get::<UiRecipes>());
    let recipes = recipes.button(|prev| move |theme, state| prev(theme, state).radius(0));
    let theme = theme.with(recipes);
    App::new("テーマの例").theme(theme).run(|| button("保存"))
}
```

recipe を差し替えても、入力・キー・focus・a11y は共通の interaction に残ります。色だけの変更なら token、個別の追加なら caller の `.style(...)` から始めます。caller の style は recipe の後に重なり、明示した値が勝ちます。

## 一つのテーマで同じ部品を別の見た目にする

recipe が受け取るのは `Look<S>` です。部位の状態 `S`（`look.selected`、`look.input` のように field を読めます）と、部品に付けた variant の値をまとめたものです。variant の型は `Variant` を実装し（`impl Variant for Segmented {}`）、標準部品はどれも `.variant(value)` でそれを受け、その部品の recipe はすべて `look.variant::<T>()` で読めます。button の `ButtonVariant` も同じ口で渡します。型ごとに一つで、後から付けた値が前の値を置き換えます。その型を知らないテーマは普通の見た目で描きます。同じテーマの中で、普通の radio group と segmented の radio group を並べられます。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{CheckState, Look, UiRecipes, Variant, radio_group};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Segmented;

impl Variant for Segmented {}

fn recipes() -> UiRecipes {
    UiRecipes::default().radio_group(|table| {
        table.item(|prev| {
            move |theme: &Theme, look: &Look<CheckState>| match look.variant::<Segmented>() {
                Some(Segmented) => style().padding_x(12).height(28).radius(6),
                None => prev(theme, look),
            }
        })
    })
}

fn view() -> impl IntoView {
    let (density, range) = (signal(Some("compact")), signal(Some("day")));
    column((
        radio_group(
            "Density",
            vec![ListItem::new("compact", "Compact"), ListItem::new("comfortable", "Comfortable")],
            density,
        ),
        radio_group("Range", vec![ListItem::new("day", "Day"), ListItem::new("week", "Week")], range)
            .orientation(Orientation::Horizontal)
            .variant(Segmented),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("variant の例").theme(Theme::light().with(recipes())).run(view)
}
```

見た目の変種を別の部品として組み直したり、部分木のテーマで切り替えたりしません。振る舞い（キー、選択、a11y）は同じ標準部品のままです。

## hover・押下・focus・disabled

状態ごとの見た目は、返す style の状態 variant で書きます：`style().hover(..)`、`.pressed(..)`、`.focus(..)`（focus を持つ間）、`.focus_visible(..)`（キーボードなどで focus が見えるべき間）、`.disabled(..)`。recipe の中で `look.input.hovered` を読んで分岐しません。

`hover` は CSS の `:hover` と同じく、pointer がその要素か子の上にある間です。子を祖先の hover で変えるとき（表の行の上で操作の列を出すなど）は、祖先に `style().group()`、子に `style().opacity(0.0).group_hover(style().opacity(1.0))` と書きます。最も近い `group` の祖先が hover の間だけ当たり、pointer の Enter/Leave を読む必要はありません。focus も同じで、`group_focus(..)` は、行の中のどれかの control に focus がある間（CSS の `:focus-within`）に当たります（キーボードで行に入ったときも操作の列が出ます）。

focus できる要素は、style に `focus` も `focus_visible` もなければ、focus が見える間だけテーマの focus ring（`focus_ring` の幅、`ColorToken::Focus` の色）を描きます。disabled の間はテーマの `disabled_opacity` で描かれ、hover と押下の見た目を持ちません。消すときは `style().focus_visible(style().ring_none())`、`style().disabled(style().opacity(1.0))` と書きます。

## icon

`Icon::named("check")` はテーマの `icons`（`IconSet`）で画像に解決されます。標準部品の印（`check`、`dot`、`chevron-down`、`chevron-right`、`x`）もこの表から引くので、kit のテーマを入れると kit の icon で描かれます。表にない名前は何も描かず、debug build では診断を出します。表示するには `kivact::control::icon(..)` を使います。

```rust,no_run
use kivact::prelude::*;
use kivact::style::Icon;

// アプリの SVG（`include_bytes!("star.svg")` など）。
static STAR: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 2l3 7h7l-6 4 2 7-6-4-6 4 2-7-6-4h7z"/></svg>"#;

fn main() -> kivact::Result<()> {
    let theme = Theme::light().with_icon("star", STAR);
    App::new("icon の例").theme(theme).run(|| icon(Icon::named("star")))
}
```

## 自分の token

部品のライブラリーは、自分の色や大きさを型にまとめて `ThemeExtension` を実装し、`Theme::with` で載せます。読むときは `theme.get::<T>()` です。style の closure や recipe で読めば、テーマを切り替えたときに作り直さずに描き直されます。テーマに載っていなければ `ThemeExtension::fallback` の値になります。`UiRecipes` のような recipe の表も同じ口（`Theme::with`、`Theme::get`）で載せて読みます。

`provide_theme` は部分木にテーマを渡します。反応的に読む closure では `current_theme` を使い、テーマを setup 時の固定値として複製しないようにします。独自 token の定数入力は、型を定義した crate の `view::constant_value!` か、呼出し側の `view::constant(値)` を使えます。公式 kit はテーマと分野固有の部品を提供し、標準の button や入力欄を別実装に置き換えません。

局所的な描画を recipe に加える場合は `Style::paint` を使います。native 編集の leaf、text/image、overlay に独自の paint を追加することはできません。必要な枠や装飾は外側の node に描き、編集の内容と caret は native の経路に任せます。
