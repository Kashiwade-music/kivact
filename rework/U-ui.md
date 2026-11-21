# 付属書 U：kivact-interaction と kivact-ui

## 1. 範囲と前提

この付属書は `crates/kivact-interaction` と `crates/kivact-ui` の改修を、実装者がそのまま実行できる unit に分けて書く。

扱う所見：
- `rework/audit/interaction-ui.md` の UI-01〜UI-25（すべて採用。UI-03・UI-14 は設計書 5.7・5.9 節のとおり、部品の Tokens を theme 変数にし、recipe を部品ごとの `<Name>Theme`（`ThemeExtension`）にする。上書きの形は `Fn(&Theme, &Look<S>, Style) -> Style`）。
- `rework/audit/api.md` の API-13（`Element` の method を隠す固有 method）、API-14（`#[track_caller]`）、API-17（設計書 DEC-05：入力部品の見えない名前は `.label()` と `field()`）、API-19（`kivact::ui::theme` module、`use_toaster`・`use_announcer`）。
- 設計書 5.1（ui の部品はすべて `#[component]` の関数で `impl Element` を返す）、5.3（handler の名前）、5.9、6.2 PERF-11、6.3、7.5。

### 1.1 wave と lane の置き方（この付属書の決定）

| wave | lane | 内容 | 理由 |
|---|---|---|---|
| W2a・W3a・W4 | U（一つの lane、順に） | 公開 API を変えない内部の改修：UI-23（doctest）、UI-02（索引と selector）、UI-05〜UI-10 の内部、UI-07・UI-16・UI-18・UI-21 の性能部分、UI-22、UI-25 の toaster、**kivact-ui の部品ごとの directory への物理的な移動（U11〜U14）** | 移動を先に済ませると、W7・W8 の lane がファイル単位で排他になる。移動は公開の path を変えないので W4 に置ける |
| W7a | U1 だけ（他の ui・interaction の lane は無い） | 共有の土台の作り直し：`recipe_table!` と `<Name>Theme`（UiRecipes の削除）、UiTokens の theme 変数化、DEC-05 の `.label()`、呼出しの形を変える改名・削除（UI-13、UI-19、UI-20 の ui 側、API-13、API-14、`selection_field`、`icon_button`、`use_toaster`）、`Element::variant` の採用（68 個の写しの削除） | これらは ui の全ファイルと `recipes.rs`・`tokens.rs`・`lib.rs` を同時に変える。並べると必ず衝突するので、一つの lane で順に行う（annex T の W1a/W1b と同じ考え） |
| W7b | U1〜U4 | interaction 側の破壊的な改修と、その呼出し元の ui：U1 = 開閉の語彙と `surface()`（U30a：disclosure・popup・dialog・tooltip、U30c：hover_card・navigation_menu、U30b：menu・select・combobox・palette。UI-12、UI-24、UI-05 の公開部分、UI-11 の disclosure・tooltip）、UI-01、`use_sheet_tap`。U2 = gesture（`use_switch_drag`、`use_swipe_dismiss`、Announcer・Toast の view の除去）。U3 = 編集の machine（`use_date_field`、`use_date_picker`、`use_draft`、`use_history`、月の swipe）。U4 = UI-17 の key | interaction の module の利用者でまとめると、ファイルが lane の間で重ならない（2.2 節の表） |
| W8 | U1〜U4 | 部品ごとの `#[component]` への書き換え（ui だけ）。lane は部品の群：U1 = 操作部品と入力、U2 = 重ねて開く部品、U3 = collection とナビゲーション、U4 = 表示と分野の部品（日付・chat・code を含む） | W7a で呼出しの形が確定しているので、W8 の書き換えは他の lane の呼出し元を壊さない（2.3 節の規則） |

`lib.rs`（kivact-ui と kivact-interaction）と `crates/kivact-ui/src/theme.rs` の `mod` 行・`pub use` 行は、同じ wave の複数の unit が足す。merge の規則は「**両方の行を残す**」。それ以外のファイルは、同じ wave の二つの unit が持たない。

### 1.2 前提にする他の付属書の成果（名前は 4 章に正確に書く）

- annex T（W1）：e2e の binary の分割（`kivact-public-api-tests` の `e2e_core`・`e2e_ui`・`e2e_native`、新しい package `kivact-kit-tests` の `e2e_kits`）、`xtask-allow(state-fields)` の印の削除、新しい `state` 検査。
- annex S（W1a）：`kivact_types::cell::{Deferred, SharedDeferred}`。W1 の時点で kivact-ui の `OnceLock` は S/T がこの型に置き換えている。
- annex R・C（W5 の鎖）：`Signal<T>`（`Binding`・`IntoBinding` の削除）、`Trigger`、`IntoHandler`・`Callback<E>`、`use_context -> T`、`#[component]` の新しい macro（4.2 節の要求を満たすもの）、`Element::variant` と core の `Variant`・`Variants`・`Look<S>`（4.2 節）。
- annex S（W6 の鎖）：`ThemeVar<T>`、`Theme::get/set`、`Color::var`、`Length::var`、中核の `color::*`・`space::*`・`radius::*`・`font::*`、CSS の setter の名前。W6 の鎖は ui の `look_*` の中の `ColorToken`・`t.space().md` などを theme 変数に書き換え済みである。
- annex E（W5a の鎖）：E17 の `EditState::goal_x()`・`EditState::move_vertically(to, extend, goal_x)` と、E18 の後の E20 の `EditState::word_at(at)`・`EditState::line_at(at, multiline)`。`set_goal_x` は無い。U36 が使う。

### 1.3 後の付属書に渡すもの

- annex K（W9〜W10）：4.4 節の対応表だけで 5 つの kit を移行できる。kit が必要とする中核の追加（sidebar、tabs の overflow、`Slant`・`Striped`、`SyntaxKind`）は**この付属書では実装しない**。annex K が W9 の K6 で `kivact-ui` に足す。
- annex G（W11）：`crates/kivact-ui/src/lib.rs` の doc の表は U23 で実物に合わせて書き直す。利用者向けの docs の書き換えは G。

### 1.4 読み方の注意（全 unit 共通）

- **clippy の `mod_module_files` は deny**（workspace の lints）。そのため directory を持つ module の root は `src/<name>.rs` で、`src/<name>/mod.rs` は作らない。W2a・W3a・W4 の unit（U01〜U10）は移動の前なので、今の file の path で書いてある。
- 行番号は使わない。場所は関数名・型名と、手順に引用した特徴のある行で探す。
- W5・W6 の後のコードを前提にする unit（W7a 以降）は、その時点の型名で書く。引用した現在の行が W5・W6 で形を変えていたら、同じ役割の行を探す。
- W6〜W10 の間、5 つの kit（`crates/kivact-ui-*`）、その catalog、`crates/kivact-ui-*/tests`、`e2e_kits` は compile しない。この付属書の W7・W8 の unit はそれらを直さない。
- handler の受け方は設計書 5.3（DEC-16）：payload の無い handler（`on_press`、`Callback<()>`）は `impl IntoHandler<(), M>`、payload のある handler は `impl Fn(E) -> R + 'static`。共有した `Callback<E>` は `move |e| cb.call(e)` で渡す。`Signal::update`・`Var::update` は `()` を返す（DEC-15）。W2a・W3a・W4 の unit は W5 の前に走るので、HEAD の名前（`Binding`、`IntoBinding`、`Notifier`）で書いてある。
- 試験は annex T の T15 の後の置き場で書く：`dev/tests/kivact-public-api/tests/e2e_ui/`（`apg_*`・`interaction_*`・`ui_*`・`feedback`・`indicators`・`metrics_and_description`・`reactive_props`）、`dev/tests/kivact-public-api/tests/e2e_core/`（`component_macro`・`components`・`async_tests`・`environment`・`i18n`・`text_editing` ほか）、kit を使う試験は package `kivact-kit-tests` の `e2e_kits`（`theme_conformance`・`ui_standard`・`apg_grid_kit`・`presence_kit`。W6〜W10 は compile しない）。E2E の命令は `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test <binary> -- <module>` で、一つの命令は一つの binary の module だけを filter に持つ。filter が 0 件の試験を走らせたら失敗（完了条件を満たさない）。命令は worktree の root で走らせる（固定の `cd` は書かない。要るときは `ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1`）。

## 2. unit の一覧

### 2.1 unit の表

規模：S ≈ 150 行まで、M ≈ 400 行まで、L ≈ 600 行まで（手で書く行。機械的な置換は含めない）。

| ID | wave | lane | 前提の unit | 所有するファイル・directory（詳細は各 unit） | 所見 | 規模 |
|---|---|---|---|---|---|---|
| U01 | W2a | U | T の W1 | `crates/kivact-interaction/Cargo.toml`、doctest を持つ 21 file の doc comment | UI-23 | S |
| U02 | W2a | U | U01 | interaction `collection.rs`（新）、`list/`、`select/handle.rs`、`tabs.rs` ほか | UI-02、PERF-11 | L |
| U03 | W2a | U | U02 | interaction `combobox/handle.rs`、`menu/level.rs`、`palette/handle.rs` | UI-02 | M |
| U04 | W3a | U | U03 | interaction `table/state.rs`、`table/cells.rs`、`tree.rs` | UI-02 | M |
| U05 | W3a | U | U04 | interaction `combobox/{machine,handle}.rs`、`palette/handle.rs` | UI-05（内部） | M |
| U06 | W3a | U | U05 | interaction `popup/{machine,api}.rs`、`dialog.rs` | UI-06 | M |
| U07 | W3a | U | U06 | interaction `list/`、`select/`、`menu/{machine,session,level}.rs` | UI-08、UI-10 | M |
| U08 | W3a | U | U07 | interaction `scalar/`、`delayed/api.rs`、`toaster/api.rs`、ui の calendar と date の view | UI-09、UI-25（toaster） | M |
| U09 | W4 | U | U08 | ui `carousel.rs`、`async_view.rs`、`button.rs`、`sparkline*.rs`、`combobox/`、interaction `menu/level.rs` | UI-07、UI-16、UI-18、UI-21（性能） | M |
| U10 | W4 | U | U09 | interaction `drag/`、`combobox/handle.rs`、`tooltip/api.rs` | UI-22 | M |
| U11 | W4 | U | U10 | ui の操作部品と入力（移動） | UI-14（配置） | L（機械的） |
| U12 | W4 | U | U11 | ui の重ねて開く部品（移動） | UI-14（配置） | L（機械的） |
| U13 | W4 | U | U12 | ui の collection とナビゲーション（移動） | UI-14（配置） | L（機械的） |
| U14 | W4 | U | U13 | ui の表示と分野の部品（移動）、共有の file の削除 | UI-14（配置） | L（機械的） |
| U20 | W7a | U1 | W6 | ui 全体（`recipe.rs`、`layer.rs`、`theme.rs`（新）、`recipes.rs` の削除）、試験 3 file | UI-03、UI-14、UI-24 (d)、API-19 | L |
| U21 | W7a | U1 | U20 | ui 全体（`tokens.rs` の削除、各 `look.rs` の変数） | UI-03、DEC-06 | L |
| U25 | W7a | U1 | U21 | `kivact-macros`、`kivact-core` の view・custom、`component_macro.rs` | （W8 の前提：4.2 節 R2〜R5） | M |
| U22 | W7a | U1 | U25 | ui 全体と呼出し元（facade、devtools、試験、例、雛形、bench） | DEC-05、API-17、UI-13 (a) | M（＋機械的） |
| U23 | W7a | U1 | U22 | ui 全体、interaction の toaster・announcer・combobox・table の一部、呼出し元 | UI-13、UI-15、UI-19、UI-20、API-13、API-14、API-19、USR-17 | L |
| U24 | W7a | U1 | U23 | ui 全体 | UI-14、5.4 節 | M（＋機械的） |
| U30a | W7b | U1 | W7a | interaction の disclosure・popup・dialog・inline_edit・tooltip、ui の対応する部品 | UI-12、UI-24、UI-11（disclosure・tooltip） | L |
| U30c | W7b | U1 | U30a | interaction の hover_card・navigation_menu、ui の hover_card・navigation_menu | UI-12、UI-13（`on_open_change`）、UI-24 | M |
| U30b | W7b | U1 | U30c | interaction の menu・select・combobox・palette、ui の対応する部品 | UI-12、UI-24、UI-05（公開）、UI-04 規則 2 | L |
| U31 | W7b | U1 | U30b | interaction の part の slot と `*_state()`（2.2 節）、ui `text_input.rs` の 1 か所 | UI-01 | M |
| U33 | W7b | U1 | U31 | interaction `sheet_tap.rs`（新）、ui `select`、`combobox/render.rs` | UI-04 | M |
| U32 | W7b | U2 | W7a | interaction `switch_drag.rs`・`swipe_dismiss.rs`（新）、`announcer.rs`、`toaster/data.rs`、ui の toggle・toast・dialog の swipe・empty_state・`combobox/notices.rs` | UI-04 | L |
| U34 | W7b | U3 | W7a | interaction `date_field.rs`・`draft.rs`（新）、ui の date の 4 部品・pagination・number_input の draft | UI-04 | L |
| U35 | W7b | U3 | U34 | interaction `history.rs`・`month_swipe.rs`（新）、ui の chat_composer・calendar・`date_parts/sheet.rs` | UI-04 | M |
| U36 | W7b | U4 | W7a | interaction `text_edit/policy.rs`、試験 `text_editing.rs` | UI-17 | M |
| U42 | W8 | U1 | W7b | interaction `scalar/`・`range_scalar.rs`・`splitter/`、ui の slider・splitter・number_input | UI-11（scalar） | M |
| U40 | W8 | U1 | U42 | ui `button.rs`、`button/look.rs`、`button_group.rs` の `member_style` | 5.1、5.9（icon_button） | M |
| U44 | W8 | U1 | U40 | 2.3 節の表の U44 の行 | 5.1、DEC-03 | L |
| U45 | W8 | U1 | U44 | 同 U45 | 5.1 | L |
| U46 | W8 | U1 | U45 | 同 U46 | 5.1、5.10（`AsyncViewTheme`） | M |
| U41 | W8 | U2 | W7b | interaction の policy の 3 file、`hover_card.rs`、`menu/press_state.rs`、`switch_drag.rs` の 1 行 | UI-04 規則 4、UI-25 | S |
| U47 | W8 | U2 | U41 | 同 U47 | 5.1、5.10（`AnnouncerOutletTheme`） | L |
| U48 | W8 | U2 | U47 | 同 U48 | 5.1 | L |
| U43 | W8 | U3 | W7b | interaction `reorder_drag.rs`（新）、ui `data_table/{filters,rows,row_drag}.rs` | UI-04 | M |
| U49 | W8 | U3 | U43 | 同 U49 | 5.1 | L |
| U50 | W8 | U3 | U49 | 同 U50 | 5.1 | L |
| U51 | W8 | U3 | U50 | 同 U51 | 5.1 | L |
| U52 | W8 | U4 | W7b | 同 U52 | 5.1、UI-21（recipe の状態） | L |
| U53 | W8 | U4 | U52 | 同 U53 | 5.1 | M |
| U54 | W8 | U4 | U53 | 同 U54 | 5.1 | L |

### 2.2 W7b の lane のファイルの排他

W7b の 4 lane は次のように分ける。各 unit の所有の欄はこの表と一致している。

| 対象 | 持つ unit | 他の unit |
|---|---|---|
| ui `dialog.rs`・`dialog/look.rs` | U30a | U32 は `dialog/drawer_swipe.rs` だけを持つ（`dialog.rs` が呼ぶ関数の署名を変えない） |
| interaction `hover_card.rs`・`navigation_menu.rs`、ui `hover_card.rs`・`navigation_menu.rs` と directory | U30c | — |
| ui `data_table/rows.rs` | U30a（変更なし） | data table の filter と row drag は W8 の U43 |
| ui の `DialogMode` → `DialogKind` | W7a の U23（済み） | U30a は `dialog.rs` の中の対応だけを直す |
| ui `combobox/notices.rs` | U32 | U30b・U31・U33 は変えない |
| interaction `scalar/`、`range_scalar.rs`、`splitter/` | U31（getter の削除だけ） | scalar の disabled（UI-11）は W8 の U42 |
| ui `number_input.rs` | U34（draft の配線） | `.disabled` の 1 行は W8 の U42 |
| ui `text_input.rs` | U31（`control_state` の 1 か所） | — |
| 試験 `interaction_toggle.rs` | U30a・U31（同じ lane の順） | U32 の switch の試験は `ui_controls.rs` に置く |
| 試験 `interaction_show_hide.rs`、`ui_disclosure_surfaces.rs` | U30a・U30c・U30b（同じ lane の順） | — |
| interaction `text_edit/policy.rs`、試験 `text_editing.rs` | U36 | — |
| `crates/kivact-ui/src/lib.rs`、`crates/kivact-ui/src/theme.rs`、`crates/kivact-interaction/src/lib.rs` | 複数（`mod`・`pub use` の行だけ） | merge の規則は「両方の行を残す」 |

### 2.3 部品の表（W8 の unit が使う。annex K の「部品の module 表」もこの表）

列の意味：
- **module**：W4 の後の file（`crates/kivact-ui/src/<module>.rs` と `crates/kivact-ui/src/<module>/`）。
- **`<Name>Theme` の part（状態）**：U20 の後の表。変数は 4.3 節。
- **設定**：U21 で表の `settings { .. }` に入るもの。
- **W7a・W7b の呼出しの変更**：W8 より前に済んでいる変更（W8 は変えない）。
- **特別な扱い**：W8 の unit がその部品だけに行うこと。「子：遅延」は子の props を `Children`（U25 R4）にし、context の中で `children.into_views()` を呼ぶ。slot の props は `Option<Rc<dyn Fn(..) -> View>>`（U25 R2）。

| 部品（関数） | W8 unit | module | `<Name>Theme` の part（状態） | 設定 | W7a・W7b の呼出しの変更 | 開閉 | 特別な扱い | 試験（`--test e2e_ui -- ..`） |
|---|---|---|---|---|---|---|---|---|
| `button` | U40 | `button` | `ButtonTheme`：root, content, busy, icon（`ButtonState`） | — | `child`→`children`、`default_size` 削除（U23） | — | U40 のとおり | `ui_controls ui_theming ui_toolbar` |
| `icon_button` | U40 | `button` | `ButtonTheme` を使う | — | 新設（U23。`tooltip(label, button)` の形） | — | U40 のとおり | `ui_controls ui_theming` |
| `checkbox` | U44 | `toggle` | `CheckboxTheme`：root, indicator, check, mixed（`CheckableState`） | — | — | — | `indeterminate: Option<Value<bool>>`。indicator の part は `has_parts()` の間だけ置く（今の `toggle/parts.rs`） | `ui_controls interaction_toggle ui_theming` |
| `switch` | U44 | `toggle` | `SwitchTheme`：root, track, thumb（`CheckableState`） | — | drag は `use_switch_drag`（U32） | — | 同上 | 同上 |
| `toggle_button` | U44 | `toggle_button` | `ToggleButtonTheme`：root（`ToggleButtonState`） | `label_reservation: ToggleLabelReservation` | — | — | — | `ui_controls` |
| `toggle_group` | U44 | `toggle_group` | `ToggleGroupTheme`：root（`ToggleGroupState`）、item（`ToggleGroupItemState`） | — | — | — | 型引数 `K: ItemKey`。`item: Option<Rc<dyn Fn(Value<ListItem<K>>, Value<ToggleButtonState>) -> View>>`。`on_edit: Option<Callback<EditPhase>>` | `ui_toolbar apg_toolbar` |
| `button_group`、`button_group_separator`、`button_group_text` | U44 | `button_group` | `ButtonGroupTheme`：root, separator, text（`ButtonGroupState`）、member（`ButtonGroupMemberState`） | `joined: bool = true` | `ButtonGroupMember` 削除（U23） | — | 子：遅延（group の向き・大きさの context） | `ui_toolbar` |
| `toolbar` | U44 | `toolbar` | `ToolbarTheme`：root（`ToolbarState`） | — | — | — | `start`・`center`・`end` は `Option<Children>`（control size の context） | `ui_toolbar apg_toolbar` |
| `text_input`、`text_area` | U45 | `text_input` | `TextInputTheme`：root、`TextAreaTheme`：root（`TextInputState`） | — | `.label()`（U22）、`on_commit`（U23）、`control_state` 削除（U31） | — | 名前の欠落の診断は `caller().accessible_name()` と `accessible_labelled_by()` で読む。`text_area` は同じ module の別の `#[component]` | `ui_field text_editing interaction_edit` |
| `number_input` | U45 | `number_input` | `NumberInputTheme`：root, input（`PartState`） | — | `.label()`（U22）、draft は `use_draft`（U34）、disabled は node（U42） | — | — | `apg_spinbutton ui_controls` |
| `tag_input` | U45 | `tag_input` | `TagInputTheme`：root, tag, remove, input（`PartState`） | — | `tag_input(tags: Signal<Vec<String>>)`・`.label()`（U22）、`on_add`・`on_remove` 削除（U23） | — | — | `ui_controls` |
| `field` | U45 | `field` | `FieldTheme`：root, label, hint, error（`FieldState`） | — | — | — | `#[component] pub fn field<C: Element + IntoView>(label: Value<String>, control: C, ..)`：`control` は型引数そのものの props（U25 の Param。`C` をそのまま受け、`Into` で包まない。12 章 API-1）。本体が `control` に `labelled_by`・`described_by`・`invalid` を付けてから view にする。試験を足す（`ui_field.rs`）：`field_takes_a_standard_or_custom_control_without_annotations`：`field("Name", text_input(name))` と、同じ file の `#[component] fn custom_input() -> impl Element { node().role(Role::TextBox).focusable(true) }` を `field("Code", custom_input())` で、型注釈も turbofish も無しに mount し、`get_by_role(Role::TextBox, "Name")` と `get_by_role(Role::TextBox, "Code")` がある | `ui_field` |
| `input_group`、`input_addon` | U45 | `input_group` | `InputGroupTheme`：root, label, members, addon, block_start, block_end（`InputGroupState`）、member（`InputGroupMemberState`） | — | `input_group_with` 削除（U23） | — | 子：遅延（input group の context） | `ui_field ui_components` |
| `slider`、`range_slider` | U46 | `slider` | `SliderTheme`：track, range, thumb（`AxisState`） | — | `.label()`（U22）、disabled は node（U42） | — | `range_slider` は同じ module の別の `#[component]` | `apg_slider interaction_slider` |
| `splitter` | U46 | `splitter` | `SplitterTheme`：handle（`AxisState`） | — | disabled は node（U42） | — | — | `apg_window_splitter interaction_splitter` |
| `split` | U46 | `split` | `SplitTheme`：grip（`AxisState`） | — | — | — | — | `interaction_splitter` |
| `dropzone` | U46 | `dropzone` | `DropzoneTheme`：root, icon, label, hint, status（`DropzoneState`） | — | — | — | `validate: Option<Rc<dyn Fn(&FileInfo) -> Option<String>>>`、`on_files: Option<Callback<Vec<FileInfo>>>` | `ui_dropzone` |
| `async_view` | U46 | `async_view` | **新** `AsyncViewTheme`：root（`()`。既定 `style().flex_direction(FlexDirection::Column)`）を `async_view/look.rs` に作る（5.10 節） | — | — | — | `#[component] pub fn async_view<T: Clone + 'static, S: AsyncSource<T>>(source: S, ready: Rc<dyn Fn(T) -> View>, loading: Option<ViewFn>, fallback: Option<Rc<dyn Fn(UiError, Callback<()>) -> View>>) -> impl Element`（12 章 API-1：`source` は型引数そのものの props。`T` は `S: AsyncSource<T>` の bound と `ready` が名指す。`loading` は C の U-REQ-6 のとおり `IntoViewFn`）。試験を足す（`async_tests.rs`、e2e_core）：`async_view_takes_a_resource_or_an_action_and_an_unannotated_ready_closure`：`let data = resource(|| async { Ok::<u32, UiError>(7) });` と `async_view(data, |n| text(n.to_string()))`、同じく `action` の結果を source にした `async_view(save, |n| text(format!("saved {n}")))` を型注釈無しで mount し、`run_until_idle` の後に「7」が見える | `async_tests`（e2e_core） |
| `dialog` | U47 | `dialog` | `DialogTheme`：surface, title, description, header, media, body, footer, scrim, close, drag_handle（`DialogState`） | `center: DialogCenter = DialogCenter::Anchor` | `mode(DialogKind)`、`description` は `Element::description`（U23）、drawer は `use_swipe_dismiss`（U32） | `dialog(title, open: Signal<bool>)`（位置） | 見える説明は `caller().accessible_description()` | `interaction_dialog ui_disclosure_surfaces` |
| `tooltip` | U47 | `tooltip` | `TooltipTheme`：surface（`PopupState`）、arrow（`TooltipArrowState`） | — | `tooltip(label, trigger)`・`Tooltip::attach` の削除（U23。12 章 API-3）、`use_tooltip(enabled)`（U30a） | 開閉は hover と focus（setter なし） | `#[component]` にしない（親の決定。設計 5.1 の手書きの builder の例外）：U23 の `Tooltip<T>` のまま、`node_mut` は trigger の root の node、`into_view` は trigger（tooltip の trigger の part 付き）と `show(open, surface)` の兄弟の二つの fragment。U47 はこの部品の recipe の variants を `into_view` の先頭で trigger の node の `authored().variants()` から読む形になっていることを確かめるだけ（U24 が `variants` field を消している） | `interaction_tooltip apg_tooltip ui_components` |
| `hover_card` | U47 | `hover_card` | `HoverCardTheme`：surface（`HoverCardSurfaceState`） | — | `on_open_change` 削除（U30c） | `.open(signal)`（U30c） | `content: ViewFn`（位置引数） | `ui_disclosure_surfaces` |
| `disclosure`、`disclosure_group` | U47 | `disclosure` | `DisclosureTheme`：root, trigger, indicator, content（`DisclosureRecipeState`） | `motion: DisclosureMotion` | U30a の形 | U30a の形（`open` の受け方は U30a のとおり） | `disclosure_group` は型引数 `K`、`panel` slot は `Rc<dyn Fn(..) -> View>` | `apg_accordion ui_disclosure_surfaces interaction_show_hide` |
| `announcer_outlet` | U47 | `announcer_outlet` | **新** `AnnouncerOutletTheme`：root（`()`。既定は今の outlet の node の style）を作る | — | 新設（U32） | — | — | `interaction_toast` |
| `menu` | U48 | `menu` | `MenuTheme`：trigger, surface, sub_surface, context_surface（`PopupState`）、item, shortcut, description, check, submenu（`MenuItemState`）、separator, label, inset（`PartState`） | — | U30b の形 | `.open(signal)`（U30b） | 型引数 `K`。`item` slot、`on_activate: Option<Callback<K>>` | `interaction_menu apg_menu apg_menu_button apg_menubar` |
| `select` | U48 | `select` | `SelectTheme`：trigger（`SelectTriggerState`）、surface, indicator（`PopupState`）、option, check, description（`ListItemState`） | — | `.label()`（U22）、`.trigger(view)`（U23）、sheet tap は `use_sheet_tap`（U33） | `.open(signal)`（U30b） | 型引数 `K`。`item` slot | `interaction_select apg_listbox ui_combobox` |
| `combobox` | U48 | `combobox` | `ComboboxTheme`：frame, input, surface, chip, remove, empty（`PartState`）、option（`ListItemState`） | — | `.label()`（U22）、`ComboboxMode` は ui（U23）、U30b の形 | `.open(signal)`（U30b） | 型引数 `K`。`filter: Option<Rc<dyn Fn(&str, &str) -> bool>>` | `ui_combobox apg_combobox interaction_combobox` |
| `command_palette` | U48 | `command_palette` | `CommandPaletteTheme`：scrim, search, search_icon, field, list, group, empty, footer（`PartState`）、panel（`PopupState`）、row, icon, hint（`ListItemState`） | — | U30b の形 | `command_palette(commands, open: Signal<bool>)`（位置。palette は dialog と同じく閉じていると意味が無い） | 型引数 `K`。`filter: Option<Rc<dyn Fn(&str, &Command<K>) -> bool>>` | `ui_command_palette interaction_palette` |
| `navigation_menu`、`navigation_menu_link` | U48 | `navigation_menu` | `NavigationMenuTheme`：root, action, panel（`NavOpen`）、trigger（`NavigationTriggerState`）、link, link_title, link_description, link_icon（`NavigationLinkState`） | — | `on_open_change` 削除（U30c）、link の `on_press`・`description` は Element（U23） | `.open(signal)`（U30c） | 子：遅延（navigation の context） | `ui_navigation` |
| `toast_region` | U48 | `toast` | `ToastTheme`：region, row（`PartState`）、group（`ToastGroupState`）、toast, title, body, icon, close（`ToastState`） | `row_transition: Option<Transition> = None` | swipe は `use_swipe_dismiss`、icon は `icon_name`（U32） | — | — | `interaction_toast` |
| `list` | U49 | `list` | `ListTheme`：root（`PartState`）、item, description（`ListItemState`） | — | — | — | 型引数 `K`。`item` slot、`on_edit: Option<Callback<EditPhase>>` | `ui_collections interaction_list_nav apg_listbox` |
| `tree`、`tree_rows` | U49 | `tree` | `TreeTheme`：root（`PartState`）、item, marker（`TreeRowRecipeState`） | — | — | — | 型引数 `K`（`tree_rows` は `K`・`P`。どちらも props が名指す）。`presentation: Rc<dyn Fn(&TreeRow<K>) -> P>`、`render: Rc<dyn Fn(K, Value<TreeRow<K>>, Part<TreeItemState>) -> Node>` | `apg_tree_view interaction_tree` |
| `tabs` | U49 | `tabs` | `TabsTheme`：list（`TabListState`）、tab, indicator, panel（`TabState`） | — | — | — | 型引数 `K`。`item` slot、`content: Option<Rc<dyn Fn(K) -> View>>` | `apg_tabs interaction_tabs` |
| `radio_group` | U49 | `radio_group` | `RadioGroupTheme`：group（`Orientation`）、item, indicator, mark, description（`CheckableState`） | — | — | — | 型引数 `K`。`item` slot、`item_node: Option<Rc<dyn Fn(Node, Value<ListItem<K>>, Value<CheckableState>) -> Node>>` | `apg_radio_group` |
| `data_table` | U50 | `data_table` | `DataTableTheme`：frame, toolbar, footer（`PartState`）、header, sort（`TableHeaderState`）、row（`DataTableRowState`）、group_header（`DataTableGroupState`）、cell（`DataTableCellState`）、check（`DataTableCheckState`）、page（`DataTablePageState`） | `sticky_shadow_transition: Option<Transition> = None`、`sticky_overlay_transition: Option<Transition> = None`、`row_status_header_key: &'static str = "data_table.row_status.column_header"` | `DataColumn` は `data_table/column.rs`、`ColumnDef::width`（U23）、filter と row drag は interaction（U43） | — | 型引数 `R`・`K`（`rows`・`key` が名指す）。`key: Rc<dyn Fn(&R) -> K>`（位置）、`row_selectable`・`row_label`・`row_color` は `Option<Rc<dyn Fn(&R) -> ..>>`、`row_details: Option<Rc<dyn Fn(Value<R>) -> View>>`、`cell_controls: Option<Rc<dyn Fn(Value<R>, usize) -> View>>`、`empty: Option<ViewFn>` | `ui_data_table interaction_table` |
| `pagination` | U51 | `pagination` | `PaginationTheme`：root, ellipsis, dots（`()`）、item（`PaginationItemState`）、controls, sizes, input, summary（`PaginationSummary`） | — | draft は `use_draft`（U34） | — | `item: Option<Rc<dyn Fn(PageItem) -> View>>`、`label_format: Option<Rc<dyn Fn(usize) -> String>>` | `ui_navigation` |
| `breadcrumb` | U51 | `breadcrumb` | `BreadcrumbTheme`：root, list, separator（`()`）、item（`BreadcrumbItemState`） | — | — | — | 型引数 `K`。`item: Option<Rc<dyn Fn(ListItem<K>) -> View>>`、`separator: Option<ViewFn>`、`on_activate: Option<Callback<K>>` | `apg_breadcrumb ui_navigation` |
| `stepper` | U51 | `stepper` | `StepperTheme`：root, previous, next（`StepperState`）、item, layout, track, leading, trailing, progress, body, indicator, label, description, content, end_content（`StepperItemState`） | — | `stepper(items, current: Signal<usize>)`、`on_step_press` 削除、`on_edit` と EditSession（12 章 UI-U-03）、`disabled` は Element（U23） | — | `StepperItem` は値の型のまま（builder ではない）。`on_edit: Option<Callback<EditPhase>>`。利用者の書込みはすべて U23 の `write` を通す | `ui_navigation interaction_edit` |
| `carousel` | U51 | `carousel` | `CarouselTheme`：root, viewport, track, slide, previous, next（`CarouselSurfaceState`） | — | `Snap` は interaction の path（U23） | — | `slides: Vec<View>`。carousel の machine 化（UI-07 の残り）は 3.4 節の「U51 の carousel」の手順 | `ui_carousel apg_carousel` |
| `item_list`、`item_row` | U51 | `item_list` | `ItemListTheme`：list_header, root（`PartState`）、marker, item, body, content, title, description, media, actions, header, footer（`ItemRowState`） | `title_lines: Option<u32> = None`、`description_lines: Option<u32> = None` | `item_row` の `on_press`・`description` は Element（U23） | — | `item_list` は子：遅延（item の context） | `ui_collections` |
| `table`、`table_header`、`table_body`、`table_footer`、`table_row`、`table_head`、`table_cell` | U51 | `table` | `TableTheme`：root, header, body, footer, row, head, cell（`PartState`） | — | — | — | 7 つの `#[component]`。`table_header`・`table_body`・`table_footer` は今 `Table` を返す：それぞれ別の生成された struct（`TableHeader`・`TableBody`・`TableFooter`）になる（呼出しの形は同じ） | `ui_collections` |
| `alert` | U52 | `alert` | `AlertTheme`：root, header, message, controls, title, description, icon, actions, content, dismiss, toggle（`AlertState`） | — | `description` は Element（U23） | — | `content: Option<ViewFn>`。見える説明は `caller().accessible_description()` | `apg_alert feedback` |
| `card` | U52 | `card` | `CardTheme`：root, header, media, content, footer, overlay（`CardState`） | — | `on_press`・`disabled` は Element（U23） | — | press は `caller.has_press()`、disabled は `caller.disabled()`（`#[caller] caller: Caller`） | `ui_components` |
| `empty_state` | U52 | `empty_state` | `EmptyStateTheme`：root, title, description, icon, actions（`EmptyStateState`） | — | `description` は Element（U23）、outlet は `announcer_outlet`（U32） | — | — | `feedback` |
| `badge` | U52 | `badge` | `BadgeTheme`：root, label（`BadgeState`） | — | `on_press` は Element（U23） | — | — | `indicators` |
| `kbd`、`kbd_combo` | U52 | `kbd` | `KbdTheme`：root（`()`）、key（`KbdKeyState`） | — | — | — | `kbd_combo` は同じ module の別の `#[component]` | `indicators` |
| `avatar` | U52 | `avatar` | `AvatarTheme`：root, image, status, status_content（`AvatarState`） | `name_tooltip: bool = false` | `on_open` 削除、`on_press` は Element（U23） | — | — | `indicators` |
| `avatar_group` | U52 | `avatar_group` | `AvatarGroupTheme`：root（`()`）、member, overflow（`usize`） | — | — | — | — | `indicators` |
| `spinner` | U52 | `spinner` | `SpinnerTheme`：root（`SpinnerState`） | — | — | — | — | `indicators` |
| `skeleton` | U52 | `skeleton` | `SkeletonTheme`：root（`SkeletonState`） | `transition: Option<Transition> = None` | — | — | — | `indicators` |
| `progress` | U52 | `progress` | `ProgressTheme`：track, fill, label（`ProgressState`） | `motion: ProgressMotion` | — | — | — | `indicators components` |
| `sparkline` | U52 | `sparkline` | `SparklineTheme`：root（**`SparklineState`** に変える。UI-21） | `round: bool = false`、`fill: SparklineFill = SparklineFill::None` | — | — | UI-21 の残り：`SparklineState { summary: Summary, values: Rc<[f64]> }`（`#[derive(Clone, Debug, PartialEq)]`、`sparkline/look.rs`）を作り、既定の recipe が `style().size(..).paint(sparkline_line(look, LINE))` で線を描く。部品は `.draw` を足さない。`sparkline_line` を `theme.rs` で再公開 | `ui_sparkline` |
| `heading` | U52 | `heading` | `HeadingTheme`：root（`HeadingState`） | — | — | — | — | `components` |
| `stat` | U52 | `stat` | `StatTheme`：root, label, value, delta, icon, chart（`StatState`） | — | — | — | — | `components` |
| `description_list`、`description` | U52 | `description_list` | `DescriptionListTheme`：root, item, term, detail（`()`） | — | — | — | `description` は同じ module の別の `#[component]` | `components` |
| `chat_message` | U53 | `chat_message` | `ChatMessageTheme`：root, body, author, time, status, content, avatar, header, footer, metadata, media, icon, actions, reactions（`ChatMessageState`） | — | `on_press` は Element（U23） | — | — | `ui_chat_code` |
| `chat_thread` | U53 | `chat_thread` | `ChatThreadTheme`：root, viewport, content, scroll_button, composer（`ChatThreadState`） | — | — | — | — | `ui_chat_code` |
| `chat_composer` | U53 | `chat_composer` | `ChatComposerTheme`：root, surface, input, send, header, footer, status, drawer, header_actions, header_context, footer_actions, send_actions（`ChatComposerState`） | — | 履歴は `use_history`（U35） | — | `on_submit: Option<Callback<String>>` | `ui_chat_code` |
| `typing_indicator` | U53 | `typing_indicator` | `TypingIndicatorTheme`：root, avatar, dots（`TypingIndicatorState`） | — | — | — | — | `ui_chat_code` |
| `code_block` | U53 | `code_block` | `CodeBlockTheme`：root, header, title, collapse, language, copy, end_content, body, viewport, code, gutter, footer（`CodeBlockState`） | — | U30a の disclosure の形 | — | `on_copy: Option<Callback<String>>`（今の handler の payload の型のまま） | `ui_chat_code` |
| `calendar` | U54 | `calendar` | `CalendarTheme`：root, month, caption, events（`PartState`）、event（`CalendarEventVisual`）、day（`CalendarDayVisual`）、range（`CalendarDayState`） | — | swipe は `use_month_swipe`（U35） | — | `month_title: Option<Rc<dyn Fn(CalendarDate) -> String>>`、`disabled_dates: Option<Rc<dyn Fn(CalendarDate) -> bool>>`、`on_activate`・`on_commit` は `Option<Callback<CalendarDate>>` | `ui_calendar apg_date_picker interaction_calendar` |
| `date_input` | U54 | `date_input` | `DateInputTheme`：root, wheel（`PartState`） | — | `.label()`（U22）、`use_date_field`（U34） | — | `format_with: Option<Rc<dyn Fn(CalendarDate) -> String>>`、`calendar: Option<Rc<dyn Fn(Calendar) -> Calendar>>`（生成された `Calendar` の struct を受けて返す） | `ui_calendar apg_date_picker` |
| `time_input` | U54 | `time_input` | `TimeInputTheme`：root, wheel（`PartState`） | — | `.label()`（U22）、`use_date_field`（U34） | — | — | `ui_calendar` |
| `date_range_input` | U54 | `date_range_input` | `DateRangeInputTheme`：root（`PartState`） | — | `.label()`（U22）、`use_draft`（U34） | — | `date_constraint`（今は呼ぶたびに足す）は `Option<Rc<dyn Fn(CalendarDate) -> bool>>` 一つにする：U23 の後の呼出し元で 2 回以上呼んでいる所は無い（`rg -n "date_constraint\(" crates dev examples` で確かめ、あれば一つの closure にまとめる） | `ui_calendar` |
| `date_time_input` | U54 | `date_time_input` | `DateTimeInputTheme`：root（`PartState`） | — | `.label()`（U22）、`use_date_picker`（U34） | — | — | `ui_calendar` |

表の状態の型の名前は**変えない**（`CheckableState` は checkbox・switch・radio_group が、`AxisState` は slider・splitter・split が共有する）。UI-14 の「部品ごとに一つの状態」は、今すでに部品ごとの型があるものはそのまま、共有の型は共有のまま使う（4.5 節 U-Q5）。

## 3. unit の手順

unit は wave の順に並べる。W7a の実行の順は U20 → U21 → U25 → U22 → U23 → U24（U25 は U22 の前提）。W7b の所有は 2.2 節の表と各 unit の所有の欄が一致している。

### 3.1 W2a・W3a・W4（lane U、公開 API を変えない）

## U01：interaction の doctest を facade（`kivact`）で書く（UI-23 の後半）

- **wave / lane / 前提**：W2a / U / annex T の W1（e2e の分割と state 規則の置換）が済んでいること。U02 より前。この unit は W5 より前に走るので、doctest の中の名前は HEAD の名前（`Binding`、`IntoBinding`、`notifier()` など）のまま書く（W5 の鎖が掃引する）。
- **所有**：`crates/kivact-interaction/Cargo.toml` と、`crates/kivact-interaction/src/` 以下の次の 30 ファイルの doc comment だけ（コードは変えない）。
  - shim を持つ 21 ファイル（括弧内は shim の数。合計 26 個）：`calendar.rs`(1)、`drag/handle.rs`(3)、`edit/handle.rs`(1)、`list/data.rs`(1)、`menu/handle.rs`(1)、`menu/state.rs`(2)、`menubar.rs`(1)、`palette/handle.rs`(2)、`policy/grid.rs`(1)、`policy/scalar.rs`(1)、`policy/scalar_mapping.rs`(1)、`range_scalar.rs`(1)、`scalar/handle.rs`(1)、`select/handle.rs`(2)、`splitter/handle.rs`(1)、`table/handle.rs`(1)、`table/model.rs`(1)、`toaster/api.rs`(1)、`toaster/data.rs`(1)、`tone.rs`(1)、`tree.rs`(1)。
  - shim を持たないが doctest が `kivact_core::`・`kivact_interaction::` を直接 `use` する 9 ファイル：`dialog.rs`、`disclosure.rs`、`delayed/api.rs`、`combobox/handle.rs`、`popup/api.rs`、`space_pan/handle.rs`、`tabs.rs`、`toggle.rs`、`tooltip/api.rs`（`edit/handle.rs` の `EditSession::once` の例も同じ種類だが、ファイルは上の 21 に入っている）。
- **目的**：UI-23 の後半。`kivact` を kivact-interaction の dev-dependency にして、doctest 26 個の `# mod kivact { … }` shim（4 行または 11 行）を消し、interaction の doctest をすべて `use kivact::…` で書く（shim の無い doctest も含む）。`state-fields` の印は付属書 T（W1）の仕事なので、この unit では触らない。
- **手順**：
  1. `crates/kivact-interaction/Cargo.toml`：`[dependencies]` の節と `[lints]` の間に次の節を足す（kivact-ui と同じ形）。`dev/xtask/policy/workspace.toml` の `[visible_to]` に `kivact` の行は無い（誰でも依存できる）ので、policy は変えない。
     ```toml
     [dev-dependencies]
     kivact = { path = "../kivact" }
     ```
  2. shim を消す。`--path-separator //` は Git Bash が `/` を path に展開するのを防ぐためのもの。改行は `\R` で CRLF と LF の両方に合う。shim の最後は必ず `#     pub extern crate kivact_interaction as interaction;` と `# }` の 2 行（26 個すべてで確認済み）。
     ```
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     rg -l --path-separator // "# mod kivact \{" crates/kivact-interaction/src | xargs perl -0pi -e 's{^[ \t]*//[/!] # mod kivact \{\R(?:[ \t]*//[/!] #.*\R)*?[ \t]*//[/!] #[ \t]+pub extern crate kivact_interaction as interaction;\R[ \t]*//[/!] # \}\R}{}mg'
     ```
     （26 個すべてが消え、`# Ok::<(), kivact::interaction::ScalarRangeError>(())` など shim の後の隠し行は残る。）
  3. 手順 2 の後で、doctest の中で内部 crate を直接 `use` している行をすべて facade に直す。`kivact::state`・`kivact::view`・`kivact::style` は `kivact_core` の同じ module の再公開、`kivact::interaction` は `kivact_interaction` そのものなので、path の頭だけを置き換えればよい。
     ```
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     rg -l --path-separator // "^\s*//[/!]\s*use kivact_(core|interaction)::" crates/kivact-interaction/src | xargs perl -pi -e 's{^(\s*//[/!]\s*)use kivact_core::}{$1use kivact::}; s{^(\s*//[/!]\s*)use kivact_interaction::}{$1use kivact::interaction::}'
     perl -pi -e 's{kivact_core::view::node\(\)\.name\(""\)\.part\(kivact_interaction::use_text_edit}{node().name("").part(kivact::interaction::use_text_edit}' crates/kivact-interaction/src/palette/handle.rs
     ```
     この結果、たとえば次のようになる。
     - `use kivact_core::view::Element;`（menubar、menu/handle、palette/handle、select/handle）→ `use kivact::view::Element;`。`use kivact_core::style::Position;` と `use kivact_core::view::LiveRegion;`（toaster/api）→ `kivact::…`。
     - `use kivact_core::{state::signal, view::{Element, column, node, text, show}};`（disclosure、delayed/api）→ `use kivact::{state::signal, view::{…}};`。dialog・combobox/handle・popup/api・tabs・tooltip/api・space_pan/handle・toggle の `use kivact_core::…` も同じ形。
     - `use kivact_interaction::{DialogMode, use_dialog};` → `use kivact::interaction::{DialogMode, use_dialog};`。`use_disclosure`、`Delays`・`use_delayed`、`ListItem`・`use_combobox`・`use_text_edit`、`EditSession`、`use_popup`、`use_space_pan`、`use_tabs`、`use_button`、`use_checkbox`、`use_tooltip` も同じ。
     - palette の doctest の `///             kivact_core::view::node().name("").part(kivact_interaction::use_text_edit(p.query()).control()).part(p.input()),` → `///             node().name("").part(kivact::interaction::use_text_edit(p.query()).control()).part(p.input()),`（`node` は `kivact::prelude` にある）。
     - intra-doc link（`[`show`](kivact_core::view::show)` など、`](kivact_core::` の形）は rustdoc の link なので変えない。
  4. 名前は確認済みなので、doctest の本文はほかに変えない。shim の prelude の名前はすべて実際の `kivact::prelude` にある（`Binding`、`IntoBinding`、`EditPhase`、`Orientation`、`Role`、`ColorToken`、`style`、`for_each`、`node_ref`、`show`、`memo`、`signal`）。`Cursor` は prelude に無いが、使っている splitter の doctest が `use kivact::style::Cursor;` を自分で書いている。prelude の `tree`、`calendar`、`slider`、`toaster`、`row` などと同じ名前の local の `let` は glob import を隠すだけで、衝突しない。
- **消すもの**：26 個の `# mod kivact { … }` shim（`//! Role: Supply facade paths …` と `//! Role: Supply the canonical imports …` の行を含む）。doctest の中の `use kivact_core::…`・`use kivact_interaction::…` の行（`use kivact::…` に置き換わる）。
- **状態の閉じ方**：型を足さない。
- **試験**：足す試験も直す試験も無い。doctest がそのまま通ること。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction
  cargo check -p kivact-interaction --all-targets
  cargo clippy -p kivact-interaction --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo test -p kivact-interaction --doc
  cargo test -p kivact-interaction --lib
  ```
- **完了条件**：
  - `rg -n "# mod kivact" crates/kivact-interaction` と `rg -n "pub extern crate kivact_interaction" crates/kivact-interaction` が何も返さない。
  - `rg -n "^\s*//[/!]\s*use kivact_(core|interaction)::" crates/kivact-interaction/src` が何も返さない。
  - `rg -n "^\s*//[/!].*\b(kivact_core|kivact_interaction)::" crates/kivact-interaction/src | rg -v "\]\(kivact_core::"` が何も返さない（残るのは intra-doc link だけ）。
  - 上のコマンドがすべて通る。

## U02：`collection` module（`Index<K>`・`Membership<K>`）と list の仲間（UI-02 の A）

- **wave / lane / 前提**：W2a / U / U01。この unit は W5 より前に走るので、HEAD の名前（`Notifier`・`notifier()`・`Binding`・`IntoBinding`・`into_binding`）で書く。W5 の鎖（付属書 R）が `Notifier`→`Trigger` などを掃引するとき、この unit が作る `collection/membership.rs` もその対象に入る。
- **所有**（すべて `crates/kivact-interaction/` 以下）：
  - 新規：`src/collection.rs`、`src/collection/index.rs`、`src/collection/membership.rs`
  - 変更：`src/lib.rs`（`mod` 行）、`src/list.rs`（`mod order;` を消す）、`src/list/state.rs`、`src/list/parts.rs`、`src/list/input.rs`、`src/list/handle.rs`、`src/tabs.rs`、`src/select/handle.rs`
  - 削除：`src/list/order.rs`
- **目的**：UI-02、PERF-11。list の項目の memo を 6 つから「part の状態と name」の 2 つにする。項目ごとの O(N) の `find`、O(S) の `contains`、項目ごとに同じ値を計算する `set_size` をなくす。list の上に作る select・tabs（radio・menubar・table の行・tree も list の part を経由する）も同時に軽くなる。
- **手順**：
  1. `src/collection.rs` を新しく作る：
     ```rust
     //! Role: Per-collection lookups that item parts read in O(1).
     //!
     //! A collection with many items (list, combobox, menu, select, tabs, tree,
     //! table, palette) builds one [`Index`] per change of its items, one
     //! [`Membership`] over its selection and one `selector` over its active
     //! key. An item reads its own key through them and follows only what
     //! concerns that key (PERF-11).

     mod index;
     mod membership;

     pub(crate) use index::{Index, Movement};
     pub(crate) use membership::Membership;
     ```
     `src/lib.rs`：`mod calendar;` の次の行に `mod collection;` を足す。
  2. `src/collection/index.rs` を新しく作る。`list/order.rs` の `Order<K>` と `Movement` を移して名前を `Index` にし、行ごとに `source`（元の slice の位置）を持たせたもの。全文：
     ```rust
     //! Role: One collection's keys in order: built once per change, read per key in O(1).

     use crate::{ListBoundary, ListItem};
     use std::collections::{HashMap, HashSet};
     use std::hash::Hash;

     /// A move in logical order. Controllers map physical keys to this policy.
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub(crate) enum Movement {
         Next,
         Previous,
         First,
         Last,
     }

     /// The first occurrence of a key in the source of an [`Index`].
     #[derive(Clone, Debug, PartialEq, Eq)]
     struct Row<K> {
         key: K,
         /// Whether the row can take the focus or a choice.
         enabled: bool,
         /// The row's place in the source slice the index was built from.
         source: usize,
     }

     /// A collection's keys in logical order, built in one memo per collection
     /// whenever its items change and read by every item part in O(1). The
     /// first occurrence of a duplicate key wins; callers report
     /// [`duplicates`](Self::duplicates) and never give a key a second identity.
     #[derive(Clone, Debug, PartialEq, Eq)]
     pub(crate) struct Index<K: Eq + Hash> {
         rows: Vec<Row<K>>,
         /// Key → its place in `rows`.
         at: HashMap<K, usize>,
         duplicates: Vec<K>,
     }

     impl<K: Eq + Hash + Clone> Index<K> {
         /// Index `source`. `key` gives an entry's key, `None` for an entry that
         /// is not an item (a menu separator or heading); `enabled` whether the
         /// entry can take the focus or a choice.
         pub(crate) fn new<T>(
             source: &[T],
             key: impl Fn(&T) -> Option<&K>,
             enabled: impl Fn(&T) -> bool,
         ) -> Self {
             let mut index = Self {
                 rows: Vec::with_capacity(source.len()),
                 at: HashMap::with_capacity(source.len()),
                 duplicates: Vec::new(),
             };
             let mut repeated = HashSet::new();
             for (place, entry) in source.iter().enumerate() {
                 let Some(found) = key(entry) else {
                     continue;
                 };
                 if index.at.contains_key(found) {
                     if repeated.insert(found.clone()) {
                         index.duplicates.push(found.clone());
                     }
                 } else {
                     index.at.insert(found.clone(), index.rows.len());
                     index.rows.push(Row {
                         key: found.clone(),
                         enabled: enabled(entry),
                         source: place,
                     });
                 }
             }
             index
         }

         /// The index of a list's items: a disabled item is not enabled.
         pub(crate) fn of_items(items: &[ListItem<K>]) -> Self {
             Self::new(items, |item| Some(&item.key), |item| !item.disabled)
         }

         /// The distinct keys in order.
         pub(crate) fn keys(&self) -> Vec<K> {
             self.rows.iter().map(|row| row.key.clone()).collect()
         }

         /// How many distinct keys there are (an item's `set_size`).
         pub(crate) fn count(&self) -> usize {
             self.rows.len()
         }

         /// The ordinal of `key` among the distinct keys, from 0.
         pub(crate) fn position(&self, key: &K) -> Option<usize> {
             self.at.get(key).copied()
         }

         /// Where the first occurrence of `key` is in the source slice.
         pub(crate) fn source(&self, key: &K) -> Option<usize> {
             self.at.get(key).map(|&row| self.rows[row].source)
         }

         pub(crate) fn duplicates(&self) -> &[K] {
             &self.duplicates
         }

         pub(crate) fn enabled(&self, key: &K) -> bool {
             self.at.get(key).is_some_and(|&row| self.rows[row].enabled)
         }

         /// Navigate only among enabled and available keys, in logical item order.
         /// An empty/all-disabled/unmounted order has no target.
         pub(crate) fn navigate(
             &self,
             current: Option<&K>,
             movement: Movement,
             boundary: ListBoundary,
             available: impl Fn(&K) -> bool,
         ) -> Option<K> {
             let keys: Vec<&K> = self
                 .rows
                 .iter()
                 .filter(|row| row.enabled && available(&row.key))
                 .map(|row| &row.key)
                 .collect();
             let last = keys.len().checked_sub(1)?;
             let current = current.and_then(|key| keys.iter().position(|candidate| *candidate == key));
             let index = match movement {
                 Movement::First => 0,
                 Movement::Last => last,
                 Movement::Next => match current {
                     Some(index) if index < last => index + 1,
                     Some(_) if boundary == ListBoundary::Clamp => last,
                     _ => 0,
                 },
                 Movement::Previous => match current {
                     Some(index) if index > 0 => index - 1,
                     Some(_) if boundary == ListBoundary::Clamp => 0,
                     _ => last,
                 },
             };
             Some(keys[index].clone())
         }
     }
     ```
  3. `src/collection/membership.rs` を新しく作る。全文：
     ```rust
     //! Role: The keys a collection's selection holds, followed per key.
     //!
     //! One memo holds the selected keys as a set. One effect compares each new
     //! set with the previous one and notifies only the notifiers of the keys
     //! that entered or left it. An item reads its key in O(1) and follows only
     //! its own notifier, so a selection change costs O(S) once per collection
     //! plus O(1) per changed key, never O(S) per item. A read is always fresh
     //! (the set is a memo); the notification arrives with the turn's user
     //! effects.

     use crate::Selection;
     use kivact_core::state::{Notifier, Value, Var, effect, memo, notifier, on_cleanup, var};
     use std::cell::RefCell;
     use std::collections::{HashMap, HashSet};
     use std::rc::Rc;

     /// The notifiers of the items that read a key, by key.
     type Watchers<K> = Rc<RefCell<HashMap<K, Vec<Notifier>>>>;

     /// Which keys a selection holds, read per key in O(1). Copy.
     pub(crate) struct Membership<K: kivact_core::state::ItemKey> {
         /// The selected keys: the collection's one memo of them.
         keys: Value<HashSet<K>>,
         /// Shared with the effect that notifies and with each item's cleanup.
         watchers: Var<Watchers<K>>,
     }

     impl<K: kivact_core::state::ItemKey> Clone for Membership<K> {
         fn clone(&self) -> Self {
             *self
         }
     }
     impl<K: kivact_core::state::ItemKey> Copy for Membership<K> {}

     impl<K: kivact_core::state::ItemKey> Membership<K> {
         /// Follow `selection`. Setup only, in the collection's owner.
         #[track_caller]
         pub(crate) fn new(selection: Selection<K>) -> Self {
             let keys = memo(move || match selection {
                 Selection::Single(binding) => {
                     binding.with(|key| key.iter().cloned().collect::<HashSet<K>>())
                 }
                 Selection::Multiple(binding) => {
                     binding.with(|keys| keys.iter().cloned().collect::<HashSet<K>>())
                 }
             });
             let watchers: Watchers<K> = Rc::new(RefCell::new(HashMap::new()));
             let notified = watchers.clone();
             let previous = RefCell::new(keys.get_untracked());
             // Derives notifications only; it writes no state.
             effect(move || {
                 let changed = keys.with(|next| {
                     let mut previous = previous.borrow_mut();
                     let changed = next
                         .symmetric_difference(&previous)
                         .cloned()
                         .collect::<Vec<K>>();
                     *previous = next.clone();
                     changed
                 });
                 if changed.is_empty() {
                     return;
                 }
                 let targets = {
                     let watchers = notified.borrow();
                     changed
                         .iter()
                         .filter_map(|key| watchers.get(key))
                         .flatten()
                         .copied()
                         .collect::<Vec<Notifier>>()
                 };
                 for target in targets {
                     target.notify();
                 }
             });
             Self {
                 keys,
                 watchers: var(watchers),
             }
         }

         /// A notifier for `key`, registered until the current owner goes.
         /// Setup only, in the item's owner.
         #[track_caller]
         pub(crate) fn watch(self, key: K) -> Notifier {
             let watch = notifier();
             let watchers = self.watchers.get();
             watchers
                 .borrow_mut()
                 .entry(key.clone())
                 .or_default()
                 .push(watch);
             on_cleanup(move || {
                 let mut watchers = watchers.borrow_mut();
                 if let Some(list) = watchers.get_mut(&key) {
                     list.retain(|candidate| *candidate != watch);
                     if list.is_empty() {
                         watchers.remove(&key);
                     }
                 }
             });
             watch
         }

         /// Whether the selection holds `key`. The running computation follows
         /// `watch` (made by [`watch`](Self::watch) for the same key) and
         /// nothing else.
         pub(crate) fn contains(self, key: &K, watch: Notifier) -> bool {
             watch.track();
             self.keys.with_untracked(|keys| keys.contains(key))
         }

         /// The selected keys as one set, tracked as a whole: for computations
         /// over the whole collection (the active row, the tab stop), never per item.
         pub(crate) fn keys(self) -> Value<HashSet<K>> {
             self.keys
         }
     }
     ```
     cleanup は Var ではなく、setup 時に取り出した `Rc` を持つ。そのため、list の owner が item の owner より先に破棄されても、破棄済みの Var には触らない。
  4. `src/list.rs`：`mod order;` の行を消す。`git rm crates/kivact-interaction/src/list/order.rs`。
  5. `src/list/input.rs`：`use super::order::Movement;` を `use crate::collection::Movement;` にする。続けて、field の名前を一括で変える：
     ```
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     perl -pi -e 's/\b(state|self)\.order\b/$1.index/g' crates/kivact-interaction/src/list/state.rs crates/kivact-interaction/src/list/input.rs
     ```
     （`state.order.with_untracked(|order| …)` は `state.index.with_untracked(|order| …)` になる。closure の引数の名前 `order` はそのままでよい。`Index` は `navigate`・`enabled`・`keys` を同じ署名で持つ。）
  6. `src/list/state.rs`：
     - import の `use super::order::Order;` を消し、`use crate::collection::{Index, Membership};`、`use kivact_core::state::{Selector, selector};`、`use std::collections::HashSet;` を足す。
     - `struct State` の `pub(super) order: Value<Order<K>>,` を次に置き換える：
       ```rust
       pub(super) index: Value<Index<K>>,
       /// The number of distinct items; one memo shared by every item.
       pub(super) set_size: Value<u32>,
       /// Which keys the selection holds, per key.
       pub(super) membership: Membership<K>,
       /// Whether a key is the active one, per key.
       pub(super) active_key: Selector<K>,
       ```
     - `create` の中：
       - `let order = memo(move || Order::new(items.get()));` を次の 2 文に置き換える：
         ```rust
         let index = memo(move || items.with(|items| Index::of_items(items)));
         let set_size =
             memo(move || index.with(|index| u32::try_from(index.count()).unwrap_or(u32::MAX)));
         ```
         `let selected_value = memo(move || selected(selection));` の直後に `let membership = Membership::new(selection);` を足す。
       - `let options = OptionsCell::new(ListSetupOptions { … anchor: signal(None::<K>).into_binding(), });` を `let options = default_setup_options::<K>();` にする。消した literal は次の新しい関数（`create` の直後に置く）に移す（`create` を clippy の `too_many_lines`（80 行）以内に保つため）：
         ```rust
         fn default_setup_options<K: kivact_core::state::ItemKey>() -> OptionsCell<ListSetupOptions<K>> {
             OptionsCell::new(ListSetupOptions {
                 retained_focus: false.into_value(),
                 reset_on_exit: true,
                 page: 10.into_value(),
                 leave: None,
                 scroll: None,
                 selectable: None,
                 anchor: signal(None::<K>).into_binding(),
             })
         }
         ```
       - `position` の memo の `.and_then(|key| order.with(|order| order.position(&key)))` を `.and_then(|key| index.with(|index| index.position(&key)))` にする。
       - `let active = active_value(order, targets, selected_value, option_values, pending, mode, position,);` の引数の `order` を `index` に、`selected_value` を `membership.keys()` に変える。その直後に `let active_key = selector(active);` を足す。
       - `Rc::new(State { … })` の `order,` を `index, set_size, membership, active_key,` にする（`selected: selected_value,` は残す。公開の `ListApi::selected()` が使う）。
     - `fn active_value` の引数 `order: Value<Order<K>>` を `order: Value<Index<K>>` に、`selected_value: Value<Vec<K>>` を `selected_value: Value<HashSet<K>>` にする。本文は 1 か所だけ変える：`.find(|key| selected.contains(key) && order.enabled(key) && available(key));` を `.find(|key| selected.contains(*key) && order.enabled(key) && available(key));` にする（`key` は `&&K` で、`HashSet::contains` は自動で deref しない）。`let selected = selected_value.get();` と `selected.is_empty()` はそのまま。これで tab stop の選択済みの優先が O(N·S) から O(N) になる。
     - `observe_list` の `state.order.with(|order| order.duplicates().to_vec())` は、手順 5 の perl で `state.index.with(...)` になっている。
  7. `src/list/parts.rs`：import の `use super::state::{State, selected};` を `use super::state::State;` にする。`pub(super) fn item` の本文（`let key = var(key);` から `(attachment, bound)` まで）を次に置き換える。`mount_item`、`observe_item`、`container` は変えない。
     ```rust
     let watch = state.membership.watch(key.clone());
     let key = var(key);
     let bound = signal(ItemMount::Unbound);
     let (membership, active_key, index, items) =
         (state.membership, state.active_key, state.index, state.items);
     // The selection and the active key are read per key: a change notifies
     // only the items whose answer changed.
     let selected = move || key.with(|key| membership.contains(key, watch));
     let active =
         move || bound.get() == ItemMount::Admitted && key.with(|key| active_key.is(key));
     let name = memo(move || {
         let at = key.with(|key| index.with(|index| index.source(key)));
         at.and_then(|at| items.with(|items| items.get(at).map(|item| item.label.get())))
             .unwrap_or_default()
     });
     let input = part(move |input| ListItemState {
         input,
         selected: selected(),
         active: active(),
     });
     let part_state = input.state();
     observe_item(&state, key, part_state);
     let set_size = state.set_size;
     let attachment = Part::new(part_state, move |host| {
         let attachment = host.part(input);
         let keys = state.clone();
         let pointers = state.clone();
         let focus_mode = state.pending;
         let attachment = attachment
             .role(Role::Option)
             .name(name)
             .selected(selected)
             .set_size(set_size)
             .pos_in_set(move || {
                 key.with(|key| {
                     index.with(|index| {
                         index
                             .position(key)
                             .map_or(0, |at| u32::try_from(at + 1).unwrap_or(u32::MAX))
                     })
                 })
             })
             .disabled(move || {
                 bound.get() == ItemMount::Rejected
                     || key.with(|key| !index.with(|index| index.enabled(key)))
             })
             .focusable(move || {
                 focus_mode.state().get().focus == machine::Focus::Physical && active()
             })
             .focusable_when_disabled(retain_disabled_focus)
             .on_key(move |event| super::input::key(&keys, &key.get(), event, None))
             .on_pointer(move |event| super::input::pointer(&pointers, &key.get(), event));
         attachment.on_attach(move |mount| {
             mount_item(state.clone(), key, bound, retain_disabled_focus, mount);
         })
     });
     (attachment, bound)
     ```
     消える memo は `selected_state`（`selected(state.selection).contains(key)`）、`active`、`disabled`、`name` の `items.iter().find(|item| &item.key == key)`、`set_size`、`position` の 6 つ。残る memo は `name` と、core の `part(..)` が作る状態の memo の 2 つ。closure `selected`・`active` は Copy の値（`Var`、`Notifier`、`Membership`、`Selector`、`Signal`）しか捕まえないので Copy になる。
  8. `src/list/handle.rs`：
     - import に `use crate::collection::{Index, Membership};` と `use kivact_core::state::Selector;` を足す。
     - `pub fn item_state` の本文を次にする（`state.selected.with(|keys| keys.contains(&key))` と `state.active.with(|active| active.as_ref() == Some(&key))` の fallback をやめる。getter と `item_states` の map は U31 まで残す）：
       ```rust
       let state = self.state();
       let watch = state.membership.watch(key.clone());
       kivact_core::state::memo(move || {
           state
               .item_states
               .with(|states| states.get(&key).copied())
               .map(|part| part.get())
               .unwrap_or_else(|| ListItemState {
                   input: PartState::default(),
                   selected: state.membership.contains(&key, watch),
                   active: state.active_key.is(&key),
               })
       })
       ```
     - `pub(crate) fn base_item` の後に、composite が使う accessor を足す：
       ```rust
       /// The list's index of its items, read per key in O(1), for composites.
       pub(crate) fn index(self) -> Value<Index<K>> {
           self.state().index
       }
       /// Which keys the selection holds, per key, for composites.
       pub(crate) fn membership(self) -> Membership<K> {
           self.state().membership
       }
       /// Whether a key is the active one, per key, for composites.
       pub(crate) fn active_key(self) -> Selector<K> {
           self.state().active_key
       }
       ```
  9. `src/tabs.rs`：
     - `pub fn item` の本文全体を次に置き換える。消える `label` memo（`options.with(|rows| key.with(|key| rows.iter().find(|row| &row.key == key) …))`）と `.name(label)` は、list の item の name が同じ `ListItem::label` を index 経由で付けるので重複していた。
       ```rust
       let (tab_ref, panel_ref) = self.refs(&key);
       let member = self.group.item(key);
       let orientation = self.orientation;
       let base = member.map_state(move |state| TabState {
           input: state.input,
           selected: state.selected,
           orientation: orientation.get(),
       });
       let part_state = base.state();
       Part::new(part_state, move |host| {
           host.part(base)
               .role(Role::Tab)
               .controls(panel_ref)
               .node_ref(tab_ref)
       })
       ```
     - `pub fn panel` の本文全体を次に置き換える（`let selected = self.selected();`、`let options = self.options;` と、`selected.with(|selected| selected == key) && options.with(|rows| rows.iter().any(…))` の memo が消える）。list の selection は `Selection::Single(selected.optional())` なので、membership の集合はちょうど `{selected}` になる。`index.position(key).is_some()` は「key が options にある」と同じ答え。
       ```rust
       let (tab_ref, panel_ref) = self.refs(&key);
       let membership = self.group.membership();
       let index = self.group.index();
       let watch = membership.watch(key.clone());
       let key = var(key);
       let orientation = self.orientation;
       let shown = memo(move || {
           key.with(|key| {
               membership.contains(key, watch)
                   && index.with(|index| index.position(key).is_some())
           })
       });
       let input = part(move |input| TabState {
           input,
           selected: shown.get(),
           orientation: orientation.get(),
       });
       let part_state = input.state();
       Part::new(part_state, move |host| {
           host.part(input)
               .role(Role::TabPanel)
               .hidden(move || !shown.get())
               .focusable(shown)
               .labelled_by(tab_ref)
               .node_ref(panel_ref)
       })
       ```
     - 使われなくなった field `options: Value<Vec<ListItem<K>>>,` を `struct TabsApi` から消し、`use_tabs` の `TabsApi { … options, … }` の `options,` の行も消す（`let options = items.into_value();` は `use_list(options, …)` で使うので残す）。
  10. `src/select/handle.rs` の `fn item_in` の本文全体を次に置き換える（項目ごとの memo `keep_trigger_focus`（`self.settings` を読む）と `popup_selected`（`list.active()` を項目ごとに比べる）が消え、共有の値を読む closure と list の `active_key` の selector になる）：
      ```rust
      let SelectApi {
          popup,
          list,
          hovered,
          settings,
          ..
      } = self;
      let keep_trigger_focus = move || {
          !sheet && settings.value().with(|options| options.keep_trigger_focus.get())
      };
      let member = list.base_item(key.clone(), false);
      let active_key = list.active_key();
      let part_state = member.state();
      Part::new(part_state, move |host| {
          let option_ref = kivact_core::view::node_ref();
          let member = host.part(member).node_ref(option_ref);
          let member = if keep_trigger_focus() {
              let selected_key = key.clone();
              member.selected(move || active_key.is(&selected_key))
          } else {
              member
          };
          let hover_key = key.clone();
          let invoke_key = key;
          member
              .on_press(move || {
                  if list.select_mounted(invoke_key.clone()) {
                      if sheet {
                          popup.open_binding().set(false);
                      } else {
                          popup.hide(PopupCloseReason::Selection);
                      }
                  }
              })
              .on_pointer(move |event: PointerEvent| {
                  match event.phase() {
                      PointerPhase::Enter | PointerPhase::Move => {
                          if keep_trigger_focus() {
                              list.highlight_mounted(hover_key.clone());
                          } else {
                              hovered.set(Some(hover_key.clone()));
                          }
                      }
                      PointerPhase::Leave
                          if hovered.get_untracked().as_ref() == Some(&hover_key) =>
                      {
                          hovered.set(None);
                      }
                      _ => {}
                  }
                  Handled::No
              })
              .on_attach(move |_| {
                  let focused = option_ref.focused();
                  effect(move || {
                      if focused.get() {
                          option_ref.scroll_into_view();
                      }
                  });
              })
      })
      ```
- **消すもの**：`src/list/order.rs`（`Order<K>`。`Movement` は `collection::index` へ移る）、`list.rs` の `mod order;`、`list/parts.rs` の項目ごとの memo 6 つ（`selected_state`、`active`、`disabled`、`name` の線形探索、`set_size`、`position`）と `selected` の import、`tabs.rs` の `item` の `label` memo、`panel` の線形探索の memo、`TabsApi::options` field、`select/handle.rs` の `popup_selected` と項目ごとの `keep_trigger_focus` memo。
- **状態の閉じ方**：
  - `Index<K>`：数える field（bool・Option・counter・Cell・RefCell・Signal・Var・Atomic）が 0 で、構築後に書く method も無いので対象外。
  - `Row<K>`：`enabled: bool` と `source: usize` を持つが、constructor（`Index::new`）の中でしか書かないので対象外（6.1 の (b) を満たさない）。
  - `Membership<K>`：数える field は `watchers: Var<…>` の 1 つだけ（`keys` は `Value`）なので対象外。registry は `watch` の登録と cleanup だけが書く寿命の記録で、effect は通知を出すだけで状態を書かない（S-5）。
  - list の `State<K>`（kivact-interaction の型なので `State` の接尾辞では除外されない）に足す field は `Value`・`Membership`・`Selector` で、どれも数える型ではない。W1 が付けた印の内容は変わらないので、印は直さない。
- **試験**：足す試験も直す試験も無い（公開の契約は変わらない）。K5（性能）のための新しい試験は足さない（6.4：時間の厳密な比較は足さない）。効果は CP1 の bench で Orchestrator が測る。そのまま通るべき E2E：
  - `e2e_ui`：`interaction_list_nav`、`interaction_tabs`、`interaction_select`、`interaction_state`、`interaction_menu`、`interaction_table`、`interaction_tree`、`interaction_show_hide`、`interaction_edit`、`apg_listbox`、`apg_tabs`、`apg_radio_group`、`apg_menu`（filter が `apg_menu_button`、`apg_menubar` にも一致する）、`apg_tree_view`、`apg_grid`、`ui_components`、`ui_controls`、`ui_data_table`
  - `e2e_kits`（package `kivact-kit-tests`）：`apg_grid_kit`、`ui_standard`
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction
  cargo check -p kivact-interaction --all-targets
  cargo check -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo test -p kivact-interaction
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_list_nav interaction_tabs interaction_select interaction_state interaction_menu interaction_table interaction_tree interaction_show_hide interaction_edit apg_listbox apg_tabs apg_radio_group apg_menu apg_tree_view apg_grid ui_components ui_controls ui_data_table
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-kit-tests --test e2e_kits -- apg_grid_kit ui_standard
  ```
- **完了条件**：
  - `test ! -e crates/kivact-interaction/src/list/order.rs`
  - `rg -nw "Order" crates/kivact-interaction/src/list` が何も返さない。
  - `rg -n "iter\(\)\s*\.find|\.contains\(key\)" crates/kivact-interaction/src/list/parts.rs crates/kivact-interaction/src/tabs.rs` が何も返さない。
  - `rg -c "memo\(" crates/kivact-interaction/src/list/parts.rs` が `1` を返す。
  - `rg -n "selector\(active\)" crates/kivact-interaction/src/list/state.rs` が 1 行を返す。
  - `rg -n "popup_selected" crates/kivact-interaction/src/select/handle.rs` が何も返さない。
  - 上のコマンドと E2E がすべて通る。

## U03：combobox の option、menu の item、command palette の行（UI-02 の B）

- **wave / lane / 前提**：W2a / U / U02（`collection::{Index, Membership}`、`ListApi::{index, membership, active_key}`）。この unit は W5 より前に走るので、HEAD の名前（`Notifier`・`Binding`・`IntoBinding`）で書く。
- **所有**：`crates/kivact-interaction/src/combobox/handle.rs`、`crates/kivact-interaction/src/menu/level.rs`、`crates/kivact-interaction/src/palette/handle.rs`
- **目的**：UI-02、PERF-11。combobox の option ごとの `options.iter().find`（mount が O(N²)）と、active・値の memo をなくす。menu の item ごとの memo を 12（U02 の前は約 17）から 5 にする（`item_entry` の線形探索と `MenuEntry` の複製をやめる）。palette の filter と群分けの O(N) の find をなくす。
- **手順**：
  1. `combobox/handle.rs`：
     - import に `use crate::collection::{Index, Membership};` と `use kivact_core::state::{Selector, Var, selector, var};` を足す。
     - `pub struct ComboboxApi` の `active: Value<Option<K>>,` の次に field を足す：
       ```rust
       /// The options by key: one index shared by every option part.
       index: Value<Index<K>>,
       /// Whether a key is the active option, per key.
       active_key: Selector<K>,
       /// Which keys the selection holds, per key.
       membership: Membership<K>,
       ```
     - `use_combobox` を clippy の `too_many_lines`（80 行）以内に保つため、`let shown = memo(…)`、`let visible = memo(…)`、`let active = memo(…)` の 3 文（本文は一字一句そのまま）を、`use_combobox` の直後に置く次の新しい関数に移す：
       ```rust
       type Shown<K> = (Value<Vec<ListItem<K>>>, Value<Vec<K>>, Value<Option<K>>);

       /// The options the query shows, their keys, and the active option: the
       /// highlighted one while it is shown and enabled, else the first enabled one.
       fn shown_rows<K: kivact_core::state::ItemKey>(
           options: Value<Vec<ListItem<K>>>,
           query: Value<String>,
           filter: Signal<Filter<K>>,
           session: MachineHandle<machine::Machine<K>>,
       ) -> Shown<K> {
           // Here: the three `let shown = memo(…)`, `let visible = memo(…)` and
           // `let active = memo(…)` statements moved from `use_combobox` unchanged.
           (shown, visible, active)
       }
       ```
       （コメントの 2 行は、移した 3 文に置き換える。）`use_combobox` の中は、`let filter = signal::<Filter<K>>(Rc::new(contains_query));` の後を次にする：
       ```rust
       let (shown, visible, active) = shown_rows(options, query, filter, session);
       let index = memo(move || options.with(|all| Index::of_items(all)));
       let active_key = selector(active);
       let membership = Membership::new(selection);
       ```
       `ComboboxApi { … }` の literal の `active,` の次に `index, active_key, membership,` を足す。
     - `pub fn item_state` の本文を次にする（fallback の `active: self.active.with(…)` と `selected: match self.selection { … values.contains(&key) … }` をやめる）：
       ```rust
       let watch = self.membership.watch(key.clone());
       memo(move || {
           self.item_states
               .with(|states| states.get(&key).copied())
               .map(|state| state.get())
               .unwrap_or_else(|| ListItemState {
                   input: PartState::default(),
                   active: self.active_key.is(&key),
                   selected: self.membership.contains(&key, watch),
               })
       })
       ```
     - `pub fn item`（今は `let item = { … all.iter().find(|option| option.key == key) … }` と `is_active`・`is_value`・`disabled` の memo を持つ）の本文全体を次に置き換える（doc comment と `#[track_caller]` は残す）：
       ```rust
       let ComboboxApi {
           options,
           index,
           active,
           active_key,
           membership,
           rows,
           selection,
           item_states,
           session,
           ..
       } = self;
       let watch = membership.watch(key.clone());
       let key = var(key);
       let name = memo(move || {
           let at = key.with(|key| index.with(|index| index.source(key)));
           at.and_then(|at| options.with(|all| all.get(at).map(|option| option.label.get())))
               .unwrap_or_default()
       });
       let is_active = move || key.with(|key| active_key.is(key));
       let is_value = move || key.with(|key| membership.contains(key, watch));
       let disabled = move || key.with(|key| !index.with(|index| index.enabled(key)));
       let input = part(move |input| ListItemState {
           input,
           selected: is_value(),
           active: is_active(),
       });
       let part_state = input.state();
       item_states.update(|states| {
           states.insert(key.get(), part_state);
       });
       on_cleanup(move || {
           if item_states.try_get_untracked().is_some() {
               item_states.update(|states| {
                   let cleanup_key = key.get();
                   if states.get(&cleanup_key) == Some(&part_state) {
                       states.remove(&cleanup_key);
                   }
               });
           }
       });
       let target = kivact_core::view::node_ref();
       Part::new(part_state, move |host| {
           host.part(input)
               .node_ref(target)
               .role(Role::Option)
               .name(name)
               .selected(move || match selection {
                   Selection::Single(_) => is_active(),
                   Selection::Multiple(_) => is_value(),
               })
               .disabled(disabled)
               .on_pointer(move |event: PointerEvent| {
                   if matches!(event.phase(), PointerPhase::Enter | PointerPhase::Move)
                       && !disabled()
                       && !active.with_untracked(|active| {
                           key.with(|key| active.as_ref() == Some(key))
                       })
                   {
                       session.send(machine::Event::Highlight(key.get()));
                   }
                   Handled::No
               })
               .on_press(move || self.choose(key.get()))
               .on_attach(move |_| {
                   register_row(rows, key.get(), target);
                   follow_active_row(target, key, active_key);
               })
       })
       ```
     - `fn item_selected` を消す（使い手は `item` だけだった）。
     - `fn follow_active_row(row: NodeRef, is_active: Value<bool>)` を次にする。active が変わったとき、通知を受けるのは前後の 2 行の effect だけになる。
       ```rust
       fn follow_active_row<K: kivact_core::state::ItemKey>(
           row: NodeRef,
           key: Var<K>,
           active_key: Selector<K>,
       ) {
           effect(move || {
               if key.with(|key| active_key.is(key)) {
                   row.scroll_into_view();
               }
           });
           on_mount(move || {
               if key.with(|key| active_key.is(key)) {
                   row.scroll_into_view();
               }
           });
       }
       ```
  2. `palette/handle.rs`：
     - import に `use crate::collection::Index;` を足す。
     - `use_command_palette` の `let options = memo(…)` の前に次を足す：
       ```rust
       let index = memo(move || {
           commands.with(|all| {
               Index::new(all, |command| Some(&command.key), |command| !command.disabled)
           })
       });
       ```
     - `use_combobox(options, Selection::Single(chosen)).filter(move |query, option| { … })` の closure の本文（`commands.with(|all| { all.iter().find(|command| command.key == option.key).is_some_and(|command| filter.with(|filter| filter(query, command))) })`）を次にする：
       ```rust
       let at = index.with(|index| index.source(&option.key));
       at.is_some_and(|at| {
           commands.with(|all| {
               all.get(at)
                   .is_some_and(|command| filter.with(|filter| filter(query, command)))
           })
       })
       ```
     - `let groups = memo(move || command_groups(visible, commands));` を `let groups = memo(move || command_groups(visible, commands, index));` にする。`fn command_groups` に 3 つ目の引数 `index: Value<Index<K>>,` を足し、`visible.with(|keys| { commands.with(|all| { for key in keys { let Some(command) = all.iter().find(|command| command.key == *key) else { continue; }; … } }) });` を次にする（`let group = …` から `run.commands.push(command.clone());` までの群分けの本文は変えない）：
       ```rust
       visible.with(|keys| {
           index.with(|index| {
               commands.with(|all| {
                   for key in keys {
                       let Some(command) = index.source(key).and_then(|at| all.get(at)) else {
                           continue;
                       };
                       // The grouping body from `let group = …` on, unchanged.
                   }
               })
           })
       });
       ```
       （コメントの 1 行は、元の群分けの本文に置き換える。）
     - 行の part（`CommandPaletteApi::item` は `self.combobox.item(key)`）は手順 1 で軽くなるので、ここでは変えない。
  3. `menu/level.rs`：
     - import：`use crate::collection::Index;` を足す。`use kivact_core::state::{Signal, Value, effect, memo, on_cleanup, signal};` を `use kivact_core::state::{Selector, Signal, Value, Var, effect, memo, on_cleanup, selector, signal, untrack, var};` にする。
     - `struct Level` の `pub(super) rows: Value<Rows<K>>,` の次に field を足す：
       ```rust
       /// This level's rows by key, in O(1).
       index: Value<Index<K>>,
       /// Whether a key is the item whose submenu is open at this level, per key.
       open_key: Selector<K>,
       ```
     - `Level::new` の `let level_rows = memo(…);` の次に次の 2 文を足し、`Self { … }` の `rows: level_rows,` の次に `index, open_key,` を足す：
       ```rust
       let index = memo(move || {
           level_rows.with(|rows| {
               Index::new(rows, |(_, entry)| entry.key(), |(_, entry)| !entry.is_disabled())
           })
       });
       let open_key = selector(move || session.path.with(|path| path.get(depth).cloned()));
       ```
       （level は index を 2 つ持つ：list の項目の index と、separator・heading を含む行から `MenuEntry` を引くためのこの行の index。どちらも level ごとに 1 つで、item ごとには作らない。）
     - 新しい関数を 2 つ足す（`item_display` の直後）：
       ```rust
       /// Read the entry of the item `key` in a level's rows, tracked, through
       /// the level's index in O(1); `None` when the level has no such item.
       fn with_entry<K: kivact_core::state::ItemKey, R>(
           rows: Value<Rows<K>>,
           index: Value<Index<K>>,
           key: Var<K>,
           f: impl FnOnce(Option<&MenuEntry<K>>) -> R,
       ) -> R {
           let at = key.with(|key| index.with(|index| index.source(key)));
           rows.with(|rows| f(at.and_then(|at| rows.get(at)).map(|(_, entry)| entry)))
       }

       /// The role and the check or submenu state of an item's node.
       fn item_role(
           member: kivact_core::view::Node,
           kind: Option<MenuItemKind>,
           check: impl Fn() -> Option<MenuCheck> + 'static,
           expanded: impl Fn() -> bool + Copy + 'static,
       ) -> kivact_core::view::Node {
           match kind {
               Some(MenuItemKind::Radio) => member
                   .role(Role::MenuItemRadio)
                   .checked(move || check().is_some_and(MenuCheck::is_checked)),
               Some(MenuItemKind::Checkbox) => member
                   .role(Role::MenuItemCheckBox)
                   .checked(move || check().is_some_and(MenuCheck::is_checked)),
               Some(MenuItemKind::Submenu) => member
                   .role(Role::MenuItem)
                   .has_popup(Some(kivact_core::view::PopupKind::Menu))
                   .expanded(expanded)
                   .selected(expanded),
               _ => member.role(Role::MenuItem),
           }
       }
       ```
     - `fn item_description(entry: Value<Option<MenuEntry<K>>>, disabled: Value<bool>, retain_disabled_focus: bool) -> Value<String>`（memo 2 つ）を、memo を作らない純粋な関数に置き換える：
       ```rust
       /// The row's description: its disabled reason while a disabled row keeps
       /// the focus, else its shortcut.
       fn item_description<K>(
           entry: Option<&MenuEntry<K>>,
           disabled: bool,
           retain_disabled_focus: bool,
       ) -> String {
           let Some(entry) = entry else {
               return String::new();
           };
           if retain_disabled_focus
               && disabled
               && let Some(reason) = entry.disabled_reason()
           {
               return reason;
           }
           entry.shortcut().to_string()
       }
       ```
     - `fn item_entry` を消す。
     - `fn watch_item` は変えない（`pressed_over: Value<super::state::PressOver>` のまま。press の machine の状態はドラッグ中の pointer の移動ごとに変わるので、`pressed_over` は memo で項目ごとの値の変化だけを effect に伝える）。
     - `pub(super) fn item` の本文全体（`let Level { session, depth, rows, group, .. } = self;` から `Part::new(…)` の終わりまで）を次に置き換える（doc comment と `#[track_caller]` は残す）：
       ```rust
       let Level {
           session,
           rows,
           index,
           open_key,
           group,
           retain_disabled_focus,
           ..
       } = self;
       let member = group.base_item(key.clone(), retain_disabled_focus);
       let key = var(key);
       // The kind of an item is part of its row's identity.
       let kind = untrack(|| with_entry(rows, index, key, |e| e.and_then(MenuEntry::item_kind)));
       let submenu = kind == Some(MenuItemKind::Submenu);
       // A Value: the menu press keeps it for its hit test.
       let disabled =
           memo(move || with_entry(rows, index, key, |e| e.is_none_or(MenuEntry::is_disabled)));
       let check = move || with_entry(rows, index, key, |e| e.and_then(MenuEntry::check));
       let destructive =
           move || with_entry(rows, index, key, |e| e.is_some_and(MenuEntry::is_destructive));
       let expanded = move || submenu && key.with(|key| open_key.is(key));
       let item_ref = kivact_core::view::node_ref();
       if let Some(press) = session.press {
           press.row(item_ref, disabled);
       }
       let pressed_over =
           memo(
               move || match session.press.and_then(|press| press.highlighted(item_ref)) {
                   None => super::state::PressOver::None,
                   Some(true) => super::state::PressOver::Over,
                   Some(false) => super::state::PressOver::Away,
               },
           );
       self.watch_item(key.get(), submenu, item_ref, disabled, pressed_over);
       let member = member.map_state(move |state| MenuItemState {
           pressed_over: pressed_over.get(),
           input: state.input,
           display: item_display(check(), submenu, expanded()),
           destructive: destructive(),
       });
       let part_state = member.state();
       self.observe_item(key.get(), part_state);
       Part::new(part_state, move |host| {
           let member = host
               .part(member)
               .node_ref(item_ref)
               .disabled(disabled)
               .description(move || {
                   let disabled = disabled.get();
                   with_entry(rows, index, key, |e| {
                       item_description(e, disabled, retain_disabled_focus)
                   })
               })
               .on_pointer(move |event: PointerEvent| self.item_pointer(event))
               .on_press(move || {
                   let keep_open =
                       with_entry(rows, index, key, |e| e.is_some_and(MenuEntry::keeps_open));
                   self.activate_item(&key.get(), disabled.get(), submenu, keep_open)
               });
           item_role(member, kind, check, expanded)
       })
       ```
       item ごとの memo はこうなる：list の base の 2 つ（part の状態と name）、`disabled`（menu の press が `Value<bool>` で持つ）、`pressed_over`、`map_state` の合計 5 つ（U02 の後で 12、U02 の前は約 17）。この unit で消える memo は `item_entry`、`shortcut`、`description`、`check`、`destructive`、`keep_open`、`expanded`（`session.path` を全 item が読んでいた）の 7 つ。
- **消すもの**：`ComboboxApi::item_selected`、combobox の item の `item`・`is_active`・`is_value`・`disabled` の memo、`menu/level.rs` の `item_entry` と memo を作る旧 `item_description`、menu の item の memo 7 つ（`item_entry`、`shortcut`、`description`、`check`、`destructive`、`keep_open`、`expanded`）、palette の filter と `command_groups` の `all.iter().find`。
- **状態の閉じ方**：
  - `ComboboxApi`：名前が `Api` で終わるので対象外。足す field は `Value`・`Selector`・`Membership` で、どれも数える型ではない。
  - `Level`：足す field は `Value<Index<K>>` と `Selector<K>` で、数える型ではない。W1 が付けた印の内容は変わらないので、印は直さない。
  - `Shown<K>` は型の別名で、struct ではない。新しい状態の型は無い。
- **試験**：足す試験も直す試験も無い。K5 の性能の試験は足さない（効果は CP1 の bench で測る）。そのまま通るべき E2E（`e2e_ui`）：`interaction_combobox`、`interaction_palette`、`interaction_menu`、`interaction_show_hide`、`interaction_select`、`apg_combobox`、`apg_menu`（`apg_menu_button`、`apg_menubar` を含む）、`ui_combobox`、`ui_command_palette`、`ui_components`。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction
  cargo check -p kivact-interaction --all-targets
  cargo check -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo test -p kivact-interaction
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_combobox interaction_palette interaction_menu interaction_show_hide interaction_select apg_combobox apg_menu ui_combobox ui_command_palette ui_components
  ```
- **完了条件**：
  - `rg -n "fn item_entry|fn item_selected" crates/kivact-interaction/src` が何も返さない。
  - `rg -n "\.find\(" crates/kivact-interaction/src/palette/handle.rs` が何も返さない。
  - `rg -n "\.find\(\|option\| option\.key == key\)" crates/kivact-interaction/src/combobox/handle.rs` が何も返さない。
  - `rg -c "memo\(" crates/kivact-interaction/src/menu/level.rs` が `7` を返す（`level_rows`、`items`、`index`、item の `disabled`・`pressed_over`、corridor の `anchor`・`surface`。変更前は 13）。
  - 上のコマンドと E2E がすべて通る。

## U04：data table の行と cell、tree（UI-02 の C）

- **wave / lane / 前提**：W3a / U / U03（`collection::Index`、`ListApi::{index, membership}`）。この unit は W5 より前に走るので、HEAD の名前（`Binding`・`IntoBinding`・`Notifier`）で書く。
- **所有**：`crates/kivact-interaction/src/table/state.rs`、`crates/kivact-interaction/src/table/cells.rs`、`crates/kivact-interaction/src/tree.rs`。`table/handle.rs` は変えない（Rows の行は U02 から list の part を通じて O(1)。残る `find` は列の検索だけで、列の数は小さい）。
- **目的**：UI-02、PERF-11。1,000×10 の表で選択を一つ変えると、今は 10,000 を超える cell の memo が O(S) で走り直す。これを、変わった key の行と cell だけを通知する形にする（cell ごとの notifier は 1 つ、memo は 0）。表の tab stop の O(N·S) と、tree の `keys` memo と O(N) の `contains` をなくす。
- **手順**：
  1. `table/state.rs`：
     - import に `use kivact_core::state::{Selector, selector};` を足し、`use std::collections::HashMap;` を `use std::collections::{HashMap, HashSet};` にする。
     - `struct State` の `pub(super) active: Value<Option<(K, C)>>,` の次に field を足す：
       ```rust
       /// Whether a row holds the active cell, per row key.
       pub(super) active_row: Selector<K>,
       /// Whether a cell is the active cell, per cell.
       pub(super) active_cell: Selector<(K, C)>,
       ```
     - `create` を clippy の `too_many_lines`（80 行）以内に保つため、`let items = memo(move || { let label = settings.with(|options| options.label.clone()); keys.with(|keys| { … }) });` の memo を、`create` の直後に置く次の新しい関数に移し、`create` では `let items = row_items(keys, settings);` と書く。`memo(move || { … })` の `{ … }` の中身（`let label = …;` から `keys.with(…)` の終わりまで）は一字一句そのまま移す：
       ```rust
       /// The rows as list items, named by the table's label function.
       fn row_items<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey>(
           keys: Value<Vec<K>>,
           settings: Value<DataTableOptions<K, C>>,
       ) -> Value<Vec<ListItem<K>>> {
           memo(move || {
               let label = settings.with(|options| options.label.clone());
               keys.with(|keys| {
                   keys.iter()
                       .map(|key| match &label {
                           Some(label) => {
                               let (label, key) = (label.clone(), key.clone());
                               ListItem::new(key.clone(), move || label(&key))
                           }
                           None => ListItem::new(key.clone(), ""),
                       })
                       .collect::<Vec<_>>()
               })
           })
       }
       ```
     - `let selected = rows.selected();` を消す。`let tab_stop = tab_stop_value(keys, columns, selected, registry);` を `let tab_stop = tab_stop_value(keys, columns, rows.membership().keys(), registry);` にする。`let active = memo(…);` の次に次を足し、`Rc::new(State { … })` の `active,` の次に `active_row, active_cell,` を足す：
       ```rust
       let active_row =
           selector(move || active.with(|active| active.as_ref().map(|(row, _)| row.clone())));
       let active_cell = selector(active);
       ```
     - `fn tab_stop_value` の引数 `selected: Value<Vec<K>>` を `selected: Value<HashSet<K>>` にする。本文は 1 か所だけ変える：`.filter(|key| selected.contains(key))` を `.filter(|key| selected.contains(*key))` にする（`key` は `&&K` で、`HashSet::contains` は自動で deref しない）。これで O(N·S) が O(N) になる。
  2. `table/cells.rs` の `pub(super) fn row`：`let key = var(key);` から `let input = part(move |input| TableRowState { … });` までを次に置き換える（`let part_state = input.state();` 以降の observe の部分、`row_name`、apply の `row.selected(is_selected)` は変えない。`is_selected` は Copy の closure なので、そのまま渡せる）：
     ```rust
     let membership = state.list.membership();
     let watch = membership.watch(key.clone());
     let key = var(key);
     let active_row = state.active_row;
     let is_selected = move || key.with(|key| membership.contains(key, watch));
     let input = part(move |input| TableRowState {
         input,
         selected: is_selected(),
         active: key.with(|key| active_row.is(key)),
     });
     ```
  3. `table/cells.rs` の `pub(super) fn cell`：
     - `let id = var((key, column)); let selected = state.list.selected(); let is_selected = memo(move || id.with(|(key, _)| selected.with(|s| s.contains(key))));` の 3 文を次にする（`watch` は `var((key, column))` の前に作る）：
       ```rust
       let membership = state.list.membership();
       let watch = membership.watch(key.clone());
       let id = var((key, column));
       let is_selected = move || id.with(|(key, _)| membership.contains(key, watch));
       ```
     - 二つの `part(move |input| TableCellState { … selected: is_selected.get(), … })` の `is_selected.get()` は `is_selected()` にする。
     - Cells の分岐の `let active = state.active; let is_active = memo(move || id.with(|id| active.with(|active| active.as_ref() == Some(id))));` を `let active_cell = state.active_cell;` にし、その分岐の `active: is_active.get(),` を `active: id.with(|id| active_cell.is(id)),` にする。apply の `.selected(is_selected)` は closure をそのまま渡す。cell ごとの memo は 0 になる（core の part の memo と notifier 1 つは残る）。
  4. `tree.rs`：
     - import に `use crate::collection::Index;` を足す。
     - `use_tree` の `let keys = memo(move || { entries.with(|entries| { entries.iter().filter(|e| !e.disabled).map(|e| e.row.key.clone()).collect::<Vec<_>>() }) });` を消す。`let list = list.leave_to(move |gone: &K| nearest_visible_ancestor(nodes, keys, gone));` を次の 2 文にする。list の項目は同じ entries から作られ、disabled な entry は `disabled` が付くので、`index.enabled(k)` は旧 `keys.contains(k)` と同じ答えを返す。
       ```rust
       let index = list.index();
       let list = list.leave_to(move |gone: &K| nearest_visible_ancestor(nodes, index, gone));
       ```
     - `fn nearest_visible_ancestor` の引数 `keys: Value<Vec<K>>` を `index: Value<Index<K>>` にし、`&& keys.with(|keys| keys.contains(&forest.keys[parent]))` を `&& index.with(|index| index.enabled(&forest.keys[parent]))` にする。
     - `TreeApi::item_part` は変えない。行の list の part は U02 で O(1) になり、`live` の memo は `info` の `HashMap` を O(1) で引くので、tree の行の memo は 4 つ（list の base の 2 つ、`live`、`map_state`。前は 8）で受け入れる（`live` を closure に分けると 5〜6 個の closure が collection の変化のたびに走り、かえって重くなる）。
- **消すもの**：`cells.rs` の行の `is_selected`・`is_active` の memo、cell の `is_selected`・`is_active` の memo、`table/state.rs` の `let selected = rows.selected();`、`tree.rs` の `keys` memo。
- **状態の閉じ方**：表の `State<K, C>` に足す field は `Selector<K>` と `Selector<(K, C)>` で、数える型ではない。W1 が付けた印の内容は変わらないので、印は直さない。新しい型は無い。
- **試験**：足す試験も直す試験も無い。K5 の性能の試験は足さない（大きな表の選択の効果は CP1 の bench で Orchestrator が測る）。そのまま通るべき E2E：
  - `e2e_ui`：`interaction_table`、`interaction_tree`、`interaction_state`、`apg_tree_view`、`apg_grid`、`ui_data_table`、`ui_components`
  - `e2e_kits`（package `kivact-kit-tests`）：`apg_grid_kit`、`ui_standard`
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction
  cargo check -p kivact-interaction --all-targets
  cargo check -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo test -p kivact-interaction
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_table interaction_tree interaction_state apg_tree_view apg_grid ui_data_table ui_components
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-kit-tests --test e2e_kits -- apg_grid_kit ui_standard
  ```
- **完了条件**：
  - `rg -n "s\.contains\(key\)|memo\(" crates/kivact-interaction/src/table/cells.rs` が `row_name` の `memo(` の 1 行だけを返す。
  - `rg -n "let keys = memo" crates/kivact-interaction/src/tree.rs` が何も返さない。
  - `rg -n "selector\(" crates/kivact-interaction/src/table/state.rs` が 2 行を返す。
  - 上のコマンドと E2E がすべて通る。

## U05：combobox の machine 状態を閉じる（UI-05 の内部）
- **wave / lane / 前提**：W3a / U / U04（同じ lane で直前。U03 が `combobox/handle.rs` の `shown`・`visible`・`active` の 3 つの memo を新しい関数 `fn shown_rows(options, query, filter, session)` に移し、`item` を `Index<K>` に変えている。この unit は machine と query と open の行だけを変え、U03 が残した行の形はそのまま使う）。この unit は W5 の前に走るので、名前は HEAD のもの（`Binding<T>`、`IntoBinding`、`into_binding()`、`binding(get, set)`）を使う。
- **所有**：`crates/kivact-interaction/src/combobox/machine.rs`、`crates/kivact-interaction/src/combobox/handle.rs`、`crates/kivact-interaction/src/palette/handle.rs`
- **目的**：UI-05 のうち公開 API を変えない部分。machine の状態を `enum ComboState<K>` にする。query を machine の状態から出して、setup で決まる設定（`OptionsCell<Binding<String>>`）にする（`Query::Local|Shared` と手書きの `PartialEq` を消す）。閉じたときの highlight の消去と inline の query の消去を close の遷移の中で行う。query を消すのは inline の list（command palette のように持ち主が閉じるもの）の close だけで、popup の close は query を消さない（field が text を見せ続けるため。`ui_combobox.rs::free_text_is_kept_when_the_list_closes` がこれを確かめる）。`follow_open`、`Event::Reset`、`Event::Share` を消し、誤ったコメントを直す。`use_combobox`、`.popup(open)`、`popup_with_focus_transfer`、`query_binding`、`with_input_ref`、`ComboboxMode` の公開の形は変えない（W7b の U30b が変える）。
- **手順**：
  1. `combobox/machine.rs` を全部次の内容にする（`Query`、`ComboboxState`、`ComboMode`、`fn reset` とそのコメント "The close transition's reset of the query and the highlight." が消える）。
     ```rust
     //! Role: Own the combobox's open state, filtering phase and highlight in one transition.
     use crate::surface::Surface;
     use kivact_core::state::{Machine as MachineTrait, Step};
     use std::marker::PhantomData;

     /// Whether the list shows, and while it shows, its surface, whether the
     /// query was typed since it opened, and the highlighted option.
     #[derive(Clone, PartialEq)]
     pub(super) enum ComboState<K> {
         Closed,
         Open {
             surface: Surface,
             /// The query was typed, or written by the caller, while open.
             filtering: bool,
             highlight: Option<K>,
         },
     }
     impl<K> ComboState<K> {
         pub(super) fn is_open(&self) -> bool {
             matches!(self, Self::Open { .. })
         }
         pub(super) fn is_filtering(&self) -> bool {
             matches!(self, Self::Open { filtering: true, .. })
         }
         pub(super) fn surface(&self) -> Option<Surface> {
             match self {
                 Self::Open { surface, .. } => Some(*surface),
                 Self::Closed => None,
             }
         }
         pub(super) fn highlight(&self) -> Option<&K> {
             match self {
                 Self::Open { highlight, .. } => highlight.as_ref(),
                 Self::Closed => None,
             }
         }
     }
     pub(super) enum Event<K> {
         /// Show the list in the popover.
         Open,
         /// Show the list in the touch sheet; an open popover moves to the sheet.
         OpenSheet,
         Close,
         /// The caller's open binding, or the palette's open, changed.
         OpenRequested(bool),
         /// The attached field lost the focus; `transferred` keeps the list
         /// open for a presentation that owns the focus instead.
         FocusLeft { transferred: bool },
         /// The user typed this query.
         Query(String),
         /// The caller wrote the shared query.
         QueryChanged,
         Highlight(K),
         /// A multiple choice toggled an option: the search clears and the
         /// list stays open.
         Toggled,
     }
     pub(super) enum Effect {
         /// Write this text to the query binding.
         WriteQuery(String),
         /// Write the caller's open binding.
         WriteOpen(bool),
     }
     /// The transition, with the presentation fixed at setup.
     pub(super) struct Machine<K> {
         /// An inline list (the command palette's) closes only when its owner
         /// closes it, and its next opening starts from an empty query. A popup
         /// keeps its query when it closes: the field still shows the text.
         inline: bool,
         key: PhantomData<fn() -> K>,
     }
     impl<K> Machine<K> {
         pub(super) fn new(inline: bool) -> Self {
             Self { inline, key: PhantomData }
         }
         fn open(surface: Surface, highlight: Option<K>) -> Step<ComboState<K>, Event<K>, Effect> {
             Step::to(ComboState::Open { surface, filtering: false, highlight })
                 .with(Effect::WriteOpen(true))
         }
         /// Close an open list. The highlight goes with the open state; an
         /// inline list also clears its query.
         fn close(&self, state: &ComboState<K>) -> Step<ComboState<K>, Event<K>, Effect> {
             if !state.is_open() {
                 return Step::stay();
             }
             let step = Step::to(ComboState::Closed).with(Effect::WriteOpen(false));
             if self.inline {
                 step.with(Effect::WriteQuery(String::new()))
             } else {
                 step
             }
         }
     }
     impl<K: Clone + PartialEq + 'static> MachineTrait for Machine<K> {
         type State = ComboState<K>;
         type Event = Event<K>;
         type Effect = Effect;
         fn transition(
             &self,
             state: &ComboState<K>,
             event: Event<K>,
         ) -> Step<ComboState<K>, Event<K>, Effect> {
             match event {
                 Event::Open | Event::OpenRequested(true) if state.is_open() => Step::stay(),
                 Event::Open | Event::OpenRequested(true) => Self::open(Surface::Popover, None),
                 Event::OpenSheet => Self::open(Surface::Sheet, state.highlight().cloned()),
                 Event::Close | Event::OpenRequested(false) => self.close(state),
                 Event::FocusLeft { transferred: true } => Step::stay(),
                 Event::FocusLeft { transferred: false } => self.close(state),
                 Event::Query(text) => {
                     let step = Step::to(ComboState::Open {
                         surface: state.surface().unwrap_or(Surface::Popover),
                         filtering: true,
                         highlight: None,
                     })
                     .with(Effect::WriteQuery(text));
                     if state.is_open() { step } else { step.with(Effect::WriteOpen(true)) }
                 }
                 Event::QueryChanged => match state {
                     ComboState::Open { surface, .. } => Step::to(ComboState::Open {
                         surface: *surface,
                         filtering: true,
                         highlight: None,
                     }),
                     ComboState::Closed => Step::stay(),
                 },
                 Event::Highlight(key) => match state {
                     ComboState::Open { surface, filtering, .. } => Step::to(ComboState::Open {
                         surface: *surface,
                         filtering: *filtering,
                         highlight: Some(key),
                     }),
                     ComboState::Closed => Step::stay(),
                 },
                 Event::Toggled => match state {
                     ComboState::Open { surface, filtering, .. } => Step::to(ComboState::Open {
                         surface: *surface,
                         filtering: *filtering,
                         highlight: None,
                     })
                     .with(Effect::WriteQuery(String::new())),
                     ComboState::Closed => Step::stay().with(Effect::WriteQuery(String::new())),
                 },
             }
         }
     }
     ```
  2. `combobox/handle.rs` の import に `use crate::surface::Surface;` と `use kivact_core::state::IntoBinding;` を足す。
  3. `pub struct ComboboxApi` の `mode: OptionsCell<PresentationMode>,` の次に field を足す：
     ```rust
     /// The query text: a local signal, or the caller's binding from
     /// [`query_binding`](Self::query_binding).
     query: OptionsCell<Binding<String>>,
     ```
  4. `pub fn use_combobox` の、コメント `// The one open state; the machine writes the caller's binding, never the reverse.` から `let query = memo(move || session.state().with(|state| state.query.get()));` までを次にする：
     ```rust
     let query = OptionsCell::new(signal(String::new()).into_binding());
     // The one open state; the machine writes the caller's binding, never the reverse.
     let mode = OptionsCell::new(PresentationMode::Inline);
     // A combobox starts inline, so its list shows; `popup` closes it at
     // setup when the caller's open is false.
     let session = machine(
         machine::ComboState::Open {
             surface: Surface::Popover,
             filtering: false,
             highlight: None::<K>,
         },
         move || machine::Machine::<K>::new(matches!(mode.get(), PresentationMode::Inline)),
         move |_, effect| match effect {
             machine::Effect::WriteQuery(text) => query.get().set(text),
             machine::Effect::WriteOpen(open) => {
                 if let PresentationMode::Popup(binding) = mode.get()
                     && binding.get_untracked() != open
                 {
                     binding.set(open);
                 }
             }
         },
     );
     let query_text = memo(move || query.value().get().get());
     ```
     その直後の行（U03 が置いた）`let (shown, visible, active) = shown_rows(options, query, filter, session);` を `let (shown, visible, active) = shown_rows(options, query_text, filter, session);` にする（`fn shown_rows` の中の `let typed = query.get();` は関数の引数 `query: Value<String>` を読むので変えない）。`fn shown_rows` の中の `active` の memo で、`let state = session.state().get();` を `let highlight = session.state().with(|state| state.highlight().cloned());` にし、その下の `state` と次の行の `.highlight`（2 行に分かれた `state` `.highlight`）を `highlight` 1 語にする（`.filter(..)` 以降の行の照合部分は U03 の形のまま）。`ComboboxApi { .. }` の literal の `mode,` の次に `query,` を足す。
  5. `pub fn query_binding` の本体（`self.session.send(machine::Event::Share(query));` と `effect_on(..)` の塊）を次にする：
     ```rust
     let query = query.into_binding();
     self.query.update(|slot| *slot = query);
     // A closed list stays closed when the caller writes the query: the
     // transition ignores the event while closed.
     self.session.feed(query, |_: String| machine::Event::QueryChanged);
     self
     ```
  6. `pub(crate) fn follow_open` を消し、同じ場所に置く：
     ```rust
     /// Open and close an inline list with its owner's `open` (the command
     /// palette's dialog). Closing clears the query and the highlight in the
     /// close transition. Setup only.
     #[track_caller]
     pub(crate) fn opened_by(self, open: Binding<bool>) {
         self.session.send(machine::Event::OpenRequested(open.get_untracked()));
         self.session.feed(open, machine::Event::OpenRequested);
     }
     ```
  7. 次の本体を置き換える（doc はそのまま）。
     - `is_open`：`let state = self.session.state(); (move || state.with(|state| state.is_open())).into_value()`（inline は open で始まり、palette だけが閉じる。「inline は常に true」の偽装を消す）。
     - `is_sheet_open`：`memo(move || self.session.state().with(|state| state.surface() == Some(Surface::Sheet)))`
     - `show_sheet`：`let in_sheet = self.session.state().with_untracked(|state| state.surface() == Some(Surface::Sheet));`（以下同じ）
     - `fn shown`：`self.session.state().with_untracked(|state| state.is_open())`
     - `pub fn query`：`binding(memo(move || self.query.value().get().get()), move |query| { self.session.send(machine::Event::Query(query)); })`
     - `is_filtering`：`memo(move || self.session.state().with(|state| state.is_filtering()))`
  8. `fn choose` の `self.session.send(machine::Event::Reset);` を `self.session.send(machine::Event::Toggled);` にする。`fn input` の `Key::Backspace if ..` の guard の式（`self` `.session` `.state()` `.with_untracked(|state| state.query.get().is_empty())` の 4 行）を `self.query.get().with_untracked(|query| query.is_empty())` にする。
  9. `palette/handle.rs` の `use_command_palette` で、コメント `// The caller's \`open\` is the one opening input: ...` の 2 行と `combobox.follow_open(open);` を次にする：
     ```rust
     // The caller's `open` opens and closes the inline combobox; its close
     // transition clears the query and the highlight.
     combobox.opened_by(open);
     ```
  10. `popup_with_focus_transfer` の本体（`session.send(machine::Event::OpenRequested(..))`、`session.feed(open, machine::Event::OpenRequested)`、`input_ref.focused()` の `effect_on`）は変えない。`.popup(open)` と `popup_with_focus_transfer` が公開のままなので、popup では caller の `Binding<bool>` と machine の状態の同期（`feed(open, Event::OpenRequested)` と `Effect::WriteOpen`）を最小の形で残す。この同期は W7b の U30b が open を caller の `Signal<bool>` 一つにするときに消す。
- **消すもの**：`machine::Query`（と手書き `PartialEq`）、`machine::ComboboxState`、`machine::ComboMode`、`fn reset`、`Event::Share`、`Event::Reset`、`Effect::Query`、`ComboboxApi::follow_open`。
- **状態の閉じ方**：`ComboState<K>` は enum（`filtering` と `highlight` は `Open` の中だけにある。名前の接尾辞 `State` で検査の対象外。`&mut self` の method なし）。`machine::Machine<K>` は constructor だけで書く `bool` 1 つ（状態ではない）。`ComboboxApi` は接尾辞 `Api`。
- **試験**：足す・直す試験なし。`interaction_combobox.rs`（全部。特に `a_shared_query_survives_setup_and_a_hidden_hide`、`writing_the_query_while_closed_keeps_the_popup_closed`、`closing_clears_the_highlight`）、`apg_combobox.rs`、`interaction_palette.rs::closing_resets_the_query_so_a_reopened_palette_lists_its_commands_again`、`ui_command_palette.rs`、`ui_combobox.rs`（`free_text_is_kept_when_the_list_closes` を含む）、`interaction_show_hide.rs::combobox_show_keeps_a_shown_list_and_its_filter` が通ること。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction
  cargo check -p kivact-interaction -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo check -p kivact-ui-creative -p kivact-ui-basic -p kivact-ui-soft -p kivact-ui-pop -p kivact-ui-game --all-targets
  KIVACT_TEST_MIRROR=off cargo test -p kivact-interaction --lib
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_combobox apg_combobox interaction_palette ui_command_palette ui_combobox interaction_show_hide
  ```
- **完了条件**：`rg -n "follow_open|Event::Reset|Event::Share|Query::Local|Query::Shared|ComboMode|ComboboxState|close transition's reset" crates/kivact-interaction/src` が何も返さない。上の命令がすべて通る。

## U06：popup の閉じ方を一つにする（UI-06 の内部）
- **wave / lane / 前提**：W3a / U / U05。この unit は W5 の前に走るので、名前は HEAD のもの（`Binding<T>`、`IntoBinding`、`DialogMode`）を使う。
- **所有**：`crates/kivact-interaction/src/popup/machine.rs`、`crates/kivact-interaction/src/popup/api.rs`、`crates/kivact-interaction/src/dialog.rs`
- **目的**：UI-06。dismissal も `Event::Close(reason)` を送り、`finish_close` を消す。modal は `content_ref().is_modal()` だけから読み、`PopupOptions.modal` と `pub(crate) PopupApi::modal` を消す。Unmount は直接 `Closed` に行き、戻すときは `Effect::Return` を出す。`Effect::Unmounted` と `State::ReturningFocus` を消す。`return_focus_target` を `FocusReturn::anchor_value` にする変更（`FocusReturnTarget` は `#[non_exhaustive]` の無い公開 enum なので variant を足すと公開 API が変わる）は、この unit では行わない。W7b の U30a が `FocusReturn::anchor_value(Value<NodeRef>) -> FocusReturn` を足し、`PopupOptions::return_focus_target` を消す。
- **手順**：
  1. `popup/machine.rs`：
     - `pub(super) enum State` を次にする（`Closing` の `reason` は読まれていないので消す）：
       ```rust
       #[derive(Clone, Copy, PartialEq)]
       pub(super) enum State {
           Closed,
           Open {
               opener: Option<NodeToken>,
               policy: FocusPolicy,
               focus: FocusRange,
           },
           /// Closed by the application, waiting for the surface to unmount.
           /// `opener` is where the unmount returns the focus; none once the
           /// close returned it.
           Closing { opener: Option<NodeToken> },
       }
       ```
     - `pub(super) enum Effect` から `Unmounted,` を消す。
     - `impl Machine` の `fn closing` を消し、代わりに置く：
       ```rust
       /// An accepted close: a reason that restores at once returns the focus
       /// now; any other leaves the return to the unmount.
       fn accepted(&self, target: Option<NodeToken>, immediate: bool) -> Step<State, Event, Effect> {
           match target {
               Some(target) if immediate => {
                   Step::to(State::Closing { opener: None }).with(Effect::Return(target))
               }
               _ => Step::to(State::Closing { opener: target }),
           }
       }
       /// The surface unmounted: closed, returning the focus to `target` when there is one.
       fn unmounted(target: Option<NodeToken>) -> Step<State, Event, Effect> {
           let step = Step::to(State::Closed);
           match target {
               Some(target) => step.with(Effect::Return(target)),
               None => step,
           }
       }
       ```
     - `transition` の `(State::Open { .. }, Event::Accepted { reason, target }) => self.closing(target, reason, matches!(..) || ..)` を `self.accepted(target, matches!(..) || ..)` にする（条件の式はそのまま。上のコメント 4 行も残す）。
     - Unmount の 3 つの腕を次にする：
       ```rust
       (State::Open { .. }, Event::Unmount { was_inside, target })
           if was_inside || (target.is_some() && self.may_restore(target)) =>
       {
           Self::unmounted(target)
       }
       (State::Closing { opener: Some(opener) }, Event::Unmount { was_inside, .. })
           if was_inside || self.may_restore(Some(*opener)) =>
       {
           Self::unmounted(Some(*opener))
       }
       (_, Event::Unmount { .. }) => Step::to(State::Closed),
       ```
  2. `popup/api.rs`：
     - `struct PopupOptions` から `modal: bool,` を消す。`impl PopupApi` の `pub(crate) fn modal(self, modal: bool) -> Self { .. }` を消す。
     - `use_popup` の effect の腕 `machine::Effect::Unmounted => cx.send(Event::Unmount { .. }),` を消す。`machine::Effect::WriteClose` の腕を次にする（caller の handler が先に閉じていた場合に setter を二度呼ばないため、`finish_close` の守りをここへ移す）：
       ```rust
       machine::Effect::WriteClose { reason, opening } => {
           if disclosure.is_open().get_untracked() {
               disclosure.open.set(false);
           }
           if !disclosure.is_open().get_untracked() {
               let target = opening.and_then(|(opener, policy)| {
                   return_target(
                       policy.0,
                       reason,
                       opener,
                       settings.get().anchor.unwrap_or(own_anchor),
                       disclosure.content.is_modal(),
                   )
               });
               cx.send(Event::Accepted { reason, target });
           }
       }
       ```
     - `fn finish_close` を消す。`fn return_target(self, reason)` の 5 つ目の引数 `self.settings.get().modal || self.content_ref().is_modal()` を `self.content_ref().is_modal()` にする。
     - `content_with` の `A11yAction::Dismiss` の handler の `self.finish_close(match reason { .. });` を `self.machine.send(Event::Close(match reason { .. }));` にする（match の腕はそのまま）。上のコメントを次にする：`// An earlier caller dismissal handler may already have closed the binding. The surface is still mounted during this handler chain, so the close effect still computes the reason's focus return.`
     - `fn watch` の effect の `if !open.get() || self.settings.get().modal {` を `if !open.get() || surface.is_modal() {` にする（`surface` は同じ関数の `let surface = self.content_ref();`）。
  3. `dialog.rs` の `use_dialog`：`use_popup(open)` に続く連鎖（今は 3 行：`let popup = use_popup(open)`、`.modal(kind != DialogMode::Popover)`、`.focus_first();`）から `.modal(kind != DialogMode::Popover)` の行を消し、`let popup = use_popup(open).focus_first();` にする（modal は `content` の `surface.modal()` が overlay に付け、`is_modal()` がそれを読む）。
  4. `return_focus_target`、`DialogApi::return_focus`、`base_policy` は変えない（W7b の U30a が変える）。
- **消すもの**：`PopupApi::finish_close`、`PopupApi::modal`、`PopupOptions::modal`、`State::ReturningFocus`、`State::Closing::reason`、`Effect::Unmounted`、`Machine::closing`。
- **状態の閉じ方**：`popup::machine::State` は enum（接尾辞 `State`）。`Closing` は `Option` 1 つ。`PopupOptions` は接尾辞 `Options`（bool が 1 つ減る）。
- **試験**：足す・直す試験なし。`interaction_dialog.rs`（全部。`escape_closes_every_kind_and_the_focus_returns_to_the_opener`、`an_outside_press_closes_all_but_an_alert_modal`、`a_popover_anchored_after_its_content_is_made_opens_and_returns_the_focus_to_the_anchor` を含む）、`interaction_menu.rs`、`interaction_select.rs`、`interaction_show_hide.rs`、`interaction_palette.rs`、`apg_menu_button.rs`、`apg_menu.rs`、`apg_menubar.rs`、`ui_command_palette.rs`、`ui_combobox.rs`、`ui_disclosure_surfaces.rs` が通ること。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction
  cargo check -p kivact-interaction -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  KIVACT_TEST_MIRROR=off cargo test -p kivact-interaction --lib
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_dialog interaction_menu interaction_select interaction_show_hide interaction_palette apg_menu_button apg_menu apg_menubar ui_command_palette ui_combobox ui_disclosure_surfaces
  ```
- **完了条件**：`rg -n "finish_close|ReturningFocus|Effect::Unmounted|fn modal\(self, modal|\.modal\(kind|settings\.get\(\)\.modal|options\.modal" crates/kivact-interaction/src` が何も返さない。上の命令が通る。

## U07：typeahead の持ち主を list の machine 一つにし、retained focus を構築時の値にする（UI-08、UI-10）
- **wave / lane / 前提**：W3a / U / U06。U02 が `list/state.rs` の `ListSetupOptions { .. }` の literal を新しい関数 `fn default_setup_options` に移し、`active_value(..)` の引数を `index`・`membership.keys()` にし、`list/parts.rs` の `fn item` を作り直している（`active` は memo ではなく closure `active()`）。この unit はその形の上で書く。W5 の前に走るので、名前は HEAD のもの（`Binding<T>`、`IntoBinding`、`into_binding()`、`binding(get, set)`）を使う。
- **所有**：`crates/kivact-interaction/src/list.rs`、`crates/kivact-interaction/src/list/{machine.rs,state.rs,parts.rs,input.rs,handle.rs}`、`crates/kivact-interaction/src/select/{machine.rs,handle.rs}`、`crates/kivact-interaction/src/menu/{machine.rs,session.rs,level.rs}`、`dev/tests/kivact-public-api/tests/e2e_ui/interaction_select.rs`、`dev/tests/kivact-public-api/tests/e2e_ui/interaction_menu.rs`（W1 の T15 が試験を `tests/e2e/` から `tests/e2e_ui/` に移している）
- **目的**：UI-08：typeahead の prefix は list の machine だけが持つ（`Event::Typeahead`／`Event::Expire` は list の machine にだけ残る）。select と menu の machine から `Typeahead` field と `Expire` を消し、`State.typed: Cell<Binding<..>>` と `shared_typeahead` を消す。select の開いた list は閉じた trigger の list の prefix に打ち込む（`typeahead_from`、setup の設定）。消去は `pub(crate) ListApi::clear_typeahead()`。UI-10：`machine::Focus`、`Event::MountFocus`、`retain_focus()` を消し、retained focus を構築時の `Value<bool>` にする。`ListOptions` は `#[non_exhaustive]` の無い公開 struct で、field を足すと公開 API が変わるので足さない。retained focus は pub(crate) の `use_list_retaining(.., retained: Value<bool>)` の構築時の引数にし、select は `keep_trigger_focus && !sheet` の memo を渡す。W7b 以降もこの形を保つ。
- **手順**：
  1. `list/machine.rs`：`enum Focus`（`Physical`、`Retained`）を消す。`State<K>` から `focus: Focus,` を消す（残りは `pending`、`typed`）。`Event` から `MountFocus(Focus),` を消し、`transition` の `Event::MountFocus(focus) => next.focus = focus,` を消す。
  2. `list/state.rs`：
     - `ListSetupOptions` から `pub(super) retained_focus: Value<bool>,` を消し、末尾に足す：
       ```rust
       /// The list whose type-to-find prefix this one types into (a select's
       /// open list types into its closed trigger's); `None` is its own.
       pub(super) typeahead: Option<MachineHandle<machine::Machine<K>>>,
       ```
     - `State` の `pub(super) typed: Cell<Binding<Typeahead>>,` を次にする：
       ```rust
       /// Whether the items leave the focus on another node (a select's
       /// trigger keeps it); fixed at construction, reactive in its value.
       pub(super) retained_focus: Value<bool>,
       ```
     - `create` の署名を次にする（引数 `retained_focus` を末尾に足す）：
       ```rust
       pub(super) fn create<K: kivact_core::state::ItemKey>(
           items: Value<Vec<ListItem<K>>>,
           selection: Selection<K>,
           policy: ListOptions,
           retained_focus: Value<bool>,
       ) -> Owned<Rc<State<K>>> {
       ```
     - `fn default_setup_options`（U02 が作った）の中の `ListSetupOptions { .. }` の literal から `retained_focus: false.into_value(),` を消し、literal の末尾に `typeahead: None,` を足す。
     - `create` の中：machine の初期値の literal から `focus: machine::Focus::Physical,` の行を消す（残りは `pending: machine::Pending::Nothing,` と `typed: Typeahead::Idle,`）。コメント `// The type-to-find buffer is the machine's: ...` の 2 行と、その下の `let typed = binding(..);`（4 行）を消す。`active_value(..)` の呼び出しの引数 `pending,` を `retained_focus,` にする。`Rc::new(State { .. })` の literal の `typed: Cell::new(typed),` を `retained_focus,` にする。
     - `impl State`：`retains_focus` の本体を `self.retained_focus.get_untracked()` にし、その次に足す：
       ```rust
       /// The machine that holds the type-to-find prefix this list types
       /// into: its own, or the one given to `ListApi::typeahead_from`.
       pub(super) fn typeahead_owner(&self) -> MachineHandle<machine::Machine<K>> {
           self.options.get().typeahead.unwrap_or(self.pending)
       }
       ```
     - `fn active_value` の引数 `pending: MachineHandle<machine::Machine<K>>,` を `retained: Value<bool>,` にし、`&& pending.state().get().focus == machine::Focus::Physical` を `&& !retained.get()` にする。
     - `fn observe_list` の `if state.pending.state().get().focus == machine::Focus::Physical {` を `if !state.retained_focus.get() {` にする。
     - import：`use std::cell::{Cell, RefCell};` → `use std::cell::RefCell;`。`use kivact_core::state::{Binding, Callback, Signal, Value, binding, effect, memo, signal};` から `binding` を消す（`Binding` は `ListSetupOptions::anchor` が使うので残す）。
  3. `list/parts.rs`：`fn item`（U02 の形）の `Part::new` の閉包の中の `let focus_mode = state.pending;` を `let retained = state.retained_focus;` にし、`.focusable(move || { focus_mode.state().get().focus == machine::Focus::Physical && active() })` を `.focusable(move || !retained.get() && active())` にする。`fn mount_item` の `let retained_focus = state.options.get().retained_focus.get_untracked();` を `let retained_focus = state.retained_focus.get_untracked();` にし、その下の `let focus = if retained_focus { machine::Focus::Retained } else { machine::Focus::Physical };` と `state.pending.send(machine::Event::MountFocus(focus));` を消す（その下の `eligible: if retained_focus { .. }` は変えない）。`use super::machine;` を消す。
  4. `list/input.rs` の `fn typed` の先頭 4 行（`let slot = state.typed.get();` から `slot.set(typed);` まで）を次にする。import に `use super::machine;` を足す：
     ```rust
     let owner = state.typeahead_owner();
     let mut typed = owner.state().with_untracked(|current| current.typed.clone());
     let query = typed.push(c);
     owner.send(machine::Event::Typeahead(typed));
     ```
  5. `list/handle.rs`：
     - `use super::{input, parts, state, typeahead};` に `machine` を足す。
     - `use_list` の `state::create(items.into_value(), selection, policy)` を `state::create(items.into_value(), selection, policy, false.into_value())` にする。`use_list` の次に足す：
       ```rust
       /// A list whose items leave the focus on another node while `retained`
       /// is true: a select's trigger keeps the focus and shows the active item
       /// as its `active_descendant`. Setup only.
       #[track_caller]
       pub(crate) fn use_list_retaining<K: kivact_core::state::ItemKey>(
           items: Value<Vec<ListItem<K>>>,
           selection: Selection<K>,
           policy: ListOptions,
           retained: Value<bool>,
       ) -> ListApi<K> {
           check_list_axis(policy.axis, "use_list");
           ListApi {
               state: state::create(items, selection, policy, retained),
           }
       }
       ```
     - `pub(crate) fn retain_focus`（関数全体）を消す。`reset_highlight` の `state.typed.get().set(typeahead::Typeahead::default());` を `state.typeahead_owner().send(machine::Event::Typeahead(typeahead::Typeahead::Idle));` にする。
     - `pub(crate) fn shared_typeahead` を消し、同じ場所に足す：
       ```rust
       /// Type into `owner`'s type-to-find prefix instead of one of this
       /// list's own: a select's open list continues the search begun at its
       /// closed trigger. Setup only.
       #[track_caller]
       pub(crate) fn typeahead_from(self, owner: ListApi<K>) -> Self {
           require_setup("ListApi::typeahead_from");
           let owner = owner.state().pending;
           self.state().options.update(|options| options.typeahead = Some(owner));
           self
       }
       /// End the type-to-find prefix this list types into. Nothing happens
       /// once the list is gone.
       pub(crate) fn clear_typeahead(self) {
           if let Some(state) = self.state.try_get() {
               state
                   .typeahead_owner()
                   .send(machine::Event::Typeahead(typeahead::Typeahead::Idle));
           }
       }
       ```
  6. `list.rs` の `pub use handle::{ListApi, use_list};` の次に `pub(crate) use handle::use_list_retaining;` を足す。
  7. `select/machine.rs`：`use crate::list::typeahead::{TYPEAHEAD_WINDOW, Typeahead};` を消す。`State::Closed { typeahead: Typeahead }` を unit variant `Closed` にし、`Open` から `typeahead: Typeahead,` を消す。`impl<K> State<K> { fn typeahead .. }` を消す。`Event` から `Typeahead(Typeahead)` と `Expire`（doc を含む）を消す。`fn sheet_state<K: Clone>(state: &State<K>) -> State<K>` を `fn sheet_state<K>() -> State<K>` にし `typeahead: ..` の行を消す（呼び出しは `sheet_state()`）。`Event::Open` の腕の条件を `matches!(state, State::Closed)` にし literal の `typeahead:` 行を消す。`Event::OpenSheet` の腕の `next = sheet_state(state);` を `next = sheet_state();` にする。`Event::Close` の腕の `next = State::Closed { typeahead: .. };` を `next = State::Closed;` にし、`State::Closed { .. } => false` を `State::Closed => false` にする（`Event::Preview` の腕も同じ）。`Event::Typeahead` と `Event::Expire` の腕を消す。手順 8 の後に `rg -n "Closed \{" crates/kivact-interaction/src/select` が何も返さないことを確かめる。
  8. `select/handle.rs`：
     - module doc の段落 "The closed trigger and the open list are two [`ListApi`]s over the same options with the listbox policy, sharing one private typeahead slot for the select's lifetime, so a letter typed at the trigger and one typed in the list continue the same search." のうち "sharing one private typeahead slot for the select's lifetime," を "and the open list types into the closed trigger's type-to-find prefix," にする（文の残りはそのまま。`//!` の行の折り返しは 100 桁に収まるように直す）。`use crate::list::typeahead::Typeahead;` を消し `use crate::list::use_list_retaining;` を足す。
     - `use_select`：`machine(` の初期値 `machine::State::Closed { typeahead: Typeahead::default(), }` を `machine::State::Closed` にする。`let typed = binding(..);`（3 行）を消す。`SelectApi { .. }` の literal の中の `settings: OptionsCell::new(SelectOptions { .. }),` を literal の前（`let hovered = binding(..);` の後）の `let settings = OptionsCell::new(SelectOptions { arrow_open: false.into_value(), keep_trigger_focus: true.into_value() });` に移し、続けて：
       ```rust
       // The open list leaves the focus on the trigger while the trigger
       // keeps it, except in the sheet, whose options take the focus.
       let retained = memo(move || {
           settings.value().with(|options| options.keep_trigger_focus.get())
               && !session.state().with(|state| {
                   matches!(state, machine::State::Open { surface: machine::Surface::Sheet, .. })
               })
       });
       // The closed trigger moves the selection through the logical order,
       // including options that are not mounted; the open list continues
       // its type-to-find prefix.
       let closed = group(options, selected);
       let list = use_list_retaining(
           options,
           Selection::Single(selected),
           ListOptions::listbox(),
           retained,
       )
       .typeahead_from(closed)
       .shared_highlight(highlight);
       ```
       literal は `closed,`、`settings,`、`list,` にする（元のコメント "The closed trigger moves the selection ..." は literal から消す）。
     - `pub fn keep_trigger_focus` の `self.list.retain_focus(enabled);` を消す。`fn content_in` の `list.retain_focus(keep_trigger_focus);` を消す（同じ関数の `keep_trigger_focus` の local はそのまま使う）。
  9. `menu/machine.rs`：`use crate::list::typeahead::Typeahead;` を消す。`State::Open` から `typeahead: Typeahead,` を消す。`Event` から `Typeahead(Typeahead)` と `Expire`（doc を含む）を消す。`transition`：Open の literal の `typeahead: Typeahead::default(),` を消し、`let State::Open { levels, want, typeahead, .. } = &mut next` を `let State::Open { levels, want, .. } = &mut next` にし、`let typing = ..;` の行と `if typing { step = step.after(..) }` を消し、`update_level_event(levels, want, typeahead, event)` を `update_level_event(levels, want, event)` にする。`fn update_level_event` の引数 `typeahead: &mut Typeahead,` と腕 `Event::Typeahead(typed) => ..`、`Event::Expire => ..` を消す。
  10. `menu/session.rs`：`use crate::list::typeahead::Typeahead;`、field `pub(super) typed: kivact_core::state::Binding<Typeahead>,`、`let typed = typeahead_binding(machine);`、literal の `typed,`、`fn typeahead_binding` を消す。
  11. `menu/level.rs`：`Level::new` の `.shared_typeahead(session.typed)` の行を消す（各 level の list が自分の prefix を持つ）。`fn mount_content` の `if depth == 0 { .. }` の塊の最後に足す：
      ```rust
      // The top level outlives its surface: closing the menu ends its search.
      on_cleanup(move || group.clear_typeahead());
      ```
  12. 試験を足す（どちらも今のコードでも通る。変更の前後で同じ振る舞いを固定する）。`interaction_select.rs` の末尾：
      ```rust
      #[test]
      fn letters_at_the_trigger_and_in_the_open_list_continue_one_search() {
          let (screen, selected) = mount_select(fruit(), None);
          let trigger = trigger(&screen);
          trigger.press_key(Key::Char('a'));
          assert_eq!(selected.get_untracked(), Some("apple"), "a letter at the closed trigger selects the first match");
          trigger.press_key_with(Key::ArrowDown, alt());
          screen.press_key(Key::Char('p'));
          screen.press_key(Key::Char('r'));
          screen.press_key(Key::Enter);
          assert_eq!(selected.get_untracked(), Some("apricot"), "\"apr\" continues the search begun at the trigger");
          assert!(closed(&screen));
          assert!(screen.errors().is_empty(), "{:?}", screen.errors());
      }
      ```
      `interaction_menu.rs` の末尾：
      ```rust
      #[test]
      fn a_letter_moves_to_the_next_item_starting_with_it_and_closing_ends_the_search() {
          let (screen, _) = mount_button(edit_entries());
          let trigger = screen.get_by_role(Role::Button, "Edit");
          trigger.press_key(Key::ArrowDown);
          assert!(item(&screen, "Copy").is_focused());
          screen.press_key(Key::Char('s'));
          assert!(item(&screen, "Select all").is_focused(), "s: the next enabled item starting with it");
          screen.press_key(Key::Escape);
          trigger.press_key(Key::ArrowDown);
          assert!(item(&screen, "Copy").is_focused());
          screen.press_key(Key::Char('p'));
          assert!(item(&screen, "Paste").is_focused(), "after reopening a letter starts a new search, not \"sp\"");
          assert!(screen.errors().is_empty(), "{:?}", screen.errors());
      }
      ```
  13. 振る舞いの差：menu の submenu は自分の prefix を持つ（以前は session に一つで、submenu へ移っても 500 ms 内は続いた。APG の「今の menu の中で探す」に合う）。
- **消すもの**：`list::machine::Focus`、`list::machine::Event::MountFocus`、`list::State::typed`、`ListSetupOptions::retained_focus`、`ListApi::retain_focus`、`ListApi::shared_typeahead`、select の `State::{Closed,Open}::typeahead`、`State::typeahead()`、`Event::Typeahead`、`Event::Expire`、select の `typed` binding、menu の `State::Open::typeahead`、`Event::Typeahead`、`Event::Expire`、`Session::typed`、`fn typeahead_binding`。
- **状態の閉じ方**：`list::machine::State<K>` は `pending` と `typed`（bool/Option/counter なし）。`list::state::State<K>` と select・menu の `State` は接尾辞 `State`。`ListSetupOptions` は接尾辞 `Options`（`typeahead: Option<..>` は setup で一度だけ書く設定）。`retained_focus: Value<bool>` は constructor だけで書く。
- **試験**：足す：上の 2 つ。通ること：`interaction_select.rs`、`interaction_menu.rs`、`interaction_list_nav.rs`、`apg_listbox.rs`、`apg_menu.rs`、`apg_menu_button.rs`、`apg_menubar.rs`、`interaction_tabs.rs`、`interaction_tree.rs`、`interaction_table.rs`、`ui_combobox.rs`（`select_touch_opens_the_sheet`）、`interaction_show_hide.rs`。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction -p kivact-public-api-tests
  cargo check -p kivact-interaction -p kivact-ui -p kivact-public-api-tests --all-targets
  cargo clippy -p kivact-interaction --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo check -p kivact-ui-creative -p kivact-ui-basic -p kivact-ui-soft -p kivact-ui-pop -p kivact-ui-game --all-targets
  KIVACT_TEST_MIRROR=off cargo test -p kivact-interaction --lib
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_select interaction_menu interaction_list_nav apg_listbox apg_menu apg_menu_button apg_menubar interaction_tabs interaction_tree interaction_table ui_combobox interaction_show_hide
  ```
- **完了条件**：`rg -n "shared_typeahead|MountFocus|machine::Focus|fn retain_focus|retain_focus\(|Cell<Binding" crates/kivact-interaction/src` と `rg -n "Typeahead|Expire|TYPEAHEAD_WINDOW" crates/kivact-interaction/src/select crates/kivact-interaction/src/menu` が何も返さない。足した 2 つの試験と上の命令が通る。

## U08：単位状態の machine を消し、effect の送信を feed にする（UI-09、UI-25 の toaster）
- **wave / lane / 前提**：W3a / U / U07。この unit は W5 の前に走るので、名前は HEAD のもの（`Binding<T>`、`toaster()`、`effect`、`effect_on`）を使う。
- **所有**：`crates/kivact-interaction/src/scalar.rs`、`crates/kivact-interaction/src/scalar/{edit_state.rs,range.rs}`（消す：`scalar/focus_machine.rs`、`scalar/range_machine.rs`）、`crates/kivact-interaction/src/delayed/api.rs`、`crates/kivact-interaction/src/toaster/api.rs`、`crates/kivact-ui/src/calendar/{view.rs,navigation.rs}`、`crates/kivact-ui/src/date_input/view.rs`、`crates/kivact-ui/src/date_range_input.rs`、`crates/kivact-ui/src/time_input/view.rs`、`dev/tests/kivact-public-api/tests/e2e_ui/ui_calendar.rs`（W1 の T15 が試験を `tests/e2e_ui/` に移している）
- **目的**：UI-09：`type State = ()` の machine 2 つを inline にする。calendar の月の読み上げを machine でなく `memo` にする。`effect(move || m.send(E(v.get())))` を `m.feed(v, E)` にする（feed は初期値を送らないので、初期値は setup で一度 `send` する。元の effect の初回実行と同じ）。UI-25：toaster の `visible` memo が column の状態全体を clone しない。
- **手順**：
  1. `scalar.rs` の `mod focus_machine;` と `mod range_machine;` を消し、2 つのファイルを消す（`git rm`）。
  2. `scalar/edit_state.rs`：`use super::focus_machine::{Effect, Event, Machine};` と `use kivact_core::state::{MachineHandle, machine};` を消し、`use kivact_core::state::{Signal, Value, effect, memo, on_cleanup, signal};` に `effect_on` を足す。`Shared` から `focus: MachineHandle<Machine>,` を消し、`Shared::new` の `let focus = machine(..);` と literal の `focus,` を消す。`impl Shared` に足す：
     ```rust
     /// Cancel the drag and finish a touch gesture with a restore once no
     /// thumb has the focus.
     fn thumb_blurred(self) {
         let any_focused = self
             .thumb_focus
             .with_untracked(|thumbs| thumbs.iter().any(|thumb| thumb.get_untracked()));
         if !any_focused {
             self.drag.cancel();
             if let Some(touch) = self.settings.get().touch {
                 touch.finish(true);
             }
         }
     }
     ```
     `register_focus` の `shared.focus.feed(focused, |focused| { .. });` を次にし、on_cleanup の中の `shared.focus.send(Event::ThumbBlur);` を `shared.thumb_blurred();` にする：
     ```rust
     effect_on(focused, move |focused: &bool| {
         if !*focused {
             shared.thumb_blurred();
         }
     });
     ```
     `pub(crate) struct Shared` の印を次の 1 行だけにする：`Shared` の doc（`/// The edit plumbing one ...`）の 1 行目より上にある `// xtask-allow(state):` で始まる行（W1 の T06 が付けた行）をすべて消し、doc の 1 行目の直前に次の 1 行を置く（字下げなし）：
     ```rust
     // xtask-allow(state): thumb_focus and host_disabled are registries of the mounted thumbs' values, one entry per mount; host_disabled goes in U42 (UI-11)
     ```
  3. `scalar/range.rs`：`use super::range_machine::{..};` を消し、`use kivact_core::state::{Binding, Value, machine};` を `use kivact_core::state::{Binding, Value, effect_on};` にする。`follow_range` と `follow_ranges` を次にする：
     ```rust
     /// The value `now` takes in `range` when the range leaves it out: a finite
     /// value outside the ends, normalized. Nothing for a value inside or not finite.
     fn outside(range: ScalarRange, now: f64) -> Option<f64> {
         if now.is_finite() && (now < range.min() || now > range.max()) {
             range.normalize(now)
         } else {
             None
         }
     }
     /// Write the value back when a range change leaves it out.
     #[track_caller]
     pub(crate) fn follow_range(binding: Binding<f64>, range: Value<ScalarRange>) {
         effect_on(range, move |range: &ScalarRange| {
             if let Some(value) = outside(*range, binding.get_untracked()) {
                 binding.set(value);
             }
         });
     }
     /// Normalize both ends from the same snapshot, writing low before high.
     #[track_caller]
     pub(crate) fn follow_ranges(low: Binding<f64>, high: Binding<f64>, range: Value<ScalarRange>) {
         effect_on(range, move |range: &ScalarRange| {
             let from = outside(*range, low.get_untracked());
             let to = outside(*range, high.get_untracked());
             if let Some(from) = from {
                 low.set(from);
             }
             if let Some(to) = to {
                 high.set(to);
             }
         });
     }
     ```
  4. `delayed/api.rs` の `fn delayed`：`effect(move || state.send(Event::Wanted(wanted.get())));` を次にし、import の `effect` を消す：
     ```rust
     state.send(Event::Wanted(wanted.get_untracked()));
     state.feed(wanted, Event::Wanted);
     ```
  5. `toaster/api.rs`：`hold_while` の `effect(move || self.column.send(ColumnEvent::Hold(id, held.get())));` を次にする：
     ```rust
     self.column.send(ColumnEvent::Hold(id, held.get_untracked()));
     self.column.feed(held, move |held| ColumnEvent::Hold(id, held));
     ```
     `toaster()` の `visible: memo(move || { let state = column.get(); state.queue.iter().take(state.limit).map(..).collect() })` を次にする（column の状態全体を clone しない）：
     ```rust
     visible: memo(move || {
         column.state().with(|state| {
             state
                 .queue
                 .iter()
                 .take(state.limit)
                 .map(|entry| (entry.id, entry.toast.clone()))
                 .collect()
         })
     }),
     ```
  6. ui の 3 か所（どれも元の effect の初回が初期値を送っていた。date と range の初回は `Effect::Validity` を出すので setup の `send` が要る）：
     - `date_input/view.rs`：`kivact_core::state::effect(move || state.send(Event::External(value.get())));` → `state.send(Event::External(value.get_untracked())); state.feed(value, Event::External);`
     - `date_range_input.rs`：`kivact_core::state::effect(move || draft.send(DraftEvent::External(value.get())));` → `draft.send(DraftEvent::External(value.get_untracked())); draft.feed(value, DraftEvent::External);`
     - `time_input/view.rs`：`kivact_core::state::effect(move || state.send(Event::External(value.get())));` → `state.send(Event::External(value.get_untracked())); state.feed(value, Event::External);`
  7. `calendar/navigation.rs`：`pub(super) enum MonthEvent`、`pub(super) struct MonthSnapshot`、`impl Machine for MonthSnapshot` を消し、`use kivact_core::i18n::{CalendarMonth, Locale, NameWidth};` の行を消す（この 3 つを使うのはこれらだけ）。
  8. `calendar/view.rs`：`fn month_navigation` を消し、同じ場所に置く：
     ```rust
     /// The shown month's announcement: its long name and year in the locale.
     fn month_announcement(month: Binding<CalendarMonth>) -> Value<String> {
         let locale = kivact_core::i18n::use_locale();
         kivact_core::state::memo(move || {
             let month = month.get();
             locale.with(|locale| {
                 format!(
                     "{} {}",
                     locale.month_name(month.month(), kivact_core::i18n::NameWidth::Long),
                     month.year()
                 )
             })
         })
     }
     ```
     `into_view` の `let (month, announcement) = month_navigation(base_month);` を `let announcement = month_announcement(base_month);` にする（`month` は同じ binding のまま。`swipe_navigation(base_month, month, ..)` は変えない）。
  9. 残る effect の送信（`effect(move || m.send(E { 複数の源 }))`）は複数の値を一つの event にまとめるので変えない：`kivact-ui/src/alert.rs`（`BannerEvent::Observed`）、`kivact-interaction/src/hover_corridor/api.rs`（`Event::Sample`）、`kivact-interaction/src/menu/level.rs`（`Event::Pointer`）、`kivact-interaction/src/toaster/api.rs` の `track_toast`（`TimerEvent::Observe`）、`kivact-ui/src/empty_state_changes.rs`（`Event::Observe(observation())`）。feed は一つの値しか受けず、memo を挟むと memo が一つ増え、等値の間引きで送る回数が変わる。設計 6.3 節が禁じるのは signal を書く effect で、これらは machine に送るだけなので残す。
  10. 試験を足す。`ui_calendar.rs` の末尾：
      ```rust
      #[test]
      fn the_status_names_the_shown_month_and_follows_navigation() {
          let (screen, month) = mount(|| {
              let month = signal(CalendarMonth::new(2026, 10));
              (month, ui::calendar(signal(Some(day(14)))).month(month))
          });
          assert!(screen.query_by_role(Role::Status, "Showing October 2026").is_some());
          screen.get_by_role(Role::Button, "Next month").click();
          assert_eq!(month.get_untracked(), CalendarMonth::new(2026, 11));
          assert!(screen.query_by_role(Role::Status, "Showing November 2026").is_some());
          month.set(CalendarMonth::new(2027, 1));
          screen.run_until_idle();
          assert!(screen.query_by_role(Role::Status, "Showing January 2027").is_some());
          assert!(screen.errors().is_empty(), "{:?}", screen.errors());
      }
      ```
- **消すもの**：`scalar/focus_machine.rs`、`scalar/range_machine.rs`、`Shared::focus`、`MonthEvent`、`MonthSnapshot`、`month_navigation`、上の 5 つの effect。
- **状態の閉じ方**：`Shared` は上の印の 1 行（registry。field を 1 つ減らすだけ。U42 が `host_disabled` を消すとき、この印も直す）。新しい状態の型は無い。
- **試験**：足す：`ui_calendar.rs::the_status_names_the_shown_month_and_follows_navigation`。通ること：`interaction_slider.rs`（`escape_a_canceled_pointer_and_the_focus_leaving_restore_the_value` を含む）、`apg_slider.rs`、`ui_controls.rs`、`interaction_delayed.rs`、`interaction_tooltip.rs`、`apg_tooltip.rs`、`ui_disclosure_surfaces.rs`、`interaction_toast.rs`、`ui_calendar.rs`、`interaction_calendar.rs`。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction -p kivact-ui -p kivact-public-api-tests
  cargo check -p kivact-interaction -p kivact-ui -p kivact-public-api-tests --all-targets
  cargo clippy -p kivact-interaction -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo xtask check --no-write --strict --only unit-machine --crate kivact-interaction
  cargo xtask check --no-write --crate kivact-ui
  KIVACT_TEST_MIRROR=off cargo test -p kivact-interaction --lib
  KIVACT_TEST_MIRROR=off cargo test -p kivact-ui --lib
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_slider apg_slider ui_controls interaction_delayed interaction_tooltip apg_tooltip ui_disclosure_surfaces interaction_toast ui_calendar interaction_calendar
  ```
  （2 行目：annex T 4.4 節の `[rework.pending]` で `unit-machine` を pending にしている `kivact-interaction` を、この unit が通す。`ok` が出ること。）
- **完了条件**：`rg -n "type State = \(\)" crates/` と `rg -n -U "machine\(\s*\(\)," crates/` と `rg -n "focus_machine|range_machine|MonthEvent|MonthSnapshot|month_navigation|let state = column.get\(\)" crates/` が何も返さない。`rg -n "effect\(move \|\| [a-z_.]+\.send\(" crates/kivact-interaction/src crates/kivact-ui/src` が `empty_state_changes.rs` の 1 行だけを返す。上の命令が通る。

## U09：carousel の feed、部品の文字列、focusable_when_disabled、sparkline の性能（UI-07、UI-16、UI-18、UI-21）
- **wave / lane / 前提**：W4 / U / U08。U03 が `menu/level.rs` の `pub(super) fn item` を作り直している（本文の先頭が `let Level { session, rows, index, open_key, group, retain_disabled_focus, .. } = self;` で、`group.base_item(key.clone(), retain_disabled_focus)` と `item_description(e, disabled, retain_disabled_focus)` がその local を読む）。この unit は W5 の前に走るので、名前は HEAD のもの（`Binding<T>`、`IntoBinding`、`use_theme() -> Value<Theme>`、`UiTokens`）を使う。
- **所有**：`crates/kivact-ui/src/{carousel.rs,async_view.rs,button.rs,sparkline.rs,sparkline_tokens.rs}`、`crates/kivact-ui/src/combobox/{render.rs,handle.rs}`、`crates/kivact-interaction/src/menu/level.rs`、`dev/tests/kivact-public-api/tests/e2e_core/i18n.rs`（W1 の T15 が `i18n.rs` を `tests/e2e_core/` に移している）
- **目的**：UI-07（公開 API を変えない範囲）：`follow_scroll` を scroll offset の feed（`effect_on`）にし `let _ = offset;` を消す。`follow_index` は signal を書かない（`scroll_to` だけの）唯一の effect として残す。この unit は carousel の machine を変えない：`kivact-interaction::carousel` の machine に `Event::ScrollObserved`／`GoTo` と `Effect::ScrollTo` を置くには、kivact-ui から呼ぶ新しい公開 method が `CarouselApi` に要る（kivact-ui から `pub(crate)` は見えない）ので、W8 の U51（carousel を持つ lane U3 の unit）が行う。UI-16：英語の固定文字列を `message` にする（監査の 4 つに加え、carousel の "Next slide"、"Previous slide"、"Slide {} / {}" も同じ所見なので `carousel.next`、`carousel.previous`、`carousel.slide` にする。既定の英語は同じ）。UI-18：button と menu level の `focusable_when_disabled` を theme に追従させる。UI-21（性能）：Vec の clone と Summary の 3 回計算をやめ、paint ごとの tokens の clone と `Vec<Vec<Point>>` を無くす（recipe の状態は `()` のまま）。
- **手順**：
  1. `carousel.rs` の `fn follow_scroll` を次にする。viewport と slide の rect は setup で取る：`NodeRef::rect()` は呼ばれたときに host へ layout の報告を頼むので、`effect_on` の callback（最初の値では走らない）の中で初めて呼ぶと、最初の scroll のときにはまだ `None` で、index が動かない（W4 の差戻し。元の `effect` は mount 直後の初回の実行で rect を頼んでいた）：
     ```rust
     /// Report the slide nearest the viewport's start to the carousel each time
     /// the native scroll offset changes. The viewport's and the slides' rects are
     /// taken here, at setup: taking a rect asks the host for the node's layout,
     /// and the first scroll must find it reported.
     fn follow_scroll(c: Layout, navigation_refs: std::rc::Rc<Vec<kivact_core::view::NodeRef>>) {
         let viewport = c.viewport.rect();
         let slides = slide_rects(&navigation_refs);
         kivact_core::state::effect_on(c.viewport.scroll_offset(), move |_: &Point| {
             let Some(view) = viewport.get_untracked() else {
                 return;
             };
             let nearest = nearest_slide(
                 &slides,
                 view,
                 c.vertical.get_untracked(),
                 c.rtl.get_untracked() == ReadingDirection::Rtl,
             );
             if let Some(at) = nearest {
                 c.api.scrolling(at);
             }
         });
     }
     ```
     `nearest_slide` の最初の引数を `slides: &[Value<Option<kivact_core::geometry::Rect>>]` にし、本体の `references.iter()` を `slides.iter()`、`r.rect().get_untracked()` を `r.get_untracked()` にする。その直前に次を足す：
     ```rust
     /// The layout rects of the slides behind `references`.
     fn slide_rects(
         references: &[kivact_core::view::NodeRef],
     ) -> Vec<Value<Option<kivact_core::geometry::Rect>>> {
         references.iter().map(kivact_core::view::NodeRef::rect).collect()
     }
     ```
     `follow_index` の中の `nearest_slide(&navigation_refs, ..)` は `nearest_slide(&slide_rects(&navigation_refs), ..)` にする。
  2. 文字列（`message` は `kivact_core::i18n::message`、`message_with` も同じ module）：
     - `combobox/render.rs` の `editor_actions`：`button("Clear")` → `button(kivact_core::i18n::message("combobox.clear", "Clear"))`、`button("Toggle options")` → `button(kivact_core::i18n::message("combobox.toggle", "Toggle options"))`。
     - `async_view.rs` の `default_fallback`：`super::button("Retry")` → `super::button(kivact_core::i18n::message("async_view.retry", "Retry"))`。
     - `carousel.rs`：`carousel()` の `root: node().name("Carousel"),` → `root: node().name(kivact_core::i18n::message("carousel.label", "Carousel")),`。`navigation_button` の `crate::button(if next { "Next slide" } else { "Previous slide" })` → `crate::button(if next { kivact_core::i18n::message("carousel.next", "Next slide") } else { kivact_core::i18n::message("carousel.previous", "Previous slide") })`。`slide_items` の `.name(format!("Slide {} / {}", at + 1, c.count))` → 
       ```rust
       .name({
           let count = c.count;
           kivact_core::i18n::message_with("carousel.slide", "Slide {index} / {count}", move || {
               [("index", (at + 1).to_string()), ("count", count.to_string())]
           })
       })
       ```
     - rustdoc に key を書く：`pub fn carousel` の doc の末尾に `/// Its name is the message \`carousel.label\` ("Carousel"), its buttons \`carousel.previous\` and \`carousel.next\`, and each slide \`carousel.slide\` ("Slide {index} / {count}").`、`combobox/handle.rs` の `pub fn combobox` の doc の例（```）の前に段落 `/// The clear and toggle buttons are named by the messages \`combobox.clear\` ("Clear") and \`combobox.toggle\` ("Toggle options").`、`async_view.rs` の `/// Display an async source. Failures include a retry button by default.` の後に `/// The button is named by the message \`async_view.retry\` ("Retry").`
     - catalog には項目を足さない（message の catalog は `examples/kivact-ui-pop/src/messages.rs` と `examples/kivact-ui-game/src/messages.rs` だけで、どちらも自分が読む key だけを並べ、combobox・carousel・async_view を使わない）。docs の key の表は付属書 G が書く。
  3. `button.rs` の `pub fn button`：`.focusable_when_disabled(matches!(use_theme().get().ext::<..>().button.disabled_focus, DisabledButtonFocus::Retain))` を次にする（`let theme = use_theme();` を `let root` の前に置く）：
     ```rust
     .focusable_when_disabled(move || {
         theme.with(|theme| {
             matches!(
                 theme.ext::<kivact_interaction::InteractionPolicy>().button.disabled_focus,
                 DisabledButtonFocus::Retain
             )
         })
     })
     ```
  4. `menu/level.rs`：`struct Level` の field `retain_disabled_focus: bool,` を `retain_disabled_focus: Value<bool>,` にし、その直上に doc `/// The theme's disabled-focus policy; rows built at an opening read it then.` を付ける（W1 の T06 がこの field の上の `xtask-allow(state-fields)` の行を消しているので、field の上に印の行は無い）。`Level::new` の `let retain_disabled_focus = use_theme().get()..== DisabledMenuFocus::Retain;` を次にする：
     ```rust
     let theme = use_theme();
     let retain_disabled_focus = memo(move || {
         theme.with(|theme| {
             theme.ext::<crate::InteractionPolicy>().menu.disabled_focus == DisabledMenuFocus::Retain
         })
     });
     ```
     使う所（`Self { .. }` の literal の `retain_disabled_focus,` は変えない）：
     - `Level::new` の items の memo の `disabled: entry.is_disabled() && !retain_disabled_focus,` → `disabled: entry.is_disabled() && !retain_disabled_focus.get(),`。
     - `content` の on_key の `if retain_disabled_focus` → `if retain_disabled_focus.get_untracked()`（`content` の先頭の `let Level { .., retain_disabled_focus, .. } = self;` は変えない）。
     - `mount_content` の `edge_item(rows, edge, retain_disabled_focus)` と `edge_item(rows, Edge::First, retain_disabled_focus)` → 第 3 引数を `retain_disabled_focus.get_untracked()` にする。
     - `pub(super) fn item`（U03 の形）：先頭の `let Level { .. } = self;` の直後に `let retain_disabled_focus = retain_disabled_focus.get_untracked();` を 1 行足す（その下の `group.base_item(key.clone(), retain_disabled_focus)` と `item_description(e, disabled, retain_disabled_focus)` は `bool` を受けるので変えない）。
     行は開くたびに作られるので、theme の切替は次に開いたときに効く（`kivact-ui/src/menu.rs` の `menu_rows` は `while_open` の中で読むので既に開くたび）。
  5. `sparkline_tokens.rs`：`SparklineTokens` の derive を `#[derive(Clone, Copy, Debug, PartialEq)]` にする（`SparklineFill` は既に `Copy`）。
  6. `sparkline.rs`：
     - `use crate::sparkline_tokens::SparklineFill;` を `use crate::sparkline_tokens::{SparklineFill, SparklineTokens};` にする。
     - `struct Sparkline` の `values: Value<Vec<f64>>,` を `summary: Value<Summary>,` にする。`struct Summary` に `#[derive(Clone, Copy, PartialEq)]` を付ける。
     - `pub fn sparkline` の本体：
       ```rust
       let values: Value<Vec<f64>> = values.into_value();
       // The one memo: the summary. The line borrows the values where it is
       // drawn, so no change copies them.
       let summary = kivact_core::state::memo(move || values.with(|values| Summary::of(values)));
       Sparkline {
           root: node()
               .hidden(move || summary.get().count < 2)
               .draw(move |d, s| {
                   values.with(|values| paint(d, s.width, s.height, values, summary.get()))
               }),
           summary,
           label: None,
           variants: Variants::default(),
       }
       ```
     - `into_view` の分解を `summary` にし、`.description(description(values))` を `.description(description(summary))` にする。`fn description(summary: Value<Summary>)` にし、本体の `let summary = Summary::of(&values.get());` を `let summary = summary.get();` にする。
     - `fn paint` と `fn runs` を消し、次の 2 つを置く：
       ```rust
       /// Draw the line and the area of each run of finite values in a box,
       /// straight into the paths. Value `i` sits at `width * i / (n - 1)`; the
       /// highest value is at `inset` from the top and the lowest at `inset` from
       /// the bottom; a flat series is drawn in the middle. A value that is not
       /// finite ends the run before it.
       fn paint(d: &mut Draw, width: f64, height: f64, values: &[f64], summary: Summary) {
           if summary.count < 2 {
               return;
           }
           let tokens = d.theme().ext::<UiTokens>().sparkline;
           let span = summary.max - summary.min;
           let last_index = (values.len().max(2) - 1) as f64;
           let point = move |i: usize| {
               let t = if span > 0.0 { (values[i] - summary.min) / span } else { 0.5 };
               Point::new(
                   width * i as f64 / last_index,
                   tokens.inset + (1.0 - t) * (height - 2.0 * tokens.inset),
               )
           };
           let mut start = 0;
           while start < values.len() {
               let end = values[start..]
                   .iter()
                   .position(|value| !value.is_finite())
                   .map_or(values.len(), |at| start + at);
               if end - start >= 2 {
                   draw_run(d, (start..end).map(point), height, tokens);
               }
               start = end + 1;
           }
       }
       /// Draw one run of at least two points: the area under it down to the
       /// bottom edge, then its line.
       fn draw_run(
           d: &mut Draw,
           points: impl Iterator<Item = Point> + Clone,
           height: f64,
           tokens: SparklineTokens,
       ) {
           let mut ends = points.clone();
           let Some(first) = ends.next() else {
               return;
           };
           let last = ends.last().unwrap_or(first);
           let line = points.clone().enumerate().fold(Path::new(), |path, (i, p)| {
               if i == 0 { path.move_to(p) } else { path.line_to(p) }
           });
           let area = points
               .fold(Path::new().move_to(Point::new(first.x, height)), |path, p| path.line_to(p))
               .line_to(Point::new(last.x, height))
               .close();
           fill(d, &area, tokens.fill);
           let stroke = Stroke::new(tokens.line_width);
           let stroke = if tokens.round {
               stroke.cap(LineCap::Round).join(LineJoin::Round)
           } else {
               stroke
           };
           d.stroke_path(&line, &stroke, tokens.line);
       }
       ```
     - recipe の状態は `|| ()` のまま。module doc はそのまま。
  7. 試験を足す。`dev/tests/kivact-public-api/tests/e2e_core/i18n.rs` の末尾（import に `use kivact::ui::{carousel, combobox};` を足す）：
     ```rust
     #[test]
     fn standard_components_name_their_controls_through_messages() {
         let screen = render_with(
             RenderOptions::from_env().messages(
                 Messages::new()
                     .with("carousel.label", "スライド")
                     .with("combobox.toggle", "候補を開閉"),
             ),
             || {
                 column((
                     carousel(signal(0usize), (text("One"), text("Two"))),
                     combobox("Fruit", vec![ListItem::new("a", "Apple")], signal(None::<&'static str>)),
                 ))
             },
         );
         assert!(screen.query_by_role(Role::Group, "スライド").is_some());
         assert!(screen.query_by_role(Role::Button, "候補を開閉").is_some());
         assert!(screen.errors().is_empty(), "{:?}", screen.errors());
     }
     ```
- **消すもの**：`follow_scroll` の `effect` と `let _ = offset;`、`sparkline::runs`、`Sparkline::values`、`button` と menu level の一度だけの theme の読み。
- **状態の閉じ方**：新しい状態の型は無い。`Summary` は counter 1 つ。`Level` の `retain_disabled_focus` は bool から `Value<bool>` に変わるだけ（`Level` の既存の印は変えない。`Level` の Signal は part slot と rect の registry で U31 が slot を消す）。
- **試験**：足す：`i18n.rs::standard_components_name_their_controls_through_messages`。通ること：`ui_carousel.rs`、`apg_carousel.rs`、`ui_sparkline.rs`（5 つ全部）、`ui_combobox.rs`、`interaction_menu.rs`、`apg_menu.rs`、`apg_menu_button.rs`、`ui_components.rs`（以上 `e2e_ui`）、`async_tests.rs`（`Retry`）、`i18n.rs`（以上 `e2e_core`）、`ui_standard.rs`（`e2e_kits`）。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction -p kivact-ui -p kivact-public-api-tests
  cargo check -p kivact-interaction -p kivact-ui -p kivact-public-api-tests --all-targets
  cargo clippy -p kivact-interaction -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo xtask check --no-write --crate kivact-ui
  cargo check -p kivact-ui-creative -p kivact-ui-basic -p kivact-ui-soft -p kivact-ui-pop -p kivact-ui-game --all-targets
  KIVACT_TEST_MIRROR=off cargo test -p kivact-ui --lib
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_carousel apg_carousel ui_sparkline ui_combobox interaction_menu apg_menu apg_menu_button ui_components
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- async_tests i18n
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-kit-tests --test e2e_kits -- ui_standard
  ```
  （W1 の T15 の割当て：`async_tests.rs`・`i18n.rs` は `e2e_core`、`ui_standard.rs` は package `kivact-kit-tests` の `e2e_kits`。）
- **完了条件**：`rg -n "let _ = offset|name\(\"Carousel\"\)|button\(\"Clear\"\)|button\(\"Toggle options\"\)|button\(\"Retry\"\)|\"Next slide\" \}|format!\(\"Slide|fn runs\(|sparkline\.clone\(\)" crates/kivact-ui/src` が何も返さない。`rg -n "focusable_when_disabled\(matches!" crates/kivact-ui/src` が何も返さない。上の命令が通る。

## U10：Api の setup の設定を一つにまとめ、tooltip を attach で作る（UI-22）
- **wave / lane / 前提**：W4 / U / U09（U05 の `query` の `OptionsCell<Binding<String>>`・`query_text`・`ComboState` の machine と、U08 の `delayed` の feed を前提にする）。この unit は W5 の前に走るので、名前は HEAD のもの（`Binding<T>`、`IntoBinding`、`into_binding()`、`use_theme() -> Value<Theme>`）を使う。
- **所有**：`crates/kivact-interaction/src/drag/{data.rs,handle.rs}`、`crates/kivact-interaction/src/combobox/handle.rs`、`crates/kivact-interaction/src/tooltip/api.rs`
- **目的**：UI-22。DragApi の `settings`・`modifiers`・`disabled` を一つの `OptionsCell<DragOptions<T>>` にする。ComboboxApi の `source`・`debounce`・`search_status`・`mode`（と U05 の `query`）を一つの `OptionsCell<ComboboxSetupOptions<K>>` にする（名前は `list/state.rs` の `ListSetupOptions` に揃える。W7b の U30b が同じ file に公開の `ComboboxOptions` を作るので、その名前は使わない）。tooltip の `engaged`・delayed の machine・その `visible` を、trigger の attach の中（`on_attach`）で作る。trigger は普通 tooltip と同時に mount するので、節約は trigger が mount しない tooltip の分だけである。初めの hover か focus で作る形は core に part の focus 進入の event が要るので、本改修では行わない。
- **手順**：
  1. `drag/data.rs`：`use kivact_core::state::Callback;` を `use kivact_core::state::{Callback, Value};` にする。`DragOptions<T>` の末尾に足す：
     ```rust
     /// The callback [`crate::DragApi::on_modifiers_changed`] gave.
     pub(super) modifiers: Option<Callback<DragUpdate<T>>>,
     /// The disabled value [`crate::DragApi::disabled`] gave.
     pub(super) disabled: Option<Value<bool>>,
     ```
     手書きの `Clone` に `modifiers: self.modifiers, disabled: self.disabled,`、`Default` に `modifiers: None, disabled: None,` を足す。
  2. `drag/handle.rs`：`pub struct DragApi` から field `modifiers: OptionsCell<Option<Callback<DragUpdate<T>>>>,` と `disabled: Signal<Option<Value<bool>>>,` の 2 行を消し（W1 の T06 がその上の `xtask-allow(state-fields)` の行を消している。`DragApi` は接尾辞 `Api` なので struct の印も無い）、`use_drag` の literal の `modifiers: OptionsCell::new(None),`、`disabled: signal(None),` を消す。使われなくなった import（`signal` など）は `cargo check` の warning に従って消す。次を置き換える：
     - `use_drag` の feed の閉包：`drag.settings.value().with(|settings| settings.disabled.is_some_and(|value| value.get()))`
     - `pub fn disabled`：`self.settings.update(|settings| settings.disabled = Some(disabled));`（doc の "Setup only." はそのまま）
     - `fn is_disabled`：`self.settings.get().disabled.is_some_and(|value| value.get_untracked())`
     - `on_modifiers_changed`：`self.settings.update(|settings| settings.modifiers = Some(f));`
     - `report_modifiers`：`let Some(callback) = self.settings.get().modifiers else {`
  3. `combobox/handle.rs`：`enum PresentationMode` の次に足す：
     ```rust
     /// What the setup calls of a [`ComboboxApi`] store.
     struct ComboboxSetupOptions<K: 'static> {
         /// The options: the caller's, or the results of [`ComboboxApi::search`].
         source: Value<Vec<ListItem<K>>>,
         debounce: Value<std::time::Duration>,
         search_status: Value<kivact_core::task::AsyncState<()>>,
         presentation: PresentationMode,
         /// The query text: a local signal, or the caller's binding.
         query: Binding<String>,
     }
     impl<K: 'static> Clone for ComboboxSetupOptions<K> {
         fn clone(&self) -> Self {
             *self
         }
     }
     impl<K: 'static> Copy for ComboboxSetupOptions<K> {}
     ```
     `pub struct ComboboxApi` の `source`、`debounce`、`search_status`、`mode`、`query` の 5 つの field（とその doc の行）を消し、`mode` があった場所に `settings: OptionsCell<ComboboxSetupOptions<K>>,` 1 つを置く。`use_combobox` の、`let source = OptionsCell::new(items.into_value());` から U05 が置いた `let query_text = memo(move || query.value().get().get());` までを次にする：
     ```rust
     let settings = OptionsCell::new(ComboboxSetupOptions {
         source: items.into_value(),
         debounce: std::time::Duration::from_millis(150).into_value(),
         search_status: (|| kivact_core::task::AsyncState::<()>::Idle).into_value(),
         presentation: PresentationMode::Inline,
         query: signal(String::new()).into_binding(),
     });
     let options = memo(move || settings.value().with(|settings| settings.source).get());
     // The one open state; the machine writes the caller's binding, never the reverse.
     // A combobox starts inline, so its list shows; `popup` closes it at
     // setup when the caller's open is false.
     let session = machine(
         machine::ComboState::Open {
             surface: Surface::Popover,
             filtering: false,
             highlight: None::<K>,
         },
         move || {
             machine::Machine::<K>::new(matches!(
                 settings.get().presentation,
                 PresentationMode::Inline
             ))
         },
         move |_, effect| match effect {
             machine::Effect::WriteQuery(text) => settings.get().query.set(text),
             machine::Effect::WriteOpen(open) => {
                 if let PresentationMode::Popup(binding) = settings.get().presentation
                     && binding.get_untracked() != open
                 {
                     binding.set(open);
                 }
             }
         },
     );
     let query_text = memo(move || settings.value().with(|settings| settings.query).get());
     ```
     `ComboboxApi { .. }` の literal から `source,`、`debounce: OptionsCell::new(..),`（2 行）、`search_status: OptionsCell::new(..),`（3 行）、`mode,`、`query,` を消し、`mode,` があった場所に `settings,` を置く。method の置換：
     ```
     perl -pi -e 's/self\.mode\.get\(\)/self.settings.get().presentation/g; s/self\.query\.get\(\)/self.settings.get().query/g' crates/kivact-interaction/src/combobox/handle.rs
     ```
     手で直す所：`query_binding` の `self.query.update(|slot| *slot = query);` → `self.settings.update(|settings| settings.query = query);`。`popup_with_focus_transfer` の `self.mode.update(|mode| *mode = PresentationMode::Popup(open));` → `self.settings.update(|settings| settings.presentation = PresentationMode::Popup(open));`。`pub fn query` の `self.query.value().get().get()` → `self.settings.value().with(|settings| settings.query).get()`。`search_status()` の本体 → `memo(move || self.settings.value().with(|settings| settings.search_status).get())`。`debounce()` → `self.settings.update(|settings| settings.debounce = delay.into_value());`。`search()` の `self.debounce.value().get().get()` → `self.settings.value().with(|settings| settings.debounce).get()`、`self.search_status.update(|status| *status = memo(..))` → `self.settings.update(|settings| settings.search_status = memo(..))`、`self.source.update(|source| *source = memo(..))` → `self.settings.update(|settings| settings.source = memo(..))`（memo の中身はそのまま）。
  4. `tooltip/api.rs`：
     - `pub struct TooltipApi` の定義（`#[derive(Clone, Copy)]` の行から閉じ括弧まで。上の `///` の doc はそのまま）を次にする（W1 の T06 が field の上の `xtask-allow(state-fields)` の行を消している。`TooltipApi` は接尾辞 `Api` なので struct の印は無い）：
       ```rust
       #[derive(Clone, Copy)]
       pub struct TooltipApi {
           options: OptionsCell<TooltipOptions>,
           trigger_state: Signal<Option<Value<PopupState>>>,
           content_state: Signal<Option<Value<PopupState>>>,
           /// What follows the attached trigger: made when it attaches and
           /// dropped when it unmounts.
           attached: Signal<Option<Attached>>,
           /// Whether the hint is shown: the attached trigger's shown state,
           /// false before it attaches.
           visible: Value<bool>,
           anchor: NodeRef,
           surface: NodeRef,
           hub: Hub,
           id: TooltipId,
           config: Value<kivact_core::style::Theme>,
       }
       ```
       その下に：
       ```rust
       /// The engagement's delayed visibility and the shown state of one trigger mount.
       #[derive(Clone, Copy)]
       struct Attached {
           delayed: DelayedApi,
           visible: Value<bool>,
       }
       ```
     - `use_tooltip` の本体を次にする（`hover` から `on_cleanup` までが `attach` へ移る）：
       ```rust
       let config = kivact_core::style::use_theme();
       let hub = Hub::of_app();
       let id = hub.take_id();
       let attached = signal(None::<Attached>);
       TooltipApi {
           options: OptionsCell::new(TooltipOptions::default()),
           trigger_state: signal(None),
           content_state: signal(None),
           attached,
           visible: (move || {
               attached.with(|attached| attached.as_ref().is_some_and(|a| a.visible.get()))
           })
           .into_value(),
           anchor: node_ref(),
           surface: node_ref(),
           hub,
           id,
           config,
       }
       ```
     - `impl TooltipApi` に足す（元の `use_tooltip` の閉包とコメントを移したもの。`Attached` は `PartialEq` を持たない（`DelayedApi` が持たない）ので、`attached` は `set` でなく `update` で書く）：
       ```rust
       /// Create what follows the attached trigger, for the life of its mount:
       /// the engagement, the delay and the shown state. Setup only, under the
       /// trigger mount's owner.
       fn attach(self) {
           let TooltipApi { options, attached, anchor, surface, hub, id, config, .. } = self;
           let hover = anchor.hovered();
           let surface_hover = surface.hovered();
           let focus = anchor.focus_within();
           let engaged = memo(move || {
               !options.value().get().disabled.is_some_and(|disabled| disabled.get())
                   && (hover.get() || surface_hover.get() || focus.get())
           });
           // The delay, or none while another tooltip is warm; both end with the
           // engagement. One tooltip at a time: the delayed machine's visibility
           // changes are the only events this tooltip sends to the App hub, so the
           // one shown last is visible, and one that another took over shows again
           // when that one hides.
           let delayed = delayed(
               engaged,
               move || {
                   if hub.warm_for(id) {
                       Duration::ZERO
                   } else {
                       config.get().ext::<crate::InteractionPolicy>().tooltip.delay
                   }
               },
               Duration::ZERO,
               move |shown| {
                   if shown {
                       hub.want(id);
                   } else {
                       hub.release(id, config.get_untracked().ext::<crate::InteractionPolicy>().tooltip.skip_delay);
                   }
               },
           );
           let visible = memo(move || delayed.is_open().get() && hub.current() == Some(id));
           attached.update(|slot| *slot = Some(Attached { delayed, visible }));
           on_cleanup(move || {
               if attached.try_get_untracked().is_some() {
                   attached.update(|slot| *slot = None);
               }
               if hub.next.try_get().is_some() {
                   hub.release(id, config.get_untracked().ext::<crate::InteractionPolicy>().tooltip.skip_delay);
               }
           });
       }
       ```
     - `fn dismiss` の本体：`if let Some(attached) = self.attached.get_untracked() { attached.delayed.hide(); }`
     - `pub fn trigger` の `Part::new(..)` の連鎖の最後（`.on_key(..)` の後）に `.on_attach(move |_| self.attach())` を足す。
  5. 数（一つの instance の setup が作るもの。part の中の memo は数えない）：

     | instance | 前（signals / memos / observers / effects / machines） | 後 |
     |---|---|---|
     | DragApi | 4 / 1 / 0 / 1 / 1（＋var 1） | 2 / 1 / 0 / 1 / 1（＋var 1） |
     | ComboboxApi（U05 の後） | OptionsCell 5 を含む signal 11 / memo 5 | OptionsCell 1 を含む signal 7 / memo 5 |
     | TooltipApi（trigger の attach 前） | 3 / 4 / 3 / 1 / 1 | 4 / 0 / 0 / 0 / 0 |
     | TooltipApi（attach 後） | 同上 | 4 / 4 / 3 / 1（feed） / 1 |

- **消すもの**：`DragApi::modifiers`、`DragApi::disabled`（field）、`ComboboxApi` の `source`・`debounce`・`search_status`・`mode`・`query` の field、`TooltipApi::delayed`、`use_tooltip` の中の `engaged`・`delayed`・`visible`・`on_cleanup`。
- **状態の閉じ方**：`DragOptions` と `ComboboxSetupOptions` は接尾辞 `Options`。`TooltipApi`・`DragApi`・`ComboboxApi` は接尾辞 `Api`。`Attached` は数える field を持たない（`DelayedApi` と `Value<bool>`）。`attached` は mount の寿命を表す slot（attach で入れ、unmount で空にする）で、W7b の U31（UI-01、part の slot の削除）の対象ではない。U31 はこの slot を残す。
- **試験**：足す・直す試験なし。`interaction_drag_edit.rs`、`interaction_slider.rs`、`ui_controls.rs`、`interaction_splitter.rs`、`interaction_combobox.rs`、`apg_combobox.rs`、`ui_combobox.rs`、`interaction_palette.rs`、`ui_command_palette.rs`、`interaction_tooltip.rs`、`apg_tooltip.rs`、`ui_disclosure_surfaces.rs`、`interaction_menu.rs` が通ること。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction
  cargo check -p kivact-interaction -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo check -p kivact-ui-creative -p kivact-ui-basic -p kivact-ui-soft -p kivact-ui-pop -p kivact-ui-game --all-targets
  KIVACT_TEST_MIRROR=off cargo test -p kivact-interaction --lib
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_drag_edit interaction_slider ui_controls interaction_splitter interaction_combobox apg_combobox ui_combobox interaction_palette ui_command_palette interaction_tooltip apg_tooltip ui_disclosure_surfaces interaction_menu
  ```
- **完了条件**：`rg -n "modifiers: OptionsCell|disabled: Signal<Option<Value<bool>>>" crates/kivact-interaction/src/drag` と `rg -n "source: OptionsCell|debounce: OptionsCell|search_status: OptionsCell|mode: OptionsCell|query: OptionsCell" crates/kivact-interaction/src/combobox` が何も返さない。`rg -n "let engaged = memo" crates/kivact-interaction/src/tooltip/api.rs` がちょうど 1 行を返し、その行は `fn attach` の本体の中にある。上の命令が通る。

## U11〜U14 共通：kivact-ui を部品ごとの directory に移す（公開 API は変えない）

U11〜U14 は W4 の lane U で、この順に一つずつ行う。どれも**移動だけ**で、関数の本体・型・公開の path（`kivact_ui::X`）を変えない。5 つの kit はこの時点でまだ compile の対象なので、kit が使う名前の path は `lib.rs` の `pub use` で今と同じに保つ。

### 移動の規則（4 unit 共通）

1. 部品 `<name>` の module の root は `crates/kivact-ui/src/<name>.rs`（今の file）。新しい directory `crates/kivact-ui/src/<name>/` を作り、その中に `look.rs` を作る。root の先頭の `use` 群の後に次の 1 行を足す：
   ```rust
   pub(crate) mod look;
   ```
   `look.rs` の先頭は `//! Role: <部品> の recipe の状態・token・recipe の表・既定の look。` の 1 行（英語で書く：`//! Role: The <name> recipe state, tokens, recipe table and default looks.`）。
2. 「関数を移す」とは、その関数の doc comment と属性ごと切り取り、移動先に貼ることである。移動先では `pub(super) fn` / `fn` を `pub(crate) fn` にする（他の部品の look から呼ばれるものがあるため。`unreachable_pub` は `pub(crate)` には出ない）。
3. 「型を移す」とは、その型と、その型の `impl`（`impl Variant for X`、`impl Default for X`、`impl Deref for X`、`constant_value!(X)` の行を含む）を切り取って貼ることである。型の可視性（`pub`）は変えない。
4. 移動先の表に「root」とある型は `crates/kivact-ui/src/<name>.rs` の `use` 群の後に、「look」とある型は `<name>/look.rs` に置く。
5. `recipes.rs` の `recipe_table! { XRecipes { .. } }` の一つ（「表」の列）は、`impl XRecipes { .. }`（`has_custom_mark` など）と一緒に `<name>/look.rs` に移す。表の中の default の path（`look_extra::select_trigger` など）は、移動先の同じ file の関数名（`select_trigger`）に直す。`UiRecipes` の `tables { .. }` と平らな field の default の path は、`crate::<name>::look::<関数名>`・`crate::<name>::look::XRecipes` に直す。
6. 移動のあと、各 file の `use` を直す（`super::values::ButtonState` → `crate::button::look::ButtonState` など）。他の部品の look 関数を使う look（`toggle_group` が `toggle_button` を使うなど）は `crate::<他の部品>::look::<関数>` で呼ぶ。これは compile の誤りを手がかりに機械的に直す。
7. `lib.rs` の `pub use` は、移した型を新しい path から同じ名前で再公開する（例：`pub use values::{ButtonState, ..}` から `ButtonState` を消し、`pub use button::look::ButtonState;` を足す）。`mod` の行も移動に合わせる。公開の名前の集合は変えない。確かめ方：unit の前後で次の出力が同じ（並びは問わない）。
   ```
   ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
   cargo doc -p kivact-ui --no-deps 2>/dev/null; ls target/doc/kivact_ui | sort > target/u_doc_after.txt
   ```
   （unit を始める前に同じ命令で `target/u_doc_before.txt` を作り、`diff` が空であること。`target/doc/kivact_ui/` の直下の `struct.*.html`・`enum.*.html`・`fn.*.html` の一覧が公開名の一覧になる。）
8. file の名前を変えるときは `git mv` を使う（例：`git mv crates/kivact-ui/src/toggle_parts.rs crates/kivact-ui/src/toggle/parts.rs`）。移した file の `mod` 宣言は root の file に移し（`mod parts;`）、中の `super::`・`crate::` の path を直す。
9. 共有の look 関数（2 つ以上の部品の表が使うもの）は `crates/kivact-ui/src/look_common.rs`（U11 で作る）に移す。名前が重なるものは表のとおり改名する。
10. 移動が終わって空になった file は `git rm` する。各 unit の「消すもの」に名前がある。
11. 部品の module の依存は view → look → `crate::tokens` → `X::tokens` の一方向にする（W4 の差戻し）。
    - 部品の token の型（`XTokens` とその field の型）は `X/tokens.rs` に置く。`X/tokens.rs` は kivact_core の型だけを読む。
    - look が使う型（variant など）は look に置き、view から読む。look は view を読まない。
    - 確認は `cargo xtask check --no-write`（workspace 全体）。`--crate` は style の規則を検査しない。

### 共有の file（U11 で作る）

- `crates/kivact-ui/src/recipe.rs`：`recipes.rs` から `pub struct Recipe<S>`、`type RecipeFn<S>`、`impl Clone/From/Deref for Recipe<S>`、`impl<S> Recipe<S>`、`macro_rules! recipe_table` を移す。macro の定義の直後に `pub(crate) use recipe_table;` を置く。macro の中の型は `$crate::recipe::Recipe`、`kivact_core::style::{Theme, Style}`、`$crate::variants::Look` と完全な path で書き直す（呼ばれる file の `use` に頼らない）。macro の中の `Recipe::from(..).named(..)` の `named` は `pub(crate)` のまま。`recipes.rs` の先頭に `use crate::recipe::{Recipe, recipe_table};`。`lib.rs` の `pub use recipes::{.., Recipe, ..}` の `Recipe` を `pub use recipe::Recipe;` にする。各 `look.rs` は `crate::recipe::recipe_table! { .. }` と path で呼ぶ。
- `crates/kivact-ui/src/control_size.rs`：`values.rs` から `ControlSize`（と `constant_value!` の行）、`InheritedControlSize`、`provide_control_size`、`default_control_size` を移す（`default_control_size` は `pub(crate)`）。`lib.rs`：`mod control_size; pub use control_size::{ControlSize, provide_control_size};`。
- `crates/kivact-ui/src/look_common.rs`：先頭 `//! Role: Share sizing and look helpers that two or more components' recipes use.`。U11〜U14 が次の関数を移す（改名は右の列）：

| 元 | 関数 | look_common での名前 | 移す unit |
|---|---|---|---|
| `look.rs` | `font_size`、`height`、`gap` | 同じ | U11 |
| `look_input.rs` | `flat` | 同じ | U11 |
| `look_extra.rs` | `trigger`、`description`、`mark`、`surface` | 同じ | U12 |
| `look_collections.rs` | `list` | 同じ | U13 |
| `look_navigation.rs` | `part` | 同じ | U13 |
| `look_metrics.rs` | `height` | `metric_height`（progress の look の private な fn。look_common には置かない。W4 の差戻し） | U14 |
| `look_indicators.rs` | `side`、`tone` | 同じ | U14 |
| `look_chat.rs` | `column`、`row`、`slot`、`muted` | 同じ | U14 |

## U11：操作部品と入力を directory に移す（移動 1/4）

- **wave / lane / 前提**：W4 / U / U10
- **所有**：`crates/kivact-ui/src/` の次：`lib.rs`、`recipes.rs`、`recipe.rs`（新設）、`control_size.rs`（新設）、`look_common.rs`（新設）、`look.rs`、`look_controls.rs`、`look_extra.rs`、`look_feedback.rs`、`look_files.rs`、`look_input.rs`、`group_look.rs`、`group_types.rs`、`toolbar_look.rs`、`collection_values.rs`、`values.rs`、`component_values.rs`、`button.rs`、`button_tokens.rs`、`toggle.rs`、`toggle_parts.rs`、`switch_drag.rs`、`switch_drag/`、`toggle_button.rs`、`toggle_button_tokens.rs`、`toggle_group.rs`、`button_group.rs`、`toolbar.rs`、`text_input.rs`、`number_input.rs`、`tag_input.rs`、`slider.rs`、`splitter.rs`、`split.rs`、`field.rs`、`input_group.rs`、`input_group_context.rs`、`dropzone.rs`、`dropzone_tokens.rs`、`tokens.rs`、`layer.rs`、それらの新しい directory。他の file は `use` の path の追従だけ。
- **目的**：UI-14 の準備（物理的な配置だけ）。W7 以降の lane が部品ごとにファイルを排他で持てるようにする。
- **手順**：
  1. 共有の file（上の節）の `recipe.rs`、`control_size.rs`、`look_common.rs` を作り、look_common の U11 の行を移す。
  2. 次の表のとおり移す。「表」は `recipes.rs` の `recipe_table!`、「平ら」は `UiRecipes` の平らな field の default の関数（`UiRecipes` に残し、path だけ直す）。

| 部品（root） | directory に移す file | `<name>/look.rs` に移すもの | root に移すもの |
|---|---|---|---|
| `button.rs` | — | `look.rs`：`button`、`button_content`、`button_icon`、`button_busy`、`tinted`（平ら）。`values.rs`：`ButtonState`。`button_tokens.rs`：`ButtonBusyMotion`、`ButtonTokens`（`button.rs` の `pub use crate::button_tokens::{..}` の行を消し、lib.rs の再公開を `button::look::{ButtonBusyMotion, ButtonTokens}` にする） | `values.rs`：`ButtonVariant` |
| `toggle.rs`（checkbox・switch） | `toggle_parts.rs` → `toggle/parts.rs`、`switch_drag.rs` → `toggle/switch_drag.rs`、`switch_drag/` → `toggle/switch_drag/` | `look.rs`：`checkbox`、`switch`、`toggle`、`indicator`、`leading`、`indicator_width`、`draw_checkbox`、`draw_switch`（平ら）。`look_controls.rs`：`empty_toggle_part`（平ら）。`values.rs`：`CheckControl`、`CheckableState`、`ThumbDrag`、`TrackInput`、`Lift` | — |
| `toggle_button.rs` | — | `look.rs`：`toggle_button`（平ら）。`values.rs`：`ToggleButtonState`。`toggle_button_tokens.rs`：`ToggleButtonTokens`、`ToggleLabelReservation` | — |
| `toggle_group.rs` | — | `group_look.rs`：`toggle_root`、`toggle_item`。`group_types.rs`：`ToggleGroupTokens`、`ToggleGroupState`、`ToggleGroupItemState`。表 `ToggleGroupRecipes` | — |
| `button_group.rs` | — | `group_look.rs`：`button_root`、`button_member`、`button_separator`、`button_text`。`group_types.rs`：`ButtonGroupTokens`、`ButtonGroupState`、`ButtonGroupMemberState`。表 `ButtonGroupRecipes` | — |
| `toolbar.rs` | — | `toolbar_look.rs`：`root_style`。`collection_values.rs`：`ToolbarTokens`、`ToolbarState`。表 `ToolbarRecipes` | `collection_values.rs`：`ToolbarEdge` |
| `text_input.rs`（text_input・text_area） | — | `look.rs`：`text_input`、`text_area`（平ら）。`values.rs`：`TextInputState` | — |
| `number_input.rs` | `number_input/draft.rs`（新設） | `look_input.rs`：`number_root`、`number_input`。表 `NumberInputRecipes` | — |
| `tag_input.rs` | — | `look_input.rs`：`tag_root`、`tag`、`tag_remove`、`tag_input`。表 `TagInputRecipes` | — |
| `slider.rs` | — | `look_controls.rs`：`slider_metrics`、`slider_track`、`slider_range`、`slider_thumb`。`values.rs`：`AxisState`。表 `SliderRecipes` | — |
| `splitter.rs` | — | `look_controls.rs`：`splitter_handle`。表 `SplitterRecipes` | — |
| `split.rs` | — | `look_controls.rs`：`split_grip`。表 `SplitRecipes` | — |
| `field.rs` | — | `look_extra.rs`：`field`、`field_label`、`field_hint`、`field_error`。`values.rs`：`FieldState`。表 `FieldRecipes` | — |
| `input_group.rs` | `input_group_context.rs` → `input_group/context.rs` | `look_feedback.rs`：`group_root`、`group_label`、`group_members`、`group_addon`、`group_member`、`group_block`。`component_values.rs`：`InputGroupState`（と `impl Default`）、`InputGroupMemberState`、`InputGroupMemberKind`。表 `InputGroupRecipes` | — |
| `dropzone.rs` | — | `look_files.rs`：`dropzone_root`、`dropzone_label`、`dropzone_hint`、`dropzone_status`、`dropzone_icon`。`dropzone_tokens.rs`：`DropzoneTokens`。表 `DropzoneRecipes` | — |

  3. `number_input.rs` の `enum NumberText`、`enum NumberEvent`、`struct NumberInputSnapshot`、`impl Machine for NumberInputSnapshot` とそれだけが使う private な関数を `number_input/draft.rs`（`//! Role: The number field's typed-text draft machine.`）に移し、root に `mod draft;`、使う名前を `pub(super)` にする。
  4. `AxisState` を使う `splitter/look.rs`・`split/look.rs` は `use crate::slider::look::AxisState;`。`CheckableState` を使う他の部品（radio_group）は `use crate::toggle::look::CheckableState;`。
  5. `lib.rs` の `mod` 行・`pub use` 行を規則 7 のとおり直す。
- **消すもの**：`button_tokens.rs`、`toggle_button_tokens.rs`、`toggle_parts.rs`（移動）、`switch_drag.rs`・`switch_drag/`（移動）、`group_look.rs`、`group_types.rs`、`toolbar_look.rs`、`dropzone_tokens.rs`、`input_group_context.rs`（移動）、`look_input.rs`（`flat` を look_common に移したあと空になる）。
- **状態の閉じ方**：新しい型は無い（移動だけ）。
- **試験**：変更しない。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-ui
  cargo check -p kivact-ui --all-targets
  cargo check -p kivact-ui-basic -p kivact-ui-soft -p kivact-ui-pop -p kivact-ui-game -p kivact-ui-creative --all-targets
  cargo clippy -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-ui
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_controls ui_theming ui_toolbar ui_field ui_dropzone
  ```
- **完了条件**：規則 7 の `diff` が空。`ls crates/kivact-ui/src/{button,toggle,toggle_button,toggle_group,button_group,toolbar,text_input,number_input,tag_input,slider,splitter,split,field,input_group,dropzone}/look.rs` がすべてある。上の命令が通る。

## U12：重ねて開く部品を directory に移す（移動 2/4）

- **wave / lane / 前提**：W4 / U / U11
- **所有**：`crates/kivact-ui/src/` の `lib.rs`、`recipes.rs`、`look_common.rs`、`look_extra.rs`、`look_controls.rs`、`look_palette.rs`、`values.rs`、`dialog.rs`、`dialog_tokens.rs`、`drawer_swipe.rs`、`tooltip.rs`、`hover_card.rs`、`hover_card_values.rs`、`menu.rs`、`select.rs`、`select/`、`selector_presentation.rs`、`selection_field.rs`、`combobox.rs`、`combobox/`、`command_palette.rs`、`disclosure.rs`、`disclosure_group.rs`、`disclosure_values.rs`、`disclosure_tokens.rs`、`navigation_menu.rs`、`navigation_link.rs`、`navigation_context.rs`、`look_navigation.rs`、`navigation_values.rs`、`toast.rs`、`toast_tokens.rs`、`toast_swipe.rs`、`toast_swipe/`、それらの新しい directory。
- **目的**：U11 と同じ（UI-14 の配置）。
- **手順**：
  1. look_common に U12 の行（`trigger`、`description`、`mark`、`surface`）を移す。
  2. 次の表のとおり移す。

| 部品（root） | directory に移す file | `<name>/look.rs` に移すもの | root に移すもの |
|---|---|---|---|
| `dialog.rs` | `drawer_swipe.rs` → `dialog/drawer_swipe.rs` | `look_extra.rs`：`dialog_surface`、`dialog_title`、`dialog_description`、`dialog_header`、`dialog_media`、`dialog_body`、`dialog_footer`、`dialog_scrim`、`dialog_close`、`drawer_handle`。`values.rs`：`DialogState`。`dialog_tokens.rs`：`DialogTokens`、`DialogCenter`。表 `DialogRecipes` | — |
| `tooltip.rs` | — | `look_extra.rs`：`tooltip`、`no_arrow`。`values.rs`：`TooltipArrowState`。表 `TooltipRecipes` | — |
| `hover_card.rs` | — | `look_extra.rs`：`hover_card`。`hover_card_values.rs`：`HoverCardSurfaceState`、`HoverCardTokens`。表 `HoverCardRecipes` | — |
| `menu.rs` | — | `look_extra.rs`：`menu_trigger`、`menu_item`、`separator`、`menu_label`、`shortcut`、`menu_description`、`menu_check`、`menu_submenu`、`menu_inset`。表 `MenuRecipes` | — |
| `select.rs` | `select/handle.rs` の中身を `select.rs` に入れて `select/handle.rs` を消す（`select.rs` の `mod handle;`・`pub use handle::..` の行を消す）。`select/machine.rs` は残す。`selector_presentation.rs` → `select/presentation.rs` | `look_extra.rs`：`select_trigger`、`option`、`option_check`、`option_description`、`select_indicator`。`values.rs`：`SelectTriggerState`。表 `SelectRecipes` | — |
| `selection_field.rs` | — | — （U23 で消す） | — |
| `combobox.rs` | `combobox/tokens.rs` の中身を `combobox/look.rs` の末尾に入れ、`combobox/tokens.rs` を消す | 表 `ComboboxRecipes`（`combobox/look.rs` へ） | — |
| `command_palette.rs` | — | `look_palette.rs` の 12 関数すべて。表 `CommandPaletteRecipes` | — |
| `disclosure.rs` | `disclosure_group.rs` → `disclosure/group.rs`（`lib.rs` の `pub use disclosure_group::{..}` は `pub use disclosure::group::{DisclosureGroup, disclosure_group};`、root に `pub(crate) mod group;`） | `look_extra.rs`：`disclosure_root`、`disclosure_trigger`、`disclosure_indicator`、`disclosure_content`。`disclosure_values.rs`：`DisclosureGroupPosition`、`DisclosureRecipeState`。`disclosure_tokens.rs`：`DisclosureTokens`、`DisclosureMotion`。表 `DisclosureRecipes` | — |
| `navigation_menu.rs` | `navigation_link.rs` → `navigation_menu/link.rs`、`navigation_context.rs` → `navigation_menu/context.rs` | `look_navigation.rs`：`navigation_root`、`panel`、`action`、`link`、`link_title`、`link_description`、`link_icon`。`navigation_values.rs`：`NavigationLinkState`、`NavigationTriggerState`。表 `NavigationMenuRecipes` | — |
| `toast.rs` | `toast_swipe.rs` → `toast/swipe.rs`、`toast_swipe/` → `toast/swipe/` | `look_controls.rs`：`toast_region`、`toast_group`、`toast`、`toast_icon`、`toast_title`、`toast_body`、`toast_close`、`toast_row`。`values.rs`：`ToastGroupState`。`toast_tokens.rs`：`ToastTokens`、`ToastRowMotion`。表 `ToastRecipes` | — |

  3. `lib.rs` の `mod`・`pub use` を規則 7 のとおり直す。
- **消すもの**：`dialog_tokens.rs`、`hover_card_values.rs`、`disclosure_values.rs`、`disclosure_tokens.rs`、`toast_tokens.rs`、`look_palette.rs`、`look_extra.rs`（tabs の 4 関数 `tabs_list`・`no_tab_indicator`・`tab`・`panel` が残るので、U12 では消さず U13 で消す）、`selector_presentation.rs`・`drawer_swipe.rs`・`toast_swipe.rs`・`toast_swipe/`・`disclosure_group.rs`・`navigation_link.rs`・`navigation_context.rs`（移動）、`select/handle.rs`、`combobox/tokens.rs`。
- **状態の閉じ方**：新しい型は無い。
- **試験**：変更しない。
- **確認（tier a）**：U11 と同じ命令。test は `-- ui_disclosure_surfaces ui_combobox ui_command_palette interaction_dialog interaction_menu interaction_select interaction_toast interaction_tooltip ui_navigation`。
- **完了条件**：規則 7 の `diff` が空。`look_palette.rs` が無い。上の命令が通る。

## U13：collection とナビゲーションを directory に移す（移動 3/4）

- **wave / lane / 前提**：W4 / U / U12
- **所有**：`crates/kivact-ui/src/` の `lib.rs`、`recipes.rs`、`look_common.rs`、`look_extra.rs`、`look_collections.rs`、`look_controls.rs`、`look_radio.rs`、`look_navigation.rs`、`look_files.rs`、`look_static_collections.rs`、`navigation_values.rs`、`navigation_tokens.rs`、`collection_values.rs`、`tree_values.rs`、`table_values.rs`、`values.rs`、`data_table_tokens.rs`、`carousel_values.rs`、`carousel_tokens.rs`、`carousel_geometry.rs`、`column_values.rs`、`data_column.rs`、`pagination_draft.rs`、`item_row.rs`、`item_context.rs`、`tokens.rs`、`list.rs`、`tree.rs`、`tabs.rs`、`radio_group.rs`、`data_table.rs`、`data_table/`、`pagination.rs`、`breadcrumb.rs`、`stepper.rs`、`carousel.rs`、`item_list.rs`、`table.rs`、それらの新しい directory。
- **目的**：U11 と同じ。
- **手順**：
  1. look_common に U13 の行（`list`、`part`）を移す。
  2. 次の表のとおり移す。

| 部品（root） | directory に移す file | `<name>/look.rs` に移すもの | root に移すもの |
|---|---|---|---|
| `list.rs` | — | `look_collections.rs`：`list_item`、`list_description`。表 `ListRecipes` | — |
| `tree.rs` | — | `look_collections.rs`：`tree_item`、`tree_marker`。`tree_values.rs`：`TreeRowRecipeState`。`collection_values.rs`：`TreeTokens`。表 `TreeRecipes` | — |
| `tabs.rs` | — | `look_extra.rs`：`tabs_list`、`no_tab_indicator`、`tab`、`panel`。表 `TabsRecipes`（と `impl TabsRecipes { has_custom_indicator }`） | — |
| `radio_group.rs` | — | `look_radio.rs`：`group`、`item`、`indicator`。`look_controls.rs`：`radio_group`、`radio_item`、`radio_indicator`、`radio_mark`、`radio_description`。表 `RadioGroupRecipes`（と `impl RadioGroupRecipes { has_custom_mark }`） | `values.rs`：`RadioGroupVariant` |
| `data_table.rs` | `column_values.rs` と `data_column.rs` を一つにして `data_table/column.rs`（`column_values.rs` を `git mv` し、`data_column.rs` の `header_context_menu` の `impl` をその末尾に入れる。root に `pub(crate) mod column;`、lib.rs は `pub use data_table::column::DataColumn;`） | `look_collections.rs`：`group_header`、`frame`、`toolbar`、`footer`、`header`、`row`、`cell`、`check`、`sort`、`page`。`values.rs`：`DataTableRowState`、`DataTableCellState`、`DataTableCheckState`、`DataTablePageState`。`table_values.rs`：`DataTableGroupState`。`data_table_tokens.rs`：すべて。表 `DataTableRecipes` | `table_values.rs`：`RowStatus` は `data_table/types.rs` の末尾へ |
| `pagination.rs` | `pagination_draft.rs` → `pagination/draft.rs` | `look_navigation.rs`：`root`、`pagination_item`、`pagination_wrapper`。`navigation_values.rs`：`PaginationItemState`、`PaginationSummary`。表 `PaginationRecipes` | `navigation_values.rs`：`PageItem`、`PaginationMode`、`PaginationWindow` |
| `breadcrumb.rs` | — | `look_navigation.rs`：`breadcrumb_root`、`breadcrumb_list`、`breadcrumb_item`。`navigation_values.rs`：`BreadcrumbItemState`。表 `BreadcrumbRecipes` | — |
| `stepper.rs` | — | `look_navigation.rs`：`stepper_root`、`stepper_item`、`stepper_layout`、`stepper_body`、`stepper_decoration`。`navigation_values.rs`：`StepperItemState`、`StepperState`。`navigation_tokens.rs`：`StepperTokens`。表 `StepperRecipes` | `navigation_values.rs`：`StepStatus`、`CollapsedSteps`、`StepIndicatorKind` |
| `carousel.rs` | `carousel_geometry.rs` → `carousel/geometry.rs` | `look_files.rs`：`carousel_gap`、`carousel_padding`、`carousel_fade_width`、`carousel_root`、`carousel_track`、`carousel_viewport`、`carousel_slide`、`carousel_control`。`carousel_values.rs`：`CarouselSurfaceState`。`carousel_tokens.rs`：`CarouselTokens`。表 `CarouselRecipes` | `carousel_values.rs`：`CarouselMode` |
| `item_list.rs`（item_list・item_row） | `item_row.rs` → `item_list/row.rs`、`item_context.rs` → `item_list/context.rs`（lib.rs：`pub use item_list::row::{ItemRow, item_row};`） | `look_static_collections.rs`：`list`→改名 `item_list_root`、`item`、`content`、`title`、`description`、`media`、`actions`、`band`、`item_body`、`list_header`、`marker`。`collection_values.rs`：`ItemListTokens`、`ItemRowState`。表 `ItemListRecipes` | `collection_values.rs`：`ItemPosition`、`ItemListMarker` |
| `table.rs` | — | `look_static_collections.rs`：`table`、`group`、`footer`、`row`、`cell`、`head`。`collection_values.rs`：`TableTokens`。表 `TableRecipes` | — |

  3. `NavigationTokens`（pagination・breadcrumb・navigation_menu が共有）は `navigation_tokens.rs` から `tokens.rs`（`UiTokens` の file）の末尾に移す（U21 で消える）。
  4. `lib.rs` の `mod`・`pub use` を規則 7 のとおり直す。
- **消すもの**：`look_extra.rs`、`look_collections.rs`、`look_radio.rs`、`look_static_collections.rs`、`look_files.rs`、`look_controls.rs`（U12 までで toast を、ここで radio を移すと空になる）、`look_navigation.rs`、`navigation_values.rs`、`navigation_tokens.rs`、`collection_values.rs`、`tree_values.rs`、`table_values.rs`、`data_table_tokens.rs`、`carousel_values.rs`、`carousel_tokens.rs`、`data_column.rs`、移動した file（`column_values.rs`、`carousel_geometry.rs`、`pagination_draft.rs`、`item_row.rs`、`item_context.rs`）。
- **状態の閉じ方**：新しい型は無い。
- **試験**：変更しない。
- **確認（tier a）**：U11 と同じ命令。test は `-- ui_collections ui_data_table ui_navigation ui_carousel apg_tabs apg_radio_group apg_tree_view apg_breadcrumb apg_carousel interaction_tree interaction_tabs interaction_table`。
- **完了条件**：規則 7 の `diff` が空。上の「消すもの」が一つも無い。上の命令が通る。

## U14：表示と分野の部品を directory に移し、残りの共有 file を消す（移動 4/4）

- **wave / lane / 前提**：W4 / U / U13
- **所有**：`crates/kivact-ui/src/` の `lib.rs`、`recipes.rs`、`look_common.rs`、`tokens.rs`、`look.rs`、`look_feedback.rs`、`look_indicators.rs`、`look_metrics.rs`、`look_chat.rs`、`look_calendar.rs`、`values.rs`、`component_values.rs`、`indicator_values.rs`、`chat_values.rs`、`chat_tokens.rs`、`code_values.rs`、`kbd_values.rs`、`calendar_values.rs`、`calendar_tokens.rs`、`date_input_tokens.rs`、`avatar_tokens.rs`、`progress_tokens.rs`、`sparkline_tokens.rs`、`alert.rs`、`card.rs`、`empty_state.rs`、`empty_state_changes.rs`、`badge.rs`、`kbd.rs`、`kbd_labels.rs`、`avatar.rs`、`avatar_initials.rs`、`avatar_group.rs`、`spinner.rs`、`skeleton.rs`、`progress.rs`、`sparkline.rs`、`heading.rs`、`stat.rs`、`description_list.rs`、`chat_message.rs`、`chat_thread.rs`、`chat_composer.rs`、`composer_history.rs`、`typing_indicator.rs`、`code_block.rs`、`code_editing.rs`、`code_gutter.rs`、`calendar.rs`、`calendar/`、`date_input.rs`、`date_input/`、`time_input.rs`、`time_input/`、`time_parse.rs`、`date_range_input.rs`、`date_range_draft.rs`、`date_time_input.rs`、`date_time_model.rs`、`date_time_sheet.rs`、`date_trigger.rs`、`date_wheel.rs`、`date_sheet.rs`、`date_parts.rs`（新設）、それらの新しい directory。
- **目的**：U11 と同じ。U14 の後、ui の look と状態の型はすべて部品の `look.rs`・`look_common.rs`・`tokens.rs`（共有の Tokens だけ）・`recipes.rs`（`UiRecipes` だけ）にある。
- **手順**：
  1. look_common に U14 の行（`metric_height`、`side`、`tone`、`column`、`row`、`slot`、`muted`）を移す。
  2. 次の表のとおり移す。

| 部品（root） | directory に移す file | `<name>/look.rs` に移すもの | root に移すもの |
|---|---|---|---|
| `alert.rs` | — | `look_feedback.rs`：`alert_root`、`alert_title`、`alert_description`、`alert_icon`、`alert_actions`、`alert_content`、`alert_control`、`alert_wrapper`。`component_values.rs`：`AlertState`、`Expansion`。表 `AlertRecipes` | `component_values.rs`：`AlertVariant` |
| `card.rs` | — | `look_feedback.rs`：`card_root`、`card_region`、`card_media`、`card_overlay`。`component_values.rs`：`CardState`、`Selectable`。表 `CardRecipes` | `component_values.rs`：`CardAction` |
| `empty_state.rs` | `empty_state_changes.rs` → `empty_state/changes.rs` | `look_feedback.rs`：`empty_root`、`empty_title`、`empty_description`、`empty_icon`、`empty_actions`。`component_values.rs`：`EmptyStateState`。表 `EmptyStateRecipes` | — |
| `badge.rs` | — | `look_indicators.rs`：`badge`、`badge_label`（平ら）。`indicator_values.rs`：`BadgeState` | — |
| `kbd.rs`（kbd・kbd_combo） | `kbd_labels.rs` → `kbd/labels.rs` | `look_indicators.rs`：`kbd`、`kbd_key`（平ら）。`kbd_values.rs`：`KbdKeyState` | — |
| `avatar.rs` | `avatar_initials.rs` → `avatar/initials.rs` | `look_indicators.rs`：`avatar`、`avatar_image`、`avatar_status`、`avatar_status_content`（平ら）。`indicator_values.rs`：`AvatarState`。`avatar_tokens.rs`：`AvatarTokens` | `indicator_values.rs`：`AvatarShape` |
| `avatar_group.rs` | — | `look_indicators.rs`：`avatar_group`、`avatar_group_member`、`avatar_group_overflow`（平ら） | — |
| `spinner.rs` | — | `look_indicators.rs`：`spinner`（平ら）。`indicator_values.rs`：`SpinnerState` | — |
| `skeleton.rs` | — | `look_indicators.rs`：`skeleton`（平ら）。`indicator_values.rs`：`SkeletonState` | `indicator_values.rs`：`SkeletonShape` |
| `progress.rs` | — | `look_metrics.rs`：`ring`、`gauge`、`progress_track`、`progress_fill`、`progress_label`。`values.rs`：`ProgressState`（と `impl PartialEq`）。`progress_tokens.rs`：`ProgressTokens`、`ProgressMotion`。表 `ProgressRecipes` | `values.rs`：`ProgressVariant` |
| `sparkline.rs` | — | `look_metrics.rs`：`sparkline_root`。`sparkline_tokens.rs`：`SparklineTokens`、`SparklineFill`。表 `SparklineRecipes` | — |
| `heading.rs` | — | `look_metrics.rs`：`heading`（平ら）。`component_values.rs`：`HeadingState` | — |
| `stat.rs` | — | `look_metrics.rs`：`stat_root`、`stat_label`、`stat_value`、`stat_delta`、`stat_slot`。`component_values.rs`：`StatState`。表 `StatRecipes` | `component_values.rs`：`Trend` |
| `description_list.rs` | — | `look_metrics.rs`：`description_root`、`description_item`、`description_term`、`description_detail`。表 `DescriptionListRecipes` | — |
| `chat_message.rs` | — | `look_chat.rs`：`author`、`message_root`、`message_body`、`reactions`。`chat_values.rs`：`ChatMessageState`。表 `ChatMessageRecipes` | `chat_values.rs`：`ChatSender`、`ChatMessageStatus`、`ChatMessageVariant`、`ChatAlign`、`ChatBubbleGroup`、`ChatReactionsSide` |
| `chat_thread.rs` | — | `look_chat.rs`：`thread_root`、`viewport`、`jump`。`chat_values.rs`：`ChatThreadState`。表 `ChatThreadRecipes` | — |
| `chat_composer.rs` | `composer_history.rs` → `chat_composer/history.rs` | `look_chat.rs`：`composer_surface`、`composer_status`、`input`。`chat_values.rs`：`ChatComposerState`。表 `ChatComposerRecipes` | `chat_values.rs`：`ComposerStatus`、`ComposerStatusPosition` |
| `typing_indicator.rs` | — | `look_chat.rs`：`typing_dots`。`chat_values.rs`：`TypingIndicatorState`。表 `TypingIndicatorRecipes` | — |
| `code_block.rs` | `code_editing.rs` → `code_block/editing.rs`、`code_gutter.rs` → `code_block/gutter.rs` | `look_chat.rs`：`code_root`、`code_header`、`code_body`、`code_viewport`、`code_text`、`code_gutter`。`code_values.rs`：`CodeBlockState`。`chat_tokens.rs`：`CodeBlockTokens`。表 `CodeBlockRecipes` | `code_values.rs`：`CodeCollapse` |
| `calendar.rs` | — | `look_calendar.rs`：`root`、`month`、`day`、`range`、`events`、`event`。`calendar_values.rs`：`CalendarDayVisual`。`calendar_tokens.rs`：`CalendarTokens`、`CalendarEventVisual`。表 `CalendarRecipes` | `calendar_values.rs`：`CalendarCaption`、`CalendarEvent` |
| `date_parts.rs`（新設。中身は `//! Role: Share the date and time fields' trigger, wheel and sheet.` と `pub(crate) mod look; pub(crate) mod sheet; pub(crate) mod trigger; pub(crate) mod wheel;`） | `date_trigger.rs` → `date_parts/trigger.rs`、`date_wheel.rs` → `date_parts/wheel.rs`、`date_sheet.rs` → `date_parts/sheet.rs` | `look_calendar.rs`：`input`、`wheel`。`date_input_tokens.rs`：`DateInputTokens` | — |
| `date_input.rs` | — | 表 `DateInputRecipes` | — |
| `time_input.rs` | `time_parse.rs` → `time_input/parse.rs` | 表 `TimeInputRecipes` | — |
| `date_range_input.rs` | `date_range_draft.rs` → `date_range_input/draft.rs` | 表 `DateRangeInputRecipes` | — |
| `date_time_input.rs` | `date_time_model.rs` → `date_time_input/model.rs`、`date_time_sheet.rs` → `date_time_input/sheet.rs` | 表 `DateTimeInputRecipes` | — |

  3. 共有の Tokens を `tokens.rs` の末尾に移す：`indicator_values.rs` の `IndicatorTokens`（spinner・skeleton・avatar_group が共有）と、`chat_tokens.rs` の `ChatTokens`（chat_thread・typing_indicator が共有）。
  4. `lib.rs` の `mod`・`pub use` を規則 7 のとおり直す。`lib.rs` に残る非公開の `mod` 行は、部品の root、`control_size`、`date_parts`、`helpers`、`layer`、`look_common`、`recipe`、`recipes`、`tokens`、`variants` だけになる。
- **消すもの**：`look.rs`、`look_feedback.rs`、`look_indicators.rs`、`look_metrics.rs`、`look_chat.rs`、`look_calendar.rs`、`values.rs`、`component_values.rs`、`indicator_values.rs`、`chat_values.rs`、`chat_tokens.rs`、`code_values.rs`、`kbd_values.rs`、`calendar_values.rs`、`calendar_tokens.rs`、`date_input_tokens.rs`、`avatar_tokens.rs`、`progress_tokens.rs`、`sparkline_tokens.rs`、移動した file。
- **状態の閉じ方**：新しい型は無い。
- **試験**：変更しない。
- **確認（tier a）**：U11 と同じ命令。test は `-- indicators feedback ui_components ui_chat_code ui_calendar ui_sparkline apg_date_picker interaction_calendar`。
- **完了条件**：規則 7 の `diff` が空。`ls crates/kivact-ui/src/look_*.rs` が `look_common.rs` だけ。`ls crates/kivact-ui/src/*_values.rs crates/kivact-ui/src/*_tokens.rs` が空。上の命令が通る。

### 3.2 W7a（lane U1 だけ）

## U20：`recipe_table!` を作り直し、部品ごとの `<Name>Theme` にして `UiRecipes` を消す（UI-03、UI-14、UI-24 (d)、API-19）

- **wave / lane / 前提**：W7a / U1 / W6 の鎖の完了（kit は compile の外）。W7a には他の ui・interaction の unit が無い。
- **所有**：`crates/kivact-ui/` 全体（`src/`、`Cargo.toml`）、`dev/tests/kivact-public-api/tests/e2e_ui/{indicators.rs, ui_chat_code.rs, ui_theming.rs}`、`examples/kivact/`、`templates/`、`dev/bench/` の中で `UiRecipes`・`*Recipes` を使う行（`rg -l "UiRecipes|\w+Recipes\b" examples/kivact templates dev/bench` で確かめる。今の HEAD では 0 件）、`crates/kivact-devtools/src/look.rs`（手順 11）。
- **目的**：recipe の表を部品ごとの `ThemeExtension` `<Name>Theme` に分け、一つの部品の既定値だけがその部品で作られるようにする（UI-03）。上書きは `Fn(&Theme, &Look<S>, Style) -> Style`（3 つ目は前の recipe の結果。UI-24 (d)）。`kivact::ui::theme` module を作り、kit 作者向けの型をそこへ移す（API-19）。
- **手順**：
  1. `crates/kivact-ui/src/recipe.rs` の `Recipe<S>` と macro を次で置き換える（`From<F>`・`Deref`・`named` を消す）。`DEFAULT` の static の型と path は、W1 の時点でこの macro が使っている遅延初期化の型（annex S の `SharedDeferred`）を**そのまま**使う。下の `kivact_core::cell::SharedDeferred` はその置き場の例ではなく、今の行の path に合わせて書く。
     ```rust
     //! Role: The recipe type and the macro that builds one component's recipe table.
     use crate::theme::Look;
     use kivact_core::style::{Style, Theme};
     use std::sync::Arc;

     type RecipeFn<S> = dyn Fn(&Theme, &Look<S>) -> Style + Send + Sync;

     /// A style derived from the theme and the actual state of a component's
     /// part, with the component's variants ([`Look`]). A theme changes it with
     /// the method of the part's name on the component's `<Name>Theme`, which
     /// receives the style the previous recipe returned.
     pub struct Recipe<S> {
         f: Arc<RecipeFn<S>>,
         name: &'static str,
         overridden: bool,
     }
     impl<S> Clone for Recipe<S> {
         fn clone(&self) -> Self {
             Self { f: self.f.clone(), name: self.name, overridden: self.overridden }
         }
     }
     impl<S: 'static> Recipe<S> {
         pub(crate) fn new(
             name: &'static str,
             f: impl Fn(&Theme, &Look<S>) -> Style + Send + Sync + 'static,
         ) -> Self {
             Self { f: Arc::new(f), name, overridden: false }
         }
         /// The style of the part in `theme` for `look`.
         pub fn style(&self, theme: &Theme, look: &Look<S>) -> Style {
             (self.f)(theme, look)
         }
         /// The recipe's name (`<Name>Theme.<part>`), shown by DevTools.
         pub fn name(&self) -> &'static str {
             self.name
         }
         /// Whether a theme replaced or wrapped the default recipe.
         pub(crate) fn is_overridden(&self) -> bool {
             self.overridden
         }
         pub(crate) fn layer(
             self,
             f: impl Fn(&Theme, &Look<S>, Style) -> Style + Send + Sync + 'static,
         ) -> Self {
             let previous = self.f;
             Self {
                 f: Arc::new(move |theme, look| f(theme, look, previous(theme, look))),
                 name: self.name,
                 overridden: true,
             }
         }
         pub(crate) fn same(&self, other: &Self) -> bool {
             Arc::ptr_eq(&self.f, &other.f)
         }
     }

     /// One component's recipe table: one recipe per part and the component's
     /// non-value settings. `<part>(f)` layers `f` over the current recipe;
     /// `<part>_recipe()` reads it; `<setting>(v)` replaces a setting and
     /// `<setting>_value()` reads it.
     macro_rules! recipe_table {
         (
             $(#[$meta:meta])*
             $name:ident {
                 $( $(#[$fmeta:meta])* $field:ident ($read:ident) : $state:ty = $default:path, )*
             }
             $( settings {
                 $( $(#[$smeta:meta])* $setting:ident ($sread:ident) : $sty:ty = $sdefault:expr, )*
             } )?
         ) => {
             $(#[$meta])*
             #[derive(Clone)]
             pub struct $name {
                 $( $field: $crate::recipe::Recipe<$state>, )*
                 $( $( $setting: $sty, )* )?
             }
             impl $name {
                 fn default_table() -> &'static Self {
                     static DEFAULT: kivact_core::cell::SharedDeferred<$name> =
                         kivact_core::cell::SharedDeferred::new();
                     DEFAULT.get_or_init(|| Self {
                         $( $field: $crate::recipe::Recipe::new(
                             concat!(stringify!($name), ".", stringify!($field)),
                             $default,
                         ), )*
                         $( $( $setting: $sdefault, )* )?
                     })
                 }
                 $(
                     $(#[$fmeta])*
                     ///
                     /// This reads the recipe.
                     pub fn $read(&self) -> &$crate::recipe::Recipe<$state> {
                         &self.$field
                     }
                     $(#[$fmeta])*
                     ///
                     /// This layers `f` over the current recipe: `f` gets the
                     /// theme, the look and the style the current recipe returns.
                     pub fn $field(
                         mut self,
                         f: impl Fn(&kivact_core::style::Theme, &$crate::theme::Look<$state>, kivact_core::style::Style)
                             -> kivact_core::style::Style + Send + Sync + 'static,
                     ) -> Self {
                         self.$field = self.$field.layer(f);
                         self
                     }
                 )*
                 $( $(
                     $(#[$smeta])*
                     ///
                     /// This reads the setting.
                     pub fn $sread(&self) -> &$sty {
                         &self.$setting
                     }
                     $(#[$smeta])*
                     ///
                     /// This replaces the setting.
                     pub fn $setting(mut self, value: $sty) -> Self {
                         self.$setting = value;
                         self
                     }
                 )* )?
             }
             impl Default for $name {
                 fn default() -> Self {
                     Self::default_table().clone()
                 }
             }
             impl PartialEq for $name {
                 fn eq(&self, other: &Self) -> bool {
                     true $( && self.$field.same(&other.$field) )*
                         $( $( && self.$setting == other.$setting )* )?
                 }
             }
             impl std::fmt::Debug for $name {
                 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                     f.debug_struct(stringify!($name))
                         $( .field(stringify!($field), &self.$field.name()) )*
                         $( $( .field(stringify!($setting), &self.$setting) )* )?
                         .finish()
                 }
             }
             impl kivact_core::style::ThemeExtension for $name {
                 fn fallback(_: &kivact_core::style::Theme) -> Self {
                     Self::default()
                 }
             }
         };
     }
     pub(crate) use recipe_table;
     ```
     `tables { .. }` の節は無くなる（部品の表は互いを含まない）。
  2. 表の型名を改名する（`UiRecipes` 以外の `*Recipes` → `*Theme`）。ui・試験・例に同じ命令をかける：
     ```
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     rg -l --path-separator // "\b[A-Z]\w*Recipes\b" crates/kivact-ui/src dev/tests/kivact-public-api/tests examples/kivact templates \
       | xargs perl -pi -e 's/\b(?!UiRecipes\b)([A-Z]\w*)Recipes\b/$1Theme/g'
     ```
     `e2e_kits` の file（`dev/tests/kivact-kit-tests/tests/e2e_kits/` の `theme_conformance.rs`、`ui_standard.rs`、`presence_kit.rs`、`apg_grid_kit.rs`）は置換の対象に入らない（annex K が直す）。
  3. 平らな field の表を新設する。各 `look.rs` の末尾に次の表を足す（default の関数は U11〜U14 でその `look.rs` に移っている）。`UiRecipes` から読んでいた `button_recipe()` などは、右の列の読み方になる。

| 置く file | 新しい表 | 部品の行（`part(read): State = default`） | 旧 `UiRecipes` の field |
|---|---|---|---|
| `button/look.rs` | `ButtonTheme` | `root(root_recipe): ButtonState = button`、`content(content_recipe): ButtonState = button_content`、`busy(busy_recipe): ButtonState = button_busy`、`icon(icon_recipe): ButtonState = button_icon` | `button`、`button_content`、`button_busy`、`button_icon` |
| `toggle/look.rs` | `CheckboxTheme` | `root(root_recipe): CheckableState = checkbox`、`indicator(indicator_recipe): CheckableState = empty_toggle_part`、`check(check_recipe): CheckableState = empty_toggle_part`、`mixed(mixed_recipe): CheckableState = empty_toggle_part` | `checkbox`、`checkbox_indicator`、`checkbox_check`、`checkbox_mixed` |
| `toggle/look.rs` | `SwitchTheme` | `root(root_recipe): CheckableState = switch`、`track(track_recipe): CheckableState = empty_toggle_part`、`thumb(thumb_recipe): CheckableState = empty_toggle_part` | `switch`、`switch_track`、`switch_thumb` |
| `toggle_button/look.rs` | `ToggleButtonTheme` | `root(root_recipe): ToggleButtonState = toggle_button` | `toggle` |
| `text_input/look.rs` | `TextInputTheme` | `root(root_recipe): TextInputState = text_input` | `text_input` |
| `text_input/look.rs` | `TextAreaTheme` | `root(root_recipe): TextInputState = text_area` | `text_area` |
| `heading/look.rs` | `HeadingTheme` | `root(root_recipe): HeadingState = heading` | `heading` |
| `avatar/look.rs` | `AvatarTheme` | `root(root_recipe): AvatarState = avatar`、`image(image_recipe): AvatarState = avatar_image`、`status(status_recipe): AvatarState = avatar_status`、`status_content(status_content_recipe): AvatarState = avatar_status_content` | `avatar`、`avatar_image`、`avatar_status`、`avatar_status_content` |
| `avatar_group/look.rs` | `AvatarGroupTheme` | `root(root_recipe): () = avatar_group`、`member(member_recipe): usize = avatar_group_member`、`overflow(overflow_recipe): usize = avatar_group_overflow` | `avatar_group`、`avatar_group_member`、`avatar_group_overflow` |
| `kbd/look.rs` | `KbdTheme` | `root(root_recipe): () = kbd`、`key(key_recipe): KbdKeyState = kbd_key` | `kbd`、`kbd_key` |
| `badge/look.rs` | `BadgeTheme` | `root(root_recipe): BadgeState = badge`、`label(label_recipe): BadgeState = badge_label` | `badge`、`badge_label` |
| `spinner/look.rs` | `SpinnerTheme` | `root(root_recipe): SpinnerState = spinner` | `spinner` |
| `skeleton/look.rs` | `SkeletonTheme` | `root(root_recipe): SkeletonState = skeleton` | `skeleton` |

     既存の look 関数の型 `fn x(theme: &Theme, state: &ButtonState) -> Style`（`Look` を取らないもの、例：`button_content`、`button_icon`）は、`fn x(theme: &Theme, state: &Look<ButtonState>) -> Style` に直す（`Look` は `Deref<Target = S>` なので本体は変えない）。default は `Recipe::new` に `$default` の path として渡るので、型が `Fn(&Theme, &Look<S>) -> Style` に合う必要がある。
  4. `crates/kivact-ui/src/recipes.rs` を消す（`UiRecipes`、その `impl ThemeExtension`、`has_checkbox_parts`、`has_switch_parts` も消える）。`lib.rs` の `mod recipes;` と、`pub use recipes::{..}` の行をすべて消す。
  5. 既存の `impl RadioGroupTheme { has_custom_mark }` と `impl TabsTheme { has_custom_indicator }`（U13 で look.rs に移したもの）の本体を、`self.mark.is_overridden()`・`self.indicator.is_overridden()` にする。`toggle/look.rs` に次を足す：
     ```rust
     impl CheckboxTheme {
         /// Whether a theme gave the checkbox its own indicator, check or mixed parts.
         pub(crate) fn has_parts(&self) -> bool {
             self.indicator.is_overridden() || self.check.is_overridden() || self.mixed.is_overridden()
         }
     }
     impl SwitchTheme {
         /// Whether a theme gave the switch its own track or thumb.
         pub(crate) fn has_parts(&self) -> bool {
             self.track.is_overridden() || self.thumb.is_overridden()
         }
     }
     ```
  6. `crates/kivact-ui/src/layer.rs` を次で置き換える（`recipes` 関数を消す）：
     ```rust
     //! Role: Apply theme recipe styles before caller overrides without extra nodes.
     use crate::recipe::Recipe;
     use crate::variants::Variants;
     use kivact_core::style::{ThemeExtension, use_theme};
     use kivact_core::view::Element;
     use std::rc::Rc;

     /// `root` styled by the recipe `pick` chooses from the component's theme
     /// table `T` in the theme in effect, given `state` and the component's
     /// `variants`, under the caller's later styles.
     pub(crate) fn with_recipe_style<N: Element, T: ThemeExtension + Default, S: 'static>(
         root: N,
         pick: fn(&T) -> &Recipe<S>,
         variants: &Variants,
         state: impl Fn() -> S + 'static,
     ) -> N {
         let variants = variants.clone();
         let name = pick(&T::default()).name();
         root.recipe_style(Rc::new(move || name), move || {
             let theme = use_theme().get();
             pick(&theme.ext::<T>()).style(&theme, &variants.look(state()))
         })
     }

     /// [`with_recipe_style`] for a builder whose state is known only once it
     /// is complete: the recipe is the root's base style, below every
     /// `.style(..)` of the caller whatever the order of the calls.
     pub(crate) fn with_base_recipe_style<N: Element, T: ThemeExtension + Default, S: 'static>(
         root: N,
         pick: fn(&T) -> &Recipe<S>,
         variants: &Variants,
         state: impl Fn() -> S + 'static,
     ) -> N {
         let variants = variants.clone();
         let name = pick(&T::default()).name();
         root.with_node(|node| {
             node.base_recipe_style(Rc::new(move || name), move || {
                 let theme = use_theme().get();
                 pick(&theme.ext::<T>()).style(&theme, &variants.look(state()))
             })
         })
     }
     ```
     `use_theme().get()` と `recipe_style`・`base_recipe_style` の名前は W5・W6 の後の形に合わせる（今の行と同じ役割の呼出し）。`Variants` の path は U24 で core に変わる。
  7. 読み手の closure を機械的に書き換える。次の perl を `crates/kivact-ui/src` の全 `.rs` に一度だけかける：
     ```
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     cat > target/u20_readers.pl <<'EOF'
     my %flat = (
       button => ['Button','root'], button_content => ['Button','content'], button_busy => ['Button','busy'],
       button_icon => ['Button','icon'], checkbox => ['Checkbox','root'], checkbox_indicator => ['Checkbox','indicator'],
       checkbox_check => ['Checkbox','check'], checkbox_mixed => ['Checkbox','mixed'], switch => ['Switch','root'],
       switch_track => ['Switch','track'], switch_thumb => ['Switch','thumb'], toggle => ['ToggleButton','root'],
       text_input => ['TextInput','root'], text_area => ['TextArea','root'], heading => ['Heading','root'],
       avatar => ['Avatar','root'], avatar_image => ['Avatar','image'], avatar_status => ['Avatar','status'],
       avatar_status_content => ['Avatar','status_content'], avatar_group => ['AvatarGroup','root'],
       avatar_group_member => ['AvatarGroup','member'], avatar_group_overflow => ['AvatarGroup','overflow'],
       kbd => ['Kbd','root'], kbd_key => ['Kbd','key'], badge => ['Badge','root'], badge_label => ['Badge','label'],
       spinner => ['Spinner','root'], skeleton => ['Skeleton','root'],
     );
     sub camel { join '', map { ucfirst } split /_/, $_[0] }
     local $/; $_ = <>;
     s{\|(\w+)\|\s*\1\s*\.\s*(\w+)_recipes\(\)\s*\.\s*(\w+)_recipe\(\)}{"|r: &".camel($2)."Theme| r.$3_recipe()"}ge;
     s{\|(\w+)\|\s*\1\s*\.\s*(\w+)_recipe\(\)}{exists $flat{$2} ? "|r: &$flat{$2}[0]Theme| r.$flat{$2}[1]_recipe()" : $&}ge;
     print;
     EOF
     for f in $(rg -l --path-separator // "_recipe\(\)" crates/kivact-ui/src); do perl target/u20_readers.pl "$f" > target/u20.tmp && mv target/u20.tmp "$f"; done
     ```
     その後 `rg -n "_recipes\(\)" crates/kivact-ui/src` が空であること。閉じた表の型（`ButtonTheme` など）を各 file の `use` に足す（`use crate::button::look::ButtonTheme;` のように。compile の誤りで分かる）。
  8. perl で直らない読み手を手で直す（今の file の中の引用で探す）：
     - `toggle/parts.rs`：`|| recipes(&use_theme().get()).has_checkbox_parts()` → `|| use_theme().get().ext::<CheckboxTheme>().has_parts()`、`has_switch_parts()` も同様に `ext::<SwitchTheme>().has_parts()`。`use super::layer::{recipes, ..}` から `recipes` を消す。
     - `radio_group.rs`：`recipes(&use_theme().get()).radio_group_recipes().has_custom_mark()` → `use_theme().get().ext::<RadioGroupTheme>().has_custom_mark()`。
     - `tabs.rs`：`recipes(&use_theme().get()).tabs_recipes().has_custom_indicator()` → `use_theme().get().ext::<TabsTheme>().has_custom_indicator()`。
     - `text_input.rs`：`let recipes = recipes(&theme); let recipe = if multiline { recipes.text_area_recipe() } else { recipes.text_input_recipe() }; recipe(&theme, &look.look(..))` → 
       ```rust
       let look = look.look(TextInputState { /* 今と同じ field */ });
       if multiline {
           theme.ext::<TextAreaTheme>().root_recipe().style(&theme, &look)
       } else {
           theme.ext::<TextInputTheme>().root_recipe().style(&theme, &look)
       }
       ```
     - `select.rs`：`pick(&crate::layer::recipes(&theme))(&theme, &variants.look(state.get()))` を含む関数の引数 `pick: fn(&UiRecipes) -> &Recipe<S>` を `pick: fn(&SelectTheme) -> &Recipe<S>` にし、本体を `pick(&theme.ext::<SelectTheme>()).style(&theme, &variants.look(state.get()))` にする。
     - 他に `rg -n "UiRecipes|recipes\(&" crates/kivact-ui/src` で出る行を同じ形（`theme.ext::<XTheme>().<part>_recipe().style(&theme, &look)`）で直す。recipe を関数として呼んでいる `(&theme, &look)` の形（`Deref` の利用）は `.style(&theme, &look)` にする。
  9. `crates/kivact-ui/src/theme.rs` を新設し、`lib.rs` に `pub mod theme;` を足す。
     ```rust
     //! Role: The kit-author contract of the standard components: recipe tables, recipe states and theme variables.
     //!
     //! Every standard component has one `<Name>Theme` (a recipe table, a
     //! [`ThemeExtension`](kivact_core::style::ThemeExtension)) and the state
     //! its recipes read. A kit changes a component's look with
     //! `theme.with(ButtonTheme::default().root(|_theme, _look, base| base.border_radius(0)))`;
     //! the third argument is the style of the recipe it replaces. A
     //! component's values (heights, colors, durations) are theme variables
     //! in the module of the component's name (`theme::button::BUSY_SPIN_MS_VAR`).
     pub use crate::recipe::Recipe;
     pub use crate::variants::{Look, Variant};
     ```
     その後に部品ごとの `pub use` を足す：U11〜U14 が `lib.rs` から再公開した「look」の型（`*State`、`*Theme`、状態の中だけで使う型：`CheckControl`、`ThumbDrag`、`TrackInput`、`Lift`、`Expansion`、`Selectable`、`DisclosureGroupPosition`、`CalendarDayVisual`、`CalendarEventVisual`、`ItemRowState`、`TreeRowRecipeState`、`DataTableCheckState`、`SelectSheetOption`、設定の型 `ButtonBusyMotion`、`DisclosureMotion`、`ProgressMotion`、`ToastRowMotion`、`SparklineFill`、`ToggleLabelReservation`、`DialogCenter`、`DataTableStickyPaint`、`DataTableRowStatusPaint`、`DataTableTreeIndent`）を、`lib.rs` の `pub use` から消して `theme.rs` の `pub use crate::<name>::look::{..};` に移す。U21 で消える型（`*Tokens`、`ButtonBusyMotion` など）もこの時点では移す。`lib.rs` の root に残すのは、部品の関数・builder の型・利用者が値として渡す enum と値の型（`ButtonVariant`、`ControlSize`、`AlertVariant`、`ProgressVariant`、`RadioGroupVariant`、`ChatMessageVariant`、`*Shape`、`PageItem`、`StepStatus`、`ToolbarEdge`、`CarouselMode` など）と `provide_control_size` だけ。
  11. `crates/kivact-devtools/src/look.rs`（付属書 K の K01 が W2a で作った DevTools の private theme）：
      `.with(UiRecipes::default().tabs(|tabs| tabs.list(|_| tab_list).tab(|_| tab)))` を
      `.with(TabsTheme::default().list(|theme, look, _| tab_list(theme, look)).tab(|theme, look, _| tab(theme, look)))` にする（前の recipe の結果は使わない。`tab_list`・`tab` の本体は変えない）。`use` の `UiRecipes` を消し、`TabsTheme`・`Look`・`TabListState`・`TabState` は手順 9 の `kivact::ui::theme` から読む（compile error が示す path に合わせる）。
  10. 試験を新しい形にする（期待値は変えない）。
      - `indicators.rs::skeleton_stops_with_reduced_motion`：
        ```rust
        let recipes = ui::theme::SkeletonTheme::default().root(move |_theme, state, base| {
            recorded.lock().expect("invariant: sample lock is healthy").push(state.pulse);
            base
        });
        ```
      - `ui_chat_code.rs::code_block_numbers_follow_native_wrapped_rows_and_recipe_line_height`：
        `kivact::ui::UiRecipes::default().code_block(|r| r.code(|previous| move |theme, state| previous(theme, state).font_size(18).line_height(LineHeight::px(31))))` →
        `kivact::ui::theme::CodeBlockTheme::default().code(|_theme, _state, base| base.font_size(18).line_height(LineHeight::px(31)))`。
      - `ui_theming.rs`：`use kivact::ui::{ButtonState, UiRecipes};` → `use kivact::ui::theme::{ButtonState, ButtonTheme, Look, SwitchTheme};`。`a_switch_thumb_recipe_animates_when_the_binding_changes` の `UiRecipes::default().switch(|_| |_, _| X).switch_track(|_| |_, _| Y).switch_thumb(|_| |_, state| Z)` → `SwitchTheme::default().root(|_, _, _| X).track(|_, _, _| Y).thumb(|_, state, _| Z)`（X・Y・Z は今の式のまま）。`a_librarys_tokens_are_read_again_on_a_theme_switch_without_a_remount` の `UiRecipes::default().button(|prev| move |theme: &Theme, state: &Look<ButtonState>| prev(theme, state).background(theme.ext::<Brand>().fill))` → `ButtonTheme::default().root(|theme: &Theme, _state: &Look<ButtonState>, base| base.background(theme.ext::<Brand>().fill))`。他の `UiRecipes` の行も同じ規則で直す。
- **消すもの**：`UiRecipes`、`recipes.rs`、`Recipe::named`、`impl From<F> for Recipe<S>`、`impl Deref for Recipe<S>`、`layer::recipes`、`has_checkbox_parts`、`has_switch_parts`、すべての `*Recipes` の名前（`*Theme` になる）、`recipe_table!` の `tables { .. }` の節。
- **状態の閉じ方**：`Recipe<S>` は `bool`（`overridden`）を一つだけ持ち、constructor と値渡しの `layer` だけが書く（6.1 節の対象外）。各 `<Name>Theme` は接尾辞 `Theme` で対象外。
- **試験**：上の 3 file の 4 関数を書き換える（期待値は変えない）。足す試験：`ui_theming.rs` に次を足す（5.9 節の上書きの契約：前の recipe の結果が 3 つ目の引数で来ること）。
  ```rust
  #[test]
  fn a_recipe_override_receives_the_previous_recipes_style() {
      let theme = Theme::light()
          .with(ButtonTheme::default()
              .root(|_, _, base| base.background(rgb(RED)))
              .root(|_, _, base| base.width(77)));
      let screen = kivact::test::render_with(
          kivact::test::RenderOptions::default().theme(theme),
          || button("Go").test_id("go"),
      );
      let go = screen.find_by_test_id("go").expect("invariant: the button is mounted");
      assert_eq!(go.bounds().width, 77.0);
      assert!(distance(at(&screen, "go", 4.0, 4.0), RED) <= 8);
      assert!(screen.errors().is_empty(), "{:?}", screen.errors());
  }
  ```
  （`render_with`・`find_by_test_id`・`bounds` は `ui_theming.rs` と `support` が今使っている名前に合わせる。`at` と `distance` は同じ file の helper。）
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-ui
  cargo check -p kivact-ui --all-targets
  cargo clippy -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-ui
  cargo check -p kivact --all-targets
  cargo check -p kivact-devtools --all-targets
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_theming indicators ui_chat_code ui_controls ui_collections
  ```
- **完了条件**：`rg -w "UiRecipes" crates/kivact-ui crates/kivact crates/kivact-devtools examples/kivact templates dev/tests/kivact-public-api/tests --glob '!theme_conformance.rs' --glob '!ui_standard.rs' --glob '!presence_kit.rs' --glob '!apg_grid_kit.rs'` が空。`rg -n "\b\w+Recipes\b" crates/kivact-ui/src` が空。`ls crates/kivact-ui/src/recipes.rs` が無い。上の命令が通る。

## U21：`UiTokens` と部品の `*Tokens` を theme 変数と `<Name>Theme` の設定にする（UI-03、DEC-06）

- **wave / lane / 前提**：W7a / U1 / U20
- **所有**：`crates/kivact-ui/` 全体、`dev/tests/kivact-public-api/tests/` の中で `UiTokens`・`*Tokens` を使う e2e_ui の file（`rg -l "UiTokens|\w+Tokens\b" dev/tests/kivact-public-api/tests` で確かめる。今の HEAD では kit の file 以外に無い）。
- **目的**：部品の値（高さ、色、時間）を theme 変数 `ThemeVar<T>` にし、値ではない設定（enum、bool、`Transition`、`Easing`、文字列の key）を `<Name>Theme` の `settings { .. }` にする（DEC-06）。`UiTokens` と部品の Tokens の型をすべて消す。
- **手順**：
  1. 変数の書き方（4.3 節の表の各行をこの形で書く）。部品の `look.rs` の `use` 群の後に置く：
     ```rust
     use kivact_core::style::{Color, Length, ThemeVar};
     /// The height of the combobox field.
     pub static FIELD_HEIGHT_VAR: ThemeVar<f64> = ThemeVar::new("combobox.field_height", 32.0);
     /// [`FIELD_HEIGHT_VAR`] as a length for a style.
     pub const FIELD_HEIGHT: Length = Length::var(&FIELD_HEIGHT_VAR);
     /// The text color of the combobox.
     pub static FOREGROUND_VAR: ThemeVar<Color> = ThemeVar::alias("combobox.foreground", &color::TEXT_VAR);
     /// [`FOREGROUND_VAR`] as a color for a style.
     pub const FOREGROUND: Color = Color::var(&FOREGROUND_VAR);
     /// The busy spin period in milliseconds.
     pub static BUSY_SPIN_MS_VAR: ThemeVar<f64> = ThemeVar::new("button.busy_spin_ms", 700.0);
     ```
     - 規則：`f64` の大きさ（px）は `<NAME>_VAR`（`ThemeVar<f64>`）と `<NAME>`（`Length`）の 2 つ。色は `<NAME>_VAR`（`ThemeVar<Color>`）と `<NAME>`（`Color`）の 2 つ。時間（ms）と数（倍率、透明度、行数でない数）は `<NAME>_VAR` だけ。
     - 変数の文字列名は `"<部品>.<field 名（snake_case）>"`（付属書 S 4.1 節の形：`"button.height_md"`）。変数は必ず `static`（`const` にしない。S 4.1 節）。
     - fallback は、W6 の後の Tokens の `impl Default` に書いてある値をそのまま写す（W6 が `ColorToken::X` を `color::X` に直しているので、色が中核の色なら `ThemeVar::alias(name, &color::X_VAR)`、`Color::TRANSPARENT` などの固定の色は `ThemeVar::new(name, Color::TRANSPARENT)`）。**旧い token が theme の共通の値に従っていたもの**（`Option<f64>` で `None` のとき `unwrap_or(<中核の値>)` を使っていたもの、`fallback(theme)` が palette から作っていたもの）は、fallback を**中核の変数への別名**にする（12 章 UI-U-02。設計 5.7：どの型の変数も fallback を別の変数にできる）。f64 の変数の別名は付属書 S 4.1 の `ThemeVar::alias("combobox.field_radius", &radius::MD_VAR)` で書く。旧い `control_height`・`disabled_opacity`・`overlay_margin` に従っていたものは `control::HEIGHT_VAR`・`control::DISABLED_OPACITY_VAR`・`control::OVERLAY_MARGIN_VAR` への別名、`focus_ring` に従っていたものは数（2）にする。`unwrap_or(16.0)` のような固定の数は数のまま。4.3 節の表の「fallback」の列が正本である。
     - `ThemeVar`・`Length::var`・`Color::var`・`color::*` の path は annex S が W6 で置いた path を使う（`rg -n "pub static ACCENT_VAR" crates/kivact-style` で場所を確かめる）。
  2. 読み手を書き換える。
     - style の setter にそのまま渡している所（`.height(tokens.field_height)`）は定数（`.height(FIELD_HEIGHT)`）にする。
     - 計算に使う所（`tokens.cell_size * 7.0`、`Duration::from_millis(motion.spin_ms)`）は `theme.get(&CELL_SIZE_VAR)`（`f64`）にする。`u64` が要る所は `theme.get(&BUSY_SPIN_MS_VAR) as u64`。theme は今その行が読んでいる theme（recipe の `t`、`use_theme().get()` の値）を使う。
     - 設定は `theme.ext::<XTheme>().<setting>_value()` で読む。
     - `t.ext::<UiTokens>()`・`use_tokens::<UiTokens>()` の行は、4.3 節の表の旧 field から新しい名前を引いて直す。`rg -n "UiTokens" crates/kivact-ui/src` の全行が対象。
  3. 設定の節を表に足す（U20 で作った表に `settings { .. }` を加える）。4.3 節の表の「設定」の行がそのまま入る。例：
     ```rust
     crate::recipe::recipe_table! {
         /// The recipes and settings of a [`dialog`](crate::dialog).
         DialogTheme {
             surface(surface_recipe): DialogState = dialog_surface,
             // ..（U20 のまま）
         }
         settings {
             /// Where a dialog centers itself: on its anchor's window or the screen's.
             center(center_value): DialogCenter = DialogCenter::Anchor,
         }
     }
     ```
  4. `crates/kivact-ui/src/tokens.rs` を消し、`lib.rs` の `mod tokens; pub use tokens::UiTokens;` を消す。部品の Tokens の型（4.3 節の表の「消す型」）を消す。設定の型として残る型（`DialogCenter`、`DisclosureMotion`、`ProgressMotion`、`SparklineFill`、`ToggleLabelReservation`）は `theme.rs` の再公開に残す。
  5. `theme.rs` に部品ごとの module を足し、その部品の変数をすべて再公開する。例：
     ```rust
     /// The combobox's theme variables.
     pub mod combobox {
         pub use crate::combobox::look::{
             ACTIVE_BACKGROUND, ACTIVE_BACKGROUND_VAR, BACKGROUND, BACKGROUND_VAR, /* .. 4.3 節の行すべて */
         };
     }
     ```
     変数を持たない部品には module を作らない。
- **消すもの**：`UiTokens`、`tokens.rs`、`ButtonTokens`、`ButtonBusyMotion`、`CalendarTokens`、`CarouselTokens`、`ChatTokens`、`CodeBlockTokens`、`ComboboxTokens`、`DataTableTokens`、`DataTableStickyPaint`、`DataTableRowStatusPaint`、`DateInputTokens`、`DialogTokens`、`DisclosureTokens`、`DropzoneTokens`、`NavigationTokens`、`StepperTokens`、`ProgressTokens`、`SparklineTokens`、`ToastTokens`、`ToastRowMotion`、`ToggleButtonTokens`、`AvatarTokens`、`IndicatorTokens`、`ToolbarTokens`、`TreeTokens`、`ItemListTokens`、`TableTokens`、`ButtonGroupTokens`、`ToggleGroupTokens`、`HoverCardTokens`。`use_tokens::<UiTokens>()` の呼出し（`use_tokens` そのものは annex S が W6 で消している）。
- **状態の閉じ方**：新しい状態の型は無い。`ThemeVar` の static は不変。表は接尾辞 `Theme`。
- **試験**：変更しない（e2e_ui の file は `UiTokens` を使っていない）。kit の試験の `UiTokens` は 4.4 節の表で annex K が直す。
- **確認（tier a）**：U20 と同じ命令。test は `-- ui_theming ui_controls ui_combobox ui_calendar ui_data_table ui_toolbar ui_sparkline ui_carousel indicators interaction_toast`。
- **完了条件**：`rg -n "\bUiTokens\b|\b[A-Z]\w*Tokens\b" crates/kivact-ui/src` が空。4.3 節の表の新しい名前がすべて `kivact_ui::theme::<部品>::` で引ける（`cargo doc -p kivact-ui --no-deps` の `target/doc/kivact_ui/theme/` の中に表の部品の directory がある）。上の命令が通る。

## U22：入力部品の見えない名前を `.label()` にする（DEC-05、API-17、UI-13 (a) の tag_input）

- **wave / lane / 前提**：W7a / U1 / U25（W7a の順は U20 → U21 → U25 → U22 → U23 → U24）
- **所有**：`crates/kivact-ui/` 全体、`crates/kivact/src/`（doc の例と prelude の行）、`crates/kivact-devtools/src/`、`crates/kivact-test/src/`（使う所があれば）、`dev/tests/kivact-public-api/tests/`（e2e_core・e2e_ui・e2e_native の file。kit の 4 file は除く）、`dev/bench/`、`examples/kivact/`、`templates/`。
- **目的**：見えない名前を持つ入力部品は名前を位置引数で取らない：`text_input(value).label("Name")`。`field("Name", control)` が control に名前を付ける。名前の無い focus できる入力は mount のときに位置付きの診断を出す（debug build、したがって `kivact::test` では error）。見える label を持つ部品（`button`、`checkbox`、`switch`、`toggle_button`、`radio_group` の項目）と、group の名前を位置引数に持つ collection（`list`、`tabs`、`tree`、`radio_group`、`toggle_group`、`button_group`、`input_group`、`splitter`）は変えない。
- **手順**：
  1. 対象の 12 関数の位置引数 `label: impl IntoText` を消し、builder に `label` を足す。新しい署名（W5 の後の型で書く）：
     ```rust
     pub fn text_input(value: Signal<String>) -> TextInput
     pub fn text_area(value: Signal<String>) -> TextInput
     pub fn select<K: ItemKey>(items: impl IntoValue<Vec<ListItem<K>>>, selected: Signal<Option<K>>) -> Select<K>
     pub fn combobox<K: ItemKey>(items: impl IntoValue<Vec<ListItem<K>>>, selected: impl IntoSelection<K>) -> Combobox<K>
     pub fn slider(value: Signal<f64>) -> Slider
     pub fn range_slider(low: Signal<f64>, high: Signal<f64>) -> Slider
     pub fn number_input(value: Signal<f64>, range: impl IntoValue<ScalarRange>) -> NumberInput
     pub fn date_input(value: Signal<Option<CalendarDate>>) -> DateInput
     pub fn date_range_input(range: Signal<Option<DateRange>>) -> DateRangeInput
     pub fn date_time_input(value: Signal<Option<DateTime>>) -> DateTimeInput
     pub fn time_input(value: Signal<Option<TimeOfDay>>) -> TimeInput
     pub fn tag_input(tags: Signal<Vec<String>>) -> TagInput
     ```
     各 builder に次の method を足す（`X` は builder の型。名前を付ける node は、今 `label` を `.name(label)` にしている node と同じ）：
     ```rust
     /// The control's accessible name, read by assistive technology and not
     /// drawn. Inside a [`field`](crate::field) the field's label names it.
     #[track_caller]
     pub fn label(mut self, label: impl IntoText) -> Self {
         self.<名前を付けていた node の field> = self.<同じ field>.name(label);
         self
     }
     ```
     今 `label` の値を別の用途（picker の title の既定、placeholder など）にも使っている builder は、その用途を `root.accessible_name()`（`into_view` の時点で読む）に置き換える。
  2. 名前の欠落の診断。12 の builder の `into_view`（または `finish`）の先頭で、名前を付ける node について次を行う。constructor に `#[track_caller]` を付け、`at: &'static Location<'static>` を builder の field に持つ。
     ```rust
     if cfg!(debug_assertions)
         && node.accessible_name().is_none()
         && node.accessible_labelled_by().is_empty()
     {
         kivact_core::custom::Owner::current().report(
             kivact_core::ErrorKind::Message,
             format!("{} at {} has no accessible name: call .label(..) or put it in field(..)", "<関数名>", at),
         );
     }
     ```
     （`ErrorKind` の path は W5 の後の `kivact_core` の再公開に合わせる。）`field(label, control)` は今どおり control に `labelled_by` を付けるので、field の中の control は診断を出さない。
  3. `tag_input` の `tags` は `Signal<Vec<String>>` を位置引数で受け、追加と削除はその signal を書く。`TagInput::on_add`・`on_remove` は U23 で消す（この unit では、内部で `on_add`・`on_remove` を呼んでいた所を `tags.update(|v| v.push(t))`・`tags.update(|v| { v.remove(i); })` に置き換え、method は残す）。
  4. 呼出し元を一度だけ次の perl で直す（**二度かけない**。二度目は 2 つ目の引数を label と見誤る）。対象：`crates/kivact-ui/src`、`crates/kivact/src`、`crates/kivact-devtools/src`、`crates/kivact-test/src`、`dev/tests/kivact-public-api/tests`（kit の 4 file を除く）、`dev/bench`、`examples/kivact`、`templates`。
     ```
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     cat > target/u22_label.pl <<'EOF'
     # Move the first argument of the named-input constructors into .label(..).
     use strict; use warnings;
     my $names = qr/text_input|text_area|select|combobox|slider|range_slider|number_input|date_input|date_range_input|date_time_input|time_input|tag_input/;
     local $/; my $s = <>;
     my $out = ''; my $pos = 0;
     while ($s =~ /(?<![\w.])(?<!fn )(?:(?:ui|kivact::ui)::)?($names)\(/g) {
         my $open = $+[0]; my $start = $-[0];
         my ($depth, $i, $str, $first_comma) = (1, $open, 0, -1);
         while ($i < length($s) && $depth > 0) {
             my $c = substr($s, $i, 1);
             if ($str) { if ($c eq '\\') { $i += 2; next } $str = 0 if $c eq '"'; }
             elsif ($c eq '"') { $str = 1 }
             elsif ($c =~ /[(\[{]/) { $depth++ }
             elsif ($c =~ /[)\]}]/) { $depth-- }
             elsif ($c eq ',' && $depth == 1 && $first_comma < 0) { $first_comma = $i }
             $i++;
         }
         next if $first_comma < 0;
         my $close = $i - 1;
         my $label = substr($s, $open, $first_comma - $open); $label =~ s/^\s+|\s+$//g;
         my $rest = substr($s, $first_comma + 1, $close - $first_comma - 1); $rest =~ s/^\s+//; $rest =~ s/,\s*$//s;
         $out .= substr($s, $pos, $open - $pos) . $rest . ").label(" . $label . ")";
         $pos = $close + 1;
         pos($s) = $pos;
     }
     $out .= substr($s, $pos);
     print $out;
     EOF
     for f in $(rg -l --path-separator // "\b(text_input|text_area|select|combobox|slider|range_slider|number_input|date_input|date_range_input|date_time_input|time_input|tag_input)\(" crates/kivact-ui/src crates/kivact/src crates/kivact-devtools/src crates/kivact-test/src dev/tests/kivact-public-api/tests dev/bench examples/kivact templates --glob '!theme_conformance.rs' --glob '!ui_standard.rs' --glob '!presence_kit.rs' --glob '!apg_grid_kit.rs'); do perl target/u22_label.pl "$f" > target/u22.tmp && mv target/u22.tmp "$f"; done
     ```
     その後、手で確かめて直す：
     - 関数の定義（`pub fn select<K..>(`）は `fn ` の後なので変わらない。`use` の行の `select,` などは `(` が続かないので変わらない。
     - 同名の別の関数・method への誤爆を探す：`git diff --stat` の file を開き、`.label(` を足された行のうち、上の 12 の ui の部品でない呼出し（独自の `fn select(..)`、`Selection::select(` など）を元に戻す。`rg -n "\.select\(|fn select\b" -g '*.rs' crates/kivact-ui/src` で候補を確かめる。
     - doc comment の中の例（`/// text_input("Name", name)`）も perl が直す。直らなかった（複数行に分かれた）例は手で直す。
  5. `crates/kivact-ui/src/lib.rs` の先頭の doc の例 `text_input("Name", name).placeholder("Your name")` → `text_input(name).label("Name").placeholder("Your name")`（perl が直していれば確かめるだけ）。
- **消すもの**：12 関数の位置引数 `label`。
- **状態の閉じ方**：builder に足す `at: &'static Location<'static>` は constructor だけが書く（6.1 節の対象外）。
- **試験**：perl が直した試験の期待値は変えない。足す試験（`ui_field.rs`。5.9 節の契約）：
  ```rust
  #[test]
  fn a_text_input_without_a_name_reports_an_error_and_a_field_names_it() {
      let (screen, _) = crate::support::mount(|| {
          let a = signal(String::new());
          let b = signal(String::new());
          ((), column((text_input(a).test_id("bare"), field("Email", text_input(b).test_id("named")))))
      });
      assert_eq!(screen.errors().len(), 1, "{:?}", screen.errors());
      assert!(format!("{:?}", screen.errors()[0]).contains("has no accessible name"));
      let named = screen.find_by_test_id("named").expect("invariant: mounted");
      assert_eq!(named.name().as_deref(), Some("Email"));
  }
  ```
  （`mount`・`find_by_test_id`・`name()` は `ui_field.rs` が今使う support の名前に合わせる。）
- **確認（tier a）**：
  ```
  cargo fmt --all
  cargo check -p kivact-ui -p kivact -p kivact-devtools -p kivact-public-api-tests --all-targets
  cargo clippy -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-ui
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_field ui_controls ui_combobox ui_calendar apg_combobox apg_slider apg_spinbutton apg_date_picker interaction_edit interaction_slider
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- text_editing
  ```
- **完了条件**：`rg -n "text_input\(\s*\"" crates/kivact-ui crates/kivact dev/tests examples/kivact templates --glob '!theme_conformance.rs' --glob '!ui_standard.rs'` が空。新しい試験が通る。上の命令が通る。

## U23：その他の呼出しの形の変更（UI-13、UI-19、UI-20 の ui 側、API-13、API-14、API-19 の改名、`icon_button`、`selection_field` の削除）

- **wave / lane / 前提**：W7a / U1 / U22
- **所有**：`crates/kivact-ui/` 全体、`crates/kivact-interaction/src/{announcer.rs, toaster/api.rs, toaster.rs, combobox.rs, combobox/handle.rs, table/model.rs, lib.rs}`、`crates/kivact/src/`（prelude と doc の行）、`dev/tests/kivact-public-api/tests/`（kit の 4 file を除く）、`examples/kivact/`、`templates/`、`dev/bench/`、`crates/kivact-devtools/src/`。
- **目的**：利用者と ui の他の部品から見た呼出しの形を、W8 の `#[component]` の形と同じにしておく。これで W8 の lane は互いの呼出し元を壊さない。
- **手順**：
  1. **handler と値（UI-13）**
     - `text_input.rs`：`pub fn on_draft_commit` を `pub fn on_commit` に改名。`rg -l "on_draft_commit" ... | xargs perl -pi -e 's/\bon_draft_commit\b/on_commit/g'`（対象の木は所有の欄の木）。
     - `stepper.rs`（UI-13、12 章 UI-U-03）：`pub fn stepper(items: impl IntoValue<Vec<StepperItem>>, current: Signal<usize>) -> Stepper`。利用者が届く項目はすべて押せる（2026-11-21 の設計者の回答で確認）。`stepper` の doc に「Every step is pressable; give `StepperItem::disabled(true)` to the steps the user may not jump to (a linear flow).」を足す。`pub fn on_step_press` を消し、`pub fn on_edit<R: HandlerResult>(mut self, f: impl Fn(EditPhase) -> R + 'static) -> Self`（field `on_edit: Option<Callback<EditPhase>>`、`callback(f)` で持つ）を足す。`into_view` の先頭で `let edit = kivact_interaction::EditSession::new(move |initial: usize| current.set(initial));` を作り、`on_edit` が `Some(cb)` なら `edit.on_edit(move |phase| cb.call(phase))` にする。利用者の操作で値を書く所（項目の押下、前・次の button、項目の key 操作、項目の a11y の Activate）は、すべて次の一つの関数を通す：
       ```rust
       let write = move |index: usize| {
           let index = index.min(items.with_untracked(|items| items.len().saturating_sub(1)));
           if index != current.get_untracked() {
               edit.once(current.get_untracked(), move || current.set(index));
           }
       };
       ```
       外からの `current.set(..)` は `write` を通らないので phase を報告しない。呼出し元の `.on_step_press(move |i| x.set(i))` は消し、`stepper(items, x)` の 2 つ目を `x`（signal）にする。
     - `tag_input.rs`：`pub fn on_add`・`pub fn on_remove` を消す（U22 で内部の利用は消えている）。呼出し元の `.on_add(..)`・`.on_remove(..)` を消す（tags の signal が値を持つ）。
     - `avatar.rs`：`pub fn on_open` を消す。`avatar()` は `.on_press(..)` が呼ばれたときだけ `use_button()` の part を一度付ける（今 `on_open` と `on_press` がそれぞれ part を付けている所を、`into_view` で `press: Option<Callback<()>>` が `Some` のときに一度だけ付ける形にする）。呼出し元の `.on_open(f)` は `.on_press(f).role(Role::Link)` にする。
  2. **`input_group_with` を消す（UI-13 (d)）**：`input_group.rs` の `pub fn input_group_with` と `enum Controls` の `Scoped` を消す。呼出し元 `input_group_with(label, move || children)` は `input_group(label, children)` にする（`IntoChildren` は `into_view` の中、group の context の中で view になるので、子の setup は今と同じく group の中で走る）。
  3. **`selection_field` を消す（UI-20）**：`selection_field.rs` を消し、`lib.rs` の `mod selection_field; pub use selection_field::{SelectionField, selection_field};` を消す。`select` に builder の method を足す：
     ```rust
     /// Show `trigger` as the select's trigger instead of the selected item's
     /// label; the select keeps its behavior, popup and semantics.
     pub fn trigger(mut self, trigger: impl IntoView) -> Self {
         self.trigger = Some(trigger.into_view());
         self
     }
     ```
     `select` の trigger を作る所で、`self.trigger` が `Some` ならその view を trigger の content に置く（今 selection_field が `value_label` と `content` で作っていた見た目を、呼出し元が `trigger(..)` に渡す）。呼出し元（`examples/kivact` に無ければ試験だけ）の `selection_field(label, value_label).content(f)` を `select(items, selected).label(label).trigger(text(value_label))` に直す。`crates/kivact/src/prelude.rs` に `selection_field`・`stepper` が残っていれば消す。
  4. **型の置き場（UI-19、UI-04 規則 3）**：
     - `crates/kivact-ui/src/lib.rs` の `pub use kivact_interaction::ComboboxMode;` を消す。`crates/kivact-interaction/src/combobox.rs`・`combobox/handle.rs` の `pub enum ComboboxMode` を ui の `combobox.rs` に移す（interaction の `use_combobox` は mode を取らない形にする：interaction の中で `ComboboxMode` を読んでいる行は、ui が渡す `bool`（`inline`）の引数にする。今の `ComboboxApi::mode(..)` の setter を `pub(crate)` ではなく `pub fn inline(self, inline: bool) -> Self` に改名する）。
     - `carousel.rs` の `pub use kivact_interaction::Snap;` を消す。ui の中の `Snap` は `kivact_interaction::Snap` と書く。`lib.rs` の `pub use carousel::{Carousel, Snap, carousel};` から `Snap` を消す。
     - `crates/kivact-interaction/src/table/model.rs`：`ColumnDef::with_width` を `width` に改名（呼出し元を直す）。同じ型の getter `pub fn width(&self) -> Option<f64>` は `pub fn preferred_width(&self) -> Option<f64>` に改名し、doc を「The width the column asked for (`width(..)`), before the user resizes it.」にする（呼出し元は `data_table/column.rs` の 1 箇所。2026-11-21 の設計者の回答）。`data_table/column.rs` は U13 で一つになっている。
  5. **`Element` の method を隠す固有 method を消す（API-13）**：`Element` trait の method の名前（今の HEAD では `description disabled draw exit_layout focusable focusable_when_disabled hidden hit_testable intrinsic_size invalid layout_id name node_mut node_ref on_exit_complete on_press recipe_style roving_focus style test_id with_node`。W5 の後の一覧は `rg -o "^\s+fn (\w+)\(" crates/kivact-core/src/view/element_data.rs -r '$1'` で取る）と同じ名前の、`Element` を実装する builder の固有 method を消す。今の HEAD の該当：
     - `disabled`：`card.rs`（`Card`）、`stepper.rs`（`Stepper`。`StepperItem::disabled` は `Element` を実装しない値の型なので残す）
     - `on_press`：`avatar.rs`、`badge.rs`、`card.rs`、`chat_message.rs`、`item_row.rs`、`navigation_link.rs`（U12 の後は `navigation_menu/link.rs`）
     - `description`：`alert.rs`、`dialog.rs`、`empty_state.rs`、`item_row.rs`、`navigation_link.rs`（`StepperItem::description` は値の型なので残す）
     消した method の代わりに、builder は `into_view`（`finish`）の先頭で root の node から呼出し側の値を読む：
     - disabled：`root.accessible_disabled()`（今ある）。
     - 押下：`self.root.authored().has_press()`（U25 の `Node::authored`。handler を複写しない）。押下で `use_button()` の part を付ける builder は、`true` のときだけ一度付ける。
     - 見える説明：`self.root.authored().description()`（`Option<Value<String>>`）。builder は、その値を今 `description` の field から描いていた node に描く。値はそのまま root の accessible description でもある（見える説明と支援技術の説明が同じになる）。
     呼出し元は変わらない（`.description("..")`・`.on_press(..)`・`.disabled(..)` は `Element` の method に解決される）。
     - **自分の押下も持つ builder（`navigation_menu/link.rs`。2026-11-21 の設計者の回答）**：`Node::on_press` は前の handler を置き換えるので、link は呼出し側の handler を取り出して、自分の押下の中で呼ぶ。core に一つ足す（`crates/kivact-core/src/view/authored.rs` の `impl Node`。この unit の所有に含める）：
       ```rust
       /// Take the press handler the authors gave this node so far (the one
       /// press resolves to: the last), leaving the node without one, so a
       /// component that must also act on press runs it inside its own:
       /// `let caller = root.take_press(); root.on_press(move || { /* call caller */ close() })`.
       #[doc(hidden)]
       pub fn take_press(&mut self) -> Option<kivact_runtime::Handler<Option<kivact_style::draw::RegionId>>> {
           // Take `handlers.activate` from every `Modifier::Caller` segment in order, then from
           // `self.common`; return the last `Some` (later wins, as the fold of the segments does).
       }
       ```
       `link.rs` は `pub fn on_press` と field `press` を消し、`into_view` で `let caller = root.take_press();`（`root.part(..)` の前）とし、`on_press` の closure の中で `caller` を、core の dispatch が `activate` を呼ぶのと同じ形（region は `None`）で呼んでから `context.close.call(())` を呼ぶ。試験（e2e_ui の navigation menu の試験の file）：`fn a_navigation_link_runs_the_callers_press_and_closes_its_panel()`：panel を開き、link（`.on_press(move || count.update(|n| *n += 1))`）を押すと count が 1 で、panel が閉じる。`Card` の `.on_press` が `CardKind::Press` を選んでいた所は、`authored().has_press()` で選ぶ。`variant` はこの unit では消さない（U24）。
  6. **`#[track_caller]`（API-14）**：`crates/kivact-ui/src` の `pub fn` のうち、本体で `into_value`・`into_text_value`・`into_text`・`into_prop` を呼ぶものすべてに `#[track_caller]` を付ける（`text_area` を含む）。一覧：`rg -n -B3 "into_(text_)?value\(\)|into_text\(\)|into_prop\(\)" crates/kivact-ui/src | rg "pub fn"`。
  7. **改名（API-19、UI-04）**：`crates/kivact-interaction/src/toaster/api.rs` の `pub fn toaster()` → `pub fn use_toaster()`、`announcer.rs` の `pub fn announcer()` → `pub fn use_announcer()`。`rg -l "\btoaster\(\)|\bannouncer\(\)" <所有の木> | xargs perl -pi -e 's/\btoaster\(\)/use_toaster()/g; s/\bannouncer\(\)/use_announcer()/g'`。`crates/kivact/src/prelude.rs` の `toaster` を `use_toaster` にする（W5 で済んでいれば確かめるだけ）。
  8. **`DialogKind`（UI-12 の ui 側。W7b の U30a が interaction の `DialogMode::Popover` を消せるように、ui の呼出しの形を先に変える）**：`dialog.rs` に
     ```rust
     /// What a dialog is: a modal dialog, an alert dialog, a drawer from an edge or a popover.
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub enum DialogKind {
         Modal,
         AlertModal,
         Drawer(Edge),
         Popover,
     }
     ```
     を足し、`Dialog::mode(mode: DialogMode)` を `Dialog::mode(kind: DialogKind)` にする（`Edge` は今の `DialogMode::Drawer` が持つ型）。dialog の中で interaction の `DialogMode` へ変換する（`Popover` → `DialogMode::Popover`。U30a がこの変換を `use_popup` に変える）。`rg -l "DialogMode" crates/kivact-ui/src dev/tests examples/kivact` の ui の呼出し（`data_table/filters.rs`、`date_input/view.rs`、`combobox/render.rs`、`select.rs`、試験）を `DialogKind` に直す。`lib.rs` の dialog の `pub use` に `DialogKind` を足す。
  9. **`icon_button`（5.9 節、USR-17）**：`button.rs` に
     ```rust
     /// A button that shows only `icon` and is named `label`, with the label as its tooltip.
     /// Its variant is `Ghost` unless the caller gives another with `.variant(..)`.
     #[track_caller]
     pub fn icon_button(icon: impl IntoValue<kivact_core::style::Icon>, label: impl IntoText) -> crate::Tooltip<Button> {
         let label = label.into_text_value();
         crate::tooltip(
             label,
             button(label)
                 .icon(kivact_core::view::icon(icon))
                 .hide_label(true)
                 .variant(ButtonVariant::Ghost),
         )
     }
     ```
     （手順 12 の `tooltip(label, trigger)` を使う。W8 の U40 で `impl Element` を返す `#[component]` にする。）`lib.rs` の `pub use button::{..}` に `icon_button` を足し、`crates/kivact/src/prelude.rs` に `icon_button` が無ければ足す。
  12. **`tooltip(label, trigger)`（12 章 API-3）**：`Tooltip::attach` を消し、trigger を位置引数にする。`tooltip.rs` の builder を次の形にする（中の behavior の配線は今の `attach` の本体のまま）：
     ```rust
     pub struct Tooltip<T> {
         trigger: T,
         label: Value<String>,
         enabled: Value<bool>,
         placement: Placement,
         align: Option<OverlayAlign>,
         offset: f64,
         content: Option<ViewFn>,
         variants: Variants,
     }
     /// A tooltip showing `label` (or `.content(..)`) for `trigger`, which stays
     /// the element the caller styles: `tooltip("Save", button("Save"))`.
     #[track_caller]
     pub fn tooltip<T: Element + IntoView>(label: impl IntoText, trigger: T) -> Tooltip<T> { /* fields as before, trigger */ }
     impl<T: Element + IntoView> Element for Tooltip<T> {
         fn node_mut(&mut self) -> &mut Node {
             self.trigger.node_mut()
         }
     }
     impl<T: Element + IntoView> IntoView for Tooltip<T> {
         fn into_view(self) -> View {
             // The body of the old `attach(self, trigger)` with `self.trigger` as `trigger`,
             // unchanged: the trigger (with the tooltip's trigger part) and the
             // `show(open, surface)` are SIBLINGS of one two-item fragment, so pointer
             // input on the tooltip never routes to the trigger. The caller's modifiers
             // are on the trigger already (`node_mut` is the trigger's root node).
             // let trigger = self.trigger.node_ref(anchor).with_node(|node| node.part(part).part(fallback));
             // (trigger, surface).into_view()
         }
     }
     ```
     `enabled`・`content`・`align`・`offset`・`placement`・`variant` の method は残す（`variant` は U24 で消える）。呼出し元を直す（`rg -n "tooltip\(" crates/kivact-ui/src dev/tests/kivact-public-api examples/kivact templates`）：
     - `avatar.rs`：`crate::tooltip(name).enabled(show).attach(root)` → `crate::tooltip(name, root).enabled(show).into_view()`（今の式の型に合わせ、`View` が要る所は `.into_view()`）
     - `alert.rs`：`crate::tooltip(label).attach(with_recipe_style(..))` → `crate::tooltip(label, with_recipe_style(..))`
     - `data_table/row_status.rs`：`tooltip(label).attach(mark)` → `tooltip(label, mark)`
     - `menu.rs`：`crate::tooltip(move || reason.get().unwrap_or_default())` の `.attach(x)` → `crate::tooltip(move || reason.get().unwrap_or_default(), x)`
     - 試験：`apg_tooltip.rs`、`ui_components.rs` の `tooltip("..")…attach(t)` → `tooltip("..", t)…`（期待値は変えない）。kit と catalog の呼出しは annex K（4.4 節の表）。
     試験を足す（`ui_components.rs`、e2e_ui）：
     ```rust
     #[test]
     fn a_tooltip_styles_its_trigger_and_a_press_on_it_does_not_press_the_trigger() {
         let (screen, pressed) = crate::support::mount(|| {
             let pressed = signal(0u32);
             (pressed, ui::tooltip("Save the file", ui::button("Save").on_press(move || pressed.update(|n| *n += 1)))
                 .test_id("save")
                 .style(style().width(120)))
         });
         let save = screen.get_by_role(Role::Button, "Save");
         assert_eq!(screen.find_by_test_id("save").expect("invariant: mounted").bounds().width, 120.0);
         assert_eq!(save.description().as_deref(), Some("Save the file"));
         save.hover();
         screen.advance(std::time::Duration::from_millis(1500));
         let tip = screen.get_by_text("Save the file");
         tip.click();
         assert_eq!(pressed.get_untracked(), 0, "a press on the tooltip does not press its trigger");
         screen.press_key(Key::Escape);
         assert!(screen.find_text("Save the file").is_none());
         assert!(screen.errors().is_empty(), "{:?}", screen.errors());
     }

     #[test]
     fn a_tooltip_wraps_a_custom_component_and_follows_enabled() {
         #[component]
         fn chip(label: Value<String>) -> impl Element {
             node().role(Role::Button).name(label).focusable(true)
         }
         let (screen, enabled) = crate::support::mount(|| {
             let enabled = signal(false);
             (enabled, ui::tooltip("Chip hint", chip("Chip")).enabled(enabled))
         });
         let chip = screen.get_by_role(Role::Button, "Chip");
         chip.focus();
         screen.advance(std::time::Duration::from_millis(1500));
         assert!(screen.find_text("Chip hint").is_none());
         enabled.set(true);
         chip.blur();
         chip.focus();
         screen.advance(std::time::Duration::from_millis(1500));
         assert!(screen.find_text("Chip hint").is_some());
         assert!(screen.errors().is_empty(), "{:?}", screen.errors());
     }
     ```
     （`hover`・`advance`・`get_by_text`・`find_text`・`blur`・`description`・`bounds` は `ui_components.rs` と support が今使う名前に合わせる。surface は trigger の兄弟なので、tooltip の上の押下は trigger に届かない。）
  11. **button の子と group の大きさ**（W8 の `#[component]` と同じ呼出しの形にする）：
     - `Button::child(child)`（呼ぶたびに足す）を `Button::children(children: impl IntoChildren)`（一度だけ）に改名する。呼出し元の `.child(a).child(b)` は `.children((a, b))` にする（`Button` に `child` が無くなるので compile の誤りで全部見つかる）。
     - `Button::default_size` を消す。`Button` の `size` を `Option<Value<ControlSize>>` にし、`finish` の中で `size.unwrap_or_else(|| default_control_size().into_value())` とする（`finish` は group の `provide_control_size` の中で走るので、group の大きさになる）。
     - `ButtonGroupMember`（`From<Button>`、`solid`、`keep_size`）を消す。group の中の button の角と区切り線（今 `impl IntoView for ButtonGroupMember` の `.style(move || ..)` にある式）は、`button.rs` の `finish` が `try_use_context::<crate::button_group::GroupPosition>()` が `Some` のときに root に足す style にする（`GroupPosition` を `pub(crate)` にする）。`solid` は「button の variant が `Primary` か `Danger`」と読む（`variants.get::<ButtonVariant>()` が無いときは既定の `Primary`）。`keep_size(true)` の代わりは `.size(..)` を明示すること。
     - `ui_toolbar.rs` の `ButtonGroupMember::from(button("Archive")).solid(|| true).keep_size(true)` を `button("Archive").size(ControlSize::Medium)` にする（既定の variant は `Primary` なので solid になる。期待値は変えない）。import の `ButtonGroupMember` を消す。
  10. **`lib.rs` の doc の表（UI-15）**：`crates/kivact-ui/src/lib.rs` の先頭の doc の表を、`lib.rs` の root が再公開している部品の関数の全部（約 80）の一覧に書き直す。列は「部品」「位置引数」「behavior」の 3 つで、位置引数の列は U22・U23 の後の形を書く。`# Looks` の節を 5.7・5.9 節の形（theme 変数、`<Name>Theme`、`.variant(..)`、`.style(..)`）で書き直し、`UiRecipes`・`UiTokens`・`ThemeColor`・`ColorName` への言及を消す。`# Kits` の節は「kit は `theme(mode) -> Theme` を一つ持ち、部品の見た目を `<Name>Theme` と theme 変数で変える」と書く。
- **消すもの**：`on_draft_commit`、`Stepper::on_step_press`、`TagInput::{on_add, on_remove}`、`Avatar::on_open`、`input_group_with`、`selection_field`、`SelectionField`、`ui::ComboboxMode` の再公開と interaction の `ComboboxMode`、`ui::Snap` の再公開、`ColumnDef::with_width`、手順 5 の固有 method、`toaster`、`announcer`、ui の呼出しの `DialogMode`。
- **状態の閉じ方**：`Select` に足す `trigger: Option<View>` は値渡しの builder だけが書く（対象外）。`Avatar` の press は `Option<Callback<()>>` 一つ。
- **試験**：perl・手で直した試験の期待値は変えない。`rg -l "on_draft_commit|on_step_press|on_add|on_remove|on_open\b|input_group_with|selection_field|ComboboxMode|with_width|toaster\(\)|announcer\(\)|DialogMode" dev/tests/kivact-public-api/tests` の e2e_ui・e2e_core の file を直す。足す試験は無い。
- **stepper の試験（UI-U-03）**：`interaction_edit.rs`（e2e_ui）の `every_value_writing_standard_component_reports_on_edit` に stepper を足す：`logs` を `vec![Vec::new(); 7]` に、mount の中で `let step = signal(0usize);` と `kivact::ui::stepper(vec![StepperItem::new("One"), StepperItem::new("Two")], step).on_edit(observe(6))` を column の最後に足し、返す tuple に `step` を足す。操作の最後に `screen.get_by_role(Role::Button, "Two").click();`（stepper の項目の role と名前は今の stepper の項目の押下の part の role に合わせる）、`assert_eq!(step.get_untracked(), 1);`。全 log が `[Begin, Commit]` の確かめはそのまま。新しい試験 `stepper_reports_no_phase_for_an_outside_write`：同じ stepper を `on_edit` 付きで mount し、`step.set(1)` の後 `run_until_idle` で log が空のまま。
- **確認（tier a）**：
  ```
  cargo fmt --all
  cargo check -p kivact-interaction -p kivact-ui -p kivact -p kivact-devtools -p kivact-public-api-tests --all-targets
  cargo clippy -p kivact-interaction -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-ui
  cargo xtask check --no-write --crate kivact-interaction
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_ apg_ interaction_toast interaction_dialog
  ```
- **完了条件**：`rg -nw "on_draft_commit|on_step_press|input_group_with|selection_field|SelectionField|with_width" crates/kivact-ui crates/kivact-interaction crates/kivact dev/tests examples/kivact templates --glob '!theme_conformance.rs' --glob '!ui_standard.rs' --glob '!presence_kit.rs' --glob '!apg_grid_kit.rs'` が空。`rg -n "pub use kivact_interaction::(Snap|ComboboxMode)" crates/kivact-ui` が空。上の命令が通る。


## U25：ui が `#[component]` で要る core と macro の受け口（W8 の前提）

- **wave / lane / 前提**：W7a / U1 / U21（**U22 より前に行う**。W7a の順は U20 → U21 → U25 → U22 → U23 → U24）。annex C の C09（macro）・C10（`Variant`・`Variants`、`Node::absorb`）の後。W7 の他の lane（E・A）は kivact-core と kivact-macros を持たないので、この unit が持てる。
- **所有**：`crates/kivact-macros/src/{parse.rs, expand.rs}`、`crates/kivact-core/src/view/{variant.rs, children.rs, view.rs の mod・pub use の行, elements/builders.rs（`Node::absorb` と新しい `Node::authored`）}`、新 `crates/kivact-core/src/view/authored.rs`、`crates/kivact-core/src/custom.rs`（`pub use` の行）、試験 `dev/tests/kivact-public-api/tests/e2e_core/component_macro.rs`。
- **目的**：設計 5.1 規則 3 と 12 章 API-1・API-2・API-4・API-5・ST-09 の受け口を作る。
  - (A) `Rc<dyn Fn(A..) -> T>` の props（Func）：`impl Fn(A..) -> T + 'static` を受け、closure の引数の型が推論される（API-2）。
  - (B) 部品の型引数そのものの props（`control: C`）：`C` をそのまま受け、`Into` で包まない（API-1）。
  - (C) 呼出し側の modifier の **snapshot**：`#[component]` の引数に `#[caller] caller: Caller` と書くと、macro がその部品の `__root` の authored な modifier を畳んだ値を渡す。thread-local の stack と `custom::caller()` は作らない（API-4・API-5・ST-09）。畳む関数は一つ（`Node::authored() -> Authored`）で、recipe・root・部分が同じ値を使う。
  - (D) disabled の合成の規則：本体が root に付けた disabled と、呼出し側の disabled は **OR** で合わさる（busy の button が呼出し側の `.disabled(false)` で押せるようにならない）。
  - (E) 遅れて view にする子 `kivact::view::Children`。(F) `Variants::with`（`Look::with_variant` のため）。
- **手順**：
  1. **(F)** `view/variant.rs` の `impl Variants` に足す：
     ```rust
     /// These values with `value` set for its type (a constant), replacing an earlier one.
     pub fn with<V: Variant>(mut self, value: V) -> Self {
         self.set(kivact_runtime::IntoValue::into_prop(value));
         self
     }
     ```
     （`V` は `#[derive(Prop)]` で `IntoValue<V>` を持つ。C10 の `set(&mut self, Prop<V>)` を使う。）
  2. **(C) `Node::authored`**：新 `view/authored.rs`（`view.rs` に `mod authored; pub use authored::Authored;`）：
     ```rust
     //! Role: Fold every authored modifier segment of a node, in order, into one snapshot.
     use crate::view::{Node, NodeRef, Variants};
     use kivact_runtime::{IntoValue, Value};

     /// What the authors of a node wrote on it so far, folded over its modifier
     /// segments in order (the segments a part or `absorb` closed, then the
     /// open one): later values win, variants merge by type. A `#[component]`
     /// body receives it as its `#[caller]` argument; the recipe of the root and
     /// of every part read the same value.
     #[derive(Clone, Default)]
     pub struct Authored {
         variants: Variants,
         disabled: Option<Value<bool>>,
         expanded: Option<Value<bool>>,
         invalid: Option<Value<bool>>,
         name: Option<Value<String>>,
         description: Option<Value<String>>,
         labelled_by: Vec<NodeRef>,
         has_press: bool,
     }

     impl Node {
         /// The authored modifiers of this node so far (see [`Authored`]).
         #[doc(hidden)]
         #[track_caller]
         pub fn authored(&self) -> Authored {
             let mut variants = Variants::default();
             let mut has_press = false;
             for common in self.authored_segments() {
                 if let Some(v) = common.variants.as_deref() {
                     variants.merge(v);
                 }
                 has_press |= common.handlers.activate.is_some();
             }
             Authored {
                 variants,
                 disabled: self.disabled_source().map(IntoValue::into_value),
                 expanded: self.expanded_source().map(IntoValue::into_value),
                 invalid: self.invalid_source().map(IntoValue::into_value),
                 name: self.name_source().map(IntoValue::into_value),
                 description: self.description_source().map(IntoValue::into_value),
                 labelled_by: self.labelled_by_sources(),
                 has_press,
             }
         }

         /// The closed caller segments in order, then the open one.
         fn authored_segments(&self) -> impl Iterator<Item = &crate::view::element_data::Common> {
             self.modifiers
                 .iter()
                 .filter_map(|m| match m {
                     crate::view::element_data::Modifier::Caller(c) => Some(c.as_ref()),
                     crate::view::element_data::Modifier::Part(_) => None,
                 })
                 .chain(std::iter::once(&self.common))
         }
     }

     impl Authored {
         pub fn variants(&self) -> Variants { self.variants.clone() }
         pub fn disabled(&self) -> Option<Value<bool>> { self.disabled }
         pub fn expanded(&self) -> Option<Value<bool>> { self.expanded }
         pub fn invalid(&self) -> Option<Value<bool>> { self.invalid }
         pub fn name(&self) -> Option<Value<String>> { self.name }
         pub fn description(&self) -> Option<Value<String>> { self.description }
         pub fn labelled_by(&self) -> &[NodeRef] { &self.labelled_by }
         /// Whether an author gave the node a press handler (no handler is copied).
         pub fn has_press(&self) -> bool { self.has_press }
     }
     ```
     `disabled_source`・`expanded_source`・`invalid_source`・`name_source`・`description_source`・`labelled_by_sources` は今の `elements/builders.rs` の関数（`semantic_inputs()` が `Modifier::Caller` の段と開いた段と `semantic_reads` を順に畳む）をそのまま使う。`Variants::merge` は C10 の `pub(crate)`。`common.handlers.activate` は W5 の後の press の field（`Node::on_press` が書く field。`rg -n "fn on_press" crates/kivact-core/src/view/node_modifiers.rs` の本体で名前を確かめる）。`Value<T>` が `Copy` でない型があれば `.clone()` にする。
     `custom.rs` に `pub use crate::view::Authored as Caller;` を足す（公開の名前は `kivact::custom::Caller`。`Authored` は `#[doc(hidden)]` の core の名前）。
  3. **(D) disabled の OR**：C09 の `Node::absorb` の本体を次にする（`debug_assert` と modifiers の移しは C09 のまま）：
     ```rust
     #[doc(hidden)]
     pub fn absorb(mut self, caller: Node) -> Node {
         let body_disabled = self.disabled_source().map(kivact_runtime::IntoValue::into_value);
         let caller_disabled = caller.disabled_source().map(kivact_runtime::IntoValue::into_value);
         let Node { common, children, modifiers, .. } = caller;
         debug_assert!(children.is_empty(), "a #[component] caller adds no children to its root");
         self.modifiers.push(crate::view::element_data::Modifier::Caller(Box::new(std::mem::take(&mut self.common))));
         self.modifiers.extend(modifiers);
         self.common = common;
         if let (Some(body), Some(outer)) = (body_disabled, caller_disabled) {
             // Either side disables the root: a busy body stays disabled under the caller's `.disabled(false)`.
             self.common.set_disabled(kivact_runtime::IntoValue::into_prop(move || body.get() || outer.get()));
         }
         self
     }
     ```
     `Node::absorb` の doc の「its name, description, disabled, …over the body's」を「its name, description, test id, variants and node refs over the body's; a disabled state of either side disables the root」に直す。設計 5.1 規則 2（disabled は本体と呼出し側の OR）のとおり。
  4. **(C) macro の `#[caller]`**：`parse.rs`：
     - 関数の引数の属性 `#[caller]` を受ける。その引数は props にしない（setter も constructor の引数も作らない）。型の最後の segment が字面 `Caller` でなければ error `"a #[caller] argument has the type `Caller`"`。`#[caller]` は一つまで（二つ目は error `"only one #[caller] argument"`）。戻り値が `impl IntoView` の部品に付けたら error `"#[caller] needs a component that returns impl Element"`。`Model` に `caller: Option<syn::Ident>` を足す。
     - `expand.rs`：`into_view` の `let __body = (move || { .. })();` の前に、`caller` が `Some(ident)` のとき `let #ident: R::custom::Caller = __root.authored();` を置く（`__root` は分解した後の値。snapshot は body の前に一度だけ作る。body はこの値を capture でき、後で他の部品の値を読むことは無い）。
  5. **(A) Func の props**：`parse.rs` の `PropKind` に足す：
     ```rust
     /// `Rc<dyn Fn(A, ..) -> T>`: takes `impl Fn(A, ..) -> T + 'static`; when `T` is literally
     /// `View`, takes `impl Fn(A, ..) -> V + 'static` with `V: IntoView`.
     Func { args: Vec<syn::Type>, output: syn::Type, view: bool },
     ```
     `kind(ty)`：型の最後の segment が字面 `Rc` で、その型引数が `dyn Fn(..) -> ..`（`syn::TypeTraitObject` の bound の path の最後の segment が `Fn`、引数が parenthesized）なら `Func`。`-> T` が無い `dyn Fn(..)` は `output` を `()` にする。`output` の最後の segment が字面 `View` なら `view: true`。`expand.rs` の kind の表（C09）に足す行：

     | kind | 位置引数・setter の型 | 保存の型 | 保存の式 | scope の中の変換 |
     |---|---|---|---|---|
     | Func（view でない） | `impl Fn(A0, A1, ..) -> T + 'static` | `::std::rc::Rc<dyn Fn(A0, A1, ..) -> T>` | `::std::rc::Rc::new(x)` | なし |
     | Func（view） | `impl Fn(A0, A1, ..) -> __KvV<Name> + 'static`（`__KvV<Name>: R::view::IntoView`。名前の規則は C09 の `ViewFn` と同じ） | `::std::rc::Rc<dyn Fn(A0, A1, ..) -> R::view::View>` | `::std::rc::Rc::new(move \|__a0: A0, __a1: A1, ..\| R::view::IntoView::into_view(x(__a0, __a1, ..)))` | なし |

     引数の型が参照（`&R`）のときは閉包の引数を `__a0: &R` と書く。`Option<Rc<dyn Fn ..>>` は optional（setter）、`#[prop(default = expr)]` は `Rc::new(expr)`。API-18 の確かめの tuple に `Func` は入れない（`Rc` は std の型）。型の別名（`type Render = Rc<dyn Fn(u32) -> View>`）は「それ以外」で、`impl Into<Render>` を受ける（5.1 規則 3）。
  6. **(B) 型引数そのものの props**：`PropKind` に `Param(syn::Ident)` を足す。`kind(ty)`：型が path の segment 一つで、その ident が部品の関数の型引数の一つと等しければ `Param`（`Option<C>` も同じ判定で optional）。expand の行：位置引数・setter の型は `C`、保存の型は `C`、保存の式は `x`、変換なし。`C` は他の props と同じく struct の型引数として残る（`phantom` の判定で `C` は「props に現れる」）。
  7. **(E)** `view/children.rs` に足し、`view.rs` に `pub use children::Children;`：
     ```rust
     /// Children made into views when the component asks: a component that
     /// provides a context to its children calls [`Children::into_views`]
     /// inside that context.
     pub struct Children(Box<dyn FnOnce() -> Vec<View>>);
     impl Children {
         /// Make the children into views now.
         pub fn into_views(self) -> Vec<View> {
             (self.0)()
         }
     }
     impl<C: IntoChildren + 'static> From<C> for Children {
         fn from(children: C) -> Self {
             Self(Box::new(move || children.into_views()))
         }
     }
     ```
     macro は変えない（字面 `Children` は「それ以外」で `impl Into<Children>` を受ける）。
  8. 試験：`component_macro.rs` に次を足す（`mount`・`find_text`・`get_by_role` はこの file が今使う support の名前に合わせる。file の先頭の `use` に `kivact::custom::Caller` と `kivact::Prop` を足す）。
     ```rust
     #[derive(Clone, Copy, Debug, PartialEq, Prop)]
     enum Look3 { Plain, Ghost, Bold }
     impl kivact::ui::theme::Variant for Look3 {}

     #[component]
     fn leaf(seen: Signal<Option<Look3>>, #[caller] caller: Caller) -> impl Element {
         seen.set(caller.variants().get::<Look3>());
         node()
     }
     #[component]
     fn inner(seen: Signal<Option<Look3>>) -> impl Element {
         leaf(seen).variant(Look3::Ghost)
     }
     #[component]
     fn outer(seen: Signal<Option<Look3>>) -> impl Element {
         inner(seen)
     }

     #[test]
     fn a_wrapper_default_variant_reaches_the_leaf_and_an_outer_variant_overrides_it() {
         let (_, (a, b)) = crate::support::mount(|| {
             let (a, b) = (signal(None), signal(None));
             ((a, b), column((outer(a), outer(b).variant(Look3::Bold))))
         });
         assert_eq!(a.get_untracked(), Some(Look3::Ghost));
         assert_eq!(b.get_untracked(), Some(Look3::Bold));
     }

     #[test]
     fn a_variant_before_or_after_a_part_is_the_same_variant() {
         let (_, (a, b)) = crate::support::mount(|| {
             let (a, b) = (signal(None), signal(None));
             let part = || kivact::custom::Part::new(Value::from(()), |n: Node| n);
             (
                 (a, b),
                 column((
                     leaf(a).variant(Look3::Bold).with_node(|n| n.part(part())),
                     leaf(b).with_node(|n| n.part(part())).variant(Look3::Bold),
                 )),
             )
         });
         assert_eq!(a.get_untracked(), Some(Look3::Bold));
         assert_eq!(b.get_untracked(), Some(Look3::Bold));
     }

     #[test]
     fn a_busy_body_stays_disabled_under_the_callers_enabled_state() {
         #[component]
         fn busy_button(pressed: Signal<u32>) -> impl Element {
             node().role(Role::Button).name("Busy").focusable(true).disabled(true)
                 .on_press(move || pressed.update(|n| *n += 1))
         }
         let (screen, pressed) = crate::support::mount(|| {
             let pressed = signal(0);
             (pressed, busy_button(pressed).disabled(false))
         });
         screen.get_by_role(Role::Button, "Busy").click();
         assert_eq!(pressed.get_untracked(), 0);
     }

     #[test]
     fn the_caller_snapshot_reports_a_press_handler_without_copying_it() {
         #[component]
         fn probe(seen: Signal<bool>, #[caller] caller: Caller) -> impl Element {
             seen.set(caller.has_press());
             node()
         }
         let (_, (a, b)) = crate::support::mount(|| {
             let (a, b) = (signal(false), signal(true));
             ((a, b), column((probe(a).on_press(|| {}), probe(b))))
         });
         assert!(a.get_untracked());
         assert!(!b.get_untracked());
     }

     #[test]
     fn a_function_prop_infers_its_closure_argument_types() {
         #[component]
         fn labels(items: Value<Vec<u32>>, render: Option<std::rc::Rc<dyn Fn(u32) -> View>>) -> impl Element {
             let render = render.unwrap_or_else(|| std::rc::Rc::new(|n: u32| text(n.to_string()).into_view()));
             row(items.get_untracked().into_iter().map(|n| render(n)).collect::<Vec<_>>())
         }
         let (screen, _) = crate::support::mount(|| ((), labels(vec![1, 2]).render(|n| text(format!("#{n}")))));
         assert!(screen.find_text("#2").is_some());
     }

     #[test]
     fn a_type_parameter_prop_takes_its_argument_without_annotations() {
         #[component]
         fn labelled<C: Element + IntoView>(label: Value<String>, control: C) -> impl Element {
             column((text(label), control))
         }
         #[component]
         fn custom_input() -> impl Element {
             node().role(Role::TextBox).name("Custom").focusable(true)
         }
         let (screen, _) = crate::support::mount(|| ((), labelled("Name", custom_input())));
         assert!(screen.find_text("Name").is_some());
     }

     #[test]
     fn lazy_children_are_made_inside_the_components_context() {
         #[derive(Clone)]
         struct Depth(u8);
         #[component]
         fn shelf(children: kivact::view::Children) -> impl Element {
             column((provide(Depth(7), move || children.into_views()),))
         }
         #[component]
         fn depth() -> impl Element {
             text(use_context::<Depth>().0.to_string())
         }
         let (screen, _) = crate::support::mount(|| ((), shelf(depth())));
         assert!(screen.find_text("7").is_some());
     }
     ```
     （`Look3` の `#[derive(Prop)]` は C09 の derive。`kivact::custom::Part::new(state, apply)` の形は今の `Part::new` に合わせ、state は `()` の定数の value にする。）
- **消すもの**：なし。（以前の版の `custom::caller()`・thread-local の stack・`Node::{variants, press_handler, take_disabled}` は作らない。）
- **状態の閉じ方**：`Authored`（`Caller`）は値の snapshot で、`authored()` の中で一度だけ作り、`&mut self` の method も内部可変の field も持たない（6.1 節の (b) を満たさないので対象外）。`Children` は `Box<dyn FnOnce>` 一つ。
- **試験**：上の 7 つを足す（`e2e_core` の `component_macro`）。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-core -p kivact-macros
  cargo check -p kivact-core -p kivact-macros -p kivact --all-targets
  cargo clippy -p kivact-core -p kivact-macros --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-core
  cargo xtask check --no-write --crate kivact-macros
  KIVACT_TEST_MIRROR=off cargo test -p kivact-macros
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- component_macro
  ```
- **完了条件**：上の 7 つの試験が通る（filter が 0 件を走らせたら失敗）。`rg -n "thread_local|fn caller\(" crates/kivact-core/src/custom` が空。上の命令が通る。

## U24：`Element::variant` を使い、68 個の `variant` の写しと ui の `Variants` を消す（UI-14、5.4 節）

- **wave / lane / 前提**：W7a / U1 / U23。annex C の C10 と U25。
- **所有**：`crates/kivact-ui/` 全体、`dev/tests/kivact-public-api/tests/`（`kivact::ui::{Variant, Look}` を使う e2e_ui の file）。
- **目的**：見た目の印 `.variant(v)` を core の `Element` の method 一つにし、ui の builder の写し 68 個と `variants: Variants` の field を消す（UI-14）。`Look<S>` を core の `Variants` の上に作る（annex C の U-REQ-4）。
- **手順**：
  1. `crates/kivact-ui/src/variants.rs` を消す。`Look<S>` を `crates/kivact-ui/src/recipe.rs` に置く：
     ```rust
     use kivact_core::view::{Variant, Variants};

     /// What a recipe reads: the part's state and the component's variants.
     #[derive(Clone)]
     pub struct Look<S> {
         state: S,
         variants: Variants,
     }
     impl<S> Look<S> {
         /// A look of `state` without variants: for a kit that draws its own
         /// part with a standard component's recipe.
         pub fn new(state: S) -> Self {
             Self { state, variants: Variants::default() }
         }
         pub(crate) fn of(state: S, variants: Variants) -> Self {
             Self { state, variants }
         }
         /// The same look with `value` added (a later value of a type replaces the earlier).
         pub fn with_variant<V: Variant>(mut self, value: V) -> Self {
             self.variants = self.variants.with(value);
             self
         }
         /// The component's variant of type `V`, if it was given one (tracked when reactive).
         pub fn variant<V: Variant>(&self) -> Option<V> {
             self.variants.get::<V>()
         }
     }
     impl<S> std::ops::Deref for Look<S> {
         type Target = S;
         fn deref(&self) -> &S {
             &self.state
         }
     }
     ```
     `theme.rs` の `pub use crate::variants::{Look, Variant};` を `pub use crate::recipe::Look; pub use kivact_core::view::Variant;` にする。
  2. `layer.rs` の `crate::variants::Variants` を `kivact_core::view::Variants`、`variants.look(state())` を `Look::of(state(), variants.clone())` にする。
  3. ui の look の `.variant::<X>().copied()`・`.cloned()` を `.variant::<X>()` にする（値で返るため）：
     ```
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     rg -l --path-separator // "variant::<" crates/kivact-ui/src | xargs perl -0pi -e 's/(\.variant::<[^>]+>\(\))\s*\.(copied|cloned)\(\)/$1/g'
     ```
     `Option<&X>` を受けていた `match`（`Some(&X::A)`）は `Some(X::A)` にする（compile の誤りで分かる）。
  4. 各 builder から `variants: Variants` の field と `pub fn variant(..)`（68 件：`rg -n "pub fn variant\(" crates/kivact-ui/src`）を消す。`self.variants` を使っていた所は、`into_view`（`finish`）の先頭で `let variants = self.root.authored().variants();`（U25 の `Node::authored`。part の前後と absorb の段をすべて順に畳む）を読む。root が `Node` でない builder は `Element::with_node` の中で読む。既定を足していた所（`variants.with(SelectSheetOption(size.get()))`）は core の `Variants::with` を使う。`inherited`・`with_defaults`・`merge` を使っていた所は、`Variants::with` を足す順（既定を先、呼出し側の値を後）で同じ結果にする。
  5. `impl Variant for X {}` の `X` が `Clone` を derive していなければ足す（C10 が W5 で行っていれば確かめるだけ）。
  6. 試験：`rg -l "kivact::ui::(Variant|Look)\b|ui::\{[^}]*\b(Variant|Look)\b" dev/tests/kivact-public-api/tests` の e2e_ui の file の import を `kivact::ui::theme::{Look, Variant}` にする。
- **消すもの**：`crates/kivact-ui/src/variants.rs`（ui の `Variants`・`Extensions`・`Extension`・`DebugWith`・旧 `Look`）、68 個の `variant` method、すべての `variants: Variants` の field。
- **状態の閉じ方**：`Look<S>` は値（constructor と値渡しの `with_variant` だけが書く）。
- **試験**：期待値は変えない。`ui_theming.rs`・`ui_controls.rs` の variant の試験が通ること。
- **確認（tier a）**：U20 と同じ命令（test は `--test e2e_ui -- ui_` と `--test e2e_ui -- apg_` の 2 つ）に、W7a の最後の確認として次を走らせ、結果を**報告だけ**する（W7a では ui の手書きの builder が W8 まで残るので `hand-element` は失敗する。失敗しても STOP しない。通るのは W8 の後）：
  ```
  cargo xtask check --no-write --strict --only element-shadow,hand-element --crate kivact-ui
  ```
- **完了条件**：`rg -n "pub fn variant\(" crates/kivact-ui/src` が空。`rg -n "variants: Variants" crates/kivact-ui/src` が空。`ls crates/kivact-ui/src/variants.rs` が無い。上の命令のうち報告だけの `--strict` の行を除くものが通る。

### 3.3 W7b（lane U1〜U4）

## U30a：開閉の語彙と `surface()`（Disclosure・Popup・Dialog・inline_edit・tooltip）

- **wave / lane / 前提**：W7b / U1 / W7a の全 unit（U20 → U21 → U25 → U22 → U23 → U24 の順で済んでいる）。この unit が始まる時点のコードは次のとおりである。
  - W5・W6 の API：`Signal<T>`（`Binding`・`IntoBinding`・`into_binding` は無い）、`binding(get, set) -> Signal<T>`、`Callback<E>`／`callback(f)`、`use_context::<T>() -> T`、`Trigger`。interaction の `impl IntoBinding<bool>` の引数は W5（annex R の掃引）で `Signal<bool>` になっている（`use_disclosure(open: Signal<bool>)`、`use_popup(open: Signal<bool>)`、`use_dialog(open: Signal<bool>, kind)`、`use_inline_edit(editing: Signal<Option<K>>)`、`InlineEditApi::field(.., draft: Signal<String>)`、`PopupApi::open_binding() -> Signal<bool>`）。payload の無い handler は `impl IntoHandler<(), M>`、payload のある handler は `impl Fn(E) -> R + 'static`（`R: HandlerResult`）を受ける。`Signal::update` は `()` を返す。
  - U06（W3）：`PopupApi::finish_close`・`PopupOptions.modal`・`PopupApi::modal` は消え、閉じる経路は `Event::Close(reason)` の一つになっている。`pub(crate) fn return_focus_target` と `PopupOptions::return_focus_target` は残っている（この unit の手順 2 で `FocusReturn::anchor_value` に置き換える）。`use_dialog` は `use_popup(open).focus_first()` で popup を作っている。
  - U10（W4）：`TooltipApi` の hover・focus の memo と delayed の machine は `fn attach`（trigger の `on_attach`）の中で作られ、`engaged` の memo は `options.value().get().disabled` を読んでいる。
  - U11〜U14（W4）：ui は部品ごとの directory になっている。この unit が触る ui の file は `dialog.rs`（`dialog/look.rs`、`dialog/drawer_swipe.rs`）、`disclosure.rs`（`disclosure/group.rs`、`disclosure/look.rs`）、`tooltip.rs`（`tooltip/look.rs`）。HoverCard と NavigationMenu は次の U30c が行う。
  - U23（W7a）：ui に `pub enum DialogKind { Modal, AlertModal, Drawer(Edge), Popover }` があり、`Dialog::mode(kind: DialogKind)`、`DialogState.kind: DialogKind`、ui と試験の呼出し（`data_table/filters.rs`、`date_input/view.rs`、`combobox/render.rs`、`select.rs`、`dialog/drawer_swipe.rs`、`ui_disclosure_surfaces.rs` ほか）はすべて `DialogKind` を使う。ui の中で interaction の `DialogMode` を使うのは、`dialog.rs` の render が `DialogKind` を `DialogMode` に変換する `match` だけである（`Popover` → `DialogMode::Popover`）。`toaster()`→`use_toaster()`、`announcer()`→`use_announcer()` も U23 で済んでいる。
  - W1：`xtask-allow(state-fields)` の印はすべて消えている。e2e の binary は annex T の割当てどおりで、`interaction_*`・`ui_*`・`apg_*` は `--test e2e_ui` で走る（kit を使う `ui_standard.rs`・`apg_grid_kit.rs` は package `kivact-kit-tests` の `e2e_kits` で、W6〜W10 は compile しないので走らせない）。
- **所有**（2.2 節の表に従う）：
  - interaction：`crates/kivact-interaction/src/{disclosure.rs, focus_return.rs, popup/api.rs, popup/machine.rs, dialog.rs, inline_edit.rs, tooltip/api.rs, lib.rs（`mod`・`pub use` の行だけ）}`。`menu/session.rs` と `select/handle.rs` は手順 2 の `.hide(` → `.close(` の改名の行だけ。
  - ui：`crates/kivact-ui/src/{dialog.rs, dialog/look.rs, disclosure.rs, tooltip.rs}`、`crates/kivact-ui/src/data_table/rows.rs`（2.2 節でこの unit が持つ。変更は無い）。`dialog/drawer_swipe.rs` は U32 が持つので触らない。
  - 試験：`dev/tests/kivact-public-api/tests/e2e_ui/{interaction_dialog.rs, interaction_show_hide.rs, interaction_tooltip.rs, interaction_toggle.rs, ui_field.rs}`
- **目的**：Disclosure・Popup・Dialog の Api を `use_x(.., open: Signal<bool>)`、`show()`・`hide()`・`toggle()`、`is_open()`、`trigger()`・`content()`、`surface(placement, content)` に揃える（UI-12、UI-24、5.9 節）。あわせて次を行う。
  - interaction の `DialogMode::Popover` を消し、popover の部品（`title`・`description`・`close_trigger`・`initial_focus_first`）を `PopupApi` に移す。ui の `DialogKind::Popover` は `use_popup` で作る。
  - `inline_edit` を `edit`・`cancel`・`editing` にする。
  - disclosure と tooltip の disabled の二重の出所を消す（UI-11）。
  - `return_focus_target` を公開の `FocusReturn::anchor_value` にする（U06 で残した分）。
- **手順**：
  1. **`disclosure.rs`**
     - `pub struct DisclosureApi` を次にする。
       ```rust
       #[derive(Clone, Copy)]
       pub struct DisclosureApi {
           pub(crate) open: Signal<bool>,
           /// The shown state: the requested state.
           shown: Value<bool>,
           // trigger_state と content_state の 2 field と、その getter と
           // attach_part_state の行は U31 まで今のまま残す
           /// The node the trigger part binds; a surface is placed at it.
           anchor: NodeRef,
           pub(crate) content: NodeRef,
       }
       ```
     - 消すもの：field の `settings: OptionsCell<DisclosureOptions>` と `host_disabled: Signal<Vec<Value<bool>>>`、`struct DisclosureOptions`、`pub fn disabled`、`fn is_disabled`、`fn register_host_disabled`、`use crate::options_cell::OptionsCell;`。
     - `pub fn use_disclosure(open: Signal<bool>) -> DisclosureApi`（署名は W5 のまま）：literal は `open`、`shown: open.into_value()`、`trigger_state`・`content_state`（今のまま）、`anchor: node_ref()`、`content: node_ref()`。
     - module の末尾（`impl DisclosureApi` の外）に private の関数を置く。
       ```rust
       /// The disabled state the caller gave the trigger node with `.disabled(..)`
       /// before attaching the part: the one source of a trigger's disabled state.
       fn trigger_disabled(host: &mut kivact_core::view::Node) -> Value<bool> {
           host.accessible_disabled().unwrap_or_else(|| false.into_value())
       }
       ```
     - `impl DisclosureApi` に足す。
       ```rust
       /// Show: write `true` unless it is shown.
       pub fn show(self) {
           if !self.open.get_untracked() {
               self.open.set(true);
           }
       }
       /// Hide: write `false` unless it is hidden.
       pub fn hide(self) {
           if self.open.get_untracked() {
               self.open.set(false);
           }
       }
       /// Invert the requested state.
       pub fn toggle(self) {
           self.open.set(!self.open.get_untracked());
       }
       ```
       今の `pub fn hide`（無条件に `false` を書くもの）はこの `hide` で置き換える。
     - `fn write(self, open: Option<bool>)` の本体を `self.open.set(open.unwrap_or_else(|| !self.open.get_untracked()));` にする（disabled の検査は trigger の part に移す）。
     - `base_trigger`：`let settings = ..;`、`let hosts = ..;`、`let disabled = move || {..};` を消し、`Part::new` を次にする（`.disabled(disabled)` の行を消す。node 自身の `.disabled(..)` が唯一の出所になる）。
       ```rust
       Part::new(part_state, move |mut host| {
           let disabled = trigger_disabled(&mut host);
           host.part(input)
               .role(Role::Button)
               .expanded(shown)
               .controls(self.content)
               .focusable(move || !disabled.get())
       })
       ```
     - `trigger()` の `Part::new` を次にする（`attach_part_state` の行は U31 まで残す）。
       ```rust
       Part::new(part_state, move |mut host| {
           let disabled = trigger_disabled(&mut host);
           let write = move |open: Option<bool>| {
               if !disabled.get_untracked() {
                   self.write(open);
               }
           };
           host.part(base)
               .node_ref(self.anchor)
               .on_press(move || write(None))
               .a11y_action(A11yAction::Toggle, move |_| write(None))
               .a11y_action(A11yAction::Expand, move |_| write(Some(true)))
               .a11y_action(A11yAction::Collapse, move |_| write(Some(false)))
       })
       ```
     - `surface` を足す（Popup・Dialog・HoverCard・NavigationPanel・Select・Combobox・Palette も同じ形。違うのは anchor と content の part だけ）。
       ```rust
       /// Mount `content` at the trigger while the disclosure is shown, in a
       /// fresh owner on every opening, with the content part on the overlay.
       #[track_caller]
       pub fn surface<V: IntoView>(self, placement: Placement, content: impl Fn() -> V + 'static) -> View {
           kivact_core::view::show(self.is_open(), move || {
               kivact_core::view::overlay(self.anchor, placement, content())
                   .with_node(|node| node.part(self.content()))
           })
           .into_view()
       }
       ```
       import に `kivact_core::view::{IntoView, Placement, View}` を足す。
     - 型の doc の「A [`disabled`](Self::disabled) disclosure accepts no input or assistive technology action.」を「A trigger node made `.disabled(..)` before the part is attached accepts no input or assistive technology action.」にする。module doc の「[`DisclosureApi::hide`] writes `false` to it」を「`show`, `hide` and `toggle` write it」にする。
  2. **`focus_return.rs` と `popup/`**
     - `focus_return.rs`：`pub enum FocusReturnTarget` に variant を足す。
       ```rust
       /// The reference this value holds when the surface opens.
       AnchorValue(kivact_core::state::Value<NodeRef>),
       ```
       `impl FocusReturn` に足す。
       ```rust
       /// Restore the node `target` holds when the surface opens, on Escape, a
       /// selection, assistive technology's dismissal, an explicit close and an
       /// unmount.
       pub fn anchor_value(target: kivact_core::state::Value<NodeRef>) -> Self {
           Self {
               target: FocusReturnTarget::AnchorValue(target),
               on: ReasonSet::DISMISS
                   .union(ReasonSet::PROGRAMMATIC)
                   .union(ReasonSet::UNMOUNT),
           }
       }
       /// This policy with an `AnchorValue` target read now, for one opening.
       pub(crate) fn resolved(self) -> Self {
           match self.target {
               FocusReturnTarget::AnchorValue(value) => Self {
                   target: FocusReturnTarget::Anchor(value.get_untracked()),
                   on: self.on,
               },
               _ => self,
           }
       }
       ```
       `fn request` の `match self.target` に腕 `FocusReturnTarget::AnchorValue(value) => value.get_untracked().restore_focus(),` を足す。
     - `popup/machine.rs`：`impl PartialEq for FocusPolicy` の match に腕 `(FocusReturnTarget::AnchorValue(a), FocusReturnTarget::AnchorValue(b)) => a == b,` を足す。
     - `popup/api.rs`：
       - `fn return_target` の `match policy.target` に腕 `FocusReturnTarget::AnchorValue(value) => value.get_untracked().mounted_node(),` を足す。
       - `struct PopupOptions` の `return_focus_target: Option<Value<NodeRef>>,` と `pub(crate) fn return_focus_target` を消す。`fn base_policy` の本体を `self.settings.get().focus_return.unwrap_or_else(|| FocusReturn::anchor(self.anchor_ref())).resolved()` にする（開くたびに値を読む今の意味を保つ）。
       - `PopupApi` に field `title: NodeRef, description: NodeRef` を足し、`use_popup` の literal で `node_ref()` にする。
       - `pub fn hide(self, reason: PopupCloseReason)` を `pub(crate) fn close(self, reason: PopupCloseReason)` に改名する（本体はそのまま）。crate の中の呼出しを全部 `.close(` にする：`rg -n "\.hide\(PopupCloseReason|popup\.hide\(reason" crates/kivact-interaction/src` の全件（`popup/api.rs`・`dialog.rs`・`menu/session.rs`・`select/handle.rs`）。
       - private の `fn toggle` を消す。trigger の `on_press(move || self.toggle())` は下の pub の `toggle` を使う。
       - `pub(crate) fn focus_first(self) -> Self` を `pub fn initial_focus_first(self) -> Self` に改名し、doc `/// Start the focus on the surface's first focusable node at each opening. Setup only.` を付ける。
       - 次を足す。`title`・`description`・`close_trigger` の part の状態は `let open = self.is_open(); part(move |input| PopupState { input, open: open.get() })` で作る。
         ```rust
         /// Show: write `true` to the open state unless it is shown.
         pub fn show(self) {
             if !self.is_open().get_untracked() {
                 self.disclosure.open.set(true);
             }
         }
         /// Close as an explicit close of the application.
         pub fn hide(self) {
             self.close(PopupCloseReason::Programmatic);
         }
         /// Show a hidden popup, hide a shown one.
         pub fn toggle(self) {
             if self.is_open().get_untracked() { self.hide() } else { self.show() }
         }
         /// The title: a level 2 heading the surface is `labelled_by`.
         #[track_caller]
         pub fn title(self) -> Part<PopupState>
             // 本体は今の DialogApi::title：host.part(input).node_ref(self.title).role(Role::Heading).level(2_u32)
         /// A description the surface is `described_by`.
         #[track_caller]
         pub fn description(self) -> Part<PopupState>
             // 本体は今の DialogApi::description：host.part(input).node_ref(self.description)
         /// A button whose activation closes the popup as a selection.
         #[track_caller]
         pub fn close_trigger(self) -> Part<PopupState>
             // host.part(input).role(Role::Button).focusable(true).on_press(move || self.close(PopupCloseReason::Selection))
         #[track_caller]
         pub fn surface<V: IntoView>(self, placement: Placement, content: impl Fn() -> V + 'static) -> View
             // 本体は手順 1 の surface。anchor は show の closure の中で読む self.anchor_ref()、part は self.content()
         ```
       - `content_with` の `Part::new` の連鎖に `.labelled_by(self.title).described_by(self.description)` を足す。
       - 型の doc の例を `let view = (node().part(menu.trigger()).name("Options").child(text("Options")), menu.surface(Placement::Below, || text("Contents")));` にする。
  3. **`dialog.rs`**
     - `pub enum DialogMode { Modal, AlertModal, Drawer(Edge) }`：`Popover` を消す。
     - `enum DialogOptionsMode` を消す。`DialogApi` を次にする（`title`・`description` の field は popup に移ったので消す）。
       ```rust
       #[derive(Clone, Copy)]
       pub struct DialogApi {
           popup: PopupApi,
           // content_state は U31 まで今のまま残す
           content_state: kivact_core::state::Signal<Option<Value<PopupState>>>,
           mode: DialogMode,
           settings: OptionsCell<DialogOptions>,
       }
       ```
     - `use_dialog`：
       ```rust
       #[track_caller]
       pub fn use_dialog(open: Signal<bool>, kind: DialogMode) -> DialogApi {
           DialogApi {
               popup: use_popup(open).initial_focus_first(),
               content_state: kivact_core::state::signal(None),
               mode: kind,
               settings: OptionsCell::new(DialogOptions::default()),
           }
       }
       ```
     - 消すもの：`fn popover`、`fn bind`（popover の anchor の misuse の検査）、`pub fn anchor`、`content()` の `DismissReason::OutsidePress` の腕の `if self.popover() { self.popup.set_opening_policy(..) }`（腕は `self.rules().outside` だけになる）、`content()` の `.on_attach(move |_| self.bind())`。
     - `rules()` の `DialogOptionsMode::AlertModal` は `DialogMode::AlertModal` にする。
     - `content()` の `Part::new` の中を `host.part(surface).modal().role(match self.mode { DialogMode::AlertModal => Role::AlertDialog, _ => Role::Dialog })` にする（`labelled_by`・`described_by` は popup の content が付ける）。
     - `title()` の本体を `self.popup.title()`、`description()` の本体を `self.popup.description()` にする。`close_trigger()` は変えない。
     - `fn close(self, _reason: PopupCloseReason)` の本体を `self.popup.open_binding().set(false);` にする（modal の focus の復帰は overlay の cleanup が行う）。
     - `return_focus` の本体を `self.popup.focus_return(FocusReturn::anchor_value(target.into_value())); self` にする。
     - 次を足す。
       ```rust
       /// Show the dialog unless it is shown.
       pub fn show(self) { self.popup.show() }
       /// Hide the dialog unless it is hidden.
       pub fn hide(self) {
           if self.is_open().get_untracked() {
               self.popup.open_binding().set(false);
           }
       }
       /// Show a hidden dialog, hide a shown one.
       pub fn toggle(self) {
           if self.is_open().get_untracked() { self.hide() } else { self.show() }
       }
       pub(crate) fn anchor_ref(self) -> NodeRef { self.popup.anchor_ref() }
       #[track_caller]
       pub fn surface<V: IntoView>(self, placement: Placement, content: impl Fn() -> V + 'static) -> View
           // 本体は手順 1 の surface。anchor は self.anchor_ref()、part は self.content()
       ```
     - 型の doc：表から `DialogMode::Popover` の行を消して 3 行にする。「at the anchor for a popover (the node with the [`trigger`](Self::trigger) part, or the [`anchor`](Self::anchor))」と「A popover is a [`PopupApi`]: … which keeps the pressed destination.」の文を消す。doc の例はそのまま。
     - module doc を次にする。
       ```
       //! Role: Dialog attaches modal, alert and drawer surface semantics.
       //!
       //! [`use_dialog`] is an unstyled modal dialog, alert dialog or drawer on a
       //! popup (`crate::popup`); its parts are `content`, `title`, `description`
       //! and `close_trigger`, with [`PopupState`]. The name, the scrim, the
       //! panel, the drawer's edge, the body's scrolling and the enter and exit
       //! animations are the caller's style (`Style::enter`, `Style::exit`). The
       //! dialog keeps the focus in and restores it through the engine's modal
       //! and the overlay's cleanup, which this module does not repeat. The
       //! initial focus (the given node, or the surface's first focusable one) is
       //! requested after the mount through the core's restore path. A surface
       //! dismissal is admitted by the [`DismissOn`] rules (assistive technology
       //! always; an alert modal's default refuses an outside press), and an
       //! admitted one only writes `false` to the open state. A popover is a
       //! [`PopupApi`] with its `title`, `description` and `close_trigger` parts.
       ```
  4. **`inline_edit.rs`**
     - `pub struct InlineEditApi<K: ItemKey> { rename: MachineHandle<Rename<K>>, editing: Value<Option<K>> }`（手書きの `Clone`・`Copy` はそのまま）。
     - `use_inline_edit` の最後を `let current = memo(move || rename.get().key()); InlineEditApi { rename, editing: current }` にする。
     - `pub fn show(self, key: K)` を `pub fn edit(self, key: K)` に改名する。
     - `pub fn is_open(self, key: &K) -> bool` を消し、次を足す。
       ```rust
       /// The key being renamed, or requested for renaming.
       pub fn editing(self) -> Value<Option<K>> { self.editing }
       /// End the rename without committing.
       pub fn cancel(self) { self.rename.send(Event::Request(None)); }
       ```
     - `field(..)` の `draft: Signal<String>` は W5 のまま変えない。
  5. **`tooltip/api.rs`**
     - `pub fn use_tooltip(enabled: impl IntoValue<bool>) -> TooltipApi`（`#[track_caller]` を付ける）。`TooltipApi` の field `options: OptionsCell<TooltipOptions>` を `enabled: Value<bool>` にし、literal は `enabled: enabled.into_value()` にする。
     - `fn attach` の分解を `let TooltipApi { enabled, attached, anchor, surface, hub, id, config, .. } = self;` にし、`engaged` を `memo(move || enabled.get() && (hover.get() || surface_hover.get() || focus.get()))` にする。
     - 消すもの：`struct TooltipOptions`、`pub fn disabled`、`use crate::options_cell::OptionsCell;`。
     - doc の例は `use_tooltip(true)` にする。
  6. **ui `dialog.rs`（`DialogKind` を interaction の型へ対応させる所だけ）**
     - private の enum を足す。
       ```rust
       /// The behavior a dialog's kind is built on.
       #[derive(Clone, Copy)]
       enum Behavior {
           Dialog(kivact_interaction::DialogApi),
           Popover(kivact_interaction::PopupApi),
       }
       impl Behavior {
           fn trigger(self) -> Part<PopupState> {
               match self { Self::Dialog(d) => d.trigger(), Self::Popover(p) => p.trigger() }
           }
           // title、description、close_trigger、content（Part<PopupState>）と is_open（Value<bool>）も同じ形で各 variant の同名 method へ委ねる
       }
       ```
     - `render` の最初の `let mut behavior = kivact_interaction::use_dialog(self.open, self.kind);` から `if let Some(target) = self.return_focus { .. }` までと、U23 が書いた `DialogKind` → `DialogMode` の `match` を、次に置き換える。続く `let mut anchor = node_ref();` は `let anchor = node_ref();` にしてこの塊の前に移し、trigger の map の中の `anchor = node_ref();` の再代入を消す。
       ```rust
       let anchor = node_ref();
       let behavior = match self.kind {
           DialogKind::Popover => {
               let popup = kivact_interaction::use_popup(self.open)
                   .anchor(anchor)
                   .initial_focus_first()
                   .focus_return(kivact_interaction::FocusReturn::anchor_value(
                       self.return_focus.unwrap_or_else(|| anchor.into_value()),
                   ));
               Behavior::Popover(match self.initial_focus {
                   Some(target) => popup.initial_focus(target),
                   None => popup,
               })
           }
           kind => {
               let mode = match kind {
                   DialogKind::Modal => DialogMode::Modal,
                   DialogKind::AlertModal => DialogMode::AlertModal,
                   DialogKind::Drawer(edge) => DialogMode::Drawer(edge),
                   DialogKind::Popover => unreachable!("invariant: matched above"),
               };
               let mut dialog = kivact_interaction::use_dialog(self.open, mode);
               if let Some(target) = self.initial_focus {
                   dialog = dialog.initial_focus(target);
               }
               if let Some(dismiss) = self.dismiss {
                   dialog = dialog.dismiss(dismiss);
               }
               if let Some(target) = self.return_focus {
                   dialog = dialog.return_focus(target);
               }
               Behavior::Dialog(dialog)
           }
       };
       ```
       popover は `.dismiss(DismissOn)` を読まない（popup の content はすべての閉じる理由を受け入れる。4.5 節で採った判断）。
     - `header`・`close_button`・`surface` の引数の型 `kivact_interaction::DialogApi` を `Behavior` にする（中の `behavior.title()` などの呼出しはそのまま）。
     - `place_panel`：`.with_node(|node| node.part(surface))` を、popover のときは `.with_node(|node| node.part(surface).role(Role::Dialog))` にする（popup の content は role を付けない。dialog の content は今までどおり自分で付ける）。
     - import：`use kivact_interaction::{DialogMode, DismissOn, PopupState};` はそのまま（`FocusReturn`・`use_popup`・`use_dialog` は上のとおり `kivact_interaction::` の path で書く）。`Part` は今の `use kivact_core::custom::Part;` を使い、`kivact_core::view::{..}` の import に `Role` を足す。
  7. **ui `disclosure.rs`**（`attach` と `into_view` の 2 か所）
      - 次の 5 行を消す。
        ```rust
        if let Some(disabled) = self.root.accessible_disabled() {
            details = details.disabled(move || always_open || disabled.get());
        } else if always_open {
            details = details.disabled(true);
        }
        ```
        代わりに次を書き、`let mut details` を `let details` にする。
        ```rust
        let disabled = match (self.root.accessible_disabled(), always_open) {
            (Some(d), _) => Some((move || always_open || d.get()).into_value()),
            (None, true) => Some(true.into_value()),
            (None, false) => None,
        };
        ```
      - `attach`：`trigger.with_node(|root| root.part(details.trigger()))` を `trigger.with_node(|root| match disabled { Some(d) => root.disabled(d), None => root }.part(details.trigger()))` にする。
      - `into_view`：trigger の `node().part(part).name(title)` を `match disabled { Some(d) => node().disabled(d), None => node() }.part(part).name(title)` にする（`.disabled` は必ず `.part` の前）。
      - `DisclosureApi::disabled` の呼出しはこの 2 か所だけである。
  8. **ui `tooltip.rs`**：`impl IntoView for Tooltip<T>`（U23 で `attach` から移した本体）の `kivact_interaction::use_tooltip().disabled(move || !self.enabled.get())` を `kivact_interaction::use_tooltip(self.enabled)` にする。
  9. **ui `data_table/rows.rs`**：`use_disclosure(open)` と `is_open()` だけを使うので変えない。
- **消すもの**：
  - interaction：`DialogMode::Popover`、`DialogOptionsMode`、`DialogApi::anchor`・`fn popover`・`fn bind`・`title`/`description` の field、`DisclosureApi::disabled`、`DisclosureOptions`、`DisclosureApi` の field `settings`・`host_disabled`、`fn is_disabled`・`fn register_host_disabled`、`TooltipApi::disabled`、`TooltipOptions`、`PopupApi::hide(reason)`（→ `pub(crate) close`）、private の `PopupApi::toggle`、`PopupApi::return_focus_target`、`PopupOptions::return_focus_target`、`InlineEditApi::{show, is_open}`。
  - ui：ui の `DialogKind` → `DialogMode::Popover` の変換。
- **状態の閉じ方**：
  - 名前の接尾辞で対象外：`DisclosureApi`、`PopupApi`、`DialogApi`、`InlineEditApi`、`TooltipApi`（Api）、`PopupOptions`、`DialogOptions`（Options）。
  - `FocusReturnTarget` は enum。
  - ui の `Behavior` enum は bool も Option も持たない。
  - ui の `Dialog` は値渡しの builder なので、6.1 節の対象ではない。
- **試験**：
  - `interaction_dialog.rs`：
    - `kinds()` を `[DialogMode; 4]` にして `DialogMode::Popover` を消す。
    - `framed` の `DialogMode::Popover => style().width(288)` の腕と `let popover = kind == DialogMode::Popover;` の分岐を消す。
    - `page_ordered` の `if kind == DialogMode::Popover { behavior.anchor(anchor) } else { .. }` を消し、`behavior` をそのまま使う。
    - `the_focus_starts_on_the_first_focusable_node_or_on_the_initial_focus` は `for kind in [DialogMode::Modal]` にする。
    - `a_popover_anchored_after_its_content_is_made_opens_and_returns_the_focus_to_the_anchor`：`let behavior = use_popup(open).initial_focus_first();` で書き直す。content は `.with_node(move |node| node.part(content).role(Role::Dialog))`、`let _ = behavior.anchor(anchor);` は残す。期待値は同じ。
    - 新設 `a_popover_popup_has_dialog_parts_and_returns_the_focus_to_its_anchor`：
      - 構成：`use_popup(open).anchor(anchor).initial_focus_first().focus_return(FocusReturn::anchor_value(anchor.into_value()))`。button「Open」（`.node_ref(anchor)`、`open.update(|v| *v = !*v)`）。overlay は `Role::Dialog` で `"Settings"` と名付ける。中に `title()` の text「Settings」、button「First」「Second」、`open.set(false)` を呼ぶ button「Done」。
      - 確かめること：
        - 開くと `get_by_role(Role::Dialog, "Settings")` があり、その `labelled_by()` の text が `["Settings"]`。
        - 「First」が focus を持つ。
        - Escape で閉じ、「Open」が focus を持つ。
        - 再び開いて「Done」を click すると閉じ、「Open」が focus を持つ。
        - `errors()` が空。
  - `interaction_show_hide.rs`：
    - 新設 `disclosure_popup_and_dialog_commands_are_idempotent`：`use_disclosure(signal(false))`、`use_popup(signal(false))`、`use_dialog(signal(false), DialogMode::Modal)` のそれぞれで、Show を 2 回 → open、Hide を 2 回 → closed、Toggle を 2 回 → closed を確かめる。
    - 新設 `popup_surface_mounts_the_content_at_its_trigger`：`let popup = use_popup(signal(false));`、`node().part(popup.trigger()).name("Options").child(text("Options"))` と `popup.surface(Placement::Below, || column(button("Inside")))`。「Options」を click すると `get_by_text("Inside")` がある。Escape で閉じ、「Options」が focus を持つ。
  - `interaction_tooltip.rs`：`use_tooltip()` を `use_tooltip(true)` にする。新設 `a_tooltip_built_disabled_shows_no_hint`：`use_tooltip(false)` の trigger に hover と focus をしても `is_open()` が false のまま。
  - `interaction_toggle.rs`：新設 `a_disabled_trigger_node_disables_the_disclosure`：`control("Details").disabled(true).part(details.trigger())` を click しても `open` が false のまま。
  - `ui_field.rs` の `inline_edit_opens_focused_commits_cancels_and_ends_with_its_row`：
    - `rename.show(id)` → `rename.edit(id)`。
    - `move || rename.is_open(&id)` → `move || rename.editing().get() == Some(id)`。
    - button「Cancel rename」（`rename.cancel()`）を足す。「Rename 2」の後でそれを押すと `editing` が `None` になり、「Name 2」の textbox が無くなることを確かめる。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction -p kivact-ui
  cargo check -p kivact-interaction --all-targets
  cargo check -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction --all-targets -- -D warnings
  cargo clippy -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo xtask check --no-write --crate kivact-ui
  KIVACT_TEST_MIRROR=off cargo test -p kivact-interaction
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_dialog interaction_show_hide interaction_tooltip interaction_toggle interaction_menu interaction_select ui_field ui_disclosure_surfaces ui_components ui_data_table ui_chat_code ui_calendar apg_tooltip apg_accordion apg_date_picker
  ```
- **完了条件**：
  - `rg -n "DialogMode::Popover|DialogOptionsMode|TooltipOptions|DisclosureOptions|return_focus_target" crates/kivact-interaction crates/kivact-ui dev/tests/kivact-public-api examples/kivact templates dev/bench` の結果が空。
  - `rg -n "host_disabled|fn is_disabled" crates/kivact-interaction/src/disclosure.rs` の結果が空（`scalar/edit_state.rs` の同名の field は W8 の U42 が消す）。
  - `rg -n "pub fn hide\(self, reason" crates/kivact-interaction` の結果が空。
  - `rg -n "DialogMode" crates/kivact-ui/src` が `dialog.rs` の render の `match`（`DialogMode::Modal`・`AlertModal`・`Drawer`）とその import だけを返す。
  - 上の新設 5 件と直した試験が通る。HoverCard と NavigationMenu の `on_open_change` はこの時点で残っている（U30c が消す）。

## U30c：開閉の語彙と `surface()`（HoverCard・NavigationMenu）

- **wave / lane / 前提**：W7b / U1 / U30a。U30a の前提（W5・W6 の API、W7a の全 unit、U11〜U14 の配置、e2e の binary）はこの unit にも当てはまる。加えて U30a の後の状態：`DisclosureApi`・`PopupApi` の開閉の語彙と `surface(placement, content)`、`PopupApi::close`（pub(crate)）、`use_tooltip(enabled)`。
- **所有**：
  - interaction：`crates/kivact-interaction/src/{hover_card.rs, navigation_menu.rs, lib.rs（`mod`・`pub use` の行だけ）}`。
  - ui：`crates/kivact-ui/src/{hover_card.rs, hover_card/look.rs, navigation_menu.rs, navigation_menu/link.rs, navigation_menu/context.rs, navigation_menu/look.rs}`。
  - 試験：`dev/tests/kivact-public-api/tests/e2e_ui/{interaction_show_hide.rs, ui_navigation.rs, ui_disclosure_surfaces.rs}`（U30a と同じ lane の後の unit）。
- **目的**：HoverCard と NavigationMenu の Api を `use_x(.., open: Signal<bool>)`、`show()`・`hide()`・`toggle()`、`is_open()`、`trigger()`・`content()`、`surface(placement, content)` に揃える（UI-12、UI-24、5.9 節）。`NavigationMenu`・`HoverCard` の `on_open_change` を消す（値は `Signal` で受ける。UI-13）。NavigationMenu は panel ごとの Api（`NavigationPanelApi`）を持つ。
- **手順**：
  1. **`hover_card.rs`**
     - 署名と型を次にする（`open` は最後の引数になる）。
       ```rust
       #[track_caller]
       pub fn use_hover_card(
           enabled: impl IntoValue<bool>,
           focus_opens: impl IntoValue<bool>,
           timing: HoverCardTiming,
           open: Signal<bool>,
       ) -> HoverCardApi
       #[derive(Clone, Copy)]
       pub struct HoverCardApi {
           surface: HoverSurfaceApi,
           state: Value<HoverCardState>,
           open: Signal<bool>,
           shown: Value<bool>,
           anchor: NodeRef,
           card: NodeRef,
       }
       ```
     - 本体の最初に次を置き、`focused` を引数の代わりにこの memo にする。`hover_surface(anchor, surface, ..)` の 2 つの `NodeRef` は `anchor`・`card` にする。他の行は今のまま。
       ```rust
       let anchor = node_ref();
       let card = node_ref();
       let (anchor_focus, card_focus) = (anchor.focus_within(), card.focus_within());
       let focus_opens = focus_opens.into_value();
       let focused = memo(move || focus_opens.get() && (anchor_focus.get() || card_focus.get()));
       ```
       literal に `anchor, card,` を足す。
     - 足すもの：
       ```rust
       /// Show the preview unless it is shown.
       pub fn show(self) {
           if !self.shown.get_untracked() {
               self.open.set(true);
           }
       }
       /// Show a hidden preview, hide a shown one.
       pub fn toggle(self) {
           if self.shown.get_untracked() { self.hide() } else { self.show() }
       }
       /// The node the preview is placed at and whose hover and focus open it.
       #[track_caller]
       pub fn trigger(self) -> Part<PopupState> {
           let shown = self.shown;
           let input = part(move |input| PopupState { input, open: shown.get() });
           Part::new(input.state(), move |host| host.part(input).node_ref(self.anchor))
       }
       /// The preview surface: Escape and an outside press hide it, and
       /// Escape inside it moves the focus back to the trigger.
       #[track_caller]
       pub fn content(self) -> Part<PopupState> {
           let shown = self.shown;
           let card_focus = self.card.focus_within();
           let input = part(move |input| PopupState { input, open: shown.get() });
           Part::new(input.state(), move |host| {
               host.part(input)
                   .node_ref(self.card)
                   .a11y_action(A11yAction::Dismiss, move |event| {
                       let A11yRequest::Dismiss(reason) = event.request else {
                           unreachable!("invariant: the Dismiss slot receives only Dismiss requests")
                       };
                       if reason == DismissReason::Escape && card_focus.get_untracked() {
                           self.anchor.focus();
                       }
                       self.hide();
                   })
           })
       }
       #[track_caller]
       pub fn surface<V: IntoView>(self, placement: Placement, content: impl Fn() -> V + 'static) -> View
           // 本体は U30a の手順 1 の surface。anchor は self.anchor、part は self.content()
       ```
       `hide` は今のまま。import は `popup/api.rs` の `content_with` が使っているものと同じ（`A11yAction`、`A11yRequest`、`DismissReason`、`Part`、`part`、`PopupState`）。
  2. **`navigation_menu.rs`**：`changed` と `on_open_change` を消し、`open: Signal<bool>` を足す。panel ごとの Api を新設する。
     ```rust
     #[track_caller]
     pub fn use_navigation_menu(open: Signal<bool>) -> NavigationMenuApi
     #[derive(Clone, Copy)]
     pub struct NavigationMenuApi {
         machine: MachineHandle<Machine>,
         open: Signal<bool>,
         next_panel: OptionsCell<usize>,
     }
     impl NavigationMenuApi {
         pub fn state(self) -> Value<NavOpen>          // 今のまま
         pub fn is_open(self) -> Value<bool>           // self.open.into_value()
         pub fn show(self)                             // self.machine.send(Event::ShowFirst)
         pub fn hide(self)                             // self.machine.send(Event::Close)
         pub fn toggle(self)                           // self.open を untracked に読み、真なら hide、偽なら show
         #[track_caller] pub fn panel(self) -> NavigationPanelApi   // 下の本体。Setup only
     }
     #[derive(Clone, Copy)]
     pub struct NavigationPanelApi {
         menu: NavigationMenuApi,
         key: usize,
         shown: Value<bool>,
         popup: PopupApi,
     }
     impl NavigationPanelApi {
         pub fn key(self) -> usize
         pub fn is_open(self) -> Value<bool>           // self.shown（Hover(key) か Pinned(key) の間）
         pub fn show(self)                             // self.menu.machine.send(Event::Pin(self.key))
         pub fn hide(self)                             // self.shown を untracked に読み、真なら self.menu.machine.send(Event::Close)
         pub fn toggle(self)                           // self.menu.machine.send(Event::Toggle(self.key))
         pub fn hover(self)                            // Event::Hover(self.key)
         pub fn leave(self)                            // Event::Leave(self.key)
         pub fn removed(self)                          // Event::PanelRemoved(self.key)
         pub fn hover_after(self, delay: Duration)     // Event::HoverAfter(self.key, delay)
         pub fn leave_after(self, delay: Duration)     // Event::LeaveAfter(self.key, delay)
         #[track_caller] pub fn trigger(self) -> Part<PopupState>
             // let base = self.popup.trigger(); Part::new(base.state(), move |host| host.part(base).expanded(self.shown))
         #[track_caller] pub fn content(self) -> Part<PopupState>   // self.popup.content()
         #[track_caller] pub fn surface<V: IntoView>(self, placement: Placement, content: impl Fn() -> V + 'static) -> View
             // show(self.shown, ..)、overlay(self.popup.anchor_ref(), placement, content())、part は self.content()
     }
     ```
     - `panel()` の本体（ui の `pinned_binding` をここへ移したもの）：
       ```rust
       let key = self.next_panel.get();
       self.next_panel.update(|next| {
           *next = next.checked_add(1).expect("navigation panel identity exhausted");
       });
       let state = self.state();
       let pinned = binding(
           move || state.get() == NavOpen::Pinned(key),
           move |pin: bool| {
               if pin {
                   self.machine.send(Event::Pin(key));
               } else if state.get_untracked() == NavOpen::Pinned(key) {
                   self.machine.send(Event::Close);
               }
           },
       );
       let shown = memo(move || matches!(state.get(), NavOpen::Hover(i) | NavOpen::Pinned(i) if i == key));
       NavigationPanelApi { menu: self, key, shown, popup: use_popup(pinned) }
       ```
     - `enum Event` に `ShowFirst` を足し、`transition` の match に腕 `Event::ShowFirst if state.open == NavOpen::Closed => activate(*state, NavOpen::Pinned(0)),` を足す（開いているときは既存の `_ => Step::stay()`）。
     - `use_navigation_menu` の本体：machine の effect を `move |_, Effect::Changed(v)| if open.get_untracked() != v { open.set(v) }` にし、machine を作った後に `machine.feed(open, |v| if v { Event::ShowFirst } else { Event::Close });` を置く。
     - 旧 pub の `panel_key`、`on_open_change`、`panel_removed(key)`、`hover_after(index, ..)`、`leave_after(index, ..)`、`hover(index)`、`leave(index)`、`toggle(index)`、`show(index)` を消す（panel の Api へ移った）。`use crate::options_cell::OptionsCell;` は `next_panel` が使うので残し、`Callback`・`callback` の import を消す。
  3. **`lib.rs`（interaction）**：`pub use navigation_menu::{NavOpen, NavigationMenuApi, NavigationPanelApi, use_navigation_menu};` にする。module doc の開閉の段落は U30b の手順 9 で書き換えるので、ここでは触らない。
  4. **ui `hover_card.rs`**
      - 消すもの：field と setter の `on_open_change`、`if let Some(callback) = self.on_open_change { effect_on(shown, ..); }` の 3 行、`effect_on` の import。
      - `open(mut self, open: Signal<bool>)` と `CardOpen::Controlled(Signal<bool>)` は W5 のまま。
      - `attach`：
        - `let card = node_ref();`、`let anchor_focus = ..;`、`let card_focus = ..;` を消す。`let anchor = node_ref();` は overlay の anchor に使うので残す。
        - `use_hover_card(..)` の呼出しを `let behavior = use_hover_card(self.enabled, move || focus != FocusTrigger::Never, self.timing, requested);` にする。
        - `let trigger = trigger.node_ref(anchor);` を `let trigger = trigger.node_ref(anchor).with_node(|n| n.part(behavior.trigger()));` にする（node は二つの ref を持つ）。
        - overlay の `.node_ref(card).on_dismiss(move |reason| {..})` を `.with_node(|n| n.part(behavior.content()))` にする。
        - その後の `match self.label { .. }` の role の分岐はそのまま。
  5. **ui `navigation_menu.rs`**
      - field の `changed` と setter の `on_open_change` を消し、次にする。
        ```rust
        open: Option<Signal<bool>>,
        /// Use this signal as the menu's open state: true while any panel is open.
        pub fn open(mut self, open: Signal<bool>) -> Self { self.open = Some(open); self }
        ```
      - `into_view`：`let api = use_navigation_menu(open.unwrap_or_else(|| signal(false)));`（`if let Some(changed)` の分岐を消す）。
      - panel ごと：`let at = api.panel_key();`、`let shown = memo(..);`、`let pinned = pinned_binding(api, at);`、`let pop = use_popup(pinned);` を `let panel = api.panel(); let shown = panel.is_open();` にする。`fn pinned_binding` と `use_popup` の import を消す。
      - `struct PanelPresentation` に field `state: Value<NavOpen>` を足し、`into_view` で `state` を入れる。
      - `navigation_panel` と `navigation_trigger` の引数 `api`・`at`・`pop` を `panel: kivact_interaction::NavigationPanelApi` 一つにする（`anchor` は残す）。中では次のとおり置き換える。
        - `navigation_panel` の `let state = api.state();` → `let state = presentation.state;`（分解の `..` の前に `state` を足す）
        - `callback(move |()| api.hide())` → `callback(move |()| panel.hide())`
        - `api.hover_after(at, d)` → `panel.hover_after(d)`、`api.leave_after(at, d)` → `panel.leave_after(d)`
        - `api.show(at)` → `panel.show()`、`api.hide()` → `panel.hide()`
        - `pop.trigger()` → `panel.trigger()`、`pop.content()` → `panel.content()`
      - trigger の `.with_node(|n| n.expanded(shown))` を消す（`panel.trigger()` が付ける）。
      - `navigation_action` は menu の `api` のまま（`api.hide()` は全体を閉じる）。
- **消すもの**：
  - interaction：`NavigationMenuApi::{on_open_change, panel_key, panel_removed, hover_after, leave_after, hover, leave}` と index を取る `show`・`toggle`、field `changed`、HoverCard の `on_open_change`。
  - ui：`HoverCard::on_open_change`、`NavigationMenu::on_open_change`、`fn pinned_binding`。
- **状態の閉じ方**：
  - 名前の接尾辞で対象外：`HoverCardApi`、`NavigationMenuApi`、`NavigationPanelApi`（Api）。
  - `NavMachineState` は変えない。
  - ui の `PanelPresentation` は数える field を持たない（`Value` だけ）。ui の `NavigationMenu`・`HoverCard` は値渡しの builder なので、6.1 節の対象ではない。
- **試験**：
  - `interaction_show_hide.rs`：
    - `navigation_menu_show_is_idempotent_and_toggle_twice_returns`：`let menu = use_navigation_menu(signal(false)); let first = menu.panel();` にする。「Show twice」は `first.show()` を 2 回、「Toggle twice」は `first.toggle()` を 2 回、「Hide twice」は `menu.hide()` を 2 回。期待値（`NavOpen::Pinned(0)`、`NavOpen::Closed`）は同じ。
  - `ui_navigation.rs`・`ui_disclosure_surfaces.rs` の hover card と navigation menu の試験が期待値を変えずに通ること（`.on_open_change(..)` を使う行は `.open(signal)` とその signal の読みに直す。期待値は同じ）。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction -p kivact-ui
  cargo check -p kivact-interaction --all-targets
  cargo check -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction --all-targets -- -D warnings
  cargo clippy -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo xtask check --no-write --crate kivact-ui
  KIVACT_TEST_MIRROR=off cargo test -p kivact-interaction
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_show_hide ui_navigation ui_disclosure_surfaces
  ```
- **完了条件**：
  - `rg -n "on_open_change|fn panel_key|fn pinned_binding" crates/kivact-interaction crates/kivact-ui dev/tests/kivact-public-api examples/kivact templates dev/bench` の結果が空。
  - 上の試験が通る。

## U30b：開閉の語彙と `surface()`（Menu・Select・Combobox・Palette）、lib の doc

- **wave / lane / 前提**：W7b / U1 / U30c。U30a の前提（W5・W6 の API、W7a の全 unit、U11〜U14 の配置、e2e の binary）はこの unit にも当てはまる。加えて次が済んでいる。
  - U05（W3）：`combobox/machine.rs` は `enum ComboState<K>`、`Event::{Open, OpenSheet, Close, OpenRequested(bool), FocusLeft { transferred }, Query(String), QueryChanged, Highlight(K), Toggled}`、`Effect::{WriteQuery(String), WriteOpen(bool)}`、`Machine::new(inline: bool)` になっている。`ComboboxApi::follow_open` は消え、`pub(crate) fn opened_by(self, open)` があり、palette が `combobox.opened_by(open);` を呼ぶ。
  - U10（W4）：`ComboboxApi` は `settings: OptionsCell<ComboboxOptions<K>>` を持ち、private の `struct ComboboxOptions<K>` の field は `source`・`debounce`・`search_status`・`presentation: PresentationMode`・`query: Signal<String>` である（`self.settings.get().query`、`self.settings.get().presentation` で読む）。
  - U23（W7a）：`ComboboxMode` は ui の型で、interaction の `ComboboxMode` は消えている（`lib.rs` の combobox の行は `pub use combobox::{ComboboxApi, use_combobox};`）。
  - U12（W4）：ui の `select/handle.rs` の中身は `select.rs`（module root）に入っている。`select/machine.rs` と `select/presentation.rs` が残る。
  - U30a・U30c：`PopupApi::{show, hide, toggle, surface, close（pub(crate)）}`、`DialogApi::anchor_ref`（pub(crate)）がある。
- **所有**：
  - interaction：`crates/kivact-interaction/src/{menu.rs, menu/handle.rs, menu/session.rs, menu/level.rs, menubar.rs（doc の例だけ）, select/handle.rs, combobox.rs, combobox/handle.rs, combobox/machine.rs, palette/handle.rs, lib.rs}`
  - ui：`crates/kivact-ui/src/{menu.rs, select.rs, combobox/render.rs}`。`combobox/notices.rs` は W7b では U32 が持つので変えない。
  - 試験：`dev/tests/kivact-public-api/tests/e2e_ui/{interaction_combobox.rs, interaction_show_hide.rs, interaction_menu.rs, interaction_palette.rs, interaction_select.rs, apg_combobox.rs, apg_listbox.rs, apg_menu.rs, apg_menu_button.rs, apg_menubar.rs}`
- **目的**：次を行う（UI-12、UI-24、UI-05 の公開部分、UI-04 規則 2）。
  - Menu・Select・Combobox・Palette の open を、位置引数の `Signal<bool>` にする。
  - `while_open` を `surface` に置き換える。
  - combobox の `popup`・`popup_with_focus_transfer`・`query_binding`・`with_input_ref` を、`use_combobox(items, selected, query, open, ComboboxOptions)` と `input_for`・`content_for` にする。
- **手順**：
  1. **menu（interaction）**
     - `menu/handle.rs`：`use_menu_with_open` を消し、`pub fn use_menu<K: ItemKey>(items: impl IntoValue<Vec<MenuEntry<K>>>, open: Signal<bool>) -> MenuApi<K>` にする（本体は旧 `use_menu_with_open` の本体）。`menu.rs` と `lib.rs` の `pub use` から `use_menu_with_open` を消す。
     - `pub fn toggle(self)` を足す：`self.top_level("toggle"); if self.session.is_open().get_untracked() { self.hide() } else { self.show() }`。
     - `while_open` を消して次を足す。
       ```rust
       /// Mount `content` while this level is open, in a fresh owner on every
       /// opening, with the content part on its overlay: a submenu at its item,
       /// the top level at the pointer for a pointer request and at the trigger
       /// otherwise (built again when the request changes between the two).
       #[track_caller]
       pub fn surface<V: IntoView>(self, placement: Placement, content: impl Fn() -> V + 'static) -> View {
           let content = std::rc::Rc::new(content);
           let (session, level) = (self.session, self.level);
           if level.depth > 0 {
               let prefix = level.prefix;
               return kivact_core::view::show(level.shown, move || {
                   let anchor = session.item_anchor(&prefix.get_untracked());
                   kivact_core::view::overlay(anchor, placement, content())
                       .with_node(|n| n.part(self.content()))
               })
               .into_view();
           }
           let origin = session.origin;
           let at_pointer = memo(move || matches!(origin.get(), Some(MenuOrigin::Pointer(_))));
           kivact_core::view::show(level.shown, move || {
               let content = content.clone();
               kivact_core::view::dynamic(at_pointer, move |pointer: bool| {
                   let layer = if pointer {
                       kivact_core::view::overlay_at(
                           move || match origin.get() {
                               Some(MenuOrigin::Pointer(point)) => point,
                               _ => kivact_core::geometry::Point::new(0.0, 0.0),
                           },
                           placement,
                           content(),
                       )
                   } else {
                       kivact_core::view::overlay(session.anchor, placement, content())
                   };
                   layer.with_node(|n| n.part(self.content()))
               })
           })
           .into_view()
       }
       ```
     - `menu/session.rs`：
       - field `pub(super) item_anchors: Signal<Vec<(Vec<K>, NodeRef)>>` を足し、`Session::new` の literal で `signal(Vec::new())` にする。
       - 次を足す。
         ```rust
         /// The node of the mounted item that opens the submenu at `path`.
         pub(super) fn item_anchor(&self, path: &[K]) -> NodeRef {
             self.item_anchors
                 .with_untracked(|all| all.iter().find(|(p, _)| p.as_slice() == path).map(|(_, r)| *r))
                 .unwrap_or_else(|| {
                     raise(ErrorKind::Message, "MenuApi::surface: the submenu's item is not mounted");
                     node_ref()
                 })
         }
         ```
       - `open_from_button` と `open_at` の `self.popup.open_binding().set(true)` はそのまま（pub(crate)）。`.hide(` → `.close(` は U30a で済んでいる。
     - `menu/level.rs` の `fn item`：`let item_ref = kivact_core::view::node_ref();` の直後に `if submenu { register_item_anchor(session, self.prefix.get_untracked(), key.clone(), item_ref); }` を足し、file の末尾に次を置く。
       ```rust
       /// Record `item` as the anchor of the submenu at `path` + `key` while the
       /// item is mounted; only that same reference is removed at its cleanup.
       fn register_item_anchor<K: kivact_core::state::ItemKey>(session: Session<K>, mut path: Vec<K>, key: K, item: NodeRef) {
           path.push(key);
           let anchors = session.item_anchors;
           anchors.update(|all| all.push((path, item)));
           on_cleanup(move || {
               if anchors.try_get_untracked().is_some() {
                   anchors.update(|all| all.retain(|(_, r)| *r != item));
               }
           });
       }
       ```
     - doc：`menu/handle.rs` の型の doc と doc の例を `use_menu(entries, signal(false))`、`file.surface(Placement::Below, move || column(rows))` にする。`while_open` に触れる文（`menu/handle.rs`、`menu.rs` の module doc）は `surface` にする。`menubar.rs` の doc の例の `use_menu(vec![..])` 2 か所を `use_menu(vec![..], signal(false))` にする。
  2. **ui `menu.rs`**
     - `Menu` の field `behavior: Behavior<K>` を消し、次を足す。
       ```rust
       items: Value<Vec<MenuEntry<K>>>,
       choose: Option<Callback<K>>,
       open: Option<Signal<bool>>,
       ```
     - import に `use kivact_core::state::{Callback, Signal, signal};` を足し、`MenuApi as Behavior` の import は `fn surface` などの引数の型が使うので残す。
     - `menu()` の literal は `items: items.into_value(), choose: None, open: None`。
     - `on_activate<R: HandlerResult>(mut self, choose: impl Fn(K) -> R + 'static)` の本体を `self.choose = Some(kivact_core::state::callback(choose)); self` にする。
     - setter を足す：`/// Use this signal as the menu's open state.` `pub fn open(mut self, open: Signal<bool>) -> Self { self.open = Some(open); self }`。
     - `render` の先頭に次を置き、以降の `self.behavior` を `behavior` にする。
       ```rust
       let behavior = kivact_interaction::use_menu(self.items, self.open.unwrap_or_else(|| signal(false)));
       let behavior = match self.choose {
           Some(choose) => behavior.on_activate(move |key| choose.call(key)),
           None => behavior,
       };
       ```
     - `fn surface` の `behavior.while_open(move || { .. })` を次にする。`build` の中は旧 closure の本体そのまま。
       ```rust
       let shown = behavior.is_open();
       let origin = behavior.origin();
       let at_pointer = kivact_core::state::memo(move || matches!(origin.get(), Some(MenuOrigin::Pointer(_))));
       let build = std::rc::Rc::new(move || { /* 旧 closure の本体 */ });
       if kind == LevelKind::Context {
           kivact_core::view::show(shown, move || {
               let build = build.clone();
               kivact_core::view::dynamic(at_pointer, move |_| build())
           })
           .into_view()
       } else {
           kivact_core::view::show(shown, move || build()).into_view()
       }
       ```
  3. **select（`select/handle.rs`）**
     - 署名：`pub fn use_select<K: ItemKey>(items: impl IntoValue<Vec<ListItem<K>>>, selected: Signal<Option<K>>, open: Signal<bool>) -> SelectApi<K>`。`SelectApi` に field `open: Signal<bool>` を足す。
     - `let open = binding(memo(..State::Open..), ..);` の塊を消し、literal の `popup` を `use_popup(open).anchor_starts_focus_range(true)` のまま（`open` は引数）にし、`open,` を足す。
     - machine を作った直後に次を置く。
       ```rust
       if open.get_untracked() {
           session.send(machine::Event::Open(machine::OpenedBy::Pointer));
       }
       session.feed(open, |shown| if shown { machine::Event::Open(machine::OpenedBy::Pointer) } else { machine::Event::Close });
       ```
     - 次を足す（machine は開いている回の詳細だけを持つ。開閉の正本は `open`）。
       ```rust
       /// Open with `event`'s details, then write the open state; a rejected
       /// write closes the machine again.
       fn open_with(self, event: machine::Event<K>) {
           self.session.send(event);
           if !self.open.get_untracked() {
               self.open.set(true);
           }
           if !self.open.get_untracked() {
               self.session.send(machine::Event::Close);
           }
       }
       ```
     - 置き換え：
       - `show()` の本体 → `if !self.open.get_untracked() { self.open_with(machine::Event::Open(machine::OpenedBy::Pointer)) }`
       - `show_sheet()` の `self.session.send(machine::Event::OpenSheet)` → `self.open_with(machine::Event::OpenSheet)`
       - `hide()` の本体 → `self.popup.close(PopupCloseReason::Programmatic)`
       - file 内の `popup.open_binding().set(true)`（Expand、Alt+Arrow、F4）→ `self.open_with(machine::Event::Open(machine::OpenedBy::Pointer))`
       - `session.send(machine::Event::Open(machine::OpenedBy::Key(event.key())))` → `self.open_with(machine::Event::Open(machine::OpenedBy::Key(event.key())))`
       - `popup.open_binding().set(false)` → `self.open.set(false)`
     - `pub fn open_binding` を消す。
     - `surface` を足す。本体は U30a の手順 1 と同じで、anchor は `self.popup.anchor_ref()`、part は `self.content()`。
     - doc の例 2 か所の `use_select(a, b)` を `use_select(a, b, signal(false))` にする。
  4. **ui `select.rs`**
     - `SelectOptions<K>` に field `open: Option<Signal<bool>>` を足し（literal は `None`）、`/// Use this signal as the select's open state.` `pub fn open(mut self, open: Signal<bool>) -> Self { self.settings.open = Some(open); self }` を足す。
     - `configure_select` を `-> (kivact_interaction::SelectApi<K>, Signal<bool>)` にし、最初の行を `let open = settings.open.unwrap_or_else(|| signal(false)); let mut behavior = kivact_interaction::use_select(settings.options, settings.selected, open)` にし、最後を `(behavior, open)` にする。
     - `into_view`：`let behavior = configure_select(&self.settings);` と `let open = behavior.open_binding();` を `let (behavior, open) = configure_select(&self.settings);` にする。
     - `touch_sheet` の factory の引数の型は W5 で `Signal<bool>` になっている（変えない）。
  5. **combobox の machine**：`combobox/machine.rs` の中身を全部、次にする。
     ```rust
     //! Role: Own the combobox list's per-opening presentation, filter phase and highlight.
     use crate::surface::Surface;
     use kivact_core::state::{Machine as MachineTrait, Step};
     use std::marker::PhantomData;

     /// How the current opening presents the list; the caller's `open` says whether it shows.
     #[derive(Clone, PartialEq)]
     pub(super) enum ComboState<K> {
         Closed,
         Open { surface: Surface, filtering: bool, highlight: Option<K> },
     }
     impl<K> ComboState<K> {
         pub(super) fn surface(&self) -> Option<Surface> {
             match self { Self::Open { surface, .. } => Some(*surface), Self::Closed => None }
         }
         pub(super) fn filtering(&self) -> bool {
             matches!(self, Self::Open { filtering: true, .. })
         }
         pub(super) fn highlight(&self) -> Option<&K> {
             match self { Self::Open { highlight, .. } => highlight.as_ref(), Self::Closed => None }
         }
     }
     pub(super) enum Event<K> {
         /// The caller's `open` became true.
         Opened,
         /// Show the list in this surface; an open list moves to it.
         Present(Surface),
         /// The caller's `open` became false.
         Closed,
         /// The user typed into the field.
         Typed,
         /// The query changed while the list is open.
         QueryChanged,
         Highlight(K),
     }
     pub(super) enum Effect {
         /// The list closed.
         Closed,
     }
     pub(super) struct Machine<K>(PhantomData<fn() -> K>);
     impl<K> Machine<K> {
         pub(super) fn new() -> Self { Self(PhantomData) }
     }
     impl<K: Clone + PartialEq + 'static> MachineTrait for Machine<K> {
         type State = ComboState<K>;
         type Event = Event<K>;
         type Effect = Effect;
         fn transition(&self, state: &ComboState<K>, event: Event<K>) -> Step<ComboState<K>, Event<K>, Effect> {
             use ComboState::{Closed, Open};
             match (state, event) {
                 (Closed, Event::Opened) => Step::to(Open { surface: Surface::Popover, filtering: false, highlight: None }),
                 (Closed, Event::Present(surface)) => Step::to(Open { surface, filtering: false, highlight: None }),
                 (Open { highlight, .. }, Event::Present(surface)) => {
                     Step::to(Open { surface, filtering: false, highlight: highlight.clone() })
                 }
                 (Open { .. }, Event::Closed) => Step::to(Closed).with(Effect::Closed),
                 (Closed, Event::Typed) => Step::to(Open { surface: Surface::Popover, filtering: true, highlight: None }),
                 (Open { surface, .. }, Event::Typed | Event::QueryChanged) => {
                     Step::to(Open { surface: *surface, filtering: true, highlight: None })
                 }
                 (Open { surface, filtering, .. }, Event::Highlight(key)) => {
                     Step::to(Open { surface: *surface, filtering: *filtering, highlight: Some(key) })
                 }
                 _ => Step::stay(),
             }
         }
     }
     ```
     （消えるもの：U05 の `Event::{Open, OpenSheet, Close, OpenRequested, FocusLeft, Query, Toggled}`、`Effect::{WriteQuery, WriteOpen}`、`Machine` の field `inline` と `new(inline)` の引数、U05 の `fn open`・`fn close` などの helper。）
  6. **combobox の handle（`combobox/handle.rs`）**
     - U10 の private の `struct ComboboxOptions<K>` を `SearchOptions<K>` に改名する（下の公開の `ComboboxOptions` と名前が重なるため）。field の `presentation` と `query` を消し（`source`・`debounce`・`search_status` が残る）、`use_combobox` の `OptionsCell::new(..)` の literal からも 2 行を消す。手書きの `Clone`・`Copy` は名前だけ直す。`enum PresentationMode` を消す。
     - 公開の型と署名：
       ```rust
       /// Construction-time presentation of a combobox's list.
       #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
       pub struct ComboboxOptions {
           /// The list is always part of the layout (a palette, a picker): no popup keys, roles or focus-out close.
           pub inline: bool,
       }
       /// A combobox over `items` whose choice is written to `selected`, whose
       /// query is `query` and whose list shows while `open` is true. Setup only.
       #[track_caller]
       pub fn use_combobox<K: ItemKey>(
           items: impl IntoValue<Vec<ListItem<K>>>,
           selected: impl crate::IntoSelection<K>,
           query: Signal<String>,
           open: Signal<bool>,
           options: ComboboxOptions,
       ) -> ComboboxApi<K>
       ```
     - `ComboboxApi` に field `open: Signal<bool>, query: Signal<String>, inline: bool` を足す（`settings`、U31 まで残す `input_state`・`content_state`・`item_states` はそのまま）。literal に `open, query, inline: options.inline,` を足す。
     - 本体：machine の作成から U05 の `let query_text = memo(..);` までを次にする。
       ```rust
       let session = machine(
           if open.get_untracked() {
               machine::ComboState::Open { surface: Surface::Popover, filtering: false, highlight: None }
           } else {
               machine::ComboState::Closed
           },
           || machine::Machine::<K>::new(),
           move |_, machine::Effect::Closed| {
               // An inline list's next opening starts from an empty query; a
               // popup keeps its query because the field still shows it.
               if options.inline && !query.with_untracked(String::is_empty) {
                   query.set(String::new());
               }
           },
       );
       session.feed(open, |shown| if shown { machine::Event::Opened } else { machine::Event::Closed });
       kivact_core::state::effect_on(query, move |_| {
           if open.get_untracked() {
               session.send(machine::Event::QueryChanged);
           }
       });
       ```
       `shown` の memo の `let typed = query_text.get();` を `let typed = query.get();` にする。`active` の memo は U05 の形のまま（`state.highlight()` は新しい `ComboState` にもある）。
     - file 内の置換（U10 の後の形から）：`self.settings.get().query` → `self.query`、`matches!(self.settings.get().presentation, PresentationMode::Popup(_))` → `!self.inline`、`matches!(self.settings.get().presentation, PresentationMode::Inline)` → `self.inline`。`rg -n "presentation|PresentationMode|query_text" crates/kivact-interaction/src/combobox/handle.rs` が空になるまで直す。
     - 次の method を置き換える／足す（doc は短く付ける）。
       ```rust
       /// The query as the field edits it: a write is typing, which opens a closed popup list.
       pub fn query(self) -> Signal<String> {
           binding(move || self.query.get(), move |text: String| self.type_query(text))
       }
       fn type_query(self, text: String) {
           self.query.set(text);
           if self.inline {
               return;
           }
           self.session.send(machine::Event::Typed);
           self.write_open();
       }
       fn write_open(self) {
           if !self.open.get_untracked() {
               self.open.set(true);
           }
       }
       pub fn is_open(self) -> Value<bool> { self.open.into_value() }
       pub fn is_sheet_open(self) -> Value<bool> {
           memo(move || self.open.get() && self.session.state().with(|s| s.surface() == Some(Surface::Sheet)))
       }
       pub fn is_filtering(self) -> Value<bool> {
           memo(move || self.open.get() && self.session.state().with(machine::ComboState::filtering))
       }
       pub fn show(self) {
           if self.inline || self.open.get_untracked() {
               return;
           }
           self.session.send(machine::Event::Present(Surface::Popover));
           self.write_open();
       }
       pub fn show_sheet(self) {
           if self.inline || self.is_sheet_open_untracked() {
               return;
           }
           self.session.send(machine::Event::Present(Surface::Sheet));
           self.write_open();
       }
       fn is_sheet_open_untracked(self) -> bool {
           self.open.get_untracked() && self.session.state().with_untracked(|s| s.surface() == Some(Surface::Sheet))
       }
       pub fn hide(self) {
           if !self.inline && self.open.get_untracked() {
               self.open.set(false);
           }
       }
       pub fn toggle(self) {
           if self.inline {
               return;
           }
           if self.open.get_untracked() { self.hide() } else { self.show() }
       }
       fn shown(self) -> bool { self.open.get_untracked() }
       #[track_caller] pub fn input(self) -> Part<PartState> { self.input_for(self.input_ref) }
       #[track_caller] pub fn input_for(self, target: NodeRef) -> Part<PartState>
       #[track_caller] pub fn content(self) -> Part<PartState> { self.content_for(self.input_ref) }
       #[track_caller] pub fn content_for(self, input: NodeRef) -> Part<PartState>
       #[track_caller]
       pub fn surface<V: IntoView>(self, placement: Placement, content: impl Fn() -> V + 'static) -> View {
           kivact_core::view::show(self.is_open(), move || {
               kivact_core::view::overlay(self.input_ref, placement, content())
                   .with_node(|n| n.part(self.content()))
                   .on_dismiss(move |_| self.hide())
           })
           .into_view()
       }
       ```
     - `input_for`：旧 `input()` の本体で、`self.input_ref` をすべて `target` にする。そのうえで：
       - popup かどうかの条件は上の置換どおり `!self.inline`。
       - Escape の腕は `Key::Escape if !self.inline && !closed => self.hide()`。
       - Backspace の条件は `self.query.with_untracked(String::is_empty)`。
       - `Part::new` の連鎖の最後に `.on_attach(move |_| self.watch_focus(target))` を付け、次を足す。
         ```rust
         /// Close a popup list when `target` loses the focus, unless the list is in the sheet.
         fn watch_focus(self, target: NodeRef) {
             if self.inline {
                 return;
             }
             let focused = target.focused();
             kivact_core::state::effect_on(focused, move |focused: &bool| {
                 if !*focused && self.session.state().with_untracked(|s| s.surface() != Some(Surface::Sheet)) {
                     self.hide();
                 }
             });
         }
         ```
     - `content_for`：旧 `content()` の本体で、`.focus_on_press(self.input_ref)` を `.focus_on_press(input)` にする。
     - `choose`：Multiple の `self.session.send(machine::Event::Toggled);` を `if !self.query.with_untracked(String::is_empty) { self.query.set(String::new()); }` にする。`search` の中で query を読む `self.query()` は field の `self.query` にする。
     - 消すもの：`query_binding`、`popup`、`popup_with_focus_transfer`、`opened_by`、`with_input_ref`。`pub(crate) fn input_ref` は palette が使うので残す。
     - `combobox.rs` と `lib.rs`：`pub use handle::{ComboboxApi, ComboboxOptions, use_combobox};`、`pub use combobox::{ComboboxApi, ComboboxOptions, use_combobox};`。
     - doc：
       - module doc の「By default the list is always there and Escape belongs to the surrounding popup or dialog's dismissal; [`popup`](ComboboxApi::popup) makes the list a popup this behavior opens and closes, and」を「The list is a popup this behavior opens and closes in the caller's `open`; with `ComboboxOptions { inline: true }` it is always there and Escape belongs to the surrounding popup or dialog's dismissal; and」にする。「The query is a `Signal<String>` the caller gives to」で始まる文は「The query is the caller's `Signal<String>`; [`query`](ComboboxApi::query) gives it to」にする。
       - 型の doc の「With [`popup`](Self::popup) the list is a popup the combobox opens and closes in the caller's `Signal<bool>`:」を「Unless it is inline, the list is a popup the combobox opens and closes in the caller's `open`:」にする。
       - doc の例：`let colors = use_combobox(options.clone(), chosen, signal(String::new()), signal(true), ComboboxOptions { inline: true });`、`use_text_edit(colors.query())`。
  7. **palette（`palette/handle.rs`）**
     - `use_command_palette(commands, open: Signal<bool>)` の中：`let query = signal(String::new());` を置き、`use_combobox(options, Selection::Single(chosen))` を `use_combobox(options, Selection::Single(chosen), query, open, ComboboxOptions { inline: true })` にする（`.filter(..)` はそのまま）。コメント `// The caller's \`open\` opens and closes the inline combobox; ...` の 2 行と `combobox.opened_by(open);` を消す。
     - 足すもの：
       ```rust
       /// Show the palette unless it is shown.
       pub fn show(self) { if !self.open.get_untracked() { self.open.set(true) } }
       /// Hide the palette unless it is hidden.
       pub fn hide(self) { if self.open.get_untracked() { self.open.set(false) } }
       /// Show a hidden palette, hide a shown one.
       pub fn toggle(self) { self.open.set(!self.open.get_untracked()) }
       /// A button that opens the palette.
       #[track_caller] pub fn trigger(self) -> Part<PopupState> { self.dialog.trigger() }
       #[track_caller]
       pub fn surface<V: IntoView>(self, placement: Placement, content: impl Fn() -> V + 'static) -> View
           // show(self.is_open(), move || overlay(self.dialog.anchor_ref(), placement, content()).with_node(|n| n.part(self.content())))
       ```
     - `pub fn query(self) -> Signal<String>` は W5 のまま。
     - `while_open` を消す。module doc と型の doc の `while_open`（`[`CommandPaletteApi::while_open`]` など）は `surface` にし、「like [`MenuApi::while_open`](crate::MenuApi::while_open), is how its content mounts; the other overlays mount theirs with `show(x.is_open(), ..)`」の文を消す。doc の例の `p.while_open(move || {` は `p.surface(Placement::Center, move || {` にする。
  8. **ui `combobox/render.rs`**
     - `configured_list` の `let local_open = signal(false);` から `if !inline { list = list.popup_with_focus_transfer(..); }` までを次にする（`data.rs`・`handle.rs` の `query: Option<Signal<String>>`・`open: Option<Signal<bool>>` は W5 のまま）。
       ```rust
       let inline = matches!(mode.unwrap_or(ComboboxMode::Popup), ComboboxMode::Inline);
       let query = controlled.unwrap_or_else(|| signal(String::new()));
       let open = if inline { signal(true) } else { controlled_open.unwrap_or_else(|| signal(false)) };
       let mut list = use_combobox(options, selection, query, open, ComboboxOptions { inline }).debounce(debounce);
       ```
     - `suggestions(..)` に最後の引数 `input: Option<NodeRef>` を足し、`list.content()` を `match input { Some(input) => list.content_for(input), None => list.content() }` にする。`popup_surface` は `None` を、`sheet_surface` は `Some(sheet_input)` を渡す。
     - `sheet_surface`：`let sheet_list = list.with_input_ref(sheet_input);` を消し、`sheet_list.input()` を `list.input_for(sheet_input)` に、`suggestions(sheet_list, ..)` を `suggestions(list, .., Some(sheet_input))` にする。
  9. **`lib.rs`（interaction）の module doc**：`//! \`control()\`. The open query is` で始まる行から、`//! origin, and a submenu mounts its level in a new owner while it is open.` で終わる行までを、次にする（行番号は使わない。W5 が文中の `Binding` を `Signal` に直していても、この範囲を丸ごと置き換える）。
     ```
     //! `control()`.
     //!
     //! Every floating or collapsible surface (disclosure, popup, dialog, menu,
     //! select, combobox, command palette, hover card, navigation panel) takes the
     //! caller's `open: Signal<bool>` as a required argument, and that signal is
     //! its only open state. `is_open()` reads it; `show()`, `hide()` and
     //! `toggle()` write it on a Copy `self`: `show` and `hide` change nothing when
     //! the surface already is so, and `toggle` inverts. A kind of surface is a
     //! suffix (`show_sheet()`). A toaster's `dismiss(id)` removes one toast from
     //! its queue; it is not a show/hide command. The parts are `trigger()` and
     //! `content()`. A surface's content mounts with `show(api.is_open(), ..)`
     //! around an `overlay(..)` carrying `content()`, or in one call with
     //! `api.surface(placement, content)`, which anchors the overlay at the
     //! trigger part and mounts the content in a fresh owner on every opening (a
     //! menu's top level again when its origin changes between the pointer and the
     //! anchor). A popup adds the overlay's focus and dismissal rules to a
     //! disclosure; a dialog, a menu, a select and a navigation panel add theirs to
     //! a popup. A delayed shown state is a [`DelayedApi`]. Other state a caller
     //! may own is a builder of that name taking a `Signal` (`sort(signal)`,
     //! `widths(signal)`), read back with `<name>_value()`.
     ```
- **消すもの**：`use_menu_with_open`、`MenuApi::while_open`、`CommandPaletteApi::while_open`、`SelectApi::open_binding`、`ComboboxApi::{popup, popup_with_focus_transfer, query_binding, with_input_ref, opened_by}`、`PresentationMode`、private の `ComboboxOptions<K>`（`SearchOptions<K>` に改名）、machine の `Event::{Open, OpenSheet, Close, OpenRequested, FocusLeft, Query, Toggled}`、`Effect::{WriteQuery, WriteOpen}`、`Machine::new` の `inline`。
- **状態の閉じ方**：
  - 名前の接尾辞で対象外：`ComboboxApi`、`SelectApi`、`MenuApi`、`CommandPaletteApi`（Api）、`ComboboxOptions`、`SearchOptions`、ui の `SelectOptions`（Options）。
  - `ComboState<K>`：enum の variant `Open` は bool と Option を 1 つずつ持つが、`&mut self` の method も内部可変の field も無く、machine が状態を丸ごと置き換えるので、6.1 節の検査の対象ではない。印は付けない。
  - menu の `Session`：struct 単位の `xtask-allow(state)` の印（annex T の W1 で付いたもの）はそのまま。新しい field `item_anchors` を書くのは submenu の item の mount と cleanup だけである。
  - ui の `Menu` は値渡しの builder なので対象外。
- **試験**（期待値は全部そのまま）：
  - `interaction_combobox.rs`：
    - `picker`：`Selection::Multiple(values)`・`Selection::Single(value)`（W5 で `into_binding` は消えている）、`use_combobox(labels.clone(), selection, signal(String::new()), open, ComboboxOptions::default())`。
    - `a_shared_query_survives_setup_and_a_hidden_hide`：`use_combobox(options(), Selection::Single(value), query, signal(false), ComboboxOptions::default())`。
    - `writing_the_query_while_closed_keeps_the_popup_closed`：`use_combobox(options(), Selection::Single(value), query, open, ComboboxOptions::default())`。
    - 新設 `a_second_input_part_drives_the_same_list`：二つの field に `input()` と `input_for(second)` を付ける（second は `node_ref()`、その field に `.node_ref(second)` と `use_text_edit(combo.query()).control()`）。second に「gr」を打つと開き、Enter で `Some("green")` になって閉じる。再び second に「gr」を打って開き、second から button「Elsewhere」へ focus を移すと閉じる。
  - `apg_combobox.rs`：`use_combobox(options, signal(None::<&str>), signal(String::new()), signal(false), ComboboxOptions::default())`。
  - `interaction_show_hide.rs`：
    - `select_show_twice_stays_open_and_toggle_twice_returns`：`use_select(colors(), selected, signal(false))`。
    - `combobox_show_keeps_a_shown_list_and_its_filter`：`use_combobox(colors(), Selection::Single(value), signal(String::new()), signal(false), ComboboxOptions::default())`。
    - `menu_show_twice_stays_open`：`use_menu(vec![MenuEntry::item("cut", "Cut")], signal(false))` とし、`menu.while_open(move || column(..).part(menu.content()).name("Edit menu"))` を `menu.surface(Placement::Below, move || column(node().part(menu.item("cut")).child(text("Cut"))))` にする。button「Toggle twice」（`menu.toggle()` を 2 回）を足し、「Show twice」の後に押すと `open` が true のまま（2 回で戻る）ことを確かめる。
  - `interaction_menu.rs`：
    - `use_menu(X)` を `use_menu(X, signal(false))` にする（3 か所）。
    - `level` の `sub.while_open(move || ..)` → `show(sub.is_open(), move || ..)`。
    - `menu_button` の `m.while_open(..)` → `show(m.is_open(), ..)`。
    - `context_area` の `m.while_open(move || {..})` → `show(m.is_open(), move || dynamic(move || matches!(origin.get(), Some(MenuOrigin::Pointer(_))), move |_| {..同じ本体..}))`（`origin` は `m.origin()`）。
  - `apg_menubar.rs`、`apg_menu.rs`、`apg_menu_button.rs`：`use_menu(X)` → `use_menu(X, signal(false))`、`X.while_open(` → `show(X.is_open(), `。
  - `interaction_palette.rs`：`p.while_open(move || {` → `show(p.is_open(), move || {`。
  - `interaction_select.rs`（2 か所）、`apg_listbox.rs`：`use_select(o, s)` → `use_select(o, s, signal(false))`。
  - `closing_resets_the_query_so_a_reopened_palette_lists_its_commands_again`（`interaction_palette.rs`）と `free_text_is_kept_when_the_list_closes`（`ui_combobox.rs`）が通ることを確かめる。
- **確認（tier a）**：U30a と同じ fmt・check・clippy・xtask・`KIVACT_TEST_MIRROR=off cargo test -p kivact-interaction`。続けて：
  ```
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_combobox interaction_show_hide interaction_menu interaction_palette interaction_select apg_combobox apg_listbox apg_menu apg_menu_button apg_menubar ui_combobox ui_command_palette ui_components ui_collections
  ```
- **完了条件**：
  - `rg -nw "use_menu_with_open|while_open|open_binding|query_binding|popup_with_focus_transfer|with_input_ref|opened_by|follow_open|PresentationMode" crates/kivact-ui dev/tests/kivact-public-api examples/kivact templates dev/bench` の結果が空。
  - `rg -nw "use_menu_with_open|while_open|query_binding|popup_with_focus_transfer|with_input_ref|opened_by|follow_open|PresentationMode" crates/kivact-interaction` の結果が空。
  - `rg -n "pub fn open_binding" crates/kivact-interaction/src` の結果が空（`PopupApi` の `pub(crate) fn open_binding` と crate の中の呼出しは残る）。
  - 名前を挙げた試験が通る。

## U31：part の slot・`*_state()` getter・項目・行・cell の状態の map を消す（UI-01）

- **wave / lane / 前提**：W7b / U1 / U30b。U30a の前提（W5・W6 の API、W7a の全 unit、U11〜U14 の配置、W1 で `xtask-allow(state-fields)` の印が無いこと、e2e の binary）はこの unit にも当てはまる。2.2 節の表により、interaction の `scalar/`・`range_scalar.rs`・`splitter/` と試験 `interaction_slider.rs` は W7b ではこの unit が持つ（getter と slot の削除だけ。UI-11 の scalar の disabled は W8 の U42）。U10 で作った tooltip の `attached: Signal<Option<Attached>>` は trigger の mount の寿命の slot で、この unit の削除の対象ではない（残す）。
- **所有**：
  - `crates/kivact-interaction/src/` の次の file：`part_slot.rs`（削除）、`lib.rs`、`calendar/{handle,state}.rs`、`combobox/handle.rs`、`dialog.rs`、`disclosure.rs`、`drag/handle.rs`、`list/{handle,parts,state}.rs`、`menu/{handle,level}.rs`、`menubar.rs`、`palette/handle.rs`、`popup/api.rs`、`range_scalar.rs`、`scalar/{handle,edit_state}.rs`、`select/handle.rs`、`space_pan/handle.rs`、`splitter/handle.rs`、`table/{handle,cells,state}.rs`、`tabs.rs`、`text_edit/handle.rs`、`toggle.rs`、`tooltip/api.rs`、`tree.rs`、`zoom_pan/handle.rs`
  - ui：`crates/kivact-ui/src/text_input.rs`（`control_state` の 1 か所だけ。2.2 節）
  - 試験：`dev/tests/kivact-public-api/tests/e2e_ui/{interaction_toggle.rs, interaction_slider.rs}`
- **目的**：part の状態を読む方法を `Part::state()` の一つにする（UI-01、UI-24 (e)、PERF-11）。
- **手順**：
  1. 一覧のコマンド：`rg -n "pub fn \w+_state\(self" crates/kivact-interaction/src`。U30b の後も 46 件ある（監査の 34 件より多い）。全部消す。
     | file | getter |
     |---|---|
     | calendar/handle.rs | `grid_state`、`cell_state` |
     | combobox/handle.rs | `input_state`、`content_state`、`item_state` |
     | dialog.rs | `trigger_state`、`content_state` |
     | disclosure.rs | `trigger_state`、`content_state` |
     | drag/handle.rs | `source_state` |
     | list/handle.rs | `list_state`、`item_state` |
     | menu/handle.rs | `trigger_state`、`content_state`、`item_state` |
     | menubar.rs | `bar_state`、`item_state` |
     | palette/handle.rs | `content_state`、`input_state`、`list_state`、`item_state` |
     | popup/api.rs | `trigger_state`、`content_state` |
     | range_scalar.rs | `thumb_state` |
     | scalar/handle.rs | `control_state` |
     | select/handle.rs | `trigger_state`、`content_state`、`item_state` |
     | space_pan/handle.rs | `control_state` |
     | splitter/handle.rs | `control_state` |
     | table/handle.rs | `grid_state`、`header_state`、`row_state`、`cell_state` |
     | tabs.rs | `list_state`、`item_state` |
     | text_edit/handle.rs | `control_state` |
     | toggle.rs | `ButtonApi::control_state`、`CheckboxApi::control_state`、`SwitchApi::control_state`、`ToggleButtonApi::control_state`（と private の `Core::control_state`） |
     | tooltip/api.rs | `trigger_state`、`content_state` |
     | tree.rs | `list_state`、`item_state` |
     | zoom_pan/handle.rs | `viewport_state` |
  2. slot の field と、その初期化と、`crate::part_slot::attach_part_state(..)` の行を消す。
     - Option の slot（`Signal<Option<Value<..>>>`）：
       - disclosure（`trigger_state`、`content_state`）
       - popup（`content_state`、`trigger_state`）
       - dialog（`content_state`）
       - tooltip（`trigger_state`、`content_state`。`attached` は残す）
       - select（`content_state`）
       - combobox（`input_state`、`content_state`）
       - menu/level.rs（`content_state`）
       - calendar/state.rs（`grid_state`）
       - list/state.rs（`container_state`）
       - table/state.rs（`grid_state`）
       - drag/handle.rs（`observed`）
       - scalar/edit_state.rs（`observed`）
       - space_pan、splitter、zoom_pan、toggle（`ButtonApi.observed`、`Core.observed`）の `observed`
       - text_edit/handle.rs（data の `observed`）
     - map：
       - combobox の `item_states`（U03 の後の `item` の中の `item_states.update(..)` と `on_cleanup(..)` の塊を含む）
       - menu/level.rs の `item_states` と `fn observe_item`、その呼出し（U03 の後は `self.observe_item(key.get(), part_state);`）
       - list/state.rs の `item_states`、list/parts.rs の `observe_item(&state, key, part_state);` の呼出しと `fn observe_item`
       - calendar/state.rs の `cell_states`、calendar/handle.rs の `let observed = state.cell_states;` の塊
       - table/state.rs の `header_states`・`row_states`・`cell_states`、table/handle.rs の `let observed = state.header_states;` の塊、table/cells.rs の `let observed = state.row_states;` と `let observed = state.cell_states;` の塊
     - 確かめるコマンド：`rg -n "part_slot|attach_part_state|item_states|row_states|cell_states|header_states|container_state|\bobserved\b" crates/kivact-interaction/src`。roving・toaster の `observed` は別物なので残す。
  3. `crates/kivact-interaction/src/part_slot.rs` を消し、`lib.rs` の `mod part_slot;` を消す。
  4. 消した getter や field の名前に触れる doc の文を消す（`rg -n "_state\]|_state\(\)" crates/kivact-interaction/src` が空になるまで）。
  5. ui `text_input.rs`：
     - `create(..)` の `let part = editing.control();` の直後に `let input = part.state();` を足す。
     - `TextInput` に field `input: Value<PartState>` を足し、`create` の literal で入れる。
     - `into_view` の `let input = self.editing.control_state();` を `let input = self.input;` にする。
- **消すもの**：`part_slot.rs`、上の 46 個の getter、slot の field と map の field、`Level::observe_item`、list の `observe_item`。
- **状態の閉じ方**：新しい型は無い。field を消すだけなので、6.1 節の対象の数は減る。`TextInput` は値渡しの builder。
- **試験**：
  - `interaction_toggle.rs`：
    - `a_parent_reads_the_disclosure_content_state_across_unmount_and_remount`：`let trigger = details.trigger(); let early = trigger.state();` に変え、`control("Details").part(trigger)` にする。content は `show(details.is_open(), move || { let part = details.content(); let state = part.state(); control("Body").part(part).child(text(move || open_text("content", state.get().open))) })` にする。`late` の分岐は消す。期待値：
      - `early` は true → false → true → false。
      - 開いている間は `content open true` がある。
      - 閉じている間は `query_by_text(&open_text("content", true)).is_none()`。
    - `a_parent_reads_the_disclosure_trigger_state_after_the_trigger_unmounts`：state を `show(has_trigger, ..)` の中で `let part = details.trigger(); let state = part.state();` から読み、その branch の中に text を置く。期待値：
      - click 後に `trigger open true`。
      - `has_trigger` を false にし、`open` を false にした後、再 mount すると `trigger open false`。
      - click で `trigger open true`。
    - `a_parent_reads_the_popup_content_state_across_closing_and_reopening`：state を `show` の中の `popup.content()` の part から読み、text も branch の中に置く。期待値：開くと `surface open true`、閉じると text が無く、再び開くと `surface open true`。
    - この 3 件で消えるのは、mount していない part の代わりの値（fallback）の期待値だけである。それは UI-01 で消す機能そのもので、緩めではない。名前を次に改める：`the_disclosure_parts_read_their_own_state_across_remounts`、`a_remounted_disclosure_trigger_part_starts_from_the_requested_state`、`the_popup_content_part_reads_its_state_while_mounted`。
  - `interaction_slider.rs` の `a_logarithmic_track_maps_the_pointer_through_the_mapping`：`let state = scalar.control_state();` を `let thumb_part = scalar.thumb(); let state = thumb_part.state();` にし、`thumb("Frequency", scalar.thumb(), ..)` を `thumb("Frequency", thumb_part, ..)` にする。期待値は同じ。
  - 他に呼出しが無いこと：`rg -n "\.(grid|cell|input|content|item|trigger|source|list|bar|thumb|control|header|row|viewport)_state\(" dev/tests/kivact-public-api examples/kivact templates dev/bench crates/kivact-ui/src` の結果が空。
- **確認（tier a）**：U30a と同じ fmt・check・clippy・xtask・`KIVACT_TEST_MIRROR=off cargo test -p kivact-interaction`。続けて：
  ```
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_toggle interaction_slider interaction_calendar interaction_drag_edit interaction_list_nav interaction_menu interaction_splitter interaction_table interaction_tabs interaction_tree interaction_zoom_pan interaction_edit interaction_combobox interaction_select interaction_palette ui_controls ui_field ui_data_table apg_tree_view apg_tabs apg_slider
  ```
- **完了条件**：
  - `rg -n "pub fn \w+_state\(self|part_slot|attach_part_state" crates/kivact-interaction crates/kivact-ui` の結果が空。
  - `rg -n "item_states|row_states|cell_states|header_states" crates/kivact-interaction` の結果が空。
  - 名前を挙げた試験が通る。

## U33：`use_sheet_tap`（select・combobox の touch sheet の tap、UI-04）

- **wave / lane / 前提**：W7b / U1 / U31。U30a の前提（W5・W6 の API、W7a の全 unit、U11〜U14 の配置、e2e の binary）はこの unit にも当てはまる。U12 の後、ui の select は `select.rs`（旧 `select/handle.rs` の中身。`select_trigger` もここにある）と `select/machine.rs`（`pub(crate) mod machine;`）である。U22 の後、`select`・`combobox` は名前を `.label("..")` で受ける。U23 で `selection_field.rs` は消えている。U30b で `configure_select` は `(behavior, open)` を返す。
- **所有**：`crates/kivact-interaction/src/{sheet_tap.rs（新設）, lib.rs}`、`crates/kivact-ui/src/select.rs`、`crates/kivact-ui/src/select/machine.rs`（削除）、`crates/kivact-ui/src/combobox/render.rs`、`dev/tests/kivact-public-api/tests/e2e_ui/ui_combobox.rs`。`combobox/notices.rs` は W7b では U32 が持つので変えない。
- **目的**：ui の `impl Machine` である sheet の tap の machine を、interaction の `use_sheet_tap` に移す（UI-04 規則 1）。
- **手順**：
  1. `crates/kivact-interaction/src/sheet_tap.rs` を新設する。file の並びは次のとおり。
     - 1 行目：`//! Role: Recognise a touch tap that should open a selection surface as a sheet.`
     - import：`crate::DragAxis`、`kivact_core::custom::{Part, PartState, part}`、`kivact_core::geometry::{Point, Rect}`、`kivact_core::state::{IntoValue, Machine as MachineTrait, MachineHandle, Step}`、`kivact_core::view::{Handled, NodeRef, PointerEvent, PointerId, PointerKind, PointerPhase, node_ref}`。
     - `crates/kivact-ui/src/select/machine.rs` の `Outcome`、`SheetGesture`、`Event`、`Readiness`、`Machine`（と `impl MachineTrait for Machine`）、`fn gesture` をそのまま移す。`pub(crate)` と field の `pub` を外して private にし、`use kivact_interaction::DragAxis;` は上の `crate::DragAxis` にする。`Effect` は `enum Effect {}`。
     - その後に次を置く。
       ```rust
       /// Copy handle for a touch tap on a selection trigger or field.
       #[derive(Clone, Copy)]
       pub struct SheetTapApi {
           gesture: MachineHandle<Machine>,
           target: NodeRef,
       }
       /// A tap is recognised only while `available`; the slop is the theme's
       /// `InteractionPolicy::select.sheet_slop`. Setup only.
       #[track_caller]
       pub fn use_sheet_tap(available: impl IntoValue<bool>) -> SheetTapApi {
           let available = available.into_value();
           let target = node_ref();
           let bounds = target.rect();
           let theme = kivact_core::style::use_theme();
           let gesture = kivact_core::state::machine(
               SheetGesture::Idle,
               move || Machine {
                   readiness: if available.get() { Readiness::Ready } else { Readiness::Unavailable },
                   bounds: bounds.get(),
                   slop: theme.get().ext::<crate::InteractionPolicy>().select.sheet_slop,
               },
               |_, effect| match effect {},
           );
           SheetTapApi { gesture, target }
       }
       impl SheetTapApi {
           /// The touched node: a tap must end inside it, and a key pressed on it forgets a tap.
           #[track_caller]
           pub fn target(self) -> Part<PartState> {
               let input = part(|input| input);
               Part::new(input.state(), move |host| {
                   host.part(input).node_ref(self.target).on_key(move |_| {
                       self.gesture.send(Event::Reset);
                       Handled::No
                   })
               })
           }
           /// Feed one pointer event of the target, from the caller's own pointer handler.
           pub fn pointer(self, event: PointerEvent) {
               self.gesture.send(Event::Pointer(event));
           }
           /// Whether a touch tap completed inside the target; reading consumes it.
           pub fn take_tap(self) -> bool {
               let tap = matches!(self.gesture.state().get_untracked(), SheetGesture::Released(Outcome::Tap));
               self.gesture.send(Event::Reset);
               tap
           }
       }
       ```
     - `lib.rs`：`mod sheet_tap;` と `pub use sheet_tap::{SheetTapApi, use_sheet_tap};` を足す。
  2. ui `select.rs` の `select_trigger`：
     - 引数の `anchor: NodeRef` を消し、`into_view` の呼出し `select_trigger(behavior, &mut self.root, anchor, ..)` から `anchor,` を消す（`into_view` の `let anchor = ..;` は trigger の `.node_ref(anchor)` と popup・sheet の surface が使うので残す）。
     - `let bounds = anchor.rect();` と `let gesture = kivact_core::state::machine(..);` の塊を、`let tap = kivact_interaction::use_sheet_tap(move || !shown.get() && !disabled.as_ref().is_some_and(|d| d.get()));` にする（`let disabled = root.accessible_disabled();` は残す）。
     - `behavior.trigger_with_sheet(..)` の chooser を `move || tap.take_tap() && eligible()` にする。
     - `Part::new(trigger_state, ..)` の中を次にする（旧 `.on_key(..Reset..)` は `target()` が受け持つ）。
       ```rust
       host.part(trigger)
           .part(tap.target())
           .on_pointer(move |event| {
               tap.pointer(event);
               Handled::No
           })
           .has_popup(move || Some(if sheet_open.get() { PopupKind::Dialog } else { PopupKind::ListBox }))
       ```
  3. ui `combobox/render.rs`：
     - `editor_view` の中で、`let shown = list.is_open();` の後に `let tap = kivact_interaction::use_sheet_tap(move || !disabled.get() && !shown.get());` を作る。editor の `.with_node(move |node| node.part(list.input()))` の後に `.with_node(move |node| node.part(tap.target()))` を足し、`.on_pointer(editor_pointer(list, disabled, input_ref, presentation))` を `.on_pointer(editor_pointer(list, disabled, tap, presentation))` にする。
     - `editor_pointer`：引数の `input_ref: NodeRef` を `tap: kivact_interaction::SheetTapApi` にし、`let bounds = ..;` と `let gesture = kivact_core::state::machine(..);` を消す。返す closure を次にする（doc を `/// Open the list from a pointer press, or the sheet from a touch tap.` にする）。
       ```rust
       move |event| {
           tap.pointer(event);
           if !disabled.get_untracked() {
               if presentation.uses_sheet() && tap.take_tap() {
                   list.show_sheet();
               } else if matches!(event.phase(), PointerPhase::Down(_))
                   && ((event.kind() != PointerKind::Touch && !shown.get_untracked())
                       || (event.kind() == PointerKind::Touch && !presentation.uses_sheet()))
               {
                   list.show();
               }
           }
           Handled::No
       }
       ```
  4. `crates/kivact-ui/src/select/machine.rs` を消し、`select.rs` の `pub(crate) mod machine;` を消す。`select.rs` の `machine::` の残りが無いことを `rg -n "machine::" crates/kivact-ui/src/select.rs` で確かめる。
- **消すもの**：`crates/kivact-ui/src/select/machine.rs`（ui の `SheetGesture`、`Outcome`、`Readiness`、`Machine`、`gesture`）、`select_trigger` の引数 `anchor`、`editor_pointer` の引数 `input_ref`。
- **状態の閉じ方**：`SheetTapApi` は Api の接尾辞で対象外。`SheetGesture` は enum。`Machine` は machine の入力の文脈の値で、遷移のたびに作り直す（状態ではない。数える field は `bounds` の Option 一つ）。
- **試験**：`ui_combobox.rs` の `select_touch_opens_the_sheet` はそのまま通す。新設 `combobox_touch_tap_opens_the_sheet`：
  - 構成：`let chosen = signal(None::<&str>);`、`combobox(vec![ListItem::new("a", "Alpha"), ListItem::new("b", "Beta")], chosen).label("Team").presentation(SelectPresentation::BottomSheet).style(style().width(280))`。
  - 確かめること：
    - `get_by_role(Role::ComboBox, "Team")` の中央で `contact_down(PointerKind::Touch, ..)` をした時点で `Role::Dialog` "Team" が無い。
    - `contact_up` のあと `advance(Duration::from_millis(350))` で "Team" の dialog がある。
    - "Beta" を click すると `chosen.get_untracked() == Some("b")`。
    - `errors()` が空。
- **確認（tier a）**：U30a と同じ fmt・check・clippy・xtask・`KIVACT_TEST_MIRROR=off cargo test -p kivact-interaction`。続けて：
  ```
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_combobox interaction_select interaction_combobox ui_controls
  ```
- **完了条件**：
  - `rg -n "SheetGesture|sheet_slop" crates/kivact-ui/src` の結果が空。
  - `rg -n "impl .*Machine for" crates/kivact-ui/src/select.rs crates/kivact-ui/src/select crates/kivact-ui/src/combobox` の結果が空（ui の他の `impl Machine` は同じ wave の U32・U34・U35 と W8 の U43 が消す）。
  - `rg -n "use_sheet_tap" crates/kivact-ui/src` が `select.rs` と `combobox/render.rs` で当たる。
  - 名前を挙げた試験が通る。

## U32：UI-04 gesture（switch drag・swipe dismiss）、Announcer の outlet、Toast の icon
- **wave / lane / 前提**：W7b / U2 / W7a の全 unit（U20 → U21 → U25 → U22 → U23 → U24 の順）が完了。この unit は次の状態から始める。
  - W5・W6 の API：`Signal<T>`（`Binding`・`IntoBinding` は無い）、`binding(get, set) -> Signal<T>`、`Callback<E>`／`callback(f)`（`f: impl Fn(E) -> R`。`Callback<()>` なら `callback(move |()| ..)`）、`IntoHandler<(), M>`（`kivact_core::state::IntoHandler`）、`Callback::from_handler`、`use_direction() -> ReadingDirection`（Value で包まない）、`Theme::light()`、`theme.with(ext)`、`Trigger`。`Signal::update`・`Var::update` は `()` を返す（DEC-15）。この unit のどの手順も `update` の戻り値を使わない。
  - handler の形（DEC-16）：payload のある handler（`on_edit`、`on_pointer`、`on_key` など）は `impl Fn(E) -> R + 'static`（`R: HandlerResult`）を受ける。payload の無い handler だけが `impl IntoHandler<(), M>` を受ける。共有した `Callback<E>` は `move |e| cb.call(e)` で渡す。
  - W7a の結果：U22 の `.label()` の呼出しの形、U23 の `on_commit`・`use_toaster()`・`use_announcer()`（`toaster()`・`announcer()` は U23 で改名済み。試験と ui の呼出し元も U23 が直している）、U23 の ui の `DialogKind`（ui の `DialogMode` の呼出しはすべて `DialogKind` になっている）、U20 の `<Name>Theme` と `crates/kivact-ui/src/theme.rs`（recipe の状態の型と `ThumbDrag`・`TrackInput`・`Lift` はここから再公開されている）、U21 の theme 変数、U24 の `Element::variant`、U25 の `#[caller] caller: Caller`（`Node::authored` の snapshot）・`kivact::view::Children`・`Rc<dyn Fn(..)>` の props。
  - W4 の移動（U11〜U14）の後の配置：`crates/kivact-ui/src/toggle.rs` と `toggle/{look.rs, parts.rs, switch_drag.rs, switch_drag/}`（`switch_drag/` は `geometry.rs`・`handle.rs`・`machine.rs`。`mod switch_drag;` は `toggle.rs` にある）、`toast.rs` と `toast/{look.rs, swipe.rs, swipe/}`（`swipe/` は `handle.rs`・`machine.rs`。`mod swipe;` は `toast.rs` にある）、`dialog.rs` と `dialog/{look.rs, drawer_swipe.rs}`、`empty_state.rs` と `empty_state/changes.rs`。module の root は `<name>.rs`（clippy `mod_module_files` は deny。`mod.rs` は作らない）。
  - W1 で `xtask-allow(state-fields)` の印はすべて消えている。e2e の binary は annex T の割当てどおり：`view.rs` は `--test e2e_core`、`interaction_*`・`ui_*` は `--test e2e_ui`。
  - 同じ W7b の他の lane との境界（2.2 節）：`dialog.rs`・`dialog/look.rs` は U30a が持つ。この unit は `dialog/drawer_swipe.rs` だけを持ち、`dialog.rs` が呼ぶ関数の署名を変えない。`combobox/notices.rs` はこの unit だけが持つ（U30b・U31・U33 は触らない）。
- **所有**：
  - 新規：`crates/kivact-interaction/src/switch_drag.rs`、`crates/kivact-interaction/src/switch_drag/machine.rs`、`crates/kivact-interaction/src/swipe_dismiss.rs`、`crates/kivact-interaction/src/swipe_dismiss/machine.rs`、`crates/kivact-ui/src/announcer_outlet.rs`、`crates/kivact-ui/src/toggle/switch_track.rs`
  - 変更：`crates/kivact-interaction/src/lib.rs`（`mod`・`pub use` の行だけ）、`crates/kivact-interaction/src/announcer.rs`、`crates/kivact-interaction/src/toaster/data.rs`、`crates/kivact-ui/src/lib.rs`（`mod`・`pub use` の行だけ）、`crates/kivact-ui/src/theme.rs`（`ThumbDrag`・`TrackInput`・`Lift` を再公開する `pub use` の行だけ）、`crates/kivact-ui/src/toggle.rs`（switch の gesture の部分だけ）、`crates/kivact-ui/src/toggle/look.rs`（`ThumbDrag`・`TrackInput`・`Lift` の定義の削除と import だけ）、`crates/kivact-ui/src/toast.rs`、`crates/kivact-ui/src/toast/swipe.rs`（中身を置き換える）、`crates/kivact-ui/src/dialog/drawer_swipe.rs`、`crates/kivact-ui/src/empty_state.rs`・`crates/kivact-ui/src/empty_state/changes.rs`（outlet の呼出しの行だけ）、`crates/kivact-ui/src/combobox/notices.rs`（`Notices::outlet` の本体と import だけ）
  - 削除：`crates/kivact-ui/src/toggle/switch_drag.rs`、`crates/kivact-ui/src/toggle/switch_drag/`（directory ごと）、`crates/kivact-ui/src/toast/swipe/`（directory ごと）
  - 試験：`dev/tests/kivact-public-api/tests/e2e_core/view.rs`、`dev/tests/kivact-public-api/tests/e2e_ui/interaction_toast.rs`、`dev/tests/kivact-public-api/tests/e2e_ui/ui_controls.rs`（`interaction_toggle.rs` は同じ W7b の lane U1 の U30a・U31 が変えるので、この unit は触らない）
- **目的**：UI-04 規則 1（入力で動く switch の drag と toast・drawer の swipe を interaction の `use_switch_drag`・`use_swipe_dismiss` に移す。toast と drawer は一つの gesture と policy）、規則 2（`Announcer::outlet` → ui の `announcer_outlet()`、`Toast::icon: ViewFn` → `Toast::icon_name: Option<&'static str>`）。
- **手順**：
  1. `crates/kivact-interaction/src/switch_drag/machine.rs` を作る：`crates/kivact-ui/src/toggle/switch_drag/machine.rs` の全文を移し、次だけを変える。
     - `use super::geometry::Geometry;` → `use super::SwitchTrack;`。`Lift`・`ThumbDrag`・`TrackInput` を import する行（U11 後は `use crate::toggle::look::{Lift, ThumbDrag, TrackInput};`）→ `use super::{Lift, ThumbDrag, TrackInput};`。`use kivact_interaction::DragAxis;` → `use crate::DragAxis;`。
     - `pub(super) struct Machine { pub value: bool, pub geometry: Geometry, pub now: u64 }` を次にする：
       ```rust
       pub(super) struct Machine {
           pub(super) value: bool,
           pub(super) track: SwitchTrack,
           /// -1 in a right-to-left subtree, else 1.
           pub(super) sign: f64,
           pub(super) now: u64,
       }
       ```
     - 本体の `self.geometry.sign` → `self.sign`、それ以外の `self.geometry.` → `self.track.`（`width`、`on_side`、`off_side`、`travel()`、`band(..)`）。置換：`perl -pi -e 's/self\.geometry\.sign/self.sign/g; s/self\.geometry\./self.track./g' crates/kivact-interaction/src/switch_drag/machine.rs`。続けて `rg -n "geometry" crates/kivact-interaction/src/switch_drag/machine.rs` が空であることを確かめる（複数行に分かれた `self\n.geometry` が残っていれば同じ規則で手で直す）。
  2. `crates/kivact-interaction/src/switch_drag.rs` を作る：
     ```rust
     //! Role: Opt-in direct manipulation of a switch thumb.
     mod machine;
     use crate::{EditSession, InteractionPolicy, ToggleState};
     use kivact_core::custom::{Part, clock_now};
     use kivact_core::state::{IntoValue, MachineHandle, Signal, Value, machine, memo};
     use kivact_core::style::use_theme;
     use kivact_core::view::{EditPhase, Handled, HandlerResult, MouseButton, PointerKind, PointerPhase, ReadingDirection, use_direction};
     use machine::{Event, State, SwitchGesture};

     /// The thumb geometry a switch drag follows, in logical pixels.
     #[derive(Clone, Copy, Debug, PartialEq)]
     pub struct SwitchTrack {
         pub width: f64,
         pub height: f64,
         /// The thumb's size while the switch is off.
         pub off_side: f64,
         /// The thumb's size while the switch is on.
         pub on_side: f64,
         /// The inset of the off thumb from the track's start.
         pub off_inset: f64,
     }
     impl SwitchTrack {
         pub(crate) fn travel(self) -> f64 {
             self.width - 2.0 - self.on_side / 2.0 - self.off_inset - self.off_side / 2.0
         }
         pub(crate) fn band(self, raw: f64, on: bool) -> f64 {
             let end = self.travel();
             let inset = if on { 2.0 } else { self.off_inset };
             let dimension = (12.0 * self.height / 28.0).min(inset / 2.0);
             let clamped = raw.clamp(0.0, end);
             let excess = raw - clamped;
             clamped
                 + excess.signum() * (1.0 - 1.0 / (excess.abs() * 0.55 / dimension + 1.0)) * dimension
         }
     }
     ```
     （`travel`・`band` の本体は `crates/kivact-ui/src/toggle/switch_drag/geometry.rs` の同名 method と同じ。）続けて `ThumbDrag`（`impl ThumbDrag` の `is_touch` を含む）、`TrackInput`、`Lift` を `crates/kivact-ui/src/toggle/look.rs` から doc と derive ごと切り取ってこの file に貼る（`rg -n "pub enum (ThumbDrag|TrackInput|Lift)" crates/kivact-ui/src` で場所を確かめる）。そして：
     ```rust
     /// A switch's drag: the thumb follows a touch or pointer along the track
     /// and the release commits one value. Inert when the theme's
     /// [`InteractionPolicy`] leaves `switch.drag` off.
     #[derive(Clone, Copy)]
     pub struct SwitchDragApi {
         session: Option<MachineHandle<machine::Machine>>,
         edit: EditSession<bool>,
         thumb: Value<Option<ThumbDrag>>,
     }

     /// Create the drag of a switch that edits `value` on `track`. Setup only.
     #[track_caller]
     pub fn use_switch_drag(value: Signal<bool>, track: impl IntoValue<SwitchTrack>) -> SwitchDragApi {
         let track = track.into_value();
         let edit = EditSession::new(move |initial| value.set(initial));
         let enabled = use_theme().get().ext::<InteractionPolicy>().switch.drag.0;
         if !enabled {
             return SwitchDragApi { session: None, edit, thumb: memo(|| None) };
         }
         let sign = if use_direction() == ReadingDirection::Rtl { -1.0 } else { 1.0 };
         let session = machine(
             State::Gesture(SwitchGesture::Idle),
             move || machine::Machine { value: value.get_untracked(), track: track.get(), sign, now: clock_now() },
             move |_, effect| match effect {
                 machine::Effect::Set(next) => edit.once(value.get_untracked(), || value.set(next)),
                 machine::Effect::Suppress(event) => event.suppress_activation(),
             },
         );
         let thumb = memo(move || session.get().visual());
         SwitchDragApi { session: Some(session), edit, thumb }
     }

     impl SwitchDragApi {
         /// Whether the theme turned the drag on.
         pub fn is_enabled(self) -> bool { self.session.is_some() }
         /// Where the thumb is during a drag; `None` while no drag runs.
         pub fn thumb(self) -> Value<Option<ThumbDrag>> { self.thumb }
         /// Report each drag's `Begin` and `Commit`. Setup only.
         pub fn on_edit<R: HandlerResult>(self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
             self.edit.on_edit(f);
             self
         }
         /// Add the drag to a switch's control part (`SwitchApi::control()`);
         /// returns `control` unchanged when the drag is off. Setup only.
         #[track_caller]
         pub fn control(self, control: Part<ToggleState>) -> Part<ToggleState> {
             let Some(session) = self.session else { return control };
             let state = control.state();
             session.feed(move || state.get().input.is_disabled(), Event::Disabled);
             Part::new(state, move |host| {
                 host.part(control).on_pointer(move |event| {
                     let touch = event.kind() == PointerKind::Touch;
                     if touch && !event.is_primary() {
                         return Handled::Yes;
                     }
                     let before = session.get();
                     session.send(Event::Pointer(event));
                     let after = session.get();
                     let handled = match event.phase() {
                         PointerPhase::Down(press) => {
                             touch && event.is_primary() && press.button == MouseButton::Primary
                         }
                         PointerPhase::Move => after.handles(event),
                         PointerPhase::Up(press) if press.button == MouseButton::Primary => {
                             before.handles(event)
                                 || (touch && before.tracks(event))
                                 || (before.tracks(event) && matches!(after, State::Released { .. }))
                         }
                         _ => false,
                     };
                     if handled { Handled::Yes } else { Handled::No }
                 })
             })
         }
     }
     ```
     （`on_pointer` の closure の本体は `crates/kivact-ui/src/toggle/switch_drag/handle.rs` の `fn switch` の中の closure と同じ。）`InteractionPolicy::switch.drag.0` の読みはこの 1 行だけ（U41 が W8 で `.0` を消す）。
  3. `crates/kivact-interaction/src/swipe_dismiss/machine.rs` を作る（`crates/kivact-ui/src/toast/swipe/machine.rs` の machine を edge の軸で一般化したもの。旧 file の `intent`・`qualifies`・`edge_sign` と試験は次で置き換わる）：
     ```rust
     //! Role: Decide a swipe toward an edge and whether its release dismisses.
     use super::{SwipeDismissPolicy, SwipePhase};
     use crate::DragVelocity;
     use kivact_core::geometry::{Edge, Point, Rect};
     use kivact_core::state::{Machine as MachineTrait, Step};
     use kivact_core::view::{MouseButton, PointerEvent, PointerId, PointerKind, PointerPhase};
     use std::time::Duration;
     /// How much more the motion toward the edge must be than the motion across it.
     const AXIS_RATIO: f64 = 1.2;
     /// The recent motion the release speed is measured over.
     const VELOCITY_WINDOW: Duration = Duration::from_millis(100);

     #[derive(Clone, PartialEq)]
     pub(super) struct Contact { pub(super) pointer: PointerId, origin: Point, size: f64, velocity: DragVelocity }
     #[derive(Clone, PartialEq)]
     pub(super) enum Swipe {
         Rest,
         Pressed(Contact),
         Dragging { contact: Contact, offset: f64 },
         Blocked { pointer: PointerId },
         Released { travel: f64, size: f64 },
     }
     impl Swipe {
         pub(super) fn phase(&self) -> SwipePhase {
             match self {
                 Self::Rest | Self::Blocked { .. } => SwipePhase::Rest,
                 Self::Pressed(_) => SwipePhase::Pressed,
                 Self::Dragging { contact, offset } => SwipePhase::Dragging { offset: *offset, size: contact.size },
                 Self::Released { travel, size } => SwipePhase::Dismissed { travel: *travel, size: *size },
             }
         }
     }
     pub(super) enum Event { Pointer(PointerEvent), Protect(PointerId) }
     pub(super) enum Effect { Dismiss }
     pub(super) struct Machine { pub(super) policy: SwipeDismissPolicy, pub(super) rect: Option<Rect>, pub(super) now: u64 }

     impl MachineTrait for Machine {
         type State = Swipe; type Event = Event; type Effect = Effect;
         fn transition(&self, state: &Swipe, event: Event) -> Step<Swipe, Event, Effect> {
             let event = match event {
                 Event::Protect(pointer) => return Step::to(Swipe::Blocked { pointer }),
                 Event::Pointer(event) => event,
             };
             if !self.policy.inputs.accepts(event.kind()) || matches!(state, Swipe::Released { .. }) {
                 return Step::stay();
             }
             let pos = event.window_pos();
             if !pos.x.is_finite() || !pos.y.is_finite() { return Step::to(Swipe::Rest); }
             if let PointerPhase::Down(press) = event.phase() { return self.press(state, &event, press.button, pos); }
             let (contact, promoted) = match state {
                 Swipe::Pressed(contact) => (contact.clone(), false),
                 Swipe::Dragging { contact, .. } => (contact.clone(), true),
                 _ => return Step::stay(),
             };
             if contact.pointer != event.id() { return Step::stay(); }
             self.follow(contact, promoted, &event, pos)
         }
     }
     impl Machine {
         fn press(&self, state: &Swipe, event: &PointerEvent, button: MouseButton, pos: Point) -> Step<Swipe, Event, Effect> {
             if !event.is_primary() && event.kind() == PointerKind::Touch { return Step::to(Swipe::Rest); }
             if matches!(*state, Swipe::Blocked { pointer } if pointer == event.id()) { return Step::to(Swipe::Rest); }
             if button == MouseButton::Primary && matches!(state, Swipe::Rest | Swipe::Blocked { .. }) && let Some(rect) = self.rect {
                 let mut velocity = DragVelocity::new(VELOCITY_WINDOW);
                 velocity.reset(self.now, pos);
                 return Step::to(Swipe::Pressed(Contact { pointer: event.id(), origin: pos, size: extent(rect, self.policy.edge).max(1.0), velocity }));
             }
             Step::stay()
         }
         fn follow(&self, mut contact: Contact, promoted: bool, event: &PointerEvent, pos: Point) -> Step<Swipe, Event, Effect> {
             let delta = Point::new(pos.x - contact.origin.x, pos.y - contact.origin.y);
             let travel = toward(self.policy.edge, delta).max(0.0);
             contact.velocity.record(self.now, pos);
             match event.phase() {
                 PointerPhase::Move => {
                     if !promoted {
                         match intent(delta, self.policy.edge, self.policy.slop) {
                             Intent::Pending => return Step::to(Swipe::Pressed(contact)),
                             Intent::Reject => return Step::to(Swipe::Rest),
                             Intent::Promote => {}
                         }
                     }
                     Step::to(Swipe::Dragging { contact, offset: travel })
                 }
                 PointerPhase::Up(_) => {
                     let speed = toward(self.policy.edge, contact.velocity.pixels_per_ms(self.now));
                     if promoted && self.policy.dismisses(travel, contact.size, speed) {
                         Step::to(Swipe::Released { travel, size: contact.size }).with(Effect::Dismiss)
                     } else {
                         Step::to(Swipe::Rest)
                     }
                 }
                 PointerPhase::Cancel => Step::to(Swipe::Rest),
                 _ => Step::stay(),
             }
         }
     }
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub(super) enum Intent { Pending, Reject, Promote }
     /// The motion toward the edge (along) and across it.
     fn split(edge: Edge, delta: Point) -> (f64, f64) {
         match edge { Edge::Top | Edge::Bottom => (delta.y, delta.x), Edge::Left | Edge::Right => (delta.x, delta.y) }
     }
     pub(super) fn sign(edge: Edge) -> f64 {
         match edge { Edge::Top | Edge::Left => -1.0, Edge::Bottom | Edge::Right => 1.0 }
     }
     /// How far `delta` goes toward `edge`.
     pub(super) fn toward(edge: Edge, delta: Point) -> f64 { split(edge, delta).0 * sign(edge) }
     fn extent(rect: Rect, edge: Edge) -> f64 {
         match edge { Edge::Top | Edge::Bottom => rect.height, Edge::Left | Edge::Right => rect.width }
     }
     pub(super) fn intent(delta: Point, edge: Edge, slop: f64) -> Intent {
         let (along, across) = split(edge, delta);
         if across.abs() > slop && across.abs() > along.abs() { return Intent::Reject; }
         if along.abs() <= slop || along.abs() <= across.abs() * AXIS_RATIO { return Intent::Pending; }
         if along * sign(edge) > 0.0 { Intent::Promote } else { Intent::Reject }
     }
     #[cfg(test)]
     mod tests {
         //! Role: Check swipe intent and dismissal thresholds.
         use super::{Intent, intent};
         use crate::SwipeDismissPolicy;
         use kivact_core::geometry::{Edge, Point};
         #[test]
         fn intent_and_flick_thresholds_match_the_edge() {
             assert_eq!(intent(Point::new(0.0, 8.0), Edge::Bottom, 8.0), Intent::Pending);
             assert_eq!(intent(Point::new(12.0, 10.0), Edge::Bottom, 8.0), Intent::Reject);
             assert_eq!(intent(Point::new(10.0, 12.0), Edge::Bottom, 8.0), Intent::Pending);
             assert_eq!(intent(Point::new(10.0, 13.0), Edge::Bottom, 8.0), Intent::Promote);
             assert_eq!(intent(Point::new(0.0, -12.0), Edge::Bottom, 8.0), Intent::Reject);
             assert_eq!(intent(Point::new(0.0, -12.0), Edge::Top, 8.0), Intent::Promote);
             assert_eq!(intent(Point::new(12.0, 0.0), Edge::Right, 8.0), Intent::Promote);
             let toast = SwipeDismissPolicy::toast(Edge::Bottom, 8.0);
             assert!(!toast.dismisses(47.0, 200.0, 100.0));
             assert!(!toast.dismisses(48.0, 200.0, 1.2));
             assert!(toast.dismisses(48.0, 200.0, 1.3));
             assert!(toast.dismisses(80.0, 200.0, 0.0));
         }
     }
     ```
     （`intent` は toast の旧 `intent`（`DragAxis::past_slop` は「`> slop`」）を edge の軸で書き直したもので、Bottom・Top の結果は旧試験と同じ。）
  4. `crates/kivact-interaction/src/swipe_dismiss.rs` を作る：
     ```rust
     //! Role: Dismiss a surface by swiping it toward its edge (a toast card, a drawer).
     mod machine;
     use crate::DrawerSwipe;
     use kivact_core::custom::{Part, PartState, clock_now, part};
     use kivact_core::geometry::{Edge, Rect};
     use kivact_core::state::{Callback, IntoHandler, MachineHandle, Value, machine, memo};
     use kivact_core::view::{Handled, PointerEvent, PointerKind, PointerPhase};
     use machine::{Event, Swipe};

     /// Which contacts a swipe follows.
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub enum SwipeInputs { All, TouchAndPen }
     impl SwipeInputs {
         fn accepts(self, kind: PointerKind) -> bool {
             self == Self::All || matches!(kind, PointerKind::Touch | PointerKind::Pen)
         }
     }
     /// How a swipe toward `edge` is recognised and when its release dismisses.
     #[derive(Clone, Copy, Debug, PartialEq)]
     pub struct SwipeDismissPolicy {
         /// The edge the surface leaves through.
         pub edge: Edge,
         pub inputs: SwipeInputs,
         /// Logical pixels a contact moves before it is a swipe or not.
         pub slop: f64,
         /// The share of the surface's size along the edge's axis that dismisses.
         pub close_fraction: f64,
         /// The least travel that dismisses by distance.
         pub min_distance: f64,
         /// The least travel of a flick.
         pub flick_distance: f64,
         /// Logical pixels per millisecond toward the edge that make a flick.
         pub flick_speed: f64,
     }
     impl SwipeDismissPolicy {
         /// A toast card's swipe: touch and pen only, 40 % of its size or 48 px,
         /// or a flick of 48 px faster than 1.2 px/ms.
         pub fn toast(edge: Edge, slop: f64) -> Self {
             Self { edge, inputs: SwipeInputs::TouchAndPen, slop, close_fraction: 0.4, min_distance: 48.0, flick_distance: 48.0, flick_speed: 1.2 }
         }
         /// A drawer's handle: every pointer, the theme's [`DrawerSwipe`].
         pub fn drawer(edge: Edge, swipe: DrawerSwipe) -> Self {
             Self { edge, inputs: SwipeInputs::All, slop: 0.0, close_fraction: swipe.close_fraction, min_distance: 0.0, flick_distance: swipe.flick_distance, flick_speed: swipe.flick_speed }
         }
         pub(crate) fn dismisses(&self, travel: f64, size: f64, speed: f64) -> bool {
             travel >= (size * self.close_fraction).max(self.min_distance)
                 || (travel >= self.flick_distance && speed > self.flick_speed)
         }
     }
     /// Where a swipe is.
     #[derive(Clone, Copy, Debug, PartialEq)]
     pub enum SwipePhase {
         Rest,
         /// A contact is down and has not become a swipe yet.
         Pressed,
         /// The surface follows the contact `offset` px toward its edge; `size` is its extent on that axis.
         Dragging { offset: f64, size: f64 },
         /// A release dismissed the surface `travel` px from its place.
         Dismissed { travel: f64, size: f64 },
     }
     #[derive(Clone, Copy)]
     pub struct SwipeDismissApi {
         session: MachineHandle<machine::Machine>,
         inputs: SwipeInputs,
         phase: Value<SwipePhase>,
         held: Value<bool>,
     }
     /// Create a swipe that calls `on_dismiss` once a release qualifies.
     /// `rect` is the surface's rectangle. Setup only.
     #[track_caller]
     pub fn use_swipe_dismiss<M>(policy: SwipeDismissPolicy, rect: Value<Option<Rect>>, on_dismiss: impl IntoHandler<(), M>) -> SwipeDismissApi {
         let on_dismiss = Callback::from_handler(on_dismiss.into_handler());
         let session = machine(
             Swipe::Rest,
             move || machine::Machine { policy, rect: rect.get(), now: clock_now() },
             move |_, machine::Effect::Dismiss| on_dismiss.call(()),
         );
         let phase = memo(move || session.get().phase());
         let held = memo(move || !matches!(session.get().phase(), SwipePhase::Rest));
         SwipeDismissApi { session, inputs: policy.inputs, phase, held }
     }
     impl SwipeDismissApi {
         pub fn phase(self) -> Value<SwipePhase> { self.phase }
         /// True from a press through the drag and a dismissing release (a toast holds its timer).
         pub fn is_held(self) -> Value<bool> { self.held }
         /// The part the contact presses and moves: the toast card or the drawer's handle. Setup only.
         #[track_caller]
         pub fn control(self) -> Part<PartState> {
             let input = part(|input| input);
             Part::new(input.state(), move |host| {
                 host.part(input).on_pointer(move |event: PointerEvent| self.pointer(&event))
             })
         }
         /// A part for a button inside the surface: its press never starts a swipe. Setup only.
         #[track_caller]
         pub fn protect(self) -> Part<PartState> {
             let input = part(|input| input);
             Part::new(input.state(), move |host| {
                 host.part(input).on_pointer(move |event: PointerEvent| {
                     if matches!(event.phase(), PointerPhase::Down(_)) {
                         self.session.send(Event::Protect(event.id()));
                     }
                     Handled::No
                 })
             })
         }
         fn pointer(self, event: &PointerEvent) -> Handled {
             let before = self.session.state().get_untracked();
             self.session.send(Event::Pointer(*event));
             let after = self.session.state().get_untracked();
             let pos = event.window_pos();
             if !self.inputs.accepts(event.kind()) || !pos.x.is_finite() || !pos.y.is_finite() || matches!(before, Swipe::Released { .. }) {
                 return Handled::No;
             }
             let mine = |swipe: &Swipe| matches!(swipe, Swipe::Dragging { contact, .. } if contact.pointer == event.id());
             match event.phase() {
                 PointerPhase::Down(_) => {
                     if matches!(&after, Swipe::Pressed(contact) if contact.pointer == event.id()) { event.capture(); }
                     Handled::No
                 }
                 PointerPhase::Move if mine(&after) => Handled::Yes,
                 PointerPhase::Up(_) if mine(&before) => Handled::Yes,
                 _ => Handled::No,
             }
         }
     }
     ```
     （`PointerEvent` は `Copy`。release の速度は、toast の旧「押下からの平均」から drawer と同じ「直近 100 ms の速度」（`DragVelocity`）になる。これは toast と drawer を一つの gesture にする設計の決定である。drawer は放したとき `Dismissed` の travel を保ったまま閉じる（旧 drawer は 0 に戻ってから閉じた）。drawer の panel は開くたびに `show` で作り直されるので、次に開いたときは `Rest` から始まる。）
  5. `crates/kivact-interaction/src/lib.rs`：`mod switch_drag;`、`mod swipe_dismiss;` と次の行を足す：
     ```rust
     pub use switch_drag::{Lift, SwitchDragApi, SwitchTrack, ThumbDrag, TrackInput, use_switch_drag};
     pub use swipe_dismiss::{SwipeDismissApi, SwipeDismissPolicy, SwipeInputs, SwipePhase, use_swipe_dismiss};
     ```
     `pub use announcer::{Announcer, use_announcer};`（U23 後の形）を `pub use announcer::{Announcement, Announcer, use_announcer};` にする。
  6. `crates/kivact-interaction/src/announcer.rs`：
     - `struct NoticeId(Generation)` の derive に `Debug` を足す（`#[derive(Clone, Debug, Default, PartialEq, Eq)]`）。
     - `struct Notice` を公開の `Announcement` に改名する：
       ```rust
       /// One message of a live region; a repeated message is a new value.
       #[derive(Clone, Debug, Default, PartialEq, Eq)]
       pub struct Announcement {
           id: NoticeId,
           pub message: String,
       }
       ```
       `Announcer` の field `polite`・`assertive` と `use_announcer` の本体の `Notice` を `Announcement` にする（`Signal<Announcement>`）。
     - `pub fn outlet(self) -> View` と `fn region` を消し、`impl Announcer` に次を足す：
       ```rust
       /// The latest message of the polite region.
       pub fn polite(self) -> Value<Announcement> { self.polite.into_value() }
       /// The latest message of the assertive region.
       pub fn assertive(self) -> Value<Announcement> { self.assertive.into_value() }
       ```
     - import を次の 3 行にする（`Position`、`style`、`IntoView`、`Role`、`View`、`dynamic`、`Node`、`node`、`text` は使わなくなる）：
       ```rust
       use kivact_core::custom::Generation;
       use kivact_core::state::{IntoValue, Signal, Value, signal};
       use kivact_core::view::LiveRegion;
       ```
     - `Announcer` の doc の「created by `outlet()`」を「mounted by `kivact::ui::announcer_outlet`」にする。`announce` は変えない（`update` の戻り値は使っていない）。
  7. `crates/kivact-ui/src/announcer_outlet.rs` を作り、`crates/kivact-ui/src/lib.rs` に `mod announcer_outlet;` と `pub use announcer_outlet::announcer_outlet;` を足す：
     ```rust
     //! Role: Mount the two live regions of an `Announcer` outside the layout.
     use kivact_core::state::Value;
     use kivact_core::style::{Position, style};
     use kivact_core::view::{LiveRegion, Node, Role, dynamic, node, text};
     use kivact_interaction::{Announcement, Announcer};

     /// The polite and the assertive live region of `announcer`, outside the
     /// layout. Mount it once.
     pub fn announcer_outlet(announcer: Announcer) -> Node {
         node()
             .hit_testable(false)
             .style(style().position(Position::Absolute).left(0).top(0).size(0, 0))
             .child(region(announcer.polite(), LiveRegion::Polite))
             .child(region(announcer.assertive(), LiveRegion::Assertive))
     }

     fn region(notice: Value<Announcement>, level: LiveRegion) -> Node {
         node()
             .role(Role::Status)
             .live_region(level)
             .hit_testable(false)
             .style(
                 style()
                     .position(Position::Absolute)
                     .left(-1)
                     .top(-1)
                     .size(1, 1)
                     .clip(true)
                     .opacity(0.0),
             )
             .child(dynamic(
                 move || notice.get(),
                 |notice| (!notice.message.is_empty()).then(|| text(notice.message)),
             ))
     }
     ```
     （`region` の本体は interaction の旧 `fn region` と同じ。W8 の U47 がこの関数を `#[component]` にし `AnnouncerOutletTheme` を作る。）
  8. ui の outlet の呼出しを直す：
     - `crates/kivact-ui/src/empty_state.rs`：`use_announcer()` の後の `notices.outlet()` → `crate::announcer_outlet(notices)`。
     - `crates/kivact-ui/src/empty_state/changes.rs`：`(content, notices.outlet()).into_view()` → `(content, crate::announcer_outlet(notices)).into_view()`。
     - `crates/kivact-ui/src/combobox/notices.rs`：`pub(super) fn outlet(self) -> View { self.outlet.outlet() }` の本体を `crate::announcer_outlet(self.outlet).into_view()` にし、`use kivact_core::view::{LiveRegion, View};` に `IntoView` を足す。`combobox/render.rs` の `.child(notices.outlet())` は変えない（`Notices::outlet` の署名は同じ）。
  9. `crates/kivact-interaction/src/toaster/data.rs`：
     - field `pub icon: Option<kivact_core::view::ViewFn<()>>` とその doc を次にする：
       ```rust
       /// The name in the theme's icon set of the icon a kit draws instead of
       /// the icon of its tone, if there is one.
       pub icon_name: Option<&'static str>,
       ```
     - `impl PartialEq for Toast` の `let icon = self.icon == other.icon;` → `let icon = self.icon_name == other.icon_name;`。その doc の「and their action and icon are the same functions」を「their icon names are and their action is the same function」にする。
     - `Toast::new` の `icon: None` → `icon_name: None`。
     - setter `pub fn icon<V: IntoView>(mut self, icon: impl kivact_core::view::IntoViewFn<V>) -> Self` とその doc を次に置き換える：
       ```rust
       /// The name in the theme's icon set of an icon drawn instead of the icon
       /// of its tone.
       pub fn icon_name(mut self, name: &'static str) -> Self {
           self.icon_name = Some(name);
           self
       }
       ```
     - `use kivact_core::view::{HandlerResult, IntoView};` → `use kivact_core::view::HandlerResult;`。
  10. `crates/kivact-ui/src/toast.rs`：
      - import：`use swipe::Swipe;`（U12 後の形。`mod swipe;` は残す）を消し、次を足す：
        ```rust
        use kivact_core::geometry::Edge;
        use kivact_core::task::spawn;
        use kivact_interaction::{SwipeDismissApi, SwipeDismissPolicy, use_swipe_dismiss};
        ```
      - `fn card_icon` の本体を次にする（`None` の腕の `with_recipe_style(..)` の式は W7a が残した形のまま、`name` を受ける closure に入れる）：
        ```rust
        content.icon_name.or_else(|| tone_icon(content.tone)).map(|name| {
            /* 旧 `None => tone_icon(content.tone).map(|name| { .. })` の closure の本体をそのまま */
        })
        ```
      - `fn toast` の戻り値の型 `(View, Option<(Swipe, Transition)>)` を `(View, Option<(SwipeDismissApi, Edge, Transition)>)` にする。`let swipe = use_theme()..toast.swipe.clone();` から `let gesture = swipe.as_ref().map(|(gesture, _)| *gesture);` までを次にする（旧 `Swipe::new` の effect の中の「A task runs in the same turn ..」の comment を `move ||` の上に移す）：
        ```rust
        let policy = use_theme()
            .get()
            .ext::<kivact_interaction::InteractionPolicy>()
            .toast
            .clone();
        let (part, swiping) = match &policy.swipe {
            ToastSwipe::Disabled => (toaster.item(id), None),
            ToastSwipe::Enabled { edge, transition } => {
                let gesture = use_swipe_dismiss(
                    SwipeDismissPolicy::toast(*edge, policy.swipe_slop),
                    card_ref.rect(),
                    // （旧 comment）
                    move || {
                        spawn(async move { toaster.dismiss(id) });
                    },
                );
                (toaster.item_while(id, gesture.is_held()), Some((gesture, *edge, transition.clone())))
            }
        };
        let gesture = swiping.as_ref().map(|(gesture, _, _)| *gesture);
        ```
      - `fn toast` の最後の `if let Some((gesture, transition)) = swipe { .. } else { .. }` を次にする：
        ```rust
        if let Some((gesture, edge, transition)) = swiping {
            let exit = transition.clone();
            let card = card
                .part(gesture.control())
                .style(move || swipe::card_style(edge, gesture.phase().get(), &transition))
                .into_view();
            (card, Some((gesture, edge, exit)))
        } else {
            (card.into_view(), None)
        }
        ```
      - `fn action_button` と `fn close_button` の引数 `gesture: Option<Swipe>` を `gesture: Option<SwipeDismissApi>` にする。`action_button` の `action.on_pointer(move |event| gesture.protect_action(event))` → `action.with_node(|node| node.part(gesture.protect()))`。`close_button` の `close.on_pointer(move |event| gesture.protect_action(event))` → `close.part(gesture.protect())`。
      - `ToastRegion::into_view` の `let (card, swipe) = build(id.get_untracked());` → `let (card, swiping) = build(id.get_untracked());`、その下の `match swipe { Some((swipe, exit)) => row.style(move || swipe.row_style(&exit)), None => row }` → `match swiping { Some((gesture, _, exit)) => row.style(move || swipe::row_style(gesture.phase().get(), &exit)), None => row }`。
  11. `crates/kivact-ui/src/toast/swipe/` を directory ごと消し、`crates/kivact-ui/src/toast/swipe.rs` の中身を次に置き換える（見た目だけ。machine は持たない）：
      ```rust
      //! Role: The look of a toast card that a swipe moves or dismissed.
      use kivact_core::geometry::Edge;
      use kivact_core::motion::{Easing, Transition};
      use kivact_core::style::{Style, TransitionProp, pct, style};
      use kivact_interaction::SwipePhase;

      fn sign(edge: Edge) -> f64 {
          match edge { Edge::Top | Edge::Left => -1.0, Edge::Bottom | Edge::Right => 1.0 }
      }
      fn shift(edge: Edge, distance: f64, base: Style) -> Style {
          match edge {
              Edge::Top | Edge::Bottom => base.translate_y(distance * sign(edge)),
              Edge::Left | Edge::Right => base.translate_x(distance * sign(edge)),
          }
      }
      /// The card's style: it follows the drag and leaves from where a
      /// dismissing release let go. Only a dismissing swipe slides the card
      /// off; any other dismissal keeps the card's own exit.
      pub(super) fn card_style(edge: Edge, phase: SwipePhase, transition: &Transition) -> Style {
          let timing = if matches!(phase, SwipePhase::Dragging { .. }) {
              Transition::tween(0.0, Easing::Linear)
          } else {
              transition.clone()
          };
          let base = style().transition(TransitionProp::All, timing);
          let (travel, size, base) = match phase {
              SwipePhase::Dragging { offset, size } => (offset, size, base),
              SwipePhase::Dismissed { travel, size } => {
                  let away = match edge {
                      Edge::Top | Edge::Bottom => style().opacity(0.0).translate_y(pct(120.0 * sign(edge))),
                      Edge::Left | Edge::Right => style().opacity(0.0).translate_x(pct(120.0 * sign(edge))),
                  };
                  (travel, size, base.exit(away.scale(1.0)))
              }
              SwipePhase::Rest | SwipePhase::Pressed => return base,
          };
          let progress = (travel / size).clamp(0.0, 1.0);
          shift(edge, travel, base).opacity(1.0 - 0.4 * progress).scale(1.0 - 0.02 * progress)
      }
      /// （旧 `Swipe::row_style` の doc をそのまま）
      pub(super) fn row_style(phase: SwipePhase, exit: &Transition) -> Style {
          if !matches!(phase, SwipePhase::Dismissed { .. }) {
              return style();
          }
          /* 旧 `toast/swipe/handle.rs` の `Swipe::row_style` の本体のうち、最初の
             `if !matches!(..) { return style(); }` より後の行をそのまま
             （W7a の U21 が `UiTokens` の読みを toast の theme の設定の読みに書き換えた形） */
      }
      ```
      `row_style` の本体が使う `use`（`use_theme` と W7a 後の toast の theme の型）は旧 `toast/swipe/handle.rs` の `use` から同じ行を写す。
  12. `crates/kivact-ui/src/dialog/drawer_swipe.rs`：`dialog.rs` が呼ぶ `pub(crate) fn handle(..) -> (Node, Value<f64>)` と `pub(crate) fn translation(edge, distance)` の署名は W7b の始めの形のまま変えない。`enum Swipe`、`enum Event`、`enum Effect`、`struct Machine`、`impl MachineTrait for Machine`、`fn toward` を消す。`handle` の本体の `let policy = use_theme()..drawer.swipe;` から `let pulled = memo(..);` の終わりまでを次にする：
      ```rust
      let policy = use_theme().get().ext::<InteractionPolicy>().drawer.swipe;
      let swipe = use_swipe_dismiss(SwipeDismissPolicy::drawer(edge, policy), panel.rect(), move || open.set(false));
      let part = swipe.control();
      let state = part.state();
      let phase = swipe.phase();
      let pulled = memo(move || match phase.get() {
          SwipePhase::Dragging { offset, .. } => offset,
          SwipePhase::Dismissed { travel, .. } => travel,
          SwipePhase::Rest | SwipePhase::Pressed => 0.0,
      });
      ```
      recipe の状態の `input: state.get().input` → `input: state.get()`（part が `Part<PartState>` になるため）。同じ状態の `kind:` の行（U23 後は `DialogKind::Drawer(edge)`）は変えない。import を次にする：`kivact_core::geometry` からは `Edge` だけ、`kivact_core::state` からは `Value`・`memo` と `open` の型 `Signal`（W5 後の形のまま）、`kivact_interaction` からは `InteractionPolicy`・`SwipeDismissPolicy`・`SwipePhase`・`use_swipe_dismiss`（`Point`、`Rect`、`Machine`、`Step`、`machine`、`DragCoordinates`、`DragPhase`、`DragUpdate`、`use_drag` を消す）。
  13. ui の switch：
      - `crates/kivact-ui/src/toggle/switch_drag.rs` と `crates/kivact-ui/src/toggle/switch_drag/` を消し、`crates/kivact-ui/src/toggle/switch_track.rs` を作る：
        ```rust
        //! Role: The thumb geometry the default switch look draws, by size.
        use crate::ControlSize;
        use kivact_interaction::SwitchTrack;

        pub(super) fn switch_track(size: ControlSize) -> SwitchTrack {
            if matches!(size, ControlSize::ExtraSmall | ControlSize::Small) {
                SwitchTrack { width: 32.0, height: 20.0, off_side: 14.0, on_side: 16.0, off_inset: 3.0 }
            } else {
                SwitchTrack { width: 40.0, height: 24.0, off_side: 16.0, on_side: 20.0, off_inset: 4.0 }
            }
        }
        ```
      - `crates/kivact-ui/src/toggle.rs`：`mod switch_drag;` → `mod switch_track;`。`switch_drag::switch as switch_gesture` を import する `use` の行を消し、次を置く：
        ```rust
        type Visual = Option<Value<Option<kivact_interaction::ThumbDrag>>>;

        /// A switch's opted-in drag on its control part, and the thumb it moves.
        fn switch_gesture(
            control: kivact_core::custom::Part<ToggleState>,
            value: kivact_core::state::Signal<bool>,
            size: Value<ControlSize>,
            on_edit: Option<kivact_core::state::Callback<kivact_core::view::EditPhase>>,
        ) -> (kivact_core::custom::Part<ToggleState>, Visual) {
            let drag = kivact_interaction::use_switch_drag(value, move || switch_track::switch_track(size.get()));
            if !drag.is_enabled() {
                return (control, None);
            }
            if let Some(observer) = on_edit {
                drag.on_edit(move |phase| observer.call(phase));
            }
            (drag.control(control), Some(drag.thumb()))
        }
        ```
        `no_gesture` の戻り値の型と `checkbox_control`・`switch_control` の引数の型の `switch_drag::Visual`（path の形を問わない）を `Visual` にする。
      - `crates/kivact-ui/src/toggle/look.rs`：`ThumbDrag`・`impl ThumbDrag`・`TrackInput`・`Lift` の定義を消し（手順 2 で interaction に移した）、`use kivact_interaction::ThumbDrag;` を足す。`rg -nw "TrackInput|Lift" crates/kivact-ui/src/toggle/look.rs` が行を返したら、その名前を同じ `use` に足す。`CheckControl::Switch { drag: Option<ThumbDrag>, .. }` などの参照は名前のまま解決する。
      - 再公開を消す：`rg -nw "ThumbDrag|TrackInput|Lift" crates/kivact-ui/src/theme.rs crates/kivact-ui/src/lib.rs` で出た `pub use` の行（U20 後は `theme.rs` の `pub use crate::toggle::look::{..}`）から `ThumbDrag`、`TrackInput`、`Lift` を消す。公開の path は `kivact::interaction::{ThumbDrag, TrackInput, Lift}` だけになる（同じ名前を二つの module から出さない）。
- **消すもの**：`Announcer::outlet`、interaction の `fn region`、`Notice`（`Announcement` に改名）、`Toast::icon`（field と setter）、ui の `toggle/switch_drag.rs` と `toggle/switch_drag/{geometry.rs,handle.rs,machine.rs}`、`toast/swipe/{handle.rs,machine.rs}`、ui の toast の `Swipe`（`new`・`held`・`protect_action`・`event`・`row_style`・`style`）、drawer の `Swipe`・`Event`・`Effect`・`Machine`・`toward`、ui の `ThumbDrag`・`TrackInput`・`Lift` の定義と再公開。
- **状態の閉じ方**：`SwitchTrack`（f64 だけ）、`ThumbDrag`・`TrackInput`・`Lift`・`SwipeInputs`（enum）は対象外。`SwitchDragApi`・`SwipeDismissApi`（Api）、`SwipeDismissPolicy`（Policy）、`SwipePhase`（Phase）は名前で対象外。switch の `Contact` と `machine::Machine`（bool と u64）、swipe の `machine::Machine`（`Option<Rect>` が一つと u64）、swipe の `Contact`（bool・Option を持たない）は constructor でだけ書く snapshot で、`&mut self` の method も内部可変も持たないので対象外。`Swipe`・`State` は enum。`Toast` の Option の field（body・duration・action・icon_name）は値渡しの builder だけで書くので対象外。`Announcement` は bool・Option を持たない。印は要らない。
- **試験**：
  - 変更 `view.rs::announcements_publish_in_separate_persistent_live_regions`：import に `use kivact::ui::announcer_outlet;` を足し、`column(notices.outlet()).test_id("notices")` → `announcer_outlet(notices).test_id("notices")`。`let notices = use_announcer();` と `use kivact::interaction::use_announcer;` は U23 の形のまま。assert は変えない。
  - 追加 `interaction_toast.rs`：import を直す。`use crate::support::mount;` → `use crate::support::{mount, mount_with};`、U23 後の `use kivact::interaction::{Toast, ToastOptions, ToasterApi, use_toaster};` に `InteractionPolicy, ToastPolicy, ToastSwipe` を足す、`use kivact::test::Screen;` → `use kivact::test::{RenderOptions, Screen};`、`use kivact::view::{LiveRegion, View};` → `use kivact::view::{LiveRegion, MouseButton, PointerKind, View};`、`use kivact::geometry::Edge;` を足す（`Easing`・`Transition` は `kivact::prelude` にある）。そして次を足す：
    ```rust
    fn swipe_options() -> RenderOptions {
        RenderOptions::with_theme(Theme::light().with(InteractionPolicy {
            toast: ToastPolicy {
                swipe: ToastSwipe::Enabled { edge: Edge::Bottom, transition: Transition::tween(150.0, Easing::Linear) },
                ..ToastPolicy::default()
            },
            ..InteractionPolicy::default()
        }))
    }

    #[test]
    fn only_a_touch_or_pen_swipe_toward_the_edge_dismisses_a_toast() {
        for kind in [PointerKind::Touch, PointerKind::Pen] {
            let (screen, ()) = mount_with(swipe_options(), || {
                let queue = use_toaster();
                queue.push(Toast::new("Saved").duration(None));
                ((), kivact::ui::toast_region(queue))
            });
            screen.advance(ms(150));
            let card = || screen.get_by_role(Role::Status, "Saved");
            let rect = card().rect().unwrap();
            let (x, y) = (rect.x + 25.0, rect.y + 5.0);
            screen.mouse_move(x, y);
            screen.mouse_down(MouseButton::Primary);
            screen.mouse_move(x, y + 52.0);
            screen.mouse_up(MouseButton::Primary);
            card();
            let close = screen.get_by_role(Role::Button, "Close").rect().unwrap();
            let (close_x, close_y) = (close.x + close.width / 2.0, close.y + close.height / 2.0);
            let contact = screen.contact_down(kind, close_x, close_y);
            screen.contact_move(contact, close_x, close_y + 52.0);
            screen.contact_up(contact);
            card();
            let contact = screen.contact_down(kind, x, y);
            screen.contact_move(contact, x, y - 52.0);
            screen.contact_up(contact);
            card();
            let contact = screen.contact_down(kind, x, y);
            screen.contact_move(contact, x + 15.0, y + 2.0);
            screen.contact_move(contact, x, y + 52.0);
            screen.contact_up(contact);
            card();
            let contact = screen.contact_down(kind, x, y);
            screen.contact_move(contact, x, y + 25.0);
            screen.contact_up(contact);
            screen.advance(ms(150));
            card();
            let contact = screen.contact_down(kind, x, y);
            screen.contact_move(contact, x, y + 52.0);
            screen.contact_up(contact);
            screen.advance(ms(32));
            assert!(screen.query_by_role(Role::Status, "Saved").is_none());
            ok(&screen);
        }
    }

    #[test]
    fn a_pending_touch_holds_only_its_toasts_timer() {
        let (screen, ()) = mount_with(swipe_options(), || {
            let queue = use_toaster();
            for title in ["Held", "Other"] {
                queue.push(Toast::new(title).duration(Some(ms(350))));
            }
            ((), kivact::ui::toast_region(queue))
        });
        screen.advance(ms(150));
        let card = screen.get_by_role(Role::Status, "Held").rect().unwrap();
        let contact = screen.contact_down(PointerKind::Touch, card.x + 24.0, card.y + 8.0);
        screen.advance(ms(400));
        assert!(screen.query_by_role(Role::Status, "Held").is_some());
        assert!(screen.query_by_role(Role::Status, "Other").is_none());
        screen.contact_cancel(contact);
        screen.advance(ms(1200));
        assert!(screen.query_by_role(Role::Status, "Held").is_none());
        ok(&screen);
    }
    ```
  - 追加 `ui_controls.rs`（switch の drag は ui の `switch` の試験。`interaction_toggle.rs` は lane U1 が変えるのでここに置く）：import を直す。U23 後の `use kivact::interaction::{ScalarRange, Toast, use_toaster};` に `InteractionPolicy, SwitchDrag, SwitchPolicy` を足し、`use kivact::view::{A11yAction, MouseButton, Orientation};` に `PointerKind` を足す（`mount_with` と `RenderOptions` は import 済み）。そして次を足す：
    ```rust
    #[test]
    fn a_switch_drag_commits_once_and_vertical_or_cancelled_contacts_do_not_toggle() {
        let options = RenderOptions::with_theme(Theme::light().with(InteractionPolicy {
            switch: SwitchPolicy { drag: SwitchDrag(true) },
            ..InteractionPolicy::default()
        }));
        let (screen, (on, writes)) = mount_with(options, || {
            let on = signal(false);
            let writes = signal(0);
            let value = binding(on, move |next| {
                writes.update(|count| *count += 1);
                on.set(next);
            });
            ((on, writes), kivact::ui::switch("Notifications", value))
        });
        let control = screen.get_by_role(Role::Switch, "Notifications");
        let bounds = control.accessibility_bounds().unwrap();
        let (x, y) = (bounds.x + 14.0, bounds.y + bounds.height / 2.0);
        let touch = || screen.contact_down(PointerKind::Touch, x, y);
        let finger = touch();
        assert!(!on.get_untracked());
        screen.contact_up(finger);
        assert!(on.get_untracked());
        assert_eq!(writes.get_untracked(), 1);
        let finger = touch();
        screen.contact_move(finger, x, y + 24.0);
        screen.contact_up(finger);
        assert!(on.get_untracked());
        assert_eq!(writes.get_untracked(), 1);
        let finger = touch();
        screen.contact_move(finger, x + 30.0, y);
        screen.contact_cancel(finger);
        assert!(on.get_untracked());
        assert_eq!(writes.get_untracked(), 1);
        control.press_key(Key::Space);
        assert!(!on.get_untracked());
        assert_eq!(writes.get_untracked(), 2);
        control.press_key(Key::Enter);
        assert!(!on.get_untracked());
        assert_eq!(writes.get_untracked(), 2);
        let finger = touch();
        screen.contact_move(finger, x + 28.0, y);
        screen.contact_move(finger, x + 30.0, y);
        assert!(!on.get_untracked());
        screen.contact_move(finger, x - 3.0, y);
        screen.contact_up(finger);
        assert!(!on.get_untracked(), "a drag that flips and returns keeps the starting side");
        assert_eq!(writes.get_untracked(), 2);
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
    ```
  - 変えずに通す：`ui_disclosure_surfaces.rs::drawer_closes_on_swipe`（slop 0 の drawer は 250 px の一回の move で Dragging になり、50 ms で 250 px の速度と距離の両方で閉じる）、`interaction_toast.rs` の既存の全試験、`interaction_toggle.rs` の全試験、kivact-interaction の unit 試験 `swipe_dismiss::machine::tests::intent_and_flick_thresholds_match_the_edge`。
  - `presence_kit.rs::toast_swipe_dismiss_exits_from_the_drag_offset`（kit を使うので `e2e_kits`。W6〜W10 は compile しない）は annex K が直す（4 章の対応表：速度の測り方が直近 100 ms になったことだけが違い、試験の動きは距離で消えるので期待値は変わらない）。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction -p kivact-ui -p kivact-public-api-tests
  cargo check -p kivact-interaction -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo xtask check --no-write --crate kivact-ui
  KIVACT_TEST_MIRROR=off cargo test -p kivact-interaction --lib swipe_dismiss
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- view::announcements_publish_in_separate_persistent_live_regions
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_toast ui_controls interaction_toggle ui_disclosure_surfaces
  ```
- **完了条件**：`rg -n "fn outlet|pub icon:|fn icon<" crates/kivact-interaction/src` が空。`rg -n "impl.*Machine for" crates/kivact-ui/src/toggle crates/kivact-ui/src/toast crates/kivact-ui/src/dialog` が空。`rg -n "pub enum (ThumbDrag|TrackInput|Lift)" crates/kivact-ui/src` が空。`rg -nw "ThumbDrag|TrackInput|Lift" crates/kivact-ui/src/theme.rs crates/kivact-ui/src/lib.rs` が空。`rg -n "kivact_core::view::(View|IntoView)|\bIntoView\b" crates/kivact-interaction/src/announcer.rs crates/kivact-interaction/src/toaster/data.rs` が空。`ls crates/kivact-ui/src/toggle/switch_drag.rs crates/kivact-ui/src/toggle/switch_drag crates/kivact-ui/src/toast/swipe` がどれも「無い」と答える。上の命令がすべて通る。

---

## U34：UI-04 edit machine（`use_date_field`・`use_draft<T>`）
- **wave / lane / 前提**：W7b / U3 / W7a の全 unit（U20 → U21 → U25 → U22 → U23 → U24 の順）が完了。この unit は次の状態から始める。
  - W5・W6 の API：`Signal<T>`（`Binding`・`IntoBinding` は無い）、`binding(get: impl IntoValue<T>, set) -> Signal<T>`、`Callback<E>`／`callback(f)`（`f: impl Fn(E) -> R`、setup の中だけ）、`effect_on`（`kivact_core::state::effect_on`）、`use_direction() -> ReadingDirection`、`Theme::light()`、`theme.with(ext)`、`Trigger`。`Signal::update`・`Var::update` は `()` を返す（DEC-15）。この unit のどの手順も `update` の戻り値を使わない（`DraftApi::update` は別の method で、`()` を返す）。
  - handler の形（DEC-16）：payload のある handler・setter（`on_commit`、`on_key`、`on_write` など）は `impl Fn(E) -> R + 'static`（`R: HandlerResult`）を受ける。payload の無い handler だけが `impl IntoHandler<(), M>` を受ける。共有した `Callback<E>` は `move |e| cb.call(e)` で渡す。
  - W7a の結果：U22 の `.label()` の呼出しの形（`text_input(value).label(..)`、`date_input(value).label(..)`、`time_input(value).label(..)`、`number_input(value, range).label(..)`）、U23 の `on_commit`（旧 `on_draft_commit`）と ui の `DialogKind`（`date_input/view.rs` の `DialogMode` の行は U23 で済んでいる）、U20 の `<Name>Theme`、U21 の theme 変数、U24 の `Element::variant`、U25 の `#[caller] caller: Caller`（`Node::authored` の snapshot）。
  - W3 の U08 の結果：date・range・time の view の `kivact_core::state::effect(move || state.send(Event::External(value.get())))` は `state.feed(value, Event::External)` の形（range は `draft.feed(value, DraftEvent::External)`）になっている。
  - W4 の移動（U11〜U14）の後の配置：`crates/kivact-ui/src/date_input.rs` と `date_input/{look.rs, machine.rs, view.rs}`、`time_input.rs` と `time_input/{look.rs, machine.rs, view.rs, parse.rs}`（旧 `time_parse.rs`）、`date_time_input.rs` と `date_time_input/{look.rs, model.rs, sheet.rs}`、`date_range_input.rs` と `date_range_input/{look.rs, draft.rs}`、`pagination.rs` と `pagination/{look.rs, draft.rs}`、`number_input.rs` と `number_input/{look.rs, draft.rs}`。module の root は `<name>.rs`（clippy `mod_module_files` は deny。`mod.rs` は作らない）。
  - W1 で `xtask-allow(state-fields)` の印はすべて消えている。e2e の binary は annex T の割当てどおり：`interaction_*`・`ui_*`・`apg_*` は `--test e2e_ui`。
  - 同じ W7b の他の lane との境界（2.2 節）：`number_input.rs` はこの unit が持つ（draft の配線）。`use_scalar(..).disabled(disabled)` の行は W8 の U42 が直すので、この unit は触らない。
- **所有**：
  - 新規：`crates/kivact-interaction/src/draft.rs`、`crates/kivact-interaction/src/date_field.rs`、`crates/kivact-interaction/src/date_field/machine.rs`
  - 変更：`crates/kivact-interaction/src/lib.rs`（`mod`・`pub use` の行だけ）、`crates/kivact-ui/src/date_input.rs`、`crates/kivact-ui/src/date_input/view.rs`、`crates/kivact-ui/src/time_input.rs`、`crates/kivact-ui/src/time_input/view.rs`、`crates/kivact-ui/src/date_time_input.rs`、`crates/kivact-ui/src/date_time_input/model.rs`、`crates/kivact-ui/src/date_time_input/sheet.rs`、`crates/kivact-ui/src/date_range_input.rs`、`crates/kivact-ui/src/date_range_input/draft.rs`、`crates/kivact-ui/src/pagination/draft.rs`、`crates/kivact-ui/src/number_input.rs`（typed text の部分だけ）
  - 削除：`crates/kivact-ui/src/date_input/machine.rs`、`crates/kivact-ui/src/time_input/machine.rs`、`crates/kivact-ui/src/number_input/draft.rs`
  - 試験：`dev/tests/kivact-public-api/tests/e2e_ui/interaction_edit.rs`
- **目的**：UI-04 規則 1：入力で動く date・time の text と picker の machine、range・page・number の draft の machine を interaction に移す（ui は入力で動く `impl Machine` も signal を書く `effect` も持たない、5.9 節・6.3 節）。date・time の field の hook の名前は `use_date_field`（中身は typed text と picker で、年・月・日の segment の編集ではないため。監査の仮名 `use_date_segments` から改名した）。
- **手順**：
  1. `crates/kivact-interaction/src/draft.rs` を作る：
     ```rust
     //! Role: Edit a draft of a value and write it to the value's signal on commit.
     use kivact_core::state::{IntoValue, Machine as MachineTrait, MachineHandle, Signal, Step, Value, binding, effect_on, machine, memo};
     use std::rc::Rc;

     /// When an accepted draft is written.
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub enum DraftCommit {
         /// Every edit whose draft is accepted writes at once (a number field, a range's endpoints).
         Live,
         /// Only `commit()` and leaving the editor write (a page field).
         Explicit,
     }
     /// How a draft shows a value and which value a draft stands for.
     pub struct DraftOptions<T: 'static, D: 'static> {
         show: Rc<dyn Fn(&T) -> D>,
         accept: Rc<dyn Fn(&D) -> Option<T>>,
         commit: DraftCommit,
         valid: Option<Signal<bool>>,
     }
     impl<T: 'static, D: 'static> DraftOptions<T, D> {
         /// `show` is the draft of a value (its text); `accept` is the value a draft
         /// stands for, `None` when it is not a valid value. Explicit commit.
         pub fn new(show: impl Fn(&T) -> D + 'static, accept: impl Fn(&D) -> Option<T> + 'static) -> Self {
             Self { show: Rc::new(show), accept: Rc::new(accept), commit: DraftCommit::Explicit, valid: None }
         }
         pub fn commit(mut self, commit: DraftCommit) -> Self { self.commit = commit; self }
         /// Write whether the draft is accepted to `valid` after every change.
         pub fn valid(mut self, valid: Option<Signal<bool>>) -> Self { self.valid = valid; self }
     }
     #[derive(Clone, PartialEq)]
     enum State<T, D> {
         /// The editor shows the value.
         Following,
         /// The user's draft; `produced` is the value its last accepted edit wrote (Live only).
         Editing { draft: D, produced: Option<T> },
     }
     enum Event<T, D> { Edit(D), Revise(D), Commit, Cancel, Blur, Changed(T), Wrote(T) }
     enum Effect<T> { Write(T), Valid(bool) }
     struct Machine<T: 'static, D: 'static> { value: T, show: Rc<dyn Fn(&T) -> D>, accept: Rc<dyn Fn(&D) -> Option<T>>, commit: DraftCommit }
     impl<T: Clone + PartialEq + 'static, D: Clone + PartialEq + 'static> MachineTrait for Machine<T, D> {
         type State = State<T, D>; type Event = Event<T, D>; type Effect = Effect<T>;
         fn transition(&self, state: &State<T, D>, event: Event<T, D>) -> Step<State<T, D>, Event<T, D>, Effect<T>> {
             let produced = match state { State::Editing { produced, .. } => produced.clone(), State::Following => None };
             let next = match event {
                 Event::Edit(draft) => {
                     let accepted = (self.accept)(&draft);
                     let step = self.to(State::Editing { draft, produced });
                     return match accepted { Some(value) if self.commit == DraftCommit::Live => step.with(Effect::Write(value)), _ => step };
                 }
                 Event::Revise(draft) => State::Editing { draft, produced },
                 Event::Wrote(value) => match state {
                     State::Editing { draft, .. } => State::Editing { draft: draft.clone(), produced: Some(value) },
                     State::Following => return Step::stay(),
                 },
                 Event::Commit => return self.commit_draft(state),
                 Event::Blur if self.commit == DraftCommit::Explicit => return self.commit_draft(state),
                 Event::Blur | Event::Cancel => State::Following,
                 Event::Changed(value) => match state {
                     State::Editing { draft, produced } if (self.accept)(draft).as_ref() == Some(&value) || produced.as_ref() == Some(&value) => return Step::stay(),
                     _ => State::Following,
                 },
             };
             self.to(next)
         }
     }
     impl<T: Clone + PartialEq + 'static, D: Clone + PartialEq + 'static> Machine<T, D> {
         /// Go to `next` and report whether its draft is accepted.
         fn to(&self, next: State<T, D>) -> Step<State<T, D>, Event<T, D>, Effect<T>> {
             let valid = match &next { State::Following => (self.accept)(&(self.show)(&self.value)).is_some(), State::Editing { draft, .. } => (self.accept)(draft).is_some() };
             Step::to(next).with(Effect::Valid(valid))
         }
         fn commit_draft(&self, state: &State<T, D>) -> Step<State<T, D>, Event<T, D>, Effect<T>> {
             let State::Editing { draft, .. } = state else { return Step::stay() };
             match (self.accept)(draft) {
                 Some(value) if self.commit == DraftCommit::Explicit => self.to(State::Following).with(Effect::Write(value)),
                 Some(_) => self.to(State::Following),
                 None => Step::stay(),
             }
         }
     }
     /// A draft of `value` that an editor changes and a commit writes.
     pub struct DraftApi<T: Clone + PartialEq + 'static, D: Clone + PartialEq + 'static> {
         session: MachineHandle<Machine<T, D>>,
         draft: Value<D>,
         editing: Value<bool>,
         valid: Value<bool>,
     }
     impl<T: Clone + PartialEq + 'static, D: Clone + PartialEq + 'static> Clone for DraftApi<T, D> { fn clone(&self) -> Self { *self } }
     impl<T: Clone + PartialEq + 'static, D: Clone + PartialEq + 'static> Copy for DraftApi<T, D> {}

     /// Create a draft of `value`. Setup only.
     #[track_caller]
     pub fn use_draft<T: Clone + PartialEq + 'static, D: Clone + PartialEq + 'static>(value: Signal<T>, options: DraftOptions<T, D>) -> DraftApi<T, D> {
         let DraftOptions { show, accept, commit, valid } = options;
         let (snapshot_show, snapshot_accept) = (show.clone(), accept.clone());
         let session = machine(
             State::Following,
             move || Machine { value: value.get_untracked(), show: snapshot_show.clone(), accept: snapshot_accept.clone(), commit },
             move |cx, effect| match effect {
                 Effect::Write(next) => { value.set(next); cx.send(Event::Wrote(value.get_untracked())); }
                 Effect::Valid(next) => if let Some(valid) = valid { valid.set(next) },
             },
         );
         session.feed(value, Event::Changed);
         let draft_show = show.clone();
         let draft = memo(move || match session.get() { State::Following => draft_show(&value.get()), State::Editing { draft, .. } => draft });
         let editing = memo(move || matches!(session.get(), State::Editing { .. }));
         let valid = memo(move || draft.with(|draft| accept(draft).is_some()));
         DraftApi { session, draft, editing, valid }
     }
     impl<T: Clone + PartialEq + 'static, D: Clone + PartialEq + 'static> DraftApi<T, D> {
         /// The draft the editor shows: the value's while following it.
         pub fn draft(self) -> Value<D> { self.draft }
         pub fn is_editing(self) -> Value<bool> { self.editing }
         /// Whether the shown draft stands for a value.
         pub fn is_valid(self) -> Value<bool> { self.valid }
         /// Replace the draft (Live: an accepted draft is written at once).
         pub fn edit(self, draft: D) { self.session.send(Event::Edit(draft)); }
         /// Change the shown draft in place, as `edit` does.
         pub fn update(self, change: impl FnOnce(&mut D)) { let mut draft = self.draft.get_untracked(); change(&mut draft); self.edit(draft); }
         /// Change the shown draft without writing, even when Live (a validity report).
         pub fn revise(self, change: impl FnOnce(&mut D)) { let mut draft = self.draft.get_untracked(); change(&mut draft); self.session.send(Event::Revise(draft)); }
         /// Write an accepted draft (Explicit) and follow the value again; a draft that is not accepted stays.
         pub fn commit(self) { self.session.send(Event::Commit); }
         /// Follow the value again without writing.
         pub fn cancel(self) { self.session.send(Event::Cancel); }
         /// Leaving the editor (`focused` turns false) commits (Explicit) or follows the value (Live). Setup only.
         #[track_caller]
         pub fn settle_on_blur(self, focused: impl IntoValue<bool>) {
             let session = self.session;
             effect_on(focused.into_value(), move |focused| if !*focused { session.send(Event::Blur) });
         }
         /// The draft as a signal an editor edits. Setup only.
         #[track_caller]
         pub fn editor(self) -> Signal<D> { binding(self.draft, move |draft| self.edit(draft)) }
     }
     ```
     （`Machine` を持つ `DraftApi` の型引数には `MachineTrait` の impl と同じ `Clone + PartialEq` の bound が要る。`MachineHandle<M>` の `M: Machine` の bound を満たすため。）
  2. `crates/kivact-interaction/src/date_field/machine.rs` を作る（ui の `date_input/machine.rs` と `time_input/machine.rs` を一つにする）：
     ```rust
     //! Role: The typed text of a date or time field: following the value or edited, and checked on commit.
     use super::{TextCheck, TypedCheck, TypedText};
     use kivact_core::state::{Machine as MachineTrait, Step};
     use std::rc::Rc;
     pub(super) enum Event<T> { External, Edit(String), Commit(String), Pick(T), PickInPlace(T), Clear }
     pub(super) enum Effect<T> { Write(Option<T>), ClosePicker, Valid(bool) }
     pub(super) struct Machine<T: 'static> { pub(super) parse: Rc<dyn Fn(&str) -> Option<T>>, pub(super) accept: Rc<dyn Fn(T) -> bool>, pub(super) check: TypedCheck }
     impl<T: Copy + 'static> Machine<T> {
         pub(super) fn accepted(&self, text: &str) -> Option<T> { (self.parse)(text).filter(|value| (self.accept)(*value)) }
         fn valid(&self, state: &TypedText) -> bool {
             match state { TypedText::Following => true, TypedText::Editing { text, .. } => text.trim().is_empty() || self.accepted(text).is_some() }
         }
         fn to(&self, next: TypedText) -> Step<TypedText, Event<T>, Effect<T>> { let valid = self.valid(&next); Step::to(next).with(Effect::Valid(valid)) }
     }
     impl<T: Copy + 'static> MachineTrait for Machine<T> {
         type State = TypedText; type Event = Event<T>; type Effect = Effect<T>;
         fn transition(&self, _: &TypedText, event: Event<T>) -> Step<TypedText, Event<T>, Effect<T>> {
             match event {
                 Event::External => self.to(TypedText::Following),
                 Event::Edit(text) => self.to(TypedText::Editing { text, check: TextCheck::Unchecked }),
                 Event::Commit(text) if text.trim().is_empty() => self.to(TypedText::Following).with(Effect::Write(None)).with(Effect::ClosePicker),
                 Event::Commit(text) => match self.accepted(&text) {
                     Some(value) => self.to(TypedText::Following).with(Effect::Write(Some(value))).with(Effect::ClosePicker),
                     None => {
                         let step = self.to(TypedText::Editing { text, check: TextCheck::Rejected });
                         if self.check == TypedCheck::WhileTyping { step.with(Effect::ClosePicker) } else { step }
                     }
                 },
                 Event::Pick(value) if (self.accept)(value) => self.to(TypedText::Following).with(Effect::Write(Some(value))).with(Effect::ClosePicker),
                 Event::PickInPlace(value) if (self.accept)(value) => self.to(TypedText::Following).with(Effect::Write(Some(value))),
                 Event::Pick(_) | Event::PickInPlace(_) => Step::stay(),
                 Event::Clear => self.to(TypedText::Following).with(Effect::Write(None)).with(Effect::ClosePicker),
             }
         }
     }
     ```
  3. `crates/kivact-interaction/src/date_field.rs` を作る：
     ```rust
     //! Role: A date or time field: its typed text, its checks and its picker.
     mod machine;
     use kivact_core::state::{Callback, MachineHandle, Signal, Value, binding, callback, machine, memo, signal};
     use kivact_core::view::{HandlerResult, NodeRef};
     use std::rc::Rc;

     /// Which panel of a picker shows.
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub enum DatePanel { Date, Time }
     /// A picker's state: closed, or open on a panel, opened from `invoker`.
     #[derive(Clone, Copy, PartialEq)]
     pub enum DatePicker { Closed, Open { panel: DatePanel, invoker: NodeRef } }
     /// The typed text of a date or time field.
     #[derive(Clone, Debug, PartialEq)]
     pub enum TypedText {
         /// The field shows the value, formatted by the caller.
         Following,
         /// The user's text and whether a commit rejected it.
         Editing { text: String, check: TextCheck },
     }
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub enum TextCheck { Unchecked, Rejected }
     /// How typed text that is no valid value shows and closes the picker.
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub enum TypedCheck {
         /// Invalid only after a commit rejected it; a rejected commit keeps the picker (a date field).
         OnCommit,
         /// Invalid as soon as it is no valid value; every commit closes the picker (a time field).
         WhileTyping,
     }
     pub struct DateFieldOptions<T: 'static> {
         parse: Rc<dyn Fn(&str) -> Option<T>>,
         accept: Rc<dyn Fn(T) -> bool>,
         check: TypedCheck,
         valid: Option<Signal<bool>>,
         on_write: Option<Callback<Option<T>>>,
     }
     impl<T: 'static> DateFieldOptions<T> {
         /// `parse` reads typed text; `accept` is the bounds. `TypedCheck::OnCommit`.
         pub fn new(parse: impl Fn(&str) -> Option<T> + 'static, accept: impl Fn(T) -> bool + 'static) -> Self {
             Self { parse: Rc::new(parse), accept: Rc::new(accept), check: TypedCheck::OnCommit, valid: None, on_write: None }
         }
         pub fn check(mut self, check: TypedCheck) -> Self { self.check = check; self }
         /// Write whether the typed text is a valid optional value after every change.
         pub fn valid(mut self, valid: Option<Signal<bool>>) -> Self { self.valid = valid; self }
         /// Call `f` with each value the field writes (not with external writes). Setup only.
         #[track_caller]
         pub fn on_write<R: HandlerResult>(mut self, f: impl Fn(Option<T>) -> R + 'static) -> Self {
             self.on_write = Some(callback(f));
             self
         }
     }
     #[derive(Clone, Copy)]
     pub struct DatePickerApi { state: Signal<DatePicker>, open: Value<bool> }
     /// A picker's open state, its panel and where the focus returns. Setup only.
     #[track_caller]
     pub fn use_date_picker() -> DatePickerApi {
         let state = signal(DatePicker::Closed);
         DatePickerApi { state, open: memo(move || matches!(state.get(), DatePicker::Open { .. })) }
     }
     impl DatePickerApi {
         pub fn show(self, panel: DatePanel, invoker: NodeRef) { self.state.set(DatePicker::Open { panel, invoker }); }
         /// Show `panel` while open; nothing while closed.
         pub fn show_panel(self, panel: DatePanel) {
             if let DatePicker::Open { invoker, .. } = self.state.get_untracked() { self.state.set(DatePicker::Open { panel, invoker }); }
         }
         pub fn hide(self) { self.state.set(DatePicker::Closed); }
         pub fn state(self) -> Value<DatePicker> { self.state.into_value() }
         pub fn is_open(self) -> Value<bool> { self.open }
         /// The open state as a signal: opening shows `panel` from `invoker`. Setup only.
         #[track_caller]
         pub fn open(self, panel: DatePanel, invoker: NodeRef) -> Signal<bool> {
             binding(self.open, move |open| if open { self.show(panel, invoker) } else { self.hide() })
         }
         /// Where the focus returns on close: the invoker, else `fallback`. Setup only.
         #[track_caller]
         pub fn return_focus(self, fallback: NodeRef) -> Value<NodeRef> {
             let state = self.state;
             memo(move || match state.get() { DatePicker::Open { invoker, .. } => invoker, DatePicker::Closed => fallback })
         }
     }
     pub struct DateFieldApi<T: Copy + PartialEq + 'static> {
         value: Signal<Option<T>>,
         session: MachineHandle<machine::Machine<T>>,
         picker: DatePickerApi,
         typed: Value<TypedText>,
         invalid: Value<bool>,
     }
     impl<T: Copy + PartialEq + 'static> Clone for DateFieldApi<T> { fn clone(&self) -> Self { *self } }
     impl<T: Copy + PartialEq + 'static> Copy for DateFieldApi<T> {}

     /// Create the typed text and the picker of a field editing `value`. Setup only.
     #[track_caller]
     pub fn use_date_field<T: Copy + PartialEq + 'static>(value: Signal<Option<T>>, options: DateFieldOptions<T>) -> DateFieldApi<T> {
         let DateFieldOptions { parse, accept, check, valid, on_write } = options;
         let picker = use_date_picker();
         // The rules the `invalid` memo reads; the machine builds its own snapshot per event.
         let rules = machine::Machine { parse: parse.clone(), accept: accept.clone(), check };
         let session = machine(
             TypedText::Following,
             move || machine::Machine { parse: parse.clone(), accept: accept.clone(), check },
             move |_, effect| match effect {
                 machine::Effect::Write(next) => {
                     value.set(next);
                     if let Some(on_write) = on_write { on_write.call(next); }
                 }
                 machine::Effect::ClosePicker => picker.hide(),
                 machine::Effect::Valid(next) => if let Some(valid) = valid { valid.set(next) },
             },
         );
         session.feed(value, |_| machine::Event::External);
         let typed = session.state();
         let invalid = memo(move || match typed.get() {
             TypedText::Following => false,
             TypedText::Editing { text, check: state } => !text.trim().is_empty() && match rules.check {
                 TypedCheck::OnCommit => state == TextCheck::Rejected,
                 TypedCheck::WhileTyping => rules.accepted(&text).is_none(),
             },
         });
         DateFieldApi { value, session, picker, typed, invalid }
     }
     impl<T: Copy + PartialEq + 'static> DateFieldApi<T> {
         pub fn typed(self) -> Value<TypedText> { self.typed }
         /// Whether the field shows its typed text as invalid.
         pub fn is_invalid(self) -> Value<bool> { self.invalid }
         pub fn edit(self, text: String) { self.session.send(machine::Event::Edit(text)); }
         /// Commit typed text: empty writes `None`, a valid value writes it, other text is rejected.
         pub fn commit(self, text: String) { self.session.send(machine::Event::Commit(text)); }
         /// Write a picked value within the bounds and close the picker.
         pub fn pick(self, value: T) { self.session.send(machine::Event::Pick(value)); }
         /// Write a picked value within the bounds and keep the picker open (a sheet's wheel).
         pub fn pick_in_place(self, value: T) { self.session.send(machine::Event::PickInPlace(value)); }
         pub fn clear(self) { self.session.send(machine::Event::Clear); }
         pub fn picker(self) -> DatePickerApi { self.picker }
         /// The text the editor shows: `format` of the value while following it. Setup only.
         #[track_caller]
         pub fn editor(self, format: impl Fn(T) -> String + 'static) -> Signal<String> {
             let (typed, value) = (self.typed, self.value);
             binding(move || match typed.get() {
                 TypedText::Following => value.get().map(&format).unwrap_or_default(),
                 TypedText::Editing { text, .. } => text,
             }, move |text| self.edit(text))
         }
     }
     ```
     `crates/kivact-interaction/src/lib.rs`：`mod draft;`、`mod date_field;` と次の 2 行を足す：
     ```rust
     pub use draft::{DraftApi, DraftCommit, DraftOptions, use_draft};
     pub use date_field::{DateFieldApi, DateFieldOptions, DatePanel, DatePicker, DatePickerApi, TextCheck, TypedCheck, TypedText, use_date_field, use_date_picker};
     ```
     （crate の中の `text_edit/draft.rs` の `pub(crate) struct DraftOptions` は `text_edit` の中の別の型で、`text_edit/handle.rs` は `use super::draft::{self, DraftOptions};` で明示に読むので衝突しない。変えない。）
  4. `crates/kivact-ui/src/date_input.rs`：`mod machine;` と `pub(crate) use machine::{OpenedBy, Picker};` を消す。`crates/kivact-ui/src/date_input/machine.rs` を消す。`crates/kivact-ui/src/date_input/view.rs`：
     - `use super::machine::{DateField, DateValidation, Event, OpenedBy, Picker, Snapshot, TypedDate};` を `use kivact_interaction::{DateFieldApi, DateFieldOptions, DatePanel, TextCheck, TypedText, use_date_field};` にする。
     - `fn date_controller(..) -> (kivact_core::state::MachineHandle<Snapshot>, Value<String>)` を `-> (DateFieldApi<CalendarDate>, Value<String>)` にし、本体を次にする（旧 effect の `valid.set(true)` は `Valid` の効果が書く）：
       ```rust
       let announcement = kivact_core::state::signal(String::new());
       let chosen = message("calendar.chosen", "Date selected");
       let options = DateFieldOptions::new(
           move |text: &str| locale.get().parse_date(text),
           move |date: CalendarDate| min.get().is_none_or(|min| date >= min) && max.get().is_none_or(|max| date <= max),
       )
       .valid(valid)
       .on_write(move |next: Option<CalendarDate>| {
           if let Some(date) = next {
               announcement.set(format!("{}: {date}", chosen.get_untracked()));
           }
       });
       (use_date_field(value, options), announcement.into_value())
       ```
     - `render`：`let (state, announcement) = date_controller(..);` の直後に `let input_ref = kivact_core::view::node_ref();` と `let button_ref = kivact_core::view::node_ref();` を移し（後ろの同じ 2 行は消す）、続けて `let open = state.picker().open(DatePanel::Date, button_ref);` を置く。旧 `let open = picker_open(state);` の行と、U08 後の `state.feed(value, Event::External);` の行を消す（`use_date_field` が feed する）。`let return_focus = kivact_core::state::memo(move || match state.get().picker { .. });` を `let return_focus = state.picker().return_focus(button_ref);` にする。`let draft = draft_binding(state, format, custom_format.clone(), locale);` を次にする：
       ```rust
       let display = custom_format.clone();
       let draft = state.editor(move |date| {
           display.as_ref().map_or_else(|| locale.get().format_date(date, format.get()), |format| format(date))
       });
       ```
       `date_editor(..)` の呼出しの引数の最後に `input_ref` を足す。
     - `render_trigger`：`let open = picker_open(state);` を `let open = state.picker().open(DatePanel::Date, kivact_core::view::node_ref());` にする（trigger の形では focus は `attach.focus.restore_focus()` が戻すので、invoker は focus の戻り先に使われない）。U08 後の `state.feed(value, Event::External);` の行があれば消す。`content` の closure の中の `Event::SheetPick(date)` → `state.pick_in_place(date)`、`Event::Pick(date)` → `state.pick(date)`（`state.send(if presentation.sheet() { .. } else { .. })` は `if presentation.sheet() { state.pick_in_place(date) } else { state.pick(date) }` にする）、`state.send(Event::Clear)` → `state.clear()`、`state.send(Event::Close)` → `state.picker().hide()`。
     - `fn picker_open` と `fn draft_binding` と `fn checked_invalid` を消す。
     - `date_diagnostic`：`match state.get().typed { TypedDate::Editing { text, validation: DateValidation::Checked } if !text.trim().is_empty() => .. }` を `match state.typed().get() { TypedText::Editing { text, check: TextCheck::Rejected } if !text.trim().is_empty() => .. }` にし、`.hidden(move || !checked_invalid(&state.get().typed))` を `.hidden(move || !state.is_invalid().get())` にする。
     - `date_editor`：引数に `input_ref: kivact_core::view::NodeRef` を足し、`draft` の型を `Signal<String>` にする（W5 後の形）。`.invalid(..)` の closure を次にする：
       ```rust
       .invalid(move || {
           let unchecked = matches!(state.typed().get(), TypedText::Editing { check: TextCheck::Unchecked, .. });
           (!unchecked && external_invalid.is_some_and(|invalid| invalid.get())) || state.is_invalid().get()
       })
       ```
       `on_key` の `state.send(Event::Open(OpenedBy::Input));` → `state.picker().show(DatePanel::Date, input_ref);`。`on_commit`（U23 後の名前）の closure の `state.send(Event::Pick(date));` → `state.pick(date);`、`state.send(Event::Commit(text));` → `state.commit(text);`。
     - `picker_content`：`selected` の binding の中を `render_trigger` と同じに直す（`Event::SheetPick`・`Event::Pick`・`Event::Clear`）。Save の `state.send(Event::Close)` → `state.picker().hide()`。
     - 型 `kivact_core::state::MachineHandle<Snapshot>` の引数をすべて `DateFieldApi<CalendarDate>` にする（`rg -n "MachineHandle<Snapshot>" crates/kivact-ui/src/date_input` が空になるまで）。使わなくなった import は手順 10 で消す。
  5. `crates/kivact-ui/src/time_input.rs`：`mod machine;` を消す。`crates/kivact-ui/src/time_input/machine.rs` を消す。`crates/kivact-ui/src/time_input/view.rs`：
     - `use super::machine::{Event, Snapshot, TimeDraft, TimeInputState};` を `use kivact_interaction::{DateFieldApi, DateFieldOptions, DatePanel, TypedCheck, use_date_field};` にする。
     - `fn time_controller` を次にする：
       ```rust
       fn time_controller(
           value: Signal<Option<TimeOfDay>>,
           seconds: Value<bool>,
           min: Value<Option<TimeOfDay>>,
           max: Value<Option<TimeOfDay>>,
       ) -> DateFieldApi<TimeOfDay> {
           let key = |time: TimeOfDay| (time.hour, time.minute, time.second);
           use_date_field(value, DateFieldOptions::new(
               move |text: &str| super::parse::parse_time(text, seconds.get()),
               move |time| min.get().is_none_or(|min| key(time) >= key(min)) && max.get().is_none_or(|max| key(time) <= key(max)),
           ).check(TypedCheck::WhileTyping))
       }
       ```
     - `fn time_text` を消し、呼出しを `state.editor(move |time| locale.get().format_time(time, TimeStyle { seconds: seconds.get() }))` にする。
     - `into_view`：`return_focus` の memo → `state.picker().return_focus(button_ref)`。`open` の `binding(move || state.get().is_open(), ..)` → `state.picker().open(DatePanel::Time, button_ref)`。`state.send(Event::Open(input_ref))` → `state.picker().show(DatePanel::Time, input_ref)`。`.invalid(move || matches!(state.get().draft(), TimeDraft::Invalid(_)))` → `.invalid(move || state.is_invalid().get())`。`on_commit`（U23 後の名前）の `state.send(Event::Commit(text))` → `state.commit(text)`。Done の `state.send(Event::Close)` → `state.picker().hide()`。U08 後の External の `feed` の行は消える（`time_controller` の中にあった）。
     - 使わなくなった import は手順 10 で消す。
  6. date・time の組（`date_time_input`）：
     - `crates/kivact-ui/src/date_time_input/model.rs`：`Panel`、`DateTimeInputState`、`Event`、`Controller` と `impl Machine for Controller` を消し、import から `Machine`・`Step` を消す。module doc を `//! Role: Project a date-time value into its date and time fields.` にする。`key`・`DateTimeFields`・`field_bindings` は残す（`binding` の lens で、入力で動く machine でも draft の状態でもない）。
     - `crates/kivact-ui/src/date_time_input.rs`：`use crate::date_time_input::model::{Controller, DateTimeFields, DateTimeInputState, Event, Panel, field_bindings};`（U14 後の形）から `Controller`・`DateTimeInputState`・`Event`・`Panel` を消し、`use kivact_interaction::{DatePanel, DatePicker, DatePickerApi, use_date_picker};` を足す。`fn picker_controller` を次にする：
       ```rust
       /// The picker's open state and panel, opened from the dialog's trigger.
       fn picker_controller() -> (DatePickerApi, Signal<bool>) {
           let picker = use_date_picker();
           let invoker = kivact_core::view::node_ref();
           (picker, picker.open(DatePanel::Date, invoker))
       }
       ```
       `let (state, open) = picker_controller();` → `let (picker, open) = picker_controller();`。`kivact_core::state::MachineHandle<Controller>` の引数（`picker_tabs`、`desktop_content`）をすべて `picker: DatePickerApi` にする。`state.send(Event::Switch(Panel::Date))` → `picker.show_panel(DatePanel::Date)`、`Panel::Time` も同じ。Done の `state.send(Event::Close)` → `picker.hide()`。`dynamic` の `move || state.get()` → `move || picker.state().get()`、その match の `DateTimeInputState::Open { panel: Panel::Date, .. }` → `DatePicker::Open { panel: DatePanel::Date, .. }`、`Panel::Time` も同じ、`DateTimeInputState::Closed` → `DatePicker::Closed`。
     - `crates/kivact-ui/src/date_time_input/sheet.rs`：`use crate::date_time_input::model::{Controller, DateTimeInputState, Event, Panel};`（U14 後の形）を `use kivact_interaction::{DatePanel, DatePicker, DatePickerApi, use_date_picker};` にする。`render` の `let state = machine(DateTimeInputState::Closed, || Controller, |_, ()| {});` から `let open = binding(..);` までを `let picker = use_date_picker(); let open = picker.open(DatePanel::Date, button_ref);` にし、`let restore = memo(..);` を `let restore = picker.return_focus(button_ref);` にする。`editors(..)`・`picker_content(..)` に渡す `state` を `picker` にし、両関数の引数 `state: kivact_core::state::MachineHandle<Controller>` を `picker: DatePickerApi` にする。`state.send(Event::Open(Panel::Date, date_ref))` → `picker.show(DatePanel::Date, date_ref)`、`state.send(Event::Open(Panel::Time, time_ref))` → `picker.show(DatePanel::Time, time_ref)`、`state.send(Event::Switch(Panel::Time))` → `picker.show_panel(DatePanel::Time)`、`state.send(Event::Close)` → `picker.hide()`、`move || state.get()` の match は上の `date_time_input.rs` と同じに直す。
  7. range（`date_range_input`）：
     - `crates/kivact-ui/src/date_range_input/draft.rs` を次に置き換える（machine を消し、draft の値だけを残す）：
       ```rust
       //! Role: The draft of a date range: its two endpoints and their fields' validity.
       use kivact_core::i18n::{CalendarDate, DateRange};
       #[derive(Clone, Copy, PartialEq)]
       pub(super) enum FieldValidity { Valid, StartInvalid, EndInvalid, BothInvalid }
       /// The endpoints the two fields published and whether each field holds valid text.
       #[derive(Clone, Copy, PartialEq)]
       pub(super) struct RangeFields { pub(super) start: Option<CalendarDate>, pub(super) end: Option<CalendarDate>, pub(super) fields: FieldValidity }
       impl RangeFields {
           pub(super) fn following(range: &Option<DateRange>) -> Self { Self { start: range.map(DateRange::start), end: range.map(DateRange::end), fields: FieldValidity::Valid } }
           pub(super) fn with_start(self, start: Option<CalendarDate>) -> Self { Self { start, fields: field_validity(self.fields, true, true), ..self } }
           pub(super) fn with_end(self, end: Option<CalendarDate>) -> Self { Self { end, fields: field_validity(self.fields, false, true), ..self } }
           pub(super) fn start_valid(self, valid: bool) -> Self { Self { fields: field_validity(self.fields, true, valid), ..self } }
           pub(super) fn end_valid(self, valid: bool) -> Self { Self { fields: field_validity(self.fields, false, valid), ..self } }
           pub(super) fn order_invalid(self) -> bool { self.fields == FieldValidity::Valid && matches!((self.start, self.end), (Some(start), Some(end)) if start > end) }
           /// The range these fields stand for, `None` when they stand for none.
           pub(super) fn candidate(self) -> Option<Option<DateRange>> {
               if self.fields != FieldValidity::Valid { return None; }
               match (self.start, self.end) {
                   (None, None) => Some(None),
                   (Some(start), Some(end)) if start <= end => Some(Some(DateRange::new(start, end))),
                   _ => None,
               }
           }
       }
       fn field_validity(state: FieldValidity, start: bool, valid: bool) -> FieldValidity {
           /* 旧 `fn field_validity` の本体をそのまま */
       }
       ```
     - `crates/kivact-ui/src/date_range_input.rs`：import の `use crate::date_input::OpenedBy;` を消し、旧 draft の型の import（`Effect as DraftEffect, Event as DraftEvent, FieldValidity, Policy as DraftPolicy, RangeDraft`）を `use crate::date_range_input::draft::{FieldValidity, RangeFields};`（U14 後の module の path）にし、`use kivact_interaction::{DraftApi, DraftCommit, DraftOptions, use_draft};` を足す。`fn range_machine` を消して次を置く：
       ```rust
       /// The range draft: the two endpoint fields write the range once they stand for one.
       fn range_draft(value: Signal<Option<DateRange>>, valid: Option<Signal<bool>>, accept: Rc<dyn Fn(DateRange) -> bool>) -> DraftApi<Option<DateRange>, RangeFields> {
           use_draft(value, DraftOptions::new(RangeFields::following, move |fields: &RangeFields| {
               fields.candidate().filter(|range| range.is_none_or(|range| accept(range)))
           })
           .commit(DraftCommit::Live)
           .valid(valid))
       }
       ```
       U08 後の `draft.feed(value, DraftEvent::External);` の行は `range_machine` と一緒に消える（`use_draft` が feed する）。`fn range_open` を消す。`render`：`let draft = range_machine(self.value, self.valid, accept.clone());` → `let draft = range_draft(self.value, self.valid, accept.clone());`、`let open = range_open(draft);` → `let open = kivact_core::state::signal(false);`。`self.picker_content(draft, ..)` の `draft` を `open` にする。
     - `picker_content` と `committing_range` の引数 `draft: MachineHandle<DraftPolicy>` を `open: Signal<bool>` にし、本体の `draft.send(DraftEvent::Close);`（preset の `on_press`、`committing_range`、Done の 3 か所）を `open.set(false);` にする。
     - `range_editors(draft: DraftApi<Option<DateRange>, RangeFields>)` にし、本体を次にする：
       ```rust
       let start = binding(move || draft.draft().get().start, move |value| draft.update(|fields| *fields = fields.with_start(value)));
       let end = binding(move || draft.draft().get().end, move |value| draft.update(|fields| *fields = fields.with_end(value)));
       let start_valid = binding(
           move || matches!(draft.draft().get().fields, FieldValidity::Valid | FieldValidity::EndInvalid),
           move |valid| draft.revise(|fields| *fields = fields.start_valid(valid)),
       );
       let end_valid = binding(
           move || matches!(draft.draft().get().fields, FieldValidity::Valid | FieldValidity::StartInvalid),
           move |valid| draft.revise(|fields| *fields = fields.end_valid(valid)),
       );
       RangeEditors { draft, start, end, start_valid, end_valid }
       ```
     - `struct RangeEditors` の `draft` の型と `order_diagnostic` の引数の型を `DraftApi<Option<DateRange>, RangeFields>` にする。`endpoint_fields` の `.invalid(move || draft.get().order_invalid())`（2 か所）と `order_diagnostic` の `.hidden(move || !draft.get().order_invalid())` の `draft.get()` を `draft.draft().get()` にする。
  8. `crates/kivact-ui/src/pagination/draft.rs`：`PageDraft`、`Event`、`Snapshot`、`impl Machine for Snapshot` を消し、`pub(super) fn input` の本体を次にする（署名は W7b の始めの形のまま。`page` は `Signal<usize>`）：
     ```rust
     let page_text = |page: &usize| page.saturating_add(1).min(u32::MAX as usize).to_string();
     let draft = use_draft(page, DraftOptions::new(page_text, move |text: &String| {
         let current = page.get_untracked();
         Some(text.parse::<usize>().unwrap_or(current.saturating_add(1)).saturating_sub(1).min(count.get_untracked().saturating_sub(1)))
     }));
     let reference = node_ref();
     draft.settle_on_blur(reference.focused());
     crate::text_input(draft.editor())
         .label(label)
         .node_ref(reference)
         .disabled(disabled)
         .on_key(move |event| if event.key() == Key::Enter { draft.commit(); Handled::Yes } else { Handled::No })
         .into_view()
     ```
     import を `use kivact_core::state::{Signal, Value};`、`use kivact_core::view::{Element, Handled, IntoView, Key, View, node_ref};`、`use kivact_interaction::{DraftOptions, use_draft};` にする。module doc は変えない。
  9. `crates/kivact-ui/src/number_input.rs`：`mod draft;` を消し、`crates/kivact-ui/src/number_input/draft.rs`（`NumberText`、`NumberEvent`、`NumberInputSnapshot`、`impl Machine`）を消す。`into_view` の `let typed = machine(..)` から `kivact_core::state::effect_on(focused, ..)` の文までのうち、`let input_ref = node_ref();` 以外の行（`machine(..)`、`let focused = ..`、`typed.feed(..)`、`let edit = binding(..)`、`effect_on(..)`）を消し、`let input_ref = node_ref();` の前に次を置く：
     ```rust
     // Scalar owns the numeric write; snapshot its accepted value for the edit phases.
     let writer = binding(shown, move |next: f64| {
         number.once(|| number.set(next));
     });
     let draft = use_draft(writer, DraftOptions::new(
         move |value: &f64| format_value(*value, range.get(), &locale.get()),
         move |text: &String| locale.get().parse_number(text),
     ).commit(DraftCommit::Live));
     ```
     `let input_ref = node_ref();` の直後に `draft.settle_on_blur(input_ref.focused());` を置く。field の `text_input(edit)`（U22 後の形。`.label(..)` はそのまま）の `edit` を `draft.editor()` にし、`.invalid(..)` を次にする：
     ```rust
     .invalid(move || {
         invalid.get() || (draft.is_editing().get() && text_invalid(&draft.draft().get(), range.get(), &locale.get()))
     })
     ```
     `fn text_invalid(text: &NumberText, ..)` を次にする：
     ```rust
     fn text_invalid(text: &str, range: ScalarRange, locale: &Locale) -> bool {
         locale.parse_number(text).is_none_or(|value| !value.is_finite() || value < range.min() || value > range.max())
     }
     ```
     import に `use kivact_interaction::{DraftCommit, DraftOptions, use_draft};` を足す。`use_scalar(value, range).disabled(disabled)` の行は変えない（W8 の U42 が直す）。
  10. 使わなくなった import を消す：手順 4〜9 で変えた ui の file について `cargo clippy -p kivact-ui --all-targets -- -D warnings` を走らせ、`unused_imports` が指す名前だけを `use` の行から消す（この unit で使わなくなるのは `machine`、`memo`、`binding`、`MachineHandle`、`Machine`、`Step`、`effect_on` のどれか。それ以外の警告が出たら手順の写し間違いなので、その手順を見直す）。
- **消すもの**：ui の `date_input/machine.rs`（`OpenedBy`、`DateValidation`、`TypedDate`、`Picker`、`DateField`、`Snapshot`）、`time_input/machine.rs`（`TimeDraft`、`TimeInputState`、`Snapshot`）、`date_time_input/model.rs` の `Panel`・`DateTimeInputState`・`Event`・`Controller`、`date_range_input/draft.rs` の `Draft`・`RangeDraft`・`Event`・`Effect`・`Policy`、`pagination/draft.rs` の `PageDraft`・`Event`・`Snapshot`、`number_input/draft.rs`（`NumberText`、`NumberEvent`、`NumberInputSnapshot`）、`fn picker_open`、`fn draft_binding`、`fn time_text`、`fn range_machine`、`fn range_open`、`fn checked_invalid`。
- **状態の閉じ方**：`DraftOptions`・`DateFieldOptions`（Options）、`DraftApi`・`DateFieldApi`・`DatePickerApi`（Api）は名前で対象外。`State<T, D>`、`TypedText`、`TextCheck`、`TypedCheck`、`DatePicker`、`DatePanel`、`DraftCommit` は enum。`State::Editing` は Option を一つだけ持つ。snapshot の `Machine<T, D>`・`machine::Machine<T>` は bool・Option を持たない。`RangeFields`（Option 2 つ）は値渡しの method だけで、`&mut self` の method も内部可変も持たないので対象外（`update`・`revise` の closure は局所の値を書く）。印は要らない。
- **試験**：
  - 追加 `interaction_edit.rs`（import に `use kivact::interaction::{DraftCommit, DraftOptions, use_draft};` を足す。`use crate::support::mount;` が無ければ足す）：
    ```rust
    #[test]
    fn an_explicit_draft_writes_on_commit_follows_external_writes_and_cancels() {
        let (screen, (page, shown, editing, valid, draft)) = mount(|| {
            let page = signal(3usize);
            let draft = use_draft(page, DraftOptions::new(|page: &usize| page.to_string(), |text: &String| text.trim().parse::<usize>().ok()));
            let shown = draft.draft();
            ((page, shown, draft.is_editing(), draft.is_valid(), draft), text(move || shown.get()))
        });
        assert_eq!(shown.get_untracked(), "3");
        draft.edit("7".to_owned());
        assert_eq!(page.get_untracked(), 3, "an explicit draft writes nothing while it is edited");
        assert!(editing.get_untracked());
        draft.commit();
        assert_eq!(page.get_untracked(), 7);
        assert!(!editing.get_untracked());
        draft.edit("x".to_owned());
        assert!(!valid.get_untracked());
        draft.commit();
        assert_eq!(page.get_untracked(), 7, "a rejected commit writes nothing");
        assert!(editing.get_untracked(), "a rejected draft stays");
        page.set(9);
        screen.run_until_idle();
        assert_eq!(shown.get_untracked(), "9", "an external write replaces an unrelated draft");
        draft.edit("4".to_owned());
        draft.cancel();
        assert_eq!(shown.get_untracked(), "9");
        assert_eq!(page.get_untracked(), 9);
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }

    #[test]
    fn a_live_draft_writes_each_accepted_edit_and_keeps_the_text_the_writer_clamped() {
        let (screen, (stored, shown, valid, draft)) = mount(|| {
            let stored = signal(5.0f64);
            let clamped = binding(stored, move |next: f64| stored.set(next.min(10.0)));
            let draft = use_draft(clamped, DraftOptions::new(|value: &f64| value.to_string(), |text: &String| text.trim().parse::<f64>().ok()).commit(DraftCommit::Live));
            let shown = draft.draft();
            ((stored, shown, draft.is_valid(), draft), text(move || shown.get()))
        });
        draft.edit("12".to_owned());
        screen.run_until_idle();
        assert_eq!(stored.get_untracked(), 10.0);
        assert_eq!(shown.get_untracked(), "12", "the typed text stays over the value it wrote");
        draft.commit();
        assert_eq!(shown.get_untracked(), "10");
        draft.edit("x".to_owned());
        screen.run_until_idle();
        assert!(!valid.get_untracked());
        assert_eq!(stored.get_untracked(), 10.0);
        stored.set(3.0);
        screen.run_until_idle();
        assert_eq!(shown.get_untracked(), "3", "an external write replaces a draft that stands for no value");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
    ```
  - 変えずに通す：`ui_calendar.rs`（`date_input_parses_typed_text_and_opens_the_calendar`、`date_input_bottom_sheet_picks_with_the_wheel`、`time_input_accepts_hours_and_minutes`、`date_fields_attach_to_existing_buttons_without_constructing_editors`、`date_range_input_keeps_invalid_order_as_a_draft`）、`apg_date_picker.rs` の全試験、`ui_navigation.rs::pagination_input_commits_and_clamps_the_one_based_draft`、`ui_navigation.rs::pagination_moves_pages_and_keeps_focus`、`apg_spinbutton.rs` の全試験、`interaction_edit.rs::every_value_writing_standard_component_reports_on_edit`。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction -p kivact-ui -p kivact-public-api-tests
  cargo check -p kivact-interaction -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo xtask check --no-write --crate kivact-ui
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_edit ui_calendar apg_date_picker ui_navigation apg_spinbutton
  ```
- **完了条件**：`rg -n "impl.*Machine for" crates/kivact-ui/src/date_input.rs crates/kivact-ui/src/date_input crates/kivact-ui/src/time_input.rs crates/kivact-ui/src/time_input crates/kivact-ui/src/date_time_input.rs crates/kivact-ui/src/date_time_input crates/kivact-ui/src/date_range_input.rs crates/kivact-ui/src/date_range_input crates/kivact-ui/src/pagination crates/kivact-ui/src/number_input.rs crates/kivact-ui/src/number_input` が空。`rg -nw "OpenedBy|TypedDate|TimeDraft|RangeDraft|PageDraft|NumberText" crates/kivact-ui/src` が空。`rg -n "\beffect(_on)?\(" crates/kivact-ui/src/number_input.rs crates/kivact-ui/src/date_range_input.rs crates/kivact-ui/src/date_range_input crates/kivact-ui/src/time_input` が空。`ls crates/kivact-ui/src/date_input/machine.rs crates/kivact-ui/src/time_input/machine.rs crates/kivact-ui/src/number_input/draft.rs` がどれも「無い」と答える。上の命令がすべて通る。

---

## U35：UI-04 `use_history`、calendar の month swipe、date sheet の panel
- **wave / lane / 前提**：W7b / U3 / U34（同じ lane の前の unit）。前提にする状態：
  - W7a の全 unit が U20 → U21 → U25 → U22 → U23 → U24 の順に済んでいる（DEC-05 の `.label()` の呼出しの形、`on_commit`、`<Name>Theme`、theme 変数、`Element::variant`、`#[caller] caller: Caller`（`Node::authored` の snapshot）、`kivact::view::Children`）。
  - W5・W6 の API：`Signal<T>`（`Binding`・`IntoBinding` は無い）、`binding(get, set) -> Signal<T>`、`Callback<E>`／`callback(f)`、`use_direction() -> ReadingDirection`（`Value` で包まない）、`Theme::light()`、`theme.with(ext)`。payload のある handler は `impl Fn(E) -> R + 'static`（DEC-16）、`Signal::update` は `()` を返す（DEC-15）。
  - U08（W3）で calendar の `MonthEvent`・`MonthSnapshot`・`month_navigation` は消えている。`calendar/view.rs` の `into_view` の `base_month` と `month` は同じ signal である。
  - U14（W4）で `composer_history.rs` は `crates/kivact-ui/src/chat_composer/history.rs`、`date_sheet.rs` は `crates/kivact-ui/src/date_parts/sheet.rs` に移っている。W1 で `xtask-allow(state-fields)` の印はすべて消えている。
  - e2e の binary：`interaction_*`・`ui_*`・`apg_*`・`text_editing.rs` は `--test e2e_ui`。
- **所有**：新規 `crates/kivact-interaction/src/history.rs`、`crates/kivact-interaction/src/month_swipe.rs`。変更 `crates/kivact-interaction/src/lib.rs`（`mod`・`pub use` の行だけ）、`crates/kivact-ui/src/chat_composer.rs`、`crates/kivact-ui/src/chat_composer/history.rs`（削除）、`crates/kivact-ui/src/calendar/{navigation.rs,view.rs,panel.rs,state.rs}`、`crates/kivact-ui/src/date_parts/sheet.rs`。試験 `dev/tests/kivact-public-api/tests/e2e_ui/ui_calendar.rs`。data table（`data_table/`、`reorder_drag.rs`）はこの unit で触らない（W8 の U43 が行う）。
- **目的**：UI-04 規則 1 のうち composer の履歴と month の swipe を interaction の behavior（`use_history`、`use_month_swipe`）にし、date sheet の panel の machine を signal にする。
- **手順**：
  1. `crates/kivact-interaction/src/history.rs` を作る。ui の `chat_composer/history.rs` の `Recall`、`History`、`Event` と `impl Machine for Policy` の本体を移し、`Policy` を private な `struct Machine;` に改名する（`History`・`Event` も private にする）。file の全体：
     ```rust
     //! Role: Recall submitted entries in a text editor with Up and Down.
     use kivact_core::custom::{EditState, NodeRefExt, Part, PartState, part};
     use kivact_core::state::{IntoValue, Machine as MachineTrait, MachineHandle, Step, Value, machine};
     use kivact_core::view::{Handled, Key, KeyEvent, Modifiers, NodeRef};

     #[derive(Clone, PartialEq)]
     enum Recall {
         Draft,
         Entry { index: usize, draft: String },
     }
     #[derive(Clone, PartialEq)]
     struct History {
         entries: Vec<String>,
         recall: Recall,
     }
     enum Event {
         Submit(String),
         Recall { previous: bool, draft: String },
     }
     struct Machine;
     impl MachineTrait for Machine {
         type State = History;
         type Event = Event;
         type Effect = String;
         fn transition(&self, state: &History, event: Event) -> Step<History, Event, String> {
             /* ui の chat_composer/history.rs の `impl Machine for Policy` の `transition` の本体をそのまま移す */
         }
     }

     /// Submitted entries an editor recalls with Up and Down.
     #[derive(Clone, Copy)]
     pub struct HistoryApi {
         session: MachineHandle<Machine>,
         editor: NodeRef,
         enabled: Value<bool>,
     }

     /// The history of the text editor `editor`, active while `enabled`. Setup only.
     #[track_caller]
     pub fn use_history(editor: NodeRef, enabled: impl IntoValue<bool>) -> HistoryApi {
         let session = machine(
             History { entries: Vec::new(), recall: Recall::Draft },
             || Machine,
             move |_, value: String| {
                 let mut state = EditState::new(&value);
                 state.move_to(0, false);
                 state.move_to(value.len(), true);
                 editor.set_edit_state(state);
             },
         );
         HistoryApi { session, editor, enabled: enabled.into_value() }
     }

     impl HistoryApi {
         /// Record a submitted entry (while enabled) and go back to the draft.
         pub fn push(self, entry: impl Into<String>) {
             if self.enabled.get_untracked() {
                 self.session.send(Event::Submit(entry.into()));
             }
         }

         /// The editor's part: a plain Up with the caret at the start recalls the
         /// previous entry, a plain Down at the end the next one or the draft. Setup only.
         #[track_caller]
         pub fn input(self) -> Part<PartState> {
             let input = part(|input| input);
             Part::new(input.state(), move |host| {
                 host.part(input).on_key(move |event: KeyEvent| self.key(&event))
             })
         }

         fn key(self, event: &KeyEvent) -> Handled {
             if !self.enabled.get_untracked()
                 || event.modifiers() != Modifiers::default()
                 || !matches!(event.key(), Key::ArrowUp | Key::ArrowDown)
             {
                 return Handled::No;
             }
             let Some(editor) = self.editor.edit_state() else {
                 return Handled::No;
             };
             let selection = editor.selection();
             let at_start = selection.start == 0;
             let at_end = selection.end == editor.value().len();
             let eligible = match event.key() {
                 Key::ArrowUp => at_start && (selection.is_empty() || at_end),
                 Key::ArrowDown => at_end && (selection.is_empty() || at_start),
                 _ => false,
             };
             if !eligible {
                 return Handled::No;
             }
             self.session.send(Event::Recall {
                 previous: event.key() == Key::ArrowUp,
                 draft: editor.value().to_owned(),
             });
             Handled::Yes
         }
     }
     ```
     （`key` の本体は ui の `chat_composer.rs` の `composer_key` の `if !keep_history.get_untracked()` から最後の `Handled::Yes` までと同じ。）`crates/kivact-interaction/src/lib.rs` に `mod history;` と `pub use history::{HistoryApi, use_history};` を足す。
  2. `crates/kivact-ui/src/chat_composer.rs`：
     - `mod history;` の行を消し、`crates/kivact-ui/src/chat_composer/history.rs` を `git rm` する。import に `use kivact_interaction::{HistoryApi, use_history};` を足す。
     - `fn input_and_submission` の `let history = ..use_history(input_ref);`（U14 で移した history module の関数を呼ぶ行）を `let history = use_history(input_ref, self.history);` にし、`let keep_history = self.history;` の行を消す。text area の builder の `.node_ref(input_ref)` の後に `.with_node(|node| node.part(history.input()))` を足す。`.on_key(move |event| composer_key(event, input_ref, send, history, keep_history))` を `.on_key(move |event| composer_key(event, send))` にする。
     - `fn submission` の引数 `history` の型（`MachineHandle<..Policy>`）を `HistoryApi` にする。`let keep_history = self.history;` を消し、`if keep_history.get_untracked() { history.send(..Event::Submit(draft.to_owned())); }` の 3 行を `history.push(draft);` にする。
     - `fn composer_key` を次にする（引数 `input`・`history`・`keep_history` と本体の履歴の部分を消す）：
       ```rust
       fn composer_key(event: KeyEvent, send: Callback<()>) -> Handled {
           if event.key() == Key::Enter && event.modifiers() == Modifiers::default() {
               send.call(());
               return Handled::Yes;
           }
           Handled::No
       }
       ```
     - 使われなくなった import（`NodeRefExt` など。`cargo clippy` の `unused_imports` が示すもの）を消す。
  3. `crates/kivact-interaction/src/month_swipe.rs` を作る。ui の `calendar/navigation.rs` から次をそのまま移す：定数 `MONTH_SWIPE_SLOP`、`MONTH_TURN_DISTANCE`、`PROJECTION`、`MONTH_TRAVEL`、`MAX_MONTHS_PER_FLICK`、`MAX_SHIFT`、`EDGE_RESISTANCE`、`VELOCITY_WINDOW`、`ENTER_OFFSET`（doc comment ごと）、`enum Swipe`、`enum Event`、`struct SwipeSnapshot`、`impl SwipeSnapshot` の二つ（`sign`・`shift`・`months` と `pending_move`・`drag`）、`impl Machine for SwipeSnapshot`。移した型と field の `pub(super)` を外して private にする。`impl Swipe { fn shift, fn entering }` は移さない（下の `MonthSwipePhase` が置き換える）。file の残り：
     ```rust
     //! Role: Turn the shown month with a horizontal touch swipe within the date bounds.
     use crate::{DragAxis, DragVelocity, project_release};
     use kivact_core::custom::{Part, PartState, clock_now, part};
     use kivact_core::geometry::Point;
     use kivact_core::i18n::{CalendarDate, CalendarMonth};
     use kivact_core::state::{IntoValue, Machine, MachineHandle, Signal, Step, Value, machine, memo};
     use kivact_core::view::{A11yAction, Handled, PointerEvent, PointerId, PointerKind, PointerPhase, ReadingDirection, use_direction};
     use std::time::Duration;

     /// Where a month swipe is.
     #[derive(Clone, Copy, Debug, PartialEq)]
     pub enum MonthSwipePhase {
         Idle,
         /// The shown months follow the finger by `shift` px.
         Dragging { shift: f64 },
         /// A release turned `months` months (0 once they entered); the trailing activation is ignored.
         Released { months: i32 },
     }

     impl MonthSwipePhase {
         /// How far the shown months follow the finger now.
         pub fn shift(self) -> f64 {
             match self {
                 Self::Dragging { shift } => shift,
                 _ => 0.0,
             }
         }
         /// The horizontal offset the newly shown months enter from: the side the
         /// swipe brought them in from, or none.
         pub fn entering(self, rtl: bool) -> f64 {
             match self {
                 Self::Released { months } if months != 0 => {
                     f64::from(months.signum()) * if rtl { -1.0 } else { 1.0 } * ENTER_OFFSET
                 }
                 _ => 0.0,
             }
         }
     }

     /// A touch swipe on a calendar that turns its shown month.
     #[derive(Clone, Copy)]
     pub struct MonthSwipeApi {
         session: MachineHandle<SwipeSnapshot>,
         phase: Value<MonthSwipePhase>,
     }

     /// A touch swipe that turns `month` within `min` and `max`. Setup only.
     #[track_caller]
     pub fn use_month_swipe(
         month: Signal<CalendarMonth>,
         min: impl IntoValue<Option<CalendarDate>>,
         max: impl IntoValue<Option<CalendarDate>>,
     ) -> MonthSwipeApi {
         let (min, max) = (min.into_value(), max.into_value());
         let rtl = use_direction() == ReadingDirection::Rtl;
         let session = machine(
             Swipe::Idle,
             move || {
                 let current = month.get_untracked();
                 SwipeSnapshot {
                     rtl,
                     now: clock_now(),
                     at_min: min.get().is_some_and(|min| current <= min.month_of()),
                     at_max: max.get().is_some_and(|max| current >= max.month_of()),
                 }
             },
             move |_, months: i32| {
                 // A flick turns as many months as it projects, stopping at the
                 // bounds; it never moves the other way.
                 let current = month.get_untracked();
                 let next = current.add_months(months);
                 let (next, moves) = if months > 0 {
                     let next = max.get_untracked().map_or(next, |max| next.min(max.month_of()));
                     (next, next > current)
                 } else {
                     let next = min.get_untracked().map_or(next, |min| next.max(min.month_of()));
                     (next, next < current)
                 };
                 if moves {
                     month.set(next);
                 }
             },
         );
         let phase = memo(move || match session.get() {
             Swipe::Dragging { shift, .. } => MonthSwipePhase::Dragging { shift },
             Swipe::Released { months } => MonthSwipePhase::Released { months },
             Swipe::Idle | Swipe::Pending { .. } => MonthSwipePhase::Idle,
         });
         MonthSwipeApi { session, phase }
     }

     impl MonthSwipeApi {
         /// Where the swipe is.
         pub fn phase(self) -> Value<MonthSwipePhase> {
             self.phase
         }
         /// The months a swipe turned to are mounted.
         pub fn entered(self) {
             self.session.send(Event::Entered);
         }
         /// The calendar's root: it captures the touch while dragging. Setup only.
         #[track_caller]
         pub fn area(self) -> Part<PartState> {
             let input = part(|input| input);
             let session = self.session;
             Part::new(input.state(), move |host| {
                 host.part(input).on_pointer(move |event: PointerEvent| {
                     session.send(Event::Pointer(event));
                     if matches!(session.get(), Swipe::Dragging { .. }) {
                         event.capture();
                         Handled::Yes
                     } else {
                         Handled::No
                     }
                 })
             })
         }
         /// A day cell: its pointer feeds the swipe, and a key or the Activate
         /// action is an independent activation. Setup only.
         #[track_caller]
         pub fn day(self) -> Part<PartState> {
             let input = part(|input| input);
             let session = self.session;
             Part::new(input.state(), move |host| {
                 host.part(input)
                     .on_pointer(move |event: PointerEvent| {
                         let dragging = matches!(session.get(), Swipe::Dragging { .. });
                         session.send(Event::Pointer(event));
                         if matches!(session.get(), Swipe::Dragging { .. }) {
                             event.capture();
                             Handled::Yes
                         } else if dragging {
                             Handled::Yes
                         } else {
                             Handled::No
                         }
                     })
                     .on_key(move |_| {
                         session.send(Event::IndependentActivation);
                         Handled::No
                     })
                     .a11y_action(A11yAction::Activate, move |_| {
                         session.send(Event::IndependentActivation);
                         Handled::No
                     })
             })
         }
     }
     ```
     （`area` の本体は ui `calendar/view.rs` の root の `.on_pointer(..)`、`day` の本体は ui `calendar/panel.rs` の `fn interactive_day` の 3 つの handler と同じ。`SwipeSnapshot` の `rtl` は setup で一度読む：W5 の後の `use_direction()` は値を返す。上の import はすべて、移した定数・型・関数かこの file の本体が使う。）`crates/kivact-interaction/src/lib.rs` に `mod month_swipe;` と `pub use month_swipe::{MonthSwipeApi, MonthSwipePhase, use_month_swipe};` を足す。
  4. `crates/kivact-ui/src/calendar/navigation.rs`：`InitialFocus`、`FocusSnapshot`、`impl Machine for FocusSnapshot` だけを残し、手順 3 で移したものを消す。先頭を次にする（mount で一度だけ初期 focus を当てる印で、入力では動かないので ui に残す）：
     ```rust
     //! Role: The calendar's one-shot initial focus.
     use kivact_core::state::{Machine, Step};
     ```
  5. `crates/kivact-ui/src/calendar/view.rs`：
     - import に `use kivact_interaction::{MonthSwipeApi, MonthSwipePhase, use_month_swipe};` を足す。
     - `fn swipe_navigation` を消す。`into_view` の `let swipe = swipe_navigation(base_month, month, self.min, self.max);` を `let swipe = use_month_swipe(base_month, self.min, self.max);` にする。
     - root の `.on_pointer(move |event| { swipe.send(super::navigation::Event::Pointer(event)); .. })` の呼出し全体を `.part(swipe.area())` にする。
     - `fn month_panels` の `swipe.state().get_untracked().entering(..)` を `swipe.phase().get_untracked().entering(..)`（引数はそのまま）に、`on_mount` の中の `swipe.send(super::navigation::Event::Entered);` を `swipe.entered();` にする。
     - `fn calendar_body` の引数 `swipe: kivact_core::state::MachineHandle<super::navigation::SwipeSnapshot>` を `swipe: MonthSwipeApi` にし、activation guard の `!matches!(swipe.get(), super::navigation::Swipe::Released { .. })` を `!matches!(swipe.phase().get(), MonthSwipePhase::Released { .. })` にする。
  6. `crates/kivact-ui/src/calendar/state.rs`：`Shared::swipe` の型を `kivact_interaction::MonthSwipeApi` にする。
  7. `crates/kivact-ui/src/calendar/panel.rs`：
     - import に `use kivact_interaction::{MonthSwipeApi, MonthSwipePhase};` を足す。
     - `.style(move || week_motion(swipe.get(), entering))` を `.style(move || week_motion(swipe.phase().get(), entering))` にする。
     - `fn week_motion(swipe: super::navigation::Swipe, entering: f64)` を `fn week_motion(swipe: MonthSwipePhase, entering: f64)` にし、本体の `matches!(swipe, super::navigation::Swipe::Dragging { .. })` を `matches!(swipe, MonthSwipePhase::Dragging { .. })` にする（`let shift = swipe.shift();` はそのまま：`MonthSwipePhase::shift`）。
     - `fn interactive_day` を次にする：
       ```rust
       fn interactive_day(swipe: MonthSwipeApi) -> Node {
           node().part(swipe.day())
       }
       ```
  8. `crates/kivact-ui/src/date_parts/sheet.rs`：`enum Event`、`struct Controller`、`impl Machine for Controller` を消す。`let state = machine(DateSheetPanel::Calendar, || Controller, |_, ()| {});` を `let panel = signal(DateSheetPanel::Calendar);` にし、`state.send(Event::MonthYear)` を `panel.set(DateSheetPanel::MonthYear)` に、`state.send(Event::Calendar)` を `panel.set(DateSheetPanel::Calendar)` に、`state.get()`（2 か所）を `panel.get()` にする。import の `kivact_core::state::{..}` から `Machine`・`Step`・`machine` を消して `signal` を足す。
- **消すもの**：ui の `chat_composer/history.rs`（`Recall`、`History`、`Event`、`Policy`）、calendar の ui 側の `Swipe`・`Event`・`SwipeSnapshot`・`impl Swipe`・swipe の定数・`fn swipe_navigation`、date sheet の `Event`・`Controller`。
- **状態の閉じ方**：`HistoryApi`・`MonthSwipeApi`（Api）、`MonthSwipePhase`（Phase）は名前で対象外。`SwipeSnapshot`（bool 3 つと u64）は constructor でだけ作る snapshot で、`&mut self` の method も内部可変も持たないので対象外。`History`（`Vec` と enum）は bool・Option・counter を持たない。`Recall`・`Swipe` は enum。印は要らない。
- **試験**：
  - 追加 `dev/tests/kivact-public-api/tests/e2e_ui/ui_calendar.rs`（import に `use kivact::view::PointerKind;` と `use std::time::Duration;` を足す。`CalendarDate` は既に import にある）：
    ```rust
    #[test]
    fn a_touch_swipe_turns_the_month_within_the_bounds_without_selecting_the_contact_day() {
        let start = CalendarDate::new(2026, 10, 14);
        let bound = CalendarDate::new(2026, 11, 30);
        let (screen, selected) = mount(move || {
            let selected = signal(Some(start));
            (selected, kivact::ui::calendar(selected).min(Some(start)).max(Some(bound)))
        });
        let swipe = |name: &str| {
            let bounds = screen.get_by_role(Role::GridCell, name).accessibility_bounds().unwrap();
            let (x, y) = (bounds.x + bounds.width / 2.0, bounds.y + bounds.height / 2.0);
            let finger = screen.contact_down(PointerKind::Touch, x, y);
            screen.contact_move(finger, x - 48.0, y + 24.0);
            screen.contact_up(finger);
            screen.advance(Duration::from_millis(500));
        };
        swipe("2026-10-20");
        assert_eq!(selected.get_untracked(), Some(start), "the contact day is not selected");
        swipe("2026-11-20");
        assert!(screen.query_by_role(Role::GridCell, "2026-12-15").is_none(), "the bound stops the swipe");
        screen.get_by_role(Role::GridCell, "2026-11-15").tap();
        assert_eq!(selected.get_untracked(), Some(CalendarDate::new(2026, 11, 15)));
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
    ```
  - 変えずに通す：`ui_chat_code.rs::chat_composer_preserves_newlines_history_stop_and_ime`、`ui_calendar.rs`・`apg_date_picker.rs`・`interaction_calendar.rs` の既存の全試験。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction -p kivact-ui -p kivact-public-api-tests
  cargo check -p kivact-interaction -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo xtask check --no-write --crate kivact-ui
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_chat_code ui_calendar apg_date_picker interaction_calendar
  ```
- **完了条件**：`rg -n "impl Machine for" crates/kivact-ui/src/calendar crates/kivact-ui/src/date_parts crates/kivact-ui/src/chat_composer.rs` が `calendar/navigation.rs` の `impl Machine for FocusSnapshot` の 1 行だけを出す。`rg -nw "SwipeSnapshot|swipe_navigation|Controller" crates/kivact-ui/src/calendar crates/kivact-ui/src/date_parts/sheet.rs` が空。`crates/kivact-ui/src/chat_composer/history.rs` が無い。命令が通る。

---

## U36：UI-17 text 編集の key（行の Home・End、単語の移動と削除）
- **wave / lane / 前提**：W7b / U4 / W7a の全 unit（U20 → U21 → U25 → U22 → U23 → U24）。前提にする状態：
  - annex E の E17（W5a の鎖）の後の editing の API：`text_edit_policy(state: &mut EditState, request: EditRequest, context: &EditContext) -> EditOutcome`（寿命の引数は無い）、`EditOutcome::{NotEditing, Done, Copy(String), Paste}`（`Done` は field を持たない。`text_edit_policy` の `match key` の腕は今と同じく「変えたか」の `bool` を返し、最後の 1 行が `Done` にする）。`EditContext` の `line_count`・`line_of`・`line_range`・`x_of`・`offset_at`（引数は `(state, ..)`）。上下の key の goal column（`vertical_key`・`vertical_target`、`EditState::goal_x`・`EditState::move_vertically`）は E17 が `text_edit/policy.rs` に入れ済みで、`move_to` と text の変更が goal を消す。E17 は試験 `text_editing.rs::up_and_down_keep_their_column_across_a_short_line` も足している。
  - annex E の E20（W5a の鎖、E18 の後）：`EditState::word_at(&self, at: usize) -> Range<usize>`（UAX #29 の単語の区間。空白の連なりは一つの区間）と `EditState::line_at(at, multiline)`。目標の列は E17 の `EditState::goal_x()` と `EditState::move_vertically(to, extend, goal_x)` だけが持ち、他の移動と編集で消える（E17）。`set_goal_x` は無い。この unit は `move_vertically` と `goal_x()` を E17 の形のまま使い、目標の列を自分で書かない。
  - `kivact_core::platform::{CommandKey, capabilities}`（`capabilities() -> Value<Capabilities>`、`Capabilities::command_key`）。`kivact::test` の platform の `command_key` は既定の `CommandKey::Ctrl`（OS に依らない）。
  - W5・W6 の API（`Signal<T>`、`binding(get, set) -> Signal<T>`）。e2e の `text_editing.rs`・`custom_text_field.rs` は `--test e2e_core`、`interaction_edit.rs`・`ui_chat_code.rs` は `--test e2e_ui`。
- **所有**：`crates/kivact-interaction/src/text_edit/policy.rs`、試験 `dev/tests/kivact-public-api/tests/e2e_core/text_editing.rs`。scalar・splitter・ui の `slider.rs`・`splitter.rs`・`number_input.rs` は触らない（UI-11 の scalar の部分は W8 の U42）。
- **目的**：UI-17：複数行の Home/End は行の端、Ctrl/Cmd+Home/End は text の端、Ctrl（command key が ⌘ の platform では Alt）+左右・Backspace・Delete は単語単位で動かし消す。上下の移動の目標の列（E17）は横の移動で消えることを試験で固める。
- **手順**：
  1. `text_edit/policy.rs` の import：`use kivact_core::view::Key;` を `use kivact_core::view::{Key, Modifiers};` にし、`use kivact_core::platform::{CommandKey, capabilities};` を足す。
  2. `text_edit_policy` の `if multiline && !command && matches!(key, Key::ArrowUp | Key::ArrowDown)` を `if multiline && !command && !modifiers.alt && matches!(key, Key::ArrowUp | Key::ArrowDown)` にする。その if の直後（`let sel = state.selection();` の前）に次の 1 行を置く：
     ```rust
     let word = matches!(key, Key::ArrowLeft | Key::ArrowRight | Key::Backspace | Key::Delete) && word_chord(modifiers);
     ```
  3. `match key` の `Key::Backspace | Key::Delete`、`Key::ArrowLeft | Key::ArrowRight`、`Key::Home`、`Key::End` の 4 つの腕を次にする（他の腕は変えない）：
     ```rust
     Key::Backspace | Key::Delete => {
         let r = if !sel.is_empty() {
             sel.clone()
         } else if key == Key::Backspace {
             (if word { word_start(state, caret) } else { state.prev_grapheme(caret) })..caret
         } else {
             caret..(if word { word_end(state, caret) } else { state.next_grapheme(caret) })
         };
         if r.is_empty() {
             false
         } else {
             state.replace(r, "");
             true
         }
     }
     Key::ArrowLeft | Key::ArrowRight => {
         let left = key == Key::ArrowLeft;
         let to = if word {
             if left { word_start(state, caret) } else { word_end(state, caret) }
         } else {
             match (left, sel.is_empty() || shift) {
                 (true, true) => state.prev_grapheme(caret),
                 (true, false) => sel.start,
                 (false, true) => state.next_grapheme(caret),
                 (false, false) => sel.end,
             }
         };
         state.move_to(to, shift);
         false
     }
     Key::Home => {
         state.move_to(line_edge(state, context, command, true), shift);
         false
     }
     Key::End => {
         state.move_to(line_edge(state, context, command, false), shift);
         false
     }
     ```
  4. `fn command_key` の後に次の 4 つの関数を足す（`vertical_key`・`vertical_target` は E17 の形のまま変えない）：
     ```rust
     /// Where Home (`start`) or End moves the caret: the text's end with the
     /// command key, in one line or without layout; else the caret's line's end.
     fn line_edge(state: &EditState, context: &EditContext, command: bool, start: bool) -> usize {
         let whole = if start { 0 } else { state.value().len() };
         if command || !context.multiline {
             return whole;
         }
         context
             .line_of(state, state.caret())
             .and_then(|line| context.line_range(state, line))
             .map_or(whole, |range| if start { range.start } else { range.end })
     }

     /// Whether the modifiers are the platform's word chord: Alt where the command key is ⌘, else Ctrl.
     fn word_chord(modifiers: Modifiers) -> bool {
         if modifiers.meta {
             return false;
         }
         match capabilities().get_untracked().command_key {
             CommandKey::Command => modifiers.alt && !modifiers.ctrl,
             CommandKey::Ctrl => modifiers.ctrl && !modifiers.alt,
         }
     }

     /// The start of the word before `at`, over the whitespace before it.
     fn word_start(state: &EditState, at: usize) -> usize {
         let text = state.value();
         let mut to = at;
         while to > 0 {
             let prev = state.prev_grapheme(to);
             if !text[prev..to].chars().all(char::is_whitespace) {
                 return state.word_at(prev).start;
             }
             to = prev;
         }
         0
     }

     /// The end of the word after `at`, over the whitespace before it.
     fn word_end(state: &EditState, at: usize) -> usize {
         let text = state.value();
         let mut to = at;
         while to < text.len() {
             let next = state.next_grapheme(to);
             if !text[to..next].chars().all(char::is_whitespace) {
                 return state.word_at(to).end;
             }
             to = next;
         }
         text.len()
     }
     ```
  5. `text_edit_policy` の doc の 2 段落目を次にする：
     ```rust
     /// It moves and deletes by grapheme, selects with Shift, goes to the start
     /// and end of the line with Home and End in a multiline editor and of the
     /// text with Ctrl/Cmd+Home/End, moves and deletes by word with Ctrl (Alt
     /// where the command key is ⌘) and the arrows, Backspace and Delete, moves
     /// a line up or down with the arrow keys keeping the column of the first
     /// Up or Down across shorter lines, breaks lines with Enter in a multiline
     /// editor, and handles Ctrl/Cmd with A, C, X and V. A pasted
     ```
     （`A pasted` 以降の文は変えない。）
- **消すもの**：Home・End の「常に text の端」の腕（手順 3 で置き換える）。
- **状態の閉じ方**：新しい型は無い。goal column は annex E の `EditState` が持つ（E17 の印の対象）。
- **試験**：
  - 追加 `dev/tests/kivact-public-api/tests/e2e_core/text_editing.rs`（`fn ctrl()` の後に helper `word`・`word_shift` を足し、試験は E17 の `up_and_down_keep_their_column_across_a_short_line` の後に置く）：
    ```rust
    /// The word chord of the test platform: its command key is Ctrl on every OS.
    fn word() -> Modifiers {
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        }
    }

    fn word_shift() -> Modifiers {
        Modifiers {
            shift: true,
            ..word()
        }
    }

    #[test]
    fn home_and_end_go_to_the_ends_of_the_line_and_ctrl_to_the_ends_of_the_text() {
        let (screen, value, _log) = field_screen("abcd\nabcd\nabcd", true);
        let notes = screen.get_by_role(Role::TextBox, "Notes");
        notes.press_key_with(Key::End, ctrl());
        notes.press_key(Key::ArrowUp);
        notes.press_key(Key::Home);
        screen.type_text("X");
        assert_eq!(value.get(), "abcd\nXabcd\nabcd", "Home goes to the start of the line");
        notes.press_key(Key::End);
        screen.type_text("Y");
        assert_eq!(value.get(), "abcd\nXabcdY\nabcd", "End goes to the end of the line");
        notes.press_key_with(Key::Home, ctrl());
        screen.type_text("S");
        notes.press_key_with(Key::End, ctrl());
        screen.type_text("E");
        assert_eq!(value.get(), "Sabcd\nXabcdY\nabcdE", "Ctrl+Home and Ctrl+End go to the ends of the text");
        notes.press_key_with(Key::Home, shift());
        screen.type_text("Z");
        assert_eq!(value.get(), "Sabcd\nXabcdY\nZ", "Shift+Home selects to the start of the line");
    }

    #[test]
    fn up_and_down_keep_the_column_until_a_horizontal_move() {
        let (screen, value, _log) = field_screen("abcdef\nab\nabcdef", true);
        let notes = screen.get_by_role(Role::TextBox, "Notes");
        notes.press_key_with(Key::Home, ctrl());
        notes.press_key(Key::End);
        notes.press_key(Key::ArrowDown);
        notes.press_key(Key::ArrowDown);
        screen.type_text("X");
        assert_eq!(value.get(), "abcdef\nab\nabcdefX", "the column of the first line survives the short line");
        notes.press_key(Key::ArrowUp);
        notes.press_key(Key::ArrowLeft);
        notes.press_key(Key::ArrowUp);
        screen.type_text("Y");
        assert_eq!(value.get(), "aYbcdef\nab\nabcdefX", "a horizontal move ends the kept column");
    }

    #[test]
    fn the_word_chord_moves_and_deletes_by_word() {
        let (screen, value, _log) = field_screen("alpha beta gamma", false);
        let f = screen.get_by_role(Role::TextBox, "Name");
        f.press_key(Key::End);
        f.press_key_with(Key::ArrowLeft, word());
        f.type_text("X");
        assert_eq!(value.get_untracked(), "alpha beta Xgamma");
        f.press_key_with(Key::ArrowLeft, word());
        f.press_key_with(Key::ArrowLeft, word());
        f.type_text("Y");
        assert_eq!(value.get_untracked(), "alpha Ybeta Xgamma", "the word chord skips the space before a word");
        f.press_key_with(Key::ArrowRight, word());
        f.type_text("Z");
        assert_eq!(value.get_untracked(), "alpha YbetaZ Xgamma");
        f.press_key_with(Key::Backspace, word());
        assert_eq!(value.get_untracked(), "alpha  Xgamma");
        f.press_key_with(Key::Delete, word());
        assert_eq!(value.get_untracked(), "alpha ");
        f.press_key_with(Key::ArrowLeft, word_shift());
        f.type_text("Q");
        assert_eq!(value.get_untracked(), "Q", "Shift with the word chord selects by word");
    }
    ```
    （二つ目：`abcdef\nab\nabcdefX` で caret は 17。Up は列 7 の x を goal に 1 行目の末尾 9 へ、Left は 8（1 行目の列 1）へ動いて goal を消し、Up は列 1 の x で 0 行目の offset 1 へ行く。三つ目：`X` の後の caret 12 から二度の単語左は 11（`Xgamma` の頭）、次に 6（`beta` の頭）。）
  - 変えずに通す：`text_editing.rs` の既存の全試験（`up_and_down_move_the_caret_between_lines_of_a_text_area`、`caret_and_deletion_step_over_graphemes_and_selections_replace`、E17 の `up_and_down_keep_their_column_across_a_short_line` を含む）、`custom_text_field.rs` の全試験、`interaction_edit.rs::text_input_reports_edit_phases_from_the_native_editor`、`ui_chat_code.rs::chat_composer_preserves_newlines_history_stop_and_ime`。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction -p kivact-public-api-tests
  cargo check -p kivact-interaction -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- text_editing custom_text_field
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_edit ui_chat_code
  ```
- **完了条件**：`rg -n "^fn (line_edge|word_chord|word_start|word_end)\(" crates/kivact-interaction/src/text_edit/policy.rs` が 4 行を出す。`git diff --stat` に `text_edit/policy.rs` と `text_editing.rs` 以外の file が無い。命令が通る。

### 3.4 W8（lane U1〜U4）

## W8 共通：部品を `#[component]` の関数にする規則

W8 の unit（U40、U44〜U54）は、部品の builder struct と `impl Element for`・`impl IntoView for` を消し、`#[component]` の関数にする（設計書 5.1、DEC-03）。完全な例が U40（button）である。各 unit は U40 の形を、2.3 節の表の自分の行に当てはめる。

### 規則 C1：呼出しの形を変えない
W7a の U22・U23 で、利用者と ui の他の部品から見た呼出しの形はもう `#[component]` の形になっている。W8 の書き換えは、**位置引数の順と型、setter の名前、setter が受ける値の種類を変えない**。他の lane の file にある呼出し（例：`async_view` の中の `spinner()`）は直さなくても compile する。変えないと書けない場合は STOP する。

### 規則 C2：props の写し方
builder の constructor の位置引数は、同じ順で必須の props にする。builder の `pub fn x(mut self, v: ..) -> Self` は、名前 `x` の `Option<..>` の props にする（`#[component]` が同名の setter を作る）。型は次の表で決める。

| builder の引数の型 | props の型 |
|---|---|
| `impl IntoText`、`impl IntoValue<String>` | `Value<String>` |
| `impl IntoValue<T>` | `Value<T>` |
| `Signal<T>`（W5 の後の入力の値） | `Signal<T>` |
| `impl IntoView` | `View` |
| `impl IntoViewFn<V>`、`impl Fn() -> V`（view を作る、引数なし） | `ViewFn` |
| `impl IntoChildren` | `Vec<View>`。ただし 2.3 節の表で「子：遅延」とある部品は `Children`（4.2 節 R4） |
| `impl IntoHandler<(), M>`、`impl Fn() -> R`（payload の無い handler） | `Callback<()>`（setter は `impl IntoHandler<(), M>`） |
| `impl Fn(E) -> R`（`R: HandlerResult`、E は `()` 以外の payload） | `Callback<E>`（setter は `impl Fn(E) -> R + 'static`。共有した `Callback<E>` は `move \|e\| cb.call(e)` で渡す） |
| `impl Fn(A, ..) -> V`（`V: IntoView`、slot） | `Rc<dyn Fn(A, ..) -> View>`（4.2 節 R2） |
| `impl Fn(A, ..) -> T`（view でも handler でもない関数：書式、判定、比較） | `Rc<dyn Fn(A, ..) -> T>`（4.2 節 R2） |
| 型引数そのもの（`field` の `control: C`、`tooltip` の `trigger: C`、`async_view` の `source: S`） | `C`（U25 の Param の kind。`Into` で包まない） |
| `bool`、`f64`、`usize`、enum などの定数 | その型 `T`（`impl Into<T>` を受ける） |

既定値がある定数（今の builder が constructor で入れている値）は `#[prop(default = <その値>)]` にする。`Option<T>` の props は、呼ばれなかったときの既定を本体で `unwrap_or(..)` する（今の builder の constructor の値と同じにする）。

### 規則 C3：呼出し側の modifier を読む
今の `finish`・`into_view` が `self.root.accessible_*()`・`authored()` で読んでいた呼出し側の値は、関数の引数 `#[caller] caller: Caller`（`kivact_core::custom::Caller`。U25）から読む：`caller.variants()`、`caller.disabled()`、`caller.expanded()`、`caller.invalid()`、`caller.name()`、`caller.description()`、`caller.labelled_by()`、`caller.has_press()`。`Caller` は body の前に一度だけ作る snapshot で、body の後に読んでも他の部品の値にはならない。本体が root に付ける disabled（busy など）は、呼出し側の disabled と absorb で OR になる（U25）ので、本体は自分の値だけを付ける。

### 規則 C4：look と recipe
`with_recipe_style(node, |r: &XTheme| r.part_recipe(), &variants, state)` の形は変えない。`variants` は本体の先頭の `let variants = caller.variants();`（root と全部の part が同じ値を使う）。

### 規則 C5：消すもの
部品の builder struct、その `impl Element for`・`impl IntoView for`、`finish`、builder の setter。生成された struct の名前は関数名の UpperCamelCase（4.2 節 R1）で、今の builder の型名と同じになる（`Button`）。`lib.rs` の `pub use` の型名はそのまま（生成された struct を再公開する）。

例外：`tooltip` は `Tooltip<T>`（手書きの `Element`・`IntoView`）のまま残す（親の決定。surface を trigger の兄弟に置くため、一つの root を返す `#[component]` にできない）。

### 規則 C6：試験
W8 の unit は試験の期待値を変えない。`rg -n "\b<型名>\b" dev/tests/kivact-public-api/tests` で builder の型名を書いている試験（`let b: Button = ..` など）だけを、生成された struct の名前に合わせる（名前は同じなので通常は直す所が無い）。

### 規則 C7：確認（各 W8 unit の tier a。`<tests>` は 2.3 節の表の試験）
```
cargo fmt -p kivact-ui
cargo check -p kivact-ui --all-targets
cargo clippy -p kivact-ui --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-ui
cargo check -p kivact -p kivact-devtools -p kivact-public-api-tests --all-targets
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- <tests の e2e_ui の module>
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- <tests の e2e_core の module>
```
`<tests>` のうち `components`・`async_tests`・`environment`・`text_editing`・`i18n` は `e2e_core`、他は `e2e_ui` の module である（1.4 節）。一つも無い binary の行は走らせない。
完了条件（各 unit）：`rg -n "impl (Element|IntoView) for" <unit が持つ file>` が空。`rg -n "#\[component\]" <unit が持つ file>` が表の部品の数だけある。命令が通る。

## U40：button と icon_button を `#[component]` にする（完全な例）

- **wave / lane / 前提**：W8 / U1 / U42（同じ lane の前の unit）、W7a の全 unit、annex C の W5（4.2 節 R1〜R5）。
- **所有**：`crates/kivact-ui/src/button.rs`、`crates/kivact-ui/src/button/look.rs`、`crates/kivact-ui/src/lib.rs` の button の `pub use` の行。
- **目的**：5.1 節（DEC-03）。`Button` の builder を `#[component] pub fn button(..) -> impl Element` にする。`icon_button` を `impl Element` を返す `#[component]` にする（5.9 節）。
- **手順**：
  1. `crates/kivact-ui/src/button.rs` を次の内容にする（`BusyPhase`・`BusyEvent`・`BusyInput`・`busy_indicator`・`busy_animation` は W7a の後の本体を写す。ここに書いた形と違う行があれば、W7a の後の行を使う。変えるのは builder の部分だけ）。
     ```rust
     //! Role: Build the themed button from a momentary toggle part.
     //!
     //! The root, the content row, the icon boxes and the busy indicator all get
     //! one [`ButtonState`]. Its `expanded` is read from the caller's semantics:
     //! `Some(open)` when a popup, menu or disclosure trigger part was attached
     //! or the caller set `.expanded`, `None` otherwise. A [`ButtonVariant`] is
     //! given with `.variant(..)` and read by the recipe with
     //! `look.variant::<ButtonVariant>()` (the default is `Primary`).
     pub(crate) mod look;

     use crate::control_size::{ControlSize, default_control_size};
     use crate::layer::{with_base_recipe_style, with_recipe_style};
     use look::{
         BUSY_REDUCED_SPIN_MS_VAR, BUSY_REVEAL_DELAY_MS_VAR, BUSY_REVEAL_FADE_MS_VAR, BUSY_SPIN_MS_VAR,
         ButtonState, ButtonTheme,
     };
     use kivact_macros::component;
     use kivact_core::custom::Caller;
     use kivact_core::motion::{Animated, Easing, FrameRate, Transition, frame_loop};
     use kivact_core::state::{IntoValue, Machine, Step, Value, machine, memo, signal};
     use kivact_core::style::{Icon, style, use_theme};
     use kivact_core::view::{Element, IntoView, Placement, Variant, Variants, View, node, row, show, text};
     use kivact_interaction::{DisabledButtonFocus, InteractionPolicy, use_button, use_tooltip};
     use std::time::Duration;

     /// What a button means. A theme's button recipe reads it with
     /// `look.variant::<ButtonVariant>()`; a button without one is `Primary`.
     #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
     pub enum ButtonVariant {
         /// The main action of its area.
         #[default]
         Primary,
         /// A secondary action.
         Secondary,
         /// An action drawn with an outline.
         Outline,
         /// An action drawn without a fill until hovered.
         Ghost,
         /// A destructive action.
         Danger,
         /// An action drawn as a link.
         Link,
     }
     impl Variant for ButtonVariant {}

     // BusyPhase, BusyEvent, BusyInput, impl Machine for BusyInput: unchanged (copy from W7a).

     /// A themed button activated through the shared button part ([`use_button`]).
     ///
     /// `label` is the visible text and the accessible name. `.icon(view)` and
     /// `.trailing(view)` put views before and after it in boxes the theme's
     /// `ButtonTheme::icon` styles; `.hide_label(true)` keeps the name and
     /// omits the visible label. `.busy(true)` shows the busy indicator and
     /// ignores presses. Without `.size(..)` the button takes the size its
     /// group or [`provide_control_size`](crate::provide_control_size) gives.
     #[component]
     pub fn button(
         label: Value<String>,
         icon: Option<View>,
         trailing: Option<View>,
         children: Option<Vec<View>>,
         size: Option<Value<ControlSize>>,
         busy: Option<Value<bool>>,
         hide_label: Option<Value<bool>>,
         #[caller] caller: Caller,
     ) -> impl Element {
         let variants = caller.variants();
         let part = use_button().control();
         let input = part.state();
         let size = size.unwrap_or_else(|| default_control_size().into_value());
         let expanded = caller.expanded();
         // Busy ignores presses; `absorb` ORs this with the caller's `.disabled(..)`.
         let mut root = node()
             .part(part)
             .focusable_when_disabled(move || {
                 matches!(
                     use_theme().get().ext::<InteractionPolicy>().button.disabled_focus,
                     DisabledButtonFocus::Retain
                 )
             })
             .name(label);
         if let Some(busy) = busy {
             root = root.busy(busy);
         }
         if let Some(busy) = busy {
             root = root.disabled(busy);
         }
         let (leading_icon, trailing_icon) = (icon.is_some(), trailing.is_some());
         let state = memo(move || ButtonState {
             input: input.get(),
             size: size.get(),
             busy: busy.is_some_and(|busy| busy.get()),
             expanded: expanded.map(|open| open.get()),
             leading_icon,
             trailing_icon,
             label_hidden: hide_label.is_some_and(|hide| hide.get()),
         });
         // The text style, then the recipe: base layers below every `.style(..)` of the caller.
         let root = with_base_recipe_style(
             root.base_style(crate::helpers::single_line_text()),
             |r: &ButtonTheme| r.root_recipe(),
             &variants,
             move || state.get(),
         );
         // A button inside a button group gets the group's corners and divider (W7a U23).
         let root = crate::button_group::member_style(root, &variants);
         let slot = {
             let variants = variants.clone();
             move |icon: View| {
                 with_recipe_style(node().child(icon), |r: &ButtonTheme| r.icon_recipe(), &variants, move || state.get())
             }
         };
         // Only a button the caller gave `.busy(..)` creates the busy clock, its
         // machine and its indicator (MEM-5).
         let busy_parts = busy.map(|_| busy_animation(state));
         let content = with_recipe_style(
             row((
                 icon.map(&slot),
                 crate::helpers::visible_label(label, hide_label),
                 trailing.map(&slot),
                 children.unwrap_or_default(),
             )),
             |r: &ButtonTheme| r.content_recipe(),
             &variants,
             move || {
                 let mut state = state.get();
                 state.busy &= !busy_parts.is_some_and(|(_, reveal)| reveal.get());
                 state
             },
         );
         let indicator = busy_parts.map(|(phase, _)| busy_indicator(state, &variants, phase));
         root.child(content).child(indicator)
     }

     /// A button that shows only `icon`, is named `label` and shows `label` as
     /// its tooltip. Its variant is `Ghost` unless the caller gives another.
     #[component]
     pub fn icon_button(icon: Value<Icon>, label: Value<String>) -> impl Element {
         crate::tooltip(
             label,
             button(label)
                 .icon(kivact_core::view::icon(icon))
                 .hide_label(true)
                 .variant(ButtonVariant::Ghost),
         )
     }

     // busy_indicator(state, variants: &Variants, phase) and busy_animation(state): copy from
     // W7a; they read the four BUSY_* variables with `use_theme().get().get(&BUSY_SPIN_MS_VAR)` (U21).
     ```
     注：
     - `button_group::member_style(root: Node, variants: &Variants) -> Node` は、W7a の U23 手順 11 で `finish` の中に書いた group の角と区切り線の式を、`button_group.rs` に `pub(crate) fn member_style` として切り出したものである（この unit で切り出す。`button_group.rs` の所有はこの関数の追加だけ。他の行は U44）。
     - `icon_button` の caller の modifier（`.variant(..)`、`.style(..)`）は tooltip の root（＝button）に absorb され、button の `#[caller]` の snapshot で `Ghost` より後に畳まれる（U25）。
     - 生成された struct は `Button`・`IconButton`。`lib.rs` は `pub use button::{Button, ButtonVariant, IconButton, button, icon_button};`。
  2. `crates/kivact-ui/src/button/look.rs` は変えない（U20・U21 で `ButtonTheme` と変数がある）。
- **消すもの**：`struct Button`（手書き）、`enum ButtonSize`、`impl Button`、`impl Element for Button`、`impl IntoView for Button`、`Button::finish`、W7a の `fn icon_button(..) -> View`。
- **状態の閉じ方**：新しい型は無い。生成された `Button` の props の field は値渡しの setter だけが書く（6.1 節の対象外）。`ButtonState` は接尾辞 `State`。
- **試験**：期待値は変えない。`ui_controls.rs`、`ui_theming.rs`、`ui_toolbar.rs`、`interaction_toggle.rs`（button の press）、`feedback.rs`（busy）、`components.rs` が通ること。足す試験（5.9 節の `icon_button` の契約。`ui_controls.rs`）：
  ```rust
  #[test]
  fn an_icon_button_is_named_by_its_label_and_hides_it() {
      let (screen, _) = crate::support::mount(|| ((), icon_button("x", "Close").test_id("close")));
      let close = screen.find_by_test_id("close").expect("invariant: mounted");
      assert_eq!(close.role(), Some(Role::Button));
      assert_eq!(close.name().as_deref(), Some("Close"));
      assert!(screen.find_text("Close").is_none(), "the label is not drawn until the tooltip opens");
      assert!(screen.errors().is_empty(), "{:?}", screen.errors());
  }
  ```
  （`find_text`・`role`・`name` は `ui_controls.rs` が今使う support の名前に合わせる。）
  さらに `ui_controls.rs` に足す（12 章 API-4・API-5）：
  ```rust
  #[test]
  fn a_busy_button_is_not_pressed_even_when_its_caller_enables_it() {
      let (screen, pressed) = crate::support::mount(|| {
          let pressed = signal(0u32);
          (pressed, button("Save").busy(true).disabled(false).on_press(move || pressed.update(|n| *n += 1)))
      });
      screen.get_by_role(Role::Button, "Save").click();
      assert_eq!(pressed.get_untracked(), 0);
      assert!(screen.errors().is_empty(), "{:?}", screen.errors());
  }

  #[test]
  fn the_root_content_and_icon_recipes_read_the_same_variant() {
      let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
      let (root, content, icon) = (seen.clone(), seen.clone(), seen.clone());
      let theme = Theme::light().with(
          ButtonTheme::default()
              .root(move |_, look, base| { root.borrow_mut().push(("root", look.variant::<ButtonVariant>())); base })
              .content(move |_, look, base| { content.borrow_mut().push(("content", look.variant::<ButtonVariant>())); base })
              .icon(move |_, look, base| { icon.borrow_mut().push(("icon", look.variant::<ButtonVariant>())); base }),
      );
      let screen = kivact::test::render_with(kivact::test::RenderOptions::default().theme(theme), || {
          icon_button("x", "Close").variant(ButtonVariant::Danger)
      });
      assert!(!seen.borrow().is_empty());
      assert!(seen.borrow().iter().all(|(_, v)| *v == Some(ButtonVariant::Danger)), "{:?}", seen.borrow());
      assert!(screen.errors().is_empty(), "{:?}", screen.errors());
  }
  ```
  （Ghost と呼出し側の override は `ui_theming.rs` に足す `an_icon_button_draws_the_ghost_look_by_default` で確かめる：`Theme::light().with(ButtonTheme::default().root(|_, look, base| if look.variant::<ButtonVariant>() == Some(ButtonVariant::Ghost) { base.background(rgb(RED)) } else { base.background(rgb(BLUE)) }))` の下で `icon_button("x", "Close").test_id("a")` と `icon_button("x", "Open").variant(ButtonVariant::Primary).test_id("b")` を mount し、`at(&screen, "a", 4.0, 4.0)` が RED に、`at(&screen, "b", 4.0, 4.0)` が BLUE に近い（`distance(..) <= 8`）。）
- **確認（tier a）**：規則 C7。`<tests>` = e2e_ui の `ui_controls ui_theming ui_toolbar interaction_toggle feedback` と e2e_core の `components`。
- **完了条件**：`rg -n "impl (Element|IntoView) for" crates/kivact-ui/src/button.rs` が空。`rg -c "#\[component\]" crates/kivact-ui/src/button.rs` が 2。命令が通る。

## U42：scalar・range scalar・splitter の disabled を node だけにする（UI-11 の scalar の部分）
- **wave / lane / 前提**：W8 / U1（lane の最初の unit）/ W7b の全 unit。前提にする状態：
  - W7b の U31 が `scalar/`・`range_scalar.rs`・`splitter/` の `*_state()` getter と `part_slot::attach_part_state` の行を消している（この unit の手順の行はそれと重ならない）。U34 の後も `crates/kivact-ui/src/number_input.rs` に `use_scalar(value, range).disabled(disabled)` の行が残っている。
  - W7a の後の呼出しの形：`slider(value).label("..")`、`number_input(value, range).label("..")`、`<Name>Theme` の recipe。W5 の後の `use_direction()` は値を返す。U08（W3）で `scalar/focus_machine.rs` は消えている。
  - W8 の他の lane はこの unit の file を持たない（lane U1 が ui の `slider`・`splitter`・`number_input` と interaction の `scalar/`・`range_scalar.rs`・`splitter/` を持つ）。U40 はこの unit の後に走る。
- **所有**：`crates/kivact-interaction/src/scalar/{edit_state.rs,handle.rs,track.rs}`、`crates/kivact-interaction/src/range_scalar.rs`、`crates/kivact-interaction/src/splitter/{handle.rs,options.rs}`、`crates/kivact-ui/src/slider.rs`、`crates/kivact-ui/src/splitter.rs`、`crates/kivact-ui/src/number_input.rs`（`use_scalar(..).disabled(disabled)` の 1 行だけ）、試験 `dev/tests/kivact-public-api/tests/e2e_ui/interaction_slider.rs`。
- **目的**：UI-11（scalar）。`ScalarApi::disabled`・`RangeScalarApi::disabled`・`SplitterApi::disabled` と scalar の `host_disabled` を消し、part を付ける node の `.disabled(..)`（`.part(..)` より前に書く）を disabled の唯一の源にする。
- **手順**：
  1. `scalar/edit_state.rs`：
     - `Shared` の field `host_disabled` とその doc、`ScalarOptions` の field `disabled` を消す。
     - `Shared::new` の `let options = settings.value();`、`let host_disabled = ..;`、`let disabled = memo(..);` を消し、`track::drag(session, snapshot, pointer, disabled)` を `track::drag(session, snapshot, pointer)` にする。struct literal の `host_disabled,` を消す。
     - `fn is_disabled` と `fn register_host_disabled`（doc ごと）を消す。
     - `fn enable_touch` の `effect(move || { if self.is_disabled() { touch.finish(false); } });` を消す（touch の設定を入れる行は残す）。
     - `fn touch_source` を次にする：
       ```rust
       pub(crate) fn touch_source(
           self,
           part: kivact_core::view::Node,
           grip: Grip,
           disabled: Value<bool>,
       ) -> kivact_core::view::Node {
           let Some(touch) = self.settings.get().touch else {
               return part;
           };
           effect(move || {
               if disabled.get() {
                   touch.finish(false);
               }
           });
           part.on_pointer(move |event| {
               if disabled.get_untracked() {
                   Handled::No
               } else {
                   touch.event(grip, event)
               }
           })
       }
       ```
     - `cargo clippy` が未使用と言う import（`memo`、`on_cleanup` など）を消す。
  2. `scalar/track.rs`：`pub(crate) fn drag` の引数 `disabled: Value<bool>` を消し、`let off = memo(move || !pointer.attached.get() || disabled.get());` を `let off = memo(move || !pointer.attached.get());` にする。doc の「Disabled while `disabled` holds a true value or ..」の部分を「Disabled while no track is attached; a source reads the disabled state of the node it is attached to at each press.」にする。
  3. `scalar/handle.rs`：
     - `pub fn disabled(self, ..)` を doc ごと消す。`use super::edit_state::{.., register_host_disabled}` から `register_host_disabled` を消す。
     - `pub fn track` の `Part::new` の closure の先頭の `register_host_disabled(shared, host.accessible_disabled());` を `let disabled = host.accessible_disabled().unwrap_or_else(|| false.into_value());` にし、`shared.touch_source(surface, Grip::Track)` を `shared.touch_source(surface, Grip::Track, disabled)` にする。
     - `fn part` の `Part::new` の closure の先頭の `register_host_disabled(shared, host.accessible_disabled());` を同じ `let disabled = ..;` にする。`control = control.disabled(move || shared.is_disabled()).node_ref(self.edit.thumb);` を `control = control.node_ref(self.edit.thumb);` にし、`shared.touch_source(control, grip)` を `shared.touch_source(control, grip, disabled)` にする。
     - `ScalarApi` の型の doc と `thumb`・`input`・`track` の doc で disabled の setter に触れている文を「`.disabled(..)` on the node a part is attached to (written before `.part(..)`) disables that part.」にする。
  4. `range_scalar.rs`：`pub fn disabled(self, ..)` を doc ごと消す。`pub fn body` の `Part::new(part_state, move |host| { .. })` を次にする：
     ```rust
     Part::new(part_state, move |mut host| {
         let disabled = host.accessible_disabled().unwrap_or_else(|| false.into_value());
         edit.shared
             .touch_source(host.part(input).focus_on_press(edit.thumb), Grip::Body, disabled)
             .part(edit.shared.drag.source(Grip::Body))
     })
     ```
  5. `splitter/options.rs`：`SplitterOptions` の field `disabled` を消す。
  6. `splitter/handle.rs`：
     - `pub fn disabled` と `fn is_disabled` を doc ごと消す。
     - `fn handle_key(self, event: &KeyEvent, shrink_key: Key, grow_key: Key)` に最後の引数 `disabled: Value<bool>` を足し、`if self.is_disabled()` を `if disabled.get_untracked()` にする。
     - `pub fn handle` の `let act = move |event: A11yEvent| { .. };` を消し、`Part::new` の closure の中の `let disabled = ..;` の直後に同じ closure を置く。その中の `!self.is_disabled()` を `!disabled.get_untracked()` にする。
     - `Part::new` の closure の `let caller_disabled = host.accessible_disabled();`、`let options = settings.value();`、`let disabled = memo(move || { .. });` の 3 文を `let disabled = host.accessible_disabled().unwrap_or_else(|| false.into_value());` の 1 文にする（`machine.feed(disabled, ..)`、`self.handle_drag(disabled, ..)`、最後の `.disabled(disabled)` はそのまま）。
     - `.on_key(move |event: KeyEvent| self.handle_key(&event, shrink_key, grow_key))` を `.on_key(move |event: KeyEvent| self.handle_key(&event, shrink_key, grow_key, disabled))` に、on_pointer の `&& !self.is_disabled()` を `&& !disabled.get_untracked()` にする。
     - `pub fn handle` の `let settings = self.settings;` を消す（読むのは消した `options` の文だけ。`reset` は `self.settings` を読む）。型の doc の「A disabled splitter accepts no input ..」の文の主語を「A handle whose node is disabled (`.disabled(..)` before `.part(..)`)」にする。
  7. ui の呼出し：
     - `crates/kivact-ui/src/slider.rs`：
       - `fn slider_parts` の `let with_disabled = move |scalar: ScalarApi| match disabled { .. };` を消し、`with_disabled(use_scalar(value, range).on_edit(report))` を `use_scalar(value, range).on_edit(report)` に、`let pair = match disabled { Some(disabled) => pair.disabled(disabled), None => pair };` を消す。`slider_parts` の引数 `disabled` を消し、呼出し側の引数からも消す。
       - `fn thumb_views` に最後の引数 `disabled: Option<Value<bool>>` を足し、`node().part(part).name(name)` を `node().disabled(move || disabled.is_some_and(|disabled| disabled.get())).part(part).name(name)` にする。呼出しに `disabled` を渡す。
       - `fn filled_span` に最後の引数 `disabled: Option<Value<bool>>` を足し、`Some(body) => node().part(body)` を `Some(body) => node().disabled(move || disabled.is_some_and(|disabled| disabled.get())).part(body)` にする（range の span の drag も disabled のとき止まる）。呼出しに `disabled` を渡す。
       - `into_view` の `let disabled = root.accessible_disabled();` とその上の注釈は残す（track は root の `.part(track_part)` で root の disabled を読む）。
     - `crates/kivact-ui/src/splitter.rs`：`if let Some(disabled) = root.accessible_disabled() { behavior = behavior.disabled(move || disabled.get()); }` を `let disabled = root.accessible_disabled();` にし、handle の `node().part(part)` を `node().disabled(move || disabled.is_some_and(|disabled| disabled.get())).part(part)` にする。注釈の「`.disabled(..)` on the splitter ..」は残す。
     - `crates/kivact-ui/src/number_input.rs`：`use_scalar(value, range).disabled(disabled)` を `use_scalar(value, range)` にする（text input の node が `.disabled(disabled)` を `.with_node(|node| node.part(number.input()))` より前に持ち、part がそれを読む）。
- **消すもの**：`ScalarApi::disabled`、`RangeScalarApi::disabled`、`SplitterApi::disabled`、`ScalarOptions::disabled`、`SplitterOptions::disabled`、`Shared::host_disabled`、`Shared::is_disabled`、`register_host_disabled`、`SplitterApi::is_disabled`、ui の `with_disabled`。
- **状態の閉じ方**：新しい型は無い。`ScalarOptions`・`SplitterOptions` は Option の field を減らすだけ（名前が `Options` で対象外）。印は要らない。
- **試験**：
  - 追加 `dev/tests/kivact-public-api/tests/e2e_ui/interaction_slider.rs`（`ScalarRange`・`mount` は既に import にある）：
    ```rust
    #[test]
    fn a_disabled_slider_and_number_field_take_their_disabled_state_from_the_node() {
        let (screen, (volume, quantity, off)) = mount(|| {
            let volume = signal(50.0);
            let quantity = signal(2.0);
            let off = signal(true);
            let range = ScalarRange::new(0.0, 10.0).unwrap().with_step(1.0).unwrap();
            ((volume, quantity, off), column((
                kivact::ui::slider(volume).label("Volume").disabled(off).style(style().width(200)),
                kivact::ui::number_input(quantity, range).label("Quantity").disabled(off),
            )))
        });
        assert!(screen.get_by_role(Role::Slider, "Volume").is_disabled());
        assert!(screen.get_by_role(Role::SpinButton, "Quantity").is_disabled());
        off.set(false);
        screen.run_until_idle();
        assert!(!screen.get_by_role(Role::Slider, "Volume").is_disabled());
        screen.get_by_role(Role::Slider, "Volume").press_key(Key::ArrowRight);
        assert!(volume.get_untracked() > 50.0);
        screen.get_by_role(Role::SpinButton, "Quantity").press_key(Key::ArrowUp);
        assert_eq!(quantity.get_untracked(), 3.0);
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
    ```
  - 変えずに通す：`interaction_slider.rs`、`apg_slider.rs`、`apg_spinbutton.rs`、`interaction_splitter.rs`、`apg_window_splitter.rs`、`interaction_edit.rs`、`ui_controls.rs` の既存の全試験。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction -p kivact-ui -p kivact-public-api-tests
  cargo check -p kivact-interaction -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo xtask check --no-write --crate kivact-ui
  cargo check -p kivact -p kivact-devtools -p kivact-public-api-tests --all-targets
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_slider apg_slider apg_spinbutton interaction_splitter apg_window_splitter interaction_edit ui_controls
  ```
- **完了条件**：`rg -n "host_disabled|register_host_disabled|fn is_disabled" crates/kivact-interaction/src/scalar crates/kivact-interaction/src/splitter` が空。`rg -n "pub fn disabled" crates/kivact-interaction/src/scalar crates/kivact-interaction/src/range_scalar.rs crates/kivact-interaction/src/splitter` が空。`rg -n "scalar\.disabled\(|pair\.disabled\(|behavior\.disabled\(|use_scalar\([^)]*\)\.disabled" crates/kivact-ui/src/slider.rs crates/kivact-ui/src/splitter.rs crates/kivact-ui/src/number_input.rs` が空。命令が通る。

## U43：data table の filter の draft と行の drag を interaction へ（UI-04 の data table の部分）
- **wave / lane / 前提**：W8 / U3（lane の最初の unit）/ W7b の全 unit。前提にする状態：
  - W7b の U34 が `crates/kivact-interaction/src/draft.rs` に `use_draft(value: Signal<T>, options: DraftOptions<T, D>) -> DraftApi<T, D>`、`DraftOptions::new(show, accept).commit(DraftCommit::Live)`、`DraftApi::{is_valid, editor}` を作っている。U30a が `data_table/rows.rs` を変え終えている。U35 が calendar の swipe・composer の履歴・date sheet の machine を ui から消している。
  - W7a の後の呼出しの形：`text_input(value).label(label)`（U22）、`DialogKind`（U23）。W5 の後：`Signal<T>`（`Binding` は無い）、`binding(get, set) -> Signal<T>`、payload のある handler は `impl Fn(E) -> R + 'static`（DEC-16）。
  - W8 の他の lane はこの unit の file を持たない（lane U3 が `data_table` と interaction の `reorder_drag.rs` を持つ）。U49 はこの unit の後に走る。
- **所有**：新規 `crates/kivact-interaction/src/reorder_drag.rs`。変更 `crates/kivact-interaction/src/lib.rs`（`mod`・`pub use` の行だけ）、`crates/kivact-ui/src/data_table/filters.rs`、`crates/kivact-ui/src/data_table/rows.rs`（`fn drag_source` と import だけ）、`crates/kivact-ui/src/data_table/row_drag.rs`（削除）、`crates/kivact-ui/src/data_table.rs`（`mod row_drag;` の行だけ）、試験 `dev/tests/kivact-public-api/tests/e2e_ui/ui_data_table.rs`。
- **目的**：UI-04 規則 1：data table の入力で動く machine（filter の draft、row の drag）を ui から出し、ui に残る `impl Machine` を表示だけの timer と初期 focus の 5 つにする。
- **手順**：
  1. `crates/kivact-interaction/src/reorder_drag.rs` を作る。ui の `data_table/row_drag.rs` の `RowDrag`・`RowDragEvent` を移して private にし、`RowPlace` を private な `struct Machine { place: ReorderPlace, step: f64 }` にする。file の全体：
     ```rust
     //! Role: Drag one item of a vertical list to move it one place per step.
     use crate::{DragApi, DragCoordinates, DragPhase, DragState, DragUpdate, Reorder, use_drag};
     use kivact_core::custom::Part;
     use kivact_core::state::{Machine as MachineTrait, Step, callback, machine};
     use kivact_core::view::HandlerResult;
     use std::marker::PhantomData;

     /// Where the dragged item is among its siblings, read at each drag update.
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub struct ReorderPlace {
         pub at: Option<usize>,
         pub len: usize,
     }

     /// Where a dragged item is: idle, or dragged with the offset at which it last moved one place.
     #[derive(Clone, Copy, PartialEq)]
     enum RowDrag {
         Idle,
         Dragging { applied: f64 },
     }
     enum RowDragEvent {
         Start,
         Update(f64),
         End,
         Cancel,
     }
     /// The dragged item's place, read for each event, and the step that moves it one place.
     struct Machine {
         place: ReorderPlace,
         step: f64,
     }
     impl MachineTrait for Machine {
         type State = RowDrag;
         type Event = RowDragEvent;
         type Effect = usize;
         fn transition(&self, state: &RowDrag, event: RowDragEvent) -> Step<RowDrag, RowDragEvent, usize> {
             match (*state, event) {
                 (_, RowDragEvent::Start) => Step::to(RowDrag::Dragging { applied: 0.0 }),
                 (_, RowDragEvent::End | RowDragEvent::Cancel) => Step::to(RowDrag::Idle),
                 (RowDrag::Dragging { applied }, RowDragEvent::Update(offset)) => {
                     let dy = offset - applied;
                     match self.place.at {
                         Some(at) if dy.abs() > self.step => {
                             let to = if dy > 0.0 {
                                 (at + 1).min(self.place.len.saturating_sub(1))
                             } else {
                                 at.saturating_sub(1)
                             };
                             Step::to(RowDrag::Dragging { applied: offset }).with(to)
                         }
                         _ => Step::stay(),
                     }
                 }
                 (RowDrag::Idle, RowDragEvent::Update(_)) => Step::stay(),
             }
         }
     }

     /// The drag of one item of a reorderable list.
     pub struct ReorderDragApi<K: 'static> {
         drag: DragApi<()>,
         key: PhantomData<fn() -> K>,
     }
     impl<K: 'static> Clone for ReorderDragApi<K> {
         fn clone(&self) -> Self {
             *self
         }
     }
     impl<K: 'static> Copy for ReorderDragApi<K> {}

     /// Report `Reorder { key, to }` to `on_move` each time the drag passes
     /// `step` px past the last move. `place` is read at each update. Setup only.
     #[track_caller]
     pub fn use_reorder_drag<K: Clone + 'static, R: HandlerResult>(
         key: K,
         step: f64,
         place: impl Fn() -> ReorderPlace + 'static,
         on_move: impl Fn(Reorder<K>) -> R + 'static,
     ) -> ReorderDragApi<K> {
         let on_move = callback(on_move);
         let session = machine(
             RowDrag::Idle,
             move || Machine { place: place(), step },
             move |_, to: usize| on_move.call(Reorder { key: key.clone(), to }),
         );
         let drag = use_drag::<()>(DragCoordinates::Window)
             .on_start(move |_| session.send(RowDragEvent::Start))
             .on_update(move |update: DragUpdate<()>| session.send(RowDragEvent::Update(update.delta.y)))
             .on_end(move |update: DragUpdate<()>| {
                 session.send(if update.phase == DragPhase::Cancel {
                     RowDragEvent::Cancel
                 } else {
                     RowDragEvent::End
                 })
             });
         ReorderDragApi { drag, key: PhantomData }
     }

     impl<K: 'static> ReorderDragApi<K> {
         /// The item's drag part. Setup only.
         #[track_caller]
         pub fn source(self) -> Part<DragState> {
             self.drag.source(())
         }
     }
     ```
     （`ReorderDragApi` の `Clone`・`Copy` は derive にしない：derive は `K: Clone`・`K: Copy` の bound を付けるが、`PhantomData<fn() -> K>` は K に依らず Copy。）`crates/kivact-interaction/src/lib.rs` に `mod reorder_drag;` と `pub use reorder_drag::{ReorderDragApi, ReorderPlace, use_reorder_drag};` を足す。
  2. `crates/kivact-ui/src/data_table/rows.rs` の `fn drag_source` の本体を次にする（署名は変えない）：
     ```rust
     let (source, key, sort) = (grid.source, grid.key.clone(), grid.sort);
     let place = id.clone();
     use_reorder_drag(
         id,
         28.0,
         move || {
             let (at, len) =
                 source.with(|rows| (rows.iter().position(|row| key(row) == place), rows.len()));
             ReorderPlace { at, len }
         },
         move |request: kivact_interaction::Reorder<K>| {
             reorder.call(request);
             sort.set(None);
         },
     )
     .source()
     ```
     `use super::row_drag::{RowDrag, RowDragEvent, RowPlace};` を消し、`use kivact_interaction::{ReorderPlace, use_reorder_drag};` を足す。`cargo clippy` が未使用と言う import（`machine`、`use_drag`、`DragCoordinates`、`DragPhase`、`DragUpdate` のうち他で使わないもの）を消す。
  3. `crates/kivact-ui/src/data_table/row_drag.rs` を `git rm` し、`crates/kivact-ui/src/data_table.rs` の `mod row_drag;` の行を消す。
  4. `crates/kivact-ui/src/data_table/filters.rs`：
     - `enum FilterDraft` とその `impl`、`enum FilterEvent`、`struct FilterInput`、`impl Machine for FilterInput` を消す。
     - `fn text_editor` の `let initial = value.get_untracked();` から `let valid = memo(move || matches!(draft.get(), FilterDraft::Valid { .. }));` までを次の 3 行にする：
       ```rust
       let input = field.clone();
       let draft = use_draft(value, DraftOptions::new(|value: &Option<FilterValue>| format_value(value.as_ref()), move |text: &String| parse_value(&input, text).ok()).commit(DraftCommit::Live));
       let valid = draft.is_valid();
       ```
     - `let draft = binding(move || draft.get().text().to_owned(), move |text| draft.send(FilterEvent::Typed(text)));` の文を消す。`text_input(draft).label(label)`（U22 の後の形）の引数 `draft` を `draft.editor()` にして `text_input(draft.editor()).label(label)` にする。`.invalid(move || !valid.get())` 以降はそのまま。
     - import に `use kivact_interaction::{DraftCommit, DraftOptions, use_draft};` を足し、`cargo clippy` が未使用と言う import（`Machine`、`Step`、`machine`、`memo`、`binding` のうち他で使わないもの）を消す。
  5. 確認：この unit の後、`rg "impl Machine for|impl.*Machine for" crates/kivact-ui/src` の出力がちょうど次の 5 行（行番号を除く）であること：
     ```
     crates/kivact-ui/src/alert.rs: impl Machine for BannerMachine {          （alert の collapse の timer）
     crates/kivact-ui/src/button.rs: impl Machine for BusyInput {             （busy の表示の遅延）
     crates/kivact-ui/src/calendar/navigation.rs: impl Machine for FocusSnapshot { （mount で一度の初期 focus）
     crates/kivact-ui/src/empty_state/changes.rs: impl Machine for Controller {    （empty state の変化の告知）
     crates/kivact-ui/src/progress.rs: impl Machine for ProgressMotionInput {  （progress の動き）
     ```
     （`select/machine.rs` は U33、`selection_field` は U23、switch・toast・drawer は U32、date 系・pagination・number は U34、calendar の swipe・composer・date sheet は U35 が消している。6 行以上出たら、出た行の machine を消すはずの unit の手順が抜けているので STOP する。）
- **消すもの**：ui の `data_table/row_drag.rs`（`RowDrag`、`RowDragEvent`、`RowPlace`）、`FilterDraft`・`FilterEvent`・`FilterInput`。
- **状態の閉じ方**：`ReorderDragApi`（Api）は名前で対象外。`ReorderPlace`（`Option<usize>` と `usize`）と `Machine`（`ReorderPlace` と f64）は constructor でだけ作る snapshot で、`&mut self` の method も内部可変も持たないので対象外。`RowDrag` は enum。印は要らない。
- **試験**：
  - 追加 `dev/tests/kivact-public-api/tests/e2e_ui/ui_data_table.rs`（import の `use kivact::interaction::{ScalarRange, SortOrder, TableFocus};` を `use kivact::interaction::{Reorder, ScalarRange, SortOrder, TableFocus};` にする）：
    ```rust
    #[test]
    fn dragging_a_row_reports_one_place_per_step() {
        let (screen, moves) = mount_with(options(), || {
            let moves = signal(Vec::<(u32, usize)>::new());
            (moves, data_table(people(), |p: &Person| p.id, columns()).name("People")
                .on_reorder(move |request: Reorder<u32>| { moves.update(|m| m.push((request.key, request.to))); }))
        });
        let cell = screen.get_by_role(Role::GridCell, "Ada").rect().expect("laid out");
        let (x, y) = (cell.x + cell.width / 2.0, cell.y + cell.height / 2.0);
        screen.mouse_move(x, y);
        screen.mouse_down(MouseButton::Primary);
        screen.mouse_move(x, y + 30.0);
        screen.mouse_move(x, y + 45.0);
        screen.mouse_up(MouseButton::Primary);
        assert_eq!(moves.get_untracked(), [(1, 1)], "a step past 28 px moves one place, a shorter one does not");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
    ```
  - 変えずに通す：`ui_data_table.rs` と `interaction_table.rs` の既存の全試験（filter の入力と検証の試験を含む）。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction -p kivact-ui -p kivact-public-api-tests
  cargo check -p kivact-interaction -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo xtask check --no-write --crate kivact-ui
  cargo check -p kivact -p kivact-devtools -p kivact-public-api-tests --all-targets
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_data_table interaction_table
  ```
- **完了条件**：手順 5 の `rg` が 5 行だけを出す。`rg -nw "RowPlace|RowDragEvent|FilterInput|FilterDraft" crates/kivact-ui/src` が空。`crates/kivact-ui/src/data_table/row_drag.rs` が無い。命令が通る。

## U44〜U54：部品の群ごとの `#[component]` 化

各 unit は、2.3 節の表の自分の行の部品を、U40 と規則 C1〜C7 のとおりに書き換える。表の「特別な扱い」の列はその部品だけの手順である。

**U24 の残り（2026-11-21 の設計者の回答）**：`Dialog`（U47）・`HoverCard`（U47）・`Menu`（U48）・`CommandPalette`（U48）・`ToastRegion`（U48）は W7a の時点で node を持たず、U24 の後も固有の `pub fn variant` と field `variants: Variants` を残している。この 5 つを `#[component]` にする unit は、root の node を `Element` にし（C1〜C7 のとおり）、固有の `variant` と `variants` の field を消して、variant を `caller` の `Node::authored` から読む（tooltip の U47 の注記と同じ形）。呼出し元の `.variant(..)` は変わらない（`Element::variant` に解決される）。

| unit | wave / lane / 前提 | 所有（`crates/kivact-ui/src/` 以下。各部品の `<name>.rs` と `<name>/` の全部） | 部品 |
|---|---|---|---|
| U44 | W8 / U1 / U40 | `toggle`、`toggle_button`、`toggle_group`、`button_group`（U40 の `member_style` の追加を除く）、`toolbar` | checkbox、switch、toggle_button、toggle_group、button_group、button_group_separator、button_group_text、toolbar |
| U45 | W8 / U1 / U44 | `text_input`、`number_input`、`tag_input`、`field`、`input_group` | text_input、text_area、number_input、tag_input、field、input_group、input_addon |
| U46 | W8 / U1 / U45 | `slider`、`splitter`、`split`、`dropzone`、`async_view` | slider、range_slider、splitter、split、dropzone、async_view |
| U41 | W8 / U2（最初） | 3.4 節の U41 の所有 | （InteractionPolicy の規則 4、UI-25） |
| U47 | W8 / U2 / U41 | `dialog`、`tooltip`、`hover_card`、`disclosure`、`announcer_outlet` | dialog、tooltip、hover_card、disclosure、disclosure_group、announcer_outlet |
| U48 | W8 / U2 / U47 | `menu`、`select`、`combobox`、`command_palette`、`navigation_menu`、`toast` | menu、select、combobox、command_palette、navigation_menu、navigation_menu_link、toast_region |
| U43 | W8 / U3（最初） | 3.4 節の U43 の所有 | （data table の machine） |
| U49 | W8 / U3 / U43 | `list`、`tree`、`tabs`、`radio_group` | list、tree、tree_rows、tabs、radio_group |
| U50 | W8 / U3 / U49 | `data_table`（`data_table.rs` と `data_table/` の全部） | data_table |
| U51 | W8 / U3 / U50 | `pagination`、`breadcrumb`、`stepper`、`carousel`、`item_list`、`table`、interaction の `carousel/{api.rs, machine.rs}` | pagination、breadcrumb、stepper、carousel、item_list、item_row、table、table_header、table_body、table_footer、table_row、table_head、table_cell |
| U52 | W8 / U4（最初） | `alert`、`card`、`empty_state`、`badge`、`kbd`、`avatar`、`avatar_group`、`spinner`、`skeleton`、`progress`、`sparkline`、`heading`、`stat`、`description_list`、`dev/xtask/policy/workspace.toml` の `[ui_machine] allowed_types` の行 | alert、card、empty_state、badge、kbd、kbd_combo、avatar、avatar_group、spinner、skeleton、progress、sparkline、heading、stat、description_list、description |
| U53 | W8 / U4 / U52 | `chat_message`、`chat_thread`、`chat_composer`、`typing_indicator`、`code_block` | chat_message、chat_thread、chat_composer、typing_indicator、code_block |
| U54 | W8 / U4 / U53 | `calendar`、`date_input`、`time_input`、`date_range_input`、`date_time_input`、`date_parts` | calendar、date_input、time_input、date_range_input、date_time_input |

### U51 の carousel：index と scroll の同期を machine の一つの経路にする（UI-07 の残り）

U51 は carousel を `#[component]` にする前に、次を行う（所有：`crates/kivact-interaction/src/carousel/{api.rs, machine.rs}`、`crates/kivact-ui/src/carousel.rs`）。

1. `carousel/machine.rs`：`Event` を `GoTo(usize)`、`ScrollObserved(usize, u64)`、`Settle` にする（`Go` → `GoTo`、`Scroll` → `ScrollObserved` の改名。遷移は今と同じ）。`Effect` を次にする：
   ```rust
   pub(super) enum Effect {
       /// Write the index and scroll the slide into view (a command or an outside write).
       Go(usize),
       /// Write the index only: the user's own scroll settled there.
       Settled(usize),
   }
   ```
   `Event::GoTo(index)` の腕は `.with(Effect::Go(index))`、`Event::Settle` の腕は `.with(Effect::Settled(*target_index))`。
2. `carousel/api.rs`：`use_carousel` を次の署名にし、effect の処理を書き換える：
   ```rust
   #[track_caller]
   pub fn use_carousel(index: Signal<usize>, count: usize, scroll_to: impl Fn(usize) + 'static) -> CarouselApi {
       let state = machine(
           CarouselState::Idle { index: index.get_untracked().min(count.saturating_sub(1)) },
           || Machine,
           move |_, effect| match effect {
               Effect::Go(at) => {
                   index.set(at);
                   scroll_to(at);
               }
               Effect::Settled(at) => index.set(at),
           },
       );
       state.feed(memo(move || index.get().min(count.saturating_sub(1))), Event::GoTo);
       CarouselApi { state, index, count }
   }
   ```
   `go_to` は `Event::GoTo` を送る。`scrolling(nearest)` は `Event::ScrollObserved` を送る（名前は変えない）。
3. ui `carousel.rs`：`fn follow_index` の effect を消し、その本体（`if !matches!(c.navigation.get(), ..)` の早期 return を除く）を `fn scroll_to_slide(view: SlideView, refs: &[NodeRef], at: usize)` にする。`SlideView` は `#[derive(Clone, Copy)] struct SlideView { viewport: NodeRef, vertical: Value<bool>, rtl: Value<ReadingDirection>, snap: Value<Snap>, padding: Value<f64> }`（`Layout` が今持つ同じ field。`Layout` はこの struct を field `view: SlideView` で持ち、`c.padding()` などの読みは `c.view.padding.get()` にする）。`SlideView` を `use_carousel` の前に作り、`use_carousel(index, count, move |at| scroll_to_slide(view, &navigation_refs, at))` を呼ぶ。`navigation_refs` の `Rc` は closure に clone して渡す。
4. `follow_scroll`（U09 の後は scroll offset の `effect_on` で `c.api.scrolling(at)` を呼ぶ）は変えない。effect が signal を書く所は残らない（`rg -n "effect\(" crates/kivact-ui/src/carousel.rs` が空）。
5. 試験：`ui_carousel.rs` と `apg_carousel.rs` が期待値を変えずに通ること。

各 unit の共通の欄：
- **目的**：5.1 節・DEC-03（ui の部品をすべて `#[component]` に）、UI-14（部品ごとの module に一つの書き方）。
- **消すもの**：表の部品の builder struct・`impl Element for`・`impl IntoView for`・`finish`・setter（規則 C5）。
- **状態の閉じ方**：新しい状態の型は無い。生成された struct の props は値渡しの setter だけが書く。
- **試験**：期待値は変えない。2.3 節の表の「試験」の列の file が通ること。U52 は UI-21 の残り（下の表の sparkline の行）のために `ui_sparkline.rs` に次を足す：
  ```rust
  #[test]
  fn a_sparkline_recipe_draws_the_line_from_its_state() {
      // A theme that colors the line red through the recipe, not a token.
      let theme = Theme::light().with(kivact::ui::theme::SparklineTheme::default().root(|_, look, base| {
          base.paint(kivact::ui::theme::sparkline_line(look, rgb(RED)))
      }));
      let screen = kivact::test::render_with(kivact::test::RenderOptions::default().theme(theme), || {
          kivact::ui::sparkline(vec![0.0, 1.0, 0.0]).test_id("spark").style(style().size(60, 20))
      });
      assert!(distance(pixel(&screen, "spark", 30.0, 2.0), RED) <= 16);
      assert!(screen.errors().is_empty(), "{:?}", screen.errors());
  }
  ```
  （`sparkline_line(look, color) -> impl Fn(&mut Draw, Size)` は U52 で `sparkline/look.rs` に作り、`theme.rs` で再公開する既定の線の描画。`pixel`・`distance` は `ui_sparkline.rs` の今の helper に合わせる。）
- **確認（tier a）**：規則 C7。`<tests>` は 2.3 節の表の「試験」の列。
- **完了条件**：規則 C7。
- **各 lane の最後の unit の追加の確認**（annex T 4.4 の pending の検査を W8 の終わりで厳密に通す）：U46・U48・U51・U54 は規則 C7 の命令に加えて次を走らせる（interaction の検査は lane の自分の branch で決まる）。
  ```
  cargo xtask check --no-write --strict --only interaction-view,unit-machine --crate kivact-interaction
  ```
  kivact-ui 全体を見る検査（`ui-machine`、`hand-element`、`element-shadow`）は 4 つの lane の部品がそろうまで通らないので、どの unit も走らせない。作業指示書の W8 の merge の後の確認（Orchestrator が統合 branch で tier b の前に走らせる。失敗は U4 の lane の fix）が次の 2 行である：
  ```
  cargo xtask check --no-write --strict --only interaction-view,ui-machine,unit-machine,hand-element,element-shadow --crate kivact-ui
  cargo xtask check --no-write --strict --only interaction-view,ui-machine,unit-machine --crate kivact-interaction
  ```
- **U52 の追加の手順**（`[ui_machine] allowed_types` の path をまとめて直す。W7b・W8 で `dev/xtask/policy/workspace.toml` を持つのは U52 だけ。所有に `dev/xtask/policy/workspace.toml` の `[ui_machine]` の `allowed_types` の行を足す）：`allowed_types` の配列を次の 5 項目にする（U14 が `empty_state_changes.rs` を `empty_state/changes.rs` に移し、`calendar/navigation.rs` の `FocusSnapshot` は mount で一度だけ初期 focus を当てる印で入力では動かないので残す。button・progress・alert の Machine は W8 の後も root の file にある）：
  ```toml
  allowed_types = ["kivact_ui::alert::BannerMachine", "kivact_ui::button::BusyInput", "kivact_ui::calendar::navigation::FocusSnapshot", "kivact_ui::empty_state::changes::Controller", "kivact_ui::progress::ProgressMotionInput"]
  ```
  （この配列は U54 の merge の後の確認で効く。U53・U54 はこの行を変えない。）

## U41：InteractionPolicy 規則 4 と UI-25（`HoverCardTiming` → `Delays`）
- **wave / lane / 前提**：W8 / U2（lane の最初の unit）/ W7b の全 unit。前提にする状態：
  - W7b の U32 が `crates/kivact-interaction/src/switch_drag.rs` を作り、その中で `InteractionPolicy` を `.switch.drag.0` で読んでいる（この読みは 1 行だけ）。U32 が `dev/tests/kivact-public-api/tests/e2e_ui/interaction_toggle.rs` に `a_switch_drag_commits_once_and_vertical_or_cancelled_contacts_do_not_toggle` を足し、import `use kivact::interaction::{InteractionPolicy, SwitchDrag, SwitchPolicy};` と `SwitchPolicy { drag: SwitchDrag(true) }` を書いている。
  - W7b の U30c の後の `use_hover_card(enabled: impl IntoValue<bool>, focus_opens: impl IntoValue<bool>, timing: HoverCardTiming, open: Signal<bool>) -> HoverCardApi`。ui の `hover_card.rs` は `use_hover_card(self.enabled, .., self.timing, requested)` で呼ぶ。
  - `InteractionPolicy` の field は今の 10 個（`menu`、`tooltip`、`hover_card`、`drawer`、`button`、`slider`、`switch`、`toast`、`select`、`scrub`）のまま（W7b までの unit は field を足さない）。
  - W5・W6 の API（`Signal<T>`、`Theme::light()`、`theme.with(ext)`）。W8 の他の lane はこの unit の file を持たない（lane U2 が interaction の policy の file と ui の `hover_card` を持つ）。U47 はこの unit の後に走る。
- **所有**：`crates/kivact-interaction/src/interaction_policy.rs`、`crates/kivact-interaction/src/menu_policy.rs`、`crates/kivact-interaction/src/surface_policy.rs`、`crates/kivact-interaction/src/hover_card.rs`、`crates/kivact-interaction/src/menu/press_state.rs`、`crates/kivact-interaction/src/menu.rs`（`pub use press_state::MenuPress;` の行だけ）、`crates/kivact-interaction/src/menu/session.rs`（policy を読む式だけ）、`crates/kivact-interaction/src/switch_drag.rs`（policy を読む行だけ）、`crates/kivact-interaction/src/lib.rs`（`pub use` の行だけ）、`crates/kivact-ui/src/hover_card.rs`（型名と field 名だけ）、試験 `dev/tests/kivact-public-api/tests/e2e_ui/interaction_toggle.rs`（U32 の試験の import と 1 行）。
- **目的**：UI-04 規則 4：`SwitchDrag(pub bool)` と `MenuPress(pub bool)` を `bool` の field `switch.drag`・`menu.press` にする。UI-25：`HoverCardTiming` を消し、`InteractionPolicy::hover_card` の型を `Delays` にする。
- **手順**：
  1. `interaction_policy.rs`：
     - `pub struct SwitchDrag(pub bool);` とその doc・derive を消す。`SwitchPolicy` を次にする：
       ```rust
       #[derive(Clone, Debug, PartialEq, Default)]
       pub struct SwitchPolicy {
           /// Whether switches use direct manipulation (default off).
           pub drag: bool,
       }
       ```
     - `MenuPolicy` の `pub press: MenuPress,` を次にする：
       ```rust
       /// Whether menus use native press, drag and release selection (default off).
       pub press: bool,
       ```
     - `use crate::menu_policy::{DisabledMenuFocus, MenuPress, SubmenuTiming};` から `MenuPress` を消す。
     - `InteractionPolicy` の `pub hover_card: crate::HoverCardTiming,` を `pub hover_card: crate::Delays,` にし、`#[derive(Clone, Debug, PartialEq, Default)]` から `Default` を外して、struct の後に次を足す：
       ```rust
       impl Default for InteractionPolicy {
           fn default() -> Self {
               Self {
                   menu: MenuPolicy::default(),
                   tooltip: TooltipTiming::default(),
                   hover_card: crate::Delays {
                       open: std::time::Duration::from_millis(600),
                       close: std::time::Duration::from_millis(300),
                   },
                   drawer: crate::DrawerPolicy::default(),
                   button: ButtonPolicy::default(),
                   slider: SliderPolicy::default(),
                   switch: SwitchPolicy::default(),
                   toast: ToastPolicy::default(),
                   select: SelectPolicy::default(),
                   scrub: ScrubPolicy::default(),
               }
           }
       }
       ```
  2. `menu_policy.rs`：`pub struct MenuPress(pub bool);` とその doc・derive を消す。`menu/press_state.rs` の `pub use crate::menu_policy::MenuPress;` と、`menu.rs` の `pub use press_state::MenuPress;` を消す。`menu/session.rs` の `let press = (kivact_core::style::use_theme().get().ext::<crate::InteractionPolicy>().menu.press).0.then(Press::new);`（fmt で複数行の式）を、括弧と `.0` を外した `let press = kivact_core::style::use_theme().get().ext::<crate::InteractionPolicy>().menu.press.then(Press::new);` にする。
  3. `surface_policy.rs`：`HoverCardTiming` と `impl Default for HoverCardTiming` を消す。先頭の Role の行を `//! Role: Configure drawer dismissal gestures.` にする。`use std::time::Duration;` が使われなくなるので消す。
  4. interaction の `hover_card.rs`：`use crate::{DelayedPhase, Delays, HoverCardTiming, HoverSurfaceApi};` から `HoverCardTiming` を消す。`use_hover_card` の引数 `timing: HoverCardTiming` を `timing: Delays` にし、本体の `Delays { open: timing.open_delay, close: timing.close_delay }` を `timing` にする。
  5. interaction の `switch_drag.rs`：`.switch.drag.0;` を `.switch.drag;` にする。
  6. `crates/kivact-interaction/src/lib.rs`：`pub use interaction_policy::{..}` から `SwitchDrag` を、`pub use menu::{..}` から `MenuPress` を、`pub use surface_policy::{DrawerPolicy, DrawerSwipe, HoverCardTiming};` から `HoverCardTiming` を消す。
  7. `crates/kivact-ui/src/hover_card.rs`：import の `HoverCardTiming` を `Delays` にし、builder の field `timing: HoverCardTiming` を `timing: Delays` に、`self.timing.open_delay = delay;` を `self.timing.open = delay;` に、`self.timing.close_delay = delay;` を `self.timing.close = delay;` にする（setter の名前 `open_delay`・`close_delay` は変えない）。
  8. `dev/tests/kivact-public-api/tests/e2e_ui/interaction_toggle.rs`：`use kivact::interaction::{InteractionPolicy, SwitchDrag, SwitchPolicy};` を `use kivact::interaction::{InteractionPolicy, SwitchPolicy};` に、`SwitchPolicy { drag: SwitchDrag(true) }` を `SwitchPolicy { drag: true }` にする。
- **消すもの**：`SwitchDrag`、`MenuPress`、`HoverCardTiming`、`InteractionPolicy` の `derive(Default)`。
- **状態の閉じ方**：`SwitchPolicy`・`MenuPolicy`・`InteractionPolicy` は名前が `Policy` で対象外。新しい型は無い。印は要らない。
- **試験**：手順 8 の 2 か所だけを直す（期待値は変えない）。変えずに通す：`interaction_toggle.rs::a_switch_drag_commits_once_and_vertical_or_cancelled_contacts_do_not_toggle`、`ui_disclosure_surfaces.rs::hover_card_opens_after_the_delay_and_on_focus`、`ui_disclosure_surfaces.rs::hover_card_controlled_binding_rejection_and_external_close_are_authoritative`、`interaction_menu.rs`・`apg_menu.rs`・`interaction_tooltip.rs` の全試験。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-interaction -p kivact-ui -p kivact-public-api-tests
  cargo check -p kivact-interaction -p kivact-ui --all-targets
  cargo clippy -p kivact-interaction -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo xtask check --no-write --crate kivact-ui
  cargo check -p kivact -p kivact-devtools -p kivact-public-api-tests --all-targets
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_toggle ui_disclosure_surfaces interaction_menu apg_menu interaction_tooltip
  ```
- **完了条件**：`rg -n "\b(SwitchDrag|MenuPress|HoverCardTiming)\b|open_delay:|close_delay:" crates/kivact-interaction/src crates/kivact-ui/src dev/tests/kivact-public-api/tests/e2e_ui/interaction_toggle.rs` が空。`rg -n "drag\.0|press\)\s*\.0" crates/kivact-interaction/src` が空。命令が通る。

## 4. 他の付属書との接点

### 4.1 この付属書が使う他の付属書の名前（前提）

| 付属書 | 名前 | 使う unit |
|---|---|---|
| T（W1） | e2e の binary `e2e_core`・`e2e_ui`・`e2e_native`（`kivact-public-api-tests`）と `e2e_kits`（`kivact-kit-tests`）、`state` 検査（6.1 節の接尾辞 `State`・`Theme`・`Api`・`Options` などの除外）、API-13・API-14 の xtask 検査（C の T-REQ-2）、ui は入力で動く `impl Machine` を持たない検査（6.3 節） | 全 unit |
| S（W1a・W6） | `kivact_types::cell::SharedDeferred`、`ThemeVar<T>`、`ThemeValue`（`Color`・`f64`・`u16`・`LineHeight`・`FontFamily`）、`Theme::{get, set}`、`Color::var`、`Length::var`、`color::*`・`space::*`・`radius::*`・`font::*`（S 4.2 節の名前）、変数の宣言の型（S 4.1 節：`static <NAME>_VAR` と `const <NAME>`、名前の文字列 `"<部品>.<field>"`） | U20、U21 |
| R（W5） | `Signal<T>`、`Trigger`、`IntoHandler<(), M>`（payload の無い handler だけ）、payload のある handler は `impl Fn(E) -> R + 'static`（DEC-16）、`Callback<E>`、`update` は `()` を返す（DEC-15）、`use_context -> T`。R の掃引 R29 が ui の呼出し元を W5 で直している | W7 以降の全 unit |
| C（W5） | C09 の `#[component]`（generic の部品、5.1 規則 9）、C10 の `Element::variant`・`kivact_core::view::{Variant, Variants}`（`Variants::get::<V>() -> Option<V>`）、C の U-REQ-1〜6 | U24、U25、W8 |
| E（W5a） | `EditState::goal_x()`、`move_vertically(to, extend, goal_x)`（E17）、`word_at(at)`、`line_at(at, multiline)`（E20、E18 の後）。`set_goal_x` は無い（INC-05） | U36 |
| K（W9） | 中核の `sidebar`（`SidebarTheme`、K11・K12）、tabs の overflow（K13）、`Slant`・`Striped`・`SyntaxKind`（K08）、`NodeRef::once_visible`・`task::clock`（K09）。**この付属書はこれらを実装しない**。K は `crates/kivact-ui/src/<名前>.rs` ＋ `<名前>/look.rs` の形（1.4 節：`mod.rs` は作らない）で置き、`theme.rs` に `pub use` を 1 行足す | — |


### 4.2 この付属書が core・macro に作る受け口（U25）と、W8 が前提にするもの

| ID | 名前・署名 | 作る unit | 理由 |
|---|---|---|---|
| R1 | `#[component]` の生成する struct の名前は関数名の UpperCamelCase（`button` → `Button`）で、関数と同じ可視性。generic の部品（5.1 規則 9） | annex C の C09（既にある） | W8 の `lib.rs` の `pub use` が型名を保つため |
| R2 | 字面 `Rc<dyn Fn(A, ..) -> T>` の props は `impl Fn(A, ..) -> T + 'static` を受ける。`T` が字面 `View` なら `impl Fn(A, ..) -> V + 'static`（`V: IntoView`） | U25 | slot（`item`、`row_details`）と関数の props（`format_with`、`filter`）の closure の引数の型を推論させる（DEC-16 と同じ理由。`impl Into<T>` の「それ以外」では推論されない） |
| R3 | `#[component]` の引数 `#[caller] caller: Caller`（`kivact_core::custom::Caller` = core の `Authored`）。macro が body の前に `__root.authored()` の snapshot を一度だけ作って渡す。method：`variants`、`disabled`、`expanded`、`invalid`、`name`、`description`、`labelled_by`、`has_press`。`Node::authored()` は `Modifier::Caller` の段と開いた段を順に畳む一つの関数で、recipe・root・部分が同じ値を使う。absorb は本体と呼出し側の disabled を OR にする | U25 | 本体は absorb の前に走るので、呼出し側が返り値に付けた modifier を読む方法が要る。thread-local の stack は呼出し元に束縛されない暗黙の状態になる（12 章 API-4・API-5・ST-09） |
| R4 | `kivact_core::view::Children`（`From<C: IntoChildren>`、`into_views(self)`） | U25 | C09 の `Vec<View>` は constructor で view にするので、子は部品の context の外で setup が走る。button_group・toolbar・input_group・item_list・navigation_menu は子に context を provide する |
| R5 | `Variants::with<V: Variant>(self, V) -> Variants` | U25 | `Look::with_variant` |
| R6 | 型引数そのものの props（`control: C`）は `C` を受ける（Param の kind） | U25 | 12 章 API-1：`field`・`tooltip`・`async_view` |

### 4.3 表「UiTokens → 部品の theme 変数」（U 変数表。U21 が作り、annex K が kit の `set` に使う）

書き方：`<module>::<NAME>` は `kivact::ui::theme::<module>::<NAME>`（`static <NAME>_VAR` と、長さ・色なら `const <NAME>`）。「設定」は `<Name>Theme` の `settings`（`theme.with(XTheme::default().<setting>(v))`、読みは `<setting>_value()`）。名前の文字列は `"<module>.<NAME の小文字>"`。fallback の「→ `x::Y`」と「中核 `color::X`」は中核の変数への**別名**（12 章 UI-U-02。中核の値を変えると部品にも届く。部品の変数を set すれば部品だけが変わる）。別名は色も数も**必ず** `ThemeVar::alias(name, &X_VAR)` で書く（`ThemeVar::new(name, color::X)` の値の形は使わない。付属書 S 4.1）。`f64` でも `Length` でない値（不透明度）は `_VAR` だけで、別名は `ThemeVar::alias(.., &control::DISABLED_OPACITY_VAR)`。旧い theme の field `control_height`・`disabled_opacity`・`overlay_margin` は付属書 S の中核の変数 `kivact::style::control::{HEIGHT, OVERLAY_MARGIN}`（const と `_VAR`）と `kivact::style::control::DISABLED_OPACITY_VAR`（`static` だけ） への別名にする。`focus_ring` は theme の field（style）のままで変数ではないので、それに従っていた部品の変数は数の fallback（2）にする。

| 旧（`UiTokens` の field） | 新 | 型 | fallback |
|---|---|---|---|
| `button.busy_motion.spin_ms` | `button::BUSY_SPIN_MS_VAR` | f64（ms） | 700 |
| `button.busy_motion.reduced_spin_ms` | `button::BUSY_REDUCED_SPIN_MS_VAR` | f64（ms） | 700 |
| `button.busy_motion.reveal_delay_ms` | `button::BUSY_REVEAL_DELAY_MS_VAR` | f64（ms） | 0 |
| `button.busy_motion.reveal_fade_ms` | `button::BUSY_REVEAL_FADE_MS_VAR` | f64（ms） | 0 |
| `avatar.name_tooltip` | `AvatarTheme` の設定 `name_tooltip` | bool | false |
| `calendar.cell_size`・`month_gap`・`week_gap` | `calendar::CELL_SIZE`・`MONTH_GAP`・`WEEK_GAP` | Length | 32・16・8 |
| `carousel.gap`・`padding`・`edge_fade_width` | `carousel::GAP`・`PADDING`・`EDGE_FADE_WIDTH` | Length | 16・0・4 |
| `chat.typing_period_ms` | `typing_indicator::PERIOD_MS_VAR` | f64（ms） | 1000 |
| `chat.follow_slack` | `chat_thread::FOLLOW_SLACK` | Length | 24 |
| `code_block.font_size`・`line_height` | `code_block::FONT_SIZE`・`LINE_HEIGHT` | Length | 14・22 |
| `code_block.copy_feedback_ms` | `code_block::COPY_FEEDBACK_MS_VAR` | f64（ms） | 2000 |
| `combobox.foreground`・`muted`・`background`・`border`・`active_background`・`chip_background`・`focus` | `combobox::FOREGROUND`・`MUTED`・`BACKGROUND`・`BORDER`・`ACTIVE_BACKGROUND`・`CHIP_BACKGROUND`・`FOCUS` | Color | 中核 `color::TEXT`・`TEXT_MUTED`・`SURFACE`・`BORDER`・`HOVER`・`SELECTION`・`FOCUS` |
| `combobox.field_radius` | `combobox::FIELD_RADIUS` | Length | → `radius::MD` |
| `combobox.focus_ring` | `combobox::FOCUS_RING` | Length | 2（`focus_ring` は theme の field で変数ではない） |
| `combobox.disabled_opacity` | `combobox::DISABLED_OPACITY_VAR` | f64 | → `control::DISABLED_OPACITY_VAR` |
| `combobox.empty_font_size` | `combobox::EMPTY_FONT_SIZE` | Length | → `font::SIZE_SMALL`（look の今の `unwrap_or(t.text().size_small)`） |
| `combobox.field_height`・`field_padding`・`chip_height`・`chip_radius`・`option_height`・`option_padding_x`・`option_padding_y`・`option_radius`・`list_height`・`popup_offset` | `combobox::FIELD_HEIGHT`・`FIELD_PADDING`・`CHIP_HEIGHT`・`CHIP_RADIUS`・`OPTION_HEIGHT`・`OPTION_PADDING_X`・`OPTION_PADDING_Y`・`OPTION_RADIUS`・`LIST_HEIGHT`・`POPUP_OFFSET` | Length | 32・4・24・4・32・8・6・4・292・6 |
| `data_table.sticky.surface`・`hover`・`selected`・`shadow` | `data_table::STICKY_SURFACE`・`STICKY_HOVER`・`STICKY_SELECTED`・`STICKY_SHADOW` | Color | 中核 `color::SURFACE`・`HOVER`・`SURFACE_ALT`・`Color::TRANSPARENT` |
| `data_table.sticky.shadow_width` | `data_table::STICKY_SHADOW_WIDTH` | Length | 0 |
| `data_table.sticky.shadow_transition`・`overlay_transition` | `DataTableTheme` の設定 `sticky_shadow_transition`・`sticky_overlay_transition` | `Option<Transition>` | None |
| `data_table.row_status.neutral`・`info`・`success`・`warning`・`danger`・`accent` | `data_table::ROW_STATUS_NEUTRAL`・`ROW_STATUS_INFO`・`ROW_STATUS_SUCCESS`・`ROW_STATUS_WARNING`・`ROW_STATUS_DANGER`・`ROW_STATUS_ACCENT` | Color | 中核 `TEXT_MUTED`・`ACCENT`・`SUCCESS`・`WARNING`・`DANGER`・`ACCENT` |
| `data_table.row_status.header_key` | `DataTableTheme` の設定 `row_status_header_key` | `&'static str` | `"data_table.row_status.column_header"` |
| `date_input.wheel_row_height`・`wheel_width` | `date_input::WHEEL_ROW_HEIGHT`・`WHEEL_WIDTH` | Length | 40・80 |
| `time_input.wheel_row_height`・`wheel_width` | `time_input::WHEEL_ROW_HEIGHT`・`WHEEL_WIDTH` | Length | 40・80 |
| `dialog.center` | `DialogTheme` の設定 `center` | `DialogCenter` | `Anchor` |
| `disclosure.motion` | `DisclosureTheme` の設定 `motion` | `DisclosureMotion` | 今の `Default` |
| `dropzone.padding`・`min_height` | `dropzone::PADDING`・`MIN_HEIGHT` | Length | 16・96 |
| `pagination.gap`・`padding`・`control_height`・`panel_padding` | `pagination::GAP`・`PADDING`・`CONTROL_HEIGHT`・`PANEL_PADDING` | Length | 8・4・32・8 |
| `breadcrumb.*`（同じ 4 つ） | `breadcrumb::GAP`・`PADDING`・`CONTROL_HEIGHT`・`PANEL_PADDING` | Length | 8・4・32・8 |
| `navigation_menu.*`（同じ 4 つ） | `navigation_menu::GAP`・`PADDING`・`CONTROL_HEIGHT`・`PANEL_PADDING` | Length | 8・4・32・8 |
| `stepper.gap`・`padding`・`min_step_width`・`connector_gap` | `stepper::GAP`・`PADDING`・`MIN_STEP_WIDTH`・`CONNECTOR_GAP` | Length | 8・4・96・8 |
| `progress.motion` | `ProgressTheme` の設定 `motion` | `ProgressMotion` | 今の `Default` |
| `sparkline.line` | `sparkline::LINE` | Color | 中核 `color::ACCENT` |
| `sparkline.line_width`・`inset` | `sparkline::LINE_WIDTH`・`INSET` | Length | 1.5・0 |
| `sparkline.round`・`fill` | `SparklineTheme` の設定 `round`・`fill` | `bool`・`SparklineFill` | false・`None` |
| `toast.row_motion.gap` | `toast::ROW_GAP` | Length | 0 |
| `toast.row_motion.transition` | `ToastTheme` の設定 `row_transition` | `Option<Transition>` | None |
| `toggle_button.label_reservation` | `ToggleButtonTheme` の設定 `label_reservation` | `ToggleLabelReservation` | `Natural` |
| `toolbar.background`・`border` | `toolbar::BACKGROUND`・`BORDER` | Color | `Color::TRANSPARENT` |
| `toolbar.padding`・`gap`・`min_height` | `toolbar::PADDING`・`GAP`・`MIN_HEIGHT` | Length | 8・4・32 |
| `tree.marker_width`・`indent`・`padding`・`gap`・`row_height`・`radius` | `tree::MARKER_WIDTH`・`INDENT`・`PADDING`・`GAP`・`ROW_HEIGHT`・`RADIUS` | Length | 16・16・→ look の `unwrap_or` の中核の変数・→ 同・→ `control::HEIGHT`・→ `radius::MD` |
| `item_list.gap`・`padding`・`row_gap`・`description_size` | `item_list::GAP`・`PADDING`・`ROW_GAP`・`DESCRIPTION_SIZE` | Length | → `space::XS`・→ look の `unwrap_or` の中核の変数・→ 同・→ `font::SIZE_SMALL` |
| `item_list.title_lines`・`description_lines` | `ItemListTheme` の設定 `title_lines`・`description_lines` | `Option<u32>` | None |
| `table.cell_padding`・`header_height`・`font_size` | `table::CELL_PADDING`・`HEADER_HEIGHT`・`FONT_SIZE` | Length | → look の `unwrap_or` の中核の変数 |
| `button_group.gap`・`radius` | `button_group::GAP`・`RADIUS` | Length | 0・0 |
| `button_group.joined` | `ButtonGroupTheme` の設定 `joined` | bool | true |
| `button_group.divider`・`solid_divider`・`text_background`・`text_color` | `button_group::DIVIDER`・`SOLID_DIVIDER`・`TEXT_BACKGROUND`・`TEXT_COLOR` | Color | `TRANSPARENT`・`TRANSPARENT`・`TRANSPARENT`・`Color::BLACK` |
| `toggle_group.gap` | `toggle_group::GAP` | Length | 4 |
| `hover_card.offset`・`viewport_margin` | `hover_card::OFFSET`・`VIEWPORT_MARGIN` | Length | 4・8 |
| `indicators.period_ms` | `spinner::PERIOD_MS_VAR` と `skeleton::PERIOD_MS_VAR`（読む部品ごと） | f64（ms） | 1000 |
| `indicators.reduced_period_ms` | `spinner::REDUCED_PERIOD_MS_VAR`・`skeleton::REDUCED_PERIOD_MS_VAR` | f64（ms） | 1000（`None` は period と同じ。kit は period を set するとき両方を set する） |
| `indicators.skeleton_transition` | `SkeletonTheme` の設定 `transition` | `Option<Transition>` | None |
| `indicators.delay_ms`・`stagger_ms` | `skeleton::DELAY_MS_VAR`・`STAGGER_MS_VAR` | f64（ms） | 0・100 |
| `indicators.line_height`・`gap` | `skeleton::LINE_HEIGHT`・`GAP` | Length | 16・6 |
| `indicators.avatar_overlap` | `avatar_group::OVERLAP` | Length | 8 |

「→ look の `unwrap_or` の中核の変数」の行は、U21 の時点の look の関数がその `None` の代わりに使っている式の中核の変数（`space::SM` など）への別名を fallback にする（数にしない）。U21 は別名が中核の値に従うことを確かめる試験を `ui_theming.rs` に足す：`component_variables_follow_the_core_variable_they_alias`：`Theme::light().set(&radius::MD_VAR, 20.0)` の下で `kivact::ui::theme::combobox::FIELD_RADIUS_VAR` を `theme.get(..)` で読むと 20、さらに `.set(&combobox::FIELD_RADIUS_VAR, 3.0)` なら 3（部品の値が共通の値より優先する）。同じ確かめを `tree::INDENT_VAR`→ その中核の変数、`item_list::GAP_VAR`→`space::XS_VAR` について行う。`indicators.*` のうち、どの部品が読むかは `rg -n "indicators\." crates/kivact-ui/src` の結果で決める：読む部品ごとに変数を作り、読まない部品には作らない。

### 4.4 表「UiRecipes → <Name>Theme」（U recipe 表）と annex K への対応（W7〜W8 の後）

**recipe の上書きの書き方**：旧 `UiRecipes::default().button(|prev| move |theme, look| prev(theme, look).border_radius(0))` → 新 `ButtonTheme::default().root(|theme, look, base| base.border_radius(0))`。旧 `UiRecipes::default().select(|t| t.trigger(|prev| ..))` → 新 `SelectTheme::default().trigger(|theme, look, base| ..)`。`theme.with(..)` に表を一つずつ入れる。recipe を読んで呼ぶ所：旧 `recipes.button_recipe()(theme, look)` → 新 `theme.ext::<ButtonTheme>().root_recipe().style(theme, look)`。旧 `Recipe` の `Deref`・`From<F>` は無い。`Look::variant::<V>()` は `Option<V>`（値。旧は `Option<&V>`。`.copied()`・`.cloned()` を消す）。`Look::new(state).with_variant(v)` は同じ。

**平らな field**（旧 `UiRecipes::<field>`）→ 新：

| 旧 | 新 | | 旧 | 新 |
|---|---|---|---|---|
| `button` | `ButtonTheme::root` | | `avatar` | `AvatarTheme::root` |
| `button_content` | `ButtonTheme::content` | | `avatar_image` | `AvatarTheme::image` |
| `button_busy` | `ButtonTheme::busy` | | `avatar_status` | `AvatarTheme::status` |
| `button_icon` | `ButtonTheme::icon` | | `avatar_status_content` | `AvatarTheme::status_content` |
| `checkbox` | `CheckboxTheme::root` | | `avatar_group` | `AvatarGroupTheme::root` |
| `checkbox_indicator` | `CheckboxTheme::indicator` | | `avatar_group_member` | `AvatarGroupTheme::member` |
| `checkbox_check` | `CheckboxTheme::check` | | `avatar_group_overflow` | `AvatarGroupTheme::overflow` |
| `checkbox_mixed` | `CheckboxTheme::mixed` | | `kbd` | `KbdTheme::root` |
| `switch` | `SwitchTheme::root` | | `kbd_key` | `KbdTheme::key` |
| `switch_track` | `SwitchTheme::track` | | `badge` | `BadgeTheme::root` |
| `switch_thumb` | `SwitchTheme::thumb` | | `badge_label` | `BadgeTheme::label` |
| `toggle` | `ToggleButtonTheme::root` | | `spinner` | `SpinnerTheme::root` |
| `text_input` | `TextInputTheme::root` | | `skeleton` | `SkeletonTheme::root` |
| `text_area` | `TextAreaTheme::root` | | `heading` | `HeadingTheme::root` |

**入れ子の表**（旧 `UiRecipes::<table>(|t| t.<part>(..))` と旧 `<X>Recipes`）→ 新 `<X>Theme`（part の名前は同じ）：`stat`→`StatTheme`、`description_list`→`DescriptionListTheme`、`chat_message`→`ChatMessageTheme`、`chat_thread`→`ChatThreadTheme`、`chat_composer`→`ChatComposerTheme`、`typing_indicator`→`TypingIndicatorTheme`、`code_block`→`CodeBlockTheme`、`empty_state`→`EmptyStateTheme`、`alert`→`AlertTheme`、`card`→`CardTheme`、`input_group`→`InputGroupTheme`、`calendar`→`CalendarTheme`、`date_input`→`DateInputTheme`、`date_range_input`→`DateRangeInputTheme`、`time_input`→`TimeInputTheme`、`date_time_input`→`DateTimeInputTheme`、`combobox`→`ComboboxTheme`、`toolbar`→`ToolbarTheme`、`pagination`→`PaginationTheme`、`breadcrumb`→`BreadcrumbTheme`、`stepper`→`StepperTheme`、`navigation_menu`→`NavigationMenuTheme`、`toggle_group`→`ToggleGroupTheme`、`button_group`→`ButtonGroupTheme`、`select`→`SelectTheme`、`menu`→`MenuTheme`、`dialog`→`DialogTheme`、`tooltip`→`TooltipTheme`、`hover_card`→`HoverCardTheme`、`tabs`→`TabsTheme`、`radio_group`→`RadioGroupTheme`、`slider`→`SliderTheme`、`toast`→`ToastTheme`、`field`→`FieldTheme`、`splitter`→`SplitterTheme`、`list`→`ListTheme`、`tree`→`TreeTheme`、`table`→`TableTheme`、`item_list`→`ItemListTheme`、`data_table`→`DataTableTheme`、`split`→`SplitTheme`、`progress`→`ProgressTheme`、`sparkline`→`SparklineTheme`（状態は `SparklineState` になる。U52）、`disclosure`→`DisclosureTheme`、`carousel`→`CarouselTheme`、`dropzone`→`DropzoneTheme`、`command_palette`→`CommandPaletteTheme`、`tag_input`→`TagInputTheme`、`number_input`→`NumberInputTheme`。新しい表：`AsyncViewTheme`（U46）、`AnnouncerOutletTheme`（U47）。各表の part と状態の型は 2.3 節の表。

**型の path**（API-19）：recipe の状態の型、`<Name>Theme`、`Look`、`Variant`、`Recipe`、設定の型（`DialogCenter`、`DisclosureMotion`、`ProgressMotion`、`SparklineFill`、`ToggleLabelReservation`）、状態の中だけの型（`CheckControl`、`Expansion`、`Selectable`、`DisclosureGroupPosition`、`CalendarDayVisual`、`CalendarEventVisual`、`ItemRowState`、`TreeRowRecipeState`、`DataTableCheckState`、`SelectSheetOption`、`DataTableTreeIndent`）は `kivact::ui::X` → `kivact::ui::theme::X`。`ThumbDrag`・`TrackInput`・`Lift` は `kivact::interaction::X`（U32）。状態の型の名前は変えない（K の R1 が例に挙げた `CheckableState` → `CheckboxState` の改名は**しない**。`CheckableState` のまま）。`impl kivact::ui::Variant for X {}` → `impl kivact::ui::theme::Variant for X {}`（X は `Clone` を derive する。C10）。

**消える型**（kit が使っていたら 4.3 節の変数か設定に置き換える）：`UiRecipes`、`UiTokens`、すべての `*Recipes`、すべての `*Tokens`（4.3 節の表）、`ButtonBusyMotion`、`ToastRowMotion`、`DataTableStickyPaint`、`DataTableRowStatusPaint`、ui の `Variants`（内部）、`ButtonGroupMember`、`SelectionField`、`ui::ComboboxMode`（`kivact::ui::ComboboxMode` は ui に移り、`kivact::interaction::ComboboxMode` が消える）、`ui::Snap`（`kivact::interaction::Snap` を使う）。

**呼出しの改名**：

| 旧 | 新 | unit |
|---|---|---|
| `text_input(label, v)`・`text_area`・`select(label, items, sel)`・`combobox(label, items, sel)`・`slider(label, v)`・`range_slider(label, lo, hi)`・`number_input(label, v, range)`・`date_input(label, v)`・`date_range_input(label, r)`・`date_time_input(label, v)`・`time_input(label, v)`・`tag_input(label, tags)` | 先頭の `label` を消し `.label(label)` を足す（`field(..)` の中なら不要） | U22 |
| `TagInput::on_add`・`on_remove`、`tags: impl IntoValue<..>` | 消す。`tags: Signal<Vec<String>>` | U22・U23 |
| `on_draft_commit` | `on_commit` | U23 |
| `stepper(items, current: Value)`、`.on_step_press(f)` | `stepper(items, current: Signal<usize>)` | U23 |
| `Avatar::on_open(f)` | `.on_press(f).role(Role::Link)` | U23 |
| `input_group_with(label, init)` | `input_group(label, children)` | U23 |
| `selection_field(label, value_label).content(f)` | `select(items, selected).label(label).trigger(view)` | U23 |
| `ColumnDef::with_width` | `ColumnDef::width` | U23 |
| `toaster()`・`announcer()` | `use_toaster()`・`use_announcer()` | U23 |
| `tooltip(label).attach(trigger)`、`tooltip(label).enabled(x).attach(t)` | `tooltip(label, trigger)`、`tooltip(label, t).enabled(x)`（`Tooltip<T>` は `Element`：caller の modifier は trigger に付く。trigger と surface は兄弟。12 章 API-3） | U23・U47 |
| `stepper(..).on_step_press(f)` | `stepper(items, current: Signal<usize>).on_edit(f)`（値は signal で受ける） | U23 |
| `Dialog::mode(DialogMode::..)` | `Dialog::mode(DialogKind::..)`（`Popover` を含む） | U23・U30a |
| `Button::child(x)`（複数回） | `Button::children((x, ..))`（一回） | U23 |
| `Button::default_size`、`ButtonGroupMember::{from, solid, keep_size}` | 消す（group の中の button は group の大きさ。`solid` は variant が `Primary`・`Danger` の button） | U23 |
| `Card::on_press`・`disabled`、`Badge::on_press` などの固有 method | `Element` の同名 method（呼出しは同じ） | U23 |
| `Alert::description`・`Dialog::description`・`EmptyState::description`・`ItemRow::description`・`NavigationMenuLink::description` | `Element::description`（見える説明と accessible description が同じ値） | U23 |
| `x.variant(v)`（builder の固有 method） | `Element::variant(v)`（呼出しは同じ。値は reactive でもよい） | U24 |
| `HoverCard::on_open_change`、`NavigationMenu::on_open_change`、`use_menu_with_open`、`DialogMode::Popover`、`MenuApi::while_open`、`CommandPaletteApi::while_open`、`ComboboxApi::{popup, popup_with_focus_transfer, query_binding, with_input_ref}`、`*_state()` の getter | 3 章の U30a・U30c・U30b・U31 のとおり（`.open(signal)`、`surface(placement, content)`、`Part::state()`） | U30a・U30c・U30b・U31 |
| `Toast::icon(view_fn)`、`Announcer::outlet()` | `Toast::icon_name("name")`、`kivact::ui::announcer_outlet(announcer)` | U32 |
| `ScalarApi::disabled(x)` など | node の `.disabled(x)` | U42 |
| `SwitchPolicy { drag: SwitchDrag(b) }`、`MenuPolicy { press: MenuPress(b), .. }`、`hover_card: HoverCardTiming { .. }` | `drag: b`、`press: b`、`hover_card: Delays { .. }` | U41 |

**部品の module 表**（K08・K11・K13 が引く）：`theme` の行 = `crates/kivact-ui/src/theme.rs`（`kivact::ui::theme`）。`variants` の行 = 部品の variant の enum は各部品の root の file（`button.rs` の `ButtonVariant` など）。K08 の `Slant`・`Striped`・`SyntaxKind` は新しい file `crates/kivact-ui/src/variant_marks.rs`（`Slant`・`Striped`）と K の決めた `syntax.rs` に置き、`lib.rs` に `pub use` を足す。`tabs` の行 = `crates/kivact-ui/src/tabs.rs` と `crates/kivact-ui/src/tabs/`（`tabs/overflow.rs` を足す）。`code_block` の行 = `crates/kivact-ui/src/code_block.rs` と `code_block/`。`sidebar` は `crates/kivact-ui/src/sidebar.rs` と `sidebar/`（`mod.rs` は作らない）。

### 4.5 設計の判断（すべて決定済み）

親が次のとおり決定した。本文はこの決定で書いてある。

- **U-Q1**：`flatten_tree_rows` は公開のまま `#[doc(hidden)]` を付ける（ui の data table が使うため。設計 5.9 の pub(crate) の一覧から外す）。`filter_commands` は `pub(crate)`。
- **U-Q2**：`Variant` の公開の path は `kivact::ui::theme::Variant` だけ（`kivact::view` からは再公開しない。annex C の C10 が `#[doc(hidden)]` にする。INC-22）。
- **U-Q3**：R2〜R6（`Rc<dyn Fn>` の props、`#[caller] caller: Caller` と `Node::authored`、absorb の disabled の OR、`view::Children`、`Variants::with`、型引数そのものの props）は W7a の U25 が kivact-core と kivact-macros に作る（12 章 API-1・API-2・API-4・API-5・ST-09）。
- **U-Q4**：context を子に provide する部品（button_group、toolbar、input_group、item_list、navigation_menu）は `Children` を使う。
- **U-Q5**：状態の型は一つにまとめず、名前も変えない（`kivact::ui::theme` に移すだけ）。
- **U-Q6**：DEC-05 の `.label()` は見えない名前を持つ入力の 12 関数だけ。
- **U-Q7**：W7a は 1 lane（U20 → U21 → U25 → U22 → U23 → U24）、W7b は 4 lane。module は `<name>.rs` と `<name>/look.rs`（設計 5.9 を改めた）。
- 5.10：`AsyncViewTheme`（U46）と `AnnouncerOutletTheme`（U47）を足す。
- 下書きの段階の判断（すべて採用し、各 unit の本文に入れた）：
  - U02〜U04：項目あたりの memo は最大 2 を基本とし、menu の item は 5（U03：press の状態を memo のまま残す。closure にすると press の drag の間 pointer の移動ごとに全行の effect が走るため）、tree の行は 4。`Membership` の通知は user effect から出る。
  - U05：閉じたときに query を消すのは inline（palette）だけ。U06：`FocusReturn::anchor_value` は U30a。U07：retained focus は `ListOptions` に足さず pub(crate) の引数。U09：carousel の machine 化は W8 の U51。U10：tooltip の `attached` の slot は U31 の削除の対象外。
  - U30a：popover の部品を popup に、`surface()` の overlay に name を付けない、`use_tooltip(enabled)`。U30c：NavigationMenu は panel ごとの Api。U30a は大きいので HoverCard と NavigationMenu を U30c に分けた（親の決定）。U30b：`use_combobox(items, selected, query, open, ComboboxOptions)`。U31：getter は 46 個。
  - U32：`Notices::outlet` は U32。U34：`use_date_segments` を `use_date_field`（`DateFieldApi`、`date_field.rs`）に改名し、`use_date_picker` を足す。U35・U43：`calendar/navigation.rs` の `FocusSnapshot` は ui に残す（U52 が `allowed_types` に入れる）。

## 5. 自己点検

1. **採用した所見がすべて unit にある**：UI-01（U31）、UI-02（U02〜U04）、UI-03（U20・U21）、UI-04（規則 1：U32・U33・U34・U35・U43・U51（carousel）、規則 2：U30b・U32、規則 3：U23、規則 4：U41）、UI-05（内部 U05、公開 U30b）、UI-06（U06、`anchor_value` は U30a）、UI-07（U09、machine 化は U51）、UI-08（U07）、UI-09（U08）、UI-10（U07）、UI-11（disclosure・tooltip U30a、scalar U42）、UI-12（U30a・U30c・U30b）、UI-13（U22・U23、`on_open_change` は U30c）、UI-14（U11〜U14・U20・U24・W8）、UI-15（lib.rs の表 U23。prelude は annex C）、UI-16（U09）、UI-17（U36）、UI-18（U09）、UI-19（U23）、UI-20（ui 側 U23、interaction の `pub(crate)` は U30a・U30c・U30b・U31 と U-Q1）、UI-21（性能 U09、recipe の状態 U52）、UI-22（U10）、UI-23（doctest U01。印の削除と検査は annex T）、UI-24（U30a・U30c・U30b の `surface()`、U20 の上書きの形、U31 の (e)）、UI-25（toaster U08、`HoverCardTiming` U41、combobox の偽の binding U30b）。API-13（U23、検査は annex T）、API-14（U23、検査は annex T）、API-17・DEC-05（U22）、API-19（U20 の `theme` module、U23 の `use_toaster`・`use_announcer`）。設計 5.1（W8）、5.3（U23・W8 の props）、5.9（`icon_button` U23・U40、開閉の語彙 U30a・U30c・U30b）、5.10（全部品の `<Name>Theme`：U20・U46・U47）、6.2 PERF-11（U02〜U04）、6.3（machine の移動 U32〜U35・U43、view の除去 U30b・U32）。
2. **設計の判断を実装者に残していない**：判断はすべて 4.5 節のとおり決定済みで、各 unit の本文に入っている。下書きの統合の注記と読み替えの段落は残っていない（各 unit の所有・前提・手順に入れた）。DEC-15・DEC-16 は手順のコードに直接書いてある。
3. **同じ wave の二つの unit が同じファイルを持たない**：W2a・W3a・W4 は lane U だけで順に行う。W7a は lane U1 だけで順に行う（U20 → U21 → U25 → U22 → U23 → U24）。W7b は 2.2 節の表で排他にした（`lib.rs` の `mod`・`pub use` の行だけ「両方残す」）。W8 は lane ごとに部品の module が別（2.3 節・U44〜U54 の表）。W8 で `button_group.rs` を U40 と U44 が持つが、同じ lane の順。`dev/xtask/policy/workspace.toml` の `[ui_machine]` の行は W8 では U52 だけが持つ。U41 は interaction の policy の file と `hover_card.rs`（interaction）を持ち、W8 の他の unit は interaction を持たない（U42 の scalar・splitter、U43 の `reorder_drag.rs` を除く。これらは互いに重ならない）。
4. **kit を直していない**：W7・W8 の unit は kit・catalog・`e2e_kits` を直さない。annex K は 4.3・4.4 節の表で移行する。sidebar・tabs の overflow・`Slant`・`Striped`・`SyntaxKind` は実装しない（annex K の K08・K11〜K13）。
5. **試験を緩めていない**：書き換える試験は期待値を保つ。12 章の対応で足した試験：`component_macro.rs` の 7 つ（U25：wrapper の variant、part の前後の variant、busy と disabled の OR、`has_press`、Func の推論、型引数の props、遅延の子）、`ui_components.rs` の tooltip の 2 つ（U23）、`interaction_edit.rs` の stepper の 2 か所（U23）、`ui_controls.rs`・`ui_theming.rs` の icon_button と busy の 3 つ（U40）、`ui_theming.rs` の別名の試験（U21）、`ui_field.rs`（U45）と `async_tests.rs`（U46）の注釈なしの試験。足す試験：`ui_theming.rs::a_recipe_override_receives_the_previous_recipes_style`（U20）、`ui_field.rs::a_text_input_without_a_name_reports_an_error_and_a_field_names_it`（U22）、`component_macro.rs` の 3 つ（U25）、`ui_controls.rs::an_icon_button_is_named_by_its_label_and_hides_it`（U40）、`ui_sparkline.rs::a_sparkline_recipe_draws_the_line_from_its_state`（U52）、3 章の U36 の text 編集の試験。

### E2E の試験の一覧（この付属書の unit が直す・走らせる file）

`dev/tests/kivact-public-api/tests/` の次の file（W1 の後は T の分割に従う binary の中）：
- ui：`ui_calendar.rs`、`ui_carousel.rs`、`ui_chat_code.rs`、`ui_collections.rs`、`ui_combobox.rs`、`ui_command_palette.rs`、`ui_components.rs`、`ui_controls.rs`、`ui_data_table.rs`、`ui_disclosure_surfaces.rs`、`ui_dropzone.rs`、`ui_field.rs`、`ui_navigation.rs`、`ui_sparkline.rs`、`ui_theming.rs`、`ui_toolbar.rs`（kit の試験 `ui_standard.rs`・`theme_conformance.rs` は `kivact-kit-tests` にあり annex K）
- apg：`apg_accordion.rs`、`apg_alert.rs`、`apg_breadcrumb.rs`、`apg_carousel.rs`、`apg_combobox.rs`、`apg_date_picker.rs`、`apg_listbox.rs`、`apg_menu.rs`、`apg_menu_button.rs`、`apg_menubar.rs`、`apg_radio_group.rs`、`apg_slider.rs`、`apg_spinbutton.rs`、`apg_tabs.rs`、`apg_toolbar.rs`、`apg_tooltip.rs`、`apg_tree_view.rs`、`apg_window_splitter.rs`（kit の部分 `apg_grid_kit.rs`・`presence_kit.rs` は `kivact-kit-tests` にあり annex K）
- interaction：`interaction_calendar.rs`、`interaction_combobox.rs`、`interaction_delayed.rs`、`interaction_dialog.rs`、`interaction_drag_edit.rs`、`interaction_edit.rs`、`interaction_list_nav.rs`、`interaction_menu.rs`、`interaction_palette.rs`、`interaction_select.rs`、`interaction_show_hide.rs`、`interaction_slider.rs`、`interaction_splitter.rs`、`interaction_state.rs`、`interaction_table.rs`、`interaction_tabs.rs`、`interaction_toast.rs`、`interaction_toggle.rs`、`interaction_tooltip.rs`、`interaction_tree.rs`、`interaction_zoom_pan.rs`
- その他（ui の部品を使う）：`components.rs`、`feedback.rs`、`indicators.rs`、`i18n.rs`、`text_editing.rs`、`environment.rs`（`provide_control_size`）、`component_macro.rs`（U25）、`async_tests.rs`
