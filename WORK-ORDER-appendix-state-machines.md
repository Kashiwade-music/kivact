# 付録: 暗黙の状態マシンの修正内容

[作業指示書](WORK-ORDER-devtools-multi-window.md) の task が適用する entry の全文。基準 commit `96e22b6b` のコードを読んで作った（どれもまだコンパイルしていない）。行番号は基準 commit のもの。前の task の変更で行がずれていたら、entry に書いた型名・関数名で探す。entry の内容がコードと合わないときは、作業指示書 0.2 の 1. に従って止めて報告する。

## entry と担当の task

| entry | 対象 | 担当の task |
|---|---|---|
| SM-SOFT-01〜10 | kivact-ui-soft | T1.3 |
| SM-INT-04、05、06、13 | kivact-ui-creative（dock、outliner） | T1.4 |
| SM-INT-14（creative の部分） | kivact-ui-creative（outliner） | T9.2 |
| SM-INT-01、02、03、07、08、09（menu の部分）、10、11、12、14（kivact-interaction の部分）、15、16 | kivact-interaction | T8.2 |
| SM-INT-09（core の部分: `kivact::view::is_mounted`） | kivact-core | T7.2 |
| SM-INT-17 | kivact-style（`Transform.stage`） | T4.2 |
| SM-UI-01〜10 | kivact-ui | T9.1 |
| SM-UI-08 の試験の案 | `dev/tests/kivact-public-api/tests/e2e/ui_standard.rs` | T9.1 |
| SM-UI-09 の試験の案 | `crates/kivact-ui-soft/tests/consumer/selector_touch.rs` | T9.1（担当ファイルに入っている） |
| SM-CORE-01、02、08、09、10、11、12 | kivact-core | T7.2 |
| SM-CORE-14 | kivact-core の部分 | T7.2 |
| SM-CORE-14 | kivact-test の部分（`driver.rs`） | T8.1 |
| SM-CORE-03、17 | kivact-runtime | T3.2 |
| SM-CORE-04 | dev/tests/gpu-interop-producer | T9.2 |
| SM-CORE-05、16 | arvane-native | T4.1 |
| SM-CORE-06 | arvane-virtual | T5.1 |
| SM-CORE-07 | kivact-test | T8.1 |
| SM-CORE-15 | kivact-engine の部分（`ArvaneTestHost::scale(window)`） | T6.1 |
| SM-CORE-15 | kivact-test の部分 | T8.1 |
| SM-CORE-13 | kivact-engine | T6.1 |
| 7 章（新しい規則で初めて検出される既存の箇所） | 全 crate。印を書くだけで、コードは変えない | T15.4 |

試験の案を持つ entry は、案を名指した既存の file に、その file の helper と import を使って足す（新しい file を作らない）。足す task は上の表のとおり。足した試験は走らせない（C1 が回す）。

各部の「KEEP table」と 7 章の表は、T15.4 で field ごとの印の理由を書くときに使う（残す理由の分類）。「Borderline decisions」は記録であり、適用するものは無い。

entry が「struct の前の印を残す」「書き換える」と書いているところは、そのとおりにしてよい。T1.1 で xtask の検査が強まると、struct の前の印は error になるが、それは T15.4 が field ごとの印に付け替える（各 entry の task で付け替えなくてよい）。ただし、entry の置き換えのコードに field ごとの印が書いてあるときは、そのとおりに写す（SM-CORE-01、SM-SOFT-02）。

分けた・移した型（`Model`・`Host`・`WinitWindow`・`VirtualShell`・`Screen`・`ViewState`）の行の探し方は、作業指示書 0.2 の 12. の対応表にある（T15.4 と、W3 以降に型を作り変える task が使う）。`VirtualShell` を分ける SM-CORE-06 は、新しい型ごとの field の印の文も書いている。


---

# Implicit state machines: kivact-ui, kivact-ui-basic, kivact-ui-pop, kivact-ui-game

Base: `feat/kashiwade/devtools-multi-window` = main 96e22b6b. Read-only review; nothing built or run.
Rules applied: architecture.md S-1..S-9, the Machine rules, and definitions (a)–(h) from the work order.

## 1. Summary counts

| Item | Count |
|---|---|
| `xtask-allow(state-fields)` lines reviewed (116 annotated structs or fields) | 170 |
| Unannotated sites scanned by hand (Cell/RefCell/Signal/var, local signals, effects, flags, Machine states and inputs) | 27 |
| KEEP (annotated lines) | 165 (date_input/machine.rs:135 moved to SM-UI-10) |
| FIX (annotated sites) | 4 entries: SM-UI-01/02/03 cover 4 annotated lines (button.rs:91, radio_group.rs:23, menu.rs:263, menu.rs:265); SM-UI-10 deletes the dead allow at date_input/machine.rs:135 |
| UNANNOTATED-FIX | 6 entries: SM-UI-04..07, SM-UI-08 (Variants), SM-UI-09 (SelectionField surface) |

Order to apply:
- SM-UI-01..07, SM-UI-09 and SM-UI-10 are independent of each other. Each touches only the files it names.
- SM-UI-08 must be applied after SM-UI-01 and SM-UI-02. Their old constructors install recipe styles before `.variant(..)` runs; with a plain `Variants` value those styles would silently lose every variant.

Notes (KEEP, not changed):
- `crates/kivact-ui/src/combobox/labels.rs:22-56` `KnownLabels`: `new()` runs before `observe()` because of the selection↔list dependency cycle in `render.rs:493-523`. It is already justified as an S-9 cache by its `effect-writes` allow. KEEP.
- `crates/kivact-ui/src/progress.rs:20-46` `ProgressMotionPhase` is an explicit Machine that only replaces a bool. Architecture.md calls such a machine unnecessary. It is redundant but not implicit, so it is out of scope.
- Every NodeBuilder implementor is already exempt from `state-fields` (dev/xtask/src/local_checks.rs:118-131). The type-level allows on builders in the KEEP table are therefore redundant. Deleting them is an optional cleanup.

## 2. KEEP table

| file:line | struct.field | category |
|---|---|---|
| crates/kivact-ui/src/alert.rs:57 | Alert.* | builder option |
| crates/kivact-ui/src/avatar.rs:18 | Avatar.* | builder option |
| crates/kivact-ui/src/avatar_group.rs:9 | AvatarGroup.* | builder option |
| crates/kivact-ui/src/badge.rs:7 | Badge.* | builder option |
| crates/kivact-ui/src/breadcrumb.rs:22 | Breadcrumb.activate | builder option |
| crates/kivact-ui/src/breadcrumb.rs:24 | Breadcrumb.item | builder option |
| crates/kivact-ui/src/button_group.rs:119 | ButtonGroupCell.separator | builder option |
| crates/kivact-ui/src/button_group.rs:180 | ButtonGroupMember.solid | builder option |
| crates/kivact-ui/src/button_group.rs:182 | ButtonGroupMember.keep_size | builder option |
| crates/kivact-ui/src/calendar_values.rs:23 | CalendarDayVisual.* | passive record (recipe projection) |
| crates/kivact-ui/src/calendar_values.rs:31 | CalendarDayVisual.today | passive record (recipe projection) |
| crates/kivact-ui/src/calendar_values.rs:33 | CalendarDayVisual.in_month | passive record (recipe projection) |
| crates/kivact-ui/src/card.rs:12 | Card.* | builder option |
| crates/kivact-ui/src/carousel.rs:13 | Carousel.* | builder option |
| crates/kivact-ui/src/carousel.rs:139 | Layout.* | passive record (immutable composition plan) |
| crates/kivact-ui/src/carousel_geometry.rs:5 | PageRequest.* | passive record (Input snapshot) |
| crates/kivact-ui/src/chat_composer.rs:11 | ChatComposer.* | builder option |
| crates/kivact-ui/src/chat_message.rs:9 | ChatMessage.* | builder option |
| crates/kivact-ui/src/chat_thread.rs:11 | ChatThread.* | builder option (history_revision is a caller-owned generation input) |
| crates/kivact-ui/src/chat_values.rs:73 | ChatMessageState.* | passive record (recipe projection) |
| crates/kivact-ui/src/chat_values.rs:93 | ChatComposerState.* | passive record (recipe projection) |
| crates/kivact-ui/src/code_block.rs:16 | CodeBlock.* | builder option |
| crates/kivact-ui/src/code_block.rs:186 | CodeSource.* | passive record (immutable composition plan) |
| crates/kivact-ui/src/code_values.rs:2 | CodeBlockState.* | passive record (recipe projection) |
| crates/kivact-ui/src/collection_values.rs:41 | ToolbarState.gap | passive record (recipe projection) |
| crates/kivact-ui/src/collection_values.rs:43 | ToolbarState.centered | passive record (recipe projection) |
| crates/kivact-ui/src/collection_values.rs:45 | ToolbarState.rtl | passive record (recipe projection) |
| crates/kivact-ui/src/collection_values.rs:78 | ItemRowState.* | passive record (recipe projection) |
| crates/kivact-ui/src/command_palette.rs:20 | CommandPalette.* | builder option |
| crates/kivact-ui/src/component_values.rs:6 | AlertState.* | passive record (recipe projection) |
| crates/kivact-ui/src/component_values.rs:19 | CardState.* | passive record (recipe projection) |
| crates/kivact-ui/src/component_values.rs:37 | InputGroupState.* | passive record (recipe projection) |
| crates/kivact-ui/src/component_values.rs:51 | InputGroupMemberState.* | passive record (recipe projection) |
| crates/kivact-ui/src/data_table_tokens.rs:37 | DataTableStickyPaint.* | passive record (Tokens) |
| crates/kivact-ui/src/date_range_input.rs:396 | RangeEditors.* | caller binding (projections of the one RangeDraft machine) |
| crates/kivact-ui/src/date_time_model.rs:52 | DateTimeFields.* | caller binding (projections of the one caller binding) |
| crates/kivact-ui/src/date_time_sheet.rs:10 | Inputs.* | caller binding |
| crates/kivact-ui/src/dialog.rs:32 | Dialog.* | builder option |
| crates/kivact-ui/src/disclosure.rs:19 | Disclosure.* | builder option |
| crates/kivact-ui/src/disclosure_tokens.rs:7 | DisclosureMotion.* | passive record (Tokens) |
| crates/kivact-ui/src/disclosure_values.rs:10 | DisclosureRecipeState.* | passive record (recipe projection) |
| crates/kivact-ui/src/dropzone.rs:11 | Dropzone.* | builder option |
| crates/kivact-ui/src/empty_state.rs:23 | EmptyState.* | builder option (comment text changes under SM-UI-06) |
| crates/kivact-ui/src/field.rs:18 | Field.* | builder option |
| crates/kivact-ui/src/group_types.rs:9 | ButtonGroupTokens.joined | passive record (Tokens) |
| crates/kivact-ui/src/group_types.rs:36 | ButtonGroupState.rtl | passive record (recipe projection) |
| crates/kivact-ui/src/group_types.rs:44 | ButtonGroupMemberState.first | passive record (recipe projection) |
| crates/kivact-ui/src/group_types.rs:46 | ButtonGroupMemberState.last | passive record (recipe projection) |
| crates/kivact-ui/src/group_types.rs:66 | ToggleGroupState.spacing | passive record (recipe projection) |
| crates/kivact-ui/src/group_types.rs:68 | ToggleGroupState.rtl | passive record (recipe projection) |
| crates/kivact-ui/src/group_types.rs:77 | ToggleGroupItemState.first | passive record (recipe projection) |
| crates/kivact-ui/src/group_types.rs:79 | ToggleGroupItemState.last | passive record (recipe projection) |
| crates/kivact-ui/src/hover_card.rs:24 | HoverCard.* | builder option |
| crates/kivact-ui/src/indicator_values.rs:21 | BadgeState.* | passive record (recipe projection) |
| crates/kivact-ui/src/input_group.rs:15 | InputGroup.* | builder option |
| crates/kivact-ui/src/item_row.rs:17 | RowContents.* | passive record (immutable composition plan) |
| crates/kivact-ui/src/list.rs:15 | List.* | builder option |
| crates/kivact-ui/src/menu.rs:37 | Menu.offset | builder option |
| crates/kivact-ui/src/menu.rs:40 | Menu.align | builder option |
| crates/kivact-ui/src/menu.rs:42 | Menu.at_least_anchor_width | builder option |
| crates/kivact-ui/src/menu.rs:44 | Menu.item | builder option |
| crates/kivact-ui/src/menu.rs:259 | Level.align | passive record (immutable composition plan) |
| crates/kivact-ui/src/menu.rs:261 | Level.at_least_anchor_width | passive record (immutable composition plan) |
| crates/kivact-ui/src/navigation_values.rs:24 | StepperItemState.collapsed | passive record (recipe projection) |
| crates/kivact-ui/src/navigation_values.rs:26 | StepperItemState.disabled | passive record (recipe projection) |
| crates/kivact-ui/src/navigation_values.rs:28 | StepperItemState.optional | passive record (recipe projection) |
| crates/kivact-ui/src/navigation_values.rs:71 | BreadcrumbItemState.* | passive record (recipe projection) |
| crates/kivact-ui/src/navigation_values.rs:108 | PaginationSummary.* | passive record (recipe projection) |
| crates/kivact-ui/src/pagination.rs:15 | Pagination.first_last | builder option |
| crates/kivact-ui/src/pagination.rs:18 | Pagination.has_more | caller binding |
| crates/kivact-ui/src/pagination.rs:20 | Pagination.page_size | caller binding |
| crates/kivact-ui/src/pagination.rs:26 | Pagination.item | builder option |
| crates/kivact-ui/src/pagination.rs:29 | Pagination.summary | builder option |
| crates/kivact-ui/src/pagination.rs:488 | ControlContent.* | passive record (immutable composition plan) |
| crates/kivact-ui/src/pagination.rs:698 | PageExtras.* | passive record (immutable composition plan) |
| crates/kivact-ui/src/progress.rs:48 | Progress.* | builder option (fixed_indeterminate is derived once from an immutable input) |
| crates/kivact-ui/src/progress_tokens.rs:3 | ProgressMotion.* | passive record (Tokens) |
| crates/kivact-ui/src/slider.rs:25 | Slider.* | builder option |
| crates/kivact-ui/src/slider.rs:173 | Geometry.* | independent fact (layout measurement) |
| crates/kivact-ui/src/spinner.rs:7 | Spinner.* | builder option |
| crates/kivact-ui/src/split.rs:28 | Split.* | builder option |
| crates/kivact-ui/src/splitter.rs:16 | Splitter.* | builder option |
| crates/kivact-ui/src/stat.rs:8 | Stat.* | builder option |
| crates/kivact-ui/src/stepper.rs:117 | Stepper.min_step_width | builder option |
| crates/kivact-ui/src/table.rs:114 | TableCell.header | builder option |
| crates/kivact-ui/src/table.rs:117 | TableCell.width | builder option |
| crates/kivact-ui/src/table_values.rs:14 | RowStatus.* | passive record (immutable descriptor) |
| crates/kivact-ui/src/tabs.rs:27 | Tabs.* | builder option |
| crates/kivact-ui/src/toggle_button.rs:83 | ToggleButton.* | builder option |
| crates/kivact-ui/src/toggle_group.rs:24 | ToggleGroup.spacing | builder option |
| crates/kivact-ui/src/toggle_group.rs:26 | ToggleGroup.icon_only | builder option |
| crates/kivact-ui/src/toggle_group.rs:28 | ToggleGroup.on_edit | builder option |
| crates/kivact-ui/src/toggle_group.rs:30 | ToggleGroup.item | builder option |
| crates/kivact-ui/src/toolbar.rs:16 | Toolbar.start | builder option |
| crates/kivact-ui/src/toolbar.rs:18 | Toolbar.center | builder option |
| crates/kivact-ui/src/toolbar.rs:20 | Toolbar.end | builder option |
| crates/kivact-ui/src/toolbar.rs:24 | Toolbar.gap | builder option |
| crates/kivact-ui/src/tooltip.rs:12 | Tooltip.* | builder option |
| crates/kivact-ui/src/tree.rs:26 | Tree.item | builder option |
| crates/kivact-ui/src/tree.rs:28 | Tree.marker_content | builder option |
| crates/kivact-ui/src/tree.rs:31 | Tree.activate | builder option |
| crates/kivact-ui/src/tree.rs:33 | Tree.header | builder option |
| crates/kivact-ui/src/tree.rs:35 | Tree.reveal | builder option |
| crates/kivact-ui/src/tree_values.rs:2 | TreeRowRecipeState.* | passive record (recipe projection) |
| crates/kivact-ui/src/values.rs:106 | ButtonState.* | passive record (recipe projection) |
| crates/kivact-ui/src/values.rs:156 | ToggleButtonState.* | passive record (recipe projection) |
| crates/kivact-ui/src/values.rs:237 | SwitchDragState.* | passive record (recipe projection of switch_drag::State::visual) |
| crates/kivact-ui/src/values.rs:251 | CheckableState.* | passive record (recipe projection) |
| crates/kivact-ui/src/values.rs:284 | DialogState.* | passive record (recipe projection) |
| crates/kivact-ui/src/values.rs:309 | SelectTriggerState.* | passive record (recipe projection) |
| crates/kivact-ui/src/values.rs:356 | DataTableRowState.* | passive record (recipe projection) |
| crates/kivact-ui/src/calendar/navigation.rs:72 | SwipeSnapshot.* | passive record (Input snapshot) |
| crates/kivact-ui/src/calendar/state.rs:9 | Shared.* | passive record (immutable composition plan) |
| crates/kivact-ui/src/calendar/state.rs:20 | Shared.outside | passive record (immutable composition plan) |
| crates/kivact-ui/src/calendar/state.rs:23 | Shared.variable_rows | passive record (immutable composition plan) |
| crates/kivact-ui/src/calendar/state.rs:28 | Shared.year_navigation | passive record (immutable composition plan) |
| crates/kivact-ui/src/calendar/state.rs:30 | Shared.month_title | passive record (immutable composition plan) |
| crates/kivact-ui/src/calendar/state.rs:32 | Shared.range | passive record (immutable composition plan) |
| crates/kivact-ui/src/calendar/state.rs:34 | Shared.marks | passive record (immutable composition plan) |
| crates/kivact-ui/src/calendar/view.rs:27 | Calendar.* | builder option |
| crates/kivact-ui/src/calendar/view.rs:33 | Calendar.month | builder option (uncontrolled initial month) |
| crates/kivact-ui/src/calendar/view.rs:38 | Calendar.outside | builder option |
| crates/kivact-ui/src/calendar/view.rs:40 | Calendar.week_start | builder option |
| crates/kivact-ui/src/calendar/view.rs:42 | Calendar.on_commit | builder option |
| crates/kivact-ui/src/calendar/view.rs:44 | Calendar.autofocus | builder option |
| crates/kivact-ui/src/calendar/view.rs:46 | Calendar.displayed_month | caller binding |
| crates/kivact-ui/src/calendar/view.rs:49 | Calendar.min | builder option |
| crates/kivact-ui/src/calendar/view.rs:51 | Calendar.max | builder option |
| crates/kivact-ui/src/calendar/view.rs:54 | Calendar.max_span | builder option |
| crates/kivact-ui/src/calendar/view.rs:56 | Calendar.variable_rows | builder option |
| crates/kivact-ui/src/calendar/view.rs:58 | Calendar.marks | builder option |
| crates/kivact-ui/src/calendar/view.rs:61 | Calendar.weekend | builder option |
| crates/kivact-ui/src/calendar/view.rs:63 | Calendar.year_navigation | builder option |
| crates/kivact-ui/src/calendar/view.rs:65 | Calendar.month_title | builder option |
| crates/kivact-ui/src/calendar/view.rs:67 | Calendar.range | builder option |
| crates/kivact-ui/src/calendar/view.rs:69 | Calendar.on_activate | builder option |
| crates/kivact-ui/src/calendar/view.rs:397 | CalendarRules.* | passive record (immutable composition plan) |
| crates/kivact-ui/src/combobox/data.rs:15 | Combobox.* | builder option |
| crates/kivact-ui/src/combobox/render.rs:219 | EditorState.* | passive record (immutable composition plan) |
| crates/kivact-ui/src/combobox/render.rs:465 | Mounted.* | passive record (immutable composition plan) |
| crates/kivact-ui/src/combobox/tokens.rs:3 | ComboboxTokens.* | passive record (Tokens) |
| crates/kivact-ui/src/data_table/rows.rs:68 | Grid.* | passive record (immutable composition plan) |
| crates/kivact-ui/src/data_table/rows.rs:104 | StickyColumns.* | independent fact (layout measurement) |
| crates/kivact-ui/src/data_table/runtime.rs:32 | ModelPlan.* | passive record (immutable composition plan) |
| crates/kivact-ui/src/data_table/types.rs:33 | DataTableContext.* | caller binding |
| crates/kivact-ui/src/data_table/types.rs:101 | DataTable.* | builder option |
| crates/kivact-ui/src/date_input/machine.rs:52 | Snapshot.* | passive record (Input snapshot) |
| crates/kivact-ui/src/date_input/machine.rs:140 | TypedSnapshot.* | passive record (Input snapshot) |
| crates/kivact-ui/src/date_input/view.rs:442 | EditorInputs.* | caller binding |
| crates/kivact-ui/src/time_input/machine.rs:36 | Snapshot.* | passive record (Input snapshot) |
| crates/kivact-ui-basic/src/attachment.rs:52 | Attachment.* | builder option |
| crates/kivact-ui-basic/src/attachment.rs:69 | Cfg.* | passive record (style projection) |
| crates/kivact-ui-basic/src/card_header.rs:7 | CardHeader.* | builder option |
| crates/kivact-ui-basic/src/native_select.rs:27 | NativeSelect.* | builder option |
| crates/kivact-ui-basic/src/questionnaire.rs:100 | QuestionnaireStep.* | passive record (authored data) |
| crates/kivact-ui-basic/src/trace.rs:16 | TraceLine.* | builder option |
| crates/kivact-ui-basic/src/chart/model.rs:99 | Frame.* | passive record (per-paint arguments) |
| crates/kivact-ui-basic/src/chart/view.rs:41 | Chart.* | builder option |
| crates/kivact-ui-basic/src/sidebar/menu.rs:21 | SidebarGroup.* | builder option |
| crates/kivact-ui-basic/src/sidebar/menu.rs:162 | SidebarMenuButton.* | builder option |
| crates/kivact-ui-basic/src/sidebar/menu.rs:177 | ButtonCfg.* | passive record (style projection) |
| crates/kivact-ui-basic/src/sidebar/menu.rs:404 | SidebarMenuSubButton.* | builder option |
| crates/kivact-ui-basic/src/sidebar/panel.rs:16 | Sidebar.* | builder option |
| crates/kivact-ui-pop/src/recipes_avatar.rs:10 | AvatarLook.* | passive record (recipe look) |
| crates/kivact-ui-game/src/kpi.rs:9 | Kpi.* | builder option |

Unannotated sites reviewed and kept (no change):

| file:line | site | category |
|---|---|---|
| crates/kivact-ui/src/data_table/navigation.rs:18 | NavigationCache(Rc<RefCell<HashMap>>) | named S-9 type (pure mount-registry cache) |
| crates/kivact-ui/src/data_table/filters.rs:297 | `sent = var(Sent<..>)` | named S-9 type (Sent) |
| crates/kivact-ui/src/data_table/filters.rs:85,122 | popover `open`, per-opening `draft` | independent fact / draft value |
| crates/kivact-ui/src/combobox/labels.rs:23 | KnownLabels.rows | named S-9 type (cache, existing effect-writes allow) |
| crates/kivact-ui/src/combobox/notices.rs:9 | Notices.label | independent fact (message argument) |
| crates/kivact-ui/src/combobox/render.rs:554 | `local_open` | independent fact (uncontrolled default) |
| crates/kivact-ui/src/date_input/view.rs:250,351 | `open`, `announcement` | independent fact / Report text |
| crates/kivact-ui/src/date_range_input.rs:190 | `open` | independent fact |
| crates/kivact-ui/src/drawer_swipe.rs:14 | `pulled` | independent fact (drag measurement) |
| crates/kivact-ui/src/button.rs:351, progress.rs:234, kivact-ui-game/src/surface.rs:73 | clock `phase`/`time` | independent fact (clock measurement) |
| crates/kivact-ui/src/tag_input.rs:160 | `pending` | independent fact (draft text) |
| crates/kivact-ui-basic/src/chart/view.rs:206-209 | `hit`, `at`, `plot` | independent fact (pointer measurement / paint-size cache) |
| crates/kivact-ui-pop/src/notifications.rs:88, resize.rs:54 | `open`, `height` | independent fact |
| crates/kivact-ui-basic/src/questionnaire.rs:54-61 | Phase::Submitted{at,answers} | named S-9 type (sent record inside a Machine) |

## 3. FIX entries (annotated sites)

### SM-UI-01 Button: Cell slots filled after construction, a "size already set" flag, duplicated icon facts

- Location: crates/kivact-ui/src/button.rs:91-240 and 263-340.
- Current fields (verbatim):

```rust
// xtask-allow(state-fields): NodeBuilder configuration and optional children; busy runtime phases live in BusyPhase.
pub struct Button {
    root: Node,
    label: Value<String>,
    icon: Option<View>,
    trailing: Option<View>,
    children: Vec<View>,
    settings: Rc<RefCell<ButtonOptions>>,
    variants: Variants,
    busy: Rc<Cell<Value<bool>>>,
    expanded: Rc<Cell<Option<Value<bool>>>>,
    state: Value<ButtonState>,
    hide_label: Rc<Cell<Option<Value<bool>>>>,
}
/// What the builder knows of the button once it is configured.
#[derive(Clone)]
struct ButtonOptions {
    size: Value<ControlSize>,
    size_explicit: bool,
    leading_icon: bool,
    trailing_icon: bool,
}
```

- Why it is implicit:
  - (f) S-7: `button()` builds the `state` memo and the root recipe style before the inputs exist (button.rs:136-169). The `busy`, `hide_label` and `settings` slots are filled later by builder calls (lines 186, 210, 233), and `expanded` is filled in `finish()` (line 269).
  - (c): `size_explicit` records that `.size()` already ran and gates `default_size()` (lines 193, 200).
  - (h) S-4: `leading_icon`/`trailing_icon` duplicate `icon.is_some()`/`trailing.is_some()` (lines 219, 225).
- Replacement code. Replace lines 91-240 with this (the `variant` impl at 241-257 and `impl NodeBuilder` at 258-262 stay unchanged):

```rust
// xtask-allow(state-fields): NodeBuilder configuration and optional children; busy runtime phases live in BusyPhase.
pub struct Button {
    root: Node,
    label: Value<String>,
    icon: Option<View>,
    trailing: Option<View>,
    children: Vec<View>,
    size: ButtonSize,
    variants: Variants,
    busy: Value<bool>,
    input: Value<PartState>,
    hide_label: Option<Value<bool>>,
}
/// Where the button's size comes from: a default a group may replace, or
/// the caller's `.size(..)`, which nothing replaces.
#[derive(Clone, Copy)]
enum ButtonSize {
    Default(ControlSize),
    Explicit(Value<ControlSize>),
}
impl ButtonSize {
    fn value(self) -> Value<ControlSize> {
        match self {
            Self::Default(size) => size.into_value(),
            Self::Explicit(size) => size,
        }
    }
}
/// A themed button activated through the shared button part ([`use_button`]).
pub fn button(label: impl IntoText) -> Button {
    let label = label.into_text_value();
    let part = use_button().control();
    let input = part.state();
    let root = node()
        .part(part)
        .focusable_when_disabled(matches!(
            use_theme()
                .get()
                .ext::<kivact_interaction::InteractionPolicy>()
                .button
                .disabled_focus,
            DisabledButtonFocus::Retain
        ))
        .name(label);
    Button {
        root,
        label,
        icon: None,
        trailing: None,
        children: Vec::new(),
        size: ButtonSize::Default(super::values::default_control_size()),
        variants: Variants::default(),
        busy: false.into_value(),
        input,
        hide_label: None,
    }
}
impl Button {
    /// Set the size explicitly, overriding a size provider or group default.
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = ButtonSize::Explicit(size.into_value());
        self
    }

    /// Set the default for this button when composing it into a group.
    pub fn default_size(mut self, size: ControlSize) -> Self {
        if let ButtonSize::Default(_) = self.size {
            self.size = ButtonSize::Default(size);
        }
        self
    }
    /// Show the busy indicator and ignore presses while true (constant or
    /// reactive). Its spin and reveal use the theme's [`ButtonBusyMotion`].
    pub fn busy(mut self, busy: impl IntoValue<bool>) -> Self {
        let busy = busy.into_value();
        self.root = self.root.busy(busy);
        self.busy = busy;
        self
    }
    /// A view before the label, in a box the theme's
    /// [`UiRecipes::button_icon`](super::UiRecipes) styles (a kit sizes
    /// the icon there by the button's size).
    /// The recipe knows it from [`ButtonState::leading_icon`].
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    /// A view after the label, in a box styled like [`Self::icon`]'s; the
    /// recipe knows it from [`ButtonState::trailing_icon`].
    pub fn trailing(mut self, icon: impl IntoView) -> Self {
        self.trailing = Some(icon.into_view());
        self
    }
    /// Keep the accessible name while omitting the visible label (while
    /// true; constant or reactive). A button with a leading
    /// [`icon`](Self::icon) and no visible label is an icon button: its
    /// recipe knows it from [`ButtonState::icon_only`] and draws a square.
    #[track_caller]
    pub fn hide_label(mut self, hide: impl IntoValue<bool>) -> Self {
        self.hide_label = Some(hide.into_value());
        self
    }
    pub fn child(mut self, child: impl IntoView) -> Self {
        self.children.push(child.into_view());
        self
    }
}
```

  Replace `fn finish` (lines 263-340) with:

```rust
impl Button {
    /// Finish the root, including themed content and busy indicator.
    fn finish(self) -> Node {
        let Button {
            mut root,
            label,
            icon,
            trailing,
            children,
            size,
            variants,
            busy,
            input,
            hide_label,
        } = self;
        // A popup or disclosure attaches its trigger before the button is
        // finished; the recipe draws the button as that trigger.
        let expanded = root.accessible_expanded();
        // Busy ignores presses as the caller's `.disabled(..)` does.
        let disabled = root.accessible_disabled();
        let root = root.disabled(move || busy.get() || disabled.is_some_and(|d| d.get()));
        let size = size.value();
        let (leading_icon, trailing_icon) = (icon.is_some(), trailing.is_some());
        let state = memo(move || ButtonState {
            input: input.get(),
            size: size.get(),
            busy: busy.get(),
            expanded: expanded.map(|open| open.get()),
            leading_icon,
            trailing_icon,
            icon_only: leading_icon && hide_label.is_some_and(|hide| hide.get()),
        });
        // The text style, then the recipe: base layers below every `.style(..)` of the caller.
        let root = with_base_recipe_style(
            root.base_style(super::helpers::single_line_text()),
            |r| r.button_recipe(),
            &variants,
            move || state.get(),
        );
        let slot = |icon: View| {
            with_recipe_style(
                node().child(icon),
                |r| r.button_icon_recipe(),
                &variants,
                move || state.get(),
            )
        };
        let (phase, reveal_pending) = busy_animation(state);
        let content = with_recipe_style(
            row((
                icon.map(slot),
                super::helpers::visible_label(label, hide_label),
                trailing.map(slot),
                children,
            )),
            |r| r.button_content_recipe(),
            &variants,
            move || {
                let mut state = state.get();
                state.busy &= !reveal_pending.get();
                state
            },
        );
        let indicator = show(
            move || state.get().busy,
            move || {
                let motion = use_theme()
                    .get()
                    .ext::<crate::UiTokens>()
                    .button
                    .busy_motion;
                let delay = if kivact_core::platform::is_motion_reduced().get() {
                    0
                } else {
                    motion.reveal_delay_ms
                };
                let fade = (delay != 0 || motion.reveal_fade_ms != 0).then(|| {
                    Animated::from(
                        0.0f64,
                        1.0f64,
                        Transition::tween(
                            motion.reveal_fade_ms as f64,
                            Easing::CubicBezier(0.25, 0.1, 0.25, 1.0),
                        )
                        .delay(delay as f64)
                        .essential(),
                    )
                });
                with_recipe_style(
                    node(),
                    |r| r.button_busy_recipe(),
                    &variants,
                    move || state.get(),
                )
                .style(move || {
                    let base = style().rotate(phase.get() * 360.0);
                    if let Some(fade) = fade {
                        base.opacity(fade.get())
                    } else {
                        base
                    }
                })
            },
        );
        root.child(content).child(indicator)
    }
}
```

  Imports at the top of button.rs:
  - line 18: `use super::layer::with_recipe_style;` → `use super::layer::{with_base_recipe_style, with_recipe_style};`
  - lines 29-30: delete `use std::cell::{Cell, RefCell};` and `use std::rc::Rc;`
  - add `use kivact_core::custom::PartState;`
- Every read or write of the old fields, and what it becomes:
  - button.rs:98 `settings`, 100 `busy`, 101 `expanded`, 102 `state`, 103 `hide_label` (declarations): replaced by the new struct above. `state` is no longer a field; it becomes a local in `finish`.
  - button.rs:105-122 `ButtonOptions` + `impl Default`: deleted; `ButtonSize` replaces it.
  - button.rs:126-129 (settings created), 130-131 (expanded/read_expanded), 132-133 (busy/read_loading), 134-135 (hide_label/read_hide), 136 (busy_value): deleted.
  - button.rs:137 `use_button().disabled(busy_value).control()` → `use_button().control()`. Busy now reaches `disabled` through the `root.disabled(..)` call in `finish`, which ORs it with the caller's value. This keeps the button part attached at construction, so semantic modifier order is unchanged.
  - button.rs:138 `let input = part.state();` stays, and `input` is stored in the new field.
  - button.rs:139-151 (`read` and the `state` memo): moved into `finish` and built from plain fields.
  - button.rs:152-169 (`variants` and the root built with `with_recipe_style(node().style(single_line_text())…)`): becomes the plain `root` in `button()`. `single_line_text()` and the recipe move to `finish` as base styles, added in that order. The layer order [single_line, recipe, caller…] is preserved.
  - button.rs:185-188 `configure`: deleted.
  - button.rs:190-195 `size` → `ButtonSize::Explicit`.
  - button.rs:198-204 `default_size` → replaces only `ButtonSize::Default`.
  - button.rs:209-210 `self.busy.set(busy)` → `self.busy = busy`.
  - button.rs:219 and 225 `configure(|s| s.leading_icon/trailing_icon = true)`: deleted; derived in `finish` from `icon.is_some()` / `trailing.is_some()`.
  - button.rs:233 `self.hide_label.set(Some(..))` → `self.hide_label = Some(..)`.
  - button.rs:268-270 `self.expanded.set(Some(..))` → local `let expanded = root.accessible_expanded();`.
  - button.rs:271 `let state = self.state;` → the local memo.
  - button.rs:285 `self.hide_label.get()` → `hide_label`.
  - crates/kivact-ui/src/button_group.rs:212 `self.button.default_size(..)`: call unchanged; behaviour is the same.
- xtask-allow lines to delete: none. button.rs:91 stays as written. Button is a NodeBuilder and is exempt anyway.
- Public signatures: unchanged. `size`, `busy`, `icon`, `trailing`, `hide_label` and `default_size` change only from `self` to `mut self`, which is not part of the signature.
- Tests:
  - dev/tests/kivact-public-api/tests/e2e/environment.rs: `advanced_semantics_publish_roles_and_reactive_states` (busy) and `control_size_provider_prefers_the_nearest_default_and_explicit_sizes` (explicit vs default size).
  - dev/tests/kivact-public-api/tests/e2e/ui_standard.rs: `a_button_is_a_named_button_that_enter_space_click_and_invoke_press`, `a_disabled_button_is_disabled_and_ignores_every_way_of_pressing_it`, `a_recipe_in_the_theme_replaces_the_look_of_its_component_only` and `what_the_caller_sets_comes_after_the_recipe_and_wins` (layer order).
  - dev/tests/kivact-public-api/tests/e2e/ui_toolbar.rs: `button_group_is_one_tab_stop` (`default_size`).
  - crates/kivact-ui-soft/tests/consumer/button_busy.rs: `soft_delays_the_spinner_but_disables_input_immediately` and `soft_disabled_button_can_show_its_tooltip_with_keyboard_focus`.
  - The expanded trigger look has no direct test.

### SM-UI-02 RadioGroup: layout slot filled in into_view

- Location: crates/kivact-ui/src/radio_group.rs:15, 23-39, 70-108, 205-237.
- Current fields (verbatim):

```rust
    orientation: Orientation,
    /// What the group's recipe layer reads once the group is built: its
    /// orientation and whether a row runs from the right.
    layout: Rc<Cell<(Orientation, bool)>>,
```

- Why it is implicit: (f) S-7, and (h). `radio_group()` installs the recipe style and the RowReverse style; both read `layout` (lines 75-95). `into_view` fills `layout` afterwards (line 235). `layout.0` also duplicates `orientation`.
- Replacement code. Delete the `layout` field and its doc (lines 31-33). Replace the body of `radio_group` (lines 75-107) with:

```rust
    RadioGroup {
        root: node().role(Role::RadioGroup).name(label),
        options: options.into_value(),
        value: value.into_binding(),
        orientation: Orientation::Vertical,
        size: super::values::default_control_size().into_value(),
        on_edit: None,
        item: None,
        item_node: None,
        variants: Variants::default(),
    }
```

  In `into_view`, remove `layout,` from the destructuring at line 212. Replace lines 233-236 with:

```rust
        let reversed = orientation == Orientation::Horizontal
            && use_direction().get() == ReadingDirection::Rtl;
        // The recipe, then the row direction: base layers below the caller's styles.
        let root = with_base_recipe_style(
            root,
            |r| r.radio_group_recipes().group_recipe(),
            &variants,
            move || orientation,
        )
        .base_style(if reversed {
            kivact_core::style::style().direction(kivact_core::style::Direction::RowReverse)
        } else {
            kivact_core::style::style()
        });
        root.orientation(orientation).child(rows).into_view()
```

  Imports: line 2 `use super::layer::{recipes, with_recipe_style};` → `use super::layer::{recipes, with_base_recipe_style, with_recipe_style};`. Delete line 15 `use std::cell::Cell;`. Keep `std::rc::Rc`, which `ItemNode` still uses.
- Every read or write of the old field, and what it becomes:
  - radio_group.rs:33 (declaration): deleted.
  - radio_group.rs:75-76 (`layout`/`read` created): deleted.
  - radio_group.rs:84 (`read.get().0` in the recipe state): becomes the constant `orientation` in `into_view`.
  - radio_group.rs:86-95 (`read.get().1` in the RowReverse style): becomes the `reversed` constant in the base style above.
  - radio_group.rs:101 (`layout,` in the struct literal): deleted.
  - radio_group.rs:212 (destructure): deleted.
  - radio_group.rs:235 `layout.set((orientation, reversed))`: deleted.
- Layer order: base styles merge in insertion order (element_data.rs:580-583), so the order is recipe, then RowReverse, then the caller's styles. That matches the old order.
- xtask-allow lines to delete: none. radio_group.rs:23 stays; it still covers `on_edit`/`item`/`item_node` (RadioGroup is a NodeBuilder anyway).
- Public signatures: unchanged.
- Tests:
  - dev/tests/kivact-public-api/tests/e2e/ui_controls.rs: `a_radio_group_chooses_by_keys_pointer_and_invoke_each_choice_one_edit`.
  - dev/tests/kivact-public-api/tests/e2e/apg_radio_group.rs: `radio_group_tab_reentry_focuses_the_checked_radio`.
  - dev/tests/kivact-public-api/tests/e2e/theme_conformance.rs: the radio_group recipe rows.
  - crates/kivact-ui-soft/tests/consumer/navigation.rs: `segmented_hint_attaches_to_the_canonical_radio_target` (horizontal).
  - RTL row-reverse has no test.

### SM-UI-03 Menu Level: three surface kinds encoded in two bools

- Location: crates/kivact-ui/src/menu.rs:158-231, 248-271, 364-422, 532-548.
- Current fields (verbatim, menu.rs:263-267):

```rust
    // xtask-allow(state-fields): Immutable renderer choice for context positioning; session owns the origin.
    context: bool,
    // xtask-allow(state-fields): Immutable renderer choice for a submenu recipe.
    /// A submenu's level, drawn with the submenu surface recipe.
    sub: bool,
```

  plus the parameters `fn render<T>(self, trigger: Option<T>, context: bool, fallback: bool)` (menu.rs:171-176).
- Why it is implicit: (a), S-1. `(context, sub)` takes only (false,false) anchored, (true,false) context and (false,true) submenu; (true,true) never occurs. `(context, fallback)` takes only (false,false), (true,false) and (true,true); `fallback` is only valid when `context` is true.
- Replacement code. Add next to `struct Level`:

```rust
/// How the top level of a menu is opened.
#[derive(Clone, Copy, PartialEq)]
enum Opening {
    Trigger,
    Context,
    ContextFallback,
}
/// Which surface one level of the open menu is.
#[derive(Clone, Copy, PartialEq)]
enum LevelKind {
    Anchored,
    Context,
    Submenu,
}
```

  Replace menu.rs:263-267 with the single field:

```rust
    kind: LevelKind,
```

  Replace `render`'s signature and part selection (menu.rs:171-183) with:

```rust
    fn render<T: NodeBuilder + IntoView>(self, trigger: Option<T>, opening: Opening) -> View {
        let part = match opening {
            Opening::ContextFallback => self.behavior.context_fallback_trigger(),
            Opening::Context => self.behavior.context_trigger(),
            Opening::Trigger => self.behavior.trigger(),
        };
        let kind = if opening == Opening::Trigger {
            LevelKind::Anchored
        } else {
            LevelKind::Context
        };
```

  Replace the Level literal at menu.rs:215-226 with:

```rust
                Level {
                    behavior: self.behavior,
                    anchor,
                    placement: self.placement,
                    align: self.align,
                    at_least_anchor_width: self.at_least_anchor_width && kind == LevelKind::Anchored,
                    kind,
                    label,
                    offset: self
                        .offset
                        .unwrap_or(if kind == LevelKind::Context { 0.0 } else { 4.0 }),
                    variants: self.variants,
                },
```

- Every read or write of the old fields, and what it becomes:
  - menu.rs:159 `self.render(Some(trigger), false, false)` → `self.render(Some(trigger), Opening::Trigger)`.
  - menu.rs:165 `self.render(Some(target), true, false)` → `self.render(Some(target), Opening::Context)`.
  - menu.rs:168 `self.render(Some(target), true, true)` → `self.render(Some(target), Opening::ContextFallback)`.
  - menu.rs:250 `self.render(None::<Node>, false, false)` → `self.render(None::<Node>, Opening::Trigger)`.
  - menu.rs:177-183 (`if fallback … else if context …`) → the `match opening` above.
  - menu.rs:220 `&& !context` → `&& kind == LevelKind::Anchored`.
  - menu.rs:221-222 `context, sub: false,` → `kind,`.
  - menu.rs:224 `if context` → `if kind == LevelKind::Context`.
  - menu.rs:371-372 (`context, sub,` in the destructure in `surface`) → `kind,`.
  - menu.rs:383-389 → `match kind { LevelKind::Submenu => |r| r.menu_recipes().sub_surface_recipe(), LevelKind::Context => |r| r.menu_recipes().context_surface_recipe(), LevelKind::Anchored => |r| r.menu_recipes().surface_recipe() }`. Use the same `fn(&UiRecipes) -> &Recipe<_>` coercion as now; if inference fails, bind the match to `let pick: fn(&crate::recipes::UiRecipes) -> &crate::recipes::Recipe<_> = match kind { … };`.
  - menu.rs:394 `if context && matches!(…)` → `if kind == LevelKind::Context && matches!(…)`.
  - menu.rs:540-541 `context: false, sub: true,` → `kind: LevelKind::Submenu,`.
- xtask-allow lines to delete: menu.rs:263 and menu.rs:265 (with the `/// A submenu's level…` doc line 266). Keep menu.rs:259 and 261.
- Public signatures: unchanged. `render` is private, and `attach_context_fallback` is `pub(crate)` with the same signature.
- Tests:
  - dev/tests/kivact-public-api/tests/e2e/apg_menu.rs: `menu_submenu_parent_publishes_menu_popup`.
  - dev/tests/kivact-public-api/tests/e2e/interaction_menu.rs: `four_entrances_open_a_context_menu_at_the_pointer_or_at_the_area`.
  - dev/tests/kivact-public-api/tests/e2e/ui_components.rs: `context_menu_reuses_renderer_for_keyboard_and_pointer_origins`.
  - dev/tests/kivact-public-api/tests/e2e/ui_data_table.rs: `row_context_menu_opens_from_a_cell_without_changing_selection` (ContextFallback).

## 4. UNANNOTATED-FIX entries

### SM-UI-04 TextInput: size kept in an Rc<Cell> slot

- Location: crates/kivact-ui/src/text_input.rs:13-14, 32-38, 76-99, 155-158, 276-303.
- Current field (verbatim):

```rust
    size: Rc<Cell<kivact_core::state::Value<ControlSize>>>,
```

- Why it is implicit: (f) S-7. The field is a shared slot written by `.size()` (line 156) and read by the base style built in `into_view` (lines 283, 297). It is not shared with anything else, so the cell only adds a hidden "written before mount" phase.
- Replacement code:

```rust
    size: kivact_core::state::Value<ControlSize>,
```

  In `create` (lines 81-83):

```rust
    let size = super::values::default_control_size().into_value();
```

  Replace `size` (lines 155-158) with:

```rust
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
```

- Every read or write, and what it becomes:
  - text_input.rs:35 (declaration): becomes the plain Value above.
  - text_input.rs:81-83 (Rc::new(Cell::new(..))): becomes the plain value.
  - text_input.rs:156 `self.size.set(..)` → assignment.
  - text_input.rs:283 `let size = self.size;`: unchanged.
  - text_input.rs:297 `size.get().get()` → `size.get()`.
  - text_input.rs:13-14: delete `use std::cell::Cell;` and `use std::rc::Rc;` (nothing else in the file uses them).
- xtask-allow lines to delete: none (there are none).
- Public signatures: unchanged.
- Tests:
  - dev/tests/kivact-public-api/tests/e2e/feedback.rs: `input_group_scopes_control_sizes_and_preserves_adornments`.
  - crates/kivact-ui-basic/tests/consumer/foundation.rs: `text_input_edits_its_binding_and_rings_on_focus` (`.size(ControlSize::Small)`).

### SM-UI-05 Toast swipe: `held` signal mirrored from the swipe Machine by an effect

- Location: crates/kivact-ui/src/toast.rs:8, 219-224; crates/kivact-ui/src/toast_swipe/handle.rs:6, 16-49; crates/kivact-ui/src/toast_swipe/machine.rs:30-33, 56, 72-78.
- Current state (verbatim):

```rust
// toast.rs:220-222
            let held = signal(false);
            let part = toaster.item_while(id, held);
            let swipe = Swipe::new(*edge, card_ref.rect(), held, toaster, id);
// toast_swipe/machine.rs:30-33
pub(super) enum Effect {
    Hold(bool),
    Dismiss,
}
// toast_swipe/handle.rs:37
                machine::Effect::Hold(value) => held.set(value),
```

- Why it is implicit: (e) S-5 and (h) S-4. `held` is a second copy of "a touch contact is active", written through `Effect::Hold` from the transitions into `Pressed` (true) and back to `Rest` (false). `Protect` moves to `Blocked` without `Hold(false)`, so the mirror can stay true while no contact exists.
- Replacement code.

  toast_swipe/machine.rs, Effect:

```rust
pub(super) enum Effect {
    Dismiss,
}
```

  toast_swipe/machine.rs:56:

```rust
        let reset = || Step::to(Swipe::Rest);
```

  toast_swipe/machine.rs:72-78 (drop `.with(Effect::Hold(true))`):

```rust
                return Step::to(Swipe::Pressed(Contact {
                    pointer: event.id(),
                    origin: pos,
                    started: self.now,
                    height: rect.height.max(1.0),
                }));
```

  toast_swipe/handle.rs: replace `new` (lines 17-49) and add `held`:

```rust
    pub(crate) fn new(
        edge: Edge,
        rect: Value<Option<Rect>>,
        toaster: ToasterApi,
        id: ToastId,
    ) -> Self {
        let session = machine(
            machine::Swipe::Rest,
            move || machine::Machine {
                edge,
                rect: rect.get(),
                now: clock_now(),
                slop: use_theme()
                    .get()
                    .ext::<InteractionPolicy>()
                    .toast
                    .swipe_slop,
            },
            move |_, effect| match effect {
                // A task runs in the same turn once the effects have settled:
                // the card's style has committed its released look (the swipe
                // exit and the theme's transition) before the card is removed,
                // so the exit starts from the drag offset instead of jumping
                // through the drag's zero-length transition.
                machine::Effect::Dismiss => {
                    spawn(async move { toaster.dismiss(id) });
                }
            },
        );
        Self { edge, session }
    }
    /// Whether a touch on the card holds its timer: from the press, through
    /// the drag and a qualifying swipe's exit, until the gesture resets.
    pub(crate) fn held(self) -> Value<bool> {
        let session = self.session;
        memo(move || {
            matches!(
                session.get(),
                machine::Swipe::Pressed(_)
                    | machine::Swipe::Dragging { .. }
                    | machine::Swipe::Released { .. }
            )
        })
    }
```

  handle.rs line 6: `use kivact_core::state::{MachineHandle, Signal, Value, machine};` → `use kivact_core::state::{MachineHandle, Value, machine, memo};`

  toast.rs:219-224 becomes:

```rust
        ToastSwipe::Enabled { edge, transition } => {
            let swipe = Swipe::new(*edge, card_ref.rect(), toaster, id);
            let part = toaster.item_while(id, swipe.held());
            (part, Some((swipe, transition.clone())))
        }
```

  toast.rs:8: delete `use kivact_core::state::signal;` (line 220 is its only use).
- Every read or write, and what it becomes:
  - toast.rs:220 (signal created): deleted.
  - toast.rs:221 (`item_while(id, held)`): `item_while(id, swipe.held())`. `item_while` takes `impl IntoValue<bool>`, so this is unchanged API.
  - toast.rs:222 (`held` passed to `Swipe::new`): parameter removed.
  - handle.rs:20 (parameter): removed.
  - handle.rs:37 (`held.set(value)`): arm removed.
  - machine.rs:31 `Hold(bool)`: removed.
  - machine.rs:56 `.with(Effect::Hold(false))`: removed.
  - machine.rs:78 `.with(Effect::Hold(true))`: removed.
  - The new reader is the `held()` projection.
- Behaviour note: the only difference is that a `Protect` arriving during a press now releases the hold, which is correct. Every other path gives the same value as before.
- xtask-allow lines to delete: none.
- Public signatures: unchanged (all `pub(crate)` or `pub(super)`).
- Tests:
  - crates/kivact-ui-soft/tests/consumer/toast_swipe.rs: `a_pending_touch_holds_only_its_toasts_timer` and `a_short_or_mouse_drag_does_not_dismiss_but_an_edge_swipe_does`.
  - dev/tests/kivact-public-api/tests/e2e/presence.rs: `toast_swipe_dismiss_exits_from_the_drag_offset`.

### SM-UI-06 EmptyState: redundant "already announced" flag

- Location: crates/kivact-ui/src/empty_state.rs:4, 15-19, 23, 99-113.
- Current state (verbatim):

```rust
#[derive(Clone, Copy, PartialEq)]
enum Announcement {
    Pending,
    Spoken,
}
// into_view:
            let phase = signal(Announcement::Pending);
            on_mount(move || {
                if phase.get() == Announcement::Pending {
                    // … announce …
                    phase.set(Announcement::Spoken);
                }
            });
```

- Why it is implicit: (c). The flag records that the announcement already happened and gates a second one. `on_mount` takes an `FnOnce` and runs once (kivact-runtime/src/reactive/setup.rs:172-175), so the guard is dead state. It is not a named S-9 type.
- Replacement code. Delete `enum Announcement` (lines 15-19). Replace lines 99-113 with:

```rust
        let notices = (self.announce == AnnouncePolicy::Once).then(|| {
            let notices = announcer();
            on_mount(move || {
                let message = description.map_or_else(
                    || title.get(),
                    |description| format!("{}. {}", title.get(), description.get()),
                );
                notices.announce(message, LiveRegion::Polite);
            });
            notices.outlet()
        });
```

  Line 4: `use kivact_core::state::{IntoText, IntoValue, Value, on_mount, signal};` → `use kivact_core::state::{IntoText, IntoValue, Value, on_mount};`
- Every read or write, and what it becomes:
  - empty_state.rs:101 (signal created): deleted.
  - empty_state.rs:103 (guard read): deleted.
  - empty_state.rs:109 (`phase.set(Spoken)`): deleted.
- xtask-allow lines: do not delete empty_state.rs:23. Change its text to `// xtask-allow(state-fields): Builder configuration and optional slots; the one-shot announcement runs in on_mount.`
- Public signatures: unchanged.
- Tests: dev/tests/kivact-public-api/tests/e2e/feedback.rs: `empty_state_announces_once`.

### SM-UI-07 DataTable row drag: `consumed` offset written by handlers

- Location: crates/kivact-ui/src/data_table/rows.rs:17, 874-912.
- Current state (verbatim):

```rust
    let consumed = signal(0.0_f64);
    let (source, key, sort) = (grid.source, grid.key.clone(), grid.sort);
    let drag = use_drag::<()>(DragCoordinates::Window)
        .on_start(move |_| consumed.set(0.0))
        .on_update(move |update: DragUpdate<()>| {
            let dy = update.delta.y - consumed.get_untracked();
            if dy.abs() <= 28.0 {
                return;
            }
            // … find destination …
            if let Some(destination) = destination {
                reorder.call(kivact_interaction::Reorder { key: id.clone(), to: destination });
                sort.set(None);
                consumed.set(update.delta.y);
            }
        });
```

- Why it is implicit: (c) and S-5. `consumed` records the drag offset already turned into a move and gates the next move. Handlers write it outside any transition, and `on_start` repairs it to 0. It is only meaningful during a drag, which is a phase the variable does not represent.
- Replacement code. Insert above `fn drag_source`:

```rust
/// Where a dragged row is: idle, or dragged with the offset at which it last moved one place.
#[derive(Clone, Copy, PartialEq)]
enum RowDrag {
    Idle,
    Dragging { applied: f64 },
}
enum RowDragEvent {
    Start,
    Update(f64),
    End,
}
/// The dragged row's place among the caller's rows, read for each event.
struct RowPlace {
    at: Option<usize>,
    len: usize,
}
impl Machine for RowPlace {
    type State = RowDrag;
    type Event = RowDragEvent;
    type Effect = usize;
    fn transition(
        &self,
        state: &RowDrag,
        event: RowDragEvent,
    ) -> Step<RowDrag, RowDragEvent, usize> {
        match (*state, event) {
            (_, RowDragEvent::Start) => Step::to(RowDrag::Dragging { applied: 0.0 }),
            (_, RowDragEvent::End) => Step::to(RowDrag::Idle),
            (RowDrag::Dragging { applied }, RowDragEvent::Update(offset)) => {
                let dy = offset - applied;
                match self.at {
                    Some(at) if dy.abs() > 28.0 => {
                        let to = if dy > 0.0 {
                            (at + 1).min(self.len.saturating_sub(1))
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
```

  Replace the body of `drag_source` (lines 885-911) with:

```rust
    let (source, key, sort) = (grid.source, grid.key.clone(), grid.sort);
    let place = id.clone();
    let row_drag = machine(
        RowDrag::Idle,
        move || {
            let (at, len) = source.with(|rows| {
                (rows.iter().position(|row| key(row) == place), rows.len())
            });
            RowPlace { at, len }
        },
        move |_, to: usize| {
            reorder.call(kivact_interaction::Reorder { key: id.clone(), to });
            sort.set(None);
        },
    );
    let drag = use_drag::<()>(DragCoordinates::Window)
        .on_start(move |_| row_drag.send(RowDragEvent::Start))
        .on_update(move |update: DragUpdate<()>| {
            row_drag.send(RowDragEvent::Update(update.delta.y))
        })
        .on_end(move |_| row_drag.send(RowDragEvent::End));
    drag.source(())
```

  Line 17: `use kivact_core::state::{Binding, Callback, Value, binding, memo, signal};` → `use kivact_core::state::{Binding, Callback, Machine, Step, Value, binding, machine, memo};` (line 885 was the only use of `signal`).
- Every read or write, and what it becomes:
  - rows.rs:885 (signal created): becomes the `row_drag` machine (initial `Idle`).
  - rows.rs:888 (`consumed.set(0.0)` in on_start): `send(Start)` → `Dragging { applied: 0.0 }`.
  - rows.rs:890 (`consumed.get_untracked()`): read as `applied` in `transition`.
  - rows.rs:894-901 (destination computed from `source`): computed from the `RowPlace` input plus the transition.
  - rows.rs:903-907 (reorder, `sort.set(None)`): the Effect `usize` runner.
  - rows.rs:908 (`consumed.set(update.delta.y)`): `Step::to(Dragging { applied: offset })`.
  - New: `on_end` → `End` → `Idle`. `on_end` also fires on cancel (drawer_swipe.rs:26-43 relies on the same behaviour).
- xtask-allow lines to delete: none.
- Public signatures: unchanged (private function).
- Tests: no test found for `DataTable::on_reorder` row dragging. ui_data_table.rs mentions reordering only in its module comment.

### SM-UI-08 Variants: shared builder cell read by styles installed before the last `.variant(..)`

- Prerequisite: apply SM-UI-01 (Button) and SM-UI-02 (RadioGroup) first. Their old constructors are two of the six constructor-time readers.
- Location:
  - crates/kivact-ui/src/variants.rs:8-12 and 110-171.
  - The four remaining constructor-time readers: calendar/view.rs:73-81 (+354), data_table/builder.rs:4-6 and 95-102 (+144) (+data_table/runtime.rs:87), number_input.rs:2 and 128-142 (+301), tag_input.rs:2 and 60-84 (+204).
  - 69 builder signatures (table below).
- Kit crates: no change. kivact-ui-basic/pop/game/soft/creative never name `Variants`; the type is `pub(super)` in kivact-ui. Their own `variant(mut self, ..)` methods (basic marker.rs:53, sidebar/panel.rs:57; soft app_shell.rs:220, citation.rs:79, divider.rs:62, field_status.rs:65, more_menu.rs:62, section.rs:94) set kit-specific enums and already take `mut self`.
- Current code (verbatim, variants.rs:110-114):

```rust
/// The variant values a component builder collects with
/// `.variant(..)`, shared with the recipe styles it has made: they are
/// read when the styles first run, once the builder is mounted.
#[derive(Clone, Default)]
pub(super) struct Variants(Rc<RefCell<Extensions>>);
```

- Why it is implicit: (f) S-7 and (g) S-6. The Rc<RefCell> is shared between a builder and the style closures its constructor already installed; later `.variant(..)` calls fill it. The phase "every `.variant(..)` happens before the first style evaluation" is expressed only by call order. node_modifiers.rs:277-280 names this exact pattern ("instead of installing a style at creation that reads them through a shared cell") as the one to avoid.
- Replacement code.

  variants.rs:8-9 (module doc), replace the last sentence with:

```rust
//! not another component or a subtree theme. A builder owns its values and
//! installs every recipe style in `into_view`, after its last `.variant(..)`.
```

  variants.rs:11: delete `use std::cell::RefCell;` (keep `std::rc::Rc`, which `Extensions` uses).

  variants.rs:110-171: replace with:

```rust
/// The variant values a component builder collects with `.variant(..)`.
/// The builder owns them and installs its recipe styles in `into_view`, so
/// each style captures the final values.
#[derive(Clone, Default)]
pub(super) struct Variants(Extensions);

impl Variants {
    /// Merge inherited group decoration while preserving panel-specific types.
    pub(super) fn inherited(&self, parent: &Self) -> Self {
        let mut entries = parent
            .0
            .0
            .iter()
            .filter(|entry| {
                !self
                    .0
                    .0
                    .iter()
                    .any(|value| (*value.value).type_id() == (*entry.value).type_id())
            })
            .cloned()
            .collect::<Vec<_>>();
        entries.extend(self.0.0.iter().cloned());
        Self(Extensions(Rc::new(entries)))
    }

    pub(super) fn add<T: Variant>(&mut self, value: T) {
        self.0 = std::mem::take(&mut self.0).with(value);
    }

    /// Inherit missing presentation types while retaining explicit caller values.
    pub(super) fn with_defaults(&self, defaults: &Self) -> Self {
        let mut merged = Self::default();
        merged.merge(defaults);
        merged.merge(self);
        merged
    }

    /// Merge presentation values, replacing only values of the same type.
    pub(super) fn merge(&mut self, other: &Self) {
        let entries = Rc::make_mut(&mut self.0.0);
        for entry in other.0.0.iter() {
            entries.retain(|existing| (*existing.value).type_id() != (*entry.value).type_id());
            entries.push(entry.clone());
        }
    }

    /// An independent recipe look for one presentation of this component.
    pub(super) fn with<T: Variant>(&self, value: T) -> Self {
        Self(self.0.clone().with(value))
    }

    /// `state` with the collected values.
    pub(super) fn look<S>(&self, state: S) -> Look<S> {
        Look {
            state,
            ext: self.0.clone(),
        }
    }
}
```

  Constructor-time readers: the recipe style moves to `into_view`.
  - calendar/view.rs:74-81: in `fn new`, delete `let variants = crate::variants::Variants::default();`. Replace the `root:` initializer and the `variants,` field with:

```rust
        root: node().with_node(|root| root.role(Role::Group)),
        variants: crate::variants::Variants::default(),
```

    calendar/view.rs:354: replace `self.root` with:

```rust
        crate::layer::with_base_recipe_style(
            self.root,
            |r| r.calendar_recipes().root_recipe(),
            &self.variants,
            kivact_core::custom::PartState::default,
        )
```

    The chain `.on_pointer(..).child((body, status)).into_view()` stays. This is a partial move of `self.root` beside a borrow of `self.variants`, which is allowed: Calendar has no Drop.
  - data_table/builder.rs:95-102: delete both comments and `let variants = Variants::default();`. Replace the `with_recipe_style(..)` call with:

```rust
    let root = column(()).role(Role::Group);
```

    builder.rs:144: `variants,` → `variants: Variants::default(),`.

    builder.rs:4 `use crate::layer::with_recipe_style;` and :6 `use kivact_core::custom::PartState;`: delete (no other use in the file).

    data_table/runtime.rs:87: replace `table.root,` with:

```rust
        crate::layer::with_base_recipe_style(
            table.root,
            |r| r.data_table_recipes().frame_recipe(),
            &table.variants,
            kivact_core::custom::PartState::default,
        ),
```

  - number_input.rs:128-142: delete `let variants = Variants::default();`. Replace the root initializer with:

```rust
    let root = grid(()).base_style(style().grid_template_columns((auto(), Track::fr(1), auto())));
```

    In the struct literal, `variants,` → `variants: Variants::default(),`.

    number_input.rs:301: `root.child((field, decrease, increase)).into_view()` →

```rust
        with_base_recipe_style(
            root,
            |r| r.number_input_recipes().root_recipe(),
            &variants,
            PartState::default,
        )
        .child((field, decrease, increase))
        .into_view()
```

    number_input.rs:2: `use super::layer::with_recipe_style;` → `use super::layer::{with_base_recipe_style, with_recipe_style};`.
  - tag_input.rs:60-73: delete `let variants = Variants::default();`, the `within` comment and `let within = input_ref.focus_within();`, and the `with_recipe_style(..)` wrapper. Keep:

```rust
    let input_ref = node_ref();
    let root = grid(())
        .base_style(style().grid_template_columns((Track::minmax(0, Track::fr(1)), auto())));
```

    tag_input.rs:83: `variants,` → `variants: Variants::default(),`.

    tag_input.rs:204: `root.child((field, chips))` →

```rust
        // The box rings while the focus is in its input.
        let within = input_ref.focus_within();
        with_base_recipe_style(
            root,
            |r| r.tag_input_recipes().root_recipe(),
            &variants,
            move || PartState {
                focused: within.get(),
                ..PartState::default()
            },
        )
        .child((field, chips))
```

    (the `.on_pointer_default(..)` chain that follows stays). tag_input.rs:2: `use super::layer::with_recipe_style;` → `use super::layer::{with_base_recipe_style, with_recipe_style};`.

  Layer order:
  - data_table, number_input and tag_input: the recipe was a style layer (above all base styles). It is now the last base style added, which is still above the builder's own base layout style and still below every caller `.style(..)`.
  - calendar: the recipe was the first base style. It is now added after any `base_style` the caller added through `with_node` before mount. No in-repo caller does that.

- Every read or write that changes (69 signatures). In each, change `(self,` to `(mut self,`. The body line stays exactly as it is. The public signature is unchanged, because `mut` on a by-value receiver is not part of the signature.

| file:line | body line (unchanged) |
|---|---|
| crates/kivact-ui/src/alert.rs:164 | self.variants.add(value); |
| crates/kivact-ui/src/avatar.rs:84 | self.variants.add(variant); |
| crates/kivact-ui/src/avatar_group.rs:57 | self.variants.add(variant); |
| crates/kivact-ui/src/badge.rs:34 | self.variants.add(variant); |
| crates/kivact-ui/src/breadcrumb.rs:69 | self.variants.add(variant); |
| crates/kivact-ui/src/button.rs:253 (line as before SM-UI-01; locate by `pub fn variant(self`) | self.variants.add(v); |
| crates/kivact-ui/src/button_group.rs:55 | self.variants.add(variant); |
| crates/kivact-ui/src/calendar/view.rs:143 | self.variants.add(variant); |
| crates/kivact-ui/src/card.rs:77 | self.variants.add(value); |
| crates/kivact-ui/src/carousel.rs:51 | self.variants.add(variant); |
| crates/kivact-ui/src/chat_composer.rs:91 | self.variants.add(variant); |
| crates/kivact-ui/src/chat_message.rs:100 | self.variants.add(variant); |
| crates/kivact-ui/src/chat_thread.rs:92 | self.variants.add(variant); |
| crates/kivact-ui/src/code_block.rs:167 | self.variants.add(variant); |
| crates/kivact-ui/src/combobox/handle.rs:133 | self.variants.add(variant); |
| crates/kivact-ui/src/command_palette.rs:101 | self.variants.add(v); |
| crates/kivact-ui/src/data_table/builder.rs:211 | self.variants.add(v); |
| crates/kivact-ui/src/date_input/view.rs:88 | self.variants.add(variant); |
| crates/kivact-ui/src/date_input/view.rs:92 (`pub(crate) fn with_variants(self,`) | self.variants.merge(variants); |
| crates/kivact-ui/src/date_range_input.rs:101 | self.variants.add(variant); |
| crates/kivact-ui/src/date_time_input.rs:50 | self.variants.add(variant); |
| crates/kivact-ui/src/description_list.rs:24 | self.variants.add(value); |
| crates/kivact-ui/src/description_list.rs:71 | self.variants.add(value); |
| crates/kivact-ui/src/dialog.rs:114 (`pub(crate) fn with_variants(self,`) | self.variants.merge(variants); |
| crates/kivact-ui/src/dialog.rs:118 | self.variants.add(v); |
| crates/kivact-ui/src/disclosure.rs:193 | self.variants.add(v); |
| crates/kivact-ui/src/disclosure_group.rs:33 | self.variants.add(variant); |
| crates/kivact-ui/src/dropzone.rs:57 | self.variants.add(variant); |
| crates/kivact-ui/src/empty_state.rs:81 | self.variants.add(value); |
| crates/kivact-ui/src/field.rs:105 | self.variants.add(v); |
| crates/kivact-ui/src/heading.rs:30 | self.variants.add(value); |
| crates/kivact-ui/src/hover_card.rs:57 | self.variants.add(variant); |
| crates/kivact-ui/src/input_group.rs:86 | self.variants.add(value); |
| crates/kivact-ui/src/item_list.rs:45 | self.variants.add(variant); |
| crates/kivact-ui/src/item_row.rs:117 | self.contents.variants.add(value); |
| crates/kivact-ui/src/kbd.rs:21 | self.variants.add(variant); |
| crates/kivact-ui/src/kbd.rs:69 | self.variants.add(variant); |
| crates/kivact-ui/src/list.rs:79 | self.variants.add(v); |
| crates/kivact-ui/src/menu.rs:92 | self.variants.add(v); |
| crates/kivact-ui/src/navigation_link.rs:41 | self.variants.add(variant); |
| crates/kivact-ui/src/navigation_menu.rs:99 | self.variants.add(variant); |
| crates/kivact-ui/src/number_input.rs:154 | self.variants.add(v); |
| crates/kivact-ui/src/pagination.rs:131 | self.variants.add(variant); |
| crates/kivact-ui/src/progress.rs:124 | self.variants.add(v); |
| crates/kivact-ui/src/radio_group.rs:121 (line as before SM-UI-02; locate by `pub fn variant(self`) | self.variants.add(v); |
| crates/kivact-ui/src/select/handle.rs:104 | self.settings.variants.add(v); |
| crates/kivact-ui/src/selection_field.rs:59 | self.variants.add(variant); |
| crates/kivact-ui/src/skeleton.rs:37 | self.variants.add(variant); |
| crates/kivact-ui/src/slider.rs:111 | self.variants.add(v); |
| crates/kivact-ui/src/spinner.rs:29 | self.variants.add(variant); |
| crates/kivact-ui/src/split.rs:140 | self.variants.add(v); |
| crates/kivact-ui/src/splitter.rs:95 | self.variants.add(v); |
| crates/kivact-ui/src/stat.rs:70 | self.variants.add(value); |
| crates/kivact-ui/src/stepper.rs:159 | self.variants.add(variant); |
| crates/kivact-ui/src/table.rs:57 | self.variants.add(variant); |
| crates/kivact-ui/src/table.rs:162 | self.variants.add(variant); |
| crates/kivact-ui/src/tabs.rs:89 | self.variants.add(v); |
| crates/kivact-ui/src/tag_input.rs:115 | self.variants.add(v); |
| crates/kivact-ui/src/text_input.rs:129 | self.variants.add(v); |
| crates/kivact-ui/src/time_input/view.rs:43 | self.variants.add(variant); |
| crates/kivact-ui/src/time_input/view.rs:47 (`pub(crate) fn with_variants(self,`) | self.variants.merge(variants); |
| crates/kivact-ui/src/toast.rs:75 | self.variants.add(v); |
| crates/kivact-ui/src/toggle.rs:80 (inside the builder macro) | self.variants.add(v); |
| crates/kivact-ui/src/toggle_button.rs:178 | self.variants.add(v); |
| crates/kivact-ui/src/toggle_group.rs:96 | self.variants.add(variant); |
| crates/kivact-ui/src/toolbar.rs:85 | self.variants.add(variant); |
| crates/kivact-ui/src/tooltip.rs:52 | self.variants.add(v); |
| crates/kivact-ui/src/tree.rs:107 | self.variants.add(variant); |
| crates/kivact-ui/src/typing_indicator.rs:33 | self.variants.add(variant); |

  Already correct, no edit: table.rs:91 (`TableRow::variant(mut self, ..)` forwards), disclosure.rs:106 (`group_variants(mut self, ..)` assigns), navigation_link.rs:55 (assigns in `into_view`). Not a builder method: variants.rs:226 (`Look::variant`). The only other mutation sites are variants.rs:143-144, covered by `let mut merged` in the replacement.

  Reviewed and unchanged:
  - Every other `variants` use in kivact-ui (about 750 lines) is a read inside `into_view`, `attach`/`finish`, or a helper called from them. Those reads keep working with value clones: `with_recipe_style(.., &variants, ..)`, `variants.clone()`, `with_variants(&variants)` and context values (item_context.rs:10, navigation_context.rs:8).
  - The only constructors that installed styles before `.variant(..)` were button, radio_group, calendar, data_table, number_input and tag_input. The list was checked by scanning every function that creates `Variants::default()`.
- xtask-allow lines to delete: none.
- Public signatures: unchanged.
- Tests:
  - dev/tests/kivact-public-api/tests/e2e/ui_standard.rs: `a_recipe_in_the_theme_replaces_the_look_of_its_component_only` (asserts "the variant reaches the recipe" on Button).
  - dev/tests/kivact-public-api/tests/e2e/metrics_and_description.rs: `description_list_pairs_terms_and_details`, `heading_has_its_level`, `progress_ring_reports_its_value`, `stat_exposes_label_and_value`.
  - dev/tests/kivact-public-api/tests/e2e/feedback.rs: `alert_dismiss_restores_focus`.
  - dev/tests/kivact-public-api/tests/e2e/theme_conformance.rs: whole file.
  - crates/kivact-ui-soft/tests/consumer/navigation.rs: `segmented_hint_attaches_to_the_canonical_radio_target` (RadioGroup variant).
  - crates/kivact-ui-basic/tests/consumer/forms.rs: `date_picker_opens_a_popover_picks_a_day_and_closes`.
  - crates/kivact-ui-basic/tests/consumer/data.rs: `payments_table`.
  - crates/kivact-ui-game/tests/consumer/controls.rs: `consumer_buttons_checks_mixed_states_and_modifiers`.
  - No test passes a variant to number_input, tag_input, data_table or calendar after construction. New tests: T9.1 adds the sketch below to the existing file `dev/tests/kivact-public-api/tests/e2e/ui_standard.rs` (append after its last test; it uses the file's own `theme_with`, `options`, `pixel_mid`, `RED`, `Theme`, `UiRecipes` and imports; do not run it):

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
struct Marked;
impl kivact::ui::Variant for Marked {}

#[test]
fn a_variant_given_after_construction_reaches_the_root_recipe() {
    let recipes = UiRecipes::default().number_input(|table| {
        table.root(|prev| {
            move |theme: &Theme, look: &Look<kivact::custom::PartState>| match look.variant::<Marked>() {
                Some(Marked) => prev(theme, look).background(rgb(0xff0000)),
                None => prev(theme, look),
            }
        })
    });
    let screen = kivact::test::render_with(options(theme_with(recipes)), || {
        kivact::ui::number_input(
            "Count",
            signal(1.0),
            kivact::interaction::ScalarRange::new(0.0, 9.0).expect("a valid range"),
        )
            .variant(Marked)
            .test_id("count")
    });
    assert_eq!(pixel_mid(&screen, "count", 2.0), RED, "the variant reaches the root recipe");
}
```

  Repeat the sketch for `tag_input(..).variant(Marked)` with `.tag_input(|t| t.root(..))`, `data_table(..).variant(Marked)` with `.data_table(|t| t.frame(..))`, and `calendar(..).variant(Marked)` with `.calendar(|t| t.root(..))`. Before SM-UI-08 these pass because of the shared cell; after it they guard the move into `into_view`.

### SM-UI-09 SelectionField: `surface` signal kept beside the caller's `open` binding

- Location: crates/kivact-ui/src/selection_field.rs:4-6, 13-17, 69-135, 178-206.
- Current state (verbatim):

```rust
        let open = self.open.unwrap_or_else(|| signal(false).into_binding());
        let surface = signal(Surface::Popup);
// popup_binding (selection_field.rs:179-206)
    binding(
        move || open.get() && surface.get() == Surface::Popup,
        move |shown| {
            if shown {
                let sheet = presentation.get().uses_sheet()
                    && matches!(
                        gesture.get(),
                        crate::select::machine::SheetGesture::Released(
                            crate::select::machine::Outcome::Tap
                        )
                    );
                surface.set(if sheet { Surface::Sheet } else { Surface::Popup });
                gesture.send(crate::select::machine::Event::Reset);
            }
            open.set(shown);
        },
    )
```

- Why it is implicit: (b), (h) and S-3. `surface` is valid only while `open` is true, but it is stored apart from it. It is written only when the field's own trigger opens through `popup_binding`. When the caller opens through its binding (`SelectionField::open`), the previous opening's surface is reused. (true, Sheet) after a touch opening, then a caller reopen, shows the drawer again.
- Decision applied: the caller's `open` binding is the one fact. The surface snapshot is computed when `open` turns true and refreshed on every opening, including one through the caller's binding.
  - The variant "`feed`/`effect_on` writes the snapshot" was evaluated and rejected because it violates S-4 and S-5 in practice. `effect_on` and `Machine::feed` are user effects that run after the turn's ops are flushed (kivact-runtime/src/reactive/setup.rs:142-143). `popup_open`/`sheet_open` (read by the dialogs' `show` during that flush) would therefore render once with the previous opening's surface: a wrong dialog mounts and then unmounts, with its focus moves.
  - The snapshot is instead a memo that tracks only `open`. It is evaluated when read, so it is always consistent with `open`, and it writes nothing (no effect write and no second owner, which satisfies S-5).
  - The one remaining write is a real transition. The closing edge of `open` is fed to the trigger's gesture machine as `Event::Reset` (Machine rule 5: send the source's change). `feed` cannot filter edges, so this is a single `effect_on(open)` whose only action is `gesture.send(Reset)` on false. The tap therefore never outlives the opening it caused, and a caller reopen snapshots a fresh gesture.
  - The reset moves from the opening edge (old `popup_binding`) to the closing edge. The lazily evaluated snapshot must still see the `Released(Tap)` of the opening tap.
- Replacement code.

  selection_field.rs:4-6 imports:

```rust
use kivact_core::state::{
    Binding, IntoBinding, IntoText, IntoValue, Value, binding, effect_on, machine, memo, signal,
    untrack,
};
```

  selection_field.rs:71-72: delete line 72 (`let surface = signal(Surface::Popup);`). After line 80 (`let presentation = self.presentation;`) insert:

```rust
        let surface = opening_surface(open, presentation, gesture);
```

  selection_field.rs:107: replace `let popup_open = popup_binding(open, surface, presentation, gesture);` with:

```rust
        let popup_open = binding(
            move || open.get() && surface.get() == Surface::Popup,
            move |shown| open.set(shown),
        );
```

  selection_field.rs:127-130 (`sheet_open`): unchanged text. It now reads the memo `surface` (a `Value<Surface>`) instead of the signal.

  selection_field.rs:178-206: delete `popup_binding` and add:

```rust
/// The surface of the current opening, a snapshot taken when the caller's
/// `open` changes. A touch tap on the trigger opens the sheet when the
/// presentation uses one. Every other opening (keyboard, mouse, the caller's
/// binding) opens the popup. Only `open` is tracked, so the snapshot holds for
/// the whole opening. Closing feeds `Reset` to the trigger gesture, so the next
/// opening never reuses the previous tap.
fn opening_surface(
    open: Binding<bool>,
    presentation: Value<crate::SelectPresentation>,
    gesture: kivact_core::state::MachineHandle<crate::select::machine::Machine>,
) -> Value<Surface> {
    effect_on(open, move |open| {
        if !*open {
            gesture.send(crate::select::machine::Event::Reset);
        }
    });
    memo(move || {
        if !open.get() {
            return Surface::Popup;
        }
        let (gesture, presentation) = untrack(|| (gesture.get(), presentation.get()));
        let tap = matches!(
            gesture,
            crate::select::machine::SheetGesture::Released(crate::select::machine::Outcome::Tap)
        );
        if tap && presentation.uses_sheet() {
            Surface::Sheet
        } else {
            Surface::Popup
        }
    })
}
```

- Every read or write, and what it becomes:
  - selection_field.rs:72 `signal(Surface::Popup)`: deleted; the `opening_surface` memo replaces it.
  - selection_field.rs:107 `popup_binding(open, surface, presentation, gesture)`: the plain binding above.
  - selection_field.rs:128 `surface.get() == Surface::Sheet`: reads the memo; text unchanged.
  - selection_field.rs:186 `surface.get() == Surface::Popup`: now in the inline `popup_open` binding.
  - selection_field.rs:189-195 (sheet decision in the setter): moved into the memo.
  - selection_field.rs:196-200 `surface.set(..)`: deleted.
  - selection_field.rs:201 `gesture.send(Event::Reset)` at opening: moved to the closing edge in `opening_surface`.
  - selection_field.rs:203 `open.set(shown)`: unchanged in the new binding.
  - selection_field.rs:104 (`on_key` → `Event::Reset`): unchanged. A keyboard opening therefore still snapshots a non-tap gesture, so the popup opens.
  - crate::select::machine (shared with Select and Combobox): no change.
- Behaviour:
  - Touch tap, keyboard and mouse openings are unchanged.
  - New: an opening through the caller's binding always shows the popup, unless the caller opens in the same turn as a touch tap on the trigger.
  - `Released(Tap)` now lasts until the opening ends instead of being cleared at the opening. Nothing reads the gesture during an opening (its `readiness` is `Unavailable` while `open`).
- xtask-allow lines to delete: none (there are none).
- Public signatures: unchanged.
- Tests:
  - Existing: crates/kivact-ui-soft/tests/consumer/selector_touch.rs: `complex_selector_uses_the_same_search_binding_in_its_two_surfaces` (touch → sheet, Escape, Enter → popup).
  - New E2E test: T9.1 (its 担当ファイル include this file; step 3) appends the sketch below to the existing file `crates/kivact-ui-soft/tests/consumer/selector_touch.rs` (it is already registered in `tests/consumer.rs`; the sketch uses the file's imports and `Role`, `PointerKind`, `Duration`, `signal`, `RenderOptions`, `render_with_handle`; do not run it). It fails before the fix: the drawer reappears.

```rust
#[test]
fn selection_field_reopened_through_the_caller_binding_takes_a_fresh_surface() {
    let (screen, open) = render_with_handle(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light))
            .window_size(500.0, 700.0),
        || {
            let open = signal(false);
            (
                open,
                kivact::ui::selection_field("Workspace", "Design")
                    .presentation(kivact::ui::SelectPresentation::BottomSheet)
                    .open(open)
                    .content(|| kivact::view::text("Choices")),
            )
        },
    );
    // The drawer reaches the bottom of the 700 px window; the popup sits under the trigger.
    let drawer_shown = || {
        let rect = screen
            .get_by_role(Role::Dialog, "Workspace")
            .rect()
            .expect("dialog laid out");
        rect.y + rect.height >= 699.0
    };
    let trigger = screen.get_by_role(Role::Button, "Workspace");
    let rect = trigger.rect().unwrap();
    let touch = screen.contact_down(
        PointerKind::Touch,
        rect.x + rect.width / 2.0,
        rect.y + rect.height / 2.0,
    );
    screen.contact_up(touch);
    screen.advance(Duration::from_millis(350));
    assert!(open.get());
    assert!(drawer_shown(), "a touch tap opens the drawer");

    open.set(false);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(350));
    assert!(screen.query_by_role(Role::Dialog, "Workspace").is_none());

    // The caller reopens: a new opening without a tap shows the popup.
    open.set(true);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(350));
    assert!(!drawer_shown(), "the caller's reopening must not reuse the drawer");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```

### SM-UI-10 Dead allow on `enum TypedEffect`

- Location: crates/kivact-ui/src/date_input/machine.rs:135.
- Current line (verbatim):

```rust
// xtask-allow(state-fields): Event snapshot contains independent inclusive bounds and locale.
```

- Why: the line sits above `pub(super) enum TypedEffect`. The state-fields rule inspects structs only (dev/xtask/src/state_fields.rs:34-37), so the allow suppresses nothing. Its text also describes a snapshot the enum is not. Line 140 already annotates `TypedSnapshot`, the struct it meant.
- Replacement: delete line 135. No other change.
- Read/write sites: none.
- xtask-allow lines to delete: date_input/machine.rs:135.
- Tests: none needed (comment only).

---

# Implicit state machines in crates/kivact-ui-soft (main 96e22b6b)

Rules applied: docs/for-kivact-developer/architecture.md S-1..S-9 and the Machine rules
(`Machine::transition` pure, `Step::{stay,to,with,after,cancel}`, `machine(initial, input, run)`,
`MachineHandle::{send,get,state,feed}`; `feed` skips the initial value).
All line numbers are against the current tree. Apply fixes in the listed order inside one file
(top to bottom line numbers shift; every edit below quotes the exact current text, so match on text).

## 1. Summary counts

| Item | Count |
|---|---|
| `// xtask-allow(state-fields)` sites reviewed | 204 |
| KEEP sites | 192 |
| FIX sites (annotated lines removed by a fix) | 12, in 4 entries (SM-SOFT-01..04) |
| UNANNOTATED-FIX entries | 6 (SM-SOFT-05..10) |

Annotated FIX lines: tab_list.rs 350, 353, 355; side_navigation/items.rs 417, 420, 422, 424, 428, 430;
side_navigation/shell.rs 59; app_shell.rs 337, 339.

## 2. KEEP table

| file:line | struct.field | category |
|---|---|---|
| crates/kivact-ui-soft/src/alert_dialog.rs:29 | AlertDialog.open | caller binding |
| crates/kivact-ui-soft/src/alert_dialog.rs:31 | AlertDialog.action | builder option |
| crates/kivact-ui-soft/src/alert_dialog.rs:33 | AlertDialog.cancel | builder option |
| crates/kivact-ui-soft/src/alert_dialog.rs:36 | AlertDialog.loading | caller binding |
| crates/kivact-ui-soft/src/app_shell.rs:84 | MobileNavConfig.has_toggle | builder option |
| crates/kivact-ui-soft/src/app_shell.rs:86 | MobileNavConfig.open | caller binding |
| crates/kivact-ui-soft/src/app_shell.rs:88 | MobileNavConfig.on_change | builder option |
| crates/kivact-ui-soft/src/app_shell.rs:90 | MobileNavConfig.content | builder option |
| crates/kivact-ui-soft/src/app_shell.rs:92 | MobileNavConfig.breakpoint | builder option |
| crates/kivact-ui-soft/src/app_shell.rs:166 | AppShell.top | builder option |
| crates/kivact-ui-soft/src/app_shell.rs:168 | AppShell.side | builder option |
| crates/kivact-ui-soft/src/app_shell.rs:170 | AppShell.banner | builder option |
| crates/kivact-ui-soft/src/app_shell.rs:331 | ShellPresentation (struct-level) | passive record |
| crates/kivact-ui-soft/src/app_shell.rs:341 | ShellPresentation.has_top | passive record |
| crates/kivact-ui-soft/src/app_shell.rs:343 | ShellPresentation.has_side | passive record |
| crates/kivact-ui-soft/src/app_shell.rs:345 | ShellPresentation.has_banner | passive record |
| crates/kivact-ui-soft/src/avatar_status_dot.rs:69 | AvatarStatusDot (struct-level) | builder option |
| crates/kivact-ui-soft/src/bottom_sheet.rs:123 | BottomSheet.scrim | caller binding |
| crates/kivact-ui-soft/src/bottom_sheet.rs:126 | BottomSheet.final_focus | builder option |
| crates/kivact-ui-soft/src/bottom_sheet.rs:128 | BottomSheet.content | builder option |
| crates/kivact-ui-soft/src/chat_dictation.rs:19 | DictationState.supported | passive record |
| crates/kivact-ui-soft/src/chat_dictation.rs:22 | DictationState.listening | passive record |
| crates/kivact-ui-soft/src/citation.rs:31 | CitationSource.title | passive record |
| crates/kivact-ui-soft/src/citation.rs:34 | CitationSource.url | passive record |
| crates/kivact-ui-soft/src/citation.rs:37 | CitationSource.image | passive record |
| crates/kivact-ui-soft/src/citation.rs:40 | CitationSource.icon | passive record |
| crates/kivact-ui-soft/src/container_reveal.rs:36 | RevealContent.inverted | builder option |
| crates/kivact-ui-soft/src/container_reveal.rs:39 | RevealContent.preserve_layout | builder option |
| crates/kivact-ui-soft/src/container_reveal.rs:42 | RevealContent.force | builder option |
| crates/kivact-ui-soft/src/container_reveal.rs:45 | RevealContent.container_force | builder option |
| crates/kivact-ui-soft/src/dialog_header.rs:37 | DialogHeader.subtitle | builder option |
| crates/kivact-ui-soft/src/dialog_header.rs:39 | DialogHeader.start | builder option |
| crates/kivact-ui-soft/src/dialog_header.rs:41 | DialogHeader.end | builder option |
| crates/kivact-ui-soft/src/dialog_header.rs:43 | DialogHeader.close | builder option |
| crates/kivact-ui-soft/src/dialog_header.rs:45 | DialogHeader.compensation | builder option |
| crates/kivact-ui-soft/src/dialog_header.rs:47 | DialogHeader.divider | builder option |
| crates/kivact-ui-soft/src/disclosure_appearance.rs:22 | DisclosureVariant (struct-level) | builder option |
| crates/kivact-ui-soft/src/divider.rs:38 | Divider.label | builder option |
| crates/kivact-ui-soft/src/divider.rs:40 | Divider.full_bleed | builder option |
| crates/kivact-ui-soft/src/item_values.rs:66 | ItemLook (struct-level) | builder option |
| crates/kivact-ui-soft/src/layout.rs:9 | LayoutInset.outer_x | passive record |
| crates/kivact-ui-soft/src/layout.rs:12 | LayoutInset.outer_y | passive record |
| crates/kivact-ui-soft/src/layout.rs:15 | LayoutInset.inner_x | passive record |
| crates/kivact-ui-soft/src/layout.rs:18 | LayoutInset.inner_y | passive record |
| crates/kivact-ui-soft/src/layout.rs:21 | LayoutInset.own_outer_x | passive record |
| crates/kivact-ui-soft/src/layout.rs:24 | LayoutInset.own_outer_y | passive record |
| crates/kivact-ui-soft/src/layout.rs:75 | Layout.header | builder option |
| crates/kivact-ui-soft/src/layout.rs:77 | Layout.footer | builder option |
| crates/kivact-ui-soft/src/layout.rs:79 | Layout.start | builder option |
| crates/kivact-ui-soft/src/layout.rs:81 | Layout.end | builder option |
| crates/kivact-ui-soft/src/layout.rs:83 | Layout.content | builder option |
| crates/kivact-ui-soft/src/layout.rs:85 | Layout.start_resize | builder option |
| crates/kivact-ui-soft/src/layout.rs:87 | Layout.end_resize | builder option |
| crates/kivact-ui-soft/src/layout.rs:90 | Layout.padding | builder option |
| crates/kivact-ui-soft/src/layout.rs:92 | Layout.content_width | builder option |
| crates/kivact-ui-soft/src/layout.rs:94 | Layout.default_dividers | builder option |
| crates/kivact-ui-soft/src/layout.rs:183 | LayoutSlots.root_ref | passive record |
| crates/kivact-ui-soft/src/layout.rs:185 | LayoutSlots.header_divider | passive record |
| crates/kivact-ui-soft/src/layout.rs:187 | LayoutSlots.footer_divider | passive record |
| crates/kivact-ui-soft/src/layout.rs:189 | LayoutSlots.has_start | passive record |
| crates/kivact-ui-soft/src/layout.rs:191 | LayoutSlots.has_end | passive record |
| crates/kivact-ui-soft/src/layout.rs:193 | LayoutSlots.content_width | passive record |
| crates/kivact-ui-soft/src/layout.rs:289 | LayoutRegion.divider | builder option |
| crates/kivact-ui-soft/src/layout.rs:291 | LayoutRegion.padding | builder option |
| crates/kivact-ui-soft/src/layout.rs:293 | LayoutRegion.padding_bottom | builder option |
| crates/kivact-ui-soft/src/layout.rs:295 | LayoutRegion.scrollable | builder option |
| crates/kivact-ui-soft/src/layout.rs:297 | LayoutRegion.width | builder option |
| crates/kivact-ui-soft/src/lightbox.rs:73 | Lightbox.index | caller binding |
| crates/kivact-ui-soft/src/lightbox.rs:75 | Lightbox.zoom | builder option |
| crates/kivact-ui-soft/src/lightbox.rs:77 | Lightbox.view | caller binding |
| crates/kivact-ui-soft/src/lightbox.rs:330 | gallery_image(zoom) | independent fact (fn parameter) |
| crates/kivact-ui-soft/src/lightbox.rs:401 | gallery_key(zoom) | independent fact (fn parameter) |
| crates/kivact-ui-soft/src/link.rs:53 | Link.label | builder option |
| crates/kivact-ui-soft/src/link.rs:56 | Link.press | builder option |
| crates/kivact-ui-soft/src/link.rs:58 | Link.new_tab_label | builder option |
| crates/kivact-ui-soft/src/link.rs:60 | Link.underline | builder option |
| crates/kivact-ui-soft/src/link.rs:62 | Link.external | builder option |
| crates/kivact-ui-soft/src/link.rs:64 | Link.standalone | builder option |
| crates/kivact-ui-soft/src/link.rs:67 | Link.size | builder option |
| crates/kivact-ui-soft/src/link.rs:69 | Link.weight | builder option |
| crates/kivact-ui-soft/src/link.rs:73 | Link.as_button | builder option |
| crates/kivact-ui-soft/src/link.rs:269 | LinkAppearance (struct-level) | passive record |
| crates/kivact-ui-soft/src/link.rs:273 | LinkAppearance.size | passive record |
| crates/kivact-ui-soft/src/link.rs:275 | LinkAppearance.weight | passive record |
| crates/kivact-ui-soft/src/link.rs:277 | LinkAppearance.underline | passive record |
| crates/kivact-ui-soft/src/link.rs:280 | LinkAppearance.standalone | passive record |
| crates/kivact-ui-soft/src/link.rs:283 | LinkAppearance.interactive | passive record |
| crates/kivact-ui-soft/src/markdown.rs:807 | Marks.strong | passive record |
| crates/kivact-ui-soft/src/markdown.rs:809 | Marks.italic | passive record |
| crates/kivact-ui-soft/src/markdown.rs:811 | Marks.deleted | passive record |
| crates/kivact-ui-soft/src/markdown.rs:813 | Marks.code | passive record |
| crates/kivact-ui-soft/src/markdown_parser.rs:16 | ListEntry.number | passive record |
| crates/kivact-ui-soft/src/markdown_parser.rs:18 | ListEntry.checked | passive record |
| crates/kivact-ui-soft/src/mobile_nav.rs:54 | MobileContext (struct-level) | passive record |
| crates/kivact-ui-soft/src/mobile_nav.rs:205 | MobileNav.open | caller binding |
| crates/kivact-ui-soft/src/mobile_nav.rs:209 | MobileNav.label | builder option |
| crates/kivact-ui-soft/src/mobile_nav.rs:211 | MobileNav.header | builder option |
| crates/kivact-ui-soft/src/mobile_nav.rs:213 | MobileNav.header_view | builder option |
| crates/kivact-ui-soft/src/mobile_nav.rs:325 | MobileNavToggle.label | builder option |
| crates/kivact-ui-soft/src/mobile_nav.rs:327 | MobileNavToggle.icon | builder option |
| crates/kivact-ui-soft/src/navigation_session.rs:14 | NavPanel (struct-level) | passive record |
| crates/kivact-ui-soft/src/nav_heading.rs:41 | NavHeadingMenuItem (struct-level) | builder option |
| crates/kivact-ui-soft/src/outline.rs:91 | Outline.active | caller binding |
| crates/kivact-ui-soft/src/outline.rs:95 | Outline.viewport | builder option |
| crates/kivact-ui-soft/src/outline.rs:97 | Outline.content | builder option |
| crates/kivact-ui-soft/src/outline.rs:100 | Outline.change | builder option |
| crates/kivact-ui-soft/src/outline.rs:102 | Outline.start | builder option |
| crates/kivact-ui-soft/src/outline.rs:104 | Outline.end | builder option |
| crates/kivact-ui-soft/src/outline.rs:243 | Navigation.viewport | passive record |
| crates/kivact-ui-soft/src/outline.rs:246 | Navigation.scroll_on_press | passive record |
| crates/kivact-ui-soft/src/outline.rs:248 | Navigation.controlled | caller binding |
| crates/kivact-ui-soft/src/outline.rs:251 | Navigation.start | passive record |
| crates/kivact-ui-soft/src/outline.rs:253 | Navigation.end | passive record |
| crates/kivact-ui-soft/src/outline.rs:489 | active_heading(active) | caller binding (fn parameter) |
| crates/kivact-ui-soft/src/outline.rs:493 | active_heading(viewport) | independent fact (fn parameter) |
| crates/kivact-ui-soft/src/outline.rs:495 | active_heading(content) | independent fact (fn parameter) |
| crates/kivact-ui-soft/src/power_search.rs:141 | FilterDraft (struct-level) | passive record |
| crates/kivact-ui-soft/src/recipes_metrics.rs:59 | MetadataAppearance (struct-level) | builder option |
| crates/kivact-ui-soft/src/recipes_table.rs:8 | TableRowAppearance (struct-level) | builder option |
| crates/kivact-ui-soft/src/scrollable_area.rs:47 | ScrollableArea (struct-level) | builder option |
| crates/kivact-ui-soft/src/scroll_observer.rs:9 | AxisScroll (struct-level) | passive record |
| crates/kivact-ui-soft/src/scroll_observer.rs:75 | ScrollOverflow (struct-level) | passive record |
| crates/kivact-ui-soft/src/section.rs:72 | Section.bleed_top | builder option |
| crates/kivact-ui-soft/src/section.rs:74 | Section.bleed_bottom | builder option |
| crates/kivact-ui-soft/src/side_navigation/context.rs:7 | SideNavCx (struct-level) | passive record |
| crates/kivact-ui-soft/src/side_navigation/items.rs:402 | ItemSlots.leading | builder option |
| crates/kivact-ui-soft/src/side_navigation/items.rs:404 | ItemSlots.selected_icon | builder option |
| crates/kivact-ui-soft/src/side_navigation/items.rs:406 | ItemSlots.end | builder option |
| crates/kivact-ui-soft/src/side_navigation/items.rs:408 | ItemSlots.actions | builder option |
| crates/kivact-ui-soft/src/side_navigation/items.rs:410 | ItemSlots.children | builder option |
| crates/kivact-ui-soft/src/side_navigation/items.rs:433 | ItemContext.selected | independent fact |
| crates/kivact-ui-soft/src/side_navigation/items.rs:435 | ItemContext.disabled | independent fact |
| crates/kivact-ui-soft/src/side_navigation/items.rs:438 | ItemContext.action | builder option |
| crates/kivact-ui-soft/src/side_navigation/shell.rs:375 | SideNavSlots (struct-level) | builder option |
| crates/kivact-ui-soft/src/side_navigation/shell.rs:378 | SideNavSlots.header | builder option |
| crates/kivact-ui-soft/src/side_navigation/shell.rs:380 | SideNavSlots.top | builder option |
| crates/kivact-ui-soft/src/side_navigation/shell.rs:382 | SideNavSlots.footer | builder option |
| crates/kivact-ui-soft/src/side_navigation/shell.rs:384 | SideNavSlots.icons | builder option |
| crates/kivact-ui-soft/src/soft_chat_tools.rs:70 | ToolCall.target | passive record |
| crates/kivact-ui-soft/src/soft_chat_tools.rs:73 | ToolCall.duration | passive record |
| crates/kivact-ui-soft/src/soft_chat_tools.rs:80 | ToolCall.error | passive record |
| crates/kivact-ui-soft/src/soft_tree.rs:5 | TreeLook (struct-level) | builder option |
| crates/kivact-ui-soft/src/stack.rs:77 | Padding.all | passive record |
| crates/kivact-ui-soft/src/stack.rs:79 | Padding.x | passive record |
| crates/kivact-ui-soft/src/stack.rs:81 | Padding.y | passive record |
| crates/kivact-ui-soft/src/stack.rs:83 | Padding.start | passive record |
| crates/kivact-ui-soft/src/stack.rs:85 | Padding.end | passive record |
| crates/kivact-ui-soft/src/stack.rs:87 | Padding.top | passive record |
| crates/kivact-ui-soft/src/stack.rs:89 | Padding.bottom | passive record |
| crates/kivact-ui-soft/src/tab_list.rs:95 | Tab (struct-level) | builder option |
| crates/kivact-ui-soft/src/tab_list.rs:101 | Tab.icon | builder option |
| crates/kivact-ui-soft/src/tab_list.rs:103 | Tab.selected_icon | builder option |
| crates/kivact-ui-soft/src/tab_list.rs:105 | Tab.label_hidden | builder option |
| crates/kivact-ui-soft/src/tab_list.rs:107 | Tab.end | builder option |
| crates/kivact-ui-soft/src/tab_list.rs:109 | Tab.panel | builder option |
| crates/kivact-ui-soft/src/tab_list.rs:163 | TabMenuOption (struct-level) | builder option |
| crates/kivact-ui-soft/src/tab_list.rs:168 | TabMenuOption.icon | builder option |
| crates/kivact-ui-soft/src/tab_list.rs:170 | TabMenuOption.disabled | builder option |
| crates/kivact-ui-soft/src/tab_list.rs:246 | TabList (struct-level) | builder option |
| crates/kivact-ui-soft/src/tab_list.rs:256 | TabList.full_bleed | builder option |
| crates/kivact-ui-soft/src/tab_list.rs:258 | TabList.content | builder option |
| crates/kivact-ui-soft/src/tab_list.rs:341 | FocusMember (struct-level) | passive record |
| crates/kivact-ui-soft/src/tab_list.rs:344 | FocusMember.selected | independent fact |
| crates/kivact-ui-soft/src/tab_list.rs:346 | FocusMember.disabled | independent fact |
| crates/kivact-ui-soft/src/tab_list.rs:836 | TabRuntime (struct-level) | passive record |
| crates/kivact-ui-soft/src/tab_list.rs:840 | TabRuntime.behavior | passive record |
| crates/kivact-ui-soft/src/tab_list.rs:843 | TabRuntime.scrollable | builder option |
| crates/kivact-ui-soft/src/thumbnail.rs:57 | Thumbnail.source | builder option |
| crates/kivact-ui-soft/src/thumbnail.rs:59 | Thumbnail.alt | builder option |
| crates/kivact-ui-soft/src/thumbnail.rs:61 | Thumbnail.label | builder option |
| crates/kivact-ui-soft/src/thumbnail.rs:63 | Thumbnail.press | builder option |
| crates/kivact-ui-soft/src/thumbnail.rs:65 | Thumbnail.remove | builder option |
| crates/kivact-ui-soft/src/thumbnail.rs:67 | Thumbnail.loading | caller binding |
| crates/kivact-ui-soft/src/thumbnail.rs:69 | Thumbnail.reveal | builder option |
| crates/kivact-ui-soft/src/thumbnail.rs:227 | thumbnail_media(source) | independent fact (fn parameter) |
| crates/kivact-ui-soft/src/thumbnail.rs:229 | thumbnail_media(alt) | independent fact (fn parameter) |
| crates/kivact-ui-soft/src/thumbnail.rs:231 | thumbnail_media(loading) | caller binding (fn parameter) |
| crates/kivact-ui-soft/src/thumbnail.rs:337 | thumbnail_open(press) | independent fact (fn parameter) |
| crates/kivact-ui-soft/src/thumbnail.rs:340 | thumbnail_open(loading) | caller binding (fn parameter) |
| crates/kivact-ui-soft/src/thumbnail.rs:388 | thumbnail_remove(remove) | independent fact (fn parameter) |
| crates/kivact-ui-soft/src/thumbnail.rs:392 | thumbnail_remove(reveal) | builder option (fn parameter) |
| crates/kivact-ui-soft/src/timestamp.rs:163 | TimestampEntry.zone | passive record |
| crates/kivact-ui-soft/src/timestamp.rs:168 | TimestampEntry.label | passive record |
| crates/kivact-ui-soft/src/timestamp.rs:171 | TimestampEntry.copyable | passive record |
| crates/kivact-ui-soft/src/timestamp.rs:211 | Timestamp.details | builder option |
| crates/kivact-ui-soft/src/timestamp.rs:214 | Timestamp.live | builder option |
| crates/kivact-ui-soft/src/token.rs:78 | Token (struct-level) | builder option |
| crates/kivact-ui-soft/src/top_nav.rs:61 | Slots (struct-level) | builder option |
| crates/kivact-ui-soft/src/top_nav.rs:193 | TopNavItem (struct-level) | builder option |
| crates/kivact-ui-soft/src/top_nav.rs:474 | TopNavMegaMenu (struct-level) | builder option |
| crates/kivact-ui-soft/src/top_nav.rs:606 | TopNavMegaMenuItem (struct-level) | builder option |
| crates/kivact-ui-soft/src/top_nav.rs:733 | TopNavMegaMenuFeaturedCard (struct-level) | builder option |

## 3. FIX entries

### SM-SOFT-01 — tab_list roving cursor survives focus leave

- Location: crates/kivact-ui-soft/src/tab_list.rs:350-422 (FocusTrack + impl), 829-833 (constructor).
- Current fields (verbatim, lines 350-358):

```rust
// xtask-allow(state-fields): One roving cursor fact plus actual read-only focus, selection and disabled projections; there is no second mutable lifecycle field.
#[derive(Clone)]
struct FocusTrack {
    // xtask-allow(state-fields): Mounted NodeRef registry with actual read-only selection and disabled projections.
    members: Rc<[FocusMember]>,
    // xtask-allow(state-fields): Actual NodeRef focus-within projection, not an independently writable lifecycle fact.
    focused_within: Value<bool>,
    roving: Signal<Option<usize>>,
}
```

- Why implicit: (b) the `Option<usize>` cursor is valid only in the "focus inside the strip" phase, but it is never cleared when focus leaves; validity is re-derived on every read by the `focused_within` gate. Consequence: arrow to tab 2, leave, Tab back in: focus lands on the selected tab, `focused_within` turns true, the stale cursor makes the focused node non-focusable and the engine moves focus elsewhere. (S-1/S-3/S-5: the phase must be a state with a transition on leave.)
- Replacement code. Insert immediately before `struct FocusTrack` (i.e. replace lines 350-358 with the block below):

```rust
/// The keyboard tab stop inside the strip; it lasts only while focus stays inside.
#[derive(Clone, Copy, PartialEq)]
enum TabCursor {
    /// The selected, or else the first enabled, member is the tab stop.
    Resting,
    /// An arrow, Home or End key moved the tab stop to this member.
    At(usize),
}
enum CursorEvent {
    /// A navigation key moved focus to this member.
    Move(usize),
    /// Keyboard focus is now inside (`true`) or outside (`false`) the strip.
    Within(bool),
}
struct TabRoving;
impl Machine for TabRoving {
    type State = TabCursor;
    type Event = CursorEvent;
    type Effect = ();
    fn transition(&self, _: &TabCursor, event: CursorEvent) -> Step<TabCursor, CursorEvent, ()> {
        match event {
            CursorEvent::Move(index) => Step::to(TabCursor::At(index)),
            CursorEvent::Within(true) => Step::stay(),
            CursorEvent::Within(false) => Step::to(TabCursor::Resting),
        }
    }
}

#[derive(Clone)]
struct FocusTrack {
    members: Rc<[FocusMember]>,
    cursor: MachineHandle<TabRoving>,
}
```

- Every read/write of the old fields and what it becomes:
  - tab_list.rs:365-368 (read `roving` + `focused_within` in `FocusTrack::focusable`). Replace

    ```rust
            let preferred = self
                .roving
                .get()
                .filter(|index| self.focused_within.get() && !self.members[*index].disabled.get());
    ```
    with
    ```rust
            let preferred = match self.cursor.get() {
                TabCursor::At(roved) if !self.members[roved].disabled.get() => Some(roved),
                TabCursor::At(_) | TabCursor::Resting => None,
            };
    ```
    (the rest of `focusable`, lines 369-380, is unchanged).
  - tab_list.rs:417 (write `self.roving.set(Some(next));` in `FocusTrack::key`) becomes `self.cursor.send(CursorEvent::Move(next));` (line 418 `self.members[next].reference.focus();` stays after it).
  - tab_list.rs:829-833 (construction in `focus_track`). Replace

    ```rust
        FocusTrack {
            members: members.into(),
            focused_within: strip_ref.focus_within(),
            roving: signal(None),
        }
    ```
    with
    ```rust
        let cursor = machine(TabCursor::Resting, || TabRoving, |_, ()| {});
        cursor.feed(strip_ref.focus_within(), CursorEvent::Within);
        FocusTrack {
            members: members.into(),
            cursor,
        }
    ```
  - No other reads (`focused_within` / `roving` appear only at 356, 357, 365-368, 417, 831, 832).
  - Imports in tab_list.rs: delete `use kivact::state::Signal;` (line 31) and `use kivact::state::signal;` (line 35) — no other uses; add `use kivact::state::Machine;`, `use kivact::state::MachineHandle;`, `use kivact::state::Step;`, `use kivact::state::machine;`.
- xtask-allow lines to delete: tab_list.rs 350, 353, 355 (all removed by the replacement block above). FocusMember (341/344/346) stays.
- Public signature change: none.
- Tests: crates/kivact-ui-soft/tests/consumer/navigation.rs::layout_components_mount_and_interact (mount + click only). No test found for keyboard roving or focus re-entry.

### SM-SOFT-02 — side navigation item: dependent Options and duplicated facts in ItemContext

- Location: crates/kivact-ui-soft/src/side_navigation/items.rs:413-500 (struct + constructor), reads at 503, 560-561, 605-610, 626, 653, 677-679, 696, 704-705, 751.
- Current fields (verbatim, lines 413-440):

```rust
#[derive(Clone, Copy)]
struct ItemContext {
    cx: SideNavCx,
    mode: ShellNavMode,
    // xtask-allow(state-fields): Derived from the presence of the immutable children factory, not an expansion phase.
    has_children: bool,
    mobile: crate::mobile_nav::AppShellMobile,
    // xtask-allow(state-fields): Inverted view of the existing collapse binding; no second mutable open state is allocated.
    open: Option<Binding<bool>>,
    // xtask-allow(state-fields): Shared DisclosureApi owns its phase; this field retains only its handle.
    disclosure: Option<kivact::interaction::DisclosureApi>,
    // xtask-allow(state-fields): Derived from separate action and disclosure availability, not a gesture phase.
    independent: bool,
    anchor: kivact::view::NodeRef,
    popup_ref: kivact::view::NodeRef,
    // xtask-allow(state-fields): Existing shared navigation-panel handle; its lifecycle remains in the canonical controller.
    popup_panel: Option<crate::navigation_session::NavPanel>,
    // xtask-allow(state-fields): The existing panel open binding, never mirrored into a second signal.
    popup: Option<Binding<bool>>,
    label: Value<String>,
    // xtask-allow(state-fields): Actual caller selection projection, independent of disclosure and popup phases.
    selected: Value<bool>,
    // xtask-allow(state-fields): Actual root availability projection, never copied into mutable local state.
    disabled: Value<bool>,
    size: Value<ControlSize>,
    // xtask-allow(state-fields): Immutable caller activation callback, independent of the navigation lifecycle.
    action: Option<Callback<()>>,
}
```

- Why implicit: (a) dependent Some/None — `disclosure` is Some only if `has_children && open.is_some()`, `popup_panel` only if `has_children && cx.collapsed.is_some()`, `popup` is Some exactly when `popup_panel` is (S-3); (h) `popup` is `popup_panel.open()` stored twice, `independent` is `disclosure.is_some() && action.is_some()` stored twice, `has_children` duplicates the presence of `ItemSlots.children` (S-4).
- Replacement code. Replace lines 413-440 with:

```rust
/// The collapse owner of an item that has a children factory and a collapse toggle.
#[derive(Clone, Copy)]
struct ItemExpansion {
    /// Inverted view of the item's collapse binding.
    open: Binding<bool>,
    disclosure: kivact::interaction::DisclosureApi,
}

/// Whether the item has a children factory, with that branch's owners.
#[derive(Clone, Copy)]
enum ItemBranch {
    /// No children factory.
    Leaf,
    /// A children factory exists.
    Parent {
        // xtask-allow(state-fields): independent fact - the collapse owner of the item, absent for ItemCollapse::AlwaysOpen
        /// `None` for `ItemCollapse::AlwaysOpen`.
        expansion: Option<ItemExpansion>,
        // xtask-allow(state-fields): independent fact - the rail flyout panel, present only inside a collapsible sidebar
        /// The rail flyout panel; present only inside a collapsible sidebar.
        flyout: Option<crate::navigation_session::NavPanel>,
    },
}

impl ItemBranch {
    fn has_children(self) -> bool {
        matches!(self, Self::Parent { .. })
    }
    fn disclosure(self) -> Option<kivact::interaction::DisclosureApi> {
        match self {
            Self::Parent {
                expansion: Some(expansion),
                ..
            } => Some(expansion.disclosure),
            Self::Parent {
                expansion: None, ..
            }
            | Self::Leaf => None,
        }
    }
    fn open(self) -> Option<Binding<bool>> {
        match self {
            Self::Parent {
                expansion: Some(expansion),
                ..
            } => Some(expansion.open),
            Self::Parent {
                expansion: None, ..
            }
            | Self::Leaf => None,
        }
    }
    fn flyout(self) -> Option<crate::navigation_session::NavPanel> {
        match self {
            Self::Parent { flyout, .. } => flyout,
            Self::Leaf => None,
        }
    }
}

#[derive(Clone, Copy)]
struct ItemContext {
    cx: SideNavCx,
    mode: ShellNavMode,
    mobile: crate::mobile_nav::AppShellMobile,
    branch: ItemBranch,
    anchor: kivact::view::NodeRef,
    popup_ref: kivact::view::NodeRef,
    label: Value<String>,
    // xtask-allow(state-fields): Actual caller selection projection, independent of disclosure and popup phases.
    selected: Value<bool>,
    // xtask-allow(state-fields): Actual root availability projection, never copied into mutable local state.
    disabled: Value<bool>,
    size: Value<ControlSize>,
    // xtask-allow(state-fields): Immutable caller activation callback, independent of the navigation lifecycle.
    action: Option<Callback<()>>,
}
```

  Replace the body of `ItemContext::new` (lines 451-499, from `let cx = nav_cx();` through the closing `}` of `Self { ... }`) with:

```rust
        let cx = nav_cx();
        let mode = shell_nav_mode();
        let mobile = use_app_shell_mobile();
        let branch = if has_children {
            let collapse = match collapse.get() {
                ItemCollapse::AlwaysOpen => None,
                ItemCollapse::Uncontrolled { default_collapsed } => {
                    Some(signal(default_collapsed).into_binding())
                }
                ItemCollapse::Controlled(binding) => Some(binding),
            };
            let expansion = collapse.map(|collapse: Binding<bool>| {
                let open = binding(move || !collapse.get(), move |open| collapse.set(!open));
                ItemExpansion {
                    open,
                    disclosure: use_disclosure(open).disabled(disabled),
                }
            });
            let flyout = cx.collapsed.is_some().then(|| {
                crate::navigation_session::NavPanel::new(
                    move || cx.rail() && !disabled.get(),
                    Delays {
                        open: Duration::from_millis(150),
                        close: Duration::from_millis(200),
                    },
                )
            });
            ItemBranch::Parent { expansion, flyout }
        } else {
            ItemBranch::Leaf
        };
        let anchor = node_ref();
        let popup_ref = node_ref();
        Self {
            cx,
            mode,
            mobile,
            branch,
            anchor,
            popup_ref,
            label,
            selected,
            disabled,
            size,
            action,
        }
```

  Add this method inside `impl ItemContext` (e.g. directly after `new`):

```rust
    fn independent(self) -> bool {
        self.branch.disclosure().is_some() && self.action.is_some()
    }
```

- Every read/write of the old fields and what it becomes (constructor writes at 485-493 are replaced above):
  - items.rs:503 `let independent = self.independent;` → `let independent = self.independent();`
  - items.rs:560 `let disclosure = self.disclosure;` → `let disclosure = self.branch.disclosure();`
  - items.rs:561 `let independent = self.independent;` → `let independent = self.independent();`
  - items.rs:605 `let has_children = self.has_children;` → `let has_children = self.branch.has_children();`
  - items.rs:606 `let disclosure = self.disclosure;` → `let disclosure = self.branch.disclosure();`
  - items.rs:607 `let independent = self.independent;` → `let independent = self.independent();`
  - items.rs:609 `let popup_panel = self.popup_panel;` → `let popup_panel = self.branch.flyout();`
  - items.rs:610 `let popup = self.popup;` → delete the line.
  - items.rs:626 `!disabled.get() && (action.is_some() || disclosure.is_some() || popup.is_some())` → `!disabled.get() && (action.is_some() || disclosure.is_some() || popup_panel.is_some())`
  - items.rs:653 `if let Some(disclosure) = disclosure.filter(|_| popup.is_none()) {` → `if let Some(disclosure) = disclosure.filter(|_| popup_panel.is_none()) {`
  - items.rs:677 `let open = self.open;` → `let open = self.branch.open();`
  - items.rs:678 `let popup_panel = self.popup_panel;` → `let popup_panel = self.branch.flyout();`
  - items.rs:679 `let popup = self.popup;` → delete the line.
  - items.rs:696 `} else if popup.is_some()` → `} else if popup_panel.is_some()`
  - items.rs:704 `let has_children = self.has_children;` → `let has_children = self.branch.has_children();`
  - items.rs:705 `let disclosure = self.disclosure;` → `let disclosure = self.branch.disclosure();`
  - items.rs:751 `let popup_panel = self.popup_panel;` → `let popup_panel = self.branch.flyout();`
  - items.rs:184 (`children.is_some()` passed as the `has_children` argument of `ItemContext::new`) is unchanged; it is now a constructor argument only.
  - No new imports (Binding, binding, signal, use_disclosure, Delays, Duration, node_ref already imported).
- xtask-allow lines to delete: items.rs 417, 420, 422, 424, 428, 430 (removed by the struct replacement). 433, 435, 438 are kept verbatim in the new struct.
- Behaviour: identical, except leaf items no longer allocate an unused collapse signal.
- Public signature change: none.
- Tests: crates/kivact-ui-soft/tests/consumer/navigation.rs::shell_adapts_navigation_and_opens_the_mobile_drawer, ::rail_navigation_has_one_actual_panel_and_rearms_after_expansion.

### SM-SOFT-03 — SideNavResize stores "uncontrolled default" beside "controlled binding"

- Location: crates/kivact-ui-soft/src/side_navigation/shell.rs:59-80 (struct + Default), 103-112 (setters), 361-366 (reader).
- Current fields (verbatim, lines 59-68):

```rust
// xtask-allow(state-fields): Construction metrics and optional caller collapse binding; the default is used only without that binding and is not a runtime lifecycle flag.
/// Initial width, width bounds and optional collapse binding for a resizable sidebar.
/// `Default` starts expanded at 260 px, bounded to 180–480 px.
pub struct SideNavResize {
    default_width: f64,
    min: f64,
    max: f64,
    default_collapsed: bool,
    collapsed: Option<Binding<bool>>,
}
```

- Why implicit: (a) `default_collapsed` is valid only while `collapsed` is None (S-3: dependent pair). The enum that expresses exactly this already exists in the same file: `SideNavCollapse::{Uncontrolled { default }, Controlled(Binding<bool>)}` (shell.rs:49-57).
- Replacement code. Replace lines 59-80 with:

```rust
/// Initial width, width bounds and collapse ownership for a resizable sidebar.
/// `Default` starts expanded at 260 px, bounded to 180–480 px.
pub struct SideNavResize {
    default_width: f64,
    min: f64,
    max: f64,
    collapse: SideNavCollapse,
}

impl Default for SideNavResize {
    fn default() -> Self {
        Self {
            default_width: 260.0,
            min: 180.0,
            max: 480.0,
            collapse: SideNavCollapse::Uncontrolled { default: false },
        }
    }
}
```

  Replace the two setters (lines 103-112) with:

```rust
    /// Start an uncontrolled sidebar collapsed.
    pub fn initially_collapsed(mut self) -> Self {
        if let SideNavCollapse::Uncontrolled { default } = &mut self.collapse {
            *default = true;
        }
        self
    }
    /// Use the caller's single collapse binding.
    pub fn collapsed(mut self, collapsed: impl IntoBinding<bool>) -> Self {
        self.collapse = SideNavCollapse::Controlled(collapsed.into_binding());
        self
    }
```

- Every read/write of the old fields:
  - shell.rs:76-77 (Default writes) → replaced above.
  - shell.rs:105 `self.default_collapsed = true;` → replaced above (only takes effect while uncontrolled, as before).
  - shell.rs:110 `self.collapsed = Some(collapsed.into_binding());` → replaced above.
  - shell.rs:362-366 in `sidebar_collapse`. Replace

    ```rust
            .map(|resize| {
                resize
                    .collapsed
                    .unwrap_or_else(|| signal(resize.default_collapsed).into_binding())
            })
    ```
    with
    ```rust
            .map(|resize| match &resize.collapse {
                SideNavCollapse::Uncontrolled { default } => signal(*default).into_binding(),
                SideNavCollapse::Controlled(collapsed) => *collapsed,
            })
    ```
  - No other reads (`range()` at 91-101 uses only `min`/`max`/`Self::default()`).
- xtask-allow lines to delete: shell.rs 59.
- Behaviour: identical for both call orders (`collapsed()` then `initially_collapsed()` keeps the binding, as before).
- Public signature change: none (fields are private; method signatures unchanged).
- Tests: no test found (navigation.rs uses `.collapsible(...)`, never `.resizable(SideNavResize)`).

### SM-SOFT-04 — ShellPresentation copies AppShellMobile facts

- Location: crates/kivact-ui-soft/src/app_shell.rs:252-263, 333-347, 348-384, 476-503, 642-646.
- Current fields (verbatim, lines 337-340):

```rust
    // xtask-allow(state-fields): Readonly fact derived from the configured mobile navigation policy.
    auto_toggle: bool,
    // xtask-allow(state-fields): Readonly construction fact indicating configured navigation content.
    enabled: bool,
```

- Why implicit: (h) `enabled` is exactly `mobile.is_enabled()` (`AppShellMobile::configured` returns a context iff `open` is Some iff `enabled`), and `auto_toggle` is the `MobileContext.auto_toggle` already stored inside `mobile` (read by `mobile.has_auto_toggle()`); every read is guarded by `mobile.is_mobile()`, which implies the context exists (S-4).
- Replacement code:
  - Delete lines 337-340 (the two annotations and the two fields).
  - `configure_mobile` (lines 348-384): change the signature (lines 348-351)

    ```rust
    fn configure_mobile(
        mobile: MobileNavMode,
        has_nav: bool,
    ) -> (AppShellMobile, MobileNavConfig, Option<ViewFn>, bool, bool) {
    ```
    to
    ```rust
    fn configure_mobile(
        mobile: MobileNavMode,
        has_nav: bool,
    ) -> (AppShellMobile, MobileNavConfig, Option<ViewFn>) {
    ```
    and the last line (383) `(mobile, config, custom, auto_toggle, enabled)` to `(mobile, config, custom)`. Lines 359 (`let enabled = ...`) and 381 (`let auto_toggle = ...`) stay; they are still used locally.
  - into_view (lines 252-263) becomes:

    ```rust
            let (mobile, config, custom) =
                configure_mobile(mobile, top.is_some() || side.is_some());
            let presentation = ShellPresentation {
                mobile,
                height,
                variant,
                has_top: top.is_some(),
                has_side: side.is_some(),
                has_banner: banner.is_some(),
            };
    ```
- Every read of the old fields and what it becomes:
  - app_shell.rs:476-485 destructure in `header`: remove `auto_toggle,` and `enabled,` from the `let Self { ... } = self;` pattern.
  - app_shell.rs:496 `move || mobile.is_mobile() && (has_top || auto_toggle && enabled),` → `move || mobile.is_mobile() && (has_top || mobile.has_auto_toggle() && mobile.is_enabled()),`
  - app_shell.rs:497 `move || mobile_bar(mobile_top.clone(), bar_side.clone(), auto_toggle),` → `move || mobile_bar(mobile_top.clone(), bar_side.clone(), mobile.has_auto_toggle()),`
  - app_shell.rs:503 `let visible = has_banner || has_top || (mobile.is_mobile() && auto_toggle && enabled);` → `let visible = has_banner || has_top || (mobile.is_mobile() && mobile.has_auto_toggle() && mobile.is_enabled());`
  - app_shell.rs:642-644 `let Self { mobile, enabled, .. } = self;` → `let Self { mobile, .. } = self;`
  - app_shell.rs:646 `move || enabled && mobile.is_mobile(),` → `move || mobile.is_enabled() && mobile.is_mobile(),`
- xtask-allow lines to delete: app_shell.rs 337, 339. (331, 341, 343, 345 stay: has_* are presence snapshots of slots moved into views.)
- Public signature change: none (private fn and struct).
- Tests: crates/kivact-ui-soft/tests/consumer/navigation.rs::shell_adapts_navigation_and_opens_the_mobile_drawer.

## 4. UNANNOTATED-FIX entries

### SM-SOFT-05 — AppShellMobile machine mirrors the drawer binding and repairs it by effect

- Location: crates/kivact-ui-soft/src/mobile_nav.rs:63-157, 170-187.
- Current code (verbatim, lines 63-94 and 107-130):

```rust
#[derive(Clone, PartialEq)]
enum AppShellMobileState {
    Desktop,
    Mobile { nav: MobileNavState },
}
#[derive(Clone, PartialEq)]
enum MobileNavState {
    Closed,
    Open,
}
enum MobileMachine {
    Desktop { nav: MobileNavState },
    Mobile { nav: MobileNavState },
}
impl Machine for MobileMachine {
    type State = AppShellMobileState;
    type Event = ();
    type Effect = ();
    fn transition(&self, _: &AppShellMobileState, (): ()) -> Step<AppShellMobileState, (), ()> {
        match self {
            Self::Mobile { nav } => Step::to(AppShellMobileState::Mobile { nav: nav.clone() }),
            Self::Desktop { nav } => {
                let step = Step::to(AppShellMobileState::Desktop);
                if *nav == MobileNavState::Open {
                    step.with(())
                } else {
                    step
                }
            }
        }
    }
}
```
```rust
        let state = machine(
            AppShellMobileState::Desktop,
            move || {
                let nav = if open.get() {
                    MobileNavState::Open
                } else {
                    MobileNavState::Closed
                };
                if mobile.is_some_and(|mobile| mobile.get()) {
                    MobileMachine::Mobile { nav }
                } else {
                    MobileMachine::Desktop { nav }
                }
            },
            move |_, ()| open.set(false),
        );
        state.feed(
            move || (mobile.is_some_and(|mobile| mobile.get()), open.get()),
            |_| (),
        );
        state.send(());
```

- Why implicit: (h) `Mobile { nav }` copies the caller-owned open binding into machine state (S-4; caller binding owner = caller); (e) the feed on `open` plus the Desktop effect `open.set(false)` re-closes the caller binding whenever it becomes true on desktop — an effect that repairs state / reverse-syncs a caller binding (S-5, 事実の持ち主 "caller binding … effect で逆同期しない"); (g) the machine is created with a placeholder `Desktop` and fixed by an immediate `state.send(())` (phase by call order, S-6). The transition ignores state and event entirely (state = f(input)).
- Replacement code. Replace lines 63-94 with:

```rust
#[derive(Clone, Copy, PartialEq)]
enum AppShellMobileState {
    Desktop,
    Mobile,
}
/// Readonly snapshot read for each viewport-width event.
struct MobileMachine {
    /// Whether the drawer binding is open when the width event arrives.
    open: bool,
}
impl Machine for MobileMachine {
    type State = AppShellMobileState;
    /// Whether the viewport is below the mobile breakpoint.
    type Event = bool;
    /// Close the drawer binding.
    type Effect = ();
    fn transition(
        &self,
        state: &AppShellMobileState,
        below: bool,
    ) -> Step<AppShellMobileState, bool, ()> {
        match (state, below) {
            (_, true) => Step::to(AppShellMobileState::Mobile),
            (AppShellMobileState::Mobile, false) if self.open => {
                Step::to(AppShellMobileState::Desktop).with(())
            }
            (_, false) => Step::to(AppShellMobileState::Desktop),
        }
    }
}
```

  Replace lines 110-130 (from `let state = machine(` through `state.send(());`) with:

```rust
        let initial = if mobile.is_some_and(|mobile| mobile.get_untracked()) {
            AppShellMobileState::Mobile
        } else {
            AppShellMobileState::Desktop
        };
        let state = machine(
            initial,
            move || MobileMachine { open: open.get() },
            move |_, ()| open.set(false),
        );
        if let Some(mobile) = mobile {
            state.feed(mobile, |below| below);
        }
```

- Every read/write and what it becomes:
  - mobile_nav.rs:142-146 `is_mobile`: body becomes
    `self.context.is_some_and(|context| context.state.get() == AppShellMobileState::Mobile)`.
  - mobile_nav.rs:148-157 `is_open`: body becomes
    `self.context.is_some_and(|context| context.state.get() == AppShellMobileState::Mobile && context.open.get())`.
  - mobile_nav.rs:171-175 `open`: replace the body with a runtime guard (S-6) so desktop requests are ignored instead of being written and repaired:
    ```rust
        if let Some(context) = self.context
            && self.is_mobile()
        {
            context.open.set(true);
        }
    ```
    and change its doc line 170 to `/// Request that the configured drawer open; do nothing outside a shell or in the desktop layout.`
  - mobile_nav.rs:183-187 `toggle`: same guard:
    ```rust
        if let Some(context) = self.context
            && self.is_mobile()
        {
            context.open.set(!context.open.get());
        }
    ```
    doc line 182 becomes `/// Invert the configured drawer's open binding; do nothing outside a shell or in the desktop layout.`
  - `close` (177-181) unchanged. `MobileNavState` is deleted (no other uses). Imports unchanged.
- xtask-allow lines to delete: none (MobileContext line 54 stays).
- Behaviour changes (intended): the drawer binding is closed only on a Mobile→Desktop layout transition; a caller binding that is `true` at startup on desktop, or set `true` by the caller while desktop, is no longer rewritten (previously rewritten to false, also firing `on_open_change` twice). `open()`/`toggle()` called on desktop are no-ops, which matches the previous net effect.
- Public signature change: none (doc text of `AppShellMobile::open`/`toggle` only).
- Tests: crates/kivact-ui-soft/tests/consumer/navigation.rs::shell_adapts_navigation_and_opens_the_mobile_drawer.

### SM-SOFT-06 — outline roving cursor survives focus leave

- Location: crates/kivact-ui-soft/src/outline.rs:226-239, 364-365, 391-398, 576.
- Current code (verbatim, 226-239):

```rust
#[derive(Clone, PartialEq)]
enum OutlineFocus {
    Unset,
    At(String),
}
struct FocusMachine;
impl Machine for FocusMachine {
    type State = OutlineFocus;
    type Event = String;
    type Effect = ();
    fn transition(&self, _: &OutlineFocus, id: String) -> Step<OutlineFocus, String, ()> {
        Step::to(OutlineFocus::At(id))
    }
}
```

- Why implicit: (b) `At(id)` is valid only while focus is inside the list, but no transition leaves it; every reader re-gates it with `focused.get()` (outline.rs:392). Same failure as SM-SOFT-01: after leaving and Tabbing back in, the stale cursor makes the focused link non-focusable and the engine moves focus (possibly out of the list).
- Replacement code. Replace lines 226-239 with:

```rust
#[derive(Clone, PartialEq)]
enum OutlineFocus {
    Unset,
    At(String),
}
enum FocusEvent {
    /// A navigation key moved focus to this heading link.
    Move(String),
    /// Keyboard focus is now inside (`true`) or outside (`false`) the list.
    Within(bool),
}
struct FocusMachine;
impl Machine for FocusMachine {
    type State = OutlineFocus;
    type Event = FocusEvent;
    type Effect = ();
    fn transition(&self, _: &OutlineFocus, event: FocusEvent) -> Step<OutlineFocus, FocusEvent, ()> {
        match event {
            FocusEvent::Move(id) => Step::to(OutlineFocus::At(id)),
            FocusEvent::Within(true) => Step::stay(),
            FocusEvent::Within(false) => Step::to(OutlineFocus::Unset),
        }
    }
}
```

- Every read/write and what it becomes:
  - outline.rs:365 `let roving = machine(OutlineFocus::Unset, || FocusMachine, |_, ()| {});` → keep, and add the next line `roving.feed(focused, FocusEvent::Within);` (line 364 `let focused = list.focus_within();` stays).
  - outline.rs:391-398 `focusable` closure becomes:
    ```rust
            let focusable = move || {
                if let OutlineFocus::At(roving) = roving.get() {
                    return roving == roving_id;
                }
                selected.get() || active.get().is_none() && index == 0
            };
    ```
  - outline.rs:576 `roving.send(items[next].id.clone());` → `roving.send(FocusEvent::Move(items[next].id.clone()));`
  - outline.rs:407 and 567 (passing `MachineHandle<FocusMachine>`) unchanged.
- xtask-allow lines to delete: none.
- Public signature change: none.
- Tests: crates/kivact-ui-soft/tests/consumer/navigation.rs::layout_components_mount_and_interact (mount only). No keyboard test found.

### SM-SOFT-07 — rich text editor: mode as bool + Option<bool>; editor target/composition mirrored

- Location: crates/kivact-ui-soft/src/rich_text.rs:6-30 (imports), 62-129 (`rich_text_toolbar`, `toolbar`), 244-285 (`IntoView for RichTextEditor`); crates/kivact-ui-soft/src/rich_blocks.rs:17 (import), 56-114 (`Editor`, `edit_blocks`), 126 (`impl Editor`), 558-646 (`BlockMachine`).
- Current state (verbatim):

```rust
// rich_text.rs:67-71
fn toolbar(
    preview: Binding<bool>,
    editor: Option<(Signal<Option<NodeRef>>, Signal<bool>)>,
    blocks: Option<Signal<bool>>,
) -> Node {
// rich_text.rs:253-256
        let previewing = signal(read_only.get());
        let target = signal(None::<NodeRef>);
        let blocks = block_editing.get().then(|| signal(true));
        let composing = signal(false);
// rich_text.rs:83-89 (toolbar mode press)
                let changed = preview.get_untracked() != selected
                    || (!selected
                        && blocks.is_some_and(|blocks| blocks.get_untracked() != block_mode));
                if changed && let Some((target, composing)) = editor {
                    composing.set(false);
                    target.set(None);
                }
// rich_text.rs:268 (Write branch)
                    on_mount(move || target.set(Some(editor)));
// rich_blocks.rs:85-94 (BlockMachine effect)
            let (next_target, next_composing) = match snapshot.phase {
                BlockEditPhase::Focused {
                    target,
                    composition,
                    ..
                } => (Some(target), composition == BlockComposition::Composing),
                BlockEditPhase::Idle | BlockEditPhase::AwaitingMount { .. } => (None, false),
            };
            target.set(next_target);
            composing.set(next_composing);
```

- Why implicit: (a)/S-1 three modes Preview/Write/Blocks encoded as `previewing: bool` + `blocks: Option<Signal<bool>>` (the block flag is meaningless while previewing); (a)/S-3 `target` and `composing` are a dependent pair (composing only meaningful with a target); (e) the BlockMachine effect mirrors its own `Focused { target, composition }` phase into those signals, and the toolbar repairs them on every mode switch (S-5); (f) the Write text area fills `target` on mount (S-7); (h) the focused block and its composition are stored both in `BlockMachine` and in the signals (S-4).
- Replacement code.

  rich_text.rs imports: delete `use kivact::state::Signal;` (line 10) and `use kivact::state::on_mount;` (line 12); add `use kivact::state::binding;`, `use kivact::state::memo;`, `use kivact::state::on_cleanup;`.

  Insert after `RichDoc`'s impl block (after line 55):

```rust
/// The editor that the formatting commands act on.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum EditTarget {
    /// No editor accepts commands.
    Detached,
    /// This editor accepts formatting.
    Ready(NodeRef),
    /// This editor is composing input; formatting waits.
    Composing(NodeRef),
}

#[derive(Clone, Copy, PartialEq)]
enum EditorMode {
    Preview,
    Write,
    Blocks,
}
```

  Replace `rich_text_toolbar` (lines 62-65) with:

```rust
/// A toolbar for switching the Markdown editor between source and preview.
pub fn rich_text_toolbar(preview: impl IntoBinding<bool>) -> impl kivact::view::Component {
    let preview = preview.into_binding();
    let mode = binding(
        move || {
            if preview.get() {
                EditorMode::Preview
            } else {
                EditorMode::Write
            }
        },
        move |mode| preview.set(mode == EditorMode::Preview),
    );
    toolbar(mode, None, false)
}
```

  Replace `toolbar` (lines 67-129) with:

```rust
fn toolbar(
    mode: Binding<EditorMode>,
    target: Option<Value<EditTarget>>,
    offers_blocks: bool,
) -> Node {
    let mode_button = |label, value: EditorMode| {
        button(label)
            .variant(ButtonVariant::Ghost)
            .size(ControlSize::Small)
            .with_node(move |node| node.selected(move || mode.get() == value))
            .on_press(move || mode.set(value))
    };
    let commands = target.map(|target| {
        let command = |label, marker| {
            button(label)
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Small)
                .disabled(move || !matches!(target.get(), EditTarget::Ready(_)))
                .on_press(move || {
                    if let EditTarget::Ready(editor) = target.get_untracked() {
                        format_selection(editor, marker);
                    }
                })
        };
        row((
            command(message("soft.rich_text.bold", "Bold"), "**"),
            command(message("soft.rich_text.italic", "Italic"), "*"),
            command(
                message("soft.rich_text.strikethrough", "Strikethrough"),
                "~~",
            ),
            command(message("soft.rich_text.inline_code", "Inline code"), "`"),
        ))
        .style(style().align_items(Align::Center).gap(4))
    });
    row((
        commands,
        offers_blocks
            .then(|| mode_button(message("soft.rich_text.blocks", "Blocks"), EditorMode::Blocks)),
        mode_button(message("soft.rich_text.write", "Write"), EditorMode::Write),
        mode_button(message("soft.rich_text.preview", "Preview"), EditorMode::Preview),
    ))
    .with_node(move |node| node.role(Role::Toolbar))
    .roving_focus(kivact::view::Orientation::Horizontal)
    .name(message("soft.rich_text.toolbar", "Editor toolbar"))
    .style(style().align_items(Align::Center).gap(4))
}
```

  Replace `IntoView for RichTextEditor` (lines 244-285) with:

```rust
impl IntoView for RichTextEditor {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            source,
            read_only,
            block_editing,
        } = self;
        let read_only = read_only.get();
        let offers_blocks = !read_only && block_editing.get();
        let mode = signal(if read_only {
            EditorMode::Preview
        } else if offers_blocks {
            EditorMode::Blocks
        } else {
            EditorMode::Write
        });
        let blocks = offers_blocks.then(|| crate::rich_blocks::block_editor(source));
        let write_ref = node_ref();
        // The Write text area's last composition report; its unmount reports `false`.
        let write_composing = signal(false);
        let surface = dynamic(
            move || mode.get(),
            move |current| match current {
                EditorMode::Preview => rich_text_view(move || source.get())
                    .style(style().padding(12))
                    .into_view(),
                EditorMode::Blocks => blocks
                    .map(|blocks| crate::rich_blocks::edit_blocks(label, blocks))
                    .into_view(),
                EditorMode::Write => {
                    on_cleanup(move || write_composing.set(false));
                    text_area(label, source)
                        .node_ref(write_ref)
                        .on_composition_change(move |active| {
                            write_composing.set(active == kivact::view::Composition::Composing)
                        })
                        .wrap(true)
                        .rows(6..=16)
                        .into_view()
                }
            },
        );
        let target = memo(move || match mode.get() {
            EditorMode::Preview => EditTarget::Detached,
            EditorMode::Write if write_composing.get() => EditTarget::Composing(write_ref),
            EditorMode::Write => EditTarget::Ready(write_ref),
            EditorMode::Blocks => blocks.map_or(EditTarget::Detached, |blocks| blocks.target()),
        });
        let toolbar =
            (!read_only).then(|| toolbar(mode.into_binding(), Some(target), offers_blocks));
        root.child(column((toolbar, surface)).style(style().gap(4)))
            .into_view()
    }
}
```

  rich_blocks.rs: delete `use kivact::state::Signal;` (line 17). Change line 57 `struct Editor {` to `pub(super) struct Editor {` (fields stay private). Replace `edit_blocks` (lines 64-114) with:

```rust
/// Create the block-editing owner once per editor; it outlives switches to Write and Preview.
pub(super) fn block_editor(source: Binding<String>) -> Editor {
    let references = var(HashMap::<usize, NodeRef>::new());
    let state = machine(
        BlockSnapshot {
            document: Document::parse(&source.get_untracked(), 0),
            phase: BlockEditPhase::Idle,
        },
        move || BlockMachine {
            source: source.get_untracked(),
            references: references.with(Clone::clone),
        },
        move |_, markdown: String| {
            if source.get_untracked() != markdown {
                source.set(markdown);
            }
        },
    );
    state.feed(move || source.get(), |_| BlockEvent::Source);
    let document = memo(move || state.get().document);
    Editor {
        source,
        document,
        state,
        references,
    }
}

/// Render the editable blocks of an existing block-editing owner.
pub(super) fn edit_blocks(label: Value<String>, editor: Editor) -> Node {
    let document = editor.document;
    let blocks = for_each(
        move || document.with(|doc| doc.blocks.clone()),
        |block| block.id,
        move |block| editor.block(label, block.with_untracked(|block| block.id), block),
    );
    column(blocks)
        .with_node(move |node| node.role(Role::Group))
        .name(label)
        .style(style().gap(8).padding(12).width(pct(100)).min_width(0))
}
```

  Add as the first method of `impl Editor` (line 126):

```rust
    /// The block field the toolbar formats, read from this owner's phase.
    pub(super) fn target(self) -> crate::rich_text::EditTarget {
        match self.state.get().phase {
            BlockEditPhase::Focused {
                target,
                composition: BlockComposition::Composing,
                ..
            } => crate::rich_text::EditTarget::Composing(target),
            BlockEditPhase::Focused { target, .. } => crate::rich_text::EditTarget::Ready(target),
            BlockEditPhase::Idle | BlockEditPhase::AwaitingMount { .. } => {
                crate::rich_text::EditTarget::Detached
            }
        }
    }
```

  In `impl Machine for BlockMachine` (558-646): line 561 `type Effect = BlockSnapshot;` → `type Effect = String;`; line 566 return type `Step<BlockSnapshot, BlockEvent, BlockSnapshot>` → `Step<BlockSnapshot, BlockEvent, String>`; line 644 `Step::to(next.clone()).with(next)` →

```rust
        let markdown = next.document.markdown();
        Step::to(next).with(markdown)
```

- Every read/write of the old state and what it becomes:
  - rich_text.rs:253 `previewing` (write: initial) → `mode` initial value; 258 read (`previewing.get()` in dynamic source) → `mode.get()`; 281 `previewing.into_binding()` → `mode.into_binding()`.
  - rich_text.rs:255 `blocks` Option<Signal<bool>> → removed; 258 read → part of `mode`; toolbar reads 79, 85, 90-92, 121 → `mode.get() == value`, `mode.set(value)`, `offers_blocks`.
  - rich_text.rs:254 `target` → removed; writes 88 (repair), 268 (on_mount) and rich_blocks.rs:93 (mirror) are deleted; reads 101, 103 → `target.get()` / `target.get_untracked()` of the `Value<EditTarget>` projection.
  - rich_text.rs:256 `composing` → removed; writes 87 (repair), 272 (Write report → now `write_composing`), rich_blocks.rs:94 (mirror, deleted); read 101 → `!matches!(target.get(), EditTarget::Ready(_))`.
  - rich_blocks.rs:64-69 `edit_blocks(label, source, target, composing)` caller rich_text.rs:265 → `crate::rich_blocks::edit_blocks(label, blocks)` with `blocks` from `block_editor(source)` created once in `into_view`.
- xtask-allow lines to delete: none.
- Behaviour: identical observable toolbar states; the BlockMachine now lives for the editor's lifetime instead of per Blocks entry (re-entering Blocks re-renders from the current document; block focus is cleared by the existing `BlockEvent::Detach` cleanups). With `read_only(true)` no block owner is created (it was never reachable).
- Public signature change: none (`rich_text_toolbar` keeps `impl IntoBinding<bool>`; `EditTarget` is `pub(crate)`).
- Tests: crates/kivact-ui-soft/tests/consumer/foundation.rs::rich_text_source_toolbar_wraps_and_unwraps_native_unicode_selection; crates/kivact-ui-soft/tests/consumer/rich_blocks.rs::block_editor_edits_heading_and_task_without_rewriting_neighboring_markdown, ::enter_splits_and_backspace_at_block_start_rejoins_without_losing_caret.

### SM-SOFT-08 — thumbnail copies the image load status into a flag

- Location: crates/kivact-ui-soft/src/thumbnail.rs:248-254, import line 12.
- Current code (verbatim, 248-254):

```rust
    if let Some(source) = source {
        let failed = signal(false);
        let mut photo = image(source)
            .fit(ImageFit::Cover)
            .style(fill())
            .hidden(failed)
            .on_image_error(move |_| failed.set(true));
```

- Why implicit: (h)/(c) `failed` records "the load already failed" by copying the engine's image status, which already has a reactive owner (`NodeRef::image_status()`, used the same way in lightbox.rs:335-336) (S-4).
- Replacement code (lines 248-254):

```rust
    if let Some(source) = source {
        let photo_ref = node_ref();
        let status = photo_ref.image_status();
        let failed = move || status.get() == kivact::view::ImageStatus::Failed;
        let mut photo = image(source)
            .fit(ImageFit::Cover)
            .node_ref(photo_ref)
            .style(fill())
            .hidden(failed)
            .on_image_error(|_| ());
```

  (`.on_image_error(|_| ())` keeps a handler so the default asset diagnostic stays replaced, exactly as before.)
- Every read/write: 249 (create) and 254 (write) replaced above; 253 read (`.hidden(failed)`) now reads the projection. Delete import `use kivact::state::signal;` (line 12; no other use). `node_ref` is already imported (line 36).
- xtask-allow lines to delete: none.
- Public signature change: none.
- Tests: no test found.

### SM-SOFT-09 — timestamp "live" phase exists only as a pending timer

- Location: crates/kivact-ui-soft/src/timestamp.rs:228-237, 241-244, 351-357, 374, 516-551.
- Current code (verbatim, 516-551):

```rust
#[derive(Clone, PartialEq)]
enum TimestampClock {
    Running { now: TimestampValue },
}
struct TimestampTick {
    value: TimestampValue,
    now: TimestampValue,
}
impl Machine for TimestampTick {
    type State = TimestampClock;
    type Event = TimestampEvent;
    type Effect = ();
    fn transition(
        &self,
        _: &TimestampClock,
        event: TimestampEvent,
    ) -> Step<TimestampClock, TimestampEvent, ()> {
        let age = difference(self.now, self.value).unsigned_abs();
        let delay = match age {
            0..=59 => 1,
            60..=3599 => 30,
            3600..=86399 => 60,
            _ => 300,
        };
        let step = Step::to(TimestampClock::Running { now: self.now });
        match event {
            TimestampEvent::Refresh => step,
            TimestampEvent::Tick => step.after(Duration::from_secs(delay), TimestampEvent::Tick),
        }
    }
}

enum TimestampEvent {
    Refresh,
    Tick,
}
```

- Why implicit: (g) whether the clock is live is expressed only by "a Tick timer happens to be pending"; `Refresh` (sent when `live` or the relative format turns off) does not cancel it, so the pending Tick fires and reschedules forever. The single variant `Running` claims a phase that does not exist (S-6).
- Replacement code. Replace lines 516-551 with:

```rust
#[derive(Clone, PartialEq)]
enum TimestampClock {
    /// Refreshed once at `now`; no tick is scheduled.
    Still { now: TimestampValue },
    /// Refreshed at `now`; the next tick is scheduled.
    Live { now: TimestampValue },
}
impl TimestampClock {
    fn now(&self) -> TimestampValue {
        match self {
            Self::Still { now } | Self::Live { now } => *now,
        }
    }
}
struct TimestampTick {
    value: TimestampValue,
    now: TimestampValue,
}
impl Machine for TimestampTick {
    type State = TimestampClock;
    type Event = TimestampEvent;
    type Effect = ();
    fn transition(
        &self,
        state: &TimestampClock,
        event: TimestampEvent,
    ) -> Step<TimestampClock, TimestampEvent, ()> {
        let age = difference(self.now, self.value).unsigned_abs();
        let delay = match age {
            0..=59 => 1,
            60..=3599 => 30,
            3600..=86399 => 60,
            _ => 300,
        };
        match (state, event) {
            (_, TimestampEvent::Still) => {
                Step::to(TimestampClock::Still { now: self.now }).cancel(TimestampEvent::Tick)
            }
            (_, TimestampEvent::Live) | (TimestampClock::Live { .. }, TimestampEvent::Tick) => {
                Step::to(TimestampClock::Live { now: self.now })
                    .after(Duration::from_secs(delay), TimestampEvent::Tick)
            }
            (TimestampClock::Still { .. }, TimestampEvent::Tick) => Step::stay(),
        }
    }
}

enum TimestampEvent {
    /// Refresh once and stop ticking.
    Still,
    /// Refresh now and keep ticking.
    Live,
    /// The scheduled tick fired.
    Tick,
}
```

- Every read/write and what it becomes:
  - timestamp.rs:229-231 initial state `TimestampClock::Running { now: TimestampValue::from(SystemTime::now()), }` → `TimestampClock::Still { now: TimestampValue::from(SystemTime::now()), }`.
  - timestamp.rs:242 `let TimestampClock::Running { now } = clock.get();` → `let now = clock.get().now();`
  - timestamp.rs:351-357 effect: `clock.send(TimestampEvent::Tick);` → `clock.send(TimestampEvent::Live);` and `clock.send(TimestampEvent::Refresh);` → `clock.send(TimestampEvent::Still);`
  - timestamp.rs:374 `let TimestampClock::Running { now } = clock.get();` → `let now = clock.get().now();`
- xtask-allow lines to delete: none (211, 214 stay).
- Public signature change: none.
- Tests: no test found.

### SM-SOFT-10 — timer starts as "Running" before it is started

- Location: crates/kivact-ui-soft/src/timer.rs:44-59, 171-201.
- Current code (verbatim, 171-174 and 44-48, 56-59):

```rust
#[derive(Clone, PartialEq)]
enum ElapsedTimer {
    Running { start: SystemTime, next: Duration },
}
```
```rust
    let clock = machine(
        ElapsedTimer::Running {
            start: SystemTime::now(),
            next: Duration::ZERO,
        },
```
```rust
    let display = memo(move || {
        let ElapsedTimer::Running { start, next } = clock.get();
        presentation(start, start + next, format.borrow().format.get()).0
    });
```

- Why implicit: (g) the machine is created with a placeholder "running since construction" state that is only made true by the `ElapsedEvent::Start` sent later in `into_view` (timer.rs:132-134); the unstarted phase exists only by call order (S-6).
- Replacement code. Replace lines 171-174 with:

```rust
#[derive(Clone, PartialEq)]
enum ElapsedTimer {
    /// Built but not mounted; `into_view` sends the first `Start`.
    Unstarted,
    Running { start: SystemTime, next: Duration },
}
```

  Replace lines 45-48 (the initial-state argument) with `ElapsedTimer::Unstarted,`.

  Replace lines 56-59 with:

```rust
    let display = memo(move || match clock.get() {
        ElapsedTimer::Unstarted => String::new(),
        ElapsedTimer::Running { start, next } => {
            presentation(start, start + next, format.borrow().format.get()).0
        }
    });
```

  Replace lines 192-196 in `transition` (from `let ElapsedTimer::Running { start, .. } = *state;` through the closing `};` of `let start = match event { ... };`) with:

```rust
        let start = match (state, event) {
            (_, ElapsedEvent::Start(start)) => start,
            (ElapsedTimer::Running { start, .. }, ElapsedEvent::Tick) => *start,
            (ElapsedTimer::Unstarted, ElapsedEvent::Tick) => return Step::stay(),
        };
```

- Every read/write: initial state (45-48), display (57), transition (192) as above; no other readers of `ElapsedTimer`.
- xtask-allow lines to delete: none.
- Behaviour: identical once mounted (`into_view` still sends `Start` synchronously).
- Public signature change: none.
- Tests: no test found (toast tests' "timer" is the toast auto-dismiss timer, not `soft::timer`).

## 5. Borderline decisions (kept, recorded for the owner)

- soft_chat_tools.rs:63-83 `ToolCall { status: ToolStatus, duration: Option<Duration>, error: Option<String> }` and chat_dictation.rs:18-29 `DictationState { supported, listening, volume, bands }`: shape (b)/(a), but both are public application-owned report records; a fix changes public types (`ToolStatus` would lose `Copy`). Kept as passive records.
- timer.rs:34/39 and timestamp.rs:210/220 `Rc<RefCell<…Settings>>`, icon.rs:79/85 `Rc<Cell<(IconSize, IconColor)>>`: builder storage read by closures created in the constructor (needed because `StyledText`/the style layer must exist at builder time). Removing them needs a wrapper node or a mirror of the inputs in machine state; kept as builder options.
- side_navigation/shell.rs:123-124 `SideNav { collapse: Option<SideNavCollapse>, resize: Option<SideNavResize> }`: two public builder options can both carry collapse ownership (resize wins); fixing needs a public builder change.
- bottom_sheet.rs:56-60 `Sheet::Settling { to }` never transitions to `Resting`; it is explicit and behaves like `Resting`, not implicit.
- rich_blocks.rs:138 and 346 `self.state.send(BlockEvent::Source)` before reading the document is a re-sync by call order (g-like); left because it guards against a feed that has not yet run in the same turn.
- The 12 `xtask-allow(state-fields)` comments on function parameters (lightbox.rs 330, 401; outline.rs 489, 493, 495; thumbnail.rs 227, 229, 231, 337, 340, 388, 392) are inert: dev/xtask/src/state_fields.rs only inspects top-level structs.

---

# Implicit state machines: kivact-interaction, kivact-ui-creative, kivact-style

Tree: W:\w\kivact @ feat/kashiwade/devtools-multi-window (= main 96e22b6b). Read-only review; nothing built or run.
Paths below are relative to `W:\w\kivact\crates\`. Line numbers are those of 96e22b6b.

## 1. Summary counts

| crate | `xtask-allow(state-fields)` lines reviewed | KEEP | FIX | UNANNOTATED-FIX |
|---|---|---|---|---|
| kivact-interaction | 53 | 50 | 3 (SM-INT-01, 02, 03) | 2 (SM-INT-07, 08) + 6 (SM-INT-09, 10, 11, 12, 15, 16) |
| kivact-ui-creative | 69 | 67 | 2 sites / 3 entries (SM-INT-04, 05, 06) | 0 + 2 (SM-INT-13, 14) |
| kivact-style | 16 | 16 | 0 | 0 |

SM-INT-09..15 (section 6) are the coordinator-decided fixes for the former deferred items D1–D7. Three of them reach outside the three crates in scope:

- SM-INT-09 adds `kivact_core::view::is_mounted` (re-exported by the facade as `kivact::view::is_mounted`).
- SM-INT-14 adds the public `EditSession::replace_initial`.
- SM-INT-11 changes the crate-private `delayed`/`hover_surface` signatures.

Suggested apply order: SM-INT-09 before SM-INT-10 (both edit menu/level.rs imports), and SM-INT-11 as one change across all of its files.

How xtask counts (dev/xtask/src/state_fields.rs): a struct is flagged when >= 2 non-annotated fields contain `Option`, `bool`, or `Cell|Signal|Var` + `u64|u32|usize`, or an `Option` sits beside a `*State`/`*Phase` enum. Structs named `*Api`, role suffixes (`Options`, `Event`, `Phase`, `Reason`, `Input`, `Report`, `Tokens`, `Recipes`) and enums are never checked. Each FIX below states whether the annotation can be deleted under that rule.

## 2. KEEP table

### kivact-interaction (50)

| site | struct.field | category |
|---|---|---|
| kivact-interaction/src/calendar/data.rs:20 | CalendarDayState.* | passive record (public drawing snapshot; changing it would break public API) |
| kivact-interaction/src/calendar/state.rs:26 | State.grid_state | independent fact (observation of the last attached grid part) |
| kivact-interaction/src/calendar/state.rs:42 | State.active | named S-9 type (pure cache: memo projection) |
| kivact-interaction/src/dialog.rs:60 | DismissOn.* | builder option |
| kivact-interaction/src/list/data.rs:306 | ListItemState.active | passive record |
| kivact-interaction/src/list/state.rs:38 | State.container_state | independent fact (observation of the attached container part) |
| kivact-interaction/src/list/state.rs:48 | State.anchor_value | named S-9 type (pure cache: memo of caller anchor binding) |
| kivact-interaction/src/list/state.rs:52 | State.active | named S-9 type (pure cache: memo projection) |
| kivact-interaction/src/menu/level.rs:23 | Level.content_state | independent fact (observation of the attached surface part) |
| kivact-interaction/src/menu/level.rs:30 | Level.shown | named S-9 type (pure cache: projection of session path) |
| kivact-interaction/src/menu/level.rs:35 | Level.aiming | named S-9 type (pure cache: projection of corridor machine) |
| kivact-interaction/src/menu/level.rs:38 | Level.retain_disabled_focus | builder option (theme policy captured at setup) |
| kivact-interaction/src/menu/session.rs:19 | Session.opened | named S-9 type (pure cache: memo of machine state) |
| kivact-interaction/src/menu/session.rs:22 | Session.origin | named S-9 type (pure cache: memo of `opened`) |
| kivact-interaction/src/menu/session.rs:27 | Session.focus_key | named S-9 type (pure cache: memo of machine state) |
| kivact-interaction/src/menu/session.rs:32 | Session.want | named S-9 type (pure cache: memo of machine state) |
| kivact-interaction/src/menu/session.rs:36 | Session.press | builder option (theme opt-in created at setup) |
| kivact-interaction/src/menu/state.rs:54 | MenuItemState.pressed_over | passive record |
| kivact-interaction/src/menu/state.rs:59 | MenuItemState.check | passive record |
| kivact-interaction/src/menu/state.rs:62 | MenuItemState.expanded | passive record |
| kivact-interaction/src/menu/state.rs:65 | MenuItemState.destructive | passive record |
| kivact-interaction/src/menu/state.rs:149 | ItemEntry.disabled | builder option |
| kivact-interaction/src/menu/state.rs:152 | ItemEntry.disabled_reason | builder option |
| kivact-interaction/src/menu/state.rs:154 | ItemEntry.icon | builder option |
| kivact-interaction/src/menu/state.rs:157 | ItemEntry.description | builder option |
| kivact-interaction/src/menu/state.rs:160 | ItemEntry.destructive | builder option |
| kivact-interaction/src/menu/state.rs:163 | ItemEntry.keep_open | builder option |
| kivact-interaction/src/navigation_menu.rs:197 | NavigationMenuApi.next_panel | independent fact (S-8 identity allocation) |
| kivact-interaction/src/popup/machine.rs:80 | Machine.focused | passive record (Machine input snapshot) |
| kivact-interaction/src/popup/machine.rs:82 | Machine.focus_inside | passive record (Machine input snapshot) |
| kivact-interaction/src/roving/registry.rs:29 | Member.disabled | caller binding |
| kivact-interaction/src/roving/registry.rs:31 | Member.hidden | caller binding |
| kivact-interaction/src/scalar/edit_state.rs:28 | Shared.host_disabled | independent fact (per-mount registry, removed with each mount) |
| kivact-interaction/src/scalar/edit_state.rs:154 | Edit.thumb_rect | named S-9 type (pure cache: layout projection) |
| kivact-interaction/src/scalar/edit_state.rs:156 | Edit.bounds | builder option |
| kivact-interaction/src/scalar/track_data.rs:14 | Target.rect | named S-9 type (pure cache: layout projection) |
| kivact-interaction/src/scalar/track_data.rs:16 | Target.bounds | builder option |
| kivact-interaction/src/scalar/track_data.rs:81 | Geometry.rect | named S-9 type (pure cache: layout projection) |
| kivact-interaction/src/table/data.rs:8 | Header.* | passive record (derived column projection) |
| kivact-interaction/src/table/data.rs:79 | TableHeaderState.* | passive record (public drawing snapshot) |
| kivact-interaction/src/table/data.rs:91 | TableRowState.* | passive record (public drawing snapshot) |
| kivact-interaction/src/table/data.rs:102 | TableCellState.* | passive record (public drawing snapshot) |
| kivact-interaction/src/table/model.rs:53 | ColumnDef.* | builder option |
| kivact-interaction/src/table/state.rs:36 | State.grid_state | independent fact (observation of the attached grid part) |
| kivact-interaction/src/table/state.rs:59 | State.tab_stop | named S-9 type (pure cache: memo projection) |
| kivact-interaction/src/table/state.rs:62 | State.active | named S-9 type (pure cache: memo projection) |
| kivact-interaction/src/toaster/data.rs:91 | Toast.* | passive record |
| kivact-interaction/src/toggle.rs:113 | Core.checked | caller binding |
| kivact-interaction/src/toggle.rs:116 | Core.edit | independent fact (EditSession machine handle) |
| kivact-interaction/src/tree.rs:332 | TreeItemState.active | passive record |

### kivact-ui-creative (67)

| site | struct.field | category |
|---|---|---|
| kivact-ui-creative/src/audio/meter.rs:38 | Meter.horizontal | builder option |
| kivact-ui-creative/src/audio/meter.rs:41 | Meter.segmented | builder option |
| kivact-ui-creative/src/audio/transport.rs:12 | TransportState.* | caller binding (application-owned value) |
| kivact-ui-creative/src/audio/view_state.rs:4 | ChannelState.* | caller binding |
| kivact-ui-creative/src/dock/data.rs:54 | DockCx.* | caller binding (Copy handles; see Deferred D5 for `drag`) |
| kivact-ui-creative/src/dock/model.rs:47 | DockTab.* | passive record |
| kivact-ui-creative/src/dock/model.rs:86 | DockTabs.* | passive record |
| kivact-ui-creative/src/dock/view.rs:29 | Dock.* | builder option |
| kivact-ui-creative/src/editors/arrangement/gesture.rs:48 | State.on_edit | builder option |
| kivact-ui-creative/src/editors/arrangement/gesture.rs:63 | State.cycle | builder option |
| kivact-ui-creative/src/editors/arrangement/gesture.rs:65 | State.cycling | caller binding |
| kivact-ui-creative/src/editors/arrangement/gesture.rs:73 | State.snap | builder option |
| kivact-ui-creative/src/editors/arrangement/gesture.rs:76 | State.active_auto | independent fact (keyboard target point) |
| kivact-ui-creative/src/editors/arrangement/model.rs:58 | TimelineTrack.muted | passive record |
| kivact-ui-creative/src/editors/arrangement/model.rs:60 | TimelineTrack.solo | passive record |
| kivact-ui-creative/src/editors/arrangement/model.rs:62 | TimelineTrack.armed | passive record |
| kivact-ui-creative/src/editors/arrangement/model.rs:64 | TimelineTrack.automation_read | passive record |
| kivact-ui-creative/src/editors/graph/gesture.rs:76 | State.on_edit | builder option |
| kivact-ui-creative/src/editors/graph/gesture.rs:85 | State.snap | builder option |
| kivact-ui-creative/src/editors/graph/gesture.rs:88 | State.press | independent fact (CanvasGesture machine handle) |
| kivact-ui-creative/src/editors/graph/gesture.rs:95 | State.active | independent fact (keyboard target) |
| kivact-ui-creative/src/editors/graph/gesture.rs:99 | State.lane_activate | builder option |
| kivact-ui-creative/src/editors/graph/gesture.rs:101 | State.item_activate | builder option |
| kivact-ui-creative/src/editors/graph/gesture.rs:103 | State.child_focus | named S-9 type (pure cache: focus projection) |
| kivact-ui-creative/src/editors/node/gesture.rs:58 | Info.checked | passive record |
| kivact-ui-creative/src/editors/node/gesture.rs:61 | Info.range | passive record |
| kivact-ui-creative/src/editors/node/gesture.rs:216 | State.on_edit | builder option |
| kivact-ui-creative/src/editors/node/gesture.rs:226 | State.selected_link | caller binding |
| kivact-ui-creative/src/editors/node/gesture.rs:231 | State.snap | builder option |
| kivact-ui-creative/src/editors/node/gesture.rs:233 | State.active | independent fact (keyboard target control) |
| kivact-ui-creative/src/editors/node/gesture.rs:246 | State.blocked | named S-9 type (pure cache: projection) |
| kivact-ui-creative/src/editors/node/model.rs:46 | NodeData.preview | passive record |
| kivact-ui-creative/src/editors/node/model.rs:49 | NodeData.muted | passive record |
| kivact-ui-creative/src/editors/shared/lane.rs:15 | LaneExtensions.lane_activate | builder option |
| kivact-ui-creative/src/editors/shared/lane.rs:17 | LaneExtensions.item_activate | builder option |
| kivact-ui-creative/src/editors/shared/lane.rs:19 | LaneExtensions.menu | builder option |
| kivact-ui-creative/src/editors/shared/model.rs:21 | EditorContext.item | passive record (public snapshot) |
| kivact-ui-creative/src/editors/shared/model.rs:23 | EditorContext.lane | passive record (public snapshot) |
| kivact-ui-creative/src/editors/shared/region_parts.rs:25 | RegionInfo.checked | passive record |
| kivact-ui-creative/src/editors/shared/region_parts.rs:27 | RegionInfo.selected | passive record |
| kivact-ui-creative/src/editors/time/gesture.rs:32 | State.on_edit | builder option |
| kivact-ui-creative/src/editors/time/gesture.rs:47 | State.press | independent fact (CanvasGesture machine handle) |
| kivact-ui-creative/src/editors/time/gesture.rs:53 | State.snap | builder option |
| kivact-ui-creative/src/editors/time/gesture.rs:59 | State.active | independent fact (keyboard target) |
| kivact-ui-creative/src/editors/time/gesture.rs:61 | State.child_focus | named S-9 type (pure cache: focus projection) |
| kivact-ui-creative/src/editors/time/gesture.rs:64 | State.lane_activate | builder option |
| kivact-ui-creative/src/editors/time/gesture.rs:66 | State.item_activate | builder option |
| kivact-ui-creative/src/editors/time/view.rs:28 | TimeEditor.lanes | builder option |
| kivact-ui-creative/src/editors/time/view.rs:30 | TimeEditor.snap | builder option |
| kivact-ui-creative/src/editors/time/view.rs:35 | TimeEditor.on_edit | builder option |
| kivact-ui-creative/src/numeric/implementation.rs:433 | NumericView.* | builder option (inputs and machine handles) |
| kivact-ui-creative/src/outliner/view.rs:31 | Outliner.* | builder option |
| kivact-ui-creative/src/outliner_data.rs:6 | TreeItem.* | passive record |
| kivact-ui-creative/src/outliner_data.rs:58 | TreeDrop.* | passive record |
| kivact-ui-creative/src/picker.rs:60 | PickerItem.section | passive record |
| kivact-ui-creative/src/picker.rs:64 | PickerItem.description | passive record |
| kivact-ui-creative/src/picker.rs:67 | PickerItem.shortcut | passive record |
| kivact-ui-creative/src/picker.rs:70 | PickerItem.icon | passive record |
| kivact-ui-creative/src/picker.rs:73 | PickerItem.enabled | passive record |
| kivact-ui-creative/src/picker.rs:186 | Parts.pick | builder option |
| kivact-ui-creative/src/picker.rs:188 | Parts.preview | builder option |
| kivact-ui-creative/src/picker.rs:190 | Parts.cancel | builder option |
| kivact-ui-creative/src/picker.rs:512 | Outcomes.pick | builder option |
| kivact-ui-creative/src/picker.rs:514 | Outcomes.preview | builder option |
| kivact-ui-creative/src/picker.rs:516 | Outcomes.cancel | builder option |
| kivact-ui-creative/src/picker.rs:576 | Session.column | builder option |
| kivact-ui-creative/src/property_row.rs:17 | PropertyRow.* | builder option |

### kivact-style (16)

| site | struct.field | category |
|---|---|---|
| kivact-style/src/draw/layer.rs:11 | Layer.* | builder option |
| kivact-style/src/draw/text_run.rs:17 | TextFormat.* | builder option |
| kivact-style/src/grid.rs:260 | GridStyle.* | passive record |
| kivact-style/src/paint/border_image.rs:44 | BorderImage.* | passive record |
| kivact-style/src/paint/gradient.rs:41 | Gradient.* | passive record |
| kivact-style/src/paint/layer.rs:13 | Paint.* | passive record |
| kivact-style/src/resolve.rs:47 | CompiledStyle.* | passive record (compilation result) |
| kivact-style/src/resolve.rs:209 | Interactive.* | passive record (compile input) |
| kivact-style/src/shader/data.rs:13 | ShaderData.* | passive record |
| kivact-style/src/sheet.rs:154 | Sides.* | passive record |
| kivact-style/src/sheet.rs:172 | Style.* | passive record |
| kivact-style/src/sheet.rs:1245 | ScrollbarStyle.* | passive record |
| kivact-style/src/text.rs:258 | TextExtras.* | passive record |
| kivact-style/src/text.rs:443 | SpanStyle.* | passive record |
| kivact-style/src/text.rs:532 | StyledRange.* | passive record |
| kivact-style/src/text.rs:542 | TextDecoration.* | passive record |

kivact-style unannotated scan: the only mutable statics are `NEXT_GRAPH: AtomicU64` (effect/filter_graph.rs:24, S-8 ID allocation) and `THEMED: Mutex<Vec<..>>` (theme/value.rs:573, interning cache). No Cell/RefCell/Signal state. Nothing to fix.

## 3. FIX entries

### SM-INT-01 — toggle `Core.host_disabled` is a slot filled at attach

- Site: `kivact-interaction/src/toggle.rs:111-122` (struct), 153-162 (`core`), 335-363 (`is_disabled`, `press`), 379-399 (`control`).
- Current fields (verbatim):

```rust
struct Core {
    // xtask-allow(state-fields): The checked binding is caller-owned input, independent of mount observation.
    checked: Binding<bool>,
    kind: Kind,
    // xtask-allow(state-fields): EditSession already owns the edit lifecycle machine; bool is its payload type.
    edit: crate::EditSession<bool>,
    observed: Signal<Option<Value<ToggleState>>>,
    options: OptionsCell<ToggleOptions>,
    // xtask-allow(state-fields): Host disabled is a mounted node input, not a lifecycle phase.
    host_disabled: Signal<Option<Value<bool>>>,
}
```

- Why implicit: (f). `host_disabled` starts `None` and is written inside the attach closure (`self.host_disabled.set(caller_disabled)`, line 383), so press handling reads a slot that is filled after construction. It also stores a value that is already in scope at the only places that read it, which is (h). With two parts made from one Core, the last attach overwrites it, so a press on the first part reads the second host's disabled state.
- Replacement: delete the field. Pass the attach-time value into `press`.

```rust
struct Core {
    // xtask-allow(state-fields): The checked binding is caller-owned input, independent of mount observation.
    checked: Binding<bool>,
    kind: Kind,
    // xtask-allow(state-fields): EditSession already owns the edit lifecycle machine; bool is its payload type.
    edit: crate::EditSession<bool>,
    observed: Signal<Option<Value<ToggleState>>>,
    options: OptionsCell<ToggleOptions>,
}

fn core(checked: Binding<bool>, kind: Kind) -> Core {
    Core {
        checked,
        kind,
        edit: crate::EditSession::new(move |value| checked.set(value)),
        observed: signal(None),
        options: OptionsCell::new(ToggleOptions::default()),
    }
}

    // in impl Core
    fn is_disabled(self, caller_disabled: Option<Value<bool>>) -> bool {
        self.options
            .get()
            .disabled
            .is_some_and(|value| value.get_untracked())
            || caller_disabled.is_some_and(|value| value.get_untracked())
    }

    fn press(self, caller_disabled: Option<Value<bool>>) {
        if !self.is_disabled(caller_disabled) {
            let mixed = self.is_mixed();
            self.edit.once(self.checked.get_untracked(), || {
                self.checked.set(mixed || !self.checked.get())
            });
        }
    }
```

- Every read and write of the old field:
  - toggle.rs:120-121: delete the annotation line and the field line.
  - toggle.rs:160 `host_disabled: signal(None),`: delete.
  - toggle.rs:340-342, reading `self.host_disabled.with_untracked(..)` in `is_disabled`: replace with `caller_disabled.is_some_and(|value| value.get_untracked())`, as in the code above.
  - toggle.rs:383 `self.host_disabled.set(caller_disabled);`: delete. `let caller_disabled = host.accessible_disabled();` on line 382 stays.
  - toggle.rs:393 `.on_press(move || self.press())`: change to `.on_press(move || self.press(caller_disabled))`.
  - toggle.rs:395 `.a11y_action(A11yAction::Toggle, move |_| self.press())`: change to `.a11y_action(A11yAction::Toggle, move |_| self.press(caller_disabled))`.
  - `Option<Value<bool>>` is `Copy`, so each closure takes its own copy. The `.disabled(move || caller_disabled...)` closure is unchanged. The imports `Signal` and `signal` are still used by `observed`.
- xtask-allow lines to delete: toggle.rs:120. After the fix, Core still has only one counted field (`observed`), so no new annotation is needed.
- Public signatures: unchanged.
- Tests: `dev/tests/kivact-public-api/tests/e2e/interaction_toggle.rs` covers roles, Space and Enter. No test presses a toggle whose host is disabled.

### SM-INT-02 — menu `SessionLevel`: per-level highlight Options and a submenu key stored twice

- Site: `kivact-interaction/src/menu/machine.rs:1-247` (the whole file).
- Current fields (verbatim):

```rust
#[derive(Clone, PartialEq)]
pub(super) enum State<K> {
    Closed,
    Open {
        opened_by: Opened,
        levels: Vec<SessionLevel<K>>,
        want: Option<Want<K>>,
        typeahead: Typeahead,
    },
}
#[derive(Clone, PartialEq)]
pub(super) struct SessionLevel<K> {
    // xtask-allow(state-fields): the key of the item that opened this level identifies the level, independent of its highlight.
    parent: Option<K>,
    highlight: Option<K>,
    submenu: Submenu<K>,
}
#[derive(Clone, PartialEq)]
pub(super) enum Submenu<K> {
    None,
    Opening { key: K },
    Open { key: K },
    Closing { key: K, by: K },
}
```

- Why implicit:
  - (a) Every level has a `highlight: Option<K>`, but at most one may be `Some`. `Event::Focus` clears all of them and sets one, and the kept-levels repair clears all of them.
  - (a)/(b) `parent` is `None` only for index 0 and `Some` for every other index. Its validity depends on the position in the Vec.
  - (h) The open child's key is stored twice: in `levels[d].submenu` (`Open{key}` / `Closing{key, ..}`) and in `levels[d + 1].parent`. `Rest` overwrites `Open{B}` with `Opening{A}` while child B stays open, so the two copies disagree.
- Replacement: the whole new file `kivact-interaction/src/menu/machine.rs` follows. `Event`, `Effect`, `Machine`, `accepted_opening` and the timer tail of `transition` are unchanged.

```rust
//! Role: Pure menu session transitions own levels, focus requests and submenu timers.
use super::order::{item_enabled, kept_levels};
use super::state::{Edge, MenuEntry, Opened, Target, Want};
use crate::list::typeahead::Typeahead;

/// The complete state of one menu, shared by every rendered level.
#[derive(Clone, PartialEq)]
pub(super) enum State<K> {
    Closed,
    Open {
        opened_by: Opened,
        levels: Levels<K>,
        want: Option<Want<K>>,
        typeahead: Typeahead,
    },
}
/// The open levels of one menu and its one highlighted item. Level `d + 1`
/// is the submenu of the item `submenus[d].key` of level `d`; every level
/// has its own submenu timer.
#[derive(Clone, PartialEq)]
pub(super) struct Levels<K> {
    /// The submenu timer of the top level.
    top: SubmenuTimer<K>,
    /// The open submenus, outermost first.
    submenus: Vec<OpenSubmenu<K>>,
    /// The highlighted item of all levels: its depth (an open level) and key.
    focus: Option<(usize, K)>,
}
/// An open submenu: the key of the item that opened it and the submenu
/// timer of its own level.
#[derive(Clone, PartialEq)]
struct OpenSubmenu<K> {
    key: K,
    timer: SubmenuTimer<K>,
}
/// The pending submenu timer of one level. The level's open submenu is
/// `Levels::child`; the timer does not store it again.
#[derive(Clone, PartialEq)]
enum SubmenuTimer<K> {
    Idle,
    /// The submenu of `key` opens after the delay because the pointer rests on it.
    Opening { key: K },
    /// The open submenu closes after the delay because the pointer rests
    /// on the plain item `by`; only leaving `by` keeps it open.
    Closing { by: K },
}
impl<K: Clone> State<K> {
    pub(super) fn path(&self) -> Vec<K> {
        match self {
            Self::Closed => Vec::new(),
            Self::Open { levels, .. } => levels.path(),
        }
    }
    pub(super) fn focus(&self) -> Option<(usize, K)> {
        match self {
            Self::Closed => None,
            Self::Open { levels, .. } => levels.focus.clone(),
        }
    }
}
impl<K: Clone> Levels<K> {
    fn new() -> Self {
        Self {
            top: SubmenuTimer::Idle,
            submenus: Vec::new(),
            focus: None,
        }
    }
    /// How many levels are open, the top level included.
    fn len(&self) -> usize {
        self.submenus.len() + 1
    }
    /// The keys of the submenu items that are open, outermost first.
    fn path(&self) -> Vec<K> {
        self.submenus
            .iter()
            .map(|submenu| submenu.key.clone())
            .collect()
    }
    /// The key of the item of level `depth` whose submenu is open.
    fn child(&self, depth: usize) -> Option<&K> {
        self.submenus.get(depth).map(|submenu| &submenu.key)
    }
    fn timer(&self, depth: usize) -> Option<&SubmenuTimer<K>> {
        match depth {
            0 => Some(&self.top),
            _ => self.submenus.get(depth - 1).map(|submenu| &submenu.timer),
        }
    }
    fn timer_mut(&mut self, depth: usize) -> Option<&mut SubmenuTimer<K>> {
        match depth {
            0 => Some(&mut self.top),
            _ => self
                .submenus
                .get_mut(depth - 1)
                .map(|submenu| &mut submenu.timer),
        }
    }
    /// Highlight `key` of the level at `depth`, or nothing.
    fn set_focus(&mut self, depth: usize, key: Option<K>) {
        self.focus = if depth < self.len() {
            key.map(|key| (depth, key))
        } else {
            None
        };
    }
    /// Keep the levels up to `depth` and stop the submenu timer of level `depth`.
    fn truncate(&mut self, depth: usize) {
        self.submenus.truncate(depth);
        if self.focus.as_ref().is_some_and(|(at, _)| *at > depth) {
            self.focus = None;
        }
        if let Some(timer) = self.timer_mut(depth) {
            *timer = SubmenuTimer::Idle;
        }
    }
}
impl<K: Clone + PartialEq> Levels<K> {
    /// Open the submenu of `key`, an item of level `depth`, as the deepest level.
    fn open(&mut self, depth: usize, key: K) {
        if depth >= self.len() {
            return;
        }
        if self.len() == depth + 2 && self.child(depth) == Some(&key) {
            return;
        }
        self.truncate(depth);
        self.submenus.push(OpenSubmenu {
            key,
            timer: SubmenuTimer::Idle,
        });
    }
    /// The pointer rests on (`over`) or leaves the item `key` of level
    /// `depth`; `opens` when it is a submenu item. Returns the timer to
    /// start: whether it opens, the depth and the key it names.
    fn rest(&mut self, depth: usize, key: K, over: bool, opens: bool) -> Option<(bool, usize, K)> {
        let child = self.child(depth).cloned();
        let timer = self.timer_mut(depth)?;
        if opens {
            let shown = matches!(timer, SubmenuTimer::Idle) && child.as_ref() == Some(&key);
            if over && !shown {
                *timer = SubmenuTimer::Opening { key: key.clone() };
                return Some((true, depth, key));
            }
            if !over && matches!(timer, SubmenuTimer::Opening { key: current } if *current == key) {
                *timer = SubmenuTimer::Idle;
            }
        } else if over && let Some(child) = child {
            *timer = SubmenuTimer::Closing { by: key };
            return Some((false, depth, child));
        } else if !over && matches!(timer, SubmenuTimer::Closing { by } if *by == key) {
            *timer = SubmenuTimer::Idle;
        }
        None
    }
}
pub(super) enum Event<K> {
    Open(Opened),
    Popup(bool),
    Entries,
    Request(Option<Want<K>>),
    Hover(usize, K),
    Focus(usize, Option<K>),
    OpenSubmenu(usize, K),
    Truncate(usize),
    Typeahead(Typeahead),
    Rest(usize, K, bool, bool),
    OpenAfter(usize, K),
    CloseAfter(usize, K),
}
pub(super) enum Effect {
    Close,
    Closed,
}
pub(super) struct Machine<K: kivact_core::state::ItemKey> {
    pub(super) entries: Vec<MenuEntry<K>>,
    pub(super) timing: crate::menu_policy::SubmenuTiming,
}
impl<K: kivact_core::state::ItemKey> kivact_core::state::Machine for Machine<K> {
    type State = State<K>;
    type Event = Event<K>;
    type Effect = Effect;
    fn transition(
        &self,
        state: &Self::State,
        event: Self::Event,
    ) -> kivact_core::state::Step<Self::State, Self::Event, Self::Effect> {
        use kivact_core::state::Step;
        let mut next = state.clone();
        if let Event::Popup(false) = event {
            return Step::to(State::Closed).with(Effect::Closed);
        }
        let event = accepted_opening(state, event);
        if let Event::Open(opened_by) = event {
            match &mut next {
                State::Closed => {
                    next = State::Open {
                        opened_by,
                        levels: Levels::new(),
                        want: None,
                        typeahead: Typeahead::default(),
                    }
                }
                State::Open {
                    opened_by: opened, ..
                } => *opened = opened_by,
            }
        }
        let State::Open {
            levels,
            want,
            typeahead,
            ..
        } = &mut next
        else {
            return Step::stay();
        };
        let timer = update_level_event(levels, want, typeahead, event);
        // All topology repairs happen in this transition, before publishing state.
        let path = levels.path();
        let kept = kept_levels(&self.entries, &path, levels.focus.as_ref());
        if kept == 0 {
            return Step::to(next).with(Effect::Close);
        }
        if kept < levels.len() {
            let parent = path
                .get(kept - 1)
                .cloned()
                .filter(|key| item_enabled(&self.entries, &path[..kept - 1], key));
            levels.truncate(kept - 1);
            levels.focus = None;
            *want = Some(Want::Command {
                depth: kept - 1,
                target: parent.map_or(Target::Edge(Edge::First), Target::Key),
            });
        }
        if want
            .as_ref()
            .is_some_and(|want| want.depth() >= levels.len())
        {
            *want = None;
        }
        let mut step = Step::to(next);
        if let Some((opens, depth, key)) = timer {
            let timing = self.timing;
            step = if opens {
                step.after(timing.open, Event::OpenAfter(depth, key))
            } else {
                step.after(timing.close, Event::CloseAfter(depth, key))
            };
        }
        step
    }
}

fn update_level_event<K: kivact_core::state::ItemKey>(
    levels: &mut Levels<K>,
    want: &mut Option<Want<K>>,
    typeahead: &mut Typeahead,
    event: Event<K>,
) -> Option<(bool, usize, K)> {
    match event {
        Event::Request(request) => *want = request,
        Event::Hover(depth, key) if !matches!(want, Some(Want::Command { .. })) => {
            *want = Some(Want::Hover { depth, key })
        }
        Event::Focus(depth, key) => levels.set_focus(depth, key),
        Event::OpenSubmenu(depth, key) => levels.open(depth, key),
        Event::Truncate(depth) => levels.truncate(depth),
        Event::Typeahead(typed) => *typeahead = typed,
        Event::Rest(depth, key, over, opens) => return levels.rest(depth, key, over, opens),
        Event::OpenAfter(depth, key) => {
            if matches!(levels.timer(depth), Some(SubmenuTimer::Opening { key: current }) if *current == key)
            {
                levels.open(depth, key);
            }
        }
        Event::CloseAfter(depth, key)
            if matches!(levels.timer(depth), Some(SubmenuTimer::Closing { .. }))
                && levels.child(depth) == Some(&key) =>
        {
            levels.truncate(depth);
        }
        _ => {}
    }
    None
}

/// Interpret an accepted external open as a menu opening without a focus request.
fn accepted_opening<K>(state: &State<K>, event: Event<K>) -> Event<K> {
    if matches!(event, Event::Popup(true)) && matches!(state, State::Closed) {
        Event::Open(Opened::External)
    } else {
        event
    }
}
```

- Mapping from old to new (every read and write of `SessionLevel`/`Submenu`; all of them are in this file):
  - `levels: vec![SessionLevel{parent: None, highlight: None, submenu: Submenu::None}]` (old 103-107) becomes `Levels::new()`.
  - `State::path()` (old 41-50) becomes `levels.path()`. `State::focus()` (old 51-59) becomes `levels.focus.clone()`. Their signatures are unchanged, so callers in session.rs:88-89 and 110 are untouched.
  - The `path`/`focus` locals in `transition` (old 128-136) become `levels.path()` and `levels.focus.as_ref()`.
  - The kept repair (old 146-149: `truncate(..)` plus clearing every `highlight`) becomes `levels.truncate(kept - 1); levels.focus = None;`.
  - `want.depth() >= levels.len()` keeps the same name: `Levels::len`.
  - `open_level` (old 173-187) becomes `Levels::open`. `truncate` (old 188-193) becomes `Levels::truncate`.
  - `Event::Focus` (old 205-208) becomes `levels.set_focus(depth, key)`.
  - `Event::Rest` (old 212-229) becomes `levels.rest(..)`. Old `Submenu::Open{key}` maps to (`SubmenuTimer::Idle` with `child(depth) == Some(key)`). Old `Closing{key: child, by}` maps to `Closing{by}` with the child read from `child(depth)`.
  - `Event::OpenAfter` / `Event::CloseAfter` (old 230-234) become `timer(depth)` / `child(depth)` checks.
  - Outside this file, only `State::Closed`, `State::Open { want | opened_by | typeahead, .. }`, `State::path` and `State::focus` are used (session.rs:68, 88-112, 258-266). No change is needed there.
- Behaviour change (intended; caused by removing the duplicate):
  - Old sequence: submenu B is open, the pointer rests on submenu item A (`Opening{A}`), then leaves A before the delay. This reset the level to `Submenu::None` while B was still open, and resting on B again restarted an open timer. If B had its own open submenu, `OpenAfter` then collapsed it.
  - Now the level is `Idle` with child B, so resting on B does nothing.
  - Every other sequence is unchanged, including `Closing` followed by a rest on the open item (that still restarts the timer, as before).
- xtask-allow lines to delete: menu/machine.rs:19. `Levels` has one counted field (`focus`); `OpenSubmenu` and `SubmenuTimer` have none, so no annotation is needed.
- Public signatures: unchanged (everything is `pub(super)` or private).
- Tests:
  - `dev/tests/kivact-public-api/tests/e2e/interaction_menu.rs`: `right_and_left_move_between_levels_and_escape_closes_one_level_at_a_time`, `choosing_calls_back_with_the_key_closes_every_level_and_returns_the_focus`, and `roles_checked_expanded_description_and_the_separator_are_published` (hovers and clicks the "More" submenu).
  - `dev/tests/kivact-public-api/tests/e2e/apg_menu.rs::menu_submenu_parent_publishes_menu_popup`.
  - `dev/tests/kivact-public-api/tests/e2e/ui_components.rs::menu_renders_nested_levels_and_custom_trigger_keeps_its_style`.
  - No test drives the `Rest` open/close timers. The behaviour change above is untested.

### SM-INT-03 — list `Origin { token, node }`: two coupled Options

- Site: `kivact-interaction/src/list/machine.rs:7-30`. Constructor: `kivact-interaction/src/list/state.rs:226-240`.
- Current fields (verbatim):

```rust
#[derive(Clone, Copy)]
pub(super) struct Origin {
    pub(super) token: Option<NodeToken>,
    // xtask-allow(state-fields): A snapshot reference locates the origin token through virtualization.
    pub(super) node: Option<NodeRef>,
}
```

- Why implicit: (a). `node` is `Some` only when `token` is `Some` (state.rs:231 finds a member only `if origin.is_some()`), so `(None, Some)` cannot happen. `kept()` re-derives this case split at every call (`self.token.is_some() && self.node.is_some_and(..)`).
- Replacement (list/machine.rs lines 7-30):

```rust
/// Where the focus was when a virtual focus request was made.
#[derive(Clone, Copy)]
pub(super) enum Origin {
    /// No node had the focus.
    Nowhere,
    /// A node that is not a member of the list had the focus.
    Outside(NodeToken),
    /// A member had the focus; its reference finds it again when
    /// virtualization unmounts it.
    Member { token: NodeToken, node: NodeRef },
}
impl PartialEq for Origin {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nowhere, Self::Nowhere) => true,
            (Self::Outside(left), Self::Outside(right)) => left == right,
            (
                Self::Member { token: left, node: a },
                Self::Member { token: right, node: b },
            ) => left == right && a.same_ref(*b),
            _ => false,
        }
    }
}
impl Origin {
    pub(super) fn kept(self, current: Option<NodeToken>) -> bool {
        match self {
            Self::Nowhere => current.is_none(),
            Self::Outside(token) => current == Some(token),
            Self::Member { token, node } => {
                current == Some(token) || (current.is_none() && node.mounted_node().is_none())
            }
        }
    }
}
```

- Every read and write of the old fields:
  - list/machine.rs:11-29: replaced by the code above. `Origin::kept` is still called at machine.rs:73 and state.rs:275; their calls are unchanged.
  - list/state.rs:227-240: replace

```rust
                let origin = self.owner.focused_node();
                let origin_ref = self.roving.with_members(|members| {
                    members
                        .values()
                        .find(|entry| origin.is_some() && entry.node.mounted_node() == origin)
                        .map(|entry| entry.node)
                });
                self.pending.send(machine::Event::Activate {
                    key: key.clone(),
                    origin: machine::Origin {
                        token: origin,
                        node: origin_ref,
                    },
                });
```

  with

```rust
                let origin = match self.owner.focused_node() {
                    None => machine::Origin::Nowhere,
                    Some(token) => self
                        .roving
                        .with_members(|members| {
                            members
                                .values()
                                .find(|entry| entry.node.mounted_node() == Some(token))
                                .map(|entry| entry.node)
                        })
                        .map_or(machine::Origin::Outside(token), |node| {
                            machine::Origin::Member { token, node }
                        }),
                };
                self.pending.send(machine::Event::Activate {
                    key: key.clone(),
                    origin,
                });
```

  - The `NodeRefExt` import in machine.rs stays (it provides `mounted_node` and `same_ref`).
- xtask-allow lines to delete: list/machine.rs:10. An enum is never checked.
- Public signatures: unchanged.
- Tests: `dev/tests/kivact-public-api/tests/e2e/flows.rs::a_list_navigation_in_a_virtual_list_reaches_the_last_row_and_keeps_the_focus` (uses `scroll_with`, the only path that builds an `Origin`).

### SM-INT-04 — dock `Shared.next_rect: Cell<u64>` is an ad-hoc generation

- Site: `kivact-ui-creative/src/dock/data.rs:21-31`, `kivact-ui-creative/src/dock/operations.rs:101-112`, `kivact-ui-creative/src/dock/view.rs:155-158`.
- Current fields (verbatim):

```rust
pub(super) type TabRects<K> = RefCell<HashMap<K, (u64, Value<Option<Rect>>)>>;

// xtask-allow(state-fields): Retained row-rectangle registry and immutable callbacks/texts; the counter is only a registration generation, not interaction phase.
pub(super) struct Shared<K: kivact::state::ItemKey> {
    pub(super) rects: TabRects<K>,
    pub(super) next_rect: Cell<u64>,
    pub(super) menu: MenuBuilder<K>,
    pub(super) on_close: Option<Callback<K>>,
    pub(super) texts: Texts,
}
```

- Why implicit: (d). A raw `u64` counter (`wrapping_add`) is used so that a stale row's `on_cleanup` does not remove a newer registration. S-9 names this exact job: `Generation` ("Monotonic identity for rejecting stale results"; it panics instead of wrapping).
- Replacement:

```rust
// data.rs
use kivact::custom::Generation;
/// Registered tab rectangles with the generation of their registering row.
pub(super) type TabRects<K> = RefCell<HashMap<K, (Generation, Value<Option<Rect>>)>>;

// xtask-allow(state-fields): Retained row-rectangle registry keyed by a named Generation token, and immutable callbacks/texts; no interaction phase.
pub(super) struct Shared<K: kivact::state::ItemKey> {
    pub(super) rects: TabRects<K>,
    pub(super) next_rect: Cell<Generation>,
    pub(super) menu: MenuBuilder<K>,
    pub(super) on_close: Option<Callback<K>>,
    pub(super) texts: Texts,
}
```

```rust
// operations.rs register_rect
    pub(super) fn register_rect(self, id: K, rect: Value<Option<Rect>>) {
        let shared = self.shared();
        let mut generation = shared.next_rect.get();
        let token = generation.next();
        shared.next_rect.set(generation);
        shared.rects.borrow_mut().insert(id.clone(), (token, rect));
        kivact::state::on_cleanup(move || {
            let mut rects = shared.rects.borrow_mut();
            if rects.get(&id).is_some_and(|(owner, _)| owner.is_current(token)) {
                rects.remove(&id);
            }
        });
    }
```

- Every read and write:
  - data.rs:22 (type alias): `u64` becomes `Generation`. Also update the doc on data.rs:21 ("token" becomes "generation").
  - data.rs:24: reworded annotation, as above.
  - data.rs:27: `Cell<u64>` becomes `Cell<Generation>`.
  - operations.rs:104-105: `next_rect.get().wrapping_add(1)` / `.set(token)` become the three `generation` lines above.
  - operations.rs:109: `*owner == token` becomes `owner.is_current(token)`.
  - operations.rs:117-120 (`tab_rect`): unchanged (reads `.1`).
  - view.rs:157: `next_rect: Cell::new(0),` becomes `next_rect: Cell::new(kivact::custom::Generation::new()),`.
- xtask-allow lines: none deleted. data.rs:24 stays but is reworded as above, because `rects` (contains `Option`) and `on_close` (`Option`) still count 2.
- Public signatures: unchanged.
- Tests: `crates/kivact-ui-creative/tests/consumer/dock.rs::tab_drags_split_right_and_bottom_and_keep_panels_mounted` (strip drop targets read the registered rects).

### SM-INT-05 — outliner `RowFactory.generation: Rc<Cell<u64>>` is an ad-hoc generation

- Site: `kivact-ui-creative/src/outliner/view.rs:215-237` (struct), 466 (init), 512-534 (`register`). `kivact-ui-creative/src/outliner/controls.rs:18-25` (`Mounted`).
- Current fields (verbatim):

```rust
// xtask-allow(state-fields): Immutable row factory inputs and owner-scoped registry handles; tree, reveal, rename and drag each own their transitions.
struct RowFactory {
    ...
    generation: Rc<Cell<u64>>,
    ...
    pressed: Rc<Cell<Option<u64>>>,
    ...
}
// controls.rs
pub(super) struct Mounted {
    pub node: NodeRef,
    pub rect: Value<Option<Rect>>,
    pub generation: u64,
}
```

- Why implicit: (d). A raw counter rejects a stale row's cleanup (`row.generation == mount`). This is the S-9 `Generation` job.
- Replacement:
  - controls.rs: add `use kivact::custom::Generation;` and change line 23 to `pub generation: Generation,`.
  - view.rs: add `use kivact::custom::Generation;`.
  - view.rs:220: `generation: Rc<Cell<u64>>,` becomes `generation: Rc<Cell<Generation>>,`.
  - view.rs:466: `let generation = Rc::new(Cell::new(0u64));` becomes `let generation = Rc::new(Cell::new(Generation::new()));`.
  - view.rs:518-519: replace

```rust
        let mount = generation.get() + 1;
        generation.set(mount);
```

  with

```rust
        let mut next = generation.get();
        let mount = next.next();
        generation.set(next);
```

  - view.rs:525: `generation: mount,` is unchanged (now a `Generation`).
  - view.rs:531: `row.generation == mount` becomes `row.generation.is_current(mount)`.
- xtask-allow lines: none deleted. view.rs:215 stays (`plan`, `on_rename`, `on_restriction`, `row_height`, `item_drop`, `has_activation` still count). After SM-INT-05 and SM-INT-06, reword it to: `// xtask-allow(state-fields): Immutable row factory inputs and owner-scoped registry handles (mount tokens are a named Generation); tree, reveal (including the pressed row), rename and drag each own their transitions.`
- Public signatures: unchanged.
- Tests: `crates/kivact-ui-creative/tests/consumer/outliner.rs::the_rows_follow_the_tree_view_keys_and_are_one_tab_stop` and `::rename_is_f2_and_a_double_click_unless_the_item_is_activated` (rows mount and remount through `register`). No test remounts the same id while the old row is still being cleaned up.

### SM-INT-06 — outliner `pressed: Rc<Cell<Option<u64>>>` mirrors the press, outside the reveal machine

- Site: `kivact-ui-creative/src/outliner/view.rs:234, 467-468, 476, 504, 623, 639-640, 673, 681` and `kivact-ui-creative/src/outliner/reveal.rs:1-102`.
- Current fields (verbatim): `pressed: Rc<Cell<Option<u64>>>,` (view.rs:234, inside the RowFactory shown in SM-INT-05) and

```rust
// reveal.rs
#[derive(Clone, Copy, PartialEq)]
pub(super) enum State {
    Idle,
    AwaitingRow(u64),
}
pub(super) fn observe(
    active: Value<Option<u64>>,
    enabled: Value<bool>,
    data: Value<Vec<TreeItem>>,
    expanded: Binding<Vec<u64>>,
    rows: Rows,
    pressed: Rc<Cell<Option<u64>>>,
) -> MachineHandle<Reveal> {
```

- Why implicit: (c)/(e).
  - A `Cell<Option>` is written directly from pointer handlers: `Some(id)` on a primary Down, `None` on Up/Cancel. No transition writes it.
  - It records that a press is in progress and gates later behaviour: reveal skips scrolling a row the user is pressing.
  - The only reader is the reveal decision, which belongs to the reveal Machine.
- Replacement (reveal.rs). The pressed row becomes Machine state, written by an event:

```rust
//! Role: Decide active-row reveal and apply ancestor expansion after transitions.
use super::controls::Rows;
use crate::TreeItem;
use kivact::state::{Binding, Machine, MachineHandle, Step, Value, machine};
use kivact::view::NodeRef;

/// Whether a reveal waits for its row to mount.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Pending {
    Idle,
    AwaitingRow(u64),
}
#[derive(Clone, Copy, PartialEq)]
pub(super) struct State {
    /// The row a primary press is on, from its press to its release; a
    /// press activates its own row without scrolling it.
    pressed: Option<u64>,
    pending: Pending,
}
pub(super) enum Event {
    Press(Option<u64>),
    Active {
        target: Option<u64>,
        ancestors: Vec<u64>,
        node: Option<NodeRef>,
    },
    Mounted {
        id: u64,
        node: NodeRef,
    },
}
pub(super) enum Effect {
    Expand(Vec<u64>),
    Scroll(NodeRef),
}
pub(super) struct Reveal;
impl Machine for Reveal {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        match event {
            Event::Press(pressed) => Step::to(State { pressed, ..*state }),
            Event::Active {
                target,
                ancestors,
                node,
            } => {
                if target.is_some() && target == state.pressed {
                    return Step::to(State {
                        pending: Pending::Idle,
                        ..*state
                    });
                }
                let pending = match (target, node) {
                    (Some(id), None) => Pending::AwaitingRow(id),
                    _ => Pending::Idle,
                };
                let mut step = Step::to(State { pending, ..*state });
                if !ancestors.is_empty() {
                    step = step.with(Effect::Expand(ancestors));
                }
                if let Some(node) = node {
                    step = step.with(Effect::Scroll(node));
                }
                step
            }
            Event::Mounted { id, node } if state.pending == Pending::AwaitingRow(id) => {
                Step::to(State {
                    pending: Pending::Idle,
                    ..*state
                })
                .with(Effect::Scroll(node))
            }
            Event::Mounted { .. } => Step::stay(),
        }
    }
}
pub(super) fn observe(
    active: Value<Option<u64>>,
    enabled: Value<bool>,
    data: Value<Vec<TreeItem>>,
    expanded: Binding<Vec<u64>>,
    rows: Rows,
) -> MachineHandle<Reveal> {
    let reveal = machine(
        State {
            pressed: None,
            pending: Pending::Idle,
        },
        || Reveal,
        move |_, effect| match effect {
            Effect::Expand(ancestors) => {
                let mut open = expanded.get_untracked();
                open.extend(ancestors);
                expanded.set(open);
            }
            Effect::Scroll(node) => node.scroll_into_view(),
        },
    );
    kivact::state::effect_on(active, move |&target| {
        if !enabled.get_untracked() {
            return;
        }
        let ancestors = target.map_or_else(Vec::new, |id| {
            closed_ancestors(id, &data.get_untracked(), &expanded.get_untracked())
        });
        let node = target.and_then(|id| rows.borrow().get(&id).map(|row| row.node));
        reveal.send(Event::Active {
            target,
            ancestors,
            node,
        });
    });
    reveal
}
// closed_ancestors: unchanged
```

  Equivalence: the old filter turned a pressed target into `None`, so the step was `Idle` with no effects. The new early return gives the same result. Computing `ancestors`/`node` for the pressed target is a pure read.
- Every read and write:
  - reveal.rs:6-7 (`use std::cell::Cell; use std::rc::Rc;`): delete.
  - reveal.rs:9-13 (`enum State`): becomes `Pending` + `struct State`.
  - reveal.rs:14 `Event`: add `Press(Option<u64>)`.
  - reveal.rs:35-58 `transition`: as above.
  - reveal.rs:67: `pressed` parameter deleted.
  - reveal.rs:68-70 initial state: as above.
  - reveal.rs:85 `let target = target.filter(|id| pressed.get() != Some(*id));`: delete.
  - view.rs:234 `pressed: Rc<Cell<Option<u64>>>,`: delete.
  - view.rs:467-468 (comment and `let pressed = Rc::new(Cell::new(None::<u64>));`): delete.
  - view.rs:476 `pressed.clone(),` (argument to `observe`): delete.
  - view.rs:504 `pressed,` (struct init): delete.
  - view.rs:623 `let pressed = self.pressed.clone();` becomes `let reveal = self.reveal;`.
  - view.rs:639-640 (`let released = pressed.clone(); let pressed = pressed.clone();`): delete.
  - view.rs:673 `pressed.set(Some(id));` becomes `reveal.send(super::reveal::Event::Press(Some(id)));`.
  - view.rs:681 `released.set(None);` becomes `reveal.send(super::reveal::Event::Press(None));`.
  - The `Rc`/`Cell` imports in view.rs stay (`generation`, `mounted`).
- xtask-allow lines: none deleted (see SM-INT-05 for the reworded view.rs:215). The new reveal `State` has one counted field and `Pending` is not a `*State`/`*Phase` enum, so it needs no annotation.
- Public signatures: unchanged.
- Tests: `crates/kivact-ui-creative/tests/consumer/outliner.rs` (both tests press rows; the double-click rename path passes through the changed handler). No test asserts that pressing a row does not scroll it.

## 4. UNANNOTATED-FIX entries

### SM-INT-07 — dialog `DialogOptions.return_focus` stores a fact the popup already owns

- Site: `kivact-interaction/src/dialog.rs:80-86, 218-227`.
- Current fields (verbatim):

```rust
#[derive(Clone, Copy, Default)]
struct DialogOptions {
    dismiss: Option<DismissOn>,
    return_focus: Option<Value<NodeRef>>,
}
```

- Why implicit: (h). `DialogApi::return_focus` writes the target twice: to the popup (`self.popup.return_focus_target(target)`, which owns it and reads it in `base_policy`) and to `DialogOptions.return_focus`. Nothing ever reads the second copy (the only `settings.get()` read is `.dismiss` on line 257).
- Replacement:

```rust
#[derive(Clone, Copy, Default)]
struct DialogOptions {
    dismiss: Option<DismissOn>,
}

    pub fn return_focus(self, target: impl kivact_core::state::IntoValue<NodeRef>) -> Self {
        self.popup.return_focus_target(target.into_value());
        self
    }
```

- Every read and write:
  - dialog.rs:85: field deleted.
  - dialog.rs:222-226: replace the body with the two lines above. This removes `let target = ...;` and the `self.settings.update(|settings| settings.return_focus = Some(target));` write. There were no reads.
- xtask-allow lines: none (the struct is unannotated, and the `*Options` suffix exempts it anyway).
- Public signatures: unchanged (`DialogApi::return_focus` keeps its signature and behaviour).
- Tests: `dev/tests/kivact-public-api/tests/e2e/interaction_dialog.rs` (popover focus return). `DialogApi::return_focus` is reached through kivact-ui date/time inputs: `dev/tests/kivact-public-api/tests/e2e/ui_calendar.rs`, `apg_date_picker.rs`.

### SM-INT-08 — list duplicate-key report uses an ad-hoc "last reported" RefCell

- Site: `kivact-interaction/src/list/state.rs:392-425` (`observe_list`).
- Current code (verbatim):

```rust
    let duplicates = RefCell::new(Vec::<K>::new());
    ...
        let next = state.order.with(|order| order.duplicates().to_vec());
        if *duplicates.borrow() != next {
            if !next.is_empty() {
                owner.report(
                    ErrorKind::Message,
                    format!(
                        "ListApi: {} duplicate logical key(s); first occurrence wins",
                        next.len()
                    ),
                );
            }
            *duplicates.borrow_mut() = next;
        }
```

- Why implicit: (c). A `RefCell<Vec<K>>` records what was last reported and gates the next report. This is the S-9 `Sent` cache ("Last value handed to an outside party"), which already exists as a named type.
- Replacement:

```rust
    let duplicates = RefCell::new(Sent::<Vec<K>>::new());
    ...
        let next = state.order.with(|order| order.duplicates().to_vec());
        let changed = duplicates.borrow_mut().update(next).map(Vec::len);
        if let Some(count) = changed
            && count > 0
        {
            owner.report(
                ErrorKind::Message,
                format!("ListApi: {count} duplicate logical key(s); first occurrence wins"),
            );
        }
```

  Add `Sent` to the import on state.rs:12: `use kivact_core::custom::{NodeRefExt, Owned, Owner, Sent};`.

  Equivalence: the first `update` always returns `Some`. The first value is the empty list, so `count == 0` and nothing is reported, the same as the old initial empty `Vec`. The borrow ends before `owner.report` runs.
- Every read and write: state.rs:395 (init), state.rs:408-420 (comparison, report, store). Nothing else reads `duplicates`.
- xtask-allow lines: none (local variable).
- Public signatures: unchanged.
- Tests: no test found (no E2E asserts the "duplicate logical key" report).

## 5. Deferred list (superseded: each item is now prescribed in section 6 as SM-INT-09..15)

These match a–h. They were held back because a local fix changes behaviour or needs a core API. The coordinator has since decided each design (D1 → SM-INT-09, D2 → 10, D3 → 11, D4 → 12, D5 → 13, D6 → 14, D7 → 15).

- D1 `kivact-interaction/src/menu/level.rs:208-209`: `let ready = signal(false); on_mount(move || ready.set(true));` gates the want effect (c). It needs a core "mounted" `Value`; `tracked_node()` rebinds before the host acknowledges, so it is not equivalent.
- D2 `kivact-interaction/src/menu/level.rs:392,329-330`: submenu items keep their own `hovered: Signal<bool>` mirroring Enter/Leave (h), while plain items use `item_ref.hovered()`. The press machine consumes Enter/Leave first, so switching to `item_ref.hovered()` changes the hover-during-press behaviour.
- D3 `kivact-interaction/src/hover_card.rs:53-67`: three effects form a two-way sync (e). `effect_on(surface.phase)` writes the caller `requested` binding, and `effect_on(requested)` calls `dismiss`. It needs a `Shown`/`Hidden` Effect on the delayed machine (`delayed/machine.rs`, `Effect` is empty today).
- D4 `kivact-interaction/src/list/state.rs:401-407`: an effect mirrors the derived `active` position into the machine (`Event::Position`), and `position` (lines 103-108) feeds back into `active` (e).
- D5 `kivact-ui-creative/src/dock/chrome.rs:524-526`: `cx.drag.is_some()` doubles as a drag-threshold latch (c). `DragApi::slop` could own it, but slop changes when the drag becomes Active (and so pointer capture).
- D6 `kivact-ui-creative/src/outliner/controls.rs:47-105`: `Painter.paint: Signal<Option<Paint>>` is kept in lockstep with its `EditSession` phase (b).
- D7 `kivact-interaction/src/palette/machine.rs`: the `Open`/`Closed` state duplicates the caller's `open` binding (initial `Open` even when `open` starts false) (h). It also deduplicates repeated `Close` events, so `State = ()` would allow extra resets.

## 6. Prescribed fixes for the former deferred items (coordinator decisions applied)

### SM-INT-09 — menu level `ready` flag becomes core `kivact::view::is_mounted()` (former D1)

- Site: `kivact-interaction/src/menu/level.rs:15, 206-213`. New core file `kivact-core/src/view/mounted.rs`; re-export in `kivact-core/src/view.rs`.
- Current code (verbatim, level.rs:206-213):

```rust
        // Focus requests addressed to this level wait for its nodes
        // to be live.
        let ready = signal(false);
        on_mount(move || ready.set(true));
        effect(move || {
            if !ready.get() {
                return;
            }
```

- Why implicit: (c). A local `Signal<bool>` records that the level has mounted and gates the want effect. "This owner's view is committed" is a core fact; the menu should not store its own copy.
- Replacement, core (new file `crates/kivact-core/src/view/mounted.rs`):

```rust
//! Role: Report whether the current owner's view has been committed.

use kivact_runtime::{Value, on_mount, signal};

/// Whether the view the current setup creates is live (tracked): `false`
/// during setup, `true` from the mount pass in which the host has applied
/// this setup's nodes and the owner's `on_mount` callbacks run (this one
/// runs in its registration order among them), until the owner is
/// disposed. Setup only.
///
/// ```no_run
/// use kivact_core::state::effect;
/// use kivact_core::view::is_mounted;
/// // During component setup:
/// let mounted = is_mounted();
/// effect(move || {
///     if mounted.get() {
///         // The nodes of this setup are live.
///     }
/// });
/// ```
#[track_caller]
pub fn is_mounted() -> Value<bool> {
    let mounted = signal(false);
    on_mount(move || mounted.set(true));
    mounted.into()
}
```

  This has one owner: the signal is created and written only inside core's `on_mount` machinery. `on_mount` callbacks run in a handler frame (runtime `run_mounts_acked`), so the write is legal, and the effect-writes check does not look at `on_mount` closures.
- Replacement, core re-export (`crates/kivact-core/src/view.rs`):
  - After line 28 `pub(crate) mod mount;`, add `mod mounted;`.
  - After line 80 `pub use modifier_keys::modifiers;`, add:

```rust
#[doc(inline)]
pub use mounted::is_mounted;
```

  - The facade path is `kivact::view::is_mounted` automatically (`crates/kivact/src/lib.rs:52` re-exports `kivact_core::view`).
  - Prelude decision: NOT in the prelude. Do not touch `crates/kivact/src/prelude.rs`; it lists names explicitly.
- Replacement, menu (level.rs:208-209):

```rust
        let ready = kivact_core::view::is_mounted();
```

  The effect body (`if !ready.get() { return; }` …) is unchanged; `ready` is now a `Value<bool>`.
- Every read and write:
  - level.rs:208 `let ready = signal(false);` becomes the line above.
  - level.rs:209 `on_mount(move || ready.set(true));`: delete.
  - level.rs:211 `ready.get()`: unchanged.
  - level.rs:15: remove `on_mount` from `use kivact_core::state::{Signal, Value, effect, memo, on_cleanup, on_mount, signal};`. Line 209 was its only use; `signal` stays (lines 87, 99, 107, 108).
- xtask-allow lines: none.
- Public signatures: added `kivact_core::view::is_mounted` / `kivact::view::is_mounted` (new public function). Nothing else changes.
- Behaviour: unchanged. It is the same signal + `on_mount` mechanics, created at the same point (`mount_content`, under the surface's `on_attach`).
- Tests: `dev/tests/kivact-public-api/tests/e2e/interaction_menu.rs::a_menu_button_opens_on_its_first_or_last_enabled_item_and_walks_over_enabled_items` and `::right_and_left_move_between_levels_and_escape_closes_one_level_at_a_time` (wants consumed after mount).
- New E2E test for the new API: T8.2 appends the sketch below to the existing file `dev/tests/kivact-public-api/tests/e2e/lifecycle.rs` (it uses the file's `crate::support::mount`; do not create a new file):

```rust
#[test]
fn is_mounted_is_false_during_setup_and_true_once_the_nodes_are_live() {
    let (screen, mounted) = crate::support::mount(|| {
        let mounted = kivact::view::is_mounted();
        assert!(!mounted.get_untracked(), "false while the setup runs");
        (mounted, text("Body"))
    });
    assert!(mounted.get_untracked(), "true after the host applied the nodes");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```

### SM-INT-10 — menu item `hovered` signal mirrors Enter/Leave; read the engine's hover (former D2)

- Site: `kivact-interaction/src/menu/level.rs:17, 247-303, 320-334, 392-400, 423`.
- Current code (verbatim excerpts):

```rust
    fn watch_item(
        self,
        key: K,
        submenu: bool,
        item_ref: kivact_core::view::NodeRef,
        disabled: Value<bool>,
        hovered: Signal<bool>,
        pressed_over: Value<Option<bool>>,
    ) {
...
            let over = item_ref.hovered();
...
    fn item_pointer(self, hovered: Signal<bool>, event: PointerEvent) -> Handled {
        let session = self.session;
        if let Some(press) = session.press {
            let handled = press.event(session, super::press_machine::Source::Menu, event);
            if handled == Handled::Yes {
                return handled;
            }
        }
        match event.phase() {
            PointerPhase::Enter => hovered.set(true),
            PointerPhase::Leave => hovered.set(false),
            _ => {}
        }
        Handled::No
    }
...
        let hovered = signal(false);
        self.watch_item(
            key.clone(),
            submenu,
            item_ref,
            disabled,
            hovered,
            pressed_over,
        );
...
                .on_pointer(move |event: PointerEvent| self.item_pointer(hovered, event))
```

- Why implicit: (h). Each item stores its own copy of the pointer's hover, written from Enter/Leave, while the engine owns hover (`NodeRef::hovered`, the architecture table "region hover"). Plain items already read `item_ref.hovered()` for `Rest`, but read the private copy for the focus-follow effect.
- Replacement (level.rs):

```rust
    fn watch_item(
        self,
        key: K,
        submenu: bool,
        item_ref: kivact_core::view::NodeRef,
        disabled: Value<bool>,
        pressed_over: Value<Option<bool>>,
    ) {
        let Self {
            session,
            depth,
            shown,
            ..
        } = self;
        // The engine owns hover: the item's node is hovered while the
        // pointer is over it, whatever presses or captures there are.
        let hovered = item_ref.hovered();
        // The pointer focuses the item it rests on; a submenu opens after a rest.
        let timing = self.timing;
        let aiming = self.aiming;
        if submenu {
            if !timing.close.is_zero() {
                observe_rect(self.anchors, key.clone(), item_ref.rect());
            }
            let key = key.clone();
            let resting = memo(move || {
                pressed_over.get().unwrap_or_else(|| hovered.get()) && !disabled.get()
            });
            effect(move || {
                session.machine.send(super::machine::Event::Rest(
                    depth,
                    key.clone(),
                    resting.get() && !aiming.get(),
                    true,
                ));
            });
        } else {
            let key = key.clone();
            effect(move || {
                session.machine.send(super::machine::Event::Rest(
                    depth,
                    key.clone(),
                    pressed_over.get().unwrap_or_else(|| hovered.get())
                        && !aiming.get()
                        && shown.get(),
                    false,
                ));
            });
        }

        let hover_key = key.clone();
        effect(move || {
            if hovered.get() && !aiming.get() && !disabled.get() && shown.get() {
                session.hover(depth, hover_key.clone());
            }
        });
    }
```

```rust
    /// Pass a pointer event on to the menu press, when the theme opted in.
    fn item_pointer(self, event: PointerEvent) -> Handled {
        match self.session.press {
            Some(press) => press.event(self.session, super::press_machine::Source::Menu, event),
            None => Handled::No,
        }
    }
```

- Every read and write:
  - level.rs:253 parameter `hovered: Signal<bool>`: delete. Add `let hovered = item_ref.hovered();` as shown.
  - level.rs:271 `hovered.get()` (submenu `resting`): unchanged text, now reads the engine Value.
  - level.rs:283 `let over = item_ref.hovered();`: delete. Line 286 `over.get()` becomes `hovered.get()`.
  - level.rs:298 `hovered.get()` (focus-follow effect, plain and submenu items): unchanged text, now the engine Value.
  - level.rs:320-334 `item_pointer`: replaced as above. The writes on 329-330 are gone.
  - level.rs:392 `let hovered = signal(false);`: delete.
  - level.rs:393-400 `self.watch_item(..)`: drop the `hovered,` argument (line 398).
  - level.rs:423 becomes `.on_pointer(move |event: PointerEvent| self.item_pointer(event))`.
  - level.rs:17: remove `PointerPhase` from the `use kivact_core::view::{..}` list (329-330 were its only uses). `Signal`, `signal`, `PointerEvent` stay.
- xtask-allow lines: none.
- Public signatures: unchanged.
- Behaviour change (accepted):
  - Hover is now the engine's hit, whatever presses or captures there are.
  - Resting on a submenu item while a press is held (a press that began elsewhere and captured the pointer) now opens the submenu after the open delay, and the hovered item takes the focus, like native menus. Before, the captured press kept Enter from reaching the item, so it stayed closed and unfocused.
  - After such a press is released, an item that was entered during the press counts as hovered at once. Before, it counted only after the pointer left and entered again.
  - Plain items' focus-follow now uses the same engine hover.
- Tests:
  - `interaction_menu.rs::choosing_calls_back_with_the_key_closes_every_level_and_returns_the_focus` ("Quit"/"Export" `.hover()` makes the item the tab stop) and `::roles_checked_expanded_description_and_the_separator_are_published` ("More" `.hover()` + click).
  - `ui_components.rs::menu_renders_nested_levels_and_custom_trigger_keeps_its_style`.
- New E2E test: T8.2 appends the sketch below to the existing file `dev/tests/kivact-public-api/tests/e2e/interaction_menu.rs` (it uses that file's own `real_window`, `menu_button`, `nested_entries`, `action`, `item`, `center`; do not create a new file):

```rust
#[test]
fn a_submenu_item_the_pointer_rests_on_during_a_held_press_opens_after_the_delay() {
    let (screen, _chosen) = mount_with(real_window(), || {
        let chosen = signal(Vec::new());
        let file = use_menu(nested_entries())
            .on_activate(move |key: &'static str| chosen.update(|c| c.push(key)));
        (chosen, column((menu_button("File", file), action("After"))))
    });
    screen.get_by_role(Role::Button, "File").press_key(Key::ArrowDown);
    let new = center(&item(&screen, "New"));
    let export = center(&item(&screen, "Export"));
    // A primary press held from another item, then a rest on the submenu item.
    screen.mouse_move(new.x, new.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(export.x, export.y);
    screen.advance(Duration::from_millis(200)); // > SubmenuTiming::default().open (140 ms)
    assert_eq!(
        item(&screen, "Export").is_expanded(),
        Some(true),
        "engine hover opens the submenu while the press is held"
    );
    assert!(item(&screen, "Export").is_focused(), "the hovered item takes the focus");
    screen.mouse_up(MouseButton::Primary);
    assert!(screen.get_by_role(Role::Menu, "Export").exists());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```

### SM-INT-11 — hover card two-way sync becomes delayed-machine effects + feed (former D3)

- Site:
  - `kivact-interaction/src/hover_card.rs:3, 43-67` (three effects).
  - `kivact-interaction/src/delayed/machine.rs` (whole file).
  - `kivact-interaction/src/delayed/api.rs:12, 69-105` plus new methods.
  - `kivact-interaction/src/hover_corridor/api.rs:38-89, 140-156`.
  - `kivact-interaction/src/tooltip/api.rs:177-187`.
- Current code (verbatim, hover_card.rs:53-67):

```rust
    effect(move || {
        if immediate.get() && surface.phase().get() == DelayedPhase::Opening {
            surface.pulse();
        }
    });
    effect_on(surface.phase(), move |phase| match phase {
        DelayedPhase::Shown => requested.set(true),
        DelayedPhase::Hidden | DelayedPhase::Dismissed => requested.set(false),
        DelayedPhase::Opening | DelayedPhase::Closing => (),
    });
    effect_on(requested, move |open| {
        if !open {
            surface.dismiss();
        }
    });
```

- Why implicit: (e).
  - One effect mirrors the delayed phase into the caller's `requested` binding, and another effect mirrors the binding back into the machine (`dismiss`): a two-way sync.
  - The first effect repairs an `Opening` state when focus arrives.
- Replacement, `delayed/machine.rs` (whole file):

```rust
//! Role: Compute delayed visibility transitions and own their timer operations.
use kivact_core::state::{Machine as MachineTrait, Step};
use std::time::Duration;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum State {
    Hidden,
    Opening,
    Shown,
    Closing,
    Dismissed,
}
pub(super) enum Event {
    Wanted(bool),
    Dismiss,
    Pulse,
    OpenDue,
    CloseDue,
    /// The owner's open request changed: `false` dismisses, `true` changes nothing.
    Request(bool),
    /// The surface may open at once (focus arrived): while `Opening`, acts as `Pulse`.
    Immediate(bool),
}
/// A change of the visibility, reported to the owner.
pub(super) enum Effect {
    /// Entered `Shown` from another state.
    Shown,
    /// Entered `Hidden` or `Dismissed` from another state.
    Hidden,
}
pub(super) struct Machine {
    pub wanted: bool,
    pub open: Duration,
    pub close: Duration,
}
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        let from = *state;
        match event {
            Event::Wanted(true) => match from {
                State::Hidden if self.open.is_zero() => enter(from, State::Shown),
                State::Hidden => enter(from, State::Opening).after(self.open, Event::OpenDue),
                State::Closing => enter(from, State::Shown).cancel(Event::CloseDue),
                _ => Step::stay(),
            },
            Event::Wanted(false) => match from {
                State::Opening => enter(from, State::Hidden).cancel(Event::OpenDue),
                State::Shown => self.close(from),
                State::Dismissed => enter(from, State::Hidden),
                _ => Step::stay(),
            },
            Event::Dismiss | Event::Request(false) => self.dismiss(from),
            Event::Request(true) => Step::stay(),
            Event::Pulse => self.pulse(from),
            Event::Immediate(true) if from == State::Opening => self.pulse(from),
            Event::Immediate(_) => Step::stay(),
            Event::OpenDue if from == State::Opening => enter(from, State::Shown),
            Event::CloseDue if from == State::Closing => enter(from, State::Hidden),
            Event::OpenDue | Event::CloseDue => Step::stay(),
        }
    }
}
impl Machine {
    fn close(&self, from: State) -> Step<State, Event, Effect> {
        if self.close.is_zero() {
            enter(from, State::Hidden).cancel(Event::CloseDue)
        } else {
            enter(from, State::Closing).after(self.close, Event::CloseDue)
        }
    }
    fn dismiss(&self, from: State) -> Step<State, Event, Effect> {
        enter(
            from,
            if self.wanted {
                State::Dismissed
            } else {
                State::Hidden
            },
        )
        .cancel(Event::OpenDue)
        .cancel(Event::CloseDue)
    }
    fn pulse(&self, from: State) -> Step<State, Event, Effect> {
        let step = if self.wanted {
            enter(from, State::Shown).cancel(Event::CloseDue)
        } else {
            self.close(from)
        };
        step.cancel(Event::OpenDue)
    }
}
/// Move to `to`, reporting an entry into `Shown` or into a hidden state.
fn enter(from: State, to: State) -> Step<State, Event, Effect> {
    let step = Step::to(to);
    if from == to {
        return step;
    }
    match to {
        State::Shown => step.with(Effect::Shown),
        State::Hidden | State::Dismissed => step.with(Effect::Hidden),
        State::Opening | State::Closing => step,
    }
}
```

  Equivalence:
  - Every old `Step::to(X)` becomes `enter(from, X)` with the same timers.
  - `Effect::Shown`/`Effect::Hidden` fire exactly when the old `effect_on(phase)` saw a change into `Shown` / into `Hidden | Dismissed` (including `Dismissed → Hidden` and `Opening → Hidden`).
  - `Request(false)` is the old `dismiss()`.
  - `Immediate(true)` in `Opening` is the old `pulse()`.
- Replacement, `delayed/api.rs`:
  - Line 12 `use super::machine::{Event, Machine, State};` becomes `use super::machine::{Effect, Event, Machine, State};`.
  - Line 70 (`use_delayed`) becomes `delayed(wanted.into_value(), move || delays.open, delays.close, |_| ())`.
  - Lines 76-105 (`delayed`): add a fourth parameter and use it in the run closure:

```rust
pub(crate) fn delayed(
    wanted: Value<bool>,
    open: impl Fn() -> Duration + 'static,
    close: Duration,
    on_visible: impl Fn(bool) + 'static,
) -> DelayedApi {
    let state = machine(
        State::Hidden,
        move || Machine {
            wanted: wanted.get_untracked(),
            open: open(),
            close,
        },
        move |_, effect| match effect {
            Effect::Shown => on_visible(true),
            Effect::Hidden => on_visible(false),
        },
    );
    // (the `phase`, `shown`, `this` and `effect(.. Event::Wanted ..)` lines are unchanged)
```

  - Add to `impl DelayedApi` (after `pulse`):

```rust
    /// Dismiss whenever `requested` changes to `false`, as `dismiss` does; a
    /// change to `true` changes nothing. Setup only.
    pub(crate) fn dismiss_on_release(self, requested: impl IntoValue<bool>) {
        self.state.feed(requested, Event::Request);
    }

    /// End a running open delay at once whenever `immediate` changes to
    /// `true`, as `pulse` does while `Opening`. Setup only.
    pub(crate) fn hurry_on(self, immediate: impl IntoValue<bool>) {
        self.state.feed(immediate, Event::Immediate);
    }
```

- Replacement, `hover_corridor/api.rs`:
  - `use_hover_surface` (line 44-50): add a last argument `|_| ()` to the `hover_surface(..)` call.
  - `hover_surface` (line 53-59): add the last parameter `on_visible: impl Fn(bool) + 'static`. Pass it as the fourth argument of `crate::delayed::delayed(wanted, move || {..}, delays.close, on_visible)` (line 65-74).
  - Add to `impl HoverSurfaceApi`:

```rust
    pub(crate) fn dismiss_on_release(self, requested: impl IntoValue<bool>) {
        self.delayed.dismiss_on_release(requested);
    }
    pub(crate) fn hurry_on(self, immediate: impl IntoValue<bool>) {
        self.delayed.hurry_on(immediate);
    }
```

- Replacement, `tooltip/api.rs:177-187`: the fourth argument is the `on_visible` closure prescribed in SM-INT-16 (it replaces the tooltip's mirror effect). If SM-INT-16 lands separately, use `|_| ()` here temporarily.
- Replacement, `hover_card.rs`:
  - Line 3 becomes `use kivact_core::state::{Binding, IntoBinding, IntoValue, Value, memo};`.
  - `DelayedPhase` stays imported (used in the `state` memo).
  - Lines 43-67 become:

```rust
    // The surface's visibility changes are the only writer of `requested`;
    // the caller's own writes reach the surface through `feed`.
    let surface = crate::hover_corridor::hover_surface(
        anchor,
        surface,
        enabled,
        immediate,
        Delays {
            open: timing.open_delay,
            close: timing.close_delay,
        },
        move |visible| requested.set(visible),
    );
    surface.hurry_on(immediate);
    surface.dismiss_on_release(requested);
```

  - The `shown` and `state` memos and `HoverCardApi { .. }` are unchanged. `HoverCardApi::dismiss` (line 110-113) is unchanged.
- Every read and write of the removed sync:
  - hover_card.rs:53-57: the pulse effect is replaced by `hurry_on`.
  - hover_card.rs:58-62: `requested.set(true|false)` moves into the `on_visible` closure, run as a machine effect in a handler frame.
  - hover_card.rs:63-67: `surface.dismiss()` on false is replaced by `dismiss_on_release` (feed → `Event::Request`).
  - The only other callers of `delayed`/`hover_surface` are `use_delayed`, `use_hover_surface` and the tooltip (above).
- xtask-allow lines: none.
- Public signatures: unchanged. `delayed` and `hover_surface` are `pub(crate)`; `use_delayed`, `use_hover_surface` and `use_hover_card` keep their signatures.
- Behaviour: unchanged. One timing detail differs: `requested` is now written inside the transition's effect step instead of one effect flush later.
- Tests: `dev/tests/kivact-public-api/tests/e2e/ui_disclosure_surfaces.rs::hover_card_opens_after_the_delay_and_on_focus` and `::hover_card_controlled_binding_rejection_and_external_close_are_authoritative` (read-only binding, external close, no reopen within the same engagement).
- New E2E test covering `Immediate`: T8.2 appends the sketch below to the existing file `dev/tests/kivact-public-api/tests/e2e/ui_disclosure_surfaces.rs` (it uses `button` from `kivact::ui` and the file's `mount`; do not create a new file):

```rust
#[test]
fn focus_during_the_open_delay_opens_the_hover_card_at_once() {
    let (screen, ()) = mount(|| {
        (
            (),
            kivact::ui::hover_card(|| text("Preview body"))
                .label("Preview")
                .open_delay(Duration::from_millis(200))
                .attach(button("Preview trigger")),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Preview trigger");
    trigger.hover();
    screen.advance(Duration::from_millis(50));
    assert!(screen.query_by_role(Role::Dialog, "Preview").is_none(), "the open delay runs");
    trigger.focus();
    assert!(screen.query_by_role(Role::Dialog, "Preview").is_some(), "focus ends the delay");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```

### SM-INT-12 — list stored `position` and its mirroring effect become a pure projection (former D4)

- Site: `kivact-interaction/src/list/machine.rs:42-54, 70`; `kivact-interaction/src/list/state.rs:94-108, 371-372, 394, 400-407`.
- Current code (verbatim):

```rust
// machine.rs
pub(super) struct State<K> {
    pub(super) pending: Pending<K>,
    pub(super) position: usize,
    pub(super) focus: Focus,
}
pub(super) enum Event<K> {
    Activate { key: K, origin: Origin },
    Cancel,
    Position(usize),
    MountFocus(Focus),
    Observe(Option<NodeToken>),
}
// state.rs
    let position = memo(move || {
        requested
            .get()
            .and_then(|key| order.with(|order| order.position(&key)))
            .unwrap_or_else(|| pending.state().get().position)
    });
...
        // History follows the derived tab stop even before an explicit focus request.
        if let Some(at) = state
            .active
            .get()
            .and_then(|key| state.order.with(|order| order.position(&key)))
        {
            pending.send(machine::Event::Position(at));
        }
```

- Why implicit: (e)/(h).
  - An effect copies the derived `active` key's ordinal into machine state (`Event::Position`).
  - That stored ordinal feeds back into `position`, which `active` reads: a fact stored twice and synced by an effect.
- Replacement:
  - machine.rs State:

```rust
#[derive(Clone, PartialEq)]
pub(super) struct State<K> {
    pub(super) pending: Pending<K>,
    pub(super) focus: Focus,
}
pub(super) enum Event<K> {
    Activate { key: K, origin: Origin },
    Cancel,
    MountFocus(Focus),
    Observe(Option<NodeToken>),
}
```

  - machine.rs `transition`: delete the arm `Event::Position(position) => next.position = position,` (line 70).
  - state.rs:94-108:

```rust
    let pending = machine(
        machine::State {
            pending: machine::Pending::Nothing,
            focus: machine::Focus::Physical,
        },
        || machine::Machine::new(),
        |_, ()| {},
    );
    // Where a search for the tab stop starts: the requested row's ordinal in
    // the current order, else the first row. A pure projection; no history.
    let position = memo(move || {
        requested
            .get()
            .and_then(|key| order.with(|order| order.position(&key)))
            .unwrap_or(0)
    });
```

  - state.rs:371-372 comment becomes `// Start at the requested row's ordinal while that row is listed; otherwise at the first row.`. The code is unchanged.
  - state.rs:394 `let pending = state.pending;`: delete (its only use was the deleted send).
  - state.rs:400-407: delete the comment and the `if let Some(at) = .. { pending.send(machine::Event::Position(at)); }` block. The rest of the effect (duplicates report, `reconcile`) is unchanged; `reconcile` still tracks `state.active`.
- Every read and write of the old position: machine.rs:45 (field), 51 (event), 70 (arm); state.rs:97 (init), 107 (read), 406 (write). `active_value`'s `position: Value<usize>` parameter is unchanged (state.rs:321, 355, 374).
- xtask-allow lines: none.
- Public signatures: unchanged.
- Behaviour change (accepted), Select-mode lists without a `leave` rule only:
  - When the requested row leaves the items, the next tab stop is the first available row, not the row that moved into the old ordinal.
  - When focus left the list (`reset_on_exit`) and the selected rows disappeared, the Tab entry starts at the first row.
  - Callers that need a neighbour keep it with the existing `leave` rule.
- Tests:
  - `dev/tests/kivact-public-api/tests/e2e/interaction_list_nav.rs` (tab stop and arrows): no removal case.
  - `dev/tests/kivact-public-api/tests/e2e/flows.rs::a_list_navigation_in_a_virtual_list_reaches_the_last_row_and_keeps_the_focus`.
- New E2E test: T8.2 appends the sketch below to the existing file `dev/tests/kivact-public-api/tests/e2e/interaction_list_nav.rs` (it uses that file's own `real_window`, `button`, `focus_row`, `row`, `no_errors`; do not create a new file):

```rust
#[test]
fn removing_the_focused_row_moves_the_tab_stop_to_the_first_row_without_a_leave_rule() {
    let (screen, rows) = mount_with(real_window(), || {
        let rows = signal(vec!["Apple", "Apricot", "Banana", "Cherry", "Date"]);
        let selected = signal(Vec::<&'static str>::new());
        let nav = use_list(
            move || rows.get().into_iter().map(|key| ListItem::new(key, key)).collect::<Vec<_>>(),
            selected.into_selection(),
            ListOptions::default(),
        );
        let list = for_each(rows, |name| *name, move |row_value| {
            let name = row_value.with(|name| *name);
            node().role(Role::Option).name(name).child(text(name)).part(nav.item(name))
        });
        (
            rows,
            column((
                button("before"),
                column(list).role(Role::ListBox).name("Fruit").part(nav.list()),
                button("after"),
            )),
        )
    });
    focus_row(&screen, "Cherry");
    rows.update(|rows| rows.retain(|name| *name != "Cherry"));
    screen.run_until_idle();
    assert!(
        row(&screen, "Apple").is_focused(),
        "the search starts at the first row (before SM-INT-12 it was Date, at Cherry's old ordinal)"
    );
    no_errors(&screen);
}
```

### SM-INT-13 — dock tab drag threshold latch becomes `DragApi::slop` (former D5)

- Site: `kivact-ui-creative/src/dock/chrome.rs:515-547` (`tab_drag`).
- Current code (verbatim, 520-534):

```rust
    use_drag::<()>(DragCoordinates::Window)
        .disabled(renaming)
        .on_update(move |update: DragUpdate<()>| {
            let id = key.get();
            let started = cx.drag.with(Option::is_some);
            if !started && update.delta.x.hypot(update.delta.y) < DRAG_THRESHOLD {
                return;
            }
            let target = cx.root.get().and_then(|root| {
                cx.current.with(|layout| {
                    drop_target(layout, root, update.pos, &id, |tab| cx.tab_rect(tab))
                })
            });
            cx.drag.set(Some(TabDrag { source: id, target }));
        })
```

- Why implicit: (c). `cx.drag.is_some()` doubles as a latch recording that the threshold was passed, and it gates the later updates. DragApi's own `Pending` state (slop) already owns that fact.
- Replacement (chrome.rs:520-534):

```rust
    use_drag::<()>(DragCoordinates::Window)
        // The drag (and its pointer capture) starts once the pointer has
        // moved past the threshold; DragApi's Pending state owns that.
        .slop(DRAG_THRESHOLD)
        .disabled(renaming)
        .on_update(move |update: DragUpdate<()>| {
            let id = key.get();
            let target = cx.root.get().and_then(|root| {
                cx.current.with(|layout| {
                    drop_target(layout, root, update.pos, &id, |tab| cx.tab_rect(tab))
                })
            });
            cx.drag.set(Some(TabDrag { source: id, target }));
        })
```

  `.on_end(..)` (535-546) is unchanged. A release inside the slop clears DragApi's `Pending` without calling `on_end`; `cx.drag` is `None` then, as before.
- Every read and write:
  - chrome.rs:524-527: the latch read is deleted.
  - chrome.rs:533/537/545 (`cx.drag.set(..)`) and the reads at chrome.rs:151, 540 and view.rs:329 (drop preview): unchanged.
  - `DRAG_THRESHOLD` (chrome.rs:32) stays as the slop value.
- xtask-allow lines: none.
- Public signatures: unchanged.
- Behaviour change (accepted):
  - DragApi's `Active` state and the pointer capture now start only after the pointer moves more than 5 px (`> slop`; before, a move of exactly 5 px also started).
  - A press that stays within 5 px never captures and never calls `on_update`/`on_end`.
- Tests: `crates/kivact-ui-creative/tests/consumer/dock.rs::tab_drags_split_right_and_bottom_and_keep_panels_mounted` (drags far past 5 px) and `::tabs_activate_by_click_arrow_keys_and_invoke_and_publish_roles` (clicks stay inside the slop).
- New E2E test: T1.4 appends the sketch below to the existing file `crates/kivact-ui-creative/tests/consumer/dock.rs` (it uses that file's `mount_dock`; do not run it):

```rust
#[test]
fn a_tab_press_that_stays_inside_the_threshold_neither_drags_nor_captures() {
    let (screen, probe) = mount_dock();
    let before = probe.layout.get_untracked();
    let at = screen.get_by_role(Role::Tab, "Text").rect().expect("laid out").center();
    screen.mouse_move(at.x, at.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(at.x + 3.0, at.y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(probe.layout.get_untracked(), before, "no move inside the slop");
    // Never captured: a later move past the threshold is an ordinary hover.
    screen.mouse_move(at.x + 60.0, at.y);
    assert_eq!(probe.layout.get_untracked(), before);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```

### SM-INT-14 — outliner `Painter.paint` moves into the EditSession phase payload (former D6)

- Crate order (schedule first):
  - `EditSession::replace_initial` lives in the crate **kivact-interaction**, file `W:\w\kivact\crates\kivact-interaction\src\edit\handle.rs` (method in `impl<T: Clone + 'static> EditSession<T>`).
  - Its event lives in `W:\w\kivact\crates\kivact-interaction\src\edit\machine.rs` (`Event::Replace`).
  - The facade re-exports it as `kivact::interaction::EditSession`.
  - Land the kivact-interaction part before the kivact-ui-creative part (`outliner/controls.rs`, `outliner/view.rs`); the creative part does not compile without it.
- Site:
  - `kivact-ui-creative/src/outliner/controls.rs:10, 42-110, 144-160`.
  - `kivact-ui-creative/src/outliner/view.rs:738-753`.
  - `kivact-interaction/src/edit/machine.rs:33-51`.
  - `kivact-interaction/src/edit/handle.rs` (new method).
- Current fields (verbatim):

```rust
pub(super) struct Painter {
    paint: Signal<Option<Paint>>,
    session: EditSession<()>,
}
```

- Why implicit: (b)/(S-3). `paint` is `Some` exactly while the session is in `Editing` from a paint: an Option kept in lockstep with another owner's phase.
- Replacement, `kivact-interaction/src/edit/machine.rs`: add the event and arm.

```rust
pub(super) enum Event<T> {
    Begin(Rc<T>),
    /// Replace the payload of the active edit.
    Replace(Rc<T>),
    Finish,
}
// in transition, before `_ => Step::stay()`:
            (EditSessionState::Editing { .. }, Event::Replace(initial)) => {
                Step::to(EditSessionState::Editing {
                    initial: Initial(initial),
                })
            }
```

- Replacement, `kivact-interaction/src/edit/handle.rs`: add to `impl<T: Clone + 'static> EditSession<T>`, after `with_initial`.

```rust
    /// Replace the value the active edit holds with `value`; a later `cancel`
    /// passes `value` to `restore`, and `with_initial` reads it. For a
    /// gesture whose growing data is the edit's payload and whose `restore`
    /// ignores it (a paint across rows). Does nothing without an active edit.
    pub fn replace_initial(self, value: T) {
        if self.is_active() {
            self.session.send(machine::Event::Replace(Rc::new(value)));
        }
    }
```

- Replacement, `controls.rs`:
  - Line 10 becomes `use kivact::state::{Binding, Callback, Value};` (drop `Signal`).
  - Lines 42-110 (the `Painter` struct and impl) become:

```rust
/// One restriction paint: a drag across rows that sets one flag on each row
/// it touches, as one edit. The paint is the payload of the edit's active
/// phase, so it exists exactly while the edit does.
#[derive(Clone, Copy)]
pub(super) struct Painter {
    session: EditSession<Paint>,
}

impl Painter {
    pub(super) fn new(session: EditSession<Paint>) -> Self {
        Painter { session }
    }

    /// Begin the paint and propose its first rows; nothing happens (and
    /// `false` returns) while an edit runs.
    fn start(self, context: &Context, paint: Paint) -> bool {
        if self.session.is_active() {
            return false;
        }
        self.session.begin(paint.clone());
        propose(context, paint.column, &paint.touched, paint.value);
        true
    }

    /// The pointer is over the row `over`: add its rows and propose again.
    fn extend(self, context: &Context, over: u64) {
        let Some(mut paint) = self.session.with_initial(Paint::clone) else {
            return;
        };
        if paint.touched.contains(&over) {
            return;
        }
        for id in ids(context, over) {
            if !paint.touched.contains(&id) {
                paint.touched.push(id);
            }
        }
        propose(context, paint.column, &paint.touched, paint.value);
        self.session.replace_initial(paint);
    }

    /// The button is released: end the paint and commit its edit.
    fn finish(self) {
        self.session.commit();
    }

    /// End the paint and cancel its edit; whether a paint ran.
    pub(super) fn cancel(self) -> bool {
        self.session.cancel()
    }

    /// A single toggle (a key or an assistive action) as one edit.
    fn once(self, paint: Paint, write: impl FnOnce()) {
        self.session.once(paint, write);
    }
}
```

  - Lines 144-160 (`toggle`): replace the body after `if let Some(item) = ... {`:

```rust
        let ids = ids(context, id);
        let value = !self::flag(&item, column);
        let paint = Paint {
            column,
            value,
            touched: ids.clone(),
        };
        context
            .painter
            .once(paint, || propose(context, column, &ids, value));
```

- Replacement, `view.rs:742`: `EditSession::new(|()| {}),` becomes `EditSession::new(|_: super::controls::Paint| {}),`. The `observed(..)` helper is generic, so it is unchanged.
- Every read and write of the old field:
  - controls.rs:47 (field) and 54 (init).
  - controls.rs:62/67 (`start`) become `session.is_active()` / `begin(paint)`.
  - controls.rs:73/85 (`extend`) become `with_initial(Paint::clone)` / `replace_initial(paint)`.
  - controls.rs:90-91 (`finish`) become `commit()`, which is a no-op without an edit.
  - controls.rs:98-99 (`cancel`) become `cancel()`, which returns whether an edit was active.
  - Callers controls.rs:207, 221, 224, 226 and view.rs:205 are unchanged.
- xtask-allow lines: none.
- Public signatures: added `EditSession::replace_initial(self, value: T)` (new public method in kivact-interaction, re-exported as `kivact::interaction::EditSession`). Nothing else changes.
- Behaviour: unchanged. The Begin, Request, Commit and Cancel order is the same; `once` joins an active paint as before.
- Tests: `crates/kivact-ui-creative/tests/consumer/outliner.rs` (the app outliner installs `on_restriction`). No test drives a paint drag.
- New E2E sketches:
  - T8.2 appends this one to the existing file `dev/tests/kivact-public-api/tests/e2e/interaction_edit.rs`, for the new method (it uses `button` from the prelude and the file's own `mount_with` and `real_window`; add `use kivact::interaction::EditSession;` to the file's imports):

```rust
#[test]
fn replace_initial_changes_what_the_active_edit_holds_and_cancel_restores() {
    let (screen, value) = mount_with(real_window(), || {
        let value = signal(0);
        let edit = EditSession::new(move |restored: i32| value.set(restored));
        (
            value,
            column((
                button("Begin").on_press(move || {
                    edit.begin(1);
                    edit.replace_initial(5);
                }),
                button("Cancel").on_press(move || {
                    edit.cancel();
                }),
                text(move || format!("{:?}", edit.with_initial(|v| *v))).test_id("held"),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Begin").click();
    assert_eq!(screen.get_by_test_id("held").text(), "Some(5)");
    screen.get_by_role(Role::Button, "Cancel").click();
    assert_eq!(value.get_untracked(), 5);
    assert_eq!(screen.get_by_test_id("held").text(), "None");
}
```

  - `crates/kivact-ui-creative/tests/consumer/outliner.rs`, for the paint (uses `app_outliner` with an event log): press the visibility toggle of row 1, `mouse_move` over rows 2 and 3, release. Expect exactly `[Begin, Request(ids [1]), Request(ids [1, 2]), Request(ids [1, 2, 3]), Commit]`. Repeat with Escape before release and expect `.., Cancel`.

### SM-INT-15 — palette Open/Closed machine duplicates the caller's `open` (former D7)

- Site: `kivact-interaction/src/palette/machine.rs` (whole file), `kivact-interaction/src/palette.rs:1-3`, `kivact-interaction/src/palette/handle.rs:21, 336-347`.
- Current code (verbatim, handle.rs:336-347):

```rust
    let session = kivact_core::state::machine(
        machine::State::Open,
        || machine::Machine,
        move |_, ()| combobox.reset(),
    );
    session.feed(open.into_value(), |open| {
        if open {
            machine::Event::Open
        } else {
            machine::Event::Close
        }
    });
```

- Why implicit: (h). The machine's `Open`/`Closed` state is a second copy of the caller's `open` binding. It starts `Open` even when `open` starts `false`, and it exists only to detect a change to closed.
- Replacement (handle.rs:336-347):

```rust
    // The caller's `open` is the only open state. Each change to closed
    // resets the query and the highlight; a notification whose value the
    // binding no longer holds is ignored.
    kivact_core::state::effect_on(open, move |&shown| {
        if !shown && !open.get_untracked() {
            combobox.reset();
        }
    });
```

  `combobox.reset()` only sends `Event::Reset` to the combobox machine (handle.rs of combobox:484-486). This is the same effect_on-plus-send shape `MachineHandle::feed` uses, so the effect-writes check is not triggered.
- Every read and write:
  - Delete the file `crates/kivact-interaction/src/palette/machine.rs`.
  - In `palette.rs`, delete line 3 `mod machine;`. Change line 1 to `//! Role: Expose the command palette behavior.`.
  - In `handle.rs`, delete line 21 `use super::machine;`.
  - Replace handle.rs:336-347 as above.
  - Nothing else referenced `machine::State`/`Event`/`Machine`.
- xtask-allow lines: none.
- Public signatures: unchanged.
- Behaviour: unchanged for every real open/close. A repeated `false` notification now resets again; the reset is idempotent (query `""`, no highlight), and the palette is closed then, so nobody can type in between.
- Tests: `dev/tests/kivact-public-api/tests/e2e/interaction_palette.rs::enter_runs_the_active_command_and_closes` and `ui_command_palette.rs::typing_filters_by_label_group_hint_and_keywords_and_enter_runs_and_closes`. No test reopens after a filtered close.
- New E2E test: T8.2 appends the sketch below to the existing file `dev/tests/kivact-public-api/tests/e2e/interaction_palette.rs` (it uses that file's own `page`, `Page::show`, `field`, `rendered`, `no_errors`; do not create a new file):

```rust
#[test]
fn closing_resets_the_query_so_a_reopened_palette_lists_its_commands_again() {
    let p = page();
    p.show();
    p.field().type_text("zz-no-match");
    assert!(p.rendered().is_empty());
    p.screen.press_key(Key::Escape);
    assert!(!p.open.get_untracked());
    p.open.set(false); // a repeated close notification only resets again
    p.show();
    assert_eq!(p.field().value(), "");
    assert!(!p.rendered().is_empty());
    p.no_errors();
}
```

### Note (outside D1–D7)

The tooltip mirror noted here earlier (`tooltip/api.rs:191`) is now prescribed as SM-INT-16 below. Its `Closing → Shown` concern does not arise: the tooltip's close delay is `Duration::ZERO`, so its delayed machine never enters `Closing`.

### SM-INT-16 — tooltip mirrors its delayed visibility into the App hub with an effect

- Depends on SM-INT-11: it uses the fourth `on_visible` parameter that SM-INT-11 adds to `crate::delayed::delayed`. Apply in the same change as SM-INT-11, or after it.
- Site: `kivact-interaction/src/tooltip/api.rs:26, 175-204`.
- Current code (verbatim, 175-204):

```rust
    // The delay, or none while another tooltip is warm; both end with the
    // engagement.
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
    );

    // One tooltip at a time: the one that was shown last is visible, and one
    // that another took over shows again when that one hides.
    effect(move || {
        if delayed.is_open().get() {
            hub.want(id);
        } else {
            hub.release(
                id,
                config
                    .get()
                    .ext::<crate::InteractionPolicy>()
                    .tooltip
                    .skip_delay,
            );
        }
    });
```

- Why implicit: (e). An effect copies the delayed machine's visibility (`is_open`) into the App hub machine, as `Want`/`Release` events, one flush after the fact. The same rule as SM-INT-11 applies: the delayed machine's own `Shown`/`Hidden` effects are the only writer, and nothing mirrors visibility in an effect.
  - The tooltip has no caller binding. Its inputs reach the delayed machine one way only: `engaged` through `delayed`'s existing `Event::Wanted` sender. That sender stays an effect, not `feed`, because `feed` skips the initial value. The hub is read back only as input (`warm_for` in the open-delay closure, `current()` in the `visible` memo). After the fix there is no two-way sync.
- Replacement (tooltip/api.rs:175-204):

```rust
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
                hub.release(
                    id,
                    config
                        .get_untracked()
                        .ext::<crate::InteractionPolicy>()
                        .tooltip
                        .skip_delay,
                );
            }
        },
    );
```

  This supersedes the tooltip bullet of SM-INT-11, which passes `|_| ()` here. Use this closure instead.
- Every read and write:
  - tooltip/api.rs:177-187 (`delayed(..)` call): the fourth argument is the closure above.
  - tooltip/api.rs:189-204 (comment and `effect(..)`): delete.
  - tooltip/api.rs:26: remove `effect` from `use kivact_core::state::{Signal, Value, effect, memo, on_cleanup, signal};` (line 191 was its only use).
  - Unchanged:
    - `hub.want` / `hub.release` (70-75).
    - `on_cleanup` release (206-217).
    - `visible` memo (205).
    - `dismiss` (269-271).
    - `warm_for` (62-66).
- xtask-allow lines: none.
- Public signatures: unchanged.
- Behaviour: unchanged.
  - The tooltip's close delay is `Duration::ZERO`, so `Closing` is never entered. `Effect::Shown` (`Hidden`/`Opening → Shown`) is then exactly the old `is_open` change false → true, and `Want` is sent then.
  - `Effect::Hidden` covers every old true → false change (`Shown → Hidden`, `Shown → Dismissed`). It also fires on false → false entries (`Opening → Hidden`, `Hidden ↔ Dismissed`, `Opening → Dismissed`). There the id is not on the hub stack, and `Release` of an absent id is `Step::stay()` (machine.rs:33-35).
  - The old effect's initial no-op `Release` (at setup) and its no-op re-runs on theme change disappear.
  - Timing: the hub now changes inside the delayed transition's effect step instead of one effect flush later.
- Tests:
  - `dev/tests/kivact-public-api/tests/e2e/interaction_tooltip.rs`: `hovering_shows_it_after_the_delay_and_leaving_hides_it_at_once`, `keyboard_focus_shows_it_and_losing_focus_hides_it`, `a_tooltip_taken_over_shows_again_when_the_other_hides` (stack and skip delay: "the warm one shows without a delay"), `the_overlay_is_a_named_tooltip_and_the_trigger_is_described_by_the_label`.
  - `dev/tests/kivact-public-api/tests/e2e/apg_tooltip.rs::tooltip_remains_open_while_pointer_is_over_tooltip`.
  - No new sketch: behaviour is unchanged.

### SM-INT-17 — kivact-style `Transform.stage` encodes an ordered phase in integer constants
- Owner: T4.2 (kivact-style). A new entry of the second review. T4.2's 担当ファイル must include `crates/kivact-style/src/transform.rs`. There is no `xtask-allow` line (the new rules do not flag it; it is a phase by S-1 and S-6).
- File: crates/kivact-style/src/transform.rs:98-100 (field), 128-134 (constants), 160, 192, 197-199, 280-282, 292-294, 304-306, 355-357, 367-369, 547.
- Current (verbatim): field `stage: u8,` (initial `stage: 0`), constants `const STAGE_RZ: u8 = 2; const STAGE_RY: u8 = 3; const STAGE_RX: u8 = 4; const STAGE_SKEW: u8 = 5; const STAGE_SCALE: u8 = 6;`, and `fn in_order(&self, stage: u8) -> bool { stage >= self.stage }`.
- Letter: S-1 and S-6. Six ordered states (none, then rotateZ, rotateY, rotateX, skew, scale) are integers; `in_order` compares them as numbers.
- Replacement:
```rust
/// Canonical stages, in the order of the canonical form: a function of a later
/// stage follows the earlier ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    /// No function set a part yet.
    None,
    RotateZ,
    RotateY,
    RotateX,
    Skew,
    Scale,
}
```
  (variant order is the comparison order; keep it.) Put it where the five constants are (delete them).
- Sites (all in transform.rs):
  - :100 `stage: u8,` becomes `stage: Stage,`; :160 `stage: 0,` becomes `stage: Stage::None,`; :547 `stage: STAGE_SCALE,` becomes `stage: Stage::Scale,`.
  - :192 `self.stage = STAGE_SCALE;` becomes `self.stage = Stage::Scale;`.
  - :197-199 `fn in_order(&self, stage: u8) -> bool { stage >= self.stage }` becomes `fn in_order(&self, stage: Stage) -> bool { stage >= self.stage }`.
  - :280 `self.in_order(STAGE_RZ)` / :282 `self.stage = STAGE_RZ;` become `Stage::RotateZ`; :292/:294 `STAGE_RY` become `Stage::RotateY`; :304/:306 `STAGE_RX` become `Stage::RotateX`; :355/:357 `STAGE_SCALE` become `Stage::Scale`; :367/:369 `STAGE_SKEW` become `Stage::Skew`.
  - `started: bool` stays (a translate-only transform has `started` and `Stage::None`).
- xtask-allow lines to delete: none.
- No public signatures change (`stage` is private, `Transform` keeps its derives).
- Tests: the unit tests of transform.rs and `dev/tests/kivact-public-api/tests/e2e` transform tests; behaviour is unchanged.

---

# Implicit state machine audit: kivact-core, kivact-engine, remaining crates, examples, dev

Base: branch feat/kashiwade/devtools-multi-window = main 96e22b6b. Read-only review. All paths are relative to `W:\w\kivact`.
Decisions recorded in ADR-06/07/09/17/18/19/21 (CloseSignal, HostPhase/AppPresence/Surface/LastPresent, Launch, FrameHealth, MotionEnv, AsyncCell, Runtime ext OnceCell, pointer press/capture four combinations) are kept as they are.

How "reviewed" is counted: one per `// xtask-allow(state-fields)` line (a struct-level allow counts once). FIX and HOST-REWRITE counts are allow lines that a fix replaces. UNANNOTATED-FIX counts are entries for sites that have no allow line.

## 1. Summary counts

| crate | reviewed | KEEP | FIX | HOST-REWRITE | UNANNOTATED-FIX |
|---|---|---|---|---|---|
| kivact-core | 40 | 36 | 4 (SM-CORE-01: 3 lines, SM-CORE-02: 1) | 0 | 6 (SM-CORE-08..12; SM-CORE-14 is HOST-REWRITE) |
| kivact-engine | 46 | 46 | 0 | 0 | 1 (SM-CORE-13) |
| arvane | 2 | 2 | 0 | 0 | 0 |
| arvane-shell | 5 | 5 | 0 | 0 | 0 |
| arvane-native | 6 | 5 | 0 | 1 (SM-CORE-05) | 1 (SM-CORE-16, HOST-REWRITE) |
| arvane-virtual | 1 | 0 | 0 | 1 (SM-CORE-06) | 0 |
| arvane-text | 6 | 6 | 0 | 0 | 0 |
| arvane-render | 2 | 2 | 0 | 0 | 0 |
| arvane-compose | 1 | 1 | 0 | 0 | 0 |
| arvane-asset | 1 | 1 | 0 | 0 | 0 |
| kivact-runtime | 8 | 7 | 1 (SM-CORE-03) | 0 | 0 |
| kivact-layout | 5 | 5 | 0 | 0 | 0 |
| kivact-test | 6 | 5 | 0 | 1 (SM-CORE-07) | 1 (SM-CORE-15, HOST-REWRITE) |
| kivact-editing | 3 | 3 | 0 | 0 | 0 |
| kivact-i18n | 2 | 2 | 0 | 0 | 0 |
| kivact-types | 2 | 2 | 0 | 0 | 0 |
| kivact-shader | 2 | 2 | 0 | 0 | 0 |
| kivact-gpu-share | 1 | 1 | 0 | 0 | 0 |
| examples | 19 | 19 | 0 | 0 | 0 |
| dev | 4 | 3 | 1 (SM-CORE-04) | 0 | 0 |
| **total** | **162** | **153** | **6 lines / 4 entries** | **3 lines / 3 entries** | **9 entries (3 HOST-REWRITE)** |

Entries by priority:
- Real implicit machines (phase-dependent Option, duplicated fact, sync flag): SM-CORE-01, 02, 05, 06, 09, 15.
- An epoch or a raw counter replaced by a named S-9 type or removed: SM-CORE-03, 08, 12, 13.
- An unnamed Sent or ReportOnce or Once replaced by the named type: SM-CORE-07, 10, 11, 16.
- A slot filled after construction (S-7): SM-CORE-04, 14 (and part of 06).

## 2. KEEP table

`file:line | struct.field | category`

### kivact-core (36)
| site | field | category |
|---|---|---|
| crates/kivact-core/src/client/trace.rs:20 | Sink.{stage,semantics,host_events,previous} | immutable config switches + diff cache (HOST-REWRITE module) |
| crates/kivact-core/src/custom/measurement.rs:17 | Constraints.{known_width,known_height} | borrowed input snapshot |
| crates/kivact-core/src/embed.rs:36 | Parent.window | construction option |
| crates/kivact-core/src/embed.rs:38 | Parent.scale | construction option |
| crates/kivact-core/src/gpu_interop/share/app_side.rs:49 | SharedImage.{current,pixels} | independent retained outputs (latest delivery) |
| crates/kivact-core/src/launch/settings.rs:16 | AppSettings.* | construction options (HOST-REWRITE module) |
| crates/kivact-core/src/view/drawable_share.rs:109 | ExportOpen.geometry | Mailbox |
| crates/kivact-core/src/view/drawable_share.rs:111 | ExportOpen.scale | setting |
| crates/kivact-core/src/view/drawable_share.rs:117 | ExportOpen.next_number | ID allocator |
| crates/kivact-core/src/view/editing_data.rs:82 | TextEditing.* | builder |
| crates/kivact-core/src/view/editing_data.rs:96 | EditorProps.* | builder/mount config |
| crates/kivact-core/src/view/elements/viewport.rs:31 | Viewport.* | NodeBuilder input |
| crates/kivact-core/src/view/element_data.rs:507 | Common.* | builder |
| crates/kivact-core/src/view/element_data.rs:751 | Constraints.* | read-only projections |
| crates/kivact-core/src/view/flow/branch.rs:146 | Show.* | builder |
| crates/kivact-core/src/view/flow/branch.rs:323 | ShowSome.* | builder |
| crates/kivact-core/src/view/flow/error_boundary.rs:34 | ErrorBoundary.* | builder |
| crates/kivact-core/src/view/flow/virtual_list.rs:62 | VirtualList.* | builder |
| crates/kivact-core/src/view/mount/create.rs:15 | FirstWrite.* | passive mount record |
| crates/kivact-core/src/view/mount/facts.rs:10 | NodeFacts.* | passive mount record |
| crates/kivact-core/src/view/mount/parts.rs:153 | ConstraintSources.* | passive mount input |
| crates/kivact-core/src/view/mount/part_input.rs:36 | Selected.* | per-event routing snapshot |
| crates/kivact-core/src/view/mount/resolve.rs:32 | Resolved.* | passive mount record |
| crates/kivact-core/src/view/mount/semantics.rs:118 | SemanticProps.* | mount config |
| crates/kivact-core/src/view/mount/style.rs:25 | States.* | reader handles |
| crates/kivact-core/src/view/mount/style.rs:71 | NodeStyle.* | mount config |
| crates/kivact-core/src/view/overlays.rs:502 | KeyRoute.* | per-dispatch result value |
| crates/kivact-core/src/view/part_data.rs:11 | PartMount.* | read-only hook argument |
| crates/kivact-core/src/view/part_data.rs:30 | PartConstraints.* | builder |
| crates/kivact-core/src/view/part_data.rs:48 | PartState.* | public observation snapshot |
| crates/kivact-core/src/view/semantics_data.rs:16 | SemanticsPatch.* | passive patch value |
| crates/kivact-core/src/view/state.rs:135 | FocusKeys.{node,selector,visible} | independent facts; selector is a projection |
| crates/kivact-core/src/view/state.rs:195 | ViewState.{initial_render,min_window_size} | independent single fact + config |
| crates/kivact-core/src/view/tree.rs:58 | Handlers.* | optional subscriptions (builder) |
| crates/kivact-core/src/view/tree.rs:184 | PartActions.* | builder |
| crates/kivact-core/src/view/tree.rs:281 | CoreNode.* | independent per-node resources |

### kivact-engine (46)
| site | field | category |
|---|---|---|
| crates/kivact-engine/src/a11y/state.rs:9 | Metadata.* | pure cache |
| crates/kivact-engine/src/check/boxes.rs:66 | Axes.{x,y} | independent geometric facts |
| crates/kivact-engine/src/host/pump.rs:60 | Host.next_wake | independent client deadline (HOST-REWRITE module) |
| crates/kivact-engine/src/host/pump.rs:70 | Host.reduced_motion_override | user setting (HOST-REWRITE module) |
| crates/kivact-engine/src/host/pump.rs:88 | Host.last_stats | statistics/delivery record (HOST-REWRITE module) |
| crates/kivact-engine/src/host/testing/test_host.rs:26 | Counts.turns | statistics (HOST-REWRITE module) |
| crates/kivact-engine/src/host/testing/test_host.rs:28 | Counts.frame_ticks | statistics (HOST-REWRITE module) |
| crates/kivact-engine/src/layer.rs:47 | Look.* | derived paint record (pure cache) |
| crates/kivact-engine/src/layer.rs:93 | SubtreeFacts.* | derived record |
| crates/kivact-engine/src/lower/context.rs:45 | Context.* | operation-scoped borrows + statistics |
| crates/kivact-engine/src/lower/glyphs.rs:25 | Colors.* | drawing options |
| crates/kivact-engine/src/measure/editor.rs:14 | EditorNode.lines | pure cache |
| crates/kivact-engine/src/measure/editor.rs:18 | EditorNode.measure_lines | pure cache |
| crates/kivact-engine/src/measure/state.rs:13 | MeasureColumns.* | pure caches |
| crates/kivact-engine/src/measure/state.rs:47 | TextExtents.* | pure caches |
| crates/kivact-engine/src/model/aggregate.rs:18 | Model.{editors,window_request,...} | payload collections |
| crates/kivact-engine/src/model/changes.rs:124 | Changes.{paint,restack,fonts} | dirty set (invalidation facts) |
| crates/kivact-engine/src/model/changes.rs:139 | Dispatched.* | consumed work record |
| crates/kivact-engine/src/model/drawables/place.rs:60 | LayerFrame.own | geometric value |
| crates/kivact-engine/src/model/drawables/place.rs:63 | LayerFrame.context | geometric value |
| crates/kivact-engine/src/model/drawables/place.rs:66 | LayerFrame.in_3d | walk fact |
| crates/kivact-engine/src/model/drawables/place.rs:69 | LayerFrame.backface | setting |
| crates/kivact-engine/src/model/drawables/place.rs:72 | LayerFrame.preserves | setting |
| crates/kivact-engine/src/model/drawables/state.rs:117 | Drawables.noted | dirty set |
| crates/kivact-engine/src/model/edit/state.rs:7 | InputSource.* | cache key |
| crates/kivact-engine/src/model/edit/state.rs:23 | SessionSource.* | cache key |
| crates/kivact-engine/src/model/lifecycle.rs:48 | RemovedPayload.* | drop-order payload |
| crates/kivact-engine/src/model/pointer/state.rs:16 | Pointer.hover_sent | Sent |
| crates/kivact-engine/src/model/pointer/state.rs:18 | Pointer.position_sent | Sent |
| crates/kivact-engine/src/model/pointer/state.rs:93 | MouseTrack.over | independent input fact (fact-owner table) |
| crates/kivact-engine/src/model/pointer/state.rs:103 | TouchTrack.over | independent input fact |
| crates/kivact-engine/src/model/pointer/state.rs:113 | PenTrack.over | independent input fact |
| crates/kivact-engine/src/model/pointer/state.rs:152 | Route.capture | independent fact (press/capture: four valid combinations) |
| crates/kivact-engine/src/model/pointer/state.rs:380 | PointerSource.pressure | sample data |
| crates/kivact-engine/src/model/pointer/state.rs:383 | PointerSource.tilt | sample data |
| crates/kivact-engine/src/mutation.rs:93 | Mutation.* | report value |
| crates/kivact-engine/src/paint/state.rs:51 | PaintState.* | pure caches + report dedup |
| crates/kivact-engine/src/paint/state.rs:161 | Step.* | passive record |
| crates/kivact-engine/src/text_input/edit_lines.rs:11 | EditLines.damage | consumed cache payload |
| crates/kivact-engine/src/text_input/records.rs:18 | Key.* | cache key |
| crates/kivact-engine/src/text_input/records.rs:66 | Kept.* | pure caches (run_id is an identity) |
| crates/kivact-engine/src/text_input/source.rs:5 | Source.* | borrowed snapshot |
| crates/kivact-engine/src/tree/child_order.rs:6 | Link.* | topology links |
| crates/kivact-engine/src/tree/child_order.rs:13 | Editing.* | topology endpoints |
| crates/kivact-engine/src/tree/child_order.rs:179 | Children.* | iterator cursors |
| crates/kivact-engine/src/tree/node.rs:113 | Place.* | derived cache |

### remaining crates (49)
| site | field | category |
|---|---|---|
| crates/arvane/src/window.rs:180 | Shell.{prefer_gpu,gpu_preference,observer} | construction options |
| crates/arvane/src/window.rs:238 | ShellSetup.* | construction options |
| crates/arvane-asset/src/store/entry.rs:123 | VariantKey.* | cache key |
| crates/arvane-compose/src/gpu/state.rs:19 | GpuEnv.* | construction options |
| crates/arvane-native/src/mirror.rs:111 | MirrorBatch.* | Mailbox payload (HOST-REWRITE module) |
| crates/arvane-native/src/native_input/state.rs:51 | ContactNote.* | platform snapshot |
| crates/arvane-native/src/port/win32/state.rs:57 | Utf16Decoder.{pending,tail} | stream decoder buffers (HOST-REWRITE module) |
| crates/arvane-native/src/port/winit/runtime.rs:88 | WinitWindow.{subclass,monitor} | RAII guard + monitor cache (HOST-REWRITE module) |
| crates/arvane-native/src/port/x11/ime.rs:102 | XimConnection.{pending,flight} | key queue + self-contained in-flight record beside explicit Connection/Context (HOST-REWRITE module) |
| crates/arvane-render/src/draw/canvas.rs:67 | GlyphRun.{hint,cached} | drawing options |
| crates/arvane-render/src/paint.rs:43 | PatternFill.{repeat_x,repeat_y} | drawing options |
| crates/arvane-shell/src/a11y/shared.rs:148 | PlatformUpdate.{focus,all_runs} | borrowed snapshot |
| crates/arvane-shell/src/clicks.rs:37 | ClickCounter.* | independent histories |
| crates/arvane-shell/src/ime.rs:5 | ShellImeText.* | snapshot value |
| crates/arvane-shell/src/pipeline/ime.rs:12 | ImeDriver.* | Sent |
| crates/arvane-shell/src/pipeline/raw.rs:10 | RawKey.* | input record |
| crates/arvane-text/src/cache.rs:81 | SpanBits.* | cache key |
| crates/arvane-text/src/cache.rs:104 | RichKey.* | cache key |
| crates/arvane-text/src/cache.rs:122 | TextKey.* | cache key |
| crates/arvane-text/src/cache.rs:282 | Cache.* | pure cache |
| crates/arvane-text/src/layouts.rs:41 | SpanKey.* | cache key |
| crates/arvane-text/src/layouts.rs:104 | Key.* | cache key |
| crates/kivact-editing/src/behavior.rs:30 | EditContext.* | borrowed snapshot |
| crates/kivact-editing/src/edit_state.rs:54 | EditState.damage | consumed cache payload |
| crates/kivact-editing/src/line_colors.rs:64 | LineStyle.* | passive value |
| crates/kivact-gpu-share/src/common.rs:8 | Identity.* | identity value |
| crates/kivact-i18n/src/format.rs:104 | PluralForms.* | passive value |
| crates/kivact-i18n/src/numbers.rs:148 | SymbolCapture.* | construction-time capture |
| crates/kivact-layout/src/columns.rs:47 | Counted.* | pure cache (topology-epoch keyed) |
| crates/kivact-layout/src/columns.rs:99 | NodeLayoutValues.* | pure caches |
| crates/kivact-layout/src/pass_state.rs:21 | LayoutState.{changed,epoch,fixed_cursor} | dirty fact + cache generation + cursor cache |
| crates/kivact-layout/src/style/node.rs:32 | NodeStyle.* | passive converted props |
| crates/kivact-layout/src/tree.rs:54 | LayoutPass.* | operation-scoped borrows |
| crates/kivact-runtime/src/motion/state.rs:62 | MotionEnv.* | readonly inputs (ADR-19) |
| crates/kivact-runtime/src/motion/state.rs:344 | LoopCell.{frames,gate,rate,motion} | count + gate + lazy memo (ADR-19) |
| crates/kivact-runtime/src/reactive/data.rs:191 | MemoCell.{value,eq} | cache + config |
| crates/kivact-runtime/src/reactive/data.rs:338 | OwnerEntry.{parent,on_error} | config; lifecycle in OwnerPhase |
| crates/kivact-runtime/src/reactive/data.rs:380 | Frame.* | execution-context value |
| crates/kivact-runtime/src/reactive/error.rs:95 | Inner.{at,source} | diagnostic facts |
| crates/kivact-runtime/src/reactive/host_types.rs:3 | Pending.* | report snapshot |
| crates/kivact-shader/src/entry.rs:16 | Interface.params_declared | independent fact |
| crates/kivact-shader/src/entry.rs:19 | Interface.takes_params | independent fact |
| crates/kivact-test/src/facts.rs:9 | Node.kind | per-query metadata |
| crates/kivact-test/src/facts.rs:12 | Node.parent | per-query metadata |
| crates/kivact-test/src/facts.rs:14 | Node.at | per-query metadata |
| crates/kivact-test/src/screen/editing_drop_probe.rs:6 | Probe.* | test diagnostics (HOST-REWRITE module) |
| crates/kivact-test/src/screen/model.rs:35 | Screen.last_contact | ID allocator (HOST-REWRITE module) |
| crates/kivact-types/src/gpu.rs:48 | GpuPreference.* | construction preference |
| crates/kivact-types/src/gpu.rs:68 | GpuAdapter.* | identity value |

### examples (19)
| site | field | category |
|---|---|---|
| examples/kivact-ui-creative/src/creative/audio_values.rs:14 | TransportState.playing | independent domain fact |
| examples/kivact-ui-creative/src/creative/audio_values.rs:16 | TransportState.recording | independent toggle (record arm) |
| examples/kivact-ui-creative/src/creative/audio_values.rs:18 | TransportState.cycling | setting |
| examples/kivact-ui-creative/src/creative/shell.rs:366 | CatalogNavigation.category | caller binding handle |
| examples/kivact-ui-creative/src/creative/shell.rs:370 | CatalogNavigation.rects | read-only measurements |
| examples/kivact-ui-creative/src/creative/shell.rs:816 | CatalogControls.compact | caller binding handle |
| examples/kivact-ui-creative/src/creative/shell.rs:818 | CatalogControls.category | caller binding handle |
| examples/kivact-ui-creative/src/creative/shell.rs:821 | CatalogControls.current | caller binding handle |
| examples/kivact-ui-creative/src/creative/shell.rs:823 | CatalogControls.sidebar_open | caller binding handle (independent of compact density) |
| examples/kivact-ui-creative/src/creative/shell.rs:827 | CatalogControls.search | caller binding handle |
| examples/kivact-ui-creative/src/creative/shell.rs:833 | CatalogControls.shading | caller binding handle |
| examples/kivact-ui-creative/src/creative/shell.rs:835 | CatalogControls.viewport_grid | caller binding handle |
| examples/kivact-ui-game/src/game/state.rs:7 | Demo.create | caller open binding |
| examples/kivact-ui-game/src/game/state.rs:9 | Demo.delete | caller open binding |
| examples/kivact-ui-game/src/game/state.rs:11 | Demo.drawer | caller open binding |
| examples/kivact-ui-game/src/game/state.rs:13 | Demo.palette | caller open binding |
| examples/kivact-ui-pop/src/pop/mission.rs:6 | Mission.* | passive catalog record |
| examples/kivact-ui-soft/src/model.rs:8 | CatalogState.dark | caller binding handle |
| examples/kivact-ui-soft/src/model.rs:10 | CatalogState.selected | caller binding handle |

### dev (3)
| site | field | category |
|---|---|---|
| dev/xtask/src/check_source.rs:15 | Source.target | parsed metadata |
| dev/xtask/src/check_source.rs:17 | Source.library | parsed metadata |
| dev/xtask/src/state_fields.rs:93 | (string literal in unit test) | test fixture, not an annotation |

Unannotated sites that were looked at and kept (no entry): engine `ControlledValue::InSync{at}` (an explicit enum), `Drawables.{asks,geometry,pools}` (u64 generations of the public export protocol), `A11yState.{constructed,published}` (testing statistics), `ShareLink.seen` (cross-thread delivery cursor), `ActionCell.last` (latest input, ADR-21 latest semantics), `Worker.stop`/`CancelToken`/`CloseSignal`/`ConnectionAlive` (single cross-thread cancel facts), arvane-native `module.rs REPORTED: AtomicBool` (a process-wide atomic ReportOnce. `Once` would deadlock on the reentrant call that the comment describes), `NativeInput.subclass_installed` (release latch shared with WM_NCDESTROY, separate from the RAII guard), layout `Epoch` (a cache generation; renaming it to `Generation` is vocabulary, not a state machine), `VirtualShell.redraw` (single pending-redraw fact consumed by Step::Redraw).

---

## 3. FIX and HOST-REWRITE entries (annotated sites)

### SM-CORE-01 — kivact-core drawable export: duplicated generation, phase-dependent latest frame, unnamed wake flag
- File: crates/kivact-core/src/view/drawable_share.rs:104-151 (struct + enum), methods 168-339, 393-397, 421-438.
- Current fields (verbatim):
```rust
pub(crate) struct ExportOpen {
    /// The current request and its generation.
    pub(crate) request: ExportRequest,
    // xtask-allow(state-fields): Mailbox: the latest geometry the app set, taken by the UI turn.
    pub(crate) geometry: Mailbox<Option<Matrix4>>,
    // xtask-allow(state-fields): The app's configured scale; a setting, not derived from the window.
    pub(crate) scale: Option<f64>,
    // xtask-allow(state-fields): Delivery record: the newest frame handed over to the app.
    pub(crate) latest: Option<DrawableFrame>,
    // xtask-allow(state-fields): ID allocator: the last generation issued.
    pub(crate) next_generation: u64,
    // xtask-allow(state-fields): ID allocator: the last frame number issued over the drawable's life.
    pub(crate) next_number: u64,
    // xtask-allow(state-fields): Wake marker: a call of the app the UI turn has not taken yet.
    pub(crate) changed: bool,
}
```
- Letters:
  - (h) `next_generation` is always equal to `request.generation()`. `restart` sets both, and every other transition keeps the generation.
  - (b) `latest` is `Some` only while `request` is `Pixels` or `GpuProvided`. `restart` clears it, and frames come only for those requests.
  - (c) `changed` is an unnamed take-once wake (Mailbox semantics). `restart`, `on_textures` and `on_frame` also write it, which is redundant: every UI-thread caller already calls `changed.notify()` (view/drawable/events.rs:21, 46, 65, 80, 95).
- Replacement (verbatim):
```rust
/// The state of an open export.
pub(crate) struct ExportOpen {
    /// The current request, its generation and, for the requests that
    /// receive frames, the newest frame.
    pub(crate) request: ExportRequest,
    // xtask-allow(state-fields): Mailbox: the latest geometry the app set, taken by the UI turn.
    pub(crate) geometry: Mailbox<Option<Matrix4>>,
    // xtask-allow(state-fields): The app's configured scale; a setting, not derived from the window.
    pub(crate) scale: Option<f64>,
    // xtask-allow(state-fields): ID allocator: the last frame number issued over the drawable's life.
    pub(crate) next_number: u64,
    /// Take-once wake of the UI task: an app call the UI turn has not taken yet.
    pub(crate) wake: Mailbox<()>,
}

/// The export request in force, with its generation.
pub(crate) enum ExportRequest {
    /// Nothing is exported.
    None { generation: u64 },
    /// Frames are read back as pixels; `latest` is the newest one received.
    Pixels {
        // xtask-allow(state-fields): S-8 ID - the generation of the app's request; a frame of another generation is dropped
        generation: u64,
        // xtask-allow(state-fields): delivery record - the newest frame received for this request
        latest: Option<DrawableFrame>,
    },
    /// The window has to make the pool; no GPU frames yet.
    #[cfg(feature = "gpu-interop")]
    GpuRequested {
        generation: u64,
        adapter: GpuAdapter,
    },
    /// The window has no GPU device; retried on `GpuExportAvailable`.
    #[cfg(feature = "gpu-interop")]
    GpuWaiting {
        generation: u64,
        adapter: GpuAdapter,
    },
    /// Textures made. `textures` is taken once by `wait`; `latest` is the newest GPU frame.
    #[cfg(feature = "gpu-interop")]
    GpuProvided {
        // xtask-allow(state-fields): S-8 ID - the generation of the app's request; a frame of another generation is dropped
        generation: u64,
        adapter: GpuAdapter,
        /// Take-once hand-over of the pool to `wait` (S-9 Mailbox, not an `Option` that is taken).
        textures: Mailbox<SharedTextures>,
        // xtask-allow(state-fields): delivery record - the newest frame received for this request
        latest: Option<DrawableFrame>,
    },
    /// The request failed; `wait` reports it.
    #[cfg(feature = "gpu-interop")]
    Failed { generation: u64, message: String },
}

impl ExportOpen {
    /// A new generation (the current one plus one) with `request` made from it;
    /// the latest frame stays with the old request.
    fn restart(&mut self, request: impl FnOnce(u64) -> ExportRequest) {
        let generation = self.request.generation() + 1;
        self.request = request(generation);
    }

    /// The newest frame of the current request, when it receives frames.
    fn latest(&self) -> Option<&DrawableFrame> {
        match &self.request {
            ExportRequest::Pixels { latest, .. } => latest.as_ref(),
            #[cfg(feature = "gpu-interop")]
            ExportRequest::GpuProvided { latest, .. } => latest.as_ref(),
            _ => None,
        }
    }

    /// Accept a frame the compositor exported when its generation is current
    /// and the current request receives frames.
    pub(crate) fn on_frame(&mut self, frame: ExportedFrame) {
        if frame.generation != self.current() {
            return;
        }
        let latest = match &mut self.request {
            ExportRequest::Pixels { latest, .. } => latest,
            #[cfg(feature = "gpu-interop")]
            ExportRequest::GpuProvided { latest, .. } => latest,
            _ => return,
        };
        self.next_number += 1;
        let (generation, number) = (frame.generation, self.next_number);
        *latest = Some(match frame.frame {
            ExportFrame::Pixels(pixels) => DrawableFrame::Pixels {
                generation,
                number,
                pixels,
            },
            ExportFrame::Gpu { slot, value } => DrawableFrame::Gpu {
                generation,
                number,
                slot,
                value,
            },
        });
    }
}
```
- Read/write sites (all in drawable_share.rs):
  - :10-11 module doc invariant: replace "(`next_generation` counts up and is never reused)" with "(`ExportOpen::restart` makes it the current request's generation plus one, so none is reused)".
  - :157 `Self::None { generation } | Self::Pixels { generation } => *generation,` becomes `Self::None { generation } | Self::Pixels { generation, .. } => *generation,`.
  - :170-175 `restart`: replace the body with the code above. This drops `next_generation += 1`, `latest = None` and `changed = true`.
  - new private fn `latest(&self)` placed after `restart`.
  - :227-232 `on_textures` Ok arm: `self.request = ExportRequest::GpuProvided { generation, adapter, textures: { let slot = Mailbox::new(); slot.put(textures); slot }, latest: None };`. Delete line 232 `self.changed = true;`.
  - :203-212 `take_textures`: no change (`textures.take()` is `Mailbox::take(&self)`, and the pattern `textures` binds the field through `&mut self.request`).
  - :277-297 `on_frame`: replace with the code above. Line 296 `self.changed = true;` goes away.
  - :310-320 `initial`: `ExportOpen { request: ExportRequest::None { generation: 0 }, geometry: Mailbox::default(), scale: None, next_number: 0, wake: Mailbox::default() }`.
  - :323-326 `take_changed`: body becomes `self.open_mut()?.wake.take()`.
  - :331-338 `DrawableShare::change`: replace `open.changed = true;` with `open.wake.put(());`.
  - :344 `request_pixels`: `open.restart(|generation| ExportRequest::Pixels { generation, latest: None });`.
  - :395 `DrawableShare::latest`: `ExportLink::Open(open) => open.latest().cloned(),`.
  - :434 `take_update`: `match open.latest() { Some(frame) if frame.number() > after => Some(Ok(ShareUpdate::Frame(frame.clone()))), _ => None }`.
  - No reader outside this file touches `latest`, `next_generation` or `ExportOpen.changed`. view/drawable/requests.rs:20 keeps calling `ExportLink::take_changed`.
- xtask-allow lines to delete: drawable_share.rs:113, :115, :119 (they go with their fields). Keep :109, :111, :117. The replacement code above carries four new field-level allows (`generation` and `latest` of `Pixels` and of `GpuProvided`): copy them as written (T1.1 counts `generation: u64` and `Option<DrawableFrame>` in one variant, so a variant without these lines is an error).
- Behaviour change: a frame of the current generation that arrives while the request is `None`, `GpuRequested`, `GpuWaiting` or `Failed` is dropped. Before, it was stored but could not be used. No public signature changes.
- Tests: dev/tests/kivact-public-api/tests/e2e/drawables.rs:579 `pixels_frames_of_old_generation_are_dropped`, :618 `wait_cursor_survives_generation_change`, dev/tests/kivact-public-api/tests/e2e/drawable_export_gpu.rs.

### SM-CORE-02 — kivact-core NodeObservation stores one scroll report three times
- File: crates/kivact-core/src/view/tree.rs:251-266 (struct), 33 (classify_scroll), 441-471 (creation/handler).
- Current fields (verbatim):
```rust
pub(crate) struct NodeObservation {
    pub(crate) rect: Signal<Option<crate::geometry::Rect>>,
    pub(crate) text_truncated: Signal<bool>,
    pub(crate) scroll: Signal<crate::geometry::Point>,
    pub(crate) scroll_state: Signal<super::node_status_data::ScrollState>,
    /// The offset, viewport and content extents of a scroll container.
    pub(crate) scroll_geometry: Signal<ScrollState>,
    /// The window layout box, without the node's own transform.
    pub(crate) layout_rect: Signal<Option<crate::geometry::Rect>>,
    pub(crate) image_status: Signal<kivact_types::ImageStatus>,
    pub(crate) scene_revision: Notifier,
    pub(crate) editing_revision: Notifier,
}
```
- Letter: (h). The scroll handler (tree.rs:465-472) writes `scroll = state.offset`, `scroll_state = classify_scroll(state)` and `scroll_geometry = state` from one report. Two of these are copies of the third. Every reader except one already reads through a `memo`, which keeps the narrow invalidation.
- Replacement (verbatim):
```rust
pub(crate) struct NodeObservation {
    // xtask-allow(state-fields): independent fact - the node's rectangle as the engine last reported it, absent before the first layout
    pub(crate) rect: Signal<Option<crate::geometry::Rect>>,
    // xtask-allow(state-fields): independent fact - whether the engine truncated the node's text
    pub(crate) text_truncated: Signal<bool>,
    /// The offset, viewport and content extents of a scroll container: the
    /// one scroll fact; the offset and the scroll position are projections.
    pub(crate) scroll_geometry: Signal<ScrollState>,
    // xtask-allow(state-fields): independent fact - the window layout box without the node's own transform, absent before the first layout
    /// The window layout box, without the node's own transform.
    pub(crate) layout_rect: Signal<Option<crate::geometry::Rect>>,
    pub(crate) image_status: Signal<kivact_types::ImageStatus>,
    pub(crate) scene_revision: Notifier,
    pub(crate) editing_revision: Notifier,
}
```
- Sites:
  - tree.rs:33 `fn classify_scroll(` becomes `pub(crate) fn classify_scroll(`.
  - tree.rs:445-448: delete the `scroll:` and `scroll_state:` initializers.
  - tree.rs:465-471: becomes
    ```rust
    let scroll_geometry = observation.scroll_geometry;
    handlers.observe_scroll(move |state| scroll_geometry.set(state));
    ```
  - crates/kivact-core/src/view/node_status.rs:18 becomes `node.observed_status(|observation| super::tree::classify_scroll(observation.scroll_geometry.get()))`. It stays inside the existing `memo`, and `.unwrap_or_default()` stays.
  - crates/kivact-core/src/view/node_ref.rs:747 becomes `.and_then(|observation| observation.scroll_geometry.try_get()).map(|state| state.offset)`. It stays inside the existing `memo`, and `.unwrap_or_default()` stays.
  - crates/kivact-core/src/view/mount/draw.rs:64 `observation.scroll.get();` becomes `observation.scroll_geometry.get();`. The render effect also reruns when the viewport or content extent changes without an offset change. That is acceptable, because draw.rs:63 already tracks `rect`.
- xtask-allow lines: the struct-level allow at tree.rs:251 is replaced by field-level allows on the three fields that count (T1.1 makes the struct-level one an error, and `scroll_geometry` is a second `Signal` of a different kind). T7.2 writes them in the replacement struct above:
  - `rect`: `// xtask-allow(state-fields): independent fact - the node's rectangle as the engine last reported it, absent before the first layout`
  - `text_truncated`: `// xtask-allow(state-fields): independent fact - whether the engine truncated the node's text`
  - `layout_rect`: `// xtask-allow(state-fields): independent fact - the window layout box without the node's own transform, absent before the first layout`
- No public signature changes.
- Tests: dev/tests/kivact-public-api/tests/e2e/node_ref.rs:12 `scroll_state_follows_the_scroll_position`, scroll_timeline.rs:45 `scroll_progress_tracks_offset`, touch_scroll.rs and flows.rs (`scroll_offset`).

### SM-CORE-03 — kivact-runtime scheduler epochs (write_epoch, stab_epoch)
- File: crates/kivact-runtime/src/reactive/data.rs:55-92 (Runtime), 132-151 (PassRuns), 225-228 (SourceNode.changed_at), 238-247 (EffectNode.pass_runs).
- Current fields (verbatim, the relevant part):
```rust
// xtask-allow(state-fields): Sequence, write and stabilization epochs are independent scheduler counters; turn and exit lifecycles have their own enums.
pub struct Runtime {
    ...
    pub(crate) seq: Cell<u64>,
    pub(crate) write_epoch: Cell<u64>,
    ...
    pub(crate) stab_epoch: Cell<u64>,
}
#[derive(Default)]
pub(crate) struct PassRuns {
    epoch: u64,
    runs: u16,
}
pub(crate) struct CycleDetected;
// SourceNode
    /// Write epoch of the last change.
    pub(crate) changed_at: u64,
// EffectNode
    /// Stabilisation pass (epoch) and the runs of this effect within it
    /// (per-effect cycle limit; a User effect never stabilises).
    pub(crate) pass_runs: PassRuns,
```
- Letters:
  - (d) `stab_epoch` and `PassRuns.epoch` exist only to reset a per-effect counter lazily when a new pass starts.
  - (d) `write_epoch` and `changed_at` are a raw u64 clock used to reject a stale effect run (`recheck_after_own_writes`). That is the S-9 Generation case, but it uses an unnamed counter.
- Replacement A (remove stab_epoch). In crates/kivact-runtime/src/reactive/turn.rs `stabilize` (lines 123-142), count runs per pass in a local map:
```rust
    pub fn stabilize(&self) {
        let mut runs: usize = 0;
        let limit = CYCLE_LIMIT * (self.graph.borrow().len() + 1);
        // Runs of each effect in this pass (the per-effect cycle limit).
        let mut own_runs: std::collections::HashMap<Key, usize> = std::collections::HashMap::new();
        loop {
            let next = self
                .sched
                .borrow_mut()
                .pop_render(&mut self.graph.borrow_mut());
            let Some(key) = next else { break };
            runs += 1;
            let own = own_runs.entry(key).or_insert(0);
            *own += 1;
            if *own > CYCLE_LIMIT || runs > limit {
                // unchanged body (abandon_render_effects, default_sink Cycle report, break)
            }
            self.run_effect(key);
        }
    }
```
  - Delete data.rs:91 `stab_epoch: Cell<u64>,` (and its doc line, if any).
  - Delete runtime.rs:46 `stab_epoch: Cell::new(0),`.
  - Delete data.rs:132-151 (`PassRuns`, `CycleDetected`, `impl PassRuns`).
  - Delete data.rs:244-246 (`pass_runs` and its doc).
  - Delete graph.rs:211 and :248 `pass_runs: PassRuns::default(),`.
  - Remove `PassRuns` from the import at graph.rs:37.
  - In turn.rs, keep `NodeData` in the import at :17 only if something still uses it (line 76 does).
  - `Key` is already imported at turn.rs:16 and derives `Hash`.
- Replacement B (write_epoch becomes the named Generation):
  - data.rs:70 `pub(crate) write_epoch: Cell<kivact_types::cell::Generation>,`.
  - data.rs:226-227 `/// Write generation of the last change.` and `pub(crate) changed_at: kivact_types::cell::Generation,`.
  - runtime.rs:26 `write_epoch: Cell::new(kivact_types::cell::Generation::new()),`.
  - graph.rs:167 `changed_at: kivact_types::cell::Generation::new(),`.
  - graph.rs:402-403:
    ```rust
    let mut write_epoch = self.write_epoch.get();
    let epoch = write_epoch.next();
    self.write_epoch.set(write_epoch);
    ```
  - graph.rs:409 `s.changed_at = epoch;` (unchanged text).
  - graph.rs:543 and :568: no textual change; the comparisons use PartialEq.
  - graph.rs:596 `fn recheck_after_own_writes(&self, key: Key, epoch_before: kivact_types::cell::Generation)`.
  - graph.rs:603 `s.changed_at > epoch_before` (unchanged text; Generation is Ord).
- xtask-allow: replace the data.rs:55 reason text with `// xtask-allow(state-fields): seq is the effect creation-order ID allocator and clock the runtime's time measurement; the write generation is a named Generation; turn and exit lifecycles have their own enums.` The line itself stays, because seq, clock and `later` still count.
- Behaviour: a nested `stabilize` no longer resets the outer pass's per-effect counts. That is stricter, and only reachable through misuse.
- No public signature changes.
- Tests: dev/tests/kivact-public-api/tests/e2e/runtime.rs (`diamond_is_glitch_free_and_runs_once` :57, `handler_writes_are_batched_into_one_effect_run_and_one_batch` :105, `effect_cleanup_runs_before_rerun_and_on_dispose_child_before_parent` :142), diagnostics.rs:125. No test found for the per-effect cycle limit; runtime.rs:270-274 only keeps a comment.

### SM-CORE-04 — dev gpu-interop-producer Vulkan built half-empty and filled later
- File: dev/tests/gpu-interop-producer/src/vulkan.rs:117-130 (struct), 185-283 (new/device/pool/open), 355-363 (draw), 386-446 (draw_with), 449-485 (Drop).
- Current fields (verbatim):
```rust
// xtask-allow(state-fields): Owned Vulkan resources; optional platform resources reflect independent enabled device extensions.
pub(crate) struct Vulkan {
    _entry: ash::Entry,
    instance: ash::Instance,
    physical: vk::PhysicalDevice,
    device: Option<ash::Device>,
    family: u32,
    queue: vk::Queue,
    width: u32,
    height: u32,
    slots: Vec<Slot>,
    commands: vk::CommandPool,
    pool: Option<Pool>,
}
```
- Letter: (f). `new` builds `Self` with `physical: null`, `device: None` and `pool: None`, then fills them. `device()` and `pool()` `expect` that they are filled. `draw` also takes the pool out and puts it back to split the borrow. The allow reason is wrong: these are not independent extensions.
- Replacement (verbatim types):
```rust
/// Everything but the texture pool; dropping it destroys the slots, the
/// command pool, the device and the instance.
struct Base {
    _entry: ash::Entry,
    instance: ash::Instance,
    physical: vk::PhysicalDevice,
    device: ash::Device,
    family: u32,
    queue: vk::Queue,
    width: u32,
    height: u32,
    slots: Vec<Slot>,
    commands: vk::CommandPool,
}

pub(crate) struct Vulkan {
    pool: std::mem::ManuallyDrop<Pool>,
    base: std::mem::ManuallyDrop<Base>,
}
```
- Steps:
  1. Add `fn open_device(instance: &ash::Instance, physical: vk::PhysicalDevice) -> Result<(ash::Device, u32, vk::Queue), String>`. Its body is the current `open` lines 247-273 (families, extension names, `create_device`, `get_device_queue`), returning `(device, family, queue)`.
  2. Add `impl Base { fn new(adapter: &GpuAdapter, (width, height): (u32, u32)) -> Result<Base, String> }`:
     - `let (entry, instance) = instance()?;`
     - `let physical = match unsafe { interop::find_physical_device(&instance, adapter) } { Ok(p) => p, Err(e) => { unsafe { instance.destroy_instance(None) }; return Err(e.to_string()); } };`
     - `let (device, family, queue) = match open_device(&instance, physical) { Ok(v) => v, Err(e) => { unsafe { instance.destroy_instance(None) }; return Err(e); } };`
     - `let mut base = Base { _entry: entry, instance, physical, device, family, queue, width, height, slots: Vec::new(), commands: vk::CommandPool::null() };`
     - `base.commands = unsafe { base.device.create_command_pool(&pool_info, None) }.map_err(failed("vkCreateCommandPool"))?;` (with `pool_info` from the current :276-278). Destroying a null pool in `Drop` is a Vulkan no-op.
     - `for _ in 0..POOL { base.slots.push(Slot::default()); base.make_slot()?; }`
     - `Ok(base)`.
  3. Move `make_slot` (:286-347) to `impl Base`. Change `let device = self.device().clone();` to `let device = self.device.clone();`.
  4. Move `draw_with` (:386-446) to `impl Base`, keeping its `device: &ash::Device, pool: &mut Pool, rgba` parameters. Inside it, `self.slots`, `self.width`, `self.height` and `self.queue` now refer to `Base`.
  5. `impl Drop for Base`: the body of the current Drop lines 467-484 without the fence wait and without `drop(self.pool.take())`. That is: destroy the slot fences, buffers and memory, then `destroy_command_pool(self.commands)`, `destroy_device`, `destroy_instance`.
  6. `impl Drop for Vulkan`: the fence wait from the current :451-462, using `self.base.device` and `self.base.slots`. If `!finished`, `return` (this leaks the pool and base, as today). Otherwise `unsafe { std::mem::ManuallyDrop::drop(&mut self.pool); std::mem::ManuallyDrop::drop(&mut self.base); }`, with a SAFETY comment: "the fences finished; neither field is used again".
  7. `Vulkan::new`:
     - `let base = Base::new(adapter, (width, height))?;`
     - `let (pool, shared) = unsafe { Pool::new(&base.instance, base.physical, &base.device, base.family, adapter, config) }.map_err(|e| e.to_string())?;` (on error `base` drops and cleans up).
     - `Ok((Vulkan { pool: std::mem::ManuallyDrop::new(pool), base: std::mem::ManuallyDrop::new(base) }, shared))`.
  8. Delete `fn device()` (:235-239), `fn pool()` (:241-245) and `fn open()` (:247-283).
     - Replace `self.pool()` (:367, :371, :377) with `self.pool`, which derefs to `Pool`.
     - Replace `self.queue` (:377) with `self.base.queue`.
     - Replace `prepare(&self.instance, self.device(), release)` (:381) with `prepare(&self.base.instance, &self.base.device, release)`.
     - Change `pitch` (:352) to use `self.base.width`.
  9. `draw` (:355-363):
     ```rust
     fn draw(&mut self, rgba: &[u8]) -> Result<FrameReady, String> {
         let device = self.base.device.clone();
         self.base.draw_with(&device, &mut self.pool, rgba)
     }
     ```
     `&mut self.pool` coerces to `&mut Pool` through DerefMut, and `self.base` and `self.pool` are disjoint fields.
- xtask-allow line to delete: vulkan.rs:117. Neither new struct has a bool or Option field.
- No public signature changes (crate-private dev tool).
- Tests: dev/tests/gpu-interop-producer/tests/read_back.rs, dev/tests/kivact-public-api/tests/e2e/gpu_interop.rs, viewport_gpu_interop.rs (Vulkan path, on hardware only).

### SM-CORE-05 — HOST-REWRITE — arvane-native mirror Entry (gpu/frame/owner)
- File: crates/arvane-native/src/mirror.rs:392-404 (struct), 417-457 (open), 459-494 (create), 510-541 (flush), 543-562 (close), 621-650 (blit).
- Current fields (verbatim):
```rust
// xtask-allow(state-fields): Native mirror facts: GPU presentation, last CPU frame, and optional test ownership vary independently when windows are pooled.
/// A native window on the UI thread.
struct Entry {
    window: Arc<Window>,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    /// A GPU surface presents to it: frames are not drawn here, and the
    /// window is destroyed, not pooled, when its test closes it.
    gpu: bool,
    accessibility: NativeAccessibility,
    frame: Option<Frame>,
    /// The test window it shows (`None` while pooled).
    owner: Option<(u64, Slot)>,
}
```
- Letter: (a). The three fields form one lifecycle:
  - Pooled: `owner None`, `frame None`, `gpu false`.
  - Owned on the CPU: `owner Some`, `frame` optional.
  - Owned on the GPU: `owner Some`, frame unused, never pooled again.
  - `gpu` with `owner None` is impossible.
  The u64 in `owner` is never read; the `tests` map holds the id.
- Target state (verbatim):
```rust
/// What a native window of the display shows.
enum Use {
    /// In the pool, showing the idle tree.
    Pooled,
    /// A test window's, drawn here on the CPU; `frame` is the last frame it was given.
    Cpu { slot: Slot, frame: Option<Frame> },
    /// A test window's; a GPU surface presents to it, so its test's close destroys it.
    Gpu { slot: Slot },
}

/// A native window on the UI thread.
struct Entry {
    window: Arc<Window>,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    accessibility: NativeAccessibility,
    usage: Use,
}
```
- Sites:
  - :453-454 `entry.usage = Use::Cpu { slot, frame: None };`
  - :486-493 `Entry { window, surface, accessibility, usage: Use::Pooled }`
  - :514 `let (Use::Cpu { slot, .. } | Use::Gpu { slot }) = &entry.usage else { return };`
  - :533 `if pending.gpu && let Use::Cpu { slot, .. } = &entry.usage { let slot = slot.clone(); entry.usage = Use::Gpu { slot }; }`
  - :536-541 `if let Some(frame) = pending.frame { if let Use::Cpu { frame: shown, .. } = &mut entry.usage { *shown = Some(frame); } blit(entry); self.counters.presented.fetch_add(1, Ordering::Relaxed); }`
  - :550 `if matches!(entry.usage, Use::Gpu { .. }) {`
  - :555-556 `entry.usage = Use::Pooled;`
  - :622-637 `let frame = match &entry.usage { Use::Gpu { .. } => return, Use::Cpu { frame, .. } => frame.as_ref(), Use::Pooled => None };` then `match frame {` (the arms keep `Some(frame) if ...`).
- xtask-allow line to delete: mirror.rs:392.
- Tests: crates/kivact-test/src/screen/real_window_tests.rs, crates/kivact-test/src/screen/gpu_tests.rs, crates/arvane-native/tests/smoke.rs.

### SM-CORE-06 — HOST-REWRITE — arvane-virtual VirtualShell frame/cpu_frame and real/headless
- File: crates/arvane-virtual/src/window.rs:87-129 (struct), 140-145, 228-231, 259-263, 274, 321-330, 335, 343-386; crates/arvane-virtual/src/backend.rs:64-73, 105, 111, 328, 389, 429, 444.
- Current fields (verbatim, the relevant part):
```rust
    pub(crate) frame: Option<(u32, u32, Arc<[u32]>)>,
    /// Frames are drawn on the GPU.
    pub(crate) gpu: bool,
    pub(crate) real: Option<Box<dyn Mirror>>,
    /// The accessibility tree without a real window.
    pub(crate) headless: Accessibility,
    ...
    pub(crate) cpu_frame: Cell<bool>,
```
- Letters:
  - (a) `frame` + `cpu_frame`: `cpu_frame` true with `frame None` is impossible. `cpu_frame` means "frame is still on the CPU surface". The native ports model the same thing with `CpuSurface::{Released,Ready,Shown}` (arvane-native/src/present.rs:8).
  - (b)/(f) `headless` is valid only while `real` is `None`, and `real` is filled in `start` (backend.rs:105) after construction.
- Target state (verbatim):
```rust
/// A frame the window keeps: physical size and XRGB pixels.
pub(crate) struct Kept {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Arc<[u32]>,
}

/// What the window shows of the frames presented to it.
pub(crate) enum Shown {
    Nothing,
    /// The last CPU presentation, still on the CPU surface: a damaged presentation starts from it.
    OnCpuSurface(Kept),
    /// A kept frame: a GPU frame read back, or the last CPU frame after the CPU surface was released.
    Kept(Kept),
}

/// Where the window's accessibility tree and frames go.
pub(crate) enum Display {
    Headless(Accessibility),
    Mirrored(Box<dyn Mirror>),
}
```
- Fields: replace `frame` and `cpu_frame` with `pub(crate) shown: Shown`. Replace `real` and `headless` with `pub(crate) display: Display`.
- Sites:
  - window.rs:142-144 `accessibility()`: `match &self.display { Display::Headless(a) => a, Display::Mirrored(real) => real.accessibility() }`.
  - window.rs:230 `keep_frame`: `self.shown = Shown::Kept(Kept { width: frame.width, height: frame.height, pixels: frame.to_softbuffer().into() });`.
  - window.rs:261 `native_accessibility: matches!(self.display, Display::Mirrored(_)),`.
  - window.rs:274, :327, :380 `if let Display::Mirrored(real) = &self.display {`.
  - window.rs:335 `match &self.display { Display::Mirrored(real) => real.with_gpu_target(make), Display::Headless(_) => Err(...) }`.
  - window.rs:322-323 `matches!(self.shown, Shown::OnCpuSurface(_))`.
  - window.rs:326 `self.shown = match std::mem::replace(&mut self.shown, Shown::Nothing) { Shown::OnCpuSurface(kept) | Shown::Kept(kept) => Shown::Kept(kept), Shown::Nothing => Shown::Nothing };`
  - window.rs:363-371:
    ```rust
    let previous = match &self.shown {
        Shown::OnCpuSurface(kept) if (kept.width, kept.height) == (width, height) => Some(kept.pixels.to_vec()),
        _ => None,
    };
    let (mut pixels, age) = match (damage, previous) {
        (Some(_), Some(previous)) => (previous, BufferAge::Previous),
        _ => (vec![0; width as usize * height as usize], BufferAge::Unknown),
    };
    ```
  - window.rs:383-384 `self.shown = Shown::OnCpuSurface(Kept { width, height, pixels });`
  - backend.rs:64 and :73 become `shown: Shown::Nothing`.
  - backend.rs:66-67 become `display: Display::Headless(Accessibility::new())`.
  - backend.rs:105 `if let Some(mirror) = self.options.open_mirror()? { self.shell.borrow_mut().display = Display::Mirrored(mirror); }`. T5.1: `VirtualBackend::new` builds only `VirtualPlatform` and an empty shared window table (`Rc<RefCell<BTreeMap<WindowId, VirtualWindow>>>`, which `Probe` and `Running` also hold). The main `VirtualWindow` is built in `start` after the mirror is opened, with `display` already decided (`open_mirror(&options.main)`; `Display::Mirrored(mirror)` when one opened, otherwise `Display::Headless(Accessibility::new())`). No post-construction write of `display`. A window opened later (`open_window`) is built the same way. `Probe::window(id)` is `None` before `start` and after the window closes.
  - backend.rs:111, :429 `matches!(..display, Display::Mirrored(_))`.
  - backend.rs:328, :444 `if let Display::Mirrored(real) = &...display`.
  - backend.rs:389 `match &self.shell.borrow().shown { Shown::OnCpuSurface(k) | Shown::Kept(k) => Some((k.width, k.height, k.pixels.clone())), Shown::Nothing => None }`.
- xtask-allow: the struct-level allow at window.rs:87 goes away with the struct (T5.1 splits `VirtualShell` into `VirtualPlatform` and `VirtualWindow`). Its reasons move to field-level allows on the fields that remain (`fail_next`, `redraw`, `ime`, `ime_text`, `soft_keyboard`, `gpu`, `occluded`, `min_size`, `focus_requests`); the exact lines are in work order T5.1 step 6. The shown frame, mirror and surface lifecycles are modeled by `Shown`, `Display` and `TestSurface`, so those fields need no allow.
- Tests: crates/kivact-test/src/screen/scene_cache_tests.rs, gpu_tests.rs, accessibility_tests.rs, dev/tests/kivact-public-api/tests/e2e/compositor.rs (damage).

### SM-CORE-07 — HOST-REWRITE — kivact-test Screen.artifacts_written is an unnamed ReportOnce
- File: crates/kivact-test/src/screen/model.rs:27-29.
- Current (verbatim):
```rust
    // xtask-allow(state-fields): independent report-once flag for failure artifact publication.
    /// Failure artifacts were written (by a failing query).
    pub(super) artifacts_written: Cell<bool>,
```
- Letter: (c).
- Replacement:
```rust
    /// Failure artifacts were written (by a failing query); they are written once.
    pub(super) artifacts_written: kivact_types::cell::ReportOnce,
```
- Sites:
  - crates/kivact-test/src/screen/driver.rs:57 `artifacts_written: kivact_types::cell::ReportOnce::new(),`
  - crates/kivact-test/src/screen/artifacts.rs:192 `if !self.artifacts_written.note() { return None; }`
- xtask-allow line to delete: model.rs:27.
- Tests: no direct test found; any failing query in crates/kivact-test/src/screen/*_tests.rs goes through this path.

---

## 4. UNANNOTATED-FIX entries

### SM-CORE-08 — kivact-core SharedImage renewal: write-only Generation inside an unnamed Sent
- File: crates/kivact-core/src/gpu_interop/share/app_side.rs:9, 19, 21-47, 123-136.
- Current (verbatim):
```rust
enum Issue {
    NotIssued,
    Issued {
        mode: ShareMode,
        size: Size,
        generation: Generation,
    },
}
```
  `Issue::issue` advances `generation`, but nothing reads it: `matches` ignores it. `NotIssued` is replaced at once (line 124) and is never observed.
- Letters: (d) dead generation counter, and (c) an unnamed Sent.
- Replacement:
  - Delete lines 21-47 (`enum Issue` + impl).
  - Line 9 `use kivact_types::cell::Generation;` becomes `use kivact_types::cell::Sent;`.
  - Delete `type Size` (line 19) if the compiler reports it unused.
  - Lines 123-136:
```rust
    let mut sent = Sent::new();
    let _ = sent.update((mode, size));
    let issued = RefCell::new(sent);
    let renewing = image.clone();
    kivact_runtime::effect(move || {
        let mode = mode_of(adapter());
        let size = renewing.size.get();
        let lost = renewing.import_lost();
        let changed = issued.borrow_mut().update((mode.clone(), size)).is_some();
        if changed || lost {
            renewing.renew(mode, size);
        }
    });
```
  (`ShareMode` derives `Clone + PartialEq`.)
- No public signature changes.
- Tests: dev/tests/kivact-public-api/tests/e2e/gpu_interop.rs, viewport_gpu_interop.rs.

### SM-CORE-09 — kivact-core Adopted.generation duplicates Adopted.target.id
- File: crates/kivact-core/src/view/element_data.rs:641-647.
- Current (verbatim):
```rust
pub(super) struct Adopted {
    pub(super) owner: Owner,
    pub(super) common: Common,
    pub(super) target: Rc<Target>,
    pub(super) generation: Generation,
    pub(super) binding: Cell<RegionBinding>,
}
```
- Letter: (h). `generation` equals `target.id`'s `TargetId::Region { generation }`. It is read only by the `debug_assert!` at view/mount/regions.rs:251-253.
- Replacement: delete the field `pub(super) generation: Generation,` (element_data.rs:645).
- Sites:
  - view/mount/regions.rs:240: delete `generation,` from the `Adopted { .. }` literal. The local `generation` at :207 is still used for `TargetId::Region`.
  - view/mount/regions.rs:251-253: delete the `debug_assert!`.
- No public signature changes.
- Tests: dev/tests/kivact-public-api/tests/e2e/parts.rs, canvas.rs (regions).

### SM-CORE-10 — kivact-core drawable request dedup is an unnamed Sent
- File: crates/kivact-core/src/view/drawable/requests.rs:24-27, 35, 46-56; crates/kivact-core/src/view/drawable/handle.rs:86.
- Current (verbatim): `type Sent = Option<(ExportRequestMode, u64, Option<f64>)>;` with `if sent.borrow().as_ref() != Some(&now) { *sent.borrow_mut() = Some(now.clone()); ... }`.
- Letter: (c).
- Replacement:
  - requests.rs:27 `type Sent = kivact_types::cell::Sent<(ExportRequestMode, u64, Option<f64>)>;`
  - requests.rs:46-56:
```rust
    if let Some(now) = request
        && sent.borrow_mut().update(now.clone()).is_some()
    {
        let (mode, generation, scale) = now;
        writer.drawable_request(HostRequest::DrawableExport {
            id,
            generation,
            mode,
            scale,
        });
    }
```
  - handle.rs:86 `let sent = RefCell::new(kivact_types::cell::Sent::new());`
- No public signature changes.
- Tests: dev/tests/kivact-public-api/tests/e2e/drawables.rs, drawable_export_gpu.rs.

### SM-CORE-11 — kivact-core virtual_list scroll_to dedup is an unnamed Sent
- File: crates/kivact-core/src/view/flow/virtual_list.rs:835-841.
- Current (verbatim):
```rust
        let last: RefCell<Option<K>> = RefCell::new(None);
        cx.effect(EffectKind::Internal, at, move || {
            let request = scroll_to.get();
            let Some(rt) = weak.upgrade() else { return };
            if *last.borrow() == request {
                return;
            }
            last.borrow_mut().clone_from(&request);
```
- Letter: (c).
- Replacement:
```rust
        let last = RefCell::new(kivact_types::cell::Sent::<Option<K>>::new());
        cx.effect(EffectKind::Internal, at, move || {
            let request = scroll_to.get();
            let Some(rt) = weak.upgrade() else { return };
            if last.borrow_mut().update(request.clone()).is_none() {
                return;
            }
```
  The rest is unchanged. A first `None` still returns at `let Some(request) = request else { return };`.
- No public signature changes.
- Tests: dev/tests/kivact-public-api/tests/e2e/flows.rs (virtual_list scroll_to).

### SM-CORE-12 — kivact-core lifecycle registrations allocate a Generation nobody reads
- File: crates/kivact-core/src/platform/lifecycle.rs:13, 15-25, 40-46.
- Current (verbatim):
```rust
#[derive(Default)]
pub(crate) struct LifecycleState {
    generation: RefCell<Generation>,
    registrations: RefCell<Registry<Registration>>,
}

struct Registration {
    _id: Generation,
    owner: Owner,
    callback: Callback<LifecycleEvent>,
}
```
- Letter: (d). A counter is advanced but never compared.
- Replacement:
  - Delete `generation: RefCell<Generation>,` (:17) and `_id: Generation,` (:22).
  - Delete `let id = state.generation.borrow_mut().next();` (:40) and `_id: id,` (:42).
  - Line 13 becomes `use kivact_types::cell::Registry;`.
- No public signature changes.
- Tests: dev/tests/kivact-public-api/tests/e2e/lifecycle.rs:12, :76.

### SM-CORE-13 — kivact-engine asset revision is a raw epoch that syncs the scene cache
- File: crates/kivact-engine/src/lower/assets.rs:52, 71, 83-92; crates/kivact-engine/src/lower/scene_cache.rs:14, 21-26.
- Current (verbatim): `revision: u64,` with `fn changed(&mut self) { self.revision = self.revision.checked_add(1).expect("invariant: resource revision fits u64"); }`. SceneCache has `revision: u64,` and `pub(crate) fn revision(&mut self, revision: u64) { if self.revision != revision { self.clear(); self.revision = revision; } }`.
- Letter: (d). An unnamed counter invalidates a pure cache. The same struct already uses the named `Generation` for `fonts`.
- Replacement:
  - assets.rs:52 `revision: kivact_types::cell::Generation,`
  - assets.rs:71 `revision: kivact_types::cell::Generation::new(),`
  - assets.rs:83 `pub(crate) fn revision(&self) -> kivact_types::cell::Generation {`
  - assets.rs:87-92 `fn changed(&mut self) { self.revision.next(); }`. `Generation::next` panics on exhaustion, as the current `expect` does.
  - scene_cache.rs:14 `revision: kivact_types::cell::Generation,` (the `#[derive(Default)]` still works).
  - scene_cache.rs:21 `pub(crate) fn revision(&mut self, revision: kivact_types::cell::Generation) {`
  - Call sites lower/context.rs:98 and assets.rs:207, :272 are unchanged.
- No public signature changes.
- Tests: crates/kivact-test/src/screen/scene_cache_tests.rs, dev/tests/kivact-public-api/tests/e2e/assets.rs, launch_fonts.rs.

### SM-CORE-14 — HOST-REWRITE — kivact-core Trace.sink filled after construction
- Owners: T7.2 does the kivact-core part (`client/trace.rs`, `launch/inspect.rs`, `launch/runtime.rs`); T8.1 does the kivact-test part (`screen/driver.rs`). T7.2 leaves kivact-test not compiling until T8.1 (the shape of `ClientMode::Test` changes).
- File: crates/kivact-core/src/client/trace.rs:15-18, 33-65, 83-84, 92-93, 131-132; crates/kivact-core/src/launch/inspect.rs:50-53, 145; crates/kivact-core/src/launch/runtime.rs:143-147, 448; crates/kivact-test/src/screen/driver.rs:36, 42, 62-63.
- Current (verbatim):
```rust
#[derive(Default)]
pub(crate) struct Trace {
    sink: RefCell<Option<Sink>>,
}
```
  `Trace::default()` is installed at runtime construction, and `Inspect::open_trace(name)` fills it later from the test driver.
- Letter: (f), S-7.
- Target state:
```rust
pub(crate) struct Trace {
    /// `None`: tracing is off for this App (no KIVACT_TRACE, or not a test launch).
    sink: Option<RefCell<Sink>>,
}
impl Trace {
    /// The trace of an App whose test is `name`; opens the file when KIVACT_TRACE selects a stream.
    pub(crate) fn new(name: Option<&str>) -> Trace { /* body of today's `open`, returning the Sink */ }
}
```
- Changes (T7.2, kivact-core):
  - `ClientMode::Test(TestTiming)` (launch/runtime.rs:448) becomes `ClientMode::Test { timing: TestTiming, name: String }`. That is a doc(hidden) launch contract change, folded into the rewrite. The one match on it, launch/inspect.rs:145 `ClientMode::Test(TestTiming(timing)) => Some(timing.clone())`, becomes `ClientMode::Test { timing: TestTiming(timing), .. } => Some(timing.clone())`.
  - Construct the trace in launch/runtime.rs:143-147 as `Trace::new(match &self.mode { ClientMode::Test { name, .. } => Some(name.as_str()), _ => None })` instead of `Trace::default()`.
  - Delete `Inspect::open_trace` (inspect.rs:50-53).
  - Readers trace.rs:83/92/131: `let Some(sink) = &Trace::of(rt).sink else { return }; let mut sink = sink.borrow_mut();`.
- Changes (T8.1, kivact-test, `screen/driver.rs`):
  - :36 `let name = super::artifacts::test_name(at);` stays; :42 `ClientMode::Test(TestTiming(Rc::new(move |elapsed| { .. })))` becomes `ClientMode::Test { timing: TestTiming(Rc::new(move |elapsed| { .. })), name: name.clone() }`.
  - Delete the two lines :62-63 `#[cfg(debug_assertions)] s.inspect().open_trace(&s.name);`.
- Tests: no direct test found (debug-only trace file).

### SM-CORE-15 — HOST-REWRITE — kivact-test Screen.scale duplicates the window's scale
- Owners: T6.1 adds the engine method (`ArvaneTestHost::scale(window)`, below); T8.1 does the kivact-test part. T8.1 may not edit the engine, and T6.1 runs before it.
- File: crates/kivact-test/src/screen/model.rs:30-31.
- Current (verbatim):
```rust
    /// Physical pixels per logical pixel.
    pub(super) scale: Cell<f64>,
```
- Letter: (h). The virtual window's `Environment::scale_factor` (arvane-virtual backend.rs:39, :273) is the owner. `Screen::set_scale` writes both copies (controls.rs:141-142).
- Replacement: delete the field.
- Sites (T8.1, kivact-test):
  - driver.rs:38 `let scale = host_options.scale;`: delete.
  - driver.rs:58 `scale: Cell::new(scale),`: delete.
  - controls.rs:141 `self.scale.set(scale);`: delete.
  - artifacts.rs:109-111 `pub fn scale(&self) -> f64 { self.scale.get() }` becomes `pub fn scale(&self) -> f64 { self.host.borrow().scale(self.window).expect("invariant: the window of a Screen is open") }`, where `self.window` is the `WindowId` the Screen handle was made for (T8.1 step 1; the main handle is `WindowId::MAIN`).
- Site (T6.1, kivact-engine): add to crates/kivact-engine/src/host/testing/test_host.rs (next to `frame_period_ns`, :358):
```rust
    /// Physical pixels per logical pixel of the test window `window`, as the window reports it.
    /// `None` when the window is closed or was never opened.
    pub fn scale(&self, window: WindowId) -> Option<f64> {
        self.probe
            .window(window)
            .map(|probe| probe.environment().scale_factor)
    }
```
  `Probe::window(&self, id) -> Option<WindowProbe>` is the multi-window probe of T5.1 (`WindowProbe::environment()` exists there, the per-window form of today's `Probe::environment()`). This is a new public method on the doc-hidden `ArvaneTestHost`; the old one-window `ArvaneTestHost::scale` does not exist (only `Probe::scale`, backend.rs:437, which T5.1 makes per window).
- Tests: crates/kivact-test/src/screen/gpu_tests.rs, scene_cache_tests.rs, dev/tests/kivact-public-api/tests/e2e/drawables.rs (set_scale/scale).

### SM-CORE-16 — HOST-REWRITE — arvane-native Linux settings watcher start is an unnamed once-flag
- File: crates/arvane-native/src/port/linux_settings.rs:30-31, 59-67.
- Current (verbatim):
```rust
/// Set when the watcher thread is started, so that every loop shares one thread.
static STARTED: AtomicBool = AtomicBool::new(false);
...
    if !STARTED.swap(true, Ordering::AcqRel) {
        let spawned = std::thread::Builder::new()
            .name("arvane-settings".into())
            .spawn(run);
        if let Err(error) = spawned {
            tracing::debug!(%error, "settings watcher thread not started");
        }
    }
```
- Letter: (c).
- Replacement:
```rust
/// Starts the one watcher thread every loop shares.
static WATCHER: std::sync::Once = std::sync::Once::new();
...
    WATCHER.call_once(|| {
        let spawned = std::thread::Builder::new()
            .name("arvane-settings".into())
            .spawn(run);
        if let Err(error) = spawned {
            tracing::debug!(%error, "settings watcher thread not started");
        }
    });
```
  `AtomicBool` stays imported (REDUCED_MOTION). The closure does not reenter `watch`. `std::sync::Once` is not in clippy.toml's disallowed-types.
- Tests: no test found (Linux portal; WSLg only).

### SM-CORE-17 — kivact-runtime TimelineCell.events is a counter nobody reads (S-8: use a Notifier)
- Owner: T3.2 (kivact-runtime). A new entry of the second review; it has no `xtask-allow` line (T1.1 flags it once its rules are in: `phase: Signal<TimelinePhase>` beside `events: Signal<u64>`).
- File: crates/kivact-runtime/src/motion/timeline/api.rs:50-60 (watcher), 484-510 (struct and `new`), 648-651 (`announce`), 741-745 (crossed calls).
- Current fields (verbatim):
```rust
    /// Counts the events the watcher must run.
    events: Signal<u64>,
```
- Letter: (h) and S-8. The value of the counter is never read: `pending` (a `RefCell<Vec<Event>>`) holds the events, and the counter only wakes the watcher. The architecture says a counter is for counts, IDs and statistics; waking a watcher is a `Notifier` (kivact-runtime/src/reactive/trigger.rs).
- Replacement:
```rust
    /// Wakes the watcher that runs `pending`.
    events: Notifier,
```
- Sites (all in api.rs):
  - top imports: add `Notifier` to the `use crate::{ .. }` list (`crate::Notifier` is exported, lib.rs:26).
  - :509 `events: owner.signal(0, at),` becomes `events: owner.notifier(at),`.
  - :50-60 the watcher becomes
    ```rust
    let events = cell.events;
    let weak = Rc::downgrade(&cell);
    crate::reactive::watch(
        move || events.track(),
        EffectKind::Internal,
        move |_| {
            if let Some(cell) = weak.upgrade() {
                cell.run_pending();
            }
        },
    );
    ```
    (`watch`'s source may return `()`; the watcher runs its callback on every notification, it does not compare values: reactive/graph.rs:86-103.)
  - :650 in `announce`: `self.events.update(|n| *n += 1);` becomes `self.events.notify();`.
  - :744 in the crossed-calls path: `self.events.update(|n| *n += 1);` becomes `self.events.notify();`.
- xtask-allow lines to delete: none. `TimelineCell` is then `phase` (a phase) beside no counted field, so it needs no allow.
- No public signatures change (`TimelineCell` is `pub(crate)`).
- Tests: dev/tests/kivact-public-api/tests/e2e/ (the timeline calls and `on_complete` tests: `grep -rln "timeline()" dev/tests/kivact-public-api/tests/e2e`) and `crates/kivact-runtime/src/motion/timeline/` unit tests; behaviour is unchanged.

---

# 7. 新しい規則で初めて検出される既存の箇所（T15.4 が印を書く）

基準 commit の `// xtask-allow(state-fields)` が無く、作業指示書 T1.1 の規則（設計書 5.3 の 4〜7。型の数え方は設計書にある）で初めて error になる既存の箇所の全件。数えた方法: 設計書 5.3 の規則を library と binary の module（各 crate の `tests/` directory の外）の全 struct・enum の variant・static に当てた。T15.4 は、この表の行ごとに、**field の直前**（`///` と `#[..]` の行の上）に次の 1 行を書く。コードは変えない。

`// xtask-allow(state-fields): <表の「印の文」の列そのまま>`

規則:
- 印を書くのは表にある field だけ。同じ型の他の field（phase の field を含む）には書かない。
- 同じ行に複数の field がある variant（`FilterField::Number`）は、field ごとに行を分けてから印を書く。
- 表の `site` は基準 commit の行番号。ずれていたら、型名と field 名で探す。
- 「印の文」は `<分類> - <その field が何の事実か>` の形。分類は 7.4 の一覧にあるもの。

## 7.1 表

| site | 型.field | 分類 | 印の文 |
|---|---|---|---|
| crates/arvane-asset/src/store/asset_store.rs:51 | AssetStore.frame | measurement | measurement - the host's frame clock written by begin_frame; the cache protects what the current and the previous frame used |
| crates/arvane-asset/src/store/asset_store.rs:54 | AssetStore.next_id | S-8 ID | S-8 ID - the next image id |
| crates/arvane-asset/src/store/asset_store.rs:55 | AssetStore.next_generation | S-8 ID | S-8 ID - the next entry generation; a job result of another generation is dropped |
| crates/arvane-asset/src/store/entry.rs:50 | Entry.generation | S-8 ID | S-8 ID - the generation stamped on the entry; a job result names it |
| crates/arvane-asset/src/store/job.rs:57 | Job::Raster.generation | S-8 ID | S-8 ID - the entry generation the job was made for |
| crates/arvane-asset/src/store/job.rs:60 | Job::Raster.crop | passive record | passive record - the part of the SVG to rasterize, absent for all of it |
| crates/arvane-asset/src/store/job.rs:74 | Done::Raster.generation | S-8 ID | S-8 ID - the entry generation the job was made for |
| crates/arvane-asset/src/store/job.rs:76 | Done::Raster.crop | passive record | passive record - the part of the SVG that was rasterized, absent for all of it |
| crates/arvane-compose/src/plan/bounds.rs:26 | Kind::Group.clip | passive record | passive record - the group's clip bounds in push coordinates, absent when it has none |
| crates/arvane-compose/src/plan/bounds.rs:27 | Kind::Group.blur | passive record | passive record - the group's blur radius, absent when it has none |
| crates/arvane-compose/src/plan/ops.rs:146 | Source::External.cpu | passive record | passive record - the CPU pixels, present when the CPU draws the frame |
| crates/arvane-compose/src/plan/ops.rs:148 | Source::External.source | passive record | passive record - the source rectangle, absent for the whole frame |
| crates/arvane-native/src/mirror.rs:106 | Counters.created | statistics | statistics - native windows created |
| crates/arvane-native/src/mirror.rs:107 | Counters.reused | statistics | statistics - pooled windows handed to a new test |
| crates/arvane-native/src/mirror.rs:108 | Counters.presented | statistics | statistics - frames shown after coalescing |
| crates/arvane-native/src/native_input/state.rs:16 | NativeInput.text | independent fact | independent fact - the focused text input's state as the application last set it |
| crates/arvane-native/src/native_input/state.rs:31 | NativeInput.reduced_motion | measurement | measurement - the system reduce-motion preference as last read |
| crates/arvane-native/src/native_input/state.rs:35 | NativeInput.locale | measurement | measurement - the system locale as last read |
| crates/arvane-native/src/port/x11/dnd.rs:48 | Drag::Hovering.accepts | independent fact | independent fact - whether the source offers a URI list we take, decided at XdndEnter |
| crates/arvane-native/src/port/x11/dnd.rs:49 | Drag::Hovering.point | independent fact | independent fact - the last XdndPosition, absent before the first one |
| crates/arvane-native/src/port/x11/notifier.rs:10 | Notifier.reasons | mailbox | mailbox - wake reasons coalesced across threads until the UI loop takes them |
| crates/arvane-native/src/port/x11/notifier.rs:11 | Notifier.thread | owned resource | owned resource - the UI thread's join handle, stored once after the spawn and taken once at close |
| crates/arvane-shell/src/window.rs:218 | StepOutcome::Idle.next_deadline | passive record | passive record - the next timer deadline the backend reports |
| crates/arvane-shell/src/window.rs:219 | StepOutcome::Idle.redraw_requested | passive record | passive record - whether a redraw was asked during the turn |
| crates/arvane-shell/src/window.rs:220 | StepOutcome::Idle.woken | passive record | passive record - whether the turn handled a wake |
| crates/arvane/src/presentation.rs:37 | RendererChange::Gpu.configure | passive record | passive record - how long the first swapchain took when it was configured in the background, absent otherwise |
| crates/arvane/src/presentation.rs:42 | RendererChange::Gpu.interop | passive record | passive record - the adapter that can import shared textures, absent without gpu-interop |
| crates/kivact-core/src/gpu_interop/import/image.rs:32 | ExternalImage.latest | delivery record | delivery record - the newest frame a FramePublisher published, independent of the import status |
| crates/kivact-engine/src/a11y/state.rs:170 | SemanticsIdAllocator.next | S-8 ID | S-8 ID - the next synthetic semantics id, counting down |
| crates/kivact-engine/src/a11y/state.rs:172 | SemanticsIdAllocator.empty_root | S-8 ID | S-8 ID - the id reserved for the empty root, independent of the published tree |
| crates/kivact-engine/src/model/drawables/state.rs:25 | ExportAsk.generation | S-8 ID | S-8 ID - the generation of the app's request in the public export protocol |
| crates/kivact-engine/src/model/drawables/state.rs:29 | ExportAsk.scale | setting | setting - the scale the app set for the frames; absent follows the window |
| crates/kivact-gpu-share/src/pool.rs:132 | Slots.next | cursor | cursor - the texture of the next frame, round robin |
| crates/kivact-gpu-share/src/pool.rs:134 | Slots.last | S-8 ID | S-8 ID - the last frame value given out |
| crates/kivact-interaction/src/combobox/handle.rs:113 | ComboboxApi.input_state | independent fact | independent fact - the observation of the attached input part, absent until it attaches |
| crates/kivact-interaction/src/combobox/handle.rs:114 | ComboboxApi.content_state | independent fact | independent fact - the observation of the attached content part, absent until it attaches |
| crates/kivact-interaction/src/disclosure.rs:54 | DisclosureApi.trigger_state | independent fact | independent fact - the observation of the attached trigger part, absent until it attaches |
| crates/kivact-interaction/src/disclosure.rs:55 | DisclosureApi.content_state | independent fact | independent fact - the observation of the attached content part, absent until it attaches |
| crates/kivact-interaction/src/drag/handle.rs:144 | DragApi.observed | independent fact | independent fact - the observation of the attached source part, absent until it attaches |
| crates/kivact-interaction/src/drag/handle.rs:151 | DragApi.disabled | builder option | builder option - the disabled value a setter supplied, absent until set |
| crates/kivact-interaction/src/drag/handle.rs:111 | Step::Stop.owns_edit | passive record | passive record - whether the ended drag owned the edit session |
| crates/kivact-interaction/src/drag/handle.rs:112 | Step::Stop.update | passive record | passive record - what on_end receives, absent for an event with invalid coordinates |
| crates/kivact-interaction/src/edit/handle.rs:85 | EditSession.observer | builder option | builder option - the observer callback a setter supplied, absent until set |
| crates/kivact-interaction/src/popup/api.rs:103 | PopupApi.content_state | independent fact | independent fact - the observation of the attached content part, absent until it attaches |
| crates/kivact-interaction/src/popup/api.rs:104 | PopupApi.trigger_state | independent fact | independent fact - the observation of the attached trigger part, absent until it attaches |
| crates/kivact-interaction/src/select/handle.rs:132 | SelectApi.content_state | independent fact | independent fact - the observation of the attached content part, absent until it attaches |
| crates/kivact-interaction/src/select/handle.rs:136 | SelectApi.preview | builder option | builder option - the preview callback a setter supplied, absent until set |
| crates/kivact-interaction/src/select/machine.rs:19 | State::Open.highlight | independent fact | independent fact - the key the keyboard highlights in the open list |
| crates/kivact-interaction/src/select/machine.rs:20 | State::Open.hovered | independent fact | independent fact - the key the pointer is over in the open list |
| crates/kivact-interaction/src/table/model.rs:296 | DataTableModelApi.sort | builder option | builder option - the sort order a setter supplied, absent until set |
| crates/kivact-interaction/src/table/model.rs:297 | DataTableModelApi.query | builder option | builder option - the text query a setter supplied, absent until set |
| crates/kivact-interaction/src/table/model.rs:298 | DataTableModelApi.column_queries | builder option | builder option - the column queries a setter supplied, absent until set |
| crates/kivact-interaction/src/table/model.rs:299 | DataTableModelApi.arrangement | builder option | builder option - the row arrangement a setter supplied, absent until set |
| crates/kivact-interaction/src/table/model.rs:300 | DataTableModelApi.typed_queries | builder option | builder option - the typed queries a setter supplied, absent until set |
| crates/kivact-interaction/src/table/model.rs:301 | DataTableModelApi.paging | builder option | builder option - the paging a setter supplied, absent until set |
| crates/kivact-interaction/src/text_edit/handle.rs:499 | Data.observed | independent fact | independent fact - the observation of the attached control part, absent until it attaches |
| crates/kivact-interaction/src/toaster/column.rs:41 | Effect::Dismiss.previous | passive record | passive record - the focus target to restore, absent when there is none |
| crates/kivact-interaction/src/toaster/column.rs:42 | Effect::Dismiss.callback | passive record | passive record - the caller's dismiss callback, absent when there is none |
| crates/kivact-interaction/src/toggle.rs:32 | ButtonApi.observed | independent fact | independent fact - the observation of the attached part, absent until it attaches |
| crates/kivact-interaction/src/toggle.rs:33 | ButtonApi.disabled | builder option | builder option - the disabled value a setter supplied, absent until set |
| crates/kivact-interaction/src/tooltip/api.rs:137 | TooltipApi.trigger_state | independent fact | independent fact - the observation of the attached trigger part, absent until it attaches |
| crates/kivact-interaction/src/tooltip/api.rs:138 | TooltipApi.content_state | independent fact | independent fact - the observation of the attached content part, absent until it attaches |
| crates/kivact-runtime/src/reactive/error.rs:34 | ErrorKind::Disposed.created_at | passive record | passive record - the diagnostic source position where the handle was created |
| crates/kivact-runtime/src/reactive/error.rs:37 | ErrorKind::Disposed.disposed_by | passive record | passive record - the diagnostic source position of the owner that released it |
| crates/kivact-runtime/src/task/async_state.rs:66 | AsyncCell.latest | delivery record | delivery record - the value of the latest success, kept while a later run is pending or failed |
| crates/kivact-runtime/src/task/state.rs:102 | TaskState.blocking_threads | construction option | construction option - the worker count of the blocking pool; absent is the default |
| crates/kivact-runtime/src/task/worker_pool.rs:24 | Pool.tx | owned resource | owned resource - the job channel sender, dropped at shutdown so the workers end |
| crates/kivact-runtime/src/task/worker_pool.rs:25 | Pool.outstanding | statistics | statistics - the number of submitted jobs not yet finished, which the shutdown waits on |
| crates/kivact-ui-creative/src/dock/items.rs:13 | Item::Chrome.shown | passive record | passive record - whether the container is shown in the flattened layout |
| crates/kivact-ui-creative/src/dock/items.rs:14 | Item::Chrome.maximized | passive record | passive record - whether the container is maximized in the flattened layout |
| crates/kivact-ui-creative/src/editors/node/gesture.rs:194 | DragMode::Link.rewire | independent fact | independent fact - the link re-routed from its input socket, fixed at the press |
| crates/kivact-ui-creative/src/editors/node/gesture.rs:195 | DragMode::Link.released | independent fact | independent fact - the input socket under the release |
| crates/kivact-ui-soft/src/rich_block_doc.rs:12 | Kind::List.number | passive record | passive record - the number of an ordered list item, absent for a bullet |
| crates/kivact-ui-soft/src/rich_block_doc.rs:13 | Kind::List.checked | passive record | passive record - the check state of a task item, absent for a plain item |
| crates/kivact-ui/src/data_table/filter_value.rs:22 | FilterField::Number.min | passive record | passive record - the inclusive lower bound, absent for none |
| crates/kivact-ui/src/data_table/filter_value.rs:22 | FilterField::Number.max | passive record | passive record - the inclusive upper bound, absent for none |
| crates/kivact-ui/src/date_range_draft.rs:17 | RangeDraft::Editing.start | independent fact | independent fact - the typed start endpoint, absent while that field is empty |
| crates/kivact-ui/src/date_range_draft.rs:18 | RangeDraft::Editing.end | independent fact | independent fact - the typed end endpoint, absent while that field is empty |
| crates/kivact-ui/src/values.rs:388 | DataTableCheckState.header | passive record | passive record - recipe projection: the box is in the header |
| dev/tests/gpu-interop-producer/src/producer.rs:28 | Producer.shared | owned resource | owned resource - the pool's handles until they are shared; taken once |
| dev/tests/gpu-interop-producer/src/producer.rs:30 | Producer.next | S-8 ID | S-8 ID - the number of the frame drawn next |

## 7.2 static

`crates/arvane-native/src/port/linux_settings.rs` は module の static が 2 つ以上あるので error になる（SM-CORE-16 で `STARTED` を `Once` にした後でも `REDUCED_MOTION` と `NEXT` の 2 つが残る）。`static` の直前（`///` の上）に次を書く。

| site | static | 分類 | 印の文 |
|---|---|---|---|
| crates/arvane-native/src/port/linux_settings.rs:22 | REDUCED_MOTION | measurement | measurement - the reduced-motion preference last read from the portal |
| crates/arvane-native/src/port/linux_settings.rs:28 | NEXT | S-8 ID | S-8 ID - the next waker registration number |

## 7.3 他の entry が印を書くもの（T15.4 は何もしない）

| 型 | 理由 | 印を書く entry |
|---|---|---|
| `kivact-core` の `ExportRequest::GpuProvided`・`::Pixels`（`generation` と `latest`） | SM-CORE-01 の置き換えのコードが field ごとの印を持つ | SM-CORE-01（T7.2） |
| `kivact-core` の `NodeObservation`（`rect`・`text_truncated`・`layout_rect`） | struct の前の印を field ごとの印に直す | SM-CORE-02（T7.2） |
| `kivact-ui-soft` の `ItemBranch::Parent`（`expansion`・`flyout`） | SM-SOFT-02 の置き換えのコードが印を持つ | SM-SOFT-02（T1.3） |
| `kivact-runtime` の `TimelineCell.events` | counter を `Notifier` にして消す | SM-CORE-17（T3.2） |
| `arvane-virtual` の `VirtualPlatform`・`VirtualWindow` | struct を分ける | SM-CORE-06（T5.1 の 6.） |

## 7.4 分類の一覧と、この表を作った規則の件数

分類（印の文の先頭に使ってよい語）: `builder option`、`construction option`、`setting`、`passive record`、`caller binding`、`independent fact`、`measurement`、`statistics`、`delivery record`、`mailbox`、`pure cache`、`S-8 ID`、`S-9 named type`、`cursor`、`owned resource`。この付録の他の KEEP 表の分類もこの語に読み替えてよい（`ID allocator` は `S-8 ID`、`Sent` と `ReportOnce` は `S-9 named type`、`pure cache: ..` は `pure cache`、`independent fact (..)` は `independent fact`）。

件数: struct 24、enum の variant 16、static の組 1。T1.1 の確認（作業指示書 T1.1 の 9.）は、この表の型と実際の error の型を突き合わせる。
