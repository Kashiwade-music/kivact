# soft-ui work order 05: CONTENT グループ

Astryx（`D:\w\meta\astryx`、neutral テーマ）の Content 系部品と Utilities を、Kivact の `crates/kivact-ui-soft` に移すための仕様。token は Astryx の名前で書く（`packages/core/src/theme/tokens.stylex.ts`）。各章で token が最初に出るところに、neutral の解決値を light / dark で書いてある（`packages/themes/neutral/src/neutralTheme.ts` と `neutralPalettes.generated.ts` から解決）。

## 目次
- [グループ概要表](#グループ概要表)
- [共通事項](#共通事項)
- [1. Avatar](#1-avatar)
- [2. AvatarStatusDot](#2-avatarstatusdot)
- [3. AvatarGroup](#3-avatargroup)
- [4. AvatarGroupOverflow](#4-avatargroupoverflow)
- [5. Thumbnail](#5-thumbnail)
- [6. EmptyState](#6-emptystate)
- [7. VisuallyHidden](#7-visuallyhidden)
- [8. Text](#8-text)
- [9. Heading](#9-heading)
- [10. Blockquote](#10-blockquote)
- [11. Citation](#11-citation)
- [12. Code（inline code）](#12-codeinline-code)
- [13. Icon](#13-icon)
- [14. CodeBlock](#14-codeblock)
- [15. Markdown](#15-markdown)
- [16. Timer](#16-timer)
- [17. Timestamp](#17-timestamp)
- [18. Utilities](#18-utilities)（InternationalizationProvider、useTranslator / useLocale、**英語の既定文言カタログ（18.3）**、Theme / useTheme、MediaTheme、SyntaxTheme、useClipboard、useImageMode、useStreamingText、useDevWarning、useMergedRefs、useHotkeys、useAnimatedLayout / useEntryAnimation）

## グループ概要表
| Astryx component (and parts) | docs URL | source dir | kivact strategy | closest kivact-ui-basic analog |
|---|---|---|---|---|
| Avatar (wrapper, content, initials/icon fallback, status slot) | https://astryx.atmeta.com/components/Avatar | packages/core/src/Avatar | NEW kit component (name tooltip = RESTYLE `kivact::ui::tooltip`) | `crates/kivact-ui-basic/src/avatar.rs` (`avatar`) |
| AvatarStatusDot (dot, ring/minus glyph, icon) | https://astryx.atmeta.com/components/AvatarStatusDot | packages/core/src/Avatar | NEW kit component (Avatar の部品) | `avatar.rs` (`Avatar::badge`) |
| AvatarGroup (group, keyboard hint) | https://astryx.atmeta.com/components/AvatarGroup | packages/core/src/AvatarGroup | NEW kit component | `avatar.rs` (`avatar_group`) |
| AvatarGroupOverflow ("+N" span/button) | https://astryx.atmeta.com/components/AvatarGroupOverflow | packages/core/src/AvatarGroup | NEW kit component (AvatarGroup の部品) | `avatar.rs` (`avatar_group_count`) |
| Thumbnail (image/skeleton/placeholder, Open button, Remove button, upload overlay) | https://astryx.atmeta.com/components/Thumbnail | packages/core/src/Thumbnail | NEW kit component (Remove = RESTYLE `kivact::ui::button`、tooltip = `kivact::ui::tooltip`) | `attachment.rs`, `primitives.rs` (skeleton, spinner) |
| EmptyState (icon, title heading, description, actions) | https://astryx.atmeta.com/components/EmptyState | packages/core/src/EmptyState | NEW kit component | `empty.rs` (`empty_state`) |
| VisuallyHidden | https://astryx.atmeta.com/components/VisuallyHidden | packages/core/src/VisuallyHidden | ほぼ不要（`.name()` / `.description()` で足りる）。live region 用の小さな kit helper `visually_hidden` | なし |
| Text (root, truncation tooltip) | https://astryx.atmeta.com/components/Text | packages/core/src/Text | NEW kit component（tooltip = RESTYLE `kivact::ui::tooltip`） | `typography.rs` (`paragraph`, `muted`, `small`) |
| Heading (level 1-6, display types) | https://astryx.atmeta.com/components/Heading | packages/core/src/Heading（doc は Text/Heading.doc.mjs） | NEW kit component | `typography.rs` (`h1`〜`h4`) |
| Blockquote (cite) | https://astryx.atmeta.com/components/Blockquote | packages/core/src/Blockquote | NEW kit component | `typography.rs` (`blockquote`) |
| Citation (label / number, icon) | https://astryx.atmeta.com/components/Citation | packages/core/src/Citation | NEW kit component | `badge.rs` |
| Code (inline) | https://astryx.atmeta.com/components/Code | packages/core/src/Code | NEW kit component | `typography.rs` (`inline_code`) |
| Icon (registry, sizes, colours) | https://astryx.atmeta.com/components/Icon | packages/core/src/Icon | 既存の `kivact::control::icon` + テーマの `IconSet`（kit が Lucide を登録）+ size/color の kit style helper | `icons.rs` (`ICON_NAMES`) |
| CodeBlock (header/title/language label, copy button, line-number gutter, highlighted lines, diff rows, collapse region, scroll viewport, corner cluster) | https://astryx.atmeta.com/components/CodeBlock | packages/core/src/CodeBlock | NEW kit component（copy = RESTYLE `kivact::ui::button` + `tooltip`、collapse = `use_disclosure`） | `code_block/view.rs`（+ `rust.rs`, `lines.rs`） |
| Markdown (root, headings + heading links, paragraph, lists/task lists, table, code, blockquote, hr, image, citation, inline marks) | https://astryx.atmeta.com/components/Markdown | packages/core/src/Markdown | NEW kit component（soft の Text/Heading/Code/CodeBlock/Blockquote/Citation/Table/checkbox で組む） | `typography.rs` |
| Timer | https://astryx.atmeta.com/components/Timer | packages/core/src/Timer | NEW kit component | なし（表示部は `typography.rs` `muted`） |
| Timestamp (text, hover card rows, copy button) | https://astryx.atmeta.com/components/Timestamp | packages/core/src/Timestamp | NEW kit component（kit の hover card と icon button を使う） | `hover_card.rs` |
| InternationalizationProvider | https://astryx.atmeta.com/components/InternationalizationProvider | packages/core/src/i18n | 既存 facility（`provide_locale` / `provide_messages` / `provide_direction`）+ ICU 用 kit helper | README の文言表 |
| useTranslator / useLocale（useDirection, useCollator, useListFormatter） | https://astryx.atmeta.com/components/useTranslator , https://astryx.atmeta.com/components/useLocale | packages/core/src/i18n | 既存 facility（`message` / `message_with` / `locale` / `current_direction`）。collator と list formatter は kit helper | なし |
| Theme / useTheme | https://astryx.atmeta.com/components/Theme , https://astryx.atmeta.com/components/useTheme | packages/core/src/theme | 既存 facility（`App::theme` / `provide_theme`）+ `soft::theme(ThemeMode)` + `soft::Tokens` | `theme.rs`, `tokens.rs` |
| MediaTheme | https://astryx.atmeta.com/components/MediaTheme | packages/core/src/theme | kit helper（`provide_theme` 境界） | なし |
| SyntaxTheme | https://astryx.atmeta.com/components/SyntaxTheme | packages/core/src/theme/syntax | kit helper（`soft::Tokens` の syntax 色） | `tokens.rs` (`CodeColors`) |
| useClipboard | https://astryx.atmeta.com/components/useClipboard | packages/core/src/hooks | kit helper（`platform::copy_text` の上） | `code_block.rs`（copy 要求の callback） |
| useImageMode | https://astryx.atmeta.com/components/useImageMode | packages/core/src/hooks | ネイティブでは不要（app が mode を渡す） | なし |
| useStreamingText | https://astryx.atmeta.com/components/useStreamingText | packages/core/src/hooks | kit helper（`frame_loop`） | なし |
| useDevWarning | https://astryx.atmeta.com/components/useDevWarning | packages/core/src/hooks | 不要（kit 内部の debug 報告） | なし |
| useMergedRefs | https://astryx.atmeta.com/components/useMergedRefs | packages/core/src/hooks | 不要（ref が無い） | なし |
| useHotkeys | https://astryx.atmeta.com/components/useHotkeys | packages/core/src/hooks | kit helper（root の `on_key`） | なし |
| useAnimatedLayout / useEntryAnimation | https://astryx.atmeta.com/components/useAnimatedLayout , https://astryx.atmeta.com/components/useEntryAnimation | packages/core/src/hooks, packages/core/src/motion | useEntryAnimation = 既存の style `enter` + `transition`。useAnimatedLayout は UNRESOLVED（layout 補間が無い） | なし |

## 共通事項
- **neutral の基礎値**（この文書の全章で使う）:
  - フォント: body と heading は `Figtree, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif`。code は `ui-monospace, "SF Mono", Monaco, Consolas, "Liberation Mono", "Courier New", monospace`。フォントの登録はアプリが行う（kivact-ui-basic と同じ）。
  - type scale は base 14 / ratio 1.2。値の表は 8.4（Text）と 9.4（Heading）。
  - spacing は rem で、16px root のとき step × 4px（`--spacing-0-5` 2px、`--spacing-1` 4px、`--spacing-1-5` 6px、`--spacing-2` 8px、`--spacing-3` 12px、`--spacing-4` 16px、`--spacing-5` 20px、`--spacing-6` 24px、`--spacing-8` 32px、`--spacing-12` 48px）。
  - radius（neutral）: `--radius-inner` 6px、`--radius-element` 10px、`--radius-container` 12px、`--radius-page` 28px、`--radius-full` 9999px。
  - size: `--size-element-sm` 28px、`-md` 32px、`-lg` 36px。`--border-width` 1px。
  - motion（neutral の `motion: {fast:125, medium:300, slow:700, ratio:0.75}` を 5ms 単位に丸めた値）: `--duration-fast-min` 95ms、`--duration-fast` 125ms、`--duration-fast-max` 165ms、`--duration-medium-min` 225ms、`--duration-medium` 300ms、`--duration-medium-max` 400ms、`--duration-slow-min` 525ms、`--duration-slow` 700ms、`--duration-slow-max` 935ms。`--ease-standard` / `--ease-entrance` / `--ease-move` = `cubic-bezier(0.24, 1, 0.4, 1)`、`--ease-exit` = `cubic-bezier(0.3, 0, 0.6, 0.6)`。
  - focus ring: `--focus-outline-width` 2px、solid、色 `--color-accent`（neutral `#1b1b1b` / `#f1f1f1`）、`--focus-outline-offset` 3px。
  - よく使う色（light / dark）: `--color-text-primary` `#000000` / `#ffffff`、`--color-text-secondary` `#474747` / `#9e9e9e`、`--color-text-disabled` `#919191` / `#525252`、`--color-background-surface` `#ffffff` / `#262626`、`--color-background-body` `#f1f1f1` / `#1b1b1b`、`--color-background-muted` `#f1f1f1` / `#1b1b1b`、`--color-border` `#00000014` / `#ffffff1A`、`--color-border-emphasized` `#d4d4d4` / `#525252`、`--color-overlay-hover` `#0000000D` / `#ffffff0D`、`--color-overlay-pressed` `#0000001A` / `#ffffff1A`、`--color-accent` `#1b1b1b` / `#f1f1f1`、`--color-accent-muted` `#f1f1f1` / `#262626`。
- **共用の型**: `TextKind`、`TextSize`、`TextColor`、`TextWeight`（8.3）は Heading・Timer・Timestamp・Code・Markdown で共用する。`IconSize` / `IconColor`（13.3）は全部品で共用する。
- **文言**: kit の部品が出す英語は `kivact::i18n::message("soft.<component>.<name>", "<English>")` で読む。全キーの表は 18.3。ICU の plural / number は 18.1 の kit helper で扱う。
- **直接使う Kivact の facility**: `kivact::control::{text, icon, node, row, column}`、`kivact::view::{provide_theme, provide_locale, provide_messages, provide_direction, current_direction}`、`kivact::i18n::{message, message_with, locale}`、`kivact::platform::{copy_text, reduced_motion}`、`kivact::task::{spawn, sleep}`、`kivact::motion::{frame_loop, Transition, Easing}`、style の `single_line()`、`line_clamp(n)`、`tabular_nums()`、`text_decoration(..)`、`enter(..)` / `exit(..)` / `transition(..)`。
- **Kivact に無いと分かったもの**（各章で UNRESOLVED として扱う）: テキストが切り詰められたかの問い合わせ、画像読み込み失敗の通知、OS のダーク / ライト外観、タイムゾーン、Intl 相当（日付・数値・複数形・grapheme 分割）、layout 変化の補間、文字色の transition。

---

## 1. Avatar

ソース: `packages/core/src/Avatar/Avatar.tsx`（945 行）, `AvatarSizeContext.ts`, `AvatarStatusLabelContext.ts`, `Avatar.doc.mjs`, `Avatar.test.tsx`, stories `apps/storybook/stories/Avatar.stories.tsx`。
docs: https://astryx.atmeta.com/components/Avatar

### 1.1 概要

人またはチームを、プロフィール写真 → イニシャル → 既定の人型アイコン の順のフォールバックで表す小さな円形（既定）メディア。コメントヘッダ、連絡先リスト、チャット、ユーザーカードで使う。

- する: 常に `name` を渡す（写真失敗時のイニシャル生成と SR 名の両方に使う）。文脈に合ったサイズ（インライン言及は `xsm`/`sm`、リスト・カードは `md`/`lg`、プロフィールヘッダは `xl`）。在席が重要な場面（チャット、チーム一覧）でのみ status dot を付ける。自前の Tooltip / HoverCard で包むときは `tooltip(false)`。インタラクティブ（href / onClick）な avatar には必ず `name` か `alt`。
- しない: status ラベル（"Online"）だけでインタラクティブ avatar を名付けない（リンク先を表さない）。ロゴや製品画像には使わない（画像やアイコンを使う）。形状を style で上書きせず `shape` を使う。

### 1.2 Anatomy

```
wrapper  (div[role=img|presentation] / a / button; position:relative; inline-flex; size×size)
├─ content  (div; 100%×100%; overflow:hidden; radius=--_avatar-radius)
│   ├─ img (src)            … 読み込み成功時のみ。alt=""（名前は wrapper が持つ）
│   ├─ img (fallbackSrc)    … src 失敗時のみ
│   ├─ fallback (initials)  … 画像が無い/両方失敗 & initials が空でない
│   └─ fallback (DefaultIcon svg) … 画像なし & initials が空
└─ status slot (div; position:absolute) … `status` が truthy のときのみ
(sibling) Tooltip layer   … showTooltip のときのみ（DOM ラッパは作らない）
```

- 静的 avatar: `div`。名前あり → `role="img"` + `aria-label`、名前なし → `role="presentation"` + `aria-hidden="true"`（Avatar.tsx:497-505）。tooltip が有効でグループ外なら `tabIndex=0`（Avatar.tsx:916）。
- `href` あり → link 要素（`data-avatar-item`）、`onClick` のみ → `<button type="button">`（`data-avatar-item`）。link/button は role を持つので名前（`aria-label`）だけを移す（Avatar.tsx:636-638, 871-905）。
- Context: `AvatarSizeContext`（数値 px、既定 36 = md、AvatarSizeContext.ts:23）を子（AvatarStatusDot）へ、`AvatarStatusLabelContext`（status ラベル報告用 ref box `{label, update}`）を子へ提供。`AvatarGroupContext` があればそちらの size/shape を優先して読む（Avatar.tsx:639-641）。

### 1.3 API

| Astryx prop | 型 | 既定 | 意味 | Kivact |
|---|---|---|---|---|
| `name` | string | – | イニシャル生成、既定の alt と tooltip | `avatar().name(impl IntoText)`。空白のみは「無し」と同じ扱い（`meaningful`, Avatar.tsx:482） |
| `alt` | string | – | SR 名（`name` より優先）。visible tooltip は `name` を使う点に注意 | `.alt(impl IntoText)` |
| `src` | string | – | 主画像 | `.src(impl Into<Asset>)` |
| `fallbackSrc` | string | – | 主画像失敗時の画像 | `.fallback_src(impl Into<Asset>)` |
| `size` | `'xsm'|'sm'|'md'|'lg'|'xl'|16|20|24|32|36|40|48|60|64|72|96|128|144|180` | `'md'` | 一辺 px。xsm 20 / sm 24 / md 36 / lg 48 / xl 128（Avatar.tsx:115-131）。グループ内ではグループの size が常に勝つ | `.size(AvatarSize)`; `enum AvatarSize { Xsm, Sm, #[default] Md, Lg, Xl, Px(u16) }`。`Px` は上記 14 値のみ有効（debug ビルドで許可外の値を `ErrorKind::Unsupported` として報告し、最も近い値に丸める） |
| `shape` | `'circle'|'rounded'|'square'` | `'circle'` | 角丸 | `.shape(AvatarShape)`; `enum AvatarShape { #[default] Circle, Rounded, Square }` |
| `status` | ReactNode | – | 右下（circle は 4 時半）に置くコンテンツ。AvatarStatusDot がラベルを報告 | `.status(impl FnOnce() -> V + 'static)`（slot closure） |
| `tooltip` | `string | boolean` | `true` | true/省略=name を表示、文字列=その文字列、false=なし | `.tooltip(AvatarTooltip)`; `enum AvatarTooltip { #[default] Name, Text(Value<String>), Off }`、`impl From<bool>` / `From<&str>` を用意 |
| `href` | string | – | リンク化 | ネイティブに URL 遷移は無い → `.on_open(impl Fn() + 'static)` で Role::Link（遷移はアプリ責務）。`as`/`target`/`rel` は **drop** |
| `as`, `target`, `rel` | – | – | link 用 | drop（意味なし） |
| `onClick` | handler | – | `<button>` 化 | `.on_press(impl Fn() + 'static)` → Role::Button。`on_open` と両方あれば `on_open` 優先（href wins, テスト「href wins over onClick」） |
| `xstyle` / `className` / `style` | – | – | – | `NodeBuilder` の `.style(..)`（kit 既定の後に適用され勝つ）。className は drop |
| `data-testid` | – | – | – | `.test_id(..)` |
| `aria-label` / `role` / `aria-hidden` (consumer) | – | – | 派生値より consumer が勝つ | `.name(..)` / `.role(..)` を NodeBuilder で上書き可能にする |

Builder 例:

```rust
soft::avatar()
    .name("Jane Doe")
    .src("jane.jpg")
    .size(soft::AvatarSize::Lg)
    .status(|| soft::avatar_status_dot(soft::AvatarStatus::Success).label("Online"))
    .on_press(move || open_profile())
```

関数シグネチャは `pub fn avatar() -> Avatar`（kivact-ui-basic の `avatar(fallback)` と異なり、Astryx は name からイニシャルを導くので引数なし）。`Avatar: NodeBuilder + IntoView`。

イニシャル算出（Avatar.tsx:418-457）: `name.trim()` を空白で分割 → 各語の「最初の文字・数字・絵文字（`\p{L}\p{N}\p{Extended_Pictographic}\p{Regional_Indicator}`）」を書記素単位で取り出す（先頭の記号 `(` `"` `-` は飛ばす。該当なしの語は捨てる）→ 0 個なら ''、1 個ならそれを大文字化、2 個以上なら「最初 + 最後」を大文字化。例: `John Doe`→`JD`, `Alice`→`A`, `Northwind Workbench (automation)`→`NA`, `—` → ''（→ 既定アイコン）。絵文字のサロゲート/ZWJ 家族絵文字を割らない（テスト）。

### 1.4 Visual spec

**サイズ（全 shape 共通）** — wrapper `width = height = size`、`flex-shrink:0`、`display:inline-flex`（Avatar.tsx:156-165, 245-249）。

| size | px | initials font-size (= size × 0.4, Avatar.tsx:85, 254-256) | default icon (size × 0.6, Avatar.tsx:545) | status dot tier |
|---|---|---|---|---|
| 16 | 16 | 6.4px | 9.6px | small |
| xsm / 20 | 20 | 8px | 12px | small |
| sm / 24 | 24 | 9.6px | 14.4px | small |
| 32 | 32 | 12.8px | 19.2px | small |
| md / 36 | 36 | 14.4px | 21.6px | small |
| 40 | 40 | 16px | 24px | medium |
| lg / 48 | 48 | 19.2px | 28.8px | medium |
| 60 / 64 / 72 | – | 24 / 25.6 / 28.8px | 36 / 38.4 / 43.2px | medium |
| 96 | 96 | 38.4px | 57.6px | large |
| xl / 128 | 128 | 51.2px | 76.8px | large |
| 144 / 180 | – | 57.6 / 72px | 86.4 / 108px | large |

**Shape**（`--_avatar-radius`, Avatar.tsx:140-150）: circle = `--radius-full`（9999px）、rounded = `--radius-element`（neutral: 0.625rem = **10px**、light/dark 同値）、square = `--radius-none`（0px）。wrapper / content（クリップ）/ interactive のフォーカスリングすべてが同じ半径を使う。

**写真**: `object-fit: cover` で 100%×100%（Avatar.tsx:181-185）。ボーダーは無し（kivact-ui-basic の Lyra avatar にある 1px outline は Astryx には **無い**）。読み込み中の背景は透明（content に背景は無い）。

**フォールバック面**（initials / icon 共通, Avatar.tsx:186-202）:
- background: `--color-neutral`（neutral light `#0000000F` / dark `#FFFFFF1A`、半透明）。単独 avatar では下地（ページ）が透ける。白 surface 上で見た目 ≈ `#F0F0F0`、dark surface `#262626` 上で ≈ `#3C3C3C`。
- 文字色: `--color-text-secondary`（light `#474747` / dark `#9E9E9E`）。forced-colors 時は単独 avatar のみ `currentColor`（loneFallback, Avatar.tsx:210-215）— ネイティブでは無視してよい。
- font: `--font-family-body`（neutral: Figtree → system fallback）、weight `--font-weight-medium` = 500、`text-transform: uppercase`、line-height は継承（中央揃えの flex なので行高は見た目に影響しない。ネイティブでは line-height = font-size × 1.0 で縦中央に置く）。
- DefaultIcon: 24 viewBox の Material 風人型シルエット（塗り `currentColor`）、`size×0.6` 角で中央。パス: `M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z`（Avatar.tsx:550）。kit の IconSet に `soft.avatar.person` として SVG を同梱する。

**Status slot 位置**（Avatar.tsx:257-278）:
- circle: `bottom = inset-inline-end = size × 0.14645`（`(1-1/√2)/2`）、`transform: translate(50%, 50%)`（dot 自身の寸法の 50%）。→ dot の中心が円周上 45°（4 時半）に乗る。RTL は `translate(-50%, 50%)` で左下へミラー。
- rounded / square: `bottom:0; inset-inline-end:0; translate(25%, 25%)`、RTL は `translate(-25%, 25%)`。
- 例 md(36) + small dot(10): 右端・下端から 5.27px 内側を基準点に dot を (+5, +5)px ずらす → dot の外接矩形は avatar の右下角から外へ約 0.27px はみ出す程度。

**グループ内**（Avatar.tsx:280-303、AvatarGroup 参照）: wrapper に `border: 2px solid --color-background-surface`（light `#FFFFFF` / dark `#262626`）、`background-color: --color-background-surface`、`box-sizing: content-box`（外形は size+4px）、`margin-inline-start: -round(size×0.25)px`。フォールバックの半透明 `--color-neutral` はこの surface 上に合成される。

**状態**:
| 状態 | 変化 |
|---|---|
| rest（静的） | 上記。hover / press の見た目変化は **無し**（tooltip が出るだけ） |
| interactive rest | cursor pointer（ネイティブ: `Cursor::Pointer` 相当があれば）。背景・枠の追加は無し（Avatar.tsx:222-239） |
| interactive hover / pressed | 見た目変化 **無し**（overlay なし、scale なし） |
| focus-visible（interactive、または tooltip による tab stop） | outline `--focus-outline-width` 2px solid `--focus-outline-color` = `--color-accent`（neutral light `#1B1B1B` / dark `#F1F1F1`）、`--focus-outline-offset` 3px、半径は `--_avatar-radius` に追従（circle なら円形リング）。グループ内では外形（ring 込み size+4）の外側 3px |
| disabled | Astryx Avatar に disabled prop は無い（`:is(:disabled,[aria-disabled=true])` で cursor default のみ） → ネイティブは `.disabled(..)`（NodeBuilder）で cursor default、押下無効のみ。不透明度変更なし |
| image error | 次段フォールバックへ（src → fallbackSrc → initials → icon）。src/fallbackSrc が変わったら再試行 |

**ASCII（md, circle, status success）**
```
   .-""""-.
  /  JD    \      36×36, bg #0000000F, text #474747 14.4px/500
  \        /●     ● = 10px dot, 中心が円周 45° 上
   '-....-'
```

**レイアウト**: 自身は固定サイズ・縮まない。テキスト折返しなし（イニシャルは最大 2 書記素）。RTL: status 位置のみミラー。

### 1.5 Motion

Avatar 自身のトランジション/アニメーションは **無し**（Avatar.tsx に transition / keyframes 無し）。
- Tooltip: `useTooltip({placement:'above'})`（Avatar.tsx:671-674）。hover 開始から **200ms** 遅延で表示（useTooltip.tsx:302）、pointer-leave 後 **100ms**（HOVER_BRIDGE_DELAY, useTooltip.tsx:50）で非表示、focus では即時表示（`focusTrigger:'auto'`）、offset `--spacing-1` = 4px（useTooltip.tsx:723）。Tooltip の enter/exit アニメーションと見た目（inverted: bg `--color-text-primary` #000000/#FFFFFF、文字 `--color-background-surface` #FFFFFF/#262626、radius `--radius-container` 12px、padding 4px 8px、body 14/20、max-width 300）は Tooltip セクションの仕様に従う（standard `kivact::ui::tooltip` の restyle）。
- reduced-motion: 該当なし。

### 1.6 Behaviour & a11y

- **アクセシブル名**（Avatar.tsx:512-524）: `nameLabel = alt || name`（空白のみは無視）。status ラベルがあれば `t('@astryx.avatar.nameWithStatus', {name, status})` = `"{name}, {status}"`（例 "Jane Doe, Online"）。名前なし + ラベル付き status → status ラベル単独。どちらも無し → decorative（Role なし・名前なし、支援技術に出さない）。
- **status ラベルの報告**: Avatar 直下に渡された要素の `label` を読むだけでなく、ラッパ越し・任意深さの `AvatarStatusDot` が context 経由でラベルを書き込み、Avatar が名前を再合成する（AvatarStatusLabelContext.ts:31-46）。ネイティブ: Avatar が `provide(AvatarStatusCx { label: Signal<Option<String>> , size: f32 }, || status())` で signal を渡し、dot がマウント時に書き込み・アンマウント時に `None` に戻す。Avatar の `.name(move || compose(..))` は reactive にする。
- **Role**: 静的 + 名前あり → `Role::Image` + name。静的 + 名前なし → `Role::Generic` で名前を付けない（kivact に presentation/aria-hidden は無いので「名前なしの非インタラクティブノード」を decorative とする。子テキスト（イニシャル）が読まれないよう、イニシャルの text ノードは Image ノード配下に置き、Image の名前で代表させる）。`on_press` → `Role::Button`、`on_open` → `Role::Link`（名前は同じ合成名）。
- **キーボード**: interactive は Tab stop、Enter / Space（button）・Enter（link）で起動（`kivact::interaction` の press 振る舞い）。静的 avatar は tooltip 有効かつグループ外のときだけ Tab stop（`.focusable(true)`）にして focus で tooltip を出す（WCAG 1.4.13 / 2.1.1）。グループ内ではグループのロービング tab stop に従う（静的 avatar は stop にならない）。
- **Tooltip の description**: 既定（name 表示）の tooltip は aria-describedby を付けない（名前と重複するため）。カスタム文字列 tooltip のみ `.description(text)` 相当で記述として結ぶ（Avatar.tsx:658-686）。
- **pointer/touch**: tooltip は hover / focus で表示、タップは trigger 振る舞い adaptive（Tooltip セクション参照）。
- **dev warning**: interactive なのに `alt`/`name`/consumer の名前がない → debug ビルドで 1 回報告（文言 "Avatar: an interactive avatar (with `href` or `onClick`) needs a meaningful accessible name. Pass `alt` or `name`."、Avatar.tsx:704-709）。status ラベルだけでは名前とみなさない。
- **既定文言 / i18n キー**:
  - `soft.avatar.name_with_status` = `"{name}, {status}"`（Astryx `@astryx.avatar.nameWithStatus`）

### 1.7 Kivact 実装方針

- **NEW kit component**（`kivact::ui` に該当なし）。最も近い kivact-ui-basic: `crates/kivact-ui-basic/src/avatar.rs`（構造: root relative → fallback/picture/outline/badge の絶対配置レイヤ）。Lyra の 1px outline と Primary 色バッジは **持ち込まない**。
- 構成: root `node()`（`Position::Relative`, size×size, `shrink(0)`, radius）→ 内側クリップノード（`.clip(true)`, radius）に「fallback（initials text または person icon）」「fallbackSrc image」「src image」を **下から順に重ねる**。→ 下記の画像失敗の扱い参照。status slot は root 直下の絶対配置ノード（`right`/`bottom` に `size×0.14645 - dot/2` を直接計算して置く。`translate(50%)` の代わりに dot の寸法を context で知っているので数値で配置できる）。
- Tooltip: `kivact::ui::tooltip(text).attach(root)`（standard tooltip を soft の recipe で restyle）。placement Above。
- Press/link: `kivact::interaction` の button 振る舞い（`kivact::ui::button` を使うと見た目が付くので、node + `Role::Button` + Activate ハンドラを kit 内で組む。kivact-ui-basic の他の press 可能なカード（`card.rs` / `item.rs`）のやり方を参照）。
- フォーカスリング: theme の `focus_ring`（2px, offset 3px, accent 色）を radius 追従で描く（`.ring(..)` 相当 or `draw_over`）。
- **難所 / UNRESOLVED**:
  1. **画像読み込み失敗の観測**: kivact の `image(asset)` は失敗時に「何も描かず App の sink に 1 回報告」するだけで、公開 API から失敗状態（`ImageState`）を読めない（`kivact-core/src/control/node.rs:328-338`、`ImageState` は `arvane-asset` 内部）。→ フォールバック: 「initials/icon（最下層）→ fallbackSrc → src（最上層）」を重ねて描く。失敗した画像は何も描かないので下層が見える。差分: Astryx は読み込み中は透明だが、この方式では読み込み中にイニシャルが見える（許容。むしろ自然）。透過 PNG の写真では下層が透けるのが唯一の副作用。**UNRESOLVED**: 将来 kivact が `image(..).on_state(..)` 等を公開したら Astryx 通りの排他表示に切り替える。
  2. **書記素分割**: kit は `kivact` のみ依存（basic の規約）なので `unicode-segmentation` を使えない。→ 簡易実装: 最初の対象 `char`（`char::is_alphanumeric()` で `\p{L}\p{N}` を近似、絵文字は U+1F000–U+1FAFF / U+2600–U+27BF / Regional Indicator U+1F1E6–U+1F1FF で近似）を取り、その後ろに続く ZWJ(U+200D)+次の char、VS16(U+FE0F)、肌色修飾子(U+1F3FB–1F3FF)、結合文字（U+0300–036F）、2 個目の Regional Indicator を連結する。**UNRESOLVED**: kivact が書記素 API を公開するならそれを使う。
  3. `href` / `target` / `rel` / `as` はネイティブ不要 → `on_open` に一本化。
  4. forced-colors は対象外。

### 1.8 受け入れ確認

- [ ] `avatar().name("John Doe")` が 36×36 の円、`JD` を 14.4px / weight 500 / `#474747`（dark `#9E9E9E`）で中央表示し、背景が `#0000000F`（dark `#FFFFFF1A`）。
- [ ] `name("Alice")` → `A`、`name("Northwind Workbench (automation)")` → `NA`、`name("—")` → 人型アイコン。
- [ ] 名前も画像も無いと人型アイコン（size×0.6）で、アクセシビリティツリーに出ない（role/name 無し）。
- [ ] `src` 失敗 → `fallback_src`、両方失敗 → イニシャル（手動: 存在しないパスで確認）。
- [ ] 5 つの名前付きサイズ（20/24/36/48/128）と shape 3 種（rounded は 10px 角丸）が docsite の AllSizes / shape 例と一致（light/dark）。
- [ ] status 付き: role Image、name `"Jane Doe, Online"`。ラッパ越しの dot でも同じ名前。dot を外すと `"Jane Doe"` に戻る。
- [ ] circle の dot が 4 時半位置、rounded/square では右下角から 25% はみ出し。RTL で左下にミラー。
- [ ] `on_press` 付き → Role::Button / name = name、Tab で到達し 2px accent リング（offset 3px、円形）。Enter/Space で発火。`on_open` 付き → Role::Link。
- [ ] 既定 tooltip: hover 200ms 後に上側へ name を表示、description は付かない。`tooltip("Jane Smith, Staff Engineer")` は表示 + description。`tooltip(false)` で出ない。名前なしでは tab stop にならない。
- [ ] 名前なし interactive avatar で debug 警告が 1 回出る。

---

## 2. AvatarStatusDot

ソース: `packages/core/src/Avatar/AvatarStatusDot.tsx`（381 行）, `AvatarStatusDot.doc.mjs`, `AvatarStatusDot.test.tsx`, `Avatar/index.ts`（variant map）, stories `AvatarStatusDot.stories.tsx`, `Avatar.stories.tsx`（StatusAcrossAllSizes / StatusShapesAtSmallSizes / StatusWithIcon）。
docs: https://astryx.atmeta.com/components/AvatarStatusDot（Avatar ページ内の sub-component、overview 非表示）

### 2.1 概要

Avatar の `status` slot に置く、avatar サイズに合わせて段階的に拡縮する在席/状態ドット。各 variant は色と **形**（塗り / リング / マイナス棒）を組にして、色だけに頼らない（WCAG 1.4.1）。`success`=オンライン/承認、`neutral`=離席/オフライン/保留、`error`=取り込み中/DND。ステータスごとに異なるアイコンを使う（同じアイコンを全 variant に使わない）。

### 2.2 Anatomy

```
dot (div; [role=img aria-label=label] when label; radius full; border solid surface)
├─ icon span (aria-hidden; iconSize×iconSize)   … icon が描画可能 かつ tier≠small
└─ glyph svg (aria-hidden; field×field)          … icon を描かないとき、variant に glyph がある場合
     ring  : <circle r=(field-stroke)/2 stroke=currentColor>
     minus : <line round-cap stroke=currentColor>
```
- Avatar 内では dot は自身の stop にならず、ラベルを Avatar の名前に合成させる。単独（Avatar 外）ではラベル付きなら `role="img"` + label（AvatarStatusDot.tsx:351）。

### 2.3 API

| Astryx prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `variant` | `'success'|'neutral'|'error'`（module augmentation で拡張可） | `'success'` | `avatar_status_dot(AvatarStatus)`; `enum AvatarStatus { #[default] Success, Neutral, Error, Custom { fill: Color, ink: Color } }`。`Custom` は glyph 無し（Astryx の augmented variant と同じく、非色の印は icon で呼び出し側が用意） |
| `label` | string | – | `.label(impl IntoText)` |
| `icon` | ReactNode | – | `.icon(impl FnOnce() -> V + 'static)`（通常 `icon("check")`）。small tier では描かず glyph を出す |
| `xstyle`/`className`/`style` | – | – | `.style(..)`（NodeBuilder） |

`pub fn avatar_status_dot(status: AvatarStatus) -> AvatarStatusDot`（`NodeBuilder + IntoView`）。avatar サイズは context（`AvatarSizeContext` 相当、無ければ 36）から読む。

### 2.4 Visual spec

**Tier 表**（AvatarStatusDot.tsx:42-69）:

| avatar size | tier | dot 外径 | border 幅 | inner field (= dot − 2×border) | glyph stroke | icon 枠 |
|---|---|---|---|---|---|---|
| ≤ 36px（16,20,24,32,36） | small | 10px | 1px | 8px | 1px | 描かない |
| 40–72px | medium | 20px | 2px | 16px | 1.5px | 12px |
| ≥ 96px | large | 32px | 4px | 24px | 2px | 18px |

- 共通: `border-radius: --radius-full`、`border-style: solid`、`border-color: --color-background-surface`（light `#FFFFFF` / dark `#262626`）、`box-sizing: border-box`、中身は flex 中央（AvatarStatusDot.tsx:125-134）。
- **success**: plate `--color-success`（neutral light `#198100` / dark `#64AF4C`）、ink `--color-on-success`（light `#FFFFFF` / dark `#111111`）。glyph 無し（ベタ塗りの丸が基準形）。
- **neutral**: plate `--color-background-surface`（`#FFFFFF` / `#262626`）、ink `--color-text-secondary`（`#474747` / `#9E9E9E`）。glyph = ring: 中心 field/2、半径 `(field − stroke)/2`（線の中心線基準 → 外縁が field にぴったり）、塗りなし、stroke = ink。例 small: field 8, r 3.5, stroke 1 → 中空の灰色リング。
- **error**: plate `--color-error`（light `#C9303A` / dark `#FF705D`）、ink `--color-on-error`（light `#FFFFFF` / dark `#111111`）。glyph = minus: y = field/2、x1 = field×0.125 + stroke/2、x2 = field×0.875 − stroke/2、round cap（AvatarStatusDot.tsx:219-273）。例 medium: field 16, stroke 1.5 → x 2.75→13.25。
- **icon**: `display:flex; line-height:0`、icon 枠 12 / 18px、色は ink（`currentColor`）を継承（AvatarStatusDot.tsx:154-171）。icon を描いたら glyph は描かない。icon 引数が `false`/空文字（描画不可）なら glyph を出す。
- **Custom variant**: plate も ink も無し（テーマが与える）、glyph 無し。
- 状態: hover/focus/disabled 等の状態は無い（非インタラクティブ）。
- ASCII（medium、各 variant）:
```
 success   neutral   error
  (●)       (○)      (⊖)     20px, 2px surface 縁
```

### 2.5 Motion

無し（transition / keyframes なし）。

### 2.6 Behaviour & a11y

- Avatar 内: 自身は Role/名前を出さず（Avatar の Image ノードが代表）、`label` を Avatar の合成名に報告。マウントで報告、ラベル変更で再報告、アンマウントで撤回（AvatarStatusDot.tsx:323-339）。
- Avatar 外: `label` があれば `Role::Image` + name、無ければ decorative。
- glyph / icon は常に装飾（支援技術に出さない）。
- 既定文言なし（label は呼び出し側）。

### 2.7 Kivact 実装方針

- **NEW kit component**（Avatar の part）。最も近い basic: `avatar.rs` の `badge`（ただし Lyra は単色 Primary 丸で tier も形も無い）。
- 描画は `node()` + `.style(radius 999, background plate, border(width, Surface))` + glyph は `Style::paint` / `draw_over` で `Draw` に circle stroke / round-cap line を描く（SVG 不要）。stroke 1.5px の sub-pixel は AA 描画で再現。
- size は `context::<AvatarCx>()` から（無ければ 36 → small）。ラベル報告は同 context の `Signal<Option<String>>` に書き込み、`on_cleanup` で `None` に戻す。
- 難所: なし（border は内側に描く = border-box 相当に注意: 外径が tier 値になるようにする）。

### 2.8 受け入れ確認

- [ ] md(36) avatar: 10px dot, 1px 白縁（dark `#262626`）。success は `#198100`（dark `#64AF4C`）の塗り丸、neutral は白地に灰リング、error は赤地 `#C9303A`（dark `#FF705D`）に白（dark `#111111`）の横棒。
- [ ] lg(48) で 20px / 2px 縁 / stroke 1.5、xl(128) で 32px / 4px 縁 / stroke 2。
- [ ] `icon(check)` は medium/large で 12/18px のアイコンを表示し glyph を消す。small tier では glyph のまま。
- [ ] 単独 dot + label → Role::Image, name = label。label 無し → ツリーに出ない。
- [ ] docsite StatusAcrossAllSizes / StatusShapesAtSmallSizes と light/dark 目視一致。

---

## 3. AvatarGroup

ソース: `packages/core/src/AvatarGroup/AvatarGroup.tsx`（200 行）, `AvatarGroupContext.ts`, `AvatarGroup.doc.mjs`, `AvatarGroup.test.tsx`, `hooks/useListFocus.ts`, stories `AvatarGroup.stories.tsx`。
docs: https://astryx.atmeta.com/components/AvatarGroup

### 3.1 概要

複数 avatar を重ねて 1 行に並べる facepile。子は Avatar を直接渡す合成 API（各 avatar が status / onClick / tooltip を個別に持てる）。

- する: 表示数（3–5 が典型）は呼び出し側でスライスし、残数は `AvatarGroupOverflow` で示す。各 Avatar に個別の status / click / tooltip。HoverCard/Tooltip で包んでも重なりは保たれる（その avatar は `tooltip(false)`）。interactive avatar（href/onClick）は単一 Tab stop + 矢印キー移動（APG roving tabindex）。
- しない: グループを入れ子にしない。子 avatar に size を設定しない（グループの size が常に勝ち、黙って無視される）。

### 3.2 Anatomy

```
group (div role=group aria-label=label [aria-describedby=hint])  inline-flex, align center
├─ Avatar × N   （各 wrapper: 2px surface ring, margin-inline-start: -overlap）
├─ AvatarGroupOverflow ?（最後）
└─ VisuallyHidden#hint "Use arrow keys to move between avatars" … interactive 子があるときのみ
```
- `AvatarGroupContext { size, shape, overlap, numericSize }` を提供（AvatarGroupContext.ts:14-19）。

### 3.3 API

| Astryx prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `children` | ReactNode（Avatar… + 任意の Overflow 1 個） | 必須 | `avatar_group(children: impl IntoChildren)`（tuple / Vec of views）。kit が context を provide してから子を build する必要があるので、内部で `provide(GroupCx, move || row(children))` とする（子 builder は IntoView 化が context 内で起こるよう、`Avatar::into_view` 内で `context()` を読む） |
| `size` | AvatarSize | `'md'` | `.size(AvatarSize)` |
| `shape` | AvatarShape | `'circle'` | `.shape(AvatarShape)`（Overflow にも適用） |
| `aria-label` | string | `"Avatars"`（i18n） | `.label(impl IntoText)`（既定は `soft.avatar_group.label`） |
| `aria-describedby` | – | – | 呼び出し側の description は `.description(..)` で、キーボードヒントと併合 |
| `data-testid` | – | – | `.test_id(..)` |
| `onKeyDown` / `onFocus` | – | – | 合成される（kit のロービングの前に呼ぶ）→ ネイティブでは NodeBuilder の `on_key` を先に通す |
| `xstyle`/`className`/`style` | – | – | `.style(..)` |

### 3.4 Visual spec

- root: `display:inline-flex; align-items:center`、折返しなし（nowrap、狭いコンテナでははみ出す: story NarrowContainer）。`padding-inline-start: overlap px`（子が無いときは 0）（AvatarGroup.tsx:78-91）。
- overlap = `round(numericSize × 0.25)`（AvatarGroup.tsx:48, 135）: 16→4, 20→5, 24→6, 32→8, 36→9, 40→10, 48→12, 60→15, 64→16, 72→18, 96→24, 128→32, 144→36, 180→45 px。全子（先頭含む）が `margin-inline-start: -overlap`。
- 各 avatar: 2px `--color-background-surface` ring（light `#FFFFFF` / dark `#262626`）、content-box なので外形 `size + 4`（Avatar.tsx:282-289）。後の子が前の子の上に描かれる（DOM 順 = 描画順、z-index 指定なし）。
- 子の size / shape はグループ値で上書き。
- ASCII（md, 3 avatars + overflow）:
```
 ( A )( B )( C )(+5)     各 40px 外形（36+2×2）、隣と 9px 重なり
   ^-- 白 2px リングで切り抜いたように見える
```
- RTL: `margin-inline-start` / `padding-inline-start` が右側になり、並びが右→左（story RightToLeft）。
- 状態: グループ自体には hover / focus の見た目なし（各子のフォーカスリング）。

### 3.5 Motion

無し。

### 3.6 Behaviour & a11y

- Role `Group`、name = `label`（既定 "Avatars"）。
- **ロービング**（useListFocus, AvatarGroup.tsx:152-156）: 対象は `data-avatar-item` を持つ interactive avatar（link/button）と Overflow ボタンのみ（status slot 内の別ボタンは対象外）。Tab stop は 1 つ（最初は先頭、以後は最後にフォーカスした項目）。`ArrowRight`/`ArrowDown` = 次（末尾で先頭へ wrap）、`ArrowLeft`/`ArrowUp` = 前（wrap）、`Home`/`End` = 先頭/末尾。horizontal で RTL を自動検出し左右を視覚方向に合わせる（useListFocus.ts:12-13, 312-315, 368-371）。
- interactive 子が 1 つ以上あるときだけ、グループに hint を description として結ぶ: `"Use arrow keys to move between avatars"`。完全に静的な facepile は tab stop もヒントも無し。
- グループ内の静的 avatar は tooltip があっても tab stop にならない（Avatar.tsx:916）。
- 既定文言 / i18n キー:
  - `soft.avatar_group.label` = `"Avatars"`
  - `soft.avatar_group.keyboard_hint` = `"Use arrow keys to move between avatars"`

### 3.7 Kivact 実装方針

- **NEW kit component**。最も近い basic: `crates/kivact-ui-basic/src/avatar.rs` の `avatar_group`（Lyra は固定 8px 重なり・先頭のみ margin 0）。Astryx は size 比例 overlap・全子に負 margin + root の start padding。
- 子を個別ノードでラップせず、Avatar 自身が group context を読んで ring / margin を付ける（HoverCard 等でラップされても崩れない Astryx の性質を保つ）。
- ロービング: `kivact::interaction::use_list`（horizontal、wrap、Home/End、Activation なし）を「interactive な子のみ」を項目とするフォーカス管理として使うか、kit 内で roving tab stop（`.focusable(move || index == current)`、`node_ext.rs` の reactive focusable）＋ key handler を書く。interactive 子の有無は子 builder が context の `Signal<u32>`（interactive 数）を増減して知らせる。
- hint は `.description(message("soft.avatar_group.keyboard_hint"))`（VisuallyHidden ノードは不要）。
- RTL: kit の direction context（basic の `basic::direction` 相当）を読み、margin/padding の側と矢印キーを反転。

### 3.8 受け入れ確認

- [ ] Role Group, name "Avatars"（`.label("Team")` で置換）。
- [ ] md: 子の外形 40px、隣接 9px 重なり、白 2px リング（dark `#262626`）。先頭もグループ枠内に収まる。
- [ ] 子に `.size(Xl)` を付けてもグループ size が勝つ。
- [ ] interactive 子 3 つ: Tab で 1 回だけ止まり、→/← で移動・端で wrap、Home/End。グループに hint description が付く。静的のみのグループは Tab で止まらず hint も無い。
- [ ] RTL で並びと矢印方向がミラー。
- [ ] docsite（Default / WithOverflow / AllSizes / Interactive）と light/dark 目視一致。

---

## 4. AvatarGroupOverflow

ソース: `packages/core/src/AvatarGroup/AvatarGroupOverflow.tsx`（240 行）, `AvatarGroupOverflow.doc.mjs`, `AvatarGroupOverflow.test.tsx`, `utils/interactionOverlay.stylex.ts`, `hooks/usePressFeedback.ts`, stories `AvatarGroupOverflow.stories.tsx`。
docs: https://astryx.atmeta.com/components/AvatarGroupOverflow（AvatarGroup ページ内 sub-component）

### 4.1 概要

AvatarGroup の末尾で、個別表示しない人数を `+N` で要約する。メンバー一覧・ポップオーバーを開くなら `onClick` でボタン化。

- する: 実際の残数を `count` に渡す（アクセシブル名が見た目と一致）。短いカスタム文字（`+12`, `99+`）。短い数は円、長い数はピルに伸びる。
- しない: `count` と食い違う文字を children に入れない。長いラベルを入れない。

### 4.2 Anatomy

```
静的 : span[role=img aria-label="{count} more"]  "+N"
押下可: button[type=button aria-label="{count} more" data-avatar-item]  "+N"
```

### 4.3 API

| Astryx prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `count` | number（必須） | – | `avatar_group_overflow(count: impl IntoValue<i64>)`。負数は 0 にクランプ（AvatarGroupOverflow.tsx:179） |
| `onClick` | `() => void` | – | `.on_press(impl Fn() + 'static)` → Role::Button、グループのロービング対象 |
| `children` | ReactNode | `+{count}` | `.content(impl IntoText)`（表示テキストのみ置換、名前は count 由来のまま） |
| `ref`/`xstyle` | – | – | NodeBuilder |

### 4.4 Visual spec

- 寸法（AvatarGroupOverflow.tsx:118-142）: `height = size + 4`、`min-width = size + 4`（border-box、2px border 込み）→ 兄弟 avatar の外形と同寸。`padding-inline: --spacing-2` = 8px（`padding-block: 0`）。短い `+N` は円、長いと横に伸びてピル（radius は `--_avatar-radius` = shape に従う: circle 9999px / rounded 10px / square 0）。
- font: `--font-family-body`（Figtree）、weight 500、font-size = `max(--text-supporting-size(12px), size × 0.35)`: 20→12, 24→12, 36→12.6, 48→16.8, 128→44.8px。user-select none。
- 色（AvatarGroupOverflow.tsx:68-99）:
  - 下地 `--color-background-surface`（light `#FFFFFF` / dark `#262626`）の上に `--color-neutral` tint（light `#0000000F` / dark `#FFFFFF1A`）を重ねる → 合成 light ≈ `#F0F0F0` / dark ≈ `#3C3C3C`（不透明。後ろの avatar が透けない）。
  - 文字 `--color-text-secondary`（`#474747` / `#9E9E9E`）。
  - border 2px solid `--color-background-surface`。
- 配置: グループ内では `margin-inline-start: -overlap`（Avatar と同じ）。グループ外（standalone）は size md(36) / circle、overlap 0（inline-flex なので幅は伸びない）。
- 状態（onClick あり、`interactionOverlayStyles.backgroundImageOnNeutral`, interactionOverlay.stylex.ts）:
  | 状態 | 変化 |
  |---|---|
  | rest | 上記 |
  | hover（hover 可能ポインタのみ） | tint の上に `--color-overlay-hover`（light `#0000000D` / dark `#FFFFFF0D`）を追加 → light ≈ `#E4E4E4` |
  | pressed（マウス :active） | tint の上に `--color-overlay-pressed`（light `#0000001A` / dark `#FFFFFF1A`）→ light ≈ `#D8D8D8` |
  | pressed（touch: data-astryx-press） | pressed overlay を alpha 1 で即時表示、離すと下記 motion でフェード |
  | focus-visible | 2px `--color-accent`（`#1B1B1B` / `#F1F1F1`）outline、offset 3px、shape の半径に追従 |
  | disabled | Astryx に prop 無し（cursor default のみ）。ネイティブは `.disabled(..)` で hover/press 無効 |
- ASCII: `(+5)` 40×40 円 / `(+4912)` 横長ピル。

### 4.5 Motion

- hover / mouse press の overlay 切替: Overflow 自身は transition 指定なし（即時切替）。
- touch press release（interactionOverlay.stylex.ts `pressRelease`）: pressed overlay の alpha を **1 → 0**、duration `200ms × min(tempo, 1.5)`（tempo 既定 1 → 200ms）、easing `cubic-bezier(0, 0, 0.2, 1)`、fill both。reduced-motion: `min(200ms, 200ms × tempo)` = 200ms（短縮のみ、無効化しない）。
- タッチ（coarse pointer）では `:active` の即時塗りを使わず press controller の塗りのみ。

### 4.6 Behaviour & a11y

- 静的: `Role::Image`、name = `"{count} more"`（例 "5 more"、ICU `{count, number}` → ロケール数値書式）。表示テキストは名前に使わない。
- onClick: `Role::Button`、同じ name、Enter / Space / クリックで発火。AvatarGroup のロービング末尾項目。
- 既定文言 / i18n キー:
  - `soft.avatar_group.overflow` = `"{count} more"`（count はロケール数値書式）
  - 表示 `+{count}` はハードコード（`+` 記号はロケール非依存）

### 4.7 Kivact 実装方針

- **NEW kit component**（AvatarGroup の part）。basic: `avatar.rs` の `avatar_group_count`（固定 32px / Muted）。
- surface 背景 + neutral tint は「合成済みの不透明色」を style で塗るか、背景 + `draw_over` で tint を重ねる。hover/press は状態スタイル（recipe の hover/press state style）で overlay を追加（press フェードは `kivact::motion` の 200ms tween）。
- press 振る舞いは `kivact::interaction` の button。グループ context の roving に登録。

### 4.8 受け入れ確認

- [ ] md グループ末尾 `+5`: 40×40 円、12.6px/500、`#474747` on ≈`#F0F0F0`（dark `#9E9E9E` on ≈`#3C3C3C`）、2px surface 縁。
- [ ] `+4912` はピルに伸び、左右 8px 余白。xsm でも文字は 12px 未満にならない。
- [ ] Role Image name "5 more"。`on_press` で Role Button、ロービング末尾、Enter/Space で発火。
- [ ] count −3 → `+0` / "0 more"。
- [ ] hover で 1 段暗く、press で 2 段暗く、touch release は 200ms でフェード。
- [ ] rounded グループでは 10px 角丸、square では角なし（AllShapes story）。

---

## 5. Thumbnail

ソース: `packages/core/src/Thumbnail/Thumbnail.tsx`（552 行）, `Thumbnail.doc.mjs`, `Thumbnail.test.tsx`, 依存 `Skeleton/Skeleton.tsx`, `Spinner/Spinner.tsx`, `Button/Button.tsx`, `hooks/useContainerReveal.ts` + `containerReveal.stylex.ts`, `hooks/useBusyAnnouncement.ts`, `utils/interactionOverlay.stylex.ts`, stories `Thumbnail.stories.tsx`。
docs: https://astryx.atmeta.com/components/Thumbnail

### 5.1 概要

画像添付のコンパクトな正方形プレビュー（64px）。アップロード中はスケルトン、成功で画像、src 無し/失敗でプレースホルダ。任意で右上の削除ボタン、クリックでライトボックス等を開く。チャットコンポーザ、アップロード一覧向け。

- する: ファイル名を `label` に（tooltip と削除ボタン名に使われる）。src 未取得のアップロード初期は `isLoading`（スケルトン）、プレビュー URL 取得後は `isLoading + src`（スピナー overlay）。`onClick` はライトボックスと組にする。
- しない: PDF 等の非画像には使わない（ファイル添付コンポーネントを使う）。src があるのに `alt` を省かない。

### 5.2 Anatomy

```
root  div[role=group aria-label=accessibleName]  64px 幅, column        (label があれば Tooltip で包む)
└─ imageContainer  (64×64, radius element, overflow hidden, bg neutral)  focus-within リング / hover overlay(::after)
   ├─ [onClick あり] button.interactiveButton[aria-label="Open {name}"]  100%×100%
   │     └─ imageContent
   │  [onClick なし] imageContent
   │     imageContent = img(cover) | Skeleton(radius 2) | placeholder(svg 24px)
   ├─ insetBorder (inset 1px border shadow)            … 画像表示時のみ
   ├─ uploadOverlay (scrim + Spinner sm onMedia)       … isLoading && src
   └─ removeSlot (absolute top 4 / end 4) > Button(secondary, sm, icon-only, close xsm)  … onRemove
```
accessibleName = `label && alt ? "{label} — {alt}" : (label ?? alt ?? "Thumbnail")`（Thumbnail.tsx:396-399、区切りは U+2014 em dash の前後スペース）。

### 5.3 API

| Astryx prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `src` | string | – | `thumbnail().src(impl Into<Asset>)` |
| `alt` | string | – | `.alt(impl IntoText)`。無ければ画像は decorative |
| `label` | string | – | `.label(impl IntoText)`（非表示。tooltip・グループ名・削除/開く名に使用） |
| `onRemove` | `(e) => void` | – | `.on_remove(impl Fn() + 'static)` |
| `onClick` | `(e) => void` | – | `.on_press(impl Fn() + 'static)`（「Open」ボタンを生成） |
| `isLoading` | bool | false | `.loading(impl IntoValue<bool>)` |
| `isDisabled` | bool | false | `.disabled(impl IntoValue<bool>)`（NodeBuilder の disabled とは別に kit が見た目と inert を制御。名前は NodeBuilder と衝突しないよう `.inactive(..)` も可 — 実装時に basic の慣例に合わせる） |
| `showRemoveOn` | `'always'|'hover'` | `'hover'` | `.show_remove_on(RemoveReveal)`; `enum RemoveReveal { #[default] Hover, Always }` |
| `xstyle`/`className`/`style`/`data-testid` | – | – | `.style(..)` / `.test_id(..)`、className drop |

`pub fn thumbnail() -> Thumbnail`（`NodeBuilder + IntoView`）。

### 5.4 Visual spec

- root: `width: 64px`、`flex-shrink: 0`、`inline-flex column`、`isolation: isolate`（Thumbnail.tsx:143-150）。
- imageContainer: 64×64（`aspect-ratio: 1`）、`border-radius: --radius-element`（neutral **10px**）、`overflow: hidden`、背景 `--color-neutral`（light `#0000000F` / dark `#FFFFFF1A`）（Thumbnail.tsx:151-158）。
- image: cover, 100%×100%, block。
- insetBorder（画像表示時のみ）: `inset 0 0 0 1px --color-border`（light `#00000014` / dark `#FFFFFF1A`）、radius 継承、ヒットテスト無効（Thumbnail.tsx:165-171）。
- placeholder（src 無し / 失敗 / 非 loading）: 24×24 の画像シルエット SVG（`M21 19V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2M8.5 13.5l2.5 3 3.5-4.5 4.5 6H5l3.5-5.5z`、塗り currentColor）を中央、色 `--color-icon-secondary`（light `#6A6A6A` / dark `#9E9E9E`）（Thumbnail.tsx:172-179, 314-325）。
- skeleton（isLoading && !src）: `Skeleton radius=2` → `--radius-element` 10px、100%×100%、色 `--color-skeleton`（light `#F1F1F1` / dark `#525252`）× opacity 0.25 から脈動（motion 参照）。
- uploadOverlay（isLoading && src）: 全面 `--color-overlay`（light `#00000080` / dark `#000000CC`）、中央に Spinner `sm`（直径 10px + stroke 2px → 14px 箱）shade `onMedia`（arc `--color-on-dark` `#FFFFFF`、track `#FFFFFF` を opacity 77/255 ≈ 0.30）（Thumbnail.tsx:297-307, 514-518; Spinner.tsx:52, 412-428）。
- 削除ボタン（Thumbnail.tsx:266-292, 446-472）:
  - 位置 `top: --spacing-1` 4px、`inset-inline-end: 4px`、z 1。
  - Button secondary / sm / icon-only を上書き: **20×20**（height 20, min-width 20, aspect 1:1, padding 0）、radius `calc(--radius-element − --spacing-1)` = 10 − 4 = **6px**、背景 `--color-overlay`（`#00000080` / `#000000CC`）、アイコン `close`（Lucide `X`）`xsm` = 0.75rem = **12px**、色 `--color-on-dark` `#FFFFFF`（light/dark 共通）。
  - hover: 背景の上に `--color-overlay-hover`（`#0000000D` / `#FFFFFF0D`）、:active: `--color-overlay-pressed`（`#0000001A` / `#FFFFFF1A`）+ `scale(0.98)`（Button.tsx:128-140）。focus-visible: 2px accent outline offset 3px（Button 共通）。
  - coarse pointer: 見た目 20px のまま、ヒット領域を各辺 2px 外へ（24px）（`--_thumbnail-hit-inset: -2px`）。
- interactive（onClick && !disabled && !loading）:
  - cursor pointer、hover（hover 可能ポインタ）で `::after` 全面に `--color-overlay-hover`、mouse :active で `--color-overlay-pressed`、touch は press controller（Overflow と同じ）。
  - focus: imageContainer に **focus-within** リング（内部 Open/Remove ボタンが focus-visible のとき、2px accent、offset 3px、radius 10px）（Thumbnail.tsx:490-497）。
- disabled: root `opacity: 0.5`、`pointer-events: none`（Thumbnail.tsx:293-296）。Open / Remove はマウントされたまま aria-disabled（フォーカス可能・名前あり・押しても何もしない）。
- loading: Open は aria-disabled（inert）、hover overlay なし。
- 状態表:
  | 状態 | 表示 |
  |---|---|
  | src あり読み込み成功 | 画像 + 1px inset border |
  | src 読み込み失敗 | placeholder（src が変われば再試行） |
  | src なし & !loading | placeholder |
  | loading & !src | skeleton |
  | loading & src | 画像 + scrim + spinner（insetBorder は画像表示中なので出る） |
- ASCII:
```
 +--------+
 |      ⊠ |  ⊠ = 20px 削除ボタン（scrim #00000080, 白 X 12px）, top/end 4px
 | image  |  64×64, radius 10
 +--------+
```
- RTL: 削除ボタンは左上へ（inset-inline-end）。

### 5.5 Motion

- **hover/press overlay**（Thumbnail.tsx:217-252）: `::after` の background-color を transition、duration `--duration-fast`（neutral **125ms**）、easing `--ease-standard` = `cubic-bezier(0.24, 1, 0.4, 1)`。touch press 塗り中は transition none（即時）、release は alpha 1→0 / 200ms / `cubic-bezier(0, 0, 0.2, 1)`（AvatarGroupOverflow と同じ）。reduced-motion で overlay transition の無効化指定は無い（125ms のまま）。
- **削除ボタンの hover reveal**（showRemoveOn = hover; containerReveal.stylex.ts:201-226 `revealLayoutPreserved`）: opacity **0 → 1**、duration 125ms、easing ease-standard、delay 0（hoverDelay 未指定）。トリガ: imageContainer の hover（hover 可能ポインタのみ）、focus-within（キーボードで内部にフォーカス）、any-pointer coarse の端末では常時 1。exit（hover 離脱）も同じ 125ms で 1→0。reduced-motion: duration 0s（即時）。レイアウトは保持（ボタンは常に配置済み・透明なだけで、フォーカス/クリック可能）。
- **Button press**: background-image / color / opacity / transform を 125ms ease-standard で transition、:active で `scale(0.98)`。reduced-motion: 0s。
- **Skeleton 脈動**（Skeleton.tsx:51-112）: opacity を 0.25 → 0.925 を 10 段の階段（0.25, 0.325, 0.4, … 0.925、各段 10% 区間保持）で、duration `--duration-medium-max`（neutral **400ms**、tempo 前の値）、`alternate`、infinite、timing linear（段の間は保持のため実質ステップ）、animation-delay **1000ms**（+ index×100ms、Thumbnail は index 0）。reduced-motion: アニメーションなし（opacity 0.25 静止）。
- **Spinner**（Spinner.tsx:228-353）: arc の stroke-dashoffset を 0 → −(直径×π) へ linear、duration `--duration-slow-min`（neutral **525ms**）、infinite、arc は −90° 起点、round cap、arc は円周の 0.375（`ARC_FRACTION`, Spinner.tsx:47）。reduced-motion: 3s に減速（停止しない）。

### 5.6 Behaviour & a11y

- root: `Role::Group`、name = accessibleName。label があれば standard tooltip（label 表示、placement above、200ms 遅延）。
- 画像: alt あり → Role::Image + name = alt。alt なし → decorative。
- Open（onClick あり）: `Role::Button`、name = `"Open {accessibleName}"`。Enter/Space/クリック。disabled / loading 中は aria-disabled 相当（`.disabled(true)` だが **フォーカス可能のまま**、Enter/Space は何もせず、クリックは祖先へも伝播させない）（Thumbnail.tsx:500-509, 533-552）。
- Remove（onRemove あり）: `Role::Button`、name = `"Remove {accessibleName}"`。クリックは Open に伝播しない（stopPropagation）。disabled 中は aria-disabled（フォーカス可・発火しない）。loading/disabled に切り替わってもフォーカスを失わない（spec:AST-085 FR4）。
- Tab 順: Open → Remove。
- loading && src になったとき "Loading" を polite で 1 回アナウンス（useBusyAnnouncement、Thumbnail.tsx:381-385）。Spinner 自体は装飾。
- dev warning: src ありで alt/label/consumer 名が無いと 1 回（文言: "`src` is set without `alt` or `label`. The image is treated as decorative and hidden from assistive technology, and the thumbnail falls back to a generic \"thumbnail\" name. Pass `alt` to describe the image content, or `label` (file name) to name the thumbnail."）。
- 既定文言 / i18n キー:
  - `soft.thumbnail.remove` = `"Remove {name}"`（Astryx `@astryx.thumbnail.remove` "Remove {accessibleName}"）
  - `soft.thumbnail.open` = `"Open {name}"`（`@astryx.thumbnail.open`）
  - `soft.thumbnail.fallback_name` = `"Thumbnail"`（`@astryx.thumbnail.fallbackName`）
  - `soft.spinner.loading` = `"Loading"`（`@astryx.spinner.loading`、Spinner と共有）
  - 合成名の区切り `" — "` はハードコード（Astryx もハードコード）

### 5.7 Kivact 実装方針

- **NEW kit component**。basic で最も近い: `crates/kivact-ui-basic/src/attachment.rs`（`attachment(..)` の media + action 構成、`AttachmentMediaVariant`）、Skeleton / Spinner は `primitives.rs`。
- 構成: root `node()` Role Group（64 幅 column）→ container `node()`（64×64, radius 10, clip, bg neutral）→ [Open: kit 内 press node Role Button] → 画像レイヤ。
- 画像失敗: Avatar と同じ制約（UNRESOLVED: 公開 API で失敗を観測できない）。→ placeholder を最下層に描き、その上に image を重ねる（失敗時は何も描かないので placeholder が見える）。insetBorder は「src があり loading でない」とき常に描く（失敗時にも 1px 枠が出るのが唯一の差分。許容）。
- 削除ボタン: standard `kivact::ui::button(name).icon(icon("close")).hide_label(true)` を使い、kit の Button recipe（secondary sm）に対し **この場所専用の style 上書き**（20×20, radius 6, bg overlay, color on-dark）を `.style(..)` で当てる（呼び出し側 style が recipe に勝つ規約）。hover reveal は container の hover / focus-within を signal 化（kivact の hover/focus 状態を `kivact::interaction` から取得）し、opacity を 125ms tween。タッチ主体（coarse）判定は kivact で取得できるなら常時表示、できなければ **UNRESOLVED**（fallback: `show_remove_on(Hover)` でも最後に触れた入力が touch なら常時表示）。
- hover-media（`@media (hover: hover)`）: ネイティブではマウス hover のみ発生するので自然に満たされる。
- `pointer-events: none`（disabled）→ `.hit_testable(false)` を root に。ただし Open/Remove のキーボードフォーカスは維持（hit_testable はフォーカスに影響しない: node.rs:255-262）。
- Tooltip: `kivact::ui::tooltip(label).attach(root)`。
- Busy アナウンス: kivact に命令的 announce API は無い → 非表示の `live_region(LiveRegion::Polite)` テキストノード（§7 の `visually_hidden`）に loading 開始時だけ "Loading" を書き、終了時に空へ戻す。

### 5.8 受け入れ確認

- [ ] 64×64, 10px 角丸、画像 cover + 1px `#00000014`（dark `#FFFFFF1A`）内枠。
- [ ] src なし → 中央に 24px 画像アイコン `#6A6A6A`（dark `#9E9E9E`）on `#0000000F`。
- [ ] loading & !src → スケルトン（1s 後に脈動開始）。loading & src → 黒半透明 scrim + 白スピナー、"Loading" が 1 回読まれる。
- [ ] `label("screenshot.png")` → Role Group name "screenshot.png"、hover で tooltip。`alt` も付けると "screenshot.png — Photo with metadata"。どちらも無いと "Thumbnail"（+ debug 警告）。
- [ ] onRemove: Role Button "Remove screenshot.png"。Hover モードでは hover / Tab フォーカスで 125ms フェードイン、Always では常時。押しても Open は発火しない。RTL で左上。
- [ ] onClick: Role Button "Open screenshot.png"、hover で overlay `#0000000D`、Tab で container に 2px accent リング。
- [ ] disabled: 全体 opacity 0.5、Open/Remove はフォーカス可能で disabled と報告され、押しても何も起きない。
- [ ] docsite（Default / WithRemove / RemoveOnHover / Loading / Uploading / Placeholder / Disabled）と light/dark 目視一致。

---

## 6. EmptyState

ソース: `packages/core/src/EmptyState/EmptyState.tsx`（212 行）, `EmptyState.doc.mjs`, `EmptyState.test.tsx`, `EmptyState.announce.test.tsx`, `hooks/AnnounceRenderedText.tsx`, stories `EmptyState.stories.tsx`。
docs: https://astryx.atmeta.com/components/EmptyState

### 6.1 概要

データが無い領域のプレースホルダ（空リスト、検索 0 件、初回セットアップ、受信箱クリア）。必ずタイトルと次の一手（ボタン）を置く。

- する: 明確なタイトル + CTA ボタン。文脈を補強するアイコン/イラスト。カードやサイドバーでは compact。
- しない: 案内なしで放置しない。"No data" のような汎用文言にしない（何がなぜ空か）。即時対応が必要なエラーには使わない（Banner を使う）。

### 6.2 Anatomy

```
container (div; column; center; text-align center)
├─ icon wrapper (div aria-hidden)              … icon がある時のみ
├─ textGroup (AnnounceRenderedText div; column; center; max-width 360)
│   ├─ title  (h{headingLevel}, 既定 h3)
│   └─ description (div)                        … ある時のみ
└─ actions (div; row / compact は column; gap 8; margin-top 4)  … ある時のみ
```
- ルートは live region **ではない**。タイトル+説明は「ページ表示後に現れた/変わった」ときだけ 1 回 polite でアナウンス。初回描画時は読まない（spec:AST-086 FR2/FR11）。祖先がすでに live region ならそちらに任せる（FR4）。非表示中は読まず、表示されたとき読む。

### 6.3 API

| Astryx prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `title` | string（必須） | – | `empty_state(title: impl IntoText)` |
| `description` | string | – | `.description(impl IntoText)` |
| `icon` | ReactNode | – | `.icon(impl FnOnce() -> V + 'static)`（装飾） |
| `actions` | ReactNode | – | `.actions(impl FnOnce() -> V + 'static)`（通常 `(button(..), button(..))` の tuple） |
| `headingLevel` | 1–6 | 3 | `.heading_level(u32)` → `Role::Heading` + `.level(n)`。見た目は不変 |
| `isCompact` | bool | false | `.compact(impl IntoValue<bool>)` |
| `xstyle`/`className`/`style`/`ref`/`role` | – | – | NodeBuilder（caller の role も合成可、テスト "composes root props, including a caller role"） |

`pub fn empty_state(title) -> EmptyState`（`NodeBuilder + IntoView`）。

### 6.4 Visual spec

| 部位 | default | compact |
|---|---|---|
| container gap | `--spacing-4` 16px | `--spacing-2` 8px |
| container padding block（上下） | `--spacing-8` 32px | `--spacing-4` 16px |
| container padding inline（左右） | `--spacing-6` 24px | `--spacing-4` 16px |
| title font-size | `--text-large-size` **17px** | `--text-label-size` **14px** |
| title line-height | `--text-large-leading` 1.4118 → **24px** | 同じ 1.4118 → 14px × 1.4118 ≈ **19.76px**（leading は比率なので compact では約 20px） |
| title weight | `--font-weight-semibold` 600 | 600 |
| title color | `--color-text-primary`（light `#000000` / dark `#FFFFFF`） | 同 |
| description font-size | `--text-body-size` 14px | `--text-supporting-size` 12px |
| description line-height | `--text-body-leading` 1.4286 → 20px | 1.4286 × 12 ≈ 17.14px |
| description weight / color | 400 / `--color-text-secondary`（`#474747` / `#9E9E9E`） | 同 |
| textGroup | column, center, max-width **360px**, gap 0（title と description は密着） | 同 |
| actions | row, center, gap 8px, margin-top 4px（→ description との実距離 16+4=20px） | **column**（縦積み）、gap 8px、margin-top 4px（実距離 8+4=12px） |

（EmptyState.tsx:31-85）
- font-family: `inherit`（親から継承。neutral の body は Figtree）。margin 0。
- アイコン: サイズ・色の指定なし（呼び出し側の Icon / 絵文字のまま。story では 48px 絵文字、compact 32px）。
- 背景・枠・影: なし（透明）。
- 状態: 非インタラクティブ（actions のボタンは各自の状態）。
- レイアウト: 親幅いっぱいの flex column（block 要素）、中身は中央寄せ・中央揃えテキスト。textGroup は 360px で折返し。RTL: 中央揃えなので見た目の差なし（actions の並び順のみミラー）。
- coarse pointer の adaptation で type scale が base 16 に変わる（large ≈ 19px, body 16px 等。tokens.md 参照）。
- ASCII:
```
            [icon]
       No results found          17/24 600 #000
 Try adjusting your search or    14/20 400 #474747
          filters.               (max 360)
   [ Go back ] [ Clear filters ]  gap 8
```

### 6.5 Motion

無し。

### 6.6 Behaviour & a11y

- title: `Role::Heading` level = headingLevel（既定 3）、name = title。
- icon: 装飾（支援技術に出さない）。
- アナウンス: 上記の「後から現れた/変わったときだけ 1 回 polite」。テキストはブロックごとに区切って読む（title と description の間に区切り）。
- 既定文言なし（全て呼び出し側）。

### 6.7 Kivact 実装方針

- **NEW kit component**。最も近い basic: `crates/kivact-ui-basic/src/empty.rs`（`empty_state(title).media(..).description(..).content(..)`）。API 名は basic に揃え、`.icon` / `.actions` を Astryx 名で提供（`.media`/`.content` は持たない）。
- 見出しは `text(title)` に `.role(Role::Heading).level(n)`。
- **アナウンス**: kivact に「マウント後の変化だけ読む」仕組みは無い → kit 内で `live_region(Polite)` の非表示テキストノード（§7 `visually_hidden`）を持ち、マウント直後のフレームでは空、以後 title/description の signal が変化したとき（または EmptyState が既にマウントされた画面に後から出現したとき＝親の再描画で新規マウントされたが App 起動から一定時間経過後）に "title. description" を書く。**UNRESOLVED**: 「ページ初回描画か、後から出現か」の判定を kivact でどう取るか（案: App の最初のフレーム描画完了フラグを kit の context に持つ）。祖先 live region 検出も UNRESOLVED（案: kit の live-region context があれば自前で読まない）。

### 6.8 受け入れ確認

- [ ] default: 上下 32 / 左右 24 余白、gap 16、title 17/24 semibold 黒（dark 白）、description 14/20 `#474747`（dark `#9E9E9E`）、最大 360 幅で折返し、actions 横並び gap 8。
- [ ] compact: 上下左右 16、gap 8、title 14px、description 12px、actions 縦積み。
- [ ] title は Role Heading level 3（`.heading_level(1)` で level 1、見た目不変）。
- [ ] icon は支援技術に出ない。空の slot ではラッパノードが作られない。
- [ ] 初回表示では読まれず、検索結果が 0 件になって後から表示されたとき 1 回読まれる。
- [ ] docsite（Default / WithIcon / WithActions / Compact / CompactWithActions）と light/dark 目視一致。

---

## 7. VisuallyHidden

ソース: `packages/core/src/VisuallyHidden/VisuallyHidden.tsx`（142 行）, `VisuallyHidden.doc.mjs`, `VisuallyHidden.test.tsx`, stories `VisuallyHidden.stories.tsx`。
docs: https://astryx.atmeta.com/components/VisuallyHidden

### 7.1 概要

視覚的には隠し、アクセシビリティツリーには残すユーティリティ（sr-only）。用途: アイコンのみのコントロールの名前、`aria-live` アナウンス領域、補足的な SR 向け文脈。

- する: アイコンのみボタンに読み上げ名を与える。live region はブロック要素（`as="div"`）+ `aria-live` で。
- しない: 全員から隠す目的に使わない（条件描画 / `hidden` を使う）。中にインタラクティブ要素を置かない（見えず、ポインタ入力を受けない）。

### 7.2 Anatomy

```
<span|as> (position absolute; 1×1; margin −1; overflow hidden; clip rect(0 0 0 0);
           white-space nowrap; border none; inset-block-start 0; inset-inline-start 0;
           pointer-events none; user-select none)
  children
```
（VisuallyHidden.tsx:74-88）。className / style / xstyle は **受け付けない**（実行時にも除去）。aria-* / role / id / data-* / ハンドラは通す。

### 7.3 API

| Astryx prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `children` | ReactNode | 必須 | `visually_hidden(text: impl IntoText)`（テキストのみ。任意ビューは不要） |
| `as` | ElementType | `'span'` | drop（ネイティブにインライン/ブロックの区別なし） |
| `xstyle` | – | 無視 | 提供しない（style を受けない builder にする: `NodeBuilder` を実装せず、`.live(LiveRegion)` / `.id_ref(NodeRef)` だけ持つ） |
| `aria-live` | – | – | `.live(LiveRegion::Polite | Assertive)` |
| `id`（describedby 先） | – | – | `NodeRef` を返す/受ける（`.node_ref(r)`）→ 他ノードの `.described_by(r)` / `.labelled_by(r)` に使う |

### 7.4 Visual spec

描画なし（1×1 px のクリップ領域、内容は不可視）。レイアウトに影響しない（absolute）。light/dark 差なし。

### 7.5 Motion

無し。

### 7.6 Behaviour & a11y

- 内容はアクセシビリティツリーにテキストとして残る。ポインタ・選択の対象外。フォーカスを受けない。
- live region として使うと内容変化が読み上げられる（Polite / Assertive）。
- 既定文言なし。

### 7.7 Kivact 実装方針

- **ほぼ不要 / kit helper**。ネイティブでは大半の用途が既存機能で足りる:
  - アイコンのみコントロールの名前 → `.name(text)`（kivact::ui::button の `.hide_label(true)` も名前を保持）。
  - 補足説明 → `.description(text)`（`kivact-core/src/view/node.rs:219`）。AvatarGroup のキーボードヒントもこれで代替。
  - live アナウンス → kivact に命令的 announce API が無いので、ここだけ kit helper が必要。
- kit helper 案: `pub(crate) fn visually_hidden(text) -> Node` = `text(..)` を `Position::Absolute`, `size(1,1)`, `.clip(true)`, `.hit_testable(false)`、色 transparent、`.live_region(..)` 任意。kit 内部（Thumbnail の busy、EmptyState のアナウンス、Toast 等）で共有し、公開もしてよい（`soft::visually_hidden`）。
- **UNRESOLVED**: kivact の a11y ツリーが 1×1・透明テキストを読み上げ対象として公開し続けるか（OS によってはオフスクリーン/極小ノードを刈る）。E2E で `screen` の role/name から見えることを確認し、刈られる場合は kivact 側に「描画しないが公開する」ノード属性の追加を提案する。

### 7.8 受け入れ確認

- [ ] `visually_hidden("Delete incident")` は画面に何も描かず、レイアウトを動かさない。
- [ ] テストの screen からテキストとして見つかる。
- [ ] `.live(Polite)` 付きノードの文字列を変えると live region の更新として報告される。
- [ ] ポインタで当たらず、Tab で止まらない。

---

## 8. Text

### 8.1 概要
テーマのタイプスケールから size / weight / line-height / font-family / 既定色を受け取る、意味ベースの本文テキスト。本文・ラベル・補足・コード断片・ディスプレイ数値に使う（`packages/core/src/Text/Text.tsx`、`Text.doc.mjs`）。
- すること: size や weight を手で組まず、意味の `type`（body / label / supporting / large / code）を選ぶ。長い内容は `maxLines` で切り詰める（切り詰められたときは hover で全文の tooltip が出るので、内容は失われない）。数値の列には `hasTabularNumbers` を付ける。
- しないこと: `type` が合っているのに size / weight を上書きする。見出しに Text を使う（Heading を使う）。生の `<p>` / `<span>` を使う。`variant` prop を渡す（存在しない）。

### 8.2 Anatomy
```
Text (root: <span> 既定。as = p | div | label | h1 | h2 | h3)  class .astryx-text[type,size,color]
└─ children (inline の rich content)
(条件付き) Truncation tooltip  — maxLines>0 で、計測して実際に切り詰められているときだけ。Tooltip 部品（anchor = root）
```
- role: as に応じた暗黙の role（span/div は generic、p は paragraph、label は label、h1-h3 は heading）。Text 自体は aria 属性を付けない。

### 8.3 API
| Astryx prop | 型 | 既定 | 意味 | Kivact |
|---|---|---|---|---|
| `type` | `'body'\|'large'\|'label'\|'supporting'\|'code'\|'display-1'\|'display-2'\|'display-3'\|'inherit'`（テーマが独自 type を足せる） | `'body'` | タイプスケール上の役割 | `soft::text(value).kind(TextKind::Body)`。enum `TextKind { Body, Large, Label, Supporting, Code, Display1, Display2, Display3, Inherit }`（`type` は Rust の予約語なので `kind`） |
| `size` | `'4xs'\|'3xs'\|'2xs'\|'xsm'\|'sm'\|'base'\|'lg'\|'xl'\|'2xl'\|'3xl'\|'4xl'` | なし | font-size だけを上書き（line-height / family / 色は type のまま） | `.size(TextSize::Lg)`。enum `TextSize { Xs4, Xs3, Xs2, Xs, Sm, Base, Lg, Xl, Xl2, Xl3, Xl4 }`（`xsm` は `Xs`） |
| `color` | `'primary'\|'secondary'\|'disabled'\|'placeholder'\|'accent'\|'inherit'` | supporting は `secondary`、他は `primary`、`inherit` type は `inherit` | 文字色 | `.color(TextColor::Secondary)`。enum `TextColor { Primary, Secondary, Disabled, Placeholder, Accent, Inherit }` |
| `weight` | `'normal'\|'medium'\|'semibold'\|'bold'` | type の既定 | 太さの上書き | `.weight(TextWeight::Semibold)` |
| `display` | `'inline'\|'block'` | `'inline'` | `maxLines>0` か `hasCapsize` のときは黙って block | ネイティブには inline/block の区別がない。Text はいつも独立の text node。inline の混在は `kivact::view::span` の rich text で作る（下記）。prop は落とす |
| `as` | `'span'\|'p'\|'div'\|'label'\|'h1'\|'h2'\|'h3'` | `'span'` | HTML 要素 | `as=h1..h3` は `.heading_level(1..=3)`（`Role::Heading` + `level`）。`p`/`label`/`div`/`span` は意味の差がないので落とす（label の関連付けは `kivact::ui::field` の役目） |
| `maxLines` | number | 0 | 切り詰める行数 | `.max_lines(n)`。1 は `style().single_line()`（`text_wrap(false)` + `TextOverflow::Ellipsis`）、2 以上は `style().line_clamp(n)` |
| `hasTruncateTooltip` | `boolean \| 'above'\|'below'\|'start'\|'end'` | true | 切り詰め時の tooltip とその位置 | `.truncate_tooltip(TruncateTooltip::Off \| Above \| Below \| Start \| End)`。既定 Above |
| `wordBreak` | `'break-word'\|'break-all'` | maxLines=1 で `break-all`、それ以外 `break-word` | 切り詰め時の改行規則 | `.word_break(WordBreak::BreakAll)`。Kivact の折り返しが break-all を持たなければ UNRESOLVED（下記 8.7） |
| `textWrap` | `'wrap'\|'nowrap'\|'balance'\|'pretty'` | なし | 折り返し | `Wrap`/`NoWrap` は `text_wrap(bool)`。`balance`/`pretty` はネイティブに無いので `Wrap` 扱い（見た目の差は軽微） |
| `justify` | `'start'\|'center'\|'end'` | `'start'` | 論理方向の揃え | `.justify(TextJustify::Center)` → `text_align`。RTL では start/end を入れ替える（`current_direction()`） |
| `hasCapsize` | bool | false | `text-box-trim: trim-both; text-box-edge: cap alphabetic` | `.capsize(true)`。上は cap-height、下は baseline で行ボックスを切る。kit で `measure_text` の ascent/cap-height から上下の余白を負に詰める（8.7） |
| `hasStrikethrough` | bool | false | 取り消し線 | `.strikethrough(true)` → `text_decoration(TextDecorationLine::LINE_THROUGH)` |
| `hasTabularNumbers` | bool | false | `font-variant-numeric: tabular-nums` | `.tabular_numbers(true)` → `style().tabular_nums()` |
| `children` | ReactNode | 必須 | 内容 | 第 1 引数 `impl IntoText`。rich 内容は `soft::rich_text(spans)` を別に用意 |
| `id`, `className`, `style`, `xstyle`, `ref` | | | | 落とす。呼び出し側は `NodeBuilder` の `.style(..)` で上書き（kit の既定より後で勝つ） |

提案シグネチャ:
```rust
pub fn text(value: impl IntoText) -> Text;           // NodeBuilder + IntoView
impl Text {
    pub fn kind(self, k: TextKind) -> Self;
    pub fn size(self, s: TextSize) -> Self;
    pub fn color(self, c: TextColor) -> Self;
    pub fn weight(self, w: TextWeight) -> Self;
    pub fn max_lines(self, n: u32) -> Self;
    pub fn truncate_tooltip(self, t: TruncateTooltip) -> Self;
    pub fn word_break(self, w: WordBreak) -> Self;
    pub fn wrap(self, w: TextWrap) -> Self;
    pub fn justify(self, j: TextJustify) -> Self;
    pub fn capsize(self, on: bool) -> Self;
    pub fn strikethrough(self, on: impl IntoValue<bool>) -> Self;
    pub fn tabular_numbers(self, on: bool) -> Self;
    pub fn heading_level(self, level: u32) -> Self;   // as="h1".."h3"
}
```
`TextKind` / `TextSize` / `TextColor` / `TextWeight` は Heading・Timer・Timestamp・Code・Markdown でも共用する。

### 8.4 Visual spec
スタイル定義: `packages/core/src/Text/text.stylex.ts`（color 26、weight 51、既定 weight 70、type 別 size 108、size 上書き 152、display 242、切り詰め 257、word-break 278、text-wrap 292、capsize 311、decoration 323、tabular 333、justify 343、tooltip 内容 359 行）。neutral では `typography.scale {base:14, ratio:1.2}` が `expandTypeScale`（`theme/expandTypeScale.ts:290-335`）で token を作り直し、`generateTypeScaleComponents`（同 :360-420）が `.astryx-text[type:*]` に family / size / line-height を当てる。

**フォント（neutral）**: `--font-family-body` = `Figtree, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif`。`--font-family-heading` も同じ Figtree（display-1/2/3 はこちら）。`--font-family-code` = `ui-monospace, "SF Mono", Monaco, Consolas, "Liberation Mono", "Courier New", monospace`。Figtree はアプリが登録する（kivact-ui-basic の README と同じ扱い）。

**type ごとの寸法（neutral、desktop: base 14 / ratio 1.2）** — line-height は計算値（unitless 比 → px）:
| type | size token → px | line-height（比 → px） | weight | family | 既定色 |
|---|---|---|---|---|---|
| body | `--text-body-size`=`--font-size-base` 14px | 1.4286 → 20px | `--font-weight-normal` 400 | body | primary |
| large | `--font-size-lg` 17px | 1.4118 → 24px | semibold 600 | body | primary |
| label | 14px | 1.4286 → 20px | medium 500 | body | primary |
| code | 14px | 1.4286 → 20px | 400 | code | primary |
| supporting | `--font-size-sm` 12px | 1.6667 → 20px | 400 | body | secondary |
| display-1 | `--font-size-5xl` 42px | 1.2381 → 52px | 400 | heading | primary |
| display-2 | `--font-size-4xl` 35px | 1.2571 → 44px | 400 | heading | primary |
| display-3 | `--font-size-3xl` 29px | 1.3793 → 40px（※） | 400 | heading | primary |
| inherit | 親の値 | 親の値 | 親の値 | 親 | inherit |

※ display-3: 静的既定 token（`tokens.stylex.ts` の `--text-display-3-leading: 1.2414` = 36px）と、neutral の scale から計算した値（`computeLeading(29)`: 29<32 なので目標 1.4 → 40.6 → 4px 格子で 40px → 1.3793）が食い違う。neutral は scale を渡しているので 40px が正しいはず。docsite の Text ページで display-3 の行高を実測して確認（UNRESOLVED: 実測で 36px なら 1.2414 を採用）。

**size 上書き**（`sizeStyles`、font-size のみ）: 4xs 6px、3xs 7px、2xs 8px、xsm 10px、sm 12px、base 14px、lg 17px、xl 20px、2xl 24px、3xl 29px、4xl 35px（`--font-size-5xl` 42px は size 上書きに無い）。

**タッチ向け adaptation（neutral `adaptations.rules`）**: `pointer: coarse` かつ viewport 幅 < `lg`（1024px、`theme/themeAdaptations.ts:54`）のとき scale が `{base:16, ratio:1.1736}` に変わる。解決値:
| type / level | size | line-height |
|---|---|---|
| body / label / code | 16px | 24px（1.5） |
| large | 19px | 28px（1.4737） |
| supporting | 14px | 20px（1.4286） |
| display-1 / 2 / 3 | 42 / 36 / 30px | 52 / 44 / 44px |
| size 上書き 4xs..4xl | 7, 8, 10, 12, 14, 16, 19, 22, 26, 30, 36px | — |
デスクトップの kit では既定で使わない。`soft::theme(mode).touch(true)` のような明示オプションで切り替える案（判断は parent に委ねる）。

**色（neutral 解決値 light / dark）**:
- primary `--color-text-primary`: `#000000` / `#ffffff`
- secondary・placeholder `--color-text-secondary`: `#474747` / `#9e9e9e`
- disabled `--color-text-disabled`: `#919191` / `#525252`
- accent `--color-text-accent`: `#1b1b1b` / `#f1f1f1`（neutral の accent は無彩色）
- inherit: 親の色

**weight 上書き**: normal 400、medium 500、semibold 600、bold 700。

**レイアウト規則**:
- 切り詰めないテキストは `overflow-wrap: break-word`（`displayStyles`、:242）。URL など行より長い語は途中で折る。ただし min-content は普通の語のまま（表の列幅に影響させない）。
- `maxLines=1`: `overflow:hidden; text-overflow:ellipsis; white-space:nowrap; display:block`（:257）。既定の word-break は `break-all`。
- `maxLines>1`: `-webkit-box` + `line-clamp:n`。既定の word-break は `break-word`（`word-break:normal; overflow-wrap:break-word`）。
- flex 行の中で切り詰めるときは親が `min-width:0` を持つ必要がある。Kivact の `single_line()` は `min_width(0)` まで含むので、そのまま使う。
- `justify` は論理値。RTL では start = 右。
- tooltip 内容: `max-width: 300px; word-break: break-word`（:359）。

**状態**: Text 自体は対話しない。hover / focus の状態はない。

### 8.5 Motion
Text 自体のアニメーションはない。切り詰め tooltip の出入りは Tooltip 部品（`kivact::ui::tooltip` の restyle）の motion に従う。値は Tooltip の担当グループの章を参照（ここでは定義しない）。

### 8.6 Behaviour & a11y
- `maxLines>0` で切り詰め tooltip を有効にすると、`useTruncation`（`Text/useTruncation.ts`）が ResizeObserver と MutationObserver で計測する。1 行は `scrollWidth > offsetWidth`、複数行は Range の高さ > `offsetHeight`。初回計測は commit 後の microtask。切り詰められているときだけ Tooltip を（lazy で）付け、内容は `textContent` の全文。placement 既定 `above`。
- tooltip は Tooltip 部品の挙動（hover と focus で開く）。Text 自体はフォーカスできないので、実質 pointer の hover だけで開く。
- 支援技術に渡る名前はいつも全文（CSS で切っているだけ）。Kivact でも `TextOverflow::Ellipsis` / `line_clamp` の名前は全文になる（`sheet.rs` のドキュメント）。
- 固有の文言はない（i18n キー不要）。

### 8.7 Kivact 実装方針
- NEW kit component `soft::text`。`kivact::control::text(value)` の上に、tokens から引いた `Style`（font_size, line_height, weight, font_family, color）を載せる。最も近い analog は `crates/kivact-ui-basic/src/typography.rs`（`styled()` で tokens を引く形）。
- type / size / color / weight の解決は 1 か所の関数 `type_style(&Tokens, TextKind) -> Style` にまとめ、Heading・Timer・Timestamp・Markdown・Code で共用する。
- 切り詰め tooltip: Kivact の公開 API には「テキストが実際に切り詰められたか」を問い合わせる手段がない（`truncated` 系の API は無い）。案 A: `kivact::primitive::primitive().measure(..)` で `Constraints::measure_text` / `measure_paragraph` を呼び、幅に対する自然幅・行数を比べて signal に書く（measure の中では signal に書けないので、layout 後に読む仕組みが必要。UNRESOLVED）。案 B（フォールバック）: `max_lines` を指定したら、計測なしで常に `kivact::ui::tooltip(full_text).attach(node)` を付ける。見た目への影響は「切り詰められていないときも hover で tooltip が出る」ことだけ。まず B で実装し、core にクエリを足すかどうかは parent が判断。
- `wordBreak: break-all` は Kivact の折り返しエンジンの対応次第（UNRESOLVED）。無ければ `break-word` 相当で代用する。1 行の ellipsis では違いは見えない。
- `hasCapsize`: ネイティブには text-box-trim が無い。kit で font metrics（ascent − cap-height、descent）を測って padding を負にするか、`Measurement::baseline` で調整する。難しければ見送り、ドキュメントに書く（使っているのは docsite の一部の例だけ）。
- `textWrap: balance / pretty` は Wrap にまとめる。
- inline の混在（`<Text>Processing for <Timer/></Text>`）は、Kivact では `row((text(..), timer(..)))` で baseline 揃え（`Align::Baseline`）にするか、`kivact::view::span` の rich text にする。kit の README に書く。

### 8.8 受け入れ確認
- [ ] `text("Body")` は 14px / 20px / 400 / Figtree / `#000000`（dark `#ffffff`）
- [ ] supporting は 12px / 20px / `#474747`（dark `#9e9e9e`）
- [ ] large は 17px / 24px / 600、label は 14px / 20px / 500、code は 14px monospace
- [ ] display-1/2/3 は 42/52、35/44、29/40(または 36)px、weight 400
- [ ] `.max_lines(1)` は 1 行で末尾に「…」。name（アクセシブル名）は全文
- [ ] `.max_lines(2)` は 2 行で切れる
- [ ] 切り詰められたテキストに hover すると全文の tooltip が上に出る（最大幅 300px）
- [ ] `.strikethrough(true)` で取り消し線、`.tabular_numbers(true)` で数字幅が揃う
- [ ] docsite https://astryx.atmeta.com/components/Text の各 type 見本と light / dark で並べて比べ、サイズ・行間・色が一致する

---

## 9. Heading

### 9.1 概要
h1〜h6 の意味を出す見出し（`packages/core/src/Heading/Heading.tsx`、doc は `Text/Heading.doc.mjs`）。`level` が要素と見た目の両方を決める。`type`（display-1/2/3）を付けると見た目だけ display スケールになり、level は意味のために残る。
- すること: 文書構造の順に h1→h2→h3 と使う。見た目と構造が違うときは `accessibilityLevel` で `aria-level` を合わせる。
- しないこと: レベルを飛ばす。Text で見出しを作る。

### 9.2 Anatomy
```
Heading (root: <h{level}>  class .astryx-heading[level,color,type,weight])
└─ children
(条件付き) Truncation tooltip（Text と同じ）
```
- role: heading、level = `accessibilityLevel ?? level`（違うときだけ `aria-level` を付ける）。

### 9.3 API
| Astryx prop | 型 | 既定 | 意味 | Kivact |
|---|---|---|---|---|
| `level` | 1-6 | 必須 | 要素と見た目 | `soft::heading(level: u32, value: impl IntoText)` → `Heading`（`Role::Heading` + `.level(level)` + `.name(value)`） |
| `type` | `'display-1'\|'display-2'\|'display-3'`（テーマ独自 type 可） | なし | 見た目を display スケールに | `.display(HeadingDisplay::D1)` |
| `weight` | normal/medium/semibold/bold | level の既定 | 太さ上書き | `.weight(TextWeight::..)` |
| `accessibilityLevel` | 1-6 | なし | 支援技術向け level | `.accessibility_level(n)` → semantics の level だけ差し替え |
| `color` | Text と同じ | `'primary'` | 文字色 | `.color(TextColor::..)` |
| `display` | inline/block | `'block'` | | 落とす |
| `maxLines`, `hasTruncateTooltip`, `wordBreak`, `textWrap`, `justify`, `hasCapsize`, `hasStrikethrough` | Text と同じ | Text と同じ | | Text と同じメソッド |
| `id`, `className`, `style`, `xstyle`, `ref` | | | | 落とす / NodeBuilder |
（Heading には `hasTabularNumbers` と `size` は無い。）

### 9.4 Visual spec
スタイル: `Text/text.stylex.ts:195`（`sizeByLevelStyles`）と、neutral の `typography.heading.weights {3:'bold', 4:'bold'}`（`themes/neutral/src/neutralTheme.ts` の typography）。family はいつも `--font-family-heading`（Figtree）。

**level ごと（neutral、desktop）**:
| level | size token → px | line-height | weight（neutral） |
|---|---|---|---|
| 1 | `--text-heading-1-size` = `--font-size-2xl` 24px | 1.3333 → 32px | `--font-weight-semibold` 600 |
| 2 | `--font-size-xl` 20px | 1.4 → 28px | 600 |
| 3 | `--font-size-lg` 17px | 1.4118 → 24px | **700**（neutral の上書き。既定は 600） |
| 4 | `--font-size-base` 14px | 1.4286 → 20px | **700**（neutral の上書き） |
| 5 | `--font-size-sm` 12px | 1.6667 → 20px | 600 |
| 6 | `--font-size-xs` 10px | 1.6 → 16px | 600 |

**display type**（level の size を置き換える。weight は `defaultWeightByTypeStyles` の 400。テーマ側の `type:display-N` 規則は weight を書かないので 400 のまま。`expandTypeScale.ts:372-386`）: display-1 42/52px、display-2 35/44px、display-3 29/40px（Text の※と同じ確認事項）。

**タッチ向け adaptation（coarse かつ幅 < 1024px）**: h1 26/36px、h2 22/32px、h3 19/28px、h4 16/24px、h5 14/20px、h6 12/20px。weight は同じ。

**色**: 既定 primary `#000000` / `#ffffff`。他の色は Text と同じ値。

**レイアウト**: いつも block（全幅の行）。margin 0（reset.css が見出しの margin を 0 にする前提。前後の間隔は呼び出し側の stack gap が決める）。切り詰めの規則は Text と同じ。

### 9.5 Motion
なし（tooltip は Tooltip 部品に従う）。

### 9.6 Behaviour & a11y
- `Role::Heading`、`level`。`accessibility_level` があればそちらを semantics に出す。
- name はテキスト全文。
- 文言なし。

### 9.7 Kivact 実装方針
- NEW kit component `soft::heading(level, value)`。analog は `crates/kivact-ui-basic/src/typography.rs` の `heading()`（`role(Role::Heading).level(level).name(value)`）。Text と同じ `type_style` 関数に level の行を足す。
- `h1()`〜`h6()` の短縮関数も用意してよい（basic と揃う）。

### 9.8 受け入れ確認
- [ ] `heading(1, "Title")` は role=heading、level=1、24px/32px/600
- [ ] level 3 と 4 は 700（bold）、他は 600
- [ ] `.display(D1)` は 42px/52px/400 で、semantics は元の level のまま
- [ ] `.accessibility_level(2)` を付けた level 4 は支援技術に level 2 として出る
- [ ] docsite https://astryx.atmeta.com/components/Heading と light / dark で一致する

---

## 10. Blockquote

### 10.1 概要
引用ブロック（`packages/core/src/Blockquote/Blockquote.tsx`）。inline-start 側の縦線と secondary の文字色。引用文、推薦文、外部からの抜粋に使う。出典が分かるときは `cite` に渡す（children の中に書かない）。注意書きや情報ボックスには使わない（Banner を使う）。

### 10.2 Anatomy
```
Blockquote (<blockquote>, class .astryx-blockquote)
├─ children（多くは Text / Markdown の段落）
└─ cite?（<cite>、block。footer で包まない: blockquote 内の footer は contentinfo landmark になるため）
```
- role: blockquote（ARIA の `blockquote` role）。cite は generic（HTML の cite に ARIA role は無い）。

### 10.3 API
| Astryx prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `children` | ReactNode | 必須 | `soft::blockquote(content: impl FnOnce() -> V)` の slot。文字列だけの短縮形 `soft::blockquote_text(value)` も用意 |
| `cite` | ReactNode | なし | `.cite(impl IntoText)` |
| `ref`, `className`, `style`, `xstyle` | | | 落とす / NodeBuilder |

### 10.4 Visual spec
`Blockquote.tsx:35-58`:
- root: `border-inline-start: var(--spacing-0-5) solid var(--color-border-emphasized)` → 2px、色 `#d4d4d4` / dark `#525252`。他の 3 辺に border なし。radius なし。
- `padding-inline-start: var(--spacing-4)` = 16px。他の padding は 0。margin はすべて 0。
- color `--color-text-secondary` `#474747` / `#9e9e9e`。font-size / line-height / family は指定なし（親から継承。docsite では body 14/20 Figtree）。
- `overflow-wrap: break-word`。
- cite: `display:block; margin-block-start: var(--spacing-2)` = 8px。`font-size: --text-supporting-size` 12px、`line-height: --text-supporting-leading` 1.6667 → 20px、`font-style: normal`（cite 既定の italic を消す）。色は root から secondary を継承。
- RTL: 線は inline-start なので右側に移る。
```
┃ The best way to predict the future
┃ is to invent it.
┃
┃ Alan Kay            ← cite 12/20, 上に 8px
 ^2px #d4d4d4, 線と本文の間 16px
```
状態なし。

### 10.5 Motion
なし。

### 10.6 Behaviour & a11y
- 対話なし。Kivact の role には blockquote が無い可能性が高い。`Role::Group` か role なし（UNRESOLVED: `kivact::view::Role` に Blockquote があればそれを使う）。
- 文言なし。

### 10.7 Kivact 実装方針
- NEW kit component。`column((content, cite))` に `border_left(2, BorderEmphasized)`（RTL では `current_direction()` を見て `border_right`）と `padding_left(16)`。analog は `crates/kivact-ui-basic/src/typography.rs` の `blockquote`（basic は italic で border-left 2px）。soft では italic にしない点に注意。
- 中のテキストの色は secondary を継承させたい。Kivact に色の継承が無ければ、slot 内の `soft::text` を `TextColor::Inherit` にしたとき親の文脈色を読む仕組みが必要（kit の provide した「現在の文字色」context を Text が読む）。最小実装: `blockquote_text` は Text を secondary で描く。

### 10.8 受け入れ確認
- [ ] 左端に 2px の線（light `#d4d4d4`、dark `#525252`）。線と文字の間 16px
- [ ] 本文は secondary 色、cite は 12px/20px、本文の 8px 下、italic でない
- [ ] RTL では線が右側
- [ ] docsite https://astryx.atmeta.com/components/Blockquote と light / dark で一致

---

## 11. Citation

### 11.1 概要
外部ソースへの inline 参照チップ（`packages/core/src/Citation/Citation.tsx`）。AI の回答や記事の中で出典を示す。`label` 型（出典タイトルのピル、任意のアイコン）と `number` 型（上付きの丸い番号、脚注風）。同じ段落で 2 つの型を混ぜない。

### 11.2 Anatomy
```
Citation (url が安全なら <a href target=_blank rel="noopener noreferrer">、無ければ <span>)
  role = url あり: doc-noteref / 無し: group
  aria-label = "Citation {number}: {title}"、title 属性 = title
 ├─ (label 型) iconWrap <span aria-hidden>（16px の丸） ─ icon(sm) または <img alt="">（12px）
 ├─ (label 型) labelText <span>（1 行 ellipsis）= title
 └─ (number 型) 数字 {number}
```
`title` は `source.title ?? String(number)`。`url` は `isSafeUrl`（`utils/safeUrl`）を通ったときだけ使う。

### 11.3 API
| Astryx prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `source` | `{title?, url?, src?, icon?}` | 必須 | `soft::citation(number: u32, source: CitationSource)`。`CitationSource { title: Option<String>, url: Option<String>, image: Option<Asset>, icon: Option<Icon> }` |
| `number` | number | 必須 | 第 1 引数 |
| `variant` | `'label'\|'number'` | `'label'` | `.variant(CitationVariant::Label \| Number)` |
| (a の href/target) | | | ネイティブではリンクを開けないので `.on_press(impl Fn())` で通知。url があってハンドラが無ければ `kivact::platform::open_path` は使えない（URL 用ではない）ので、kit は url を `on_press` の引数で渡すだけにする: `.on_open(impl Fn(&str))` |
| `ref`, `xstyle`, `className`, `style`, `data-testid` | | | 落とす / NodeBuilder（`.test_id`） |

### 11.4 Visual spec
`Citation.tsx:75-185`。
**label 型**（:76-113）:
- inline-flex、align center、gap `--spacing-1` 4px、`vertical-align: baseline`。
- height `--spacing-5` = 20px。padding-inline `--spacing-2` 8px（左右）。アイコンがあると padding-inline-start は `--spacing-0-5` 2px（:110）。上下 padding 0。
- margin-inline-start `--spacing-0-5` 2px（前の語との間）。
- border 1px（`--border-width`）solid `--color-border` `#00000014` / dark `#ffffff1A`。radius `--radius-element` = 0.625rem = 10px（高さ 20px なので実質ピル形）。
- 背景なし（透明）。文字: `--text-supporting-size` 12px、`--text-supporting-weight` 400、line-height 1.6667（20px）、色 `--color-text-secondary` `#474747` / `#9e9e9e`。下線なし。
- max-width 15em（12px × 15 = 180px）、overflow hidden。labelText は `overflow:hidden; text-overflow:ellipsis; white-space:nowrap; min-width:0`。
- iconWrap（:166-180）: 16×16px（`--spacing-4`）、radius full、背景 `--color-background-surface` `#ffffff` / `#262626`、1px solid `--color-border`、overflow hidden、shrink しない。中の img は 12×12px（`--spacing-3`）。ReactNode のアイコンは Icon size `sm`（16px）で描かれ、16px の丸の中に収まる（img と違い 12px にはならない点に注意）。
- hover（url があり、`@media (hover:hover)` のときだけ。:114-128）: 背景 `--color-overlay-hover` `#0000000D` / `#ffffff0D`、文字色 `--color-text-primary` `#000000` / `#ffffff`。cursor pointer。
- url が無いときは hover の変化なし、cursor default。

**number 型**（:130-163）:
- inline-flex、中央揃え、`vertical-align: super`（上付き）。
- min-width 20px、height 20px、padding-inline `--spacing-1` 4px。radius full（円 / ピル）。
- 文字 12px、weight `--font-weight-semibold` 600、line-height 20px、色 secondary。
- 背景 `--color-accent-muted` `#f1f1f1` / dark `#262626`。
- hover（url あり、hover 可能なポインタ）: 背景 `--color-overlay-hover` `#0000000D` / `#ffffff0D` に置き換わる（muted が消えてより薄くなる）。

**focus-visible**: Citation 自身は focus ring の style を持たず、`reset.css` にも `a:focus-visible` の規則は無い。つまり docsite ではブラウザ既定の outline が出る。kit ではほかの部品と同じ soft 共通の focus ring（`--focus-outline-width` 2px、solid、`--focus-outline-color` = `--color-accent` `#1b1b1b` / `#f1f1f1`、`--focus-outline-offset` 3px）を付ける。radius は label 型 10px、number 型は full。

```
label:  [(◉) OpenAI research pap…]   高さ 20, radius 10, border 1px
number:  text text¹²                  上付き 20×20 の丸, 背景 accent-muted
```

### 11.5 Motion
- label: `transition-property: background-color, border-color, color`、duration `--duration-fast-max`（neutral 165ms）、easing `--ease-standard` `cubic-bezier(0.24, 1, 0.4, 1)`（:93-95）。trigger は hover の出入り。enter / exit とも同じ。
- number: `background-color` だけ、同じ 165ms / ease-standard（:149-151）。
- reduced motion: 個別の切り替えはない（色の transition だけなのでそのまま）。

### 11.6 Behaviour & a11y
- url あり: リンク。role `doc-noteref`、Enter（と click）で開く。target=_blank。Kivact: `Role::Link`（doc-noteref は無いので Link）、Enter / Space / Activate で `on_open(url)`。
- url なし: role group、フォーカスできない。
- name: `soft.citation.label` = "Citation {number}: {title}"（`@astryx.citation.label`）。tooltip 相当の `title` 属性はネイティブでは Tooltip にする（任意。既定はなし）。
- icon / img は装飾（aria-hidden / alt=""）。

### 11.7 Kivact 実装方針
- NEW kit component。`row` + `kivact::interaction::use_button` 相当の押下（`on_press`）。analog: `crates/kivact-ui-basic/src/badge.rs`（小さなピル）。
- `vertical-align: super` / baseline: 段落の中に置く場合、Kivact に inline 配置が無いので、`row` の中で `Align::Baseline` と、number 型は `translate_y(-ascent の約 0.4)` 程度上げる（CSS の super は親フォントの約 1/3 em 上。14px 本文なら約 −4〜−5px）。Markdown の中での位置合わせは Markdown 章を参照。
- transition: Kivact の `TransitionProp::Paint` が補間するのは opacity・背景・border 色・transform・影・filter で、文字色は含まれない（`kivact-style/src/sheet.rs:150-158`）。背景は `Transition::tween(165, Easing::CubicBezier(0.24, 1.0, 0.4, 1.0))` で補間し、文字色は即時に切り替える（差は 165ms だけなので見た目への影響は小さい）。
- hover は `style().hover(..)`。`@media (hover:hover)` の条件は、Kivact では pointer hover しか来ないのでそのまま。
- url の安全性チェック（`isSafeUrl`: http(s) / mailto 等だけ許す）は kit に移植するか、ネイティブでは不要（開く判断はアプリ）。不要と判断し、url はそのまま `on_open` に渡す。

### 11.8 受け入れ確認
- [ ] url ありの citation は role=link、name="Citation 1: OpenAI research paper"
- [ ] label 型: 高さ 20px、12px secondary 文字、1px の薄い枠、180px で「…」
- [ ] アイコン付きは左 padding が 2px、16px の丸い白背景のアイコン枠
- [ ] hover で背景が薄く暗くなり、文字が primary に変わる（165ms）
- [ ] number 型: 20px の丸、背景 `#f1f1f1`（dark `#262626`）、600、上付き
- [ ] docsite https://astryx.atmeta.com/components/Citation と light / dark で一致

---

## 12. Code（inline code）

### 12.1 概要
本文の中の短いコード参照（関数名、変数、パス、フラグ）。等幅フォントと muted 背景（`packages/core/src/Code/Code.tsx`）。複数行や独立したスニペットには CodeBlock を使う。コピーやナビゲーションの操作部品として使わない。

### 12.2 Anatomy
```
Code (<code>, class .astryx-code[color])
└─ children（文字列）
```
role: code（ARIA 1.3 の `code`）。

### 12.3 API
| Astryx prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `children` | ReactNode | 必須 | `soft::code(value: impl IntoText)` |
| `color` | `'primary'\|'secondary'\|'inherit'` | `'primary'` | `.color(CodeColor::Primary)` |
| `size` | `'inherit'` | なし（既定は `--text-code-size`） | `.size_inherit(true)`（周りの文字と同じ size / line-height） |
| `ref`, `className`, `style`, `xstyle` | | | 落とす / NodeBuilder |

### 12.4 Visual spec
`Code.tsx:30-41`:
- font-family `--font-family-code`（ui-monospace …）。font-size `--text-code-size` 14px。line-height `inherit`（周りの行の高さ。単独なら 20px）。weight は継承（400）。
- 背景 `--color-background-muted` `#f1f1f1` / dark `#1b1b1b`。
- padding-inline `--spacing-1` 4px、padding-block 0。radius `--radius-inner` 0.375rem = 6px。border なし。
- `overflow-wrap: break-word; word-break: break-word`（長い識別子は折れる）。
- color: primary `#000000` / `#ffffff`、secondary `#474747` / `#9e9e9e`、inherit。
- `size="inherit"`: font-size と line-height を親から継承。
- 状態なし。
```
Call  ⟦ useTheme() ⟧  to read tokens     背景 #f1f1f1, 左右 4px, 角 6px
```

### 12.5 Motion
なし。

### 12.6 Behaviour & a11y
対話なし。Kivact に code の role が無ければ role なし（テキストとして読まれる）。文言なし。

### 12.7 Kivact 実装方針
- NEW kit component。`text(value)` に font_family / background / padding / radius。analog: `crates/kivact-ui-basic/src/typography.rs` の `inline_code`。
- 段落の中に混ぜる場合は rich text の span（`kivact::view::span(..).font_family(..).background(..)`）。span の背景は角丸と左右 padding を持てない可能性があるので（UNRESOLVED: span の `background` が padding / radius を持てるか確認）、持てなければ Markdown の段落では「行内の独立 node を row で並べる」方式を使う（Markdown 章の段落レイアウトと一緒に決める）。

### 12.8 受け入れ確認
- [ ] 等幅 14px、背景 `#f1f1f1`（dark `#1b1b1b`）、左右 4px、角 6px
- [ ] `.color(Secondary)` で `#474747`
- [ ] 長い識別子が親幅で折り返す
- [ ] docsite https://astryx.atmeta.com/components/Code と light / dark で一致

---

## 13. Icon

### 13.1 概要
テーマのアイコン集合から名前で引いて描く、または SVG コンポーネントを直接描く（`packages/core/src/Icon/Icon.tsx`、`globalIconRegistry.tsx`、`defaultIcons.tsx`、`IconSize.stylex.ts`、`IconDefaultSizeContext.ts`、`useIcon.ts`）。
- すること: 意味の名前（`close`, `info` …）を使う（テーマを変えると絵も変わる）。アイコンの差し替えはテーマで（`defineTheme({icons})`）。独立した意味のあるアイコンには `label` を付ける。色は token で。
- しないこと: アイコンだけで意味を伝える。任意の px でサイズを変える。outline と filled を同じ文脈で混ぜる。

### 13.2 Anatomy
```
名前で指定:  <span class=.astryx-icon[size,color] inline-flex center>  ← サイズと色はこの箱
              └─ テーマの登録物（neutral: Lucide の <svg width=1em height=1em aria-hidden>）
部品で指定:  <svg class=.astryx-icon …>（IconType をそのまま描く）
a11y: label あり → role="img" aria-label=label / 無し → aria-hidden="true"
```
登録名の解決（`globalIconRegistry.tsx` の `selectIconEntry`）: (1) 現在のテーマの `icons[name]` → (2) `registerIcons()` で入れた全域の登録 → (3) core の `defaultIcons[name]` → 無ければ何も描かない（null）。登録物は ReactNode か `({size, variant}) => ReactNode` の関数（テーマが `iconVariants` に挙げた variant だけ渡される）。

### 13.3 API
| Astryx prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `icon` | `IconType \| IconName \| 'ns:name'` | 必須 | 既存の `kivact::control::icon(name)`（`kivact-core/src/control/node.rs:374`）。kit は関数を足さず、テーマの `IconSet` に名前を入れる（basic と同じ規則）。サイズと色の既定を当てる薄い helper `soft::icon(name).size(IconSize::Md).color(IconColor::Secondary)` は任意 |
| `color` | `'primary'\|'secondary'\|'tertiary'\|'disabled'\|'accent'\|'info'\|'success'\|'error'\|'warning'\|'inherit'\|'blue'\|'red'\|'green'\|'gray'\|'cyan'\|'teal'\|'yellow'\|'orange'\|'pink'\|'purple'` | `'inherit'` | `IconColor` enum（同じ 20 個） |
| `size` | `'xsm'\|'sm'\|'md'\|'lg'` | 文脈（`IconDefaultSizeProvider`）→ 無ければ `'md'` | `IconSize { Xs, Sm, Md, Lg }`。文脈の既定サイズは kit 内部の context（`provide_context`）で、Button などの部品が子のアイコンに渡す |
| `variant` | テーマが定義する名前 | なし | Kivact の `IconSet` は名前 1 つにつき 1 枚。variant は「名前に接尾辞」（`"info"` / `"info.filled"`）で表す。neutral は variant を持たないので当面不要 |
| `label` | string | なし | `.name(label)`（Kivact では name があれば読み上げ、無ければ装飾） |
| `xstyle`, `className`, `style`, `ref` | | | NodeBuilder |
| （hook）`useIcon(name)` | | | 不要（`icon(name)` がその場で現在のテーマを読む） |
| `registerIcons()` | 全域登録 | | 不要（テーマが唯一の経路。Kivact の方針どおり） |
| `getIconRegistry()` / `getIcon()` / `getExtendedIcon()` | | | 不要。必要なら `current_theme().icons().get(name)` |

### 13.4 Visual spec
**サイズ**（`IconSize.stylex.ts`。rem、16px root）: xsm 0.75rem = **12px**、sm 1rem = **16px**、md 1.25rem = **20px**、lg 1.5rem = **24px**。名前指定では外側の span がこの幅・高さを持ち、`font-size: min(w, h)`。中の svg は 1em なので同じ大きさ。テーマは `components.icon['size:sm']` で幅・高さを変えられる（`--_icon-width/height`）が、neutral は変えていない。

**neutral の絵**: Lucide（lucide-react、`themes/neutral/src/icons.tsx`）。viewBox 24、`stroke: currentColor`、stroke-width 2（Lucide 既定）、round cap / join、fill none。

**色**（`Icon.tsx:95-160`、neutral 解決値 light / dark）:
| color | token | light | dark |
|---|---|---|---|
| primary | `--color-icon-primary` | `#000000` | `#ffffff` |
| secondary / tertiary | `--color-icon-secondary` | `#6a6a6a` | `#9e9e9e` |
| disabled | `--color-icon-disabled` | `#919191` | `#525252` |
| accent | `--color-accent` | `#1b1b1b` | `#f1f1f1` |
| info | `--color-icon-info`（= text-info） | `#0067ce` | `#a1caff` |
| success | `--color-icon-success` | `#2f7d33` | `#90ca90` |
| warning | `--color-icon-warning` | `#836700` | `#eec448` |
| error | `--color-icon-error` | `#c0262c` | `#ffaea7` |
| blue | `--color-icon-blue` | `#003978` | `#88bcff` |
| red | `--color-icon-red` | `#76000c` | `#ff9890` |
| green | `--color-icon-green` | `#00490b` | `#90ca90` |
| gray | `--color-icon-gray` | `#474747` | `#9e9e9e` |
| cyan | `--color-icon-cyan` | `#004351` | `#71c7dd` |
| teal | `--color-icon-teal` | `#00463d` | `#81c9bb` |
| yellow | `--color-icon-yellow` | `#4b3900` | `#e2b623` |
| orange | `--color-icon-orange` | `#622700` | `#ff9e55` |
| pink | `--color-icon-pink` | `#70003f` | `#fd92bd` |
| purple | `--color-icon-purple` | `#5c0e6c` | `#e496f6` |
| inherit | 親の color | | |

**レイアウト**: inline-flex、中央揃え、`flex-shrink: 0`（行の中で縮まない）。RTL で自動反転はしない（chevronLeft/Right は呼ぶ側が方向を見て選ぶ）。

**名前の一覧と neutral の Lucide 対応**（`IconName`、`globalIconRegistry.tsx` の型。neutral は 29 個すべてを上書き）。Kivact の名前は kivact-ui-basic と同じ Lucide の kebab 名を使い、core 側の標準名（`IconSet::standard()` の `check`, `x`, `chevron-down`, `chevron-right`, `minus`, `plus`, `dot`, `toast-*`）とも衝突しないようにする:
| Astryx 名 | neutral の Lucide 部品 | Kivact の名前（提案） | core default（neutral に無いときの絵） |
|---|---|---|---|
| close | X | `x` | ✕ 2 本線 |
| chevronDown | ChevronDown | `chevron-down` | ▾ |
| chevronLeft | ChevronLeft | `chevron-left` | ‹ |
| chevronRight | ChevronRight | `chevron-right` | › |
| chevronsLeft | ChevronsLeft | `chevrons-left` | « |
| chevronsRight | ChevronsRight | `chevrons-right` | » |
| check | Check | `check` | ✓ |
| success | CheckCircle（= `circle-check-big`） | `circle-check-big` | ✓ in circle（solid） |
| error | XCircle（= `circle-x`） | `circle-x` | ✕ in circle（solid） |
| warning | AlertTriangle（= `triangle-alert`） | `triangle-alert` | △! （solid） |
| info | Info | `info` | ⓘ（solid） |
| calendar | Calendar | `calendar` | |
| clock | Clock | `clock` | |
| externalLink | ExternalLink | `external-link` | |
| menu | Menu | `menu` | ☰ |
| moreHorizontal | MoreHorizontal（= `ellipsis`） | `ellipsis` | ⋯ |
| search | Search | `search` | |
| upload | Upload | `upload` | |
| arrowUp | ArrowUp | `arrow-up` | |
| arrowDown | ArrowDown | `arrow-down` | |
| arrowsUpDown | ArrowUpDown | `arrow-up-down` | |
| funnel | Filter（= `funnel`） | `funnel` | |
| eyeSlash | EyeOff | `eye-off` | |
| viewColumns | Columns（= `columns-2`） | `columns-2` | |
| copy | Copy | `copy` | clipboard |
| checkDouble | CheckCheck | `check-check` | ✓✓ |
| wrench | Wrench | `wrench` | |
| stop | Square | `square` | ■（solid、角丸） |
| microphone | Mic | `mic` | |
| `numberInput:stepperDown`（名前空間付き。テーマに無ければ core default） | （neutral は上書きしない）core default: stroke 1.5 の小さな下向き chevron `M7.5 9.75l4.5 4.5 4.5-4.5` | `soft-stepper-down`（kit 独自 SVG） | |
※ lucide-react の別名（CheckCircle → circle-check-big、AlertTriangle → triangle-alert、MoreHorizontal → ellipsis、Filter → funnel、Columns → columns-2）は lucide-react の版によって違う。docsite の絵と照らして確定する（UNRESOLVED: 特に success が `circle-check` か `circle-check-big` か）。

**core の各部品が使う名前**（非テストの .ts/.tsx を走査した結果。テーマ経由の名前だけ。部品ごとの描き方は各担当章）:
| 部品 | 名前 |
|---|---|
| Banner | chevronDown, close（＋ status 色） |
| Breadcrumbs, ButtonGroup, ComplexSelector, MultiSelector, Selector, TopNav | chevronDown |
| Calendar, Carousel | chevronLeft, chevronRight |
| Chat | arrowUp（送信）, check, checkDouble（既読）, chevronDown, clock, error, microphone（音声入力）, stop（生成停止）, success, warning, wrench（ツール呼び出し） |
| CodeBlock | copy, check（コピー済み）, chevronRight（折りたたみ） |
| Collapsible | chevronDown, chevronRight |
| CommandPalette, Tokenizer | search |
| ContextMenu | chevronLeft（RTL のサブメニュー） |
| DateInput, DateTimeInput | calendar, chevronDown, chevronLeft, chevronRight（DateTimeInput は clock も） |
| DateRangeInput | calendar |
| Dialog, Drawer, Toast, Token, Thumbnail, PowerSearch | close |
| DropdownMenu | chevronDown, chevronLeft, chevronRight, clock |
| Field | close（クリア）, info, search |
| FileInput | upload |
| Indicator, Markdown（タスクリスト）, Typeahead | check |
| Lightbox | chevronLeft, chevronRight, close |
| Link | externalLink |
| MobileNav | close, menu |
| MoreMenu | moreHorizontal |
| NumberInput | `numberInput:stepperDown` |
| Pagination | chevronLeft, chevronRight, chevronsLeft, chevronsRight |
| SideNav | chevronDown, chevronLeft |
| Stepper | chevronLeft, chevronRight, error, success, warning |
| TabList | check, chevronDown, chevronLeft, chevronRight |
| Table | arrowDown, arrowUp, check, chevronDown, chevronRight, close, funnel, info |
| TextArea | warning |
| TimeInput | clock |
| Timestamp | copy, check |
| TreeList | chevronRight |
| hooks（useClipboard 系の部品、input の状態アイコン） | check, copy, search |
（`menu`, `eyeSlash`, `viewColumns`, `arrowsUpDown` は型にはあるが core の部品は使っていない。Table の拡張部品向け。kit では登録だけしておく。）
kit 独自の部品で他に要る Lucide 名（basic の `ICON_NAMES` から流用）: `minus`, `plus`, `circle-alert`, `chevron-up` など、各担当章で必要になったものを `soft::ICON_NAMES` に足す。

**状態**: アイコン単体に状態はない。親部品が disabled なら `IconColor::Disabled` または inherit で親の色。

### 13.5 Motion
なし（親部品のアイコン切り替え、たとえば copy → check は各部品の章）。

### 13.6 Behaviour & a11y
- 既定は装飾（Kivact では name なし）。`label` があれば `.name(label)`、role img（Kivact の `icon()` は image node。name を付ければ読み上げられる）。
- 文言なし。

### 13.7 Kivact 実装方針
- 既存の facility に対応: `kivact::control::icon(name)` + テーマの `IconSet`（`Theme::with_icon`）。kit は `soft::theme(mode)` で Lucide の SVG（ISC ライセンス）を `assets/icons/*.svg` に埋め込んで登録する。やり方は `crates/kivact-ui-basic/src/icons.rs`（`ICON_NAMES` と `bytes()`）をそのまま真似る。
- サイズ・色の既定: `IconSize` / `IconColor` から `Style`（`size(px)` と `color(..)`）を作る helper `soft::icon_style(size, color)` を kit に置く。`icon()` は monochrome なので color がそのまま塗り色になる。
- 文脈の既定サイズ（`IconDefaultSizeProvider`）: kit 内部の `provide_context(IconSizeContext(Sm))` を Button などが子に渡し、`soft::icon()` helper が読む。
- 16px 未満の箱では core の icon が縮む（node.rs のドキュメント）。xsm 12px は箱を 12px にすれば良い。
- Lucide の stroke-width 2 は 24 viewBox 基準なので、12px では 1px 線になる（ブラウザと同じ）。

### 13.8 受け入れ確認
- [ ] `icon("x")` などが Lucide の絵で出る。登録されていない名前は何も描かず、debug ビルドで報告される
- [ ] サイズ 12 / 16 / 20 / 24px
- [ ] `IconColor::Secondary` は `#6a6a6a`（dark `#9e9e9e`）、`Error` は `#c0262c`（dark `#ffaea7`）
- [ ] name を付けたアイコンだけが支援技術に出る
- [ ] docsite https://astryx.atmeta.com/components/Icon の名前一覧と並べ、29 個の絵が同じ形

---

## 14. CodeBlock

### 14.1 概要
読み取り専用の、構文ハイライト付きコード表示（`packages/core/src/CodeBlock/CodeBlock.tsx`、`tokenizer.ts`、`grammar.ts`、`languages/*.ts`、`highlightStyles.ts`、`highlightRanges.ts`、`CodeBlock.doc.mjs`）。ヘッダー（タイトルと言語ラベル）、コピーボタン、行番号、強調行、unified diff 表示、折り返し、最大高さとスクロール、折りたたみを持つ。
- すること: 内容に合う `language` を指定する（分からなければ `plaintext`）。ファイルを表すときは `title` を付ける（ヘッダーにコピーボタンと並ぶ）。本文中の短い参照は Code（§12）、独立した複数行は CodeBlock。
- しないこと: 5 行未満の短いスニペットに行番号を付ける。スクロールする容器の中に入れる（`maxHeight` を使う）。

### 14.2 Anatomy
```
CodeBlock root <pre class=.astryx-code-block[size,language,container]>
├─ Header row（title か言語ラベルがあるときだけ。sticky top:0）
│   ├─ Header（折りたたみ可能なら role=button tabIndex=0 aria-expanded aria-controls=region）
│   │    └─ Title <span>: [chevron(折りたたみ時のみ)] {title}{" — "}{languageLabel}
│   └─ Controls: [endContent …] [Copy IconButton]
├─ (折りたたみ可能なら) Collapse region <div id=region inert=折りたたみ中> ─ inner
│   └─ Code viewport（scroll。内容がはみ出すときだけ Tab 停止。role=group、name = 言語ラベル か "Code"）
│        └─ Code wrapper ─ <code>
│             └─ Line <div data-line=n data-diff=added|removed|header>（行番号は ::before）
│                  ├─ (diff) marker <span>（+ / - / 空白）
│                  └─ line content（トークンの span 列）
└─ (ヘッダーなしで controls があるとき) Corner cluster: [endContent …] [Copy]（コードの右上の別カラム。コードに重ならない）
```

### 14.3 API
| Astryx prop | 型 | 既定 | 意味 | Kivact |
|---|---|---|---|---|
| `code` | string | 必須 | コード本文。末尾の改行 1 つは空行にしない（`CodeBlock.tsx:1001-1007`） | `soft::code_block(code: impl IntoValue<String>) -> CodeBlock` |
| `language` | `string \| CodeLanguage` | `'plaintext'` | 組み込み識別子、または言語値（`defineCodeLanguage` / `createCodeLanguage`） | `.language(impl Into<CodeLanguageInput>)`。`&str` は組み込み識別子、`soft::CodeLanguage` は独自言語（14.7） |
| `title` | string | なし | ヘッダーのタイトル | `.title(impl IntoText)` |
| `hasLanguageLabel` | bool | true | 言語名をヘッダーに出す（plaintext は出さない） | `.language_label(bool)` |
| `hasLineNumbers` | bool | false | 行番号 | `.line_numbers(impl IntoValue<bool>)` |
| `highlightLines` | number[] | なし | 強調する行（1 始まり） | `.highlight_lines(impl IntoValue<Vec<u32>>)` |
| `isDiff` | bool | false | unified diff として描く | `.diff(bool)` |
| `hasCopyButton` | bool | true | コピーボタン | `.copy_button(bool)` |
| `onCopy` | `() => void` | なし | コピー成功後に呼ぶ | `.on_copy(impl Fn() + 'static)` |
| `endContent` | ReactNode | なし | Copy の前に置く呼び出し側の操作（ヘッダーを作らない。押しても折りたたまない） | `.end_content(impl FnOnce() -> V)` |
| `isWrapped` | bool | false | 長い行を折り返す | `.wrap(impl IntoValue<bool>)` |
| `maxHeight` | number(px) \| string | なし | viewport の最大高さ。超えたら縦スクロール | `.max_height(impl Into<Length>)` |
| `isCollapsible` | bool | false | ヘッダーで折りたためる | `.collapsible(bool)` |
| `collapsibleThreshold` | number | 10 | この行数以上で折りたたみ可能になる | `.collapsible_threshold(usize)` |
| `size` | `'sm' \| 'md'` | `'md'` | 文字サイズ | `.size(CodeBlockSize::Sm \| Md)` |
| `width` | CSS 幅 | `'fit-content'` | 幅 | `.width(CodeBlockWidth::Fit \| Fill \| Px(f32))`（`'100%'` は `Fill`） |
| `container` | `'card' \| 'section'` | `'card'` | 枠と背景の有無 | `.container(CodeBlockContainer::Card \| Section)` |
| `syntaxTheme` | `SyntaxThemeDefinition` | なし | このブロックだけの配色 | `.syntax_theme(soft::SyntaxTheme)`（§18 の SyntaxTheme と同じ型） |
| `tokenizer` | 関数（deprecated） | なし | | 落とす（壊してよい方針）。独自 tokenize は `CodeLanguage::custom(..)` |
| `highlightMode` | `'auto'\|'ranges'\|'spans'` | `'auto'` | ブラウザの描画方式の切り替え | 落とす（ネイティブでは意味がない） |
| `ref`, `xstyle`, `className`, `style`, `data-testid` | | | | 落とす / NodeBuilder（`.test_id(..)`） |

折りたたみ状態は Astryx では内部 state（初期は展開）。kit は既定で内部 signal、呼び出し側が持ちたいときは `.expanded(Signal<bool>)` を受ける。

### 14.4 Visual spec
**root**（`CodeBlock.tsx:126-135`、`containerStyles` :73-92）
- `display:flex; flex-direction:column; overflow:hidden; isolation:isolate`。背景 `--color-syntax-background`。neutral の syntax theme（`themes/neutral/src/neutralTheme.ts` の `defineSyntaxTheme`）では `#ffffff` / dark `#111111`。
- `container='card'`: border 1px（`--border-width`）solid `--color-border` `#00000014` / `#ffffff1A`、radius `--radius-element` 0.625rem = 10px（中身は overflow hidden で角に沿って切れる）。
- `container='section'`: radius 0、border なし、`--color-syntax-background: transparent`（root と sticky ヘッダーの両方が透明）。
- width（`dynamicStyles.width` :94-99）: `fit-content`（最長行の幅）。そのとき min-width `min(100%, 400px)`、max-width 100%。`'100%'` なら親いっぱい。

**ヘッダー行**（:152-194）
- flex、縦中央、space-between。padding-inline `--spacing-4` 16px、padding-block `--spacing-2` 8px（行番号の有無に関わらず 8px）。背景は root と同じ `--color-syntax-background`。sticky top 0、z-index 1（縦スクロール時に残る。viewport はヘッダーの下にあるので実際には root の中でだけ効く）。
- 行番号ありのとき（`headerWithDivider` :176）だけ、下に 1px `--color-border` の区切り線。行番号なしのとき（`headerCompact`）は線なしで、コード側が `margin-block-start: -8px`（`codeWrapperCompact` :199）でヘッダーに寄る。
- タイトル（:185-194）: font `--font-family-code`、`--text-supporting-size` 12px、weight `--font-weight-medium` 500、line-height 1.6667（20px）、色 `--color-syntax-comment` `#6a6a6a` / `#9e9e9e`。表示は `{title} — {languageLabel}`（間は空白 + em dash + 空白。:1185）。言語ラベルは識別子のまま（`tsx`、`rust`）。diff では `diff`（plaintext のとき）または `<lang> diff`。
- ヘッダーの高さ: タイトル 20px と Copy ボタン 28px の大きい方 + 上下 8px = 44px（Copy なしなら 36px）。
- controls（:427-432）: flex、gap `--spacing-1` 4px、shrink しない。
```
┌──────────────────────────────────────────────┐ radius 10, 1px #00000014
│ › status.ts — tsx                      [⧉]  │ ← header 44px, 12px mono #6a6a6a
├──────────────────────────────────────────────┤ ← 行番号ありのときだけ線
│  1 │ const status = response.ok ? 'ok' : …   │
│  2 │ export default status;                  │
└──────────────────────────────────────────────┘
```

**コード本体**（:256-279）
- `<code>`: block、`min-width: max-content`（行の背景がスクロール幅いっぱいに伸びる）、padding-block `--spacing-3` 12px、padding-inline 16px。font `--font-family-code`（`ui-monospace, "SF Mono", Monaco, Consolas, "Liberation Mono", "Courier New", monospace`）。地の文字色 `--color-syntax-variable` `#111111` / `#e2e2e2`。`tab-size: 2`、`white-space: pre`（折り返さない）。
- size（:404-409）: `md` は `--text-code-size` 14px、`sm` は `--text-supporting-size` 12px。行の line-height はどちらも `--text-code-leading` 1.4286（md 20px、sm 12×1.4286 ≈ 17.14px）。
- wrap（`codeWrapped` :273）: `min-width:auto; white-space:pre-wrap; word-break:break-all; overflow-wrap:break-word`（語の途中でも折る）。
- 1 行のブロック（ヘッダーなし）は高さ 12+20+12 = 44px。

**行番号**（:285-320）
- 行ごとの 2 列 grid: `[ガター幅] [コード]`。ガター幅 = 最大行番号の桁数 × `ch`（等幅の「0」の幅。`dynamicStyles.gutterWidth` :104）。列間 = 12 + 1 + 16 = 29px。
- 番号は右寄せ、色 `--color-syntax-punctuation` `#6a6a6a` / `#9e9e9e`、選択不可、行の先頭の視覚行に揃う（折り返した行でも 1 行目に付く）。
- 縦の区切り線: `<code>` の上端から下端まで、left = 16px + ガター幅 + 12px の位置に 1px `--color-border`。

**強調行**（`lineHighlighted` :330-334）: 背景 `--color-accent-muted` `#f1f1f1` / `#262626`。左右に 16px はみ出して（margin-inline −16px、padding-inline 16px）コード領域の端から端まで塗る。diff の色より後に適用され、diff の色を置き換える（重ねない）。

**diff**（:335-403、トークン定義 `theme/syntax/tokens.ts`、未設定時の扱い `theme/syntax/defineSyntaxTheme.ts` の `defineSyntaxTheme`）
- 行の種類（`tokenizer.ts` の `diffLineKind` / `splitDiffLines` / `fileHeaderLines`）: `@@` で始まる行は header。`--- ` の次が `+++ ` でその次が `@@` なら、その 2 行も file header。`+` は added、`-` は removed、それ以外は context。先頭の空白がない context 行は glyph なし（行全体がコード）。
- glyph 列: 行は `[1ch] [コード]` の grid、gap `--spacing-2` 8px。行番号ありなら `[ガター] [auto] [コード]` で、marker に margin-inline-start 29px と margin-inline-end 8px。glyph は選択不可で、支援技術には読まれる。
- 色（neutral は diff キーを定義していないので既定と fallback を使う）:
  - added 行の背景 `--color-syntax-added` = `color-mix(--color-success-muted 25%, transparent)`。light `#bce0bb` の 25% = `#bce0bb40`、dark `#90ca903D` の 25% ≈ `#90ca900F`。
  - removed 行の背景 `--color-syntax-removed` = `--color-error-muted` の 25%。light `#ffc4be40`、dark ≈ `#ff98900F`。
  - 行の左右 16px はみ出しは強調行と同じ。
  - added / removed の marker と行番号: neutral の syntax theme は marker を定義していないので、defineSyntaxTheme の fallback で `var(--color-syntax-punctuation)` = `#6a6a6a` / `#9e9e9e`。core 既定の緑・赤（`--color-text-green` / `--color-text-red`）ではない。
  - context の marker: `--color-text-secondary` `#474747` / `#9e9e9e`。
  - header 行: 背景 `--color-syntax-diff-header`。neutral では fallback の `transparent`。文字色 `--color-syntax-punctuation`。トークン色は付けず生の文字列。
  - コードのトークン色はどの行でも同じ（変更行でも同じ識別子は同じ色）。
- コピーは glyph を含む元のソースをそのまま返す。

**Copy ボタン**（:1101-1122）: ghost の IconButton、size `sm`（`--size-element-sm` 28px の正方形。Button 章の ghost / sm の見た目。hover は overlay の塗り）。アイコンは Icon size sm 16px、`copy`（neutral: Lucide `copy`）。成功後 2000ms は `check`。ボタンの文字色は `--color-syntax-comment` `#6a6a6a` / `#9e9e9e`（`copyButton` :410-416）。
- corner cluster（ヘッダーなしのとき。:417-443）: root が `grid-template-columns: minmax(0,1fr) auto` の 2 列になり（`rootCorner` :147）、右列に controls。flex、gap 4px、`align-self:start`、margin-inline 8px。高さ = 2×12px + 1 行の高さ（md 14×1.4286 = 44px、sm 12×1.4286 + 24 ≈ 41.14px）。controls はこの中で縦中央、つまり 1 行目の中央に来る。

**折りたたみ**（:202-245、:1055-1069、:1162-1183）
- 条件: ヘッダーがあり、`isCollapsible`、行数 ≥ `collapsibleThreshold`（既定 10）。初期は展開。
- chevron: タイトルの先頭、14×14px の箱、margin-inline-end 4px、色 syntax-comment。中に Icon `chevronRight` xsm 12px。展開中は 90° 回転（下向き）、折りたたみ中は 0°（inline-end 向き）。RTL では箱ごと左右反転（`rtlStyles.mirror`）。
- ヘッダー全体がボタン（cursor pointer、user-select none、focus-visible で共通の focus ring: 2px solid `--focus-outline-color`（= `--color-accent` `#1b1b1b` / `#f1f1f1`）、offset 3px）。
- 折りたたみ中: region の grid 行が `0fr`（高さ 0）になり、`inert`（Tab にも支援技術にも出ない）。ヘッダーは残る。

**スクロール**: viewport は両方向スクロール。`maxHeight` で縦の上限（数値は px）。はみ出しているときだけ viewport が Tab で止まり、矢印でスクロールできる（`useScrollableArea`）。`maxHeight={0}` も有効（高さ 0）。

**状態のまとめ**
| 状態 | 変化 |
|---|---|
| Copy hover | ghost ボタンの hover 面（Button 章） |
| Copy 押下成功 | アイコン copy → check、name "Copy code" → "Copied"、2000ms 後に戻る |
| ヘッダー focus-visible | focus ring 2px / offset 3px |
| 折りたたみ | 本体の高さ 0、chevron 0° |
| viewport focus-visible（はみ出し時） | 共通の focus ring（viewport の外周） |

### 14.5 Motion
| 対象 | trigger | プロパティ | from → to | duration | easing | 出典 |
|---|---|---|---|---|---|---|
| 折りたたみ本体 | ヘッダーの押下 | grid 行（= 高さ） | 1fr ↔ 0fr（実高さ ↔ 0） | `--duration-medium` neutral 300ms（開く・閉じる同じ） | `--ease-standard` cubic-bezier(0.24, 1, 0.4, 1) | `CodeBlock.tsx:202-215` |
| chevron の回転 | 同上 | rotate | 90° ↔ 0° | `--duration-fast` neutral 125ms | ease-standard | :235-245 |
| chevron の出現 | 折りたたみ可能になったとき（mount 時など） | width / margin-inline-end / opacity | 0 / 0 / 0 → 14px / 4px / 1（タイトルが右へ滑る） | 300ms、1 回 | ease-standard | :113-124、:226-231 |
| Copy アイコン | コピー成功 | 切り替えのみ（アニメーションなし） | copy → check、2000ms 後 check → copy | — | — | `hooks/useClipboard.ts`（`DEFAULT_RESET_AFTER_MS = 2000`） |
reduced motion: chevron の出現アニメーションは無し（`animationName: none`）。折りたたみと回転の transition には reduced-motion の分岐がない（そのまま動く）。kit では Kivact の `reduced_motion()` が true のとき 3 つとも即時にする（Astryx より強めだが見た目の差は小さい）。

### 14.6 Behaviour & a11y
- Copy（`useClipboard`）: `navigator.clipboard.writeText(code)` が成功したら isCopied=true、polite で "Copied" を読み上げ、2000ms 後に false。2000ms 以内の再コピーはタイマーを延長する（テスト「keeps the copied indicator a full 2s after a rapid re-copy」）。成功時だけ `onCopy`。押下はヘッダーに伝えない（折りたたまない）。tooltip はいつも "Copy code"（コピー後も変えない）。name はコピー中だけ "Copied"。
- 折りたたみヘッダー: `role=button`、Enter / Space で切り替え、`aria-expanded`、`aria-controls`。タイトルも言語ラベルも無いときの name は "Code"。Copy ボタンと endContent はヘッダーのボタンの外にある（入れ子にしない）。
- viewport: `role=group`、name = 言語ラベル（diff なら `tsx diff`）、無ければ "Code"。はみ出しているときだけ Tab 停止。
- endContent は CodeBlock が名前も見た目も付けない（呼び出し側が ghost sm の IconButton にする想定）。
- 文言（i18n キー）:
  - `soft.code_block.copy_code` = "Copy code"（`@astryx.codeBlock.copyCode`。tooltip と通常時の name）
  - `soft.code_block.copied` = "Copied"（`@astryx.codeBlock.copied`。コピー後の name と読み上げ）
  - `soft.code_block.code` = "Code"（`@astryx.codeBlock.code`。viewport / ヘッダーの既定 name）

**ハイライトの仕組み（Astryx）**
- 正規表現の列による行単位のスキャナ（`tokenizer.ts:594-641` の `tokenizeLine`）。各位置でパターンを上から順に試し（sticky `y` フラグ）、最初に当たったものを採る。当たらなければ 1 文字進む。言語の `defaultType` と同じ種類はトークンにしない。トークンは行の中だけで、行をまたぐ構文（複数行のブロックコメント、Python の三重引用符）は続きの行で色が切れる（テスト `tokenizer.test.ts:78-81`）。
- 2000 文字以上（`SYNC_TOKENIZE_THRESHOLD`）は 800 文字ごとに yield する非同期 tokenize（:570-715）。トークンが届くまでは無色で出る。
- トークン種類（`highlightStyles.ts` の `TOKEN_TYPES`）と neutral の色（`--color-syntax-*`、light / dark）:
| 種類 | light | dark |
|---|---|---|
| keyword | `#6b187c` | `#efa8ff` |
| type | `#6b187c` | `#efa8ff` |
| string | `#0b5615` | `#a4d6a3` |
| comment | `#6a6a6a` | `#9e9e9e` |
| operator | `#6a6a6a` | `#9e9e9e` |
| punctuation | `#6a6a6a` | `#9e9e9e` |
| number | `#733100` | `#ffb37e` |
| constant | `#733100` | `#ffb37e` |
| function | `#00458c` | `#a1caff` |
| variable（地の色） | `#111111` | `#e2e2e2` |
| tag | `#8a0011` | `#ffaea7` |
| attribute | `#584400` | `#eec448` |
| property | `#005348` | `#90d7c8` |
| background | `#ffffff` | `#111111` |
- 組み込み言語の識別子（`tokenizer.ts:151-455`）: `typescript` `javascript` `tsx` `jsx` `ts` `js`／`json`／`html` `xml` `svg`／`css` `scss` `less`／`python` `py`／`bash` `sh` `zsh` `shell`／`php`／`hack`／`yaml` `yml`／`markdown` `md`／`c` `h`／`cpp` `c++` `cc` `cxx` `hpp` `hh`／`sql`／`rust` `rs`／`kotlin` `kt` `kts`／`swift`／`go` `golang`。それ以外（`plaintext` を含む）は色なし。
- 別 import の言語モジュール（`languages/*.ts`、`defineCodeLanguage` + `cLike`）: `dockerfile`（docker）、`graphql`（gql）、`java`、`lua`、`objective-c`（objc, m, mm）、`powershell`（ps1, pwsh）、`ruby`（rb）、`thrift`、`toml`。
- C 系の共通形（`grammar.ts` の `cLike`）: comment（`/* */`, `//`）→ 言語固有 → `"…"` / `'…'` string → number（0x / 0b / 0o / 10 進 / 指数 / 接尾辞）→ keyword 集合 → function（識別子の直後に `(`）→ type（大文字始まり）→ operator（`+-*/%=!<>&|^~?:` の連続）→ punctuation（`{}()[];,.@`）。JS 系は他に template string、`@decorator` を constant。JSON は `"key":` を property、`true/false/null` を constant、地は punctuation。HTML は tag / attribute / string / comment。CSS は property / function / tag / keyword（`!important` など）/ number+単位。

### 14.7 Kivact 実装方針
- NEW kit component `soft::code_block`。一番近い analog は `crates/kivact-ui-basic/src/code_block/view.rs`（読み取り専用の複数行 text box `TextEditing::new(..).multiline(true).read_only(true).highlight(..)` を `Role::TextBox` として置き、行ごとの文字色を `TextHighlight { range, color }` で渡す。ドラッグ選択と Ctrl+C がそのまま使える）。色の表は basic の `CodeColors`（`tokens.rs:80`）に倣い、soft の `Tokens` に `syntax: SyntaxColors`（上表の 14 色 + diff の 5 色）を持たせる。
- ハイライト: basic の `rust_lines`（`code_block/rust.rs`。正規表現を使わない手書きの字句解析で、行をまたぐコメントと文字列にも対応）と同じ方式で、`cLike` 相当の汎用スキャナ（keyword 集合、コメント区切り、引用符、数値、`ident(`、大文字型、演算子、区切り）を Rust で書き、上の言語表を表データで持つ。行をまたぐコメントの色は Astryx より良くなるが、それは見た目の改善として許す。**判断事項（parent）**: kit は公開 `kivact` crate だけに依存する（`crates/kivact-ui-basic/Cargo.toml` の dependencies は `kivact` だけ）。`regex` や `syntect` を足せば Astryx の正規表現を移植しやすいが、依存方針に反する。手書きスキャナを推奨する。
- 独自言語: `soft::CodeLanguage::new(name, identifiers).grammar(SimpleGrammar { .. })` または `CodeLanguage::custom(name, identifiers, fn(&str) -> Vec<(Range<usize>, TokenKind)>)`。
- 行ごとの背景（強調行、diff の added / removed / header）: Kivact の `TextHighlight` は文字色だけで、行の背景を持たない。折り返しなしのときは行の高さが一定（line_height × i）なので、text box の後ろに同じ高さの行の帯（`column` の各行 node、または `Style::paint` の矩形）を重ねて描く。**折り返しありで行番号・行背景を使うとき**は視覚行の数が分からないので（UNRESOLVED: text box から行ごとの折り返し高さを得る API があるか確認）、その組み合わせだけ「行ごとの text node を column に並べる」描き方に切り替える（行をまたぐ選択はできなくなる）。
- 行番号: basic の `gutter()`（数字を改行で連結した右寄せ text）と同じ。幅は `measure_text("0") × 桁数`。区切り線は gutter の右に 1px の border。
- コピー: Astryx は自分でクリップボードに書くので、kit も `kivact::platform::copy_text(code)`（`kivact-core/src/platform/services.rs:154`）を `kivact::task::spawn` で呼び、成功時に copied を 2000ms 立てて `on_copy` を呼ぶ。basic は「コピー要求を callback で知らせるだけ」の方針なので、どちらにするかは parent の判断（推奨: soft は自分で書く。Astryx の挙動と同じ）。2000ms の表示は basic と同じ `kivact::interaction::use_delayed` で作る。読み上げ "Copied" は live region（VisuallyHidden 章の helper）。
- Copy ボタン: `kivact::ui::button(name).icon(icon("copy")).hide_label(true).variant(ButtonVariant::Ghost).size(ControlSize::Small)` を restyle で使い、tooltip は `kivact::ui::tooltip("Copy code").attach(..)`。
- 折りたたみ: `kivact::interaction::use_disclosure` でヘッダーをボタンにする（`kivact::ui::disclosure` の restyle も可。analog `crates/kivact-ui-basic/src/disclosure.rs`）。高さのアニメーションは `kivact::motion::animated` で 0 ↔ 内容高さ。折りたたみ中は子を focus 不可にする。
- viewport: `kivact::control::scroll(child)`（`container.rs:75`）、両軸。`max_height` を付ける。Tab 停止を「はみ出し時だけ」にするのは、scroll にその機能が無ければ常に Tab 停止にする（UNRESOLVED）。
- sticky ヘッダー: ヘッダーは scroll の外（column の 1 段目）に置けば、それだけで sticky と同じ見た目になる。
- `fit-content` 幅: 最長行の幅を `measure_text` で測り、`min(親幅, max(400, 最長行 + 32))` にする。
- CSS の `ch` は等幅の「0」の幅なので、`measure_text("0")` で得る。

### 14.8 受け入れ確認
- [ ] `code_block("const a = 1;").language("tsx")` は 1px 枠（`#00000014`）と角 10px、背景 `#ffffff`（dark `#111111`）。ヘッダーに 12px mono の `tsx`
- [ ] keyword は `#6b187c`、string は `#0b5615`、number は `#733100`、function は `#00458c`（dark はそれぞれ `#efa8ff` / `#a4d6a3` / `#ffb37e` / `#a1caff`）
- [ ] title と language の両方があると `status.ts — tsx` と表示される
- [ ] Copy を押すとアイコンが check、name が "Copied" になり、2 秒後に戻る。tooltip は "Copy code" のまま。クリップボードにソースそのものが入る
- [ ] 行番号は右寄せ `#6a6a6a`、縦の区切り線があり、ヘッダーの下にも線が出る
- [ ] `highlight_lines([2])` の行は端から端まで `#f1f1f1`（dark `#262626`）
- [ ] diff: `+` の行は薄い緑、`-` の行は薄い赤、`@@` の行は背景なし。glyph は独立した列
- [ ] 10 行以上で collapsible なら、ヘッダーの押下（Enter / Space も）で 300ms で閉じ、chevron が 125ms で回る。role=button、expanded が切り替わる
- [ ] ヘッダーなしのとき Copy は右上の別カラムで、1 行目の中央に来て、コードに重ならない
- [ ] `.wrap(true)` で長い行が折り返し、行番号は各行の 1 行目に付く
- [ ] docsite https://astryx.atmeta.com/components/CodeBlock の各例と light / dark で並べて一致

---

## 15. Markdown

### 15.1 概要
Markdown 文字列を Astryx の部品で描く（`packages/core/src/Markdown/Markdown.tsx`、`parser.ts`、`ast.ts`、`streaming.ts`、`url.ts`、`plugins/*`、`plugin-renderer/`、`remark.ts`、`Markdown.doc.mjs`）。利用者が書いた内容、AI の応答、ドキュメントに使う。見出し、リスト、表、コードブロック、引用、出典チップを揃った見た目で出し、`isStreaming` で逐次表示する。
- すること: ページの見出し階層に合わせて `headingLevelStart` を設定する（h2 の節の中なら 3）。幅の広いレイアウトでは `contentWidth` で読みやすい行長にする（既定 680px）。
- しないこと: 自分で組めるレイアウトに使う（そのときは Text / Heading を直接使う）。

### 15.2 Anatomy
```
Markdown root <div role="document" class=.astryx-markdown[density]>   （display="inline" なら <span>、role なし）
├─ heading     <h{n} id=slug>（Heading 部品ではなく素の hN に heading token。§9 と同じ寸法）
│    └─ (heading links plugin) 行 = [hN] [# copy ボタン]
├─ paragraph   <div role="paragraph">（<p> にしない）
├─ code        CodeBlock（§14。isCollapsible、isWrapped）
├─ blockquote  Blockquote（§10）。中は block の再帰
├─ list        List（listStyle disc/decimal、density compact）> ListItem
│    └─ (pure task list) CheckboxList（read-only、compact、label 非表示 "Task list"）> CheckboxListItem
├─ table       Table（dividers=grid、textOverflow=wrap）> Header / Body / Row / HeaderCell / Cell
├─ hr          <hr>
├─ image       <div><img></div>（段落が画像だけのとき）
├─ math        components.math があるときだけ。無ければ `$$…$$` の生文字
└─ extension   plugin が描く block
inline: text / strong / emphasis / delete / inlineCode(Code §12) / link(<a>) / image(<img>) /
        citation(Citation §11) / break(<br>) / inlineMath / extension
```

### 15.3 API
| Astryx prop | 型 | 既定 | 意味 | Kivact |
|---|---|---|---|---|
| `children` | string | 必須 | Markdown 本文 | `soft::markdown(source: impl IntoValue<String>) -> Markdown` |
| `display` | `'block' \| 'inline'` | `'block'` | inline は段落の中に埋める短い Markdown（inline 構文だけ） | `.inline(true)`（inline は rich text の span 列として返す） |
| `density` | `'default' \| 'compact'` | `'default'` | block 間の間隔 | `.density(MarkdownDensity::Default \| Compact)` |
| `headingLevelStart` | 1-6 | 1 | `#` を何番の見出しにするか（6 で止まる） | `.heading_level_start(u32)` |
| `isStreaming` | bool | false | 逐次表示とフェード | `.streaming(impl IntoValue<bool>)` |
| `onLinkClick` | `(href, event) => void \| false` | なし | リンク押下。false で既定動作を止める | `.on_link(impl Fn(&str) + 'static)`（ネイティブには既定動作が無いので、押下は常にこれで通知） |
| `sources` | `Record<id, {title?, url?, src?, icon?}>` | なし | `[id]` / `【id】` を出典チップにする | `.sources(impl IntoValue<Vec<(String, CitationSource)>>)` |
| `citationStyle` | `'label' \| 'number'` | `'label'` | チップの型 | `.citation_style(CitationVariant)` |
| `contentWidth` | number(px) \| string | **680** | 文章（見出し・段落・リスト・引用）の最大幅。表とコードは min-width として使う | `.content_width(Option<f32>)`（None で無制限） |
| `contentAlign` | `'start' \| 'center'` | `'start'` | contentWidth が狭いときの寄せ | `.content_align(ContentAlign::Start \| Center)` |
| `components` | 各要素の差し替え | なし | code / inlineCode / math / citation / link / heading / paragraph / image / blockquote / hr | `.render_code(Fn(&str, Option<&str>) -> View)` などの個別 slot。最初の実装は `code` / `link` / `image` / `heading` の 4 つで十分 |
| `plugins` | `createMarkdownPlugin` の列 | なし | 構文追加・AST 変換・拡張描画 | 最初は対象外（UNRESOLVED: kit の plugin API は第 2 段階）。heading links と soft breaks だけ組み込みオプションにする: `.heading_links(HeadingLinks { id_prefix, base_url })`、`.soft_breaks(true)` |
| `inlinePlugins` | `{pattern, getEndIndex?, render}[]` | なし | テキストだけを対象にした置換 | `.inline_rule(matcher: impl Fn(&str) -> Option<(Range<usize>, View)>)`（正規表現を使わない関数） |
| `autolink` | `'gfm'` | なし | 素の URL・メールを自動リンク | `.autolink(true)` |
| `codeLanguages` | `CodeLanguage[]` | なし | fence が使える独自言語 | `.code_languages(Vec<soft::CodeLanguage>)` |
| `ref`, `xstyle`, `className`, `style`, `id`, `data-testid` | | | | 落とす / NodeBuilder |

### 15.4 Visual spec
**root**（`Markdown.tsx:452-462`）: font `--font-family-body`（Figtree …）、`--text-body-size` 14px、line-height `--text-body-leading` 1.4286（20px）、色 `--color-text-primary` `#000000` / `#ffffff`。width 100%、min-width 0、`overflow-wrap: break-word`。inline 版（:463-471）は size / line-height / 色 / family を親から継承する。

**block 間の間隔**（`getElementSpacing` :1247-1287、値 :517-591）。上下とも margin で、隣どうしは margin が重なる（CSS の margin collapse。大きい方が残る）。先頭の block は上の margin 0、最後の block は下の margin 0（:590-595 `noMarginBlockStart/End`）。
| 要素 | default（上 / 下） | compact（上 / 下） |
|---|---|---|
| h1-h3 | 24px / 12px（`--spacing-6` / `--spacing-3`） | 16 / 8 |
| h4-h6 | 16 / 8 | 12 / 4 |
| paragraph、extension | 12 / 12 | 4 / 4 |
| code、math | 16 / 16 | 8 / 8 |
| blockquote | 16 / 16 | 8 / 8 |
| list | 12 / 12 | 4 / 4 |
| table | 16 / 16 | 8 / 8 |
| hr | 24 / 24 | 12 / 12 |
| image | 12 / 12 | 8 / 8 |
**注意**: Kivact に margin collapse は無い。隣り合う 2 つの block の間隔は `max(前の下, 次の上)` で計算して gap として入れる（例: 段落 → h2 は max(12, 24) = 24px、h2 → 段落は max(12, 12) = 12px）。入れ子（引用やリストの中）でも同じ規則を再帰で使う。

**contentWidth**: 見出し・段落・リスト・引用・extension は `max-width: 680px`（既定）。`contentAlign='center'` なら左右 auto で中央。表とコードブロックは `min-width: min(680px, 100%)`（中央寄せも同じ）。

**見出し**（:472-506）: family `--font-family-heading`、色 primary。寸法は §9 の表と同じ（h1 24/32 600、h2 20/28 600、h3 17/24 **700**、h4 14/20 **700**、h5 12/20 600、h6 10/16 600。neutral）。level = `min(depth + headingLevelStart − 1, 6)`。id はスラッグ（`parser.ts:5241` の `slugify`: trim → 小文字 → `'` と `"` を消す → `[^a-z0-9]+` を `-` に → 両端の `-` を除く。空なら `section`。重複は `-1`、`-2` …）。

**段落**: 文字は root の body 14/20。強調 `strong` は weight `--font-weight-semibold` 600（:610）。`emphasis` は italic（ブラウザ既定）。`delete` は取り消し線（ブラウザ既定）と色 `--color-text-secondary` `#474747` / `#9e9e9e`（:613）。

**inline code**: Code（§12）そのまま。表のセルの中では `word-break: normal`（識別子を途中で折らない。:441-443）。

**リンク**（:616-619）: 色 `--color-text-accent` `#1b1b1b` / `#f1f1f1`、下線。hover / focus の専用 style は無い（下線のまま。focus は共通の focus ring）。`http://` / `https://` は新しいタブで開く外部リンク、それ以外は内部リンク（router の Link 部品）。安全でない URL は `<span>` の文字だけになる（15.6）。

**画像**（:607-609）: `max-width: 100%`、radius `--radius-element` 10px。段落が画像 1 つだけなら block（上下 12px）。src が安全でないと `[alt]` の文字。

**hr**（:600-605）: 上辺だけ 1px solid `--color-border` `#00000014` / `#ffffff1A`。他の border 0。

**引用**: Blockquote（§10）。中の block は同じ再帰の間隔規則。

**コードブロック**（:1556-1594）: CodeBlock（§14）を `isCollapsible`、`isWrapped`（長い行は折り返す）、その他は既定（card、fit-content 幅、Copy あり、言語ラベルあり、行番号なし）で描く。fence の言語: info 文字列の最初の語（`parseFenceLanguage`: `diff` は plaintext の diff、`diff-<lang>` は `<lang>` の diff）。`codeLanguages` に同じ識別子があれば組み込みより優先。外側の div は `max-width: 100%`。

**リスト**（:1693-1931、List `List/List.tsx:134-182`、ListItem `List/ListItem.tsx:157-345`、Item `Item/Item.tsx:346-362, 551-560`）
- List: `density="compact"`、行間 gap `--spacing-0-5` 2px。各行は Item: padding-block `--spacing-1` 4px、内側の左右 padding 8px（Markdown は `blockIndent` で margin-inline −8px を当てて打ち消すので、marker は文章の左端に揃う）、marker と本文の gap 8px、radius 10px（背景は無いので見えない）。本文は body 14/20。
- 箇条書きの marker: 幅 16px の列の中央に 6×6px。上端は `(20px − 6px)/2 = 7px` 下げる（1 行目の中央）。深さで形が変わる（`BULLET_MARKERS` :117）: 1 段目 disc（`--color-text-primary` の塗り丸）、2 段目 circle（1px の primary の輪、中は透明）、3 段目 square（6px の塗り四角）、4 段目でまた disc。
- 番号付き: `1.` 形式（counter + `.`）、右寄せ、min-width 16px、`tabular-nums`、body 14/20、色 primary。深さで decimal → lower-alpha（`a.`）→ lower-roman（`i.`）（`NUMBER_MARKERS` :122）。`start` を尊重する。番号の列は List 全体で共有（`grid-template-columns: max-content minmax(0,1fr)`、列間 8px）で、桁が増えても本文の開始位置は揃う。番号付きの List は `markerGutter`（margin-inline-start −24px、padding-inline-start 16px）で、番号が文章の左余白にはみ出す形になる。
- 入れ子: 子の List は親の本文の位置（marker 16px + gap 8px = 24px 右）から始まる。
- task list（すべての項目が `[ ]` / `[x]` で、各項目が段落だけ）: CheckboxList（read-only、compact、label 非表示で name "Task list"）。checkbox は 20px（CheckboxInput sm）で、上端を `(20 − 20)/2 = 0` に揃える。checked の見た目は CheckboxList 担当章。混在や入れ子の task 項目は通常のリストで、marker の位置に read-only の checkbox（name は項目の本文）。
```
• First item            1. Step one
  ◦ nested                 a. sub step
    ▪ deeper                  i. deeper
☑ Done (task list, read-only)
```

**表**（:1932-2031、`markdownTableCellStyles` :403-446。Table 本体の見た目は Table 担当章）
- Table を `dividers="grid"`（セルの間の線）、`textOverflow="wrap"` で使い、表の外周に 1px `--color-border`（`border-collapse` で内側の線と 1 本にまとまる）。
- セル: padding 上下左右 `--spacing-2` 8px、`box-sizing: content-box`、max-width なし、`overflow-wrap: break-word; word-break: normal`。ヘッダーのセルは省略記号にせず折り返す（`white-space: normal; text-overflow: clip`）。
- 列ごとの最小幅（`computeTableColumnMinWidths` :1319-1345、単位 ch）: `min(24, max(4, ceil(本文の最長セルの文字数 / 2), min(ヘッダーの文字数, 20)))`。ヘッダーのセルにだけ min-width を付ける。
- 揃え: `:---:` は中央、`---:` は end、それ以外は start。
- 表全体は Table の横スクロール領域の中（はみ出したら横スクロール。Markdown の外側の div はスクロールしない）。

**出典チップ**: Citation（§11）。`citationStyle` の型。番号は文書の中で最初に出た順に 1 から（同じ id は同じ番号）。

**heading links**（`plugins/HeadingLinksRenderer.tsx`、`hooks/containerReveal.stylex.ts`。`createMarkdownHeadingLinks()` を入れたときだけ）
- 行: flex、`align-items: last baseline`。見出しの右（inline-end）に margin 6px（`--spacing-1-5`）あけて幅 28px（`--size-element-sm`、coarse pointer では 24px）の枠。
- 中の ghost sm の icon-only Button（28×28、coarse 24×24、padding 0）。表示は `#` の文字（見出しと同じ font、色 `--color-text-secondary` `#474747` / `#9e9e9e`）。コピー後 1500ms は Icon `check` sm 16px。ボタンは見出しの縦中央に置く（top 50% と translate −50%）。
- 表示: 普段は opacity 0（場所は取ったまま）。見出しの行に hover（`hover:hover` のポインタ）するか、行の中に focus があるか、coarse pointer の端末なら opacity 1。

**状態**: Markdown 自体の状態は無い。リンク・Copy・heading link・表のスクロールは各部品の状態。

### 15.5 Motion
| 対象 | trigger | プロパティ | from → to | duration / easing | 出典 |
|---|---|---|---|---|---|
| ストリーミングの新しい文字 | `isStreaming` 中に表示文字が増えたとき | opacity | 0 → 1（`@starting-style`） | `--duration-medium` neutral 300ms、`--ease-standard` cubic-bezier(0.24, 1, 0.4, 1) | `Markdown.tsx:659-679` |
| 逐次表示の速度 | `isStreaming` | 表示文字数 | `useStreamingText`（§18）: tick = `max(4, round(--duration-fast-min / 10))` = neutral round(95/10) = 10ms。rAF の各 frame で前の tick から 10ms 以上たっていれば 10 文字（natural）進める。60Hz では 1 frame（16.7ms）ごとに 10 文字 ≈ 600 文字/秒 | `hooks/useStreamingText.ts` |
| フェード中の span の上限 | | | 同時にフェードする塊は最大 `min(ceil(--duration-fast-max / tick), 12)` = neutral min(ceil(165/10), 12) = 12 個。古い塊から確定（フェードなし）に移る | `Markdown.tsx` の `maxSpans`、`streaming.ts` の `computeBoundaries` |
| heading link の出現 | 行の hover / focus-within | opacity | 0 ↔ 1 | `--duration-fast` neutral 125ms、ease-standard、遅延 0 | `hooks/containerReveal.stylex.ts:216-224` |
| CodeBlock / CheckboxList / Table | | | | 各章 | |
- ストリーミングの仕組み: 描画ごとに「前回の表示文字数」を境界として記録する（最大 12 個のリングバッファ）。最も古い境界より前の文字は確定、境界と境界の間はそれぞれ別の span でフェードする。span の key は境界の値で固定なので、途中で作り直されない。テキストを数えるのは text ノードだけ（inline code は長さを数えるが割らない。コードブロックは 1 つの塊として数える）。
- reduced motion: `useStreamingText` は逐次表示をやめて全文を返す。フェードの duration は 0s（即時）。heading link の fade も 0s。
- caret（点滅カーソル）は無い。

### 15.6 Behaviour & a11y
**対応する構文**（独自の CommonMark 0.31 寄りパーサ。`parser.ts:3639-4047` の block 判定、inline は `parseInlineImpl` :2422 以降）
- block: ATX 見出し `#`〜`######`（3 つまでの字下げ可）、fenced code（```` ``` ```` / `~~~`、3 文字以上、字下げ 3 まで、閉じない fence は文書の終わりまで）、HR（`---` `***` `___`、空白入り可）、段落だけの画像、GFM 表（両端の `|` は省略可）、引用 `>`（入れ子、lazy continuation）、箇条書き `-` `*` `+`、番号付き `1.` `1)`、GFM task list `[ ]` `[x]`、link reference definitions、拡張 block（plugin）、display math `$$`（`components.math` があるときだけ）。入れ子は 100 段まで（超えると平文）。
- inline: `*` / `_` の emphasis と strong（CommonMark の delimiter 規則。`***x***` は strong の中に emphasis）、`~~` 取り消し、`` ` `` code span、inline / reference / 角括弧 destination のリンクと画像、hard break（行末の空白 2 つ、または `\`）、backslash escape、文字参照（`&copy;` `&#169;` `&#xA9;`）、出典 `[id]` / `【id】`（id が `sources` にあるときだけ）、autolink（`autolink:'gfm'` のときだけ: `https?://…`、`www.…`、`<scheme:url>`、`<email>`、`user@host`。末尾の `?!.,:*_~` と対になっていない `)` は除く）、inline math `$…$`（opt-in）。
- 対応しないもの: setext 見出し（`===` / `---` の下線）、字下げ 4 つのコードブロック、生の HTML（文字として出る）、脚注（`[^1]` はそのまま文字）。soft break は空白（`markdownSoftBreaksPlugin` で改行にできる）。（UNRESOLVED: setext と字下げコードが本当に無いかは docsite で `Heading\n===` を入れて確認。parser にその分岐は見当たらない）
- URL の安全性（`url.ts`）: リンクは `sanitizeUrl`（`utils/safeUrl`）を通ったものだけ。通らなければ文字だけの span。画像は制御文字を除いたあと `javascript:` / `data:` / `vbscript:` で始まるものを拒否し、`[alt]` の文字にする。

**ストリーミング中の未完成構文の扱い**（`parser.ts:4390` の `trimStreamingArtifacts`、:4652 の `trimUnsettledStructural`、:4304 の `findSettledBoundary`）。最後の行の末尾だけを直す:
- 閉じていない `[` / `![`（後ろに `](` と `)` が無い）は、その位置から隠す（リンク文字が生で一瞬出ない）。
- 末尾の閉じていない `` ` `` の連続は隠す。
- 末尾の `*` の連続（3 つまで）で開き側が無ければ隠す。
- 行の途中の閉じていない `**bold` / `*ital` は、内容があれば閉じ記号を補って書式付きで出す（`**bold ` → `**bold** `）。開き記号の後が空白だけなら隠す。
- 末尾の `~~`（開き無し）と単独の `~` は隠す。`~~` が奇数個なら最後の `~~` から隠す。
- math が有効なら閉じていない `$` / `$$` の塊を隠す。
- 末尾の行が中身の無い list marker（`- ` / `1. `）、区切り行の無い表のヘッダー候補（`|` を含む行）、ヘッダーの無い区切り行、空行なら、その行を出さない。表のヘッダーと区切りが揃ったら、データ行は届いたそばから出す。
- 確定した block（空行で区切られ、次の行が行頭から始まるもの。開いた fence の中は確定しない）は再 parse しない（incremental state）。

**操作と a11y**
- root `role="document"`。段落は `role="paragraph"`（div）。見出しは heading + level。
- リンク: Enter / click で `onLinkClick(href)`。外部リンクは新しいタブ。Kivact: `Role::Link`、Enter / Space / Activate で `on_link(href)`。
- heading link ボタン: name `soft.markdown_heading_links.copy` = "Copy link to {heading}"（`@astryx.markdownHeadingLinks.copy`）、コピー後 1500ms は `soft.markdown_heading_links.copied` = "Link copied"（name と polite の読み上げ）。修飾キーなしの左クリック / Enter / Space で、`permalinkBaseUrl` + `#id` の URL をコピーする（移動もスクロールもしない）。
- task list: `soft.markdown.task_list` = "Task list"（`@astryx.markdown.taskList`。CheckboxList の非表示 label、本文を取れない task 項目の name）。checkbox は read-only（押しても変わらない）。
- 表: Table のスクロール領域が名前と Tab 停止を持つ（Table 章。`@astryx.table.label` = "Table"）。`@astryx.markdown.table` = "Table" は en.json にあるが core では使われていない（`soft.markdown.table` として予約だけ）。
- コードブロック: §14 の文言（Copy code / Copied / Code）。

### 15.7 Kivact 実装方針
- NEW kit component `soft::markdown`。analog は `crates/kivact-ui-basic/src/typography.rs`（見出し・段落・引用・inline code・箇条書きの tokens の引き方）と `code_block/`（コード）。描画は「AST → kit 部品」の再帰関数 1 つ（`render_block(&Block, ctx) -> View`、`render_inline(&[Inline]) -> Vec<Span>`）。
- **パーサ（parent の判断事項）**: kit は公開 `kivact` crate だけに依存する（`crates/kivact-ui-basic/Cargo.toml`、README の「The only normal and development dependency is the public `kivact` crate」）。
  - 案 A（推奨）: kit 内に小さな手書きパーサを持つ。Astryx のパーサも CommonMark 全部ではなく、上の「対応する構文」の範囲。block を行単位で判定し（ATX 見出し、fence、HR、表、引用、リスト、段落）、inline は delimiter stack で emphasis を解く。ストリーミング用の `trim_streaming_artifacts` と「確定した block の再利用」も同じ crate に置ける。
  - 案 B: `pulldown-cmark` を依存に足す。正確だが依存方針に反する。さらに Astryx の差分（setext なし、`【id】` 出典、未完成構文の補完、task list の描き分け）を後から足す必要がある。
- inline の描画: Kivact の `kivact::control::rich_text(spans)` + `kivact::view::span(..)` で、strong（weight 600）、emphasis（`.italic(true)`）、delete（`text_decoration(LINE_THROUGH)` + secondary 色）、link（accent 色 + 下線）を表す。inline code（背景・角丸・左右 padding）、出典チップ、画像は span では表せない（UNRESOLVED: rich_text に inline の view を埋め込めるか確認。§12.7 と同じ問題）。埋め込めなければ、段落を「span の連続」と「inline view」に分け、折り返し可能な flow row（`row` + wrap）に並べる。その場合、単語単位で折り返せるように、テキストを語ごとの node に分ける必要がある（性能と選択に影響）。**ここが最大の難所**。
- リンクの押下: span 単位の hit test が rich_text にあるか（UNRESOLVED）。無ければリンクを独立した inline node にする（上と同じ flow 方式）。
- リスト: `kivact::ui::list` はインタラクティブな listbox なので使わない。kit の `column` で marker 列 16px + gap 8px + 本文。番号の列幅はリスト全体の最大値を `measure_text` で測る。
- task list: `kivact::ui::checkbox` を read-only（`.disabled` ではなく、押下を無視する read-only の見た目）で使う。read-only の API が無ければ、checkbox の見た目だけ描く kit の小部品にする。
- 表: kit の Table 部品（担当章）を使う。最小幅の ch は `measure_text("0")` × 値。
- コード: `soft::code_block(..).collapsible(true).wrap(true)`。
- block の間隔は 15.4 の「max(前の下, 次の上)」で gap を計算して column に入れる。
- ストリーミング: §18 の `use_streaming_text` を `kivact::motion::frame_loop` で実装し、新しい塊は span ごとの opacity を `kivact::motion::animated` で 0 → 1（300ms、ease-standard）。span ごとの透明度を rich_text が持てない場合は、塊（最大 12 個）を独立 node にするか、色の alpha を時間で上げる（`span.color(color.with_alpha(t))`）。後者を推奨（node を増やさない）。
- heading links: 見出しの行に hover / focus-within の検出が要る。`style().group_hover(..)`（`kivact-style/src/sheet.rs` の `group` / `group_hover`）で opacity 0 → 1。permalink の URL のコピーは `kivact::platform::copy_text`。
- 画像: `kivact::control::image(asset)`。URL からの読み込みはアプリの責任なので、`render_image(src, alt) -> View` の slot を必須にし、既定は `[alt]` の文字（Astryx の安全でない src と同じ見た目）。
- plugins / remark / frontmatter / source lines / source decoration / semantic fence / math: 第 2 段階（UNRESOLVED）。最初は `components` 相当の slot と、heading links・soft breaks・autolink のオプションだけ。

### 15.8 受け入れ確認
- [ ] root は role=document、本文 14px / 20px、`#000000`（dark `#ffffff`）
- [ ] `# A\n\ntext\n\n## B` で h1（24/32）と h2（20/28）、段落 → h2 の間隔 24px、h2 → 段落 12px、先頭の h1 の上は 0
- [ ] `headingLevelStart = 3` で `#` が level 3（17px / 700）になる
- [ ] `**b**` は 600、`*i*` は italic、`~~s~~` は取り消し線 + `#474747`
- [ ] リンクは `#1b1b1b` + 下線、押すと `on_link` に href が渡る。`javascript:` のリンクは文字だけ
- [ ] 箇条書きは 6px の丸、2 段目は輪、3 段目は四角。番号付きは `1.`、2 段目 `a.`、3 段目 `i.`
- [ ] task list は read-only の checkbox の列、name "Task list"
- [ ] 表は外周と格子の 1px 線、セル padding 8px、`:-:` 列は中央揃え
- [ ] fence は CodeBlock（折り返しあり、10 行以上で折りたたみ可能、Copy あり）。`diff-ts` fence は ts の diff
- [ ] hr は 1px の線で上下 24px
- [ ] `[1]` と `sources` で出典チップ。番号は出た順
- [ ] ストリーミング: 文字が約 600 文字/秒で増え、新しい部分が 300ms でフェードイン。`**bo` は太字で出て、`[lin` は出ない。中身の無い `- ` 行は出ない。reduced motion では全文が即時
- [ ] heading links を有効にすると、見出しに hover / focus で `#` ボタンが 125ms で出て、押すと "Link copied" になる
- [ ] docsite https://astryx.atmeta.com/components/Markdown の Default / Compact / AIResponse / Streaming の例と light / dark で一致

---

## 16. Timer

### 16.1 概要
進行中の作業の経過時間を標準の書式で出す（`packages/core/src/Timer/Timer.tsx`）。React の再描画なしに DOM の文字だけを書き換える。`elapsed`（コンパクトな単位。1 時間を超えると秒を落とす）と `clock`（m:ss / h:mm:ss。秒はずっと出る）。
- すること: 秒〜時間にまたがる短い表示は elapsed。1 時間を超えても秒に意味がある（ストップウォッチ）なら clock。Timer が mount される前に始まった処理には `startTime` を渡す。
- しないこと: 日付・タイムゾーン・相対日付に使う（Timestamp を使う）。毎 tick 読み上げてよい場面以外で `aria-live` を付ける。

### 16.2 Anatomy
```
Text (type=supporting, color=secondary, class .astryx-timer)   ← className/style/xstyle はここ
└─ <time dateTime="PT{n}S">  font は全部 inherit、tabular-nums  ← ref と残りの props（aria-live 等）はここ
    └─ "12s" / "3m 05s" / "1h 05m" / "3:05" / "1:02:03"
```
role は付けない（live region でもない。テストで固定: `Timer.test.tsx:294-301`）。

### 16.3 API
| Astryx prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `startTime` | number（Unix ms） | mount 時刻 | `soft::timer().start(impl IntoValue<SystemTime>)`。無ければ mount 時刻 |
| `format` | `'elapsed'\|'clock'` | `'elapsed'` | `.format(TimerFormat::Elapsed \| Clock)` |
| `type` | TextType | `'supporting'` | `.kind(TextKind::..)`（Text と共用） |
| `size` | TextSize | なし | `.size(..)` |
| `color` | TextColor | `'secondary'` | `.color(..)` |
| `weight` | TextWeight | なし | `.weight(..)` |
| `aria-live` など rest props | | | `.live(Live::Polite)` を任意で（既定なし） |
| `ref`, `xstyle`, `className`, `style` | | | NodeBuilder |

### 16.4 Visual spec
- 既定は Text の supporting: 12px / line-height 20px / 400 / Figtree、色 `--color-text-secondary` `#474747`（dark `#9e9e9e`）。`<time>` は `font-variant-numeric: tabular-nums`（`Timer.tsx:30-40`）なので桁が変わっても幅が揺れない。
- 書式（`getPresentation`、`Timer.tsx:55-95`）。経過秒 s = floor(経過 ms / 1000)、負は 0:
  - elapsed: s < 60 → `"{s}s"`（例 `0s`, `42s`）。分 m = floor(s/60) < 60 → `"{m}m {ss}s"`（秒は 2 桁 0 埋め: `3m 05s`）。それ以上 → `"{h}h {mm}m"`（`1h 05m`、`26h 00m`。時は 0 埋めしない、上限なし）。
  - clock: h > 0 → `"{h}:{mm}:{ss}"`（`1:02:03`）、それ以外 `"{m}:{ss}"`（`0:00`, `12:34`）。
  - dateTime 属性: `PT{s}S`（elapsed の 1 時間以上は分単位に丸めた秒数）。
- 状態・レイアウト: inline。折り返さない程度の短さなので特別な規則なし。

### 16.5 Motion
アニメーションはない。更新の cadence（`getMillisecondsUntilNextChange`、`Timer.tsx:97-113`）:
- 表示が次に変わる瞬間に合わせて setTimeout: `precision − (経過 ms mod precision)`。precision は elapsed で 1 時間以上なら 60 000ms、それ以外 1000ms。clock はいつも 1000ms。
- startTime が未来: `startTime − now + 1000ms` 後に再確認（上限 2 147 483 647ms）。それまで 0 表示。
- 文字が変わったときだけ書き換える。

### 16.6 Behaviour & a11y
- 対話なし。既定では読み上げない。アクセシブル名は表示文字列。
- 単位 `s` `m` `h` と区切りは Astryx でもハードコード（i18n されていない）。kit も同じにする。翻訳したい場合の任意キー案: `soft.timer.seconds` = "{s}s"、`soft.timer.minutes_seconds` = "{m}m {ss}s"、`soft.timer.hours_minutes` = "{h}h {mm}m"（既定は使わない）。

### 16.7 Kivact 実装方針
- NEW kit component。`signal(String)` を `kivact::task::spawn` の中の `sleep(次の変化まで)` ループで書き、`soft::text(signal)` に `tabular_nums()` を足して描く。mount 中だけ回す（task は owner と一緒に cancel される前提。`on_cleanup` で止める）。
- 壁時計は `std::time::SystemTime`（kit は std を使ってよい）。テストでは時刻を差し込めるように `TimerClock` を context で差し替え可能にしておくと E2E が書きやすい（任意）。
- analog: なし。表示部は `crates/kivact-ui-basic/src/typography.rs` の `muted`。

### 16.8 受け入れ確認
- [ ] mount 直後 `0s`、5 秒後 `5s`、65 秒で `1m 05s`
- [ ] `start` を 2 時間前にすると `2h 00m` で、1 分ごとにしか変わらない
- [ ] clock 書式で `0:00` → `0:01` … `1:00:00`
- [ ] 12px secondary、数字の幅が揺れない
- [ ] 既定で live region でない
- [ ] docsite https://astryx.atmeta.com/components/Timer と light / dark で一致

---

## 17. Timestamp

### 17.1 概要
日時を読める文字にする（`packages/core/src/Timestamp/Timestamp.tsx`、`formatInstant.ts`、`formatRelativeTime.ts`、`tooltipEntries.ts`、`TimestampHoverCard.tsx`）。作成・更新・予定の時刻に使う。最近は相対、精度が要るなら絶対、`auto` なら自動で切り替え。相対表示には hover card で完全な絶対日時（コピー可）が出る。
- すること: フィードや一覧は `auto`。同じ列の中で書式を揃える。複数タイムゾーンの読者がいれば `isTimezoneShown`。ゾーンを比べる必要があれば `tooltipEntries`（2 つ以上のゾーンなら各行に label）。ダッシュボードは `isLive`。
- しないこと: 生の Unix 秒や ISO を見せる。利用者向け UI で `system_*` 書式を使う。相対表示の hover card を消す。card にゾーンを詰め込む（最大幅 300px）。`EST` のような固定オフセット名を timezoneID に使う（`America/New_York` を使う）。

### 17.2 Anatomy
```
[HoverCard trigger（showTooltip のとき）: hover/focus で開く、placement above、focusTrigger always、点線下線]
└─ Text (type=supporting, color=secondary, class .astryx-timestamp[format])
    └─ <time dateTime=ISO, tabIndex=0（tooltip あり）, aria-label=完全な日時（相対書式のとき）>  表示文字列
HoverCard の中身（label = "Timestamp details"）:
<dl grid>                         列: [label] value [action]（必要な列だけ）
  ├─ <dt> label（任意）  <dd> value  <div> CopyButton（isCopyable の行）
  └─ …
```

### 17.3 API
| Astryx prop | 型 | 既定 | Kivact |
|---|---|---|---|
| `value` | `string \| number` | 必須 | `soft::timestamp(value: impl IntoValue<Instant>)`。`soft::Instant` は Unix ms（i64）を持つ型。`From<SystemTime>`、`Instant::unix_seconds(i64)`、`Instant::unix_millis(i64)`、`Instant::parse_iso(&str) -> Option<Instant>`。Astryx の数値ヒューリスティック（`value < 1e12` なら秒、それ以外 ms）も `Instant::from_number(f64)` として残す |
| `format` | `'relative'\|'relative_short'\|'auto'\|'date'\|'date_long'\|'date_weekday'\|'date_time'\|'time'\|'system_date'\|'system_date_time'\|'system_time'\|'unix_seconds'` | `'auto'` | `.format(TimestampFormat::..)`（同名の 12 variant） |
| `autoThreshold` | number（秒） | 604 800（7 日） | `.auto_threshold(Duration)` |
| `hasTooltip` | bool | true | `.hover_card(bool)` |
| `tooltipEntries` | `{timezoneID?, format?, label?, isCopyable?}[]` | なし | `.entries(Vec<TimestampEntry>)`。`TimestampEntry { zone: Option<TimeZone>, format: InstantFormat（既定 Full）, label: Option<String>, copyable: bool }` |
| `isTimezoneShown` | bool | false | `.show_time_zone(bool)` |
| `isLive` | bool | false | `.live(bool)` |
| `type`, `size`, `color`, `weight` | Text と同じ | supporting / なし / secondary / なし | Text と同じ |
| `data-testid`, `ref`, `xstyle`, `className`, `style`, rest | | | NodeBuilder |

### 17.4 Visual spec
- 本体: Text supporting 12px / 20px / 400、`#474747`（dark `#9e9e9e`）。`<time>` は全部 inherit（`Timestamp.tsx` の `styles.time`）。tabular-nums は付かない。
- hover card がある（相対書式か entries あり）とき: trigger に HoverCard の hover 表示 `text-decoration: underline dashed`、色 `--color-border-emphasized` `#d4d4d4`（dark `#525252`）、`text-underline-offset: --spacing-0-5` 2px（`HoverCard/HoverCard.tsx:103-108`）。Kivact の text_decoration に dashed が無ければ（UNRESOLVED）、`draw_over` で 1px の破線を自前で描く（3px 線 / 3px 間隔程度、docsite で実測）。
- card の中身（`TimestampHoverCard.tsx`）:
  - `<dl>` は grid、`align-items:center`、column-gap `--spacing-4` 16px、row-gap `--spacing-2` 8px、margin / padding 0。
  - 列テンプレート: label と action あり `auto 1fr auto`、label のみ `auto 1fr`、action のみ `1fr auto`、どちらも無し `1fr`。
  - label: `--text-supporting-size` 12px / 20px / 400、色 secondary、`white-space: nowrap`。
  - value: `--text-body-size` 14px / 20px / 400、色 `--color-text-primary` `#000000` / `#ffffff`、nowrap。
  - action: flex、右寄せ。CopyButton = IconButton `variant="ghost" size="sm"`、アイコン `copy` → コピー後 `check`（Icon size sm 16px、color inherit）。IconButton と HoverCard の箱（背景・角・影・padding・最大幅 300px）はそれぞれの担当章の値を使う。
- 既定の card（entries なし）: 1 行だけ。label なし、value = `full` 書式、コピー可 → 列は `1fr auto`。
- 表示文字列（locale `en` のときの Intl 出力。kit は英語でこれと同じ文字を作る）:
  | format | 例 | 規則 |
  |---|---|---|
  | `date` | `Mar 21, 2025` | month short, day numeric, year numeric |
  | `date_long` | `March 21, 2025` | month long |
  | `date_weekday` | `Fri, Mar 21, 2025` | weekday short を前に |
  | `date_time` | `Mar 21, 2025, 3:45 PM` | date + hour numeric + minute 2-digit（12 時間制）。`isTimezoneShown` で末尾に ` PDT` など短いゾーン名 |
  | `time` | `3:45 PM` | 同上、ゾーン名は `isTimezoneShown` のとき |
  | `full`（tooltip / aria 用） | `March 21, 2025 at 3:45:07 PM PDT` | month long + 秒 + short ゾーン名。aria-label は long ゾーン名（`Pacific Daylight Time`） |
  | `system_date` | `2025-03-21` | 壁時計（ローカルまたは指定ゾーン） |
  | `system_date_time` | `2025-03-21 15:45:07` | 24 時間 |
  | `system_time` | `15:45:07` | |
  | `unix_seconds` | `1742597107` | floor(ms/1000)、ゾーン無関係 |
  | `relative`（long） | `now`, `5 seconds ago`, `in 5 minutes`, `3 hours ago`, `yesterday`, `3 days ago`, `2 months ago`, `1 year ago` | 下の規則 |
  | `relative_short`（narrow） | `now`, `5s ago`, `in 5m`, `3h ago`, `1d ago`, `2mo ago`, `1y ago` | narrow は「yesterday」を使わない |
  calendar はいつも gregory。
- 相対時刻の規則（`formatRelativeTime.ts`）: diff = round((now − date)/1000) 秒。|diff| < 10、または未来で |diff| ≤ 30 → `now`。それ以外: < 60 → 秒、< 3600 → floor 分、< 86 400 → floor 時、< 30 日 → floor 日、< 365 日 → floor(÷30 日) 月、それ以上 floor(÷365 日) 年。過去は「… ago」、未来は「in …」。long で −1 日のときだけ `yesterday`（numeric auto）。
- auto: |diff| ≤ autoThreshold（既定 7 日）なら `relative`、超えたら `date_time`。
- tooltipEntries のゾーン名表示（`tooltipEntries.ts` `shouldShowZoneName`）: `full` はいつも、`date_time` / `time` はエントリが複数ゾーンにまたがるか、名前付きゾーンのときだけ。`timezoneID` が `local`（大小無視）か未指定ならローカル。未知の ID はローカルに落として dev 警告。
- 不正な value: 何も描かない（dev 警告 "could not parse value … as a date. Rendering nothing."）。

### 17.5 Motion
- Timestamp 自体のアニメーションはない。hover card の開閉は HoverCard の担当章（delay 300ms で開く、hideDelay 200ms で閉じる: `HoverCard.tsx:261-262`）。
- コピー ボタン: `copy` → `check` の切り替えは即時。1500ms（`COPY_FEEDBACK_MS`）後に `copy` に戻る。
- live 更新の cadence（`getLiveInterval`）: |diff| < 60 秒 → 1 秒ごと、< 1 時間 → 30 秒ごと、< 1 日 → 60 秒ごと、それ以上 → 5 分ごと。`isLive` かつ相対書式のときだけ。diff が変わるたびに間隔を選び直す。

### 17.6 Behaviour & a11y
- hover card があるときだけ `<time>` が tab 順に入る（tabIndex 0）。focus でも card が開く（focusTrigger always）。Escape / ポインタの離脱で閉じる（HoverCard の挙動）。
- 相対書式では aria-label に完全な日時（long ゾーン名）を入れる。読み上げは「March 21, 2025 at 3:45:07 PM Pacific Daylight Time」。見た目は「3 hours ago」。
- コピー: 押すとクリップボードに value を書き、`Copied` を polite でアナウンス、ボタンの tooltip が `Copy` → `Copied`、aria-label が `Copy {value}` → `Copied`。
- 文言（`@astryx.timestamp.*` → soft キー）:
  | soft key | English |
  |---|---|
  | `soft.timestamp.details_label` | Timestamp details（card の名前） |
  | `soft.timestamp.copy` | Copy（tooltip） |
  | `soft.timestamp.copy_value` | Copy {value}（ボタンの名前） |
  | `soft.timestamp.copied` | Copied |
- 日付書式の文言（Kivact に Intl が無いので kit が持つ。提案キー）: `soft.date.month_1`〜`_12`（January…December）、`soft.date.month_short_1`〜`_12`（Jan…Dec）、`soft.date.weekday_short_1`〜`_7`（Mon…Sun）、`soft.date.am` = AM、`soft.date.pm` = PM、テンプレート `soft.date.date` = "{month_short} {day}, {year}"、`soft.date.date_long` = "{month} {day}, {year}"、`soft.date.date_weekday` = "{weekday_short}, {month_short} {day}, {year}"、`soft.date.time` = "{hour12}:{minute} {ampm}"、`soft.date.date_time` = "{date}, {time}"、`soft.date.full` = "{month} {day}, {year} at {hour12}:{minute}:{second} {ampm}"。相対: `soft.relative_time.now` = now、`soft.relative_time.yesterday` = yesterday、long は `soft.relative_time.{second|minute|hour|day|month|year}_{past|future}_{one|other}`（例 `minute_past_one` = "{count} minute ago"、`minute_past_other` = "{count} minutes ago"、`minute_future_other` = "in {count} minutes"）、narrow は `soft.relative_time.short_{s|m|h|d|mo|y}_{past|future}`（"{count}s ago" / "in {count}s"）。Calendar の章でも月名・曜日を使うので、同じ `soft.date.*` を共用する。

### 17.7 Kivact 実装方針
- NEW kit component。本体は `soft::text`、card は kit の hover card（HoverCard の担当章。basic なら `crates/kivact-ui-basic/src/hover_card.rs` が analog）。コピー ボタンは kit の icon button + `kivact::platform::copy_text`。
- 書式は kit 内の小さな formatter に書く（Intl が無いので）。英語のテンプレートは上の message キーで差し替え可能にし、`kivact::i18n::message_with` で埋める。複数形は kit の `plural_en(count)`（one / other）で選ぶ。英語以外の複数形規則は UNRESOLVED（parent 判断: 当面 one/other だけ）。
- タイムゾーン: Rust std にはローカルのゾーンも tz データベースも無い。kit は `kivact` だけに依存する制約があるので、(a) ローカルの UTC オフセットとゾーン略称を app から `soft::provide_time_zone(TimeZone)` で受け取る、(b) 名前付きゾーン（`America/New_York`）は app が渡す `TimeZone` 実装（オフセットと名前を返す trait）でだけ扱う、を提案。これは parent の判断が要る（UNRESOLVED: tz の依存を kit に許すか）。
- live 更新: `kivact::task::spawn` + `sleep(間隔)` で `now` signal を更新。
- 時刻の注入: テスト用に `now` を context から読めるようにする（任意）。

### 17.8 受け入れ確認
- [ ] 5 分前の値は `5 minutes ago`、10 日前は `Mar 11, 2025, 3:45 PM` の形（auto）
- [ ] `relative_short` で `5m ago`、1 日前は `1d ago`（long は `yesterday`）
- [ ] 相対表示は点線の下線、Tab でフォーカスでき、hover / focus で card が出る。card の名前は "Timestamp details"
- [ ] card に完全な日時とコピー ボタン。押すと clipboard に入り、アイコンが check になり 1.5 秒で戻る
- [ ] 相対表示の name は完全な日時
- [ ] live で 1 分未満の値が毎秒更新される
- [ ] docsite https://astryx.atmeta.com/components/Timestamp と light / dark で一致

---

## 18. Utilities

この節は見た目を持たない仕組みを扱う。各項目に「既存の Kivact facility に対応 / kit helper にする / ネイティブでは不要」とその理由を書く。

### 18.1 InternationalizationProvider
- 何をするか（`packages/core/src/i18n/InternationalizationProvider.tsx`）: `locale`（BCP 47）、`messages`（locale → カタログ）、`overrides`（locale → key → 文字列。messages より優先）、`dir`（無ければ `Intl.Locale.getTextInfo()` から推定、`getLocaleDirection.ts`）を context に置く。解決は locale の連鎖（`en-US` → `en`）で overrides → messages → core の英語 fallback の順（`resolve.ts`）。見つからなければ key をそのまま返し、dev 警告 `missing key: … (locale: …)`。メッセージは ICU MessageFormat（`intl-messageformat`）。Provider が無いときの既定は locale `en`、dir `ltr`。core は 29 locale の翻訳 JSON を `packages/core/locales/*.json` に同梱している（ja-JP を含む）。
- Kivact: **既存の facility に対応**。`kivact::view::provide_locale` + `kivact::view::provide_messages`（App では `App::messages`）、方向は `kivact::view::provide_direction`。`Messages::layered` が overrides の役。
- kit がやること: 英語の既定は各呼び出しの `message(key, default)` の default 引数に持つ（basic と同じ）。Astryx 同梱の翻訳（MIT）を `soft::messages(locale: &str) -> Messages` として移植するかは parent 判断（任意。キーの対応表は 18.3 にある）。
- ICU の `plural` / `number` / `select` は Kivact の `message_with`（`{name}` の置換だけ。`kivact-i18n/src/context.rs:140,171`）では表せない。カタログの 64 件が ICU 構文を含む。kit helper `soft::i18n::format(key, default, args)` を作り、`{n, plural, one {…} other {…}}` と `{n, number}` だけ解釈する（英語の one/other 規則と 3 桁区切り）。`select` は使う部品側で key を分ける。

### 18.2 useTranslator / useLocale（と useDirection, useCollator, useListFormatter）
- `useTranslator()` は `(key, values?) => string`（`i18n/useTranslator.ts`）。`useLocale()` は locale 文字列。`useDirection()` は 'ltr' | 'rtl'。`useCollator()` は locale の `Intl.Collator`。`useListFormatter()` は `Intl.ListFormat`（"A, B, and C"）。
- Kivact: `useTranslator` → `kivact::i18n::message` / `message_with`（reactive な `Value<String>` を返す）。**既存の facility**。`useLocale` → `kivact::i18n::locale()`。**既存**。`useDirection` → `kivact::view::current_direction()`。**既存**。`useCollator` → kit helper（大小無視・数字順の比較だけ。Rust の `str::cmp` を lowercase で）。`useListFormatter` → kit helper `soft::i18n::join_list(items)`（英語 "A, B, and C"、`soft.list.and` = "and"）。

### 18.3 英語の既定文言カタログ（core 部品が出す全文言）
出典は `packages/core/locales/en.json`（568 キー）。部品ごとに、Astryx のキー、提案の soft キー（機械的に `@astryx.<部品>.<名前>` → `soft.<snake>.<snake>`）、英語の既定文を並べる。"used in" はそのキーを参照している `packages/core/src/<Dir>`（非テストのファイルを走査）。"not used by core components" のグループは core 外のパッケージ（chart、lab の DiffViewer / RichTextEditor / Comments / EmojiPicker / TransferList / ListInput など）か、実行時に組み立てるキーのもの。kit がその部品を作るときに使う。
規則:
- `{name}` の置換は `message_with`。`{x, plural, …}` / `{x, number}` / `{x, select, …}` は 18.1 の kit helper で処理する。
- 標準部品（`kivact::ui::dialog` など）を restyle するところでは、標準部品が自分の無印キー（`dialog.close`, `toast.close`, `slider.minimum` … kivact-ui-basic README 参照）を読み続ける。soft キーは kit の部品の文言に使う。
- 他の担当章で提案したキーと名前が食い違う場合は、この表の soft キーを正とする。
- 置換の変数名（`{accessibleName}` など）は en.json のままにする。

**pagination** (used in: Pagination, i18n)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.pagination.label` | `soft.pagination.label` | Pagination |
| `@astryx.pagination.previous` | `soft.pagination.previous` | Go to previous page |
| `@astryx.pagination.next` | `soft.pagination.next` | Go to next page |
| `@astryx.pagination.previousBy` | `soft.pagination.previous_by` | Go back {step, number} {step, plural, one {page} other {pages}} |
| `@astryx.pagination.nextBy` | `soft.pagination.next_by` | Go forward {step, number} {step, plural, one {page} other {pages}} |
| `@astryx.pagination.first` | `soft.pagination.first` | Go to first page |
| `@astryx.pagination.last` | `soft.pagination.last` | Go to last page |
| `@astryx.pagination.goToPage` | `soft.pagination.go_to_page` | Go to page {page, number} |
| `@astryx.pagination.goToPageInput` | `soft.pagination.go_to_page_input` | Go to page |
| `@astryx.pagination.pageLabel` | `soft.pagination.page_label` | Page |
| `@astryx.pagination.ofTotalPages` | `soft.pagination.of_total_pages` | / {total, number} |
| `@astryx.pagination.pageIndicators` | `soft.pagination.page_indicators` | Page indicators |
| `@astryx.pagination.itemsPerPage` | `soft.pagination.items_per_page` | Items per page |
| `@astryx.pagination.count` | `soft.pagination.count` | {from, number}–{to, number} of {total, number} |
| `@astryx.pagination.pageOfTotal` | `soft.pagination.page_of_total` | Page {current, number} of {total, number} |
| `@astryx.pagination.pageAnnounce` | `soft.pagination.page_announce` | Page {current, number} |

**powersearch** (used in: PowerSearch)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.powersearch.editor.field` | `soft.powersearch.editor.field` | Field |
| `@astryx.powersearch.editor.operator` | `soft.powersearch.editor.operator` | Operator |
| `@astryx.powersearch.editor.addFilter` | `soft.powersearch.editor.add_filter` | + Add filter |
| `@astryx.powersearch.editor.removeFilter` | `soft.powersearch.editor.remove_filter` | Remove filter |
| `@astryx.powersearch.editor.groupOperator` | `soft.powersearch.editor.group_operator` | Group operator |
| `@astryx.powersearch.editor.group` | `soft.powersearch.editor.group` | Group |
| `@astryx.powersearch.editor.delete` | `soft.powersearch.editor.delete` | Delete |
| `@astryx.powersearch.editor.cancel` | `soft.powersearch.editor.cancel` | Cancel |
| `@astryx.powersearch.editor.apply` | `soft.powersearch.editor.apply` | Apply |
| `@astryx.powersearch.valueEditor.value` | `soft.powersearch.value_editor.value` | Value |
| `@astryx.powersearch.valueEditor.values` | `soft.powersearch.value_editor.values` | Values |
| `@astryx.powersearch.valueEditor.time` | `soft.powersearch.value_editor.time` | Time |
| `@astryx.powersearch.valueEditor.date` | `soft.powersearch.value_editor.date` | Date |
| `@astryx.powersearch.valueEditor.relativeDate` | `soft.powersearch.value_editor.relative_date` | Relative date |
| `@astryx.powersearch.valueEditor.startDate` | `soft.powersearch.value_editor.start_date` | Start date |
| `@astryx.powersearch.valueEditor.endDate` | `soft.powersearch.value_editor.end_date` | End date |
| `@astryx.powersearch.valueEditor.entities` | `soft.powersearch.value_editor.entities` | Entities |
| `@astryx.powersearch.valueEditor.searchPlaceholder` | `soft.powersearch.value_editor.search_placeholder` | Search… |
| `@astryx.powersearch.valueEditor.enterValuePlaceholder` | `soft.powersearch.value_editor.enter_value_placeholder` | Enter value… |
| `@astryx.powersearch.valueEditor.addValuesPlaceholder` | `soft.powersearch.value_editor.add_values_placeholder` | Add values… |
| `@astryx.powersearch.valueEditor.enterNumberPlaceholder` | `soft.powersearch.value_editor.enter_number_placeholder` | Enter number… |
| `@astryx.powersearch.valueEditor.selectValuesPlaceholder` | `soft.powersearch.value_editor.select_values_placeholder` | Select values… |
| `@astryx.powersearch.operator.contains` | `soft.powersearch.operator.contains` | contains |
| `@astryx.powersearch.operator.notContains` | `soft.powersearch.operator.not_contains` | does not contain |
| `@astryx.powersearch.operator.startsWith` | `soft.powersearch.operator.starts_with` | starts with |
| `@astryx.powersearch.operator.notStartsWith` | `soft.powersearch.operator.not_starts_with` | does not start with |
| `@astryx.powersearch.operator.endsWith` | `soft.powersearch.operator.ends_with` | ends with |
| `@astryx.powersearch.operator.notEndsWith` | `soft.powersearch.operator.not_ends_with` | does not end with |
| `@astryx.powersearch.operator.is` | `soft.powersearch.operator.is` | is |
| `@astryx.powersearch.operator.isNot` | `soft.powersearch.operator.is_not` | is not |
| `@astryx.powersearch.operator.equals` | `soft.powersearch.operator.equals` | is |
| `@astryx.powersearch.operator.notEquals` | `soft.powersearch.operator.not_equals` | is not |
| `@astryx.powersearch.operator.greaterThan` | `soft.powersearch.operator.greater_than` | is greater than |
| `@astryx.powersearch.operator.lessThan` | `soft.powersearch.operator.less_than` | is less than |
| `@astryx.powersearch.operator.greaterThanOrEqual` | `soft.powersearch.operator.greater_than_or_equal` | is greater than or equal to |
| `@astryx.powersearch.operator.lessThanOrEqual` | `soft.powersearch.operator.less_than_or_equal` | is less than or equal to |
| `@astryx.powersearch.operator.before` | `soft.powersearch.operator.before` | is before |
| `@astryx.powersearch.operator.after` | `soft.powersearch.operator.after` | is after |
| `@astryx.powersearch.operator.between` | `soft.powersearch.operator.between` | is between |
| `@astryx.powersearch.operator.isTrue` | `soft.powersearch.operator.is_true` | is true |
| `@astryx.powersearch.operator.isFalse` | `soft.powersearch.operator.is_false` | is false |
| `@astryx.powersearch.operator.isAnyOf` | `soft.powersearch.operator.is_any_of` | is any of |
| `@astryx.powersearch.operator.isNoneOf` | `soft.powersearch.operator.is_none_of` | is none of |
| `@astryx.powersearch.valueEditor.itemsCount` | `soft.powersearch.value_editor.items_count` | {count, number} {count, plural, one {item} other {items}} |
| `@astryx.powersearch.valueEditor.entitiesCount` | `soft.powersearch.value_editor.entities_count` | {count, number} {count, plural, one {entity} other {entities}} |
| `@astryx.powersearch.valueEditor.dateRange` | `soft.powersearch.value_editor.date_range` | date range |
| `@astryx.powersearch.valueEditor.filtersCount` | `soft.powersearch.value_editor.filters_count` | {count, number} {count, plural, one {filter} other {filters}} |
| `@astryx.powersearch.resultCount` | `soft.powersearch.result_count` | {count, number} {count, plural, one {result} other {results}} |
| `@astryx.powersearch.label` | `soft.powersearch.label` | Search |
| `@astryx.powersearch.placeholder` | `soft.powersearch.placeholder` | Search… |
| `@astryx.powersearch.allFiltersRemoved` | `soft.powersearch.all_filters_removed` | All filters removed |

**alertDialog** (used in: AlertDialog)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.alertDialog.cancel` | `soft.alert_dialog.cancel` | Cancel |

**appShell** (used in: AppShell)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.appShell.mobileNavigation` | `soft.app_shell.mobile_navigation` | Mobile navigation |
| `@astryx.appShell.skipToContent` | `soft.app_shell.skip_to_content` | Skip to content |

**avatar** (used in: Avatar)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.avatar.nameWithStatus` | `soft.avatar.name_with_status` | {name}, {status} |

**avatarGroup** (used in: AvatarGroup)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.avatarGroup.label` | `soft.avatar_group.label` | Avatars |
| `@astryx.avatarGroup.keyboardHint` | `soft.avatar_group.keyboard_hint` | Use arrow keys to move between avatars |
| `@astryx.avatarGroup.overflow` | `soft.avatar_group.overflow` | {count, number} more |

**banner** (used in: Banner)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.banner.dismiss` | `soft.banner.dismiss` | Dismiss |
| `@astryx.banner.dismissTitled` | `soft.banner.dismiss_titled` | {dismiss} {title} |
| `@astryx.banner.collapse` | `soft.banner.collapse` | Collapse |
| `@astryx.banner.expand` | `soft.banner.expand` | Expand |

**calendar** (used in: Calendar, DateInput, DateTimeInput)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.calendar.previousMonth` | `soft.calendar.previous_month` | Previous month |
| `@astryx.calendar.nextMonth` | `soft.calendar.next_month` | Next month |
| `@astryx.calendar.daySelected` | `soft.calendar.day_selected` | {date}, selected |
| `@astryx.calendar.dayRangeStart` | `soft.calendar.day_range_start` | {date}, range start |
| `@astryx.calendar.dayRangeEnd` | `soft.calendar.day_range_end` | {date}, range end |
| `@astryx.calendar.dayRangeStartAndEnd` | `soft.calendar.day_range_start_and_end` | {date}, range start and range end |
| `@astryx.calendar.dayInRange` | `soft.calendar.day_in_range` | {date}, in range |
| `@astryx.calendar.rangeStartAnnounce` | `soft.calendar.range_start_announce` | Start date {date}. Select an end date. |
| `@astryx.calendar.rangeCompleteAnnounce` | `soft.calendar.range_complete_announce` | Selected range: {start} to {end}. |
| `@astryx.calendar.rangeClearedAnnounce` | `soft.calendar.range_cleared_announce` | Cleared start date {date}. Select a start date. |

**carousel** (used in: Carousel)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.carousel.label` | `soft.carousel.label` | Carousel |
| `@astryx.carousel.scrollLeft` | `soft.carousel.scroll_left` | Scroll left |
| `@astryx.carousel.scrollRight` | `soft.carousel.scroll_right` | Scroll right |
| `@astryx.carousel.slideLabel` | `soft.carousel.slide_label` | Slide {current, number} of {total, number} |

**chart** (used in: not used by core components)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.chart.label` | `soft.chart.label` | Chart |
| `@astryx.chart.labelWithSeries` | `soft.chart.label_with_series` | Chart of {series} by {xKey} |
| `@astryx.chart.dataTableCaption` | `soft.chart.data_table_caption` | {label} data |

**chartLegend** (used in: not used by core components)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.chartLegend.label` | `soft.chart_legend.label` | Chart legend |

**chat** (used in: Chat)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.chat.status.sending` | `soft.chat.status.sending` | Sending |
| `@astryx.chat.status.sent` | `soft.chat.status.sent` | Sent |
| `@astryx.chat.status.delivered` | `soft.chat.status.delivered` | Delivered |
| `@astryx.chat.status.read` | `soft.chat.status.read` | Read |
| `@astryx.chat.status.failed` | `soft.chat.status.failed` | Failed |
| `@astryx.chat.messageAriaLabel` | `soft.chat.message_aria_label` | Message {status} |
| `@astryx.chat.pastedText.expand` | `soft.chat.pasted_text.expand` | Expand |
| `@astryx.chat.pastedText.label` | `soft.chat.pasted_text.label` | {lines, plural, =1 {{chars, plural, other {{chars} chars}}} other {{lines} lines, {chars, plural, other {{chars} chars}}}} |
| `@astryx.chat.composer.placeholder` | `soft.chat.composer.placeholder` | Type a message… |
| `@astryx.chat.composerDrawer.label` | `soft.chat.composer_drawer.label` | Items |
| `@astryx.chat.composerInput.label` | `soft.chat.composer_input.label` | Message input |
| `@astryx.chat.speechRecognition.noSpeechDetected` | `soft.chat.speech_recognition.no_speech_detected` | No speech was detected. |

**checkboxList** (used in: CheckboxList)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.checkboxList.item.checkbox` | `soft.checkbox_list.item.checkbox` | Checkbox |

**commandPalette** (used in: CommandPalette)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.commandPalette.emptySearch` | `soft.command_palette.empty_search` | No results |
| `@astryx.commandPalette.emptyBootstrap` | `soft.command_palette.empty_bootstrap` | Type to search |
| `@astryx.commandPalette.resultCount` | `soft.command_palette.result_count` | {count, number} {count, plural, one {result} other {results}} |
| `@astryx.commandPalette.noResultsFor` | `soft.command_palette.no_results_for` | No results for {query} |
| `@astryx.commandPalette.loading` | `soft.command_palette.loading` | Loading |
| `@astryx.commandPalette.label` | `soft.command_palette.label` | Command palette |
| `@astryx.commandPalette.input.placeholder` | `soft.command_palette.input.placeholder` | Search… |
| `@astryx.commandPalette.input.label` | `soft.command_palette.input.label` | Search commands |
| `@astryx.commandPalette.list.label` | `soft.command_palette.list.label` | Commands |
| `@astryx.commandPalette.footer.navigate` | `soft.command_palette.footer.navigate` | Navigate |
| `@astryx.commandPalette.footer.select` | `soft.command_palette.footer.select` | Select |
| `@astryx.commandPalette.footer.close` | `soft.command_palette.footer.close` | Close |

**dateRangeInput** (used in: DateRangeInput)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.dateRangeInput.presetDateRanges` | `soft.date_range_input.preset_date_ranges` | Preset date ranges |
| `@astryx.dateRangeInput.placeholder` | `soft.date_range_input.placeholder` | Select date range |
| `@astryx.dateRangeInput.dialogLabel` | `soft.date_range_input.dialog_label` | Choose date range |

**dateTimeInput** (used in: DateTimeInput, TimeInput)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.dateTimeInput.timePlaceholder` | `soft.date_time_input.time_placeholder` | Select a time |
| `@astryx.dateTimeInput.timeHint12h` | `soft.date_time_input.time_hint12h` | e.g., 2:30 PM |
| `@astryx.dateTimeInput.timeHint24h` | `soft.date_time_input.time_hint24h` | e.g., 14:30 |
| `@astryx.dateTimeInput.timeSuffix` | `soft.date_time_input.time_suffix` | {label} time |
| `@astryx.dateTimeInput.timeOptionsLabel` | `soft.date_time_input.time_options_label` | {label} options |
| `@astryx.dateTimeInput.placeholder` | `soft.date_time_input.placeholder` | Select a date |
| `@astryx.dateTimeInput.dialogLabel` | `soft.date_time_input.dialog_label` | Choose date and time |
| `@astryx.dateTimeInput.pickerMode` | `soft.date_time_input.picker_mode` | Date/time section |
| `@astryx.dateTimeInput.dateTab` | `soft.date_time_input.date_tab` | Date |
| `@astryx.dateTimeInput.timeTab` | `soft.date_time_input.time_tab` | Time |
| `@astryx.dateTimeInput.openTimePicker` | `soft.date_time_input.open_time_picker` | Open {label} |
| `@astryx.dateTimeInput.saveDatePicking` | `soft.date_time_input.save_date_picking` | Save date |
| `@astryx.dateTimeInput.hourWheel` | `soft.date_time_input.hour_wheel` | Hour |
| `@astryx.dateTimeInput.minuteWheel` | `soft.date_time_input.minute_wheel` | Minute |
| `@astryx.dateTimeInput.secondWheel` | `soft.date_time_input.second_wheel` | Second |
| `@astryx.dateTimeInput.meridiemWheel` | `soft.date_time_input.meridiem_wheel` | AM/PM |
| `@astryx.dateTimeInput.meridiemAM` | `soft.date_time_input.meridiem_am` | AM |
| `@astryx.dateTimeInput.meridiemPM` | `soft.date_time_input.meridiem_pm` | PM |

**dialog** (used in: Dialog, Drawer)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.dialog.close` | `soft.dialog.close` | Close |

**dropdownMenu** (used in: ContextMenu, DropdownMenu)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.dropdownMenu.label` | `soft.dropdown_menu.label` | Menu |
| `@astryx.dropdownMenu.back` | `soft.dropdown_menu.back` | Back |
| `@astryx.dropdownMenu.backTo` | `soft.dropdown_menu.back_to` | Back to {parent} |

**lightbox** (used in: Lightbox)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.lightbox.close` | `soft.lightbox.close` | Close |
| `@astryx.lightbox.previous` | `soft.lightbox.previous` | Previous |
| `@astryx.lightbox.next` | `soft.lightbox.next` | Next |
| `@astryx.lightbox.mediaViewer` | `soft.lightbox.media_viewer` | Media viewer |
| `@astryx.lightbox.zoom` | `soft.lightbox.zoom` | Zoom |
| `@astryx.lightbox.zoomedIn` | `soft.lightbox.zoomed_in` | Zoomed in, use arrow keys to pan |
| `@astryx.lightbox.zoomedOut` | `soft.lightbox.zoomed_out` | Zoomed out |
| `@astryx.lightbox.imagePosition` | `soft.lightbox.image_position` | Image {index, number} of {total, number} |
| `@astryx.lightbox.mediaPosition` | `soft.lightbox.media_position` | {alt}, {index, number} of {total, number} |

**chatTypingIndicator** (used in: not used by core components)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.chatTypingIndicator.one` | `soft.chat_typing_indicator.one` | {name} is typing… |
| `@astryx.chatTypingIndicator.many` | `soft.chat_typing_indicator.many` | {names} are typing… |
| `@astryx.chatTypingIndicator.others` | `soft.chat_typing_indicator.others` | {count, number} {count, plural, one {other} other {others}} |

**listInput** (used in: not used by core components)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.listInput.emptyTitle` | `soft.list_input.empty_title` | No {itemName}s yet |
| `@astryx.listInput.emptyDescription` | `soft.list_input.empty_description` | Add a {itemName} to get started. |
| `@astryx.listInput.addItem` | `soft.list_input.add_item` | Add {itemName} |
| `@astryx.listInput.removeItem` | `soft.list_input.remove_item` | Remove {itemName} {position, number} |
| `@astryx.listInput.removeUnavailable` | `soft.list_input.remove_unavailable` | Remove is unavailable while the list is disabled |
| `@astryx.listInput.reorderItem` | `soft.list_input.reorder_item` | Reorder {itemName} {position, number} |
| `@astryx.listInput.fieldLabelWithPosition` | `soft.list_input.field_label_with_position` | {header}, {itemName} {position, number} of {total, number} |
| `@astryx.listInput.reorderInstructions` | `soft.list_input.reorder_instructions` | Use Arrow Up or Arrow Down to move this item one position. Press Space or Enter to pick it up for extended keyboard reordering. |
| `@astryx.listInput.announceAdded` | `soft.list_input.announce_added` | Added {itemName} {position, number}. |
| `@astryx.listInput.announceRemoved` | `soft.list_input.announce_removed` | Removed {itemName} {position, number}. |
| `@astryx.listInput.announceGrabbed` | `soft.list_input.announce_grabbed` | {itemName} {position, number} grabbed. Use arrow keys to move, Space or Enter to drop, and Escape to cancel. |
| `@astryx.listInput.announceMovedToPosition` | `soft.list_input.announce_moved_to_position` | {itemName} moved to position {position, number} of {total, number}. |
| `@astryx.listInput.announceReorderCancelled` | `soft.list_input.announce_reorder_cancelled` | Reordering cancelled. |
| `@astryx.listInput.announceReturnedToPosition` | `soft.list_input.announce_returned_to_position` | {itemName} returned to position {position, number}. |
| `@astryx.listInput.announceDropped` | `soft.list_input.announce_dropped` | {itemName} dropped at position {position, number} of {total, number}. |
| `@astryx.listInput.announceAlreadyAtBoundary` | `soft.list_input.announce_already_at_boundary` | This {itemName} is already {boundary, select, first {first} last {last} other {}}. |

**diffViewer** (used in: not used by core components)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.diffViewer.label` | `soft.diff_viewer.label` | Diff |
| `@astryx.diffViewer.renamedPath` | `soft.diff_viewer.renamed_path` | {oldPath} → {newPath} |
| `@astryx.diffViewer.statusAdded` | `soft.diff_viewer.status_added` | Added |
| `@astryx.diffViewer.statusDeleted` | `soft.diff_viewer.status_deleted` | Deleted |
| `@astryx.diffViewer.statusRenamed` | `soft.diff_viewer.status_renamed` | Renamed |
| `@astryx.diffViewer.addedCount` | `soft.diff_viewer.added_count` | +{count, number} |
| `@astryx.diffViewer.removedCount` | `soft.diff_viewer.removed_count` | −{count, number} |
| `@astryx.diffViewer.stats` | `soft.diff_viewer.stats` | {added, plural, one {# line added} other {# lines added}}, {removed, plural, one {# line removed} other {# lines removed}} |
| `@astryx.diffViewer.added` | `soft.diff_viewer.added` | Added |
| `@astryx.diffViewer.removed` | `soft.diff_viewer.removed` | Removed |
| `@astryx.diffViewer.lineNumberButton` | `soft.diff_viewer.line_number_button` | {side, select, old {Original line {line, number}} other {Changed line {line, number}}} |
| `@astryx.diffViewer.columnOldLine` | `soft.diff_viewer.column_old_line` | Original line number |
| `@astryx.diffViewer.columnNewLine` | `soft.diff_viewer.column_new_line` | Changed line number |
| `@astryx.diffViewer.columnChange` | `soft.diff_viewer.column_change` | Change |
| `@astryx.diffViewer.columnCode` | `soft.diff_viewer.column_code` | Code |
| `@astryx.diffViewer.columnOldCode` | `soft.diff_viewer.column_old_code` | Original code |
| `@astryx.diffViewer.columnNewCode` | `soft.diff_viewer.column_new_code` | Changed code |
| `@astryx.diffViewer.binary` | `soft.diff_viewer.binary` | Binary file not shown |
| `@astryx.diffViewer.noChanges` | `soft.diff_viewer.no_changes` | No content changes |
| `@astryx.diffViewer.largeChange` | `soft.diff_viewer.large_change` | Large change: {count, plural, one {# line} other {# lines}} |
| `@astryx.diffViewer.showFullDiff` | `soft.diff_viewer.show_full_diff` | Show full diff |
| `@astryx.diffViewer.hideFullDiff` | `soft.diff_viewer.hide_full_diff` | Hide full diff |

**markdownHeadingLinks** (used in: Markdown)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.markdownHeadingLinks.copied` | `soft.markdown_heading_links.copied` | Link copied |
| `@astryx.markdownHeadingLinks.copy` | `soft.markdown_heading_links.copy` | Copy link to {heading} |

**richTextEditor** (used in: not used by core components)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.richTextEditor.tabEscapeHint` | `soft.rich_text_editor.tab_escape_hint` | Press Escape then Tab to move focus out of the editor. |
| `@astryx.richTextEditor.toolbar.label` | `soft.rich_text_editor.toolbar.label` | Text formatting |
| `@astryx.richTextEditor.toolbar.actions` | `soft.rich_text_editor.toolbar.actions` | Formatting actions |
| `@astryx.richTextEditor.toolbar.undo` | `soft.rich_text_editor.toolbar.undo` | Undo |
| `@astryx.richTextEditor.toolbar.redo` | `soft.rich_text_editor.toolbar.redo` | Redo |
| `@astryx.richTextEditor.toolbar.historyDivider` | `soft.rich_text_editor.toolbar.history_divider` | History and block formats |
| `@astryx.richTextEditor.toolbar.blockFormat` | `soft.rich_text_editor.toolbar.block_format` | Block format |
| `@astryx.richTextEditor.toolbar.formatDivider` | `soft.rich_text_editor.toolbar.format_divider` | Block and inline formats |
| `@astryx.richTextEditor.toolbar.paragraph` | `soft.rich_text_editor.toolbar.paragraph` | Paragraph |
| `@astryx.richTextEditor.toolbar.heading1` | `soft.rich_text_editor.toolbar.heading1` | Heading 1 |
| `@astryx.richTextEditor.toolbar.heading2` | `soft.rich_text_editor.toolbar.heading2` | Heading 2 |
| `@astryx.richTextEditor.toolbar.heading3` | `soft.rich_text_editor.toolbar.heading3` | Heading 3 |
| `@astryx.richTextEditor.toolbar.bulletedList` | `soft.rich_text_editor.toolbar.bulleted_list` | Bulleted list |
| `@astryx.richTextEditor.toolbar.numberedList` | `soft.rich_text_editor.toolbar.numbered_list` | Numbered list |
| `@astryx.richTextEditor.toolbar.blockQuote` | `soft.rich_text_editor.toolbar.block_quote` | Block quote |
| `@astryx.richTextEditor.toolbar.bold` | `soft.rich_text_editor.toolbar.bold` | Bold |
| `@astryx.richTextEditor.toolbar.italic` | `soft.rich_text_editor.toolbar.italic` | Italic |
| `@astryx.richTextEditor.toolbar.underline` | `soft.rich_text_editor.toolbar.underline` | Underline |
| `@astryx.richTextEditor.toolbar.strikethrough` | `soft.rich_text_editor.toolbar.strikethrough` | Strikethrough |
| `@astryx.richTextEditor.toolbar.inlineCode` | `soft.rich_text_editor.toolbar.inline_code` | Inline code |
| `@astryx.richTextEditor.toolbar.link` | `soft.rich_text_editor.toolbar.link` | Link |
| `@astryx.richTextEditor.link.insertTitle` | `soft.rich_text_editor.link.insert_title` | Insert link |
| `@astryx.richTextEditor.link.editTitle` | `soft.rich_text_editor.link.edit_title` | Edit link |
| `@astryx.richTextEditor.link.url` | `soft.rich_text_editor.link.url` | URL |
| `@astryx.richTextEditor.link.invalidUrl` | `soft.rich_text_editor.link.invalid_url` | Enter a valid http, https, mailto, or tel URL. |
| `@astryx.richTextEditor.link.remove` | `soft.rich_text_editor.link.remove` | Remove link |
| `@astryx.richTextEditor.link.cancel` | `soft.rich_text_editor.link.cancel` | Cancel |
| `@astryx.richTextEditor.link.add` | `soft.rich_text_editor.link.add` | Add link |
| `@astryx.richTextEditor.link.update` | `soft.rich_text_editor.link.update` | Update link |

**markdown** (used in: Markdown)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.markdown.taskList` | `soft.markdown.task_list` | Task list |
| `@astryx.markdown.table` | `soft.markdown.table` | Table |

**mobileNav** (used in: MobileNav)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.mobileNav.closeNavigation` | `soft.mobile_nav.close_navigation` | Close navigation |
| `@astryx.mobileNav.toggle.open` | `soft.mobile_nav.toggle.open` | Open navigation |
| `@astryx.mobileNav.navigation` | `soft.mobile_nav.navigation` | Navigation |

**multiSelector** (used in: MultiSelector)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.multiSelector.selectAll` | `soft.multi_selector.select_all` | Select all |
| `@astryx.multiSelector.searchPlaceholder` | `soft.multi_selector.search_placeholder` | Search… |
| `@astryx.multiSelector.searchOptions` | `soft.multi_selector.search_options` | Search options |
| `@astryx.multiSelector.empty` | `soft.multi_selector.empty` | No options |
| `@astryx.multiSelector.selectAllPartiallySelected` | `soft.multi_selector.select_all_partially_selected` | {label}, partially selected |
| `@astryx.multiSelector.selectPlaceholder` | `soft.multi_selector.select_placeholder` | Select… |
| `@astryx.multiSelector.clearAll` | `soft.multi_selector.clear_all` | Clear all {label} |
| `@astryx.multiSelector.selectionCleared` | `soft.multi_selector.selection_cleared` | Selection cleared |
| `@astryx.multiSelector.allSelected` | `soft.multi_selector.all_selected` | All selected |
| `@astryx.multiSelector.selectionCount` | `soft.multi_selector.selection_count` | {count, number} of {total, number} selected |
| `@astryx.multiSelector.selectedCount` | `soft.multi_selector.selected_count` | {count, plural, one {# selected} other {# selected}} |
| `@astryx.multiSelector.createOption` | `soft.multi_selector.create_option` | Create "{query}" |
| `@astryx.multiSelector.optionCreated` | `soft.multi_selector.option_created` | {label} created |
| `@astryx.multiSelector.emptySearchResults` | `soft.multi_selector.empty_search_results` | No results found |
| `@astryx.multiSelector.resultCount` | `soft.multi_selector.result_count` | {count, number} {count, plural, one {result} other {results}} |

**popover** (used in: Popover)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.popover.close` | `soft.popover.close` | Close popover |

**selector** (used in: ComplexSelector, Selector)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.selector.searchPlaceholder` | `soft.selector.search_placeholder` | Search… |
| `@astryx.selector.searchOptions` | `soft.selector.search_options` | Search options |
| `@astryx.selector.empty` | `soft.selector.empty` | No options |
| `@astryx.selector.placeholder` | `soft.selector.placeholder` | Select… |
| `@astryx.selector.clearLabel` | `soft.selector.clear_label` | Clear {label} |
| `@astryx.selector.emptySearchResults` | `soft.selector.empty_search_results` | No results found |
| `@astryx.selector.resultCount` | `soft.selector.result_count` | {count, number} {count, plural, one {result} other {results}} |

**sideNav** (used in: SideNav)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.sideNav.label` | `soft.side_nav.label` | Side navigation |
| `@astryx.sideNav.resizeSidebar` | `soft.side_nav.resize_sidebar` | Resize sidebar |
| `@astryx.sideNav.heading.openMenu` | `soft.side_nav.heading.open_menu` | {heading}, Open menu |
| `@astryx.sideNav.heading.dialogLabel` | `soft.side_nav.heading.dialog_label` | Navigation menu |

**slider** (used in: Slider)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.slider.minimumThumb` | `soft.slider.minimum_thumb` | Minimum value |
| `@astryx.slider.maximumThumb` | `soft.slider.maximum_thumb` | Maximum value |

**tabList** (used in: TabList)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.tabList.label` | `soft.tab_list.label` | Tabs |

**table** (used in: Table)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.table.label` | `soft.table.label` | Table |
| `@astryx.table.noData` | `soft.table.no_data` | No data |
| `@astryx.table.columnResize.resizeColumn` | `soft.table.column_resize.resize_column` | Resize column {label} |
| `@astryx.table.filter.allPlaceholder` | `soft.table.filter.all_placeholder` | All |
| `@astryx.table.filter.reset` | `soft.table.filter.reset` | Reset |
| `@astryx.table.filter.apply` | `soft.table.filter.apply` | Apply |
| `@astryx.table.rowStatus.columnHeader` | `soft.table.row_status.column_header` | Row status |
| `@astryx.table.selection.bulkActionsLabel` | `soft.table.selection.bulk_actions_label` | Bulk actions |
| `@astryx.table.selection.clearAll` | `soft.table.selection.clear_all` | Unselect All |
| `@astryx.table.selection.selectedCount` | `soft.table.selection.selected_count` | {count, number} selected |
| `@astryx.table.selection.selectAllRows` | `soft.table.selection.select_all_rows` | Select all rows |
| `@astryx.table.selection.selectRow` | `soft.table.selection.select_row` | Select row |
| `@astryx.table.selection.selectRowNamed` | `soft.table.selection.select_row_named` | Select {label} |
| `@astryx.table.sort.ascending` | `soft.table.sort.ascending` | Sort ascending |
| `@astryx.table.sort.descending` | `soft.table.sort.descending` | Sort descending |
| `@astryx.table.sort.clear` | `soft.table.sort.clear` | Clear sort |
| `@astryx.table.sort.priority` | `soft.table.sort.priority` | Priority {rank, number} of {total, number} |
| `@astryx.table.pagination.label` | `soft.table.pagination.label` | Table pagination |
| `@astryx.table.pagination.labelAbove` | `soft.table.pagination.label_above` | {label} (top) |
| `@astryx.table.pagination.labelBelow` | `soft.table.pagination.label_below` | {label} (bottom) |

**toast** (used in: Toast)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.toast.dismiss` | `soft.toast.dismiss` | Dismiss notification |
| `@astryx.toast.viewport` | `soft.toast.viewport` | Notifications |

**tokenizer** (used in: Tokenizer)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.tokenizer.clearAll` | `soft.tokenizer.clear_all` | Clear all |
| `@astryx.tokenizer.tokenAdded` | `soft.tokenizer.token_added` | Added {label} |
| `@astryx.tokenizer.tokenRemoved` | `soft.tokenizer.token_removed` | Removed {label} |
| `@astryx.tokenizer.search` | `soft.tokenizer.search` | Search {label} |
| `@astryx.tokenizer.searchPlaceholder` | `soft.tokenizer.search_placeholder` | Search… |
| `@astryx.tokenizer.noResultsDescription` | `soft.tokenizer.no_results_description` | Try a different search. |
| `@astryx.tokenizer.create` | `soft.tokenizer.create` | Create "{value}" |
| `@astryx.tokenizer.add` | `soft.tokenizer.add` | Add |
| `@astryx.tokenizer.addValue` | `soft.tokenizer.add_value` | Add {value} |
| `@astryx.tokenizer.done` | `soft.tokenizer.done` | Done |
| `@astryx.tokenizer.save` | `soft.tokenizer.save` | Save |
| `@astryx.tokenizer.selectionLimit` | `soft.tokenizer.selection_limit` | Select up to {count, plural, one {# item} other {# items}} |
| `@astryx.tokenizer.discardChangesTitle` | `soft.tokenizer.discard_changes_title` | Discard changes? |
| `@astryx.tokenizer.discardChangesDescription` | `soft.tokenizer.discard_changes_description` | Your changes to {label} haven't been saved. |
| `@astryx.tokenizer.keepEditing` | `soft.tokenizer.keep_editing` | Keep editing |
| `@astryx.tokenizer.discardChanges` | `soft.tokenizer.discard_changes` | Discard changes |
| `@astryx.tokenizer.clearAllTitle` | `soft.tokenizer.clear_all_title` | Clear all selected items? |
| `@astryx.tokenizer.clearAllDescription` | `soft.tokenizer.clear_all_description` | This removes all selected items from {label}. |
| `@astryx.tokenizer.nothingToClear` | `soft.tokenizer.nothing_to_clear` | Nothing selected |
| `@astryx.tokenizer.allRemoved` | `soft.tokenizer.all_removed` | All items removed |
| `@astryx.tokenizer.selectedCount` | `soft.tokenizer.selected_count` | {count, number} selected |
| `@astryx.tokenizer.clearSelection` | `soft.tokenizer.clear_selection` | Clear selection |

**topNav** (used in: TopNav)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.topNav.heading.openMenu` | `soft.top_nav.heading.open_menu` | {hasHeading, select, true {{heading}, } other {}}Open menu |
| `@astryx.topNav.landmarkLabel` | `soft.top_nav.landmark_label` | Top navigation |
| `@astryx.topNav.heading.dialogLabel` | `soft.top_nav.heading.dialog_label` | Navigation menu |

**treeList** (used in: TreeList)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.treeList.toggleChildren` | `soft.tree_list.toggle_children` | Toggle children |

**typeahead** (used in: Tokenizer, Typeahead)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.typeahead.emptySearchResults` | `soft.typeahead.empty_search_results` | No results found |
| `@astryx.typeahead.loading` | `soft.typeahead.loading` | Loading |
| `@astryx.typeahead.searchResults` | `soft.typeahead.search_results` | Search results |
| `@astryx.typeahead.clearSelection` | `soft.typeahead.clear_selection` | Clear selection |
| `@astryx.typeahead.searchPlaceholder` | `soft.typeahead.search_placeholder` | Search… |
| `@astryx.typeahead.resultCount` | `soft.typeahead.result_count` | {count, number} {count, plural, one {result} other {results}} |

**breadcrumbs** (used in: Breadcrumbs)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.breadcrumbs.label` | `soft.breadcrumbs.label` | Breadcrumb |

**contextMenu** (used in: ContextMenu)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.contextMenu.label` | `soft.context_menu.label` | Context menu |

**dateInput** (used in: DateInput, DateRangeInput, DateTimeInput, TimeInput)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.dateInput.placeholder` | `soft.date_input.placeholder` | Select a date |
| `@astryx.dateInput.dialogLabel` | `soft.date_input.dialog_label` | Choose date |
| `@astryx.dateInput.closeCalendar` | `soft.date_input.close_calendar` | Close calendar |
| `@astryx.dateInput.openCalendar` | `soft.date_input.open_calendar` | Open calendar |
| `@astryx.dateInput.toggleCalendarClose` | `soft.date_input.toggle_calendar_close` | Close calendar |
| `@astryx.dateInput.clear` | `soft.date_input.clear` | Clear {label} |
| `@astryx.dateInput.invalidDate` | `soft.date_input.invalid_date` | Invalid date |
| `@astryx.dateInput.resetPicking` | `soft.date_input.reset_picking` | Reset |
| `@astryx.dateInput.savePicking` | `soft.date_input.save_picking` | Save |
| `@astryx.dateInput.doneChoosingMonth` | `soft.date_input.done_choosing_month` | Done |
| `@astryx.dateInput.chooseMonthYear` | `soft.date_input.choose_month_year` | {monthYear}, Choose month and year |
| `@astryx.dateInput.monthWheel` | `soft.date_input.month_wheel` | Month |
| `@astryx.dateInput.yearWheel` | `soft.date_input.year_wheel` | Year |

**link** (used in: Link)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.link.newTab` | `soft.link.new_tab` | (opens in new tab) |

**moreMenu** (used in: MoreMenu)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.moreMenu.label` | `soft.more_menu.label` | More options |

**outline** (used in: Outline)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.outline.label` | `soft.outline.label` | Table of contents |

**resizable** (used in: Resizable)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.resizable.collapsed` | `soft.resizable.collapsed` | Collapsed |
| `@astryx.resizable.handle.label` | `soft.resizable.handle.label` | Resize handle |

**timeInput** (used in: DateTimeInput, TimeInput)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.timeInput.placeholder` | `soft.time_input.placeholder` | Select a time |
| `@astryx.timeInput.invalidTime` | `soft.time_input.invalid_time` | Invalid time |
| `@astryx.timeInput.openPicker` | `soft.time_input.open_picker` | Open {label} |
| `@astryx.timeInput.clearLabel` | `soft.time_input.clear_label` | Clear {label} |

**button** (used in: Button)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.button.loading` | `soft.button.loading` | Loading |

**spinner** (used in: Chat, CheckboxInput, ComplexSelector, DateInput, DateRangeInput, DateTimeInput, DropdownMenu, FileInput, MultiSelector, Selector, Spinner, TextArea, TextInput, Thumbnail, TimeInput)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.spinner.loading` | `soft.spinner.loading` | Loading |

**chatComposerDrawer** (used in: Chat)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.chatComposerDrawer.expand` | `soft.chat_composer_drawer.expand` | Expand {label} |
| `@astryx.chatComposerDrawer.collapse` | `soft.chat_composer_drawer.collapse` | Collapse {label} |

**chatDictationButton** (used in: Chat)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.chatDictationButton.startDictation` | `soft.chat_dictation_button.start_dictation` | Start dictation |
| `@astryx.chatDictationButton.stopDictation` | `soft.chat_dictation_button.stop_dictation` | Stop dictation |

**chatLayout** (used in: Chat)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.chatLayout.newMessages` | `soft.chat_layout.new_messages` | New messages |

**chatLayoutScrollButton** (used in: Chat)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.chatLayoutScrollButton.scrollToBottom` | `soft.chat_layout_scroll_button.scroll_to_bottom` | Scroll to bottom |

**chatMessage** (used in: Chat)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.chatMessage.messageFrom` | `soft.chat_message.message_from` | Message from {sender} |

**chatSendButton** (used in: Chat)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.chatSendButton.stop` | `soft.chat_send_button.stop` | Stop |
| `@astryx.chatSendButton.send` | `soft.chat_send_button.send` | Send |

**chatToolCalls** (used in: Chat)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.chatToolCalls.error` | `soft.chat_tool_calls.error` | Error: {message} |
| `@astryx.chatToolCalls.status.pending` | `soft.chat_tool_calls.status.pending` | Pending |
| `@astryx.chatToolCalls.status.running` | `soft.chat_tool_calls.status.running` | Running |
| `@astryx.chatToolCalls.status.complete` | `soft.chat_tool_calls.status.complete` | Complete |
| `@astryx.chatToolCalls.status.error` | `soft.chat_tool_calls.status.error` | Failed |
| `@astryx.chatToolCalls.groupLabel` | `soft.chat_tool_calls.group_label` | {count} tool calls |

**chatTriggerMenu** (used in: Chat)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.chatTriggerMenu.searching` | `soft.chat_trigger_menu.searching` | Searching… |
| `@astryx.chatTriggerMenu.suggestions` | `soft.chat_trigger_menu.suggestions` | Suggestions |

**citation** (used in: Citation)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.citation.label` | `soft.citation.label` | Citation {number}: {title} |

**codeBlock** (used in: CodeBlock)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.codeBlock.copied` | `soft.code_block.copied` | Copied |
| `@astryx.codeBlock.copyCode` | `soft.code_block.copy_code` | Copy code |
| `@astryx.codeBlock.code` | `soft.code_block.code` | Code |

**field** (used in: DateRangeInput, Field)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.field.optional` | `soft.field.optional` | Optional |
| `@astryx.field.required` | `soft.field.required` | Required |

**fileInput** (used in: FileInput)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.fileInput.clearLabel` | `soft.file_input.clear_label` | Clear {label} |
| `@astryx.fileInput.dropHint` | `soft.file_input.drop_hint` | Drop files here |
| `@astryx.fileInput.errorInvalidType` | `soft.file_input.error_invalid_type` | "{fileName}" is not an accepted file type |
| `@astryx.fileInput.errorMaxFiles` | `soft.file_input.error_max_files` | Maximum {maxFiles} files allowed |
| `@astryx.fileInput.errorMaxSize` | `soft.file_input.error_max_size` | "{fileName}" exceeds {maxSize} limit |
| `@astryx.fileInput.fileSelected` | `soft.file_input.file_selected` | 1 file selected: {fileName} |
| `@astryx.fileInput.fileSize.bytes` | `soft.file_input.file_size.bytes` | {size, number, ::group-off} B |
| `@astryx.fileInput.fileSize.kilobytes` | `soft.file_input.file_size.kilobytes` | {size, number, :: .0 group-off} KB |
| `@astryx.fileInput.fileSize.megabytes` | `soft.file_input.file_size.megabytes` | {size, number, :: .0 group-off} MB |
| `@astryx.fileInput.filesSelected` | `soft.file_input.files_selected` | {count} files selected |
| `@astryx.fileInput.placeholder` | `soft.file_input.placeholder` | Choose file |
| `@astryx.fileInput.placeholderMultiple` | `soft.file_input.placeholder_multiple` | Choose files |
| `@astryx.fileInput.required` | `soft.file_input.required` | Required |
| `@astryx.fileInput.triggerWithFiles` | `soft.file_input.trigger_with_files` | {label}, {fileNames} |

**kbd** (used in: Kbd)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.kbd.key.alt` | `soft.kbd.key.alt` | Alt |
| `@astryx.kbd.key.backspace` | `soft.kbd.key.backspace` | Backspace |
| `@astryx.kbd.key.command` | `soft.kbd.key.command` | Command |
| `@astryx.kbd.key.control` | `soft.kbd.key.control` | Control |
| `@astryx.kbd.key.controlShort` | `soft.kbd.key.control_short` | Ctrl |
| `@astryx.kbd.key.down` | `soft.kbd.key.down` | Down arrow |
| `@astryx.kbd.key.enter` | `soft.kbd.key.enter` | Enter |
| `@astryx.kbd.key.escape` | `soft.kbd.key.escape` | Escape |
| `@astryx.kbd.key.escapeShort` | `soft.kbd.key.escape_short` | Esc |
| `@astryx.kbd.key.left` | `soft.kbd.key.left` | Left arrow |
| `@astryx.kbd.key.plus` | `soft.kbd.key.plus` | Plus |
| `@astryx.kbd.key.right` | `soft.kbd.key.right` | Right arrow |
| `@astryx.kbd.key.shift` | `soft.kbd.key.shift` | Shift |
| `@astryx.kbd.key.tab` | `soft.kbd.key.tab` | Tab |
| `@astryx.kbd.key.up` | `soft.kbd.key.up` | Up arrow |
| `@astryx.kbd.shortcut` | `soft.kbd.shortcut` | {shortcut} + {key} |

**keyboardHint** (used in: hooks)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.keyboardHint.toNavigate` | `soft.keyboard_hint.to_navigate` | to navigate |

**metadataList** (used in: MetadataList)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.metadataList.showMore` | `soft.metadata_list.show_more` | Show more |
| `@astryx.metadataList.showLess` | `soft.metadata_list.show_less` | Show less |

**input** (used in: MultiSelector, Selector, hooks)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.input.readOnly` | `soft.input.read_only` | Read only |
| `@astryx.input.statusButton.error` | `soft.input.status_button.error` | Error details |
| `@astryx.input.statusButton.warning` | `soft.input.status_button.warning` | Warning details |
| `@astryx.input.statusButton.success` | `soft.input.status_button.success` | Success details |

**numberInput** (used in: NumberInput)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.numberInput.clearLabel` | `soft.number_input.clear_label` | Clear {label} |
| `@astryx.numberInput.decrementLabel` | `soft.number_input.decrement_label` | Decrement {label} |
| `@astryx.numberInput.incrementLabel` | `soft.number_input.increment_label` | Increment {label} |
| `@astryx.numberInput.invalidNumber` | `soft.number_input.invalid_number` | Invalid number |

**sideNavCollapseButton** (used in: SideNav)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.sideNavCollapseButton.expandSidebar` | `soft.side_nav_collapse_button.expand_sidebar` | Expand sidebar |
| `@astryx.sideNavCollapseButton.collapseSidebar` | `soft.side_nav_collapse_button.collapse_sidebar` | Collapse sidebar |

**sideNavItem** (used in: SideNav)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.sideNavItem.expand` | `soft.side_nav_item.expand` | Expand {label} |
| `@astryx.sideNavItem.collapse` | `soft.side_nav_item.collapse` | Collapse {label} |
| `@astryx.sideNavItem.submenuLabel` | `soft.side_nav_item.submenu_label` | {label} submenu |

**tableFiltering** (used in: Table)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.tableFiltering.filterByColumn` | `soft.table_filtering.filter_by_column` | Filter {header} |

**tableGroupedRows** (used in: Table)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.tableGroupedRows.expandGroup` | `soft.table_grouped_rows.expand_group` | Expand group {groupKey} |
| `@astryx.tableGroupedRows.collapseGroup` | `soft.table_grouped_rows.collapse_group` | Collapse group {groupKey} |

**tableRowExpansion** (used in: Table)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.tableRowExpansion.collapseRow` | `soft.table_row_expansion.collapse_row` | Collapse row |
| `@astryx.tableRowExpansion.expandRow` | `soft.table_row_expansion.expand_row` | Expand row |
| `@astryx.tableRowExpansion.columnHeader` | `soft.table_row_expansion.column_header` | Row expansion |
| `@astryx.tableRowExpansion.collapseAllRows` | `soft.table_row_expansion.collapse_all_rows` | Collapse all rows |
| `@astryx.tableRowExpansion.expandAllRows` | `soft.table_row_expansion.expand_all_rows` | Expand all rows |

**stepper** (used in: Stepper)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.stepper.label` | `soft.stepper.label` | Progress |
| `@astryx.stepper.previousStep` | `soft.stepper.previous_step` | Previous step |
| `@astryx.stepper.nextStep` | `soft.stepper.next_step` | Next step |

**step** (used in: Stepper)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.step.goToStep` | `soft.step.go_to_step` | Go to step {stepNumber, number}: {label} |
| `@astryx.step.goToStepWithStatus` | `soft.step.go_to_step_with_status` | Go to step {stepNumber, number}: {label}, {status} |
| `@astryx.step.optional` | `soft.step.optional` | Optional |
| `@astryx.step.status.completed` | `soft.step.status.completed` | completed |
| `@astryx.step.status.warning` | `soft.step.status.warning` | warning |
| `@astryx.step.status.error` | `soft.step.status.error` | error |

**switch** (used in: Switch)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.switch.loading` | `soft.switch.loading` | Loading |

**tableTree** (used in: Table)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.tableTree.collapseRow` | `soft.table_tree.collapse_row` | Collapse row |
| `@astryx.tableTree.expandRow` | `soft.table_tree.expand_row` | Expand row |
| `@astryx.tableTree.collapseAllRows` | `soft.table_tree.collapse_all_rows` | Collapse all rows |
| `@astryx.tableTree.expandAllRows` | `soft.table_tree.expand_all_rows` | Expand all rows |

**textArea** (used in: TextArea)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.textArea.counter` | `soft.text_area.counter` | {count, number}/{limit, number} |
| `@astryx.textArea.charactersRemaining` | `soft.text_area.characters_remaining` | {count, number} {count, plural, one {character} other {characters}} remaining |
| `@astryx.textArea.charactersOverLimit` | `soft.text_area.characters_over_limit` | {count, number} {count, plural, one {character} other {characters}} over the limit |

**inputClearButton** (used in: Field)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.inputClearButton.tooltip` | `soft.input_clear_button.tooltip` | Clear |

**textInput** (used in: MultiSelector, Selector, TextInput, Tokenizer)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.textInput.clearLabel` | `soft.text_input.clear_label` | Clear {label} |

**thumbnail** (used in: Thumbnail)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.thumbnail.remove` | `soft.thumbnail.remove` | Remove {accessibleName} |
| `@astryx.thumbnail.open` | `soft.thumbnail.open` | Open {accessibleName} |
| `@astryx.thumbnail.fallbackName` | `soft.thumbnail.fallback_name` | Thumbnail |

**timestamp** (used in: Timestamp)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.timestamp.copyValue` | `soft.timestamp.copy_value` | Copy {value} |
| `@astryx.timestamp.copy` | `soft.timestamp.copy` | Copy |
| `@astryx.timestamp.copied` | `soft.timestamp.copied` | Copied |
| `@astryx.timestamp.detailsLabel` | `soft.timestamp.details_label` | Timestamp details |

**token** (used in: Token)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.token.remove` | `soft.token.remove` | Remove {label} |

**transferList** (used in: not used by core components)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.transferList.selectedLabel` | `soft.transfer_list.selected_label` | Selected |
| `@astryx.transferList.availableLabel` | `soft.transfer_list.available_label` | Available |
| `@astryx.transferList.ungroupedLabel` | `soft.transfer_list.ungrouped_label` | Other |
| `@astryx.transferList.searchPlaceholder` | `soft.transfer_list.search_placeholder` | Search… |
| `@astryx.transferList.searchLabel` | `soft.transfer_list.search_label` | Search {label} |
| `@astryx.transferList.selectedEmpty` | `soft.transfer_list.selected_empty` | No selected options |
| `@astryx.transferList.availableEmpty` | `soft.transfer_list.available_empty` | No available options |
| `@astryx.transferList.noResults` | `soft.transfer_list.no_results` | No results |
| `@astryx.transferList.clear` | `soft.transfer_list.clear` | Clear |
| `@astryx.transferList.addAll` | `soft.transfer_list.add_all` | Add all |
| `@astryx.transferList.addOption` | `soft.transfer_list.add_option` | Add {label} |
| `@astryx.transferList.removeOption` | `soft.transfer_list.remove_option` | Remove {label} |
| `@astryx.transferList.reorderOption` | `soft.transfer_list.reorder_option` | Reorder {label} |
| `@astryx.transferList.transferDisabled` | `soft.transfer_list.transfer_disabled` | {label} cannot be moved |
| `@astryx.transferList.reorderDisabled` | `soft.transfer_list.reorder_disabled` | {label} cannot be reordered |
| `@astryx.transferList.reorderInstructions` | `soft.transfer_list.reorder_instructions` | Press Space or Enter to pick up an item. Use Arrow Up, Arrow Down, Home, or End to move it. Press Space or Enter to drop, or Escape to cancel. |
| `@astryx.transferList.announceAdded` | `soft.transfer_list.announce_added` | {label} added. {count, number} {count, plural, one {item} other {items}} selected. |
| `@astryx.transferList.announceRemoved` | `soft.transfer_list.announce_removed` | {label} removed. {count, number} {count, plural, one {item} other {items}} selected. |
| `@astryx.transferList.announceBulkAdded` | `soft.transfer_list.announce_bulk_added` | {count, number} {count, plural, one {item} other {items}} added. |
| `@astryx.transferList.announceBulkRemoved` | `soft.transfer_list.announce_bulk_removed` | {count, number} {count, plural, one {item} other {items}} removed. |
| `@astryx.transferList.announceGrabbed` | `soft.transfer_list.announce_grabbed` | {label} picked up, position {position, number} of {total, number}. Use arrow keys to move, Space or Enter to drop, or Escape to cancel. |
| `@astryx.transferList.announceMoveCancelled` | `soft.transfer_list.announce_move_cancelled` | {label} move cancelled. |
| `@astryx.transferList.announceDropped` | `soft.transfer_list.announce_dropped` | {label} dropped at position {position, number} of {total, number}. |
| `@astryx.transferList.announceReturned` | `soft.transfer_list.announce_returned` | {label} returned to position {position, number}. |
| `@astryx.transferList.announceMovedToPosition` | `soft.transfer_list.announce_moved_to_position` | {label}, position {position, number} of {total, number}. |
| `@astryx.transferList.announceSearchResults` | `soft.transfer_list.announce_search_results` | {count, number} {count, plural, one {item} other {items}} found. |

**transferListSelector** (used in: not used by core components)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.transferListSelector.apply` | `soft.transfer_list_selector.apply` | Apply |
| `@astryx.transferListSelector.cancel` | `soft.transfer_list_selector.cancel` | Cancel |
| `@astryx.transferListSelector.triggerLabel` | `soft.transfer_list_selector.trigger_label` | {count, number} selected |

**emojiPicker** (used in: not used by core components)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.emojiPicker.label` | `soft.emoji_picker.label` | Emoji |
| `@astryx.emojiPicker.searchLabel` | `soft.emoji_picker.search_label` | Search emoji |
| `@astryx.emojiPicker.frequentlyUsed` | `soft.emoji_picker.frequently_used` | Frequently used |
| `@astryx.emojiPicker.searchResults` | `soft.emoji_picker.search_results` | Search results |
| `@astryx.emojiPicker.noResults` | `soft.emoji_picker.no_results` | No emoji match “{query}” |
| `@astryx.emojiPicker.loading` | `soft.emoji_picker.loading` | Loading emoji |
| `@astryx.emojiPicker.loadFailed` | `soft.emoji_picker.load_failed` | More emoji couldn't be loaded. |
| `@astryx.emojiPicker.groupSmileysEmotion` | `soft.emoji_picker.group_smileys_emotion` | Smileys & emotion |
| `@astryx.emojiPicker.groupPeopleBody` | `soft.emoji_picker.group_people_body` | People & body |
| `@astryx.emojiPicker.groupAnimalsNature` | `soft.emoji_picker.group_animals_nature` | Animals & nature |
| `@astryx.emojiPicker.groupFoodDrink` | `soft.emoji_picker.group_food_drink` | Food & drink |
| `@astryx.emojiPicker.groupTravelPlaces` | `soft.emoji_picker.group_travel_places` | Travel & places |
| `@astryx.emojiPicker.groupActivities` | `soft.emoji_picker.group_activities` | Activities |
| `@astryx.emojiPicker.groupObjects` | `soft.emoji_picker.group_objects` | Objects |
| `@astryx.emojiPicker.groupSymbols` | `soft.emoji_picker.group_symbols` | Symbols |
| `@astryx.emojiPicker.groupFlags` | `soft.emoji_picker.group_flags` | Flags |

**comments** (used in: not used by core components)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.comments.retry` | `soft.comments.retry` | Retry |
| `@astryx.comments.actionsLabel` | `soft.comments.actions_label` | Actions for {name}'s comment |
| `@astryx.comments.addReaction` | `soft.comments.add_reaction` | Add reaction |
| `@astryx.comments.agentBadge` | `soft.comments.agent_badge` | Agent |
| `@astryx.comments.agentLogo` | `soft.comments.agent_logo` | {name} logo |
| `@astryx.comments.chooseReaction` | `soft.comments.choose_reaction` | Choose a reaction |
| `@astryx.comments.copyLink` | `soft.comments.copy_link` | Copy comment link |
| `@astryx.comments.currentVersion` | `soft.comments.current_version` | Current version |
| `@astryx.comments.delete` | `soft.comments.delete` | Delete |
| `@astryx.comments.deleteDescription` | `soft.comments.delete_description` | Delete {name}'s comment? This action cannot be undone. |
| `@astryx.comments.deleteFailed` | `soft.comments.delete_failed` | This comment couldn't be deleted. |
| `@astryx.comments.deleteFailedRetry` | `soft.comments.delete_failed_retry` | This comment couldn't be deleted. Try again. |
| `@astryx.comments.deleteTitle` | `soft.comments.delete_title` | Delete comment? |
| `@astryx.comments.edit` | `soft.comments.edit` | Edit |
| `@astryx.comments.editFailed` | `soft.comments.edit_failed` | Your edit couldn't be saved. |
| `@astryx.comments.editPlaceholder` | `soft.comments.edit_placeholder` | Edit comment |
| `@astryx.comments.edited` | `soft.comments.edited` | Edited |
| `@astryx.comments.editingContext` | `soft.comments.editing_context` | Editing comment |
| `@astryx.comments.hideReplies` | `soft.comments.hide_replies` | Hide replies |
| `@astryx.comments.historyEmptyDescription` | `soft.comments.history_empty_description` | This comment has no earlier versions. |
| `@astryx.comments.historyEmptyTitle` | `soft.comments.history_empty_title` | No edit history |
| `@astryx.comments.historyFailed` | `soft.comments.history_failed` | Edit history couldn't load |
| `@astryx.comments.historyLoadFailed` | `soft.comments.history_load_failed` | The comment's edit history couldn't be loaded. Try again. |
| `@astryx.comments.historyLoading` | `soft.comments.history_loading` | Loading edit history |
| `@astryx.comments.historySubtitle` | `soft.comments.history_subtitle` | Comment by {name} |
| `@astryx.comments.historyTitle` | `soft.comments.history_title` | Edit history |
| `@astryx.comments.linkCopied` | `soft.comments.link_copied` | Comment link copied |
| `@astryx.comments.otherReactors` | `soft.comments.other_reactors` | {count, plural, one {# other} other {# others}} |
| `@astryx.comments.outdated` | `soft.comments.outdated` | Outdated |
| `@astryx.comments.reactionCelebrate` | `soft.comments.reaction_celebrate` | Celebrate |
| `@astryx.comments.reactionCount` | `soft.comments.reaction_count` | {reaction}: {count, number} |
| `@astryx.comments.reactionFailed` | `soft.comments.reaction_failed` | Your reaction couldn't be saved. |
| `@astryx.comments.reactionLaugh` | `soft.comments.reaction_laugh` | Haha |
| `@astryx.comments.reactionLike` | `soft.comments.reaction_like` | Like |
| `@astryx.comments.reactionLove` | `soft.comments.reaction_love` | Love |
| `@astryx.comments.reactionSad` | `soft.comments.reaction_sad` | Sad |
| `@astryx.comments.reactionWow` | `soft.comments.reaction_wow` | Wow |
| `@astryx.comments.reopen` | `soft.comments.reopen` | Reopen |
| `@astryx.comments.reopenFailed` | `soft.comments.reopen_failed` | This thread couldn't be reopened. |
| `@astryx.comments.reply` | `soft.comments.reply` | Reply |
| `@astryx.comments.replyFailed` | `soft.comments.reply_failed` | Your reply couldn't be posted. |
| `@astryx.comments.replyPlaceholder` | `soft.comments.reply_placeholder` | Reply to {name}… |
| `@astryx.comments.replyingContext` | `soft.comments.replying_context` | Replying to {name} |
| `@astryx.comments.resolve` | `soft.comments.resolve` | Mark as resolved |
| `@astryx.comments.resolveFailed` | `soft.comments.resolve_failed` | This thread couldn't be resolved. |
| `@astryx.comments.resolved` | `soft.comments.resolved` | Resolved |
| `@astryx.comments.resolvedBy` | `soft.comments.resolved_by` | Resolved by {name} |
| `@astryx.comments.errorAnnouncement` | `soft.comments.error_announcement` | {title} {description} |
| `@astryx.comments.retryFailed` | `soft.comments.retry_failed` | Retrying didn't work. |
| `@astryx.comments.sendFailed` | `soft.comments.send_failed` | Couldn't send this comment. |
| `@astryx.comments.threadLabel` | `soft.comments.thread_label` | Thread by {name} |
| `@astryx.comments.unread` | `soft.comments.unread` | New |
| `@astryx.comments.unsavedDraft` | `soft.comments.unsaved_draft` | Unsaved draft. Dismiss again to discard it. |
| `@astryx.comments.versionAuthor` | `soft.comments.version_author` | by {name} |
| `@astryx.comments.viewHistory` | `soft.comments.view_history` | View edit history |
| `@astryx.comments.viewReplies` | `soft.comments.view_replies` | View {count, plural, one {# reply} other {# replies}} |
| `@astryx.comments.commentsLabel` | `soft.comments.comments_label` | Threads in {label} |
| `@astryx.comments.emptyDescription` | `soft.comments.empty_description` | Start the discussion with a comment. |
| `@astryx.comments.emptyReadOnlyDescription` | `soft.comments.empty_read_only_description` | There is no discussion on this item. |
| `@astryx.comments.emptyTitle` | `soft.comments.empty_title` | No comments yet |
| `@astryx.comments.label` | `soft.comments.label` | Comments |
| `@astryx.comments.loadFailed` | `soft.comments.load_failed` | Comments couldn't load |
| `@astryx.comments.loadOlder` | `soft.comments.load_older` | Load earlier comments |
| `@astryx.comments.loading` | `soft.comments.loading` | Loading comments |
| `@astryx.comments.olderFailed` | `soft.comments.older_failed` | Earlier comments couldn't load |
| `@astryx.comments.postFailed` | `soft.comments.post_failed` | Your comment couldn't be posted. |

**commentComposer** (used in: not used by core components)

| Astryx key | soft key | English default |
|---|---|---|
| `@astryx.commentComposer.placeholder` | `soft.comment_composer.placeholder` | Write a comment… |
| `@astryx.commentComposer.noAgents` | `soft.comment_composer.no_agents` | No agents |
| `@astryx.commentComposer.agents` | `soft.comment_composer.agents` | Agents |
| `@astryx.commentComposer.uploadFailed` | `soft.comment_composer.upload_failed` | Images couldn't be added |
| `@astryx.commentComposer.imagesOnly` | `soft.comment_composer.images_only` | Only image files can be added |
| `@astryx.commentComposer.chooseImages` | `soft.comment_composer.choose_images` | Choose images |
| `@astryx.commentComposer.imageAlt` | `soft.comment_composer.image_alt` | Image attachment |
| `@astryx.commentComposer.removeQuote` | `soft.comment_composer.remove_quote` | Remove quoted text |
| `@astryx.commentComposer.attachImages` | `soft.comment_composer.attach_images` | Attach images |
| `@astryx.commentComposer.addEmoji` | `soft.comment_composer.add_emoji` | Add emoji |
| `@astryx.commentComposer.cancel` | `soft.comment_composer.cancel` | Cancel |

### 18.4 Theme / useTheme
- `Theme`（`theme/Theme.tsx`）: `theme`（`defineTheme` の結果）と `mode: 'light'|'dark'|'system'`（既定 system）を受け取り、`display:contents` の div に `color-scheme` と `data-theme` を付け、文字色 `--color-text-primary` と font `--font-family-body` を当てる。一番外側の Theme は `<html data-theme>` も同期する。入れ子にすると部分木だけ別テーマ・別モードにできる。全 token は `light-dark()` なので、light / dark の切り替えは `color-scheme` だけで起きる。
- `useTheme()`（`theme/useTheme.ts`）: `{name, mode('light'|'dark'。system はメディアクエリで解決), token(name), tokens}`。
- Kivact: **既存の facility に対応**。kit は `soft::theme(mode: ThemeMode) -> Theme` を 1 つ出す（kivact-ui の Kits 規約。basic の `basic::theme` と同じ形）。`App::theme(..)` か `kivact::view::provide_theme` で入れる。入れ子の Theme は `provide_theme` の境界。tokens は `soft::Tokens`（`ThemeExtension`）を `theme.get::<soft::Tokens>()` で読む。色は `soft::Color` enum（`ThemeColor`）。`useTheme().token(..)` は `current_theme().get::<soft::Tokens>()`。
- light / dark の出し方: `soft::theme(ThemeMode::Light)` と `soft::theme(ThemeMode::Dark)` が、それぞれ neutral の light 値・dark 値をすべて解決済みで持つ（`light-dark()` は使わず、2 つの完全な表）。切り替えは app が `ThemeMode` の signal を持ち、`provide_theme(move || soft::theme(mode.get()), ..)` にする。mounted の部品は再 mount なしで新しい token を読む（basic の README と同じ）。
- `mode="system"`: Kivact の公開 API には OS の外観（ダーク / ライト）を読む関数が見当たらない（`kivact::platform` にあるのは `reduced_motion`, `scale_factor` など）。UNRESOLVED: core に `platform::color_scheme()` を足すか、app が OS から読んで渡す。kit は `ThemeMode` を受け取るだけにする。
- タッチの adaptation（coarse pointer かつ幅 < 1024px で type scale を base 16 に）: 8.4 参照。`soft::theme` のオプションにする案。

### 18.5 MediaTheme
- 何をするか（`theme/MediaTheme.tsx`、`theme/onMediaTokens.ts`）: 画像・動画の上、scrim、toast、tooltip など「反転した面」の上の内容に token の上書きを当てる。`mode: 'dark'|'light'|'auto'|'off'`、`fallback`（auto で判定できないとき、既定 dark）。`dark` の面では `color-scheme: dark` にし、`--color-text-primary` / `--color-icon-primary` / `--color-accent` を `--color-on-dark`（neutral `#ffffff`）に。`light` の面では `color-scheme: light`、同じ 3 つを `--color-on-light`（neutral `#111111`）に。他の token は `light-dark()` が反対側に解決されるだけ。`auto` は親の実際の背景色を合成して、周りの文字色が 3:1 以上で読めるなら `off`、そうでなければ on-dark と on-light のどちらがコントラストが高いかで選ぶ（`hooks/useAutoMediaMode.ts` の `pickMediaMode`）。背景画像があると判定できず fallback。
- Kivact: **kit helper**。`soft::media_theme(MediaMode::Dark, content)` = `provide_theme` の境界で、現在の soft テーマの反対モードの Tokens を取り、text-primary / icon-primary / accent を on-dark / on-light に差し替えたテーマを渡す。`auto` は DOM の計測ができないので、呼び出し側が面の色を渡す `MediaMode::Auto { surface: Rgba }` にして、同じコントラスト計算（WCAG 相対輝度）を kit でする。

### 18.6 SyntaxTheme
- 何をするか（`theme/syntax/*`、`SyntaxTheme.doc.mjs`）: CodeBlock などのコードの色。テーマ全体は `defineTheme({syntax})`、部分木だけ変えるときに `<SyntaxTheme theme={preset}>`。token は `--color-syntax-{keyword,string,comment,number,function,type,variable,operator,constant,tag,attribute,property,punctuation,background}` と diff 用の `added, removed, diff-header, added-marker, removed-marker`。core の既定は palette 由来（keyword = text-accent、string = text-green …、`theme/syntax/tokens.ts`）。neutral は独自の `astryx-neutral` 定義を持つ。プリセット 10 種（one-dark-pro, dracula, monokai, nord, tokyo-night, catppuccin-mocha, github-light, github-dark, solarized-light, one-light、`theme/syntax/presets.ts`）。
- neutral の解決値（light / dark）: keyword `#6b187c` / `#efa8ff`、string `#0b5615` / `#a4d6a3`、comment `#6a6a6a` / `#9e9e9e`、number `#733100` / `#ffb37e`、function `#00458c` / `#a1caff`、type `#6b187c` / `#efa8ff`、variable `#111111` / `#e2e2e2`、operator `#6a6a6a` / `#9e9e9e`、constant `#733100` / `#ffb37e`、tag `#8a0011` / `#ffaea7`、attribute `#584400` / `#eec448`、property `#005348` / `#90d7c8`、punctuation `#6a6a6a` / `#9e9e9e`、background `#ffffff` / `#111111`。diff 系の扱いは 14 章（CodeBlock）。
- Kivact: **kit helper**。`soft::SyntaxColors`（basic の `CodeColors` に相当、`crates/kivact-ui-basic/src/tokens.rs`）を `soft::Tokens` に持たせ、部分木の差し替えは `soft::syntax_theme(colors, content)`（`provide_theme` 境界で Tokens の syntax だけ差し替え）。プリセットは任意（kit の量を増やすだけなので、要るものだけ）。

### 18.7 useClipboard
- `useClipboard({announce?, resetAfterMs = 2000})` → `{copy(text): Promise<bool>, isCopied}`（`hooks/useClipboard.ts`）。成功で `isCopied = true`、`announce` があれば polite でアナウンス、`resetAfterMs` 後に false。連続で押すとタイマーを張り直す。失敗は false を返すだけ。部品ごとの reset: Timestamp 1500ms、CodeBlock は 14 章。
- Kivact: **kit helper**。`kivact::platform::copy_text(text)`（Future）の上に `soft::use_clipboard(reset_after: Duration) -> Clipboard { copied: Signal<bool>, copy: impl Fn(String) }` を作る。アナウンスは kit の live region（VisuallyHidden 章の helper）。

### 18.8 useImageMode
- 画像（src）を CORS で取って 10×10 に縮め、平均色の知覚明度（sRGB を 2.4 乗で線形化 → Y → 0.56 乗）が `threshold`（既定 0.5）を超えれば `light`、そうでなければ `dark`。`region`（0..1 の割合）で一部だけを見られる。失敗や src 無しは `fallback`（既定 null）（`hooks/useImageMode.ts`）。MediaTheme の mode を画像に合わせるのに使う。
- Kivact: **ネイティブでは当面不要**。公開 API で画像の画素を読む手段が無く（`Asset` は描くだけ）、画像のデコードを kit に入れると `kivact` 以外の依存が要る。画像の明暗は app が知っている（自分で読み込んだ画像）ので、`MediaMode::Dark/Light` を app が渡す。必要になったら core に「Asset の平均色」を足す（UNRESOLVED、優先度低）。

### 18.9 useStreamingText
- `useStreamingText(target, isStreaming, {speed})` → 表示する部分文字列（`hooks/useStreamingText.ts`）。LLM の回答を一定の速さで見せる。rAF の各フレームで、前回から `tickMs` 以上たっていれば表示長を `charsPerTick` 進める。
  - `charsPerTick`: natural 10、fast 4、instant ∞。
  - `tickMs` = max(4, round(`--duration-fast-min` ÷ (natural 10 / fast 20)))。neutral の `--duration-fast-min` は 95ms なので natural 10ms、fast 5ms。どちらも 60Hz のフレーム間隔（16.7ms）より短いので、実際は毎フレーム進む: natural 10 文字 / フレーム（約 600 文字/秒）、fast 4 文字 / フレーム（約 240 文字/秒）。
  - 注意: doc（`useStreamingText.doc.mjs:30`）は「natural ~2 chars/frame、fast ~4 chars/frame（backlog に応じる）」と書いており、コードと合わない。UNRESOLVED: docsite の useStreamingText のデモで実測して決める。決まるまではコードの値に従う。
  - 切る位置は grapheme の境界に合わせる（絵文字や結合文字を割らない）。
  - `isStreaming` が false になったら全文を一度に出す。target が空になったら 0 に戻す。
  - reduced motion か `instant` なら全文をそのまま返す。
- Kivact: **kit helper**。`soft::use_streaming_text(target: Signal<String>, streaming: Signal<bool>, speed: StreamingSpeed) -> Memo<String>`。`kivact::motion::frame_loop` で毎フレーム進め、`kivact::platform::reduced_motion()` を見る。grapheme の境界は `kivact` に公開の分割 API が無いので、kit に簡易版（`char` 境界 + 結合文字・ZWJ・異体字セレクタ・肌色修飾・国旗のペアを続ける）を置く（Avatar の頭文字と同じ helper を共用）。

### 18.10 useDevWarning
- `useDevWarning(component, message, condition)`: 開発時に 1 回だけ `console.warn`（`hooks/useDevWarning.ts`、`utils/devWarning.ts`）。
- Kivact: **ネイティブでは不要**（kit 内部の書き方の問題）。kit 内では `cfg!(debug_assertions)` のときに一度だけ報告する小さな関数を置く。Kivact の `icon()` が使う `report_later` は公開でなければ `eprintln!` で良い。利用者向けの API にはしない。

### 18.11 useMergedRefs
- 複数の React ref を 1 つにまとめる。
- Kivact: **不要**。ref の概念が無い（node は builder で組み、状態は signal）。

### 18.12 useHotkeys
- `useHotkeys([{keys, onPress, allowInInputs?, isDisabled?}])`: window の keydown で、最初に合ったものを実行して `preventDefault`（`hooks/useHotkeys.ts`）。IME 変換中と既に処理済みのイベントは無視。入力欄（input / textarea / select / contenteditable）にフォーカスがあるときは `allowInInputs` のものだけ。
- 記法: `"mod+k"`, `"shift+?"`, `"ctrl+alt+delete"`, `"escape"`。`+` で区切り、大小無視、最後が key、それ以外が修飾。修飾: `mod`（Apple なら meta、それ以外 ctrl）、`ctrl`, `meta`, `alt`, `shift`。ctrl / meta / alt は「ちょうど一致」（指定しないなら押されていてはいけない）。shift は「指定したら必要、指定しなければ問わない」（`?` のような shift で出る記号のため）。key の別名: `esc`→escape、`space`→" "、`up/down/left/right`→arrow*、`return`→enter、`plus`→"+"。比べるのは `event.key` の小文字。
- Kivact: **kit helper**。`soft::hotkeys(&[Hotkey::new("mod+k", f)])` を、root（または `provide_*` 境界）の `on_key` に付ける。`KeyEvent.key` と `modifiers` で同じ規則を実装する。入力欄の判定は `KeyEvent` の target role（`input.rs:135-138` のコメントにある「グローバルショートカットがフォーム部品を避けるための role」）を使う。Windows では `mod` = Ctrl。

### 18.13 useAnimatedLayout / useEntryAnimation（概要。詳細は motion.md）
- `useAnimatedLayout({containerRef})` → 不透明な handle。TreeList や Collapsible に渡すと、その部品の大きさが変わったとき、それと containerRef の中の後続の内容が一緒に滑らかに動く（FLIP: `translate` のみ、`motion/animatedLayout.ts`）。spring は `springs.settle` = `{duration: 0.5s, bounce: 0}`（`motion/uikit/springs.ts:41`）を 60 fps でサンプルした `linear()`。0.5px 未満の移動はしない。fixed / sticky と top layer は動かさない。reduced motion: fine pointer なら即時、coarse pointer なら opacity だけの cross-fade 200ms `cubic-bezier(0.42, 0, 0.58, 1)`。
  - Kivact: Kivact の `Transition` は Paint（opacity、背景、border 色、transform、影、filter）だけで、layout の変化を補間しない（`kivact-style/src/sheet.rs:150-158`）。kit helper として「高さが変わる部品の後続を translate で追従させる」仕組みが要るが、layout 後の位置を読む公開 API が無い。UNRESOLVED（motion.md の担当と parent の判断。フォールバックは「動かさず即時に移る」= Astryx で handle を渡さないときの挙動と同じ）。
- `useEntryAnimation(preset)` → mount 時の入場アニメーション用 style。ページの最初の描画より後に mount されたものだけ動かす（初回ロードやハイドレーション中は動かさない）。
  | preset | from → to | duration | easing |
  |---|---|---|---|
  | slideDown（既定） | opacity 0, translateY(−`--spacing-2` = −8px) → 1, 0 | `--duration-fast-max`（neutral 165ms） | `--ease-standard` `cubic-bezier(0.24, 1, 0.4, 1)` |
  | slideUp | opacity 0, translateY(+8px) → 1, 0 | 165ms | 同上 |
  | fadeIn | opacity 0 → 1 | 165ms | 同上 |
  | scaleIn | opacity 0, scale(0.95) → 1, 1 | 165ms | 同上 |
  `animation-fill-mode: backwards`。reduced motion では animation なし（`hooks/useEntryAnimation.ts`）。
  - Kivact: **既存の facility に対応**。`style().enter(style().opacity(0.0).translate_y(-8)).transition(TransitionProp::Paint, Transition::tween(165, Easing::CubicBezier(0.24, 1.0, 0.4, 1.0)))`。reduced motion の扱いは Kivact の transition が持つ（decorative な paint transition は即時になる）。「最初の描画より後に mount されたものだけ」は kit 側で、App の最初のフレーム後に立てるフラグ（`frame_loop` の 1 フレーム目）を context に置いて判定する。

### 18.14 受け入れ確認（Utilities）
- [ ] `App::messages` で `soft.code_block.copy_code` を差し替えると、CodeBlock の名前が変わる
- [ ] `{count, plural, …}` を含む文言（例 `soft.avatar_group.overflow` = "{count, number} more"）が 1 と 2 で正しく出る
- [ ] `soft::theme(Light)` → `soft::theme(Dark)` の切り替えで、mount 済みの部品が再 mount なしで色を変える
- [ ] `soft::media_theme(Dark, ..)` の中の Text primary が `#ffffff` になる
- [ ] `soft::hotkeys` の `mod+k` が Windows の Ctrl+K で動き、text_input にフォーカスがあるときは動かない（`allow_in_inputs` なし）
- [ ] `use_streaming_text` が reduced motion で全文を即時に返す
- [ ] 入場アニメーションが 165ms で上から 8px 降りてくる。最初の描画で出る要素は動かない
