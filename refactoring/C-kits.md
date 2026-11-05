# 付録 C: 公式 kit の整理

[設計書](../REFACTORING_DESIGN.md) 11.3 節と 12 章の付録。部品の名前と引数の順は付録 B.8 の改名の表と設計書 18 章が優先する（例: `calendar(selection)`、`card(..).on_press`、`toggle_button`、`item_row`、`RadioGroupVariant::Segmented`）。`file:line` は `main` @ `a3536ef`、path は `crates/` からの相対。

**前提（作者の決定）:** 5 つの kit には repo の外に実際の利用者がいる。kit を縮めない（部品も option も減らさない）。ただし利用者は API の変更に追従するので、API を壊すことは避けない。中核へ上がる部品は、上がる前の kit の版の機能の**和集合**を持つ。

- C.1 kit の規約の具体
- C.2 中核へ上がる部品
- C.3 kit の hook の行き先
- C.4 名前の衝突の改名
- C.5 kit ごとの作業
- C.6 試験と catalog

---

## C.1 kit の規約の具体（設計書 12 章の KIT-1〜KIT-10）

| 規約 | basic | pop | game | creative | soft |
|---|---|---|---|---|---|
| KIT-1 theme | `theme(mode)`（変わらない） | 同 | `ThemeOptions` trait（`theme.rs:105-124`、`density`）を消し、`theme_with(mode, ThemeOptions { density })` | `ThemeOptions` trait（`theme.rs:65-84`、`accent`・`density`）を消し、`theme_with(mode, ThemeOptions { accent, density })`。`install()`（`:88-104`）は `theme_with` に統合 | `theme(mode)`。theme が入れている 15 の中核の policy の拡張（`theme.rs:70-167`）は `InteractionPolicy` と `UiPolicy` の二つの値と recipe の表へ移す（設計書 11.1 節）。i18n の key を theme の値に入れている箇所（`:165`）は message の key の参照にする |
| KIT-2 token と色 | `Color`（40 variant）→ `ColorName` | `Color`（19）→ `ColorName` | `Color`（15）→ `ColorName`。palette の色の literal（`theme.rs:33-70`）を `Tokens` へ移す | `Color`（21）→ `ColorName` | `Colors`（58 field の struct）は残し、`ColorName` enum を足す。`pub fn tokens()`（`theme_access.rs:8`、`lib.rs:273`）を消し、`kivact::style::tokens::<Tokens>()` を使う |
| KIT-2 literal の色（`tokens.rs`・`theme.rs`・`color.rs` の外） | 0 | 19 → 0 | 21 → 0 | 28 → 0 | 16 → 0（`lightbox.rs:192,209,248,318`、`shadows.rs` など） |
| KIT-3 recipe の表 | 埋めていない表（checkbox の三つの part、switch の二つの part、tree、tag_input、number_input）は埋めない。README の recipe の表に「中核の既定」と書く（KIT-3） | button_icon、checkbox・switch の part、splitter、split、tree | button_icon、part、split、tree | button_busy、button_icon、part、field、tree、split、command_palette、tag_input、number_input | 全部（現状のまま） |
| KIT-3 内部の flag の variant | `Flush`（`recipes_dialog.rs:20`）と `MenuVariant`（`recipes_menu.rs:45`）を公開の variant の型にする（`basic::DialogLook::Flush`、`basic::MenuLook`。`impl Variant`）。crate private の variant で標準部品を内部用に曲げない | — | — | — | — |
| KIT-5 slot | `slot.rs` の `Slot(Box<dyn FnOnce>)` を消し、`impl IntoView` と `ViewFn` にする（`FnOnce` の slot 28 か所） | `slot.rs` を消す（5 か所） | `slot.rs` を消す（4 か所） | `slot.rs` を消す（3 か所） | `toolbar.rs:29` などの local の `type Slot` を消す。`content` の三種類（`Fn`・`IntoView`・`FnOnce`）は、現在の code がその内容を開くたびに作り直しているもの（`app_shell.rs:83`、`banner.rs:88`、`bottom_sheet.rs:132`、`collapsible.rs:141` の `Fn` のもの）は `impl Fn() -> V`、一度だけ mount するもの（それ以外）は `impl IntoView` にする（API-6。code の事実で決まる） |
| KIT-5 children の trait | `GroupItems`（`children.rs:18`）を消し `IntoChildren` | — | — | — | `GroupMembers`、`InputGroupItems`、`IntoListItems`、`IntoMetadataItems` を消し `IntoChildren` |
| KIT-6 variant | `BadgeVariant` などの部品の enum を `impl Variant` の型にする | 同（`Hue` は既に `Variant`） | 同 | 同 | `Segmented`＋`SegmentedLayout`＋`SegmentedIconOnly` を一つの variant の型 `SegmentedLook { layout, icon_only }` に |
| KIT-6 inherent の `disabled` | — | — | — | — | 18 か所の固有の `disabled`（`link.rs:97`、`nav_heading.rs:50`、`segmented_control.rs:87`、`tab_list.rs:128`、`token.rs:126` ほか）を消し、`NodeBuilder::disabled(impl IntoValue<bool>)` を使う |
| KIT-6 option の束 | — | — | — | `WindowOptions` は維持。`ToolWindowBuilder` の `*Builder` の接尾辞を消す（`ToolWindow`） | `CollapsibleConfig`、`ResizableConfig`、`MobileNavConfig`、`OverflowOptions` の公開 field の config を builder の setter にする（C.3 の行き先に従う） |
| KIT-6 型の alias | — | — | — | — | `MultiSelector`、`Tokenizer`、`TopNavHeading`、`TopNavMenuItem`、`SyntaxTheme`、`List`/`Table`/`TreeDensity = ItemDensity` の alias を消し、一つの名前にする。`item()` と `list_item()` の二重を消す |
| KIT-8 message の key | `basic.` の 3 段。README の「unprefixed」の記述（`lib.rs:41-46`）と、存在しない `basic::direction` の記述（README `:63`、catalog `docs/direction.rs:18`）を直す | `pop.date.error.format`（4 段）→ `pop.date_input.format_error` | — | `creative.dialog.close` → 標準の `dialog.close` | `soft.kbd.key.control`（4 段）→ `soft.kbd.control`、`soft.chat_layout_scroll_button.scroll_to_bottom` → `soft.chat_layout.scroll_to_bottom`、`soft.markdown_heading_links.copy` → `soft.markdown.copy_heading_link`、`soft.date_time.*` の年月の label → 中核の `date_input.*`。"Loading" の 4 つの key（`basic.spinner.loading`、`game.skeleton.label`、`pop.loader.label`、`soft.spinner.loading`）→ 標準の `spinner.loading` |
| RTL | 13 か所（維持） | 0（中核の行の自動反転に任せていることを README に書く） | 0（同） | 0（同） | 自前の `rtl()`（`stack.rs:46`）52 か所と `current_direction()` 29 か所を `use_direction()` にする |
| breakpoint | — | — | — | — | `selector_surface.rs:27-28` の 768・500、`alert_dialog.rs:111,115` の 640 を soft の `media_query::Breakpoint` にする |
| `provide((), ..)` の回避策 | `sidebar/state.rs:3-8,85-89` | — | — | — | 41 か所。part が provider より先に作られて文脈を読めない問題は、設計書 7.4 節の `Part` を mount のときに当てる形で解ける。全部を通常の `provide` にする |

---

## C.2 中核へ上がる部品

**引数の型の表（設計書 0.3 節の M-7）。** この節が名前だけを書いた builder のメソッドの引数は、次の表で決める（設計書 10 章の規則を当てたもの）。

| 引数の意味 | 型 |
|---|---|
| 表示する文字列（`label`、`title`、`description`、`placeholder`） | `impl IntoText` |
| 実行中に変わりうる値（`min`・`max` は `impl IntoValue<Option<T>>`、`week_start` は `impl IntoValue<Weekday>`、`placement` は `impl IntoValue<Placement>`、`tone` は `impl IntoValue<Tone>`、`orientation` は `impl IntoValue<Orientation>`、そのほか `T` の名前のもの） | `impl IntoValue<T>` |
| 一度だけ mount する内容（`icon`、`media`、`header`、`footer`、`actions`、`trailing`） | `impl IntoView` |
| 開くたび・項目ごとに作る内容（`content`、`item` の closure） | `impl Fn(..) -> V + 'static` |
| 子の列（`start`・`center`・`end`、`ui::table(..)`、`item_list(..)`、`button_group(.., children)`） | `impl IntoChildren` |
| handler（`on_dismiss`、`on_activate`、`on_files`） | 設計書 10.4 節の表 |
| 位置の列（`SplitterApi::snaps`） | `impl IntoValue<Vec<f64>>`（0.0〜1.0 の割合） |

### Tier 1: 振る舞いが重複しているもの（この再設計で必ず上げる）

| 部品 | 新しい中核の API（`ui` は見た目、`interaction` は振る舞い） | 元の kit の版 | 和集合として持つ機能 |
|---|---|---|---|
| calendar | `ui::calendar(selected: impl Into<Binding<Option<CalendarDate>>>)`、`ui::calendar_range(range: impl Into<Binding<Option<DateRange>>>)`。builder: `.month(Binding<CalendarMonth>)`、`.today(impl IntoValue<CalendarDate>)`、`.week_start(impl IntoValue<Weekday>)`（既定は locale）、`.min(..)`、`.max(..)`、`.disabled_dates(Fn(CalendarDate) -> bool)`、`.marks(Fn(CalendarDate) -> Option<Tone>)`。振る舞いは既存の `use_calendar_grid` | basic `calendar/view.rs:81,90`、soft `calendar.rs:62,67`、pop `date/calendar.rs:76`（`Binding<CalendarDate>` と `today`） | pop の非 `Option` の binding は `Option` に揃える。pop の calendar の印（`date/calendar.rs:389`）を `marks` に。`week_start` と `week_starts_on` の二つの名前を `week_start` に。月名・曜日名は `kivact-i18n` の `month_name`・`weekday_name`（basic・soft の MONTHS の表と soft の `timestamp.rs` の英語の表を消す） |
| date inputs | `ui::date_input(label, value: impl Into<Binding<Option<CalendarDate>>>)`、`ui::date_range_input(label, range)`、`ui::time_input(label, value: impl Into<Binding<Option<TimeOfDay>>>)`、`ui::date_time_input(label, value: impl Into<Binding<Option<DateTime>>>)`。`.format(DateStyle)`、`.min/.max`、`.week_start(..)`。文字の入力の解釈は `Locale::parse_date`（en-US と ISO の形式。soft の複数の形式 `date_input.rs:54-84` と pop の `parse_date` `date/text.rs:21` を統合） | basic `date_picker`・`date_range_picker`（`date_picker.rs:137,212`）、pop `date_input`・`date_range_input`、soft の 4 つ | `TimeOfDay`・`DateTime` は `kivact::i18n` に足す。範囲の選び方は `CalendarSelection::Range`（pop の再実装 `range_input.rs:187-200` を消す）。soft の touch の wheel（`date_touch.rs`）は `use_scalar` の上の `ui::date_input` の touch の look として recipe で選ぶ。状態は付録 A の SM-K18〜K20 |
| carousel | `ui::carousel(index: impl Into<Binding<usize>>, slides: impl IntoChildren)`。`.looping()`、`.snap(impl IntoValue<SnapMode>)`、`.orientation(impl IntoValue<Orientation>)`、`.mode(CarouselMode)`（構造の option）。`interaction::use_carousel` | basic（`Binding<usize>`、translate）、soft（scroll、`use_carousel`） | 両方の model を `CarouselMode::{Translate, Scroll}`（構造の option、既定は `Translate`）で持つ。Shift+wheel の横 scroll は `ui::carousel` の中の wheel の handler に移す（soft の手書き `carousel.rs:327-344` を中核の部品へ移すだけで、engine の scroll の規則は変えない） |
| dropzone | `ui::dropzone(label)`、`.on_files(Fn(Vec<FileInfo>))`、`.validate(Fn(&FileInfo) -> Option<String>)`（`Some` が見せる error の文。`field` の `error` と同じ形）、`.multiple()`、`.pick_on_press()`。`interaction::use_dropzone` | pop `form.rs:87-140`、game `input.rs:83-200`、soft `file_input.rs:95-130` | game の `PickStatus` と soft の status の文を `Dropzone` の状態（付録 A の SM-K31）に。soft の `std::fs::metadata` による検証は caller の `validate` へ |
| disclosure group（accordion） | `ui::disclosure_group(selection: impl Into<Selection<K>>)`、`.item(key, title, content: impl Fn() -> V)`。`interaction::use_disclosure_group` | soft `collapsible.rs:298-327`（GroupCx）、basic の `collapsible` の並び | soft の `collapsible(label, open)` は `ui::disclosure(label, open)`、basic の `collapsible(open).content(..).attach(trigger)` は `ui::disclosure(..).attach(trigger)`（trigger を caller が持つ形を option で持つ）。「無効なら常に開く」は `always_open()` |
| hover card | `ui::hover_card(content: impl Fn() -> V).attach(trigger)`（戻り値は `ui::menu(..).attach(trigger)` と同じ形の一つの view）。`.placement(..)`、`.open(impl Into<Binding<bool>>)`（制御する場合）。遅延は `InteractionPolicy`（basic 600/300 ms、soft 300/200 ms は各 kit の Policy の値） | basic `hover_card.rs:53,94`、soft `hover_card.rs:35,104` | 中核の `use_hover_surface` の上に作る（両 kit とも使っていない） |
| combobox 系 | `ui::combobox(label, options: impl IntoValue<Vec<ListItem<K>>>, selection: impl Into<Selection<K>>)`。`.query(impl Into<Binding<String>>)`、`.free_text()`、`.mode(ComboboxMode)`、`.tokens()`（複数の選択を token で見せる） | basic `combobox.rs`、soft `typeahead.rs`・`selector.rs`・`MultiSelector`・`Tokenizer`、creative `picker.rs` | soft の `Selector` は `ui::select`（soft の recipe）。`complex_selector`・`power_search` は soft に残す（soft 固有）が、内部は `use_combobox` |
| select の touch の sheet | 既存の `ui::select` に入っている（付録 A の SM-U03） | soft `selector.rs:143-184`、`selector_surface.rs` | soft の sheet の look は recipe |
| drawer の swipe | `ui::dialog(..).kind(DialogKind::Drawer(side))` に swipe で閉じる振る舞いを足す（`use_drag` の velocity、`UiPolicy.drawer_swipe`） | basic `drawer.rs`（独自の velocity）、soft `bottom_sheet.rs`（snap） | basic の `drawer` は消える（標準の dialog の Drawer）。soft の bottom sheet の snap は soft に残す（SM-K28） |
| copy feedback | `interaction::use_copy_feedback(duration) -> CopyFeedbackApi`（`copy(text)`、`copied() -> Value<bool>`） | soft `code_block.rs:339,520-533`、basic `code_block/view.rs:297-321` | 部品は作らない（code block は Tier 3） |
| toolbar | `ui::toolbar(label)`、`.start(impl IntoChildren)`、`.center(..)`、`.end(..)`。roving focus は中核の `roving_focus` | game `button.rs:41`（`toolbar(children) -> Node`）、soft `toolbar.rs:45` | game の `toolbar(children)` は `.start(children)`。soft の key の hint（SM-K25）は soft の recipe でなく soft の部品として残す |
| toggle group と segmented | `ui::toggle_group(label, options: impl IntoValue<Vec<ListItem<K>>>, selection: impl Into<Selection<K>>)`。segmented は `ui::radio_group(..).variant(ui::Segmented)`（標準の variant。各 kit は recipe で look を持つ） | basic `toggle_group`、pop・game・creative の `Segmented`（`choice.rs`）、soft `segmented_control`・`toggle_button_group` | soft の `ToggleSelection`（`toggle_button_group.rs:17-78`）と basic の `Selection`（`choice.rs:79-121`）は中核の `Selection` に。`pressed_icon(Fn(&K) -> Option<&'static str>)` は `ListItem` の icon の pressed の版を option に |
| button group | `ui::button_group(label, children: impl IntoChildren)` | basic `button_group.rs:53`、game `button.rs:33`、soft `button_group.rs:91` | soft の手書きの roving（`:126-197`）を消す |
| pagination | `ui::pagination(page: impl Into<Binding<usize>>, count: impl IntoValue<usize>)`、`.siblings(impl IntoValue<usize>)`（page の変更は binding が持つので handler は作らない） | basic（静的な part）、game `navigation.rs:126`、soft `pagination.rs`・`table_pagination` | basic の静的な part の組み立ては、`ui::pagination` の slot（`.item(Fn(PageItem) -> V)`）で表す。game の作り直しのときの focus の復元（SM-K33）は roving の list が持つ |
| stepper | `ui::stepper(items: impl IntoValue<Vec<StepperItem>>, current: impl IntoValue<usize>)` | pop `patterns.rs:50`、game `navigation.rs:12`、soft `stepper.rs` | `StepperItem { label, description, state: StepState }`（`Step` は `kivact::state::Step` と衝突するので使わない） |
| breadcrumb | `ui::breadcrumb(items: impl IntoValue<Vec<ListItem<K>>>).on_activate(Fn(K))` | basic（part）、pop `patterns.rs:15`、game `navigation.rs:93`（`Vec<String>`）、soft `breadcrumbs` | 末尾の項目は `CurrentItem::Page` |
| navigation menu | `ui::navigation_menu(label)`、`.item(key, label, content: impl Fn() -> V)`。`interaction::use_navigation_menu`（状態は basic の `NavOpen::{Closed, Hover(i), Pinned(i)}` を `Machine` にしたもの） | basic `navigation_menu.rs`、soft `top_nav.rs:516-531`・`side_nav.rs:562-581` | soft の top nav と side nav の flyout は `use_navigation_menu` を使う（SM-K12・K13） |
| tree | 既存の `ui::tree`。kit は自分の版を消して recipe で restyle する | game `layout.rs:109`、creative `outliner/view.rs:230`、soft `soft_tree.rs:128` | creative の outliner は分野固有の機能（列、可視・lock の toggle）を `ui::tree` の slot と `use_tree` の上の creative の部品として残す。soft の `tree_list` は `ui::tree` と soft の recipe（soft は既に `.tree()` の recipe を埋めている） |
| static table | `ui::table(..)`、`table_header`、`table_body`、`table_footer`、`table_row`、`table_head`、`table_cell`、`TableAlign` | basic `table.rs:46-182`、soft `soft_table.rs:30-182`（同じ関数名） | game の `TableCell` enum（`table.rs:12`、data_table の cell の look）は `GameCellLook` と改名して衝突を避ける |
| item list（SOFT_UI_WORK_ORDER の C-33） | `ui::item_list(children: impl IntoChildren)`、`ui::item(title)`（`.media(..)`、`.description(..)`、`.actions(..)`、`.on_press(..)`） | basic `item`・`item_group`・`item_separator`、soft `list`・`item`・`list_item`、pop `list_item`（`ListItem` struct） | `Role::List`・`ListItem` の semantics、marker、divider、density |

### Tier 2: 見た目の部品で、三つ以上の kit が持つもの（この再設計で上げる）

| 部品 | 新しい中核の API | 元の kit の版 | 注 |
|---|---|---|---|
| badge | `ui::badge(label: impl IntoText)`、`.tone(impl IntoValue<Tone>)`、`.variant(impl Variant)` | basic `badge.rs:16,56`、pop `display.rs:31,46`、game `display.rs:68,74,84`（`badge(label, tone) -> Node`、`tag_badge`）、soft `feedback.rs:12,40` | kit の `BadgeVariant` は各 kit の `impl Variant` の型（pop の `Dot`・`Count`・`Hue(Hue)` を含む） |
| avatar | `ui::avatar(name: impl IntoText)`、`.src(impl IntoValue<Option<Asset>>)`、`.size(impl IntoValue<ControlSize>)`、`.status(impl IntoValue<Option<Tone>>)`。`ui::avatar_group(children: impl IntoChildren)`、`.max(impl IntoValue<usize>)` | basic `avatar.rs:66`・`avatar_group`、pop `display.rs:155`、game `display.rs:481`・`avatar_stack`・`avatar_more`、soft `avatar.rs:55`・`avatar_group`・`avatar_status_dot` | 画像の失敗の fallback は `NodeRef::image_status()`（SM-K32）。pop の `hue(f64)`・game の `color(Color)` は各 kit の variant |
| skeleton | `ui::skeleton()`、`.lines(impl IntoValue<usize>)`、`.shape(impl IntoValue<SkeletonShape>)` | basic `primitives.rs:195-225`、pop `display.rs:857`、game `feedback.rs:245-300`、soft | shimmer の look は recipe、reduced motion では止める（policy の field は作らない） |
| spinner | `ui::spinner()`、`.size(impl IntoValue<ControlSize>)` | basic `primitives.rs:175`、pop `display.rs:771`・`dots_loader`、soft `spinner.rs` | 名前は標準の `spinner.loading`。動きは `Animated` 一つ（三つの仕組みを一つに） |
| empty state | `ui::empty_state(title)`、`.description(..)`、`.icon(impl IntoView)`、`.actions(impl IntoView)`、`.announce()` | basic `empty.rs`、pop `display.rs:915`、game `feedback.rs:323`、soft `empty_state.rs` | 読み上げは SM-K23 |
| alert と banner | `ui::alert(title)`、`.description(..)`、`.tone(..)`、`.icon(..)`、`.on_dismiss(..)`、`.variant(..)`（banner は variant） | basic、pop `display.rs:650`、game `feedback.rs:30,145`、soft `banner.rs`、creative `feedback.rs:45`（notice） | 閉じた後の focus の復元は中核（SM-K22） |
| card | `ui::card(children: impl IntoView)`、`.header(impl IntoView)`、`.media(impl IntoView)`、`.footer(impl IntoView)`。`ui::clickable_card(..).on_press(..)`、`ui::selectable_card(selected: impl Into<Binding<bool>>)` | basic `card.rs:121`（slot の closure）、soft `card.rs:43`・clickable・selectable | basic の `FnOnce` の slot は `impl IntoView` に（API-6）。pop の `stat_card` は Tier 3 |
| kbd | `ui::kbd(keys: impl IntoText)`、`ui::kbd_combo(keys: impl IntoValue<Vec<Key>>).modifiers(impl IntoValue<Modifiers>)`（OS の記号は `Capabilities.command_key`） | basic `kbd`・`kbd_group`、soft `kbd.rs`（creative `KeyGlyph` は keyboard ではなく keyframe glyph のため domain に残す） | soft の macOS の分岐（`kbd.rs:14`）を消す（DEP-3） |
| input group | `ui::input_group(label, children: impl IntoChildren)`、`ui::input_addon(text)` | basic `input_group.rs:95`、game `input.rs:45`、soft `input_group.rs` | basic と soft の `input_group_text` は `input_addon` に |

### Tier 3: この再設計で上げる（設計書 17.15 節。中核の API はそこの表）

| 部品 | kit |
|---|---|
| chat（bubble、message、thread、typing、composer） | basic `chat.rs`、pop `messaging.rs`、soft `soft_chat*.rs` |
| code block | basic `code_block/`、soft `code_block.rs`（copy の feedback だけは Tier 1 で中核へ） |
| progress ring・gauge・ring | pop `progress_ring`、game `ring`・`gauge` |
| stat card・KPI | pop `stat_card`、game `kpi` |
| section heading | pop `display.rs:321`、game `surface.rs:69`、soft heading・section |
| key value・property list・metadata | pop `key_value`、game `property_list`、soft `metadata_list`、creative `property_row` |
| aspect ratio | basic `primitives`、soft `aspect_ratio.rs`（中核の `Style::aspect_ratio` で足りるので、kit の部品を消すかどうかを後で決める） |

---

## C.3 kit の hook の行き先

| hook | 行き先 |
|---|---|
| soft `use_carousel` | `interaction::use_carousel`（C.2） |
| soft `use_collapsible` | 消す。`interaction::use_disclosure`（`always_open` を足す） |
| soft `use_lightbox` | soft に残す。`use_dialog` と `use_zoom_pan` の上に作り直す（SM-K05） |
| soft `use_resizable` | 消す。`use_splitter` に collapse と snap を足す（SM-K06） |
| soft `use_scroll_state`、`use_scroll_overflow` | 消す。`NodeRef::scroll_state()`（中核） |
| soft `use_overflow` | 消す（未使用） |
| soft `use_streaming_text` | soft に残す（SM-K08） |
| soft `use_app_shell_mobile` | soft に残す。`AppShellMobile` を enum に（SM-K09） |
| basic の `use_list` を roving だけに使う箇所（`choice.rs:239-245`、`navigation_menu.rs:164-175`） | 上がった部品の中へ（`choice.rs` は `ui::toggle_group`、`navigation_menu.rs` は `ui::navigation_menu`。どちらも中で roving を持つ） |

soft の breakpoint の関数（`viewport_below`、`viewport_from`、`is_compact`、`container_below`）は soft に残す（kit の look の判断）。

---

## C.4 名前の衝突の改名（API-10）

| kit | 現在の名前 | 衝突の相手 | 新しい名前 |
|---|---|---|---|
| 全 kit | `Color`（theme の色の enum） | prelude の `kivact::style::Color` | `ColorName` |
| soft | `text`、`Text` | prelude `text` | `styled_text`、`StyledText` |
| soft | `stack`、`Stack` | prelude `stack`（重ねる箱） | `flex_stack`、`FlexStack`（`hstack`・`vstack` は維持） |
| soft | `grid`、`Grid`、`grid_span`、`Span` | prelude `grid`、`kivact::view::Span` | `layout_grid`、`LayoutGrid`、`grid_span`、`GridSpan` |
| soft | `icon`、`Icon` | prelude `icon`、`kivact::style::Icon` | `icon_view`、`IconView` |
| soft | `list`、`List`、`item`、`Item`、`list_item` | prelude `list` | C.2 の `ui::item_list`・`ui::item` へ上がるので消える |
| soft | `Selector`、`selector` | `kivact::state::Selector` | 消える（`ui::select`） |
| soft | `Instant`（`timestamp.rs:13`） | `std::time::Instant` | `TimestampValue` |
| soft | `Typography`、`Spacing`、`Radius`（token） | `kivact::style::theme::{Typography, Spacing, Radii}` | `TypeTokens`、`SpaceTokens`、`RadiusTokens` |
| soft | `TextStyle` | `kivact::draw::DrawText`（旧 `TextStyle`）・`kivact-types::text::TextStyle` | `TypeStyle` |
| soft | `Motion` | `kivact::motion` | `MotionTokens` |
| basic | `message`、`Message` | `kivact::i18n::message` | `chat_message`、`ChatMessage` |
| basic | `Item` | C.2 の `ui::item` | 消える |
| pop | `ListItem`（`display.rs:566`）、`list_item` | prelude `ListItem` | 消える（`ui::item`） |
| pop | `message` | `kivact::i18n::message` | `chat_message` |
| game | `tree` | `kivact::ui::tree` | 消える（`ui::tree` の restyle） |
| game | `TableCell` | C.2 の `ui::table_cell` | `GameCellLook` |
| creative | `Range`、`range` | `std::ops::Range` | `ValueRange`、`value_range` |
| 全 kit | `badge`、`avatar`、`card`、`calendar`、`combobox`、`toolbar`、`pagination`、`stepper`、`breadcrumb`、`dropzone`、`carousel`、`hover_card`、`collapsible`、`drawer`、`empty_state`、`skeleton`、`spinner`、`alert`、`kbd`、`input_group`、`button_group`、`toggle_group`、`table` 系 | C.2 で上がる `kivact::ui` の名前 | 消える（kit は recipe と variant を出す） |

---

## C.5 kit ごとの作業

作業手順書の unit K-1〜K-7 が、ここを kit ごとに実行する。どの kit も、中核へ上がった部品の kit の版を消したら、その部品の **recipe** と **variant** を kit に足し、catalog の page と E2E を中核の部品で書き直す。

### basic

- 消す（中核へ）: calendar、calendar_range、date_picker、date_range_picker、carousel、collapsible、hover_card、drawer、combobox、toggle_group、button_group、pagination、breadcrumb、navigation_menu、table 系、item・item_group・item_separator、badge、avatar・avatar_group・avatar_group_count、skeleton、spinner、empty_state、alert、card・card_header、kbd・kbd_group、input_group 系
- 消す（Tier 3、中核へ）: chat（bubble・message）、code_block（`rust_lines` は `ui::code_block(..).highlight(..)` に渡す関数として残す）、aspect_ratio
- 残す: chart、questionnaire、input_otp、attachment、trace、sidebar、menubar、native_select、message_scroller、typography、marker、scroll_area、label、separator、aspect_ratio、field_set 系、line_tabs、list_popup
- 状態機械: SM-K02、K12、K15、K16、K24、K28（drawer）
- 規約: C.1 の basic の列

### pop

- 消す（中核へ）: calendar、date_input、date_range_input、dropzone、stepper、breadcrumb、badge、avatar、skeleton、spinner・dots_loader（`spinner` の variant へ）、empty_state、alert、list_item
- 消す（Tier 3、中核へ）: progress_ring、section_heading、stat_card、key_value、thread・message・message_group・typing_message
- 残す: panel、sparkline、timeline_item、upload_row、chip_list・removable_chip、resizable_text_area、notification_menu、agenda_item、tab_label
- 状態機械: SM-K20、K31（pop の分）
- game と recipe の file の骨組みがほぼ同じ（`recipes_field`・`recipes_palette`・`recipes_input`・`recipes_small`・`recipes_collections`）。共通化は kit の外へ出さない（kit は facade だけに依存する）が、両 kit の file の構成を揃え、差分が look の値だけになるようにする

### game

- 消す（中核へ）: toolbar、button_group、tree、breadcrumb、pagination、stepper、badge・tag_badge、avatar・avatar_stack・avatar_more、alert・banner、skeleton、empty_state、input_group 系、dropzone
- 消す（Tier 3、中核へ）: kpi、gauge、ring、property_list、section_heading
- 残す: tool_button、chevron、activity、icon_menu、cut_corner、halo、hud、mono、card_label、navigation_label、`Boxed`・`ChoiceCard`・`Navigation` の variant
- 状態機械: SM-K31（game の分）、K33
- 規約: `ThemeOptions` trait → `theme_with`。palette の literal を `Tokens` へ

### creative

- 消す（中核へ）: outliner の木の振る舞いの部分（`ui::tree` の上に作り直す）、notice（`ui::alert` の variant）、`viewport_surface`（`kivact::view::viewport`、設計書 17.13 節）、property_row の表示の部分（`ui::description_list` で組む）
- 残す: 編集の部品（arrangement、graph_editor、node_editor、timeline、dope_sheet）、color（color_wheel、color_popup、gradient_editor、bezier_curve、curve_preview）、numeric（number、value_slider、knob、fader、`value_range`、property_row、property_state_button）、dock、panel、panel_group、media_tile、tree_row、document_tabs、window、window_button、popup_button、split_button、search_picker、pie_menu、audio（meter、mixer_strip、transport）
- 状態機械: SM-K11、K30
- 編集の部品の zoom・pan の model の不揃い（`zoom: Binding<f64>` ＋ `pan: Binding<Point>`、`ViewTransform`、`GraphView`）を `use_zoom_pan` の `ViewTransform` 一つにする

### soft

- 消す（中核へ）: calendar、date_input、date_range_input、date_time_input、time_input、carousel・`use_carousel`、collapsible・collapsible_group・`use_collapsible`、hover_card、typeahead・selector・multi_selector・tokenizer（`ui::combobox`・`ui::select`）、toolbar、button_group、toggle_button_group、segmented_control（`ui::radio_group` の `Segmented`）、pagination、stepper、breadcrumbs、tab_list の roving、list・item・list_item、table 系、tree_list、badge、avatar・avatar_group・avatar_group_overflow・avatar_status_dot、skeleton、spinner、empty_state、banner（`ui::alert` の variant）、card・clickable_card・selectable_card、kbd、input_group、file_input（`ui::dropzone`）、`use_resizable`・resize_handle（`ui::splitter`）、`use_scroll_state`・`use_scroll_overflow`・`use_overflow`
- 残す: app_shell、layout 系（`flex_stack`、hstack、vstack、center、stack_item、`layout_grid`、`grid_span`、section、form_layout、form_row、divider、scrollable_area、container_reveal）、side_nav・top_nav・mobile_nav（`use_navigation_menu` の上）、nav_heading、outline、more_menu、tab_menu、status_dot、token、`styled_text`、heading、link、`icon_view`、code、markdown、blockquote、citation、thumbnail、timer、timestamp、visually_hidden、indicator、complex_selector、power_search、rich_text_editor、field_status、alert_dialog、bottom_sheet、dialog_header、drawer_header、lightbox、media_overlay、metadata_list、overflow_list、table_selection_toolbar、table_row_status、chat の部品のうち soft 固有のもの（chat_tool_calls、chat_token、streaming_text）、breakpoint の関数
- 消す（Tier 3、中核へ）: code_block、heading・section（`ui::heading`）、metadata_list（`ui::description_list`）、chat_bubble・chat_message・chat_message_list・chat_composer（`ui::chat_*`）、aspect_ratio
- 状態機械: SM-K01、K03〜K10、K13、K14、K16〜K19、K21〜K29、K32
- 規約: C.1 の soft の列。soft は文書が薄い（`pub fn` 889 に `///` 328 行）。消さずに残る公開 item の rustdoc を basic と同じ水準にする（`missing-docs` の検査は今は外れているが、この unit で soft の残る公開 item に書く）

---

## C.6 試験と catalog

- **kit の E2E を一つの binary に:** soft の 33 の test file を `tests/consumer.rs` と `tests/consumer/<topic>.rs`（`#[path]`）にまとめる（basic・pop・game・creative と同じ形）。
- **試験の helper を中核へ:** 4 kit に写されている `evidence()`（`KIVACT_CATALOG_EVIDENCE` のとき screenshot・支援技術の木・debug の木を保存）を `Screen::evidence(name)` に、`options()`（theme と reduced motion）を `RenderOptions::with_theme(theme).reduced_motion()` に、`assert_clean`・`clean` を `Screen::assert_layout_clean()` にする。pop と game の catalog の試験の `chapters`・`chapter`・`mount`・`layout_check` の近い写しも同じ API で書き直す。
- **中核へ上がった部品の試験:** 部品の振る舞いの E2E は `dev/tests/kivact-public-api/tests/e2e/` に一つずつ書く（kit ごとには書かない）。kit の E2E は、kit の recipe が当たっていることと、kit に残る部品の振る舞いを確かめる。`theme_conformance`（`dev/tests/kivact-public-api/tests/e2e/theme_conformance.rs:285-291`）は C.1 の KIT-3 の確認を足す。
- **catalog の font:** NotoSansJP（4 つの catalog で同じ bytes）、Inter（basic・pop）、GeistMono（basic・creative）を `examples/shared-assets/fonts/` に一つずつ置き、各 catalog の `fonts.rs` は相対 path の `include_bytes!` で読む。font の loop は `App::fonts(iter)`。license の文書は同じ directory に置き、`THIRD-PARTY-NOTICES.md` を直す。
- **creative の feature `gpu-interop`:** 消す（設計書 17.3 節の DEP-4。`viewport` は中核へ移り、`gpu_frame` は facade の feature `gpu-interop` で出る）。
- **pop と game の recipe:** 値だけが違う recipe を `UiTokens` の値にし、file を形の違う recipe だけにする（設計書 17.7・18.15 節、unit K-0）。
- **catalog の page:** 中核へ上がった部品の page は、その kit の look（recipe）で中核の部品を見せる page に書き直す（page を消さない: kit の利用者の見本）。
