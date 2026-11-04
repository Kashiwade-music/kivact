# kivact-ui-soft 作業指示書 06: TABLE, LIST & CHAT

対象: Astryx（D:\w\meta\astryx, packages/core/src）の List / Item / MetadataList / OverflowList / TreeList / Table（全 table hook）/ Chat（レイアウト・メッセージ・コンポーザー）。Neutral テーマ（packages/themes/neutral）の light / dark の解決値を各所に併記する（`値 / 値` は light / dark）。トークン名は packages/core/src/theme/tokens.stylex.ts の Astryx 名。

本書の判断の前提（Kivact 側）:
- 標準 `kivact::ui::list` は **選択リスト（listbox）**、`kivact::ui::data_table` は **grid（セルか行に focus）+ filter/sort/page/select/resize のモデル** である（crates/kivact-ui/src/list.rs, data_table/*.rs, data_column.rs）。
- strategy（docs/for-kivact-developer/strategy.md）§5: list・table・tree のキー操作と semantics は中核。§6 rule 5「共通の意味は中核へ」: 2 つ以上の公式 kit が同じ意味の部品を持つなら中核の標準部品か interaction にする。kit で複製しない。→ 本書では該当する不足を **CORE REQUEST** と明記し、kit 内コピーを設計しない。暫定実装が必要な場合も、標準部品の形の API にして後で restyle に置き換えられるようにする。

## 目次

- [グループ一覧](#グループ一覧)
- [Kivact 比較と CORE REQUEST の総覧](#kivact-比較と-core-request-の総覧)
- [1. List (+ ListItem, useListFocus)](#1-list--listitem-uselistfocus)
- [2. Item](#2-item)
- [3. MetadataList (+ MetadataListItem)](#3-metadatalist--metadatalistitem)
- [4. OverflowList](#4-overflowlist)
- [5. TreeList (+ useTreeFocus)](#5-treelist--usetreefocus)
- [6. Table (+ TableBody, TableCell, TableFooter, TableHeader, TableHeaderCell, TableRow, TableSelectionToolbar, useGridFocus)](#6-table--tablebody-tablecell-tablefooter-tableheader-tableheadercell-tablerow-tableselectiontoolbar-usegridfocus)
- [7. Table の機能（table hooks）](#7-table-の機能table-hooks) — 7.1 sortable / 7.2 filtering / 7.3 pagination / 7.4 selection / 7.5 columnResize / 7.6 columnSettings / 7.7 stickyColumns / 7.8 rowExpansion / 7.9 groupedRows / 7.10 tree / 7.11 rowIndex / 7.12 rowStatus / 7.13 対応表
- [8. Chat: レイアウトとメッセージ](#8-chat-レイアウトとメッセージ) — 8.1 ChatLayout (+ ScrollButton) / 8.2 ChatMessageList / 8.3 ChatMessage (+ Bubble, Metadata) / 8.4 ChatSystemMessage / 8.5 ChatToolCalls / 8.6 ChatTokenizedText / 8.7 useStreamingText
- [9. Chat: コンポーザー](#9-chat-コンポーザー) — 9.1 ChatComposer / 9.2 ChatComposerInput / 9.3 ChatComposerDrawer / 9.4 ChatComposerTokenElement / 9.5 ChatDictationButton / 9.6 ChatSendButton
- [UNRESOLVED 一覧](#unresolved-一覧)

## グループ一覧

| Astryx component (and parts) | docs URL | source dir | kivact strategy | closest kivact-ui-basic analog |
|---|---|---|---|---|
| List (+ ListItem) | https://astryx.atmeta.com/components/List | packages/core/src/List | NEW kit component（静的 role=list。標準 `kivact::ui::list` は listbox なので別物）。静的リストの意味は CORE REQUEST。listbox 系の見た目は `kivact::ui::list` の RESTYLE | item.rs（`item_group` + `item`）, typography.rs（`bullet_list`） |
| Item | https://astryx.atmeta.com/components/Item | packages/core/src/Item | NEW kit component（menu / select / list / command_palette の item recipe もこの寸法で RESTYLE） | item.rs |
| MetadataList (+ MetadataListItem) | https://astryx.atmeta.com/components/MetadataList | packages/core/src/MetadataList | NEW kit component | （無し）field.rs / item.rs が近い |
| OverflowList | https://astryx.atmeta.com/components/OverflowList | packages/core/src/OverflowList | NEW kit component（測定は node_ref().rect()。不可視測定 node は CORE REQUEST） | breadcrumb.rs（ellipsis）, pagination.rs |
| TreeList (+ useTreeFocus) | https://astryx.atmeta.com/components/TreeList | packages/core/src/TreeList, hooks/useTreeFocus.ts | CORE REQUEST: 標準 `kivact::ui::tree`（use_tree 上）→ soft は RESTYLE。暫定は use_tree 上の kit 実装 | （無し）。kivact-ui-game `tree`, kivact-ui-creative `outliner` |
| Table (+ TableBody, TableCell, TableFooter, TableHeader, TableHeaderCell, TableRow) | https://astryx.atmeta.com/components/Table | packages/core/src/Table | data-driven: RESTYLE `kivact::ui::data_table` + 不足は CORE REQUEST。children モード（静的表）は NEW kit component | table.rs（静的 shadcn table） |
| TableSelectionToolbar | https://astryx.atmeta.com/components/TableSelectionToolbar | packages/core/src/Table | kit（data_table の `.toolbar(ctx)` slot に置く） | button_group.rs |
| useTableSortable / useTableSortableState | https://astryx.atmeta.com/components/useTableSortable | packages/core/src/Table/plugins/sortable | RESTYLE（sort recipe）+ CORE REQUEST（複数列ソート等） | table.rs |
| useTableFiltering / useTableFilterState | https://astryx.atmeta.com/components/useTableFiltering | …/plugins/filtering | CORE REQUEST（列ごとの filter state と UI） | — |
| useTablePagination | https://astryx.atmeta.com/components/useTablePagination | …/plugins/pagination | kit（footer slot で soft pager）+ CORE REQUEST（page size, hasMore） | pagination.rs |
| useTableSelection / useTableSelectionState | https://astryx.atmeta.com/components/useTableSelection | …/plugins/selection | RESTYLE（check / row recipe）+ CORE REQUEST（行ごとの選択可否） | — |
| useTableColumnResize | https://astryx.atmeta.com/components/useTableColumnResize | …/plugins/columnResize | RESTYLE + CORE REQUEST（Shift 大ステップ、Home/End、列ごとの可否） | — |
| useTableColumnSettings | https://astryx.atmeta.com/components/useTableColumnSettings | …/plugins/columnSettings | 表示切替は既存。列順・列 id は CORE REQUEST | — |
| useTableStickyColumns | https://astryx.atmeta.com/components/useTableStickyColumns | …/plugins/stickyColumns | CORE REQUEST（列 pin）、影は kit | — |
| useTableRowExpansion | https://astryx.atmeta.com/components/useTableRowExpansion | …/plugins/rowExpansion | CORE REQUEST（展開 state・chevron 列） | disclosure.rs |
| useTableGroupedRows | https://astryx.atmeta.com/components/useTableGroupedRows | …/plugins/groupedRows | CORE REQUEST | — |
| useTableTreeData / useTableTreeState | https://astryx.atmeta.com/components/useTableTreeData | …/plugins/tree | CORE REQUEST（tree rows） | — |
| useTableRowIndex | https://astryx.atmeta.com/components/useTableRowIndex | …/plugins/rowIndex | CORE REQUEST（小: 表示順 index）+ kit | — |
| useTableRowStatus | https://astryx.atmeta.com/components/useTableRowStatus | …/plugins/rowStatus | CORE REQUEST（状態列）+ RESTYLE | badge.rs / marker.rs |
| useGridFocus / useListFocus | https://astryx.atmeta.com/components/useGridFocus , …/useListFocus | packages/core/src/hooks | covered by `kivact::interaction`（use_table の TableFocus::Cells / roving・use_list）。新設しない | — |
| ChatLayout (+ ChatLayoutScrollButton) | https://astryx.atmeta.com/components/ChatLayout | packages/core/src/Chat | NEW kit component + CORE REQUEST（stick-to-bottom 追従 `use_follow_end`） | message_scroller.rs |
| ChatMessageList | https://astryx.atmeta.com/components/ChatMessageList | packages/core/src/Chat | NEW kit component + CORE REQUEST（Role::Log / busy） | message_scroller.rs, chat.rs（`message_group`） |
| ChatMessage (+ ChatMessageBubble, ChatMessageMetadata) | https://astryx.atmeta.com/components/ChatMessage | packages/core/src/Chat | NEW kit component（メッセージ semantics は CORE REQUEST 候補） | chat.rs（`message`, `bubble`, `bubble_group`） |
| ChatSystemMessage | https://astryx.atmeta.com/components/ChatSystemMessage | packages/core/src/Chat | NEW kit component | primitives.rs（`separator`）, typography.rs（`muted`） |
| ChatToolCalls | https://astryx.atmeta.com/components/ChatToolCalls | packages/core/src/Chat | NEW kit component（開閉は use_disclosure） | trace.rs, disclosure.rs |
| ChatTokenizedText | https://astryx.atmeta.com/components/ChatTokenizedText | packages/core/src/Chat | NEW kit component + CORE REQUEST（rich text の inline box） | badge.rs |
| useStreamingText | https://astryx.atmeta.com/components/useStreamingText | packages/core/src/hooks | kit hook | — |
| ChatComposer | https://astryx.atmeta.com/components/ChatComposer | packages/core/src/Chat | NEW kit component | input_group.rs（`input_group_textarea`） |
| ChatComposerInput | https://astryx.atmeta.com/components/ChatComposerInput | packages/core/src/Chat | RESTYLE `kivact::ui::text_area` + kit wrapper。行内 token は CORE REQUEST（fallback: chip 行） | input_group.rs, combobox.rs |
| ChatComposerDrawer | https://astryx.atmeta.com/components/ChatComposerDrawer | packages/core/src/Chat | NEW kit component（use_disclosure） | disclosure.rs, attachment.rs |
| ChatComposerTokenElement | https://astryx.atmeta.com/components/ChatComposerTokenElement | packages/core/src/Chat | NEW kit component（chip fallback） | badge.rs |
| ChatDictationButton | https://astryx.atmeta.com/components/ChatDictationButton | packages/core/src/Chat | NEW kit component（音声と認識はアプリ） | button.rs |
| ChatSendButton | https://astryx.atmeta.com/components/ChatSendButton | packages/core/src/Chat | RESTYLE `kivact::ui::button` + kit wrapper | button.rs |

## Kivact 比較と CORE REQUEST の総覧

### Table vs `kivact::ui::data_table` / kivact-ui-basic `table.rs`

| Astryx table 機能 | data_table の現状 | 判定 |
|---|---|---|
| 列 title / text / custom cell (renderCell) | `DataColumn::new`, `.cell` | 既存（RESTYLE） |
| 単一列ソート、header 押下 | `.sort`, `DataColumn::compare/sortable` | 既存（RESTYLE） |
| 複数列ソート・未ソート循環・sort メニュー | 無し | CORE REQUEST |
| 列 filter（型付き値・popover/inline UI） | 列 filter 関数 + 全体 1 文字列のみ | CORE REQUEST |
| ページング（Prev/Next） | `.page` | 既存（見た目は kit で pager） |
| page 番号 UI・page size・hasMore | 無し | page size / hasMore は CORE REQUEST、見た目は kit |
| checkbox 選択・select-all（tri-state） | `.selection` | 既存（RESTYLE） |
| 行ごとの選択不可 / disabled | 無し | CORE REQUEST |
| 列リサイズ（drag / key / AT） | `.resizable` | 既存（RESTYLE）+ Shift 大ステップ・Home/End・列ごとの可否は CORE REQUEST |
| 列表示切替 | `DataColumn::visible` | 既存 |
| 列順変更・安定列 id | 無し | CORE REQUEST |
| sticky 列 + スクロール影 | 無し | CORE REQUEST（影は kit） |
| 行展開（chevron 列, aria-expanded） | `.row_details`（常時 mount、表示は caller） | CORE REQUEST |
| グループ行 | 無し | CORE REQUEST |
| tree rows | 無し（use_tree は別系統） | CORE REQUEST |
| 行番号列 | 無し | CORE REQUEST（小） |
| 行状態列（dot / icon + tooltip） | `row_color`（色帯のみ） | CORE REQUEST |
| density / dividers / stripe / hover / verticalAlign | `row_height` のみ、Variant で見た目 | density と行 index は CORE REQUEST、他は kit Variant |
| 比率幅 + 下限、align start/center/end | 固定幅、numeric(end) のみ | CORE REQUEST |
| wrap / truncate（省略時のみ tooltip） | 常に 1 行 | CORE REQUEST |
| 空状態 slot | 固定文言 `data_table.empty` | CORE REQUEST |
| static table semantics（grid でない、Tab が行内 control） | 常に grid（TableFocus::Cells / Rows） | CORE REQUEST（`TableFocus::None`） |
| header / 行の右クリックメニュー | 無し | CORE REQUEST |
| aria-rowindex / rowcount | 無し | CORE REQUEST |
| row glide（展開時の後続行の移動） | 無し | CORE REQUEST（keyed list の animated layout） |
| 行 drag 並べ替え、行 action 列、cell_controls | 有り（Astryx に無い） | soft でも recipe で描く |
| children モード（静的表） | basic `table.rs` が kit で持つ | NEW kit component（`soft::table` 系）。2 kit 目なので静的表の意味も CORE REQUEST 候補（優先度低） |

### List vs `kivact::ui::list`

- Astryx List / ListItem は **静的な role=list**（行ごとに独立した button / anchor、density・dividers・edge・marker・header）。`kivact::ui::list` は **listbox（option、矢印移動、単一/複数選択、typeahead）**。意味が違うので Astryx List を `kivact::ui::list` の restyle にしてはいけない。
- `kivact::ui::list` は soft テーマで Item の寸法（radius 10、hover overlay、selected accent-muted、label 14/20、description 12/20）に RESTYLE する（Astryx の listbox 系 = Selector / CheckboxList / RadioList の見た目）。
- 静的リストの意味は basic（`item_group` + `item`, `bullet_list`）と soft の 2 kit が持つ → **CORE REQUEST**: 標準の静的 item list（Role::List / ListItem, marker, dividers, density）。
- swipe actions（touch）は現状 soft のみ → kit 内（pointer kind の公開が無ければ CORE REQUEST）。

### Chat vs kivact-ui-basic `chat.rs` / `message_scroller.rs`（と kivact-ui-pop `messaging.rs`）

| Astryx | basic / pop | 判定 |
|---|---|---|
| ChatMessageBubble（filled / ghost, group 角, name/metadata 揃え, width） | basic `bubble`（7 variant, align, reactions, on_press）, pop `message` | soft は NEW kit（見た目が違う）。意味は共通 → CORE REQUEST 候補（メッセージ semantics） |
| ChatMessage（sender 配置, avatar, name, metadata, article） | basic `message`（align, avatar, header, footer）, pop `message`（mine, initials, timestamp） | 同上。`Role::Article` は CORE REQUEST（小） |
| ChatMessageList（role=log, aria-live polite, aria-busy, 下寄せ, 過去ログ読込） | basic `message_group` + `message_scroller` の column | CORE REQUEST: `Role::Log` + busy + 新着告知（3 kit 共通の意味） |
| ChatLayout の追従スクロール（lock/unlock, 10px / 100px 閾値, spring, 新着検知） | basic `message_scroller`（1.5px で follow、Scroll to end） | **CORE REQUEST**: `kivact::interaction::use_follow_end`（2 kit で同じ意味）。basic / soft はその上の見た目 |
| ChatSystemMessage, ChatToolCalls, ChatTokenizedText, Composer 一式 | 無し（basic `trace` が ToolCalls に近い） | kit。行内 token のみ CORE REQUEST（テキスト編集の拡張口） |

### CORE REQUEST 一覧（本グループ由来、優先度順の目安）

1. `kivact::ui::tree` 標準部品（use_tree 上、TreeRecipes、ガイド線用の `is_last` / 祖先 `is_last`）— game `tree` と creative `outliner` で既に重複（5 章）。
2. `kivact::interaction::use_follow_end`（stick-to-bottom 追従）— basic message_scroller と soft ChatLayout（8.1）。
3. data_table: `TableFocus::None`（静的 table）、density、行 index、比率幅 + 下限 + align、wrap/truncate、empty slot、右クリックメニュー、aria-rowindex（6 章）。
4. data_table: 複数列ソート、列 filter state/UI、行展開、グループ行、tree rows、sticky 列、行状態、列順、行ごとの選択可否、page size / hasMore、resize の拡張（7 章）。
5. keyed list の animated layout（項目の位置変化の補間と exit 保持）— TreeList の展開 glide、Table の row glide（5 / 6 / 7.8 章）。
6. `Role::Log`、busy 状態、`Role::Article`（8.2 / 8.3）。
7. rich text / text edit の inline box（行内 atomic token）と text_area の行数範囲 auto-grow（8.6 / 9.2）。
8. 静的 item list の標準部品（1 章）、「レイアウトするが描画・AT・focus から除外する node」（4 章）、pointer kind（touch 判定）（2 章）。

---

## 1. List (+ ListItem, useListFocus)

docs: https://astryx.atmeta.com/components/List ・ source: `packages/core/src/List/` (List.tsx, ListItem.tsx, ListContext.tsx, List.doc.mjs, ListItem.doc.mjs, List.test.tsx, ListMarkers.a11y.chromium.spec.ts) ・ stories: `apps/storybook/stories/List.stories.tsx`

### 1. 概要

- 縦に並ぶ「関連した項目の集まり」を表示する **静的なコンテンツリスト**（`<ul role="list">` / `<ol role="list">`）。選択リスト（listbox）ではない。行の見た目と操作は `Item` に委譲し（ListItem = `Item as="li"`）、List は density / dividers / edge / marker / header を文脈（`ListContext`）で配る。
- 使う: 設定項目の一覧、連絡先、手順（番号付き）、箇条書き、Card/Section/Dialog 内の行一覧。行は `onClick`（不可視 button）、`href`（不可視 anchor）、`interactiveRef`（中の checkbox 等へ委譲）で対話的にできる。
- 使わない（bestPractices, List.doc.mjs）: 単一項目や無関係な内容のレイアウト。対話的な行の中に別の対話要素を入れる（クリック対象の入れ子）。クリック可能な行と不可の行を区別なく混在させる。ユーザーが「選ぶ」リストは listbox 系（RadioList / CheckboxList / Selector）であり List ではない。
- 推奨: `header` で名前を付ける（aria-labelledby）。start/end スロットで icon・avatar・badge。見出しに文字頭を揃えたいときは `edge`（静的行 = `flush`、Card 等の中の対話行 = `outset` / `bleed`）。

### 2. Anatomy

```
List (header あり時のみ外側 div: flex column)
├─ header div#id  (margin-bottom 8px)           … aria-labelledby の参照先
└─ ul|ol role="list"  [.astryx-list]            … flex column, gap 2px (dividers 時 0)
   └─ ListItem = Item as="li" [.astryx-list-item]
      ├─ marker (listStyle≠none): dot / circle / square / 番号 / task checkbox
      ├─ startContent span
      ├─ content: button | a | span (flex column)
      │   ├─ label span
      │   └─ description span#descId
      └─ endContent span (margin-inline-start:auto)
```
- `listStyle='decimal'` は `<ol>`、それ以外は `<ul>`。どちらにも明示的に `role="list"`（Safari/VoiceOver が list-style:none で list 意味を落とすため。List.tsx:269-274）。
- 番号付き（decimal）では ol が `display:grid; grid-template-columns: max-content minmax(0,1fr); column-gap: 8px`（List.tsx:154-161）、各 li が `grid-template-columns: subgrid; grid-column: 1 / -1`（ListItem.tsx:158-165）で全行共通の番号列を持つ（桁数が変わっても本文の開始位置が揃う）。
- `start≠1` の decimal は `<ol start>` を出し、CSS counter を `start-1` から開始（List.tsx:185-189, 268）。

### 3. API

| Astryx prop | 型 / 既定 | 意味 | Kivact 提案 |
|---|---|---|---|
| children | ReactNode (ListItem) | 行 | `soft::list((list_item(..), ..))` の子 view、または `soft::list_items(items: impl IntoValue<Vec<T>>, key, row: impl Fn(Value<T>) -> ListItem)` |
| density | `'compact'｜'balanced'｜'spacious'` / balanced | 行の上下 padding | `.density(ListDensity::{Compact, Balanced, Spacious})` |
| hasDividers | bool / false | 行間の罫線 | `.dividers(bool)` |
| edge | `'padded'｜'flush'｜'outset'｜'bleed'` / padded | 行の文字・背景・罫線と容器端の関係 | `.edge(ListEdge::{Padded, Flush, Outset, Bleed})` |
| edgeCompensation | `'inline'` (deprecated) | = edge outset | **削除**（互換を持たない方針） |
| header | ReactNode | 上の見出し、aria-labelledby | `.header(|| view)`。header の node を `labelled_by` で list に結ぶ |
| listStyle | `'none'｜'disc'｜'decimal'｜'circle'` / none | marker | `.marker(ListMarker::{None, Disc, Circle, Decimal})`（square / lower-alpha / lower-roman は Markdown 内部用。kit では `Square, LowerAlpha, LowerRoman` も enum に入れてよい） |
| start | number / 1 | 番号の開始 | `.start(u32)` |
| xstyle / className / style | — | 外部スタイル | NodeBuilder の `.style(..)` で代替。className は **drop** |
| data-testid | string | — | `.test_id(..)` |

ListItem:

| prop | 型 / 既定 | Kivact 提案 |
|---|---|---|
| label | ReactNode（string なら 1 行 ellipsis） | `list_item(label: impl IntoText)`; リッチ内容は `.label_view(|| v)` |
| description | ReactNode | `.description(impl IntoText)` / `.description_view(|| v)` |
| startContent / endContent | ReactNode | `.start(|| v)` / `.end(|| v)`（kivact-ui-basic item の `media` / `actions` 相当） |
| align | `'center'｜'start'｜'first-line'` / center | `.align(ItemAlign::{Center, Start, FirstLine})` |
| onClick | handler | `.on_press(|| ..)` |
| href / target / rel | — | **native では無意味**。`.on_press` に置換（アプリが open/navigate を呼ぶ）。target/rel は drop |
| interactiveRef | ref | `.delegate_to(node_ref)`: 行面の press を中の control へ転送し、行自体は tab stop を持たない |
| isDisabled | bool | `.disabled(impl IntoValue<bool>)` |
| isSelected | bool | `.selected(impl IntoValue<bool>)`（静的リストでは aria-current 相当の状態） |
| swipeActions / swipeBehavior | → Item | `.swipe_actions(SwipeActions{leading, trailing})` / `.swipe_behavior(SwipeBehavior::{Reveal, Commit})`（Item 節参照） |

### 4. Visual spec

List（root）: `display:flex; flex-direction:column; gap: --spacing-0-5 = 2px`（List.tsx:139-151）。`hasDividers` で gap 0（174-176）。`overflow-inline: clip`（swipe で横にはみ出す行を切る。hidden ではない＝スクロール容器を作らない）。header は `margin-bottom: --spacing-2 = 8px`（180-182）。

行（Item のスタイルを継承、詳細は 2. Item）: density 別高さ（1 行 label, description なし）
| density | padding-block | padding-inline (`--_item-inset-inline`) | 1 行の高さ | label+description |
|---|---|---|---|---|
| compact | 4px | 8px | 28px | 48px |
| balanced | 8px | 8px | 36px | 56px |
| spacious | 12px | 12px | 44px | 64px |

（label = `--text-body` 14px/20px、description = `--text-supporting` 12px/20px）

dividers（ListItem.tsx:201-212）: 各行 `border-block-end: 1px solid --color-border`（light `#00000014` / dark `#FFFFFF1A`）、`:last-child` は 0。dividers 時は行の radius 0（embeddedStyles.noRadius）。

edge（ListItem.tsx:177-200、List.tsx:168-173）:
- `padded`: 既定。行は内側 inset 8px（spacious 12px）を持ち、角丸 `--radius-element` = 10px の hover/selected 背景。
- `flush`: `--_item-inset-inline: 0` → 左右 padding 0。文字は見出しと同じ線、背景も内容幅で止まる。
- `outset`: List が容器 padding 分だけ左右に負 margin で広がり（`margin-inline: calc(-1 * var(--container-padding-inline-*))`, width 100%+両 padding）、各行は `margin-inline: max(容器padding - inset, 0)`。結果: 文字は見出しの線、背景は inset 分だけ容器 padding に食い込む。
- `bleed`: List は同様に容器端まで広がり、行は `padding-inline: max(容器padding, inset)`、radius 0。背景・罫線が容器端まで届く（Card 内の Table 行と同じ）。
- 容器 padding は Card/Section/Dialog/BottomSheet が公開する `--container-padding-inline-start/end`。Neutral の Card / Section padding は `--spacing-3` = 12px（neutralTheme.ts components.card/section）。容器が無い場合は padded と同じ見た目。

marker（ListItem.tsx:220-314）:
- 容器 `width: --spacing-4 = 16px; align-self: baseline; margin-top: calc((1em*1.4286 - 6px)/2)` = 7px（14px 文字時）。
- disc: 6×6 円、塗り `--color-text-primary`（`#000000` / `#ffffff`）。circle: 6×6 円、1px 枠 text-primary、塗りなし。square: 6×6 正方形 塗り text-primary。
- 番号: `font-size 14px; line-height 20px; font-variant-numeric: tabular-nums; min-width 16px; white-space: nowrap; justify-content: flex-end`（右揃え）、文字色 text-primary、内容 `counter(n) "."`（lower-alpha / lower-roman も）。番号が行の inline inset より広いとき、`margin-inline-start: calc(-1 * inset)` で inset 分だけ行の内側余白にぶら下がり、それより広いと共有列を広げる。
- task（Markdown の `- [ ]`）: 読み取り専用 CheckboxInput size sm（20×20px）、`margin-top: (20-20)/2 = 0`、align-self flex-start。
- marker と start content / label の間隔は Item の gap 8px（decimal は列 gap 8px）。
- RTL: すべて logical（margin-inline / padding-inline）なので自動で鏡像。番号の右揃えは inline-end 側揃え。

```
balanced, padded, dividers, decimal:
 ┌──────────────────────────────────────────┐
 │  1.  [ic] Notifications            [>]   │ 36px  (8px inset, gap 8)
 ├──────────────────────────────────────────┤ 1px --color-border
 │  2.  [ic] Privacy                         │
 │           Control your data (12px secondary)
 └──────────────────────────────────────────┘
```

状態（行）: hover / pressed / focus-visible / selected / disabled は Item と同じ（2.4 参照）。

### 5. Motion

- List 自体のアニメーションは無い。
- 行の背景色 transition: `background-color --duration-fast-min (95ms) --ease-standard cubic-bezier(0.24,1,0.4,1)`（Item.tsx `interactive`）。
- touch press の flash / fade（`interactionOverlayStyles`、Item 節参照）、swipe（Item 節）。
- reduced motion: 背景色 transition はそのまま（短い色変化）。press fade は最大 200ms 以下、swipe の settle は 0ms。

### 6. Behaviour & a11y

- role: `list`、子 `listitem`（`<li>`）。header があれば `aria-labelledby=header id`（header が無いときは書かない：利用者の aria-labelledby を消さないため, List.tsx:264-267）。
- 行の選択状態: `listitem` では aria-selected が不正なので `aria-current="true"`（Item.tsx の `allowsAriaSelected` 判定）。
- キーボード: List 自体は roving を持たない。対話行は各行の不可視 button / anchor がそれぞれ Tab stop（Tab で順に移動、Enter/Space で押下）。`interactiveRef` 行は中の control が唯一の tab stop。
- 番号付きの ListMarkers a11y spec: マーカーは CSS 生成内容で読み上げ対象外（"list, N items" は role で確保）。
- 既定文言: なし。

#### useListFocus（`packages/core/src/hooks/useListFocus.ts`）

List は使わないが、グループの一覧に含まれるので記す。線形リストの roving focus 汎用 hook（Menu, Toolbar, TabList, ButtonGroup, ContextMenu, TreeList の一部などが使用）。
- options: `itemSelector`（既定 `'[role="menuitem"]'`）、`boundarySelector`（入れ子リストの区切り）、`wrap`（既定 true）、`onEscape`（与えると Escape を consume）、`orientation` `'horizontal'｜'vertical'｜'both'`（既定 vertical; horizontal は container の computed direction で RTL を自動判定し左右を反転）、`hasHomeEnd`（既定 true）、`hasPaging`（既定 false: PageDown = 表示中の最後の有効項目、再押下で 1 viewport 先。非 wrap。スクロールしないリストでは End/Home と同じ）、`hasRovingTabIndex`（既定 false: true で tabindex 0/-1 を管理）、`hasCaretGuard`（入れ子の text input の caret が端でない／選択があるとき矢印を奪わない、contenteditable からは常に奪わない）。
- 戻り値: `listRef, handleKeyDown, handleFocus, focusItem(i), focusFirst(), focusLast(), ownsEvent(e), getItems()`。
- Kivact 対応: `kivact::interaction` の roving（crates/kivact-interaction/src/roving.rs）と `use_list`（policy: wrap, orientation, typeahead, page）が同じ意味を持つ。soft kit は **新設しない**（中核の部品をそのまま使う）。caret guard 相当は `use_text_edit` 側との協調が必要で、不足していれば CORE REQUEST（roving が入れ子 text edit の caret 位置を尊重するオプション）。

### 7. Kivact 実装方針

- **NEW kit component**（`soft::list` / `soft::list_item`）。理由: 標準 `kivact::ui::list` は `Role::ListBox` + `Role::Option` の **選択リスト**（use_list, typeahead, 単一/複数選択）であり、Astryx List（静的 `role=list`、行ごとに独立した button/anchor）とは意味が違う。Astryx List を `kivact::ui::list` の restyle にすると、静的行が option になり矢印キー移動・選択が付いてしまい誤り。
- 標準 `kivact::ui::list` の restyle（ListRecipes{root, item, description}）は soft テーマとして **別途必須**（Astryx の listbox 系 = Selector の popup や CheckboxList/RadioList の見た目を、Item の見た目: radius 10px, hover overlay, selected `--color-accent-muted` で描く）。
- 静的リストの意味（`Role::List` + `Role::ListItem`、header による labelled_by、行ごとの press）は Astryx 以外の kit にもある（kivact-ui-basic `item_group` + `item`、typography の `bullet_list`）。2 つ以上の公式 kit が同じ意味を持つので strategy §6 rule 5 に従い **CORE REQUEST**: 「非選択の静的リスト（Role::List/ListItem、marker 種類、dividers、density）」を `kivact::ui` 標準部品（例 `kivact::ui::item_list`）として中核へ。中核に入るまでの暫定として soft kit 内に実装する場合も、API は上記の形にして後で restyle に置き換えられるようにする。
- 行は 2. Item の `soft::item` を `Role::ListItem` で使う（kivact-ui-basic `crates/kivact-ui-basic/src/item.rs` が最も近い参考）。
- edge `outset`/`bleed`: 容器の padding を読む仕組み（CSS 変数継承）が native に無い。提案: soft の card/section/dialog が自分の inline padding を `provide_context(ContainerPadding{start, end})` で配り、list がそれを `use_context` で読んで負 margin を計算する。容器が無い時は 0（= padded と同じ見た目）。
- 番号列の subgrid: native では list が全行の番号文字列幅の最大値を測る必要がある。簡易法: 番号は tabular figures なので `max_digits × digit_advance + "."` を text 計測し、全行の marker 幅に同じ値を与える（min 16px）。
- `overflow-inline: clip`: 横方向だけ clip（scroll 容器にしない）。Kivact の `Overflow::Hidden` が縦スクロールと干渉しないか確認。干渉する場合は swipe 中の行だけを clip する。
- task marker は `kivact::ui::checkbox`（read-only）を 20px で restyle 済みのものを置く。

### 8. 受け入れ確認

- [ ] E2E: `Role::List` が名前（header テキスト）を持ち、子が `Role::ListItem` で件数が一致する。
- [ ] E2E: on_press 付き行は Tab で到達でき Enter/Space で press が 1 回呼ばれる。disabled 行は到達不可・press されない。
- [ ] E2E: selected 行が current/selected 状態を AT に出す。
- [ ] 目視: balanced 1 行 36px、compact 28px、spacious 44px（inline 12px）。行間 2px、dividers 時 0 で 1px 罫線、最後の行に罫線なし。
- [ ] 目視: decimal で 1..10 の番号が右揃え・本文開始位置が全行同じ。disc/circle の 6px マークが 1 行目中央（top 7px）。
- [ ] 目視: Card 内 `outset` で文字が Card 見出しの線、hover 背景が左右 8px（12-8 ではなく inset 分）食い込む。`bleed` で背景が Card 端まで・角丸なし。
- [ ] light/dark 両方で docsite https://astryx.atmeta.com/components/List と比較（罫線 `#00000014` / `#FFFFFF1A`、選択 `#f1f1f1` / `#262626`）。

---

## 2. Item

docs: https://astryx.atmeta.com/components/Item ・ source: `packages/core/src/Item/` (Item.tsx, ItemSwipeLayer.tsx, useSwipeAction.ts, ItemDescriptionContext.tsx, ItemMarkerColumnsContext.tsx, Item.doc.mjs, Item.test.tsx, Item.swipeLayer.test.tsx, ItemMarkerColumns.a11y.chromium.spec.ts) ・ stories: `apps/storybook/stories/Item.stories.tsx`

### 1. 概要

- 「start content + label + description + end content」の汎用行プリミティブ。ListItem、Menu の項目、Selector の option、連絡先・通知・ファイル行、TreeList の行など多くの部品の土台。
- 推奨（Item.doc.mjs bestPractices）: 名前付きスロットで 8 割をまかなう。density は menu/密な一覧 = compact、通常 = balanced、ゆったり = spacious。可変長には `labelLines` / `descriptionLines`。折り返す label の横に小さなマーク（status dot, radio, timestamp）を置くなら `align="first-line"`、背の高いスロットの上端揃えは `align="start"`。swipe 操作には必ず行内の別経路（menu / endContent のボタン）を用意。
- 避ける: 対話的 Item の中に button/link を入れる、ビュー間ナビゲーションに使う、既読/未読など受信箱固有の振る舞いを直接足す（薄い wrapper を作る）。

### 2. Anatomy

```
root (as: div|li|span|任意)  flex row, align-items:center, gap 8px, padding-inline inset, radius 10px
├─ marker?                       (ListItem が渡す)
├─ [markerContent span]          番号付き時のみ: start/label/end をひとまとめにする flex
├─ startContent span             flex: 0 0 auto
├─ content                       flex column, flex:1, min-width:0
│    = <button type=button>   (onClick)
│    | <a href>               (href)
│    | <span>                 (静的 / role 指定 / interactiveRef)
│    ├─ label span            14px/20px text-primary
│    └─ description span#id   12px/20px text-secondary
├─ endContent span               flex: 0 0 auto, margin-inline-start:auto
└─ swipe panels (leading / trailing)  absolute, swipeActions 時のみ
```
- `role` を与えると（menuitem, option, treeitem 等）不可視 button/anchor は描かず、onClick を root に付ける（親が keyboard を持つ）。
- `ItemDescriptionContext` が description の id を配り、スロット内の control が `aria-describedby` で参照できる。

### 3. API

| prop | 型 / 既定 | 意味 | Kivact 提案 |
|---|---|---|---|
| as | `'div'｜'li'｜'span'｜Component` / div | root の要素 | 不要。`.role(Role::..)` で意味を決める（li → `Role::ListItem`） |
| marker | ReactNode | 先頭マーク | `.marker(|| v)`（kit 内部 / list 用） |
| startContent | ReactNode | 先頭（icon/avatar/checkbox） | `.start(|| v)` |
| label | ReactNode（必須） | 主テキスト | `item(label: impl IntoText)`、`.label_view(|| v)` |
| description | ReactNode | 副テキスト | `.description(impl IntoText)` |
| endContent | ReactNode | 末尾（badge/時刻/ボタン） | `.end(|| v)` |
| align | `'center'｜'start'｜'first-line'` / center | スロットの縦位置 | `.align(ItemAlign)` |
| density | `'compact'｜'balanced'｜'spacious'` / balanced | 縦 padding（spacious は横も） | `.density(ItemDensity)` |
| labelLines / descriptionLines | number | 最大行数で ellipsis | `.label_lines(u32)` / `.description_lines(u32)` |
| layout | `'stacked'｜'inline'` / stacked | label と description を縦 / 1 行 | `.layout(ItemLayout::{Stacked, Inline})` |
| onClick | handler | button 意味で押せる | `.on_press(|| ..)` |
| interactiveRef | ref | 行面の click を中の control へ委譲 | `.delegate_to(NodeRef)` |
| href / target / rel | — | anchor 行 | native では **drop**、`.on_press` で代替 |
| isHighlighted | bool / false | hover/keyboard focus の見た目を強制 | `.highlighted(impl IntoValue<bool>)` |
| isSelected | bool / false | 選択の見た目 + aria-selected/aria-current | `.selected(impl IntoValue<bool>)` |
| isDisabled | bool / false | 無効 | `.disabled(..)` |
| swipeActions | `{leading?: ItemSwipeAction[]; trailing?: ItemSwipeAction[]}` | touch swipe の動詞 | `.swipe_actions(SwipeActions{leading: Vec<SwipeAction>, trailing: Vec<SwipeAction>})` |
| swipeBehavior | `'reveal'｜'commit'` / reveal | swipe の挙動 | `.swipe_behavior(SwipeBehavior)` |
| xstyle/className/style/data-testid | — | — | `.style(..)` / `.test_id(..)`、className drop |

`ItemSwipeAction`: `id?`, `label: string`（休止中の button の名前）, `icon?`, `onActivate: () => void|Promise`, `isDisabled?`, `variant?: 'neutral'｜'accent'｜'destructive'`（既定 accent）, `hasRemoval?`（発火後に端で止まる、既定 false）→ `SwipeAction::new(label, on_activate).icon("name").variant(SwipeVariant::{Neutral, Accent, Destructive}).disabled(bool).removal(bool)`。

### 4. Visual spec

root（Item.tsx `styles.root`）: `display:flex; align-items:center; gap: --spacing-2 = 8px; padding-inline: var(--_item-inset-inline) = --spacing-2 = 8px; position:relative; box-sizing:border-box; text-align:start; border-radius: --radius-element = 10px`。

density（`densityStyles`）: compact `padding-block 4px`、balanced `8px`、spacious `12px` + inset 12px。

| density | 高さ（label のみ） | label+description | inline padding |
|---|---|---|---|
| compact | 28px | 48px | 8px |
| balanced | 36px | 56px | 8px |
| spacious | 44px | 64px | 12px |

テキスト:
- label: `color: var(--_item-label-color, --color-text-primary)` = `#000000` / `#ffffff`; `font-size --text-body-size 14px; line-height 20px`（weight は継承 = 400）。string label は既定で 1 行 ellipsis（overflow hidden, nowrap）。`labelLines=n` で n 行 clamp。親が `--_item-label-color` を上書き可（destructive menu item が赤にする等）→ kivact では `ItemTone` 的な label color の上書き口を用意。
- description: `color: var(--_item-description-color, --color-text-secondary)` = `#474747` / `#9e9e9e`; `12px / 20px`。string は 1 行 ellipsis。
- layout inline: content が row、`align-items:center; column-gap: --spacing-1 = 4px`、label は縮まない、description が先に ellipsis。
- start/end content: `flex: 0 0 auto; display:flex`、end は `margin-inline-start: auto`（右端寄せ）。

状態:
| 状態 | 変化 |
|---|---|
| rest（対話行） | 背景 transparent、`cursor: pointer` |
| hover（hover 可能な pointer のみ, `@media (hover:hover)`） | 背景 `--color-overlay-hover` = `#0000000D` / `#FFFFFF0D` |
| pressed（mouse :active） | 背景 `--color-overlay-pressed` = `#0000001A` / `#FFFFFF1A`。coarse pointer では :active を使わず press controller が描く（下記） |
| highlighted（isHighlighted） | 背景 overlay-hover（hover と同じ） |
| selected | 背景 `--color-accent-muted` = `#f1f1f1` / `#262626`（hover より後に適用され、selected が勝つ） |
| focus-visible（内側の button/anchor が keyboard focus） | root に outline `2px solid --color-accent`（`#1b1b1b` / `#f1f1f1`）、offset 3px（`focusOutlineProps.focusWithin` = `:has(:focus-visible)`） |
| disabled | content と endContent が `opacity: 0.5`、root `pointer-events:none; cursor:default`（role 指定時は親に任せ pointer-events は残す）。hover/press は描かない（ENABLED ガード） |
| 静的（非対話）行 | hover/press なし |

swipe panel（ItemSwipeLayer.tsx:90-189）:
- panel: absolute, top/bottom 0, 幅 = 移動量（leading は inline-start 側、trailing は inline-end 側）、`overflow:hidden; border-radius: inherit`。entries の背景 `--color-background-surface`（`#ffffff` / `#262626`）。
- entry: `flex column; align/justify center; gap 4px; min-width 72px; height 100%; padding-inline 12px; font 12px/20px weight 500 (--font-weight-medium); white-space:nowrap`。icon は label の上。
- variant: neutral = 背景 `--color-neutral`（`#0000000F` / `#FFFFFF1A`）+ 文字 text-primary、accent = `--color-accent`（`#1b1b1b` / `#f1f1f1`）+ `--color-on-accent`（`#ffffff` / `#111111`）、destructive = `--color-error`（`#c9303a` / `#ff705d`）+ `--color-on-error`（`#ffffff` / `#111111`）。forced-colors 時 1px CanvasText 枠。disabled entry opacity 0.5。
- leading の entries は row-reverse（外側が最後 = 端）。

RTL: start/end・padding は logical。swipe も logical（leading = inline-end 方向への drag、RTL では左向き）。

### 5. Motion

| 対象 | trigger | property | from→to | duration / easing | file:line |
|---|---|---|---|---|---|
| 対話行の背景 | hover/selected/highlight 変化 | background-color | → token | `--duration-fast-min` 95ms, `--ease-standard` cubic-bezier(0.24,1,0.4,1) | Item.tsx `styles.interactive` |
| touch press | 指 down 後 150ms 保持（`PRESS_ONSET_MS`）、10px 以上動く/scroll でキャンセル（`PRESS_SLOP_PX`） | 背景 overlay-pressed の alpha | 1（即時） | 短い tap は 100ms flash（`PRESS_FLASH_MS`） | utils/pressGesture.ts:38-67 |
| press release | 指 up | `--astryx-press-alpha` 1→0 | 200ms（`pressConsts.releaseMs`, motion tempo 係数 ≤1.5 倍）`cubic-bezier(0, 0, 0.2, 1)`、fill both | utils/interactionOverlay.stylex.ts |
| swipe drag | 指の追従 | root `transform: translate3d(travel,0,0)`、panel は `-travel` | 0s（指が駆動） | Item.tsx `swipeRoot` |
| swipe settle（戻り/休止/slide-out） | release | transform, panel width | `SWIPE_SETTLE_MS` = 200ms, `ease-out`（= cubic-bezier(0,0,0.58,1)） | useSwipeAction.ts:86, ItemSwipeLayer.tsx:107-109 |
| 背景と transform の併用 | swipe 行 | `transition: background-color 95ms ease-standard, transform var(duration) ease-out` | | Item.tsx `swipeRoot` |

swipe の閾値（useSwipeAction.ts:72-88）: 軸決定 10px（縦優位なら scroller に譲る）、commit 点 = max(panel 幅 + 48px, 行幅 × 0.5)、commit 点を越えた移動は 0.35 倍の抵抗、reveal で release 時 panel 幅 × 0.5 以上なら休止（open）、fling = 速度 ≥ 0.6px/ms かつ 移動 ≥ 44px で commit、slide-out は行端 + 24px まで、swipe 後 400ms はクリック抑止。commit 発火後は spring back（hasRemoval なら端で停止）。
reduced motion: settle duration 0（即座に位置へ）。press fade は 200ms 以下に短縮のみ。

### 6. Behaviour & a11y

- onClick 行: 内側 `<button type=button>` が tab stop、Enter/Space で押下。行面の他の部分の click も onClick を呼ぶ（ただし内側の button/a/input/select/textarea 起点の click は除外: `handleContainerClick`）。
- href 行: 不可視 anchor が tab stop。disabled では `role=link aria-disabled tabindex=-1` で行き先なし。
- interactiveRef 行: 行面 click を中の control に委譲（`useClickableContainer`）、行は tab stop を持たない（WCAG 4.1.2 一つの option に一つの focusable）。
- role 指定時: aria-selected は option/tab/row/gridcell/columnheader/rowheader/treeitem でのみ出す。それ以外で selected なら `aria-current="true"`。`aria-disabled` は disabled 時。
- swipe: touch（pointerType）のみ。mouse では開始しない。休止中（resting）の panel entry は実 button（名前 = label）、Escape で閉じる。commit の panel は `aria-hidden` で presentational。休止でない panel は `inert`。
- 既定文言: なし（swipe label は呼び出し側）。

### 7. Kivact 実装方針

- **NEW kit component** `soft::item`（`crates/kivact-ui-basic/src/item.rs` が最も近い：variant/size/media/actions/on_press）。押下・hover・focus は `kivact::interaction` の press（use_press 相当）と `PartState` の state style で描く。
- 標準部品の restyle で Item の見た目を共有するもの: `kivact::ui::list` の item recipe、`menu` の item recipe、`select` の option recipe、`command_palette` の item recipe はすべて Item の寸法（density 別 padding、radius 10px、hover overlay、selected accent-muted、label 14/20、description 12/20）で書く。recipe 用の共通関数 `soft::recipes::item_row(state, density)` を kit 内に置く。
- focus ring: Item は「内側 button が focus-visible のとき root に ring」。kivact では root を focusable にし（または press part を root に付け）、focus-visible state style で outline 2px offset 3px を描く。offset 3px の outline は行の外にはみ出すので、親が clip しないこと（list の inline clip と衝突する場合は offset を内側 -2px に変える fallback。UNRESOLVED: docsite で行 ring の offset が実際に 3px 外側に見えるか要目視）。
- `first-line` 揃え（CSS baseline + zero-width strut）: native では label 1 行目の line box 中心 = padding-top + 10px にスロット中心を合わせる計算で代替（スロット高さ h なら top = padding-top + (20 - h)/2、h > 20 なら text を下げる）。
- 複数行 clamp（`-webkit-line-clamp`）: kivact style の `line_clamp(n)`（crates/kivact-style/src/sheet.rs:1171）+ `text_overflow(TextOverflow::Ellipsis)` で表せる。1 行は `single_line_text` 相当（nowrap + ellipsis）。
- swipe actions: `kivact::interaction::use_drag` で実装可能（軸ロック 10px、抵抗、fling 速度は drag update の速度から）。ただし「touch のときだけ」の判定に pointer type が必要。PointerEvent に pointer kind が無ければ **CORE REQUEST**。行内に panel を描き、行の translate を `Style::translate` で。スワイプ操作は汎用の意味（一覧の行の swipe 動詞）で他 kit にも出うるが、現時点で他の公式 kit に無いので kit 内実装でよい（2 つ目が出たら core へ）。
- `delegate_to`（clickable container）: press を子 control の activate に転送する口。kivact の node_ref に `activate()`（accessibility Activate action の発火）が公開されていれば実装可能、無ければ CORE REQUEST。
- 静的 Item の意味（行 + 名前 + description）は basic にも Item があるため、strategy §6 rule 5 で「中核の標準 `item` 行」を検討対象として記録（CORE REQUEST 候補、優先度低：見た目の差が大きく意味は単純なので、まず kit 内で実装し List の core 化と同時に扱う）。

### 8. 受け入れ確認

- [ ] E2E: on_press 付き item が `Role::Button`（または指定 role）で label を名前、description を description として出す。Enter/Space/クリックで 1 回 press。
- [ ] E2E: disabled item は press されず、disabled 状態を出す。selected item は role に応じて selected / current を出す。
- [ ] E2E: delegate_to 行の面クリックで中の checkbox がトグルし、Tab stop は checkbox 1 つだけ。
- [ ] 目視: 高さ compact 28 / balanced 36 / spacious 44px、radius 10px、gap 8px、hover `#0000000D`（dark `#FFFFFF0D`）、pressed `#0000001A`、selected `#f1f1f1`（dark `#262626`）、背景変化 95ms。
- [ ] 目視: label が長いと 1 行 ellipsis、`description_lines(2)` で 2 行 + …。inline layout で description から先に省略。
- [ ] 目視: first-line 揃えで 8px の status dot が折り返し label の 1 行目中央に来る。
- [ ] touch（可能なら）: 横 drag で panel が出て、半分以上で休止、Escape で閉じる。reduced motion で即時。
- [ ] light/dark で docsite https://astryx.atmeta.com/components/Item と比較。

---

## 3. MetadataList (+ MetadataListItem)

docs: https://astryx.atmeta.com/components/MetadataList ・ source: `packages/core/src/MetadataList/` (MetadataList.tsx, MetadataListItem.tsx, MetadataListContext.tsx, MetadataList.doc.mjs, MetadataListItem.doc.mjs, MetadataList.test.tsx) ・ stories: `apps/storybook/stories/MetadataList.stories.tsx`

### 1. 概要

- オブジェクトの属性（品質・状態・ステータスなど）を **key-value の組** で構造的に表示する（`<dl>`/`<dt>`/`<dd>`）。詳細パネル、設定のサマリ、レコード情報。
- bestPractices（MetadataList.doc.mjs）: 値が短いなら label 位置 `start`、長い/複雑なら `top`。長い一覧は `maxNumOfItems` で畳む。フォーム入力の代わりに使わない。key-value 構造でないデータに使わない。

### 2. Anatomy

```
div.root (flex column)
├─ title div (margin-bottom 12px)            … 任意
├─ dl#contentId (grid / flex)
│   ├─ [label=start]  dt(label) + dd(value)   … MetadataListItem は Fragment で dt/dd を直接 grid に置く
│   └─ [label=top / horizontal] div.stackedWrapper > dt + dd
└─ button "Show more"/"Show less"            … maxNumOfItems 超過時, aria-controls=dl, aria-expanded
```

### 3. API

MetadataList:
| prop | 型 / 既定 | 意味 | Kivact 提案 |
|---|---|---|---|
| children | MetadataListItem[] | 項目 | `soft::metadata_list((metadata_item(..), ..))` |
| columns | `'multi'｜'single'｜number` / single | 列数。multi = `repeat(auto-fill, minmax(280px,1fr))` | `.columns(MetadataColumns::{Single, Multi, Count(u8)})` |
| label | `{position?: 'start'｜'top'; width?: number｜string}` | label 位置と幅（start 時の label 列幅） | `.label_position(LabelPosition::{Start, Top})`, `.label_width(f32)` |
| maxNumOfItems | number | これを超えると畳む（vertical のみ） | `.max_items(usize)` + 展開状態 `.expanded(Binding<bool>)`（省略時は内部 signal） |
| orientation | `'vertical'｜'horizontal'` / vertical | horizontal = 折返す横並び（label は常に top） | `.orientation(Orientation::{Vertical, Horizontal})` |
| title | ReactNode | 上の見出し | `.title(|| view)` |
| xstyle/className/style/data-testid | — | — | `.style` / `.test_id`、className drop |

既定の label 位置: multi または columns>1 なら `top`、それ以外 `start`（MetadataList.tsx `defaultLabelConfig`）。horizontal は常に top。

MetadataListItem:
| prop | 型 | Kivact 提案 |
|---|---|---|
| label | string（必須） | `metadata_item(label: impl IntoText, value: impl IntoView)` |
| children | ReactNode（値） | 第 2 引数の view |
| icon | ReactNode | `.icon(icon("name"))`（label の前） |
| data-testid | string（start 時は `-label` / `-value` に分配） | `.test_id(..)` |

### 4. Visual spec

レイアウト（MetadataList.tsx styles）:
| 条件 | dl のレイアウト |
|---|---|
| single + start | `grid; grid-template-columns: auto minmax(0,1fr); gap: 8px (行) 16px (列); align-items: baseline`。`label.width` 指定時 `<width> minmax(0,1fr)` |
| multi + start（columns=n） | `repeat(n, auto minmax(0,1fr))`、gap 16px。columns='multi' + start は `repeat(auto-fill, minmax(280px,1fr))` gap 16px |
| single + top | `grid; 1fr; gap 12px` |
| multi + top | `repeat(auto-fill, minmax(280px, 1fr))`（columns=n は `repeat(n, 1fr)`）、gap 16px |
| horizontal | `flex row wrap; gap 16px` |

テキスト（MetadataListItem.tsx styles）:
- label（dt）: `color --color-text-secondary`（`#474747` / `#9e9e9e`）、`14px / 20px`、weight `--font-weight-medium` 500、`display:flex; align-items:center; gap 8px`、start 配置時 `min-height: 24px`、`word-break: break-word`。
- value（dd）: `color --color-text-primary`（`#000000` / `#ffffff`）、14px/20px、weight 400、start 時 `min-height 24px`、break-word。
- stacked（top）: wrapper `flex column; gap: 2px`、label/value に min-height なし。
- icon: `inline-flex; color --color-text-secondary`、label の前、gap 8px。icon サイズは呼び出し側（通常 Icon sm 16px）。
- title: `margin-bottom: --spacing-3 = 12px`（中身は呼び出し側の Heading 等）。
- Show more/less ボタン: 背景なし・枠なし、`padding: 8px 0`、`color --color-accent`（`#1b1b1b` / `#f1f1f1`）、14px/20px weight 500、`align-self:flex-start`、cursor pointer。hover/press の見た目変化は定義なし（UNRESOLVED: docsite で hover 時の下線等が無いことを確認）。focus-visible はブラウザ既定 outline のみ（スタイル未指定）→ kit では標準 focus ring（2px accent, offset 3px）を付ける。

```
 Status        Active                      (label 500 secondary | value 400 primary)
 Owner         Kashiwade                   row gap 8, col gap 16, label 列 = auto
 Created       2026-11-03
 Show more                                  (accent 500, padding 8px 0)
```

RTL: grid の列順が自動で鏡像。状態: なし（静的）。disabled/selected なし。

### 5. Motion

- アニメーションなし。Show more は即時に項目が増減する（transition 定義なし）。

### 6. Behaviour & a11y

- `<dl>` / `<dt>` / `<dd>` の意味（description list）。kivact の `Role`（crates/kivact-types/src/semantics.rs）には DescriptionList/Term/Definition が無いので、`Role::List` + 各組を `Role::ListItem`（名前 = label、description = 値テキスト）にする。term/definition の役割が必要になった時点で `Role` への追加を CORE REQUEST（AccessKit には DescriptionList/Term/Definition がある）。
- Show more ボタン: `aria-controls` = dl、`aria-expanded` = 展開状態。押すと全件 / maxNumOfItems 件を切替。フォーカスはボタンに残る。
- 既定文言: `@astryx.metadataList.showMore` "Show more" → `soft.metadata_list.show_more`、`@astryx.metadataList.showLess` "Show less" → `soft.metadata_list.show_less`。

### 7. Kivact 実装方針

- **NEW kit component**（`soft::metadata_list`, `soft::metadata_item`）。近い kivact-ui-basic: 無し（`field.rs` の field_content / `item.rs` が近い要素）。`kivact::interaction::use_disclosure` を Show more/less に使う（aria-expanded / controls の意味が得られる）。
- grid（auto 列 + minmax(0,1fr)）: kivact layout に CSS grid が無い場合、start 配置は「label 列幅 = 全 label の最大測定幅」を `measure_text` で求めて各行 `row(label.width(w), value.grow(1))` にする。`auto-fill minmax(280px,1fr)` は container 幅 / 280 の床で列数を出し、行ごとに等幅で並べる（node_ref().rect() で幅を読む）。
- 中核への要求: key-value 表示は一般的だが、他の公式 kit に同じ部品が無いので kit 内でよい（rule 5 の対象外）。

### 8. 受け入れ確認

- [ ] E2E: label/value の組が AT で label 名と値テキストを持つ。Show more ボタンが expanded=false → 押すと true になり項目数が全件になる。
- [ ] 目視: start 配置で label 列が最長 label 幅に揃い、行間 8px・列間 16px、label 500 グレー（`#474747` / `#9e9e9e`）、value 黒/白。
- [ ] 目視: columns=multi で 280px 以上の列が自動で並ぶ。label=top で label の 2px 下に値。
- [ ] 目視: horizontal で横並び・折返し gap 16px。
- [ ] light/dark で docsite と比較。

---

## 4. OverflowList

docs: https://astryx.atmeta.com/components/OverflowList ・ source: `packages/core/src/OverflowList/OverflowList.tsx`, `hooks/useOverflow.ts`, `hooks/computeOverflow.ts` ・ stories: `apps/storybook/stories/OverflowList.stories.tsx`

### 1. 概要

- 横一列（または maxRows 行まで折返し）に項目を並べ、入りきらない項目を隠して `overflowRenderer` で「+N」バッジやメニューにまとめる。breadcrumb、toolbar、tag 列など。
- bestPractices: 意味のある overflowRenderer（"+N more" badge、dropdown、件数）を渡す。行に既にメニューがあるなら `onOverflowChange` で隠れた項目をそのメニューへ流す（2 つ目のアンカーを作らない）。重要項目は `minVisibleItems` で残し、`maxVisibleItems` で固定上限。`maxRows` で 2 行 tag cloud 等。縦積みには使わない。

### 2. Anatomy

```
(fragment)
├─ measure div  aria-hidden inert  (absolute, visibility:hidden, height 0, overflow hidden, flex row nowrap)
│    └─ 全 children + overflowRenderer(全項目) の最大幅版
└─ container div [.astryx-overflow-list] (flex row, align-items center, overflow hidden, nowrap, min-width 0)
     ├─ [collapseFrom=start] overflowRenderer(hidden items)
     ├─ visible children
     └─ [collapseFrom=end]   overflowRenderer(hidden items)
```
OverflowList 自体は role を持たない（意味は子と renderer が持つ）。

### 3. API

| prop | 型 / 既定 | 意味 | Kivact 提案 |
|---|---|---|---|
| children | ReactNode（各 1 要素） | 項目 | `soft::overflow_list(items: impl IntoValue<Vec<T>>, key, item: impl Fn(Value<T>) -> V)` |
| gap | SpacingStep `0｜0.5｜1｜1.5｜2｜3｜4｜5｜6｜8｜10` / 2 | 項目間隔（px = step×4） | `.gap(Spacing)` または `.gap(f32)` 既定 8 |
| minVisibleItems | number / 0 | 最低表示数 | `.min_visible(usize)` |
| maxVisibleItems | number | 上限 | `.max_visible(usize)` |
| maxRows | number | 折返し行数上限（>1 で multi row） | `.max_rows(usize)` |
| collapseFrom | `'start'｜'end'` / end | どちら端から隠すか | `.collapse_from(Edge::{Start, End})` |
| behavior | `'observeParent'｜'observeSelf'` / observeSelf | 利用可能幅の基準 | `.measure(OverflowMeasure::{OwnWidth, ParentWidth})` |
| overflowRenderer | `(hidden: OverflowItem[]) => ReactNode` | 隠れた項目の表示 | `.overflow(|hidden: Value<Vec<T>>| view)` |
| onOverflowChange | `(hidden) => void` | 隠れた項目の通知（測定確定後・同一フレーム） | `.on_overflow_change(|hidden: Vec<K>| ..)` |
| xstyle/className/style | — | — | `.style(..)` |

### 4. Visual spec

- container: `display:flex; align-items:center; overflow:hidden; white-space:nowrap; min-width:0`、gap は `gapStyles[gap]`（既定 `--spacing-2` = 8px）。
- multi row（maxRows>1）: `flex-wrap: wrap; align-content: flex-start; white-space: normal; max-height: rowHeight*maxRows + gap*(maxRows-1)`（rowHeight は測定値）。
- `observeParent` で overflow 中は `width: 100%`。
- 自身の色・枠・余白は無い。見た目は子と renderer（例: Badge "+3"、IconButton + DropdownMenu）で決まる。
- 計算（computeOverflow.ts）: 1 行の場合、先頭から貪欲に詰め、**最後の項目以外**は「indicator 幅 + gap」を予約して判定。`visibleCount = clamp(fit, min, max ?? count)`。multi row は行パッキングで最終行だけ indicator 分を予約。1 項目が行より広い場合はその行を単独で占め（見た目は clip）。
- RTL: flex の順序が自動で鏡像。collapseFrom は論理（start/end）。

### 5. Motion

- アニメーションなし（項目の出入りは即時）。測定は layout effect 内で paint 前に確定するため 1 フレームのちらつきもない。kivact でも「測定 → 同一フレームで反映」にする（不可能なら初回は全件を clip 表示し、次フレームで確定）。

### 6. Behaviour & a11y

- 測定用 DOM は `aria-hidden` + `inert`（AT とフォーカスから除外）。
- 隠れた項目は描画されない（AT からも消える）。renderer 側のメニューが代替経路。
- keyboard: 独自のものは無い（子の Tab 順）。
- 既定文言: なし。

### 7. Kivact 実装方針

- **NEW kit component**（`soft::overflow_list`）。近い kivact-ui-basic: `breadcrumb.rs`（ellipsis）、`pagination.rs`。
- 測定は公開 API で可能: `kivact::view::node_ref()` の `rect()`（Value<Option<Rect>>）で container 幅と各項目の幅を読める（message_scroller.rs が同じ手法を使用）。測定用の不可視の行（absolute, height 0, clip）に全項目と indicator を置き、その子の rect から `computeOverflow` 同等の純関数で visible_count を出し、表示用の行には visible な項目だけを `for_each` で描く。
- 注意点: 測定用ノードの子を AT とフォーカスから外す口（`inert` 相当: `.hidden_from_accessibility()` / focus 不可）が必要。無ければ **CORE REQUEST**（「レイアウトはするが描画・AT・フォーカスから除外する node」= CSS `visibility:hidden` + `inert`）。また、同一フレームで測定結果を反映できない場合は 1 フレーム遅れの再レイアウトになる（許容、UNRESOLVED: kivact の layout→effect→relayout が同一フレーム内で収束するか要確認）。
- 汎用の「幅に収まる数を測る」hook は toolbar・breadcrumb・tag_input の折返し等でも使えるので、2 つ目の公式 kit が必要とした時点で `kivact::interaction::use_overflow` として core 化する（今は kit 内、記録のみ）。

### 8. 受け入れ確認

- [ ] E2E: 幅 300px で 10 個の tag を並べ、隠れた数 N の "+N" が出る。幅を広げると N が減る（grow back）。
- [ ] E2E: `on_overflow_change` が隠れた key の列で 1 回だけ呼ばれ、同じ構成では再度呼ばれない。
- [ ] E2E: 測定用ノードが AT ツリーに現れない、Tab で到達しない。
- [ ] 目視: gap 8px、indicator は最後の可視項目の 8px 右、collapse_from(Start) では左端に indicator。
- [ ] 目視: max_rows(2) で 2 行まで折返し、2 行目の末尾に indicator。

---

## 5. TreeList (+ useTreeFocus)

docs: https://astryx.atmeta.com/components/TreeList ・ source: `packages/core/src/TreeList/` (TreeList.tsx, TreeListItem.tsx, TreeListBranches.tsx, TreeListTypes.ts, TreeList.doc.mjs, TreeList.test.tsx, __tests__), `hooks/useTreeFocus.ts` ・ stories: `apps/storybook/stories/TreeList.stories.tsx`, `TreeListA11y.stories.tsx`, `TreeListTreeViewA11y.stories.tsx`

### 1. 概要

- 階層データを展開/折畳みできる木で表示（ファイルエクスプローラ、入れ子カテゴリ）。**データ駆動**（`items` の再帰配列）、展開状態は内部管理（`isExpanded: true` で初期展開を種付け）。
- bestPractices: 各ノードに意味のある label と icon。重要な枝は初期展開。leaf はツリーに展開可能項目が 1 つでもあれば chevron 列分の余白を取る（独自 CSS で字下げを調整しない）。4–5 階層を超えて入れ子にしない。平坦なデータには List を使う。

### 2. Anatomy

```
div.root [.astryx-tree-list] (position:relative, isolation:isolate,
         --tree-list-indent:16px, --tree-list-row-gap:2px)
├─ header div#hid (margin-bottom 8px)     … 任意, aria-labelledby
└─ ul role="tree"
   └─ li role="treeitem" (aria-level, aria-posinset, aria-setsize, aria-expanded?, aria-selected?, tabindex roving)
      ├─ div.treeBranches (padding-inline-start 8px) > guide 線 (absolute, variant=lineGuides)
      ├─ div.rowWrapper (padding-block: row-gap/2 = 1px)
      │   └─ div.contentWrapper (row: radius 10, padding-inline 8, density padding-block, hover/selected 背景)
      │       └─ div.indentedContent (padding-inline-start = indent)
      │           ├─ chevron button[data-tree-toggle] (16×16, tabindex -1, aria-expanded, aria-label "Toggle children")
      │           ├─ startContent
      │           ├─ content: a | button | span (tabindex -1) > label#id + description#id
      │           └─ endContent (margin-inline-start:auto)
      └─ ul role="group"  (展開時の子)
```

### 3. API

TreeList:
| prop | 型 / 既定 | 意味 | Kivact 提案 |
|---|---|---|---|
| items | `TreeListItemData[]` | 再帰データ | `soft::tree_list(nodes: impl IntoValue<Vec<TreeNode<K>>>, expanded: Binding<Vec<K>>)`（kivact の TreeNode は flat: key, parent, label）+ 行の見た目は `.row(|node: Value<TreeRowData<K>>, state| view)` |
| density | `'compact'｜'balanced'｜'spacious'` / balanced | 行の上下 padding | `.density(TreeDensity)` |
| variant | `'lineGuides'｜'noGuides'` / lineGuides | 階層ガイド線 | `.variant(TreeVariant::{LineGuides, NoGuides})` |
| header | ReactNode | 見出し、aria-labelledby | `.header(|| view)` |
| animatedLayout | `useAnimatedLayout` の handle | 展開/折畳みで後続行を glide | kit では常時有効（`.animate(bool)` 既定 true）。container を指定する概念は不要 |
| aria-label / aria-labelledby | — | 木の名前 | `soft::tree_list` の第 1 引数 `label: impl IntoText`（標準部品の慣例） |
| onKeyDown | — | 先に呼ばれる | 不要（drop） |
| xstyle/className/style/data-testid | — | — | `.style` / `.test_id` |

TreeListItemData（行）: `id`, `label`, `description?`, `startContent?`, `endContent?`, `children?`, `onClick?`, `href?`, `target?`, `isDisabled?`, `isSelected?`, `isExpanded?`（初期のみ）, `xstyle?/className?/style?`
→ `TreeNode::new(key, parent, label).disabled(bool)` + kit 側の付帯データ `TreeRowData{description, start: icon name, end slot}`; `onClick` → `.on_activate(|key| ..)`（use_tree の on_activate）; `href/target` → drop（on_activate で代替）; `isSelected` → caller の selection Binding（`ListSelection::Single(Binding<Option<K>>)` / `Multiple`）; `isExpanded` → `expanded` Binding の初期値。

### 4. Visual spec

ルート変数: `--tree-list-indent: --spacing-4 = 16px`（1 階層の字下げ）、`--tree-list-row-gap: --spacing-0-5 = 2px`（行間）。header `margin-bottom 8px`。

行（TreeListItem.tsx:55-317）:
- rowWrapper: `padding-block: calc(row-gap/2) = 1px`（行の上下 1px、合計 2px の間隔）。
- contentWrapper: `border-radius --radius-element 10px; display:flex; align-items:center; gap 8px; padding-inline 8px; overflow:hidden`。
- density: compact `padding-block 4px`、balanced `8px`、spacious `12px`。文字は全 density で `14px / 20px`。→ 行の高さ compact 28 / balanced 36 / spacious 44px（+ 上下 1px の gap）。description 付きは +20px（12px/20px）。
- indentedContent: `padding-inline-start: --_tree-indent`。
  - 展開可能（子あり）行: `indent = level × 16px`（level は 0 起点）。
  - leaf でツリーに展開可能項目がある: `indent = level × 16 + 16 + 8 = level×16 + 24px`（chevron 16px + gap 8px 分を空け、兄弟の label 位置に揃える）。
  - 全体が平坦: `level × 16`（実際は 0）。
- chevron: button 16×16px、`color --color-icon-secondary`（`#6a6a6a` / `#9e9e9e`）、`border-radius --radius-inner 6px`、背景なし。glyph は `chevronRight` 16×16px。展開時 `rotate(90deg)`（下向き）、折畳み `rotate(0)`。RTL は `scaleX(-1) rotate(..)`（左向き → 下向き）。
- label: `--color-text-primary`（`#000000` / `#ffffff`）14/20。description: `--color-text-secondary`（`#474747` / `#9e9e9e`）12/20。
- start/end content: `flex-shrink 0; align-items:center`、end は右寄せ。

状態:
| 状態 | 変化 |
|---|---|
| hover（対話行 = onClick/href あり、または子ありで onClick なし） | `background-image: linear-gradient(--color-overlay-hover)`（`#0000000D` / `#FFFFFF0D`）、hover 可能 pointer のみ |
| pressed | overlay-pressed（`#0000001A` / `#FFFFFF1A`）、touch は press controller（Item 節と同じ 150ms onset / 200ms fade） |
| selected | `background-color --color-accent-muted`（`#f1f1f1` / `#262626`） |
| focus-visible（treeitem が roving focus） | contentWrapper に focus ring 2px `--color-accent` offset 3px（`focusWithinOrPublished`: li の :focus-visible を変数で公開し行に描く）。focus 行は z-index 2 に持ち上げ（ガイド線より上） |
| disabled | 行全体 `opacity 0.5; pointer-events none`。focus は可能（roving で skip、AT には残る） |
| expanded/collapsed | chevron の回転のみ |

ガイド線（TreeListBranches.tsx、variant=lineGuides）:
- 縦線のみ（横の肘線は無い）。幅 1px、`border-radius 1px`、色 `--color-border-emphasized`（`#d4d4d4` / `#525252`）。線の container は幅 `--spacing-5` 20px、z-index 1、pointer-events none、線は container 中央。
- x 位置: `inset-inline-start = 8px + 16px/2 − 20px/2 + level × 16px = 6px + level×16` の container の中央 → 線の中心 x = **16px + level × 16px**（各親の chevron 中心の真下）。
- 祖先レベルごと: その祖先が最後の子でなければ、その level の線を li の高さ全体（`100% + 1px`、次の兄弟と繋がる）に描く。
- 自分のレベル（nestedLevel−1）: 最後の子なら `height: calc(100% − row-gap/2)`（行の下端で止める）、それ以外は `100% + 1px`。
- 線は hover/selected の背景より上（z-index 1）に描かれ、背景で線が途切れない。

```
balanced, lineGuides:
 [v] src                       ← chevron 16px (rotate 90°), indent 0
  │  [v] components            ← indent 16
  │   │      Button.tsx        ← leaf: indent 16+24 = 40
  │   │      Card.tsx
  │       package.json  (last)  ← 線は行下端で止まる
     README.md                  ← leaf level0: indent 24
```

### 5. Motion

| 対象 | trigger | property | 値 | duration / easing | file:line |
|---|---|---|---|---|---|
| chevron 回転 | 展開/折畳み | transform | rotate(0) ⇄ rotate(90deg) | `--duration-fast` 125ms, `--ease-standard` cubic-bezier(0.24,1,0.4,1)（animatedLayout 無し時） | TreeListItem.tsx:253-285 |
| 行の hover 背景 | hover | background-image | → overlay-hover | 125ms ease-standard | TreeListItem.tsx:148-156 |
| 子グループの出入りと後続行の glide（animatedLayout あり） | 展開/折畳み | 後続行の translateY、子グループの clip-reveal | before→after の差分 | 共通 spring `springs.settle` = `{duration: 0.5s, bounce: 0}`（臨界減衰, ω0 = 2π/0.5 ≈ 12.57 rad/s）、`linear()` に sampling。chevron も同じ timing | motion/animatedLayout.ts:92-170, motion/uikit/springs.ts:41 |
| 折畳みの exit | 折畳み | 子グループを exit 中は描画維持（inert, `data-tree-exiting`）し motion 終了で除去 | | 同上 | TreeList.tsx `holdLayoutExit` |
| press release | touch | overlay alpha 1→0 | 200ms cubic-bezier(0,0,0.2,1) | utils/interactionOverlay.stylex.ts |

reduced motion: glide は行わず、coarse pointer では opacity のみの cross-fade 200ms `cubic-bezier(0.42, 0, 0.58, 1)`（motion/uikit/timing.ts:92-110、animatedLayout.ts:159-167 で 200ms 上限）。fine pointer では即時。chevron 回転も即時（UNRESOLVED: chevron の transition が reduced-motion で 0 になるかは applyLayoutChangeTransition の reduced 分岐に依存、要確認）。

kit 方針: animatedLayout を常に有効にする（docsite の TreeList 例は glide する）。

### 6. Behaviour & a11y

useTreeFocus（WAI-ARIA APG tree, `hooks/useTreeFocus.ts`）:
| キー | 動作 |
|---|---|
| ↓ / ↑ | 次 / 前の可視 treeitem（disabled は skip、wrap しない） |
| → | 折畳み親 → 展開。展開済み親 → 最初の子へ。leaf → 何もしない |
| ← | 展開済み親 → 折畳み。それ以外 → 親へ |
| Home / End | 最初 / 最後の可視 item |
| Enter / Space | 行の内側 action（a[href] / button(非 toggle)）を click。無ければ親は展開トグル |
| 文字入力 | typeahead（500ms でバッファリセット）。label 先頭一致で次の item へ |
| RTL | ← → が反転（container の computed direction） |

- roving tabindex: tab stop は 1 つ。初期値 = 最初の selected かつ enabled、無ければ最初の enabled（TreeList.tsx `findInitialTabbableId`）。chevron・内側 a/button は tabindex -1。
- 折畳みで focus を持つ子が消えるとき、focus はその親行へ移る（`useFocusLeavingChange` + `moveFocusLeftBehind`）。
- 行面 click: onClick があればそれ、無く子があれば展開トグル。href 行は面全体がリンク（修飾キー付き click も anchor に再送）。
- ARIA: `tree`（名前 = header か aria-label）、`treeitem` に `aria-level`（1 起点）, `aria-posinset`, `aria-setsize`, `aria-expanded`（子あり時）, `aria-selected`（selected 時のみ true）, `aria-disabled`, `aria-labelledby` = label id, `aria-describedby` = description id。子は `role=group`。
- 既定文言: `@astryx.treeList.toggleChildren` "Toggle children" → `soft.tree_list.toggle_children`（chevron button の名前）。

### 7. Kivact 実装方針

- 振る舞いは `kivact::interaction::use_tree`（crates/kivact-interaction/src/tree.rs）がほぼ同じ意味を持つ: flat な `TreeNode{key, parent, label, disabled}`、caller-owned `expanded: Binding<Vec<K>>`、`ListSelection`、APG のキー、`filter`、`on_activate`、`TreeItemState{input, selected, active, branch, level}`。use_list 経由で typeahead も可能。
- 一方、標準 `kivact::ui` に tree 部品は無く、kivact-ui-creative が `outliner`（use_tree ベース、crates/kivact-ui-creative/src/outliner.rs）を、kivact-ui-game が `tree(entries, selection)`（use_tree ベース、crates/kivact-ui-game/src/layout.rs:98-）をそれぞれ独自に持つ。**既に 2 つの公式 kit が tree view の意味を持っており**、soft の TreeList で 3 つ目になるため、strategy §6 rule 5 により **CORE REQUEST: 標準 `kivact::ui::tree(label, nodes, expanded)`**（use_tree + 行の slot `.item(|node, state| view)` + `TreeRecipes{root, row, chevron, guide, label, description}` + `TreeItemState` に `is_last` / `ancestors_last: Vec<bool>`（ガイド線用）を追加）を中核に足し、soft kit は recipe で restyle する。creative の outliner はドメイン（restriction flag, drop plan）を持つので kit に残る。
- core 化までの暫定: soft kit 内に `soft::tree_list` を use_tree 上に作るが、API は上の標準形に合わせる（後で中身を差し替え）。
- use_tree に無い/要確認の機能（CORE REQUEST 候補）:
  - ガイド線用の位置情報（`is_last`, 祖先ごとの `is_last`）: `TreeRow` に追加。
  - 展開/折畳みの glide（後続行の移動 + 子グループの reveal）: kivact-motion の補間で行の y を spring（duration 0.5s, bounce 0）で動かす。全行の位置アニメーションは「keyed list の layout 変化アニメーション」の汎用口が必要 → **CORE REQUEST**（for_each の項目の位置変化を補間する animated layout。Table の row glide、Chat のメッセージ追加でも同じ要求）。
  - 折畳み中の子を exit 完了まで描画維持: 同じ animated layout の exit 保持。
  - 行の内側 action（href / onClick）: use_tree の `on_activate` で代替。
- 見た目は kivact-ui-basic に tree が無いので、行は `soft::item` と同じ寸法で描く。ガイド線は各行の `Style::paint`（局所描画）で 1px 縦線を描くのが簡単（行の左から 16 + level×16 px）。

### 8. 受け入れ確認

- [ ] E2E: `Role::Tree`（名前）> `Role::TreeItem` に level / posinset / setsize / expanded が出る。
- [ ] E2E: ↓↑ で可視行を移動、→ で展開→子へ、← で折畳み→親へ、Home/End、Enter で on_activate、文字で typeahead。disabled 行は skip。
- [ ] E2E: 展開中の子に focus がある状態で親を折畳むと focus が親へ移る。Tab で木に入ると selected 行（無ければ先頭）に入る。
- [ ] 目視: 行高 balanced 36px + 行間 2px、字下げ 16px/階層、leaf は +24px、chevron 16px グレー（`#6a6a6a` / `#9e9e9e`）が展開で 90° 回転（125ms）。
- [ ] 目視: ガイド線 1px `#d4d4d4` / `#525252` が親 chevron の中心下を通り、最後の子で止まる。hover 背景で線が切れない。
- [ ] 目視: 展開時に後続行が 0.5s の臨界減衰で滑り、reduced motion では即時。
- [ ] light/dark で docsite https://astryx.atmeta.com/components/TreeList と比較。

---

## 6. Table (+ TableBody, TableCell, TableFooter, TableHeader, TableHeaderCell, TableRow, TableSelectionToolbar, useGridFocus)

docs: https://astryx.atmeta.com/components/Table ・ source: `packages/core/src/Table/` (Table.tsx, BaseTable.tsx, TableHeader/Body/Footer/Row/Cell/HeaderCell.tsx, TableSelectionToolbar.tsx, table.stylex.ts, useTableCellStyles.ts, columnUtils.ts, glidingDivider.ts, rowDivider.ts, tableContextMenu.tsx, types.ts, useBaseTablePlugins.ts, __tests__/) ・ stories: `apps/storybook/stories/Table.stories.tsx`, `TableA11y.stories.tsx`, `PowerSearchWithTable.stories.tsx`

### 1. 概要

- 行と列で構造化データを表示する。**データ駆動**（`data` + `columns` + `plugins`）が基本で、`children` モード（TableHeader/TableBody/TableFooter/TableRow/TableCell を直接組む静的な表）も持つ。ソート・選択・ページング・列管理などはすべて **plugin**（hook が返す `TablePlugin`）で後付けする（7 章）。
- bestPractices（Table.doc.mjs）: density と dividers をデータの密度と読み方に合わせる。セルは renderCell で Badge / StatusDot / Avatar を組む。children モードでは行を必ず TableHeader/Body/Footer に入れる。幅は省略 = 等分（最小 60px）、`proportional(n, {minWidth})` で比率と下限、`pixel(n)` で固定。全 plugin を一度に有効にしない。テキストの多い列で 60px 下限に頼らない。一貫した列を持たないデータには使わない（List / Card を使う）。
- 重要: Astryx Table は **ネイティブ `<table>` の意味（role table）** であり、ARIA grid / セル単位の矢印キー移動を持たない（`useGridFocus` は Calendar 専用で Table は使わない）。行内の control（checkbox, sort ボタン, resize handle, chevron）がそれぞれ Tab stop。

### 2. Anatomy

```
[plugin chrome above: pagination(position above) …]
div role="group" aria-label=<table name|"Table"> [.astryx-table-scroll-wrapper]   … 横スクロール viewport（スクロール可能時 tabindex 0）
└─ div (content)
   ├─ beforeTable (plugin chrome)
   ├─ table [.astryx-table] (width 100%, border-collapse, table-layout fixed; min-width = 列下限の合計)
   │   ├─ thead = TableHeader
   │   │   └─ tr = TableRow(isHeaderRow)
   │   │       └─ th scope=col = TableHeaderCell  (data-column-key, title=見出し文字列)
   │   │           ├─ before / [label + after(sort icon, filter funnel)] / overlay(resize handle) / below(inline filter)
   │   ├─ tbody = TableBody
   │   │   ├─ tr = TableRow  (aria-rowindex? , aria-selected? , aria-level?)
   │   │   │   └─ td = TableCell (context menu trigger wrapper 任意)
   │   │   └─ [空] tr > td colSpan=全列 > EmptyState(compact, "No data")
   │   └─ tfoot = TableFooter (children モードのみ)
   └─ afterTable (plugin chrome)
[plugin chrome below: pagination …]

TableSelectionToolbar（表の外に置く）: Toolbar(role=toolbar, size sm, variant muted)
   start: 製品の一括操作 | end: "{n} selected" + ghost Button "Unselect All"
```
- plugin pipeline（types.ts `TablePlugin`）: `transformColumns → transformTable → transformHeaderCell → transformHeaderRow → transformBodyCell → transformBodyRow → transformScrollWrapper → transformTableContext`。plugin は named record（`plugins={{sort, selection}}`）で渡し、`useBaseTablePlugins` が安定した配列にする。header/body cell は `contextMenuActions` を足せ、表が 1 セル（ヘッダ）/ 1 行ごとに 1 つの右クリックメニューに集約する。

### 3. API

Table（data-driven）:
| prop | 型 / 既定 | 意味 | Kivact 提案（`kivact::ui::data_table` の restyle + 不足は core へ） |
|---|---|---|---|
| data | `T[]` | 行 | `data_table(rows, key, columns)` の `rows` |
| columns | `TableColumn<T>[]`（省略で data のキーから自動生成） | 列 | `Vec<DataColumn<R>>`（自動生成は **drop**） |
| idKey | `keyof T ｜ (item)=>string｜number`（省略で index） | 行 key | `key: impl Fn(&R) -> K` |
| plugins | `Record<string, TablePlugin>` | 機能の追加 | plugin 機構は native では不要。各機能は builder method（7 章）。 |
| density | `'compact'｜'balanced'｜'spacious'` / balanced | セル padding | **CORE REQUEST** `DataTable::density(TableDensity)`（recipe が読む状態として。現状 `row_height` のみ） |
| dividers | `'rows'｜'columns'｜'grid'｜'none'` / rows | 罫線 | `.variant(soft::TableDividers::{Rows, Columns, Grid, None})`（Variant で recipe が読む。kit 内で可） |
| isStriped | bool / false | 偶数行に muted 背景 | `.variant(soft::TableStriped(true))`。行 index を recipe が知る必要 → **CORE REQUEST**: `DataTableRowState` に `index: usize`（表示順） |
| hasHover | bool / false | 行 hover 背景 | `.variant(soft::TableHover(true))`（row recipe の hover state で描く。DataTableRowState.interaction.input に hover あり） |
| verticalAlign | `'middle'｜'top'｜'bottom'` / middle | セル縦位置 | `.variant(soft::TableVerticalAlign::..)` |
| textOverflow | `'wrap'｜'truncate'` / wrap | 既定セル文字の折返し / 1 行省略 + 省略時のみ tooltip | `.variant(soft::TableTextOverflow::{Wrap, Truncate})`。標準 data_table は現在 `single_line_text`（常に 1 行）→ wrap と「省略時だけ tooltip」は **CORE REQUEST** |
| emptyState | `ReactNode｜false` / `<EmptyState title="No data" isCompact/>` | 空表示 | **CORE REQUEST** `DataTable::empty(|| view)`（現状は固定文言 `data_table.empty`）。文言は soft では "No data" |
| rowIndexStart / rowCount | number | aria-rowindex / aria-rowcount | page 時に data_table が自動で出すべき（**CORE REQUEST**: Row の index/count semantics） |
| aria-label / aria-labelledby | — | 表と scroll region の名前 | `.name(..)` / `labelled_by`（data_table は既定 `data_table.label`） |
| ref/xstyle/className/style | — | — | `.style(..)`、className drop |

TableColumn<T>:
| field | 型 / 既定 | Kivact 対応 |
|---|---|---|
| key | string | 列の位置 index（DataColumn は位置で識別）。安定 key が必要な機能（列設定・幅保存）のため **CORE REQUEST**: `DataColumn::key(&'static str)` 等の列 id |
| header | ReactNode（既定 = key の先頭大文字化） | `DataColumn::new(title, text)` の title。リッチ header は **CORE REQUEST** `DataColumn::header(|| view)` |
| width | `proportional(n,{minWidth})｜pixel(n)`、省略 = proportional(1) 下限 60px、proportional 既定下限 120px | 現状 `DataColumn::width(px)` 固定のみ → **CORE REQUEST** `DataColumn::flex(f32).min_width(f32)`（比率と下限、合計下限超過で横スクロール） |
| align | `'start'｜'center'｜'end'` / start | `numeric(true)` = end のみ → **CORE REQUEST** `DataColumn::align(TableAlign::{Start, Center, End})` |
| sortable | `bool｜{sortKey}` | `DataColumn::compare` / `sortable()` |
| resizable | bool / true | 列単位の可否 → **CORE REQUEST** `DataColumn::resizable(bool)`（現状は表全体の `.resizable(widths)`） |
| filter | field key / `{field, operator}` | 7.2 参照 |
| renderCell | `(item)=>ReactNode` | `DataColumn::cell(|row| view)` |

children モード（静的表）: `TableHeader`, `TableBody`, `TableFooter`（`thead/tbody/tfoot`, xstyle のみ）、`TableRow`（`isHeaderRow`, xstyle）、`TableCell`（`scope, headers, colSpan, rowSpan, contextMenuActions`）、`TableHeaderCell`（`scope, contextMenuActions`）。Table の context（density 等）を読んで同じ見た目になる。→ Kivact: `soft::table((soft::table_header(..), soft::table_body(..), soft::table_footer(..)))`, `soft::table_row(..)`, `soft::table_cell(content).col_span(n)`, `soft::table_head(content)`（kivact-ui-basic `table.rs` と同形、density/dividers を `.density()` / `.dividers()` で親から provide_context）。

TableSelectionToolbar:
| prop | 型 / 既定 | Kivact 提案 |
|---|---|---|
| selection | `TableSelectionState`（useTableSelectionState の出力: selectedKeys, selectedCount, hasSelection, clearSelection） | `soft::table_selection_toolbar(selection: Binding<Vec<K>>)` |
| label | string / "Bulk actions" | `.label(..)`（toolbar の名前） |
| renderSelectionLabel | `(count)=>ReactNode` / "{count} selected" | `.selection_label(|count| view)` |
| clearLabel | string / "Unselect All" | `.clear_label(..)` |
| startContent 他 Toolbar props（size 既定 sm, variant 既定 muted, gap, dividers） | | `.actions(|| view)` |

### 4. Visual spec

表全体: `font-family: inherit; color --color-text-primary`（`#000000` / `#ffffff`）。`width 100%; border-collapse: collapse; table-layout: fixed`（children モードは auto）。line-height は reset の `1.5` を継承（14px → 21px）。

セルの density（TableCell.tsx / TableHeaderCell.tsx densityStyles）:
| density | padding-block | padding-inline | 本文 font | 行高（1 行, 罫線除く） |
|---|---|---|---|---|
| compact | 4px | 8px | `--text-body-size` 14px | 29px（21+8） |
| balanced | 8px | 12px | 14px | 37px（21+16） |
| spacious | 12px | 16px | 14px | 45px（21+24） |
truncate モードの既定セルは `<Text type=body maxLines=1>`（14px/20px）なので行高は 28 / 36 / 44px。
- 容器（Card/Section/Layout）内: scroll wrapper が容器 padding 分だけ左右（と先頭/末尾なら上下）に負 margin で広がり（Table.tsx `containerBleed`）、最初/最後の列の外側 padding は `max(容器padding, 8px)`（容器が無い時は density の値）（table.stylex.ts `containerEdgeStyles`）。→ 罫線と hover 背景が Card 端まで届き、文字は Card の内容線に揃う。Neutral Card padding 12px。

ヘッダセル（TableHeaderCell.tsx）: `font-weight --font-weight-semibold 600; color --color-text-secondary`（`#474747` / `#9e9e9e`）; `font-size --text-label-size 14px`; `text-align: start`（列 align で上書き）; 常に `border-bottom: 1px solid --color-border`（`#00000014` / `#FFFFFF1A`）; 常に 1 行省略（`overflow hidden; text-overflow ellipsis; white-space nowrap; max-width 0`）; 見出し文字列は `title` 属性（hover で全文）。label と after スロット（sort icon 等）は `inline-flex; gap 4px`。背景なし（header 行は hover/stripe しない）。

本文セル（TableCell.tsx）:
- wrap（既定）: `overflow hidden; overflow-wrap/word-break: break-word; max-width 0` → 行が伸びる。
- truncate: 1 行 ellipsis、省略されている時だけ tooltip（Text の smart tooltip）。
- vertical-align: middle（既定）/ top / bottom。
- dividers:
  - rows（既定）/ grid: 各セル `border-bottom 1px solid --color-border`、**最後の本文行は無し**（`tableRowMarker` の :last-child）。
  - columns / grid: `border-inline-end 1px solid --color-border`、最後の列は無し（header も同じ）。
  - none: 本文の罫線なし（header 下の 1px は常にある）。
- stripe（isStriped）: 偶数行（展開パネル行を数えない: `:nth-child(even of :not([data-expansion-panel]))`）に `--color-background-muted`（`#f1f1f1` / `#1b1b1b`）。
- hover（hasHover）: hover 可能 pointer のみ、行背景 `--color-overlay-hover`（`#0000000D` / `#FFFFFF0D`）、`transition background-color 125ms (--duration-fast) --ease-standard`。striped+hover は hover が勝つ。disabled 行は hover しない。
- selected（selection plugin）: 行背景 `--color-accent-muted`（`#f1f1f1` / `#262626`）+ aria-selected。7.4 参照。
- 行の押下・focus の見た目は表自体には無い（内側 control の focus ring のみ）。
- 列幅（columnUtils.ts）: pixel = `width = min-width = n px`; proportional = `width: (n/Σn)%` + `min-width: 下限`（proportional() の既定 120px、width 省略時 60px）; 表の min-width = pixel 合計 + 比率列の下限から求めた必要幅。超えると scroll wrapper が横スクロール。
- 空状態: `<td colSpan=全列>` に EmptyState compact: `flex column center; gap 8px; padding 16px; title 14px (--text-label-size) weight 600 text-primary line-height 1.4118`、文言 "No data"。
- scroll wrapper: `overflow-x: auto; overscroll contain`、スクロール可能な時のみ tabindex 0（矢印キーで横スクロール、focus ring は標準）。スクロール影は base には無い（sticky columns plugin が持つ, 7.7）。

TableSelectionToolbar（Toolbar size sm, variant muted）: `display flex; align-items center; justify-content space-between; min-height 28px (--size-element-sm)`、背景 `--color-background-muted`（`#f1f1f1` / `#1b1b1b`）、padding-block 8px（Section の paddingBlock 2）、padding-inline は Neutral Section 既定 12px（UNRESOLVED: Toolbar の edge compensation `calc(容器padding(既定16px) - 8px)` が muted でも適用されるか要目視）、slot 内 gap 4px（`--spacing-1`）。end: "{n} selected"（14px body）+ ghost Button sm "Unselect All"。角丸は Section の既定（UNRESOLVED: docsite で muted toolbar の radius を要確認、Section には radius 指定が見当たらない）。選択 0 件で非表示（`return null`、アニメーションなし）。

```
balanced, dividers=rows, hasHover:
┌──────────────────────────────────────────────────────┐
│ Name ⇅          Role            Amount ⇅  (600 #474747) │ 37px
├──────────────────────────────────────────────────────┤ 1px --color-border
│ Alice           Admin              $250.00          │ 37px  padding 8/12
├──────────────────────────────────────────────────────┤
│ Bob             Viewer              $10.00          │ hover: #0000000D
└──────────────────────────────────────────────────────┘ (最後の行は罫線なし)
```
RTL: すべて logical（padding-inline, border-inline-end, inset-inline）で自動鏡像。sort / chevron は RTL で `scaleX(-1)`。

### 5. Motion

| 対象 | trigger | property | 値 | duration / easing | file:line |
|---|---|---|---|---|---|
| 行 hover | pointer hover | background-color | → overlay-hover | 125ms `--ease-standard` cubic-bezier(0.24,1,0.4,1) | TableRow.tsx hoverRowStyles |
| row glide（row expansion + animatedLayout 時） | 展開パネルの開閉 | 後続 `<tr>` の translateY（Web Animations）、罫線は glide 中だけ `::after` の代替線で行と一緒に動く | before→after | 共通 spring `springs.settle` {duration 0.5s, bounce 0}（臨界減衰）を `linear()` に sampling | glidingDivider.ts, motion/animatedLayout.ts, plugins/rowExpansion |
| 右クリックメニュー | contextmenu | ContextMenu 自身の開閉（menu の work order 参照） | | | tableContextMenu.tsx |
| selection toolbar | 選択 0↔1 | なし（即時に出現/消滅） | | | TableSelectionToolbar.tsx |
reduced motion: glide なし（coarse pointer では 200ms opacity cross-fade, cubic-bezier(0.42,0,0.58,1)）。hover の 125ms 色変化は維持。

### 6. Behaviour & a11y

- role: `table`（ネイティブ）、`rowgroup`（thead/tbody）、`row`、`columnheader`（th scope=col）、`cell`。**grid ではない**: セル間の矢印移動・roving は無い。Tab は行内の control を順に辿る。
- scroll region: `role=group`、名前 = 表の aria-labelledby > aria-label > "Table"（`@astryx.table.label`）。スクロール可能な時だけ tab stop（TableScroll.a11y spec）。
- `aria-rowindex`（1 起点, page offset 込み）と `aria-rowcount`（総数, 不明なら -1）は `rowIndexStart`/`rowCount` 指定時のみ。header 行には付けない。
- `aria-sort`（sort plugin）, `aria-selected`（selection plugin）, `aria-level`/`aria-expanded`（tree plugin）を行/見出しに付ける。
- 右クリック（contextmenu, Shift+F10, Menu キー）: header セル / 行ごとに plugin の action を group 単位（区切り線入り）でまとめた ContextMenu。checked action は ✓ icon。lazily 生成（開く時に action getter を評価）。メニューは focus を奪わず、矢印キーで入る。セルの padding も含めて右クリック対象（trigger wrapper がセル全体を埋め padding を持つ）。
- useGridFocus（`hooks/useGridFocus.ts`、Table は不使用・Calendar 用）: options `columns`（行の列数）, `cellSelector`, `isCellFocusable`, `getFocusTarget`, `onNavigateBefore/After(column, offset)`（端を越えた移動の通知）, `onPageUp/Down`, `hasRovingTabIndex`; keys ←→（RTL 反転）↑↓（列数だけ移動）, Home/End（行頭/行末）, Ctrl/Cmd+Home/End（最初/最後のセル）, PageUp/Down（callback）。Kivact の `use_table` の `TableFocus::Cells` がこの意味を持つ。
- 既定文言（→ 提案 key）: "Table" `@astryx.table.label` → `soft.table.label`; "No data" `@astryx.table.noData` → `soft.table.no_data`; "Bulk actions" → `soft.table.selection.bulk_actions`; "Unselect All" → `soft.table.selection.clear_all`; "{count} selected" → `soft.table.selection.selected_count`。

### 7. Kivact 実装方針

**方針: RESTYLE `kivact::ui::data_table` + 不足機能は CORE REQUEST**（strategy §5「list・table・tree のキー操作と semantics は中核」、§6 rule 5「共通の意味は中核へ」。表の機能は kivact-ui-basic / kivact-ui-game も同じ data_table を使っており、soft 用に kit 内コピーを作らない）。children モードの静的表は kivact-ui-basic `crates/kivact-ui-basic/src/table.rs` と同じ形の **NEW kit component**（`soft::table` ほか、振る舞いなし）。

data_table の recipe（`DataTableRecipes`, crates/kivact-ui/src/recipes.rs:368-）で描くもの:
| recipe | soft での中身 |
|---|---|
| frame | 枠なし・背景なし（Astryx の表は自前の外枠を持たない。Card に置くと Card が枠） |
| toolbar | `display row; gap 8px; padding-bottom 8px`（既定 filter field は soft text_input の見た目） |
| header | 4. のヘッダセル（600, secondary, 1px 下罫線, density padding, 1 行省略）。`TableHeaderState.sortable` で cursor pointer |
| sort | sort icon 12px: 未ソート `arrowsUpDown` `--color-icon-secondary` opacity 0.35（header hover / focus で 1）、昇順 `arrowUp` / 降順 `arrowDown` `--color-icon-accent`（`#1b1b1b` / `#f1f1f1`）。label との gap 4px |
| row | stripe / hover / selected（`DataTableRowState.interaction.selected` → accent-muted）/ 下罫線、row_color は inline-start の 3px 帯（Astryx には無い。kit では row status 7.12 の dot に置換推奨） |
| cell | density padding、numeric → end 揃え、`TableCellState.active`（Cells focus 時の tab stop）は focus ring 2px accent inset（Astryx に無い状態。core の grid focus を残す場合のみ） |
| check | 選択列 36px 幅・中央、checkbox sm 20px（soft checkbox recipe） |
| footer / page | 7.3 の Pagination の見た目（下記 gap を参照） |

**標準 data_table が既に持つ Astryx 機能**: 列 title/text/cell view、単一列ソート（asc→desc）、列ごとの filter 関数 + 全体 filter 文字列、列表示切替（`DataColumn::visible`）、固定幅、end 揃え（numeric）、checkbox 複数選択 + ページ単位 select-all（tri-state）+ 件数表示、ページング（Previous/Next + Page x of y）、列リサイズ（キー・AT・pointer, Escape で戻す, min 18 / max 2048 / step 10）、行の名前、行 action 列、drag 並べ替え、toolbar/footer slot、行の details（常時 mount）、cell_controls、row_height、row_color、空表示文言、横スクロール。

**不足（CORE REQUEST として中核の data_table / interaction に追加）**:
1. `TableFocus::None`（静的 table semantics: role Table / Cell、セル roving なし、行内 control が Tab 順）— Astryx の既定挙動。現状 data_table は常に grid（Cells/Rows）。
2. density（compact/balanced/spacious）を recipe が読む状態として（`DataTableRowState`/`DataTableCellState` に density か、`Variant` の標準化）。
3. `DataTableRowState.index`（stripe 用の表示順 index）と行の hover 状態の明示（hasHover の切替）。
4. 列: 安定 id（`DataColumn::key`）、比率幅 + 下限（`flex`, `min_width`）と表の min-width 計算、`align(Start/Center/End)`、列ごとの `resizable(false)`、リッチ header view。
5. セル文字の wrap / truncate 切替と「省略時のみ tooltip」。
6. 空状態 slot `.empty(|| view)`。
7. aria-rowindex / aria-rowcount 相当（page offset 込み）。
8. セル / 見出しの右クリックメニュー（`.header_menu(|col| Vec<MenuEntry>)`, `.row_menu(|row| Vec<MenuEntry>)`、標準 `menu(..).attach_context(target)` を内部で使う）。sort などの標準 action を自動で入れる。
9. 7 章の各機能（multi sort、filter UI、grouped rows、row expansion、tree rows、sticky columns、row index、row status、column settings、selection toolbar、pagination の page 番号 / page size）。
10. row glide（展開時に後続行が spring で動く）= TreeList と共通の「keyed list の animated layout」（5 章の CORE REQUEST と同じ）。

**kit 内でよいもの**: dividers / stripe / verticalAlign の Variant（見た目のみ）、`soft::table` 静的表、TableSelectionToolbar（ただし選択数・解除は data_table の `DataTableContext.selection` から作れるので、data_table の `.toolbar(ctx)` slot に置く形で実装）。

その他:
- 容器 bleed（Card 端まで罫線を伸ばす）: List の edge と同じく、soft の card/section が `ContainerPadding` を context で配り、data_table の frame recipe が負 margin を付ける（recipe は style のみ返すので、frame recipe が theme から読める値として context→style の経路が必要。無理なら `soft::card` 側で `.bleed_child(true)`）。UNRESOLVED: recipe 関数が context を読めるか。
- line-height 21px（reset 1.5）: soft の theme text line-height を表だけ 1.5 にするか、行高を 37px に合わせる。docsite で行高を目視確認（UNRESOLVED）。

### 8. 受け入れ確認

- [ ] E2E: data_table が（TableFocus::None 実装後）`Role::Table` で名前を持ち、columnheader / row / cell の数がデータと一致。Tab が行内 control だけを順に辿る。
- [ ] E2E: sort 可能な見出しを押すと sort 状態が asc → desc（→ 未ソート）と変わり、見出しの sort semantics が更新される。
- [ ] E2E: 空データで "No data" が表示される。横幅が足りない時 scroll region が focus 可能になり、矢印で横スクロール。
- [ ] 目視: balanced 行 37px（truncate 36px）、padding 8/12、header 600 `#474747`、罫線 `#00000014`（dark `#FFFFFF1A`）、最後の本文行に罫線なし。
- [ ] 目視: dividers=grid で縦罫線、none で本文罫線なし。striped で偶数行 `#f1f1f1`（dark `#1b1b1b`）、hover `#0000000D` が 125ms で出る。
- [ ] 目視: Card 内で罫線と hover が Card 端まで届き、最初の列の文字が Card の内容線に揃う。
- [ ] 目視: selection toolbar が選択 1 件以上で muted 背景・28px 以上で出て、"2 selected" と "Unselect All" が右端。
- [ ] light/dark で docsite https://astryx.atmeta.com/components/Table と比較。

---

## 7. Table の機能（table hooks）

source: `packages/core/src/Table/plugins/<feature>/` と `packages/core/src/Table/use*.doc.mjs` ・ stories: `apps/storybook/stories/Table{Sortable,Filtering,Pagination,Selection,ColumnResize,ColumnSettings,StickyColumns,RowExpansion,GroupedRows,Tree,RowIndex,RowStatus}.stories.tsx`

共通: Astryx の hook はどれも **state を持たない plugin**（state は利用者が持ち、hook は描画と操作を足す）。Kivact では `kivact::ui::data_table` の builder method（caller-owned `Binding`）として表す。下の「Kivact」欄の判定: **RESTYLE**（既に data_table にあり recipe で見た目を合わせるだけ）／**CORE REQUEST**（data_table / interaction への追加。strategy §5・§6 rule 5 により kit で複製しない）／**kit**（見た目だけの追加）。

### 7.1 useTableSortable（+ useTableSortableState）

- 概要: 見出しを押して並べ替え。**複数列ソート**（Shift+click で 2 番目以降を追加）と「未ソートに戻る」循環を持つ。state は順序付き配列 `TableSortState = {sortKey, direction: 'ascending'|'descending'}[]`（先頭が主キー）。`useTableSortableState` は data を実際に並べ替える helper（`data, defaultSort, sort?, onSortChange?, comparators?: Record<sortKey, (a,b)=>number>, allowUnsortedState, isMultiSortEnabled` → `sortedData` 他）。
- 追加される部品: sortable 列の見出し内容を `<button>`（label + icon + 順位数字）に置換、見出しの右クリックメニューに「Sort ascending / Sort descending / Clear sort」。
- API → Kivact: `sort, onSortChange` → 既存 `.sort(Binding<Option<(usize, SortOrder)>>)` を **`.sort(Binding<Vec<(usize, SortOrder)>>)`（複数列）へ拡張**; `allowUnsortedState`（既定 true）→ `.sort_cycle(SortCycle::{AscDescNone, AscDesc})`; `isMultiSortEnabled`（既定 false）→ `.multi_sort(bool)`; 列 `sortable: true | {sortKey}` → `DataColumn::compare/sortable`。
- 見た目（useTableSortable.tsx:125-167）: button `display flex; align-items center; gap 4px; width/height 100%; padding 0; 背景なし; border-radius --radius-inner 6px; font/color inherit`; align end/center で justify。icon 12px（`size="xsm"`）: 未ソート `arrowsUpDown` 色 `--color-icon-secondary`（`#6a6a6a` / `#9e9e9e`）**opacity 0.35**、見出し hover と button focus-visible で opacity 1; 昇順 `arrowUp` / 降順 `arrowDown` 色 `--color-icon-accent`（`#1b1b1b` / `#f1f1f1`）opacity 1。複数ソート時、順位数字 `font-size 10px; line-height 1; color --color-accent` を icon の右に。focus-visible: 標準 ring 2px accent offset 3px（`focusOutlineProps.focusVisible`）。
- Motion: なし（opacity 変化も transition 指定なし → 即時）。
- Behaviour & a11y: click/Enter/Space（button）で次方向: なし→asc→desc→(なし|asc)。Shift+click（multi 有効時）: その列だけ循環し、未登録なら末尾に asc で追加、なしになったら除去。通常 click は sort 全体を置換。見出しに `aria-sort="ascending|descending"`（ソート中のみ）。複数ソート時は button の `aria-describedby` に hidden 文 "Priority {rank} of {total}"。文言: `@astryx.table.sort.ascending` "Sort ascending" → `soft.table.sort.ascending`; `.descending` "Sort descending" → `soft.table.sort.descending`; `.clear` "Clear sort" → `soft.table.sort.clear`; `.priority` "Priority {rank, number} of {total, number}" → `soft.table.sort.priority`。
- Kivact: data_table は単一列・asc→desc の 2 状態のみ、header part 全体が押下対象（`TableHeaderState{sort, sortable}`）。→ **CORE REQUEST**: 複数列ソート（順位を `TableHeaderState.sort_rank: Option<usize>` で recipe に渡す）、未ソートへの循環、Shift 追加、見出しの右クリック sort メニュー。icon の見た目は sort recipe で **RESTYLE**。
- 受け入れ: [ ] 見出し押下で asc→desc→none、aria-sort が追従。[ ] Shift+押下で 2 列目が追加され "1" "2" が表示。[ ] 未ソート icon が 0.35 で hover 時 1。

### 7.2 useTableFiltering（+ useTableFilterState）

- 概要: 列ごとのフィルタ UI。値の型は PowerSearch の field 定義（`searchConfig`）から決まる（string → TextInput、integer/float → NumberInput、enum → Selector、enum_list → MultiSelector、date_absolute → DateInput、time → TimeInput、string_list/entity_list → token 入力。nested/empty/date_relative/date_range/custom は未対応で描かない）。**plugin は data を絞らない**: 利用者が `toSearchFilters(filters)` + `applyFilters` で絞る。`useTableFilterState(initial?)` → `{filters, onFilterChange(key, value|null), clearAll}`。
- variant: `'popover'`（既定: 見出しに funnel ボタン → popover 内に control + Reset / Apply）、`'inline'`（見出し label の下に md control を常時表示）、`'inline-compact'`（同 sm control）。
- API → Kivact: `filters: Record<colKey, value>` / `onFilterChange` → `.column_filters(Binding<HashMap<usize, FilterValue>>)`; `variant` → `.filter_ui(FilterUi::{Popover, Inline, InlineCompact})`; 列 `filter: 'field' | {field, operator}` → `DataColumn::filter_field(FilterField::{Text, Number{min,max}, Choice(Vec<..>), Choices(Vec<..>), Date, Time, Tokens})`。
- 見た目（useTableFiltering.tsx:363-423, 986-1030）:
  - popover trigger: `inline-flex center; padding 0; 背景なし; border-radius --radius-element 10px`、coarse pointer で `min 44×44px`; icon `funnel` 12px、未設定 `--color-icon-secondary` opacity 0.35（th hover / focus-visible で 1）、設定済み `--color-icon-accent` opacity 1。label との間は見出しの `after` スロット gap 4px。
  - popover: placement below / alignment start、内容幅 240px、control（md）、その下 `margin-top 8px; gap 8px` に Reset（ghost sm）・spacer・Apply（primary sm）。popover 自体の見た目は Popover の work order に従う（`kivact::ui::dialog` kind Popover の soft recipe）。
  - inline: label の下 `flex column; gap 4px; margin-top 4px; min-width 0`。filter の無い列には高さ合わせの placeholder（inline 32px / compact 28px）。
- Behaviour: popover は開く時に現在値を draft にコピーし、Apply でのみ確定（`onFilterChange(key, draft)` して閉じる）、Reset は null を確定して閉じる、外側クリック/Escape は破棄。inline は入力ごとに即時確定（空文字 → null）。trigger は `aria-haspopup="dialog"`、名前 "Filter {header}"。文言: `@astryx.tableFiltering.filterByColumn` "Filter {header}" → `soft.table.filter.by_column`; `@astryx.table.filter.reset` "Reset" → `soft.table.filter.reset`; `.apply` "Apply" → `soft.table.filter.apply`; `.allPlaceholder` "All" → `soft.table.filter.all`（Selector の全件 placeholder）。
- Kivact: data_table は「列の filter 関数 + 全体で 1 つの文字列」（toolbar の text field）のみ。→ **CORE REQUEST**: 列ごとの filter 値 state（型付き FilterValue）と、その値で `use_table_model` が行を絞る経路、見出しの filter trigger part（popover / inline）。control 自体（text_input, number_input, select, tag_input, date）は既存の標準部品を soft recipe で **RESTYLE**。PowerSearch 連携は Astryx 固有なので drop（field 定義を DataColumn に直接持たせる）。
- 受け入れ: [ ] funnel を押すと 240px の popover、入力して Apply で行が絞られ funnel が accent・opacity 1 に。[ ] Reset で解除。[ ] inline で入力ごとに絞り込み。

### 7.3 useTablePagination

- 概要: 表の上/下に Pagination を描く。`page`（1 起点）, `onPageChange`, `totalItems` か `totalPages` か `hasMore`（cursor 型）, `pageSize`（既定 10）, `onPageSizeChange` + `pageSizeOptions`（page size selector）, `variant: 'pages'|'count'|'compact'|'dots'|'none'`（既定 pages）, `size: 'sm'|'md'`（既定 md）, `position: 'below'|'above'|'both'|'none'`（既定 below）, `align: 'start'|'center'|'end'`（既定 center）, `label`（既定 "Table pagination"）。**data は切らない**（利用者が `paginateData` 等で切る）。
- 見た目: wrapper `display flex`、below は `margin-top 8px`、above は `margin-bottom 8px`、justify は align。Pagination 本体の寸法（ghost の page ボタン、current 強調、"X–Y of Z"、dots）は Pagination コンポーネントの work order に従う（別グループ）。both 時は 2 つの nav に別名 "{label} (top)" / "{label} (bottom)"。
- 文言: `@astryx.table.pagination.label` "Table pagination" → `soft.table.pagination.label`; `.labelAbove` "{label} (top)" → `soft.table.pagination.label_above`; `.labelBelow` "{label} (bottom)" → `soft.table.pagination.label_below`。
- Kivact: data_table `.page(Binding<usize>, size)`（0 起点、Previous / Next + "Page x of y"、data_table 自身が行を切る）。→ 見た目の差: **kit**（footer slot `DataTableContext.page` から soft の pagination（pages/count/compact/dots 変種）を描けば実現できる）。page size selector と `hasMore`（総数不明）は **CORE REQUEST**（`DataTableContext` に page size の Binding と「総数不明」モード）。位置 above/both は `.toolbar(ctx)` slot に同じ pager を置けば kit で可。
- 受け入れ: [ ] 25 行 / 10 件で 3 ページ、Next で 2 ページ目、表の下中央に 8px 空けて pager。[ ] pageSizeOptions で 25 を選ぶと 1 ページに。

### 7.4 useTableSelection（+ useTableSelectionState, TableSelectionToolbar）

- 概要: 先頭に checkbox 列（key `__xds_selection`, 固定 36px, resize 不可, 内容中央）を足し、行を選ぶ。`useTableSelection(config)`: `getIsItemSelected, onSelectItem({item, isSelected}), onSelectAll({isAllSelected}), getIsAllSelected, getIsIndeterminate?, getIsItemSelectable?`（false の行は checkbox なし）, `getIsItemEnabled?`（false は disabled checkbox）, `getRowLabel?`（"Select {label}"）, `hasRowHighlight?`（既定 true: 選択行を accent wash）。`useTableSelectionState({data, idKey, selectedKeys, onSelectedKeysChange, getIsItemSelectable?, getIsItemEnabled?})` → `{selectionConfig, selectionState{selectedKeys, selectedCount, hasSelection, clearSelection}}`。
- 見た目: header に select-all checkbox（sm 20px, 全選択で checked、一部で indeterminate）、各行に checkbox sm 20px。選択行の背景 `--color-accent-muted`（`#f1f1f1` / `#262626`）を inline style で（sticky セルにも `--table-row-overlay` で伝播）。checkbox の見た目は CheckboxInput の work order。
- Behaviour: checkbox click / Space で `onSelectItem`。select-all は全選択可能行（state helper では data 全体）の on/off。行に `aria-selected="true|false"`（highlight を切っても出す）。行クリックでの選択は無い（checkbox のみ）。文言: `@astryx.table.selection.selectAllRows` "Select all rows" → `soft.table.selection.select_all`; `.selectRow` "Select row" → `soft.table.selection.select_row`; `.selectRowNamed` "Select {label}" → `soft.table.selection.select_named`; Toolbar の "{count, number} selected" / "Unselect All" / "Bulk actions"（6 章）。
- Kivact: data_table `.selection(Binding<Vec<K>>)` が checkbox 列・ページ単位 select-all（tri-state）・"n of m row(s) selected." を持つ → 基本は **RESTYLE**（check recipe: 36px 列・中央, row recipe: selected → accent-muted）。差分:
  - select-all の範囲: Astryx は全データ（state helper）、data_table はページ内。意味の差として記録（soft は data_table に従う）。
  - 行ごとの「選択不可（checkbox なし）」「disabled checkbox」: **CORE REQUEST** `.selectable(|row| bool)` / `.selection_enabled(|row| bool)`。
  - `hasRowHighlight=false`: recipe で selected を描かない Variant（**kit**）。
  - footer の件数文言は soft では表示しない（Astryx は TableSelectionToolbar で "{n} selected"）→ footer slot を差し替え（**kit**）。
  - TableSelectionToolbar: `.toolbar(ctx)` slot に `soft::table_selection_toolbar(ctx.selection)` を置く（**kit**）。
- 受け入れ: [ ] header checkbox で全行選択、1 行外すと indeterminate。[ ] 選択行が `#f1f1f1`（dark `#262626`）。[ ] 名前 "Select Alice"。[ ] toolbar に "2 selected"、Unselect All で全解除。

### 7.5 useTableColumnResize

- 概要: 見出しの末端の境界を drag / キーで動かして列幅を変える。`columnWidths?: Record<colKey, px>`（controlled）, `onColumnResizeEnd(updates)`（pointerup / キー確定時に **変化した全列** の px を返す: 比率列が layout shift しないよう他列も px に固定）, `minWidth?`（全体の下限、無ければ列の下限、columns 未指定時の fallback 50px）, `maxWidth?`（既定 ∞）, `columns?`（比率列の検出: 比率列の handle は隣の列を調整、**最後の比率列には handle なし**）。列 `resizable: false` で handle なし。
- 見た目（useTableColumnResize.tsx:212-331）: handle は見出しセル内 `position absolute; inset-inline-end 0; top 0; width 8px; height 表の高さ分（--table-resize-height）; cursor ew-resize; z-index 1; touch-action none`。線は `::after` で inline-end 端に `width var(--indicator-width, 1px)`、色 `--indicator-color`:
  - 休止: transparent。header 行 hover 中（hover 可能 pointer）: **全境界** が `--color-border`（`#00000014` / `#FFFFFF1A`）で薄く見える。
  - handle 自身の hover: `--color-accent`（`#1b1b1b` / `#f1f1f1`）。focus-visible: accent。drag 中: 幅 2px・accent。
  - 横スクロール中（scroll gate）は hover の表示を抑止、停止後 120ms（`SCROLL_SETTLE_MS`）で復帰。
  - coarse pointer: `pointer-events none`（touch では drag 不可、keyboard と AT は残る）。
- Motion: 線の `background-color 150ms ease, width 150ms ease`（CSS `ease` = cubic-bezier(0.25,0.1,0.25,1)）。
- Behaviour & a11y: `role="separator" aria-orientation="vertical" aria-valuenow=現在幅 aria-valuemin=min aria-valuemax=max(∞なら無し)`、名前 "Resize column {label}"（`@astryx.table.columnResize.resizeColumn` → `soft.table.column_resize`）。キー: ←/→ ±10px（Shift で ±50px、RTL で反転）、Home = 最小幅、End = 最大幅（有限時）。drag 中は表の user-select を無効化。
- Kivact: data_table `.resizable(Binding<Vec<f32>>)`（min 18 / max 2048 / step 10、Escape で drag 前に戻す、`data_table.resize` "Resize {title}"）→ **RESTYLE**（handle 幅 6px → 8px、線の色の 3 状態は splitter の state style で）。差分 **CORE REQUEST**: Shift の大ステップ 50px、Home/End、列ごとの resizable、min/max の設定、header 行 hover で全境界を表示するための「header 行 hover」状態を handle recipe に渡す、比率列の隣列調整。
- 受け入れ: [ ] header に pointer を乗せると全境界に薄線、handle 上で accent。[ ] drag で幅が変わり on_end が 1 回。[ ] focus して → で +10、Shift+→ で +50、Home で最小。

### 7.6 useTableColumnSettings（+ useTableColumnSettingsState）

- 概要: **列の表示/順序** を `activeColumnKeys`（表示順）で決める plugin（`transformColumns` で filter + 並べ替え）。UI は持たない（story では Toolbar 内の MultiSelector を列選択に使う）。state helper: `{columns: ColumnSettingsOption[] (key, label, isAlwaysVisible?, group?), activeColumnKeys, onChangeActiveColumnKeys, defaultActiveColumnKeys?}` → `{columnSettingsConfig, activeColumnKeys, toggleColumn, isColumnActive, isColumnToggleable, showAllColumns, resetToDefault}` と MultiSelector 用 options。
- 見た目: 自前なし。列選択 UI は soft の multi select（`kivact::ui::select` 複数 / menu の checkbox 項目）で作る。
- Kivact: `DataColumn::visible(value)` で表示切替は可能（**RESTYLE 不要**）。**順序の変更**（列の並べ替え）と安定した列 id は **CORE REQUEST**（`.column_order(Binding<Vec<ColumnId>>)`）。列選択メニューの部品（"Columns" ボタン + checkbox メニュー + Reset）は他 kit でも同じ意味になるので、data_table の toolbar 既定部品候補として **CORE REQUEST**（優先度低）。
- 受け入れ: [ ] メニューで列を外すと消え、isAlwaysVisible の列は外せない（disabled）。[ ] Reset で既定列に戻る。

### 7.7 useTableStickyColumns

- 概要: 先頭/末尾の連続した列を横スクロール中も固定（`startKeys`, `endKeys`）。
- 見た目（useTableStickyColumns.tsx:204-328）: 固定セル `position sticky`（start は inset-inline-start = 累積幅、end は inset-inline-end）、背景 `var(--table-sticky-background, --color-background-card)`（`#ffffff` / `#1b1b1b`）を不透明に塗り、その上に行の overlay（stripe / hover / selected の色）を `::before` で再現（`transition background-color 125ms ease-standard`）。header 固定セル z-index 3、本文 z-index 1。固定セル自身の下罫線を再描画（行の罫線を継承）。
  - スクロール影: start 側の固定列の inline-end 外側に幅 6px の `linear-gradient(to right, --color-shadow, transparent)`（`#0000001A` / `#0000004D`）、end 側は逆向き。**スクロールで固定列の下に内容が潜っている時だけ** opacity 1（それ以外 0）、`transition opacity 150ms ease`。RTL で向き反転。
- Kivact: data_table は grid 全体を横 scroll するだけで固定列なし。→ **CORE REQUEST**: 列の pin（start/end）。native layout では「スクロール容器の中で特定の列だけスクロールを打ち消す」sticky 配置の口が必要（無ければ、固定列だけ別の非スクロール列として並べ、行高を同期する二重描画）。スクロール影の opacity は scroll offset（`node_ref().scroll_offset()`）から計算可能（**kit** で描ける）。
- 受け入れ: [ ] 横スクロールで Name 列が残り、潜り始めた瞬間に 6px の影が 150ms で出る。[ ] hover / 選択時も固定セルの背景が行と同じ色。

### 7.8 useTableRowExpansion

- 概要: 行の下に全幅の詳細パネルを開く。`expandedKeys: Set`, `onToggle(key)`, `getRowKey`, `renderExpanded(item)`, `getIsItemExpandable?`, `panelVariant: 'transparent'|'muted'`（既定 transparent）, `hasRowClickExpansion?`（行のどこを押しても開閉、pointer のみ、既定 false）, `animatedLayout?`（開閉で後続行を glide）。
- 追加部品: 先頭に chevron 列（固定 40px, resize 不可, 見出しは visually hidden "Row expansion"）、行ごとに chevron button、展開時に `<tr data-expansion-panel>`（`<td colSpan=全列>`）。行の右クリックメニューに "Expand row / Collapse row"。
- 見た目（useTableRowExpansion.tsx:156-340）: chevron button `24×24px（min(--spacing-6, 40px)）; radius --radius-inner 6px; 背景なし; color --color-icon-secondary`、hover（hover 可能 pointer）で `background-image: overlay-hover` + `color --color-icon-primary`（`#000000` / `#ffffff`）、`transition color 125ms ease-standard`。icon `chevronRight` 12px、展開で `rotate(90deg)`（`transition transform 125ms ease-standard`、RTL は scaleX(-1)）。column 内で中央（margin で density padding を相殺）。パネル cell: `padding-block 16px; padding-inline-end 20px; padding-inline-start = 40px + density の inline padding`（8/12/16）。muted variant で背景 `--color-background-muted`。展開中は行の下罫線を消し、パネルの下に罫線（行 + パネルで 1 組）。stripe はパネルを数えない。
- Motion: chevron 回転 125ms。animatedLayout 時: 後続行が spring `{duration 0.5s, bounce 0}` で glide、パネルは同じ motion で entrance（罫線は glidingDivider で行と一緒に動く）。reduced: glide なし。
- a11y: chevron `aria-expanded`、名前 "Expand row" / "Collapse row"。文言 key: `@astryx.tableRowExpansion.expandRow` → `soft.table.row_expansion.expand`; `.collapseRow` → `soft.table.row_expansion.collapse`; `.columnHeader` "Row expansion" → `soft.table.row_expansion.column_header`; `.expandAllRows` / `.collapseAllRows` "Expand all rows" / "Collapse all rows" → `soft.table.row_expansion.expand_all` / `collapse_all`。
- Kivact: data_table `.row_details(slot)` は「常時 mount の行下 view、表示は利用者が制御」で chevron 列・展開 state・aria-expanded・panel の罫線処理が無い。→ **CORE REQUEST**: `.row_expansion(expanded: Binding<Vec<K>>, render)`（chevron 列 part + aria-expanded + 行クリック展開オプション + expandable 判定）、glide は 5 章の animated layout と共通。chevron と panel の見た目は recipe で **RESTYLE**。
- 受け入れ: [ ] chevron 押下でパネルが開き chevron が 90° 回転（125ms）、Enter/Space でも開閉、aria-expanded 更新。[ ] パネルの左端が本文列の文字と揃う（40 + 12px）。

### 7.9 useTableGroupedRows

- 概要: flat data を `groupBy(item)` で節に分け、各節に全幅の見出し行（chevron + label + 件数）を挿入、折畳むとその節の行を隠す。`{data, groupBy, collapsedGroups: Set, onToggleGroup, renderGroupHeader?(key, count, collapsed), getRowKey?, groupOrder?}` → `{plugin, data（見出し入りの flat 列）, idKey}`。
- 見た目（useTableGroupedRows.tsx:120-215）: 見出し行 `cursor pointer; user-select none; background --color-background-muted（#f1f1f1 / #1b1b1b）; border-bottom 1px solid --color-border`。cell `padding-block 8px; padding-inline-start 0; padding-inline-end 12px`、内側 `flex; align-items center; gap 4px; padding-inline-start 4px`、既定見出しは横スクロール中も左に固定（sticky, width fit-content）。chevron button: 背景なし・padding 0、color `--color-icon-secondary` → hover `--color-icon-primary`、icon `chevronRight` **16px**、展開で rotate(90deg)（`transition transform 150ms`、easing 既定 = `ease`）。label: weight 600 text-primary、件数 "(n)" weight 400 text-secondary。
- a11y: chevron `aria-expanded`（!collapsed）、名前 "Expand group {groupKey}" / "Collapse group {groupKey}"（`@astryx.tableGroupedRows.expandGroup/collapseGroup` → `soft.table.group.expand` / `collapse`）。行全体 click でも開閉（pointer の便宜、keyboard は chevron）。
- Kivact: 無し。→ **CORE REQUEST**: `.group_by(|row| GroupKey, collapsed: Binding<Vec<GroupKey>>)` と見出し行 part（`Role::Row` + 展開 button）、group 見出しの recipe。sort/filter/page との順序（group は sort 後に作る）を use_table_model 側で定義する必要がある。
- 受け入れ: [ ] "Engineering (4)" の見出し行が muted 背景、押すと 4 行が消え chevron が 0° に（150ms）。

### 7.10 useTableTreeData（+ useTableTreeState）

- 概要: 入れ子データ（`childrenKey` の子配列）を、指定列（`treeColumnKey`、既定は先頭列）に字下げ + expander を付けて木として表示。`useTableTreeState({data, idKey, childrenKey?, defaultExpandedIds?, expandedIds?, onExpandedIdsChange?})` → `{visibleData, treeConfig, expandedIds, isAllExpanded: bool|'indeterminate', expandAll, collapseAll}`。`useTableTreeData(config)`: `getRowMeta(item) → {level, hasChildren, isExpanded}`, `onToggleItem`, `hasExpandableRows`（false なら何もしない）, `isAllExpanded/onExpandAll/onCollapseAll` + `hasExpandAllControl`（既定 false: 見出しに全展開トグル）, `indent: 'sm'|'md'|'lg'`（既定 md）, `hasRowClickExpansion?`。
- 見た目（useTableTreeData.tsx:231-310）: tree 列セルの内容 `flex; align-items center; gap 4px`、`padding-inline-start = level × 字下げ`（sm 12px / md 16px / lg 24px）。expander button `24×24px; radius 6px; 背景なし; color --color-icon-secondary → hover --color-icon-primary + overlay-hover 背景; transition color, background-color 150ms`、icon `chevronRight` 12px、展開 rotate(90deg)（`transition transform 150ms`）。leaf は 24×24 の spacer で揃える。見出しの expand-all は同じ button（全展開時 rotate 90）。
- a11y: 行に `aria-level`（1 起点）と子あり時 `aria-expanded`（ref で付与）。table role のまま（treegrid にはしない）。expander 名 "Expand row" / "Collapse row"、全体 "Expand all rows" / "Collapse all rows"（`@astryx.tableTree.*` → `soft.table.tree.expand` / `collapse` / `expand_all` / `collapse_all`）。
- Kivact: data_table は平坦な行のみ。`kivact::interaction::use_tree` は木の展開 state と key を持つが table と結合していない。→ **CORE REQUEST**: data_table の tree rows（`.tree(parent: impl Fn(&R) -> Option<K>, expanded: Binding<Vec<K>>)`、tree 列の指定、aria level/expanded、全展開）。use_tree の flatten をそのまま use_table_model に流す形が自然。expander の見た目は recipe で **RESTYLE**。
- 受け入れ: [ ] 子を持つ行に chevron、押すと子行が 16px 字下げで現れる。[ ] leaf の文字が兄弟の chevron 右と揃う。[ ] 全展開トグルが indeterminate→全展開→全折畳み。

### 7.11 useTableRowIndex

- 概要: 先頭に行番号列（key `__rowIndex`, 固定 48px, end 揃え, resize 不可, 見出し既定 "#"）。`{data, getRowKey?, label?, startFrom?（既定 1）}`。番号は `data` 内の位置（ソート後の data を渡せば表示順）。
- 見た目: `font-family --font-family-code`（ui-monospace 系）, `font-size --text-supporting-size 12px`, `tabular-nums`, `color --color-text-secondary`（`#474747` / `#9e9e9e`）。
- Kivact: 無し。→ 表示だけなら `DataColumn::new("#", ..)` で利用者が作れるが、表示順 index が row から得られない。**CORE REQUEST**（小）: cell slot に表示順 index を渡す（`DataTableRowState.index` と共通）。見た目は **kit**。
- 受け入れ: [ ] 1..n が右揃え等幅 12px グレー、ページ 2 で 11 から（startFrom 指定時）。

### 7.12 useTableRowStatus

- 概要: 先頭に状態列（key `__rowStatus`, 固定 28px, 見出しは visually hidden "Row status"）。`getStatus(item) → {color, icon?, label} | {status: 'success'|'warning'|'error', label} | null`。
- 見た目: 中央揃え。color のみ → 8×8px の円（`border-radius 50%`）。color 名の解決: accent `--color-icon-accent`（`#1b1b1b`/`#f1f1f1`）、success/green `--color-icon-green`（`#00490b`/`#90ca90`）、error/red `--color-icon-red`（`#76000c`/`#ff9890`）、warning/orange `--color-icon-orange`（`#622700`/`#ff9e55`）、yellow `--color-icon-yellow`（`#4b3900`/`#e2b623`）、blue `--color-icon-blue`（`#003978`/`#88bcff`）、gray `--color-icon-gray`（`#474747`/`#9e9e9e`）、それ以外は CSS 色そのまま。icon 指定 → 12px icon（color は Icon の名前付き色、未知色は custom）。semantic status → icon 名 = status（success / warning / error icon）と同名色。
- a11y: マーカーは `role="img" aria-label=label`、Tooltip で label を表示（hover/focus）。文言 `@astryx.table.rowStatus.columnHeader` "Row status" → `soft.table.row_status.column_header`。
- Kivact: `row_color(fn -> Option<Color>)`（行端の色帯）が最も近いが、意味（名前付き状態 + tooltip + img role）が無い。→ **CORE REQUEST**: `.row_status(|row| Option<RowStatus{tone: Tone, icon, label}>)`（`kivact::ui::Tone` を再利用）と状態列 part。見た目（dot / icon）は recipe で **RESTYLE**。`row_color` は soft では状態 dot で描く。
- 受け入れ: [ ] 状態列に 8px の色 dot、hover で tooltip "Failed"、AT 名 "Failed"。

### 7.13 機能対応表

| hook | `kivact::ui::data_table` の現状 | 不足 | 置き場所 |
|---|---|---|---|
| useTableSortable | 単一列 asc→desc（`.sort`, `DataColumn::compare/sortable`, `TableHeaderState`） | 複数列・順位表示・未ソート循環・Shift 追加・sort メニュー | CORE REQUEST（icon の見た目は RESTYLE） |
| useTableSortableState | model が並べ替える | comparator は DataColumn にあり（同等） | RESTYLE（不要） |
| useTableFiltering | 列 filter 関数 + 全体 1 文字列 + toolbar text field | 列ごとの型付き filter 値・popover/inline UI・Apply/Reset | CORE REQUEST |
| useTableFilterState | — | 上の state | CORE REQUEST（同上） |
| useTablePagination | `.page(index, size)`、Previous/Next + "Page x of y" | page 番号 UI・page size selector・hasMore・上/両方の位置 | 見た目は kit（footer/toolbar slot）、page size と hasMore は CORE REQUEST |
| useTableSelection | `.selection` checkbox 列・ページ select-all・件数 | 行ごとの選択不可/disabled、highlight 切替、全データ select-all | 主に RESTYLE、選択可否は CORE REQUEST |
| useTableSelectionState | selection Binding で同等 | — | RESTYLE |
| TableSelectionToolbar | `.toolbar(ctx)` slot で作れる | 既定部品なし | kit |
| useTableColumnResize | `.resizable(widths)`（key/AT/pointer, Escape） | 大ステップ・Home/End・列ごとの可否・min/max・header hover 表示・比率列 | RESTYLE + CORE REQUEST |
| useTableColumnSettings | `DataColumn::visible` | 列順の変更・列 id・列選択メニュー | CORE REQUEST |
| useTableStickyColumns | なし（全体横スクロール） | 列 pin・不透明背景・スクロール影 | CORE REQUEST（影は kit） |
| useTableRowExpansion | `.row_details`（常時 mount、表示は利用者） | chevron 列・展開 state・aria-expanded・panel 罫線・行クリック展開・glide | CORE REQUEST |
| useTableGroupedRows | なし | 節見出し行・折畳み | CORE REQUEST |
| useTableTreeData / useTableTreeState | なし（use_tree は別系統） | tree rows・字下げ・expander・aria-level | CORE REQUEST |
| useTableRowIndex | なし | 表示順 index の受け渡し | CORE REQUEST（小）+ kit |
| useTableRowStatus | `row_color`（色帯のみ） | 名前付き状態・tooltip・img role | CORE REQUEST + RESTYLE |
| Table 基本（6 章） | grid semantics・固定幅・numeric | static table focus・density・比率幅・align・wrap/truncate・empty slot・右クリックメニュー・aria-rowindex | CORE REQUEST |

---

## 8. Chat: レイアウトとメッセージ

docs: https://astryx.atmeta.com/components/Chat（各部品は https://astryx.atmeta.com/components/ChatLayout 等）・ source: `packages/core/src/Chat/`（ChatLayout.tsx, ChatLayoutScrollButton.tsx, ChatMessageList.tsx, ChatMessage.tsx, ChatMessageBubble.tsx, ChatMessageMetadata.tsx, ChatSystemMessage.tsx, ChatToolCalls.tsx, ChatTokenizedText.tsx, ChatContext.tsx, chatMessage.vars.stylex.ts, useChatStreamScroll.ts, useChatNewMessages.ts, __tests__/）, `hooks/useStreamingText.ts` ・ stories: `apps/storybook/stories/Chat*.stories.tsx`

全体の bestPractices（Chat.doc.mjs）: MessageList > Message > Bubble で組む。density は list で一括（compact = サイドバー、balanced = 通常、spacious = 長文）、メッセージ単位で上書き可。独立した行（LLM の tool event 等）は `gap` で行間だけ調整。同じ送信者の連続は bubble の `group`（first/middle/last）で角を締める。日付区切りは `ChatSystemMessage variant="divider"`。name は最初の bubble、metadata は最後の bubble に。独自内容は ghost bubble に包む（全幅なら `width="100%"`）。emptyState を必ず用意。AI の rich 応答は ghost bubble。してはいけない: system message に送信者の内容や長文、ChatMessage の入れ子、message list への固定高さ、同じ送信者で filled と ghost の混在。

kivact の共通前提（下の各 7. で参照）:
- 公開 API で可能なもの: `node_ref().scroll_offset()` / `scroll_to(Point, ScrollBehavior)` / `rect()` / `scroll_into_view()`（crates/kivact-core/src/view/node_ref.rs）、part の `live_region(LiveRegion::{Off, Polite, Assertive})`（view/part.rs:513）、`Style::backdrop_blur(px)`（kivact-style/src/sheet.rs:738）、`mask(linear_gradient(..))`、`line_clamp`、`text_overflow(Ellipsis)`、`kivact::interaction::use_disclosure`。
- 無いもの: `Role::Log` / `Role::Article`（Role 一覧: crates/kivact-types/src/semantics.rs）、busy（aria-busy）状態、IntersectionObserver 相当（可視判定は rect と scroll offset から計算）。

### 8.1 ChatLayout (+ ChatLayoutScrollButton)

#### 1. 概要
- 全画面チャットの外殻: メッセージは流れる本文、composer は下端の「frosted glass」dock に固定、自動追従スクロールと「最下部へ」ボタンを内蔵。layout 自身がスクロール容器（既定）か、`scrollRef` で外側の容器を使う。
- bestPractices（ChatLayout.doc.mjs）: children に ChatMessageList、`composer` に ChatComposer。emptyState を渡す。ページ側がスクロールするなら scrollRef。固定高さを付けない（flex:1 で埋める）。1 つのスクロール容器に複数の ChatLayout を置かない。

#### 2. Anatomy
```
div.root [.astryx-chat-layout] (relative, flex column, flex:1, min-height 0, overflow-y auto ※self-scroll時)
├─ div.messageArea (flex column, grow 1, shrink 0, padding-bottom 24px, margin-inline auto, max-width by density)
│    └─ children (ChatMessageList) | emptyState (center, min-height 200)
└─ div.dockContainer (sticky bottom:0 ※self-scroll / fixed ※scrollRef, z0, isolation, pointer-events none)
     ├─ scrollButton (ChatLayoutScrollButton)       … 中央, padding-bottom 12px
     ├─ div.blurLayer (absolute bottom, backdrop blur 12px, mask 上端フェード)
     └─ div.dock (relative z1, pointer-events auto, padding by density) > div.dockInner (max-width by density) > composer
```
ChatLayoutScrollButton:
```
div.wrapper (flex, justify center, padding-bottom 12px)
└─ div.container (pill: radius full, popover bg, shadow-med, height 32, overflow hidden, inert when hidden)
    └─ Button ghost md (icon chevronDown 20px, icon-only か label 付き)
```

#### 3. API
ChatLayout:
| prop | 型 / 既定 | Kivact 提案 |
|---|---|---|
| children | ReactNode | `soft::chat_layout(messages: impl IntoView)` |
| composer | ReactNode（必須） | `.composer(|| view)` |
| emptyState | ReactNode | `.empty(|| view)`（children が空かどうかは caller が `messages.is_empty()` を `.is_empty(value)` で渡す。ReactNode の空判定は native に無い） |
| scrollButton | `ReactNode｜null` / 既定ボタン | `.scroll_button(ScrollButton::{Default, Hidden, Custom(slot)})` |
| scrollRef | ref | `.external_scroll(NodeRef)`（外側の scroll を追従対象にする） |
| density | `'compact'｜'balanced'｜'spacious'` / balanced | `.density(ChatDensity)` |
| xstyle/className/style/data-testid | | `.style` / `.test_id` |

ChatLayoutScrollButton: `isVisible: bool`, `label?: string`（指定で幅が広がり "New messages" 等を表示）, `onClick` → `soft::chat_scroll_button(visible: impl IntoValue<bool>).label(impl IntoText).on_press(..)`。

#### 4. Visual spec
- root: `container-type: inline-size`（子の container query 用、native では不要）、self-scroll 時 `overflow-y: auto; overflow-x: hidden`。
- density:
| density | messageArea max-width / padding-inline | blur 高さ / mask フェード | dock padding (inline / bottom) | dockInner max-width |
|---|---|---|---|---|
| compact | 100% / 0 | 80px / 上 24px で透明→不透明 | 8px / 8px | 100% |
| balanced | 100% / 0 | 100px / 36px | 12px / 12px | 100% |
| spacious | 800px / 16px | 120px / 48px | 16px / 12px | 800px |
- messageArea: `padding-block-end 24px`（composer との間）。
- blurLayer: `backdrop-filter: blur(12px)`、`mask-image: linear-gradient(to bottom, transparent, black <フェード長>)`（上端が透明で、フェード長より下は完全に blur）、bottom 0 に貼り付き、pointer-events none。色の塗りは無い（blur のみ）。
- emptyState: `flex center; flex 1; min-height 200px`。
- ScrollButton container: `border-radius 9999px; background --color-background-popover`（`#ffffff` / `#1b1b1b`）; `box-shadow --shadow-med`（light: `0 2px 4px rgba(0,0,0,.05), 0 4px 12px rgba(0,0,0,.10)`; dark: `0 2px 4px rgba(0,0,0,.35), 0 4px 12px rgba(0,0,0,.50), inset 0 0 0 1px rgba(255,255,255,.12)`）; `height --size-element-md 32px`; collapsed `max-width 32px`（円）、label 付き `max-width 200px`。中の Button: ghost md（32px、icon `chevronDown` md 20px、文字色 text-primary）、`padding-inline 8px`、label 付きは `padding-inline-end 12px`、button の角も full。hover は ghost button の overlay-hover（`#0000000D` / `#FFFFFF0D`）、press scale 0.98。
- hidden: `opacity 0; visibility hidden; pointer-events none; max-width 32px`、`inert`。
```
            ┌──────────────┐
            │ ⌄ New messages│  ← pill 32px, popover bg + shadow-med (中央, composer の 12px 上)
            └──────────────┘
 ░░░░░░░░░░░ blur(12px), 上 36px でフェード ░░░░░░░░░░░
 ┌──────────────────────────────────────────────────┐
 │ composer (radius 28)                              │  dock padding 12
 └──────────────────────────────────────────────────┘
```

#### 5. Motion
| 対象 | trigger | property | from→to | duration / easing | file:line |
|---|---|---|---|---|---|
| ScrollButton の出入り | isVisible 変化 | opacity, visibility, max-width | 0→1 / 32px→200px（label 時） | `--duration-fast-max` 165ms, `--ease-standard` cubic-bezier(0.24,1,0.4,1)。reduced motion: 0s | ChatLayoutScrollButton.tsx styles.container |
| label の伸縮 | label 有無 | max-width 32 ⇄ 200px | 同上 | 同上 | 同上 |
| 追従スクロール（stream） | content の高さ変化（ResizeObserver）で locked 時 | scrollTop | 現在→最下部 | rAF の離散 spring: 60fps 1 frame あたり `v = (0.7·v + 0.05·diff) / 1.25; pos += v·(dt/16.67ms)`、`|diff|<0.5 && |v|<0.1` で終了（damping 0.7, stiffness 0.05, mass 1.25） | useChatStreamScroll.ts:158-235 |
| 最初の fill | 初回 content | scrollTop | 即時（instant） | — | useChatStreamScroll.ts |
| scrollToBottom（ボタン） | press | 同じ spring（`behavior:'instant'` で即時） | | | |
reduced motion: 追従は常に即時ジャンプ（spring なし）、ボタンの transition 0s。

#### 6. Behaviour & a11y
- 追従（useChatStreamScroll）: 既定 locked。**上へのスクロール（wheel / touch / scrollbar / キー、どの手段でも「hook が最後に置いた位置より上に着地した scroll」）で即 unlock**。scrollend で最下部から `lockThreshold` 10px 以内なら再 lock。最下部から `buttonThreshold` 100px を超えると `isScrolledUp`（ボタン表示）。追従中は scroll anchoring を無効化（`overflow-anchor: none !important` を属性で）。`scrollToMessage(el)`（要素を上端へ, lock 変更なし）、`scrollToLastMessage`、`lock/unlock`。
- 新着（useChatNewMessages）: content の最後の `.astryx-chat-message` が変わり、かつ unlock 中なら `hasNewMessages`。ボタンは `isScrolledUp || hasNewMessages` で表示、新着時は label "New messages"。押すと新着を消して spring で最下部。
- ScrollButton の名前: label か "Scroll to bottom"。非表示時は inert（Tab で到達しない）。
- 文言: `@astryx.chatLayout.newMessages` "New messages" → `soft.chat.layout.new_messages`; `@astryx.chatLayoutScrollButton.scrollToBottom` "Scroll to bottom" → `soft.chat.layout.scroll_to_bottom`。

#### 7. Kivact 実装方針
- **NEW kit component**（`soft::chat_layout`, `soft::chat_scroll_button`）。最も近い analog: `crates/kivact-ui-basic/src/message_scroller.rs`（追従 + "Scroll to end" ボタン）。
- 追従スクロール: message_scroller と同じく `node_ref().scroll_offset()` / `rect()` / `scroll_to(.., ScrollBehavior::Instant)` で実装可能。spring は kit 側で毎フレーム `scroll_to` を呼ぶ（kivact のフレーム callback / animation tick を使う）。差分: basic は「最下部 1.5px 以内なら follow」の単純判定、Astryx は「hook が置いた位置より上 = ユーザーの意図」判定・10px 再 lock・100px ボタン閾値・新着検知・spring。
- **CORE REQUEST**（strategy §6 rule 5）: 「末尾に追従するスクロール（stick-to-bottom）」は kivact-ui-basic `message_scroller` と soft ChatLayout の 2 つの公式 kit で同じ意味になる（ログ表示・ターミナル・チャットで汎用）。`kivact::interaction::use_follow_end(viewport: NodeRef, content: NodeRef) -> FollowEnd{locked: Value<bool>, away: Value<bool>, has_new: Value<bool>, scroll_to_end(behavior), unlock()}`（閾値 lock 10px / away 100px を option、ユーザー操作の判定、reduced motion で即時、spring は core の motion で）として中核に入れ、basic の message_scroller と soft の chat_layout はその restyle / 組み合わせにする。
- frosted glass: `Style::backdrop_blur(12)` + `mask(linear_gradient(180).stop(TRANSPARENT, 0).stop(BLACK, 36))` で表現可能（`Gradient::stop(color, at: Length)` は px を受ける: crates/kivact-style/src/paint/gradient.rs:124）。backdrop blur が未対応のバックエンドでは fallback: `--color-background-body` を alpha 0.85 で塗る同じ形のグラデーション。
- dock の sticky（self-scroll 時）: scroll 容器内の sticky 配置が無ければ、layout を `column(scroll(messages).grow(1), dock)` の 2 段にし、dock を scroll 外に置く（blur layer は scroll の上に absolute で重ねる）。見た目は同じ。
- emptyState の判定は caller が値で渡す（ReactNode の空判定は無い）。

#### 8. 受け入れ確認
- [ ] E2E: メッセージ追加で最下部が見え続ける（locked）。上へスクロールすると追従が止まり、100px 以上離れると "Scroll to bottom" ボタンが出る（名前で検出）。
- [ ] E2E: 離れている間に新しいメッセージが来るとボタン名が "New messages" になり、押すと最下部へ移動して消える。ボタン非表示中は Tab で到達しない。
- [ ] 目視: ボタンが 32px の円 → label 時 200px まで 165ms で伸びる pill、popover 背景 + shadow-med（dark は 1px の白 inset）。
- [ ] 目視: composer の上 100px に blur の帯、上端 36px で消える。spacious で本文と composer が 800px 幅中央。
- [ ] reduced motion: 追従・ボタンが即時。

### 8.2 ChatMessageList

#### 1. 概要
- メッセージの縦並び。density を子へ配る。短い会話を下寄せ（`align='bottom'` の spacer）。上端到達で過去ログ読込（`scrollToTopAction`）。`role="log"` の live region。

#### 2. Anatomy
```
div role="log" aria-live="polite" aria-busy?(isStreaming) tabindex=0 [.astryx-chat-message-list] (flex column, flex 1)
└─ div.inner (flex column, gap/padding by density)  ← ChatLayout の新着監視対象
   ├─ sentinel div aria-hidden（scrollToTopAction 時）
   ├─ loading: Spinner md（読込中, 中央, padding-block 12px）
   ├─ spacer (flex 1)（align=bottom）
   └─ children | emptyState
```

#### 3. API
| prop | 型 / 既定 | Kivact 提案 |
|---|---|---|
| children | ReactNode | `soft::chat_message_list(items: impl IntoValue<Vec<T>>, key, row: impl Fn(Value<T>) -> V)`（keyed for_each で追加時に既存を再 mount しない） |
| emptyState | ReactNode | `.empty(|| view)` |
| scrollToTopAction | `() => Promise<void>` | `.on_reach_top(|| async)` → kivact では `.on_reach_top(impl Fn() + 'static)` と `.loading_top(impl IntoValue<bool>)`（非同期は app の task） |
| density | `ChatDensity` / balanced | `.density(..)` |
| gap | SpacingStep | `.gap(f32)` |
| align | `'top'｜'bottom'` / bottom | `.align(ChatListAlign::{Top, Bottom})` |
| isStreaming | bool / false | `.streaming(impl IntoValue<bool>)` |

#### 4. Visual spec
| density | gap | padding-block | padding-inline |
|---|---|---|---|
| compact | 8px | 8px | 12px |
| balanced | 16px | 16px | 16px |
| spacious | 24px | 24px | 24px |
- `gap` 指定で行間だけ上書き（padding は density のまま）。
- loading: Spinner md（diameter 14px, stroke 3px; Spinner の work order）中央、`padding-block 12px`。
- tabindex 0 のため focus-visible 時はブラウザ既定の outline（スタイル指定なし）→ kit では標準 focus ring（2px accent, offset 3px → list の内側に収めるため offset -2px 推奨、UNRESOLVED: docsite で list focus 時の見た目を要確認）。

#### 5. Motion
- 新規メッセージの入場アニメーションは **無い**（追加は即時、スクロール追従のみ）。過去ログ読込中の Spinner 回転は Spinner の motion。

#### 6. Behaviour & a11y
- `role="log"`, `aria-live="polite"`: 追加された内容が丁寧に読み上げられる。`isStreaming` の間 `aria-busy="true"` で、ストリーム完了時に 1 回だけ読み上げ。
- tab stop（tabindex 0）: キーボードでログをスクロールできる。
- 上端の sentinel が見えたら `scrollToTopAction` を transition で実行、実行中は Spinner と busy announcement（`@astryx.spinner.loading` "Loading"）。
- 文言: `soft.chat.list.loading`（"Loading", Spinner と共通 key でも可）。

#### 7. Kivact 実装方針
- **NEW kit component**。近い analog: kivact-ui-basic `message_scroller` の中身の column（gap 24）と `bubble_group` / `message_group`。
- `role=log` が kivact `Role` に無い → `Role::List`（または Group）+ `live_region(LiveRegion::Polite)` で代替。aria-busy 相当が無いので、ストリーム中は live region を `Off` にし、完了時に `Polite` に戻して最終テキストを読ませる（または core の announce API で完了時に 1 回告知）。→ **CORE REQUEST**: `Role::Log` と busy 状態（AccessKit には `Role::Log` と busy がある）。chat ログは basic / pop / soft の 3 kit が持つ意味なので rule 5 で「メッセージログ」の semantics（log role + busy + 新着告知）を中核の part にする。
- 上端到達: scroll offset.y ≤ 0（+ 数 px）で `on_reach_top` を呼ぶ（IntersectionObserver 不要）。読込後にスクロール位置を保つ（先頭に挿入された高さ分 offset を足す）必要がある — Astryx は browser の scroll anchoring 任せ。kivact では kit 側で `rect()` の差分を足す（UNRESOLVED: 挿入と同じフレームで補正できるか）。

#### 8. 受け入れ確認
- [ ] E2E: list が live region（polite）で、メッセージ追加時に新しい内容が告知される。streaming 中は告知されず完了時に 1 回。
- [ ] E2E: 先頭までスクロールすると on_reach_top が 1 回呼ばれ、Spinner が出る。
- [ ] 目視: balanced で行間 16px・内側 16px、メッセージが少ない時は下寄せ、align Top で上寄せ。

### 8.3 ChatMessage (+ ChatMessageBubble, ChatMessageMetadata)

#### 1. 概要
- ChatMessage: 送信者（`sender: 'user'|'assistant'|'system'`）ごとの配置と avatar / name / metadata の枠（`<article>`）。user は右（row-reverse）、assistant は左、system は中央。
- ChatMessageBubble: 吹き出し。`filled`（`--color-neutral` の塗り）/ `ghost`（塗りなし・padding 維持）。連続する bubble は `group` で送信者側の角を締める。name / metadata を bubble の文字列に揃えて上下に置ける。
- ChatMessageMetadata: 「timestamp · footer · status」の行。status は sending（pulse）/ sent / delivered / read / error。

#### 2. Anatomy
```
article [.astryx-chat-message] aria-label="Message from {sender}" | aria-labelledby=name
 (flex, align-items flex-start; user: row-reverse; system: center; gap by density ※avatar 時)
├─ avatarWrap (shrink 0, margin-top = --_chat-avatar-offset: 名前行があれば 20px)
└─ contentColumn (flex column, flex 1, min-width 0; align: assistant start / user end / system center + max-width 90%)
   ├─ name div#id (12px 600 secondary, margin-bottom 4px)     ← ChatMessage.name
   ├─ childrenWrap (flex column, gap by density, align by sender)
   │   ├─ [data-chat-name] nameRow (height 20, padding-inline = bubble の inline padding, margin-bottom -6px, user: text-align end)
   │   ├─ bubble div (radius 28, padding, max-width max(80%, 280px))
   │   └─ metadata row (padding-inline = bubble, margin-top -6px, user: end)
   │        └─ ChatMessageMetadata: [timestamp] · [footer] · [status icon + label]
   └─ metadata div                                            ← ChatMessage.metadata
```

#### 3. API
ChatMessage:
| prop | 型 / 既定 | Kivact 提案 |
|---|---|---|
| sender | `'user'｜'assistant'｜'system'`（必須） | `soft::chat_message(sender: ChatSender, content: impl IntoView)` |
| children | ReactNode | 第 2 引数 |
| avatar | ReactNode | `.avatar(|| view)`（system では無視） |
| name | ReactNode | `.name(impl IntoText)`（最初が raw 内容の時用） |
| metadata | ReactNode | `.metadata(|| view)` |
| density | ChatDensity（list から継承） | `.density(..)` |

ChatMessageBubble:
| prop | 型 / 既定 | Kivact 提案 |
|---|---|---|
| children | ReactNode | `soft::chat_bubble(content)` |
| variant | `'filled'｜'ghost'` / filled | `.variant(BubbleVariant::{Filled, Ghost})` |
| name | ReactNode | `.name(impl IntoText)` |
| metadata | ReactNode | `.metadata(|| view)` |
| group | `'first'｜'middle'｜'last'` | `.group(BubbleGroup::{First, Middle, Last})` |
| width | number(px)｜string | `.width(Length)`（指定で max-width 上限を外す） |

ChatMessageMetadata: `timestamp?: ReactNode`, `footer?: ReactNode`, `status?: 'sending'|'sent'|'delivered'|'read'|'error'` → `soft::chat_metadata().timestamp(impl IntoText).footer(|| v).status(MessageStatus::{Sending, Sent, Delivered, Read, Error})`。

#### 4. Visual spec
ChatMessage（ChatMessage.tsx styles）:
- avatar との gap: compact 6px / balanced 8px / spacious 12px（avatar がある時のみ）。
- avatar の上 offset: メッセージ内に bubble の name 行（`[data-chat-name]`）があれば 20px（`--spacing-5`）、無ければ 0 → avatar の上端が bubble の上端に揃う。
- name（ChatMessage.name）: `12px (--text-supporting-size) / 20px; weight 600; color --color-text-secondary`（`#474747` / `#9e9e9e`）、`margin-bottom 4px`。
- childrenWrap の gap（bubble 間）: compact 2px / balanced 4px / spacious 6px。
- system: 中央、content column max-width 90%。

ChatMessageBubble（ChatMessageBubble.tsx styles）:
- `display flex column; max-width: max(80%, 280px); border-radius --radius-chat 28px`（compact は `--radius-container` 12px）; `font --text-body 14px / 20px, --font-family-body`; break-word。
- filled（assistant / user 共通）: 背景 `--color-neutral`（`#0000000F` / `#FFFFFF1A`）、文字 `--color-text-primary`。ghost: 背景 transparent、padding-block 0（inline padding は維持）。
- padding: compact / balanced `12px 16px`、spacious `16px 20px`。
- group の角（送信者側だけ `--radius-inner` 6px に）:
| group | assistant（左） | user（右） |
|---|---|---|
| first | 左下 6px | 右下 6px |
| middle | 左上・左下 6px | 右上・右下 6px |
| last | 左上 6px | 右上 6px |
（logical: start/end なので RTL で反転）
- name 行（bubble.name）: `height 20px; flex center; padding-inline = 16px（spacious 20px）; margin-bottom -6px`（bubble に 6px 寄せる）、user は end 揃え。文字の書式は呼び出し側（Text supporting 等）。
- metadata 行: `padding-inline 16px（spacious 20px）; margin-top -6px`、user は end 揃え。
- width 指定: `width: <値>; max-width: none`。

ChatMessageMetadata（ChatMessageMetadata.tsx styles）:
- `flex; align-items center; gap 4px; margin-top 4px; 12px / 20px; weight 400; color --color-text-secondary`。user は row-reverse。
- 区切り "·" を timestamp / footer / status の間に。
- status: icon 12px（color inherit）+ label。sending `clock` + pulse、sent `check`、delivered `checkDouble`、read `checkDouble`、error `error` + 色 `--color-text-error`（`#c0262c` / `#ffaea7`）。

```
assistant (balanced):                         user:
 (A)  Navi                                          Cindy  
      ╭───────────────────────╮       ╭────────────────────╮
      │ How can I help?        │       │ Hey, how's it going? │
      ╰───────────────────────╯       ╰────────────────────╯
      2:30 PM                                 2:31 PM · ✓✓ Read
 bubble: #0000000F (dark #FFFFFF1A), radius 28, padding 12/16
```

#### 5. Motion
| 対象 | trigger | property | 値 | duration / easing | file:line |
|---|---|---|---|---|---|
| status "sending" の pulse | status = sending | opacity | 1 → 0.5 → 1（keyframes 0%,100%:1 / 50%:0.5） | 1.5s `ease-in-out`（cubic-bezier(0.42,0,0.58,1)）infinite。reduced motion: なし | ChatMessageMetadata.tsx pulseKeyframes |
- bubble / message の入場アニメーションは無い。

#### 6. Behaviour & a11y
- `<article>`、名前 = name 要素（aria-labelledby）か "Message from {sender}"（`@astryx.chatMessage.messageFrom` → `soft.chat.message.from`、sender は "user"/"assistant"/"system" の生文字列。i18n では sender 名を caller が渡す方がよい）。
- status: `title` = label、`aria-label` = "Message {status}"（status は小文字の label: `@astryx.chat.messageAriaLabel` → `soft.chat.message.status_label`）。status label: `@astryx.chat.status.sending` "Sending" / `.sent` "Sent" / `.delivered` "Delivered" / `.read` "Read" / `.failed` "Failed" → `soft.chat.status.sending|sent|delivered|read|failed`。
- bubble 自体は非対話（press なし）。

#### 7. Kivact 実装方針
- **NEW kit component**（`soft::chat_message`, `soft::chat_bubble`, `soft::chat_metadata`）。近い analog: kivact-ui-basic `crates/kivact-ui-basic/src/chat.rs`（`bubble`: variant/align/reactions/on_press、`message`: align/avatar/header/footer、`bubble_group`, `message_group`）。差分: Astryx は sender で配置が決まる（align を別に持たない）、group で角を変える、name/metadata を bubble の padding に揃える、metadata の status。
- `article` role が無い → `Role::Group` + 名前（"Message from …"）で代替（**CORE REQUEST**（小）: `Role::Article`）。
- 送信者メッセージの意味（送信者・時刻・状態・avatar を持つ 1 件）は basic `message` / pop `message`（kivact-ui-pop/src/messaging.rs）/ soft で 3 kit が持つ。rule 5 により「メッセージ 1 件の semantics（role, 名前, status の告知）」を中核の part（例 `kivact::interaction::message_part(sender, status)`）にすることを **CORE REQUEST** として記録（見た目は各 kit）。優先度は 8.1 の追従・8.2 の log より低い。
- `max(80%, 280px)`: 親幅 × 0.8 と 280 の大きい方を max_width に（親幅は `rect()` か layout の percentage + min 指定: `max_width(pct(80))` と `min(280)` の組合せが無ければ kit で計算）。
- 角ごとの radius: kivact style の `radius_top_left/top_right/bottom_right/bottom_left`（crates/kivact-style/src/sheet.rs:689-723）を使う。logical（start/end）ではなく物理角なので、RTL では kit が左右を入れ替える。

#### 8. 受け入れ確認
- [ ] E2E: 各メッセージが名前付き group（"Message from assistant" / name）で、status の名前 "Message read" が読める。
- [ ] 目視: assistant 左・user 右・system 中央。bubble 背景 `#0000000F`（dark `#FFFFFF1A`）、radius 28（compact 12）、padding 12/16。
- [ ] 目視: group first/middle/last で送信者側の角だけ 6px。name 行が bubble の文字頭に揃い 6px 寄る。avatar 上端が bubble 上端に揃う。
- [ ] 目視: "sending" が 1.5s で明滅、error が赤（`#c0262c` / `#ffaea7`）。

### 8.4 ChatSystemMessage

#### 1. 概要
- 送信者でない短い通知（会話開始、参加/退出、日付区切り）。`variant='default'`（中央の小さい文字）/ `'divider'`（両側に罫線の日付区切り = Divider の label 付き）。bestPractices: 短く自己完結、divider は時間/節の境界、default は事実の通知、icon だけで意味を伝えない、送信者の内容に使わない。

#### 2. Anatomy
```
default: div.root (flex center, gap 8, padding-block 4) > span.content (inline-flex, gap 6) > [icon span] + text
divider: div (width 100%) > Divider label=children  ─────── Today ───────
```

#### 3. API
`children: ReactNode`, `variant: 'default'|'divider'`（既定 default）, `icon?: ReactNode` → `soft::chat_system_message(text: impl IntoText).variant(SystemMessageVariant::{Default, Divider}).icon(icon("name"))`。

#### 4. Visual spec
- default: `font --font-family-body 12px（--text-supporting-size）weight 400; color --color-text-secondary`（`#474747` / `#9e9e9e`）; text-align center; `padding-block 4px`; icon と text の gap 6px（icon サイズは呼び出し側、通常 xsm 12px / sm 16px）。line-height は継承（14/20 の場合 12px 文字で 20px 程度、UNRESOLVED: 実測で要確認）。
- divider: Divider（label 付き）: 罫線 1px `--color-border`（`#00000014` / `#FFFFFF1A`）、label `padding-inline 12px; 12px; color --color-text-secondary`、罫線は label の左右に伸びる（Divider の work order 参照）。

#### 5. Motion
- なし。

#### 6. Behaviour & a11y
- live role を持たない（list の log が告知する）。divider も通常の本文。

#### 7. Kivact 実装方針
- **NEW kit component**。近い analog: kivact-ui-basic `primitives.rs` の `separator` + `typography::muted`、chat.rs の message_group。divider は soft の separator（label 付き）を使う。

#### 8. 受け入れ確認
- [ ] 目視: 中央の 12px グレー文字、divider で左右に 1px 罫線と 12px の余白。
- [ ] E2E: log の子として追加すると告知される（list 側の live region）。

### 8.5 ChatToolCalls

#### 1. 概要
- LLM の tool / function 呼出しの表示。1 件ならグループ枠なしで 1 行、複数なら折畳める要約行（折畳み時は最新の呼出し + 件数、展開時は "N tool calls"）。各行: status icon（pending/running は spinner、complete/error は icon）、name（monospace）、node pill、target、+/- 統計、duration、詳細の展開（resultDetail）。
- bestPractices（ChatToolCalls.doc.mjs）: 全呼出しに target、完了に duration、自動件数で足りなければ label、出力がある呼出しには resultDetail（CodeBlock）、ストリーム時は key 必須。status を省略しない（省略 = complete）、message 外で使わない、個々の行を独自 wrapper で包まない。

#### 2. Anatomy
```
div.root (flex column, margin-top 8px) [.astryx-chat-tool-calls]
├─ (1 件) CallRow
└─ (複数) header div role=button tabindex=0 aria-expanded aria-controls
    │  折畳み: [status icon] name target … [wrench 12px + count] [chevron ⌄]
    │  展開  : [wrench 16px] "N tool calls" ……………………… [chevron ⌃]
    └─ div#content (grid rows 0fr⇄1fr, inert when collapsed) > inner (overflow hidden) > list > CallRow*
CallRow: div[role=button? tabindex? aria-expanded? aria-controls?] (flex, gap 6, min-height 24, padding-block 2)
   ├─ status span 16×16 (spinner sm / icon 12px) + VisuallyHidden status 文
   ├─ name (code 12px 500 secondary, ellipsis, min 4ch)
   ├─ Badge neutral (node)
   ├─ target (12px secondary, ellipsis, 先に縮む)
   ├─ stats (+N green, -N red, 任意)
   ├─ duration (complete 時)
   └─ chevron ⌄ 12px (detail あり, margin-inline-start auto)
   └─ detail div#id (padding-inline-start 22px = 16 + 6, padding-bottom 8)
```

#### 3. API
`calls: ChatToolCallItem[]`（name, status: 'pending'|'running'|'complete'|'error'（既定 complete）, target?, duration?, node?, additions?, deletions?, stats?: ReactNode, errorMessage?, key?, data?, resultDetail?: ReactNode）, `label?`, `isExpanded?` / `defaultIsExpanded?`（既定 false）/ `onExpandedChange?`
→ `soft::chat_tool_calls(calls: impl IntoValue<Vec<ToolCall>>).label(..).expanded(Binding<bool>)`、`ToolCall::new(key, name).status(ToolStatus::{Pending, Running, Complete, Error}).target(..).duration(..).node(..).diff(additions, deletions).error(msg)` + 詳細 `.detail(|call| view)`（行ごとの開閉は kit 内 state か `Binding<Vec<K>>`）。

#### 4. Visual spec（ChatToolCalls.tsx:106-345）
- root `margin-top 8px`。行 `flex; align-items center; gap 6px; min-height 24px; padding-block 2px`。
- status icon 16×16 円枠（radius full）: pending = Spinner sm（diameter 10, stroke 2, shade subtle）色 `--color-text-disabled`（`#919191` / `#525252`）; running = Spinner sm 色 `--color-accent`（`#1b1b1b` / `#f1f1f1`）; complete = `success` icon 12px 色 `--color-icon-success`（= text-success `#2f7d33` / `#90ca90`）; error = `error` icon 12px 色 `--color-icon-error`（`#c0262c` / `#ffaea7`）、title = errorMessage。
- name: `--font-family-code`、12px / 20px、weight 500、`--color-text-secondary`、1 行 ellipsis、min-width 4ch、shrink 1。
- node: Badge neutral（h 20, padding 0 8, radius full, 12px 500, 背景 `--color-background-gray` `#e2e2e2` / `#303030`、文字 `--color-text-gray` `#1b1b1b` / `#d4d4d4`）。
- target: body font 12px/20px secondary、ellipsis、shrink 10（最初に縮む）。
- stats: 12px secondary、gap 4px; additions `--color-text-success`（`#2f7d33` / `#90ca90`）"+N"; deletions `--color-text-error` "-N"。
- duration: 12px secondary、nowrap。
- detail あり行: `cursor pointer; border-radius 10px; padding-inline 4px; margin-inline -4px`、hover（hover 可能 pointer）背景 `--color-overlay-hover`。focus-visible: 標準 ring（グループ内は clip されるので offset −2px = inset）。
- group header: 折畳み時 count = wrench icon 12px + 数字（12px secondary, gap 2px）; 展開時 wrench icon 16px（16×16, secondary）+ label（12px/20px weight 500 secondary）。chevron 14×14 box、icon `chevronDown` 12px secondary、展開で `rotate(180deg)`。
- group content inner: `padding-inline 4px; margin-inline -4px`（hover 背景が文字頭に揃う）。
- detail: `padding-inline-start 22px; padding-bottom 8px`。
- （pileCard 系のスタイルは定義のみで未使用 — 実装不要）

#### 5. Motion
| 対象 | trigger | property | 値 | duration / easing | file:line |
|---|---|---|---|---|---|
| group の開閉 | header 押下 | grid-template-rows | 0fr ⇄ 1fr（高さ 0 ⇄ 内容高） | `--duration-medium` 300ms, `--ease-standard` cubic-bezier(0.24,1,0.4,1)。reduced: none | ChatToolCalls.tsx groupContent |
| chevron | 開閉 | transform | rotate(0) ⇄ rotate(180deg) | `--duration-fast` 125ms ease-standard。reduced: none | chevronTransition |
| 行 detail | 行押下 | 即時に出入り（アニメーションなし） | | | CallRow |
| spinner | running/pending | Spinner の回転 | | Spinner の work order | |

#### 6. Behaviour & a11y
- header / detail 行: `role=button`, tabindex 0, Enter / Space で開閉、`aria-expanded`, `aria-controls`（detail は開いている時のみ）。折畳み中の content は `inert`。
- 各 status は VisuallyHidden で読み上げ: "Pending" / "Running" / "Complete" / "Failed"、error で errorMessage があれば "Error: {message}"。
- 文言: `@astryx.chatToolCalls.status.pending|running|complete|error` → `soft.chat.tool_calls.pending|running|complete|failed`; `@astryx.chatToolCalls.error` "Error: {message}" → `soft.chat.tool_calls.error`; `@astryx.chatToolCalls.groupLabel` "{count} tool calls" → `soft.chat.tool_calls.group_label`。

#### 7. Kivact 実装方針
- **NEW kit component**。近い analog: kivact-ui-basic `crates/kivact-ui-basic/src/trace.rs`（`trace`, `trace_collapsible`, `trace_line`）と `disclosure.rs`（collapsible）。開閉は `kivact::ui::disclosure` / `kivact::interaction::use_disclosure` で意味（Button + expanded + controls）を得る。
- grid-rows 0fr→1fr の高さアニメーション: kivact-motion で content の測定高さ（`rect()`）へ height を 300ms ease-standard で補間し、clip する。標準 `disclosure` が同じ開閉アニメーションを持つなら restyle で流用（UNRESOLVED: 標準 disclosure の開閉 motion の有無）。
- spinner: soft の spinner（Spinner の work order）。

#### 8. 受け入れ確認
- [ ] E2E: 3 件で header が button（collapsed）、Enter で expanded=true、3 行が現れる。各行の status が名前に含まれる（"Running" 等）。
- [ ] E2E: detail 付き行が button で、押すと detail が出る。
- [ ] 目視: 行 24px、name monospace 12px、+12 緑 / -3 赤、running は accent の spinner、complete は緑 ✓。
- [ ] 目視: 開閉が 300ms で高さ補間、chevron 180° 回転 125ms。reduced motion で即時。

### 8.6 ChatTokenizedText

#### 1. 概要
- 送信済みメッセージの本文中の token（`@mention` や `/command` の serialized 値）を Badge か独自表示に置き換えて表示。composer の `ChatComposerToken` と同じ定義を共有（入力時と表示時で token の位置がずれない）。

#### 2. Anatomy
```
span.root (display inline) [.astryx-chat-tokenized-text]
├─ "text …"
├─ span.token (inline-flex, align center, height 1lh, vertical-align top) > Badge | token.render()
└─ "… text"
```

#### 3. API
`children: string`（serialized 本文）, `tokens?: ChatComposerToken[]`（`{value, label, variant?, icon?}` の Badge 形か `{value, render}`）→ `soft::chat_tokenized_text(text: impl IntoValue<String>, tokens: impl IntoValue<Vec<ChatToken>>)`、`ChatToken::badge(value, label).variant(BadgeVariant).icon("name")` / `ChatToken::custom(value, |..| view)`。

#### 4. Visual spec
- token 枠: `display inline-flex; align-items center; height 1lh`（行の高さ = 20px）、`vertical-align top`（行内で上揃え、行高を増やさない）。
- Badge: h 20px, padding 0 8px, radius full, 12px weight 500, gap 4px（icon 時）。variant 色（Neutral theme の badge 上書き）: neutral/gray 背景 `#e2e2e2` / `#303030` 文字 `#1b1b1b` / `#d4d4d4`; blue `#b9d7ff` / `#253c5a` 文字 `#003978` / `#a1caff`; red `#ffc4be`/`#5b2b28` 文字 `#76000c`/`#ffaea7`; green `#bce0bb`/`#2b422b` 文字 `#00490b`/`#90ca90`; yellow `#fae19e`/`#453a1c` 文字 `#4b3900`/`#eec448`; orange `#ffc7a1`/`#503424` 文字 `#622700`/`#ffb37e`; purple `#f7d5ff`/`#4a2f51` 文字 `#5c0e6c`/`#eaacf8`; teal `#a9e2d6`/`#28413c` 文字 `#00463d`/`#90d7c8`; cyan `#9ae2f4`/`#274046` 文字 `#004351`/`#85d5e9`; pink `#ffc0d7`/`#572b3d` 文字 `#70003f`/`#ffa9ca`（Badge の work order が正）。
- 照合: 空でない token.value を literal に正規表現 OR で照合、左から最初の一致で置換（長さ順ではない — 先に列挙した token が優先）。

#### 5. Motion
- なし。

#### 6. Behaviour & a11y
- token は Badge のテキスト（label）として読まれる。serialized 値（`@u123`）は読み上げられない。

#### 7. Kivact 実装方針
- **NEW kit component**。行内に view（Badge）を混在させる inline layout が必要。kivact の text は span（`SpanStyle`）を持つが、**行内に任意の view（atomic inline box）を置く口が公開 API に無い**（text_edit / text の inline object）。→ **CORE REQUEST**: rich text の inline box（幅・高さを持つ置換要素を文字列中に置き、行折返しに参加させる）。composer の inline token（9.2）と共通。
- 暫定 fallback（見た目を保つ）: token を **span style の装飾**（背景色 + 角丸 + 左右 padding 相当の文字間）で描く。kivact の SpanStyle に背景色・角丸が無ければ、token を含む行は `row(wrap)` に text 断片と Badge を並べる flow layout（単語単位で折返す）で近似する（UNRESOLVED: 長い段落での折返し品質）。

#### 8. 受け入れ確認
- [ ] 目視: "Hi @Alice see /deploy" の @Alice が青 Badge（20px, pill）、行高が増えない。
- [ ] E2E: AT に "Alice" が読まれ、`@u123` の生値は読まれない。

### 8.7 useStreamingText

#### 1. 概要
- 断続的に届くストリーム文字列を一定速度で「流れるように」表示する hook。`useStreamingText(targetText, isStreaming, {speed})` → 表示する部分文字列。Chat の bubble / Markdown に渡して使う（Chat 部品は内部で使わない。利用者が bubble の中身に適用）。

#### 2. Anatomy
- view なし（文字列を返すだけ）。カーソル・caret・fade は **無い**。

#### 3. API
| 引数 | 型 / 既定 | 意味 | Kivact 提案 |
|---|---|---|---|
| targetText | string | 現時点で受信済みの全文 | `use_streaming_text(target: impl IntoValue<String>, streaming: impl IntoValue<bool>) -> Value<String>` |
| isStreaming | bool | false で即全文 | 同上 |
| options.speed | `'natural'｜'fast'｜'instant'` / natural | 速度 | `.speed(StreamSpeed::{Natural, Fast, Instant})` |

#### 4. Visual spec / 計算
- tick 間隔: `--duration-fast-min` を基準に natural = base/10、fast = base/20（最小 4ms、四捨五入）。Neutral（95ms）: natural **10ms**、fast **5ms**（theme 無しの fallback: 50ms / 8ms）。
- 1 tick で進む文字数（UTF-16 単位）: natural **10**、fast **4**、instant = 全部（useStreamingText.ts `CHARS_PER_TICK`）。rAF ごとに経過時間 ≥ tick なら進めるので、60fps では 1 frame に 1 tick → natural ≈ 10 文字/frame（約 600 文字/秒）、fast ≈ 4 文字/frame（約 240 文字/秒）。注: doc comment の "natural ~2 chars/frame, fast ~4 chars/frame" とコードが食い違う。**コードの値（natural 10, fast 4）を正とする**（UNRESOLVED: 意図は doc 側の可能性。docsite の demo で速度を目視確認）。
- 切り出し位置は grapheme cluster 境界に丸める（絵文字・ZWJ・国旗を割らない）。
- target が空になったら表示長 0 にリセット（新しいメッセージ）。isStreaming が false になった瞬間に全文。

#### 5. Motion
- 上記の文字送り（rAF）。reduced motion: 進行表示をせず常に全文。instant: 全文。

#### 6. Behaviour & a11y
- 読み上げは ChatMessageList の `isStreaming`（aria-busy）と組み合わせ、完了時に 1 回だけ告知させる。

#### 7. Kivact 実装方針
- **kit の hook**（`soft::use_streaming_text`）: kivact のフレーム tick（animation / task の interval）で表示長の signal を進める。grapheme 境界は kivact-text の segmentation（UNRESOLVED: 公開 API に grapheme 境界関数があるか要確認。無ければ `unicode-segmentation` 相当の判定を core に CORE REQUEST、kit は外部 crate に依存できない）。
- 汎用性: ストリーム表示はチャット以外（ログ、AI 補完）にも使え、basic にも将来必要になり得るので、2 つ目の kit が必要とした時点で `kivact::interaction` へ（記録のみ）。

#### 8. 受け入れ確認
- [ ] 1000 文字を一度に与え streaming=true で、約 100 frame（Neutral natural）で全文に達する。streaming=false で即全文。
- [ ] 絵文字 "👨‍👩‍👧" を含む文で途中表示が壊れた glyph にならない。
- [ ] reduced motion で即全文。

---

## 9. Chat: コンポーザー

source: `packages/core/src/Chat/`（ChatComposer.tsx, ChatComposerInput.tsx（ChatComposerTokenElement を含む）, ChatComposerDrawer.tsx, ChatDictationButton.tsx, ChatSendButton.tsx, ChatPastedTextToken.tsx, useTriggerMenu.tsx, useChatComposerTokens.ts, useChatPasteAsToken.ts, chatComposerSelection.ts, useChatDictation.ts, useSpeechRecognition.ts, ChatContext.tsx, __tests__/ChatComposerReturnKey.a11y.chromium.spec.ts ほか）・ stories: `apps/storybook/stories/ChatComposer*.stories.tsx`, `ChatDictation.stories.tsx`, `ChatSendButtonAudit.stories.tsx`

kivact の共通前提: `kivact::ui::text_area`（native 複数行、IME・clipboard、`forward_keys(keys)` で Enter 等を横取り可、IME 変換中はキーを IME が先に消費: crates/kivact-ui/src/text_input.rs:160-170）、`text_input.on_submit`、`kivact::ui::tag_input`（chip + 入力）、`kivact::ui::menu` / `command_palette`、`kivact::interaction::use_text_edit`、kivact-ui-basic `input_group.rs`（`input_group_textarea`）と `attachment.rs`。**行内の atomic token（編集可能テキストの中の chip）と音声認識は公開 API に無い**（下記）。

### 9.1 ChatComposer

#### 1. 概要
- チャット入力欄の外殻: 角の大きい浮いた面（`--radius-chat` 28px）に header（左: 添付・mention ボタン / 右: context 使用量等）、入力（既定 ChatComposerInput）、footer（左: 任意の action / 右: sendActions + 送信ボタン）、上の drawer（添付）、上下の status 帯（error / warning）。値は controlled / uncontrolled。Enter 送信は入力側が持つ。
- 推奨（props doc）: headerActions は icon-only `size="sm"`、footerActions / sendActions は `size="md"`（送信ボタンと同じ高さ）。drawer には ChatComposerDrawer。

#### 2. Anatomy
```
div.root [.astryx-chat-composer] (relative, isolation, flex column; --_chat-composer-radius 28, --_chat-composer-padding 12)
├─ status (statusPosition=top)          … 本体の下に 28px 潜り込む帯
├─ drawer (ChatComposerDrawer)          … 本体の上、28px 潜り込む
├─ div.body (z2, flex column, padding 12, gap 8, radius 28, popover bg, shadow/border)
│   ├─ header (flex, space-between, gap 8, min-height 28)  [headerActions | headerContext]
│   ├─ inputArea > input (ChatComposerInput 既定)
│   └─ footer (flex, space-between, gap 8, min-height 32)  [footerActions | sendActions + sendButton]
└─ status (statusPosition=bottom, 既定)  … 本体の上に 28px 潜り込む帯
```

#### 3. API
| prop | 型 / 既定 | Kivact 提案 |
|---|---|---|
| onSubmit | `(value) => void`（必須） | `soft::chat_composer(value: Binding<String>).on_submit(|text: String| ..)`（trim 済み、空は送らない、送信後に value を空に） |
| onStop | `() => void` | `.on_stop(|| ..)` |
| isStopShown | bool / false | `.stop_shown(impl IntoValue<bool>)` |
| value / onChange | controlled | 第 1 引数の `Binding<String>`（uncontrolled は caller が `signal(String::new())`） |
| placeholder | string / "Type a message…" | `.placeholder(..)` |
| isDisabled | bool / false | `.disabled(..)`（stop 表示中は footer 右だけ操作可） |
| density | `'compact'｜'balanced'｜'spacious'` / balanced | `.density(..)`（compact 以外は同じ見た目、下記） |
| elevation | `'none'｜'low'` / low | `.elevation(ComposerElevation::{None, Low})` |
| drawer | ReactNode | `.drawer(|| view)` |
| headerActions / headerContext | ReactNode | `.header_actions(|| v)` / `.header_context(|| v)` |
| input | ReactNode（既定 ChatComposerInput） | `.input(|ctx| view)`（独自入力） |
| footerActions / sendActions | ReactNode | `.footer_actions(|| v)` / `.send_actions(|| v)` |
| sendButton | ReactNode（既定 ChatSendButton） | `.send_button(|ctx| view)` |
| status | `{type: 'error'｜'warning', message?}` | `.status(impl IntoValue<Option<ComposerStatus>>)` |
| statusPosition | `'top'｜'bottom'` / bottom | `.status_position(..)` |

#### 4. Visual spec（ChatComposer.tsx:135-340）
- body: `padding 12px`（compact 8px, gap 4px）; `gap 8px`; `border-radius 28px`; 背景 `--color-background-popover`（`#ffffff` / `#1b1b1b`）; `cursor: text`（面のクリックで入力へ focus）。
- 中のボタンの角: `--_button-radius = max(--radius-element 10px, 28 − 12 = 16px)` = **16px**（compact は 28−8 = 20px）。
- elevation low（既定）: `box-shadow --shadow-low`（light `0 2px 4px rgba(0,0,0,.05), 0 4px 8px rgba(0,0,0,.10)`; dark `0 2px 4px rgba(0,0,0,.25), 0 4px 8px rgba(0,0,0,.40), inset 0 0 0 1px rgba(255,255,255,.08)`）、hover（hover 可能 pointer）と focus-within で `--shadow-med`（light `…, 0 4px 12px rgba(0,0,0,.10)`; dark `0 2px 4px rgba(0,0,0,.35), 0 4px 12px rgba(0,0,0,.50), inset 1px rgba(255,255,255,.12)`）。
- elevation none: `border 1px solid --color-border-emphasized`（`#d4d4d4` / `#525252`）、focus-within で border `--color-accent`（`#1b1b1b` / `#f1f1f1`）+ `inset 0 0 0 2px --color-accent-muted`（`#f1f1f1` / `#262626`）; hover（focus 外）で `inset 0 0 0 2px color-mix(--color-border-emphasized 30%, transparent)`; padding は 1px 減らして `11px`。
- keyboard で editor に focus した時のみ、body に標準 focus ring（2px accent, offset 3px; pointer focus では出さない）。
- header: `min-height 28px; gap 8`; 左 gap 4、右 gap 8・`12px/20px secondary`。footer: `min-height 32px; gap 8`; 左右 gap 4。
- disabled: root `opacity 0.6; pointer-events none`（stop 表示中は footer 右だけ pointer-events auto）。
- status 帯: `flex; gap 8; padding-inline 16px; 12px/20px body font`; icon `error` / `warning` 20px（md, color inherit）。bottom: `padding-top 12+28 = 40px; padding-bottom 12px; margin-top −28px`、下の 2 角 28px（本体の下に潜って上端が隠れる）。top は上下反転。error: 背景 `--color-error-muted`（`#ffc4be` / `#ff98903D`）文字 `--color-on-error-muted`（`#76000c` / `#ffaea7`）; warning: `--color-warning-muted`（`#fae19e` / `#e2b6233D`）文字 `--color-on-warning-muted`（`#4b3900` / `#eec448`）。forced-colors で 1px CanvasText outline。
```
 ╭────────────────────────────────────────────╮
 │ [📎][@]                         12k / 200k  │ header 28
 │ Type a message…                             │ input 22px/行
 │ [+]                              [🎤] [↑]  │ footer 32
 ╰────────────────────────────────────────────╯  radius 28, padding 12, shadow-low
  ⚠ Rate limited                                 ← status 帯 (error-muted)
```

#### 5. Motion
| 対象 | trigger | property | 値 | duration / easing | file:line |
|---|---|---|---|---|---|
| 浮き | hover / focus-within | box-shadow | low ⇄ med | `--duration-fast` 125ms, `--ease-standard` cubic-bezier(0.24,1,0.4,1) | ChatComposer.tsx styles.body |
| 枠（elevation none） | focus-within | border-color, inset shadow | emphasized ⇄ accent | 125ms ease-standard | 同上 |
| status 帯の出現 | status 変化 | なし（即時） | | | |
reduced motion: 125ms の影変化は維持（UNRESOLVED: reduced で 0 にする指定は無い）。

#### 6. Behaviour & a11y
- 面（body）のクリック: button / a / [role=button] / contenteditable / token 以外なら入力へ focus。
- submit: trim して空なら無視、disabled なら無視、`onSubmit(trimmed)` 後に値を空に。
- status: message の出現を live 告知（error は assertive、warning は polite、初回描画は告知しない: `useAnnounceRenderedText`）。
- 文言: `@astryx.chat.composer.placeholder` "Type a message…" → `soft.chat.composer.placeholder`。

#### 7. Kivact 実装方針
- **NEW kit component**（`soft::chat_composer`）。近い analog: kivact-ui-basic `crates/kivact-ui-basic/src/input_group.rs`（`input_group` + `input_group_textarea` + `input_group_button`）。入力は既定で 9.2 の `soft::chat_composer_input`、送信は 9.6。
- 影の切替は state style（hover / focus_within）で。focus-within の判定が style state に無ければ、入力の focus signal から recipe 状態を作る。keyboard focus のときだけ ring: 入力 part の `PartState.focus_visible`（crates/kivact-core/src/view/part.rs:84-86, host が可視の focus 表示を求める時 true）を composer body の recipe 状態に渡す。
- status の告知: part の `live_region(Assertive / Polite)` で。
- composer 自体の意味（送信できる入力欄、Stop 付き）は basic の chat にも無く、pop にも無いので kit 内（rule 5 対象外）。

#### 8. 受け入れ確認
- [ ] E2E: 文字を入れて Enter で on_submit("text") が 1 回、入力が空になる。空白だけでは送らない。disabled で送らない。
- [ ] E2E: status error を出すと assertive に告知される。
- [ ] 目視: radius 28、padding 12、popover 背景、shadow-low → hover で med（125ms）。elevation none で 1px `#d4d4d4`、focus で accent + 2px inset。
- [ ] 目視: status 帯が本体の下から 28px 潜り込んだ帯として見え、error は `#ffc4be`（dark は半透明赤）。

### 9.2 ChatComposerInput

#### 1. 概要
- contentEditable の rich 入力。Enter 送信（Shift+Enter 改行、IME 変換中は送らない）、trigger menu（`@` mention、`/` command など: SearchSource で同期/非同期検索）、行内 token（Badge chip）の挿入と serialize（token は `value` 文字列になる）、長文貼付けの token 化（既定 200 文字超）、送信履歴（↑↓）、ファイル drop/paste、最大 8 行で内部スクロール。

#### 2. Anatomy
```
div.root [.astryx-chat-composer-input] (relative, flex column, min-height 22)
├─ placeholder div aria-hidden (absolute top 0, 空の時のみ)
├─ div contenteditable tabindex=0 role="textbox" aria-multiline="true"   … trigger 中は role="combobox" aria-expanded aria-controls aria-activedescendant aria-haspopup=listbox
│    ├─ text …
│    └─ span[data-astryx-token][contenteditable=false] (inline-flex, height 1lh) > Badge | custom | PastedTextToken
└─ trigger menu popover (listbox; min-width 180, max-height 240)
```

#### 3. API
| prop | 型 / 既定 | Kivact 提案 |
|---|---|---|
| value / onChange | string（serialized） | `soft::chat_composer_input(label, value: Binding<String>)` |
| placeholder | string / "Type a message…" | `.placeholder(..)` |
| maxRows | number / 8 | `.max_rows(u32)` |
| triggers | `ChatComposerTrigger[]`: `character`, `searchSource`, `renderItem?`, `onSelect(item) → string｜token`, `deserialize?`, `emptySearchText?`（既定 "No results"）, `loadingText?`（"Searching…"）, `menuLabel?`（"Suggestions"）, `hasMultiWordQuery?`（既定 false: 空白で query 終了。true で改行か約 64 文字まで） | `.trigger(ChatTrigger::new('@', search: impl Fn(String) -> Task<Vec<Item>>).on_select(|item| Insert::Text(..) | Insert::Token(ChatToken)).item(|item| view).empty_text(..).multi_word(bool))` |
| debounceMs | number / 150 | `.debounce(Duration)` |
| hasHistory | bool / true | `.history(bool)` |
| label | string / "Message input" | 第 1 引数 |
| isDisabled | bool | `.disabled(..)` |
| onPaste | `(event, text) => bool` | `.on_paste(|text| -> bool)`（true で自前処理） |
| pasteAsToken | `UseChatPasteAsTokenReturn｜false` / 200 文字超で token | `.paste_as_token(Option<usize>)`（None で無効） |
| onFiles | `(files) => void` | `.on_files(|paths: Vec<PathBuf>| ..)`（drop / paste; kivact のファイル drop event 依存） |
| onSubmit | `(value) => void` | `.on_submit(..)` |
| onKeyDown | handler（Enter 処理前の seam） | `.on_key(|KeyEvent| Handled)` |
| enterKeyHint | 既定 'send' | touch keyboard の表示。native では **drop**（モバイル対象外） |
| handleRef | `{insertToken, expandToken, insertText, focus, getValue}` | `ChatInputHandle`（`.handle(h)`）: `insert_token(ChatToken)`, `insert_text(&str)`, `focus()`, `value()` |

#### 4. Visual spec（ChatComposerInput.tsx:252-321）
- 行高 **22px**（`LINE_HEIGHT_PX`、`--text-body-leading` ではない）、`font --text-body 14px`（iOS の coarse pointer のみ 16px 下限 → native では不要）、`--font-family-body`、文字 `--color-text-primary`、caret `--color-accent`（`#1b1b1b` / `#f1f1f1`）、`padding-inline 4px`、`white-space: pre-wrap; word-break: break-word`、`max-height = maxRows × 22 = 176px`（超えると内部縦スクロール）、min-height 22px。
- placeholder: 同じ位置・同じ 14px/22px・`padding-inline 4px`、色 `--color-text-secondary`（`#474747` / `#9e9e9e`）（ChatComposer の textarea 版の placeholder は `--color-text-disabled` だが既定入力はこちら）。
- token: `inline-flex; align-items center; height 1lh (22px); vertical-align top` に Badge（h 20, pill, 12px 500。色は 8.6 の表）。
- 長文 token（ChatPastedTextToken）: Badge neutral のラベル "{lines} lines, {chars} chars"（1 行なら "{chars} chars"）。hover / tap で HoverCard: `flex column; gap 8; max-width 480px`、本文 `--font-family-code 12px/20px text-primary pre-wrap, max-height 240px スクロール`、footer に `12px secondary` のラベルと ghost Button "Expand"（押すと token を元の文字列に展開）。
- disabled: `opacity 0.5; pointer-events none`。
- trigger menu（useTriggerMenu.tsx:118-180）: popover 面（Popover の soft recipe）内 `padding 4px; min-width 180px; max-height 240px; overflow-y auto`。item `padding 8px; radius 10px; 14px/20px text-primary`、label 1 行 ellipsis、highlight 背景 `--color-overlay-hover`（`#0000000D` / `#FFFFFF0D`）。group 見出し `padding 8px 8px 4px; 12px/20px secondary`。empty / loading `padding 12px; center; 12px secondary`。popover は caret 位置の一時 anchor span に付く（below）。

#### 5. Motion
- 入力自体のアニメーションなし。trigger menu の開閉は Popover の motion（Popover の work order）。HoverCard は HoverCard の motion。

#### 6. Behaviour & a11y
| キー | 動作 |
|---|---|
| Enter（Shift なし） | IME 変換中なら何もしない（`isImeKeyEvent`）。trigger menu が開いていて highlight があればその項目を選択。そうでなければ serialize → trim → 空でなければ履歴に積み `onSubmit`、入力を空に |
| Shift+Enter | 改行 |
| ↑ / ↓（menu 開） | highlight 移動（wrap） |
| Enter / Tab（menu 開・highlight あり） | 選択（`onSelect` の結果を text か token として挿入） |
| Escape（menu 開） | menu を閉じる（その query では再度開かない） |
| ↑（caret が先頭、hasHistory） | 前の送信を呼び出す（下書きは保存）。↓（caret 末尾）で次 / 下書きに戻る。途中位置では通常の caret 移動 |
| Backspace（token 直後 / token + 空白直後） | token を丸ごと削除 |
- trigger: `character` 入力で query 開始、debounce 150ms で検索、`hasMultiWordQuery` false なら空白で終了。読込中 "Searching…"（busy 告知）、0 件で "No results"（`emptySearchText` 未指定時。`""` で非表示）。
- focus 時は caret を末尾に置く（先頭だと ↑ が履歴呼出しになるため）。
- 貼付け: 200 文字超は pasted token 化。ファイルは `onFiles`。
- ARIA: 通常 `role=textbox aria-multiline=true`、名前 "Message input"; menu 中 `role=combobox aria-expanded=true aria-controls=listbox aria-activedescendant=項目 aria-haspopup=listbox`、menu は `role=listbox`（名前 "Suggestions"）。
- 文言: `@astryx.chat.composerInput.label` "Message input" → `soft.chat.input.label`; `@astryx.chatTriggerMenu.searching` "Searching…" → `soft.chat.trigger.searching`; `@astryx.chatTriggerMenu.suggestions` "Suggestions" → `soft.chat.trigger.suggestions`; "No results"（ハードコード）→ `soft.chat.trigger.no_results`; `@astryx.chat.pastedText.label`（ICU plural "{chars} chars" / "{lines} lines, {chars} chars"）→ `soft.chat.pasted_text.label`; `@astryx.chat.pastedText.expand` "Expand" → `soft.chat.pasted_text.expand`。

#### 7. Kivact 実装方針
- 基本は **`kivact::ui::text_area` の RESTYLE + kit wrapper**: text_area（native 複数行・IME・clipboard）に `forward_keys([Enter, ArrowUp, ArrowDown, Escape, Tab])` で Enter 送信 / 履歴 / menu 操作を実装（IME 変換中は IME がキーを先に消費するので「変換中 Enter で送らない」は満たされる）。Shift+Enter を改行に通すには、forward した Enter の handler が Shift 時に改行を挿入する（UNRESOLVED: forward_keys が修飾キーを区別して「Shift+Enter だけ native に通す」ことができるか。できなければ handler で `insert_text("\n")` 相当の API が必要 → CORE REQUEST）。
- 行高 22px・最大 8 行で内部スクロール・内容に応じた自動伸長: text_area の auto-grow（内容高に合わせ min 1 行・max N 行）が必要。無ければ **CORE REQUEST**（`text_area.rows(min..=max)`）。
- **行内 atomic token（contentEditable の中の chip）**: 公開 API の text_area / use_text_edit は文字列のみで、編集テキストの中に view を置く inline object が無い → **CORE REQUEST**（strategy §5「テキスト編集の拡張口は中核（段階的に）」: 編集テキスト中の置換要素 = 1 文字として caret 移動・削除・選択される inline box、serialize 値を持つ）。**fallback（見た目を保つ）**: token を text_area の **上の chip 行**（`kivact::ui::tag_input` の chip と同じ見た目の Badge、× で削除）に並べ、送信時に `token.value` を本文の先頭（または挿入時の caret 位置を記録した位置）に差し込む。mention の候補 menu は caret 位置ではなく入力欄の下端に出す。
- trigger menu: 入力文字列の caret 直前の trigger 文字を検出（text_area の selection / caret 位置が公開されていれば）→ `kivact::ui::menu` か `select` 風の listbox popup（`kivact::interaction::use_list` + popup）を入力欄に attach。combobox の意味（aria-activedescendant）は `kivact::interaction::combobox`（crates/kivact-interaction/src/combobox.rs）を使う。caret 座標に popup を出すには caret rect の公開が必要（無ければ入力欄に anchor する fallback）。
- 送信履歴・貼付け token 化・ファイル drop は kit 内ロジック（file drop event の公開 API の有無は UNRESOLVED）。

#### 8. 受け入れ確認
- [ ] E2E: `Role::TextBox`（名前 "Message input"）。Enter で送信、Shift+Enter で改行、日本語 IME 変換中の Enter で送信されない。
- [ ] E2E: "@" を入力すると listbox（"Suggestions"）が開き、↓ と Enter で選ぶと token（fallback では chip）が入る。Escape で閉じる。
- [ ] E2E: 空欄で ↑ を押すと直前の送信文が戻る。
- [ ] 目視: 1 行 22px、8 行で止まり内部スクロール、placeholder `#474747` / `#9e9e9e`、caret accent。
- [ ] 目視: 300 文字の貼付けが "1 lines, 300 chars" ではなく "300 chars" の neutral badge になり、hover で 480px 幅のプレビュー。

### 9.3 ChatComposerDrawer

#### 1. 概要
- composer 本体の上に重なる引出し（添付ファイル、context chip、preview）。`count` を与えると折畳める（折畳み時は Badge(count) + label の要約、展開時は上端の小さな grab handle）。

#### 2. Anatomy
```
div.root [.astryx-chat-composer-drawer] (relative z1, overflow hidden, padding 12/16/(12+28), margin-bottom −28, 上 2 角 28px)
├─ toggleRow div role=button tabindex=0 aria-expanded aria-controls aria-label="Expand/Collapse {label}" (count 指定時)
│   ├─ toggleContent (折畳み時表示): Badge neutral(count) + label
│   └─ collapseBarHandle (展開時表示): 20×2px の pill
└─ contentGrid#id (grid rows 1fr ⇄ 0fr, inert when collapsed) > content (flex wrap, gap 4)
```

#### 3. API
| prop | 型 / 既定 | Kivact 提案 |
|---|---|---|
| children | ReactNode | `soft::chat_composer_drawer(content: impl IntoView)` |
| count | number（指定時のみ折畳み可） | `.count(impl IntoValue<usize>)` |
| label | string / "Items" | `.label(..)` |
| collapsedSummary | ReactNode | `.collapsed_summary(|| view)` |
| isCollapsed / defaultIsCollapsed（false）/ onCollapsedChange | | `.collapsed(Binding<bool>)` |

#### 4. Visual spec（ChatComposerDrawer.tsx styles）
- root: `padding-inline 16px; padding-top 12px; padding-bottom 12 + 28 = 40px; margin-bottom −28px`（composer 本体の下に 28px 潜る）; 背景 = `--color-background-surface`（`#ffffff` / `#262626`）の上に `--color-background-muted`（`#f1f1f1` / `#1b1b1b`）を重ねた色（= light `#f1f1f1`, dark `#1b1b1b`）; 上 2 角 `--radius-chat` 28px; overflow hidden。
- toggleRow: `height 20px; padding-inline 16px; margin-inline −16px`（行全体が押せる）、cursor pointer。
- toggleContent（折畳み時）: `inline-flex; align center; height 20; gap 8; radius full`; Badge neutral（count, h 20）+ label `14px; line-height 20px; color --color-text-secondary → row hover で --color-text-primary`。展開時 opacity 0。
- collapseBarHandle（展開時）: 中央上端、`20×2px; radius full; 背景 --color-icon-secondary（#6a6a6a / #9e9e9e）→ hover で --color-icon-primary`。折畳み時 opacity 0。
- content: `flex; flex-wrap: wrap; gap 4px; align-items flex-start`。

#### 5. Motion
| 対象 | trigger | property | 値 | duration / delay / easing | file:line |
|---|---|---|---|---|---|
| 高さ | 開閉 | grid-template-rows | 1fr ⇄ 0fr | 300ms (`--duration-medium`) ease-standard | contentGrid |
| content の不透明度（開く） | 展開 | opacity 0→1 | | 300ms, **delay 125ms**（`--duration-fast`）, ease-standard | content |
| content の不透明度（閉じる） | 折畳み | opacity 1→0 | | 125ms, delay 0 | contentCollapsed |
| 要約 / handle の切替 | 開閉 | opacity | 0 ⇄ 1 | 125ms ease-standard | toggleContent / collapseBarHandle |
| label / handle 色 | hover | color / background-color | secondary → primary | 125ms ease-standard | collapseLabel |
reduced motion: 指定なし（UNRESOLVED: docsite で reduced motion 時の挙動確認。kit では reduced で高さ・opacity を即時にする）。

#### 6. Behaviour & a11y
- toggleRow: `role=button`、Enter/Space で開閉、`aria-expanded = !collapsed`、`aria-controls = content`、名前 "Expand {label}" / "Collapse {label}"。要約部は `aria-hidden` + inert（名前は button が持つ）。折畳み中の content は inert。
- 文言: `@astryx.chat.composerDrawer.label` "Items" → `soft.chat.drawer.label`; `@astryx.chatComposerDrawer.expand` "Expand {label}" → `soft.chat.drawer.expand`; `.collapse` "Collapse {label}" → `soft.chat.drawer.collapse`。

#### 7. Kivact 実装方針
- **NEW kit component**。開閉の意味は `kivact::interaction::use_disclosure`（Button + expanded + controls）。近い analog: kivact-ui-basic `disclosure.rs`（collapsible）+ `attachment.rs`（`attachment_group`）。
- 高さ補間は kivact-motion（content の rect 高 → 0、300ms ease-standard）+ clip、opacity は delay 付き別 tween。composer 本体との重なり（margin −28）は column の負 margin（kivact style が負の margin を許すか要確認、無ければ本体を z 上位の重ね配置）。

#### 8. 受け入れ確認
- [ ] E2E: count=3 で button "Collapse Items"（expanded）。押すと "Expand Items" になり content が inert。
- [ ] 目視: 本体の上に muted の帯が 28px 潜り込む、上端中央に 20×2 の handle。折畳みで "3 Items" の badge 要約。
- [ ] 目視: 高さ 300ms、開く時は 125ms 遅れて中身が fade in。

### 9.4 ChatComposerTokenElement

#### 1. 概要
- 入力の中に置く token 1 個の要素（`data-astryx-token`, `data-astryx-token-value`, `contenteditable=false`）。独自入力（`ChatComposer.input` に自作の editor）でも ChatComposerInput と同じ token 表示・serialize を使うための部品。

#### 2. Anatomy
`span[data-astryx-token][data-astryx-token-value=value][contenteditable=false] (inline-flex, align center, height 1lh, vertical-align top)` > Badge(label, variant, icon) | token.render()

#### 3. API
`token: ChatComposerToken`（`{value, label, variant?, icon?}` | `{value, render}`）, ref/xstyle/className/style → `soft::chat_token(token: ChatToken)`（9.2 の chip / 8.6 と同じ型）。

#### 4. Visual spec
- 9.2 / 8.6 と同じ: 行高に合わせた inline-flex 箱、Badge h 20px pill 12px 500、variant 色は 8.6 の表。

#### 5. Motion
- なし。

#### 6. Behaviour & a11y
- 編集不可の 1 塊（caret は前後にのみ置ける、Backspace で丸ごと削除は入力側の処理）。読み上げは Badge の label。

#### 7. Kivact 実装方針
- 行内 atomic token の CORE REQUEST（9.2）が入るまでは、chip 行 fallback の chip 1 個として実装（`soft::chat_token` = soft badge + 削除ボタン任意）。

#### 8. 受け入れ確認
- [ ] 目視: token が本文行の高さに収まり、行高を押し広げない。
- [ ] serialize: token を含む入力の送信文字列に `value` が入る。

### 9.5 ChatDictationButton

#### 1. 概要
- 音声入力の toggle。待機中はマイク icon、聞き取り中は icon の代わりに 5 本の周波数バー（実際のマイク入力に反応、音量が 20% を超えると accent から error 色へ混ざる）。`dictation` には `useChatDictation`（Web Speech API + AudioContext 解析）の戻り値を渡す。非対応環境では既定で非表示。

#### 2. Anatomy
```
span.wrapper (relative, inline-flex center) [.astryx-chat-dictation-button]
├─ barsContainer (absolute, flex center, aria-hidden, gap, height)   … 聞き取り中のみ
│   └─ bar × 5 (width, height 100%, radius full, scaleY(level))
└─ Button ghost icon-only (microphone icon | 聞き取り中は icon なし)
```

#### 3. API
| prop | 型 / 既定 | Kivact 提案 |
|---|---|---|
| dictation | `UseSpeechRecognitionReturn`（isSupported, isListening, isSpeaking, volume 0..1, bands: number[], rawBands, interimTranscript, start/stop/abort/toggle） | `soft::chat_dictation_button(state: impl IntoValue<DictationState{supported, listening, volume: f32, bands: Vec<f32>}>).on_toggle(|| ..)` — **認識と音声取得はアプリが持つ**（strategy §5: 音声はアプリ） |
| size | `'sm'｜'md'` / md | `.size(ControlSize::{Small, Medium})` |
| isHiddenWhenUnsupported | bool / true | `.hide_when_unsupported(bool)` |
| label | string | `.label(..)` |

useChatDictation の option（参考、kit では app 側の責務）: `lang`（既定 locale）, `continuous`（true）, `interimResults`（true）, `transformTranscript`, `onTranscript(t, isFinal)`, `onResult`, `onError`, `onStart`, `onEnd`, `hasSounds`（false）, `audioContext`, `inputRef`（認識結果を ChatComposerInput に挿入）。解析: `fftSize 256, smoothingTimeConstant 0.5`、周波数データから band 平均。

#### 4. Visual spec（ChatDictationButton.tsx）
- Button: ghost, icon-only, size md = 32×32（sm 28×28）、角は composer 内なら `--_button-radius` 16px（外では Button 既定）、icon `microphone`（md 20px / sm 16px）、色 text-primary。hover overlay-hover、press scale 0.98、disabled（非対応で表示する場合）opacity 0.5。
- バー: 5 本、md `width 2.5px; gap 2px; 高さ 18px`、sm `2px; 1.5px; 14px`; `border-radius full`; `transform-origin center`; `scaleY(0.08 + level × 0.92)`。level = `min((band / 0.2)^0.5, 1)`（小さい音を持ち上げる）。色: volume < 0.2 で `--color-accent`（`#1b1b1b` / `#f1f1f1`）、≥ 0.2 で `color-mix(accent, --color-error (#c9303a / #ff705d) を min((volume−0.2)/0.1, 1)×100%)`（volume 0.3 で完全に error 色）。
```
 idle:  [ 🎤 ]        listening:  [ ▮▮▮▮▮ ]  (中央 5 本, 高さ 18 を scaleY)
```

#### 5. Motion
| 対象 | trigger | property | 値 | duration / easing | file:line |
|---|---|---|---|---|---|
| バー | 音量（毎フレームの band 更新） | transform scaleY, background-color | 0.08..1 | 60ms `ease-out`（cubic-bezier(0,0,0.58,1)）。reduced: 0s（即時、バーは動く） | ChatDictationButton.tsx styles.bar |
| Button | hover/press | Button の 125ms transition | | | Button.tsx |

#### 6. Behaviour & a11y
- 押すと `dictation.toggle()`（開始/停止）。名前: 待機 "Start dictation" / 聞き取り中 "Stop dictation"（label で上書き）。バーは aria-hidden。非対応: 既定で描かない、`isHiddenWhenUnsupported=false` なら disabled で表示。
- 音声認識エラー "No speech was detected."（`@astryx.chat.speechRecognition.noSpeechDetected`）は onError 経由で app が表示。
- 文言: `@astryx.chatDictationButton.startDictation` → `soft.chat.dictation.start`; `.stopDictation` → `soft.chat.dictation.stop`; `soft.chat.dictation.no_speech`（"No speech was detected."、app 用に提供）。

#### 7. Kivact 実装方針
- **NEW kit component**（見た目 + toggle のみ）。音声取得・認識・FFT は **アプリの責務**（strategy §5 音声 = アプリ）。kit は `DictationState`（listening, volume, bands）を値で受け、`on_toggle` を返すだけ。認識結果の挿入は app が composer の `value` Binding に書く（または `ChatInputHandle::insert_text`）。
- バーは `Style::paint`（局所描画）で 5 本の角丸矩形を描き、scaleY を 60ms で補間（kivact の補間 API）。color-mix は kit で RGB 線形補間。
- core への要求なし（音声は非目標側）。

#### 8. 受け入れ確認
- [ ] E2E: `Role::Button` 名 "Start dictation"、押すと on_toggle、listening=true で名前 "Stop dictation"。supported=false で描かれない。
- [ ] 目視: listening でマイク icon が消え 5 本のバーが音量で伸縮（60ms）、大きな音で赤に寄る。

### 9.6 ChatSendButton

#### 1. 概要
- composer の丸い送信 / 停止 toggle。送信: primary（accent 塗り）+ `arrowUp`、送るものが無いと disabled。停止（`isStopShown`）: secondary（neutral 塗り）+ `stop`、`onStop` を呼ぶ。既定値は composer context から読む。

#### 2. Anatomy
`Button(icon-only, variant primary|secondary, size md|sm)` [.astryx-chat-send-button]、radius = `var(--_button-radius, 9999px)`。

#### 3. API
| prop | 型 / 既定 | Kivact 提案 |
|---|---|---|
| isStopShown | bool / context | `soft::chat_send_button().stop_shown(..)` |
| isDisabled | bool / `!canSend` | `.disabled(..)` |
| onSend / onStop / onClick | handlers（onClick は選択 action の後に合成） | `.on_send(..)` / `.on_stop(..)` |
| sendIcon / stopIcon | ReactNode / `arrowUp` / `stop` | `.send_icon(icon(..))` / `.stop_icon(..)` |
| size | `'sm'｜'md'` / md | `.size(..)` |

#### 4. Visual spec
- size md 32×32 / sm 28×28、icon-only（icon md 20px / sm 16px）。角: composer 内 16px（`max(10, 28−12)`）、単独では 9999px（円）。
- send（primary）: 背景 `--color-accent`（`#1b1b1b` / `#f1f1f1`）、icon `--color-on-accent`（`#ffffff` / `#111111`）。disabled: opacity 0.5、cursor default、press なし。
- stop（secondary）: 背景 `--color-neutral`（`#0000000F` / `#FFFFFF1A`）、icon `--color-text-primary`。stop 中は disabled にしない（composer disabled でも押せる）。
- hover / press: Button の overlay（hover 時 overlay-hover の重ね）、press `scale(0.98)`（mouse :active、touch は press controller）。
- focus-visible: 標準 ring 2px accent offset 3px。

#### 5. Motion
| 対象 | trigger | property | 値 | duration / easing |
|---|---|---|---|---|
| Button | hover / press / disabled | background-image, background-color, color, opacity, transform | scale 1 → 0.98 | `--duration-fast` 125ms, `--ease-standard`。reduced: 0s（Button.tsx:121-140） |
| send ⇄ stop の切替 | isStopShown | variant と icon の即時切替（morph アニメーションなし。色は上記 125ms で遷移） | | |

#### 6. Behaviour & a11y
- 名前: "Send" / "Stop"（icon-only なので label が accessible name）。send は context の `onSubmit(value)`（trim / 空チェックは composer）。
- 文言: `@astryx.chatSendButton.send` → `soft.chat.send`; `@astryx.chatSendButton.stop` → `soft.chat.stop`。

#### 7. Kivact 実装方針
- **RESTYLE `kivact::ui::button` + kit wrapper**: `button(message("soft.chat.send", "Send")).icon(icon("arrow-up")).hide_label(true).variant(ButtonVariant::Primary).size(ControlSize::Medium)`、stop は `ButtonVariant::Secondary` + `icon("square")`（stop glyph の名前は soft の icon set に合わせる）。角 16px は composer 内で Variant（`soft::ComposerButton`）を付けて button recipe が読む。
- core への要求なし。

#### 8. 受け入れ確認
- [ ] E2E: 空入力で "Send" が disabled、文字を入れると enabled、押すと on_submit。stop_shown で名前 "Stop"、押すと on_stop。
- [ ] 目視: 32px、composer 内で角 16px、send は黒/白の塗り、stop は薄いグレー、press で 0.98 に縮む（125ms）。

---

## UNRESOLVED 一覧

| # | 箇所 | 内容 | 確認方法 |
|---|---|---|---|
| U1 | グループ一覧 | hook（useTable* / useGridFocus / useListFocus / useStreamingText）と TableSelectionToolbar の docs URL は `https://astryx.atmeta.com/components/<Name>` の命名規則からの推定 | docsite のナビゲーションで実在を確認し、無ければ親（Table / Chat）のページを参照 |
| U2 | 2. Item 7 | 行の focus ring（2px, offset 3px が行の外へはみ出す）が docsite でも外側に見えるか、List の inline clip で切れないか | docsite List / Item の keyboard focus を目視 |
| U3 | 3. MetadataList 4 | Show more / less ボタンの hover / focus の見た目（スタイル定義なし） | docsite で hover と Tab focus を目視 |
| U4 | 4. OverflowList 7 | kivact で測定 → 表示数の反映が同一フレームで収束するか | 実装後に E2E で 1 フレーム目のちらつきを確認 |
| U5 | 5. TreeList 5 | animatedLayout 無しの chevron 回転（125ms）が reduced motion で 0 になるか | applyLayoutChangeTransition の reduced 分岐を追う / docsite を reduced motion で確認 |
| U6 | 6. Table 4 | 既定セルの line-height（reset の 1.5 → 行高 37px）が docsite でも 37px か | docsite Table の行高を実測 |
| U7 | 6. Table 4 | TableSelectionToolbar（Toolbar sm + Section muted）の inline padding と角丸 | docsite TableSelectionToolbar を実測 |
| U8 | 6. Table 7 | data_table の recipe 関数が容器 padding（context）を読めるか（Card bleed の実現方法） | crates/kivact-ui/src/layer.rs の recipe 呼び出し経路を確認 |
| U9 | 8.2 ChatMessageList 4 | log（tabindex 0）の focus 表示 | docsite で Tab focus を目視 |
| U10 | 8.2 ChatMessageList 7 | 過去ログを先頭に挿入した同じフレームでスクロール位置を補正できるか | 実装後の E2E |
| U11 | 8.4 ChatSystemMessage 4 | default variant の line-height（継承値） | docsite で実測 |
| U12 | 8.5 ChatToolCalls 7 | 標準 `kivact::ui::disclosure` に高さの開閉アニメーションがあるか | crates/kivact-ui/src/disclosure.rs を確認 |
| U13 | 8.6 ChatTokenizedText 7 | SpanStyle に背景・角丸が無い場合の flow layout 近似の折返し品質 | 試作で長文を確認 |
| U14 | 8.7 useStreamingText 4 | doc comment（natural ≈2 文字/frame）とコード（natural 10, fast 4 文字/tick）の食い違い。本書はコードを正とした | docsite の demo で速度を目視 |
| U15 | 8.7 useStreamingText 7 | grapheme 境界判定が kivact の公開 API にあるか | crates/kivact（text 関連）を確認、無ければ CORE REQUEST |
| U16 | 9.1 ChatComposer 5 | 影の 125ms 遷移が reduced motion で残るか（指定なし） | docsite を reduced motion で確認 |
| U17 | 9.2 ChatComposerInput 7 | `forward_keys` で Shift+Enter だけを native に通せるか / 改行挿入 API の有無、text_area の auto-grow、caret rect の公開 | crates/kivact-interaction/src/text_edit* を確認 |
| U18 | 9.2 ChatComposerInput 7 | ファイル drop / paste のイベントが公開 API にあるか | kivact-core の view event を確認 |
| U19 | 9.3 ChatComposerDrawer 5 | reduced motion 時の挙動（指定なし）。kit では即時にする | docsite を reduced motion で確認 |
