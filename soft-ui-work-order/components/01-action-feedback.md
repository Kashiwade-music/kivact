# 01 — Action & Feedback & small indicators(kivact-ui-soft 作業指示書)

対象: Astryx `packages/core/src` の Button / ButtonGroup / IconButton / ToggleButton / ToggleButtonGroup / Link(+LinkProvider) / MoreMenu / SegmentedControl / SegmentedControlItem / Toolbar / Badge / Banner / ProgressBar / Skeleton / Spinner / StatusDot / Indicator(Checkbox・Check・Radio) / Kbd / Token と、それらが依存する共有 hook。

テーマは **Neutral**(`packages/themes/neutral/src/neutralTheme.ts`)。値は「light / dark」の順で書く。`#RRGGBBAA` は末尾 2 桁がアルファ(例 `#0000000F` = 黒 6%)。px は既定の root 16px 前提(`rem` × 16)。

## 目次

0. [グループ概要表](#0-グループ概要表)
1. [共通基盤(このグループ全体で使う解決済み値と挙動モデル)](#1-共通基盤)
2. [Button](#2-button)
3. [IconButton](#3-iconbutton)
4. [ButtonGroup](#4-buttongroup)
5. [ToggleButton](#5-togglebutton)
6. [ToggleButtonGroup](#6-togglebuttongroup)
7. [Link(+ LinkProvider / useLinkify)](#7-link)
8. [MoreMenu](#8-moremenu)
9. [SegmentedControl / SegmentedControlItem](#9-segmentedcontrol--segmentedcontrolitem)
10. [Toolbar](#10-toolbar)
11. [Badge](#11-badge)
12. [Banner](#12-banner)
13. [ProgressBar](#13-progressbar)
14. [Skeleton](#14-skeleton)
15. [Spinner](#15-spinner)
16. [StatusDot](#16-statusdot)
17. [Indicator(CheckboxIndicator / CheckIndicator / RadioIndicator)](#17-indicator)
18. [Kbd](#18-kbd)
19. [Token](#19-token)
20. [共有 hook の実装メモ](#20-共有-hook-の実装メモ)
21. [UNRESOLVED 一覧](#21-unresolved-一覧)

---

## 0. グループ概要表

| Astryx component (and parts) | docs URL | source dir | kivact strategy | closest kivact-ui-basic analog |
|---|---|---|---|---|
| Button(Icon / Label / End content / Spinner) | https://astryx.atmeta.com/components/Button | `packages/core/src/Button` | RESTYLE `kivact::ui::button`(recipes `button` / `button_content` / `button_icon` / `button_busy`)+ kit Variant `soft::Elevation`, `soft::BusyReveal` | `crates/kivact-ui-basic/src/button.rs` + `recipes.rs`(button recipe) |
| IconButton | https://astryx.atmeta.com/components/IconButton | `packages/core/src/IconButton` | RESTYLE `kivact::ui::button` + kit wrapper `soft::icon_button(label, icon)`(= `.icon(..).hide_label(true)`) | `button.rs`(icon size) |
| ButtonGroup(Button / Divider) | https://astryx.atmeta.com/components/ButtonGroup | `packages/core/src/ButtonGroup` | NEW kit component `soft::button_group`(子は standard button を per-child 修飾) | `crates/kivact-ui-basic/src/button_group.rs` |
| ToggleButton(Icon / Pressed icon / Label / Spinner) | https://astryx.atmeta.com/components/ToggleButton | `packages/core/src/ToggleButton` | RESTYLE `kivact::ui::toggle`(recipe `toggle`)+ kit helper `soft::toggle_icon` | `crates/kivact-ui-basic/src/choice.rs`(toggle look) |
| ToggleButtonGroup | https://astryx.atmeta.com/components/ToggleButtonGroup | `packages/core/src/ToggleButton/ToggleButtonGroup.tsx` | NEW kit component `soft::toggle_button_group`(toggle + list interaction) | `crates/kivact-ui-basic/src/choice.rs`(`toggle_group`) |
| Link(Label / Left icon / Right icon / external icon) | https://astryx.atmeta.com/components/Link | `packages/core/src/Link` | NEW kit component `soft::link` | `crates/kivact-ui-basic/src/breadcrumb.rs`(`breadcrumb_link`) |
| LinkProvider(注記のみ) | https://astryx.atmeta.com/components/LinkProvider | `packages/core/src/Link/LinkProvider.tsx` | covered by another component(native では不要。`soft::provide_navigator` で on_press 既定を差し込む案のみ) | なし |
| MoreMenu(Trigger / Menu surface / rows) | https://astryx.atmeta.com/components/MoreMenu | `packages/core/src/MoreMenu` | RESTYLE + kit wrapper: `kivact::ui::menu(..).attach(kivact::ui::button(..))` を `soft::more_menu` で包む | `crates/kivact-ui-basic/src/recipes_menu.rs` |
| SegmentedControl(track) | https://astryx.atmeta.com/components/SegmentedControl | `packages/core/src/SegmentedControl` | RESTYLE `kivact::ui::radio_group` + `.variant(soft::Segmented)` + kit 付加(keyboard hint / disabled reason) | (basic に無し)`crates/kivact-ui-creative/src/choice.rs` の `Segmented`、basic では `choice.rs` |
| SegmentedControlItem | https://astryx.atmeta.com/components/SegmentedControlItem | `packages/core/src/SegmentedControl/SegmentedControlItem.tsx` | covered by SegmentedControl(radio_group の option recipe) | 同上 |
| Toolbar(Outer chrome / start / center / end slots) | https://astryx.atmeta.com/components/Toolbar | `packages/core/src/Toolbar` | NEW kit component `soft::toolbar` | `crates/kivact-ui-basic/src/menubar.rs`(roving の参考)、`button_group.rs` |
| Badge(Icon / Label) | https://astryx.atmeta.com/components/Badge | `packages/core/src/Badge` | NEW kit component `soft::badge` | `crates/kivact-ui-basic/src/badge.rs` |
| Banner(frame / status surface / icon / title / description / end area / content surface) | https://astryx.atmeta.com/components/Banner | `packages/core/src/Banner` | NEW kit component `soft::banner`(内部の toggle/dismiss は standard button) | `crates/kivact-ui-basic/src/alert.rs` |
| ProgressBar(Label / Value text / Track / Fill / Mark) | https://astryx.atmeta.com/components/ProgressBar | `packages/core/src/ProgressBar` | RESTYLE `kivact::ui::progress`(recipes `track` / `fill`)+ kit wrapper `soft::progress_bar`(label 行・marks) | `recipes.rs` の progress recipe |
| Skeleton | https://astryx.atmeta.com/components/Skeleton | `packages/core/src/Skeleton` | NEW kit component `soft::skeleton` | `crates/kivact-ui-basic/src/primitives.rs`(`skeleton`) |
| Spinner | https://astryx.atmeta.com/components/Spinner | `packages/core/src/Spinner` | NEW kit component `soft::spinner`(button の busy recipe も同じ描画) | `primitives.rs`(`spinner`) |
| StatusDot | https://astryx.atmeta.com/components/StatusDot | `packages/core/src/StatusDot` | NEW kit component `soft::status_dot` | `badge.rs`(小物の作法) |
| Indicator: CheckboxIndicator / CheckIndicator / RadioIndicator | https://astryx.atmeta.com/components/Indicator | `packages/core/src/Indicator` | RESTYLE: `kivact::ui::checkbox` / `radio_group` / `menu`・`select`・`list` の check mark recipe。加えて kit 内部 paint 関数 `soft::paint::checkbox_mark` 等(公開は任意) | `recipes_controls.rs`(checkbox / radio recipe) |
| Kbd | https://astryx.atmeta.com/components/Kbd | `packages/core/src/Kbd` | NEW kit component `soft::kbd` | `primitives.rs`(`kbd` / `kbd_group`) |
| Token(Icon / Label / End content / Remove button) | https://astryx.atmeta.com/components/Token | `packages/core/src/Token` | NEW kit component `soft::token`(tag_input の tag recipe にも同じ見た目を流用) | `badge.rs` + `recipes_controls.rs`(tag_input の tag) |

---

## 1. 共通基盤

この章の値は各コンポーネント節でも初出時に再掲するが、ここを正とする。

### 1.1 解決済みトークン(Neutral)

出典: `packages/core/src/theme/tokens.stylex.ts`、`packages/themes/neutral/src/neutralTheme.ts`(tokens 87-273, components 275-380)、`neutralPaletteRefs.generated.ts`。

| Astryx token | light | dark | 備考 |
|---|---|---|---|
| `--color-accent` | `#1b1b1b` | `#f1f1f1` | Neutral のアクセントは「ほぼ黒/ほぼ白」 |
| `--color-on-accent` | `#ffffff` | `#111111` | |
| `--color-neutral` | `#0000000F` | `#FFFFFF1A` | secondary ボタン地・セグメント track・Kbd 地 |
| `--color-background-surface` | `#ffffff` | `#262626` | |
| `--color-background-body` | `#f1f1f1` | `#1b1b1b` | ページ地 |
| `--color-background-card` | `#ffffff` | `#1b1b1b` | |
| `--color-background-popover` | `#ffffff` | `#1b1b1b` | |
| `--color-background-muted` | `#f1f1f1` | `#1b1b1b` | |
| `--color-overlay-hover` | `#0000000D` | `#FFFFFF0D` | hover 重ね塗り |
| `--color-overlay-pressed` | `#0000001A` | `#FFFFFF1A` | pressed 重ね塗り / ToggleButton の on 地 |
| `--color-text-primary` | `#000000` | `#ffffff` | |
| `--color-text-secondary` | `#474747` | `#9e9e9e` | |
| `--color-text-disabled` | `#919191` | `#525252` | |
| `--color-text-accent` | `#1b1b1b` | `#f1f1f1` | |
| `--color-icon-secondary` | `#6a6a6a` | `#9e9e9e` | |
| `--color-icon-disabled` | `#919191` | `#525252` | |
| `--color-border` | `#00000014` | `#FFFFFF1A` | |
| `--color-border-emphasized` | `#d4d4d4` | `#525252` | |
| `--color-skeleton` | `#f1f1f1` | `#525252` | |
| `--color-track` | `#CCD3DB` | `#5A5E66` | **Neutral は上書きしない**(core 既定値のまま) |
| `--color-tint-hover` | `#000000` | `#ffffff` | `color-mix` 用 |
| `--color-on-dark` | `#ffffff` | `#ffffff` | |
| `--color-info` / `--color-on-info` | `#0074e2` / `#ffffff` | `#6d9cfe` / `#111111` | |
| `--color-success` / `--color-on-success` | `#198100` / `#ffffff` | `#64af4c` / `#111111` | |
| `--color-warning` / `--color-on-warning` | `#ffce2f` / `#111111` | `#ffce2f` / `#111111` | |
| `--color-error` / `--color-on-error` | `#c9303a` / `#ffffff` | `#ff705d` / `#111111` | |
| `--color-info-muted` / `--color-on-info-muted` | `#b9d7ff` / `#003978` | `#88bcff3D` / `#a1caff` | |
| `--color-success-muted` / `--color-on-success-muted` | `#bce0bb` / `#00490b` | `#90ca903D` / `#90ca90` | |
| `--color-warning-muted` / `--color-on-warning-muted` | `#fae19e` / `#4b3900` | `#e2b6233D` / `#eec448` | |
| `--color-error-muted` / `--color-on-error-muted` | `#ffc4be` / `#76000c` | `#ff98903D` / `#ffaea7` | |

装飾色(Badge / Token 用。`--color-background-X` / `--color-text-X`):

| X | background light | background dark | text light | text dark |
|---|---|---|---|---|
| red | `#ffc4be` | `#5b2b28` | `#76000c` | `#ffaea7` |
| orange | `#ffc7a1` | `#503424` | `#622700` | `#ffb37e` |
| yellow | `#fae19e` | `#453a1c` | `#4b3900` | `#eec448` |
| green | `#bce0bb` | `#2b422b` | `#00490b` | `#90ca90` |
| teal | `#a9e2d6` | `#28413c` | `#00463d` | `#90d7c8` |
| cyan | `#9ae2f4` | `#274046` | `#004351` | `#85d5e9` |
| blue | `#b9d7ff` | `#253c5a` | `#003978` | `#a1caff` |
| purple | `#f7d5ff` | `#4a2f51` | `#5c0e6c` | `#eaacf8` |
| pink | `#ffc0d7` | `#572b3d` | `#70003f` | `#ffa9ca` |
| gray | `#e2e2e2` | `#303030` | `#1b1b1b` | `#d4d4d4` |

寸法・タイポ:

| token | 値 |
|---|---|
| `--spacing-0-5` / `-1` / `-1-5` / `-2` / `-3` / `-4` / `-5` | 2 / 4 / 6 / 8 / 12 / 16 / 20 px |
| `--size-element-sm` / `-md` / `-lg` | 28 / 32 / 36 px |
| `--border-width` | 1px |
| `--radius-inner` / `-element` / `-container` / `-page` / `-full` | 6px(0.375rem) / 10px(0.625rem) / 12px(0.75rem) / 28px / 9999px(Neutral は inner と element を既定 4/8 から拡大) |
| `--text-label-size` / `-leading` / weight | 14px / 1.4286(=20px) / `--font-weight-medium` 500 |
| `--text-body-size` / `-leading` / weight | 14px / 20px / 400 |
| `--text-supporting-size` / `-leading` / weight | 12px / 1.6667(=20px) / 400 |
| `--font-weight-semibold` / `-bold` | 600 / 700 |
| font family | Neutral: body/heading = `Figtree`(fallback `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif`)、code = `ui-monospace` |
| touch 適応 | Neutral は `pointer: coarse` かつ幅 < `lg` で type base 14→16px(ratio 1.1736)。本グループの文字サイズは全てこの scale を参照するので、touch+狭幅では label/body 16px, supporting ≈13.6px になる(neutralTheme.ts:72-80)。kivact で同等の適応を持つかは foundation/tokens.md の判断に従う |

フォーカスリング(`tokens.stylex.ts:239-246`、`utils/focusOutline.stylex.ts:91-106`): `--focus-outline-width` 2px, `--focus-outline-style` solid, `--focus-outline-color` = `--color-accent`(`#1b1b1b` / `#f1f1f1`), `--focus-outline-offset` 3px。**キーボードフォーカス(`:focus-visible`)のときだけ**描く。outline は要素の border-radius に沿う(radius + offset の角丸)。

影(Neutral, neutralTheme.ts:256-272):

| token | light | dark |
|---|---|---|
| `--shadow-low` | `0 2px 4px rgba(0,0,0,.05), 0 4px 8px rgba(0,0,0,.10)` | `0 2px 4px rgba(0,0,0,.25), 0 4px 8px rgba(0,0,0,.40), inset 0 0 0 1px rgba(255,255,255,.08)` |
| `--shadow-med` | `0 2px 4px rgba(0,0,0,.05), 0 4px 12px rgba(0,0,0,.10)` | `0 2px 4px rgba(0,0,0,.35), 0 4px 12px rgba(0,0,0,.50), inset 0 0 0 1px rgba(255,255,255,.12)` |
| `--shadow-high` | `0 4px 6px rgba(0,0,0,.10), 0 12px 24px rgba(0,0,0,.15)` | `0 4px 6px rgba(0,0,0,.50), 0 12px 24px rgba(0,0,0,.70), inset 0 0 0 1px rgba(255,255,255,.15)` |

モーション(Neutral `motion: {fast:125, medium:300, slow:700, ratio:0.75}`, neutralTheme.ts:85。min=base×0.75, max=base/0.75 を 5ms 丸め):

| token | 値 |
|---|---|
| `--duration-fast-min` / `fast` / `fast-max` | 95 / 125 / 165 ms |
| `--duration-medium-min` / `medium` / `medium-max` | 225 / 300 / 400 ms |
| `--duration-slow-min` / `slow` / `slow-max` | 525 / 700 / 935 ms |
| `--ease-standard` = `--ease-entrance` = `--ease-move` | `cubic-bezier(0.24, 1, 0.4, 1)` |
| `--ease-exit` | `cubic-bezier(0.3, 0, 0.6, 0.6)` |

Neutral は `tempo` を設定しないので `--_astryx-pre-tempo-duration-*` は未定義 → ループ系(Spinner / Skeleton)は通常トークン値を使う。

kivact の `Easing::CubicBezier(x1,y1,x2,y2)` / `Transition` でそのまま表現できる。CSS の `ease` は `CubicBezier(0.25, 0.1, 0.25, 1.0)`、`ease-in-out` は `CubicBezier(0.42, 0, 0.58, 1)`(kivact の `Easing::EaseInOut` は 3 次多項式で別物なので**使わない**)。

### 1.2 hover / pressed の重ね塗りモデル(interactionOverlay)

出典 `utils/interactionOverlay.stylex.ts:210-336`。Astryx の操作面は「地の色 + 半透明オーバーレイ」を重ねる。

- **backgroundImage 型**(Button, Token など地色を持つ面): 地色は `background-color` に置き、その上に `linear-gradient(overlay, overlay)` を重ねる。
  - rest: 重ね無し
  - hover(hover 可能なポインタのみ, `@media (hover: hover)`): `--color-overlay-hover` を 1 枚重ねる(`#0000000D` / `#FFFFFF0D`)
  - pressed(マウス `:active`): `--color-overlay-pressed` を 1 枚重ねる(`#0000001A` / `#FFFFFF1A`)。hover と pressed は置換(足し算ではない)
  - disabled / aria-disabled: 重ねない(ENABLED ガード)
- **backgroundColor 型**(透明地の面: SegmentedControlItem 非選択): `background-color` 自体を transparent → overlay-hover → overlay-pressed に置き換える。
- **pressedBackgroundColor 型**(Link): hover では塗らず、pressed のみ overlay-pressed。
- **合成色の計算**: kivact では 1 色にしてよい。合成 = `src_over(base, overlay)`。例: secondary(`#0000000F`)+ hover(`#0000000D`)= 黒 α ≈ 0.06+0.051×0.94 ≈ 0.108 → `#0000001C` 相当。暗色も同式(白)。
- **遷移**: Button は `background-image` を transition 対象に含めるが、CSS はグラデーションを補間しないので **hover/pressed の重ねは即時に切り替わる**。kivact でも「即時切替」を正とする(フェードを足さない)。
- **タッチ(coarse pointer)の押下**(`utils/pressGesture.ts:16-78`, `utils/pressFeedback.ts`): 指は `:active` を使わず、コントローラが属性で押下を描く。
  - onset 150ms 指が静止したら pressed を描く(それ以前に 10px 以上動く/スクロールが起きたら描かない)
  - 150ms 未満で離した「速いタップ」は離した瞬間に pressed を描き 100ms 保持してからフェード
  - 解放フェード: 強度 1→0 を **200ms, `cubic-bezier(0, 0, 0.2, 1)`**(`interactionOverlay.stylex.ts:81-92,151`)。reduced motion でも 200ms を上限にそのまま
  - 直前 150ms 以内にスクロールしていた面へのタッチは「ブレーキ」扱いで押下を描かない
  - kivact 実装: `style().pressed(..)` は host 側の押下状態に依存する。タッチ時の onset 遅延・フェードが engine 側で表現できない場合は UNRESOLVED(§21)。最低限「離したら 200ms で pressed overlay をフェードアウト」を `Transition` で付ける。

### 1.3 共通ルール

- disabled の見た目は原則 `opacity: 0.5`(Button / Link / Token / SegmentedControl 全体 / Indicator)。kivact の `theme.disabled_opacity` を 0.5 にする。
- cursor: 有効時 pointer、無効時 default(native ではカーソル形状 API があれば合わせる)。
- RTL: Astryx は全て論理プロパティ(inline-start/end)で書かれている。kivact-ui-basic と同様に `kivact::view::current_direction()` を読んで kit 側で行方向・角丸・矢印キーを反転する。
- アイコン: Neutral は Lucide(`themes/neutral/src/icons.tsx`)。本グループで使う semantic 名 → Lucide 名: `close`→`x`、`chevronDown`→`chevron-down`、`check`→`check`、`success`→`circle-check`、`error`→`circle-x`、`warning`→`triangle-alert`、`info`→`info`、`externalLink`→`external-link`、`moreHorizontal`→`ellipsis`。Lucide の stroke-width は既定 2、描画サイズは `1em`(親の icon box サイズ)。
- Icon のサイズ段(`Icon/IconSize.stylex.ts`): `xsm` 12px / `sm` 16px / `md` 20px / `lg` 24px。Icon 既定色は `inherit`(currentColor)。

### 1.4 文言(i18n キー案)

| 提案キー `soft.*` | Astryx 元キー | English default |
|---|---|---|
| `soft.button.loading` | `@astryx.button.loading` | Loading |
| `soft.spinner.loading` | `@astryx.spinner.loading` | Loading |
| `soft.more_menu.label` | `@astryx.moreMenu.label` | More options |
| `soft.link.new_tab` | `@astryx.link.newTab` | (opens in new tab) |
| `soft.banner.dismiss` | `@astryx.banner.dismiss` | Dismiss |
| `soft.banner.dismiss_titled` | `@astryx.banner.dismissTitled` | {dismiss} {title} |
| `soft.banner.expand` | `@astryx.banner.expand` | Expand |
| `soft.banner.collapse` | `@astryx.banner.collapse` | Collapse |
| `soft.token.remove` | `@astryx.token.remove` | Remove {label} |
| `soft.keyboard_hint.to_navigate` | `@astryx.keyboardHint.toNavigate` | to navigate |
| `soft.kbd.shortcut` | `@astryx.kbd.shortcut` | {shortcut} + {key} |
| `soft.kbd.key.alt` / `.backspace` / `.command` / `.control` / `.control_short` / `.down` / `.enter` / `.escape` / `.escape_short` / `.left` / `.plus` / `.right` / `.shift` / `.tab` / `.up` | `@astryx.kbd.key.*` | Alt / Backspace / Command / Control / Ctrl / Down arrow / Enter / Escape / Esc / Left arrow / Plus / Right arrow / Shift / Tab / Up arrow |

---

## 2. Button

出典: `Button/Button.tsx`(924 行)、`Button/Button.doc.mjs`、`Button/Button.test.tsx`、`utils/interactionOverlay.stylex.ts`、`utils/focusOutline.stylex.ts`、`Spinner/Spinner.tsx`。

### 2.1 概要

アクションを起動する。primary は画面内で最重要の 1 つだけ。他は secondary / ghost。ラベルは動作を述べる(「Save changes」)。時間のかかる操作は loading を出す。icon-only には必ず label(=アクセシブル名)と tooltip。destructive は不可逆操作では確認ステップと併用。**遷移だけなら Link を使う**(Button はアクション用)。

### 2.2 Anatomy

```
<button role=button aria-busy? aria-disabled?>      ← root(height = size、inline-flex, gap 8)
  [spinner overlay span aria-hidden]                 ← loading 時のみ。absolute inset 0, grid center
  <span display:contents aria-hidden=loading?>       ← content wrapper(loading 時に文字色 transparent)
     [icon box 16|20px]                              ← Icon(任意)
     [label span: overflow hidden, ellipsis]         ← label(icon-only では無し)
     [endContent span: inline-flex, color inherit]   ← Badge / chevron 等(icon-only では無し)
  </span>
  <VisuallyHidden role=status aria-live=polite>"Loading"</VisuallyHidden>
</button>
[tooltip layer]                                      ← tooltip 指定時(placement above)
```

```
 md secondary:  ┌──────────────────────────┐ 32px
                │ [✓]  Save changes  (3)   │   padding 8 / 12, gap 8, radius 10
                └──────────────────────────┘
 icon-only md:  ┌────┐ 32×32
                │ ⚙  │
                └────┘
```

### 2.3 API

| Astryx prop | 型 / 既定 | 意味 | Kivact |
|---|---|---|---|
| `label` | string(必須) | 表示文字かつアクセシブル名 | `kivact::ui::button(label)` |
| `variant` | `'primary' \| 'secondary' \| 'ghost' \| 'destructive'` / `'secondary'` | 見た目 | `.variant(ButtonVariant::Primary \| Secondary \| Ghost \| Danger)`。**注意: kivact::ui::button の既定は `Primary`**。soft の docs/catalog では常に明示する。`ButtonVariant::Outline` / `Link` は Astryx に無いが standard の意味なので recipe は描く(2.4.6) |
| `size` | `'sm' \| 'md' \| 'lg'` / `'md'`(SizeContext 継承) | 高さ | `.size(ControlSize::Small \| Medium \| Large)`。`ExtraSmall` は Astryx に無い → 2.4.6 の提案値 |
| `elevation` | `'none' \| 'low' \| 'med' \| 'high'` / `'none'` | FAB 用の静止影。ButtonGroup 内では無視 | kit Variant `.variant(soft::Elevation::Low)` 等(`enum Elevation { None, Low, Med, High }`, `impl Variant`) |
| `isDisabled` | bool / false | 無効 | `NodeBuilder::disabled(..)` |
| `isLoading` | bool / false | spinner + 再起動抑止(フォーカスは保持) | `.loading(value)` |
| `isInterruptible` | bool / false | pending 中も押せる(spinner/aria-busy は出す) | kivact の `.loading` は「押下を無視」なので不可。interruptible が要る場合は `.loading` を使わず kit Variant `soft::BusyLook(true)` で「見た目だけ busy」を描く(recipe が spinner を描き、押下は通す)— 提案 |
| `onClick` | handler | クリック | `.on_press(f)` |
| `clickAction` | `(e) => void \| Promise<void>` | 非同期。pending 中 loading | native に Promise 無し。呼び出し側が `loading` signal を持ち `on_press` で true→完了で false。**action 起因の loading は spinner を 225ms 遅延表示**するので `.variant(soft::BusyReveal::Delayed)` を併用(既定 `Immediate`) |
| `icon` | ReactNode | 先頭アイコン | `.icon(icon("check"))` |
| `isIconOnly` | bool / false | 正方形、label は aria-label | `.hide_label(true)`(+ `.icon`)。`soft::icon_button` 参照 |
| `width` | number(px) \| string | 幅。既定は内容幅、親幅で頭打ち | `.style(style().width(px))` / 100% は `width_pct(100)` 等の kivact style |
| `children` | ReactNode | 表示文字の差し替え(label は名前のまま) | kivact::ui::button は label を可視文字にも使う。差し替えは `button(name).hide_label(true).child(text(..))`(名前は label のまま、可視は child)。この形では icon を付けない(`icon_only` 判定になって正方形化するため)。使用頻度は低いので catalog では扱わなくてよい |
| `endContent` | Icon \| Badge | 末尾。icon-only では無視。色は継承 | アイコン: `.trailing_icon(icon("chevron-down"))`、Badge: `.child(soft::badge("3"))`(順序は icon → label → trailing → children) |
| `tooltip` | string | hover/focus で上に表示。disabled でも focus 可能にするため aria-disabled 化 | `kivact::ui::tooltip(text).attach(button(..))`。disabled + tooltip のとき focus を保つ扱いは §2.6 |
| `href` / `as` / `target` / `rel` | — | link として描画 | **native では無効(drop)**。遷移は `.on_press` で app 側 navigator を呼ぶ。外部 URL を開く必要があれば app の責務 |
| `type` / `name` / `value` / `form` | — | HTML form | drop |
| `xstyle` / `className` / `style` | — | — | `.style(..)` で代替(caller の style が recipe に勝つ) |
| `ref` | — | — | `NodeRef`(`.reference(..)`) |

テーマ var: `--_button-radius`(既定 `--radius-element`)、`--button-focus-offset`(既定 3px)、`--button-icon-only-aspect`(1/1)。kit Tokens に `button_radius`, `button_focus_offset` として持つ。

### 2.4 Visual spec

#### 2.4.1 共通(Button.tsx:83-128 `styles.base`)

- display inline-flex、align center、justify center、gap `--spacing-2` = **8px**
- padding: block 8px / inline 12px(`--spacing-2` / `--spacing-3`)。height は size で固定(下表)なので block padding は実質内容中央寄せの余白
- border なし(destructive の forced-colors 時のみ 1px)
- radius `--radius-element` = **10px**
- font: inherit family(Figtree)、`--text-label-size` 14px、`--text-label-leading` 20px、weight 500
- white-space nowrap、min-width 0、max-width 100% → **狭い行では label が ellipsis で切れる**(label span: overflow hidden + text-overflow ellipsis, Button.tsx:188-192)。名前は常にフル label
- icon box: `iconBoxSizeStyles[iconSize]`、sm/md ボタン → 16px、lg → 20px(Button.tsx:280-284)。アイコン色 = 文字色(currentColor)

#### 2.4.2 size(Button.tsx:204-214)

| size | height | padding (block/inline) | icon | icon-only |
|---|---|---|---|---|
| sm | 28px | 8 / 12 | 16px | 28×28, padding 0 |
| md | 32px | 8 / 12 | 16px | 32×32, padding 0 |
| lg | 36px | 8 / 12 | 20px | 36×36, padding 0 |

icon-only(Button.tsx:164-173): aspect-ratio 1/1、padding 0、flex-shrink 0(行が狭くても潰れない)、max-width none。

#### 2.4.3 variant × state(Button.tsx:234-267、neutralTheme.ts:276-281)

| variant | rest bg | text/icon | hover(+overlay-hover 重ね) | pressed(+overlay-pressed 重ね) | focus-visible ring |
|---|---|---|---|---|---|
| primary | `--color-accent` `#1b1b1b` / `#f1f1f1` | `--color-on-accent` `#ffffff` / `#111111` | bg + `#0000000D` / `#FFFFFF0D` | bg + `#0000001A` / `#FFFFFF1A` | 2px `#1b1b1b` / `#f1f1f1`, offset 3px |
| secondary | `--color-neutral` `#0000000F` / `#FFFFFF1A` | `--color-text-primary` `#000000` / `#ffffff` | 同上 | 同上 | 同上 |
| ghost | transparent | `--color-text-primary` | overlay-hover のみ | overlay-pressed のみ | 同上 |
| destructive(**Neutral 上書き**) | `--color-error-muted` `#ffc4be` / `#ff98903D` | `--color-on-error-muted` `#76000c` / `#ffaea7` | 同上 | 同上 | **2px `--color-error` `#c9303a` / `#ff705d`**, offset 3px(Button.tsx:262-265) |

注: destructive の core 既定は「`--color-error` 塗り + `--color-on-error` 文字」だが Neutral は muted(淡い赤地 + 濃い赤文字)に上書きしている。docsite の Neutral プレビューは muted の方。

dark の destructive 地 `#ff98903D` は半透明なので背後(通常 surface `#262626` / body `#1b1b1b`)と合成される。kivact は α 付き色で塗ればよい。

#### 2.4.4 その他の状態

- **disabled**(真の disabled: isDisabled / group disabled): `opacity: 0.5`(Button.tsx:151-153)、overlay なし、scale なし、cursor default。tooltip が無ければ native disabled(フォーカス不可)。
- **busy-only**(isLoading/pending かつ disabled でない): opacity は**下げない**(spinner をくっきり見せる)。overlay / scale なし(`styles.inactive` Button.tsx:142-150)。フォーカス可能のまま(aria-disabled)。
- **loading の見た目**(Button.tsx:406-458, 786-821): content wrapper の文字色を transparent にして(icon/label/endContent が消えるが幅は保持)、中央に Spinner `size=sm shade=inherit` を重ねる。Spinner sm = 直径 10px + stroke 2px → 14px 箱。arc は currentColor(= variant の文字色)、track は currentColor の 30% 不透明度(§15)。
- **elevation**(Button.tsx:221-226、単独時のみ): none = 影なし、low/med/high = §1.1 の `--shadow-*`。
- **expanded**(menu trigger として開いている時、DropdownMenu.tsx:127,139-141): `--color-overlay-pressed` を背景画像として重ねる(= pressed と同じ見た目が開いている間持続)。kivact では `ButtonState::expanded == Some(true)` で pressed 色を描く。
- **forced colors**: destructive は 1px CanvasText の border。native では高コントラストモード対応がある場合のみ(任意)。

#### 2.4.5 レイアウト規則

- 既定幅 = 内容幅、ただし親幅を超えない(max-width 100%)。flex 行の中では縮んで label が ellipsis。icon-only は縮まない。
- `width` 指定時も親幅で頭打ち(テスト `caps an explicit width at the container`)。
- RTL: icon は inline-start 側(右)に来る。内容順は論理順のまま行を反転。

#### 2.4.6 Astryx に無い kivact の意味(提案値)

- `ButtonVariant::Outline`: 地 transparent、1px `--color-border-emphasized`(`#d4d4d4` / `#525252`)の内側 border、文字 `--color-text-primary`、hover/pressed は ghost と同じ overlay。radius 10px。(Astryx に無いので docsite 比較対象外)
- `ButtonVariant::Link`: §7 Link の accent 見た目(文字 `--color-text-accent`、hover で下線)、padding 0、高さは行高 20px。
- `ControlSize::ExtraSmall`: height 24px(`--size-element-sm` − 4px)、padding-inline 8px、文字 12px/20px weight 500、icon 12px(`xsm`)、radius 6px(`--radius-inner`)。— 提案(UNRESOLVED: Astryx 由来の値ではない)

### 2.5 Motion

| trigger | property | from → to | duration | easing | 備考 / file:line |
|---|---|---|---|---|---|
| 状態変化全般 | background-color, color, opacity, transform | 各値 | `--duration-fast` **125ms** | `--ease-standard` `cubic-bezier(0.24, 1, 0.4, 1)` | reduced motion: 0s。Button.tsx:121-127 |
| hover / pressed overlay | background-image | — | 即時(グラデーションは補間されない) | — | §1.2 |
| マウス押下 | transform | scale(1) → **scale(0.98)** | 125ms(上の transition) | ease-standard | 解放で 0.98→1 も 125ms。**ButtonGroup 内では scale しない**(`!buttonGroup && styles.pressable`, Button.tsx:750)。disabled/busy でも scale しない。coarse pointer ではタッチ押下(onset 後)で scale(0.98)。Button.tsx:129-141 |
| タッチ押下の解放 | pressed overlay 強度 | 1 → 0 | 200ms | `cubic-bezier(0, 0, 0.2, 1)` | §1.2 |
| loading 開始(clickAction / interruptible 起因 = `BusyReveal::Delayed`) | 内容色 → transparent / spinner opacity 0→1 | 内容は 225ms 維持してから 1ms で消える。spinner は 225ms 遅延後 0→1 を **125ms**(timing 関数は CSS 既定 `ease` = `cubic-bezier(0.25,0.1,0.25,1)`) | delay `--duration-medium-min` **225ms** | ease | 225ms 以内に終わった action では spinner を一切出さない。reduced motion: delay 0。Button.tsx:406-458 |
| loading 開始(isLoading 直指定 = `Immediate`) | — | 即座に内容消去 + spinner 表示(フェード無し) | 0 | — | Button.tsx:647, 788-806 |
| spinner 回転 | §15 参照 | 1 回転 525ms linear 無限 | | | reduced motion: 3s/回転(停止はしない) |

### 2.6 Behaviour & a11y

- role button。名前 = label。icon-only / loading 中 / children で表示差し替え時は aria-label=label(Button.tsx:833-837)。
- キー: Enter / Space で起動(標準 button)。aria-disabled 時は Enter/Space を握りつぶし、他のキー(Escape, 矢印)は通す(Button.tsx:725-733)。
- busy(loading)中: `aria-busy=true`、aria-disabled(フォーカス保持)、クリックは preventDefault + stopPropagation(祖先の clickable card に伝播させない, Button.tsx:693-709)。同一 tick の二重クリックは ref ガードで 1 回だけ(interruptible は除外)。
- disabled + tooltip: native disabled にせず aria-disabled(フォーカス可能でツールチップを読める)。kivact: `.disabled(true)` がフォーカスも奪う仕様なら、tooltip 付き disabled は「`.disabled` を使わず押下を無視 + `NodeExt` で disabled セマンティクスのみ」— UNRESOLVED(kivact の disabled がフォーカス可能性を残せるか)。
- live region: 非表示の `role=status aria-live=polite` に loading 中「Loading」を入れる。最初から loading でマウントされた場合は 1 フレーム後に文字を入れる(空 region を先に作る)。kivact: `live_region(LiveRegion::Polite)` の非表示ノードに `soft.button.loading` を入れる。
- ghost は edge-compensation 対象(`data-astryx-edge-comp`): Toolbar / Banner の端に置かれたとき、コンテナがスロットを外側へ引いて文字位置を揃える(§10.4)。kivact では `soft::EdgeComp` マーカー Variant か、toolbar 側で「端の子が ghost button か」を知る手段が必要。

### 2.7 Kivact 実装方針

- RESTYLE。`UiRecipes::button`(root)/ `button_content`(行)/ `button_icon`(アイコン箱)/ `button_busy`(spinner)を soft で全部書く。`look.variant::<ButtonVariant>()`, `look.variant::<soft::Elevation>()`, `look.variant::<soft::BusyReveal>()` を読む。
- root style: `height(size)`, `padding(8,12)`, `radius(10)`, `background(variant)`, `.hover(style().background(composite_hover))`, `.pressed(style().background(composite_pressed).scale(0.98))`, `.focus_visible(outline 2px accent offset 3)`, `.transition(TransitionProp::Paint, Transition::new(125ms, CubicBezier(0.24,1,0.4,1)))`。icon-only → `width = height`, padding 0。
- 研究対象: `crates/kivact-ui-basic/src/button.rs`、`crates/kivact-ui-basic/src/recipes.rs` の button recipe(状態 style の書き方、expanded 時に押下 nudge しない扱い)。
- 難所:
  - busy の回転周期: kivact::ui::button の busy part は「0.7s で 1 回転」固定(button.rs 冒頭)。Astryx Neutral は 525ms/回転。recipe の paint で角度を自前計算できないなら 0.7s を許容(UNRESOLVED §21)。
  - loading 時の「内容は透明だが幅は保持」: `button_content` recipe で loading 時 `opacity(0)` にし(幅は残る)、busy を中央重ね。
  - Delayed reveal: busy recipe の style に `enter(style().opacity(0))` + 225ms delay の transition が書けるか。delay 非対応なら kit 側で loading signal を 225ms 遅らせた派生 Value を作る(`kivact-task` のタイマ)。
  - 単独時のみ scale: ButtonGroup 内では kit が `soft::InGroup` Variant を付けて recipe が scale を外す。

### 2.8 受け入れ確認

- [ ] role=button、名前 = label。icon-only は名前 = label で可視文字なし。
- [ ] variant 4 種 × size 3 種が light/dark で docsite と一致(高さ 28/32/36、radius 10、primary 黒地白字 / dark 白地黒字、destructive は淡赤地+濃赤字)。
- [ ] hover で overlay(5%)、押下で 10% + scale 0.98、離すと 125ms で戻る。
- [ ] Tab フォーカスでのみ 2px リング(offset 3px)。destructive は赤リング。
- [ ] loading: 内容が消え中央 spinner、aria-busy、フォーカス維持、押下無視、「Loading」が読み上げられる。
- [ ] Delayed: 200ms で終わる処理では spinner が一度も出ない。
- [ ] disabled: 50% 不透明、押下無効。
- [ ] 狭い行で label が ellipsis、icon-only は正方形維持。

---

## 3. IconButton

出典: `IconButton/IconButton.tsx`(55 行)、`IconButton.doc.mjs`、`IconButton.test.tsx`。

### 3.1 概要

アイコンだけのボタン。ツールバー・表の行・狭い UI 用。aria-label は具体的に(「Delete conversation」)。ghost を推奨(密な領域)。**tooltip を付ける**(label はスクリーンリーダーにしか届かない)。意味がアイコンで自明でないなら文字付き Button を使う。

### 3.2 Anatomy

`<Button {...props} icon={icon} isIconOnly />` そのもの(IconButton.tsx 全体)。children / endContent / isIconOnly は受け付けない。

### 3.3 API

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `label` | string(必須) | 第 1 引数 = name |
| `icon` | ReactNode(必須) | 第 2 引数 `impl IntoView` |
| `variant` / `size` / `elevation` / `isDisabled` / `isLoading` / `tooltip` / `onClick` / `clickAction` | Button と同じ(variant 既定 secondary) | Button と同じ |

提案: `pub fn icon_button(label: impl IntoText, icon: impl IntoView) -> kivact::ui::Button`(= `button(label).icon(icon).hide_label(true).variant(ButtonVariant::Secondary)`)。戻り値が standard Button なので全 modifier が効く。tooltip は自動付与しない(Astryx も自動ではない)が、catalog の例では必ず `tooltip(label).attach(..)` を併記する。

### 3.4 Visual spec

Button の icon-only と同一: sm 28×28 / md 32×32 / lg 36×36、padding 0、radius 10px、icon 16 / 16 / 20px。variant の色・状態は §2.4.3。

### 3.5 Motion

Button と同一(§2.5)。

### 3.6 Behaviour & a11y

名前 = label(aria-label)、可視文字なし。tooltip 付きは aria-describedby で tooltip 文を説明として結ぶ(Button.tsx:841-849)。kivact: `tooltip(..).attach(..)` が description を結ぶか確認。

### 3.7 Kivact 実装方針

kit wrapper 関数 1 つ。研究対象 `crates/kivact-ui-basic/src/button.rs`(icon size)。`ButtonState::icon_only` で正方形を描くのは §2 の recipe。

### 3.8 受け入れ確認

- [ ] `soft::icon_button("Settings", icon("settings"))` が role=button name="Settings"、32×32。
- [ ] ghost で hover 時のみ 5% 地が出る(light/dark)。

---

## 4. ButtonGroup

出典: `ButtonGroup/ButtonGroup.tsx`(230 行)、`ButtonGroupContext.ts`、`ButtonGroup.doc.mjs`、`ButtonGroup.test.tsx`、Button.tsx:460-560(group 時の角丸・境界線)。

### 4.1 概要

同じ対象への関連操作(コピー/切り取り/貼り付け、undo/redo)を 1 つの連結コントロールにまとめる。全員同じ variant。2〜4 個まで(それ以上は Toolbar / DropdownMenu)。異質な操作(Save と Delete)を混ぜない、ナビゲーションに使わない(SegmentedControl / TabList)、入れ子にしない。group は 1 つの Tab stop、矢印で移動。作業中を示すために group を disabled にしない(フォーカスが飛ぶ)— 起動したボタンに progress を出す。

### 4.2 Anatomy

```
<div role=group aria-label={label} aria-disabled? data-orientation>   ← inline-flex, align stretch
  <Button/>  <Button/>  <DropdownMenu trigger/> ...                    ← 各 member
</div>
```

```
horizontal (secondary, md):
┌──────────┬──────────┬──────────┐ 32px
│   Copy   │   Cut    │  Paste   │   外角のみ radius 10、内側 0
└──────────┴──────────┴──────────┘   区切り = 各非先頭 member の inline-start 1px #00000014
vertical:
┌──────────┐
│   Copy   │   上端だけ radius、区切りは block-start 1px
├──────────┤
│   Cut    │
└──────────┘
```

### 4.3 API

| prop | 型 / 既定 | 意味 | Kivact |
|---|---|---|---|
| `children` | Button / IconButton / DropdownMenu | member | `soft::button_group(label, children: impl soft::GroupMembers)`(tuple / Vec。basic の `GroupItems` と同じ受け方) |
| `label` | string(必須) | group の名前 | 第 1 引数 |
| `orientation` | `'horizontal' \| 'vertical'` / horizontal | 並び | `.orientation(Orientation::Horizontal \| Vertical)` |
| `size` | sm/md/lg / md | member の既定 size(member 側指定が勝つ) | `.size(ControlSize)` — kivact にはサイズ継承 context が無いので、kit が各 member builder に `.size(..)` を適用する(member trait `soft::GroupMember` を Button / Toggle / MoreMenu に実装し、member が size を明示していなければ適用。明示判定ができないなら「group の size を常に適用」でよい — UNRESOLVED) |
| `elevation` | none/low/med/high / none | group 全体の影 | `.variant(soft::Elevation)` 相当のメソッド `.elevation(Elevation)` |
| `isDisabled` | bool / false | 全 member 無効 | `.disabled(v)` を kit が全 member に伝播 |
| `data-testid` / `xstyle` | — | — | `.test_id(..)` / `.style(..)` |

### 4.4 Visual spec

- root: inline-flex、align-items stretch、vertical は column(ButtonGroup.tsx:93-101)。gap 0。
- member 角丸(Button.tsx:499-553): horizontal は先頭の start-start / end-start、末尾の start-end / end-end のみ `--radius-element` 10px、他 0。vertical は先頭の上 2 角、末尾の下 2 角。単独 member は 4 角。**末尾判定はレイヤ要素(tooltip popover 等)を無視**して最後の実 member。
- 区切り: 非先頭 member の inline-start(vertical は block-start)に 1px solid `--color-border` `#00000014` / `#FFFFFF1A`。primary / destructive member の区切りは `--color-on-accent` `#ffffff` / `#111111`(Button.tsx:554-559)。border-box なので member の幅に 1px を含む。
- member の押下 scale は無し(group 内は `pressable` を外す)。hover/pressed overlay は member ごと。
- elevation(ButtonGroup.tsx:107-121): low/med/high で group に `--shadow-*` + radius 10px。member 自身の elevation は無視(group が面を持つ)。
- disabled group: 各 member が opacity 0.5、group は Tab stop を持たない。
- RTL: 行が反転し、先頭(右端)が start 側角丸。

### 4.5 Motion

独自アニメーションなし。member は §2.5(scale 抜き)。

### 4.6 Behaviour & a11y

- role group + aria-label。aria-orientation は付けない(data 属性のみ, テスト `reflects orientation via data-orientation, not aria-orientation`)。
- キー(useListFocus, roving tabindex): Tab stop は 1 つ(初期は先頭 member)。horizontal: ←/→(RTL では反転)、vertical: ↑/↓。Home/End で端へ。端で wrap。disabled member はスキップ・tab stop にしない。tab stop の member が消える/disabled になったら修復。member が開いている menu を持つ間は矢印をその menu に譲る(boundary: `[role=group], [popover]`)。link member も順番に含む。
- 全体 disabled: tab stop を出さない。

### 4.7 Kivact 実装方針

- NEW kit component。`crates/kivact-ui-basic/src/button_group.rs` を土台(per-child style で境界線と角丸を付ける方式がそのまま使える)。違い: (a) 区切りは「非先頭 member の start 側 1px」— basic と同じ発想、(b) 角丸は外角だけ 10px、(c) roving focus を kit で実装: 各 member の `focusable(move || active == i)`、root の `on_key` で矢印/Home/End を処理し `NodeRef::focus()`、RTL 反転、disabled スキップ。
- member に `soft::InGroup { orientation, index, count, solid: bool }` 相当の Variant を付け、button recipe が角丸・区切り色・scale 無しを描く方法でもよい(Variant は mount 前に与える定数なので可)。
- 研究対象: `crates/kivact-ui-basic/src/button_group.rs`、`crates/kivact-ui-basic/src/menubar.rs`(roving)。

### 4.8 受け入れ確認

- [ ] role=group name=label。Tab 1 回で入り 1 回で出る。←→ で移動・wrap、Home/End。
- [ ] 外角のみ丸く、境界 1px(primary は白/黒の線)。
- [ ] isDisabled で全 member 50%・Tab stop なし。
- [ ] elevation=low で group 全体に影、角丸 10。
- [ ] RTL で矢印方向と角丸が反転。

---

## 5. ToggleButton

出典: `ToggleButton/ToggleButton.tsx`(382 行)、`ToggleButton.doc.mjs`、`ToggleButton.test.tsx`。

### 5.1 概要

押すたびに on/off が切り替わるボタン(お気に入り、太字)。on の状態は塗り潰しアイコン等で一目で分かるように(outline 星 → solid 星)。label は on/off で変えない。関連トグルは ToggleButtonGroup に。一度きりの操作(Submit)には使わない。永続設定の on/off は Switch。

### 5.2 Anatomy

```
<Button variant=ghost aria-pressed={isPressed} data-is-pressed>
  [icon | pressedIcon]
  <span inline-flex column center>              ← label wrapper
     <span weight: pressed?600:500>{label}</span>
     <span aria-hidden h0 hidden weight600>{label}</span>   ← 幅予約(太字時の幅で固定)
  </span>
</Button>
```

### 5.3 API

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `label` | string(必須) | `kivact::ui::toggle(label, pressed)` |
| `isPressed` + `onPressedChange(next, e)` | controlled | 第 2 引数 `impl Into<Binding<bool>>`(caller-owned signal) |
| `pressedChangeAction(next)` | async | native 無し。呼び出し側が binding の変化を見て処理。楽観的更新は binding を先に書けば同等。loading 表示は kivact::ui::toggle に loading が無いため UNRESOLVED(必要なら kit Variant `soft::BusyLook` を toggle recipe でも読む) |
| `size` | sm/md/lg / md(group から継承) | `.size(ControlSize)` |
| `elevation` | / none | `.variant(soft::Elevation)` |
| `isDisabled` | / false | `.disabled(..)` |
| `isLoading` | / false | UNRESOLVED(上記) |
| `icon` / `pressedIcon` | ReactNode | `.icon(soft::toggle_icon(pressed_value, "star", "star-filled"))` — kit helper が pressed に応じてアイコンを差し替える view を返す(Lucide に filled 版は無いので、filled は `fill` 付き描画か別アイコン名。catalog は「star」→ on 時 fill=currentColor の星) |
| `isIconOnly` | / false | `.hide_label(true)` |
| `children` | ReactNode | 表示差し替え(既定 label) — drop 推奨 |
| `tooltip` | string | `tooltip(..).attach(toggle(..))` |
| `value` | string | group member の識別子(§6 では ListItem key) |

doc に「icon-only は label から自動 tooltip」とあるが、コード(ToggleButton.tsx:357)は `tooltip` prop をそのまま渡すだけで自動付与しない。**コードを正**とし自動付与しない。

### 5.4 Visual spec

- 本体は **ghost Button**(§2.4)。高さ 28/32/36、padding 8/12、radius 10、gap 8、文字 14/20。
- off: transparent 地、文字 `--color-text-primary` `#000000` / `#ffffff`、weight 500。
- on(pressed, ToggleButton.tsx:44-66): `background-color: --color-overlay-pressed` `#0000001A` / `#FFFFFF1A`、label weight **600**(semibold)。hover 時はさらに overlay-hover 画像を上に重ねる(黒 10% の上に黒 5%)。押下中は overlay-pressed を重ねる(10% の上に 10%)。
- 幅予約: label を semibold で描いたときの幅を常に確保(on/off で幅が変わらない)。kivact: label 行の min-width を semibold 計測幅にするか、semibold で計測した不可視 text を高さ 0 で重ねる(basic と同じテクニックがあれば流用)。icon-only では予約しない。
- disabled: opacity 0.5。forced-colors: on 時 Highlight / HighlightText。

### 5.5 Motion

Button と同じ(背景 125ms ease-standard、押下 scale 0.98 は単独時のみ — ToggleButton は ButtonGroup context に属さないので scale する)。font-weight の切替は即時(補間しない)。

### 5.6 Behaviour & a11y

- role button + `aria-pressed=true|false`。Enter/Space で反転。kivact `use_toggle_button` が pressed セマンティクスを出す。
- disabled + tooltip: フォーカス可能だが反転しない(テスト `keeps a tooltip-bearing disabled member focusable but inert`)。

### 5.7 Kivact 実装方針

RESTYLE `kivact::ui::toggle`(recipe `toggle: ToggleButtonState`)。`state.pressed` で地色と weight、`state.icon_only` で正方形。研究対象 `crates/kivact-ui-basic/src/choice.rs`(`ToggleVariant` と toggle look)、`recipes_controls.rs`。

### 5.8 受け入れ確認

- [ ] role=button、pressed 状態が a11y に出る。Space で切替。
- [ ] on で 10% 地 + 太字、幅が変わらない。
- [ ] off は透明、hover で 5%。light/dark 一致。
- [ ] pressed アイコン差し替えが機能。

---

## 6. ToggleButtonGroup

出典: `ToggleButton/ToggleButtonGroup.tsx`(264 行)、`ToggleButtonGroup.doc.mjs`、`ToggleButton.test.tsx:600-1054`。

### 6.1 概要

ToggleButton を single(排他)/ multiple で束ねる。member は `value` を持ち、押下状態は group が管理(member の isPressed / onPressedChange / pressedChangeAction は無視)。ToggleButton 以外を混ぜない。

### 6.2 Anatomy

```
<div role=group aria-label={label}>   ← inline-flex, align center, gap 4px(vertical: column, stretch)
  <ToggleButton value="b"/> <ToggleButton value="i"/> ...
</div>
```

### 6.3 API

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `label` | string(必須) | 第 1 引数 |
| `type` | `'single' \| 'multiple'` / single | 値の型で決める(basic::toggle_group と同じ): `Binding<Option<K>>` = single、`Binding<Vec<K>>` = multiple |
| `value` / `onChange` | `string \| null` / `string[]` | caller-owned binding |
| `orientation` | / horizontal | `.orientation(..)` |
| `size` | / md | `.size(ControlSize)` |
| `isDisabled` | / false | `.disabled(..)`(group 無効は全 member 無効。member 自身の disabled は group が有効でも維持) |
| children | ToggleButton(value) | `items: impl IntoValue<Vec<ListItem<K>>>`(label / icon 名 / disabled)。`.icon_only(bool)`、`.pressed_icon(|key| -> Option<&'static str>)`、`.item(|item, pressed| view)` slot(任意) |

提案シグネチャ: `soft::toggle_button_group<K>(label, items, value: impl soft::SelectionValue<K>) -> ToggleButtonGroup<K>`。

### 6.4 Visual spec

- root: inline-flex、align center、**gap 4px**(`--spacing-1`)、vertical は column + stretch(ToggleButtonGroup.tsx:62-72)。member は分離した ghost ToggleButton(連結しない、境界線なし)。
- member の見た目は §5.4。

### 6.5 Motion

member の §5.5 のみ。

### 6.6 Behaviour & a11y

- role group + 名前。member は aria-pressed。
- **roving 無し**: 各 member が個別の Tab stop(useListFocus を使っていない)。矢印キーは何もしない。basic::toggle_group(矢印で移動)とは**挙動が違う**ので注意。
- single: 押下中 member を再度押すと null(全解除)。別 member で置換。multiple: トグル追加/削除(押した順に配列末尾へ追加)。

### 6.7 Kivact 実装方針

NEW kit component。各 item を `kivact::ui::toggle(label, derived_binding)` で作る(derived binding は value の包含を読み、書き込みで single/multiple の規則を適用)。研究対象 `crates/kivact-ui-basic/src/choice.rs`(`SelectionValue` sealed trait の扱い、ただし roving は外す)。

### 6.8 受け入れ確認

- [ ] role=group name=label。各 toggle が個別に Tab で辿れる。
- [ ] single: 選択の移動と再押下による解除。multiple: 複数 on。
- [ ] 間隔 4px、連結しない。

---

## 7. Link

出典: `Link/Link.tsx`(487 行)、`Link/Link.doc.mjs`、`Link/LinkProvider.tsx`、`Link/computeTargetAndRel.ts`、`Link/useLinkify.tsx`、`hooks/useInteractiveRole.ts`、`Text/text.stylex.ts`。

### 7.1 概要

ページ遷移のための文字リンク。文言は遷移先を述べる(「click here」禁止)。行内でなく単独で置くときは `isStandalone`(本文サイズを付与)。`label` は中身が文字でない(アイコンだけ)ときだけ付ける — 文字リンクに付けると読み上げが上書きされて有害。遷移しない操作は Button。

### 7.2 Anatomy

```
<a href target rel aria-label? aria-disabled?>         ← inline-flex, align center, gap 2px
   <Text type color display maxLines>{children}</Text>  ← 文字(色は Text 側が持つ)
   [Icon externalLink 12px color=inherit]               ← isExternalLink のみ
   [VisuallyHidden "(opens in new tab)"]                ← isExternalLink のみ
</a>
role 解決(useInteractiveRole): href あり & 有効 → <a> / onClick あり → <button type=button>(link 見た目) /
   InteractiveRoleContext の役割 / それ以外 → inert。disabled + href は <a role=link aria-disabled tabIndex=0>(href を外す)
[Tooltip placement=above]                                ← tooltip 指定時は外側を包む
```

### 7.3 API

| prop | 型 / 既定 | 意味 | Kivact |
|---|---|---|---|
| `children` | ReactNode(必須) | 表示 | `soft::link(text)`(`impl IntoText`)。リッチ中身は `soft::link_with(content: impl IntoView)` |
| `href` | string | 遷移先 | native では URL を持たない。`.on_press(f)` を起動手段にする。`.href(url)` は「`soft::provide_navigator(..)` が設定されていればそれに URL を渡す」糖衣として任意提供 |
| `as` / LinkProvider `component` | component | ルータ統合 | drop。代わりに `soft::provide_navigator(f: impl Fn(&str))`(LinkProvider 相当の context)— 任意 |
| `label` | string | アクセシブル名の上書き(アイコンリンク用) | `.name(..)`(NodeBuilder) |
| `hasUnderline` | bool / false | 常時下線 | `.underline(true)` |
| `isDisabled` | bool / false | 無効(Tab stop は残し disabled として読む) | `.disabled(..)` — Tab stop を残す点は §7.6 |
| `isExternalLink` | bool / false | 新規タブ + 外部アイコン + SR 文 | `.external(true)`(アイコン `external-link` 12px と SR 用文言 `soft.link.new_tab`) |
| `newTabLabel` | string / "(opens in new tab)" | SR 文の上書き | `.new_tab_label(text)` |
| `target` / `rel` / `download` / `referrerPolicy` | — | HTML | drop |
| `onClick` | handler | クリック | `.on_press(f)` |
| `tooltip` | string | | `tooltip(..).attach(link)` |
| `isStandalone` | bool / false | 本文サイズ 14/20 を付与 | `.standalone(true)` |
| `type` | TextType / `'body'` | 文字種 | `.text_type(soft::TextType::Body \| Label \| Supporting \| Large \| Code \| Inherit)` |
| `size` / `weight` | Text の上書き | | `.text_size(..)` / `.weight(..)` |
| `color` | `'accent' \| 'primary' \| 'secondary' \| 'disabled' \| 'placeholder' \| 'inherit'` / accent | 文字色 | `.color(soft::LinkColor::Accent \| Primary \| Secondary \| Disabled \| Placeholder \| Inherit)` |
| `display` | / inline | | 行内(段落中)は kivact の rich text 内リンクが必要(UNRESOLVED)。単独は inline-flex ノード |
| `maxLines` | number / 0 | 行数で省略 | `.max_lines(n)` |

### 7.4 Visual spec

- root(Link.tsx:62-84): inline-flex、align center、gap `--spacing-0-5` **2px**、font は継承(family/size/leading/weight)。isStandalone 時 14px / 20px(Link.tsx:115-118)。
- 文字色(Text が持つ, Text/text.stylex.ts:26-45): accent `--color-text-accent` `#1b1b1b` / `#f1f1f1`、primary `#000000` / `#ffffff`、secondary `#474747` / `#9e9e9e`、disabled `#919191` / `#525252`、placeholder = secondary、inherit。
- 既定 type body = 14px / 20px / weight 400(`type='inherit'` で周囲の文字に従う)。
- 下線: 既定なし。hover(hover 可能ポインタ・有効時)で underline。`hasUnderline` で常時。
- hover 色(Link.tsx:125-159): **リンク要素**の color を `color-mix(in srgb, <色>, --color-tint-hover 15%)` に変える。ただし内側の Text が自分の色を持つため、**文字色そのものは変わらず、変わるのは下線色と外部アイコン色**。値: accent → `#171717` / `#f3f3f3`、primary → `#000000` / `#ffffff`(不変)、secondary → `#3c3c3c` / `#adadad`。差はごく僅かなので、kivact では「hover = 下線が出る(文字色と同じ色)」で十分。
- pressed(`pressedBackgroundColor`, Link.tsx:390,458): マウス押下中 `--color-overlay-pressed` `#0000001A` / `#FFFFFF1A` を背景に(角丸なし、文字の箱そのまま)。hover では背景を塗らない。
- disabled(Link.tsx:110-114): opacity 0.5、pointer 無効、cursor default。
- focus-visible: 2px accent リング、offset 3px、radius 0。
- 外部アイコン: `externalLink`(Lucide `external-link`)12px(`xsm`)、色 inherit、文字の後ろ(inline-end。RTL では左)。
- maxLines>0: max-width 100%, min-width 0、Text が ellipsis / line clamp。
- button 化(href なし + onClick): 背景透明・border なし・padding 0・text-align inherit で見た目は同一(Link.tsx:97-106)。

```
 Documentation ↗      ← accent 文字 + 12px external icon(gap 2px)
 ‾‾‾‾‾‾‾‾‾‾‾‾‾        ← hover 時のみ下線
```

### 7.5 Motion

| trigger | property | duration | easing | file:line |
|---|---|---|---|---|
| hover in/out | color(下線・アイコン色), text-decoration | `--duration-fast` 125ms | `--ease-standard` `cubic-bezier(0.24, 1, 0.4, 1)` | Link.tsx:81-83。text-decoration は補間されず即時に出入り |
| 押下解放(タッチ) | pressed overlay | 200ms | `cubic-bezier(0, 0, 0.2, 1)` | §1.2 |

### 7.6 Behaviour & a11y

- href → role link、Enter で起動(Space では起動しない — link 標準)。button 化した場合は role button で Enter/Space。
- disabled の a: role link + aria-disabled + tabIndex 0(Tab stop を残す)、Enter/Space を抑止(`disabledInteractionProps`)、href・onClick を外す。kivact: `Role::Link` ノードで `focusable(true)` を保ち、disabled セマンティクスを出して on_press を呼ばない。
- 外部リンク: 名前の末尾に「(opens in new tab)」が加わる(VisuallyHidden)。kivact は name を `"{text} {new_tab}"` にする(Astryx と同じく名前に含める)。

### 7.7 LinkProvider(注記)と useLinkify

- LinkProvider は「サブツリーの全リンク(Link / Button href / Token href …)が使う `<a>` を router の Link に差し替える」context。native には `<a>` もルータも無いので **コンポーネント化しない**。必要なら `soft::provide_navigator(f)` を用意し、`.href(url)` 糖衣がこれを呼ぶ。
- `useLinkify`(Link ディレクトリ内): 文字列中の URL(`https?://[^\s<>'")\]},]+`)とメール(`[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}`)を Link に置換する hook。今回の対象外。作るなら URL は external 扱い、メールは `mailto:`。

### 7.8 Kivact 実装方針

- NEW kit component。`Role::Link` の 1 ノード(text + 任意 icon)。`use_button()` の part で press / Enter を受け、Space は起動しない(link セマンティクス。use_button が Space も起動するなら on_key で Space を消費して握りつぶす)。
- 下線は「text decoration は state style で変えられない」(kivact-ui `values.rs` ButtonState::input のコメント)ため、`PartState` の hover を読んで paint で下線を描く(kivact-ui の `ButtonVariant::Link` と同じ手法)。
- 研究対象: `crates/kivact-ui-basic/src/breadcrumb.rs`(`breadcrumb_link` の press と下線)、kivact-ui `look.rs` の Link 分岐。

### 7.9 受け入れ確認

- [ ] role=link name=文字。Enter で on_press。Space は起動しない。
- [ ] 既定 accent 色(`#1b1b1b` / `#f1f1f1`)、hover で下線、外れたら消える。
- [ ] external: 12px アイコン、名前末尾に「(opens in new tab)」。
- [ ] disabled: 50%、Tab で到達できるが起動しない、disabled として読まれる。
- [ ] 押下中に 10% の背景。

---

## 8. MoreMenu

出典: `MoreMenu/MoreMenu.tsx`(189 行)、`MoreMenu.doc.mjs`、`MoreMenu.test.tsx`、`DropdownMenu/DropdownMenu.tsx:127-147`(trigger の open 見た目)。

### 8.1 概要

「⋯」の icon-only ボタンで副次アクションのメニューを開く(表の行、カードヘッダー、ツールバー)。主要アクションは外に出したまま、副次だけ入れる。項目が多ければ divider / section で分ける。

### 8.2 Anatomy

```
DropdownMenu
 ├─ trigger: Button(icon-only, variant ghost, size md, icon moreHorizontal, label "More options", tooltip=label,
 │            aria-haspopup=menu, aria-expanded, hasChevron=false)
 └─ menu surface(DropdownMenu の popover)→ 詳細は DropdownMenu 担当の指示書
```

### 8.3 API

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `items` | `DropdownMenuOption[]`(必須) | `entries: impl IntoValue<Vec<MenuEntry<K>>>` + `choose: impl Fn(K) -> R` |
| `label` | string / "More options" | `.label(text)`(既定 `soft.more_menu.label`)。trigger の名前と tooltip 文の両方 |
| `variant` | ButtonVariant / ghost | `.variant(ButtonVariant)` |
| `size` | ButtonSize / md | `.size(ControlSize)` |
| `icon` | ReactNode / 三点 | `.icon(view)`(既定 `icon("ellipsis")`) |
| `isDisabled` | / false | `.disabled(..)` |
| `placement` | above/below/start/end / below | `.placement(Placement)` |
| `alignment` | start/center/end / start | `.align(OverlayAlign)`(overflow menu は end 推奨) |
| `presentation` | popover / bottom-sheet / adaptive / popover | drop(native は popover のみ)。小型タッチ端末で bottom sheet にする方針は menu / drawer 担当と調整(UNRESOLVED) |
| `isMenuOpen` / `onOpenChange` | controlled | kivact::ui::menu に open binding があれば接続、無ければ drop(UNRESOLVED) |

提案: `pub fn more_menu<K, R>(entries, choose) -> soft::MoreMenu<K>`。`IntoView` で `kivact::ui::menu(label, entries, choose).attach(trigger)` を返し、trigger = `button(label).icon(icon("ellipsis")).hide_label(true).variant(ButtonVariant::Ghost).size(size)`。tooltip(label) も trigger に付ける(tooltip と menu の attach の入れ子順は実装で確認)。

### 8.4 Visual spec

- trigger = §3 IconButton の ghost、md 32×32、icon `ellipsis` 16px、色 `--color-text-primary` `#000000` / `#ffffff`。
- menu が開いている間: trigger に `--color-overlay-pressed` `#0000001A` / `#FFFFFF1A` を重ねる(pressed と同じ見た目が持続)。kivact では `ButtonState::expanded == Some(true)` で描く(button recipe の責務)。
- chevron は付けない。
- menu surface・行の見た目は DropdownMenu の指示書に従う。参考: surface padding 4px、radius `--radius-container` 12px(DropdownMenu.tsx:175-178)。

### 8.5 Motion

trigger は Button(§2.5)。開閉アニメーションは menu recipe 側(本書の対象外)。

### 8.6 Behaviour & a11y

- trigger role button、名前「More options」、`aria-haspopup=menu`、`aria-expanded`。tooltip で同文を表示。
- Enter/Space/↓ で開く(menu 標準)。マウス押下で即開き、押したままドラッグして行で離すと実行(useMenuPress, §20.6)。指で 500ms 長押しでも開く。

### 8.7 Kivact 実装方針

RESTYLE + kit wrapper。menu の見た目は menu recipe(DropdownMenu 担当)。研究対象 `crates/kivact-ui-basic/src/recipes_menu.rs`、basic README の「popover/menu trigger は押下中に動かず、開いている間 muted fill を保つ」記述(同じ方針)。

### 8.8 受け入れ確認

- [ ] role=button name="More options"(既定)、tooltip が同文。
- [ ] 開くと trigger が押下色のまま、閉じると戻る。押下中に scale しない(popup trigger)。
- [ ] 項目選択で choose が呼ばれ閉じる。

---

## 9. SegmentedControl / SegmentedControlItem

出典: `SegmentedControl/SegmentedControl.tsx`(346 行)、`SegmentedControlItem.tsx`(277 行)、`SegmentedControlContext.ts`、両 `.doc.mjs`、`SegmentedControl.test.tsx`、`hooks/useKeyboardHint.tsx`、neutralTheme.ts:333-357。

### 9.1 概要

少数の排他的な選択肢(表示モード: Grid / List 等)を横並びのセグメントで選ぶ。ページ切替などのナビゲーションには TabList。選択肢が多い・長いなら Selector。icon-only セグメントにも label を与える(名前になる)。全体 disabled の理由は `disabledMessage` で付ける(Tooltip で包まない)。

### 9.2 Anatomy

```
<div role=radiogroup aria-label aria-disabled? aria-describedby?>   ← track: --color-neutral 地, padding 4px(Neutral), gap 2px
   <button role=radio aria-checked tabIndex=0|-1 data-value>         ← segment
      [icon span 14|16|18px]  [label span ellipsis]
   </button> ...
   {keyboard hint popover}                                           ← 初回キーボード入場時のみ
</div>
{disabled reason tooltip}
```

```
md (Neutral):
┌─────────────────────────────────────────┐ 32px  track radius 10, padding 4, gap 2
│ ┌──────────┐                            │
│ │  ▦ Grid  │   ☰ List      ▤ Board      │ 24px segments, radius 8
│ └──────────┘                            │ 選択 = surface 地(白 / #262626)、影なし
└─────────────────────────────────────────┘
```

### 9.3 API

SegmentedControl:

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `value` / `onChange` | string(必須) | `kivact::ui::radio_group(label, options, value: Binding<Option<K>>)`(None = 該当なし) |
| `label` | string(必須、非表示) | 第 1 引数 |
| `size` | sm/md/lg / md | `.size(ControlSize::Small \| Medium \| Large)` |
| `layout` | `'hug' \| 'fill'` / hug | `.variant(soft::SegmentedLayout::Hug \| Fill)` |
| `isDisabled` | / false | `.disabled(..)`(選択中 segment に Tab stop を残す) |
| `disabledMessage` | string | kit wrapper の `.disabled_reason(text)`(disabled 時に group へ tooltip + 各 segment の description) |
| children | SegmentedControlItem | `options: Vec<ListItem<K>>`(`ListItem::new(key, label).icon("layout-grid").disabled(..)`) |

SegmentedControlItem:

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `value` | string | `ListItem` の key |
| `label` | string(必須) | `ListItem` label(icon-only 時は名前) |
| `isLabelHidden` | / false | 全体 icon-only は `.variant(soft::SegmentedIconOnly)`。個別は `.item(|item, state| ..)` slot で icon のみ描く(名前は ListItem label) |
| `icon` | ReactNode | `ListItem::icon(name)` |
| `isDisabled` | / false | `ListItem::disabled(true)` |
| `onClick` | handler | drop(選択は binding で観測) |

kit wrapper 案: `soft::segmented_control(label, options, value) -> soft::SegmentedControl<K>`(内部 = `radio_group(..).orientation(Orientation::Horizontal).variant(soft::Segmented)`)+ `.size`, `.layout`, `.icon_only`, `.disabled`, `.disabled_reason`。`soft::Segmented` 単体の Variant も公開し、radio_group に直接付けても同じ見た目になるようにする。

### 9.4 Visual spec

track(SegmentedControl.tsx:95-145 + neutralTheme.ts:333-337):
- display inline-flex、align center、width fit-content(hug)。label 付き segment があるとき max-width 100% / min-width 0(親幅で頭打ちし segment label が ellipsis)。全 icon-only のときは縮めない。
- **padding 4px**(Neutral 上書き `var(--spacing-1)`。core 既定 2px)。gap **2px**(`--spacing-0-5`)。
- 地 `--color-neutral` `#0000000F` / `#FFFFFF1A`。radius `--radius-element` **10px**(全 size 同じ)。
- 外形高さ = segment 高 + 8px = sm **28** / md **32** / lg **36**(Neutral の意図「外形高さは変えず内側を広く」)。
- fill: display flex、width 100%、各 segment flex 1 / min-width 0。
- 全体 disabled: opacity 0.5(track ごと)。

segment(SegmentedControlItem.tsx:74-178 + neutralTheme.ts:338-357):
- inline-flex、center、gap **4px**、border なし、地 transparent、position relative。
- 高さ(Neutral): sm **20px**(28−8)/ md **24px**(32−8)/ lg **28px**(36−8)。
- padding-inline: sm 8px / md 12px / lg 12px。
- radius(同心): `max(0, --_segmented-control-radius − --_segmented-control-padding)` = 10 − 2 = **8px**。注: Neutral は track の `padding` だけを 4px にしており private var `--_segmented-control-padding` は 2px のまま → 8px(幾何的な同心は 6px だがソース通り 8px。docsite で確認推奨 §21)。
- 文字: sm 12px(`--text-supporting-size`)/ md・lg 14px、line-height 20px、weight 500、色 `--color-text-secondary` `#474747` / `#9e9e9e`。nowrap、label ellipsis。
- icon box: sm 14px / md 16px / lg 18px、shrink 0。icon 色 = 文字色。
- 状態:
  - rest(非選択): 地 transparent、文字 secondary。
  - hover(非選択・有効・hover 可能ポインタ): 地 `--color-overlay-hover` `#0000000D` / `#FFFFFF0D`。
  - pressed(非選択・有効): 地 `--color-overlay-pressed` `#0000001A` / `#FFFFFF1A`。
  - selected: 地 `--color-background-surface` **`#ffffff` / `#262626`**、文字 `--color-text-primary` `#000000` / `#ffffff`、weight **600**、box-shadow: core は `--shadow-low` だが **Neutral は none**(neutralTheme.ts:353-355)。selected には hover/pressed overlay を付けない(押下フィードバックも無し)。
  - disabled(個別 or 全体): 文字 `--color-text-disabled` `#919191` / `#525252`、overlay なし。
  - focus-visible: 2px accent リング(`#1b1b1b` / `#f1f1f1`)offset 3px、8px 角丸に沿う。
  - forced-colors: selected = Highlight / HighlightText。
- RTL: segment の並びが反転。

### 9.5 Motion

| trigger | property | from → to | duration | easing | file:line |
|---|---|---|---|---|---|
| 選択の移動 | 旧: 地 surface→transparent、文字 primary→secondary / 新: 逆 | 各 segment で独立のクロスフェード(**スライドする indicator は無い**) | 125ms | `cubic-bezier(0.24, 1, 0.4, 1)` | SegmentedControlItem.tsx:98-100 |
| hover / pressed | background-color | transparent ↔ overlay | 125ms | 同上 | 同上(background-color 型なので補間される) |
| font-weight 500↔600 | — | 即時 | — | — | |
| keyboard hint 出現 / 消失 | opacity 0 ↔ 1 | | **150ms** | CSS 既定 `ease` = `cubic-bezier(0.25, 0.1, 0.25, 1)` | useKeyboardHint.tsx:139-143 |

reduced motion: 分岐なし(Astryx はそのまま 125ms)。kivact の decorative 規則で即時になっても可。

### 9.6 Behaviour & a11y

- role radiogroup(名前 = label)、各 segment role radio + aria-checked。icon-only は radio の名前 = label。
- Tab stop は 1 つ: 選択中 segment(選択中が disabled でもそこ)。値が一致しなければ「自分で disabled でない最初の segment」、それも無ければ先頭。
- ←/→(RTL では反転)で移動し **選択も追従**(selection follows focus)。端で wrap。Home/End。disabled はスキップ。外から Tab で入っただけでは値を変えない。
- クリック: 非選択 segment で onChange。選択中を押しても何もしない。
- 全体 disabled: 何も選べない。Tab stop は選択中 segment に残り disabled と読まれる。`disabledMessage` があれば hover / キーボードフォーカスで上に tooltip、各 segment の description に理由。タップでも理由を開き何も選ばない。
- keyboard hint: 初めてキーボードで(`:focus-visible`)外から入った時だけ、フォーカス segment の下(start 揃え、間隔 8px)に「← → to navigate」。最初の矢印・3000ms・group からのフォーカス離脱で消え、そのインスタンスでは二度と出ない。aria-hidden。§20.3。

### 9.7 Kivact 実装方針

- RESTYLE `kivact::ui::radio_group`: kivact の radio group は「矢印でフォーカス移動と選択、Tab は選択中に入る、disabled はスキップ、RTL で右から」(radio_group.rs のドキュメント)で Astryx と一致。soft の radio group recipes に `soft::Segmented` 分岐を作り、group recipe = track、option recipe = segment、indicator は描かない。
- 研究対象: `crates/kivact-ui-creative/src/choice.rs` + `crates/kivact-ui-creative/src/recipes_controls.rs`(Segmented 分岐の実例)、`crates/kivact-ui-pop/src/choice.rs`。basic は `recipes_controls.rs`(radio)。
- keyboard hint と disabled reason は wrapper 型で追加(§20.3)。`.item(..)` slot で icon+label 行を描き label を ellipsis。
- 微差: kivact の radio policy(`ListPolicy::radio()` は axis None = 4 方向キー)。Astryx は水平矢印のみ。上下でも動く点は許容(軽微、§21)。

### 9.8 受け入れ確認

- [ ] role=radiogroup name=label、radio ×N、checked 状態。
- [ ] Neutral md: 外形 32px、track padding 4、segment 24px、選択は白(dark #262626)・影なし・太字。
- [ ] →/← で選択が移り wrap、Home/End。Tab で入っただけでは値不変。
- [ ] 非選択 hover 5% / 押下 10%。選択切替が 125ms でフェード。
- [ ] 初回キーボード入場で「← → to navigate」が下に 3 秒出て、矢印で消える。2 回目は出ない。
- [ ] disabled + reason: 50%、hover/focus で理由 tooltip、選択不可。
- [ ] fill で等幅、狭幅で label ellipsis。

---

## 10. Toolbar

出典: `Toolbar/Toolbar.tsx`(396 行)、`Toolbar.doc.mjs`、`Toolbar.test.tsx`、`Section/Section.tsx`、`Layout/edgeCompensation.stylex.ts`、`hooks/useListFocus.ts`、`hooks/useKeyboardHint.tsx`。

### 10.1 概要

操作群の横並びバー(カードヘッダーのフィルタ/追加、一括選択バー)。副次(Back)は start、主要(Save)は end。一時的なツールバー(一括選択)は地色や境界で区別。下の内容とは divider か地色で分ける。size はツールバーに 1 回だけ指定し子に継承させる。項目が多ければ MoreMenu へ。アプリ全体のナビゲーションには使わない(TopNav / LayoutHeader)。

### 10.2 Anatomy

```
<Section variant=transparent paddingBlock=2 dividers?>          ← outer chrome(地・境界・padding)
  <div role=toolbar aria-label aria-orientation>                 ← min-height 28px
     center なし: [start slot][end slot]      flex, space-between, align center
     center あり: [start][center][end]        grid 1fr auto 1fr
     {keyboard hint}
  </div>
</Section>
```

```
md, Neutral(padding 12px):
┌──────────────────────────────────────────────────────┐
│  [Back]  [Filter]                    [Cancel] [Save] │ 32px buttons, slot gap 4px
└──────────────────────────────────────────────────────┘ ← dividers=['bottom'] で 1px #00000014
```

### 10.3 API

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `label` | string(必須) | `soft::toolbar(label)` |
| `startContent` / `centerContent` / `endContent` | ReactNode | `.start(|| view)` / `.center(|| view)` / `.end(|| view)`(slot closure `impl FnOnce() -> V`) |
| `size` | sm/md/lg / md | `.size(ControlSize)`。kivact には SizeContext が無く子 Button に継承できない。方針: kit が `soft::provide_control_size(size)` context を出し、soft の recipe が「builder の size が既定のまま」の時に context を使う…は明示 Medium と区別不能なので、当面は catalog/doc で「子に同じ size を付ける」と明記(UNRESOLVED §21) |
| `gap` | SpacingStep / 1(=4px) | `.gap(px: f32)`(既定 4.0) |
| `orientation` | / horizontal | `.orientation(..)` |
| `variant` | `'section' \| 'transparent' \| 'muted'` / transparent | `.variant(soft::SectionVariant::Section \| Transparent \| Muted)` |
| `dividers` | `('top'\|'bottom'\|'start'\|'end')[]` | `.dividers(&[soft::Edge::Bottom, ..])` |

### 10.4 Visual spec

- outer(Section): 地 = section `--color-background-surface` `#ffffff` / `#262626`、transparent、muted `--color-background-muted` `#f1f1f1` / `#1b1b1b`。
- padding: Toolbar は Section に `paddingBlock=2`(8px)を渡すが、**Neutral の `section: {base: {padding: var(--spacing-3)}}` が theme layer(StyleX より上位の `@layer astryx-theme`)で全辺を上書き**するため、Neutral では **全辺 12px**(neutralTheme.ts:371-375)。docsite で要確認(§21)。
- dividers: 指定辺に 1px solid `--color-border` `#00000014` / `#FFFFFF1A`(start/end は論理方向)。
- toolbar 行(Toolbar.tsx:52-104): min-height `--size-element-sm` 28px、align center。center なし: flex + space-between。start のみ → start slot が flex `1 1 0%`。end のみ → end slot が margin-inline-start auto。center あり: grid `1fr auto 1fr`(center slot は justify center、min-width 0)。vertical: column + stretch。
- slot: flex、align center、gap = `gap`(既定 4px)。end slot は justify flex-end。
- edge compensation(Toolbar.tsx:138-142, 344-383): slot の先頭/末尾の子が ghost Button(edge-comp 対象)なら、その側の slot margin を `-(container-padding-inline-start − 8px)` 外へ引く。Section の `--container-padding-inline-start` は Neutral でも 16px のままと読めるので **−8px**(端の ghost ボタンの地が padding 内へ 8px はみ出し、文字位置が内容と揃う)。実測推奨(§21)。
- 子要素サイズ: size=sm/md/lg → 28/32/36px。

### 10.5 Motion

独自アニメーションなし。keyboard hint の 150ms opacity(§9.5)のみ。

### 10.6 Behaviour & a11y

- role toolbar、aria-label、aria-orientation。
- roving(useListFocus, Toolbar.tsx:250-257): Tab stop 1 つ。horizontal ←/→(RTL 反転)、vertical ↑/↓、Home/End、wrap、disabled スキップ。対象 = `button, input, a[href], [tabindex]` のうち最も近い toolbar/popover 祖先がこの toolbar のもの(入れ子 toolbar・popover 内は対象外)。
- caret guard: text input 内の ←/→/Home/End は、キャレットが進行方向の端にあるときだけ toolbar 移動。選択範囲があれば常に input に任せる。
- keyboard hint: horizontal「← →」/ vertical「↑ ↓」+「to navigate」(§20.3)。

### 10.7 Kivact 実装方針

- NEW kit component。`node().role(Role::Toolbar).name(label).orientation(..)`。
- roving は kit で実装: slot 内の任意 focusable 子孫を列挙する公開 API は見当たらない → slot の子を `soft::toolbar_item(view)` で包んで登録してもらう方式(各 item は `focusable(move || active == i)`、root の `on_key` で移動 → `NodeRef::focus()`)。text input の caret guard は caret 位置が取れなければ「input 内では矢印を奪わない」に簡略化(UNRESOLVED)。
- 研究対象: `crates/kivact-ui-basic/src/menubar.rs`(roving)、`crates/kivact-ui-basic/src/button_group.rs`(行レイアウト)。

### 10.8 受け入れ確認

- [ ] role=toolbar name=label、orientation が a11y に出る。
- [ ] Tab 1 回で入り、←→ で項目間移動・wrap、Home/End。
- [ ] start/end の両端寄せ、center は中央固定(1fr auto 1fr)。
- [ ] dividers=bottom で 1px 線。variant muted で地色。
- [ ] 初回キーボード入場で hint。

---

## 11. Badge

出典: `Badge/Badge.tsx`(277 行)、`Badge.doc.mjs`、`Badge.test.tsx`、neutralTheme.ts:283-331。

### 11.1 概要

注目させる必要がある状態(エラー、警告、要対応)だけに付ける小さなラベル。success/warning/error/info は「システム状態」専用(濃い塗り)。色名 variant(blue, purple, teal …)はカテゴリタグ用。1〜2 語。アイコンは文字と併用。全行に同じ badge を付けない、メタデータ(期間・件数・日付)には使わない、クリック可能にしない(読み取り専用)。

### 11.2 Anatomy

```
<span title={string label}>      ← pill: inline-flex, h 20, padding 0 8, radius full, gap 4
   {icon}                        ← 任意(サイズは呼び出し側。未指定 Icon は 20px になるので 12〜16px を指定)
   <span ellipsis>{label}</span>
</span>
```

```
 ( Active )   20px 高、12px/20px weight 500
```

### 11.3 API

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `label` | ReactNode(数値可) | `soft::badge(text)`(`impl IntoText`) |
| `variant` | `neutral info success warning error blue cyan green orange pink purple red teal yellow`(+ Neutral テーマが `gray` も定義)/ neutral | `.variant(soft::BadgeVariant::{Neutral, Info, Success, Warning, Error, Blue, Cyan, Green, Orange, Pink, Purple, Red, Teal, Yellow, Gray})` |
| `icon` | ReactNode | `.icon(view)` |
| (BaseProps) | — | NodeBuilder modifiers |

basic::badge にある `on_press` は **付けない**(Astryx は read-only を明言)。

### 11.4 Visual spec(Badge.tsx:35-180)

- inline-flex、align center、justify center、gap 4px、**height 20px**(`--spacing-5`)、padding 0 / 8px、radius 9999px(pill)。
- font: inherit family、12px、line-height 20px、weight 500。nowrap、max-width 100%、min-width 0、label は ellipsis。label がある場合 min-height = 1 行(文字が大きくなると pill も伸びる)。
- 色(Neutral, 地 / 文字):

| variant | light bg | light text | dark bg | dark text |
|---|---|---|---|---|
| neutral(Neutral 上書きで gray と同じ) | `#e2e2e2` | `#1b1b1b` | `#303030` | `#d4d4d4` |
| gray | `#e2e2e2` | `#1b1b1b` | `#303030` | `#d4d4d4` |
| info | `#0074e2` | `#ffffff` | `#6d9cfe` | `#111111` |
| success | `#198100` | `#ffffff` | `#64af4c` | `#111111` |
| warning | `#ffce2f` | `#111111` | `#ffce2f` | `#111111` |
| error | `#c9303a` | `#ffffff` | `#ff705d` | `#111111` |
| red / orange / yellow / green / teal / cyan / blue / purple / pink | §1.1 装飾色表の background | 同 text | 同 | 同 |

- 状態なし(非インタラクティブ)。アイコン色は文字色を継承。forced-colors: semantic variant に 1px CanvasText outline(inset −1px)。

### 11.5 Motion

なし。

### 11.6 Behaviour & a11y

- ただの文字(generic)。label が読まれる。label が文字列/数値なら title 属性(hover で全文)。kivact では名前を常にフル label にし、hover 全文表示は任意(tooltip を付けるなら truncate 時のみが理想 — Astryx も未実装 #5585)。

### 11.7 Kivact 実装方針

NEW kit component(`row` + text)。研究対象 `crates/kivact-ui-basic/src/badge.rs`(variant enum と NodeBuilder の作法)。

### 11.8 受け入れ確認

- [ ] 高さ 20、pill、12px medium。15 variant の色が light/dark で上表どおり。
- [ ] 長い label が親幅で ellipsis、名前はフル。
- [ ] フォーカスが当たらない、押下できない。

---

## 12. Banner

出典: `Banner/Banner.tsx`(806 行)、`Banner.doc.mjs`、`Banner.test.tsx`、`Collapsible/useCollapsible`。

### 12.1 概要

info / warning / error / success の持続的な通知(自動で消えない)。card はページ内容中、section はページ全幅。info/success は dismissable に、error は直すまで残す。タイトルは短く(「Payment failed」)。自動で消える短命メッセージは Toast。同 status の banner を積まない。children は既定で折りたたみ(toggle 付き)、行動に必要な内容なら `collapsible={false}`。status は色とアイコンだけで live region を作らない(非同期結果は useAnnounce で別途、必要時のみ role=status、緊急のみ role=alert)。色/アイコンだけに意味を持たせず、タイトル文で status を言う。

### 12.2 Anatomy

```
<div banner-frame>                                     ← flex column。elevation 影(card+elevated なら radius 12)
  <div banner (header) status地>                       ← flex wrap, align start(1 行 & actions なら center)
     <div iconWrapper aria-hidden> Icon(md 20px, inherit) </div>
     <div headerContent flex1 minW0 [basis 8rem if endContent]>
        <div title>  14/20 600 </div>
        <div description> 12/20 400 </div>             ← 任意
     </div>
     <div endArea ml:auto mb:-4px gap 8 wrap>           ← endContent / toggle / dismiss
        {endContent}  [ghost sm icon-only chevron toggle]  [ghost sm icon-only close]
     </div>
  </div>
  <div banner-content card地 border(左右下)>            ← children(展開時のみ)
</div>
```

```
card, info, dismissable, with content (expanded):
╭───────────────────────────────────────────────────────╮
│ ⓘ  New update available              [Retry] [⌃] [✕] │  header: info-muted 地, padding 12/16, 上 2 角 radius 12
│    Version 2.0 is ready to install                    │
├───────────────────────────────────────────────────────┤
│  • detail list …                                      │  content: card 地, 1px border(左右下), 下 2 角 radius 12
╰───────────────────────────────────────────────────────╯
```

### 12.3 API

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `status` | `'info' \| 'warning' \| 'error' \| 'success'`(必須) | `soft::banner(status: soft::BannerStatus, title)`(`enum BannerStatus { Info, Warning, Error, Success }`。`Tone` からの `From` も用意: Danger→Error) |
| `title` | ReactNode(必須) | 第 2 引数 `impl IntoText` |
| `description` | ReactNode | `.description(text)` |
| `icon` | ReactNode / status 既定 | `.icon(view)` |
| `isDismissable` | / false | `.dismissable(true)`(内部 dismissed 状態で自分で消える) |
| `onDismiss` | () => void | `.on_dismiss(f)` |
| `dismissLabel` | string | `.dismiss_label(text)` |
| `endContent` | ReactNode | `.end(|| view)`(slot closure) |
| `container` | `'card' \| 'section'` / card | `.container(soft::BannerContainer::Card \| Section)` |
| `elevation` | / none | `.elevation(soft::Elevation)` |
| `collapsible` | `bool \| {defaultIsOpen} \| {isOpen, onOpenChange}` / true(閉で開始) | `.collapsible(false)` / `.open(binding: Binding<bool>)`(caller-owned。省略時は内部 signal false 開始) |
| `children` | ReactNode | `.content(|| view)` |
| `role`(BaseProps) | — | `.role(Role::Status \| Role::Alert)`(任意) |

### 12.4 Visual spec(Banner.tsx:228-441)

- frame: flex column。elevation≠none で `--shadow-*`、card かつ elevated なら radius 12px。
- header: flex、wrap、align-items flex-start(description なし かつ actions ありの 1 行時は center)、column-gap 8px、row-gap 12px、padding **12px / 16px**。
  - card: content 非表示時は 4 角 `--radius-container` **12px**、content 表示時は上 2 角のみ。section: radius 0。
  - 地 / 前景(status, Neutral):

| status | light bg | light fg | dark bg | dark fg |
|---|---|---|---|---|
| info | `#b9d7ff` | `#003978` | `#88bcff3D` | `#a1caff` |
| warning | `#fae19e` | `#4b3900` | `#e2b6233D` | `#eec448` |
| error | `#ffc4be` | `#76000c` | `#ff98903D` | `#ffaea7` |
| success | `#bce0bb` | `#00490b` | `#90ca903D` | `#90ca90` |

  - 枠線なし(左アクセント線も無い)。dark の地は 24% アルファで背後と合成。
- icon: 既定 `info`(Lucide `info`)/ `warning`(`triangle-alert`)/ `error`(`circle-x`)/ `success`(`circle-check`)、**20px**、色 = header fg。aria-hidden。
- title: 14px / 20px / **600**、色 = status fg(inherit)、`overflow-wrap: anywhere`。
- description: 12px / 20px / 400、色 = status fg(status 時は secondary にならない)。title との間隔 0。
- endContent がある場合 headerContent の flex-basis 8rem(128px)。狭いと endArea 全体が次行へ折り返す(row-gap 12px)。
- endArea: flex、align center、wrap、justify flex-end、gap 8px、shrink 0、max-width 100%、margin-inline-start auto、**margin-block −4px**(28px の sm ボタンを 20px 行に収めヘッダー高を増やさない)。文字色は `--color-text-primary` に戻す(consumer の endContent 用)。edge-comp inset 8px(端の ghost ボタンを 8px 外へ)。
- toggle / dismiss: ghost、size sm(28×28)、icon-only、icon 16px(`chevron-down` / `x`)、文字色は status fg(`--color-text-primary` を on-muted 色で上書き)。hover/pressed は ghost の overlay。
- content area: 地 `--color-background-card` `#ffffff` / `#1b1b1b`、padding 12px / 16px、border 1px `--color-border` `#00000014` / `#FFFFFF1A`(左右と下。上は header と接する)、card なら下 2 角 12px。

### 12.5 Motion

| trigger | property | from → to | duration | easing | file:line |
|---|---|---|---|---|---|
| 展開 / 折りたたみ | chevron icon rotation | 0° ↔ 180° | 125ms | `cubic-bezier(0.24, 1, 0.4, 1)` | Banner.tsx:358-368。reduced motion 0s |
| content 表示 / 非表示 | — | **即時**(高さアニメなし) | — | — | Banner.tsx:639, 792-801 |
| dismiss | — | 即時に消える(フェードなし) | — | — | Banner.tsx:590-592 |
| 内部ボタン | §2.5 | | | | |

### 12.6 Behaviour & a11y

- root は role を持たない(status 色は視覚のみ)。caller が role=status / alert を付けられる。
- toggle: 名前・tooltip = 展開時「Collapse」/ 折りたたみ時「Expand」、`aria-expanded`、`aria-controls`(content 表示時のみ)。
- dismiss: tooltip「Dismiss」、名前「Dismiss {title}」(title が文字列のとき。違えば「Dismiss」)。`dismissLabel` が両方を置換。押すと自分を消して onDismiss、**フォーカスを banner に入る前の要素へ戻す**(focus / pointerdown を capture で記録, Banner.tsx:596-625)。kivact で直前フォーカスを覚える公開手段が無ければ「次の focusable へ」— UNRESOLVED。
- collapsible=false: toggle なし、aria-expanded なし、content 常時表示。

### 12.7 Kivact 実装方針

- NEW kit component。header は wrap する row、toggle/dismiss は `kivact::ui::button(..).variant(ButtonVariant::Ghost).size(ControlSize::Small).icon(..).hide_label(true)` + tooltip。status fg を子ボタンに渡すため kit Variant `soft::OnStatus(BannerStatus)` を button に付けて recipe が文字色を差し替える。
- 開閉は `kivact::interaction::use_disclosure`(expanded / controls が揃う)。見た目は自前。
- 研究対象: `crates/kivact-ui-basic/src/alert.rs`(icon / title / description / action 配置)。

### 12.8 受け入れ確認

- [ ] 4 status の地/文字/アイコンが light/dark で一致。左線・枠なし。
- [ ] padding 12/16、角丸 12(card)、section は角なし。
- [ ] children あり: 既定で閉、「Expand」で開き chevron が 125ms で 180° 回転、「Collapse」に変わる。content は白地(dark `#1b1b1b`)+ 1px 枠。
- [ ] dismiss: 名前「Dismiss {title}」、押すと消えフォーカスが元へ。
- [ ] endContent 付きで狭幅にすると actions が下の行に折り返す。

---

## 13. ProgressBar

出典: `ProgressBar/ProgressBar.tsx`(788 行)、`ProgressBarTouch.ts`、`ProgressBarMarkTooltip.tsx`、`ProgressBar.doc.mjs`、`ProgressBar.test.tsx`、`motion/uikit/progressBar.ts`、`motion/uikit/timing.ts`、neutralTheme.ts:359-363。

### 13.1 概要

確定(量が分かる)/ 不確定(分からない)の進捗。色は文脈に合わせる(accent 一般、success 完了、warning/error 警告)。label は必須(隠しても SR 用に残す)。アイコンや文字をバーの中に入れない(外にレイアウトで並べる)。一瞬で終わる処理には使わない。同じ処理に複数本を重ねない(1 本 + value label)。

### 13.2 Anatomy

```
<div container column gap 4, width 100%, min-width 48>
  <div header flex space-between baseline>               ← label か value label を出す時
     <span id=label>{label}</span>    <span>{75%}</span>
  </div>                                                  ← 両方隠すなら VisuallyHidden label のみ
  <div trackContainer relative>
     <div role=progressbar aria-valuenow/min/max/valuetext aria-labelledby>  ← track 8px, radius full, overflow hidden
        <div fill/>                                       ← 確定: 幅 100% を translateX で送る / 不確定: 幅 40% が往復
     </div>
     <span role=img aria-label tabIndex=0 mark/> ...      ← marks(track の外、絶対配置、tooltip)
  </div>
</div>
```

```
 Upload progress                                  75%
 ████████████████████████████████████░░░░░░░░░░░░░░░    8px track, radius full
                                 ▏← mark 2×8px
```

### 13.3 API

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `value` | number / 0(不確定時無視) | `soft::progress_bar(label, value: impl IntoValue<f64>)`。内部で `kivact::ui::progress(Some(value / max))` |
| `max` | number / 100 | `.max(f64)` |
| `label` | string(必須) | 第 1 引数(progress の name。表示もする) |
| `isLabelHidden` | / false | `.label_hidden(true)` |
| `hasValueLabel` | / false | `.value_label(true)` |
| `formatValueLabel` | `(v, max) => string` / `round(v/max*100)%` | `.format_value(|v, max| String)`(aria-valuetext にも使う) |
| `variant` | `accent success warning error neutral` / accent | `.variant(Tone::Accent \| Success \| Warning \| Danger \| Neutral)`(kivact progress は `Tone` を Variant として受ける) |
| `isIndeterminate` | / false | `.indeterminate(true)`(= `progress(None)`) |
| `marks` | `{value, label}[]` | `.marks(vec![soft::ProgressMark::new(80.0, "Goal")])` |
| `isDisabled` | / false | `.disabled(..)`(見た目のみ: 色を disabled に) |

素の `kivact::ui::progress(..)` も soft recipe で同じ track/fill を描く(label 行と marks は wrapper の追加分)。

### 13.4 Visual spec(ProgressBar.tsx:194-511)

- container: column、gap 4px、width 100%、min-width 48px。
- label: 14px / 20px / 500、`--color-text-primary` `#000000` / `#ffffff`(disabled: `--color-text-disabled` `#919191` / `#525252`)。
- value label: 14px / 20px / 400、`--color-text-secondary` `#474747` / `#9e9e9e`(disabled: disabled 色)。header は space-between / baseline 揃え。注: isLabelHidden + value label の場合 label は絶対配置で隠れ、value label が 1 つだけ残るので **左寄せ**になる(space-between の単独要素)。
- track: height **8px**、width 100%、radius full、overflow hidden、地 `--color-background-muted`(Neutral は progress-bar で `--color-border-emphasized` に差し替え)= **`#d4d4d4` / `#525252`**。
- fill 色(variant): accent `#1b1b1b` / `#f1f1f1`、success `#198100` / `#64af4c`、warning `#ffce2f`、error `#c9303a` / `#ff705d`、neutral = `--color-text-disabled` `#919191` / `#525252`、disabled(isDisabled 時は variant に関わらず)= `#919191` / `#525252`。fill も radius full(先端が丸い)。
- 確定 fill: 幅 100% の帯を `translateX(fill% − 100%)` で置く(RTL: `translateX(100% − fill%)`)。値 0 でも(マウス/トラックパッド環境では)帯は track 外にあり見えない。
- 不確定 fill: 幅 **40%**、radius full、§13.5 のスライド。reduced motion: 幅 100% で opacity パルス。
- marks(ProgressBar.tsx:422-511): 縦棒 **2px × 8px**、track の中心に center 揃え(`translate(-50%,-50%)`、RTL は `translate(50%,-50%)`)、`inset-inline-start: pct%`。色: fill の上(値 >0 かつ mark% ≤ 描画%)では variant の on 色(accent → `--color-on-accent` `#ffffff` / `#111111`、success/warning/error → それぞれの on 色、neutral → text-primary、disabled → text-secondary)、track の上では `--color-text-primary` `#000000` / `#ffffff`(disabled 時 text-secondary)。focus-visible で 2px accent リング(offset 3px)。hover / focus で label の tooltip。
- タッチ(coarse pointer)+ marks あり: 値 >0 なら fill は最低 16px(= 8px × 2)描く。値 0 の fill は opacity 0。

### 13.5 Motion

| trigger | property | from → to | duration | easing | 備考 / file:line |
|---|---|---|---|---|---|
| value 変化(マウス環境) | fill transform(位置) | 旧% → 新% | `--duration-medium` **300ms** | `cubic-bezier(0.24, 1, 0.4, 1)` | ProgressBar.tsx:326-354。reduced motion 0s |
| value 変化(coarse pointer) | 同上 | 旧 → 新 | `1000ms × |Δ|/100`(track 全長を 1s で進む速度) | **linear**(`progress.curves.fill = [0,0,1,1]`) | ProgressBarTouch.ts、motion/uikit/progressBar.ts。reduced 0s |
| 初回表示 | — | アニメなし(CSS は初期値から transition しない)。kivact の `ProgressState::shown` は「0 から value へ動く」ので **初回は即値にする**か許容(UNRESOLVED) | | | |
| 不確定 | fill translateX | −100% → +250%(自幅基準。track 基準で左端 −40% → 100%、帯が左外から右外へ抜ける) | **1.5s** 無限 | CSS `ease-in-out` `cubic-bezier(0.42, 0, 0.58, 1)` | ProgressBar.tsx:156-175, 361-404。RTL: +100% → −250% |
| 不確定 + reduced motion | opacity | 1 → 0.84(alternate で往復) | **2000ms**/片道 | `cubic-bezier(0.42, 0, 0.58, 1)` | 幅 100%。ProgressBar.tsx:181-188, 535-547 |

### 13.6 Behaviour & a11y

- track が role progressbar。確定: aria-valuenow = clamp(value, 0, max)、valuemin 0、valuemax max、valuetext = format(既定「75%」)。不確定: value 系属性なし。名前 = label(labelledby)。
- value / max の非有限値は 0 扱い、max ≤ 0 なら 0%。
- marks は progressbar の外で個別にフォーカス可能(role img + name = label)、hover/focus で tooltip。不確定では marks を出さない。
- 非インタラクティブ(marks 以外)。

### 13.7 Kivact 実装方針

- RESTYLE `kivact::ui::progress`(recipes `track` / `fill`、`ProgressState { value, size, shown, phase }`)+ kit wrapper(label 行・value label・marks)。
- 確定 fill: paint で `shown` を読んで幅 = shown × track 幅、radius full。kivact-ui の `shown` の移動時間・曲線が 300ms / ease-standard でない場合は wrapper 側で `animated(value, Transition::new(300ms, CubicBezier(0.24,1,0.4,1)))` を作り、それで描く。
- 不確定: `phase`(約 1.3s 周期の 0..1)を使うと周期が Astryx(1.5s)と違う。正確にするなら kit の `frame_loop` で 1.5s 周期の位相を作り、`x = -0.4 + 1.4 × bezier_ease_in_out(t)`(track 比)で 40% 幅の帯を描く。reduced motion では kivact の phase が止まるので、kit の frame_loop で 2s alternate の opacity パルスを描く(essential motion 扱いにできるかは kivact の frame_loop の reduced-motion 規則次第 — UNRESOLVED)。
- 研究対象: `crates/kivact-ui-basic/src/recipes.rs` の progress recipe、`crates/kivact-ui/src/progress.rs`。

### 13.8 受け入れ確認

- [ ] role=progressbar、name=label、value 0..max、valuetext「75%」。不確定では value なし。
- [ ] track 8px `#d4d4d4` / `#525252`、fill 黒/白(accent)、両端丸。
- [ ] 値変更で 300ms ease-standard で伸縮。
- [ ] 不確定: 40% 帯が 1.5s で左から右へ抜け続ける。reduced motion では全幅が 1↔0.84 で脈動。
- [ ] marks: 2×8px、fill 上は白(dark 黒)、track 上は黒(dark 白)、Tab で到達し tooltip。
- [ ] disabled: fill とラベルが disabled 色。

---

## 14. Skeleton

出典: `Skeleton/Skeleton.tsx`(252 行)、`Skeleton.doc.mjs`、`Skeleton.test.tsx`。

### 14.1 概要

読み込み中のレイアウト保持プレースホルダ。読み込む内容の大きさと形に合わせる。複数並べるときは `index` で波状にずらす。寸法が分からないなら Spinner。同じ領域で Spinner と併用しない。いつまでも出さない(長引けばエラー/空状態)。

### 14.2 Anatomy

`<div aria-hidden="true">`(中身なし、1 要素)。

### 14.3 API

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `width` / `height` | number(px) \| string / '100%' | `soft::skeleton().width(px).height(px)` または `.style(..)`(既定は親いっぱい) |
| `radius` | `'none' \| 0 \| 1 \| 2 \| 3 \| 4 \| 'rounded'` / 3 | `.radius(soft::SkeletonRadius::None \| R0 \| R1 \| R2 \| R3 \| R4 \| Rounded)` |
| `index` | number / 0 | `.index(n)`(遅延 = 1000 + 100×n ms) |

### 14.4 Visual spec(Skeleton.tsx:78-138)

- 地 `--color-skeleton` **`#f1f1f1` / `#525252`**、静止時 opacity **0.25**。
- prefers-contrast: more → 地 = `color-mix(skeleton, text-primary 30%)` = `#a9a9a9` / `#868686`。forced-colors: GrayText、opacity 1。
- radius: none / 0 → 0px、1 → `--radius-inner` 6px、2 → `--radius-element` 10px、3(既定)・4 → `--radius-container` 12px、rounded → 9999px。(doc コメントの「4 = 16px」は誤り。コードは 4 も container)
- 寸法既定 100% × 100%。

### 14.5 Motion(Skeleton.tsx:26-112)

| trigger | property | keyframes | duration | easing / iteration | delay |
|---|---|---|---|---|---|
| マウント後 | opacity | 0.25 → 0.325 → 0.4 → 0.475 → 0.55 → 0.625 → 0.7 → 0.775 → 0.85 → 0.925 の **10 段の階段**(各段 10% 区間保持、段の切替は 0.01% の瞬間ランプ = 実質ステップ) | `--duration-medium-max` **400ms**(片道) | linear、`alternate`(上がって下がる)、無限 | **1000ms + 100ms × index**(遅延中は 0.25 のまま) |
| reduced motion | — | アニメなし(0.25 で静止) | | | |

周期 = 800ms(400 上り + 400 下り)。kivact 実装: `frame_loop` で t を取り `step = floor(((t−delay) mod 800 を三角波化) / 40ms)`、opacity = 0.25 + 0.075 × step(0..9)。または `Easing::steps_end(10)` 相当の Transition を alternate/repeat で(kivact の `Transition::repeat` + `alternate` + `Easing::Steps` が使えればそれが最短)。

### 14.6 Behaviour & a11y

aria-hidden(読み上げない)。読み込み中であることは周囲(例: `aria-busy`)で伝える。

### 14.7 Kivact 実装方針

NEW kit component(1 ノード)。研究対象 `crates/kivact-ui-basic/src/primitives.rs`(`skeleton`: 「reduced motion では不透明で静止」の実装と Transition の decorative 指定)。basic と違い **reduced motion 時は 0.25 で静止**、`rows` は持たない。

### 14.8 受け入れ確認

- [ ] 1 秒待ってから 10 段の明滅開始(0.25↔0.925)、800ms 周期。index でずれる。
- [ ] light は非常に淡い(`#f1f1f1` × 0.25)、dark は `#525252` × 0.25〜0.925。
- [ ] a11y ツリーに出ない。reduced motion で静止。

---

## 15. Spinner

出典: `Spinner/Spinner.tsx`(636 行)、`Spinner.doc.mjs`、`Spinner.test.tsx`。

### 15.1 概要

寸法の分からない読み込み・処理中表示。何を読み込んでいるか label で説明。暗い/アクセント地の上では `onMedia`。寸法が分かる領域は Skeleton。同一画面に複数並べない(全体で 1 つ)。

### 15.2 Anatomy

```
label なし: <span role=status aria-label="Loading"> <svg aria-hidden> <circle track/> <circle arc/> </svg> </span>
label あり: <div column center gap 8> <span role=status aria-labelledby=labelId>svg</span> <Text id body bold>{label}</Text> </div>
```

### 15.3 API

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `size` | `'sm' \| 'md' \| 'lg' \| 'xl'` / md | `soft::spinner().size(soft::SpinnerSize::Sm \| Md \| Lg \| Xl)` |
| `shade` | `'default' \| 'subtle' \| 'onMedia' \| 'inherit'` / default | `.shade(soft::SpinnerShade::Default \| Subtle \| OnMedia \| Inherit)` |
| `label` | ReactNode | `.label(text)`(下に表示し名前にもなる)/ リッチは `.label_view(|| view)` + `.name(..)` |
| `aria-label` | string / "Loading" | `.name(..)`(既定 `soft.spinner.loading`) |

### 15.4 Visual spec(Spinner.tsx:47-431)

| size | 直径 | stroke | 箱(直径 + stroke×2) |
|---|---|---|---|
| sm | 10px | 2px | 14px |
| md | 14px | 3px | 20px |
| lg | 18px | 3px | 24px |
| xl | 28px | 4px | 36px |

- 2 つの円(同心、同じ stroke 幅): track(全周)と arc(弧長 = 円周 × **0.375** = 135°)。stroke-linecap **round**、fill none。arc は 12 時から開始(rotate −90°)。
- shade の色:

| shade | arc | track | track 不透明度 |
|---|---|---|---|
| default | `--color-accent` `#1b1b1b` / `#f1f1f1` | `--color-track` `#CCD3DB` / `#5A5E66`(Neutral 未上書き) | 1 |
| subtle | `--color-text-secondary` `#474747` / `#9e9e9e` | `--color-track` 同上 | 1 |
| onMedia | `--color-on-dark` `#ffffff` | `#ffffff` | 77/255 ≈ 0.302 |
| inherit | currentColor | currentColor | 0.3 |

- label あり: column、align center、gap 8px、label は Text body bold = 14px / 20px / **700**、`--color-text-primary`。
- 縦位置 vertical-align middle、flex-shrink 0。

### 15.5 Motion(Spinner.tsx:212-354)

| trigger | property | from → to | duration | easing | 備考 |
|---|---|---|---|---|---|
| 常時 | arc の stroke-dashoffset(= 弧が円周上を進む) | 0 → −円周(1 周) | `--duration-slow-min` **525ms** | linear、無限 | **時計回り**。円全体を回転させず弧だけを進める(WebKit の丸キャップ揺れ対策)。native では「弧の開始角 = −90° + 360° × t」で描けばよい |
| reduced motion | 同上 | | **3s**/周(止めない) | linear | role=status と名前で busy を伝える |
| 位相同期 | — | 画面上の全 Spinner の回転位相を揃える(animation.startTime = 0 = 文書タイムライン原点) | | | kivact: 角度を「マウント時刻」ではなく共通時計(例 `frame_loop` の絶対時刻 mod 525ms)から求める |

### 15.6 Behaviour & a11y

role status、名前 = aria-label ?? 文字列 label ?? 「Loading」。文字列 label 表示時は labelledby で二重読みを避ける。非インタラクティブ。

### 15.7 Kivact 実装方針

- NEW kit component(paint で 2 円弧)。研究対象 `crates/kivact-ui-basic/src/primitives.rs`(`spinner`: essential motion `Transition::decorative(false)` の指定)。
- Button の busy part(`button_busy` recipe)は同じ描画関数(sm, inherit)を使う。ただし kivact::ui::button の busy は 0.7s/周の回転が固定(§2.7, §21)。

### 15.8 受け入れ確認

- [ ] role=status name="Loading"(既定)。label 付きは label が名前。
- [ ] md: 20px 箱、135° の弧が 525ms で時計回り 1 周、丸キャップ、track は `#CCD3DB` / `#5A5E66`。
- [ ] 4 shade の色。reduced motion で 3s/周に減速(停止しない)。
- [ ] 複数の spinner が同位相で回る。

---

## 16. StatusDot

出典: `StatusDot/StatusDot.tsx`(257 行)、`StatusDot.doc.mjs`、`StatusDot.test.tsx`。

### 16.1 概要

オンライン/オフライン等を示す 8px の色点。色だけでは伝わらない(WCAG 1.4.1)ので、二値の有無として使う・可視 label と並べる・status ごとに違う icon を入れる・別の場所で伝える、のいずれかで補う。`label` は必須(SR 用)。pulse は即時の注意が必要な状態だけ(装飾目的禁止)。

### 16.2 Anatomy

`<span role=img aria-label={label}> [<span aria-hidden icon 8×8>] </span>` + 任意 Tooltip(hover)。

### 16.3 API

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `variant` | `'success' \| 'warning' \| 'error' \| 'accent' \| 'neutral'`(必須) | `soft::status_dot(variant: soft::StatusDotVariant, label)` |
| `label` | string(必須) | 第 2 引数(名前) |
| `isPulsing` | / false | `.pulsing(true)` |
| `tooltip` | string | `tooltip(text).attach(dot)` |
| `icon` | ReactNode | `.icon(view)`(8px 枠内、currentColor = ink) |

### 16.4 Visual spec(StatusDot.tsx:38-136)

- 8×8px 円(radius 50%)、inline-flex center、shrink 0。
- 色(plate / ink):

| variant | light plate | light ink | dark plate | dark ink |
|---|---|---|---|---|
| success | `#198100` | `#ffffff` | `#64af4c` | `#111111` |
| warning | `#ffce2f` | `#111111` | `#ffce2f` | `#111111` |
| error | `#c9303a` | `#ffffff` | `#ff705d` | `#111111` |
| accent | `#1b1b1b` | `#ffffff` | `#f1f1f1` | `#111111` |
| neutral | `--color-icon-secondary` `#6a6a6a` | surface `#ffffff` | `#9e9e9e` | surface `#262626` |

- icon は 8×8 の枠に描く(line-height 0)。forced-colors: success/warning/error に 1px CanvasText outline。

### 16.5 Motion

| trigger | property | keyframes | duration | easing | reduced motion |
|---|---|---|---|---|---|
| isPulsing | opacity | 1 → 0.5(50%)→ 1 | **2s** 無限 | CSS `ease-in-out` `cubic-bezier(0.42, 0, 0.58, 1)`(各区間に適用 = 1→0.5 を 1s、0.5→1 を 1s) | アニメ無し(不透明で静止)。StatusDot.tsx:32-64 |

### 16.6 Behaviour & a11y

role img、名前 = label。フォーカス不可(tooltip は hover のみで届く)。

### 16.7 Kivact 実装方針

NEW kit component(1 ノード、`Role::Image`)。pulse は decorative Transition(repeat Forever + alternate、各 1s)。研究対象 `crates/kivact-ui-basic/src/badge.rs`。

### 16.8 受け入れ確認

- [ ] role=img name=label、8px 円、5 variant の色(light/dark)。
- [ ] pulsing: 2s で 1→0.5→1。reduced motion で停止。

---

## 17. Indicator

出典: `Indicator/CheckboxIndicator.tsx`(259 行)、`CheckIndicator.tsx`(176 行)、`RadioIndicator.tsx`(199 行)、`types.ts`、`indicatorRegistry.ts`、`useIndicator.ts`、`indicator.markers.stylex.ts`、`Indicator.doc.mjs`、`Indicator.test.tsx`、`hooks/useIndicatorFocusRing.tsx`。

### 17.1 概要

「状態を絵にする」装飾部品。チェックボックスの箱、ラジオの円、選択行のチェックマーク。所有コンポーネント(CheckboxInput, RadioListItem, listbox option, menu の選択項目)が input・role・名前・フォーカス・キーを持ち、indicator は aria-hidden の絵だけ。family: singleSelection(unchecked / checked)、multiSelection(+ indeterminate)。テーマで restyle(`checkbox-indicator` 等の target)または差し替え(`indicators: {check: RadioIndicator}` で全単一選択マークをラジオに)。hover/pressed は prop ではなく所有者の祖先マーカー(CSS `indicatorScope`)で届く。所有者は全状態で常に indicator を描画し state を渡す。

### 17.2 Anatomy

```
CheckboxIndicator: <span aria-hidden box> <svg check 10×10 viewBox/> <span dash/> </span>  (children で中身差し替え可)
RadioIndicator:    <span aria-hidden circle> [<span dot/>] </span>
CheckIndicator:    checked → <Icon check sm aria-hidden/>、unchecked → 何も描かない(children あり → 16×16 slot)
```

### 17.3 API

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `state` | `'unchecked' \| 'checked' \| 'indeterminate'`(radio/check は前 2 つ) | recipe が受ける `CheckState`(`CheckedState`)/ `ListItemState` / `MenuItemState` の checked |
| `size` | `'sm' \| 'md'` / md | `ControlSize::Small` → sm、`Medium`/`Large` → md(Astryx は 2 段のみ。Large は md を使う) |
| `isDisabled` | / false | state の disabled |
| `children` | ReactNode | 置換コンテンツ。kivact では kit の paint 関数を差し替える形(`soft::IndicatorPainter` を Tokens に持たせ theme で差し替え可能に)— 任意 |

公開 API としての indicator コンポーネントは必須ではない。kit 内部に `soft::paint::checkbox_mark(state, size, disabled, hovered)` / `radio_mark(..)` / `check_mark(..)` を持ち、checkbox / radio_group / menu / select / list の recipe から呼ぶ。

### 17.4 Visual spec

CheckboxIndicator(CheckboxIndicator.tsx:25-157):
- 箱: sm **20×20** / md **24×24**、border 1px、radius `--radius-inner` **6px**、border-box。
- unchecked: border `--color-border-emphasized` `#d4d4d4` / `#525252`、地 `--color-background-surface` `#ffffff` / `#262626`。
  - 所有者 hover(hover 可能ポインタ): border `mix(border-emphasized, tint 20%)` = `#aaaaaa` / `#757575`、地 `mix(surface, tint 5%)` = `#f2f2f2` / `#313131`。
- checked / indeterminate: border = 地 = `--color-accent` `#1b1b1b` / `#f1f1f1`。hover: `mix(accent, tint 15%)` = `#171717` / `#f3f3f3`。
- checkmark: viewBox 0 0 10 10、path `M8.5 2.5 L4 7.5 L1.5 5`、stroke 1.5(viewBox 単位。描画 sm 12px → 1.8px、md 14px → 2.1px)、round cap / join、色 `--color-on-accent` `#ffffff` / `#111111`。サイズ sm **12×12** / md **14×14**、中央。
- indeterminate bar: sm **10×2** / md **12×2**、radius full、地 on-accent。
- disabled: opacity 0.5、border `--color-border` `#00000014` / `#FFFFFF1A`(hover でも不変)。disabled かつ unchecked の地 `--color-background-muted` `#f1f1f1` / `#1b1b1b`。
- forced-colors: mark は CanvasText。

RadioIndicator(RadioIndicator.tsx:25-124):
- 円: sm **20** / md **24**、border 1px、radius full。色・hover・disabled は Checkbox と同じ規則(unchecked / checked / disabled / disabledUnchecked)。
- dot(checked 時のみ): sm **8×8** / md **10×10**、radius full、地 on-accent `#ffffff` / `#111111`、中央。

CheckIndicator(CheckIndicator.tsx:41-173):
- checked: Icon `check`(Lucide `check`)**16px**(sm/md とも `sm`)、色 `--color-accent` `#1b1b1b` / `#f1f1f1`、disabled は `--color-icon-disabled` `#919191` / `#525252`。
- unchecked: 何も描かない(場所も取らない — 所有者が列幅を確保する)。
- children 指定時: 16×16 の slot(色 accent/disabled)。

フォーカス(useIndicatorFocusRing): 隠れた input が `:focus-visible` のとき、所有者が **indicator 要素そのもの**に 2px accent outline(offset 3px)を描く。よってリングは indicator の角丸(箱 6px / 円)に沿う。

### 17.5 Motion

| trigger | property | duration | easing | file:line |
|---|---|---|---|---|
| 状態・hover 変化 | 箱/円の background-color, border-color | 125ms | `cubic-bezier(0.24, 1, 0.4, 1)` | CheckboxIndicator.tsx:35-40、RadioIndicator.tsx:39-44。reduced motion 0s |
| checkmark / dash / dot の出現 | — | 即時(display 切替。描画アニメ無し) | — | |

### 17.6 Behaviour & a11y

常に aria-hidden。role・名前・focus・tabIndex を持たせてはいけない(二重読み・hidden 内 focusable 違反)。所有者が全て持つ。

### 17.7 Kivact 実装方針

- RESTYLE: `kivact::ui::checkbox` の recipe(`CheckState`: checked / indeterminate / size / invalid、`PartState` の hover・focus_visible)で箱を描く。radio_group の option indicator、menu の `menu-check` / `menu-radio`、select/list の選択マークも同じ paint 関数。
- hover は所有者(行全体)の hover を使う: kivact の recipe state は所有者の PartState なので自然に一致。
- checkmark パスを正確に出すため、theme の icon set に Astryx のパスを持つ `check` 系アイコンを登録するか、paint でパスを直接描く。CheckIndicator の 16px check は Lucide `check`。
- 研究対象: `crates/kivact-ui-basic/src/recipes_controls.rs`(checkbox / radio recipe)、`recipes_menu.rs`(menu mark)。

### 17.8 受け入れ確認

- [ ] checkbox md 24px / sm 20px、radius 6、unchecked 灰枠白地、checked 黒地白チェック(dark: 白地黒チェック)、indeterminate 横棒。
- [ ] 行 hover で枠が濃く(`#aaaaaa` / `#757575`)。125ms で色遷移。
- [ ] radio: 円 + dot 10px(md)。
- [ ] キーボードフォーカスで indicator の形に沿った 2px リング。
- [ ] disabled: 50%、淡い枠。

---

## 18. Kbd

出典: `Kbd/Kbd.tsx`(229 行)、`Kbd.doc.mjs`、`Kbd.test.tsx`。

### 18.1 概要

キーボードショートカットの表示(tooltip、メニュー項目、説明文)。操作の近くに置く。`ctrl`/`cmd` ではなく `mod` を使う(プラットフォームで ⌘ / Ctrl に切替)。ショートカットを唯一の発見手段にしない。

### 18.2 Anatomy

```
<span role=img aria-label="Command + K">  ← inline-flex, gap 4
   <kbd aria-hidden>⌘</kbd> <kbd aria-hidden>K</kbd>
</span>
```

```
 ┌──┐ ┌──┐
 │⌘ │ │K │   20px 高、min-width 20、radius 6、下辺だけ 2px の濃い線
 └━━┘ └━━┘
```

### 18.3 API

| prop | 型 | Kivact |
|---|---|---|
| `keys` | string(`"mod+shift+p"`。`+` 区切り、`plus` で `+` キー、別名 `esc`→escape / `return`→enter) | `soft::kbd(keys: &str)`(同じ文法をパース) |

basic の `kbd(text)` / `kbd_group` と違い、1 つの関数で複数キー + アクセシブル名を作る。

### 18.4 Visual spec(Kbd.tsx:39-65)

- wrapper: inline-flex、align center、gap 4px、shrink 0。
- 各キー: inline-flex center、**min-width 20px、height 20px**(border 込み)、padding-inline 4px、radius `--radius-inner` **6px**、地 `--color-neutral` `#0000000F` / `#FFFFFF1A`、**border-bottom 2px solid `--color-border-emphasized` `#d4d4d4` / `#525252`**(他の 3 辺は無し)、文字 `--color-text-primary` `#000000` / `#ffffff`、font `--font-family-body`(Figtree)12px / 20px / 500、user-select none。
- 表示文字: `mod` → Apple は `⌘`、他は「Ctrl」(`soft.kbd.key.control_short`)/ `ctrl` → `⌃`(全プラットフォーム)/ `alt` → `⌥` / `shift` → `⇧` / `enter` → `↵` / `backspace` → `⌫` / `tab` → `⇥` / `up` `↑` / `down` `↓` / `left` `←` / `right` `→` / `plus` → `+` / `escape` → 「Esc」(`soft.kbd.key.escape_short`)/ その他は大文字化(`k` → `K`)。

### 18.5 Motion

なし。

### 18.6 Behaviour & a11y

- root role img、名前 = 各キーの読み上げ語を「{shortcut} + {key}」で連結(例「Command + Shift + P」、非 Apple「Control + Shift + P」)。語: Alt / Backspace / Command / Control / Down arrow / Enter / Escape / Left arrow / Plus / Right arrow / Shift / Tab / Up arrow、それ以外は大文字。個々の kbd は aria-hidden。
- Apple 判定はクライアントで(SSR では非 Apple)。kivact では `kivact::platform` の OS 判定(macOS / iOS で ⌘)。

### 18.7 Kivact 実装方針

NEW kit component。研究対象 `crates/kivact-ui-basic/src/primitives.rs`(`kbd` / `kbd_group`)。下辺だけの 2px border は style の辺別 border(無ければ paint で下辺に 2px の帯を描く)。

### 18.8 受け入れ確認

- [ ] `soft::kbd("mod+k")` が macOS で「⌘」「K」、Windows で「Ctrl」「K」。名前「Command + K」/「Control + K」。
- [ ] 20px 高、下辺 2px 線、角丸 6、12px medium。light/dark。

---

## 19. Token

出典: `Token/Token.tsx`(578 行)、`Token/TokenLink.tsx`(145 行)、`Token.doc.mjs`、`Token.test.tsx`。

### 19.1 概要

エンティティを行内に示すチップ/タグ(フィルタ値、複数選択値、ユーザ、ラベル)。色でカテゴリを区別(緑 Active、赤 Blocked、青 In Review)。取り消し可能な選択には `onRemove`。種類を早く識別できるなら先頭アイコン。1〜3 語(溢れたら ellipsis)。主要アクションやナビゲーションには使わない(Button / Link)。label を隠すのはアイコンが自明な時だけ。1 グループで色を使いすぎない(2〜3 色)。

### 19.2 Anatomy

```
static:      <span> {icon} <span label/> {endContent} {removeButton} </span>
href:        <a> 同上 </a>                       (onRemove ありなら <span container> <a invisible>…</a> <button remove/> </span>)
onClick:     <span container focus-within-ring onClick>
                {icon} <button invisible>{label}</button> {endContent} {removeButton}
             </span>
removeButton: <button aria-label="Remove {label}"> <Icon close 12px/> </button>   (16×16, 当たり判定 24×24)
```

```
md: ╭────────────────────╮ 24px
    │ ◉ Design   ✕       │  radius 6, padding 0 8, gap 4, 12px medium
    ╰────────────────────╯
```

### 19.3 API

| prop | 型 / 既定 | Kivact |
|---|---|---|
| `label` | string(必須) | `soft::token(label)` |
| `size` | sm/md/lg / md | `.size(ControlSize::Small \| Medium \| Large)` |
| `color` | `default red orange yellow green teal cyan blue purple pink gray` / default | `.color(soft::TokenColor::Default \| Red \| ..)` |
| `icon` | ReactNode | `.icon(view)` |
| `isDisabled` | / false | `.disabled(..)` |
| `onRemove` | (e) => void | `.on_remove(f)`(remove ボタンを出す) |
| `onClick` | (e) => void | `.on_press(f)`(token 全体がボタン) |
| `href` | string | `.on_press(f)` + Link role にする `.as_link(true)`(native に URL 無し) |
| `description` | string | `.description(text)`(a11y description) |
| `endContent` | ReactNode | `.end(view)` |
| `isLabelHidden` | / false | `.label_hidden(true)`(名前は label のまま) |

### 19.4 Visual spec(Token.tsx:123-266)

- inline-flex、align center、gap 4px、padding-block 0、border なし、radius `--radius-inner` **6px**、font inherit family / 12px / 20px / 500、nowrap、max-width 100%、overflow hidden、label ellipsis。
- size: sm **20px**(28−8)/ md **24px**(32−8)/ lg **28px**(36−8)、padding-inline 8px(全 size)、文字 12px(全 size)。
- 色: default = 地 `--color-neutral` `#0000000F` / `#FFFFFF1A`、文字 `--color-text-primary` `#000000` / `#ffffff`。他は §1.1 装飾色表(地 = background-X、文字 = text-X、gray 含む)。
- interactive(onClick / href): hover で overlay-hover、押下で overlay-pressed を地の上に重ねる(backgroundImage 型, §1.2)。cursor pointer。
- focus: href(remove なし)は token 自体に focus-visible リング。onClick / href+remove は内側のボタン/リンクにフォーカスし、**token 全体の外形に** 2px accent リング(focus-within, offset 3px)。
- disabled: opacity 0.5、pointer 無効。
- remove button(Token.tsx:182-202): 16×16、radius full、padding 0、margin-inline-end **−4px**(右余白を 8→4px に詰める)、icon `x` **12px**、色 inherit、当たり判定を各辺 4px 拡張(24×24)。自身の focus-visible リング(2px accent、円形)。hover 時の独自の塗りは**無い**(Astryx 通り)。
- isLabelHidden: label は視覚的に非表示(アイコンのみの token)。

### 19.5 Motion

| trigger | property | duration | easing | 備考 |
|---|---|---|---|---|
| hover / pressed(interactive) | background-image overlay | 125ms 指定だが補間されず即時 | ease-standard | Token.tsx:147-155 |
| 押下解放(タッチ) | pressed overlay | 200ms | `cubic-bezier(0,0,0.2,1)` | §1.2 |

### 19.6 Behaviour & a11y

- static: role なし(文字として読まれる)。href: role link。onClick: 内側 role button(名前 = label、labelledby/describedby は内側へ)。
- remove: role button、名前「Remove {label}」、クリックは伝播停止(token の onClick を起こさない)。disabled 時は aria-disabled で Enter/Space 抑止。
- container の onClick は内側の button / a 以外をクリックしたときに発火(テキスト部分でも起動)。
- disabled + href(remove なし): `<a role=link aria-disabled tabIndex=0>`(Tab stop 残し、起動しない)。

### 19.7 Kivact 実装方針

- NEW kit component。onClick 型: root = `Role::Group`(または generic)の row、label 部を `use_button()` の part を持つノード(名前 = label)、remove は別の `kivact::ui::button` 相当の小ノード。focus-within リングは「内側ボタンの focus_visible 状態」を root の paint が読む(PartState を共有 Value にして root style に反映)— kivact に focus-within 相当の state style があればそれを使う(UNRESOLVED)。
- `kivact::ui::tag_input` の tag recipe(「One tag: its text and its remove button」)にも同じ見た目を適用する(Tokenizer 担当と共有)。
- 研究対象: `crates/kivact-ui-basic/src/badge.rs`、`crates/kivact-ui-basic/src/recipes_controls.rs`(tag_input の tag / remove)。

### 19.8 受け入れ確認

- [ ] md 24px、radius 6、padding 8、12px medium、11 色(light/dark)。
- [ ] on_remove: 「Remove {label}」ボタン、押しても on_press は呼ばれない、当たり判定 24px。
- [ ] on_press: Tab で入ると token 全体にリング、Enter/Space で起動、hover 5% / 押下 10%。
- [ ] disabled: 50%、起動しない。
- [ ] 長い label が ellipsis。

---

## 20. 共有 hook の実装メモ

### 20.1 usePressFeedback(`hooks/usePressFeedback.ts`, `utils/pressFeedback.ts`, `utils/pressGesture.ts`, `utils/interactionOverlay.stylex.ts`)

- 押下を描く全要素に付ける印(`data-astryx-pressable`)と、文書レベルの押下コントローラの導入(参照カウント)。描画は `interactionOverlayStyles` のいずれか。
- 動作: マウス/デスクトップは CSS `:active` のまま。coarse pointer(スマホ/タブレット)では `:active` を捨て、コントローラが `data-astryx-press="on" | "fading"` を書く。強度 `--astryx-press-alpha`(0..1)で `color-mix(overlay-pressed × α)` を塗る。
- 定数: onset **150ms**、slop **10px**、速いタップの flash **100ms**、release **200ms** `cubic-bezier(0, 0, 0.2, 1)`(テーマ tempo で ×min(tempo,1.5)、reduced motion では 200ms 以下)、scroll brake **150ms**。メニュー内(`data-astryx-menu-press`)は useMenuPress が担当するので対象外。
- Kivact: engine の host pressed 状態 + `style().pressed(..)` に任せ、解放時のフェード(200ms)を Paint transition で付ける。onset 遅延・スクロール取り消しは host の press 判定が持つかを foundation/motion.md 側で確認(§21)。

### 20.2 useInteractiveRole / InteractiveRoleContext(`hooks/useInteractiveRole.ts`, `InteractiveRoleContext/InteractiveRoleContext.ts`)

- 返り値 `'link' | 'button' | 'inert'`: href あり & 有効 → link、onClick あり → button、親の context(Popover / DropdownMenu の trigger 文脈)が役割を与えていればそれ、どれも無ければ inert。disabled は href を無視(disabled link 回避)。
- 使用: Link、Token(他に Thumbnail, Item, ClickableCard)。
- Kivact: kit 内部関数 `fn interactive_role(has_link: bool, has_press: bool, disabled: bool) -> soft::InteractiveRole` と、trigger 側が `kivact::view::provide(soft::InteractiveRole::Button)` で子に役割を伝える context(`try_context::<soft::InteractiveRole>()`)。

### 20.3 useKeyboardHint(`hooks/useKeyboardHint.tsx`)

- roving composite(SegmentedControl, Toolbar, TabList)に「矢印で移動できる」ことを初回だけ教える一時表示。
- 表示条件: container への最初の `:focus-visible` 入場(外から)。isEnabled=false(SegmentedControl は disabled 時)なら出さない。一度消えたらそのインスタンスでは再表示しない。
- 消える条件: 矢印キー(←→↑↓)の最初の押下 / **3000ms** 経過 / container 外へのフォーカス移動。container 内でフォーカスが移ったら anchor を新しい要素へ付け替える。
- 位置: フォーカス要素の **下**、inline-start 揃え、間隔 8px(`--spacing-2`)。top layer(クリップされない)。
- 見た目(useKeyboardHint.tsx:112-155): 地 `--color-background-popover` `#ffffff` / `#1b1b1b`、radius `--radius-element` 10px、影 `--shadow-low`、padding 4px / 8px、文字 12px / 20px、`--color-text-secondary` `#474747` / `#9e9e9e`、nowrap、pointer-events none。中身 = Kbd(§18)で矢印キー(horizontal: `←` `→`、vertical: `↑` `↓`、both: 4 つ)を gap 4px で並べ、その後に margin-inline-start 4px で「to navigate」(`soft.keyboard_hint.to_navigate`)。
- motion: opacity 0 ↔ 1、**150ms**、CSS 既定 `ease`(`cubic-bezier(0.25, 0.1, 0.25, 1)`)、出現・消失とも。
- a11y: aria-hidden(読み上げない)。
- Kivact: kit 内部 `soft::keyboard_hint(orientation)` を overlay(anchor = 現在フォーカスの NodeRef、placement below/start, offset 8)で実装。表示は `Style::enter(style().opacity(0))` / `exit(..)` + 150ms Paint transition。overlay の anchor 付け替えができなければ「最初にフォーカスした要素に固定」で可。研究対象 `crates/kivact-ui-basic/src/hover_card.rs`(trigger に付く overlay)。

### 20.4 useIndicatorFocusRing(`hooks/useIndicatorFocusRing.tsx`)

- 本物の input が視覚的に隠れているコントロール(CheckboxInput, RadioListItem)で、input が `:focus-visible` の間だけ indicator 要素(container の最初の子)の inline style に標準リング(2px solid accent、offset 3px)を書き、blur で消す。disabled なら描かない。
- Kivact: checkbox / radio recipe の indicator 部が root の `PartState.focus_visible` を読んでリングを描く(角丸 = 箱 6px + offset、円は円)。hook 相当の公開 API は不要。

### 20.5 useClickableContainer(`hooks/useClickableContainer.ts`)

- カード等の「面全体クリック」を、内部のボタン/リンクのクリックと衝突させずに実現。クリック対象から container までの間に interactive 要素(`button, a, input, select, textarea, [role=button|link|checkbox|radio|switch|tab|menuitem|option|combobox|listbox|slider|spinbutton]`, pressable container)があれば無視。テキスト選択中(選択範囲あり)なら無視。onClick が無ければ interactiveRef(主要リンク/ボタン)へ click を代理。href: 通常クリックで遷移、Ctrl/Meta・target=_blank は新規タブ、中クリック(onMouseUp で button===1)も新規タブ。disabled なら何もしない。
- 本グループでは直接使わない(ClickableCard / SelectableCard 用)が、Token の container click(内側の button / a 以外)も同じ考え方。
- Kivact: kivact の hit testing では内側の press ハンドラが優先される前提で、container の `on_press` は内側ボタンで発火しないことを確認するだけでよい。テキスト選択判定は選択可能テキストを使う場合のみ。

### 20.6 useMenuPress(`hooks/useMenuPress.ts`, `hooks/menuPressGesture.ts`)

- macOS/iOS のメニュー押下モデル: **離した位置の行が実行される**。押している間ハイライトがポインタに追従。マウスは trigger の押下で即座に開き、そのままドラッグして行で離せる。指は trigger に **500ms** 静止で開く(移動許容 **13px**)。
- settle 規則: 開いた直後のリリースは、ポインタがメニューに入ったか押下が **300ms** 続いた場合のみ実行(開いた瞬間にポインタ下の行を誤実行しない)。
- マウスをメニュー外で離す → 閉じる。指をメニュー外で離す → 開いたまま。
- 端の自動スクロール: メニュー上下端 **24px** 以内で保持すると 16ms ごとに 6px スクロール。
- タッチ後にブラウザが送る遅延 click(**400ms** 以内)は握りつぶす。disabled 行・divider・見出しの上ではハイライトを消す。
- 使用: DropdownMenu(MoreMenu の trigger)、ContextMenu、Selector。
- Kivact: `kivact::ui::menu` / `use_menu` の挙動。press-drag-release を持たない場合は menu 担当の指示書で扱う(§21)。

### 20.7 useLongPress(`hooks/useLongPress.ts`, `motion/uikit/longPress.ts`)

- 単一指の長押しを検出し開始点 `{x, y}` を返す(iOS Safari は長押しで contextmenu を出さないため、ContextMenu へのタッチ経路)。既定 **500ms**、いずれかの軸で **10px** を超える移動で取消、指を離す・2 本指・アンマウントでも取消。touch のみ(マウスは contextmenu を併用)。
- best practice: 開く面は報告点の下に出す。主要操作の唯一の手段にしない。
- Kivact: `on_pointer` + タイマ(`kivact-task`)で kit 内部 helper `soft::long_press(f)` を作るか、`Menu::attach_context` がタッチ長押しを扱うならそれに任せる(§21)。

---

## 21. UNRESOLVED 一覧

1. **Button busy の回転周期**: `kivact::ui::button` の busy part は 0.7s/周の固定回転。Astryx Neutral は 525ms/周・弧 135°・時計回り。`button_busy` recipe の paint で角度を自前の時計から描けるか未確認。不可なら 0.7s を許容。
2. **Button の Delayed reveal(225ms 遅延 + 125ms フェード)**: recipe の transition に delay を書けるか未確認。不可なら kit で loading を遅延させた派生 Value を作る。
3. **isInterruptible / ToggleButton の pressedChangeAction 中 spinner**: kivact の `.loading` は押下を無視し、toggle には loading が無い。「見た目だけ busy」の kit Variant 案(`soft::BusyLook`)で可か要判断。
4. **disabled + tooltip のフォーカス保持**: kivact の `.disabled(true)` がフォーカス不可にする場合、Astryx の「aria-disabled でフォーカス可能・tooltip 読める」挙動を再現する手段。
5. **サイズ継承(SizeContext)**: ButtonGroup / Toolbar / ToggleButtonGroup の `size` を子 standard button に伝える手段が kivact に無い。kit が子 builder に `.size()` を適用する方式(GroupMember trait)で行くか、doc で「子にも付ける」とするか。
6. **Toolbar の roving**: 任意 focusable 子孫の列挙 API が無い前提で `soft::toolbar_item` 登録方式を提案。text input の caret guard(キャレット位置取得)可否。
7. **Toolbar の padding**: Neutral の section 上書きで全辺 12px になると判断したが(theme layer が StyleX より上位)、docsite 実測で 8px(block)/12px(inline)の可能性もある。要実測。edge compensation 量(−8px)も同様。
8. **SegmentedControlItem の角丸 8px**: Neutral が track padding のみ 4px に上書きし private var(2px)は据え置きのため 10−2=8px。幾何的同心(6px)ではない。docsite で確認推奨。
9. **SegmentedControl の上下矢印**: kivact `ListPolicy::radio()` は 4 方向キーで移動。Astryx は水平のみ。軽微として許容するか。
10. **ProgressBar（C-41）**: 標準部品は初回 0→値、400ms の EaseOut、不確定表示は 1.3s 周期。Astryx の初回即値、300ms の standard 曲線、1.5s 周期、reduced motion 時の 2s 脈動に合わせるには、標準部品でモーション設定を公開する必要がある。
11. **タッチ押下モデル**(onset 150ms / slop 10px / flash 100ms / release 200ms / scroll brake 150ms): engine 側の host press 判定で表現できるか。foundation/motion.md と整合を。
12. **Link の行内(段落中)配置**: kivact の text に押下可能な inline span を埋め込めるか。
13. **Token の focus-within リング**: 内側ボタンのフォーカスで外側 token にリングを描く state style の有無。
14. **Banner dismiss 後のフォーカス復帰先**: banner に入る直前のフォーカス要素を記録する公開手段。
15. **MoreMenu**: `presentation`(bottom-sheet / adaptive)と `isMenuOpen` / `onOpenChange`(open binding)の kivact::ui::menu 対応。useMenuPress の press-drag-release を kivact menu が持つか。
16. **useLongPress**: kit で実装するか `attach_context` 任せか。
17. **ControlSize::ExtraSmall / ButtonVariant::Outline の値**: Astryx に無いため §2.4.6 の提案値(Astryx 由来ではない)。
18. **ToggleButton doc と実装の不一致**: doc は icon-only で自動 tooltip と書くがコードは付与しない。本書はコードに従った。
19. **ToggleButton の文字幅（C-40）**: 選択時に semibold になる文字の幅を未選択時から確保するには、標準部品がラベルの測定方法を公開する必要がある。

