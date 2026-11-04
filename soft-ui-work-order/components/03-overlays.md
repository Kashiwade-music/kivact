# kivact-ui-soft 作業指示書 — 03 Overlays & Menus

対象: Astryx（`D:\w\meta\astryx`, `packages/core/src/<Dir>`）のオーバーレイ／メニュー系コンポーネントを、Kivact の `crates/kivact-ui-soft` として NEUTRAL テーマの見た目・動きどおりに再実装するための仕様。値は neutral テーマ（`packages/themes/neutral/src/neutralTheme.ts` + `neutralPaletteRefs.generated.ts`、未上書きトークンは `packages/core/src/theme/tokens.stylex.ts`）で解決済み。file:line は `packages/core/src/` からの相対パス。

## 目次

- 0. 共通: 解決済みトークン早見表 / 共通オーバーレイ規則
- 1. Dialog（1.p1 DialogHeader, 1.p2 useImperativeDialog）
- 2. AlertDialog（2.p1 useImperativeAlertDialog）
- 3. Drawer（3.p1 DrawerHeader）
- 4. BottomSheet
- 5. BottomSheetSwitcher
- 6. Popover（usePopover）
- 7. HoverCard（useHoverCard）
- 8. Tooltip（useTooltip）
- 9. Toast（useToast, ToastViewport）
- 10. CommandPalette（Input, List, Item, Group, Footer, Empty）
- 11. DropdownMenu（Item, CheckboxItem, RadioGroup, RadioItem, Divider, Group, SubMenu）
- 12. ContextMenu（ContextMenuItem）
- 13. Lightbox（useLightbox）
- 14. Overlay（OverlayScrim, useOverlay）
- 15. Layer / useLayer（配置・フリップ・シフト・ビューポート余白・dismiss スタック）
- 16. Hooks: useFocusTrap / useScrollLock / useAnnounce

## グループ概要

| Astryx component (and parts) | docs URL | source dir | kivact strategy | closest kivact-ui-basic analog |
|---|---|---|---|---|
| Dialog | https://astryx.atmeta.com/components/Dialog | `Dialog/` | RESTYLE `kivact::ui::dialog`（`DialogKind::Modal`）+ kit wrapper（purpose / fullscreen / position / max_height） | `recipes_dialog.rs`（dialog.tsx）, `layers.rs` |
| DialogHeader | https://astryx.atmeta.com/components/DialogHeader | `Dialog/` | RESTYLE（DialogRecipes header/title/description/close）+ wrapper の start/end スロット | `recipes_dialog.rs` |
| useImperativeDialog | https://astryx.atmeta.com/components/useImperativeDialog | `Dialog/` | NEW kit helper（Copy ハンドル: show/hide/is_open/view） | なし |
| AlertDialog | https://astryx.atmeta.com/components/AlertDialog | `AlertDialog/` | RESTYLE `kivact::ui::dialog`（`DialogKind::AlertModal`）+ kit wrapper `soft::alert_dialog` | `alert_dialog.rs`, `recipes_dialog.rs` |
| useImperativeAlertDialog | https://astryx.atmeta.com/components/useImperativeAlertDialog | `AlertDialog/` | NEW kit helper | なし |
| Drawer | https://astryx.atmeta.com/components/Drawer | `Drawer/` | RESTYLE `kivact::ui::dialog`（`DialogKind::Drawer(side)`）+ kit wrapper（非モーダル・RTL side） | `recipes_dialog.rs`（sheet.tsx）, `drawer.rs`（※スワイプ版、仕様違い） |
| DrawerHeader | https://astryx.atmeta.com/components/DrawerHeader | `Drawer/` | RESTYLE（DialogHeader と同寸）+ wrapper | `recipes_dialog.rs` |
| BottomSheet | https://astryx.atmeta.com/components/BottomSheet | `BottomSheet/` | NEW kit component（use_dialog + use_drag、detent・stack・replace） | `drawer.rs`（距離+速度で閉じるスワイプ） |
| BottomSheetSwitcher | https://astryx.atmeta.com/components/BottomSheetSwitcher | `BottomSheet/` | covered by BottomSheet（非推奨。replace ハンドオフで代替） | なし |
| Popover / usePopover | https://astryx.atmeta.com/components/Popover | `Popover/` | RESTYLE `kivact::ui::dialog`（`DialogKind::Popover`）+ kit wrapper（presentation, fallback close） | `recipes_dialog.rs`（popover.tsx）, `recipes_popup.rs` |
| HoverCard / useHoverCard | https://astryx.atmeta.com/components/HoverCard | `HoverCard/` | NEW kit component（use_delayed + overlay） | `hover_card.rs` |
| Tooltip / useTooltip | https://astryx.atmeta.com/components/Tooltip | `Tooltip/` | RESTYLE `kivact::ui::tooltip`（+ TooltipConfig 200ms/skip 0 を provide） | `recipes_popup.rs`（tooltip.tsx） |
| Toast / useToast (+ToastViewport) | https://astryx.atmeta.com/components/Toast | `Toast/` | RESTYLE `kivact::ui::toast_region` + kit wrapper（uniqueID, swipe, リスト型スタック） | `recipes_controls.rs`（sonner の toast。見た目は流用不可） |
| CommandPalette (+Empty, Footer, Group, Input, Item, List) | https://astryx.atmeta.com/components/CommandPalette | `CommandPalette/` | RESTYLE `kivact::ui::command_palette` | `recipes_command.rs` |
| DropdownMenu (+CheckboxItem, Divider, Group, Item, RadioGroup, RadioItem, SubMenu) | https://astryx.atmeta.com/components/DropdownMenu | `DropdownMenu/` | RESTYLE `kivact::ui::menu`（`.attach`）+ kit wrapper（header/footer, sheet presentation） | `recipes_menu.rs`, `list_popup.rs` |
| ContextMenu / ContextMenuItem | https://astryx.atmeta.com/components/ContextMenu | `ContextMenu/` | RESTYLE `kivact::ui::menu(..).attach_context` | `recipes_menu.rs`（context-menu.tsx） |
| Lightbox / useLightbox | https://astryx.atmeta.com/components/Lightbox | `Lightbox/` | NEW kit component（全画面 modal overlay + zoom/pan） | `carousel.rs`, `layers.rs` |
| Overlay (OverlayScrim, useOverlay) | https://astryx.atmeta.com/components/Overlay | `Overlay/` | NEW kit component `soft::media_overlay`（メディア上の hover/focus 表示。モーダル scrim ではない） | なし（`card.rs` の media が近い） |
| Layer / useLayer / LayerProvider | https://astryx.atmeta.com/components/Layer | `Layer/` | covered by `kivact::view::overlay`（soft は既定 offset・入場アニメ・16px gutter の共通 helper のみ） | `layers.rs` |
| useFocusTrap | https://astryx.atmeta.com/components/useFocusTrap | `hooks/` | covered by kivact 基盤（`overlay.modal(true)` + FocusReturn） | なし |
| useScrollLock | https://astryx.atmeta.com/components/useScrollLock | `hooks/` | covered by kivact 基盤（modal overlay） | なし |
| useAnnounce | https://astryx.atmeta.com/components/useAnnounce | `hooks/` | NEW kit helper `soft::announce`（常設 LiveRegion ノード） | なし |

## 0. 共通

### 0.1 解決済みトークン早見表（neutral）

| token | light | dark | 主な用途 |
|---|---|---|---|
| `--color-background-surface` | `#ffffff` | `#262626` | Dialog / Drawer / BottomSheet / HoverCard 面, Tooltip 文字 |
| `--color-background-popover` | `#ffffff` | `#1b1b1b` | Popover / DropdownMenu / ContextMenu 面 |
| `--color-background-inverted` | `#0A1317` | `#FFFFFF` | Toast（core 既定。neutral 未上書き） |
| `--color-background-error-inverted` | `#AA071E` | `#E3193B` | Toast error（core 既定） |
| `--color-text-primary` | `#000000` | `#ffffff` | 本文, Tooltip 背景 |
| `--color-text-secondary` | `#474747` | `#9e9e9e` | 補足・見出し |
| `--color-icon-secondary` | `#6a6a6a` | `#9e9e9e` | メニュー行アイコン |
| `--color-text-error` | `#c0262c` | `#ffaea7` | destructive 行 |
| `--color-error-muted` / `--color-on-error-muted` | `#ffc4be` / `#76000c` | `#ff98903D` / `#ffaea7` | destructive ボタン（neutral の component override） |
| `--color-accent` | `#1b1b1b` | `#f1f1f1` | focus ring |
| `--color-accent-muted` | `#f1f1f1` | `#262626` | 選択中の行 |
| `--color-neutral` | `#0000000F` | `#FFFFFF1A` | secondary ボタン |
| `--color-overlay` | `#00000080` | `#000000CC` | scrim |
| `--color-overlay-hover` | `#0000000D` | `#FFFFFF0D` | hover / highlight |
| `--color-overlay-pressed` | `#0000001A` | `#FFFFFF1A` | pressed / 開いているトリガー |
| `--color-border` | `#00000014` | `#FFFFFF1A` | hairline, divider, pill |
| `--color-border-emphasized` | `#d4d4d4` | `#525252` | hover indication 下線 |
| `--color-on-dark` | `#ffffff` | `#ffffff` | Lightbox 文字 |
| `--shadow-low` | `0 2px 4px rgba(0,0,0,.05), 0 4px 8px rgba(0,0,0,.10)` | `0 2px 4px rgba(0,0,0,.25), 0 4px 8px rgba(0,0,0,.40), inset 0 0 0 1px rgba(255,255,255,.08)` | Popover, メニュー |
| `--shadow-med` | `0 2px 4px rgba(0,0,0,.05), 0 4px 12px rgba(0,0,0,.10)` | `0 2px 4px rgba(0,0,0,.35), 0 4px 12px rgba(0,0,0,.50), inset 0 0 0 1px rgba(255,255,255,.12)` | HoverCard, Toast |
| `--shadow-high` | `0 4px 6px rgba(0,0,0,.10), 0 12px 24px rgba(0,0,0,.15)` | `0 4px 6px rgba(0,0,0,.50), 0 12px 24px rgba(0,0,0,.70), inset 0 0 0 1px rgba(255,255,255,.15)` | Dialog, Drawer, BottomSheet |
| `--radius-inner` / `-element` / `-container` / `-page` / `-full` | 6 / 10 / 12 / 28 / 9999 px | 同 | |
| `--spacing-n` | n × 4px（0-5 = 2px, 1-5 = 6px） | | |
| `--size-element-sm/md/lg` | 28 / 32 / 36 px | | ボタン高 |
| `--focus-outline-*` | 2px solid `--color-accent`, offset 3px | | |
| type | body 14/20 400, label 14/20 500, supporting 12/20 400, large 17/24 600, h2 20/28 600, h3 17/24 **700**, h4 14/20 **700**（neutral の bold 上書き）; font Figtree | | |
| duration | fast-min 95, fast 125, fast-max 165, medium-min 225, medium 300, medium-max 400, slow-min 525, slow 700, slow-max 935 ms | | neutral motion `{fast:125, medium:300, slow:700, ratio:0.75}`（5ms 丸め） |
| easing | `--ease-standard` = `--ease-entrance` = `--ease-move` = cubic-bezier(0.24, 1, 0.4, 1); `--ease-exit` = cubic-bezier(0.3, 0, 0.6, 0.6) | | |

### 0.2 オーバーレイ共通の約束（この章全体で繰り返し出るもの）

- **入場アニメ（アンカー型）**: opacity 0→1 + 配置方向から 8px + scale 0.95→1、165ms、ease-standard。**退場アニメは無い**（即時）。詳細は 15.5。
- **モーダル scrim**: Dialog / AlertDialog / CommandPalette / Drawer / Lightbox は `--color-overlay` + `backdrop-filter: blur(2px)`。BottomSheet は blur 無し。
- **Escape は最上位レイヤー 1 枚だけ**（15.6）。外側押下はトリガー上を除外。
- **ビューポート余白 16px**: アンカー型レイヤーは各ウィンドウ端から 16px 以上離し、最大サイズはウィンドウ − 32px（15.4）。
- **reduced motion**: マウス環境では即時、タッチ（coarse）では最大 200ms のフェードのみ。デスクトップネイティブでは「即時」を既定とする。
- **i18n**: 既定テキストは `soft.<component>.<name>` キー（各章 6 項に列挙）。
- **非ネイティブ要素の扱い**: `href`/`target`/`rel` は削除し `on_press`/`on_select` に、`xstyle`/`className`/`style` は削除し `.style(..)`（recipe の後に勝つ）に、`isInline` はカタログ用プレビュー関数に置き換える。

## 1. Dialog

docs: https://astryx.atmeta.com/components/Dialog ・ source: `packages/core/src/Dialog/`（`Dialog.tsx` 874 行, `DialogHeader.tsx`, `DialogContext.ts`, `useImperativeDialog.tsx`）

### 1.1 概要

ページ操作をブロックし、ユーザーの応答を待つモーダル。削除確認・編集フォーム・規約同意など「飛ばせない判断」に使う。`purpose` で閉じ方を選ぶ（info=何でも閉じる / form=背景クリック不可 / required=一切自動で閉じない）。

- 使う: タイトル付きで 1 タスクに集中した内容。入力を含むなら `purpose=form`。
- 使わない: インラインや Toast で足りる単純な通知。ダイアログの入れ子（ステップ化で置き換える）。単純な確認に fullscreen。

### 1.2 Anatomy

```
<dialog aria-modal="true" [role="alertdialog" if purpose=required] aria-labelledby=<titleId>>   ← surface (styles.dialog)
  ::backdrop                                         ← scrim (overlay color + blur 2px)
  <div inner>                                         ← styles.inner + container(padding 16)
    <Layout>                                          ← 呼び出し側が組む
      header : DialogHeader（title h2 / subtitle / startContent / endContent / close）
      content: LayoutContent（本文。スクロール担当）
      footer : LayoutFooter（アクション, end 揃え）
```

- タイトルの id は `DialogContext.titleId`（`useId`）。DialogHeader がその id で `<h2>` を出すと、Dialog の callback ref が見つけて `aria-labelledby` を自動設定（`Dialog.tsx` attachDialog）。呼び出し側が `aria-label`/`aria-labelledby` を渡せばそちらが優先。名前がない開いたダイアログは dev 警告。
- `ModalAnnounceProvider`（モーダル内の live 通知を dialog 内に向ける）、`LayerDepthProvider`（入れ子レイヤーの深さ）、`LayerContentBoundary` で包む。

### 1.3 API

| prop | 型 | 既定 | 意味 | Kivact |
|---|---|---|---|---|
| `isOpen` | bool | 必須 | 開閉 | `dialog(title, open: Binding<bool>)` の `open`（呼び出し側 signal） |
| `onOpenChange` | (bool)=>unknown | 必須 | 閉要求で `false` | binding への書き込みで代替（追加で `.on_open_change(f)` 不要） |
| `children` | ReactNode | 必須 | 中身（通常 Layout） | `.content(|| ..)` / `.footer(|| ..)`、ヘッダは下記 `.header(..)` |
| `width` | number\|string | `400` | 希望幅(px)。`min(100%, 100dvw − 16 − 16)` で clamp | `.width(Length)`（既存） |
| `maxHeight` | number\|string | `'75dvh'` | 最大高 | `.max_height(Length)` 新設（既定 = ウィンドウ高 × 0.75） |
| `position` | `{top?, bottom?, start?, end?}` | なし=中央 | 静的配置。start/end は論理（RTL 反転） | `.position(DialogPosition { top, bottom, start, end })` 新設（kit 側 wrapper で可） |
| `variant` | `'standard' \| 'fullscreen'` | `'standard'` | fullscreen は全画面 | `.variant(soft::DialogVariant::Fullscreen)` |
| `purpose` | `'required' \| 'form' \| 'info'` | `'info'` | 閉じ方 | `.purpose(soft::DialogPurpose::Info|Form|Required)` → 内部で `.dismiss(Dismiss{escape, outside})` と kind に変換: Info=`{true,true}`, Form=`{true,false}`, Required=`{false,false}` + `DialogKind::AlertModal`（role alertdialog） |
| `padding` | 0\|0.5\|1\|1.5\|2\|3\|4\|5\|6\|8\|10 | テーマ既定=4 (16px) | 内側 padding（spacing step） | `.padding(SpacingStep)`（f32 step）。既定 16px |
| `isInline` | bool | `false` | ドキュメント用インライン表示（モーダル挙動なし） | ネイティブではカタログ用 `soft::dialog_preview(..)`（surface の見た目だけを描くビュー）として別関数。本番 API には不要 |
| `xstyle` / `className` / `style` | | | | 削除。`NodeBuilder` の `.style(..)` が recipe 後に勝つ |
| `ref` | | | | `.node_ref(..)` |
| `aria-label` / `aria-labelledby` | | | 名前の上書き | `.name(..)` / `labelled_by` |

注意（実装と doc の差）: doc.mjs は「form は interaction 後に backdrop click を無効」と書くが、実装（`Dialog.tsx:585-586`）は `allowEscape = purpose !== 'required'`, `allowBackdropClick = purpose === 'info'` で **form は常に backdrop click 無効**。実装に合わせる。

### 1.4 Visual spec

surface（`Dialog.tsx:184-215` `styles.dialog`、`open` 218-232）:

| 項目 | 値 | neutral light | neutral dark |
|---|---|---|---|
| background | `--color-background-surface` | `#ffffff` | `#262626` |
| radius | `--_dialog-radius` = `--radius-container` | 12px | 12px |
| shadow | `--shadow-high` | `0 4px 6px rgba(0,0,0,.10), 0 12px 24px rgba(0,0,0,.15)` | `0 4px 6px rgba(0,0,0,.50), 0 12px 24px rgba(0,0,0,.70), inset 0 0 0 1px rgba(255,255,255,.15)` |
| border | なし（0） | – | – |
| width | 400px 既定。max = `min(親幅, ウィンドウ幅 − 32px)`（`Dialog.tsx:297-299` gutter `--spacing-4`=16px ×2） | | |
| height | fit-content。max = 75% of ウィンドウ高（`75dvh`） | | |
| 配置 | `position: fixed; margin: auto` → 画面中央。`position` 指定時は margin 0 + top/bottom/inset-inline-start/end（未指定辺は auto） | | |
| 内側 padding | `--astryx-dialog-padding` → `--spacing-4` = 16px 全辺（`Layout/container.stylex.ts:135-140`）。Layout の header/content/footer は互いに 16px 間隔、divider なしの境界は片側 padding を潰して 16px 1 本になる | | |
| overflow | inner は `overflow: hidden; min-height: 0; flex 1 1 auto; border-radius: inherit`（`Dialog.tsx:273-280`）。スクロールは LayoutContent 側 | | |

scrim（`::backdrop`, `Dialog.tsx:237-242`）:

| 項目 | light | dark |
|---|---|---|
| color `--color-overlay` | `#00000080`（黒 50%） | `#000000CC`（黒 80%） |
| backdrop-filter | `blur(2px)` | 同 |

fullscreen（`Dialog.tsx:243-251`）: width/height = 100dvw/100dvh、radius 0、margin 0、inset 0。padding はテーマ既定時のみ safe-area を考慮: 各辺 `max(16px, env(safe-area-inset-*))`（RTL で left/right を入れ替え, `Dialog.tsx:159-164`, 252-271）。ネイティブでは safe-area = 0 として 16px。

DialogHeader（1.p1 参照）を含む典型ヘッダ:

```
┌──────────────────────────────────────────── 400 ┐
│16                                            16 │
│  Dialog Title (h2 20/28 600)            [ × ]   │ ← × は 32×32 ghost, 上下 -8px / 右 -8px 補正
│  subtitle (12px / lh 1.4286≈17px, secondary)    │
│                                                 │
│  本文 body 14/20 #000 / #fff                    │
│                                                 │
│                         [Cancel] [ Primary ]    │ ← footer, end 揃え, gap 8
│16                                               │
└─────────────────────────────────────────────────┘ radius 12, shadow-high
```

状態: Dialog 自体に hover/pressed はない。focus-visible は `focusOutlineProps.focusVisible`（2px `--focus-outline-color`=`--color-accent` light `#1b1b1b` / dark `#f1f1f1`、offset 3px）を surface 自身に適用（surface が focus を受けた時のみ）。disabled/invalid/loading なし。

テキスト既定: モーダル内は `layerTextReset`（body: Figtree 14px/20px 400、`--color-text-primary` light `#000000` / dark `#ffffff`）から始まる。

RTL: `position.start/end` は inset-inline で反転。fullscreen safe-area padding も反転。入場方向ベクトルは物理座標（トリガーの実位置）なので反転処理不要。

### 1.5 Motion

| トリガー | プロパティ | from → to | duration | easing | 備考 / file:line |
|---|---|---|---|---|---|
| open（standard） | opacity, transform | `opacity 0; translate(dx, dy) scale(0.95)` → `opacity 1; translate(0,0) scale(1)` | `--duration-medium-max` = **400ms** (neutral) | `--ease-standard` = cubic-bezier(0.24, 1, 0.4, 1) | `Dialog.tsx:145-152`, 197-206。fill-mode backwards |
| 方向ベクトル (dx, dy) | | トリガー要素中心 − ビューポート中心 を正規化し **16px** 倍、整数丸め。トリガーが body/なしなら (0, 16px) | | | `getDialogDirection` `Dialog.tsx:92-107`, 600-611。すなわち「トリガーのある方向から」中央へ寄ってくる |
| open（fullscreen） | opacity | 0 → 1 | 400ms | ease-standard | `Dialog.tsx:154-157`, 252-259 |
| reduced motion + coarse pointer | opacity | 0 → 1（移動・拡縮なし） | **200ms** | cubic-bezier(0.42, 0, 0.58, 1)（easeInOut） | `Dialog.tsx:203-213`, 222-229 |
| reduced motion + fine pointer | – | 即時表示（animation none） | 0 | – | 同上 |
| ページロード時に既に open | – | アニメなし（`openSettled`） | 0 | – | `Dialog.tsx:233-236`, `useSettledOpen`。ネイティブ: マウント時点で open=true なら enter を省略 |
| close | – | **退場アニメなし**（`dialog.close()` で即 display none）。scrim も即消え | 0 | – | `Dialog.tsx:624-633` |
| scrim | – | `::backdrop` に animation 指定なし → 即時表示/即時消去 | | | |

ネイティブ実装上の注意: 入場は DialogRecipes::surface の `enter` style（opacity 0 + translate + scale 0.95 → 1）に 400ms / bezier(0.24,1,0.4,1) で表す。translate はトリガーの NodeRef から測る必要があり、recipe だけでは方向が取れない場合は (0, 16px) 固定にフォールバックし UNRESOLVED（kivact::ui::dialog がトリガー位置を DialogState に渡すか要確認）とする。

### 1.6 Behaviour & a11y

- ロール: `<dialog aria-modal="true">` = Role::Dialog（kind Modal）。`purpose=required` の時のみ `role="alertdialog"`（`Dialog.tsx:864`）。名前 = DialogHeader の title テキスト。
- 開く: `showModal()` 後、`[data-autofocus]` 要素があればそこへ focus（AlertDialog の Cancel が使用）。なければ DialogHeader が `<h2 tabIndex=-1>` タイトルに focus（`DialogHeader.tsx:193-197`、outline none）。ネイティブ: `.initial_focus(NodeRef)` が既存。ヘッダがあればタイトル、AlertDialog は Cancel。
- focus 表示はモダリティ追従: ポインタで開いたら focus ring を隠し、キー操作で開いたら出す（`focusForLastInput`）。
- 閉じた時: 開く直前の activeElement（トリガー）へ focus を戻す（`Dialog.tsx:592-599`, 628-633）。
- Escape: `useLayerDismissal` のスタックで最上位レイヤーだけが反応。`purpose != required` なら `onOpenChange(false)`。required は `escapeBehavior: 'block'`（Escape を飲み込み下のレイヤーにも渡さない）。
- 背景クリック: `event.target === dialog`（= backdrop 部分）かつ info の時だけ閉じる（`Dialog.tsx:695-699`）。
- スクロールロック: open 中は `useScrollLock(true)`（16 章）。
- `useModalOutlet`: モーダル中、他の layer（Toast など）を dialog 内へ出すための outlet 登録。
- フォーカストラップ: ネイティブ `<dialog>` の inert で実現。Kivact: `overlay(..).modal(true)` 相当（`DialogKind::Modal` で既存）。
- テキスト: 閉じるボタン `@astryx.dialog.close` = "Close" → 既存キー `dialog.close` を使うか `soft.dialog.close` を新設（推奨: `soft.dialog.close` = "Close"、ツールチップと名前の両方）。

### 1.7 Kivact 実装方針

- **RESTYLE `kivact::ui::dialog`（`DialogKind::Modal`）**。見た目は `DialogRecipes`（surface/title/description/header/media/body/footer/scrim/close）で全部表現できる: surface = bg surface, radius 12, shadow-high, padding 16, width 400/max(window−32), max-height 75%; scrim = overlay 色 + backdrop blur 2px（kivact-core は `Filter::new().blur(..)` の backdrop_filter をサポート: `kivact-core/src/test/screen/gpu_tests.rs:176`）。
- `purpose` / `variant(Fullscreen)` / `position` / `max_height` / `padding` は kit 側 wrapper `soft::dialog(title, open)`（中で `kivact::ui::dialog` を組み立てる関数）または `Variant` 型で表す。Fullscreen は surface recipe が `DialogState` の variant を読んで width/height 100%, radius 0 に切り替える。
- 参考: `crates/kivact-ui-basic/src/recipes_dialog.rs`（dialog.tsx の recipe 一式）, `crates/kivact-ui-basic/src/layers.rs`（scrim と close box）, `crates/kivact-ui/src/dialog.rs`。
- 難所: (1) 入場方向がトリガー位置依存 → DialogState にトリガー矩形がなければ固定 (0,16px) で妥協（UNRESOLVED）。(2) DialogHeader の「タイトル h2 に初期 focus」はネイティブ dialog が "content の最初のコントロール" に focus する仕様と異なる（`kivact-ui/src/dialog.rs` 冒頭コメント）。タイトルノードに `.focusable(true)` + `initial_focus(title_ref)` を kit wrapper が設定する。(3) `isInline` はカタログ用の静的プレビューで代替。

### 1.8 受け入れ確認

- [ ] `role=dialog` が名前（タイトル文字列）付きで出る。required 時は `role=alertdialog`。
- [ ] 開くとタイトル（または data-autofocus 相当）に focus、閉じるとトリガーに戻る。
- [ ] info: Escape と背景押下で閉じる / form: Escape のみ / required: どちらでも閉じない。
- [ ] ポップオーバーをダイアログ内で開き Escape → ポップオーバーだけ閉じる。
- [ ] 幅 400、ウィンドウ幅 360 では 328（左右 16 の余白）。高さは 75% 超でスクロール。
- [ ] 目視（light/dark）: 白 / #262626 の 12px 角丸、shadow-high（dark はリム 1px 白 15%）、scrim 黒 50% / 80% + 2px ぼかし。
- [ ] 入場: 400ms で 0.95→1 拡大＋トリガー側から 16px 寄る。閉じは即時。reduced motion で移動なし。
- [ ] fullscreen: 角丸 0・全面・フェードのみ。

### 1.p1 DialogHeader

`Dialog/DialogHeader.tsx`（docs: https://astryx.atmeta.com/components/DialogHeader）

#### 概要
ダイアログ上端の見出し行。タイトル（必須、ダイアログ名になり初期 focus を受ける）、サブタイトル、start/end スロット、閉じるボタン（`onOpenChange` を渡した時のみ）。Drawer は DrawerHeader（3 章）を使う。

#### Anatomy
```
LayoutHeader [hasDivider → border-bottom 1px --color-border]
 └ div.container (flex, align-items:flex-start, space-between, gap 12px)
    ├ div.start-content (flex, center, gap 8, flex-shrink 0)        ← startContent（戻るボタン等）
    ├ div.title-block (flex column, gap 0, flex 1, min-width 0, margin-block -6px)
    │   ├ h2#titleId tabIndex=-1 (outline none)                       ← title
    │   └ Text body sm secondary                                      ← subtitle
    └ div.end-content (flex, center, gap 8, flex-shrink 0, [margin-block -8px] [margin-inline-end -8px])
        ├ endContent
        └ Button ghost icon-only "Close" (icon close)                  ← onOpenChange がある時
```

#### API
| prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `title` | ReactNode | 必須 | `dialog(title, ..)` の title（既存）。リッチ表示が必要なら `.title_view(|| ..)` |
| `subtitle` | ReactNode | – | `.description(text)`（既存。見た目を subtitle 扱い: 12px secondary） |
| `onOpenChange` | fn | – | 指定時のみ閉じるボタン → `.closable(bool)`（既存） |
| `startContent` | ReactNode | – | `.header_start(|| ..)` 新設（kit wrapper） |
| `endContent` | ReactNode | – | `.header_end(|| ..)` 新設 |
| `endContentEdgeCompensation` | `'inline'|'block'|'all'` | close 有無で自動 | `.header_end_compensation(EdgeCompensation)` 任意。既定: close があれば block+inline |
| `hasDivider` | bool | Layout の既定 → false | `.header_divider(bool)` |

#### Visual spec
- container gap `--spacing-3` = 12px（`DialogHeader.tsx:41-46`）。
- title: Heading level 2 = `--text-heading-2-size` 20px / leading 1.4 (28px) / `--text-heading-2-weight` 600（neutral は h3/h4 のみ bold、h2 は semibold 600）、色 text-primary（`#000000` / `#ffffff`）、Figtree。
- title-block の `margin-block = 32/2 − 8 − 20×1.4/2 = −6px`（`DialogHeader.tsx:54-67`）: タイトル行の中心を閉じるボタン中心に合わせる。
- subtitle: Text type=body size=sm → font-size `--font-size-sm` 12px、line-height は body の 1.4286 を継承（≈17.1px。supporting の 20px ではない, `Text/text.stylex.ts:165-167`）、`--color-text-secondary` light `#474747` / dark `#9e9e9e`。title との gap 0。
- end slot: gap 8px。close がある時 `margin-block: -8px`、`margin-inline-end: -8px`（`DialogHeader.tsx:47-53`）でアイコンボタンの見かけの余白を打ち消す。
- 閉じるボタン: Button `variant=ghost`, size md → 32×32 正方形（`--size-element-md`）、radius `--radius-element` 10px、背景 transparent、アイコン close 16px（md ボタンの icon size = sm = 1rem）, 色 `--color-text-primary`。hover: `--color-overlay-hover` を重ねる（light `#0000000D` / dark `#FFFFFF0D`）、pressed: `--color-overlay-pressed`（`#0000001A` / `#FFFFFF1A`）+ scale(0.98)（マウス時のみ）, transition 125ms ease-standard。ツールチップ "Close" 付き（Tooltip 8 章）。
- hasDivider: LayoutHeader 下端 1px `--color-border`（light `#00000014` / dark `#FFFFFF1A`）。
- RTL: flex が自動反転、end compensation は inline-end（論理）。

#### Motion
自身のアニメなし。閉じるボタンの背景/transform transition 125ms (`--duration-fast`) `--ease-standard`（Button.tsx:118-125）。

#### Behaviour & a11y
- タイトル h2 はマウント時に `focus()`（inline 時は抑止）。名前は h2 のテキスト。
- 閉じるボタン: name = tooltip = "Close"（`@astryx.dialog.close`）→ `soft.dialog.close`。
- 閉じるボタンは DOM 上タイトルの後 → Tab 順はタイトル → endContent → Close → 本文…（Astryx）。kivact の dialog は close を最後の子に置くので順序が異なる（許容。UNRESOLVED ではなく差分として記録）。

#### Kivact 実装方針
`DialogRecipes::header`/`title`/`description`/`close` の restyle。start/end スロットは kit wrapper が `media`（既存スロット）を流用せず、ヘッダ行を自前で組み、`hide_title(true)` で標準見出しを消して代替する案もあるが、名前付けと初期 focus を保つため **標準 header を使い start/end は wrapper の新スロット**を推奨。参考 `crates/kivact-ui-basic/src/recipes_dialog.rs`。

#### 受け入れ確認
- [ ] 見出し 20px semibold、× ボタン中心とタイトル 1 行目中心が一致。
- [ ] subtitle 12px グレー。
- [ ] onOpenChange 無し（closable false）で × が出ない。
- [ ] × ホバーで薄いグレー塗り（light 5% 黒 / dark 5% 白）。

### 1.p2 useImperativeDialog

`Dialog/useImperativeDialog.tsx`（docs: https://astryx.atmeta.com/components/useImperativeDialog）

- 概要: open state を持たずに `dialog.show(content, options?)` / `dialog.hide()` で出すフック。戻り値 `{show, hide, isOpen, element}`、`element` をツリーに置く。`show` の options は前回分にマージ（`{...prev, ...new}`）。閉じても content は保持（次の show で置換）。
- Kivact API 提案: 
  ```rust
  let dlg = soft::use_imperative_dialog(DialogOptions::default()); // Copy handle
  dlg.show(|| view, Some(opts)); dlg.hide(); dlg.is_open() -> bool;
  // ツリーに dlg.view() を 1 回置く
  ```
  内部は `signal(false)` + `signal(Option<Rc<dyn Fn() -> View>>)` + `soft::dialog`。Kivact では caller-owned signal が慣習なので、ハンドル自体が signal を持つ Copy 型にする。
- 視覚・モーションは Dialog と同一。
- 受け入れ: show で開き、Escape で閉じ `is_open()==false`、再 show で新しい内容。

## 2. AlertDialog

docs: https://astryx.atmeta.com/components/AlertDialog ・ source: `packages/core/src/AlertDialog/`（`AlertDialog.tsx` 262 行, `useImperativeAlertDialog.tsx`）

### 2.1 概要
破壊的・不可逆な操作（削除、権限剥奪、未保存変更の破棄）の前に確認する。WAI-ARIA APG Alert Dialog パターン。外側クリックでは閉じない、Escape はキャンセル。

- 使う: 行動ラベルを具体的に（"Delete project"）。説明に結果を書く。Cancel を最も安全な初期 focus に。処理中は `isActionLoading` で開いたまま保持し、終わってから閉じる。
- 使わない: 非破壊操作（通常 Dialog を使う）。色だけで危険を伝える。

### 2.2 Anatomy
```
Dialog(purpose="form", width=400, role="alertdialog", aria-labelledby=title, aria-describedby=desc)
 └ Layout
    ├ content: LayoutContent
    │    ├ h2#title   (Heading level 2)
    │    └ Text#desc  (body, secondary)
    └ footer: LayoutFooter
         └ Stack  >640px: horizontal, gap 8, hAlign end, wrap   → [Cancel(ghost)] [Action(destructive)]
                  ≤640px: vertical,   gap 8, stretch, nowrap  → [Action] / [Cancel]（両方 width 100%）
```
ヘッダ（DialogHeader）・閉じる × は無い。

### 2.3 API
| prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `isOpen` / `onOpenChange` | | 必須 | `soft::alert_dialog(title, description, open)` |
| `title` | string | 必須 | 第 1 引数 |
| `description` | string | 必須 | 第 2 引数（`.description` へ。aria-describedby） |
| `actionLabel` | string | 必須 | `.action(label, on_action)` |
| `onAction` | fn | 必須 | 同上。**自動で閉じない** |
| `cancelLabel` | string | "Cancel"（`@astryx.alertDialog.cancel`） | `.cancel_label(text)`、既定キー `soft.alert_dialog.cancel` = "Cancel" |
| `actionVariant` | ButtonVariant | `'destructive'` | `.action_variant(ButtonVariant::Danger)` 既定 Danger |
| `isActionLoading` | bool | – | `.action_loading(impl IntoValue<bool>)` |
| `width` | number\|string | 400 | `.width(Length)` |
| `isInline` | bool | false | プレビュー専用（inline 時 role=group） |
| `xstyle`/`className`/`style` | | | 削除 |

### 2.4 Visual spec
- surface/scrim/padding/幅/影/radius は Dialog と同一（1.4）: bg `#ffffff`/`#262626`, radius 12, `--shadow-high`, scrim `#00000080`/`#000000CC` + blur 2px, padding 16、幅 400（max ウィンドウ−32）。
- title: h2 20px/28px 600 text-primary。description: body 14px/20px 400 `--color-text-secondary`（`#474747` / `#9e9e9e`）。title と description の間隔 0（ブロック積み）。
- content と footer の間 16px（footer の padding-block-start = `--layout-padding-inner-y` 16px、divider なしで content 下 padding は 0 に潰れる）。
- Cancel: Button ghost md: 高さ 32px, padding 8px 12px, radius 10px, label 14px/20px 500 text-primary, 背景透明, hover overlay-hover。
- Action（destructive, neutral の component override `neutralTheme.ts` button `variant:destructive`）: 背景 `--color-error-muted` light `#ffc4be` / dark `#ff98903D`、文字 `--color-on-error-muted` light `#76000c` / dark `#ffaea7`。focus ring 色は `--color-error`（light `#c9303a` / dark `#ff705d`）。loading: Button の spinner（Button 担当グループ参照）。
- 狭幅（ウィンドウ幅 ≤ 640px, `AlertDialog.tsx:42`）: 縦積み、Action が上、両ボタン width 100%。>640px: 横並び end 揃え、gap 8px、収まらなければ折り返し。
- ボタンは `max-width: 100%; min-width: 0`、ラベルは 1 行 ellipsis。

### 2.5 Motion
Dialog と同じ（400ms ease-standard、方向付き translate 16px + scale 0.95→1、退場なし、reduced motion: coarse で 200ms フェード / fine で即時）。

### 2.6 Behaviour & a11y
- role `alertdialog`、`aria-labelledby`=title、`aria-describedby`=description。
- 初期 focus: Cancel（`data-autofocus`, `AlertDialog.tsx:195`）。狭幅で順序が入れ替わっても Cancel。
- purpose=form → Escape で `onOpenChange(false)`（キャンセル扱い）、背景クリックでは閉じない。
- Action 押下は `onAction` のみ（閉じない）。
- 文字列: `soft.alert_dialog.cancel` = "Cancel"。

### 2.7 Kivact 実装方針
- **RESTYLE `kivact::ui::dialog`（`DialogKind::AlertModal`）+ kit wrapper `soft::alert_dialog`**。AlertModal は外側押下で閉じない仕様が既にある（`kivact-interaction/src/dialog.rs:63-65`）。`.dismiss(Dismiss{escape: true, outside: false})`、`.closable(false)`、`.initial_focus(cancel_ref)`。
- footer は wrapper が `row`/`column` を幅で切替（ウィンドウ幅 ≤ 640 の reactive 判定。kivact でウィンドウ幅を読む API を使用、なければ dialog 幅で代替）。
- 参考: `crates/kivact-ui-basic/src/alert_dialog.rs`（AlertDialogSize と footer の扱い）, `recipes_dialog.rs`。

### 2.8 受け入れ確認
- [ ] `role=alertdialog` 名前=title、説明=description。
- [ ] 開くと Cancel に focus。Escape で閉じる。背景押下で閉じない。
- [ ] Action 押下で閉じない（呼び出し側が閉じる）。loading 中スピナー。
- [ ] 幅 > 640: [Cancel][Delete] 右寄せ。≤ 640: Delete が上で全幅。
- [ ] 目視: Delete ボタンが薄赤（light `#ffc4be` 地に `#76000c` 字 / dark 赤 24% 地に `#ffaea7` 字）。

### 2.p1 useImperativeAlertDialog
`AlertDialog/useImperativeAlertDialog.tsx`。`show(options)`（全 props − isOpen/onOpenChange）/`hide()`/`isOpen`/`element`。options が null の間 element は null。閉じても options は保持。Kivact: `soft::use_imperative_alert_dialog()` → Copy ハンドル `.show(AlertOptions{title, description, action_label, on_action, ..})`, `.hide()`, `.is_open()`, `.view()`。受け入れ: show→Cancel focus→Escape→is_open false。

## 3. Drawer

docs: https://astryx.atmeta.com/components/Drawer ・ source: `packages/core/src/Drawer/`（`Drawer.tsx` 661 行, `DrawerHeader.tsx`, `useDrawerDialogPresence.ts`）

### 3.1 概要
インスペクタ／詳細ビュー（「表の行をクリック → 詳細」）用の全高サイドパネル。inline 軸（start/end）からスライドし、ページの上に浮く（レイアウトを押し出さない）。既定はモーダル（scrim あり）、`hasScrim={false}` で非モーダル（背後のページが操作可能、master-detail）。

- 使う: 下のリストを見せたまま文脈詳細を出す。選択状態から `isOpen` を導出し、`onOpenChange` で選択を消す。閉じアニメ中は中身を残す（null にすると滑り出し中に空になる）。必ず目に見える閉じる手段（DrawerHeader + onOpenChange）を置く。未保存入力は `purpose=form`。
- 使わない: 短い確認・小フォーム（Dialog/AlertDialog）。ページを押し出したい場合（ドック型カラム）。上下シート（BottomSheet）。Drawer の入れ子（兄弟として並べる。最後に開いたものが上、Escape はそれだけ閉じる）。
- **Astryx Drawer にはドラッグ／スワイプで閉じる機能は無い**（ジェスチャは BottomSheet の担当）。kivact-ui-basic の `drawer`（スワイプで閉じる）とは仕様が違う点に注意。

### 3.2 Anatomy
```
<dialog aria-label=label [aria-modal=true if hasScrim] [role=alertdialog if required && hasScrim] [popover=manual if !hasScrim] data-side>
  ::backdrop (hasScrim)                                   ← scrim
  <div tabIndex=-1 content (scroll-y, container padding 16)>   ← 初期 focus 先（パネル本体）
     children（通常 Layout: header=DrawerHeader / content / footer）
```
- 名前は必須 prop `label`（DrawerHeader の title は名前にならない）。
- `[data-autofocus]` 要素があれば開いた時そこへ focus。

### 3.3 API
| prop | 型 | 既定 | 意味 | Kivact |
|---|---|---|---|---|
| `isOpen` / `onOpenChange` | | 必須 | 完全制御 | `soft::drawer(label, open: Binding<bool>)` |
| `side` | `'start' \| 'end'` | `'end'` | 出てくる辺（論理） | `.side(DrawerSide::Start|End)` → `DialogKind::Drawer(Side::Left/Right)`（RTL で反転させる。kivact の Side は物理なので wrapper が方向を見て変換） |
| `width` | number\|string | 400 | デスクトップ幅。≤640px では `min(width, 100dvw − 56px)` | `.width(Length)` |
| `experimental.isFullWidthOnMobile` | bool | false | ≤640px で全幅 | `.full_width_on_mobile(bool)` |
| `label` | string | 必須 | aria-label | 第 1 引数 |
| `hasScrim` | bool | true | true=モーダル（focus trap・scroll lock・外側クリック）、false=非モーダル | `.scrim(bool)`。false → `DialogKind::Popover` 的非モーダル（実装は 3.7） |
| `purpose` | required\|form\|info | info | Dialog と同じ（非モーダルでは form=info） | `.purpose(DialogPurpose)` |
| `padding` | SpacingStep | 4 (16px) | content の inset。0 で full-bleed | `.padding(step)` |
| `children` | ReactNode | 必須 | 中身（スクロール領域内） | `.content(|| ..)`。ヘッダ/フッタは 3.p1 |
| `data-testid` | | | | `.test_id(..)` |
| `xstyle`/`className`/`style`/`onClick`/`onKeyDown` | | | | 削除（`.style` で上書き） |

### 3.4 Visual spec
パネル（`Drawer.tsx:98-128`, `end` 129-140, `start` 150-161）:

| 項目 | 値 | light | dark |
|---|---|---|---|
| 位置 | `position: fixed; top 0; bottom 0; height 100dvh`、end なら inline-end 0 / start なら inline-start 0 | | |
| 幅 | 400px（>640px）。≤640px: `min(400px, ウィンドウ幅 − 56px)`（`Drawer.tsx:94-96`, 222-232, 520-523）。full-width オプションで 100%。max 100dvw | | |
| background | `--color-background-surface` | `#ffffff` | `#262626` |
| 境界線 | ページ側の 1 辺だけ（end パネルなら inline-start 辺）`--border-width` 1px solid `--color-border` | `#00000014` | `#FFFFFF1A` |
| radius | 0 | | |
| shadow | `--shadow-high` | `0 4px 6px rgba(0,0,0,.1), 0 12px 24px rgba(0,0,0,.15)` | `0 4px 6px rgba(0,0,0,.5), 0 12px 24px rgba(0,0,0,.7), inset 0 0 0 1px rgba(255,255,255,.15)` |
| overflow | clip（パネル）、content は `overflow-y: auto; overscroll contain; touch-action pan-y` | | |
| content padding | 16px 全辺（`--astryx-drawer-padding` → `--spacing-4`）。block-end は + safe-area-inset-bottom（ネイティブ 0） | | |

scrim（`Drawer.tsx:177-196`）: `--color-overlay`（light `#00000080` / dark `#000000CC`）+ `backdrop-filter: blur(2px)`。opacity 0→1 transition。hasScrim=false では scrim なし。

```
┌───────────────── page ─────────────┬──────── 400 ───────┐
│ (scrim 黒50% + blur2)              │ Title          [×] │ ← DrawerHeader（Dialog と同寸）
│                                    │────────────────────│ (hasDivider 時)
│                                    │ content (scroll)   │
│                                    │                    │
│                                    │        [actions]   │
└────────────────────────────────────┴────────────────────┘
                                     ↑ 1px border(--color-border) + shadow-high
```

RTL: `side=end` は左端に付き、transform は `translateX(-100%)` へ反転（`Drawer.tsx:136-139`）。

状態: hover/pressed なし。focus はパネル `outline: none`、content `outline: none`（content の tabIndex=-1 は初期 focus の着地点）。

### 3.5 Motion
| トリガー | プロパティ | from → to | duration | easing | file:line |
|---|---|---|---|---|---|
| open | transform | end: `translateX(100%)`→`0`（start: `-100%`→`0`、RTL 反転） | `--duration-medium` = **300ms** | `--ease-standard` cubic-bezier(0.24,1,0.4,1) | `Drawer.tsx:118-123`, 141-149, 162-170（`@starting-style`） |
| close | transform | `0` → `translateX(±100%)`（開いた時の side を保持して戻る, `exitSideRef`） | 300ms | **ease-standard**（exit 用 easing ではない点に注意） | 同上 |
| scrim open | opacity | 0 → 1 | 300ms | ease-standard | `Drawer.tsx:177-196` |
| scrim close | opacity | 1 → 0 | 300ms | ease-standard | 同 |
| close 完了 | – | transitionend 待ち。duration が読めない時 250ms、さらに +50ms のバックストップで強制的に top layer から外し非表示 | | | `useDrawerDialogPresence.ts:38-45`, 207-242 |
| reduced motion | 全 transition | 0.01s（実質即時） | | | `Drawer.tsx:124-127`, 185-189 |
| 既に開いた状態でマウント | 0.01s（アニメなし） | | | | `openSettled` `Drawer.tsx:171-176` |

ネイティブ: パネルは `DialogRecipes::surface` の `enter`/`exit` style で translate x ±100%（パネル幅基準）を 300ms、scrim は scrim recipe の enter/exit で opacity 0↔1 300ms。閉じ中も中身を描画し続けること。

### 3.6 Behaviour & a11y
- モーダル（hasScrim）: `showModal()`、focus trap（`useFocusTrap`, 16 章）、scroll lock、`aria-modal=true`。非モーダル: `showPopover()`（manual）、trap なし、ページはスクロール・操作可能。
- 初期 focus: `[data-autofocus]` → なければ content div（tabIndex −1）＝パネル本体（最初のボタンではない）。
- 閉じた後: 開いた要素へ focus を戻す（exit 完了後）。
- Escape: layer スタック最上位の drawer のみ（兄弟は最後に開いたもの→順に剥がれる）。required は block。非モーダルでも Escape は効く。
- scrim クリック: hasScrim && info のみ。
- ロール: `dialog`（required かつモーダルは `alertdialog`）、名前 = label。
- テキスト: DrawerHeader の閉じる = `@astryx.dialog.close` "Close" → `soft.dialog.close` を共用。

### 3.7 Kivact 実装方針
- **RESTYLE `kivact::ui::dialog` の `DialogKind::Drawer(side)`（モーダル時）+ kit wrapper `soft::drawer`**。recipe: surface = bg surface・ページ側 1px border・shadow-high・幅 400/≤640 で min(400, w−56)・radius 0、enter/exit = translate ±100% 300ms ease-standard、scrim = overlay 色 + blur 2px（backdrop filter）・opacity 300ms。
- 非モーダル（`hasScrim=false`）は `DialogKind::Drawer` がモーダル固定のため、wrapper が `style().position(Position::Fixed)` + inset で固定パネルを描き、Escape は `use_dialog`（Popover kind 相当の dismiss）で扱う。外側押下では閉じない（Astryx でも非モーダルは scrim がないので外側クリックで閉じない）。
- 参考: `crates/kivact-ui-basic/src/recipes_dialog.rs`（sheet.tsx の Drawer kind recipe）, `crates/kivact-ui-basic/src/drawer.rs`（※スワイプ付きの別物。soft では Astryx に合わせてスワイプ無し）, `crates/kivact-ui-basic/src/layers.rs`。
- 難所: (1) 物理 Side ↔ 論理 start/end の RTL 変換。(2) 閉じアニメ完了まで描画継続（kivact dialog の exit style で可能か確認。不可なら UNRESOLVED）。(3) `100dvh` → ウィンドウ高。

### 3.8 受け入れ確認
- [ ] role=dialog 名前=label、モーダル時は背後操作不可・Tab がパネル内で循環。
- [ ] 開くと content（または data-autofocus）に focus、閉じるとトリガーへ戻る。
- [ ] 兄弟 drawer 2 枚 → Escape で後の 1 枚だけ閉じる。
- [ ] hasScrim=false でページがクリック・スクロール可能、Escape で閉じる。
- [ ] 幅 400、ウィンドウ 600px 幅で 544px（56px のページが見える）。
- [ ] 目視: 右から 300ms で滑り込む（減速カーブ）、閉じも 300ms で右へ。scrim 黒 50%/80% + ぼかし 2px がフェード。dark でパネル #262626 と左 1px 白 10% 線。

### 3.p1 DrawerHeader
`Drawer/DrawerHeader.tsx`（docs: https://astryx.atmeta.com/components/DrawerHeader）

DialogHeader（1.p1）と**同一寸法**: container flex/align-start/space-between/gap 12px、title-block column gap 0 / flex 1 / min-width 0 / margin-block −6px、title = h2（20px/28px 600）、subtitle = body sm 12px（line-height 1.4286 ≈ 17px）secondary（`#474747`/`#9e9e9e`）、end slot gap 8px + close 時 margin-block −8px / margin-inline-end −8px、close = ghost md icon-only 32×32 radius 10、tooltip/name "Close"（`@astryx.dialog.close`）。

DialogHeader との差分:
- タイトルは**名前にならない**（Drawer の `label` が名前）。タイトルに id/tabIndex/自動 focus なし（`DrawerHeader.tsx` Heading level 2 のみ）。
- props は同じ: `title`, `subtitle`, `onOpenChange`（close 表示）, `startContent`, `endContent`, `endContentEdgeCompensation`, `hasDivider`。

Kivact: `soft::drawer(..).header(DrawerHeader{..})` もしくは wrapper メソッド `.title(text)`, `.subtitle(text)`, `.closable(bool)`, `.header_start(|| ..)`, `.header_end(|| ..)`, `.header_divider(bool)`。kivact dialog の title を表示用にし、名前は `.name(label)` で上書きする。

受け入れ: × 押下で閉じる、タイトルは名前に含まれない（role=dialog の name は label）。

## 4. BottomSheet

docs: https://astryx.atmeta.com/components/BottomSheet ・ source: `packages/core/src/BottomSheet/`（`BottomSheet.tsx` 1242 行, `BottomSheetPanel.tsx` 937, `useSheetGestures.ts` 1869, `snapOffsets.ts`, `bottomSheetStack.ts` 2000, `useMobileKeyboard.ts`, `bottomSheetEscape.ts`, `BottomSheetEdgeTint.tsx`, `BottomSheetHostContext.ts`）

### 4.1 概要
画面下端からせり上がるモバイル／タッチ向けシート（フィルタ、共有、クイックアクション、フォーム、詳細）。グラブハンドル、ドラッグで高さを変えるスナップ点（detent）、スワイプで閉じる、`purpose` による閉じ方制御、シートの積み重ね（stack）と置き換え（replace）を持つ。Dialog と同じく padding 付きコンテナ（既定 16px）。

- 使う: モバイル優先の面。ドリルインは次のシートを上に重ねる（各シートの isOpen はデータの近くに置く）。高さは内容に合わせて `hug`（短い内容）/`capped`（リスト）/`tall`（フォーム・ストリーミング）。入力保護は `purpose=form`。
- 使わない: 長すぎる内容（ステップごとに別シートにし、1 つの state 値で開いているステップを表す → replace）。3 段を超える積み重ね（深い段は 2 段目の後退量を共有し、1 枚ずつしか閉じられない）。

### 4.2 Anatomy
```
<dialog aria-label=label [aria-modal=true if hasScrim&&open] [role=alertdialog if required]>   ← 全画面 fixed, 透明, display grid / align-content end / justify-items center
  ::backdrop                                       ← scrim（overlay 色, opacity = --_sheet-scrim-opacity）
  <div panel tabIndex=-1 (ref/DOM props はここ)>     ← styles.sheet：max-width 640, 上 2 角 radius 12, 3 辺 hairline
     <div handleBar aria-hidden>  ::before = pill   ← 絶対配置 24px 帯, ドラッグ専用（a11y なし）
     <div body (scroll, focus-visible ring, 名前付きスクロール領域)>
        <div content (flow-root, container padding 16)>  children
  <BottomSheetEdgeTint aria-hidden>                ← iOS Safari ツールバー着色用の不可視帯（ネイティブ不要）
```
スタック時は dialog（positioner）自体に recede transform が乗る。

### 4.3 API
| prop | 型 | 既定 | 意味 | Kivact |
|---|---|---|---|---|
| `isOpen` / `onOpenChange` | | 必須（Switcher 内を除く） | 制御 | `soft::bottom_sheet(label, open: Binding<bool>)` |
| `label` | string | 必須 | aria-label（見出しから名前を取らない） | 第 1 引数 |
| `children` | ReactNode | 必須 | スクロール領域内の内容 | `.content(|| ..)` |
| `purpose` | required\|form\|info | info | info: Escape・scrim タップ・スワイプで閉じる / form: Escape のみ / required: どれも不可 + role alertdialog | `.purpose(DialogPurpose)` |
| `height` | `'hug'\|'capped'\|'tall'\|number\|string` | `'capped'` | hug=内容に合わせ最大 92%、capped=62%（スクロール）、tall=92% 固定 | `.height(SheetHeight::Hug|Capped|Tall|Px(f32)|Fraction(f32))` |
| `snapPoints` | `(number\|string)[]` | なし（開閉のみ） | 追加の静止高さ。number=ビューポート比 (0,1]、`'50%'`、`'320px'`。シート自身の高さが常に最上段 | `.snap_points([SnapPoint::Fraction(0.5), SnapPoint::Px(96.0)])` |
| `hasScrim` | bool | true | true=モーダル（scrim, focus 封じ込め, scroll lock, scrim タップで閉じる, stack 参加）/ false=非モーダル（scrim/trap/lock なし, stack 不参加） | `.scrim(bool)` |
| `padding` | SpacingStep | 4 (16px) | content padding。0 で端まで | `.padding(step)` |
| `finalFocusRef` | RefObject | – | 閉じた時の focus 戻り先の明示 | `.final_focus(NodeRef)` |
| `sheetId` | string | – | 非推奨（Switcher 用） | 5 章参照。単体シートでは不要 |
| `ref`/`xstyle`/`className`/`style` | | | panel div に付く | `NodeBuilder` 修飾子は panel に適用 |

コールバック追加案（Astryx 内部の onSnap 相当）: `.on_snap(|visible_height_px| ..)` は任意。

### 4.4 Visual spec
panel（`BottomSheetPanel.tsx:131-202`）:

| 項目 | 値 | light | dark |
|---|---|---|---|
| 幅 | 100%、`max-width: 640px`、水平中央 | | |
| 高さ | `--_sheet-budget` + 48px（下に 48px はみ出させる overscroll 予約）。budget: hug = fit-content（max 92% + 48）, capped = 62% of ビューポート高, tall = 92%, 数値 = px, 文字列 = CSS 長（`BottomSheetPanel.tsx:76-80`, 446-453）。短いビューポートでは使える高さいっぱい | | |
| margin-block-end | −48px、padding-block-end = 48px + safe-area-bottom（`BottomSheetPanel.tsx:166-167`）→ 上へ引っ張った時に下が見えないための予約 | | |
| background | `--color-background-surface` | `#ffffff` | `#262626` |
| border | 上・左右の 3 辺に 1px `--color-border`（下辺なし） | `#00000014` | `#FFFFFF1A` |
| radius | 上 2 角 `--radius-container` 12px、下 2 角 0 | | |
| shadow | `--shadow-high` | `0 4px 6px rgba(0,0,0,.1), 0 12px 24px rgba(0,0,0,.15)` | `0 4px 6px rgba(0,0,0,.5), 0 12px 24px rgba(0,0,0,.7), inset 0 0 0 1px rgba(255,255,255,.15)` |
| overflow | hidden, outline none | | |

handle bar（`BottomSheetPanel.tsx:216-248`）:
- 絶対配置で panel 上端に重なる帯、高さ `--spacing-6` = 24px、左右いっぱい、z-index 1、中央寄せ。
- 背景: `linear-gradient(to bottom, surface 60%, transparent)`（light `#ffffff` → 透明 / dark `#262626` → 透明）。下にスクロールしてきた内容が切れ目なく消える。
- pill（`::before`）: 幅 `--size-element-lg` = 36px、高さ `--spacing-1` = 4px、radius full、色 `--color-border`（light `#00000014` / dark `#FFFFFF1A`）。上端から 10–14px の位置。
- cursor: grab。touch-action: pinch-zoom（1 本指ドラッグはシート、ピンチはブラウザ）。

body（`BottomSheetPanel.tsx:249-271`）: flex-grow 1、min-height 0、背景 surface（dark テーマの inset リムを均等に隠す）、focus-visible は内側オフセットの focus ring（2px accent, offset −2px: light `#1b1b1b` / dark `#f1f1f1`）。内容はシート上端から始まりハンドルの下に潜る（予約なし）。content は `display: flow-root; height 100%; min-height: min-content` + padding 16px。

scrim（`BottomSheet.tsx:166-185`）: `--color-overlay` light `#00000080` / dark `#000000CC`、**blur なし**（Dialog/Drawer と違う）、opacity = `--_sheet-scrim-opacity`（ドラッグ・peek で変化, 下表）。

dialog（`BottomSheet.tsx:128-154`）: 全画面 fixed、透明、`display: grid; align-content: end; justify-items: center`。panel の横の空き部分タップは dialog に当たり light dismiss 判定。

```
          ┌──────────── ≤640 ─────────────┐   ← 上 2 角 12px, 3 辺 1px border
          │            ▬▬▬▬ (36×4)        │   ← handle bar 24px（グラデ背景）
          │ content (padding 16)          │
          │ ...                           │
          │                               │
══════════╧═══════════════════════════════╧══ viewport bottom（下に 48px 予約が隠れている）
```

スナップ点（`snapOffsets.ts`）:
- 各点の可視高さ（px）を求め、シート全高より短いものだけ「offset = 全高 − 可視高」に変換、0（全開）を先頭に昇順。互いに **48px 未満**の offset は 1 つにまとめる（小さい方＝高い方を残す, `DETENT_DEDUP_PX = 48`）。
- **peek**: 最短の静止点の可視高さが全高の **25% 以下**（`PEEK_MAX_HEIGHT_RATIO = 0.25`）なら peek。peek は内容をリフローせず全高のまま下へスライドするだけ（transform のみ）。他の detent はレイアウト高さも縮める（静止時に 1 回だけ、遷移なしで）。
- scrim 不透明度（`scrimOpacityForOffset`）: peek あり → 下から 2 番目の detent までは 1、peek で **0.3**（`MIN_PEEK_SCRIM_OPACITY`）、間は線形。peek なし → 最短 detent までは 1、そこから dismiss 線（下記）まで線形に 0。

スタック（`BottomSheet.tsx:225-249`）: 覆われたシートは上端中心（transform-origin 50% 0）で縮小・上昇:
| 深さ | transform |
|---|---|
| 1（直下） | `translateY(-8px) scale(0.96)` |
| ≥2 | `translateY(-16px) scale(0.92)`（以降は全部この値を共有） |
scrim はスタックの一番下のシートだけが描く（上のシートの backdrop は透明）。下のシートが閉じても同フレームで次のシートが scrim を引き継ぐ（フェードなし: `--_sheet-scrim-duration` を 0s に固定）。

RTL: 左右対称なので反転不要（border の inline 辺は論理値）。

### 4.5 Motion
| トリガー | プロパティ | from → to | duration | easing | file:line |
|---|---|---|---|---|---|
| open | transform | `translateY(100%)` → `0` | `--duration-medium` **300ms** | `--ease-standard` (0.24,1,0.4,1) | `BottomSheetPanel.tsx:171-191`（@starting-style） |
| close（pointer） | transform | `0` → `translateY(100%)` | 300ms | `--ease-exit` cubic-bezier(0.3, 0, 0.6, 0.6) | `BottomSheetPanel.tsx:210-235` |
| close（coarse pointer） | transform | 同上 | 300ms | cubic-bezier(0.3, 0, 0.6, 0.6)（固定値） | 同上 |
| scrim open | opacity | 0 → `--_sheet-scrim-opacity`(1) | 300ms | ease-standard | `BottomSheet.tsx:166-185` |
| scrim close | opacity | → 0 | 300ms | **linear**（シートが滑り出す間ずっと薄くなる） | `BottomSheet.tsx:193-205` |
| ドラッグ中 | transform | 指に追従（transition none） | 0 | – | `useSheetGestures.ts:1794-1797` |
| リリース後の settle | transform | 現在位置 → 目標 detent | 300ms | ease-standard（panel の transition） | 同上 |
| 上方向オーバースクロール | transform | raw × **0.35**、最大 **−48px**、離すと 0 に戻る（300ms ease-standard） | | | `useSheetGestures.ts:117-121`, 1267-1270 |
| detent 近傍マグネット | 位置 | detent から **40px** 以内で `pull = 1 − (d/40)²` だけ detent へ引き寄せ（dismiss ゾーンでは無効） | | | `useSheetGestures.ts:99-101`, 152-170 |
| stack recede / return | transform | 0 ↔ 深さ別 transform（上表） | 300ms | ease-standard | `BottomSheet.tsx:225-237` |
| replace（置換） | 新シート: 通常の open（下から 300ms）。旧シートが新シートより高い場合: 同時に `translateY(alignmentOffset)` で下げて上端を揃える（300ms ease-standard）。両方終わったら旧シート opacity 1 → 0（300ms ease-standard）。scrim は一定（凹みなし）。旧シートは最初から inert / aria-hidden | | | | `BottomSheetPanel.tsx:203-208`, `BottomSheet.doc.mjs` Replacing |
| 遷移の完了検出 | transitionend 待ち + バックストップ (duration + 50ms) | | | | `BottomSheetPanel.tsx:88`, waitForTransition |
| haptic | detent に settle し base と違う時 `vibrate(8)`（reduced motion 時は無し） | | | | `useSheetGestures.ts:141-148` |
| reduced motion + coarse | transform なし、opacity クロスフェード **200ms**（入場 0→1、退場 1→0）。transform の遷移は 0.01s | | | | `BottomSheetPanel.tsx:122-129`, 171-196, 230-241 |
| reduced motion + fine | 即時（0.01s） | | | | 同上 |
| ページロード時に開いている | 0.01s（アニメなし） | | | | `sheetSettled` 198-202, `scrimSettled` 187-192 |

### 4.6 Behaviour & a11y
ジェスチャ（`useSheetGestures.ts`）— 全定数:
- 開始: ハンドル上の pointerdown で即ドラッグ開始。body 上では「指の下のスクロール要素が全部上端にある」時だけ、下方向に **8px**（`DRAG_PROMOTION_SLOP`）を超えたらドラッグに昇格。上方向に動いたら解除（スクロールに任せる）。スクロール中に上端へ到達して更に引いた場合は **4px**（`CONTENT_END_HANDOFF_SLOP`）超過後にシートへ引き継ぐ。
- 2 本目の指 = ピンチ → ドラッグ終了（ズームはブラウザ）。
- 速度: 直近 **100ms** 窓のサンプルで `(y − y_first) / span` px/ms。離した時点で最後のサンプルが 100ms より古ければ速度 0。
- 投射: ドラッグ移動量 > **48px**（`PROJECTION_MIN_TRAVEL`）なら `projected = offset + v × 499ms`（`0.998 / (1 − 0.998)`、UIKit の通常減速率）。それ以下は投射なし。
- dismiss 線: `dismissOffset = 最短 detent の offset + 最短 detent の可視高 × 0.4`（`DISMISS_OVERSHOOT_RATIO`）。下方向（dir>0）で projected > dismissOffset → 閉じる（purpose が許せば。許さなければ最短 detent に戻す）。
- それ以外: `resolveSettleOffset` — ドラッグ方向の detent だけを候補に（下ドラッグなら開始位置以下、上なら以上）、`clamp(projected, 0, 最短 offset)` に最も近いもの。
- リサイズ・回転・snapPoints 変更時は「選んでいた detent の index」を保って位置を再計算（アニメなし）。

キーボード / フォーカス:
- 開く: モーダル dialog の既定フォーカス手順（panel tabIndex −1）。置換で入ってくるシートは `[data-autofocus]` を優先。
- body はスクロール可能な間だけキーボードで到達できる名前付き領域（label を再利用）。
- Escape・scrim タップ・下スワイプ・プラットフォームの閉要求は**スタック最上位のシートにだけ**届き、そのシートの purpose が適用。スタック全体を一度に閉じる操作は無い。
- 閉じる: 最上位を閉じると、露出したシートの中の「開いた元のコントロール」へ（無ければその panel）。最後の 1 枚なら `finalFocusRef` → 開いた要素（キーボード操作後のみ focus ring）。
- ハンドルは aria-hidden、キーボードでの高さ変更手段なし（Astryx 仕様）。
- tall かつ全開のシートだけモバイルキーボード対応（入力が見えるようスクロール、下に 48px のクリアランス `MOBILE_KEYBOARD_BOTTOM_CLEARANCE`）。ネイティブデスクトップでは不要。
- ロール: dialog（required は alertdialog）、名前 = label、モーダル時 aria-modal。
- 既定テキストなし（i18n キー不要）。

### 4.7 Kivact 実装方針
- **NEW kit component `soft::bottom_sheet`**。土台: `kivact::interaction::use_dialog`（モーダル・dismiss・focus 戻し）+ `use_drag`（ハンドル / body のドラッグ）+ `overlay`（modal(true)）。panel の位置は transform の y を signal で持ち、ドラッグ中は直接、離したら 300ms ease-standard の補間（kivact-motion の tween/transition で）で目標 detent へ。
- 参考: `crates/kivact-ui-basic/src/drawer.rs`（距離と速度で閉じる判断をする既存スワイプドロワー。最も近い）, `recipes_dialog.rs`, `layers.rs`。
- body のスクロールとドラッグの受け渡し（スクロール位置が 0 の時だけ下ドラッグをシートへ）は kivact の scroll ノードのオフセットを読んで判定。難しければ「ハンドル帯（上 24px）だけでドラッグ」にフォールバック（UNRESOLVED: body からのドラッグ昇格の可否）。
- スタック・置換・scrim の引き継ぎは kit 内のグローバルな sheet stack（thread_local の Vec）で表現。recede transform（−8px/0.96, −16px/0.92）を下のシートに 300ms で適用。
- 不要: EdgeTint（iOS Safari 専用）、safe-area、dvh、モバイルキーボード処理（デスクトップネイティブ）。
- 難所: (1) scrim opacity をドラッグに連動させる（scrim を kit 自前の fixed ノードで描けば容易）。(2) peek の「レイアウト高さを変えず transform だけ」は transform 一本でやれば自然に満たせる（detent ごとにレイアウトを縮めるのは静止時に 1 回）。(3) haptic は無視可。

### 4.8 受け入れ確認
- [ ] role=dialog 名前=label。開くと panel に focus、閉じると開いた要素へ戻る。
- [ ] 下から 300ms で上がる（減速）、閉じは 300ms で下へ（加速カーブ）、scrim は閉じで線形に消える。
- [ ] ハンドルを 50% detent 付近まで下げて離すと 50% に吸着。速く下に投げると閉じる。ゆっくり最短 detent の 40% 超まで下げると閉じ、未満なら戻る。
- [ ] form: スワイプ・scrim タップで閉じない、Escape で閉じる。required: すべて閉じない。
- [ ] 上に引くと最大 48px だけ抵抗付き（0.35 倍）で伸び、離すと戻る。
- [ ] 2 枚目を開くと 1 枚目が上へ 8px・0.96 倍に後退、scrim は 1 枚分の濃さのまま。
- [ ] peek（`['96px', '50%']` on tall）で scrim が 0.3 まで薄くなる。
- [ ] 目視: 幅 640 上限、上 2 角 12px、36×4 の pill（light 8% 黒 / dark 10% 白）、dark で 3 辺 hairline。

## 5. BottomSheetSwitcher

docs: https://astryx.atmeta.com/components/BottomSheetSwitcher ・ source: `BottomSheet/BottomSheetSwitcher.tsx`（1082 行）, `BottomSheetSwitcherContext.ts`

### 5.1 概要
**非推奨**。複数の BottomSheet（`sheetId`）を相互排他のフローとして束ね、`activeSheet` の ID で 1 枚だけを対話可能にする。現在は「各ステップを独立 BottomSheet にし、1 つの state 値で開いているステップを表す」方式が推奨で、同じ置換ハンドオフ（4.5 replace）が得られる。

### 5.2 Anatomy
```
<div flow display:contents>  （scrim フロー） / <div flowNonModal fixed inset0 isolation:isolate z-index 1000 pointer-events none>（非モーダル）
  BottomSheet sheetId="a"  → 各自 <dialog>
  BottomSheet sheetId="b"
```

### 5.3 API
| prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `activeSheet` | string\|null | 必須 | `Binding<Option<K>>` |
| `onActiveSheetChange` | (string\|null)=>void | 必須 | binding 書き込み |
| `hasScrim` | bool | true | `.scrim(bool)`（全ステップ共通） |
| `children` | BottomSheet[] (sheetId) | 必須 | `.step(key, label, || content)` の列 |
| `onCancel` | (event)=>void | – | 削除（各ステップの purpose で代替） |
| `ref` | Ref<HTMLDialogElement> | – | 削除 |

### 5.4 Visual spec
各ステップは BottomSheet と完全に同じ見た目。非モーダル時の wrapper は z-index 1000、isolation isolate（`BottomSheetSwitcher.tsx:98-119`）。

### 5.5 Motion
ステップ切替 = BottomSheet の replace: 新ステップが下から 300ms ease-standard で入場、旧ステップが新より高ければ同時に下がって上端を揃え（300ms ease-standard）、両方完了後に旧ステップを 300ms でフェードアウト。scrim は一定。reduced motion は BottomSheet の規則に従う。

### 5.6 Behaviour & a11y
- 対話可能なのは active の 1 枚。旧ステップは inert / aria-hidden。
- active シートの purpose に従った自動 dismiss で `onActiveSheetChange(null)`。
- フロー全体が stack の 1 段として扱われ、覆われると丸ごと後退。
- focus はフローを開いた要素へ戻る（どのステップで閉じても）。

### 5.7 Kivact 実装方針
**covered by BottomSheet**（置換ハンドオフを `soft::bottom_sheet` に実装し、複数シートを 1 つの `Signal<Option<Step>>` から `open = move || step.get() == Some(id)` で開く）。互換のための別コンポーネントは作らない方針を推奨（Kivact は未公開で互換不要）。どうしても必要なら `soft::bottom_sheet_flow(step: Binding<Option<K>>)` の薄いラッパーで、内部は独立シートの列。

### 5.8 受け入れ確認
- [ ] ステップ A の「Continue」で B に置換: B が下から上がり、A は（A が高ければ）下がって揃った後フェード。scrim 一定。
- [ ] B で Escape → フロー全体が閉じ、focus はフローを開いたボタンへ。

## 6. Popover

docs: https://astryx.atmeta.com/components/Popover ・ source: `packages/core/src/Popover/`（`Popover.tsx` 1105 行, `usePopover.tsx` 671, `lazyPopoverBottomSheet.ts`）, 位置決めは `Layer/useLayer.tsx`（15 章）

### 6.1 概要
ボタン等のトリガーに付くクリック開閉のオーバーレイ（button + dialog パターン）。副次アクション、インライン確認、補足情報など、フル Dialog に値しない対話的内容。ホバープレビューは HoverCard、短い補助文は Tooltip。

- 使う: 1 タスク / 1 情報に絞る。閉じ方を明確に（外側クリック or 明示ボタン）。テーマは popover ターゲット経由。
- 使わない: Popover の入れ子。入力の複雑さだけで presentation を決めること。スクロールがある＝不適と決めつけること（上限付きでスクロールしてよい）。

### 6.2 Anatomy
```
<span anchorWrapper display:inline-flex>  ← アンカー（押下 scale がアンカー位置を揺らさないよう子ではなく wrapper を anchor にする）
  trigger（button / [role=button]）  aria-haspopup="dialog" aria-expanded aria-controls=<id>
<div popover layer (useLayer; position-area, fallbacks, gutter, animation)>
  <div role="dialog" aria-modal=true aria-label=label tabIndex=-1 surface (bg popover, radius 12, shadow-low, padding 12)>
     content
     <div fallback-close sr-only> <Button secondary "Close popover"> </div>   ← Tab で到達した時だけ表示（サーフェス下に中央寄せで出る）
```
`presentation='bottom-sheet'`/`'adaptive'` の時は content を BottomSheet（4 章）に入れて出す。

### 6.3 API
| prop | 型 | 既定 | 意味 | Kivact |
|---|---|---|---|---|
| `children` | ReactNode | – | トリガー（button を含むこと） | `.attach(trigger)`（既存 `Dialog::attach`） |
| `anchorRef` | RefObject | – | 兄弟モードの外部アンカー | `.anchor(NodeRef)` 新設（kit wrapper） |
| `content` | ReactNode | 必須 | 内容 | `.content(|| ..)` |
| `placement` | above\|below\|start\|end | `'below'` | 論理配置 | `.placement(Placement::Below)`（Start/End は reading direction で解決: kivact 既存） |
| `alignment` | start\|center\|end | `'start'` | 揃え | `.align(OverlayAlign::Start)` |
| `isOpen` / `onOpenChange` | | 非制御可 | | `dialog(label, open: Binding<bool>)`（Kivact は常に caller-owned signal） |
| `isEnabled` | bool | true | false でトリガー無効 | トリガー側 `.disabled(..)` |
| `width` | number\|string | `'auto'`（= トリガー幅以上, ビューポート上限） | 幅 | `.width(Length)`。既定は `at_least_trigger_width` 相当 |
| `padding` | SpacingStep | `3`（12px） | サーフェス内 padding | `.padding(step)` |
| `label` | string | – | dialog の名前 | 第 1 引数（title, `.hide_title(true)` で非表示） |
| `role` | `'dialog'\|'none'` | `'dialog'` | none は中身が menu/listbox 等の役割を持つ場合 | `.role_none(true)` → kit 側で Role::Group/なし（UNRESOLVED: kivact dialog が role 無しを許すか） |
| `isModal` | bool | true | aria-modal（role=dialog 時） | 視覚無関係。`Dialog` の Popover kind は非モーダル。a11y 属性としてのみ `.modal_semantics(bool)` |
| `hasCloseButton` | bool | true | 隠し閉じるボタン | `.fallback_close(bool)` |
| `closeButtonLabel` | string | "Close popover" | その名前 | `soft.popover.close` = "Close popover" |
| `hasAutoFocus` | bool | true | 開いた時に最初の本物のコントロールへ focus（無ければサーフェス） | `.auto_focus(bool)` / `.initial_focus(NodeRef)`（既存） |
| `hasLightDismiss` | bool | true | 外側クリックで閉じる | `.dismiss(Dismiss{outside, ..})` |
| `hasEscapeDismiss` | bool | true | Escape で閉じる（light dismiss off の時のみ完全に効く） | `.dismiss(Dismiss{escape, ..})` |
| `presentation` | popover\|bottom-sheet\|adaptive | `'popover'` | 面の種類。adaptive = 粗いポインタ かつ コンパクト画面（テーマ md 幅未満 or 高さ < 500px）で BottomSheet | `.presentation(Presentation::Popover|BottomSheet|Adaptive)`。デスクトップネイティブでは Adaptive = Popover（UNRESOLVED: タッチ判定 API） |
| `xstyle`/`className`/`style` | | | サーフェスに | `.style(..)` |

`usePopover`（hook）: `{triggerRef, contentRef, anchorId, show({skipAutoFocus}), hide, toggle, isOpen, id, render(children, props), triggerProps}`、options = `onShow, onHide, xstyle, className, style, hasLightDismiss, hasEscapeDismiss, hasAutoFocus, hasCloseButton, closeButtonLabel, dialogLabel, role, isModal, hasSurface(=true), padding, surfaceTarget`。Kivact では `soft::use_popover()` → `kivact::interaction::use_dialog(DialogKind::Popover)` のハンドルに surface recipe を被せる薄い関数として提供（`trigger()` part と `render(|| ..)`）。`hasSurface=false` は「位置決めだけ」= `overlay(anchor, placement, ..)` 直接利用。

### 6.4 Visual spec
サーフェス（`usePopover.tsx:79-87`, padding `Popover.tsx:513`）:

| 項目 | 値 | light | dark |
|---|---|---|---|
| background | `--color-background-popover` | `#ffffff` | `#1b1b1b` |
| radius | `--_popover-radius` = `--radius-container` | 12px | 12px |
| shadow | `--shadow-low` | `0 2px 4px rgba(0,0,0,.05), 0 4px 8px rgba(0,0,0,.10)` | `0 2px 4px rgba(0,0,0,.25), 0 4px 8px rgba(0,0,0,.40), inset 0 0 0 1px rgba(255,255,255,.08)` |
| border | なし | | |
| padding | `--spacing-3` 12px 全辺（`padding` prop で 0/2/4/8/12/16/20/24/32/40px） | | |
| 文字 | body 14px/20px 400 text-primary（`#000000` / `#ffffff`）Figtree（layerTextReset） | | |
| 幅 | 既定: `min-width = min(トリガー幅, ビューポート幅 − 2×gutter)`、内容に合わせて伸びる。`width` 指定時はその値（ビューポート上限まで） | | |
| 最大 | max-inline = ビューポート幅 − 2×16px、max-block = ビューポート高 − 2×16px（gutter = `--spacing-4`, 15.3 参照）。溢れたら `overflow: auto; overscroll-behavior: contain`（`Popover.tsx:428-440`） | | |
| アンカーとの距離 | `offset = --spacing-1` = **4px**（`Popover.tsx:1055`） | | |

隠し閉じるボタン: 通常 1×1 clip。focus-within で表示: サーフェス下端の外側に中央寄せ（`translateY(100%)`）、上 padding 4px、Button secondary md（高さ 32, 背景 `--color-neutral` light `#0000000F` / dark `#FFFFFF1A`、文字 text-primary、label 14px/500、radius 10px, padding 8/12）。

focus: サーフェス（tabIndex −1）に focus-visible リング 2px accent（light `#1b1b1b` / dark `#f1f1f1`）offset 3px（`focusOutlineProps.focusVisible`）。

```
 [Open popover ▾]
 ┌──────────────────────┐   ← 4px 下, start 揃え, min-width = トリガー幅
 │ 12  content       12 │   radius 12, shadow-low, bg #fff / #1b1b1b
 └──────────────────────┘
```

配置・フリップ: 15 章の useLayer ルール（placement below / alignment start → `position-area: self-block-end span-self-inline-end`、fallbacks: flip-block → flip-inline → 両方 → slide same → slide opposite）。

### 6.5 Motion
| トリガー | プロパティ | from → to | duration | easing | file:line |
|---|---|---|---|---|---|
| open（placement=below） | opacity, transform | `0; translateY(-8px) scale(0.95)` → `1; translateY(0) scale(1)` | `--duration-fast-max` **165ms** | `--ease-standard` (0.24,1,0.4,1), fill backwards | `Layer/layerAnimations.stylex.ts:30-36`, 92-101; `Popover.tsx:1006` |
| open（above） | | `translateY(+8px) scale(0.95)` → 0/1 | 165ms | 同 | 38-44 |
| open（end / start） | | `translateX(∓8px)`（end: −8px から, start: +8px から。RTL で符号反転）scale 0.95 → 1 | 165ms | 同 | 46-81 |
| transform-origin | 既定（中央）。UNRESOLVED ではなく CSS 既定 50% 50% | | | | |
| close | – | **退場アニメなし**（即時非表示） | 0 | – | layerAnimations は入場のみ |
| reduced motion（fine） | – | 即時 | | | 113-152 |
| reduced motion（coarse） | opacity | 0 → 1 | `min(165ms, 200ms)` = 165ms | ease-standard | 同上 |
| bottom-sheet presentation | BottomSheet の motion（4.5） | | | | |

### 6.6 Behaviour & a11y
- トリガー: click / Enter / Space（role=button 要素は Enter/Space を自前で click 化, `Popover.tsx:840-846`）でトグル。`aria-haspopup="dialog"`, `aria-expanded`, `aria-controls`。aria-disabled のトリガーは focus 可能だが開かない。
- 開く: `hasAutoFocus` → content 内の最初の本物のコントロール（隠し close を除く）→ 無ければサーフェス自体。マウス起点でも input 保持したい時は `show({skipAutoFocus})`。
- focus trap: 開いている間 Tab はポップオーバー内で循環（`useFocusTrapInternal`）。最後のコントロールの次に隠し Close が現れる。
- 閉じる: 外側押下（hasLightDismiss）、Escape（layer スタック最上位のみ）、隠し Close。閉じたら開いた時の要素へ focus を戻す。
- ロール: `dialog`（aria-modal=true が既定）、名前 = label（無ければ dev 警告）。role=none なら属性なし。
- テキスト: `soft.popover.close` = "Close popover"。

### 6.7 Kivact 実装方針
- **RESTYLE `kivact::ui::dialog`（`DialogKind::Popover`）+ kit wrapper `soft::popover(label, open)`**。surface recipe（Popover kind 時）: bg popover, radius 12, shadow-low, padding 12, offset 4, min-width = トリガー幅（`at_least_anchor_width(true)`）, max = window − 32 で内部スクロール。enter style = 165ms の opacity+translate 8px+scale 0.95（配置方向に依存 → recipe が `DialogState` の実際の配置側を読めるか要確認。読めなければ placement 引数から固定）。exit は即時。
- フリップ/シフト: kivact の overlay ホストが flip と window 内クランプを持つ（`kivact-core/src/view/overlays.rs:67-90`）。gutter 16px（ウィンドウ端から）は kivact のクランプに余白指定が無ければ UNRESOLVED（15.7 参照）。
- 隠し Close: kivact dialog の close ボタン（closable）を「focus 時のみ表示」の recipe にする（close recipe が focus 状態を受け取れるなら opacity/clip 切替）。
- 参考: `crates/kivact-ui-basic/src/recipes_dialog.rs`（popover.tsx の Popover kind）, `recipes_popup.rs`。
- presentation adaptive/bottom-sheet: `soft::bottom_sheet` に content を渡す分岐（kit wrapper 内）。

### 6.8 受け入れ確認
- [ ] トリガー押下で `role=dialog` 名前=label が開き、最初のコントロールに focus。再押下・Escape・外側押下で閉じ、トリガーへ focus が戻る。
- [ ] Tab を最後まで送ると "Close popover" ボタンが下に現れ、押すと閉じる。
- [ ] 下端近くのトリガーでは上にフリップ、右端では end 揃えにフリップ / スライドして画面内（端から 16px）。
- [ ] 目視: トリガー下 4px、角丸 12、影が薄い（shadow-low）、dark は #1b1b1b に 8% 白リム。入場 165ms で上から 8px 降りつつ 0.95→1。閉じは即時。

## 7. HoverCard

docs: https://astryx.atmeta.com/components/HoverCard ・ source: `packages/core/src/HoverCard/`（`HoverCard.tsx` 456 行, `useHoverCard.tsx` 741）

### 7.1 概要
ホバーまたはフォーカスで出る補足カード（プロフィール、リンク要約、インライン定義）。ページ遷移なしに文脈を足す。

- 使う: 補足に留める。テキストトリガーには破線下線。細かい制御は useHoverCard。touchTrigger は auto のまま。ブロック文脈に置く（`<p>`/見出し/リンク直下は補正 portal になり DOM・Tab 順が変わる）。
- 使わない: 重要な操作・必須情報を入れる。Tooltip や Popover で足りる場合。操作必須の内容（カーソルが離れると消える）。

### 7.2 Anatomy
```
<span|div wrapper display:contents|inline>   ← children（テキストなら破線下線）
<div layer role="dialog" aria-label=label | role="group"  surface (bg surface, radius 12, shadow-med)>
   <div content padding 12>  content（Body / 任意の Actions）
```

### 7.3 API
| prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `children` | ReactNode | 必須 | `.attach(trigger)`（kivact-ui-basic hover_card と同形） |
| `content` | ReactNode | 必須 | `soft::hover_card(|| content)` |
| `placement` | above\|below\|start\|end | `'above'` | `.placement(Placement::Above)` |
| `alignment` | start\|center\|end | `'center'` | `.align(OverlayAlign::Center)` |
| `delay` | number ms | **300** | `.open_delay(Duration::from_millis(300))` |
| `hideDelay` | number ms | **200** | `.close_delay(Duration::from_millis(200))` |
| `focusTrigger` | auto\|always\|never | `'auto'`（自然に focus 可能な要素のみ） | `.focus_trigger(FocusTrigger::Auto|Always|Never)` |
| `touchTrigger` | auto\|tap\|none | `'auto'` | `.touch_trigger(TouchTrigger)`（デスクトップネイティブでは影響小） |
| `isEnabled` | bool | true | `.enabled(impl IntoValue<bool>)` |
| `label` | string | – | 指定時 role=dialog + 名前、無指定は role=group | `.label(text)` |
| `onOpenChange` | fn | – | `.on_open_change(f)` |
| `hasHoverIndication` | 'auto'\|bool | 'auto'（children が文字列/数値なら下線） | `.hover_indication(bool)`（テキストトリガー用 helper `soft::hover_card_text(text)` を推奨） |
| `isDefaultOpen` | bool | – | `.default_open(true)` |
| `isOpen` | bool | 非制御 | `.open(Binding<bool>)`：true で保持、false はホバーを抑止（focus では開く） |
| `xstyle`/`className`/`style` | | | サーフェスへ `.style(..)` |

`useHoverCard`（hook）: options = `placement, alignment, delay, hideDelay, focusTrigger, touchTrigger, isEnabled, label, onShow, onHide, isDefaultOpen, isOpen, xstyle...`、戻り値 = trigger ref/props, `render(children, props)`, `show/hide`, `isOpen`。Kivact: `soft::use_hover_card(cfg)` で trigger part と `render(|| ..)` を返す（`kivact::interaction::use_delayed` + overlay）。

### 7.4 Visual spec
（`useHoverCard.tsx` styles: container / content, render 640-665）

| 項目 | 値 | light | dark |
|---|---|---|---|
| background | `--color-background-surface` | `#ffffff` | `#262626` |
| radius | `--_hovercard-radius` = `--radius-container` | 12px | |
| shadow | `--shadow-med` | `0 2px 4px rgba(0,0,0,.05), 0 4px 12px rgba(0,0,0,.10)` | `0 2px 4px rgba(0,0,0,.35), 0 4px 12px rgba(0,0,0,.50), inset 0 0 0 1px rgba(255,255,255,.12)` |
| padding | `--spacing-3` 12px 全辺 | | |
| border | なし | | |
| offset | 4px（`--spacing-1`, `useHoverCard.tsx:646`） | | |
| 幅 | 内容幅（明示なし）。ビューポート − 32px 上限（useLayer viewportFit） | | |
| 文字 | body 14/20 text-primary | | |

テキストトリガーの hover indication（`HoverCard.tsx` styles.hoverIndication）: `text-decoration: underline dashed`, 色 `--color-border-emphasized`（light `#d4d4d4` / dark `#525252`）、`text-underline-offset: --spacing-0-5` = 2px。

```
          ┌──────────────────┐
          │12 Profile card 12│  ← above / center, 4px 上
          └────────┬─────────┘
             @username  (破線下線 #d4d4d4)
```

### 7.5 Motion
- 入場: layerAnimations[placement]（既定 above: `opacity 0; translateY(+8px) scale(0.95)` → `1; 0; 1`）、165ms、ease-standard、fill backwards（`useHoverCard.tsx:658`）。reduced: fine=即時、coarse=165ms フェード。
- 退場: アニメなし（hideDelay 200ms 経過後に即時消去）。
- 遅延: hover 入り → 300ms 後に表示。trigger/カードから離れる → 200ms 後に非表示。カード上に乗っている間は hide をキャンセル（`useHoverCard.tsx:668-683`）。

### 7.6 Behaviour & a11y
- ホバー: trigger の pointer enter で `scheduleShow(300)`、leave で `scheduleHide(200)`。カードへ移動すれば保持（WCAG 1.4.13 hoverable）。
- フォーカス: focusTrigger auto なら、自然に focus 可能なトリガーの focusin で**即時表示**（遅延なし, `useHoverCard.tsx:429-446`）。focus がカード内へ移る場合は閉じない。focus がトリガー/カード外へ出たら scheduleHide。
- タッチ（auto）: アクションを持たないトリガーはタップで開く、ボタン/リンク/フォーム部品はタップを部品に譲る（aria-disabled の部品は開く）。タップで開いたカードは外側タップで閉じる。
- Escape: layer スタック最上位なら閉じ、トリガーへ focus を戻す（再表示ガード付き）。制御モードでは `onOpenChange(false)` のみ呼ぶ。
- ロール: label あり → `role=dialog` 名前付き / なし → `role=group`。非モーダル、focus を奪わない（自動 focus なし）。
- テキストなし（kivact-ui-basic の `basic.hover_card.label` "Preview" 相当を名前に使うなら `soft.hover_card.label` = "Preview" を任意で用意。Astryx 既定は名前なし group）。

### 7.7 Kivact 実装方針
- **NEW kit component `soft::hover_card`**（kivact-ui-basic `hover_card.rs` と同形: content 関数を受け `.attach(trigger)` が trigger とオーバーレイを返す）。遅延は `kivact::interaction::use_delayed(Delays{open: 300ms, close: 200ms})`。overlay は `overlay(anchor, Placement::Above).align(Center).offset(4)`。
- 参考: `crates/kivact-ui-basic/src/hover_card.rs`, `recipes_popup.rs`。
- 差分: basic の hover card は Role::Group 固定。soft は label 指定時 Role::Dialog に。focus 即時表示（focus 時は open delay 0）。

### 7.8 受け入れ確認
- [ ] ホバー 300ms で表示、離れて 200ms で消える。カードへ移動すると消えない。
- [ ] Tab で trigger に focus → 即表示。Escape で閉じ、trigger に focus 維持、再表示しない。
- [ ] label 指定で role=dialog（名前付き）、未指定で role=group。
- [ ] 目視: trigger 上 4px、中央揃え、bg #fff / #262626、shadow-med、角丸 12、padding 12。テキスト trigger に破線下線（2px 下）。

## 8. Tooltip

docs: https://astryx.atmeta.com/components/Tooltip ・ source: `packages/core/src/Tooltip/`（`Tooltip.tsx` 405 行, `useTooltip.tsx` 767）, `Layer/useTouchTrigger.ts`, `Layer/usePressPin.ts`, `Layer/layerSlideRules.ts`（anchor-clear）

### 8.1 概要
ホバー／キーボードフォーカスで出る短い補助テキスト（アイコンボタンの名前、補足）。反転色の小さなラベル。対話的内容は Popover / HoverCard へ。

### 8.2 Anatomy
```
<span|div wrapper> trigger (aria-describedby=<tooltipId>; toggle 時 aria-expanded/aria-controls)
<div layer role="tooltip" id surface(inverted, radius 12)>   ← onMouseEnter=cancelHide, onMouseLeave=scheduleHide
   <div content padding 4/8, max-width 300, word-break break-word> text
```
矢印（arrow）は**無い**。

### 8.3 API
| prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `children` | ReactNode | – | `tooltip(text).attach(trigger)`（既存） |
| `anchorRef` | RefObject | – | `.anchor(NodeRef)`（kit wrapper） |
| `content` | ReactNode | 必須 | `tooltip(text)` / リッチは `.content(|| ..)`（既存） |
| `placement` | above\|below\|start\|end | `'above'` | `.placement(Placement::Above)`（既存） |
| `alignment` | start\|center\|end | `'center'` | `.align(OverlayAlign::Center)`（既存） |
| `delay` | ms | **200** | `TooltipConfig{delay: 200ms, skip_delay: 0}` を soft テーマ境界で `provide`（既定 500/300 から変更） |
| `hideDelay` | ms | **0**（実効 100ms の hover bridge） | `.hide_delay(Duration)` 新設、既定 100ms |
| `focusTrigger` | auto\|always\|never | `'auto'` | `.focus_trigger(..)` |
| `touchTrigger` | auto\|tap\|none | `'auto'` | `.touch_trigger(..)` |
| `triggerBehavior` | adaptive\|hover\|toggle | `'adaptive'` | `.behavior(TooltipBehavior::Adaptive|Hover|Toggle)`。Toggle = ボタンのクリック/Enter/Space で固定表示（pin）、同じ操作で閉じる |
| `isEnabled` | bool | true | `.enabled(..)` |
| `onOpenChange` | fn | – | `.on_open_change(f)` |
| `hasHoverIndication` | 'auto'\|bool | 'auto' | `.hover_indication(bool)`（破線下線: HoverCard と同値） |
| `isDefaultOpen` / `isOpen` | bool | – | `.open(Binding<bool>)` |
| `xstyle`/`className`/`style` | | | `.style(..)` |

`useTooltip`（hook）: `ref`, `describedBy`, `render(children, props)`, `show/hide`, `isOpen`, `triggerProps` 等。Kivact: `kivact::interaction::use_tooltip()`（既存）。

### 8.4 Visual spec
（`useTooltip.tsx:52-74`）

| 項目 | 値 | light | dark |
|---|---|---|---|
| background | `--color-text-primary`（反転） | `#000000` | `#ffffff` |
| text | `--color-background-surface` | `#ffffff` | `#262626` |
| radius | `--radius-container` | **12px**（小さな 1 行ラベルでも 12px → 28px 高の 1 行ならほぼピル型） | |
| padding | block 4px（`--spacing-1`）/ inline 8px（`--spacing-2`） | | |
| font | `--font-family-body`（Figtree）, `--text-body-size` 14px / `--text-body-leading` 20px, weight 400 | | |
| max-width | 300px（content）、`word-break: break-word` で折返し | | |
| 高さ（1 行） | 20 + 4 + 4 = 28px | | |
| shadow / border | なし | | |
| offset | 4px（`--spacing-1`, `useTooltip.tsx:723`） | | |
| arrow | なし（kivact `TooltipRecipes::arrow` は `no_arrow` のまま） | | |

```
      ╭──────────╮
      │  Close   │   28px 高, bg #000 / #fff, 文字 #fff / #262626, 角丸 12
      ╰──────────╯
          4px
        [ × ]
```

配置: above/center。フリップ・スライドは 15 章。さらに Tooltip は `keepsAnchorClear: true`（`useTooltip.tsx:322`）: 決定後の箱がアンカーを覆った場合（上下どちらにも収まらない背の高い内容）、第 2 のフォールバック列に切替: `beside end` → `beside start` → `flip-block` → `fit 同側` → `fit 反対側`（`Layer/layerSlideRules.ts:212-218`）。fit は「その側の余白に高さを合わせて内部スクロール」（最小はトリガーが残す余白の半分）。ネイティブ: kivact の overlay にこの規則は無い → 「アンカーを覆わない」ことだけ保証し、溢れたら高さを余白に合わせてスクロール（UNRESOLVED: beside への横移動）。

### 8.5 Motion
- 入場: layerAnimations.above（`opacity 0; translateY(+8px) scale(0.95)` → `1; 0; 1`）、`--duration-fast-max` 165ms、ease-standard (0.24,1,0.4,1)、fill backwards。placement に応じて方向が変わる（6.5 表と同じ）。
- 退場: なし（即時）。
- reduced motion: fine=即時、coarse=165ms フェードのみ。
- 遅延: hover → 200ms で表示。leave → hideDelay>0 ならその値、0 なら 100ms（`HOVER_BRIDGE_DELAY`, `useTooltip.tsx:44-50`, 428-437）で非表示。サーフェス上ではキャンセル。キーボード focus（:focus-visible）→ **即時**表示（`useTooltip.tsx:503-525`）。プログラム focus・タップ focus では出さない。
- skip-delay（連続ホバーで遅延省略）グループは**無い**（kivact 既定の 300ms skip を 0 にする）。

### 8.6 Behaviour & a11y
- `role="tooltip"`、trigger に `aria-describedby`（説明として読まれる。名前ではない）。
- Escape: layer スタック最上位なら閉じる（ダイアログ内でもダイアログは閉じない）。
- toggle 動作: ボタンのクリック/タップ/Enter/Space で pin、再操作・Escape・外側押下・focus 離脱で解除。pin 時のみテキストを 1 回 live 通知。trigger は aria-expanded/aria-controls を報告。押下はトリガーで止まる（周囲のクリック可能行を発火しない）。
- タッチ（adaptive + auto）: アクション無しのトリガーはタップで開く。
- テキスト: なし（内容は呼び出し側）。

### 8.7 Kivact 実装方針
- **RESTYLE `kivact::ui::tooltip`**。TooltipRecipes::surface: bg = 反転色（soft の Color::TextPrimary）、文字 = surface 色、radius 12、padding 4/8、max-width 300、14/20。arrow = なし。offset 4（`Tooltip::offset(4.0)` の既定を recipe 側で持てなければ kit の theme 既定値として設定）。enter = 165ms opacity+translate 8px+scale 0.95。
- タイミング: soft テーマ境界で `provide(TooltipConfig{delay: 200ms, skip_delay: 0ms})`。キーボード focus 即時と 100ms hide bridge は kivact-interaction の tooltip が対応していなければ UNRESOLVED（`use_tooltip` が focus 時に delay を適用するかを確認し、必要なら interaction 側の拡張を提案）。
- 参考: `crates/kivact-ui/src/tooltip.rs`, `crates/kivact-ui-basic/src/recipes_popup.rs`（tooltip.tsx の recipe と arrow）。

### 8.8 受け入れ確認
- [ ] ホバー 200ms で `role=tooltip` 出現、trigger の description = テキスト。離れて ~100ms で消える。tooltip 上へ移動しても消えない。
- [ ] Tab focus で即時表示、Escape で消える（親 Dialog は閉じない）。
- [ ] 300px を超える文は折返し。
- [ ] 画面上端のボタンでは下にフリップ。
- [ ] 目視: light は黒地白字、dark は白地 #262626 字、角丸 12、矢印なし、trigger から 4px。

## 9. Toast

docs: https://astryx.atmeta.com/components/Toast ・ source: `packages/core/src/Toast/`（`Toast.tsx` 415 行, `ToastViewport.tsx` 859, `useToast.tsx`, `useToastGesture.ts` 414, `ToastContext.ts`, `types.ts`）

### 9.1 概要
短い非ブロッキング通知（「保存しました」「送信しました」）。決定を求めないフィードバック。本番では `useToast()`（ToastViewport が配置・積み重ね・自動消去・重複排除を担う）。`Toast` コンポーネント単体はプレビュー用のインライン描画。

- 使う: 数語の短文。可逆操作には endContent に短い Undo（必要なら `isAutoHide:false`）。連打で重複する通知は `uniqueID`。注意が要る失敗は error（自動では消えない）。
- 使わない: ブロッキングな重大エラー（Banner を使う）。長文・複数行（5 秒で消える）。フォームのバリデーションエラー（インラインで）。

**重要: Astryx の Toast は sonner 風の「重なったデッキ（奥ほど縮小）」ではない**。縦に並ぶリスト（gap 8px、最大 5 件表示、ホバーでの展開もなし）。kivact-ui-basic の sonner スタック recipe とは別物として実装すること。

### 9.2 Anatomy
```
<div viewport popover=manual role=region aria-label="Notifications" tabIndex=-1 (fixed, z 500, padding 16)>   ← toasts がある時だけ role/label
  <div row data-toast-id (grid; rows 1fr; padding-block-end 8 = gap)>      ← 高さの開閉アニメ担当
    <div inner (overflow hidden → 安定後 visible)>
      <div toast role=status|alert aria-live=polite|assertive aria-atomic (card 400w, padding 16, radius 12)>
        [MediaTheme 反転]
        <div inner flex gap 12>
          <div content flex 1 (overflow-wrap anywhere)> body
          <div endContent (flex, gap 8, 高さ 20px, margin-inline-end -4)> endContent  [× ghost sm]
```
並び: bottom 系は column（古い→新しい、**最新が下端＝画面端に最も近い**）、top 系は column-reverse（最新が上端）。

### 9.3 API
`useToast()` → `showToast(options) => dismiss()`。`ToastOptions`（`types.ts`）:

| option | 型 | 既定 | 意味 | Kivact（`kivact::ui::Toast` / `Toaster`） |
|---|---|---|---|---|
| `body` | ReactNode | 必須 | メッセージ | `Toast::new(text)`（title = 本文。Astryx は 1 本文のみ。`.body(..)` は使わない） |
| `type` | `'info' \| 'error'` | `'info'` | 背景色と live の強さ | `.tone(Tone::Neutral)` / `.tone(Tone::Danger)`。他の Tone は info と同じ見た目にマップ |
| `isAutoHide` | bool | info=true / error=false | | `.duration(Some(5s))` / `.duration(None)` |
| `autoHideDuration` | ms | 5000 | | `.duration(Some(Duration::from_millis(n)))` |
| `endContent` | ReactNode | – | 末尾の短いアクション（Undo 等） | `.action(label, f)`（既存, ボタン 1 個）。任意ビューが要るなら kit 拡張 `.end_content(|| ..)`（UNRESOLVED: Toast はデータなので view slot を持てるか） |
| `renderContent` | (props)=>ReactNode | – | カード中身を丸ごと置換 | 削除推奨（必要なら soft の recipe 差し替えで） |
| `uniqueID` | string | – | 重複排除キー | `Toast::key(String)` 新設 or kit 側 `soft::toast_unique(toaster, key, toast)` で既存 id を dismiss→push |
| `collisionBehavior` | `'overwrite' \| 'ignore'` | `'overwrite'` | 同キー存在時: 置換 / 無視 | 上記 helper の引数 |
| `onHide` | (reason 'auto'\|'manual') | – | 削除時 | `Toast::on_hide(f)` 新設（UNRESOLVED） |

`ToastViewport` props: `position: 'topEnd'|'topStart'|'bottomEnd'|'bottomStart'`（既定 `'bottomEnd'`）, `maxVisible`（既定 **5**）, `inset {top,bottom,start,end}`（px）, `isTopLayer`（既定 true: モーダル中はモーダル内へ再ホストして前面・クリック可能に）。Kivact: `toast_region(toaster()).variant(soft::ToastPosition::BottomEnd)`、`Toaster::max_visible(5)`（既存）, `.inset(..)` 新設。

### 9.4 Visual spec
viewport（`ToastViewport.tsx:62-131`）:
- `position: fixed`、z-index 500、flex column、padding 各辺 `max(16px, safe-area)`（ネイティブ 16px）、inline 両端 0 に張り、`align-items` で end/start 寄せ。bottom 系は bottom: 0、top 系は top: 0。pointer-events none（行のみ auto）。
- 行: `max-width 400px; width 100%`、間隔 = 各行 `padding-block-end 8px`（`--spacing-2`）、画面端側の行は 0。

card（`Toast.tsx:74-131`）:

| 項目 | info light | info dark | error light | error dark |
|---|---|---|---|---|
| background | `--color-background-inverted` `#0A1317` | `#FFFFFF` | `--color-background-error-inverted` `#AA071E` | `#E3193B` |
| 文字（MediaTheme 反転） | text-primary(dark 側) `#ffffff` | text-primary(light 側) `#000000` | `#ffffff` | `#ffffff`（error は fallback dark） |
| × アイコン色 | 同上（inherit） | | | |

（neutral テーマは `--color-background-inverted` と `-error-inverted` を上書きしないので core 既定値。MediaTheme は塗り色の明暗から自動選択: 暗い地 → dark 側トークン, 明るい地 → light 側。`Toast.tsx:356-358`）

- 幅 400px、max 100%。padding 16px 全辺。radius `--radius-container` 12px。shadow `--shadow-med`（light `0 2px 4px rgba(0,0,0,.05), 0 4px 12px rgba(0,0,0,.10)` / dark `0 2px 4px rgba(0,0,0,.35), 0 4px 12px rgba(0,0,0,.50), inset 0 0 0 1px rgba(255,255,255,.12)`）。border なし。
- font: body 14px/20px 400 Figtree（`typeScaleDefaults` 固定値）。
- inner: flex、align-items flex-start、gap 12px（`--spacing-3`）、nowrap。本文は `overflow-wrap: anywhere` で折返し。
- endContent: flex、center、gap 8px、**高さ 20px（本文 1 行目の行高）に固定**して中央揃え、margin-inline-end −4px。
- × ボタン: Button ghost sm icon-only = 28×28px、radius 10px、icon close 16px、色 inherit。hover: `--color-overlay-hover`（反転 MediaTheme 下なので light toast 上では dark 側値 `#FFFFFF0D`、dark toast 上では `#0000000D`）。名前/ツールチップ "Dismiss notification"。

```
                                   ┌──────────────── 400 ────────────────┐
                                   │16 Changes saved           [Undo][×] │  ← 高さ 52（1 行）
                                   └─────────────────────────────────────┘
                                                 8px gap
                                   ┌─────────────────────────────────────┐
                                   │16 Upload failed (error #AA071E)  [×]│
                                   └─────────────────────────────────────┘
                                                                      16 ↘ 画面端
```

RTL: viewport の inline padding（safe-area 左右）を反転、end/start 寄せは論理。

### 9.5 Motion
| トリガー | プロパティ | from → to | duration | easing | file:line |
|---|---|---|---|---|---|
| 入場（card） | opacity, transform | `0; translateY(--_toast-slide-y)` → `1; 0`。slide-y = bottom 系 **+8px**（下から上がる）/ top 系 **−8px** | `--duration-fast` **125ms** | ease-standard (0.24,1,0.4,1) | `Toast.tsx:84-97`（@starting-style）, `ToastViewport.tsx:157-162` |
| 入場（行の高さ） | grid-template-rows, padding | `0fr; pb 0` → `1fr; pb 8px`（既存の toast が押し出されて滑らかに動く） | 125ms | ease-standard | `ToastViewport.tsx:142-156` |
| 退場（card） | opacity, transform | `1` → `0`、translateY → slide-y（端の方向へ 8px）、スワイプ時は swipe-exit-y | 125ms | ease-standard | `Toast.tsx:117-120` |
| 退場（行） | grid rows / padding | `1fr` → `0fr`、pb → 0（下の toast が詰まる）。grid-template-rows の transitionend で DOM から除去 | 125ms | ease-standard | `ToastViewport.tsx:170-173`, 265-276 |
| 安定後 | inner overflow hidden → visible（影を描くため） | | | | 174-189 |
| スワイプ中 | transform/opacity | `translateY(travel)`、opacity `1 − 0.4×progress`、scale `1 − 0.02×progress`（progress = travel / カード高, 0..1）、transition 0s | | | `useToastGesture.ts:196-212` |
| スワイプ確定 | translateY → ±120%（端の方向）+ 通常の退場 | 125ms | ease-standard | 236-251 |
| スワイプ不成立 | 0 に戻る | 125ms | ease-standard | |
| reduced motion | 全て 0.01ms | | | | `Toast.tsx:91-94`, `ToastViewport.tsx:149-152` |

スタックの「奥行き scale / offset」は**無い**（縦リスト）。最大表示 5 件: 6 件目が来ると最古が非表示（キューに残り、前が消えると再び現れる。現れる時は入場アニメ, `ToastViewport.tsx:538`）。ホバーでの展開もない。

### 9.6 Behaviour & a11y
- 自動消去: info 5000ms。タイマーは **pointer がカード上**・**focus がカード内**・**タッチ/ペンのジェスチャ中**・**ウィンドウ blur 中** のいずれかで一時停止（ソース別に管理、すべて解除で再開）。停止時の残り時間は最低 1000ms 保証（`Toast.tsx:240-246`）。各 toast 独立（hold-all ではない）。
- スワイプ（タッチ/ペンのみ、マウスドラッグは除外。カード内のボタン等の上では開始しない）: 縦方向に **8px** 超かつ `|dy| > |dx|×1.2` で端方向の意図確定（横 8px 超が先なら中止、反対方向なら中止）。dismiss = travel ≥ `max(カード高×0.4, 48px)` または（travel ≥ 48px かつ 平均速度 travel/elapsed > **1.2 px/ms**）。理由は 'manual'。
- F6: toasts がある時、ページ側から F6 で最新 toast（その最初の focus 可能要素）へ移動。全部消えたら元の要素へ focus を戻す。focus 中の toast が消えたら隣の toast へ。
- live: 各カードは info=`role=status`/polite、error=`role=alert`/assertive、aria-atomic。加えて viewport が常設の live region（useAnnounce, 16.3）でテキストを通知（endContent は通知しない）。
- viewport: toasts がある時のみ `role=region` 名前 "Notifications"。
- モーダル中: viewport を最新のモーダル内に再ホストし、前面かつ操作可能に。
- テキスト: `soft.toast.dismiss` = "Dismiss notification"、`soft.toast.region` = "Notifications"。

### 9.7 Kivact 実装方針
- **RESTYLE `kivact::ui::toast_region` + kit wrapper**。ToastRecipes: region（fixed, 右下, padding 16, 列 gap 8, 最大幅 400）、toast（反転背景・radius 12・shadow-med・padding 16・幅 400）、title（=body 14/20 反転文字色）、close（ghost sm 28×28）、icon は使わない（Astryx は tone アイコンなし → icon recipe は size 0）。`hold_all_on_hover(false)`（個別停止）。
- 入退場: 行高の開閉（grid rows 0fr↔1fr）は kivact の layout アニメで「高さ 0 ↔ 実高」を 125ms で。できなければ card の opacity/translate だけで妥協（UNRESOLVED: 行高アニメ）。
- 反転色: soft テーマ側に `ToastInverted` 色（info `#0A1317`/`#FFFFFF`、error `#AA071E`/`#E3193B`）と前景（`#ffffff`/`#000000`）を持つ。MediaTheme は再現不要（固定の前景色で十分）。
- スワイプ: kivact の toast_region にスワイプがなければ kit wrapper で `use_drag` を toast item part に付ける（タッチのみ。デスクトップでは close ボタンで足りるので優先度低）。
- uniqueID / onHide: kivact `Toaster` に無い → kit helper で id マップを持つ（UNRESOLVED: Toaster 拡張の可否）。
- 参考: `crates/kivact-ui/src/toast.rs`, `crates/kivact-interaction/src/toaster.rs`, `crates/kivact-ui-basic/src/recipes_controls.rs`（sonner スタックの toast recipe。**流用しない**、リスト型に作り直す）。

### 9.8 受け入れ確認
- [ ] push すると右下（端から 16px）に `role=status` の toast、5 秒後に消える。error は `role=alert` で消えない。
- [ ] ホバー中・focus 中は消えない、離れると残り時間（最低 1 秒）で消える。
- [ ] 6 件 push → 5 件だけ表示、1 件消すと隠れていた最古が再出現。
- [ ] × で消える（名前 "Dismiss notification"）。F6 で toast に focus が入る。
- [ ] 目視: light は濃紺黒 `#0A1317` 地に白字、dark は白地に黒字、error は赤地白字。角丸 12、幅 400、8px 間隔、入場は 8px 下から 125ms でフェードイン、退場で下の列が詰まる。

## 10. CommandPalette

docs: https://astryx.atmeta.com/components/CommandPalette ・ source: `packages/core/src/CommandPalette/`（`CommandPalette.tsx` 726 行 + Input/List/Item/Group/Footer/Empty, `CommandPaletteContext.ts`, `busyStartSettle.ts`）

### 10.1 概要
キーボード主導のランチャー（Cmd/Ctrl-K）。コマンド・ナビゲーション・アクションへの検索可能なダイアログ。`searchSource`（`search(query)` と `bootstrap()`）が項目を供給し、`auxiliaryData.group` で自動グループ化。picker モード（`value`/`onValueChange`）あり。

- 使う: bootstrap 結果を用意（入力前から候補）。group で整理。同一画面に複数あるなら具体的 label。renderItem は非対話（行全体が 1 option）。
- 使わない: 単純なドロップダウン（Menu / Selector）。過剰な件数・グループ。IME 変換や Home/End を壊すキー奪取。

### 10.2 Anatomy
```
Dialog(width 640, maxHeight 480, purpose info, aria-label="Command palette")
 └ Layout defaultHasDividers
    ├ header: LayoutHeader hasDivider padding 0
    │    └ CommandPaletteInput: [🔍 search 16px] [input role=combobox aria-controls=list aria-activedescendant] [spinner sm][endContent]
    ├ content: LayoutContent padding 0
    │    └ CommandPaletteList role=listbox aria-label="Commands" (scroll, padding 4, gap 2)
    │         ├ CommandPaletteGroup role=group aria-label=heading
    │         │    ├ heading (12px secondary)
    │         │    └ CommandPaletteItem role=option aria-selected data-value
    │         └ CommandPaletteEmpty（0 件時）
    └ footer: LayoutFooter hasDivider padding 0
         └ CommandPaletteFooter: [↑][↓] Navigate   [↵] Select   [Esc] Close
```

### 10.3 API
| prop | 型 | 既定 | Kivact（`kivact::ui::command_palette(open, commands, on_run)`） |
|---|---|---|---|
| `isOpen` / `onOpenChange` | | 必須 | 既存 open binding |
| `searchSource` | `SearchSource<T>` | 必須 | `commands: impl IntoValue<Vec<Command<K>>>` + `.filter(f)`（既存）。非同期検索は kit 拡張 `.search(|query| async ..)`（UNRESOLVED） |
| `input` | ReactNode | `<CommandPaletteInput/>` | `.placeholder(text)`（既存）。入力 slot 差替えは不要 |
| `footer` | ReactNode | `<CommandPaletteFooter/>` | `.footer(|| ..)`（既存）。既定フッタは soft が描く |
| `renderItem` | (item, isSelected)=>ReactNode | label のみ | `Command` の icon / detail(shortcut) / keywords（既存フィールド）＋ `.item(|cmd, state| view)` 新設可 |
| `emptySearchText` | ReactNode | "No results" | `.empty_text(text)`（既存） |
| `emptyBootstrapText` | ReactNode | "Type to search" | `.bootstrap_text(text)` 新設 |
| `value` / `onValueChange` | string | – | picker モード: `.selected(Binding<Option<K>>)` 新設 |
| `label` | string | "Command palette" | 名前（kivact は `palette.search` を検索欄に使う）→ `.name(..)` |
| `width` | number\|string | 640 | `.width(Length)` |
| `maxHeight` | number\|string | 480 | `.max_height(Length)` |
| `isInline` | bool | false | プレビュー専用 |

### 10.4 Visual spec
外枠: Dialog（1.4）と同じ: bg surface `#ffffff`/`#262626`、radius 12、`--shadow-high`、scrim `#00000080`/`#000000CC` + blur 2px、幅 640（max ウィンドウ−32）、max-height 480、padding 0（各リージョン padding 0、子が自前の padding）。header/footer の境界に 1px `--color-border`（`#00000014`/`#FFFFFF1A`）。

| 部品 | layout | typography / color | file:line |
|---|---|---|---|
| Input wrapper | flex、center、gap 8、padding 12px(上下) 16px(左右)、shrink 0 → 高さ 44px | – | `CommandPaletteInput.tsx:52-60` |
| 検索アイコン | `search` 16px（Icon sm）| `--color-text-secondary` `#474747`/`#9e9e9e` | 61-67, 206 |
| input | flex 1、枠なし、透明、padding 0 | body 14px/20px、text-primary `#000`/`#fff`、placeholder "Search…" は text-secondary | 87-109 |
| spinner | Spinner sm（検索 pending 時、**150ms 遅延**で出現）、end 群 gap 4 | | 68-86, 375 |
| List | scroll-y、padding 4px、column、gap 2px（`--spacing-0-5`） | | `CommandPaletteList.tsx:31-41` |
| Group | column、gap 2、padding-block 4 | | `CommandPaletteGroup.tsx:27-33` |
| Group heading | padding 4px 12px | supporting 12px/20px、text-secondary | 34-42 |
| Item | flex、center、gap 8、width 100%、padding 8px 12px、radius `--radius-inner` 6px → 高さ 36px | font-size label 14px、line-height 20px（継承）、weight 400（継承）、text-primary | `CommandPaletteItem.tsx:34-56` |
| Item hover（pointer） | bg `--color-overlay-hover` `#0000000D`/`#FFFFFF0D`（interactionOverlay） | | 182 |
| Item highlighted（キーボード/ホバー追従） | bg `--color-overlay-hover` 同上 | | 57-59 |
| Item selected（picker の現在値） | bg `--color-accent-muted` `#f1f1f1` / `#262626` | | 64-66 |
| Item disabled | opacity 0.5、cursor default | | 60-63 |
| Empty | flex center、padding 32px(上下) 16px(左右)、中央揃え、`overflow-wrap: anywhere` | supporting 12/20、text-secondary | `CommandPaletteEmpty.tsx:27-45` |
| Footer | flex wrap、center、gap 16、padding 8px 16px | supporting 12/20、text-secondary | `CommandPaletteFooter.tsx:35-49` |
| Footer hint | flex center gap 4：Kbd(up) Kbd(down) "Navigate" / Kbd(enter) "Select" / Kbd(escape) "Close" | Kbd は Kbd コンポーネント（別グループ）の見た目 | 94-112 |

```
┌──────────────────────────── 640 ────────────────────────────┐
│ 🔍 Search…                                          (◌)      │ 44
├─────────────────────────────────────────────────────────────┤ 1px border
│  Navigation                         (12px secondary)        │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Go to Home                (highlighted: 5% overlay)     │ │ 36, radius 6
│ └─────────────────────────────────────────────────────────┘ │
│   Go to Settings                                            │
├─────────────────────────────────────────────────────────────┤
│ [↑][↓] Navigate   [↵] Select   [Esc] Close   (12px)         │ 36
└─────────────────────────────────────────────────────────────┘
```

### 10.5 Motion
- ダイアログ入場: Dialog と同じ（400ms ease-standard, 方向 translate 16px + scale 0.95→1, 退場なし, reduced motion 規則も同じ）。
- spinner: `@starting-style opacity 0` → 1、duration 1ms、**delay 150ms**（ほぼ即時の検索ではちらつかない, `CommandPaletteInput.tsx:75-86`）。
- item の hover/highlight/pressed 背景: **transition なしで即時切替**（Item 自身に transition 指定がなく、`utils/interactionOverlay.stylex.ts` は色を切り替えるだけ）。押下（:active）は `--color-overlay-pressed`（`#0000001A`/`#FFFFFF1A`）。タッチ時のみ離した後に押下色が 200ms（`pressConsts.releaseMs`）でフェード。
- ハイライト変更時は `scrollIntoView({block: 'nearest'})`（アニメなし）。

### 10.6 Behaviour & a11y
- 開くと input に自動 focus。input = `role=combobox`、`aria-expanded`、`aria-autocomplete=list`、`aria-controls`=listbox、`aria-activedescendant`=ハイライト中 option。名前 "Search commands"（`@astryx.commandPalette.input.label`）、placeholder は名前にしない。
- キー: ↑/↓ = ハイライト移動（disabled はスキップ）、PageUp/PageDown = 先頭/末尾、**Home/End は caret 移動（奪わない）**、Space は入力、Enter = ハイライト項目を選択して閉じる、Escape = Dialog の layer スタックで閉じる（ネストした面が先）。IME 変換中のキーは奪わない。
- マウス: option 上の mouseover でハイライト追従。クリックで選択して閉じる。
- 選択: `selectItem(value)`（picker モードでは onValueChange）→ close。
- 通知: 検索確定ごとに件数 "{count} results" を polite 通知、0 件は Empty の表示文 "No results for {query}" 相当を通知（pending 中は通知しない）、busy 開始時 "Loading"。閉じる時に通知をクリア。
- ロール: dialog（名前 "Command palette"）> listbox（"Commands"）> group（heading）> option（aria-selected, aria-disabled）。
- テキスト（i18n キー案）: `soft.command_palette.label` "Command palette", `.input_label` "Search commands", `.placeholder` "Search…", `.list_label` "Commands", `.empty_search` "No results", `.empty_bootstrap` "Type to search", `.no_results_for` "No results for {query}", `.result_count` "{count} results"（複数形）, `.loading` "Loading", `.footer_navigate` "Navigate", `.footer_select` "Select", `.footer_close` "Close"。

### 10.7 Kivact 実装方針
- **RESTYLE `kivact::ui::command_palette`**。CommandPaletteRecipes: scrim（Dialog と同値 + blur 2px）, panel（640×max480, radius 12, shadow-high, bg surface）, search（padding 12/16, gap 8, 下 1px border）, search_icon（16px secondary）, field（14/20, placeholder secondary）, list（padding 4, gap 2）, group（12px secondary, padding 4/12）, item（36px, padding 8/12, radius 6, highlighted overlay-hover, selected accent-muted, disabled 0.5）, empty（padding 32/16, 12px secondary）, footer（padding 8/16, gap 16, 上 1px border）。
- フッタ既定（Kbd ヒント 3 つ）は soft が `.footer(..)` で既定提供。spinner は非同期検索を入れる場合のみ。
- 参考: `crates/kivact-ui/src/command_palette.rs`, `crates/kivact-ui-basic/src/recipes_command.rs`（cmdk 風 recipe）。
- 差分/難所: Home/End を caret に残す、PageUp/PageDown で端へ（kivact `use_command_palette` の挙動確認, UNRESOLVED）。検索結果件数の live 通知（`kivact::a11y` の announce を使う）。

### 10.8 受け入れ確認
- [ ] 開くと検索欄（role=combobox, 名前 "Search commands"）に focus。入力で絞り込み、↓↑ でハイライト、Enter で実行して閉じる。
- [ ] PageDown で末尾、Home/End は caret が動くだけ。
- [ ] 0 件で中央に "No results"、入力前で bootstrap 0 件なら "Type to search"。
- [ ] Escape で閉じ、トリガーへ focus。
- [ ] 目視: 幅 640、入力行 44px と下線、行 36px・角丸 6・ハイライトは薄いグレー（light 5% 黒 / dark 5% 白）、選択中は `#f1f1f1` / `#262626`、フッタ 12px グレー + Kbd。

## 11. DropdownMenu

docs: https://astryx.atmeta.com/components/DropdownMenu ・ source: `packages/core/src/DropdownMenu/`（`DropdownMenu.tsx` 1792 行, `DropdownMenuItem.tsx`, `DropdownMenuCheckboxItem.tsx`, `DropdownMenuRadioGroup.tsx`, `DropdownMenuRadioItem.tsx`, `DropdownMenuDivider.tsx`, `DropdownMenuGroup.tsx`, `DropdownMenuSubMenu.tsx` 1403, `MenuBottomSheet*.tsx`, `menuWidth.ts`, `useMenuOverflow.ts`, `useMenuDrillIn.tsx`, `menuItemHover.ts`, `renderDropdownItems.tsx`）, `hooks/useMenuPress.ts`, `hooks/menuPressGesture.ts`, `hooks/useMenuHover.ts`, `hooks/useTypeahead.ts`, `hooks/useListFocus.ts`, 共通の行は `Item/Item.tsx`

### 11.1 概要
ボタンから開くアクションメニュー（APG menu button）。`items` データ駆動 / compound children の 2 モード。セクション・区切り・サブメニュー（ラップトップではフライアウト、スマホではドリルイン + Back 行）、チェック/ラジオ項目。`presentation` で popover / bottom-sheet / adaptive。

- 使う: 簡潔な動詞ラベル。多い時はセクションと区切り。短い平坦なアクション群のタッチ面は bottom-sheet / adaptive。presentation は製品側で明示的に選ぶ。
- 使わない: ナビゲーション（ナビゲーション部品を使う）。10–12 件超をグループ化せずに並べる。

### 11.2 Anatomy
```
Button trigger (aria-haspopup="menu" aria-expanded aria-controls) [label][chevronDown 16]
<layer popover (offset 4, layerAnimations[placement])>
  <div surface (usePopover: bg popover, radius 12, shadow-low)>   ← header/footer がある時は role=dialog aria-modal 名前=メニュー名
    [header region (padding 4)]
    <div role="menu" aria-labelledby=trigger (flex column, gap 2, padding 4, max-height 300, scroll)>
       DropdownMenuGroup role=group aria-labelledby=heading
          heading (12px secondary)
          DropdownMenuItem role=menuitem tabIndex=-1 (Item: [icon][label/description][endContent])
       DropdownMenuDivider (Divider 1px, margin-block 4)
       DropdownMenuCheckboxItem role=menuitemcheckbox aria-checked ([checkbox indicator][label])
       DropdownMenuRadioGroup role=group aria-label
          DropdownMenuRadioItem role=menuitemradio aria-checked
       DropdownMenuSubMenu trigger role=menuitem aria-haspopup=menu aria-expanded [label][chevronRight 16]
          └ flyout layer (placement end / alignment start / offset 4)  <div role=menu ...>
    [footer region (padding 4)]
```

### 11.3 API
DropdownMenu:
| prop | 型 | 既定 | 意味 | Kivact（`kivact::ui::menu(label, entries, on_select)`） |
|---|---|---|---|---|
| `button` | Button props（onClick 除く） | `{label: 'Menu'}` | 組み込みトリガー | `menu(label, ..)` の trigger（MenuRecipes::trigger）。`size` は行サイズにも波及（下） |
| `renderTrigger` | (props)=>ReactNode | – | 任意のトリガー | `.attach(trigger)`（既存） |
| `items` | DropdownMenuOption[] | – | `{label, onClick?, href?, icon?, description?, endContent?, isDisabled?, variant?, hasCloseOnSelect?, id?}` / `{type:'divider'}` / `{type:'section', title?, items}` | `MenuEntry::item(key, label).icon(..).description(..).shortcut(..).disabled(..).destructive(..).keep_open(..)`, `MenuEntry::separator()`, `MenuEntry::group(title, entries)`, `MenuEntry::submenu(..)`（既存） |
| `children` | compound | – | Item/Divider/SubMenu/Checkbox/Radio | 同上 MenuEntry（`checkbox`, `radio`） |
| `presentation` | popover\|bottom-sheet\|adaptive | `'popover'` | data モードのみ sheet 可 | `.presentation(..)`（kit wrapper。sheet は `soft::bottom_sheet` + アクションリスト） |
| `isMenuOpen` / `onOpenChange` | | 非制御 | | `.open(Binding<bool>)` |
| `menuWidth` | number\|string | トリガー幅 | 最小幅（内容で伸びる）。キーワードは inline-size。ビューポート上限 | `.min_width(Length)`; 既定 `at_least_trigger_width(true)`（既存） |
| `menuMaxHeight` | number | 300 上限 | 300px 上限を引き上げ | `.max_height(px)` |
| `header` / `footer` | ReactNode | – | 行以外のコントロール（フィルタ欄、Create ボタン）。あると popover が modal dialog になり Tab が header→menu→footer を循環 | `.header(|| ..)`, `.footer(|| ..)` 新設（UNRESOLVED: kivact menu に region が無い） |
| `placement` | above\|below\|start\|end | `'below'` | | `.placement(..)`（既存） |
| `alignment` | start\|center\|end | `'start'` | | `.align(..)`（既存） |
| `onClick` | fn | – | 受理されたトリガー起動 | trigger の `.on_press` |
| `hasChevron` | bool | true | トリガーの chevronDown | `.chevron(bool)` |

DropdownMenuItem / ContextMenuItem: `icon`, `label`(必須), `description`, `onClick`, `href`/`target`/`rel`（行を本物のリンクに → ネイティブでは `on_select` で開く。href は削除）, `isDisabled`, `endContent`（ショートカット等 → `.shortcut(text)`）, `hasCloseOnSelect`(true → `keep_open(false)`), `variant: 'default'|'destructive'`。
DropdownMenuCheckboxItem: `label, description, icon, value: bool, onChange, isDisabled, hasCloseOnSelect`(既定 **false**), `endContent` → `MenuEntry::checkbox(key, label, checked)`。
DropdownMenuRadioGroup: `value, onChange, label`(必須, group 名), `hasCloseOnSelect`(既定 **true**)。DropdownMenuRadioItem: `value, label, description, icon, isDisabled` → `MenuEntry::radio(..)`（グループは `MenuEntry::group`）。
DropdownMenuGroup: `title?`。DropdownMenuDivider: なし。DropdownMenuSubMenu: `label, icon, items/children, isDisabled, menuWidth, header/footer, hasSpinner`（読込中は chevron の代わりに Spinner sm）。

### 11.4 Visual spec
サーフェス（usePopover 既定 surface + menu div `DropdownMenu.tsx:138-185`）:

| 項目 | 値 | light | dark |
|---|---|---|---|
| background | `--color-background-popover` | `#ffffff` | `#1b1b1b` |
| radius | `--_dropdown-menu-radius` = `--radius-container` | 12px | |
| shadow | `--shadow-low` | `0 2px 4px rgba(0,0,0,.05), 0 4px 8px rgba(0,0,0,.10)` | `… rgba(0,0,0,.25), … rgba(0,0,0,.40), inset 0 0 0 1px rgba(255,255,255,.08)` |
| border | なし | | |
| padding | `--_dropdown-menu-padding` = `--spacing-1` 4px | | |
| 行間 gap | `--spacing-0-5` 2px | | |
| 幅 | 既定 min-width = トリガー幅（ビューポート上限）、内容で伸びる | | |
| 高さ | max `min(300px, ビューポート高 − 32)`、超えたら縦スクロール（overscroll contain） | | |
| offset | 4px（`DropdownMenu.tsx:1732`） | | |

トリガー（Button, 既定 variant secondary, size md）: 開いている間 `background-image: overlay-pressed`（light `#0000001A` / dark `#FFFFFF1A`）を重ね押下中の見た目を保持（`DropdownMenu.tsx:127`, 139-141）。末尾 chevronDown 16px inherit。

行（DropdownMenuItem, `DropdownMenuItem.tsx:57-115` + `Item.tsx:346-525`）:

| size（= button.size） | padding block / inline | 高さ（label 1 行） |
|---|---|---|
| sm | 4px / 8px | 28px |
| md（既定） | 6px / 8px | 32px |
| lg | 8px / 8px | 36px |

- radius `max(0, 12 − 4)` = **8px**（サーフェス radius − padding で同心円）。gap 8px（Item）。
- label: body 14px/20px 400、`--color-text-primary`（`#000000`/`#ffffff`）、1 行 ellipsis（Item 既定）。description: supporting 12px/20px `--color-text-secondary`（`#474747`/`#9e9e9e`）、label の下。
- icon（startContent）: 16px、色 `--color-icon-secondary`（light `#6a6a6a` / dark `#9e9e9e`）。endContent（ショートカット等）は末尾 `margin-inline-start: auto`。
- 状態: rest 透明 / highlighted（= DOM focus。ポインタ移動で focus が行に移る `menuItemHover.ts`）`--color-overlay-hover`（`#0000000D`/`#FFFFFF0D`）/ pressed（hover 可能環境の :active）`--color-overlay-pressed`（`#0000001A`/`#FFFFFF1A`）/ focus-visible（キーボード）: 上記背景 + 2px focus ring（accent `#1b1b1b`/`#f1f1f1`）を行の内側（offset −2px）に / disabled: opacity 0.5、cursor default、ポインタ無効 / destructive: 文字・説明・アイコンを `--color-text-error`（light `#c0262c` / dark `#ffaea7`）、背景の hover は通常と同じ。
- 背景色 transition: Item の interactive `background-color` `--duration-fast-min` **95ms** ease-standard（`Item.tsx:406-414`）。

CheckboxItem / RadioItem（`DropdownMenuCheckboxItem.tsx:46-87`, `DropdownMenuRadioItem.tsx:43-83`）:
- size 別 padding は**適用されない**（Item の balanced = block 8px / inline 8px → 高さ 36px）。radius 8px、focus 背景 overlay-hover、disabled 0.5。
- マーカー: checkbox / radio インジケータ（menu size sm → indicator sm、それ以外 md。見た目は CheckboxInput / RadioList 担当グループの仕様）を label の**前**に配置（coarse pointer では末尾へ移動）。装飾扱い（aria-hidden, pointer-events none）。

Group heading（`DropdownMenuGroup.tsx:40-50`）: padding 4px 8px、supporting 12px/20px、text-secondary、選択不可。
Divider: Divider 1px `--color-border`（`#00000014`/`#FFFFFF1A`）、margin-block 4px（`DropdownMenuDivider.tsx:32-36`）。
RadioGroup: column gap 2px。

SubMenu トリガー（`DropdownMenuSubMenu.tsx:137-187`）: 行と同寸（size 別 padding 同じ）、末尾 chevronRight 16px secondary（RTL で反転）。flyout 開中は背景 `--color-overlay-hover` 固定。フライアウト（189-256）: bg popover, radius 12, shadow-low, padding 4, gap 2, **min-width 160px**, max-height 300, placement end / alignment start / offset 4。

```
[ Options ▾ ]  ← 開中 pressed 色
┌──────────────────────┐ 4px
│ ✎  Edit        ⌘E    │ 32px, radius 8, hover 5% overlay
│ ⧉  Duplicate         │
│ ──────────────────── │ 1px border, ±4px
│ SECTION (12px gray)  │
│ 🗑  Delete (red)      │
│ More              ›  │ ──► ┌──────────┐ flyout (end, 4px)
└──────────────────────┘     │ Sub item │ min 160
                             └──────────┘
```

BottomSheet presentation（`MenuBottomSheet.tsx`, `menuBottomSheet.stylex.ts`, `MenuBottomSheetActionList.tsx`）: BottomSheet（4 章）の content padding 0、menu 内容は上 16px / 左右 4px。行は ListItem（spacious）風、destructive は text-error、区切り margin-block 4px、セクション見出し padding-inline 12px。ドリルインは見出し行 [← Back] + 見出し（gap 4, mb 8）。

RTL: placement start/end、chevron の向き、フライアウト側（end = 左）が反転。サブメニュー開閉キーも反転（下記）。

### 11.5 Motion
- メニュー入場: layerAnimations[placement]（below: `opacity 0; translateY(−8px) scale(0.95)` → `1; 0; 1`）165ms（`--duration-fast-max`）ease-standard、fill backwards（`DropdownMenu.tsx:1738`）。フライアウト: layerAnimations.end（`translateX(−8px) scale(0.95)` から、RTL は +8px）同値（`DropdownMenuSubMenu.tsx:1394`）。
- 退場: **なし**（menu div に `opacity` 125ms transition が宣言されているが opacity を 0 にするルールは無く、popover は即時非表示）。
- 行の背景: 95ms ease-standard。
- reduced motion: fine 即時 / coarse 165ms フェード。
- bottom-sheet presentation: BottomSheet の motion。ドリルインのビュー切替はアニメなし（display 切替）。

### 11.6 Behaviour & a11y
キーボード（APG menu button, `DropdownMenu.tsx` + `useListFocus` + `useTypeahead`）:
| キー | 場所 | 動作 |
|---|---|---|
| Enter / Space / ↓ | トリガー（閉） | 開いて最初の有効行へ focus |
| ↑ | トリガー（閉） | 開いて**最後**の有効行へ |
| ↓ / ↑ | メニュー | 次 / 前の有効行（**端で循環** wrap） |
| Home / End | メニュー | 先頭 / 末尾 |
| PageDown / PageUp | メニュー | スクロールするメニューでページ送り（しなければ End/Home） |
| 文字 | メニュー | typeahead: label 先頭一致の次の行へ（description 等は対象外）、**750ms** で入力リセット |
| Enter / Space | 行 | 実行（`hasCloseOnSelect` なら閉じてトリガーへ focus） |
| → (RTL は ←) | サブメニュー行 | フライアウトを開き先頭行へ |
| ← (RTL は →) / Escape | フライアウト内 | フライアウトを閉じ親行へ |
| Escape | メニュー | 閉じてトリガーへ focus |
| Tab | メニュー | header/footer 無し: 閉じて（preventDefault せず）トリガーの次へ。有り: dialog 内循環 |

ポインタ（`useMenuPress`, `menuPressGesture.ts`）:
- マウスは **pointerdown で開く**。押したまま行へドラッグして離すとその行を実行（press-drag-release）。開いた押下は **300ms**（`MENU_PRESS_SETTLE_MS`）以上続かないと離しても実行しない。離した後 **400ms** は後続 click を無視（`CLICK_GUARD_WINDOW_MS`）。
- タッチ: トリガーに **500ms**（`LONG_PRESS_DURATION_MS`）指を置くと指の下で開く、移動許容 **13px**。通常タップは click で開閉。
- 行のハイライトはポインタ移動で focus を移す。スクロールするメニューの上下端 **24px** に指/ポインタを置くと 16ms ごとに 6px 自動スクロール。
- サブメニュー hover（`useMenuHover.ts`）: 入って **150ms** で開く、出て **200ms** で閉じる。親行からフライアウトの近い辺へ向かう「安全三角形」（辺に 8px の余裕）内の移動中は閉じタイマーを再始動。hover で開いた直後 **500ms** 内のクリックは閉じずに固定。閉じた直後 **300ms** は同じトリガーへの mouseenter を無視。タッチ（coarse）ではドリルイン（Back 行 "Back to {parent}"）。
- 外側押下・Escape で閉じる（layer スタック最上位のみ）。中クリック（リンク行）も閉じる。

ARIA: trigger `aria-haspopup=menu`（header/footer ありは dialog）、`aria-expanded`、`aria-controls`。menu `role=menu` 名前 = トリガー（aria-labelledby）、既定名 "Menu"。行 `menuitem` / `menuitemcheckbox` / `menuitemradio` + `aria-checked`、`aria-disabled`、全行 tabIndex −1（roving）。サブメニュー行 `aria-haspopup=menu` `aria-expanded`。グループ `role=group` + heading で名前。マウント時に既に open なら focus を動かさない。
テキスト: `soft.dropdown_menu.label` "Menu", `soft.dropdown_menu.back` "Back", `soft.dropdown_menu.back_to` "Back to {parent}", サブメニュー読込 `soft.spinner.loading` "Loading"。

### 11.7 Kivact 実装方針
- **RESTYLE `kivact::ui::menu`**（`.attach(trigger)`）。MenuRecipes: surface/sub_surface（bg popover, radius 12, shadow-low, padding 4, gap 2, max-height 300 スクロール, min-width = trigger / sub 160）, item（size 別 28/32/36、padding 4–8/8、radius 8、highlighted overlay-hover、pressed overlay-pressed、disabled 0.5、destructive text-error、focus ring 内側）, separator（1px border, ±4px）, label（12px secondary, padding 4/8）, shortcut（末尾、12px? → Astryx は endContent 任意。soft では supporting 12px text-secondary を推奨）, description（12/20 secondary）, check/radio 箱（indicator の見た目）, trigger（secondary ボタン + 開中 pressed 色 + chevronDown）。
- 行サイズ: `Variant` で `soft::MenuSize::Sm|Md|Lg`。
- サブメニューの hover 遅延: kivact は open 140ms / close 0（`kivact-interaction/src/menu/level.rs:19`, 205-209）。Astryx は 150 / 200 + 安全三角形 → interaction 側の設定追加が必要（UNRESOLVED: 遅延が設定可能か、safe triangle の有無）。
- press-drag-release / long-press 開: kivact menu の press model が対応していなければ UNRESOLVED（デスクトップでは click 開閉で可）。
- header/footer, bottom-sheet presentation, ドリルイン: kit wrapper（`soft::dropdown_menu`）で `kivact::ui::dialog(Popover)` + `list` に組み直す必要があり重い。header/footer は UNRESOLVED（優先度中）。
- 参考: `crates/kivact-ui/src/menu.rs`, `crates/kivact-interaction/src/menu/`, `crates/kivact-ui-basic/src/recipes_menu.rs`, `list_popup.rs`, `menubar.rs`。

### 11.8 受け入れ確認
- [ ] trigger（button, aria-haspopup=menu）で `role=menu` が開き先頭 menuitem に focus。↑ で開くと末尾。
- [ ] ↓ を押し続けると末尾から先頭へ循環。文字入力で該当行へ。Escape で閉じ trigger に focus。
- [ ] checkbox 行は押しても閉じず aria-checked が反転。radio 行は選ぶと閉じる。
- [ ] サブメニュー行 → で開き ← で閉じる。マウスで斜めに移動しても閉じない。
- [ ] disabled 行は飛ばされ、押しても何もしない。destructive は赤字。
- [ ] 目視: trigger 下 4px、角丸 12、影薄、行 32px・角丸 8・hover 5% overlay、区切り 1px、見出し 12px グレー、入場 165ms で上から 8px。dark は #1b1b1b + 8% 白リム。

## 12. ContextMenu

docs: https://astryx.atmeta.com/components/ContextMenu ・ source: `packages/core/src/ContextMenu/`（`ContextMenu.tsx` 914 行）。ContextMenuItem = DropdownMenuItem の再エクスポート（docs: https://astryx.atmeta.com/components/ContextMenuItem）

### 12.1 概要
右クリック（タッチは長押し）で、ポインタ位置に開く文脈メニュー。DropdownMenu の行描画・キーボード操作を再利用。

- 使う: 簡潔な動詞。セクションと区切り。adaptive でデスクトップはカーソル位置 / タッチは BottomSheet。全アクションを他の UI からも到達可能に。モバイルでは MoreMenu 等の見える導線も。
- 使わない: 重要操作の唯一の入口にする。10–12 件超をグループ化せずに。

### 12.2 Anatomy
```
<div|span trigger (position: relative; user-select none)>   ← children（右クリック領域）
   children
   <div cursorAnchor (absolute 1×1px at local x,y)>          ← メニューのアンカー
<layer popover anchored to cursorAnchor (below / start, offset 0)>
   <div role="menu" aria-label="Context menu" (surface)>  rows（DropdownMenu と同じ部品）
```

### 12.3 API
| prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `children` | ReactNode | 必須 | `menu(..).attach_context(target)`（既存） |
| `items` | ContextMenuOption[] | – | `{label, onClick?, icon?, isDisabled?, variant?, items?}` / divider / section → MenuEntry（既存） |
| `menuContent` | ReactNode | – | compound（同上） |
| `menuWidth` | number\|string | `'160px'` | `.min_width(Length)`、既定 160 |
| `size` | sm\|md\|lg | `'md'` | `.variant(soft::MenuSize::..)`（行 28/32/36） |
| `label` | string | "Context menu" | `.name(..)`、既定 `soft.context_menu.label` |
| `onOpenChange` | fn | – | `.on_open_change(f)` |
| `presentation` | popover\|bottom-sheet\|adaptive | `'popover'` | kit wrapper |
| `isDisabled` | bool | false | true でブラウザ既定の右クリックメニュー → ネイティブでは「開かない」 |
| `triggerXstyle` / `triggerAs` | | 'div' | 削除（`attach_context` に渡すノードで決まる） |

### 12.4 Visual spec
（`ContextMenu.tsx:120-215`）
- menu: flex column, gap 2px, padding 4px, max-height **300px**（スクロール）, radius `--radius-container` 12px, bg `--color-background-popover`（`#ffffff`/`#1b1b1b`）, shadow `--shadow-low`, **min-width 160px**。
- 行: DropdownMenu と同一（size md 32px、radius 8、hover overlay-hover、destructive text-error …, 11.4）。
- 位置: カーソル点（トリガー内ローカル座標の 1px アンカー）に対し placement **below** / alignment **start**、offset **0**（`ContextMenu.tsx:905-906`）→ メニューの左上角がカーソル直下。はみ出しは 15 章のフリップ（上へ / 左へ）。トリガーのスクロールに追従。
- trigger: 見た目変化なし（user-select none）。
- bottom-sheet: sheet 内 menu bg surface、上 20px（16+4）/ 左右 8px / 下 4px の inset。

### 12.5 Motion
入場: layerAnimations.below（`translateY(−8px) scale(0.95)` + opacity 0 → 1）165ms ease-standard。退場: なし（opacity 125ms transition 宣言のみ）。reduced: fine 即時 / coarse フェード。

### 12.6 Behaviour & a11y
- 開く: 右クリック（contextmenu）→ カーソル位置。キーボード: focus 中のトリガーで **Shift+F10 / ContextMenu キー** → トリガーの**左下**（ローカル (0, 高さ)）に開く（座標 (0,0) かつ detail 0 で判定）。タッチ: 長押し **500ms**（useLongPress、移動で取消）。
- 開いた時: 右クリック/長押しでは最初の行に focus（ring なし）、キーボード起動では ring あり。
- 操作: DropdownMenu と同じ（↑↓ 循環、Home/End、typeahead 750ms、Enter/Space 実行、→/← サブメニュー、Escape で閉じ trigger へ focus、Tab で閉じる）。
- 閉じる: 外側押下、Escape、行実行。開いた押下由来の後続 mousedown は外側押下とみなさない。
- ロール: `menu` 名前 "Context menu"。
- テキスト: `soft.context_menu.label` = "Context menu"。

### 12.7 Kivact 実装方針
- **RESTYLE `kivact::ui::menu(..).attach_context(target)`**（既存）。MenuRecipes::context_surface を surface と同じ（min-width 160, max-height 300）に。ポインタ位置配置は kivact の `attach_context` が `overlay_at(point)` 系で担う想定 → placement below/start, offset 0 を確認。
- Shift+F10 / Menu キー: kivact の attach_context が対応していなければ kit で `on_key` を付け trigger 左下に `overlay_at` で開く（UNRESOLVED: attach_context のキーボード起動の有無）。
- 参考: `crates/kivact-ui/src/menu.rs`（attach_context）, `crates/kivact-ui-basic/src/recipes_menu.rs`（context-menu.tsx の幅広トップレベル）。

### 12.8 受け入れ確認
- [ ] 対象上で右クリック → カーソル直下に `role=menu`（名前 "Context menu"）、最初の行に focus。
- [ ] 画面右下隅で右クリック → 上・左にフリップして画面内。
- [ ] focus したトリガーで Shift+F10 → トリガー左下に開く。
- [ ] Escape で閉じトリガーへ focus。
- [ ] 目視: 最小幅 160、角丸 12、行 32px、DropdownMenu と同じ見た目。

## 13. Lightbox

docs: https://astryx.atmeta.com/components/Lightbox ・ source: `packages/core/src/Lightbox/`（`Lightbox.tsx` 811 行, `useLightbox.tsx`）

### 13.1 概要
画像・動画を全画面で原寸表示するオーバーレイ。単体 / ギャラリー（前後移動）、画像の任意ズーム（2 倍トグル）とパン、動画はネイティブコントロール。

- 使う: すべての画像に alt。複数枚はギャラリー + onIndexChange。高解像度で拡大価値がある時だけ hasZoom。
- 使わない: 画像以外の内容。キャプションに対話要素（プレーンテキストに）。

### 13.2 Anatomy
```
<dialog aria-label=<current alt | "Media viewer"> (fixed inset 0, 100vw × 100dvh, 透明)>
  ::backdrop  scrim (overlay + blur 2px)
  <div container (flex column, center, 100%×100%, relative)>
    IconButton ghost "Close" [×]            ← absolute top 12 / inline-end 12
    IconButton ghost "Previous" [‹]          ← absolute top 50% / inline-start 12（ギャラリー時、先頭で disabled）
    <div mediaGroup (column, center, max 100%, overflow hidden)>
       <div imageWrapper [role=button aria-pressed aria-label="Zoom" if zoomable]>
          <img alt> | <video controls aria-label=alt>
       <div caption>  caption
    IconButton ghost "Next" [›]              ← absolute top 50% / inline-end 12
    <div counter>  "2 / 5"                   ← absolute top 12 / inline-start 12（2 枚以上）
```

### 13.3 API
| prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `isOpen` / `onOpenChange` | | 必須 | `soft::lightbox(open: Binding<bool>, media)` |
| `media` | `LightboxMedia \| LightboxMedia[]`（`{src, alt, caption?, type?: 'image'\|'video'}`） | 必須 | `impl IntoValue<Vec<LightboxMedia>>`。`LightboxMedia { image: ImageSource, alt: String, caption: Option<String>, kind: MediaKind::Image }`。動画は UNRESOLVED（kivact に video ノードが無ければ非対応として除外） |
| `index` / `onIndexChange` | number | – | `.index(Binding<usize>)` |
| `defaultIndex` | number | 0 | binding の初期値で代替 |
| `hasZoom` | bool | false | `.zoom(bool)` |
| `hasAutoPlay` | bool | false | 動画のみ（UNRESOLVED） |
| `xstyle` | | | `.style(..)` |

`useLightbox({media, ...})` → `{open(i=0), close, isOpen, index, element, triggerProps, getTriggerProps(i)}`（trigger は Enter/Space でも開く）。Kivact: `soft::use_lightbox(media)` → Copy ハンドル `.open(i)`, `.close()`, `.view()`, `.trigger(i)`（Part）。

### 13.4 Visual spec
（`Lightbox.tsx:130-265`）
- 全画面 dialog、背景透明、scrim `::backdrop` = `--color-overlay`（light `#00000080` / dark `#000000CC`）+ `backdrop-filter: blur(2px)`。
- メディア: `max-width/max-height 100%`, `object-fit: contain`、中央。
- キャプション: `--color-on-dark` `#ffffff`（両テーマ）、`--text-large-size` 17px / leading 24px（weight は継承 = body 400。text-large-weight 600 は適用されない）、中央揃え、padding 上 8px / 左右 12px / 下 0、max-width 600px。
- カウンタ: `#ffffff`、body 14/20、左上 12px（RTL で右上）。表記 "{index+1} / {total}"。
- ボタン: IconButton ghost md = 32×32、radius 10px、アイコン 16px（close / chevronLeft / chevronRight）、色 `--color-on-dark` #ffffff。hover: `--color-overlay-hover`（light `#0000000D` / dark `#FFFFFF0D`）。Close は右上 12px、Prev は左中央 12px、Next は右中央 12px（`translate: 0 -50%`）。先頭/末尾では Prev/Next が disabled（opacity 0.5）でアンマウントしない。
- カーソル: zoomable = zoom-in、ズーム中 = grab、ドラッグ中 = grabbing。

```
┌──────────────────────────────────────────────┐ scrim 黒50%/80% + blur 2
│ 2 / 5                                    [×] │
│                                              │
│ [‹]          ┌──────────────┐           [›]  │
│              │    image     │                │
│              └──────────────┘                │
│        A scenic coastline… (17px white)      │
└──────────────────────────────────────────────┘
```

### 13.5 Motion
- 開閉: **アニメなし**（dialog/scrim に animation/transition 指定なし）。
- ズーム / パン: img の `transform` = `scale(2) translate(pan.x/2 px, pan.y/2 px)`、transition `transform 200ms ease-out`（reduced motion 0ms）、ドラッグ中は transition none（`Lightbox.tsx:198-212`, 680-685 付近）。
- 画像切替: アニメなし。スワイプでの前後移動は**無い**。

### 13.6 Behaviour & a11y
- キー: ←/→ = 前/次（RTL では ← が次。端で止まる、循環しない）、ズーム可能時: `+`/`=` で 2 倍、`-` で 1 倍、画像（role=button）に focus して Enter/Space でトグル、ズーム中の矢印は **50px** ずつパン（矢印はギャラリー移動ではなくパンになる）。Escape で閉じる。
- ポインタ: ダブルクリックでズームトグル、ズーム中ドラッグでパン、scrim（dialog / container 自体）クリックで閉じる（ドラッグ直後のクリックは無視）。
- dialog: showModal（focus trap, top layer）、scroll lock、閉じたらトリガーへ focus。
- 名前: 現在の alt（無ければ "Media viewer"）。画像変更時は polite で "Image {i} of {n}"（alt があれば "{alt}, {i} of {n}"）を通知。ズーム変化時 "Zoomed in, use arrow keys to pan" / "Zoomed out"。
- テキスト: `soft.lightbox.close` "Close", `.previous` "Previous", `.next` "Next", `.media_viewer` "Media viewer", `.zoom` "Zoom", `.zoomed_in` "Zoomed in, use arrow keys to pan", `.zoomed_out` "Zoomed out", `.image_position` "Image {index} of {total}", `.media_position` "{alt}, {index} of {total}"。

### 13.7 Kivact 実装方針
- **NEW kit component `soft::lightbox`**: `kivact::ui::dialog` を使わず、`overlay(..).modal(true)` 全画面 + 自前 scrim（overlay 色 + `Filter::blur(2)` の backdrop filter）。ズーム/パンは `kivact::interaction::use_zoom_pan`（scale を 1↔2 の 2 段に制限）か、単純に transform signal + 200ms ease-out tween。
- 参考: `crates/kivact-ui-basic/src/carousel.rs`（前後ボタンと index）, `crates/kivact-ui-basic/src/layers.rs`（scrim）。
- 難所: 動画（UNRESOLVED）、`object-fit: contain`（kivact の image ノードの fit 指定で）。

### 13.8 受け入れ確認
- [ ] 開くと role=dialog（名前 = alt）、Close に到達可、Escape・scrim クリックで閉じトリガーへ focus。
- [ ] ギャラリー: → で次、末尾で Next が disabled、カウンタ "5 / 5"。
- [ ] hasZoom: ダブルクリックで 2 倍（200ms）、矢印でパン、`-` で戻る。
- [ ] 目視: 黒 50%（dark 80%）+ ぼかしの上に画像、白い 17px キャプション、四隅の白アイコン。

## 14. Overlay

docs: https://astryx.atmeta.com/components/Overlay ・ source: `packages/core/src/Overlay/`（`Overlay.tsx`, `OverlayScrim.tsx`, `useOverlay.tsx`, `overlay.markers.stylex.ts`）

**注意: Astryx の Overlay は「モーダルの背景 scrim」ではない**。画像・カード・動画など境界のある面の上に、短いアクションやラベルを scrim 付きで重ねるメディアオーバーレイ（hover / focus で出す）。Dialog/Drawer/Lightbox の scrim は各自の `::backdrop`（overlay 色 + blur 2px）、BottomSheet は blur なしで、この Overlay とは無関係。

### 14.1 概要
- 使う: 下の面に属する短い文脈アクション/ラベル（"Quick view" ボタン、タイトル帯）。内容はコンパクトに。
- 使わない: 面の外に浮く内容（Popover / Tooltip / Dialog）。

### 14.2 Anatomy
```
<div root overlayScope (position: relative; overflow: clip)>
  children（base: 画像・カード）
  <div OverlayScrim (absolute; flex column; gap 8; padding 12)>  content
```
`useOverlay(options)` は既存コンテナに付けるフック: `{containerRef, containerProps, element, renderOverlay}`。

### 14.3 API
| prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `content` | ReactNode | 必須 | `soft::media_overlay(|| content)` |
| `children` | ReactNode | – | `.base(|| view)` または `soft::media_overlay(..).attach(base)` |
| `showOn` | `'hover'\|'always'\|'focus'\|'hover-or-focus'` | `'always'` | `.show_on(ShowOn::Always|Hover|Focus)`（HoverOrFocus = Hover の別名） |
| `isOpen` | bool | – | `.open(impl IntoValue<bool>)`（showOn より優先） |
| `scrim` | `'dark'\|'light'\|false` | `'dark'` | `.scrim(OverlayScrim::Dark|Light|None)` |
| `position` | `'fill'\|'bottom'\|'top'` | `'fill'` | `.position(OverlayPosition::Fill|Bottom|Top)` |
| `align` | `'start'\|'center'\|'end'` | `'end'` | `.align(OverlayAlign)` |
| `xstyle`/`className`/`style`/`ref` | | | `.style(..)`, `.node_ref(..)` |

### 14.4 Visual spec
（`OverlayScrim.tsx:50-184`）
- scrim box: absolute、flex column、gap 8px、padding 12px、pointer-events none（表示中は auto）。
- position: fill = inset 0 / bottom = 下端の帯（左右 0, bottom 0, 高さは内容）/ top = 上端の帯。
- align（flex column なので alignItems = 横、justifyContent = 縦）: start = 左上（flex-start / flex-start）, center = 中央, **end = 右上**（alignItems flex-end, justifyContent flex-start。下寄せではない点に注意）。
- scrim 色: dark = `--color-overlay`（light `#00000080` / dark `#000000CC`）, light = `color-mix(white 60%, transparent)` = `rgba(255,255,255,0.6)`（両テーマ）, false = 背景なし。
- root: `position: relative; overflow: clip`（角丸は base 側が持つ）。

### 14.5 Motion
| 状態 | fill | bottom / top 帯 |
|---|---|---|
| 非表示 | opacity 0, visibility hidden | + translateY(100%) / translateY(−100%)（面の外へ） |
| 表示 | opacity 1, visible, translateY(0) | 同 |
| マウント時（always/isOpen） | @starting-style opacity 0 → 1 | opacity 0 + translateY(±100%) → 0 |
transition: `opacity, visibility, transform` `--duration-fast` **125ms** `--ease-standard`。reduced motion 0s。hover モードは祖先（root）の :hover（hover 可能環境のみ）または :focus-within で表示。

### 14.6 Behaviour & a11y
- hover モード: ホバーまたは root 内 focus（キーボード到達性）で表示。タッチ端末: 帯（bottom/top）は常時表示、fill はタップでトグル（useClickableContainer）。
- focus モード: root 内 focus-within のみ。
- isOpen が最優先、次にタッチトグル、最後に CSS。
- ロール付与なし（content 側の要素が自分のロールを持つ）。テキストなし。

### 14.7 Kivact 実装方針
- **NEW kit component `soft::media_overlay`**（kivact-ui-basic に直接の類似なし。近いのは `card.rs` の media 部分と `layers.rs` の scrim 色）。root を `stack`/relative にし、scrim を absolute 子で重ね、hover/focus-within を root の state style で切替（kivact の state style が「子孫の focus」を読めない場合は `on_focus_within` 相当のイベントで signal を立てる: UNRESOLVED）。
- 帯のスライドは translate y ±100%（自身の高さ基準）を 125ms で。

### 14.8 受け入れ確認
- [ ] showOn=hover: ホバーで 125ms フェードイン、離れると消える。Tab で中のボタンに focus すると表示。
- [ ] position=bottom: 下からスライドイン。
- [ ] 目視: dark scrim 黒 50%（dark テーマ 80%）、light scrim 白 60%、内容は既定で右上（align end）に 12px 内側。

## 15. Layer / useLayer（レイヤー基盤・配置規則）

docs: https://astryx.atmeta.com/components/Layer ・ https://astryx.atmeta.com/components/useLayer ・ source: `packages/core/src/Layer/`（`useLayer.tsx` 2052 行, `layerSlideRules.ts`, `layerViewportInset.stylex.ts`, `layerAnimations.stylex.ts`, `layerStack.ts`, `useLayerDismissal.ts`, `outsidePress.ts`, `LayerProvider.tsx`, `layerInset.ts`, `useTouchTrigger.ts`, `usePressPin.ts`, `modalOutlet.ts`）

### 15.1 概要
Popover / HoverCard / Tooltip / DropdownMenu / ContextMenu / Selector などが共有する「浮遊レイヤー」の土台。ブラウザの Popover API（top layer）+ CSS anchor positioning で配置し、共通の dismiss スタック・ビューポート余白・入場アニメを提供。`LayerProvider` はアプリ直下に 1 つ（toast 設定と inset 宣言）。一般利用者は上位部品を使い、useLayer は部品作者向け。

### 15.2 Anatomy
```
trigger (anchor-name: --<anchorId>)
<template marker>   ← JSX 位置。親が安全なら同位置に、危険（<p> 等）なら補正 portal へ
<div popover="auto|manual" id role? aria-label? (position-anchor, position-area, position-try-fallbacks, margins, viewportFit)>
   <LayerContentBoundary> children
```
mode: `'context'`（アンカー配置）/ `'fixed'`（`{x, y}` の top/left 直接指定）。

### 15.3 API
`useLayer({mode, onShow, onHide, lightDismiss=false, lazyMount})` → `{ref(trigger), anchorId, show, hide, isOpen, id, render(children, props)}`。
`render` props（context）: `positioning: 'anchor'|'custom'`（既定 anchor）, `placement: 'above'|'below'|'start'|'end'`（既定 **above**）, `alignment: 'start'|'center'|'end'`（既定 **center**）, `offset`（px/長さ, 既定 **0**; 標準クリアランスは `--spacing-1` 4px）, `role`, `aria-label`, `xstyle`, `className`, `style`, `as: 'div'|'span'`, `onMouseEnter/Leave`。fixed: `x, y, xstyle, className, style`。
`LayerProvider({toast: {position, maxVisible, inset}, inset: {blockStart, blockEnd, inlineStart, inlineEnd}})`。

Kivact 対応: `kivact::view::overlay(anchor: NodeRef, Placement, child).align(OverlayAlign).offset(px).modal(bool).on_dismiss(f)`、`overlay_at(point, ..)`（fixed）が既存（`kivact-core/src/view/overlays.rs:129-211`）。soft は独自 API を作らず、各部品の既定値（offset 4 等）と入場アニメ recipe を共通 helper `soft::layer_enter(placement)` として持つ。`LayerProvider.inset` → `soft::layer_inset(Insets)` を `provide` で配る（kivact overlay のクランプ余白に反映できれば。UNRESOLVED）。

### 15.4 Visual spec — 配置アルゴリズム（正確な規則）
1. **基本位置**（`getPositionArea`, `useLayer.tsx:964-1007`）: 論理 self-* キーワード:
   | placement | alignment start | center | end |
   |---|---|---|---|
   | below | `self-block-end span-self-inline-end`（左端揃えで右へ伸びる） | `self-block-end`（中央） | `self-block-end span-self-inline-start`（右端揃え） |
   | above | `self-block-start span-self-inline-end` | `self-block-start` | `self-block-start span-self-inline-start` |
   | end | `self-inline-end span-self-block-end`（上端揃え） | `self-inline-end` | `self-inline-end span-self-block-start` |
   | start | `self-inline-start span-self-block-end` | `self-inline-start` | `self-inline-start span-self-block-start` |
   start/end はレイヤー自身の書字方向で解決（RTL で自動反転）。
2. **クリアランス（offset）と余白（gutter）**（`useLayer.tsx:84-112`）: 配置軸のアンカー側の margin = offset、反対側の margin = gutterBlock / gutterInline。flip 時は両者が入れ替わる（クリアランスは常にアンカー側）。
3. **ビューポート余白**（`layerViewportInset.stylex.ts`）: 各辺 `max(16px(--spacing-4), safe-area) + アプリ宣言 inset`。ネイティブでは **16px + inset**。レイヤー箱の最大 = ビューポート幅 − 2×16 / 高さ − 2×16（`viewportFit`, `useLayer.tsx:113-131`）。中身が大きければレイヤー内部でスクロール/クリップ（部品が決める）。整列軸の遠い側にも gutter（center は両側, `getGutterStyles`）。
4. **フォールバック順**（`getPositionTryFallbacks`, `useLayer.tsx:1009-1043`）: 収まらない時に順に試す:
   - 非 center: `flip-block` → `flip-inline` → `flip-block flip-inline` → slide(same) → slide(opposite)
   - center + above/below: 上記 3 flip → `<same> span-left` → `<same> span-right` → `<opposite> span-left` → `<opposite> span-right` → slide(same) → slide(opposite)
   - center + start/end: 3 flip → `<same> span-top` → `span-bottom` → `<opposite> span-top` → `span-bottom` → slides
   - **slide** = 整列軸全体にまたがる配置（アンカー中心寄せ + ブラウザの overflow shift でビューポート内に押し込む、両端に gutter）。同じ側を先に、次に反対側（`layerSlideRules.ts`）。
5. **アンカーが画面外**（`getSelfAlignment`, 1045-1066）: 非 center 整列は `unsafe self-start/end` に固定して画面端に押し込まない。
6. **アンカーを覆わない（Tooltip のみ, `keepsAnchorClear`）**: 決定後の箱がアンカーを覆ったら第 2 リスト: `beside end` → `beside start` → `flip-block` → `fit 同側` → `fit 反対側`（fit = その側の余白に高さを合わせ内部スクロール、最小は上下余白の半分）。
7. **矢印**: Astryx のレイヤーには矢印（arrow）は**無い**。

ネイティブ近似（kivact overlay ホスト: 収まらなければ反対側へ flip、ウィンドウ内にクランプ、start/end 整列は溢れたら反対へ）: 1・2・4 の flip とクランプは既存挙動で満たせる。不足: ウィンドウ端 16px の gutter（クランプ余白）、slide の span 系、6 の anchor-clear。→ UNRESOLVED（kivact-core の overlay に「viewport margin」パラメータ追加を提案）。

### 15.5 Motion（共通入場 `layerAnimations.stylex.ts`）
| placement | from | to |
|---|---|---|
| below | `opacity 0; translateY(−8px) scale(0.95)` | `opacity 1; translateY(0) scale(1)` |
| above | `translateY(+8px) scale(0.95)` | 同 |
| end | `translateX(−8px) scale(0.95)`（RTL: +8px） | 同 |
| start | `translateX(+8px) scale(0.95)`（RTL: −8px） | 同 |
- 8px = `--spacing-2`。duration `--duration-fast-max` = **165ms**（neutral）、easing `--ease-standard` cubic-bezier(0.24, 1, 0.4, 1)、`animation-fill-mode: backwards`、transform-origin は既定（中心）。
- reduced motion + fine pointer: なし（即時）。reduced motion + coarse: opacity のみ 0→1、`min(165ms, 200ms)`。
- 退場アニメは共通では**無い**（即時非表示）。
- 注意: 入場方向はフリップ前の placement で決まる（CSS は実際に選ばれた fallback を知らない）。フリップして上に出ても「上から」ではなく placement=below の「上から 8px 降りる」動きのまま（Astryx の実挙動）。ネイティブでは実際の側に合わせてもよいが、合わせるなら差分として記録。

### 15.6 Behaviour & a11y
- **dismiss スタック**（`layerStack.ts`, `useLayerDismissal.ts`）: 開いているレイヤーは 1 つのスタックに登録され、Escape は**最上位の 1 枚だけ**を閉じる（1 回の押下で 1 枚）。document ごとに 1 つの keydown（bubble 相）。中身が `preventDefault`/`stopPropagation` した Escape は無視（エディタ優先）。IME 変換中の Escape は無視。`escapeBehavior: 'close' | 'block'`（block = 閉じないが下にも渡さない）。
- **外側押下**（`outsidePress.ts`）: lightDismiss のレイヤーはレイヤー外の押下で閉じる。トリガー上の押下は除外（トグルが二重発火しない）。押下はレイヤー内から周囲の祖先ハンドラへ伝播しない（`layerClickBoundary`）。
- **top layer の順序**: 後に開いたものが前面（ブラウザの chronological order）。モーダル中のレイヤーはモーダルの中へ再ホスト（`modalOutlet.ts`）。
- **タッチ**（`useTouchTrigger.ts`, `usePressPin.ts`）: ホバー無し環境で tap を開く/トリガーに譲るの判定（`touchTrigger` 参照）、押下ピン留め。
- **テキストリセット**: レイヤー内は body 14px/20px 400 Figtree から開始（`layerTextReset.stylex.ts`、色は継承）。祖先の surface/group 文脈は持ち込まない。

### 15.7 Kivact 実装方針
- 新規部品は作らない（**covered by `kivact::view::overlay` / kivact::ui の各部品**）。soft 側で持つもの: (a) 入場アニメの共通 recipe helper（上表）、(b) 部品ごとの既定 offset（Popover/HoverCard/Tooltip/DropdownMenu = 4px、ContextMenu = 0、SubMenu = 4px）、(c) ビューポート gutter 16px。
- Escape 1 枚ずつ / 外側押下 / トリガー除外は kivact の `on_dismiss` と `kivact::interaction` の popup/dialog が担う（既存）。

### 15.8 受け入れ確認
- [ ] Popover の中で Tooltip を出し Escape → Tooltip だけ閉じる、2 回目で Popover。
- [ ] 下端のトリガーの below メニューは上にフリップ、画面端から 16px 以上離れる。
- [ ] 幅がウィンドウより広いレイヤーはウィンドウ幅 − 32px に収まる。
- [ ] RTL で placement=end のフライアウトが左側に出る。

## 16. Hooks: useFocusTrap / useScrollLock / useAnnounce

docs: https://astryx.atmeta.com/components/useFocusTrap ・ https://astryx.atmeta.com/components/useScrollLock ・ https://astryx.atmeta.com/components/useAnnounce ・ source: `packages/core/src/hooks/useFocusTrap.ts`, `useScrollLock.ts`, `useAnnounce.ts`, `ModalAnnounceProvider.tsx`

いずれも視覚を持たないフック。Kivact では基盤（kivact-core / kivact::interaction）が既に同等機能を持つため **covered by kivact 基盤**。soft は新 API を作らず、各部品がその挙動を満たしているかを受け入れ確認で見る。

### 16.1 useFocusTrap
- API: `useFocusTrap({isActive, onEscape?})` → `{containerRef, focusFirst()}`。
- 挙動: WAI-ARIA dialog の focus trap。document の focus を監視し、コンテナ内からの Tab/Shift+Tab がコンテナ外へ出たら内側の先頭/末尾へ戻す（循環）。**focus がコンテナ外にある時の Tab はページに任せる**（非モーダル面がページの Tab 順を奪わない）。マウスの外側クリックは横取りしない（light dismiss は別）。無効化/アンマウント時、起動前に focus していた要素へ戻す（ただし既に他へ移っている、または一度もトラップに入っていなければ戻さない）。Escape は共有 dismiss スタック経由（`useLayerDismissal`）。
- 使う: モーダル。使わない: tooltip / dropdown（light dismiss で足りる）。
- Kivact: `overlay(..).modal(true)` と `kivact::interaction::FocusReturn`（`focus_return.rs`）で既存。受け入れ: モーダル内で Tab が循環、閉じて元へ戻る。

### 16.2 useScrollLock
- API: `useScrollLock(isLocked: boolean)`。
- 挙動: body を `position: fixed` で固定して背景スクロールを止め（iOS 対策）、解除で元のスクロール位置に戻す。デスクトップではスクロールバー分のガターを `scrollbar-gutter: stable` で保持し、ページ（fixed 要素含む）が横にずれない。複数同時ロックは参照カウント。
- 使う: 全画面モーダル（Dialog, モーダル Drawer, モーダル BottomSheet, Lightbox）。使わない: popover / tooltip。
- Kivact: モーダル overlay 中は背後の入力がブロックされるため、ホイールで背後がスクロールしないことを確認（`modal(true)` の既存挙動。スクロールバー消失によるレイアウトずれはネイティブでは発生しない）。

### 16.3 useAnnounce
- API: `useAnnounce({hasReplace=false})` → `announce(message, politeness='polite'|'assertive')`。
- 挙動: 常設の視覚的に隠れた live region（polite / assertive の 2 つ、初回使用時に空で作成し常駐）へ書く。**2000ms 後に自動クリア**（`CLEAR_DELAY_MS`, `useAnnounce.ts:120`）。同一フレームの複数メッセージは順に結合、同一の繰り返しは 1 回。`hasReplace` インスタンスは未読の前回メッセージを置換（検索件数など）。空文字はクリア。開いている Astryx モーダル内から呼ぶと、そのモーダル内の region に届く（ネイティブ dialog はページの region を隠すため）。閉じる瞬間の通知は閉じた後に露出する region へ。
- 使う: 視覚だけで伝わる状態（件数、"No results"、読込、保存完了、検証エラー: WCAG 4.1.3）。polite が基本、assertive はエラー/緊急のみ。結果を通知（"12 results"）。使わない: 既に DOM にラベル付きで見えている内容の重複通知。
- Kivact: `kivact-core` の `LiveRegion`（`NodeExt::live_region`, `primitive/node_ext.rs:223`）で常設 region を soft が 1 つ持つ helper `soft::announce(text, Politeness)` を提供（2 秒後クリア、モーダル内なら最上位モーダル内の region へ）。UNRESOLVED: kivact に imperative announce API があるか（無ければ kit の常設 live region ノードに書き込む方式）。

### 16.4 受け入れ確認
- [ ] Dialog 内で Tab/Shift+Tab が循環し、閉じると元のボタンへ戻る。
- [ ] 非モーダル Drawer では Tab がページへ抜けられる。
- [ ] CommandPalette で検索すると "N results" が polite で通知され、2 秒後に region が空になる。
- [ ] Toast error は assertive で通知。

