# 02. Form controls — Astryx → kivact-ui-soft 作業指示書

対象: Astryx（`D:\w\meta\astryx`、NEUTRAL テーマ `packages/themes/neutral`）のフォーム系コンポーネント一式を、Kivact の native GUI kit `crates/kivact-ui-soft` として再実装するための仕様。`kivact-ui-basic` が shadcn/ui を移植したのと同じ方式で、`kivact::ui` の標準部品は theme recipe で RESTYLE し、それ以外は public `kivact` crate だけで作る kit component にする。token は Astryx 名（`packages/core/src/theme/tokens.stylex.ts`）で書き、各コンポーネントの初出で neutral の light / dark 解決値を併記する。全体の token 表は `foundation/tokens.md`、motion の役割は `foundation/motion.md` を参照。ただし本書のアニメーション値は本書だけで完結する。

## 目次

- 0. 共通前提（neutral 解決値の早見表・横断的な未解決事項）
- 1. TextInput
- 2. TextArea
- 3. NumberInput
- 4. Field（FieldLabel、共有 input container spec 4.4.4、InputClearButton、PanelSearchInput）
- 5. FieldStatus
- 6. InputGroup（InputGroupText）
- 7. FileInput
- 8. CheckboxInput
- 9. CheckboxList（CheckboxListItem）
- 10. RadioList（RadioListItem）
- 11. Switch
- 12. Slider
- 13. Selector（SelectorOption、SelectorBottomSheet）
- 14. ComplexSelector
- 15. MultiSelector
- 16. Typeahead（TypeaheadItem）
- 17. BaseTypeahead
- 18. Tokenizer（Token chip、TokenizerBottomSheet、useEndLaneReserve）
- 19. PowerSearch（PowerSearchToken / EditPopover / FilterEditor / ValueEditor）
- 20. Calendar
- 21. DateInput（TouchDateField、MonthScroller、Wheel / MonthYearWheels、NativeDateField、共有 touch geometry）
- 22. DateRangeInput
- 23. DateTimeInput（NativeDateSegment、NativeTimeSegment、TouchDateTimeField）
- 24. TimeInput（TouchTimeField）
- 25. RichTextEditor（packages/richtext 全体: Toolbar、RichTextView、plugins、Markdown 往復、段階的実装計画）
- 26. 実装ノート: hooks
  - 26.1 useInputContainer
  - 26.2 useInputStatusIcon
  - 26.3 useCombobox
  - 26.4 useMultiCombobox
  - 26.5 useTypeahead
  - 26.6 internal/tokenizer-presentation.ts
  - 26.7 useCollator / useLocale（日付・数値フォーマット）

## グループ概要

| Astryx component (and parts) | docs URL | source dir | kivact strategy | closest kivact-ui-basic analog |
|---|---|---|---|---|
| TextInput（box、start icon、clear button、spinner、status icon） | https://astryx.atmeta.com/components/TextInput | packages/core/src/TextInput（+ Field/inputStyles.stylex.ts, Field/InputClearButton.tsx, hooks/useInputContainer.ts, hooks/useInputStatusIcon.tsx） | RESTYLE `kivact::ui::text_input` + kit wrapper `soft::text_input` | crates/kivact-ui-basic/src/input_group.rs（`borderless`, `inline_addon`）+ recipes.rs `text_input` |
| TextArea（box、overlays、文字数カウンタ、resize grip） | https://astryx.atmeta.com/components/TextArea | packages/core/src/TextArea | RESTYLE `kivact::ui::text_area` + kit wrapper `soft::text_area` | crates/kivact-ui-basic/src/input_group.rs（`input_group_textarea`）+ recipes.rs `text_area` |
| NumberInput（numberParser.ts、numberInputCommit.ts、stepper 列、units、invalid alert） | https://astryx.atmeta.com/components/NumberInput | packages/core/src/NumberInput | NEW kit component `soft::number_input`（`kivact::ui::number_input` は recipes の RESTYLE のみ） | crates/kivact-ui/src/number_input.rs + crates/kivact-ui-basic/src/input_group.rs |
| Field（共有 internals: inputStyles, fieldParts, fieldStatusPlacement, textEntryTraits, inputDisabledFocus, InputClearButton, PanelSearchInput） | https://astryx.atmeta.com/components/Field | packages/core/src/Field | RESTYLE `kivact::ui::field` + kit wrapper（`soft::field`、kit 内部の共有 input container） | crates/kivact-ui-basic/src/field.rs + recipes_field.rs, primitives.rs `label().target()`, crates/kivact-ui/src/field.rs |
| FieldLabel | https://astryx.atmeta.com/components/FieldLabel | packages/core/src/Field（FieldLabel.tsx） | covered by another component（§4 Field の `soft::field_label`） | crates/kivact-ui-basic/src/field.rs, typography.rs |
| FieldStatus | https://astryx.atmeta.com/components/FieldStatus | packages/core/src/FieldStatus | NEW kit component `soft::field_status` | `kivact::ui::field` の error part、crates/kivact-ui-basic/src/alert.rs |
| InputGroup | https://astryx.atmeta.com/components/InputGroup | packages/core/src/InputGroup | NEW kit component `soft::input_group` | crates/kivact-ui-basic/src/button_group.rs + input_group.rs |
| InputGroupText | https://astryx.atmeta.com/components/InputGroupText | packages/core/src/InputGroup（InputGroupText.tsx） | covered by another component（§6.x `soft::input_group_text`） | crates/kivact-ui-basic/src/input_group.rs（input_group_text）/ button_group.rs（button_group_text） |
| FileInput（input mode、dropzone mode、hidden trigger、clear、validation） | https://astryx.atmeta.com/components/FileInput | packages/core/src/FileInput | NEW kit component `soft::file_input` | crates/kivact-ui-basic/src/input_group.rs + attachment.rs |
| CheckboxInput（Indicator/CheckboxIndicator） | https://astryx.atmeta.com/components/CheckboxInput | packages/core/src/CheckboxInput（+ Indicator/CheckboxIndicator.tsx） | RESTYLE `kivact::ui::checkbox` + kit wrapper `soft::checkbox_input` | crates/kivact-ui-basic/src/recipes.rs（`toggle_row`, `draw_checkbox`, `edge`） |
| CheckboxList | https://astryx.atmeta.com/components/CheckboxList | packages/core/src/CheckboxList（+ List, Item） | NEW kit component `soft::checkbox_list`（行の箱は RESTYLE した `kivact::ui::checkbox`） | crates/kivact-ui-basic/src/field.rs（field_set）+ item.rs |
| CheckboxListItem | https://astryx.atmeta.com/components/CheckboxListItem | packages/core/src/CheckboxList（CheckboxListItem.tsx） | covered by another component（§9 `soft::checkbox_list_item`） | crates/kivact-ui-basic/src/item.rs |
| RadioList（Indicator/RadioIndicator） | https://astryx.atmeta.com/components/RadioList | packages/core/src/RadioList（+ Indicator/RadioIndicator.tsx） | RESTYLE `kivact::ui::radio_group` + kit wrapper `soft::radio_list` | crates/kivact-ui-basic/src/recipes_controls.rs（`radio_group` recipes） |
| RadioListItem | https://astryx.atmeta.com/components/RadioListItem | packages/core/src/RadioList（RadioListItem.tsx） | covered by another component（§10、`RadioGroup::item` slot） | crates/kivact-ui-basic/src/recipes_controls.rs（`radio_item`） |
| Switch（switchFill, switchTouch, useSwitchTouch） | https://astryx.atmeta.com/components/Switch | packages/core/src/Switch | RESTYLE `kivact::ui::switch` + kit wrapper `soft::switch_field` | crates/kivact-ui-basic/src/recipes.rs（`switch_metrics`, `draw_switch`） |
| Slider（range、marks、値 tooltip、sliderTouchController） | https://astryx.atmeta.com/components/Slider | packages/core/src/Slider | RESTYLE `kivact::ui::slider` / `range_slider` + kit wrapper `soft::slider` | crates/kivact-ui-basic/src/recipes_controls.rs（slider recipes） |
| Selector（SelectorBottomSheet、selectorPresentation） | https://astryx.atmeta.com/components/Selector | packages/core/src/Selector | RESTYLE `kivact::ui::select` + NEW kit component `soft::selector` | crates/kivact-ui-basic/src/combobox.rs + list_popup.rs / recipes_popup.rs, native_select.rs |
| SelectorOption | https://astryx.atmeta.com/components/SelectorOption | packages/core/src/Selector（SelectorOption.tsx） | covered by another component（§13.x `soft::selector_option` part） | crates/kivact-ui-basic/src/list_popup.rs |
| ComplexSelector | https://astryx.atmeta.com/components/ComplexSelector | packages/core/src/ComplexSelector | NEW kit component `soft::complex_selector`（RESTYLE `kivact::ui::dialog(DialogKind::Popover).attach(trigger)` を内包） | crates/kivact-ui-basic/src/date_picker.rs（button + Popover dialog） |
| MultiSelector（create row、select-all、option actions/grid、count/labels/badges trigger） | https://astryx.atmeta.com/components/MultiSelector | packages/core/src/MultiSelector | NEW kit component `soft::multi_selector` | crates/kivact-ui-basic/src/combobox.rs（`Vec<K>` chips mode）+ list_popup.rs |
| Typeahead | https://astryx.atmeta.com/components/Typeahead | packages/core/src/Typeahead | NEW kit component `soft::typeahead`（kit 内 headless engine: `kivact::interaction::use_text_edit` + overlay） | crates/kivact-ui-basic/src/combobox.rs（+ list_popup.rs, item.rs） |
| TypeaheadItem | https://astryx.atmeta.com/components/TypeaheadItem | packages/core/src/Typeahead（TypeaheadItem.tsx） | covered by another component（§16.x） | crates/kivact-ui-basic/src/item.rs |
| BaseTypeahead | https://astryx.atmeta.com/components/BaseTypeahead | packages/core/src/Typeahead（BaseTypeahead.tsx） | NEW kit component（Typeahead / Tokenizer 共有 engine、任意で public `soft::base_typeahead`） | crates/kivact-ui-basic/src/combobox.rs（`popup` fn） |
| Tokenizer（Token chip、TokenizerBottomSheet、useEndLaneReserve、clear-all） | https://astryx.atmeta.com/components/Tokenizer | packages/core/src/Tokenizer | RESTYLE `kivact::ui::tag_input` + NEW kit component `soft::tokenizer` | crates/kivact-ui-basic/src/combobox.rs（chips_field + popup） |
| PowerSearch（PowerSearchToken、EditPopover、FilterEditor、ValueEditor、formatFilterValue、usePowerSearchConfig/Source） | https://astryx.atmeta.com/components/PowerSearch | packages/core/src/PowerSearch | NEW kit component `soft::power_search`（`soft::tokenizer` の上に構築） | crates/kivact-ui-basic/src/combobox.rs + date_picker.rs |
| Calendar（MonthGrid、DayCell、hooks、dayCellUtils、getInitialFocusDate、standalone weekday names） | https://astryx.atmeta.com/components/Calendar | packages/core/src/Calendar | NEW kit component（`kivact::interaction::use_calendar_grid` + `kivact::i18n::CalendarDate/CalendarMonth`） | crates/kivact-ui-basic/src/calendar/（view.rs, panel.rs, caption.rs） |
| DateInput（PointerDateField: text combobox + calendar popover） | https://astryx.atmeta.com/components/DateInput | packages/core/src/DateInput | NEW kit component（RESTYLE `kivact::ui::text_input` + `kivact::ui::dialog` Popover + soft Calendar） | crates/kivact-ui-basic/src/date_picker.rs |
| DateInput › TouchDateField（bottom sheet） | https://astryx.atmeta.com/components/DateInput | packages/core/src/DateInput | NEW kit component（`kivact::ui::dialog` Drawer(Side::Bottom) / BottomSheet kit part） | crates/kivact-ui-basic/src/drawer.rs |
| DateInput › MonthScroller（横スナップの月ページャ、DateTimeInput と共有） | https://astryx.atmeta.com/components/DateInput | packages/core/src/DateInput | NEW kit component（`use_drag` + `kivact::motion` spring の自前ページャ） | crates/kivact-ui-basic/src/carousel.rs, calendar/panel.rs |
| DateInput › Wheel / MonthYearWheels（DateTimeInput / TimeInput と共有） | https://astryx.atmeta.com/components/DateInput | packages/core/src/DateInput | NEW kit component（`use_drag` + spring の 1 次元 wheel、Role::ListBox） | なし（最も近いのは crates/kivact-ui-basic/src/list_popup.rs） |
| DateInput › NativeDateField / nativeDateSegments | https://astryx.atmeta.com/components/DateInput | packages/core/src/DateInput | covered by another component（native → TouchDateField sheet。Kivact に OS picker なし） | — |
| DateInput › 共有 touch geometry / hooks（tokens.stylex.ts, monthGeometry, useScrollSettle, usePointerDragScroll, useOwnScrollGesture） | https://astryx.atmeta.com/components/DateInput | packages/core/src/DateInput | NEW kit-internal module（定数 + gesture の意味論） | crates/kivact-ui-basic/src/drawer.rs（use_drag の使い方） |
| DateRangeInput（trigger、preset sidebar、範囲 Calendar popover） | https://astryx.atmeta.com/components/DateRangeInput | packages/core/src/DateRangeInput | NEW kit component | crates/kivact-ui-basic/src/date_picker.rs（DateRangePicker）+ calendar/view.rs |
| DateTimeInput（date/time segments、calendar popover、time-options listbox、NativeDateSegment、NativeTimeSegment、nativePickerSegmentStyles、TouchDateTimeField） | https://astryx.atmeta.com/components/DateTimeInput | packages/core/src/DateTimeInput | NEW kit component | crates/kivact-ui-basic/src/date_picker.rs + input_group.rs + combobox.rs / list_popup.rs + drawer.rs |
| TimeInput（typed time field、TouchTimeField sheet） | https://astryx.atmeta.com/components/TimeInput | packages/core/src/TimeInput | RESTYLE `kivact::ui::text_input` + kit wrapper | crates/kivact-ui-basic/src/input_group.rs + field.rs / recipes_field.rs |
| RichTextEditor（RichTextEditorToolbar、link Dialog、RichTextView、AutoLink 等 plugins、Markdown 往復） | https://astryx.atmeta.com/components/RichTextEditor | packages/richtext/src（+ blocks/、stories RichTextEditor*.tsx） | NEW kit component（`soft::rich_text_editor` / `rich_text_toolbar` / `rich_text_view` + `RichDoc` + Markdown module。MVP は RESTYLE した `kivact::ui::text_area` 上の Markdown ソース編集 + `kivact::control::rich_text` による view） | crates/kivact-ui-basic/src/code_block/view.rs + typography.rs + input_group.rs（toolbar は button_group.rs / menubar.rs）。rich text の analog は無い |

---

## 0. 共通前提

### 0.1 neutral 解決値の早見表

各セクションでも初出で値を書くが、横断的に使う値をここにまとめる（`packages/themes/neutral/src/neutralTheme.ts` + `neutralPaletteRefs.generated.ts`、core 既定 `packages/core/src/theme/tokens.stylex.ts`）。

| 区分 | token | light | dark |
|---|---|---|---|
| 色 | `--color-accent` | #1b1b1b | #f1f1f1 |
| | `--color-on-accent` | #ffffff | #111111 |
| | `--color-text-primary` / `-secondary` / `-disabled` | #000000 / #474747 / #919191 | #ffffff / #9e9e9e / #525252 |
| | `--color-background-surface` / `-body` / `-popover` | #ffffff / #f1f1f1 / #ffffff | #262626 / #1b1b1b / #1b1b1b |
| | `--color-background-muted` / `--color-accent-muted` | #f1f1f1 / #f1f1f1 | #1b1b1b / #262626 |
| | `--color-border` / `--color-border-emphasized` | #00000014 / #d4d4d4 | #ffffff1A / #525252 |
| | `--color-overlay-hover` / `--color-overlay-pressed` | #0000000D / #0000001A | #ffffff0D / #ffffff1A |
| | `--color-tint-hover` | #000000 | #ffffff |
| | `--color-track`（neutral 未上書き → core 既定） | #CCD3DB | #5A5E66 |
| 角丸 | `--radius-inner` / `-element` / `-container` / `-full` | 6px / 10px / 12px / 9999px | 同 |
| 寸法 | `--size-element-sm/md/lg` | 28 / 32 / 36px | 同 |
| | `--border-width` | 1px | 同 |
| | spacing | rem 単位、1rem = 16px（`--spacing-1` 4px, `-2` 8px, `-3` 12px, `-4` 16px, `-5` 20px） | 同 |
| focus | `--focus-outline-*` | 2px solid `--color-accent`、offset 3px | 同 |
| 文字 | font family | Figtree（fallback: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif）、code は ui-monospace | 同 |
| | type scale | base 14、ratio 1.2（coarse pointer かつ幅 < lg のとき base 16、ratio 1.1736） | 同 |
| | body / label / supporting | 14/20 w400 / 14/20 w500 / 12/20 w400 | 同 |
| motion | `--duration-fast-min` / `-fast` / `-fast-max` | 95 / 125 / 165ms | 同 |
| | `--duration-medium-min` / `-medium` / `-medium-max` | 225 / 300 / 400ms | 同 |
| | `--duration-slow-min` / `-slow` / `-slow-max` | 525 / 700 / 935ms | 同 |
| | `--ease-standard` = `--ease-entrance` = `--ease-move` | cubic-bezier(0.24, 1, 0.4, 1) | 同 |
| | `--ease-exit` | cubic-bezier(0.3, 0, 0.6, 0.6) | 同 |
| 押下 | mouse | `:active` で `--color-overlay-pressed` を即時に重ねる。hover（hover:hover デバイスのみ）は `--color-overlay-hover` | |
| | touch（`utils/pressGesture.ts`） | onset 150ms、slop 10px、素早いタップは 100ms の flash、release fade 200ms cubic-bezier(0, 0, 0.2, 1) | |
| popover | Layer enter（`Layer/layerAnimations.stylex.ts`） | opacity 0→1、anchor 方向へ 8px の translate、scale 0.95→1、165ms ease-standard、exit アニメーションなし。reduced motion: fine pointer はなし、coarse は 200ms 以下の fade。**Typeahead / MultiSelector の popup は enter アニメーション自体を持たない**（§15, §16） | |
| spring | Kivact 換算 | (response r, dampingRatio ζ) → `kivact::motion::Spring { stiffness: (2π/r)², damping: 2ζ·(2π/r), mass: 1.0 }` | |

color-mix(in srgb, A, B p%) は sRGB の各チャンネルの線形補間で、各セクションに計算済みの hex を書いてある。native には color-mix が無いので、kit の Tokens に派生色として持たせる。

### 0.2 横断的な未解決事項（Kivact 側）

各セクションの 7 節（Kivact 実装方針）で詳しく書くが、複数コンポーネントに共通する課題を先にまとめる。判断が要るものは「要判断」と書いた。

1. **paint 系トグルのトランジション（要判断）**: `kivact::ui::checkbox` / `switch` / `radio_group` の indicator は recipe の `Style::paint` で描かれ、補間されない。そのため Astryx の 125ms 色トランジションと Switch の thumb スライドを再現できない。§11.7 では core に `CheckState` の補間済み progress を足すか、thumb や indicator を子ノード化して recipe を足すことを推奨している。
2. **coarse pointer 判定**: Astryx は `(pointer: coarse)` でタッチ用の表示（Switch / Slider の UIKit 物理、bottom sheet 表示、44/48px の hit 領域、touch 押下の遅延）に切り替える。Kivact には相当する値が無い。暫定案は `platform::capabilities().touch` と、直前の pointer 種別で近似すること。
3. **focusable な disabled**: Astryx の disabled は `aria-disabled` で、フォーカス可能なまま操作だけを止める（spec AST-085）。Kivact の `.disabled(true)` がフォーカスを保つかは未確認。
4. **ARIA の不足**: `aria-required` / `aria-busy` / `aria-readonly` / `aria-haspopup` / `aria-current` の設定 API が見当たらない。代替案（description 文、live region）を各節に書いた。
5. **text 編集**: native editor は折り返さない（`crates/kivact-core/src/primitive/text_editing.rs:164`）。TextArea の soft wrap と RichTextEditor の WYSIWYG はこれに阻まれる（§2.7, §25.7）。
6. **i18n**: Intl に相当するもの（数値の記号、時刻の hour cycle、CLDR の月名・曜日名、collation）が無い。kit 側に小さなロケール表を持つ案を §3.7 と §26.7 に書いた。
7. **status 付き field の focus ring**: Astryx では status の box-shadow がベースの inset focus ring を置き換えるので、status 付き field にフォーカスしても内側リングが出ない。これは Astryx の実挙動として再現する（§4.4.4）。dark では `--color-accent-muted`（#262626）が surface と同色なので、そもそも inset リングは見えない。

---

## 1. TextInput

ソース: `packages/core/src/TextInput/TextInput.tsx`（636 行）、`TextInput.doc.mjs`、`TextInput.test.tsx`、共有: `Field/inputStyles.stylex.ts`、`Field/inputDisabledFocus.ts`、`Field/InputClearButton.tsx`、`Field/textEntryTraits.ts`、`hooks/useInputContainer.ts`、`hooks/useInputStatusIcon.tsx`。Storybook: `apps/storybook/stories/TextInput.stories.tsx`（Default / SizeVariants / StatusVariations / Clearable / ClearableWithStatus / Disabled / DisabledWithMessage / ReadOnly / StatusVariantComparison など）。Docs: https://astryx.atmeta.com/components/TextInput

### 1.1 概要

TextInput は名前・メール・検索語など、短い単一行テキストを入力するフィールド。Field（ラベル・説明・ステータスメッセージ）で包まれ、入力ボックス内に開始アイコン、クリアボタン、ローディングスピナー、ステータスアイコンを持てる。

- 使う: 1 行で済む短い値。必須／形式チェックがある項目は validation status（メッセージ付き）と組み合わせる。検索・フィルタ欄には `hasClear` を付ける。
- 避ける: 複数行（コメント・説明文）→ TextArea。placeholder をラベルの代わりにしない。全項目を required にしない。無効理由を出すのに Tooltip で包まず `disabledMessage` を使う。
- ラベルは常に描画（`isLabelHidden` でも読み上げ用に残す）。ラベルを隠すのは虫眼鏡アイコン付き検索欄のように文脈で明らかな時だけ。

### 1.2 Anatomy

```
Field (column, gap 4px, isolation)                       ← Field セクション (4.x) 担当
├─ FieldLabel  <label for={inputId}>  (+ Optional/Required 表示, labelTooltip の info アイコン)
├─ description <… id=descriptionID>
├─ [attached の場合] status wrapper
│   └─ input container  div.astryx-text-input
│        data-size="sm|md|lg" data-status="error|warning|success"? data-disabled? data-readonly?
│        ├─ startIcon            Icon size=sm(16px) color=secondary            (任意)
│        ├─ VisuallyHidden label (InputGroup 内のみ, id=inputLabelID)
│        ├─ <input type={type}>  flex:1, 枠なし透明                           (role: textbox / searchbox 等ネイティブ)
│        ├─ Spinner sm aria-hidden                                            (isBusy 時)
│        ├─ InputClearButton (Button ghost sm icon-only "close")              (hasClear && value!=="" && !disabled && !readOnly)
│        └─ status affordance    attached: Icon md / tooltip: <button aria-label="Error details"> + tooltip layer
├─ disabledMessage tooltip layer (portal, placement above)
└─ FieldStatus (attached = ボックス下に重なる / detached = 離れて下)   ← Field セクション担当
```

DOM 順 = 視覚順（左→右, LTR）: startIcon → input → Spinner → Clear → status。InputGroup 内では Field が描画されず、ボックスだけが返る（`TextInput.tsx:595-603`）。

```
┌──────────────────────────────────────────────┐  h=32 (md)
│ 🔍  Search docs…                    ◌  ✕  ⚠ │  pad 4/8, gap 8
└──────────────────────────────────────────────┘  border 1px, radius 10px
  ↑16px icon      ↑ input(flex 1)   ↑14 ↑20 ↑20
```

### 1.3 API

| Astryx prop | 型 / 既定 | 意味 | Kivact 案 |
|---|---|---|---|
| `label` | `string`（必須） | ラベル。常に描画 | `soft::text_input(label, value)` 第 1 引数 |
| `value` | `string`（必須） | 制御値 | 第 2 引数 `impl Into<Binding<String>>`（caller 所有 signal） |
| `onChange` | `(value, e) => void` | 値変更 | binding の write 側（`binding(read, write)`）。別途 `.on_change(Fn(String))` を設けても良い |
| `changeAction` | `(value, e) => void \| Promise` | onChange 後の非同期処理。pending 中は楽観値表示＋スピナー | `.change_action(Fn(String) -> impl Future)`（`kivact::task::action` で pending を busy として扱う） |
| `type` | `'text'\|'password'\|'email'\|'search'\|'tel'\|'url'` / `'text'` | HTML type | `.password(bool)` のみ意味あり。他はタッチキーボード用ヒントなのでネイティブでは **drop**（`InputType` enum を残すなら Password 以外は無視） |
| `size` | `'sm'\|'md'\|'lg'` / `'md'`（`SizeContext` から継承可） | 高さ 28/32/36 | `.size(ControlSize::Small/Medium/Large)`（`ExtraSmall` は Small 扱い） |
| `isLabelHidden` | bool / false | ラベルを視覚的に隠す | `.label_hidden(bool)` |
| `description` | string | ラベルと入力の間の説明 | `.description(text)` |
| `isOptional` / `isRequired` | bool / false（排他） | Optional / Required 表示、required は `aria-required` | `.optional(bool)` / `.required(bool)` |
| `isDisabled` | bool / false | 0.5 で減光・編集不可・送信除外。**タブ順に残る**（aria-disabled） | NodeBuilder `.disabled(..)`（後述：フォーカス可能のまま） |
| `isReadOnly` | bool / false | 減光なし・編集不可。disabled が優先 | `.read_only(impl IntoValue<bool>)`（既存 `TextInput::read_only`） |
| `disabledMessage` | string | disabled 時の理由ツールチップ（hover / keyboard focus） | `.disabled_message(text)` |
| `startIcon` | `ReactNode \| IconType` | 先頭アイコン | `.start_icon(impl IntoView)`（通常 `icon("search")`） |
| `status` | `{type:'error'\|'warning'\|'success', message?}` | 枠色・アイコン・メッセージ。error で aria-invalid | `.status(impl IntoValue<Option<InputStatus>>)`（Field 共通型。名前は Field セクションに合わせる） |
| `statusVariant` | `'attached'\|'detached'\|'tooltip'` / `'attached'` | メッセージ配置 | `.status_placement(StatusPlacement::Attached/Detached/Tooltip)` |
| `isLoading` | bool / false | スピナー＋aria-busy | `.loading(impl IntoValue<bool>)` |
| `placeholder` | string | 空の時のヒント | `.placeholder(text)`（既存） |
| `width` | `number\|string` | Field 全体の幅（px or `'100%'`） | `.width(impl Into<Length>)`（Field ルートに適用） |
| `labelTooltip` | string | ラベル末尾の info アイコンのツールチップ | `.label_tooltip(text)` |
| `hasClear` | bool / false | 値がある時クリアボタン | `.clearable(bool)` |
| `hasAutoFocus` | bool / false | マウント時フォーカス | `.auto_focus(bool)`（mount 時に `NodeRef::focus()`） |
| `htmlName` | string | フォーム送信名 | **drop**（ネイティブにフォーム送信なし） |
| `onEnter` | `() => void` | Enter（IME 確定中は発火しない） | `.on_enter(Fn())` → 既存 `TextInput::on_submit` |
| `onKeyDown` | `(e) => void` | 全 keydown | NodeBuilder `.on_key(..)` |
| `autoComplete` | string | ブラウザ自動補完ヒント | **drop** |
| `hasSpellCheck` | bool / 未設定 | スペルチェック | **drop**（ネイティブ editor に spellcheck なし） |
| `enterKeyHint` / `inputMode` / `autoCapitalize` / `hasAutoCorrect` | `textEntryTraits.ts` | タッチキーボード属性 | **drop**（デスクトップネイティブでは無意味） |
| `containerXstyle/ClassName/Style` | — | Field ルートのスタイル | `.container_style(Style)` |
| `controlXstyle/ClassName/Style` | — | ボックスのスタイル | NodeBuilder `.style(..)` はネイティブ入力に、ボックスへは `.control_style(Style)` |
| `ref`, `id`, `aria-describedby`, `aria-labelledby`, `data-*` | — | | `.node_ref(..)`, `.described_by(NodeRef)`, `.labelled_by(NodeRef)`, `.test_id(..)` |

### 1.4 Visual spec

#### 共通ボックス（Field 共通の input container spec, 4.4 参照）— 解決値

`inputWrapperStyles.base`（`Field/inputStyles.stylex.ts:32-73`）＋ TextInput の `sizeStyles`（`TextInput.tsx:90-100`）。

| 項目 | トークン | neutral light | neutral dark |
|---|---|---|---|
| 高さ sm / md / lg | `--size-element-sm/md/lg` | 28 / 32 / 36px（border-box, 枠込み） | 同左 |
| padding | block `--spacing-1`, inline `--spacing-2` | 上下 4px / 左右 8px | 同左 |
| gap（子要素間） | `--spacing-2` | 8px | 同左 |
| border | `--border-width` solid `--color-border-emphasized` | 1px `#d4d4d4` | 1px `#525252` |
| radius | `--radius-element`（`--_field-radius` 経由） | 0.625rem = 10px | 10px |
| 背景 | `--color-background-surface` | `#ffffff` | `#262626` |
| 入力文字 | `--font-family-body` / `--text-body-size` / `--text-body-leading` | Figtree 14px / 20px（1.4286）/ 400 | 同左 |
| 文字色 | `--color-text-primary` | `#000000` | `#ffffff` |
| placeholder | `--color-text-secondary` | `#474747` | `#9e9e9e` |
| hover リング（focus-within でない・非 disabled・`hover:hover`） | `inset 0 0 0 2px color-mix(--color-border-emphasized 30%, transparent)` | `#d4d4d44D`（rgba(212,212,212,.30)） | `#5252524D` |
| focus-within 枠 | `--color-accent` | `#1b1b1b` | `#f1f1f1` |
| focus-within リング | `inset 0 0 0 2px --color-accent-muted` | `#f1f1f1` | `#262626`（= surface と同色なので dark では実質見えない） |
| outline | none（ボックスは outline を使わず枠＋inset シャドウでフォーカス表示） | | |

inset シャドウは枠の内側（padding box の縁）に 2px で描かれる。position relative, z-index 1, display flex, align-items center, box-sizing border-box。

#### TextInput 固有

- ネイティブ `<input>`（`TextInput.tsx:59-84`）: display block, flex 1, min-width 0, 枠/padding 0, 背景透明, outline none。行高 20px がボックス内（md: 内寸 32−2−8 = 22px）で縦中央。
- startIcon: `renderIconSlot(startIcon, {size:'sm', color:'secondary'})` → 16×16px（1rem）, 色 `--color-icon-secondary` light `#6a6a6a` / dark `#9e9e9e`。lucide アイコン（neutral レジストリ, stroke 2, `size: 1em`）。
- Spinner（isBusy）: size sm = 直径 10px + stroke 2px → 14×14px ボックス。弧色 `--color-accent`（`#1b1b1b` / `#f1f1f1`）、トラック `--color-track`（neutral は上書きしない → core 既定 light `#CCD3DB` / dark `#5A5E66`, stroke-opacity 1）、弧長 = 円周 × 0.375、`strokeLinecap: round`。
- クリアボタン（`InputClearButton.tsx:118-230`）: Button `variant=ghost size=sm isIconOnly`、高さを 20px に上書き＋ aspect-ratio 1/1 → **20×20px**、padding 0、radius `var(--_button-radius, --radius-element)` = 10px、背景透明。アイコン `close`（neutral = lucide `X`）size sm 16px、色 `--color-icon-secondary`（`#6a6a6a` / `#9e9e9e`）。hover（hover:hover）: 背景画像 `--color-overlay-hover`（light `#0000000D` / dark `#ffffff0D`）、:active: `--color-overlay-pressed`（`#0000001A` / `#ffffff1A`）＋ `scale(0.98)`。focus-visible: outline 2px solid `--color-accent` offset 3px。coarse pointer のみ `::after` で当たり判定を 24×24 に拡大（inset −2px）。ツールチップ「Clear」（placement above）。
- ステータスアイコン（`useInputStatusIcon.tsx:194-204`）: Icon size md = 20×20px, 色 `--color-icon-<status>`。neutral グリフ: error = lucide `XCircle`, warning = `AlertTriangle`, success = `CheckCircle`。tooltip 配置ではネイティブ枠を外した `<button>`（padding 0, 透明, radius 9999px, focus-visible outline 2px accent **offset 2px**, cursor pointer）で包む（`useInputStatusIcon.tsx:61-83`）。

#### 状態

| 状態 | 変化（light / dark） | 根拠 |
|---|---|---|
| rest | 上表 | `inputStyles.stylex.ts:32-73` |
| hover（ボックス上, 非 focus） | inset 2px `#d4d4d44D` / `#5252524D` | `:66-69` |
| focus（ボックス内のどこか = input / クリアボタン / ステータスボタン） | 枠 `#1b1b1b` / `#f1f1f1`, inset 2px `#f1f1f1` / `#262626`, hover リングは出ない | `:48-54, :70` |
| pressed | ボックス自体に変化なし | — |
| disabled | ボックス opacity 0.5、枠は常に `--color-border-emphasized`、hover リングなし、cursor default。**キーボードで input にフォーカス（focus-visible）した時だけ**枠 accent ＋ inset accent-muted を復活（ポインタフォーカスでは出ない）。クリアボタン非表示。文字色はそのまま（減光は opacity のみ） | `inputStyles.stylex.ts:74-80`, `inputDisabledFocus.ts:18-32` |
| readOnly | 見た目の変化なし（不透明度 1、hover/focus リングも通常通り）。クリアボタン非表示、入力不可 | `TextInput.tsx:569, 585` |
| loading / busy | Spinner sm を input の後・クリアの前に表示。ボックスの見た目は不変 | `:584` |
| status error | 枠 `--color-border-error` light `#c0262c` / dark `#ffaea7`（focus 中も同色を維持）。hover: `--shadow-inset-error` = inset 2px `#de47454D`（両モード同値）。アイコン XCircle 20px 色 `--color-icon-error`（= `#c0262c` / `#ffaea7`） | `:89-117, :123-151, :159-187` |
| status warning | 枠 `#836700` / `#eec448`。hover inset 2px `#f8d36a4D`。アイコン AlertTriangle 色同上 | 同上 |
| status success | 枠 `#2f7d33` / `#90ca90`。hover inset 2px `#2f7d334D`。アイコン CheckCircle 色同上 | 同上 |
| status ＋ focus | 枠は status 色のまま。**accent-muted の inset フォーカスリングは消える**（status の `boxShadow` スタイルが後勝ちでプロパティ全体（:focus-within 分岐を含む）を置き換えるため。StyleX はプロパティ単位で最後の値が勝つ）。docsite で要目視確認 | `TextInput.tsx:511-513` |
| status ＋ disabled | 枠 status 色、hover シャドウなし、キーボード focus-visible 時は inset accent-muted のみ復活 | `:509-513` |

#### レイアウト

- ボックス幅 = Field 幅（Field はカラム flex で親幅に伸び、`min-width: 0` で縮む）。`width` は Field ルートに適用。
- input は 1 行・折り返しなし・はみ出しは横スクロール（ネイティブ）。アイコン類は `flex-shrink` しない（Button/Spinner は shrink 0）。
- RTL: 論理プロパティと flex row なので左右反転（startIcon が右端、ステータスが左端）。
- InputGroup 内（`InputGroup/groupStyles.ts:75-102`）: flex 1, height 100%, 2 つ目以降 `margin-inline-start: -1px`、外側の角だけ 10px、他 0、focus-within で z-index 1。ステータスアイコン・メッセージは group が持つので描画しない。
- 注: TextInput.tsx の JSDoc（`size` の説明 "18px / 26px"）は古い。実値はトークン 28/32/36px。

### 1.5 Motion

| 対象 | トリガ | プロパティ | from→to | 時間 / easing | reduced motion | 出典 |
|---|---|---|---|---|---|---|
| ボックス | hover / focus / status / disabled の変化（入り・出とも同じ） | `border-color`, `box-shadow` | 状態表の値間 | `--duration-fast` = 125ms, `--ease-standard` = cubic-bezier(0.24, 1, 0.4, 1), delay 0 | 0s（即時） | `inputStyles.stylex.ts:58-63` |
| クリアボタン | hover / press | `background-image`（overlay）, `transform` scale 1→0.98, color, opacity | — | 125ms cubic-bezier(0.24, 1, 0.4, 1)。ただし background-image（グラデーション）は補間されないので **overlay は即時切替**、scale のみ 125ms で動く | 0s | `Button.tsx:84-136`, `interactionOverlay.stylex.ts:245-266` |
| クリアボタン（touch） | press 解放 | `--astryx-press-alpha` 1→0 | — | 200ms cubic-bezier(0, 0, 0.2, 1), fill both（onset 150ms, slop 10px） | min(200ms, …) | `interactionOverlay.stylex.ts:139-208` |
| クリアボタン / スピナー / ステータスアイコンの出現・消滅 | 値・busy の変化 | — | — | アニメーションなし（即時マウント/アンマウント） | — | — |
| Spinner sm | busy 中常時 | 弧の `stroke-dashoffset` | 0 → −(10px × π) = −31.416px | 525ms（`--duration-slow-min`: neutral slow 700 × 0.75）, linear, infinite。弧は −90° 回転済み（12 時起点） | 3s/周（停止ではなく減速） | `Spinner.tsx:228-231, 338-350` |
| ツールチップ（disabledMessage / ステータス / Clear） | hover・focus | — | — | Tooltip セクション参照（show delay 200ms, Layer enter: opacity 0→1 + 8px + scale 0.95→1, 165ms ease-standard） | — | `Tooltip/useTooltip.tsx` |

### 1.6 Behaviour & a11y

キーボード:
- 通常のネイティブ編集（キャレット・選択・IME・クリップボード）。
- Enter: `onEnter()` を呼ぶ（IME 変換確定中の Enter（`isComposing` / keyCode 229）は呼ばない）。その後 `onKeyDown(e)` に全キーを渡す（`TextInput.tsx:542-563`）。
- Tab 順: input → クリアボタン（通常の Button なのでタブ可能）→ tooltip 配置のステータスボタン。
- disabled 中: フォーカスは可能（タブ順に残る）。Enter は `preventDefault`（フォーム送信を止める）だけで `onEnter`/`onKeyDown` は呼ばない。onFocus/onBlur 以外の caller ハンドラ（`on[A-Z]*`）はすべて外す（`:376-382`）。

ポインタ:
- ボックスの非インタラクティブ領域（アイコン・padding・attached のステータスアイコン）をクリック → input にフォーカス（`useInputContainer`, text 系 type は `.focus()`）。クリック先が button 等インタラクティブ子要素の場合や、ボックス内にテキスト選択がある場合は何もしない。disabled 時は何もしない（`hooks/useInputContainer.ts:319-354`, `useClickableContainer.ts`）。
- クリアボタン: pointerdown / mousedown を `preventDefault` → input のフォーカスを奪わない。クリックで `onChange('')`、input にフォーカス（ポインタ時は `preventScroll`）（`TextInput.tsx:462-477`）。キーボード起動（detail 0）でも input にフォーカスを戻す。

値:
- `isDisabled || isReadOnly` の間は handleChange が何もしない（`:444-459`）。
- `changeAction`: onChange の後（`defaultPrevented` でなければ）transition 内で楽観値をセットして await。楽観値 ≠ 制御値の間 busy（スピナー・aria-busy）。

a11y（ネイティブ input に付与）:
- role: ネイティブ（textbox, type=search は searchbox）。name: Field の `<label for>`。caller の `aria-labelledby` があれば Field ラベル id ＋ caller id。
- `aria-describedby` = caller → description → ステータスメッセージ（attached/detached でメッセージあり・group 外）→ tooltip 配置のツールチップ id → disabledMessage ツールチップ id（存在する要素だけを参照）。
- `aria-invalid="true"` は status error のみ（warning / success は付けない）。`aria-required`、`aria-busy`、`aria-disabled="true"`（ネイティブ disabled は使わない）＋ `readOnly`。
- busy 開始時にコンテナ内のライブリージョンで「Loading」をアナウンス（`useBusyAnnouncement`）。スピナー自体は aria-hidden。
- disabledMessage: `useTooltip({placement:'above', focusTrigger:'always'})` をボックスに付与 → hover と focusin で開く。
- tooltip 配置ステータス: `<button aria-label="Error details">`、メッセージを describedby。fine pointer は hover / focus で開く、`hover: none` 環境ではタップでトグル、Escape で閉じる、blur で非制御に戻る（`useInputStatusIcon.tsx:138-183`）。

既定テキスト（i18n キー案）:
| Astryx key | 英語 | soft キー案 |
|---|---|---|
| `@astryx.textInput.clearLabel` | `Clear {label}`（クリアボタンの accessible name） | `soft.text_input.clear` |
| `@astryx.inputClearButton.tooltip` | `Clear`（ツールチップ） | `soft.input.clear_tooltip` |
| `@astryx.input.statusButton.error/warning/success` | `Error details` / `Warning details` / `Success details` | `soft.input.status_error` / `soft.input.status_warning` / `soft.input.status_success` |
| `@astryx.spinner.loading` | `Loading` | `soft.spinner.loading` |

### 1.7 Kivact 実装方針

戦略: **RESTYLE `kivact::ui::text_input` + kit wrapper**。

1. RESTYLE（`UiRecipes::text_input` / `.text_input(|_| recipe)`）: `kivact::ui::text_input` を直接使うアプリ向けに、ネイティブフィールド自身にボックス look を与える。`TextInputState { input, size, invalid }` から: 高さ（Small 28 / Medium 32 / Large 36, ExtraSmall→28）、`padding_y(4).padding_x(8)`、`border(1, border_emphasized)`、`radius(10)`、`background(surface)`、`font_size(14).line_height(20/14)`、`placeholder_color(text_secondary)`、`caret_color(text_primary)`。`focus(style().border_color(accent).shadow(Shadow::new(0,0,0,accent_muted).spread(2).inset()))`、非フォーカス時 `hover(style().shadow(Shadow::new(0,0,0,#d4d4d44D).spread(2).inset()))`、`invalid` → 枠 error 色。`transition(TransitionProp::Paint, Transition::tween(125, Easing::CubicBezier(0.24, 1.0, 0.4, 1.0)))`。ネイティブ editor の leaf は `Style::paint` 非対応なので style のみで表現する（`kivact-ui/src/text_input.rs` 冒頭コメント）。参考: `crates/kivact-ui-basic/src/recipes.rs:304-340`（`field` / `text_input`）。
2. kit wrapper `soft::text_input(label, value) -> soft::TextInput`: ネイティブ入力は子を持てないので、ボックスは wrapper の row ノードで描く。`crates/kivact-ui-basic/src/input_group.rs` が手本（`borderless(..)`: 内側の `kivact::ui::text_input` を枠・リング・背景なしにし、外側ノードが `NodeRef::focus_within()` を読んでフォーカス look を切り替える。`inline_addon` の `.on_press(move || input.activate())` で padding/アイコンのクリックを入力に転送）。
   - row: `[start_icon][text_input grow 1][spinner][clear][status]`、`gap(8)`, `padding_y(4).padding_x(8)`, 高さはサイズ表。hover リングは `focus_within` が false の時だけ `.hover(..)` に入れる（Astryx の `:hover:not(:focus-within)`）。
   - クリアボタン: `kivact::ui::button(message_with("soft.text_input.clear", ..)).variant(ButtonVariant::Ghost).hide_label(true).icon(icon("x"))` を 20×20 に、`kivact::ui::tooltip("Clear").attach(..)`。ポインタ押下でフォーカスを奪わない必要がある → ボタンを `.focusable(true)` のまま pointer down でフォーカス移動を抑止できるか要確認。できなければ press 後に入力へ `NodeRef::focus()` で戻す（Astryx も最終的に input にフォーカス）。
   - Spinner: `kivact::motion` の `Animated` で dashoffset 相当（または回転弧）を描く kit `soft::spinner(size)`（Spinner セクションに合わせる）。
   - ステータス: Field 共通の status icon 部品（tooltip 配置は `kivact::ui::tooltip(message).attach(button)`）。
   - disabled: Astryx は「減光＋フォーカス可能」。kivact の `.disabled(true)` はハンドラを止めるがフォーカス可否は要確認。フォーカス可能を保てない場合は `read_only(true)` ＋ semantics `disabled` ＋ 0.5 opacity で代替し、keyboard focus-visible 時のみリング復活。
   - `change_action`: `kivact::task::action(ActionPolicy::..., run)` の pending を busy に合成し、楽観値は wrapper 内 signal で保持。
   - Field（ラベル・説明・ステータスメッセージ）は Field セクションの `soft::field` に委譲。
- 落とすもの: forced-colors（Highlight 枠）、iOS 16px フォント下限、`(hover: hover)` 判定（デスクトップは常に hover 可）、`(pointer: coarse)` のクリア当たり判定拡大（タッチ対応時に 24px へ）。
- 難所: dark での focus inset リングが surface と同色（`#262626`）で見えない点は仕様どおり再現する（枠の accent 色でフォーカスが分かる）。

### 1.8 受け入れ確認

- [ ] role textbox, name = ラベル。`isLabelHidden` でも name が残る。
- [ ] md 高さ 32px（sm 28 / lg 36）、radius 10px、枠 1px `#d4d4d4`（dark `#525252`）、背景 `#ffffff`（dark `#262626`）。
- [ ] hover で内側 2px の淡いリング、フォーカスで枠が `#1b1b1b`（dark `#f1f1f1`）に 125ms で変わる。
- [ ] padding / アイコンクリックで入力にフォーカス。クリアボタン（name "Clear {label}"、ツールチップ "Clear"）で値が空になりフォーカスが入力に戻る。値が空・disabled・readOnly ではクリアが出ない。
- [ ] Enter で on_enter、IME 変換確定の Enter では呼ばれない。
- [ ] status error: 枠 `#c0262c`（dark `#ffaea7`）、XCircle 20px、aria-invalid=true。warning/success は invalid にならない。
- [ ] disabled: 0.5 減光、Tab でフォーカスでき、キーボードフォーカス時のみリング、disabledMessage がツールチップで出る（describedby に含まれる）。
- [ ] loading: 14px スピナーがクリアの左に出て aria-busy、"Loading" が 1 回アナウンスされる。
- [ ] docsite（light/dark）の Default / Sizes / Status / Clearable / Disabled と目視比較。

---

## 2. TextArea

ソース: `packages/core/src/TextArea/TextArea.tsx`（750 行）、`TextArea.doc.mjs`、`TextArea.test.tsx`、共有は TextInput と同じ（`inputStyles.stylex.ts`, `inputDisabledFocus.ts`, `useInputContainer.ts`, `useInputStatusIcon.tsx`, `textEntryTraits.ts`）＋ `utils/characters.ts`（`characterCount`）、`hooks/useAnnounce.ts`。Storybook: `TextArea.stories.tsx`（CustomRows / SizeVariants / WithMaxLength / MaxLengthOverLimit / Loading / LoadingWithStatus / CombinedFeatures / VerticalFormAlignment …）。Docs: https://astryx.atmeta.com/components/TextArea

### 2.1 概要

複数行の長文（コメント・説明・メッセージ）用の入力。高さは `rows`（既定 3）で決まり、ユーザーが縦方向にリサイズできる。`maxLength` で文字数カウンタを表示（強制はしない）。

- 使う: 複数行になる入力。上限がある時は `maxLength`＋カウンタ。status で inline 検証（success / warning=ソフト制限 / error=ハード失敗）。
- Return は改行のまま。`enterKeyHint`（send / done）で表示を変えるのは自前の Enter 処理がその通りに動く時だけ。
- 避ける: 名前・メールなど短い 1 行 → TextInput。placeholder だけで目的を伝えない。status type なしでメッセージだけ出さない。disabled 理由は `disabledMessage`。

### 2.2 Anatomy

```
Field (column, gap 4px)                                   ← Field セクション
├─ FieldLabel / description
└─ input container  div.astryx-text-area  data-size data-status data-disabled data-readonly
    (display:block, padding 0, position relative = オーバーレイの基準, --_textarea-inline-padding: 8px)
    ├─ startIcon overlay   span position:absolute top 8px, inset-inline-start 8px, pointer-events none
    │    └─ Icon sm 16px secondary
    ├─ <textarea class=astryx-text-area-control rows={rows}>   width 100%, resize: vertical
    ├─ end slot overlay    span absolute top 8px, inset-inline-end 8px, flex gap 8px, pointer-events none
    │    ├─ Spinner sm (aria-hidden)          (isBusy)
    │    └─ status affordance (Icon md / tooltip button は pointer-events:auto)
    └─ counter  div.astryx-text-area-counter  id=counterID  absolute bottom 4px, inset-inline-end 8px
         ├─ Icon warning sm (超過時のみ, 装飾)
         └─ "{count}/{limit}"
├─ disabledMessage tooltip layer
└─ FieldStatus
```

```
┌───────────────────────────────────────┐
│ ✎ Describe the issue in detail…    ◌ ⚠│ ← overlay は top 8px
│                                        │
│                                        │
│                                 12/200 │ ← counter bottom 4px
└──────────────────────────────────────◢┘ ← ネイティブ resize グリップ（右下）
```

### 2.3 API

| Astryx prop | 型 / 既定 | 意味 | Kivact 案 |
|---|---|---|---|
| `label`, `value`, `onChange`, `changeAction`, `isLoading`, `placeholder`, `isLabelHidden`, `description`, `isOptional`, `isRequired`, `isDisabled`, `isReadOnly`, `disabledMessage`, `status`, `statusVariant`, `width`, `labelTooltip`, `startIcon`, `hasAutoFocus`, `htmlName`, `autoComplete`, `container*/control*`, traits | TextInput と同じ | 同上 | `soft::text_area(label, value)` に TextInput と同じメソッド群（`htmlName`/`autoComplete`/traits は drop） |
| `rows` | number / 3 | 表示行数（高さ） | `.rows(u32)` |
| `size` | `'sm'\|'md'\|'lg'` / `'md'` | 内側 padding のみ（高さは rows） | `.size(ControlSize)` |
| `maxLength` | number | 上限（書記素数）。カウンタ表示、超過で error 表示＋aria-invalid。**入力を止めない** | `.max_length(usize)`（kit 側の表示用。既存 `TextInput::max_length` はハード制限なので**使わない**） |
| `hasSpellCheck` | bool / **true** | スペルチェック | drop |
| `onPaste` | `(e) => void` | 貼り付け | drop（必要なら binding の write で検出） |
| `onFocus` / `onBlur` | `(e) => void` | | `.on_focus(Fn())` / `.on_blur(Fn())`（`NodeRef::focused()` を `effect_on`） |
| ※ `onEnter` / `onKeyDown` なし | — | — | NodeBuilder `.on_key(..)` は使える |

### 2.4 Visual spec

#### ボックス（Field 共通の input container spec, 4.4 参照）— 解決値と差分

ボックスは `inputWrapperStyles.base` を使うが `styles.wrapper`（`TextArea.tsx:89-105`）で上書き:
- display **block**（flex ではない）、padding **0**（block/inline とも）、z-index 1。高さ指定なし（textarea の高さで決まる）。
- 枠 1px `--color-border-emphasized`（`#d4d4d4` / `#525252`）、radius 10px、背景 `#ffffff` / `#262626`、hover リング inset 2px `#d4d4d44D` / `#5252524D`、focus-within 枠 `#1b1b1b` / `#f1f1f1` ＋ inset 2px `#f1f1f1` / `#262626`、transition 125ms cubic-bezier(0.24,1,0.4,1) — すべて TextInput と同一。
- `--_textarea-inline-padding: --spacing-2` = 8px（テーマ変数。textarea・アイコン・カウンタが同じ値に揃う）。

#### textarea（`TextArea.tsx:106-134`）

| 項目 | 値 |
|---|---|
| 幅 | 100%（ボックス全幅）, min-width 0, box-sizing border-box |
| padding | block `--spacing-1` 4px（lg は `--spacing-2` 8px, `:193-199`）、inline 8px |
| 文字 | Figtree 14px / 20px / 400, 色 `#000000` / `#ffffff`, placeholder `#474747` / `#9e9e9e` |
| 背景 / 枠 / outline | 透明 / なし / none |
| resize | vertical（ネイティブ右下グリップ。ボックス padding 0 なのでグリップはボックス内側の真の角） |
| 高さ（rows=3, 枠込み） | sm・md: 3×20 + 4×2 + 1×2 = **70px**、lg: 60 + 16 + 2 = **78px**。rows=n で 20n + 10（lg 20n + 18） |

予約 padding（オーバーレイと文字が重ならないように）:
- startIcon あり: `padding-inline-start: calc(8px + --spacing-6)` = 8 + 24 = **32px**（`:140-142`）。
- スピナー or ステータスアイコンが**実際に描画される**時: `padding-inline-end` 32px（`:145-147`）。両方: 8 + `--spacing-12` 48 = **56px**（`:150-152`）。detached 配置ではアイコンが出ないので予約しない（`:703-709`）。
- maxLength あり: `padding-bottom: --spacing-7` = **28px**（`:154-156`）。例: md rows=3 ＋カウンタ = 60 + 4 + 28 + 2 = 94px。

オーバーレイ:
- startIcon: absolute, top 8px, inline-start 8px, 16px, 色 `--color-icon-secondary` `#6a6a6a` / `#9e9e9e`, pointer-events none（`:157-163`）。
- end slot: absolute, top 8px, inline-end 8px, flex, align center, gap 8px, pointer-events none（tooltip ボタンだけ auto）。中身: Spinner sm（14px）→ ステータスアイコン md 20px（`:164-172`, `:714-719`）。
- カウンタ（`:175-190`）: absolute, bottom 4px, inline-end 8px, flex, align center, gap 4px, pointer-events none。文字 `--text-supporting-size` 12px / `--text-supporting-leading` 1.6667（20px）/ 400, 色 `--color-text-secondary` `#474747` / `#9e9e9e`。表示 `{count}/{limit}`（数値はロケール整形）。

#### 状態

| 状態 | 変化 |
|---|---|
| rest / hover / focus | TextInput と同じ（ボックス）。フォーカスは textarea にあり :focus-within で枠 accent |
| disabled | ボックス opacity 0.5、hover リングなし、キーボード focus-visible 時のみリング復活（`inputDisabledFocusStyles.ring`）、textarea cursor default、編集不可 |
| readOnly | 変化なし（編集不可のみ） |
| loading | end slot に Spinner、`padding-inline-end` 32px（ステータスもあれば 56px） |
| status error / warning / success | 枠・hover シャドウ・アイコン色は TextInput と同値（error `#c0262c`/`#ffaea7` 枠, hover `#de47454D`; warning `#836700`/`#eec448`, `#f8d36a4D`; success `#2f7d33`/`#90ca90`, `#2f7d334D`）。status 中は focus inset リングが消える点も同じ |
| カウンタ通常 | `#474747` / `#9e9e9e` |
| カウンタ 80% 以上 | 見た目の変化なし（読み上げだけ） |
| カウンタ超過（count > limit） | 色 `--color-text-error` `#c0262c` / `#ffaea7`、先頭に warning アイコン（AlertTriangle）sm 16px（色 inherit = error 色）、textarea に `aria-invalid="true"`（ボックス枠は変えない） |

レイアウト: ボックス幅 = Field 幅。文字は折り返す（ネイティブ textarea のソフトラップ）、行数を超えると textarea 内で縦スクロール。RTL: オーバーレイ・padding は論理プロパティで反転（カウンタは左下、ネイティブ resize グリップは Chrome では左下に移る）。horizontal-labels の FormLayout ではラベルが `padding-top: 1px + 4px` で 1 行目に揃う（Field 側）。

### 2.5 Motion

| 対象 | トリガ | プロパティ | 時間 / easing | reduced | 出典 |
|---|---|---|---|---|---|
| ボックス | hover / focus / status / disabled | border-color, box-shadow | 125ms cubic-bezier(0.24, 1, 0.4, 1) | 0s | `inputStyles.stylex.ts:58-63` |
| カウンタ色・アイコン | 超過の出入り | — | 即時（transition なし） | — | `TextArea.tsx:175-190` |
| Spinner sm | busy | dashoffset 0 → −31.416px | 525ms linear infinite | 3s/周 | `Spinner.tsx:228-231, 338-350` |
| オーバーレイ / カウンタの出現 | — | — | 即時 | — | — |
| ツールチップ | — | — | Tooltip セクション参照 | — | — |

### 2.6 Behaviour & a11y

- キーボード: ネイティブ複数行編集。Enter は改行（onEnter なし）。Tab は textarea から抜ける（ネイティブ textarea は Tab を文字として入れない）。
- ポインタ: ボックス内の非インタラクティブ部分クリックで textarea にフォーカス。**disabled または busy 中はコンテナクリックの転送をしない**（`effectivelyDisabled = isDisabled || isBusy`, `TextArea.tsx:581-589`）。
- リサイズ: ネイティブ縦リサイズグリップ（ドラッグで高さ変更、幅は不変）。
- 値: disabled / readOnly 中は handleChange 無効。`changeAction` は TextInput と同じ楽観更新＋busy。
- カウンタ: `characterCount`（書記素単位。絵文字・国旗シーケンスは 1 文字）。maxLength がある時だけ計測。ゾーン（under < 80% ≤ near ≤ 100% < over）が**変わった時だけ**アナウンス: near → polite「{n} characters remaining」、over → assertive「{n} characters over the limit」。under に戻っても何も言わない（`TextArea.tsx:517-548`）。
- a11y: role textbox（multiline）、name = Field ラベル（caller の aria-labelledby がある時のみ `fieldLabelID + caller`）。`aria-describedby` = caller → description → ステータスメッセージ（tooltip 以外）→ tooltip id → **counterID** → disabledMessage tooltip。`aria-invalid` = status error **または** 超過。`aria-required`、`aria-busy`、`aria-disabled` + readOnly。busy 開始で「Loading」アナウンス。

既定テキスト:
| Astryx key | 英語 | soft キー案 |
|---|---|---|
| `@astryx.textArea.counter` | `{count, number}/{limit, number}` | `soft.text_area.counter` |
| `@astryx.textArea.charactersRemaining` | `{count} character remaining` / `{count} characters remaining`（plural） | `soft.text_area.remaining_one` / `soft.text_area.remaining_other` |
| `@astryx.textArea.charactersOverLimit` | `{count} character over the limit` / `… characters over the limit` | `soft.text_area.over_limit_one` / `soft.text_area.over_limit_other` |
| ほか status / spinner / clear tooltip は TextInput と共通 | | |

### 2.7 Kivact 実装方針

戦略: **RESTYLE `kivact::ui::text_area` + kit wrapper `soft::text_area`**。

- RESTYLE（`UiRecipes::text_area`）: ボックス look（枠・radius 10・背景・focus 枠＋inset シャドウ・hover リング・transition 125ms）＋ `padding_y(4/8).padding_x(8)`、`font_size(14).line_height(20/14)`、`min_height(rows=3 相当 70)`。参考 `crates/kivact-ui-basic/src/recipes.rs:342-349`（text_area）と `input_group.rs:412-414`（`input_group_textarea`）。
- wrapper: 外側ノード（position relative 相当のスタック: kivact の absolute 配置 / overlay 子）にボックス look、内側 `kivact::ui::text_area` は borderless（`input_group.rs` の `borderless`）。startIcon / end slot / カウンタは `hit_testable(false)` の絶対配置子。予約 padding は上記の値を内側フィールドに与える。
- **難所 1（ソフトラップ）**: kivact のネイティブ editor は「scroll horizontally and do not wrap」（`kivact-core/src/primitive/text_editing.rs:164`、basic の recipes.rs:337 コメントでも "a native text area does not wrap"）。Astryx の折り返し表示はそのまま再現できない → UNSUPPORTED。フォールバック: 折り返しなし・横スクロールで実装し、カタログ/README に差分として明記（エンジン側に wrap オプションが入れば差し替え）。
- **難所 2（rows / resize）**: 高さは `rows` から `20*rows + 2*pad + 2` を計算して `height(..)` に与える。ネイティブ resize グリップは無いので、kit で右下 12×12 程度のグリップノードを描き `kivact::interaction::use_drag` で縦方向の高さ signal を更新（最小 = 1 行分）。グリップ形状は Chrome の斜線 2 本（`--color-text-secondary` 相当）で近似。
- カウンタ: 書記素数は `kivact` 依存のみで数える必要がある（unicode-segmentation 不可）→ 近似として `chars().count()` から結合文字（U+0300–036F, U+200D ZWJ 連結, 異体字セレクタ U+FE0F, 地域指示子ペア, 肌色修飾 U+1F3FB–1F3FF）を除く簡易クラスタ計数を kit 内に実装。アナウンスは `live_region(LiveRegion::Polite/Assertive)` の不可視ノードに文字列を書き込む。
- `max_length` を既存 `TextInput::max_length` に渡さない（Astryx は超過入力を許す）。
- 落とすもの: spellcheck、onPaste、traits、forced-colors、iOS フォント下限。

### 2.8 受け入れ確認

- [ ] role textbox（multiline）、name = ラベル、Enter で改行。
- [ ] rows=3・md で高さ 70px（lg 78px）、padding 4/8（lg 8/8）、radius 10px、枠 `#d4d4d4` / `#525252`。
- [ ] フォーカスで枠 `#1b1b1b` / `#f1f1f1` に 125ms で遷移。padding クリックでフォーカス、busy・disabled 中はクリック転送しない。
- [ ] startIcon は左上 (8,8) に 16px、文字開始は 32px から。ステータスアイコンは右上 (top 8, end 8)。
- [ ] maxLength=10 で "12/10" が赤（`#c0262c` / `#ffaea7`）＋警告アイコン、aria-invalid=true、80% 到達で "N characters remaining" が polite、超過で "N characters over the limit" が assertive に 1 回ずつ。
- [ ] 縦リサイズグリップで高さが変わる（幅は変わらない）。
- [ ] docsite（light/dark）の Default / WithMaxLength / MaxLengthOverLimit / Loading / StatusVariations と比較（折り返し差分は既知）。

---

## 3. NumberInput

ソース: `packages/core/src/NumberInput/NumberInput.tsx`（1183 行）、`numberParser.ts`（365 行）、`numberInputCommit.ts`（103 行）、`NumberInput.doc.mjs`、`NumberInput.test.tsx`（2725 行）、`numberParser.test.ts`、`numberParser.docblock.test.ts`、`numberInputCommit.test.ts`。共有: `inputStyles.stylex.ts`, `inputDisabledFocus.ts`, `InputClearButton.tsx`, `useInputContainer.ts`, `useInputStatusIcon.tsx`, `utils/interactionOverlay.stylex.ts`, `hooks/usePressFeedback.ts`, `utils/unicodeDigits.ts`, `utils/ime.ts`。Storybook: `NumberInput.stories.tsx`（PasteFormattedNumber / WithMinMax / WithStep / FormattedDisplay / WithNumberSteppers / WheelBehavior / WithUnits / IntegerOnly / Clearable / ClearableWithUnits …）。Docs: https://astryx.atmeta.com/components/NumberInput

### 3.1 概要

数量・寸法・割合などの数値入力。テキストとして自由に打てて（貼り付けた `1,234,234,234` もロケールで読む）、**blur か Enter で一括コミット**し、有効な値の時だけ `onChange` を呼ぶ。min/max でクランプ、step で矢印キー・ホイール・（任意の）ステッパーボタンによる増減。単位表示、整形表示（`formatValue`）、クリア。

- 使う: min / max / step で有効値へ誘導。単位（"%", "GB"）を表示。スクロール面の中では `isWheelEnabled={false}`。
- 避ける: 数字を含む自由テキスト → TextInput。isOptional と isRequired の同時指定。disabled 理由は `disabledMessage`。

### 3.2 Anatomy

```
Field (labelIcon も渡せる)                                  ← Field セクション
└─ input container  div.astryx-number-input  data-size data-status data-disabled data-readonly
    ├─ startIcon               Icon sm 16px secondary                     (任意)
    ├─ VisuallyHidden label    (InputGroup 内のみ)
    ├─ <input type="text" role="spinbutton" inputMode="decimal|numeric">  flex 1
    ├─ <input type="hidden" name>   (formatValue && htmlName && !disabled)
    ├─ units  <span id=unitsID>     14/20 secondary, shrink 0              (任意)
    ├─ VisuallyHidden <div role="alert" aria-live="assertive">  "Invalid number" | ""
    ├─ InputClearButton             (hasClear && value != null && !disabled && !readOnly)
    ├─ status affordance            (Icon md / tooltip button)
    └─ number steppers column       (hasNumberSteppers; pointer:coarse では display none)
         ├─ <button tabIndex=-1 aria-label="Increment {label}">  chevron (180° 回転)
         └─ <button tabIndex=-1 aria-label="Decrement {label}">  chevron, border-top 1px
```

```
┌──────────────────────────────────────┬──┐
│ #  1,250            GB    ✕   ⚠      │ ˄│  h=32, steppers 列 幅 16px, 各 15px
│                                      ├──┤
│                                      │ ˅│
└──────────────────────────────────────┴──┘
```

### 3.3 API

| Astryx prop | 型 / 既定 | 意味 | Kivact 案 |
|---|---|---|---|
| `label` | string（必須） | ラベル | `soft::number_input(label, value)` 第 1 引数 |
| `value` | `number \| null \| undefined`（必須） | コミット済み値。null/undefined = 空 | 第 2 引数 `impl Into<Binding<Option<f64>>>` |
| `onChange` | `hasClear` なし: `(number) => void` / `hasClear: true`: `(number \| null) => void` | 有効なコミットだけ通知（判別共用体） | binding の write。`.clearable(true)` でない時は kit が `None` を書かない |
| `hasClear` | bool / false | クリアボタン、空コミットで null | `.clearable(bool)` |
| `min` / `max` | `number \| null` | 範囲。範囲外はコミット時にクランプ | `.min(f64)` / `.max(f64)`（無制限可。`ScalarRange` は有限必須なので使わない） |
| `step` | `number \| null` / 1 | 増減量（無効値・≤0・非有限・integerOnly で非整数なら 1） | `.step(f64)` |
| `isIntegerOnly` | bool / false | 整数のみ。inputMode numeric | `.integer_only(bool)` |
| `formatValue` | `(number) => string` | 非編集時の表示整形。フォーカス中は生値 | `.format_value(Fn(f64) -> String)` |
| `isWheelEnabled` | bool / **true** | フォーカス中のホイールで増減 | `.wheel(bool)` |
| `hasNumberSteppers` | bool / false | 末尾の ▲▼ ボタン（coarse pointer では非表示） | `.steppers(bool)` |
| `units` | `string \| null` | 末尾の単位テキスト | `.units(impl IntoText)` |
| `startIcon` | ReactNode \| IconType | 先頭アイコン（truthy 判定） | `.start_icon(impl IntoView)` |
| `labelIcon` | ReactNode \| IconType | ラベル前アイコン（Field に渡す） | `.label_icon(impl IntoView)` |
| `onFocus` / `onBlur` / `onEnter` / `onKeyDown` | | onKeyDown は ArrowUp/Down を先に受け、preventDefault で増減を取り消せる | `.on_focus` / `.on_blur` / `.on_enter` / `.on_key`（`Handled::Yes` で増減取り消し） |
| `size`, `status`, `statusVariant`, `isDisabled`, `isReadOnly`, `disabledMessage`, `placeholder`, `width`, `labelTooltip`, `description`, `isLabelHidden`, `isOptional`, `isRequired`, `hasAutoFocus` | TextInput と同じ（readOnly は増減も全経路で無効） | | TextInput と同じメソッド |
| `htmlName` / `autoComplete` | | フォーム送信（formatValue 時は hidden input で生値） | drop |
| （locale） | `useLocale()` | 解析・編集表示のロケール | `kivact::i18n::locale()` を読む。`.locale(Locale)` で上書き可 |

### 3.4 Visual spec

#### ボックス（Field 共通の input container spec, 4.4 参照）— 解決値と差分

- 高さ sm/md/lg 28/32/36px、枠 1px `#d4d4d4` / `#525252`、radius 10px、背景 `#ffffff` / `#262626`、gap 8px、hover リング inset 2px `#d4d4d44D` / `#5252524D`、focus-within 枠 `#1b1b1b` / `#f1f1f1` ＋ inset 2px `#f1f1f1` / `#262626`、transition 125ms cubic-bezier(0.24,1,0.4,1) — TextInput と同一。
- padding は shorthand でなく 4 辺個別（`NumberInput.tsx:80-101`）: 既定 上 4 / 下 4 / start 8 / end 8px。テーマ変数 `--astryx-number-input-padding[-inline|-block][-start|-end]` で上書き可能（ステッパー列が追従するため）。
- `hasNumberSteppers` かつ fine pointer: **padding-inline-end 0**（ステッパー列が右端の枠に接する, `:102-107`）。coarse pointer では通常の 8px に戻りステッパー非表示。

#### input・units

- input（`:108-132`）: TextInput と同じ（Figtree 14/20, `#000000` / `#ffffff`, placeholder `#474747` / `#9e9e9e`, flex 1, min-width 0）。**テキスト揃えは start（中央寄せではない）**。
- 無効な下書き中（`!isInputValid`）: 入力文字色が `--color-text-secondary` `#474747` / `#9e9e9e` に変わる（`:136-138`）。ボックス枠は変えない。
- units（`:139-145`）: Figtree 14px / 20px、`--color-text-secondary` `#474747` / `#9e9e9e`、flex-shrink 0、input の直後（gap 8px）。

#### ステッパー列（`:146-202`）

| 項目 | 値 |
|---|---|
| 列 | flex column, align-self stretch, 幅 `--spacing-4` = **16px**, shrink 0。上下 margin = −padding（−4px）で枠内の全高に伸ばす（md: 32−2 = 30px） |
| 列の左境界 | `border-inline-start` 1px `--color-border-emphasized` `#d4d4d4` / `#525252` |
| 角 | overflow hidden、`border-start-end-radius` / `border-end-end-radius` = `var(--_field-radius)` = 10px |
| ボタン（各） | flex 1 → md で各 15px（sm 13 / lg 17）、padding 0、枠なし、背景 `--color-background-surface`、色 `--color-icon-secondary` `#6a6a6a` / `#9e9e9e`、cursor pointer、outline none |
| 下（decrement）ボタン | `border-block-start` 1px `#d4d4d4` / `#525252`（box-sizing border-box なので 15px に含む） |
| アイコン | `numberInput:stepperDown`（neutral レジストリに無いので core 既定 SVG: viewBox 24, path `M7.5 9.75l4.5 4.5 4.5-4.5`, stroke currentColor 1.5, round cap/join）size **xsm = 12px**。increment は `rotate(180deg)` |
| hover（hover:hover, 有効時） | 背景画像 `--color-overlay-hover` `#0000000D` / `#ffffff0D` を surface の上に重ねる |
| pressed（:active） | `--color-overlay-pressed` `#0000001A` / `#ffffff1A` |
| 無効（disabled / readOnly / 境界到達） | 色 `--color-icon-disabled` `#919191` / `#525252`、cursor default、overlay なし（`aria-disabled`） |
| focus | ボタンはタブ不可・フォーカスを奪わないので focus look なし |

#### 状態

| 状態 | 変化 |
|---|---|
| rest / hover / focus | TextInput と同じ |
| 無効な下書き | 文字色 secondary、`aria-invalid="true"`、assertive alert「Invalid number」。※**範囲外の値（例 max=5 で "10"）も parseNumberInput では null なので「無効」表示になる**が、コミット時はクランプされる |
| disabled | ボックス 0.5、hover なし、キーボード focus-visible 時のみリング、ステッパー両方無効色、クリア非表示 |
| readOnly | 見た目変化なし。クリア非表示、ステッパー両方無効色（aria-disabled）、矢印・ホイール無効 |
| status error / warning / success | TextInput と同値（枠・hover シャドウ・アイコン、focus 時 inset リングなし）。error で aria-invalid |
| 境界到達（value == max 等） | 該当ステッパーだけ無効色 |

レイアウト: ボックス幅 = Field 幅。input は flex 1 で残りを取る。RTL: 論理プロパティでステッパー列が左端・角丸は左側（start-end / end-end）、chevron は上下方向なので反転不要。

### 3.5 Motion

| 対象 | トリガ | 内容 | 時間 / easing | reduced | 出典 |
|---|---|---|---|---|---|
| ボックス | hover / focus / status | border-color, box-shadow | 125ms cubic-bezier(0.24, 1, 0.4, 1) | 0s | `inputStyles.stylex.ts:58-63` |
| ステッパー hover / pressed | pointer | background-image overlay | **即時**（stepper には transition 宣言なし、グラデーションは補間不可） | — | `interactionOverlay.stylex.ts:245-266` |
| ステッパー touch press | — | coarse では列ごと非表示なので発生しない | — | — | `NumberInput.tsx:150-153` |
| 無効文字色 / alert | 下書きの有効性変化 | 即時 | — | — | — |
| クリアボタン | TextInput と同じ（scale 0.98, 125ms） | | | | |
| 値の変化（増減） | — | 数値の補間なし（即時置換） | — | — | — |

### 3.6 Behaviour & a11y

状態: `value`（caller のコミット値）、`pendingInput: string | null`（下書き、null = 下書きなし）、`isFocused`。

表示文字列（`NumberInput.tsx:687-704`）:
1. pending があればそれをそのまま。
2. value が null → `""`。
3. フォーカス中 → `formatEditableNumber(value, locale)` = JS `String(value)` の `.` をロケール小数点に置換（de-DE `1.5` → `"1,5"`。グルーピングなし）。
4. 非フォーカス → `formatValue(value)`（あれば）／`String(value)`。

入力（タイプ・貼り付け）: `pendingInput = text` のみ。**打鍵中にクランプ・onChange は一切しない**（"does not clamp while typing"）。disabled / readOnly は無視。

有効性（表示用, `:707-712`）: pending が null か空白のみ → 有効。それ以外は `parseNumberInput(text, {min,max,isIntegerOnly,locale})` が null でなければ有効（= ロケール解析成功 ∧ integerOnly なら整数 ∧ min ≤ v ≤ max）。

コミット `resolveNumberInputCommit`（`numberInputCommit.ts:432-468`）:
- 空白のみ → hasClear なら `clear`（value ≠ null の時だけ `onChange(null)`）、なければ `revert`。
- 解析失敗 or integerOnly で非整数 → `revert`（onChange しない）。
- 範囲: integerOnly なら min を ceil・max を floor（内側へ丸め）。min > max → `revert`。範囲外は境界値にクランプして `commit {value, didClamp}`。
- commit で value と異なる時だけ `onChange(v)`。
- **blur**: 必ず pending を破棄（以後は制御値を表示 → revert 時は元の値に戻る、クランプ時はクランプ値）、`isFocused=false`、`onBlur`。
- **Enter**: 同じ判定。pending を破棄するのは「クランプ付き commit」の時だけ（それ以外は打った文字列がそのまま残る。revert 時も無効文字列が残る）。その後 `onEnter()`、`onKeyDown(e)`。
- IME 変換中の Enter / 矢印は無視（`isImeKeyEvent`, `:873-875`）。
- 途中で disabled になった後の blur は何もコミットしない。
- 制御値が外から変わっても、フォーカス中の下書きがあれば下書きを優先表示（"keeps a controlled update behind an invalid focused draft"）。

フォーカス時（`:736-757`）: Tab で入った時のように全選択された状態でフォーカスが来て、表示が整形値→編集値に変わる場合は、差し替え後に再度全選択する（タイプで置換できるように）。クリックで入った時はキャレット位置を保つ。

増減 `getSteppedValue`（`:469-542`）:
- 有効 step = step（有限・> 0・integerOnly なら整数）でなければ 1。step 基点 = min（非 null かつ integerOnly なら整数の時）、なければ 0。
- 現在値 = pending なし → value ／ pending 空白 → null ／ それ以外 → parse(pending) ?? value。
- 現在値 null: +1 → `min ?? 0`、−1 → `max ?? 0`（integerOnly は ceil / floor）。
- それ以外: pos = (cur − base)/step、tol = ε·max(1,|pos|)·4。+1: `floor(pos+tol)+1`、−1: `ceil(pos−tol)−1`、値 = base + n·step（オフステップ値は要求方向のステップ位置に揃う: 0.25, step 0.1 → ↑0.3 / ↓0.2）。
- 小数精度 = min(12, max(step の小数桁, base の小数桁)) で `toFixed` 丸め（0.1+0.2 問題回避）→ その後 min/max でクランプ。非有限・integerOnly で非整数なら現在値のまま。−0 → 0。
- `stepValue`: disabled / readOnly なら何もしない。pending を破棄し、値が変わる時だけ `onChange`。
- `canIncrement = next(+1) !== 現在値`、`canDecrement` 同様（ステッパーの aria-disabled）。

キーボード:
| キー | 動作 |
|---|---|
| ArrowUp / ArrowDown（修飾なし） | まず `onKeyDown(e)`。`defaultPrevented` なら終了。そうでなければ preventDefault して ±1 step |
| 修飾付き矢印 | ネイティブ（キャレット移動等） |
| Enter | コミット（上記）→ `onEnter` → `onKeyDown` |
| PageUp / PageDown / Home / End | **特別扱いなし**（ネイティブのキャレット操作） |
| Escape | 特別扱いなし（下書きは残る） |
| Tab | 次へ（blur コミット）。ステッパーボタンは tabIndex −1 なので飛ばす |
| disabled 時 | Enter は preventDefault のみ、他は何もしない |

ホイール（`:898-928`）: 非 passive のネイティブ listener。`document.activeElement === input`、非 disabled / readOnly、deltaY ≠ 0、修飾キーなしの時だけ preventDefault + stopPropagation（ページはスクロールしない）し、deltaY < 0 → +1、> 0 → −1。1 イベント 1 step（delta 量は無関係）。`isWheelEnabled=false` やフォーカスなしではページスクロールに任せる。

ステッパーボタン: pointerdown を preventDefault（フォーカスを奪わない）。click で（無効でなければ）input にフォーカスして ±1 step。無効時の click は何もしない。

クリア: `onChange(null)`（hasClear 時）、pending 破棄、input にフォーカス。表示条件は**コミット値**が非 null（下書きでなく）。

ポインタ: ボックスの非インタラクティブ部分クリックで input にフォーカス（role spinbutton の input にも `.focus()`）。

ロケール解析 `parseLocaleNumber`（`numberParser.ts:262-343`）— Codex が Rust に移植する仕様:
1. NFKC 正規化、不可視文字 `[U+200B–U+200D, U+200E, U+200F, U+061C, U+2066–U+2069, U+FEFF]` 除去、Unicode 10 進数字（\p{Nd}）を ASCII へ（`utils/unicodeDigits.ts`）。trim。
2. 会計表記 `( … )` → 負。先頭・末尾の通貨記号（\p{Sc}）を 1 つずつ除去。
3. 符号は先頭または末尾（`+`, `-`, U+2212, U+2012, U+2013）。括弧と符号の併用は null。再度通貨除去。
4. 末尾の指数 `e±n` を分離。
5. 本体は `[0-9 , . ' space U+2019 U+066B U+066C]+` のみ許可（`SEPARATOR_CHARS`）。
6. ロケール記号（Intl から: group, decimal, primary/secondary グループ桁, en-IN は 3/2）で読む: 小数点は 1 回まで、グループは「最後 = primary 桁ちょうど、内側 = secondary 桁ちょうど、最初 ≤ secondary 桁」、グループ区切りは 1 個以上必要。成功ならそれ。
7. 失敗なら機械形式 `^(\d+\.?\d*|\.\d+)$`。
8. それでも失敗なら: 区切り文字種が 1〜2 種の時、各候補を「3 桁グループ」として試す（空白・アポストロフィ・U+2019 は 1 回で可、`,` `.` は 2 回以上）。もう一方は小数点（1 回のみ）。
9. `Number()`、指数があれば `toPrecision(15)` で丸め。非有限 → null。−0 → 0。

代表例（`numberParser.test.ts`）: `1,234,234,234` en-US / de-DE → 1234234234。`1,234` en-US → 1234、de-DE → 1.234。`1.234` de-DE → 1234、`1.2345` de-DE → 1.2345。`1,5` de-DE → 1.5、en-US → null。`1,234.56` de-DE → null。`$1,234.56` en-US → 1234.56。`1.234,56 €` de-DE → 1234.56。`.5` → 0.5、`5.` → 5、`-` → null、`123-456-789` → null、`1 234` en-US → 1234、`12,34,567` en-US → null（en-IN なら有効）、アラビア数字 `١,٢٣٤` → 1234。

a11y:
- input: `type="text"`、`role="spinbutton"`、`inputMode` numeric（integerOnly）/ decimal。`aria-valuemin` / `aria-valuemax`（設定時）、`aria-valuenow` = **コミット値**（下書きではない）、`aria-valuetext` = formatValue の結果（formatValue がある時だけ）。
- `aria-describedby` = caller → description → ステータスメッセージ → tooltip id → **unitsID** → disabledMessage tooltip。
- `aria-invalid` = status error または無効下書き。`aria-required`、`aria-disabled` + readOnly。
- 無効下書き中、VisuallyHidden の `role="alert" aria-live="assertive"` に「Invalid number」（有効になると空文字）。
- ステッパー: `aria-label` "Increment {label}" / "Decrement {label}"、`aria-disabled`、tabIndex −1。
- 名前規則・disabledMessage・status tooltip は TextInput と同じ。

既定テキスト:
| Astryx key | 英語 | soft キー案 |
|---|---|---|
| `@astryx.numberInput.clearLabel` | `Clear {label}` | `soft.number_input.clear` |
| `@astryx.numberInput.incrementLabel` | `Increment {label}` | `soft.number_input.increment` |
| `@astryx.numberInput.decrementLabel` | `Decrement {label}` | `soft.number_input.decrement` |
| `@astryx.numberInput.invalidNumber` | `Invalid number` | `soft.number_input.invalid` |
| クリアツールチップ / status / spinner | TextInput と共通 | |

### 3.7 Kivact 実装方針

戦略: **NEW kit component `soft::number_input`**（＋ 標準 `kivact::ui::number_input` を直接使う場合のために `NumberInputRecipes`（root / input）をボックス look に RESTYLE するのみ）。

標準 `kivact::ui::number_input`（`crates/kivact-ui/src/number_input.rs`）は採用できない: ① 値が `Binding<f64>`（空・null 不可）、② `ScalarRange` が有限境界必須、③ 打鍵ごとに即書き込み（Astryx は blur/Enter で一括コミット）、④ Rust `f64::parse`（ロケール・グルーピング・通貨・会計表記なし）、⑤ 減算｜入力｜加算の左右配置で `minus`/`plus` ボタン（Astryx は末尾の縦 2 段 chevron 列、任意）、⑥ PageUp/PageDown/Home/End を持つ。

構成（`soft::number_input(label, value: impl Into<Binding<Option<f64>>>)`）:
- ボックス = TextInput と同じ kit wrapper row（`input_group.rs` の `borderless` パターン）。子: `[start_icon][field][units][alert][clear][status][steppers]`。
- field = `kivact::ui::text_input(label, display_binding)`（borderless）。`display_binding` の read は「pending / 編集値 / 整形値」の memo、write は `pending.set(Some(text))`。
  - `.role(Role::SpinButton)`、`.value_min(..)` / `.value_max(..)`（Some の時）、`.value_now(..)`（コミット値）、`.value_text(..)`（formatValue 時）、`.invalid(move || status_error || !input_valid)`。
  - `.forward_keys(vec![Key::ArrowUp, Key::ArrowDown])` で矢印をハンドラへ回し、`.on_key(..)` で修飾なし時に caller の on_key → 未処理なら step。Enter は `.on_submit(..)` でコミット。
  - `.on_wheel(..)`: `field_ref.focused().get_untracked()` かつ有効時のみ `PreventDefault`/`Yes` を返して step（ページスクロール抑止）。
  - `.on_a11y(A11yAction::Increment/Decrement, ..)` で支援技術からの増減（kivact interaction の spin button と同様）。
  - blur コミット: `effect_on(field_ref.focused(), |now| if !now { commit(Blur) })`（`kivact-ui/src/number_input.rs` の focus 監視と同じ手法）。
  - IME: `TextInput` は「IME composition consumes keys first」なので Enter/矢印のガードはエンジン任せ（`.on_composition_change` で composing を保持して二重防御可）。
  - 既存 `TextInput::draft(..)` は Escape で取り消し・無効時は pending 継続と Astryx（blur で revert、Escape 無処理）と挙動が違うので**使わない**。
- `numberParser.ts` / `numberInputCommit.ts` / `getSteppedValue` を純関数として Rust に移植（単体テストは Astryx の test 表をそのまま移植: `numberParser.test.ts`, `numberInputCommit.test.ts`, NumberInput.test の stepping 群）。
  - 難所 1（Intl なし）: ロケールの group / decimal / グループ桁は kivact に API がない → kit 内の小さな表（en: `,` `.` 3/3、de/es/it/nl/pt-BR/id/tr: `.` `,`、fr/ru/pl/cs/sv/nb/fi/uk: U+202F/U+00A0→space `,`、de-CH: `’` `.`、en-IN/hi: `,` `.` 3/2、ar: U+066C U+066B、ja/zh/ko: `,` `.`）を `kivact::i18n::locale()` の言語/地域で引き、未知は en。
  - 難所 2（NFKC・\p{Nd}）: 外部 crate 不可 → 全角数字 U+FF10–FF19、空白族（U+00A0, U+2007, U+2009, U+202F, U+3000）→ space、全角記号（，．＋－）→ ASCII の限定的な折りたたみ＋ Nd の「0」コードポイント表（U+0660, U+06F0, U+07C0, U+0966, U+09E6, …, U+FF10）による桁変換を kit 内に持つ。
  - 難所 3（JS `String(number)`）: Rust の `f64` Display は指数表記を使わない（1e21 → "1000000000000000000000"、1e-7 → "0.0000001"）。JS 互換（≥1e21 / <1e-6 で指数）の `js_number_to_string` を実装するか、差分を許容して明記。
- ステッパー列: 幅 16px の column ノード（`border_left(1, border_emphasized)`、右側角 10px、clip）に 2 つの kit ボタンノード（`.focusable(false)`、`on_press` で `field_ref.focus()` → step、`.disabled(..)` = 境界 / disabled / readOnly）。アイコンは 12px の chevron（core 既定 path）を `icon(..)` 登録 or `draw` で描く。hover/pressed は `hover(style().background(overlay_hover))`, `pressed(..)`（即時、transition なし）。デスクトップは常に表示（coarse pointer 判定 API が無いため。タッチ専用ビルドでは非表示に）。
- 無効アラート: 不可視ノード `.role(Role::Alert).live_region(LiveRegion::Assertive)` に文字列を出し入れ。
- 参考ファイル: `crates/kivact-ui/src/number_input.rs`（focus 監視・value 表示の分離）、`crates/kivact-ui-basic/src/input_group.rs`（ボックス合成）、`crates/kivact-interaction/src/scalar.rs` 冒頭（spin button のキーと AT 連携の注意）。

### 3.8 受け入れ確認

- [ ] role spinbutton、name = ラベル、value_now = コミット値、min/max 設定時に範囲が公開される。
- [ ] "42" と打ってもフォーカス中は on_change されず、Tab（blur）で 42 がコミット。Enter でもコミットし on_enter が呼ばれる。
- [ ] max=5 で "10" → 入力中は文字が `#474747`、aria-invalid、"Invalid number" アラート。blur で 5 にクランプ表示。
- [ ] "abc" → blur で元の値に戻り on_change なし。hasClear で空にして blur → None がコミット。
- [ ] en-US で `1,234,234,234` 貼り付け → blur で 1234234234。de-DE で `1,5` → 1.5、フォーカス中の表示は `1,5`。
- [ ] ↑/↓ で ±step（0.25, step 0.1 → 0.3 / 0.2）、値 None で ↑ → min（なければ 0）。境界で止まり該当ステッパーが無効色。
- [ ] フォーカス中のホイールで増減しページはスクロールしない。フォーカスなし・`.wheel(false)` ではページがスクロール。
- [ ] ステッパー: 幅 16px・各 15px（md）、左境界 1px、上下の間に 1px 線、hover で `#0000000D` 重ね、Tab で止まらない、クリック後フォーカスは入力。
- [ ] format_value(`{n} items`) は非フォーカス時のみ、フォーカスで生値・全選択。
- [ ] readOnly: 見た目不変、矢印・ホイール・ステッパーすべて無効。
- [ ] docsite（light/dark）の WithNumberSteppers / WithUnits / FormattedDisplay / Clearable / StatusVariations と比較。


---

## 4. Field

Source: `packages/core/src/Field/` (`Field.tsx`, `FieldLabel.tsx`, `inputStyles.stylex.ts`, `fieldParts.ts`, `fieldStatusPlacement.ts`, `textEntryTraits.ts`, `inputDisabledFocus.ts`, `InputClearButton.tsx`, `PanelSearchInput.tsx`, `types.ts`)。Docs: https://astryx.atmeta.com/components/Field 。Stories: `apps/storybook/stories/Field.stories.tsx`, `FieldLabel.stories.tsx`。

この節は 2 つの役割を持つ。(a) 公開コンポーネント `Field` / `FieldLabel` の仕様。(b) **4.4 の「Field 共通の input container spec」**。TextInput / TextArea / NumberInput / DateInput / TimeInput / Selector / MultiSelector / Typeahead / Tokenizer / FileInput は全てこの共通 box を土台にしているので、それらの節は「Field 共通の input container spec (4.4)」を参照し、差分だけを書く。

### 4.1 概要

- `Field` は**低レベルの field shell**。label、description、optional/required indicator、label icon、label tooltip、status message (FieldStatus) を、caller が持っている control (native input、custom widget、3rd party) にまとめて付ける。
- 使う場面 (`Field.doc.mjs` bestPractices)
  - label / helper text / optional・required / validation status が必要な custom control、native control、third-party widget を包むとき。
  - label は常に渡す。見た目から隠すときは `isLabelHidden`。
  - `inputID` / `descriptionID` を使い、label (`htmlFor`) と description (`aria-describedby`) を内側の control に結び付ける。
  - `statusVariant="attached"` は、包む control が **不透明で枠線があり高さ固定** のときだけ使う。custom、背が高い、半透明、枠線なしの control には `detached` を使う。
- 使わない場面
  - TextInput / Typeahead / Selector / DateInput / TextArea を `Field` で包まない。これらは自分で Field shell を描く。
  - `isOptional` と `isRequired` を同時に指定しない (dev warning が出て、`isOptional` が優先される。`Field.tsx:248-252`)。
  - 別の手段で field の目的が伝わらないのに label を隠さない。
- Field-composed input の styling 契約 (`fieldParts.ts`, family:input-fields FR10): Field を内部で描く input は root の `xstyle`/`className`/`style` を**受け付けない**。field 全体 (label + control + status の box) は `containerXstyle/containerClassName/containerStyle`、枠と背景を描く control box は `controlXstyle/controlClassName/controlStyle` で styling する。InputGroup の中や `horizontal-labels` FormLayout の中では Field box が描かれないので、`container*` は効かない (dev warning、`fieldParts.ts:122-174`)。

### 4.2 Anatomy

既定 (vertical / horizontal FormLayout) の DOM (`Field.tsx:331-360`):

```
div.astryx-field  (flex column, gap 4px*, isolation:isolate, min-width:0, width?)
├─ FieldLabel
│   ├─ (description なし) <label for=inputID | span id=labelID> …
│   └─ (description あり) div (flex column, gap 0)
│        ├─ <label|span>.astryx-field-label  [labelIcon] label [ " ∙ " Optional|Required ] [info icon + Tooltip]
│        └─ span#descriptionID  description   (click → control に転送)
├─ statusVariant = 'attached':
│   div.inputStatusWrapper (flex column, isolation:isolate, --_field-status-overlap)
│   ├─ children (control。共通 input container は z-index:1)
│   └─ FieldStatus variant=attached (position:relative; z-index:-1)
└─ statusVariant = 'detached' | 'tooltip':
    ├─ children
    └─ FieldStatus variant=detached   ('tooltip' では描かない)
```
\* gap 4px は `isLabelHidden=false` のときだけ (`Field.tsx:339`)。

`horizontal-labels` FormLayout の中 (`Field.tsx:298-328`):

```
div.astryx-field[data-layout=horizontal-labels]  (display:contents → 親 grid `auto 1fr` に溶ける)
├─ div (padding-top: calc(1px + 4px) = 5px)   ← grid column 1
│   └─ FieldLabel (description は渡さない)
└─ div.inputStatusWrapper                      ← grid column 2
    ├─ description (Text type=supporting display=block id=descriptionID、hidden label なら VisuallyHidden)
    ├─ children
    └─ FieldStatus (variant=statusVariant)
```

Anatomy 名 (`Field.doc.mjs` anatomy): Container / Label / Ordinary description / Horizontal-label description / Control slot / Status message / Input status icon (26.2 参照) / Input clear button / Input clear icon / Optional/Required indicator / Label tooltip trigger / Label tooltip surface。

Theme targets: `astryx-field` (`data-layout`)、`astryx-field-label`、`astryx-field-status` (`data-type`,`data-variant`)、`astryx-input-status-icon` (`data-size`,`data-status`)、`astryx-input-clear-button`、`astryx-input-clear-icon`。Private vars: `--_field-radius` (= `--radius-element`)、`--_field-status-overlap`、`--_input-clear-hit-inset`、`--_input-clear-hit-content`。

### 4.3 API

#### Field props (`Field.tsx:110-203`)

| prop | type | default | 意味 | Kivact |
|---|---|---|---|---|
| `label` | `string` | 必須 | label 文字列。常に描く (隠しても AT 用に残る) | `soft::field(label, control)` の第 1 引数 |
| `inputID` | `string` | 必須 | label の `htmlFor` が指す control の id | **削除**。kit が `control.labelled_by(label_ref)` と `.name(..)` で自動配線 (`kivact::ui::field` と同じ) |
| `labelID` | `string` | – | label 要素自身の id (group が `aria-labelledby` で参照) | 削除。`.group_label(true)` のとき kit が `labelled_by` を張る |
| `isGroupLabel` | `boolean` | `false` | label を `<span>` で描く (radiogroup 等の group 名) | `.group_label(bool)`。true のとき label を押しても control を activate しない |
| `isLabelHidden` | `boolean` | `false` | label と description を視覚的に隠す (sr-only、レイアウト 0) | `.label_hidden(impl IntoValue<bool>)` |
| `description` | `string` | – | label と control の間の helper text | `.description(impl IntoText)` |
| `descriptionID` | `string` | `${inputID}-desc` | description の id | 削除 (`described_by(node_ref)` で自動) |
| `isOptional` | `boolean` | `false` | 「Optional」indicator | `.optional(impl IntoValue<bool>)` |
| `isRequired` | `boolean` | `false` | 「Required」indicator | `.required(impl IntoValue<bool>)`。両方 true なら optional 優先 (debug build で `eprintln!` 相当の警告) |
| `isDisabled` | `boolean` | `false` | 関連 control が disabled。label を disabled 色にする | NodeBuilder の `.disabled(..)` を root に付け、kit は `root.accessible_disabled()` を読んで label の look を切り替える (子 control は自動では disable されない。control 側にも渡す) |
| `labelIcon` | `ReactNode \| IconType` | – | label 先頭の icon (size sm, color inherit) | `.label_icon(impl IntoView)` (通常 `icon("name")`) |
| `labelTooltip` | `string` | – | label 末尾の info icon の tooltip | `.label_tooltip(impl IntoText)` |
| `status` | `{type, message?, messageID?}` | – | status。message があれば FieldStatus を描く | `.status(impl IntoValue<Option<soft::InputStatus>>)` (caller-owned signal / memo)。`messageID` は削除 |
| `statusVariant` | `'attached' \| 'detached' \| 'tooltip'` | `'attached'` | status の置き方 | `.status_placement(soft::StatusPlacement::{Attached, Detached, Tooltip})` |
| `width` | `number \| string` | – | field 全体 (label/control/status) の幅。number=px | `.width(impl Into<Length>)` (root の style に入れる。`pct(100)` 可) |
| `children` | `ReactNode` | 必須 | control | 第 2 引数 `control: C where C: NodeBuilder + IntoView` |
| `ref`, `xstyle`, `className`, `style` | – | – | root へ | `NodeBuilder` modifiers (`.style(..)`, `.node_ref(..)`, `.test_id(..)`)。`className` は削除 |

提案型:

```rust
pub enum StatusKind { Warning, Error, Success }          // InputStatusType
pub struct InputStatus { pub kind: StatusKind, pub message: Option<String> }
impl InputStatus { pub fn error(m: impl Into<String>) -> Self; pub fn warning(..); pub fn success(..);
                   pub fn marker(kind: StatusKind) -> Self /* message なし */ }
pub enum StatusPlacement { #[default] Attached, Detached, Tooltip }   // FieldStatusPlacement
```

`kivact::interaction::Tone` は流用しない: `Tone::Warning` は urgent (assertive) 扱いだが、Astryx の warning は polite (4.6)。

#### FieldLabel props (`FieldLabel.tsx:119-188`)

`label`, `inputID`, `labelID`, `isGroupLabel`, `isLabelHidden`, `isDisabled`, `isOptional`, `isRequired`, `labelIcon`, `labelTooltip`, `description: ReactNode` (Field と違い ReactNode、中に link/button を置ける), `descriptionID`, `ref` (label 要素へ), `xstyle/className/style` (label 要素へ)。

Kivact: `soft::field_label(label) -> FieldLabel`、`.target(NodeRef)` (= `inputID`。`basic::label(..).target(..)` と同じ形)、`.group_label(bool)`、`.label_hidden(..)`、`.optional(..)`、`.required(..)`、`.label_icon(..)`、`.label_tooltip(..)`、`.description_view(impl FnOnce() -> V)` (ReactNode description)、`.description(impl IntoText)`。CheckboxInput / Switch の節はこれを使う。

#### 共有内部 API

- `InputStatusType = 'warning'|'error'|'success'`、`InputStatus {type, message?}`、`InputSize = 'sm'|'md'|'lg'` (`types.ts`) → `StatusKind` / `InputStatus` / `kivact::ui::ControlSize::{Small, Medium, Large}`。
- `FieldStatusPlacement = keyof FieldStatusVariantMap | 'tooltip'` (`fieldStatusPlacement.ts:18`) → `StatusPlacement`。
- `useFieldInputStyling(component, rest, containerProps)` (`fieldParts.ts:122-174`): `hasFieldBox = !inGroup && direction != 'horizontal-labels'`。Kivact: soft input は `try_context::<soft::InputGroupCtx>()` と `try_context::<soft::FormLayoutCtx>()` を読んで Field shell を描くか決める。`container*` → soft input の `.container_style(Style)`、`control*` → `.control_style(Style)` (NodeBuilder modifiers は native の input ノードに付く。`kivact::ui::TextInput` と同じ)。
- `textEntryTraitAttributes({enterKeyHint, inputMode, autoCapitalize, hasAutoCorrect})` (`textEntryTraits.ts:96-116`): touch keyboard 用の native 属性。値: `EnterKeyHint = enter|done|go|next|previous|search|send`、`InputMode = none|text|tel|url|email|numeric|decimal|search`、`AutoCapitalize = off|none|on|sentences|words|characters`、`hasAutoCorrect → autocorrect on|off`。prop がなければ属性も出さない。Kivact desktop native には対応する概念がない → **全て削除** (または将来の IME/soft keyboard API 用に受け取って無視)。
- `InputClearButton {label, onClick, xstyle?, iconClassName?}` → kit 内部 `soft::input_clear_button(label, on_press)` (4.4.6)。
- `PanelSearchInput` (Selector / MultiSelector の popup 内検索行) → kit 内部 `soft::panel_search_input(label, clear_label, value: Binding<String>)` (4.4.7)。

### 4.4 Visual spec

#### 4.4.0 Token 解決表 (neutral theme。この節で使う全 token)

| token | light | dark | 出典 |
|---|---|---|---|
| `--color-background-surface` | `#ffffff` | `#262626` | neutralTheme.ts:90 (neutral.light[100] / dark[15]) |
| `--color-background-muted` | `#f1f1f1` | `#1b1b1b` | :94 |
| `--color-border-emphasized` | `#d4d4d4` | `#525252` | :165 |
| `--color-accent` | `#1b1b1b` | `#f1f1f1` | :96 |
| `--color-accent-muted` | `#f1f1f1` | `#262626` | :97 |
| `--color-text-primary` | `#000000` | `#ffffff` | :118 |
| `--color-text-secondary` | `#474747` | `#9e9e9e` | :119 |
| `--color-text-disabled` | `#919191` | `#525252` | :120 |
| `--color-icon-secondary` | `#6a6a6a` | `#9e9e9e` | :146 |
| `--color-border-warning` = `--color-icon-warning` = `--color-text-warning` | `#836700` | `#eec448` | tokens.stylex.ts:84-91 で text role の alias。text は neutralTheme.ts:135 |
| `--color-border-error` = `--color-icon-error` = `--color-text-error` | `#c0262c` | `#ffaea7` | :136 |
| `--color-border-success` = `--color-icon-success` = `--color-text-success` | `#2f7d33` | `#90ca90` | :134 |
| `--shadow-inset-warning` | `inset 0 0 0 2px #f8d36a4D` (rgba(248,211,106,.30)) | 同じ | :271 |
| `--shadow-inset-error` | `inset 0 0 0 2px #de47454D` (rgba(222,71,69,.30)) | 同じ | :272 |
| `--shadow-inset-success` | `inset 0 0 0 2px #2f7d334D` (rgba(47,125,51,.30)) | 同じ | :270 |
| `--color-overlay-hover` | `#0000000D` (5%) | `#ffffff0D` | :108 |
| `--color-overlay-pressed` | `#0000001A` (10%) | `#ffffff1A` | :112 |
| `--spacing-1` / `-1-5` / `-2` | 4px / 6px / 8px | 同じ | tokens.stylex.ts:189-191 |
| `--border-width` | 1px | 同じ | tokens.stylex.ts:223 |
| `--radius-element` (= `--_field-radius`) | 0.625rem = **10px** | 同じ | neutralTheme.ts:233 |
| `--radius-full` | 9999px | 同じ | |
| `--size-element-sm/md/lg` | 28 / 32 / 36px | 同じ (neutral は上書きしない) | tokens.stylex.ts:211-213 |
| `--font-family-body` | `Figtree, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif` | 同じ | neutralTheme.ts typography |
| `--text-body-size/leading` | 14px / 20px (1.4286) | 同じ | expandTypeScale (base 14, ratio 1.2) |
| `--text-label-size/leading` | 14px / 20px | 同じ | |
| `--text-supporting-size/leading` | 12px / 20px (1.6667) | 同じ | step −1 = round(14/1.2)=12 |
| `--font-weight-normal/medium` | 400 / 500 | 同じ | |
| `--duration-fast` | **125ms** | 同じ | neutral motion {fast:125, ratio:.75} |
| `--duration-fast-max` | **165ms** (125/0.75=166.7→5ms 丸め) | 同じ | expandMotionScale.ts |
| `--ease-standard` | `cubic-bezier(0.24, 1, 0.4, 1)` | 同じ | tokens.stylex.ts:325 (neutral は easing を上書きしない) |
| `--focus-outline-*` | 2px solid `--color-accent`, offset 3px | 同じ | tokens.stylex.ts:240-243 |

Touch + 狭幅 adaptation (`pointer: coarse` かつ幅 < lg): type scale が base 16 / ratio 1.1736 に変わり、body/label = 16px/24px、supporting = 14px/20px になる (neutralTheme.ts adaptations)。desktop native では通常 desktop 値でよい。kit は `ThemeMode` とは別の `soft::Density::Touch` を将来用に用意してよい (任意)。

#### 4.4.1 Field container

| 部位 | 値 | file:line |
|---|---|---|
| root | flex column, `isolation: isolate`, `min-width: 0` | Field.tsx:37-51 |
| root gap | label が見えるとき `--spacing-1` = 4px。`isLabelHidden` では gap なし (hidden label は display:contents/sr-only なのでレイアウトに出ない) | :52-54, :339 |
| width | `width` prop → root の `width` (number=px, string はそのまま)。指定がなくても min-width 0 なので flex row の中で field ごと縮む | :87-89, :340 |
| inputStatusWrapper | flex column, isolation isolate。`--_field-status-overlap` = 子の `data-size` に応じ `calc(--size-element-* / 2)` → sm 14px / md 16px (既定、custom control も) / lg 18px | :64-76 |
| attachedStatusLayer | FieldStatus に `position: relative; z-index: -1` | :77-82 |
| horizontal-labels label wrapper | `padding-top: calc(1px + 4px)` = 5px (input の上 border + 上 padding に揃える。textarea でも上揃え) | :58-63 |

#### 4.4.2 FieldLabel

| 部位 | 値 | file:line |
|---|---|---|
| label 要素 | `display:flex; align-items:center; gap:4px`、font `--font-family-body` 14px / line-height 20px / weight 500、color `--color-text-secondary` (light `#474747` / dark `#9e9e9e`)、cursor pointer | FieldLabel.tsx:63-76 |
| disabled | color `--color-text-disabled` (light `#919191` / dark `#525252`)、cursor default | :77-80 |
| labelIcon | Icon size `sm` = 16×16px、color inherit (= label 色)、label text の前、gap 4px | :292 |
| Optional/Required | `<span>` 内に aria-hidden の `" ∙ "` (前後スペース付き U+2219) + 文言。font 12px / 20px / weight 400、color `--color-text-secondary` | :96-101, :294-299 |
| label tooltip trigger | Icon `info` (Lucide `Info`) size sm 16px、color inherit、末尾。Tooltip placement `above`、`touchTrigger="tap"` | :300-313 |
| labelGroup (description あり) | flex column, gap 0 (label と description は 1 ブロック) | :52-55 |
| description | font 12px / 20px / weight 400、color `--color-text-secondary`。control に click を転送する場合 cursor pointer | :102-116 |
| hidden | label と description に sr-only (1×1px, clip, absolute, margin −1)。labelGroup は display:contents | :56-62, :81-95 |

indicator の表示ルール (`FieldLabel.tsx:225-241`): FormLayout の `defaultOptionality` と一致する指定は表示しない。

| defaultOptionality | isRequired | isOptional |
|---|---|---|
| `'optional'` | 「Required」表示 | 非表示 |
| `'required'` | 非表示 | 「Optional」表示 |
| 未設定 | 「Required」表示 | 「Optional」表示 |

両方 true は `Optional` が勝つ。

ASCII (light、md、description と attached error):

```
Email ∙ Required  ⓘ          ← 14/500 #474747、" ∙ Required" 12/400 #474747、ⓘ 16px
Enter your work email         ← 12/400 #474747
╭──────────────────────────╮  ← 32px、border 1px #c0262c、radius 10px、bg #ffffff
│ bad-email             ⊗  │  ← text 14/20 #000、status icon 20px #c0262c
├┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┤  ← (FieldStatus が control 下半分 16px の裏に潜る)
│ Please enter a valid email│  ← bg #ffc4be, text #76000c, 12/20, padding 8px
╰──────────────────────────╯  ← 下角だけ radius 10px
```

#### 4.4.3 Status message の配置

FieldStatus の見た目は 5 節。Field 側の規則:

- `attached` (既定): FieldStatus を control と同じ inputStatusWrapper に入れ、`margin-top: -overlap` で control の下半分の裏に潜らせる。control の z-index 1、status は z-index −1 で isolation 内に閉じる。control の丸い下角の隙間に status の色が見える。overlap が高さの半分なのは、CSS が描ける最大の角丸 (pill) でも隙間が出ないようにするため (Field.tsx:68-70)。
- `detached`: control の後ろの sibling。Field root の gap 4px + FieldStatus 自身の `margin-top` 4px = **control と status の間 8px**。
- `tooltip`: FieldStatus を描かない。control が on-field status icon の tooltip で message を出す (26.2)。それでも Field が message を announce する (4.6)。

#### 4.4.4 Field 共通の input container spec (shared input wrapper box)

**ここが他の input の節が参照する正本。** 出典は `inputStyles.stylex.ts` (`inputWrapperStyles`, `inputStatus*Styles`)、`inputDisabledFocus.ts`、そして全 single-line member が付ける size 別の高さ (例: `TextInput.tsx:90-100`)。

この box は「visual control box」: border と背景を描き、start icon / native input / spinner / clear button / status icon を横一列に持つ要素。native input 自身は枠なし・背景透明。

**(a) Rest (base) — `inputStyles.stylex.ts:33-73`**

| property | 値 (light / dark) |
|---|---|
| box-sizing | border-box |
| position / z-index | relative / 1 (attached FieldStatus より上) |
| layout | `display:flex; align-items:center` (横一列、縦中央) |
| gap | `--spacing-2` = 8px (icon、input、clear、status icon の間) |
| padding-block | `--spacing-1` = 4px |
| padding-inline | `--spacing-2` = 8px (左右。RTL でも対称) |
| border | `--border-width` 1px solid `--color-border-emphasized` (`#d4d4d4` / `#525252`) |
| radius | `--_field-radius` = `--radius-element` = 10px (4 隅) |
| background | `--color-background-surface` (`#ffffff` / `#262626`) |
| box-shadow | none |
| outline | none (focus は border + inset shadow で表す。outline ring は使わない) |
| transition | `border-color, box-shadow` 125ms `cubic-bezier(0.24,1,0.4,1)`。reduced motion では 0s |

**(b) 高さ (size 別)** — 共通 base は高さを持たない。single-line member は wrapper に `height: --size-element-*` を付ける (`TextInput.tsx:90-100`; NumberInput / DateInput / TimeInput / Selector / Typeahead も同じ token。差分は各節):

| size | height | 内容領域 (高さ − 2×border − 2×4px padding) |
|---|---|---|
| `sm` | 28px | 18px |
| `md` (既定) | 32px | 22px |
| `lg` | 36px | 26px |

padding / gap / font は size で変わらない (変わるのは高さだけ)。size は `SizeProvider` (InputGroup、FormLayout など) から継承でき、prop が優先 (`useSize(sizeProp,'md')`)。

**(c) 中身の既定 (TextInput の値。他 member も原則同じ)** — `TextInput.tsx:59-88`

| 部位 | 値 |
|---|---|
| native input | `flex:1; min-width:0`、border/padding 0、背景透明、outline none |
| text font | `--font-family-body`, 14px / line-height 20px、weight 指定なし (= 400)、color `--color-text-primary` (`#000000` / `#ffffff`) |
| placeholder | `--color-text-secondary` (`#474747` / `#9e9e9e`) |
| start icon | Icon size sm 16px、color `--color-icon-secondary` (`#6a6a6a` / `#9e9e9e`)、input の前 |
| busy spinner | Spinner sm、input の後、clear の前 |
| clear button | 4.4.6。値があり disabled/readOnly でないときだけ |
| status icon | 末尾。26.2 (md 20px、status 色) |
| 並び順 (inline-start → end) | `[startIcon] [input] [spinner] [clear] [statusIcon]` |

**(d) Hover — `inputStyles.stylex.ts:64-69`**

条件: pointer が box 上、box の中に focus が**ない**、disabled でない、かつ hover できる pointer (`@media (hover: hover)`)。

- box-shadow: `inset 0 0 0 2px color-mix(in srgb, --color-border-emphasized 30%, transparent)` → light `rgba(212,212,212,0.30)` / dark `rgba(82,82,82,0.30)`。border はそのまま。
- 結果として 1px border の内側に 2px の薄い帯が出て、枠が 3px に太ったように見える。

**(e) Focus (focus-within) — `:48-54`, `:70`**

条件: box の中のどこか (input、clear button、status button) に focus がある (`:focus-within`。keyboard でも pointer でも出る)。

- border-color: `--color-accent` (`#1b1b1b` / `#f1f1f1`)
- box-shadow: `inset 0 0 0 2px --color-accent-muted` (`#f1f1f1` / `#262626`)
- hover の inset shadow はこれで置き換わる。
- 注意: dark では `--color-accent-muted` (#262626) が背景 (#262626) と同じなので、inset 帯は**見えない**。dark の focus は「border が #f1f1f1 になる」だけに見える。これが Astryx の実際の見た目なので、そのまま再現する。
- forced colors: focus 時の border は `Highlight` (desktop native では対象外)。

**(f) Invalid / status (warning / error / success)** — `inputStatusBorderStyles` (:89-117)、`inputStatusHoverShadowStyles` (:123-151)、`inputStatusFocusWithinStyles` (:159-187)。TextInput は `base → size → disabled → disabledFocus.ring → statusBorder → statusHover(!disabled) → statusFocusWithin → inGroup` の順で重ねる (`TextInput.tsx:506-516`)。

| state | border | box-shadow |
|---|---|---|
| status rest | `--color-border-{kind}` (warning `#836700`/`#eec448`、error `#c0262c`/`#ffaea7`、success `#2f7d33`/`#90ca90`) | none |
| status hover (hover-capable, 非 focus, 非 disabled) | 同上 | `--shadow-inset-{kind}` = inset 2px (warning rgba(248,211,106,.30)、error rgba(222,71,69,.30)、success rgba(47,125,51,.30)。light/dark 共通) |
| status + focus-within | **status 色のまま** (accent にならない) | **none**。`inputStatusHoverShadowStyles` が `boxShadow` を丸ごと上書きするので、base の focus inset (accent-muted) が消える。status 付きで focus しても border の色は変わらず、inset 帯も出ない |

- `aria-invalid="true"` は error のときだけ native input に付く。warning / success は見た目だけ。
- Selector の trigger のように wrapper 自身が focus を受ける部品は `inputStatusFocusStyles` (`:focus`、:194-213) を使う。色は同じ。

**(g) Disabled** — `inputWrapperStyles.disabled` (:74-80) + `inputDisabledFocusStyles` (`inputDisabledFocus.ts`)

- box 全体 `opacity: 0.5`、cursor default、border `--color-border-emphasized`、box-shadow none (hover ring なし)。中身 (text、icon) も同じ opacity で一緒に薄くなる。
- disabled の input は**tab 順に残る** (spec:AST-085)。native `disabled` ではなく `aria-disabled="true"` + `readOnly` + change handler の guard。Enter で form submit しない (`TextInput.tsx:536-545`)。`name` 属性を外す (form に送らない)。
- disabled の field に keyboard focus があるとき (`:has(> [aria-disabled="true"]:focus-visible)`) は focus ring を戻す: border `--color-accent`、`inset 0 0 0 2px --color-accent-muted` (`inputDisabledFocus.ts:18-35`)。opacity 0.5 はそのまま。変種:
  - `ring`: input が wrapper の直下 (TextInput、NumberInput、TextArea)
  - `nestedRing` (:36-53): input が深い位置 (Typeahead、Tokenizer)。`:has(input[aria-disabled=true]:focus-visible)`
  - `withinRing` (:54-70): 中のどこかに focus (TimeInput、DateRangeInput、DateTimeInput)。`:focus-within`
  - `inputFocusRing` (:71-86): text input の `:focus` (DateInput)
- status 付きで disabled なら border は status 色 (statusBorder が後に適用される)、hover shadow は付かない。
- clear button は disabled では出ない。disabled message の tooltip (`disabledMessage`) は TextInput 節。

**(h) ReadOnly**

- container に**見た目の変化はない** (`data-readonly="readonly"` の theme target だけ。`TextInput.tsx:504`)。border、背景、hover ring、focus ring は rest と同じ。
- native input は `readOnly`。clear button は出ない。値は変わらない。

**(i) InputGroup の中** — `groupStyles.inGroup` (6.4)。radius は群の端だけ残り、高さ 100%、隣と 1px 重なり、focus-within で z-index 1。status icon と Field shell は描かない。

**(j) 状態の優先順 (Codex 向け要約)**

```
border-color = status? status色
             : disabled? (keyboard focus on control ? accent : emphasized)
             : focus_within? accent : emphasized
box-shadow   = disabled? (keyboard focus on control ? inset2 accent-muted : none)
             : status? (hover && !focus_within ? inset2 status-shadow : none)
             : focus_within? inset2 accent-muted
             : hover? inset2 emphasized@30% : none
opacity      = disabled? 0.5 : 1
```

**(k) Layout ルール**

- wrapper は親の幅いっぱい (Field root が flex column なので stretch)。Field root は `min-width: 0` なので row の中で縮む。
- input の text は 1 行、はみ出しは native input のスクロール (truncate の ellipsis はしない)。
- RTL: flex の並びが反転する (start icon が右)。padding は左右対称なので変化なし。

#### 4.4.5 Native control 以外を包む Field

Field で native `<input>` や custom control を包んだ場合、Field は box を描かない。control の見た目は caller 側。attached status は `--_field-status-overlap` の既定値 16px (md) で control の裏に潜る (Field.test.tsx:58)。

#### 4.4.6 InputClearButton (`InputClearButton.tsx`)

- 中身は `Button variant="ghost" size="sm" isIconOnly` + Icon `close` (Lucide `X`) size sm 16px color `--color-icon-secondary` (`#6a6a6a` / `#9e9e9e`) (:111-143)。
- box: `height: 20px` (Button sm の 28px を上書き)、`aspect-ratio: 1/1` → **20×20px**、padding 0、`flex-shrink: 0`、radius `--radius-element` 10px (20px 四方なので実質円)。背景透明、text 色 `--color-text-primary`。
- hover (hover-capable): background-image に `--color-overlay-hover` (`#0000000D` / `#ffffff0D`) を重ねる。pressed (mouse `:active`): `--color-overlay-pressed` (`#0000001A` / `#ffffff1A`) + `transform: scale(0.98)`。transition 宣言は `background-image, background-color, color, opacity, transform` 125ms `cubic-bezier(0.24,1,0.4,1)` (Button.tsx:118-125) だが、overlay は linear-gradient なので補間されず**即時**に切り替わる。補間されるのは scale だけ。touch の press (150ms の onset delay、200ms `cubic-bezier(0,0,0.2,1)` の release fade) は Button 節を参照。
- focus-visible: 共有の outline 2px solid `--color-accent` offset 3px (Button の focusOutline)。container の focus-within も同時に出る (button は container の中)。
- tap target: coarse pointer のときだけ `::after` overlay を inset −2px で出して 24×24 にする (:34-74)。fine pointer では overlay なし。
- pointerdown と mousedown で `preventDefault` → input の focus を奪わない (:133-137)。
- accessible name = caller の文脈 label (例 `Clear Email` = `@astryx.textInput.clearLabel` "Clear {label}")。見える tooltip は常に一般語 `"Clear"` (`@astryx.inputClearButton.tooltip`)。
- click 後の focus (TextInput 側 `TextInput.tsx:465-482`): `onChange('')` のあと input に focus を戻す (keyboard 起動 (`detail==0`) は通常 focus、pointer は `preventScroll`)。

#### 4.4.7 PanelSearchInput (`PanelSearchInput.tsx`; Selector / MultiSelector の popup 上端の検索行)

```
┌─ panel ──────────────────┐
│ ┌──────────────────────┐ │  ← wrapper padding 4px、field は option row と同じ box
│ │ ⌕  Search…         ✕ │ │
│ └──────────────────────┘ │
├──────────────────────────┤  ← divider は panel 側が持つ
```

| 部位 | 値 | file:line |
|---|---|---|
| wrapper | padding-block 4px、padding-inline 4px | :65-68 |
| field | flex row, align center, gap 8px, width 100%, padding-block `--spacing-1-5` 6px, padding-inline 8px, radius 10px、**border なし・背景なし**。md で高さ 6+20+6 = 32px (option row と同じ) | :72-87 |
| keyboard focus | `inset 0 0 0 2px --color-accent` (`#1b1b1b` / `#f1f1f1`)。**keyboard で focus したときだけ** (pointer focus では出ない。`useInteractionModalityTracking` で判定)。focus が clear button に移ったら消える | :98-103, :501-520 |
| transition | box-shadow 125ms `cubic-bezier(0.24,1,0.4,1)`、reduced motion 0s | :81-86 |
| search icon | Icon `search` sm 16px `--color-icon-secondary` | :554 |
| input | flex 1, min-width 0, 枠/背景なし、font 14px/20px、color `--color-text-primary`、placeholder `--color-text-secondary`、outline none | :110-137 |
| clear | 値が空でないとき InputClearButton (DOM 上 input の後。Tab 順のため)。押すと値 '' にして input に focus を戻す | :522-541, :573-575 |

native 属性: `enterKeyHint="search"`、`autocorrect="off"`、`aria-label=label`。

### 4.5 Motion

| 対象 | trigger | property | from → to | duration | easing | reduced motion | file:line |
|---|---|---|---|---|---|---|---|
| input container | hover / focus-within / status / disabled の切り替え | border-color, box-shadow (inset) | 状態間 | 125ms (`--duration-fast`) | `cubic-bezier(0.24, 1, 0.4, 1)` | 0s (即時) | inputStyles.stylex.ts:58-63 |
| PanelSearchInput field | keyboard focus ring の on/off | box-shadow | none ↔ inset 2px accent | 125ms | 同上 | 0s | PanelSearchInput.tsx:81-86 |
| InputClearButton | hover / press | background overlay (即時)、transform (補間) | overlay 0 → 5% / 10% (即時)、scale 1 → 0.98 | scale のみ 125ms | 同上 | 0s | Button.tsx:118-140 |
| FieldStatus (attached/detached) | mount (初回 paint 後に挿入されたときだけ) | opacity, translateY | 0, −8px → 1, 0 | 165ms (`--duration-fast-max`) | 同上、fill-mode backwards | animation なし | 5.5 参照 |
| label tooltip | hover/focus/tap | Tooltip 節参照 (show delay 200ms、hide の hover bridge 100ms。useTooltip.tsx:50, :302) | | | | | |

Field / FieldLabel 自体には animation がない。status の出入りで layout が変わるとき (status box の高さ) は即時 (高さ animation なし)。exit animation なし (FieldStatus は即 unmount)。

### 4.6 Behaviour & a11y

- **label の関連付け**: `<label htmlFor=inputID>`。label を押すと control が activate される (text input は focus、checkbox は toggle)。`isGroupLabel` のときは `<span id=labelID>` で `htmlFor` なし、group が `aria-labelledby` で参照する。
- **accessible name に indicator が入る**: Optional/Required の文言は label 要素の中にあるので、control の name は「Email Required」になる (` ∙ ` は aria-hidden)。`aria-required` は各 control が `useResolvedRequired` で別に付ける (form の defaultOptionality で indicator が隠れても `aria-required` は付く)。
- **description**: `aria-describedby` 用 id (`descriptionID` か `${inputID}-desc`)。description は label の**外の sibling** (label 内に入れると name に混ざり二重読みになるため)。description を click すると `useInputContainer` で control に転送する (text input は focus、checkbox/switch は click)。description 内の link / button の click は転送しない。text 選択中の click も転送しない。group label では転送しない (FieldLabel.tsx:249-288)。
- **label tooltip**: info icon は focus できない Tooltip trigger (Tooltip は element child に tabIndex を足さず、`aria-describedby` だけを付ける。`Tooltip.tsx:340-353, :388-400`)。そのため keyboard だけでは開けない (Astryx の挙動どおり再現する)。hover で開く。touch / pen の tap は icon で止まり (`useTapStop('click')`)、help を開閉するだけで control を activate しない。mouse click は label の通常動作 (control を focus / toggle) を保つ (Field.doc.mjs labelTooltip、FieldLabel.test.tsx:141-252)。
- **status message の id**: `messageID` か `${inputID}-status`。control の `aria-describedby` に入れるのは caller (Field-composed input は自動)。
- **announcement** (`useFieldStatusAnnouncement.ts`): message があるとき、mount 時も含めて、message または type が変わるたびに persistent live region で読み上げる。**error = assertive、warning / success = polite**。空 message は読まない。FieldStatus 要素自体には `role`/`aria-live` を付けない (同時に mount された live region は確実には読まれないため)。`statusVariant='tooltip'` では FieldStatus を描かないので Field 自身が同じ hook で announce する (Field.tsx:271-279)。同じ message は 1 回だけ、変わったときだけ再度読む (Field.test.tsx:416)。
- **disabled**: label が disabled 色になる。control 側は aria-disabled のまま focus 可能 (4.4.4 g)。
- **keyboard**: Field 自体は focus stop を持たない。Tab は control へ。
- **既定文言** (i18n key 提案):

| Astryx key | English | Kivact key |
|---|---|---|
| `@astryx.field.optional` | Optional | `soft.field.optional` |
| `@astryx.field.required` | Required | `soft.field.required` |
| `@astryx.inputClearButton.tooltip` | Clear | `soft.input.clear` |
| `@astryx.textInput.clearLabel` | Clear {label} | `soft.input.clear_label` (TextInput 節と共有) |

### 4.7 Kivact 実装方針

**方針: RESTYLE `kivact::ui::field` + kit wrapper `soft::field`。**

1. `kivact::ui::field` の restyle (`FieldRecipes` の `root/label/hint/error`、`FieldState { invalid }`)。標準の field は `label → control → hint → error` の順で、hint が control の**下**、error は `Role::Alert` の text だけ、status は error しかない。Astryx の構造 (description が control の上、warning/success、attached の overlap、indicator、icon、tooltip) は recipe だけでは再現できない。それでも標準 field を使う app のために近い look を入れる:
   - `root`: column, gap 4, `min_width(0)`, width 100%。
   - `label`: 14px / weight 500 / line-height 20px / `--color-text-secondary`。**invalid でも赤くしない** (Astryx の label は status で色が変わらない。basic の recipe は赤くするので真似しない)。
   - `hint`: 12px / 20px / `--color-text-secondary`。
   - `error`: detached error box の look (bg `--color-error-muted`、text `--color-on-error-muted`、padding 8、radius 10、margin_top 4)。
   参考: `crates/kivact-ui-basic/src/recipes_field.rs`。
2. kit 部品 `soft::field(label, control)` (Astryx と同じ構造)。`crates/kivact-ui/src/field.rs` の配線をそのまま真似る: `node_ref()` を label / description / status / control に作り、`control.name(label).labelled_by(label_ref).node_ref(control_ref)`、description があれば `.described_by(desc_ref)`、message があれば `.described_by(status_ref)`、error なら `.invalid(..)` (control の既存 invalid と OR。`accessible_invalid()` を読む)。label の `on_press(move || control_ref.activate())` が `<label htmlFor>` と useInputContainer の代わりになる (`NodeRef::activate` は text field を focus し、press handler のある control は activate する)。description にも同じ `on_press` を付ける (`group_label` のときは付けない)。
3. Layout: root `column` + `gap(4)` (label hidden なら 0)。label hidden は label と description に `.hidden(true)` を付けない (AT から消える)。代わりに **layout から外して name / description として残す**: label text は control の `.name(..)` に入っているので、見える label ノードは描かずに済む。description は control の `.description(text)` (NodeBuilder) に入れる。
4. attached status: control と status を `column` の wrapper にまとめる。status ノードに `margin_top(-overlap)`、`padding_top(overlap + 8)`、`z_index(0)`、control 側 wrapper に `z_index(1)`、status に `hit_testable(false)`。overlap は control の `ControlSize` から 14/16/18 (custom control は 16)。z_index が sibling 間で効かない場合は、status の上端を control の下端にそろえて (overlap 0)、status の上 2 角を角丸の半径 10px ぶん上にのばした `Style::paint` (自前描画) で代替する。
5. horizontal-labels: kit の `soft::form_layout` が `provide(FormLayoutCtx { direction, default_optionality })` を出す。Field は `try_context::<FormLayoutCtx>()` を読み、`HorizontalLabels` なら自分の root を作らず、2 列 grid (`display_grid`, `auto 1fr`) の行に label (padding_top 5) と column (description, control, status) を差し込む。kivact に `display: contents` はないので、FormLayout 側が各 field を grid row として受け取る API (`form_layout(rows)`) にする。
6. announcement: 標準 field の `Role::Alert` は mount 時に読まれないことがある。kit field は **常に mount されたままの、見えない live region ノード**を 1 つ持つ (`text(memo(msg)).live_region(memo(kind → Assertive|Polite))`、style `size(0,0)`、`hit_testable(false)`)。status の message をそこへ書き写す (persistent region)。見える FieldStatus ノードには `Role::Alert` / `live_region` を付けない。`Tooltip` placement でも同じ region を使う。message が変わらなければ書き直さない。
7. label tooltip: `kivact::ui::tooltip(text).placement(Placement::Above).attach(icon("info"))`。icon は press を止める (`on_press` を空にして label の activate へ伝えない) が、touch で tap したときだけ tooltip を開閉したい。kivact の tooltip が tap toggle を持つか UNRESOLVED (`crates/kivact-ui/src/tooltip.rs` を読んで確認。なければ `use_tooltip()` + `on_press` で `shown` を toggle)。
8. 必要な kivact パーツ: `kivact::control::{node, column, row, text, icon}`、`NodeExt::{labelled_by, described_by, live_region, accessible_disabled, accessible_invalid}`、`NodeBuilder::{name, description, invalid, disabled, hidden, hit_testable}`、`NodeRef::{activate, focus_within, focus_visible, hovered}`、`kivact::ui::tooltip`、`kivact::view::{provide, try_context, show_some}`。
9. 最も近い kivact-ui-basic: `crates/kivact-ui-basic/src/field.rs` (field_set など layout)、`recipes_field.rs` (recipe)、`primitives.rs` の `label(..).target(..)`、`crates/kivact-ui/src/field.rs` (配線の手本)。
10. 難所と fallback:
    - `aria-required` を表す semantics が kivact にない (`NodeExt` に required がない)。Astryx でも indicator 文言が name に入るので、name を `"{label} Required"` / `"{label} Optional"` にする (indicator が隠れている場合の required は UNRESOLVED: description に "Required" を足すか、kivact に semantics 追加を提案)。
    - aria-disabled のまま focus 可能な disabled field: kivact の `.disabled(true)` は activation と key を止める。tab 順に残るかは UNRESOLVED。TextInput 節で `read_only(true)` + 見た目だけ disabled + `.disabled` を付けない、という方式を検討する。
    - `color-mix(... 30%, transparent)` は `Color::with_alpha(0.3)` で同じ。
    - `@media (hover: hover)` は desktop では常に真。touch 入力の hover は kivact が出さない前提。
    - forced colors は対象外。

### 4.8 受け入れ確認

- [ ] `field("Email", text_input(..))` の control の accessible name が "Email"。label を押すと input が focus される。
- [ ] `.required(true)` で label に " ∙ Required" (12px、#474747) が出て、name が "Email Required"。`.optional(true)` と併用すると Optional だけ。
- [ ] FormLayout の `default_optionality = Required` の中では `.required(true)` の indicator が出ない。
- [ ] `.description(..)` が label の下・control の上に 12px で出て、control の description になる。押すと control が focus される。
- [ ] `.label_hidden(true)`: label と description が見えず、gap も消え、name / description は残る。
- [ ] `.label_tooltip(..)`: label 末尾に 16px info icon。hover で上に tooltip。
- [ ] `.disabled(true)`: label 色が #919191 (light) / #525252 (dark)。
- [ ] Attached error: control の border #c0262c、下に #ffc4be の box が control の下半分の裏から出る (control の丸い下角に色が見える)。message が assertive で 1 回読まれる。warning / success は polite。
- [ ] Detached: control と status の間 8px、status は 4 隅とも 10px 角丸で先頭 icon 付き。
- [ ] Tooltip placement: message box は出ず、それでも message が読み上げられる。
- [ ] 共通 container (TextInput で確認): md 32px、border 1px #d4d4d4、radius 10px、bg #fff。hover で内側 2px の薄い帯。focus で border #1b1b1b + 内側 2px #f1f1f1。dark では border #525252 → focus #f1f1f1、bg #262626。
- [ ] status 付きで focus しても border が status 色のまま、内側の帯がない。status 付きの hover は status 色の 30% 帯。
- [ ] disabled container は全体 50% opacity、hover しても変化なし。readOnly は rest と同じ見た目で clear が出ない。
- [ ] 状態変化の色が 125ms で補間される (reduced motion では即時)。
- [ ] docsite の Field ページ (light / dark) の StatusVariants と並べて目視比較する。

---

## 5. FieldStatus

Source: `packages/core/src/FieldStatus/` (`FieldStatus.tsx`, `useFieldStatusAnnouncement.ts`, `index.ts`)、entry animation は `packages/core/src/hooks/useEntryAnimation.ts`。Docs: https://astryx.atmeta.com/components/FieldStatus (overview では非表示、Field group)。Story: `FieldStatus.stories.tsx` (Variants, AuditMatrix)。

### 5.1 概要

- form field の validation feedback (error / warning / success) の message box。Field が内部で使う。Switch、CheckboxInput、custom control も直接使う。
- `attached`: 枠付き input の直下で、その input の message であるとき。
- `detached`: checkbox、switch、custom control など、重ねると不自然な control。
- 使わない: 一般的な alert やページ単位の通知 (Banner / Toast を使う)。
- `variant='tooltip'` は deprecated (FieldStatus 自身には tooltip 表示がない。field control の `statusVariant="tooltip"` を使う。次の minor で削除)。**Kivact では最初から持たない。**

### 5.2 Anatomy

```
div#id.astryx-field-status[data-type][data-variant]     ← Message box (必須)
├─ attached: message (text のみ)
└─ detached: span.detachedContent (flex row, gap 4px, align flex-start)
     ├─ span.detachedIcon (inline-flex, height = 1 行分 20px, 縦中央)
     │    └─ Icon (warning|error|success, sm, inherit, aria-hidden) .astryx-field-status-icon[data-type]
     └─ span  message                                    ← Message text (必須)
```

role なし、aria-live なし、aria-hidden なし (text は普通に読める。announcement は別経路)。

### 5.3 API

| prop | type | default | 意味 | Kivact |
|---|---|---|---|---|
| `type` | `'error'\|'warning'\|'success'` | 必須 | 色と icon | 第 1 引数 `StatusKind` |
| `message` | `string` | 必須 | 本文 | 第 2 引数 `impl IntoText` |
| `variant` | `'attached'\|'detached'` (`tooltip` は deprecated) | `'attached'` | 置き方 | `.variant(FieldStatusVariant::{Attached, Detached})` |
| `id` | `string` | – | `aria-describedby` 用 | 削除。`.node_ref(r)` を caller が作り `control.described_by(r)` |
| `ref`, `xstyle`, `className`, `style`, その他 DOM props | | | root へ | NodeBuilder modifiers。`className` は削除 |

```rust
soft::field_status(kind: StatusKind, message: impl IntoText) -> FieldStatus   // NodeBuilder + IntoView
    .variant(FieldStatusVariant::Detached)
    .overlap(f32)          // attached のときの潜り込み量 (既定 16 = md)。Field が size から渡す
    .announce(bool)        // 既定 true: 4.7(6) の persistent live region に書く。Field 内では Field が担当するので false
```

`FieldStatusVariantMap` の module augmentation (theme が variant を足す仕組み) は Kivact では `.variant(impl kivact::ui::Variant)` の拡張点で代替する (任意)。

### 5.4 Visual spec

**共通 (`FieldStatus.tsx:53-72`)**

| property | 値 |
|---|---|
| font | `--font-family-body`, `--text-supporting-size` 12px / line-height 20px、weight 400 (指定なし) |
| wrap | `overflow-wrap: anywhere` (URL など切れ目のない文字列も field 幅で折り返す) |
| 幅 | 親の幅いっぱい (block)。高さは text 行数で伸びる |

**色 (`colorStyles`, :106-119)**

| kind | background (light / dark) | text・icon (light / dark) |
|---|---|---|
| warning | `--color-warning-muted` `#fae19e` / `#e2b6233D` (rgba(226,182,35,.24)。#1b1b1b 上で ≈ `#4b401d`) | `--color-on-warning-muted` `#4b3900` / `#eec448` |
| error | `--color-error-muted` `#ffc4be` / `#ff98903D` (rgba(255,152,144,.24)。#1b1b1b 上で ≈ `#523937`) | `--color-on-error-muted` `#76000c` / `#ffaea7` |
| success | `--color-success-muted` `#bce0bb` / `#90ca903D` (rgba(144,202,144,.24)。#1b1b1b 上で ≈ `#374537`) | `--color-on-success-muted` `#00490b` / `#90ca90` |

(neutralTheme.ts:139-141, :154-159。dark の背景は半透明なので、下にある色に重ねて描く。合成色は body 背景 #1b1b1b 上の概算。)

**attached (`:73-83`)** — overlap は `var(--_field-status-overlap, --spacing-1-5)`。Field の中では sm 14 / md 16 / lg 18px、Field の外では 6px。

| property | 値 (md) |
|---|---|
| margin-top | `-overlap` = −16px |
| padding-top | `overlap + 8px` = 24px |
| padding-bottom | 8px |
| padding-inline | 8px |
| radius | 下 2 隅 (`end-start`, `end-end`) だけ `--radius-element` 10px。上 2 隅 0 (control の裏に隠れる) |
| pointer | `pointer-events: none` (重なった部分の click を control に通す) |
| 重なり | Field が z-index −1 を付け、control (z-index 1) の裏に描く |
| icon | なし (on-field の status icon が代わり) |

見える高さ (1 行): control の下端から 8 + 20 + 8 = 36px。

**detached (`:84-103`)**

| property | 値 |
|---|---|
| margin-top | 4px (Field の gap 4px と合わせて control から 8px) |
| padding | 8px 全周 |
| radius | 10px 全周 |
| content | flex row, align flex-start, gap 4px |
| icon | warning=`AlertTriangle`、error=`XCircle`、success=`CheckCircle` (Lucide)、size sm 16px、color inherit (= text 色)、aria-hidden。box 高さ 20px (= 12px × 1.6667) で 1 行目の中央にそろう |
| 複数行 | icon は 1 行目にそろったまま、text が折り返す |

ASCII (detached warning, light):

```
╭────────────────────────────────────────╮  bg #fae19e, radius 10px
│ ⚠ This will be visible to others       │  icon 16px #4b3900, text 12/20 #4b3900
╰────────────────────────────────────────╯  padding 8px, gap 4px
```

state: hover / focus / disabled はない (静的な box)。RTL: detached の icon は inline-start (右) に移る。

### 5.5 Motion

| trigger | property | from → to | duration | easing | fill | reduced motion | file:line |
|---|---|---|---|---|---|---|---|
| mount (初回 page paint の**後**に挿入されたときだけ。SSR hydrate 時やページ読み込みと同時の mount では再生しない) | opacity, transform | opacity 0, `translateY(-8px)` (`-1 × --spacing-2`) → opacity 1, `translateY(0)` | 165ms (`--duration-fast-max`) | `cubic-bezier(0.24, 1, 0.4, 1)` | backwards | animation なし | useEntryAnimation.ts:32-38, :59-67、FieldStatus.tsx:215 |
| unmount / message 変更 | なし (即時) | | | | | | |

注: attached でも translateY するので、出現中の 165ms は box が control の裏から少し上 (−8px) から降りてくる。control が z-index 上なので、上端は隠れたまま。

### 5.6 Behaviour & a11y

- announcement は 4.6 と同じ (`useFieldStatusAnnouncement`: error assertive、他は polite。mount 時も読み、message / type が変わると再度読む。空なら読まない)。
- 要素には role / aria-live を付けない。aria-hidden も付けない。
- detached の icon は aria-hidden (status の意味は text が持つ。色だけに頼らない WCAG 1.4.1 のために見た目の icon がある)。
- focus を持たない。pointer: attached は pointer を通す。
- 既定文言なし (message は caller)。

### 5.7 Kivact 実装方針

- **NEW kit component** `soft::field_status`。最も近い analog: `kivact::ui::field` の error part (`crates/kivact-ui/src/field.rs`、`FieldRecipes::error`)、見た目は `crates/kivact-ui-basic/src/alert.rs` (tone 別の色付き box + 先頭 icon)。
- 描画: `node()` + `row((icon_box, text(message)))`。style: `padding(8)`、`radius(10)` (attached は `radius_corners(0, 0, 10, 10)`)、`background(muted)`、`color(on_muted)`、font 12/20、text は折り返し (`single_line` を付けない)。attached は `margin_top(-overlap)`、`padding_top(overlap + 8)`、`hit_testable(false)`、`z_index` は Field が管理 (4.7(4))。
- 色: kit の `Tokens` に `warning_muted / on_warning_muted / error_muted / on_error_muted / success_muted / on_success_muted` を light/dark で持つ (dark は alpha 付き `Color`)。
- icon: theme の icon set に Lucide `triangle-alert` (AlertTriangle)、`circle-x` (XCircle)、`circle-check` (CheckCircle) を入れ、`icon(name)` を 16px、`color` は親から継承 (kivact の icon が親の text color を使えない場合は `on_muted` を明示)。
- motion: `.enter(style().opacity(0.0).translate_y(-8.0))` + `.transition(TransitionProp::Paint, Transition::tween(165, Easing::CubicBezier(0.24, 1.0, 0.4, 1.0)))` (`decorative` 既定 true → reduced motion で即時)。window を開いた時点で既に status がある場合は再生しない: kit が「最初の frame が描けた」flag (例: root の `on_mount` 後に立てる signal) を持ち、flag が立つ前に mount された FieldStatus には `.enter(..)` を付けない。kivact の `enter` が初回 mount で再生されるかは UNRESOLVED (`kivact-style/src/sheet.rs:869` の doc を確認)。
- announcement: `.announce(true)` のとき 4.7(6) の persistent live region に書く。Field の中では Field 側が持つので重複させない。
- 難所: 半透明の dark 背景は kivact の `Color::with_alpha` でそのまま描ける。attached の「control の裏に潜る」は 4.7(4) の z_index か paint の fallback。

### 5.8 受け入れ確認

- [ ] `field_status(Error, "This field is required")` (attached): bg #ffc4be、text #76000c、上角 0・下角 10px、icon なし。
- [ ] Detached: 先頭に 16px の status icon、padding 8、4 隅 10px。2 行に折り返しても icon は 1 行目の高さにそろう。
- [ ] 長い URL が field の幅で折り返す。
- [ ] 後から挿入すると 165ms で上 8px から fade-in。reduced motion では即時。window 起動時から表示されているものは animation しない。
- [ ] error は assertive、warning / success は polite で、mount 時に 1 回読まれる。要素自身に alert role がない。
- [ ] dark: warning bg は半透明の黄、text #eec448。
- [ ] docsite FieldStatus の Variants / AuditMatrix と light / dark で比較する。

---

## 6. InputGroup

Source: `packages/core/src/InputGroup/` (`InputGroup.tsx`, `InputGroupText.tsx`, `groupStyles.ts`, `InputGroupContext.ts`, `InputGroupInternalContext.ts`)。Docs: https://astryx.atmeta.com/components/InputGroup (overview では非表示、Field group)。Story: `InputGroup.stories.tsx`。

### 6.1 概要

- 1 つの input と prefix / suffix の addon (text、icon、button) を 1 つの見た目の単位につなぐ。URL (`https://`)、通貨 (`$`)、単位 (`kg`)、action button 付きの検索など。
- 使う (`InputGroup.doc.mjs` bestPractices): 単位や形式を示す text addon、静的な prefix/suffix は `InputGroupText`、input に付く action は `Button` (角は自動でつながる)。対応する single-line input: TextInput、NumberInput、TimeInput、DateInput、Typeahead、Selector、MultiSelector。内側の input の label は具体的に (group label と内側の label が結合される)。
- 使わない: 1 つの group に複数の text input、無関係な input、TextArea / Slider / Switch / CheckboxInput / RadioList。

### 6.2 Anatomy

```
Field (label=group label, isGroupLabel, statusVariant="detached")
├─ FieldLabel: <span id=labelID>  label [∙ Optional|Required] [ⓘ]   + description (descriptionID)
├─ div[role=group].astryx-input-group[data-size][data-status]
│     aria-labelledby=labelID  aria-describedby="descriptionID statusMessageId"  aria-disabled?
│   ├─ InputGroupText  (prefix addon)        [data-astryx-input-group-item]
│   ├─ <input member> (TextInput 等。Field shell なし、自分の label は VisuallyHidden span) [data-astryx-input-group-item]
│   └─ Button / InputGroupText (suffix addon) [data-astryx-input-group-item]
└─ FieldStatus variant=detached (status.message があるとき)
```

Context:
- `InputGroupContext {isInGroup: true, labelID, describedByIDs}` (公開 hook `useInputGroup()`): member input はこれを見て Field shell と status icon を描かず、`groupStyles.inGroup` を付け、ARIA を結合する。
- `InputGroupInternalContext {isDisabled}` (非公開): disabled の継承。入れ子の group でも外側が disabled なら disabled (内側の `isDisabled={false}` で解除できない)。
- `SizeProvider value={size}`: member の既定 size。

### 6.3 API

| prop | type | default | 意味 | Kivact |
|---|---|---|---|---|
| `children` | `ReactNode` | 必須 | InputGroupText、Button、対応 input | 第 2 引数 `impl soft::GroupMembers` (tuple / Vec。`basic::GroupItems` と同じ形) |
| `label` | `string` | 必須 | group の label (見える) | 第 1 引数 |
| `isLabelHidden` | `boolean` | `false` | label を視覚的に隠す | `.label_hidden(..)` |
| `description` | `string` | – | label と group の間の helper text | `.description(..)` |
| `isDisabled` | `boolean` | `false` | group 全体を disabled (member にも継承) | NodeBuilder `.disabled(..)` + context で member に伝える |
| `isOptional` / `isRequired` | `boolean` | `false` | indicator | `.optional(..)` / `.required(..)` |
| `size` | `'sm'\|'md'\|'lg'` | `'md'` (外側の SizeProvider があればそれ) | group の高さと member の既定 size | `.size(ControlSize)` |
| `status` | `{type, message?}` | – | 「group の border に適用」と doc にあるが、実装は detached FieldStatus と `data-status` のみ (6.4 注) | `.status(impl IntoValue<Option<InputStatus>>)` |
| `labelTooltip` | `string` | – | label 末尾の tooltip | `.label_tooltip(..)` |
| `data-testid` | `string` | – | group div へ | `.test_id(..)` |
| `ref`, `xstyle`, `className`, `style`, 他 DOM props | | | **group div へ** (Field root ではない) | NodeBuilder modifiers は group の行ノードへ |

```rust
soft::input_group(label: impl IntoText, members: impl GroupMembers) -> InputGroup
    .size(ControlSize::Small).description("..").status(status).label_hidden(true)
soft::input_group_text(content: impl IntoView) -> InputGroupText   // NodeBuilder + IntoView
```

### 6.4 Visual spec

**group 行 (`InputGroup.tsx:35-57`)**

| property | 値 |
|---|---|
| layout | `display: inline-flex; align-items: stretch` (member は高さいっぱい)。Field root (flex column) の子なので実際は横幅いっぱいに伸びる |
| height | `--size-element-{size}`: sm 28 / md 32 / lg 36px |
| background | transparent (枠と背景は各 member が描く) |
| border / radius | なし (member が描く) |
| disabled | `opacity: 0.5`、cursor default |

**member (対応 input の wrapper) — `groupStyles.inGroup` (`groupStyles.ts:75-101`)**

| property | 値 |
|---|---|
| flex | `flex: 1; min-width: 0` (input が残り幅を取る) |
| height | 100% (= group 高さ) |
| margin-inline-start | 先頭以外 `-1px` (隣の border と重ねて 1 本の線にする) |
| radius | start 側 2 隅は**先頭 child のときだけ** 10px、end 側 2 隅は**最後の member のときだけ** 10px。他は 0 |
| focus-within | `z-index: 1` (focus 中の border を隣より上に) |
| その他 | 4.4.4 の共通 container (border 1px #d4d4d4/#525252、bg surface、hover / focus / status / disabled)。**status icon は描かない** (26.2)。Field shell (label、description、status box) も描かない |

「最後」の判定は `:nth-last-child(1 of :not([popover]):not(template):not(dialog))` (`groupStyles.ts:34-36`): popover / template / dialog 要素は数えない。Kivact では overlay は木の外に出るので、単純に「最後の member」でよい。

**Button addon — `groupStyles.buttonAddon` (`groupStyles.ts:47-73`)** (group の**直下**の Button だけ。input 内部の clear / calendar button は対象外)

| property | 値 |
|---|---|
| radius | 先頭でなければ start 2 隅 0、後ろに member があれば end 2 隅 0 |
| height | 100% |
| focus-visible | `z-index: 2` (focus ring を隣より上に) |
| look | Button のまま。既定 variant `secondary` = bg `--color-neutral` (`#0000000F` / `#ffffff1A`)、text `--color-text-primary`、**border なし**、padding-inline 12px、label 14/500。primary は bg `--color-accent` (`#1b1b1b`/`#f1f1f1`) text `--color-on-accent` (`#ffffff`/`#111111`)。詳細は Button 節 |
| margin | 重ねない (Button に border がないので -1px なし) |

**InputGroupText — 6.x 参照。**

**disabled の合成**: group の opacity 0.5 と、継承で disabled になった member 自身の opacity 0.5 (input container、Button) が重なり、member は実効 **0.25**、InputGroupText は 0.5 になる (CSS opacity は入れ子で掛け算)。これが Astryx の実際の描画なので、そのまま再現する (意図かどうかは UNRESOLVED。docsite の Disabled story で目視確認)。

**status (注意)**: doc は「group の border に適用」と書くが、`InputGroup.tsx` / `groupStyles.ts` には status で border を変える style がない。InputGroupContext に status は含まれず、member input も group の status を読まない。実装上の結果:
- group の下に **detached** FieldStatus (control から 8px、先頭 icon 付き) が出る。
- `data-status` が付く (theme target 用)。neutral theme には該当する component override がない。
- member の border は member 自身の `status` prop がなければ neutral のまま。
→ Kivact は**実装どおり** (message box だけ) にする。docsite の WithErrorStatus story で border が赤くならないことを確認する (UNRESOLVED: 目視未確認)。

**ASCII (md、light、prefix text + input + secondary button)**

```
Price ∙ Required
╭────┬───────────────────────┬────────╮   高さ 32px
│ $  │ 0.00                  │  Save  │   $: bg #f1f1f1 text #474747 14/20 padding-inline 8
╰────┴───────────────────────┴────────╯   input: 共通 container、角 0 (左右とも内側)
 ↑ 先頭 child: 左 2 隅 10px                ↑ Save: bg #0000000F、border なし、右 2 隅 10px
╭──────────────────────────────────────╮
│ ⊗ Price is required                  │   detached FieldStatus (group から 8px 下)
╰──────────────────────────────────────╯
```

**RTL**: logical property なので、先頭 (start) の角丸は右側、-1px margin は右側に付く。

### 6.5 Motion

group 自体の animation はない。member の container は 4.5 の 125ms transition (border / inset shadow)。focus-within の z-index 変化は即時。Button addon は Button 節 (overlay 125ms、press scale 0.98)。status box の出現は 5.5 (165ms slideDown)。

### 6.6 Behaviour & a11y

- group: `role="group"`、`aria-labelledby` = group label の span (`isGroupLabel` なので `<label>` ではない。label を押しても何も起きない。description の click 転送もない)、`aria-describedby` = description id + status message id (あるものだけ)、`aria-disabled="true"` (disabled 時)。
- **member の name**: `aria-labelledby = "groupLabelID inputLabelID"` (例: group "Price" + input "Amount" → "Price Amount")。member の自分の label は VisuallyHidden の span で描く (`TextInput.tsx:526`)。member の `aria-describedby` に group の describedByIDs が先頭で入る (`utils/inputAria.ts:47-60`)。Selector / MultiSelector / Typeahead / NumberInput / DateInput も同じ (InputGroup.test.tsx:80-263)。
- **disabled の継承**: `isDisabled` が member input と group 直下の Button に伝わる (`aria-disabled="true"`)。入れ子の group も外側に従う。group が enable に戻ると、継承で disabled だった member は戻るが、自分で `isDisabled` を指定した member は disabled のまま (InputGroup.test.tsx:281-397)。
- keyboard: group は focus stop ではない。Tab は member を DOM 順に回る (prefix button → input → suffix button)。focus 中の member が z-index で前に出る。
- pointer: InputGroupText を押しても input に focus は移らない (Astryx の InputGroupText に click 転送はない)。
- 既定文言なし。

### 6.7 Kivact 実装方針

- **NEW kit component** `soft::input_group`。`basic::input_group` は「外側 1 つの枠 + 枠なしの内側 control」(shadcn) なので構造が違う。Astryx は **member ごとに枠を持ち、1px 重ねて角を端だけ残す** (ButtonGroup 型)。最も近い analog: `crates/kivact-ui-basic/src/button_group.rs` (`GroupItems::into_styled_views(|index| ..)` で index ごとに style を足す) と `input_group.rs` (group の `Role::Group`、`current_direction()` による RTL 反転)。
- 構造:
  - root = `soft::field` 相当の column (label は group label: `labelled_by` を group 行に張る。label を押しても activate しない)、group 行、detached status。
  - group 行 = `row(..)`、`role(Role::Group)`、`.labelled_by(label_ref)`、description / status があれば `.described_by(..)`、高さ `control_height(size)`、`align_items(Align::Stretch)`。disabled なら `opacity(0.5)`。
  - `provide(soft::InputGroupCtx { label: Value<String>, size, disabled: Value<bool>, described_by: Vec<NodeRef> }, ..)`。soft の対応 input (`soft::text_input` 等) は `try_context::<InputGroupCtx>()` があれば: Field shell を描かない、status icon を描かない、name を `"{group label} {own label}"` にする (`labelled_by(group_label_ref)` + 自分の hidden label を順に張れるならそれ、無理なら `.name(format!(..))`)、group の description / status を `described_by` に足す、`disabled = own || group` (caller の明示 disabled は残す)。
  - edge: members を index で包み `provide(GroupEdge { first: i == 0, last: i == n - 1 }, ..)`。member は container の `radius_corners` を edge から決める (first: start 2 隅 10、last: end 2 隅 10、他 0)、`margin_left(-1)` (first 以外、RTL は `margin_right`)、`grow(1).min_width(0)`、高さ 100% (`align_items(Stretch)` に任せる)。focus-within 中は `z_index(1)`。Button addon (`kivact::ui::Button`) は edge 側の角を 0、高さ stretch、`focus_visible` 中 `z_index(2)`。button 判定は basic と同じく `Any` で型を見る (`InputGroupButton`/`Button`)。
- 色・container は 4.4.4 の共通 recipe (26.1 の `soft::input_container`) をそのまま使う。
- 難所: disabled の opacity 合成 (0.5 × 0.5) は kivact でも入れ子 opacity なら自然に掛け算になる (要確認。ならなければ member 側で 0.25 を明示)。z_index による focus 中 border の前面化が sibling 間で効くか UNRESOLVED (効かなければ focus 中の member の border を 1px 外側に描く `draw_over` で代替)。

### 6.8 受け入れ確認

- [ ] `input_group("Price", (input_group_text("$"), text_input("Amount", v)))`: role group の name が "Price"、textbox の name が "Price Amount"。
- [ ] 高さ md 32px、sm 28、lg 36。`$` の box は bg #f1f1f1 (dark #1b1b1b)、左角だけ 10px。input は左角 0、右角 10px。境目の線が 1 本 (2px にならない)。
- [ ] input に focus すると border #1b1b1b が隣の addon の線より前に出る。
- [ ] suffix の secondary Button は border なし、右角 10px、左角 0、高さが group と同じ。
- [ ] description があれば group の description、status message があれば group の下 8px に detached box (icon 付き) が出て group と textbox の description に入る。border は赤くならない。
- [ ] `.disabled(true)`: group 0.5 opacity、input と button も disabled (aria-disabled 相当)、input / button は実効 0.25。
- [ ] RTL で prefix が右、角丸と重なりが鏡像。
- [ ] docsite InputGroup (WithPrefix / WithActionButton / Disabled / WithErrorStatus / SmallSize) と light / dark で比較する。

### 6.x InputGroupText

Source: `InputGroupText.tsx` (`:32-68` styles)。Docs は InputGroup のページ内。

**6.x.1 概要**: InputGroup の中の静的な prefix / suffix (`$`、`kg`、`https://`、icon)。text でも icon でもよい。

**6.x.2 Anatomy**: `div.astryx-input-group-text[data-astryx-input-group-item]` > children。role なし (ただの text)。

**6.x.3 API**: `children: ReactNode` (必須)、`ref`、`xstyle`、`className`、`style`、DOM props。Kivact: `soft::input_group_text(content: impl IntoView)` (文字列なら `text(..)`、icon なら `icon("search")`)。NodeBuilder modifiers 可。

**6.x.4 Visual spec**

| property | 値 (light / dark) |
|---|---|
| layout | flex, align-items center、高さは group の stretch (28/32/36) |
| padding-inline | 8px。padding-block 0 |
| background | `--color-background-muted` (`#f1f1f1` / `#1b1b1b`) |
| text | `--font-family-body` 14px / 20px (`--text-body-*`)、weight 400、color `--color-text-secondary` (`#474747` / `#9e9e9e`)、`white-space: nowrap`、`flex-shrink: 0` |
| border | 1px solid `--color-border-emphasized` (`#d4d4d4` / `#525252`) |
| margin-inline-start | 先頭 child 以外 −1px |
| radius | 先頭 child なら start 2 隅 10px、最後の child なら end 2 隅 10px、他 0 (`:last-child` で判定。input member の判定と違い popover 等を除外しない) |
| icon | caller が指定 (story では `Icon search sm secondary` = 16px `#6a6a6a`/`#9e9e9e`) |
| state | hover / focus / disabled の専用 style なし。group disabled の 0.5 opacity だけかかる |

**6.x.5 Motion**: なし。

**6.x.6 Behaviour & a11y**: 静的 text。押しても何もしない。member の name には入らない (Astryx は group label + input label だけで name を作る。`$` は読まれない)。

**6.x.7 Kivact 実装方針**: `row(content)` + style (padding_x 8、background muted、border 1 emphasized、font 14/20、color text_secondary、shrink 0、`single_line`)。edge は InputGroup の `GroupEdge` context から角と margin を決める。closest: `basic::input_group_text` (`input_group.rs`) と `basic::button_group_text` (`button_group.rs`)。

**6.x.8 受け入れ確認**: [ ] bg #f1f1f1 / #1b1b1b、border 1px、text #474747 14px。[ ] 先頭なら左角 10px・右角 0。[ ] 最後なら右角 10px。[ ] 長い text でも折り返さず縮まない。


---

## 7. FileInput

ソース: `packages/core/src/FileInput/FileInput.tsx`（1026 行）、`FileInput.doc.mjs`、`FileInput.test.tsx`（1664 行）、`__tests__/FileInputAudit.a11y.chromium.spec.ts`、`FileInputTooltipPress.a11y.chromium.spec.ts`。共有: `Field/InputClearButton.tsx`、`hooks/useInputStatusIcon.tsx`、`hooks/useClickableContainer.ts`、`hooks/useAnnounce.ts`、`hooks/useBusyAnnouncement.ts`、`i18n/useListFormatter.ts`。**`Field/inputStyles.stylex.ts` は使わない**（独自のボックススタイルを持つ）。Storybook: `apps/storybook/stories/FileInput.stories.tsx`（Default / WithDescription / MultipleFiles / ImagesOnly / DropzoneMode / Required / Optional / Disabled / DisabledWithMessage / Loading / WithErrorStatus / WithSuccessStatus / WithTooltip / AllVariations / StatusVariantComparison / AuditMatrix）。Docs: https://astryx.atmeta.com/components/FileInput

### 7.1 概要

ファイル選択フィールド。`mode='input'`（既定）はテキスト入力風のコンパクトな 1 行、`mode='dropzone'` は破線枠の大きな領域でドラッグ＆ドロップも受け付ける。accept / maxSize / maxFiles による検証とエラー表示、選択の読み上げ、クリア、`changeAction` による即時アップロード（楽観表示＋スピナー）を内蔵。

- 使う: 常に `accept` を指定。`maxSize` / `maxFiles` で大きすぎるアップロードを防ぐ（検証とエラー表示は自動）。制約（形式・サイズ）は description で伝える。即時アップロードには `changeAction`。スペースが許すなら dropzone を使う（D&D が期待される操作）。
- 避ける: フォルダ/ディレクトリのアップロード（非対応）。disabled 理由を Tooltip で包まず `disabledMessage` を使う。

### 7.2 Anatomy

```
Field (label の htmlFor = ネイティブ input の id)           ← Field セクション (4.x)
├─ FieldLabel / description
├─ control  div.astryx-file-input  data-mode="input|dropzone" data-status?   (非インタラクティブな器。クリックでピッカー)
│   ├─ VisuallyHidden > <button type=button tabIndex=0 aria-label=…>          ← 唯一のフォーカス対象 (trigger)
│   ├─ <input type=file aria-hidden tabIndex=-1>  1×1px clip (視覚的に隠す)
│   ├─ content（mode × 状態で変わる, 下表）
│   │    upload Icon (lucide Upload) / placeholder | file names / Spinner / status affordance
│   └─ InputClearButton   (hasFiles && !disabled && !busy)
├─ VisuallyHidden <span id=requiredID>"Required"</span>   (required 解決時)
├─ disabledMessage tooltip layer
└─ FieldStatus (attached / detached メッセージ; 検証エラーもここ)
```

content の中身（`FileInput.tsx:822-887`）:

| mode | 状態 | 子（順） |
|---|---|---|
| input | idle・未選択 | upload icon sm → placeholder（"Choose file" / "Choose files"）→ status icon |
| input | idle・選択済み | upload icon sm → ファイル名（`, ` 連結）→ status icon → Clear |
| input | busy | ファイル名 or placeholder（**primary 色のファイル名スタイル**）→ Spinner sm → tooltip 配置の status ボタンのみ（Clear なし、upload icon なし） |
| dropzone | idle・未選択 | upload icon md → placeholder（drag-over 中は "Drop files here"）→ tooltip 配置の status ボタンのみ |
| dropzone | idle・選択済み | ファイル名（中央・折り返し）→ tooltip status → Clear（下に中央配置） |
| dropzone | busy | Spinner md → tooltip status |

```
input mode (h=32):
┌──────────────────────────────────────────┐
│ ⇪  Choose file                         ⚠ │   gap 8, pad 4/8
└──────────────────────────────────────────┘
┌──────────────────────────────────────────┐
│ ⇪  resume_final_v3.pdf, cover…      ⚠  ✕ │
└──────────────────────────────────────────┘

dropzone mode (破線, pad 24/16, gap 8):
┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐
            ⇪   (20px)
│          Choose files                    │
└ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘   高さ 98px
```

### 7.3 API

| Astryx prop | 型 / 既定 | 意味 | Kivact 案 |
|---|---|---|---|
| `label` | string（必須） | ラベル。trigger の name | `soft::file_input(label, value)` 第 1 引数 |
| `value` | `File \| File[] \| null`（必須, 制御） | 選択ファイル | 第 2 引数 `impl Into<Binding<Vec<PathBuf>>>`（単一モードは長さ ≤1。空 = null） |
| `onChange` | `(File \| File[] \| null) => void`（必須） | 選択・削除・ドロップ・検証後に通知 | binding の write |
| `changeAction` | `(files) => Promise<void>` | 即時アップロード。pending 中は楽観値＋busy | `.change_action(Fn(Vec<PathBuf>) -> impl Future)`（`kivact::task::action`） |
| `accept` | string（HTML accept 形式, 例 `"image/*"`, `".pdf,.doc"`） | 受け入れ形式 | `.accept(impl IntoText)`（同じ書式を解析） |
| `isMultiple` | bool / false | 複数選択 | `.multiple(bool)` |
| `maxSize` | number（bytes） | 超過ファイルを拒否 | `.max_size(u64)` |
| `maxFiles` | number | 複数時の上限（超過分は切り捨て＋エラー） | `.max_files(usize)` |
| `mode` | `'input' \| 'dropzone'` / `'input'` | 見た目と D&D | `.mode(FileInputMode::Input / Dropzone)` |
| `isDisabled` | bool / false | 0.5 減光、ピッカー不可、trigger はフォーカス可、フォーム除外 | NodeBuilder `.disabled(..)` |
| `disabledMessage` | string | 無効理由ツールチップ（hover / focus） | `.disabled_message(text)` |
| `isRequired` / `isOptional` | bool / false | 表示＋ trigger の describedby に "Required" | `.required(bool)` / `.optional(bool)` |
| `isLoading` | bool / false | busy（スピナー・aria-busy・選択不可） | `.loading(impl IntoValue<bool>)` |
| `status` | `{type, message?}` | 検証状態（指定時は内部検証エラーより優先） | `.status(impl IntoValue<Option<InputStatus>>)` |
| `statusVariant` | `'attached'\|'detached'\|'tooltip'` / `'attached'` | dropzone は attached 非対応（次 minor で detached に解決） | `.status_placement(..)`。**dropzone では Attached を Detached に解決し既定も Detached**（互換不要なので先取り） |
| `description` | string | 説明 | `.description(text)` |
| `placeholder` | string / "Choose file" or "Choose files" | 未選択時テキスト | `.placeholder(text)` |
| `isLabelHidden`, `width`, `labelTooltip` | | TextInput と同じ | 同上 |
| `container*` / `control*` | | Field ルート / ボックス | `.container_style(..)` / `.control_style(..)` |
| `ref`, `id`, `aria-*`, `data-testid`, `onClick` 等 | ref/id/data はネイティブ input に、`aria-*` は trigger にコピー | | `.node_ref(..)` / `.test_id(..)` は trigger に |
| ※ `size` / `isReadOnly` なし | — | — | 追加しない |

### 7.4 Visual spec

#### input mode（コンパクト）— `styles.compact`（`FileInput.tsx:196-232`）

Field 共通 box（4.4）と似ているが**別定義**。差分を太字で示す。

| 項目 | トークン | light | dark |
|---|---|---|---|
| 高さ | `--size-element-md`（固定、size prop なし） | 32px | 32px |
| padding / gap | block `--spacing-1` / inline `--spacing-2` / gap `--spacing-2` | 4px / 8px / 8px | 同左 |
| border | 1px solid `--color-border-emphasized` | `#d4d4d4` | `#525252` |
| radius | `--radius-element`（直接参照） | 10px | 10px |
| 背景 | `--color-background-surface` | `#ffffff` | `#262626` |
| hover リング（非 focus-within, hover:hover） | **`inset 0 0 0 2px color-mix(--color-accent 20%, transparent)`** | **`#1b1b1b33`** | **`#f1f1f133`** |
| focus-within | 枠 `--color-accent` のみ。**inset フォーカスリングなし** | `#1b1b1b` | `#f1f1f1` |
| cursor | pointer（disabled / busy は default） | | |
| transition | border-color, box-shadow `--duration-fast` `--ease-standard` | 125ms cubic-bezier(0.24,1,0.4,1) | |

#### dropzone mode — `styles.dropzone`（`FileInput.tsx:143-173`）

| 項目 | 値 |
|---|---|
| レイアウト | flex **column**, align-items center, justify center, gap 8px |
| padding | block `--spacing-6` **24px**, inline `--spacing-4` **16px** |
| border | 1px **dashed** `--color-border-emphasized` `#d4d4d4` / `#525252`。focus-within で `--color-accent` `#1b1b1b` / `#f1f1f1` |
| radius / 背景 | 10px / `#ffffff` / `#262626` |
| hover（`dropzoneHover`, 選択可能時のみ, `:178`） | inset 2px `#1b1b1b33` / `#f1f1f133`。**`:not(:focus-within)` 条件なし**（フォーカス中も hover で出る） |
| drag-over（`dropzoneActive`, `:182-185`） | 枠 `--color-accent` `#1b1b1b` / `#f1f1f1`、背景 `--color-accent-muted` `#f1f1f1` / `#262626`（dark は surface と同色なので枠だけが変わって見える）。placeholder が "Drop files here" に |
| transition | border-color, box-shadow, **background-color** 125ms cubic-bezier(0.24,1,0.4,1) |
| 高さ | 内容次第: idle 1+24+20+8+20+24+1 = **98px**、選択済み＋Clear 98px（Clear 無しなら 70px）、busy 70px |

#### 文字とアイコン

| 部品 | 値（light / dark） | 出典 |
|---|---|---|
| placeholder | Figtree 14px / 20px / 400、`--color-text-secondary` `#474747` / `#9e9e9e`、text-align center（input mode は start, flex 1, min-width 0）、user-select none | `:258-273, :299-303` |
| ファイル名 | Figtree 14/20、`--color-text-primary` `#000000` / `#ffffff`、ellipsis・nowrap・flex 1・min-width 0。dropzone は center ＋ `white-space: normal`（折り返す）、input は start | `:274-298` |
| upload アイコン | neutral lucide `Upload`、input = sm 16px / dropzone = md 20px、`--color-icon-secondary` `#6a6a6a` / `#9e9e9e`（テーマターゲット `astryx-file-input-icon`） | `:843-848, :871-876` |
| Spinner | dropzone md = 直径 14 + stroke 3 → 20px、input sm = 14px。弧 `--color-accent`、トラック `--color-track` `#CCD3DB` / `#5A5E66` | `:826, :864` |
| クリア | TextInput と同じ InputClearButton（20×20, ghost, close 16px secondary, tooltip "Clear"） | `:1008-1013` |
| ステータスアイコン | input mode: attached（＝メッセージなし tooltip も）は Icon md 20px `--color-icon-<status>`、tooltip 配置は `<button>`。dropzone: tooltip 配置のボタンだけ（attached / detached ではボックス内にアイコンを出さない） | `:820, :884` |
| ネイティブ input | absolute 1×1px, margin −1, overflow hidden, clip rect(0,0,0,0) | `:247-257` |

#### 状態

| 状態 | input mode | dropzone |
|---|---|---|
| rest | 上表 | 上表 |
| hover | inset 2px accent 20%（非フォーカス時のみ） | inset 2px accent 20%（フォーカス中も） |
| focus（trigger にキーボードフォーカス） | 枠 accent。リングなし。trigger は視覚的に隠れているのでボックスの枠だけがフォーカス表示 | 破線枠が accent |
| drag-over | （D&D なし） | 枠 accent ＋背景 accent-muted、文言 "Drop files here" |
| 選択済み | ファイル名 primary、Clear 表示 | 中央にファイル名（折り返し）、下に Clear |
| busy（isLoading / changeAction pending） | cursor default、hover リングなし（`styles.busy` boxShadow none）、Spinner、Clear なし | 同左（Spinner md のみ） |
| disabled | opacity 0.5、cursor default、枠 emphasized（focus-within で accent）。**hover リングは抑止されない**（コンテナ自身は aria-disabled でないため `:where(:not(:disabled,[aria-disabled]))` が真のまま。Astryx の実挙動として要 docsite 確認、再現推奨） | opacity 0.5、cursor default、hover リングなし（`dropzoneHover` を付けない）、枠 emphasized / focus-within accent |
| status error / warning / success | 枠 `--color-border-<status>`（error `#c0262c` / `#ffaea7`、warning `#836700` / `#eec448`、success `#2f7d33` / `#90ca90`）。**focus 中も status 色**（`statusBorderStyles` は条件なしの borderColor で focus-within 分岐ごと置換）。hover は accent 20% のまま（status 用 hover シャドウなし）。アイコン（input mode, attached/tooltip） | 枠 status 色（破線のまま）。drag-over 中も status 色が勝つ（`statusBorderStyles` が後） |
| 検証エラー（内部） | status 未指定時、`{type:'error', message: <エラー列挙>}` として上と同じ表示＋ FieldStatus メッセージ | 同左 |

レイアウト: ボックス幅 = Field 幅。input mode のテキストは 1 行 ellipsis。RTL: 論理プロパティで反転（upload icon が右、Clear が左）。dropzone は中央揃えなので反転の影響はアイコン・テキスト内のみ。

### 7.5 Motion

| 対象 | トリガ | プロパティ | 時間 / easing | reduced | 出典 |
|---|---|---|---|---|---|
| input box | hover / focus / status | border-color, box-shadow | 125ms cubic-bezier(0.24, 1, 0.4, 1), delay 0 | 0s | `FileInput.tsx:213-218` |
| dropzone | hover / focus / drag-over / status | border-color, box-shadow, background-color | 125ms cubic-bezier(0.24, 1, 0.4, 1) | 0s | `FileInput.tsx:162-167` |
| placeholder ↔ "Drop files here"、内容の切替（選択・busy） | 状態変化 | テキスト・子の差し替え | 即時（アニメーションなし。dropzone の高さ 98↔70px も即時） | — | — |
| Spinner md（dropzone） | busy | dashoffset 0 → −(14π) = −43.98px | 525ms linear infinite | 3s/周 | `Spinner.tsx:228-231, 338-350` |
| Spinner sm（input） | busy | dashoffset 0 → −31.416px | 525ms linear infinite | 3s/周 | 同上 |
| Clear ボタン | hover / press | overlay 即時、scale 0.98 125ms | | 0s | `Button.tsx:84-136` |
| ツールチップ | — | Tooltip セクション参照 | | | |

### 7.6 Behaviour & a11y

起動（ピッカーを開く）:
- trigger（隠しボタン）に Tab でフォーカス → Enter / Space で `preventDefault` してネイティブ input を `click()`（busy / disabled 時は開かない, `:731-741`）。
- ボックスの非インタラクティブ部分（アイコン・テキスト・padding）クリック → ピッカー（`useClickableContainer`。Clear / status ボタンのクリックは除外。テキスト選択中は無視）。disabled / busy 時はハンドラ自体を外す（`:933-934`）。クリックでは trigger にフォーカスは移らない。
- Field ラベルクリック → ネイティブ input の既定動作でピッカー。disabled / busy 時は input の onClick で `preventDefault`（`:997-999`）。

選択結果 `handleFiles`（`:614-679`）:
1. disabled / busy なら無視（busy 中に遅れて返ったピッカー結果も無視）。
2. `accept` 検証（カンマ区切り, trim, 小文字）: `.ext` はファイル名の末尾一致（大小無視）、`type/*` は MIME の前方一致、その他は MIME 完全一致。不一致ごとに `"{fileName}" is not an accepted file type`。
3. `maxSize`: `size > maxSize` を除外し `"{fileName}" exceeds {maxSize} limit`（maxSize は B / KB / MB 整形）。
4. `isMultiple && maxFiles`: 有効数 > maxFiles なら先頭 maxFiles 件に切り詰め `Maximum {maxFiles} files allowed`。
5. エラーがあれば全メッセージをロケールのリスト形式（Intl.ListFormat, en: "A, B, and C"）で 1 つの error status に。無ければ内部エラーをクリア。
6. 有効 0 件 → `onChange(null)`。それ以外 → `onChange(multiple ? valid : valid[0])`。
7. エラー 0 件の時だけ polite アナウンス: 1 件 `1 file selected: {fileName}`、複数 `{count} files selected`。このピックで始まる busy は「Loading」を重ねてアナウンスしない（`isPickAnnounced`）。エラーは FieldStatus が assertive に 1 回だけ読む。
8. `changeAction` があれば transition 内で楽観値をセットして await（pending 中 busy）。
9. ネイティブ input の value を毎回 `''` に戻す（同じファイルを再選択できる）。

ドラッグ＆ドロップ（dropzone のみ, `:753-804`）:
- dragenter / dragover: preventDefault + stopPropagation。選択可能なら drag-over 状態に。
- dragleave: `relatedTarget` がボックス内（子要素への移動）なら無視、外へ出た時だけ解除（ちらつき防止）。
- drop: 状態解除。選択可能かつ dropzone ならファイルを `handleFiles`（accept もここで検証）。
- busy / disabled になった瞬間に drag-over 状態を破棄し、以後のドラッグも無視。
- input mode ではハンドラを付けない（D&D 不可）。

クリア（`:692-720`）: イベント伝播停止（ピッカーを開かない）、内部エラー解除、`onChange(null)`、changeAction があれば `changeAction(null)`（楽観的に空表示＋busy）。フォーカスは trigger へ（キーボード起動は即時、ポインタは次フレームで `preventScroll`）。

disabled: trigger は `aria-disabled` でタブ順に残る。ネイティブ input は `form=""` でフォーム送信・required 検証から外れる。disabledMessage はボックスに anchored（hover と focusin で開く, placement above）。

a11y（trigger ボタンに集約。ネイティブ input は aria-hidden・tabIndex −1）:
- role button、name = 未選択時 `label`、選択時 `"{label}, {fileNames}"`（aria-label は子テキストを抑止するのでファイル名を合成）。
- `aria-describedby` = caller → description → ステータスメッセージ（tooltip 以外）→ status tooltip id → requiredID（"Required"）→ disabledMessage tooltip。
- `aria-invalid` = status error（内部検証エラー含む）、`aria-busy`、`aria-disabled` = disabled || busy。caller の `aria-*`（label / labelledby / hidden / disabled / busy / invalid を除く）を trigger にコピー。
- 必須は `aria-required` ではなく隠し "Required" テキストを describedby で伝える（`FormLayout defaultOptionality="required"` でも同様）。
- status tooltip ボタンのクリックではピッカーを開かない（"does not open the file picker from the dropzone status button"）。

既定テキスト:
| Astryx key | 英語 | soft キー案 |
|---|---|---|
| `@astryx.fileInput.placeholder` | `Choose file` | `soft.file_input.placeholder` |
| `@astryx.fileInput.placeholderMultiple` | `Choose files` | `soft.file_input.placeholder_multiple` |
| `@astryx.fileInput.dropHint` | `Drop files here` | `soft.file_input.drop_hint` |
| `@astryx.fileInput.clearLabel` | `Clear {label}` | `soft.file_input.clear` |
| `@astryx.fileInput.triggerWithFiles` | `{label}, {fileNames}` | `soft.file_input.trigger_with_files` |
| `@astryx.fileInput.fileSelected` | `1 file selected: {fileName}` | `soft.file_input.file_selected` |
| `@astryx.fileInput.filesSelected` | `{count} files selected` | `soft.file_input.files_selected` |
| `@astryx.fileInput.errorInvalidType` | `"{fileName}" is not an accepted file type` | `soft.file_input.error_invalid_type` |
| `@astryx.fileInput.errorMaxSize` | `"{fileName}" exceeds {maxSize} limit` | `soft.file_input.error_max_size` |
| `@astryx.fileInput.errorMaxFiles` | `Maximum {maxFiles} files allowed` | `soft.file_input.error_max_files` |
| `@astryx.fileInput.fileSize.bytes` | `{size} B`（グループなし整数） | `soft.file_input.size_bytes` |
| `@astryx.fileInput.fileSize.kilobytes` | `{size} KB`（小数 1 桁固定, グループなし, size = bytes/1024） | `soft.file_input.size_kb` |
| `@astryx.fileInput.fileSize.megabytes` | `{size} MB`（小数 1 桁, size = bytes/1024²） | `soft.file_input.size_mb` |
| `@astryx.fileInput.required` | `Required` | `soft.file_input.required` |
| リスト結合 | Intl.ListFormat（en: `A, B, and C`） | `soft.list.and_two` = `{a} and {b}`, `soft.list.and_many` = `{items}, and {last}` |
| Clear tooltip / status button / Loading | TextInput と共通 | `soft.input.clear_tooltip` 等 |

### 7.7 Kivact 実装方針

戦略: **NEW kit component `soft::file_input`**（`kivact::ui` に対応する標準部品なし）。kivact-ui-basic にもファイル選択部品はない。最も近い手本は `crates/kivact-ui-basic/src/input_group.rs`（ボックス＋内側コントロール＋addon、`focus_within` によるフォーカス look）と `attachment.rs`（ファイル名チップの文字処理）。

構成:
- ボックス: 非フォーカスの row（input）/ column（dropzone）ノード。スタイルは上表を reactive closure で（`trigger_ref.focus_within()` / drag-over signal / busy / disabled / status）。破線は `border_style(BorderStyle::Dashed)`、hover リングは `hover(style().shadow(Shadow::new(0,0,0,accent@20%).spread(2).inset()))`、transition は `transition(TransitionProp::Paint, Transition::tween(125, Easing::CubicBezier(0.24, 1.0, 0.4, 1.0)))`（Paint は border 色・背景色・先頭 2 シャドウを補間するので要件を満たす）。
- trigger: Astryx の「隠しボタン＋非インタラクティブな器」を、ボックス内の content 部分（icon＋テキスト）を覆う focusable ノード（`role(Role::Button)`, `.name(..)`, `.described_by(..)`, `.invalid(..)`, `.on_press(open_picker)`）で表現し、Clear / status ボタンは兄弟に置く（ボタンの入れ子を作らない）。trigger 自身は focus look を持たず（`focus_visible(style().ring_none())`）、ボックスが `focus_within` で枠 accent。ボックス余白のクリックは `on_press(move || trigger_ref.activate())`（basic の `inline_addon` と同じ）。
- ピッカー: `kivact::task::spawn(async move { kivact::platform::pick_file(FileDialog{ title, filters, .. }).await })`（複数は `pick_files`）。`accept` を `FileDialog::filters`（拡張子リスト）に変換。**難所: ネイティブには MIME 型がない** → `.ext` はそのまま、`image/*` 等は kit 内の拡張子表（image: png jpg jpeg gif webp bmp svg avif heic ico tif tiff / video: mp4 mov webm mkv avi / audio: mp3 wav ogg flac m4a aac / text: txt md csv json xml html / application/pdf: pdf ほか主要なもの）で近似し、ドロップ時も同じ表で検証。
- サイズ: `std::fs::metadata(path).len()`（読めない時は 0 扱い or 拒否、方針を README に明記）。
- D&D: dropzone のボックスに `.on_file(|e: FileEvent| ..)`: `FilePhase::Enter | Over` → drag-over true、`Leave` → false、`Drop` → false ＋ `handle_files(e.files)`。子への移動で Leave が来るかはエンジン依存なので、Leave 後すぐの Over で復帰するなら十分（ちらつきは 1 フレーム以内）。要実機確認。
- 値: `Binding<Vec<PathBuf>>`。表示名は `file_name()`、複数は `", "` 連結。
- 検証・リスト結合・サイズ整形・アナウンスは純関数として kit 内に実装（`message_with` でテンプレート展開。KB/MB は `format!("{:.1}", v)`）。アナウンスは `live_region(LiveRegion::Polite)` の不可視ノード、エラーは Field の status（Alert）経由で 1 回。
- busy: `isLoading || change_action pending`。busy 中は trigger `disabled` semantics（ただしフォーカス可能のまま、TextInput 同様要確認）、ピッカー・ドロップ無視。
- Spinner / Clear / status アイコン / Field / Tooltip は他セクションの kit 部品を共用。
- 落とすもの: forced-colors、iOS フォント下限、`form=""`（フォームなし）、ref のネイティブ input 転送。
- 難所まとめ: MIME 判定の近似、OS ドラッグの Leave/Over 粒度、ネイティブピッカー中のフォーカス復帰（ダイアログ終了後に trigger へ `NodeRef::focus()` で戻すと Astryx の「フォーカスは trigger に残る」と一致）。

### 7.8 受け入れ確認

- [ ] role button の trigger が 1 つだけタブ停止し、name = ラベル（選択後は "Resume, cv.pdf"）。Enter / Space でファイルダイアログ（テストでは `Screen::respond_file_dialog`）。
- [ ] input mode: 高さ 32px、枠 1px `#d4d4d4` / `#525252`、radius 10px、upload 16px＋ "Choose file"（secondary 色）。hover で内側 2px `#1b1b1b33` / `#f1f1f133`、Tab フォーカスで枠 `#1b1b1b` / `#f1f1f1`（内側リングなし）。
- [ ] dropzone: 破線枠、高さ 98px、upload 20px＋"Choose files" 中央。ファイルドラッグ中は枠 accent・背景 `#f1f1f1`（dark 不変）・"Drop files here"、ドロップで on_change、input mode ではドロップ無視。
- [ ] accept=".pdf" に .png を渡すと error status `"x.png" is not an accepted file type`、aria-invalid、FieldStatus に表示、選択アナウンスなし。maxSize 超過・maxFiles 超過も同様（複数エラーは 1 メッセージに列挙）。
- [ ] 成功時 "1 file selected: cv.pdf" / "3 files selected" が polite に 1 回。
- [ ] Clear（name "Clear Resume"、ツールチップ "Clear"）で空になりフォーカスが trigger に戻る。disabled / busy では Clear 非表示。
- [ ] loading: input はスピナー 14px、dropzone はスピナー 20px のみ、ピッカーもドロップも無効、aria-busy。
- [ ] disabled: 0.5 減光、Tab でフォーカス可、ピッカー開かない、disabledMessage がツールチップで出る。required: describedby に "Required"。
- [ ] docsite（light/dark）の Default / DropzoneMode / MultipleFiles / Disabled / Loading / WithErrorStatus / StatusVariantComparison と目視比較。


---

## 8. CheckboxInput

ソース: `packages/core/src/CheckboxInput/CheckboxInput.tsx`（行番号は以下 `CheckboxInput.tsx:N`）、見た目は indicator に委譲 `packages/core/src/Indicator/CheckboxIndicator.tsx`、ラベルは `packages/core/src/Field/FieldLabel.tsx`、押下 overlay は `packages/core/src/utils/interactionOverlay.stylex.ts` / `utils/pressFeedback.ts` / `utils/pressGesture.ts`、focus ring は `hooks/useIndicatorFocusRing.tsx` + `utils/focusOutline.stylex.ts`。Docs: https://astryx.atmeta.com/components/CheckboxInput 、Story: `apps/storybook/stories/CheckboxInput.stories.tsx`, `CheckboxA11y.stories.tsx`。

### 8.1 概要

単一の on/off（+ indeterminate）値を切り替えるチェックボックス。設定（"Enable notifications"）、規約同意、opt-in に使う。複数の選択肢をまとめるときは CheckboxList（§9）。

- DO: 常に可視ラベルを付ける（`isLabelHidden` は周囲の文脈で自明なときだけ）。説明が要る選択肢には `description` を付ける。"select all" で一部だけ選択されているときは indeterminate を使う。
- DON'T: 排他選択に使わない（→ RadioList）。即時反映のアクションに使わない（→ Switch / Button）。disabled のチェックボックスを Tooltip で包まない（disabled はホバーイベントを出さない）→ `disabledMessage` を使う。
- a11y 要件（doc.mjs `usage.accessibility`）: 未チェックの枠線・チェック時の塗りは背景に対して 3:1 以上（WCAG 1.4.11）。Hover / Pointer down では tint と pressed overlay 適用後の最終色で測る。

### 8.2 Anatomy

```
div.astryx-checkbox-input (root, width 指定はここ)          [themeProps('checkbox-input', {size})]
└ div (row: flex, align-items:center, gap 8px)  ← press 面・indicatorScope・hover 判定はこの row 全体
  ├ div (checkboxWrapper: 20×20 / 24×24, position:relative, isolation:isolate)
  │   ::after = 押下 overlay（radius inner）
  │ ├ input[type=checkbox] (opacity:0, absolute, 中央配置, z-index 1, 実体の focus/role/state)
  │ └ span (display:contents, focus ring の描画先を一意にする)
  │    └ span.astryx-checkbox-indicator (aria-hidden, 箱)
  │       ├ svg(viewBox 0 0 10 10) チェックマーク path "M8.5 2.5L4 7.5L1.5 5"
  │       ├ span 横棒（indeterminate）
  │       └ (busy 時) Spinner sm（上 2 つを置き換え）
  └ div (labelWrapper: column)
     └ FieldLabel (label.astryx-field-label.astryx-checkbox-label for=input)
        ├ [labelIcon 16px] label text [ " ∙ Optional/Required"]
        └ span description (id=descriptionID, クリックで input に転送)
FieldStatus variant="detached"（status.message があるとき、row の下）
Tooltip（disabledMessage 用, placement above, row にホバー/フォーカスで表示, 位置は input に anchor）
```

役割: `input` が `role=checkbox`（native）。indicator は装飾（`aria-hidden="true"`）。

### 8.3 API

| Astryx prop | 型 / default | 意味 | Kivact |
|---|---|---|---|
| `label` | `string`（必須） | 可視ラベル兼 accessible name | `soft::checkbox_input(label, value)` の第 1 引数（`kivact::ui::checkbox(label, ..)` に渡る） |
| `value` | `boolean \| 'indeterminate'`（必須） | 状態 | 呼び出し側所有 `Binding<bool>` + `.indeterminate(impl IntoValue<bool>)`（`kivact::ui::Checkbox::indeterminate` が既存）。あるいは kit 側 enum `CheckValue { Unchecked, Checked, Indeterminate }` を `Binding<CheckValue>` で受ける薄いラッパー |
| `onChange` | `(checked, e) => void` | 変更通知 | Binding への書き込みで足りる。追加通知は `.on_change(impl Fn(bool))` |
| `changeAction` | `(checked, e) => void \| Promise` | 非同期アクション。pending 中は楽観的に新値を表示し busy（spinner） | `.change_action(impl Fn(bool) -> Task)`（kivact::task）。pending 中は表示値 = 新値、`busy=true`。完了まで入力無効 |
| `isLoading` | `boolean` = false | spinner 表示＋操作禁止 | `.loading(impl IntoValue<bool>)` |
| `isDisabled` | `boolean` = false | 無効 | NodeBuilder `.disabled(..)` |
| `disabledMessage` | `string` | disabled 時の理由 tooltip。フォーカス可能のまま（aria-disabled） | `.disabled_message(impl IntoText)`：disabled でも focusable を維持し、hover/focus で tooltip |
| `isReadOnly` | `boolean` = false | 見た目は通常（減光なし）、操作のみ禁止。`aria-readonly` | `.read_only(impl IntoValue<bool>)` |
| `isRequired` / `isOptional` | `boolean` = false | ラベル末尾に " ∙ Required" / " ∙ Optional" 表示、`aria-required` | `.required(..)` / `.optional(..)`（FormLayout 既定は §4 Field 参照） |
| `isLabelHidden` | `boolean` = false | ラベルを sr-only に | `.hide_label(..)`（既存 `Checkbox::hide_label`） |
| `description` | `string` | ラベル下の補足 | `.description(impl IntoText)` |
| `labelIcon` | `ReactNode \| IconType` | ラベル前のアイコン | `.label_icon(icon(name))` |
| `status` | `{type:'error'\|'warning'\|'success', message}` | detached status box、error で `aria-invalid` | `.status(impl IntoValue<Option<FieldStatus>>)`（§5 FieldStatus の型を共用） |
| `size` | `'sm' \| 'md'` = `'md'` | 箱 20 / 24px | `.size(ControlSize::Small \| Medium)`（Small→sm, Medium→md。ExtraSmall は Small、Large は Medium に丸める） |
| `width` | `SizeValue` | フィールド全体の幅 | `.style(style().width(..))` |
| `htmlName` | `string` | フォーム送信名 | **不要**（native フォームなし）→ drop |
| `onFocus` / `onBlur` | handler | | NodeBuilder の focus ハンドラ（kivact 側既存のもの）で代替 |
| `ref`, `id`, `xstyle`, `className`, `style`, `aria-describedby` | | | `xstyle/className/style` は drop（`.style(..)` へ）。`aria-describedby` は `.described_by(..)` 相当があればそれ、なければ drop |

### 8.4 Visual spec

寸法（`CheckboxInput.tsx:167-176`, `CheckboxIndicator.tsx:126-157`）:

| size | 箱 | チェックマーク svg | indeterminate 棒 | row gap | coarse pointer 時の hit 最小 |
|---|---|---|---|---|---|
| sm | 20×20 | 12×12（viewBox 10 → stroke 1.5 は 1.8px 相当） | 10×2, radius full | 8px（`--spacing-2`）| input 24×24（`CheckboxInput.tsx:145-150`）|
| md | 24×24 | 14×14（stroke 1.5×1.4 = 2.1px 相当） | 12×2, radius full | 8px | 24×24 |

- 箱: `box-sizing:border-box`, border `--border-width` = 1px solid, radius `--radius-inner` = **6px**（neutral）。箱内は flex center。
- チェックマーク: path `M8.5 2.5 L4 7.5 L1.5 5`（viewBox 0..10）、stroke 1.5（viewBox 単位）、round cap/join、fill none。色 `--color-on-accent`（light #ffffff / dark #111111）。
- 棒: 背景 `--color-on-accent`、radius full。
- ラベル（`FieldLabel.tsx:63-76`）: font `--font-family-body`（Figtree）、`--text-label-size` 14px / `--text-label-leading` 1.4286 (=20px) / weight `--font-weight-medium` 500、色 `--color-text-secondary`（light #474747 / dark #9e9e9e）。ラベル内 gap 4px（labelIcon 16px と text の間）。Optional/Required サフィックスは 12px/20px w400 text-secondary、`" ∙ "` は aria-hidden。
- description（`FieldLabel.tsx:102-108`）: 12px / 20px / w400 / `--color-text-secondary`。label と description の間の gap 0（labelGroup）。
- row の高さ = max(箱, ラベルブロック)。ラベル 1 行なら 24px（md）/ 20px（sm）。doc コメントの "28px/36px row height" は実装と一致しない（実測は box と label の大きい方）。**ラベルは align-items:center** なので、description 付きの複数行ラベルでは箱は縦中央（first-line 揃えではない）。
- status（detached, §5 参照）: row の下 margin-top 4px、padding 8px、radius `--radius-element` 10px。

状態別（`CheckboxIndicator.tsx:25-123`）:

| 状態 | border | background | mark | 備考 |
|---|---|---|---|---|
| unchecked rest | `--color-border-emphasized` #d4d4d4 / #525252 | `--color-background-surface` #ffffff / #262626 | なし | |
| unchecked hover（row 全体のホバー、hover:hover デバイスのみ） | color-mix(border-emphasized, tint-hover 20%) = **#aaaaaa / #757575** | color-mix(surface, tint-hover 5%) = **#f2f2f2 / #313131** | | `--color-tint-hover` #000 / #fff |
| checked / indeterminate rest | `--color-accent` #1b1b1b / #f1f1f1 | 同左 | on-accent #ffffff / #111111 | |
| checked hover | mix(accent, tint 15%) = **#171717 / #f3f3f3** | 同左 | | |
| pressed（mouse, row の `:active`） | 上記に加え `::after` overlay `--color-overlay-pressed` #0000001A / #ffffff1A（radius 6px、箱全面） | | | `CheckboxInput.tsx:95-126` |
| pressed（touch） | 同じ overlay を pressFeedback の `data-astryx-press` で描く（onset 150ms 後, §8.5） | | | |
| focus-visible | 箱自体に outline 2px solid `--color-accent`（#1b1b1b / #f1f1f1）、offset 3px（radius は箱の 6px に追従） | | | input ではなく indicator に描く（`useIndicatorFocusRing.tsx:59-95`）。キーボードフォーカス時のみ |
| disabled（unchecked） | `--color-border` #00000014 / #ffffff1A | `--color-background-muted` #f1f1f1 / #1b1b1b | | 箱全体 opacity 0.5、hover 変化なし、press overlay なし |
| disabled（checked） | `--color-border`（accent border を上書き） | accent | on-accent | opacity 0.5 |
| disabled ラベル | 色 `--color-text-disabled` #919191 / #525252 | | | cursor default |
| readOnly | 変化なし（減光しない） | | | hover/press はそのまま描かれる（indicatorScope は disabled 時のみ外れる）|
| invalid（status error） | 箱の見た目は変わらない。status box（赤）と `aria-invalid` のみ | | | |
| loading / busy | 箱の状態色はそのまま、中身を Spinner sm に置換 | | | Spinner: 直径 10px、stroke 2px（box 14px）、shade `inherit` = currentColor（unchecked 時 `--color-accent`、checked 時 `--color-on-accent`）、track は同色 opacity 0.3、arc 37.5% |
| forced-colors | mark/棒 `CanvasText` | | | native では不要（drop）|

レイアウト: label 側は `flex-direction: column`。`isLabelHidden` 時は row gap 0（ラベルは sr-only で幅 0）。RTL: row は flex なので box は reading-start 側へ自動ミラー。input は物理 `left:50%; translate(-50%,-50%)` で中央（RTL 対称）。

ASCII（md, light, unchecked → checked）:
```
 ┌──────┐            ┌──────┐
 │      │  Label     │  ✓   │  Label        box 24px, r=6, gap 8
 └──────┘            └──────┘
            Description (12/20 secondary)
```

### 8.5 Motion

| トリガー | プロパティ | from → to | duration / easing | reduced motion | 出典 |
|---|---|---|---|---|---|
| 状態変化・hover | 箱の background-color, border-color | 旧色 → 新色 | `--duration-fast` **125ms**, `--ease-standard` cubic-bezier(0.24, 1, 0.4, 1) | 0s（即時） | `CheckboxIndicator.tsx:35-40` |
| チェックマーク / 棒の出現 | display none ↔ block | — | **アニメーションなし（即時）** | — | `CheckboxIndicator.tsx:94-123` |
| mouse 押下 | `::after` overlay | transparent → overlay-pressed | 即時（transition なし） | 同じ | `CheckboxInput.tsx:95-126` |
| touch 押下 | overlay の強さ `--astryx-press-alpha` | 0 → 1（finger が 10px slop 内で 150ms 静止後に on）、release で 1 → 0 | release fade **200ms** cubic-bezier(0, 0, 0.2, 1)。150ms 未満の素早いタップはリフト時に 100ms full 表示してから 200ms fade | fade は `min(200ms, 200ms×tempo)`（neutral は tempo 未設定 → 200ms）| `interactionOverlay.stylex.ts`, `pressGesture.ts:38-67` |
| busy 開始 | Spinner 回転 | dash offset 0 → -π·d（1 周） | `--duration-slow-min` **525ms** linear infinite | 3s/周 | `Spinner.tsx:228-231, 280-290` |
| disabledMessage tooltip | Layer enter: opacity 0→1, translateY(8px)→0, scale 0.95→1 | | 165ms（`--duration-fast-max`）ease-standard, show delay 200ms | fine pointer: なし / coarse: fade ≤200ms | `Layer/layerAnimations.stylex.ts` |

### 8.6 Behaviour & a11y

- クリック: row 全体（箱・ラベル・description）がヒット領域。description のクリックは `useInputContainer` 経由で input に転送（テキスト選択中・中のリンク/ボタンは転送しない）。
- キーボード: Tab でフォーカス、Space でトグル（native checkbox）。Enter は何もしない。
- indeterminate から操作すると checked（native の挙動：checked=!checked → true）。onChange は `true`。
- disabled: native `disabled` を使わず `aria-disabled="true"`。フォーカス可能のまま、トグル不可、`onClick` も渡さない。form から切り離す。disabledMessage があれば row ホバー / フォーカスで tooltip（describedby にも追加）。
- readOnly / busy / disabled: onChange をブロックし indeterminate を復元。
- 露出: role checkbox、name = label、state checked/mixed（native indeterminate プロパティ、`aria-checked` は付けない）、`aria-required`（実効値）、`aria-invalid`（status error）、`aria-busy`（busy）、`aria-readonly`、describedby = [consumer, description, status message, disabled reason]。
- busy 開始時に live announce "Loading"（`@astryx.spinner.loading`、`useBusyAnnouncement`）。
- i18n キー案: `soft.checkbox.loading` = "Loading"、`soft.field.optional` = "Optional"、`soft.field.required` = "Required"（§4 と共用）。

### 8.7 Kivact 実装方針

- 戦略: **RESTYLE `kivact::ui::checkbox` + kit wrapper `soft::checkbox_input`**。箱は `UiRecipes::checkbox`（`CheckState`）の `Style::paint` で描く（`crates/kivact-ui/src/look.rs:170-205` の `draw_checkbox` と同じ構造。kivact-ui-basic では `crates/kivact-ui-basic/src/recipes.rs:150-221` の `toggle_row` / `draw_checkbox` / `edge` が手本）。
  - recipe 内で `state.input`（hovered / pressed / focus_visible / disabled）から上表の色を選ぶ。hover 色は事前計算済み hex を Tokens に持たせる（color-mix は native に無い → kit Tokens に `checkbox_border_hover` 等の派生色を定義）。
  - root（row）の `toggle_row` 相当: `padding_left = 1(ring 余白) + 24 + 8`、min_height 24、ラベルは 14px/20px w500 text-secondary。focus ring は row ではなく箱の周囲に paint（outline 2px, offset 3px, radius 6+3=9px の角丸ストローク）。
  - チェックマーク: viewBox 10 の polyline (8.5,2.5)→(4,7.5)→(1.5,5) を box 内 12/14px 正方形にスケールし stroke 1.5×scale、round cap/join。kivact core の既定 polyline 座標とは違うので必ず置き換える。
- wrapper（`soft::checkbox_input`）が足すもの: description（ラベル下、クリックでトグル）、`status`（§5 FieldStatus detached）、`disabled_message`（`kivact::ui::tooltip(..).attach(row)`、disabled でも focusable を維持できるか要確認 — kivact::ui の disabled がフォーカス不可にするなら、wrapper 側で `aria-disabled` 相当（focusable + 操作ブロック）を持つ必要あり → **UNRESOLVED: kivact::ui::Checkbox に "focusable disabled" モードがあるか**）、`read_only`、`loading`/`change_action`（busy 中は spinner を箱に描画し入力をブロック）、required/optional サフィックス。
- 色トランジション 125ms: recipe の paint は補間されない（`TransitionProp::Paint` が補間するのは opacity / 単色背景 / border 色 / transform / shadow のみ、custom paint は即時）。箱を paint で描く限り色は即時切替になる。忠実度を上げるなら箱を「子ノード（background + border を Style で持つ）」として描き `Style::transition(TransitionProp::Paint, Transition::tween(125, Easing::CubicBezier(0.24, 1.0, 0.4, 1.0)))` を掛ける必要がある → kivact::ui::checkbox の recipe は Style のみなので、core に「indicator 子ノードの recipe」を足すか、即時切替を許容するか判断が要る（**UNRESOLVED / 要判断**。fallback: 即時切替でも見た目の差は 125ms のみで許容範囲）。
- spinner 回転: `kivact::motion::animated` の無限 repeat（`Transition::repeat(Repeat::Forever)`）で角度を回し paint で arc を描く。
- touch の press 遅延（150ms onset）: kivact の pressed 状態はホストが即時に立てる想定。touch でのみ遅延させる API は無いと思われる → fallback: 即時 pressed（UNRESOLVED: kivact の PartState が pointer 種別を持つか）。

### 8.8 受け入れ確認

- [ ] `role=checkbox` name="<label>" が 1 つ。Space でトグル、Enter では変化しない。
- [ ] indeterminate 時 AT 上 mixed。クリックで checked=true になる。
- [ ] ラベル・description クリックでトグル。
- [ ] disabled + disabledMessage: Tab でフォーカスでき、tooltip が出て、Space で変化しない。
- [ ] readOnly: 減光なし・操作不可。
- [ ] loading: 箱に spinner、操作不可、"Loading" がアナウンスされる。
- [ ] 目視（docsite light/dark）: 箱 24px 角丸 6px、未チェック枠 #d4d4d4（dark #525252）、チェック時塗り #1b1b1b（dark #f1f1f1）に白（dark #111111）のチェック、ホバーで枠が濃く（#aaaaaa / #757575）、キーボードフォーカスで箱の 3px 外側に 2px リング。
- [ ] sm で箱 20px・チェック 12px。

---

## 9. CheckboxList

ソース: `packages/core/src/CheckboxList/CheckboxList.tsx`, `CheckboxListItem.tsx`, `CheckboxListContext.tsx`、行は `packages/core/src/List/List.tsx` / `List/ListItem.tsx` / `Item/Item.tsx`、外枠は `Field/Field.tsx`。Docs: https://astryx.atmeta.com/components/CheckboxList 、Story: `CheckboxList.stories.tsx`。

### 9.1 概要

少数（3〜7 個）のチェックボックスをグループで常時表示し、複数を on/off させる。設定画面、フィルタパネル、フォーム向け。

- DO: 項目は 3〜7 個。helper text がある項目では `hasDividers` を on。グループラベルは選択肢の意味を書く（"Export formats"）。
- DON'T: 単一選択に使わない（→ RadioList）。endContent にボタン/リンクを置かない（行全体がタップ可能で競合）。disabled を Tooltip で包まない（→ `disabledMessage`）。項目が多く検索が要るなら MultiSelector。

### 9.2 Anatomy

```
Field (div.astryx-field.astryx-checkbox-list; column)        §4 Field 参照
├ FieldLabel (span — group label, id=labelID) + description
├ div[role=group][aria-labelledby=labelID][aria-describedby=desc status reason]
│ └ ul (List: column, gap 2px; hasDividers 時 gap 0)
│    └ li (ListItem → Item; delegation 行) × N         [astryx-item / list-item]
│       ├ span.startContent → CheckboxInput (size sm|md, isLabelHidden, aria-labelledby=行ラベル)
│       ├ span.content (column)
│       │  ├ span label (14/20 text-primary, 文字列なら 1 行 ellipsis)
│       │  └ span description (12/20 text-secondary)
│       └ span.endContent (margin-inline-start:auto)
└ FieldStatus detached（status.message）
Tooltip（disabledMessage, group 全体に hover/focus）
```

各行のフォーカス可能要素はチェックボックス 1 つだけ（タブストップ 1 / 項目）。行面のクリックは `interactiveRef` 経由でチェックボックスへ委譲。

### 9.3 API

CheckboxList:

| prop | 型 / default | 意味 | Kivact |
|---|---|---|---|
| `label` | string（必須） | グループラベル | `soft::checkbox_list(label, selected: Binding<Vec<K>>)` |
| `value` / `onChange` | `string[]` / `(values)=>void` | collection mode（未指定なら各 item が standalone） | `Binding<Vec<K>>`（K: Clone+Eq+Hash）。standalone mode は item 側 `checked` binding |
| `changeAction` | `(values)=>void\|Promise` | 非同期。pending 中はトグルした項目だけ spinner | `.change_action(impl Fn(Vec<K>) -> Task)` |
| `density` | `'compact' \| 'balanced' \| 'spacious'` = balanced | 行 padding。compact はチェックボックス sm | `.density(ListDensity::{Compact,Balanced,Spacious})` |
| `hasDividers` | bool = false | 行間の区切り線 | `.dividers(bool)` |
| `isDisabled`, `disabledMessage`, `isReadOnly` | | 全体 | `.disabled(..)`, `.disabled_message(..)`, `.read_only(..)` |
| `isLabelHidden`, `description`, `status`, `width` | | Field と同じ | 同名メソッド |
| `children` | ReactNode | CheckboxListItem 群 | `.items(impl IntoValue<Vec<CheckItem<K>>>)` もしくは children view（`checkbox_list_item(..)` の列） |
| `containerXstyle/ClassName/Style`, `controlXstyle/...` | | | drop（`.style` に一本化） |

CheckboxListItem:

| prop | 型 / default | 意味 | Kivact |
|---|---|---|---|
| `label` | ReactNode（必須） | 行ラベル（文字列なら 1 行 ellipsis） | `soft::checkbox_list_item(key, label)` |
| `value` | string | collection mode のキー（必須） | key K |
| `description` | ReactNode | 行の補足 | `.description(..)` |
| `endContent` | ReactNode | 末尾スロット | `.end(\|\| view)` |
| `isDisabled`, `isLoading` | bool | 行単位 | `.disabled(..)`, `.loading(..)` |
| `isChecked` / `onCheck` | `bool\|'indeterminate'` / `(checked)=>void` | standalone mode | `.checked(Binding<bool>)` + `.indeterminate(..)` |
| `align` | `'center'\|'start'\|'first-line'` | 縦揃え | `.align(ItemAlign::..)` |
| `aria-label` | string | rich label 時の名前 | `.name(..)` |
| `onClick` | handler | 行クリック通知（チェックボックス上で発火） | `.on_press(..)` |

### 9.4 Visual spec

- Field: §4 の Field レイアウト（ラベル→コントロール→status の縦並び）。グループラベルは FieldLabel（14/20 w500 text-secondary）。
- List（`List.tsx:134-183`）: `ul` column、項目間 gap `--spacing-0-5` = **2px**、`hasDividers` 時 gap 0。
- 行（Item, `Item.tsx:346-357, 550-562`）: flex row, align-items center, gap 8px, padding-inline `--_item-inset-inline` = 8px（spacious は 12px）、radius `--radius-element` **10px**。padding-block: compact 4px / balanced 8px / spacious 12px。
  - balanced md の 1 行行高 = 8 + max(24 box, 20 label) + 8 = **40px**。compact（box sm 20）= 4+20+4 = **28px**。spacious = 12+24+12 = **48px**。
- divider（`ListItem.tsx:201-212`）: border-block-end 1px `--color-border` #00000014 / #ffffff1A、最後の行は 0。行の角丸は残る。
- ラベル: 14px / 20px、`--color-text-primary` #000000 / #ffffff（Item の label。FieldLabel ではないので w400）。文字列ラベルは `overflow:hidden; text-overflow:ellipsis; white-space:nowrap`。description 12px/20px `--color-text-secondary`、文字列なら同じく 1 行省略。
- 行背景:

| 状態 | 背景 | 出典 |
|---|---|---|
| rest | transparent | |
| hover（hover:hover） | `--color-overlay-hover` #0000000D / #ffffff0D（行全面、radius 10px）＋ 箱の hover tint（§8.4） | `interactionOverlay.stylex.ts` backgroundColor arm |
| pressed（mouse :active） | `--color-overlay-pressed` #0000001A / #ffffff1A ＋ 箱の ::after overlay | |
| checked（disabled/readOnly でない） | `--color-accent-muted` **#f1f1f1 / #262626**（hover 時は overlay-hover が上書き＝hover/pressed 色が優先） | `CheckboxListItem.tsx:45-49, 330-338` |
| focus-visible（チェックボックスにキーボードフォーカス） | 行に outline 2px accent offset 3px（`Item` の `focusWithin`, radius 10px）**と** 箱自身の ring（§8.4）の両方が描かれる | `Item.tsx:911-912`, a11y spec `focusOwner()` |
| disabled（行 or 全体） | 行背景なし、content（label+description）opacity 0.5、箱は §8.4 disabled、pointer-events none | `Item.tsx:430-432, 785-790` |
| readOnly | checked 背景も出さない。箱は通常色 | |
| loading（項目） | 箱が spinner。`aria-busy` を li に | |

注意: `selected` の xstyle は interactionOverlay の後に適用されるが、overlay の hover/active は `:hover` 条件付きで優先度が高い（StyleX の条件付き値）→ hover 中は overlay 色、非 hover で accent-muted。**UNRESOLVED（軽微）**: hover 時に accent-muted と overlay-hover が重なるか置き換わるかは CSS 優先度依存。docsite で checked 行をホバーして確認すること（置換と判断）。

ASCII（balanced, hasDividers）:
```
Export formats                       (14/20 w500 secondary)
┌──────────────────────────────────┐
│ [✓]  CSV                         │  ← checked: bg #f1f1f1, r=10, h=40
├──────────────────────────────────┤  1px #00000014
│ [ ]  JSON                        │
│      Machine readable            │  12/20 secondary
└──────────────────────────────────┘
```

RTL: flex のため startContent（箱）は右へ。endContent は `margin-inline-start:auto` で逆側。

### 9.5 Motion

- 行背景: `transition: background-color --duration-fast-min` **95ms** `--ease-standard` cubic-bezier(0.24,1,0.4,1)（`Item.tsx:378-385`）。reduced motion の個別指定なし（そのまま 95ms。native では reduced motion で即時にしてよい）。
- 箱の色: §8.5 と同じ 125ms。
- touch 押下: 行が pressable（pressFeedback）。onset 150ms / slop 10px / flash 100ms / release 200ms cubic-bezier(0,0,0.2,1)。
- 項目の追加・並べ替えアニメーションなし。

### 9.6 Behaviour & a11y

- `role=group`、`aria-labelledby` = グループラベル、describedby = description, status, disabled reason。
- 各チェックボックス: name は文字列ラベル（hidden label）か、rich label のときは行ラベル span を `aria-labelledby`（hidden label は "Checkbox" = `@astryx.checkboxList.item.checkbox`）。describedby = 行 description + グループの disabled reason。
- キーボード: Tab で各チェックボックスを順に巡回（roving ではない）、Space でトグル。矢印キーの移動はない。
- 行のどこをクリックしてもトグル（中のボタン/リンク/入力は除外）。
- collection mode: トグルで `[...value, v]` / `value.filter(≠v)`（追加は末尾）。changeAction pending 中は該当項目だけ busy。
- disabled 全体 + disabledMessage: 各チェックボックスは aria-disabled で focusable、tooltip はグループに hover/focus で。
- i18n キー案: `soft.checkbox_list.item_checkbox` = "Checkbox"。

### 9.7 Kivact 実装方針

- 戦略: **NEW kit component**（`soft::checkbox_list` / `soft::checkbox_list_item`）。中身の各箱は RESTYLE した `kivact::ui::checkbox`（`.hide_label(true)`）を使い、行は kit の Item 行（§ List/Item を担当する別グループの kit `item` があればそれを使う）。
  - 行のヒット領域拡大: 行ノードの `on_press` でチェックボックスの Binding をトグル（チェックボックス自体がフォーカスを持つ。行はフォーカス不可）。
  - focus-within ring: 行が子のフォーカス可視を知る必要がある。kivact で「子孫が focus_visible」状態を親 Style が読めるか不明 → **UNRESOLVED**。fallback: 箱の ring のみ（行 ring は省略）でも可。
  - グループ: `kivact::ui::field(label, control)` は単一コントロール前提。グループラベル + description + status を持つ kit `soft::field` を §4 の方針で作り、それを外枠に使う。
- 参考: `crates/kivact-ui-basic/src/field.rs`（`field_set` / `field_group`）、`crates/kivact-ui-basic/src/item.rs`（行レイアウト）、`crates/kivact-ui-basic/src/recipes.rs`（checkbox paint）。
- `kivact::ui::list`（listbox セマンティクス, use_list）は使わない（選択モデル・ロールが違う）。

### 9.8 受け入れ確認

- [ ] `role=group` name=<label> の中に `role=checkbox` が項目数ぶん。Tab で 1 つずつ移動。
- [ ] 行の空き領域・description クリックでトグル、endContent のボタンでは行がトグルしない。
- [ ] checked 行の背景が #f1f1f1（dark #262626）、hover で #0000000D 相当のオーバーレイ。
- [ ] compact で行高 28px・箱 20px、balanced 40px、spacious 48px（1 行時）。
- [ ] hasDividers で行間 1px 線、最後の行の下に線なし。
- [ ] changeAction 中、トグルした行だけ spinner。
- [ ] 文字列ラベルは 1 行で省略記号。

---

## 10. RadioList

ソース: `packages/core/src/RadioList/RadioList.tsx`, `RadioListItem.tsx`, `RadioListReasonContext.ts`、見た目 `Indicator/RadioIndicator.tsx`、行は `Item/Item.tsx`。Docs: https://astryx.atmeta.com/components/RadioList 、Story: `RadioList.stories.tsx`, `RadioGroupA11y.stories.tsx`。

### 10.1 概要

小さな選択肢集合（2〜7）から 1 つだけ選ぶ。全選択肢が常時見えるので比較しやすい。

- DO: 選択肢は少なく（2〜7）。ラベルは一目で区別できる簡潔さで。
- DON'T: 複数選択に使わない（→ CheckboxList）。長いリストに使わない（→ Selector）。horizontal で 4 個超を並べない（折り返しが不格好）。disabled を Tooltip で包まない（→ `disabledMessage`）。
- a11y: 円の縁（未選択）と塗り（選択）は背景に 3:1 以上。

### 10.2 Anatomy

```
Field (div.astryx-field)                         §4
├ FieldLabel (span group label, id=labelID) [labelTooltip info icon]
├ div[role=radiogroup] (.astryx-radio-list; flex; vertical: column gap 8 / horizontal: row gap 20)
│   aria-labelledby, aria-describedby, aria-invalid, aria-required
│ └ RadioListItem × N  → Item（div, padding 0, radius 0, 背景なし）  [.astryx-radio-list-item]
│    ├ startContent
│    │  ├ div.radioWrapper (20/24 正方, relative, ::after = 押下 overlay 円)
│    │  │  ├ input[type=radio] (opacity 0, name=group, aria-labelledby=行ラベル)
│    │  │  └ span(display:contents) → span.astryx-radio-indicator (円) → span dot
│    │  └ [startContent（アイコン等）]
│    ├ content: span#labelID (label) / span#descriptionID (description)
│    └ endContent
└ FieldStatus detached
Tooltip（disabledMessage）
```

### 10.3 API

RadioList:

| prop | 型 / default | 意味 | Kivact |
|---|---|---|---|
| `label` | string（必須） | グループ名（可視） | `kivact::ui::radio_group(label, options, value)` は label を描かない → kit wrapper `soft::radio_list(label, options, value)` が Field ラベルを描く |
| `value` / `onChange` | `string` / `(v)=>void`（必須） | 選択値（`''` = 未選択） | `Binding<Option<K>>`（`None` = 未選択） |
| `orientation` | `'vertical'\|'horizontal'` = vertical | 並び | `.orientation(Orientation::..)`（既存） |
| `size` | `'sm'\|'md'` = md | 円 20 / 24 | `.size(ControlSize::Small\|Medium)`（既存） |
| `isDisabled`, `disabledMessage` | | 全体無効と理由 | `.disabled(..)`, `.disabled_message(..)` |
| `isRequired`, `isOptional` | | サフィックス・aria-required | `.required(..)`, `.optional(..)` |
| `status` | InputStatus | error で aria-invalid | `.status(..)` |
| `labelTooltip` | string | ラベル末尾 info アイコンの tooltip | `.label_tooltip(..)` |
| `isLabelHidden`, `description`, `width` | | | 同名 |
| `htmlName` | string | form 名 | drop |
| `children` | RadioListItem 群 | | `options: impl IntoValue<Vec<ListItem<K>>>`（kivact `ListItem` は label/description/icon/disabled を持つ）+ `.item(\|item, state\| view)` slot で start/end content を描く |

RadioListItem: `label`(ReactNode, 必須), `value`(string, 必須), `description`, `isDisabled`, `startContent`, `endContent`, `align`, `aria-label`, `onClick`。→ `ListItem<K>`（label, description, disabled）+ kit 拡張 `RadioOption<K> { item: ListItem<K>, start: Option<IconName>, end: Option<..> }` もしくは `.item(..)` slot で描画。

### 10.4 Visual spec

- グループ: vertical は column gap `--spacing-2` **8px**、horizontal は row gap `--spacing-5` **20px**（`RadioList.tsx:70-82`）。折り返しなし（flex-wrap 指定なし）。
- 行（`RadioListItem.tsx:131-170`）: Item の padding/radius/背景を **全て 0 / transparent に上書き**。行は「円 + gap 8px + テキスト」だけの素の面で、行全体のホバー背景は出ない（ホバーは円の tint のみ）。
- 円（`RadioIndicator.tsx:284-383`）: sm 20 / md 24、border 1px、radius full。dot: sm 8 / md 10、radius full、`--color-on-accent`。

| 状態 | 円 border | 円 background | dot |
|---|---|---|---|
| unchecked rest | `--color-border-emphasized` #d4d4d4 / #525252 | `--color-background-surface` #ffffff / #262626 | なし |
| unchecked hover（行のどこか, hover:hover） | #aaaaaa / #757575（20% tint） | #f2f2f2 / #313131（5% tint） | |
| checked | accent #1b1b1b / #f1f1f1 | accent | on-accent #ffffff / #111111（sm 8, md 10） |
| checked hover | #171717 / #f3f3f3 | 同 | |
| pressed（mouse, 行 `:active`、ただし行内の a/button/[role=button|link] が active のときは除く） | 円の `::after` に `--color-overlay-pressed` #0000001A / #ffffff1A（円形） | | |
| pressed（touch） | 同 overlay を data-astryx-press で（150ms onset） | | |
| focus-visible | 円に outline 2px accent offset 3px（円形）＋ 行の focusWithin ring（行は radius 0・padding 0 なので行の外接矩形 +3px に 2px 矩形 ring）| | |
| disabled | border `--color-border`、unchecked bg `--color-background-muted` #f1f1f1 / #1b1b1b、円 opacity 0.5、ラベル/description opacity 0.5 | | |
| invalid | 見た目変化なし（status box のみ） | | |

- ラベル: Item label = 14px/20px `--color-text-primary`（#000 / #fff）、w400。**RadioListItem はラベルを span で包むため折り返す（省略しない）**。description 12px/20px text-secondary（文字列でも span で包まれるので折り返し）。
- 行の高さ（1 行・md）= max(24, 20) = 24px。行は align-items center（`align` で start / first-line に変更可）。
- 行の focusWithin ring は「行（padding 0, radius 0）」を囲むので、ラベル末端まで含む矩形になる。**UNRESOLVED（要目視）**: docsite でキーボードフォーカス時に矩形の行リングが実際に見えるか確認。見えない場合は円のリングだけ実装する。

ASCII（vertical md）:
```
Plan
(●) Monthly        ← checked: 24px 円 #1b1b1b, dot 10px 白
( ) Yearly
    Save 20%       ← 12/20 secondary
                     行間 8px, 行背景なし
```

### 10.5 Motion

- 円の background/border: 125ms `--ease-standard`（reduced: 0s）（`RadioIndicator.tsx:298-303`）。
- dot の出現: 即時（条件付きレンダリング、アニメーションなし）。
- 押下 overlay: mouse 即時 / touch は §8.5 と同じ press feedback。
- 行背景トランジション（Item の 95ms）は背景が transparent 固定なので不可視。

### 10.6 Behaviour & a11y

- `role=radiogroup`（`aria-labelledby` グループラベル）。各 `input[type=radio]` は name 共有、`aria-labelledby` = 行ラベル span（`aria-label` 指定時はそれ）、describedby = 行 description + グループ disabled reason。
- キーボード（native radio）: Tab でグループに入る（選択済みがあればそれ、なければ先頭。Shift+Tab で入った場合は末尾）、↑/← 前・↓/→ 次（循環、disabled はスキップ、移動と同時に選択）。Space で現在の項目を選択。
- 未選択グループへの Tab 進入は `handleFocus` で先頭の enabled radio に正規化（`RadioList.tsx:303-364`）。
- 行のどこをクリックしても選択（Item delegation）。
- disabled: 項目単位は native disabled（フォーカス不可）。グループ全体 + disabledMessage のときは aria-disabled で focusable（理由を読ませる）。
- required: `aria-required`（実効）、radio の `aria-invalid` は status=error のときだけ true。
- i18n: Optional/Required は §4 と共用。

### 10.7 Kivact 実装方針

- 戦略: **RESTYLE `kivact::ui::radio_group` + kit wrapper `soft::radio_list`**。`RadioGroupRecipes`（`group` / `item` / `indicator` / `description`）を設定:
  - `group(Orientation)`: vertical column gap 8 / horizontal row gap 20。
  - `item(CheckState)`: padding 0, gap 8, 背景なし, ラベル 14/20 primary。focus ring は item ではなく indicator 側で描く。
  - `indicator(CheckState)`: 20/24 円の paint（border/fill/dot、hover/pressed/disabled/focus ring）。
  - `description`: 12/20 secondary。
  - 参考: `crates/kivact-ui-basic/src/recipes_controls.rs:26-97`（radio_group recipes）。
- wrapper が足すもの: Field ラベル（グループラベル + description + labelTooltip + required/optional）、status box、disabledMessage tooltip、start/end content（`.item(..)` slot で描画）。
- `Binding<Option<K>>` で未選択を表現。Tab 進入の正規化は use_list の radio policy が担う（kivact 既存挙動を確認すること。roving tabindex なら同等）。
- 色トランジション 125ms は paint では補間されない（§8.7 と同じ課題）。

### 10.8 受け入れ確認

- [ ] `role=radiogroup` name=<label>、子に `role=radio` が項目数。矢印キーで移動と同時に選択、端で循環。
- [ ] 未選択で Tab 進入 → 先頭に、Shift+Tab 進入 → 末尾に。
- [ ] 行ラベル/description クリックで選択。
- [ ] 行背景は常に透明、ホバーで円の枠だけ濃くなる。
- [ ] horizontal で項目間 20px、vertical で 8px。
- [ ] disabled 項目はスキップされ、円とテキストが半透明。
- [ ] 目視: checked 円 #1b1b1b + 白 dot 10px（dark: #f1f1f1 + #111111）。

---

## 11. Switch

ソース: `packages/core/src/Switch/Switch.tsx`（以下 `Switch.tsx:N`）、`switch.markers.stylex.ts`、`switchFill.ts`、`switchTouch.ts`（純粋モデル）、`useSwitchTouch.ts`（DOM/アニメーション）、UIKit 定数 `packages/core/src/motion/uikit/switch.ts`、ばね `motion/uikit/springs.ts:398-401`、rubber band `motion/rubberBand.ts`、hit area `utils/hitArea.ts`。neutral 上書き: `packages/themes/neutral/src/neutralTheme.ts:353-357`（switch スコープで `--color-background-gray: var(--color-border-emphasized)`）。Docs: https://astryx.atmeta.com/components/Switch 、Story: `Switch.stories.tsx`。

### 11.1 概要

即時に反映される on/off 設定のトグル。ラベル、説明、loading、validation を持つ。

- DO: 保存操作なしで即時反映される設定に使う。設定内容を表す簡潔なラベルを付ける。
- DON'T: フォーム送信で反映される選択に使わない（→ checkbox）。多値には使わない（厳密に on/off）。disabled を Tooltip で包まない（→ `disabledMessage`）。
- タッチ画面では iOS の UISwitch と同じ動き（押すと thumb が持ち上がる、横ドラッグで反転、指を離すと確定、タップはブラウザの click でトグル）。thumb は常に円。coarse pointer では不可視の hit 領域が 44×44 まで広がる（隣接ターゲットとの中点で止まる）。reduced motion ではタッチ画面はクロスフェード、マウスは即時。
- a11y: on/off 両トラックが背景に 3:1、thumb がトラックに 3:1（hover/pressed 適用後）。

### 11.2 Anatomy

```
div.astryx-switch-field (root; width)                         ← consumer の ...rest / onClick
└ div row (flex, align center, gap 8; spread 時 space-between & width 100%)   ← pressable, switchScope
  ├ [labelPosition=start なら label が先]
  ├ div.switchWrapper (sm 32×20 / md 40×24, relative, isolation, touch-action: pan-y pinch-zoom)
  │ ├ input[type=checkbox][role=switch] (opacity 0, absolute, 中央, wrapper と同寸; coarse 時は hit insets で 44px 方向へ拡大)
  │ └ div.astryx-switch (track, aria-hidden)  ::before=touch 押下層  ::after=色フェード層
  │    └ div.astryx-switch-thumb (円)  ::before=touch 押下  ::after=持ち上げ影
  │       └ (busy) Spinner sm
  └ div labelWrapper (column, justify center, min-height 20/24)
     └ FieldLabel (.astryx-switch-label)  [labelIcon] label [∙ Optional/Required] [info icon + tooltip] / description
FieldStatus detached（margin-top 8px）
Tooltip（disabledMessage）
```

### 11.3 API

| prop | 型 / default | 意味 | Kivact |
|---|---|---|---|
| `label` | string（必須） | 可視ラベル / name | `soft::switch_field(label, on: Binding<bool>)` → 内部で `kivact::ui::switch(label, on)` |
| `value` | boolean（必須） | 状態 | `Binding<bool>` |
| `onChange` | `(checked, e)=>void` | | Binding 書き込み + `.on_change(..)` |
| `changeAction` | async | 楽観表示 + busy | `.change_action(..)` |
| `isLoading` | bool=false | thumb に spinner、操作不可 | `.loading(..)` |
| `isDisabled` | bool=false | 無効（常に aria-disabled で focusable） | `.disabled(..)` |
| `disabledMessage` | string | 理由 tooltip | `.disabled_message(..)` |
| `isRequired`/`isOptional` | bool | サフィックス・aria-required | `.required(..)` / `.optional(..)` |
| `labelIcon` | icon | ラベル前アイコン（16px） | `.label_icon(..)` |
| `labelTooltip` | string | ラベル末尾 info アイコン（16px）の tooltip | `.label_tooltip(..)` |
| `labelPosition` | `'start'\|'end'` = end | ラベルをスイッチの前/後に置く | `.label_position(LabelPosition::{Start, End})` |
| `labelSpacing` | `'hug'\|'spread'` = hug | spread: ラベルとスイッチを両端に（幅 100%） | `.label_spacing(LabelSpacing::{Hug, Spread})` |
| `status` | InputStatus | detached status、error で aria-invalid | `.status(..)` |
| `size` | `'sm'\|'md'` = md | 32×20 / 40×24 | `.size(ControlSize::Small \| Medium)` |
| `isLabelHidden`, `description`, `width` | | | 同名 |
| `htmlName`, `ref`, `xstyle`, `className`, `style`, `onFocus/onBlur`, `aria-*` | | | htmlName/xstyle/className/style は drop、focus ハンドラは NodeBuilder、`aria-labelledby` は `.labelled_by(..)` 相当 |

### 11.4 Visual spec

寸法（`Switch.tsx:63-170`）:

| size | track | track padding | thumb off | thumb on | on 時 translateX | on 時の内側余白 |
|---|---|---|---|---|---|---|
| sm | 32×20 | 2px | 14×14 | **16×16** | 12px | 上下 2px, inline-end 2px |
| md | 40×24 | 4px | 16×16 | **20×20** | 14px | 上下 2px, inline-end 2px |

- thumb は on で**大きくなる**（md 16→20 / sm 14→16）。off 時は padding 内の inline-start（md: 左 4px、上下 4px）、on 時は translateX で右へ（md: 4+14 = 18px 起点、幅 20 → 右余白 2px）。
- track radius full、thumb radius full。track 枠線なし（forced-colors のみ 1px）。
- 色:

| 状態 | track | thumb |
|---|---|---|
| off rest | `--color-background-gray` → neutral の switch スコープ上書きで `--color-border-emphasized` = **#d4d4d4 / #525252** | `--color-background-surface` **#ffffff / #262626** |
| off hover（row 全体, hover:hover） | color-mix(上記, tint 5%) = **#c9c9c9 / #5b5b5b** | 同 |
| on rest | `--color-accent` **#1b1b1b / #f1f1f1** | surface #ffffff / #262626 |
| on hover | mix(accent, tint 15%) = **#171717 / #f3f3f3** | 同 |
| pressed（mouse, row `:active`） | track と thumb の両方に linear-gradient overlay `--color-overlay-pressed` #0000001A / #ffffff1A を重ねる（コントロール全体が暗く） | |
| pressed（touch） | track `::before`（thumb の下）に overlay、thumb `::before` にも overlay、thumb は持ち上がる（§11.5） | |
| focus-visible | track に outline 2px solid accent（#1b1b1b / #f1f1f1）offset 3px、pill 形（`Switch.tsx:451-467`）。disabled でも focusable なので表示される | |
| disabled | track opacity 0.5（thumb 含む）、off の track は gray のまま（hover tint なし） | |
| busy | thumb 中央に Spinner sm（直径 10, stroke 2, box 14、shade default: arc `--color-accent` #1b1b1b / #f1f1f1、ring `--color-track` #CCD3DB / #5A5E66）、input pointer-events none | |
| invalid | 変化なし（status のみ） | |

- ラベル: FieldLabel（14/20 w500 `--color-text-secondary` #474747 / #9e9e9e、disabled は `--color-text-disabled` #919191 / #525252）。labelWrapper min-height 20/24、縦中央。description 12/20 secondary。
- row gap 8px（label hidden 時 0）。spread: `justify-content: space-between; width: 100%`。
- status: detached、margin-top **8px**（`statusGap`）、padding 8、radius 10。
- RTL: on 時の translateX は `[dir=rtl]` で負（-12 / -14px）。touch 時は inset（inline-start/end）なので自動ミラー。
- coarse pointer: input の幅/高さを `100% + hit insets`（44px まで、隣接ターゲットとの中点で止める, `utils/hitArea.ts`）。見た目は変わらない。

ASCII（md, light）:
```
off:  ╭────────────────╮        on:  ╭────────────────╮
      │ ●              │ Label        │            ●   │ Label
      ╰────────────────╯              ╰────────────────╯
      40×24 #d4d4d4, ● 16px #fff       #1b1b1b, ● 20px #fff
```

### 11.5 Motion

**fine pointer（マウス・トラックパッド）**:

| トリガー | プロパティ | duration / easing | reduced | 出典 |
|---|---|---|---|---|
| 値変化 | thumb の transform（translateX 0 ↔ 12/14px）と width/height（14↔16 / 16↔20） | `--duration-fast` **125ms**, `--ease-standard` cubic-bezier(0.24, 1, 0.4, 1) | 0s（即時） | `Switch.tsx:533-547` |
| 値変化・hover | track background-color | 125ms ease-standard | 0s | `Switch.tsx:335-349` |
| 押下 | overlay（gradient） | 即時 | | |
| マウスでのドラッグ | thumb をドラッグして反転可能（同じ flip rule、アニメーターなし。値が変わった時点で上記 125ms で移動） | | | `useSwitchTouch.ts:204-207` |

**coarse pointer（タッチ。UIKit iOS 26.2 実測値, `motion/uikit/switch.ts`）**: thumb の CSS transition は無効化し、compositor アニメーター（`motion/animator.ts`）で駆動。

| 動き | 内容 | パラメータ（Kivact `Spring{stiffness, damping, mass: 1}` 換算） |
|---|---|---|
| lift（指が触れた瞬間。スクロールビュー内では content delay 後） | thumb を中心基準で scale。scale = min(58/37, 38.333/24, 1+inset/thumbW, 1+inset/thumbH)。md off: inset 4, thumb 16 → **1.25**、md on: inset 2, thumb 20 → **1.10**、sm off: inset 3, thumb 14 → **1.214**、sm on: 1+2/16 = **1.125**。同時に thumb `::after` の影（`--shadow-low`）の opacity 0→1 | response 0.23, ζ 0.55 → **k 746.3, c 30.05** |
| settle（指を離した後。ただし lift 開始から最低 **235ms** 経ってから） | scale → 1、影 → 0 | response 0.5, ζ 0.7 → **k 157.9, c 17.59** |
| travel（リリース後 or プログラム変更で端へ） | thumb translate（その時点の速度を引き継ぐ） | response 0.3, ζ 1 → **k 438.6, c 41.89** |
| tracking（ドラッグ中） | 毎 move で指位置（rubber band 適用）に再ターゲット | response 0.36, ζ 1 → **k 304.6, c 34.91** |
| rubber band（端を越えたドラッグ） | `(1 − 1/(x·c/d + 1))·d`、c = 0.55、d = 12 × trackH/28（md 10.29, sm 8.57）。ただし `inset − liftGrowth` が上限（md off: 4 − 16×0.25/2 = **2px**） | |
| track 色（on へ） | 色フェード層（track `::after`、反対側の色）を opacity で | mass 1, **stiffness 85.377, damping 18.480** |
| track 色（off へ） | 同上、速い | mass 1, **stiffness 246.740, damping 31.416** |
| 押下層 release | track/thumb `::before` opacity 1 → 0 | **200ms** cubic-bezier(0, 0, 0.2, 1)（reduced: ≤200ms） |
| 持ち上げ影 | UIKit は (0, 7) blur 3 α0.12。Astryx は `--shadow-low`（neutral light: `0 2px 4px oklch(0 0 0/5%), 0 4px 8px oklch(0 0 0/10%)`、dark: 25% / 40% + `inset 0 0 0 1px oklch(1 0 0/8%)`）の opacity フェード | |
| reduced motion（touch） | 何も移動・ばね運動しない。値変化は 200ms のクロスフェード（`timing.reducedMotionCrossFadeMs`）。ドラッグ中の指追従は直接操作として維持。lift しない | |

ドラッグの flip rule（`switchTouch.ts`）: 指が着地点から「反対端方向に `trackWidth − thumbWidth`（md off: 24px）」進むと arm、次のイベントで反転。着地点より戻れば元の側に戻る。リリース時: 反転していればその側を確定、反転していなければ**現在値の反対**（指が外にずれて離れてもトグル）。横移動 10px（pan hysteresis）でドラッグ開始、縦は page scroll に譲る。リリース後ブラウザの click を最大 **300ms** 待ち、来なければ自前で click。haptic なし。

### 11.6 Behaviour & a11y

- `input[type=checkbox][role=switch]`、name = label、checked は native。Space でトグル（Enter はトグルしない＝native checkbox）。
- row 全体（ラベル・トラック）クリックでトグル。
- disabled: 常に `aria-disabled="true"`（focusable）、onChange/onClick/onKeyDown/onKeyUp を遮断、form から切り離し。disabledMessage があれば tooltip。
- busy（isLoading / changeAction pending）: onChange 遮断、`aria-busy`、busy 開始ごとに 1 回 "Loading" を announce（`@astryx.switch.loading`）。
- `aria-invalid` = status error なら true、isRequired なら false（Chromium の未チェック required 誤判定回避）、それ以外は付けない。
- describedby = [consumer, description, status, disabled reason]。
- i18n キー案: `soft.switch.loading` = "Loading"。

### 11.7 Kivact 実装方針

- 戦略: **RESTYLE `kivact::ui::switch` + kit wrapper `soft::switch_field`**。`UiRecipes::switch`（`CheckState`）の paint で track/thumb を描く（core 既定 `crates/kivact-ui/src/look.rs:165-168` の `draw_switch`、basic `crates/kivact-ui-basic/src/recipes.rs:150-185` の `switch_metrics` / `toggle_row` が手本）。サイズ: `ControlSize::Small`→sm（32×20）、`Medium`→md（40×24）。thumb の on/off サイズ差（16→20）を必ず再現。
- wrapper: description、labelPosition（start は label → switch の順。kivact::ui::switch の root は label 込みの行なので、`.hide_label(true)` にして wrapper で FieldLabel を並べる方式が素直）、labelSpacing spread、status、disabledMessage、loading/changeAction、labelTooltip、required/optional。
- **最大の課題（UNRESOLVED / 要判断）**: thumb 移動・サイズ・track 色の 125ms トランジションは recipe の `Style::paint` では補間されない（paint は state から即時描画、`TransitionProp::Paint` が補間するのはノード自身の opacity / 単色背景 / border 色 / transform / shadow / filter）。選択肢:
  1. core への変更提案: `CheckState` に補間済み `progress: Value<f32>`（0=off, 1=on。`kivact::motion::animated` + `Transition::tween(125, Easing::CubicBezier(0.24, 1.0, 0.4, 1.0))`）を持たせ paint が読む。または switch の thumb を子ノード化して `switch_thumb` recipe（Style の translate / size / background を `TransitionProp::Paint` で補間）を追加。
  2. kit 独自 `soft::switch` を `kivact::interaction::use_switch`（`Toggle`）+ 子ノードで組む（public kivact のみで可能。ただし「標準部品は RESTYLE」の方針から外れる）。
  3. fallback: 即時切替（見た目は正しいが動きが無い）。
  推奨は 1（paint 系トグル＝checkbox/radio の色トランジションも同時に解決する）。
- touch の UIKit 物理（lift / settle / tracking / rubber band / flip）: `kivact::interaction::use_drag`（`DragPhase`, `DragUpdate`）で横ドラッグを取り、thumb のオフセットを `kivact::motion::Animated` + `Transition::spring(Spring{..})` で駆動（上表の k/c）。kivact で「pointer が coarse か / イベントの pointer 種別」を取得できるか不明 → **UNRESOLVED**。取得できない場合: ドラッグ反転 + travel spring は全ポインタで有効、lift/影はタッチ判定できるときだけ。
- hit area 44px 拡張: kivact のヒットテストを見た目より広げられるか不明（hit inset が無ければ透明パディングで代替。ただしレイアウトが変わるので coarse 時のみ）。
- 参考ファイル: `crates/kivact-ui-basic/src/recipes.rs`（switch paint）、`crates/kivact-ui/src/toggle.rs`（Switch builder: `size`, `hide_label`, `variant`）。

### 11.8 受け入れ確認

- [ ] `role=switch` name=<label>、Space でトグル、Enter ではトグルしない。
- [ ] disabled でも Tab で到達、トグル不可、disabledMessage tooltip 表示。
- [ ] loading 中は thumb に spinner、"Loading" が 1 回だけ読まれる。
- [ ] labelPosition=start でラベルが左（RTL では右）、spread でラベルとスイッチが両端。
- [ ] 目視 md light: off track #d4d4d4 に白 16px thumb（左 4px）、on track #1b1b1b に白 20px thumb（右 2px）。dark: off #525252 + #262626 thumb、on #f1f1f1 + #262626 thumb。
- [ ] マウスでクリック時、thumb が 125ms でスライドしつつ拡大（ease-standard: 速く始まりゆっくり終わる）。reduced motion で即時。
- [ ] キーボードフォーカスで track の 3px 外に 2px pill リング。
- [ ] (タッチ端末) 押すと thumb が膨らみ影が出る、横ドラッグで反転、離すと確定。

---

## 12. Slider

ソース: `packages/core/src/Slider/Slider.tsx`（以下 `Slider.tsx:N`）、`useSliderTouch.ts`、`sliderTouchController.ts`、UIKit 定数 `motion/uikit/slider.ts`、ばね `motion/uikit/springs.ts:453-469`、値 tooltip `Tooltip/useTooltip.tsx`。Docs: https://astryx.atmeta.com/components/Slider 、Story: `Slider.stories.tsx`, `SliderA11y.stories.tsx`。

### 12.1 概要

境界内の数値（単一）または範囲（2 thumb）をドラッグで選ぶ。marks（目盛り）、値の書式化、縦向きに対応。音量・価格・割合など連続範囲を探るとき。

- DO: ラベルを必ず付ける（非表示でも）。値は単位付きで書式化（"$50", "75%"）。
- DON'T: 正確な数値入力に使わない（→ NumberInput 併用）。ステップが粗すぎて位置が数個しかないなら SegmentedControl / radio。disabled を Tooltip で包まない（→ `disabledMessage`）。
- タッチ: UISlider と同じ追従（thumb だけがドラッグを受ける・hit は thumb より大きい・値は指の移動量に追従しジャンプしない・リリース後 coast してから `onChangeEnd`・トラックからのスワイプはページスクロール・トラック/mark タップで値設定）。
- a11y: thumb はトラックと背景に 3:1（pressed overlay 込み）。

### 12.2 Anatomy

```
Field (label = group label span, description, status detached)          §4
└ div.astryx-slider (sliderRow: flex, align center, gap 8)
  ├ (htmlName) input[type=hidden] × values           → native では不要
  ├ div.astryx-slider-control (trackContainer; h 20 / 縦 w20×h160; relative;
  │     touch-action none(fine)/manipulation(coarse); user-select none)   range 時 role=group aria-labelledby=label
  │ └ div.travelBox (absolute inset 0; container-type:size)
  │    ├ div.astryx-slider-track (背景トラック 4px)
  │    ├ div.fillClip (4px, overflow hidden, radius full) → div.fillMover → div.fill (accent)
  │    ├ div.marks → [div.mark (2×8)] [span.markLabel]
  │    └ [span valueAnchor (tooltip の位置基準)] div.astryx-slider-thumb[role=slider][tabindex=0] × 1|2
  │         (coarse 時 ::before = 48px hit 領域) + Tooltip(値)
  └ span.textValue (valueDisplay='text')
VisuallyHidden "Required"（required 時、describedby）
```

### 12.3 API

| prop | 型 / default | 意味 | Kivact |
|---|---|---|---|
| `label` | string（必須） | Field ラベル（group label） | `soft::slider(label, value: Binding<f64>)` / `soft::range_slider(label, low, high)` → 内部 `kivact::ui::slider` / `range_slider` |
| `value` | `number \| [number, number]`（必須） | 単一 or 範囲 | 単一: `Binding<f64>`、範囲: `Binding<f64>` ×2（既存 range_slider の形） |
| `onChange` | `(v)=>void` | 連続変更 | Binding 書き込み |
| `onChangeEnd` | `(v)=>void` | ドラッグ終了・キー操作ごと・coast 後 | `.on_edit(\|phase\| ..)`（`EditPhase` の終了）を `.on_change_end(..)` として公開 |
| `min` / `max` / `step` | 0 / 100 / 1 | | `.range(min..=max)`, `.step(step)`（既存） |
| `minStepsBetweenThumbs` | number = 0 | 範囲 thumb の最小間隔（step 単位） | `.min_steps_between(n)`（kivact::ui に無ければ kit 側で clamp。**要確認**） |
| `orientation` | `'horizontal'\|'vertical'` = horizontal | | `.orientation(..)`（既存） |
| `formatValue` | `(v)=>string` | 表示・aria-valuetext | `.format(impl Fn(f64) -> String)` |
| `valueDisplay` | `'tooltip'\|'text'\|'none'` = tooltip | 値の出し方 | `.value_display(ValueDisplay::{Tooltip, Text, None})` |
| `marks` | `{value, label?}[]` | 目盛りとラベル | `.marks(impl IntoValue<Vec<SliderMark>>)`（`SliderMark { value: f64, label: Option<String> }`） |
| `isDisabled`, `disabledMessage`, `isRequired`, `isOptional`, `status`, `labelTooltip`, `isLabelHidden`, `description`, `width` | | | 同名 |
| `htmlName`, `ref`, `id`, container*/control* xstyle 系 | | | drop |
| `onPointer*`, `onKeyDown`, `onFocus`, `onBlur` | | consumer ハンドラ | drop（必要なら NodeBuilder） |

### 12.4 Visual spec

定数: `TRACK_SIZE = 4`, `THUMB_SIZE = 20`（`Slider.tsx` 定数部）。

- control（horizontal）: 高さ 20px（coarse 時 min 24px）、幅 100%。vertical: 幅 20px（coarse min 24）、高さ **160px**。cursor pointer。
- 背景トラック: 太さ 4px、全長、縦中央、radius full、色 `--color-track` **#CCD3DB / #5A5E66**（neutral は上書きしていないので core 既定。他の neutral 色と違いわずかに青み）。
- fill: 4px、`--color-accent` **#1b1b1b / #f1f1f1**。単一は始端（RTL では右端）から thumb 中心まで、範囲は thumb0 中心〜thumb1 中心。
- thumb 位置: 移動範囲はトラック両端から thumb 半径 10px 内側。中心 x = 10 + p·(W − 20)（p = (v−min)/(max−min)）。トラック自体はコントロール全幅（thumb は端でトラック端に揃う）。
- thumb: 20×20 円、背景 `--color-accent`（**白い縁・影なし**、塗りのみ）、z-index 1、cursor grab。

| 状態 | thumb |
|---|---|
| rest | accent #1b1b1b / #f1f1f1 |
| hover（thumb 自身, hover:hover） | mix(accent, tint 15%) = **#171717 / #f3f3f3** |
| dragging（pointer 押下中の thumb） | + linear-gradient `--color-overlay-pressed` #0000001A / #ffffff1A を重ねる |
| focus-visible（**キーボード操作によるフォーカス時のみ**。ポインタでフォーカスしても出さない） | outline 2px solid accent offset 3px（円形） |
| disabled | thumb 背景 `--color-background-muted` #f1f1f1 / #1b1b1b、control 全体 opacity 0.5、cursor default |
| invalid | 見た目変化なし（aria-invalid と status のみ） |

- marks: horizontal 2×8px（縦長の棒）、vertical 8×2px、radius full、トラック中央線上に中心。色: 範囲内（単一: v ≤ value / 範囲: start ≤ v ≤ end）は accent、それ以外は `--color-track`。位置は thumb 中心と同じ式（端 10px 内側）。
- mark label: 12px（`--text-supporting-size`）、`--color-text-secondary` #474747 / #9e9e9e、nowrap。horizontal は中心線から下 **14px**（`THUMB_SIZE/2 + 4`）に上端、translateX(-50%) で中央揃え。vertical は中心線から inline-start 側 14px、translateY(50%)。mark/label はクリックでその値へ。**mark label はコントロールの高さに含まれない**（absolute）ので、下に余白が必要なら呼び出し側で確保（Astryx も同じ）。
- text 表示（`valueDisplay='text'`）: トラック右に gap 8px、14px（`--text-label-size`）、`--color-text-primary`、nowrap、flex-shrink 0、範囲は `"{a} – {b}"`（en dash 前後スペース）。
- 値 tooltip（`valueDisplay='tooltip'`, 既定）: thumb 上（vertical は start 側）、offset 4px、表示遅延 **0ms**、hover/focus で開き、**ドラッグ中はその thumb の tooltip を強制表示**（他 thumb は閉じる）。見た目は Tooltip: bg `--color-text-primary` #000000 / #ffffff、文字 `--color-background-surface` #ffffff / #262626、radius 12px、14/20、padding 4px 8px。disabledMessage 表示中は値 tooltip を出さない。
- coarse pointer: 各 thumb の `::before` が 48×48 の hit 領域（各辺 −14px）。範囲で thumb が近いときは向かい合う側の拡張を `clamp(0, (spread·(travel−20) − 20)/2, 14)` に縮める。
- RTL（horizontal）: 進行方向が右→左（thumb 位置・fill・矢印キーがミラー）。
- Field レイアウト: §4（ラベル上、status 下）。

ASCII（horizontal, 単一, marks + label）:
```
Volume
          ┌──┐ 40%
──────────●────────────────────   track 4px #CCD3DB / fill #1b1b1b / thumb 20px
 |    |    |    |    |            marks 2×8
 0   25   50   75  100            12px secondary, 中心線から 14px 下
```

### 12.5 Motion

| トリガー | プロパティ | duration / easing | reduced | 出典 |
|---|---|---|---|---|
| hover / 押下 / disabled 切替 | thumb background-color, box-shadow | `--duration-fast` **125ms** ease-standard cubic-bezier(0.24, 1, 0.4, 1) | 0s | `Slider.tsx:405-415` |
| 値変化（fine pointer: クリック・ドラッグ・キー） | thumb 位置・fill | **トランジションなし（即時）** | | |
| 値 tooltip 出現 | Layer enter（opacity 0→1, translateY(8px)→0, scale .95→1） | 165ms ease-standard | fine: なし / coarse: fade ≤200ms | `Layer/layerAnimations.stylex.ts` |

coarse pointer（タッチ、`sliderTouchController.ts` + `motion/uikit/slider.ts`）:

| 動き | 内容 | パラメータ（Kivact Spring 換算, mass 1） |
|---|---|---|
| lift（指が thumb に触れる） | thumb を中心基準 scale 1 → **1.55** | response 0.215, ζ 0.6 → **k 854.0, c 35.07** |
| tracking | 値 = 着地時の値 + (max−min) × 移動量 / トラック全長（gain 1）。移動量は pan 開始閾値 **10px** を越えた点から数える（ジャンプしない） | 直接（ばねなし） |
| release coast | 指を離した速度 v(px/s) で投影距離 = (v/1000)·0.996/(1−0.996) = 0.249·v px、その **40%** だけ進む目標へばね（範囲端で止まる） | response 0.4, ζ 1 → **k 246.7, c 31.42** |
| drop（離した時） | scale → 1（coast と並行） | response 0.5, ζ 0.7 → **k 157.9, c 17.59** |
| 端の haptic | 値が端に入るたび 1 回（`impactMedium`）、離れると再アーム | native: プラットフォーム haptic があれば |
| 補足 | coast が 0.5px 未満かつ 1px/s 未満なら再生しない。`onChangeEnd` は coast 完了後 | |
| トラック/mark のタップ | 指を離した時点で値を設定（UISlider には無い挙動、WCAG 2.5.7 のため） | |
| トラックからのスワイプ | ページスクロール（thumb 以外ではドラッグしない） | |
| mouse の thumb ドラッグ | 10px hysteresis 後に同じ相対追従（方向問わず） | |

reduced motion: animator の Reduce Motion 方針（ばねの代わりに 200ms クロスフェード or 即時）。

### 12.6 Behaviour & a11y

- 各 thumb: `role=slider`, `tabindex=0`, `aria-valuemin/max`（範囲時は相手 thumb ± minGap で制約）、`aria-valuenow`, `aria-valuetext`（formatValue 時）、`aria-orientation`, `aria-disabled`, `aria-invalid`。単一は label で `aria-labelledby`、範囲は thumb ごと `aria-label` "Minimum value" / "Maximum value" + control が `role=group` で label に labelledby。
- キーボード（`Slider.tsx:1300-1380`）: →/↑ +step、←/↓ −step（horizontal RTL では ←/→ 反転）、PageUp/PageDown ±10·step、Home/End で min/max（snap しない）。キー毎に `onChangeEnd`。
- マウス（fine）: トラック pointerdown → その位置の値（snap）へ最寄り thumb を移動しドラッグ開始（pointer capture、絶対追従）、mark 上ならその値。thumb 上の pointerdown はフォーカスのみ（ドラッグは相対追従の pan）。範囲で両 thumb が同値のときは動かした方向で thumb を選ぶ。pointerup / lostpointercapture で `onChangeEnd`。
- 値の正規化: clamp(min,max) → snap = min + round((v−min)/step)·step（小数精度は min と step の桁数で丸め）。範囲は交差しない（minGap 維持）。
- required: describedby に視覚非表示 "Required"（**ハードコード英語** → i18n キー化）。
- i18n キー案: `soft.slider.minimum_thumb` = "Minimum value"、`soft.slider.maximum_thumb` = "Maximum value"、`soft.slider.required` = "Required"。

### 12.7 Kivact 実装方針

- 戦略: **RESTYLE `kivact::ui::slider` / `kivact::ui::range_slider` + kit wrapper `soft::slider`**。`SliderRecipes`（`track` / `range` / `thumb`, `AxisState`）:
  - track: 4px, radius full, `--color-track`（kit Token として #CCD3DB / #5A5E66）。コントロールの box は 20px（縦 20×160）。
  - range: accent 塗り。
  - thumb: 20px 円 accent、hover/pressed/disabled を state style で、focus ring は `focus_visible`（キーボード時のみ表示されることを確認）。
  - 参考: `crates/kivact-ui-basic/src/recipes_controls.rs:99-176`（slider recipes）。
  - 端の 10px 内側インセット（thumb がトラック端からはみ出さない）: kivact::ui::slider の thumb 位置計算が同じ前提か要確認（**UNRESOLVED**）。違えば recipe の padding で調整。
- wrapper が足すもの: Field ラベル/description/status/required、marks（track の `Style::paint` で棒を描き、ラベルは下に絶対配置のノード）、値 tooltip（`kivact::ui::tooltip` を thumb に attach、delay 0、ドラッグ中は強制 open — kivact::ui::Tooltip に controlled open があるか要確認）、text 表示、disabledMessage、minStepsBetweenThumbs。
- フォーカスリングのキーボード限定表示: Astryx は modality 追跡で pointer フォーカス時に ring を出さない。kivact の `focus_visible` が同等なら recipe だけで済む。
- タッチ物理（lift 1.55 / coast / 相対追従）: kivact::ui::slider が絶対追従なら、タッチ相対追従は core 変更なしでは再現困難 → **UNRESOLVED**。fallback: 絶対追従 + lift のみ（thumb の scale を `Transition::spring(Spring { stiffness: 854.0, damping: 35.07, mass: 1.0 })`）。
- `onChangeEnd` は `.on_edit(EditPhase)` の終了フェーズで実装。

### 12.8 受け入れ確認

- [ ] 単一: `role=slider` name=<label>、→ で +step、PageUp で +10step、Home/End で端。値は aria-valuenow / valuetext に反映。
- [ ] 範囲: slider が 2 つ（"Minimum value" / "Maximum value"）、group name=<label>、交差しない。
- [ ] トラッククリックで最寄り thumb がジャンプ、mark クリックでその値。
- [ ] hover/focus で値 tooltip（遅延なし）、ドラッグ中は表示し続ける。
- [ ] marks: 範囲内の棒が accent、ラベルが中心線から 14px 下。
- [ ] disabled: 半透明、thumb が #f1f1f1（dark #1b1b1b）、操作不可。
- [ ] 目視 light: 4px #CCD3DB トラック、#1b1b1b fill、20px #1b1b1b thumb（縁なし）。dark: #5A5E66 トラック、#f1f1f1 fill/thumb。
- [ ] マウスでクリックしてフォーカスしても ring は出ず、Tab で入ると 3px 外に 2px ring。
- [ ] vertical: 高さ 160px、↑ で増加、tooltip は start 側。



---

## 13. Selector

docs: https://astryx.atmeta.com/components/Selector ・ source: `packages/core/src/Selector/`（`Selector.tsx` 2210 行, `SelectorOption.tsx`, `SelectorBottomSheet.tsx`, `selectorPresentation.stylex.ts`, `useSelectorPresentation.ts`, `SelectorRowLayoutContext.ts`, `hooks.ts`, `utils.ts`, `types.ts`）・ stories: `apps/storybook/stories/Selector.stories.tsx`, `SelectorEmptyStateA11y.stories.tsx`

### 13.1 概要

- 単一値をリストから選ぶドロップダウン（combobox + listbox）。フォーム・設定画面で「中程度の数」の選択肢に使う。閉じたトリガーで文字を打つとネイティブ select と同じく一致項目を**直接確定**（開かない）、開いている間は文字でハイライト移動・Enter で確定。`hasSearch` では閉じたトリガーへの打鍵でポップアップが開き検索欄に文字が入る。
- 使う: 可視ラベルを付ける／8 項目を超えたら section・divider で整理／カスタム行は `renderOption`（`SelectorOption` を JSX 子として渡さない）／意味のある placeholder（"Choose a country"）／ナビ・ツールバーで secondary ボタン横なら `variant="secondary"`、ghost ボタン横なら `ghost`（その場合 status は `statusVariant="tooltip"` 推奨）／コンパクトなタッチ画面でシートにしたいなら `presentation="adaptive"`／InputGroup 内は短い prefix/suffix のときだけ。
- 使わない: アクションメニュー（→ DropdownMenu）／2 択（→ SegmentedControl・Radio）／ナビゲーション（リンクにする）／yes/no（→ Switch・Checkbox）／セクション無しで ~20 超（→ Typeahead）／disabled 理由を Tooltip で包む（→ `disabledMessage`）。

### 13.2 Anatomy

```
Field (standalone 時のみ。InputGroup 内では省略; label / description / status message)
└─ div.control-box  [theme target astryx-selector; anchor; onClick=onTriggerClick; press feedback]
   ├─ startIcon?            (Icon sm, color secondary)
   ├─ VisuallyHidden#inputLabel   (InputGroup 内のみ: label)
   ├─ VisuallyHidden#readOnlyDesc (readOnly 時: "Read only")
   ├─ button#trigger  role=combobox（hasSearch かつ editable のときは role 無し）
   │    aria-haspopup=listbox|dialog(sheet) aria-expanded aria-controls=listbox
   │    aria-activedescendant aria-labelledby aria-describedby aria-required
   │    aria-invalid aria-busy aria-disabled aria-readonly tabIndex=0
   │    └─ valueContent: [option icon (startIcon 無い時)] [span.label = 選択 label | placeholder]
   │                    or renderValue(option)（SelectorRowLayoutContext 下）
   ├─ input[type=hidden name=htmlName]?（Kivact では不要）
   ├─ Spinner sm aria-hidden （busy 時）
   ├─ InternalInputClearButton （hasClear && value!=null && !disabled && !readOnly）
   └─ status icon | status button(tooltip) | chevronDown（readOnly では何も出ない）
Popup（popover: usePopover surface "astryx-selector-popup" role=none / bottom-sheet: SelectorBottomSheet）
└─ [hasSearch] div
     ├─ PanelSearchInput (role=combobox, aria-autocomplete=list, aria-activedescendant)
     ├─ Divider（full width, margin 0）
     └─ div#listbox role=listbox aria-labelledby=trigger
   [no search] div#listbox role=listbox（sheet 時 aria-label=label, tabIndex=0, onKeyDown）
     ├─ div role=option id=`${listboxId}-item-${i}` aria-selected aria-disabled  [astryx-selector-option-row]
     │    ├─ span.itemContent → SelectorOption(icon,label,description) | renderOption(item)
     │    └─ span.markColumn → CheckIndicator（未選択時は null → :empty で列ごと消える）
     ├─ Divider aria-hidden（divider option。検索中は出さない）
     ├─ div role=group aria-label=title → div.sectionHeading aria-hidden + option…
     └─ div.emptyState role=presentation （0 件時）
```

- 選択肢データ: `string | {value,label?,description?,disabled?,icon?,action?} | {type:'divider'} | {type:'section',title?,options}`（`types.ts:13-54`）。`action` は Selector では描画されず dev warning（MultiSelector のみ対応）。

### 13.3 API

| Astryx prop | 型 / 既定 | 意味 | Kivact |
|---|---|---|---|
| `label` | string（必須） | Field ラベル・アクセシブル名・sheet 見出し | `soft::selector(label, options, value)` 第 1 引数 |
| `options` | `SelectorOptionType[]` | 上記 4 形 | `impl IntoValue<Vec<SelectorEntry<K>>>`。`enum SelectorEntry<K>{ Option(ListItem<K>), Divider, Section{ title: Option<String>, items: Vec<ListItem<K>> } }`（`ListItem` は icon/description/disabled を既に持つ） |
| `value` / `onChange` | string / `(v)=>void`。`hasClear` 時は `string\|null` | 制御値 | 第 3 引数 `impl Into<Binding<Option<K>>>`（caller-owned） |
| `changeAction` | `(v)=>void\|Promise` | 非同期確定・楽観表示・busy | 削除。caller が binding を書き、非同期中は `.loading(signal)` を渡す |
| `hasClear` | bool=false | clear ボタン + Delete/Backspace で null | `.clearable(impl IntoValue<bool>)` |
| `isLabelHidden` | false | ラベル視覚非表示 | `.label_hidden(bool)` |
| `description` | string | ラベル下説明 | `.description(text)` |
| `isOptional`/`isRequired` | false | Field 表示 + aria-required | `.optional(bool)` / `.required(bool)` |
| `isDisabled` | false | 減光・操作不可・tab stop は残す | NodeBuilder `.disabled(..)` |
| `disabledMessage` | string | disabled 時の理由 tooltip（hover/focus） | `.disabled_message(text)` |
| `isReadOnly` | false | 値表示・フォーカス可・surface と編集 UI を除去（disabled 優先） | `.read_only(impl IntoValue<bool>)` |
| `isLoading` | false | Spinner + busy | `.loading(impl IntoValue<bool>)` |
| `placeholder` | `'Select…'` | 未選択表示 | `.placeholder(text)` |
| `size` | `'sm'\|'md'\|'lg'`='md'（SizeContext 継承） | 高さ 28/32/36 | `.size(ControlSize::{Small,Medium,Large})` |
| `variant` | `'input'\|'secondary'\|'ghost'`='input' | トリガー外観 | `.variant(SelectorVariant::{Input,Secondary,Ghost})`（`kivact::ui::Variant` 実装） |
| `status` | `{type:'warning'\|'error'\|'success', message?}` | 枠色 + アイコン + メッセージ | `.status(FieldStatus{ tone: StatusTone, message: Option<Text> })` |
| `statusVariant` | `'attached'\|'detached'\|'tooltip'`。既定 attached（secondary/ghost では attached→detached に強制） | メッセージの置き方 | `.status_placement(StatusPlacement::..)` |
| `width` | SizeValue | Field 全体幅 | `.width(Length)` |
| `labelTooltip` | string | ラベル末尾 info tooltip | `.label_tooltip(text)` |
| `startIcon` | ReactNode\|IconType | 先頭アイコン（選択肢の icon より優先） | `.start_icon(impl IntoView)` |
| `renderOption` | `(option)=>ReactNode` | 行内容差し替え（mark 列は残る） | `.option(|item: Value<ListItem<K>>, state: Value<ListItemState>| view)`（`kivact::ui::Select::option` と同形） |
| `renderValue` | `(option)=>ReactNode` | 閉じたトリガー内の表示 | `.value_view(|item: Value<ListItem<K>>| view)` |
| `indicatorPosition` | `'start'\|'end'`='end' | check の位置 | `.indicator_position(IndicatorPosition::{Start,End})` |
| `hasSearch` | false | パネル上部に検索行 | `.searchable(bool)` |
| `searchPlaceholder` | `'Search…'` | | `.search_placeholder(text)` |
| `emptyText` | `'No options'` | 0 件表示 + polite 読み上げ | `.empty_text(text)` |
| `emptySearchText` | `'No results found'` | 検索 0 件 | `.empty_search_text(text)` |
| `placement` | `LayerPlacement`（未指定=選択行をトリガーに重ねる macOS 方式） | | `.placement(Option<Placement>)`（None=overlay 方式） |
| `presentation` | `'popover'\|'bottom-sheet'\|'adaptive'`='popover' | | `.presentation(Presentation::{Popover,BottomSheet,Adaptive})` |
| `isDefaultOpen` | false | マウント時に開く | `.default_open(bool)` |
| `htmlName` | string | hidden input | **drop**（ネイティブにフォーム送信なし） |
| `ref` | Ref<button> | トリガー | `.node_ref(NodeRef)`（トリガーに付く） |
| `onClick`/`onFocus`/`onKeyDown` | handler | トリガーに合成 | 原則 drop。必要なら `.on_press(..)`（内部トグルの前に呼ぶ）のみ |
| `id`, `aria-labelledby`, `aria-describedby` | | | `.labelled_by(NodeRef)` / `.described_by(NodeRef)` を追加結合 |
| `containerXstyle/ClassName/Style`, `controlXstyle/ClassName/Style` | | Field 外箱 / コントロール箱へ | `.container_style(Style)` / `.style(Style)`（NodeBuilder は control box に効く） |
| `data-testid` | | | `.test_id(..)` |

### 13.4 Visual spec

**共通トークン（neutral 解決値 light / dark）**

| 用途 | token | light | dark |
|---|---|---|---|
| トリガー背景 (input) | `--color-background-surface` | #ffffff | #262626 |
| トリガー枠 | `--color-border-emphasized` | #d4d4d4 | #525252 |
| 文字 | `--color-text-primary` | #000000 | #ffffff |
| placeholder / section 見出し / empty | `--color-text-secondary` | #474747 | #9e9e9e |
| chevron / start icon / search icon | `--color-icon-secondary` | #6a6a6a | #9e9e9e |
| フォーカス枠・check・focus ring | `--color-accent` | #1b1b1b | #f1f1f1 |
| フォーカス内側リング | `--color-accent-muted` | #f1f1f1 | #262626 |
| secondary 塗り | `--color-neutral` | #0000000F | #ffffff1A |
| hover overlay / 行ハイライト | `--color-overlay-hover` | #0000000D | #ffffff0D |
| pressed overlay | `--color-overlay-pressed` | #0000001A | #ffffff1A |
| popup 背景 | `--color-background-popover` | #ffffff | #1b1b1b |
| popup 影 | `--shadow-low` | `0 2px 4px rgba(0,0,0,.05), 0 4px 8px rgba(0,0,0,.10)` | `0 2px 4px rgba(0,0,0,.25), 0 4px 8px rgba(0,0,0,.40), inset 0 0 0 1px rgba(255,255,255,.08)` |
| divider | `--color-border` | #00000014 | #ffffff1A |
| error 枠/アイコン | `--color-border-error`→`--color-text-error` | #c0262c | #ffaea7 |
| warning | `--color-border-warning` | #836700 | #eec448 |
| success | `--color-border-success` | #2f7d33 | #90ca90 |
| status hover inset (2px) | `--shadow-inset-error/warning/success` | #de47454D / #f8d36a4D / #2f7d334D | 同左（両モード同値） |
| radius | `--radius-element` / `--radius-container` | 10px / 12px | 同 |
| font | `--font-family-body` | Figtree, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif | 同 |

**トリガー（control box）** — `inputWrapperStyles.base`（`Field/inputStyles.stylex.ts:32-73`）の上に `styles.triggerContainer`（`Selector.tsx:127-153`）+ `sizeStyles`（`:524-540`）。

- 箱: `display:flex; align-items:center; justify-content:space-between; gap 8px (--spacing-2); width 100%; box-sizing border-box; padding-inline 12px (--spacing-3); border 1px solid; radius 10px`。
- 高さは padding で決まる: line-height = `max(min(20px(--spacing-5), max(size−2px, 1.5em)), 20px, 1rem)` = 20px。padding-block = `(size − 20 − 2)/2` → **sm 3px / md 5px / lg 7px**、min-height 28/32/36。1 行なら外寸ちょうど 28/32/36。`renderValue` が 2 行（例 SelectorOption stacked + description: 20+20）なら 48/52/56（1 行増えるごとに +20px）。
- 文字: 14px（`--text-label-size`）、weight 400（input）、色 text-primary。未選択時は箱全体 `color: text-secondary`（placeholder）。label span は `flex:1; min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; text-align:start`（`:186-193`）。iOS のみ coarse 時 16px floor（Kivact では drop）。
- 内側 button: padding 0、背景透明、`flex:1 1 0; min-width:0; gap 8px`、自前 outline なし（`:155-182`）。
- アイコン: startIcon / 選択肢 icon / chevron / status icon はすべて Icon sm = **16×16**、lucide stroke 2、`flex-shrink:0`。chevron/start は `--color-icon-secondary`、status icon は `--color-icon-{error,warning,success}`（= 上表の status 色）。
- 子の並び: `[startIcon] [button: (option icon) label] [Spinner sm] [clear 20×20] [status | chevron]`。

```
input / md (32px)
┌──────────────────────────────────────────┐ 1px #d4d4d4, r10, bg #fff
│ 12 [ico16] 8 Apple…………………… 8 [✕20] 8 [⌄16] 12 │  14px/20px, pb 5
└──────────────────────────────────────────┘
```

**バリアント差分**

| | input | secondary (`:290-314`) | ghost (`:257-283`) |
|---|---|---|---|
| 幅 | 100%（Field 幅） | auto（内容幅） | auto |
| 枠 | 1px emphasized | 0 | 0 |
| 背景 | surface | `--color-neutral` + overlay は `background-image` で重ねる | transparent + overlay |
| 文字 weight | 400 | 500 (`--font-weight-medium`) | 500 |
| line-height | 上記 cap 式 | `max(20px,20px,1rem)`=20px | 同 |
| hover | inset `0 0 0 2px` color-mix(emphasized 30%) = light `#d4d4d44D` / dark `#5252524D`（`(hover:hover)` かつ非 focus・非 disabled のみ） | overlay-hover を塗りの上に（box-shadow なし） | overlay-hover |
| pressed（mouse :active） | 変化なし | overlay-pressed + `scale(0.98)` | 同 |
| focus | `:focus-within` で 枠 → `--color-accent` + `box-shadow: inset 0 0 0 2px --color-accent-muted`（マウス focus でも出る） | `:has(:focus-visible)` のときのみ outline `2px solid --color-accent`, offset 3px（`focusOutlineStyles.focusWithin`） | 同 |
| 高さ | 28/32/36 | 28/32/36（min-height） | 同 |

- secondary/ghost の overlay（`interactionOverlayStyles.backgroundImage`）: hover `linear-gradient(overlay-hover)`、mouse `:active` は overlay-pressed、coarse pointer では `:active` を無効にし `data-astryx-press="on"` で pressed（共通 press feedback: onset 150ms, slop 10px, quick-tap flash 100ms, release fade 200ms `cubic-bezier(0,0,0.2,1)`）。
- **状態**
  - disabled（`inputWrapperStyles.disabled`）: opacity 0.5、枠 emphasized、hover ring なし、cursor default。secondary/ghost は overlay と scale も無効（`:315-322`）。input では disabled でもキーボード focus 時のみ accent 枠 + inset ring を復活（`inputDisabledFocusStyles.ring`, `Field/inputDisabledFocus.ts:18-32`）。tab stop は残る（aria-disabled）。
  - readOnly: 減光なし、cursor default、chevron/clear なし、secondary/ghost は overlay/scale 無効（`:323-333`）。
  - status（input のみ枠色）: `inputStatusBorderStyles[type]`（focus 中も status 色を維持）+ hover inset `--shadow-inset-{type}`。status icon は chevron を置換（`statusVariant` が detached 以外）。tooltip placement では icon が button になり focus-visible ring（2px accent, offset 3px, radius 10px）。
  - open: chevron 180° 回転。
  - pointer で閉じた直後の focus 復帰: `selectorPresentationStyles.pointerRestoredFocus` = 枠 emphasized、box-shadow/outline なし（`selectorPresentation.stylex.ts:110-117`）→ マウス操作後はリングを出さない。
  - busy（isLoading / 楽観更新中）: Spinner sm（Spinner 節参照）を clear の前に表示、aria-busy。
- **InputGroup 内**: Field 無し、`groupStyles.inGroup`（高さ 100%、隣接側の角丸 0、先頭以外 margin-inline-start −1px）、`triggerInGroup`（min-height 0, padding-block 0, line-height 20px）、value 箱は `max-height:100%` で切る。`renderValue` 内の `SelectorOption` は `SelectorRowLayoutContext='inline'` により 1 行化（13.x 参照）。ghost は inGroup を適用しない。
- **clear ボタン**: ghost/sm/icon-only Button、**20×20**（`height:20px; aspect-ratio 1/1`）、X アイコン 16px icon-secondary、tooltip "Clear"、名前 "Clear {label}"。coarse pointer 時のみヒット領域を −2px 拡張（24×24）。pointerdown を preventDefault（フォーカスを奪わない）。

**ポップアップ surface（popover presentation）** — `usePopover` の surface（`Popover/usePopover.tsx:83-88`）+ `layer.render` の位置決め要素に `styles.popover` と `layerAnimations[placement]`（`Selector.tsx:1934-1945`）。

- 背景 `--color-background-popover`（#ffffff / #1b1b1b）、**radius 12px**（`--radius-container`）、**shadow `--shadow-low`**（上表）、枠線なし（dark は shadow の inset 1px 8% 白がリムになる）。surface 自体の padding なし。
- 幅: `min-width: anchor-size(width)` = control box の外幅（枠込み）以上、内容が広ければ広がる。最大は viewport − 両側 gutter（gutter = max(16px, safe-area)）。
- 位置: placement 指定時 or `hasSearch` 時 = `placement ?? 'below'`、alignment start、**offset 4px**（`--spacing-1`）、収まらなければ反対側へ flip / viewport 内へスライド（Layer の fallback）。
- placement 未指定かつ検索なし（既定）= **選択行オーバーレイ**: `useSelectedItemOffset`（`hooks.ts:71-157`）。offset 0 で anchor 直下に置いた後、`margin-block-start: −offset px` で持ち上げる。`desiredTop = anchorCenterY − (選択行(無ければ先頭行)の listbox 内中心Y − scrollTop) − 1px(光学補正)`、`clampedTop = clamp(desiredTop, 0, viewportH − listboxH)`、`offset = max(0, anchorBottom − clampedTop)`。測定（transform を除いた offsetTop 系）が終わるまで listbox は `opacity:0`（`dropdownHidden`）。
- listbox（`styles.dropdown`, `:354-377`）: `max-height 300px; overflow-y:auto; padding 4px`。input variant かつ popover のときだけ inline padding **5px**（4 + border 1px; `:378-382`）→ 行テキストがトリガーのテキストと同じ x（枠1+12px）に揃う。
- 検索行（`PanelSearchInput`, `Field/PanelSearchInput.tsx:62-137`）: 外側 padding 4px（input+popover は inline 5px）、field = `flex; gap 8px; padding 6px 8px; radius 10px` → 高さ 32px。虫眼鏡 16px icon-secondary、入力 14px / line-height 20px（`--text-label-leading` 1.4286）text-primary、placeholder text-secondary、枠・背景なし。キーボード focus 時のみ `box-shadow: inset 0 0 0 2px --color-accent`（outline ではない）。query がある間 clear 20×20（"Clear Search options"）。下に Divider 1px `--color-border`（margin 0、listbox の外、スクロールしない）。

**オプション行**（`styles.item` `:441-462` + `itemSizeStyles` `:548-557`）

| size | padding (block / inline) | 1 行の行高 | description 付き |
|---|---|---|---|
| sm | 4 / 8 | 28px | 48px |
| md | 6 / 8 | 32px | 52px |
| lg | 8 / 8 | 36px | 56px |

- `display:flex; align-items:center; justify-content:space-between; gap 8px; width 100%; radius 10px; font 14px; color text-primary; 背景 transparent; outline none`。
- 内容（SelectorOption → Item）: label 14px/20px（`--text-body-size/leading`）text-primary、単一行 ellipsis。description 12px/20px（`--text-supporting-*`）text-secondary、文字列なら単一行 ellipsis。icon 16px icon-secondary、icon と label の gap 8px。
- mark 列（`:475-484`）: `inline-flex; min-width 1rem(16px); flex-shrink 0`。中身は `useIndicator('check')` = CheckIndicator: 選択時のみ lucide `check` 16×16、色 `--color-accent`（#1b1b1b / #f1f1f1）、disabled 行では `--color-icon-disabled`（#919191 / #525252）。未選択時は何も描かず `:empty{display:none}` で列ごと消える＝**未選択行は mark 分の幅を予約しない**（選択行と未選択行でラベル可用幅が 24px 違う、仕様）。`indicatorPosition='start'` で mark を先頭へ。
- 状態: highlighted（キーボード/ホバー/押下追従）= 背景 `--color-overlay-hover`（#0000000D / #ffffff0D）、遷移なし（即時）。selected = `font-weight 500` + check（背景は変えない）。disabled = opacity 0.5、cursor default、ハイライトされない。pressed の別色はない。
- section 見出し（`:425-433`）: padding 4px 8px、12px/20px、text-secondary、user-select none、罫線なし → 高さ 28px。group 自体に余白なし。
- divider option（`:436-438`）: 1px `--color-border`、margin-block 4px、listbox の content 幅いっぱい。
- empty state（`:412-418`）: padding 12px、text-align center、14px、text-secondary（行高は継承 20px → 44px）。
- ポップアップ内の基本文字は Layer の text reset（14px / 20px / 400）。

**bottom-sheet presentation**（`SelectorBottomSheet.tsx`）: lazy `BottomSheet`（`height="hug"`, `purpose="info"` = swipe/scrim/Escape で閉じる）の中に `Section paddingBlockStart=16px, paddingBlockEnd=0, paddingInline=4px` → `Heading level 3`（`margin-block-end 8px; margin-inline 12px`）= label → 同じ panel content。sheet の surface・scrim・ドラッグは BottomSheet 節（オーバーレイ群文書 §4）に従う。sheet は coarse かつ compact 画面でしか出ないため neutral の adaptation（base 16 / ratio 1.1736）が効く: label 16px/24px、supporting 14px/20px、heading-3 19px/28px bold → 行は sm 32 / md 36 / lg 40px。sheet では input variant の 5px 補正は付かない。
- `adaptive` の判定: `(width < 768px and pointer: coarse), (height < 500px and pointer: coarse)`（`theme/themeAdaptations.ts:83-94`）。開いている間は解決済み presentation を固定。

**RTL**: すべて logical（padding-inline, text-align start, margin-inline）。mark・アイコン順は反転。chevron は左右対称なので反転不要。horizontal の enter keyframe のみ RTL 用に符号反転（下表）。

### 13.5 Motion

| 対象 | trigger | property: from → to | duration | easing | 備考 / file:line |
|---|---|---|---|---|---|
| popup 入場（placement=below / 既定 overlay も below 扱い） | 開く | opacity 0→1, `translateY(-8px) scale(0.95)` → `translateY(0) scale(1)` | `--duration-fast-max` **165ms** | `--ease-standard` cubic-bezier(0.24,1,0.4,1)、fill backwards、delay 0 | transform-origin = 既定 center。`Layer/layerAnimations.stylex.ts:32-38, 92-101, 117-125` |
| popup 入場 above | 開く | `translateY(+8px) scale(0.95)` → 0/1 | 165ms | 同 | `:40-46`。flip しても方向は**要求した** placement のまま |
| popup 入場 end / start | 開く | end: `translateX(-8px)`、start: `translateX(+8px)`（RTL で符号反転）, scale 0.95→1 | 165ms | 同 | `:48-83` |
| popup 退場 | 閉じる | なし（即時に消える） | 0 | — | exit アニメーション無し |
| reduced motion（fine pointer） | | アニメーション無し（即表示） | 0 | | `'@media (prefers-reduced-motion: reduce)': 'none'` |
| reduced motion（coarse） | | opacity 0→1 のみ | `min(165ms,200ms)` = 165ms | ease-standard | `fadeIn` `:87-90` |
| listbox 本体の fade（overlay 方式のみ） | 位置測定完了 | opacity 0→1 | 125ms (`--duration-fast`) | CSS `ease` = cubic-bezier(0.25,0.1,0.25,1) | RM+fine: 0s、RM+coarse: 200ms cubic-bezier(0.42,0,0.58,1)。`Selector.tsx:354-377, 400-404` |
| chevron 回転 | open 切替 | rotate 0° ↔ 180° | 125ms | ease-standard | transform-origin center。**Selector には RM 上書きなし**（RM でも回る）`:246-254` |
| トリガー枠/影 | hover・focus・status | border-color, box-shadow | 125ms | ease-standard | RM 0s。`inputStyles.stylex.ts:58-63` |
| secondary/ghost press | mouse :active / `data-astryx-press=on` | transform scale 1→0.98（overlay 色は gradient のため即時切替） | 125ms（base の duration を継承） | ease-standard | RM 0s。touch の release fade 200ms cubic-bezier(0,0,0.2,1) |
| 行ハイライト | hover / key | background | 即時 | — | 遷移なし |
| 検索欄 focus ring | keyboard focus | box-shadow none → inset 2px accent | 125ms | ease-standard | RM 0s。`PanelSearchInput.tsx:81-86` |
| bottom sheet | 開閉 | BottomSheet 節に従う | | | |
| keyboard スクロール | ハイライト移動 | scrollIntoView `block: nearest`（ホバーではスクロールしない） | 即時 | | `hooks/useHighlightedOptionScroll.ts` |

Kivact 変換: popup = `style().enter(style().opacity(0.0).translate_y(-8).scale(0.95)).transition(TransitionProp::Paint, Transition::tween(165, Easing::CubicBezier(0.24,1.0,0.4,1.0)))`（decorative=true で RM 即時。coarse の fade は `kivact::platform::capabilities().touch && reduced_motion()` のとき enter を `opacity(0.0)` だけにして `.decorative(false)` の 165ms）。exit は設定しない。

### 13.6 Behaviour & a11y

**キーボード（トリガー、`hasSearch=false`）** — `useCombobox.onKeyDown`（`hooks.ts:346-497`）の前に `useTypeahead` が走る（`Selector.tsx:1403-1422, 1469-1482`）。

| キー | 閉 | 開 |
|---|---|---|
| ArrowDown | 開いて index 0 をハイライト（disabled でも 0 — 源のまま） | 次の enabled へ。末尾で止まる（wrap しない） |
| ArrowUp | 開いて最終 index をハイライト | 前の enabled へ。先頭で止まる |
| Enter / Space | 開いて選択値（無ければ 0）をハイライト | ハイライト（enabled）を確定して閉じる |
| Home / PageUp | （preventDefault のみ） | 先頭 enabled |
| End / PageDown | 同 | 末尾 enabled |
| Escape | — | 閉じる（focus はトリガー） |
| Tab | 通常 | 閉じて通常の Tab 移動 |
| Delete / Backspace | `hasClear` かつ値ありなら clear | 無視 |
| 文字キー | typeahead: 一致を**即確定**（開かない）し label を polite で読み上げ | typeahead: ハイライト移動 |

- typeahead: バッファ 750ms でリセット、同一文字の連打は次の一致へ循環（1 文字時は現在位置 +1 から探索）、スペースはバッファ途中なら文字として扱う、ダイアクリティカル畳み込み（`utils/typeaheadMatch.ts`）、disabled はスキップ、閉/クリア時にバッファ破棄。
- `hasSearch=true`: トリガーは role 無しの button（aria-haspopup/expanded/controls は残る）。トリガー上の印字可能文字（Ctrl/Meta 以外）で開き query に追記（`onSearchSeed`）。開くと rAF 後に検索入力へ focus、caret を末尾へ。検索入力が combobox: ArrowUp/Down/PageUp/PageDown/Enter/Escape を処理、Home/End は caret、Space は文字。開いた時点でハイライト無し（-1）→ 矢印を押すまで Enter は何もしない。Tab: query ありなら clear ボタンへ（開いたまま）、clear ボタンから Tab で閉じる、Shift+Tab は閉じる。IME 変換中のキーは無視。閉じると query・typeahead・live region をクリア。
- ポインタ: control box クリックでトグル（disabled/readOnly は無反応）。行ホバーでハイライト（スクロールしない）。listbox の press model（`useMenuPress`）: 押したまま行の上を動くとハイライト追従、**離した行**を確定。mouse を外で離すと閉じる、指は開いたまま。行 click で確定して閉じる、disabled 行は無反応。外側押下で light dismiss。
- フォーカス: DOM focus はトリガー（or 検索入力）に留まり、ハイライトは `aria-activedescendant`（トリガーへは listbox がマウント済みのときのみ）。閉じた時: Escape・確定・何もない所の押下 → トリガーへ戻す。外側の別の focusable を押した場合はそこに残す。pointer 由来の復帰時はリング抑制。sheet では listbox（or 検索）に rAF で focus、sheet 退場完了後にトリガーへ focus。
- readOnly: combobox role を維持、aria-readonly、aria-expanded=false、haspopup/controls なし、description に "Read only"。
- disabled: aria-disabled、tab stop 維持、caller handler は focus/blur 以外遮断、開いている surface 内の確定も拒否。`disabledMessage` は control box の hover / focusin で tooltip（above）、aria-describedby に入る。
- role/name/state: trigger `combobox`（name = Field label（InputGroup では group label + input label）、value はボタン文字列）、`aria-haspopup=listbox`（sheet では `dialog`）、`aria-required`、`aria-invalid`（error のみ）、`aria-busy`。listbox `aria-labelledby=trigger`（sheet かつ検索なしは `aria-label=label`）。option `aria-selected`=値と一致、`aria-disabled`。section は `group` + `aria-label=title`、見出しは aria-hidden、divider は aria-hidden、empty は `presentation`。
- live announcements（polite）: busy 開始 "Loading"、閉トリガー typeahead 確定時の label、検索の件数 "{count} results"（query 空でクリア、loading 中は無言、0 件は描画された empty テキストを読み上げ）。
- 既定テキスト（i18n key 提案）:

| key | English |
|---|---|
| `soft.selector.placeholder` | Select… |
| `soft.selector.search_placeholder` | Search… |
| `soft.selector.search_options` | Search options |
| `soft.selector.empty` | No options |
| `soft.selector.empty_search` | No results found |
| `soft.selector.result_count` | {count} result / {count} results（複数形） |
| `soft.selector.clear` | Clear {label} |
| `soft.input.clear_tooltip` | Clear |
| `soft.input.clear_search` | Clear {label}（= "Clear Search options"） |
| `soft.input.read_only` | Read only |
| `soft.input.status.error` / `.warning` / `.success` | Error details / Warning details / Success details |
| `soft.spinner.loading` | Loading |

### 13.7 Kivact 実装方針

- **戦略: RESTYLE `kivact::ui::select` + NEW kit component `soft::selector`**。
  - `kivact::ui::select` は `SelectRecipes`（trigger / surface / option / check / indicator / description）を 13.4 の input/md の見た目に restyle する（標準 select を使うアプリも同じ見た目になる）。ただし標準 select の挙動は Astryx と異なる（閉トリガーの矢印で値が変わり開かない、開くと実 focus が行へ移る、section/検索/clear/status/variant/overlay 位置決め/sheet が無い）ため、Astryx 互換の部品は kit の `soft::selector` として新規に作る。
  - 参考: `crates/kivact-ui/src/select.rs`（行スロット・recipe 適用の形）、`crates/kivact-ui-basic/src/combobox.rs`（`use_combobox(..).popup(open)` と overlay の組み方、side button が focus を奪わない `focus_on_press(input)`）、`list_popup.rs`（surface/row style と check の local paint）、`recipes_popup.rs`（SelectRecipes の差し替え方）、`native_select.rs`。
- 部品:
  - 振る舞いは 26.3 の kit フック（`soft::use_combobox`、`d-hooks.md`）で実装し、popup の focus 復帰・dismiss 理由は `kivact::interaction::use_popup(open)` + `FocusReturn`、配置は `kivact::view::overlay(anchor, Placement::Below, ..).align(OverlayAlign::Start).offset(4.0).at_least_anchor_width(true).on_dismiss(..)`。
  - トリガー: `node().role(Role::ComboBox).expanded(open).controls(list_ref).active_descendant(..).labelled_by(..).described_by(..).invalid(..)`（`NodeExt`）。`aria-haspopup` / `aria-required` / `aria-busy` / `aria-readonly` に当たる setter は kivact に見当たらない（`kivact-core/src/primitive/node_ext.rs` 確認）→ **UNRESOLVED**: 当面 description（described_by の非表示テキスト "Read only" 等）で補う。
  - live region: 非表示 node に `.live_region(LiveRegion::Polite)` + text signal。
  - Field 外枠は `kivact::ui::field` を restyle したもの（Field 節）を使う。status/placement は Field 節の仕様。
  - check mark・chevron は `kivact::control::icon("check" / "chevron-down")`（lucide 名）。mark 列は「選択時のみ子を持ち、未選択時は列自体を出さない」(`view::dynamic`)。
  - sheet: kit の BottomSheet（オーバーレイ群 §4）。adaptive 判定は `kivact::platform::capabilities().touch` + `kivact::platform::window_size()`（w<768 or h<500）。primary pointer が coarse かを直接得る API は無い → **UNRESOLVED**（最後の pointer の `PointerKind::Touch` で近似を提案）。
- ネイティブで難しい点と代替:
  - `anchor-size(width)` → `at_least_anchor_width(true)`。
  - 選択行オーバーレイ: `Overlay::offset` が負値を受けるか未確認 → **UNRESOLVED**。受ければ `offset = -(anchorH/2 + rowCenterY − scrollTop + 1)` を row 高さ（sm28/md32/lg36 + 4px padding、section 見出し 28、divider 9）から算出し、viewport clamp を自前で行う。受けなければ `overlay_at(point, Placement::Below, ..)` に anchor の window 座標から計算した点を渡す。測定前フレームは listbox を opacity 0 にし、確定後 125ms で 1 へ。
  - `color-mix(... 30%)` → 事前計算 `#d4d4d44D` / `#5252524D`。`light-dark()` → ThemeMode で分岐。
  - gradient overlay の重ね塗り → secondary は「塗り #0000000F の上に overlay #0000000D」を合成済み 1 色（light hover ≈ rgba(0,0,0,.111)）で描くか、`draw_over` で半透明矩形を重ねる。
  - dark の `inset 0 0 0 1px rgba(255,255,255,.08)` は shadow の 2 本目以降が Paint transition 対象外でも静的描画は可。inset shadow 非対応なら 1px 枠 `rgba(255,255,255,.08)` で代替。
  - iOS 16px floor・`touch-action`・`htmlName`・StyleX theme target は drop。

### 13.8 受け入れ確認

- [ ] role=combobox（name=label）が tab で focus でき、ArrowDown で listbox が開き最初の option が active descendant になる。Enter で確定・閉じる・focus はトリガー。
- [ ] 閉じたトリガーで "b" → "Banana" に値が変わり（開かない）、読み上げが出る。"b" 連打で次の b 項目へ循環。
- [ ] Escape / 外側押下で閉じる。外側の別ボタンを押した場合 focus はそのボタン。
- [ ] `hasClear` で ✕（name "Clear {label}"）が出て、Delete でも null になる。
- [ ] `hasSearch` で開くと検索入力（role=combobox）に focus、件数 "N results" が polite で読まれ、0 件で "No results found"。section 見出しは一致 0 の group ごと消え、divider は検索中出ない。
- [ ] disabled: 減光 0.5・tab stop あり・開かない。`disabledMessage` が hover/focus で上に出る。readOnly: 減光なし、chevron なし、開かない。
- [ ] 目視（light/dark, docsite Default/SizeVariants/GhostVariant/SecondaryVariant/WithStatus/WithSections/Searchable）: 高さ 28/32/36、枠 #d4d4d4/#525252 r10、focus で黒/白枠＋淡い内側リング、hover で 2px 薄リング。popup は r12・shadow-low、既定では選択行がトリガー上に重なって開き、上から 8px 降りつつ 0.95→1 で 165ms、閉じは即時。行 hover は薄いグレー、選択行は medium + 右端 check、未選択行は mark 幅を空けない。chevron が 125ms で反転。
- [ ] sheet（compact touch, `presentation="adaptive"`）: 見出し=label、行 36px（md）、閉じると focus がトリガーへ。

### 13.x SelectorOption（`SelectorOption.tsx`, `SelectorOption.doc.mjs`）

1. **概要**: `renderOption` / `renderValue` 内で Selector と同じ行レイアウト（icon + label + description + endContent）を作るヘルパ。JSX 子として Selector に渡さない。
2. **Anatomy**: `Item`（`Item/Item.tsx`）を埋め込み用リセット（padding 0, radius 0, `min-width:0`; `SelectorOption.tsx:18-28`）で包んだもの: `[startContent: icon sm secondary] [content: label / description] [endContent (margin-inline-start:auto)]`。theme target `astryx-selector-option`。role なし（行の role は Selector 側）。
3. **API**: `icon?: ReactNode|IconType`、`label: ReactNode`、`description?: ReactNode`、`layout?: 'stacked'|'inline'`='stacked'、`endContent?: ReactNode`、`xstyle/className/style`（drop → `.style`）。Kivact: `soft::selector_option(label).icon(view).description(text).layout(OptionLayout::{Stacked,Inline}).end(|| view)`（`Node` を返す builder、NodeBuilder 実装）。`SelectorRowLayoutContext` は kit 内部の `provide_context`/`use_context`（or 引数）で `Inline` を強制。
4. **Visual spec**: gap 8px（Item root）。label 14px/20px text-primary、文字列 label は 1 行 ellipsis。description 12px/20px text-secondary、文字列 or inline 時は 1 行 ellipsis。stacked = 縦積み（行高 40px 分）。inline = 横並び `column-gap 4px`、`align-items:center`、label `flex-shrink:0`、description が先に縮んで ellipsis（`Item.tsx:469-485`）。icon 16px `--color-icon-secondary`（#6a6a6a / #9e9e9e）。状態色は持たない（行が持つ）。InputGroup 内トリガーでは layout が強制的に inline（`SelectorRowLayoutContext`, `SelectorOption.tsx:479-490`）。
5. **Motion**: なし。
6. **Behaviour & a11y**: 非インタラクティブ。テキストは行（option）の名前に含まれる。
7. **Kivact 実装方針**: NEW kit part。`kivact::ui` の `helpers::label_and_description` 相当を kit 内で再実装（`kivact-ui-basic/src/item.rs` が近い）。inline/stacked の切替は flex direction のみ。
8. **受け入れ確認**: [ ] stacked で label の下に 12px description。[ ] InputGroup 内トリガーで 1 行になり description から ellipsis。[ ] 狭いポップアップで長い label が ellipsis（はみ出さない）。

## 14. ComplexSelector

docs: https://astryx.atmeta.com/components/ComplexSelector ・ source: `packages/core/src/ComplexSelector/ComplexSelector.tsx`（976 行）, `ComplexSelector.doc.mjs`, tests ・ stories: `apps/storybook/stories/ComplexSelector.stories.tsx`（FruitRipenessGrid, TreeListWithSearch, CategoryTreeSelector, SecondaryVariant, ControlledToolbarTrigger, AuditMatrix）

### 14.1 概要

- Selector の 1 行リストより豊かな**任意コンテンツ**（グリッド、ツリー、日付、ステージ付きエディタ）で値を選ぶための「フィールド + トリガー + dialog popover」シェル。field・trigger・popover・focus 復帰・changeAction を持ち、中身の a11y 構造は render prop 側が持つ。
- 使う: secondary/ghost ボタン横で `variant`、`startIcon`、広い surface は `alignment="end"`／Apply 型は下書きを中身に持ち Apply でだけ `onChange`／中身は RadioList・Calendar・TreeList・グリッド等の適切な構造で組み、`useGridFocus`/`useListFocus` を使う／選択で閉じたいときだけ `close()`／WCAG 2.2 で中身を評価。
- 使わない: トリガー ARIA・focus 管理・changeAction を自前で作り直す／単純な 1 列テキストリスト（→ Selector）。

### 14.2 Anatomy

```
Field（label, description, status message。renderTrigger 時は無し）
└─ div.control-box [astryx-complex-selector; anchor; onClick→trigger.click()]
   ├─ startIcon?（Icon sm secondary）
   ├─ button#trigger role=combobox aria-haspopup=dialog aria-expanded aria-controls=content
   │     aria-labelledby=label aria-describedby aria-required aria-invalid aria-busy aria-disabled
   │   └─ span.triggerText = triggerLabel ?? placeholder
   ├─ span.statusTip（statusVariant=tooltip のときのみ: info-tip button）
   ├─ Spinner sm aria-hidden（busy）
   └─ Icon chevronDown [astryx-complex-selector-indicator-icon state=expanded|collapsed]
Popover（usePopover: role=dialog aria-modal=true aria-label=label tabIndex=-1, surface astryx-complex-selector-popup）
└─ div#content（padding 12, max-height, overflow auto）
   └─ children(value, onChange, close, {isOpen,isBusy,triggerId,contentId})
renderTrigger モード: caller の control に props を spread（Field/chevron/status/spinner 無し）+ Popover
```

### 14.3 API

| Astryx prop | 型 / 既定 | Kivact |
|---|---|---|
| `label` | string（必須） | `soft::complex_selector(label, value)` |
| `value` / `onChange` | `Value` / `(v)=>void` | 第 2 引数 `impl Into<Binding<V>>`（V: Clone + 'static） |
| `changeAction` | async | drop → caller が `.loading(signal)` |
| `children` | `(value, onChange, close, state) => ReactNode` | `.content(move |cx: ComplexSelectorContext<V>| view)`（開くたびに新しい branch で呼ぶ; `cx.value(): Value<V>`, `cx.commit(V)`（disabled 中は無視）, `cx.close()`, `cx.is_open()`, `cx.is_busy()`, `cx.trigger_ref()`, `cx.content_ref()`） |
| `triggerLabel` | ReactNode | `.trigger_label(impl IntoText)` または `.trigger_content(|| view)` |
| `placeholder` | `'Select…'`（`@astryx.selector.placeholder`） | `.placeholder(text)` |
| `isLabelHidden`, `description`, `isOptional`, `isRequired`, `labelTooltip`, `width` | Selector と同じ | 同名 builder |
| `isDisabled` | false（開かない・commit も拒否） | `.disabled(..)` |
| `isLoading` | false | `.loading(..)` |
| `status`, `statusVariant` | Selector と同じ（secondary/ghost は attached→detached） | `.status(..)`, `.status_placement(..)` |
| `size` | `'sm'\|'md'\|'lg'`='md'（SizeContext を**読まない**） | `.size(ControlSize)` |
| `variant` | `'input'\|'secondary'\|'ghost'`='input' | `.variant(SelectorVariant::..)` |
| `startIcon` | | `.start_icon(view)` |
| `placement` | `LayerPlacement`='below' | `.placement(Placement)` |
| `alignment` | `LayerAlignment`='start' | `.align(OverlayAlign::{Start,Center,End})` |
| `handleRef` | `{open, close, toggle, isOpen}` | caller-owned `.open(impl Into<Binding<bool>>)`（開閉状態を caller が持つ Kivact 流儀。`onOpenChange` もこれで代替） |
| `onOpenChange` | `(isOpen)=>void` | 上記 binding の書き込みで観測 |
| `renderTrigger` | `(props)=>ReactNode` | `.attach(trigger: impl NodeBuilder + IntoView) -> View`（`kivact::ui::Dialog::attach` と同形。trigger に role ComboBox / expanded / controls / name=label を付与） |
| `contentXstyle` | | `.content_style(Style)` |
| `onClick` / `onKeyDown` | | `.on_press(..)`（内部トグルの前。Kivact では preventDefault 相当が無いので「false を返すと開かない」`impl Fn() -> bool` を提案） / drop |
| container*/control*/`data-testid`/`ref`/`id` | | Selector と同じ扱い |

### 14.4 Visual spec

- トリガー箱は Selector とほぼ同じ（`ComplexSelector.tsx:88-138`）。相違点:
  - **高さ固定** `height: 28/32/36px`（`:250-258`）、`padding-block 8px` は固定高の中で中央寄せになり実質無効、line-height `--text-label-leading` 1.4286 → 20px、padding-inline 12px、gap 8px、14px、weight 400（secondary/ghost は 500）。内容が増えても高さは変わらない（1 行のみ、`triggerText` は ellipsis）。
  - secondary は **width 100% のまま**（Selector と違い auto にしない; `:178-200`）。ghost は auto。
  - status: **枠色を変えない**（`inputStatusBorderStyles` 不使用）。status は Field のメッセージ（attached/detached）か、tooltip placement の info-tip button のみ。chevron は常に表示（status icon で置換しない）。
  - focus: input は `:focus-within` で枠 accent + inset 2px accent-muted（Selector と同じ）に**加えて**、トリガーが `:focus-visible` のとき control box に outline `2px solid --color-accent` offset 3px（`focusOutlineStyles.focusWithinCombobox`, `:866`）。secondary/ghost は outline のみ。マウス focus は outline なし。pointer 復帰時のリング抑制は無い。
  - disabled: opacity 0.5、枠 emphasized、cursor default、secondary/ghost は overlay/scale 無効。readOnly・disabledMessage・clear・InputGroup 対応は無い。
  - placeholder 色 text-secondary は `triggerLabel == null` のとき箱全体。
  - 内部 button に radius 10px（見た目には影響しない）。
- ポップアップ（`:640-647, 789-794`）: surface = popover 背景 #ffffff / #1b1b1b、radius 12px、`--shadow-low`（Selector 表と同値）、min-width = control box 幅、placement 既定 below / alignment start、**offset 4px**、viewport gutter 16px。
- content 箱（`:237-249`）: `box-sizing border-box; padding 12px (--spacing-3); overflow: auto; max-height: min(480px, 100dvh − gutterTop − gutterBottom)`（gutter = max(16px, safe-area) + LayerProvider inset）。中身の見た目は caller 次第。
- 寸法表

| size | 高さ | padding-inline | icon | 文字 |
|---|---|---|---|---|
| sm | 28 | 12 | 16 | 14/20 |
| md | 32 | 12 | 16 | 14/20 |
| lg | 36 | 12 | 16 | 14/20 |

### 14.5 Motion

| 対象 | 内容 | duration / easing | file:line |
|---|---|---|---|
| popup 入場 | `layerAnimations[placement]`: below = opacity 0 + `translateY(-8px) scale(0.95)` → 1 / 0 / 1（above +8px、end −8px X、start +8px X、RTL 反転） | 165ms, cubic-bezier(0.24,1,0.4,1), fill backwards | `:793`, `Layer/layerAnimations.stylex.ts` |
| popup 退場 | なし（即時） | — | |
| reduced motion | fine: アニメ無し / coarse: opacity のみ 165ms | | |
| chevron | rotate 0↔180°、origin center | 125ms ease-standard、**RM で 0s**（Selector と違う） | `:220-233` |
| 枠/影 | border-color, box-shadow | 125ms ease-standard, RM 0s | `inputStyles.stylex.ts:58-63` |
| secondary/ghost press | scale 0.98 + overlay | 125ms ease-standard（touch release 200ms cubic-bezier(0,0,0.2,1)） | `:150-200` |

### 14.6 Behaviour & a11y

- 開閉: トリガー click でトグル（caller `onClick` が先に走り preventDefault で開かない）。control box の他所（padding・icon・chevron）を押すと `trigger.click()` に経由。ArrowDown（閉・非 disabled）で開く。他のキーは caller `onKeyDown` へ。Enter/Space は button の既定 click で開く。
- 開くと focus trap、rAF 後に content 内の最初の focusable に focus（無ければ dialog 自体）。Escape・外側押下（light dismiss）で閉じる。閉じたら**常に**トリガーへ focus を戻してから `onOpenChange(false)`。
- content の `close()` で閉じる。`onChange` helper は disabled 中は何もしない（開いたままの中身からの commit も拒否）。
- role/state: trigger `combobox` + `aria-haspopup="dialog"`、`aria-expanded`、`aria-controls=content`、name = label（`aria-labelledby`）、`aria-required`、`aria-invalid`（error）、`aria-busy`、`aria-disabled`（tab stop 維持）。popup `dialog` `aria-modal=true` `aria-label=label`。renderTrigger 時は caller control に `role=combobox`、`aria-label=label`（caller が名前を持たない場合）、同じ disclosure 属性。
- live: busy 開始 "Loading"（polite）。
- i18n: `soft.selector.placeholder`（Select…）、`soft.spinner.loading`、status button 名は Selector と共通。

### 14.7 Kivact 実装方針

- **戦略: NEW kit component `soft::complex_selector`（中身は RESTYLE `kivact::ui::dialog(DialogKind::Popover)` を `.attach(trigger)`）**。最も近い前例は `crates/kivact-ui-basic/src/date_picker.rs`（ボタン + `DialogKind::Popover` + `.hide_title(true)` + `.width(Length::Auto)` + `Flush` variant + 確定後 `trigger_ref.restore_focus()`）と `recipes_dialog.rs`（Popover kind の surface recipe）。
- dialog popover の surface recipe はオーバーレイ群 §6 Popover と同一（bg popover / r12 / shadow-low / offset 4 / at_least_anchor_width）。ComplexSelector 用に content padding 12px・max-height `min(480, window_h − 32)`・内部スクロール（`scroll(..)`）を kit 側で付ける。
- トリガーは Selector のトリガー部品（13.7）を共有し、状態差分（固定高、status 枠なし、focus outline 追加）を `ComplexTrigger` variant で表す。
- `aria-haspopup="dialog"` 相当の setter は無い（**UNRESOLVED**、13.7 と同じ）。dialog の名前は `dialog(label, open)` の title（`.hide_title(true)`）で付く。
- focus 復帰: `use_popup` の `FocusReturn` で全 close reason → anchor を指定（Astryx は常にトリガーへ戻す）。
- 難所: なし（blur 等の CSS 依存なし）。`100dvh` → `kivact::platform::window_size().height` と `safe_area()`。

### 14.8 受け入れ確認

- [ ] combobox（name=label, haspopup dialog 相当）を Enter / ArrowDown で開くと dialog（name=label）が出て、中の最初のコントロールに focus。
- [ ] Escape / 外側押下で閉じ、focus がトリガーへ戻る。`close()` を呼ぶ中身（例 FruitRipenessGrid でセル選択）で閉じる。
- [ ] disabled: 開かず、開いた中身からの commit も値を変えない。
- [ ] 目視（light/dark, docsite FruitRipenessGrid / SecondaryVariant）: 高さ固定 32、内容が長くても 1 行 ellipsis、popup はトリガー下 4px・r12・shadow-low・padding 12、最大 480px でスクロール、入場 165ms（上から 8px + 0.95→1）、閉じは即時、キーボード focus で外側 3px の 2px リング。

## 15. MultiSelector

docs: https://astryx.atmeta.com/components/MultiSelector ・ source: `packages/core/src/MultiSelector/`（`MultiSelector.tsx` 2809 行, `hooks.ts` useMultiCombobox, `types.ts`, `MultiSelector.doc.mjs`, `useMultiCombobox.doc.mjs`, `CreateRow.a11y.chromium.spec.ts`, `OptionActions.a11y.chromium.spec.ts`）・共有: `Selector/SelectorBottomSheet.tsx`, `useSelectorPresentation.ts`, `selectorPresentation.stylex.ts` ・ stories: `MultiSelector.stories.tsx`（Default, SelectAll, Searchable, TriggerModes, DisabledItems, CreateFromQuery, RowActions, RowActionsRtl, Sizes, Status…）

### 15.1 概要

- チェックボックス付きドロップダウンで複数値を選ぶ。選択結果はトリガーに件数 / ラベル列 / バッジで表示。フィルタや有限集合からの複数選択に。
- 使う: 中程度の有限集合／~15 超で検索／`renderOption` でもチェックボックスは MultiSelector が持つ／大半が全選択しそうなら select-all／InputGroup 内は count か labels 表示で 1 行に／secondary・ghost の使い分けは Selector と同じ／compact touch は `adaptive`。
- 使わない: 単一値（→ Selector）／検索なしで ~20 超／disabled 理由を Tooltip で包む。

### 15.2 Anatomy

```
Field（standalone のみ）
└─ div.control-box [astryx-multi-selector; anchor; onClick→trigger.click()]
   ├─ startIcon? / VisuallyHidden(inputLabel, readOnly)
   ├─ button#trigger role=combobox（hasSearch&&editable 時は role なし）
   │    aria-haspopup=listbox|grid|dialog aria-expanded aria-controls aria-activedescendant …
   │   └─ span.triggerContent → placeholder | "N selected" | "A, B, C, +N" | [Badge][Badge][Badge] +N
   ├─ Spinner / Clear all ✕ / status icon | chevron
Popup（popover: role=none surface astryx-multi-selector-popup, **入場アニメ無し** / bottom-sheet）
└─ [hasSearch] PanelSearchInput + Divider
   div.dropdown（scroll container, padding 4, max-height 300）
   └─ div#listbox role=listbox|grid aria-multiselectable=true
        ├─ [create row] option "Create "q""（チェックボックス無し; 先頭）
        ├─ [select-all row] option（tri-state checkbox; 実アイテムがある時のみ）
        ├─ option|row … [astryx-multi-selector-option size select-all selected disabled]
        │    list: [inert CheckboxInput] [label | renderOption(item)] （indicatorPosition=end で checkbox を末尾）
        │    grid: gridcell#…-option(onClick toggle)[checkbox,label] + gridcell#…-action[item.action]
        ├─ Divider aria-hidden / group|rowgroup aria-label=title + heading aria-hidden
        └─ emptyState role=presentation
```

- 並び順: 開いた瞬間に選択済みだった値を**各グループ（divider 区切りの塊 / section）内で先頭へ**（`sortedItems`, `MultiSelector.tsx:1342-1398`）。開いている間は固定（トグルしても並び替えない）、閉じるとリセット。先頭に create 行、その次に select-all。

### 15.3 API

| Astryx prop | 型 / 既定 | Kivact |
|---|---|---|
| `label` | string（必須） | `soft::multi_selector(label, options, values)` |
| `options` | Selector と同じ型（`action` 有効） | `impl IntoValue<Vec<SelectorEntry<K>>>`（`ListItem` に action を持てないため `SelectorEntry::Option` を `MultiOption<K>{ item: ListItem<K>, action: Option<Rc<dyn Fn() -> View>> }` にするか、`.option_action(|key| Option<View>)` で供給） |
| `value` / `onChange(value, change?)` | `string[]` | 第 3 引数 `impl Into<Binding<Vec<K>>>`。create は `.on_create(|query: String| ..)`（caller が options と values を同一更新で追加） |
| `changeAction` | async | drop → `.loading(..)` |
| `isLabelHidden`, `description`, `isOptional`, `isRequired`, `isDisabled`, `isReadOnly`, `disabledMessage`, `isLoading`, `placeholder`（'Select…'）, `size`, `variant`, `status`, `statusVariant`, `width`, `labelTooltip`, `startIcon`, `presentation`, `isDefaultOpen`, `searchPlaceholder`（'Search…'）, `emptyText`（'No options'）, `emptySearchText`（'No results found'） | Selector と同じ | Selector と同名 builder |
| `hasClear` | false | `.clearable(bool)`（全解除。名前 "Clear all {label}"） |
| `hasSelectAll` / `selectAllLabel` | false / 'Select all' | `.select_all(bool)` / `.select_all_label(text)` |
| `hasSearch` | false | `.searchable(bool)` |
| `hasCreate` | false（hasSearch 必須、無いと dev warn） | `.on_create(..)` を与えたら有効（`searchable` 必須、無ければ debug assert） |
| `triggerDisplay` | `'count'\|'labels'\|'badges'`='count' | `.display(MultiDisplay::{Count,Labels,Badges{ max: usize }})`（max 既定 3 = `maxBadges`） |
| `formatValue` | `(items:{value,label}[])=>string` | `.format_value(|items: &[(K, String)]| String)` |
| `maxBadges` | 3 | 上記 Badges.max |
| `renderOption` | | `.option(|item, state| view)`（select-all / create 行には使わない） |
| `indicatorPosition` | `'start'\|'end'`=**'start'** | `.indicator_position(..)` |
| `renderTrigger` | | `.attach(trigger) -> View` |
| `handleRef` / `onOpenChange` | | `.open(impl Into<Binding<bool>>)` |
| `htmlName`, `onClick`, `onFocus`, `onKeyDown`, container*/control*, `ref`, `id`, `data-testid` | | Selector と同じ扱い（onClick は内部トグル**後**に走る点のみ異なる） |

### 15.4 Visual spec

（トークン解決値は 13.4 の表と同じ。追加分: `--color-text-disabled` #919191 / #525252、`--color-on-accent` #ffffff / #111111、`--color-background-gray` #e2e2e2 / #303030、`--color-text-gray` #1b1b1b / #d4d4d4、`--color-background-muted` #f1f1f1 / #1b1b1b、`--color-icon-disabled` #919191 / #525252。）

**トリガー**（`inputWrapperStyles.base` + `triggerContainer` `:131-148` + `sizeStyles` `:472-482`）

- **高さ固定** 28/32/36px、padding 8px 12px（固定高内で中央寄せ）、gap 8px、14px / line-height 20px（`--text-label-leading`）、text-primary、未選択時は箱全体 text-secondary。
- variant: input = Selector input と同じ（枠・hover inset・focus・status 枠/hover・disabled・readOnly・pointer 復帰抑制まで同一; `:2611-2643`）。secondary = 枠 0、`--color-neutral` 塗り + overlay、weight 500、**width 100% のまま**（`:259-281`）。ghost = 枠 0、透明 + overlay、weight 500、width auto（`:231-256`）。secondary/ghost の focus は outline 2px accent offset 3px（`:has(:focus-visible)`）。press scale 0.98。
- triggerContent（`:183-192`）: `flex:1 1 0; min-width:0; overflow:hidden; gap 4px; align-items:center`。
  - count（既定）: "{n} selected"（`formatValue` 優先）1 行 ellipsis。
  - labels: 先頭 3 件を ", " で結合、残りがあれば ", +N"（`formatValue` 優先）1 行 ellipsis。
  - badges: 先頭 `maxBadges`（3）件を Badge neutral（neutral テーマ上書き: 背景 `--color-background-gray` #e2e2e2 / #303030、文字 `--color-text-gray` #1b1b1b / #d4d4d4、高さ 20px、padding-inline 8px、radius 9999px、12px/20px weight 500、gap 4px）。残りは `+N` span（14px, text-secondary, weight 500, shrink 0）。`flex-wrap: wrap` だが箱が固定高 + overflow hidden のため 2 行目は切れて見えない（源のまま）。
- 子の並び・アイコン・clear（20×20, name "Clear all {label}"）・status icon・chevron は Selector と同じ。

```
md, badges
┌────────────────────────────────────────────┐
│ 12 (Name)4(Email)4(Role) +2 ……… 8 [✕] 8 [⌄] 12 │  32px 固定
└────────────────────────────────────────────┘
```

**ポップアップ surface**: Selector と同じ surface（bg popover #ffffff / #1b1b1b、radius 12px、`--shadow-low`、`min-width = anchor 幅`）、**常に placement below / alignment start / offset 4px**（overlay 位置決めなし、`placement` prop なし; `:2460-2465`）。dropdown 箱（`:321-326`）: `max-height 300px; overflow-y:auto; padding 4px`（input variant の 5px 補正は**無い**）。検索行は Selector と同じ PanelSearchInput（外側 padding 4px）+ Divider で、スクロール箱の外。

**オプション行**（`styles.item` `:368-391` + `itemSizeStyles` `:484-495`）

| 行 | sm | md | lg |
|---|---|---|---|
| 通常 padding | 4px 全周 | 6px / 8px | 8px 全周 |
| checkbox | sm 20×20 | md 24×24 | md 24×24 |
| 行高（1 行） | 28px | 36px | 40px |
| select-all padding（`:497-510`） | 2px / 4px | 4px / 8px | 4px / 8px |
| select-all 行高 | 24px | 32px | 32px |

- 行: `flex; align-items:center; gap 8px; width 100%; radius 10px; 14px; **weight 500（全行）**; text-primary; transparent`。label span は 1 行 ellipsis（`:419-424`）。Selector と違い label 単独で description は出さない（renderOption で出す）。
- highlighted（hover / key）: 背景 `--color-overlay-hover`（#0000000D / #ffffff0D）、即時。
- disabled: opacity 0.5、文字 `--color-text-disabled`（#919191 / #525252）、checkbox 自体も disabled 表示（下記）→ 二重に薄くなる（源のまま）。
- selected: 行の見た目は変えず checkbox が checked になるだけ。
- 装飾 checkbox（`CheckboxInput` を `inert` + pointer-events none で包む、`:2099-2115`; 見た目は `Indicator/CheckboxIndicator.tsx`）: border 1px、radius 6px（`--radius-inner`）。
  - unchecked: 枠 `--color-border-emphasized` #d4d4d4 / #525252、背景 `--color-background-surface` #ffffff / #262626。
  - checked: 背景・枠 `--color-accent` #1b1b1b / #f1f1f1、check 12px（sm）/ 14px（md）色 `--color-on-accent` #ffffff / #111111。
  - indeterminate（select-all の一部選択）: 同じ accent 塗り + 横棒 10×2（sm）/ 12×2（md）on-accent、radius full。
  - disabled: opacity 0.5、枠 `--color-border` #00000014 / #ffffff1A、unchecked は背景 `--color-background-muted` #f1f1f1 / #1b1b1b。
  - checkbox の hover tint は pointer-events none のため発火しない。
  - `indicatorPosition='end'` で checkbox を `margin-inline-start:auto` で行末へ（`:412-414`）。
- create 行: 先頭、label `Create "<query>"`、checkbox 無し、通常行と同じ padding/色、renderOption を使わない。
- select-all 行: 実アイテムが 1 件以上あるときのみ表示、区切り線なし、checkbox は全 enabled 選択で checked / 一部で indeterminate / 無しで unchecked（フィルタ中は表示中の enabled のみが対象）。
- グリッド行（option に `action` が 1 つでもあると以後ずっと grid; `:1261-1269`）: 行 = `flex; gap 8px`、option cell = `flex:1 1; min-width 0; gap 8px; align-self stretch`（クリックでトグル）、action cell = `flex; shrink 0; radius 10px`、中身は caller のボタン。キーボードで action cell がアクティブかつ keyboard modality のときだけ outline `2px solid --color-accent` offset 3px（`:445-460`）。select-all / create の action cell は空。
- section 見出し・divider・empty state は Selector と同じ値（`:352-365, 463-469`）。
- bottom-sheet: Selector と同じ `SelectorBottomSheet`（13.4）。

### 15.5 Motion

| 対象 | 内容 | 値 | file:line |
|---|---|---|---|
| popup 入場 | **なし**（`popover.render` の xstyle が `styles.popover` だけで `layerAnimations` を付けていない） | 即時表示 | `MultiSelector.tsx:2460-2465` |
| popup 退場 | なし | 即時 | |
| chevron | rotate 0↔180°, origin center | 125ms ease-standard、RM 上書きなし | `:222-230` |
| トリガー枠/影 | border-color, box-shadow | 125ms ease-standard, RM 0s | `inputStyles.stylex.ts:58-63` |
| secondary/ghost press | scale 1→0.98 + overlay | 125ms ease-standard、touch release 200ms cubic-bezier(0,0,0.2,1) | `:244-280` |
| checkbox 状態 | background-color, border-color（check/棒は display 切替で即時） | 125ms ease-standard, RM 0s | `CheckboxIndicator.tsx:34-40` |
| 行ハイライト | 即時 | | |
| 検索欄 focus ring | inset 2px accent | 125ms ease-standard | `PanelSearchInput.tsx:81-86` |
| sheet | BottomSheet 節 | | |

注: Selector / ComplexSelector の popup は 165ms で入場するが MultiSelector は源上アニメ無し。docsite と一致させるため Kivact でも enter を付けない（意図的差異として扱う。気になる場合は UNRESOLVED として Selector と揃えるか判断を仰ぐ）。

### 15.6 Behaviour & a11y

**キーボード**（`useMultiCombobox.onKeyDown`, `hooks.ts:189-382`; 詳細は 26.4）

| キー | 閉 | 開 |
|---|---|---|
| ArrowDown | 開いて index 0 | 次の enabled（末尾で停止） |
| ArrowUp | 開いて最終 index | 前の enabled（先頭で停止） |
| Enter / Space | 開いて index 0（選択値ではない） | ハイライトをトグル（**閉じない**）。action cell 上なら caller control を click |
| Home/PageUp, End/PageDown | — | 先頭 / 末尾 enabled |
| Escape | — | 閉じる |
| Tab | — | 閉じる |
| Delete/Backspace | `hasClear` かつ値あり → 全解除 | — |
| ArrowRight/Left（grid、行ハイライト中） | — | inline-end で action cell へ（action がある行のみ）、inline-start で option cell へ。RTL では左右反転 |
| 文字 | typeahead: 一致があれば開いてハイライト（確定しない） | ハイライト移動 |

- typeahead は独自実装: 500ms でリセット、文字を連結するだけ（同一文字循環なし）、常に先頭から探索、disabled 除外、畳み込み一致（`findTypeaheadMatch`）。
- 検索あり: Selector と同じ（検索入力が combobox、Space は文字、Home/End は caret、PageUp/Down で端、Tab→clear）。加えて grid で行ハイライト中は ←/→ を cell 移動に使う。Enter をハイライト無しで押し `createQuery` があれば create。
- create: `hasCreate && hasSearch && !isLoading`、trim した query が空でなく、既存 label（小文字化比較）と一致しないとき先頭に create 行。選ぶ（click / Enter）と `onChange([...value, query], {type:'create', query})`、query クリア、"{label} created" を読み上げ。
- ポインタ: 行 click でトグル（開いたまま）、hover でハイライト（スクロールしない）。grid では option cell click のみトグル、action cell のボタンは caller のもの（行トグルしない）、action cell hover でハイライトは action 側へ。useMenuPress の press-drag は**無い**。
- フォーカス: トリガー（or 検索入力）に留まり `aria-activedescendant`（grid の action 中は `…-action` cell id）。renderTrigger 時と sheet 時は listbox 自体が tabIndex 0 で rAF 後に focus を受けキーを処理。閉時の focus 復帰は Selector と同じ（renderTrigger 時は常にトリガーへ、focus 不能な trigger には tabIndex −1 を付与）。
- role/state: trigger `combobox` + `aria-haspopup=listbox`（grid 時 `grid`、sheet 時 `dialog`）。popup `listbox`/`grid` + `aria-multiselectable=true`。行 `option`/`row` + `aria-selected`（select-all は全選択時 true）、select-all 一部選択時は `aria-label="{label}, partially selected"`（`aria-selected="mixed"` は使わない）、grid 行は `aria-labelledby=…-option`（action は名前に含めない）。group/rowgroup + aria-label。action control に名前が無ければ dev warning。
- live（polite）: 選択変化ごとに "{count} of {total} selected" / "All selected" / "Selection cleared"、create 後 "{label} created"、検索件数 "{count} results"（同一フレーム内は最後だけ、replace 型）、0 件で create 可能なら `Create "q"`、0 件は empty テキスト、busy "Loading"。
- i18n key 提案:

| key | English |
|---|---|
| `soft.multi_selector.placeholder` | Select… |
| `soft.multi_selector.selected_count` | {count} selected |
| `soft.multi_selector.select_all` | Select all |
| `soft.multi_selector.select_all_partial` | {label}, partially selected |
| `soft.multi_selector.clear_all` | Clear all {label} |
| `soft.multi_selector.search_placeholder` | Search… |
| `soft.multi_selector.search_options` | Search options |
| `soft.multi_selector.empty` | No options |
| `soft.multi_selector.empty_search` | No results found |
| `soft.multi_selector.result_count` | {count} result / {count} results |
| `soft.multi_selector.selection_count` | {count} of {total} selected |
| `soft.multi_selector.all_selected` | All selected |
| `soft.multi_selector.selection_cleared` | Selection cleared |
| `soft.multi_selector.create_option` | Create "{query}" |
| `soft.multi_selector.option_created` | {label} created |

（`soft.input.*`, `soft.spinner.loading` は 13.6 と共通）

### 15.7 Kivact 実装方針

- **戦略: NEW kit component `soft::multi_selector`**（標準 `kivact::ui` に複数選択 select は無い。`kivact::ui::tag_input` は性質が異なる）。最も近い analog は `crates/kivact-ui-basic/src/combobox.rs` の `Vec<K>` モード（`use_combobox(options, ListSelection::Multiple(values)).popup(open)`、chips）と `list_popup.rs`。
- 振る舞いは 26.4 の kit フック `soft::use_multi_combobox`。`kivact::interaction::use_combobox` の Multiple は「トグルで query をクリア」「矢印 wrap」「Backspace で最後を削除」等 Astryx と違うので直接は使わず、kit フック内で `use_popup`・`NodeExt::{active_descendant, multiselectable, selected, role}` を組む。
- grid モード: `Role::Grid` / `Role::Row` / `Role::GridCell` は kivact にある（`kivact-types/src/semantics.rs`）。action cell の active 表示は `draw_over` ではなく outline 相当の style（2px accent、外側 3px）を kit の focus ring helper で。action の起動は caller の View の press を直接呼べないので、`MultiOption.action` を「View + `on_activate: Fn()`」の組にする（Astryx は DOM の最初のボタンを click）。
- checkbox 見た目は `kivact::ui::checkbox` の restyle（Checkbox 節）と同じ描画関数を kit 内で共有し、非インタラクティブ（focus/press を持たない node）として描く。
- badges は kit の Badge（neutral variant, neutral テーマ上書き色）を再利用。
- 難所: 固定高トリガー内の badge 折返しは clip（`overflow hidden`）で 2 行目を隠す。`aria-haspopup="grid"` 等の setter 無し（**UNRESOLVED**）。replace 型 live region（同一フレームで最後だけ）は「同フレーム内の書き込みを最後の値でまとめる」signal で近似。

### 15.8 受け入れ確認

- [ ] combobox（name=label）を Enter で開くと listbox（multiselectable）で先頭 option が active。Space で checked になり**開いたまま**、"1 of 4 selected" が読まれる。
- [ ] 開き直すと選択済みがグループ先頭に並ぶ（開いている間は並び替えない）。
- [ ] select-all: 一部選択で indeterminate 表示かつ名前 "Select all, partially selected"、押すと enabled 全選択 → もう一度で全解除（disabled の選択は保持）。
- [ ] `hasCreate`（CreateFromQuery）: 未知の query で先頭に `Create "q"`、Enter で on_create が query を受け取り "q created"。
- [ ] RowActions: → で action cell に移り（RTL では ←）、Enter で caller ボタンが動きトグルしない。
- [ ] `triggerDisplay` count / labels / badges が "2 selected" / "A, B, +1" / バッジ 3 + "+N"。
- [ ] 目視（light/dark, docsite Default / Sizes / SelectAll / TriggerModes）: 行高 sm28/md36/lg40、全行 medium、checkbox 20/24px r6 で checked は黒地白チェック（dark は白地黒チェック）、popup はトリガー下 4px・r12・shadow-low で**アニメ無しで即表示**。


---

## 16. Typeahead

Source: `packages/core/src/Typeahead/` (`Typeahead.tsx`, `BaseTypeahead.tsx`, `TypeaheadItem.tsx`, `types.ts`, `createStaticSource.ts`, `busyIndicatorLane.tsx`, `committedChoicesKey.tsx`, `trailingKeyDown.tsx`, `utils.ts`, `index.ts`)。Docs: https://astryx.atmeta.com/components/Typeahead 。Stories: `apps/storybook/stories/Typeahead.stories.tsx`, `BaseTypeahead.stories.tsx`。Tests: `Typeahead.test.tsx` (BaseTypeahead の大半の挙動もここで固定), `BaseTypeahead.test.tsx`, `BaseTypeahead.incremental.test.tsx`, `TypeaheadItem.test.tsx`, `*.a11y.chromium.spec.ts`。

この節の構成: 16.1〜16.8 が `Typeahead` 本体 (Field 付きの単一選択 search field)。検索エンジン部分 (input / async source / listbox popup / keyboard) は `BaseTypeahead` (17 節) の実装をそのまま使うので、**popup (listbox / option / empty state / group heading) の look は 16.4.5〜16.4.8 に全部書く** (17 節はそれを参照する)。`TypeaheadItem` は 16.x。

### 16.1 概要

- 大きい、または動的なデータ集合から **1 件だけ** 選ぶ search-as-you-type field。入力に応じて `searchSource` (同期 / 非同期) へ問い合わせ、結果を input の下の listbox popup に出す。選んだ値は field の中に **Token** として表示される。
- 使う場面 (`Typeahead.doc.mjs` bestPractices 要約)
  - placeholder は何を検索できるかを示す文にする。
  - 人気 / 最近の候補を入力前に見せたいときは `hasEntriesOnFocus`。
  - remote source には debounce (既定 150ms) を残す。
  - remote source で「手元の一致をすぐ出し、server の答えを後ろに足す」ときは `search` 内で `context.update(local)` を呼び、`debounceMs={0}`、server request の遅延は source 側で持ち、最終返り値は表示済みの行を先頭に保つ。
  - 1 行の prefix / suffix addon が要るときは InputGroup (6 節) の中で使う。
- 使わない場面
  - 短い静的リスト → Selector (13 節)。
  - 複数選択 → Tokenizer。
  - label で区別できない Typeahead を並べない。
  - disabled の Typeahead を Tooltip で包まない → `disabledMessage` を使う。
- **match 部分の強調表示 (太字 / mark) は存在しない**。行は item の label をそのまま描く (`TypeaheadItem.tsx`, stories / tests を検索して確認済み)。Kivact 版でも追加しないこと。

### 16.2 Anatomy

```
Field (4 節; InputGroup 内では描かない)                      Typeahead.tsx:797-824
└─ div.astryx-typeahead  (control box = Field 共通 input container + 差分)  :653-682
   ├─ [startIcon]  Icon size sm, color secondary                        :683-684
   ├─ [VisuallyHidden label]  (InputGroup 内のみ)                        :685-687
   ├─ div contentLane  (relative, flex:1, min-width:0)                   :692
   │   ├─ [span hidden id=tokenValueId] value.label                       :700-702
   │   ├─ [Token]  (value があり edit mode でない間; input の上に absolute) :703-719
   │   │    └─ span.token > button(invisible, aria-labelledby="<label> <value>")
   │   └─ <input role=combobox>  (BaseTypeahead; token 表示中は opacity 0 / tabIndex -1)
   └─ div endLane (flex, gap 4, margin-inline-start:auto)  — busy か clear がある時だけ  :367-392
       ├─ [Spinner sm role=img aria-label="Loading"]  (検索中)
       └─ [InputClearButton aria-label="Clear selection" tooltip "Clear"]  (hasClear && value && !disabled)
[disabledMessage tooltip]  (isDisabled && disabledMessage)              :788-789

popup (top layer, BaseTypeahead が描く)                                  BaseTypeahead.tsx:1753-1867
└─ [popover] layer (manual, position-area below/start, offset 4px)
   └─ div.astryx-popover  (surface: bg popover, radius 12, shadow-low)
      └─ div#listbox role=listbox aria-label="Search results" .astryx-typeahead-dropdown
         ├─ (空) div role=option aria-disabled=true .astryx-typeahead-empty-state  → emptySearchText
         └─ (結果) 表示順 = ungrouped 行 → 各 group → query 由来行 (Tokenizer の Create)
             ├─ div role=option id=<listbox>-option-<encodeURIComponent(id)> aria-selected tabIndex=-1
             │   ├─ span itemContent → renderItem(item) | TypeaheadItem(item)
             │   └─ [Icon check sm primary]  (value の行)
             └─ div role=group aria-label=<group>
                 ├─ div aria-hidden=true  (group heading)
                 └─ option ...
```

- group は `item.auxiliaryData.group` (string) で決まる (`utils/groupItems.ts:30-33`)。group の出現順を保持し、ungrouped は先頭 (`ungroupedFirst: true`)。highlight index も**表示順**で数える (`displayOrder`, `BaseTypeahead.tsx:443-454`)。
- theme target (class): `astryx-typeahead` (visualProps `size`, `status`), `astryx-typeahead-dropdown`, `astryx-typeahead-empty-state`, `astryx-typeahead-item`。

### 16.3 API

#### Typeahead props (`Typeahead.tsx:94-221`)

| Astryx prop | 型 / 既定 | 意味 | Kivact 案 |
|---|---|---|---|
| `label` | string (必須) | field label。Field が描く。InputGroup 内では visually hidden | `soft::typeahead(label, source, value)` 第 1 引数 |
| `searchSource` | `SearchSource<T>` (必須) | `search(query, context?)` / `bootstrap()` / `cancel?()` | 第 2 引数 `impl SearchSource<T>` (下記 trait) |
| `value` | `T \| null` (必須) | 選択中の item | 第 3 引数 `impl Into<Binding<Option<T>>>` (caller 所有) |
| `onChange` | `(item \| null) => void` (必須) | 選択 / clear | binding の write 側。補助に `.on_change(Fn(Option<T>))` |
| `renderItem` | `(item) => ReactNode` | 行の中身。既定 `TypeaheadItem` | `.render_item(impl Fn(&T) -> View + 'static)` |
| `placeholder` | string / `'Search…'` | token 表示中は出さない | `.placeholder(text)` |
| `hasEntriesOnFocus` | bool / false | focus 時に `bootstrap()` の結果を出す | `.entries_on_focus(bool)` |
| `maxMenuItems` | number / 10 | 表示上限 (source の結果を slice) | `.max_items(usize)` |
| `minQueryLength` | number / 1 | grapheme 数がこれ未満なら検索しない・閉じる | `.min_query_len(usize)` |
| `emptySearchText` | ReactNode / `'No results found'` | 0 件時の内容。`null` は既定へ、`''` で何も描かない | `.empty_text(text)` と `.empty(impl FnOnce() -> V)` (slot)。`Some("")` で空 |
| `isDisabled` | bool / false | 減光。**tab stop は残る** (aria-disabled + readOnly)。開かない・選ばない | NodeBuilder `.disabled(..)` (実装は 16.7) |
| `disabledMessage` | string | disabled 理由 tooltip (hover / keyboard focus、placement above) | `.disabled_message(text)` |
| `hasClear` | bool / **true** | value がある時 clear ボタン | `.clearable(bool)` (既定 true。TextInput と既定が逆なので注意) |
| `hasAutoFocus` | bool / false | mount 時 focus | `.auto_focus(bool)` |
| `size` | `'sm'\|'md'\|'lg'` / `'md'` (SizeContext 継承) | 28/32/36、token と行 padding も変わる | `.size(ControlSize)` (`ExtraSmall`→Small) |
| `debounceMs` | number / 150 | 入力から検索までの遅延。0 で即時 | `.debounce_ms(u32)` |
| `onChangeQuery` | `(q) => void` | query 変化通知 (edit mode 開始時は value.label で呼ぶ) | `.on_query(impl Fn(String))` |
| `onOpenChange` | `(open) => void` | popup 開閉通知 | `.on_open_change(impl Fn(bool))` |
| `onKeyDown` | handler | **built-in 処理の後**に呼ばれる (cancel 不可)。clear ボタン / token 上の key も受ける | `.on_key_after(impl Fn(&KeyEvent))` |
| `isLabelHidden`, `description`, `isRequired`, `isOptional`, `status`, `statusVariant` (`'attached'`), `labelTooltip`, `width` | Field 共通 | 4 節と同じ | `.label_hidden`, `.description`, `.required`, `.optional`, `.status(Option<InputStatus>)`, `.status_placement(StatusPlacement)`, `.label_tooltip`, `.width(Length)` |
| `startIcon` | `ReactNode \| IconType` | 先頭 icon (sm, secondary) | `.start_icon(impl IntoView)` |
| `containerXstyle/ClassName/Style` | — | Field root | `.container_style(Style)` |
| `controlXstyle/ClassName/Style` | — | control box | `.control_style(Style)` |
| `ref`, `id`, `aria-labelledby`, `aria-describedby`, その他 input 属性 | — | combobox input へ転送。caller の labelledby は field label と連結 | `.node_ref(..)` (input), `.labelled_by(NodeRef)` (追加), `.described_by(NodeRef)` |

#### SearchSource / SearchableItem (`types.ts`)

```ts
interface SearchableItem<A> { id: string; label: string; action?: ReactNode /*未描画・dev warn*/; element?: ReactNode; auxiliaryData?: A /* .group で grouping */ }
interface SearchContext<T> { update(items: T[]): void }
interface SearchSource<T> { search(q, ctx?): T[] | Promise<T[]>; bootstrap(): T[] | Promise<T[]>; cancel?(): void }
```

Kivact 案 (kit 共通型。Tokenizer / CommandPalette 系と共有):

```rust
pub struct SearchItem<A = ()> {
    pub id: String,               // DOM id 由来: 一意キー
    pub label: String,            // 表示・accessible name
    pub group: Option<String>,    // auxiliaryData.group
    pub data: A,                  // auxiliaryData
}
pub enum SearchAnswer<T> {        // 同期と非同期を区別する (同期は busy を出さない)
    Ready(Vec<T>),
    Later(Pin<Box<dyn Future<Output = Result<Vec<T>, UiError>>>>),
}
#[derive(Clone, Copy)] pub struct SearchContext<T> { /* generation + sink */ }
impl<T> SearchContext<T> { pub fn update(self, items: Vec<T>); }
pub trait SearchSource<T>: 'static {
    fn search(&self, query: &str, cx: SearchContext<T>) -> SearchAnswer<T>;
    fn bootstrap(&self) -> SearchAnswer<T>;
    fn cancel(&self) {}
}
pub fn static_source<A: Clone>(items: Vec<SearchItem<A>>) -> StaticSource<A>;   // createStaticSource
impl<A> StaticSource<A> { pub fn keywords(self, f: impl Fn(&SearchItem<A>) -> Vec<String> + 'static) -> Self; }
```

- `item.element` (SSR 用の pre-rendered node) は native では不要 → **drop** (`render_item` で代替)。`action` は Astryx でも描かないので **drop**。
- `createStaticSource` (`createStaticSource.ts:56-82`): `search(q)`: `q.toLowerCase().trim()` が空なら全件、そうでなければ `label.toLowerCase().includes(q)` または `keywords(item)` のどれかが `includes(q)`。`bootstrap()` は全件 (表示は `maxMenuItems` で切る)。大文字小文字以外の folding はしない。`utils.ts` はこれと型の server-safe re-export だけ (native では意味なし)。
- 内部配線 (package-internal、公開しない):
  - `busyIndicatorLane.tsx`: base が「検索開始 / 終了」を wrapper (Typeahead / Tokenizer) に渡す edge-triggered store。wrapper があると base は自分の Spinner を描かず、wrapper の end lane に 1 個だけ描く。Kivact では `Signal<bool>` 1 個 (`busy`) を engine が持ち、end lane の leaf だけが読む (再描画範囲を絞る意図はそのまま)。
  - `committedChoicesKey.tsx`: Tokenizer が「確定済み値の key / ids / 上限到達」を渡す。Typeahead では未使用。Kivact では engine の `.committed(Value<CommittedChoices>)` (Tokenizer 用、Typeahead は渡さない)。
  - `trailingKeyDown.tsx`: Typeahead の `onKeyDown` を base の key 処理の**後**に呼ぶための context。Kivact では engine の `.on_key_after(..)`。

### 16.4 Visual spec

#### 16.4.0 Token 解決表 (neutral; light / dark)

| Token | light | dark | 用途 |
|---|---|---|---|
| `--color-background-surface` | #ffffff | #262626 | control box 背景 |
| `--color-border-emphasized` | #d4d4d4 | #525252 | box 枠 (rest) |
| `--color-accent` | #1b1b1b | #f1f1f1 | focus 枠、focus ring (outline)、Spinner 弧 |
| `--color-accent-muted` | #f1f1f1 | #262626 | focus 時 inset 2px ring |
| `--color-text-primary` | #000000 | #ffffff | input 文字、行 label、Token 文字 |
| `--color-text-secondary` | #474747 | #9e9e9e | placeholder、empty state、group heading、description |
| `--color-icon-secondary` | #6a6a6a | #9e9e9e | start icon、clear (×) icon |
| `--color-icon-primary` | #000000 | #ffffff | 選択行の check |
| `--color-background-popover` | #ffffff | #1b1b1b | popup surface |
| `--color-overlay-hover` | #0000000D | #ffffff0D | highlight 行 (合成: #f2f2f2 / #272727)、hover 系 |
| `--color-overlay-pressed` | #0000001A | #ffffff1A | Token / clear の press |
| `--color-neutral` | #0000000F | #ffffff1A | Token 背景 (surface 上の合成: #f0f0f0 / #3c3c3c) |
| `--color-track` | #CCD3DB | #5A5E66 | Spinner track (neutral は上書きしない; core 既定) |
| `--color-border-error` = `--color-text-error` | #c0262c | #ffaea7 | status error 枠 |
| `--color-border-warning` = `--color-text-warning` | #836700 | #eec448 | status warning 枠 |
| `--color-border-success` = `--color-text-success` | #2f7d33 | #90ca90 | status success 枠 |
| `--shadow-inset-error` | inset 0 0 0 2px #de47454D | 同じ | status error hover |
| `--shadow-inset-warning` | inset 0 0 0 2px #f8d36a4D | 同じ | status warning hover |
| `--shadow-inset-success` | inset 0 0 0 2px #2f7d334D | 同じ | status success hover |
| `--shadow-low` | `0 2px 4px #0000000D, 0 4px 8px #0000001A` | `0 2px 4px #00000040, 0 4px 8px #00000066, inset 0 0 0 1px #ffffff14` | popup surface (`neutralTheme.ts:256-259`; oklch α を hex 化) |
| `--radius-element` | 10px | 同じ | box 角丸、行角丸 |
| `--radius-container` | 12px | 同じ | popup 角丸 |
| `--radius-inner` | 6px | 同じ | Token 角丸 |
| `--spacing-1` / `-1-5` / `-2` / `-3` / `-4` | 4 / 6 / 8 / 12 / 16px | 同じ | |
| `--size-element-sm/md/lg` | 28 / 32 / 36px | 同じ | box min-height |
| `--text-body-size/leading` | 14px / 20px (w400) | 同じ | input |
| `--text-label-size/leading` | 14px / 20px | 同じ | 行 label (weight は 400 に上書き) |
| `--text-supporting-size/leading` | 12px / 20px | 同じ | Token、description、empty、heading |
| `--font-weight-medium` | 500 | 同じ | Token、選択行 (custom content のみ効く; 後述) |
| focus outline | 2px solid `--color-accent`, offset 3px | 同じ | Token の focus |

#### 16.4.1 Control box (Field 共通の input container spec, 4.4 参照) と差分

共通 box (`Field/inputStyles.stylex.ts:32-81`) の解決値: `display:flex; align-items:center; border 1px solid --color-border-emphasized; radius 10px; bg --color-background-surface; padding-inline 8px`。hover (enabled, 非 focus, hover 可能 pointer): `inset 0 0 0 2px #d4d4d44D` (light) / `#5252524D` (dark)。focus-within: 枠 `--color-accent` + `inset 0 0 0 2px --color-accent-muted` (dark では surface と同色なので ring は見えない。そのまま再現)。

Typeahead の差分 (`Typeahead.tsx:227-244, 324-328`):

| 項目 | 値 |
|---|---|
| min-height | sm 28px / md 32px / lg 36px (`height` ではなく `min-height`; ただし wrap しないので実質固定) |
| padding-block | **3px** (= 4px − 1px。Token (size−8px) を入れても高さが跳ねないための補正) |
| padding-inline | 8px (共通のまま) |
| gap | **4px** (共通の 8px を上書き) |
| flex-wrap | なし (1 行固定。長い値は Token が ellipsis) |
| cursor | text (disabled 時 default) |

box 内の並び (LTR、`[]` は任意): `[start icon 16px] —4— [content lane flex:1 min-width:0] —4— [end lane]`。

- **content lane** (`:277-284`): `position:relative; display:flex; align-items:center; align-self:stretch; flex:1; min-width:0`。input と Token がこの中にあり、end lane とは重ならない。
- **input** (`BaseTypeahead.tsx:286-310`): `display:block; flex:1; min-width:60px; border 0; padding 0; bg transparent; outline none`; font `--font-family-body` (Figtree)、14px / 20px、w400、`--color-text-primary`、placeholder `--color-text-secondary`。(iOS coarse pointer だけ 16px 下限 → native では drop)。disabled 時 cursor default。
- **start icon** (`:683-684`): Icon size sm = 16×16px、color `--color-icon-secondary` (#6a6a6a / #9e9e9e)。
- **end lane** (`:255-260`): `display:flex; align-items:center; gap 4px; margin-inline-start:auto`。中身が無い (非 busy かつ clear なし) ときはノード自体を作らない (gap を取らない)。順序: Spinner → clear ボタン。
- **Spinner** (検索中のみ): `Spinner size="sm"` = 直径 10px + stroke 2px×2 → **14×14px box**; 弧 = 円周の 0.375、stroke-linecap round、色 `--color-accent`、track `--color-track` (opacity 1)。始点 rotate(−90deg) (12 時)。`Spinner.tsx:47-56, 296-352`。
- **clear ボタン** (`Field/InputClearButton.tsx:32-76, 113-143`; Field 4.4.6 と同じ部品): Button ghost / sm / icon-only を **height 20px** (aspect 1:1 → 20×20)、radius `--radius-element` 10px (= 円)、icon `close` 16px `--color-icon-secondary`。hover (hover 可能 pointer): bg `--color-overlay-hover` (surface 上 #f2f2f2 / #313131)、mouse press: `--color-overlay-pressed` (#e5e5e5 / #3c3c3c) + scale 0.98。tooltip 文言 "Clear"、accessible name "Clear selection"。pointerdown / mousedown は preventDefault (focus を奪わない)。coarse pointer では hit area を 24px に広げる (`::after inset -2px`)。

#### 16.4.2 Token (選択値) の配置と look

`value != null && !isEditing` の間だけ表示 (`:484, 693-719`)。

- 配置 (`tokenOverlay`, `:312-321`): content lane に対して `position:absolute; inset-block-start:0; inset-inline-start:-3px; inset-inline-end:0; width:fit-content; margin-inline-end:auto`。つまり Token の始端は control box の内側 border から 5px (外側から 6px)、上端は box 外側上端から 4px。幅は label に合わせて縮み、content lane の終端 (= end lane の手前) で **ellipsis** (Token の `max-width:100%; overflow:hidden`、label `text-overflow:ellipsis`)。start icon がある場合は icon 終端から 4−3 = **1px** の隙間 (そのまま再現)。
- input はその下に **残したまま** `opacity:0; pointer-events:none` + tabIndex −1 (`:293-299, 759-765`)。field の幅は input の固有幅で決まり、値の長さで field 幅が変わらない (shrink-to-fit 親でも同じ幅)。
- Token look (`Token/Token.tsx:123-219`; Token 節と同じ部品、color `default`、clickable 形):
  - 高さ: sm 20px / md 24px / lg 28px (= `--size-element-*` − 8px)。box の content 高さ (min-height − 2px border − 6px padding) と一致する。
  - padding-inline 8px、padding-block 0、gap 4px、radius 6px、border なし。
  - bg `--color-neutral` (#0000000F / #ffffff1A。surface 上合成 #f0f0f0 / #3c3c3c)、文字 `--color-text-primary`。
  - font: 12px / 20px / **w500** (sm も 12px)。white-space nowrap。
  - hover (hover 可能 pointer): bg に `--color-overlay-hover` を重ねる (合成 #e4e4e4 / #464646)。mouse press: `--color-overlay-pressed` (合成 #d8d8d8 / #505050)。cursor pointer。
  - focus (内側 button が focus-visible): Token 外周に outline 2px `--color-accent` offset 3px。
  - disabled: Token 自身 opacity 0.5 (box の 0.5 と重なり実効 0.25)、pointer-events none。
- 内側は透明な `<button>` (aria-labelledby = field label id + 値 id → 「Fruit Apple」、aria-describedby = input と同じ)。これが値表示中の Tab stop。

#### 16.4.3 状態ごとの box

| 状態 | 変化 |
|---|---|
| rest | 枠 #d4d4d4 / #525252、bg #ffffff / #262626、shadow none |
| hover (enabled, 非 focus) | `inset 0 0 0 2px` #d4d4d44D / #5252524D |
| focus-within (input / Token button / clear が focus) | 枠 #1b1b1b / #f1f1f1 + `inset 0 0 0 2px` #f1f1f1 / #262626 |
| open (popup 表示) | box 自体は変化なし (focus-within のまま)。input に `aria-expanded=true` |
| loading | end lane に Spinner 14px が出る (clear の左)。box 不変 |
| value 選択済み | Token 表示、placeholder 非表示、clear ボタン表示 (hasClear) |
| edit mode | Token 消え、input に value.label が入り全選択、検索が走る |
| status error / warning / success | 枠 = status 色 (focus 中も status 色のまま)。hover (非 focus): `--shadow-inset-<status>`。**status がある間は focus 時の accent-muted inset ring は出ない** (status hover 用 boxShadow が base の boxShadow 全体を置き換えるため; `inputStatusHoverShadowStyles` `:123-151`)。error のときだけ input に `aria-invalid=true` |
| disabled | box opacity 0.5、枠 #d4d4d4 / #525252 (status 色より優先)、hover ring なし、cursor default、clear 非表示、Token も disabled。input に keyboard focus-visible が来たときだけ focus look (枠 accent + inset accent-muted) を復活 (`inputDisabledFocusStyles.nestedRing`, `Field/inputDisabledFocus.ts`) |
| disabled + disabledMessage | 上記 + box への hover / keyboard focus で tooltip (16.5 / Tooltip 仕様) |
| InputGroup 内 | Field を描かず、label は visually hidden。box の角丸・枠の結合は InputGroup 6 節 (`groupStyles.inGroup`) に従う |

#### 16.4.4 ASCII sketch (md, 幅 320px、LTR)

```
Fruit                                              ← Field label (4 節)
┌────────────────────────────────────────────────┐
│ ⌕  Search fruits...                            │  32px, 枠 1px, r10, pad 3/8, gap 4
└────────────────────────────────────────────────┘
        ↓ 4px (offset)
╭────────────────────────────────────────────────╮ r12, shadow-low, bg popover, pad 4
│╭──────────────────────────────────────────────╮│
││ Apple                                        ││ 行 32px (pad 6/8), r10, highlight bg
│╰──────────────────────────────────────────────╯│
│  Banana                                        │
│  Cherry                                        │
╰────────────────────────────────────────────────╯

値選択後:
┌────────────────────────────────────────────────┐
│ ▐ Apple ▌                                (×)   │  Token 24px r6 / clear 20×20
└────────────────────────────────────────────────┘
検索中 (edit mode / 入力中):
┌────────────────────────────────────────────────┐
│ App|                                     ◌ (×) │  Spinner 14px, gap 4
└────────────────────────────────────────────────┘
```

#### 16.4.5 Popup surface と位置 (BaseTypeahead が描く; Typeahead / Tokenizer / 直接利用で共通)

- 部品: `usePopover({hasLightDismiss:false, hasCloseButton:false, hasAutoFocus:false, role:'none'})` (`BaseTypeahead.tsx:871-885`)。surface (`Popover/usePopover.tsx:79-88`):
  - bg `--color-background-popover` (#ffffff / #1b1b1b)
  - radius `--radius-container` **12px**
  - shadow `--shadow-low`: light `0 2px 4px #0000000D, 0 4px 8px #0000001A`; dark `0 2px 4px #00000040, 0 4px 8px #00000066, inset 0 0 0 1px #ffffff14` (dark だけ 1px の白 8% の縁)
  - **border なし** (layer base で `border 0`)、surface 自身の padding なし。
- layer root (`Layer/useLayer.tsx:62-150, 1840-1900`): margin / padding / border 0、bg transparent、overflow visible、text reset (Figtree 14/20 w400、text-align start)。
- 位置 (`:1852-1866`): placement **below**、alignment **start** (LTR は左端揃え、RTL は右端揃え)、anchor との距離 `--spacing-1` = **4px**。入らなければ above へ flip (CSS position-try-fallbacks; 標準 fallback 列)。viewport の各辺から gutter `max(16px, safe-area)` を保つ。最大 inline / block size = viewport − 両 gutter (16px×2)。
- anchor: Typeahead では control box 全体 (`anchorRef={wrapperRef}`)。BaseTypeahead 直接利用で anchorRef 無しなら input 自身。
- 幅 (`:329-339`): `min-width = min(anchor の幅, viewport − 32px)`。内容が長ければそれ以上に広がる (最大 viewport − 32px)。`menuWidth` (BaseTypeahead のみ) 指定時は `width = min(menuWidth px, 100vw − 32px)`。
- listbox div (`dropdown`, `:314-320`): `box-sizing:border-box; max-height:300px; overflow-y:auto; padding 4px`。行間 gap 0。
- **held height** (`:323-325, 649-680`): 同じ query で 2 回目以降の list (incremental) が来たとき、popup が field の **上** にあれば現在の高さで固定、**下** にあれば `min(300, max(現在高さ, 下の余白))` で固定して、追加行はスクロールで見せる (表示済み行が動かず、popup が反対側へ flip しない)。新しい list (置き換え) / 閉じる / 選択で解除。

#### 16.4.6 Option 行 (`BaseTypeahead.tsx:349-393, 418-429, 1785-1817`)

| 項目 | 値 |
|---|---|
| layout | `display:flex; align-items:center; width:100%; box-sizing:border-box; text-align:start`。中身 `itemContent` は `flex:1; min-width:0; overflow:hidden; display:flex`、その後ろに check。**行の gap は 0** (check は content の直後に接する) |
| padding | 基本 8px 全周。size 上書き: sm `4px 8px` / md `6px 8px` / lg `8px 8px` |
| 高さ (既定 TypeaheadItem、description なし) | sm 28px / md 32px / lg 36px (label 20px + padding)。description ありは +20px |
| radius | `--radius-element` 10px (popup 12px − padding 4px の入れ子) |
| bg rest | transparent |
| highlighted (keyboard または mouse move) | bg `--color-overlay-hover` → 合成 **#f2f2f2 / #272727**。transition なし (即時) |
| pressed | 専用 look なし (press overlay を持たない)。click で即選択 |
| selected (= value の行) | 行に `font-weight:500` + 末尾に `check` icon 16px `--color-icon-primary` (#000000 / #ffffff)。**既定 TypeaheadItem の label は自分で weight 400 を指定しているので太くならない** (太字が効くのは renderItem の中身が weight を継承する場合だけ)。`aria-selected=true` |
| selected + highlighted | 両方 (bg + check) |
| disabled option | engine には無い (全行選択可能)。TypeaheadItem の `isDisabled` は見た目だけ (16.x) |
| cursor | pointer |
| truncation | 既定 label / description は 1 行 ellipsis。長い label は popup 幅 (最大 viewport−32px) まで広がり、そこで ellipsis |
| focus | 行は tabIndex −1 (focus は input に残る)。行の focus ring なし |

#### 16.4.7 Group heading (`:340-348, 1829-1840`)

`padding-inline 8px; padding-top 8px; padding-bottom 4px`、12px / 20px / w400、`--color-text-secondary`、user-select none、`aria-hidden` (group 名は `role=group aria-label` で伝える)。高さ 32px。heading の前に区切り線や余白は無い (先頭 group も pt 8px)。

#### 16.4.8 Empty state (`:394-399, 1766-1776`)

- 表示条件: `results.length === 0 && hasSearched && !isSearchPending` (= 最新 query の検索が settle して 0 件。debounce 中・検索中は出さない)。
- look: padding 12px 全周、text-align center、12px / line-height 20px (layer の body leading を継承)、w400、`--color-text-secondary`。popup 全体の高さ = 4 + 44 + 4 = **52px**。
- `role=option aria-disabled=true`。中身は `emptySearchText` (既定 "No results found")。

### 16.5 Motion

| 対象 | trigger | property / from→to | duration / easing | reduced motion | file:line |
|---|---|---|---|---|---|
| control box 枠・影 | hover / focus-within / status 変化 | border-color, box-shadow | `--duration-fast` **125ms**, `--ease-standard` cubic-bezier(0.24, 1, 0.4, 1) | 0s (即時) | `Field/inputStyles.stylex.ts:59-64` |
| **popup 表示 / 非表示** | open / close | **アニメーション無し** (即時に出る / 消える)。BaseTypeahead は `layerAnimations` を付けず、usePopover も surface に animation を持たない | — | — | `BaseTypeahead.tsx:1853-1866`, `Popover/usePopover.tsx:79-88` (Selector 等が使う Layer enter 165ms を**付けないこと**) |
| option highlight | ArrowUp/Down, Home/End, mouse move | bg transparent ↔ overlay-hover | 即時 (transition 無し) | — | `BaseTypeahead.tsx:366-380` |
| keyboard 移動時のスクロール | highlight 変化 (keyboard / 新 list) | `scrollIntoView({block:'nearest'})` 即時 (smooth 無し)。mouse hover では**スクロールしない** | — | — | `hooks/useHighlightedOptionScroll.ts:67` |
| Spinner | busy 中 | stroke-dashoffset 0 → −π·d (1 周)、arc 0.375 | `--duration-slow-min` **525ms** / 1 周、linear、infinite | **3s** / 1 周 (止めない) | `Spinner/Spinner.tsx:338-352` |
| Token hover / press | pointer | background-image (overlay) | 125ms, ease-standard。touch: onset 150ms, quick-tap flash 100ms, release fade 200ms cubic-bezier(0, 0, 0.2, 1) | — | `Token/Token.tsx` `interactive`, `utils/interactionOverlay.stylex.ts` |
| clear ボタン | hover / press | background-image, transform scale 1 → 0.98 (mouse :active のみ) | 125ms, ease-standard | 0s | `Button/Button.tsx:83-140` |
| clear ボタン tooltip "Clear" | hover / focus | Tooltip 仕様: 表示遅延 200ms、Layer enter (opacity 0→1, translateY ∓8px→0, scale 0.95→1, **165ms**, ease-standard, fill backwards) | | fine pointer: 無し / coarse: fade ≤200ms | `Tooltip/useTooltip.tsx`, `Layer/layerAnimations.stylex.ts` |
| disabledMessage tooltip | box hover / focusin | 同上、placement **above** (下から上へ 8px: translateY(+8px)→0) | 同上 | 同上 | `Typeahead.tsx:464-470` |
| edit mode 開始 / 終了 | click / blur / Escape | Token の消滅・出現は即時 | — | — | `Typeahead.tsx:494-618` |
| 選択後の focus 移動 | 選択 | 次の animation frame で Token の button へ focus | 1 rAF | — | `:543-551` |

### 16.6 Behaviour & a11y

#### Keyboard (combobox input 上; BaseTypeahead `handleKeyDown` `:1473-1610` + Typeahead `:599-618`)

処理順: (1) BaseTypeahead の `onKeyDown` prop (Typeahead の Escape 処理。preventDefault すると built-in を飛ばす) → (2) IME composing 中なら何もしない → (3) built-in → (4) Typeahead の consumer `onKeyDown` (trailing、cancel 不可)。

| key | popup closed | popup open |
|---|---|---|
| 文字入力 / paste | query 更新 → debounce 後に検索 (16.6 search model) | 同左 |
| ArrowDown | `hasEntriesOnFocus` か query 非空なら開く: 現在の source の結果が残っていれば表示して先頭を highlight、source が差し替わっていて query があれば再検索、それ以外で hasEntriesOnFocus なら bootstrap (query が minQueryLength 未満なら何もしない) | 次の行 (末尾→先頭に wrap) |
| ArrowUp | 何もしない (既定動作) | 前の行 (先頭→末尾に wrap) |
| Home / End | caret 移動 (既定) | 先頭 / 末尾の行を highlight (preventDefault: caret は動かない) |
| Enter | 既定 (form submit 等) | highlight 行を選択。highlight 無しでも preventDefault (submit しない) |
| Escape | Typeahead edit mode 中: Token を復元し、次 frame で Token button へ focus | popup を閉じるだけ (edit mode は維持) |
| Tab | 既定の focus 移動 | popup を keydown 時点で閉じ、focus は次へ移る |
| disabled 時 | 何も開かない | Enter は preventDefault のみ、Escape / Tab は閉じる |

- **最初の行が自動で highlight** される (検索結果・bootstrap を出すたびに index 0。0 件なら −1)。Enter 1 回で先頭を選べる。
- 同じ query に後から行が追加された場合、highlight は同じ item (id) に残る。置き換えなら先頭。
- `aria-activedescendant` = highlight 行の id (open かつ highlight ≥0 のときだけ)。option id は item id 由来なので、行が後ろに足されても同じ option を指し続ける (SR が読み直さない)。

#### Pointer / touch

- option 上の **mouse move** で highlight (pointer が静止したまま list が出現・reflow しても highlight は動かない)。hover ではスクロールしない。click で選択。
- popup が開いている間: input / clear ボタン / Token (= anchor 内) と popup 内の press は「内側」、それ以外の press で閉じる (outside press、`useOutsidePress`)。スクロール開始 (pointercancel) では閉じない。pointer 押下中に開く要求が来たら click 後の rAF まで show を遅らせる (開いた press 自体で閉じないため)。
- 既に focus のある input を click → (閉じていれば) 再度開く (#6845)。focus を得た click は focus 処理だけが開く (bootstrap 二重発火なし)。
- control box (Token 以外の余白) の click → input に focus (値があれば edit mode)。Token の click → edit mode。
- clear ボタン: `onChange(null)`、edit mode で入っていた query を空にし (onChangeQuery('') も発火)、input に focus。押下で focus を奪わない。

#### Focus 管理

- popup を開いても focus は input のまま (`hasAutoFocus:false`, `role:'none'`)。
- 値表示中: input は tabIndex −1 (見えない tab stop を作らない)。Tab stop は Token の button (+ clear ボタン)。
- 選択後: query を空に、popup を閉じ、input を focus → 次 rAF で Token button へ focus (Typeahead)。
- edit mode (`:494-522`): value を editingValue に退避、`onChangeQuery(value.label)`、input に label を入れ (同期的に query state へ反映してから focus — iOS keyboard のため)、`focus()` + 全選択。label で検索が走る (hasEntriesOnFocus でも bootstrap ではなく label 検索)。
- blur (focus が control box の外へ): edit mode 中で未選択なら Token を復元 (onChange は呼ばない)。popup も focus が anchor / popup の外へ出たら閉じる。

#### Search model (async source; BaseTypeahead 側の挙動。Typeahead の見た目に直結)

1. 入力 → `onChangeQuery(q)`。前の debounce を破棄。
2. `q` が空で `!hasEntriesOnFocus`、または grapheme 数が `0 < len < minQueryLength`: 進行中の検索を無効化 + `cancel()`、結果を空に、busy 解除、announce クリア、popup を閉じる。「No results」は出さない。
3. それ以外: `debounceMs` (既定 150) 後に、q 非空なら `search(q, ctx)`、空なら `bootstrap()`。`debounceMs <= 0` なら即時。
4. `search`: 前の検索を `cancel()`、generation を進め、busy=true (Spinner + `aria-busy`)。
   - `ctx.update(items)` (settle 前、非空、同 generation のときだけ): その list を表示して popup を開く。2 回目以降は「表示済み行が先頭に同順で残る」(append-only) なら追記、違反なら dev warning を出して置き換え。
   - 解決値 = 最終 list。`maxMenuItems` で slice。popup は「結果あり または q 非空」で開く (→ 0 件なら empty state)。
   - 失敗: 表示済みの partial list があればそれで確定、無ければ 0 件 (empty state)。popup を閉じたことによる cancel の reject は無言 (announce しない)。
   - 古い generation の結果はすべて捨てる (out-of-order response、source 差し替え、選択、disabled 化)。
   - **新しい query の検索中は、前の query の行がそのまま表示され続ける** (最初の list が来た時点で置き換わる)。empty state は「最新 query が settle した」時だけ。
5. `bootstrap` (focus / click / ArrowDown / 空 query の debounce): 同期配列なら busy を出さずに即表示、Promise なら busy。0 件または失敗なら popup を閉じる (空の popup を出さない)。focus 時に既に現在 source の結果があれば bootstrap せず再表示。
6. 閉じる (Escape / Tab / outside / blur / 選択) と `cancel()` が呼ばれ、highlight −1、held height 解除。
7. `searchSource` の差し替え: 旧 source を `cancel()`、旧結果は再表示しない (focus で新 source を bootstrap)、busy 解除。

#### Role / name / state / announcements

- input: `role=combobox`, `aria-autocomplete=list`, `aria-expanded`, `aria-controls=<listbox id>`, `aria-activedescendant`, `aria-busy` (検索中), `aria-labelledby` (Field label [+ caller]), `aria-describedby` (description, status message, disabledMessage tooltip, caller), `aria-required` (Required 解決値), `aria-invalid` (status error のみ), disabled 時 `aria-disabled=true` + readOnly (tab stop 維持)。`autocomplete=off`、autocorrect off、enterkeyhint search (native では drop)。
- listbox: `role=listbox`, name "Search results"。option: `role=option`, `aria-selected` (= value の行)、name は行のテキスト。group: `role=group` + aria-label。empty: `role=option aria-disabled=true`。
- Spinner: `role=img` name "Loading" (live region ではない)。
- live announcements (polite, 共有 live region; modal 内ではその dialog の region):
  - 検索開始ごとに 1 回 "Loading" (1 rAF 後。結果が先に出たら取り消し) — `hooks/useBusyAnnouncement.ts`。
  - 非空 query の結果 N>0 件: "{count} result(s)" (`@astryx.typeahead.resultCount`)。focus 時の bootstrap は announce しない。
  - 0 件: empty state に**描画されたテキスト**を読む (aria-hidden 部分は除く。テキストが無ければ何も読まない)。
  - query が閾値未満 / 空に戻ったら announce をクリア。
- 既定テキスト (英語) と提案 i18n key:

| Astryx key | English | 提案 key |
|---|---|---|
| `@astryx.typeahead.searchPlaceholder` | Search… | `soft.typeahead.placeholder` |
| `@astryx.typeahead.emptySearchResults` | No results found | `soft.typeahead.empty` |
| `@astryx.typeahead.loading` | Loading | `soft.typeahead.loading` |
| `@astryx.typeahead.searchResults` | Search results | `soft.typeahead.results` |
| `@astryx.typeahead.clearSelection` | Clear selection | `soft.typeahead.clear_selection` |
| `@astryx.typeahead.resultCount` | {count} result / {count} results | `soft.typeahead.result_count_one` / `soft.typeahead.result_count_other` (kivact の `message_with` に plural が無いので 2 key) |
| `@astryx.inputClearButton.tooltip` | Clear | `soft.input.clear` (Field 4 節と共有) |

### 16.7 Kivact 実装方針

**方針: NEW kit component `soft::typeahead`** (kivact::ui に相当する標準部品が無い)。最も近い analog は `crates/kivact-ui-basic/src/combobox.rs` (input group + `use_combobox().popup(open)` + `overlay(anchor, Placement::Below, ..)`) と `list_popup.rs`。

1. 構成
   - Field shell: 4 節の `soft::field(label, control)` に委譲 (label / description / status / required / optional / label tooltip / width)。InputGroup 内では 6 節の文脈 (`try_context`) を見て field を描かず、label は control の `.name(..)` だけにする。
   - control box: `row` ノードに Field 共通 box style (4.4) + 差分 (min_height 28/32/36、padding_y 3、padding_x 8、gap 4、`text_wrap(false)`)。focus look は `NodeRef::focus_within()` を読んで切替 (basic の `chips_field` / `input_group::borderless` と同じ手法)。hover ring は `focus_within` が false の時だけ。
   - content lane: `node().style(style().position(Relative).grow(1).min_width(0).align_self(Stretch))` に、borderless の `kivact::ui::text_input` (native editor; `use_text_edit` 経由) と、値表示中の Token (`position(Absolute)`, top 0, left −3, right 0, `max_width(pct(100))`, 幅 fit-content) を重ねる。input は token 表示中 `opacity(0.0)` + `hit_testable(false)` + focus 順から外す (`focusable(false)` 相当。プログラムからの `NodeRef::focus()` は可能なこと要確認)。
   - end lane: `show(busy || clear, ..)` で row (gap 4) を作り、Spinner (kit の `soft::spinner(Small)`; Spinner 節の仕様) と clear ボタン (Field 4.4.6 の `soft` clear button) を置く。
   - popup: `show(open, || overlay(box_ref, Placement::Below, listbox).align(OverlayAlign::Start).offset(4).at_least_anchor_width(true).on_dismiss(..))`。surface style: `background(popover)`, `radius(12)`, shadow 2 枚 (`Shadow::new(0,2,4,#0000000D)`, `Shadow::new(0,4,8,#0000001A)`; dark は `#00000040`, `#00000066` + `Shadow::new(0,0,0,#ffffff14).spread(1).inset()`), border 無し。中に `scroll(column(rows))` (`max_height(300)`, padding 4, `max_height` は held height 時に可変)。**enter / exit transition を付けない** (Astryx は即時)。
2. 振る舞い (engine)
   - `kivact::interaction::use_combobox` は**そのままでは使わない**ことを推奨: (a) Home/End は caret 移動 (Astryx は先頭/末尾 highlight) で、active を外から設定する public API が無い (`step` / `choose` は private)、(b) option の `selected` semantics が active を表す (Astryx は `aria-selected` = value)、(c) pointer `Enter` でも active になる (Astryx は move のみ)、(d) active が変わると常に scroll into view (Astryx は hover でスクロールしない)、(e) options は同期 `Value<Vec<ListItem>>` で filter 前提。
   - 代わりに kit 内部の headless `TypeaheadEngine<T>` を public API だけで組む: `kivact::primitive::part(..)` で input part (`.controls(list_ref)`, `.active_descendant(..)`, `.expanded(open)`, `.on_key(..)`)、listbox (`Role::ListBox`, name `soft.typeahead.results`)、option (`Role::Option`, `.selected(is_value)`, `.on_pointer(Move のみで highlight)`, `.on_press(select)`)、`kivact::interaction::use_text_edit(query)` で IME 付き編集。basic `combobox.rs:170-215` (`popup`) と `:319-467` (`into_view` の input / popup 配線) が手本。use_combobox を使う場合は上記 (a)〜(d) を受け入れ差分として 16.8 に記録すること。
   - async: `kivact::task::{spawn, sleep}` で debounce と検索を実行。generation counter (`Cell<u64>` / `Signal<u64>`) で古い結果を捨てる (`kivact::task::resource` は latest-wins だが `context.update` の partial list と同期/非同期の区別を表せないので spawn を推奨)。`SearchContext::update` は generation を捕まえた sink。`cancel()` は新検索・close・source 差し替え・unmount・disabled 化で呼ぶ。
   - 状態: `query: Signal<String>`, `results: Signal<Vec<T>>`, `highlight: Signal<Option<String /*id*/>>` (id で保持すると append 時の維持が自然), `busy: Signal<bool>`, `has_searched`, `last_searched_query`, `open: Signal<bool>`, `held_height: Signal<Option<f32>>`。
   - held height: 2 回目以降の list 到着時に popup ノードの rect と anchor rect / window 高さを読んで `max_height` を固定 (`NodeRef` の layout rect 取得 API を使用。無ければ UNRESOLVED → 固定せず追記のみ)。
   - Typeahead 層: `editing: Signal<Option<T>>`、edit mode 開始で `query.set(label)` → focus → `select_all` (text edit の selection API。名称は `use_text_edit` の docs で確認)、blur (focus_within が false へ) で復元、Escape (popup closed 時) で復元 + 次 frame で Token focus (`kivact::view` の after-frame / `on_mount` で `NodeRef::focus()`)。
   - outside press: `overlay(..).on_dismiss` が anchor 上の press を outside と判定するか要確認 (basic combobox は anchor を input group にしている)。anchor (control box) 内の press で閉じてはいけない。
3. disabled: Astryx は「focus 可能な disabled」(tab stop 維持、readOnly)。Field 4.7 / TextInput 1.7 の方針に合わせ、`read_only(true)` + semantics disabled + opacity 0.5 + engine 側で open / select を拒否。keyboard focus-visible 時だけ focus look を戻す。
4. a11y: `aria-busy` に相当する semantics は kivact に無い (`kivact-types/src/semantics.rs` を確認) → 不可視の常駐 live region ノード (`live_region(LiveRegion::Polite)`) に "Loading" / 件数 / empty テキストを書く。Spinner は `Role::Image` + name "Loading"。
5. RTL: すべて logical (start/end)。kit の direction context に従い、start icon / Token / end lane / check の左右と popup の alignment (start = RTL で右端) を反転。
6. 落とすもの: iOS 16px font floor、`enterKeyHint`、autocorrect/autocomplete 属性、forced-colors outline、SSR (`item.element`, `utils.ts`)、coarse pointer の hit 拡大 (touch 対応時に追加)。
7. 難所: `fit-content` 幅で end 側 inset に bound された absolute Token (kivact で absolute + left/right 両指定時に intrinsic 幅になるか要確認。ならなければ Token に `max_width(lane 幅 + 3)` を計測で与える)、held height の計測、IME 中の Enter/Escape 無視 (`KeyEvent` に composing 判定があるか要確認; `use_text_edit` が IME を扱うので composing 中の key は届かない可能性が高い)。

### 16.8 受け入れ確認

- [ ] `role=combobox` name = field label ("Fruit")。focus しただけでは開かない (hasEntriesOnFocus=false)。
- [ ] "a" 入力 → 約 150ms 後に `role=listbox` name "Search results" が出て、先頭 option が active descendant。Enter でその値になり、Token "Apple" (button name "Fruit Apple") が表示され focus が Token button に移る。
- [ ] ArrowDown / ArrowUp が wrap、Home / End が先頭 / 末尾、Escape は popup だけ閉じる、Tab で閉じて次の control へ。
- [ ] 0 件: "No results found" (12px, secondary, 中央, popup 高さ 52px)。debounce 中・検索中には出ない。
- [ ] async source: 検索中に end lane に Spinner 14px (clear の左)、"Loading" が 1 回 polite で読まれる。結果到着で "3 results"。
- [ ] incremental (`context.update`): 先に出た行が動かず、後続行が下に足され、highlight が同じ item に残る。popup の高さが固定され追加分はスクロール。
- [ ] Token click / box click → edit mode (label が入り全選択、label で検索)。何も選ばず blur → Token 復元。clear → 値 null、input 空、focus は input。
- [ ] `hasEntriesOnFocus`: focus で bootstrap の 5 件。同期 source では Spinner が出ない。0 件 bootstrap は popup を出さない。
- [ ] `minQueryLength=3`: 1〜2 文字では検索もしない・開かない。
- [ ] disabled: opacity 0.5、Tab で focus でき (aria-disabled)、文字入力・開く・選ぶができない。disabledMessage は hover / keyboard focus で上に tooltip。
- [ ] status error: 枠 #c0262c / #ffaea7、focus 中も同色、inset ring なし、`invalid`。
- [ ] 目視 (docsite light/dark): box 32px r10 枠 #d4d4d4/#525252、popup r12 shadow-low (dark は白 8% 縁)、popup は 4px 下・左端揃え・**アニメーション無しで即時表示**、行 32px r10 highlight #f2f2f2/#272727、選択行に check、group heading 12px secondary。sm/lg で box 28/36、行 28/36、Token 20/28。
- [ ] 長い値: Token が clear ボタン手前で ellipsis、field 幅は値で変わらない。RTL で end lane が左、Token が右。

### 16.x TypeaheadItem

Source: `Typeahead/TypeaheadItem.tsx`, `TypeaheadItem.doc.mjs`。Story: `Typeahead.stories.tsx` "Parts / TypeaheadItem with icon"。

1. **概要**: popup の既定行内容。label + 任意の icon / avatar + description。`renderItem` から明示的に使って icon / description を足すための部品 (`renderItem={(item) => <TypeaheadItem item={item} icon={<Avatar/>} description={role}/>}`)。行の highlight / 選択 / click は外側の option wrapper (16.4.6) が持ち、TypeaheadItem は見た目だけ。
2. **Anatomy** (`:131-160`): `item.element` があればそれだけを返す。無ければ `div.astryx-typeahead-item (flex row)` > `[icon]` + `div content (flex column)` > `span label` + `[span description]`。
3. **API**:

| prop | 型 / 既定 | 意味 | Kivact 案 |
|---|---|---|---|
| `item` | `SearchableItem` (必須) | label を読む | `soft::typeahead_item(&item)` |
| `icon` | ReactNode | label 前の icon / avatar | `.icon(impl IntoView)` |
| `description` | string | label 下の補足 | `.description(text)` |
| `isDisabled` | bool / false | **見た目だけ** opacity 0.5 (選択は防がない) | `.disabled_look(bool)` |
| `group` | string | 受け取るが**未使用** (grouping は `auxiliaryData.group`) | drop |
| `xstyle/className/style/ref` | — | | NodeBuilder `.style(..)` / `.node_ref(..)` |

4. **Visual spec** (`:66-99`):
   - container: `display:flex; align-items:center; gap 8px; min-height 0`。option wrapper の中で `min-width:0; width:100%` (`defaultItem`)。
   - content: `flex column; flex:1; min-width:0`。
   - label: 14px / 20px / **w400** (`--text-label-size/leading` だが weight は normal に上書き) / `--color-text-primary` (#000000 / #ffffff)、1 行 ellipsis。
   - description: 12px / 20px / w400 / `--color-text-secondary` (#474747 / #9e9e9e)、1 行 ellipsis。
   - icon: caller の node をそのまま (story は 16×16)。縦中央揃え。
   - disabled look: container opacity 0.5。
   - 行の高さは option wrapper の padding と合算 (md: 6 + 20 [+20] + 6)。
5. **Motion**: なし。
6. **Behaviour & a11y**: role なし (option wrapper が `role=option`、name は中の text 全体 = label + description)。icon は caller が装飾扱いにする (`aria-hidden`)。
7. **Kivact 実装方針**: NEW kit part `soft::typeahead_item(&SearchItem<A>) -> TypeaheadItem` (row: icon, column(label, description))。`render_item` の既定として engine が使う。basic の `item.rs` (`item()` の media / title / description) が近い。
8. **受け入れ確認**: description 付き行が 44px (md) で 2 行 (14px 黒 / 12px 灰)。icon と content の間 8px。長い label / description が ellipsis。`isDisabled` は 0.5 で見えるが選択できる (Astryx 通り)。

## 17. BaseTypeahead

Source: `Typeahead/BaseTypeahead.tsx`, `BaseTypeahead.doc.mjs`。Docs: https://astryx.atmeta.com/components/BaseTypeahead (overview から隠し: `isHiddenFromOverview: true`、Typeahead の sub component)。Stories: `apps/storybook/stories/BaseTypeahead.stories.tsx` (Default, LogicalDropdownPlacement, Loading, EmptyResults, EmptyThenPending, CustomRenderer, DisabledStates, SizeVariants, NarrowLongResult)。

### 17.1 概要

- Typeahead と Tokenizer が内部で使う **combobox engine**: 素の `<input>`、検索 (async source)、keyboard navigation、**styled な結果 popup** だけを描く。input の枠・背景・padding・Token・Field (label / status) は**描かない**。
- 使う場面: Typeahead / Tokenizer では足りない完全 custom な field chrome を自前で組むときだけ。自前 wrapper を作り、`anchorRef` で popup をその wrapper に揃え、accessible name (visible label か aria-label) を必ず与える。
- 使わない場面: 標準 field なら Typeahead / Tokenizer。BaseTypeahead に input chrome や選択値表示を期待しない。Escape を「pending な source 処理の取消」とみなさない (popup を隠すだけで、遅れて来た応答が再度開くことがある)。

### 17.2 Anatomy

```
<input role=combobox>                      (styles.input; 枠なし)          BaseTypeahead.tsx:1668-1743
[span loadingStatus > Spinner sm]          (busy かつ wrapper が lane を持たない時)  :1744-1752
popup (16.4.5〜16.4.8 と同一)              (anchor = anchorRef ?? input)   :1753-1867
```

Typeahead との違いの要点: wrapper div / Field / Token / clear / end lane / edit mode / disabledMessage tooltip が無い。Spinner は input の**直後に inline** で並ぶ (wrapper の flex row の中で input の右)。

### 17.3 API (`BaseTypeahead.tsx:89-279`)

Typeahead と共通: `searchSource`, `value`, `onChange`, `renderItem`, `placeholder` (既定 'Search…'), `hasEntriesOnFocus`, `maxMenuItems` (10), `minQueryLength` (1), `emptySearchText`, `isDisabled`, `hasAutoFocus`, `onChangeQuery`, `onOpenChange`, `debounceMs` (150), `size` (`'md'`; **行 padding だけ**を変える)。

BaseTypeahead 固有:

| prop | 型 / 既定 | 意味 | Kivact 案 |
|---|---|---|---|
| `menuWidth` | number | popup の要求幅 px (viewport−32px で clamp)。無ければ anchor 幅を min-width | `.menu_width(f32)` |
| `isFocusableDisabled` | bool / false | disabled でも native disabled にせず aria-disabled + readOnly (tab stop 維持)。開く・選ぶ・入力を拒否、Escape/Tab は閉じる | engine 既定動作 (kit では disabled は常に focusable、16.7-3) |
| `anchorRef` | `RefObject<HTMLElement>` | popup の位置合わせ先。無ければ input | `.anchor(NodeRef)` |
| `inputXStyle` / `xstyle` / `className` / `style` | StyleX | input への追加 style (Typeahead は token 表示中の `inputHidden` に使う) | `.input_style(Style)` |
| `inputTabIndex` | number | input の tabIndex (Typeahead は token 中 −1) | `.input_focusable(impl IntoValue<bool>)` |
| `onKeyDown` | `(e) => void` | built-in **前**に呼ぶ。`preventDefault()` で built-in を skip | `.on_key_before(impl Fn(&KeyEvent) -> Handled)` (`Handled::Yes` で skip) |
| `inputId`, `ariaDescribedBy`, `ariaLabelledBy` | string | 旧 alias (native の id / aria-* より優先) | `.node_ref`, `.described_by`, `.labelled_by` |
| `__queryEntries` | `(query, results) => T[]` (internal) | query 文字列由来の追加行 (Tokenizer の "Create …")。結果の後ろ・全 group の後に付き、minQueryLength を無視、最終 list のときだけ付く | engine の crate 内 API (`pub(crate)`)。Tokenizer 節で使用 |
| `ref` | input ref | | `.node_ref(..)` |

### 17.4 Visual spec

- **input** (`:286-313`): 16.4.1 の input と同じ (`display:block; flex:1; min-width:60px; border 0; padding 0; bg transparent; outline none; Figtree 14/20 w400; color text-primary; placeholder text-secondary`)。focus ring も枠も描かない — caller の wrapper が描く (story の custom field: 枠 1px `--color-border-emphasized`、radius 10、min-height 32、padding-inline 8、`:has(input:focus-visible)` で outline 2px accent offset 3px)。disabled: cursor default のみ (減光は caller)。
- **loading status** (`:404-410`): `display:inline-flex; align-items:center; justify-content:center; flex-shrink:0; padding 4px` → Spinner sm (14px) を含めて **22×22px**。input の直後に in-flow (input の文字が下に潜らない)。Typeahead / Tokenizer 内では描かれない (busy lane へ委譲)。
- **popup / option / group heading / empty state**: 16.4.5〜16.4.8 と完全に同じ。`size` は option padding (sm 4/8, md 6/8, lg 8/8) にだけ効く。`menuWidth` 指定時は `width:min(menuWidth, 100vw−32px)`。
- 状態: input 自身は見た目の状態を持たない (placeholder 色だけ)。disabled (`isFocusableDisabled` なし) は native disabled。

### 17.5 Motion

16.5 のうち engine 部分と同じ: popup は**アニメーション無し**、highlight 即時、keyboard スクロール `block:nearest` 即時、Spinner 525ms/周 linear (reduced 3s)。枠の transition は無い (caller 次第)。

### 17.6 Behaviour & a11y

- 16.6 の Keyboard / Pointer / Search model / Role・announce がすべて BaseTypeahead の挙動 (Typeahead 固有の edit mode・Token・Escape 復元・選択後 Token focus を除く)。
- 選択後は query を空にし popup を閉じ、input に focus を戻す (BaseTypeahead 単体では選択値は input に表示されない。caller が value を描く)。
- busy announce: wrapper が lane を持たない時は BaseTypeahead 自身が "Loading" を 1 回 announce し、自分の Spinner を描く。
- accessible name は caller の責任 (`aria-label` / `aria-labelledby`)。
- committed choices (Tokenizer): key が変わったら確定済み id の行を即座に除き、開いていれば同じ query を再検索 / focus list を再 bootstrap、上限到達なら閉じる (`:1188-1263`)。
- 未対応: `SearchableItem.action` は描かず dev warning (`:580-589`)。

### 17.7 Kivact 実装方針

**方針: NEW kit 内部部品 (`soft::typeahead` の engine)**。16.7-2 の `TypeaheadEngine<T>` をそのまま BaseTypeahead に当てる。公開するかは任意: 公開するなら `soft::base_typeahead(source, value)` (`.anchor(NodeRef)`, `.menu_width`, `.on_key_before`, `.input_style`, `.input_focusable`, `.size`, ほか 17.3) とし、input は borderless `kivact::ui::text_input`、loading は input 直後の 22px box (padding 4 + Spinner 14)。Typeahead / Tokenizer は同 engine を使い、busy は engine の `busy: Value<bool>` を end lane が読む (busyIndicatorLane の代替)。closest analog: `crates/kivact-ui-basic/src/combobox.rs` (`popup(..)` 関数) と `list_popup.rs`。

### 17.8 受け入れ確認

- [ ] caller 自前の wrapper + `.anchor(wrapper)` で popup が wrapper 幅以上・4px 下・start 揃え。anchor 無しなら input に揃う。
- [ ] LogicalDropdownPlacement story (RTL audit 対象, `menuWidth=320`, 狭い field): LTR で anchor 左端、RTL で右端に揃い、viewport 端から 16px 以上離れる。
- [ ] async source で input 右に 22px の loading box (Spinner 14px) が in-flow で出て、input 文字と重ならない。
- [ ] `.on_key_before` が `Handled::Yes` を返すと ArrowDown / Enter の built-in が動かない。
- [ ] `menu_width(480)` で popup 幅 480px (狭い window では window−32px)。
- [ ] NarrowLongResult: popup が viewport 両端から 16px 以内に収まり、listbox / 各 option に横スクロールが無い (長い label は ellipsis)。
- [ ] EmptyThenPending: 0 件表示の後に新しい query の検索中は empty state が消え、settle まで出ない。


---

## 18. Tokenizer

Source: `packages/core/src/Tokenizer/` (`Tokenizer.tsx`, `TokenizerBottomSheet.tsx`, `useEndLaneReserve.ts`, `TokenizerClearAllMessageContext.ts`), chip = `packages/core/src/Token/Token.tsx`, 検索エンジン = `packages/core/src/Typeahead/BaseTypeahead.tsx`, 共通枠 = `packages/core/src/Field/inputStyles.stylex.ts`。Docs: https://astryx.atmeta.com/components/Tokenizer 。Stories: `apps/storybook/stories/Tokenizer.stories.tsx`, `MobileTokenizer.stories.tsx`。

### 18.1 概要

- 検索可能なデータソース (`searchSource`) から複数の項目を選び、選んだものを入力欄の中に取り外し可能な chip (Token) として並べる multi-select typeahead。チームメンバー追加、タグ付け、フィルタ選択などに使う。
- 推奨 (Tokenizer.doc.mjs:306-345): 何を検索できるか分かる placeholder ("Search people...")、上限があるなら `maxEntries`、ソースに無い値を作らせるなら `hasCreate`、検証は `status` で即時に示す。
- 非推奨: 単一選択 (→ Typeahead)、Tokenizer 内の個別 token に独自色を付ける、ラベルを隠す (文脈で明らかな時のみ `isLabelHidden`)、disabled Tokenizer を Tooltip で包む (→ `disabledMessage`)。
- 「ペースト分割」は Astryx に**存在しない**: ペーストは入力と同じく 1 つのクエリとして検索される (Tokenizer.test.tsx:1549-1595 がそれを固定)。区切り文字で複数 token に分割する処理は無い。Kivact もこれに合わせる (分割を実装しない)。

### 18.2 Anatomy

```
Field (label / description / status — Field 共通)
└─ div role="group" aria-label={label}  … control box (inputWrapperStyles.base + styles.wrapper)
   ├─ [startIcon] span (tokens 有り時 marginInlineStart 5px)
   ├─ Token × N           … value の各項目 (renderToken があれば span.token で包んだ任意要素)
   │   ├─ span.label (ellipsis)
   │   └─ button aria-label="Remove {label}" (× glyph 12px)
   ├─ input role="combobox" (BaseTypeahead)  … tokens の直後に in-flow で続く
   ├─ input[type=hidden] name={htmlName} × N (form 用; native では不要)
   └─ EndLane (absolute, inline-end, 1 行目に固定)
       ├─ Spinner sm (検索中 / tokens 有り時は idle でも不可視で常駐)
       ├─ endContent
       └─ InputClearButton (hasClear && tokens>0 && !disabled) aria-label "Clear all", tooltip "Clear"
Popover (top layer, BaseTypeahead) role="listbox" aria-label "Search results"
   ├─ [group role="group" aria-label={heading}] + 見出し (aria-hidden)
   ├─ option role="option" × M (TypeaheadItem or renderItem)
   ├─ option "Create "{value}"" (hasCreate; 常に末尾)
   └─ empty: role="option" aria-disabled="true" "No results found"
```

- `tokenOverflowBehavior="unfocusedInline"`: blur 中は tokens を `OverflowList` に入れ 1 行 + `+N more` (Tokenizer.tsx:1423-1436)。
- `tokenOverflowBehavior="unfocusedLayer"`: 場所取り用 placeholder div (高さ固定) + focus 時に top layer popover の中へ上記 wrapper 全体を出す (Tokenizer.tsx:1723-1787)。
- 内部専用 `presentation: 'bottom-sheet' | 'adaptive'` (18.9 参照): wrapper の代わりに `button` trigger (tokens 表示のみ) + `TokenizerBottomSheet`。

### 18.3 API

Astryx props (Tokenizer.tsx:180-337) → Kivact 提案。kit 関数は `soft::tokenizer`。検索ソース型 `SearchSource<K>` / 項目型 `SearchItem<K>` は Typeahead 節と**共有**する (同一型にすること)。

| Astryx prop | 型 / default | 意味 | Kivact |
|---|---|---|---|
| `label` | string (必須) | Field ラベル、group と combobox の名前 | `tokenizer(label, source, value)` 第 1 引数 |
| `searchSource` | `SearchSource<T>` (必須) | `search(query, ctx)` / `bootstrap()`; `ctx.update(items)` で途中結果 | 第 2 引数 `impl SearchSource<K>` (同期 `Vec` か `Future` を返す) |
| `value` + `onChange(items, change)` | `T[]` (必須) | 制御値。change = `add`/`create`/`remove`/`reorder` | 第 3 引数 `impl Into<Binding<Vec<SearchItem<K>>>>` (caller-owned)。メタ情報は `.on_change(Fn(&TokenizerChange<K>))` |
| `placeholder` | string | tokens 0 個の時だけ表示 | `.placeholder(text)` |
| `maxEntries` | number | 上限。到達で入力を視覚的に隠す | `.max_entries(usize)` |
| `hasClear` | false | Clear all ボタン | `.clear_all(bool)` |
| `hasCreate` | false | 自由入力から token 作成 ("Create "x"") | `.creatable(bool)` |
| `hasEntriesOnFocus` | false | focus だけで bootstrap を開く | `.entries_on_focus(bool)` |
| `maxMenuItems` | 10 | 候補最大数 (create 行は別枠で上乗せ) | `.max_menu_items(usize)` |
| `menuWidth` | number | dropdown 固定幅 (入力幅未満にはならない) | `.menu_width(f32)` |
| `minQueryLength` | 1 | これ未満は検索しない (create 行は出る) | `.min_query_length(usize)` |
| `emptySearchText` | ReactNode, default "No results found" | 0 件時の内容 | `.empty(impl FnOnce() -> V)` slot、既定は文字列 |
| `debounceMs` | 150 | 入力→検索の遅延 | `.debounce(Duration)` |
| `isDisabled` | false | 無効。tab stop は残る (aria-disabled + readOnly) | `NodeBuilder::disabled(..)` |
| `disabledMessage` | string | disabled 理由 tooltip (above) | `.disabled_message(text)` |
| `status` / `statusVariant` | `{type, message}` / `'attached'` | 検証表示 | `.status(FieldStatus)` / `.status_placement(StatusPlacement)` (Field 共通) |
| `description`, `isRequired`, `isOptional`, `labelTooltip`, `isLabelHidden`, `width` | | Field 共通 | 同名 snake_case (Field 節と統一) |
| `startIcon` | ReactNode / IconType | 先頭アイコン (16px secondary) | `.start_icon(impl IntoView)` |
| `endContent` | ReactNode | end lane に置く任意内容 | `.end_content(impl FnOnce() -> V)` |
| `size` | `'md'` (sm/md/lg) | 高さ 28/32/36 | `.size(ControlSize)` |
| `tokenOverflowBehavior` | `'none'` | `none`/`unfocusedInline`/`unfocusedLayer` | `.overflow(TokenOverflow::{Wrap, CollapseInline, CollapseLayer})` |
| `renderToken(item, onRemove)` | | chip の差し替え | `.token(Fn(SearchItem<K>, RemoveFn) -> View)` slot |
| `renderItem(item)` | | 候補行の差し替え | `.item(Fn(SearchItem<K>) -> View)` slot |
| `onChangeQuery` | | クエリ変更通知 | `.query(Binding<String>)` (caller-owned query) |
| `onFocus` / `onBlur` | | field 全体への出入り時のみ | `.on_focus_enter(Fn)` / `.on_focus_leave(Fn)` |
| `onKeyDown` | | 組込み処理の**後**に呼ぶ (取消不可) | `.on_key(Fn(&KeyEvent))` (Handled を返さない観察専用) |
| `hasAutoFocus` | false | mount 時 focus | `.auto_focus(bool)` |
| `handleRef` (`focus()`/`blur()`) | | 命令的 focus | `.node_ref(NodeRef)` (`NodeRef::focus()` で代替) |
| `htmlName` | | hidden input で form 送信 | **drop** (native に form 送信なし。値は Binding で読む) |
| `ref`, `id`, `aria-*`, `data-*` | | input へ転送 | `id`→`test_id`、`aria-labelledby/describedby`→`.labelled_by/.described_by` |
| `containerXstyle/ClassName/Style`, `controlXstyle/ClassName/Style` | | 見た目上書き | **drop** → `.style(..)` (NodeBuilder) と recipe |
| `experimental.presentation` (内部専用) | `'popover'` | `bottom-sheet`/`adaptive` | `.presentation(TokenizerPresentation::{Popover, BottomSheet, Adaptive})` (26.6 参照) |

`TokenizerChange<K>`: `enum { Add(SearchItem<K>), Create(SearchItem<K>), Remove(SearchItem<K>), Reorder }` (`Reorder` は Astryx 型にあるが発火元が無い。定義だけ残す)。Clear all は `Remove(最後の項目)` を 1 回だけ送り値を `[]` にする (Tokenizer.tsx:1291-1294 の慣習)。

### 18.4 Visual spec

#### 18.4.1 control box (wrapper)

Field 共通の input container spec (section 4 参照) を `inputWrapperStyles.base` (inputStyles.stylex.ts:32-73) そのまま使う。値の再掲:

- box-sizing border-box, display flex, align-items center, **flex-wrap wrap**, gap `--spacing-1` 4px (Tokenizer.tsx:374-383 が base の gap 8px を上書き), height auto, cursor text (disabled は default)。
- border 1px solid `--color-border-emphasized` (light #d4d4d4 / dark #525252)、radius `--radius-element` 10px、背景 `--color-background-surface` (light #ffffff / dark #262626)、outline none。
- 高さ: `min-height` = `--size-element-sm/md/lg` = 28 / 32 / 36px (Tokenizer.tsx:542-546)。
- padding: tokens 0 個 → block 4px / inline 8px (base)。tokens 1 個以上 (`wrapperWithTokens`, Tokenizer.tsx:384-397) → block 3px / inline-start 3px / inline-end `3px + end-lane reserve` / **row-gap 3px**。3px = radius-element 10 − radius-inner 6 − border 1 で token と枠を同心円にするための値 (`TOKEN_ROW_INSET = calc(--spacing-1 − 1px)`)。column gap は 4px のまま。
- 高さ計算 (md): 1+3+24(token)+3+1 = 32px。2 行: 1+3+24+3+24+3+1 = 59px。sm: 1+3+20+3+1=28、lg: 1+3+28+3+1=36。
- start icon: `renderIconSlot(startIcon, {size:'sm', color:'secondary'})` = 16×16、`--color-icon-secondary` (light #6a6a6a / dark #9e9e9e)。tokens 有り時は `marginInlineStart: calc(8px − 4px + 1px)` = 5px (Tokenizer.tsx:398-404) で文字の開始位置を 8px に戻す。

```
tokens 0:  ┌────────────────────────────────────────┐  h32
           │ 8px [🔍] Search people...         [⟳][×]│
           └────────────────────────────────────────┘
tokens 2+: ┌────────────────────────────────────────┐
           │3[Alice ×]4[Bob ×]4 |caret       [⟳][×]8│ ← lane は 1 行目中央に固定
           │3[Charlie Davidson ×]                    │ ← row-gap 3px、lane の幅分右を空ける
           └────────────────────────────────────────┘
```

#### 18.4.2 状態

| 状態 | 変化 (light / dark) |
|---|---|
| rest | 上記 |
| hover (pointer が hover 可能、focus-within でない、disabled でない) | `box-shadow: inset 0 0 0 2px color-mix(in srgb, --color-border-emphasized 30%, transparent)` = inset 2px `#d4d4d44D` / `#5252524D` (inputStyles.stylex.ts:64-69) |
| focus-within (input に focus) | border `--color-accent` (#1b1b1b / #f1f1f1) + `inset 0 0 0 2px --color-accent-muted` (#f1f1f1 / #262626)。dark では accent-muted = surface (#262626) なので内側リングは事実上見えない (これが正) |
| disabled | `opacity: 0.5`、border `--color-border-emphasized`、box-shadow none、cursor default (inputStyles.stylex.ts:74-80)。input が `aria-disabled` で keyboard focus 中 (`:has(input[aria-disabled=true]:focus-visible)`) のみ border accent + inset 2px accent-muted を復活 (inputDisabledFocus.ts:36-50)。tokens は `isDisabled` → Token 自身も opacity 0.5 (合成で約 0.25 に見える) かつ × ボタン無し、Clear ボタン非表示 |
| status=error | border `--color-border-error` = `--color-text-error` (#c0262c / #ffaea7)。hover: `--shadow-inset-error` = inset 2px `#de47454D` (light/dark 共通)。**focus-within でも border は error 色のまま、inset accent-muted リングは出ない** (status の boxShadow 定義が base の boxShadow 全体を置き換えるため; inputStyles.stylex.ts:123-151) |
| status=warning | border #836700 / #eec448、hover inset 2px `#f8d36a4D` |
| status=success | border #2f7d33 / #90ca90、hover inset 2px `#2f7d334D` |
| maxEntries 到達 | input を `width 0 / min-width 0 / flex 0 0 0 / padding 0 / opacity 0 / position absolute` (Tokenizer.tsx:434-441)。見えないが focus 可能 |
| loading | end lane に Spinner sm (18.4.5) |
| readOnly | Tokenizer 自体に readOnly prop は無い (disabled のみ) |

transition: `border-color, box-shadow` `--duration-fast` 125ms `--ease-standard` cubic-bezier(0.24, 1, 0.4, 1)、reduced motion で 0s (inputStyles.stylex.ts:58-63)。

#### 18.4.3 inline input

BaseTypeahead.tsx:286-311: border 0、padding 0、font `--font-family-body` (Figtree)、`--text-body-size` 14px / leading 20px、color `--color-text-primary` (#000000 / #ffffff)、背景 transparent、placeholder `--color-text-secondary` (#474747 / #9e9e9e)。iOS の coarse pointer のみ font-size max(16px, body)。

- tokens 0 個: `flex: 1; min-width: 60px`、inline-end padding = end-lane reserve (`useEndLaneReserve`)。
- tokens 有り (`inputCompact`, Tokenizer.tsx:442-473): placeholder は `' '` (空白 1 文字; 描画しないが「空」判定に使う)。
  - 入力文字あり: `flex: 1 1 40px; min-width: 40px; padding-inline-start: 5px` → 40px 入らなければ次の行へ折り返す。
  - 空かつ focus 中: `flex: 1 1 0; min-width: 0; padding-inline-start: 5px` → caret が最後の token の 5px 右に立つ。行を増やさない。
  - 空かつ blur (または readOnly): `min-width 0; padding-start 0; margin-inline-start: -4px` (column gap を打ち消す) → 幅 0 で余分な行を作らない。
- caret 色は text-primary (UNRESOLVED: Astryx は caret-color 未指定 → ブラウザ既定 = currentColor)。

#### 18.4.4 Token chip (Tokenizer が使う仕様)

Token 本体の節は別にあるが、Tokenizer が既定で描く `Token label size onRemove isDisabled` の完全仕様 (Token.tsx:123-266, 325-339):

| 項目 | sm | md | lg |
|---|---|---|---|
| height | 20px (`calc(28px − 8px)`) | 24px | 28px |
| padding-inline | 8px | 8px | 8px |
| font | `--text-supporting-size` 12px / leading 20px / weight `--font-weight-medium` 500 | 同 | 同 |

- display inline-flex、align center、gap 4px、padding-block 0、border なし、radius `--radius-inner` 6px、white-space nowrap、max-width 100%、overflow hidden。label span は ellipsis + `min-width:0` (長い label は行幅で省略)。
- 色 (default color): 背景 `--color-neutral` (light `#0000000F` / dark `#ffffff1A`; surface 上の合成値 light #f0f0f0 / dark #3c3c3c)、文字 `--color-text-primary` (#000000 / #ffffff)。Tokenizer 内では色指定しないこと (bestPractices)。
- remove button: 16×16、radius full、padding 0、`margin-inline-end: -4px` (右端の見た目余白は 4px)、color inherit、中身 `Icon close` size `xsm` 12×12 (neutral は lucide `X`, stroke 2, currentColor)。hit 領域は `::after inset -4px` で 24×24。**hover / pressed の見た目変化なし** (overlay 未適用)。focus-visible: outline 2px solid `--color-accent` offset 3px (focusOutlineProps.focusVisible)。
- Tokenizer 側 wrapper style `styles.token` (Tokenizer.tsx:405-413): `display flex; flex-shrink 0; max-width 100%; position relative; z-index 1`。
- disabled: opacity 0.5、pointer-events none。Tokenizer の disabled では × を出さない。
- 追加・削除のアニメーションは**無い** (即時に出入り)。

```
md: ┌8px─ Alice ─4─[×]─4┐  h24, radius 6
    └────────────────────┘
```

#### 18.4.5 end lane

Tokenizer.tsx:414-425, 548-561, 680-724:
- `position: absolute; inset-inline-end: 8px; top: calc(size/2 − 1px)` (sm 13 / md 15 / lg 17px、padding box 基準 = border box 上端から size/2) + `translateY(-50%)` → **常に 1 行目の縦中央**。display flex、align center、gap 8px、z-index 1。
- 並び: Spinner → endContent → Clear ボタン。
- reserve (`useEndLaneReserve`, 18.10 参照): lane の未変形 border-box 幅 `w` を測り `--_tokenizer-end-lane-reserve = calc(8px + w px)` を wrapper に書く。tokens 有り: wrapper `padding-inline-end = 3px + reserve` (全行が lane の手前で折り返す)。tokens 無し: input の `padding-inline-end = reserve`。lane が空なら変数削除 → 0。
- Spinner: `size="sm"`、直径 10px + stroke 2px → 箱 14×14、arc 色 `--color-accent` (#1b1b1b / #f1f1f1)、track `--color-track` (#CCD3DB / #5A5E66)、arc 長 = 円周 × 0.375。tokens 有り・enabled・上限未満なら検索していなくても `visibility:hidden` で常駐し幅を確保 (検索のたびに折り返しが変わらないため)。
- Clear ボタン (InputClearButton.tsx:32-144): `Button variant="ghost" size="sm" isIconOnly`、高さ上書き **20px**・aspect 1/1 → 20×20、radius `--radius-element` 10px、背景 transparent、icon `close` size sm 16px color secondary (#6a6a6a / #9e9e9e)。hover (hover:hover): `--color-overlay-hover` (#0000000D / #ffffff0D)、mouse press: `--color-overlay-pressed` (#0000001A / #ffffff1A) + `scale(0.98)`。coarse pointer のみ hit 領域 24×24 (`::after inset -2px`)。tooltip "Clear" (Tooltip 共通 spec)。pointerdown / mousedown を preventDefault して input の focus を奪わない。

#### 18.4.6 dropdown (BaseTypeahead popover)

Typeahead 節と共通。要点 (BaseTypeahead.tsx:313-430, Popover/usePopover.tsx:79-88):
- top layer、placement below / alignment start / offset 4px、anchor = **control box 全体** (`anchorRef={wrapperRef}`)、min-width = anchor 幅 (viewport cap 内)、`menuWidth` 指定時はその幅 (anchor 幅未満にしない)。
- surface: 背景 `--color-background-popover` (#ffffff / #1b1b1b)、radius `--radius-container` 12px、shadow `--shadow-low` (light `0 2px 4px rgba(0,0,0,.05), 0 4px 8px rgba(0,0,0,.10)` / dark `0 2px 4px rgba(0,0,0,.25), 0 4px 8px rgba(0,0,0,.40), inset 0 0 0 1px rgba(255,255,255,.08)`)。
- list: max-height 300px、overflow-y auto、padding 4px。
- option: flex、align center、width 100%、padding 8px、radius 10px、size 別 padding-block sm 4 / md 6 / lg 8px (sm は inline 8px)。highlight 背景 `--color-overlay-hover` (#0000000D / #ffffff0D)。label `--text-label-size` 14/20 weight 400 text-primary、description 12/20 text-secondary、どちらも 1 行 ellipsis。
- group 見出し: padding 8px 8px 4px、12/20、text-secondary。
- empty: padding 12px、center、12px、text-secondary、"No results found"。
- Create 行: label `Create "{value}"` (`@astryx.tokenizer.create`)、常に最後 (グループの後)。

#### 18.4.7 overflow モード

- `unfocusedInline` (blur 中 かつ tokens 有り): wrapper `height = size` 固定 (28/32/36)、`flex-wrap: nowrap; overflow: hidden`。tokens を OverflowList (gap 4px, 親幅観測) に入れ、入らない分を `+{n} more` テキスト (12px supporting、`--color-text-secondary`、padding-inline 4px、nowrap、shrink 0) に置換 (Tokenizer.tsx:474-490)。input は `inputAtMax` で隠す。focus で通常の折り返し表示に戻る (同じ場所で縦に伸びる)。
- `unfocusedLayer`: 場所取り div は常に `height = size` 固定 (レイアウトは動かない)。blur 中は OverflowList 表示。focus 時は top layer に完全な wrapper を出す: `width: anchor-size(width)`、`top: anchor(top)`、`left: anchor(start)` で場所取り div の真上に重ね、下方向に伸びる (他コンテンツの上に被さる)。popover surface は無く wrapper 自身の枠・背景で描く。**RTL では physical `left` のため鏡像化されない (Astryx の既知ギャップ)** → Kivact では logical に正しく鏡像化する。
- いずれもアニメーション無し。

#### 18.4.8 レイアウト / RTL

- 幅: Field の `width` (既定は親幅いっぱい)。token は行幅で ellipsis。
- RTL: すべて logical (padding-inline、inset-inline-end、margin-inline) → 自動鏡像化。lane は inline-end (RTL では左)。

### 18.5 Motion

| トリガ | 対象 | from→to | duration / easing | reduced motion | 出典 |
|---|---|---|---|---|---|
| hover / focus-within / status | wrapper border-color, box-shadow | 18.4.2 | 125ms cubic-bezier(0.24, 1, 0.4, 1) | 0s | inputStyles.stylex.ts:58-63 |
| Clear ボタン hover/press | background-image, transform | overlay、scale 1→0.98 | 125ms cubic-bezier(0.24, 1, 0.4, 1); touch は press 150ms 遅延 / quick tap 100ms / release 200ms cubic-bezier(0,0,0.2,1) | 0s | Button.tsx:118-140 |
| 検索中 | Spinner arc | stroke-dashoffset 0 → −π·d (1 周) | `--duration-slow-min` 525ms linear 無限 | 3s/周 (止めない) | Spinner.tsx:228-231, 339-350 |
| dropdown 開閉 | — | 即時 (entry animation 無し) | — | — | BaseTypeahead.tsx:1820-1838 |
| token 追加/削除、overflow 展開 | — | 即時 | — | — | — |
| disabledMessage tooltip | Tooltip 共通 (show delay 200ms、enter = Layer enter 165ms) | | | | useTooltip |

### 18.6 Behaviour & a11y

キーボード (input 上; BaseTypeahead.tsx:1471-1600 + Tokenizer.tsx:1305-1318):

| キー | menu 閉 | menu 開 |
|---|---|---|
| 文字入力 / ペースト | クエリ更新 → `debounceMs` (150ms) 後に検索。`minQueryLength` 未満は検索せず menu 閉 (hasCreate なら Create 行だけ出る) | 同 |
| ArrowDown | `hasEntriesOnFocus` かクエリ有りなら開いて先頭を highlight | 次へ (末尾→先頭に wrap) |
| ArrowUp | — | 前へ (先頭→末尾に wrap) |
| Home / End | caret 移動 | 先頭 / 末尾を highlight |
| Enter | 何もしない | highlight を確定 → token 追加、クエリ消去、menu 閉、input に focus |
| Escape | — | menu を閉じる |
| Tab | 通常移動 | menu を閉じてから通常移動 |
| Backspace (クエリ空、tokens 有り) | 最後の token を削除 (`preventDefault`)、"Removed {label}" を polite 通知 | 同 |
| IME 変換中 | すべて無視 (確定 Enter で選択しない) | 同 |

- 呼び出し側 `onKeyDown` は組込み処理の後に呼ばれ、input 以外 (× ボタン、Clear) のキーも受ける。
- pointer: wrapper のどこを押しても input に focus (Tokenizer.tsx:1321-1334)。option は mouse **移動**で highlight (静止ポインタ上に menu が開いても highlight は動かない)、click で確定。外側 press / blur で menu 閉。
- focus 順: DOM 順 = [token1 ×] … [tokenN ×] [input] [Clear]。**外から focus が入った時 (Tab/Shift+Tab/クリック) は常に input へ転送** (Tokenizer.tsx:1060-1080) → 外から tokens の × を順に Tab しなくてよい。field 内で Shift+Tab すると × ボタンへ戻れる。`onFocus`/`onBlur` は field 全体への出入りでのみ発火。
- × / Clear 後は input に focus を戻す。Clear は値を `[]` にして "All items removed" (PowerSearch 内では "All filters removed" — `TokenizerClearAllMessageContext`) を通知。
- 選択済み id は候補から除外。Create は「選択済み id と同一」「選択済み label と大小無視で同一」「結果 label と同一」の場合出さない。作成時の項目は `{id: 値, label: 値}`、change `create`。
- `maxEntries` 到達: 検索ソースを空に差し替え、`hasEntriesOnFocus` 無効、開いている list を閉じる。input は不可視だが focus 可能 (Backspace で減らせる)。
- disabled: input は `aria-disabled="true"` + readOnly で tab stop を保持 (`unfocusedLayer` かつ `disabledMessage` 無しの時のみ native disabled で tab 外)。入力・開閉・選択・Backspace 削除すべて無効。`disabledMessage` 有りなら wrapper に tooltip (placement above、hover と keyboard focus で表示)、input の aria-describedby に紐付け。
- semantics: wrapper `role=group` name=label。input `role=combobox`, `aria-expanded`, `aria-controls`=listbox, `aria-activedescendant`=highlight option, `aria-autocomplete=list`, `aria-busy` (検索中), `aria-invalid` (status=error のみ), `aria-required`, `aria-describedby` = description + status message + disabled 理由。listbox name "Search results"。option `aria-selected`。Token の × は `button` name "Remove {label}"。Clear は name "Clear all"、tooltip "Clear"。Spinner は `role=img` name "Loading" (live ではない)。
- 通知 (polite live region): "Added {label}" / "Removed {label}" / "All items removed" / 検索開始時 "Loading" / 結果件数 "{count} results" (Typeahead 共通) / 0 件時は表示した empty 内容のテキスト。mount 時と入力中は token 通知しない。
- touch keyboard: `enterkeyhint=search`、autocorrect/autocapitalize off。

i18n keys (英語既定):

| key | English |
|---|---|
| `soft.tokenizer.create` | Create "{value}" |
| `soft.tokenizer.token_added` | Added {label} |
| `soft.tokenizer.token_removed` | Removed {label} |
| `soft.tokenizer.all_removed` | All items removed |
| `soft.tokenizer.clear_all` | Clear all |
| `soft.tokenizer.more` | +{count} more (Astryx は直書き英語; key 化する) |
| `soft.token.remove` | Remove {label} |
| `soft.input_clear.tooltip` | Clear |
| `soft.typeahead.empty` | No results found |
| `soft.typeahead.loading` | Loading |
| `soft.typeahead.results` | Search results |
| `soft.typeahead.result_count` | {count} result / {count} results (plural) |
| sheet 用 (18.9) | `soft.tokenizer.search` "Search {label}", `soft.tokenizer.search_placeholder` "Search…", `soft.tokenizer.no_results_description` "Try a different search.", `soft.tokenizer.add` "Add", `soft.tokenizer.add_value` "Add {value}", `soft.tokenizer.done` "Done", `soft.tokenizer.clear_all_title` "Clear all selected items?", `soft.tokenizer.clear_all_description` "This removes all selected items from {label}.", `soft.tokenizer.clear_selection` "Clear selection", `soft.tokenizer.nothing_to_clear` "Nothing selected", `soft.tokenizer.selected_count` "{count} selected" |

(`@astryx.tokenizer.save` / `selectionLimit` / `discardChanges*` / `keepEditing` は catalog にあるがコードから参照されていない — 実装不要。)

### 18.7 Kivact 実装方針

- 戦略: **RESTYLE `kivact::ui::tag_input` + NEW kit component `soft::tokenizer`**。
  - `kivact::ui::tag_input` (crates/kivact-ui/src/tag_input.rs) は文字列 `Vec<String>` + Enter で追加するだけで、検索・候補・非同期・上限・overflow が無い。soft テーマはこれを `TagInputRecipes` (recipes.rs:457-468: `root_recipe` / `tag_recipe` / `remove_recipe` / `input_recipe`) で 18.4.1 / 18.4.4 の見た目に restyle する (root = control box、tag = Token md、remove = 16×16 × 12px icon、input = flat 14/20)。なお tag_input は editor を semantic 先頭に置く grid 構成で focus 順が Astryx と異なるが、標準部品なので変えない。
  - Astryx 相当の機能は kit 部品 `soft::tokenizer` として新規実装する。
- 最も近い kivact-ui-basic: **`crates/kivact-ui-basic/src/combobox.rs`** の `chips_field` (multiple combobox: 折り返す chip 行 + borderless input、Backspace で最後を削除、`focus_on_press(input_ref)`、`anchor.focus_within()` で focus ring) と `popup()` (overlay + `at_least_anchor_width(true)`)。`list_popup.rs` の `surface_style` / `row_style` の書き方も参照。
- 部品:
  - 入力: `kivact::ui::text_input(label, query)` を枠なしで (`input_group.rs` の `borderless` 相当)、`.forward_keys(vec![ArrowDown, ArrowUp, Enter, Escape, Home, End])`、Backspace は `on_key_default` (tag_input.rs と同じ手法)。
  - 候補: `kivact::interaction::use_combobox(options, ListSelection::Single(intercept))` + `.popup(open)`。`options` は検索結果を入れる `signal(Vec<ListItem<K>>)`、`.filter(|_, _| true)` (絞り込みはソース側)。`intercept` は「書き込まれたら add して None に戻す」`binding(..)` にすると、Single の「選択で閉じる」挙動がそのまま Astryx と一致する。Multiple は toggle 動作で Astryx と違うので使わない。
  - 非同期検索: `kivact::task::resource(query_with_debounce, |q| source.search(q))` (latest-wins で古い結果を捨てる)。debounce は `kivact::task::sleep(Duration::from_millis(150))` を future 先頭で await。`ctx.update` (途中結果) は resource の中間 publish が無いので、途中結果を `signal` に直接書く `spawn` 版で実装 (Typeahead 節と共通化)。
  - dropdown: `kivact::view::overlay(control_ref, Placement::Below, ..).offset(4).at_least_anchor_width(true).on_dismiss(..)`。
  - end lane: Kivact の layout では absolute 子の幅を padding に反映する CSS 変数が無い。**lane を測らず、wrapper を `[tokens+input の wrap 行 (flex 1, min-width 0)] [lane (shrink 0, align-self flex-start, 高さと余白は 18.10)]` の 2 カラム row にする**と同じ見た目になる (全行が lane の手前で折り返し、lane は 1 行目中央)。これが `useEndLaneReserve` の native 版 (18.10)。
  - overflow (`CollapseInline`): blur 中は 1 行に収まる token 数を `on_layout`/measure で求めて `+N more` を出す。kivact-ui-basic に OverflowList 相当は無いので kit 内 helper (幅測定 → 表示数 signal) を作る。`CollapseLayer`: `overlay(placeholder_ref, ..)` で placeholder の**上端に重ねる**配置が必要。`Placement::Below` + `.offset(-(size))` (負の offset) が効くか要確認 — 効かなければ `overlay_at(placeholder の window 左上, Placement::Below, ..)` で代替。幅は `at_least_anchor_width(true)` + max 幅も anchor 幅に固定。
  - announcements: 不可視の `text(..)` ノードに `NodeExt::live_region(LiveRegion::Polite)`、文言を差し替えて通知 (Typeahead 節と共有の `announce` helper)。
  - disabled tooltip: `kivact::ui::tooltip(msg).attach(control)`。
  - focus 転送: control の `focus_within()` が false→true になった瞬間に、focus 先が input 以外 (× ボタン) かつ直前の focus が field 外なら `input_ref.focus()`。kivact に relatedTarget 相当が無ければ「直前フレームの focus_within が false だった」ことで判定する。
- 難所 / 代替:
  - `:placeholder-shown` による input 幅切替 (18.4.3) は query 空 / focus 状態の signal から style を計算すれば同等。
  - hover ring は `(hover: hover)` 限定 → タッチ専用環境 (`kivact::platform::capabilities().touch` かつ mouse 無し) では出さない。判定できなければ常に hover style を有効にしてよい (タッチでは hover が来ない)。
  - `color-mix` は事前計算した 8 桁 hex を使う。
  - form 送信 (`htmlName`) は drop。

### 18.8 受け入れ確認

- [ ] `role=group` 名 "{label}" の中に `role=combobox` 名 "{label}" がある。tokens 0 個で placeholder が見え、高さ sm/md/lg = 28/32/36px。
- [ ] tokens を足すと枠の内側余白が 3px になり token (md 24px, radius 6) と枠 (radius 10) が同心円に見える。2 行目との間隔 3px、token 間 4px。
- [ ] 文字入力 150ms 後に候補が出る。ArrowDown/Up が両端で wrap、Enter で token 追加・クエリ消去・menu 閉、"Added X" が読まれる。
- [ ] クエリ空で Backspace → 最後の token 削除、"Removed X"。× ボタン (名前 "Remove X") でも同じで focus は input。
- [ ] `hasClear`: 20×20 の × が右端 1 行目中央、tooltip "Clear"、押すと全消去 + "All items removed"、押しても input の focus が外れない。
- [ ] 長い token が lane の下に潜らない (lane の手前で ellipsis / 折り返し)。RTL で lane が左。
- [ ] `hasCreate`: 未知語で末尾に `Create "foo"`、選択済み label と同名では出ない。
- [ ] `maxEntries` 到達で input が消え、候補が出ない。Backspace は効く。
- [ ] Tab で field に入ると (tokens があっても) input に focus。Shift+Tab で × ボタンに戻れる。
- [ ] disabled: 半透明、× と Clear 無し、Tab で focus でき aria-disabled、`disabledMessage` の tooltip が上に出る。
- [ ] status error: 赤枠 (#c0262c / #ffaea7)、focus しても赤枠のまま。hover で薄い赤の inset 2px。
- [ ] `unfocusedInline`: blur で 1 行 + "+N more" (12px secondary)、focus で展開。`unfocusedLayer`: レイアウトは動かず、focus で同じ幅のパネルが field の上端から下に被さる。
- [ ] docsite (light/dark) と比較: chip 背景 light #f0f0f0 相当 / dark #3c3c3c 相当、focus 枠 #1b1b1b / #f1f1f1、dropdown 背景 #ffffff / #1b1b1b + shadow-low、highlight 行の薄い灰。

### 18.9 TokenizerBottomSheet (内部 presentation)

#### 概要
`experimental.presentation = 'bottom-sheet' | 'adaptive'` の時だけ使う touch 向け表示 (公開 API には未露出; 26.6)。`adaptive` は「primary pointer が coarse かつ (viewport 幅 < 768px または 高さ < 500px)」で sheet、それ以外 popover (`useAdaptivePresentation`, themeAdaptations.ts:83-93)。開いている sheet は閉じて exit が終わるまで presentation を変えない (latch)。

#### Anatomy
```
button (trigger) aria-haspopup="dialog" aria-expanded
 ├─ [startIcon] ├─ Token × N (× 無し) | placeholder span
VisuallyHidden "{n} selected"
BottomSheet (height "tall" = 92dvh, padding 0, label = field label)  ← 押した時だけ mount
 └─ body (column, padding 32px top / 12px inline / 12px bottom)
    ├─ search row (pb 8px): PanelSearchInput "Search {label}", placeholder "Search…", min-height 44, padding-block 8
    ├─ list (flex 1, overflow-y auto, aria-busy)
    │   ├─ [create row] Text "Create "x"" + Button secondary lg "Add" (aria-label "Add x")
    │   ├─ [pending] Spinner md (中央, min-height 44)
    │   ├─ [empty] EmptyState compact (search icon lg secondary, "No results found", "Try a different search.")
    │   └─ role=list > role=listitem(aria-setsize, aria-posinset) > button role=checkbox aria-checked aria-label
    │        row: min-height 44, padding 8px/4px, gap 8, radius 10, Text body 14/20 + CheckboxIndicator md (末尾)
    ├─ footer (クエリ空の時のみ; pt 12, gap 8): Button secondary lg "Clear all" | Button primary lg "Done" (各 flex 1)
    └─ [AlertDialog] "Clear all selected items?" / "This removes all selected items from {label}." / action "Clear selection"
```

#### API
Tokenizer の props をそのまま使う。sheet で意味を持たない `renderItem`, `renderToken`, `endContent`, `hasClear`, `hasEntriesOnFocus`, `menuWidth`, `tokenOverflowBehavior`, `hasAutoFocus` は dev 警告して無視 (Tokenizer.tsx:891-925)。Kivact は `debug_assert!` 相当の `tracing::warn` 1 回。

#### Visual spec
- trigger: control box と同じ (`inputWrapperStyles.base` + wrapper + `sizeStyles` + tokens 有りなら 3px inset)、`width 100%`、`text-align start`、cursor pointer (disabled は default)。placeholder `--color-text-secondary`、padding-inline 4px。tokens は `Token label size` (× 無し)。disabled は opacity 0.5 (forced-colors 時のみ GrayText)。
- 行 (TokenizerBottomSheet.tsx:112-131): 背景 transparent、文字 `--color-text-primary`、radius 10px、button のため hover/press 色は無し (UNRESOLVED: press feedback 未適用、touch でも色変化なし)。上限到達で未選択行は aria-disabled (indicator disabled border `--color-border` #00000014 / #ffffff1A)。
- その他 (BottomSheet, PanelSearchInput, CheckboxIndicator, EmptyState, AlertDialog, Button) は各節の spec をそのまま使う。

#### Motion
BottomSheet / AlertDialog / Spinner の各節に従う (sheet 自体の enter/exit、scrim、drag)。行のトグルは即時、行は並べ替えない。

#### Behaviour & a11y
- trigger 押下 → sheet を open (query は前回強制 close 時の値か空)。Done / swipe / scrim / Escape で閉じると query を消す。disabled になって強制的に閉じた時は query を保持。閉じた後 focus は trigger に戻る。
- 行の並び: open 時点で選択済みの項目 (value 順、クエリにマッチするもの) → 結果に無い選択済み項目 → 検索結果。open 中のトグルでは**行が動かない** (checkbox だけ変わる)。次回 open で選択済みが先頭に再集約。
- 行押下で add/remove をトグルし "Added X" / "Removed X" を sheet 内 live region で通知。
- 検索: 空クエリは bootstrap を即時、文字は `debounceMs` 後。`minQueryLength` 未満は検索しない (pending も empty も通知も無し)。同期ソースは layout 前に確定し Loading を一瞬も描かない。結果は `maxMenuItems` で切る。決着ごとに "{n} results" (create 行込み) を 1 回通知。空クエリで 0 件なら何も出さない。
- 長いリスト: 50 行ずつ描画、底から 200px 以内のスクロール、または最後の描画行に focus が来たら次の 50 行。
- focus 中の行が新しい結果で消えたら search field に focus。
- create 行の Add: 追加 → クエリ消去 → search field に focus。
- Clear all: 値 0 なら aria-disabled + tooltip "Nothing selected"。押すと AlertDialog。確定 → 全削除、focus は search field、"All items removed"。キャンセル → focus は Clear all に戻る。
- trigger: `aria-describedby` = 呼び出し側 id + description + status + disabled 理由 + "{n} selected" + 各 token。

#### Kivact 実装方針
- `kivact::ui::dialog(label, open)` の `DialogKind::Drawer(Side::Bottom)` (soft テーマで BottomSheet の見た目に restyle) か、kit の bottom sheet 部品 (BottomSheet 節) を使う。行は `kivact::ui::list` ではなく `node().role(Role::CheckBox).checked(..)` の button 群 (`use_list` の roving は不要; Tab 順で巡回)。Clear 確認は `dialog(..).kind(DialogKind::AlertModal)`。
- 50 行バッチは `scroll` の offset 監視 (`on_scroll`) で `rendered_count` signal を増やす。
- adaptive 判定: `kivact::platform::capabilities().touch` と `window_size()` (幅 < 768 または 高さ < 500)。「primary pointer が coarse」とは厳密には異なる (UNRESOLVED: kivact に primary pointer 種別の環境値が無い; touch 能力で近似)。
- 最も近い kivact-ui-basic: `drawer.rs` (swipe dismiss) + `combobox.rs`。

#### 受け入れ確認
- [ ] trigger は button、aria-haspopup=dialog、押すと高さ 92% の sheet、上に 44px の検索欄、下に Clear all / Done。
- [ ] 行トグルで並びが変わらない。閉じて再度開くと選択済みが先頭。
- [ ] Clear all → 確認 → 全消去、focus は検索欄。キャンセルで Clear all に focus。
- [ ] Done で閉じると trigger に focus、次回 open 時クエリは空。

### 18.10 useEndLaneReserve (Tokenizer 内部 hook)

- 役割 (useEndLaneReserve.ts:90-147): absolute 配置の end lane (Spinner / endContent / Clear) は flow 上の幅を持たないため、lane を `observeResize` (ページ全体で 1 つの共有 ResizeObserver、1 フレーム 1 コールバック) で測り、**`offsetWidth` (transform の影響を受けない border-box 幅、整数)** を使って lane の親 (wrapper) に `--_tokenizer-end-lane-reserve: calc(8px + {w}px)` を書く。幅 0 なら変数を削除。lane unmount 時に observer 解除 + 変数削除。値は React state を通さない (再 render ゼロ)。
- 読み手: tokens 無し → input の `padding-inline-end: var(reserve, 0px)`。tokens 有り → wrapper の `padding-inline-end: calc(3px + var(reserve, 0px))` (全行が lane の手前で止まる)。`inputAtMax` / overflow 折り畳み時は input に reserve を付けない。
- 計算の意味: input の content box は wrapper padding 分だけ枠から内側、lane も枠から 8px 内側 → 必要な逃げ = inset 8px + lane 幅 (padding 同士が相殺)。
- テスト (Tokenizer.test.tsx:2133-2507) が固定している観察可能な性質: (1) 変形 (`scale`) 下でも未変形幅を予約、(2) Spinner だけでも予約、(3) token 行も input も lane の手前で止まる、(4) 折り畳み経路では input に padding を付けない、(5) 検索 1 回で field の commit / token の再 render を増やさない、(6) 全 field で observer 1 つ、(7) lane が消えたら幅を返す。
- Kivact: 測定は不要。18.7 の「2 カラム row」構成 (`row((wrap_area.grow(1).min_width(0), lane.shrink(0).align_self(Start)))`) にすれば (1)-(4)(7) が layout だけで満たされる。lane を `align_self(Start)`、高さ `size − 2×(1px border + padding-block)` (tokens 有り: sm/md/lg = 20/24/28、tokens 無し: 18/22/26)、中身を縦中央寄せにすると、lane の中心が border-box 上端から `size/2` に来て Astryx の `top: calc(size/2 − 1px); translateY(-50%)` と一致する。tokens 無し時は wrapper 内 padding 8px、tokens 有り時は lane と枠の間が 8px になるよう lane 側に `margin-inline-end: 5px` (= 8 − 3) を付ける。Spinner は idle でも領域を確保 (`visibility` 相当 = `opacity(0)` かつ semantics 非公開) して検索ごとに折り返しが変わらないようにする。

## 19. PowerSearch

Source: `packages/core/src/PowerSearch/` (PowerSearch.tsx, PowerSearchToken.tsx, PowerSearchEditPopover.tsx, PowerSearchFilterEditor.tsx, PowerSearchValueEditor.tsx, formatFilterValue.ts, resolveOperatorLabel.ts, resolveDateTimeRangePart.ts, useInternalConfig.ts, usePowerSearchConfig.ts, usePowerSearchSource.ts, types.ts)。Docs: https://astryx.atmeta.com/components/PowerSearch 。Stories: `PowerSearch.stories.tsx`, `PowerSearchWithTable.stories.tsx`, `MobilePowerSearch.stories.tsx`。

### 19.1 概要

- 各 token が「field + operator + value」の 1 フィルタを表す構造化フィルタバー。複数条件を組み合わせる多次元フィルタに使う。単一 field のキーワード検索なら普通の text input を使う。
- 推奨 (PowerSearch.doc.mjs:251-276): field 名と alias を分かりやすく、`resultCount` で結果件数を返す。非推奨: 単純なキーワード検索に使う、disabled を Tooltip で包む (→ `disabledMessage`)。
- 内部的には **Tokenizer そのもの** (PowerSearch.tsx:1120-1150): 検索ソース = field 候補、value = 現在のフィルタ群、renderToken = クリックで編集できる filter token、追加/編集は token 下に開く edit popover。

### 19.2 Anatomy

```
div (root, themeProps 'power-search'; popover の anchor)
└─ Tokenizer (label "Search" 非表示, hasEntriesOnFocus, debounce 0, maxMenuItems 1000)
   ├─ [startIcon]
   ├─ filter token × N  (Token clickable: span container + invisible button + endContent + ×)
   │    "Status: is" + " " + **Active**
   ├─ input combobox (placeholder "Search…" はフィルタ 0 個の時のみ)
   └─ end lane: [resultCount "12 results"] [endContent] [Clear all]
field menu (listbox): TypeaheadItem(icon, label, description) — 空クエリは全 field (group 見出し付き)
edit popover (top layer, role none, below/start, offset 4px, anchor = root)
└─ PowerSearchEditPopover (container-type inline-size)
   ├─ content (padding 16): HStack gap 8 [Field Selector][Operator Selector][Value editor]
   └─ footer (padding 12, top 0): [Delete (edit 時)] ……… [Cancel][Apply]
   (operator が nested の時: Field Selector + TreeList の入れ子フィルタ編集)
```

公開サブ部品: `PowerSearchToken` (既定 token)、`PowerSearchFilterEditor` (既定 editor = EditPopover の薄い wrapper; 両方とも `components` override の土台として export)。

### 19.3 API

| Astryx prop | 型 / default | Kivact |
|---|---|---|
| `config` | `PowerSearchConfig {name, fields[], contentSearchFieldKey?}` | 第 1 引数 `power_search(config, filters)` の `PowerSearchConfig` (所有値 or `Value<PowerSearchConfig>`) |
| `filters` + `onChange(filters, changeType, index)` | `PowerSearchFilter[]`; `'add'/'edit'/'remove'` | 第 2 引数 `impl Into<Binding<Vec<PowerSearchFilter>>>` + `.on_change(Fn(PowerSearchChange))` (`Add(index)`/`Edit(index)`/`Remove(index)`) |
| `label` | "Search" | `.label(text)` |
| `isLabelHidden` | **true** | `.hide_label(bool)` |
| `placeholder` | "Search…" | `.placeholder(text)` |
| `hasAutoFocus` | false | `.auto_focus(bool)` |
| `hasClear` | **true** | `.clear_all(bool)` |
| `isReadOnly` | false | `.read_only(bool)` |
| `isDisabled` / `disabledMessage` | false | `.disabled(..)` / `.disabled_message(..)` |
| `startIcon` | | `.start_icon(view)` |
| `onFocus`/`onBlur` | | `.on_focus_enter` / `.on_focus_leave` |
| `status` / `statusVariant` | | Field 共通 |
| `menuWidth` | | `.menu_width(f32)` |
| `maxTokenLength` | 40 | `.max_token_length(usize)` |
| `maxOperatorMenuItems` | (10) | `.max_value_menu_items(usize)` |
| `maxSearchResults` | 10 | `.max_search_results(usize)` |
| `popoverSaveButtonLabel` | "Apply" | `.save_label(text)` |
| `timezoneID` | | `.time_zone(TimeZoneId)` (formatFilterValue のみ使用) |
| `tokenOverflowBehavior` | 'none' | `.overflow(TokenOverflow)` |
| `endContent` | | `.end_content(slot)` |
| `resultCount` | number / string | `.result_count(impl IntoValue<Option<ResultCount>>)` (`Count(u64)` / `Text(String)`) |
| `size` | 'md' | `.size(ControlSize)` |
| `components` | `{[opType]: {Token?, Editor?}}` | `.token_for(OperatorKind, Fn(PowerSearchTokenProps) -> View)` / `.editor_for(OperatorKind, Fn(PowerSearchEditorProps) -> View)` |
| `ref` / `handleRef` (`focusTypeahead`/`blurTypeahead`) | | `.node_ref(..)`, input の NodeRef 取得 `.input_ref(NodeRef)` |
| `xstyle/className/style`, `data-testid` | | `.style(..)`, `.test_id(..)` |

データ型 (types.ts; Rust では enum に):
- `OperatorValue` (editor 種別): `Empty | String{search_source?, arbitrary_allowed} | StringList{search_source?, arbitrary_allowed, tokenization?} | Integer{min,max,units} | Float{min,max,units} | Time{min,max} | DateAbsolute{date_only} | DateRelative{past_allowed=true, future_allowed=true} | DateRange{interval_presets, relative_presets} | Enum{values} | EnumList{values} | EntityList{search_source?, arbitrary_allowed, tokenization?, render_item?} | Custom{editor, get_string} | Nested`。
- `FilterValue` (保存値): `Empty | String(String) | StringList(Vec<String>) | Integer(i64) | Float(f64) | Time(String "HH:MM") | DateAbsolute{unix_seconds} | DateRelative(String e.g. "7d_ago") | DateRange{start, end: DateTimeRangePart} | Enum(String) | EnumList(Vec<String>) | EntityList(Vec<Entity{id,label,photo?}>) | Custom(String json) | Nested(Vec<PowerSearchFilter>)`。
- `DateTimeRangePart = Now | Absolute{unix_seconds} | Relative{back_value, unit: second..year, anchor_key?}` (`resolveDateTimeRangePart`: month = 2,592,000s、year = 31,536,000s の固定秒)。
- `PowerSearchOperator {key, value: OperatorValue, label: Text}` — Astryx の `label` (生文字列) / `i18nKey` (catalog) の排他 union は Rust では `OperatorLabel::{Text(String), Message(&'static str key)}`。
- `PowerSearchField {key, label, operators, icon?, default_operator?, group?, description?, typeahead_aliases, typeahead_min_query_length?, value_match_allowed=true}`。
- `PowerSearchFilter {field, operator, value, read_only}`、`PartialFilter {field, operator?, value?}`。
- `OperatorTokenizationConfig {regex?, sort?}` は**型だけで未使用** (どのコードも読まない) → Kivact は型を用意しなくてよい (ペースト分割なし)。

ヘルパー API:
- `createPowerSearchConfig(defs)` / `usePowerSearchConfig(defs)` (usePowerSearchConfig.ts): `{key, type: string|number|boolean|date|enum|enum_list|string_list, label?, enumValues?}` から既定 operator 付き field を生成し、`applyFilters(filters, rows)` (全フィルタ AND) を返す → Kivact `power_search_config(&[FieldDef])` → `(PowerSearchConfig, fn apply_filters)`。既定 operator 表:

| type | operators (key: 表示) | default |
|---|---|---|
| string | contains, not_contains "does not contain", starts_with, not_starts_with "does not start with", ends_with, not_ends_with "does not end with", is, is_not "is not" (すべて value `string`) | contains |
| number | equals "is", not_equals "is not", greater_than "is greater than", less_than "is less than", greater_than_or_equal "is greater than or equal to", less_than_or_equal "is less than or equal to" (value `float`) | equals |
| date | before "is before" / after "is after" (`date_absolute`), between "is between" (`date_range`) | after |
| boolean | is_true "is true", is_false "is false" (`empty`) | is_true |
| enum | is, is_not (`enum`), is_any_of "is any of", is_none_of "is none of" (`enum_list`) | is |
| enum_list / string_list | is_any_of, is_none_of | is_any_of |

  一致判定: 文字列は小文字化して比較、enum_list/string_list は any/none、date_range は Now 基準で解決して両端含む。time / date_relative / entity_list / custom / nested は常に true。

### 19.4 Visual spec

#### 19.4.1 バー
Tokenizer (18.4) と同一。差分:
- ラベル既定非表示。placeholder はフィルタ 0 個の時のみ。
- result count (PowerSearch.tsx:140-147, 1091-1107): end lane の先頭に `12px / leading 20px`、`--color-text-secondary` (#474747 / #9e9e9e)、nowrap。数値は "{n} result(s)" (Intl 数値書式)。
- Clear は `hasClear && !isReadOnly` (既定 on)。

#### 19.4.2 filter token (既定 renderToken, PowerSearch.tsx:838-946)
- `Token label="{field}: {operator}" size={PowerSearch size} endContent={value} onClick onRemove`。label 部 12px / 20px / weight 500、value 部は同サイズで **weight 700** (`--font-weight-bold`)、label と value の間は Token の gap 4px。色は default (背景 `--color-neutral` #0000000F / #ffffff1A、文字 #000000 / #ffffff)。高さ sm 20 / md 24 / lg 28、padding-inline 8px、radius 6px。
- 単一 entity (photo 有り) の entity_list は先頭 icon に Avatar 16px。
- clickable (編集可能時): span container + 透明 button (label 部) + value + ×。
  - hover (hover:hover): background-image に `--color-overlay-hover` (#0000000D / #ffffff0D) を重ねる。mouse press: `--color-overlay-pressed` (#0000001A / #ffffff1A)。touch: press 150ms 遅延・slop 10px・quick tap 100ms・release 200ms cubic-bezier(0,0,0.2,1)。transition `background-image` 125ms cubic-bezier(0.24, 1, 0.4, 1) (Token.tsx:147-155)。cursor pointer。
  - focus: container に **focus-within** で outline 2px solid `--color-accent` (#1b1b1b / #f1f1f1) offset 3px (Token.tsx:515-523)。
- 非編集 (isReadOnly / isDisabled / filter.isReadOnly): click も × も無し (static span)。isDisabled は opacity 0.5 (wrapper の 0.5 と重なる)。
- 値テキスト (PowerSearchTokenValue, PowerSearch.tsx:159-360) — **見た目に出る方**:
  - 予算 `adjusted = max(maxTokenLength − len(field label) − len(operator label), 10)` 文字 (grapheme 単位)。`truncateString`: `adjusted + 3` 文字以内はそのまま、超えたら先頭 `adjusted` 文字 + `"..."`。
  - string / enum(label) / date_relative / custom(getString): 上記切り詰め。integer/float: 生の数値 (書式・単位なし)。time: そのまま。date_absolute: `Intl.DateTimeFormat(locale, {year numeric, month short, day numeric})` 例 "Jan 5, 2026" を切り詰め。date_range: "date range"。
  - list 系 (string_list / enum_list(label) / entity_list(label)): 0 個 → 値なし、1 個 → 切り詰め、複数で合計文字数 > 予算 → "{n} items"、以内 → ", " 連結。nested: "1 filter" / "{n} filters"。empty: 値なし。
  - ("items"/"date range"/"filter(s)" は Astryx では直書き英語。Kivact は i18n key にする。)
- tokenizer item label (読み上げ・通知用文字列、PowerSearch.tsx:654-687): `"{field}: {operator} {formatFilterValue(..)}"`。

#### 19.4.3 field menu (Tokenizer の dropdown)
18.4.6 と同じ。行は `TypeaheadItem` (gap 8px): icon (field.icon、無ければ operator 種別で `search` / `calendar` (date 系) / `clock` (time) / `menu` (enum 系)、16px、`--color-icon-secondary` #6a6a6a / #9e9e9e) + label 14/20 + description 12/20 secondary。空クエリ時は全 field を「group 無し → 各 group (見出し付き)」の順で最大 1000 行。

#### 19.4.4 edit popover (PowerSearch.tsx:126-138, 1153-1158; PowerSearchEditPopover.tsx:51-104)
- 配置: anchor = PowerSearch root、placement below、alignment start、offset 4px、`width = anchor 幅`、`min-width = min(400px, viewport 上限)`。収まらなければ layer runtime が flip / slide。
- surface: usePopover 既定 = 背景 `--color-background-popover` (#ffffff / #1b1b1b)、radius `--radius-container` 12px、`--shadow-low` (18.4.6 と同値)。close ボタンなし。
- container: overflow hidden、`container-type: inline-size`。
- content: padding 16px。行 = HStack gap 8px (vAlign 未指定 → align-items 未設定 = CSS 既定 stretch; Stack.tsx:261-275、stack.stylex.ts:16-36。子は全て 32px なので実質上下揃い)、`flex-wrap: nowrap`、**popover 幅 < 400px では wrap**。
  - Field Selector: `flex 1 1; min-width 0`、Selector size md (32px)、label 非表示 "Field"。
  - Operator Selector: `flex-grow 1; flex-shrink 0; max-width 100%` (長い訳語は trigger 内で ellipsis)、label "Operator"。operator が 1 個かつ empty 型なら非表示。
  - Value editor cell: `flex-grow 2; min-width 0`。
- footer: padding 12px、top 0。HStack `justify: space-between`, gap 8: 左 `Delete` (ghost sm, edit mode かつ非 readOnly のみ; それ以外は空 div)、右 `Cancel` (ghost sm) + `Apply` (primary sm, value 未入力で disabled)。empty 型 operator では footer ごと出さない。
- Button sm: 高さ 28px、padding-inline 12px、radius 10px、label 14/20 w500。primary 背景 `--color-accent` (#1b1b1b / #f1f1f1)・文字 `--color-on-accent` (#ffffff / #111111)、ghost 背景 transparent・文字 text-primary。disabled opacity 0.5。hover/press overlay + scale 0.98 (Button 節)。
- nested 型: content = VStack gap 8 [Field Selector 行] + `TreeList density="balanced"`: root ラベル = group operator Selector (operator 2 個以上) かテキスト (operator label か "Group")、子 = 各 sub-filter 行 (field Selector 幅 200px / operator Selector 幅 180px / value editor grow 2; 行も 400px 未満で wrap) + 末尾に × (ghost sm icon-only "Remove filter")、各 group の最後に "+ Add filter" (ghost sm)。すべて展開固定。

#### 19.4.5 value editor (PowerSearchValueEditor.tsx)
| operator 型 | editor | 値の保存 |
|---|---|---|
| empty | なし | 選んだ瞬間に保存 |
| string | `TextInput` (label "Value", placeholder "Enter value…")。`searchSource` 有りなら `Typeahead` (placeholder "Search…", debounce 150) | Typeahead は候補確定で即保存 |
| string_list | `Tokenizer` (label "Values", placeholder "Add values…", `hasCreate` = arbitrary 許可 or ソース無し, debounce ソース有 150 / 無 0) | Apply |
| integer / float | `NumberInput` (placeholder "Enter number…", min/max/units, integer は整数のみ) | Enter で確定値を即保存 (不正 draft は保存しない) |
| time | `TimeInput` (label "Time") | Apply |
| date_absolute | `DateInput` (label "Date"; unixSeconds ↔ ISO 日付、UTC 日付部) | Apply |
| date_relative | `Selector` (label "Relative date"): {1,3,7,14,30} days / {1,2,4} weeks / {1,3,6,12} months × "ago" / "from now" (value `"7d_ago"` / `"7d_from_now"`、label "7 days ago"; 英語直書き) | 選択で即保存 |
| date_range | `DateRangeInput` (label "date range", hasClear false; UTC 0 時の ABSOLUTE 両端で保存) | Apply |
| enum | `Selector` (label "Value") | 選択で即保存 |
| enum_list | `Tokenizer` (placeholder "Select values…", hasEntriesOnFocus, debounce 0, maxMenuItems = 全件) | Apply |
| entity_list | `Tokenizer` (label "Entities", placeholder "Search…", renderItem, photo を保持) | Apply |
| custom | 呼び出し側 `Editor{value, onChange, placeholder "Enter value…", isDisabled}` | Apply |
| nested | (EditPopover 側の TreeList) | Apply |

各 editor の見た目は TextInput / NumberInput / Selector / DateInput / DateRangeInput / TimeInput / Typeahead / Tokenizer 各節の spec のまま (size md)。

### 19.5 Motion

| トリガ | 対象 | from→to | duration / easing | reduced motion | 出典 |
|---|---|---|---|---|---|
| edit popover 表示 | opacity, transform | 0 → 1、translateY(−8px) scale(0.95) → 0 / 1 | `--duration-fast-max` 165ms cubic-bezier(0.24, 1, 0.4, 1)、fill backwards | fine pointer: なし / coarse: opacity のみ min(165ms, 200ms) | layerAnimations.stylex.ts:31-37, 92-123; PowerSearch.tsx:1157 |
| edit popover 閉 | — | 即時 (exit animation なし) | — | — | |
| filter token hover/press | background-image | overlay | 125ms cubic-bezier(0.24, 1, 0.4, 1) | — | Token.tsx:147-155 |
| バー / field menu / Clear / Spinner | Tokenizer 18.5 と同じ | | | | |
| editor 内部 (Selector, DateInput など) | 各節 | | | | |

### 19.6 Behaviour & a11y

フィルタ追加の流れ (filter-token grammar):
1. input に focus (hasEntriesOnFocus) → 全 field の一覧 (group 順)。
2. 入力 (debounce 0) → `usePowerSearchSource.search(query)` (usePowerSearchSource.ts:42-209)。q = 小文字 trim:
   - a. field label か alias が q を含む → **field 行** (id `field.key`; default operator で開く)。`typeaheadMinQueryLength` 未満の field は飛ばす。
   - b. `"{field label} {operator label}"` が q を含む → **field+operator 行** (id `field:op`, label "Status is")。
   - c. 値マッチ (`isValueMatchAllowed !== false` の field のみ): q が `"{field} {op} "` で始まり残りがある → string/string_list は生値をそのまま `Title contains "foo"` (引用符付き)、enum/enum_list は label が残りを含む各値 `Genre is Fiction`。これが 1 つも無く、q が `"{field} "` で始まり、残りがどの operator label の接頭辞でもない → 全 operator について同じ値候補。
   - d. `contentSearchFieldKey` 有りで、q がどの field label / "field op" とも完全一致しない → 先頭に **content 検索行** `"{query}"` (その field の default operator、value = string)。
   - e. `maxSearchResults` (10) で切る。重複 id は除外。
3. 行を確定 →
   - 値付き行 (c, d) / empty 型 operator → 即フィルタ追加 (`onChange([...], 'add', len)`)。
   - それ以外 → edit popover を **adding** で開く (field / operator 決定済み、value 未定)。`requestAnimationFrame` 後に popover.show() + バー input を blur。value editor の最初の focus 可能要素に rAF 後 focus (field 切替で editor が作り直された時も)。
4. editor: Enter = Apply (value 有り、IME 変換中でない、focus が button でない、子が `defaultPrevented` していない時)、Escape = Cancel。Selector/enum など「選択で即保存」は onSave。保存/取消/削除 → popover 閉 → **バー input に focus**、`onChange(filters, 'add'|'edit'|'remove', index)`。外側クリック (light dismiss) でも閉じる (この場合は再 focus しない)。
5. field を変える → operator は新 field の default、value 消去 (default が empty なら即保存)。operator を変える → value 型が同じなら値を保持、違えば消去 (empty なら即保存)。同じものを選び直しても保存しない。

編集・削除:
- token (label 部) クリック or focus して Enter/Space → edit popover を **editing** で開く (現在値入り)。token の mousedown は preventDefault (input focus 転送を起こさず、候補 menu も開かない)。empty 型 filter のクリックは popover を開かず同じ値で `'edit'` を確定 (empty 用 Editor override がある時は開く)。
- × / Backspace (input 空) → 該当 filter を削除 (`'remove'`, index)。Clear all → 全削除、"All filters removed" を通知。
- `isReadOnly`: token はクリック・削除不可、Clear 非表示。ただしバーの field menu は開け、選ぶと read-only の editor (全 Selector disabled、Delete 無し、value 無しで Apply disabled) が開く (Astryx の挙動; parity を保つ)。
- `isDisabled`: Tokenizer の disabled と同じ + token 非活性。
- 同じ field を複数回追加できる (token id `filter-{index}-{field}-{op}` は field 候補 id と衝突しないため除外されない)。
- focus 順: [token1 label button][token1 ×]…[input][Clear]。外から Tab → input。input から Shift+Tab → 直前 token の ×、もう一度 → token label button (PowerSearch.test.tsx:646-671)。
- semantics: token label button の名前は `"{field}: {operator}"` のみ (値は button 外の endContent; Astryx の仕様。Kivact では button に `described_by(value text)` を付けると値も伝わり改善になるが、名前自体は変えない)。× は "Remove {field}: {operator}"。popover は `role=none` (中身の listbox / form 部品がそのまま意味を持つ)。result count は mount 後の変化時のみ polite 通知 ("{n} results" / 文字列そのまま)。
- 通知: Tokenizer の "Added {label}" は field 行選択時にも出る (例 "Added Status")。削除は "Removed Status: is Active"。

i18n keys:

| key | English |
|---|---|
| `soft.power_search.label` | Search |
| `soft.power_search.placeholder` | Search… |
| `soft.power_search.all_filters_removed` | All filters removed |
| `soft.power_search.result_count` | {count} result / {count} results |
| `soft.power_search.editor.field` / `.operator` / `.group_operator` / `.group` | Field / Operator / Group operator / Group |
| `soft.power_search.editor.add_filter` / `.remove_filter` | + Add filter / Remove filter |
| `soft.power_search.editor.delete` / `.cancel` / `.apply` | Delete / Cancel / Apply |
| `soft.power_search.value.value` / `.values` / `.time` / `.date` / `.relative_date` / `.start_date` / `.end_date` / `.entities` / `.date_range` | Value / Values / Time / Date / Relative date / Start date / End date / Entities / date range |
| `soft.power_search.value.search_placeholder` / `.enter_value` / `.add_values` / `.enter_number` / `.select_values` | Search… / Enter value… / Add values… / Enter number… / Select values… |
| `soft.power_search.value.items_count` / `.entities_count` / `.filters_count` | {count} item(s) / {count} entity/entities / {count} filter(s) |
| `soft.power_search.value.relative_ago` / `.relative_from_now` | {amount} {unit} ago / {amount} {unit} from now (Astryx は直書き; key 化) |
| `soft.power_search.operator.contains` … `.is_none_of` | 19.3 の表の英語 (contains, does not contain, starts with, does not start with, ends with, does not end with, is, is not, is (equals), is not (not_equals), is greater than, is less than, is greater than or equal to, is less than or equal to, is before, is after, is between, is true, is false, is any of, is none of) |

### 19.7 Kivact 実装方針

- 戦略: **NEW kit component** `soft::power_search` — kit の `soft::tokenizer` (18.7) を中核に組む。標準部品で RESTYLE できる対象は無い。
- 最も近い kivact-ui-basic: `crates/kivact-ui-basic/src/combobox.rs` (chip + 候補)、編集 popover の部品構成は `date_picker.rs` (trigger 下の popover に calendar/入力を置く) と README の「popover = `kivact::ui::dialog` の `DialogKind::Popover`」。
- 部品対応:
  - バー = `soft::tokenizer(label, field_source, items)`。`items` は `filters` から `memo` で導出した `Vec<SearchItem<FilterKey>>`、tokenizer の書き込み (add/remove) は `binding(items, move |new| ..)` の setter で filter 操作に翻訳 (PowerSearch.tsx:690-752 と同じ分岐)。`.token(..)` slot で filter token を描く。
  - filter token: kit の `soft::token(label).end_content(value).on_press(..).on_remove(..)` (Token 節)。押下可能 token は `kivact::interaction::use_toggle_button` ではなく単純な press (`node().role(Role::Button).on_press`) + focus-within ring。
  - edit popover: `kivact::view::overlay(root_ref, Placement::Below, editor).offset(4).at_least_anchor_width(true).on_dismiss(..)`、最小幅 `min(400, window 幅 − 余白)` を style で、enter は `Style::enter` で opacity 0→1 + translate_y(−8)→0 + scale 0.95→1、`Transition { duration: 165ms, easing: Easing::CubicBezier(0.24, 1.0, 0.4, 1.0) }`、`kivact::platform::reduced_motion()` 時は無し (touch 環境は opacity のみ ≤165ms)。`dialog(.., DialogKind::Popover)` を使う場合は modal / focus trap が付かない設定であること (Astryx は focus を editor 内へ移すが trap しない、role none)。
  - Field / Operator / enum / relative-date: `kivact::ui::select(..)` (soft の Selector restyle)。string: `kivact::ui::text_input`、number: `kivact::ui::number_input`、string_list / enum_list / entity_list: `soft::tokenizer`、date / date_range / time: kit の DateInput / DateRangeInput / TimeInput 部品 (各節)、Typeahead: kit の typeahead。nested: kit の TreeList 部品 (無ければ `kivact::interaction::use_tree` で組む)。
  - 「幅 < 400px で wrap」: container query は無いので overlay content の `on_layout` 幅 (or 最終幅 = max(anchor 幅, min 幅) を計算) から `wrap(bool)` を切り替える。
  - Enter / Escape: editor root の `on_key_default` (子が Handled::Yes を返したキーは来ない = `defaultPrevented` 相当)。focus 中ノードが button なら Enter は無視。IME 変換中は `KeyEvent` の composing フラグで除外 (UNRESOLVED: kivact の KeyEvent に composition 状態フィールドがあるか未確認; text_input が IME 中のキーを forward しない実装ならそれで足りる)。
  - rAF 後 focus: `on_mount` 内で次フレーム (`kivact::task::spawn(async { sleep(0) })` 相当) に最初の focus 可能子へ `NodeRef::focus()`。
  - 文字列切り詰め: grapheme 単位が理想だが kivact に公開 grapheme API が無いので `char` (Unicode scalar) 単位で切る (Astryx の Intl.Segmenter 非対応時の fallback と同等)。
  - 数値書式 / 日付書式: Astryx の見える token は数値無書式・日付 "MMM d, yyyy" (locale 依存)。kivact::i18n に数値・日付書式 API があればそれを、無ければ en-US 固定書式 (UNRESOLVED: kivact::i18n の Intl 相当未確認)。
- 2 系統の値整形に注意: 画面の token は `PowerSearchTokenValue` (19.4.2)、通知・アクセシブル文字列と公開 `PowerSearchToken` は `formatFilterValue` (数値は Intl + units、date_absolute は時刻付き `{year, month short, day, hour, minute 2-digit}` + timezone、list は `joined.length <= max` なら連結 / 超えたら "{n} items"、entity は "{n} entities"、nested は "{n} filters")。Kivact では両方を 1 モジュールの 2 関数として移植する。

### 19.8 受け入れ確認

- [ ] combobox 名 "Search" (ラベル非表示)、placeholder "Search…"。focus で field 一覧 (group 無し → 見出し付き group)。
- [ ] "tit" 入力で "Title" と "Title contains" … 行、"title contains foo" で `Title contains "foo"` 行、選ぶと popover を経ずに token 追加。
- [ ] `contentSearchFieldKey` 設定時、未知語で先頭に `"word"` 行。
- [ ] field 行を選ぶと token 列の下に幅 = バー幅 (最小 400px) の popover が 165ms で下からふわっと出る (8px 上から)、value editor に focus。Apply は value 未入力で disabled。Enter で保存、Escape で取消、どちらもバー input に focus が戻る。
- [ ] enum の値 / relative date / Typeahead 候補は選んだ瞬間に保存。boolean (is true) は field 選択だけで token 追加。
- [ ] token は "Status: is **Active**" (値が太字)。hover で薄い灰、keyboard focus で token 全体に 2px リング (offset 3px)。クリック / Enter で現在値入りの editor (Delete 有り)。
- [ ] 長い値は "{予算}文字..."、複数値で長ければ "3 items"。
- [ ] `resultCount=12` で end lane に "12 results" (12px secondary)、変更時のみ読み上げ。
- [ ] Clear all で全削除 + "All filters removed"。
- [ ] popover 幅 < 400px (狭い画面) で Field / Operator / Value が折り返す。
- [ ] docsite light/dark: popover 背景 #ffffff / #1b1b1b、radius 12、shadow-low (dark は 1px の白 8% inset 縁)、Apply は黒地白字 / 白地黒字。


---

## 20. Calendar

ソース: `D:\w\meta\astryx\packages\core\src\Calendar\`（`Calendar.tsx` 1429 行、`styles.ts`、`dayCellUtils.ts`、`getInitialFocusDate.ts`、`getStandaloneShortWeekdayNames.ts` + `standaloneShortWeekdayNames.generated.ts`、`hooks/useCalendarDays.ts` / `useCalendarConstraints.ts` / `useCalendarNavigation.ts`、`utils.ts`）。Docs: https://astryx.atmeta.com/components/Calendar 。Stories: `apps/storybook/stories/Calendar.stories.tsx`（Default, WithSelectedDate, RangeSelection, RangeWithValue, RangeWithMaxSpan, RangeWithSpanBounds, TwoMonths, TwoMonthsRangeSelection, TwoMonthsNarrowContainer, MinMaxBoundary, WindowAwayFromToday, WithDateConstraints, WeekdaysOnly, WithWeekNumbers, MondayStart, RTL, RTLRange, AllVariations, Themed*）、`CalendarGridA11y.stories.tsx`。

Calendar は DateInput（popover）、DateRangeInput、DateTimeInput の共通部品でもある。DateInput のタッチ面（MonthScroller）は Calendar の `useCalendarDays` を再利用するが Calendar 本体は使わない（21 章参照）。

### 20.1 概要

- 目的: 月グリッドから単一日付（`mode: 'single'`）または日付範囲（`mode: 'range'`）を選ばせる。予約フロー、スケジューリング、日付フィルタなど「周辺の日付が文脈として見える必要がある」場面向け（doc.mjs `usage.description`）。
- 使う: `min`/`max` で選択可能な窓を限定（未来日だけ、今四半期だけ）。開始・終了が必要なら range。週末・祝日などは `dateConstraints` で無効化し、理由を説明する。月境界をまたぐ選択が多いなら 2 か月並べる。
- 使わない: 誕生日のように遠い過去・未来の日付（テキスト入力の方が速い）。理由の説明なしに大きな日付ブロックを無効化しない。
- 単体コンポーネントとしてページに置ける（背景色を持たない inline-block）。DateInput のポップオーバー内でも同じものが表示される。

### 20.2 Anatomy

DOM / role 構造（`Calendar.tsx:695-787`, `MonthGrid` `:1101-1203`, `DayCell` `:1335-1428`）:

```
div.astryx-calendar  (role なし, data mode=single|range, onKeyDown=Escape で range 進行中を取消)
├─ div header  (flex, space-between)
│  ├─ Button ghost icon-only  label="Previous month"  (.astryx-calendar-nav nav=prev, disabled 状態)
│  │    └─ span(navIcon, RTL で mirror) > Icon chevronLeft sm
│  ├─ span monthYearLabel  "January 2026"（2 か月なら "January 2026 – February 2026"）
│  └─ Button ghost icon-only  label="Next month"   (nav=next)
└─ div monthsContainer (flex-wrap, gap 16)
   └─ [numberOfMonths 個] div monthGrid
       └─ div role="grid" aria-label="January 2026" aria-multiselectable(range のみ true)
           ├─ div role="row" (display: contents)  ← 見出し行
           │   ├─ [hasWeekNumbers] div role="columnheader" > VisuallyHidden "week"(Intl.DisplayNames weekOfYear)
           │   └─ div role="columnheader" × 7   "Su" "Mo" …（CLDR stand-alone short）
           └─ div role="row" × 6（hasVariableRowCount なら 4〜6）
               ├─ [hasWeekNumbers] div role="rowheader"  ISO 週番号
               └─ div role="gridcell" aria-selected(選択/範囲内なら true) × 7
                   ├─ [isInRange]   div rangeBg   (範囲帯, absolute)
                   ├─ [isInPreview] div previewBg (ホバー予告帯, absolute)
                   └─ button type=button data-date="2026-01-15"
                        aria-label="Thursday, January 15, 2026, selected" など
                        aria-disabled / aria-current="date"(今日) / tabIndex roving
                        .astryx-calendar-day [selected|today|disabled|in-range|marker]
                        └─ 日番号テキスト "15"
```

- `hasOutsideDays=false` のとき隣月の日は空の `div role="gridcell"`（ボタン無し）で 7 列幾何を保つ（`:1268-1272`）。
- 月ラベル span には live 属性が無い。月が変わると `useAnnounce()`（polite live region）で `monthYearLabel` を読み上げる（初回マウント時は除外、`:418-426`）。
- 日付ボタンの順序は DOM 順 = 視覚順（RTL では CSS grid が右から並べる）。

### 20.3 API

Astryx props（`Calendar.tsx:118-229`, doc.mjs props）→ Kivact 提案。

| Astryx prop | 型 / 既定 | 意味 | Kivact |
|---|---|---|---|
| `mode` | `'single'｜'range'` / `'single'` | 選択モード | builder 関数を分ける: `soft::calendar(selected)` / `soft::calendar_range(range)`（basic と同じ） |
| `value` | single: `ISODateString｜null`、range: `{start,end}｜null` | 制御値 | `selected: impl Into<Binding<Option<CalendarDate>>>` / `range: impl Into<Binding<Option<DateRange>>>`（`kivact::i18n::{CalendarDate, DateRange}`） |
| `defaultValue` | 同上 | 非制御初期値 | 呼び出し側 `signal(Some(..))` を渡すだけ（専用メソッド不要） |
| `onChange` | single `(iso, Date)`、range `(range)` | 選択確定時 | Binding への書き込みで足りる。追加で `.on_change(impl Fn(CalendarDate)+'static)` / range は `.on_change(impl Fn(DateRange))`（range は 2 クリック目でのみ発火） |
| `numberOfMonths` | `1｜2` / 1 | 並べる月数（範囲外は 1 に clamp） | `.number_of_months(MonthCount::One｜Two)`（enum。usize ではなく 1/2 に限定） |
| `min` / `max` | ISO | 選択可能範囲（境界日を含む） | `.min(CalendarDate)` / `.max(CalendarDate)` |
| `dateConstraints` | `Array<(Date)=>boolean>` | どれか 1 つでも false なら無効 | `.date_constraint(impl Fn(CalendarDate)->bool + 'static)`（複数回呼べて AND 合成） |
| `maxRangeSpan` | number | range: 開始日選択後、開始から `span-1` 日を超える日を無効 | `.max_range_span(u32)` |
| `minRangeSpan` | number / 1 | range: 開始日以外で距離 `< span-1` の日を無効。開始日を再クリック: `min≤1` なら 1 日範囲確定、`>1` なら進行中選択を取消 | `.min_range_span(u32)` |
| `focusDate` + `onFocusDateChange` | ISO | 表示月の制御（両方そろって初めて制御） | `.month(impl Into<Binding<CalendarMonth>>)`（呼び出し側所有の表示月。未指定なら内部 signal） |
| `handleRef.navigateTo(iso)` | imperative | 表示月を移動 | 不要: `.month(binding)` の binding に書けば同じ。 |
| `hasOutsideDays` | bool / true | 隣月の日を表示（灰色、選択不可） | `.show_outside_days(impl IntoValue<bool>)` |
| `hasWeekNumbers` | bool / false | ISO 週番号列 | `.show_week_numbers(bool)` |
| `hasVariableRowCount` | bool / false | false=常に 6 行、true=必要行数 | `.variable_row_count(bool)` |
| `weekStartsOn` | `0..6｜'sun'..'sat'` / 0 | 週の開始曜日。**既定は locale に関係なく日曜** | `.week_start(Weekday)`。既定 `Weekday::Sunday`（Astryx 準拠。basic の「locale の first_day_of_week」既定とは異なるので注意） |
| `ref` | DOM ref | root | 不要（`NodeBuilder` で root を修飾） |
| `xstyle` / `className` / `style` | - | root スタイル | `NodeBuilder` の `.style(..)` に置換。className は廃止 |
| `onKeyDown` 他 DOM handler | - | root に合成 | 廃止（必要なら NodeBuilder の handler） |

テーマターゲット（doc.mjs theming）: `astryx-calendar`(visualProps `mode`)、`astryx-calendar-nav`(`nav`=prev/next, state `disabled`)、`astryx-calendar-day`(states `selected`, `today`, `disabled`, `in-range`, `marker`=`today-only｜today-in-range`)。Kivact では kit の `Tokens` / recipe（`CalendarDayState` に相当する state 構造体）で表す。

提案 builder 例:

```rust
let day = signal(None::<CalendarDate>);
soft::calendar(day)
    .min(CalendarDate::new(2026, 1, 1))
    .date_constraint(|d| !matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday))
    .number_of_months(soft::MonthCount::Two)
    .show_week_numbers(true)
```

### 20.4 Visual spec

トークン解決（neutral, light / dark）: `--color-text-primary` (#000000 / #ffffff)、`--color-text-secondary` (#474747 / #9e9e9e)、`--color-accent` (#1b1b1b / #f1f1f1)、`--color-on-accent` (#ffffff / #111111)、`--color-accent-muted` (#f1f1f1 / #262626)、`--color-overlay-hover` (#0000000D / #ffffff0D)、`--color-overlay-pressed` (#0000001A / #ffffff1A)、`--color-border-emphasized` (#d4d4d4 / #525252)、`--color-icon-secondary` は使わない（nav アイコンは `color="inherit"` = ボタン文字色 text-primary）。`--size-element-sm` 28px、`--size-element-md` 32px、`--spacing-1` 4px、`--spacing-2` 8px、`--spacing-3` 12px、`--spacing-4` 16px、`--radius-full` 9999px、`--radius-element` 10px。タイプ: `--text-body-size` 14px、`--text-label-size` 14px、`--text-supporting-size` 12px、`--font-weight-semibold` 600、`--font-weight-normal` 400（coarse pointer かつ < lg のアダプテーションでは body 16 / supporting 14）。

**寸法スケッチ（1 か月, neutral）**

```
 ← padding 12 →                                         ← 12 →
 ┌──────────────────────────────────────────────────────────┐
 │ [<]        January 2026 (14px/600)               [>]     │ header h=32, marginBottom 8, gap 8
 │                                                          │
 │  Su   Mo   Tu   We   Th   Fr   Sa    ← 12px/400 secondary, 各 32×36 (pb 4)
 │  28   29   30   31  ( 1)   2    3    ← 行 h=32, セル幅 1fr(=32)
 │   4    5 ░░6░░░░7░░░░8░░░░9░ (10)    ← 範囲帯 accent-muted, 端は丸め
 │  …（固定 6 行）                                          │
 └──────────────────────────────────────────────────────────┘
 幅 = 12 + 7×32 + 12 = 248px（min-width 220px）、高さ = 12+32+8+36+6×32+12 = 292px
```

- Root `calendarStyles.calendar`（`styles.ts:33-37`）: `display:inline-block`, padding 12px（全辺）, `min-width:220px`。背景なし（親の背景が見える。DateInput では popover 面 #ffffff / #1b1b1b）。
- Header（`styles.ts:38-44`）: flex, align center, space-between, gap 8px, margin-bottom 8px。
  - Nav ボタン: core `Button` variant ghost、size 既定 md、`isIconOnly` → 32×32（`aspect-ratio 1/1`, padding 0）、radius `--radius-element` 10px、背景 transparent、文字/アイコン色 text-primary、アイコン `chevronLeft`/`chevronRight` size sm = 16×16。hover（hover:hover）: overlay-hover を背景画像として重ね（#0000000D / #ffffff0D）、mouse :active: overlay-pressed（#0000001A / #ffffff1A）+ `scale(0.98)`。disabled（min/max 月に到達）: `opacity 0.5`、cursor default、hover なし。focus-visible: 2px solid `--color-accent`、offset 3px。RTL ではアイコン span に `rtlStyles.mirror`（scaleX(-1)）。
  - Month/year label（`styles.ts:45-51`）: `flex:1`, `text-align:center`, 600, 14px (`--text-label-size`), text-primary。line-height は指定なし（継承; ページ既定 20px と想定）。テキストは `Intl.DateTimeFormat(locale,{year:'numeric',month:'long'})` + stand-alone 大文字化（es fr it nl pl ru uk の言語のみ先頭を大文字化, `utils/plainDate.ts:342-358`）。2 か月なら ` – `（前後スペース付き en dash）で連結。
- Months container（`styles.ts:52-59`）: flex, `flex-wrap: wrap`, gap 16px。2 か月は横幅 ~488px 未満で縦に折り返す（WCAG 1.4.10）。各 monthGrid は `flex:1 1 0`。
- Days grid（`styles.ts:94-100`）: CSS grid `repeat(7, 1fr)`、週番号ありは `auto repeat(7, 1fr)`。行 div は `display:contents`。
- 曜日見出し `dayName`（`styles.ts:78-90`）: width 32, height 36（32 + 4）、padding-bottom 4、中央揃え、12px、400、text-secondary。文字は CLDR stand-alone **short**（英語 "Su Mo Tu We Th Fr Sa"、ja "日 月 火 …"、30 言語の表は `standaloneShortWeekdayNames.generated.ts`。exact → language → en の順で解決）。`weekStartsOn` 分回転。
- 週番号 `weekNumber`（`:101-109`）: 32×32、中央、12px、text-secondary。見出し角 `weekNumberHeader` は幅 32、視覚的に空（VisuallyHidden テキストのみ）。ISO 8601 週番号（その週の木曜基準, `plainDateGetWeekNumber`）。週の代表日は「その行で最初の月内の日」。
- Day cell `cell`（`:132-139`）: relative, flex center, height 32, `isolation:isolate`。幅は 1fr（=32px; 2 か月で広がっても列幅は伸びる）。
- Day button `day`（`:196-229`）: 28×28（`--size-element-sm`）、`border-radius: 9999px`（円）、border なし、padding 0、font 14px（`--text-body-size`）、family inherit、weight 400、`position:relative; z-index:1`。`::before` が上下左右 -2px に広がり 32×32 の当たり判定（隣のセルと隙間なく接する）。cursor pointer（disabled は default）。
- 日の状態（`dayCellTheme` `:247-308`、適用順は `Calendar.tsx:1409-1423`。後勝ち）:

| 状態 | 条件 | 見た目 (light / dark) |
|---|---|---|
| rest | 月内・有効 | 背景 transparent、文字 text-primary (#000000 / #ffffff) |
| hover (hover:hover, 有効のみ) | | 背景画像 overlay-hover。白地上で #f2f2f2、dark popover(#1b1b1b)上で #262626 |
| pressed (mouse :active) | | overlay-pressed。白地 #e6e6e6 / dark #323232。タッチは usePressFeedback（onset 150ms 等、brief の Press feedback 参照） |
| focus-visible | | outline 2px solid accent (#1b1b1b / #f1f1f1)、offset 3px、円に沿う |
| today（単独） | 今日・単一選択されていない・範囲外 | `box-shadow: inset 0 0 0 1px --color-border-emphasized`（#d4d4d4 / #525252）の細リング。文字色・太さは変わらない |
| today in range | 今日・範囲内（端点含む） | `inset 0 0 0 1px --color-text-primary`（#000000 / #ffffff）。端点が今日なら accent 塗り＋この内側リング（on-accent 上に primary 色の 1px リング） |
| selected / range start / range end（endpoint） | single 選択日、または range の start/end | 背景 `--color-accent` (#1b1b1b / #f1f1f1)、文字 `--color-on-accent` (#ffffff / #111111)。hover でも overlay が重なる（light #1a1a1a / dark #f2f2f2 程度、ほぼ不変）。forced-colors: Highlight / HighlightText |
| in range（中間） | start と end の間 | ボタン自体は transparent・text-primary のまま。背後の rangeBg 帯が `--color-accent-muted` (#f1f1f1 / #262626) |
| preview（range: 開始選択後のホバー） | start〜hover 日 | previewBg 帯 `--color-overlay-hover` (#0000000D / #ffffff0D)。range 帯と同じ形状ルール |
| outside（隣月） | `hasOutsideDays` かつ月外 | 文字 text-secondary、`opacity 0.5` だが **outside は常に effectivelyDisabled なので dayDisabled の `opacity 0.3` が後勝ちで実効 0.3**。結果色: 白地 #c8c8c8、dark #1b1b1b 上 #424242。選択・今日・範囲表示は一切付かない（2 か月表示で同日が両ペインに出るため） |
| disabled | min/max 外、constraint、span 制約 | `opacity 0.3`、hover/pressed 画像なし、cursor default。月内 disabled の実効色: 白地 #b3b3b3、dark #1b1b1b 上 #5f5f5f |

- 範囲帯の形（`styles.ts:119-193`, `dayCellUtils.ts:113-159`）: `BAND_INSET = 2px`。帯は top/bottom 2px（高さ 28 = ボタンと同じ）、左右は既定 0（隣セルと連続）。`roundStart` = 範囲開始日 or 行の第 1 列 or 前の日が連続を切る（disabled / outside）→ inline-start 側を 2px 内側に寄せ、start 側 2 角を 9999px で丸める。`roundEnd` 同様に inline-end 側。範囲開始日・終了日では該当側を 2px 内寄せ（＝帯の端がボタン円と一致）。1 日範囲は両端丸め＝円。preview 帯も同じ（`previewStart`/`previewEnd` は常に端を丸め）。帯は logical（inset-inline / border-start-start-radius）なので RTL で自動反転。
- 範囲の連続性: 隣月の日（outside）は帯を持たず、その隣のセルにキャップ（丸め）が付く（#2715）。範囲内の **disabled 日自身には帯が描かれる**（`computeDayCellState.isInRange` は disabled を見ない）が、キャップ判定（`computeDayNeighborContinuity`）は disabled を「連続切れ」とみなすため、disabled 日の両隣のセルはそちら側が丸まる（disabled 日の帯は両隣の丸めと、行端・自身の判定で決まる）。この非対称はそのまま再現すること。
- 2 か月時: 各グリッドは独立。範囲は両グリッドにまたがって描画（outside コピーには付かない）。
- RTL: grid の列順・帯の丸め・nav アイコンが反転。ArrowLeft/Right も反転（useGridFocus）。
- 高さ: 既定 6 行固定で高さ不変（292px）。`hasVariableRowCount` で 4〜6 行。

### 20.5 Motion

- 日ボタンの色: `transition-property: background-color, color`、`--duration-fast` 125ms、`--ease-standard` cubic-bezier(0.24, 1, 0.4, 1)。`prefers-reduced-motion: reduce` で none（`styles.ts:214-219`）。box-shadow（今日リング）と opacity は transition 対象外（即時）。
- hover/pressed overlay（`interactionOverlayStyles.backgroundImage`）: マウスは即時描画。タッチは press model（onset delay 150ms、slop 10px、quick-tap flash 100ms、release fade 200ms cubic-bezier(0,0,0.2,1)）。
- Nav ボタン: core Button の transition（background-image, background-color, color, opacity, transform 125ms ease-standard、reduced で 0s）。press `scale(0.98)`。
- **月送りアニメーションは無い**（`Calendar/` 内に keyframes / transition は day の色以外なし。月が変わるとグリッドは即座に再描画）。月送りの変化は live region の読み上げのみ。範囲帯・preview 帯の出現も即時（transition なし）。
- DateInput 内で使われるときの出現は popover の Layer enter（21.5 参照）。

### 20.6 Behaviour & a11y

キーボード（`useGridFocus`, `hooks/useGridFocus.ts:455-590` + `Calendar.tsx:483-596, 988-1010`）。フォーカスは日 `<button>` に乗る roving tabindex（グリッドごとに 1 つ tabIndex=0）:

| キー | 動作 |
|---|---|
| ArrowRight / ArrowLeft | ±1 日。RTL では左右反転。無効セルはその方向にスキップ。グリッド端を越えると `navigateMonth(±1, focused, 1)`: その方向に最初の有効日を最大 366 日先まで探し、見つからなければ何もしない、min/max を越える候補に当たったら止まる |
| ArrowDown / ArrowUp | ±7 日（同じ列）。無効は 7 日刻みでスキップ。端越えは同上（offset 7） |
| Home / End | 現在行の最初/最後の有効セル |
| Ctrl/Cmd+Home / End | グリッド（その月）の最初/最後の有効セル |
| PageUp / PageDown | 前/次月の同日（月末は clamp）。min/max 外なら移動しない。その日が無効ならその月の最初の有効日、月に有効日が無ければ移動しない。**Shift+PageUp で年移動は無い** |
| Enter / Space | ネイティブ button の click → 選択 |
| Escape | range で開始日選択済み（進行中）のときのみ: 進行中選択を取消（value は変えない）、`preventDefault + stopPropagation`（親 popover を閉じない）。それ以外は伝播（DateInput の popover が閉じる） |
| Tab | グリッドの 1 つの tab stop。ヘッダーの Prev → Next → 各グリッドの tab stop の順 |

- 初期 tab stop（月ごと, `:899-921`）: 選択日（その月にあり有効）＞今日（その月にあり有効）＞その月最初の有効日。
- キーで月をまたぐと: 目標日を `pendingFocus` に置き、表示外の月なら最小移動で表示月を変える（後方なら目標月を先頭、前方なら末尾に）。表示月を持つグリッドが描画後にその日へ tabindex=0 を移してフォーカス。目標日が無効なら月の最初の有効日、それも無ければ最初のフォーカス可能ボタン。制御 `focusDate` の親が月変更を拒否したら pending は破棄（root の blur で目標以外にフォーカスが移ったら破棄）。
- 無効日は `aria-disabled` で**フォーカス可能のまま**（ポインタでフォーカスでき、クリック・ホバーは無視）。矢印キーはスキップ。outside 日も aria-disabled（クリックしても何もしない）。
- Nav ボタン: `min` が現在の先頭表示月と同月以前なら Prev 無効、`max` が最後の表示月と同月以前なら Next 無効。押下で表示月 ±1（日フォーカスは動かさない）。
- 選択:
  - single: クリックで value=その日、`onChange(iso, Date)`。popover なら DateInput が閉じる。
  - range: 1 回目で開始日（`rangeSelectionStart`）を保持し、その日だけ endpoint 表示（rangeStart=rangeEnd）。live 告知 "Start date {date}. Select an end date."。ホバーで preview 帯（開始〜ホバー日、順不同）。2 回目で `{start,end}`（前後を入れ替えて start≤end）を確定し `onChange(range)`、告知 "Selected range: {start} to {end}."。開始日の再クリック: `minRangeSpan ≤ 1` なら 1 日範囲を確定、`> 1` なら取消して "Cleared start date {date}. Select a start date."。開始日選択中は `maxRangeSpan`/`minRangeSpan` が開始日からの距離で他の日を無効化（開始日自身は常に有効）。
- 開く月（`getInitialFocusDate.ts`）: `focusDate` ＞ 選択値（range は start）＞ 今日。今日が `min` より前なら `min` の月、`max` より後なら `max` の月（2 か月なら最後のペインが max の月になるよう 1 か月前倒し、ただし min の月より前にはしない）。
- 公開ロール・名前:
  - grid: `role=grid`, name = 月ラベル（"January 2026"）, range では `aria-multiselectable=true`。
  - columnheader: 曜日 short 名。週番号列見出しは "week"（`Intl.DisplayNames(locale,{type:'dateTimeField'}).of('weekOfYear')`, 無ければ "week"）。
  - rowheader: 週番号。
  - gridcell: `aria-selected=true`（single 選択日・範囲内すべて）。
  - day button の name: 日付を `DATE_FORMAT_WITH_WEEKDAY`（weekday long, month long, day, year → "Thursday, January 15, 2026"）で整形し、状態語を付加: selected `"{date}, selected"`、range start `"{date}, range start"`（進行中の 1 クリック目も）、range end `"{date}, range end"`、1 日完了範囲 `"{date}, range start and range end"`、中間 `"{date}, in range"`。今日は `aria-current="date"`。
- 告知（polite, `useAnnounce`）: 表示月の変化（初回除く）、range の開始/完了/取消。
- 既定テキスト（Astryx key → 提案 Kivact key / English）:
  - `@astryx.calendar.previousMonth` → `soft.calendar.previous_month` "Previous month"
  - `@astryx.calendar.nextMonth` → `soft.calendar.next_month` "Next month"
  - `@astryx.calendar.daySelected` → `soft.calendar.day_selected` "{date}, selected"
  - `@astryx.calendar.dayRangeStart` → `soft.calendar.day_range_start` "{date}, range start"
  - `@astryx.calendar.dayRangeEnd` → `soft.calendar.day_range_end` "{date}, range end"
  - `@astryx.calendar.dayRangeStartAndEnd` → `soft.calendar.day_range_start_and_end` "{date}, range start and range end"
  - `@astryx.calendar.dayInRange` → `soft.calendar.day_in_range` "{date}, in range"
  - `@astryx.calendar.rangeStartAnnounce` → `soft.calendar.range_start_announce` "Start date {date}. Select an end date."
  - `@astryx.calendar.rangeCompleteAnnounce` → `soft.calendar.range_complete_announce` "Selected range: {start} to {end}."
  - `@astryx.calendar.rangeClearedAnnounce` → `soft.calendar.range_cleared_announce` "Cleared start date {date}. Select a start date."
  - 追加（Kivact に aria-current が無いための代替, 21.7 参照）: `soft.calendar.today` "Today"
  - 月名・曜日名・"week" は文言キーではなくロケールデータ（26.7 参照）。

### 20.7 Kivact 実装方針

- 戦略: **NEW kit component**（`kivact::ui` に calendar は無い）。振る舞いは `kivact::interaction::{use_calendar_grid, CalendarPolicy, CalendarSelection, CalendarGrid, CalendarDayState, CalendarSpan}`、日付型は `kivact::i18n::{CalendarDate, CalendarMonth, DateRange, Weekday}`。最も近い既存: `crates/kivact-ui-basic/src/calendar/`（`view.rs` builder、`panel.rs` 月パネルと day cell、`caption.rs` 月名、`state.rs`）。構造はそれを踏襲し、見た目と以下の差分を kit 側で実装する。
- `CalendarGrid` と Astryx の挙動差（kit で吸収すること）:
  1. **PageUp/PageDown + Shift = 年移動**（kivact）。Astryx は Shift なし。kit で Shift 付きを `on_key_default` で先取りして消費するか、そのまま許容する（害は小さい）。推奨: 許容（拡張として）だが E2E では非 Shift のみ確認。
  2. **無効日はフォーカス不可**（kivact）。Astryx はポインタでフォーカス可能・aria-disabled。キーボード挙動（スキップ）は一致するので許容。
  3. **outside 日は kivact ではクリック可能**。Astryx は選択不可。kit の cell 描画で outside の press を無視し、`accessible_disabled(true)` を付ける（`CalendarPolicy::disabled` に「表示中の月の外」を含められないため cell 側で処理）。
  4. **range の 1 クリック目で binding に 1 日範囲を書く**（kivact）。Astryx は 2 クリック目まで onChange しない。kit は grid に kit 内部の `signal::<Option<DateRange>>()` を渡し、`CalendarPolicy::on_commit` で完成時のみ呼び出し側 binding に書く。外部から binding が変わったら内部 signal に反映。これで Escape（kivact は「1 クリック前の範囲に戻す」）も Astryx と同じ見た目になる。
  5. **span 制約は開始日依存**: 内部 signal の「進行中か」と開始日を kit が保持し、`CalendarPolicy::disabled` クロージャ内で読む（reactive に読めることを確認。読めなければ cell 側 `disabled` 上書き）。
  6. **hover preview** は kivact に無い: 各 day node の hover（pointer enter/leave）で kit の `hovered: Signal<Option<CalendarDate>>` を更新し、preview 帯を描く。disabled/outside では更新しない。
  7. 月境界越えの矢印で「有効日を最大 366 日先まで探す」「min/max で止まる」— kivact は「移動は無効日をスキップ」「表示外の月ならページ」なので概ね一致。min/max は `disabled` に含めるので kivact が止まらず無限に探す可能性 → 要確認（UNRESOLVED: kivact のスキップ上限）。
  8. Nav ボタンの min/max 無効化は kivact `previous_month()`/`next_month()` part に無い → kit が `accessible_disabled` + press 無視を付ける。
  9. 月変更の告知: kivact に useAnnounce 相当は live_region。kit で visually hidden な `node().live_region(LiveRegion::Polite)`（`NodeExt::live_region`）に月ラベルを書く（初回は書かない）。range 告知も同じ領域。
  10. `aria-current="date"` は Kivact に無い → 今日の day node に `described_by`（または `has_description`）で "Today"（`soft.calendar.today`）を付ける。name には混ぜない（Astryx の名前と一致させるため）。
  11. `Role::RowHeader` が無い → 週番号セルは `Role::Cell` + name（"week 3" 等ではなく番号のみ）で代替。`ColumnHeader` の週見出しは name を "week" ローカライズ名に。
- 描画: `panel.rs` と同様に「セル = 帯 node（absolute）+ ボタン node」。帯の 4 形状（なし／左丸め／右丸め／両丸め、各 inset 2px）は kivact `CalendarSpan`(Start/Middle/End/None) では不足（行端・連続切れのキャップが必要）→ `dayCellUtils` の `computeRangeRounding` / `computeDayNeighborContinuity` を Rust に移植（純関数、そのまま単体テスト可能）。丸めは `radius_corners`（logical 方向は RTL で入れ替え）。
- 今日リングは inset 1px の ring（`Style::ring` もしくは `border(1, color)` を 28px 円の内側に; border はレイアウトを変えないこと）。endpoint + today in range は accent 塗り＋内側 1px text-primary リング。
- hover/pressed overlay: `Style::hover(style().background(overlay_hover合成色))` / `pressed(..)`。合成色は 20.4 の表の値を使ってよい（背景の上に乗せる半透明でも可）。
- focus ring: 2px accent, offset 3px（basic の `wide_focus_ring` 相当を soft の値で）。
- 曜日見出し: CLDR stand-alone short 表（26.7）。月ラベル: ロケールの "MMMM y" パターン（26.7）。
- 2 か月の折り返し: 親幅 < 488px で縦並び（kivact の wrap レイアウト、または幅を見て `Direction::Column`）。
- 難所: hover-media（`@media (hover:hover)`）→ タッチ入力時は hover を付けない（PointerKind で判定）。forced-colors は対象外でよい。

### 20.8 受け入れ確認

- [ ] `role=grid` name "January 2026"（表示月）、7 個の columnheader（en: Su…Sa）、6 行。`weekStartsOn=Monday` で先頭が "Mo"。
- [ ] 今日のボタンは細い #d4d4d4（dark #525252）内側リング、選択日は #1b1b1b 塗り白文字（dark #f1f1f1 塗り #111111 文字）、28px 円。
- [ ] 隣月の日は薄く（実効 30%）クリックしても値が変わらない。`show_outside_days(false)` で空セル。
- [ ] Prev/Next ボタン name "Previous month"/"Next month"。`min` の月で Prev が 50% 不透明で押せない。
- [ ] 矢印キーで日移動、月末で ArrowRight → 翌月 1 日にフォーカス、表示月が変わり月名が読み上げられる。PageDown で翌月同日（1/31 → 2/28）。Home/End で行頭/行末。
- [ ] range: 1 クリック目で開始日のみ塗り、ホバーで薄い preview 帯、2 クリック目で帯（#f1f1f1 / #262626）が両端丸めで描かれ、change が 1 回だけ発火。逆順クリックで start≤end に入れ替わる。Escape で進行中取消。
- [ ] `maxRangeSpan=7` で開始日から 7 日目以降が無効化。`minRangeSpan=2` で開始日再クリックが取消になる。
- [ ] 日ボタンの名前に ", selected" / ", range start" / ", range end" / ", in range" が付く。
- [ ] disabled 日の両隣で帯が丸まる（週末無効の範囲を選ぶ）。
- [ ] 2 か月表示でヘッダーが "January 2026 – February 2026"、狭い幅で縦に折り返す。
- [ ] 週番号列（ISO）。RTL で列・矢印・帯の丸めが反転。
- [ ] docsite（light/dark）と並べて、ヘッダー高さ・セル 32px 間隔・今日リング・範囲帯の形が一致。

---

## 21. DateInput

ソース: `D:\w\meta\astryx\packages\core\src\DateInput\`（`DateInput.tsx`（PointerDateField + 分岐）、`TouchDateField.tsx`、`NativeDateField.tsx`、`nativeDateSegments.ts`、`MonthScroller.tsx`、`MonthYearWheels.tsx`、`Wheel.tsx`、`monthGeometry.ts`、`tokens.stylex.ts`、`useScrollSettle.ts`、`usePointerDragScroll.ts`、`useOwnScrollGesture.ts`）。共有: `utils/inputPresentation.ts`、`utils/dateParser.ts`、`utils/plainDate.ts`、`Field/inputStyles.stylex.ts`、`Field/inputDisabledFocus.ts`、`Popover/usePopover.tsx`、`BottomSheet/`。Docs: https://astryx.atmeta.com/components/DateInput 。Stories: `apps/storybook/stories/DateInput.stories.tsx`（Default, WithValue, MondayFirstWeek, Formats, NativePickerModes, WithDescription, WithHiddenLabel, Optional, Required, Disabled, DisabledWithMessage, SmallSize, WithMinMax, WithMaxDateInLayout, TwoMonthCalendar, With{Error,Warning,Success}Status, OpenCalendarAudit, AllVariations, Clearable, ClearedCalendarSelection, ClearableWithStatus, StatusVariantComparison, ThemedClearIcon, ThemedCalendarToggleIcon, StylingParts）。

DateRangeInput / DateTimeInput / TimeInput は別章（別担当）。それらが再利用する **MonthScroller / Wheel / MonthYearWheels / tokens / 3 つのスクロール hook / monthGeometry / presentation 解決** はこの章が正（21.10〜21.13）。

### 21.1 概要

- 目的: 日付を**入力（タイプ）するかカレンダーから選ぶ**フォームフィールド。スケジュール、締切、予約日など特定のカレンダー日付が必要な場面。
- 1 コンポーネント 3 面（`DateInput.tsx:1126-1152`）。`presentation` とポインタで決まる:
  - `desktop`（PointerDateField）: タイプ可能なテキスト入力 + カレンダー popover。
  - `sheet`（TouchDateField）: 読み取り専用フィールド + 下からのボトムシート（横スワイプの月スクローラー + 月/年ホイール）。
  - `native`（NativeDateField）: `<input type="date">` で OS ピッカー。
- bestPractices: ラベル・説明を明確に。`min`/`max`/`dateConstraints` で制限。任意項目なら `hasClear`。サーバー保存なら `changeAction` でローディング表示。fine pointer では APG date picker combobox（ArrowDown / Alt+ArrowDown で開く、Escape で閉じてフィールドへ）。短い prefix/suffix は InputGroup 内で。Don't: 日付でない自由文、文脈なしのラベル非表示、カレンダーだけに頼る（タイプの方が速い）、disabled を Tooltip で包む（`disabledMessage` を使う）。

### 21.2 Anatomy

**閉じたフィールド（3 面共通の見た目, `DateInput.tsx:895-1043`）**

```
Field (label / description / status message / labelTooltip / optional・required 表示)  ← Field 章参照
└─ div.astryx-date-input  (input wrapper; size, status, disabled)   ← popover の anchor
   ├─ [InputGroup 内] VisuallyHidden label
   ├─ button  aria-label="Open calendar"｜"Close calendar"  (先頭! タブ順 1 番目; touch/native は tabIndex=-1)
   │    └─ Icon calendar sm secondary  .astryx-date-input-toggle-icon[data-state=expanded|collapsed]
   ├─ input type=text role=combobox  aria-haspopup=dialog aria-expanded aria-controls aria-autocomplete=none
   │     (native 面は type=date + 上に重ねた span 表示テキスト)
   ├─ div role=alert aria-live=assertive (VisuallyHidden) "Invalid date"
   ├─ [hasClear && value && enabled] InputClearButton  "Clear {label}"
   ├─ [busy] Spinner sm (aria-hidden)
   ├─ [status] status icon（statusVariant=tooltip ならフォーカス可能 info ボタン）
   └─ [popover open] Layer: div role=dialog aria-modal aria-label="Choose date" (popover 面)
        ├─ Calendar mode=single（20 章）
        └─ 隠し close ボタン（フォーカス時のみ見える）"Close calendar"
```

**タッチ面のシート（`TouchDateField.tsx:989-1224, 1329-1341`）**

```
BottomSheet label="Choose date" height=hug padding=0
└─ div sheetBody (padding 24 16 16 16)
   └─ div surface (column)
      ├─ div header (h=36)
      │   ├─ button title aria-expanded aria-label="{monthYear}, Choose month and year"  "March 2026 ⌄"
      │   ├─ span[data-arrows] IconButton ghost sm "Previous month" / "Next month"
      │   └─ span[data-action=reset] Button ghost sm "Reset"
      ├─ div weekdays aria-hidden (7 × "Sun" "Mon" …)
      ├─ div body (grid 1 セルに 2 枚重ね, h = 6×44 = 264)
      │   ├─ div[data-panel=calendar]  MonthScroller（21.10）
      │   └─ div[data-panel=wheels] (不透明プレート)  MonthYearWheels（21.11）
      └─ div footer（grid 1 セルに 2 枚重ね）
          ├─ Button primary md 100% "Save"   (calendar 面)
          └─ Button secondary md 100% "Done" (wheels 面)
```

### 21.3 API

| Astryx prop | 型 / 既定 | 意味 | Kivact 提案 |
|---|---|---|---|
| `label` | string（必須） | ラベル。input の名前 | `soft::date_input(label: impl IntoText, value: impl Into<Binding<Option<CalendarDate>>>)` |
| `value` / `onChange` | ISO / `(v｜undefined)` | 値と変更通知（クリアで undefined） | Binding。追加で `.on_change(impl Fn(Option<CalendarDate>))` |
| `changeAction` | `(v)=>void｜Promise` | onChange 後の非同期アクション。完了まで optimistic 値表示＋busy | `.change_action(impl Fn(Option<CalendarDate>) -> impl Future<Output=()> + 'static)`（`kivact::task` で spawn、未完了中は busy） |
| `isLabelHidden` | bool / false | ラベル視覚非表示 | `.label_hidden(bool)` |
| `description` | string | 説明 | `.description(impl IntoText)` |
| `isOptional` / `isRequired` | bool | 表示と aria-required | `.optional(bool)` / `.required(bool)` |
| `isDisabled` | bool / false | 無効（フォーカス可能のまま aria-disabled） | `.disabled(impl IntoValue<bool>)` |
| `disabledMessage` | string | 無効理由 tooltip（上, hover/focus） | `.disabled_message(impl IntoText)` |
| `isLoading` | bool / false | busy。Spinner、aria-busy、入力/選択/クリア不可 | `.loading(impl IntoValue<bool>)` |
| `min` / `max` | ISO | 範囲 | `.min(CalendarDate)` / `.max(CalendarDate)` |
| `dateConstraints` | 関数配列 | 追加無効条件 | `.date_constraint(impl Fn(CalendarDate)->bool + 'static)`（複数可） |
| `placeholder` | string / "Select a date" | 空時 | `.placeholder(impl IntoText)` |
| `size` | `'sm'｜'md'｜'lg'` / md（SizeContext 継承） | 高さ 28/32/36 | `.size(ControlSize::Small｜Medium｜Large)` |
| `status` | `{type:'warning'｜'error'｜'success', message?}` | 枠色・アイコン・メッセージ | `.status(FieldStatus::error("…"))` 等（Field 章の型を共用） |
| `statusVariant` | `'attached'｜'detached'｜'tooltip'` / attached | メッセージ配置 | `.status_placement(StatusPlacement::…)` |
| `width` | number｜string | Field 全体の幅 | `.width(impl Into<Length>)` |
| `labelTooltip` | string | ラベル末尾 info | `.label_tooltip(impl IntoText)` |
| `hasClear` | bool / false | 値があるとき × | `.clearable(bool)` |
| `numberOfMonths` | 1｜2 / 1 | popover のカレンダー月数（sheet/native では無視） | `.number_of_months(MonthCount)` |
| `weekStartsOn` | / 0 (Sunday) | 週開始（native では無視） | `.week_start(Weekday)` 既定 Sunday |
| `format` | `'date'｜'date_long'｜'date_weekday'｜'system_date'｜(iso)=>string` / `date_long` | 確定値の表示形式（入力中テキストには適用しない） | `.format(DateFormat::{Short, Long, Weekday, Iso})` + `.format_with(impl Fn(CalendarDate)->String)`。既定 `Long` |
| `presentation` | `'popover'｜'bottom-sheet'｜'native'｜'adaptive-bottom-sheet'｜'adaptive-native'` / `adaptive-native` | 面の選択 | `.presentation(DatePresentation::{Popover, BottomSheet, Adaptive})`。下記 |
| `nativePicker` | deprecated | 旧名 | **廃止**（互換不要） |
| `containerXstyle/ClassName/Style` | | Field 箱 | `.container_style(Style)`（必要なら）。className 廃止 |
| `controlXstyle/ClassName/Style` | | 枠付きコントロール箱 | NodeBuilder `.style(..)` を control 箱に適用（root は Field） |
| `ref`, `id`, `aria-*`, DOM handlers | | input へ | `ref` → `.node_ref(NodeRef)` を input に。DOM handler 廃止 |

`format` の出力（`utils/plainDate.ts:518-544`, locale en）: `date` = `{month:'short',day:'numeric',year:'numeric'}` "Mar 21, 2026"、`date_long` = `{year,month:'long',day}` "March 21, 2026"、`date_weekday` = `{weekday:'short',month:'short',day,year}` "Sat, Mar 21, 2026"（Astryx doc は "Wed" と書くが 2026-03-21 は土曜）、`system_date` = "2026-03-21"（Intl を通さない）。すべて Gregorian、UTC 固定で整形（タイムゾーン非依存）。

`presentation` の解決（`utils/inputPresentation.ts:117-133`）: popover→desktop、bottom-sheet→sheet、native→native、adaptive-bottom-sheet→coarse?sheet:desktop、adaptive-native（既定）→coarse?native:desktop。coarse = `(pointer: coarse)`（主ポインタ）。Kivact には OS の日付ピッカーも「主ポインタが coarse」の環境値も無いので:
- `DatePresentation::Popover` → desktop、`BottomSheet` → sheet、`Adaptive`（既定）→ **開く操作の PointerKind で決める**（`PointerKind::Touch`（pen は fine 扱い）のタップで開いたら sheet、マウス/キーボードなら popover）。ただし閉じたフィールドの見た目は sheet と desktop で同じだが、desktop はタイプ可能・sheet は readOnly なので、「タッチでフォーカスした時点で readOnly 化」する必要がある → 代案として App/kit レベルの `soft::TouchMode` 設定（`provide` で与える）を既定判定に使い、PointerKind はその補助とする。`native` は Kivact に存在しないので sheet に写像する（Astryx の既定 adaptive-native は「タッチでは OS」→ Kivact では「タッチでは sheet」）。

### 21.4 Visual spec

#### 共通の閉じたフィールド（input wrapper）

Field 共通の input container spec（`Field/inputStyles.stylex.ts:32-81`、Field 章 section 4 参照）をそのまま使用。値（neutral light / dark）:
- box-sizing border-box、display flex、align center、**gap 8px**（`--spacing-2`）、padding 4px 8px（block / inline）、border 1px solid `--color-border-emphasized`（#d4d4d4 / #525252）、radius `--radius-element` 10px、背景 `--color-background-surface`（#ffffff / #262626）。
- 高さ（DateInput 固有 `sizeStyles`, `DateInput.tsx:141-154`）: sm 28px、md 32px、lg 36px。**min-width 180px**（全サイズ）。
- hover（hover:hover, 非 focus-within, 非 disabled）: `box-shadow: inset 0 0 0 2px` + `color-mix(border-emphasized 30%, transparent)` = #d4d4d44D / #5252524D。
- focus-within: border `--color-accent`（#1b1b1b / #f1f1f1）、`inset 0 0 0 2px --color-accent-muted`（#f1f1f1 / #262626）。dark では accent-muted = surface なので内側リングは事実上見えず 1px #f1f1f1 枠のみ。
- disabled（`isDisabled` のみ。busy では付けない）: opacity 0.5、border border-emphasized、box-shadow none。ただし input にキーボードフォーカスがある間（`:has(input:focus)`）は focus 枠を復元（`inputDisabledFocusStyles.inputFocusRing`、`Field/inputDisabledFocus.ts:71-80`）。
- status 枠（`inputStatusBorderStyles` + `FocusWithin`）: error `--color-border-error` = `--color-text-error`（#c0262c / #ffaea7）、warning `--color-text-warning`（#836700 / #eec448）、success `--color-text-success`（#2f7d33 / #90ca90）。focus 中も status 色を保持。status hover: `--shadow-inset-error` inset 2px #de47454D、warning #f8d36a4D、success #2f7d334D（light/dark 同値）。
- transition: border-color, box-shadow 125ms ease-standard（reduced: 0s）。

内容:
- カレンダートグル（先頭, `DateInput.tsx:89-104, 933-965`）: padding 0、背景 transparent、radius 10px、flex center。中身 Icon `calendar` 16×16、色 `--color-icon-secondary`（#6a6a6a / #9e9e9e）。focus-visible で 2px accent offset 3px のリング。disabled/busy で cursor default（アイコン色は変えない; wrapper の opacity で薄くなる）。
- input（`DateInput.tsx:108-138`）: flex 1、min-width 0、border/padding なし、font body family（Figtree）、**14px / line-height 20px**（iOS の coarse で max(16px)）、色 text-primary、背景 transparent、outline none。placeholder 色 text-secondary（#474747 / #9e9e9e）。
- 入力中に解釈不能なテキスト: 文字色 text-secondary（`inputInvalid`）＋ aria-invalid。
- クリアボタン: InputClearButton（高さ 20px のゴースト icon ボタン、close アイコン 16px。Field 章参照）。
- Spinner sm（busy）、status icon（Field 章）。
- 並び（LTR）: `[📅][テキスト……………][×][spinner][status]`。RTL は反転（アイコンは mirror しない）。

#### Desktop popover

- popover 面（`Popover/usePopover.tsx:79-88`）: 背景 `--color-background-popover`（#ffffff / #1b1b1b）、radius `--radius-container` 12px、`--shadow-low`（light: `0 2px 4px #0000000D, 0 4px 8px #0000001A`; dark: `0 2px 4px #00000040, 0 4px 8px #00000066, inset 0 0 0 1px #ffffff14`）。popover 自身の padding なし（Calendar の padding 12px が内側余白）。
- 位置: anchor = input wrapper、placement below、alignment start（inline-start 揃え）、offset 0（フィールド直下にぴったり。ビューポート端では自動フリップ/ガター）。
- 中身: Calendar（20 章）。結果サイズ 248×292（1 か月）。
- 隠し close ボタン: 通常 1×1 clip。フォーカス時に popover 下に中央寄せで現れる Button secondary "Close calendar"（padding-top 4px）。

#### Touch sheet（TouchDateField）— 21.9 で詳述
#### Native field — 21.12 で詳述

### 21.5 Motion

- フィールド枠: border-color, box-shadow 125ms cubic-bezier(0.24,1,0.4,1)、reduced 0s。
- popover 出現（Layer enter, `Layer/layerAnimations.stylex.ts`）: opacity 0→1 + translateY(-8px)→0（placement below なので上から下へ、`--spacing-2`）+ scale 0.95→1、`--duration-fast-max` 165ms、`--ease-standard`、fill backwards。exit: brief の Layer 既定（UNRESOLVED: popover の exit は Layer 章で確認）。reduced: fine pointer はアニメなし、coarse は ≤200ms のフェードのみ。
- Calendar 内部: 20.5（日の色 125ms、月送りアニメ無し）。
- クリアボタン・Spinner: 各コンポーネント章。
- touch 面: 21.9 / 21.10 / 21.11。

### 21.6 Behaviour & a11y（desktop = PointerDateField）

- フォーカス順: トグルボタン → input → クリア → status ボタン。disabled でもトグルは tab stop を保持（aria-disabled）。
- 開く: (a) トグルクリック: 開いて popover 内の最初のフォーカス可能要素 = **"Previous month" ボタン**にフォーカス（`usePopover` の autofocus。テスト `DateInput.test.tsx:1382-1390`）。開いていればトグルで閉じる。(b) input クリック: フォーカスを input に残したまま開く（`skipAutoFocus`）。(c) input で ArrowDown / Alt+ArrowDown（閉じているとき）: フォーカスを input に残して開く。disabled/busy では開かない。
- 閉じる: input で Escape（開いているとき; preventDefault）。popover 内で Escape（focus trap の onEscape）。日を選択（`handleDateSelect` → change → close）。フィールド外のポインタ押下（フィールドとラベルは除外。ラベル押下は input にフォーカスするので「閉じて同じ押下で再オープン」を避ける）。フォーカスが field/popover 以外の要素へ移ったとき（window 切替などで行き先が無い場合は開いたまま）。light dismiss は無効（テキスト入力は invoker になれないため）。
- 閉じた後のフォーカス: フォーカスが「宙に浮いた」（Escape、非フォーカス領域のクリック、日ボタンのアンマウント）場合のみ input に戻す。別コントロールをクリックして閉じた場合は奪わない。
- タイプ入力: 入力中は pendingInput を表示。キーごとに `parseDateInput(text, locale)` し、解釈可能・制約内・現在値と異なる → **即 onChange** し、カレンダー表示月をその月へ移動。解釈不能（空でない）→ 文字色 secondary、aria-invalid=true、assertive alert "Invalid date"。Enter / blur で確定: 空 → クリア（値があれば onChange(undefined)）、解釈可能かつ制約内 → onChange、それ以外 → 黙って元の表示に戻す。外部から value が変わったら pending を破棄。IME 変換中の Enter/Escape は無視（`isImeKeyEvent`）。
- 受理する書式（`utils/dateParser.ts`）: ISO `YYYY-M-D`、英語月名（"January 25, 2026"、"Jan 25 2026"、"25 January 2026"、年なしは今年）、数字 `a/b/YYYY`・`a-b-YYYY`・`a.b.YYYY`（区切り一致必須）、年なし `a/b`。a,b ≤12 の曖昧時は locale が day-first か（`Intl.DateTimeFormat(locale).formatToParts` で day が month より前か）で決める。数字だけ（"2026"）は不可。最後に JS `new Date()` フォールバック（Kivact では不要、削ってよい）。月名はローカライズされない（英語のみ）。
- 確定値の表示: `format`（21.3）。カレンダーで選んだ値も同じ。
- クリア: `hasClear` かつ値ありかつ有効時のみ表示。押すと onChange(undefined) し input にフォーカス（ポインタ押下時は次フレームで `preventScroll`）。
- disabled: `aria-disabled=true` + readOnly、input・トグルともフォーカス可。disabledMessage があれば wrapper に tooltip（placement above、hover とフォーカスで表示、input/トグルの aria-describedby に追加）。開いたままのカレンダーで選んでも拒否（閉じない）。
- busy（isLoading または changeAction 未完了）: aria-busy、Spinner、入力・選択・クリア不可（クリアボタン非表示）、busy 開始を `@astryx.spinner.loading` で告知。disabled の見た目は付けない。
- ARIA: input `role=combobox`、`aria-haspopup=dialog`、`aria-expanded`、`aria-controls`（開いているとき popover id）、`aria-autocomplete=none`、`autocomplete=off`、`aria-required`、`aria-invalid`（status error または解釈不能）、`aria-labelledby`/`aria-describedby`（description、status message、tooltip）。トグル name は開閉で "Open calendar" / "Close calendar"。popover は `role=dialog aria-modal=true` name "Choose date"。
- 既定テキスト（Astryx → 提案 key / English）:
  - `@astryx.dateInput.placeholder` → `soft.date_input.placeholder` "Select a date"
  - `@astryx.dateInput.dialogLabel` → `soft.date_input.dialog_label` "Choose date"
  - `@astryx.dateInput.closeCalendar` → `soft.date_input.close_calendar` "Close calendar"（popover 内の隠し閉じる）
  - `@astryx.dateInput.openCalendar` → `soft.date_input.open_calendar` "Open calendar"
  - `@astryx.dateInput.toggleCalendarClose` → `soft.date_input.toggle_calendar_close` "Close calendar"（トグルの開状態）
  - `@astryx.dateInput.clear` → `soft.date_input.clear` "Clear {label}"
  - `@astryx.dateInput.invalidDate` → `soft.date_input.invalid_date` "Invalid date"
  - `@astryx.dateInput.resetPicking` → `soft.date_input.reset` "Reset"
  - `@astryx.dateInput.savePicking` → `soft.date_input.save` "Save"
  - `@astryx.dateInput.doneChoosingMonth` → `soft.date_input.done_choosing_month` "Done"
  - `@astryx.dateInput.chooseMonthYear` → `soft.date_input.choose_month_year` "{monthYear}, Choose month and year"
  - `@astryx.dateInput.monthWheel` → `soft.date_input.month_wheel` "Month"
  - `@astryx.dateInput.yearWheel` → `soft.date_input.year_wheel` "Year"
  - 共用: `soft.calendar.previous_month` / `soft.calendar.next_month`（シートの矢印）、`soft.spinner.loading`（Spinner 章）。

### 21.7 Kivact 実装方針

- 戦略: **NEW kit component**（`soft::date_input`）。内部は RESTYLE 部品の組み立て: テキスト入力は `kivact::ui::text_input`（native editing と IME は `use_text_edit` 由来; IME 中の Enter 抑止はここで満たされるか要確認）を kit の field wrapper に入れ、popover は `kivact::ui::dialog(label, open).kind(DialogKind::Popover)` を `.attach(wrapper)`（`crates/kivact-ui-basic/src/date_picker.rs` の `Picker::view` と同じ形）、中身は 20 章の `soft::calendar`。ラベル等は `kivact::ui::field` か kit の field（`crates/kivact-ui-basic/src/field.rs`, `recipes_field.rs`, `input_group.rs` が参考）。
- basic の `date_picker.rs` との違い: basic はボタン型トリガー（タイプ不可）。soft は Astryx どおり**タイプ可能な combobox テキスト入力 + 先頭のトグルボタン**。`Calendar::on_commit` + `autofocus` の内部 API の使い方は流用できる。
- 開閉ロジック: `use_popup` / dialog の Dismiss を `{escape: true, outside: false}` にし、外側押下は kit で判定（wrapper とラベルを除外）。input クリック・ArrowDown で開くときは autofocus しない（basic の `autofocus(true)` を条件付きに）。トグルで開くときは最初のフォーカス可能 = Prev ボタン（Astryx 準拠）。
- 閉じた後のフォーカス返却: 「フォーカスがどこにも無い」なら input の `NodeRef::focus`。`FocusReturn` / `PopupCloseReason`（`kivact::interaction`）で理由を見て分岐。
- タイプ解析: `parseDateInput` を Rust に移植（ISO / 英語月名 / 数字 + locale day-first 判定）。day-first 判定は 26.7 のロケール日付パターン表から（Intl 無し）。
- 表示整形: `format` の 3 形式は 26.7 の `soft::i18n::format_date(date, DateStyle, &Locale)` を使う。
- busy/changeAction: 楽観値 `signal` を kit が持ち、future 完了で解除。`accessible_*` に busy が無い → live_region で "Loading" を告知し、Spinner を表示（Kivact に aria-busy 無し: UNRESOLVED/代替）。
- `aria-haspopup`/`aria-controls`/`aria-required` 相当: Kivact `NodeExt` に無い（`expanded`, `controls`, `accessible_invalid`, `described_by` はある）。`controls(popover_ref)` と `expanded(open)` を使い、required は description に含めるか Field の表示のみ。
- 面の切替（Adaptive）: 21.3 の通り PointerKind／kit 設定で決める。
- 難所: `:has()` による disabled+focus のリング復元 → kit で input の focused 状態を読んで wrapper の border を切り替える（`Style::group_focus` 相当があれば利用）。`color-mix` はあらかじめ計算した #RRGGBBAA を使う。

### 21.8 受け入れ確認（desktop）

- [ ] `role=combobox` name = label、空なら placeholder "Select a date"。高さ 32（md）、min-width 180、先頭に 16px カレンダーアイコン、枠 #d4d4d4（dark #525252）、角丸 10。
- [ ] ホバーで内側 2px の薄い枠、フォーカスで黒（dark #f1f1f1）1px 枠 + 内側 #f1f1f1 2px。
- [ ] input クリックで popover が直下・左揃えで開き、フォーカスは input のまま。ArrowDown でも同じ。トグルで開くと "Previous month" にフォーカス。
- [ ] 日をクリック → 値 "March 21, 2026"（date_long）に、popover が閉じ input にフォーカス。
- [ ] "3/21/2026" とタイプした瞬間に change が発火しカレンダーが 3 月へ。"abc" で文字が灰色化し "Invalid date" が告知、blur で元に戻る。en-GB では "03/04/2026" が 3 April。
- [ ] Escape で閉じる、フィールド外クリックで閉じる、ラベルクリックでは閉じない。
- [ ] `clearable` で値ありのとき × が出て、押すと空になりフォーカスが input に戻る。
- [ ] `min`/`max` 外をタイプしても確定されない。カレンダーでは無効表示。
- [ ] disabled: 50% 不透明、フォーカスは可能、開かない、disabledMessage の tooltip が上に出る。loading: Spinner、操作不可。
- [ ] status error で枠 #c0262c（dark #ffaea7）。
- [ ] popover の影・角丸 12・背景（dark #1b1b1b）が docsite と一致（light/dark）。

### 21.9 TouchDateField（bottom-sheet 面）

1. 概要: タッチ用。閉じたフィールドは desktop と**同じ見た目**だがタイプ不可（readOnly, `inputMode="none"`, caret 透明, `user-select:none`, cursor pointer）。タップで BottomSheet を開き、1 画面 1 か月の横スワイプ月スクローラーで日を選ぶ。遠い月はタイトルを押して月/年ホイールで移動。`numberOfMonths` は無視。
2. Anatomy: 21.2 のシート図。トグルボタンは `tabIndex=-1`（input 自体がタブ停止）。
3. API: DateInput と同一（`numberOfMonths` 無視）。Kivact では同じ `soft::date_input` の sheet 面。
4. Visual spec（`TouchDateField.tsx:162-552`。neutral。coarse pointer のときタイプスケールは base16/ratio1.1736 → large 19px/28, body 16/24, supporting 14/20 になる点に注意。以下は fine 値 / coarse 値）:
   - 閉じたフィールド: 21.4 と同じ（`styles.wrapper` gap 8）。
   - シート: BottomSheet（BottomSheet 章参照。背景 `--color-background-surface` #ffffff / #262626、高さ hug、padding 0）。`sheetBody` padding: inline 16px、block-start 24px（グラブハンドルが乗る帯）、block-end 16px。
   - header: flex、align center、space-between、gap 8、**高さ 36px**（`--size-element-lg`）、inline padding なし（グリッドと左端を揃える）。
     - title ボタン: flex、gap 4、高さ 100%、padding-inline 8、margin-inline-start -8（テキストをグリッド端に揃える）、border なし、radius 10、背景 transparent + hover/pressed overlay（`interactionOverlayStyles.backgroundColor`: hover #0000000D / #ffffff0D, mouse :active #0000001A / #ffffff1A。タッチは press model）、文字 text-primary、**17px / 600**（`--text-large-size`, coarse 19px）、nowrap、ellipsis（`titleText` が縮む）。後ろに Icon `chevronDown` 16px secondary（#6a6a6a / #9e9e9e）。focus-visible リング。
     - 月矢印: span flex gap 2px（`--spacing-0-5`）、`margin-inline-start:auto`（右端寄せ）。IconButton ghost sm（28×28; coarse では min 44×44）、chevronLeft/Right 16px（RTL mirror）。範囲端で行き先が無い矢印は **visibility:hidden**（幅は保持、タブ順からも消える）。
     - Reset: Button ghost sm（高さ 28; coarse で min-block 44）label "Reset"。
   - weekdays 行: grid 7 列、高さ 28px（`--size-element-sm`）、中央、12px/400（coarse 14px）、text-secondary。名前は CLDR **abbreviated**（"Sun Mon Tue …"、`DATE_FORMAT_WEEKDAY_ONLY` = `{weekday:'short'}`）。aria-hidden（各日の名前に曜日が含まれるため）。スクローラーの外（スクロールしない）。
   - body: grid（2 パネルを同じセルに重ねる）、高さ `6 × 44 = 264px`、relative。
   - wheels パネル（上層）: 背景 `--color-background-surface`（不透明プレート, #ffffff / #262626）、`isolation:isolate`。
   - footer: padding-top 8、grid 1 列で Save / Done を同セルに重ねる。Save: Button primary md（高さ 32）幅 100%（背景 accent #1b1b1b / #f1f1f1、文字 on-accent）。Done: Button secondary md 幅 100%（背景 `--color-neutral` #0000000F / #ffffff1A、文字 text-primary）。
   - `styles.divider`（1px border 色, margin-top 4）は定義のみで未使用。
5. Motion（すべて `SWAP_DURATION = --duration-fast` 125ms）:
   - wheels パネル（と footer の Done）: `opacity 0↔1` + `visibility`、125ms **linear**（ease-standard ではない: フェードは距離が無いので線形, `:495-501`）。表示中 opacity 1 / visible、非表示 opacity 0 / hidden / pointer-events none。visibility は離散補間なので fade out 中は見えたまま、終了時に hidden。
   - calendar パネル（と Save）: opacity は動かさず `visibility` のみ 125ms で切替（覆い終わった瞬間に hidden、戻るときは最初のフレームから visible）。
   - 月矢印・Reset・weekdays 行: opacity + visibility 125ms linear でフェード（プレートが header/weekdays を覆わないため）。weekdays は hidden でも高さを保持。
   - title の chevron: `transform: rotate(0→180deg)` 125ms `--ease-standard` cubic-bezier(0.24,1,0.4,1)（唯一「移動」するもの）。
   - reduced motion: 上記すべて 0.01s。
   - シート自体: BottomSheet enter = translateY(100%)→0、`--duration-medium` 300ms、ease-standard（exit は別曲線; BottomSheet 章参照）。reduced: coarse で 200ms フェード。
6. Behaviour & a11y:
   - 開く: input のタップ/クリック、トグル、キー ArrowDown / Enter / Space（それ以外のキーでは開かない、IME 中は無視）。毎回 **calendar 面で開く**（前回 wheels でも）。
   - 日のタップで **即 onChange、シートは開いたまま**（訂正しやすい）。Save / グラブハンドル / スクリム / Escape はいずれも「閉じるだけ」（既に確定済み）。
   - Reset: 値をクリアし、今日の月が範囲内かつ現在月と異なれば今日の月へ smooth スクロール（範囲外なら月は動かさない）。disabled なら何もしない。wheels 表示中は非表示。
   - 月矢印: ±1 月（範囲 clamp）、`scrollToMonth(target, 'smooth')`。
   - title: wheels の開閉トグル（`aria-expanded`、name "{monthYear}, Choose month and year"）。wheels の変更は即 MonthScroller を `scrollToMonth(.., 'auto')` で操る（隠れていてもレイアウト保持）。wheels 表示中は scroller からの月変更報告を無視（エコー循環防止）。wheels を閉じたら現在月へ再アサート。Done で wheels を閉じる（calendar に戻る、シートは閉じない）。
   - 表示範囲: `min`/`max` があればその月、無ければ開いた時点の値（無ければ今日）の月 ±600 か月（`DEFAULT_MONTH_REACH`）。アンカーはマウント時に固定。
   - クリア: onChange(undefined) 後、`setTimeout(0)` で input に `focus({preventScroll})`（iOS のスクロール飛び対策）。
   - inert: 非表示側パネル・矢印・Reset に inert（タブ不可・a11y ツリー外）。
   - busy 告知: シートが開いていればシート内に、閉じていれば input に。
   - ARIA: input は desktop と同じ combobox（`aria-expanded`=シート開閉、`aria-invalid` は status error のみ）。シートは dialog name "Choose date"。
7. Kivact 実装方針: シートは `kivact::ui::dialog(..).kind(DialogKind::Drawer(Side::Bottom))`（basic `drawer.rs` のスワイプ閉じも参考）か BottomSheet kit 部品（別章）を使用。2 パネル重ね = `Position::Absolute` で同じ箱に重ね、visibility は「描画するが hit-test/フォーカス/a11y から外す」（Kivact の `show(..)` で消すとスクロール位置が失われるので、opacity 0 + 無効化 + a11y 非公開で残す。inert 相当の API が無ければ `accessible_disabled` + focusable(false) を個別に）。フェードは `Style::transition(TransitionProp::Paint, Transition::tween(125, Easing::Linear))`、chevron 回転は `Transition::tween(125, Easing::CubicBezier(0.24,1.0,0.4,1.0))`。reduced motion は `kivact::platform::reduced_motion()` で 0。
8. 受け入れ確認:
   - [ ] タッチで開くとシートが下から 300ms で出る。header 36px、タイトル 17px 太字（coarse 19px）+ ⌄。
   - [ ] 日タップで値が変わりシートは閉じない。Save で閉じる。
   - [ ] タイトルで wheels がフェード 125ms で重なり、矢印/Reset/曜日行が同時にフェード、⌄ が 180° 回転。Done で戻る。
   - [ ] min の月で Prev 矢印が消える（Next の位置は動かない）。
   - [ ] Reset で値が空になり今月へ戻る。

### 21.10 MonthScroller（タッチ面の月ページャー）

1. 概要: 月を**横一列**に並べたスクロールコンテナ。各ペインはスクロールポートと同じ幅で `scroll-snap-align:start`、`scroll-snap-type: x mandatory` → 必ず 1 か月ちょうどで止まり、高さ一定。ファイル冒頭コメントは「vertically」と書くが実装は inline 軸（`MonthScroller.tsx:93-115` の `overflowX:auto`, `x mandatory`, `pan-x`）。DateTimeInput の touch 面も使う。
2. Anatomy: `div[data-scroller=months]`（scroller）> `div spacer`（幅 = 月数 × ペイン幅）> 表示中心 ±3（`OVERSCAN`）のペインのみ絶対配置（`insetInlineStart = row × paneSize`）。各ペイン `div role=grid aria-label="March 2026" data-month` > 6 × `div role=row`(contents) > 7 × `div role=gridcell aria-selected` > `button data-date data-outside? aria-label="Saturday, March 21, 2026" aria-disabled aria-current=date` > `span puck` > 日番号。曜日見出しはペイン内に無い。
3. API（内部）: `minMonthIndex`, `maxMonthIndex`, `initialMonthIndex`, `onVisibleMonthChange(index)`, `selectedDate`, `today`, `isDateDisabled`, `weekStartsOn`, `onSelect(iso)`, handle `scrollToMonth(index, 'smooth'|'auto')`。月インデックス = `year*12 + (month-1)`（`monthGeometry.ts`）。Kivact: kit 内部 `month_scroller(..)` + `MonthScrollerHandle { scroll_to_month(i32, ScrollBehavior) }`。
4. Visual spec（`MonthScroller.tsx:92-259`）:
   - scroller: 高さ 264（6×44）、box-sizing border-box、横スクロール、縦 hidden、`overscroll-behavior: contain`、スクロールバー非表示、`touch-action: pan-x`。
   - pane: 絶対配置、高さ 264、grid 7 列 × 6 行（`repeat(6, 1fr)` → 44px 行）、幅 = スクロールポート幅（シート幅 − 32 → 1 列 ≈ (幅−32)/7、375px 端末で ≈ 49px）。常に 6 行。
   - day button: セル全面（min 44×44）、padding 0、背景 transparent、文字 text-primary、14px（coarse 16px）/400、`user-select:none`、tap highlight 透明。focus-visible リング（2px accent, offset 3px）。
   - puck（`:170-183`）: **36×36**（44−8）円、border 1px solid transparent、背景 transparent。
     - hover（hover:hover かつ有効）: 背景 overlay-hover（#0000000D / #ffffff0D）。
     - today: border `--color-border-emphasized`（#d4d4d4 / #525252）、文字 600。
     - selected: 背景 accent（hover 中も）、border accent、文字 on-accent、600。
   - outside（隣月）: 文字 text-secondary + opacity 0.5、ただし常に disabled なので **opacity 0.3 が後勝ち**。実効色 白地 #c8c8c8、dark #262626 上 #4a4a4a。
   - disabled（月内）: opacity 0.3（色はそのまま）。実効 #b3b3b3 / dark #676767。
   - RTL: ペインは右から並ぶ（`insetInlineStart`）、scrollLeft は負方向（`rowAtScrollOffset` / `scrollOffsetForRow` が符号を正規化）。
5. Motion:
   - puck: `background-color, color, border-color` 125ms、timing 未指定（CSS 既定 `ease` = cubic-bezier(0.25, 0.1, 0.25, 1)）。
   - ページング: ブラウザのスクロール（指: ネイティブ慣性 + mandatory snap。`scroll-snap-stop` 無しなので強いフリックで複数月進む）。矢印/キー/Reset: `scrollTo({behavior:'smooth'})`（時間はブラウザ依存: UNRESOLVED。Kivact では `ScrollBehavior::Smooth` = 240ms を使う）。OVERSCAN を超える遠距離ジャンプ（wheels など）はアニメ無しで即移動。
   - settle 補正（`:607-645`）: スクロール停止（21.13 の useScrollSettle: `scrollend` または最後の scroll から 220ms 静止＋指が離れている）後、1 フレーム後に再サンプルし、0.5px 以上動いていれば何もしない（まだ移動中）。steer 中なら steer 先、そうでなければ最寄りペインへ。ずれが 1px 以上なら smooth で補正。
   - re-center 中は 1 フレームだけ snap を無効化（WebKit の再スナップ暴走対策; 指操作中は無効化しない）。
   - reduced motion（`:481-520`）: smooth 指定のステップは移動せず、coarse pointer では **opacity 1→0→1 のディップ 200ms、easing cubic-bezier(0.42, 0, 0.58, 1)**、中間（100ms）で瞬間移動。fine pointer では即時。次のステップは進行中のディップを置き換える。
6. Behaviour & a11y:
   - 指: 横スワイプでページング。ジェスチャー共有は useOwnScrollGesture('inline')（21.13）: 6px で軸ロック、`dy ≤ 2·dx`（≈63.4° まで）なら横として確保しシートのドラッグに渡さない、縦優勢ならシートへ（下スワイプで閉じる）。確保したがブラウザがパンしなかった斜めスワイプは 32px 以上で `scrollBy(±paneSize, smooth)` で 1 ページ送る。
   - 表示月の報告: rAF 間引きの scroll で中心行が変わるたびに `onVisibleMonthChange`（タイトル更新）。steer 中（プログラム移動中）は報告しない。ユーザー入力（touchstart, wheel, pointerdown, keydown）で steer 解除。
   - キーボード（日ボタン上）: ArrowLeft/Right ±1 日（RTL 反転）、ArrowUp/Down ±7 日、Home/End で表示週の先頭/末尾列。月をまたぐと `scrollToMonth(smooth)` し、マウント後にその実日（outside コピーでない）へ `focus({preventScroll})`。範囲外の月へは動かない。**PageUp/PageDown は実装されていない**（コメントは言及するが switch に無い）。無効日へもフォーカスは移る（aria-disabled、Enter で選択されない）。
   - tab stop: ペインごとに 1 つ（キーボードで触った日＞選択日＞その月の 1 日）。
   - 選択: タップで（disabled/outside 以外）`onSelect(iso)`。
   - ARIA: grid name = 月ラベル、gridcell aria-selected、button name = weekday long 付き日付、today は aria-current=date。
7. Kivact 実装方針: Kivact には scroll-snap が無い（`scroll(child)` コンテナ、`NodeRef::scroll_to/scroll_by(Point, ScrollBehavior)`、`scroll_offset()` のみ）。2 案:
   - (A) 推奨: **スクロールコンテナを使わず自前ページャー**。全月を描かず、中心 ±3 ペインを `translate_x` で並べ、`kivact::interaction::use_drag(DragCoordinates::..)` で指/マウスのドラッグを追従。リリース時に速度 v(px/s) から投射距離 `v × 0.998/(1−0.998)/1000`（iOS UIScrollView normal 減速率 0.998/ms 相当; Astryx 由来ではない提案値）を足して最寄りペイン（複数ペイン可）を目標にし、`kivact::motion::animated(offset, Transition::spring(Spring{stiffness: 246.7, damping: 31.4, mass: 1.0}))`（response 0.4s, ζ=1 の提案値）で着地。軸ロック（6px, dy ≤ 2dx）と、縦ドラッグはシートへ渡す判定を同じ hook 内で行う。矢印・キー・Reset は同じ spring（または `Transition::tween(240, Easing::CubicBezier(0.24,1.0,0.4,1.0))`）。reduced motion: 200ms の opacity ディップ（cubic-bezier(0.42,0,0.58,1)、中間で瞬間移動）または即時。
   - (B) `scroll` + `scroll_to(Smooth)` と settle 補正（停止判定: `scroll_offset` が 220ms 変化しない & ポインタが離れている → 最寄りペインへ Smooth）。慣性はホスト依存。
   - 仮想化（OVERSCAN 3）・月インデックス計算（`monthGeometry` の純関数）はそのまま移植。`CalendarMonth::weeks(first, 6)` で 6 行固定を得る。日セルの描画・キーボード移動は自前（ここは `use_calendar_grid` を使ってもよいが、「月またぎでページャーをアニメさせる」「outside は disabled」の差を吸収すること）。
8. 受け入れ確認:
   - [ ] 横スワイプで 1 か月ずつ止まり、半端な位置で止まらない。強いフリックで数か月進む。
   - [ ] 斜め 45〜60° のスワイプでも月が変わる。下スワイプはシートを閉じる。
   - [ ] puck 36px、今日は灰枠＋太字、選択は黒塗り（dark #f1f1f1）。
   - [ ] キー ArrowRight で月末から翌月へページが動きフォーカスが移る。

### 21.11 Wheel / MonthYearWheels（月・年ホイール）

1. 概要: タイトルから開く「遠い月への近道」。月ホイールと年ホイールを横に 2 列。スクロールコンテナ + `scroll-snap-align:center` + 端のパディング + 静止時コミット。見た目の円筒（行が中心から離れるほど傾いて薄くなる）は scroll-driven animation。DateTimeInput の時刻ホイールも同じ `Wheel`。
2. Anatomy: `div wheels`（flex, gap 8, 高さ 264）> 2 × `div column`（flex 1 1 0, relative）> `div band`(aria-hidden, 選択帯) + `div role=listbox aria-label="Month"|"Year" aria-activedescendant tabIndex=0`（scroller）> `div role=option id aria-selected aria-disabled` × n > `span itemInner`（テキスト）。
3. API（内部）: `Wheel{label, options:[{value:number,label,isDisabled?}], value, onChange, isActive}`。`MonthYearWheels{monthIndex, minMonthIndex, maxMonthIndex, onChange(monthIndex), monthLabel, yearLabel, isActive}`。月名は `DATE_FORMAT_MONTH_ONLY`（`{month:'long'}` "January"; 2021-xx-15 を UTC 整形）、年は `String(year)`（ロケール数字化しない）。範囲外の月も行として残し disabled（行数が変わるとジャンプするため）。年は min〜max 年すべて、どれか 1 か月でも範囲内なら有効。年変更時は同じ月で範囲 clamp。Kivact: kit 内部 `wheel(label, options, value: Binding<i32>)`。
4. Visual spec（`Wheel.tsx:66-211`, `tokens.stylex.ts`）:
   - 幾何: `daySize` 44px、`wheelItemSize` 28px、`WEEKS_PER_PANE` 6 → scroller 高さ 264、上下 padding `(264 − 28)/2 = 118px`（先頭/末尾行も中央に来られる）。表示 ~9 行。
   - scroller: box-sizing border-box、縦スクロール、横 hidden、`scroll-snap-type: y mandatory`、`overscroll-behavior: contain`、`perspective: 520px`、`transform-style: preserve-3d`、スクロールバー非表示、outline none（focus-visible リングは別途: 2px accent offset 3px）、`touch-action: pan-y pinch-zoom`。
   - item: 高さ 28、flex center、padding-inline 8、`scroll-snap-align: center`、border/背景なし、**17px（`--text-large-size`; coarse 19px）/ 400**、text-primary、nowrap、cursor pointer、user-select none。
     - active（中央帯の下にある行; スクロール中は指に追従）: 色 `--color-text-accent`（#1b1b1b / #f1f1f1）、600。
     - disabled: `--color-text-disabled`（#919191 / #525252）、cursor default。
     - transition: color, font-weight 125ms（timing 既定 ease）。
   - itemInner（falloff はここに掛ける。行自体を transform するとスナップ位置がずれるため）: 100%×100%、overflow hidden、ellipsis、backface hidden。
   - **falloff**（`@supports (animation-timeline: view())` のみ）: keyframes `0%: opacity .3, rotateX(52deg) scale(.86)` / `50%: opacity 1, rotateX(0) scale(1)` / `100%: opacity .3, rotateX(-52deg) scale(.86)`、`animation-timeline: view(y)`、`animation-range: cover 0% cover 100%`、linear、fill both。0% = 行がスクロールポート下端に入る瞬間、50% = 中央、100% = 上端から出る瞬間。cover 区間の長さ = 264 + 28 = 292px。よって行中心と帯中心の距離 d（px, 下が正）に対し `t = |d| / 146`（0〜1）: `opacity = 1 − 0.7t`、`scale = 1 − 0.14t`、`rotateX = 52° × (d/146)`（下の行は手前上端が奥へ倒れる向き = +X 回転）。reduced motion では opacity のみ（`fadeOnly`）。
   - band: absolute、inset-inline 0、top `calc(50% − 14px)`、高さ 28、radius 10（`--radius-element`）、背景 `--color-neutral`（#0000000F / #ffffff1A）、pointer-events none。行テキストの**下**（DOM で scroller より前）。
   - wheels パネル全体は不透明 surface プレートの上（21.9）。
5. Motion / 物理（ネイティブスクロールに委ねる部分と自前部分）:
   - 指: ブラウザの慣性・ラバーバンド・mandatory snap（中央）。
   - 静止判定とコミット（useScrollSettle, 21.13）: `scrollend` または最後の scroll から 220ms 静止、かつ指/マウスドラッグが離れている。静止行 = `round(scrollTop / 28)`。その行が disabled なら最寄りの有効行（同距離なら現在コミット値側）へ smooth スクロールし、2 回目の静止でコミット（UIDatePicker と同じ）。有効かつ値が違えば `onChange`。
   - マウスドラッグ（usePointerDragScroll, 21.13）: 4px スロップ後にドラッグ開始、snap を無効化し `scrollTop = origin − Δy` で追従、離したら最寄り行へ smooth、`scrollend` か 260ms で snap 復帰。ドラッグ後の click は 1 回だけ握りつぶす。
   - 外部から値が変わった/表示されたとき（park）: 静止中のみ。初回表示は即時（auto）、以降は smooth で選択行へ回す。
   - キー/行タップ: `scrollTo(index×28, smooth)` と同時に即 `onChange`。
   - reduced motion（`Wheel.tsx:351-414`）: 自前の smooth スクロール（キー、タップ、無効行からの移動）は移動せず、coarse pointer またはタッチドラッグのリリース起因なら **opacity ディップ 200ms**（現在不透明度→0→1、各区間 easing cubic-bezier(0.42,0,0.58,1)、全体 linear）で中間 100ms に瞬間移動。fine pointer では即時。ディップ中の追加ステップは着地先だけ更新。指のスクロールと慣性はそのまま。
6. Behaviour & a11y:
   - 1 タブ停止の listbox。ArrowDown/Up ±1、PageDown/Up ±10、Home/End 先頭/末尾（移動先 disabled なら何もしない）。`aria-activedescendant` = 中央の行、`aria-selected` = コミット値、`aria-disabled`。
   - 行タップ: その行を中央へ + 即コミット（disabled は無視）。
   - ホイール上の touch はすべてホイールが確保（useOwnScrollGesture('all')）→ シートのスワイプ閉じはハンドル/ヘッダー/スクリムから。ピンチズームはページに残す。
   - `isActive=false`（非表示中）はリスナー・初期スクロールを止める。
7. Kivact 実装方針: スクロールコンテナを使わず**自前の 1 次元ホイール**を推奨。行を `translate_y` で配置し、オフセット `offset: f32` を `use_drag`（垂直）で追従。リリース時: 速度から投射（減速率 0.998/ms の提案値、`projection = v/1000 × 0.998/(1−0.998)`）→ 目標行 `round((offset+projection)/28)` を [0,n−1] に clamp → disabled なら `nearest_enabled_index`（Astryx の `nearestEnabledIndex` を移植）→ `animated(offset, Transition::spring(Spring{stiffness:246.7, damping:31.4, mass:1.0}))` で着地、`on_complete` でコミット。マウスホイール（`on_wheel_default`）は 1 ノッチ 1 行。falloff は毎フレーム各行の d から `opacity`/`scale` を計算（Kivact の transform は 2D のみで rotateX/perspective が無い → `scale_xy(1 − 0.14t, (1 − 0.14t) × cos(52°·d/146))` で縦方向の圧縮として近似し、さらに行の y 位置を `146 × sin(θ)` 系に寄せると円筒感が出る。最低限 opacity と scale_y の近似で可）。band は単純な角丸矩形。listbox は `Role::ListBox` + `active_descendant(NodeRef)` + 各行 `Role::Option` + `selected(..)` + `accessible_disabled(..)`（`kivact::primitive::NodeExt`）。reduced motion: `kivact::platform::reduced_motion()` 時はディップ（`Transition::tween(200, Easing::CubicBezier(0.42,0.0,0.58,1.0))` の opacity）または即時。
8. 受け入れ確認:
   - [ ] 中央に 28px の薄灰帯（#0000000F / #ffffff1A）、中央行は 17px 太字・accent 色、上下の行が徐々に薄く小さくなる。
   - [ ] フリックで慣性が付き、必ず行の中央で止まり、止まった時点で値が変わる（指を置いたまま静止してもコミットしない）。
   - [ ] 範囲外の月で止めると最寄りの有効月まで自動で回る。
   - [ ] キー ↑↓ PageUp/Down Home End が効く。マウスでドラッグできる。

### 21.12 NativeDateField / nativeDateSegments（OS ピッカー面）

1. 概要: `<input type="date">` に委ね、OS が描く（iOS ホイール、Android ダイアログ）。フィールドは他の Astryx 入力と同じ外見で、表示テキストは DateInput 自身が上に重ねて描く（`format`/`placeholder` を効かせるため）。`numberOfMonths`/`weekStartsOn` は無視、`min`/`max` は属性として渡すが iOS は強制しないのでコミット時に検査（`dateConstraints` も同様）。
2. Anatomy: wrapper > button(tabIndex=-1, "Open calendar") > Icon calendar（data-state 常に collapsed）／ span slot(relative, flex 1) > input type=date（非制御, 値は ISO）+ span overlay(aria-hidden, absolute, 表示テキスト) ／ alert live region ／ クリア ／ Spinner ／ status。
3. API: DateInput と同じ。
4. Visual spec（`NativeDateField.tsx:93-250`）: wrapper は 21.4 と同じ（gap 8）。input は appearance none、高さ 1 行（`1lh`, fallback 14×1.4286=20px）、UA のカレンダーグリフ非表示、`::-webkit-date-and-time-value` / `::-webkit-datetime-edit` の余白 0・左寄せ。overlay 表示中は input の文字を透明化（`color` と `-webkit-text-fill-color` を transparent）。overlay: 両端 inset 0（はみ出さない）、block 表示、14px/20px（iOS coarse 16px）、nowrap、ellipsis、pointer-events none。値あり text-primary、プレースホルダ text-secondary、拒否中の値 text-secondary。
5. Motion: フィールド枠の 125ms のみ。ピッカーは OS。
6. Behaviour & a11y: トグル/タップで `focus()` + `showPicker()`（失敗は無視）。値変化は React onChange と native input/change の両方で拾い、同じ生値は 1 回だけ処理。解析可能で制約違反 → 拒否値を保持し assertive "Invalid date"、blur で元に戻す（拒否状態も解除）。空 → クリア。フォーカス中は外部値を書き戻さない（iOS でシートが切り離されるため）。エンジン判定（`nativeDateSegments.ts`）: 画面外に type=date を作り `::-webkit-datetime-edit` に padding 100px を当てて幅が増えれば「セグメント編集可能」（Chromium/macOS WebKit）、`::-webkit-date-and-time-value` で増えれば「ピッカーのみ」（iOS/Android）、どちらでもなければ unknown → coarse でなければ編集可能とみなす（Firefox）。編集可能エンジンではフォーカス中は overlay を消し、ネイティブのセグメントを見せる。クリア後はフォーカスを戻さない（ピッカーが再度開くため）。busy 中もピッカーは閉じない（`isEffectivelyDisabled = isDisabled || isLoading`、changeAction 中は commit ガードのみ）。
7. Kivact 実装方針: **covered by another component** — Kivact に OS 日付ピッカーは無い。`DatePresentation` に Native を設けず、Astryx の `native`/`adaptive-native` は sheet（21.9）に写像する。「セグメント編集」の体験が欲しい場合は将来 `input_otp` 風の年/月/日セグメント入力（`crates/kivact-ui-basic/src/input_otp.rs` が参考）を検討（範囲外）。
8. 受け入れ確認: Kivact では該当なし（`presentation(Native)` が存在しないことのみ確認）。

### 21.13 共有: tokens.stylex.ts / monthGeometry / useScrollSettle / usePointerDragScroll / useOwnScrollGesture

1. 概要: タッチ面（DateInput, DateTimeInput, TimeInput, DateRangeInput の touch field）の幾何定数とスクロール挙動。Kivact では kit 内部モジュール（例 `soft::date::touch`）として 1 か所に置き、他の日付・時刻コンポーネントが共有する。
2. Anatomy: なし（定数と hook）。
3. API / 定数（そのまま Rust 定数に）:
   - `tokens.stylex.ts`: `WEEKS_PER_PANE = 6`、`daySize = 44px`（タッチ最小ターゲット; テーマで変えさせない）、`wheelItemSize = 28px`、`paneBlockSize = 6×44 = 264px`、`paneRows = repeat(6, 1fr)`、`wheelEdgePadding = (264−28)/2 = 118px`。
   - `monthGeometry.ts`: `DEFAULT_MONTH_REACH = 600`、`to_month_index(y,m) = y*12 + (m−1)`、`from_month_index(i) = (floor(i/12), ((i%12+12)%12)+1)`、`clamp_index`、`row_at_scroll_offset(offset, pane, rows, rtl) = clamp(round((rtl?−offset:offset)/pane), 0, rows−1)`（pane≤0 なら 0）、`scroll_offset_for_row(row, pane, rtl) = ±row*pane`、`pane_window(center, rows, overscan) = [max(0,c−o), min(rows−1,c+o)]`。
   - `useScrollSettle`: `SCROLL_QUIET_MS = 220`。scroll ごとに再武装、touchstart で停止判定を保留、touchend/touchcancel で 220ms 再武装、マウスドラッグ保持中（`isHeldRef`）は静止とみなさない、ドラッグ解放（pointerup）で再武装、`scrollend` があれば即 settle（保持中を除く）。1 ジェスチャー 1 回だけ `onSettle`。`isAtRestRef` を公開（移動中に位置を書き換えないため）。
   - `usePointerDragScroll`: mouse の主ボタンのみ。`DRAG_SLOP = 4px`、`SETTLE_FALLBACK_MS = 260`。pointerdown は stopPropagation（シートのドラッグに渡さない、ただしキャプチャはスロップ超過後）、ドラッグ中 snap 無効・user-select none、解放で最寄り行へ smooth、`scrollend` または 260ms で snap 復帰、ドラッグ直後の click を 1 回キャプチャ段階で抑止。
   - `useOwnScrollGesture(axis)`: `'all'`（ホイール）は touchstart で即確保（stopPropagation）。`'inline'`（月スクローラー）は `AXIS_LOCK_SLOP = 6px` 超えの最初の move で `dy ≤ dx × VERTICAL_DOMINANCE(2)` なら確保（≈63.4°まで横扱い）、それ以外はシートへ。確保したのに要素が 1px もパンしなかった場合、横移動 `≥ SWIPE_DISTANCE(32px)` なら `onSwipe(direction)`（左へ動かした = +1）。preventDefault はしない（passive）。touchend/cancel は伝播させる。
4. Visual spec: 3 の幾何値。
5. Motion: 3 の時間定数。ブラウザの smooth スクロール時間は UNRESOLVED（Kivact の `ScrollBehavior::Smooth` 240ms で代替）。reduced motion のディップ: 200ms、cubic-bezier(0.42, 0, 0.58, 1)（`motion/uikit/timing.ts` `reducedMotionCrossFade`）、coarse またはタッチリリース時のみ、fine は即時（`motion/reducedMotion.ts:79-104`）。
6. Behaviour & a11y: 3 の通り。iOS 固有の回避（scrollend 無し、snap index 再スナップ、momentum の不規則イベント）は Kivact の自前物理（21.10/21.11 案 A）では不要になる。ただし「指が触れている間・慣性中はコミットしない」「プログラム移動のエコーを報告しない（steer）」「親シートとジェスチャーを軸で分け合う」という**意味論は維持**すること。
7. Kivact 実装方針: ジェスチャーは `kivact::interaction::use_drag`（`DragPhase`, `DragUpdate` から位置と時間を取り速度を計算; 時刻は `kivact::primitive::clock_now`）。シート側（BottomSheet/drawer）とは、同じドラッグを両方が取らないよう kit 内で「子が確保したら親は無視」のフラグ（コンテキスト）を共有する（DOM の stopPropagation の代わり）。アニメは `kivact::motion::{animated, Transition, Spring, Easing}`。
8. 受け入れ確認: 21.10 / 21.11 の項目に含まれる。単体テスト: `month_index` 往復、`row_at_scroll_offset` の RTL 符号、`pane_window` の端、`nearest_enabled_index` の同距離タイブレーク。


---

## 22. DateRangeInput

ソース: `packages/core/src/DateRangeInput/DateRangeInput.tsx`(891 行)、`DateRangeInput.doc.mjs`、`DateRangeInput.test.tsx` / `DateRangeInput.fr10.test.tsx`、stories `apps/storybook/stories/DateRangeInput.stories.tsx`。ポップオーバー内の月グリッドは `Calendar`(section 20)の `mode="range"`、外枠は Field 共通の input container(section 4)。Docs: https://astryx.atmeta.com/components/DateRangeInput

### 22.1 概要

- 開始日と終了日を 1 つのフィールドで選ぶ。ボタン型トリガーを押すと、2 か月並びのカレンダー(既定 `numberOfMonths=2`)と任意のプリセット列を持つポップオーバーが開く。用途はデータの期間フィルタ、レポート期間、分析ダッシュボード、予約フロー。
- **使う**: 「Last 7 days」のような頻出範囲があるときはプリセットを付ける。`min`/`max` で選択可能範囲を絞る。`hasClear`(既定 true)は残し、フィルタを戻せるようにする。ラベルと説明で何の期間か明示する。
- **使わない**: 単一日付で済む場合(DateInput、section 21 を使う)。周囲の文脈なしにラベルを隠すこと。無効化理由を `Tooltip` で包んで出すこと(`disabledMessage` を使う)。
- 値の流れ: 1 回目のクリックはコンポーネント内部の「選択中アンカー」になるだけで、`onChange` は**呼ばれない**。2 回目のクリックかプリセット押下で `{start, end}` を確定し、`onChange` → `changeAction` の順に呼んでからポップオーバーを閉じる。

### 22.2 Anatomy

```
Field (label / description / required / status message)            ← Field 共通 (section 4)
└─ div.control  [themeProps 'date-range-input' size,status,disabled]  ← inputWrapperStyles.base + sizeStyles
   ├─ button[aria-label="Open calendar"|"Close calendar"][tabindex=-1]  ← カレンダーアイコン (16px, secondary)
   ├─ button#id  (トリガー, the semantic control)
   │     aria-label="{label}: {range|placeholder}"  aria-haspopup="dialog"
   │     aria-expanded  aria-controls=<popover id while open>  aria-describedby  aria-invalid  aria-busy  aria-disabled
   │     text = "Jan 5 – Jan 9" | placeholder
   ├─ InputClearButton (hasClear && value!=null && !disabled/busy)  aria-label="Clear {label}", tooltip "Clear"
   ├─ VisuallyHidden#triggerNameID  (caller aria-labelledby がある時だけ)
   ├─ Spinner sm aria-hidden (busy)
   └─ status icon (attached/tooltip のみ)
VisuallyHidden#requiredID "Required"  (required 時。button は aria-required を持てないため describedby で伝える)
Popover (usePopover: role=dialog aria-label="Choose date range", focus trap, 隠し Close ボタン "Close calendar")
└─ div.popoverLayout (display:flex, 横並び)
   ├─ div[role=group][aria-label="Preset date ranges"]  (presets.length>0)  [themeProps 'date-range-input-presets']
   │    └─ button × N  aria-current="true"(適用中)  aria-disabled(無効)  [themeProps 'date-range-input-preset' selected,disabled]
   └─ Calendar mode="range" (開いている間だけマウント)  ← section 20
        header: [‹ Previous month] "January 2026 – February 2026" [Next month ›]
        months: grid[role=grid aria-multiselectable=true aria-label="January 2026"] × numberOfMonths
```

ASCII(md, 2 か月 + プリセット):

```
┌────────────────────────────────┐
│ [cal] Jan 5 – Jan 9        (x) │  h=32, min-w=180
└────────────────────────────────┘
 ┌──────────────┬─────────────────────────────────────────────────────┐
 │ Last 1 day   │ (‹)        January 2026 – February 2026          (›) │
 │ Last 3 days  │  Su Mo Tu We Th Fr Sa        Su Mo Tu We Th Fr Sa    │
 │▓Last 7 days ▓│   …  (●━━━━━━━━●)  …           …                    │
 │ This month   │                                                     │
 └──────────────┴─────────────────────────────────────────────────────┘
   ▓ = 適用中プリセット (accent-muted)   ●━━● = 端点 solid + 帯 accent-muted
```

### 22.3 API

| Astryx prop | 型 / 既定 | 意味 | Kivact |
|---|---|---|---|
| `label` | string(必須) | ラベル。トリガー名は `"{label}: {値 or placeholder}"` | `date_range_input(label, value)` 第 1 引数 |
| `value` | `DateRange \| null`(必須) | 確定済み範囲 | 第 2 引数 `impl Into<Binding<Option<DateRange>>>`(`kivact::i18n::DateRange`) |
| `onChange` | `(v) => void`(必須) | 確定時 / クリア時(null) | Binding への書き込みで表現。追加通知は `.on_change(impl Fn(Option<DateRange>))` |
| `changeAction` | `(v) => void\|Promise` | onChange の後に走る非同期処理。保留中は楽観表示 + busy | `.change_action(kivact::task::Action<T, Option<DateRange>>)`。`pending_len()>0` を busy とし、その間は表示を新値で保つ |
| `isLoading` | bool / false | busy(スピナー + 操作不可) | `.loading(impl IntoValue<bool>)` |
| `isLabelHidden` | bool / false | ラベル視覚非表示 | `.label_hidden(bool)` |
| `description` | string | ラベル下の補足 | `.description(text)` |
| `isOptional` / `isRequired` | bool / false | 排他。required は VisuallyHidden "Required" を describedby | `.optional(bool)` / `.required(bool)` |
| `isDisabled` | bool / false | 無効。ネイティブ disabled は使わず `aria-disabled`、フォーカス可能 | `.disabled(impl IntoValue<bool>)`(NodeBuilder の disabled ではなくフォーカス可能 disabled) |
| `disabledMessage` | string | isDisabled 時にツールチップ + describedby | `.disabled_message(text)` |
| `min` / `max` | ISODateString | 選択可能範囲(両端含む) | `.min(CalendarDate)` / `.max(CalendarDate)` |
| `dateConstraints` | `Array<(Date)=>bool>` | どれかが false なら無効日 | `.date_constraint(impl Fn(CalendarDate)->bool + 'static)`(複数回呼べる) |
| `maxRangeSpan` | number | 両端含む最大日数。アンカー確定後、`|距離| > max-1` の日を無効化 | `.max_range_span(u32)` |
| `minRangeSpan` | number / 1 | 両端含む最小日数。アンカーから `0<距離<min-1` を無効化。アンカー自身は常に有効 | `.min_range_span(u32)` |
| `presets` | `Array<{label, getRange}>` | サイドバーのクイック選択 | `.presets(impl IntoValue<Vec<DateRangePreset>>)`、`DateRangePreset::new(label, impl Fn()->DateRange)` |
| `hasClear` | bool / **true** | 値がある時にクリアボタン | `.clearable(impl IntoValue<bool>)` |
| `placeholder` | string / "Select date range" | 空表示 | `.placeholder(text)` |
| `size` | `'sm'\|'md'\|'lg'` / 'md'(SizeContext 継承) | 高さ 28/32/36 | `.size(ControlSize::Small/Medium/Large)` |
| `status` | `{type, message?}` | warning/error/success | `.status(impl IntoValue<Option<FieldStatus>>)`(section 4 の型を共有) |
| `statusVariant` | attached/detached/tooltip / attached | ステータス文言の配置 | `.status_placement(StatusPlacement)` |
| `labelTooltip` | string | ラベル末尾の info アイコン | `.label_tooltip(text)` |
| `numberOfMonths` | 1\|2 / **2** | 月数 | `.months(MonthCount::One/Two)`(または `u8` 1/2 に clamp) |
| `weekStartsOn` | 0–6 / 'sun'…'sat' / 0 | 週頭 | `.week_start(Weekday)` |
| `width` | SizeValue | Field 全体の幅 | `.width(impl Into<Length>)` |
| `ref`, `id`, `aria-*`, `data-*`, DOM ハンドラ | — | トリガーに渡る。コンポーネント側ハンドラの**後**に走り、既定動作を取り消せない | `NodeBuilder` 修飾子をトリガー node に適用(`.test_id`, `.name` 等)。`onClick` 相当は `.on_press(..)` を内部処理の後に chain |
| `aria-label` / `aria-labelledby` | — | aria-label はトリガー名を置換、labelledby は内部名の**後**に連結 | `.name(..)` が置換、`.labelled_by(node_ref)` は追加 |
| `containerXstyle/ClassName/Style` | — | Field 箱のスタイル | `.field_style(Style)`(InputGroup 内では無効) |
| `controlXstyle/ClassName/Style` | — | 枠付き control 箱 | `.style(..)`(control 箱に適用) |
| root `xstyle/className/style` | — | 受け付けない(FR10) | 提供しない |

### 22.4 Visual spec

**Control 箱**(`DateRangeInput.tsx:701-729`、Field 共通 container spec = section 4。`inputStyles.stylex.ts:32-81`)

| 項目 | 値 |
|---|---|
| layout | `display:flex; align-items:center; gap 8px (--spacing-2); position:relative; z-index 1; box-sizing border-box` |
| padding | block 4px (--spacing-1) / inline 8px (--spacing-2) |
| 高さ | sm 28px (`--size-element-sm`) / md 32px / lg 36px(`DateRangeInput.tsx:211-224`) |
| min-width | 180px(全サイズ) |
| border | 1px solid `--color-border-emphasized` (light #d4d4d4 / dark #525252) |
| radius | `--radius-element` 10px |
| 背景 | `--color-background-surface` (light #ffffff / dark #262626) |
| hover(hover:hover、非 focus-within、非 disabled) | `box-shadow: inset 0 0 0 2px color-mix(--color-border-emphasized 30%, transparent)` = light #d4d4d44D / dark #5252524D |
| focus-within | border `--color-accent`(light #1b1b1b / dark #f1f1f1)+ `inset 0 0 0 2px --color-accent-muted`(light #f1f1f1 / dark #262626)。トリガーがクリックでフォーカスされても出る(`:focus-within`) |
| disabled / busy | `opacity 0.5`、border emphasized、hover リングなし。ただしフォーカス中は `inputDisabledFocusStyles.withinRing` で focus-within リングを復元(`Field/inputDisabledFocus.ts` withinRing) |
| status=error | border `--color-border-error`(= text-error: light #c0262c / dark #ffaea7)、hover `inset 0 0 0 2px #de47454D`(light/dark 共通)、focus-within 時も border は error 色のまま(内側リングは accent-muted) |
| status=warning | border light #836700 / dark #eec448、hover `inset 0 0 0 2px #f8d36a4D` |
| status=success | border light #2f7d33 / dark #90ca90、hover `inset 0 0 0 2px #2f7d334D` |
| 並び(LTR) | [calendar icon][trigger flex:1][clear][spinner][status icon]。RTL は flex の向きで鏡像。アイコン自体は反転しない |

**カレンダーアイコンボタン**(`:145-162, 730-753`): 枠・背景なし、padding 0、radius 10px、16px(`Icon size="sm"` = 1rem)の `calendar` グリフ、色 `--color-icon-secondary`(light #6a6a6a / dark #9e9e9e)。`tabIndex=-1`(タブ停止しない)、focus-visible 時のみ共通フォーカスリング(2px solid `--focus-outline-color` = accent #1b1b1b / #f1f1f1、offset 3px)。disabled でも見た目同じ(cursor default のみ)。

**トリガー**(`:109-144`): `flex:1; min-width:0`、枠/背景/padding なし、`--font-family-body`(Figtree)、body 14px / 行高 20px(`--text-body-size`/`--text-body-leading` 1.4286)、色 `--color-text-primary`(light #000000 / dark #ffffff)、`text-align:start; white-space:nowrap; overflow:hidden; text-overflow:ellipsis`、outline none(リングは箱の focus-within が担う)。空のときは placeholder 色 `--color-text-secondary`(light #474747 / dark #9e9e9e)。iOS(coarse + webkit-touch-callout)のみ `max(16px, body)`。

**表示文字列**(`formatRangeDisplay :226-237`): start と end が両方「今年」なら `{month:'short', day:'numeric'}`(例 "Jan 5")、それ以外は `{month:'short', day:'numeric', year:'numeric'}`(例 "Jan 5, 2025")。区切りは前後スペース付き en dash `" – "`。1 日範囲でも "Jan 5 – Jan 5"(DateRangePicker basic の単日省略とは異なる)。

**クリアボタン**: `InputClearButton`(section 4)= ghost sm Button、高さ 20px、`close` 16px secondary、tooltip "Clear"。coarse pointer で ::after により上下左右 2px ヒット拡張。

**Spinner**: sm(直径 10 + stroke 2×2 = 14px 箱)、Spinner section 参照(525ms linear 無限)。

**ポップオーバー面**(`Popover/usePopover.tsx:79-88`): 背景 `--color-background-popover`(light #ffffff / dark #1b1b1b)、radius `--radius-container` 12px、`--shadow-low`(light `0 2px 4px #0000000D, 0 4px 8px #0000001A` / dark `0 2px 4px #00000040, 0 4px 8px #00000066, inset 0 0 0 1px #ffffff14`)、面自体の padding 0。配置 `placement:'below', alignment:'start'`(control 箱の左下、RTL では右下)、アンカーとの間隔 4px(`--spacing-1`)、ビューポート内に収まるよう最大サイズ制限。

**プリセット列**(`:166-208`)

| 項目 | 値 |
|---|---|
| 列 | `flex-direction:column; gap 4px; padding 12px (--spacing-3); min-width 140px; border-inline-end 1px solid --color-border-emphasized (#d4d4d4 / #525252)` |
| ボタン rest | `display:block; width:100%; padding 4px 8px; radius 10px; 背景 transparent; font body family; 14px/20px(label size/leading); weight は未指定 = 400; 色 text-primary; text-align start` |
| hover(hover:hover、非 disabled) | 背景 `--color-overlay-hover`(light #0000000D / dark #ffffff0D) |
| pressed | 変化なし(press feedback 未適用) |
| 適用中(`value` と完全一致、aria-current) | 背景 `--color-accent-muted`(#f1f1f1 / #262626)、文字 `--color-accent`(#1b1b1b / #f1f1f1)。hover でも背景は変わらない(後勝ち) |
| disabled | 文字 `--color-text-disabled`(#919191 / #525252)、背景 transparent、hover なし |
| focus-visible | 共通フォーカスリング |

**Calendar(range)内容**: レイアウトは section 20。range 特有の差分のみ以下(`Calendar/styles.ts`, `Calendar.tsx:1060-1429`, `dayCellUtils.ts:42-160`)。

- ルート: `padding 12px; min-width 220px; display:inline-block`。ヘッダーは 1 行のみ: 前月 ghost icon ボタン / 中央ラベル / 次月ボタン。2 か月時のラベルは `"January 2026 – February 2026"`(各月 standalone 表記を `" – "` で連結、`Calendar.tsx:401-407`)、weight semibold 600、14px、text-primary。
- 月の並び: `display:flex; flex-wrap:wrap; gap 16px (--spacing-4)`、各月 `flex:1 1 0`。2 か月で約 488px(12+224+16+224+12)。それより狭いと 2 段に折り返す。
- セル: 高さ 32px、列幅 32px(1fr、曜日見出し 32px)。日ボタン 28×28 円形(radius full)、14px。
- **範囲帯**: `isInRange` のセルの背面に絶対配置の帯。`top/bottom 2px`、左右は 0(隣セルと連続)。色 `--color-accent-muted`(light #f1f1f1 / dark #262626)。端(開始/終了)、行の先頭/末尾列、隣が無効日・月外日で途切れる所では、その側を `inset 2px` + `radius 9999px` で丸める → 端点では帯の丸キャップが 28px 円と一致する。
- **端点**: start/end の日ボタンは solid `--color-accent`(#1b1b1b / #f1f1f1)+ 文字 `--color-on-accent`(#ffffff / #111111)。強制カラーでは Highlight/HighlightText。
- **ホバープレビュー**: アンカー選択中に有効日へホバーすると、アンカー〜ホバー日の区間に `--color-overlay-hover`(#0000000D / #ffffff0D)の帯を描く(同じ inset/丸め規則、`previewStart/previewEnd` は常に端を丸める)。アンカー = ホバー日ならプレビューなし。
- **選択中の表示**: アンカー確定後は、確定済み `value` の範囲は描かれず、アンカー 1 日だけが「start=end」の端点(solid 円 + 帯キャップ)として描かれる(`Calendar.tsx:1073-1077`)。
- today: 単独なら `inset 0 0 0 1px --color-border-emphasized`(#d4d4d4 / #525252)、範囲内なら `inset 0 0 0 1px --color-text-primary`(#000000 / #ffffff)。端点が today の場合も範囲内扱いで text-primary リング(端点 solid の上に描かれる)。
- 無効日(min/max/constraints/span): `opacity 0.3`、hover overlay なし、クリック/ホバー無視。`maxRangeSpan` / `minRangeSpan` 由来の無効はアンカーがある間だけ適用(`useCalendarConstraints.ts:123-137`)。
- 月外日: opacity 0.5、text-secondary、クリック不可(aria-disabled)。
- 初期表示月: `value.start` の月 → 無ければ今日。今日が `min` より前なら `min` の月、`max` より後なら `max` の月を**最後のペイン**に置く(2 か月なら `max-1` か月から、ただし `min` の月より前にはしない)(`getInitialFocusDate.ts:57-93`)。

### 22.5 Motion

| 対象 | トリガー | 内容 | 出典 |
|---|---|---|---|
| control 箱 border/box-shadow | hover / focus-within / status 変化 | `transition: border-color, box-shadow 125ms (--duration-fast) cubic-bezier(0.24, 1, 0.4, 1)`。reduced-motion で 0s | `inputStyles.stylex.ts:58-63` |
| ポップオーバー enter | open | `opacity 0→1` + `translateY(-8px) scale(0.95) → translateY(0) scale(1)`、165ms (`--duration-fast-max`)、`cubic-bezier(0.24, 1, 0.4, 1)`、fill backwards。exit アニメなし(即消去)。reduced-motion: fine pointer は無し、coarse は opacity のみ ≤200ms | `Layer/layerAnimations.stylex.ts:32-37, 94-97` |
| 日ボタン bg/color | 選択・端点変化 | `background-color, color 125ms cubic-bezier(0.24, 1, 0.4, 1)`、reduced-motion で none | `Calendar/styles.ts:214-219` |
| 日ボタン hover / press | pointer | 共通 press feedback(mouse: hover #0000000D/#ffffff0D、:active #0000001A/#ffffff1A 即時; touch: 150ms onset、slop 10px、quick-tap 100ms、release 200ms cubic-bezier(0, 0, 0.2, 1)) | `utils/pressFeedback.ts`、`interactionOverlay.stylex.ts` |
| 範囲帯 / プレビュー帯 | 状態変化 | **アニメなし**(即時差し替え) | `Calendar/styles.ts:141-193` |
| プリセット hover | hover | **トランジション指定なし**(即時) | `DateRangeInput.tsx:176-199` |
| Spinner | busy | Spinner section 参照 | — |
| disabledMessage tooltip | hover/focus | Tooltip 共通(show delay 200ms、hide bridge 100ms、Layer enter、placement above) | `useTooltip.tsx` |

Kivact: `Easing::CubicBezier(0.24, 1.0, 0.4, 1.0)` で `Style::transition(TransitionProp::Paint, Transition{ 125ms })`、ポップオーバーは dialog recipe の `Style::enter` に opacity+translate(-8)+scale(0.95)→1 を 165ms。`kivact::platform::reduced_motion()` が true なら transition 0、enter は(touch 時のみ)opacity 200ms 以下。

### 22.6 Behaviour & a11y

- **開閉**: トリガーのクリック / Enter / Space、またはアイコンボタンのクリックでトグル(`handleToggle :632-640`)。disabled / busy ではどちらも何もしない(呼び出し側 onClick も呼ばない)。開くと focus trap 内の最初のフォーカス可能要素へフォーカス: プリセットがあれば先頭プリセット、無ければ「Previous month」ボタン(テスト `moves focus into the popover…`)。トリガーの `aria-expanded` 同期、`aria-controls` は開いている間だけ。
- **閉じる**: Escape(カレンダーでアンカー選択中なら 1 回目の Escape は**選択取消のみ**で伝播停止、`Calendar.tsx:598-612`。2 回目で閉じる)、外側クリック(light dismiss)、隠し Close ボタン "Close calendar"(フォーカス時のみ可視)、範囲確定、プリセット確定。閉じたらフォーカスは開いた時にフォーカスされていた要素(通常トリガー)へ戻る。
- **範囲選択**(`Calendar.tsx:615-693`): 1 回目のクリック/Enter = アンカー(値は書かない)、ポライト通知 "Start date {Wednesday, January 5, 2026}. Select an end date."。2 回目 = 順序を正規化して `{start,end}` 確定 → 通知 "Selected range: {start} to {end}." → `onChange` → ポップオーバー閉。アンカーを再クリック: `minRangeSpan ≤ 1` なら 1 日範囲を確定、`> 1` なら取消して "Cleared start date {date}. Select a start date." を通知。
- **span 制約**: アンカーがある間だけ `|d-anchor| > max-1` と `0 < |d-anchor| < min-1` を無効化。既に cap より広い `value` を書き換えることはない。
- **プリセット**: Tab 移動の独立ボタン群(listbox ではない)。適用中は `aria-current="true"`。無効条件 = `isDisabled`、span 違反、いずれかの端点が min/max/constraints 違反(`:817-824`)。無効でもフォーカス可(aria-disabled)。押下で `getRange()` の結果を確定して閉じる。`isLoading` だけの busy 中はプリセットは有効表示のまま押すと**閉じるがコミットしない**(テスト `preserves loading-only selection dismissal`)。
- **disabled 中に開いたまま**: `isDisabled` に変わってもポップオーバーは閉じず、フォーカスも動かさず、日/プリセットのコミットだけをブロック。
- **クリア**: `onChange(null)`。キーボード起動(detail 0)ならトリガーへフォーカスを戻す、ポインタなら何もしない(`:659-672`)。クリアボタンはマウス押下でフォーカスを奪わない(pointerdown preventDefault)。
- **カレンダーのキー**(section 20 と同一): 矢印 1 日/1 週(無効日はスキップ、表示外の月へはビューを最小限シフト)、PageUp/PageDown 1 か月、roving tabindex。ヘッダー前後ボタンは 1 か月ずつ(2 か月表示でも 1 か月送り)。月が変わると月ラベルをポライト通知。
- **ロール/名前**: トリガー `role=button`、名前 `"{label}: {Jan 5 – Jan 9}"` または `"{label}: Select date range"`、`aria-haspopup=dialog`、`aria-invalid` は status=error のみ、`aria-busy`、`aria-describedby` = description, status message(tooltip 変種なら tooltip 内容), "Required", disabledMessage, caller の順。ポップオーバー `role=dialog` 名 "Choose date range"(modal)。グリッド `aria-multiselectable=true`、gridcell `aria-selected`(範囲内)、日ボタン名 "{date}, range start" / "range end" / "range start and range end" / "in range" / 単に日付、today は `aria-current=date`。
- **busy 通知**: busy 開始時に "Loading" をトリガー起点でアナウンス(`useBusyAnnouncement`)。
- **既定文言と i18n キー案**(Astryx キー → soft キー):
  - `@astryx.dateRangeInput.placeholder` "Select date range" → `soft.date_range_input.placeholder`
  - `@astryx.dateRangeInput.dialogLabel` "Choose date range" → `soft.date_range_input.dialog_label`
  - `@astryx.dateRangeInput.presetDateRanges` "Preset date ranges" → `soft.date_range_input.presets`
  - トリガー名テンプレ(Astryx はハードコード `${label}: ${…}`)→ `soft.date_range_input.trigger_name` "{label}: {value}"
  - 範囲区切り(ハードコード " – ")→ `soft.date_range_input.range` "{start} – {end}"
  - `@astryx.dateInput.openCalendar` "Open calendar" / `toggleCalendarClose` "Close calendar" / `closeCalendar` "Close calendar" / `clear` "Clear {label}" → `soft.date_input.open_calendar` / `.close_calendar` / `.clear`(section 21 と共有)
  - `@astryx.calendar.*`(rangeStartAnnounce, rangeCompleteAnnounce, rangeClearedAnnounce, dayRange*, previousMonth, nextMonth)→ `soft.calendar.*`(section 20 と共有)
  - `@astryx.field.required` "Required"、`@astryx.spinner.loading` "Loading"、`@astryx.inputClearButton.tooltip` "Clear" → `soft.field.required` / `soft.spinner.loading` / `soft.input_clear.tooltip`

### 22.7 Kivact 実装方針

- **戦略**: NEW kit component `soft::date_range_input`。外枠は soft の field-box(section 4 の共通 container をそのまま使う kit 内部 helper)、トリガーは `node().role(Role::Button)` + `kivact::interaction::use_button`(または basic と同じ `pressable` 相当)、ポップオーバーは `kivact::ui::dialog(label, open).kind(DialogKind::Popover).attach(control_box)`(soft の dialog recipe で Popover 面を描く)、中身は soft Calendar(section 20)の range 版。
- **最も近い basic**: `crates/kivact-ui-basic/src/date_picker.rs`(`DateRangePicker`: outline ボタン + Popover dialog + `calendar_range`、`on_commit` で閉じて `trigger_ref.restore_focus()`)と `crates/kivact-ui-basic/src/calendar/view.rs`。枠付き箱 + 先頭アイコン + 末尾クリアは `input_group.rs` / `recipes_field.rs` を参照。
- **interaction との差分(要注意)**: `kivact::interaction::use_calendar_grid` の `CalendarSelection::Range` は 1 回目の選択で **binding に 1 日範囲を書き込み**、Escape で元に戻す(`calendar.rs` モジュール doc)。Astryx は 1 回目で値を書かない。→ kit 内でローカル `signal(Option<DateRange>)` を grid に渡し、`CalendarPolicy::on_commit` が来た時だけ caller の binding に書く。ローカル値が caller と異なり on_commit 未着の間を「アンカー選択中」とし、`anchor = local.start()` を kit 側 signal で保持する(span 制約・プレビュー・通知に使う)。開くたびにローカルを caller 値へリセット。
  - `minRangeSpan > 1` でアンカー再クリック: interaction は 1 日範囲をコミットしてしまうので、on_commit で `start==end && min>1` を検出したらローカルを caller 値に戻し、取消通知を出し、ポップオーバーは閉じない。
  - span による無効化は `CalendarPolicy::disabled(move |d| base(d) || span_blocks(anchor.get(), d))`(anchor signal を読む)。disabled クロージャがリアクティブに再評価されない場合は gap として interaction 側に `CalendarPolicy::range_span(min, max)` または `CalendarGrid::pending_anchor()` の追加を提案。
  - ホバープレビュー: 各 `cell(..)` の `CalendarDayState.input.hovered` から hovered 日を kit signal に集約して帯を計算。
- 範囲帯 / キャップ: セル node の背面に `Style::paint` で帯(top/bottom 2px、端は 2px inset + 14px 半径)。隣接連続判定は `dayCellUtils.ts` の `computeDayNeighborContinuity` をそのまま移植。
- プリセット: 縦 `column` の kit ボタン(`use_button`)、`Role::Button`、`current` 状態は `aria-current` 相当の semantics が無ければ name に含めず state のみ(gap: kivact に `aria-current` があるか要確認。無ければ selected 状態で代替)。
- 通知: `Part::live_region(LiveRegion::Polite)` を持つ不可視 node に文言を書く。
- disabledMessage: `kivact::ui::tooltip(text).attach(control_box)`、disabled 時のみ有効。
- busy: `Action::pending_len()` と `.loading()` の OR。busy 中は表示値を新値(楽観)に保つ。
- 難所: (1) 2 か月並びの折り返し(488px 未満で縦積み)— kivact の flex wrap で再現。(2) 共通フォーカスリングの offset 描画。(3) range の "1 回目は値を書かない" 契約(上記)。(4) 月ラベルの standalone 大文字化は kivact i18n に無い → `soft.calendar.month_*` メッセージで代替。

### 22.8 受け入れ確認

- [ ] role=button 名 "Range: Select date range" のトリガー、h=32(md)/28/36、min-width 180、border #d4d4d4(dark #525252)、radius 10。
- [ ] value 設定時の文字列: 今年同士 "Mar 15 – Mar 22"、年違い "Dec 28, 2025 – Jan 3, 2026"。
- [ ] トリガー押下で role=dialog "Choose date range" が下・左揃えで 165ms フェード+8px スライドで出現。プリセットありなら先頭プリセット、なしなら "Previous month" にフォーカス。
- [ ] 既定 2 か月表示、ヘッダー "January 2026 – February 2026"、前後ボタンは 1 か月送り。幅 488px 未満で縦積み。
- [ ] 1 回目クリックで onChange が呼ばれず "Start date … Select an end date." が通知され、既存範囲の帯が消えアンカーのみ solid。ホバーでプレビュー帯(#0000000D)。
- [ ] 2 回目クリック(逆順でも)で start<=end に正規化して onChange、ポップオーバー閉、トリガーにフォーカス。
- [ ] 選択中 Escape は取消のみ(ダイアログ残存)、2 回目で閉じる。
- [ ] maxRangeSpan=7: アンカーから 7 日目以降が opacity 0.3 で押せない。minRangeSpan=2: アンカー再クリックで取消通知。
- [ ] プリセット: 適用中は aria-current + 背景 #f1f1f1(dark #262626)・文字 #1b1b1b(dark #f1f1f1)。min/max/span 違反は文字 #919191 で押しても無反応。
- [ ] クリア: キーボードで押すとトリガーにフォーカス、マウスではフォーカス移動なし。
- [ ] disabled: opacity 0.5、トリガーはフォーカス可で aria-disabled、disabledMessage がホバー/フォーカスで上に出る。
- [ ] error status: border #c0262c(dark #ffaea7)、hover で 2px 内側リング #de47454D、aria-invalid=true。
- [ ] docsite と light/dark で目視比較(範囲帯の丸キャップが日円と一致すること)。

---

## 23. DateTimeInput

ソース: `packages/core/src/DateTimeInput/DateTimeInput.tsx`(2101 行)、`NativeDateSegment.tsx`、`NativeTimeSegment.tsx`、`nativePickerSegmentStyles.ts`、`TouchDateTimeField.tsx`(1513 行)、`DateTimeInput.doc.mjs`、tests(`DateTimeInput.test.tsx`, `DateTimeInputTouch.test.tsx`, `NativePickerSegments.test.tsx`, `Presentation.test.tsx`)、stories `DateTimeInput.stories.tsx`。共有部品: Calendar(section 20)、DateInput の日付パース・MonthScroller・MonthYearWheels・Wheel・`nativeDateSegments.ts`・touch geometry(section 21)、TimeInput の時刻整形/パース(section 24)。Docs: https://astryx.atmeta.com/components/DateTimeInput

### 23.1 概要

- 日付と時刻を 1 ラベルの下で選ぶ複合フィールド。2 つの枠付き「セグメント」(日付・時刻)を横に並べ、コンテナ幅 400px 未満で各々全幅の 2 行に折り返す(ビューポートではなくコンテナ幅基準)。値は `"YYYY-MM-DDTHH:MM[:SS]"`。
- 表示面(`presentation`): 既定 `adaptive-native` = fine pointer は Astryx の typed field + カレンダーポップオーバー、coarse pointer は OS の date/time コントロール。`native` = 常に OS、`adaptive-bottom-sheet` = fine は typed、coarse は Date/Time 統合ボトムシート、`popover` / `bottom-sheet` は常にその面。
- **使う**: スケジューリング、イベント作成、締切。min/max で範囲を絞る。任意項目なら `hasClear`。`hourFormat` は通常未指定にしてロケールの時間制に従わせる。
- **使わない**: 日付だけ(DateInput)、時刻だけ(TimeInput)、文脈なしのラベル非表示、Tooltip で無効理由を包むこと。
- 重要な契約: **日付が無い限り値は emit されない**。日付を先に選ぶと時刻は「既存の時刻 → ネイティブ下書き → 現在時刻(now)」で補完して即 emit。時刻だけ入力して日付が無い場合、pointer 面では blur で入力が捨てられる(native 面は下書きとして保持)。

### 23.2 Anatomy(pointer 面 = `PointerDateTimeField`)

```
Field (label / description / required / status message は常に "detached")   ← section 4
└─ div.row [themeProps 'date-time-input' size,status,disabled]  display:flex wrap gap 8px  (枠なし)
   ├─ div.dateSegment [themeProps 'date-time-input-date-segment']  ← inputWrapperStyles.base, flex 1 1 196px
   │   ├─ button "Open calendar"|"Close calendar"  (タブ停止あり、disabled 時 describedby=理由)
   │   │     └─ Icon calendar 16px secondary [themeProps 'date-time-input-toggle-icon' state]
   │   ├─ input#id type=text role=combobox  aria-haspopup=dialog aria-expanded aria-controls
   │   │     aria-autocomplete=none autocomplete=off aria-labelledby(caller時) aria-describedby
   │   │     aria-required aria-invalid aria-busy aria-disabled readonly(disabled時)
   │   ├─ VisuallyHidden role=alert aria-live=assertive  ("Invalid date" while typed text invalid)
   │   ├─ InputClearButton "Clear {label}" (hasClear && value!==undefined && !disabled/busy)
   │   └─ Spinner sm (busy)          ※ status icon は描かない (detached 固定)
   └─ div.timeSegment [themeProps 'date-time-input-time-segment']  ← inputWrapperStyles.base, flex 1 1 196px
       ├─ div.icon > Icon clock 16px secondary [themeProps 'date-time-input-clock-icon']  (装飾、ボタンではない)
       ├─ input type=text  aria-label="{label} time"
       │     (timeOptionInterval 指定時のみ) role=combobox aria-expanded aria-controls aria-autocomplete=list aria-activedescendant
       └─ VisuallyHidden role=alert ("Invalid time")
Popover (Calendar mode=single、placement below/start、dialog "Choose date and time")   ← 開いている間だけ Calendar をマウント
Popover (timeOptions, role=none) > div[role=listbox aria-label="{label} time options"] [themeProps 'date-time-input-time-listbox']
   └─ div[role=option aria-selected tabindex=-1] × N  [themeProps 'date-time-input-time-option']
Tooltip (disabledMessage)
```

```
┌──────────────────────────────┐  ┌──────────────────────────────┐
│ [cal] March 20, 2026     (x) │  │ (clock) 2:30 PM              │   各 h=32 (md), flex-basis 196, gap 8
└──────────────────────────────┘  └──────────────────────────────┘
          ↓ コンテナ < 400px
┌──────────────────────────────────────────────┐
│ [cal] March 20, 2026                     (x) │
└──────────────────────────────────────────────┘
┌──────────────────────────────────────────────┐   (gap 8)
│ (clock) 2:30 PM                              │
└──────────────────────────────────────────────┘
```

### 23.3 API

| Astryx prop | 型 / 既定 | 意味 | Kivact |
|---|---|---|---|
| `label` | string(必須) | フィールドラベル(日付 input の名前) | `date_time_input(label, value)` |
| `value` | `ISODateTimeString`(undefined=空) | 値 | `impl Into<Binding<Option<DateTime>>>`。kit 型 `soft::DateTime { date: CalendarDate, time: TimeOfDay }`(`TimeOfDay` は section 24 で定義、seconds 保持) |
| `onChange`(必須) / `changeAction` | — | 変更 / 非同期 | binding 書き込み + `.on_change(..)` / `.change_action(Action<T, Option<DateTime>>)` |
| `isLoading` | false | busy | `.loading(..)` |
| `isLabelHidden`, `description`, `isOptional`, `isRequired`, `labelTooltip`, `width` | — | Field 共通 | 22.3 と同名メソッド |
| `isDisabled` / `disabledMessage` | — | フォーカス可能 disabled + 理由 | `.disabled(..)` / `.disabled_message(..)` |
| `min` / `max` | ISODateTimeString | 日付部分 → カレンダー範囲、時刻部分 → **その境界日に限り**時刻範囲 | `.min(DateTime)` / `.max(DateTime)` |
| `dateConstraints` | fn[] | 無効日 | `.date_constraint(..)` |
| `hasSeconds` | false | 秒を含める | `.seconds(bool)` |
| `hourFormat` | '12h'\|'24h' / ロケール | 時間制 | `.hour_cycle(HourCycle::H12/H24)`(未指定 = ロケール解決、24.7 参照) |
| `timeIncrement` | 1\|5\|10\|15\|30 / 1 | ↑↓ ステップ(分)。≠1 だと native time を使わない | `.time_step(MinuteStep)`(enum `One, Five, Ten, Fifteen, Thirty`) |
| `timeOptionInterval` | 5\|10\|15\|30\|60 | 指定すると時刻欄が combobox + プリセット時刻リストになる | `.time_options(OptionInterval)`(enum `Five, Ten, Fifteen, Thirty, Sixty`) |
| `hasClear` | **false** | クリアボタン(日付セグメント内) | `.clearable(..)` |
| `placeholder` | "Select a date" | 日付 placeholder | `.placeholder(..)` |
| `timePlaceholder` | "Select a time" | 時刻 placeholder | `.time_placeholder(..)` |
| `timeLabel` | "{label} time" | 時刻欄の名前(リスト名にも波及) | `.time_label(..)` |
| `size` | md | 28/32/36 | `.size(ControlSize)` |
| `status` | — | 枠色 + **detached** メッセージ(statusVariant prop は無い) | `.status(..)` |
| `numberOfMonths` | **1** | ポップオーバーの月数(sheet では無視) | `.months(MonthCount)` |
| `weekStartsOn` | 0 | 週頭 | `.week_start(Weekday)` |
| `presentation` | `adaptive-native` | 面の選択 | `.presentation(DateTimePresentation)`。kivact に OS ピッカーは無いので enum は `Popover, BottomSheet, Adaptive`(既定 `Adaptive` = `platform::capabilities().touch` で sheet、他は popover)。Astryx の `native`/`adaptive-native` は Kivact では `Adaptive` に写像し、native セグメントの見た目は 23.x で記述する「静止表示テキスト」として sheet 面の閉じた欄に流用 |
| `nativePicker`(deprecated) | touch/always/never | presentation の旧名 | 提供しない(互換不要) |
| base props(ref, id, aria, data, DOM handlers) | — | **日付 input** に渡る | NodeBuilder 修飾子を日付 input に |
| `containerXstyle` / `controlXstyle` 系 | — | Field 箱 / **row**(枠なしの外側行)に適用 | `.field_style(..)` / `.style(..)`(row) |

### 23.4 Visual spec(pointer 面)

- **row**(`DateTimeInput.tsx:165-174`): `display:flex; flex-wrap:wrap; gap 8px`、枠・背景なし。native 面のみ `min-inline-size 180px`。
- **セグメント箱**(日付・時刻とも、`:1701-1720, 1842-1869`): Field 共通 container そのまま(22.4 の表と同値: border 1px #d4d4d4 / #525252、radius 10、背景 #ffffff / #262626、padding 4/8、gap 8、hover リング #d4d4d44D / #5252524D、focus-within accent border + accent-muted 内側リング、status 枠色、disabled opacity 0.5 + withinRing)。高さ sm 28 / md 32 / lg 36、**min-width なし**、`flex:1; flex-basis:196px; min-width:0`(`:230-239`)。各セグメントは独立に focus-within/hover を描く(片方だけ光る)。status はセグメント両方に同じ枠色。
- **カレンダートグル**(`:175-192`): 22.4 のアイコンボタンと同じ(透明、radius 10、calendar 16px #6a6a6a / #9e9e9e)。**ただし tabIndex 指定なし = タブ停止する**(DateRangeInput は -1)。
- **テキスト input**(`:199-229`): 枠/padding なし、flex 1、Figtree、14/20、text-primary、placeholder text-secondary、iOS のみ 16px 下限。typed 値が不正(パース不可、時刻は範囲外も)の間は文字色 `--color-text-secondary`(#474747 / #9e9e9e)。
- **日付表示書式**: `DATE_FORMAT_LONG` = `{year:'numeric', month:'long', day:'numeric'}`(en: "March 20, 2026")。**時刻表示書式**: 24.4 と同じ(en 12h "2:30 PM"、24h "14:30"、秒あり "2:30:05 PM")。
- **時計アイコン**(`:193-198`): `display:flex; align-items/justify center; flex-shrink 0` の div 内に clock 16px secondary。クリックすると wrapper のクリック委譲で時刻 input にフォーカス(timeOptions 時はリストを開く)。
- **clear / spinner** は日付セグメントの末尾。status icon なし。
- **カレンダーポップオーバー**: 22.4 のポップオーバー面と同じ(#ffffff / #1b1b1b、radius 12、shadow-low、below/start、4px clearance)。中身 Calendar single(section 20 のまま、`numberOfMonths` 既定 1)。アンカーは**日付セグメント箱**。
- **時刻オプション リスト**(`:242-291`):

| 項目 | 値 |
|---|---|
| ポップオーバー面 | 同上(背景 popover、radius 12、shadow-low)。アンカー = 時刻セグメント箱、below/start |
| listbox | `box-sizing border-box; max-height 300px; overflow-y auto; padding 4px; min-width = アンカー幅 (anchor-size(width))` |
| option rest | `display:flex; align-items:center; width:100%; padding-inline 8px; padding-block sm 4px / md 6px (--spacing-1-5) / lg 8px; radius 10px; body 14/20; text-primary; 背景 transparent; text-align start` |
| highlighted(キーボード or ホバー) | 背景 `--color-overlay-hover`(#0000000D / #ffffff0D) |
| selected(現在値と一致) | `font-weight 500`(チェックアイコンなし) |
| 項目生成 | 0:00 から interval ごと、境界日の min/max 外は除外。ラベルは時刻表示書式(例 15 分: "12:00 AM", "12:15 AM", …) |

- **RTL**: row とセグメント内の flex 順が鏡像。カレンダーの前後シェブロンのみ `rtlStyles.mirror` で反転(section 20)。

### 23.5 Motion

| 対象 | 内容 | 出典 |
|---|---|---|
| セグメント箱 border/shadow | 125ms `cubic-bezier(0.24, 1, 0.4, 1)`、reduced 0s | `inputStyles.stylex.ts:58-63` |
| カレンダー / 時刻リスト ポップオーバー enter | opacity 0→1、`translateY(-8px) scale(0.95)`→0/1、165ms `cubic-bezier(0.24, 1, 0.4, 1)`、exit なし、reduced(coarse)は opacity ≤200ms | `layerAnimations.stylex.ts:32-37, 94-97` |
| option highlight | トランジションなし(即時) | `DateTimeInput.tsx:268-273` |
| リストの自動スクロール | キーボードハイライトを nearest へスクロール(ホバーではスクロールしない)。スクロールは即時 | `useHighlightedOptionScroll` |
| 行の折り返し | アニメなし | — |
| touch 面 | 23.x TouchDateTimeField 参照 | — |

### 23.6 Behaviour & a11y(pointer 面)

**日付セグメント**
- input クリック → カレンダーを開く(フォーカスは input に残す `skipAutoFocus`)。ArrowDown / Alt+ArrowDown(閉時)→ 開く(フォーカス維持)。Escape(開時)→ 閉じる。Enter → typed 下書きを確定。トグルボタン → 開閉(開く時は Calendar 内の最初の要素 = Previous month にフォーカス)。
- 入力中、パースできて有効日で現在値と違えば**即 emit**し、`calendarRef.navigateTo` で表示月を追従(`:1058-1087`)。blur / Enter で下書き確定: 空 → 値クリア(`onChange(undefined)`)、不正 → 破棄して整形表示に戻る。パースは DateInput と同じ `parseDateInput(text, locale)`(section 21)。
- カレンダーで日を選ぶ → emit → 閉じる。閉じた時フォーカスが宙に浮いていれば(Escape / 非フォーカス領域クリック)日付 input に戻す。別コントロールをクリックして閉じた場合は奪わない。
- **時刻補完**(`handleDateChange :986-1021`): 時刻 = 現在値の時刻 ?? native 下書き ?? **現在時刻**(秒は hasSeconds 時のみ)。新日付が min の日付と同日で時刻 < min 時刻なら min 時刻に、max 日と同日で > max 時刻なら max 時刻にクランプ。
- 不正入力中は `role=alert` に "Invalid date"、`aria-invalid=true`。

**時刻セグメント(リストなし)**
- 入力中、`parseLocaleTime`(24.6)で読めて範囲内(境界日のみ min/max 時刻)なら、**日付がある場合だけ** 即 emit。blur: 空 → 何もしない(部分 datetime は出さない、表示は元に戻る)、有効 → emit(日付ありのみ)、不正 → 破棄。
- ↑/↓: 現在の時刻(無ければ now)に ±timeIncrement 分(深夜でラップ、秒は保持、スナップしない)。範囲内かつ日付ありで emit し、新時刻をポライト通知(`:1581-1609`)。IME 変換中は無視。
- focus 中かつ空なら placeholder を形式ヒント "e.g., 2:30 PM" / "e.g., 14:30" に差し替え(disabled 時は差し替えない)。

**時刻セグメント(timeOptionInterval 指定時)**(`:1186-1630`)
- 時刻 input が `role=combobox`(aria-autocomplete=list)。クリック / Alt+ArrowDown で開く。開く時のハイライト = 現在値「以下で最大」の option(値なし/全 option より前なら先頭)。
- 開いている間: ↑↓ でハイライト移動(端で止まる、ラップなし)、Home/End、Enter = 「ハイライトがタイピング追従中なら typed 値、そうでなければハイライト option」を確定、Escape で閉じる、Tab で閉じてフォーカスは移動(blur で typed 確定)。閉じている間の ↑↓ は従来どおりステップ。
- タイピング中もリストは絞り込まず、パースできた時刻に近い option へハイライトを追従。
- option は pointerdown で preventDefault(input のフォーカスを保つ)→ click で確定 → 閉じて input にフォーカス。同じ値なら emit しない。ホバーでハイライト。
- 外側押下(時刻セグメントとリスト以外)で閉じる(light dismiss ではなく手動 outside-press)。disabled/busy になった、または option が 0 件になったら自動で閉じる。フォーカスがセグメント/リスト外へ移ったら閉じる。
- `aria-activedescendant` = ハイライト option id、option `aria-selected` = 現在値一致。

**共通**
- disabled: 両 input フォーカス可(aria-disabled + readonly)、入力/ステップ/ポップオーバーをブロック、disabledMessage ツールチップを row に。busy = loading または changeAction 保留中(pointer 面では保留中も操作不可)。
- クリア: 値と native 下書きを消し、日付 input にフォーカス(キーボード即時、ポインタは次フレームで preventScroll)。
- 既定文言と i18n キー案: `@astryx.dateTimeInput.placeholder` "Select a date" → `soft.date_time_input.placeholder`; `.timePlaceholder` "Select a time" → `.time_placeholder`; `.dialogLabel` "Choose date and time" → `.dialog_label`; `.timeSuffix` "{label} time" → `.time_suffix`; `.timeOptionsLabel` "{label} options" → `.time_options`; `.timeHint12h` "e.g., 2:30 PM" / `.timeHint24h` "e.g., 14:30" → `soft.time.hint_12h` / `soft.time.hint_24h`(section 24 と共有); `@astryx.dateInput.invalidDate` "Invalid date" → `soft.date_input.invalid`; `@astryx.timeInput.invalidTime` "Invalid time" → `soft.time_input.invalid`; open/close calendar / clear は 22.6 と同じ。sheet 専用は 23.x 参照。

### 23.7 Kivact 実装方針

- **戦略**: NEW kit component `soft::date_time_input`。presentation で 2 つの内部 view(pointer / sheet)を切替: `if presentation.resolves_to_sheet() { touch_field(..) } else { pointer_field(..) }`(`kivact::platform::capabilities().touch` で Adaptive を解決、ライフ中の変化は追わない)。
- pointer 面: 各セグメント = soft field-box + flush variant の `kivact::ui::text_input`(`.forward_keys(vec![Key::ArrowUp, Key::ArrowDown, Key::Enter, Key::Escape, Key::Home, Key::End])` でキーを kit ハンドラへ)。下書き = text_input の Binding<String>、確定は kit 側の blur/Enter ハンドラ(`.draft(on_commit)` は Enter/Escape 所有になるため、時刻リスト時の Enter 優先順位を保つには `forward_keys` + 自前 blur 処理の方が安全)。
- カレンダー: `kivact::ui::dialog(..).kind(DialogKind::Popover).attach(date_segment)` + soft Calendar single(section 20)。`autofocus` は「トグルボタンから開いた時のみ」。input クリック/ArrowDown で開く時はフォーカスを input に残す(dialog の initial focus を抑止できない場合は gap: `Dialog::autofocus(false)` 相当が必要)。
- 時刻リスト: `kivact::interaction::use_combobox` または `use_list` + `use_popup`(input がフォーカスを保持する activedescendant 型)。最も近い basic: `crates/kivact-ui-basic/src/combobox.rs` と `list_popup.rs`(ただし basic の option はチェック付き・行パディングが異なるので、23.4 の表の値で描く)。
- 日付パース/整形は section 21 の kit helper、時刻は section 24 の `TimeOfDay` helper(`parse_time`, `format_time`, `adjust_time`, `clamp_time`, `in_range`)を共有。
- 最も近い basic ファイル: `crates/kivact-ui-basic/src/date_picker.rs`(ポップオーバー + calendar の配線)、`input_group.rs`(枠内 icon + input)、`combobox.rs`(時刻リスト)、`drawer.rs`(sheet)。
- 難所: (1) `anchor-size(width)` → リストの min-width をアンカー node の実測幅に(kivact の Placement/overlay がアンカー幅を渡せるか確認、無ければ `node_ref` で計測)。(2) フォーカスが宙に浮いた時だけ戻す挙動 → `PopupCloseReason`(focus_return.rs)で Escape / outside-empty の時だけ restore。(3) 「日付なしで時刻を捨てる」挙動は Astryx の仕様なので再現する(勝手に下書き保持しない)。

### 23.8 受け入れ確認(pointer 面)

- [ ] ラベル "Meeting time" の combobox(日付)と textbox "Meeting time time" が 8px 間隔で横並び、各 h=32、コンテナ 399px で 2 行に折返し。
- [ ] 日付 input クリックでカレンダーが下に開き、フォーカスは input のまま。トグルボタンで開くと "Previous month" にフォーカス。
- [ ] 日付選択で値が `YYYY-MM-DDT<現在時刻>` になり、閉じる。時刻 "2:30 PM" 表示(en)。
- [ ] min="2026-03-20T09:00" で 3/20 を選ぶと 09:00 未満の時刻が 9:00 AM にクランプ。
- [ ] 時刻欄で ↑ が timeIncrement 分進み、新時刻が通知される。日付が無いと ↑ は無反応。
- [ ] timeOptionInterval=15: Alt+↓ で listbox(max-h 300、padding 4)が時刻欄幅以上で開き、↓/Enter で確定、選択中は太字 500、ハイライト #0000000D。
- [ ] 不正入力で文字色 #474747・aria-invalid・"Invalid time" alert、blur で元表示へ。
- [ ] status=error で両セグメント枠 #c0262c、メッセージは detached、セグメント内に status icon なし。
- [ ] hasClear: 日付セグメント末尾に × 、押すと値 undefined、日付 input にフォーカス。
- [ ] light/dark で docsite と目視比較(Default, WithTimeOptions, NarrowContainer, SizeVariants)。

### 23.9 サブパート

#### 23.9.1 nativePickerSegmentStyles.ts(native セグメント共有スタイル)

1. 概要: NativeDateSegment / NativeTimeSegment が共有する StyleX(`nativePickerSegmentStyles.ts:18-136`)。OS の `<input type=date|time>` を Astryx の枠内に収め、OS の内部表示を透明にしてロケール整形済みテキストを上に重ねる仕組み。
2. Anatomy: `iconButton`(先頭ボタン)/ `slot`(relative flex 1 の span)/ `input`(OS コントロール)/ `overlay`(absolute、aria-hidden の表示テキスト)。
3. API: 内部のみ。
4. Visual: iconButton = 22.4 と同値(透明、radius 10)。slot `position:relative; display:flex; align-items:center; flex:1; min-inline-size:0`。input = 23.4 のテキスト input と同じ文字設定 + `height: 1lh`(= 20px、iOS は 16px×1.4286)、`appearance:none`、`::-webkit-calendar-picker-indicator{display:none}`、`::-webkit-date-and-time-value` / `::-webkit-datetime-edit` の余白 0・text-align start。`inputTextHidden` = 文字色 transparent(overlay 表示中)。overlay `position:absolute; inset-inline 0; inset-block 0; pointer-events:none; nowrap; ellipsis`、値あり text-primary(#000000 / #ffffff)、placeholder text-secondary(#474747 / #9e9e9e)、不正時 text-secondary。
5. Motion: なし。
6. Behaviour: overlay は「フォーカス中かつエンジンが編集可能セグメントを出す(`hasEditableDateSegments`、section 21)」時だけ外れ、OS の編集セグメントを見せる。
7. Kivact: OS ピッカーが無いので再実装しない。**overlay テキストの見た目**(整形済み値 / placeholder、ellipsis)だけを sheet 面の閉じた読み取り専用欄(TouchDateTimeField / TouchTimeField)に流用する。
8. 受け入れ: sheet 面の閉じた欄で値が 1 行 ellipsis、placeholder が #474747。

#### 23.9.2 NativeDateSegment

1. 概要: native 面の日付半分。`<input type="date">` を非制御で持ち、ネイティブ `input`/`change` リスナで値を拾う(iOS で React の change が来ないため)(`NativeDateSegment.tsx:88-337`)。
2. Anatomy: `button[aria-label="Open calendar"][tabindex=-1]`(calendar アイコン、`date-time-input-toggle-icon` state collapsed 固定)+ `span.slot > input#id type=date + span.overlay(aria-hidden)` + `VisuallyHidden role=alert`("Invalid date")。
3. API(内部): `value, onChange, placeholder, min, max, isDateDisabled, isEffectivelyDisabled, isFocusableDisabled, isEffectivelyRequired, isBusy, statusType, ariaDescribedBy, controlProps`。
4. Visual: 23.9.1。overlay の値書式 = `formatSharedDate(.., 'date_long')`(en "March 20, 2026"、section 21 と同じ)。
5. Motion: なし。
6. Behaviour: 外部値は**非フォーカス時のみ** DOM に反映。コミット: 空 → `onChange(undefined)`、パース後 `isDateDisabled` なら拒否(`rejectedValue`、aria-invalid、alert)、それ以外は ISO を emit。blur 時、フォーカス中に値が変わっていなければ DOM 値を最終コミット。ボタン → `focus()` + `showPicker()`。disabled はフォーカス可(aria-disabled + readonly)。
7. Kivact: **非対応(OS 日付ピッカーなし)**。`Native`/`AdaptiveNative` は Kivact では採らず sheet/popover に写像(23.3)。拒否時の挙動(invalid 表示 + alert)は sheet の日付選択では起きない(無効日は選べない)。
8. 受け入れ: Kivact では N/A(presentation 写像のテストのみ: touch 環境で sheet、非 touch で popover)。

#### 23.9.3 NativeTimeSegment(TimeInput と共有)

1. 概要: `<input type="time" step=60>` を Astryx 枠内に置く共有部品(`NativeTimeSegment.tsx:107-364`)。DateTimeInput の時刻半分と TimeInput の native 面が使う。
2. Anatomy: `button[aria-label=openPickerLabel][tabindex=-1]`(clock 16px secondary)+ `span.slot > input type=time + overlay` + alert "Invalid time"。名前は `inputLabel`("{label} time")を labelledby が無い時だけ aria-label に。
3. API(内部): `value, onChange, placeholder, inputLabel, openPickerLabel, ariaLabelledBy, iconThemeProps, hasAutoFocus, min, max, hourFormat, isEffectivelyDisabled, isFocusableDisabled, isEffectivelyRequired, isBusy, statusType, ariaDescribedBy, controlProps`。
4. Visual: 23.9.1。overlay = `formatLocaleTime(value, locale, hourFormat, false)`(**秒は常に表示しない**、native は分精度)。
5. Motion: なし。
6. Behaviour: 小数秒を落とし `HH:MM` に正規化。範囲外は拒否表示。DateTimeInput では日付未選択の時刻を `nativeTimeDraft` に保持し、日付確定時に結合。クリア時は OS ピッカー再オープンを避けるためフォーカスを戻さない。changeAction 保留中も native は操作可能(iOS ホイールが外れるため)。
7. Kivact: 非対応(OS 時刻ピッカーなし)。TimeInput の `Native` 系は TouchTimeField(24.9)に写像。
8. 受け入れ: N/A(写像テストのみ)。

#### 23.9.4 TouchDateTimeField(Date/Time 統合ボトムシート)

**1. 概要**: coarse pointer(または `bottom-sheet`)の面。2 つの読み取り専用セグメントを押すと 1 枚の BottomSheet が開き、SegmentedControl で Date / Time パネルを切替える。日付は MonthScroller(縦スワイプのページング月グリッド、section 21)、時刻は Wheel 列(section 21 の Wheel)。値は**即時コミット**(Save はパネル移動/閉じるだけ)。`timeIncrement` / `timeOptionInterval` / `numberOfMonths` は無視。

**2. Anatomy**(`TouchDateTimeField.tsx:1037-1511`)

```
Field (status detached)
└─ div.touchRow (flex wrap, gap 8, inline-size 100%)
   ├─ div.dateSegment (field box, flex 1 1 196) — 余白クリックで Date パネルを開く
   │   ├─ button[tabindex=-1] "Open calendar"  (calendar icon, toggle-icon state = sheet open?expanded:collapsed)
   │   ├─ input type=text role=combobox readonly inputmode=none  aria-haspopup=dialog aria-expanded=(open && panel=date)
   │   ├─ InputClearButton / Spinner
   └─ div.timeSegment (field box) — 余白クリックで Time パネル
       ├─ button[tabindex=-1] "Open {label} time" (clock icon)
       └─ input role=combobox readonly aria-label="{label} time" aria-expanded=(open && panel=time)
BottomSheet (label "Choose date and time", height hug, padding 0)
└─ div.touchSheetBody  padding-inline 16 / top 24 / bottom 16
   └─ div.touchSurface  column gap 12
      ├─ SegmentedControl label="Date/time section" layout=fill: [Date][Time]
      └─ div.touchPanelStack (grid, 2 パネルを同一セルに重ねる)
         ├─ panel[data-panel=date]  (column gap 8)  非アクティブ時 aria-hidden + inert + visibility hidden
         │   ├─ header (h 36, gap 4)
         │   │   ├─ button.title aria-expanded aria-label="{March 2026}, Choose month and year"  "March 2026 ⌄"
         │   │   └─ span.actions (margin-inline-start auto, gap 2)  ※wheels 表示中は hidden + inert
         │   │        [‹ IconButton ghost sm "Previous month"] [› "Next month"] [Button ghost sm "Reset"]
         │   └─ dateSurfaceStack (grid 重ね)
         │       ├─ surface[calendar]: weekdays row (h 28) + MonthScroller + footer [Save date (primary md, 100%)]
         │       └─ surface[wheels]:  spacer (h 28) + MonthYearWheels + footer [Done (secondary md, 100%)]
         └─ panel[data-panel=time] role=group aria-label="{label} time"
             ├─ wheels row (h 264, gap 8): Wheel Hour | Wheel Minute | (Wheel Second) | (Wheel AM/PM)
             └─ footer [Save (primary md, 100%)]
```

**3. API**: DateTimeInput と同一(23.3)。sheet 固有の追加 API なし。

**4. Visual spec**

| 部位 | 値(出典 `TouchDateTimeField.tsx:155-383`) |
|---|---|
| 閉じたセグメント | pointer 面と同じ field box、高さ 28/32/36、flex-basis 196。input に `caret-color:transparent; cursor:pointer; user-select:none` |
| 日付表示 | `DATE_FORMAT_LONG`("March 20, 2026")、時刻表示 = 24.4 書式(秒は hasSeconds 時)。時刻は値が無くても下書きがあれば表示 |
| sheet 面 | BottomSheet section 参照: 背景(BottomSheet 既定)、上辺/左右 1px `--color-border`(#00000014 / #ffffff1A)、上角 radius 12、`--shadow-high`(light `0 4px 6px #0000001A, 0 12px 24px #00000026` / dark `0 4px 6px #00000080, 0 12px 24px #000000B3, inset 0 0 0 1px #ffffff26`)、grab handle は本体上の 24px 帯に浮く |
| body | padding-inline 16px、top 24px、bottom 16px、`touchSurface` gap 12px |
| SegmentedControl | layout fill(2 等分)、neutral 上書き: track padding 4px、item 高さ size-8px(md 24px)、選択 item の影なし(SegmentedControl section 参照) |
| panel | column gap 8px |
| header | 高さ 36px(`--size-element-lg`)、gap 4px |
| title ボタン | padding-inline 8px、radius 10、枠/背景なし、text-primary、Figtree、**17px (`--text-large-size`) / weight 600**、内部 gap 4px、テキスト nowrap ellipsis、chevronDown 16px secondary |
| 前後 IconButton | ghost sm(28px)、coarse pointer で min 44×44(`dateInputTouchSizes.daySize`)、移動不可の側は `visibility:hidden`(レイアウトは保持)、シェブロンは RTL 反転 |
| Reset | ghost sm Button、coarse で min-height 44 |
| actions | gap 2px(`--spacing-0-5`)、wheels 表示中は visibility hidden + opacity 0 |
| weekdays | `grid repeat(7,1fr)`、高さ 28px、12px(`--text-supporting-size`)weight 400、text-secondary、中央。表記は `{weekday:'short'}`("Sun")— Calendar の 2 文字("Su")とは異なる |
| MonthScroller | 高さ 264px(6 週 × 44px)、section 21 |
| wheels spacer | 28px |
| time wheels 行 | `display:flex; inline-size 100%; block-size 264px; gap 8px`、各 Wheel は等分(Wheel の行高 28px、帯、端 padding (264-28)/2 = 118px は section 21) |
| Hour wheel | 12h: "1"…"12"(先頭ゼロなし)、24h: "00"…"23" |
| Minute / Second | "00"…"59" |
| AM/PM | "AM" / "PM"(Astryx はここだけロケール語ではなく i18n キー固定語) |
| 無効行 | min/max(その日付が境界日の場合のみ)に重ならない時/分/秒/AM-PM 行は Wheel の disabled 表示(section 21) |
| footer | `display:flex; margin-block-start:auto; padding-block-start 4px`。Save date / Save = primary md 全幅、Done = secondary md 全幅 |
| 非アクティブ panel/surface | `visibility:hidden; opacity:0; pointer-events:none`(グリッド重ねで高さは最大パネルに揃う) |

**5. Motion**

| 対象 | 内容 | 出典 |
|---|---|---|
| sheet enter | `transform translateY(100%)→0`(@starting-style)300ms (`--duration-medium`) `cubic-bezier(0.24, 1, 0.4, 1)`。reduced-motion: 移動なし、coarse では opacity 0→1 200ms、fine は即時(0.01s) | `BottomSheet/BottomSheetPanel.tsx:164-189` |
| sheet exit | `translateY(0→100%)` 300ms、fine pointer `--ease-exit` cubic-bezier(0.3, 0, 0.6, 0.6)、coarse も同値 cubic-bezier(0.3, 0, 0.6, 0.6)。reduced: coarse は opacity→0 200ms、fine は即時 | `BottomSheetPanel.tsx:206-221` |
| scrim | opacity 300ms `--ease-standard`(BottomSheet section 参照) | `BottomSheet.tsx:180-207` |
| Date/Time パネル切替、calendar↔wheels 切替 | **トランジションなし**(visibility/opacity 即時) | `TouchDateTimeField.tsx:269-273, 351-355` |
| 前後ボタンでの月移動 | MonthScroller の smooth scroll(section 21 の物理) | `:772-786` |
| Wheel スナップ / 慣性 | section 21 | — |
| SegmentedControl 選択インジケータ | SegmentedControl section 参照 | — |

**6. Behaviour & a11y**

- セグメントの input クリック / ↓ / Enter / Space → そのパネルで sheet を開く(`handleInputKeyDown :981-997`)。セグメント余白のクリックも同様。disabled/busy(loading のみ。changeAction 保留中は開ける)では開かない。disabled/loading になったら開いている sheet を閉じる。
- Date パネル: 日タップで**即 emit**(時刻 = 値の時刻 ?? 下書き ?? now、境界日ならクランプ)。"Save date" は日付があるまで disabled、押すと Time パネルへ移り Time タブにフォーカス(次タスク)。タイトルで month/year wheels をトグル(`aria-expanded`)、Done で戻る。Reset = 値クリア + 下書きクリア + 今月へ smooth スクロール(今月が範囲外なら移動なし)。前後ボタンは min/max 月で止まる(既定可動範囲は DateInput の `DEFAULT_MONTH_REACH`、section 21)。
- Time パネル: 各 Wheel の変更で時刻を合成し境界日の min/max にクランプして下書きへ。日付があれば即 emit。12h の hour 行の無効判定は現在の AM/PM 半日内での到達性。"Save" で閉じる。
- 閉じた時に日付が未選択なら時刻下書きを破棄。
- クリア(閉じたセグメントの ×): 値・下書きクリア、sheet を閉じ、次タスクで日付 input にフォーカス(preventScroll)。
- ARIA: 閉じた input は `role=combobox` + `aria-haspopup=dialog` + パネル別 `aria-expanded`、読み取り専用。sheet は modal dialog "Choose date and time"。非アクティブパネルは `aria-hidden` + `inert`。busy 開始は sheet 内からアナウンス。
- 文言キー案: `@astryx.dateTimeInput.pickerMode` "Date/time section" → `soft.date_time_input.picker_mode`; `.dateTab` "Date" / `.timeTab` "Time" → `.date_tab` / `.time_tab`; `.openTimePicker` "Open {label}" → `.open_time_picker`; `.saveDatePicking` "Save date" → `.save_date`; `.hourWheel` "Hour" / `.minuteWheel` "Minute" / `.secondWheel` "Second" / `.meridiemWheel` "AM/PM" / `.meridiemAM` "AM" / `.meridiemPM` "PM" → `soft.time.hour_wheel` / `.minute_wheel` / `.second_wheel` / `.meridiem_wheel` / `.am` / `.pm`; `@astryx.dateInput.savePicking` "Save" / `.resetPicking` "Reset" / `.doneChoosingMonth` "Done" / `.chooseMonthYear` "{monthYear}, Choose month and year" / `.monthWheel` "Month" / `.yearWheel` "Year" → `soft.date_input.save` / `.reset` / `.done` / `.choose_month_year` / `.month_wheel` / `.year_wheel`(section 21 と共有)。

**7. Kivact 実装方針**

- sheet: soft の BottomSheet(別 section)を使う。無ければ `kivact::ui::dialog(..).kind(DialogKind::Drawer(Side::Bottom))` + soft dialog recipe、スワイプ閉じは `crates/kivact-ui-basic/src/drawer.rs`(swipe dismiss)を参照。
- SegmentedControl: soft の SegmentedControl(別 section)。パネルは `grid` 重ね + 非アクティブを `hidden` 状態(semantics から除外 = inert 相当)にし、レイアウトは保持(visibility hidden 相当: kivact で「描かないが場所を取る」スタイルが無ければ opacity 0 + `.disabled(true)` + semantics hidden で代替)。
- MonthScroller / MonthYearWheels / Wheel は section 21 の kit 部品をそのまま使う(DateTimeInput 固有の差分なし。Wheel の `isActive` はアクティブパネル時のみ true)。
- 閉じた欄は `kivact::ui::text_input(..).read_only(true)` に caret 非表示の recipe、または `node().role(Role::ComboBox)` + text(編集不要なのでこちらが簡単)。
- 最も近い basic: `drawer.rs`、`date_picker.rs`、`calendar/view.rs`。

**8. 受け入れ確認**

- [ ] touch 環境(または `presentation(BottomSheet)`)で 2 つの読み取り専用 combobox。日付欄タップで sheet が下から 300ms でスライドし Date タブ選択。
- [ ] 日タップで即時に値が変わる(閉じた欄に反映)。"Save date" で Time パネルへ、Time タブにフォーカス。
- [ ] Hour/Minute/AM-PM の 3 ホイール(hasSeconds で 4、24h で AM/PM なし、時は "00".."23")。
- [ ] min が当日 09:30 のとき、AM の 12 と 1–8 時が disabled、9 時選択中は 00–29 分が disabled。
- [ ] Reset で値が消え今月へスクロール。タイトルで月/年ホイールに切替、ヘッダー右のボタン群が消える。
- [ ] Save で sheet が閉じる(exit 300ms、加速カーブ)。
- [ ] light/dark で docsite(NativePickerModes / bottom sheet 設定)と目視比較。

---

## 24. TimeInput

ソース: `packages/core/src/TimeInput/TimeInput.tsx`(1001 行)、`TouchTimeField.tsx`(635 行)、`TimeInput.doc.mjs`、tests(`TimeInput.test.tsx`, `NativeTimeInput.test.tsx`, `Presentation.test.tsx`, `TimeInput.fr10.test.tsx`)、`utils/timeFormat.ts`、`utils/timeParser.ts`、`utils/inputPresentation.ts`、stories `TimeInput.stories.tsx`。native 面は `DateTimeInput/NativeTimeSegment.tsx`(23.9.3)。Docs: https://astryx.atmeta.com/components/TimeInput

### 24.1 概要

- 時刻を 1 つ選ぶフィールド。fine pointer では**自由入力のテキストフィールド**(セグメント分割ではない。"2:30 PM", "14:30", "1430", "2pm" などを解釈)で、↑↓ による分ステップを持つ。coarse pointer の既定は OS の time ピッカー(Kivact では非対応 → sheet に写像)、`bottom-sheet` 系では時/分/(秒)/(AM-PM) ホイールのボトムシート(TouchTimeField)。
- 値は ISO `"HH:MM"` / `"HH:MM:SS"`。表示はロケールの時間制(12h/24h)と日中区分語で整形。
- **使う**: `hourFormat` は未指定にしてロケールに従わせる(ユーザー嗜好はロケールタグ `en-US-u-hc-h23` で)。営業時間などは `min`/`max`。形式や目的のヒントを description / placeholder に。エラーは `status` で。任意項目は `hasClear`。
- **使わない**: 日付+時刻の複合(DateTimeInput か DateInput と併用)、ラベル非表示、Tooltip で無効理由を包むこと。

### 24.2 Anatomy(typed 面 = `TimeField`)

```
Field (status は statusVariant に従う)                         ← InputGroup 内では Field を描かず VisuallyHidden で名前/説明/状態
└─ div.control [themeProps 'time-input' size,status,disabled]  ← inputWrapperStyles.base + sizeStyles (+ groupStyles.inGroup)
   ├─ div.icon > Icon clock 16px secondary        (装飾。クリックは wrapper 委譲で input にフォーカス)
   ├─ input#id type=text  placeholder  aria-labelledby aria-describedby aria-required aria-invalid aria-busy aria-disabled readonly(disabled)
   ├─ VisuallyHidden role=alert aria-live=assertive ("Invalid time" while invalid)
   ├─ Spinner sm (busy)
   ├─ InputClearButton "Clear {label}" (hasClear && value && !disabled)
   └─ status icon (attached/tooltip、group 外)
Tooltip (disabledMessage)
```

```
┌──────────────────────┐
│ (clock) 2:30 PM  (x) │  h=32 (md), min-w=120
└──────────────────────┘
 focus + 空:  (clock) e.g., 2:30 PM   ← placeholder がヒントに変わる
```

### 24.3 API

| Astryx prop | 型 / 既定 | 意味 | Kivact |
|---|---|---|---|
| `label` | string(必須) | 名前 | `time_input(label, value)` |
| `value` / `onChange` | `ISOTimeString \| undefined` | 値 / 変更(クリアで undefined) | `impl Into<Binding<Option<TimeOfDay>>>`(kit 型 `soft::TimeOfDay { hour: u8, minute: u8, second: u8 }`、`to_iso(with_seconds)` / `from_iso`) |
| `changeAction` / `isLoading` | — | 非同期 / busy | `.change_action(Action<T, Option<TimeOfDay>>)` / `.loading(..)` |
| `isLabelHidden`, `description`, `isOptional`, `isRequired`(aria-required を input に), `labelTooltip`, `width` | — | Field 共通 | 22.3 と同名 |
| `isDisabled` / `disabledMessage` | — | フォーカス可能 disabled(readonly + aria-disabled)/ 理由 | `.disabled(..)` / `.disabled_message(..)` |
| `min` / `max` | ISOTimeString | 範囲(両端含む)。範囲外の確定は拒否 | `.min(TimeOfDay)` / `.max(TimeOfDay)` |
| `hasSeconds` | false | 秒の表示・解析 | `.seconds(bool)` |
| `hasClear` | **false** | クリア | `.clearable(..)` |
| `hasAutoFocus` | false | マウント時フォーカス | `.autofocus(bool)` |
| `hourFormat` | '12h'\|'24h' / ロケールの時間制 | 表示の時間制 | `.hour_cycle(HourCycle)` |
| `increment` | 1 | ↑↓ の分数(整数のみ、小数なら不変) | `.step(u32)`(分) |
| `presentation` | `adaptive-native` | `text-input` / `popover`(= text-input) / `bottom-sheet` / `native` / `adaptive-bottom-sheet` / `adaptive-native` | `.presentation(TimePresentation)`: `TextInput`, `BottomSheet`, `Adaptive`(既定。touch なら sheet、他は typed)。`native` / `adaptive-native` は Kivact に OS ピッカーが無いため `Adaptive` に写像。Astryx の adaptive-native は `hasSeconds` か `increment≠1` なら coarse でも typed を使うが、Kivact の sheet は秒も扱えるので常に sheet で良い |
| `nativePicker`(deprecated) | — | 旧 API | 提供しない |
| `placeholder` | "Select a time" | 空表示(focus 中はヒントに置換) | `.placeholder(..)` |
| `size` | md | 28/32/36 | `.size(ControlSize)` |
| `status` / `statusVariant` | — / attached | 状態 | `.status(..)` / `.status_placement(..)` |
| base props | — | **input** に渡る。onFocus/onBlur/onKeyDown は内部処理の後 | NodeBuilder 修飾子を input に |
| `aria-labelledby` | — | Field ラベル(group 内は group+input ラベル)の後に連結 | `.labelled_by(..)` |
| `containerXstyle` / `controlXstyle` 系 | — | Field 箱 / control 箱 | `.field_style(..)` / `.style(..)` |

### 24.4 Visual spec

- **control 箱**: Field 共通 container(22.4 の表と同値)。高さ sm 28 / md 32 / lg 36、**min-width 120px**(`TimeInput.tsx:148-161`)。InputGroup 内は `groupStyles.inGroup`(InputGroup section 参照)。
- **時計アイコン**(`:109-114`): flex 中央、flex-shrink 0、clock 16px `--color-icon-secondary`(#6a6a6a / #9e9e9e)。
- **input**(`:115-145`): `display:block; flex:1; min-width:0`、枠/padding/背景なし、Figtree、body 14px / 20px、text-primary(#000000 / #ffffff)、placeholder text-secondary(#474747 / #9e9e9e)、outline none、iOS のみ 16px 下限。不正(パース不可または min/max 外)の間は文字色 text-secondary。disabled は cursor default のみ(箱が opacity 0.5)。
- **表示書式**(`utils/timeFormat.ts:224-244`): `Intl.DateTimeFormat(locale, {hour: 12h?'numeric':'2-digit', minute:'2-digit', second: hasSeconds?'2-digit':undefined, hourCycle: 12h?'h12':'h23', timeZone:'UTC'})`、狭い空白(U+202F 等)は通常スペースへ。en 12h "2:30 PM" / "12:05 AM"、24h "14:30" / "09:05"、秒 "2:30:05 PM"。ko-KR 12h "오후 2:30"、de-DE 24h "14:30"。12h 表記で 02:00 と 14:00 が区別できないロケール(fr-CM 等)は英語で整形。
- **ヒント placeholder**: focus 中かつ表示値が空 → 12h "e.g., 2:30 PM" / 24h "e.g., 14:30"。
- clear / spinner / status icon: 22.4 と同じ(status icon は md 20px、attached/tooltip のみ。detached と group 内は無し)。
- disabled / status / hover / focus-within: 22.4 と同じ(withinRing)。
- RTL: flex 鏡像(アイコンが右端)。

### 24.5 Motion

| 対象 | 内容 | 出典 |
|---|---|---|
| control 箱 border/box-shadow | 125ms `cubic-bezier(0.24, 1, 0.4, 1)`、reduced 0s | `inputStyles.stylex.ts:58-63` |
| placeholder ↔ ヒント差替え、不正色 | 即時(トランジションなし) | `TimeInput.tsx:593-600, 143-145` |
| disabledMessage tooltip | Tooltip 共通 | — |
| sheet | 24.9 参照 | — |

### 24.6 Behaviour & a11y(typed 面)

- **入力**(`:603-620`): 毎回下書きを保持。`parseLocaleTime(text, locale, hasSeconds)` で読めて、min/max 内で、現在値と異なれば**即 emit**(blur を待たない)。
- **解釈ルール**(`timeFormat.ts:251-306`, `timeParser.ts:155-279`): ロケールの数字・区切り(":"、fi-FI ".", fr-CA "h")・日中区分語(前後どちらでも)を剥がして英語寛容パーサへ。受理形: "2:30 PM", "2:30pm", "2:30 p.m.", "14:30", "1430", "143000"(秒)、"2pm", "2 PM", "14"(24h の時のみ)。meridiem 付きの時は 1–12 のみ。分・秒 0–59。hasSeconds=false の時は秒を捨てて "HH:MM"。
- **blur**(`:634-667`): 下書きが空 → 値があればクリア(`onChange(undefined)`)。有効 → 異なれば emit。不正 → 黙って整形表示に戻す(alert で "Invalid time" は入力中に既に出ている)。disabled 中は下書きを捨てるだけ。
- **↑/↓**(`:677-732`): preventDefault、現在値(無ければ**現在時刻**、秒込み)に ±increment 分。24h でラップ(23:59 + 1 → 00:00)、秒は保持、スナップなし(10:07 +15 → 10:22)。結果が min/max 外なら何もしない。emit 後に整形済み新時刻をポライト通知。IME 変換中・disabled は無視。
- **Enter**: disabled 時のみ preventDefault(フォーム送信防止)。それ以外は既定(送信)。
- **クリック委譲**: アイコン/余白クリックで input にフォーカス(`useInputContainer`)。
- **クリア**: `onChange(undefined)`、input にフォーカス(キーボード即時、ポインタは次フレーム preventScroll)。
- **ARIA**: input は textbox(combobox ではない)、`aria-invalid` = status error または不正入力、`aria-required`、`aria-busy`、`aria-describedby` = description, status message(または tooltip), disabledMessage, caller。不正中は `role=alert` assertive "Invalid time"。InputGroup 内では status メッセージを `useAnnounce` で(error は assertive、他は polite)通知。busy 開始で "Loading"。
- **時間制の解決**: `hourFormat ?? ロケールの hourCycle`(`resolveHourFormat`)。`-u-hc-h23` 等の拡張を尊重。未指定 provider のロケールは 'en' → 12h。
- 文言キー案: `@astryx.timeInput.placeholder` "Select a time" → `soft.time_input.placeholder`; `.invalidTime` "Invalid time" → `soft.time_input.invalid`; `.openPicker` "Open {label}" → `soft.time_input.open_picker`; `.clearLabel` "Clear {label}" → `soft.time_input.clear`; ヒントは `soft.time.hint_12h` / `soft.time.hint_24h`; ロケール日中区分語は Intl 由来 → `soft.time.am` "AM" / `soft.time.pm` "PM"(Kivact では message で置換); `soft.input_clear.tooltip`, `soft.spinner.loading`。

### 24.7 Kivact 実装方針

- **戦略**: RESTYLE `kivact::ui::text_input` + kit wrapper `soft::time_input`。wrapper が field box(section 4 共通)、先頭 clock アイコン、末尾 clear/spinner/status icon、時刻ロジックを持ち、内側は flush variant の `text_input(label, draft)` に `.forward_keys(vec![Key::ArrowUp, Key::ArrowDown])`(`text_input.rs:150-170` の規則)。下書き String binding の変化で即 emit、blur で確定(`TextInput` の focus 変化は NodeBuilder の on_focus/on_blur 相当で受ける。`draft()` は Enter/Escape を所有して既定送信を変えるので使わない)。
- 時刻ドメインは kit 内 `time` モジュール: `TimeOfDay`、`HourCycle`、`format_time(t, locale, cycle, seconds)`、`parse_time(text, locale, seconds)`(timeParser の英語寛容規則を完全移植 + 12h/24h のロケール区切り)、`adjust_minutes(t, delta)`(整数のみ、ラップ、秒保持)、`clamp_time`、`in_range`。DateTimeInput と sheet も共有。
- **gap(ロケール時間制)**: kivact の `kivact::i18n::Locale` は `tag()` / `language()` / `first_day_of_week()` のみで hour cycle も日中区分語も無い(`crates/kivact-i18n/src/locale.rs`)。対応: (1) タグの `-u-hc-h11|h12|h23|h24` を解析、(2) 無ければ 12h 地域の小表(US, CA, AU, NZ, IN, PH, PK, EG, SA, KR, TW 等。CLDR 完全一致は不可)、(3) 日中区分語は `soft.time.am`/`pm` メッセージ。英語以外の Intl 完全互換は UNRESOLVED(Intl/CLDR 非搭載)。
- アナウンス: 不可視 node に `Part::live_region(LiveRegion::Polite)`(ステップ)/ `Assertive`(Invalid time)。
- 最も近い basic: `crates/kivact-ui-basic/src/input_group.rs`(`input_group(input_group_input(..)).start(icon)`)と `field.rs` / `recipes_field.rs`。
- 難所: `min-width 120` と `flex:1` の input 内でのテキスト ellipsis なし(input は横スクロール)— text_input の既定に任せる。

### 24.8 受け入れ確認

- [ ] textbox "Start time"(combobox ではない)、h=32、min-w 120、clock アイコン 16px #6a6a6a(dark #9e9e9e)。
- [ ] en で value "14:30" → "2:30 PM"、hourFormat 24h → "14:30"、hasSeconds で "2:30:05 PM"。
- [ ] focus で空なら placeholder "e.g., 2:30 PM"、disabled では変わらない。
- [ ] "1430" と入力した瞬間に onChange("14:30")。"25:00" は文字色 #474747・aria-invalid・"Invalid time" alert、blur で元に戻り onChange 無し。
- [ ] ↑ で +increment(15 なら 10:07→10:22)、23:55 +15 → 00:10、max 超えなら変化なし。新時刻がポライト通知。
- [ ] 空にして blur で onChange(undefined)。hasClear の × でクリアし input にフォーカス。
- [ ] disabled: opacity 0.5、フォーカス可、入力/↑↓不可、Enter で送信しない、disabledMessage ツールチップ。
- [ ] status error/warning/success の枠色と hover リング(22.4 の値)。
- [ ] light/dark で docsite(Default, TwentyFourHourFormat, WithSeconds, WithIncrement, StatusVariantComparison)と目視比較。

### 24.9 サブパート: TouchTimeField(時刻ボトムシート)

**1. 概要**: `presentation` が sheet に解決した時の面(`TouchTimeField.tsx:188-633`)。閉じた欄は読み取り専用 combobox、押すと時/分/(秒)/(AM-PM) ホイールの BottomSheet。ホイール変更は**即コミット**(min/max にクランプ)、Save は閉じるだけ。`increment` と `hasAutoFocus` は無視。DateTimeInput の Time パネルと同じ挙動(DEC-3)。

**2. Anatomy**

```
Field (statusVariant に従う) / InputGroup 内は wrapper のみ
└─ div.control [themeProps 'time-input'] (field box, sizeStyles min-w 120)
   ├─ button[tabindex=-1] aria-label="Open {label}"  > Icon clock 16px secondary
   ├─ input type=text role=combobox readonly inputmode=none aria-haspopup=dialog aria-expanded aria-autocomplete=none
   ├─ Spinner / InputClearButton / status icon
   ├─ BottomSheet (label = field label, height hug, padding 0)
   │   └─ div.sheetBody (column gap 8, padding-inline 16 / top 24 / bottom 16)
   │       ├─ div.wheels (flex, h 264, gap 8): Wheel "Hour" | "Minute" | ("Second") | ("AM/PM")
   │       └─ div.footer > Button primary md 100% "Save"
   └─ Tooltip (disabledMessage)
```

**3. API**: TimeInput と同一(24.3)。

**4. Visual spec**

| 部位 | 値(`TouchTimeField.tsx:79-142`) |
|---|---|
| 閉じた箱 | field box、高さ 28/32/36、min-width 120 |
| clock ボタン | `display:flex; 中央; padding 0; 枠/背景なし; flex-shrink 0`、**radius 指定なし**(他のアイコンボタンは 10px)、cursor pointer |
| input | `flex:1; min-width:0; 枠/padding/背景なし; outline none; cursor pointer`。**フォント/色の指定なし**(UA 既定に落ちる)→ UNRESOLVED: Astryx の取りこぼしと判断。Kivact では typed 面と同じ body 14/20・text-primary・placeholder text-secondary で描くことを推奨 |
| 表示 | `formatLocaleTime(value, locale, hourFormat, hasSeconds)` |
| sheet body | `display:flex; column; gap 8px; padding-inline 16px; padding-block-start 24px; padding-block-end 16px` |
| wheels | `display:flex; inline-size 100%; block-size 264px (dateInputTouchGeometry.paneBlockSize = 6×44); gap 8px` |
| hour 表記 | 12h "1"…"12"、24h "00"…"23"。minute/second "00"…"59"。AM/PM は `soft.time.am`/`pm` |
| footer | `display:flex; margin-block-start:auto; padding-block-start 4px`、Save primary md 全幅 |
| sheet 面 / Wheel 表示 | 23.9.4 / section 21 と同じ |

**5. Motion**: sheet enter/exit とスクリム = 23.9.4 の表と同値(300ms、enter `cubic-bezier(0.24, 1, 0.4, 1)`、exit `cubic-bezier(0.3, 0, 0.6, 0.6)`、reduced は coarse で 200ms フェード)。Wheel の慣性・スナップは section 21。閉じた箱 = 125ms border/shadow。

**6. Behaviour & a11y**

- input / clock ボタンのクリックで開く(disabled・loading では開かない。changeAction 保留中は開ける)。キーボードで開くキー割当は**無い**(Enter は disabled 時に送信防止のみ)— Kivact では Astryx の DateTimeInput sheet と揃えて ↓ / Enter / Space で開くことを推奨(差分として明記)。
- ホイール初期値: 値 ?? マウント時点の現在時刻。合成規則(`:327-354`): 12h hour 変更は現在の AM/PM 内、AM/PM 変更は 12h の時を保ったまま半日移動、秒は hasSeconds 時のみ。`clampTime(raw, min, max)` 後に値と異なり範囲内なら emit。
- 無効行: hour は「その時間帯 [h:00, h:59:59] が min/max と重ならない」、minute は「その分(秒ありなら 59 秒幅)が重ならない」、second は範囲外、AM/PM は半日が重ならない。
- クリア: `onChange(undefined)` のみ(フォーカス移動なし)。
- ARIA: input `role=combobox` + `aria-haspopup=dialog` + `aria-expanded`、sheet は modal dialog(名 = field label)。各 Wheel は listbox(名 "Hour" 等、section 21)。busy は sheet 内からアナウンス。
- 文言: `@astryx.dateTimeInput.hourWheel` 等と `@astryx.dateInput.savePicking` "Save"(23.9.4 のキー案を共有)、`@astryx.timeInput.openPicker` "Open {label}"。

**7. Kivact 実装方針**: soft BottomSheet(なければ `kivact::ui::dialog(..).kind(DialogKind::Drawer(Side::Bottom))`)+ section 21 の Wheel を 2–4 本。`time` モジュール(24.7)の合成/クランプを使う。閉じた欄は `node().role(Role::ComboBox)` + text + `.expanded(open)`。最も近い basic: `crates/kivact-ui-basic/src/drawer.rs`、`input_group.rs`。

**8. 受け入れ確認**

- [ ] `presentation(BottomSheet)` で読み取り専用 combobox。押すと "Start time" sheet、Hour/Minute/AM-PM の 3 ホイール(24h なら AM/PM なし、hasSeconds で Second 追加)。
- [ ] ホイールを回すたびに閉じた欄の値が即変化、max 超過はクランプ。
- [ ] min "09:00" / max "17:00": AM の 1–8 と 12 が disabled、PM の 6–11 が disabled。
- [ ] Save で閉じる(300ms 加速 exit)。× でクリア。
- [ ] light/dark で docsite(BottomSheetWithMinMax)と目視比較。


---

## 25. RichTextEditor

Source: `D:\w\meta\astryx\packages\richtext` (package `@astryxdesign/richtext`)。主要ファイル:
`src/RichTextEditor.tsx`, `src/RichTextEditorToolbar.tsx`, `src/RichTextView.tsx`, `src/editorTheme.ts` (全ブロックの見た目), `src/editorNodes.ts`, `src/CodeBlockHeaderPlugin.tsx`, `src/CodeSyntaxPlugin.tsx`, `src/codeLineTokens.ts`, `src/codeLanguages.ts`, `src/TaskCheckboxPlugin.tsx`, `src/TableColumnFloorPlugin.tsx`, `src/TableScrollRegionPlugin.tsx`, `src/RichTextEditorAutoLinkPlugin.tsx`, `src/linkUtils.ts`, `src/MarkdownExtensionsPlugin.tsx`, `src/markdownExtensions.ts`, `src/markdownExtensionNode.ts`, `src/textSemantics.ts`, `src/markdown*.ts` (import/export 規則), `src/RichTextEditor.doc.mjs`, `README.md`, `blocks/RichTextEditorShowcase.tsx`。
Stories: `apps/storybook/stories/RichTextEditor.stories.tsx` (Default / WithToolbar / ResponsiveToolbar / ResponsiveToolbarStressTest / WithLinks / WithAutoLink / WithDescription / Required / WithCharacterLimit / CustomTransformers / ErrorStatus / DetachedStatus / TooltipStatus / ReadOnly / WithInitialValue / ControlledPersistence / ImperativeRef / MarkdownPlugins / MarkdownPluginsImportExport / MarkdownSerializers / MarkdownParity* / CodeLanguageModules)、`RichTextEditor.playground.tsx` (全機能を1つの editor に載せた sandbox)、`RichTextEditor.imagePlugin.tsx` (storybook 専用の image node。package の機能ではない)。
a11y spec: `src/__tests__/*.a11y.chromium.spec.ts` (Toolbar / RichTextView / TableCellSelection / RichTextCodeLanguages / Playground / MarkdownPlugins / MarkdownPluginsImportExport / MarkdownParitySandbox)。
Docs: https://astryx.atmeta.com/components/RichTextEditor

> 前提: Astryx 版は Lexical (contenteditable の DOM editor) の上に作られた WYSIWYG editor。Kivact のネイティブ editor は「1つの plain string・1つの text style・折り返しなし」なので、**同じものをそのまま作ることはできない**。25.1〜25.6 は Astryx の仕様を正確に書いた「目標の見た目と振る舞い」。25.7 で、いまの Kivact で作れる範囲 (MVP) と、engine の拡張が必要な後の段階を分けて書く。25.8 の受け入れ確認も段階ごとに分ける。

### 25.1 概要

- 目的: label 付きの form field の中に置く rich text (WYSIWYG) editor。書式は見出し、引用、箇条書き / 番号付きリスト / task list、太字・斜体・下線・取り消し線・inline code、link、fenced code block (言語ラベル + copy + syntax color)、GFM table、水平線 (thematic break)。Markdown と相互変換でき、**何も編集していない block は書いたとおりの byte で戻る** (round-trip)。
- 組み合わせ部品:
  - `RichTextEditor`: field 本体 (label / description / status / counter / placeholder を持つ)。
  - `RichTextEditorToolbar`: `toolbar` slot に入れる書式 toolbar (undo/redo、block 形式 Selector、inline 書式 toggle、link dialog)。
  - `RichTextView`: 保存済みの内容を **読み取り専用の文書として** 描く (form field ではない)。editor と同じ theme なので見た目は同じ。
  - `RichTextEditorAutoLinkPlugin`: 入力 / 貼り付けされた URL と email を自動で link にする。
  - `markdownToEditorStateJSON` / `editorStateJSONToMarkdown`: headless 変換 (server でも動く)。
  - `createRichTextExtension`: core Markdown の syntax plugin (例: mention `@{ada}`、callout `:::note … :::`) を editor / view で 1 つの塊 (atomic node) として描く。
- いつ使うか (doc.mjs bestPractices の要約):
  - 書式付きの長文 (メモ、説明文、コメント) を書かせるとき。内容は onChange で保存し、`defaultValue` / `RichTextView value` で戻す。
  - toolbar は `toolbar={<RichTextEditorToolbar/>}` で入れる。field の上端に端から端まで置かれ、4px の gutter を持つ。幅が足りないと action 行が横 scroll する (More menu は作らない)。
  - link は toolbar の Link ボタンか Mod+K で dialog を出す。URL は http/https/mailto/tel だけ許す。既定で新しいタブで開く (`target=_blank rel="noopener noreferrer"` を node のデータに書く)。
  - GFM table と thematic break は import / 編集 / export できる。
  - custom node を使うなら editor と view の両方に同じものを登録する。
- 使わないとき: 1 行入力や plain text → TextInput / TextArea を使う (doc.mjs:277-280)。

### 25.2 Anatomy

#### 25.2.1 RichTextEditor (`RichTextEditor.tsx:629-786`)

```
Field (label, description, Optional/Required, labelTooltip, status message。4. Field 節)
└─ div.astryx-rich-text-editor  [data-size, data-status]   ← 共通 input container (4.4.4) を column に組み替えた枠
   ├─ (toolbar slot)  RichTextEditorToolbar               ← 枠の上端に端から端まで
   ├─ div.editorBody  (padding、toolbar があれば上に 1px の区切り線)
   │  ├─ div.editorRoot (position relative。body 文字の基準)
   │  │  ├─ ContentEditable  role=textbox aria-multiline=true   ← 文書本体 (各 block は 25.4.5)
   │  │  ├─ placeholder div (aria-hidden、absolute 0,0)       ← 空のときだけ
   │  │  ├─ code block header × N (absolute。文書の外の兄弟)   ← CodeBlockHeaderPlugin
   │  │  └─ task checkbox × N (absolute。文書の外の兄弟)       ← TaskCheckboxPlugin
   │  └─ status icon (absolute、top 8 / inline-end 8)          ← status があるとき
   └─ VisuallyHidden #tabEscapeHint ("Press Escape then Tab …")
└─ counter div "n/max" (+ VisuallyHidden aria-live=polite)    ← maxLength があるとき。枠の外の下
```

- 文書 (ContentEditable の子) の block: paragraph `<p>`、heading `<h1>`〜`<h6>`、quote `<blockquote>`、list `<ul>/<ol>` + `<li>`、code `<code>` (block。行は text + `<br>`)、table `div.tableScrollableWrapper > table > tr > th|td`、horizontal rule `<hr>`、Markdown plugin node (decorator)。inline: text `<span>`、太字 `<strong>`、斜体 `<em>`、inline code `<code>`、link `<a>`、取り消し線は `<del>` で包む (textSemantics.ts)、太字斜体は `<em><strong>` (textSemantics.ts)。
- code block header と task checkbox は **文書の中ではなく兄弟** として、計測した座標に absolute で重ねる (編集できない、Markdown に出ない)。CodeBlockHeaderPlugin.tsx:1-17、TaskCheckboxPlugin.tsx:5-24。

#### 25.2.2 RichTextEditorToolbar (`RichTextEditorToolbar.tsx:911-1071`)

```
Toolbar (role=toolbar, aria-label "Text formatting", size sm, roving tabindex)
└─ startContent: HStack gap 4px role=group aria-label "Formatting actions" (横 scroll する行)
   ├─ IconButton Undo (ghost, tooltip "Undo", glyph は RTL で左右反転)
   ├─ IconButton Redo (ghost, tooltip "Redo", RTL 反転)
   ├─ Divider vertical aria-label "History and block formats"
   ├─ Selector "Block format" (label 非表示, ghost, size sm, startIcon = 今の block の glyph)
   │     options: Paragraph(¶) / Heading 1(H1) / Heading 2(H2) / Heading 3(H3) / Bulleted list / Numbered list / Block quote
   ├─ Divider vertical aria-label "Block and inline formats"
   ├─ ToggleButton Bold / Italic / Underline / Strikethrough / Inline code  (icon only, aria-pressed)
   ├─ ToggleButton Link (aria-haspopup=dialog, aria-expanded)               ← hasLink のとき
   └─ [Divider vertical + endContent]                                       ← endContent があるとき
Dialog (purpose=form, width 400)                                              ← link 用。toolbar の兄弟
└─ form > Layout
   ├─ DialogHeader title "Insert link" | "Edit link" (+ close)
   ├─ LayoutContent: TextInput "URL" (autofocus, status error detached)
   └─ LayoutFooter: HStack gap 8 end: [Remove link (destructive, 編集時のみ)] [Cancel (secondary)] [Add link | Update link (primary, submit)]
```

#### 25.2.3 RichTextView (`RichTextView.tsx:210-349`)

```
div (width 100%, position relative, body 文字)
└─ ContentEditable (editable=false、role なし、aria-readonly / aria-autocomplete を消す)
   └─ 同じ block 群 (25.4.5)。table の wrapper は role=group aria-label "Table"、はみ出すときだけ tabindex 0
├─ code block header × N (copy button は文書の後ろの tab 順)
└─ task checkbox × N (read-only)
(value が壊れていれば errorFallback だけを描く)
```

### 25.3 API

#### 25.3.1 RichTextEditor props (`RichTextEditor.tsx:307-483`) → Kivact

| Astryx prop | 型 / default | 意味 | Kivact (`soft::rich_text_editor`) |
|---|---|---|---|
| `label` | string (必須) | 常に描く label | `rich_text_editor(label, doc)` の第1引数 |
| `isLabelHidden` | bool / false | label を視覚的に隠す (aria-label にする) | `.label_hidden(bool)` |
| `description` | string | label と editor の間 | `.description(text)` |
| `isOptional` / `isRequired` | bool / false | "Optional"/"Required" 表示。required は aria-required | `.optional(bool)` / `.required(bool)` |
| `labelTooltip` | string | label 末尾の info icon tooltip | `.label_tooltip(text)` |
| `defaultValue` | string (Lexical JSON) / – | 初期 state。mount 時に 1 回だけ読む (uncontrolled) | **置き換え**: caller-owned `Binding<RichDoc>` (第2引数)。Lexical JSON 互換は捨てる。初期値は `RichDoc::from_markdown(..)` で作る |
| `onChange` | (EditorState, LexicalEditor) | 内容変更 (selection だけの変化と history merge は無視) | `.on_change(Fn(&RichDoc))`。binding の更新で足りるなら不要 |
| `placeholder` | string | 空のとき | `.placeholder(text)` |
| `isReadOnly` | bool / false | 編集不可。全不透明、tab 順に残る、aria-readonly | `.read_only(impl IntoValue<bool>)` |
| `isDisabled` | bool / false | 編集不可、0.5 で暗く、tab 順から外れる、aria-disabled。readOnly より優先 | `.disabled(impl IntoValue<bool>)` |
| `status` | `{type: warning\|error\|success, message?}` | 枠の色 + status icon + message。error は aria-invalid | `.status(impl IntoValue<Option<FieldStatus>>)` (4./5. 節と同じ型) |
| `statusVariant` | attached\|detached\|tooltip / attached | message の置き方 (5. FieldStatus 節) | `.status_variant(StatusVariant)` |
| `width` | number\|string | field の幅 | `.width(Length)` |
| `minHeight` | SizeValue / `'4.5rem'` (72px) | 編集面 (ContentEditable) の最小高さ。内容に合わせて伸びる | `.min_height(Length)`、既定 72px |
| `size` | sm\|md\|lg / md (SizeContext から) | 編集面の padding だけ変わる (lg で上下 8px) | `.size(RichTextEditorSize)` |
| `nodes` | Lexical node class 配列 | custom node の登録 | **drop**。代わりに `.extensions(Vec<MarkdownExtension>)` (25.3.5) |
| `toolbar` | ReactNode | 上端の toolbar slot | `.toolbar(|| soft::rich_text_toolbar())` (slot closure) |
| `plugins` | ReactNode | Lexical plugin | **drop**。必要な拡張点は個別 method (`.auto_link(..)`, `.extensions(..)`) |
| `hasMarkdownShortcuts` | bool / true | `# ` などの入力 shortcut | `.markdown_shortcuts(bool)` |
| `transformers` | Transformer[] / DEFAULT_TRANSFORMERS | Markdown 規則の唯一の出所 | **drop** (固定の規則セット)。必要なら `MarkdownOptions` の feature flag (tables/rules/tasks) |
| `markdownExtensions` | RichTextMarkdownExtension[] | plugin node を描く | `.extensions(..)` |
| `codeLanguages` | CodeLanguage[] | fence 名 → 言語 (色とラベル) | `.code_languages(Vec<CodeLanguage>)` |
| `hasAutoFocus` | bool / false | mount で focus | `.auto_focus(bool)` |
| `tabEscapeHint` | string / 翻訳済み既定文 | SR 用の "Escape then Tab" ヒント。`""` で省略 | `.tab_escape_hint(Option<text>)` 既定 `soft.rich_text_editor.tab_escape_hint` |
| `maxLength` | number | counter を出す (強制はしない) | `.max_length(usize)` |
| `namespace` | string / 'astryx-editor' | Lexical の id | **drop** |
| `xstyle` / `className` / `style` | – | – | **drop** (theme recipe で調整) |
| `ref` (RichTextEditorRef) | focus / clear / getEditorState / getMarkdown / getHTML / getEditor | 命令的 handle | `.handle(RichTextEditorHandle)` (caller が `RichTextEditorHandle::new()` で持つ)。`focus()`, `clear()` (editable のときだけ。空 paragraph 1 つに戻す), `doc()`, `markdown() -> String` (keep-source export), `plain_text()`。`getHTML` は後段 (25.7)、`getEditor` は drop |

theming target: `astryx-rich-text-editor`、visual props `size`, `status` (doc.mjs:200-211)。Kivact では recipe key `soft.rich_text_editor` の state に `size` / `status` / `focused` / `hovered` / `disabled` / `read_only` を渡す。

#### 25.3.2 RichTextEditorToolbar props (`RichTextEditorToolbar.tsx:458-517`)

| Astryx prop | 型 / default | 意味 | Kivact (`soft::rich_text_toolbar()`) |
|---|---|---|---|
| `label` | string / "Text formatting" | toolbar の aria-label | `.label(text)` 既定 `soft.rich_text_editor.toolbar.label` |
| `headingLevels` | ('h1'\|'h2'\|'h3')[] / [h1,h2,h3] | Selector に出す見出し | `.heading_levels(&[1,2,3])` |
| `size` | sm\|md\|lg / sm | toolbar と control の size | `.size(ControlSize)` 既定 Sm |
| `hasLink` | bool / true | Link ボタン + Mod+K | `.link(bool)` |
| `promptForUrl` | (selectedText) => string\|null | 同期の URL 取得 (dialog の代わり) | `.prompt_for_url(Fn(&str) -> Option<String>)` (ほぼ使わない。残してよい) |
| `hasNewTabLinks` | bool / true | 作る link に target=_blank | `.new_tab_links(bool)`: ネイティブでは link node の `open_in_new_window` フラグとして保存するだけ (activation は app の `on_link` が決める) |
| `endContent` | ReactNode | divider の後ろに追加 control | `.end_content(|ctx: RichTextCommands| ..)` slot。`RichTextCommands` (editor へ command を送る handle) を渡す |
| `ref`/`className`/`style`/`xstyle`/DOM handler | – | Toolbar へ素通し | drop |

`RICHTEXT_ICON_KEYS` (`richtext:bold` など 15 個。:199-215) → kit の icon registry key `soft.rich_text.icon.<name>` で差し替え可能にする。

#### 25.3.3 RichTextView props (`RichTextView.tsx:103-153`)

| Astryx | 意味 | Kivact (`soft::rich_text_view(doc)`) |
|---|---|---|
| `value` (Lexical JSON, 必須) | 描く内容。変わると描き直す (SyncValuePlugin) | `rich_text_view(impl IntoValue<RichDoc>)` |
| `nodes` | custom node | drop (extensions) |
| `markdownExtensions` | plugin node の描画 | `.extensions(..)` |
| `codeLanguages` | 言語 | `.code_languages(..)` |
| `plugins` | read-only plugin | drop |
| `namespace` | – | drop |
| `onParseError` / `errorFallback` (既定 null) | JSON が壊れていたとき | `RichDoc` は型付きなので parse 失敗はない。Markdown 文字列から作る API (`rich_text_view_markdown(&str)`) は失敗しないので drop。`.error_fallback` は不要 |
| (なし) | link を押したとき | **追加**: `.on_link(Fn(&LinkTarget))` (ネイティブでは browser が開かないので必須の拡張点) |

#### 25.3.4 その他の公開物

- `RichTextEditorAutoLinkPlugin({matchers?, onChange?})` (`RichTextEditorAutoLinkPlugin.tsx:107-112`) → `.auto_link(AutoLink::default())`。`AutoLink::matchers(Vec<LinkMatcher>)`、`on_change(Fn(Option<&str>, Option<&str>))`。既定 matcher: URL (`URL_MATCHER`, linkUtils.ts:41-42。scheme なしは `https://` を付け `sanitizeUrl` を通す) と email (`EMAIL_MATCHER` :45-46 → `mailto:`)。どちらも新しいタブ属性付き。
- `sanitizeUrl(url) -> string` / `validateUrl(url) -> bool` (linkUtils.ts:59-92): trim、空なら `about:blank`/false。scheme がなければ `https://` を前に付けて URL として parse。protocol が `http: https: mailto: tel:` 以外、または parse 失敗なら `about:blank`/false。成功なら正規化した href。→ `soft::rich_text::sanitize_url(&str) -> Option<String>` (None = 拒否)。
- `markdownToEditorStateJSON(md, {transformers?, nodes?, extensions?})` / `editorStateJSONToMarkdown(json, opts)` (markdownSerializers.ts:79-108) → `RichDoc::from_markdown(&str, &MarkdownOptions) -> RichDoc` / `RichDoc::to_markdown(&self) -> String`。
- `createRichTextExtension(plugin)` / `RichTextExtensionError {plugin, capability, claimant}` (markdownExtensions.ts:64-106) → `MarkdownExtension` trait (25.3.5)。

#### 25.3.5 Kivact の提案 API (まとめ)

```rust
// crates/kivact-ui-soft/src/rich_text/mod.rs
pub struct RichDoc { /* blocks + 各 top-level block の authored source 記録 (25.6.8) */ }
pub enum Block {
    Paragraph(Inline),
    Heading { level: u8 /*1..=6*/, text: Inline },
    Quote(Vec<Inline>),                       // Astryx の quote は入れ子にならない (25.6.8)
    List(List),
    Code { language: Option<String>, info: String, code: String },
    Table { align: Vec<CellAlign>, header: Vec<Inline>, rows: Vec<Vec<Inline>> },
    Rule,
    Extension(ExtensionNode),                 // block plugin node
}
pub struct List { pub kind: ListKind /*Bullet{marker}|Ordered{start}*/, pub items: Vec<ListItem> }
pub struct ListItem { pub task: Option<bool>, pub content: Inline, pub children: Vec<List> }
pub struct Inline(pub Vec<InlineRun>);
pub struct InlineRun { pub text: String, pub marks: Marks /*BOLD|ITALIC|UNDERLINE|STRIKE|CODE*/,
                       pub link: Option<LinkTarget>, pub extension: Option<ExtensionNode>, pub hard_break_after: bool }
pub struct LinkTarget { pub url: String, pub title: Option<String>, pub new_window: bool, pub auto: bool }

pub fn rich_text_editor(label: impl IntoText, doc: impl Into<Binding<RichDoc>>) -> RichTextEditor;
pub fn rich_text_toolbar() -> RichTextToolbar;
pub fn rich_text_view(doc: impl IntoValue<RichDoc>) -> RichTextView;
pub struct RichTextEditorHandle; // new(), focus(), clear(), doc(), markdown(), plain_text()
pub trait MarkdownExtension { fn name(&self) -> &str; fn parse_inline(..); fn parse_block(..);
                              fn render(&self, node: &ExtensionNode) -> View; }
pub enum RichTextEditorSize { Sm, Md, Lg }
```

### 25.4 Visual spec

#### 25.4.0 この component で使う token (neutral, light / dark)

| token | light | dark | 用途 |
|---|---|---|---|
| `--color-border-emphasized` | #d4d4d4 | #525252 | 枠 (rest)、quote の左線 |
| `--color-accent` | #1b1b1b | #f1f1f1 | focus-within の枠、focus ring、table cell 選択 ring |
| `--color-accent-muted` | #f1f1f1 | #262626 | focus-within の inset 2px ring (dark では surface と同色で見えない。Astryx の実装どおり) |
| `--color-background-surface` | #ffffff | #262626 | 枠の背景 |
| `--color-border` | #00000014 | #ffffff1A | toolbar 下の区切り、table の罫線、hr、code block の枠 |
| `--color-text-primary` | #000000 | #ffffff | 本文 |
| `--color-text-secondary` | #474747 | #9e9e9e | placeholder、quote の文字、table header の文字、counter |
| `--color-text-accent` | #1b1b1b | #f1f1f1 | link の文字 (neutral では青ではない) |
| `--color-text-error` | #c0262c | #ffaea7 | counter 超過、error 枠 (`--color-border-error` = text-error) |
| `--color-text-warning` / `--color-border-warning` | #836700 | #eec448 | warning 枠 |
| `--color-text-success` / `--color-border-success` | #2f7d33 | #90ca90 | success 枠 |
| `--shadow-inset-error` / `-warning` / `-success` | inset 0 0 0 2px #de47454D / #f8d36a4D / #2f7d334D | 同じ | status 付き hover ring |
| `--color-background-muted` | #f1f1f1 | #1b1b1b | inline code の背景 |
| `--color-overlay-hover` / `-pressed` | #0000000D / #0000001A | #ffffff0D / #ffffff1A | toolbar control の hover / 押下・pressed |
| `--focus-outline-*` | 2px solid accent, offset 3px | 同 | hr / plugin node 選択 ring、toolbar control の focus |
| `--color-syntax-background` | #ffffff | #111111 | code block の背景 |
| `--color-syntax-comment` (= operator, punctuation) | #6a6a6a | #9e9e9e | code header の言語ラベルと copy icon、comment |
| `--color-syntax-keyword` (= type) | #6b187c | #efa8ff | |
| `--color-syntax-string` | #0b5615 | #a4d6a3 | |
| `--color-syntax-number` (= constant) | #733100 | #ffb37e | |
| `--color-syntax-function` | #00458c | #a1caff | |
| `--color-syntax-variable` | #111111 | #e2e2e2 | |
| `--color-syntax-tag` | #8a0011 | #ffaea7 | |
| `--color-syntax-attribute` | #584400 | #eec448 | |
| `--color-syntax-property` | #005348 | #90d7c8 | |
| `--color-overlay` | #00000080 | #000000CC | link dialog の backdrop |

syntax の値は `neutralTheme.ts:15-32` (defineSyntaxTheme) を palette ref で解いたもの。

Typography (neutral。fine pointer / coarse pointer かつ < lg の adaptation): family body/heading = `Figtree` (fallback `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif`)、code = `ui-monospace, "SF Mono", Monaco, Consolas, "Liberation Mono", "Courier New", monospace`。

| type token | fine (base 14 / 1.2) | coarse (base 16 / 1.1736) | weight |
|---|---|---|---|
| `--text-body-*` | 14 / 20 | 16 / 24 | 400 |
| `--text-heading-1-*` | 24 / 32 | 26 / 36 | 600 (semibold) |
| `--text-heading-2-*` | 20 / 28 | 22 / 32 | 600 |
| `--text-heading-3-*` | 17 / 24 | 19 / 28 | **700** (neutral: heading weight 3 = bold) |
| `--text-heading-4-*` | 14 / 20 | 16 / 24 | **700** (neutral: 4 = bold) |
| `--text-heading-5-*` | 12 / 20 | 14 / 20 | 600 |
| `--text-heading-6-*` | 10 / 16 | 12 / 20 | 600 |
| `--text-code-*` | 14 / 20 | 16 / 24 | 400 |
| `--text-supporting-*` | 12 / 20 | 14 / 20 | 400 |

(expandTypeScale.ts:225-330 の式: size = round(base·ratio^step)、line height = 4px 格子に丸め、<20px は ×1.5、20〜31px は ×1.4、最小 size+4。)
`--font-weight-medium` 500、`--font-weight-semibold` 600、`--font-weight-bold` 700。

#### 25.4.1 Field の枠 (`RichTextEditor.tsx:141-155, 652-675` + `Field/inputStyles.stylex.ts:30-77`)

共通 input container (4.4.4) を使い、`styles.wrapper` で次のように変える:

- 並び: `flex-direction: column`, `align-items: stretch`, gap 0, padding 0 (block / inline とも)、`min-height: auto`。高さは中身で決まる。
- `--container-padding-*` を 0px にする (外側の Section の padding が toolbar に漏れないように)。
- 枠: 1px solid `--color-border-emphasized` (#d4d4d4 / #525252)。radius `--radius-element` 10px。背景 `--color-background-surface` (#ffffff / #262626)。
- 状態:
  - hover (pointer が hover でき、focus-within でも disabled でもない): `box-shadow: inset 0 0 0 2px` color-mix(border-emphasized 30%, transparent) = #d4d4d44D / #5252524D。
  - focus-within (editor 本体、toolbar、checkbox、code header のどれかに focus): 枠 `--color-accent` (#1b1b1b / #f1f1f1) + `inset 0 0 0 2px --color-accent-muted` (#f1f1f1 / #262626)。forced-colors では枠 `Highlight`。**toolbar の button に focus があっても枠は focus 色になる** (`:focus-within`)。
  - status (warning/error/success): 枠をその status 色に (rest も focus-within も同じ。25.4.0)。hover の ring は `--shadow-inset-<status>`。ただし disabled と read-only では status の hover ring を付けない (:665-669)。
  - disabled: 全体 (toolbar も含む) `opacity: 0.5`、枠は border-emphasized、shadow なし、cursor default。
  - read-only: 暗くしない。hover ring と focus-within の見た目は通常どおり出る (base の rule がそのまま効く)。
- transition: `border-color, box-shadow` を `--duration-fast` 125ms、`--ease-standard` cubic-bezier(0.24, 1, 0.4, 1)。reduced motion は 0s (inputStyles.stylex.ts:58-63)。

#### 25.4.2 編集面・placeholder・status icon・counter

- `editorBody` (:156-172): width 100%、padding 上下 `--spacing-1` 4px、左右 `--spacing-2` 8px。size `lg` だけ上下 `--spacing-2` 8px (sm / md は 4px。:243-249)。
  - toolbar があるとき: 上に 1px solid `--color-border` (#00000014 / #ffffff1A) の区切り線。
  - status icon があるとき: inline-end の padding = 8px + `--spacing-6` 24px = **32px** (20px icon + gap)。
- `editorRoot` (:190-207): position relative、width 100%。`font-family` body、`font-size` `--text-body-size` 14px (iOS の coarse pointer だけ max(16px, body) — ネイティブでは coarse adaptation の 16px を使えばよい)、`line-height` `--text-body-leading` 20px。
- ContentEditable (:173-181): block、width 100%、枠・padding・outline なし。文字色 `--color-text-primary`。`min-height` = `minHeight` prop (既定 4.5rem = **72px**)。**focus ring は描かない** (focus の表示は枠の focus-within だけ)。
- placeholder (:182-189): absolute、top 0、inline-start 0 (= editorRoot の左上 = 1 行目の text と同じ位置)、文字は body と同じ 14/20、色 `--color-text-secondary` (#474747 / #9e9e9e)。pointer を通す、選択できない、aria-hidden。内容が空 (paragraph 1 つで文字なし) のときだけ。
- status icon (:208-215): absolute、top `--spacing-2` 8px、inline-end 8px、z-index 1、20px の status icon (5. FieldStatus 節の icon と色。`useInputStatusIcon`)。statusVariant=tooltip のときは focus できる info-tip button。
- counter (:226-236): 枠の外、下。`display: flex; justify-content: flex-end`、margin-top `--spacing-1` 4px、`--text-supporting-size` 12px (line-height は継承。Field の中なので 20px)、色 `--color-text-secondary`。`charCount > maxLength` で `--color-text-error` (#c0262c / #ffaea7)。表示は `"{count}/{max}"` (plain-text の文字数)。

高さの例 (fine pointer、md、toolbar なし): 1 (枠) + 4 + 72 + 4 + 1 = **82px**。toolbar あり: 1 + 36 (toolbar 帯) + 1 (区切り) + 4 + 72 + 4 + 1 = **119px**。lg は +8px。

```
 label  Optional                                  ⓘ
 description text (secondary)
┌──────────────────────────────────────────────────────────────────┐ ← 1px border-emphasized, r10
│↶ ↷ │ ¶ Paragraph ⌄ │ B  I  U  S  </>  🔗 │ …endContent  → scroll  │ ← 4px + 28px control + 4px
├──────────────────────────────────────────────────────────────────┤ ← 1px --color-border
│ Write something…                                         (⚠)      │ ← 4px/8px inset, placeholder secondary
│                                                                    │   (status icon top8/end8, padding-end 32)
│                                                   min-height 72px  │
└──────────────────────────────────────────────────────────────────┘
                                                              12/280  ← supporting 12px secondary
```

#### 25.4.3 Toolbar (`RichTextEditorToolbar.tsx:132-181, 911-999`)

- Toolbar (core Toolbar、size `sm`): 外側は Section、block padding `--spacing-2` 8px、min-height `--size-element-sm` 28px、gap `--spacing-1` 4px、背景なし (透明)。
- RichText だけの変更 `compactToolbar` (:158-163): Section の inline padding を `--spacing-1` 4px にする。margin-block を上下 -4px にする。結果: **control は枠の内側から上 4px・下 4px・左右 4px** の位置。toolbar 帯の高さ = 4 + 28 + 4 = 36px。
- action 行 (HStack gap 4px、`actions` :171-180): `flex: 1 1 0; width 0; min-width 0; max-width 100%`、`overflow-x: auto; overflow-y: hidden`、`overscroll-behavior-x: contain`、scrollbar thin。**幅が足りないときは行全体が横 scroll する**。control は縮めない。Selector は `fixedSlot` (flex-shrink 0) に入れて潰れないようにする。
- 並び: Undo、Redo、│、Block format Selector、│、Bold、Italic、Underline、Strikethrough、Inline code、Link、(│ + endContent)。
- Divider (vertical): 1px、色 `--color-border` (#00000014 / #ffffff1A)。`align-self: stretch; height auto` で行の高さ (28px) いっぱい。
- Undo / Redo: IconButton ghost、sm 28×28 (icon only は正方形)、radius `--radius-element` 10px、icon 16px (stroke 2 / viewBox 24)、色 `--color-text-primary`。tooltip "Undo" / "Redo"。RTL では glyph を左右反転 (`rtlStyles.mirror`)。disabled (`!editable || !canUndo`) で opacity 0.5。
- Block format Selector: ghost、size sm (28px)、label 非表示 (aria-label "Block format")、`startIcon` = 今の block の glyph。選んでいる option の label (例 "Paragraph") + chevron を表示。見た目は 13. Selector 節の ghost / sm。option は icon 付き: `¶`、`H1`、`H2`、`H3` (TextGlyph: `--text-supporting-size` 12px、`--font-weight-bold` 700、`tabular-nums`。:147-151)、Bulleted list (3 本線 + 点)、Numbered list (3 本線 + 1 2 3)、Block quote (引用符)。
- 書式 ToggleButton (core ToggleButton = Button ghost): sm 28×28、icon only、icon 16px。
  - rest: 背景透明、icon `--color-text-primary`。
  - hover (hover:hover): `--color-overlay-hover` を重ねる (#0000000D / #ffffff0D)。
  - 押している間 (mouse :active): `--color-overlay-pressed` (#0000001A / #ffffff1A) + `scale(0.98)`。
  - pressed (aria-pressed=true。その書式が selection に付いている): 背景 `--color-overlay-pressed` (#0000001A / #ffffff1A) (ToggleButton.tsx:44-66)。forced-colors では `Highlight` / `HighlightText`。
  - focus-visible: 2px solid `--color-accent`、offset 3px (`--focus-outline-offset`)。
  - disabled (editor が read-only / disabled): opacity 0.5。
  - icon only の toggle は label を tooltip に自動で出す (ToggleButton.tsx:178-183)。
  - Link: pressed = (caret が link の中) または (dialog が開いている)。
- icon (すべて 16×16、viewBox 24、stroke currentColor 2px、round cap/join。:232-418): Bold `M6 4h7a4 4 0 0 1 0 8H6zM6 12h8a4 4 0 0 1 0 8H6z`、Italic `M10 4h6M8 20h6M14 4l-4 16`、Underline `M6 4v6a6 6 0 0 0 12 0V4M5 21h14`、Strikethrough `M4 12h16M8 6a4 3 0 0 1 8 0M8 16a4 3 0 0 0 8 0`、Code `M9 8l-4 4 4 4M15 8l4 4-4 4`、Link (2 つの鎖)、Quote `M6 17h3l2-4V7H5v6h3zM15 17h3l2-4V7h-6v6h3z`、Bullet (線 3 本 x 9→20 + 点 r1.5 at x 4.5)、Number (線 3 本 x 10→20 + 文字 1/2/3 fontSize 6)、Undo `M9 7L4 12l5 5M4 12h11a5 5 0 0 1 0 10`、Redo `M15 7l5 5-5 5M20 12H9a5 5 0 0 0 0 10`。path はそのまま kivact の path icon にできる。

#### 25.4.4 Link dialog (`RichTextEditorToolbar.tsx:1000-1069`)

- Dialog: purpose form、width 400px、背景 `--color-background-surface`、radius `--radius-container` 12px、shadow `--shadow-high`、backdrop `--color-overlay` (#00000080 / #000000CC) + blur 2px (Dialog.tsx:186-240)。中身の余白と header / footer は Dialog / Layout の規約 (Dialog 節)。
- header: title "Insert link" (新規) / "Edit link" (既存の link の上)、close button。
- content: TextInput "URL" (label 表示、width 100%、autofocus)。error のとき statusVariant detached で "Enter a valid http, https, mailto, or tel URL."。
- footer: 右寄せ、gap 8px。編集時だけ左端に "Remove link" (destructive: 背景 `--color-error` #c9303a / #ff705d、文字 `--color-on-error` #ffffff / #111111)、"Cancel" (secondary)、"Add link" / "Update link" (primary: 背景 accent、文字 on-accent)。

#### 25.4.5 文書の typography と block (`editorTheme.ts`。editor と view で共通)

共通規則:
- **block の間隔は CSS の margin で、隣どうしは大きい方だけ残る (margin collapse)**。文書の最初の block は上 margin 0、最後の block は下 margin 0 (decorator の caret 境界要素の隣でも 0。:77-88)。ネイティブでは `gap(prev, next) = max(prev.margin_end, next.margin_start)`、最初と最後は 0 として計算する。
- 方向: 各 block は周り (surface) の direction を継ぐ (`direction: inherit`。Lexical の `dir=auto` を打ち消す。:62-70)。block 内の bidi は Unicode の規則のまま。
- **prose measure**: paragraph / heading / quote / list は `max-width: 680px`。code と table は editor の幅いっぱいまで使える。

| block | font (fine / coarse) | 上 margin | 下 margin | その他 | file:line |
|---|---|---|---|---|---|
| paragraph | body 14/20 / 16/24, 400, 色継承 | 12px (`--spacing-3`) | 12px | max-width 680。weight と color は継承 (table header cell 内で効く) | :71-94 |
| h1 | heading-1 24/32 / 26/36, 600 | 24px (`--spacing-6`) | 12px | max-width 680 | :95-109 |
| h2 | 20/28 / 22/32, 600 | 24px | 12px | | :110-124 |
| h3 | 17/24 / 19/28, 700 | 24px | 12px | | :125-139 |
| h4 | 14/20 / 16/24, 700 | 16px (`--spacing-4`) | 8px (`--spacing-2`) | | :140-154 |
| h5 | 12/20 / 14/20, 600 | 16px | 8px | | :155-169 |
| h6 | 10/16 / 12/20, 600 | 16px | 8px | | :170-184 |
| quote | body (継承) | 16px | 16px | 25.4.5.2 | :185-201 |
| ul / ol | body | 12px | 12px | 25.4.5.3 | :202-227 |
| code block | code 14/20 / 16/24 | 16px | 16px | 25.4.5.6 | :432-478 |
| table (wrapper) | body | 16px | 16px | 25.4.5.7 | :303-364 |
| hr | – | 24px | 24px | 25.4.5.8 | :400-417 |
| plugin block | paragraph と同じ | 12px | 12px | 25.4.5.9 | theme `markdownExtensionBlock` |

heading は `--font-family-heading` (= Figtree)。heading の後ろの paragraph の間隔は max(12, 12) = 12px、h4 の後ろでも max(8, 12) = **12px** (collapse のため)。paragraph → h1 は max(12, 24) = 24px。

##### 25.4.5.1 inline の書式 (:282-299)

| mark | 見た目 |
|---|---|
| bold | `--font-weight-semibold` **600** (700 ではない) |
| italic | `font-style: italic` |
| underline | `text-decoration: underline` |
| strikethrough | `text-decoration: line-through`。DOM では `<del>` で包む (意味だけ。見た目は同じ) |
| underline + strikethrough 同時 | UNRESOLVED: 2 つの class がどちらも `text-decoration` を 1 つだけ指定するので、CSS ではどちらか一方だけになる (StyleX の出力順で決まる。確認できなかった)。ネイティブでは**両方の線を描く**ことを推奨 (意図に近い) |
| inline code | font `--font-family-code`、**大きさは周りの文字を継承** (heading の中なら heading の大きさ。`--text-code-size` ではない)、背景 `--color-background-muted` (#f1f1f1 / #1b1b1b)、左右 padding `--spacing-1` 4px、radius `--radius-inner` 6px。上下 padding なし (背景は行の inline box の高さ = font の ascent+descent 分) |
| link | 色 `--color-text-accent` (#1b1b1b / #f1f1f1)、underline、cursor pointer (disabled では default)。auto link も同じ見た目 |
| bold italic | `<em><strong>`。見た目は 600 + italic |

##### 25.4.5.2 quote (:185-201)

- margin-inline 0、max-width 680、`padding-inline-start: --spacing-4` 16px、`border-inline-start: 2px solid --color-border-emphasized` (#d4d4d4 / #525252)。文字色 `--color-text-secondary` (#474747 / #9e9e9e)。文字は body 14/20。中の改行 (hard line break) はそのまま次の行。RTL では線が右側。
- quote は入れ子にならない (`> >` の内側の `>` は文字)。

##### 25.4.5.3 list (:202-281)

- ul / ol: `padding-inline-start: --spacing-6` 24px、marker は外側 (`list-style-position: outside`)、max-width 680。
- marker は **入れ子の深さ (ul と ol を区別せず数える) を 3 で割った余り** で決まる: ul は disc / circle / square、ol は decimal / lower-alpha / lower-roman (:228-240)。marker の色は文字色 (primary)、大きさは body。番号付きは `start` から数える (例 27. 始まり)。
- li: padding 上 4px (`--spacing-1`)、下 4px、margin 下 2px (`--spacing-0-5`。最後の li は 0)、margin 上 0。→ 行の text と次の行の text の間は 4 + 2 + 4 = 10px。
- 入れ子 list: Lexical は入れ子 list を **marker なしの専用 li (wrapper)** に入れる (親 item の次の兄弟)。wrapper の li は padding 上 0、margin 上 `--spacing-2` 8px、marker なし。親 li の margin 下 2px と wrapper の margin 上 8px は collapse して 8px。→ 親 item の行 box の下端から入れ子 list の上端まで = 親 li の下 padding 4 + 8 = **12px** (:241-245 のコメントどおり)、入れ子の最初の text まではさらに入れ子 li の上 padding 4px で 16px。入れ子 list 自身の margin は first/last-child なので 0。
- task item (25.4.5.4)。

##### 25.4.5.4 task list (:264-281 + TaskCheckboxPlugin.tsx)

- task は「bullet list の item の性質」。同じ list に task item と普通の item が混ざってよい (普通の item は marker を出す)。番号付き list の item は task にならない (TaskCheckboxPlugin.tsx:84)。
- task item: marker なし、`margin-inline-start: -24px` (list の 24px の字下げを打ち消す)、`padding-inline-start: 20px + 8px = 28px`。→ **text は list の外端から 28px**。
- checkbox: core CheckboxInput の size `sm` (20×20 の box。見た目は 8. CheckboxInput 節)、label 非表示 (accessible name = item の text)。位置は absolute: 上端 = item の上端 + item の padding-top (4px) = 1 行目の行 box の上端 (body 14/20 なら行の高さと box の高さが同じ 20px で揃う。coarse 16/24 では box が行の上寄せになる)、横 = item の box の左端 (RTL は右端 − 20px)。
- 状態: checked / unchecked。editor では押すと切り替わる。view と read-only / disabled の editor では read-only。tab stop にはならない (tabindex −1)。

##### 25.4.5.5 link (25.4.5.1 を参照)

- 新規 link は toolbar / dialog か Mod+K、または auto link。hover の見た目の変化はない (色も線も rest と同じ)。

##### 25.4.5.6 code block (:426-478 + CodeBlockHeaderPlugin.tsx + CodeSyntaxPlugin.tsx)

- 箱: block、`font-family --font-family-code`、`--text-code-size` 14px / `--text-code-leading` 20px (coarse 16/24)、背景 `--color-syntax-background` (#ffffff / #111111)、枠 1px solid `--color-border` (#00000014 / #ffffff1A)、radius `--radius-element` 10px。
- padding:
  - 言語あり (`data-language` があって `plaintext` でない): 上 = 8 + 28 + 12 = **48px** (header の分)、右 (inline-end) 16px。
  - 言語なし: 上 12px、右 = 16 + 28 + 2×8 = **60px** (隅の copy button の分)。
  - 下 12px、左 (inline-start) 16px。
- 幅: `width: fit-content` (一番長い行の幅)、`min-width: min(680px, 100%)`、`max-width: 100%`。box-sizing border-box。
- 長い行は**折り返す** (`white-space: pre-wrap; word-break: break-all; overflow-wrap: break-word`)。横 scroll はしない (編集面の中の scroll 領域は keyboard で scroll できないため。:428-431)。
- margin 上下 16px。
- 文字色: tokenizer の token ごとに `--color-syntax-<type>` (25.4.0)。token にならない部分は文字色を継承 (`--color-text-primary`)。言語が分からない block は色なし。
- header (言語あり。CodeBlockHeaderPlugin.tsx:46-90, 175-181): code の箱の border の内側の左上に重ね、幅 = 箱の内側の幅。flex、`justify-content: space-between`、`align-items: center`、padding 上下 8px・左右 16px。
  - 左: 言語ラベル。`codeLanguages` に一致すればその `name`、なければ fence に書いた語そのまま (例 `ts`)。`--text-supporting-size` 12px / leading 20px、font code、weight medium 500、色 `--color-syntax-comment` (#6a6a6a / #9e9e9e)、1 行で省略 (`…`)。
  - 右: copy IconButton (ghost、sm 28×28、icon `copy` → copy 後 2 秒 `check`、icon size sm、色 `--color-syntax-comment`)。tooltip "Copy code"。hover / press は ghost button の overlay。
- 言語なしの block: copy button だけを右上の隅 (上 8px・右 8px) に置く (`corner` :59-66)。
- header は文書の外の兄弟なので、caret は入らない、Markdown に出ない。block の位置・大きさが変わるたびに次の frame で測り直して置き直す (animation なし)。

##### 25.4.5.7 table (:300-395 + TableColumnFloorPlugin.tsx + TableScrollRegionPlugin.tsx)

- wrapper: `overflow-x: auto`、max-width 100%、1 列 grid (`minmax(0, 1fr)`。親を広げず自分が scroll する)、margin 上下 16px。
- table: `border-collapse: collapse`、width 100%、外枠 1px solid `--color-border`。
- 行: 下に 1px solid `--color-border` (最後の行は透明)。
- cell: padding 8px (上下左右)、`box-sizing: content-box`、`text-align: start` (GFM の列揃え `:---` start / `:---:` center / `---:` end がある列はそれに従う)、`vertical-align: middle`、`overflow-wrap: break-word`、右 (inline-end) に 1px solid `--color-border` (最後の列は 0。外枠と重ならない)。中の paragraph の margin は first/last-child 規則で 0。
- header cell (1 行目): `--font-weight-semibold` 600、色 `--color-text-secondary` (#474747 / #9e9e9e)。
- 列の最小幅 (TableColumnFloorPlugin.tsx:26-55): 各列 `floor = clamp(max(4, ceil(最長 cell の文字数 / 2), min(header の文字数, 20)), ≤ 24)` を `ch` 単位で header cell の `min-width` にする (padding の内側)。文字数は cell の plain text の長さ。
- 選択された cell (drag、Shift+矢印、行 / 列の選択): `outline: 2px solid --color-accent`、offset −2px (cell の内側に描く。背景は塗らない。:365-395)。view と read-only でも drag 選択は ring を描く。
- view では wrapper が `role=group` 名前 "Table"。中身がはみ出しているときだけ tab stop (tabindex 0) になる (矢印で scroll するため)。editor では付けない (caret が scroll させる)。

##### 25.4.5.8 horizontal rule (:396-425)

- 枠は上だけ 1px solid `--color-border` (#00000014 / #ffffff1A)、margin 上下 24px (`--spacing-6`)。
- 選択 (click、または矢印で上に来た): `outline: 2px solid --color-accent`、offset 3px。Backspace で消える。

##### 25.4.5.9 Markdown plugin node (mention / callout など。MarkdownExtensionsPlugin.tsx)

- plugin の renderer が描く (core Markdown と同じ DOM)。inline node は周りの text の太字 / 斜体 / 取り消し線の中に入る (`del > em > strong` の順で包む)。block node は paragraph と同じ間隔と measure。
- plugin が与えられていない node は **source をそのまま文字で** 出す。
- click で node 全体を選択し、`hrSelected` と同じ focus ring (2px accent、offset 3px)。

##### 25.4.5.10 その他

- 選択範囲 (text selection) の色: UNRESOLVED (Astryx は指定しておらず browser の既定 `::selection`)。ネイティブでは platform 既定か `--color-accent-muted` を推奨しない (dark で見えない)。Kivact の text field の既定 selection 色を使う。
- image: package の機能ではない (storybook の demo だけ)。

#### 25.4.6 RichTextView (`RichTextView.tsx:57-71`)

- root: width 100%、position relative、font body (Figtree 14/20)、色 `--color-text-primary`。枠・背景・padding なし (置かれた場所の背景に描く)。
- 中の block は 25.4.5 と同じ。code header の copy button は押せる。task checkbox は read-only。link は押せる (browser が開く)。
- value が壊れていると `errorFallback` (既定 何も描かない) だけ。

#### 25.4.7 layout / RTL / responsive

- field 全体の幅は `width` prop (なければ親に従う)。高さは内容に合わせて伸びる (内部に縦 scroll はない。親が scroll する)。
- 狭いとき: toolbar の行は横 scroll。prose は 680px で止まる (広い editor では右に余白)。code は最長行の幅まで、editor 幅で折り返す。table は自分の wrapper で横 scroll。
- RTL: quote の線は右、list の字下げと marker は右、task checkbox は右端、code header は測った物理座標で置く、undo / redo の glyph は反転、toolbar の並びは右から。
- coarse pointer (touch) かつ < lg: 文字が 25.4.0 の coarse 列になる (body 16/24 など)。

### 25.5 Motion

| 何が | trigger | property | 値 | 時間 / easing | reduced motion | file:line |
|---|---|---|---|---|---|---|
| field の枠 | hover / focus-within / status | border-color, box-shadow | 25.4.1 | 125ms (`--duration-fast`)、cubic-bezier(0.24, 1, 0.4, 1) | 0s (即時) | Field/inputStyles.stylex.ts:57-63 |
| toolbar の button / toggle | hover、押下、pressed、disabled | background-color / background-image (overlay)、color、opacity、transform | 押下で scale 1 → 0.98 (mouse の :active のみ。touch は press model: onset 150ms、離して 200ms fade cubic-bezier(0, 0, 0.2, 1)) | 125ms、ease-standard | 0s | Button/Button.tsx:118-139 |
| toolbar control の tooltip | hover / focus | Layer enter: opacity 0→1、anchor 方向へ 8px、scale 0.95→1 | 表示まで 200ms 待つ。hide bridge 100ms | 165ms (`--duration-fast-max`)、ease-standard、fill backwards | fine pointer: なし / coarse: fade ≤200ms | useTooltip / layerAnimations.stylex.ts |
| Block format Selector の popover | 開く | Layer enter (上と同じ) | | 165ms、ease-standard | 同上 | 13. Selector 節 |
| link Dialog | 開く | opacity 0→1、translateY 16px→0、scale 0.95→1。backdrop は即時 | | 400ms (`--duration-medium-max`)、cubic-bezier(0.24, 1, 0.4, 1)、fill backwards | coarse: opacity だけ 200ms cubic-bezier(0.42, 0, 0.58, 1) / fine: なし | Dialog/Dialog.tsx:145-157, 196-226 |
| link Dialog | 閉じる | 即時に消える (exit animation なし。Dialog 節を確認) | | | | |
| copy icon | copy 成功 | `copy` → `check` の差し替え | 2 秒後に戻る (useClipboard DEFAULT_RESET_AFTER_MS 2000) | 即時 | – | CodeBlockHeaderPlugin.tsx:132-158 |
| task checkbox | 切り替え | CheckboxInput の check の transition | 8. CheckboxInput 節 | | | |
| code header / task checkbox の位置 | 内容・大きさの変化 | 位置の置き直し | 次の frame (rAF) で即時。animation なし | – | – | CodeBlockHeaderPlugin.tsx:199-263 |
| syntax color | 入力 | 色の付け直し | 即時 (変わった行だけ) | – | – | CodeSyntaxPlugin.tsx |
| block の追加 / 削除、書式の切り替え、placeholder の出入り | – | なし (即時) | | | | |
| toolbar の keyboard hint | toolbar に keyboard で入る | 一時的な矢印 hint badge | Toolbar 節 (useKeyboardHint、既定 3000ms で消える) | | | hooks/useKeyboardHint.tsx |

### 25.6 Behaviour & a11y

#### 25.6.1 keyboard (editor の編集面)

| key | 動作 | 出所 |
|---|---|---|
| 文字入力 / IME | caret 位置に入る。書式は caret 前の text の書式を引き継ぐ | Lexical |
| Enter | 新しい paragraph。list の中なら新しい item (空の item で Enter → list を抜ける / 1 段浅く [Lexical])。**task item の中 (空でない) なら新しい未チェックの task**。item の先頭で Enter なら、text と状態は新しい item に移り、残った空の item が未チェックになる | markdownTaskList.ts:164-198 |
| Shift+Enter | 同じ block の中で改行 (hard line break)。Markdown では `\` + 改行で出る (heading の中は改行だけ) | markdownHardLineBreak.ts |
| Mod+B / Mod+I / Mod+U | 太字 / 斜体 / 下線の切り替え (Mod = mac は Cmd、他は Ctrl) | Lexical core |
| Mod+Z / Mod+Shift+Z | undo / redo (Windows では Ctrl+Y も redo [Lexical, inferred]) | HistoryPlugin |
| Mod+K | link dialog を開く (toolbar があり `hasLink` のときだけ。修飾が Mod だけのときに限る。Mod+Shift+K などでは動かない) | RichTextEditorToolbar.tsx:183-191, 819-838 |
| Mod+Enter | caret のある task item のチェックを切り替え (Shift/Alt 付きは無視) | markdownTaskList.ts:200-219 |
| Tab / Shift+Tab | 字下げ / 戻す (list item は入れ子に、code block の中では tab 文字 [Lexical])。**table の中でも cell 移動ではなく字下げ** | TabIndentationPlugin、RichTextEditor.tsx:712-721 |
| Escape → Tab (または Shift+Tab) | editor から focus を出す (WCAG 2.1.2)。Escape で「出る準備」、次の Tab で普通の focus 移動。間に別のキー (修飾キー単独を除く) を押すか blur すると解除。Escape は Lexical の blur を上書きして消費する (popover を閉じる plugin は優先度が上なので先に動く) | RichTextEditor.tsx:794-879 |
| 矢印 | caret 移動。table の中では cell 間を移動。hr に乗ると hr を選択 | Lexical |
| Backspace / Delete | 削除。選択した hr / plugin node はまるごと消える | |
| Shift+矢印 / drag | 範囲選択。table では cell 範囲選択 (ring を描く) | |
| Mod+A / C / X / V | 全選択 / copy / cut / paste (HTML と plain text) | Lexical |

Markdown shortcut (`hasMarkdownShortcuts`、行の先頭で入力): `# `〜`###### ` → 見出し (行頭に空白なし)、`> ` → quote、`- ` / `* ` / `+ ` → bullet list、`1. ` → 番号付き list (その番号から)、` ``` ` / ` ```lang ` → code block、`---` / `***` / `___` → 水平線、inline の `**x**` `__x__` `*x*` `_x_` `~~x~~` `` `x` `` `[text](url)` [Lexical, inferred]。`- [ ] ` は shortcut では task にならない (`- ` で先に bullet になる。task は import か task item の中の Enter で作る。playground.tsx:935-939)。**code block の中では shortcut は効かない**。table を入力 shortcut で作れるかは UNRESOLVED (TABLE は import / export の transformer。shortcut としての動作は確認できず)。

#### 25.6.2 toolbar の振る舞い (`RichTextEditorToolbar.tsx:586-909`)

- Tab で toolbar に入る (tab stop は 1 つ。roving tabindex)。左右矢印で control 間を移動 (Toolbar の `useListFocus` horizontal。Home/End の有無は Toolbar 節)。toolbar → (Tab) → 編集面 の順 (toolbar が DOM で先)。
- 状態の同期: selection が変わるたびと更新のたびに:
  - inline 書式 5 つ: range selection がその書式を持つと pressed。plugin node だけを選んでいるときは、その node 全部が持つ書式。
  - Link: anchor の node かその親が link なら pressed (dialog を開いている間も pressed)。
  - block 形式: anchor の top-level block が list なら一番近い list の種類 (number → Numbered list、それ以外 → Bulleted list)、heading ならその level、quote → Block quote、それ以外 → Paragraph。h4〜h6 にいるとき Selector の値は options にない (UNRESOLVED: 表示がどうなるか。ネイティブでは "Paragraph" 以外の値なしの表示を推奨)。
  - Undo / Redo の enabled は history の can-undo / can-redo。
  - editor が read-only / disabled → toolbar の全 control が disabled。
- 書式 toggle を押す → その書式を切り替え (bold/italic/strikethrough は選択内の plugin node にも付く)。
- Selector で選ぶ → 選択範囲の block をその形式にする (同じ値を選び直しても paragraph に戻らない。冪等)。Bulleted / Numbered は list の挿入 command、quote / heading / paragraph は block の種類を置き換え。
- Link ボタン: mousedown を preventDefault (editor の selection を保つ)。押すと:
  - `promptForUrl` があるとき: link の中なら link を外す。でなければ prompt を呼び、空でない値なら link にする。
  - ないとき: dialog を開く。URL 欄の初期値 = 既存 link の URL、なければ選択 text が `http` で始まればそれ、なければ `https://`。編集時 (link の中) は title "Edit link" で Remove ボタンあり。
  - submit (Enter / Add / Update): trim → `sanitizeUrl`。`about:blank` (不正) なら error "Enter a valid http, https, mailto, or tel URL." を出して閉じない (入力するとすぐ消える)。正しければ保存した selection を戻して link を付ける (選択が空で link 化できないときは URL を text とした link を caret に挿入)。`hasNewTabLinks` なら target=_blank rel=noopener noreferrer を node に書く。dialog を閉じて次の frame で editor に focus を戻す。
  - Remove: selection を戻して link を外し、閉じて editor に戻る。Cancel / Escape / close: 閉じる (focus は Dialog の focus return)。
- endContent の control は editor が編集できないとき自分で disabled にする (playground の規約)。

#### 25.6.3 auto link

- 入力や paste の text に `URL_MATCHER` / `EMAIL_MATCHER` が一致すると AutoLinkNode にする。URL は sanitize (scheme なしは https://)、email は `mailto:`。新しいタブ属性付き。auto link は Markdown では **元の text のまま** 出る (link 構文にしない)。

#### 25.6.4 task / table / code / rule / plugin node

- task checkbox を押す → その item の checked を切り替え (editor の history に入る)。keyboard では Mod+Enter (checkbox は tab stop を持たない)。
- table: 矢印で cell 間。Tab は字下げのまま (Escape→Tab で出られるように)。cell の結合と cell の背景色はない。新しい table は endContent などから `INSERT_TABLE` command で (header は行だけ)。
- code block: copy button は block の text 全部 (`getTextContent`) を clipboard に入れ、ラベルが 2 秒 "Copied" になり、"Copied" を読み上げる。言語は fence の info string の最初の語。
- hr: click か矢印で選択 (ring)、Backspace で削除。
- plugin node: caret はまたいで進む (中に入らない)。Backspace / cut / copy / paste は 1 つの塊として扱う。click で全体選択。

#### 25.6.5 mention / typeahead

- **Astryx の package には mention の typeahead (`@` で候補 popup) はない**。mention は「Markdown plugin」(`createMarkdownPlugin` で作った syntax、例 `@{ada}`) として import され、atomic な inline node として描かれるだけ (markdownExtensions.ts、stories の `RichTextEditor.markdownPlugins.tsx:30-127`。playground の "Mention" ボタンは caret に `@{ada}` を挿入するだけ)。transform を持つ plugin は `RichTextExtensionError` で拒否 ("RichText cannot adopt Markdown plugin \"…\"")、同じ plugin を 2 回採用しても拒否。
- Kivact でも typeahead は本 component の範囲外 (25.7 の後段の任意項目)。

#### 25.6.6 a11y (role / name / state)

- 編集面: `role=textbox`、`aria-multiline=true`。名前 = label (`aria-labelledby`)、label を隠すときは `aria-label`。`aria-describedby` = description、status message (tooltip 以外)、tooltip layer (tooltip のとき)、placeholder、counter、tab escape hint の順。`aria-placeholder`。`aria-required` (required かつ optional でない)、`aria-invalid` (status error)。
  - read-only: `aria-readonly=true`、tabindex 0 (tab 順に残る)。
  - disabled: `aria-disabled=true`、aria-readonly なし、tab 順から外れる。
- 文書の中: heading / list / listitem / link / table (row, columnheader, cell) / separator は native role。取り消し線は `<del>` (deletion)、太字斜体は emphasis + strong。
- task item: list item の role のまま、`aria-owns` で checkbox を中に持つ。checkbox: role checkbox、名前 = item の text、checked 状態。
- toolbar: `role=toolbar` 名前 "Text formatting"、中の group "Formatting actions"、separator に名前 ("History and block formats"、"Block and inline formats")。toggle は `aria-pressed`、Link は `aria-haspopup=dialog` + `aria-expanded`。
- RichTextView: textbox ではない (role を消す。名前は持たない)。文書の内容として読まれる。label 付きの read-only field が欲しいなら `RichTextEditor isReadOnly`。
- live: counter は 80% (`COUNTER_WARNING_THRESHOLD`) 以上で polite に "n characters remaining" / 超過で "n characters over the limit"。code の copy で "Copied"。
- axe: view も editor も axe violation なし (spec)。

#### 25.6.7 既定の文言 (英語) と提案 i18n key

| Astryx key | 英語 | 提案 key |
|---|---|---|
| `@astryx.richTextEditor.tabEscapeHint` | Press Escape then Tab to move focus out of the editor. | `soft.rich_text_editor.tab_escape_hint` |
| `…toolbar.label` | Text formatting | `soft.rich_text_editor.toolbar.label` |
| `…toolbar.actions` | Formatting actions | `soft.rich_text_editor.toolbar.actions` |
| `…toolbar.undo` / `.redo` | Undo / Redo | `…toolbar.undo` / `…toolbar.redo` |
| `…toolbar.historyDivider` | History and block formats | `…toolbar.history_divider` |
| `…toolbar.blockFormat` | Block format | `…toolbar.block_format` |
| `…toolbar.formatDivider` | Block and inline formats | `…toolbar.format_divider` |
| `…toolbar.paragraph` | Paragraph | `…toolbar.paragraph` |
| `…toolbar.heading1/2/3` | Heading 1 / Heading 2 / Heading 3 | `…toolbar.heading1` 〜 `heading3` |
| `…toolbar.bulletedList` / `.numberedList` / `.blockQuote` | Bulleted list / Numbered list / Block quote | `…toolbar.bulleted_list` / `numbered_list` / `block_quote` |
| `…toolbar.bold` / `.italic` / `.underline` / `.strikethrough` / `.inlineCode` / `.link` | Bold / Italic / Underline / Strikethrough / Inline code / Link | `…toolbar.bold` … `…toolbar.inline_code` / `…toolbar.link` |
| `…link.insertTitle` / `.editTitle` | Insert link / Edit link | `soft.rich_text_editor.link.insert_title` / `edit_title` |
| `…link.url` | URL | `…link.url` |
| `…link.invalidUrl` | Enter a valid http, https, mailto, or tel URL. | `…link.invalid_url` |
| `…link.remove` / `.cancel` / `.add` / `.update` | Remove link / Cancel / Add link / Update link | `…link.remove` / `cancel` / `add` / `update` |
| `@astryx.codeBlock.copyCode` / `.copied` | Copy code / Copied | `soft.rich_text_editor.code.copy` / `copied` |
| `@astryx.table.label` | Table | `soft.rich_text_editor.table.label` |
| `@astryx.textArea.charactersRemaining` / `…OverLimit` | {count} character(s) remaining / over the limit | TextArea と共有 (`soft.text_area.characters_remaining` / `characters_over_limit`) |
| (Field) | Optional / Required | Field と共有 |

#### 25.6.8 Markdown の import / export (round-trip の保証)

規則の出所: `markdownSource.ts` (source 保存の核)、`markdownTable.ts:289-309` (`DEFAULT_TRANSFORMERS` の順: THEMATIC_BREAK → LIST_EXPORT → TASK_LIST → TILDE_CODE → Lexical TRANSFORMERS (ORDERED_LIST / LINK / CODE / QUOTE / HEADING を差し替え) → HARD_LINE_BREAK → TABLE)。

**保証 (spec:AST-062)**
1. **何も変えていない top-level block は、import したときの byte のまま export される** (字下げ、escape、文字参照、fence の info / meta、editor が文字として見せる未対応構文 (HTML、参照 link 定義、脚注、front matter)、BOM、CRLF も含む)。40 文書の corpus で byte 一致 (markdownSource.test.tsx:57-112, 190)。
2. **編集した block、追加した block だけを canonical な Markdown で書き直す**。書き直した block は、読み直すと editor で見えていたとおりになる (必要な escape を付ける)。
3. 隣どうしの区切り (空行 1 つか改行 1 つか) は記録を使う。既定は改行 2 つ。

**仕組み (ネイティブでも同じ考え方で実装する)**
- import: 文書を「空行で区切った塊 (chunk)」に分ける (fenced code は空行をまたいで 1 塊。thematic break の行は単独の塊。ただし paragraph 直下の `-` だけの行は setext の下線としてその塊に残す)。塊ごとに parse し、塊から出来た top-level block 群 (group) の先頭に `{authored source, block 数, 後ろの区切り byte, canonical 出力の hash, 改行コード, 次の group}` を記録する。文書全体に `{BOM, 先頭, 末尾, 改行コード}`。
- export: group ごとに「block 数が同じ かつ 今の canonical 出力の hash が記録と同じ」なら authored source をそのまま、違えば canonical に書き直す。
- ネイティブ版では `RichDoc` の各 top-level block に `source: Option<SourceRecord>` を持たせ、編集 command がその block の record を消す (または hash 比較)。

**構文ごとの規則 (要点)**

| 構文 | import | export (書き直し時) | 備考 / test |
|---|---|---|---|
| ATX 見出し | `^ {0,3}#{1,6}\s` (字下げ 3 まで)。4 空白は paragraph | `#`×level + 空白 + text。閉じ `#` は落ちる | setext 見出しは**未対応**: `Title\n---` は paragraph "Title ---" (下線は文字)。heading の中の改行は `\` なし。markdownHeadingMarkers.test.ts |
| paragraph | 行の折り返し (soft break) は空白 1 つに結合 (前後の空白は詰める) | 1 行で書く | markdownSource.test.tsx:489-527 |
| 強調 | `*` `_` `**` `__` `***` `~~` (`==` highlight は Lexical の transformer にあるが見た目の theme なし: UNRESOLVED) | `*x*` `**x**` `***x***` `~~x~~` | |
| inline code | CommonMark の code span (同じ長さの backtick 列で閉じる。両端の空白 1 つを削る) | backtick | markdownCodeSpans.test.ts |
| link | `[t](url "title")`、`<url>` destination、括弧の釣り合い、backslash escape、文字参照の decode | 空白を含む / `<` 始まりは `<url>`、括弧が釣り合わなければ `\(` `\)`、title は `"` を escape | `javascript:` `vbscript:` `data:text/html` (entity 化や大文字も) は**拒否**し、元の text のまま (link にしない)。markdownRefusedLinks.test.tsx |
| auto link (`<https://…>` / 裸の URL) | 文字のまま (AutoLink plugin が後から link にする) | 元の text | |
| image `![a](x)` | 未対応 (`!` + link になる) | | |
| hard break | 行末の 2 空白 / `\` | 記録があればその形、新規は `\` + 改行 | |
| quote | `^ {0,3}>[ \t]?`、lazy continuation、入れ子なし | `> ` | markdownQuoteMarkers.test.ts |
| bullet list | `- * +`。marker を記録 | 記録した marker、既定 `-` | 字下げは CommonMark の内容列で解釈して 4 空白/段に正規化 (markdownListIndentation.ts) |
| ordered list | `N.` (入れ子の start も保持。`1. a\n   27. b` → 27) | `${start+i}. `。`1)` 形式は書き直すと `1.` | markdownOrderedList.test.ts |
| 入れ子 list の export | | 前の item の内容列まで字下げ (最初は +2) | |
| task | `- [ ] ` / `- [x] ` / `- [X] ` (bullet + 空白 1 + `[ ]`/`[x]`/`[X]` + 空白)。`[]`、`[ ]x`、`[y]`、bullet なしは task でない | `- [ ] ` / `- [x] ` (X は小文字に) | markdownTaskList.test.tsx |
| fenced code | ` ``` ` / `~~~` (3 個以上、字下げ 3 まで)。言語 = info の最初の語。閉じ fence は同じ文字で同じ長さ以上。閉じなければ文書末まで | 元の fence 文字と長さ (中に同じ文字の連続があればそれより 1 長く)、info の残り (meta) は保持、言語を変えたら先頭語だけ差し替え | 4 空白字下げの code block は未対応 (文字のまま保存)。markdownCodeFence.test.ts |
| table (GFM) | `|` を含む行 + delimiter 行 (`:?-+:?`)。外側の `|` は任意、字下げ可、短い行は空 cell で補う、`\|` は `|` | `| a | b |` + `| :--- | :---: | ---: | --- |` (canonical)。cell は 1 行に、`|` → `\|` | 未変更なら非 canonical の形のまま。markdownTable.test.tsx |
| thematic break | `-` `*` `_` を 3 個以上 (空白可、字下げ 3 まで)。`--`、`-*-`、`--- x`、4 空白は rule でない | 新規は `---`、import したものは元の形 | markdownThematicBreak.test.tsx |
| 文字参照 | `&name;` `&#d;` `&#xh;` を decode (code の中は除く)。不正な数値参照は文字のまま | 書き直すと literal の `&copy;` は `\&copy;` | markdownCharacterReferences.test.ts |
| backslash escape | `\` + ASCII 記号 → その文字 (code の中は除く) | 行頭の `#` `>` `-` `+` `1.` `---` `===` や `[` `]` `&ref;` を escape | markdownEscapes.test.ts、markdownSpacedBlockStarts.test.ts |
| plugin (extension) | core の parser が認識した source を atomic node に。inline は書式を保持、block は top-level のみ | **常に元の source** | markdownExtensions.test.ts |

code 言語: 組み込み tokenizer (core CodeBlock) = typescript/javascript/tsx/jsx/ts/js、json、html/xml/svg、css/scss/less、python/py、bash/sh/zsh/shell、php、hack、yaml/yml、markdown/md、c/h、cpp/c++/cc/cxx/hpp/hh、sql、rust/rs、kotlin/kt/kts、swift、go/golang。追加 module (`codeLanguages`): dockerfile、graphql、java、lua、objective-c、powershell、ruby、thrift、toml。token の種類: keyword, string, comment, number, function, type, operator, punctuation, variable, constant, property, tag, attribute。header のラベルは listed language の `name`、なければ fence の語そのまま。

### 25.7 Kivact 実装方針

戦略: **NEW kit component** (`soft::rich_text_editor` / `soft::rich_text_toolbar` / `soft::rich_text_view` + `soft::rich_text::RichDoc`)。標準の `kivact::ui::text_area` を restyle しただけでは足りない。部品として `kivact::ui::{text_area, toggle, select, dialog, tooltip, checkbox, button, field}` と `kivact::interaction::{use_text_area, use_toggle_button, use_dialog, use_menubar/use_list (roving), use_tooltip}` を使う。

#### 25.7.1 Kivact でいま出来ること / 出来ないこと (確認した事実)

- **編集面は折り返さない (確認済み)**: `crates/kivact-core/src/primitive/text_editing.rs:164` `/// Accept line breaks. Native editors scroll horizontally and do not wrap.`。`crates/kivact-text/src/lib.rs:63` "Lines are shaped unwrapped"、`shaping.rs:68` が max_width None で shape。`crates/kivact-ui-basic/src/recipes.rs:337-339` も "a native text area does not wrap"。2. TextArea 節と同じ問題。
- 編集面は **plain な `Rc<str>` 1 つ + text style 1 つ**。範囲ごとに変えられるのは **glyph の色だけ** (`TextEditing::highlight`、text_editing.rs:111-126。`TextHighlight {range, color}`、`crates/kivact-core/src/view/editing_data.rs:53`)。太さ・大きさ・斜体・下線・背景は範囲ごとに変えられない。全行が同じ高さ (`kivact-text/src/lib.rs:63-65`)。editor は leaf で、子も custom draw も持てない (text_editing.rs:12-15) → checkbox や copy button を中に置けない。
- **読み取り専用の rich text はある**: `kivact::control::rich_text(Vec<Span>)` (`crates/kivact-core/src/control/text.rs:70`) は 1 段落、**折り返す**。`span()` (`crates/kivact-core/src/view/span.rs:34`) は font_size / weight / color / font_family / italic / letter_spacing / decoration (underline, line-through) / **background** (inline code の帯) を持つ。ただし span の hit test や link の press はない。
- `Draw::layout_paragraph(text, style, max_width)` (`crates/kivact-style/src/draw/paragraph.rs:259`) は折り返しと caret / hit / rects を持つが **plain text だけ** (span なし)。
- 編集の policy (`kivact::interaction::text_edit_policy`, `crates/kivact-interaction/src/text_edit/policy.rs`): 左右 (grapheme)、Home/End は **文書の先頭 / 末尾**、Backspace/Delete、Enter で `\n`、Mod+A/C/X/V。**上下キー、単語移動、undo/redo はない**。`kivact-ui-basic` の code_block (`code_block/view.rs:211-268`) が custom behavior で上下・行の Home/End を足している (手本)。
- clipboard は **plain text だけ** (`kivact::platform::copy_text` / `read_text`)。
- a11y: `Role::TextBox` は常に AccessKit `TextInput` (multiline にならない)。範囲ごとの書式属性は出せない。`Role::Toolbar` はある。
- layout: `style().wrap(true)` (flex-wrap) はある (`kivact-style/src/sheet.rs:385`)。

#### 25.7.2 段階計画

**Phase 1 — MVP (engine の変更なしで作れる)**

1. `RichDoc` (25.3.5) と Markdown の import / export を **GUI なしの純 Rust module** で作る (`soft::rich_text::markdown`)。25.6.8 の規則と keep-source (block ごとの source 記録 + canonical hash) を実装し、Astryx の test (corpus の byte 一致、構文ごとの例) を Rust の unit test に移す。kit は `kivact` だけに依存する決まり (`crates/kivact-ui-basic/Cargo.toml`) なので parser は自前で書く (CommonMark の部分集合。規則は 25.6.8 で閉じている)。外部 crate (pulldown-cmark) を使うかは parent の判断事項として残す。
2. **`rich_text_view` を完全な見た目で作る** (読み取り専用は今の Kivact で再現できる):
   - block ごとに node: paragraph / heading / quote の行 / list item / table cell = `kivact::control::rich_text(spans)` (折り返す。bold = weight 600、italic、underline / line-through (両方同時も可)、inline code = code family + `background(--color-background-muted)`、link = `--color-text-accent` + underline)。
   - 間隔は 25.4.5 の表から `max(prev.end, next.start)` で計算して column の各子の margin にする (CSS の collapse を kit が計算)。prose は `max_width(680)`。
   - list: marker を自前の node (disc / circle / square の小さな円・四角を path で、番号は `rich_text`) にして 24px の列に置く。深さ % 3 で種類。task は `kivact::ui::checkbox` (restyle は 8. CheckboxInput 節) を read-only で marker の位置に (28px の字下げ)。
   - quote: 左 (inline-start) に 2px の線を持つ row。
   - code block: `kivact-ui-basic/src/code_block/view.rs` を手本に、読み取り専用の `TextEditing` + `highlight` で syntax color (色だけで足りる)。tokenizer は core CodeBlock の規則を Rust に移す (`kivact-ui-basic/src/code_block/rust.rs` の形。MVP は ts/js、json、rust、python、bash、css、html から)。**折り返しは出来ない** (native editor) → 長い行は block の中で横 scroll (Astryx との差。後述 fallback)。header (言語ラベル + copy `kivact::ui::button` ghost sm) は TextEditing の外の兄弟 node として上に置ける (Kivact では普通の column なので位置の測定は不要)。
   - table: `style().grid(..)` 相当の行と列 (各 cell は rich_text)、外枠・罫線、header の 600 / secondary、列の最小幅 (TableColumnFloor の式を `ch` ≈ code の "0" の幅で換算)、横 scroll は `kivact::control::scroll(..).axes(Horizontal)` で、はみ出すときだけ focus 可能な `Role::Group` "Table"。
   - hr、plugin node (`MarkdownExtension::render`)。
   - **link の press**: `rich_text` の span は押せない。fallback: link を含む段落だけ「inline flow」にする — 単語ごと (空白で区切った run ごと) の小さな `text` node を `style().direction(Row).wrap(true)` に並べ、link の run は `Role::Link` + `on_press` (+ hover / focus ring) を持つ node にする。bidi の並べ替えと単語の途中での折り返しは失われる (受け入れる)。link がない段落は `rich_text` のまま。
3. **editor の MVP = 「Markdown source 編集 + toolbar + preview」**:
   - 編集面は `kivact::ui::text_area` (2. TextArea 節の restyle / borderless wrapper) で、中身は **Markdown の source 文字列**。保存の正は Markdown なので round-trip は自明に byte 一致 (25.6.8 の保証 1 を満たす)。
   - `highlight` で source に色を付ける: 記号 (`#`, `>`, `-`, `*`, `` ` ``, `[`, `](url)`) は `--color-text-secondary`、code fence の中は syntax color、link text は `--color-text-accent`。太さや大きさは変えられないので見出しは「色だけ」。
   - toolbar (`rich_text_toolbar`) は source を書き換える command にする: Bold → 選択を `**…**` で包む / 外す、Italic `*…*`、Strikethrough `~~…~~`、Inline code `` `…` ``、Underline → **Markdown に下線はない** (Astryx も export では失われる: UNRESOLVED の扱い。MVP では下線ボタンを出さない)、Link → dialog で URL を受けて `[text](url)`、Block format → 行頭の `# ` / `## ` / `### ` / `- ` / `1. ` / `> ` を付け替え、Undo / Redo → kit が持つ history (source の snapshot を `kivact::interaction::EditSession` の Commit ごとに積む。Mod+Z / Mod+Shift+Z を `forward_keys` で拾う)。pressed 状態は caret 位置の Markdown を parse して決める。
   - Mod+K、Mod+Enter (`- [ ]` ↔ `- [x]`)、Enter の list 継続 (前の行の marker を次の行に付ける、空の item で Enter → marker を消す)、Tab / Shift+Tab (行頭に 2〜4 空白 / 戻す) と Escape→Tab の脱出を custom `behavior` で実装 (code_block の behavior が手本)。上下キーは code_block の行移動 behavior を流用。
   - 「Write / Preview」の切り替え (`kivact::ui::tabs` を toolbar の endContent 側に) で `rich_text_view` を同じ枠の中に出す。read-only の editor は最初から preview 表示。
   - Field の部品 (label, description, status, counter, placeholder) は 4. Field / 2. TextArea の kit wrapper をそのまま使う。
4. toolbar の見た目と roving: `Role::Toolbar` の row、`kivact::ui::toggle` (restyle、ghost sm 28px、pressed = overlay-pressed)、`kivact::ui::button` (undo/redo)、`kivact::ui::select` (block format、ghost)、divider、横 scroll は `scroll(..).axes(Horizontal)`。左右矢印の roving は `kivact::interaction::use_menubar` か `use_list` (ListMode horizontal) を使う。手本: `crates/kivact-ui-basic/src/button_group.rs`、`menubar.rs`。link dialog は `kivact::ui::dialog` (Modal) + `kivact::ui::text_input`。

**Phase 2 — engine に「編集面の折り返し」が入ったら** (core への依頼 A: `TextEditing` に `wrap(true)` と幅を渡し、`EditLines` を可変高さの soft-wrap 行にする。`TextGeometry` は soft wrap を既に扱う: `kivact-editing/src/geometry.rs:21`)

- source 編集面が折り返す (TextArea 節と同時に解消)。
- **block 単位の WYSIWYG** に進める: `RichDoc` の block ごとに 1 つの `TextEditing` (block の text style で: heading は 24px/600、code は monospace + 背景、quote は secondary 色 + 左線) を column に並べ、list の marker、task の checkbox、code の header は TextEditing の外の兄弟 node にする (Astryx が overlay でやっていることを layout で素直に出来る)。block 間の caret 移動 (最初の行で ↑ → 前の block の同じ x、Backspace を先頭で → 前の block と結合、Enter → block を分割) は kit の behavior と focus 移動で実装。inline の書式は **色だけ** (`highlight`) と、記号を薄く残す「半 WYSIWYG」(例 `**太字**` を secondary 色の `**` + primary 色の本文)。
- 範囲選択は block の中だけ。Mod+A は「文書全体選択モード」(全 block に選択色を塗り、copy は Markdown 全体) を kit が持つ。

**Phase 3 — engine に「範囲ごとの書式つき編集」が入ったら** (core への依頼 B: `TextEditing` の highlight を `SpanStyle` (weight / italic / decoration / background / font_size) に広げ、shaping に反映。依頼 C: span の hit test と inline box (checkbox、plugin node を行の中に置く)。依頼 D: AccessKit `MultilineTextInput` と text range の書式属性)

- 本当の inline WYSIWYG (記号を隠し、太字は 600、inline code は背景帯)。link の caret 内判定と press、Mod+クリックで開く。
- table の cell ごとの editor と cell 間の矢印移動、cell 範囲選択の ring。
- plugin node (mention) を inline box として atomic に扱う。任意: mention の typeahead (`@` 入力で `kivact::interaction::use_combobox` の popup を caret の rect (`NodeRefExt::edit_session` + `text_layout`) に anchor) — Astryx にはない追加機能なので要相談。

**後回し / 対象外**
- HTML clipboard (`getHTML`、HTML の copy / paste): Kivact の clipboard は plain text だけ (arboard の `set_text`/`get_text`)。copy は Markdown を plain text として出す。paste は plain text を Markdown として解釈する (Markdown なら書式が戻る)。
- Lexical の JSON state、`nodes` / `plugins` / `transformers` / `getEditor`: 捨てる (互換は不要)。
- image (storybook の demo だけ)。
- CSS Custom Highlight API に依存した「DOM を変えずに色を塗る」仕組み: Kivact の `highlight` がその役 (色だけ) をそのまま果たすので問題なし。

#### 25.7.3 ネイティブで難しい点と fallback (見た目を保つ)

| Astryx の仕組み | 問題 | fallback |
|---|---|---|
| 編集面の折り返し (prose、code の `pre-wrap; break-all`) | native editor は折り返さない (text_editing.rs:164) | Phase 1: 横 scroll。field の幅を広めに既定 (例 100%)。preview / view は折り返す。Phase 2 で解消 |
| 範囲ごとの weight / italic / 下線 / 背景 (編集中) | highlight は色だけ | Phase 1–2: 色 + 記号表示。Phase 3 で解消 |
| margin collapse、`:first-child` の margin 0、`:has()` | CSS 依存 | kit が block 列を作るときに数値で計算 |
| code header / task checkbox を測って absolute で重ねる (rAF + ResizeObserver) | 不要 | Kivact では兄弟 node として layout に置く。測定なし |
| `ch` 単位の列最小幅 | `ch` がない | code font ではなく body font の "0" の advance を `kivact` の text measure で測って掛ける (Phase 1 は 7.5px/ch 近似でも可。UNRESOLVED: 正確な換算は font 依存) |
| `color-mix()` の hover ring | – | 計算済み hex (#d4d4d44D / #5252524D) を recipe に直書き |
| backdrop blur 2px (link dialog) | kivact で blur が使えるかは Dialog 節の判断 | 使えなければ blur なしの overlay 色だけ |
| `forced-colors` (Highlight / HighlightText) | Kivact に forced-colors media がない | 対象外 (platform の high contrast は後段) |
| `@media (hover: hover)` / `(pointer: coarse)` | kivact は pointer の種類を知っている前提で recipe 分岐 (他の節と同じ) | 他の節の方針に従う |
| link を browser で開く (`target=_blank`) | ネイティブに browser はない | `on_link(LinkTarget)` を app に渡す。既定は `kivact::platform` の URL を開く API があればそれ (UNRESOLVED: 公開 API の有無は未確認)、なければ何もしない |
| Lexical の `dir=auto` 打ち消し | – | block は親の direction を継ぐ (Kivact の既定) |
| syntax 色の部分再計算 (CodeLineTokens) | 性能の工夫 | MVP は毎回全体を tokenize (block 単位なので十分)。大きな block だけ `kivact-editing` の `LineColors` (行ごとの色 + 持ち越し状態) を使う |

#### 25.7.4 手本にするファイル

- `crates/kivact-ui-basic/src/code_block/view.rs` (読み取り専用 TextEditing + highlight + 上下キーの behavior + copy)、`code_block/lines.rs` / `rust.rs` (token 化)。
- `crates/kivact-ui-basic/src/typography.rs` (heading / paragraph / blockquote / inline_code / bullet_list の見た目を node で作る形。`Role::Heading` + `level`)。
- `crates/kivact-ui-basic/src/input_group.rs` (borderless な内側 field + 外側の枠 + `focus_within`)、`recipes_field.rs`、`recipes.rs:336-341` (text_area の recipe)。
- `crates/kivact-ui-basic/src/button_group.rs`、`menubar.rs` (toolbar の並びと roving)。
- `crates/kivact-core/src/control/text.rs:70` (`rich_text`)、`crates/kivact-core/src/view/span.rs` (`span`)。
- 2. TextArea 節 (枠、counter、折り返し問題の扱い)、4. Field 節 (4.4.4 共通 input container)、13. Selector 節 (ghost Selector)、8. CheckboxInput 節 (task checkbox)。

### 25.8 受け入れ確認

**Phase 1 (MVP) で確認すること**

- [ ] role=textbox (multiline) の編集面が label の名前で見つかる。label を隠しても名前は同じ。description / placeholder / counter / "Press Escape then Tab to move focus out of the editor." が説明として付く。
- [ ] required で required 状態、status error で invalid 状態。read-only は focus でき read-only と読まれ、暗くならない。disabled は focus できず、全体 (toolbar 含む) が 50% の不透明度。
- [ ] 枠: rest #d4d4d4 / #525252 1px、radius 10px、hover で内側 2px の薄い ring、focus-within (toolbar の button に focus があるときも) で枠 #1b1b1b / #f1f1f1 + 内側 2px #f1f1f1 (dark #262626)。125ms で変わる。status の色 (error #c0262c / #ffaea7 など)。
- [ ] toolbar: role=toolbar "Text formatting"。並びは Undo, Redo, │, Block format, │, Bold, Italic, Underline (MVP では出さないなら非表示を明記), Strikethrough, Inline code, Link。control は 28px、枠から 4px 内側、間隔 4px。toolbar の下に 1px の区切り線。幅 240px でも control は潰れず行が横 scroll する。Tab で toolbar に 1 回止まり、左右矢印で移動。
- [ ] 書式 toggle は pressed 状態 (aria-pressed) が caret 位置の書式に従う。押すと選択に書式が付く / 外れる。Undo / Redo が history に従って enabled / disabled。
- [ ] Mod+K で "Insert link" dialog。URL 欄の初期値規則、不正 URL で "Enter a valid http, https, mailto, or tel URL."、Add で link、link 上では "Edit link" + "Remove link"。閉じると editor に focus が戻る。
- [ ] Mod+Enter で task のチェックが切り替わる。Enter で list / task が続く。Escape → Tab で editor の外へ出る。Tab は字下げ。
- [ ] Markdown: 未編集の文書を開いて保存すると byte 一致 (CRLF / BOM / 字下げ / escape 含む)。test corpus を Rust に移して pass。
- [ ] rich_text_view: 25.4.5 の各 block の大きさ・太さ・間隔 (paragraph 12px、h1-h3 上 24 下 12、h4-h6 上 16 下 8、quote / code / table 16px、hr 24px、最初と最後は 0、collapse)、prose 680px、quote の左線 2px #d4d4d4 / #525252 と secondary 文字、list marker の深さごとの種類、task の 20px checkbox と 28px の字下げ、inline code の背景 #f1f1f1 / #1b1b1b と radius 6px、link の色 #1b1b1b / #f1f1f1 + 下線、code block (背景 #ffffff / #111111、枠、radius 10、言語ラベル 12px #6a6a6a / #9e9e9e、copy → "Copied" 2 秒)、table (罫線、header 600 secondary、はみ出すと "Table" group が focus 可能で横 scroll)、hr。docsite の light / dark と並べて目視比較。
- [ ] view の link を押すと `on_link` が呼ばれる。task checkbox は view で変えられない。

**Phase 2 / 3 で追加すること**

- [ ] 編集面で長い段落が折り返す (Phase 2)。block 単位の WYSIWYG で見出しが 24/32 600 で編集できる。
- [ ] 編集中の太字が 600、inline code に背景、link が caret 内で Link pressed (Phase 3)。table の cell 間を矢印で移動、cell 範囲選択に 2px の ring。
- [ ] Markdown shortcut (`# `, `> `, `- `, `1. `, ` ``` `, `---`) が行頭で効き、code block の中では効かない。


---

## 26. 実装ノート: hooks

コンポーネントの裏にある hooks と内部モジュールについて、それぞれの役割、それが課す見た目・挙動の約束、Kivact kit 側でのモデル化を書く。

### 26.1 useInputContainer

Source: `packages/core/src/hooks/useInputContainer.ts` (+ `.doc.mjs`, `.test.tsx`)。土台: `packages/core/src/hooks/useClickableContainer.ts`。

**何をするか**

input wrapper (4.4.4 の共通 input container) の、input 以外の場所 (padding、start icon、status icon の周り、spinner) を押したときに、内側の control を「押したことにする」hook。戻り値は wrapper に付ける `{onClick, onMouseUp}`。

- 転送の仕方 (`useInputContainer.ts:111-133`)。上から順に判定:
  1. popup trigger (`role="combobox"` か `aria-haspopup` が `true|menu|listbox|tree|grid|dialog`) → `.click()`。focus だけでは popup が開かない control (DateInput の `<input type=text role=combobox aria-haspopup=dialog>` は onClick で calendar を開く) のため。type 判定より**先**に見る (:60-66, :116-120)。
  2. `<input>` で type が `text|password|email|number|search|tel|url|date|datetime-local|month|time|week` → `.focus()` (:23-36)。
  3. `<textarea>` → `.focus()`。
  4. その他の HTMLElement (checkbox、radio、file など) → `.click()`。
  5. それ以外で `focus` を持つもの → `.focus()`。
- `useClickableContainer` の安全策 (`useClickableContainer.ts:207-274`):
  - `disabled` なら何もしない (TextInput は `disabled: isDisabled` を渡す)。
  - container 内に text 選択があれば何もしない (drag 選択の終わりの click を転送しない)。
  - click の target と container の間に interactive 要素 (`button, a, input, select, textarea`, `[role=button|link|checkbox|radio|switch|tab|menuitem|option|combobox|listbox|slider|spinbutton]`, 入れ子の `[data-pressable-container]`。ただし `[aria-readonly="true"]` は除く) があれば何もしない。clear button、calendar toggle、status button、description 内の link の click はそれぞれ自分で処理され、input の focus は奪われない (:40-72, :110-112)。open 中の Tooltip / HoverCard の surface 内の press も同じく除外。
  - container に `data-pressable-container="true"` を付ける (:200-205)。
  - `onMouseUp` は href 用の middle-click 処理で、input container では実質何もしない。
- 使用箇所: TextInput、NumberInput、TimeInput、TextArea などの wrapper、FieldLabel の description (label 外の description を押すと control へ転送。`FieldLabel.tsx:258-288`)。
- tests (`useInputContainer.test.tsx`): combobox + `aria-haspopup=dialog` の text input は click される (focus ではない)。`aria-haspopup` だけの control も click。plain `type=text` / `type=number` / textarea は focus (click ではない)。checkbox は click。

**課す見た目 / 振る舞いの契約**

- 共通 input container の**全面が hit target**。高さ 28/32/36px の box のどこを押しても、caret が input に入る (popup trigger なら popup が開く)。padding 4px/8px や 16px icon の上でも同じ。
- container の中の入れ子 control (clear button 20×20、status button、calendar button) は**自分の action だけ**をする。押しても input に focus を移さない。さらに clear button は pointerdown を `preventDefault` するので、既に input にある focus も失わない (4.4.6)。
- disabled の field では container を押しても何も起きない (ただし disabled の input も keyboard の Tab では focus できる。4.4.4 g)。
- 文字を drag 選択して container の上で離しても、選択は壊れない。
- container を押して focus したときの見た目は 4.4.4 e の focus-within (border accent + inset 2px accent-muted)。pointer 由来の focus でも出る (`:focus-within` は modality を見ない)。

**Kivact kit でのモデル化**

- 核は `kivact::view::NodeRef::activate()` (`crates/kivact-core/src/view/node_ref.rs:299`)。doc どおり「HTML の `<label>` を押したときのように、bound node 以下で最初に focus できる node に focus を与え (indicator なし)、その press handler を走らせる。text field は press handler がないので focus だけ」。つまり Astryx の 3 分岐 (focus / click / popup trigger の click) が `activate()` 1 つで同じ結果になる: text field → focus、checkbox → toggle、`on_press` で popup を開く trigger → 開く。disabled / hidden / modal 外では何もしない (= `disabled` guard と同じ)。
- 入れ子 control の除外: `basic::input_group` が同じ形を使っている (`crates/kivact-ui-basic/src/input_group.rs` の `inline_addon`: addon ノードに `.on_press(move || input.activate())` を付け、doc に「Pressing an addon outside its buttons focuses the control」)。kivact の press は内側の button が受けると外側の `on_press` に届かない前提。clear button / status button は普通の `kivact::ui::button` か自前 `node().on_press(..)` にすればよい。
- text 選択の guard: native text field 内の drag は text field 自身が受けるので不要。
- 提案: 共通 input container を **kit 内部の helper 1 つ**にまとめる。

```rust
// crates/kivact-ui-soft/src/input_container.rs (kit 内部。pub(crate))
pub(crate) struct InputContainer {
    root: Node,               // 枠と背景を描く visual control box (4.4.4)
    control: NodeRef,         // 内側の native text field / trigger
    size: ControlSize,
    status: Value<Option<StatusKind>>,
    disabled: Value<bool>,
    read_only: Value<bool>,
    in_group: Option<GroupEdge>,   // InputGroup の中なら角と margin を変える (6.4)
}
pub(crate) fn input_container(control: NodeRef, size: ControlSize) -> InputContainer;
impl InputContainer {
    fn status(self, v: impl IntoValue<Option<StatusKind>>) -> Self;
    fn disabled(self, v: impl IntoValue<bool>) -> Self;
    fn read_only(self, v: impl IntoValue<bool>) -> Self;
    fn start(self, icon: impl IntoView) -> Self;      // 16px secondary
    fn end(self, view: impl IntoView) -> Self;        // spinner / clear / status icon の順
    fn child(self, control: impl IntoView) -> Self;   // borderless native field
}
```

  - `root.on_press(move || if !disabled.get() { control.activate() })`。
  - style は reactive closure 1 つで 4.4.4 (j) の優先順をそのまま書く。`focus_within = control_wrapper_ref.focus_within()` (`node_ref.rs:495`)、`hovered = root_ref.hovered()` (`:541`)、`keyboard_focus = control.focus_visible()` (`:475`。disabled の field の ring 復活用)。
    - rest: `height(control_height)`、`padding_y(4).padding_x(8).gap(8)`、`border(1, BorderEmphasized)`、`radius(10)`、`background(BackgroundSurface)`、`direction(Row).align_items(Center)`、`min_width(0)`。
    - hover: `.shadow(Shadow::new(0, 0, 0, border_emphasized.with_alpha(0.3)).spread(2).inset())`。
    - focus: `border_color(Accent)` + `Shadow::new(0,0,0, AccentMuted).spread(2).inset()`。
    - status: border を status 色。hover 時だけ status の inset shadow (`with_alpha(0.3)` の warning `#f8d36a` / error `#de4745` / success `#2f7d33`)。focus 中は shadow なし。
    - disabled: `opacity(0.5)`、shadow なし。keyboard focus 中だけ accent border + inset 帯。
    - transition: `.transition(TransitionProp::Paint, Transition::tween(125, Easing::CubicBezier(0.24, 1.0, 0.4, 1.0)))` (border color と最初の 2 つの shadow が補間対象。`kivact-style/src/sheet.rs:150-158`)。`decorative` 既定 true で reduced motion は即時。
  - 内側の native field は `basic::input_group` の `borderless()` と同じく枠・ring・背景を透明にし、`disabled` の look と focus の ring も消す (container が描く)。text は 14/20 `TextPrimary`、placeholder `TextSecondary`。
  - InputGroup の中 (`try_context::<InputGroupCtx>()`) なら `GroupEdge` に従い角を落とし `margin_left(-1)`、focus-within で `z_index(1)`。
- `kivact::ui::text_input` / `number_input` の recipe (`UiRecipes::text_input`) は native field 自身の style しか返せず、icon や clear を同じ box に入れられない。そのため **soft の TextInput 系は kit wrapper (container + borderless native field)** にし、標準 `text_input` の recipe には「単体で使われたときの共通 container の look」(4.4.4 a-h を native field に直接) を入れる。native field は `Style::paint` を使えないので、inset shadow は `shadow(..inset())` で出せるか確認する (出せなければ container 方式に寄せる。UNRESOLVED)。
- FieldLabel の description の click 転送も同じ `on_press(move || control.activate())`。

### 26.2 useInputStatusIcon

Source: `packages/core/src/hooks/useInputStatusIcon.tsx` (+ `.doc.mjs`, `.test.tsx`)。使用箇所: TextInput、TextArea、NumberInput、DateInput、FileInput、Selector 系 (ComplexSelector は size `sm`)。

**何をするか**

枠付き input の container の中 (末尾) に置く「on-field status affordance」と、その a11y 配線を 1 か所で作る。入力 `{status?, statusVariant='attached', isInGroup=false, size='md'}`、出力 `{statusIcon: ReactNode, describedBy: string | undefined}`。

- 何も描かない条件 (`useInputStatusIcon.tsx:187-192`): status がない、`statusVariant='detached'` (detached の message box が自分で先頭 icon を持つので重複させない)、InputGroup の中 (group が status を持つ)。このとき `describedBy` も undefined。
- `attached`、または `tooltip` で message がない → **ただの icon** (操作不可、focus 不可)。message は attached の FieldStatus box が持つ。`describedBy` undefined。
- `tooltip` で message がある → **focus できる `<button type=button>`** の中に icon、その tooltip に message (:212-232)。
  - `aria-label` = 種類ごとの文言 (`@astryx.input.statusButton.error` "Error details" / `.warning` "Warning details" / `.success` "Success details")。
  - `aria-describedby` = tooltip の id (message が button の description)。同じ id を `describedBy` として返し、caller は input の `aria-describedby` に足す (input 側でも message が読まれる)。tooltip 要素が DOM にあるときだけ id が出るので、宙に浮いた参照がない (tests `useInputStatusIcon.test.tsx:120-160`)。
  - 開き方: `useTooltip({placement: 'above', isEnabled: hasTooltip, isOpen: tapOpen})`。focus か fine pointer の hover で開く (uncontrolled)。hover できない端末 (`(hover: none)`) では click で `tapOpen` を true/false に toggle して controlled にする (:156-168)。blur で uncontrolled に戻す (:152-154)。tap で開いた状態は document の Escape で閉じる (uncontrolled のときの Escape は useTooltip が処理) (:172-183)。
- icon (:194-203): Icon `STATUS_ICON[type]` (warning → Lucide `AlertTriangle`、error → `XCircle`、success → `CheckCircle`。FieldStatus detached と同じ glyph)、size 既定 `md` = 20×20px、`color={type}` → `--color-icon-warning` (light `#836700` / dark `#eec448`)、`--color-icon-error` (`#c0262c` / `#ffaea7`)、`--color-icon-success` (`#2f7d33` / `#90ca90`)。theme target `astryx-input-status-icon[data-size][data-status]`。
- 見た目の span / button (:52-84):
  - icon anchor: `inline-flex; align-items: center` (container の中で縦中央)。
  - status button: `inline-flex`、中央寄せ、padding 0、margin 0、border なし、背景透明、color inherit、cursor pointer (disabled なら default)、`pointer-events: auto` (TextArea の end slot が `pointer-events: none` でも hover が届くように。test :163)、radius `--radius-full` (9999px → 円)、focus-visible で共有 outline (2px solid `--color-accent` = `#1b1b1b` / `#f1f1f1`) を **offset 2px** (標準 3px だと field の border を越えるため)。button の大きさは icon と同じ 20×20px。
- tooltip の見た目は Tooltip 節 (bg `--color-text-primary`、text `--color-background-surface`、radius 12px、body 14/20、padding 4px 8px、max-width 300px、offset 4px、show delay 200ms、hide bridge 100ms、enter は opacity 0→1 + anchor 方向へ 8px + scale 0.95→1、165ms `cubic-bezier(0.24,1,0.4,1)`)。

**課す見た目 / 振る舞いの契約**

- 全ての枠付き input で、同じ status に同じ icon が同じ場所 (container の末尾、clear button の後) に出る。
- container の並び: `[startIcon] [input] [spinner] [clear] [statusIcon]`、gap 8px。status icon 20px は container の内容領域 (md 22px) に収まる (sm では 18px 領域より 2px 大きいが、container は `align-items: center` なので上下に 1px はみ出して中央に置かれる)。
- `attached`: icon は飾り。message は下の box。
- `detached`: container の中に icon はない (box 側に 16px icon)。
- `tooltip`: icon が Tab で止まる button になり、focus / hover / tap で上に message の tooltip。button の name は "Error details" など、description が message。input も message で describe される。focus 中は container の focus-within も出る (button は container の中なので border accent。ただし status 付きなので border は status 色のまま、4.4.4 f)。
- InputGroup の中では icon なし。group の status は group 下の detached box (6.4)。
- 色は status border と同じ色 (= `--color-text-{kind}`)。

**Kivact kit でのモデル化**

- kit 内部の helper `soft::status_affordance(..)` を 26.1 の `InputContainer::end(..)` に渡す。

```rust
pub(crate) struct StatusAffordance { pub view: Option<View>, pub describes: Option<NodeRef> }
pub(crate) fn status_affordance(
    status: Value<Option<InputStatus>>,
    placement: StatusPlacement,
    in_group: bool,
    size: IconSize,              // 既定 Md = 20px
) -> StatusAffordance;
```

  - 何も描かない条件は Astryx と同じ (`status.is_none() || placement == Detached || in_group`)。status は reactive なので `show_some(..)` で出し入れする。
  - plain icon: `icon(name)` を 20px、color = kit `Tokens` の `text_warning / text_error / text_success`。`hit_testable(false)` にせず、container の `on_press` → `activate()` に任せる (Astryx でも icon を押すと input に focus が移る)。
  - tooltip 版: `kivact::ui::button(message_key)` ではなく、見た目を持たない focus stop `node().role(Role::Button).name(message("soft.input.status_button.error")).description(msg)` + 20px icon、radius 9999、focus_visible の ring 2px accent (offset 2 は kivact の ring に offset がなければ ring を 2px 外側に描く `draw_over` で近似。UNRESOLVED)。`kivact::ui::tooltip(msg).placement(Placement::Above).attach(button)` で tooltip を付ける。touch の tap toggle は `use_tooltip()` の `shown` を見て `on_press` で切り替える (kivact の tooltip が tap を扱うかは UNRESOLVED。`crates/kivact-interaction/src/tooltip.rs` の `TooltipConfig` を確認)。Escape で閉じる。
  - `describes`: kivact の tooltip の中身は開いているときだけ mount される (`show(handle.shown(), factory)`)。tooltip ノードを `described_by` に使うと閉じている間は参照が切れる。そこで input には **`.description(msg)` (NodeBuilder) を直接付ける**か、常に mount されている見えない text ノードを作って `described_by` に使う。button 側も `.description(msg)`。
  - 色と icon 名は FieldStatus (5.7) と共有: `triangle-alert` / `circle-x` / `circle-check`。
- 文言 key (提案): `soft.input.status_button.error` = "Error details"、`soft.input.status_button.warning` = "Warning details"、`soft.input.status_button.success` = "Success details"。
- 参考: `crates/kivact-ui-basic/src/combobox.rs` (container の中の clear / toggle button)、`crates/kivact-ui/src/tooltip.rs`、`crates/kivact-interaction/src/tooltip.rs` (`use_tooltip`, `TooltipApi::{trigger, content, shown}`)。
- 受け入れ確認:
  - [ ] attached error の TextInput の末尾に 20px の XCircle (#c0262c / #ffaea7)。押すと input に focus。
  - [ ] detached / InputGroup の中では icon がない。
  - [ ] tooltip placement: Tab で icon に止まり、name "Error details"、上に message の tooltip。input の description も message。Escape で閉じる。
  - [ ] message のない tooltip placement は plain icon (focus しない)。


### 26.3 useCombobox

source: `packages/core/src/Selector/hooks.ts:159-670`（`useCombobox`）, `:61-157`（`useSelectedItemOffset`, Selector 専用・非公開）, docs `Selector/useCombobox.doc.mjs`, tests `Selector/useCombobox.test.tsx`, `useCombobox.public.test.tsx`。公開 export（`@astryxdesign/core/Selector`）。利用者: Selector, CommandPalette, プロダクトの自前テキスト欄（spec AST-075）。

**役割**: 単一選択 combobox の「ハイライト（active option）」と鍵盤処理だけを持つ。何も描画しない。DOM focus は常にコントロール（button か input）に留まり、ハイライトは `aria-activedescendant` で伝える。open 状態・値・リスト描画・件数読み上げは caller の責務。typeahead は含まない（caller が `useTypeahead` を前段に置く）。

**入力**: `selectableItems: ReadonlyArray<{value: string; disabled?: boolean}>`（描画順そのまま）、`value?`、`isDisabled=false`、`isOpen`（caller 所有）、`hasSearch=false`、`onOpen(): unknown`（`false` を返すと拒否 → ハイライトを触らない）、`onClose()`、`onSelect?(value)`、`onClear?()`、`onSearchSeed?(char)`、`listboxId`。

**出力**: `highlightedIndex`（-1 = なし）、`setHighlightedIndex`、`getItemId(i) = "${listboxId}-item-${i}"`、`onTriggerClick`、`onKeyDown`（ボタン/トリガー用モデル）、`onItemSelect(item)`、`onItemMouseEnter(item, i)`、`getFieldProps({onKeyDown?})`、`getListboxProps()`、`getOptionProps(i)`（テキスト欄用モデル）。

**状態機械**（state = `isOpen`(外部) × `highlightedIndex`）

| 事象 | ガード | 結果 |
|---|---|---|
| `onTriggerClick` | disabled → 無視 | 開→ `closeAndReset`（highlight=-1, onClose）。閉→ `onOpen()`; 成功かつ !hasSearch なら highlight = 選択値の index（無ければ 0） |
| `selectItem(item)` | disabled or item.disabled → 無視 | `onSelect(value)` → `closeAndReset` |
| hover（`onItemMouseEnter`） | item.disabled → 無視 | highlight=i（**スクロールしない**フラグ付き） |
| highlight 変化（キー由来） | isOpen && i>=0 | `document.getElementById(getItemId(i)).scrollIntoView({block:'nearest'})`（`hooks/useHighlightedOptionScroll.ts`）。hover 由来はスキップ（暴走スクロール防止） |
| close（hook 外で閉じた場合） | | caller が `setHighlightedIndex(-1)` を呼ぶ契約 |

**キーマップ A: `onKeyDown`（ボタン型トリガー / Selector, `hooks.ts:346-497`）** — 修飾キー判定なし、IME ガードなし（Selector の検索欄側で `isImeKeyEvent` を見る）。

| キー | 閉 | 開 |
|---|---|---|
| ArrowDown | preventDefault; `onOpen()` 成功で highlight=0（disabled でも 0 にする） | 次の enabled index（末尾で停止、wrap なし） |
| ArrowUp | preventDefault; 成功で highlight=`len-1` | 前の enabled（先頭で停止） |
| Space | hasSearch なら素通り（文字入力）。それ以外は Enter と同じ | 同 |
| Enter | preventDefault; 開いて !hasSearch なら選択値 or 0 をハイライト | highlight が enabled なら選択して閉じる |
| Escape | 何もしない（preventDefault もしない） | preventDefault; 閉じる |
| Tab | — | 閉じる（preventDefault しない → focus は自然に移動） |
| Home / PageUp | preventDefault のみ | 先頭 enabled |
| End / PageDown | preventDefault のみ | 末尾 enabled |
| Delete / Backspace | `!hasSearch && onClear && value!=null` なら preventDefault + `onClear()` | 何もしない |
| 1 文字キー（ctrl/meta なし） | hasSearch && onSearchSeed: `onOpen()` 成功なら `onSearchSeed(key)` | hasSearch: `onSearchSeed(key)`（focus が検索欄へ移るまでの取りこぼし防止） |

**キーマップ B: `getFieldProps().onKeyDown`（自前の 1 行テキスト欄, `:515-597`）** — `isDisabled`・IME 変換中・修飾キー付き（Escape は例外）は何もしない。

| キー | 閉 | 開 |
|---|---|---|
| ArrowDown | items>0 なら `onOpen()`→ preventDefault, highlight=最初の enabled（無ければ -1） | preventDefault; 次の enabled、**末尾→先頭へ wrap**。何も無ければ先頭 |
| ArrowUp | 何もしない（caret に任せる） | preventDefault; 前へ wrap、何も無ければ末尾 |
| Enter | 何もしない | highlight が enabled のときだけ preventDefault + 選択（それ以外は欄自身の Enter: 送信・改行） |
| Escape（修飾キー可） | — | preventDefault（外側レイヤーを同じ押下で閉じさせない）+ 閉じる |
| その他（Home/End 含む） | 欄へ | 欄へ（Home/End の扱いは AST-075 OQ2 で未決） |

caller の `onKeyDown` は hook が preventDefault しなかったときだけ後段で呼ばれる。

**active-descendant モデル**

- option id = `${listboxId}-item-${index}`（index は `selectableItems` = 描画順の flat index。section/divider は数えない）。
- ボタン型（Selector）: トリガー（検索ありなら検索 input）の `aria-activedescendant` = `isOpen && highlight>=0 ? getItemId(highlight)`（Selector はさらに listbox マウント済みを条件にする）。option の `aria-selected` は**値と一致**（Selector 側で付与）。
- 欄型（getter）: `isListboxShown = isOpen && items>0`。`aria-expanded`・`aria-controls` はこの値（空リストは listbox と見なさない）。`aria-activedescendant` は shown かつ範囲内のときだけ。`getOptionProps(i)` は `aria-selected = (i === highlight)`（**active を selected として出す**）、`aria-disabled`、`onMouseMove`（hover ハイライト、同 index なら何もしない）、`onClick`（選択）。`getListboxProps` は `onMouseDown: preventDefault`（押下で欄の focus を失わない）。
- 件数の読み上げは caller が `useAnnounce` で行う（AST-075 DEC-2）。0 件時は欄に表示している文言を読む。listbox の中に option 以外（create ボタン、empty 文言）を置かない。

**テストで固定されている挙動**（`useCombobox.test.tsx`）: 閉じていると collapsed で参照なし／開くと expanded + listbox 名付き／開いていても 0 件なら listbox を出さず collapsed／矢印で active が移り focus は欄のまま（FR1）／active 無しの ArrowUp は末尾から／Enter は active があるときだけ欄の Enter を止める／Escape は外側レイヤーを閉じさせない／0 件の open リストも Escape で閉じる／ArrowDown で隠れたリストを開く・0 件なら開かない／矢印・Enter・Escape 以外は欄へ届く／Shift+Enter 等の chord は欄のもの／`onKeyDown`（トリガー型）は従来どおり／IME 変換中のキーは移動・選択・閉じのいずれもしない／hover で active、click で選択、focus は欄／listbox 自体の押下でも focus は欄。

**Kivact との比較**

| 項目 | Astryx useCombobox | `kivact::interaction::use_combobox`（`crates/kivact-interaction/src/combobox.rs`） | `kivact::interaction::use_select`（`select.rs`）/ `kivact::ui::select` |
|---|---|---|---|
| focus | コントロールに留まる + activedescendant | 同じ（`input()` part が `active_descendant` と `controls`） | **開くと実 focus が option へ**（roving） |
| ハイライトの単位 | index（描画順） | key（`active: Value<Option<K>>`） | list focus |
| 矢印（開） | トリガー型: 端で停止 / 欄型: wrap | wrap | 端で停止 |
| 矢印（閉） | 開く（トリガー型は 0 / 末尾、欄型は ArrowDown のみ） | ArrowDown/Up で開く | **値を前後に変更**（開かない）。Alt+↓/F4/Enter/Space で開く |
| query 変更時 | ハイライトは触らない（caller が決める） | 最初の enabled match を自動 active | — |
| Enter（閉） | トリガー型: 開く / 欄型: 何もしない | 何もしない | 開く |
| Space / Home / End / PageUp / PageDown / Tab 閉じ / Delete clear / search seed | あり（トリガー型） | なし（Home/End は caret） | Home/End あり、他は一部 |
| option の selected 状態 | トリガー型: 値 / 欄型: active | active（`ListItemState.selected` は値、Role::Option の selected は active） | 値 |
| 閉じ方 | caller の popover（light dismiss） | `popup(open)` は欄の blur で閉じる、Escape | popup の close reason |
| typeahead | 外付け `useTypeahead`（750ms、同字循環） | なし | 内蔵（500ms） |

`kivact-ui-basic/src/combobox.rs` は use_combobox を text field + chips に使う例（`forward_keys(vec![ArrowDown, ArrowUp, Enter, Escape])`、side button の `focus_on_press(input)`、`ComboMode::{Closed,Browsing,Filtering}` で「開いた直後は全件・打鍵後に絞り込み」）。`list_popup.rs` は行 style / check paint。

**kit が公開すべきもの（提案）** — Astryx が公開フックとして持つので soft kit も公開する（kivact-ui-basic の慣習: Copy ハンドル + `Part` を返す、状態は caller 所有）。

```rust
// soft::use_combobox — Selector と CommandPalette 相当が使う
pub fn use_combobox<K: Clone + Eq + Hash + 'static>(
    options: impl IntoValue<Vec<ListItem<K>>>,   // 描画順（section を flatten 済み）
    value: impl Into<Binding<Option<K>>>,
    open: impl Into<Binding<bool>>,
) -> ComboboxModel<K>;                            // Copy
impl ComboboxModel<K> {
    pub fn searchable(self, on: bool) -> Self;            // hasSearch
    pub fn clearable(self, on: impl IntoValue<bool>) -> Self; // onClear 有効化（値を None に）
    pub fn on_search_seed(self, f: impl Fn(char) + 'static) -> Self;
    pub fn disabled(self, v: impl IntoValue<bool>) -> Self;
    pub fn highlighted(self) -> Value<Option<K>>;
    pub fn set_highlighted(self, key: Option<K>);         // スクロールあり（キー由来扱い）
    pub fn hover(self, key: K);                            // スクロールなし
    pub fn trigger(self) -> Part<PopupState>;   // キーマップ A + click トグル + Role::ComboBox/expanded/controls/active_descendant
    pub fn field(self) -> Part<PartState>;      // キーマップ B（text_input に付ける）
    pub fn listbox(self) -> Part<PartState>;    // Role::ListBox, focus_on_press(trigger/field)
    pub fn option(self, key: K) -> Part<ListItemState>; // Role::Option, selected=値(trigger) / active(field)
    pub fn typeahead(self, labels: ..) -> Self; // 任意: 750ms・同字循環・閉=即確定/開=ハイライト
}
```

- 実装は `kivact::interaction::use_popup(open)`（focus 復帰・dismiss 理由・`FocusReturn`）、`kivact::primitive::part(..)` の `.on_key / .active_descendant / .controls / .role / .expanded`、`NodeRef` の registry（key → row NodeRef）で active descendant を解決、`scroll_into_view` 相当（キー由来のみ）。
- 遷移は純関数 `fn step(state: ComboState, key: Key, ctx: &Ctx) -> (ComboState, Effect)` に切り出し、UI 無しの unit test で上表を固定する（Codex に表をそのままテスト化させる）。
- 既知の差を意図的に保つ点: トリガー型の端停止、閉 ArrowDown が disabled の index 0 を指す quirk は「最初の enabled」に直してよい（互換は不要）。`aria-haspopup` 相当の setter が kivact に無い点は **UNRESOLVED**。

### 26.4 useMultiCombobox

source: `packages/core/src/MultiSelector/hooks.ts`（395 行）, docs `MultiSelector/useMultiCombobox.doc.mjs`。公開 export（`@astryxdesign/core/MultiSelector`）。利用者: MultiSelector。

**役割**: 複数選択 combobox のハイライトと鍵盤処理。useCombobox と同じ index ベースだが、**トグルしてもリストを閉じない**、行が「option cell + action cell」の grid になれる、閉トリガー用の typeahead を内蔵。属性は一切付けない（caller が返り値から ARIA を配線）。

**入力**: `selectableItems: MultiSelectorOptionData[]`（**DOM と同順**。MultiSelector は create 行・select-all 番兵 `__xds_select_all__`・選択済み先頭並べを含めた `sortedItems` を渡す）、`isDisabled`、`isOpen`、`hasSearch`、`onOpen`、`onClose`、`onToggle(value)`、`onClear?`、`hasValue`、`listboxId`、`isGrid=false`、`rowHasAction?(i)`、`onActivateAction?(i)`。

**出力**: `highlightedIndex`、`setHighlightedIndex`（必ず cell を `'option'` に戻す）、`highlightedCell: 'option'|'action'`、`getItemId(i) = "${listboxId}-item-${i}"`、`getActionCellId(i) = "${listboxId}-item-${i}-action"`、`activeDescendantId`（`isOpen && i>=0` のとき、grid かつ action なら action cell id、他は row id）、`onTriggerClick`、`onKeyDown`、`onItemMouseEnter(item, i, cell='option')`。

**状態機械**: state = `isOpen`(外部) × `highlightedIndex` × `highlightedCell` × `typeahead` buffer（500ms タイマー）。

| 事象 | 結果 |
|---|---|
| `onTriggerClick` | 開→ 閉じて highlight=-1。閉→ `onOpen()` 成功かつ !hasSearch なら highlight=**0**（選択値ではない） |
| hover option cell | item.disabled でなければ highlight=i, cell=option（スクロールなし） |
| hover action cell | disabled でも highlight=i, cell=action（caller の control は option の可否と無関係に使える, AST-058 FR4） |
| 行移動（キー） | cell は常に option に戻る（FR5）、scrollIntoView nearest |
| トグル | `onToggle(value)`、**開いたまま**、ハイライト維持 |

**キーマップ（`hooks.ts:189-382`）**

| キー | 閉 | 開 |
|---|---|---|
| grid の inline-end（LTR: →, RTL: ←; `isRtlElement(currentTarget)`） | — | 行ハイライト中・cell=option・行に action → preventDefault, cell=action。他は何もせず return |
| grid の inline-start | — | cell=action → preventDefault, cell=option |
| ArrowDown | 開いて highlight=0 | 次の enabled（端停止） |
| ArrowUp | 開いて highlight=len-1 | 前の enabled（端停止） |
| Enter / Space（Space は hasSearch なら素通り） | preventDefault; 開いて !hasSearch なら highlight=0 | cell=action → `onActivateAction(i)`（選択は変えない）。それ以外 enabled なら `onToggle` |
| Tab | — | 閉じる |
| Escape | — | preventDefault; 閉じる |
| Home / PageUp | preventDefault | 先頭 enabled |
| End / PageDown | preventDefault | 末尾 enabled |
| Delete / Backspace | `!hasSearch && onClear && hasValue` → `onClear()` | — |
| 1 文字（!hasSearch, ctrl/meta なし） | バッファ+=key。一致（`findTypeaheadMatch`, enabled のみ, 先頭から, 畳み込み prefix）があれば `onOpen()` して highlight=match。一致無しなら開かずバッファだけ更新 | 同（開いたまま highlight 移動） |

- typeahead は useCombobox 系（`useTypeahead` 750ms・同字循環・現在位置 +1 から）と**異なる**: 500ms、連結のみ、毎回先頭から。
- MultiSelector 側の追加: 検索欄では Enter かつ highlight<0 かつ create 可能で create、grid で行ハイライト中の ←/→ を hook へ渡す、IME 変換中は無視、keyboard modality のときだけ action cell にリングを描く（`readModality()` を各キー/hover 後に読む）。

**active-descendant モデル**: focus はトリガー（or 検索欄、renderTrigger/sheet 時は listbox 自身 tabIndex 0）に留まる。listbox/grid に `aria-multiselectable=true`。row（option）の `aria-selected` = 値に含まれるか（select-all は全 enabled 選択時）。grid の row は `aria-labelledby=${id}-option` で action を名前に含めない。action cell 上では activedescendant が gridcell を指す。

**Kivact との比較**

| 項目 | Astryx useMultiCombobox | `kivact::interaction::use_combobox(.., ListSelection::Multiple)` | `kivact::interaction::use_list`（multiple policy） |
|---|---|---|---|
| トグル後 | 開いたまま、query 維持（create 時のみ caller がクリア） | 開いたまま、**query をクリア** | — |
| 矢印 | 端停止 | wrap | policy 次第、実 focus |
| Backspace（空 query） | 閉・非検索で全解除（onClear） | **最後の 1 件を削除** | — |
| select-all / create / grid action cell | あり（caller が行に混ぜる + hook が cell を管理） | なし | なし |
| typeahead | 内蔵 500ms | なし | 内蔵（`use_list` の typeahead） |
| focus | activedescendant | activedescendant | roving |

`kivact-ui-basic/src/combobox.rs` の `Vec<K>` モード（chips + 検索欄）が最も近い既存 UI だが、Astryx MultiSelector はボタン型トリガー + パネル内検索で構造が違う。

**kit が公開すべきもの（提案）**

```rust
pub enum MultiRow<K> { Create(String), SelectAll, Option { key: K, disabled: bool, has_action: bool } }
pub enum MultiCell { Option, Action }

pub fn use_multi_combobox<K: Clone + Eq + Hash + 'static>(
    rows: impl IntoValue<Vec<MultiRow<K>>>,   // 描画順（create→select-all→選択済み先頭並べ）
    values: impl Into<Binding<Vec<K>>>,
    open: impl Into<Binding<bool>>,
) -> MultiComboboxModel<K>;                    // Copy
impl MultiComboboxModel<K> {
    pub fn searchable(self, on: bool) -> Self;
    pub fn grid(self, on: bool) -> Self;                       // 一度 true になったら固定（latch）
    pub fn on_create(self, f: impl Fn(String) + 'static) -> Self;
    pub fn on_activate_action(self, f: impl Fn(K) + 'static) -> Self;
    pub fn clearable(self, on: impl IntoValue<bool>) -> Self;
    pub fn disabled(self, v: impl IntoValue<bool>) -> Self;
    pub fn highlighted(self) -> Value<Option<usize>>;
    pub fn highlighted_cell(self) -> Value<MultiCell>;
    pub fn trigger(self) -> Part<PopupState>;     // Role::ComboBox, expanded, controls, active_descendant
    pub fn field(self) -> Part<PartState>;        // 検索欄（Space=文字, Home/End=caret, ←/→ は grid 行中のみ）
    pub fn listbox(self) -> Part<PartState>;      // Role::ListBox|Grid, multiselectable(true)
    pub fn row(self, index: usize) -> Part<ListItemState>;   // Role::Option|Row, selected=値
    pub fn option_cell(self, index: usize) -> Part<PartState>; // Role::GridCell（click=toggle）
    pub fn action_cell(self, index: usize) -> Part<PartState>; // Role::GridCell
}
```

- select-all の状態計算（全 enabled 選択→true / 一部→indeterminate / 無→false、フィルタ中は表示中の enabled のみ対象、解除時は disabled の選択を保持）と「開いた瞬間の選択を先頭へ・開いている間は固定」の並べ替えは model 側の helper（`fn order_selected_first(entries, selected_at_open) -> Vec<MultiRow<K>>`）として公開し、MultiSelector と caller が共有する。
- RTL は kit の direction（`basic::direction` と同じ仕組み）を読む。
- 読み上げ（"{count} of {total} selected" 等）は model ではなく component 側（15.6）。
- 未解決: `aria-haspopup="grid"` 相当の setter が kivact に無い（**UNRESOLVED**、26.3 と同じ）。


### 26.5 useTypeahead

Source: `packages/core/src/hooks/useTypeahead.ts`, `useTypeahead.doc.mjs`, `useTypeahead.test.tsx`; matching は `packages/core/src/utils/typeaheadMatch.ts` (useTreeFocus / MultiSelector と共有)。

**判定: Typeahead コンポーネント (16 節) とは無関係。** menu / listbox / tree などの collection に付ける **APG の type-to-select (first-character navigation) hook** である。text input に付けてはいけない (doc の bestPractices: "Use it on a text input" は禁止)。Typeahead / BaseTypeahead はこの hook を import していない。利用箇所: `DropdownMenu/DropdownMenu.tsx`, `DropdownMenu/DropdownMenuSubMenu.tsx`, `ContextMenu/ContextMenu.tsx`, `Selector/Selector.tsx`, `Breadcrumbs/BreadcrumbItem.tsx`, `NavMenu/NavHeadingMenu.tsx`, `TopNav/TopNavMenu.tsx` (doc の relatedComponents: DropdownMenu, Selector, TreeList)。

#### API

```ts
useTypeahead({
  getItemLabels: () => ReadonlyArray<string | null | undefined>, // DOM 順。null/空 = 照合しない slot (index は保持)
  onMatch: (index: number) => void,                              // caller が focus / highlight を動かす
  getCurrentIndex?: () => number,                                // 既定 -1 (= 現在なし)
  resetMs?: number,                                              // 既定 750
  isDisabled?: (index: number) => boolean,                       // skip する index
}) => { onKeyDown(e): boolean /* printable を消費したら true */, reset(): void }
```

hook 自身は focus を動かさない。caller は `if (!typeahead.onKeyDown(e)) handleArrowKeys(e)` の順で呼ぶ (先に typeahead)。

#### 挙動 (tests で固定)

1. 対象 key: `e.key.length === 1` で、ctrl / meta なし、IME composing でない (`isComposing` / `nativeEvent.isComposing`)、かつ Space 以外。**Alt は除外しない** (macOS Option+a → "å" を打てるように。AltGr は ctrlKey を立てるので除外される)。
2. Space: buffer が空なら typeahead ではない (menu の activation に残す)。buffer が非空なら buffer に追加 ("new y" のような複語 label)。ctrl/meta 付き Space、composing Space は常に無視。
3. buffer: 押下ごとに追加し、最後の押下から **750ms** (resetMs) 無操作で空に戻す。unmount / React `<Activity>` で隠れた時も timer を止め buffer を空に。`reset()` で即クリア (close 時など)。
4. 同一文字の連打 (fold 後に同じ) は buffer を伸ばさず、その 1 文字のまま → 同じ頭文字の item を順に巡回 (APG)。"a" → "ap" → "apr" は絞り込み。
5. 検索開始位置: `current = getCurrentIndex()`。current ≥ 0 かつ buffer が 1 文字なら **current の次**から (今の item を再一致させない)。buffer が 2 文字以上なら current を含めて (絞り込み中は今の item に留まれる)。current < 0 (または範囲外) なら先頭から。末尾で wrap。`isDisabled(index)` の item は飛ばす。
6. 照合 (`typeaheadMatch.ts`): label を trim し、prefix 一致。fold = NFD → U+0300–036F の結合記号除去 → 6 種の stroked letter (đ ł ø ħ ŧ ƀ、大文字含む) と Turkish ı を基本字へ → NFC 再合成 → lowercase。**locale に依存しない**。typed 側に記号付き文字 (アクセント、stroked、ı / İ) があれば、まず exact (NFC + Turkish case pairing: I↔ı, İ↔i) で探し、無ければ folded で探す。先頭の結合記号単独 key は無視 (返り値 false)。文字の後に打った結合記号 (Thai 声調、Devanagari anusvara) は buffer に残す。
7. 一致なし: highlight は動かさないが、printable を消費したので `true` を返す。label 配列が空なら `false`。

#### Kivact への対応付け

| Astryx | kivact::interaction の相当 | 差分 |
|---|---|---|
| `useTypeahead` (collection 用 type-to-select) | `ListPolicy { typeahead: true, .. }` — `ListPolicy::listbox()` / `ListPolicy::menu()` preset で有効。`use_list(..)` (list) と `use_menu(..)` (`MenuApi`; level 間で `shared_typeahead` を共有) に組み込み済み。`kivact::ui::list` / `menu` / `select` もこの経由 | **独立した public hook は無い**。buffer は crate 内 `list::typeahead::Typeahead` (`pub(crate)`)、検索は `typeahead_find` (`pub(crate)`) (`crates/kivact-interaction/src/list/typeahead.rs`, `list/input.rs:150-239`) |
| `resetMs` 750ms | `TYPEAHEAD_WINDOW_NS` = **500ms** 固定 | 時間窓が短い・設定不可 |
| fold (アクセント / stroked / Turkish / NFD) | `to_lowercase()` のみ | "é" で "e…" に、"e" で "É…" に一致しない |
| Alt+文字を許可 | Alt 修飾は除外 (`!modifiers.alt`) | macOS の Option 合成文字が typeahead にならない |
| buffer 中の Space を追加 | Space は `Key::Space` で別処理 (typeahead にならない) | 複語 label の 2 語目へ飛べない |
| 1 文字は current の次から / 複数文字は current から / 同一文字連打は巡回 | 同じ (`TypeaheadQuery { needle, from_next }`) | 一致 |
| disabled skip | 同じ (disabled と focus 不可を skip) | 一致 |
| exact 優先 (記号付き typed) | 無し | |
| `getCurrentIndex` | list の focused / active key | 一致 |
| `onMatch(index)` → caller が focus | `ListActivation::Automatic` なら選択 + focus、Manual なら focus のみ (`list/input.rs:225-238`) | caller の自由度は無い (policy で決まる) |

#### 実装方針

1. **RESTYLE する標準部品** (`kivact::ui::menu`, `select`, `list`, `tabs` …) と、`use_list` / `use_menu` 上に作る kit 部品 (soft の DropdownMenu, ContextMenu, Selector の listbox, TreeList など): kivact 内蔵の typeahead をそのまま使う (kit からは差し替えられない)。上表の差分 (500ms、fold なし、Alt 除外、Space 非対応) は**既知の差分**として各節の受け入れ確認に記す。Astryx と同一にしたい場合は kivact-interaction 側の改修 (window の設定化、fold 関数、Alt 許可、buffer 中 Space) を upstream 提案とする — kit 内で回避はしない。
2. **kit が `use_list` を使わずに自前の collection を持つ場合** (例: 閉じた Selector trigger 上の type-to-select、canvas 描画の list) だけ、kit 内部 helper `soft::type_select::TypeSelect` を Astryx 準拠で移植する:
   - `TypeSelect::new(reset_ms: u32 /* 750 */)`、`fn key(&mut self, event: &KeyEvent, labels: &[Option<String>], current: Option<usize>, disabled: impl Fn(usize) -> bool) -> (Handled, Option<usize>)`、`fn reset(&mut self)`。時刻は `kivact::primitive::clock_now()` (list/input.rs と同じ時計)、または `kivact::task::sleep(750ms)` を spawn して buffer を空にする (owner 破棄で cancel → unmount 時の reset を兼ねる)。
   - printable 判定: `Key::Char(c)` で `!c.is_control()`、ctrl / meta なし (**alt は許可**)、`Key::Space` は buffer 非空のときだけ ' ' として追加。IME composing 中の key は `use_text_edit` を通らない collection には来ない前提 (届く場合の判定 API は UNRESOLVED: `KeyEvent` に composing flag があるか確認)。
   - fold: kit は `kivact` 以外に依存できない (basic README と同じ規約) ため Unicode NFD は使えない。lowercase に加え、Latin-1 / Latin Extended-A のアクセント付き文字 → 基本字の静的表 (á→a, ç→c, ñ→n, ő→o …) と stroked 6 種 + ı を手書きで持つ。CJK / Hangul / 結合記号付き文字の扱いは lowercase のみ (Astryx との差分として記録)。exact 優先は「typed に表の対象文字が含まれるならまず未 fold で比較」で再現する。
3. Typeahead (16 節) とは混同しないこと: Typeahead の文字入力は text field への入力であり、type-to-select は付けない。


### 26.6 internal/tokenizer-presentation.ts

Source: `packages/core/src/internal/tokenizer-presentation.ts` (25 行), test `packages/core/src/internal/tokenizer-presentation.test.ts` (29 行)。

#### 何をしているか
- ロジックを持たない **re-export 専用 entry**。`@astryxdesign/core/internal/tokenizer-presentation` として、`Tokenizer` と同一の関数値を `TokenizerWithPresentation` という名前で出し、型 `TokenizerPresentationOptions` / `TokenizerWithPresentationProps` を出す (tokenizer-presentation.ts:19-25)。
- 目的: Tokenizer の sheet presentation (`experimental.presentation: 'popover' | 'bottom-sheet' | 'adaptive'`, Tokenizer.tsx:156-178, 348-359) は実装済みだが UX 未完成 (`component:Tokenizer` DEC-3) のため、**公開 `TokenizerProps` には `experimental` を載せず**、Astryx 自身のパッケージ (Core / Lab; lint `@astryx/no-core-internal-import` で制限) だけがこの entry 経由で使える。公開 Tokenizer は常に popover presentation。
- `TokenizerWithPresentation` は `Tokenizer as <T>(props: TokenizerWithPresentationProps<T>) => JSX.Element` という**型だけのキャスト** (Tokenizer.tsx:1835-1839)。実体は同じ関数で、実装は最初から `experimental` を読んでいる (Tokenizer.tsx:778, 813-824)。`TokenizerWithPresentationProps` は `Omit<TokenizerProps,'ref'>` + `experimental?` + `ref?: Ref<HTMLInputElement | HTMLButtonElement>` (sheet では ref が trigger button に付く)。

#### テストが固定している契約 (tokenizer-presentation.test.ts:10-28)
1. entry の `TokenizerWithPresentation` は `Tokenizer/Tokenizer` の `Tokenizer` と**同一値** (`toBe`)。
2. entry の export 名は `['TokenizerWithPresentation']` **ちょうど 1 つ** (型は実行時に存在しない)。
3. 公開 `Tokenizer` barrel (`Tokenizer/index.ts`) の値 export は `['Tokenizer']` のみで、`TokenizerWithPresentation` も `TokenizerBottomSheet` も含まない。

#### presentation の解決規則 (entry が露出させるもの; 実装は Tokenizer.tsx)
- `presentation` 既定 `'popover'`。`'adaptive'` は `useAdaptivePresentation`: 「primary pointer が coarse」かつ「viewport 幅 < 768px (テーマの md) または 高さ < 500px」で `'bottom-sheet'`、それ以外 `'popover'` (hooks/useAdaptivePresentation.ts:28-62, theme/themeAdaptations.ts:76-93)。SSR / 固定値では media query を読まない。
- latch: sheet が open 中、または close 後 exit 完了で trigger に focus が戻るまで (`isSheetPresented`) は sheet presentation を維持 (回転などで adaptive が変わっても面を引き抜かない)。
- presentation 切替で focus を持つ control が消える場合、新しい control (input ↔ trigger) が focus を引き継ぐ (callback ref の hand-off、Tokenizer.tsx:856-882)。引き継ぎは「外から focus が入った」扱いにしない (`onFocus` を呼ばない)。
- sheet presentation で効かない props (`renderItem`, `renderToken`, `endContent`, `hasClear`, `hasEntriesOnFocus`, `menuWidth`, `tokenOverflowBehavior`≠none, `hasAutoFocus`) は dev で 1 回警告。
- sheet chunk は sheet presentation が有効になった時点で preload、表示は press 時のみ (closed dialog を DOM に置かない)。
- 開いている間に disabled になったら close (exit は走らせ focus を trigger へ)、再 enable で再オープンしない。最初から disabled なら sheet を mount しない。

#### Kivact 実装方針
- Rust には「内部 entry」相当の公開範囲が無い (crate 外へは `pub` か非公開か)。kivact-ui-soft は Astryx の一般利用者向け挙動 (常に popover) を既定に保ちつつ、presentation を**公開メソッドとして実装する**: `soft::tokenizer(..).presentation(TokenizerPresentation::Popover | BottomSheet | Adaptive)`、既定 `Popover`。rustdoc に「experimental: sheet の UX は Astryx 側でも未確定」と明記する (Astryx 互換のために隠す必要はない; kit 利用者が API 変更に追随する前提)。別名型 `TokenizerWithPresentation` は作らない (Rust では 1 つの builder に method が 1 つ増えるだけ)。
- `Adaptive` の判定: `kivact::platform::capabilities().touch` と `kivact::platform::window_size()` (幅 < 768 または 高さ < 500) を reactive に読む。kivact に primary pointer 種別の環境値が無いので touch 能力で近似 (UNRESOLVED: タッチ対応ノート PC では Astryx は popover、近似では sheet になり得る — 必要なら最後の pointer event の `PointerKind::Touch` を併用)。latch は `presented: signal(bool)` を「open 開始で true / close 後に trigger が focus を受けた時 false」で管理。
- trigger の ref: popover 時は input、sheet 時は trigger button に `.node_ref(..)` を付け替える (NodeRef は 1 つ、付け先を presentation で切替)。
- テスト移植 (consumer test): (1) 既定は popover (combobox が存在し dialog trigger が無い)、(2) `BottomSheet` 指定で `Role::Button` + `has_popup dialog` の trigger が出て combobox が無い、(3) `Adaptive` は小さい touch 窓で sheet・大きい窓で popover、(4) sheet open 中に窓サイズを変えても sheet が残り、閉じた後 focus が新しい control に移る。export 名テスト (契約 2, 3) は Rust では意味が無いので移植しない。


### 26.7 useCollator / useLocale（日付・数値フォーマット）

ソース: `D:\w\meta\astryx\packages\core\src\i18n\`（`useLocale.ts` + `useLocale.doc.mjs`、`useCollator.ts` + `useCollator.doc.mjs`、`InternationalizationContext.ts`、`InternationalizationProvider.tsx` + `.doc.mjs`、`resolve.ts`、`useDirection.ts`、`getLocaleDirection.ts`、`useListFormatter.ts`、`useComponentTranslator.ts`、`types.ts`）と、ロケールを消費する日付ヘルパ `utils/plainDate.ts`、`utils/dateParser.ts`、`Calendar/getStandaloneShortWeekdayNames.ts`。カタログは `packages/core/locales/*.json`（en + 29 ロケール: af-ZA ar-SA ca-ES cs-CZ da-DK de-DE el-GR es-ES fi-FI fr-FR he-IL hu-HU it-IT ja-JP ko-KR nl-NL no-NO pl-PL pt-BR pt-PT ro-RO ru-RU sr-SP sv-SE tr-TR uk-UA vi-VN zh-CN zh-TW）。

#### Astryx の仕組み（要点）

- **ロケールの源は 1 つ**: `InternationalizationProvider`（props: `locale: string`（BCP 47）、`messages?`（追加カタログ）、`overrides?`（ロケール別の疎な上書き）、`dir?`（'ltr'｜'rtl'。省略時 `Intl.Locale(locale).getTextInfo().direction`、失敗時 ltr。DOM の `dir` は設定しない））が context `{locale, direction, messages, overrides, translate}` を配る。
- `useLocale()`（`useLocale.ts:75-78`）: context の `locale` を返すだけ。**provider が無ければ `'en'`**。doc の方針: 整形ヘルパ（`plainDateFormat`、`formatInstant`、チャート整形）や `useCollator` に渡すロケールは必ずこれから取る。`navigator.language` やハードコード文字列を代替に使わない。
- `useCollator(options?: Intl.CollatorOptions)`（`useCollator.ts:38-44`）: `new Intl.Collator(ctx.locale, options)` を `[locale, options]` で memo。生の `Intl.Collator` や `localeCompare` を直接使わない方針。core 内の利用は Table のソート（`Table/plugins/sortable/useTableSortableState.tsx:269` で `{numeric: true}`）。**Typeahead/Selector のフィルタは collator を使わず `toLowerCase().includes()`**（`Typeahead/createStaticSource.ts:64-74`, `Selector/Selector.tsx:931`）。
- `useDirection()`: context の direction。`useListFormatter()`: `Intl.ListFormat(locale, {type:'conjunction', style:'narrow'})`。
- 文言解決（`resolve.ts:9-17, 120-153`）: キー検索順 = ①その locale の override → ②親 locale の override（`pt-BR`→`pt`、`Intl.Locale.baseName` で正規化した tag を末尾から削る）→ ③その locale のカタログ → ④親 locale のカタログ → ⑤ 同梱 English（読み込まれたモジュールが登録した分）→ ⑥キー文字列そのもの（dev 警告 1 回）。値がある場合は ICU MessageFormat（`intl-messageformat`、plural/select 可）で整形しキャッシュ。
- 日付整形（`utils/plainDate.ts`）: `plainDateFormat(pd, options, locale='en')` は暦を常に `gregory`、日付フィールドだけなら UTC 真夜中を UTC で整形（ホストのタイムゾーンに依存しない）、`Intl.DateTimeFormat` を (locale, options) でキャッシュ。日付コンポーネントが使う形:

| 定数 / 関数 | Intl options | en の例 | 用途 |
|---|---|---|---|
| `DATE_FORMAT_WITH_WEEKDAY` | weekday long, year numeric, month long, day numeric | "Saturday, March 21, 2026" | 日ボタンの accessible name、range 告知 |
| `DATE_FORMAT_LONG`（`date_long`） | year, month long, day | "March 21, 2026" | DateInput 既定表示 |
| `DATE_FORMAT_SHORT_WITH_YEAR`（`date`） | month short, day, year | "Mar 21, 2026" | `format='date'` |
| `DATE_FORMAT_SHORT_WITH_WEEKDAY`（`date_weekday`） | weekday short, month short, day, year | "Sat, Mar 21, 2026" | `format='date_weekday'` |
| `system_date` | （Intl を通さない） | "2026-03-21" | `format='system_date'` |
| `DATE_FORMAT_MONTH_YEAR` + `plainDateFormatStandaloneMonthYear` | year, month long ＋ 言語 es/fr/it/nl/pl/ru/uk のみ先頭大文字化 | "March 2026"（fr "Mars 2026"） | Calendar ヘッダー、grid 名、シートのタイトル |
| `DATE_FORMAT_MONTH_ONLY` | month long | "January" | 月ホイール（2021-xx-15 で整形） |
| `DATE_FORMAT_WEEKDAY_ONLY` | weekday short（CLDR abbreviated） | "Sun" | タッチ面の曜日行（1970-01-04=日曜から回転） |
| `getStandaloneShortWeekdayNames(locale)` | Intl では出せない CLDR **stand-alone short** を生成表で | "Su Mo Tu We Th Fr Sa"、ja "日 月 火 水 木 金 土"、fr "di lu ma me je ve sa" | Calendar の曜日見出し。exact baseName → language → en |
| `plainDateWeekFieldName(locale)` | `Intl.DisplayNames(locale,{type:'dateTimeField'}).of('weekOfYear')`、無ければ "week" | "week"、de "Woche"、ja "週" | 週番号列見出し（視覚的には空） |
| `isLocaleDayFirst(locale)` | `formatToParts(2000-01-15)` で day が month より前か | en(-US) false、en-GB/de/fr true、ja false（y/M/d 順） | 数字日付タイプ入力の曖昧解消 |

- 数値: 日付コンポーネント内の数字（日番号、年ホイール、週番号）は **JS の数値をそのまま文字列化**（常に ASCII 数字、桁区切りなし）。月/年ラベルと日付名は Intl 経由なので、CLDR が既定で非ラテン数字を使うロケール（例 `ar` 系の一部）では Intl 側だけ数字体系が変わり得る（UNRESOLVED: `ar-SA` の Intl 既定数字は実行環境の CLDR 版に依存。ネイティブではラテン数字に統一してよい）。数値整形は NumberInput（`formatEditableNumber` / `parseNumberInput` with locale）や PowerSearch（`Intl.NumberFormat(locale)`）が使う（それぞれの章）。
- 週の開始: Astryx の Calendar / DateInput は **locale から週開始を推定しない**（`weekStartsOn` 既定 0=日曜）。
- 日付コンポーネントが使う translator キー（`useComponentTranslator`、英語既定値）:
  - Calendar: `@astryx.calendar.previousMonth` "Previous month"、`.nextMonth` "Next month"、`.daySelected` "{date}, selected"、`.dayRangeStart` "{date}, range start"、`.dayRangeEnd` "{date}, range end"、`.dayRangeStartAndEnd` "{date}, range start and range end"、`.dayInRange` "{date}, in range"、`.rangeStartAnnounce` "Start date {date}. Select an end date."、`.rangeCompleteAnnounce` "Selected range: {start} to {end}."、`.rangeClearedAnnounce` "Cleared start date {date}. Select a start date."
  - DateInput（3 面）: `@astryx.dateInput.placeholder` "Select a date"、`.dialogLabel` "Choose date"、`.closeCalendar` "Close calendar"、`.openCalendar` "Open calendar"、`.toggleCalendarClose` "Close calendar"、`.clear` "Clear {label}"、`.invalidDate` "Invalid date"、`.resetPicking` "Reset"、`.savePicking` "Save"、`.doneChoosingMonth` "Done"、`.chooseMonthYear` "{monthYear}, Choose month and year"、`.monthWheel` "Month"、`.yearWheel` "Year"、共用 `@astryx.spinner.loading`。
  - 参考（別担当コンポーネント）: `@astryx.dateRangeInput.presetDateRanges` "Preset date ranges"、`@astryx.dateTimeInput.*`（timePlaceholder "Select a time", timeHint12h "e.g., 2:30 PM", timeHint24h "e.g., 14:30", timeSuffix "{label} time", timeOptionsLabel "{label} options", placeholder, dialogLabel "Choose date and time", pickerMode "Date/time section", dateTab "Date", timeTab "Time", openTimePicker "Open {label}", saveDatePicking "Save date", hourWheel "Hour", minuteWheel "Minute", secondWheel "Second", meridiemWheel "AM/PM", meridiemAM "AM", meridiemPM "PM"）、`@astryx.timeInput.*`（placeholder "Select a time", invalidTime "Invalid time", openPicker "Open {label}", clearLabel "Clear {label}"）。

#### ネイティブ kit に必要なもの

1. **ロケールの源**: `kivact::i18n::locale()`（`Value<Locale>`, reactive。`App::locale` があればそれ、無ければホスト OS のロケール、どちらも無ければ `en-US`）。kit は独自の provider を持たず、これを唯一の源にする（`soft::use_locale()` を作るならこれの薄いラッパ）。Astryx の「provider 無し = 'en'」に対し Kivact は `en-US` 既定だが、英語既定文言と en 表の結果は同じ。部分木だけ変えるのは `kivact::view::provide_locale`。方向は `kivact::view::current_direction()`（Astryx の useDirection 相当）。
2. **文言**: `kivact::i18n::message(key, default)` / `message_with(key, default, args)`（reactive。`{name}` 置換のみ、plural/select なし → 複数形が要る文言はキーを分ける）。キーは `soft.<component>.<name>`（20.6 / 21.6 の表）。Astryx の 29 ロケールの訳は `packages/core/locales/*.json` の `defaultMessage` を Kivact の `Messages` へ変換するツール（xtask）で同梱カタログ化できる（ICU の `{date}` 形式は Kivact の `{date}` と互換。plural を含むキーは日付系には無い）。解決順（override → 親 → カタログ → 親 → en）は App が `Messages::layered` で組む。
3. **日付整形（Intl の代替）**: Kivact は「日付と数値の書式は将来の作業」（`kivact-i18n/src/lib.rs` の doc）で、`Locale::first_day_of_week()` しか持たない。kit は依存を `kivact` のみに保つため、**CLDR から生成した Rust 表**を同梱する（例 `soft::i18n::cldr`、生成は xtask、手編集禁止）。対象は Astryx の 30 ロケール（en + 上記 29、解決は exact tag → language → en。`pt-PT` と `pt`(BR)、`zh-TW` と `zh`(CN) を区別）。1 ロケールあたり必要なデータ:
   - 月名: format-wide（"January"）、stand-alone-wide、abbreviated（"Jan"）。
   - 曜日名: wide（"Saturday"）、abbreviated（"Sat"）、**stand-alone short**（"Sa"; Calendar 見出し。Astryx の `standaloneShortWeekdayNames.generated.ts` の 30 言語分をそのまま移植してよい）。
   - 日付パターン（skeleton → pattern）: `yMMMM`（月年）、`yMMMMd`（date_long）、`yMMMd`（date）、`yMMMEd`（date_weekday）、`yMMMMEEEEd`（a11y 名）。例 ja: "2026年3月"、"2026年3月21日"、"2026年3月21日"、"2026年3月21日(土)"、"2026年3月21日土曜日"。
   - stand-alone 大文字化フラグ（es fr it nl pl ru uk）。
   - "week of year" フィールド名（"week"/"Woche"/"週"…）。
   - day-first 判定（パターン中の d が M より前か）。
   - 提案 API（kit 内部、または公開）:
     ```rust
     pub enum DateStyle { Short, Long, ShortWithWeekday, FullWithWeekday, MonthYear, MonthYearStandalone, MonthOnly, WeekdayAbbrev, Iso }
     pub fn format_date(date: CalendarDate, style: DateStyle, locale: &Locale) -> String;
     pub fn weekday_names(width: WeekdayWidth, first: Weekday, locale: &Locale) -> [String; 7];
     pub fn week_field_name(locale: &Locale) -> String;
     pub fn is_day_first(locale: &Locale) -> bool;
     pub fn parse_date_input(text: &str, locale: &Locale, today: CalendarDate) -> Option<CalendarDate>; // dateParser.ts の移植（英語月名のみ・ASCII 数字のみ）
     ```
     コンポーネントは `memo(move || format_date(d, style, &locale.get()))` で reactive に使う。数字は常にラテン数字。暦は Gregorian のみ。タイムゾーンは関与しない（`CalendarDate` は暦日）。
   - basic は月名を `message("basic.calendar.month_1", "January")` のように文言キーで持つ方式（`crates/kivact-ui-basic/src/calendar/caption.rs`）。soft はロケール表を正とし、App が上書きしたい場合のみ `soft.calendar.month_N` 等のキーを優先する二段構えにしてもよい（任意）。
4. **週の開始**: soft の Calendar/DateInput の既定は Astryx どおり `Weekday::Sunday`（locale 非依存）。ロケール準拠にしたい App は `.week_start(kivact::i18n::locale().get().first_day_of_week())` を渡す（CLDR region 表は Kivact 側に既にある）。
5. **数値整形**: 日付コンポーネント自体は不要（ASCII 数字直書き）。NumberInput 等のために、表に decimal/group 記号（en "." ","、de "," "."、fr "," U+202F、ru "," U+00A0、ja "." "," …）と grouping 3 桁、マイナス記号を含めておくと共用できる（`format_number(f64, &NumberStyle, &Locale)`）。
6. **照合（collation）**: `useCollator` の主用途はテーブルの文字列ソート（`{numeric: true}`）。ICU を持たない kit では近似実装:
   ```rust
   pub struct CollatorOptions { pub numeric: bool, pub sensitivity: Sensitivity /* Base|Accent|Case|Variant, 既定 Variant */ }
   pub struct Collator { locale: Locale, options: CollatorOptions }
   impl Collator { pub fn new(locale: &Locale, options: CollatorOptions) -> Self; pub fn compare(&self, a: &str, b: &str) -> std::cmp::Ordering; }
   pub fn use_collator(options: CollatorOptions) -> Value<Collator>; // locale() の memo
   ```
   比較は多段: (1) 一次キー = 小文字化（`char::to_lowercase`）＋ラテン文字の発音区別記号を落とす表（À-ž の範囲、ß→ss、æ→ae 等）、`numeric` なら連続数字を数値として比較、(2) 一次が同じなら発音区別記号の有無、(3) 大文字小文字（小文字先）、(4) コードポイント。CJK は一次キー = コードポイント順（ja/zh の読み順ソートは対象外: UNRESOLVED/非対応と明記）。sv/fi の å ä ö のような言語固有順は表で上書き可能にしておく。
   フィルタ（Typeahead/Selector/Combobox の絞り込み）は Astryx どおり **単純な小文字化 contains**（collator 不要）。
7. **テスト観点**: en/ja/fr/de/ar-SA で `format_date` の 5 形式、`weekday_names` の short/abbrev と週開始回転、fr の月年先頭大文字化、`is_day_first`（en-US false / en-GB true / de true / ja false）、`parse_date_input`（"3/4/2026" が en-US で 3 月 4 日、en-GB で 4 月 3 日、"2026" は None）、Collator の numeric（"item2" < "item10"）。


