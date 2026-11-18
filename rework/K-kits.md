# 付属書 K：kit・catalog・kivact-plot・catalog-shell と kit が要る中核の追加

版 1.0（2026-11-18）。設計の正本は `REWORK_DESIGN.md`（以下「設計」）。本付属書は設計 5.1・5.7・5.9・5.10・6.3・7.6・DEC-14 と監査 `rework/audit/kits.md`（KIT-01〜KIT-20）、`rework/audit/tooling.md` の TL-13 を unit にする。

## 1. 範囲と前提

### 1.1 範囲

| 対象 | 内容 |
|---|---|
| `crates/kivact-ui-{basic,pop,game,creative,soft}`（以下「kit」） | 骨格の統一（KIT-06）、theme 変数（KIT-07 変更して採用、DEC-06）、`<Name>Theme` の recipe、`#[component]`（DEC-03）、variant の改名と束の分解（KIT-04、KIT-06）、kit 固有の所見（KIT-01〜03、09、11、13、16、17、19）、README（KIT-16）、kit の試験（TL-13、KIT-20） |
| `examples/kivact-ui-{basic,pop,game,creative,soft}`（以下「catalog」） | `examples/catalog-shell` への移行、英語化（KIT-12） |
| 新しい crate `crates/kivact-plot` | basic の `plot` を移す（KIT-15、設計 5.10） |
| 新しい crate `examples/catalog-shell`（`publish = false`） | 5 つの catalog の共通の shell・page 一覧・検索・試験の walker（KIT-12、KIT-20） |
| kit が要る中核の追加 | kivact-style：`icons!`（KIT-08）。kivact-ui：`Slant`・`Striped`（KIT-04）、`SyntaxKind` と構文色の theme 変数（KIT-05）、`sidebar`（KIT-10）、`TabOverflow::{Menu, Scroll}`（KIT-09）。kivact-core：`NodeRef::once_visible`（KIT-11）。`task::clock`（KIT-17、設計 5.5）は付属書 R の R23（W5a）が作り、K42 が使う |
| `crates/kivact-devtools` | kit への依存を消す（KIT-18） |
| `dev/tests/kivact-kit-tests`（package `kivact-kit-tests`、binary `e2e_kits`。T15 が作る） | W11a の K44 だけ（theme_conformance の書き直しと TL-13 の共通の検査） |

範囲外：`docs/for-kivact-ui-user/`（110 file。付属書 G が W11 で新しい API に直す。KIT-16 の決定どおり日本語の docs は全ページ残す）、`dev/tests/docs`（付属書 G）、`docs/for-kivact-developer/`（付属書 G）。

### 1.2 前提（先の wave で他の付属書が済ませていること）

| wave | 付属書 | 本付属書が使うもの |
|---|---|---|
| W1 | T | `state` 検査（struct 単位の `// xtask-allow(state): ...`）、`xtask-allow(state-fields)` の全削除、e2e の分割（`e2e_kits` binary）、`structural-options` 検査の削除 |
| W5 | R・C | `#[component]`（設計 5.1 の規則 1〜8）、`Signal<T>` への統合（`Binding`・`IntoBinding` の削除）、`IntoHandler`・`Callback<E>`、`#[derive(Prop)]`（`constant_value!` は `#[doc(hidden)]`）、`Element::variant<V: Variant>(self, v: impl IntoValue<V>)`、`Trigger`、`use_context -> T`、prelude（5.11） |
| W6 | S | `ThemeVar<T>`、`Color::var`、`Length::var`、`Theme::set`・`Theme::get`、中核の変数 `kivact::style::{color, space, radius, font}`、`Theme::with_fonts`・`Theme::with_icons`、`Color::relative_luminance`・`Color::contrast_ratio`、CSS の名前（`font_weight` など）、`ThemeColor`・`ColorToken`・`Palette`・`Spacing`・`Radii`・`use_tokens` の削除、DEC-14 の除外（kit・catalog・kit の試験・`e2e_kits` を compile から外す） |
| W7〜W8 | U | `kivact::ui::theme::{<Name>Theme, <Name>State, Look, Variant}`、部品の theme 変数（`kivact::ui::theme::<component>::*`）、`UiRecipes`・`UiTokens` の削除、`kivact::ui` の部品の `#[component]` 化、開閉の語彙、`Look::variant::<V>()` |

### 1.3 本付属書が後の wave に渡すもの

- W11 の付属書 G：新しい kit の API（4.4 節の公開名の表）、`kivact-plot`、`catalog-shell`。docs の書き直しはこの表を使う。
- 最終 checkpoint（AC-4）：5.12 節の kit 関係の削除名（`ColorName`、`BadgeHue`、`ItemHue`、`StatHue`、`ProgressHue`、`WeekendHue`、`ButtonTone`（game）、`CodeTone`、`CodeColors`、`TokenKind`、`GameInputGroupVariant`、`GameCellVariant`、`PopBubble`）が `crates/`・`examples/` に残らない。

### 1.4 設計の決定の要約（本付属書が前提にする 7.6 節の判定）

KIT-05：中核は `SyntaxKind` と構文色の theme 変数だけを持ち、Rust の tokenizer は basic に残す。KIT-07：`theme_colors!` macro は作らず、theme 変数で `ColorName` ごと消す。KIT-08：`icons!`。kit の theme と icon set は `thread_local!` の static（設計 12 章 PF-02 が `cached_theme` と fn pointer の共有をやめた）。KIT-09：soft の `tab_list` の overflow は中核の `tabs` へ。KIT-12：catalog の文言は英語。KIT-16：日本語の docs は全ページ残す、migration 表と recipe の一覧は消す。KIT-17：時計は `task::clock`。Q1：soft の大きな分野の部品（markdown、rich_blocks など）は soft に残し、使われていない関数だけ消す。Q3：`kivact-plot`。Q6：sidebar は中核へ。

## 2. unit の一覧

### 2.1 一覧

| ID | wave | lane | 前提の unit | 所有するファイル・directory | 所見 | 規模 |
|---|---|---|---|---|---|---|
| K01 | W2a | K | — | `crates/kivact-devtools/Cargo.toml`、`src/app.rs`、`src/lib.rs`、`src/look.rs`（新）、`src/panels/performance.rs`、`src/widgets.rs`、`src/widgets/counters_chart.rs`（新）、`Cargo.lock` | KIT-18 | S |
| K02 | W2a | K | K01 | `crates/kivact-ui-soft/src/mobile_nav.rs` | KIT-01 | S |
| K03 | W2a | K | K02 | `crates/kivact-ui-creative/src/editors/`（`shared/gesture.rs`、`shared/targets.rs`、`time/view.rs`、`time/input.rs`、`arrangement/view.rs`、`arrangement/input.rs`、`graph/view.rs`、`graph/input.rs`、`node/view.rs`、`node/input.rs`、`node/gesture.rs`）、`src/picker.rs`、`src/audio/meter.rs` | KIT-02 | M |
| K04 | W2a | K | K03 | `crates/kivact-ui-game/src/surface.rs` | KIT-11（halo） | S |
| K05 | W2a | K | K04 | `crates/kivact-ui-soft/src/timestamp.rs` | KIT-03（日付の計算） | S |
| K06 | W9 | K6 | W8 の完了 | root `Cargo.toml`（`members` の行）、`crates/kivact-plot/`（新：`Cargo.toml`、`src/lib.rs`）、`examples/catalog-shell/`（新：`Cargo.toml`、`src/lib.rs`）、`dev/xtask/policy/workspace.toml`、`Cargo.lock` | KIT-12、KIT-15 | S |
| K07 | W9 | K6 | K06 | `crates/kivact-style/src/icons_macro.rs`（新）、`crates/kivact-style/src/lib.rs`、`crates/kivact-style/src/facade.rs` | KIT-08 | S |
| K08 | W9 | K6 | K07 | `crates/kivact-ui/src/variant_marks.rs`（新）、`crates/kivact-ui/src/syntax.rs`（新）、`crates/kivact-ui/src/syntax_colors.rs`（新）、`crates/kivact-ui/src/lib.rs`、`crates/kivact-ui/src/theme.rs` | KIT-04、KIT-05 | M |
| K09 | W9 | K6 | K08 | `crates/kivact-core/src/view/node_ref_visibility.rs`（新）、`crates/kivact-core/src/view.rs`（`mod` 行）、`dev/tests/kivact-public-api/tests/e2e_core/node_ref.rs` | KIT-11 | S |
| K10 | W9 | K6 | K09 | `crates/kivact-plot/`（全体）、`crates/kivact-ui-basic/src/plot.rs`、`crates/kivact-ui-basic/src/plot/`、`crates/kivact-ui-basic/src/nice.rs`、`crates/kivact-ui-basic/tests/consumer/plot.rs`（移動して消す） | KIT-15、KIT-10（目盛り） | L |
| K11 | W9 | K6 | K10 | `crates/kivact-ui/src/sidebar.rs` と `crates/kivact-ui/src/sidebar/`（新）、`crates/kivact-ui/src/lib.rs`、`crates/kivact-ui/src/theme.rs`、`crates/kivact-ui-basic/src/sidebar.rs`、`crates/kivact-ui-basic/src/sidebar/`（移動して消す）、`dev/tests/kivact-public-api/tests/e2e_ui/ui_sidebar.rs`（新）と `e2e_ui` の `main.rs` の `mod` 行 | KIT-10（sidebar） | L |
| K12 | W9 | K6 | K11 | `crates/kivact-ui/src/sidebar/`、`crates/kivact-ui/src/lib.rs`、`dev/tests/kivact-public-api/tests/e2e_ui/ui_sidebar.rs` | KIT-10（soft の分） | M |
| K13 | W9 | K6 | K12 | `crates/kivact-ui/src/tabs.rs`、`crates/kivact-ui/src/tabs/` と `crates/kivact-ui/src/tabs/overflow.rs`（新）、`crates/kivact-ui/src/lib.rs`、`dev/tests/kivact-public-api/tests/e2e_ui/ui_tabs_overflow.rs`（新）と `e2e_ui` の `main.rs` の `mod` 行 | KIT-09（overflow）、UI-K-01、UI-K-02 | L |
| K14 | W9 | K6 | K13 | `examples/catalog-shell/`（全体） | KIT-12、KIT-20 | M |
| K15 | W9 | K1 | W8 の完了 | `crates/kivact-ui-basic/`（`src/plot.rs`・`src/plot/`・`src/nice.rs`・`src/sidebar.rs`・`src/sidebar/`・`tests/consumer/plot.rs` を除く） | KIT-06、KIT-07、KIT-08（theme の cache） | M |
| K16 | W9 | K1 | K15 | 同上 | KIT-06（recipe 1） | M |
| K17 | W9 | K1 | K16 | 同上 | KIT-06（recipe 2） | M |
| K18 | W9 | K1 | K17 | 同上 | DEC-03（部品 1） | L |
| K19 | W9 | K1 | K18 | 同上 | DEC-03（部品 2） | L |
| K20 | W9 | K2 | W8 の完了 | `crates/kivact-ui-pop/` | KIT-06、KIT-07、KIT-13（pop）、KIT-14 | L |
| K21 | W9 | K2 | K20 | `crates/kivact-ui-pop/` | DEC-03、KIT-04（hue） | M |
| K22 | W9 | K3 | W8 の完了 | `crates/kivact-ui-creative/` | KIT-06、KIT-07 | M |
| K23 | W9 | K3 | K22 | `crates/kivact-ui-creative/` | DEC-03（部品 1） | L |
| K24 | W9 | K3 | K23 | `crates/kivact-ui-creative/` | DEC-03（部品 2：editors） | L |
| K25 | W9 | K3 | K24 | `crates/kivact-ui-creative/` | DEC-03（部品 3） | L |
| K26 | W9 | K4 | W8 の完了 | `crates/kivact-ui-game/` | KIT-06、KIT-07、KIT-13 | L |
| K27 | W9 | K4 | K26 | `crates/kivact-ui-game/` | DEC-03、KIT-04（`ButtonTone`、改名） | M |
| K28 | W9 | K5 | W8 の完了 | `crates/kivact-ui-soft/` | KIT-06、KIT-07、KIT-19 | L |
| K29 | W9 | K5 | K28 | `crates/kivact-ui-soft/` | KIT-06（recipe 1） | L |
| K30 | W9 | K5 | K29 | `crates/kivact-ui-soft/` | KIT-06（recipe 2）、束の分解 | L |
| K31 | W9 | K5 | K30 | `crates/kivact-ui-soft/` | DEC-03（部品 1） | L |
| K32 | W9 | K5 | K31 | `crates/kivact-ui-soft/` | DEC-03（部品 2） | L |
| K33 | W9 | K5 | K32 | `crates/kivact-ui-soft/` | DEC-03（部品 3） | L |
| K34 | W10 | K1 | K19、K07、K08、K10、K11 | `crates/kivact-ui-basic/`（全体） | KIT-05、KIT-08、KIT-10、KIT-15 | M |
| K35 | W10 | K1 | K34、K14 | `crates/kivact-ui-basic/`、`examples/kivact-ui-basic/` | KIT-12、KIT-16、KIT-20、TL-13 | L |
| K36 | W10 | K2 | K21、K07、K08 | `crates/kivact-ui-pop/` | KIT-04、KIT-08 | S |
| K37 | W10 | K2 | K36、K14 | `crates/kivact-ui-pop/`、`examples/kivact-ui-pop/` | KIT-12、KIT-16、KIT-20、TL-13 | L |
| K38 | W10 | K3 | K25、K07、K14 | `crates/kivact-ui-creative/`、`examples/kivact-ui-creative/` | KIT-08、KIT-12、KIT-16、KIT-20、TL-13 | L |
| K39 | W10 | K4 | K27、K07、K08、K09 | `crates/kivact-ui-game/` | KIT-04、KIT-08、KIT-11（kpi） | S |
| K40 | W10 | K4 | K39、K14 | `crates/kivact-ui-game/`、`examples/kivact-ui-game/` | KIT-12、KIT-13（docs と catalog）、KIT-16、KIT-20、TL-13 | L |
| K41 | W10 | K5 | K33、K07、K08、K13 | `crates/kivact-ui-soft/` | KIT-05、KIT-08、KIT-09、KIT-17（色） | L |
| K42 | W10 | K5 | K41、K11、K12、R23（W5a） | `crates/kivact-ui-soft/` | KIT-03（残り）、KIT-10、KIT-17（時計） | L |
| K43 | W10 | K5 | K42、K14 | `crates/kivact-ui-soft/`、`examples/kivact-ui-soft/` | KIT-12、KIT-16、KIT-20、TL-13 | L |
| K44 | W11a | K | K35、K37、K38、K40、K43 | `dev/tests/kivact-kit-tests/Cargo.toml`、`dev/tests/kivact-kit-tests/tests/e2e_kits/`（全体）、`crates/kivact-ui-*/tests/consumer/` の 4.6 節で「移す」とした file | TL-13、KIT-16（README の検査の削除）、KIT-20 | M |

### 2.2 排他の規則

- 並行の lane の unit は同じ file を持たない。同じ lane の unit は順に実行するので、同じ file を順に持ってよい（kit の lane は kit の directory を wave の間ずっと持つ）。
- W9 の basic の lane（K15〜K19）は `crates/kivact-ui-basic/src/{plot.rs, plot/, nice.rs, sidebar.rs, sidebar/}` と `crates/kivact-ui-basic/tests/consumer/plot.rs` を持たない。この 6 つは K6 の lane の K10・K11 が持ち、`git mv` で外へ出す。
- root `Cargo.toml` は K06 だけが変える（新しい 2 crate の `members` の 2 行）。kit と catalog の行は変えない：W6〜W9 の除外は gate の `--exclude` だけで行う（INC-09、作業指示書の全体の指示 4）。`Cargo.lock` の conflict は `cargo metadata --format-version 1 > /dev/null` を流して作り直す。
- `crates/kivact-ui/src/lib.rs` は W9 では K6 の lane だけが持つ（K08、K11、K13 が順に `mod` 行を足す）。
- `dev/xtask/policy/workspace.toml` は W9 では K06 だけが持つ。

### 2.3 kit の lane の依存（W10 の unit がどの K6 の unit を要るか）

| kit の unit | 要る K6 の unit | 使うもの |
|---|---|---|
| K34（basic） | K07、K08、K10、K11 | `icons!`、`SyntaxKind`・構文色、`kivact_plot::nice::round_step`、中核の `sidebar` と `SidebarTheme` |
| K35（basic catalog） | K14 | `catalog_shell` |
| K36（pop） | K07、K08 | `icons!`、`Slant`・`Striped` |
| K37（pop catalog） | K14 | `catalog_shell` |
| K38（creative） | K07、K14 | `icons!`、`catalog_shell` |
| K39（game） | K07、K08、K09 | `icons!`、`Slant`・`Striped`、`NodeRef::once_visible` |
| K40（game catalog） | K14 | `catalog_shell` |
| K41（soft） | K07、K08、K13 | `icons!`、`SyntaxKind`、`TabOverflow` |
| K42（soft） | K11、K12（と R23 の `kivact::task::clock`） | R23 の `task::clock`、中核の `sidebar`・`SidebarTheme` と soft のための拡張 |
| K43（soft catalog） | K14 | `catalog_shell` |

W9 の kit の unit（K15〜K33）は K6 の unit を一つも要らない。W9 の間、kit の `icons.rs` は今の `OnceLock` の実装をそのまま使い（変えない）、W10 の unit が `thread_local!` の static に置き換える。

## 3. unit の手順

### 3.0 共通の規則（全 K unit の正本。各 unit は番号でここを引く）

#### 3.0.1 kit の骨格（KIT-06、設計 5.10 の正本）

```
crates/kivact-ui-<kit>/
  Cargo.toml              [dependencies] は kivact だけ（basic だけ kivact-plot も。4.1 節の 1、parent 採用）
  README.md               3.0.10 の雛形（移行表・recipe の一覧を置かない）
  assets/icons/*.svg      kit 自身の icon（今の file をそのまま）
  src/lib.rs              crate doc（1 段落と使い方の例）→ mod 行 → pub use（群ごとに空行。群の順は下の file の順）
  src/theme.rs            pub fn theme(mode: ThemeMode) -> Theme（thread_local! の static。3.0.7）
                          ThemeOptions と theme_with(mode, options) は game と creative だけ
                          fn build(mode: ThemeMode[, options: &ThemeOptions]) -> Theme
  src/color.rs            kit の色の theme 変数と pub(crate) fn set_colors(theme: Theme, mode: ThemeMode) -> Theme
  src/size.rs             kit の長さの theme 変数と pub(crate) fn set_sizes(theme: Theme[, ..]) -> Theme（f64 の token がある kit だけ）
  src/fonts.rs            kit の font family の theme 変数（`pub static <NAME>_VAR: ThemeVar<FontFamily>` だけ。const は作らない：S 4.1。game の DISPLAY_FAMILY_VAR・MONO_FAMILY_VAR、creative の MONO_FAMILY_VAR だけ）
  src/icons.rs            icons! の一覧と pub(crate) fn set() -> IconSet（thread_local! の static。W10 で入れる）
  src/variants.rs         kit が標準部品に足す variant の型（enum・marker・newtype）と impl Variant。他は置かない
  src/recipe_parts.rs     二つ以上の recipes_<group>.rs が使う recipe の helper（pub(crate)。要らない kit は作らない）
  src/recipes_<group>.rs  <Name>Theme の recipe と pub(crate) fn install(theme: Theme) -> Theme（3.0.3）
  src/<domain>.rs、src/<domain>/   分野の部品（#[component]）。recipe を置かない
  tests/consumer.rs と tests/consumer/*.rs   3.0.8
```

規則：

1. `tokens.rs`、`color_name.rs`、`theme_access.rs`、`recipe_style.rs`、`recipes.rs`（`UiRecipes` を組む file）は作らない。今ある kit はその unit で消す。
2. `src/recipes_<group>.rs` の `<group>` は次の表の値だけ。一つの file は 400 code 行まで。越えるときは `recipes_<group>_<部品名>.rs` に分ける（例 `recipes_controls_button.rs`）。

| group | `<Name>Theme` の部品（annex U の部品名） |
|---|---|
| `controls` | button、toggle、toggle_button、checkbox、switch、radio_group、slider、range_slider、text_input、text_area、number_input、field、split、splitter、progress、disclosure、disclosure_group |
| `overlays` | select、combobox、menu、menubar、dialog、tooltip、hover_card、command_palette、popover |
| `collections` | list、item_list、data_table、table、tree、toolbar、toggle_group、button_group、carousel |
| `navigation` | tabs、breadcrumb、pagination、stepper、navigation_menu、sidebar |
| `feedback` | toast、alert、empty_state、skeleton、spinner、badge、kbd、avatar、avatar_group、indicator、typing_indicator |
| `metrics` | stat、heading、description_list、sparkline |
| `chat` | chat_message、chat_thread、chat_composer |
| `input` | input_group、date_input、date_range_input、date_time_input、time_input、tag_input、dropzone |
| `calendar` | calendar、date_sheet、date_wheel |
| `code` | code_block |

上の表に無い `<Name>Theme`（annex U が足したもの）は、annex U 4 章の部品の群の表の群と同じ名前の group に置く。

3. variant の型は `variants.rs` だけに置く。分野の部品の props の enum（soft の `TextColor` など、variant ではなく props の型）は、その部品の module に置く。
4. kit の message の key は `<kit>.<部品>.<名前>`（`basic.plot.value` は `kivact-plot` に移るので `plot.value`）。中核の部品の key（`dialog.close` など）は中核の key のまま。

#### 3.0.2 theme 変数への変換（KIT-07 変更して採用、DEC-06）

T1. kit の `Tokens` の `Color` の field `x` は `src/color.rs` の theme 変数にする。名前は field 名の大文字（`muted_foreground` → `MUTED_FOREGROUND`）。

```rust
use kivact::style::{Color, ThemeMode, ThemeVar, color, rgb, rgba};

/// `--muted-foreground`: secondary text.            // 今の field の doc をそのまま写す
pub static MUTED_FOREGROUND_VAR: ThemeVar<Color> =
    ThemeVar::alias("basic.muted_foreground", &color::TEXT_MUTED_VAR); // 名前は "<kit>.<field>"、fallback は T1a（別名でない変数は `ThemeVar::new("basic.card", rgb(0xffffff))`）
pub const MUTED_FOREGROUND: Color = Color::var(&MUTED_FOREGROUND_VAR);
```

T1a. fallback（設計 12 章 UI-U-02、5.7）：今の `theme.rs` が中核の値（`Palette` の field、`Spacing`・`Radii`・`Typography` の field、`control::HEIGHT_VAR` などの変数）に kit の token **そのもの**を入れている行（`surface: p.background` のように、式でなく token の読みだけの行）ごとに、その kit の変数の fallback を対応する中核の変数の別名にする（`pub static BACKGROUND_VAR: ThemeVar<Color> = ThemeVar::alias("basic.background", &color::SURFACE_VAR);`。別名の書き方はこれ一つ：annex S 4.1、INC2 N-11。値の形の `Color::var(&X)` を fallback に書かない）。kit の theme が無い所（素の theme や他の kit の下）でも、kit の部品が中核の値（mode・利用者の set）に従う。別名にする対象は kit の theme.rs を見て決まり、kit ごとの表（K15、K20、K22、K26、K28 の「別名の fallback」）が全部を挙げる。その他の変数の fallback は今の `Tokens::new(ThemeMode::Light)` の値。kit の theme の中では `set_colors`・`set_sizes` が全部の kit の変数に値を set するので、中核の変数 → kit の変数 → 中核の変数の循環は起きない。色以外の型（`f64`、`FontFamily`）の別名も同じ `ThemeVar::alias(name, &X_VAR)`。kit ごとの表の「`X` ← `color::Y`」は `ThemeVar::alias("<kit>.<x>", &color::Y_VAR)` と読む。

T2. 配列の field `xs: [Color; N]` は `XS_1`〜`XS_N`（例 basic `chart` → `CHART_1`〜`CHART_5`）。
T3. `f64` の field は `src/size.rs` の `ThemeVar<f64>` にし、`pub const X: Length = Length::var(&X_VAR);` を置く。
T4. `mode` の field は消す。読んでいた所は `theme.mode()` を読む。`density`、font の名前（`String`）などの残りの field は kit ごとの表（各 unit）に従う。
T5. `color.rs` に次の関数を置き、今の `Tokens::new(mode)`（と kit の options）が計算する値を、一つ残らず同じ値で set する。値は今の `tokens.rs`（soft は `color.rs`・`tokens.rs`）の式を写す。light と dark で同じ値の変数も set する。

```rust
/// Set every color variable of the kit to its value in `mode`.
pub(crate) fn set_colors(theme: Theme, mode: ThemeMode) -> Theme {
    let dark = mode == ThemeMode::Dark;
    let pick = |light: u32, dark_value: u32| rgb(if dark { dark_value } else { light });
    theme
        .set(&BACKGROUND_VAR, pick(0xffffff, 0x0a0a0a))
        .set(&FOREGROUND_VAR, pick(0x0a0a0a, 0xfafafa))
        // ... Tokens の field の宣言順に、全部
}
```

1 関数が 80 code 行を越えるときは、群ごとに `fn set_colors_<群>(theme: Theme, dark: bool) -> Theme` に分け、`set_colors` はそれを順に呼ぶ。

T6. `theme.rs` の今の `Palette`・`Spacing`・`Radii`・`Typography` は、annex S 4.2 節の表「中核の theme 変数」（以下「S 表」。`color::{SURFACE, SURFACE_ALT, TEXT, TEXT_MUTED, ACCENT, ACCENT_TEXT, BORDER, FOCUS, DANGER, SUCCESS, WARNING, OVERLAY, SHADOW, SELECTION, HOVER, PRESSED, RAISED}`、`space::{XS, SM, MD, LG, XL}`、`radius::{SM, MD, LG}`、`font::{SIZE, SIZE_SMALL, SIZE_LARGE}`（const と `_VAR`）、`font::{FAMILY_VAR, WEIGHT_VAR, WEIGHT_STRONG_VAR, LINE_HEIGHT_VAR}`（static だけ））の `_VAR` への `set` に置き換える（`Palette::text_muted` → `color::TEXT_MUTED_VAR`、`Typography::family` → `font::FAMILY_VAR` など field の大文字。旧い名前の対応は S 4.3 節の表）。`with_control_height(x)`・`with_disabled_opacity(x)`・`with_overlay_margin(x)` は `Theme` から消える（annex S の S15、INC2 N-05）ので、`.set(&control::HEIGHT_VAR, x)`・`.set(&control::DISABLED_OPACITY_VAR, x)`・`.set(&control::OVERLAY_MARGIN_VAR, x)`（`kivact::style::control`）にする。`with_mode`・`with_focus_ring`・`with_focus_ring_offset`・`with_icons`・`with_fonts` は `Theme` の builder のまま書く。今の値が kit の token のとき、中核の変数に kit の変数を別名として set する：`.set(&color::ACCENT_VAR, &PRIMARY_VAR)`（変数を値にする書き方はこれ一つ。INC2 N-11）。各 unit の表の「`A_VAR` ← `B`」で `B` が変数（kit の const の名前か `_VAR`）のときは `.set(&A_VAR, &B_VAR)` と書く。今の値が式（`p.ring.with_alpha(0.5)`）のときは、その式を `Theme::get` で計算して set する（例 `let ring = theme.get(&RING_VAR); theme.set(&color::FOCUS_VAR, ring.with_alpha(0.5))`。`theme` は kit の変数を set した後の値）。
T7. 今の `UiTokens`（`kivact::ui::UiTokens { .. }` と `ui.<部品>.<field> = ..`）は、annex U 4.3 節の表「UiTokens → 部品の theme 変数」（以下「U 変数表」）の変数への `set` に置き換える。今の値が `Some(x)` の `Option` の field は `x` を set する。U 変数表に行が無い field は 4.5 節に書いた kit ごとの扱いに従う。
T8. 呼出し側の置換（recipe、draw、style、分野の部品のすべて）：

| 今の書き方 | 置き換え |
|---|---|
| `t.ext::<Tokens>().x`、`p.x`（`let p = t.ext::<Tokens>()`、`use_tokens::<Tokens>()`、`d.theme().ext::<Tokens>()`、`tokens()`）を style の setter・draw の呼出しにそのまま渡す | `X` |
| 上の値で計算する（`.with_alpha(..)`、`.mix(..)`、`.mix_in(..)`、`.resolve(..)`、比較、算術、`Copy` の局所変数に入れて二度以上使う） | `t.get(&X_VAR)`（recipe は引数の `t: &Theme`、draw は `d.theme()`、style の closure の中は `use_theme().get()`） |
| `ColorName::FooBar` | `FOO_BAR` |
| `ColorToken::Foo`、`Color::token(ColorToken::Foo)` | S 表の「ColorToken → color 変数」の行の変数 |
| `t.colors().x`、`t.space().x`、`t.radius().x`、`t.text().size` など | S 表の行の変数 |
| 今の `Tokens` の helper の関数（pop `Tokens::hue`、`Tokens::accent`、creative `Accent::colors`、`LabelTone::color`、game `Tokens::palette`） | 各 unit の表の置き換え |
| `.style(move \|\| ...)` の closure の中の reactive な読みが theme と token だけ | closure を外して `.style(style()...)` にする（設計 5.7 規則 5） |

T9. 色の literal（`rgb`、`rgba`、`hsl`）は `color.rs` と `theme.rs` だけに書く（xtask `kit-literal-color`）。kit の他の file に literal の色が要るときは `color.rs` に変数か `pub(crate) const` を作る。
T10. 消すもの（各 kit の foundation の unit）：`Tokens`、`impl ThemeExtension for Tokens`、`ColorName`、`impl ThemeColor for ColorName`、kit の `Palette` を返す関数、`tokens.rs`、`color_name.rs`。

#### 3.0.3 recipe の変換（`UiRecipes` → `<Name>Theme`。UI-03、UI-24、設計 5.9）

R1. 対応表は annex U 4.4 節の表「UiRecipes → <Name>Theme」（以下「U recipe 表」）。書き方は同じ節の「recipe の上書きの書き方」（`Look::variant::<V>()` は `Option<V>` の値）。この表は今の `UiRecipes` の setter（`.button`、`.select` など）と、入れ子の `*Recipes` の setter（`SelectRecipes::trigger` など）を、新しい `<Name>Theme` と part の setter に対応させる。recipe が読む状態の型の改名（`ButtonState` → `ButtonState`、`CheckableState` → `CheckboxState` など）も同じ表に従う。
R2. recipe の関数の形：

```rust
// 今
fn button(t: &Theme, s: &Look<ButtonState>) -> Style { style().height(..) }
UiRecipes::default().button(|_| button)
// 変換後（今の recipe は中核の recipe を置き換えていたので、3 つ目の引数 base は使わない）
fn button(t: &Theme, s: &Look<ButtonState>, _base: Style) -> Style { style().height(..) }
ButtonTheme::default().root(button)
```

```rust
// 今（前の recipe を受けて足す形）
r.root(|prev| move |t: &Theme, s: &_| prev(t, s).gap(6))
// 変換後（3 つ目の引数が前の recipe の結果）
SomeTheme::default().root(|_t: &Theme, _s: &Look<SomeState>, base: Style| base.gap(6))
```

R3. 入れ子の表の関数（`pub(crate) fn select(r: SelectRecipes) -> SelectRecipes { r.trigger(|_| a).popup(|_| b) }`）は、`<Name>Theme` を返す関数にする：`pub(crate) fn select() -> SelectTheme { SelectTheme::default().trigger(a).popup(b) }`。
R4. 各 `recipes_<group>.rs` の末尾に次を置く。`.with(..)` は U recipe 表の順。

```rust
/// Install the kit's recipes of the <group> components.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(button()).with(toggle()).with(checkbox())
}
```

R5. `theme.rs` の `build` は、色・長さ・中核の変数を set した後に、次の順に `install` を呼ぶ（その kit に無い group は飛ばす）：controls、overlays、collections、navigation、feedback、metrics、chat、input、calendar、code。foundation の unit はこの呼出しを書かず、group を作る unit が自分の 1 行を足す。
R6. recipe の中の token の読みは T8 に従う。`Look` の読み（variant、part の状態）は annex U の `Look<S>` の API に従う（`s.variant::<V>()` は `Option<&V>` か `Option<V>` か、annex U の定義のまま）。
R7. 今 kit の分野の file にある recipe の関数（basic `alert.rs` の `recipes` など、各 unit の表）は、該当する `recipes_<group>.rs` に移す。移した後、元の file に variant の型が残るなら `variants.rs` に移す。
R8. 今の recipe の中で毎回作っていた gradient の `Paint`（pop `button.rs` など）は、色を theme 変数にしたうえでそのまま recipe の中で作る（`linear_gradient(90).colors([BLUE, BLUE_GRADIENT_END])`）。gradient の端の色が今 `mix_in` で計算されていたものは、kit の色の変数にして `set_colors` で計算した値を set する（KIT-14：recipe の評価ごとの計算と `Tokens` の確保をなくす）。

#### 3.0.4 variant の規則（KIT-04、KIT-06）

V1. 名前は `<Component>Variant`。`<Component>` は標準部品の関数名の UpperCamel（`radio_group` → `RadioGroup`）。分野の部品に付く variant は、その分野の部品の関数名を使う。
V2. 衝突の規則（parent 採用、設計 5.10）：`kivact::ui` が同じ名前の型を公開しているとき（W9 の時点の確認：`rg -n "pub (enum|struct) <Name>\b" crates/kivact-ui/src`。今は `AlertVariant`、`ButtonVariant`、`ChatMessageVariant`、`ProgressVariant`、`RadioGroupVariant`）、kit の型は `<Component>Tone` にする。
V3. kit 名の接頭辞（`Game`、`Pop`、`Soft`）を付けない。`*Look`・`*Appearance`・`*Hue`（pop の newtype）・`*Style` を公開の型名に使わない。
V4. 束の分解：`*Look`・`*Appearance` の struct（field が `Value<_>`・`Option<Value<_>>`・値）は消す。各 field を次のどれかの variant にし、呼出し側は field ごとに `.variant(..)` を付ける。
   - field の型が variant の enum `E`（kit の型）：`E` をそのまま使う（`impl Variant for E` が無ければ足す）。
   - field の型が中核の型（`ControlSize` など）で、中核がその型に `Variant` を実装している：そのまま使う。
   - それ以外（`bool`、`f64`、`usize`、`Color`、中核の型で `Variant` の無いもの）：newtype `pub struct <Component><Field>(pub T);`（`#[derive(Clone, Copy, Debug, PartialEq)]`、`T: Default` なら `Default` も）を作る。field 名は UpperCamel（`header_bordered` → `CardHeaderBordered`）。
   - field が束の外で使われていない（各 unit の表で「消す」とした field）：消し、今の既定値を recipe の定数にする。
   - recipe は `s.variant::<E>()` が無いとき、今の束の既定値を使う。
V5. `Value<T>` を包むだけの newtype（pop `BadgeHue(Value<Hue>)` など）は消す。呼出し側は `.variant(Hue::Pink)` か `.variant(hue_signal)`（`Element::variant` は `impl IntoValue<V>` を受ける）。
V6. `impl kivact::ui::Variant for X {}` は annex U の path（`kivact::ui::theme::Variant`）にする。
V7. kit の `constant_value!` は C09（W5a）が `#[derive(Prop)]` にしてある（INC-21）。型を `variants.rs` などに移すときは、その derive を付けたまま移す。`constant_value!` を書かない。
V8. variant の型の doc は「`kivact::ui::<部品>(..).variant(<型>::<値>)` で選ぶ。他の theme は無視して普通の見た目で描く」の一文を持つ。

#### 3.0.5 部品の `#[component]` への変換（設計 5.1、DEC-03）

C1. 今の形：`pub struct X { root: Node, f1: .., f2: .. }` ＋ `pub fn x(a: .., b: ..) -> X` ＋ setter（`pub fn s(mut self, v: impl ..) -> Self`）＋ `impl Element for X` ＋ `impl IntoView for X { fn into_view(self) -> View { 本体 } }`。これを一つの関数にする：

```rust
#[component]
pub fn x(
    a: Value<String>,                    // 今の位置引数（同じ順）。型は C3 の表
    b: Signal<bool>,
    s: Option<Value<f64>>,               // 今 Option の field の setter
    #[prop(default = 8.0)] gap: f64,     // 今 既定値つきの field の setter
    children: Vec<View>,                 // 今 子を受ける引数
) -> impl Element {
    // 今の into_view の本体。self.f は引数の f。root への style・modifier は本体の root の node に付ける
}
```

macro が `pub struct X` を生成するので、公開の型名 `X` は残る（4.4 節の表で「部品」と書いた行）。

C2. 本体の root：今の `root: Node` の field と `impl Element` は消す。呼出し側の modifier は macro の `absorb`（設計 5.1 規則 2）で本体の root に移る。今の本体が `root.child(..)` のように `root` を使っていたら、`node().child(..)` にする。今 `*stat.node_mut() = root` のように内側の部品の node に root を差し替えていたら、その内側の部品を本体の root にする（差し替えの行を消す）。
C3. props の型（設計 5.1 規則 3）：

| 今の引数・setter の型 | props の型 |
|---|---|
| `impl IntoText` | `Value<String>` |
| `impl IntoValue<T>` | `Value<T>` |
| `impl IntoBinding<T>`、`Binding<T>`（W5 で `Signal<T>` に変わった後の型） | `Signal<T>` |
| `impl Into<Callback<E>>`、`impl Fn(E) + 'static` の handler | `Callback<E>` |
| `impl IntoView` | `View`（任意なら `Option<View>`） |
| `impl IntoViewFn<V>` | `ViewFn`（任意なら `Option<ViewFn>`） |
| 子の列（`impl IntoView` の tuple、`Vec<..>`） | `Vec<View>` |
| 引数なしの flag の setter（`fn keep_mounted(mut self) -> Self`） | `#[prop(default = false)] keep_mounted: bool` |
| 値の setter で、field に既定値があった | `#[prop(default = <今の既定値>)] s: <型>`（`Value<T>` にも書ける。annex C の C09） |
| 値の setter で、field が `Option<..>` だった | `s: Option<<型>>` |
| その他の型 `T` | `T`（macro が `impl Into<T>` を受ける） |

C4. setter の名前が `Element` の method と同じもの（`disabled`、`variant`、`name`、`label`、`test_id`、`node_ref`、`style`、`role` など。annex C の `Element` の method の一覧）は props にしない（API-13）。本体がその値を要るときは、部品の引数に `#[caller] caller: Caller` を書いて読む（設計 5.1 規則 10。thread-local の `custom::caller()` は無い）。
C5. handler の名前（設計 5.3）：payload のない押下は `on_press`、key の項目の実行は `on_activate`、確定は `on_commit`、編集の段階は `on_edit`。`on_open_change` のような開閉の通知は消し、`open: Signal<bool>`（位置引数か setter。設計 5.9 の規則）にする。
C6. 見えない名前（DEC-05）：kit の部品が text field・slider・spin button・combobox を作り、今その accessible name を位置引数で取るときは、位置引数から外す。呼出し側は `.label("Name")`（`Element` の modifier）か `field("Name", control)` で名前を付ける。各 unit の表で該当する部品を挙げる。
C7. 引数のない `pub fn x() -> impl Element`・`-> Node`・`-> impl IntoView` の部品は `#[component] pub fn x() -> impl Element`。
C8. 部品でない公開の関数（`Shape` を返す game `cut_corner`、data を返す creative `color_from_hsv` など）はそのまま。各 unit の表で「関数」と書く。
C9. 部品の data の型（`ListItem` に入る record、model の struct、options の struct）はそのまま。`ThemeOptions` 以外の options の struct（creative `ToolWindowOptions`）もそのまま。
C10. 今 `xtask-allow(...)` の印が付いていた field は、部品の引数になって field ではなくなるので、印も消える。新しく `state` 検査の対象になる型（設計 6.1）を作ったときだけ、その unit が印の文面を決める。
C11. generic な builder（`<K: ItemKey>`）は generic な `#[component]` にする（`#[component] pub fn x<K: ItemKey>(..)`）。
C12. tooltip（設計 12 章 API-3、5.9）：`tooltip(label)<setter…>.attach(trigger)` は `tooltip(label, trigger)<setter…>` にする（`Tooltip::attach` は無い。setter の `.placement(..)` などは順のまま後ろに付ける）。kit と catalog の対象は `rg -n "tooltip\(" crates/kivact-ui-<kit>/src examples/kivact-ui-<kit>/src crates/kivact-ui-<kit>/tests` で探し、`.attach(` が続く呼出しを全部直す（今は basic `menubar.rs`・`chart/view.rs`・`chart/tooltip.rs`、soft `more_menu.rs`・`text.rs`・`thumbnail.rs`・`timestamp.rs`、creative `property_row.rs`、basic・creative・soft の catalog の tooltip の page。`side_navigation/` は K42 で消える）。recipe の `ToastTheme`・`TooltipTheme` の part の setter（`.tooltip(..)` の表）は別物で、U recipe 表のまま。

#### 3.0.6 段階的な compile（W9 の kit の lane）

DEC-14 で W6 から kit は gate の compile の対象の外にある（gate の `--exclude`。kit は workspace の member のまま）。W9 の kit の lane は次の順で crate を compile できる形に戻す。

S1. lane の最初の unit（K15、K20、K22、K26、K28）は root `Cargo.toml` を変えない（INC-09）。その unit は `src/lib.rs` を、その unit で変換した module の `mod` 行と `pub use` 行だけにする。まだ変換していない module の file は消さず、`mod` 行を書かない（compile されない）。
S2. 後の unit は、自分が変換した module の `mod` 行と `pub use` 行を足す。変換した module が、まだ `mod` 行の無い module の `pub(crate)` の item を要るときは、その module も同じ unit で変換して足す。
S3. lane の途中の unit の確認は `--lib` だけにし、clippy は `-A dead_code` を付ける（まだ足していない module が使う helper が dead になるため）。これは command の option で、code に `#[allow]` は書かない。
S4. lane の最後の unit（W10 の K34・K36・K38・K39・K41〜K43）は、`--all-targets` と `-D warnings` だけで通す。kit の試験（`tests/consumer`）を戻すのは各 kit の catalog の unit（K35、K37、K38、K40、K43）。それまで `tests/` は compile されない（`cargo check -p <kit> --lib` は試験を読まない）。
S5. `crates/kivact-ui-<kit>/Cargo.toml` の `[dev-dependencies]` は今のまま（`kivact` の feature `test`）。

#### 3.0.7 theme の cache と icon（KIT-08、設計 12 章 PF-02）

kit の theme は UI thread ごとに一度だけ作り、`thread_local!` の static に保って clone する（同じ mode は同じ `Arc` の theme：`PartialEq` が `Arc::ptr_eq` で真になり、入れ直しても restyle しない）。fn pointer を key にする `cached_theme` は使わない（関数の address は同一性を保証しない）。`thread_local!` の初期化は最初の参照のときに一度だけ走るので、`OnceCell` などの遅延の型は要らない。

```rust
// src/theme.rs（basic・pop・soft）
use kivact::style::{Theme, ThemeMode};

thread_local! {
    /// The kit's light theme, built once per UI thread on first use.
    static LIGHT: Theme = build(ThemeMode::Light);
    /// The kit's dark theme, built once per UI thread on first use.
    static DARK: Theme = build(ThemeMode::Dark);
}

/// The complete theme of the kit in `mode`. The same mode returns the same
/// shared theme, so installing it again never restyles.
pub fn theme(mode: ThemeMode) -> Theme {
    match mode {
        ThemeMode::Light => LIGHT.with(Theme::clone),
        ThemeMode::Dark => DARK.with(Theme::clone),
    }
}

fn build(mode: ThemeMode) -> Theme {
    let theme = Theme::light();                       // dark の kit も Theme::light() から始める（今の kit と同じ。creative だけ今のとおり mode で選ぶ）
    let theme = crate::color::set_colors(theme.with_mode(mode), mode);
    let theme = crate::size::set_sizes(theme);        // size.rs がある kit だけ
    let theme = set_core(theme, mode);                // T6・T7。theme.rs の private 関数
    let theme = theme.with_icons(crate::icons::set());
    crate::recipes_controls::install(theme)           // R5 の順に続ける
}
```

game と creative：

```rust
thread_local! {
    static LIGHT: Theme = build(ThemeMode::Light, &ThemeOptions::default());
    static DARK: Theme = build(ThemeMode::Dark, &ThemeOptions::default());
}

pub fn theme(mode: ThemeMode) -> Theme {
    match mode {
        ThemeMode::Light => LIGHT.with(Theme::clone),
        ThemeMode::Dark => DARK.with(Theme::clone),
    }
}

/// The theme of the kit in `mode` with `options`; the default options return [`theme`].
pub fn theme_with(mode: ThemeMode, options: ThemeOptions) -> Theme {
    if options == ThemeOptions::default() {
        theme(mode)
    } else {
        build(mode, &options)
    }
}
```

icon（W10 の各 kit の unit。W9 の間は今の `icons.rs` を変えない）：

```rust
//! Role: The kit's embedded icons (license), which the kit's theme puts in its icon set.
use kivact::style::{IconSet, icons};

const ICONS: [(&str, &[u8]); N] = icons!("assets/icons", ["activity", "align-center", /* 今の名前を全部 */]);

// basic と soft だけ（今 ICON_NAMES を公開している kit）。他の kit はこの 3 項目を書かない。
const NAMES: [&str; N] = names();

/// Every icon name of the kit, in the order of [`ICONS`].
pub const ICON_NAMES: &[&str] = &NAMES;

const fn names() -> [&'static str; N] {
    let mut out = [""; N];
    let mut i = 0;
    while i < N {
        out[i] = ICONS[i].0;
        i += 1;
    }
    out
}

thread_local! {
    /// The kit's icon set, built once per UI thread and shared by its light and dark themes.
    static SET: IconSet = build();
}

pub(crate) fn set() -> IconSet {
    SET.with(IconSet::clone)
}

fn build() -> IconSet {
    let mut set = IconSet::standard();
    set.extend(ICONS);
    set.extend([/* 今 set() が足していた別名の icon（basic の "dot"、"menu-radio"、"toast-*" など）をそのまま */]);
    set
}
```

`N` は `icons!` に並べた名前の数（数字の literal で書く）。並べる名前は、今その kit の theme の icon set に入っている名前（今の `set()` が `extend` していた一覧）の全部。pop と game は今 `NAMES` と `IMAGES`（game は `match` の腕）が別の一覧なので、両方にある名前だけを並べる。片方にしか無い名前は今も set に入っていないので並べない（KIT-08 の「黙って落ちる名前」を消す）。並べた名前の svg が `assets/icons/` に無いと compile が失敗するので、それで照合できる。

#### 3.0.8 試験と catalog（TL-13、KIT-12、KIT-20）

kit の試験（各 kit の catalog の unit が行う）：

1. `tests/consumer.rs` と `tests/consumer/*.rs` の file ごとに、4.6 節の表の判定（残す／消す／移す）に従う。
   - 「消す」：その file の試験が確かめる振る舞いは、`e2e_ui` の `interaction_*`・`apg_*` か、`e2e_kits` の theme_conformance の `all_kit_themes_preserve_standard_component_contracts` が同じ入力で確かめている。file を消し、`tests/consumer.rs` の `#[path]` と `mod` の 2 行を消す。
   - 「移す」：K44 が theme_conformance の共通の試験にして、K44 が元の file を消す。kit の unit は残して新しい API に直す。
   - 「残す」：kit の分野の部品か、kit の theme だけが持つ振る舞い。新しい API に直す。
2. 標準部品の配線を確かめる試験（「残す」のうち標準部品のもの）は `tests/consumer/controls.rs` 一つにまとめ、600 code 行までにする（TL-13）。分野の部品の試験は分野ごとの file のまま。
3. 期待値は変えない。新しい API で同じ入力を流し、同じことを確かめる。期待値が token の値（`Tokens::new(..).primary`）なら、同じ値を `theme.get(&PRIMARY_VAR)` で読む。
4. `xtask-allow(state-fields)` の印（W1 で消えているはず）が残っていたら消す。

catalog（KIT-12）：

```
examples/kivact-ui-<kit>/
  Cargo.toml        [dependencies] kivact、kivact-ui-<kit>、catalog-shell
                    [dev-dependencies] kivact（feature test）、catalog-shell（feature test）
                    [features] default = ["gpu", "complex-scripts"]、gpu = ["kivact/gpu"]、complex-scripts = ["kivact/complex-scripts"]（今のまま）
  src/main.rs       15 行まで（下）
  src/lib.rs        pub fn pages() -> Vec<Page>、pub fn theme(mode: ThemeMode) -> Theme、
                    pub const THEME: CatalogTheme = CatalogTheme::new(theme)、pub const START: &str
  src/pages/<slug>.rs   page ごとに一つ。中の demo は `// example: <id>` と `// end example` の間に書く
  assets/fonts/     kit だけが使う font（今のまま）
  tests/catalog.rs  catalog_shell::check の walker を呼ぶ試験＋ catalog 固有の試験
```

```rust
//! Role: Entry point of the <Kit> kit catalog app.
use kivact::prelude::*;
use kivact_ui_<kit>_catalog::{START, THEME, pages};

fn main() -> kivact::Result<()> {
    App::new("Kivact UI · <Kit>")
        .window_size(1440, 900)
        .run(move || catalog_shell::catalog("<Kit>", pages(), THEME).start(START))
}
```

`theme(mode)` は `<kit>::theme(mode).with_fonts(FONTS.iter().copied())`（設計 12 章 API-13：`with_fonts(impl IntoIterator<Item = impl Into<Asset>>)`。static の bytes をそのまま渡し、`.leak()` は書かない。`FONTS` は `catalog_shell::FONTS` か、kit の font を足した catalog の `static FONTS: &[&[u8]]`）に、S 表の font family の変数の set を足したもの。

catalog の文言（title、説明、demo の中の文字）は英語にする（KIT-12、Q4）。demo の中の `Toast::icon(<view の関数>)` は annex U の `Toast::icon_name(<その kit の icon set の名前>)` にする（`rg -n "\.icon\(\|\|" examples/kivact-ui-<kit>/src` で toast の行を探す。今あるのは basic の `docs/toast.rs` と pop の `pop/shell.rs` だけ。kit の recipe の `ToastTheme` の `icon` の part は別物で、U recipe 表のまま）。日本語の文字を含む行は 0 行にする（確認：`rg -c '[\p{Hiragana}\p{Katakana}\p{Han}]' examples/kivact-ui-<kit>` が何も出さない）。ただし catalog の font が日本語を含むことを確かめる試験（basic の `every_page_character_is_in_the_catalog_fonts`）は catalog-shell の check に移し、文字列の literal は試験の中だけに置く。

#### 3.0.9 確認のコマンド（tier a の雛形）

W9 の途中の unit（S3）：

```
cargo fmt -p kivact-ui-<kit>
cargo check -p kivact-ui-<kit> --lib
cargo clippy -p kivact-ui-<kit> --lib -- -D warnings -A dead_code
cargo xtask check --no-write --crate kivact-ui-<kit>
```

kit の最後の unit（S4）と catalog の unit：

```
cargo fmt -p kivact-ui-<kit> -p kivact-ui-<kit>-catalog
cargo check -p kivact-ui-<kit> -p kivact-ui-<kit>-catalog --all-targets
cargo clippy -p kivact-ui-<kit> -p kivact-ui-<kit>-catalog --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-ui-<kit>
cargo xtask check --no-write --crate kivact-ui-<kit>-catalog
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-<kit>
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-<kit>-catalog
```

#### 3.0.10 README の雛形（KIT-16）

````markdown
# kivact-ui-<kit>

<一文：何の見た目か。今の README の 1 段落目を短くしたもの>

## Use it

```rust
use kivact::prelude::*;
use kivact_ui_<kit> as <kit>;

fn main() -> kivact::Result<()> {
    App::new("App").theme(<kit>::theme(ThemeMode::Light)).run(|| button("Save"))
}
```

The standard components of `kivact::ui` draw in this look under the theme. The kit's own components and variants are listed in the API documentation (`cargo doc -p kivact-ui-<kit> --open`). Guides: `docs/for-kivact-ui-user/en/<kit>/`.

## License

<今の README の license の節をそのまま>
````

消すもの：「Migrating from the previous release」の節、`<!-- kivact-recipes -->` から `<!-- /kivact-recipes -->` までの一覧、部品の一覧の表、message の key の表（key は crate doc に書く）。

---

## K01：kivact-devtools が kit に依存しない（private の theme と counters の図）

- **wave / lane / 前提**：W2a / K / なし
- **所有**：`crates/kivact-devtools/Cargo.toml`、`crates/kivact-devtools/src/app.rs`、`crates/kivact-devtools/src/look.rs`（新）、`crates/kivact-devtools/src/lib.rs`（`mod look;` の 1 行）、`crates/kivact-devtools/src/panels/performance.rs`、`crates/kivact-devtools/src/widgets.rs`、`crates/kivact-devtools/src/widgets/counters_chart.rs`（新）、`Cargo.lock`
- **目的**：DevTools の GUI が basic kit の全体（chart、plot、sidebar…）を link しているのをやめる。theme は 100 行未満の private の theme、counters の折れ線は canvas で描く（KIT-18 の決定「private theme」）。W6 以後 kit は gate の compile から外れるので W2a で済ませる（DEC-14）。
- **手順**：
  1. `crates/kivact-devtools/Cargo.toml` の `[dependencies]` から `kivact-ui-basic = { path = "../kivact-ui-basic", version = "0.1.0" }` の行を消す。
  2. `crates/kivact-devtools/src/look.rs` を作る（今の API で書く。W6 の S の掃引が後で直す）：

     ```rust
     //! Role: The DevTools app's own theme: neutral colors, square corners and 12 px text.
     //!
     //! The values are the Lyra Neutral ones the app used through the Basic kit;
     //! the standard components keep their own recipes.

     use kivact::style::theme::{Palette, Radii, Spacing, Typography};
     use kivact::style::{Color, LineHeight, Theme, ThemeMode, rgb, rgba};

     /// The app's theme in `mode`.
     pub(crate) fn theme(mode: ThemeMode) -> Theme {
         let dark = mode == ThemeMode::Dark;
         let pick = |light: u32, dark_value: u32| rgb(if dark { dark_value } else { light });
         let base = Theme::light();
         let colors = Palette {
             surface: pick(0xffffff, 0x0a0a0a),
             surface_alt: pick(0xf5f5f5, 0x262626),
             text: pick(0x0a0a0a, 0xfafafa),
             text_muted: pick(0x737373, 0xa1a1a1),
             accent: pick(0x171717, 0xe5e5e5),
             accent_text: pick(0xfafafa, 0x171717),
             border: if dark { rgba(0xffffff, 0.10) } else { rgb(0xe5e5e5) },
             focus: rgba(if dark { 0x737373 } else { 0xa1a1a1 }, 0.5),
             danger: pick(0xe7000b, 0xff6467),
             success: pick(0x16a34a, 0x4ade80),
             warning: pick(0xd97706, 0xfbbf24),
             overlay: Color::BLACK.with_alpha(0.1),
             shadow: Color::BLACK.with_alpha(0.1),
             selection: rgba(if dark { 0xe5e5e5 } else { 0x171717 }, 0.15),
             hover: pick(0xf5f5f5, 0x262626),
             pressed: pick(0xf5f5f5, 0x262626).mix(pick(0x0a0a0a, 0xfafafa), 0.05),
             raised: pick(0xffffff, 0x171717),
         };
         let text = Typography {
             family: Some("Inter, sans-serif".into()),
             size: 12.0,
             size_small: 12.0,
             size_large: 14.0,
             weight: 400,
             weight_strong: 500,
             line_height: LineHeight::Scale(16.0 / 12.0),
             ..base.text().clone()
         };
         base.with_mode(mode)
             .with_colors(colors)
             .with_space(Spacing { xs: 4.0, sm: 8.0, md: 12.0, lg: 16.0, xl: 24.0 })
             .with_radius(Radii { sm: 0.0, md: 0.0, lg: 0.0 })
             .with_text(text)
             .with_focus_ring(1.0)
             .with_control_height(32.0)
     }
     ```

     `pressed` の `mix` の呼び方は、今の `crates/kivact-ui-basic/src/theme.rs` の `pressed: p.secondary.mix(p.foreground, 0.05)` と同じにする（この値は basic の今の値と同じになる）。
  3. `crates/kivact-devtools/src/lib.rs` に `mod look;` を足す（他の `mod` 行と同じ並び）。
  4. `crates/kivact-devtools/src/app.rs`：関数 `shell` の `provide_theme` の最初の closure の `let theme = kivact_ui_basic::theme(mode);` から `theme.with_text(text)` までを `crate::look::theme(mode)` の 1 式にする（`mode` を求める `match` は今のまま）。
  5. `crates/kivact-devtools/src/widgets/counters_chart.rs` を作る：

     ```rust
     //! Role: The counters history: one line per counter over the samples, on a canvas.

     use kivact::draw::{Draw, Path, Stroke};
     use kivact::geometry::{Point, Rect, Size};
     use kivact::prelude::*;
     use kivact::style::rgb;

     /// The counter names in the order of a row.
     const SERIES: [&str; 10] = [
         "Reactive", "Owners", "Objects", "Constants", "Nodes",
         "Timers", "Frame loops", "Animations", "Turns", "Blocking",
     ];
     /// One color per counter (Tableau 10), the same in both schemes.
     const COLORS: [u32; 10] = [
         0x4e79a7, 0xf28e2b, 0xe15759, 0x76b7b2, 0x59a14f,
         0xedc948, 0xb07aa1, 0xff9da7, 0x9c755f, 0xbab0ac,
     ];
     /// Room around the lines.
     const INSET: f64 = 8.0;

     /// The lines of `rows` (one row per sample, one value per counter) and their legend.
     pub(crate) fn counters_chart(rows: Value<Vec<[f64; 10]>>) -> impl IntoView {
         let legend = SERIES
             .iter()
             .zip(COLORS)
             .map(|(name, color)| {
                 row((
                     node().style(style().size(8, 8).background(rgb(color))),
                     text(*name).style(style().font_size(11)),
                 ))
                 .style(style().gap(4).align_items(Align::Center))
             })
             .collect::<Vec<_>>();
         column((
             canvas(move |draw, size| rows.with(|rows| paint(draw, size, rows)))
                 .name("Counters chart")
                 .style(style().width(pct(100)).height(180)),
             row(legend).style(style().gap(12).wrap(true)),
         ))
         .style(style().gap(6))
     }

     /// One polyline per counter, all scaled to the largest value of every counter.
     fn paint(draw: &mut Draw, size: Size, rows: &[[f64; 10]]) {
         let area = Rect::new(INSET, INSET, size.width - 2.0 * INSET, size.height - 2.0 * INSET);
         if rows.len() < 2 || area.width <= 0.0 || area.height <= 0.0 {
             return;
         }
         let max = rows.iter().flatten().copied().fold(0.0_f64, f64::max).max(1.0);
         let step = area.width / (rows.len() - 1) as f64;
         for (series, color) in COLORS.iter().enumerate() {
             let mut path = Path::new();
             for (index, values) in rows.iter().enumerate() {
                 let at = Point::new(
                     area.x + step * index as f64,
                     area.bottom() - values[series] / max * area.height,
                 );
                 path = if index == 0 { path.move_to(at) } else { path.line_to(at) };
             }
             draw.stroke_path(&path, &Stroke::new(1.5), rgb(*color));
         }
     }
     ```

  6. `crates/kivact-devtools/src/widgets.rs` に `pub(crate) mod counters_chart;` を足す（名前の順）。
  7. `crates/kivact-devtools/src/panels/performance.rs`：
     - `use kivact_ui_basic as basic;` を消し、`use crate::widgets::counters_chart::counters_chart as counters_lines;` を足す。
     - 関数 `counters_chart(session, counters)` の本体で、`let config = basic::ChartConfig::new([..]);` の文を消す。`let rows = memo(..)` の closure の `.map(|(sample, counters)| { basic::ChartRow::new(format!("#{sample}"), [ ..10 個の式.. ]) })` を `.map(|(_, counters)| [ ..同じ 10 個の式を同じ順で.. ])` にする。
     - `basic::chart(config, rows).kind(basic::ChartKind::Line).height(180),` を `counters_lines(rows),` にする。
  8. `cargo metadata --format-version 1 > /dev/null` で `Cargo.lock` を直す。
- **消すもの**：`kivact-devtools` の `kivact-ui-basic` への依存、`performance.rs` の `basic::` の参照すべて。
- **状態の閉じ方**：新しい状態の型なし。
- **試験**：足さない。`crates/kivact-devtools/tests/app/panels/performance.rs` の既存の試験が今の期待値で通る。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-devtools
  cargo check -p kivact-devtools --all-targets
  cargo clippy -p kivact-devtools --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-devtools
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-devtools
  ```

- **完了条件**：`rg -n "kivact-ui-basic|kivact_ui_basic" crates/kivact-devtools` が何も出さない。`cargo tree -p kivact-devtools -e normal | rg kivact-ui-` が何も出さない。上の command がすべて通る。

## K02：soft の mobile drawer の Machine が呼出し側の open を写さない

- **wave / lane / 前提**：W2a / K / K01
- **所有**：`crates/kivact-ui-soft/src/mobile_nav.rs`
- **目的**：「drawer が開いているか」を持つのを呼出し側の binding 一つにする。Machine は「mobile の layout か」だけを持ち、mobile を出るときに drawer を閉じる効果だけを出す。`Sync`・`SetOpen`・`Toggle` の event と `MobileMachine { open }` を消す（KIT-01）。
- **手順**（今の API。`Binding` は W5 で `Signal` になるが、この unit は今の名前で書く）：
  1. `enum AppShellMobileState` を次にする（名前を `Layout` に変える）：

     ```rust
     /// Whether the shell uses its mobile layout.
     #[derive(Clone, Copy, PartialEq, Eq)]
     enum Layout {
         Desktop,
         Mobile,
     }
     ```

  2. `enum AppShellMobileEvent`、`struct MobileMachine`、`impl Machine for MobileMachine` を消し、次を置く：

     ```rust
     /// The layout follows the viewport; leaving the mobile layout closes the drawer.
     struct LayoutMachine;

     impl Machine for LayoutMachine {
         type State = Layout;
         /// Whether the viewport is below the mobile breakpoint.
         type Event = bool;
         /// Close the drawer.
         type Effect = ();
         fn transition(&self, state: &Layout, mobile: bool) -> Step<Layout, bool, ()> {
             match (state, mobile) {
                 (Layout::Desktop, true) => Step::to(Layout::Mobile),
                 (Layout::Mobile, false) => Step::to(Layout::Desktop).with(()),
                 _ => Step::stay(),
             }
         }
     }
     ```

  3. `struct MobileContext` と `impl MobileContext` を次にする（`auto_toggle` の上の `xtask-allow(state-fields)` の行は W1 で消えている。残っていれば消す）：

     ```rust
     // xtask-allow(state): open is the caller's binding and the only owner of the drawer's openness; layout is the viewport's machine; auto_toggle is fixed at construction.
     #[derive(Clone, Copy)]
     struct MobileContext {
         layout: MachineHandle<LayoutMachine>,
         open: Binding<bool>,
         toggle: NodeRef,
         drawer: NodeRef,
         auto_toggle: bool,
     }
     impl MobileContext {
         fn is_mobile(self) -> bool {
             self.layout.get() == Layout::Mobile
         }
         fn is_open(self) -> bool {
             self.is_mobile() && self.open.get()
         }
         /// Closing always applies; opening only in the mobile layout.
         fn set_open(self, open: bool) {
             if !open || self.is_mobile() {
                 self.open.set(open);
             }
         }
     }
     ```

  4. `AppShellMobile::configured` の `let initial = ..` から `state.feed(move || open.get(), AppShellMobileEvent::Sync);` までを次にし、続く `Self { context: Some(MobileContext { state, .. }) }` の `state,` を `layout, open,` にする：

     ```rust
     let initial = if mobile.is_some_and(|mobile| mobile.get_untracked()) {
         Layout::Mobile
     } else {
         Layout::Desktop
     };
     let layout = machine(initial, || LayoutMachine, move |_, ()| {
         if open.get_untracked() {
             open.set(false);
         }
     });
     if let Some(mobile) = mobile {
         layout.feed(mobile, |mobile| mobile);
     }
     ```

  5. `AppShellMobile::toggle` の本体を次にする：

     ```rust
     if let Some(context) = self.context
         && context.is_mobile()
     {
         context.set_open(!context.open.get_untracked());
     }
     ```

     `show`、`hide`、`open_binding` は `context.set_open(..)` を呼ぶ今の形のまま。
  6. file の他の `context.state` の読み（`rg -n "\.state\b" crates/kivact-ui-soft/src/mobile_nav.rs`）を `context.layout` か `is_mobile()`・`is_open()` にする。使わなくなった `use` を消す。
- **消すもの**：`AppShellMobileState`、`AppShellMobileEvent`、`MobileMachine`、`Sync` の `feed`。
- **状態の閉じ方**：`MobileContext` は Signal 系の field（`open`）と bool（`auto_toggle`）を持ち、`set_open` が `open` を書くので `state` 検査の対象。手順 3 の 1 行の印で閉じる。`Layout` は 2 値の enum。
- **試験**：足さない。`crates/kivact-ui-soft/tests/consumer/navigation.rs` の `shell_adapts_navigation_and_opens_the_mobile_drawer` が今の期待値で通る。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-ui-soft
  cargo check -p kivact-ui-soft --all-targets
  cargo clippy -p kivact-ui-soft --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-ui-soft
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft --test consumer -- navigation
  ```

- **完了条件**：`rg -n "AppShellMobileEvent|MobileMachine|::Sync" crates/kivact-ui-soft/src/mobile_nav.rs` が何も出さない。上の command が通り、filter `navigation` が 1 件以上を実行する（0 件は失敗）。

## K03：creative の effect を Machine の入力にする（`observe_inputs` の 4 つの写し）

- **wave / lane / 前提**：W2a / K / K02
- **所有**：`crates/kivact-ui-creative/src/` の `editors/shared/gesture.rs`、`editors/shared/targets.rs`、`editors/time/view.rs`、`editors/time/input.rs`、`editors/arrangement/view.rs`、`editors/arrangement/input.rs`、`editors/graph/view.rs`、`editors/graph/input.rs`、`editors/node/view.rs`、`editors/node/input.rs`、`editors/node/gesture.rs`、`picker.rs`、`audio/meter.rs`
- **目的**：effect で gesture の状態を補修するのをやめる（KIT-02）。focus・表示・有効は memo にして Machine に `feed` する。drag 中の item が data から消えたかは、drag の event のたびに gesture が調べ、消えていたら取り消す。picker・meter・targets の「effect で machine に send」は `feed` にする。目標：`rg -n "effect\(" crates/kivact-ui-creative/src` が `editors/shared/ruler.rs`（入力の誤りを報告するだけで状態を書かない）の 1 か所だけになる。
- **手順**：
  1. `editors/shared/gesture.rs`：
     - `enum DecisionEvent` の `InputsChanged { focused, visible, enabled, data_valid, rollback }` を `Inputs { ok: bool, rollback: Rc<dyn Fn()> }` にする。
     - `DecisionMachine::transition` の `InputsChanged` の腕を次にする：

       ```rust
       DecisionEvent::Inputs { ok, rollback } => {
           if ok {
               Step::stay()
           } else {
               Step::to(Press::Idle).with(rollback)
           }
       }
       ```

     - `CanvasGesture::inputs_changed` を消し、次を置く：

       ```rust
       /// Cancel the press whenever `ok` turns false (focus left, hidden or disabled).
       pub(crate) fn watch_inputs(self, ok: Value<bool>, rollback: impl Fn() + 'static) {
           let rollback: Rc<dyn Fn()> = Rc::new(rollback);
           self.controller.feed(ok, move |ok| DecisionEvent::Inputs {
               ok,
               rollback: rollback.clone(),
           });
       }
       ```

     - `CanvasGesture::install` に最後の引数 `stale: impl Fn() -> bool + 'static` を足し、`let hooks = Rc::new((marquee, selection, callbacks));` を `let hooks = Rc::new((marquee, selection, callbacks, stale));` にする。`on_update` の closure の先頭に `if (updates.3)() { self.cancel(|| updates.2.3(false)); return; }`、`on_end` の closure の先頭に `if update.phase == DragPhase::End && (hooks.3)() { self.cancel(|| hooks.2.3(false)); return; }` を足す。`finish` に渡す `&hooks.0, &hooks.1, &hooks.2` は今のまま。
     - file に新しい関数を足す：

       ```rust
       /// Whether a canvas editor takes input: focused inside, visible and enabled.
       pub(crate) fn inputs_ok(
           reference: kivact::view::NodeRef,
           availability: Value<kivact::custom::PartState>,
       ) -> Value<bool> {
           let focus = reference.focus_within();
           kivact::state::memo(move || {
               let input = availability.get();
               focus.get() && reference.is_visible() && !input.is_hidden() && !input.is_disabled()
           })
       }
       ```

  2. `time`、`arrangement`、`graph` の `view.rs`：`fn observe_inputs` を消し、その呼出し（`observe_inputs(state, reference, availability.state());`）を次にする：

     ```rust
     state.press.watch_inputs(
         crate::editors::shared::gesture::inputs_ok(reference, availability.state()),
         move || state.cancel(),
     );
     ```

  3. 同じ 3 つの editor の `install(..)` の呼出し（`time/input.rs` の `s.press.install(`、`arrangement/input.rs` の `state.press.install(`、`graph/input.rs` の `self.press.install(`）に、最後の引数 `stale` を足す。中身は手順 2 で消した `observe_inputs` の `missing` の式をそのまま写し、`.get()` を `.get_untracked()` にしたもの。time の例：

     ```rust
     move || {
         let items = state.data.get_untracked();
         let lanes = state.visible.get_untracked();
         state.initial().is_some_and(|snapshot| match snapshot {
             crate::editors::time::gesture::Snapshot::Items(initial) => initial.iter().any(|v| {
                 !items
                     .iter()
                     .any(|i| i.id() == v.0 && lanes.iter().any(|lane| lane.id == i.lane()))
             }),
             _ => false,
         })
     }
     ```

     arrangement は `state.data`・`state.track_data` と `super::gesture::Snapshot::{Clips, Automation}` の今の 3 つの腕、graph は `state.drag_state()` の `DragKind::{Keys, Tangent}` の今の腕を同じ形で写す（graph の `if let Some(gesture) = gesture` の外は `false`）。
  4. node：
     - `node/view.rs` の `fn observe_inputs` を消す。代わりに `pub(super) fn stale(state: State) -> bool` を作る。本体は今の `observe_inputs` の `slot_valid`、`connection_valid`、`drag_valid` の 3 つの式を写し（`state.nodes.get()` を `state.nodes.get_untracked()` に、`with_untracked`・`untrack` はそのまま）、`!(slot_valid && connection_valid && drag_valid)` を返す。
     - `observe_inputs(state);` の呼出しを次にする：

       ```rust
       let focus = state.reference.focus_within();
       let ok = kivact::state::memo(move || focus.get() && !state.blocked.get());
       state.press.watch_inputs(ok, move || state.cancel());
       ```

     - `node/gesture.rs` の `pub(crate) fn inputs(..)`（`state.mode` の入力）を呼んでいたのは消した `observe_inputs` だけなので、`inputs` とその event を消す（`rg -n "\.inputs\(" crates/kivact-ui-creative/src/editors/node` が他を出さないことを確かめる）。focus を失ったときに接続の mode を終える動き（試験 `a_keyboard_connection_ends_when_the_editor_loses_focus`）は、`state.cancel()` が mode を `Idle` に戻すことで保つ。`state.cancel()` が mode を戻さないときは、`cancel` の本体の最初に今の `inputs` が focus を失ったときに行っていた遷移の event を mode の machine に送る文を足す。
     - `node/input.rs` の `state.press.install(` の最後の引数に `move || super::view::stale(state)` を足す。key の handler で `NodeMode::Connecting` を扱う関数（`rg -n "Connecting" crates/kivact-ui-creative/src/editors/node/input.rs` の最初の関数）の先頭に `if super::view::stale(state) { state.cancel(); return Handled::Yes; }` を足す。
  5. `editors/shared/targets.rs` の関数 `targets`：`effect(move || controller.send(TargetEvent::Synchronize(source())));` を次にする：

     ```rust
     let source = kivact::state::memo(source);
     controller.send(TargetEvent::Synchronize(source.get_untracked()));
     controller.feed(source, TargetEvent::Synchronize);
     ```

  6. `picker.rs`：
     - `fn follow_active` の本体を次にする：

       ```rust
       let preview = move |key: Option<K>| {
           key.and_then(|key| {
               lists
                   .with_untracked(|all| {
                       all.iter()
                           .position(|list| list.iter().any(|item| item.key == key))
                   })
                   .map(|index| (index, key))
           })
       };
       columns.send(ColumnEvent::Preview(preview(active.get_untracked())));
       columns.feed(active, move |key| ColumnEvent::Preview(preview(key)));
       ```

     - `fn follow_columns` の `if multiple { .. }` の中を次にする：

       ```rust
       let by_query = move |query: String| {
           ColumnEvent::QueryChanged(
               lists.with_untracked(|all| column_availability(all, &query.trim().to_lowercase())),
           )
       };
       let by_items = move |all: Vec<Vec<PickerItem<K>>>| {
           ColumnEvent::ItemsChanged(column_availability(
               &all,
               &query.get_untracked().trim().to_lowercase(),
           ))
       };
       columns.send(by_query(query.get_untracked()));
       columns.send(by_items(lists.get_untracked()));
       columns.feed(query, by_query);
       columns.feed(lists, by_items);
       ```

       `feed` が `PickerItem<K>` の `PartialEq` を要るときは、`PickerItem` の derive に `PartialEq` を足す。
  7. `audio/meter.rs` の `impl IntoView for Meter` の `effect(move || { clip.send(ClipEvent::LevelChanged(level.get())); });` を次にする：

     ```rust
     clip.send(ClipEvent::LevelChanged(level.get_untracked()));
     clip.feed(level, ClipEvent::LevelChanged);
     ```

  8. 使わなくなった `use`（`effect`、`untrack` など）を消す。
- **消すもの**：4 つの `fn observe_inputs`、`DecisionEvent::InputsChanged`、`CanvasGesture::inputs_changed`、node の `gesture.rs` の `inputs`、`picker.rs` の 3 つの `effect(`、`targets.rs`・`meter.rs` の `effect(`。
- **状態の閉じ方**：新しい状態の型なし（`DecisionEvent::Inputs` は event）。
- **試験**：足さない。`crates/kivact-ui-creative/tests/consumer/` の `editor_gestures.rs`、`arrangement_edit.rs`、`graph_domain.rs`、`node_editor.rs`（`a_keyboard_connection_ends_when_the_editor_loses_focus` を含む）、`picker.rs`、`advanced.rs` が今の期待値で通る。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-ui-creative
  cargo check -p kivact-ui-creative --all-targets
  cargo clippy -p kivact-ui-creative --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-ui-creative
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-creative
  ```

- **完了条件**：`rg -n "effect\(" crates/kivact-ui-creative/src` が `editors/shared/ruler.rs` の 1 行だけを出す。`rg -n "observe_inputs|InputsChanged|inputs_changed" crates/kivact-ui-creative/src` が何も出さない。上の command が通る。

## K04：game の halo の Machine をやめる

- **wave / lane / 前提**：W2a / K / K03
- **所有**：`crates/kivact-ui-game/src/surface.rs`
- **目的**：`transition` が `if running {Running} else {Stopped}` で、Effect が状態と同じ値の Machine（signal と同じもの）を消す（KIT-11 の halo の部分。kpi の部分は K39）。
- **手順**：
  1. `enum HaloMotionPhase`、`struct HaloMotionInput`、`impl Machine for HaloMotionInput` を消す。
  2. 関数 `halo` の `let motion = machine(..);` の文と、`motion.send(running.get_untracked());`、`motion.feed(running, |running| running);` の 2 文を消し、`running` の memo の後に次を置く：

     ```rust
     let run = move |running: bool| {
         if running {
             clock.start(FrameRate::Hz(30.));
         } else {
             clock.stop();
         }
     };
     run(running.get_untracked());
     kivact::state::effect_on(running, move |running| run(*running));
     ```

  3. `use kivact::state::{Machine, Step, machine, memo, signal};` を `use kivact::state::{memo, signal};` にする。
- **消すもの**：`HaloMotionPhase`、`HaloMotionInput`。
- **状態の閉じ方**：なし。
- **試験**：足さない。`crates/kivact-ui-game/tests/consumer/theme.rs` の `consumer_theme_density_and_halo_pixels_follow_public_theme` が通る。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-ui-game
  cargo check -p kivact-ui-game --all-targets
  cargo clippy -p kivact-ui-game --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-ui-game
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-game
  ```

- **完了条件**：`rg -n "HaloMotion|impl Machine" crates/kivact-ui-game/src/surface.rs` が何も出さない。上の command が通る。

## K05：soft timestamp の日付の計算を kivact::i18n にまとめる

- **wave / lane / 前提**：W2a / K / K04
- **所有**：`crates/kivact-ui-soft/src/timestamp.rs`
- **目的**：同じ暦の計算の 3 つ目の写しを消し、`kivact::i18n::CalendarDate` の `to_days`・`from_days` を使う（KIT-03 の (1) の内部の部分）。`Rc<RefCell<TimestampSettings>>` は K42 の `#[component]` 化で部品の引数になって消える。月・曜日の名前の i18n 化と `copyable` の削除は表示と公開の API を変えるので K42。
- **手順**：
  1. `use kivact::i18n::CalendarDate;` を足す。
  2. `TimestampValue::parse_iso` の `let days = days_from_civil(year, month, day);` と、続く `if civil_from_days(days) != (year, month, day) { .. }` の文を次にする（`year`・`month`・`day` は今の局所変数）：

     ```rust
     let date = CalendarDate::checked(
         i32::try_from(year).ok()?,
         u8::try_from(month).ok()?,
         u8::try_from(day).ok()?,
     )?;
     let days = date.to_days();
     ```

  3. `calendar_text` の `let (year, month, day) = civil_from_days(days);` を次にする：

     ```rust
     let date = CalendarDate::from_days(days);
     let (year, month, day) = (
         i64::from(date.year()),
         i64::from(date.month()),
         i64::from(date.day()),
     );
     ```

  4. `fn days_from_civil` と `fn civil_from_days` を消す。
- **消すもの**：soft の `days_from_civil`、`civil_from_days`。
- **状態の閉じ方**：なし。
- **試験**：足さない。soft の試験が今の期待値で通る。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-ui-soft
  cargo check -p kivact-ui-soft --all-targets
  cargo clippy -p kivact-ui-soft --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-ui-soft
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft
  ```

- **完了条件**：`rg -n "days_from_civil|civil_from_days" crates/kivact-ui-soft` が何も出さない。上の command が通る。

---

## K06：新しい 2 つの crate の枠と workspace の配線

- **wave / lane / 前提**：W9 / K6 / W8 の完了（CP2）
- **所有**：root `Cargo.toml`（`members` の 2 行）、`crates/kivact-plot/Cargo.toml`（新）、`crates/kivact-plot/src/lib.rs`（新）、`examples/catalog-shell/Cargo.toml`（新）、`examples/catalog-shell/src/lib.rs`（新）、`dev/xtask/policy/workspace.toml`、`Cargo.lock`
- **目的**：`kivact-plot`（KIT-15）と `catalog-shell`（KIT-12）を workspace に置き、policy に載せる。中身は K10・K14 が入れる。
- **手順**：
  1. root `Cargo.toml` の `members` の `"crates/kivact-ui-soft",` の次の行に `"crates/kivact-plot",`、`"examples/kivact-ui-soft",` の次の行（kit の catalog の行の後）に `"examples/catalog-shell",` を足す。
  2. `crates/kivact-plot/Cargo.toml`：

     ```toml
     [package]
     name = "kivact-plot"
     authors.workspace = true
     version.workspace = true
     edition.workspace = true
     license.workspace = true
     repository.workspace = true
     description = "Interactive XY plots and heatmaps for Kivact, colored by theme variables and exportable to PNG and SVG"

     [dependencies]
     kivact = { path = "../kivact", version = "0.1.0", default-features = false }

     [dev-dependencies]
     kivact = { path = "../kivact", default-features = false, features = ["test"] }

     [lints]
     workspace = true
     ```

  3. `crates/kivact-plot/src/lib.rs`：1 行 `//! Role: Interactive XY plots and heatmaps on the public Kivact facade.` だけ。
  4. `examples/catalog-shell/Cargo.toml`：

     ```toml
     [package]
     name = "catalog-shell"
     publish = false
     version.workspace = true
     edition.workspace = true
     license.workspace = true
     description = "The shared shell, page model and page walker of the five Kivact UI kit catalogs"

     [features]
     # The page walker of the catalogs' tests (catalog_shell::check).
     test = ["kivact/test"]

     [dependencies]
     kivact = { path = "../../crates/kivact", default-features = false }

     [lints]
     workspace = true
     ```

  5. `examples/catalog-shell/src/lib.rs`：1 行 `//! Role: The shared shell of the Kivact UI kit catalogs: page list, search, theme toggle and pages.` だけ。
  6. `dev/xtask/policy/workspace.toml`：
     - `[visible_to]` の `kivact-ui-soft-catalog = []` の行の次に次の 2 行を足す：

       ```toml
       catalog-shell = ["kivact-ui-basic-catalog", "kivact-ui-pop-catalog", "kivact-ui-game-catalog", "kivact-ui-creative-catalog", "kivact-ui-soft-catalog"]
       kivact-plot = ["kivact-ui-basic", "kivact-ui-basic-catalog", "kivact-docs"]
       ```

     - `api_packages` の配列の `"kivact-ui-basic", ..` の行に `"kivact-plot",` を足す。
     - `[[style.exceptions]]` の `path = "examples/kivact-ui-basic/src/docs.rs"`、`item = "docs::PAGES"` の塊の `item` を `"docs::pages"` にする（K35 が static の `PAGES` を関数 `pages()` にする。reason は「Glue: the page registry only lists each child page module's `page()` in navigation order.」のまま）。
     - `[[style.exceptions]]` の `path = "crates/kivact-ui-basic/tests/*"` の塊の次に、同じ形の塊を足す：`rule = "glob-imports"`、`path = "crates/kivact-plot/tests/*"`、`reason = "Plot E2E tests are written from the user's side and import `kivact::prelude::*`."`。
  7. `cargo metadata --format-version 1 > /dev/null`。
- **消すもの**：なし。
- **状態の閉じ方**：なし。
- **試験**：なし。
- **確認（tier a）**：

  ```
  cargo check -p kivact-plot -p catalog-shell
  cargo xtask check --no-write --crate kivact-plot
  cargo xtask check --no-write --crate catalog-shell
  ```

- **完了条件**：`cargo metadata --format-version 1 --no-deps | rg -c '"name":"(kivact-plot|catalog-shell)"'` が 2。上の command が通る。

## K07：`icons!`（kivact-style）

- **wave / lane / 前提**：W9 / K6 / K06
- **所有**：`crates/kivact-style/src/icons_macro.rs`（新）、`crates/kivact-style/src/lib.rs`（`mod` の 1 行）、kivact-style の facade（今は `crates/kivact-style/src/facade.rs`。W6 で `kivact::style` の名前を出す file が変わっていればその file）
- **目的**：名前を二度書く icon の一覧をやめる（KIT-08）。icon set の共有は各 kit の `icons.rs` の `thread_local!` の static で行う（3.0.7。設計 12 章 PF-02：fn pointer を key にする共有の表は作らない。KIT-08 の `IconSet::shared` は作らない）。
- **手順**：
  1. `crates/kivact-style/src/icons_macro.rs`：

     ```rust
     //! Role: The `icons!` list: each icon name written once, its svg embedded from a directory.

     /// `[(name, bytes); N]` of the svg files `<dir>/<name>.svg`, with `<dir>`
     /// relative to the calling crate's `Cargo.toml`:
     /// `icons!("assets/icons", ["home", "bell"])`.
     #[macro_export]
     macro_rules! icons {
         ($dir:literal, [$($name:literal),* $(,)?]) => {
             [$((
                 $name,
                 include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $dir, "/", $name, ".svg"))
                     as &'static [u8],
             )),*]
         };
     }
     ```

  2. `crates/kivact-style/src/lib.rs` に `mod icons_macro;` を足す（`mod icon;` の次）。
  3. facade に `pub use crate::icons;` を足す（`IconSet` を出している行の近く）。
- **消すもの**：なし（kit の `OnceLock` は各 kit の W10 の unit が `thread_local!` にして消す）。
- **状態の閉じ方**：新しい型なし。
- **試験**：足さない（`icons!` は W10 の各 kit の `icons.rs` が使い、名前の svg が無ければ compile が失敗する）。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-style
  cargo check -p kivact-style --all-targets
  cargo clippy -p kivact-style --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-style
  ```

- **完了条件**：上の command が通る。`rg -n "macro_rules! icons" crates/kivact-style/src` が 1 行。

## K08：`Slant`・`Striped` と `SyntaxKind`・構文色の theme 変数（kivact-ui）

- **wave / lane / 前提**：W9 / K6 / K07
- **所有**：`crates/kivact-ui/src/variant_marks.rs`（新。annex U 4.4 節の部品の module 表）、`crates/kivact-ui/src/syntax.rs`（新）、`crates/kivact-ui/src/syntax_colors.rs`（新）、`crates/kivact-ui/src/lib.rs`、`crates/kivact-ui/src/theme.rs`（`kivact::ui::theme`）
- **目的**：pop と game の同じ marker を中核の汎用の印にする（KIT-04）。basic と soft の二つの構文の分類と色の組を、中核の `SyntaxKind` と theme 変数の色にまとめる（KIT-05。中核は分類と色だけ、tokenizer は kit）。
- **手順**：
  1. `crates/kivact-ui/src/variant_marks.rs`（新。`//! Role: Generic variant marks any theme may honour: slanted and striped components.`）に書く（`Variant` は `kivact_core` の定義を annex U が使う path。他の部品の root の file の `impl Variant for ButtonVariant` と同じ `use`）。`lib.rs` に `mod variant_marks;` と `pub use variant_marks::{Slant, Striped};`：

     ```rust
     /// Draw a standard component slanted: `button("Go").variant(Slant)`.
     /// A theme without a slanted look draws the plain component.
     #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
     pub struct Slant;

     impl Variant for Slant {}

     /// Draw diagonal stripes on a standard component's fill:
     /// `progress(value).variant(Striped)`. A theme without stripes draws the
     /// plain component.
     #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
     pub struct Striped;

     impl Variant for Striped {}
     ```

  2. `crates/kivact-ui/src/syntax_colors.rs`（構文色の theme 変数。名前は `ui.syntax.<kind>`）：

     ```rust
     //! Role: The theme colors of source tokens, one variable per syntax kind.

     use kivact_core::style::{Color, ThemeVar, color, rgb};

     macro_rules! syntax_color {
         ($var:ident, $name:ident, $init:expr, $doc:literal) => {
             #[doc = $doc]
             pub static $var: ThemeVar<Color> = $init;
             #[doc = $doc]
             pub const $name: Color = Color::var(&$var);
         };
     }

     syntax_color!(KEYWORD_VAR, KEYWORD, ThemeVar::new("ui.syntax.keyword", rgb(0xcf222e)), "Keywords (`fn`, `let`).");
     syntax_color!(TYPE_VAR, TYPE, ThemeVar::new("ui.syntax.type", rgb(0x953800)), "Type names.");
     syntax_color!(STRING_VAR, STRING, ThemeVar::new("ui.syntax.string", rgb(0x0a3069)), "String and character literals.");
     syntax_color!(COMMENT_VAR, COMMENT, ThemeVar::new("ui.syntax.comment", rgb(0x6e7781)), "Comments.");
     syntax_color!(OPERATOR_VAR, OPERATOR, ThemeVar::alias("ui.syntax.operator", &color::TEXT_VAR), "Operators.");
     syntax_color!(PUNCTUATION_VAR, PUNCTUATION, ThemeVar::alias("ui.syntax.punctuation", &color::TEXT_VAR), "Punctuation.");
     syntax_color!(NUMBER_VAR, NUMBER, ThemeVar::new("ui.syntax.number", rgb(0x0550ae)), "Numeric literals.");
     syntax_color!(CONSTANT_VAR, CONSTANT, ThemeVar::new("ui.syntax.constant", rgb(0x0550ae)), "Constants and literals other than numbers.");
     syntax_color!(FUNCTION_VAR, FUNCTION, ThemeVar::new("ui.syntax.function", rgb(0x8250df)), "Called or defined functions.");
     syntax_color!(VARIABLE_VAR, VARIABLE, ThemeVar::alias("ui.syntax.variable", &color::TEXT_VAR), "Variables.");
     syntax_color!(TAG_VAR, TAG, ThemeVar::new("ui.syntax.tag", rgb(0x116329)), "Markup tags.");
     syntax_color!(ATTRIBUTE_VAR, ATTRIBUTE, ThemeVar::new("ui.syntax.attribute", rgb(0x6639ba)), "Attributes.");
     syntax_color!(PROPERTY_VAR, PROPERTY, ThemeVar::alias("ui.syntax.property", &color::TEXT_VAR), "Properties and fields.");
     syntax_color!(MACRO_VAR, MACRO, ThemeVar::new("ui.syntax.macro", rgb(0x0550ae)), "Macro invocations.");
     syntax_color!(LIFETIME_VAR, LIFETIME, ThemeVar::new("ui.syntax.lifetime", rgb(0x116329)), "Lifetimes and labels.");
     ```

     `color::TEXT` は S 表の「文字の色」の変数（`Palette::text` の行）。別名の fallback は `ThemeVar::alias(..)`（4.2 節の 4）。`kivact_core::style` の path は kivact-ui の他の file と同じ書き方にする。
  3. `crates/kivact-ui/src/syntax.rs`：

     ```rust
     //! Role: The kinds of source tokens a code block colors, and the ranges they paint.

     use crate::syntax_colors as colors;
     use kivact_core::style::{Color, StyledRange};
     use std::ops::Range;

     /// What a source token is. A kit's tokenizer returns these; the theme
     /// colors each kind through `kivact::ui::theme::syntax`.
     #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
     pub enum SyntaxKind {
         Keyword,
         Type,
         String,
         Comment,
         Operator,
         Punctuation,
         Number,
         Constant,
         Function,
         Variable,
         Tag,
         Attribute,
         Property,
         Macro,
         Lifetime,
     }

     /// A tokenizer of one source line: byte ranges of `line` and their kinds.
     pub type Tokenizer = fn(&str) -> Vec<(Range<usize>, SyntaxKind)>;

     impl SyntaxKind {
         /// The theme color of the kind.
         pub const fn color(self) -> Color {
             match self {
                 Self::Keyword => colors::KEYWORD,
                 Self::Type => colors::TYPE,
                 Self::String => colors::STRING,
                 Self::Comment => colors::COMMENT,
                 Self::Operator => colors::OPERATOR,
                 Self::Punctuation => colors::PUNCTUATION,
                 Self::Number => colors::NUMBER,
                 Self::Constant => colors::CONSTANT,
                 Self::Function => colors::FUNCTION,
                 Self::Variable => colors::VARIABLE,
                 Self::Tag => colors::TAG,
                 Self::Attribute => colors::ATTRIBUTE,
                 Self::Property => colors::PROPERTY,
                 Self::Macro => colors::MACRO,
                 Self::Lifetime => colors::LIFETIME,
             }
         }
     }

     /// The colored ranges of `line` for `code_block(..).highlight(..)`:
     /// tokens in order, each inside the line and after the previous one;
     /// a token that overlaps the previous one or leaves the line is dropped.
     pub fn syntax_ranges(line: &str, tokens: &[(Range<usize>, SyntaxKind)]) -> Vec<StyledRange> {
         let mut offset = 0;
         tokens
             .iter()
             .filter_map(|(range, kind)| {
                 if range.start < offset || range.is_empty() || line.get(range.clone()).is_none() {
                     return None;
                 }
                 offset = range.end;
                 Some(StyledRange {
                     range: range.clone(),
                     color: Some(kind.color()),
                     background: None,
                     decoration: None,
                 })
             })
             .collect()
     }

     #[cfg(test)]
     mod tests {
         //! Role: Tests for the syntax ranges.

         use super::*;

         #[test]
         fn overlapping_and_outside_tokens_are_dropped() {
             let ranges = syntax_ranges(
                 "let x",
                 &[(0..3, SyntaxKind::Keyword), (2..4, SyntaxKind::Variable), (4..9, SyntaxKind::Variable), (4..5, SyntaxKind::Variable)],
             );
             assert_eq!(ranges.iter().map(|r| r.range.clone()).collect::<Vec<_>>(), [0..3, 4..5]);
         }
     }
     ```

     `StyledRange` の field が W6 で変わっていたら、`crates/kivact-ui/src/code_block` の module が `StyledRange` を作る書き方に合わせる。
  4. `crates/kivact-ui/src/lib.rs` に `mod syntax;` と `mod syntax_colors;` を足し、`pub use syntax::{SyntaxKind, Tokenizer, syntax_ranges};` と、手順 1 の `pub use` と合わせて `kivact::ui::{Slant, Striped, SyntaxKind, Tokenizer, syntax_ranges}` が出る。
  5. `crates/kivact-ui/src/theme.rs`（`kivact::ui::theme`）に `pub use crate::syntax_colors as syntax;` を足す（`kivact::ui::theme::syntax::KEYWORD` など）。
- **消すもの**：なし（kit の写しは W10 の kit の unit が消す）。
- **状態の閉じ方**：なし。
- **試験**：`crates/kivact-ui/src/syntax.rs` の `overlapping_and_outside_tokens_are_dropped`（上）。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-ui
  cargo check -p kivact-ui --all-targets
  cargo clippy -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-ui
  cargo test -p kivact-ui --lib syntax
  ```

- **完了条件**：`rg -n "pub struct (Slant|Striped)\b|pub enum SyntaxKind" crates/kivact-ui/src` が 3 行。上の command が通る。

## K09：`NodeRef::once_visible`（kivact-core）

- **wave / lane / 前提**：W9 / K6 / K08
- **所有**：`crates/kivact-core/src/view/node_ref_visibility.rs`（新）、`crates/kivact-core/src/view.rs`（`mod` の 1 行）、`dev/tests/kivact-public-api/tests/e2e_core/node_ref.rs`
- **目的**：kpi の「一度見えたら真」の latch を中核に一度だけ実装する（KIT-11）。時計（KIT-17 の `task::clock`）はこの unit で作らない：付属書 R の R23 が W5a で `kivact::task::clock` を作った（INC-20）。K42 がそれを使う。
- **手順**：
  1. `crates/kivact-core/src/view/node_ref_visibility.rs`（import は kivact-core の他の file が `machine`・`memo` を使う書き方に合わせる。`rg -n "use kivact_runtime::.*machine" crates/kivact-core/src | head -3`）：

     ```rust
     //! Role: A latch that turns true once a node has been inside its window enough.

     use super::NodeRef;
     use kivact_runtime::{Machine, Step, Value, machine, memo};

     /// Whether the node has been seen.
     #[derive(Clone, Copy, PartialEq, Eq)]
     enum Seen {
         Waiting,
         Seen,
     }

     struct Latch;

     impl Machine for Latch {
         type State = Seen;
         /// Whether the node is visible enough now.
         type Event = bool;
         type Effect = ();
         fn transition(&self, state: &Seen, visible: bool) -> Step<Seen, bool, ()> {
             match state {
                 Seen::Waiting if visible => Step::to(Seen::Seen),
                 _ => Step::stay(),
             }
         }
     }

     impl NodeRef {
         /// Become true once at least `fraction` (0 to 1) of the bound node's
         /// height is inside its window, and stay true: a reveal-on-scroll latch.
         #[track_caller]
         pub fn once_visible(&self, fraction: f64) -> Value<bool> {
             let rect = self.rect();
             let window = crate::platform::window_size();
             let visible = memo(move || {
                 rect.get().is_some_and(|rect| {
                     let shown = (rect.y + rect.height).min(window.get().height) - rect.y.max(0.0);
                     shown >= rect.height * fraction
                 })
             });
             let initial = if visible.get_untracked() { Seen::Seen } else { Seen::Waiting };
             let latch = machine(initial, || Latch, |_, ()| {});
             latch.feed(visible, |visible| visible);
             let state = latch.state();
             memo(move || state.get() == Seen::Seen)
         }
     }
     ```

  2. `crates/kivact-core/src/view.rs` に `mod node_ref_visibility;` を足す（`node_ref` の `mod` 行の次）。
  3. 試験を足す（e2e_core。`mount` は `crate::support::mount`）：
     - `dev/tests/kivact-public-api/tests/e2e_core/node_ref.rs` に：

       ```rust
       #[test]
       fn once_visible_turns_true_when_scrolled_in_and_stays_true() {
           let (screen, (scroller, seen)) = mount(|| {
               let scroller = node_ref();
               let tile = node_ref();
               let seen = tile.once_visible(0.3);
               (
                   (scroller, seen),
                   scroll(column((
                       node().style(style().height(2000).flex_shrink(0)),
                       node().node_ref(tile).style(style().height(100).flex_shrink(0)),
                   )))
                   .node_ref(scroller)
                   .style(style().size(200, 300)),
               )
           });
           assert!(!seen.get(), "the tile starts below the window");
           scroller.scroll_to(Point::new(0.0, 10000.0), ScrollBehavior::Instant);
           screen.run_until_idle();
           assert!(seen.get(), "the tile is inside the window");
           scroller.scroll_to(Point::new(0.0, 0.0), ScrollBehavior::Instant);
           screen.run_until_idle();
           assert!(seen.get(), "the latch stays true");
       }
       ```

- **消すもの**：なし（kit の写しは K39）。
- **状態の閉じ方**：`Seen` は 2 値の enum。新しい状態の型は他に無い。
- **試験**：上の 1 つ（e2e_core の `node_ref`）。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-core -p kivact-public-api-tests
  cargo check -p kivact-core --all-targets
  cargo clippy -p kivact-core --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-core
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- once_visible
  ```

- **完了条件**：上の試験が通り、`e2e_core` の filter `once_visible` が 1 件を実行する（0 件は失敗）。`rg -n "pub fn once_visible\b" crates/kivact-core/src` が 1 行。

## K10：`kivact-plot`（basic の plot を移す）

- **wave / lane / 前提**：W9 / K6 / K09
- **所有**：`crates/kivact-plot/`（全体）、`crates/kivact-ui-basic/src/plot.rs`、`crates/kivact-ui-basic/src/plot/`、`crates/kivact-ui-basic/src/nice.rs`、`crates/kivact-ui-basic/tests/consumer/plot.rs`（移す）
- **目的**：basic の plot（2.9k 行）を kit と無関係の crate にする。色は `kivact_plot::color::{SERIES_1..SERIES_5, GRID, AXIS}` の theme 変数で受け、既定値は中核の色から導く（KIT-15、設計 5.10）。目盛りの丸め（`round_step`）を公開し、basic の chart が共有する（KIT-10）。
- **手順**：
  1. file を移す（`git mv`）：
     - `crates/kivact-ui-basic/src/plot/{axis,data,decimate,heatmap,input,model,paint,scale,series,ticks,view}.rs` → `crates/kivact-plot/src/` の同じ名前
     - `crates/kivact-ui-basic/src/nice.rs` → `crates/kivact-plot/src/nice.rs`
     - `crates/kivact-ui-basic/tests/consumer/plot.rs` → `crates/kivact-plot/tests/plot.rs`
     - `crates/kivact-ui-basic/src/plot.rs` は消す（中の `mod` と `pub use` は手順 3 の `lib.rs` に移す）。
  2. path の書換え：

     ```
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT/crates/kivact-plot" || exit 1
     perl -pi -e 's/\bsuper::super::(axis|data|decimate|heatmap|input|model|paint|scale|series|ticks|view)\b/crate::$1/g; s/\bcrate::plot::/crate::/g' src/*.rs
     perl -pi -e 's/\bsuper::(axis|data|decimate|heatmap|input|model|paint|scale|series|ticks|view)\b/crate::$1/g' src/*.rs
     perl -pi -e 's/\bpub\(super\)/pub(crate)/g' src/*.rs
     ```

     試験の module の `use super::*;` は残る（親の module を指す）。
  3. `crates/kivact-plot/src/lib.rs`：

     ```rust
     //! Role: Interactive XY plots and heatmaps on the public Kivact facade.
     //!
     //! [`plot`] draws a [`PlotModel`]: lines, points, stems, histograms and
     //! heatmaps with linear or logarithmic axes. Dragging pans, the wheel zooms
     //! and a double click returns to the automatic range, each as one edit.
     //! Colors are theme variables ([`color`]): any kit may set them, and
     //! without a kit they follow the core colors. A plot is exported to PNG or
     //! SVG through `kivact::draw::Offscreen`.

     mod axis;
     pub mod color;
     mod data;
     mod decimate;
     mod heatmap;
     mod input;
     mod model;
     pub mod nice;
     mod paint;
     mod scale;
     mod series;
     mod ticks;
     mod view;

     pub use axis::{AxisScale, PlotAxis, TickFormat};
     pub use data::PlotData;
     pub use model::{PlotModel, PlotRange, PlotView};
     pub use series::{LineDash, PlotSeries};
     pub use view::{Plot, plot};
     ```

  4. `crates/kivact-plot/src/nice.rs`：`pub(crate) fn round_step` を `pub fn round_step` にし、file の doc を `//! Role: The round tick step shared by plots and charts.` にする。
  5. `crates/kivact-plot/src/color.rs`（新）：

     ```rust
     //! Role: The theme colors of a plot: five series, the grid and the axes.
     //!
     //! A kit sets them with `Theme::set`; without a kit they follow the core
     //! colors of the theme.

     use kivact::style::{Color, ThemeVar, color};

     macro_rules! plot_color {
         ($var:ident, $name:ident, $init:expr, $doc:literal) => {
             #[doc = $doc]
             pub static $var: ThemeVar<Color> = $init;
             #[doc = $doc]
             pub const $name: Color = Color::var(&$var);
         };
     }

     plot_color!(SERIES_1_VAR, SERIES_1, ThemeVar::alias("plot.series_1", &color::ACCENT_VAR), "The first series without its own color.");
     plot_color!(SERIES_2_VAR, SERIES_2, ThemeVar::alias("plot.series_2", &color::SUCCESS_VAR), "The second series.");
     plot_color!(SERIES_3_VAR, SERIES_3, ThemeVar::alias("plot.series_3", &color::WARNING_VAR), "The third series.");
     plot_color!(SERIES_4_VAR, SERIES_4, ThemeVar::alias("plot.series_4", &color::DANGER_VAR), "The fourth series.");
     plot_color!(SERIES_5_VAR, SERIES_5, ThemeVar::alias("plot.series_5", &color::TEXT_MUTED_VAR), "The fifth series.");
     plot_color!(GRID_VAR, GRID, ThemeVar::alias("plot.grid", &color::BORDER_VAR), "Grid lines and the plot frame.");
     plot_color!(AXIS_VAR, AXIS, ThemeVar::alias("plot.axis", &color::TEXT_MUTED_VAR), "Axis labels, ticks and titles.");

     /// The series colors in order; series `i` without a color uses entry `i % 5`.
     pub(crate) const SERIES: [Color; 5] = [SERIES_1, SERIES_2, SERIES_3, SERIES_4, SERIES_5];
     ```

     `color::ACCENT` などは S 表の `Palette::{accent, success, warning, danger, text_muted, border}` の行の変数。
  6. `crates/kivact-plot/src/paint.rs`：
     - `use crate::tokens::Tokens;` と `Frame` の field `tokens: &'a Tokens` を消す。`paint` の `let tokens = draw.theme().ext::<Tokens>();` と `tokens: &tokens,` を消す。
     - `tokens.<field>` の置換（`rg -n "tokens\." crates/kivact-plot/src/paint.rs` が出す 13 か所）：`card` → S 表の `Palette::surface` の変数、`chart[..]` → `crate::color::SERIES[index % 5]`、`border`（grid と frame の 2 か所）→ `crate::color::GRID`、`muted_foreground` → `crate::color::AXIS`、`foreground` → S 表の `Palette::text` の変数、`ring` → S 表の `Palette::focus` の変数、`popover` → S 表の `Palette::raised` の変数、`popover_foreground` → S 表の `Palette::text` の変数、readout の `border` → S 表の `Palette::border` の変数。
     - `fn series_color(series, index, tokens)` を `fn series_color(series: &PlotSeries, index: usize) -> Color { series.color.unwrap_or(crate::color::SERIES[index % 5]) }` にし、呼出しから `tokens` の引数を消す。
     - `fn colormap_stops(draw, tokens)` を次にする（viridis は theme で変えない）：

       ```rust
       /// The viridis colormap of heatmaps, from the lowest value to the highest.
       const VIRIDIS: [u32; 9] = [
           0x440154, 0x472d7b, 0x3b528b, 0x2c728e, 0x21918c, 0x28ae80, 0x5ec962, 0xaddc30, 0xfde725,
       ];

       /// The heatmap colormap as RGBA stops in `0..=1`.
       fn colormap_stops(draw: &Draw) -> Vec<[f64; 4]> {
           VIRIDIS
               .iter()
               .map(|hex| {
                   let c = kivact::style::rgb(*hex).components(draw.theme());
                   [c.r, c.g, c.b, c.a]
               })
               .collect()
       }
       ```

  7. message の key：`rg -l "basic\.plot\." crates/kivact-plot | xargs perl -pi -e 's/"basic\.plot\./"plot./g'`。
  8. `crates/kivact-plot/src/view.rs`：builder の `Plot` と `fn plot(label, model)` と setter（`range`、`on_edit`、`height`）と `impl Element for Plot`・`impl IntoView for Plot` を、3.0.5 の C1 に従って一つの関数にする：

     ```rust
     #[component]
     pub fn plot(
         model: Value<PlotModel>,
         range: Option<Signal<PlotRange>>,
         on_edit: Option<Callback<EditPhase>>,
         #[prop(default = Length::from(240))] height: Value<Length>,
     ) -> impl Element {
         // 今の into_view の本体。今の label は使わない（名前は呼出し側の `.label(..)`。DEC-05）
     }
     ```

     今の本体が `label` を root の `.name(label)` に付けていた文は消す（呼出し側の `.label(..)` が absorb で root に付く）。`root: column(()).style(style().width(pct(100)).min_width(0))` は本体の root の `column((..))` にその style を付ける。
  9. `crates/kivact-plot/tests/plot.rs`：`use kivact_ui_basic::..` を `use kivact_plot::..` にし、`basic::theme(..)` で theme を作っていた所を `Theme::light()`（dark の試験は `Theme::dark()`）にし、`plot("Label", model)` を `plot(model).label("Label")` にする。message の key の試験（`the_description_is_written_in_the_locale`）の key を `plot.description` にする。期待値は変えない。file の先頭の `use` で `support` の module を使っていたら、要る関数を同じ file に写す。
  10. `kivact::draw::Offscreen` で書き出す doc の例（paint.rs の `render`・`render_svg` の言及）は、`kivact::draw::Offscreen` の W9 の API のまま。
- **消すもの**：`crates/kivact-ui-basic/src/plot.rs`、`crates/kivact-ui-basic/src/plot/`、`crates/kivact-ui-basic/src/nice.rs`、`crates/kivact-ui-basic/tests/consumer/plot.rs`、`Plot` の `label` 位置引数。
- **状態の閉じ方**：新しい状態の型なし。
- **試験**：`crates/kivact-plot/tests/plot.rs` の 9 つ（`a_plot_is_a_focusable_image_described_by_its_series_and_range`、`the_description_is_written_in_the_locale`、`a_drag_pans_the_range_live_and_commits_once_on_release`、`a_press_without_a_move_reports_no_edit`、`escape_during_a_drag_restores_the_range_it_started_from`、`a_wheel_step_zooms_x_in_one_edit_at_once`、`a_horizontal_wheel_pans_x_towards_larger_values`、`keys_zoom_pan_and_fit_with_one_edit_each`、`a_double_click_returns_the_range_to_auto_in_one_edit`）と、移った unit 試験（`nice`、`scale`、`ticks`、`heatmap`、`series` などの `#[cfg(test)]`）。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-plot
  cargo check -p kivact-plot --all-targets
  cargo clippy -p kivact-plot --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-plot
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-plot
  ```

- **完了条件**：`rg -n "Tokens|kivact_ui_basic|basic\." crates/kivact-plot` が何も出さない。`ls crates/kivact-ui-basic/src/plot* crates/kivact-ui-basic/src/nice.rs` が「無い」と言う。上の command が通る。

## K11：中核の `sidebar`（basic の sidebar を `kivact::ui` へ移す）

- **wave / lane / 前提**：W9 / K6 / K10
- **所有**：`crates/kivact-ui/src/sidebar.rs` と `crates/kivact-ui/src/sidebar/`（新。annex U 4.4 節の module 表のとおり `mod.rs` は作らない。`sidebar/look.rs`・`sidebar/recipes.rs`・`sidebar/vars.rs` を含む）、`crates/kivact-ui/src/lib.rs`、`crates/kivact-ui/src/theme.rs`、`crates/kivact-ui-basic/src/sidebar.rs`、`crates/kivact-ui-basic/src/sidebar/`（移して消す）、`dev/tests/kivact-public-api/tests/e2e_ui/ui_sidebar.rs`（新）、`e2e_ui` の `main.rs`（`mod ui_sidebar;` の 1 行）
- **目的**：basic と soft が別々に持つ sidebar（1.2k 行と 1.5k 行）の振る舞い（開閉、icon の列、rail、key の toggle）を中核の一つにする（KIT-10、設計 5.9）。他の中核の部品と同じく、形は recipe の表 `SidebarTheme`（annex U の `recipe_table!` の形）で、値は部品の theme 変数（DEC-06）で決まる。kit は `SidebarTheme` の part を差し替え（K34、K42）、変数を set する。
- **決定（この unit が作る中核の API。4.3 節にも載せる）**：

  ```rust
  // kivact::ui
  pub enum SidebarSide { Left, Right }                         // 既定 Left
  pub enum SidebarFrame { Sidebar, Floating, Inset }           // 既定 Sidebar（basic の SidebarVariant。幅を変えるので variant ではなく props）
  pub enum SidebarCollapsible { Offcanvas, Icon, None }        // 既定 Offcanvas
  pub enum SidebarMenuButtonSize { Default, Sm, Lg }           // 既定 Default。Variant（`.variant(SidebarMenuButtonSize::Lg)`）

  #[component]
  pub fn sidebar_provider(
      open: Signal<bool>,
      sidebar: View,
      page: View,
      #[prop(default = SidebarSide::Left)] side: SidebarSide,
      #[prop(default = SidebarFrame::Sidebar)] frame: SidebarFrame,
      #[prop(default = SidebarCollapsible::Offcanvas)] collapsible: SidebarCollapsible,
  ) -> impl Element;
  #[component]
  pub fn sidebar(
      header: Option<View>,
      content: Option<View>,
      footer: Option<View>,
      #[prop(default = false)] rail: Value<bool>,
  ) -> impl Element;
  #[component] pub fn sidebar_trigger(on_press: Option<Callback<()>>) -> impl Element;
  #[component] pub fn sidebar_group(label: Value<String>, action: Option<View>, content: Option<View>) -> impl Element;
  #[component] pub fn sidebar_menu(children: Vec<View>) -> impl Element;
  #[component]
  pub fn sidebar_menu_button(
      label: Value<String>,
      icon: Option<View>,
      #[prop(default = false)] active: Value<bool>,
      badge: Option<Value<String>>,
      action: Option<View>,
      on_press: Option<Callback<()>>,
      sub: Option<View>,
  ) -> impl Element;
  #[component]
  pub fn sidebar_menu_sub_button(
      label: Value<String>,
      #[prop(default = false)] active: Value<bool>,
      on_press: Option<Callback<()>>,
  ) -> impl Element;
  #[component] pub fn sidebar_separator() -> impl Element;
  #[component] pub fn sidebar_input(value: Signal<String>) -> impl Element;   // 名前は `.label(..)`（DEC-05）
  #[component] pub fn sidebar_menu_skeleton() -> impl Element;

  // kivact::ui::theme::sidebar（部品の theme 変数）
  BACKGROUND, FOREGROUND, PRIMARY, PRIMARY_FOREGROUND, ACCENT, ACCENT_FOREGROUND, BORDER, RING, PAGE（Color）
  WIDTH（256）、WIDTH_ICON（48）、ITEM_RADIUS（0）（Length）

  // kivact::ui::theme（recipe の表と、recipe が読む状態）
  /// What the panel-level parts of a sidebar read.
  #[derive(Clone, Copy, Debug, PartialEq)]
  pub struct SidebarState {
      pub open: bool,
      pub icon_only: bool,                 // collapsible が Icon で閉じている
      pub side: SidebarSide,
      pub frame: SidebarFrame,
      pub collapsible: SidebarCollapsible,
  }
  /// What a row of a sidebar menu (and its badge, action and sub list) reads.
  #[derive(Clone, Copy, Debug, PartialEq)]
  pub struct SidebarItemState {
      pub active: bool,
      pub icon_only: bool,
      pub has_action: bool,
      pub rtl: bool,
      pub size: SidebarMenuButtonSize,
  }
  pub struct SidebarTheme { /* recipe_table! が作る。part と状態の型は下の表 */ }
  ```

  `SidebarTheme` の part（左は part 名、右は recipe が読む状態。今の basic の code で style を付けている node と、それを作る関数）：

  | part | 状態 | 今の style（移した file の関数） |
  |---|---|---|
  | `root` | `SidebarState` | `sidebar_provider` の `row(body).style(..)` |
  | `panel` | `SidebarState` | `panel.rs` の `build` の panel の node（`.style(move \|\| { let p = .. })` の 160 行目付近） |
  | `header` | `SidebarState` | `build` の header の `column(view).style(style().shrink(0).gap(8).padding(8))` |
  | `content` | `SidebarState` | `build` の `scroll(view).style(..)` |
  | `footer` | `SidebarState` | `build` の footer の column（header と同じ style） |
  | `rail` | `SidebarState` | `build_rail` |
  | `inset` | `SidebarState` | `inset`（page の側） |
  | `trigger` | `SidebarState` | `trigger.rs` の button |
  | `group` | `SidebarState` | `sidebar_group` の root（`menu.rs` 37 行目付近） |
  | `group_label` | `SidebarState` | `sidebar_group` の into_view の label の node（80 行目付近） |
  | `group_action` | `SidebarState` | `sidebar_group` の action の node（96 行目付近） |
  | `group_content` | `SidebarState` | `sidebar_group` の content の node（110 行目付近） |
  | `menu` | `SidebarState` | `sidebar_menu` |
  | `menu_button` | `SidebarItemState` | `button_style` |
  | `menu_button_icon` | `SidebarItemState` | `build_button` の icon の node（378 行目付近） |
  | `menu_badge` | `SidebarItemState` | `menu_badge` |
  | `menu_tooltip` | `SidebarItemState` | `menu_tooltip` |
  | `menu_action` | `SidebarItemState` | `menu_action` |
  | `menu_sub` | `SidebarItemState` | `menu_sub` |
  | `sub_button` | `SidebarItemState` | `sidebar_menu_sub_button` の into_view の style（464 行目付近） |
  | `separator` | `SidebarState` | `sidebar_separator` |
  | `input` | `SidebarState` | `sidebar_input` |
  | `skeleton` | `SidebarState` | `sidebar_menu_skeleton` |

  上の表に無い style を持つ node が移した code にあれば、その node を作る関数の名前を part 名にして同じ形で足す（「今の style を持つ node は一つ残らず part を通る」が完了条件）。

  `sidebar` は provider の外で使うと、開いた sidebar として描く（今の basic と同じ）。provider は `SidebarSide`・`SidebarFrame`・`SidebarCollapsible` と `open` を context で `sidebar` と部品に渡す（今の `SidebarCx`）。`Ctrl/Cmd+B` で開閉する（今のまま）。
- **手順**：
  1. 移す：`git mv crates/kivact-ui-basic/src/sidebar crates/kivact-ui/src/sidebar` と `git mv crates/kivact-ui-basic/src/sidebar.rs crates/kivact-ui/src/sidebar.rs`（`mod.rs` は作らない。annex U 1.4 節）。
  1a. 移した `sidebar/menu.rs` の `menu_tooltip` の `tooltip(..).attach(..)` は C12 の `tooltip(label, trigger)` にする。
  2. path と名前の書換え：

     ```
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT/crates/kivact-ui/src" || exit 1
     perl -pi -e 's/\bSidebarVariant\b/SidebarFrame/g; s/\bkivact::/kivact_core::/g' sidebar.rs sidebar/*.rs
     ```

     `kivact_core::interaction`・`kivact_core::ui` になった path は kivact-ui の中の path（`crate::..`、`kivact_interaction::..`）に直す（kivact-ui の他の部品の file の `use` に合わせる）。
  3. `crates/kivact-ui/src/sidebar/vars.rs`（新。部品の theme 変数。fallback は中核の変数）：

     ```rust
     //! Role: The sidebar's theme variables.

     use kivact_core::style::{Color, Length, ThemeVar, color, radius};

     macro_rules! sidebar_color {
         ($var:ident, $name:ident, $init:expr, $doc:literal) => {
             #[doc = $doc]
             pub static $var: ThemeVar<Color> = $init;
             #[doc = $doc]
             pub const $name: Color = Color::var(&$var);
         };
     }

     sidebar_color!(BACKGROUND_VAR, BACKGROUND, ThemeVar::alias("ui.sidebar.background", &color::SURFACE_ALT_VAR), "The panel.");
     sidebar_color!(FOREGROUND_VAR, FOREGROUND, ThemeVar::alias("ui.sidebar.foreground", &color::TEXT_VAR), "Text on the panel.");
     sidebar_color!(PRIMARY_VAR, PRIMARY, ThemeVar::alias("ui.sidebar.primary", &color::ACCENT_VAR), "A primary mark on the panel.");
     sidebar_color!(PRIMARY_FOREGROUND_VAR, PRIMARY_FOREGROUND, ThemeVar::alias("ui.sidebar.primary_foreground", &color::ACCENT_TEXT_VAR), "Text on the primary mark.");
     sidebar_color!(ACCENT_VAR, ACCENT, ThemeVar::alias("ui.sidebar.accent", &color::HOVER_VAR), "The active and hovered row.");
     sidebar_color!(ACCENT_FOREGROUND_VAR, ACCENT_FOREGROUND, ThemeVar::alias("ui.sidebar.accent_foreground", &color::TEXT_VAR), "Text on the active row.");
     sidebar_color!(BORDER_VAR, BORDER, ThemeVar::alias("ui.sidebar.border", &color::BORDER_VAR), "The panel's edge and separators.");
     sidebar_color!(RING_VAR, RING, ThemeVar::alias("ui.sidebar.ring", &color::FOCUS_VAR), "The focus ring of rows.");
     sidebar_color!(PAGE_VAR, PAGE, ThemeVar::alias("ui.sidebar.page", &color::SURFACE_VAR), "The page beside an inset sidebar.");

     /// The width of an open sidebar.
     pub static WIDTH_VAR: ThemeVar<f64> = ThemeVar::new("ui.sidebar.width", 256.0);
     /// The width of an open sidebar.
     pub const WIDTH: Length = Length::var(&WIDTH_VAR);
     /// The width of an icon-collapsed sidebar.
     pub static WIDTH_ICON_VAR: ThemeVar<f64> = ThemeVar::new("ui.sidebar.width_icon", 48.0);
     /// The width of an icon-collapsed sidebar.
     pub const WIDTH_ICON: Length = Length::var(&WIDTH_ICON_VAR);
     /// The corner radius of rows.
     pub static ITEM_RADIUS_VAR: ThemeVar<f64> = ThemeVar::new("ui.sidebar.item_radius", 0.0);
     /// The corner radius of rows.
     pub const ITEM_RADIUS: Length = Length::var(&ITEM_RADIUS_VAR);
     ```

     `color::SURFACE_ALT` などは S 表の `Palette` の行の変数。`radius` の import は使わなければ消す。
  3a. `crates/kivact-ui/src/sidebar/look.rs`（新）：手順の決定の `SidebarState`・`SidebarItemState` と、`SidebarTheme` を annex U の `recipe_table!` の形（U 3 章の button の例）で書く。part と状態の型は決定の表のとおり。`SidebarTheme` は `ThemeExtension`、`Default` は手順 3b の既定の recipe を入れた表。`crates/kivact-ui/src/theme.rs` に `pub use crate::sidebar::look::{SidebarItemState, SidebarState, SidebarTheme};` を足す。
  3b. `crates/kivact-ui/src/sidebar/recipes.rs`（新）：決定の表の part ごとに既定の recipe `fn <part>(t: &Theme, s: &Look<<状態>>, _base: Style) -> Style` を置く。本体は「今の style」の列の式を写し、手順 4 の置換を当て、今 closure の中で読んでいた reactive な値（`cx.open.get()`、`cx.icon_only()`、`active.get()`、`cfg.size.get()`、`rtl` など）を状態の field（`s.open`、`s.icon_only`、`s.active`、`s.size`、`s.rtl`）にする。basic の Lyra の literal は次の中核の値にする（kit の形は K34・K42 が part で差し替える）：`.radius(0)` → `vars::ITEM_RADIUS`、`.font_size(12)` → S 表の `font::SIZE_SMALL`、行の `padding(8)`・`gap(8)` → S 表の `space::SM`。高さ（`SidebarMenuButtonSize` の 32・28・48）と幅（`WIDTH`・`WIDTH_ICON`）は今の値のまま。
  3c. 部品の本体：移した code の各 node の `.style(..)` を、annex U が部品に recipe を当てるのと同じ書き方（U 3 章の button の例の「part を node に付ける」行）で `SidebarTheme` の part に置き換える。状態は本体で `memo` にして渡す（`SidebarState` は `cx` から、`SidebarItemState` は `active`・`cx.icon_only()`・`has_action`・`use_direction()`・`size` から）。
  4. 色の置換（移した file の中）：`use_tokens::<crate::Tokens>()` と `p.<field>` と `crate::ColorName::<Name>` を次で置き換える：`sidebar` → `vars::BACKGROUND`、`sidebar_foreground` → `vars::FOREGROUND`、`sidebar_primary` → `vars::PRIMARY`、`sidebar_primary_foreground` → `vars::PRIMARY_FOREGROUND`、`sidebar_accent` → `vars::ACCENT`、`sidebar_accent_foreground` → `vars::ACCENT_FOREGROUND`、`sidebar_border` → `vars::BORDER`、`sidebar_ring` → `vars::RING`、`background`（inset の page と `sidebar_input` の背景）→ `vars::PAGE`、その他の basic の token（`muted`、`border` など）→ S 表の対応する中核の変数。`state.rs` の `pub(crate) const WIDTH: f64 = 256.0;` と `WIDTH_ICON` は消し、読んでいた所を `vars::WIDTH`・`vars::WIDTH_ICON` にする（`width_of` が `f64` を返して計算していたら `Theme::get(&vars::WIDTH_VAR)` で読む）。`.radius(0)` は `.border_radius(vars::ITEM_RADIUS)`。style の closure が token だけを読んでいたら closure を外す（T8 の最後の行）。
  5. 部品を `#[component]` にする（3.0.5 の C1〜C7）：`Sidebar`+`sidebar()`、`SidebarGroup`+`sidebar_group`、`SidebarMenuButton`+`sidebar_menu_button`、`SidebarMenuSubButton`+`sidebar_menu_sub_button`、`SidebarTrigger`+`sidebar_trigger`、`sidebar_provider`、`sidebar_menu`、`sidebar_separator`、`sidebar_input`、`sidebar_menu_skeleton`。決定の署名に合わせ、`Sidebar::{side, variant, collapsible}` の setter は provider の props に移す。provider の本体は、今 `sidebar.side` などを読んでいた所を自分の props から読み、`sidebar.build(cx)` を `provide(cx, move || sidebar)`（`sidebar` は `View`。context から `cx` を読む）にする。`sidebar` の本体は `state::current()` から `cx` を読み、無いとき（provider の外）は今の `impl IntoView for Sidebar` と同じ「開いた sidebar」の `cx` を作る。`SidebarMenuButton::size` の setter は消し、本体が `SidebarMenuButtonSize` を読む所は、`sidebar_menu_button` の引数に `#[caller] caller: Caller` を足し（設計 5.1 規則 10、12 章 API-4）、`.variant(..)` で付いた値を `caller.variants().get::<SidebarMenuButtonSize>().unwrap_or_default()` で読む（`Caller` の読み方は annex C・U の `Caller` の API のまま）。`sidebar_input(label, value)` は `label` の位置引数を消す。
  6. message の key：`rg -n '"basic\.sidebar\.' crates/kivact-ui/src/sidebar*` の key を `ui.sidebar.` の接頭辞にする（`perl -pi -e 's/"basic\.sidebar\./"ui.sidebar./g'`）。
  7. 公開：`crates/kivact-ui/src/lib.rs` に `mod sidebar;` と `pub use sidebar::{SidebarCollapsible, SidebarFrame, SidebarMenuButtonSize, SidebarSide, sidebar, sidebar_group, sidebar_input, sidebar_menu, sidebar_menu_button, sidebar_menu_skeleton, sidebar_menu_sub_button, sidebar_provider, sidebar_separator, sidebar_trigger};` を足す（`Sidebar` などの生成された struct も同じ行に足す）。`crates/kivact-ui/src/theme.rs` に `pub use crate::sidebar::vars as sidebar;` を足す。
  8. 試験 `dev/tests/kivact-public-api/tests/e2e_ui/ui_sidebar.rs`（新。basic の `navigation_layout.rs` の `sidebar_toggles_by_trigger_shortcut_and_rail_between_256_and_48` を中核の API に写したもの。その試験の本体をそのまま写し、`basic::` を `kivact::ui::` に、theme を `Theme::light()` に、`.collapsible(..)` を provider の props にする。幅の期待値 256 と 48 は変えない）。名前は `sidebar_toggles_by_trigger_shortcut_and_rail_between_256_and_48`。`e2e_ui` の `main.rs` に `mod ui_sidebar;`。
- **消すもの**：`crates/kivact-ui-basic/src/sidebar.rs`、`crates/kivact-ui-basic/src/sidebar/`（移動）、`SidebarVariant`（`SidebarFrame` に改名）、`state.rs` の `WIDTH`・`WIDTH_ICON` の定数、basic の token の参照、部品の本体の中の inline の style（part に移った）。
- **状態の閉じ方**：`SidebarCx`（`open: Signal<bool>` と enum 3 つ。Copy、`&mut self` なし、`toggle` が `open` を書く）は Signal 系が 1 つだけで検査の対象外。`SidebarState`・`SidebarItemState` は recipe の状態（接尾辞 `State`、設計 6.1 (c)）、`SidebarTheme` は接尾辞 `Theme` で対象外。
- **試験**：`e2e_ui` の `ui_sidebar::sidebar_toggles_by_trigger_shortcut_and_rail_between_256_and_48`。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-ui -p kivact-public-api-tests
  cargo check -p kivact-ui --all-targets
  cargo clippy -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-ui
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_sidebar
  ```

- **完了条件**：`e2e_ui` の filter `ui_sidebar` が 1 件以上を実行する（0 件は失敗）。`rg -n "Tokens|ColorName|basic" crates/kivact-ui/src/sidebar*` が何も出さない。`rg -n "\.style\(" crates/kivact-ui/src/sidebar` が `recipes.rs` の外で、part を当てる書き方以外の行を出さない。上の試験が通る。

## K12：中核の sidebar に soft の side navigation が要る部分を足す

- **wave / lane / 前提**：W9 / K6 / K11
- **所有**：`crates/kivact-ui/src/sidebar/`（`look.rs`・`recipes.rs` を含む）、`crates/kivact-ui/src/lib.rs`（`SidebarResize` の公開の 1 語）、`dev/tests/kivact-public-api/tests/e2e_ui/ui_sidebar.rs`
- **目的**：soft の `side_nav`（K42 で消す）が持ち、中核の sidebar に無い振る舞いを中核に足し、soft を中核の上の見た目（`SidebarTheme` の part と変数）だけにする（KIT-10）。
- **決定（足す API）**：

  ```rust
  /// The width of a resizable sidebar: the inner edge drags it between `min` and `max`.
  #[derive(Clone, Copy, Debug, PartialEq)]
  pub struct SidebarResize {
      pub width: Signal<f64>,
      pub min: f64,
      pub max: f64,
  }
  // sidebar_provider に：resize: Option<SidebarResize>
  // sidebar_menu_button に：active_icon: Option<View>（active の間 icon の代わりに描く）、
  //                          end: Option<View>（label の後ろ、badge の前）、
  //                          sub_open: Option<Signal<bool>>（sub の開閉。None は常に開く）
  // sidebar_group に：description: Option<Value<String>>（label の下）、
  //                    #[prop(default = false)] label_hidden: Value<bool>（label を見えなくし、名前としては残す）
  ```

- **手順**：
  1. `sidebar/state.rs` に `SidebarResize` を置き、`lib.rs` の `pub use sidebar::{..}` に足す。`SidebarTheme`（`sidebar/look.rs`）に part を 4 つ足す：`resize_handle`（`SidebarState`）、`group_description`（`SidebarState`）、`menu_button_end`（`SidebarItemState`）、`menu_button_chevron`（`SidebarItemState`。`sub_open` があるときの開閉の印）。既定の recipe を `sidebar/recipes.rs` に置く（手順 2〜4 の style。`SidebarItemState` に `pub expanded: Option<bool>`（`sub_open` の値。無いとき `None`）を足す）。
  2. provider：`resize` が `Some` で `open` が真のとき、panel の幅は `resize.width`（`min`〜`max` に丸める）。panel の内側の端に 4 px の drag の取っ手（`role(Role::Splitter)`、名前は message `ui.sidebar.resize` の既定 "Resize sidebar"、`value_now`/`value_min`/`value_max` は幅）を置く。drag は `kivact::interaction::use_drag`（今の soft `side_navigation/shell.rs` の resize の実装を写す：`rg -n "resize" crates/kivact-ui-soft/src/side_navigation/shell.rs`）。←→ の key は 8 px ずつ変える。`resize` が `None` のときは今のまま（`vars::WIDTH`）。
  3. `sidebar_menu_button`：`active_icon` が `Some` で `active` が真の間は `icon` の代わりに描く。`end` は label の後ろに置く（badge の前）。`sub_open` が `Some` のとき、button の press は `on_press` を呼んだ後に `sub_open` を反転し、`sub` は `show(sub_open, ..)` で開閉し、button は `expanded(sub_open)` を持つ。`None` のときは今のまま常に開く。
  4. `sidebar_group`：`description` は label の下に `FOREGROUND` の弱い色（S 表の `Palette::text_muted` の変数）の 1 行。`label_hidden` が真の間、label の node を見えなくし（`.hidden(..)` か今 kivact-ui が「見えないが名前に残す」に使う方法。`rg -n "visually_hidden|VisuallyHidden" crates/kivact-ui/src | head -3`）、group の名前は label のまま。
  5. 試験を `ui_sidebar.rs` に足す：
     - `a_resizable_sidebar_follows_its_width_between_min_and_max`：`width = signal(300.0)`、`SidebarResize { width, min: 200.0, max: 400.0 }`。取っ手を role `Splitter` で取り、→ の key で `width` が 308、`width.set(1000.0)` で panel の rect の幅が 400。
     - `a_menu_button_with_sub_open_toggles_its_sub_items`：`sub_open = signal(false)`。sub の項目が無い → button を click → `sub_open` が真、項目がある、button の `is_expanded()` が `Some(true)`。
- **消すもの**：なし。
- **状態の閉じ方**：`SidebarResize` は `Signal` 一つと f64 2 つの値（constructor でだけ書く）。検査の対象外。
- **試験**：上の 2 つ。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-ui -p kivact-public-api-tests
  cargo check -p kivact-ui --all-targets
  cargo clippy -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-ui
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_sidebar
  ```

- **完了条件**：`ui_sidebar` の 3 つの試験が通り、filter `ui_sidebar` が 3 件を実行する（0 件は失敗）。

## K13：中核の `tabs` の overflow（`TabOverflow::{Menu, Scroll}`）

- **wave / lane / 前提**：W9 / K6 / K12
- **所有**：`crates/kivact-ui/src/tabs.rs` と `crates/kivact-ui/src/tabs/`（以下 `<tabs>`。annex U 4.4 節の module 表）、`crates/kivact-ui/src/tabs/overflow.rs`（新）、`crates/kivact-ui/src/lib.rs`（`TabOverflow` の公開）、`dev/tests/kivact-public-api/tests/e2e_ui/ui_tabs_overflow.rs`（新）、`e2e_ui` の `main.rs`（`mod` の 1 行）
- **目的**：soft の `tab_list`（966 行、自前の roving と overflow）が持つ「幅に収まらない tab」の扱いを中核の `tabs` に移す（KIT-09、設計 5.9）。Menu は溢れた tab の自然な大きさを測り続けて振動しない（UI-K-01）。Scroll は選択と layout の変化から選んだ tab を見える所へ出す（UI-K-02）。soft は K41 で `tab_list` を消し、中核の tabs を recipe で描く。
- **決定（API）**：

  ```rust
  /// What a tab strip does with tabs wider than its box.
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
  pub enum TabOverflow {
      /// The tabs that do not fit move into a "More" menu at the end of the strip.
      Menu,
      /// The strip scrolls along its axis; the selected tab scrolls into view.
      Scroll,
  }
  // tabs の props に：overflow: Option<TabOverflow>（None は今のまま、はみ出す）
  ```

- **手順**：
  1. `<tabs>/overflow.rs`：

     ```rust
     //! Role: How many tabs fit a strip before the "More" button.

     /// The number of leading tabs of `widths` that fit `strip` with `gap`
     /// between them, leaving room for a `more` button when not all fit.
     pub(crate) fn visible_count(strip: f64, widths: &[f64], gap: f64, more: f64) -> usize {
         let total: f64 = widths.iter().sum::<f64>() + gap * widths.len().saturating_sub(1) as f64;
         if total <= strip {
             return widths.len();
         }
         let room = strip - more - gap;
         let mut used = 0.0;
         for (index, width) in widths.iter().enumerate() {
             let next = used + width + if index == 0 { 0.0 } else { gap };
             if next > room {
                 return index;
             }
             used = next;
         }
         widths.len()
     }

     #[cfg(test)]
     mod tests {
         //! Role: Tests for the tab overflow count.

         use super::*;

         #[test]
         fn all_tabs_fit_or_the_rest_leave_room_for_more() {
             assert_eq!(visible_count(300.0, &[100.0, 100.0], 4.0, 32.0), 2);
             assert_eq!(visible_count(250.0, &[100.0, 100.0, 100.0], 4.0, 32.0), 2);
             assert_eq!(visible_count(120.0, &[100.0, 100.0, 100.0], 4.0, 32.0), 0);
         }
     }
     ```

  2. tabs の部品の本体（tab の行を作る所。`rg -n "fn tabs" crates/kivact-ui/src/tabs.rs crates/kivact-ui/src/tabs`）に `overflow: Option<TabOverflow>` の props を足す。`None` のときは今のまま。以下の「軸」は `orientation` が `Horizontal` なら幅（`rect.width`）、`Vertical` なら高さ（`rect.height`）。
  3. `Some(TabOverflow::Scroll)`（UI-K-02。設計 12 章）：
     - tab の行を `scroll(行).axes(<軸に合う ScrollAxes>)` で包み、scrollbar を見せない（今 kivact-ui が見えない scrollbar に使う style。`rg -n "scrollbar" crates/kivact-ui/src | head -3`）。行の外側の `scroll` に `let strip = node_ref();` を付ける。
     - 選択の key の `selector`（設計 5.2）を本体で一つ作る：`let is_selected = selector(selected);`（`selected` は tabs が今の選択を読む `Value<K>`／`Signal<K>`）。
     - 各 tab の項目の setup（`for_each` の行の関数の中）で、その tab の node に `let me = node_ref();` を付け、次の effect を一つ作る。effect は host への要求（`scroll_into_view`）だけを行い、signal を書かない（設計 12 章 UI-K-02。xtask の `effect-writes` の対象外）：

       ```rust
       effect(move || {
           if is_selected.is(&key) {
               // Re-run when the strip's layout changes (resize, label, font, theme).
               let _ = strip.rect().get();
               let _ = me.rect().get();
               me.scroll_into_view();
           }
       });
       ```

       選ばれていない tab の effect は `is(&key)` だけを追跡し、layout を追跡しない。選択の handler の中の `scroll_into_view` の呼出し（今あれば）は消す（経路は上の effect 一つ）。
  4. `Some(TabOverflow::Menu)`（UI-K-01。設計 12 章）：溢れた tab は layout から外さず、`position: absolute` と `visibility: hidden` で自然な大きさを測り続ける。
     - 行の style に `position(Position::Relative)`、`width(pct(100))`（縦は `height(pct(100))`）、`min_width(0)`（縦は `min_height(0)`）を足す。行に `let strip = node_ref();`。
     - 各 tab の項目の setup で `let me = node_ref();` を付け、本体の `let sizes: Signal<BTreeMap<K, Value<Option<Rect>>>> = signal(BTreeMap::new());` に `sizes.update(|m| { m.insert(key.clone(), me.rect()); })` で登録し、`on_cleanup(move || { sizes.update(|m| { m.remove(&key); }); })` で外す（setup と cleanup の書込み。effect ではない）。
     - "More" の button（icon `more-horizontal`、無ければ `chevron-down`。名前は message `ui.tabs.more` の既定 "More tabs"。style は `TabsTheme` の tab の recipe と同じ part を使う）に `let more = node_ref();` を付け、常に描く。
     - 見える数：

       ```rust
       let shown = memo(move || {
           let axis = |r: Rect| if vertical { r.height } else { r.width };
           let Some(strip) = strip.rect().get() else { return usize::MAX };
           let more = more.rect().get().map_or(0.0, axis);
           let lengths: Vec<f64> = items.with(|items| {
               sizes.with(|m| items.iter().map(|k| m.get(k).and_then(|r| r.get()).map_or(0.0, axis)).collect())
           });
           overflow::visible_count(axis(strip), &lengths, gap, more)
       });
       ```

       `items` は tabs の項目の key の列、`gap` は行の gap の値（`theme.get(..)` で読む長さ）、`vertical` は `orientation == Vertical`。`MORE_WIDTH` の定数は置かない（More の実際の大きさを測る。UI-K-01）。
     - tab の index が `shown` 以上の間、その tab の node に `style(style().position(Position::Absolute).left(0).top(0).visibility(Visibility::Hidden))`、`.focusable(false)`、`.hit_testable(false)` を当てる（`Visibility::Hidden` は「layout を保ち、描画・支援技術・入力から隠す」。`hidden(..)` は使わない：layout から外れて幅が 0 になり、見える数が振動する）。`shown` 以上の tab が無い間は、More の button に同じ 3 つを当てる（測るために残す）。
     - More の button を押すと、`shown` 以上の tab を項目にした `menu` を開く。項目は radio の印を持ち、今の選択の tab に印が付く。項目を選ぶと、その tab を選ぶ（tabs の選択の signal に書く。handler の中）。
     - 溢れた tab に focus があった（`me.focused().get_untracked()`）ときに `shown` が変わってその tab が溢れたら、More の button に focus を移す：上の Scroll の effect と同じ形の、host への要求だけの effect（`if is_hidden && me.focused().get() { more.focus(); }`）。
  5. `lib.rs` の tabs の `pub use` に `TabOverflow` を足す。
  6. 試験 `dev/tests/kivact-public-api/tests/e2e_ui/ui_tabs_overflow.rs`（8 つの tab、label "Tab 1".."Tab 8"、幅 200 の箱。`e2e_ui` の `main.rs` に `mod ui_tabs_overflow;`）：
     - `scroll_overflow_shows_an_initially_last_selection`：選択の初期値が 8 番目、`overflow(TabOverflow::Scroll)`。mount の後、"Tab 8" の rect が箱の rect の中。
     - `scroll_overflow_follows_an_external_selection`：`selected.set(<8 番目の key>)`（tabs の外から）→ "Tab 8" が箱の中。`selected.set(<1 番目>)` → "Tab 1" が箱の中。
     - `scroll_overflow_keeps_the_selection_visible_after_a_resize`：8 番目を選び、箱の幅の signal を 120 にして `run_until_idle` → "Tab 8" が箱の中。
     - `menu_overflow_is_stable_across_flushes`：`overflow(TabOverflow::Menu)`。`role(Tab)` の見える数を数え、`screen.run_until_idle()` を 3 回続けても同じ数で、"More tabs" の button がある。
     - `menu_overflow_follows_a_resize_both_ways`：箱の幅を 120 → 2000 → 200 に変える。2000 で "More tabs" が無く 8 つ全部が見え、120 で見える数が 200 のときより少ない。
     - `menu_overflow_follows_label_and_theme_changes`：一つ目の label の signal を長い文字列にすると見える数が減る。theme の `font::SIZE_VAR` を 24 にした theme に変えると見える数が減る。
     - `menu_overflow_works_for_vertical_tabs`：`orientation(Orientation::Vertical)`、高さ 120 の箱。"More tabs" があり、項目に "Tab 8" がある。
     - `menu_overflow_moves_the_selected_tab_into_more`：8 番目を選んでから幅を 120 にする。`query_by_role(Role::Tab, "Tab 8")` が `None`（支援技術から隠れる）、"More tabs" を押すと menu の "Tab 8" の項目が checked。menu の "Tab 2" を選ぶと選択の signal が 2 番目。
- **消すもの**：なし（soft の `tab_list` は K41）。
- **状態の閉じ方**：`shown` は memo、`sizes` は関数の中の signal（項目の setup と cleanup だけが書く登録）。新しい型なし。
- **試験**：上の 8 つと `overflow.rs` の unit 試験。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-ui -p kivact-public-api-tests
  cargo check -p kivact-ui --all-targets
  cargo clippy -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-ui
  cargo test -p kivact-ui --lib overflow
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_tabs_overflow
  ```

- **完了条件**：上の試験が通り、`e2e_ui` の filter `ui_tabs_overflow` が 8 件を実行する（0 件は失敗）。`rg -n "pub enum TabOverflow" crates/kivact-ui/src` が 1 行。`rg -n "MORE_WIDTH|hidden\(true\)" crates/kivact-ui/src/tabs.rs crates/kivact-ui/src/tabs` が何も出さない。

## K14：`catalog-shell`（5 つの catalog の共通の shell）

- **wave / lane / 前提**：W9 / K6 / K13
- **所有**：`examples/catalog-shell/`（全体）
- **目的**：basic の catalog の良い設計（`Page{slug, name, ..., source: include_str!, examples}` と `// example:` の印）を共通の crate にし、5 つの shell・page の登録・2 つの scroll-spy の写しを一つにする（KIT-12）。catalog の試験の「全 page を開いて誤りが無い」の walker を一つにする（KIT-20）。
- **決定（API）**：

  ```rust
  // examples/catalog-shell/src/lib.rs
  pub struct Page { /* slug, name, category, description, usage, source, align, examples */ }
  pub struct Example { /* id, title, description, view */ }
  pub enum PreviewAlign { Center, Start, End }
  pub struct CatalogTheme(fn(ThemeMode) -> Theme);
  pub static FONTS: &[&[u8]];   // Inter、Geist Mono、Noto Sans JP（examples/shared-assets/fonts）
  impl Page {
      pub fn new(slug: &'static str, name: &'static str, category: &'static str, source: &'static str) -> Page;
      pub fn description(self, text: &'static str) -> Page;
      pub fn usage(self, code: &'static str) -> Page;
      pub fn align(self, align: PreviewAlign) -> Page;
      pub fn example(self, example: Example) -> Page;
      pub fn snippet(&self, id: &str) -> String;
  }
  impl Example { pub fn new<V: IntoView>(id: &'static str, title: &'static str, description: &'static str, view: fn() -> V) -> Example; pub fn view(&self) -> View; }
  impl CatalogTheme { pub const fn new(theme: fn(ThemeMode) -> Theme) -> CatalogTheme; pub fn get(&self, mode: ThemeMode) -> Theme; }
  #[component]
  pub fn catalog(
      title: &'static str,
      pages: Vec<Page>,
      theme: CatalogTheme,
      #[prop(default = "")] start: &'static str,
      #[prop(default = ThemeMode::Light)] mode: ThemeMode,
  ) -> impl IntoView;
  #[cfg(feature = "test")] pub mod check;   // 下の手順 6
  ```

- **手順**：
  1. `src/model.rs`：`examples/kivact-ui-basic/src/model.rs` の `Page`・`PreviewAlign`・`Example` と `Page::snippet` を写し、`pub(crate)` を `pub` にする。`Page` に `category: &'static str` を足す（左の一覧の見出し）。`Page.description`・`Page.usage` の doc を英語にする（「A Japanese description」→「An English description」）。`Page::new`・`description`・`usage`・`align`・`example` を足す（値渡しの builder。`description` と `usage` の既定は `""`、`align` は `Center`、`examples` は空）。`Example` の `view` の型は `Box<dyn Fn() -> View>`（`Send + Sync` を外す。page の表は static に置かないので要らない）。
  2. `src/page.rs`：`examples/kivact-ui-basic/src/page.rs` を写す。`basic::lead(..)` は `text(..).style(style().font_size(16).color(color::TEXT_MUTED))`（S 表の変数）、`basic::ColorName::X` と `look::tokens().x` は S 表の中核の変数、"使い方" は "Usage"、"コンポーネント" は "Components"。`usage` が空の page は "Usage" の節を出さない。test id（`catalog-page`、`catalog-title`、`catalog-usage-code`、`example-<slug>-<id>-preview`、`example-<slug>-<id>-code`）は今のまま。
  3. `src/shell.rs`：`examples/kivact-ui-basic/src/shell.rs` を写し、次を変える：
     - 入力は `pages: Rc<Vec<Page>>`、`theme: CatalogTheme`、`title`、`start`、`mode`。`PAGES` の static を使わない。
     - 一覧は `category` の順に見出し（`text(category)`、小さい弱い文字）を入れる。並びは `pages` の順（catalog が決める）。検索は今の `component_items` と同じく名前と slug の小文字の部分一致。
     - theme：`provide_theme(move || theme.get(if dark.get() { ThemeMode::Dark } else { ThemeMode::Light }), ..)`。今の `FAMILY` と `with_text` は消す（font family は catalog の theme 関数が set する）。
     - 文言：title は `text("Kivact UI")` と `text(title)`、theme の button の名前は "Toggle theme"、検索の placeholder と名前は "Search components"、空の表示は "No results"。
     - 色：`basic::ColorName::Background` → `color::SURFACE`、`Foreground` → `color::TEXT`、`MutedForeground` → `color::TEXT_MUTED`、`Border` → `color::BORDER`、一覧の行の `p.muted` → `color::SURFACE_ALT`、`p.ring.with_alpha(0.5)` → `color::FOCUS`（S 表の変数。style の closure の中でも変数のまま）。
     - 開始の page：`start` の slug の page、無ければ 0 番。
     - toast：`kivact::ui::toast_region(kivact::ui::use_toaster())` を今と同じ場所に置く（`toaster()` は W7〜W8 で `use_toaster()` に改名済み）。
     - test id（`catalog-theme`、`catalog-search`、`catalog-nav-scroll`、`catalog-nav-<slug>`）は今のまま。
  4. `src/fonts.rs`：

     ```rust
     //! Role: The catalogs' shared offline OFL fonts.

     /// Inter, Geist Mono and Noto Sans JP (subset), for the catalogs and their tests.
     pub static FONTS: &[&[u8]] = &[
         include_bytes!("../../shared-assets/fonts/inter/Inter[opsz,wght].ttf"),
         include_bytes!("../../shared-assets/fonts/geistmono/GeistMono[wght].ttf"),
         include_bytes!("../../shared-assets/fonts/notosansjp/NotoSansJP-subset[wght].ttf"),
     ];
     ```

  5. `src/lib.rs`：crate doc（「A catalog is a list of [`Page`]s and a theme function; [`catalog`] shows them with a searchable list, a theme toggle and the page area. A page file defines its demos between `// example: <id>` and `// end example`, so the code on screen is the code that runs.」）、`mod fonts; mod model; mod page; mod shell;`、`#[cfg(feature = "test")] pub mod check;`、`pub use fonts::FONTS; pub use model::{Example, Page, PreviewAlign}; pub use shell::{CatalogTheme, catalog};`。`catalog` の `#[component]` は `shell.rs` に置く。
  6. `src/check.rs`（feature `test`）：`examples/kivact-ui-basic/tests/catalog.rs` の `mount_in`、`layout_check`、`reveal`、`open`、`visit_every_page`、`bundled`、`glyph_cell`、`cell_pixels` と `every_page_character_is_in_the_catalog_fonts` の本体を、次の公開の関数に写す（期待値は同じ）：

     ```rust
     /// Mount `catalog` at 1440×900, scale 1, reduced motion, in `mode`.
     pub fn mount(title: &'static str, pages: fn() -> Vec<Page>, theme: CatalogTheme, mode: ThemeMode) -> Screen;
     /// Open every page from the list in both modes: the page is selected, its
     /// title is its name, every example has its preview and code, the screen
     /// has no errors and no layout violation outside `clipped` (slugs whose
     /// previews clip on purpose).
     pub fn every_page_opens_cleanly(title: &'static str, pages: fn() -> Vec<Page>, theme: CatalogTheme, clipped: &[&str]);
     /// Every visible character above ASCII of the page sources is drawn by one of the theme's fonts.
     pub fn every_page_character_is_in_the_fonts(pages: fn() -> Vec<Page>, theme: CatalogTheme);
     /// Click `catalog-nav-<slug>` after scrolling it into the list.
     pub fn open(screen: &Screen, slug: &str);
     ```

     `mount` は `RenderOptions::with_theme(theme.get(mode))`（font は theme が持つ。`TestFonts::Only` は W5 で消えた）と `move || catalog(title, pages(), theme).mode(mode)`。`every_page_opens_cleanly` は `visit_every_page` を Light と Dark で呼ぶ。`catalog-usage-code` の確認は `page.usage` が空でない page だけ。`screen.evidence(..)` は今のまま。
- **消すもの**：なし（catalog の写しは各 catalog の unit が消す）。
- **状態の閉じ方**：shell の `dark`・`selected`・`query` は関数の中の signal（`locals.rs` の検査の対象）。`selected: Signal<Option<usize>>` と `dark: Signal<bool>` は別の事実（表示中の page と配色）で、`locals.rs` が組を指摘したら `// xtask-allow(state): dark picks the theme and selected the shown page; neither writes the other.` を `catalog` の本体の `let dark` の行の上に置く。
- **試験**：catalog-shell 自身には試験を置かない（`check` を各 catalog の試験が呼ぶ）。
- **確認（tier a）**：

  ```
  cargo fmt -p catalog-shell
  cargo check -p catalog-shell --all-targets
  cargo check -p catalog-shell --all-targets --features test
  cargo clippy -p catalog-shell --all-targets --features test -- -D warnings
  cargo xtask check --no-write --crate catalog-shell
  ```

- **完了条件**：上の command が通る。`rg -c '[\p{Hiragana}\p{Katakana}\p{Han}]' examples/catalog-shell/src` が何も出さない。

---

### 3.1 lane K1：basic（K15〜K19、K34、K35）

## K15：basic の土台（theme 変数・theme・variant・段階の lib.rs）

- **wave / lane / 前提**：W9 / K1 / W8 の完了
- **所有**：`crates/kivact-ui-basic/`（`src/plot.rs`、`src/plot/`、`src/nice.rs`、`src/sidebar.rs`、`src/sidebar/`、`tests/consumer/plot.rs` を除く）
- **目的**：basic を compile の対象に戻し、骨格の土台（`color.rs`、`theme.rs`、`variants.rs`）を作る。`Tokens`・`ColorName`・`CodeColors`・`theme_access.rs` を消す（KIT-06、KIT-07、DEC-06）。
- **手順**：
  1. root `Cargo.toml` は変えない（kit・catalog は workspace の member のまま。gate の `--exclude` が外していたのをやめるだけ。INC-09）。
  2. `src/color.rs` を書き直す（T1〜T5）。変数（名前 ← 今の `Tokens` の field。fallback は今の `Tokens::new(ThemeMode::Light)` の値）：
     `BACKGROUND`、`FOREGROUND`、`CARD`、`CARD_FOREGROUND`、`POPOVER`、`POPOVER_FOREGROUND`、`PRIMARY`、`PRIMARY_FOREGROUND`、`SECONDARY`、`SECONDARY_FOREGROUND`、`MUTED`、`BUBBLE_TINTED`、`BUBBLE_TINTED_HOVER`、`MUTED_FOREGROUND`、`ACCENT`、`ACCENT_FOREGROUND`、`DESTRUCTIVE`、`BORDER`、`INPUT`、`RING`、`SWITCH_THUMB`、`CHART_1`〜`CHART_5`（← `chart[0..5]`）、`SIDEBAR`、`SIDEBAR_FOREGROUND`、`SIDEBAR_PRIMARY`、`SIDEBAR_PRIMARY_FOREGROUND`、`SIDEBAR_ACCENT`、`SIDEBAR_ACCENT_FOREGROUND`、`SIDEBAR_BORDER`、`SIDEBAR_RING`（変数の名前の文字列は `"basic.<field>"`、`chart` は `"basic.chart_1"`〜）。
     `pub(crate) fn set_colors(theme: Theme, mode: ThemeMode) -> Theme` は今の `Tokens::new(mode)` の値を全部 set する。`code: CodeColors` と `heatmap` は basic の変数にしない（`code` は K34 で中核の構文色に set、`heatmap` は K10 で `kivact-plot` の定数になった）。
  2a. 別名の fallback（T1a）：`BACKGROUND` ← `color::SURFACE`、`FOREGROUND` ← `color::TEXT`、`MUTED` ← `color::SURFACE_ALT`、`MUTED_FOREGROUND` ← `color::TEXT_MUTED`、`PRIMARY` ← `color::ACCENT`、`PRIMARY_FOREGROUND` ← `color::ACCENT_TEXT`、`BORDER` ← `color::BORDER`、`DESTRUCTIVE` ← `color::DANGER`、`POPOVER` ← `color::RAISED`。他は light の literal。
  3. `src/theme.rs` を書き直す（3.0.7 の `theme`・`build`。basic は `size.rs` を作らない）。`set_core` は次の対応で set する（T6・T7。左は今の `theme.rs` の値、右の変数名は S 4.2 節の表の名前）：

     | 今 | set する変数 ← 値 |
     |---|---|
     | `Palette::surface: p.background` | `color::SURFACE_VAR` ← `BACKGROUND` |
     | `surface_alt: p.muted` | `color::SURFACE_ALT_VAR` ← `MUTED` |
     | `text: p.foreground` | `color::TEXT_VAR` ← `FOREGROUND` |
     | `text_muted: p.muted_foreground` | `color::TEXT_MUTED_VAR` ← `MUTED_FOREGROUND` |
     | `accent: p.primary` | `color::ACCENT_VAR` ← `PRIMARY` |
     | `accent_text: p.primary_foreground` | `color::ACCENT_TEXT_VAR` ← `PRIMARY_FOREGROUND` |
     | `border: p.border` | `color::BORDER_VAR` ← `BORDER` |
     | `focus: p.ring.with_alpha(0.5)` | `color::FOCUS_VAR` ← `theme.get(&RING_VAR).with_alpha(0.5)` |
     | `danger: p.destructive` | `color::DANGER_VAR` ← `DESTRUCTIVE` |
     | `success`、`warning` | 今の literal（`rgb(if dark { 0x4ade80 } else { 0x16a34a })` など）をそのまま |
     | `overlay`、`shadow` | `Color::BLACK.with_alpha(0.1)` |
     | `selection: p.primary.with_alpha(0.15)` | `theme.get(&PRIMARY_VAR).with_alpha(0.15)` |
     | `hover: p.muted` | `color::HOVER_VAR` ← `MUTED` |
     | `pressed: p.secondary.mix(p.foreground, 0.05)` | `theme.get(&SECONDARY_VAR).mix(theme.get(&FOREGROUND_VAR), 0.05, &theme)`（S の `mix` の署名） |
     | `raised: p.popover` | `color::RAISED_VAR` ← `POPOVER` |
     | `Spacing { xs: 4, sm: 8, md: 12, lg: 16, xl: 24 }` | `space::{XS,SM,MD,LG,XL}_VAR` に同じ値 |
     | `Radii { sm: 0, md: 0, lg: 0 }` | `radius::{SM,MD,LG}_VAR` に 0 |
     | `Typography { size: 12, size_small: 12, size_large: 14, weight: 400, weight_strong: 500, line_height: Scale(16/12) }` | `font::{SIZE,SIZE_SMALL,SIZE_LARGE,WEIGHT,WEIGHT_STRONG,LINE_HEIGHT}_VAR` に同じ値 |
     | `with_focus_ring(1.0)`、`with_control_height(32.0)` | `.with_focus_ring(1.0)` は builder のまま、`.set(&control::HEIGHT_VAR, 32.0)`（S 4.3。INC2 N-05） |
     | `UiTokens.chat.follow_slack: 1.5`、`code_block {font_size 14, line_height 23, copy_feedback_ms 2000}`、`indicators.reduced_period_ms: Some(1000)`、`calendar {cell_size 28, month_gap 16, week_gap 8}`、`combobox {list_height 252, popup_offset 4, field_radius Some(0), chip_radius 0, option_radius 0}` | U 変数表の各行の変数に同じ値 |

     今の `.with(p)`（Tokens）と `.with(standard_tokens())` と `.with(crate::recipes::recipes())` は書かない。`build` の `install` の呼出しは K16・K17 が足す。
  4. `src/variants.rs`（新）：今の次の型の定義（derive、doc、`impl Variant`）をここへ移す（元の file からは消す。元の file はまだ compile されない）：
     - そのまま：`AlertDialogSize`（alert_dialog.rs）、`AttachmentSize`・`AttachmentMediaVariant`（attachment.rs）、`BadgeVariant`（badge.rs）、`CardSize`（card.rs）、`ChatLayout`（chat.rs）、`ToggleVariant`（choice.rs）、`CodeBlockSize`（code_block/view.rs）、`DatePickerVariant`（date_picker.rs）、`DisclosureVariant`（disclosure.rs）、`DrawerVariant`（drawer.rs）、`EmptyMediaVariant`（empty.rs）、`FieldLegendVariant`（field.rs）、`InputGroupControl`・`InputGroupButtonSize`（input_group.rs）、`ItemVariant`・`ItemSize`・`ItemMediaVariant`（item.rs）、`TabsVariant`（line_tabs.rs）、`MarkerVariant`（marker.rs）、`MenuVariant`（recipes_menu.rs）、`NativeSelectSize`（native_select.rs。今 `Variant` でなければ移さない）。
     - 改名：`AlertLook` → `AlertTone`（V2：中核に `AlertVariant` がある）、`BubbleVariant` → `ChatBubbleVariant`（chat_colors.rs。V1）。
     - 束の分解（V4）：`CardLook { size, header_bordered }` → `CardSize` と新しい `pub struct CardHeaderBordered(pub bool);`。`ItemLook { size }` → `ItemSize`。`CodeBlockLook { size }` → `CodeBlockSize`。`InputGroupButtonLook { size }` → `InputGroupButtonSize`。4 つの `*Look` と、その `impl Default`・`impl Variant`・setter を消す。
     - `#[derive(Prop)]`（C09 が付けた）は保つ（V7）。`impl kivact::ui::Variant for X {}` は V6。
     - 移した後、`date_picker.rs`、`disclosure.rs`、`drawer.rs`、`alert_dialog.rs`、`chat_colors.rs` に variant しか残らなければ、その file を消す（`chat_colors.rs` の `pub(crate) fn colors` は K17 で `recipes_chat.rs` に移る）。
  5. 消す：`src/tokens.rs`、`src/theme_access.rs` の中の `Tokens` を読む helper（file 自体は K16 が `recipe_parts.rs` に移すので、この unit では `mod` 行を書かず残す）、`src/color.rs` の `ColorName` と `impl ThemeColor`。
  6. `src/lib.rs`：crate doc は今の 1 段落目と使い方の段落を残し、`Tokens`・`ColorName` を説明する文と「# Texts」の節の README への言及を消す。module は `mod color; mod icons; mod theme; mod variants;` だけ。`pub use` は `pub use color::{..};`（手順 2 の全部の `NAME` と `NAME_VAR`。`rg -o 'pub (static|const) ([A-Z0-9_]+)' -r '$2' crates/kivact-ui-basic/src/color.rs | sort | tr '\n' ','` で列を作る）、`pub use icons::ICON_NAMES;`、`pub use theme::theme;`、`pub use variants::{..};`（手順 4 の全部）。
- **消すもの**：`Tokens`、`CodeColors`、`ColorName`、`AlertLook`、`CardLook`、`ItemLook`、`CodeBlockLook`、`InputGroupButtonLook`、`BubbleVariant`（改名）、`src/tokens.rs`。
- **状態の閉じ方**：新しい状態の型なし（`CardHeaderBordered` は値）。
- **試験**：なし（`tests/` は K35 で戻す）。
- **確認（tier a）**：3.0.9 の途中の unit の 4 行（`<kit>` = `basic`）。
- **完了条件**：`rg -n "ColorName|struct Tokens|ext::<Tokens>|use_tokens" crates/kivact-ui-basic/src/{color,theme,variants,lib}.rs` が何も出さない。上の command が通る。

## K16：basic の recipe 1（controls と overlays）

- **wave / lane / 前提**：W9 / K1 / K15
- **所有**：K15 と同じ
- **目的**：今の `recipes.rs`・`recipes_controls.rs`・`recipes_field.rs`・`recipes_popup.rs`・`recipes_dialog.rs`・`recipes_menu.rs`・`recipes_command.rs`・`recipes_combobox.rs`・`button.rs`・`theme_access.rs`・`list_popup.rs`・`layers.rs`・`choice.rs`・`parts.rs` を、3.0.3 の形の `recipes_controls*.rs`・`recipes_overlays*.rs`・`recipe_parts.rs` にする（KIT-06）。
- **手順**：
  1. 共有の helper を `src/recipe_parts.rs` に集める：`theme_access.rs` の全部（`CHECK`、`MINUS`、`field_fill`、`stroke_glyph`、`stroke_loader`、`wide_focus_ring` など）、`button.rs` の `Look`・`metrics`・`trigger_style`、`list_popup.rs` の全部、`layers.rs` の全部、`choice.rs` の `toggle_style`、`parts.rs` の `pressable`・`sized_icon`。元の file を消す。T8 で token の読みを変数にする（helper が `p: &Tokens` を引数に取っていたら引数を消し、本体で変数を読む）。
  2. recipe の関数の行き先（関数名は今のまま。3.0.3 の R2 の署名に直す）：

     | 今の場所の関数 | 行き先 | U recipe 表の setter（今の `UiRecipes` の setter） |
     |---|---|---|
     | `recipes.rs`：`button`、`button_content`、`button_busy`、`button_icon` | `recipes_controls_button.rs` | `.button`、`.button_content`、`.button_busy`、`.button_icon` |
     | `recipes.rs`：`switch_metrics`、`toggle_row`、`checkbox`、`switch`、`edge`、`invalid_edge`、`toggle_edge`、`draw_checkbox`、`draw_switch` | `recipes_controls_check.rs` | `.checkbox`、`.switch` |
     | `recipes.rs`：`field`、`text_input`、`text_area`；`recipes_field.rs`：`field` | `recipes_controls_field.rs` | `.text_input`、`.text_area`、`.field` |
     | `recipes_controls.rs`：`toggle`、`radio_group`、`slider`、`splitter`、`split`、`progress`、`disclosure` | `recipes_controls.rs`（同じ名前の file を書き直す） | 同名の setter |
     | `recipes_controls.rs`：`toast` | `recipes_feedback.rs`（新。K17 が続きを足す） | `.toast` |
     | `recipes_popup.rs`：`select`、`menu`、`tooltip`、`tooltip_arrow` | `recipes_overlays.rs` | `.select`、`.menu`、`.tooltip` の `surface`・`arrow` |
     | `recipes_popup.rs`：`tabs` | `recipes_navigation.rs`（新） | `.tabs` |
     | `recipes_dialog.rs`：`dialog` と helper | `recipes_overlays_dialog.rs` | `.dialog` |
     | `recipes_menu.rs` の recipe（`MenuVariant` は K15 で移した） | `recipes_overlays.rs` | `.menu` の中 |
     | `recipes_command.rs`：`command_palette` | `recipes_overlays_command.rs` | `.command_palette` |
     | `recipes_combobox.rs`：`combobox` | `recipes_overlays_combobox.rs` | `.combobox` |
     | `recipes.rs`：`hover_card_surface` | `recipes_overlays.rs` | `.hover_card` の `surface` |
     | `recipes.rs`：`skeleton`、`spinner`、`kbd` | `recipes_feedback.rs` | `.skeleton`、`.spinner`、`.kbd` |
     | `recipes.rs`：`carousel_*` | `recipes_collections.rs`（新。K17 が続きを足す） | `.carousel` の各 part |

     1 file が 400 code 行を越えたら `recipes_<group>_<部品>.rs` に分ける（3.0.1 の規則 2）。
  3. 各 file の末尾に R4 の `install`。`recipes_controls_button.rs`・`recipes_controls_check.rs`・`recipes_controls_field.rs` の `install` は `recipes_controls.rs` の `install` から順に呼ぶ（`pub(crate) fn install(theme: Theme) -> Theme { let theme = crate::recipes_controls_button::install(theme); .. theme.with(toggle()).with(..) }`）。overlays も同じ。`recipes_feedback.rs`・`recipes_collections.rs`・`recipes_navigation.rs` の `install` は、この unit の分だけを入れる。
  4. `theme.rs` の `build` に `let theme = crate::recipes_controls::install(theme);`、`crate::recipes_overlays::install`、`crate::recipes_collections::install`、`crate::recipes_navigation::install`、`crate::recipes_feedback::install` を R5 の順に足す（`recipes_collections`・`navigation`・`feedback` の行は、K17 で中身が増えても 1 行のまま）。
  5. `recipes.rs` と、中身を移し終えた `recipes_popup.rs`、`recipes_dialog.rs`、`recipes_menu.rs`、`recipes_command.rs`、`recipes_combobox.rs`、`recipes_field.rs`、`button.rs`、`theme_access.rs`、`list_popup.rs`、`layers.rs`、`choice.rs`、`parts.rs` を消す。
  6. `lib.rs` に新しい file の `mod` 行を足す（`pub use` は無い）。
- **消すもの**：上の手順 5 の file、`UiRecipes` を組む `recipes()`・`build()` と `OnceLock`。
- **状態の閉じ方**：なし。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：`ls crates/kivact-ui-basic/src/recipes.rs crates/kivact-ui-basic/src/recipes_popup.rs` が「無い」。`rg -n "UiRecipes|Recipes\b" crates/kivact-ui-basic/src` が何も出さない（U の新しい型の名前は `*Theme`）。上の command が通る。

## K17：basic の recipe 2（collections・navigation・feedback・metrics・chat・input・calendar・code）

- **wave / lane / 前提**：W9 / K1 / K16
- **所有**：K15 と同じ
- **目的**：分野の file にある recipe と残りの `recipes_*.rs` を `<Name>Theme` にし、recipe の置き場を `recipes_<group>.rs` だけにする（KIT-06）。
- **手順**：
  1. 行き先（関数は今の名前。R2・R3 の形にする）：

     | 今の場所 | 行き先 | U recipe 表の setter |
     |---|---|---|
     | `recipes_collections.rs`：`list`、`data_table` | `recipes_collections.rs`（K16 の file に足す） | `.list`、`.data_table` |
     | `table.rs`：`recipes` | `recipes_collections.rs`（関数名 `table`） | `.table` |
     | `item.rs`：`recipes` | `recipes_collections.rs`（関数名 `item_list`） | `.item_list` |
     | `recipes_groups.rs`：`toggle_group`、`button_group` | `recipes_collections.rs` | `.toggle_group`、`.button_group` |
     | `line_tabs.rs`：`is_line`、`list`、`tab`、`panel` | `recipes_navigation.rs`（K16 の `tabs` から呼ぶ） | `.tabs` の中 |
     | `breadcrumb.rs`、`navigation_menu.rs`、`pagination.rs` の `recipes` | `recipes_navigation.rs`（関数名 `breadcrumb`、`navigation_menu`、`pagination`） | 同名の setter |
     | `badge.rs`：`recipe` | `recipes_feedback.rs`（関数名 `badge`） | `.badge` |
     | `avatar.rs`：`avatar`、`image`、`group`、`member`、`overflow`、`status` | `recipes_feedback_avatar.rs` | `.avatar`、`.avatar_image`、`.avatar_group`、`.avatar_group_member`、`.avatar_group_overflow`、`.avatar_status` |
     | `empty.rs`：`recipes`、`alert.rs`：`recipes` | `recipes_feedback.rs`（関数名 `empty_state`、`alert`） | `.empty_state`、`.alert` |
     | `recipes_metrics.rs` の全部、`typography.rs` の `heading` | `recipes_metrics.rs`（書き直す） | `.stat`、`.sparkline`、`.heading` など今の setter |
     | `chat.rs`：`recipes`、`chat_colors.rs`：`colors`、`message_scroller.rs`：`recipes` | `recipes_chat.rs` | `.chat_message`、`.chat_thread` |
     | `card.rs`：`recipes`、`input_group.rs`：`button`、`button_icon`、`control`、`recipes` | `recipes_input.rs`（`card` は `recipes_collections.rs`） | `.card`、`.input_group` |
     | `recipes_calendar.rs`：`calendar` | `recipes_calendar.rs`（書き直す） | `.calendar` |
     | `code_block/view.rs` の `recipes` | `recipes_code.rs` | `.code_block` |

  2. recipe が `look.variant::<CardLook>()` などの消えた束を読んでいたら、V4 で分けた型を一つずつ読む（`card`：`s.variant::<CardSize>()` と `s.variant::<CardHeaderBordered>().map_or(false, |b| b.0)`。`item_list`：`ItemSize`。`code_block`：`CodeBlockSize`。`input_group` の button：`InputGroupButtonSize`）。束の既定値（`CardSize::Default`、`false` など）を `unwrap_or` の値にする。`AlertLook` の読みは `AlertTone`、`BubbleVariant` は `ChatBubbleVariant`。
  3. 各 `recipes_<group>.rs` の `install` に R4 の `.with(..)` を足し、`theme.rs` の `build` に、まだ無い group（metrics、chat、input、calendar、code）の `install` を R5 の順に足す。
  4. 中身が無くなった file を消す：`table.rs`、`item.rs`、`recipes_groups.rs`、`line_tabs.rs`、`breadcrumb.rs`、`navigation_menu.rs`、`pagination.rs`、`badge.rs`、`avatar.rs`、`empty.rs`、`alert.rs`、`chat.rs`、`chat_colors.rs`、`message_scroller.rs`、`card.rs`、`input_group.rs`、`code_block/view.rs`（`code_block/` の他の file は K34）、`recipes_calendar.rs` の古い形。`typography.rs` は `heading` だけを移し、部品は K19 が変換する。
  5. `lib.rs` に新しい file の `mod` 行を足す。
- **消すもの**：上の手順 4 の file。
- **状態の閉じ方**：なし。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：`rg -ln "fn [a-z_]+\(\w*: &Theme" crates/kivact-ui-basic/src | rg -v "recipes_|recipe_parts"` が何も出さない（recipe の関数が `recipes_*` と `recipe_parts` の外に無い）。上の command が通る。

## K18：basic の部品 1（`#[component]`）

- **wave / lane / 前提**：W9 / K1 / K17
- **所有**：K15 と同じ
- **目的**：basic の手書きの builder を `#[component]` にする（DEC-03、3.0.5）。
- **手順**：次の file の公開の部品を 3.0.5 の C1〜C9 で変換し、`lib.rs` に `mod` 行と `pub use` 行を足す（4.4.1 節の表の「部品」の行の名前）：
  - `attachment.rs`：`attachment`（`Attachment` を生成）、`attachment_group`、`attachment_action`（今 `View` を返す関数なら部品、data を返すなら関数のまま。C8）。`AttachmentState` は props の enum としてこの file に残す。
  - `card_header.rs`：`card_header`。
  - `field.rs`：`field_set`、`field_group`、`field_content`、`field_separator`（`FieldSet`・`FieldSeparator` を生成）。
  - `input_otp.rs`：`input_otp`。今 `input_otp(label, value)` のように名前を位置引数で取っていたら C6 で外す（呼出し側 `.label(..)`）。`OtpPattern` は props の型として残す。
  - `marker.rs`：`marker`。
  - `menubar.rs`：`menubar`。
  - `native_select.rs`：`native_select`。名前の位置引数は C6 で外す。
  - `scroll_area.rs`：`scroll_area`。
- **消すもの**：変換した builder の手書きの struct・setter・`impl Element`・`impl IntoView`。
- **状態の閉じ方**：C10。新しい印は要らない。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：`rg -n "impl (kivact::view::)?(Element|IntoView) for" crates/kivact-ui-basic/src/{attachment,card_header,field,input_otp,marker,menubar,native_select,scroll_area}.rs` が何も出さない。上の command が通る。

## K19：basic の部品 2（`#[component]`）

- **wave / lane / 前提**：W9 / K1 / K18
- **所有**：K15 と同じ
- **目的**：K18 の続き（DEC-03）。
- **手順**：次の file を 3.0.5 で変換し、`lib.rs` に足す：
  - `primitives.rs`：`label`（`Label`）、`separator`（`Separator`）、`item_separator`、`kbd_group`、`table_caption`、`breadcrumb_ellipsis`。
  - `typography.rs`：`paragraph`、`lead`、`large`、`small`、`muted`、`blockquote`、`inline_code`、`bullet_list`（C7・C9。今 `text: impl IntoText` の引数は `Value<String>`）。`pub(crate) const MONO` は `recipe_parts.rs` に移す。
  - `questionnaire.rs`：`questionnaire`。`Questionnaire`（生成）、`QuestionnaireAnswer`・`QuestionnaireChoice`・`QuestionnaireStep`（data）はそのまま。
  - `trace.rs`：`trace`、`trace_line`（`TraceLine`）、`trace_collapsible`。`pub(crate) fn content` は private の helper のまま。
  - `empty.rs`・`item.rs` に部品が残っていれば（`rg -n "^pub fn" crates/kivact-ui-basic/src/{empty,item}.rs`）同じく変換する。
- **消すもの**：変換した builder の手書きの部分。
- **状態の閉じ方**：C10。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：`rg -n "impl (kivact::view::)?(Element|IntoView) for" crates/kivact-ui-basic/src` が `chart/` と `code_block/` の file だけを出す。上の command が通る。

## K34：basic の chart・code_block・sidebar・icon（中核の追加を使う）

- **wave / lane / 前提**：W10 / K1 / K19、K07、K08、K10、K11
- **所有**：`crates/kivact-ui-basic/`（全体）、`crates/kivact-ui-basic/Cargo.toml`、`Cargo.lock`
- **目的**：中核の追加が要る残りを変換し、basic を完成させる：chart の目盛りを `kivact-plot` と共有（KIT-10）、構文色を中核の `SyntaxKind` に（KIT-05）、sidebar を中核の変数の set と `SidebarTheme` の part に（KIT-10）、icon を `icons!` と `thread_local!` に（KIT-08）。
- **手順**：
  1. `Cargo.toml` の `[dependencies]` に `kivact-plot = { path = "../kivact-plot", version = "0.1.0" }` を足す（4.1 節の 1、parent 採用）。
  2. chart：`src/chart/scale.rs` の `use crate::nice::round_step;` を `use kivact_plot::nice::round_step;` にする。`text_width`（字幅の見積り）は残す（hit test は描画の外で文字を測れないため。KIT-10 の「測った幅に置き換える」は `kivact-plot` の `plot_area` が描画の中で測る方法で、chart の hit test には使えない。4.2 節の 5、決定済み）。`chart.rs`・`chart/*.rs` の `Chart` builder と `chart(config, rows)` を `#[component]` にする（3.0.5）。`ChartConfig`、`ChartKind`、`ChartRow`、`ChartSeries` は data のまま。色の `tokens.chart[i]` は `[CHART_1, .., CHART_5][i % 5]`（T8）。
  3. basic の theme に plot と構文の色を set する（`theme.rs` の `set_core` の最後）：
     - `kivact_plot::color::SERIES_1_VAR`〜`SERIES_5_VAR` ← `CHART_1`〜`CHART_5`、`GRID_VAR` ← `BORDER`、`AXIS_VAR` ← `MUTED_FOREGROUND`。
     - `kivact::ui::theme::syntax::{KEYWORD,STRING,NUMBER,COMMENT,FUNCTION,TYPE,MACRO,LIFETIME,ATTRIBUTE}_VAR` ← 今の `CodeColors::new(mode)` の `keyword, string, number, comment, function, type_name, macro_call, lifetime, attribute` の値（light・dark の `pick`）。`OPERATOR_VAR`、`PUNCTUATION_VAR`、`VARIABLE_VAR`、`PROPERTY_VAR` ← `FOREGROUND`（今 basic は Plain・Punctuation を前景色のまま描く）。`CONSTANT_VAR` ← number の値、`TAG_VAR` ← lifetime の値。
     - sidebar：`kivact::ui::theme::sidebar::{BACKGROUND,FOREGROUND,PRIMARY,PRIMARY_FOREGROUND,ACCENT,ACCENT_FOREGROUND,BORDER,RING}_VAR` ← `SIDEBAR`、`SIDEBAR_FOREGROUND`、`SIDEBAR_PRIMARY`、`SIDEBAR_PRIMARY_FOREGROUND`、`SIDEBAR_ACCENT`、`SIDEBAR_ACCENT_FOREGROUND`、`SIDEBAR_BORDER`、`SIDEBAR_RING`、`PAGE_VAR` ← `BACKGROUND`。
  4. code_block：`src/code_block/lines.rs` の `CodeTone` を消す。`CodeSpan { text, tone: CodeTone }` を `CodeSpan { text: String, kind: Option<SyntaxKind> }`（`None` は色を付けない）にし、`CodeSpan::new(text, kind: Option<SyntaxKind>)`。`src/code_block/rust.rs` の tokenizer の `CodeTone::X` を `Some(SyntaxKind::X)` に（`Plain` → `None`、`Punctuation` → `Some(SyntaxKind::Punctuation)`、`Type` → `Type`、`Macro` → `Macro`、`Lifetime` → `Lifetime`、`Attribute` → `Attribute`、他は同名）。`src/code_block/highlight.rs` の `tone_color` を消し、`rust_lines` の署名を `pub fn rust_lines(lines: &Vec<CodeLine>, text: &str, index: &usize) -> (Vec<StyledRange>, usize)` にし、本体は span を `(range, kind)` の列にして `kivact::ui::syntax_ranges(text, &tokens)` を返す（`kind` が `None` の span は列に入れない）。呼出し側（`code_block(..).highlight(..)` を使う所。catalog と docs）は `highlight(lines, rust_lines)` になる。
  5. sidebar：今の basic の sidebar の公開名（`Sidebar*`、`sidebar*`）は K11 で中核へ移った。`lib.rs` に sidebar の行が残っていれば消す。basic の Lyra の形は `SidebarTheme` で入れる：`src/recipes_navigation_sidebar.rs`（新）に `pub(crate) fn sidebar() -> SidebarTheme` を置き、K11 で中核の既定にした値と違う basic の今の値を part で差し替える。差し替える part と値（今の basic の `sidebar/menu.rs`・`panel.rs` の式。`git show 2b8df5ac:crates/kivact-ui-basic/src/sidebar/menu.rs` で読む）：`menu_button`（`button_style` の全部：`font_size(12)`、`padding(8)`、`gap(8)`、`border_radius(0)`、active の間 `font_weight(500)`、`focus_visible(style().outline(2, SIDEBAR_RING))`）、`sub_button`（`sidebar_menu_sub_button` の style：`font_size(12)`、`border_radius(0)`）、`group_label`（`font_size(12)`、`padding_x(8)`、`MUTED_FOREGROUND` の 70% の色）、`header`・`footer`（`padding(8)`、`gap(8)`）。recipe は R2 の形で、`base` に今の値を足す（`|_t, _s, base| base.font_size(12)` など。状態で分かれる式は `s.active` などで分ける）。`recipes_navigation.rs` の `install` に `.with(crate::recipes_navigation_sidebar::sidebar())` を足す。basic の README と crate doc の sidebar の説明は「`kivact::ui::sidebar` を basic の theme が描く」にする。
  6. icon：`src/icons.rs` を 3.0.7 の icon の形にする（`ICON_NAMES` を出す。別名の icon：`("dot", RADIO_DOT)`、`("menu-radio", check.svg)`、`toast-info`・`toast-success`・`toast-warning`・`toast-danger`・`toast-accent` は今の `set()` のまま `build` で足す）。`OnceLock` と `#[expect(clippy::disallowed_types)]` を消す。
  7. `lib.rs`：K34 で変換した module（`chart`、`code_block`）の `mod` と `pub use` を足し、4.4.1 節の表の「残す」「改名」の名前が全部 `pub use` にあることを確かめる。crate doc の最後の段落に message の key の一覧（`basic.questionnaire.*` など、`rg -o '"basic\.[a-z_.]+"' crates/kivact-ui-basic/src | sort -u`）を書く。
  8. 3.0.6 の S4：`cargo clippy .. --lib -- -D warnings`（`-A dead_code` なし）で dead の item が出たら消す。
- **消すもの**：`CodeTone`、`CodeColors`（K15 で消えていなければ）、`tone_color`、basic の `icons.rs` の `OnceLock`、`nice` の参照。
- **状態の閉じ方**：なし。
- **試験**：なし（K35）。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-ui-basic
  cargo check -p kivact-ui-basic --lib
  cargo clippy -p kivact-ui-basic --lib -- -D warnings
  cargo xtask check --no-write --crate kivact-ui-basic
  ```

- **完了条件**：`rg -n "OnceLock|CodeTone|CodeColors|crate::nice|ColorName|Tokens" crates/kivact-ui-basic/src` が何も出さない。上の command が通る。

## K35：basic の試験・catalog・README（TL-13、KIT-12、KIT-16、KIT-20）

- **wave / lane / 前提**：W10 / K1 / K34、K14
- **所有**：`crates/kivact-ui-basic/`（`tests/`、`README.md`）、`examples/kivact-ui-basic/`（全体）、`Cargo.lock`
- **目的**：basic の試験を新しい API で戻し、TL-13 の判定で整理する。catalog を catalog-shell に移す。README を雛形にする。
- **手順**：
  1. root `Cargo.toml` は変えない（kit・catalog は workspace の member のまま。gate の `--exclude` が外していたのをやめるだけ。INC-09）。
  2. 試験（3.0.8）：`crates/kivact-ui-basic/tests/consumer/` を 4.6 節の basic の表に従って消す・残す・直す。`plot.rs` は K10 で移った。「残す」の試験は新しい API（`#[component]` の呼び方、`.label(..)`、`.variant(AlertTone::Destructive)` など）に直し、期待値は変えない。`tests/consumer.rs` の `#[path]`・`mod` の行を残る file だけにする。
  3. catalog を catalog-shell に移す（3.0.8 の catalog の形）：
     - `Cargo.toml`：`[dependencies]` に `catalog-shell = { path = "../catalog-shell" }`、`[dev-dependencies]` に `catalog-shell = { path = "../catalog-shell", features = ["test"] }` を足す。features `gpu`・`complex-scripts` と package 名 `kivact-ui-basic-catalog` は変えない（`dev/bench/run.sh` が使う）。
     - page の file は `src/docs/<slug>.rs` のまま（module 名 `docs` を変えない。K06 が policy の例外の `item` を `docs::pages` にした）。各 file の `pub(crate) fn page() -> Page { Page { slug, name, description, usage, source: include_str!("<slug>.rs"), align, examples: vec![Example::new(..)] } }` を `pub(crate) fn page() -> Page { Page::new("<slug>", "<Name>", "<category>", include_str!("<slug>.rs")).description("..").usage("..").align(..).example(Example::new(..)) }` にする。`category` は 4.7 節の basic の表。
     - `src/docs.rs` の `PAGES` の `LazyLock` をやめて `pub fn pages() -> Vec<Page> { vec![accordion::page(), ..] }`（今の `PAGES` の順）にする（file 名と module 名は `docs` のまま）。`#[expect(clippy::disallowed_types)]` を消す。
     - 文言を英語にする：各 page の `description`、`Example` の説明、demo の中の日本語の文字（ボタンの label など）を英語に訳す。`usage` の import の例は新しい API の path にする。
     - demo の中の API を新しい API に直す（3.0.2〜3.0.5 と同じ置換：`basic::ColorName::X` → `basic::X`、`*Look` → 分けた variant、`basic::plot` → `kivact_plot::plot`、`basic::sidebar*` → `kivact::ui::sidebar*`）。toast の icon：annex U が `Toast::icon`（`ViewFn`）を `Toast::icon_name(Option<&'static str>)`（icon set の名前）にした。`src/docs/toast.rs` の "Loading toast" の例の `.icon(|| kivact::ui::spinner().with_node(|node| node.role(Role::Generic)))` を `.icon_name("loader-circle")`（basic の icon set にある Lucide の `loader-circle`。回らない静止の icon になる）にし、その上の comment「The spinner is a status of its own; ...」を消す。`plot.rs` の page の `usage` は `use kivact_plot::{plot, PlotModel, ..};`。catalog の `Cargo.toml` の `[dependencies]` に `kivact-plot = { path = "../../crates/kivact-plot" }` を足す。
     - `src/lib.rs`：`mod docs;`、`pub use docs::pages;`、`pub fn theme(mode: ThemeMode) -> Theme { kivact_ui_basic::theme(mode).with_fonts(catalog_shell::FONTS.iter().copied()).set(&kivact::style::font::FAMILY_VAR, "Inter, 'Noto Sans JP', sans-serif") }`、`pub const THEME: CatalogTheme = CatalogTheme::new(theme);`、`pub const START: &str = "button";`（K1 の計測が今と同じ画面で始まる）。
     - `src/main.rs`：3.0.8 の形（title "Basic"）。
     - 消す：`src/shell.rs`、`src/page.rs`、`src/model.rs`、`src/look.rs`、`src/fonts.rs`、`src/pages.rs`（今の `PageInfo` の写し。`docs::pages` が代わる）。
     - `tests/catalog.rs`：次の 3 つの試験にする。今の `theme_toggle_keeps_the_page_and_the_input_state` と `search_filters_the_list_and_the_keyboard_walks_it` は catalog-shell の振る舞いの試験なので、`catalog_shell::check::mount` を使って本体を写し、`catalog_with_mode(mode)` を `check::mount("Basic", pages, THEME, mode)` にする。全 page の walker は：

       ```rust
       #[test]
       fn every_page_opens_from_the_list_and_renders_cleanly() {
           catalog_shell::check::every_page_opens_cleanly("Basic", pages, THEME, &["carousel"]);
       }

       #[test]
       fn every_page_character_is_in_the_catalog_fonts() {
           catalog_shell::check::every_page_character_is_in_the_fonts(pages, THEME);
       }
       ```

       今の `mount`、`mount_in`、`layout_check`、`reveal`、`open`、`visit_every_page`、`bundled`、`glyph_cell`、`cell_pixels` の関数を消す（catalog-shell に移った）。
  4. `crates/kivact-ui-basic/README.md` を 3.0.10 の雛形にする（「Migrating from the previous release」（170 行目〜）、`<!-- kivact-recipes -->` の一覧を消す）。
  5. `examples/kivact-ui-basic/README.md` を 5 行にする（何の catalog か、`cargo run -p kivact-ui-basic-catalog`、英語）。
- **消すもの**：上の catalog の file、basic の 4.6 節の「消す」の試験、README の 2 つの節。
- **状態の閉じ方**：なし。
- **試験**：`crates/kivact-ui-basic/tests/consumer/` の「残す」の試験（4.6 節）、`examples/kivact-ui-basic/tests/catalog.rs` の 4 つ（`every_page_opens_from_the_list_and_renders_cleanly`、`every_page_character_is_in_the_catalog_fonts`、`theme_toggle_keeps_the_page_and_the_input_state`、`search_filters_the_list_and_the_keyboard_walks_it`）。
- **確認（tier a）**：3.0.9 の最後の unit と catalog の 7 行（`<kit>` = `basic`）。 加えて pending の検査を strict で通す（INC-14）：`cargo xtask check --no-write --strict --only hand-element,element-shadow --crate kivact-ui-basic`。
- **完了条件**：`rg -c '[\p{Hiragana}\p{Katakana}\p{Han}]' examples/kivact-ui-basic` が何も出さない。`rg -n "Migrating from the previous release|kivact-recipes" crates/kivact-ui-basic/README.md` が何も出さない。上の command が通る。

---

### 3.2 lane K2：pop（K20、K21、K36、K37）

## K20：pop の土台と recipe（theme 変数・theme・variant・hue・recipe）

- **wave / lane / 前提**：W9 / K2 / W8 の完了
- **所有**：`crates/kivact-ui-pop/`
- **目的**：pop を compile の対象に戻し、骨格に揃える（KIT-06、KIT-07）。4 つの `XHue` の newtype と `WeekendHue` を消し、`Hue` を直接の variant にする（KIT-04 の hue の部分）。recipe の literal の font（`"Barlow, sans-serif"`）をやめて theme の font を使う（KIT-13 の pop の部分）。recipe の評価ごとの色の計算を theme の set に移す（KIT-14）。
- **手順**：
  1. root `Cargo.toml` は変えない（kit・catalog は workspace の member のまま。gate の `--exclude` が外していたのをやめるだけ。INC-09）。
  2. `src/color.rs`（T1〜T5。名前の文字列は `"pop.<field>"`）：
     - `Tokens` の色の field：`BACKGROUND`、`SURFACE`、`SURFACE_MUTED`、`SURFACE_ACTIVE`、`LINE`、`LINE_STRONG`、`TEXT`、`TEXT_MUTED`、`TEXT_FAINT`、`BLUE`、`BLUE_DEEP`、`CYAN`、`YELLOW`、`PINK`、`MINT`、`RED`、`ORANGE`、`PURPLE`、`NAVY`。
     - `hue_backgrounds[i]`・`hue_foregrounds[i]`（`Tokens::hue` の index の順 Blue、Mint、Yellow、Pink、Red、Gray）：`BLUE_BACKGROUND`、`MINT_BACKGROUND`、`YELLOW_BACKGROUND`、`PINK_BACKGROUND`、`RED_BACKGROUND`、`GRAY_BACKGROUND` と `BLUE_FOREGROUND`〜`GRAY_FOREGROUND`。
     - `tokens.rs` の引数なしの関数（`accent_gradient_end`、`accent_text`、`pink_gradient_end`、`danger_gradient_end`、`control_shadow`、`subtle_shadow`、`glass_surface`、`glass_border`、`glass_hover`、`selected_track`、`sparkline_fill`、`yellow_gradient_end`、`red_gradient_end`、`overlay_scrim`、`progress_stripe`）：同じ名前の大文字の変数（`ACCENT_GRADIENT_END` など）。値は関数の今の値（light と dark で同じ）。呼出し `crate::tokens::accent_gradient_end()` は `ACCENT_GRADIENT_END` にする。
     - 引数のある関数（`avatar_halo(hue)`、`avatar_glow(hue)`、`avatar_gradient_start(hue)`、`avatar_gradient_end(hue)`）：今の本体のまま `color.rs` の `pub(crate) fn` に移す。
     - KIT-14：`button.rs` の `p.blue.mix_in(p.cyan, 0.3, t)` は新しい変数 `BLUE_GRADIENT_END`（`set_colors` で `rgb(<blue>).mix(rgb(<cyan>), 0.3, &theme)` を set。S の `mix` の署名）にし、recipe は `BLUE_GRADIENT_END` を読む。`rg -n "mix_in|\.mix\(" crates/kivact-ui-pop/src` が出す他の式で、引数がすべて pop の色と定数のものも、同じく `<結果の意味>` の名前の変数にする（変数名は「式の一つ目の色の名前」＋`_` ＋「用途」。用途は式の置かれた recipe の関数名：例 `SURFACE_ACTIVE_BUTTON_HOVER`）。引数に state（hover など）が入る式は recipe に残し `t.get(&X_VAR)` で読む。
  3. `src/size.rs`（T3）：`PANEL_PADDING`、`CONTROL_HEIGHT`、`CONTROL_RADIUS`、`PANEL_RADIUS`（名前の文字列 `"pop.<field>"`）と `set_sizes`（今の値。mode で変わらない）。
  3a. 別名の fallback（T1a）：`SURFACE` ← `color::SURFACE`、`SURFACE_ACTIVE` ← `color::SURFACE_ALT`、`TEXT` ← `color::TEXT`、`TEXT_MUTED` ← `color::TEXT_MUTED`、`BLUE` ← `color::ACCENT`、`LINE` ← `color::BORDER`、`RED` ← `color::DANGER`、`MINT` ← `color::SUCCESS`、`YELLOW` ← `color::WARNING`（今の `theme.rs` の `Palette` の token の行）。他は light の literal。
  4. `Hue`（`tokens.rs` → `variants.rs`）：`#[derive(Prop)]`（C09 が付けた。V7）を保つ。`impl Variant for Hue {}` を `variants.rs` に置く。`Tokens::hue(&self, hue)` を `impl Hue { pub fn colors(self) -> (Color, Color) }`（`BLUE_BACKGROUND, BLUE_FOREGROUND` などの組を返す）に、`Tokens::accent(&self, hue)` を `impl Hue { pub fn accent(self) -> Color }`（Blue → `BLUE`、Mint → `MINT`、Yellow → `YELLOW`、Pink → `PINK`、Red → `RED`、Gray → `TEXT_MUTED`）にする。呼出し `p.hue(h)` → `h.colors()`、`p.accent(h)` → `h.accent()`。
  5. `src/variants.rs`（新）：
     - そのまま移す：`ButtonTone`（V2：中核に `ButtonVariant` があるので Tone のまま）、`Slant`・`Striped`（W9 の間はここ。K36 で消す）、`BadgeVariant`、`SectionNumber`、`DiamondSteps`、`Segments`、`Hue`。
     - 改名：`PopBubble` → `ChatBubbleVariant`、`SpinnerLook` → `SpinnerVariant`。
     - 束の分解：`AvatarLook { size: Value<f64>, hue: Value<f64>, halo: Value<bool>, online: Value<bool> }` → `pub struct AvatarSize(pub f64);`、`pub struct AvatarHue(pub f64);`、`pub struct AvatarHalo(pub bool);`、`pub struct AvatarOnline(pub bool);`（V4）。recipe は今の `AvatarLook::default()` の値を既定にする。
     - 消す（V5）：`BadgeHue`、`ItemHue`、`StatHue`、`ProgressHue`。recipe の `s.variant::<BadgeHue>().map(|h| h.0.get())` のような読みは `s.variant::<Hue>()` にする（`Element::variant` が `Value` を解決して渡す）。
     - `WeekendHue` → `pub struct CalendarWeekdayHues(pub [Option<Hue>; 7]);`（月曜から日曜の順。`#[derive(Clone, Copy, Debug, Default, PartialEq)]`、`impl Variant`）。calendar の recipe の `s.variant::<WeekendHue>().and_then(|w| (w.0)(weekday))` は `s.variant::<CalendarWeekdayHues>().and_then(|w| w.0[weekday.index_from_monday()])`（`kivact::i18n::Weekday` の月曜からの index の関数。`rg -n "pub fn .*monday|fn index" crates/kivact-i18n/src/date.rs` の名前を使う。無ければ `match` で 0〜6 を返す private 関数を `recipes_calendar.rs` に書く）。
  6. `src/theme.rs`：3.0.7 の形（pop は options なし）。`set_core`（T6・T7）：今の `Palette` の各 field の値（`surface: p.surface` → `color::SURFACE_VAR` ← `SURFACE` など。literal の `rgba(..)` はそのまま）、`Spacing`、`Radii { sm: 4, md: 8, lg: 14 }`、`Typography { size: 14, size_small: 12, size_large: 20, line_height: Scale(1.5) }`、`with_focus_ring(3.0)`、`with_control_height(36.0)`、`ui_tokens` の全行（`indicators.reduced_period_ms`、`code_block.font_size`・`line_height`、`button_group.*`、`toolbar.*`、`sparkline`、`calendar`）を U 変数表の変数へ。
  7. recipe（3.0.3）：file の行き先と、各 file の `install` の中身（今の `recipes.rs` の `build` の setter の順）：

     | 今の file | 行き先 |
     |---|---|
     | `recipes.rs` の `button`、`button_icon`、`checkbox`、`switch`、`text_input`、`text_area`；`recipes_extra.rs` の `button_content`、`button_busy`、`radio_group`、`slider`；`recipes_controls.rs`；`recipes_small.rs`（`toggle`、`progress`、`disclosure`）；`recipes_field.rs`；`button.rs` の `button_style`；`form.rs` の `input_style`；`choice.rs` の `check_style`、`choice_option`、`choice_list` | `recipes_controls.rs`（400 行を越えるので `recipes_controls_button.rs`・`recipes_controls_check.rs`・`recipes_controls_field.rs` に分ける。`button_style`・`input_style`・`check_style` は使う file に置き、二つ以上が使えば `recipe_parts.rs`） |
     | `recipes_extra.rs` の `select`、`menu`、`dialog`、`tooltip`；`choice.rs` の `select_option`、`select_list`；`recipes_palette.rs` | `recipes_overlays.rs` |
     | `recipes_collections.rs`、`recipes_items.rs`（`item_list`、`table`、`tree`）、`recipes_groups.rs` | `recipes_collections.rs` |
     | `recipes_extra.rs` の `tabs`；`tabs.rs` の `item_label`；`recipes_navigation.rs` | `recipes_navigation.rs` |
     | `recipes_extra.rs` の `toast`；`recipes_feedback.rs`（`empty_state`、`alert`、`card`）；`recipes_avatar.rs`；`recipes_badge.rs`；`recipes_indicators.rs` | `recipes_feedback.rs`（400 行を越えたら `recipes_feedback_avatar.rs`、`recipes_feedback_badge.rs` に分ける） |
     | `recipes_metrics.rs` | `recipes_metrics.rs` |
     | `recipes_chat.rs`（`message`、`thread`、`composer`、`typing`）；`code` は `recipes_code.rs` | `recipes_chat.rs`、`recipes_code.rs` |
     | `recipes_feedback.rs` の `input_group`；`recipes_input.rs`；`recipes_files.rs` | `recipes_input.rs` |
     | `recipes_calendar.rs` | `recipes_calendar.rs` |

     中身を移した `recipes.rs`、`recipes_extra.rs`、`recipes_small.rs`、`recipes_field.rs`、`recipes_items.rs`、`recipes_groups.rs`、`recipes_avatar.rs`、`recipes_badge.rs`、`recipes_indicators.rs`、`recipes_palette.rs`、`recipes_files.rs`、`button.rs`、`form.rs`、`choice.rs` を消す（`tabs.rs` は `tab_label` の部品が残るので K21）。
  8. KIT-13（pop）：`rg -n 'font_family\("Barlow' crates/kivact-ui-pop/src` の 3 か所の `.font_family("Barlow, sans-serif")` を消す（theme の font を継ぐ）。
  9. `src/lib.rs`：crate doc（今の 1 段落目と 2 段落目。`Tokens` の文を消す）、`mod color; mod icons; mod recipe_parts;`（作ったとき）`mod recipes_*; mod size; mod theme; mod variants;`、`pub use color::{..}`、`pub use size::{..}`、`pub use theme::theme;`、`pub use variants::{..}`。分野の部品の module はまだ書かない。
- **消すもの**：`Tokens`、`ColorName`、`tokens.rs`、`BadgeHue`、`ItemHue`、`StatHue`、`ProgressHue`、`WeekendHue`、`PopBubble`、`SpinnerLook`、`AvatarLook`、上の手順 7 の file、`UiRecipes` と `OnceLock` の `recipes()`。
- **状態の閉じ方**：新しい状態の型なし。
- **試験**：なし（K37）。
- **確認（tier a）**：3.0.9 の途中の unit（`<kit>` = `pop`）。
- **完了条件**：`rg -n "ColorName|Tokens|BadgeHue|ItemHue|StatHue|ProgressHue|WeekendHue|PopBubble|SpinnerLook|AvatarLook|UiRecipes" crates/kivact-ui-pop/src` が何も出さない。上の command が通る。

## K21：pop の分野の部品（`#[component]`）

- **wave / lane / 前提**：W9 / K2 / K20
- **所有**：`crates/kivact-ui-pop/`
- **目的**：pop の分野の部品を `#[component]` にする（DEC-03）。
- **手順**：3.0.5 で次を変換し、`lib.rs` に `mod` 行と `pub use` 行を足す：`chips.rs`（`chip_list` → `ChipList`、`removable_chip` → `RemovableChip`）、`display.rs`・`display/feedback.rs`（`empty_illustration`。`display.rs` が mod 宣言だけなら `display/feedback.rs` を `src/empty_illustration.rs` に移して `display.rs` を消す）、`messaging.rs`（`thread` → `Thread`）、`notifications.rs`（`notification_menu` → `NotificationMenu`。`NotificationItem` は data）、`patterns.rs`（`agenda_item` → `AgendaItem`）、`resize.rs`（`resizable_text_area`。text area の名前を位置引数で取っていれば C6）、`tabs.rs`（`tab_label`）、`timeline.rs`（`timeline_item` → `TimelineItem`）、`upload.rs`（`upload_row`）。`on_*` の handler の名前は C5（`notification_menu` の既読の通知は `on_activate(Fn(K))` か今の名前が `on_press` の形ならそれ）。
- **消すもの**：変換した builder の手書きの部分。
- **状態の閉じ方**：C10。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：`rg -n "impl (kivact::view::)?(Element|IntoView) for" crates/kivact-ui-pop/src` が何も出さない。上の command が通る。

## K36：pop の `Slant`・`Striped` を中核へ、icon（W10）

- **wave / lane / 前提**：W10 / K2 / K21、K07、K08
- **所有**：`crates/kivact-ui-pop/`
- **目的**：pop と game が別々に定義していた `Slant`・`Striped` を中核の `kivact::ui::{Slant, Striped}` にする（KIT-04）。icon を `icons!` と `thread_local!` の icon set にする（KIT-08）。
- **手順**：
  1. `src/variants.rs` の `Slant` と `Striped` の定義と `impl Variant` を消す。`rg -l "\bSlant\b|\bStriped\b" crates/kivact-ui-pop/src | xargs perl -pi -e 's/\bcrate::(Slant|Striped)\b/kivact::ui::$1/g; s/\bcrate::variants::(Slant|Striped)\b/kivact::ui::$1/g'` の後、`use crate::{.., Slant, ..}` の import を `use kivact::ui::{Slant, Striped};` にする。`lib.rs` の `pub use variants::{..}` から 2 つを消す。
  2. `src/icons.rs` を 3.0.7 の icon の形にする（`ICON_NAMES` は出さない）。並べる名前は今の `NAMES` と `IMAGES` の両方にある名前（3.0.7 の最後の段落）。`OnceLock` と `#[expect(clippy::disallowed_types)]` を消す。
  3. 3.0.6 の S4（`-A dead_code` なしで通す）。
- **消すもの**：`pop::Slant`、`pop::Striped`、`NAMES`・`IMAGES` の二重の一覧、`OnceLock`。
- **状態の閉じ方**：なし。
- **試験**：なし（K37）。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-ui-pop
  cargo check -p kivact-ui-pop --lib
  cargo clippy -p kivact-ui-pop --lib -- -D warnings
  cargo xtask check --no-write --crate kivact-ui-pop
  ```

- **完了条件**：`rg -n "struct (Slant|Striped)|OnceLock|const NAMES|const IMAGES" crates/kivact-ui-pop/src` が何も出さない。上の command が通る。

## K37：pop の試験・catalog・README

- **wave / lane / 前提**：W10 / K2 / K36、K14
- **所有**：`crates/kivact-ui-pop/`（`tests/`、`README.md`）、`examples/kivact-ui-pop/`、`Cargo.lock`
- **目的**：TL-13、KIT-12、KIT-16、KIT-20 を pop に当てる。
- **手順**：
  1. root `Cargo.toml` は変えない（kit・catalog は workspace の member のまま。gate の `--exclude` が外していたのをやめるだけ。INC-09）。
  2. 試験：4.6 節の pop の表。
  3. catalog（3.0.8）：今の 13 の section（`shell.rs` の `catalog_sections` の順：overview、tokens、buttons、forms、selection、display、table、feedback、overlays、navigation、messaging、schedule、cards）を、それぞれ一つの page にする。`src/pages/<slug>.rs` の slug は `overview`、`tokens`、`buttons`、`forms`、`selection`、`display`、`table`、`feedback`、`overlays`、`navigation`、`messaging`、`schedule`、`cards`。各 page は `Page::new(slug, <英語の名前>, <4.7 節の category>, include_str!("<slug>.rs"))` と、今の section の関数の中身を `// example: <slug>` … `// end example` の間に置いた一つの `Example`（`Example::new("<slug>", <英語の名前>, "", <slug>_demo)`）。section の関数が shell の signal（`confirm`、`form_dialog`、`command`、`sheet`、`drawer_title`、`notify`）を引数に取っていたら、その signal を page の demo の関数の中で作る（`signal(false)` など）。shell だけが使う関数（`install_scroll_spy`、`section_navigation`、`catalog_sidebar`、`catalog_topbar`、`catalog_content`、`catalog_body`、`select_section`、`catalog_brand`、`activity_counter`、`is_palette_shortcut`、`palette_commands`）は消す。overlay の demo（`student_drawer`、`request_dialog`、`operation_confirmation`）は `overlays` の page の demo に移す。`hero.rs` の hero は `overview` の page の demo にする。toast の icon（annex U の `Toast::icon_name`）：`shell.rs` の `notification_callback` の `toast.icon(|| kivact::view::icon("gift"))` と `toast.icon(|| kivact::view::icon("sword"))` を `toast.icon_name("gift")`・`toast.icon_name("sword")`（pop の icon set にある名前）にし、この関数を使う page（`tokens`・`buttons` などの `notify` を受ける page）の helper として `src/pages/common.rs` に移す。条件の `toast.title.contains("報酬")`・`contains("作戦開始")`・`== "コピーしました"` は、手順 4 で訳した英語の title（呼出し側の文字列と同じ訳）で比べる。
  4. 英語化：catalog の日本語の文字列（`rg -n '[\p{Hiragana}\p{Katakana}\p{Han}]' examples/kivact-ui-pop/src`、約 660 行）を英語に訳す。`src/messages.rs`（日本語の message の表）と `locale()` を消す。
  5. `src/lib.rs`・`src/main.rs`：3.0.8 の形。`theme(mode)` は `kivact_ui_pop::theme(mode).with_fonts(FONTS.iter().copied())`。`FONTS` は catalog の `static FONTS: &[&[u8]]`（Inter、`assets/fonts/barlow/Barlow-BoldItalic.ttf`、`Barlow-ExtraBoldItalic.ttf`、Noto Sans JP。今の `fonts.rs` の 4 つ）。`START = "overview"`。消す：`src/fonts.rs`、`src/messages.rs`、`src/pop.rs`、`src/pop/shell.rs`、`src/pop/keyed_catalog.rs`、`src/pop/loader.rs`（page に移した中身以外）。`src/pop/{common,components_helpers,data}.rs` の helper と data は `src/pages/common.rs`・`src/pages/data.rs` に移す。
  6. `tests/catalog.rs`：`every_page_opens_from_the_list_and_renders_cleanly`（`check::every_page_opens_cleanly("Pop", pages, THEME, &[])`）と `every_page_character_is_in_the_catalog_fonts` を置く。今の 5 つの試験（`pop_source_chapters_scroll_in_the_original_shell`、`pop_forms_edit_drag_choose_date_and_keep_state_in_dark_theme`、`pop_modals_drawer_calendar_and_conversation_use_live_models`、`pop_palette_commands_open_their_dialogs`、`pop_two_quick_messages_both_get_a_reply`）のうち、`pop_source_chapters_scroll_in_the_original_shell` と `pop_palette_commands_open_their_dialogs` は shell の振る舞い（scroll-spy と palette）なので消す。残りの 3 つは `check::open(&screen, "<slug>")` で page を開いてから今の操作を行い、英語にした label で要素を取る。期待値は変えない。
  7. `crates/kivact-ui-pop/README.md` を 3.0.10 の雛形にする（「Migrating from the previous release」（122 行目〜）と recipe の一覧を消す。「prefixed with `pop.`」の誤った文も消える）。
- **消すもの**：上の catalog の file と試験、README の 2 つの節。
- **状態の閉じ方**：なし。
- **試験**：上の catalog の 5 つと 4.6 節の pop の「残す」。
- **確認（tier a）**：3.0.9 の最後の unit と catalog（`<kit>` = `pop`）。 加えて pending の検査を strict で通す（INC-14）：`cargo xtask check --no-write --strict --only hand-element,element-shadow --crate kivact-ui-pop --crate kivact-ui-pop-catalog`。
- **完了条件**：`rg -c '[\p{Hiragana}\p{Katakana}\p{Han}]' examples/kivact-ui-pop` が何も出さない。`rg -n "install_scroll_spy|messages\(\)" examples/kivact-ui-pop` が何も出さない。上の command が通る。

---

### 3.3 lane K3：creative（K22〜K25、K38）

## K22：creative の土台と recipe

- **wave / lane / 前提**：W9 / K3 / W8 の完了
- **所有**：`crates/kivact-ui-creative/`
- **目的**：creative を compile の対象に戻し、骨格に揃える（KIT-06、KIT-07）。
- **手順**：
  1. root `Cargo.toml` は変えない（kit・catalog は workspace の member のまま。gate の `--exclude` が外していたのをやめるだけ。INC-09）。
  2. `src/color.rs`（T1〜T5。`"creative.<field>"`）：`CANVAS`、`ISLAND`、`HEADER`、`STRONG`、`FIELD`、`HAIR`、`HAIR_STRONG`、`TEXT`、`TEXT_HIGH`、`MUTED`、`DIM`、`ACCENT`、`ACCENT_HIGH`、`ACCENT_INK`、`SELECTION`、`KEY`、`X`、`Y`、`Z`、`SUCCESS`、`WARNING`。`set_colors(theme, mode, accent: Accent)`：`ACCENT`・`ACCENT_HIGH`・`ACCENT_INK` は今の `accent.colors()` の 3 つの値、他は今の `Tokens::new(mode, ..)` の値。今の `color.rs` の引数なしの関数（`transparent_ink`、`menu_hover`、`overlay_surface`、`modal_backdrop` など。`rg -n "^pub\(crate\) fn [a-z_]+\(\)" crates/kivact-ui-creative/src/color.rs`）は同じ名前の大文字の変数にする。
  2a. 別名の fallback（T1a）：`ISLAND` ← `color::SURFACE`、`TEXT` ← `color::TEXT`、`MUTED` ← `color::TEXT_MUTED`、`HAIR` ← `color::BORDER`、`X` ← `color::DANGER`、`SUCCESS` ← `color::SUCCESS`、`WARNING` ← `color::WARNING`、`STRONG` ← `color::PRESSED`（今の `theme_with` の `Palette` の token の行）。他は light の literal。
  3. `src/size.rs`：`ROW`、`GAP`（`"creative.row"`、`"creative.gap"`）と `set_sizes(theme, density: Density)`（今の `Tokens::new` の density ごとの値）。
  4. font（INC-18）：`body_font` は `font::FAMILY_VAR`（S 表。static だけ）に今の値を set する。`mono_font` は `src/fonts.rs`（新）の `pub static MONO_FAMILY_VAR: ThemeVar<FontFamily> = ThemeVar::new("creative.mono_family", FontFamily::from("monospace"));`（const は作らない）にし、`build` で `.set(&MONO_FAMILY_VAR, <今の mono_font の値>)` する。`lib.rs` で `pub use fonts::MONO_FAMILY_VAR;`。読んでいた所（`t.ext::<Tokens>().mono_font.as_str()` など）は `.font_family(t.get(&MONO_FAMILY_VAR).as_str())`。`FontFamily::from` が const fn でなければ、fallback は `FontFamily::SYSTEM` にし、`build` で必ず set する。
  5. `Accent`・`Density`・`LabelTone`（`tokens.rs` → `theme.rs` の隣の `src/options.rs`。`ThemeOptions` と一緒）：`Accent::colors` はそのまま（`set_colors` が使う。`pub(crate)` にしない：今 pub）。`LabelTone::color(self, palette: &Tokens) -> Color` を `LabelTone::color(self) -> Color` にし、今の `match` の腕の `palette.<field>` を変数（`TEXT`、`MUTED`、`ACCENT`、`WARNING`、`X` など今の field）にする。
  6. `src/theme.rs`：3.0.7 の game・creative の形。`ThemeOptions { accent: Accent, density: Density }`（今のまま。`#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]`）。`build(mode, options)` の最初は今のとおり `if dark { Theme::dark() } else { Theme::light() }`。`set_core`：今の `Palette`（`surface: p.island` など。literal の `rgb(..)`・`rgba(..)` はそのまま）、`Typography`、その他の今の `theme_with` の `with_*` の行を T6・T7 で。
  7. `src/variants.rs`：`NoticeVariant`（`feedback.rs` から移す）。creative には他の variant の型が無い（`rg -n "impl .*Variant for" crates/kivact-ui-creative/src`）。
  8. recipe（3.0.3）：`recipes.rs` の `build` の中の closure の recipe（`button`、`toggle_button`、`checkbox`、`switch`、`text_input`、`text_area`、`select` の各 part、`toggle_group`、`menu` の各 part、`dialog` の各 part、`tooltip`、`tabs` の各 part）は、closure の中身を名前のある関数にして移す：controls（button、toggle、checkbox、switch、text_input、text_area、`recipes_controls.rs` の radio_group、slider、splitter、progress、disclosure）→ `recipes_controls.rs`、overlays（select、menu、dialog、tooltip）→ `recipes_overlays.rs`、collections（toggle_group、`recipes_collections.rs` の table、item_list、list、data_table）→ `recipes_collections.rs`、tabs → `recipes_navigation.rs`、feedback（`recipes_controls.rs` の toast、`recipes_feedback.rs` の alert、empty_state、card）→ `recipes_feedback.rs`、`recipes_feedback.rs` の input_group → `recipes_input.rs`、`recipes_indicators.rs` → `recipes_feedback.rs`、`recipes_metrics.rs` → `recipes_metrics.rs`。`control_style.rs`・`dock_style.rs` の helper は使う側（分野の部品が使うなら K23 の部品の file、recipe だけが使うなら `recipe_parts.rs`）。`recipes.rs` を消す。
  9. `src/lib.rs`：crate doc（今の 4 段落。`Tokens` の言及を消す）、foundation と recipe の module と `pub use`（`color`、`size`、`theme::{ThemeOptions, theme, theme_with}`、`options::{Accent, Density, LabelTone}`、`variants::NoticeVariant`）。`#![forbid(unsafe_code)]` は残す。
- **消すもの**：`Tokens`、`ColorName`、`tokens.rs`、`recipes.rs`、`UiRecipes` と `OnceLock`。
- **状態の閉じ方**：なし。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit（`<kit>` = `creative`）。
- **完了条件**：`rg -n "ColorName|Tokens|UiRecipes" crates/kivact-ui-creative/src` が何も出さない。上の command が通る。

## K23：creative の部品 1

- **wave / lane / 前提**：W9 / K3 / K22
- **所有**：`crates/kivact-ui-creative/`
- **目的**：DEC-03（3.0.5）。
- **手順**：次の module を変換し `lib.rs` に足す：`audio/`（`meter` → `Meter`、`mixer_strip` → `MixerStrip`、`transport` → `Transport`。`ChannelState`、`TransportState` は data）、`bezier/`（`bezier_curve` → `BezierCurve`。`CubicBezier` は data）、`color_popup/`（`color_popup`）、`color_wheel/`（`color_wheel` → `ColorWheel`。`ColorModel`、`color_from_hsv`、`color_to_hsv` は data と関数）、`controls.rs`、`control_style.rs`、`curve_preview.rs`（`curve_preview` → `CurvePreview`）、`dock/`・`dock_style.rs`（`dock` → `Dock`。`DockAxis`、`DockChild`、`DockLayout`、`DockNode`、`DockPlacement`、`DockSide`、`DockTab`、`DockTabs` は data）、`dope_sheet.rs`（`dope_sheet` → `DopeSheet`）、`feedback.rs`（`split_button` → `SplitButton`、`tool_window` → `ToolWindow`、`popup_button`、`tool_window_button`。`ToolWindowApi`、`ToolWindowOptions`、`WindowPosition` は data）、`gradient/`（`gradient_editor` → `GradientEditor`。`GradientStop` は data）、`key_glyph.rs`（`KeyGlyph` は data）、`layout.rs`（`panel` → `Panel`、`media_tile`、`tree_row`）。`meter(label, level)` の `label` は見えない名前だが meter は入力ではないので位置引数のまま（C6 は入力だけ）。`transport(binding)` の `Binding` は `Signal`（C3）。
- **消すもの**：手書きの builder の部分。
- **状態の閉じ方**：C10。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：`rg -n "impl (kivact::view::)?(Element|IntoView) for" crates/kivact-ui-creative/src/{audio,bezier,color_popup,color_wheel,dock,gradient}` と `crates/kivact-ui-creative/src/{controls,curve_preview,dope_sheet,feedback,key_glyph,layout}.rs` が何も出さない。上の command が通る。

## K24：creative の部品 2（editors）

- **wave / lane / 前提**：W9 / K3 / K23
- **所有**：`crates/kivact-ui-creative/`
- **目的**：DEC-03。
- **手順**：`editors/`（`arrangement` → `Arrangement`、`graph_editor` → `GraphEditor`、`node_editor` → `NodeEditor`。`editors::shared` と model の data の型はそのまま）と `timeline.rs`（`timeline_editor` → `Timeline`）を変換し `lib.rs` に足す（今の `pub use crate::editors::..` の行を全部戻す）。K03 で入れた `watch_inputs`・`stale` はそのまま。
- **消すもの**：手書きの builder の部分。
- **状態の閉じ方**：C10。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：`rg -n "impl (kivact::view::)?(Element|IntoView) for" crates/kivact-ui-creative/src/editors crates/kivact-ui-creative/src/timeline.rs` が何も出さない。上の command が通る。

## K25：creative の部品 3

- **wave / lane / 前提**：W9 / K3 / K24
- **所有**：`crates/kivact-ui-creative/`
- **目的**：DEC-03。
- **手順**：次を変換し `lib.rs` に足す：`numeric/`（`number`、`value_slider`、`knob`、`fader` の 4 つの関数を、それぞれ `#[component]` にする。共通の本体は今の `make(label, value, kind)` と `impl IntoView for Numeric` の本体を `fn numeric_view(label, value, kind, <今の Numeric の setter の値>) -> impl Element`（private）にし、4 つの部品がそれを呼ぶ。setter（`Numeric` の `pub fn`）は 4 つの部品の同じ props にする。生成される struct は `Number`、`ValueSlider`、`Knob`、`Fader`。`Numeric` の型名は消える。`label` は見える label なので位置引数のまま（DEC-05）。`NumericKind`、`PropertyState`、`evaluate_expression` は data と関数）、`outliner/`・`outliner_data.rs`（`outliner` → `Outliner`。`Restriction`、`RestrictionFlag`、`TreeDrop`、`TreeItem`、`OutlinerState` は data）、`panel_group/`（`panel_group`。`InspectorPanel` は data）、`picker.rs`（`search_picker` → `SearchPicker`。`PickerItem` は data）、`pie/`（`pie_menu` → `PieMenu`。`PieEntry`、`PieMode`、`PieState` は data）、`property_row.rs`（`property_row` → `PropertyRow`、`property_state_button` → `PropertyStateButton`）、`range/`（`range` → `Range`。`label` は見えない名前で、range は入力なので C6：`range(value).label("..")`）、`tabs.rs`（`document_tabs` → `DocumentTabs`）。3.0.6 の S4 は K38。
- **消すもの**：`Numeric`（型名）、手書きの builder の部分、`range` の `label` 位置引数。
- **状態の閉じ方**：C10。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：`rg -n "impl (kivact::view::)?(Element|IntoView) for" crates/kivact-ui-creative/src` が何も出さない。上の command が通る。

## K38：creative の icon・試験・catalog・README（W10）

- **wave / lane / 前提**：W10 / K3 / K25、K07、K14
- **所有**：`crates/kivact-ui-creative/`、`examples/kivact-ui-creative/`、`Cargo.lock`
- **目的**：KIT-08、TL-13、KIT-12、KIT-16、KIT-20 を creative に当てる。
- **手順**：
  1. `src/icons.rs` を 3.0.7 の icon の形にする（`ICON_NAMES` は出さない）。`OnceLock` を消す。3.0.6 の S4 で lib を通す。
  2. root `Cargo.toml` は変えない（kit・catalog は workspace の member のまま。gate の `--exclude` が外していたのをやめるだけ。INC-09）。
  3. 試験：4.6 節の creative の表。
  4. catalog（3.0.8）：今の `common.rs` の `SECTIONS`（22 個）を、それぞれ一つの page にする（slug は section の英語の名前の kebab-case：`buttons`、`numeric-fields`、`selection-input`、`indicators-labels`、`transport`、`knobs-faders`、`meters`、`mixer-console`、… 今の 22 個の順）。`category` は section の group の番号（今の `Category::{Controls, Audio, Layout, Animation, Compositing, Feedback}`）の英語名。section の view の関数（`shell.rs` の `use super::..` で並ぶ `buttons`、`selection`、`knobs` など）を page の demo にする。shell の chrome（`accent_controls`、`application_menus`、`catalog_status`、`performance_meters`、`transport_readout`）は `overview` の page の demo にまとめる（accent の切り替えの demo は `provide_theme(move || creative::theme_with(mode, ThemeOptions { accent: accent.get(), ..Default::default() }), ..)` で page の中だけに効かせる）。`editors.rs` の `fn mix` を消し、`Color::mix`（S の署名）を使う。`src/creative/shell.rs`、`src/creative.rs`、`src/fonts.rs` を消す。日本語の文字列（約 86 行）を英語に訳す。`theme(mode)` は `kivact_ui_creative::theme(mode).with_fonts(FONTS.iter().copied())`（`FONTS` は Geist、Geist Mono、Noto Sans JP の 3 つの bytes の `static FONTS: &[&[u8]]`）``（今の `fonts.rs` の 3 つ。`static FONTS` を catalog の `lib.rs` に置く）。`START = "buttons"`。
  5. `tests/catalog.rs`：`every_page_opens_from_the_list_and_renders_cleanly`（`check::every_page_opens_cleanly("Creative", pages, THEME, &[])`）と `every_page_character_is_in_the_catalog_fonts`。今の試験（`rg -n "^fn |#\[test\]" examples/kivact-ui-creative/tests/catalog.rs`）のうち shell の chrome（chapter の sidebar、scroll）を確かめるものは消し、部品を操作するものは `check::open(&screen, "<slug>")` で page を開いてから今の操作を行う。期待値は変えない。
  6. `crates/kivact-ui-creative/README.md` を 3.0.10 の雛形にする（「Migrating from the previous release」（309 行目〜）と recipe の一覧を消す。Tabler の license の節は残す）。
- **消すもの**：`OnceLock`、catalog の shell、`fn mix`、README の 2 つの節。
- **状態の閉じ方**：なし。
- **試験**：上の catalog の試験と 4.6 節の creative の「残す」。
- **確認（tier a）**：3.0.9 の最後の unit と catalog（`<kit>` = `creative`）。 加えて pending の検査を strict で通す（INC-14）：`cargo xtask check --no-write --strict --only hand-element,element-shadow --crate kivact-ui-creative`。
- **完了条件**：`rg -n "OnceLock|fn mix" crates/kivact-ui-creative/src examples/kivact-ui-creative/src` が何も出さない。`rg -c '[\p{Hiragana}\p{Katakana}\p{Han}]' examples/kivact-ui-creative` が何も出さない。上の command が通る。

---

### 3.4 lane K4：game（K26、K27、K39、K40）

## K26：game の土台と recipe（KIT-13 の options を含む）

- **wave / lane / 前提**：W9 / K4 / W8 の完了
- **所有**：`crates/kivact-ui-game/`
- **目的**：game を compile の対象に戻し、骨格に揃える（KIT-06、KIT-07）。font を `ThemeOptions` で選べるようにし、recipe が theme の変数の font を読むようにする（KIT-13）。
- **手順**：
  1. root `Cargo.toml` は変えない（kit・catalog は workspace の member のまま。gate の `--exclude` が外していたのをやめるだけ。INC-09）。
  2. `src/color.rs`（`"game.<field>"`）：`PRIMARY_HOVER`、`PRIMARY_STRONG`、`PRIMARY_WEAK`、`SUCCESS_WEAK`、`WARNING_WEAK`、`DANGER_WEAK`、`ON_DARK`、`DARK_SHADOW`、`TOAST_TEXT`、`TAPE_STRIPE`、`DROPZONE_STRIPE`、`CYAN`、`YELLOW`、`NAVY`、`CANVAS`、`SURFACE_3`、`LINE_STRONG`、`TEXT_2`、`TEXT_3`、`SIDEBAR`、`SIDEBAR_ALT`、`SIDEBAR_TEXT`、`SIDEBAR_DIM`。`set_colors(theme, mode)` は今の `Tokens::light()` と dark の値（`rg -n "fn (light|dark)|ThemeMode::Dark" crates/kivact-ui-game/src/tokens.rs` の各 table）。
  2a. 別名の fallback（T1a）：`TEXT_2` ← `color::TEXT_MUTED`、`PRIMARY_WEAK` ← `color::SELECTION`、`YELLOW` ← `color::FOCUS`、`SURFACE_3` ← `color::PRESSED`（今の `Tokens::palette` の token の行）。size の `CONTROL_HEIGHT` は今 `with_control_height` に入れていたので `ThemeVar::alias("game.control_height", &control::HEIGHT_VAR)`、theme では `.set(&control::HEIGHT_VAR, &CONTROL_HEIGHT_VAR)` と書く（density の値は `CONTROL_HEIGHT_VAR` に set）。他は light の literal。
  3. `src/size.rs`：`CONTROL_HEIGHT`、`ROW_HEIGHT`、`PADDING` と `set_sizes(theme, density: Density)`（今の `Tokens::new(mode, density)` の density ごとの値）。
  4. font（KIT-13、INC-18）：`src/fonts.rs`（新）に `DISPLAY_FAMILY_VAR`（名前 `"game.display_family"`。`pub static DISPLAY_FAMILY_VAR: ThemeVar<FontFamily> = ThemeVar::alias("game.display_family", &font::FAMILY_VAR);`。設計 12 章 UI-U-02 が INC-18 の `FontFamily::SYSTEM` を置き換える） と `pub static MONO_FAMILY_VAR: ThemeVar<FontFamily> = ThemeVar::new("game.mono_family", FontFamily::from("monospace"));`（`FontFamily::from` が const fn でなければ fallback は `FontFamily::SYSTEM` にし、`build` で `"monospace"` を必ず set する）を置く。const の `DISPLAY_FAMILY`・`MONO_FAMILY` は作らない。`light`・`dark` の `build` は `.set(&DISPLAY_FAMILY_VAR, <今の `Tokens::light()` の `display_family` の値>)` と `.set(&MONO_FAMILY_VAR, <今の `mono_family` の値>)` をする。recipe の `.font_family(t.ext::<Tokens>().display_family.as_str())` は `.font_family(t.get(&DISPLAY_FAMILY_VAR).as_str())`、`mono_family` と literal の `.font_family("monospace")` は `.font_family(t.get(&MONO_FAMILY_VAR).as_str())`（`rg -n "font_family\(" crates/kivact-ui-game/src`。recipe の外の分野の部品は `use_theme().get().get(&..)` で読む）。
  5. `src/theme.rs`（3.0.7 の options の形）：

     ```rust
     /// The options of the console theme.
     #[derive(Clone, Debug, Default, PartialEq, Eq)]
     pub struct ThemeOptions {
         /// The row and control metrics.
         pub density: Density,
         /// The family of headings, buttons and numbers; `None` keeps the theme's family.
         pub display_font: Option<String>,
         /// The family of codes and identifiers; `None` keeps `monospace`.
         pub mono_font: Option<String>,
     }
     ```

     `build(mode, options)`：`set_colors`、`set_sizes(.., options.density)`、`set_core`、`options.display_font` が `Some(f)` なら（手順 4 の既定の set の後で）`.set(&DISPLAY_FAMILY_VAR, f.as_str())`、`mono_font` も同じ。`theme_with(mode, options: ThemeOptions)` は値で受ける（`Clone` で `==` を比べる）。`set_core` は今の `Tokens::palette()` の計算（`colors.surface = rgb(0xffffff)` など mode ごとの literal と、`text_muted = self.text_2` などの token）と今の `theme_with` の `Radii`・`Spacing`・`Typography`・`with_*`・`ui.<部品>.<field>` の全行を T6・T7 で書く。`Tokens::palette` を消す。
  6. `Density`（`tokens.rs` → `src/theme.rs`）はそのまま公開。
  7. `src/variants.rs`：
     - そのまま：`AvatarVariant`（`side: f64, color: Color` の値の struct）、`BadgeVariant`、`SkeletonVariant`、`HeadingVariant`、`StatVariant`、`ChoiceCard`（marker）、`Slant`・`Striped`（K39 まで）。
     - 改名：`GameInputGroupVariant` → `InputGroupVariant`、`GameCellVariant`（`table.rs`）→ `TableCellVariant`、`Navigation`・`Boxed` の 2 つの marker → `pub enum TabsVariant { Navigation, Boxed }`（recipe の `s.variant::<Navigation>().is_some()` は `s.variant::<TabsVariant>() == Some(TabsVariant::Navigation)` の形に）。
     - 値にする：`GaugeSegments(pub Value<usize>)` → `GaugeSegments(pub usize)`（V5：`Element::variant` が `Value` を解決する）。
     - `ButtonTone`（`Dark` の 1 値）は K27 で消す（ここでは移すだけ）。
  8. recipe（3.0.3）：`recipes.rs` の `build` の closure の recipe（`button`、`button_content`、`button_busy`、`toggle`、`checkbox`、`switch`、`text_input`、`text_area`）は `recipes_controls.rs` へ。`recipes_extra.rs` の `select_*`、`menu_*`、`dialog_*`、`tooltip_surface` → `recipes_overlays.rs`；`tabs_list`、`tab` → `recipes_navigation.rs`；`radio_*`、`slider_*`、`splitter_handle` → `recipes_controls.rs`；`toast_*` → `recipes_feedback.rs`。`recipes_small.rs`（progress、disclosure）と `recipes_progress.rs` → `recipes_controls.rs`。`recipes_field.rs` → `recipes_controls.rs`。`recipes_badge.rs`、`recipes_avatar.rs`、`recipes_indicators.rs`、`recipes_feedback.rs` → `recipes_feedback.rs`（400 行を越えたら `recipes_feedback_<部品>.rs`）。`recipes_items.rs`、`recipes_collections.rs`、`recipes_groups.rs`（toolbar、button_group、toggle_group）→ `recipes_collections.rs`；`recipes_groups.rs` の input_group → `recipes_input.rs`；`recipes_files.rs`、`recipes_input.rs` → `recipes_input.rs`；`recipes_navigation.rs`、`recipes_metrics.rs`、`recipes_palette.rs`（command_palette → `recipes_overlays.rs`）。`choice_style.rs` の helper と `button.rs` の recipe の helper → 使う recipe の file か `recipe_parts.rs`。中身の無くなった file を消す。
  9. `src/lib.rs`：crate doc（今の 3 段落。「Builders implement `Element` and therefore `Element` as well.」の文を消す）、foundation と recipe の module と `pub use`（`color`、`size`、`fonts::{DISPLAY_FAMILY_VAR, MONO_FAMILY_VAR}`、`theme::{Density, ThemeOptions, theme, theme_with}`、`variants`）。
- **消すもの**：`Tokens`、`ColorName`、`tokens.rs`、`Tokens::palette`、`GameInputGroupVariant`、`GameCellVariant`、`Navigation`・`Boxed`（型）、`recipes.rs`、`recipes_extra.rs` など移し終えた file、`UiRecipes` と `OnceLock`。
- **状態の閉じ方**：`ThemeOptions` は `Options` の接尾辞で対象外（設計 6.1）。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit（`<kit>` = `game`）。
- **完了条件**：`rg -n "ColorName|Tokens|GameInputGroupVariant|GameCellVariant|display_family|mono_family|UiRecipes" crates/kivact-ui-game/src` が何も出さない。上の command が通る。

## K27：game の部品と `ButtonTone` の削除

- **wave / lane / 前提**：W9 / K4 / K26
- **所有**：`crates/kivact-ui-game/`
- **目的**：DEC-03。一つの値しかない `ButtonTone { Dark }` を消し、navy の button を `ButtonVariant::Secondary` で描く（KIT-04 の決定）。
- **手順**：
  1. `ButtonTone` を消す。`recipes_controls.rs` の button の recipe で `s.variant::<ButtonTone>() == Some(ButtonTone::Dark)` の枝の style を、`ButtonVariant::Secondary` の枝の style にする（今の Secondary の枝は消える。Dark の navy の style が Secondary になる）。
  2. 部品を変換し `lib.rs` に足す：`button.rs`（`tool_button`、`chevron`）、`choice.rs`（`card_label`、`navigation_label`）、`display.rs`（`activity`。`ActivityEntry` は data）、`kpi.rs`（`kpi` → `Kpi`。`reveal_once` の Machine はこの unit では残し、K39 で `once_visible` にする）、`overlays.rs`（`icon_menu` → `IconMenu`）、`surface.rs`（`halo`、`hud`、`mono` は部品、`cut_corner` は `Shape` を返す関数のまま）、`table.rs`（`TableCellVariant` は K26 で移した。他の部品があれば変換）。
- **消すもの**：`ButtonTone`、手書きの builder の部分。
- **状態の閉じ方**：C10。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：`rg -n "ButtonTone|impl (kivact::view::)?(Element|IntoView) for" crates/kivact-ui-game/src` が何も出さない。上の command が通る。

## K39：game の `Slant`・`Striped`・kpi の latch・icon（W10）

- **wave / lane / 前提**：W10 / K4 / K27、K07、K08、K09
- **所有**：`crates/kivact-ui-game/`
- **目的**：KIT-04（marker を中核へ）、KIT-11（kpi の latch を `NodeRef::once_visible` に）、KIT-08（icon）。
- **手順**：
  1. `Slant`・`Striped`：K36 の手順 1 と同じ（`crates/kivact-ui-game` で）。
  2. `src/kpi.rs`：`enum Reveal`、`struct RevealMachine`、`impl Machine for RevealMachine`、`fn reveal_once` を消し、`reveal_once(tile)` の呼出しを `tile.once_visible(0.3)` にする。使わなくなった `machine`・`effect`・`platform::window_size` の import を消す。
  3. `src/icons.rs`：3.0.7 の icon の形。並べる名前は今の `NAMES` と `match` の腕の両方にある名前。
  4. 3.0.6 の S4。
- **消すもの**：`game::Slant`、`game::Striped`、`Reveal`、`RevealMachine`、`reveal_once`、`NAMES`、`OnceLock`。
- **状態の閉じ方**：なし。
- **試験**：なし（K40）。
- **確認（tier a）**：K36 の確認の 4 行（`-p kivact-ui-game`）。
- **完了条件**：`rg -n "struct (Slant|Striped)|RevealMachine|reveal_once|OnceLock|const NAMES" crates/kivact-ui-game/src` が何も出さない。上の command が通る。

## K40：game の試験・catalog・README（KIT-13 の docs と catalog を含む）

- **wave / lane / 前提**：W10 / K4 / K39、K14
- **所有**：`crates/kivact-ui-game/`（`tests/`、`README.md`）、`examples/kivact-ui-game/`、`Cargo.lock`
- **目的**：TL-13、KIT-12、KIT-16、KIT-20。catalog が font を data の literal で書くのをやめ `ThemeOptions` を使う（KIT-13）。
- **手順**：
  1. root `Cargo.toml` は変えない（kit・catalog は workspace の member のまま。gate の `--exclude` が外していたのをやめるだけ。INC-09）。
  2. 試験：4.6 節の game の表。`tests/consumer/theme.rs` の `consumer_theme_density_and_halo_pixels_follow_public_theme` の `ThemeOptions { density: .. }` は `ThemeOptions { density: .., ..Default::default() }` にする（field が増えた）。
  3. catalog（3.0.8）：今の 11 の section（`section_names()` の順：Foundations、Buttons、Inputs、Selection、Navigation、Data Display、Data Table、Feedback、Overlays、Native Layout、Effects）を page にする（slug は英語名の kebab-case）。`category` は 4.7 節。section の中身の関数（`controls.rs`、`foundations.rs`、`system.rs`、`effects.rs`、`window.rs`、`frame.rs`、`specimens.rs` の関数）を page の demo にする。shell の関数（`app_shell`、`section`、`reveal`、`sidebar`、`install_scroll_spy`、`palette_commands`、`command_handler`、`catalog_content`、`catalog_topbar`、`catalog_key`、`catalog_layers`、`sidebar_button_label`、`sidebar_button`、`sidebar_brand`、`sidebar_environment`、`palette_search`、`operator_menu`）は消し、`hero` は `foundations` の page の最初の demo にする。density の切り替え（`density_controls`）は `foundations` の page の demo にし、`provide_theme(move || game::theme_with(mode, ThemeOptions { density: density.get(), ..options() }), ..)` で demo の中だけに効かせる。
  4. KIT-13：catalog の theme を

     ```rust
     fn options() -> ThemeOptions {
         ThemeOptions {
             display_font: Some("Barlow Condensed".into()),
             mono_font: Some("JetBrains Mono".into()),
             ..ThemeOptions::default()
         }
     }

     pub fn theme(mode: ThemeMode) -> Theme {
         kivact_ui_game::theme_with(mode, options()).with_fonts(FONTS.iter().copied())
     }
     ```

     にし、`data.rs` の `.font_family("Barlow Condensed")`（`rg -n 'font_family\("Barlow' examples/kivact-ui-game/src`）を消す。`FONTS` は今の `fonts.rs` の 6 つ（Barlow Condensed 4、JetBrains Mono、Noto Sans JP）。
  5. 英語化（約 457 行）。`src/messages.rs` と `locale()` を消す。`START = "foundations"`。消す：`src/game.rs`、`src/game/shell.rs`、`src/fonts.rs`、`src/messages.rs`。残る `src/game/*.rs` は `src/pages/` に移す。
  6. `tests/catalog.rs`：`every_page_opens_from_the_list_and_renders_cleanly`（`check::every_page_opens_cleanly("Game", pages, THEME, &[])`）、`every_page_character_is_in_the_catalog_fonts`。今の試験のうち shell（sidebar、scroll-spy、palette の shortcut）を確かめるものは消し、部品を操作するものは `check::open` で page を開いてから今の操作を行う。
  7. `crates/kivact-ui-game/README.md` を 3.0.10 の雛形にする（「Migrating from the previous release」（50 行目〜）と recipe の一覧を消す）。README の使い方の例の theme は `game::theme_with(ThemeMode::Dark, game::ThemeOptions { density: game::Density::Compact, ..Default::default() })`（`needless_update` にならない：field が 3 つある）。
- **消すもの**：上の catalog の file、`data.rs` の literal の font、README の 2 つの節。
- **状態の閉じ方**：なし。
- **試験**：上の catalog の試験と 4.6 節の game の「残す」。
- **確認（tier a）**：3.0.9 の最後の unit と catalog（`<kit>` = `game`）。 加えて pending の検査を strict で通す（INC-14）：`cargo xtask check --no-write --strict --only hand-element,element-shadow --crate kivact-ui-game --crate kivact-ui-game-catalog`。
- **完了条件**：`rg -n 'font_family\("Barlow' examples/kivact-ui-game` と `rg -c '[\p{Hiragana}\p{Katakana}\p{Han}]' examples/kivact-ui-game` が何も出さない。上の command が通る。

---

### 3.5 lane K5：soft（K28〜K33、K41〜K43）

## K28：soft の土台（theme 変数・内部の定数・theme・policy・variant・使われない関数の削除）

- **wave / lane / 前提**：W9 / K5 / W8 の完了
- **所有**：`crates/kivact-ui-soft/`
- **目的**：soft を compile の対象に戻し、入れ子の `Tokens`（512 行）・`Colors`（660 行）・`ColorName`（183 行）を theme 変数と crate の中の定数にする。公開するのは色・間隔・角丸だけ（KIT-19、KIT-07）。interaction の policy を `theme_policy.rs` に分け、`theme.rs` を他の kit と同じ形にする（KIT-19）。使われていない公開の関数を消す（KIT-09）。
- **手順**：
  1. root `Cargo.toml` は変えない（kit・catalog は workspace の member のまま。gate の `--exclude` が外していたのをやめるだけ。INC-09）。
  2. `src/color.rs` を書き直す（T1〜T5。名前の文字列は `"soft.<名前の小文字>"`。値は今の `Colors::light()`・`dark()`、`Hues`、`DataColors`、`SyntaxColors` の値）：
     - `Colors` の `Color` の field（`bg_body` 〜 `text_error`）：field 名の大文字（`BG_BODY`、`TEXT_PRIMARY`、`ON_ACCENT`、`OVERLAY_SCRIM` など、今の 50 余りの field 全部）。
     - `Colors.hue.<色>.<部分>`（色 = red、orange、yellow、green、teal、cyan、blue、purple、pink、gray。部分 = bg、border、icon、text）：`HUE_<色>_<部分>`（例 `HUE_RED_BG`）。
     - `Colors.data.<名前>`（blue、cyan、orange、red、purple、teal、green、brown、pink、indigo、neutral）：`DATA_<名前>`。`<名前>_ramp: [Color; 5]`（blue、shamrock、orange、pink、purple、red、teal、yellow、gray）：`DATA_<名前>_RAMP_1`〜`_5`。
     - `Colors.syntax`：`keyword`〜`property` の 13 個は soft の変数にしない（K41 で中核の構文色に set）。`background`、`added`、`removed`、`diff_header`、`added_marker`、`removed_marker`：`SYNTAX_BACKGROUND` など。
     - 別名の fallback（T1a）：今の `theme.rs` の `fn palette(tokens)` で `Palette` の field に `tokens.colors.<x>` をそのまま入れている行ごとに、`<X>` の fallback を `color::<その field の大文字>` にする（`rg -n "tokens\.colors\.[a-z_]+," crates/kivact-ui-soft/src/theme.rs` の行）。`SPACE_*`・`RADIUS_*` で今の `Spacing`・`Radii` に入れているものは `space::*`・`radius::*` の別名。他は light の literal。
     - `set_colors` は 80 行を越えるので、`set_colors_base`、`set_colors_status`、`set_colors_hue`、`set_colors_data`、`set_colors_syntax` に分ける（T5）。
  3. `src/space.rs`（新。公開）：`SpaceTokens` の field（`s0`、`s0_5`、`s1`、`s1_5`、`s2`〜`s12`、`card_padding`、`section_padding`、`dialog_padding`、`layer_viewport_gutter`）を `SPACE_<field の大文字>`（例 `SPACE_S0_5`、`SPACE_CARD_PADDING`）の `ThemeVar<f64>` と `Length` の const に。`src/radius.rs`（新。公開）：`RadiusTokens`（`none`、`inner`、`element`、`container`、`page`、`chat`、`full`）を `RADIUS_<大文字>`。`set_space`・`set_radius` を置く（mode によらない今の値）。
  4. crate の中だけの定数（KIT-19。公開しない）：
     - `src/metrics.rs`：`Sizes` の field を `pub(crate) const <大文字>: f64`（`CONTROL_MD` など）。
     - `src/type_scale.rs`：`TypeTokens` の family・weight・size を `pub(crate) const`、`TypeStyle` の 15 個（`display1` 〜 `supporting`）を `pub(crate) const <名前の大文字>: TypeStyle`。`TypeStyle` の struct は `pub(crate)`。`letter_spacing` も const。
     - `src/motion.rs`：`MotionTokens` の field を `pub(crate) const`（`Easing` が const で作れないときは `pub(crate) fn <名前>() -> Easing`）。
     - `src/shadows.rs`：`Shadows` の field を `pub(crate) fn <名前>() -> Shadow`（色は手順 2 の変数。今 mode で literal が変わる影の色は `SHADOW_<名前>` の色の変数にして `set_colors` で set）。
     今の `tokens.colors.x`・`t.spacing.sN`・`t.radius.x`・`t.sizes.x`・`t.typography.x`・`t.motion.x`・`t.shadows.x` の読みは、T8 と上の定数への置換（`t.sizes.control_md` → `crate::metrics::CONTROL_MD` など）。
  5. `src/theme_policy.rs`（新）：今の `theme.rs` の `fn interaction_policy(tokens: &Tokens) -> InteractionPolicy` を `pub(crate) fn policy() -> InteractionPolicy` にして移す（token の読みは手順 4 の定数）。
  6. `src/theme.rs`：3.0.7 の形（soft は options なし）。`set_core`：今の `palette(&tokens)`、`Spacing`、`Radii`、`typography(&tokens)`、`with_focus_ring(sizes.focus_ring_width)`、`with_focus_ring_offset`、`with_control_height(sizes.control_md)`、`with_disabled_opacity`、`with_overlay_margin(spacing.layer_viewport_gutter)` を T6 で、`ui_tokens(&tokens, mode)` とその helper（`indicators_tokens`、`combobox_tokens`、`button_tokens`、`disclosure_tokens`、`progress_tokens`、`toast_tokens`、`toggle_button_tokens`、`data_table_tokens`）の全 field を T7 で。`.with(crate::theme_policy::policy())`。theme.rs が 400 行を越えたら `set_core` の U 変数の部分を `src/theme_components.rs` に分ける。
  7. 消す（KIT-09。どれも変換しないので、ここで file か item を消す）：
     - file ごと：`container_reveal.rs`（`reveal_container`、`reveal_content`、`RevealContent`、`RevealForce`、`RevealVisibility`）、`scroll_observer.rs`（`use_scroll_state`、`use_scroll_overflow`、`use_overflow`、`AxisScroll`、`OverflowOptions`、`ScrollOverflow`、`ScrollState`）、`aspect_ratio.rs`、`tab_list.rs`、`tab_entries.rs`（`Tab`、`TabEntry`、`TabLayout`、`TabList`、`TabMenu`、`TabMenuOption`、`TabOverflow`、`TabPattern`、`tab`、`tab_list`、`tab_menu`。見た目は K41 で中核の tabs の recipe になる）、`color_name.rs`、`tokens.rs`。
     - item：`media_query.rs` の `container_below`、`is_compact`、`viewport_from`（`viewport_below` は `app_shell.rs` が使うので `pub(crate)`、`Breakpoint` は公開のまま）、`soft_chat_tools.rs` の `streaming_text`、`use_streaming_text`、`StreamSpeed`、`chat_token`、`chat_tokenized_text`（`ChatToken`・`ChatTokenTone` も、残る関数が使わなければ消す）、`rich_text.rs` の `rich_text_toolbar`（`rich_text_view` は `rich_text.rs` の中で使うので `pub(crate)`）、`link.rs` の `link_with`（`pub(crate)`）、`icon.rs` の `icon_style`（`pub(crate)`）、`grid.rs` の `grid_span`（`GridSpan`・`GridSpanItem` は `grid_span` だけが使うなら消す）、`stack.rs` の `flex_stack`（`FlexStack` も同じ）、`layout.rs` の `layout_footer`、`theme_scope.rs` の `syntax_theme`（`provide_theme` と `Theme::set` で同じことができる。一つの書き方）。
  8. `src/variants.rs`（新）：今 `impl kivact::ui::Variant for X` を持つ soft の型（`rg -n "Variant for" crates/kivact-ui-soft/src`）をここへ移す。そのうち：
     - 改名：`BannerContainer` → `BannerVariant`、`BubbleVariant` → `ChatBubbleVariant`。
     - 束の分解（V4。recipe は K29・K30 で直す）：

       | 束 | 分けた後の variant |
       |---|---|
       | `AvatarAppearance { size }` | `AvatarSize` |
       | `AvatarGroupAppearance { size, shape }` | `AvatarSize`、`pub struct AvatarGroupShape(pub kivact::ui::AvatarShape)` |
       | `BannerAppearance { container, elevation }` | `BannerVariant`、`Elevation` |
       | `CardAppearance { variant, elevation, padding: Option<Space> }` | `CardVariant`、`Elevation`、`pub struct CardPadding(pub Space)` |
       | `ChatAppearance { density, bubble, elevation, system }` | `ChatDensity`、`ChatBubbleVariant`、`ComposerElevation`、`SystemMessageVariant` |
       | `CodeAppearance { size, width, container }` | `CodeBlockSize`、`CodeBlockWidth`、`CodeBlockContainer` |
       | `PaginationAppearance { size: ControlSize }` | `pub struct PaginationSize(pub ControlSize)` |
       | `StepperAppearance { density, position, collapsed, connector_gap, item_densities }` | `StepDensity`、`StepIndicatorPosition`、`StepCollapsedVariant`。`connector_gap`（今の既定 0）と `item_densities`（今の既定 空）は `stepper.rs` の外に使う所が無いので消し、recipe は既定の値のまま |
       | `TableCellAppearance { density, vertical_align, divider }` | `ItemDensity`、`pub struct TableCellAlign(pub Align)`、`pub struct TableCellDivider(pub bool)` |
       | `TableRowAppearance { dividers, hover, striped }` | `pub struct TableRowDividers(pub bool)`、`pub struct TableRowHover(pub bool)`、`pub struct TableRowStriped(pub bool)` |
       | `MetadataLook`（`MetadataAppearance { columns, position, width, orientation }`） | `MetadataColumns`、`LabelPosition`、`pub struct MetadataLabelWidth(pub f64)`、`pub struct MetadataOrientation(pub Orientation)` |
       | `ItemLook { density, align, layout, highlighted }` | `ItemDensity`、`ItemAlign`、`ItemLayout`、`pub struct ItemHighlighted(pub bool)` |
       | `ListLook { density, dividers, edge }` | `ItemDensity`、`pub struct ListDividers(pub bool)`、`ListEdge` |
       | `SegmentedLook { layout, icon_only }` | `SegmentedLayout`、`pub struct SegmentedIconOnly(pub bool)` |
       | `TreeLook { density, guides }` | `ItemDensity`、`pub struct TreeGuides(pub bool)`（既定 `true`） |

       束の型、その `impl`、`Default`、setter を消す。
  9. `src/lib.rs`：crate doc を「Role: Astryx Neutral as a Kivact UI kit: the look of every standard component, an app frame (shell, top and side navigation), and content parts (markdown, rich blocks, chat tools, tables).」と使い方の例にする。module は foundation（`color`、`space`、`radius`、`metrics`、`type_scale`、`motion`、`shadows`、`theme`、`theme_policy`、`variants`、`icons`）だけ。`pub use` は `color::{..}`、`space::{..}`、`radius::{..}`、`theme::theme`、`variants::{..}`、`icons::ICON_NAMES`。
- **消すもの**：`Tokens`、`Colors`、`Hues`、`HueColors`、`DataColors`、`SyntaxColors`、`ColorName`、`MotionTokens`、`RadiusTokens`、`Sizes`、`SpaceTokens`、`TypeStyle`（公開として）、`TypeTokens`、`Shadows`（公開として）、手順 7 の全部、手順 8 の束、`BannerContainer`・`BubbleVariant`（改名）。
- **状態の閉じ方**：新しい状態の型なし（newtype は値）。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit（`<kit>` = `soft`）。
- **完了条件**：`ls crates/kivact-ui-soft/src/{tokens,color_name,container_reveal,scroll_observer,aspect_ratio,tab_list,tab_entries}.rs` が全部「無い」。`rg -n "pub (fn|struct|enum) (reveal_container|use_scroll_state|streaming_text|chat_token|is_compact|flex_stack|grid_span|layout_footer|syntax_theme)\b" crates/kivact-ui-soft/src` が何も出さない。上の command が通る。

## K29：soft の recipe 1（controls・overlays・feedback）

- **wave / lane / 前提**：W9 / K5 / K28
- **所有**：`crates/kivact-ui-soft/`
- **目的**：KIT-06（3.0.3）。
- **手順**：
  1. 行き先（今の `recipes.rs` の `build` の setter と関数）：

     | 今 | 行き先 |
     |---|---|
     | `recipes_controls.rs`、`recipes_controls_choice.rs`、`recipes_controls_value.rs`、`recipes_field.rs`（`field`、`text_input`、`text_area`、`number_input`。`tag_input` は K30）、`recipes_collections.rs` の `disclosure`、`splitter`、`split`、`indicator.rs` の recipe、`segmented_control.rs` の recipe（`track` など） | `recipes_controls.rs`（今の 3 つの file を `recipes_controls.rs`・`recipes_controls_choice.rs`・`recipes_controls_value.rs` のまま R2 の形にし、`recipes_controls.rs` の `install` が他の 2 つの `install` を呼ぶ） |
     | `recipes_select.rs`、`recipes_combobox.rs`、`recipes_menu.rs`、`recipes_dialog.rs`、`recipes_popup.rs` の `hover_card`・`tooltip`、`recipes_popup_command.rs` | `recipes_overlays.rs`（400 行を越えるので `recipes_overlays_select.rs`、`recipes_overlays_menu.rs`、`recipes_overlays_dialog.rs`、`recipes_overlays_command.rs` に分ける） |
     | `recipes_popup.rs` の `toast`、`recipes_badge.rs`、`recipes_skeleton.rs`、`spinner.rs` の `spinner`、`kbd.rs` の `root`・`key`、`avatar.rs` の recipe（`root`、`picture`、`status`、`status_content`、`group`、`member`、`overflow`）、`banner.rs` の `alert`、`empty_state.rs` の `empty_state` | `recipes_feedback.rs`（`recipes_feedback_avatar.rs` に avatar を分ける） |
     | `recipe_style.rs` の helper（`fast`、`ring` など） | `recipe_parts.rs`（file 名を変える） |

  2. recipe が読む束を V4 の型に直す（K28 の表。例：`s.variant::<AvatarAppearance>().map(|a| a.size.get())` → `s.variant::<AvatarSize>()`、`SegmentedLook` → `SegmentedLayout` と `SegmentedIconOnly`）。
  3. `theme.rs` の `build` に controls、overlays、feedback の `install` を R5 の順に足す。`lib.rs` に module を足す。中身を移し終えた file（`recipes_select.rs`、`recipes_combobox.rs`、`recipes_menu.rs`、`recipes_dialog.rs`、`recipes_popup.rs`、`recipes_popup_command.rs`、`recipes_badge.rs`、`recipes_skeleton.rs`、`recipe_style.rs`）を消す。`spinner.rs`、`kbd.rs`、`avatar.rs`、`banner.rs`、`empty_state.rs`、`indicator.rs`、`segmented_control.rs` に部品が残るなら K31〜K33 が変換する。
- **消すもの**：上の file。
- **状態の閉じ方**：なし。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：上の消す file が無い。上の command が通る。

## K30：soft の recipe 2（collections・navigation・metrics・chat・input・calendar・code）

- **wave / lane / 前提**：W9 / K5 / K29
- **所有**：`crates/kivact-ui-soft/`
- **目的**：KIT-06。recipe の置き場を `recipes_<group>.rs` だけにする。
- **手順**：
  1. 行き先：

     | 今 | 行き先 |
     |---|---|
     | `recipes_collections.rs` の `list`、`tree`、`data_table`、`recipes_table.rs`、`soft_list.rs` の `recipes`、`soft_tree.rs`、`recipes_groups.rs`（`button_group`、`toggle_group`、`toolbar`）、`recipes_carousel.rs`、`card.rs` の `card` | `recipes_collections.rs`（`recipes_collections_table.rs` に table と data_table を分ける） |
     | `recipes_collections.rs` の `tabs`、`recipes_navigation.rs`、`recipes_pagination.rs`、`breadcrumbs.rs` の `breadcrumb`、`stepper.rs` の `stepper` | `recipes_navigation.rs`（`recipes_navigation_pagination.rs` を分ける） |
     | `recipes_metrics.rs` | `recipes_metrics.rs` |
     | `recipes_chat.rs` | `recipes_chat.rs` |
     | `input_group.rs` の `input_group`、`recipes_field.rs` の `tag_input`、`recipes_dropzone.rs` | `recipes_input.rs` |
     | `recipes_calendar.rs`（`calendar`、`field`、`wheel`） | `recipes_calendar.rs` |
     | `recipes_code.rs` | `recipes_code.rs` |

  2. 束の読みを V4 の型に（`TableRowAppearance` → 3 つの newtype、`ItemLook`・`ListLook`・`TreeLook`・`MetadataLook`・`CodeAppearance`・`PaginationAppearance`・`StepperAppearance`・`ChatAppearance`・`CardAppearance`）。
  3. `theme.rs` の `build` に残りの group の `install` を足す。中身を移し終えた file（`recipes_table.rs`、`recipes_groups.rs`、`recipes_carousel.rs`、`recipes_pagination.rs`、`recipes_dropzone.rs`、`soft_tree.rs`）を消す。`soft_list.rs`、`card.rs`、`breadcrumbs.rs`、`stepper.rs`、`input_group.rs` に部品が残るなら K31〜K33。
- **消すもの**：上の file。
- **状態の閉じ方**：なし。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：`rg -ln "fn [a-z_]+\(\w*: &Theme" crates/kivact-ui-soft/src | rg -v "recipes_|recipe_parts"` が何も出さない。上の command が通る。

## K31：soft の部品 1（基礎と内容）

- **wave / lane / 前提**：W9 / K5 / K30
- **所有**：`crates/kivact-ui-soft/`
- **目的**：DEC-03（3.0.5）。
- **手順**：次の module の公開の部品を変換し `lib.rs` に足す（4.4.5 節の表の「部品」の行）：`alert_dialog.rs`、`avatar.rs`（`avatar_silhouette`）、`avatar_status_dot.rs`、`banner.rs`（`banner_icon`）、`blockquote.rs`、`card.rs`（`with_card_insets` は context を置く関数のまま）、`citation.rs`、`code.rs`（`code`。`CodeColor` は props の型）、`dialog_header.rs`、`divider.rs`、`empty_state.rs`、`feedback.rs`（`status_dot`）、`field_status.rs`、`form_layout.rs`、`hover_card.rs`（`hover_card_text`）、`icon.rs`（`icon_view`）、`indicator.rs`（`check_indicator`、`checkbox_indicator`、`radio_indicator`）、`key_hints.rs`（`keyboard_hint`、`with_key_hints`）、`kbd.rs`、`link.rs`（`link`）、`section.rs`（`section`。`Edges` は中核の `Edges` と名前が重なるので `SectionEdges` に改名）、`spinner.rs`、`text.rs`（`styled_text`）、`token.rs`（`token`）、`visually_hidden.rs`、`theme_scope.rs`（`media_theme`。`luminance`・`contrast` の private 関数を消し、`Color::relative_luminance()`・`Color::contrast_ratio(other)` を使う。KIT-17）。名前を位置引数で取る入力の部品は C6（`rg -n "^pub fn [a-z_]+\((label|name): impl IntoText" crates/kivact-ui-soft/src`）。
- **消すもの**：手書きの builder の部分、`theme_scope.rs` の `luminance`・`contrast`、`Edges`（改名）。
- **状態の閉じ方**：C10。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：上の file に `impl (kivact::view::)?(Element|IntoView) for` が無い。`rg -n "fn luminance|fn contrast" crates/kivact-ui-soft/src` が何も出さない。上の command が通る。

## K32：soft の部品 2（layout と collection）

- **wave / lane / 前提**：W9 / K5 / K31
- **所有**：`crates/kivact-ui-soft/`
- **目的**：DEC-03。
- **手順**：次を変換し `lib.rs` に足す：`bottom_sheet.rs`（`bottom_sheet`、`bottom_sheet_flow`。`DialogPurpose`、`SheetHeight`、`SnapPoint` は props の型）、`breadcrumbs.rs`、`choice_list.rs`（`checkbox_list`、`checkbox_list_item`、`radio_list`）、`grid.rs`（`layout_grid`）、`item_values.rs`（props の型だけ）、`layout.rs`（`layout`、`layout_content`、`layout_header`、`layout_panel`）、`media_overlay.rs`、`more_menu.rs`、`outline.rs`、`power_search.rs`、`scrollable_area.rs`、`soft_list.rs`、`soft_overflow.rs`（`overflow_list`）、`soft_table.rs`（`table_row_index`、`table_row_status`、`table_selection_toolbar`）、`stack.rs`（`center`、`hstack`、`vstack`、`stack_item`）、`stepper.rs`、`thumbnail.rs`、`lightbox.rs`（`lightbox`、`use_lightbox` は hook のまま）、`segmented_control.rs`（部品があれば）、`input_group.rs`（部品があれば）、`navigation_session.rs`（内部）。
- **消すもの**：手書きの builder の部分。
- **状態の閉じ方**：C10。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：上の file に `impl (kivact::view::)?(Element|IntoView) for` が無い。上の command が通る。

## K33：soft の部品 3（内容の大きな部品と app の枠の一部）

- **wave / lane / 前提**：W9 / K5 / K32
- **所有**：`crates/kivact-ui-soft/`
- **目的**：DEC-03。sidebar・timestamp・timer・code の構文は中核の追加が要るので W10（K41・K42）。
- **手順**：次を変換し `lib.rs` に足す：`markdown.rs`・`markdown_parser.rs`（`markdown`）、`rich_blocks.rs`・`rich_block_doc.rs`、`rich_text.rs`（`rich_text_editor`）、`soft_chat_tools.rs`（`chat_tool_calls`）、`chat_dictation.rs`（`chat_dictation_button`）、`chat_appearance.rs`（V4 で束が消えた後に残る型を `variants.rs` に移し、file を消す）、`disclosure_appearance.rs`（同じ。file を消す）、`nav_heading.rs`（`nav_heading_menu`、`nav_heading_menu_item`、`side_nav_heading`、`top_nav_heading`）、`nav_icon.rs`、`top_nav.rs`（`top_nav`、`top_nav_item`、`top_nav_menu`、`top_nav_menu_item`、`top_nav_mega_menu`、`top_nav_mega_menu_item`、`top_nav_featured_card`）、`avatar_options.rs`、`media_query.rs`（`Breakpoint` だけ）。
- **消すもの**：手書きの builder の部分、`chat_appearance.rs`、`disclosure_appearance.rs`。
- **状態の閉じ方**：C10。
- **試験**：なし。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：`rg -n "impl (kivact::view::)?(Element|IntoView) for" crates/kivact-ui-soft/src` が `app_shell.rs`、`mobile_nav.rs`、`side_navigation/`、`timestamp.rs`、`timer.rs`、`code_syntax.rs` の file だけを出す。上の command が通る。

## K41：soft の tabs・構文・icon（W10）

- **wave / lane / 前提**：W10 / K5 / K33、K07、K08、K13
- **所有**：`crates/kivact-ui-soft/`
- **目的**：soft の tab の見た目を中核の `tabs`（`TabOverflow`）の recipe にする（KIT-09）。`TokenKind` と `SyntaxColors` を中核の `SyntaxKind` と構文色にする（KIT-05）。icon を `icons!` と `thread_local!` の icon set に（KIT-08）。
- **手順**：
  1. tabs：今の `tab_list.rs`（K28 で消した。`git show HEAD~<n>:crates/kivact-ui-soft/src/tab_list.rs` か `git log --diff-filter=D -- crates/kivact-ui-soft/src/tab_list.rs` で最後の版を読む）の tab と strip の style を、`recipes_navigation.rs` の `tabs` の recipe に足す。variant を 2 つ `variants.rs` に足す：`pub enum TabsVariant { #[default] Tabs, Navigation }`（今の `TabPattern`）と `pub enum TabsLayout { #[default] Hug, Fill }`（今の `TabLayout`）。recipe は `s.variant::<TabsVariant>()`・`s.variant::<TabsLayout>()` で今の `tab_list` の `pattern`・`layout` の分岐を写す。overflow は中核の `tabs(..).overflow(TabOverflow::Scroll)`（今の `TabOverflow::Scroll` の既定は、呼出し側が付ける）。今の `tab_list` の `divider`・`full_bleed` は `TabsDivider(pub bool)`、`TabsFullBleed(pub bool)` の newtype の variant（V4）。
  2. 構文：`code_syntax.rs` の `TokenKind` を消し、tokenizer の `TokenKind::X` を `SyntaxKind::X`（同名）にする。`type Tokenizer` を消し `kivact::ui::Tokenizer` を使う。`code_highlight(language, line, palette: &SyntaxColors)` を `code_highlight(language: &CodeLanguageInput, line: &str) -> Vec<StyledRange>` にし、本体を `kivact::ui::syntax_ranges(line, &language.tokenize(line))` にする。呼出し（`rg -n "code_highlight" crates/kivact-ui-soft/src`）から palette の引数を消す。`theme.rs` の `set_core` に、今の `SyntaxColors::light()`・`dark()` の 13 個（`keyword`、`r#type`、`string`、`comment`、`operator`、`punctuation`、`number`、`constant`、`function`、`variable`、`tag`、`attribute`、`property`）を `kivact::ui::theme::syntax::<大文字>_VAR` に set する行を足す。`MACRO_VAR` ← function の値、`LIFETIME_VAR` ← constant の値。
  3. icon：`icons.rs` を 3.0.7 の icon の形（`ICON_NAMES` を出す）。`OnceLock` を消す。
  4. `lib.rs` に `code_syntax` の module と `pub use code_syntax::{CodeLanguage, CodeLanguageInput, code_highlight};`、`variants` の新しい型を足す。
- **消すもの**：`TokenKind`、soft の `type Tokenizer`、`code_highlight` の palette の引数、`OnceLock`。
- **状態の閉じ方**：なし。
- **試験**：なし（K43）。
- **確認（tier a）**：3.0.9 の途中の unit。
- **完了条件**：`rg -n "TokenKind|SyntaxColors|OnceLock" crates/kivact-ui-soft/src` が何も出さない。上の command が通る。

## K42：soft の side navigation を中核の sidebar に、timestamp・timer を `clock` に（W10）

- **wave / lane / 前提**：W10 / K5 / K41、K11、K12、R23（W5a の `kivact::task::clock`）
- **所有**：`crates/kivact-ui-soft/`
- **目的**：soft の `side_nav`（1.5k 行）を消し、中核の `sidebar` の上の見た目にする（KIT-10）。timestamp の i18n 化・`copyable` の削除・時計の共有（KIT-03 の残り、KIT-17）。lib を完成させる（3.0.6 の S4）。
- **手順**：
  1. sidebar：`side_navigation.rs` と `side_navigation/` を消す（`SideNav`、`SideNavCollapse`、`SideNavCollapseButton`、`SideNavItem`、`SideNavResize`、`SideNavSection`、`ItemCollapse`、`side_nav`、`side_nav_collapse_button`、`side_nav_item`、`side_nav_section`）。対応（呼出し側の書換えの正本。4.4.5 節にも載せる）：

     | 今 | 中核 |
     |---|---|
     | `side_nav(children)` | `sidebar().content(sidebar_menu(children))`（`sidebar_provider` の中） |
     | `.header(h)`、`.top_content(t)` | `.header(column((h, t)))` |
     | `.footer(f)`、`.footer_icons(i)` | `.footer(column((f, i)))` |
     | `.collapsible(SideNavCollapse::Controlled(collapsed))` | `sidebar_provider(open, ..)` の `open` は `collapsed` の否定の `binding`、`.collapsible(SidebarCollapsible::Icon)` |
     | `.collapse_button(true)`、`side_nav_collapse_button()` | `sidebar_trigger()` を呼出し側が置く |
     | `.resizable(SideNavResize::default().width(w).range(min, max))` | provider の `.resize(SidebarResize { width, min, max })` |
     | `side_nav_item(label).icon(i).selected_icon(s).selected(b).end_content(e).actions(a).children(c).collapsible(ItemCollapse::Controlled(open)).on_press(f)` | `sidebar_menu_button(label).icon(i).active_icon(s).active(b).end(e).action(a).sub(c).sub_open(open).on_press(f)` |
     | `side_nav_item(..).size(ControlSize::Sm)` | `.variant(SidebarMenuButtonSize::Sm)` |
     | `side_nav_section(title, children).subtitle(s).end_content(e).header_hidden(h)` | `sidebar_group(title).content(sidebar_menu(children)).description(s).action(e).label_hidden(h)` |

     soft の Astryx の形は `SidebarTheme` で入れる：`src/recipes_navigation_sidebar.rs`（新）に `pub(crate) fn sidebar() -> SidebarTheme` を置き、`recipes_navigation.rs` の `install` に `.with(crate::recipes_navigation_sidebar::sidebar())` を足す。part と、写す今の soft の style（消した `side_navigation/` の最後の版。`git show 2b8df5ac:crates/kivact-ui-soft/src/side_navigation/items.rs` と `shell.rs`）：`menu_button`・`menu_button_icon`・`menu_button_end`・`menu_button_chevron` ← `items.rs` の `side_row_style` と `icon_box`（`selected` を `s.active` に、`ControlSize` の高さを `s.size` の `Sm`・`Default`・`Lg` に対応させ `crate::metrics::CONTROL_SM`・`CONTROL_MD`・`CONTROL_LG` で読む）、`content` ← `shell.rs` の `sidebar_items_style`、`footer` ← `sidebar_bottom_style`、`resize_handle` ← `resize_grip` の style、`panel` ← `sidebar_width` と `sidebar_collapse` の中の panel の style（幅の計算は中核の `WIDTH`・`WIDTH_ICON` の変数の set で行い、recipe には色・境界・角丸だけを書く）、`group_label`・`group_description` ← `items.rs` の `side_nav_section` の title と subtitle の style。soft の theme（`set_core`）に sidebar の変数を足す：`kivact::ui::theme::sidebar::BACKGROUND_VAR` ← `BG_SURFACE`、`FOREGROUND_VAR` ← `TEXT_PRIMARY`、`ACCENT_VAR` ← `OVERLAY_HOVER`、`ACCENT_FOREGROUND_VAR` ← `TEXT_PRIMARY`、`PRIMARY_VAR` ← `ACCENT`、`PRIMARY_FOREGROUND_VAR` ← `ON_ACCENT`、`BORDER_VAR` ← `BORDER`、`RING_VAR` ← `FOCUS_RING`、`PAGE_VAR` ← `BG_BODY`、`ITEM_RADIUS_VAR` ← 今の `RadiusTokens::element` の値、`WIDTH_VAR` ← 今の soft の side nav の既定の幅（`rg -n "width" crates/kivact-ui-soft/src/side_navigation/shell.rs` の `SideNavResize::default` の値。git の最後の版で読む）。
  2. `app_shell.rs`：今 `side` の factory（`ViewFn`）を `side_navigation` の部品で包んでいた所を、`sidebar_provider(open, sidebar().content(factory()), page)` にする（desktop）。mobile の drawer は今のまま `mobile_nav(factory)`。`AppShell`・`MobileNavConfig`・`MobileNavMode`・`ShellHeight`・`ShellVariant` と `app_shell` を `#[component]` にする（3.0.5）。`mobile_nav.rs`（`mobile_nav`、`mobile_nav_toggle`、`use_app_shell_mobile` は hook のまま、`AppShellMobile`・`DrawerSide` は型）も変換する。K02 の `MobileContext` の印はそのまま。
  3. timestamp（KIT-03 の残り）：
     - `Timestamp` の builder を `#[component] pub fn timestamp(value: Value<TimestampValue>, ..)` にする（今の setter：`format`、`auto_threshold`、`hover_card`、`entries`、`time_zone`、`show_time_zone`、`live`、`kind`、`size`、`color`、`weight` を props に。既定は今の値）。`Rc<RefCell<TimestampSettings>>` と `TimestampSettings` は消える（props を本体が読む）。
     - `TimestampEntry::copyable`（doc が「効果なし」）を消す。
     - 時計：今の `machine(TimestampClock::Still { now: SystemTime::now() }, ..)` と `TimestampTick`、`TimestampClock`、`TimestampEvent` を消し、`let now = kivact::task::clock(Duration::from_secs(1));` を使う（`live` が偽のときは今と同じく `now` を読まずに mount の時刻 `SystemTime::now()` を一度使う）。
     - 名前：`calendar_text` の `MONTHS`・`WEEKDAYS` の表と `"{month_name} {day}, {year} at {clock}"`・`AM/PM` の組み立てを、`kivact::i18n` の `Locale` の月・曜日の名前と日付・時刻の書式（`rg -n "pub fn (month_name|weekday_name|format_date|format_time|hour_cycle)" crates/kivact-i18n/src`）にする。`Locale` は `kivact::i18n::use_locale()`（今 soft が locale を読む書き方。`rg -n "use_locale|locale\(\)" crates/kivact-ui-soft/src | head -3`）。`TimestampFormat::{SystemDate, SystemDateTime, SystemTime, UnixSeconds}` の ISO の文字列は今のまま。
  4. timer：`timer.rs` の Machine（`SystemTime::now()` の tick）を `kivact::task::clock(Duration::from_secs(1))` にし、`Timer` を `#[component] pub fn timer(..)` にする。
  5. `code_syntax.rs` に部品があれば変換する。
  6. 3.0.6 の S4：`-A dead_code` なしで lib を通し、dead の item を消す。
- **消すもの**：`side_navigation.rs`、`side_navigation/`、上の表の公開名、`TimestampSettings`、`TimestampClock`、`TimestampTick`、`TimestampEvent`、`MONTHS`・`WEEKDAYS`、`TimestampEntry::copyable`、timer の Machine。
- **状態の閉じ方**：`MobileContext`（K02 の印）。新しい型なし。
- **試験**：なし（K43）。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-ui-soft
  cargo check -p kivact-ui-soft --lib
  cargo clippy -p kivact-ui-soft --lib -- -D warnings
  cargo xtask check --no-write --crate kivact-ui-soft
  ```

- **完了条件**：`rg -n "SideNav|side_nav\b|ItemCollapse|copyable|SystemTime::now|impl (kivact::view::)?(Element|IntoView) for" crates/kivact-ui-soft/src` が `timestamp.rs` の `live` が偽のときの 1 行（`SystemTime::now`）だけを出す。上の command が通る。

## K43：soft の試験・catalog・README

- **wave / lane / 前提**：W10 / K5 / K42、K14
- **所有**：`crates/kivact-ui-soft/`（`tests/`、`README.md`）、`examples/kivact-ui-soft/`、`Cargo.lock`
- **目的**：TL-13、KIT-12、KIT-16、KIT-20 を soft に当てる。
- **手順**：
  1. root `Cargo.toml` は変えない（kit・catalog は workspace の member のまま。gate の `--exclude` が外していたのをやめるだけ。INC-09）。
  2. 試験：4.6 節の soft の表。`navigation.rs` の side nav の試験（`rail_navigation_has_one_actual_panel_and_rearms_after_expansion`、`domain_navigation_shares_hover_pinned_and_menu_focus_authority`、`shell_adapts_navigation_and_opens_the_mobile_drawer`）は K42 の表で中核の sidebar の部品に書き換え、期待値は変えない。`metrics.rs` の `MetadataLook::default().columns(columns)` は `.variant(columns)`。
  3. catalog（3.0.8）：今の `pages.rs` の 113 の `PageInfo { slug, name, category }` を、それぞれ一つの `Page::new(slug, name, category, include_str!(..))` にする。page の demo は、今の `docs/*.rs` の `match` で slug ごとに分かれた腕の view。腕ごとに `fn <slug の snake_case>() -> impl IntoView` を作り、`// example: <slug>` … `// end example` で囲み、`Example::new("<slug>", name, "", <その関数>)` で page に足す。page の file は今の `docs/<群>.rs`（`collections`、`content`、`controls`、`dates`、`feedback`、`forms`、`navigation`、`overlays`、`utilities`）のまま `src/pages/<群>.rs` に移し、`include_str!` はその群の file を指す（同じ群の page は同じ source を持つ。snippet は印で切り出すので page ごとに正しい）。`pages()` は今の `PAGES` の順に `Page` を並べる。消す：`src/pages.rs` の `PageInfo` と `PAGES`、`src/docs.rs`、`src/docs/examples.rs`（slug の文字列の `match` の連鎖）、`src/docs/specimen.rs`、`src/shell.rs`、`src/page.rs`、`src/model.rs`、`src/look.rs`、`src/fonts.rs`。`theme(mode)` は `kivact_ui_soft::theme(mode).with_fonts(FONTS.iter().copied())`（`FONTS` は今の 2 つの Figtree）。`START = "button"`。demo の中の API は新しい API（K28 の束の分解、K42 の sidebar の表、`TableRowAppearance::new(true, true, false)` → `.variant(TableRowDividers(true)).variant(TableRowHover(true))`）。
  4. `tests/catalog.rs`：`every_page_opens_through_search_in_both_appearances` と `search_opens_a_page_and_the_appearance_toggle_keeps_it_open` を消し、`every_page_opens_from_the_list_and_renders_cleanly`（`check::every_page_opens_cleanly("Soft", pages, THEME, &["carousel"])`）と `every_page_character_is_in_the_catalog_fonts` を置く。残りの 6 つ（`carousel_example_moves_the_slides_and_reveals_the_previous_control`、`carousel_example_snaps_after_horizontal_wheel_input`、`table_example_filters_and_expands_projects`、`standard_examples_keep_their_controls_when_the_appearance_changes`、`file_example_uses_picker_storage_and_clear_without_accepting_field_drops`、`item_list_examples_keep_native_positions_in_both_appearances`）は `check::mount` と `check::open` を使って今の操作と期待値のまま。`tests/lookbook.rs` は page を開く部分を `check::open` にし、期待値は変えない。
  5. `crates/kivact-ui-soft/README.md` を 3.0.10 の雛形にする（「Migrating from the previous release」（283 行目〜）と recipe の一覧を消す）。
- **消すもの**：上の catalog の file と試験、README の 2 つの節。
- **状態の閉じ方**：shell の signal の `xtask-allow(state-fields)`（`model.rs` の `CatalogState`）は file ごと消える。
- **試験**：上の catalog の 8 つと 4.6 節の soft の「残す」。
- **確認（tier a）**：3.0.9 の最後の unit と catalog（`<kit>` = `soft`）。 加えて pending の検査を strict で通す（INC-14）：`cargo xtask check --no-write --strict --only hand-element,element-shadow --crate kivact-ui-soft`。
- **完了条件**：`rg -n "PageInfo|match slug|specimen" examples/kivact-ui-soft/src` が何も出さない。上の command が通る。

---

### 3.6 W11a

## K44：`e2e_kits` を戻し、theme_conformance を新しい API にし、kit の共通の検査を一つにする

- **wave / lane / 前提**：W11a / K / K35、K37、K38、K40、K43
- **所有**：`dev/tests/kivact-kit-tests/Cargo.toml`、`dev/tests/kivact-kit-tests/tests/e2e_kits/`（全体）、4.6 節で「移す」とした kit の試験の file（`crates/kivact-ui-basic/tests/consumer/foundation.rs`、`toasts.rs`、`crates/kivact-ui-pop/tests/consumer/interactions.rs`、`overlays.rs`、`crates/kivact-ui-game/tests/consumer/overlays.rs`、`crates/kivact-ui-creative/tests/consumer/interactions.rs` と各 `tests/consumer.rs`）
- **目的**：DEC-14 で gate から外した `e2e_kits`（package `kivact-kit-tests`）を戻す（W10 の tier b は `--exclude kivact-kit-tests`。INC-12）。theme_conformance を `<Name>Theme` と theme 変数で書き直し、README の recipe の一覧の検査を消す（KIT-16）。kit ごとに写していた共通の検査（theme の切替え、toast の寿命）を 5 つの kit にかけた一つの試験にする（TL-13）。
- **手順**：
  1. `dev/tests/kivact-kit-tests/Cargo.toml` は変えない（T15 が作ったまま 5 つの kit の dev-dependency と `e2e_kits` の `[[test]]` を持つ。W6〜W10 は gate の `--exclude kivact-kit-tests` で外れていただけ。INC-09・INC-12）。
  2. `dev/tests/kivact-kit-tests/tests/e2e_kits/theme_conformance.rs`：
     - `kit_readme_lists_the_filled_recipes`、`filled_recipe_keys`、`filled_extended_recipe_keys`、`filled_feedback_recipe_keys`、`filled_content_recipe_keys`、`recipes_without` を消す。
     - `kit_recipes_paint(kit, mode, theme, screen)`：「kit の recipe を外すと描画が変わる」を、`recipes_without(kit, name)` の代わりに `theme.clone().with(<Name>Theme::default())`（U recipe 表の、`components()` の 12 の部品の `<Name>Theme`）で比べる。期待値（`painted_differently` が真）は変えない。
     - `all_kit_themes_preserve_standard_component_contracts` と `standard_button_respects_the_theme_focus_ring_offset` は新しい API（`Theme::light()` の focus ring offset は S 表の変数の `set`、部品は `#[component]` の呼び方、`.label(..)`）に直し、期待値は変えない。
     - 新しい試験を 2 つ足す（kit は今の 5 つの `(name, theme)` の表、mode は Light と Dark）：
       - `all_kit_themes_restyle_on_switch_and_keep_component_state`：`contract_view` を `provide_theme(move || if dark.get() { kit(Dark) } else { kit(Light) }, ..)` で mount し、text input に入力、checkbox を check、tab を選び、slider を動かす。`dark.set(true)` の後、4 つの値が同じで、`screen.errors()` が空で、`painted_differently`（前後の screenshot）が真。移す元：basic `theme_switch_restyles_mounted_components_and_keeps_their_state`、pop `theme_switch_changes_existing_nodes_and_pixels_without_remounting_state`・`theme_change_preserves_the_shared_tab_and_range_semantic_tree`、creative `creative_segments_and_numeric_semantics_survive_a_theme_change`（creative の numeric の部分は creative の `consumer/numeric_value.rs` に残す）。
       - `all_kit_themes_are_one_shared_theme_per_mode`（TV-09、PF-02）：5 つの kit の `theme(Light)` を二度呼んだ値が `==`（`Arc::ptr_eq` が先に判定する）、`theme(Light) != theme(Dark)`。game と creative は `theme_with(mode, ThemeOptions::default()) == theme(mode)`。
       - `all_kit_themes_keep_the_toast_lifetime`：`toast_region(use_toaster())` に title と description の toast を出し、live region の role で読め、pointer を乗せている間は時間が進んでも消えず、外して既定の時間が過ぎると消える。移す元：basic `toast_shows_title_description_and_leaves_after_five_seconds`、pop `notification_has_live_semantics_and_expires_on_the_host_clock`、game `consumer_toast_expiry_and_hover_pause`、creative `toasts_are_live_pause_for_the_pointer_and_expire_on_the_host_clock`。時間は移す元の試験の値（basic 5 秒など）ではなく中核の既定の時間（U 変数表の toast の時間の変数を `theme.get` で読む）で進める。
  3. 移した元の試験を消す：上の 6 つの関数（creative の numeric の部分を除く）を消し、file が空になったら file と `tests/consumer.rs` の `#[path]`・`mod` の行を消す。
  4. annex T（T15）が `e2e_kits` に入れた他の kit の試験（`presence_kit.rs`、`apg_grid_kit.rs`、`ui_standard.rs`）を新しい API に直す（`UiRecipes::default().toast(..)` は `ToastTheme::default().<part>(..)`、`kivact_ui_soft::theme(..)` などはそのまま）。期待値は変えない。
- **消すもの**：README の検査、`recipes_without` と `filled_*` の helper、移した 6 つの kit の試験。
- **状態の閉じ方**：なし。
- **試験**：`e2e_kits` の `theme_conformance` の 4 つ（2 つは新しい）と、annex T が `e2e_kits` に入れた試験。
- **確認（tier a）**：

  ```
  cargo fmt -p kivact-kit-tests
  cargo check -p kivact-kit-tests --all-targets
  cargo clippy -p kivact-kit-tests --all-targets -- -D warnings
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-kit-tests --test e2e_kits
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-basic -p kivact-ui-pop -p kivact-ui-game -p kivact-ui-creative
  cargo xtask check --no-write --crate kivact-kit-tests
  ```

- **完了条件**：`rg -n "kivact-recipes|filled_recipe_keys|UiRecipes" dev/tests` が何も出さない。`e2e_kits` が通る。

---

## 4. 他の付属書との接点

### 4.1 設計判断（parent が決めた）

1. **basic が `kivact-plot` に依存する**：parent が採用した（設計 4.2 を改めた）。K10 が `kivact_plot::nice::round_step` を公開し、K34 が basic の `Cargo.toml` に足し、basic の theme が `kivact_plot::color` の変数に chart の色を set する。K06 の `visible_to` の `kivact-plot` の行は `kivact-ui-basic` を含む。
2. **中核の `sidebar` も `<Name>Theme` を持つ**：parent の決定（設計 5.10）。K11 が `SidebarState`・`SidebarItemState`・`SidebarTheme`（annex U の `recipe_table!` の形、U 3 章の button の例）を `crates/kivact-ui/src/sidebar/look.rs` に作り、既定の recipe は `kivact::ui::theme::sidebar::*` の theme 変数を読む。K12 が soft のための part を足す。basic（K34）と soft（K42）は `SidebarTheme` の part で自分の形を入れ、変数で色と幅を入れる。
3. **kit の variant が中核の `<Component>Variant` と衝突するときは `<Component>Tone`**（V2）：parent が採用した（設計 5.10 を改めた）。

本付属書に未決の要設計判断は残っていない。

### 4.2 他の付属書の名前・仕組み（すべて照合済み・決定済み）

1. **DEC-14 の除外の仕組み**（解決済み、INC-09）：除外は gate の `--exclude` だけで行い、root `Cargo.toml` の `members`/`exclude` は変えない。kit・catalog・`kivact-kit-tests` は workspace の member のまま。本付属書の unit は root `Cargo.toml` の kit・catalog の行を変えない（K06 の新しい 2 crate の行だけを足す）。`kivact-kit-tests` の `Cargo.toml` は T15 が作ったまま（kit の dev-dependency と `e2e_kits` の `[[test]]` を持つ）で、K44 は変えない。
2. **S 表の名前**（解決済み、INC-18）：本付属書の code の中核の変数の名前は S 4.2 節の表のとおり：`color::{SURFACE, SURFACE_ALT, TEXT, TEXT_MUTED, ACCENT, ACCENT_TEXT, BORDER, FOCUS, DANGER, SUCCESS, WARNING, OVERLAY, SHADOW, SELECTION, HOVER, PRESSED, RAISED}`（const と `_VAR`）、`space::{XS, SM, MD, LG, XL}`、`radius::{SM, MD, LG}`、`font::{SIZE, SIZE_SMALL, SIZE_LARGE}`（const と `_VAR`）と `font::{FAMILY_VAR, WEIGHT_VAR, WEIGHT_STRONG_VAR, LINE_HEIGHT_VAR}`（static だけ）。`with_mode`・`with_focus_ring`・`with_focus_ring_offset` は `Theme` の builder のまま、control height・disabled opacity・overlay margin は `control::{HEIGHT_VAR, DISABLED_OPACITY_VAR, OVERLAY_MARGIN_VAR}` の set（T6、INC2 N-05）。`Color::mix` は `mix(other, t, &Theme)`、`Color::resolve(theme)` は `components(&theme)`（S 4.3）。font family の型は `FontFamily`（`FontFamily::SYSTEM`、`From<&str>`、`as_str()`）。
3. **部品の本体が `Element` の modifier の値を読む方法**（決定、設計 5.1 規則 10・12 章 API-4）：`#[component]` の引数 `#[caller] caller: Caller`（macro が吸収した modifier の snapshot を渡す）。K11 手順 5 が使う。
4. **別名**（決定、設計 12 章 UI-U-02・5.7、INC2 N-11）：すべての型で可。書き方は一つ：fallback は `ThemeVar::alias(name, &X_VAR)`、theme の中は `theme.set(&VAR, &X_VAR)`。kit の変数の fallback は T1a。
5. **basic chart の `text_width`**（決定済み、parent 採用）：K34 は `text_width`（字幅の見積り）を残す。chart の hit test は描画の外で行うので、plot が描画の中で測る幅は使えない。
6. **`#[prop(default = expr)]`**（決定済み）：annex C の C09 は `Option` でない全ての props の型（`Value<T>` を含む）に `#[prop(default = expr)]` を受ける。本付属書の `#[prop(default = ..)]` はそのまま書く。
7. **annex U の名前**（確定）：U recipe 表は annex U 4.4 節、U 変数表は 4.3 節。部品の module 表（4.4 節の最後）：`theme` = `crates/kivact-ui/src/theme.rs`、`Slant`・`Striped` = 新しい `crates/kivact-ui/src/variant_marks.rs`、`tabs` = `crates/kivact-ui/src/tabs.rs` と `tabs/`、`sidebar` = `crates/kivact-ui/src/sidebar.rs` と `sidebar/`（`mod.rs` は作らない）、`code_block` = `crates/kivact-ui/src/code_block.rs` と `code_block/`。`Look::variant::<V>()` は `Option<V>`。
8. **`cached_theme`**（決定、設計 12 章 PF-02）：作らない・使わない。kit の theme と icon set は `thread_local!` の static（3.0.7）。
9. **`Color::relative_luminance()`・`Color::contrast_ratio(other)`**（解決済み、INC-41）：annex S の S09 が作る。K31 が soft の `theme_scope.rs` で使う。
10. **e2e の directory**（解決済み、INC-10・INC-11）：annex T の分割の後、`dev/tests/kivact-public-api/tests/{e2e_core, e2e_ui, e2e_native}/` と、別の package `dev/tests/kivact-kit-tests/tests/e2e_kits/`（package `kivact-kit-tests`）がある。kit の部分の module は `apg_grid_kit`・`presence_kit`・`ui_standard`・`theme_conformance`。`crate::support::mount` は各 binary にある（K09・K11・K12・K13・K44）。
11. **`TestFonts::Only` の削除**（解決済み、INC-41）：annex C の C14。catalog-shell の `check::mount` は font を theme（`Theme::with_fonts`）で渡す。

### 4.3 本付属書が作り、他が使う中核の名前

| 名前 | 場所 | unit |
|---|---|---|
| `icons!($dir:literal, [$($name:literal),*])` | `kivact::style::icons` | K07 |
| `Slant`、`Striped`（marker、`Variant`） | `kivact::ui` | K08 |
| `SyntaxKind`（15 値）、`Tokenizer`、`syntax_ranges(line, tokens) -> Vec<StyledRange>`、`SyntaxKind::color(self) -> Color` | `kivact::ui` | K08 |
| 構文色の変数 `KEYWORD`・`TYPE`・`STRING`・`COMMENT`・`OPERATOR`・`PUNCTUATION`・`NUMBER`・`CONSTANT`・`FUNCTION`・`VARIABLE`・`TAG`・`ATTRIBUTE`・`PROPERTY`・`MACRO`・`LIFETIME`（と `_VAR`） | `kivact::ui::theme::syntax` | K08 |
| `NodeRef::once_visible(&self, fraction: f64) -> Value<bool>` | `kivact::view` | K09 |
| （使う側）`kivact::task::clock(every: Duration) -> Value<SystemTime>` | `kivact::task` | R23（W5a）が作る。K42 が使う |
| `kivact_plot::{plot, Plot, PlotModel, PlotData, PlotSeries, PlotRange, PlotAxis, AxisScale, TickFormat, LineDash, PlotView}`、`kivact_plot::color::{SERIES_1..SERIES_5, GRID, AXIS}`（と `_VAR`）、`kivact_plot::nice::round_step` | `kivact-plot` | K10 |
| `sidebar_provider`、`sidebar`、`sidebar_trigger`、`sidebar_group`、`sidebar_menu`、`sidebar_menu_button`、`sidebar_menu_sub_button`、`sidebar_separator`、`sidebar_input`、`sidebar_menu_skeleton`、`SidebarSide`、`SidebarFrame`、`SidebarCollapsible`、`SidebarMenuButtonSize`、`SidebarResize` | `kivact::ui` | K11、K12 |
| `SidebarTheme`（part：`root`、`panel`、`header`、`content`、`footer`、`rail`、`inset`、`trigger`、`group`、`group_label`、`group_action`、`group_content`、`group_description`、`menu`、`menu_button`、`menu_button_icon`、`menu_button_end`、`menu_button_chevron`、`menu_badge`、`menu_tooltip`、`menu_action`、`menu_sub`、`sub_button`、`separator`、`input`、`skeleton`、`resize_handle`）、`SidebarState`、`SidebarItemState` | `kivact::ui::theme` | K11、K12 |
| sidebar の変数 `BACKGROUND`・`FOREGROUND`・`PRIMARY`・`PRIMARY_FOREGROUND`・`ACCENT`・`ACCENT_FOREGROUND`・`BORDER`・`RING`・`PAGE`・`WIDTH`・`WIDTH_ICON`・`ITEM_RADIUS`（と `_VAR`） | `kivact::ui::theme::sidebar` | K11 |
| `TabOverflow::{Menu, Scroll}`、`tabs(..).overflow(..)` | `kivact::ui` | K13 |
| `catalog_shell::{catalog, Page, Example, PreviewAlign, CatalogTheme, FONTS, check}` | `examples/catalog-shell` | K14 |
| message の key `ui.sidebar.*`、`ui.sidebar.resize`、`ui.tabs.more`、`plot.value`、`plot.description` | | K10〜K13 |

付属書 G への引継ぎ：設計 4.2 の依存の図（4.1 の 1）、`docs/for-kivact-ui-user/` の kit の docs を 4.4 の表で直す、5.10 の kit の骨格（3.0.1）を architecture.md に書く。

### 4.4 kit の公開名の運命（今の `lib.rs` の `pub use` の全部）

記号：**残す**＝同じ名前（部品の builder は `#[component]` が同じ名前の struct を生成する）、**部品**＝関数を `#[component]` にする（関数名と struct 名は残る）、**改名 → X**、**消す**、**移す → path**。data・options・props の型は「残す」。

#### 4.4.1 basic（K15〜K19、K34）

| 今の名前 | 運命 |
|---|---|
| `AlertLook` | 改名 → `AlertTone`（V2） |
| `Attachment`、`attachment`、`attachment_group`、`attachment_action` | 部品（`attachment_action` は今の戻り値が view なら部品、data なら関数のまま） |
| `AttachmentMediaVariant`、`AttachmentSize`、`AttachmentState` | 残す |
| `BadgeVariant`、`CardSize`、`ChatLayout`、`EmptyMediaVariant`、`ItemMediaVariant`、`ItemSize`、`ItemVariant`、`TabsVariant`、`MarkerVariant`、`MenuVariant`、`AlertDialogSize`、`ToggleVariant`、`CodeBlockSize`、`DatePickerVariant`、`DisclosureVariant`、`DrawerVariant`、`FieldLegendVariant`、`InputGroupButtonSize`、`InputGroupControl`、`NativeSelectSize`、`OtpPattern` | 残す（variant は `variants.rs`） |
| `CardLook` | 消す → `CardSize` と新しい `CardHeaderBordered(pub bool)` |
| `ItemLook`、`CodeBlockLook`、`InputGroupButtonLook` | 消す → `ItemSize`、`CodeBlockSize`、`InputGroupButtonSize` |
| `BubbleVariant` | 改名 → `ChatBubbleVariant` |
| `CardHeader`、`card_header`、`Marker`、`marker`、`Menubar`、`menubar`、`ScrollArea`、`scroll_area`、`FieldSeparator`、`FieldSet`、`field_content`、`field_group`、`field_separator`、`field_set`、`InputOtp`、`input_otp`、`NativeSelect`、`native_select`、`Label`、`Separator`、`breadcrumb_ellipsis`、`item_separator`、`kbd_group`、`label`、`separator`、`table_caption`、`Questionnaire`、`questionnaire`、`TraceLine`、`trace`、`trace_collapsible`、`trace_line`、`blockquote`、`bullet_list`、`inline_code`、`large`、`lead`、`muted`、`paragraph`、`small` | 部品（`input_otp`・`native_select` の名前の位置引数は C6 で外す） |
| `QuestionnaireAnswer`、`QuestionnaireChoice`、`QuestionnaireStep` | 残す |
| `Chart`、`chart` | 部品（K34） |
| `ChartConfig`、`ChartKind`、`ChartRow`、`ChartSeries` | 残す |
| `Sidebar`、`SidebarCollapsible`、`SidebarGroup`、`SidebarMenuButton`、`SidebarMenuButtonSize`、`SidebarMenuSubButton`、`SidebarSide`、`SidebarTrigger`、`sidebar`、`sidebar_group`、`sidebar_input`、`sidebar_menu`、`sidebar_menu_button`、`sidebar_menu_skeleton`、`sidebar_menu_sub_button`、`sidebar_provider`、`sidebar_separator`、`sidebar_trigger` | 移す → `kivact::ui::*`（K11。`Sidebar::{side, variant, collapsible}` は provider の props。basic の Lyra の形は basic の theme が入れる `SidebarTheme` の part（K34）） |
| `SidebarVariant` | 移して改名 → `kivact::ui::SidebarFrame` |
| `CodeLine`、`CodeSpan`、`rust_lines`、`rust_source` | 残す（`CodeSpan.tone` → `kind: Option<SyntaxKind>`、`rust_lines` の引数から色を外す。K34） |
| `CodeTone` | 消す → `kivact::ui::SyntaxKind` |
| `ColorName` | 消す → `basic::<NAME>` の変数（K15 の表） |
| `ICON_NAMES` | 残す（`icons!` の一覧から作る） |
| `AxisScale`、`LineDash`、`Plot`、`PlotAxis`、`PlotData`、`PlotModel`、`PlotRange`、`PlotSeries`、`PlotView`、`TickFormat`、`plot` | 移す → `kivact_plot::*`（`plot` の名前の位置引数は外す） |
| `theme` | 残す |
| `CodeColors`、`Tokens` | 消す（変数と中核の構文色） |
| 新しい名前 | `BACKGROUND` などの色の変数（と `_VAR`）、`CardHeaderBordered`、`ChatBubbleVariant`、`AlertTone` |

#### 4.4.2 pop（K20、K21、K36）

| 今の名前 | 運命 |
|---|---|
| `ButtonTone` | 残す（V2。中核に `ButtonVariant` がある） |
| `Slant`、`Striped` | 消す → `kivact::ui::{Slant, Striped}`（K36） |
| `ChipList`、`RemovableChip`、`chip_list`、`removable_chip`、`empty_illustration`、`Thread`、`thread`、`NotificationMenu`、`notification_menu`、`AgendaItem`、`agenda_item`、`resizable_text_area`、`tab_label`、`TimelineItem`、`timeline_item`、`upload_row` | 部品 |
| `NotificationItem` | 残す |
| `ColorName`、`Tokens` | 消す → 色・長さの変数 |
| `AvatarLook` | 消す → `AvatarSize(pub f64)`、`AvatarHue(pub f64)`、`AvatarHalo(pub bool)`、`AvatarOnline(pub bool)` |
| `BadgeHue`、`ItemHue`、`StatHue`、`ProgressHue` | 消す → `.variant(Hue::X)` か `.variant(hue_signal)` |
| `WeekendHue` | 消す → `CalendarWeekdayHues(pub [Option<Hue>; 7])` |
| `BadgeVariant`、`SectionNumber`、`DiamondSteps`、`Segments`、`Hue`、`theme` | 残す（`Hue` に `colors()`・`accent()`） |
| `PopBubble` | 改名 → `ChatBubbleVariant` |
| `SpinnerLook` | 改名 → `SpinnerVariant` |

#### 4.4.3 game（K26、K27、K39）

| 今の名前 | 運命 |
|---|---|
| `chevron`、`tool_button`、`card_label`、`navigation_label`、`activity`、`Kpi`、`kpi`、`IconMenu`、`icon_menu`、`halo`、`hud`、`mono` | 部品 |
| `cut_corner` | 残す（`Shape` を返す関数） |
| `ChoiceCard` | 残す（marker） |
| `Navigation`、`Boxed` | 消す → `TabsVariant::{Navigation, Boxed}` |
| `ColorName`、`Tokens` | 消す → 色・長さの変数、`DISPLAY_FAMILY_VAR`・`MONO_FAMILY_VAR`（static だけ） |
| `ActivityEntry`、`AvatarVariant`、`BadgeVariant`、`SkeletonVariant`、`HeadingVariant`、`StatVariant`、`Density`、`theme`、`theme_with` | 残す |
| `ButtonTone` | 消す → `ButtonVariant::Secondary`（navy） |
| `Slant`、`Striped` | 消す → `kivact::ui::{Slant, Striped}`（K39） |
| `GameInputGroupVariant` | 改名 → `InputGroupVariant` |
| `GaugeSegments` | 残す（中身を `Value<usize>` から `usize` に） |
| `GameCellVariant` | 改名 → `TableCellVariant` |
| `ThemeOptions` | 残す（field に `display_font`、`mono_font` を足す。`Copy` でなくなる） |

#### 4.4.4 creative（K22〜K25）

| 今の名前 | 運命 |
|---|---|
| `Meter`、`MixerStrip`、`Transport`、`meter`、`mixer_strip`、`transport`、`BezierCurve`、`bezier_curve`、`color_popup`、`ColorWheel`、`color_wheel`、`CurvePreview`、`curve_preview`、`Dock`、`dock`、`DopeSheet`、`dope_sheet`、`Arrangement`、`arrangement`、`GraphEditor`、`graph_editor`、`NodeEditor`、`node_editor`、`SplitButton`、`ToolWindow`、`popup_button`、`split_button`、`tool_window`、`tool_window_button`、`GradientEditor`、`gradient_editor`、`Panel`、`media_tile`、`panel`、`tree_row`、`Outliner`、`outliner`、`panel_group`、`SearchPicker`、`search_picker`、`PieMenu`、`pie_menu`、`PropertyRow`、`PropertyStateButton`、`property_row`、`property_state_button`、`Range`、`range`、`DocumentTabs`、`document_tabs`、`Timeline`、`timeline_editor`、`fader`、`knob`、`number`、`value_slider` | 部品（`range` の名前の位置引数は C6 で外す） |
| `Numeric` | 消す（`number`・`value_slider`・`knob`・`fader` が `Number`・`ValueSlider`・`Knob`・`Fader` を生成） |
| `ChannelState`、`ArrangementState`、`Interpolation`、`Curve`、`CurveKey`、`CurveTangent`、`GraphRequest`、`GraphView`、`KeyMove`、`GraphState`、`NodeControl`、`NodeControlValue`、`NodeData`、`NodeLink`、`NodeRequest`、`NodeSocket`、`NodeState`、`Clip`、`EditorContext`、`EditorFocus`、`ItemTime`、`Lane`、`Keyframe`、`TimeRequest`、`TimeState`、`OutlinerState`、`TransportState`、`CubicBezier`、`ColorModel`、`color_from_hsv`、`color_to_hsv`、`DockAxis`、`DockChild`、`DockLayout`、`DockNode`、`DockPlacement`、`DockSide`、`DockTab`、`DockTabs`、`ArrangementRequest`、`AutomationPoint`、`TimelineMarker`、`TimelineTool`、`TimelineTrack`、`TrackFlag`、`TrackKind`、`EditorEvent`、`RulerTicks`、`NoticeVariant`、`ToolWindowApi`、`ToolWindowOptions`、`WindowPosition`、`GradientStop`、`KeyGlyph`、`NumericKind`、`PropertyState`、`evaluate_expression`、`Restriction`、`RestrictionFlag`、`TreeDrop`、`TreeItem`、`InspectorPanel`、`PickerItem`、`PieEntry`、`PieMode`、`PieState`、`ThemeOptions`、`theme`、`theme_with`、`Accent`、`Density`、`LabelTone` | 残す（`LabelTone::color` は引数なし） |
| `ColorName`、`Tokens` | 消す → 色・長さの変数、`MONO_FAMILY_VAR`（static だけ） |

#### 4.4.5 soft（K28〜K33、K41、K42）

| 今の名前 | 運命 |
|---|---|
| `AlertDialog`、`alert_dialog`、`AppShell`、`app_shell`、`avatar_silhouette`、`AvatarStatusDot`、`avatar_status_dot`、`banner_icon`、`Blockquote`、`blockquote`、`blockquote_text`、`BottomSheet`、`bottom_sheet`、`bottom_sheet_flow`、`ChatDictationButton`、`chat_dictation_button`、`CheckboxList`、`checkbox_list`、`checkbox_list_item`、`radio_list`、`Citation`、`citation`、`Code`、`code`、`DialogHeader`、`DrawerHeader`、`dialog_header`、`drawer_header`、`Divider`、`divider`、`StatusDot`、`status_dot`、`FieldStatus`、`field_status`、`FormLayout`、`form_layout`、`form_row`、`LayoutGrid`、`layout_grid`、`hover_card_text`、`IconView`、`icon_view`、`Indicator`、`check_indicator`、`checkbox_indicator`、`radio_indicator`、`keyboard_hint`、`Layout`、`layout`、`layout_content`、`layout_header`、`layout_panel`、`Lightbox`、`lightbox`、`Link`、`link`、`Markdown`、`markdown`、`MediaOverlay`、`media_overlay`、`MobileNav`、`MobileNavToggle`、`mobile_nav`、`mobile_nav_toggle`、`MoreMenu`、`more_menu`、`NavHeadingMenu`、`NavHeadingMenuItem`、`SideNavHeading`、`nav_heading_menu`、`nav_heading_menu_item`、`side_nav_heading`、`top_nav_heading`、`nav_icon`、`Outline`、`outline`、`PowerSearch`、`power_search`、`RichTextEditor`、`rich_text_editor`、`ScrollableArea`、`scrollable_area`、`Section`、`section`、`ChatToolCalls`、`chat_tool_calls`、`OverflowList`、`overflow_list`、`TableSelectionToolbar`、`table_row_index`、`table_row_status`、`table_selection_toolbar`、`center`、`hstack`、`stack_item`、`vstack`、`StyledText`、`styled_text`、`Thumbnail`、`thumbnail`、`Timer`、`timer`、`Timestamp`、`timestamp`、`Token`、`token`、`TopNav`、`TopNavItem`、`TopNavMegaMenu`、`TopNavMegaMenuFeaturedCard`、`TopNavMegaMenuItem`、`TopNavMenu`、`top_nav`、`top_nav_featured_card`、`top_nav_item`、`top_nav_mega_menu`、`top_nav_mega_menu_item`、`top_nav_menu`、`top_nav_menu_item`、`VisuallyHidden`、`visually_hidden`、`pagination_count`、`pagination_page_summary`、`pagination_status` | 部品（view を返さない関数は関数のまま） |
| `with_card_insets`、`with_key_hints`、`use_lightbox`、`LightboxHandle`、`use_app_shell_mobile`、`AppShellMobile`、`media_theme`、`MediaMode`、`code_highlight`、`CodeLanguage`、`CodeLanguageInput` | 残す（`code_highlight` の色の引数を外す。K41） |
| `MobileNavConfig`、`MobileNavMode`、`ShellHeight`、`ShellVariant`、`DrawerSide`、`DialogPurpose`、`SheetHeight`、`SnapPoint`、`AvatarSize`、`AvatarStatus`、`BreadcrumbsVariant`、`CardVariant`、`ChatDensity`、`ComposerElevation`、`SystemMessageVariant`、`DictationState`、`CitationSource`、`CitationVariant`、`CodeColor`、`EdgeCompensation`、`ChevronPosition`、`DisclosureDensity`、`DisclosureVariant`、`DividerVariant`、`Elevation`、`EmptyStateVariant`、`StatusDotVariant`、`FieldStatusVariant`、`StatusKind`、`FormDirection`、`FormLayoutCx`、`Optionality`、`GridColumns`、`GridRepeat`、`IconColor`、`IconSize`、`ICON_NAMES`、`ItemAlign`、`ItemDensity`、`ItemLayout`、`ListEdge`、`LayoutHeight`、`LayoutInset`、`LayoutRegion`、`LightboxMedia`、`LinkColor`、`ContentAlign`、`HeadingLinks`、`MarkdownDensity`、`OverlayPosition`、`OverlayScrim`、`ShowOn`、`Breakpoint`、`OutlineDensity`、`OutlineItem`、`SearchFilter`、`SearchOperator`、`BadgeVariant`、`CodeBlockContainer`、`CodeBlockSize`、`CodeBlockWidth`、`DropzoneVariant`、`ButtonGroupVariant`、`ToolbarVariant`、`HeadingDisplay`、`LabelPosition`、`MetadataColumns`、`SkeletonRadius`、`RichDoc`、`ScrollAxis`、`SegmentedLayout`、`ToolCall`、`ToolStatus`、`CollapseFrom`、`OverflowMeasure`、`RowStatusDot`、`RowStatusTone`、`SpinnerShade`、`SpinnerVariant`、`CenterAxis`、`ContainerInset`、`Space`、`StackAlign`、`StackItem`、`StackItemSize`、`StackWrap`、`StepCollapsedVariant`、`StepDensity`、`StepIndicatorPosition`、`TextColor`、`TextJustify`、`TextKind`、`TextSize`、`TextWeight`、`TextWrap`、`TruncateTooltip`、`WordBreak`、`theme`、`RemoveReveal`、`TimerFormat`、`InstantFormat`、`TimeZone`、`TimestampEntry`、`TimestampFormat`、`TimestampValue`、`TokenColor` | 残す（`TimestampEntry` から `copyable` を消す） |
| `Edges` | 改名 → `SectionEdges`（中核の `Edges` と重なる） |
| `BannerContainer` | 改名 → `BannerVariant` |
| `BubbleVariant` | 改名 → `ChatBubbleVariant` |
| `AvatarAppearance`、`AvatarGroupAppearance`、`BannerAppearance`、`CardAppearance`、`ChatAppearance`、`CodeAppearance`、`PaginationAppearance`、`StepperAppearance`、`TableCellAppearance`、`TableRowAppearance`、`MetadataLook`、`ItemLook`、`ListLook`、`SegmentedLook`、`TreeLook` | 消す → K28 手順 8 の表の variant（新しい newtype：`AvatarGroupShape`、`CardPadding`、`PaginationSize`、`TableCellAlign`、`TableCellDivider`、`TableRowDividers`、`TableRowHover`、`TableRowStriped`、`MetadataLabelWidth`、`MetadataOrientation`、`ItemHighlighted`、`ListDividers`、`SegmentedIconOnly`、`TreeGuides`） |
| `Colors`、`DataColors`、`HueColors`、`Hues`、`SyntaxColors`、`ColorName`、`Tokens`、`MotionTokens`、`RadiusTokens`、`Sizes`、`SpaceTokens`、`TypeStyle`、`TypeTokens`、`Shadows` | 消す → 色・間隔・角丸の公開の変数と crate の中の定数（K28） |
| `TokenKind` | 消す → `kivact::ui::SyntaxKind` |
| `syntax_theme` | 消す（`provide_theme` と `Theme::set`） |
| `RevealContent`、`RevealForce`、`RevealVisibility`、`reveal_container`、`reveal_content`、`container_below`、`is_compact`、`viewport_below`、`viewport_from`、`aspect_ratio`、`icon_style`、`link_with`、`layout_footer`、`rich_text_toolbar`、`rich_text_view`、`AxisScroll`、`OverflowOptions`、`ScrollOverflow`、`ScrollState`、`use_overflow`、`use_scroll_overflow`、`use_scroll_state`、`ChatToken`、`ChatTokenTone`、`StreamSpeed`、`chat_token`、`chat_tokenized_text`、`streaming_text`、`use_streaming_text`、`FlexStack`、`flex_stack`、`GridSpan`、`GridSpanItem`、`grid_span` | 消す（KIT-09。`viewport_below`・`icon_style`・`link_with`・`rich_text_view` は crate の中で `pub(crate)`） |
| `Tab`、`TabEntry`、`TabLayout`、`TabList`、`TabMenu`、`TabMenuOption`、`TabOverflow`、`TabPattern`、`tab`、`tab_list`、`tab_menu` | 消す → 中核の `tabs` と `TabOverflow`、soft の `TabsVariant`・`TabsLayout`・`TabsDivider`・`TabsFullBleed`（K41） |
| `ItemCollapse`、`SideNav`、`SideNavCollapse`、`SideNavCollapseButton`、`SideNavItem`、`SideNavResize`、`SideNavSection`、`side_nav`、`side_nav_collapse_button`、`side_nav_item`、`side_nav_section` | 消す → 中核の `sidebar` の部品（K42 の対応表）。soft の形は soft の theme が入れる `SidebarTheme` の part（K42）。`SideNavHeading` などの見出しの部品は残す |
| 新しい名前 | 色の変数（`BG_BODY` など、`HUE_*`、`DATA_*`、`SYNTAX_*`）、`SPACE_*`、`RADIUS_*`（と `_VAR`）、上の newtype、`TabsVariant`、`TabsLayout`、`TabsDivider`、`TabsFullBleed`、`SectionEdges`、`BannerVariant`、`ChatBubbleVariant` |

### 4.5 kit ごとの引継ぎの注意

- U 変数表に行が無い `UiTokens` の field（例 game の `ui.button_group.text_background`）は、その値が kit の色なら、kit の recipe（`<Name>Theme`）でその色を使う style を足す。値が長さなら、kit の `size.rs` の変数にして recipe で使う。どちらも U の部品の既定の見た目と同じになることを、kit の catalog の page を開いて `screen.errors()` が空であることで確かめる（K35・K37・K38・K40・K43 の walker）。
- `dev/xtask/policy/workspace.toml` の `examples/kivact-ui-basic/src/docs.rs` の例外は K06（W9）が `item = "docs::pages"` に直し、K35 が module 名 `docs` を保つ（W10・W11a の K の unit は workspace.toml を持たない）。
- `kivact-plot` は recipe の状態（`<Name>State`）を持たない（`plot` は theme 変数の色だけを読む）ので、`dev/xtask/policy/workspace.toml` の `recipe_state_packages` を変えない（INC-40）。`catalog-shell` も同じ。
- `dev/bench/run.sh` は `kivact-ui-basic-catalog` の package 名と features `gpu`・`complex-scripts` を使う。K35 は変えない。開始の page は今と同じ `button`。

### 4.6 kit の試験の判定（TL-13、KIT-20）

記号：**残す**（新しい API に直す）、**消す**（右の e2e が同じ振る舞いを確かめる）、**移す**（K44 が theme_conformance の共通の試験にする）、**→ controls.rs**（標準部品の配線の試験として `tests/consumer/controls.rs` に集める。600 code 行まで）。

basic（`crates/kivact-ui-basic/tests/consumer/`）：

| file | 試験 | 判定 |
|---|---|---|
| `code_block.rs` | `code_block_reports_copy_requests_and_shows_copied_for_two_seconds` | → controls.rs |
| `data.rs` | `data_table_filters_rows_and_returns_to_the_first_page`、`data_table_sorts_with_the_header_buttons` | 消す（`e2e_ui/ui_data_table.rs`、`interaction_table.rs`） |
| `data.rs` | `every_chart_kind_draws_its_series_in_their_colors_in_both_themes`、`the_default_value_text_is_written_in_the_app_locale`、`hovering_a_chart_shows_the_tooltip_card_with_the_values` | 残す（file を `chart.rs` に改名） |
| `display.rs` | `pagination_pages_follows_its_binding_and_disables_the_ends` | → controls.rs |
| `forms.rs` | `questionnaire_submits_once_and_records_skips`、`input_otp_fills_slots_completes_and_backspaces`、`native_select_chooses_by_keyboard_and_list` | 残す（分野の部品） |
| `forms.rs` | `field_shows_label_description_and_error` | → controls.rs |
| `forms.rs` | `radio_group_arrows_move_and_select`、`slider_keys_and_drag_change_the_binding`、`combobox_filters_chooses_and_clears`、`calendar_moves_with_keys_changes_month_and_skips_disabled_dates`、`date_picker_opens_a_popover_picks_a_day_and_closes` | 消す（`apg_radio_group.rs`、`apg_slider.rs`、`apg_combobox.rs`、`interaction_calendar.rs`、`apg_date_picker.rs`） |
| `foundation.rs` | `theme_switch_restyles_mounted_components_and_keeps_their_state` | 移す |
| `foundation.rs` | `checkbox_and_switch_toggle_with_pointer_and_keyboard` | 消す（theme_conformance の contract） |
| `foundation.rs` | `text_input_edits_its_binding_and_rings_on_focus` | → controls.rs |
| `navigation.rs` | `dropdown_menu_navigates_by_keyboard_and_skips_labels`、`context_menu_opens_at_the_pointer_and_by_keyboard` | 消す（`apg_menu_button.rs`、`interaction_menu.rs`） |
| `navigation.rs` | `navigation_menu_shows_one_panel_at_a_time` | → controls.rs |
| `navigation_layout.rs` | `split_drags_and_keys_move_the_boundary_within_min_sizes`、`carousel_steps_with_buttons_and_keys_and_stops_at_the_ends` | 消す（`apg_window_splitter.rs`、`apg_carousel.rs`） |
| `navigation_layout.rs` | `line_tabs_select_move_and_mark_the_selected_tab_with_an_underline` | → controls.rs |
| `navigation_layout.rs` | `sidebar_toggles_by_trigger_shortcut_and_rail_between_256_and_48` | 消す（K11 で `e2e_ui/ui_sidebar.rs` に移った） |
| `overlays.rs` | `accordion_in_single_mode_keeps_one_section_open`、`alert_dialog_ignores_outside_presses_and_reports_its_buttons`、`popover_opens_from_its_trigger_and_dismisses_with_focus_restored`、`hover_card_opens_after_its_delay_and_stays_while_hovered`、`command_filters_groups_and_chooses_with_the_keyboard` | 消す（`apg_accordion.rs`、`interaction_dialog.rs`、`interaction_dialog.rs`、`interaction_delayed.rs`、`interaction_palette.rs`） |
| `overlays.rs` | `sheet_attaches_to_each_side_and_closes_on_escape` | → controls.rs |
| `toasts.rs` | `toast_shows_title_description_and_leaves_after_five_seconds` | 移す |
| `plot.rs` | 9 つ | K10 で `kivact-plot` に移った |

pop：

| file | 試験 | 判定 |
|---|---|---|
| `controls.rs` | `button_states_and_user_modifiers_use_public_contract`、`checkbox_switch_and_mixed_states_are_accessible` | 残す |
| `controls.rs` | `toggles_and_accordion_follow_the_shared_toggle_rules`、`slider_drag_keyboard_and_accessibility_share_one_binding` | 消す（theme_conformance の contract、`apg_slider.rs`） |
| `data.rs` | `table_search_sort_selection_paging_are_external_consumer_behavior` | 消す（`interaction_table.rs`、`ui_data_table.rs`） |
| `dates.rs` | 2 つ | 消す（`apg_date_picker.rs`、`interaction_calendar.rs`） |
| `forms.rs` | `inputs_and_tag_editor_commit_native_edits` | → controls.rs |
| `forms.rs` | `dropzone_reports_public_file_events_and_platform_picker_results` | 消す（`ui_dropzone.rs`） |
| `interactions.rs` | `theme_change_preserves_the_shared_tab_and_range_semantic_tree`、`theme_switch_changes_existing_nodes_and_pixels_without_remounting_state` | 移す |
| `interactions.rs` | `tooltip_shows_on_focus_below_the_trigger_and_wraps_a_long_label` | 消す（`apg_tooltip.rs`） |
| `overlays.rs` | `notification_has_live_semantics_and_expires_on_the_host_clock` | 移す |
| `overlays.rs` | `modal_dismisses_and_restores_focus_to_its_opener`、`command_search_rebuilds_rows_and_executes_the_filtered_action`、`menu_skips_disabled_items_and_restores_the_custom_trigger` | 消す（`interaction_dialog.rs`、`interaction_palette.rs`、`apg_menu_button.rs`） |

game：

| file | 試験 | 判定 |
|---|---|---|
| `controls.rs` | `consumer_buttons_checks_mixed_states_and_modifiers` | 残す |
| `controls.rs` | `consumer_checkbox_switch_and_disclosure_follow_the_shared_toggle_rules`、`consumer_table_preserves_selection_and_updates_cells` | 消す（contract、`interaction_table.rs`） |
| `interactions.rs` | `consumer_editing_tags_numbers_native_picker_and_clipboard` | → controls.rs |
| `interactions.rs` | `tree_is_one_tab_stop_with_arrow_keys_and_selects_only_leaves`、`dropzone_failure_clears_on_the_next_drop`、`pagination_moves_focus_only_when_the_page_changes` | 消す（`apg_tree_view.rs`、`ui_dropzone.rs`、`ui_navigation.rs`） |
| `overlays.rs` | `consumer_toast_expiry_and_hover_pause` | 移す |
| `overlays.rs` | `consumer_modal_menu_command_search_and_focus_restore`、`consumer_tooltip_focus_delay_and_blur_use_setup_bound_observation`、`consumer_dialog_drawer_and_popover_are_named_dialogs_that_assistive_technology_can_dismiss` | 消す（`interaction_dialog.rs`・`interaction_menu.rs`・`interaction_palette.rs`、`apg_tooltip.rs`、`interaction_dialog.rs`） |
| `theme.rs` | `consumer_theme_density_and_halo_pixels_follow_public_theme` | 残す（density と halo は game だけ） |

creative：`interactions.rs` の `creative_segments_and_numeric_semantics_survive_a_theme_change`（theme の部分）と `toasts_are_live_pause_for_the_pointer_and_expire_on_the_host_clock` は移す。`check_switch_toggle_and_disclosure_follow_the_shared_toggle_rules`（contract）、`modal_contains_input_and_restores_opener_focus`（`interaction_dialog.rs`）、`nested_menu_keys_choose_a_leaf_key_and_delayed_tooltip_dismisses`（`apg_menu.rs`、`apg_tooltip.rs`）は消す。他の 22 の file の試験は分野の部品なので全部残す（TL-13：node editor、dock、color wheel、arrangement、numeric、bezier）。`interactions.rs` の `reordered_document_tabs_keep_identity_and_navigate_in_visible_order` は残す。

soft：

| file | 判定 |
|---|---|
| `progress_motion.rs`、`tab_indicator_motion.rs`、`toast_motion.rs` | 消す（時間の厳密な比較。設計 6.4） |
| `slider_touch_motion.rs`、`switch_touch.rs` | 消す（`apg_slider.rs`、theme_conformance の contract と `touch_scroll.rs`） |
| `collections.rs` の `tree_list_expands_and_collapses_with_arrow_keys`、`reactive_list_adds_a_list_item` | 消す（`apg_tree_view.rs`、`ui_collections.rs`）。`chat_composer_submits_trimmed_text_and_clears_draft_on_enter` は `chat_keyboard.rs` に移して残す |
| `button_busy.rs`、`chat_keyboard.rs`、`chat_scroll.rs`、`date_controls.rs`、`date_touch.rs`、`disabled_menu.rs`、`empty_state_announcement.rs`、`form_layout.rs`、`foundation.rs`、`grouped_table.rs`、`menu_press.rs`、`metrics.rs`、`navigation.rs`、`overlay_interaction.rs`、`rich_blocks.rs`、`search.rs`、`selector_focus.rs`、`selector_keyboard.rs`、`selector_touch.rs`、`table_column_filters.rs`、`table_row_status.rs`、`table_sticky_columns.rs`、`table_tree.rs`、`text_truncation_tooltip.rs`、`toast_swipe.rs`、`toggle_label.rs`、`tree_reveal.rs`、`typed_table_filters.rs` | 残す（soft の分野の部品か、soft の `InteractionPolicy` に依る振る舞い。selector・toast の swipe は soft の policy の値に依るので共通にしない）。`selector_touch.rs` の `selection_field_reopened_through_the_caller_binding_takes_a_fresh_surface` は `selection_field` が消えた（設計 5.9）ので、`select(..).trigger(view)` で同じ操作をする形に直す |

### 4.7 catalog の page の category

- basic：全 page を `"Components"`（shadcn の docs と同じ一つの一覧）。
- pop：`overview` は `"Overview"`、他の 12 は `"Components"`。
- game：`foundations` は `"Overview"`、他の 10 は `"Console"`。
- creative：今の section の group（`Category::{Controls, Audio, Layout, Animation, Compositing, Feedback}`）の英語名。
- soft：今の `PageInfo.category` のまま。

## 5. 自己点検

### 5.1 採用・変更して採用の所見と unit

| 所見 | unit |
|---|---|
| KIT-01 | K02（W2a） |
| KIT-02 | K03（W2a） |
| KIT-03 | K05（日付の計算、W2a）、K42（`Rc<RefCell>` の削除・i18n の書式・`copyable` の削除・R23 の `clock`） |
| KIT-04 | K20（hue の newtype と `WeekendHue`）、K27（game `ButtonTone`）、K08（中核の `Slant`・`Striped`）、K36・K39（kit の写しの削除）、`Element::variant(impl IntoValue<V>)` は annex C |
| KIT-05 | K08（`SyntaxKind`・構文色）、K34（basic）、K41（soft） |
| KIT-06 | 3.0.1 の骨格、K15〜K19、K20〜K21、K22〜K25、K26〜K27、K28〜K33（改名・recipe の置き場・lib.rs） |
| KIT-07（変更して採用） | K15、K20、K22、K26、K28（`ColorName` ごと theme 変数） |
| KIT-08 | K07（`icons!`）、3.0.7（theme と icon set の `thread_local!`。PF-02）、K34・K36・K38・K39・K41（kit の置換） |
| KIT-09 | K28（使われない関数・`aspect_ratio`・`tab_list`）、K13（中核の overflow）、K41（soft の tab の見た目） |
| KIT-10 | K10（目盛りの共有）、K11・K12（中核の sidebar と `SidebarTheme`）、K34（basic の `SidebarTheme` の part）、K42（soft の `SidebarTheme` の part）。chat の recipe は kit ごと（監査の判断のまま） |
| KIT-11 | K04（halo、W2a）、K09（`once_visible`）、K39（kpi） |
| KIT-12 | K14（catalog-shell）、K35・K37・K38・K40・K43（英語化と移行） |
| KIT-13 | K26（`ThemeOptions` の font）、K20（pop の literal の font）、K40（catalog と README） |
| KIT-14 | 3.0.2 の T5・T8（`Tokens::fallback` の確保と recipe の `Tokens` の読みが無くなる）、3.0.3 の R8・K20 手順 2（gradient の色を theme の set に）。中核の `ext` の fallback の memo（ST-07）は annex S |
| KIT-15 | K06、K10 |
| KIT-16 | K35・K37・K38・K40・K43（README）、K44（README の検査の削除）、W1 の annex T（kit の `xtask-allow(state-fields)` の削除）。日本語の docs は G |
| KIT-17 | K31（`Color::relative_luminance`・`contrast_ratio` を soft で使う。関数は annex S）、K42（R23 の `task::clock` を使う） |
| KIT-18 | K01（W2a） |
| KIT-19 | K28（公開を色・間隔・角丸に、policy を `theme_policy.rs` に） |
| KIT-20 | K14（`check`）、K35・K37・K38・K40・K43 |
| TL-13 | 4.6 節の表、各 kit の catalog の unit（消す・残す）、K44（移す） |

### 5.2 設計の判断を実装者に残していないか

- 名前・署名・行き先・変数名はすべて書いた。他の付属書の名前に依る所は 4.2 節に照合済み・決定済みとして書いた（代わりの手順は残していない）。
- 「必要なら」と読める所は、判定の command と両方の場合の操作を書いた（例：K03 の `feed` が closure を受けないときの memo、K07 の `IconSet` の field の可視性、K36 の import）。
- 要設計判断は残っていない（4.1 の 3 つは parent が決めた）。
- `<Component>Tone`（V2）の当て方は全 kit で同じ：今の中核の `AlertVariant`・`ButtonVariant`・`ChatMessageVariant`・`ProgressVariant`・`RadioGroupVariant` と衝突する kit の型は basic `AlertLook` → `AlertTone`（K15）と pop `ButtonTone`（名前のまま、K20）の 2 つだけ。他の改名（`ChatBubbleVariant`、`SpinnerVariant`、`InputGroupVariant`、`TableCellVariant`、`TabsVariant`、`BannerVariant`）は衝突しない（W9 の時点で V2 の `rg` で確かめ、annex U が同じ名前を足していれば `<Component>Tone` にする）。game の `ButtonTone` は KIT-04 で消える（`Tone` の名前の残りではない）。

### 5.3 同じ wave の unit が同じ file を持たないか

- W2a：lane K だけ（K01〜K05 は順に実行）。他の lane の unit は kit と devtools を持たない（W2 の他の lane は R・C・S・U・A。annex T の W1 の印の削除は W1 で終わる）。
- W9：K6 の lane は `crates/kivact-style/src/{icon_shared,icons_macro,lib,facade,icon}.rs`、`crates/kivact-ui/src/`（sidebar・tabs・variants・syntax・lib・theme）、`crates/kivact-core/src/`（view・task・ViewState）、`crates/kivact-plot/`、`examples/catalog-shell/`、`crates/kivact-ui-basic/src/{plot.rs,plot/,nice.rs,sidebar.rs,sidebar/}`、`dev/xtask/policy/workspace.toml`、`dev/tests/kivact-public-api/tests/{e2e_core,e2e_ui}/` の 4 つの file と `main.rs`。K1〜K5 の lane はそれぞれの kit の directory だけ（K1 は上の 5 つの path を除く）。root `Cargo.toml` の `members` の行は 2.2 節の規則（自分の行だけ、conflict は両方残す）。
- W10：K1〜K5 はそれぞれの kit と catalog の directory。K6 の lane は W10 に unit を持たない。
- W11a：K44 は `dev/tests/kivact-kit-tests/` と、移す試験の 6 つの kit の file だけ（SCHEDULE の W11a の K の行）。

### 5.4 状態の規則（設計 6.1）

新しく作る状態の型：`MobileContext`（K02、印の 1 行）。他の新しい型（`Layout`、`Seen`、`SidebarResize`、`TabOverflow`、`CatalogTheme`、`Page`、`Example`、newtype の variant）は enum・値・constructor だけで書く型で、検査の対象外。kit の builder は `#[component]` の引数になり field を持たない（C10）。
