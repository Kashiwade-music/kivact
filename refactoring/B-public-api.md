# 付録 B: 公開 API の最終形

[設計書](../REFACTORING_DESIGN.md) 10 章の付録。現在の名前は `main` @ `a3536ef`。

- B.1 module ごとの item（正準パス）
- B.2 prelude の固定の一覧
- B.3 移動・改名・削除の表
- B.4 handler の表
- B.5 prop の形の変更の表
- B.6 `Rc`・`Arc`・`Box<dyn>`・`Location` を署名から消す表
- B.7 書き方の例（ガイドと例の新しい形）

---

## B.1 module ごとの item

この表にない公開 item を作らない（足すときは設計書を直す）。kit から上がる部品（付録 C.2）は `ui` と `interaction` の欄に含む。`(新)` は今回足すもの。

### `kivact`（root）

`App`、`Result`、`UiError`、`ErrorKind`

`App` のメソッド: `new(title)`、`window_size`、`min_window_size`、`theme`、`locale`、`messages`、`font`、`fonts(iter)`(新)、`blocking_threads`、`gpu_preference`（feature `gpu-interop`）、`run(root)`、`embed(parent, root)`（feature `embed`）。

### `kivact::state`

| 種類 | item |
|---|---|
| handle | `Signal`、`Value`、`Binding`、`Var`、`Callback`、`Selector`、`Trigger`(新) |
| setup の関数 | `signal`、`memo`、`effect`、`effect_on`、`var`、`binding`、`callback`、`selector`、`trigger`(新)、`untrack`、`on_mount`、`on_cleanup` |
| 文脈 | `provide`、`use_context`、`try_use_context`、`use_app_state` |
| prop の trait | `IntoValue`（文書に出る公開のメソッドは `into_value` だけ。`into_prop` は `constant_value!` と `kivact-style` の impl が実装するので `#[doc(hidden)]` の pub のまま残す）、`IntoText`、`IntoReactive` |
| macro | `constant_value!` |
| 状態機械 | `Machine`、`machine`、`Step`、`MachineHandle`、`MachineCx` |
| macro の内部（文書に出さない） | `__private`（`#[doc(hidden)]` の module。`constant_value!` が展開先の crate から使う `Prop` などを置く。利用者は名前を書かない） |

### `kivact::view`

| 種類 | item |
|---|---|
| 木 | `IntoView`、`View`、`ViewFn`(新)、`IntoChildren`（`ui` から移動）、`Presence`(新、`Entering`/`Present`/`Leaving`) |
| 要素 | `node`、`Node`、`NodeBuilder`、`text`、`rich_text`、`span`、`Span`、`image`、`icon`、`canvas`、`viewport`(新、creative から)、`Viewport`(新)、`ViewportApi`(新)、`column`、`row`、`stack`、`grid`、`scroll`、`Scroll`、`ScrollAxes`、`spacer`、`ImageFit`、`ExitLayout` |
| 制御の流れ | `show`、`Show`、`show_some`、`ShowSome`、`dynamic`、`for_each`、`for_index`、`virtual_list`、`VirtualList`、`RowHeight`、`error_boundary`、`ErrorBoundary`、`Retry`(新) |
| 読み方向 | `ReadingDirection`、`provide_direction`、`use_direction` |
| 参照 | `NodeRef`、`node_ref`、`ScrollBehavior`（`Smooth(Easing)` を足す）、`ScrollState`(新)、`ImageStatus`(新)。`NodeRef` に `presence()`、`snapshot()`、`scroll_state()`、`image_status()` を足す |
| overlay | `overlay`、`overlay_at`、`Overlay`、`OverlayAlign`、`Placement` |
| event | `KeyEvent`、`PointerEvent`、`WheelEvent`、`FileEvent`、`A11yEvent`、`Key`、`KeyCode`、`Modifiers`、`MouseButton`、`PointerButtons`、`PointerId`、`PointerKind`、`PointerPhase`、`ButtonPress`、`FilePhase`、`DismissReason`、`Composition`(新)、`Reorder`(新) |
| handler | `Handled`、`HandlerResult`（一つ。定義は `kivact-runtime`、正準パスはここ）、`IntoAction`(新、設計書 17.6 節) |
| 入力の種類(新) | `InputPurpose`（設計書 17.16 節）。`PointerEvent::capture()`・`release_capture()`(新、17.17 節) |
| 編集 | `EditPhase`（`interaction` から移動） |
| 入力の状態（App 全体。どこでも呼べる） | `modifiers`、`pointer_position` |
| semantics の値 | `Role`、`CheckedState`、`CurrentItem`、`LiveRegion`、`Orientation`、`PopupKind`、`SortDirection`、`A11yAction`、`A11yRequest` |

### `kivact::style`

`Style`、`style`、`Theme`、`ThemeMode`、`ThemeExtension`、`ThemeColor`、`ColorToken`、`Color`、`rgb`、`rgba`、`hsl`、`Length`、`pct`、`em`、`auto`、`Align`、`Justify`、`Direction`、`Position`、`Overflow`、`ScrollbarWidth`、`Track`、`GridAutoFlow`、`GridPlacement`、`Background`、`BorderStyle`、`Cursor`、`FontFeature`、`FontKerning`、`LineHeight`、`TextAlign`、`TextDecorationLine`、`TextLength`、`TextOverflow`、`TextTransform`、`TransitionProp`（`Text`・`Size` を足す）、`StyledRange`(新、`ui::code_block` と `custom::TextEditing::line_styles` が使う、範囲と style の組)、`Display`（`Contents` を足す）、`Visibility`(新)、`TextDecorationStyle`(新)、`ScrollSnap`(新)、`ThemeMode::System`(新)、`Icon`、`IconSet`、`StandardIcon`(新)、`Asset`（root から移動）、`Ext`(新)、`use_theme`、`provide_theme`、`tokens`(新)、`theme::{Palette, Radii, Spacing, Typography}`

`Theme` の host 側のメソッド（`install`、`of`）と `Style::compile`・`merge`・`display_grid`・`with_scroll_axes` は公開しない（設計書 7.6 節）。`grid` の箱は `view::grid` だけ、scroll の軸は `view::scroll(..).axes(..)` だけで作る（二つ目の経路を消す）。

### `kivact::draw`

`Draw`、`Path`（`contains`、`stroke_contains`、`arc_to`（canvas の形）、`svg_arc_to`（旧 `arc_to`）、`parse_svg` を足す）、`PathContour`、`Paint`、`CanvasGradient`、`Gradient`、`GradientExtend`、`RadialExtent`、`linear_gradient`、`radial_gradient`、`conic_gradient`、`BlendMode`、`CornerRadius`、`Filter`、`Shadow`、`Shape`、`Stroke`、`LineCap`、`LineJoin`、`FillRule`、`Layer`、`ImageOptions`、`InlineBox`、`InlineBoxPlacement`、`InlineParagraph`、`IntrinsicSize`、`TextBaseline`、`TextMetrics`、`TextParagraph`、`DrawText`（旧 `TextStyle`）、`RegionId`、`Pixels`（旧 `RgbaImage` も統合）、`Point`、`Rect`、`Size`、`Transform`、`Offscreen`

### `kivact::motion`

`Animatable`、`Animated`、`animated`、`Easing`、`FrameInfo`、`FrameLoop`、`frame_loop`、`FrameRate`、`Repeat`、`Spring`、`Transition`（`retarget(Retarget)` を足す）、`Retarget`(新)

`FrameLoop::while_shown(node_ref)`(新、付録 A の SM-K33)。

### `kivact::task`

`Action`、`ActionPolicy`、`ActionOutcome`(新)、`AsyncState`、`CancelToken`、`Resource`、`Task`、`action`、`blocking`、`resource`、`sleep`、`spawn`、`timeout`

`ActionFn`・`BlockingWork` は署名の境界にだけ現れる sealed の trait として `#[doc(hidden)]` のまま残す（利用者は名前を書かない）。

### `kivact::i18n`

`Locale`、`Messages`、`message`、`message_with`、`interpolate`、`use_locale`、`provide_locale`、`provide_messages`、`CalendarDate`、`CalendarMonth`、`DateRange`、`Weekday`、`TimeOfDay`(新、`{ hour: u8, minute: u8, second: u8 }`)、`DateTime`(新、`{ date: CalendarDate, time: TimeOfDay }`)、`HourCycle`(新、`H12`/`H23`)、`NumberStyle`(新、`{ fraction_digits: FractionDigits, grouping: Grouping }`)、`FractionDigits`(新)、`Grouping`(新)、`DateStyle`(新、`Short`/`Medium`/`Long`/`Iso`)、`TimeStyle`(新、`{ seconds: bool }`)、`NameWidth`(新、`Narrow`/`Short`/`Long`)

`PluralCategory`(新)、`PluralForms`(新)。`Messages::with_plural(key, PluralForms)`(新)。`Locale` の書式のメソッド(新、ICU4X で全 locale。設計書 17.14 節): `plural_category(f64) -> PluralCategory`、`compare(&str, &str) -> Ordering`、 `format_number(f64, NumberStyle) -> String`、`parse_number(&str) -> Option<f64>`、`parse_date(&str) -> Option<CalendarDate>`、`format_date(CalendarDate, DateStyle) -> String`、`format_time(TimeOfDay, TimeStyle) -> String`、`month_name(u8, NameWidth)`、`weekday_name(Weekday, NameWidth)`、`hour_cycle()`、`first_day_of_week()`（既存）。今回の表は en-US と、既存の `first_day_of_week` の表だけ（設計書 16.1 節）。

### `kivact::platform`

| 種類 | item（読み出しは全部 `Value<T>` を返し、どこでも呼べる） |
|---|---|
| 環境 | `window_size`、`scale_factor`、`safe_area`、`keyboard_inset`(新)、`fullscreen`、`reduced_motion`、`surface_active`、`font_families`、`capabilities`、`pointer_settings`(新、`primary: PointerClass` を含む)、`theme_hint`(新)、`is_initial_render` |
| 状態の復元(新) | `save_state(key, bytes)`、`restored_state(key)`、`LifecycleEvent::SaveState` |
| サービス（future を返す） | `copy_text`、`read_text`、`pick_file`、`pick_files`、`pick_folder`、`save_file`、`open_path`、`reveal_path`、`request_window_size`、`set_fullscreen`、`set_reduced_motion` |
| lifecycle | `LifecycleEvent`、`on_lifecycle` |
| 型 | `Capabilities`（`state_restoration` を足す）、`Insets`、`FileDialog`、`PointerSettings`(新)、`PointerClass`(新)、`ThemeHint` |

### `kivact::diagnostics`

`why`、`Why`、`WhyDep`、`WhyProp`、`errors`、`Stats`、`Turn`、`TurnRecord`、`TraceRun`、`TraceWrite`（`ErrorKind` は root へ）

### `kivact::interaction`

| 種類 | item |
|---|---|
| hook と handle | `use_button`→`ButtonApi`、`use_checkbox`→`CheckboxApi`、`use_switch`→`SwitchApi`、`use_toggle_button`→`ToggleButtonApi`、`use_disclosure`→`DisclosureApi`、`use_disclosure_group`(新)→`DisclosureGroupApi`、`use_popup`→`PopupApi`、`use_dialog`→`DialogApi`、`use_tooltip`・`use_tooltip_when`→`TooltipApi`、`use_delayed`→`DelayedApi`、`use_hover_surface`→`HoverSurfaceApi`、`use_hover_corridor`→`HoverCorridorApi`、`use_menu`→`MenuApi`、`use_menubar`→`MenubarApi`、`use_navigation_menu`(新)→`NavigationMenuApi`、`use_select`→`SelectApi`、`use_combobox`→`ComboboxApi`、`use_command_palette`→`CommandPaletteApi`、`use_list`→`ListApi`、`use_tabs`→`TabsApi`、`use_tree`→`TreeApi`、`use_table`→`TableApi`、`use_table_model`→`TableModelApi`、`use_scalar`→`ScalarApi`、`use_range_scalar`→`RangeScalarApi`、`use_splitter`→`SplitterApi`、`use_drag`→`DragApi`、`use_zoom_pan`→`ZoomPanApi`、`use_space_pan`→`SpacePanApi`、`use_text_edit`・`use_text_area`→`TextEditApi`、`use_inline_edit`（`ui` から移動）→`InlineEditApi`、`use_follow_end`→`FollowEndApi`、`use_calendar_grid`→`CalendarGridApi`、`use_carousel`(新)→`CarouselApi`、`use_dropzone`(新)→`DropzoneApi`、`use_copy_feedback`(新)→`CopyFeedbackApi`、`toaster`→`ToasterApi`、`announcer`→`AnnouncerApi` |
| data | `ListItem`、`MenuEntry`、`Command`、`TreeNode`、`Tone`、`Toast`、`ToastOptions`、`ScalarRange`、`ScalarRangeError`、`Selection`(新、旧 `ListSelection`)、`CalendarSelection`、`FileInfo`(新)、`CarouselMode`(新)、`SnapMode`(新)、`HoverCardTiming`(新)、`ListPolicy`、`ListActivation`、`Side`、`Dismiss`、`DialogKind`、`ComboboxMode`(新)、`FilterValue`、`DragUpdate`、`*State`（part の状態の型。ui の slot の closure が受け取るものを含む） |
| policy | `InteractionPolicy`(新、設計書 11.1 節) と field の型 `MenuPress`、`PressDrift`(新)、`SubmenuTiming`、`DisabledMenuFocus`、`TooltipConfig`、`HoverCardTiming`(新) |
| 関数 | `filter_commands`、`flatten_tree_rows`、`text_edit_policy`、`nearest_snap`、`project_release`、`pull_to_snap`、`resist_overshoot` |

### `kivact::ui`

| 種類 | item |
|---|---|
| 標準部品の token(新) | `UiTokens`、`StateTokens`（設計書 17.7 節） |
| 非同期と Tier 3 の部品(新) | `async_view`、`chat_thread`、`chat_message`、`typing_indicator`、`chat_composer`、`code_block`、`stat`、`heading`、`description_list`、`description`、`Trend`、`progress` の `Ring`・`Gauge` の variant（設計書 17.15・17.19 節） |
| 既存の部品 | `button`、`checkbox`、`switch`、`toggle`、`select`、`menu`、`dialog`、`tooltip`、`tabs`、`radio_group`、`slider`、`range_slider`、`splitter`、`split`、`progress`、`disclosure`、`toast_region`、`command_palette`、`tag_input`、`number_input`、`list`、`data_table`、`DataColumn`、`text_input`、`text_area`、`field`、`tree`、`tree_rows`、`tree_rows_with_reveal`（各 builder の型） |
| kit から上がる部品(新) | 付録 C.2 の Tier 1・2（`calendar`、`date_input`、`date_range_input`、`date_time_input`、`time_input`、`carousel`、`dropzone`、`disclosure_group`、`hover_card`、`combobox`、`toolbar`、`toggle_group`、`button_group`、`pagination`、`stepper`、`breadcrumb`、`navigation_menu`、`table`（静的）、`item_list`、`badge`、`avatar`、`avatar_group`、`skeleton`、`spinner`、`empty_state`、`alert`、`card`、`kbd`、`input_group`） |
| 文脈 | `ControlSize`、`provide_control_size` |
| 上がる部品の値(新) | `StepperItem`、`StepState`、`PageItem`、`SkeletonShape`、`TableAlign`、`DrawerSwipe`、`DisclosureMotion` |
| recipe と look | `UiRecipes`、`Recipe`、`*Recipes`（部品ごとの表）、`Look`、`Variant`、`Segmented`(新、標準の variant)、`UiRecipes::filled`(新、埋めた表の名前の一覧) |
| 状態の型 | `ButtonState`、`CheckState`、`FieldState`、… （recipe が受け取るもの） |
| policy | `UiPolicy`(新) と field の型 `ButtonBusyMotion`、`DisabledButtonFocus`、`DialogCenter`、`ProgressMotion`、`SliderTouchMotion`、`SwitchDrag`、`ToastRowMotion`、`ToastSwipe`、`ToastSwipeEdge`、`ToggleLabelReservation`、`DrawerSwipe`(新)、`DisclosureMotion`(新) |

`ui` は `interaction` の型を再公開しない（`Toaster` などは `kivact::interaction` か prelude から使う）。

### `kivact::custom`（旧 `primitive`）

設計書 10.10 節の一覧（17.12 節の `Decoration`、`LineIndex`、`LineRange`、`Node::draw_with_layout` を含む）と、`PartMount`（`Node::on_mount` の引数）、`Owned`（`Owner::store` の戻り値）。`Owner` は core の newtype（現在の `kivact_core::primitive::Owner`）で、公開メソッド: `current()`、`root()`、`is_alive()`(新)、`provide`、`context`、`store`、`report`。`api: &'static str` の引数は `#[track_caller]` で置き換えて消す。

### `kivact::test`（feature `test`、`kivact-test` crate）

`render`、`render_with`、`RenderOptions`（`AppSettings` を包む）、`Screen`、`Element`、`SemanticNode`、`Within`、`Contact`、`ImeText`、`Screenshot`、`LayoutCheck`、`LayoutRule`、`LayoutViolation`、`TestFonts`、`MirrorMode`(新、旧 `WindowMode`)、`gpu_unavailable`、`failure_dir`、`TestParent`（feature `embed`）

`Screen` に足す(新): `evidence(name)`、`assert_layout_clean()`、`script() -> Script`（`kivact::test::Script`。engine の試験 host の台本を包む kivact-test の型で、`respond(kind, f)` などを持つ）。`native_unavailable()`(新、native の window を開けない環境で embed の E2E を skip する理由を返す)。`Element::read_only` を消し、真偽の問い合わせを `is_*` に揃える（`invalid` → `is_invalid`、`password` → `is_password`）。`render_with_handle` は `render_with` に統合する。

### `kivact::embed`、`kivact::gpu_interop`

現在の item のまま。`embed::Parent`、`Embedded`。`gpu_interop` の 21 item。

---

## B.2 prelude の固定の一覧

`kivact::prelude` はこの一覧だけを出す（`kivact/src/prelude.rs` に一つずつ書き、glob の再公開をしない）。入門のガイド（`docs/for-kivact-user/*/getting-started`）が import なしで使う名前の全部。

```rust
pub use crate::{App, UiError};
pub use crate::state::{
    Binding, Callback, IntoText, IntoValue, Signal, Value, Var, binding, callback, effect, effect_on,
    memo, on_cleanup, on_mount, provide, signal, untrack, use_context, var,
};
pub use crate::view::{
    CheckedState, EditPhase, Handled, IntoView, Key, KeyEvent, Modifiers, Node, NodeBuilder, NodeRef,
    Orientation, PointerEvent, Role, ScrollAxes, View, canvas, column, dynamic, error_boundary,
    for_each, for_index, grid, icon, image, node, node_ref, row, scroll, show, show_some, spacer,
    stack, text, virtual_list,
};
pub use crate::style::{
    Align, Color, ColorToken, Justify, Style, Theme, ThemeColor, ThemeMode, pct, provide_theme, rgb,
    style, use_theme,
};
pub use crate::task::{Action, ActionPolicy, AsyncState, Resource, action, blocking, resource, spawn};
pub use crate::i18n::message;
pub use crate::interaction::{ListItem, MenuEntry, Selection};
pub use crate::ui::{
    ButtonVariant, DataColumn, button, checkbox, data_table, dialog, field, list, menu, radio_group,
    select, slider, switch, tabs, text_area, text_input, tooltip,
};
```

`Result` は入れない（利用者の code の `std::result::Result` を隠すため。`main` は `kivact::Result<()>` と書く）。

---

## B.3 移動・改名・削除の表

| 現在 | 再設計後 | 備考 |
|---|---|---|
| `kivact::{Signal, Value, Binding, Var, Callback, Selector}` | `kivact::state::*` | root から消す |
| `kivact::{signal, memo, effect, effect_on, var, binding, untrack, on_mount, on_cleanup}` | `kivact::state::*` | |
| `kivact::Asset` | `kivact::style::Asset`（`image`・`font` の引数の型と同じ module） | |
| `kivact::view::{callback, selector}` | `kivact::state::{callback, selector}` | |
| `kivact::view::constant` | 削除 | literal か `constant_value!` |
| `kivact::view::{context, try_context}` | `kivact::state::{use_context, try_use_context}` | |
| `kivact::view::provide` | `kivact::state::provide` | |
| `kivact::view::{provide_theme}` | `kivact::style::provide_theme` | |
| `kivact::view::{provide_locale, provide_messages}` | `kivact::i18n::*` | |
| `kivact::view::{IntoReactive, IntoText, IntoValue}` | `kivact::state::*` | |
| `kivact::view::Reactive` | `kivact::custom::Reactive` | |
| `kivact::view::constant_value!` | `kivact::state::constant_value!` | |
| `kivact::view::{HandlerResult, KeyHandlerResult}` | `kivact::view::HandlerResult` | 一つにする（設計書 10.4 節） |
| `kivact::view::mouse_position` | `kivact::view::pointer_position` | `Value<Option<Point>>`、どこでも呼べる |
| `kivact::view::current_modifiers` | `kivact::view::modifiers` | `Value<Modifiers>` を返す。どこでも呼べる |
| `kivact::view::current_direction` | `kivact::view::use_direction` | `Value<ReadingDirection>` を返す |
| `kivact::control::*` | `kivact::view::*` | module を消す |
| `kivact::primitive::*` | `kivact::custom::*` | module を消す |
| `kivact::primitive::NodeExt` | 削除 | `Node` の固有メソッドと `NodeBuilder`（設計書 7.4 節） |
| `kivact::primitive::{Part, PartState, part, PartMount}` | `kivact::custom::{Part, PartState, part}`、`PartMount` は `Node::on_mount` の引数 | |
| `kivact::primitive::{app_state}` | `kivact::state::use_app_state` | |
| `kivact::primitive::TextParagraph` | 削除（`kivact::draw::TextParagraph` だけ） | |
| `kivact::style::current_theme` | `kivact::style::use_theme` | `Value<Theme>` を返す |
| `kivact::draw::TextStyle` | `kivact::draw::DrawText` | 名前の衝突を避ける |
| `kivact::draw::RgbaImage` | `kivact::draw::Pixels` | 一つの画素の型 |
| `kivact::diagnostics::ErrorKind` | `kivact::ErrorKind` | |
| `kivact::i18n::locale` | `kivact::i18n::use_locale` | |
| `kivact::interaction::{Combobox, Disclosure, Popup, List, Table, Tree, Menubar, CommandPalette, Scalar, RangeScalar, Drag, ZoomPan, TextEdit, Delayed, FollowEnd, SpacePan, CalendarGrid, Toaster, Announcer}` | `*Api` を付ける | |
| `kivact::interaction::{use_content_reveal, ContentReveal}` | 削除 | 出現と退場は engine の presence（設計書 17.10 節）。`NodeRef::presence()` と `Style::enter_size`・`exit_size` |
| `kivact::interaction::Toggle` | `SwitchApi`、`ToggleButtonApi` | `use_switch`・`use_toggle_button` が返す |
| `kivact::interaction::ListSelection` | `kivact::interaction::Selection` | `From` で signal から作る |
| `kivact::interaction::IntoScalarRange` | 削除 | `impl IntoValue<ScalarRange>` |
| `kivact::interaction::EditPhase` | `kivact::view::EditPhase` | |
| `kivact::interaction::TooltipConfig`（文脈） | `kivact::interaction::InteractionPolicy` の field | |
| interaction の theme 拡張（`MenuPress`、`SubmenuTiming`、`DisabledMenuFocus`） | `kivact::interaction::InteractionPolicy` の field | |
| ui の theme 拡張（`UiRecipes` を除く 11 型） | `kivact::ui::UiPolicy` の field、または recipe の表 | 設計書 11.1 節 |
| `kivact::view::OverlayViewport` | `Theme` の `overlay_margin` | |
| `kivact::ui::{FilterValue, Command, DialogKind, Dismiss, EditPhase, ListActivation, ListItem, MenuEntry, ScalarRange, ScalarRangeError, Side, Toast, ToastOptions, Toaster, Tone, TreeNode, toaster}` | 再公開を削除 | 正準パスは `interaction`（`EditPhase` は `view`） |
| `kivact::ui::use_inline_edit` | `kivact::interaction::use_inline_edit` | behavior の層 |
| `kivact::ui::IntoChildren` | `kivact::view::IntoChildren` | |
| `kivact::ui::Menu::at_least_trigger_width(bool)` | `at_least_anchor_width()` | `Overlay` と揃える |
| `kivact::ui::Disclosure::animated(bool)` | 削除 | `UiPolicy` |
| `kivact::ui::Menu::behavior()` | 削除 | private の alias が公開されていた |
| `kivact::ui::{Button, TextInput}::into_node` | 削除 | `IntoView` で足りる |
| `kivact::ui::DataColumn::{text_of, cell_of, has_cell, is_sortable, is_filtered, is_numeric, width_value, definition}` | `pub(crate)` | builder の data の読み出しは内部 |
| `kivact::test::{WindowMode}` | `kivact::test::MirrorMode` | 設計書 9.3 節 |
| `kivact::test::render_with_handle` | `render_with` に統合 | |
| `Signal::id()` | 削除 | |
| `IntoValue::into_prop`、`IntoText::into_text`、`Prop` | `#[doc(hidden)]` にする。`Prop` は `kivact::state::__private::Prop` だけから届く | `constant_value!` が利用者の crate で展開され、`kivact-style` も `IntoValue` を実装するので、sealed の trait にはできない |
| `Transition::as_wire` | 削除 | |
| `KeyEvent.region`（field） | `KeyEvent::region()`（メソッド） | `PointerEvent` と揃える |
| `Callback::call_if_alive`（hidden） | `pub(crate)` | |
| `F64Signal`（private module の pub trait） | 診断の `#[diagnostic::on_unimplemented]` で置き換えて削除 | |

---

## B.4 handler の表

設計書 10.4 節の語彙を全部品に当てる。戻り値は全部 `R: HandlerResult`。

| 型 | 再設計後の handler | 現在からの変更 |
|---|---|---|
| `Node`・`NodeBuilder` | `on_press(impl IntoAction<M>)`、`on_key(Fn(KeyEvent))`、`on_key_up(Fn(KeyEvent))`、`on_pointer(Fn(PointerEvent))`、`on_wheel(Fn(WheelEvent))`、`on_file(Fn(FileEvent))`、`on_exit_complete(impl IntoAction<M>)` | payload のない handler は closure と `Callback<()>` の両方を受ける（設計書 17.6 節） |
| `Node`（作者向けの固有） | `on_key_default`、`on_pointer_default`、`on_wheel_default`、`on_ime(Fn(ImeEvent))`、`a11y_action(action, Fn(A11yEvent))`、`on_mount(Fn(PartMount))` | `on_a11y`・`on_a11y_event` を `a11y_action` に。`Part::on_bind` を `Node::on_mount` に |
| `image` の `Node` | `on_image_error(Fn(UiError))` | `&str` を `UiError`（`ErrorKind::Asset`）に |
| `Overlay` | `on_dismiss(Fn(DismissReason))`、`on_dismiss_default(Fn(DismissReason))` | `on_dismiss_after` を改名 |
| `ErrorBoundary` | `on_error(Fn(UiError) -> R)`、`fallback(Fn(UiError, Retry) -> V)` | `-> R`、`Rc<dyn Fn()>` を `Retry` に |
| `Animated` | `on_complete(Fn())` | 変わらない |
| `Action` | `on_complete(Fn(ActionOutcome<T>))`（builder、`self -> Self`） | `on_done(&self, Fn(&Result<..>))` から |
| `ToastOptions` | `on_hide(Fn())` | `-> R` |
| `TextEditApi`、`ui::TextInput` | `on_submit(Fn())`、`on_composition(Fn(Composition))`、`on_edit(Fn(EditPhase))`、`draft(on_commit: Fn(String))` → `on_draft_commit(Fn(String))` | `on_composition_change(Fn(bool))` を改名。位置引数の `draft` を handler に |
| `custom::TextEditing` | `on_text_change(Fn(TextEditEvent))`、`on_selection(Fn(TextSession))` | `on_edit(TextEditEvent)` は `EditPhase` の `on_edit` と衝突するので改名 |
| `SelectApi`、`ui::Select` | `on_preview(Fn(Option<K>))`、`on_edit(Fn(EditPhase))`(新) | `on_edit` を足す |
| `MenuApi`、`ui::Menu` | `on_activate(Fn(K))` | 位置引数の `choose` から |
| `CommandPaletteApi`、`ui::CommandPalette` | `on_activate(Fn(K))` | 位置引数の `run` から |
| `TreeApi`、`ui::TreeView` | `on_activate(Fn(K))` | 変わらない（`ui::TreeView` は `-> R` を足す） |
| `ui::TagInput` | `on_add(Fn(String))`、`on_remove(Fn(usize))`、`on_edit(Fn(EditPhase))`(新) | 位置引数から |
| `ui::DataTable` | `on_reorder(Fn(Reorder<K>))` | tuple の引数と `-> T` から |
| `ui::Dropzone` | `on_files(Fn(Vec<FileInfo>))`、`validate(Fn(&FileInfo) -> Option<String>)`（validator。`field` の `error` と同じく、`Some` が利用者に見せる error の文） | 新しい部品 |
| `ListApi` | `leave_to(Fn(&K) -> Option<K>)` | `on_leave` を改名（event ではない） |
| `CalendarPolicy` | `on_activate(Fn(CalendarDate))` | `on_commit` を改名 |
| `DragApi` | `on_start`、`on_update`、`on_end`（`Fn(DragUpdate<T>)`） | 変わらない |
| 値を書く behavior と部品（`EditApi`、`ScalarApi`、`RangeScalarApi`、`SplitterApi`、`ZoomPanApi`、`CheckboxApi`(新)、`SwitchApi`(新)、`ToggleButtonApi`(新)、`TabsApi`(新)、`ui` の対応する全部品） | `on_edit(Fn(EditPhase))` | 設計書 11.2 節 |
| `platform` | `on_lifecycle(Fn(LifecycleEvent))` | 変わらない |
| `state` | `on_mount(FnOnce())`、`on_cleanup(FnOnce())` | 変わらない（lifecycle の登録で、`FnOnce` を xtask の例外に列挙する） |

---

## B.5 prop の形の変更の表

| 現在 | 再設計後 |
|---|---|
| `NodeBuilder::hit_testable(bool)` | `hit_testable(impl IntoValue<bool>)` |
| `NodeBuilder::focusable_when_disabled(bool)` | `focusable_when_disabled(impl IntoValue<bool>)` |
| `Part::modal(bool)`、`Overlay::modal(bool)` | `modal()`（構造の option） |
| `Overlay::keep_anchor_clear(bool)`、`at_least_anchor_width(bool)` | `keep_anchor_clear()`、`at_least_anchor_width()` |
| `Overlay::offset(impl Into<f64>)`、`viewport_margin(f32)` | 両方 `impl Into<f64>` |
| `VirtualList::overscan(f32)`、`keep_focused(bool)`（既定 true） | `overscan(impl Into<f64>)`、`release_offscreen_focus()`（現在の `keep_focused(false)`） |
| `ImeSession::password(bool)`、`TextEditing::password`、`TextEditApi::password`、`ui::TextInput::password` | 四つとも `purpose(impl IntoValue<InputPurpose>)`（`InputPurpose::Password`。設計書 17.16 節） |
| `TextEditing::multiline(bool)`、`report_text(bool)` | `multiline()`、`report_text()` |
| `Span::italic(bool)` | `italic()` |
| `span(impl Into<String>)` | `span(impl IntoText)` |
| `Span::font_size(impl Into<f64>)`、`weight(u16)`、`color(impl ThemeColor)` | 変わらない（API-4 の整数の概念: weight） |
| `NodeBuilder::test_id(impl AsRef<str>)` | `test_id(impl Into<String>)` |
| `Theme::with_icon(impl Into<Cow<'static, str>>, ..)`、`Icon::named(..)` | `impl Into<String>` |
| `Theme::{with_focus_ring, with_focus_ring_offset, with_disabled_opacity, with_control_height}(f32)` | `impl Into<f64>` |
| `Color::with_alpha(f32)`、`Style::opacity(f32)` | `impl Into<f64>` |
| `Measurement::baseline(f32)` | `impl Into<f64>` |
| `ui::TextInput::wrap(bool)`、`TextEditApi::wrap(bool)` | `wrap(impl IntoValue<bool>)` |
| `ui::Menu::offset(f32)`、`ui::Tooltip::offset(f32)` | `impl Into<f64>` |
| `ui::Select::open_on_arrow(bool)`、`retain_trigger_focus(bool)`（ui と interaction） | `impl IntoValue<bool>` |
| `ui::Disclosure::keep_mounted(bool)`、`ui::Tabs::keep_mounted(bool)` | `keep_mounted()` |
| `ui::DataTable::tree_expand_all(bool)` | `expand_all()` |
| `ui::ToastRegion::hold_all_on_hover(bool)` | `impl IntoValue<bool>` |
| `ui::split(.., sizes: Binding<Vec<f32>>)`、`Split::min_size(f32)` | `Binding<Vec<f64>>`、`impl Into<f64>` |
| `ui::DataColumn::{width, flex, min_width}(f32)` | `impl Into<f64>`（`DataColumn` は record なので plain な値のまま。`numeric(bool)`・`resizable(bool)` も record の例外） |
| `ui::number_input(.., range: ScalarRange)`、`slider.range(impl IntoScalarRange)`、`splitter(.., impl IntoScalarRange)` | 全部 `impl IntoValue<ScalarRange>`（`0.0..=1.0` と `0..=100` が書ける） |
| 数値の reactive な prop（`impl IntoValue<f64>`） | `4`・`4.0`・`4.0_f32` が書ける（`IntoValue<f64>` を `i32`・`u32`・`f32` に足す） |
| `ui::List::multiple(impl Into<Binding<Vec<K>>>)`、`ui::tree(.., ListSelection<K>)` | `.selection(impl Into<Selection<K>>)` |
| `ui::disclosure(title, open, content)` | `disclosure(title, open).content(..)` |
| `motion::Transition::decorative(bool)`（既定 true） | `essential()`（現在の `decorative(false)`） |
| `kivact::style::theme` の kit の拡張 trait（`ThemeOptions`） | kit の `theme_with(mode, ThemeOptions)`（付録 C.1） |

---

## B.6 署名から消す `Rc`・`Arc`・`Box<dyn>`・`Location`

| 現在 | 再設計後 |
|---|---|
| `ErrorBoundary::fallback(Fn(UiError, Rc<dyn Fn()>))` | `Fn(UiError, Retry)` |
| `NodeExt::accessible_name() -> Option<Rc<dyn Fn() -> String>>` | `Node::accessible_name() -> Option<Value<String>>` |
| `Owner::store(name, value: Rc<T>)` | `Owner::store(name, value: T) -> Owned<T>` |
| `Theme::get::<T>() -> Arc<T>` | `Theme::get::<T>() -> Ext<T>`。`pub struct Ext<T>(Arc<T>)`（`kivact::style`）、`Clone`、`Deref<Target = T>`、`T: Clone` のとき `to_owned(&self) -> T`。kit は `kivact::style::tokens::<T>() -> Ext<T>` を使う |
| `pub type Recipe<S> = Arc<dyn Fn(&Theme, &Look<S>) -> Style + Send + Sync>` | `pub struct Recipe<S>`（`From<impl Fn(&Theme, &Look<S>) -> Style + Send + Sync + 'static>`、`Clone`） |
| `UiRecipes::clone(&theme.get::<UiRecipes>())` | `theme.get::<UiRecipes>().to_owned()`（`Ext<T>` の固有メソッドで、`T` を返す。std の `ToOwned` は実装しない） |
| `TestFonts::Only(Vec<Arc<[u8]>>)` | `TestFonts::Only(Vec<Asset>)` |
| `raise(kind, message, at: &'static Location<'static>) -> !` | `#[track_caller] raise(kind, message) -> !` |
| `NodeExt::at(&'static Location)`、`Owner::report(kind, message, at)`、`Owned::get_at(at)`、`NodeRefExt::tracked_node(at)` | `#[track_caller]` の同名のメソッド（`at` の引数を消す）。`NodeExt::at(..)`（報告する位置の付け替え）は `Node::located_at_caller()`（引数なし、`#[track_caller]`）にする |
| `Owner::current(api: &'static str)`、`Owner::root(api)`、`app_state(api, init)` | `#[track_caller]`、引数の `api` を消す |

---

## B.7 書き方の例

ガイドと例は、ここに示す形で書き直す（作業手順書の F-1）。どれも `use kivact::prelude::*;` だけで書ける（`kivact::state` の状態機械の例を除く）。data の constructor（`ListItem::new`、`MenuEntry::item` など）の名前は、作業の時点のものに合わせる（この付録は形を示すもので、constructor の名前は決めない）。

### counter（変わらない）

```rust
use kivact::prelude::*;

fn app() -> impl IntoView {
    let count = signal(0);
    let doubled = memo(move || count.get() * 2);
    column((
        text(move || format!("Count: {}", count.get())),
        text(move || format!("Doubled: {}", doubled.get())),
        button("Increment").on_press(move || count.update(|n| *n += 1)),
    ))
    .style(style().padding(24).gap(12))
}
```

### props・子・callback（`Callback` と `Binding` が prelude にある）

```rust
use kivact::prelude::*;

fn stat(label: impl IntoText, value: impl IntoValue<i32>) -> impl IntoView {
    let value = value.into_value();
    row((text(label), text(move || value.get().to_string()))).style(style().gap(8))
}

// 二つの場所で使う callback は `callback(..)` で Copy にする（`impl Fn() + Copy` を教えない）。
fn toolbar(on_add: Callback<()>) -> impl IntoView {
    row((button("Add").on_press(move || on_add.call(())), button("Add again").on_press(move || on_add.call(()))))
}

// 一度だけ mount する子は値（impl IntoView）。
fn panel(title: impl IntoText, child: impl IntoView) -> impl IntoView {
    column((text(title), child)).style(style().gap(8).padding(12))
}
```

### 選択（`Selection` が signal の型で single と multiple を選ぶ）

```rust
use kivact::prelude::*;

fn files() -> impl IntoView {
    let items = signal(vec![ListItem::new(1, "a.txt"), ListItem::new(2, "b.txt")]);
    let one = signal(None::<u32>);
    let many = signal(Vec::<u32>::new());
    column((list(items).selection(one), list(items).selection(many)))
}
```

### menu と palette（位置引数の callback がない）

```rust
use kivact::prelude::*;

fn file_menu() -> impl IntoView {
    menu("File", vec![MenuEntry::item("open", "Open"), MenuEntry::item("save", "Save")])
        .on_activate(|key: &'static str| tracing::info!(key, "menu"))
}
```

### 状態機械で独自の部品を書く

```rust
use kivact::prelude::*;
use kivact::state::{Machine, MachineCx, Step, machine};
use std::time::Duration;

#[derive(Clone, PartialEq)]
enum CopyState { Idle, Copied }
enum CopyEvent { Pressed, Expired }
enum CopyEffect { Write, StartTimer }
#[derive(Clone, PartialEq, Eq, Hash)]
enum CopyTimer { Reset }
struct CopyInput { text: String }

struct CopyMachine;
impl Machine for CopyMachine {
    type State = CopyState;
    type Event = CopyEvent;
    type Effect = CopyEffect;
    type Input = CopyInput;
    type Timer = CopyTimer;
    fn transition(state: &CopyState, event: CopyEvent, _input: &CopyInput) -> Step<CopyState, CopyEffect> {
        match (state, event) {
            (_, CopyEvent::Pressed) => Step::to(CopyState::Copied).with(CopyEffect::Write).with(CopyEffect::StartTimer),
            (CopyState::Copied, CopyEvent::Expired) => Step::to(CopyState::Idle),
            (CopyState::Idle, CopyEvent::Expired) => Step::stay(),
        }
    }
}

fn copy_button(text: impl IntoValue<String>) -> impl IntoView {
    let text = text.into_value();
    let m = machine::<CopyMachine>(
        CopyState::Idle,
        move || CopyInput { text: text.get_untracked() },
        move |cx: MachineCx<CopyMachine>, effect| match effect {
            CopyEffect::Write => { spawn(kivact::platform::copy_text(text.get_untracked())); }
            CopyEffect::StartTimer => cx.after(CopyTimer::Reset, Duration::from_secs(2), CopyEvent::Expired),
        },
    );
    button(move || if m.get() == CopyState::Copied { "Copied" } else { "Copy" })
        .on_press(move || m.send(CopyEvent::Pressed))
}
```

（この例は書き方を示すためのもので、実際の copy の feedback は `kivact::interaction::use_copy_feedback` を使う。）

### 独自の control（`Node` の固有メソッドで semantics を書き、`NodeExt` の import が要らない）

```rust
use kivact::prelude::*;
use kivact::interaction::use_switch;

pub fn my_switch(label: impl IntoText, on: impl Into<Binding<bool>>) -> impl IntoView {
    let api = use_switch(on);
    node()
        .part(api.control())          // role・checked・key・press は part が当てる
        .child(text(label))
        .style(style().padding(4))
}
```
