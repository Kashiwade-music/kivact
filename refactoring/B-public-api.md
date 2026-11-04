# 付録 B: 公開 API の最終形

2026-11-05 改訂: B.8 と 18 章を B.1〜B.6 に反映し、B.10 に旧→新の一覧を置いた。設計書 18 章と食い違ったら 18 章に従う。

[設計書](../REFACTORING_DESIGN.md) 10 章と 18 章の付録。現在の名前は `main` @ `a3536ef`（W1 の V-1a〜V-6 の後の code で確かめた）。B.1〜B.6 が最終の名前と path で、B.10 が旧→新の機械的な置き換えの表である。B.10 にない item の名前と path は変えない。unit の ID は 2026-11-05 の作業手順書の新しい ID（N-D1・N-D2・N-F1〜N-F6・N-G1〜N-G5、I-4a、K-1a など）。

- B.1 module ごとの item（正準パス）
- B.2 prelude の固定の一覧
- B.3 移動・改名・削除の注意（行は B.10）
- B.4 handler の表
- B.5 prop の形の変更の表
- B.6 `Rc`・`Arc`・`Box<dyn>`・`Location` を署名から消す表
- B.7 書き方の例（ガイドと例の新しい形）
- B.8 （B.1〜B.6 と B.10 に畳んだ）
- B.9 slot の種類の表
- B.10 旧→新の一覧（機械的な置き換えの表）

---

## B.1 module ごとの item

この表にない公開 item を作らない（足すときは設計書を直す）。kit から上がる部品（付録 C.2）は `ui` と `interaction` の欄に含む。`(新)` は今回足すもの。ここに書いた名前は最終の名前で、書いた module が正準パスである。表に載せていない現在の公開 item（`DelayedPhase`、`DragCoordinates`、`ColumnDef` など）は、B.10 に行がなければ名前と path を変えない。

### `kivact`（root）

`App`、`Result`、`UiError`、`ErrorKind`（`diagnostics` から移動）

`App` のメソッド: `new(title)`、`window_size(width, height)`、`min_window_size(width, height)`、`theme(impl IntoValue<Theme>)`（reactive。root の owner の下で評価する。設計書 18.11 節）、`locale`、`messages`、`font(impl Into<Asset>)`（`#[track_caller]`。呼んだ位置を font と一緒に保存し、font が拒まれたときの報告に使う）、`blocking_threads(n)`（呼ばないときは既定の数）、`gpu_preference`（feature `gpu-interop`）、`run(root)`、`embed(parent, root)`（feature `embed`）。

`App` は `kivact_core::launch::AppOptions`（field は全部 private。facade は再公開しない。`blocking_threads` は `Option<usize>`、font は `(Asset, &'static Location)` の列で持つ）を一つ持ち、各メソッドは `AppOptions` の同名のメソッドを呼ぶ（`AppOptions::theme` も `impl IntoValue<Theme>` を受ける）。`fonts(iter)` は作らない（`font` を繰り返し呼ぶ。`test::RenderOptions::fonts(TestFonts)` と名前が衝突するため）。

### `kivact::state`

| 種類 | item |
|---|---|
| handle | `Signal`、`Value`、`Binding`、`Var`、`Callback`、`Selector`、`Notifier`(新、`notify()`・`track()`) |
| setup の関数 | `signal`、`memo`、`effect`、`effect_on`、`var`、`binding`、`callback`、`selector`、`notifier`(新)、`untrack`、`on_mount`、`on_cleanup` |
| 文脈 | `provide`、`use_context`、`try_use_context`、`use_app_state` |
| prop と binding の trait | `IntoValue`（文書に出る公開のメソッドは `into_value` だけ。`into_prop` は `constant_value!` と `kivact-style` の impl が実装するので `#[doc(hidden)]` の pub のまま残す）、`IntoText`（読み出しは `into_text_value`。`into_text` は `#[doc(hidden)]`）、`IntoReactive`、`IntoBinding`(新、sealed。impl は `Signal<T>` と `Binding<T>`)、`ItemKey`(新、`Clone + Eq + Hash + 'static` の blanket の supertrait)、`PropValue`(新、`Clone + PartialEq + 'static`) |
| macro | `constant_value!`、`bind!`(新、`bind!(x.a.b)` は `x.project(|v| &v.a.b, |v| &mut v.a.b)` に展開し `Binding<U>` を返す) |
| handle のメソッド | `Signal::project(get_ref, get_mut) -> Binding<U>`・`Binding::project`（旧 `field`）、`Binding::optional`（残す）、`Callback::to_fn`（残す）。`From<F: Fn() -> R> for Callback<()>`(新、設計書 18.2 節) |
| 状態機械 | `Machine`（関連型は `State`・`Event`・`Effect` の三つ。`fn transition(&self, state: &Self::State, event: Self::Event) -> Step<Self::State, Self::Event, Self::Effect>`。設計書 18.13 節）、`machine(initial, input: impl Fn() -> M, run: impl Fn(MachineCx<M>, M::Effect))`、`Step<S, Ev, Ef>`（`stay`、`to`、`with`、`after(delay, event)`、`cancel(event)`）、`MachineHandle`（`send`、`state`、`get`、`feed`）、`MachineCx`（`send`） |
| macro の内部（文書に出さない） | `__private`（`#[doc(hidden)]` の module。`constant_value!` が展開先の crate から使う `Prop` などを置く。利用者は名前を書かない） |

### `kivact::view`

| 種類 | item |
|---|---|
| 木 | `IntoView`（`Fn() -> R where R: Into<String>` の closure にも実装する。設計書 18.7 節）、`View`、`ViewFn`(新)、`IntoViewFn`(新、sealed、`Fn() -> V` の blanket)、`Component`(新、`IntoView + NodeBuilder` の blanket)、`IntoChildren`（`ui` から移動）、`ListSource`(新、`for_each` の source。行の型は関連型 `Row`)、`Presence`(新、`Entering`/`Present`/`Exiting`) |
| 要素 | `node`、`Node`、`NodeBuilder`（公開のまま。sealed にしない: kit が約 190 の builder で実装する。必須のメソッド `node_mut` は `#[doc(hidden)]`、`with_node` を足す）、`text`、`rich_text`、`span`、`Span`、`image`、`icon`、`canvas`、`viewport`(新、creative から)、`Viewport`(新)、`column`、`row`、`stack`、`grid`、`scroll`、`Scroll`、`ScrollAxes`、`spacer`、`ImageFit`、`ExitLayout`、`Asset`（root から移動。`image`・`App::font`・`avatar.src` が使う） |
| 制御の流れ | `show`、`Show`、`show_some`、`ShowSome`、`dynamic`、`for_each`、`for_index`、`virtual_list`、`VirtualList`、`RowHeight`、`error_boundary`、`ErrorBoundary`、`Retry`(新、Copy の handle、`run()`) |
| 読み方向 | `ReadingDirection`、`provide_direction`、`use_direction` |
| 参照 | `NodeRef`、`node_ref`、`ScrollBehavior`（`Smooth(Easing)` を足す）、`ScrollState`(新)、`ImageStatus`(新)。`NodeRef` に `presence()`、`snapshot()`、`scroll_state()`、`image_status()` を足す |
| overlay | `overlay`、`overlay_at`、`Overlay`、`OverlayAlign`、`Placement` |
| event | `KeyEvent`、`PointerEvent`、`WheelEvent`、`FileEvent`、`A11yEvent`、`Key`、`KeyCode`、`Modifiers`、`MouseButton`、`PointerButtons`、`PointerId`、`PointerKind`、`PointerPhase`、`ButtonPress`、`FilePhase`、`DismissReason`（`Timeout` を足す）、`Composition`(新、`Idle`/`Composing`) |
| handler | `Handled`、`HandlerResult`（一つ。定義は `kivact-runtime`、正準パスはここ） |
| 入力の種類(新) | `InputPurpose`（設計書 17.16 節）。`PointerEvent::capture()`・`release_capture()`(新、17.17 節) |
| 編集 | `EditPhase`（`interaction` から移動） |
| 入力の状態（App 全体。どこでも呼べる） | `modifiers() -> Value<Modifiers>`、`pointer_position() -> Value<Option<Point>>` |
| semantics の値 | `Role`、`CheckedState`、`CurrentItem`、`LiveRegion`、`Orientation`、`PopupKind`、`SortDirection`、`A11yAction`、`A11yRequest` |

`Viewport` は `.fit(impl IntoValue<ImageFit>)`、`.gpu_frame(..)`（feature `gpu-interop`）、`.transform(ZoomPanApi)` を持つ。座標の変換は `ZoomPanApi::to_content`・`to_view`（変換を持つ側。`ViewportApi` は作らない）。

### `kivact::geometry`(新)

`Point`、`Size`、`Rect`、`Insets`、`Edge`（`kivact-types` の再公開。field は `f64`。`Edge` は旧 `interaction::Side` と `ui::ToastSwipeEdge` を一つにした `Left`/`Right`/`Top`/`Bottom`）

### `kivact::style`

`Style`、`style`、`Theme`、`ThemeMode`（`System` は作らない）、`ThemeExtension`、`ThemeRef`(新、`pub struct ThemeRef<T>(Arc<T>)`、`Clone`、`Deref<Target = T>`、`T: Clone` のとき `to_owned(&self) -> T`)、`ThemeColor`、`ColorToken`、`Color`、`rgb`、`rgba`、`hsl`、`Length`、`pct`、`em`、`auto`、`Align`、`Justify`、`Direction`、`Position`、`Overflow`、`ScrollbarWidth`、`Track`、`GridAutoFlow`、`GridPlacement`、`Background`、`BorderStyle`、`Cursor`、`FontFeature`、`FontKerning`、`LineHeight`、`TextAlign`、`TextDecorationLine`、`TextLength`、`TextOverflow`、`TextTransform`、`TransitionProp`（`Text`・`Size` を足す）、`StyledRange`(新、`ui::code_block` と `custom::TextEditing::line_styles` が使う、範囲と style の組)、`Display`(新、`Flex`/`Grid`/`Contents`)、`Visibility`(新)、`TextDecorationStyle`(新)、`ScrollSnap`(新)、`Icon`、`IconSet`、`StandardIcon`(新)、`use_theme`、`provide_theme`、`use_tokens`(新)、`theme::{Palette, Radii, Spacing, Typography}`

`Style` のメソッド: `merge`（公開のまま。kit が recipe を合わせるのに使う）、`display(Display)`(新、`display_grid` を置き換える)、`enter_size(Orientation)`・`exit_size(Orientation)`(新、設計書 17.10 節。`Axis` は作らない)、`layout_transition(Transition)`・`visibility(Visibility)`・`scroll_snap(ScrollSnap)`(新、17.18 節)。`Theme` のメソッド: `ext::<T>() -> ThemeRef<T>`（旧 `get`）、`overlay_margin`(新、旧 `view::OverlayViewport`)。`From<ThemeHint> for ThemeMode`(新)。`use_tokens::<T>() -> ThemeRef<T>`。

host 側の口（`Theme::install`、`Theme::of`、`Style::compile`、span の style の `compile`、`Draw::record`、`Style::with_scroll_axes`）は公開しない: `kivact_style::internal`（`#[doc(hidden)]` の pub module、`kivact-core` だけが使う）の自由関数 `install_theme`、`theme_of`、`compile_style`、`compile_span_style`、`record`、`with_scroll_axes` にし、同名の固有メソッドを消す。facade はこの module を再公開しない。scroll の軸は `view::scroll(..).axes(..)` だけで作る。

### `kivact::draw`

`Draw`、`Path`（`contains`、`stroke_contains`、`arc_to`（canvas の形）、`svg_arc_to`（旧 `arc_to`）、`parse_svg` を足す）、`PathContour`、`Paint`、`CanvasGradient`、`Gradient`、`GradientExtend`、`RadialExtent`、`linear_gradient`、`radial_gradient`、`conic_gradient`、`BlendMode`、`CornerRadius`、`Filter`、`Shadow`、`Shape`、`Stroke`、`LineCap`、`LineJoin`、`FillRule`、`Layer`、`ImageOptions`、`InlineBox`、`InlineBoxPlacement`、`InlineParagraph`、`IntrinsicSize`、`TextBaseline`、`TextMetrics`、`TextParagraph`、`TextRun`、`text_run`、`TextFormat`（旧 `TextStyle`）、`RegionId`、`Pixels`（旧 `RgbaImage` も統合）、`Transform`、`Offscreen`

（`Point`、`Rect`、`Size` は `kivact::geometry` へ移る。）

### `kivact::motion`

`Animatable`、`Animated`、`animated`、`Easing`、`FrameInfo`、`FrameLoop`、`frame_loop`、`FrameRate`、`Repeat`、`Spring`、`Transition`（`kivact_types::motion::Transition` の再公開。`essential()`、`retarget(Retarget)` を足す）、`Retarget`(新)

`FrameLoop::while_present(node_ref)`(新、付録 A の SM-K33)。

### `kivact::task`

`Action`、`ActionMode`（旧 `ActionPolicy`）、`ActionOutcome`(新)、`AsyncState`、`AsyncSource`(新、sealed。`Resource<T>`（retry = `refetch`）と `Action<T, I>`（retry = `retry`）が実装する)、`CancelToken`、`Resource`、`Task`、`action(mode, run)`、`blocking`、`resource`、`sleep`、`spawn`、`timeout`、`on_tokio`(新、feature `tokio`)

`#[derive(Clone)] pub enum ActionOutcome<T> { Succeeded(T), Failed(UiError) }`。完了通知だけを表し、キャンセル時の通知は既存どおり発生させない。

`ActionFn`・`BlockingWork` は署名の境界にだけ現れる sealed の trait として `#[doc(hidden)]` のまま残す（利用者は名前を書かない）。`IntoValue<AsyncState<T>> for Resource`・`Action` は作らない（設計書 18.5 節）。

### `kivact::i18n`

`Locale`、`Messages`、`message`、`message_with`、`interpolate`、`use_locale`、`provide_locale`、`provide_messages`、`use_messages`(新、API-7 の対)、`CalendarDate`、`CalendarMonth`、`DateRange`、`Weekday`、`TimeOfDay`(新、`{ hour: u8, minute: u8, second: u8 }`)、`DateTime`(新、`{ date: CalendarDate, time: TimeOfDay }`)、`HourCycle`(新、`H12`/`H23`)、`NumberStyle`(新、`{ fraction_digits: FractionDigits, grouping: Grouping }`)、`FractionDigits`(新)、`Grouping`(新)、`DateStyle`(新、`Short`/`Medium`/`Long`/`Iso`)、`TimeStyle`(新、`{ seconds: bool }`)、`NameWidth`(新、`Narrow`/`Short`/`Long`)

`PluralCategory`(新)、`PluralForms`(新)。`Messages::with_plural(key, PluralForms)`(新)。`Locale` の書式のメソッド(新、ICU4X で全 locale。設計書 17.14 節): `plural_category(f64) -> PluralCategory`、`compare(&str, &str) -> Ordering`、 `format_number(f64, NumberStyle) -> String`、`parse_number(&str) -> Option<f64>`、`parse_date(&str) -> Option<CalendarDate>`、`format_date(CalendarDate, DateStyle) -> String`、`format_time(TimeOfDay, TimeStyle) -> String`、`month_name(u8, NameWidth)`、`weekday_name(Weekday, NameWidth)`、`hour_cycle()`、`first_day_of_week()`（既存）。

### `kivact::platform`

| 種類 | item（読み出しは全部 `Value<T>` を返し、どこでも呼べる） |
|---|---|
| 環境 | `window_size`、`scale_factor`、`safe_area`、`keyboard_inset`(新)、`is_fullscreen`、`is_motion_reduced`、`is_surface_active`、`font_families`、`capabilities`、`pointer_settings`(新、`primary: PointerClass` を含む)、`theme_hint`(新)、`is_initial_render` |
| 状態の復元(新) | `save_state(key, bytes)`、`restored_state(key)`、`LifecycleEvent::SaveState` |
| サービス（future を返す） | `copy_text`、`read_text`、`pick_file`、`pick_files`、`pick_folder`、`save_file`、`open_path`、`reveal_path`、`request_window_size`、`set_fullscreen`、`set_reduced_motion` |
| lifecycle | `LifecycleEvent`、`on_lifecycle` |
| 型 | `Capabilities`（`state_restoration` を足す）、`FileDialog`、`PointerSettings`(新)、`PointerClass`(新)、`ThemeHint` |

（`Insets` は `kivact::geometry` へ移る。）

### `kivact::diagnostics`

`why`、`Why`、`WhyDep`、`WhyProp`、`errors`、`Stats`、`Turn`、`TurnRecord`、`TraceRun`、`TraceWrite`（`ErrorKind` は root へ）

### `kivact::interaction`

| 種類 | item |
|---|---|
| hook と handle | `use_button`→`ButtonApi`、`use_checkbox`→`CheckboxApi`、`use_switch`→`SwitchApi`、`use_toggle_button`→`ToggleButtonApi`、`use_disclosure`→`DisclosureApi`、`use_disclosure_group`(新)→`DisclosureGroupApi`、`use_popup`→`PopupApi`、`use_dialog(open, mode)`→`DialogApi`、`use_tooltip`→`TooltipApi`（`.disabled(..)` を持つ）、`use_delayed`→`DelayedApi`、`use_hover_surface`→`HoverSurfaceApi`、`use_hover_corridor`→`HoverCorridorApi`、`use_menu`→`MenuApi`、`use_menubar`→`MenubarApi`、`use_navigation_menu`(新)→`NavigationMenuApi`、`use_select`→`SelectApi`、`use_combobox`→`ComboboxApi`、`use_command_palette`→`CommandPaletteApi`、`use_list`→`ListApi`、`use_tabs`→`TabsApi`、`use_tree`→`TreeApi`、`use_data_table`→`DataTableApi`、`use_data_table_model`→`DataTableModelApi`、`use_scalar`→`ScalarApi`、`use_range_scalar`→`RangeScalarApi`、`use_splitter`→`SplitterApi`、`use_drag`→`DragApi`、`use_zoom_pan`→`ZoomPanApi`、`use_space_pan`→`SpacePanApi`、`use_text_edit`・`use_text_area`→`TextEditApi`、`use_inline_edit`（`ui` から移動）→`InlineEditApi`、`use_follow_end`→`FollowEndApi`、`use_calendar_grid`→`CalendarGridApi`、`use_carousel`(新)→`CarouselApi`、`use_dropzone`(新)→`DropzoneApi`、`use_copy_feedback`(新)→`CopyFeedbackApi` |
| reactive の部品を作る関数と handle（`use_` でないので `*Api` を付けない。API-7・API-15） | `toaster()`→`Toaster`、`announcer()`→`Announcer` |
| `*Api` の読み出しの語 | 開いているかは `is_open()`（`shown()` を消す）、`CopyFeedbackApi::is_copied()`、`ZoomPanApi::is_space_armed()`、`DragApi::track_velocity()`（構造の option） |
| data | `ListItem`、`MenuEntry`、`Command`、`TreeNode`、`Tone`、`Toast`、`ToastOptions`、`ScalarRange`、`ScalarRangeError`、`Selection`(新、旧 `ListSelection`)、`IntoSelection`(新、sealed。impl は `Signal<Option<K>>`・`Signal<Vec<K>>`・`Binding<Option<K>>`・`Binding<Vec<K>>`)、`CalendarSelection`、`CalendarOptions`（旧 `CalendarPolicy`）、`FileInfo`(新)、`CarouselMode`(新)、`Snap`(新)、`ListOptions`（旧 `ListPolicy`）、`ListRangeOptions`（旧 `ListRangePolicy`）、`ListActivation`、`DismissOn`（旧 `Dismiss`）、`DialogMode`（旧 `DialogKind`。`Drawer(Edge)`）、`ComboboxMode`(新)、`FilterValue`、`DragUpdate`、`Reorder`(新、`Reorder<K> { key, to }`)、`EditSession`、`*State`（part の状態の型。ui の slot の closure が受け取るものを含む） |
| policy | `InteractionPolicy`(新、設計書 18.15 節。field は `menu: MenuPolicy`、`tooltip: TooltipTiming`、`hover_card: HoverCardTiming`、`button: ButtonPolicy`、`slider: SliderPolicy`、`switch: SwitchPolicy`、`toast: ToastPolicy`、`select: SelectPolicy`、`drawer: DrawerPolicy`)。入れ子の struct(新): `MenuPolicy { press, drift, submenu, disabled_focus }`、`ButtonPolicy { disabled_focus }`、`SliderPolicy { touch }`、`SwitchPolicy { drag }`、`ToastPolicy { swipe, swipe_slop: f64 }`、`SelectPolicy { sheet_slop: f64 }`、`DrawerPolicy { swipe }`。field の型: `MenuPress`、`PressDrift`(新)、`SubmenuTiming`、`DisabledMenuFocus`、`TooltipTiming`（旧 `TooltipConfig`）、`HoverCardTiming`(新)、`DisabledButtonFocus`（`ui` から）、`SliderTouchMotion`（`ui` から）、`SwitchDrag`（`ui` から）、`ToastSwipe`（`ui` から）、`DrawerSwipe`(新) |
| 関数 | `filter_commands`、`flatten_tree_rows`、`text_edit_policy`、`nearest_snap`、`project_release`、`pull_to_snap`、`resist_overshoot` |

data の record と `*Api` の、同じ名詞の setter と読み出し（例: `ListItem::icon(..)` と `icon_value()`）は、読み出しを名詞（`icon()`）にし、setter を `with_<名詞>`（`with_icon(..)`）にする（API-14。B.10 の (決定) の行）。

### `kivact::ui`

| 種類 | item |
|---|---|
| 標準部品の token(新) | `UiTokens`（部品ごとの入れ子の struct。`UiTokens { button: ButtonTokens, menu: MenuTokens, .. }`。各 struct はその部品の recipe の隣の file に置き、field は `<部分>_<性質>`。設計書 18.15 節）、`*Tokens`、`StateTokens<T> { rest, hovered, pressed, disabled, selected, focused }` |
| 見た目の動きの token の型 | `ButtonBusyMotion`（`UiTokens.button.busy_motion`）、`ProgressMotion`（`progress.motion`）、`ToastRowMotion`（`toast.row_motion`）、`ToggleLabelReservation`（`toggle_button.label_reservation`）、`DialogCenter`（`dialog.center`）、`DisclosureMotion`(新、`disclosure.motion`) |
| 非同期と Tier 3 の部品(新) | `async_view(source: impl AsyncSource<T>, ready)`（`.loading(..)`、`.fallback(Fn(UiError, Retry) -> V)`）、`chat_thread`、`chat_message`、`typing_indicator`、`chat_composer`、`code_block`、`stat`、`heading(text, level)`、`description_list`、`description`、`Trend`（設計書 17.15・17.19・18.5 節） |
| 既存の部品 | `button`、`checkbox`、`switch`、`toggle_button`（旧 `toggle`）、`select`、`menu`、`dialog`、`tooltip`、`tabs`、`radio_group`、`slider`、`range_slider`、`splitter`、`split`、`progress`、`disclosure`、`toast_region`、`command_palette`、`tag_input`、`number_input`、`list`、`data_table`、`DataColumn`、`text_input`、`text_area`、`field`、`tree`、`tree_rows`（`.reveal(..)` を持つ）と、各 builder の型（`Button`、`Checkbox`、`Switch`、`ToggleButton`（旧 `Toggle`）、`Select`、`Menu`、`Dialog`、`Tooltip`、`Tabs`、`RadioGroup`、`Slider`、`Splitter`、`Split`、`Progress`、`Disclosure`、`ToastRegion`、`CommandPalette`、`TagInput`、`NumberInput`、`List`、`DataTable`、`TextInput`、`Field`、`Tree`（旧 `TreeView`）） |
| kit から上がる部品(新) | 付録 C.2 の Tier 1・2（`calendar`、`date_input`、`date_range_input`、`date_time_input`、`time_input`、`carousel`、`dropzone`、`disclosure_group`、`hover_card`、`combobox`、`toolbar`、`toggle_group`、`button_group`、`pagination(count, page)`、`stepper`、`breadcrumb`、`navigation_menu`、`table`（静的）と `table_header`・`table_body`・`table_footer`・`table_row`・`table_head`・`table_cell`、`item_list`、`item_row`、`badge`、`avatar`、`avatar_group`、`skeleton`、`spinner`、`empty_state`、`alert`、`card`、`kbd`、`kbd_combo`、`input_group`） |
| 文脈 | `ControlSize`、`provide_control_size`、`use_control_size`(新、API-7 の対) |
| 上がる部品の値(新) | `StepperItem`（builder の record: `StepperItem::new(label).description(..).status(..)`）、`StepStatus`、`PageItem`、`SkeletonShape` |
| recipe と look | `UiRecipes`、`Recipe`（struct）、`*Recipes`（部品ごとの表）、`Look`、`Variant`、`*Variant`（`ButtonVariant`、`RadioGroupVariant::Segmented`(新)、`ProgressVariant::{Bar, Ring, Gauge}`(新)、`AlertVariant::Banner`(新) など）、`UiRecipes::filled`(新、埋めた表の名前の一覧) |
| 状態の型 | `ButtonState`（`busy`）、`CheckableState`（旧 `CheckState`）、`FieldState`、… （recipe が受け取るもの） |

`ui` は `interaction` の型を再公開しない（`Toaster` などは `kivact::interaction` か prelude から使う）。`UiPolicy` は作らない（振る舞いの knob は `InteractionPolicy`、見た目の動きは `UiTokens`。設計書 18.15 節）。

`tree_rows(..)` の公開 builder は `TreeRows<K>`（複数 root の `IntoView`）。`reveal(gap, transition)` はこの builder に置く。

### `kivact::custom`（旧 `primitive`）

設計書 10.10 節の一覧（17.12 節の `Decoration`、`LineIndex`、`LineRange`、`Node::draw_with_layout` を含む）と、`PartMount`（`Node::on_attach` の引数）、`Owned`（`Owner::store` の戻り値。`get(&self) -> T`（`T: Clone`）、`with(f)`、`try_get`、`try_with`）、`raise(kind, message)`（`#[track_caller]`）。`Part::new(state: Value<S>, apply: impl FnOnce(Node) -> Node + 'static)`（modifier を自分では持たない。設計書 7.4 節）。`Owner` は core の newtype（現在の `kivact_core::primitive::Owner`）で、公開メソッド: `current()`、`root()`、`is_alive()`(新)、`provide`、`context`、`store`、`report`（どれも `#[track_caller]`）。`api: &'static str` と `at: &'static Location` の引数は `#[track_caller]` で置き換えて消す。

Part の参照が必要な呼出し側は `node_ref()` を作り、実 node に `Node::node_ref(target)` を付ける（Part closure または builder の `with_node`）。`Part::node_ref` は残さない。behavior 内部の参照も同じ実 node に束縛できる。

### `kivact::test`（feature `test`、`kivact-test` crate）

`render`、`render_with`、`render_with_handle`（残す。`-> (Screen, H)`）、`RenderOptions`、`Screen`、`Element`、`SemanticNode`、`Within`、`Contact`、`ImeText`、`Screenshot`、`LayoutCheck`、`LayoutRule`、`LayoutViolation`、`TestFonts`（`Only(Vec<Asset>)`）、`MirrorMode`（旧 `WindowMode`。`Off`/`Auto`/`Required`）、`Script`(新)、`gpu_unavailable`、`native_unavailable`(新)、`failure_dir`、`TestParent`（feature `embed`）

`RenderOptions` は `AppOptions` を一つ持つ builder（field は private。作業手順書 C-1）: `AppOptions` と同名のメソッド `window_size`、`min_window_size`、`theme`、`locale`、`messages`、`font`、`blocking_threads`、`gpu_preference` と、試験だけのメソッド `host_locale(tag)`、`reduced_motion()`、`dark()`、`scale(f64)`、`fonts(TestFonts)`、`mirror(MirrorMode)`、`gpu()`、`frame_period_ns(u64)`、`title(..)`、`from_env()`、`with_theme(theme)`（`default().theme(theme)`）。

`Screen` に足す(新): `evidence(name)`、`assert_layout_clean()`、`script() -> Script`（engine の試験 host の台本を包む kivact-test の型で、`respond(kind, f)` などを持つ）。`Element::read_only` を消し、真偽の問い合わせを `is_*`・`has_*` に揃える（`invalid` → `is_invalid`、`password` → `is_password`、`Screen::exited` → `has_exited`、`Screen::soft_keyboard_shown` → `is_soft_keyboard_shown`）。engine の試験の feature の名前は `testing` のまま（`kivact-engine/Cargo.toml`。facade の feature は `test`）。

### `kivact::embed`、`kivact::gpu_interop`

現在の item のまま。`embed::Parent`、`Embedded`。`gpu_interop` の 21 item。

---

## B.2 prelude の固定の一覧

`kivact::prelude` はこの一覧だけを出す（`kivact/src/prelude.rs` に module の順に一つずつ書き、glob の再公開をしない）。規則（設計書 18.6 節）: (1) 入門のガイドと B.7 の例が import なしで使う名前、(2) prelude の関数の位置引数の型。builder の option の型（`ControlSize`、`Edge`、`*Variant`、`style()` の enum など）は入れず、その module から import する。

「作る unit」が「既存」の名前は今の code にある（path の移動は B.10）。N-D1 は、既存の名前と Phase N の unit（N-D1、N-D2、N-F1〜N-F6）が作る名前を全部 `prelude.rs` に書く（build は fan-out の後に通る）。G-3・G-5 が作る名前は、その unit が一行足す。

| module | 名前 | 作る unit |
|---|---|---|
| `kivact` | `App`、`UiError` | 既存 |
| `kivact::state` | `Binding`、`Callback`、`IntoText`、`IntoValue`、`Signal`、`Value`、`Var`、`binding`、`callback`、`constant_value!`、`effect`、`effect_on`、`memo`、`on_cleanup`、`on_mount`、`provide`、`signal`、`untrack`、`var` | 既存 |
| `kivact::state` | `IntoBinding`、`ItemKey`、`PropValue`、`bind!` | N-D2 |
| `kivact::state` | `use_context`（旧 `view::context`） | N-F1 |
| `kivact::geometry` | `Point`、`Size` | 既存（`kivact::geometry` は N-D1 が作る） |
| `kivact::view` | `CheckedState`、`EditPhase`、`Handled`、`IntoView`、`Key`、`KeyEvent`、`Modifiers`、`Node`、`NodeBuilder`、`NodeRef`、`Orientation`、`Placement`、`PointerEvent`、`Role`、`RowHeight`、`ScrollAxes`、`View`、`canvas`、`column`、`dynamic`、`error_boundary`、`for_each`、`for_index`、`grid`、`icon`、`image`、`node`、`node_ref`、`row`、`scroll`、`show`、`show_some`、`spacer`、`stack`、`text`、`virtual_list` | 既存 |
| `kivact::view` | `Component` | N-D2 |
| `kivact::view` | `InputPurpose` | G-3 |
| `kivact::style` | `Align`、`Color`、`ColorToken`、`Justify`、`Style`、`Theme`、`ThemeColor`、`ThemeMode`、`pct`、`provide_theme`、`rgb`、`style` | 既存 |
| `kivact::style` | `use_theme`（旧 `current_theme`）、`use_tokens` | N-F1 |
| `kivact::task` | `Action`、`AsyncState`、`Resource`、`action`、`blocking`、`resource`、`spawn` | 既存 |
| `kivact::task` | `ActionMode`（旧 `ActionPolicy`） | N-F1 |
| `kivact::i18n` | `message` | 既存 |
| `kivact::interaction` | `ListItem`、`MenuEntry`、`Toast`、`Tone`、`toaster` | 既存 |
| `kivact::interaction` | `IntoSelection`、`Selection` | N-D2 |
| `kivact::interaction` | `DialogMode`（旧 `DialogKind`）、`DismissOn`（旧 `Dismiss`） | N-F2 |
| `kivact::ui` | `DataColumn`、`button`、`checkbox`、`data_table`、`dialog`、`field`、`list`、`menu`、`radio_group`、`select`、`slider`、`switch`、`tabs`、`text_area`、`text_input`、`toast_region`、`tooltip` | 既存 |
| `kivact::ui` | `async_view` | G-5 |

`Result` は入れない（利用者の code の `std::result::Result` を隠すため。`main` は `kivact::Result<()>` と書く）。旧い一覧の `ButtonVariant` は 18.6 節の規則（`*Variant` は module から import する）で外した。`IntoSelection` の正準パスは `Selection` と同じ `kivact::interaction`（`kivact-core` は `Selection` を知らないため）。

---

## B.3 移動・改名・削除の注意

移動・改名・削除の行は全部 B.10 にある（旧 B.3 と B.8 の行は、連鎖を畳んで B.10 に入れた）。ここには B.10 の行だけでは分からない理由を残す。

- **`IntoValue::into_prop`・`IntoText::into_text`・`Prop`:** `#[doc(hidden)]` にし、`Prop` は `kivact::state::__private::Prop` だけから届く。`constant_value!` が利用者の crate で展開され、`kivact-style` も `IntoValue` を実装するので、sealed の trait にはできない。
- **`render_with_handle` は残す。** 戻り値の形（`(Screen, H)`）が `render_with` と違い、新しい trait なしには一つの関数にできない。旧 B.3 の「`render_with` に統合」の行は消した。
- **`Style::merge` は公開のまま、`display_grid` は `display(Display)` に置き換える。** kit が recipe を合わせるのに `merge` を使い、soft が style の closure の中で条件付きで grid にしている。内部へ移すのは `with_scroll_axes` だけ（利用者は core の `scroll` だけ）。
- **`NodeBuilder` は sealed にしない。** kit が約 190 の builder で実装する。必須の `node_mut` は `#[doc(hidden)]` で、文書に出る口は `with_node` だけ。
- **`Toaster`・`Announcer` は改名しない。** `toaster()`・`announcer()` は `use_` の付かない reactive の部品を作る関数なので、`*Api`（`use_*` が返す handle）にしない（API-7・API-15）。旧 B.3 の `ToasterApi`・`AnnouncerApi` は取り消した。
- **`InteractionPolicy` の field の型は `interaction` に置く。** ui の部品の振る舞いの knob（`DisabledButtonFocus`、`SliderTouchMotion`、`SwitchDrag`、`ToastSwipe`）も `InteractionPolicy` に入るので、型は `kivact-interaction` へ移る（`kivact-interaction` は `kivact-ui` に依存できない）。
- **lower の型は公開の path に出ない。** lower は engine の内部の module（`kivact_engine::paint::lower`）で、facade から届く署名に出さない。

---

## B.4 handler の表

設計書 10.4 節と 18.2 節の語彙を全部品に当てる。payload のある handler は `impl Fn(E) -> R + 'static`（`R: HandlerResult`）、payload のない handler は `impl Into<Callback<()>>`（closure と `Callback<()>` の両方を受ける）。

`Handled` は `Yes`・`No`・`PreventDefault`。既存の `and` による伝播と既定動作の合成を保ち、`HandlerResult` の四つの impl はこの一つの型に揃える。

| 型 | 再設計後の handler | 現在からの変更 |
|---|---|---|
| `Node`・`NodeBuilder` | `on_press(impl Into<Callback<()>>)`、`on_key(Fn(KeyEvent))`、`on_key_up(Fn(KeyEvent))`、`on_pointer(Fn(PointerEvent))`、`on_wheel(Fn(WheelEvent))`、`on_file(Fn(FileEvent))`、`on_exit_complete(impl Into<Callback<()>>)` | payload のない handler は `impl Into<Callback<()>>`（`From<F: Fn() -> R> for Callback<()>`。設計書 18.2 節）。`on_exit_complete(Callback<()>)` から |
| `Node`（作者向けの固有） | `on_key_default`、`on_pointer_default`、`on_wheel_default`、`on_ime(Fn(ImeEvent))`、`a11y_action(action, Fn(A11yEvent))`、`on_attach(Fn(PartMount))` | `on_a11y`・`on_a11y_event` を `a11y_action` に。`Part::on_bind` を `Node::on_attach` に（`state::on_mount` と分ける） |
| `image` の `Node` | `on_image_error(Fn(UiError))` | `&str` を `UiError`（`ErrorKind::Asset`）に |
| `Overlay` | `on_dismiss(Fn(DismissReason))`、`on_dismiss_default(Fn(DismissReason))` | `on_dismiss_after` を改名 |
| `ErrorBoundary` | `on_error(Fn(UiError) -> R)`、`fallback(Fn(UiError, Retry) -> V)` | `-> R`、`Rc<dyn Fn()>` を `Retry` に |
| `Animated` | `on_complete(impl Into<Callback<()>>)` | `Fn()` から（payload がない） |
| `Action` | `on_complete(Fn(ActionOutcome<T>))`（builder、`self -> Self`） | `on_done(&self, Fn(&Result<..>))` から |
| `ToastOptions` | `on_dismiss(Fn(DismissReason))`（`DismissReason::Timeout` を足す） | `on_hide(Fn())` から |
| `TextEditApi`、`ui::TextInput` | `on_submit(impl Into<Callback<()>>)`、`on_composition_change(Fn(Composition))`、`on_edit(Fn(EditPhase))`、`on_draft_commit(Fn(String))` | `on_composition_change(Fn(bool))` の payload を `Composition` に。`TextInput::draft(on_commit)` を `on_draft_commit` に |
| `InlineEditApi` | `field(key, label, draft) -> TextEditApi`、その `.on_draft_commit(Fn(String)).control()` を描画 node に付ける | interaction は ui に依存しない。label は control の名前、commit/cancel/blur/cleanup は既存動作を保つ |
| `custom::TextEditing` | `on_text_change(Fn(TextEditEvent))`、`on_selection_change(Fn(TextSession))` | `on_edit(TextEditEvent)`（`EditPhase` の `on_edit` と衝突）と `on_selection` を改名 |
| `SelectApi`、`ui::Select` | `on_active_change(Fn(Option<K>))`、`on_edit(Fn(EditPhase))`(新) | `on_preview` を改名。`on_edit` を足す |
| `MenuApi`、`ui::Menu` | `on_activate(Fn(K))` | 位置引数の `choose` から |
| `CommandPaletteApi`、`ui::CommandPalette` | `on_activate(Fn(K))` | 位置引数の `run` から |
| `TreeApi`、`ui::Tree` | `on_activate(Fn(K))` | 変わらない（`ui::Tree` は `-> R` を足す） |
| `ui::TagInput` | `on_add(Fn(String))`、`on_remove(Fn(usize))`、`on_edit(Fn(EditPhase))`(新) | 位置引数から |
| `ui::DataTable` | `on_reorder(Fn(Reorder<K>))` | tuple の引数と `-> T` から |
| `ui::Dropzone` | `on_files(Fn(Vec<FileInfo>))`、`validate(Fn(&FileInfo) -> Option<String>)`（validator。`field` の `error` と同じく、`Some` が利用者に見せる error の文） | 新しい部品 |
| `ui::ChatComposer` | `on_submit(impl Into<Callback<()>>)` | 新しい部品 |
| `ListApi` | `leave_to(Fn(&K) -> Option<K>)` | `on_leave` を改名（event ではない） |
| `CalendarOptions` | `on_activate(Fn(CalendarDate))` | `CalendarPolicy::on_commit` を改名 |
| `DragApi` | `on_start`、`on_update`、`on_end`（`Fn(DragUpdate<T>)`） | 変わらない |
| 値を書く behavior と部品（`EditSession`、`ScalarApi`、`RangeScalarApi`、`SplitterApi`、`ZoomPanApi`、`CheckboxApi`(新)、`SwitchApi`(新)、`ToggleButtonApi`(新)、`TabsApi`(新)、`SelectApi`(新)、`ui` の対応する全部品） | `on_edit(Fn(EditPhase))` | 設計書 11.2 節 |
| `platform` | `on_lifecycle(Fn(LifecycleEvent))` | 変わらない |
| `state` | `on_mount(FnOnce())`、`on_cleanup(FnOnce())` | 変わらない（lifecycle の登録で、`FnOnce` を xtask の例外に列挙する） |

---

## B.5 prop の形の変更の表

| 現在 | 再設計後 |
|---|---|
| `NodeBuilder::hit_testable(bool)` | `hit_testable(impl IntoValue<bool>)` |
| `NodeBuilder::focusable_when_disabled(bool)` | `focusable_when_disabled(impl IntoValue<bool>)` |
| `Part::modal(bool)`、`Overlay::modal(bool)` | `Overlay::modal()`（構造の option。`Part` は modifier を持たず、`Part::new` の closure の中で `Node` に書く） |
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
| 二方向の値の引数 `impl Into<Binding<T>>` | `impl IntoBinding<T>`（`From<Signal<T>> for Binding<T>` は消す） |
| `ui::TextInput::wrap(bool)`、`TextEditApi::wrap(bool)` | `wrap(impl IntoValue<bool>)` |
| `ui::Menu::offset(f32)`、`ui::Tooltip::offset(f32)` | `impl Into<f64>` |
| `ui::Menu::at_least_trigger_width(bool)` | `at_least_anchor_width()`（`Overlay` と揃える） |
| `ui::Select::open_on_arrow(bool)` | `open_on_arrow(impl IntoValue<bool>)` |
| `ui::Select::retain_trigger_focus(bool)`、`SelectApi::retain_trigger_focus(bool)` | `keep_trigger_focus(impl IntoValue<bool>)` |
| `ui::Disclosure::keep_mounted(bool)`、`ui::Tabs::keep_mounted(bool)` | `keep_mounted()` |
| `ui::DataTable::tree_expand_all(bool)` | `initially_expanded()` |
| `ui::ToastRegion::hold_all_on_hover(bool)` | `impl IntoValue<bool>` |
| `ui::Button::loading(impl IntoValue<bool>)`、`ButtonState.loading` | `busy(impl IntoValue<bool>)`、`ButtonState.busy` |
| `ui::Button::size(ControlSize)` | `size(impl IntoValue<ControlSize>)` |
| `ui::split(panes, sizes: impl Into<Binding<Vec<f64>>>)`、`Split::min_size(f32)` | `split(sizes: impl IntoBinding<Vec<f64>>, panes: impl IntoChildren)`、`min_size(impl Into<f64>)` |
| `ui::splitter(name, size, range: impl IntoScalarRange)` | `splitter(label, size: impl IntoBinding<f64>, range: impl IntoValue<ScalarRange>)` |
| `ui::DataColumn::{width, flex, min_width}(f32)` | `impl Into<f64>`（`DataColumn` は record なので plain な値のまま。`numeric(bool)`・`resizable(bool)` も record の例外） |
| `ui::number_input(.., range: ScalarRange)`、`slider.range(impl IntoScalarRange)` | 全部 `impl IntoValue<ScalarRange>`（`0.0..=1.0` と `0..=100` が書ける） |
| 数値の reactive な prop（`impl IntoValue<f64>`） | `4`・`4.0`・`4.0_f32` が書ける（`IntoValue<f64>` を `i32`・`u32`・`f32` に足す） |
| `ui::list(label, items, selected)`、`ui::List::multiple(..)` | `list(label, items).selection(impl IntoSelection<K>)` |
| `ui::tree(label, nodes, expanded, selection: ListSelection<K>)` | `tree(label, nodes).selection(impl IntoSelection<K>).expanded(impl IntoBinding<Vec<K>>)` |
| `ui::disclosure(title, open, content)` | `disclosure(title, open).content(..)` |
| `ui::tree_rows_with_reveal(..)` | `tree_rows(..).reveal(..)` |
| `ui::command_palette(open, commands, run)` | `command_palette(commands, open).on_activate(run)` |
| `motion::Transition::decorative(bool)`（既定 true） | `essential()`（現在の `decorative(false)`） |
| `kivact::style::theme` の kit の拡張 trait（`ThemeOptions`） | kit の `theme_with(mode, ThemeOptions)`（付録 C.1） |
| 公開の署名の bound `Clone + Eq + Hash + 'static`、`Clone + PartialEq + 'static` | `ItemKey`、`PropValue` |

---

## B.6 署名から消す `Rc`・`Arc`・`Box<dyn>`・`Location`

| 現在 | 再設計後 |
|---|---|
| `ErrorBoundary::fallback(Fn(UiError, Rc<dyn Fn()>))` | `Fn(UiError, Retry) -> V` |
| `NodeExt::accessible_name() -> Option<Rc<dyn Fn() -> String>>` | `Node::accessible_name() -> Option<Value<String>>` |
| `Owner::store(name, value: Rc<T>)` | `Owner::store(name, value: T) -> Owned<T>` |
| `Owned::get() -> Rc<T>`、`Owned::try_get() -> Option<Rc<T>>` | `Owned::get() -> T`・`try_get() -> Option<T>`（`T: Clone`）と `Owned::with(f: impl FnOnce(&T) -> R) -> R`・`try_with(..) -> Option<R>` |
| `Theme::get::<T>() -> Arc<T>` | `Theme::ext::<T>() -> ThemeRef<T>`。`pub struct ThemeRef<T>(Arc<T>)`（`kivact::style`）、`Clone`、`Deref<Target = T>`、`T: Clone` のとき `to_owned(&self) -> T`。kit は `kivact::style::use_tokens::<T>() -> ThemeRef<T>` を使う |
| `pub type Recipe<S> = Arc<dyn Fn(&Theme, &Look<S>) -> Style + Send + Sync>` | `pub struct Recipe<S>`（`From<impl Fn(&Theme, &Look<S>) -> Style + Send + Sync + 'static>`、`Clone`） |
| `UiRecipes::clone(&theme.get::<UiRecipes>())` | `theme.ext::<UiRecipes>().to_owned()`（`ThemeRef<T>` の固有メソッドで、`T` を返す。std の `ToOwned` は実装しない） |
| `TestFonts::Only(Vec<Arc<[u8]>>)` | `TestFonts::Only(Vec<Asset>)` |
| `raise(kind, message, at: &'static Location<'static>) -> !` | `kivact::custom::raise(kind, message) -> !`（`#[track_caller]`） |
| `NodeExt::at(&'static Location)`、`Owner::report(kind, message, at)`、`Owned::get_at(at)`、`NodeRefExt::tracked_node(at)` | `#[track_caller]` で `at` の引数を消す: `Owner::report(kind, message)`、`NodeRefExt::tracked_node()`、`Owned::get_at` は消す（`Owned::get` が `#[track_caller]`）。`NodeExt::at(..)`（報告する位置の付け替え）は `Node::located_at_caller()`（引数なし、`#[track_caller]`）にする |
| `Owner::current(api: &'static str)`、`Owner::root(api)`、`app_state(api, init)` | `#[track_caller]`、引数の `api` を消す（`Owner::current()`、`Owner::root()`、`use_app_state(init)`） |
| `AppSettings` の公開の field（`fonts: Vec<Asset>` など） | `AppOptions` の private な field（font は `(Asset, &'static Location)` で持ち、`Location` は facade から届く署名に出ない） |

---

## B.7 書き方の例（第 4 回のレビューで書き直した）

ガイドと例は、この形で書き直す（作業手順書の F-1）。どれも `use kivact::prelude::*;` だけで書ける（状態機械の例は `kivact::state` の import を足す）。data の constructor（`ListItem::new`、`MenuEntry::item` など）の名前は作業の時点のものに合わせる。

### counter（変わらない）

```rust
use kivact::prelude::*;

fn app() -> impl IntoView {
    let count = signal(0);
    let doubled = memo(move || count.get() * 2);
    column((
        row(("Count: ", move || count.get().to_string())),     // closure の子は reactive な文字
        text(move || format!("Doubled: {}", doubled.get())),
        button("Increment").on_press(move || count.update(|n| *n += 1)),
    ))
    .style(style().padding(24).gap(12))
}
```

### props・子・callback

```rust
use kivact::prelude::*;

fn stat(label: impl IntoText, value: impl IntoValue<i32>) -> impl Component {
    let value = value.into_value();
    row((text(label), text(move || value.get().to_string()))).style(style().gap(8))
}

// payload のない callback の prop は impl Into<Callback<()>>。呼ぶ側は closure をそのまま渡す。
fn toolbar(on_add: impl Into<Callback<()>>) -> impl Component {
    let on_add = on_add.into();
    row((button("Add").on_press(on_add), button("Add again").on_press(on_add)))
}

// 一度だけ mount する子は値（impl IntoView）。root が一つなので impl Component を返す。
fn panel(title: impl IntoText, child: impl IntoView) -> impl Component {
    column((text(title), child)).style(style().gap(8).padding(12))
}

fn app() -> impl IntoView {
    let hits = signal(0);
    panel("Stats", column((stat("Hits", hits), toolbar(move || hits.update(|h| *h += 1)))))
        .style(style().max_width(480))
}
```

### form（struct の field の binding）

```rust
use kivact::prelude::*;

#[derive(Clone, PartialEq, Default)]
struct Profile { name: String, plan: Option<Plan>, agree: bool }
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Plan { Free, Pro }
constant_value!(Plan);

fn profile_form() -> impl Component {
    let draft = signal(Profile::default());
    let error = memo(move || draft.with(|p| p.name.is_empty()).then(|| "Name is required".to_owned()));
    column((
        field("Name", text_input("Name", bind!(draft.name))).error(error),
        select("Plan", vec![ListItem::new(Plan::Free, "Free"), ListItem::new(Plan::Pro, "Pro")], bind!(draft.plan)),
        checkbox("I agree", bind!(draft.agree)),
        button("Save").on_press(move || save(draft.get())),
    ))
}
# fn save(_: Profile) {}
```

### 一覧（行は Binding を受け取る）

```rust
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Todo { id: u32, title: String, done: bool }

fn todos_view(todos: Signal<Vec<Todo>>) -> impl IntoView {
    for_each(todos, |t| t.id, move |id, todo| {
        // todos が Signal<Vec<Todo>> なので todo は Binding<Todo>。
        // 引数を row と名付けると prelude の row 関数が隠れるので避ける（E0618）。
        row((
            checkbox(move || todo.get().title, bind!(todo.done)),
            button("Remove").on_press(move || todos.update(|v| v.retain(|t| t.id != id))),
        ))
    })
}
```

### 非同期（retry できる）

```rust
use kivact::prelude::*;

fn user_view(id: Signal<u32>) -> impl IntoView {
    let user = resource(id, |id| async move { load_user(id).await });
    async_view(user, |u: User| text(u.name))      // 既定の loading と、retry 付きの失敗の表示
}
# #[derive(Clone)] struct User { name: String }
# async fn load_user(_: u32) -> Result<User, UiError> { todo!() }
```

### menu（位置引数の callback がない）

```rust
use kivact::prelude::*;

fn file_menu(on_open: impl Into<Callback<()>>, on_save: impl Into<Callback<()>>) -> impl IntoView {
    let (open, save) = (on_open.into(), on_save.into());
    menu("File", vec![MenuEntry::item("open", "Open"), MenuEntry::item("save", "Save")])
        .on_activate(move |key| match key { "open" => open.call(()), _ => save.call(()) })
}
```

### 状態機械で独自の部品を書く

```rust
use kivact::prelude::*;
use kivact::state::{Machine, Step, machine};
use std::time::Duration;

#[derive(Clone, PartialEq)]
enum CopyState { Idle, Copied }
enum CopyEvent { Pressed, Expired }
enum CopyEffect { Write(String) }

struct CopyButton { text: String }               // Self が遷移の読む入力
impl Machine for CopyButton {
    type State = CopyState;
    type Event = CopyEvent;
    type Effect = CopyEffect;
    fn transition(&self, state: &CopyState, event: CopyEvent) -> Step<CopyState, CopyEvent, CopyEffect> {
        match (state, event) {
            (_, CopyEvent::Pressed) => Step::to(CopyState::Copied)
                .with(CopyEffect::Write(self.text.clone()))
                .after(Duration::from_secs(2), CopyEvent::Expired),
            (CopyState::Copied, CopyEvent::Expired) => Step::to(CopyState::Idle),
            (CopyState::Idle, CopyEvent::Expired) => Step::stay(),
        }
    }
}

fn copy_button(text: impl IntoValue<String>) -> impl Component {
    let text = text.into_value();
    let m = machine(CopyState::Idle, move || CopyButton { text: text.get() }, |_, effect| match effect {
        CopyEffect::Write(s) => { spawn(kivact::platform::copy_text(s)); }
    });
    button(move || if m.get() == CopyState::Copied { "Copied" } else { "Copy" })
        .on_press(move || m.send(CopyEvent::Pressed))
}
```

（実際の copy の feedback は `kivact::interaction::use_copy_feedback` を使う。この例は書き方を示すためのもの。）

### 独自の control（part が host の node の性質を読む）

```rust
use kivact::prelude::*;
use kivact::interaction::use_switch;

pub fn my_switch(label: impl IntoText, on: impl IntoBinding<bool>) -> impl Component {
    let api = use_switch(on);
    node()
        .part(api.control())          // role・checked・key・press。caller の disabled も part が読む
        .child(text(label))
        .style(style().padding(4))
}
```

---

## B.8 （B.1〜B.6 と B.10 に畳んだ）

第 4 回のレビューの改名の表（設計書 18.12 節）は、2026-11-05 に B.1〜B.6 の名前へ当て、旧→新の行は B.10 に移した。設計書と作業手順書の「付録 B.8 の名前」は、B.1〜B.6 と B.10 の名前と読む。

---

## B.9 slot の種類の表（API-6、設計書 18.8 節）

| 部品 | `impl IntoView`（一度だけ mount） | `impl IntoViewFn` / `impl Fn(..) -> V`（作り直す） |
|---|---|---|
| `button` | `icon`、`trailing` | — |
| `dialog` | — | `content`、`footer`、`media`（開くたびに作る） |
| `tooltip`、`hover_card` | — | `content` |
| `disclosure` | — | `content`（開いたときに作る） |
| `disclosure_group` | — | `.panel(key, title, content)` の `content` |
| `menu`、`list`、`select`、`radio_group`、`tree`、`tabs`、`toggle_group`、`pagination`、`breadcrumb` | — | `item`（項目ごと） |
| `tabs` | — | `content(Fn(K) -> V)`（選ばれた tab ごと） |
| `command_palette` | `footer` | `item` |
| `splitter`、`split` | `start`、`end`（`split` は子の列） | — |
| `card`、`alert`、`empty_state`、`item_row`、`stat`、`chat_message` | `header`、`media`、`icon`、`actions`、`footer`、`content` | — |
| `toolbar` | `start`、`center`、`end`（子の列） | — |
| `async_view` | — | `ready`（位置引数）、`loading`、`fallback` |
| `error_boundary` | — | `fallback` |
| `viewport` | — | — |
| kit の部品 | 付録 C.1 の KIT-5 の規則（現在の code が開くたびに作り直しているものは `Fn`、それ以外は `IntoView`）で決め、各 kit の README の表に書く | 同左 |

---

## B.10 旧→新の一覧（機械的な置き換えの表）

旧 B.3・旧 B.8・旧 B.4・B.5・B.6 と作業手順書 N-7 の試験の API の変更を、旧→最終の一行ずつにした（連鎖は畳んである。例: `on_composition_change(Fn(bool))` → `on_composition` → `on_composition_change(Fn(Composition))` は一行）。この表にない item の名前と path は変えない。「unit」はその行の最終形を作る unit。`N-F/N-G（付録 B.10 の fan-out）` の行は、定義の段（N-D1・N-D2）の後に、crate の組ごとの fan-out の unit（N-F1〜N-F6、N-G1・N-G2）が自分の file に当てる（kit・例・文書・試験の呼び出し元も同じ）。N-D1・N-D2・N-F2 などと書いた行も、ほかの crate の呼び出し元はその fan-out が直す。`(決定)` の行は文書から決まらなかったもので、設計書 18.1 節（API の規則）に最も合う形を選んだ。

- **B.10.1** は今の code にある名前。機械的な置き換えはこの表で行う。
- **B.10.2** は今の code になく、旧い設計書・付録・作業手順書に予定の名前として書かれていたもの。作る unit は最終の名前で作る。

### B.10.1 今の code にある名前

#### module と path

| 旧の path/名前 | 新の path/名前 | 種類 | unit |
|---|---|---|---|
| `kivact::{Signal, Value, Binding, Var, Callback, Selector}` | `kivact::state::{Signal, Value, Binding, Var, Callback, Selector}` | 移動 | N-D1 |
| `kivact::{signal, memo, effect, effect_on, var, binding, untrack, on_mount, on_cleanup}` | `kivact::state::{..}`（同じ名前） | 移動 | N-D1 |
| `kivact::Asset` | `kivact::view::Asset` | 移動 | N-D1 |
| `kivact::control::*` | `kivact::view::*`（同じ名前） | 移動 | N-D1 |
| `kivact::primitive::*` | `kivact::custom::*`（同じ名前。下の行を除く） | 移動 | N-D1 |
| `kivact::primitive::{Part, PartState, part, PartMount, Owner}` | `kivact::custom::{Part, PartState, part, PartMount, Owner}` | 移動 | N-D1 |
| `kivact::primitive::app_state(api, init)` | `kivact::state::use_app_state(init)`（`#[track_caller]`） | 移動・改名・署名の変更 | N-D1 |
| `kivact::primitive::TextParagraph` | 削除（`kivact::draw::TextParagraph` だけ） | 削除 | N-D1 |
| `kivact::primitive::NodeExt`（trait） | 削除（全メソッドを `Node` の固有メソッドへ。`use kivact::primitive::NodeExt;` は消す） | 削除 | N-D2 |
| `kivact::view::{callback, selector}` | `kivact::state::{callback, selector}` | 移動 | N-D1 |
| `kivact::view::provide` | `kivact::state::provide` | 移動 | N-D1 |
| `kivact::view::provide_theme` | `kivact::style::provide_theme` | 移動 | N-D1 |
| `kivact::view::{provide_locale, provide_messages}` | `kivact::i18n::{provide_locale, provide_messages}` | 移動 | N-D1 |
| `kivact::view::{IntoReactive, IntoText, IntoValue}` | `kivact::state::{IntoReactive, IntoText, IntoValue}` | 移動 | N-D1 |
| `kivact::view::constant_value!` | `kivact::state::constant_value!` | 移動 | N-D1 |
| `kivact::view::Reactive` | `kivact::custom::Reactive` | 移動 | N-D1 |
| `kivact::view::context::<T>()` | `kivact::state::use_context::<T>()` | 移動・改名 | N-F1（path は N-D1） |
| `kivact::view::try_context::<T>()` | `kivact::state::try_use_context::<T>()` | 移動・改名 | N-F1（path は N-D1） |
| `kivact::view::constant(x)` | 削除（literal か `constant_value!`） | 削除 | N-F1 |
| `kivact::view::KeyHandlerResult` | 削除（`kivact::view::HandlerResult` 一つ） | 削除 | N-F/N-G（付録 B.10 の fan-out） |
| `kivact::view::mouse_position()` | `kivact::view::pointer_position() -> Value<Option<Point>>`（どこでも呼べる） | 改名・署名の変更 | N-F1 |
| `kivact::view::current_modifiers()` | `kivact::view::modifiers() -> Value<Modifiers>`（どこでも呼べる） | 改名・署名の変更 | N-F1 |
| `kivact::view::current_direction()` | `kivact::view::use_direction() -> Value<ReadingDirection>` | 改名・署名の変更 | N-F1 |
| `kivact::view::OverlayViewport` | 削除（`Theme::overlay_margin`） | 削除 | I-0 |
| `kivact::style::current_theme()` | `kivact::style::use_theme() -> Value<Theme>` | 改名・署名の変更 | N-F1 |
| `kivact::draw::TextStyle` | `kivact::draw::TextFormat` | 改名 | N-D1 |
| `kivact::draw::RgbaImage` | `kivact::draw::Pixels` | 改名 | N-D1 |
| `kivact::draw::{Point, Size, Rect}` | `kivact::geometry::{Point, Size, Rect}` | 移動 | N-D1 |
| `kivact::platform::Insets` | `kivact::geometry::Insets` | 移動 | N-D1 |
| `kivact::diagnostics::ErrorKind` | `kivact::ErrorKind` | 移動 | N-D1 |
| `kivact::i18n::locale()` | `kivact::i18n::use_locale() -> Value<Locale>` | 改名 | N-F1 |
| `kivact::interaction::EditPhase` | `kivact::view::EditPhase` | 移動 | N-D1 |
| `kivact::ui::{FilterValue, Command, DialogKind, Dismiss, EditPhase, ListActivation, ListItem, MenuEntry, ScalarRange, ScalarRangeError, Side, Toast, ToastOptions, Toaster, Tone, TreeNode, toaster}`（再公開） | 削除（正準パスは `kivact::interaction`、`EditPhase` は `kivact::view`。改名される型は下の行の名前） | 削除 | N-D1 |
| `kivact::ui::IntoChildren` | `kivact::view::IntoChildren` | 移動 | N-D1 |
| `kivact::ui::{InlineEdit, use_inline_edit}` | `kivact::interaction::{InlineEditApi, use_inline_edit}` | 移動・改名 | N-D1（改名は N-F2） |
| `kivact::interaction::Side` | `kivact::geometry::Edge`（`DialogKind::Drawer(Side)` も `DialogMode::Drawer(Edge)`） | 移動・改名 | N-D1 |
| `kivact::ui::ToastSwipeEdge` | `kivact::geometry::Edge` | 移動・改名 | N-D1 |
| `kivact_core::test`（`kivact::test` の中身） | `kivact_test` crate（`kivact::test` は `kivact_test` の再公開。path は変わらない） | 移動 | C-1 |
| `kivact_reactive::Trigger`、`Owner::trigger(at)`（runtime の内部） | `kivact::state::Notifier`、`kivact::state::notifier()`（`notify()`・`track()`。R-6 が runtime に最終の名前で作り、N-1 が出す） | 移動・改名 | N-D1 |

#### state と custom

| 旧の path/名前 | 新の path/名前 | 種類 | unit |
|---|---|---|---|
| `Signal::id() -> NodeId` | 削除 | 削除 | N-D1 |
| `Signal::field(get_ref, get_mut)`、`Binding::field(get_ref, get_mut)` | `Signal::project(get_ref, get_mut) -> Binding<U>`、`Binding::project(..)`（`bind!(x.a.b)` も使える） | 改名 | N-D2 |
| `Signal::item(key, id)` | 削除（`for_each` の行が `Binding<T>` を受ける） | 削除 | N-D2 |
| `impl From<Signal<T>> for Binding<T>` と引数の `impl Into<Binding<T>>` | 削除。引数は `impl IntoBinding<T>` | 削除・署名の変更 | N-D2 |
| `Callback::call_if_alive`（hidden） | `pub(crate)` | 削除 | N-D1 |
| `F64Signal`（private module の pub trait） | 削除（`#[diagnostic::on_unimplemented]` の診断で置き換える） | 削除 | N-D1 |
| `IntoValue::into_prop`、`IntoText::into_text` | `#[doc(hidden)]` | 署名の変更 | N-D1 |
| `Prop` | `kivact::state::__private::Prop`（`#[doc(hidden)]`） | 移動 | N-D1 |
| 公開の署名の bound `Clone + Eq + Hash + 'static`、`Clone + PartialEq + 'static` | `ItemKey`、`PropValue` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `raise(kind, message, at)`（`kivact_reactive`） | `kivact::custom::raise(kind, message)`（`#[track_caller]`） | 移動・署名の変更 | N-F1 |
| `Owner::current(api)`、`Owner::root(api)` | `Owner::current()`、`Owner::root()`（`#[track_caller]`） | 署名の変更 | N-F1 |
| `Owner::report(kind, message, at)` | `Owner::report(kind, message)`（`#[track_caller]`） | 署名の変更 | N-F1 |
| `Owner::store(name, value: Rc<T>)` | `Owner::store(name, value: T) -> Owned<T>` | 署名の変更 | N-F1 |
| `Owned::get() -> Rc<T>`、`Owned::try_get() -> Option<Rc<T>>` | `Owned::get() -> T`・`try_get() -> Option<T>`（`T: Clone`）、`Owned::with(f)`・`try_with(f)` | 署名の変更 | N-F1 (決定) |
| `Owned::get_at(at)` | 削除（`Owned::get` が `#[track_caller]`） | 削除 | N-F1 |
| `NodeRefExt::tracked_node(at)` | `NodeRefExt::tracked_node()`（`#[track_caller]`） | 署名の変更 | N-F1 |
| `NodeExt::at(&'static Location)` | `Node::located_at_caller()`（引数なし、`#[track_caller]`） | 改名・署名の変更 | N-F1 |
| `NodeExt::accessible_name() -> Option<Rc<dyn Fn() -> String>>` | `Node::accessible_name() -> Option<Value<String>>` | 署名の変更 | N-F1 |
| `NodeExt` のそのほかの全メソッド | `Node` の同名の固有メソッド（by value） | 移動 | N-D2 |
| `NodeBuilder::node_mut` | `#[doc(hidden)]`（公開のまま、sealed にしない）。`with_node(self, f)` を足す | 署名の変更 | N-D2 |
| `Part<S>` の modifier のメソッド（`view/part.rs` の約 40。`role`、`on_secondary_context` など） | 削除（`Part::new(state, apply: impl FnOnce(Node) -> Node + 'static)` の closure の中で `Node` の固有メソッドを呼ぶ） | 削除・署名の変更 | N-D2 |
| `Part::on_bind(Fn(PartMount))` | `Node::on_attach(Fn(PartMount))` | 移動・改名 | N-D2 |
| `Part::on_a11y(action, Fn())`、`Part::on_a11y_event(action, Fn(A11yEvent))` | `Node::a11y_action(action, Fn(A11yEvent) -> R)` | 移動・改名 | N-D2 |
| `ui::Scroll::role`、`ui::Button::selected`・`expanded`（部品の builder の semantics のメソッド） | 削除（部品の中で `with_node` か構築時の `Node` で設定する） | 削除 | N-F3 |
| `TextEditing::on_edit(Fn(TextEditEvent))` | `TextEditing::on_text_change(Fn(TextEditEvent))` | 改名 | N-F/N-G（付録 B.10 の fan-out） |
| `TextEditing::on_selection(Fn(TextSession))` | `TextEditing::on_selection_change(Fn(TextSession))` | 改名 | N-F/N-G（付録 B.10 の fan-out） |
| `TextEditing::password(bool)`、`ImeSession::password(bool)` | `purpose(impl IntoValue<InputPurpose>)` | 改名・署名の変更 | G-3 |
| `TextEditing::multiline(bool)`、`report_text(bool)` | `multiline()`、`report_text()` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `Measurement::baseline(f32)` | `baseline(impl Into<f64>)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |

#### view

| 旧の path/名前 | 新の path/名前 | 種類 | unit |
|---|---|---|---|
| `KeyEvent.region`（field） | `KeyEvent::region()`（メソッド） | 署名の変更 | V-5（済み） |
| payload のない handler の引数 `impl Fn() -> R`（`on_press`、`on_submit`、`Animated::on_complete` など） | `impl Into<Callback<()>>` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `on_exit_complete(Callback<()>)` | `on_exit_complete(impl Into<Callback<()>>)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `Node::on_image_error(Fn(&str))` | `on_image_error(Fn(UiError))`（`ErrorKind::Asset`） | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `Overlay::on_dismiss_after(..)` | `Overlay::on_dismiss_default(Fn(DismissReason))` | 改名 | N-F/N-G（付録 B.10 の fan-out） |
| `Overlay::modal(bool)`、`Part::modal(bool)` | `Overlay::modal()`（`Part` は N-3 の行） | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `Overlay::keep_anchor_clear(bool)`、`Overlay::at_least_anchor_width(bool)` | `keep_anchor_clear()`、`at_least_anchor_width()` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `Overlay::viewport_margin(f32)` | `viewport_margin(impl Into<f64>)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ErrorBoundary::on_error(Fn(UiError))` | `on_error(Fn(UiError) -> R)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ErrorBoundary::fallback(Fn(UiError, Rc<dyn Fn()>))` | `fallback(Fn(UiError, Retry) -> V)`（`Retry::run()`） | 署名の変更 | N-D2 |
| `VirtualList::overscan(f32)` | `overscan(impl Into<f64>)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `VirtualList::keep_focused(bool)` | `release_offscreen_focus()`（現在の `keep_focused(false)`） | 改名・署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `NodeBuilder::hit_testable(bool)`、`focusable_when_disabled(bool)` | `impl IntoValue<bool>` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `NodeBuilder::test_id(impl AsRef<str>)` | `test_id(impl Into<String>)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `span(impl Into<String>)` | `span(impl IntoText)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `Span::italic(bool)` | `italic()` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |

#### style・draw・motion・task・platform

| 旧の path/名前 | 新の path/名前 | 種類 | unit |
|---|---|---|---|
| `Theme::get::<T>() -> Arc<T>` | `Theme::ext::<T>() -> ThemeRef<T>` | 改名・署名の変更 | N-F1 |
| `UiRecipes::clone(&theme.get::<UiRecipes>())`（書き方） | `theme.ext::<UiRecipes>().to_owned()` | 署名の変更 | N-F1 |
| `pub type Recipe<S> = Arc<dyn Fn(..)>` | `pub struct Recipe<S>`（`From<impl Fn(&Theme, &Look<S>) -> Style + Send + Sync + 'static>`） | 署名の変更 | N-F1 |
| `App::theme(Theme)` | `App::theme(impl IntoValue<Theme>)` | 署名の変更 | N-F1 |
| `Theme::install`、`Theme::of`、`Style::compile`、span の style の `compile`、`Draw::record` | facade から削除（`kivact_style::internal::{install_theme, theme_of, compile_style, compile_span_style, record}`。`kivact-core` だけが使う） | 移動・改名 | C-2 |
| `Style::with_scroll_axes(axes)` | facade から削除（`kivact_style::internal::with_scroll_axes(style, axes)`。公開の経路は `view::scroll(..).axes(..)`） | 移動 | C-2 |
| `Style::display_grid()` | `Style::display(Display::Grid)`（`Display` と `Style::display` は E-8 が作る） | 改名・署名の変更 | C-2 |
| `Style::opacity(f32)`、`Color::with_alpha(f32)` | `impl Into<f64>` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `Theme::with_icon(impl Into<Cow<'static, str>>, ..)`、`Icon::named(..)` | `impl Into<String>` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `Theme::{with_focus_ring, with_focus_ring_offset, with_disabled_opacity, with_control_height}(f32)` | `impl Into<f64>` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `Path::arc_to`（SVG の形） | `Path::svg_arc_to`（`arc_to` は canvas の形 `arc_to(p1, p2, radius)` になる） | 改名 | G-1a |
| `Transition::as_wire` | 削除 | 削除 | V-6（済み） |
| `Transition::decorative(bool)` | `Transition::essential()`（現在の `decorative(false)`） | 改名・署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `kivact::task::ActionPolicy`、`action(policy, run)` | `kivact::task::ActionMode`、`action(mode, run)` | 改名 | N-F1 |
| `Action::on_done(&self, Fn(&Result<T, UiError>))` | `Action::on_complete(self, Fn(ActionOutcome<T>)) -> Self` | 改名・署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `platform::fullscreen() -> bool` | `platform::is_fullscreen() -> Value<bool>` | 改名・署名の変更 | N-F1 |
| `platform::reduced_motion() -> bool` | `platform::is_motion_reduced() -> Value<bool>` | 改名・署名の変更 | N-F1 |
| `platform::surface_active() -> bool` | `platform::is_surface_active() -> Value<bool>` | 改名・署名の変更 | N-F1 |
| `platform::{window_size, scale_factor, safe_area, font_families, capabilities, is_initial_render}`（snapshot を返す） | 同じ名前で `Value<T>` を返す | 署名の変更 | N-F1 |

#### interaction

| 旧の path/名前 | 新の path/名前 | 種類 | unit |
|---|---|---|---|
| `Combobox` | `ComboboxApi` | 改名 | N-F2 |
| `Delayed` | `DelayedApi` | 改名 | N-F2 |
| `Disclosure`（interaction） | `DisclosureApi` | 改名 | N-F2 |
| `Drag` | `DragApi` | 改名 | N-F2 |
| `FollowEnd` | `FollowEndApi` | 改名 | N-F2 |
| `List`（interaction） | `ListApi` | 改名 | N-F2 |
| `Menubar` | `MenubarApi` | 改名 | N-F2 |
| `CommandPalette`（interaction） | `CommandPaletteApi` | 改名 | N-F2 |
| `Popup` | `PopupApi` | 改名 | N-F2 |
| `RangeScalar` | `RangeScalarApi` | 改名 | N-F2 |
| `Scalar` | `ScalarApi` | 改名 | N-F2 |
| `SpacePan` | `SpacePanApi` | 改名 | N-F2 |
| `TextEdit` | `TextEditApi` | 改名 | N-F2 |
| `Tree`（interaction） | `TreeApi` | 改名 | N-F2 |
| `ZoomPan` | `ZoomPanApi` | 改名 | N-F2 |
| `CalendarGrid` | `CalendarGridApi` | 改名 | N-F2 |
| `Table`、`use_table` | `DataTableApi`、`use_data_table` | 改名 | N-F2 |
| `TableModel`、`use_table_model` | `DataTableModelApi`、`use_data_table_model` | 改名 | N-F2 |
| `Toggle`（`use_switch`・`use_toggle_button` の戻り値） | `SwitchApi`（`use_switch`）、`ToggleButtonApi`（`use_toggle_button`） | 改名 | N-F2 |
| `use_hover_surface(..) -> Delayed` | `use_hover_surface(..) -> HoverSurfaceApi` | 署名の変更 | N-F2 |
| `use_hover_corridor(..) -> Value<bool>` | `use_hover_corridor(..) -> HoverCorridorApi` | 署名の変更 | N-F2 |
| `use_tooltip_when(enabled)` | `use_tooltip().disabled(..)`（真偽が反転する: `move || !enabled.get()`） | 削除・署名の変更 | N-F2 |
| `{ComboboxApi, DelayedApi, DialogApi, DisclosureApi, MenuApi, PopupApi, SelectApi, TooltipApi}::shown()` | `is_open()` | 改名 | N-F2 |
| `ZoomPan::space_armed()` | `ZoomPanApi::is_space_armed()` | 改名 | N-F2 |
| `DialogKind` | `DialogMode` | 改名 | N-F2 |
| `use_dialog(open, kind: DialogKind)` | `use_dialog(open, mode: DialogMode)` | 署名の変更 | N-F2 |
| `Dismiss { escape, outside }` | `DismissOn { escape, outside }`（`DialogApi::dismiss(DismissOn)`・`ui::Dialog::dismiss(DismissOn)`） | 改名 | N-F2 |
| `CalendarPolicy` | `CalendarOptions` | 改名 | N-F2 |
| `CalendarPolicy::on_commit(Fn(CalendarDate))` | `CalendarOptions::on_activate(Fn(CalendarDate))` | 改名 | N-F/N-G（付録 B.10 の fan-out） |
| `ListPolicy`、`ListRangePolicy` | `ListOptions`、`ListRangeOptions` | 改名 | N-F2 |
| `ListSelection` | `Selection`（`IntoSelection<K>` で受ける） | 改名 | N-D2 |
| `IntoScalarRange` | 削除（`impl IntoValue<ScalarRange>`） | 削除 | N-D2 |
| `ListApi::on_leave(Fn(&K) -> Option<K>)` | `ListApi::leave_to(..)` | 改名 | N-F/N-G（付録 B.10 の fan-out） |
| `SelectApi::on_preview(Fn(Option<K>))` | `SelectApi::on_active_change(Fn(Option<K>))` | 改名 | N-F/N-G（付録 B.10 の fan-out） |
| `SelectApi::retain_trigger_focus(bool)` | `keep_trigger_focus(impl IntoValue<bool>)` | 改名・署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `TextEditApi::on_composition_change(Fn(bool))` | `on_composition_change(Fn(Composition))` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `TextEditApi::password(bool)` | `purpose(impl IntoValue<InputPurpose>)` | 改名・署名の変更 | G-3 |
| `TextEditApi::wrap(bool)` | `wrap(impl IntoValue<bool>)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ToastOptions::on_hide(Fn())` | `ToastOptions::on_dismiss(Fn(DismissReason))`（`DismissReason::Timeout` を足す） | 改名・署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `TooltipConfig`（文脈） | `TooltipTiming`（`InteractionPolicy.tooltip` の型） | 改名・移動 | I-0 |
| interaction の theme 拡張 `MenuPress`、`SubmenuTiming`、`DisabledMenuFocus` | `InteractionPolicy.menu.{press, submenu, disabled_focus}` の型（`ThemeExtension` の impl を外す） | 移動 | I-0 |
| `use_content_reveal`、`ContentReveal` | 削除（engine の presence。`NodeRef::presence()` と `Style::enter_size`・`exit_size`） | 削除 | I-7 |
| `ListItem::icon(..)`・`icon_value()`、`ListItem::description(..)`・`description_value()` | setter `with_icon(..)`・`with_description(..)`、読み出し `icon()`・`description()` | 改名 | N-F2 (決定) |
| `MenuEntry::{icon, description, shortcut, disabled_reason}(..)` と `{icon_value, description_value, shortcut_value, disabled_reason_value}()` | setter `with_icon`・`with_description`・`with_shortcut`・`with_disabled_reason`、読み出し `icon()`・`description()`・`shortcut()`・`disabled_reason()` | 改名 | N-F2 (決定) |
| `Command::{group, hint, keywords, icon}(..)` と `{group_value, hint_value, keywords_value, icon_value}()` | setter `with_group`・`with_hint`・`with_keywords`・`with_icon`、読み出し `group()`・`hint()`・`keywords()`・`icon()` | 改名 | N-F2 (決定) |
| `ListApi::anchor(binding)`・`anchor_value()` | `ListApi::with_anchor(impl IntoBinding<Option<K>>)`・`anchor()` | 改名 | N-F2 (決定) |
| `DataTableApi::sort(binding)`・`sort_value()`、`widths(binding)`・`widths_value()`（旧 `Table`） | `with_sort(..)`・`sort()`、`with_widths(..)`・`widths()` | 改名 | N-F2 (決定) |
| `ColumnDef::label() -> String`・`label_value() -> Value<String>` | `ColumnDef::label() -> Value<String>`（snapshot の版は消す。設計書 10.3 節） | 削除・改名 | N-F2 (決定) |
| `ColumnDef::width(..)`・`width_value()` | `ColumnDef::with_width(..)`・`width()` | 改名 | N-F2 (決定) |

#### ui

| 旧の path/名前 | 新の path/名前 | 種類 | unit |
|---|---|---|---|
| `ui::toggle`、`ui::Toggle` | `ui::toggle_button`、`ui::ToggleButton` | 改名 | N-F3 |
| `ui::TreeView` | `ui::Tree` | 改名 | N-F3 |
| `ui::CheckState` | `ui::CheckableState` | 改名 | N-F3 |
| `ui::Button::loading(..)`、`ButtonState.loading` | `busy(impl IntoValue<bool>)`、`ButtonState.busy` | 改名 | N-F3 |
| `ui::Button::size(ControlSize)` | `size(impl IntoValue<ControlSize>)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::Button::trailing_icon(..)` | `trailing(impl IntoView)` | 改名 | N-F3 |
| `ui::Button::into_node`、`ui::TextInput::into_node` | 削除（`IntoView` で足りる） | 削除 | N-D1 |
| `ui::Menu::behavior()` | 削除（private の alias が公開されていた） | 削除 | N-D1 |
| `ui::menu(label, entries, choose)` | `menu(label, entries).on_activate(choose)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::Menu::at_least_trigger_width(bool)` | `at_least_anchor_width()` | 改名・署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::Menu::offset(f32)`、`ui::Tooltip::offset(f32)` | `offset(impl Into<f64>)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::command_palette(open, commands, run)` | `command_palette(commands, open).on_activate(run)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::tag_input(label, tags, on_add, on_remove)` | `tag_input(label, tags).on_add(..).on_remove(..)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::Select::on_preview(..)` | `ui::Select::on_active_change(Fn(Option<K>))` | 改名 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::Select::option(..)` | `ui::Select::item(..)` | 改名 | N-F3 |
| `ui::Select::open_on_arrow(bool)` | `open_on_arrow(impl IntoValue<bool>)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::Select::retain_trigger_focus(bool)` | `keep_trigger_focus(impl IntoValue<bool>)` | 改名・署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::TextInput::on_composition_change(Fn(bool))` | `on_composition_change(Fn(Composition))` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::TextInput::draft(on_commit)` | `ui::TextInput::on_draft_commit(Fn(String))` | 改名 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::TextInput::password(..)` | `purpose(impl IntoValue<InputPurpose>)` | 改名・署名の変更 | G-3 |
| `ui::TextInput::wrap(bool)` | `wrap(impl IntoValue<bool>)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::InlineEdit::field(key, label, draft, on_commit)` | `InlineEditApi::field(key, label, draft).on_draft_commit(..).control()` を描画 node に付ける | 署名の変更、field は TextEditApi | N-F/N-G（付録 B.10 の fan-out） |
| `ui::DataTable::on_reorder(Fn((K, usize)) -> T)` | `on_reorder(Fn(Reorder<K>) -> R)`（`kivact::interaction::Reorder`） | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::DataTable::tree_expand_all(bool)` | `initially_expanded()` | 改名・署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::DataColumn::{text_of, cell_of, has_cell, is_sortable, is_filtered, is_numeric, width_value, definition}` | `pub(crate)` | 削除 | N-D1 |
| `ui::DataColumn::{width, flex, min_width}(f32)` | `impl Into<f64>` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::list(label, items, selected)` | `list(label, items).selection(impl IntoSelection<K>)` | 署名の変更 | N-F3 |
| `ui::List::multiple(..)` | 削除（`.selection(..)` に `Signal<Vec<K>>` を渡す） | 削除 | N-F3 |
| `ui::tree(label, nodes, expanded, selection)` | `tree(label, nodes).selection(impl IntoSelection<K>).expanded(impl IntoBinding<Vec<K>>)` | 署名の変更 | N-F3 |
| `ui::tree_rows_with_reveal(..)` | `tree_rows(..).reveal(..)` | 削除・署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::disclosure(title, open, content)` | `disclosure(title, open).content(..)` | 署名の変更 | N-F3 |
| `ui::Disclosure::keep_mounted(bool)`、`ui::Tabs::keep_mounted(bool)` | `keep_mounted()` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::Dialog::kind(DialogKind)` | `ui::Dialog::mode(DialogMode)` | 改名 | N-F3 |
| `ui::split(panes, sizes)` | `split(sizes: impl IntoBinding<Vec<f64>>, panes: impl IntoChildren)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::Split::min_size(f32)` | `min_size(impl Into<f64>)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::splitter(name, size, range: impl IntoScalarRange)` | `splitter(label, size, range: impl IntoValue<ScalarRange>)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::number_input(label, value, range: ScalarRange)`、`ui::Slider::range(impl IntoScalarRange)` | `range: impl IntoValue<ScalarRange>` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| `ui::ToastRegion::hold_all_on_hover(bool)` | `hold_all_on_hover(impl IntoValue<bool>)` | 署名の変更 | N-F/N-G（付録 B.10 の fan-out） |
| ui の theme 拡張 `DisabledButtonFocus`、`SliderTouchMotion`、`SwitchDrag`、`ToastSwipe` | `kivact::interaction::{DisabledButtonFocus, SliderTouchMotion, SwitchDrag, ToastSwipe}`（`InteractionPolicy.button.disabled_focus`・`slider.touch`・`switch.drag`・`toast.swipe` の型） | 移動 | I-0 |
| ui の theme 拡張 `ButtonBusyMotion`、`DialogCenter`、`ProgressMotion`、`ToastRowMotion`、`ToggleLabelReservation` | `UiTokens.button.busy_motion`・`dialog.center`・`progress.motion`・`toast.row_motion`・`toggle_button.label_reservation` の型（`kivact::ui` のまま。`ThemeExtension` の impl を外す） | 移動 | I-0 |
| `ui::DataTableStickyPaint`、`ui::DataTableRowStatusPaint`（theme 拡張） | `DataTableRecipes` の field の型 | 移動 | I-0 |
| gesture の閾値の literal（menu の 10.0・13.0 px、select の 10 px、toast の 8.0 px） | `InteractionPolicy.menu.drift`、`select.sheet_slop`、`toast.swipe_slop` の既定値 | 移動 | I-0 |
| kit の `ThemeOptions` trait（game・creative） | `theme_with(mode, ThemeOptions)` | 削除・署名の変更 | K-0 |
| kit の `tokens()` | `kivact::style::use_tokens::<Tokens>()` | 削除 | K-0 |
| kit の `Color` enum（5 kit） | `ColorName` | 改名 | K-0 |

#### app と test

| 旧の path/名前 | 新の path/名前 | 種類 | unit |
|---|---|---|---|
| `AppSettings`（`kivact-core/src/view/root.rs`）、`AppConfig`（`client/app_client.rs`） | `kivact_core::launch::AppOptions`（field は private。`AppOptions::check_window_sizes`） | 改名 | C-1 |
| `kivact::test::WindowMode::{Auto, Real, Headless}` | `kivact::test::MirrorMode::{Auto, Required, Off}` | 改名 | A-1b |
| `RenderOptions.theme`（公開の field） | `RenderOptions::theme(..)`（`with_theme(theme)` は `default().theme(theme)`） | 署名の変更 | C-1 |
| `RenderOptions.locale` | `RenderOptions::locale(..)` | 署名の変更 | C-1 |
| `RenderOptions.host_locale` | `RenderOptions::host_locale(tag)` | 署名の変更 | C-1 |
| `RenderOptions.messages` | `RenderOptions::messages(..)` | 署名の変更 | C-1 |
| `RenderOptions.reduced_motion` | `RenderOptions::reduced_motion()` | 署名の変更 | C-1 |
| `RenderOptions.dark` | `RenderOptions::dark()` | 署名の変更 | C-1 |
| `RenderOptions.width`・`height` | `RenderOptions::window_size(width, height)` | 署名の変更 | C-1 |
| `RenderOptions.scale` | `RenderOptions::scale(f64)` | 署名の変更 | C-1 |
| `RenderOptions.fonts` | `RenderOptions::fonts(TestFonts)` | 署名の変更 | C-1 |
| `RenderOptions.window: WindowMode` | `RenderOptions::mirror(MirrorMode)` | 改名・署名の変更 | C-1 |
| `RenderOptions.gpu` | `RenderOptions::gpu()` | 署名の変更 | C-1 |
| `RenderOptions.frame_period_ns` | `RenderOptions::frame_period_ns(u64)` | 署名の変更 | C-1 |
| `RenderOptions.title` | `RenderOptions::title(..)` | 署名の変更 | C-1 |
| `RenderOptions.gpu_preference` | `RenderOptions::gpu_preference(..)` | 署名の変更 | C-1 |
| `RenderOptions { .., ..Default::default() }`（struct の literal） | `RenderOptions::default()` か `from_env()` からの builder の連鎖 | 署名の変更 | C-1 |
| `TestFonts::Only(Vec<Arc<[u8]>>)` | `TestFonts::Only(Vec<Asset>)` | 署名の変更 | N-F1 |
| `Element::read_only()` | 削除（`Element::is_read_only()`） | 削除 | N-F1 |
| `Element::invalid()` | `Element::is_invalid()` | 改名 | N-F1 |
| `Element::password()` | `Element::is_password()` | 改名 | N-F1 |
| `Screen::exited()` | `Screen::has_exited()` | 改名 | N-F1 (決定) |
| `Screen::soft_keyboard_shown()` | `Screen::is_soft_keyboard_shown()` | 改名 | N-F1 (決定) |

### B.10.2 予定の名前（今の code にない）

| 旧の path/名前 | 新の path/名前 | 種類 | unit |
|---|---|---|---|
| `kivact::view::IntoAction<M>`（設計書 17.6 節） | 削除（payload のない handler は `impl Into<Callback<()>>`） | 削除 | N-D2 |
| `kivact::state::{Trigger, trigger}`（設計書 10.1・10.3 節、R-6） | `kivact::state::{Notifier, notifier}` | 改名 | N-D1 |
| `kivact::style::Asset`（作業手順書 N-1 の手順 3） | `kivact::view::Asset` | 移動 | N-D1 |
| `kivact::draw::DrawText`（旧 B.1・B.3、設計書 10.1 節） | `kivact::draw::TextFormat` | 改名 | N-D1 |
| `kivact::view::Reorder`（旧 B.1） | `kivact::interaction::Reorder<K>` | 移動 | N-F2 |
| `kivact::view::ViewportApi`（`to_content`・`to_view`。設計書 17.13 節） | 削除（`ZoomPanApi::to_content`・`to_view`） | 削除 | K-8 |
| `Presence::Leaving`（設計書 17.10 節、E-9） | `Presence::Exiting` | 改名 | E-9b |
| `Style::enter_size(Axis)`・`exit_size(Axis)`（E-9） | `Style::enter_size(Orientation)`・`exit_size(Orientation)` | 署名の変更 | E-9a |
| `FrameLoop::while_shown(node_ref)` | `FrameLoop::while_present(node_ref)` | 改名 | K-0 |
| `kivact::style::Ext<T>` | `kivact::style::ThemeRef<T>` | 改名 | N-F1 |
| `kivact::style::tokens::<T>()`（設計書 17.7 節、付録 C.1） | `kivact::style::use_tokens::<T>()` | 改名 | N-F1 |
| `ThemeMode::System`（設計書 17.18 節） | 削除（`platform::theme_hint()` と `From<ThemeHint> for ThemeMode`） | 削除 | G-4c |
| `Retry::retry()` | `Retry::run()` | 改名 | N-D2 |
| `ToasterApi`、`AnnouncerApi`（旧 B.3） | `Toaster`、`Announcer`（今の名前のまま） | 改名 | N-F2 |
| `kivact::state::IntoSelection`（旧 B.2） | `kivact::interaction::IntoSelection` | 移動 | N-D2 (決定) |
| `impl Into<Selection<K>>`（設計書 10.8 節） | `impl IntoSelection<K>`（`From<..> for Selection<K>` は作らない） | 署名の変更 | N-D2 |
| `list(items).selection(..)`、`tree(nodes).selection(..)`（設計書 10.8 節） | `list(label, items).selection(..)`、`tree(label, nodes).selection(..).expanded(..)` | 署名の変更 | N-F3 |
| `Node::on_mount(Fn(PartMount))`（旧 B.4、設計書 7.4 節） | `Node::on_attach(Fn(PartMount))` | 改名 | N-D2 |
| `on_composition(Fn(Composition))`（旧 B.4、設計書 10.4 節） | `on_composition_change(Fn(Composition))` | 改名 | N-F/N-G（付録 B.10 の fan-out） |
| `on_preview(Fn(Option<K>))`（設計書 10.4 節） | `on_active_change(Fn(Option<K>))` | 改名 | N-F/N-G（付録 B.10 の fan-out） |
| `ToastOptions::on_hide(..) -> R`（旧 B.4、設計書 10.4 節の `on_hide`） | `ToastOptions::on_dismiss(Fn(DismissReason))` | 改名 | N-F/N-G（付録 B.10 の fan-out） |
| `Machine` の `Input`・`Timer` の関連型、`transition(state, event, input)`、`MachineCx::after`・`cancel`、`TimerKey`（設計書 6.5 節） | 削除。`transition(&self, state, event)`、`Step::after(delay, event)`・`cancel(event)`（設計書 18.13 節） | 削除・署名の変更 | R-6 |
| `Step<S, E>`、`machine(initial, input: Fn() -> M::Input, run)` | `Step<S, Ev, Ef>`、`machine(initial, input: Fn() -> M, run)` | 署名の変更 | R-6 |
| `Settings<S>`（設計書 11.1 節、crate private） | `OptionsCell<S>` | 改名 | I-0 |
| behavior の `*Settings`（`DialogSettings` など、crate private） | `*Options` | 改名 | I-0 |
| `ui::UiPolicy` と field（設計書 11.1 節） | 削除（振る舞いは `InteractionPolicy` の入れ子、見た目の動きは `UiTokens`） | 削除 | I-0 |
| `InteractionPolicy` の平らな field `menu_press`、`menu_press_drift`、`submenu_timing`、`disabled_menu_focus`、`tooltip_config`、`hover_card_timing`（設計書 11.1 節） | `menu.press`、`menu.drift`、`menu.submenu`、`menu.disabled_focus`、`tooltip`、`hover_card` | 改名 | I-0 |
| `UiPolicy.toast_swipe_slop`、`select_sheet_slop`、`drawer_swipe`、`disclosure_motion` | `InteractionPolicy.toast.swipe_slop`、`select.sheet_slop`、`drawer.swipe`、`UiTokens.disclosure.motion` | 移動・改名 | I-0 |
| `ui::DrawerSwipe`（旧 B.1） | `kivact::interaction::DrawerSwipe`（`InteractionPolicy.drawer.swipe`） | 移動 | K-2a |
| `UiTokens` の平らな field `<部品>_<部分>_<性質>`（設計書 17.7 節） | 部品ごとの入れ子 `UiTokens.<部品>.<部分>_<性質>` | 改名 | K-0 |
| `StateTokens { rest, hover, pressed, disabled, selected, focus }` | `StateTokens { rest, hovered, pressed, disabled, selected, focused }` | 改名 | K-0 |
| `retain_focus()`（`use_list`、設計書 10.6 節） | `keep_focus()` | 改名 | I-4a |
| `DragApi::velocity()`（設計書 11.1 節、I-3） | `DragApi::track_velocity()` | 改名 | I-3 |
| `CopyFeedbackApi::copied()` | `CopyFeedbackApi::is_copied()` | 改名 | K-2a |
| `SnapMode`（旧 B.1） | `Snap` | 改名 | K-1b |
| `combobox(..).tokens()`（設計書 10.6 節） | `combobox(..).chips()` | 改名 | K-2b |
| `disclosure_group(..).item(key, title, content)` | `disclosure_group(..).panel(key, title, content)` | 改名 | K-2a |
| `navigation_menu(..).item(key, label, content)` | `navigation_menu(..).link(key, label, content)` | 改名 | K-3b |
| `ui::item(title)`（付録 C.2） | `ui::item_row(title)` | 改名 | K-4 |
| `ui::async_view(state: impl IntoValue<AsyncState<T>>, ready)`（設計書 17.19 節） | `ui::async_view(source: impl AsyncSource<T>, ready)` | 署名の変更 | G-5 |
| `async_view(..).error(..)` | `async_view(..).fallback(Fn(UiError, Retry) -> V)` | 改名 | G-5 |
| `impl IntoValue<AsyncState<T>> for Resource<T>`・`Action`（設計書 17.6 節） | 削除（`AsyncSource`。`AsyncState` だけを持つときは `dynamic`） | 削除 | G-5 |
| `ui::pagination(page, count)`（付録 C.2） | `ui::pagination(count, page)` | 署名の変更 | K-3b |
| `ui::heading(level, text)`（設計書 17.15 節） | `ui::heading(text, level)` | 署名の変更 | K-9b |
| `ui::calendar(selected)`・`calendar_range(range)`（kit の版） | `ui::calendar(selection: impl IntoBinding<CalendarSelection>)`（`CalendarSelection` は値の enum。一日か範囲かを値で持つ。`IntoSelection` ではない） | 署名の変更 | K-1a |
| `ui::clickable_card(..)`、`ui::selectable_card(selected)` | `card(..).on_press(..)`、`card(..).selected(impl IntoBinding<bool>)` | 削除・署名の変更 | K-6b |
| `ui::kbd_combo(Vec<Key>)` | `ui::kbd_combo(impl IntoValue<Vec<Key>>)` | 署名の変更 | K-6a |
| `StepperItem { label, description, state }`（公開の field）、`StepState` | `StepperItem::new(label).description(..).status(..)`（builder の record）、`StepStatus` | 署名の変更・改名 | K-3b |
| `ui::TableAlign`（付録 C.2） | `kivact::style::TextAlign` | 削除 | K-4 |
| `ui::Segmented`（付録 C.2） | `ui::RadioGroupVariant::Segmented` | 改名 | K-0 |
| progress の `Ring`・`Gauge`、alert の banner（付録 C.2） | `ui::ProgressVariant::{Bar, Ring, Gauge}`、`ui::AlertVariant::Banner` | 改名 | K-9b（progress）、K-6b（alert） |
| `basic::DialogLook`、`basic::MenuLook`、soft `SegmentedLook`、game `GameCellLook`（付録 C.1・C.4） | `DialogVariant`、`MenuVariant`、`SegmentedVariant`、`GameCellVariant` | 改名 | K-7a〜K-7e |
| `App::fonts(iter)`（旧 B.1） | 作らない（`App::font` を繰り返す。`RenderOptions::fonts(TestFonts)` と衝突するため） | 削除 | C-1 (決定) (決定) |
| `RenderOptions::with_theme`・`reduced_motion`（旧 作業手順書 N-7） | `RenderOptions::with_theme(theme)`、`reduced_motion()`（C-1 の builder に含む） | 署名の変更 | C-1 |
