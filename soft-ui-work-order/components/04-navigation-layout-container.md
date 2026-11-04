# 04. Navigation, Layout & Containers — kivact-ui-soft 作業指示書

Astryx(neutral テーマ)の Navigation / Layout / Container 系コンポーネントを `crates/kivact-ui-soft` に再実装するための仕様。値は neutral の light / dark を解決済みで併記する。kit の import は `use kivact_ui_soft as soft;` を前提に API を `soft::xxx` で書く。トークン名は Astryx 名(packages/core/src/theme/tokens.stylex.ts)、ファイル参照は `D:\w\meta\astryx` からの相対パス。

## 目次

0. [グループ共通の前提](#0-グループ共通の前提全セクションが参照する)(spacing 表、container padding 伝播、breakpoint、状態表現、アイコン、Layer 出現、i18n、Popover 面、hover/press overlay、Button 要点、settle spring)
1. [AppShell (+ useAppShellMobile)](#1-appshell--useappshellmobile)
2. [Layout (+ LayoutHeader, LayoutContent, LayoutFooter, LayoutPanel)](#2-layout--layoutheader-layoutcontent-layoutfooter-layoutpanel)
3. [Stack / HStack / VStack / StackItem](#3-stack--hstack--vstack--stackitem)
4. [Grid / GridSpan](#4-grid--gridspan)
5. [Center](#5-center)
6. [Section](#6-section)
7. [FormLayout](#7-formlayout)
8. [AspectRatio](#8-aspectratio)
9. [Divider](#9-divider)
10. [Resizable / ResizeHandle / useResizable](#10-resizable--resizehandle--useresizable)
11. [ScrollableArea (+ useScrollableArea, useScrollOverflow, useOverflow)](#11-scrollablearea--usescrollablearea-usescrolloverflow-useoverflow)
12. [Breadcrumbs / BreadcrumbItem](#12-breadcrumbs--breadcrumbitem)
13. [Outline](#13-outline)
14. [Pagination](#14-pagination)
15. [SideNav (+ SideNavHeading, SideNavItem, SideNavSection, SideNavCollapseButton)](#15-sidenav--sidenavheading-sidenavitem-sidenavsection-sidenavcollapsebutton)
16. [TopNav (+ TopNavHeading, TopNavItem, TopNavMenu, TopNavMegaMenu, TopNavMegaMenuItem, TopNavMegaMenuFeaturedCard)](#16-topnav--topnavheading-topnavitem-topnavmenu-topnavmegamenu-topnavmegamenuitem-topnavmegamenufeaturedcard)
17. [MobileNav (+ MobileNavToggle)](#17-mobilenav--mobilenavtoggle)
18. [NavIcon](#18-navicon)
19. [NavMenu (NavHeadingMenu, NavHeadingMenuItem)](#19-navmenu-navheadingmenu-navheadingmenuitem)
20. [Stepper / Step](#20-stepper--step)
21. [TabList / Tab / TabMenu](#21-tablist--tab--tabmenu)
22. [Card](#22-card)
23. [ClickableCard](#23-clickablecard)
24. [SelectableCard](#24-selectablecard)
25. [Carousel](#25-carousel)
26. [Collapsible / CollapsibleGroup / useCollapsible](#26-collapsible--collapsiblegroup--usecollapsible)
27. [useContainerReveal](#27-usecontainerreveal)
28. [useMediaQuery(レスポンシブ・ブレークポイント)](#28-usemediaqueryレスポンシブブレークポイント)

## グループ概要

| Astryx component (and parts) | docs URL | source dir | kivact strategy | closest kivact-ui-basic analog |
|---|---|---|---|---|
| AppShell (+ useAppShellMobile) | https://astryx.atmeta.com/components/AppShell | packages/core/src/AppShell | NEW kit component(内部で Layout / MobileNav) | crates/kivact-ui-basic/src/sidebar.rs(`sidebar_provider`) |
| Layout, LayoutHeader, LayoutContent, LayoutFooter, LayoutPanel | https://astryx.atmeta.com/components/Layout | packages/core/src/Layout | NEW kit component(region は `column`/`row`/`scroll` + style、padding 伝播 context) | crates/kivact-ui-basic/src/card.rs, sidebar.rs |
| Stack, HStack, VStack, StackItem | https://astryx.atmeta.com/components/Stack | packages/core/src/Stack, HStack, VStack | kivact `row`/`column` + style ヘルパ(薄い builder) | なし(basic は素の `column(..).style(style().gap(..))`) |
| Grid, GridSpan | https://astryx.atmeta.com/components/Grid | packages/core/src/Grid | kivact `grid` + `Track` ヘルパ(上限付き列は rect 読み) | なし |
| Center | https://astryx.atmeta.com/components/Center | packages/core/src/Center | kivact `row` + style ヘルパ | なし |
| Section | https://astryx.atmeta.com/components/Section | packages/core/src/Section | NEW kit component(小、bleed + 告知) | crates/kivact-ui-basic/src/card.rs |
| FormLayout | https://astryx.atmeta.com/components/FormLayout | packages/core/src/FormLayout | NEW kit component(layout + Field 用 context) | crates/kivact-ui-basic/src/field.rs(`field_group`/`field_set`) |
| AspectRatio | https://astryx.atmeta.com/components/AspectRatio | packages/core/src/AspectRatio | 薄い kit 部品(`Style::aspect_ratio`) | crates/kivact-ui-basic/src/primitives.rs(`aspect_ratio`) |
| Divider | https://astryx.atmeta.com/components/Divider | packages/core/src/Divider | NEW kit component(小、separator + label + bleed) | crates/kivact-ui-basic/src/primitives.rs(`separator`) |
| Resizable / ResizeHandle / useResizable | https://astryx.atmeta.com/components/Resizable | packages/core/src/Resizable | RESTYLE `kivact::ui::split`/`splitter` + kit wrapper(`use_splitter` + collapse/snap/px) | crates/kivact-ui-basic/src/recipes_controls.rs(`splitter`/`split` recipe) |
| ScrollableArea (+ useScrollableArea, useScrollOverflow, useOverflow) | https://astryx.atmeta.com/components/ScrollableArea | packages/core/src/ScrollableArea, packages/core/src/hooks | RESTYLE `kivact::control::scroll` + kit wrapper(名前付き viewport、overflow hooks) | crates/kivact-ui-basic/src/scroll_area.rs |
| Breadcrumbs, BreadcrumbItem | https://astryx.atmeta.com/components/Breadcrumbs | packages/core/src/Breadcrumbs | NEW kit component(項目 menu は RESTYLE `kivact::ui::menu`) | crates/kivact-ui-basic/src/breadcrumb.rs |
| Outline | https://astryx.atmeta.com/components/Outline | packages/core/src/Outline | NEW kit component(`use_list` + scroll-spy) | なし(行は sidebar.rs が近い) |
| Pagination | https://astryx.atmeta.com/components/Pagination | packages/core/src/Pagination | NEW kit component(RESTYLE button / number_input / select を内部使用) | crates/kivact-ui-basic/src/pagination.rs |
| SideNav, SideNavHeading, SideNavItem, SideNavSection, SideNavCollapseButton | https://astryx.atmeta.com/components/SideNav | packages/core/src/SideNav (+ NavItem) | NEW kit component(大) | crates/kivact-ui-basic/src/sidebar.rs + sidebar/ |
| TopNav, TopNavHeading, TopNavItem, TopNavMenu, TopNavMegaMenu, TopNavMegaMenuItem, TopNavMegaMenuFeaturedCard | https://astryx.atmeta.com/components/TopNav | packages/core/src/TopNav | NEW kit component(大、hover popup は `use_popup` + `use_delayed`) | crates/kivact-ui-basic/src/navigation_menu.rs, menubar.rs |
| MobileNav, MobileNavToggle | https://astryx.atmeta.com/components/MobileNav | packages/core/src/MobileNav | RESTYLE `kivact::ui::dialog(DialogKind::Drawer(side))` + kit wrapper | crates/kivact-ui-basic/src/sidebar.rs(mobile sheet), recipes_dialog.rs |
| NavIcon | https://astryx.atmeta.com/components/NavIcon | packages/core/src/NavIcon | NEW kit component(表示のみ) | crates/kivact-ui-basic/src/avatar.rs(円の見た目) |
| NavMenu (NavHeadingMenu, NavHeadingMenuItem) | https://astryx.atmeta.com/components/NavMenu (UNRESOLVED) | packages/core/src/NavMenu | NEW kit component(`use_list` menu policy) | crates/kivact-ui-basic/src/list_popup.rs, recipes_menu.rs |
| Stepper, Step | https://astryx.atmeta.com/components/Stepper | packages/core/src/Stepper | NEW kit component | crates/kivact-ui-basic/src/questionnaire.rs |
| TabList, Tab, TabMenu | https://astryx.atmeta.com/components/TabList | packages/core/src/TabList | RESTYLE `kivact::ui::tabs` + kit wrapper(nav pattern、scroll strip、TabMenu) | crates/kivact-ui-basic/src/line_tabs.rs |
| Card | https://astryx.atmeta.com/components/Card | packages/core/src/Card | NEW kit component(容器 + padding 告知) | crates/kivact-ui-basic/src/card.rs |
| ClickableCard | https://astryx.atmeta.com/components/ClickableCard | packages/core/src/ClickableCard | NEW kit component(Card + `use_button`) | crates/kivact-ui-basic/src/item.rs |
| SelectableCard | https://astryx.atmeta.com/components/SelectableCard | packages/core/src/SelectableCard | RESTYLE checkbox semantics(`use_checkbox`)+ kit wrapper(Card) | crates/kivact-ui-basic/src/choice.rs, card.rs |
| Carousel | https://astryx.atmeta.com/components/Carousel | packages/core/src/Carousel | NEW kit component(native 横スクロール strip) | crates/kivact-ui-basic/src/carousel.rs(構造は流用しない) |
| Collapsible, CollapsibleGroup, useCollapsible | https://astryx.atmeta.com/components/Collapsible | packages/core/src/Collapsible | RESTYLE `kivact::ui::disclosure` + kit wrapper(group) | crates/kivact-ui-basic/src/disclosure.rs |
| useContainerReveal | https://astryx.atmeta.com/components/useContainerReveal (UNRESOLVED) | packages/core/src/hooks | kit style ヘルパ(`group` / `group_hover` / `group_focus`) | なし |
| useMediaQuery | https://astryx.atmeta.com/components/useMediaQuery (UNRESOLVED) | packages/core/src/hooks, packages/core/src/theme/themeAdaptations.ts | kit env ヘルパ(`window_size()` memo) | なし |

### レイアウト primitive の判定(要求: 「kivact の row/column/style ヘルパか、本物のコンポーネントか」)

| Astryx | 判定 | 中身 | prop → spacing の要点 |
|---|---|---|---|
| Stack / HStack / VStack | **ヘルパ**(薄い builder `soft::hstack`/`vstack`、semantics なし) | `row` / `column` + `style()` | gap n → row-gap = column-gap = `--spacing-n`(0.1)。padding*: 辺 > 軸 > 全体 |
| StackItem | **ヘルパ** | `node()` + `grow`/`shrink`/`min_*(0)`/`align_self` | spacing なし。fill = grow 1、static = grow 0 shrink 0 |
| Grid / GridSpan | **ヘルパ**(上限付き列のみ rect を読む reactive style) | `grid` + `Track` / `GridPlacement` | gap / rowGap / columnGap n → `--spacing-n`。rowHeight px → grid-auto-rows |
| Center | **ヘルパ** | `row` + align/justify center | padding* は Stack と同じ |
| AspectRatio | **ヘルパ寄りの小部品**(basic にも同名) | `aspect_ratio` + clip + 絶対配置の子 | spacing なし |
| Divider | **本物**(小) | separator semantics + label + bleed | label の左右 `--spacing-3` 12px |
| Section | **本物**(小) | 背景 + bleed + container padding 告知 | 未指定 = テーマ(neutral 12px)、n → `--spacing-n` |
| FormLayout | **本物**(小) | flex/grid + Field 用 context | vertical gap 16px、horizontal-labels gap 12px / 16px |
| Layout (+ 4 region) | **本物** | `column`/`row`/`scroll` + padding 解決 context | 2.4 の padding 解決表 |
| Card | **本物** | 容器 + padding 告知 | 未指定 = テーマ(neutral 12px)、border 1px 分を差し引き |
| ScrollableArea | **本物**(小、RESTYLE scroll) | `scroll` + 名前 + 溢れ時のみ tab stop | padding n → content box の `--spacing-n`(既定 0) |


## 0. グループ共通の前提(全セクションが参照する)

### 0.1 SpacingStep → px(全レイアウト primitive 共通)

Astryx のレイアウト系 props(`gap` / `padding*` / `rowGap` / `columnGap`)はすべて `SpacingStep = 0 | 0.5 | 1 | 1.5 | 2 | 3 | 4 | 5 | 6 | 8 | 10`(packages/core/src/utils/types.ts:45)を取り、step `n` は常にトークン `--spacing-n`(小数点は `-`: `0.5` → `--spacing-0-5`)に 1:1 で写る(packages/core/src/Stack/stack.stylex.ts:93-137, packages/core/src/Layout/padding.stylex.ts:17-29)。トークンは rem だが root 16px で 4px×step。neutral テーマは spacing を上書きしないので light/dark 共通で次の値になる。

| step | token | px |
|---|---|---|
| 0 | `--spacing-0` | 0 |
| 0.5 | `--spacing-0-5` | 2 |
| 1 | `--spacing-1` | 4 |
| 1.5 | `--spacing-1-5` | 6 |
| 2 | `--spacing-2` | 8 |
| 3 | `--spacing-3` | 12 |
| 4 | `--spacing-4` | 16 |
| 5 | `--spacing-5` | 20 |
| 6 | `--spacing-6` | 24 |
| 8 | `--spacing-8` | 32 |
| 10 | `--spacing-10` | 40 |

(`--spacing-7` 28px / `-9` 36px / `-11` 44px / `-12` 48px はトークンとして存在するが、公開 props からは選べない。container 内部の `SpacingToken` だけが持つ: packages/core/src/Layout/container.stylex.ts:49-64。)

Kivact 側の提案: `pub enum Space { S0, S0_5, S1, S1_5, S2, S3, S4, S5, S6, S8, S10 }` と `impl Space { pub fn px(self) -> f32 }` を `soft::Space` として公開し、kit の全 builder の `.gap(..)` / `.padding(..)` はこの enum を取る。値は `soft::Tokens`(kit の `ThemeExtension`)の `spacing` 表から読む(テーマ差し替えで変わる余地を残す)。任意 px が欲しい呼び出し側は素の `style().gap(n)` を使えばよい。

`SizeValue = number | string`(utils/types.ts:13)は「number = px、string = CSS 値そのまま」。Kivact では `impl Into<Length>`(px 数値 / `pct(..)` / calc)に置き換える。`'100%'` → `pct(100)`。`'auto'` → `auto()`。`'50vh'` 等の viewport 単位は reactive style で `kivact::platform::window_size().height * 0.5` と計算する(window_size は tracked: crates/kivact-core/src/platform/env.rs:100-121)。

論理方向: Astryx は `paddingInlineStart/End`, `marginInlineStart` 等の論理プロパティで RTL を自動反転する。Kivact の `Style` は物理(`padding_left` / `padding_right`)なので、kit の全コンポーネントは `kivact::view::current_direction()` を読み、RTL で left/right を入れ替える(水平 stack は `Direction::RowReverse`)。kivact-ui-basic の `basic::direction` と同じ扱い(crates/kivact-ui-basic/README.md「`basic::direction` provides a reading direction」)。

### 0.2 Container padding 伝播(Card / Section / Dialog / Drawer / BottomSheet / Layout / Divider / Table が共有)

Astryx は「padding を持つ容器が自分の padding を継承 CSS 変数で子孫に告知し、子孫がそれを読んで足す/打ち消す」仕組みを持つ(packages/core/src/Layout/container.stylex.ts:66-626, packages/core/src/Layout/padding.stylex.ts:455-535)。

- `--container-padding-{inline-start,inline-end,block-start,block-end}`: 容器の実 padding。子孫は **引いて** bleed する(Section の外枠 `margin-inline: calc(-1 * var(--container-padding-inline-*))`、Divider `isFullBleed`、Layout 外枠)。
- `--layout-padding-{outer,inner}-{x,y}` / `--layout-padding-own-outer-{x,y}`: Layout の各 region が **足す** padding。容器(Card/Section/Dialog)が outer/inner に自分の padding を入れるので、Card 内の LayoutHeader は Card の padding で inset される。
- テーマ既定の padding: Card は `var(--astryx-card-padding, --spacing-4)`、Section は `var(--_section-padding-propagated, var(--astryx-section-padding, --spacing-4))`、Dialog / Drawer / BottomSheet も同形(container.stylex.ts:104-162)。**neutral テーマは card と section の base padding を `--spacing-3` = 12px に上書きする**(packages/themes/neutral/src/neutralTheme.ts の `components.card.base.padding` / `components.section.base.padding`)。Dialog/Drawer/BottomSheet は 16px のまま。
- オーバーレイ根(Dialog, BottomSheet, Drawer, MobileNav, Lightbox, layer surface)は `overlayPaddingReset`(padding.stylex.ts:519-535)で `--container-padding-*` を 0、`--layout-padding-*` と `--_section-padding-propagated` を `initial`(= 各 reader の既定値に落ちる)にリセットする。

Kivact 実装: CSS 変数の継承は無いので、**context で同じ情報を流す**。

```rust
#[derive(Clone, Copy, Default)]
pub struct ContainerInset {           // --container-padding-* (bleed で引く量)
    pub start: f32, pub end: f32, pub top: f32, pub bottom: f32,
}
#[derive(Clone, Copy, Default)]
pub struct LayoutInset {              // --layout-padding-* (Layout region が足す量)。None = 既定 16px
    pub outer_x: Option<f32>, pub outer_y: Option<f32>,
    pub inner_x: Option<f32>, pub inner_y: Option<f32>,
    pub own_outer_x: Option<f32>, pub own_outer_y: Option<f32>,
}
```

容器は `kivact::view::provide(ContainerInset{..}, || child)` で子を包み、bleed する子は `kivact::view::try_context::<ContainerInset>()` を読み負の margin を付ける。オーバーレイ(kit の drawer / mobile nav、および `kivact::ui::dialog` の content)は `ContainerInset::default()` と `LayoutInset::default()` を provide し直す。first-child / last-child 条件(Section の上下 bleed)は context では分からないので、親容器が子に「先頭/末尾」フラグを渡すか、bleed したい子(Section, Divider, Table)を明示 API(`.bleed(Edges)`)にする(後述の各セクション参照)。

### 0.3 ブレークポイント(詳細は 28. useMediaQuery)

`DEFAULT_WIDTH_BREAKPOINTS = { sm: 640, md: 768, lg: 1024, xl: 1280, '2xl': 1536 }`(packages/core/src/theme/themeAdaptations.ts:50-57)。AppShell の mobile 判定既定は `md`(768px 未満が mobile)。FormLayout `horizontal-labels` は `max-width: 480px` で縦積みに崩れる。

### 0.4 共通の状態表現

- フォーカスリング: `--focus-outline-width` 2px / `--focus-outline-style` solid / `--focus-outline-color` = `--color-accent`(light `#1b1b1b` / dark `#f1f1f1`)/ `--focus-outline-offset` 3px。`:focus-visible` のみ(packages/core/src/utils/focusOutline.stylex.ts)。Kivact の `Style::ring(width, color)`(crates/kivact-style/src/sheet.rs:745)は border box の外側に接して描かれ、offset 引数が無い。3px の隙間付きリングは kit 共通ヘルパ `soft::focus_ring(node_ref)` として実装する: `style().focus_visible(style().ring_none())` で既定リングを消し、base style の `.paint(..)`(state style 内の paint は無視される: sheet.rs:926-929)で `node_ref.focus_visible()` を読み、true のとき「border box を 3px 膨らませ、半径も +3px した角丸矩形を 2px で stroke」する(stroke 中心は外側 4px)。全コンポーネントがこのヘルパを使う。UNRESOLVED: `paint` がノード境界の外側に描けるか(クリップされるなら `draw_over` か親側で描く)。
- hover は Astryx では `@media (hover: hover)` で囲まれ、タッチ端末ではスティッキー hover が出ない。Kivact はポインタが mouse のときだけ hover state が立つ前提でよい(タッチの hover は出さない)。
- `--color-overlay-hover`(light `#0000000D` / dark `#FFFFFF0D`)と `--color-overlay-pressed`(light `#0000001A` / dark `#FFFFFF1A`)は「下地に重ねる半透明」。Kivact では背景色の上に重ねる 2 枚目の塗り(`Style::paint` か、絶対配置の overlay 子ノード)で描く。単色合成で近似する場合は下地 `#ffffff` 上で light hover = `#f2f2f2`、pressed = `#e6e6e6`、dark 下地 `#1b1b1b` 上で hover = `#272727`、pressed = `#323232`。
- 縮小モーション: Astryx は `@media (prefers-reduced-motion: reduce)` で duration を 0 にする。Kivact は `Transition` が既定で decorative なので reduced motion で自動的にスキップされる。「本質的」な動き(進捗など)だけ `.decorative(false)` にする。

### 0.5 アイコン

neutral テーマのアイコンは **Lucide**(packages/themes/neutral/src/icons.tsx)。semantic 名 → Lucide 名: `close`→`x`, `chevronDown`→`chevron-down`, `chevronLeft`→`chevron-left`, `chevronRight`→`chevron-right`, `chevronsLeft`→`chevrons-left`, `chevronsRight`→`chevrons-right`, `check`→`check`, `success`→`circle-check`, `error`→`circle-x`, `warning`→`triangle-alert`, `info`→`info`, `menu`→`menu`, `moreHorizontal`→`ellipsis`, `externalLink`→`external-link`, `search`→`search`。kivact-ui-basic の `basic::ICON_NAMES`(crates/kivact-ui-basic/src/icons.rs)と同じ Lucide 系なので `kivact::control::icon("chevron-down")` で描ける。ただし basic の集合に無い名前(少なくとも `menu`)があるので、soft テーマは自前の Lucide 集合(上記全部 + 各セクションで挙げる名前)を `Theme::with_icon` で入れる。Icon サイズ(packages/core/src/Icon/Icon.tsx:72-93, 196-199): `xsm` 12px, `sm` 16px, `md` 20px(省略時), `lg` 24px。色は既定 `inherit`、`secondary` = `--color-icon-secondary`(light `#6a6a6a` / dark `#9e9e9e`)。

### 0.6 Layer(popover / menu)の出現アニメーション(このグループの menu 系が共有)

packages/core/src/Layer/layerAnimations.stylex.ts:32-152。placement ごとの enter keyframes:
- below: `opacity 0, translateY(-8px) scale(0.95)` → `opacity 1, translateY(0) scale(1)`(`--spacing-2` = 8px)
- above: `translateY(+8px)`、end: `translateX(-8px)`(RTL `+8px`)、start: `translateX(+8px)`(RTL `-8px`)
- duration `--duration-fast-max` = **165ms**(neutral)、easing `--ease-standard` = `cubic-bezier(0.24, 1, 0.4, 1)`、fill backwards。
- reduced motion: アニメーション無し(coarse pointer + reduced では 165ms 以下の opacity フェードのみ)。
- **exit はアニメーション無し**(即時に消える)。

Kivact: `style().enter(style().opacity(0.0).translate_y(-8).scale_xy(0.95, 0.95)).transition(TransitionProp::Paint, Transition::tween(165, Easing::CubicBezier(0.24, 1.0, 0.4, 1.0)))`、`transform_origin` は popover の trigger 側の辺。標準 `kivact::ui::menu` / `dialog(Popover)` を restyle する場合はその recipe に同じ enter を入れる。

### 0.7 i18n

Astryx の英語既定文(packages/core/locales/en.json)を、このファイルでは `soft.<component>.<name>` キーとして提案する。kivact では `kivact::i18n::message("soft.x.y", "English")` / テンプレートは `message_with`(crates/kivact-ui-basic/README.md「Texts and their message keys」と同じ作法)。ICU の plural / number は kivact の `message_with` の能力に合わせて単純化してよい(UNRESOLVED: plural 対応の有無)。

### 0.8 Popover 既定面(menu / mega menu / heading menu が共有)

packages/core/src/Popover/usePopover.tsx:83-88: 背景 `--color-background-popover`(light `#ffffff` / dark `#1b1b1b`)、`border-radius: --radius-container` 12px、`box-shadow: --shadow-low`(light `0 2px 4px rgba(0,0,0,0.05), 0 4px 8px rgba(0,0,0,0.10)` / dark `0 2px 4px rgba(0,0,0,0.25), 0 4px 8px rgba(0,0,0,0.40), inset 0 0 0 1px rgba(255,255,255,0.08)`)、枠線なし。Kivact: `style().background(..).radius(12).shadow(Shadow::new(0,2,4,..)).shadow(Shadow::new(0,4,8,..))`、dark の inset は `Shadow::..inset()`(crates/kivact-style/src/effect.rs:63)。

### 0.9 Hover / press overlay(nav item・outline link・card 等が共有)

packages/core/src/utils/interactionOverlay.stylex.ts:94-336 の `interactionOverlayStyles`。fine pointer(マウス)では:
- hover(`@media (hover: hover)` かつ非 disabled): 背景 `--color-overlay-hover`(light `#0000000D` / dark `#FFFFFF0D`)。
- `:active`(押下中): `--color-overlay-pressed`(light `#0000001A` / dark `#FFFFFF1A`)。即時(押下に transition なし。コンポーネント側に `background-color` transition があればそれに従う)。
- variant `backgroundImage` は同じ色を `linear-gradient` の重ね画像として既存背景の上に乗せる(下地色を保ったまま暗くする)。`backgroundImageOnNeutral` は下地に `--color-neutral`(light `#0000000F` / dark `#FFFFFF1A`)を常に敷き、その上に hover / pressed を重ねる。
- coarse pointer(タッチ): `:active` を無効にし、press controller が `data-astryx-press="on"` を付けた間 pressed 色、離すと `"fading"` で `--astryx-press-alpha` を 1→0 に **200ms `cubic-bezier(0, 0, 0.2, 1)`** でフェード(reduced motion でも最大 200ms)。指の押下は遅延後に塗る(遅延値は pressGesture.ts。UNRESOLVED: 正確な ms)。

Kivact: `style().hover(style().background(soft::Color::OverlayHover)).pressed(style().background(soft::Color::OverlayPressed))`。下地色がある場合は「下地 + overlay を合成した色」を kit の Color 解決で返す(0.4 の合成値)か、overlay 用の子ノードを absolute で重ねる。タッチの 200ms フェードは `pressed` 解除時の paint transition `Transition::tween(200, Easing::CubicBezier(0.0, 0.0, 0.2, 1.0))` で近似。

### 0.10 Button(ghost / secondary)の要点(Pagination・nav toggle・collapse button・Carousel 矢印などが内部で使う)

Button 本体は別グループの担当(標準 `kivact::ui::button` を restyle)。このグループの部品が中で使う値だけ固定しておく(packages/core/src/Button/Button.tsx:95-260, 737-770):
- 高さ: sm `--size-element-sm` 28px / md 32px / lg 36px。padding 8px 12px(icon only は padding 0 で正方形: `aspect-ratio 1/1`)。gap 8px。radius `--radius-element` 10px。文字 `--text-label-*` 14px / 500 / 20px。
- ghost: 背景透明、文字 `--color-text-primary`(light `#000000` / dark `#ffffff`)。secondary: 背景 `--color-neutral`(light `#0000000F` / dark `#FFFFFF1A`)。primary: 背景 `--color-accent`(`#1b1b1b` / `#f1f1f1`)、文字 `--color-on-accent`(`#ffffff` / `#111111`)。
- hover / pressed: 0.9 の `backgroundImage` overlay(背景色の上に hover `#0000000D`/`#FFFFFF0D`、pressed `#0000001A`/`#FFFFFF1A`)。押下で `transform: scale(0.98)`(coarse では press controller 経由)。
- transition: `background-image, background-color, color, opacity, transform` 125ms `cubic-bezier(0.24, 1, 0.4, 1)`、reduced motion 0s。
- disabled: opacity 0.5、cursor default、overlay と scale 無し。
- Kivact: `button(label).variant(ButtonVariant::Ghost).size(ControlSize::Medium)`、icon only は `.icon(icon(..)).hide_label(true)`(kivact-ui-basic と同じ作法)。soft の button recipe(別グループ)が上記値を持つ前提で、このグループはそれを呼ぶだけにする。

### 0.11 settle spring(レイアウト移動の共通ばね)

Astryx の animated layout(AppShell の content glide、SideNav の rail 折畳み等)は motion role `springs.settle` = `SETTLE_SPRING = {duration: 0.5, bounce: 0}`(SwiftUI 形式、packages/core/src/motion/uikit/springs.ts:41)を使い、`settleTime(0.001)` までサンプリングする(packages/core/src/motion/animatedLayout.ts:138-170)。
- 質量 1 の物理値: ω = 2π / 0.5 = 12.566 rad/s、**stiffness = 157.91、damping = 25.13(臨界減衰)、mass = 1**。0.1% に収まるまで ≈ **735ms**(上記から計算)。
- Kivact: `Transition::spring(Spring { stiffness: 157.91, damping: 25.13, mass: 1.0 })`(crates/kivact-types/src/motion.rs:96-104)。
- reduced motion: fine pointer では **即時**、coarse pointer では opacity のみのクロスフェード `min(200ms, reducedMotionCrossFade)` を `curves.move` の bezier で(packages/core/src/motion/uikit/timing.ts:92-121)。kivact では decorative transition が自動で省略されるので即時に落ちる(coarse 用クロスフェードは省略可)。
- 「quick change」ばね `{duration 0.5, bounce 0.15}`(springs.ts:47): ω = 12.566、stiffness 157.91、damping = 4π(1-0.15)/0.5 = 21.36。


## 1. AppShell (+ useAppShellMobile)

docs: https://astryx.atmeta.com/components/AppShell 。source: packages/core/src/AppShell (AppShell.tsx, AppShellMobileContext.tsx, observeMainOverflow.ts, AppShell.doc.mjs, useAppShellMobile.doc.mjs, AppShellSafeArea.test.tsx, __tests__/AppShellMainKeyboard.a11y.browser.spec.ts ほか)。storybook: apps/storybook/stories/AppShell.stories.tsx。

**判定: 本物の kit コンポーネント(大)。** ページ全体の枠。内部は 2. Layout(header / start / content)を使い、ブレークポイント未満で 17. MobileNav(drawer)へ nav を移す。kivact-ui-basic で最も近いのは crates/kivact-ui-basic/src/sidebar.rs の `sidebar_provider`(inset と sidebar、mobile 時 sheet)。

### 1.1 概要
- アプリの各ページのルート枠。topNav / sideNav / banner / main content の slot を持ち、レスポンシブなモバイルナビと skip-to-content を自動で持つ。SideNav の折り畳みは SideNav 側の `collapsible`。
- `height="fill"`: ダッシュボード(内部スクロール)。`"auto"`: 内容と一緒にページが伸びる。
- `contentPadding`: フォーム/設定/本文は 4(16px)、表/ダッシュボード/地図は 0。
- 各 nav slot に `label` を付ける(TopNav と SideNav は別 navigation landmark)。ページ見出し(h1)は children の先頭に置く。
- AppShell を入れ子にしない、サブページの枠に使わない(→ Layout)、自前の skip link や `<main>` を足さない(AppShell.doc.mjs bestPractices)。

### 1.2 Anatomy
```
AppShell root (div.astryx-app-shell; flex column; 100dvh | min 100dvh; overflow clip)  ← AppShellMobileContext
├─ Skip link (a href="#astryx-app-shell-main")  "Skip to content"   ← 最初のフォーカス先。focus 時だけ見える
├─ Layout (height, padding 0)
│   ├─ header slot
│   │   ├─ header (div role=banner, .astryx-app-shell-header)   ← topNav か banner があるとき
│   │   │   └─ LayoutHeader(padding 0, divider = variant section && topNav)
│   │   │       ├─ banner (div, shrink 0)
│   │   │       └─ topNav(below breakpoint では TopNavRenderContext="mobile-bar")
│   │   └─ auto mobile top bar (div[role=banner if no header])   ← below bp && sideNav のみ && toggle 有効
│   │       └─ LayoutHeader → div role=navigation "Mobile navigation" (h 48px, px 8px)
│   │           ├─ sideNav(SideNavRenderContext="topbar": heading だけ描く)
│   │           └─ MobileNavToggle
│   ├─ start slot: LayoutPanel(padding 0, .astryx-app-shell-sidenav) ← above bp && sideNav
│   │   └─ sideNav(SideNavShellContext: rail 折畳み時のコンテンツ glide 用)
│   └─ content slot
│       └─ wrapper(display:contents | elevated: relative flex 1)
│           ├─ elevated backdrop(absolute; surface; 左上 radius 28px)  ← elevated && topNav && inline sideNav
│           └─ LayoutContent role=main id=astryx-app-shell-main (padding=contentPadding ?? 0)
│               └─ children
└─ Mobile nav(below bp): MobileNav drawer に sideNav/topNav を "drawer" 描画で入れる(React Activity で閉じても保持)
```

### 1.3 API

AppShell(AppShell.tsx:138-306):

| prop | type | default | 意味 | Kivact |
|---|---|---|---|---|
| children | ReactNode(必須) | — | main 内容 | `soft::app_shell(|| content)` |
| topNav | ReactNode | なし | 通常 TopNav | `.top_nav(|| soft::top_nav(..))` |
| sideNav | ReactNode | なし | 通常 SideNav | `.side_nav(|| soft::side_nav(..))` |
| banner | ReactNode | なし | topNav の上の告知 | `.banner(|| ..)` |
| contentPadding | SpacingStep | `0` | main の padding | `.content_padding(Space)` |
| height | `'fill' \| 'auto'` | `'fill'` | fill: 100dvh 固定・内部スクロール / auto: 最小 100dvh・ページスクロール・nav は sticky | `.height(ShellHeight::Fill \| Auto)` |
| variant | `'wash' \| 'surface' \| 'section' \| 'elevated'` | `'elevated'` | nav と content の背景の対比 | `.variant(ShellVariant::..)` |
| mobileNav | `false \| MobileNavConfig \| ReactNode` | config 既定 | false: 無効 / object: 自動 drawer の調整 / ReactNode: 完全自前 drawer | `.mobile_nav(MobileNavMode::Disabled \| Auto(MobileNavConfig) \| Custom(Box<dyn FnOnce()->View>))` |
| xstyle / className / style / ref / data-testid | — | — | root | 破棄 / NodeBuilder(`.test_id`) |

MobileNavConfig(AppShell.tsx:158-205):

| field | type | default | Kivact |
|---|---|---|---|
| hasToggle | boolean | true | `.has_toggle(bool)` — false で自動トグル(ハンバーガー)を出さない |
| isOpen | boolean | 非制御 | `.open(Signal<bool>)`(caller-owned。省略時は内部 signal) |
| onOpenChange | (bool) => void | なし | `.on_open_change(impl Fn(bool))` |
| content | ReactNode | なし | drawer の中身を差し替え(below bp のときだけ描画) |
| breakpoint | `'sm'\|'md'\|'lg'\|'xl'\|'2xl'\|'none'` | `'md'` | mobile 判定 `width < bp`(等しいときは広い側)。none = 常に desktop |
| defaultIsMobile | boolean | なし | SSR 初期値 → **不要**(破棄) |

useAppShellMobile()(AppShellMobileContext.tsx:18-62)の戻り: `isMobile`, `isMobileNavOpen`, `mobileNavId`, `toggleMobileNav()`, `openMobileNav()`, `closeMobileNav()`, `isMobileNavEnabled`, `hasAutoToggle`。provider 外では安全な既定値(すべて false / no-op、hasAutoToggle true)。Kivact: `soft::use_app_shell_mobile() -> AppShellMobile`(`try_context` で取得、無ければ既定値)。`mobileNavId` は kivact では drawer の `NodeRef`(`controls` 関係に使う)。

### 1.4 Visual spec

**root / variant 背景**(AppShell.tsx:312-336, 784-809)

| variant | root 背景 | nav 領域(header / sideNav) | content(main) 背景 | divider |
|---|---|---|---|---|
| wash | `--color-background-body`(light `#f1f1f1` / dark `#1b1b1b`) | body | body | なし |
| surface | `--color-background-surface`(light `#ffffff` / dark `#262626`) | surface | surface | なし |
| section | surface | (なし = root の surface) / auto のとき header は surface | なし(root) | header 下 1px と sideNav 内側 1px(`--color-border` light `#00000014` / dark `#FFFFFF1A`) |
| elevated(既定) | body | body | surface。**topNav と inline sideNav が両方あるとき** content は透明で、背後の backdrop(surface, 左上だけ `--radius-page` 28px)が「body の上に浮いた角丸面」を作る | なし |

- 高さ: fill → `height: 100dvh`、auto → `min-height: 100dvh`。`display:flex; flex-direction:column; position:relative; overflow:clip`。
- Layout は padding 0。header slot の LayoutHeader も padding 0、sideNav の LayoutPanel も padding 0(nav 自身が内側余白を持つ)、main の LayoutContent は `contentPadding ?? 0`。
- auto のとき: header は `position: sticky; top: 0; z-index: 1`、sideNav は `position: sticky; top: <header 実測高さ>; height: calc(100dvh - header 高さ); overflow: clip; flex column`(AppShell.tsx:469-527, 817-831)。header 高さは ResizeObserver で `--_app-shell-header-height` に書く。
- auto mobile top bar(below bp、topNav 無し、sideNav あり、toggle 有効): `display:flex; align-items:center; height: --spacing-12 = 48px; padding-inline: --spacing-2 = 8px`(AppShell.tsx:459-464)。中身は sideNav の heading(topbar 描画)+ MobileNavToggle。
- Safe area(AppShell.tsx:477-505): header は `padding-top: env(safe-area-inset-top)`、左右に `safe-area-inset-left/right`(RTL で入替え)。main は `padding-bottom: container-block-end + safe-area-inset-bottom`。header も top bar も無い shell は上 inset を持たない。Kivact: `kivact::platform::safe_area()`(crates/kivact-core/src/platform/env.rs:84)。

**skip link**(AppShell.tsx:337-391)
- 休止: 1×1px、`clip-path: inset(50%)`、margin −1px、absolute(視覚的に隠すが focus 可能)。
- `:focus`: `position: fixed; top: 8px; inset-inline-start: 8px; padding: 8px 16px; width/height auto; overflow visible; white-space normal`。背景 `--color-background-surface`(`#ffffff` / `#262626`)、文字 `--color-text-accent`(light `#1b1b1b` / dark `#f1f1f1`)、`font-weight: --font-weight-semibold` 600、`font-size: --text-body-size` 14px、下線なし、`z-index: 9999`、枠線 0、角丸なし。focus-visible で標準フォーカスリング(2px accent, offset 3px)。

**main のフォーカス表示**(AppShell.tsx:395-418, 1127-1226)
- main は fill かつ内容がはみ出しているとき(observeMainOverflow で監視)だけ `tabindex=0`(スクロール領域としてタブ停止)、それ以外は `tabindex=-1`(skip link の移動先)。
- Tab で到達したときだけリング: 2px `--color-accent`、`outline-offset: -2px`(内側に描く)。elevated の角丸面のときはリングの左上も 28px。ポインタやプログラム focus ではリングなし。

```
elevated, topNav + sideNav (desktop)
┌──────────────────────────────────────────────┐ body #f1f1f1
│ TopNav                                       │
├────────────┬─────────────────────────────────┤
│ SideNav    │╭────────────────────────────────┤ ← backdrop surface #ffffff, 左上 r=28
│ (body)     ││ main (contentPadding)          │
│            ││                                │
└────────────┴┴────────────────────────────────┘
```

**状態**: mobile(below bp)では sideNav panel を描かず、topNav は "mobile-bar" 描画(heading + endContent + toggle)、nav 本体は MobileNav drawer へ。

### 1.5 Motion
- **SideNav rail 折畳み時のコンテンツ glide**(AppShell.tsx:943-1080, SideNavShellContext): sideNav が rail ↔ 展開で幅を変えると、main 列を旧位置から新位置へ `travel = before.left - after.left`(RTL は right)だけずらして戻すアニメーション(`playLayoutSwap` / `rideWith`)。同時に旧内容のコピーを opacity `held → 0`、新内容の子を `entering → 1` でクロスフェード。content が左へ動く(rail が縮む)場合は、背景色の box-shadow `-travel*(1-p)px 0 0 0 <bg>` で露出部を覆う。動きの種類とばね定数は SideNav の railMotion が決める(15. SideNav の Motion を参照)。途中で逆転したときは進捗 p から held/entering を引き継ぐ(AppShell.tsx:995-1007)。
- Kivact: main 列に `translate_x` を `Transition::spring(..)` で 0 に戻す(paint transition で表現可能)。クロスフェードは旧内容の複製が要るので **省略可**(フォールバック: 列の translate のみ、内容は即時)。reduced motion では全て無し。
- **mobile nav の閉じ保持**: 閉じた後 `MAX_CLOSE_DELAY_MS(250ms) + 50ms = 300ms` drawer を mounted(Activity visible)のまま保ち、閉じアニメーションを最後まで描く(AppShell.tsx:643-663, MobileNav.tsx:286)。reduced motion では 0ms。Kivact では drawer 自体の `exit` style が同じ役目をする(17 参照)。
- skip link / 背景 / variant 切替に transition なし。

### 1.6 Behaviour & a11y
- landmark: header = `role=banner`、main = `role=main`(id `astryx-app-shell-main`)、auto top bar = `role=navigation` 名前 "Mobile navigation"、SideNav / TopNav は各自 navigation landmark(label 必須)。
- skip link: ページ最初の Tab 停止。Enter/クリックで main に `focus()`(AppShell.tsx:683-685)。
- main がスクロール領域のとき: Tab で main 自体に停止し、矢印 / PageUp/Down / Home/End でスクロール。ただし main の最初のフォーカス可能要素が「普通のリンクかボタン」なら Tab は main を飛ばしてそこへ入る(scrollKeyboardDelegation.ts: PASSIVE_ROLES 内の最初の sequential な要素)。
- mobile drawer はモーダル(focus trap、閉じたら toggle へフォーカス返却)。drawer の開閉は `useAppShellMobile` で子孫から制御可能。ルート変更後に `closeMobileNav()` を呼ぶのが推奨。
- 既定テキスト(i18n 提案キー):
  - `soft.app_shell.skip_to_content` = "Skip to content"(`@astryx.appShell.skipToContent`)
  - `soft.app_shell.mobile_navigation` = "Mobile navigation"(`@astryx.appShell.mobileNavigation`)

### 1.7 Kivact 実装方針
- `soft::app_shell(content) -> AppShell`(slot closure)。内部で `soft::layout()`(2)を使い、`provide(AppShellMobile{..}, ..)` で mobile 状態を流す。
- mobile 判定: `memo(move || window_size().width < bp_px)`(28)。`ShellBreakpoint::None` は常に false。
- 開閉 state: caller signal があればそれ、無ければ内部 `signal(false)`。`on_open_change` に変化を通知。
- drawer: 17. MobileNav(kit の drawer、内部は `kivact::interaction::use_dialog(DialogKind::Drawer(Side::Start))` 相当)。below bp でのみ mount。
- skip link: kivact はウィンドウアプリなので「Tab の最初に来る、focus 時だけ見えるボタン」として `button(message("soft.app_shell.skip_to_content", ..))` を `position(Absolute)` + `opacity(0)` → `focus_visible(style().opacity(1.0))`。押下で main の `NodeRef::focus()`。
- main のタブ停止: kivact の `scroll(..)` が自分でキーボードスクロールを持つかを確認し(UNRESOLVED: `Role::ScrollView` がフォーカス可能か)、overflow 時だけ focusable にする。overflow 判定は main の `rect()` と内容の `rect()` を比較。
- sticky(auto): kivact は `Position::Sticky` がある(crates/kivact-types/src/layout.rs:40-47)ので header / sideNav にそのまま使える。
- elevated の左上 28px 角丸: `radius_corners(..)`(sheet.rs:689)で top-left だけ 28(RTL では top-right)。
- rail glide: SideNav が幅を変える直前/直後に AppShell の `SideNavShell` context の `before_rail_change()` / `after_rail_change()` を呼び、main 列の `translate_x` を一瞬 `travel` にジャンプしてから spring で 0 へ(`Animated::jump_to` → target 0)。
- 最も近い basic: crates/kivact-ui-basic/src/sidebar.rs(`sidebar_provider` + mobile 時 sheet、`basic.sidebar.toggle`)。

### 1.8 受け入れ確認
- [ ] 起動直後の Tab で "Skip to content" が左上(8px, 8px)に現れ、Enter で main にフォーカスが移る(main はリングなし or Tab 到達時のみ内側リング)。
- [ ] role banner / main / navigation(TopNav, SideNav の label)が a11y tree に 1 つずつ。
- [ ] 既定 variant elevated で topNav + sideNav のとき、main 面が白(dark `#262626`)で左上 28px 角丸、周囲は `#f1f1f1`(dark `#1b1b1b`)。
- [ ] variant section で header 下と sideNav 右に 1px 線、wash で全面 body 色、surface で全面 surface 色。
- [ ] ウィンドウ幅 767px で sideNav が消え、top bar(高さ 48px)にハンバーガー。押すと drawer が開き、Esc で閉じてトグルへフォーカスが戻る。768px で desktop 表示に戻る。
- [ ] height auto で header が上端に sticky、sideNav が header の下で sticky。
- [ ] main が溢れていると Tab で main に止まり矢印でスクロール、溢れていなければ止まらない。
- [ ] rail 折畳みで main がスッと横移動して落ち着く(reduced motion では即時)。


## 2. Layout (+ LayoutHeader, LayoutContent, LayoutFooter, LayoutPanel)

docs: https://astryx.atmeta.com/components/Layout (LayoutHeader / LayoutContent / LayoutFooter / LayoutPanel は sub-component)。source: packages/core/src/Layout (Layout.tsx, LayoutHeader.tsx, LayoutContent.tsx, LayoutFooter.tsx, LayoutPanel.tsx, LayoutSlotsContext.ts, LayoutAreaContext.ts, LayoutDividerContext.ts, container.stylex.ts, padding.stylex.ts, edgeCompensation.stylex.ts, layoutEdges.stylex.ts, layoutReset.ts)。

**判定: 本物の kit コンポーネント。** 見た目はほぼ余白と 1px 境界線だけだが、5 つの slot の有無に応じて各 region の padding を決め、親容器(Card / Section / Dialog / Drawer)の padding を打ち消して自分の region で付け直す「container padding 伝播」(0.2)の中核。region 自体は kivact の `column` / `row` / `scroll` に style を当てたもの。

### 2.1 概要
- header / start / content / end / footer の名前付き 5 region を、ページ内または境界付きの容器(Card, Dialog, Drawer)の中に並べる汎用 primitive。
- 単純な縦横積みは HStack / VStack。ページシェルとアプリ全体のナビゲーションは AppShell(Layout.doc.mjs bestPractices)。
- Card / Dialog の「ヘッダ + スクロールする本文 + アクション付きフッタ」を作るときに使う(Card.doc.mjs bestPractices)。

### 2.2 Anatomy
```
Layout outer (div.astryx-layout)                    ← 親容器 padding を負 margin で打ち消す
└─ Layout inner (div, flex column)                   ← --container-padding-* を 0 に; own padding / contentWidth を告知
    ├─ [header slot]  LayoutHeader (div[data-divider]?) → inner div(padding)
    ├─ middle (div, flex row; flex:1; min-height:0)
    │   ├─ [start slot]  LayoutPanel (area=start)
    │   ├─ content wrapper (div, StackItem fill)
    │   │   └─ [content slot | children] LayoutContent
    │   └─ [end slot]    LayoutPanel (area=end)
    └─ [footer slot]  LayoutFooter (div[data-divider]?) → inner div(padding)
```
各 slot は任意の ReactNode を取る。LayoutHeader 等を使うのが通常。slot の有無は `LayoutSlotsContext{hasHeader, hasFooter, hasStart, hasEnd}`(Layout.tsx:370-381)で子 region に伝わり、panel は `LayoutAreaContext`('start' | 'end')で自分の側を知る(Layout.tsx:284-295)。

### 2.3 API

Layout(Layout.tsx:190-279):

| prop | type | default | 意味 | Kivact |
|---|---|---|---|---|
| header / footer / start / end | ReactNode | なし | 各 slot | `soft::layout().header(|| ..).footer(|| ..).start(|| ..).end(|| ..)`(slot closure) |
| content / children | ReactNode | なし | 中央 slot(children も content と同じ) | `.content(|| ..)` |
| height | `'fill' \| 'auto'` | `'fill'` | fill: 容器の高さを満たす / auto: 内容で伸びる(min-height 100%) | `.height(LayoutHeight::Fill \| Auto)` |
| padding | SpacingStep | なし | **この Layout の** region の外周 padding(入れ子 Layout には継承しない)。0 = 全 region full bleed | `.padding(Space)` |
| contentWidth | SizeValue | なし | 各 region の揃え幅の最大(中央寄せ) | `.content_width(impl Into<Length>)` |
| defaultHasDividers | boolean | なし | 子 LayoutHeader / LayoutFooter の hasDivider 既定 | `.default_dividers(bool)` |
| xstyle / className / style / ref | — | — | outer に付く | 破棄 / NodeBuilder |

LayoutHeader(LayoutHeader.tsx:82-130) / LayoutFooter(LayoutFooter.tsx:80-119):

| prop | type | default | Kivact |
|---|---|---|---|
| children | ReactNode | — | `soft::layout_header(children)` / `soft::layout_footer(children)` |
| hasDivider | boolean | `defaultHasDividers ?? false` | `.divider(bool)` |
| height | SizeValue | なし(内容) | `.height(..)` |
| padding | SpacingStep | なし(伝播値) | `.padding(Space)` |
| paddingBlockEnd(header のみ) | SpacingStep | なし | `.padding_bottom(Space)` |
| label | string | なし | `aria-label` → `.name(..)` |
| role | AriaRole | なし | `.role(Role::..)`(例: header に `Role::Banner` 相当は使わない。必要時のみ) |

LayoutContent(LayoutContent.tsx:110-143):

| prop | type | default | Kivact |
|---|---|---|---|
| children | ReactNode | — | `soft::layout_content(children)` |
| padding | SpacingStep | なし | `.padding(Space)` |
| isScrollable | boolean | **true** | `.scrollable(bool)`(true = 内部を `kivact::control::scroll`) |
| label / role | — | なし | `.name(..)` / `.role(..)` |

LayoutPanel(LayoutPanel.tsx:110-178):

| prop | type | default | Kivact |
|---|---|---|---|
| children | ReactNode | — | `soft::layout_panel(children)` |
| hasDivider | boolean | false | `.divider(bool)`(内側の辺に 1px) |
| padding | SpacingStep | なし | `.padding(Space)` |
| isScrollable | boolean | **true** | `.scrollable(bool)` |
| width | SizeValue | なし(内容) | `.width(..)` |
| resizable | ResizableProps(useResizable の戻り) | なし | `.resizable(ResizableState)`(10. Resizable を参照。指定時は width を無視し hook のサイズを使う) |
| label / role | — | なし | `.name(..)` / `.role(..)` |

### 2.4 Visual spec

描画は「1px の divider 線」だけ。背景・角丸・影なし(親容器のものが見える)。色: divider = `--color-border`(light `#00000014` / dark `#FFFFFF1A`)。

**(a) Layout 外枠(Layout.tsx:67-173)**
- outer: `margin: calc(-1 * --container-padding-{inline-start,inline-end,block-start,block-end})`。つまり Card(neutral padding 12px)の中に置くと Card の内側 12px を打ち消して Card の縁に密着する。
- inner: `--container-padding-*` を 0 にリセット(Layout 内の bleed 子はもう打ち消さない)、`--layout-padding-own-outer-{x,y}` を `padding` prop の値(未指定なら `initial`)に。`display:flex; flex-direction:column`。
- height fill: outer と inner とも `height: calc(100% + block-start + block-end)`(打ち消した分も含めて親を満たす)、`max-height: var(--container-max-height, none)`。auto: `min-height:100%`。
- middle: `display:flex; flex-direction:row; flex:1; min-height:0`。header 直後に divider の無い header がある場合 `--_layout-flush-block-start: 0px`、footer 直前に divider の無い footer がある場合 `--_layout-flush-block-end: 0px`(Layout.tsx:102-122。header の下 padding と content の上 padding が二重にならないようにする)。
- `padding={0}` は fullBleed: own-outer を 0px(全 region の外周 0)。

**(b) padding の解決規則**(各 region、明示 `padding` が無いとき。px は neutral の値)

定義: `OUTER_X = own-outer-x ?? outer-x ?? 16`、`OUTER_Y = own-outer-y ?? outer-y ?? 16`、`INNER_X = inner-x ?? 16`、`INNER_Y = inner-y ?? 16`。
- 素のページ(容器なし): すべて 16px。
- neutral Card / Section の中: 容器が outer/inner を自分の padding = **12px** にする(container.stylex.ts:164-216)。
- Dialog / Drawer / BottomSheet の中: 16px(neutral は上書きなし)。
- Layout の `padding={n}`: OUTER_X / OUTER_Y だけが `--spacing-n` になる(own-outer)。INNER は容器値のまま。

| region | 上 | 下 | 開始側(LTR 左) | 終了側 |
|---|---|---|---|---|
| LayoutHeader inner | OUTER_Y | INNER_Y | OUTER_X | OUTER_X |
| LayoutFooter inner | INNER_Y | OUTER_Y | OUTER_X | OUTER_X |
| LayoutContent | header 無し: OUTER_Y / header あり & divider 無し: **0** / header に divider あり: INNER_Y | footer 無し: OUTER_Y / footer あり & divider 無し: **0** / divider あり: INNER_Y | start panel 無し: OUTER_X / あり: INNER_X | end panel 無し: OUTER_X / あり: INNER_X |
| LayoutPanel (start) | header 無し: OUTER_Y / あり: INNER_Y | footer 無し: OUTER_Y / あり: INNER_Y | OUTER_X | INNER_X(divider 無しなら margin-inline-end: −INNER_X で詰める) |
| LayoutPanel (end) | 同上 | 同上 | INNER_X(divider 無しなら margin-inline-start: −INNER_X) | OUTER_X |

出典: LayoutHeader.tsx:37-67, LayoutFooter.tsx:35-65, LayoutContent.tsx:34-108, 180-241, LayoutPanel.tsx:36-101, 208-288。
- panel の「divider 無しで margin を詰める」(LayoutPanel.tsx:95-100, 236-251)により、panel と content の間隔は INNER_X 1 つ分(16px)になる。divider ありでは panel 側 INNER_X + 線 1px + content 側 INNER_X。
- 明示 `padding={n}` を region に付けると、その region は 4 辺 `--spacing-n` に固定され、上記の条件分岐(noStart 等)は適用されない。`padding={0}` は 0 + 子への告知も 0。
- 各 region は自分の padding を `--container-padding-*` として子孫に告知する(region 内の Divider full-bleed / Section / Table が region の縁まで伸びる)。

**(c) divider**
- LayoutHeader: `border-bottom: 1px solid --color-border`(LayoutHeader.tsx:68-72)。LayoutFooter: `border-top` 同色(LayoutFooter.tsx:66-70)。
- LayoutPanel start: `border-inline-end`、end: `border-inline-start`、1px `--color-border`(LayoutPanel.tsx:81-91)。
- header / footer は `flex-shrink:0`。panel は `flex-shrink:0; overflow:clip`(scrollable 既定で `overflow:auto`)。content は `flex:1; height:100%; min-height:0; overflow:clip`(scrollable 既定で `overflow:auto`)。

**(d) contentWidth(Layout.tsx:49-65, 132-188; LayoutContent.tsx:80-97)**
- header / footer inner: `max-width: contentWidth; margin-inline:auto`(中央寄せ)。
- panel なし: LayoutContent は全幅(スクロールバーは外縁)のまま、左右 padding を `max(P, (W - CW)/2 + P)` にして内容だけを CW 幅の枠に揃える(W = middle の幅、P = その辺の通常 padding)。
- panel 1 枚: middle を `max-width: CW` にし(margin 0)、start panel なら middle に `padding-inline-start: max(0, (100% - CW)/2)`。panel は CW 枠に揃い、content は反対側の開いた縁まで伸びる。
- panel 2 枚: middle 全体を `width:100%; max-width: CW; margin-inline:auto`。
- `%` を含む contentWidth(calc 内の % も)は内部揃えを諦め、middle に単純な `max-width + margin auto`(supportsInternalContentWidthAlignment)。

```
Layout(header+divider, start panel(divider), content, footer) in page (all 16px)
┌──────────────────────────────────────────────┐
│ 16  Header                                16 │ ↕16 / 16
├──────────────────────────────────────────────┤ 1px border
│16 Panel 16│16 Content (scroll)            16 │
│           │                                  │
│           │                                  │  content 下: footer 無 divider → 0
│ 16  Footer (上16 = INNER_Y)               16 │ ↕16 / 16
└──────────────────────────────────────────────┘
```

**(e) 状態**
なし(hover / focus / selected 等は持たない)。

### 2.5 Motion
なし(transition 指定なし)。panel 幅のアニメーションは SideNav 側(15)が持つ。

### 2.6 Behaviour & a11y
- role は付けない。region に `role` / `label` を渡すと landmark になる(例: panel に `role="complementary" label="Filters"`)。
- LayoutContent / LayoutPanel は既定でスクロール容器。Astryx は単に `overflow:auto`(tabindex は付けない)。キーボードでのスクロール担保が要る場合は 11. ScrollableArea を中に置く。
- 既定テキストなし。

### 2.7 Kivact 実装方針
- `soft::layout() -> Layout` builder(slot closure 方式、`impl IntoView`)。内部:
  - `provide(LayoutSlots{has_header, has_footer, has_start, has_end, header_divider, footer_divider}, ..)` で region に条件を渡す(divider の有無は flush 判定に必要なので、Layout が header/footer の builder 値を見て決める: header slot が `soft::layout_header` 以外の任意 View のときは divider 無しとみなす)。
  - `provide(LayoutInset{..}, ..)`(0.2)を読み・上書きして region の padding を上表の規則で数値に解決し、各 region は普通の `padding_*` を付ける。
  - outer の負 margin は `try_context::<ContainerInset>()`。fill 高さは `height(pct(100))` + 親の inset を足した calc、または「親容器が padding を持たない子枠を提供する」設計にしてもよい(推奨: soft の容器(card/dialog)は Layout を子に取ったとき自分の padding を 0 にし、Layout へ LayoutInset を渡す。見た目は同じでずっと単純)。
- region: header/footer = `column(..)`(shrink 0)、middle = `row((start?, content, end?))`、content / panel の scroll は `kivact::control::scroll(column(..)).axes(ScrollAxes::Vertical)`。
- divider は `border_bottom(1, soft::Color::Border)` 等。RTL では panel の start/end の線と padding を左右反転。
- contentWidth の内部揃えは `node_ref().rect()` で middle の幅 W を読み、上式で padding を計算する reactive style(`100cqi` の代替)。
- `resizable` は 10 の `soft::ResizableState`(caller-owned signal)を受け、panel の幅をその signal で決める。
- 最も近い basic: crates/kivact-ui-basic/src/sidebar.rs(`sidebar_provider` が inset と panel を並べる)と crates/kivact-ui-basic/src/card.rs(header / content / footer の padding)。

### 2.8 受け入れ確認
- [ ] ページ直下の Layout で header / content / footer の左右 padding が 16px、header 上 16px・下 16px、divider なし header の直後の content 上 padding 0。
- [ ] header に `divider(true)` で 1px 線(light `#00000014` / dark `#FFFFFF1A`)、content 上 padding 16px。
- [ ] neutral card の中の Layout で card の縁から 12px inset(16px ではない)、header divider が card の左右端まで届く。
- [ ] start panel(divider なし)と content の間隔が 16px(32px ではない)。divider ありで panel 16 + 1px + 16。
- [ ] content が既定でスクロールし、header / footer は固定。`height(Auto)` で全体が内容高さになる。
- [ ] `content_width(720)` で 1200px 幅の中に header / content の内容が 720px 枠で中央揃え、content のスクロールバーは右端。
- [ ] RTL で start panel が右側、divider がその左辺。


## 3. Stack / HStack / VStack / StackItem

docs: https://astryx.atmeta.com/components/Stack (HStack / VStack / StackItem は同ページの sub-component)。source: packages/core/src/Stack (Stack.tsx, StackItem.tsx, stack.stylex.ts, stackItem.stylex.ts), packages/core/src/HStack/HStack.tsx, packages/core/src/VStack/VStack.tsx。

**判定: kivact の `row` / `column` + style ヘルパ。** 状態も semantics も持たない flex 箱なので、kit は「Space enum を受ける薄い builder」(`soft::hstack` / `soft::vstack` / `soft::stack` / `soft::stack_item`)を出すだけで、描画は `kivact::control::row` / `column` に `style()` を重ねたもの。新しい interaction は不要。

### 3.1 概要
- 一軸の flex レイアウト。`gap` で子同士の間隔を作る(margin を手で付けない)。
- `StackItem size="fill"` で 1 つの子に残り幅/高さを取らせる。
- Stack の入れ子より先に `wrap="wrap"` を試す(Stack.doc.mjs bestPractices)。
- 2 次元配置は Grid、名前付き region は Layout、ページシェルは AppShell。

### 3.2 Anatomy
```
Stack (div, display:flex)            ← role なし。as で要素を変えられる
├─ StackItem (div, 任意)             ← flex-grow / align-self / min-size reset
│   └─ content
└─ content
```
HStack = `Stack direction="horizontal"`、VStack = `Stack direction="vertical"`(HStack.tsx:70-87, VStack.tsx:69-86)。

### 3.3 API

Stack(Stack.tsx:110-260、既定値は Stack.doc.mjs):

| prop | type | default | 意味 | Kivact |
|---|---|---|---|---|
| direction | `'horizontal' \| 'vertical'` | `'vertical'` | 主軸 | `soft::stack(Axis, children)`; `Axis::{Horizontal, Vertical}` |
| gap | SpacingStep | なし(0) | 子の間隔。row-gap と column-gap の両方に同値(stack.stylex.ts:93-137) | `.gap(Space)` → `style().gap(space.px())` |
| hAlign | start/center/end/between/around/evenly/stretch | なし | 水平方向の揃え。horizontal なら主軸、vertical なら交差軸 | `.h_align(StackAlign)` |
| vAlign | 同上 | なし | 垂直方向の揃え | `.v_align(StackAlign)` |
| justify | start/center/end/between/around/evenly | なし | 主軸 alias(hAlign/vAlign が勝つ) | `.justify(StackJustify)` |
| align | start/center/end/stretch | なし(CSS 既定 stretch) | 交差軸 alias | `.align(StackCross)` |
| wrap | nowrap/wrap/wrap-reverse | `'nowrap'` | flex-wrap | `.wrap(StackWrap)` |
| padding | SpacingStep | なし | 4 辺 | `.padding(Space)` |
| paddingInline / paddingBlock | SpacingStep | なし | 軸ごと(padding を上書き) | `.padding_inline(Space)` / `.padding_block(Space)` |
| paddingInlineStart/End, paddingBlockStart/End | SpacingStep | なし | 1 辺(優先度: 辺 > 軸 > 全体、Stack.tsx:207-213) | `.padding_inline_start(..)` ほか |
| isScrollable | boolean | false | `overflow: auto` | `.scrollable(true)` → 内部で `kivact::control::scroll(..)` に包む(axes は direction に合わせる) |
| width / height / maxWidth / minHeight | SizeValue | なし | インライン style(style prop より勝つ: Stack.tsx:240-254) | `.width(impl Into<Length>)` ほか、または `NodeBuilder::style` |
| as | ElementType | `'div'` | 要素型 | **不要**。必要なら `.role(Role::List)` 等を NodeBuilder で付ける |
| xstyle / className / style | — | — | 見た目の追加 | **破棄**。`NodeBuilder::style(..)` が後勝ちで同じ役 |
| ref | Ref | — | — | `NodeBuilder` の `node_ref` 系に任せる |

HStack: `hAlign: StackMainAlignment`(main)、`vAlign: StackCrossAlignment`(cross, doc 上の既定 `'stretch'` = CSS 既定)、`justify`→hAlign、`align`→vAlign(HStack.tsx:21-55)。VStack: `hAlign: cross`(既定 stretch)、`vAlign: main`(VStack.tsx:21-55)。

StackItem(StackItem.tsx:31-121, stackItem.stylex.ts):

| prop | type | default | 意味 | Kivact |
|---|---|---|---|---|
| size | `'static' \| 'fill'` | `'static'` | static = `flex-grow:0; flex-shrink:0`、fill = `flex-grow:1`(shrink は CSS 既定 1) | `soft::stack_item(child).size(ItemSize::Fill)` |
| crossAlignSelf | start/center/end/stretch | なし | align-self | `.cross_align_self(StackCross)` |
| isScrollable | boolean | false | overflow auto | `.scrollable(true)` |
| as / xstyle / className / style / ref | — | — | Stack と同じ扱い | 同上 |

常に `min-height: 0; min-width: 0` を付ける(stackItem.stylex.ts:21-26)。これにより `<StackItem size="fill" isScrollable>` がそれだけで完全なスクロール領域になる。

提案シグネチャ:
```rust
pub fn hstack(children: impl IntoView) -> Stack;      // Axis::Horizontal
pub fn vstack(children: impl IntoView) -> Stack;      // Axis::Vertical
pub fn stack(axis: Axis, children: impl IntoView) -> Stack;
impl Stack { gap, padding, padding_inline, padding_block, padding_inline_start, padding_inline_end,
             padding_block_start, padding_block_end, h_align, v_align, justify, align, wrap, scrollable }
impl NodeBuilder for Stack {}  impl IntoView for Stack {}
pub fn stack_item(child: impl IntoView) -> StackItem; // .size(ItemSize), .cross_align_self(StackCross), .scrollable(bool)
pub enum StackAlign { Start, Center, End, Between, Around, Evenly, Stretch }
pub enum StackWrap { NoWrap, Wrap, WrapReverse }
```

### 3.4 Visual spec(prop → style の厳密対応)

描画は一切しない(背景・枠・影・文字色なし)。すべてレイアウト値。

- base: `display:flex`(stack.stylex.ts:86-90)。direction horizontal → `flex-direction: row`、vertical → `column`(stack.stylex.ts:49-58)。Kivact: `row(..)` / `column(..)`。RTL の horizontal は `Direction::RowReverse`(CSS の row は RTL で右から並ぶため)。
- gap n → `column-gap` と `row-gap` を **同じ** `--spacing-n` に(0.1 の表)。例 `gap={2}` = 8px、`gap={4}` = 16px、`gap={10}` = 40px。Kivact `style().gap(px)` は両軸同値なので一致。
- 主軸揃え(justify-content): start → `flex-start`、center → `center`、end → `flex-end`、between → `space-between`、around → `space-around`、evenly → `space-evenly`(stack.stylex.ts:22-41)。Kivact `Justify::{Start, Center, End, SpaceBetween, SpaceAround, SpaceEvenly}`。
- 交差軸揃え(align-items): start → `flex-start`、center、end → `flex-end`、stretch(stack.stylex.ts:5-18)。未指定は CSS 既定 `stretch`。Kivact `Align::{Start, Center, End, Stretch}`(既定 Stretch)。
- hAlign / vAlign の解決(Stack.tsx:190-205): `resolvedH = hAlign ?? (horizontal ? justify : align)`、`resolvedV = vAlign ?? (horizontal ? align : justify)`。horizontal なら main = resolvedH / cross = resolvedV、vertical なら main = resolvedV / cross = resolvedH。cross 側に between/around/evenly が来た場合、StyleX の `alignItemsStyles[...]` に該当キーが無いので **何も付かない**(= stretch のまま)。cross 側に `stretch` 以外の main 専用値を渡した場合も同様に無視。main 側に `stretch` が来た場合も `justifyContentStyles` に無いので無視。Kivact も同じく無視する(debug で警告してよい)。
- wrap: nowrap / wrap / wrap-reverse(stack.stylex.ts:60-71)。Kivact `style().wrap(true)`。`wrap-reverse` は kivact に無い(`wrap: bool` のみ: crates/kivact-style/src/sheet.rs:385)→ UNRESOLVED。代替: 子の順序を反転し `wrap(true)` + `Direction::ColumnReverse` 相当は不可なので、`wrap-reverse` は「wrap と同じ」に落とし、debug 警告。
- padding: 各辺 `--spacing-n`(padding.stylex.ts:341-452 の inline-start/end, block-start/end 表)。優先順位は 1 辺指定 > 軸指定 > 全体。RTL では inline-start が右。
- isScrollable → `overflow:auto`。Kivact は `scroll(child).axes(..)`: horizontal stack なら `ScrollAxes::Horizontal`、vertical なら `Vertical`(CSS auto は両軸だが実用上は主軸; 両方必要なら `Both`)。
- width/height/maxWidth/minHeight: number は px、string はそのまま。インライン style として最後に載る(呼び出し側 style より勝つ)。
- StackItem: `min-width:0; min-height:0` 常時。static = `grow 0, shrink 0`、fill = `grow 1`。Kivact: `style().min_width(0).min_height(0).grow(0).shrink(0)` / `.grow(1)`。
- テーマ target: `astryx-stack`(visualProps direction, gap, wrap)、`astryx-stack-item`(size)(Stack.doc.mjs theming)。neutral は上書きなし。

```
hstack(gap=2, v_align=center)
┌────────────────────────────────────┐
│[A]←8px→[B]←8px→[C]                 │   main: start(既定), cross: center
└────────────────────────────────────┘
```

### 3.5 Motion
なし(transition 指定なし)。

### 3.6 Behaviour & a11y
- role なし、フォーカスしない、キー操作なし。`as="ul"` 等で意味を持たせる用途は Kivact では `.role(..)` で。
- `isScrollable` のとき、Kivact `scroll` は `Role::ScrollView` を持ちホイール/キーでスクロール(Astryx 側はただの `overflow:auto` で tabindex を付けない。キーボードでのスクロール可能領域の扱いは 11. ScrollableArea を使うべき)。
- 既定テキストなし。

### 3.7 Kivact 実装方針
- 新規 kit 部品だが **中身は `row` / `column` / `scroll` + `style()`**。crates/kivact-ui-basic には同等の部品は無い(basic は素の `column((..)).style(style().gap(12))` を推奨: crates/kivact-ui-basic/README.md 冒頭例)。soft では Space enum を強制するため薄い builder を置く。
- `Stack` は `Node` を内包し `NodeBuilder` を実装(呼び出し側の `.style(..)` が後勝ち)。base style は先頭レイヤに prepend する(`kivact::control::scroll` と同じ作法: crates/kivact-core/src/control/container.rs:115-120)。
- `wrap-reverse` のみ未対応(上記)。それ以外はすべて 1:1 で表現できる。
- 注意: kivact の `stack(children)`(crates/kivact-core/src/control/container.rs:177)は **重ね合わせ(z 方向)** で Astryx Stack とは別物。kit の関数名 `soft::stack` と衝突しないよう prelude を glob import しない運用、または kit 側を `soft::flex_stack` にする(推奨: `soft::hstack` / `soft::vstack` を主に使わせ、汎用版は `soft::stack_dir(axis, ..)` と命名)。

### 3.8 受け入れ確認
- [ ] `hstack` の gap S0..S10 で子の間隔が 0/2/4/6/8/12/16/20/24/32/40px。
- [ ] `vstack().h_align(Center)` で子が水平中央、`hstack().h_align(Between)` で両端揃え。
- [ ] cross 軸に `Between` を与えても stretch のまま(無視される)。
- [ ] `stack_item(..).size(Fill)` が残り幅を取り、`static` は縮まない(長い兄弟があっても幅維持)。
- [ ] `stack_item(..).size(Fill).scrollable(true)` が親の高さ内でスクロールする(min-size reset が効いている)。
- [ ] RTL で hstack の子が右から並び、`padding_inline_start` が右側に付く。
- [ ] 何も描かない(背景透明)— docsite の Stack playground(Card 3 枚、gap 2)と並びが一致(light/dark 差なし)。


## 4. Grid / GridSpan

docs: https://astryx.atmeta.com/components/Grid 。source: packages/core/src/Grid (Grid.tsx, GridSpan.tsx)。

**判定: kivact の `grid` + style ヘルパ。** 状態・semantics なし。ただし `columns={{minWidth, max}}` の上限付き列数は CSS の `min()/max()/calc()` を使うので、Kivact ではコンテナ幅を読む reactive style が要る(薄い builder `soft::grid` の中で実装)。

### 4.1 概要
- 2 次元の等幅トラック配置。スマホでも崩れてよいものは **常に** `columns={{minWidth: 240, max: 4}}` 形式(最大 4 列、各 240px 以上、狭いと 1 列)。
- `columns={4}`(数値)は絶対にリフローしないので、7 曜日のような固定行専用。
- `minWidth` 単独は狭い画面でオーバーフローするので `max` と必ず組む。`repeat:'fill'`(既定)は幅を揃え、`'fit'` は余白を埋めて伸ばす。
- HStack + wrap でグリッドを作らない(Grid.doc.mjs bestPractices)。

### 4.2 Anatomy
```
Grid (div, display:grid)
├─ item …                 ← 普通の子は 1 セル
└─ GridSpan (div, display:grid; grid-template-columns:minmax(0,1fr); height:100%; min-width:0)
     └─ content           ← columns / rows ぶん跨ぐ
```

### 4.3 API

Grid(Grid.tsx:51-149):

| prop | type | default | 意味 | Kivact |
|---|---|---|---|---|
| columns | `number \| {minWidth: number, max?: number, repeat?: 'fill'\|'fit'}` | なし(= 1 列 `1fr`) | 列定義 | `.columns(GridColumns)`; `GridColumns::Fixed(u16)` / `GridColumns::Responsive{ min_width: f32, max: Option<u16>, repeat: GridRepeat }` |
| gap | SpacingStep | なし | 両軸 gap | `.gap(Space)` |
| rowGap / columnGap | SpacingStep | なし | 軸別(gap を上書き) | `.row_gap(Space)` / `.column_gap(Space)` |
| rowHeight | number | なし | 暗黙行の高さ px(`grid-auto-rows`) | `.row_height(f32)` |
| align | start/center/end/stretch | なし(CSS `normal`) | align-items | `.align(Align)` |
| justify | start/center/end/stretch | なし(CSS `normal`) | justify-items | `.justify(Align)` |
| width / height / maxWidth / minHeight | SizeValue | なし | サイズ | `impl Into<Length>` |
| xstyle / className / style / ref | — | — | — | 破棄(NodeBuilder::style) |

GridSpan(GridSpan.tsx:24-44):

| prop | type | default | 意味 | Kivact |
|---|---|---|---|---|
| columns | `number \| 'full'` | なし | `span n` / `'full'` = `1 / -1` | `soft::grid_span(child).columns(Span::Cols(n))` / `Span::Full` |
| rows | number | なし | `grid-row: span n` | `.rows(u16)` |

### 4.4 Visual spec(prop → style)
- 描画なし(背景・枠なし)。
- `columns` の template(Grid.tsx:406-430):
  - 数値 N>0 → `repeat(N, 1fr)`。Kivact `Track::repeat(N, Track::fr(1))`。注意: CSS の `1fr` は `minmax(auto,1fr)` なので子の min-content で広がり得る(doc にも明記)。kivact の `Track::fr` も同じ意味(crates/kivact-style/src/grid.rs:73-80)なので一致。
  - object・max なし → `repeat(auto-fill|auto-fit, minmax(<minWidth>px, 1fr))`。Kivact `Track::repeat_auto_fill(Track::minmax(min_width, Track::fr(1)))`(fit は `repeat_auto_fit`)。
  - object・max あり(buildCappedTemplate, Grid.tsx:341-366) → `repeat(mode, minmax(min(100%, max(<minWidth>px, perColumn)), 1fr))`、`perColumn = calc((100% - (max-1) * gap) / max)`(gap は columnGap ?? gap、どちらも無ければ `calc(100% / max)`)。Kivact: グリッドの `NodeRef::rect()` 幅 W を読み、`track_min = min(W, max(min_width, (W - (max-1)*gap_px) / max))` を毎回計算して `Track::repeat_auto_fill(Track::minmax(track_min, Track::fr(1)))` を返す reactive style にする。これで「最大 max 列・各 min_width 以上・W < min_width なら 1 列で W 幅」が CSS と同じになる。
  - 未指定 / 0 → `1fr` 1 列。
- gap n → `gap: --spacing-n`(Grid.tsx:200-234)、rowGap → `row-gap`、columnGap → `column-gap`(後勝ち)。px は 0.1 の表。
- rowHeight → `grid-auto-rows: <n>px`(Grid.tsx:165-167)。Kivact: `style().grid_auto_rows(row_height)`(crates/kivact-style/src/sheet.rs:1259-1261)。
- align → `align-items: start|center|end|stretch`、justify → `justify-items`(Grid.tsx:170-198)。未指定は CSS `normal`(実質 stretch)。Kivact `align_items(Align::..)` / `justify_items(Align::..)`(crates/kivact-style/src/sheet.rs:1272-1275)。未指定は何も付けない。
- GridSpan: `min-width:0; display:grid; grid-template-columns:minmax(0,1fr); height:100%`(GridSpan.tsx:46-57)+ `grid-column: span n | 1 / -1`、`grid-row: span n`。Kivact `grid(child).style(style().min_width(0).grid_template_columns(Track::minmax(0, Track::fr(1))).height(pct(100)).grid_column(GridPlacement::span(n)))`、full は `grid_column((1, -1))`。
- テーマ target `astryx-grid`(align, columns, gap, justify)/ `astryx-grid-span`。neutral 上書きなし。

```
grid().columns(Responsive{min_width:240, max:Some(4)}).gap(S4)  幅 1100px
┌────────┬────────┬────────┬────────┐   perColumn=(1100-3*16)/4=263 → 4 列 263px
│   A    │   B    │   C    │   D    │
└────────┴────────┴────────┴────────┘
幅 400px → max(240, (400-48)/4=88)=240 → 1 列 (2*240+16>400)
```

### 4.5 Motion
なし。

### 4.6 Behaviour & a11y
role なし、フォーカスなし。並び順 = DOM 順 = 読み上げ順。既定テキストなし。

### 4.7 Kivact 実装方針
- `soft::grid(children) -> Grid`(NodeBuilder)、`soft::grid_span(child) -> GridSpan`。内部は `kivact::control::grid` と `kivact::style::{Track, GridPlacement}`(crates/kivact-core/src/control/container.rs:54, crates/kivact-style/src/grid.rs)。
- 上限付きレスポンシブ列だけ `node_ref()` の `rect()` を読む reactive style(crates/kivact-ui-basic/src/message_scroller.rs:62-68 の rect 読み取りパターン)。初回レイアウト前(rect 未計測)は `max` 列の均等割りではなく min_width ベースの `repeat_auto_fill(minmax(min_width, 1fr))` を仮に使う。
- kivact-ui-basic に同等部品なし。

### 4.8 受け入れ確認
- [ ] `Fixed(3)` は幅に関係なく常に 3 列。
- [ ] `Responsive{240, max 4}` で 1100px → 4 列、700px → 2 列、400px → 1 列(全幅)、200px → 1 列 200px(オーバーフローしない)。
- [ ] `repeat: Fit` で子が 2 個だけのとき 2 個が全幅を分け合う(Fill では空トラックが残る)。
- [ ] `grid_span().columns(Full)` が行全体、`rows(2)` + `row_height(100)` が 2 行分(100+gap+100)。
- [ ] docsite Grid ページの例と列数・gap が一致(light/dark 差なし)。


## 5. Center

docs: https://astryx.atmeta.com/components/Center 。source: packages/core/src/Center/Center.tsx。

**判定: kivact の `row` + style ヘルパ。** `display:flex`(row)に `align-items:center` / `justify-content:center` を付けるだけ。

### 5.1 概要
- 空状態、ローディング、ログインフォームなど「領域の真ん中に置く」用途。
- `axis="vertical"` は高さが無いと効かないので高さを与える。`isInline` はテキスト行中のアイコン/バッジ中央寄せ用。
- ページ全体の構造には使わない(Layout / AppShell)。横並びの中央寄せは `Stack hAlign="center"`(Center.doc.mjs bestPractices)。

### 5.2 Anatomy
```
Center (div, display:flex | inline-flex; flex-direction:row 既定)
└─ content(1 個以上)
```

### 5.3 API(Center.tsx:62-159)

| prop | type | default | Kivact |
|---|---|---|---|
| axis | `'both' \| 'horizontal' \| 'vertical'` | `'both'` | `soft::center(child).axis(CenterAxis::Both)` |
| width / height / maxWidth / minHeight | SizeValue | なし | `impl Into<Length>` |
| padding, paddingInline, paddingInlineStart/End, paddingBlock, paddingBlockStart/End | SpacingStep | なし | Stack と同じ(`.padding(Space)` ほか、優先度 辺 > 軸 > 全体: Center.tsx:198-203) |
| isInline | boolean | false | `.inline(true)`(下記) |
| children | ReactNode(必須) | — | `child: impl IntoView` |
| xstyle / className / style / ref | — | — | 破棄 / NodeBuilder |

### 5.4 Visual spec
- base `display:flex`(主軸 = row)、isInline なら `inline-flex`(Center.tsx:30-43)。
- axis both → `align-items:center` + `justify-content:center`、horizontal → `justify-content:center` のみ、vertical → `align-items:center` のみ(Center.tsx:209-210)。row 方向なので horizontal = 主軸、vertical = 交差軸。
- sizing は width/height/maxWidth/minHeight をそのまま(number は px)。padding は 0.1 の表。
- 描画なし。テーマ target `astryx-center`(axis)。
- Kivact: `row(child).style(style().align_items(Align::Center).justify_content(Justify::Center))`。`inline-flex` は kivact に概念が無い → `isInline` は「幅を内容に合わせる」(`grow(0)`、親の stretch を受けない `align_self(Align::Start)`)で近似。テキストの行内に置く用途はテキスト run 内に箱を置けないので UNRESOLVED(呼び出し側で `row((text, icon))` にする)。

### 5.5 Motion
なし。

### 5.6 Behaviour & a11y
レイアウト専用。role・名前を付けない(内容側が持つ)。

### 5.7 Kivact 実装方針
`soft::center(child) -> Center`(NodeBuilder)。中身は `kivact::control::row`。basic に同等なし。

### 5.8 受け入れ確認
- [ ] `center(spinner).style(style().height(200))` で spinner が上下左右中央。
- [ ] `axis(Horizontal)` で上寄せ・左右中央、`axis(Vertical)` で左寄せ・上下中央。
- [ ] padding S4 で 16px 内側に余白。


## 6. Section

docs: https://astryx.atmeta.com/components/Section 。source: packages/core/src/Section/Section.tsx。

**判定: 本物の kit コンポーネント(小)。** 背景を塗り、container padding を子孫に告知し(0.2)、親容器の padding を打ち消して端まで bleed する。

### 6.1 概要
- ページの region(設定グループ、フォームの節、サイドバー領域)を作る正しい方法。Card と違い「独立して並べ替え/削除できる 1 個のもの」ではなくページの一部。
- まず既定 variant(surface)。muted は特定 region に注意を引くときだけ。同じ背景の Section が並ぶときは dividers で区切る。見出し + Stack と組む(Section.doc.mjs bestPractices)。

### 6.2 Anatomy
```
Section outer (div)  ← 親容器の --container-padding-* を負 margin で打ち消す(bleed)。サイズ props はここ
└─ Section inner (div, class astryx-section) ← 背景・padding・divider 枠。子へ padding を告知
    └─ children
```
`<section>` 要素ではなく div。role も label も付けない(Section.tsx:283-361)。

### 6.3 API(Section.tsx:128-227)

| prop | type | default | 意味 | Kivact |
|---|---|---|---|---|
| variant | `'section' \| 'transparent' \| 'muted'` | `'section'` | 背景 | `soft::section(children).variant(SectionVariant::Surface \| Transparent \| Muted)` |
| padding | SpacingStep | テーマ既定(neutral 12px; テーマ無しで 16px) | 内側 4 辺 | `.padding(Space)`; 未指定 = テーマ値 |
| paddingInline / paddingBlock | SpacingStep | なし | 軸だけ上書き(他軸はテーマ値/padding を保持) | `.padding_inline(Space)` / `.padding_block(Space)` |
| paddingInlineStart/End, paddingBlockStart/End | SpacingStep | なし | 1 辺 | 同名 snake_case |
| dividers | `('top'\|'bottom'\|'start'\|'end')[]` | なし | 1px 枠線の辺 | `.dividers(Edges)`(bitflags: `Edges::TOP \| Edges::BOTTOM \| START \| END`) |
| width / height / maxWidth / minHeight | SizeValue | なし | outer のサイズ | `impl Into<Length>` |
| children | ReactNode | — | 内容 | `impl IntoView` |
| xstyle / className / style / ref | — | — | outer に付く | 破棄 / NodeBuilder(outer) |

### 6.4 Visual spec
- 背景(Section.tsx:49-59): `section` → `--color-background-surface`(light `#ffffff` / dark `#262626`)、`transparent` → 透明、`muted` → `--color-background-muted`(light `#f1f1f1` / dark `#1b1b1b`)。
- 角丸なし、影なし、既定の枠なし。
- padding: 未指定時は `container({useThemeDefault:'section'})`(Section.tsx:269-310)→ `var(--_section-padding-propagated, var(--astryx-section-padding, 16px))`。neutral は `components.section.base.padding = var(--spacing-3)` なので **12px**(light/dark 共通)。祖先 Section が明示 padding を持つとその値が伝播して既定より勝つ(`sectionPaddingPropagationStyles`, padding.stylex.ts:464-479)。明示 `padding={n}` は 4 辺 `--spacing-n`。軸/辺指定はそれに上書き。
- dividers(Section.tsx:90-111): 各辺 `1px solid --color-border`(light `#00000014` / dark `#FFFFFF1A`)。start/end は論理(RTL で反転)。
- bleed(Section.tsx:62-87): outer は `margin-inline: calc(-1 * --container-padding-inline-*)`、**最初の子のときだけ** `margin-top: -block-start`、**最後の子のときだけ** `margin-bottom: -block-end`。つまり Card(padding 12px)の中の Section は左右 12px はみ出して Card の縁まで塗り、先頭なら上端、末尾なら下端まで届く。inner は `--container-padding-*` を 0 にリセット後、自分の padding を告知、`height:100%`。
- テーマ target `astryx-section`(variant)、`theming.container: true`。

```
Card (padding 12)                         Card
┌──────────────────────┐                  ┌──────────────────────┐
│ ┌──────────────────┐ │   bleed 後 →     │██ Section (surface) ██│ ← 上・左右の 12px を打ち消し
│ │ Section          │ │                  │██  内容 (12px inset) ██│
│ └──────────────────┘ │                  │──────────────────────│ dividers bottom 1px
└──────────────────────┘                  └──────────────────────┘
```

### 6.5 Motion
なし。

### 6.6 Behaviour & a11y
- レイアウト専用。landmark にしない(必要なら呼び出し側が `.role(Role::Region).name(..)`)。
- 既定テキストなし。

### 6.7 Kivact 実装方針
- `soft::section(children) -> Section`(NodeBuilder は outer)。
- bleed: `try_context::<ContainerInset>()`(0.2)を読み、outer に `margin_left(-start)` `margin_right(-end)`(RTL 反転)。上下 bleed は first/last-child 判定が要るため、**容器側(soft::card / layout content / dialog content)が子を並べるときに first/last を知らせる** のは重いので、Section に `.bleed_top(bool)` / `.bleed_bottom(bool)` を持たせ、既定は「親が ContainerInset を provide していて、かつ自分が先頭/末尾」なら true にしたいが判定不能 → 既定 false、kit 内部の card/dialog が自分の first/last 子 Section を検出できない限り **UNRESOLVED**。実用案: `soft::card(..)` 等の容器が子 `View` を列で受けるとき、先頭/末尾の子に `SectionEdge` context を provide する(`for_index` で index 0 と last を判定)。
- 子への告知: inner で `provide(ContainerInset{ start: pad_l, end: pad_r, top: pad_t, bottom: pad_b }, ..)` と `provide(SectionPadding(p), ..)`(明示 padding のときのみ; 伝播用)。
- テーマ既定 padding は `soft::Tokens.section_padding`(neutral 12.0)。
- kivact-ui-basic に対応なし(basic の card.rs が最も近い容器実装: crates/kivact-ui-basic/src/card.rs)。

### 6.8 受け入れ確認
- [ ] 既定 Section は背景 light `#ffffff` / dark `#262626`、padding 12px、角丸 0。
- [ ] `variant(Muted)` で light `#f1f1f1` / dark `#1b1b1b`。
- [ ] `dividers(TOP|BOTTOM)` で上下に 1px の `#00000014` / `#FFFFFF1A`。
- [ ] padding 12px の card 内で Section が左右の縁まで塗る(card 内容の左端より 12px 外へ)。
- [ ] 入れ子 Section で外側 `padding(S6)` が内側既定 padding に伝播(内側も 24px)。
- [ ] docsite Section ページ(light/dark)と背景色・余白が一致。


## 7. FormLayout

docs: https://astryx.atmeta.com/components/FormLayout 。source: packages/core/src/FormLayout (FormLayout.tsx, FormLayoutContext.ts)。Field 側の連携: packages/core/src/Field/Field.tsx:240-330, packages/core/src/Field/FieldLabel.tsx:223。

**判定: 本物の kit コンポーネント(レイアウト + context)。** 見た目は flex/grid だけだが、`direction` と `defaultOptionality` を context で Field に渡し、Field の構造(ラベル列)を変える。

### 7.1 概要
- フォームの「どこに置くか」だけを担う(状態・送信は持たない。`<form>` で包む)。
- 縦積みが基本。対になる項目(姓 + 名、市 + 州 + ZIP)は vertical の中に horizontal を入れ子。設定画面のラベル横並びは `horizontal-labels`。
- 無関係な項目を横に並べない。`horizontal-labels` は CSS Grid なので最外側にだけ置き、他の FormLayout の中に入れない(FormLayout.doc.mjs bestPractices)。

### 7.2 Anatomy
```
FormLayout (div, class astryx-form-layout)         ← FormLayoutContext{direction, defaultOptionality}
├─ (任意) Heading
├─ Field …(Field は context を読む)
└─ (任意) footer(Submit / Cancel ボタン群)
```
horizontal-labels のとき Field は `display:contents` になり、ラベル wrapper が 1 列目、入力 + 説明 + status が 2 列目に入る(Field.tsx:291-330)。

### 7.3 API(FormLayout.tsx:72-114, FormLayoutContext.ts:27-50)

| prop | type | default | 意味 | Kivact |
|---|---|---|---|---|
| direction | `'vertical' \| 'horizontal' \| 'horizontal-labels'` | `'vertical'` | 配置 | `soft::form_layout(children).direction(FormDirection::Vertical \| Horizontal \| HorizontalLabels)` |
| defaultOptionality | `'optional' \| 'required'` | なし | フォームの既定。`optional` なら isRequired の項目だけ「必須」表示、`required` なら isOptional の項目だけ「任意」表示。`required` 下の無印項目は aria-required を出す(native required は付けない) | `.default_optionality(Optionality::Optional \| Required)` |
| children | ReactNode | — | — | `impl IntoView` |
| xstyle / className / style / ref | — | — | — | 破棄 / NodeBuilder |

### 7.4 Visual spec(FormLayout.tsx:37-66)
- vertical: `display:flex; flex-direction:column; gap: --spacing-4`(16px)。
- horizontal: 上に加え `display:grid; grid-auto-flow:column; grid-auto-columns:1fr`(子 1 個 = 1 等幅トラック、論理 inline 順)。gap は base の 16px(row/column とも)。
- horizontal-labels: `display:grid; grid-template-columns: auto 1fr; gap: 12px 16px`(row-gap `--spacing-3` 12px / column-gap `--spacing-4` 16px); `align-items:start`。`@media (max-width: 480px)` で `display:flex; flex-direction:column; gap:16px; align-items:stretch` に戻る。
- horizontal-labels の Field ラベル wrapper は `padding-top: calc(--border-width + --spacing-1)` = 1 + 4 = **5px**(入力の上枠 + 上 padding にラベル文字を揃える: Field.tsx:58-62)。
- 描画なし(背景・枠なし)。テーマ target `astryx-form-layout`(direction)。

```
horizontal-labels (幅 > 480px)
┌──────────────┬──────────────────────────────┐
│ Name   (5px↓)│ [ text input              ]  │  row-gap 12px
│ Email        │ [ text input              ]  │
│              │  supporting text / status    │
└──────────────┴──────────────────────────────┘
  auto 列       ↔16px↔  1fr 列
```

### 7.5 Motion
なし。

### 7.6 Behaviour & a11y
- role なし(`<form>` は呼び出し側)。
- `defaultOptionality` は FieldLabel の「Optional」「Required」表示と aria-required を制御(文言キーは Field 側の担当グループ。参考: `soft.field.optional` = "Optional"、`soft.field.required` = "Required" を提案)。

### 7.7 Kivact 実装方針
- `soft::form_layout(children) -> FormLayout`。`kivact::view::provide(FormLayoutCx{direction, default_optionality}, || ..)` で子を包み、kit の `soft::field`(standard `kivact::ui::field` を restyle + kit wrapper する担当グループ)が `try_context::<FormLayoutCx>()` を読む。
- vertical → `column(..).style(style().gap(16))`。horizontal → `grid(..).style(style().grid_auto_flow(GridAutoFlow::Column).grid_auto_columns(Track::fr(1)).gap(16))`。
- horizontal-labels は CSS の `display:contents` が無いので、FormLayout が `grid(..)` (`grid_template_columns((auto(), Track::fr(1)))`, row-gap 12 / column-gap 16)を作り、**field が context を見て「ラベル」と「入力側」の 2 つの兄弟 View を返す**(`field` の into_view が 2 要素の fragment を返す)。fragment の子が grid の直接の子になるかは kivact の fragment 実装次第 → UNRESOLVED(駄目なら FormLayout が `soft::form_row(label, control)` を明示的に受ける API にする)。
- 480px 崩し: FormLayout の `node_ref().rect()` ではなく **viewport 幅** 基準(`@media`)なので `kivact::platform::window_size().width <= 480.0` で column に切り替える reactive style。
- 最も近い basic: crates/kivact-ui-basic/src/field.rs(`field_group` / `field_set`, orientation 付き)。

### 7.8 受け入れ確認
- [ ] vertical で項目間 16px。
- [ ] horizontal で 3 項目が等幅 3 列(gap 16px)。
- [ ] horizontal-labels でラベル列が最長ラベル幅、入力列が残り、行間 12px、列間 16px、ラベル文字が入力文字のベースラインと揃う(上 5px)。
- [ ] ウィンドウ幅 480px 以下で horizontal-labels が縦積み(ラベル上・入力下、gap 16px)。
- [ ] `default_optionality(Required)` で無印項目に「Optional」表示が出ず、isOptional の項目だけに出る。


## 8. AspectRatio

docs: https://astryx.atmeta.com/components/AspectRatio 。source: packages/core/src/AspectRatio/AspectRatio.tsx。

**判定: 薄い kit 部品(style ヘルパ相当)。** kivact の `Style::aspect_ratio`(crates/kivact-style/src/sheet.rs:372-374)と `overflow(Hidden)` でほぼ 1:1。kivact-ui-basic にも `basic::aspect_ratio`(crates/kivact-ui-basic/src/primitives.rs)がある。

### 8.1 概要
- 幅はコンテナから取り、高さを比率から導く。動画・画像・サムネイル用。祖先に確定幅が要る。
- 比率は分数で書く(`16/9`)。画像・動画は `fit="cover"` にして子に width/height/objectFit を重ねない。子は 1 個(fit 指定時は直接の子が全部引き伸ばされるので、キャプション等は 1 つの wrapper に入れる)。
- 一般レイアウト・入れ子・高さだけの制約・shrink-to-fit 親(inline-flex 等: 幅 0 に潰れる)は禁止(AspectRatio.doc.mjs bestPractices)。

### 8.2 Anatomy
```
Ratio box (div, position:relative; width:100%; aspect-ratio; overflow:clip)  ← shape=ellipse で border-radius 50%
└─ Content slot (div, position:absolute; inset 0; 100%×100%)  ← fit で子のサイズ決め
    └─ child(img / video / 任意)
```

### 8.3 API(AspectRatio.tsx:40-118)

| prop | type | default | 意味 | Kivact |
|---|---|---|---|---|
| ratio | number(必須) | — | width / height | `soft::aspect_ratio(ratio: f32, child)` |
| shape | `'rectangle' \| 'ellipse'` | `'rectangle'` | 楕円クリップ(ratio 1 で円) | `.shape(RatioShape::Ellipse)` |
| fit | `'cover' \| 'contain' \| 'center'` | なし(子が自分で決める) | cover: 子を 100%×100% にし img/video は object-fit cover; contain: 同 contain; center: 自然サイズで中央 | `.fit(RatioFit::Cover \| Contain \| Center)` |
| children | ReactNode(必須) | — | — | `child: impl IntoView` |
| xstyle / className / style / ref | — | — | 応答的に比率を変えるための逃げ道 | 破棄。応答比率は `ratio` に `impl IntoValue<f32>`(signal)を受ける |

### 8.4 Visual spec(AspectRatio.tsx:120-182)
- container: `position:relative; width:100%; overflow:clip; min-height:0; flex-shrink:0; aspect-ratio: <ratio>`。
- ellipse: `border-radius: 50%`(楕円クリップ。ratio 16/9 なら横長楕円)。
- child slot: `position:absolute; top:0; inset-inline-start:0; width:100%; height:100%`。
- fit center: slot が `display:flex; align-items:center; justify-content:center`。
- fit cover / contain: slot の直接の子すべてを `width:100%; height:100%`、`img/video` は `object-fit: cover|contain`。
- 描画なし(背景なし。媒体が無い領域は透明)。テーマ target `astryx-aspect-ratio`(shape)。

### 8.5 Motion
なし。

### 8.6 Behaviour & a11y
role / 名前を付けない。子(画像の alt 等)が説明を持つ。

### 8.7 Kivact 実装方針
- `node().style(style().width(pct(100)).aspect_ratio(ratio).overflow(Overflow::Hidden).shrink(0).min_height(0))` + 子を `stack`(重ね)ではなく絶対配置 `position(Position::Absolute)` + inset 0 で入れる。
- ellipse: `radius(CornerRadius::..)` は円弧半径なので 50% 楕円は `clip-path`(sheet.rs:424 の `clip_path` setter)に楕円 path を与える。kivact に `pct` 半径があれば `radius(pct(50))`(UNRESOLVED: CornerRadius が % / 楕円を受けるか; sheet.rs:682 のコメントは「elliptical ones too」と影を説明しているので楕円半径はある可能性が高い)。
- cover / contain: kivact の image 要素の fit(`ImageFit::Cover` 等)を子が image のときに設定。任意の子は 100%×100% に伸ばすだけ。
- 既存 `basic::aspect_ratio`(crates/kivact-ui-basic/src/primitives.rs)の実装をそのまま流用し、shape と fit を追加する。

### 8.8 受け入れ確認
- [ ] 幅 320px の親で `aspect_ratio(16/9)` の高さが 180px。
- [ ] `shape(Ellipse)` + ratio 1 で円形にクリップ。
- [ ] `fit(Cover)` の画像が箱を埋めて切り抜かれ、`Contain` で余白(透明)が出る。
- [ ] `fit(Center)` で小さいアイコンが中央、拡大されない。


## 9. Divider

docs: https://astryx.atmeta.com/components/Divider 。source: packages/core/src/Divider/Divider.tsx。

**判定: 本物の kit コンポーネント(小)。** `role=separator` と aria-orientation、ラベル、full-bleed(0.2 の ContainerInset を読む)を持つ。最も近い basic は `basic::separator`(crates/kivact-ui-basic/src/primitives.rs)。

### 9.1 概要
- 内容の区切り。関連する節の間は subtle、強いコントラストが要る境界は strong。見出し代わりのラベル付きも可。
- 乱用しない。まず余白とレイアウトで区切る(Divider.doc.mjs bestPractices)。

### 9.2 Anatomy
```
Divider group (div role=separator aria-orientation=… [aria-labelledby=label])
├─ Rule (div, 1px 線)
├─ Label (div, 任意)             ← label があるときだけ
└─ Rule (div, 1px 線)            ← label があるときだけ 2 本目
```

### 9.3 API(Divider.tsx:38-68)

| prop | type | default | 意味 | Kivact |
|---|---|---|---|---|
| orientation | `'horizontal' \| 'vertical'` | `'horizontal'` | 向き | `soft::divider().orientation(Orientation::Horizontal \| Vertical)`(kivact の `Orientation`) |
| label | ReactNode | なし | 中央ラベル(数値 0 は表示、null/bool/空文字は非表示) | `.label(|| text("OR"))`(slot closure)または `.label_text(impl IntoText)` |
| variant | `'subtle' \| 'strong'` | `'subtle'` | 線の濃さ | `.variant(DividerVariant::Subtle \| Strong)` |
| isFullBleed | boolean | false | 親容器 padding を負 margin で打ち消し端まで伸ばす | `.full_bleed(true)` |
| aria-label / aria-labelledby | string | なし | 明示名(label より優先) | `.name(..)`(NodeBuilder) |
| xstyle / className / style / ref | — | — | — | 破棄 / NodeBuilder |

### 9.4 Visual spec(Divider.tsx:70-131)
- horizontal group: `display:flex; align-items:center; width:100%`。vertical group: `display:inline-flex; flex-direction:column; align-items:center; height:100%`。**group 自体に margin は無い**(上下の余白は親の gap で作る)。
- Rule: 太さ `--border-width` = 1px(horizontal は height、vertical は width)、`flex: 1 1`(残りを等分)。
  - subtle: `background: --color-border`(light `#00000014` / dark `#FFFFFF1A`)
  - strong: `background: --color-border-emphasized`(light `#d4d4d4` / dark `#525252`)
- Label: `flex-shrink:0; padding-inline: --spacing-3`(左右 12px、vertical では `padding-inline:0; padding-block:12px`)、`font-size: --text-supporting-size` 12px、`line-height: --text-supporting-leading` 1.6667(= 20px)、`color: --color-text-secondary`(light `#474747` / dark `#9e9e9e`)。font-weight と family は継承(neutral では Figtree 400)。
- isFullBleed horizontal: `margin-inline: calc(-1 * --container-padding-inline-*)`, `width: calc(100% + start + end)`。vertical: block 方向に同様(height を伸ばす)。
- テーマ target `astryx-divider`(orientation, variant)。

```
───────────────────────── OR ─────────────────────────
  rule (1px, flex 1)    ←12→ label 12px/20px ←12→   rule
```

### 9.5 Motion
なし。

### 9.6 Behaviour & a11y
- `role=separator`、`aria-orientation`。ラベルがあり `aria-label` が無ければラベル要素の id で `aria-labelledby`(Divider.tsx:159-175)。フォーカスしない(静的 separator)。
- Kivact: `.role(Role::Separator)` + `.orientation(..)`。ラベルがあるとき `labelled_by(label_ref)` か、ラベル文字列を `.name(..)` に入れる。

### 9.7 Kivact 実装方針
- `soft::divider() -> Divider`(NodeBuilder)。horizontal = `row((rule, label?, rule?))`、rule = `node().style(style().height(1).grow(1).shrink(1).background(soft::Color::Border))`。
- full bleed: `try_context::<ContainerInset>()`(0.2)で `margin_left(-start).margin_right(-end)`、幅は親幅 + start + end(`width(pct(100))` に負 margin で十分かは kivact の flex 実装依存: 負 margin で box が広がらない場合は `width(Length::calc(pct(100) + start + end))` を使う)。
- `basic::separator`(crates/kivact-ui-basic/src/primitives.rs)をラベル・variant・bleed 付きに拡張する形。

### 9.8 受け入れ確認
- [ ] role separator、orientation horizontal/vertical が a11y tree に出る。
- [ ] subtle 線は light `#00000014` / dark `#FFFFFF1A`、strong は `#d4d4d4` / `#525252`、太さ 1px。
- [ ] `label("OR")` で線 2 本 + 中央ラベル(12px/20px, 左右 12px 余白, secondary 色)、名前が "OR"。
- [ ] padding 12px の card 内で `full_bleed(true)` が card の左右端まで届く。
- [ ] vertical divider が親 row の高さいっぱいに伸びる。


## 10. Resizable / ResizeHandle / useResizable

docs: https://astryx.atmeta.com/components/Resizable (hook: useResizable)。source: packages/core/src/Resizable (ResizeHandle.tsx, useResizable.ts, resizableModel.ts, utils.ts, Resizable.doc.mjs, useResizable.doc.mjs, ResizeHandle.test.tsx, useResizable.test.ts)。

**判定: RESTYLE + kit wrapper。** ドラッグ・矢印キー・Home/End・ARIA の value semantics は `kivact::interaction::use_splitter`(crates/kivact-interaction/src/splitter.rs)と同じ。標準 `kivact::ui::splitter` / `split`(`.grip(true)`)の recipe を soft の「pill grip」に差し替えたうえで、Astryx 固有の **px 単位サイズ・折り畳み(collapse)・snap・Shift 大ステップ・ダブルクリック折畳み・永続化** を kit の `soft::use_resizable` + `soft::resize_handle` で包む。

### 10.1 概要
- hook 先行の設計: `useResizable()` がサイズ状態を持ち、`ResizeHandle` が pill 型グリップの separator。返り値の props を LayoutPanel / SideNav の `resizable` prop に渡す(余計な wrapper DOM を作らない)。
- handle が複数あるときは各 handle に label("Resize sidebar", "Resize terminal")。minSize を小さくし過ぎない(読めなくなる)。完全に隠せるパネルは collapsible に。`autoSaveId` でユーザーが選んだサイズを保存(Resizable.doc.mjs / useResizable.doc.mjs bestPractices)。

### 10.2 Anatomy
```
ResizeHandle (div role=separator tabindex=0 aria-orientation aria-value*)   ← 線(divider)。focus ring はここ
├─ Grab zone (div, absolute, 16px 幅, pointer 受け)                          ← 実際のドラッグ対象
└─ Grip pill (div, absolute, 3×32px, radius full)   ← children で置換可能
```
`position="overlay"` では handle 自体が隣接パネルの端に absolute で重なる(幅 16px の透明帯)。

### 10.3 API

useResizable(options)(useResizable.ts:60-200, useResizable.doc.mjs):

| option | type | default | 意味 | Kivact(`soft::use_resizable(ResizableConfig)`) |
|---|---|---|---|---|
| defaultSize | `number \| 'Npx' \| 'N%' \| pixel() \| percent(v,{min\|max})` | 250 | 初期サイズ。% は 1 回だけ px に解決(containerRef か viewport 基準) | `.default_size(ResizeSize::Px(250.0) \| Percent{ value, floor: Option<f32>, ceil: Option<f32> })` |
| minSize | 同上 | 50 | 下限(% は基準変化で再解決) | `.min_size(ResizeSize)` |
| maxSize | 同上 | Infinity | 上限 | `.max_size(ResizeSize)` |
| containerRef | RefObject | なし(viewport) | % の基準要素(未レイアウト時は 1200px 仮基準) | `.container(NodeRef)` |
| direction | `'horizontal' \| 'vertical'` | `'horizontal'` | 測る軸。handle と一致必須 | `.axis(Orientation)` |
| collapsible | boolean | false | 閾値未満のドラッグで 0 に折畳む | `.collapsible(bool)` |
| collapsedSize | number | 40 | 折畳み閾値 px | `.collapse_threshold(f32)` |
| snaps | number[] | なし | ドラッグ中、常に最寄りの snap 値に吸着(clampSize: resizableModel.ts:22-46) | `.snaps(Vec<f32>)` |
| shrinkOrder | number | なし | 複数 region のカスケード優先度(小さいほど先に縮む) | `.shrink_order(i32)`(UNRESOLVED: multi-region の実装詳細は未読。単一 region を先に実装) |
| autoSaveId | string | なし | localStorage `astryx-resizable:<id>` に `{size, isCollapsed}` | `.persist(key)` → kit はアプリ提供の保存 callback に委譲(`on_persist: impl Fn(PersistedResize)`)。永続化はアプリの責務(kivact-ui-basic README の方針) |
| defaultIsCollapsed | boolean | false | 初期折畳み(非制御) | `.default_collapsed(bool)` |
| isCollapsed / onCollapseChange | boolean / fn | 非制御 | 制御折畳み | `.collapsed(Binding<bool>)` |

返り値: `size`(px)、`isCollapsed`、`collapse()`、`expand()`、`resize(px)`、`props`(ResizableProps: handle / panel に渡す内部値)。Kivact: `ResizableState { size: Signal<f32>, collapsed: Signal<bool>, .. }`(Copy handle)に `collapse()`, `expand()`, `resize(px)`。

ResizeHandle(ResizeHandle.tsx:300-371):

| prop | type | default | 意味 | Kivact(`soft::resize_handle(state)`) |
|---|---|---|---|---|
| resizable | ResizableProps(必須) | — | hook の props | 第 1 引数 `ResizableState` |
| direction | `'horizontal' \| 'vertical'` | `'horizontal'` | horizontal = 左右に並ぶパネル(縦線、col-resize) | `.axis(Orientation)` |
| position | `'inline' \| 'overlay'` | `'inline'` | inline: 1px(または 0px)の flex 子 / overlay: パネル端に重なる 16px 帯 | `.overlay(bool)` |
| isReversed | boolean | false | 終了側パネルを制御(ドラッグ方向反転) | `.reversed(bool)` |
| isDisabled | boolean | false | 操作不可(tabindex -1) | `.disabled(..)`(NodeBuilder) |
| hasDivider | boolean | false | 全長 1px 線を見せる(同背景のパネル間) | `.divider(bool)` |
| isAlwaysVisible | boolean | **true** | 休止時も pill を表示 | `.always_visible(bool)` |
| pillPlacement | `'start'\|'end'\|'center'\|'auto'` | `'auto'` | pill の位置。auto = 内容側(isReversed から)、折畳み時は反転 | `.pill_placement(PillPlacement)` |
| label | string | "Resize handle" | aria-label | `.name(..)` / 既定は message |
| children | ReactNode | なし | pill + divider を置換 | `.grip(|| view)` |

### 10.4 Visual spec(ResizeHandle.tsx:123-298)

handle(inline, horizontal):
- `width: 1px; height: 100%; flex-shrink:0; display:flex; align/justify center; position:relative; cursor: col-resize`(vertical は `height:1px; width:100%; cursor: row-resize`)。
- hasDivider=false(既定): 背景透明、**幅 0**(レイアウト上の太さ 0。grab zone と pill だけが重なる)。
- hasDivider=true: 背景 `--color-border`(light `#00000014` / dark `#FFFFFF1A`)。hover/focus 中(ドラッグ中でない)も `--color-border`(変化なし)、ドラッグ中 `--color-border-emphasized`(light `#d4d4d4` / dark `#525252`)。
- disabled: `cursor: default; pointer-events: none`。
- overlay: `position:absolute; z-index:2; 透明`、horizontal は `inset-inline-end:0; top/bottom 0; width: var(--resize-handle-hit-area, 16px)`、vertical は `inset-block-end:0; height:16px`。

grab zone(hit area):
- `position:absolute; z-index:1; touch-action:none; user-select:none`、horizontal `width: --spacing-4` = 16px、上下いっぱい。
- 位置: pill が center のとき線の中央に centering(`centerInline(0px)`)。start/end のときは pill 側へ寄せる: `translateX(dir * (3px + 4px) - 6.5px)`(RTL は `+ 6.5px`)、dir = start −1 / end +1。vertical は translateY 同式。

pill(grip):
- `position:absolute; z-index:2; pointer-events:none; border-radius: --radius-full`(9999px)、`top: 50%`。
- horizontal: **3px × 32px**(`--spacing-8`)。vertical: 32px × 3px。
- 色: 休止 `--color-border`(`#00000014` / `#FFFFFF1A`)、hover/focus `--color-border`(opacity 1)、ドラッグ中 `--color-border-emphasized`(`#d4d4d4` / `#525252`)。
- 可視性: isAlwaysVisible → opacity 1、false → opacity 0(hover/focus/drag で 1)。
- 位置: center → 線の中央(`-50%`)。start/end → `translate(dir * (100% + 4px), -50%)`、つまり線から 4px 離れて pill 幅 3px ぶん外側(start = 左/上、end = 右/下。RTL で論理反転)。
- auto の解決(ResizeHandle.tsx:69-82): `panelSide = isReversed ? end : start`。通常は panelSide 側(= 制御しているパネルの側)、**折畳み中は反対側**(内容の側)。

フォーカス: 標準リング(2px `--color-accent` light `#1b1b1b` / dark `#f1f1f1`、offset 3px)を handle に(`focusOutlineProps.focusVisible`)。

```
 panel A          │ panel B            pillPlacement=start, divider=true
                 ▌│                    ▌ = pill 3×32 (線から 4px 左)
                  │                    │ = 1px divider
          [ 16px grab zone ]
```

ドラッグ中は document.body に `cursor: col-resize|row-resize; user-select: none`(ResizeHandle.tsx:113-121)。

### 10.5 Motion
- handle 背景色: `transition: background-color --duration-fast(neutral 125ms) --ease-standard cubic-bezier(0.24, 1, 0.4, 1)`(ResizeHandle.tsx:131-133)。
- pill: `transition: opacity, background-color, transform 125ms cubic-bezier(0.24, 1, 0.4, 1)`(ResizeHandle.tsx:229-231)。pillPlacement auto の折畳み反転は transform の 125ms で横に移る。
- パネルサイズ自体は **アニメーションしない**(ドラッグ・キー・collapse/expand とも即時。SideNav の rail 折畳みは 15 で別途)。
- reduced motion: 既定 CSS transition のまま(明示の 0s 指定なし)。Kivact は decorative transition として reduced motion で省略してよい。

### 10.6 Behaviour & a11y
- role `separator`、`aria-orientation` は線の向き(horizontal handle → `vertical`)、`aria-valuenow` = size(折畳み中は `max(size, min)`)、`aria-valuemin` = min、`aria-valuemax` = 実効 max(有限時のみ)、折畳み中 `aria-valuetext` = "Collapsed"、`aria-label` = label、disabled で `aria-disabled` + tabindex −1(ResizeHandle.tsx:720-752)。
- キー(ResizeHandle.tsx:588-686): 
  - ←/→(horizontal)・↑/↓(vertical): 10px(Shift で 50px)。→/↓ が拡大(RTL の horizontal は反転、isReversed でさらに反転)。
  - snaps があるとき: 矢印は隣の snap 点へ(Shift で 2 つ先)。
  - 折畳み中: 拡大方向の矢印で `max(min + step, collapsedSize)` に展開。縮小方向は無視。
  - Home: min へ。End: max(有限時)へ。Enter: collapsible なら折畳み/展開トグル。
- ポインタ: grab zone の pointerdown でキャプチャしドラッグ(`delta * rtl * sign`)。ダブルクリックで collapsible なら折畳みトグル。pointercancel / lostpointercapture でドラッグ中止(サイズは途中値のまま: `_onResizeCancel` は基準解放のみ)。
- ドラッグ中、`raw < collapsedSize`(collapsible)で即折畳み(size 0 通知)、閾値以上に戻ると展開。snaps は最寄り値に常時吸着。
- 既定テキスト: `soft.resizable.handle` = "Resize handle"(`@astryx.resizable.handle.label`)、`soft.resizable.collapsed` = "Collapsed"(`@astryx.resizable.collapsed`)。

### 10.7 Kivact 実装方針
- `soft::use_resizable(cfg) -> ResizableState`: px の `Signal<f32>` と `Signal<bool>`。% 指定は `container` の `rect()`(無ければ `window_size()`)で初回 1 回解決、min/max の % は reactive に再解決。
- `soft::resize_handle(state)`: `kivact::interaction::use_splitter(binding, range, axis)` を使う。binding は「書き込み時に collapse 閾値と snap を適用する」カスタム `Binding<f64>` にし、range = `[min, max]`(折畳みは 0 を書く特別経路)。`key_step(10.0)`。Shift+矢印の 50px と snap 隣接移動、Enter の collapse トグル、ダブルクリックは use_splitter に無い → handle ノードに先に key handler を付けて処理し、残りを splitter に渡す(UNRESOLVED: use_splitter の Enter = `reset_to` と衝突するので reset_to は設定しない)。Escape でのドラッグ取消は use_splitter が開始値に戻す(Astryx は戻さない; kivact の挙動で可)。
- 見た目は `kivact::ui::split` / `splitter` の recipe(`SplitterRecipes` / `SplitRecipes`)を soft 用に書き、pill(3×32、radius full)と 16px の hit area を描く。kivact-ui-basic の resizable.tsx 版 recipe(crates/kivact-ui-basic/src/recipes_controls.rs:322-400 の `splitter` / `splitter_handle` / `split` / `split_grip`)を参考に。
- LayoutPanel / SideNav の `.resizable(state)` は `state.size` を width に使う。
- カーソル: `style().cursor(Cursor::EwResize | NsResize)`。ドラッグ中のウィンドウ全体カーソルは use_drag のキャプチャ中ポインタで代替(UNRESOLVED: kivact でドラッグ中グローバルカーソルを固定できるか)。

### 10.8 受け入れ確認
- [ ] role separator、名前 "Resize handle"(または指定 label)、value now/min/max が px で読める。
- [ ] →で 10px、Shift+→で 50px 拡大、Home で min、End で max。RTL では ← で拡大。
- [ ] collapsible で 40px 未満までドラッグすると 0 に折畳み、aria-valuetext "Collapsed"。Enter とダブルクリックで折畳み/展開トグル。
- [ ] snaps [200, 300, 400] でドラッグ中に 3 値にしか止まらない、矢印で隣の snap へ。
- [ ] pill は 3×32px、線から 4px 離れて内容側、ドラッグ中だけ `#d4d4d4`(dark `#525252`)。always_visible(false) で hover 時だけ出る(125ms フェード)。
- [ ] divider(true) で 1px の `#00000014` / `#FFFFFF1A` 線、false で線なし(太さ 0)。


## 11. ScrollableArea (+ useScrollableArea, useScrollOverflow, useOverflow)

docs: https://astryx.atmeta.com/components/ScrollableArea 。source: packages/core/src/ScrollableArea/ScrollableArea.tsx; hooks: packages/core/src/hooks/useScrollableArea.ts, useScrollOverflow.ts, useOverflow.ts, computeOverflow.ts, scrollGeometry.ts, scrollKeyboardDelegation.ts, scrollbarGutter.ts。

**判定: RESTYLE + kit wrapper。** 本体は kivact の `scroll(child).axes(..)`(crates/kivact-core/src/control/container.rs:75-168)。soft はスクロールバーの色、「溢れているときだけタブ停止する名前付き viewport」、padding の告知、full-bleed を足す。kivact-ui-basic の対応: crates/kivact-ui-basic/src/scroll_area.rs(`basic::scroll_area`)。**オーバーフロー影/フェードは ScrollableArea 自体には無い**(useScrollOverflow を使う側、例: Carousel / TabList が描く)。

### 11.1 概要
- ネイティブのスクロール viewport と、観測される実 content box を持つ。viewport は「要求軸が実際にスクロール可能なときだけ」タブ順に入り、overscroll 抑止も実効軸だけに掛かる。
- 必ず簡潔な `label`(キーボード利用者がスクロールする内容の名前)。軸は内容の意図で inline / block / both。入れ子は既定の `overscroll="allow"` のまま。
- 子を直接の flex/grid 子のままにしたい・% 高さ基準を保ちたい場合は hook(useScrollableArea)を使う。
- ネイティブスクロールバーを、代わりの可視・操作可能なオーバーフロー手掛かり無しに隠さない。ScrollableArea の外にさらに overflow wrapper を重ねない(ScrollableArea.doc.mjs bestPractices)。

### 11.2 Anatomy
```
Viewport (div role=group|region aria-label=label tabindex=0 only while scrollable)  .astryx-scrollable-area
└─ Content box (div; min 100%×100%; inline/both 軸では inline-size: max-content)  ← padding はここ
    └─ children
```
data 属性で状態を公開: `data-scroll-axis`, `data-scrollable-inline|block`, `data-scroll-inline-start|end`, `data-scroll-block-start|end`(useScrollableArea.ts:540-560)。

### 11.3 API

ScrollableArea(ScrollableArea.tsx:105-147):

| prop | type | default | 意味 | Kivact(`soft::scrollable_area(label, child)`) |
|---|---|---|---|---|
| label | string(必須) | — | viewport の名前 | 第 1 引数 `impl IntoText` |
| axis | `'inline' \| 'block' \| 'both'` | `'block'` | スクロール軸(論理) | `.axis(ScrollAxis::Inline \| Block \| Both)` → `ScrollAxes::{Horizontal, Vertical, Both}` |
| role | `'group' \| 'region'` | `'group'` | viewport の semantics | `.role(Role::Group \| Role::Region)` |
| overscroll | `'allow' \| 'contain'` | `'allow'` | 端で祖先へスクロールを渡すか | `.overscroll(Overscroll::Allow \| Contain)` |
| stickyContainment | `'whenScrollable' \| 'always'` | `'whenScrollable'` | 収まっているとき sticky の境界を外へ渡すか | `.sticky_containment(..)`(kivact では「溢れていない時は scroll ノードを作らない」かどうかの選択。下記) |
| padding / paddingInline / …Start/End / paddingBlock / … | SpacingStep | padding 0 | content box の padding(告知もする) | `.padding(Space)` ほか |
| isFullBleed | boolean | false | viewport が親容器の padding を打ち消す | `.full_bleed(true)` |
| width / height / maxWidth / minHeight | SizeValue | なし | viewport のサイズ | `impl Into<Length>` |
| children | ReactNode | — | — | `child: impl IntoView` |
| xstyle / className / style / ref | — | — | — | 破棄 / NodeBuilder |

useScrollableArea(options) → `{getViewportProps, getContentProps, state}`。options: `axis`、`keyboardAccess: {owner:'viewport', label, role?} | {owner:'content'} | {owner:'contentOrViewport', label, role?}`、`overscroll`、`stickyContainment`。state: inline/block それぞれ `{isScrollable, atStart, atEnd}`。Kivact: `soft::use_scroll_state(viewport: NodeRef, content: NodeRef, axes) -> ScrollState { inline: AxisScroll, block: AxisScroll }`(各 `Memo<bool>`)。

useScrollOverflow() → `{scrollRef, overflowStart, overflowEnd, hasOverflow}`(水平のみ、1px 許容)。Kivact: `soft::use_scroll_overflow(viewport: NodeRef) -> ScrollOverflow { start: Memo<bool>, end: Memo<bool>, any: Memo<bool> }`(`rect()` と `scroll_offset()` と content 幅から計算)。

useOverflow(itemCount, {gap=0, minVisibleItems=0, collapseFrom='end', behavior='observeSelf'}) → `{containerRef, measureRef, visibleCount, hasOverflow}`。隠し測定コンテナに全項目 + `data-overflow-indicator` の「+N」要素を描き、利用可能幅に入る数を返す(computeOverflow.ts:73-107: 先頭から幅 + gap を足し、最後の項目以外では indicator 幅 + gap を予約、`minVisibleItems` 未満では止めない)。Kivact: `soft::use_overflow(widths: Memo<Vec<f32>>, available: Memo<f32>, indicator: Memo<f32>, opts) -> Memo<usize>`(測定は各項目の `NodeRef::rect()`; 隠し測定コンテナは opacity 0 + absolute で同じ子を描く)。Breadcrumbs / TabList / TopNav の「収まらない分を More メニューへ」に使う。

### 11.4 Visual spec(ScrollableArea.tsx:47-92)
- viewport: `box-sizing:border-box; max-inline-size:100%; max-block-size:100%`。overflow は hook が決める: 溢れている軸(または always のとき要求軸)は `auto`、要求外の軸は `hidden`、溢れていない & whenScrollable なら `clip`(useScrollableArea.ts:490-508)。
- スクロールバー: `scrollbar-color: --color-neutral transparent`(thumb light `#0000000F` / dark `#FFFFFF1A`、track 透明)、`scrollbar-width: auto`、`scrollbar-gutter: auto`。coarse pointer と forced-colors では `auto`(OS 既定)。Kivact: `style().scrollbar_color(soft::Color::Neutral, Color::TRANSPARENT).scrollbar_width(ScrollbarWidth::Auto)`(crates/kivact-style/src/sheet.rs:458-463)。
- content: `min-inline-size:100%; min-block-size:100%`、inline/both では `inline-size: max-content`(内容幅で横に溢れる)。padding は 0.1 の表(既定 0)。
- focus-visible: 標準リング(2px `--color-accent` light `#1b1b1b` / dark `#f1f1f1`、offset 3px)を viewport に。
- isFullBleed: viewport に `margin-inline: -container-padding-inline-*`, `max-inline-size: calc(100% + start + end)`, 先頭なら上、末尾なら下も打ち消す。
- 背景・枠なし。テーマ target `astryx-scrollable-area`(axis)。

### 11.5 Motion
なし(ネイティブのスクロールのみ。smooth scroll 指定なし)。

### 11.6 Behaviour & a11y
- viewport の role(group / region)と名前は常に付く(owner が viewport / contentOrViewport のとき)。`tabindex=0` は実効軸がある(溢れている)ときだけ。溢れていないときは tabindex を外す(プログラム focus 中だけ −1 を保持)。
- フォーカス中は矢印・PageUp/PageDown・Home/End・Space でネイティブスクロール。
- `contentOrViewport`: 前方 Tab 進入時、内容の最初の sequential な要素が「普通のリンクかボタン」(PASSIVE_ROLES 内)ならそちらへ委譲し viewport を飛ばす。入力欄・複合ウィジェット・入れ子のスクロール領域なら viewport に止まる。Shift+Tab で委譲先から戻るときは viewport を飛ばす(scrollKeyboardDelegation.ts)。
- overscroll contain: 実効軸の端で祖先にスクロールを渡さない。
- 既定テキストなし(label は呼び出し側必須)。

### 11.7 Kivact 実装方針
- `soft::scrollable_area(label, child) -> ScrollableArea`: `scroll(column(child).style(content_style)).axes(..).role(..).name(label)`。
- 溢れ判定: viewport と content の `NodeRef::rect()` を比較(1px 許容)して `overflowing: Memo<bool>`。`.focusable(move || overflowing.get())`(NodeBuilder::focusable は reactive 値を取る: crates/kivact-core/src/view/node.rs:267)。
- kivact の `scroll` は「Wheel at the edge of a short area passes to the parent」(container.rs:73)= allow 相当。contain は UNRESOLVED(kivact に overscroll 制御が無ければ allow のみ実装)。
- stickyContainment: kivact の sticky は「最も近いスクロール祖先」基準なので、whenScrollable を再現するには溢れていないとき scroll ノードを作らない必要がある → 実装コストに見合わないので **always 相当の挙動で固定**し差分を記録(UNRESOLVED)。
- contentOrViewport の Tab 委譲: kivact のフォーカス順を制御する API が必要(UNRESOLVED)。最低限 owner=viewport のみ実装。
- 最も近い basic: crates/kivact-ui-basic/src/scroll_area.rs と crates/kivact-ui-basic/src/message_scroller.rs(rect と scroll_offset の読み方)。

### 11.8 受け入れ確認
- [ ] 内容が収まっているとき Tab で止まらず、溢れると止まる(名前 = label、role group)。
- [ ] フォーカス中に ↓ / PageDown / End でスクロール。focus-visible でリング(2px, offset 3px)。
- [ ] スクロールバー thumb が light `#0000000F` / dark `#FFFFFF1A` 相当で track 透明。
- [ ] `axis(Inline)` で長い 1 行が横スクロール、`padding(S4)` が内容の周りに 16px。
- [ ] `use_scroll_overflow` が左端で start=false/end=true、右端で start=true/end=false、RTL で論理反転。
- [ ] `use_overflow` で幅を縮めると見える数が減り、+N indicator 分を予約する。


## 12. Breadcrumbs / BreadcrumbItem

docs: https://astryx.atmeta.com/components/Breadcrumbs (BreadcrumbItem は sub-component)。source: packages/core/src/Breadcrumbs (Breadcrumbs.tsx, BreadcrumbItem.tsx, docs, Breadcrumbs.test.tsx)。storybook: Breadcrumbs.stories.tsx, BreadcrumbItem.stories.tsx, BreadcrumbsA11y.stories.tsx。

**判定: NEW kit component**(項目メニューは RESTYLE `kivact::ui::menu`)。最も近い basic: crates/kivact-ui-basic/src/breadcrumb.rs(`breadcrumb`, `breadcrumb_link`, `breadcrumb_page`, `breadcrumb_separator`, `breadcrumb_ellipsis`)。**Astryx Breadcrumbs は自動省略(ellipsis)を持たない**(長い trail は折り返す)。

### 12.1 概要
- root から現在ページまでのリンクの列。詳細ページ・設定パネルの見出しの上に置く。ラベルは短く、リンク先ページのタイトルと一致させる。
- 密な UI(管理画面・サイドバー)では `variant="supporting"`。最後の項目は現在ページで、リンクにしない(isCurrent で自動)。
- 複数の trail があるページでは各 trail に別 label。主ナビゲーションの代わりにしない、親の無いトップページに出さない、5 階層を超えない(Breadcrumbs.doc.mjs bestPractices)。
- 既定の "/" は RTL で自動ミラー。`›` のような Bidi_Mirrored な文字は自前で反転しない。矢印 `→` や Icon 区切りは呼び出し側で mirror する。

### 12.2 Anatomy
```
nav (aria-label="Breadcrumb")                 .astryx-breadcrumbs
└─ ol (flex, wrap, gap 4px, list-style none)
    └─ li (BreadcrumbItem, flex, gap 4px)     .astryx-breadcrumb-item
        ├─ separator span[aria-hidden]  "/"   ← 先頭の li では display:none
        └─ 次のいずれか
           ├─ a[href] (link)               ← href あり
           ├─ button (link 見た目)         ← onClick のみ
           ├─ button[aria-haspopup=menu] + chevron ▾ → popover menu (role=menu)  ← menu あり
           └─ span (current / 非リンク)     ← isCurrent か href も onClick も無い
              [startIcon] + label
```

### 12.3 API

Breadcrumbs(Breadcrumbs.tsx:68-92):

| prop | type | default | Kivact(`soft::breadcrumbs(items)`) |
|---|---|---|---|
| children | BreadcrumbItem[] | 必須 | `items: impl IntoView`(`soft::breadcrumb_item` のタプル/Vec) |
| separator | ReactNode | `'/'` | `.separator(|| view)`(既定 "/" テキスト) |
| variant | `'default' \| 'supporting'` | `'default'` | `.variant(BreadcrumbsVariant::Default \| Supporting)` |
| label | string | "Breadcrumb" | `.name(..)`(既定 message `soft.breadcrumbs.label`) |
| xstyle / className / style / ref | — | — | 破棄 / NodeBuilder |

BreadcrumbItem(BreadcrumbItem.tsx:81-136):

| prop | type | default | 意味 | Kivact(`soft::breadcrumb_item(label)`) |
|---|---|---|---|---|
| children | ReactNode | 必須 | ラベル | `label: impl IntoText`(または `.content(|| view)`) |
| href | string | なし | リンク先 | **native では無意味** → `.on_press(impl Fn())`(ナビゲーションはアプリ) |
| onClick | (e) => void | なし | クリック | `.on_press(..)` |
| isCurrent | boolean | 自動(最後の項目、明示 current が無ければ) | `aria-current=page`、太字、リンクにしない。`false` で自動を拒否 | `.current(Option<bool>)` |
| startIcon | ReactNode | なし | ラベル前のアイコン | `.icon(icon("house"))` |
| menu | DropdownMenuOption[] \| ReactNode | なし | 兄弟ページのメニュー(href/onClick より優先) | `.menu(Vec<MenuEntry>)` → 標準 `kivact::ui::menu` |
| menuSize | `'sm' \| 'md' \| 'lg'` | supporting → sm、他 → md | メニュー項目サイズ | `.menu_size(ControlSize)` |
| as | LinkComponentType | なし | ルーターのリンク要素 | 破棄 |

### 12.4 Visual spec(BreadcrumbItem.tsx:142-264, Breadcrumbs.tsx:98-114)
- ol: `display:flex; align-items:center; flex-wrap:wrap; gap: --spacing-1` = 4px。margin/padding 0。
- li: `display:flex; align-items:center; gap: 4px`。先頭 li は separator を `display:none`。
- 文字: default → `--text-body-size` 14px / `--text-body-leading` 1.4286(= 20px)。supporting → `--text-supporting-size` 12px / 1.6667(= 20px)。family は継承(neutral Figtree)。
- link / button / menu trigger: `display:flex; align-items:center; gap:4px; padding-block: 4px`(inline 0)、`color: --color-text-secondary`(light `#474747` / dark `#9e9e9e`、default も supporting も同じ)、下線なし、hover(hover 端末、非 disabled)で **underline**、cursor pointer。背景なし、角丸なし。
- current: `font-weight: --font-weight-semibold` 600。default は `--color-text-primary`(light `#000000` / dark `#ffffff`)、supporting は `--color-text-secondary`(`#474747` / `#9e9e9e`)。padding なし(リンクは padding-block 4px あるので行高が揃う点に注意: current span には padding が無い。li の align-items center で中央揃え)。
- 非リンク(href / onClick / menu いずれも無く、current でもない)も current と同じ見た目(semibold + primary/secondary)だが aria-current は付かない(自動判定で最後なら付く)。
- separator: `color: --color-text-secondary`、`padding-block: 4px`、`user-select:none`、`aria-hidden`。既定 "/" は RTL でミラー(`rtlStyles.mirror` = scaleX(-1))。
- startIcon wrapper: `display:flex; align-items:center; flex-shrink:0`(アイコンサイズは呼び出し側。doc 例は `size="sm"` = 16px)。
- menu trigger の chevron: `chevronDown`(Lucide `chevron-down`)、size xsm、箱 `--text-supporting-size` = 12×12px、色 inherit(= secondary)。
- menu popover(BreadcrumbItem.tsx:246-264): `min-width:160px; margin-block:4px`、placement below / align start。中身 `display:flex; flex-direction:column; gap: --spacing-0-5` = 2px; `padding: 4px; max-height:300px; overflow-y:auto; border-radius: --radius-container` = 12px; user-select none。背景・影は popover 既定面(packages/core/src/Popover/usePopover.tsx:83-88): `--color-background-popover`(light `#ffffff` / dark `#1b1b1b`)、`border-radius: --radius-container` 12px、`box-shadow: --shadow-low`(light `0 2px 4px rgba(0,0,0,.05), 0 4px 8px rgba(0,0,0,.10)` / dark `0 2px 4px rgba(0,0,0,.25), 0 4px 8px rgba(0,0,0,.40), inset 0 0 0 1px rgba(255,255,255,.08)`)、枠線なし。
- focus-visible: link / button / trigger に標準リング(2px `--color-accent` light `#1b1b1b` / dark `#f1f1f1`、offset 3px)。
- テーマ target: `astryx-breadcrumbs`(variant), `astryx-breadcrumb-item`(variant), `astryx-breadcrumb-item-menu-trigger`(variant), `astryx-breadcrumb-menu`。

```
Home / Projects / Astryx ▾ / Settings
^secondary  ^secondary  ^menu trigger  ^current: semibold primary
     ←4→/←4→
```

### 12.5 Motion
- menu popover の出現: 0.6 の Layer below(165ms, `cubic-bezier(0.24, 1, 0.4, 1)`, `opacity 0 / translateY(-8px) / scale(0.95)` → 1)。閉じは即時。
- 下線の出現は transition 無し(即時)。

### 12.6 Behaviour & a11y
- WAI-ARIA APG Breadcrumb: `nav` landmark(名前 "Breadcrumb")+ `ol` + current 項目に `aria-current="page"`。自動判定: isCurrent 未指定の最後の項目に、明示 `aria-current="page"` の項目が他に無ければ付与(BreadcrumbItem.tsx:341-371)。
- 各リンク/ボタンは Tab 停止。Enter / クリックで activate。
- menu 項目(APG Menu Button): trigger は `aria-haspopup=menu`, `aria-controls`。Enter / Space / ArrowDown で開いて先頭項目にフォーカス。メニュー内は ↑↓(wrap しない)、typeahead、Enter/Space で選択、Escape で閉じ trigger へ、Tab で閉じる。光で外側クリック閉じ(light dismiss)。
- 既定テキスト: `soft.breadcrumbs.label` = "Breadcrumb"(`@astryx.breadcrumbs.label`)。

### 12.7 Kivact 実装方針
- `soft::breadcrumbs(items)`: `row(..).role(Role::Navigation).name(label)` 内に `row(..).style(style().wrap(true).gap(4))`(ol 相当の list role: `Role::List`、各項目 `Role::ListItem`)。
- 項目: リンク見た目の press 可能ノード(`kivact::interaction::use_button` + `Role::Link`)、`hover(style().underline())` は text の装飾なので state style で可能か要確認(state style は paint のみ; 下線は text 属性なので reactive style で `node_ref.hovered()` を読む)。current は `Role::Link` を付けずテキストのみ + 「現在ページ」状態(kivact に aria-current 相当が無ければ name に含めない。UNRESOLVED: kivact の semantics に current フラグがあるか)。
- separator: `text("/")` を `aria-hidden` 相当(`.hidden_from_accessibility(true)` 等。UNRESOLVED: API 名)。RTL ミラーは `scale_xy(-1, 1)`。
- menu: `kivact::ui::menu(label, entries).attach(trigger)` を restyle(menu recipe は DropdownMenu 担当グループ)。trigger に chevron-down 12px。
- 自動 current: `breadcrumbs` が items の Vec を受けるので、最後の項目かつ明示 current が無いとき current にする(DOM 探索ではなく builder 時に決定)。
- 参考: crates/kivact-ui-basic/src/breadcrumb.rs(構造とキー `basic.breadcrumb.label`)。

### 12.8 受け入れ確認
- [ ] navigation landmark "Breadcrumb"、list 内に項目、最後が current(リンクでない、semibold、light `#000000` / dark `#ffffff`)。
- [ ] リンク色 light `#474747` / dark `#9e9e9e`、hover で下線、Tab でリング。
- [ ] 区切り "/" が項目間 4px、先頭の前には無い、RTL で反転。
- [ ] supporting で 12px 文字、current も secondary 色。
- [ ] menu 項目は ▾ 付き、Enter で開き先頭にフォーカス、Escape で閉じて trigger に戻る。出現が 165ms でふわっと下向きに出る。
- [ ] 幅が足りないと 2 行に折り返す(省略しない)。


## 13. Outline

docs: https://astryx.atmeta.com/components/Outline 。source: packages/core/src/Outline (Outline.tsx, useScrollSpy.ts, useOutlineFromDOM.ts, useOutlineFromMarkdown.ts, parseOutlineFromMarkdown.ts, types.ts, OutlineFocus.a11y.chromium.spec.ts)。

**判定: NEW kit component。** 同一ページ内の見出しへの目次。roving list は `kivact::interaction::use_list`(vertical, 単一タブ停止)、scroll-spy は `NodeRef::rect()` + `scroll_offset()` で自作。kivact-ui-basic に直接の対応なし(縦リストの見た目は crates/kivact-ui-basic/src/sidebar.rs の menu button が近い)。

### 13.1 概要
- ドキュメント、ヘルプセンター、wiki、長い設定ページ用の目次サイドバー。アプリのルート遷移には使わない(→ SideNav / TopNav)、展開できる階層には使わない(→ TreeList)。
- フラットな見出しリストを渡し、`level` でインデント。独自スクロールロジックが active を持つときは `activeId`(制御モード、内蔵 scroll-spy 無効)。密なサイドバーは `density="compact"`。
- 内容が viewport 以外(分割ペイン、モーダル、パネル)でスクロールするときは `scrollContainerRef`。固定ヘッダが重なるなら `offset` にその高さ。`onNavigateEnd` は「到着」ではない(ユーザーが中断しても呼ばれる)(Outline.doc.mjs bestPractices)。

### 13.2 Anatomy
```
nav (aria-label="Table of contents", flex row, gap 2px)        .astryx-outline
├─ track (div aria-hidden, 2px 幅, order -1 → 視覚的に先頭/左)
│   └─ divider line (2px, --color-border, radius full, 上下いっぱい)
├─ ul role=list (flex column, gap 2px, flex 1, min-width 0)
│   └─ li role=listitem
│       └─ a[href="#id"] [aria-current=location]  (roving tabindex)   .astryx-outline-item
│           └─ span label (1 行 ellipsis)
└─ indicator (span aria-hidden, 2px, --color-icon-primary)  ← active リンクの top/height に追従   .astryx-outline-indicator
```

### 13.3 API(Outline.tsx:59-134, types.ts:12-21)

| prop | type | default | 意味 | Kivact(`soft::outline(items)`) |
|---|---|---|---|---|
| items | `{id, label, level(1-6)}[]` | 必須 | 見出し | `items: impl IntoValue<Vec<OutlineItem>>`、`OutlineItem { id: Key, label: Text, level: u8, target: NodeRef }`(native は id で要素を引けないので見出しの `NodeRef` を持たせる) |
| activeId | string | 非制御(scroll-spy) | 制御 active | `.active(Binding<Option<Key>>)` |
| onActiveIdChange | (id) => void | なし | 変化通知 | `.on_active_change(impl Fn(Key))` |
| label | string | "Table of contents" | nav 名 | `.name(..)` |
| density | `'default' \| 'compact'` | `'default'` | 項目 padding | `.density(OutlineDensity::Default \| Compact)` |
| onNavigateStart / onNavigateEnd | (id) => void | なし | スクロール開始 / 終了(中断含む、必ず 1 回ずつ) | `.on_navigate_start(..)` / `.on_navigate_end(..)` |
| offset | number | 0 | 固定ヘッダの高さ px(活性化ラインと着地点を同量ずらす) | `.offset(f32)` |
| scrollContainerRef | RefObject | 最寄りのスクロール祖先 | 監視するスクロール容器 | `.scroll_container(NodeRef)`(native では **必須** にする: 祖先探索が無いため) |
| hasScrollOnClick | boolean | true | 押下でスムーズスクロールするか | `.scroll_on_press(bool)` |
| xstyle / className / style / ref / data-testid | — | — | — | 破棄 / NodeBuilder |

useOutlineFromDOM / useOutlineFromMarkdown: DOM / Markdown から items を作る補助。Kivact では Markdown 担当(別グループ)の見出し情報から `Vec<OutlineItem>` を作る関数 `soft::outline_items_from(..)` を後で用意(本グループでは API 名だけ予約)。

### 13.4 Visual spec(Outline.tsx:140-250)
- root: `display:flex; flex-direction:row; position:relative; gap: --spacing-0-5` = 2px; width 100%。
- track: 幅 2px、`order:-1`(リストの inline-start 側)。line: `position:absolute; 上下 0; width 2px; background: --color-border`(light `#00000014` / dark `#FFFFFF1A`); `border-radius: --radius-full`。
- indicator: 幅 2px、`background: --color-icon-primary`(light `#000000` / dark `#ffffff`)、radius full、z-index 1、`top` / `height` は active リンクの top / height(CSS anchor positioning)。
- list: column、gap 2px。
- link: `display:flex; align-items:center; width:100%; border-radius: --radius-element`(neutral 10px)、`color: --color-text-secondary`(light `#474747` / dark `#9e9e9e`)、`font-weight: 400`、`font-size: 14px / line-height 20px`(`--text-body-*`)、下線なし、text-align start、cursor pointer。
  - density default: `padding-block: 8px; padding-inline-end: 8px`。compact: `padding-block: 4px; padding-inline-end: 8px`。
  - インデント `padding-inline-start`: level 1 と 2 → `--spacing-3` 12px、level 3 → `--spacing-7` 28px、level 4 → `--spacing-11` 44px、level 5–6 → `--spacing-12` 48px(`max(1, min(4, level-1 || 1))`: Outline.tsx:252-266)。
  - 1 項目の高さ: default 8+20+8 = **36px**、compact **28px**。
- 状態:
  - hover(hover 端末): 文字 `--color-text-primary`(`#000000` / `#ffffff`)+ 背景 `--color-overlay-hover`(`#0000000D` / `#FFFFFF0D`)(0.9)。
  - pressed: 背景 `--color-overlay-pressed`(`#0000001A` / `#FFFFFF1A`)。
  - active(aria-current=location): 文字 primary + `font-weight: 600`。背景は付かない(hover 時のみ)。indicator がこのリンクの左に。
  - focus-visible: 標準リング(2px `--color-accent`、offset 3px、角丸 10px に沿う)。
- label: `overflow:hidden; text-overflow:ellipsis; white-space:nowrap`。Kivact `style().single_line()`(sheet.rs:528-530)。
- RTL: track は右側、インデントは右から。

```
│ Introduction            ← level 1 (indent 12)
┃ Getting started         ← active: indicator ┃ (2px, primary), semibold primary
│   Install               ← level 3 (indent 28)
│     Options             ← level 4 (indent 44)
```

### 13.5 Motion
- indicator: `transition: top, height --duration-fast-min`(neutral **95ms**)`--ease-standard` = `cubic-bezier(0.24, 1, 0.4, 1)`(Outline.tsx:175-177)。active が変わると bar が新しい項目へ滑り、高さも合わせて伸縮。
- link: `transition: background-color, color 125ms(--duration-fast) cubic-bezier(0.24, 1, 0.4, 1)`(Outline.tsx:211-213)。
- クリック時のスクロール: `scrollIntoView({behavior:'smooth', block:'start'})`(offset 0)または `scrollBy({top: delta, behavior:'smooth'})`(offset あり)。スムーズスクロールの曲線はブラウザ既定(UNRESOLVED: 値なし)。settle 監視の上限 1200ms(useScrollSpy.ts:46)。reduced motion では即時ジャンプ。
- Kivact: indicator は `translate_y` を paint transition 95ms で動かし、高さは `animated(height, Transition::tween(95, ..))` を reactive style で読む(高さは layout 値なので)。代替: indicator を active 項目の子として描き、項目切替時に `enter` で opacity フェードのみ(滑りが無くなる)— 推奨は前者。

### 13.6 Behaviour & a11y
- `nav` landmark(名前 "Table of contents")、`ul role=list` / `li role=listitem`、各リンクは `a[href=#id]`、active に `aria-current="location"`。
- 単一タブ停止(roving tabindex): 初期 / 非フォーカス時は active リンクが tabindex 0。↑↓ で移動、Home/End で端、Enter(リンクの既定)/ Space で navigate(Outline.tsx:374-469)。修飾キー付きクリックは既定のリンク動作に任せる。
- navigate: スクロール → URL hash を pushState → 見出し要素にフォーカス(tabindex 無ければ −1 を付けて preventScroll focus)(Outline.tsx:275-288, 420-425)。
- scroll-spy(useScrollSpy.ts:140-185): 見出しの top が「スクロール容器 top + offset + 見出しの scroll-margin-top」+1px 以下になったら passed。active = 最後に passed した見出し。どれも passed していなければ先頭、容器が最下端(残り 2px 以内)なら最後。スクロールとリサイズを rAF でまとめて再計算。ナビゲーション中のスムーズスクロール中は途中の項目に active を飛ばさない(navigationRef)。
- 既定テキスト: `soft.outline.label` = "Table of contents"(`@astryx.outline.label`)。

### 13.7 Kivact 実装方針
- `soft::outline(items)`: `row((track, list, indicator))`。list は `kivact::interaction::use_list` を vertical・roving・activation=Enter/Space で(`ListActivation`)。項目は `Role::Link` 相当 + 選択状態(current)。
- scroll-spy: `scroll_container.scroll_offset()` と各 `item.target.rect()` を読む `memo` で active を算出(上記ルール、offset と 1px/2px 許容)。容器の高さと内容高さは container の `rect()` と内容の rect。
- navigate: `scroll_container.scroll_to(Point{ y: target_top - offset }, ScrollBehavior::Smooth(kivact::motion::Easing::EaseOut))`(crates/kivact-core/src/view/node_ref.rs:229)→ `target.focus()`。hash 更新は native では不要(破棄)。`on_navigate_end` はスクロール終了検知(offset が目標に達する or ユーザー入力で中断 or 1200ms)で 1 回呼ぶ。
- indicator: active 項目の rect を list 基準に変換し、`translate_y` + 高さを animated で追従。
- 最も近い basic: なし(roving list の作法は crates/kivact-ui-basic/src/sidebar.rs と `kivact::ui::list`)。

### 13.8 受け入れ確認
- [ ] navigation "Table of contents" の中に list / listitem / link、active に current。
- [ ] Tab で 1 回だけ止まり(active 項目)、↓↑ で移動、Space でその見出しへスムーズスクロールしフォーカスが見出しへ。
- [ ] 内容をスクロールすると active が見出し通過に合わせて変わり、indicator(2px, `#000000` / `#ffffff`)が 95ms で滑る。最下端で最後の項目が active。
- [ ] level 1/2 は 12px、3 は 28px、4 は 44px、5–6 は 48px インデント。default 36px / compact 28px 行。
- [ ] hover で文字 primary + 背景 `#0000000D`(dark `#FFFFFF0D`)、角丸 10px。
- [ ] 長いラベルは 1 行で省略記号。


## 14. Pagination

docs: https://astryx.atmeta.com/components/Pagination 。source: packages/core/src/Pagination/Pagination.tsx(+ Pagination.doc.mjs, Pagination.test.tsx)。

**判定: NEW kit component。** 中身は ghost button(0.10、標準 `kivact::ui::button` の restyle)、`kivact::ui::number_input`(input variant)、`kivact::ui::select`(page size)の組み合わせ。最も近い basic: crates/kivact-ui-basic/src/pagination.rs(`pagination`, `pagination_link`, `pagination_previous/next`, `pagination_ellipsis`, `PaginationPages`)。

### 14.1 概要
- 表・リスト・カードグリッドの **下** に置き、ページを前後に移動する。
- variant: `pages`(表で特定ページへ飛ぶ)、`count` + page size selector(表示件数を選ぶ大きなリスト)、`compact`(狭い場所)、`dots`(カルーセルや手順。総数が小さいとき、10 ページ超では使わない)、`input`(ページ番号を直接入力)、`none`(前後のみ)。
- 総数が分かるなら `totalItems` を渡す。1 ページに収まるときは出さない。上に置かない(Pagination.doc.mjs bestPractices)。

### 14.2 Anatomy
```
nav (aria-label="Pagination", flex, space-between, gap 16px)         .astryx-pagination
├─ [page size selector] (Selector, 80px, label hidden "Items per page")   ← pageSizeOptions があるとき
└─ controls (flex, gap 4px)
    ├─ [first «] ghost icon button          ← input(既定)/ compact(hasFirstLast=true) かつ総数既知
    ├─ previous ‹ ghost icon button
    ├─ indicator(variant 別)
    │   pages:   [1] … [4] [5*] [6] … [20]   ← ghost button(active は neutral 背景 + medium)、… は span
    │   count:   "11–20 of 200"
    │   compact: "Page 2 of 20"
    │   dots:    group "Page indicators" ● ○ ○ (roving)
    │   input:   "Page" [ n ] "/ 20"
    ├─ next › ghost icon button
    └─ [last »] ghost icon button
```

### 14.3 API(Pagination.tsx:69-173)

| prop | type | default | 意味 | Kivact(`soft::pagination(page)`) |
|---|---|---|---|---|
| page | number(1 始まり) | 必須 | 現在ページ | `page: impl Into<Binding<u32>>`(caller-owned) |
| onChange | (page) => void | 必須 | 変更 | binding への書き込み + `.on_change(impl Fn(u32))`(任意) |
| changeAction | (page) => Promise | なし | 非同期処理(楽観的更新) | **破棄**(アプリが binding 変更を受けて処理。kivact に React transition は無い) |
| totalItems | number | なし | 総件数(totalPages より優先) | `.total_items(impl IntoValue<u64>)` |
| totalPages | number | なし | 総ページ数 | `.total_pages(impl IntoValue<u32>)` |
| hasMore | boolean | false | cursor 方式で次があるか | `.has_more(impl IntoValue<bool>)` |
| pageSize | number | 10 | 1 ページ件数(正の整数化) | `.page_size(Binding<u32>)` |
| pageSizeOptions | number[] | なし | 件数セレクタの候補 | `.page_size_options(Vec<u32>)` |
| onPageSizeChange | (n) => void | なし | 件数変更(自動で page 1 へ) | binding 書き込み |
| variant | `'pages'\|'count'\|'compact'\|'dots'\|'input'\|'none'` | `'pages'` | 中央の表示 | `.variant(PaginationVariant::..)` |
| pageLabel | string | "Page" | input variant の前置ラベル | `.page_label(impl IntoText)` |
| hasFirstLast | boolean | input: true / compact: false | « » ボタン | `.first_last(bool)` |
| step | number | 1 | 前後ボタンの移動量(範囲内にクランプ) | `.step(u32)` |
| siblingCount | number | 1 | pages で現在の両隣に出す数 | `.siblings(u32)` |
| size | `'sm' \| 'md'` | `'md'` | ボタン等のサイズ | `.size(ControlSize::Small \| Medium)` |
| isDisabled | boolean | false | 全体無効 | `.disabled(..)` |
| label | string | "Pagination" | nav 名 | `.name(..)` |
| xstyle / className / style / ref / data-testid | — | — | — | 破棄 / NodeBuilder |

総数 ≤ 0(totalItems ≤ 0 または totalPages ≤ 0)のときは何も描かない(Pagination.tsx:474-479)。

**ページ窓アルゴリズム `generatePageRange(current, total, siblings)`**(Pagination.tsx:304-353、そのまま移植):
1. `slots = 5 + 2*siblings`。`total <= slots` なら `1..=total` 全部。
2. `left = max(current - siblings, 1)`、`right = min(current + siblings, total)`。
3. `showLeft = left > 2`、`showRight = right < total - 1`。
4. `!showLeft && showRight`: `1..=(3 + 2*siblings)`, `…`, `total`。
5. `showLeft && !showRight`: `1`, `…`, `(total - (3+2*siblings) + 1)..=total`。
6. 両方: `1`, `…`, `left..=right`, `…`, `total`。
例(siblings 1, total 20): current 1 → `1 2 3 4 5 … 20`、current 10 → `1 … 9 10 11 … 20`、current 20 → `1 … 16 17 18 19 20`。常に 7 スロット。

### 14.4 Visual spec(Pagination.tsx:180-287)
- nav: `display:flex; align-items:center; justify-content:space-between; gap: --spacing-4` 16px(page size selector が左、controls が右端)。
- controls: `display:flex; align-items:center; gap: --spacing-1` 4px。
- 前後・最初/最後ボタン: ghost・icon only(0.10)。md = 32×32px、sm = 28×28px、radius 10px、アイコン `chevronLeft` / `chevronRight` / `chevronsLeft` / `chevronsRight`(Lucide `chevron-left` 等)、md で 20px、sm で 16px、色 = `--color-text-primary`(light `#000000` / dark `#ffffff`)。RTL ではアイコンを左右反転(`rtlStyles.mirror`)。端で disabled(opacity 0.5)。tooltip = ボタン名(disabled 時は無し)。
- compact variant の first/last: first に `margin-inline-end: -4px`、last に `margin-inline-start: -4px`(前後ボタンと 0px 間隔に詰める)。
- pages の番号ボタン: ghost・md 高さ 32px(sm 28px)、padding-inline 12px、文字 `--text-label` 14px/500/20px(button 既定)、文字色 primary。**active**: 背景 `--color-neutral`(light `#0000000F` / dark `#FFFFFF1A`)+ `font-weight: --font-weight-medium` 500(button 既定と同じ 500 なので太さは変わらない)、`aria-current=page`。hover は overlay(`#0000000D` / `#FFFFFF0D`)、押下 scale 0.98。
- ellipsis "…": `inline-flex; center; min-width: 32px(sm 28px); height 32px(sm 28px); color: --color-text-secondary`(light `#474747` / dark `#9e9e9e`); `font-size: --text-label-size` 14px(sm は 12px); `aria-hidden`; user-select none。
- count / compact テキスト: `Text type=body size=sm color=secondary` → 12px、line-height 1.4286(≈17px)、400、`#474747` / `#9e9e9e`、nowrap。
- dots: container `flex; gap 4px`。dot: md **8×8px**(`--spacing-2`)、sm **6×6px**(`--spacing-1-5`)、円、border なし、背景 `--color-neutral`(`#0000000F` / `#FFFFFF1A`)、active `--color-accent`(light `#1b1b1b` / dark `#f1f1f1`)、disabled opacity 0.5、cursor pointer、focus-visible で標準リング。
- input: group `inline-flex; gap 4px; nowrap`。label "Page" と total "/ 20" は `--color-text-secondary`、14px(sm 12px)、user-select none、label は `aria-hidden`(入力には隠しラベル "Go to page")。NumberInput は幅 = `--size-element-md` 32px(sm 28px)、整数のみ、min 1、max total(総数不明時 disabled)。
- page size selector: `display:flex; align-items:center; gap 8px`、Selector 幅 80px、size = sm/md。
- disabled 全体: 各ボタンが disabled(opacity 0.5)。

```
[10 ▾]                                    ‹  1  …  9 [10] 11  …  20  ›
 page size (80px)                          ↑ 32×32 ghost    ↑active: neutral bg
```

### 14.5 Motion
- ボタン: 0.10 の transition(125ms `cubic-bezier(0.24, 1, 0.4, 1)`、押下 scale 0.98)。
- dot: `background-color 125ms --ease-standard`(Pagination.tsx:237-239)。active が移るとき前の dot が neutral に、新しい dot が accent にクロスフェード。
- ページ変更そのものにアニメーションなし。

### 14.6 Behaviour & a11y
- `nav` landmark(名前 "Pagination")。
- ページ変更時に live announce: 総数既知なら "Page {current} of {total}"、不明なら "Page {current}"(Pagination.tsx:484-503)。
- 前後: `step` だけ移動しクランプ。first/last は 1 / total。総数不明時は hasMore で next を有効化、first/last は非表示。
- dots: `role=group` 名前 "Page indicators"、roving tabindex(active が 0)、←→ で移動しつつ **フォーカス移動でページも変わる**(handleDotsFocus: Pagination.tsx:510-525)、wrap あり。各 dot の名前 "Go to page {n}"、active に aria-current=page。
- pages: 各ボタン名 "Go to page {n}"、active に aria-current=page。
- input: Enter/blur で確定(NumberInput の commit)、同じ値なら無視。範囲外はクランプ。
- page size 変更で page 1 に戻る。
- 既定テキスト(提案キー):
  - `soft.pagination.label` = "Pagination"
  - `soft.pagination.previous` = "Go to previous page"、`soft.pagination.next` = "Go to next page"
  - `soft.pagination.previous_by` = "Go back {step} pages"(step=1 は使わない; plural 簡略)、`soft.pagination.next_by` = "Go forward {step} pages"
  - `soft.pagination.first` = "Go to first page"、`soft.pagination.last` = "Go to last page"
  - `soft.pagination.go_to_page` = "Go to page {page}"、`soft.pagination.go_to_page_input` = "Go to page"
  - `soft.pagination.page_label` = "Page"、`soft.pagination.of_total_pages` = "/ {total}"
  - `soft.pagination.page_indicators` = "Page indicators"、`soft.pagination.items_per_page` = "Items per page"
  - `soft.pagination.count` = "{from}–{to} of {total}"、`soft.pagination.page_of_total` = "Page {current} of {total}"、`soft.pagination.page_announce` = "Page {current}"

### 14.7 Kivact 実装方針
- `soft::pagination(page_binding)`: `row((size_selector?, row(controls)))`。nav は `Role::Navigation` + name。
- `generate_page_range` を純関数として移植しユニットテスト(上の 3 例 + total <= 7)。
- live announce: kivact の announce API(UNRESOLVED: `kivact::view` の live region / announce 関数名。kivact-ui-basic の pagination.rs の実装を確認して合わせる)。
- dots の roving: `kivact::interaction::use_list` を horizontal・wrap・「フォーカス = 選択」ポリシーで(`ListPolicy` の selection follows focus)。
- number_input / select は標準 `kivact::ui` を soft recipe で。
- basic の crates/kivact-ui-basic/src/pagination.rs は shadcn の「リンク列」型なので API は soft で作り直す(page window と variant を持つ)。

### 14.8 受け入れ確認
- [ ] navigation "Pagination"。total 20, page 10 で `‹ 1 … 9 10 11 … 20 ›`、10 が current(背景 `#0000000F` / dark `#FFFFFF1A`)。
- [ ] page 1 で ‹ が disabled(半透明)、page 20 で › が disabled。
- [ ] 番号押下でページが変わり "Page 9 of 20" が読み上げられる。
- [ ] count variant: "11–20 of 200"(12px, secondary)。compact: "Page 2 of 20"、first_last(true) で « » が前後ボタンに密着。
- [ ] dots: 8px の点、active が `#1b1b1b`(dark `#f1f1f1`)、← → でフォーカスとページが同時に移動、端で wrap。
- [ ] input: "Page [10] / 20"、99 を入力して Enter → 20 にクランプ。
- [ ] size sm でボタン 28px、文字 12px、dot 6px。


## 15. SideNav (+ SideNavHeading, SideNavItem, SideNavSection, SideNavCollapseButton)

docs: https://astryx.atmeta.com/components/SideNav (sub-components は同ページ)。source: packages/core/src/SideNav (SideNav.tsx, SideNavHeading.tsx, SideNavItem.tsx, SideNavSection.tsx, SideNavCollapseButton.tsx, SideNavCollapseContext.ts, SideNavFocusTargetsContext.ts, SideNavRenderContext.ts, railMotion.ts, docs, SideNavRailMotion.test.tsx, SideNav.slotPresence.test.tsx)。行の共通見た目: packages/core/src/NavItem/navItemStyles.stylex.ts。

**判定: NEW kit component(大)。** 縦のナビゲーション。折り畳み(rail: 260px ↔ 48px)、リサイズ(10. Resizable)、入れ子項目の開閉(Disclosure パターン)、rail 時の flyout、heading の menu popover、AppShell 内での描画モード切替(inline / drawer / drawer-content / topbar)。最も近い basic: crates/kivact-ui-basic/src/sidebar.rs + crates/kivact-ui-basic/src/sidebar/(`sidebar`, `sidebar_menu_button`, `sidebar_group`, `SidebarCollapsible`)。

### 15.1 概要
- 5 個以上の行き先、または階層グループが要るアプリの主ナビゲーション。
- section で関連項目をまとめる。outline と filled のアイコン対(`icon` / `selectedIcon`)で選択状態を形でも区別。現在ページは `isSelected`(aria-current=page)。
- 折り畳み中、子を持つ項目は flyout で子を出す(hover 端末では少し待って開き、離れると閉じる。クリックで開いたものは明示的に閉じるまで開いたまま。タッチはタップ)。flyout にしか無い操作を置かない。
- TopNav がアプリ名を出しているときに SideNavHeading で重複させない。コンテンツのフィルタに使わない(SideNav.doc.mjs bestPractices)。

### 15.2 Anatomy
```
nav role=navigation aria-label="Side navigation" (column, h 100%, w 260 | 48, overflow hidden, bg inherit)   .astryx-side-nav
├─ sticky top (column, gap 8, padding 8 8 8 8, sticky top 0, z 1, bg inherit)       ← header か topContent があるとき
│   ├─ header: SideNavHeading
│   └─ topContent(例: 作成ボタン)
├─ scrollable (flex 1, overflow-y auto, padding-inline 8; top pad 4|8, bottom pad 4|8)
│   └─ children: SideNavSection* / SideNavItem*
│        SideNavSection (div role=group aria-labelledby=title, padding-block 4)
│        ├─ header (title / subtitle / endContent)   ← rail 時・isHeaderHidden は VisuallyHidden
│        └─ items (column, gap 2)
│             SideNavItem (div row wrapper)
│             ├─ row: a|button (navItem: h 32, px 8, gap 8, radius 10) [icon][label][endContent][chevron]
│             │    or row wrapper div: primary a|button + expand toggle button + actions
│             └─ children group (div role=group, grid 1fr↔0fr アニメーション) → SideNavItem (depth+1)
└─ sticky bottom (column, gap 8, padding 4 8 8, margin-top auto, sticky bottom 0)   ← footer / footerIcons / collapse button
    ├─ footer
    └─ footer row (flex, gap 4)  [SideNavCollapseButton][footerIcons…]   (子のサイズは sm に cascade)
[resizable] wrapper (relative, flex, h 100%) + ResizeHandle(overlay, pill end, hover 時のみ表示, "Resize sidebar")
```

### 15.3 API

SideNav(SideNav.tsx:325-422, SideNav.doc.mjs):

| prop | type | default | 意味 | Kivact(`soft::side_nav(children)`) |
|---|---|---|---|---|
| header | ReactNode | なし | sticky 上部(通常 SideNavHeading) | `.header(|| soft::side_nav_heading(..))` |
| topContent | ReactNode | なし | header 下(作成ボタン等) | `.top_content(|| ..)` |
| children | ReactNode | 必須 | section / item(スクロール) | `children: impl IntoView` |
| footer | ReactNode | なし | アイコン列の上 | `.footer(|| ..)` |
| footerIcons | ReactNode | なし | 下部アイコン列(子に size sm を cascade) | `.footer_icons(|| ..)` |
| collapsible | `boolean \| {defaultIsCollapsed?, isCollapsed?, onCollapsedChange?, hasButton? (true), buttonLabel?}` | false | rail 折り畳み | `.collapsible(SideNavCollapse::Uncontrolled{ default: bool } \| Controlled(Binding<bool>))` + `.collapse_button(bool)` |
| resizable | `boolean \| {defaultWidth? 260, minWidth? 180, maxWidth? 480, autoSaveId?, onWidthChange?, defaultIsCollapsed?, isCollapsed?, onCollapseChange?}` | false | inline-end のリサイズハンドル。160px 未満へのドラッグで折り畳み(COLLAPSE_THRESHOLD: SideNav.tsx:92) | `.resizable(SideNavResize{ default_width: 260.0, min: 180.0, max: 480.0, .. })` |
| handleRef | Ref | — | 非推奨(外部 collapse button 用) | **破棄**(同じ Binding を外部 button に渡す) |
| xstyle / className / style / ref / data-testid | — | — | — | 破棄 / NodeBuilder |

collapsible と resizable の両方が collapse state を持つと resizable が勝つ(dev 警告: SideNav.tsx:113-171)。

SideNavItem(SideNavItem.tsx:390-477):

| prop | type | default | 意味 | Kivact(`soft::side_nav_item(label)`) |
|---|---|---|---|---|
| label | string | 必須 | ラベル(rail 時は名前) | 第 1 引数 |
| icon / selectedIcon | ReactNode \| IconType | なし | 通常 / 選択時アイコン(sm 16px) | `.icon(icon("house"))` / `.selected_icon(..)` |
| isSelected | boolean | false | 現在ページ(aria-current=page) | `.selected(impl IntoValue<bool>)` |
| isDisabled | boolean | false | 無効(aria-disabled、フォーカスは可) | `.disabled(..)` |
| href / onClick | string / fn | なし | 遷移 | `.on_press(impl Fn())`(href は破棄) |
| endContent | ReactNode | なし | 受動的な右端(badge, count) | `.end_content(|| ..)` |
| actions | ReactNode | なし | 行内の二次操作(icon button, menu)。size sm を cascade。rail 時は非表示 | `.actions(|| ..)` |
| children | ReactNode | なし | 子項目(入れ子) | `.children(|| (item, item))` |
| collapsible | `boolean \| {defaultIsCollapsed?, isCollapsed?, onCollapsedChange?}` | 子があれば開閉可(初期展開)。`false` で常時展開 | `.collapsible(ItemCollapse::..)` |
| size | `'sm' \| 'md' \| 'lg'` | `'md'` | 行サイズ | `.size(ControlSize::Small \| Medium \| Large)` |
| as | LinkComponentType | — | — | 破棄 |

SideNavHeading(SideNavHeading.tsx:296-346):

| prop | type | Kivact(`soft::side_nav_heading(heading)`) |
|---|---|---|
| heading | string(必須) | 第 1 引数 |
| icon | ReactNode(通常 NavIcon) | `.icon(|| soft::nav_icon(icon("box")))` |
| headingHref / superheadingHref / subheadingHref | string | `.on_heading_press(..)` 等(href は破棄) |
| superheading / subheading | string | `.superheading(..)` / `.subheading(..)` |
| headerEndContent | ReactNode | `.end_content(|| ..)`(rail 時は非表示) |
| menu | ReactNode(通常 NavHeadingMenu) | `.menu(|| soft::nav_heading_menu(..))` |

SideNavSection(SideNavSection.tsx:95-119): `title`(必須), `subtitle`, `endContent`, `isHeaderHidden`(false), `children` → `soft::side_nav_section(title, children).subtitle(..).end_content(..).header_hidden(bool)`。

SideNavCollapseButton(SideNavCollapseButton.tsx:77-110): `collapsible?: {isCollapsed, onCollapsedChange}`(SideNav 外に置くときだけ)、`label?`(指定時はテキスト付きボタン)、`size?`(footer 内は sm、外では md)、`children?`。mobile では描画しない。→ `soft::side_nav_collapse_button()`.`binding(Binding<bool>)`。

### 15.4 Visual spec

**nav 本体**(SideNav.tsx:177-312)
- 幅 **260px**(rail 時 `--spacing-12` = **48px**)、高さ 100%、`overflow:hidden`、背景 `inherit`(AppShell の nav 領域色: elevated/wash では `--color-background-body` light `#f1f1f1` / dark `#1b1b1b`)。
- sticky top: `padding: 8px`(上下左右 `--spacing-2`)、`gap: 8px`、背景 inherit。rail 時 `align-items:center`。
- scrollable: `padding-inline: 8px`、上 padding = top ありで 4px / 無しで 8px、下 padding = bottom ありで 4px / 無しで 8px。rail 時 `display:flex; flex-direction:column; align-items:center`。
- sticky bottom: `padding: 4px 8px 8px`、`gap: 8px`、`margin-top:auto`。footer row: `flex; align-items:center; gap 4px`(rail 時 `flex-direction: column-reverse` = collapse button が一番下)。rail 時 bottom の上 padding 0、中央揃え。
- topbar 描画(AppShell の mobile top bar 内): `row; space-between; height 48px; width 100%`、header + footerIcons(gap 4、margin-inline-start auto)。
- resizable: wrapper `relative; flex; h 100%; overflow clip`。handle は overlay(16px 帯)・pill end・`isAlwaysVisible=false`(hover / focus で pill 表示)・focus ring は内側(`outline-offset: -2px`)。

**SideNavItem 行**(navItemStyles.stylex.ts:52-156, SideNavItem.tsx:93-292)
- 行: `display:flex; align-items:center; gap: 8px; width:100%; height: --size-element-md 32px; padding-inline: 8px; border-radius: --radius-element 10px; 背景 transparent; color: --color-text-primary`(light `#000000` / dark `#ffffff`); `font-size: --text-label-size 14px; font-weight: 400; line-height 20px`; text-align start; 下線なし。
- size: sm → 高さ 28px・padding-inline 4px。md → 32px・8px。lg → 36px・8px。
- アイコン: size sm(16px)、色: 通常 `--color-icon-secondary`(light `#6a6a6a` / dark `#9e9e9e`)、選択時 inherit(= text primary)、disabled は `--color-icon-disabled`(`#919191` / `#525252`)。
- label: `flex:1; min-width:0; 1 行 ellipsis`。endContent: shrink 0, flex center。
- hover: 0.9 の overlay `--color-overlay-hover`(light `#0000000D` / dark `#FFFFFF0D`)、押下 `--color-overlay-pressed`(`#0000001A` / `#FFFFFF1A`)。
- **selected**: 背景 `--color-neutral`(light `#0000000F` / dark `#FFFFFF1A`)、`font-weight: 500`(medium)。hover / active / press 中も neutral のまま(overlay を重ねない)。forced-colors では Highlight。
- disabled: 文字 `--color-text-disabled`(light `#919191` / dark `#525252`)、pointer-events none。
- focus-visible: 標準リング(2px `--color-accent` light `#1b1b1b` / dark `#f1f1f1`、offset 3px、角丸 10px)。actions 付きの行では行全体(wrapper)にリング(`focusWithinFirstChild`)。
- 入れ子インデント(depth ≥ 1): 行の `padding-inline-start` = sm: `4px + depth*24px`、md/lg: `8px + depth*24px`(SideNavItem.tsx:282-292)。行自体はリスト全幅(インデントは内容だけ)。
- 開閉 chevron: `chevronDown`(Lucide `chevron-down`)、Icon size lg 指定だが箱は `--spacing-6` 24×24px、色 inherit。**展開中は `rotate(180deg)`(上向き)**、閉じると 0deg。
- 独立トグル(子あり + href/onClick あり + 非 rail): 行内に別ボタン `28×28px`(`--size-element-sm`)、radius 10px、透明、overlay hover、中に同じ chevron。primary 部分は `display:flex; gap 8px; flex 1; align-self stretch`。
- actions: `flex-shrink 0; flex; gap 4px`、子は size sm。

**rail(折り畳み)時の項目**(SideNavItem.tsx:659-781)
- icon の無い項目は描画しない。行は `width: 32px`(sm 28 / lg 36)・`padding-inline 0`・中央揃え(アイコンだけ)。
- leaf: 右(inline-end)に tooltip でラベル(placement end、focus でも表示)。
- 子あり: ボタン → hover(showDelay 150ms 既定)/クリックで右側に flyout popover: `border-radius 12px; padding 4px; min-width 180px; margin-inline 4px`、背景・影は 0.8 Popover 既定面。先頭にヘッダ `padding 4px 8px; 12px / 600 / 20px; --color-text-secondary`(ラベル名)、その下に子項目(depth 0 として通常行で描画)。項目クリックで閉じる。

**SideNavSection**(SideNavSection.tsx:39-89)
- root: `column; padding-block: 4px`。header: `flex; align-items:center; gap 8px; padding: 4px 8px; cursor default; user-select none`。
- title: `--text-supporting-size` 12px / `font-weight 600` / line-height 20px / `--color-text-secondary`(`#474747` / `#9e9e9e`) / 1 行 ellipsis。subtitle: 12px / 400 / 20px / secondary / ellipsis。
- items: `column; gap: 2px`(`--spacing-0-5`)。
- rail 時と isHeaderHidden: header を VisuallyHidden(読み上げのみ)。

**SideNavHeading**(SideNavHeading.tsx:76-290)
- root: `flex; align-items:center; gap 8px; min-height: --spacing-8 32px; padding-inline-start 8px(NavIcon を含むと 0); padding-inline-end 8px`、色 inherit。
- interactive(heading 全体がリンク/menu trigger): radius 10px、hover で `--color-overlay-hover`、font-weight 400。rail 時は hover 背景なし・中央・padding 0。
- テキスト列: superheading 12px / 20px / secondary / ellipsis、**heading `--text-large-size` 17px / 600 / line-height 1.4118(24px) / `--color-text-primary`** / ellipsis、subheading 12px / 20px / secondary。
- menu chevron: `chevronDown` size sm 16px、色 `--color-icon-secondary`(`#6a6a6a` / `#9e9e9e`)、箱 min 28×28px(`--spacing-7`)。
- **menu popover**: placement below / align start、**heading に重なって開く**(`min-width: anchor幅 + 16px; margin-top: -(anchor 高さ) - 8px; margin-inline-start: -8px`: SideNavHeading.tsx:285-289)。中身 `padding 4px`、先頭に heading の複製ボタン(`min-height 32px; padding-inline 8px(icon あり 0) 8px; margin: 4px 4px 8px`、chevron は `rotate(180deg)` = 上向き、押すと閉じる)、その下に `role=menu` の NavHeadingMenu(19)。面は 0.8 Popover 既定。rail 時は icon ボタンの下に普通に開く(`min-width: anchor 幅; margin-top 4px`)。

**SideNavCollapseButton**: ghost icon-only(footer 内 sm = 28×28px)、アイコン `chevronLeft`(Lucide `chevron-left`)16px、RTL でミラー、**折り畳み中は rotate(180deg)**(右向き)。

```
expanded (260)                         rail (48)
┌────────────────────────┐             ┌──────┐
│ (●) Acme ▾             │ heading     │ (●)  │
│ ──────────             │             │      │
│ WORKSPACE (12/600)     │ section     │      │
│ [■] Home         ← sel │ 32px row    │ [■]  │ ← 32×32, tooltip "Home"
│ [□] Projects   3   ▴   │             │ [□]  │ ← hover で flyout
│      Alpha             │ depth1 +24  │      │
│                        │             │      │
│ [‹] [⚙]                │ footer row  │ [⚙]  │
└────────────────────────┘             │ [›]  │
                                       └──────┘
```

### 15.5 Motion
- **rail 折り畳み / 展開**(railMotion.ts:112-209 + animated layout): nav 幅を 0.11 の settle spring(stiffness 157.91, damping 25.13, mass 1、≈735ms)で 260 ↔ 48 に動かす(inline-end 辺を hold して play)。AppShell 内では main 列が同じ spring で一緒に滑る(1.5)。進捗 p(0→1)に対して:
  - 折り畳み時: ラベル(テキスト)opacity = `1 - clamp(p / 0.4)`(最初の 40% で消える)。旧(展開)の複製 opacity = `1 - clamp((p - 0.7) / 0.3)`。新しい rail(アイコンだけの描画)opacity = `clamp((p - 0.7) / 0.3)`(最後 30% で現れる)。
  - 展開時: 旧(rail)の複製 opacity = `1 - clamp(p / 0.6)`(最初 60% で消える)。新しい展開描画のテキスト要素 opacity = `clamp((p - 0.4) / 0.6)`(40% 以降で現れる)。
  - 途中反転時は現在の幅・labels・collapsed 値から続行。
  - reduced motion: fine pointer は即時、coarse は ≤200ms の opacity クロスフェード。
- **子グループ開閉**(SideNavItem.tsx:143-163): `grid-template-rows: 1fr ↔ 0fr` を `--duration-medium`(neutral **300ms**)`--ease-standard` `cubic-bezier(0.24, 1, 0.4, 1)` で。開くとき overflow は 300ms 後に visible へ(delay 300ms)、閉じるとき即 hidden。reduced motion 0s。
- **chevron 回転**: `transform 125ms(--duration-fast) cubic-bezier(0.24, 1, 0.4, 1)`、0 ↔ 180deg(項目 chevron、collapse button chevron とも)。reduced motion 0s。
- 行の hover / selected 背景: transition 指定なし(即時)。
- flyout / heading menu: **出現・消失ともアニメーション無し(即時)**。どちらも `usePopover` フックを直接使い、`layerAnimations` を渡していない(`layerAnimations` を使うのは Popover コンポーネント・DropdownMenu・Breadcrumb menu などのみ: `grep layerAnimations` の結果)。
- resize handle: 10.5 参照。
- Kivact: 幅は `animated(width_target, Transition::spring(Spring{stiffness:157.91, damping:25.13, mass:1.0}))` を reactive style で読む。ラベル/アイコン層の opacity は同じ animated の進捗 p = `(260 - w) / (260 - 48)`(折り畳み時)から上式で計算して `opacity(..)` に入れる(複製は作らず、「展開レイアウト」と「rail レイアウト」を同時に重ね描きして opacity を振る)。子グループの高さは `animated(height, Transition::tween(300, CubicBezier(0.24,1,0.4,1)))` + `overflow(Hidden)`。chevron は `rotate` を paint transition 125ms。

### 15.6 Behaviour & a11y
- `nav` landmark 名 "Side navigation"(SideNav.tsx:846-860)。section は `role=group` + `aria-labelledby`(title)。
- 項目: link または button。selected に `aria-current=page`、disabled に `aria-disabled`(フォーカス可、クリック無効)。子ありは WAI-ARIA Disclosure: トグル(行自体 or 独立トグル)に `aria-expanded` と `aria-controls`、子グループは `role=group` で閉じている間 `aria-hidden` + `inert`。
- 子あり項目の押下: href/onClick が無ければ開閉、あれば primary は遷移し独立トグルが開閉(SideNavItem.tsx:613-639)。drawer 内(MobileNav)で押すと遷移後に drawer を閉じる(`closeMobileNav`)。
- rail 時: leaf はアイコンのみ・名前 = label(または aria-label)・tooltip(focus でも)。子ありは flyout(hover: showDelay 150ms / hideDelay 200ms / safe-triangle 8px、クリックで固定。Esc / 外側クリックで閉じる)。flyout は dialog 名 "{label} submenu"。
- 折り畳み時のフォーカス維持(SideNav.tsx:629-706): 展開中にフォーカスしていた行が rail でも残る(icon あり・depth 0)ならその行へ、残らないなら collapse toggle → 直前の残る行 → 直後の残る行の順でフォーカスを移す。
- heading menu: trigger は heading 全体(hover で showDelay 0 で開く、クリックでも)。menu は `role=menu`、↑↓・typeahead・Enter/Space・Esc(19 参照)。trigger 名 "{heading}, Open menu"、popover の dialog 名 "Navigation menu"。
- collapse button: 名前 "Collapse sidebar" / "Expand sidebar"。mobile では非表示。
- resize handle: 名前 "Resize sidebar"、aria-controls = nav。
- 既定テキスト(提案キー):
  - `soft.side_nav.label` = "Side navigation"
  - `soft.side_nav.resize` = "Resize sidebar"
  - `soft.side_nav.heading_open_menu` = "{heading}, Open menu"
  - `soft.side_nav.heading_dialog` = "Navigation menu"
  - `soft.side_nav.collapse` = "Collapse sidebar"、`soft.side_nav.expand` = "Expand sidebar"
  - `soft.side_nav.item_expand` = "Expand {label}"、`soft.side_nav.item_collapse` = "Collapse {label}"、`soft.side_nav.item_submenu` = "{label} submenu"

### 15.7 Kivact 実装方針
- `soft::side_nav(children)`: `column((sticky_top?, scroll(column(children)), sticky_bottom?))`、`Role::Navigation` + name。sticky は `Position::Sticky`、または単に top/bottom を scroll の外に置く(nav 内で scroll するのは中段だけなので sticky 不要: **推奨**)。
- collapse state: `Binding<bool>` を `provide(SideNavCx{ collapsed, toggle, depth, render_mode }, ..)` で子へ。項目は `try_context` で rail かどうかを見る。
- 描画モード(inline / drawer / drawer-content / topbar)は AppShell(1)が `provide(SideNavRender::Drawer)` 等で渡す。
- 項目: `kivact::interaction::use_button`(link 相当)+ `use_disclosure`(子グループ)。flyout は `kivact::ui::dialog(DialogKind::Popover)` ではなく hover で開く popup が要るので `kivact::interaction::use_popup` + `use_delayed`(show 150 / hide 200)で kit 内実装(safe triangle は省略可: UNRESOLVED)。
- rail tooltip: 標準 `kivact::ui::tooltip(label).attach(item)`(placement end)。
- heading menu の「重ねて開く」配置: kivact の `overlay(anchor, placement, child)`(crates/kivact-core/src/view/overlays.rs:166)で below/start に置き、負の offset(-anchor 高さ - 8px, -8px)を付ける(UNRESOLVED: Overlay に offset 指定があるか。無ければ placement Over/Inside 相当で anchor の左上に合わせる)。
- resizable: 10 の `soft::use_resizable` を min 180 / max 480 / default 260 / collapse threshold 160 / collapsible で。
- rail motion: 上記 Motion の Kivact 案。AppShell 連携は `try_context::<SideNavShell>()` で before/after を呼ぶ。
- basic の sidebar.rs は shadcn の offcanvas/icon collapsible を持つので、構造(provider, trigger, menu button sizes)は参考になるが寸法・状態色は本仕様に置き換える。

### 15.8 受け入れ確認
- [ ] navigation "Side navigation"、幅 260px、項目行 32px・radius 10px・左右 8px 内側。
- [ ] 選択項目の背景 light `#0000000F` / dark `#FFFFFF1A`、weight 500、aria-current=page。hover は `#0000000D` / `#FFFFFF0D`。
- [ ] section title 12px/600 secondary、項目間 2px。
- [ ] 子あり項目を押すと 300ms で子グループが開き chevron が 125ms で上向きに回る。閉じると子は inert。
- [ ] collapse button(28px, ‹)で 48px rail に spring で縮み、ラベルが先に消えアイコンだけが残る。chevron は › に回る。rail の leaf に hover/focus で右 tooltip。
- [ ] rail の子あり項目に hover すると 150ms 後に右へ flyout(見出し + 子)。
- [ ] resizable で 180〜480px、160px 未満にドラッグで rail。handle は hover 時のみ pill。
- [ ] heading menu が heading に重なって開き、複製 heading の chevron が上向き、Esc で閉じて heading にフォーカス。
- [ ] RTL で全体が反転(chevron・インデント・handle が左)。


## 16. TopNav (+ TopNavHeading, TopNavItem, TopNavMenu, TopNavMegaMenu, TopNavMegaMenuItem, TopNavMegaMenuFeaturedCard)

docs: https://astryx.atmeta.com/components/TopNav 。source: packages/core/src/TopNav (TopNav.tsx, TopNavHeading.tsx, TopNavItem.tsx, TopNavMenu.tsx, TopNavMegaMenu.tsx, TopNavMegaMenuItem.tsx, TopNavMegaMenuFeaturedCard.tsx, TopNavContext.ts, TopNavRenderContext.ts, TopNavMobileContentContext.ts, docs, TopNavMegaMenu.reduced-motion.source-build.test.mjs)。

**判定: NEW kit component(大)。** 横のナビゲーションバー。hover で開くメニュー(TopNavMenu / TopNavMegaMenu)は `kivact::interaction::use_popup` + `use_delayed`(hover intent)で kit 実装。AppShell の below breakpoint では「mobile-bar」(heading + endContent + toggle)と「drawer」(MobileNav 内で縦並び + 開閉セクション)に描画を切り替える。最も近い basic: crates/kivact-ui-basic/src/navigation_menu.rs(`navigation_menu`, `NavItem`, `NavLink`; shadcn の viewport アニメーション付き)、crates/kivact-ui-basic/src/menubar.rs。

### 16.1 概要
- 画面上部の主ナビゲーション。heading にプロダクトのロゴと名前。主項目は 5 個以下。ページ内容のフィルタに使わない(→ Tabs)。メニューは 1 階層まで(TopNav.doc.mjs bestPractices)。
- `centerContent`(タブ・検索・主ナビ)を渡すと 3 列 grid で真の中央寄せ。

### 16.2 Anatomy
```
nav role=navigation aria-label="Top navigation" (padding 8px, width 100%, align center)   .astryx-top-nav
  centerContent 無し: display:flex
  ├─ left section (flex, gap 16px, flex 1 1 0, min-width 0)
  │   ├─ heading (flex, shrink 0): TopNavHeading
  │   └─ startContent (flex, gap 4px): TopNavItem / TopNavMenu / TopNavMegaMenu / Breadcrumbs …
  └─ endContent (flex, gap 4px, shrink 0, margin-inline-start auto)
  centerContent 有り: display:grid; grid-template-columns: 1fr auto 1fr
  ├─ left section | ├─ center (flex, center, gap 4px) | └─ right section (flex, end, gap 4px)

mobile-bar: nav (flex, padding 8) [heading (flex 0 1 auto)] … [endContent + MobileNavToggle](margin-inline-start auto, gap 4)
drawer:     MobileNav(header=heading) > items (column, gap 2px) [start + center] + Divider(margin-block 8) + mobileContent(SideNav 等)
```

### 16.3 API

TopNav(TopNav.tsx:130-169):

| prop | type | default | Kivact(`soft::top_nav()`) |
|---|---|---|---|
| heading | ReactNode | なし | `.heading(|| soft::top_nav_heading(..))` |
| startContent / children | ReactNode | なし | `.start(|| ..)`(children は alias。soft では start のみ) |
| centerContent | ReactNode | なし | `.center(|| ..)` |
| endContent | ReactNode | なし | `.end(|| ..)` |
| label | string | "Top navigation" | `.name(..)` |
| xstyle / className / style / ref | — | — | 破棄 / NodeBuilder |

TopNavHeading(TopNavHeading.tsx:238-294): `logo`(ReactNode, 通常 NavIcon か画像)、`logoLabel`(既定 heading)、`heading`、`headingHref`、`superheading`/`Href`、`subheading`/`Href`、`headerEndContent`、`menu`(NavHeadingMenu)。SideNavHeading とほぼ同じ(icon → logo)。→ `soft::top_nav_heading(heading).logo(|| ..).menu(|| ..)`。

TopNavItem(TopNavItem.tsx:93-151):

| prop | type | default | Kivact(`soft::top_nav_item(label)`) |
|---|---|---|---|
| label | string(必須) | — | 第 1 引数(icon only 時は名前のみ) |
| href / target / rel / download / referrerPolicy / as | — | — | 破棄 → `.on_press(..)` |
| isSelected | boolean | false | `.selected(..)`(aria-current=page) |
| isDisabled | boolean | false | `.disabled(..)`(tabindex −1、遷移しない) |
| isIconOnly | boolean | false | `.icon_only(true)`(icon 必須) |
| icon | ReactNode | なし | `.icon(icon(..))` |
| children | ReactNode | なし | `.content(|| ..)`(label の代わりに表示) |
| size | `'sm'\|'md'\|'lg'` | `'md'` | drawer 描画時の navItem サイズだけに効く(デスクトップは固定 padding) |

TopNavMenu(TopNavMenu.tsx:254-308): `label`(必須, trigger 表示)、`items: {title, description?, icon?, href?, onClick?}[]`(必須)、`delay` = 150、`hideDelay` = 200。→ `soft::top_nav_menu(label, items: Vec<TopNavMenuItem>)`.`delay_ms(..)`.`hide_delay_ms(..)`。

TopNavMegaMenu(TopNavMegaMenu.tsx:283-306): `label`(必須)、`items`(ReactNode, 通常 TopNavMegaMenuItem 群)、`featured`(ReactNode, 通常 FeaturedCard)、`delay` = 150、`hideDelay` = **250**、`onOpenChange`。→ `soft::top_nav_mega_menu(label).items(|| ..).featured(|| ..).on_open_change(..)`。

TopNavMegaMenuItem(TopNavMegaMenuItem.tsx:136-156): `title`(必須)、`description`、`icon`、`href` / `onClick`、`as`。→ `soft::top_nav_mega_menu_item(title)`。

TopNavMegaMenuFeaturedCard(TopNavMegaMenuFeaturedCard.tsx:83-107): `title`(必須)、`description`、`image`(URL)、`imageAlt`、`linkLabel` + `linkHref`(両方あるとき「label →」リンク)、`children`。→ `soft::top_nav_featured_card(title).image(Asset).link(label, on_press)`。

### 16.4 Visual spec

**TopNav バー**(TopNav.tsx:44-128): `padding: 8px`(全辺)、`align-items:center`、背景なし(AppShell header の背景を継承: elevated / wash は body `#f1f1f1` / `#1b1b1b`)、下線なし(AppShell variant section のときだけ header に 1px divider)。left section gap **16px**(heading と項目群の間)、項目群の gap **4px**、end の gap 4px。高さは中身で決まる(項目 32px + 上下 8px = 48px)。

**TopNavHeading**(TopNavHeading.tsx:60-232): SideNavHeading と同値。`flex; gap 8px; min-height 32px; padding-inline 8px(NavIcon を含むと start 0)`; 色 `--color-text-primary`。heading 文字 **17px / 600 / 24px**(`--text-large-*`)、super/subheading 12px / 20px / secondary(`#474747` / `#9e9e9e`)。interactive(リンク / menu)は radius 10px・hover `--color-overlay-hover`。menu chevron は `chevronDown` 16px・`--color-icon-secondary`(`#6a6a6a` / `#9e9e9e`)・箱 28×28px。menu popover は heading に **重ねて** 開く(min-width = anchor + 16px、margin-top = −anchor 高さ − 8px、margin-inline-start −8px)、中に heading 複製(chevron 上向き)+ role=menu(19)。面は 0.8 Popover 既定。

**TopNavItem**(TopNavItem.tsx:45-91)
- `inline-flex; align-items:center; gap 8px; padding: 6px 12px`(`--spacing-1-5` / `--spacing-3`)、radius 10px、`14px / 500 / 20px`(`--text-label-*`)、色 `--color-text-secondary`(`#474747` / `#9e9e9e`)、下線なし。高さ 6+20+6 = **32px**。
- hover: 背景 `--color-overlay-hover`(`#0000000D` / `#FFFFFF0D`)、文字色は変えない。押下 `--color-overlay-pressed`(`#0000001A` / `#FFFFFF1A`)。
- selected: 文字 `--color-text-primary`(`#000000` / `#ffffff`)、`font-weight 600`、背景 `--color-neutral`(`#0000000F` / `#FFFFFF1A`)(hover / press 中も neutral)。
- disabled: 文字 `--color-text-disabled`(`#919191` / `#525252`)、pointer-events none。
- icon only: `padding-inline: 8px`(正方形に近い 36×32px: icon 20px の場合。UNRESOLVED: アイコンサイズは呼び出し側)。
- focus-visible: 標準リング(2px accent `#1b1b1b` / `#f1f1f1`、offset 3px)。
- drawer 描画: SideNav 行と同じ navItem(15.4: 32px, px 8, radius 10, primary 色, selected neutral + 500)。

**TopNavMenu**(TopNavMenu.tsx:64-245)
- trigger: TopNavItem と同じ寸法(padding 6px 12px, 14/500/20, secondary, radius 10, gap 8px)+ `chevronDown` 16px(1em 箱)。hover で overlay-hover。**open 中**: 文字 primary + 背景 `--color-overlay-hover` 固定。chevron は open で `rotate(180deg)`。
- popover: placement below、align = 配置 slot(start / center / end)、offset 4px。面は 0.8 Popover 既定(radius 12, shadow low)。container `column; gap 4px; min-width 280px; padding 4px; user-select none`。
- item: `flex; align-items:center; gap 12px; padding 12px; radius 10px`、hover `--color-overlay-hover`、transition bg 125ms。icon 枠 **40×40px**・radius 10px・背景 `--color-neutral`(`#0000000F` / `#FFFFFF1A`)。content `column; gap 4px`: title 14px / 600 / 20px / primary、description 12px / 400 / 20px / secondary。
- drawer 描画: セクション header = navItem 行(space-between)+ chevron 16px(開で 180deg)。items は `grid-template-rows: 0fr ↔ 1fr` の開閉、item は navItem + `padding-inline-start 24px`、icon 20×20、text column gap 2px、description 12px secondary 400。

**TopNavMegaMenu**(TopNavMegaMenu.tsx:72-277)
- trigger: TopNavMenu と同じ(open で primary + overlay-hover、chevron 180deg)。
- panel: **nav 全体を anchor** にして below、align = slot。外枠 `display:flex(開時) column; max-height: calc(100% - 12px)`。
- panel container: 背景 `--color-background-popover`(`#ffffff` / `#1b1b1b`)、**上辺だけ 1px `--color-border`**(`#00000014` / `#FFFFFF1A`)、radius 12px、`--shadow-low`(0.8)、overflow hidden。
- panel content: `flex; wrap; gap 24px; padding 12px; max-width: min(960px, 100dvw - 16px); overflow-y auto`。
  - items: `Grid columns={2} gap={2}`(2 列 `1fr 1fr`, gap 8px)、`flex: 2 1 300px; min-width 0`。
  - featured: `flex: 1 1 200px; radius 12px; 背景 --color-background-muted(#f1f1f1 / #1b1b1b); overflow hidden; column`。
- MegaMenuItem(desktop): `flex; align-items:flex-start; gap 12px; padding 12px; radius 10px; width 100%`、hover overlay、bg transition 125ms。icon 枠 40×40・radius 10・`--color-neutral` 背景・アイコン色 `--color-icon-secondary`。title 14 / 600 / 20 primary、description 12 / 400 / 20 secondary、content gap 4px。
- MegaMenuItem(drawer): navItem + `padding-inline-start 24px; align-items:flex-start`、icon 枠 32×32・radius 10・neutral・margin-top 2px、content gap 2px、description 12px secondary。
- FeaturedCard: column。image `width 100%; height 140px; object-fit cover`。body `column; gap 8px; padding 16px`: title 14 / 600 / 20 primary、description 12 / 20 secondary、link 12px / 600 / 20px / `--color-text-accent`(light `#1b1b1b` / dark `#f1f1f1`)+ " →"(RTL でミラー)、下線なし。
- drawer 描画の featured: `margin-top 8px; margin-inline-start 24px; radius 12; muted 背景`。

```
[● Acme ▾]   Home  [Products ▾]  Docs                     [🔍] [👤]
 heading      ↑item  ↑mega trigger(open: primary + hover bg)
┌──────────────────────────────────────────────────────────────┐ ← nav 下に panel(上辺 1px, r12, shadow low)
│ [▢] Analytics          [▢] Ads        │ ┌──────────────────┐ │
│     Measure…               Reach…     │ │  image 140px     │ │
│ [▢] Automation         [▢] Commerce   │ │  Title / desc    │ │
│                                       │ │  Learn more →    │ │
└──────────────────────────────────────────────────────────────┘
```

### 16.5 Motion
- trigger / item: `background-color, color 125ms(--duration-fast) cubic-bezier(0.24, 1, 0.4, 1)`。
- chevron: `transform 125ms` 同 easing、0 ↔ 180deg。
- TopNavMenu popover と heading menu: **アニメーション無し(即時に出て即時に消える)**。`usePopover` フックには既定の出現アニメーションが無く(packages/core/src/Popover/usePopover.tsx に `layerAnimations` の参照なし)、TopNavMenu も render options に xstyle を渡していない(TopNavMenu.tsx:648-653)。
- **MegaMenu panel**(TopNavMegaMenu.tsx:129-159): 開: `opacity 0 → 1`、`translateY(-4px) → 0`、**225ms(`--duration-medium-min`)`cubic-bezier(0.24, 1, 0.4, 1)`**。閉: 同じ値で逆方向(allow-discrete で display / overlay も 225ms 保持)。reduced motion + coarse: opacity のみ 200ms `cubic-bezier(0.42, 0, 0.58, 1)`、reduced + fine: 0s。
- drawer 内の開閉: `grid-template-rows 0fr ↔ 1fr` 300ms(`--duration-medium`)ease-standard、開時 overflow visible は 300ms 後。
- hover intent: TopNavMenu show 150ms / hide 200ms、MegaMenu show 150ms / hide 250ms、heading menu show 0ms / hide 200ms(useMenuHover 既定: packages/core/src/hooks/useMenuHover.ts:200-202)。safe triangle(trigger からメニューへ斜めに移動しても閉じない)padding 8px、hover で開いた直後 500ms 以内のクリックは閉じずに固定(click guard)、閉じた直後 300ms は再オープン抑止。
- Kivact: panel は `enter(style().opacity(0.0).translate_y(-4)).exit(style().opacity(0.0).translate_y(-4)).transition(Paint, Transition::tween(225, CubicBezier(0.24, 1.0, 0.4, 1.0)))`。hover intent は `kivact::interaction::use_delayed`(Delays{ show: 150, hide: 250 })。

### 16.6 Behaviour & a11y
- `nav` landmark 名 "Top navigation"。
- TopNavItem: link。selected に aria-current=page、disabled に aria-disabled + tabindex −1。icon only は名前 = label。
- TopNavMenu: trigger は button(hover / click / Enter で開く)、popover は `role=menu` 名 = label、項目 `role=menuitem` tabindex −1 roving。↑↓(useListFocus)、typeahead、Enter/Space で click、Escape で閉じて trigger へ。light dismiss。押したまま項目へスライドして離すと実行。
- TopNavMegaMenu: panel は `role=group` 名 = label(menu ではない)、中のリンク/ボタンを roving(PANEL_ITEM_SELECTOR)。light dismiss、Esc で閉じる。`onOpenChange` 通知。
- mobile-bar: start/center は表示せず、MobileNavToggle(drawer に中身がある & hasAutoToggle)。drawer: TopNavItem 押下で `closeMobileNav()`、TopNavMenu / MegaMenu はアコーディオン(button `aria-expanded` / `aria-controls`、閉時 `aria-hidden` + `inert`)。
- heading menu: trigger 名 "{heading}, Open menu"(heading 無しなら "Open menu")、dialog 名 "Navigation menu"。
- 既定テキスト(提案キー):
  - `soft.top_nav.label` = "Top navigation"
  - `soft.top_nav.heading_open_menu` = "{heading}, Open menu"(heading 無し: "Open menu")
  - `soft.top_nav.heading_dialog` = "Navigation menu"

### 16.7 Kivact 実装方針
- `soft::top_nav()`: centerContent 有無で `row(..)` / `grid(..).style(style().grid_template_columns((Track::fr(1), auto(), Track::fr(1))))`。描画モードは AppShell から `provide(TopNavRender::Inline | MobileBar | Drawer)`、配置 slot は `provide(TopNavSlot::Start | Center | End)` で menu の align に使う。
- TopNavItem: link 見た目の press ノード(`use_button` + `Role::Link`)、selected / disabled は state style。
- TopNavMenu: trigger + `kivact::interaction::use_popup`(below, align = slot, offset 4)+ `use_delayed`(hover intent)+ `use_list`(menu policy, typeahead)。面は soft popover style。標準 `kivact::ui::menu` を restyle して使う案もある(MenuEntry に description と 40px アイコン枠を足せるなら)— 項目レイアウトが特殊なので kit 実装を推奨。
- MegaMenu: anchor を nav 全体の NodeRef にする(`overlay(nav_ref, Placement::Below, ..)`)。2 列 grid は `soft::grid().columns(Fixed(2)).gap(S2)`。
- drawer 用のアコーディオンは `kivact::interaction::use_disclosure` + 高さ animated(15.7 と同じ)。
- safe triangle: UNRESOLVED(kivact の hover popup に安全三角形が無ければ hide 遅延 250ms のみで代替)。
- basic の navigation_menu.rs は shadcn の共有 viewport アニメ(幅/高さが項目間で morph)を持つが、Astryx にはその morph は **無い**(項目ごとに独立 popover)。流用しないこと。

### 16.8 受け入れ確認
- [ ] navigation "Top navigation"、高さ 48px(padding 8 + 項目 32)、heading と項目群の間 16px、項目間 4px。
- [ ] 項目文字 14px/500 secondary、hover で `#0000000D` 背景、selected は primary 600 + `#0000000F`(dark `#FFFFFF1A`)背景・aria-current。
- [ ] Menu trigger に 150ms ホバーで下に menu(min 280px、項目 12px padding・40px アイコン枠)、離れて 200ms で閉じる。開いている間 trigger は primary + hover 背景、chevron 上向き。
- [ ] ↓ で menu 項目へ、文字入力で該当項目、Esc で閉じて trigger にフォーカス。
- [ ] MegaMenu は nav の下に 225ms で 4px 下降しながらフェードイン、2 列項目 + featured(muted 背景、画像 140px)。閉じも 225ms。
- [ ] centerContent ありで中央 slot が画面中央に正確に揃う。
- [ ] mobile 幅で heading + end + ハンバーガーのみ。drawer 内で Menu がアコーディオンになり 300ms で開閉。


## 17. MobileNav (+ MobileNavToggle)

docs: https://astryx.atmeta.com/components/MobileNav 。source: packages/core/src/MobileNav (MobileNav.tsx, MobileNavToggle.tsx, MobileNav.doc.mjs, MobileNavCloseTiming.test.ts, MobileNavEntryAnimation.test.tsx, MobileNavFocus.test.tsx, MobileNavReopen.test.tsx, MobileNavScrollbarGutter.test.tsx, MobileNavPinchZoom.a11y.chromium.spec.ts ほか)。

**判定: RESTYLE + kit wrapper。** 本質はモーダルな横 drawer = 標準 `kivact::ui::dialog(..)` の `DialogKind::Drawer(Side::Start | End)`(crates/kivact-interaction/src/dialog.rs の `use_dialog`)。soft の dialog recipe に MobileNav 用 variant(`soft::MobileNavLook`)を足し、kit の `soft::mobile_nav(..)` が AppShell の mobile context と header / close button / content 構造を被せる。basic の対応: crates/kivact-ui-basic/src/sidebar.rs の mobile sheet、および sheet.tsx 相当の Drawer recipe(crates/kivact-ui-basic/src/recipes_dialog.rs)。

### 17.1 概要
- 狭い viewport 用の、端から滑り出るナビゲーション drawer。SideNav のモバイル版で同じ子(SideNavSection / SideNavItem)を受ける。nav 項目は変数に切り出して SideNav と共有する。
- 中身の目的が自明でないときは header を付ける。AppShell 内では MobileNavToggle を trigger に使い、toggle に isOpen / onOpenChange を渡さない。デスクトップでは使わない(MobileNav.doc.mjs bestPractices)。
- AppShell は below breakpoint で自動的に sideNav / topNav を MobileNav に入れる(1.2)。

### 17.2 Anatomy
```
dialog (modal, fixed inset 0, 100vw × 100dvh, 透明, aria-label=header|"Navigation")   .astryx-mobile-nav
├─ ::backdrop (--color-overlay + blur 2px, opacity 0→1)
└─ drawer (div tabindex -1, absolute top/bottom 0, start|end 側, width 320 (max 85vw), surface 背景, column)
    ├─ header (flex, space-between, min-height 48, padding-inline 8, border-bottom 1px)
    │   ├─ header: string → Heading h2 (margin-inline-start 4) / ReactNode / なし(右寄せ)
    │   └─ Close button (ghost icon-only, "Close navigation", icon close)
    └─ content (flex 1, overflow-y auto, padding 8, overscroll contain)
        └─ children
MobileNavToggle: ghost icon-only button (icon menu) aria-expanded aria-controls=drawer   ← mobile かつ有効時のみ描画
```

### 17.3 API

MobileNav(MobileNav.tsx:382-439):

| prop | type | default | 意味 | Kivact(`soft::mobile_nav(content)`) |
|---|---|---|---|---|
| isOpen | boolean | AppShell context | 開閉 | `.open(Binding<bool>)`(省略時は `use_app_shell_mobile()` の状態) |
| onOpenChange | (bool) => void | AppShell context | 閉じ要求(backdrop / Esc / close button) | binding 書き込み |
| children | ReactNode(必須) | — | 中身 | `content: impl FnOnce() -> V`(開くたび新しい branch: kivact-ui-basic の drawer と同じく slot 関数) |
| header | ReactNode | なし | 見出し(string は h2) | `.header(impl IntoText)` / `.header_view(|| ..)` |
| width | number | 320 | 幅 px(最大 85vw) | `.width(f32)` |
| side | `'start' \| 'end' \| 'auto'` | `'auto'` | 出る側。auto = 開いた時のフォーカス要素(trigger)の中心が画面左半分なら start、右半分なら end。初期値 end | `.side(DrawerSide::Start \| End \| Auto)` |
| label | string | header 文字列 or "Navigation" | dialog 名 | `.name(..)` |
| xstyle / className / style / ref / data-testid | — | — | — | 破棄 / NodeBuilder |

MobileNavToggle(MobileNavToggle.tsx:34-52): `children?`(アイコン差替え)、`label?` = "Open navigation"、他は button props。mobile でないか mobile nav 無効なら描画しない。→ `soft::mobile_nav_toggle()`.

### 17.4 Visual spec(MobileNav.tsx:93-272)
- backdrop: `background: --color-overlay`(light `#00000080` / dark `#000000CC`)、`backdrop-filter: blur(2px)`、opacity 0 → 1。
- drawer: `position:absolute; top:0; bottom:0`; 幅 `width`px(既定 **320px**)、`max-width: 85vw`; 背景 `--color-background-surface`(light `#ffffff` / dark `#262626`); `display:flex; flex-direction:column; overflow:hidden`; **角丸なし・影なし**。
  - start 側: 左端(RTL 右端)、内側の辺に `1px solid --color-border`(light `#00000014` / dark `#FFFFFF1A`)。閉: `translateX(-100%)`(RTL `+100%`)、開: `translateX(0)`。
  - end 側: 右端、左辺に 1px border、閉: `translateX(100%)`(RTL `-100%`)。
- header: `display:flex; align-items:center; justify-content:space-between; min-height: --spacing-12 48px; padding-inline: 8px; flex-shrink 0; border-bottom: 1px solid --color-border`。header 無しは `justify-content:flex-end`(close button だけ右)。string header は `Heading level 2` = `--text-heading-2` 20px / 600 / 28px / `--color-text-primary`(`#000000` / `#ffffff`)、`margin-inline-start: 4px`。
- close button: ghost icon-only md 32×32px、icon `close`(Lucide `x`)md 20px、色 inherit = text primary。
- content: `flex:1; overflow-y:auto; overflow-x:hidden; padding: 8px`(上下左右)、overscroll contain、`touch-action: pan-y pinch-zoom`。中の SideNav 項目は SideNav と同じ行スタイル(15.4)。drawer-content 描画の footer は `margin-top:auto; padding-top 8px; gap 8px`、footerIcons は `flex; gap 4px`。
- dialog 自体: 透明、`overflow:clip`、`touch-action: pinch-zoom`(ピンチズームを許す)、overlayPaddingReset。
- MobileNavToggle: ghost icon-only md 32×32px、icon `menu`(Lucide `menu`、basic の ICON_NAMES に無いので soft が追加)20px、色 text primary。**アイコンのモーフ(☰→✕)は無い**。

```
┌──────────────────────┐▒▒▒▒▒▒▒▒▒▒▒▒▒   ▒ = backdrop #00000080 + blur 2px
│ Acme            [✕]  │▒▒▒▒▒▒▒▒▒▒▒▒▒   header 48px, border-bottom
├──────────────────────┤▒▒▒▒▒▒▒▒▒▒▒▒▒
│ [■] Home             │▒▒▒▒▒▒▒▒▒▒▒▒▒   content padding 8
│ [□] Projects         │▒▒▒▒▒▒▒▒▒▒▒▒▒
└──────────────────────┘▒▒▒▒▒▒▒▒▒▒▒▒▒
  320px (max 85vw), surface, 内側辺 1px border
```

### 17.5 Motion(MobileNav.tsx:135-230, 286-348)
- 開く: drawer `transform: translateX(∓100%) → 0`(@starting-style)、**300ms(`--duration-medium`)`--ease-standard` = cubic-bezier(0.24, 1, 0.4, 1)**。backdrop opacity 0 → 1 同 300ms 同 easing。dialog の `display` は allow-discrete で 300ms 保持。
- 閉じる: drawer を `translateX(∓100%)` へ 300ms 同 easing、backdrop 1 → 0 300ms。`dialog.close()`(フォーカス返却)は `min(250ms, 最短 transition-duration(300ms) × 0.6) = **180ms**` 後(resolveCloseDelay)。reduced motion では transition 0.01s・close 遅延 0。
- 再オープン(閉じアニメ中に開く)は close タイマーを取り消して開き直す。
- Kivact: `kivact::ui::dialog(.., DialogKind::Drawer(Side::Start))` の soft recipe で `enter(style().translate_x(pct(-100)))` / `exit(..)` + `transition(Paint, Transition::tween(300, CubicBezier(0.24, 1.0, 0.4, 1.0)))`、backdrop は `enter/exit opacity 0` 300ms。exit transition が終わるまで kivact がノードを残す(crates/kivact-style/src/sheet.rs:871-889)ので close 遅延の手当ては不要。
- backdrop blur 2px: kivact `backdrop_blur(2)`(sheet.rs:738)。

### 17.6 Behaviour & a11y
- `showModal()` のモーダル dialog: focus trap、背景 inert、`aria-label` = header 文字列 / label / "Navigation"。開いた時のフォーカスは drawer(tabindex −1)内。閉じると開いた要素(trigger)にフォーカスを戻す(接続されていれば)。
- 閉じ要求: Esc(cancel)、backdrop クリック(dialog 自身へのクリック)、close button。layer dismissal で入れ子 layer がある間は Esc を内側に譲る。
- 開いている間 `document.documentElement` を `overflow: clip` にし、スクロールバー幅の gutter を保持してレイアウトずれを防ぐ(holdScrollbarGutter)。
- SideNavItem を押すと drawer を閉じる(15.6)。
- MobileNavToggle: `aria-expanded` = open、`aria-controls` = drawer id、押下で toggle。
- 既定テキスト(提案キー):
  - `soft.mobile_nav.navigation` = "Navigation"(dialog 名の既定)
  - `soft.mobile_nav.close` = "Close navigation"
  - `soft.mobile_nav.open` = "Open navigation"(toggle)

### 17.7 Kivact 実装方針
- `soft::mobile_nav(content)` は `kivact::ui::dialog(name, ..).kind(DialogKind::Drawer(side))` を包み、`.content(..)` に header + `scroll(column(content))` を入れる。close button は soft の dialog recipe 側の閉じボタンを使わず kit で header 内に置く(`.closable(false)` にして自前: kivact-ui-basic README の AlertDialog と同じ作法)。
- 幅: `style().width(320).max_width(window_size().width * 0.85)`(reactive)。
- side auto: 開いた時点の focus 要素の rect 中心 x と `window_size().width / 2` を比較(kivact で「現在フォーカス中ノードの rect」を取れるか UNRESOLVED。取れなければ AppShell が toggle の NodeRef を渡す: toggle は通常右上なので end)。
- スクロールロック / gutter: native ウィンドウでは不要(破棄)。
- toggle: `button(..).variant(Ghost).icon(icon("menu")).hide_label(true)`、`expanded(open)` と controls 関係(kivact の semantics に controls があれば)。

### 17.8 受け入れ確認
- [ ] mobile 幅でハンバーガー(32px, "Open navigation", expanded=false)。押すと右(または左)から 320px の drawer が 300ms で滑り出し、背景が `#00000080`(dark `#000000CC`)+ 2px ぼかし。
- [ ] dialog 名 = header 文字列、無ければ "Navigation"。Tab が drawer 内で循環。
- [ ] Esc / 背景クリック / ✕ で 300ms で滑り戻り、フォーカスがハンバーガーへ戻る。
- [ ] header 48px・下 1px 線、string header は 20px/600。content 8px padding でスクロール。
- [ ] 幅 360px の画面で drawer 幅 306px(85vw)。
- [ ] RTL で start drawer が右から出る。reduced motion で即時。


## 18. NavIcon

docs: https://astryx.atmeta.com/components/NavIcon 。source: packages/core/src/NavIcon/NavIcon.tsx。

**判定: NEW kit component(極小、表示専用)。** 円形の accent 背景にアイコンを置くだけ。basic に同等なし(見た目は `basic::avatar` の円に近い: crates/kivact-ui-basic/src/avatar.rs)。

### 18.1 概要
- TopNavHeading / SideNavHeading などナビゲーション見出しでアプリ・セクションを示す円形アイコン。
- Icon など同程度の大きさのアイコンを渡す。操作用(ボタン)には使わない(NavIcon.doc.mjs bestPractices)。

### 18.2 Anatomy
```
span.astryx-nav-icon (flex center, 円, accent 背景)
└─ icon
```

### 18.3 API(NavIcon.tsx:41-49)

| prop | type | default | Kivact |
|---|---|---|---|
| icon | ReactNode(必須) | — | `soft::nav_icon(icon("box"))` |
| xstyle / className / style / ref | — | — | 破棄 / NodeBuilder |

### 18.4 Visual spec(NavIcon.tsx:27-39)
- `width/height: --size-element-md` = **32×32px**、`border-radius: 50%`、`flex-shrink:0`、中央揃え。
- 背景 `--color-accent`(light `#1b1b1b` / dark `#f1f1f1`)、前景(アイコン色)`--color-on-accent`(light `#ffffff` / dark `#111111`)。
- 中のアイコンサイズは呼び出し側(doc 例 `size="sm"` 16px)。
- 状態なし。NavIcon を含む heading は `padding-inline-start: 0` になる(SideNavHeading.tsx:82-85, 246-249)ので、kit の heading は「icon が NavIcon か」を builder で知る必要がある(`soft::side_nav_heading(..).icon(..)` が `NavIcon` 型を受けたら start padding 0)。

### 18.5 Motion
なし。

### 18.6 Behaviour & a11y
装飾。role も名前も持たない(heading テキストが名前)。

### 18.7 Kivact 実装方針
`node().style(style().width(32).height(32).radius(CornerRadius::full()).background(soft::Color::Accent).align_items(Align::Center).justify_content(Justify::Center).color(soft::Color::OnAccent)).child(icon)`(UNRESOLVED: kivact の完全円の radius 指定名。無ければ radius 16)。

### 18.8 受け入れ確認
- [ ] 32px の円、light は黒地 `#1b1b1b` に白アイコン、dark は `#f1f1f1` 地に `#111111` アイコン。
- [ ] SideNavHeading / TopNavHeading 内で左 padding が 0 になり、円が行の左端に揃う。


## 19. NavMenu (NavHeadingMenu, NavHeadingMenuItem)

docs: https://astryx.atmeta.com/components/NavMenu (URL は UNRESOLVED: doc 名は NavMenu.doc.mjs。NavHeadingMenu として出ている可能性)。source: packages/core/src/NavMenu (NavHeadingMenu.tsx, NavHeadingMenuItem.tsx, NavMenuContext.tsx, NavMenu.doc.mjs, NavHeadingMenu.test.tsx)。

**判定: NEW kit component(小)**、ただしメニューのキーボード挙動は `kivact::interaction::use_menu` / `use_list` を流用。SideNavHeading / TopNavHeading の `menu` slot に入れる「アプリ/ワークスペース切替メニュー」。標準 `kivact::ui::menu` は自前の trigger とポップアップを持つので、heading の popover の **中身** としてはこの kit 部品を使う。basic の近い例: crates/kivact-ui-basic/src/list_popup.rs / recipes_menu.rs。

### 19.1 概要
- ナビ見出しのメニュー(アプリ切替、アカウント、ワークスペース)。項目はアイコン + ラベル + 任意の説明。
- 見出し(SideNavHeading / TopNavHeading)の `menu` にだけ置く。項目選択でメニューは閉じる。

### 19.2 Anatomy
```
NavHeadingMenu (div role=menu, column, gap 2px, min-width 160|200|240)     .astryx-nav-heading-menu
└─ NavHeadingMenuItem (a[href] | div, role=menuitem, tabindex -1)          .astryx-nav-heading-menu-item
    ├─ [icon] (sm 16px, secondary)
    └─ content (column, flex 1)
        ├─ label (Text body, 1 行)
        └─ [description] (Text supporting, 1 行)
```

### 19.3 API

NavHeadingMenu(NavHeadingMenu.tsx:59-74):

| prop | type | default | Kivact(`soft::nav_heading_menu(items)`) |
|---|---|---|---|
| children | ReactNode(必須) | — | `items: impl IntoView`(nav_heading_menu_item の列) |
| size | `'sm' \| 'md' \| 'lg'` | `'md'` | `.size(ControlSize::..)`(min-width と item padding) |
| minWidth | number \| string | size 既定 | `.min_width(f32)` |
| onKeyDown 等 | — | — | 破棄 |

NavHeadingMenuItem(NavHeadingMenuItem.tsx:92-109):

| prop | type | default | Kivact(`soft::nav_heading_menu_item(label)`) |
|---|---|---|---|
| label | ReactNode(必須) | — | 第 1 引数 `impl IntoText`(または `.content(|| ..)`) |
| icon | ReactNode \| IconType | なし | `.icon(icon(..))` |
| description | ReactNode | なし | `.description(..)` |
| href / onClick | string / fn | なし | `.on_press(impl Fn())` |
| isDisabled | boolean | false | `.disabled(..)` |

### 19.4 Visual spec
- menu: `column; gap: --spacing-0-5` 2px; min-width sm 160 / md 200 / lg 240px。面(背景・影・角丸 12px・padding 4px)は親の heading popover(15.4 / 16)が持つ(0.8 Popover 既定面)。
- item(NavHeadingMenuItem.tsx:34-90): `flex; align-items:center; gap 8px; width 100%; border-radius: --radius-element 10px; font-family body; font-size --text-label-size 14px; color --color-text-primary`(light `#000000` / dark `#ffffff`); 背景透明、text-align start、outline none、user-select none。
  - padding: sm `4px 8px`、md `8px 8px`、lg `12px 12px`。
  - hover(hover 端末)**と :focus(キーボード移動先)**: 背景 `--color-overlay-hover`(light `#0000000D` / dark `#FFFFFF0D`)。フォーカスリングは出さない(outline none; 背景がフォーカス表示)。
  - disabled: opacity 0.5、cursor default。
- icon: size sm 16px、`--color-icon-secondary`(light `#6a6a6a` / dark `#9e9e9e`)。
- label: Text body 14px / 400 / 20px、1 行 ellipsis(tooltip 付き truncation)。description: Text supporting 12px / 400 / 20px、`--color-text-secondary`(`#474747` / `#9e9e9e`)、1 行。
- md の 1 行項目の高さ = 8 + 20 + 8 = 36px、説明付きは 56px。

### 19.5 Motion
自身には無い(popover の出現は親: 0.6 の 165ms enter)。

### 19.6 Behaviour & a11y
- `role=menu`、各項目 `role=menuitem` / tabindex −1(roving はフォーカス移動で)、disabled は `aria-disabled`(href あり disabled でもフォーカスは可)。
- キー: ↑↓ で移動(useListFocus)、typeahead(項目テキスト)、Enter / Space で選択(click)、Escape で閉じる(heading の closeMenu)、Tab で閉じる(親 popover の挙動)。
- ポインタ: 押したまま項目上をスライドして離した項目を実行(useMenuPress)、メニュー外で離すと閉じる。
- 選択で `onClick` → `closeMenu()`。
- 既定テキストなし。

### 19.7 Kivact 実装方針
- `soft::nav_heading_menu(items)`: `column(items).role(Role::Menu)`。キー操作は `kivact::interaction::use_menu`(MenuEntry ベース)ではなく任意 View の項目なので `use_list`(menu policy, typeahead, activation Enter/Space)を使い、`provide(NavMenuCx{ close, size })` で項目に閉じ関数とサイズを渡す。
- 項目のフォーカス表示は背景のみ: `style().focus(style().background(soft::Color::OverlayHover)).focus_visible(style().ring_none())`。
- press-slide(押したまま移動)は kivact の menu 実装に同等があれば流用(UNRESOLVED)、無ければ通常クリックのみ。

### 19.8 受け入れ確認
- [ ] heading の menu を開くと role menu の中に menuitem が並び、↓ で次項目の背景が `#0000000D`(dark `#FFFFFF0D`)になる(リングなし)。
- [ ] Enter で項目が実行されメニューが閉じ、フォーカスが heading に戻る。
- [ ] 文字入力で該当ラベルへジャンプ。
- [ ] md: 行 36px・角丸 10px、説明付きは 2 行(12px secondary)。disabled は半透明で実行されない。


## 20. Stepper / Step

docs: https://astryx.atmeta.com/components/Stepper (Step は sub-component)。source: packages/core/src/Stepper (Stepper.tsx, Step.tsx, StepStatus.ts, StepperContext.ts, stepper.stylex.ts, Stepper.doc.mjs, Step.doc.mjs, Stepper.test.tsx)。

**判定: NEW kit component。** 多段フローの進捗表示。段は `ol > li`、非線形(`onStepClick`)のとき各段がボタン。狭い横幅では自動で「区切りトラック + 要約行(前/次ボタン)」に縮退。最も近い basic: crates/kivact-ui-basic/src/questionnaire.rs(段の進行 UI)と crates/kivact-ui/src/progress.rs。

### 20.1 概要
- 段ラベルは短く("Payment")。説明が長い段は vertical。横の Stepper は 1 段あたり `minimumStepWidth`(既定 112px)未満になるとラベルを落として区切りトラックにし、`collapsedVariant` の要約行を下に出す。ページ側に Back/Continue があるなら `'withLabel'`、周辺 UI が見出しもナビも持つなら `'hiddenLabel'`。
- 前の段に戻れる非線形フローは `onStepClick`。`status` は意味色(accent / success / warning / error)だけ。凝った表示は custom icon。
- 3 段未満・7 段超では使わない(Stepper.doc.mjs bestPractices)。

### 20.2 Anatomy
```
[horizontal] frame (div, column, --step-connector-gap 0)        .astryx-stepper-frame
├─ ol aria-label="Progress" (flex row, align start, gap 4px | on-track 0)   .astryx-stepper
│   └─ li Step [aria-current=step]  (column, flex 1, min-width 0)            .astryx-step
│       ├─ bar (4px 高さ, radius full, track + accent fill)  aria-hidden     .astryx-step-bar
│       ├─ button | div (density padding)       ← onStepClick && !compact のとき button
│       │   ├─ icon-label row (flex, gap 8): [indicator 16px][label][VisuallyHidden status][• Optional][endContent]
│       │   └─ description row (padding-start 24 = indicator 16 + gap 8)
│       └─ content (children; padding-top 8)
└─ [compact summary] (flex, space-between, gap 8): [‹ prev] [summary slot: active step の indicator+label+description, 中央] [next ›]
[vertical] ol (flex column, gap 4px) > li (row: bar 4px 幅 | body column)
[on-track] indicator が track 上(各段に前後 2 本の connector segment)
```

### 20.3 API

Stepper(Stepper.tsx:97-146):

| prop | type | default | 意味 | Kivact(`soft::stepper(active, steps)`) |
|---|---|---|---|---|
| activeStep | number(0 始まり) | 必須 | 現在段。より前は completed | `active: impl IntoValue<usize>` |
| children | Step 群 | 必須 | — | `steps: impl IntoView`(`soft::step(i, label)` の列) |
| orientation | `'horizontal' \| 'vertical'` | `'horizontal'` | 向き | `.orientation(Orientation::..)` |
| onStepClick | (index) => void | なし | 非線形。設定で各段がボタン(compact 時は要約の前/次) | `.on_step_press(impl Fn(usize))` |
| label | string | "Progress" | ol の名前 | `.name(..)` |
| density | `'compact' \| 'balanced' \| 'spacious'` | `'balanced'` | 全段の padding | `.density(StepDensity::..)` |
| indicatorPosition | `'separated' \| 'on-track'` | `'separated'` | indicator を track と分けるか track 上か | `.indicator_position(..)` |
| horizontalOptions | `{minimumStepWidth: number, collapsedVariant: 'withLabelAndControls'\|'withLabel'\|'hiddenLabel'}` | `{112, 'withLabelAndControls'}` | 横の縮退 | `.min_step_width(f32)` / `.collapsed_variant(..)` |
| `--step-connector-gap`(CSS var) | length | 0px | on-track の connector が indicator 手前で空ける隙間(0〜8px にクランプ) | `.connector_gap(f32)` |

Step(Step.tsx:88-160):

| prop | type | default | 意味 | Kivact(`soft::step(index, label)`) |
|---|---|---|---|---|
| step | number(必須) | — | 0 始まり index(重複は警告) | 第 1 引数 |
| label | string(必須) | — | ラベル | 第 2 引数 |
| description | string | なし | 補足 | `.description(..)` |
| children | ReactNode | なし | 段の中身(compact 時 hidden) | `.content(|| ..)` |
| status | `'accent'\|'success'\|'warning'\|'error'` | なし | 意味色 | `.status(StepStatus::..)` |
| isDisabled | boolean | false | 無効(フォーカス可・activate しない) | `.disabled(..)` |
| isOptional | boolean | false | "• Optional" | `.optional(true)` |
| endContent | ReactNode | なし | ラベル行末 | `.end_content(|| ..)` |
| indicator | `'auto'\|'number'\|'none'\|ReactNode` | `'auto'` | auto: 未着手=番号、現在=リング、完了=チェック。number: 常に番号。none: 無し。node: custom icon | `.indicator(StepIndicator::Auto \| Number \| None \| Custom(Box<dyn ..>))` |
| density | 同上 | Stepper の値 | 段ごと上書き | `.density(..)` |

### 20.4 Visual spec(Step.tsx:187-978, Stepper.tsx:148-212)

**共通寸法**
- bar / connector の太さ `--spacing-1` = **4px**、radius full(on-track の segment は radius 0)。
- track 色 `--color-border`(light `#00000014` / dark `#FFFFFF1A`)、fill 色 `--color-accent`(light `#1b1b1b` / dark `#f1f1f1`)。fill は `::before` を scaleX(0↔1)(縦は scaleY、origin 左/上、RTL は右)。
- 完了段と現在段の bar は filled、未着手は empty。
- indicator 箱 `--spacing-4` = **16×16px**(番号・チェック・リング・custom 共通)。indicator + gap 8 = 24px が説明・中身の字下げ(INDICATOR_GUTTER)。
- horizontal bar: `width 100%; height 4px; margin-bottom: 2px`。step は `flex:1; min-width:0; column; align stretch`。ol の gap 4px(on-track は 0)。
- vertical: li `row; align stretch; gap 2px`、bar は `width 4px; align-self stretch`、body column flex 1。ol gap 4px。

**indicator の状態**(Step.tsx:518-613, 1135-1274)
| 状態 | auto 表示 | 色 |
|---|---|---|
| not-started | 番号バッジ(`step+1`) | 背景 `--color-background-muted`(`#f1f1f1` / `#1b1b1b`)、文字 `--color-text-secondary`(`#474747` / `#9e9e9e`) |
| in-progress(現在) | CurrentIcon: 半径 7 の 2px リング + 半径 4 の塗り円(16px SVG) | `--color-accent`(`#1b1b1b` / `#f1f1f1`) |
| completed | CheckCircleIcon: 半径 8 の塗り円 + 白抜きチェック(stroke = `--color-background-surface` `#ffffff` / `#262626`、1.75px、round) | `--color-accent` |
| disabled | 番号: muted 背景 + `--color-text-disabled`(`#919191` / `#525252`)+ opacity 0.5 / アイコン: `--color-icon-disabled` + opacity 0.5 |
| status success / warning / error(現在以外) | Icon `success`(circle-check)/ `warning`(triangle-alert)/ `error`(circle-x)、16px | `--color-icon-success` light `#2f7d33` / dark `#90ca90`、`--color-icon-warning` `#836700` / `#eec448`、`--color-icon-error` `#c0262c` / `#ffaea7` |
| 番号バッジ + status(到達済み) | — | accent: `--color-accent` 背景 + `--color-on-accent`(`#ffffff` / `#111111`)。success: `--color-success`(`#198100` / `#64af4c`)+ `--color-on-success`(`#ffffff` / `#111111`)。warning: `--color-warning` `#ffce2f` + `--color-on-warning` `#111111`。error: `--color-error`(`#c9303a` / `#ff705d`)+ `--color-on-error`(`#ffffff` / `#111111`) |
| 番号バッジ completed / in-progress(status なし) | — | 背景 accent、文字 `--color-background-surface`(`#ffffff` / `#262626`) |
- 番号バッジ: `display:grid; place-items:center; 16×16; radius full; font-size --font-size-xs 10px; weight 600`。forced-colors で 1px outline。

**label / description**
- label: `14px / 400 / 20px(--text-body-*)`、`--color-text-primary`(`#000000` / `#ffffff`)、1 行 ellipsis、min-width 0。現在段は **600**。未着手は `--color-text-secondary`。disabled は `--color-text-disabled`。
- optional: "•" と "Optional" を 14px secondary。
- description: `12px(--text-supporting-size)`、line-height **16px**(`--spacing-4`)、secondary、block。indicator がある段は `padding-inline-start 24px`。

**density / 押下領域**(separated)
- block padding: compact 4px / balanced 8px / spacious 12px。inline padding: spacious 12px、他 8px。中身(children)は `padding-top 8px; padding-inline-start = inline pad + 24px; padding-inline-end = inline pad`。
- button(onStepClick): `display:flex; column; width 100%; text-align start; radius 10px; 背景 transparent`、hover `--color-overlay-hover`(`#0000000D` / `#FFFFFF0D`)、押下 `--color-overlay-pressed`、focus-visible で標準リング。disabled は `aria-disabled`(cursor default)。

**on-track**(Step.tsx:738-977, 1425-1760)
- horizontal: 各段の上部に `row` の track: [前 segment(flex 1, min 8px)][indicator][後 segment(flex 1)]、先頭段の前 segment と末尾段の後 segment は `visibility:hidden`。ラベル列は中央寄せ(`otLabelWrapH`: column, gap 2px, text-align center; ラベル行 `flex; center; wrap; gap 4px`)。前 segment は「現在段以前」で filled、後 segment は「完了段」で filled。
- vertical: 左に indicator 列(幅 16px、column center): [lead segment(高さ = density space)][indicator][rail segment(flex 1, min 8px)]、本文 `column; gap 2px`。行全体 padding = density space(4/8/12)。content がある段は content 横にも 3 本目の segment。
- `--step-connector-gap`: segment を `clip-path: inset(..)` で indicator 側だけ削る(0〜8px)。

**compact summary**(Stepper.tsx:187-211, 497-592)
- 1 段あたり幅 `rootWidth / stepCount < 112px` で compact。bar だけが並び(ラベルは VisuallyHidden)、下に summary 行: `flex; align center; justify space-between; gap 8px`。中央 slot に現在段の indicator + label(+ description)を `column; align center; gap 2px; text-align center` で描く(aria-hidden。名前は各 li の VisuallyHidden にある)。
- 左右に ghost IconButton(md 32×32、`chevronLeft` / `chevronRight` 20px、RTL ミラー、tooltip = 名前)。隣の **有効な** 段が無ければ disabled。`withLabel` はボタン無し、`hiddenLabel` は summary 行自体無し。

```
separated horizontal (balanced)
████████████ ████████████ ░░░░░░░░░░░░ ░░░░░░░░░░░░   ← 4px bars (accent / border)
(✓) Account  (◉) Payment  (3) Review    (4) Done
   desc         desc 600     secondary
```

### 20.5 Motion(Step.tsx:255-323, 407-501, 1096-1128)
- **進捗 fill**: 「ちょうど 1 段進んだ」ときだけ、新たに覆われた区間の fill を `transform: scaleX(0 → 1)`(縦は scaleY)で **300ms(`--duration-medium`)linear**。戻る・2 段以上ジャンプ・マウント時・reduced motion はすべて即時(0s)。
- on-track では 1 区間を 2 本(縦は content 付きで 3 本)の segment が分担するので、時間を長さ比で分ける: 横は到着側 0.5 / 出発側 0.5、縦は到着 0.3 / 出発 0.7(content segment があれば出発の 0.25 が rail、0.75 が content)。順番に delay を付けて前線が途切れず進む: 出発 segment(前の段の後ろ)は delay 0・duration 300×share、到着 segment(次の段の前)は delay 300×出発share・duration 300×到着share。
- 押下領域の背景: `background-color 95ms(--duration-fast-min) cubic-bezier(0.24, 1, 0.4, 1)`、reduced 0s。
- indicator の切替(番号 → リング → チェック)は即時。
- Kivact: fill は `scale_xy(progress, 1.0)` + `transform_origin(左)` を paint transition `Transition::tween(300, Easing::Linear)` + `.delay(..)`。「1 段進んだか」は前回 active を `signal` で保持して判定し、それ以外は transition を付けずに値を置く(`Animated::jump_to`)。

### 20.6 Behaviour & a11y
- `ol` 名 "Progress"、各 `li`、現在段に `aria-current="step"`。
- 状態の読み上げ: 完了 / success は "completed"、warning は "warning"、error は "error" を VisuallyHidden で label の後に。現在段は aria-current で伝え、未着手は何も足さない。
- onStepClick: 段のボタン名 = "Go to step {n}: {label}"(状態ありは "Go to step {n}: {label}, {status}")、`aria-describedby` = description。disabled はフォーカス可能・"unavailable" として読まれ(aria-disabled)、押しても activate しない。Tab で段を順に移動、Enter/Space で activate。
- compact: 段ボタンは消え、要約の前/次ボタン("Previous step" / "Next step")が disabled 段を飛ばして移動。各 li は VisuallyHidden の label と状態を保つ。
- 既定テキスト(提案キー):
  - `soft.stepper.label` = "Progress"
  - `soft.stepper.previous` = "Previous step"、`soft.stepper.next` = "Next step"
  - `soft.step.go_to` = "Go to step {number}: {label}"、`soft.step.go_to_with_status` = "Go to step {number}: {label}, {status}"
  - `soft.step.optional` = "Optional"
  - `soft.step.status_completed` = "completed"、`soft.step.status_warning` = "warning"、`soft.step.status_error` = "error"

### 20.7 Kivact 実装方針
- `soft::stepper(active, steps)` は `provide(StepperCx{ active, previous_active, orientation, on_press, density, indicator_position, compact })` で段に状態を渡す。`previous_active` は active の変化を effect で記録。
- compact 判定: frame の `node_ref().rect()` 幅 / 段数 < min_step_width。段数は steps の Vec 長(builder 時)で分かるので登録の仕組みは不要。
- indicator の SVG(CurrentIcon / CheckCircleIcon)は `Style::paint` で円と線を描く(16px 箱に r=7 の 2px リング + r=4 塗り、r=8 塗り + チェック path `M4.75 8.25 L7 10.5 L11.25 6`)。
- 段ボタンは `kivact::interaction::use_button`、名前は message テンプレート。
- on-track は separated の後に実装してよい(優先度低)。clip-path の gap は `clip_path`(sheet.rs:424)で。
- 最も近い basic: crates/kivact-ui-basic/src/questionnaire.rs(進行と前後ボタン)。

### 20.8 受け入れ確認
- [ ] list "Progress"、現在段に aria-current=step、完了段の名前に "completed"。
- [ ] 3 段目が現在のとき bar 1–3 が accent(`#1b1b1b` / `#f1f1f1`)、4 は `#00000014` / `#FFFFFF1A`。indicator は ✓ ✓ ◉ (4)。
- [ ] activeStep を 1 増やすと新しい bar が左から 300ms linear で伸びる。戻す・2 段飛ばしは即時。
- [ ] 現在段ラベル 600、未着手は secondary、description 12px / 16px 行高で 24px 字下げ。
- [ ] onStepClick 有りで段にホバー背景(角丸 10px)、Tab/Enter で移動、disabled 段は押しても変わらない。
- [ ] 幅を 5 段 × 112px 未満に縮めると bar だけになり、下に "‹ (◉) Payment ›" 要約。端で ‹ / › が disabled。
- [ ] status error の未来段は赤い ⓧ(`#c0262c` / dark `#ffaea7`)、number indicator + error は `#c9303a` 背景に白数字。


## 21. TabList / Tab / TabMenu

docs: https://astryx.atmeta.com/components/TabList (Tab / TabMenu は sub-component)。source: packages/core/src/TabList (TabList.tsx, Tab.tsx, TabMenu.tsx, TabListContext.ts, tab.markers.stylex.ts, docs, TabList.test.tsx, __tests__)。

**判定: RESTYLE `kivact::ui::tabs` + kit wrapper。** 標準 `kivact::ui::tabs(name, items, binding)`(crates/kivact-ui/src/tabs.rs:37-120: roving・`.activation(..)`・`.item(..)` slot・`.content(..)` panel)を soft の「下線タブ」recipe で描く(kivact-ui-basic の `TabsVariant::Line` と同じ作法: crates/kivact-ui-basic/src/line_tabs.rs)。標準 tabs に無いもの — ナビゲーション pattern(`role` 無し = nav landmark + aria-current)、横スクロール strip のフェードマスクと矢印、TabMenu(タブ位置のドロップダウン)、full bleed — は kit wrapper `soft::tab_list(..)` が担う。

### 21.1 概要
- ビュー間のナビゲーション。ラベルは短く。溢れ処理は有効のまま(strip が横スクロールし選択タブを常に表示)。厳選した追加項目は TabMenu。
- hasDivider と横のアクションボタンを並べるときは Button のサイズを TabList と揃える(md と md)。
- その場でパネルを切り替えるなら `role="tablist"` + 各 Tab に `panelId`。ビュー間の移動なら role 無し(nav)。
- padded な LayoutHeader / Card / Section の中で端まで伸ばすなら isFullBleed。
- 手順には使わない(→ Stepper)。6–8 個を超えて並べない。SegmentedControl / ToggleButton と混同しない(TabList.doc.mjs bestPractices)。

### 21.2 Anatomy
```
nav aria-label="Tabs"  |  div (role="tablist" のとき)       .astryx-tab-list  (flex, stretch, gap 2px)
├─ strip (div [role=tablist aria-label]) (flex, gap inherit 2px, flex 1, 横スクロール, scrollbar 非表示, フェード mask)   .astryx-tab-strip
│   ├─ Tab (button | a) [role=tab aria-selected aria-controls] or [aria-current=true]   .astryx-tab
│   │   ├─ hover bg (absolute, 全面, radius 10)
│   │   ├─ [icon 14|16|18px]
│   │   ├─ label (inline-grid: 表示テキスト + 600 太さの不可視 sizer で幅固定)
│   │   ├─ [endContent]
│   │   └─ indicator (absolute bottom, 左右 12px 内側, 高さ 2px, radius full)   .astryx-tab-indicator
│   └─ TabMenu trigger (button aria-haspopup=menu [data-tab-menu])   .astryx-tab-menu
│       └─ label(選択中 option のラベル or label)+ chevron ▾ + indicator(選択時)
│           popover (role=menu): heading(label) + menuitemradio × N(✓)
├─ ‹ arrow button (absolute start, 円, popover 面)   ← overflowStart && hover 端末
└─ › arrow button (absolute end)                      ← overflowEnd && hover 端末
```

### 21.3 API

TabList(TabList.tsx:93-168):

| prop | type | default | 意味 | Kivact(`soft::tab_list(name, binding)`) |
|---|---|---|---|---|
| value / onChange | string / fn | 必須 | 選択値(caller-owned) | `binding: impl Into<Binding<K>>` |
| size | `'sm'\|'md'\|'lg'` | `'md'`(SizeContext 継承) | 高さ | `.size(ControlSize::..)` |
| layout | `'hug' \| 'fill'` | `'hug'` | hug: 内容幅 / fill: 等分で全幅 | `.layout(TabLayout::Hug \| Fill)` |
| hasDivider | boolean | false | 下に 1px 線 | `.divider(bool)` |
| isFullBleed | boolean | false | 親容器の inline padding を打ち消す | `.full_bleed(bool)` |
| role | AriaRole | なし(= nav pattern) | `'tablist'` で tabs pattern | `.pattern(TabPattern::Navigation \| Tabs)` |
| overflow | `'auto'\|'scroll'\|'visible'` | `'auto'`(= scroll) | 溢れ処理 | `.overflow(TabOverflow::Scroll \| Visible)` |
| aria-label / aria-labelledby | string | "Tabs" | 名前 | 第 1 引数 `name` |
| children | Tab / TabMenu | — | 中身 | `.tabs(..)` に `soft::tab(value, label)` / `soft::tab_menu(label, options)` の列 |

Tab(Tab.tsx:47-98):

| prop | type | default | 意味 | Kivact(`soft::tab(value, label)`) |
|---|---|---|---|---|
| value | string(必須) | — | 一致で選択 | 第 1 引数 `K` |
| label | string(必須) | — | 表示 / 名前 | 第 2 引数 |
| isLabelHidden | boolean | false | アイコンのみ(名前は label) | `.label_hidden(true)` |
| href / as | — | — | nav pattern でリンク | 破棄(`on_change` で遷移) |
| panelId | string | なし | tabs pattern の aria-controls | `.panel(NodeRef)`(標準 tabs の `.content(..)` を使うなら自動) |
| icon / selectedIcon | ReactNode | なし | 非選択 / 選択時アイコン | `.icon(..)` / `.selected_icon(..)` |
| endContent | ReactNode | なし | badge / 状態ドット | `.end_content(|| ..)` |
| aria-disabled | boolean | — | 無効 | `.disabled(..)` |

TabMenu(TabMenu.tsx:48-71): `label`(必須: 未選択時の trigger 文字と dropdown 見出し)、`options: {value, label, icon?}[]`(必須)。→ `soft::tab_menu(label, options: Vec<TabMenuOption<K>>)`。

### 21.4 Visual spec

**TabList / strip**(TabList.tsx:190-381)
- root: `display:flex; align-items:stretch; gap: --spacing-0-5 2px; max-width 100%; min-width 0; relative`。fill で width 100%。
- hasDivider: `border-bottom: 1px solid --color-border`(light `#00000014` / dark `#FFFFFF1A`)、`padding-bottom: 4px`、indicator を線の上に重ねるため `--_tab-indicator-bottom = -(4px + 1px) = -5px`(indicator が divider に乗る)。divider 無しは indicator bottom −1px。
- strip(scroll): `overflow-x:auto; overflow-y:hidden; scrollbar-width:none; scroll-behavior:smooth`(reduced: auto)、`scroll-padding-inline = bleed + 32px`。フォーカスリングが切れないよう、strip 内に focus-visible があるとき四辺に `ring bleed = 2px + 3px = 5px` の padding と同量の負 margin。
- **フェードマスク**(溢れ時): 開始側が溢れていれば `mask-image: linear-gradient(to right, transparent <bleed>, black <bleed+32px>)`、終了側は逆向き、両方なら両端。RTL で反転。mask の変化は `transition: mask-image 300ms ease-standard`(reduced 0)。
- 矢印ボタン(hover 端末のみ表示、overflowStart / overflowEnd のとき): `absolute; top 0; 円(radius full)`、サイズ = tab 高さ(sm 28 / md 32 / lg 36 の正方形)、背景 `--color-background-popover`(`#ffffff` / `#1b1b1b`)、`--shadow-low`、文字 `--color-text-secondary`(`#474747` / `#9e9e9e`)→ hover で primary、アイコン `chevronLeft` / `chevronRight` 16px。`aria-hidden`・tabindex −1(キーボードは矢印キーでスクロールされる)。押下で strip 幅の **80%** だけ scrollBy(smooth)。
- full bleed: root を `margin-inline: -container-padding-*` で広げ、strip は `padding-inline = bleed + max(container padding - 12px, 0)`(先頭/末尾タブの内側 12px と合わせて端のラベルが揃う)。

**Tab**(Tab.tsx:110-267)
- `relative; inline-flex; align/justify center; gap 4px; padding-inline: 12px`(`--spacing-3`)、高さ sm 28 / md 32 / lg 36px、radius 10px、背景透明、`14px / 400 / 20px`(`--text-label-*`)、色 `--color-text-secondary`(`#474747` / `#9e9e9e`)、nowrap、hug は shrink 0、fill は flex 1 + center。
- hover: 内側の hoverBg(全面・radius 10)が `--color-overlay-hover`(`#0000000D` / `#FFFFFF0D`)、押下 `--color-overlay-pressed`(`#0000001A` / `#FFFFFF1A`)。文字色は変えない。
- selected: 文字 `--color-text-primary`(`#000000` / `#ffffff`)、`font-weight 600`。ラベルは 600 の不可視 sizer と重ねてあるので **選択で幅が変わらない**。
- indicator: `absolute; bottom -1px(divider 時 -5px); left/right 12px; height 2px; radius full`。selected で `--color-accent`(`#1b1b1b` / `#f1f1f1`)opacity 1、非選択は透明 opacity 0。
- icon: sm 14 / md 16 / lg 18px 角。
- disabled(aria-disabled): cursor default、hover / press 無し(色は secondary のまま。UNRESOLVED: disabled の文字色指定は無い)。
- focus-visible: 標準リング(2px accent、offset 3px、角丸 10px)。

**TabMenu**(TabMenu.tsx:77-248)
- trigger: Tab と同じ寸法・色(secondary 400、選択 option があれば primary 600)。label は選択 option のラベル(無ければ `label`)、600 sizer 付き。chevron 枠 16×16px、`chevronDown` 16px、open で rotate 180deg。選択 option があれば下に indicator(2px、色 `--color-icon-primary` `#000000` / `#ffffff`: Tab の accent とわずかに違う)。hover bg は Tab と同じ。
- trigger の tabindex: 選択 option を持つとき 0(roving の tab stop)、それ以外 −1。
- dropdown(popover below / start、面は 0.8 Popover 既定): `column; gap 2px; padding 4px`。heading: 12px / 600 / 20px / secondary、padding 4px 12px。item(menuitemradio): `flex; space-between; gap 8px; padding 8px 12px; radius 10; 14 / 400 / 20; primary`、hover overlay-hover、選択は weight 500 + 右に `check` 16px `--color-accent`。option icon は左(sm)。

```
 Overview   Reports   [Settings]  More ▾                ← hug, md 32px
            ‾‾‾‾‾‾‾                                       ← 2px accent indicator (左右 12px 内側)
──────────────────────────────────────────────────────  ← hasDivider 1px (indicator は線に重なる)
```

### 21.5 Motion
- Tab 文字色: `color 125ms(--duration-fast) cubic-bezier(0.24, 1, 0.4, 1)`。
- hover bg: `background-color 125ms` 同 easing。
- indicator: **スライドしない**。各タブの indicator が `opacity, background-color 125ms` 同 easing でフェード(旧タブが消え新タブが現れる)。
- TabMenu chevron: `transform 125ms`、0 ↔ 180deg。dropdown の出現はアニメーション無し(usePopover 直使用: 16.5 と同じ理由)。
- strip のスクロール: smooth(ブラウザ既定曲線)、矢印 1 回で 80% ページ。フェードマスク切替は 300ms。
- Kivact: 色・不透明度は paint transition 125ms。mask の切替は kivact の `mask(..)`(gradient: sheet.rs:419-422)を opacity 違いの 2 つの gradient で持ち、paint transition できないなら即時切替(UNRESOLVED: mask の補間可否。不可ならフェード無し)。

### 21.6 Behaviour & a11y
- nav pattern(role 無し): `nav` 名 "Tabs"(または指定)、各タブはボタン/リンク、選択に `aria-current="true"`。
- tabs pattern(`role="tablist"`): strip が `role=tablist`(名前付き)、各 Tab `role=tab` + `aria-selected` + `aria-controls`(panelId)。tablist の中に tab 以外を置くと dev 警告(TabMenu は置かない)。
- キー: roving tabindex(選択タブが tab stop)。tabs pattern は ←→、nav pattern は ←→↑↓ で移動(useListFocus orientation)、Home/End。**フォーカス移動だけでは選択しない**(manual activation: Enter / Space / クリックで onChange)。初回キー操作時に keyboard hint(useKeyboardHint)を出す(UNRESOLVED: hint の見た目は別グループ)。
- フォーカスしたタブ・選択したタブは strip 内に自動でスクロールして見せる(scroll-padding 32px + bleed を考慮、幅超過タブは先頭揃え)。リサイズ時も選択タブを再表示。
- TabMenu: trigger は `aria-haspopup=menu`, `aria-expanded`, `aria-controls`。開くと先頭項目にフォーカス、↑↓、Enter / クリックで選択して閉じる、Esc / Tab で閉じて trigger にフォーカス。項目は `menuitemradio` + `aria-checked`。
- 既定テキスト: `soft.tab_list.label` = "Tabs"(`@astryx.tabList.label`)。

### 21.7 Kivact 実装方針
- tabs pattern: 標準 `kivact::ui::tabs(name, items, binding).activation(ListActivation::Manual)`(既定は `Automatic` = フォーカスで選択なので **必ず Manual にする**: crates/kivact-interaction/src/list/data.rs:119-123)+ soft の `TabsRecipes`(下線・hover bg・600 sizer)。`.item(|item, state| ..)` slot で icon / label(太さ固定)/ endContent を描く。
- nav pattern / TabMenu を含む strip: 標準 tabs は MenuEntry 型の項目しか持てないので、kit の `soft::tab_list` が `kivact::interaction::use_tabs`(TabsApi)または `use_list`(roving, manual activation)で自前に組む。TabMenu は `kivact::ui::menu(label, entries).attach(trigger)` を restyle(checkmark 付き radio 項目)して trigger をタブ列に置く。
- 太さで幅が変わらない工夫: label を `stack((text_600_invisible, text_visible))`(kivact の `stack` は重ね合わせ: 最初の子がサイズを決める)で再現できる。
- 横スクロール: `scroll(row(tabs)).axes(ScrollAxes::Horizontal)` + `scrollbar_width(None)`、溢れ判定は 11 の `soft::use_scroll_overflow`。選択タブの表示は `NodeRef::scroll_into_view()`(node_ref.rs:402)か `scroll_to` で 32px の余白込みに。矢印は hover 端末(`!capabilities().touch`)のみ。
- 最も近い basic: crates/kivact-ui-basic/src/line_tabs.rs(標準 tabs の line variant recipe)。

### 21.8 受け入れ確認
- [ ] nav "Tabs"(または tablist + tab の selected / controls)。
- [ ] md タブ高 32px、文字 14px secondary、選択で primary 600 だが **タブ幅が変わらない**、下に 2px accent 線(左右 12px 内側)。
- [ ] 選択変更で旧 indicator がフェードアウト、新 indicator がフェードイン(125ms、滑らない)。
- [ ] hover で角丸 10px の `#0000000D` 背景。
- [ ] ←→ でフォーカスだけ移動、Enter で選択。Home/End。
- [ ] 狭い幅で strip が横スクロール、端に 32px のフェード、マウス時は円形の ‹ › が出て 80% ずつ送る。選択タブは常に見える。
- [ ] hasDivider で 1px 線、indicator が線の上に重なる。
- [ ] TabMenu: "More ▾" → menu(見出し + 項目)、選んだ項目名が trigger に出て indicator(`#000000` / `#ffffff`)が付く、✓ が選択項目に。


## 22. Card

docs: https://astryx.atmeta.com/components/Card 。source: packages/core/src/Card/Card.tsx(+ Card.doc.mjs, Card.test.tsx)。padding 伝播: packages/core/src/Layout/container.stylex.ts(0.2)。

**判定: NEW kit component(容器)。** 角丸・枠・背景・影・テーマ padding を持ち、子に container padding を告知する(Layout / Section / Divider の bleed 基準)。最も近い basic: crates/kivact-ui-basic/src/card.rs(`card`, `card_header`, `CardSize`)。

### 22.1 概要
- 独立して並べ替え・削除・操作できる **離散した 1 個のもの**(1 人のプロフィール、1 件の通知、1 つの指標、グリッドの 1 商品)の枠。既定のレイアウト道具ではない。
- 「独立に並べ替え/削除できるか?」が Yes なら Card、No なら見出し + Stack か Section。余白とタイポグラフィでグループ化できるなら Card を外す。兄弟カードの padding を揃える。ヘッダ + スクロール本文 + アクション付きフッタは Layout と組む。
- スマホでは行/グリッドに並べてよい(カードはトラックに縮み、折り返せない内容は端で clip)。長い ID/URL は 1 行省略、幅の広い内容は ScrollableArea。
- ページ region を Card で包まない(→ Section)、同じ形のカードを機械的に並べない、Card を入れ子にしない、ステータスを色 variant で表さない(→ Banner / Badge)(Card.doc.mjs bestPractices)。

### 22.2 Anatomy
```
div.astryx-card (radius 12, overflow clip, [border 1px], bg, box-shadow = ring + elevation, padding = container)
└─ children
```

### 22.3 API(Card.tsx:177-237)

| prop | type | default | 意味 | Kivact(`soft::card(children)`) |
|---|---|---|---|---|
| children | ReactNode | — | 内容 | `children: impl IntoView` |
| padding | SpacingStep | テーマの card padding(neutral **12px**、テーマ無し 16px) | 4 辺 padding。指定はテーマ値の上書き | `.padding(Space)` |
| variant | `'default'\|'transparent'\|'muted'\|'blue'\|'cyan'\|'gray'\|'green'\|'orange'\|'pink'\|'purple'\|'red'\|'teal'\|'yellow'` | `'default'` | 背景 | `.variant(CardVariant::..)` |
| elevation | `'none'\|'low'\|'med'\|'high'` | `'none'` | 常時の影 | `.elevation(Elevation::..)` |
| width / height / maxWidth / minHeight | SizeValue | なし | サイズ。height 固定(auto 以外)で `overflow:auto`(スクロール) | `impl Into<Length>` |
| xstyle / className / style / ref | — | — | — | 破棄 / NodeBuilder |

### 22.4 Visual spec(Card.tsx:59-173)
- `border-radius: --radius-container` = **12px**(`--_card-radius`)、`min-width:0`、`overflow:clip`、`box-sizing: border-box`。
- **default のみ** `1px solid --color-border`(light `#00000014` / dark `#FFFFFF1A`)、padding は container 値から 1px 引く(外形からの内容位置は border 有無で同じ)。他の variant は枠なし。
- 背景:
  | variant | light | dark |
  |---|---|---|
  | default | `--color-background-card` `#ffffff` | `#1b1b1b` |
  | transparent | 透明 | 透明 |
  | muted | `--color-background-muted` `#f1f1f1` | `#1b1b1b` |
  | blue | `#b9d7ff` | `#253c5a` |
  | cyan | `#9ae2f4` | `#274046` |
  | gray | `#e2e2e2` | `#303030` |
  | green | `#bce0bb` | `#2b422b` |
  | orange | `#ffc7a1` | `#503424` |
  | pink | `#ffc0d7` | `#572b3d` |
  | purple | `#f7d5ff` | `#4a2f51` |
  | red | `#ffc4be` | `#5b2b28` |
  | teal | `#a9e2d6` | `#28413c` |
  | yellow | `#fae19e` | `#453a1c` |
  (`--color-background-<hue>`、neutralTheme.ts の palette 参照から解決。dark の default card `#1b1b1b` は body と同色なので、dark では枠線が主な区切りになる)
- 影(`--_card-elevation`、`--_card-ring` と合成: `box-shadow: ring, elevation`):
  - none: なし
  - low: light `0 2px 4px rgba(0,0,0,.05), 0 4px 8px rgba(0,0,0,.10)` / dark `0 2px 4px rgba(0,0,0,.25), 0 4px 8px rgba(0,0,0,.40), inset 0 0 0 1px rgba(255,255,255,.08)`
  - med: light `0 2px 4px .05, 0 4px 12px .10` / dark `0 2px 4px .35, 0 4px 12px .50, inset 1px white .12`
  - high: light `0 4px 6px .10, 0 12px 24px .15` / dark `0 4px 6px .50, 0 12px 24px .70, inset 1px white .15`
- padding: 未指定はテーマ値(neutral 12px)。`padding={n}` で 4 辺 `--spacing-n`。子に outer/inner = 同値を告知(0.2)→ 中の Layout は Card の縁に密着し region が 12px inset、Section / Divider full-bleed は Card の縁まで。
- 状態なし(hover / focus は ClickableCard / SelectableCard)。テーマ target `astryx-card`(variant, elevation)、`theming.container: true`。

```
┌──────────────────────┐  r=12, 1px #00000014 (default)
│ 12px                 │
│   content            │
│                      │
└──────────────────────┘
```

### 22.5 Motion
なし。

### 22.6 Behaviour & a11y
role なし(容器)。既定テキストなし。

### 22.7 Kivact 実装方針
- `soft::card(children) -> Card`(NodeBuilder): `column(children).style(style().radius(12).overflow(Overflow::Hidden).background(..).border(1, soft::Color::Border).padding(pad - 1))`、elevation は `shadow(..)` を 2〜3 枚(dark の inset は `Shadow::..inset()`)。ring と elevation を合成するため、Card は ring 用の inset shadow を先頭に入れる口(`pub(crate) fn ring(..)`)を SelectableCard に提供する。
- `provide(ContainerInset{ 12,12,12,12 }, ..)` と `LayoutInset{ outer/inner = 12 }` を子へ(0.2)。
- height 固定で `scroll(..)` に切替。
- basic の crates/kivact-ui-basic/src/card.rs は shadcn の header/content/footer 構造を持つが、Astryx Card は slot を持たない素の容器(構造は Layout に任せる)。

### 22.8 受け入れ確認
- [ ] 既定: 角丸 12px、1px `#00000014`(dark `#FFFFFF1A`)、背景 `#ffffff`(dark `#1b1b1b`)、内側 12px。
- [ ] `variant(Muted)` で枠なし `#f1f1f1`、`Blue` で `#b9d7ff`(dark `#253c5a`)。
- [ ] `elevation(Med)` で影(dark は 1px の明るい縁も)。
- [ ] `padding(S6)` で 24px(default は border 1px 込みで内容が外形から 24px)。
- [ ] Card 内の Layout header の divider が Card の左右端まで届く。
- [ ] height 固定で内容がスクロール。


## 23. ClickableCard

docs: https://astryx.atmeta.com/components/ClickableCard 。source: packages/core/src/ClickableCard/ClickableCard.tsx(+ docs, ClickableCard.test.tsx)、packages/core/src/hooks/useClickableContainer.ts、packages/core/src/utils/interactionOverlay.stylex.ts。

**判定: NEW kit component(Card + 押下可能)。** Card(22)の上に「カード面全体が 1 つの動作/遷移」を足す。中の入れ子ボタン・リンクは独立に動く(nested interactive 問題の解決)。basic に同等なし(`basic::item` の clickable が近い: crates/kivact-ui-basic/src/item.rs)。

### 23.1 概要
- 詳細ページへの遷移、または 1 つの動作のトリガになるカード。中にボタン/リンクを自由に入れてよい(それらは自分のイベントを処理し、カードの動作は起きない)。
- 選択の切替には使わない(→ SelectableCard)(ClickableCard.doc.mjs bestPractices)。

### 23.2 Anatomy
```
Card (div, relative, cursor pointer, ::after overlay, focus-within ring)   .astryx-clickable-card
├─ 隠しフォーカス対象: <a href> | <button> (sr-only, aria-label=label, tabindex 0)   ← キーボード / AT 用の唯一の tab stop
├─ ::after (absolute inset 0, hover / pressed overlay)
└─ children(入れ子の操作要素は独立)
```

### 23.3 API(ClickableCard.tsx:204-275)

| prop | type | default | 意味 | Kivact(`soft::clickable_card(label, children)`) |
|---|---|---|---|---|
| label | string(必須) | — | 隠しボタン/リンクの名前 | 第 1 引数 |
| onClick | (e) => void | なし | カード面(入れ子操作要素以外)のクリック | `.on_press(impl Fn())` |
| href / target | string | なし / `'_self'` | 遷移(中クリック・Cmd/Ctrl で新規タブ) | 破棄 → `on_press`(新規ウィンドウはアプリ判断。修飾キーは `on_press_with(|mods| ..)` で渡す案: UNRESOLVED) |
| isDisabled | boolean | false | 無効(tab stop は残り "disabled" と読まれる) | `.disabled(..)` |
| padding / variant / elevation / width / height / maxWidth / children | — | Card と同じ(padding doc 既定 4 だが実際はテーマ値 = neutral 12px) | — | Card と同じ |

### 23.4 Visual spec(ClickableCard.tsx:75-198 + Card)
- Card の見た目(22.4)すべて + `position:relative; cursor:pointer; color: inherit; 下線なし`。
- **hover overlay**(`::after` inset 0、角丸は Card の clip で 12px): 休止 透明 → hover(hover 端末, 非 disabled)`--color-overlay-hover`(light `#0000000D` / dark `#FFFFFF0D`)→ 押下 `:active` `--color-overlay-pressed`(`#0000001A` / `#FFFFFF1A`)。overlay はカードの内容の **上** に重なる(pointer-events none)。
- default variant(枠あり)の hover: 枠色 `--color-border` → `--color-border-emphasized`(light `#d4d4d4` / dark `#525252`)。
- 他 variant: 枠なし(`border-width:0`)、overlay のみ。
- focus: 隠し要素が `:focus-visible` のとき **カード全体** に標準リング(2px `--color-accent` light `#1b1b1b` / dark `#f1f1f1`、offset 3px、角丸 12px に沿う)。中の入れ子要素が focus-visible のときもカードにリングが付く(`:has(:focus-visible)`: focusOutline.stylex.ts:107-121)。
- disabled: `opacity 0.5`、cursor default、overlay と hover 枠変化なし。
- タッチ: press controller が押下を塗り、離すと 200ms でフェード(0.9)。

### 23.5 Motion
- overlay: `background-color 125ms(--duration-fast) cubic-bezier(0.24, 1, 0.4, 1)`(タッチ press 中は transition none で即時、release は 200ms `cubic-bezier(0,0,0.2,1)` の alpha フェード)。
- 枠色: `border-color 125ms` 同 easing。
- 押下の scale は無し(Button と違う)。
- Kivact: `.hover(style().border_color(BorderEmphasized))` + overlay 子ノード(absolute inset 0、radius 12)の `hover(background(OverlayHover)).pressed(background(OverlayPressed))`、`transition(Paint, tween(125, CubicBezier(0.24,1,0.4,1)))`。overlay は「親が hover」で変わる必要があるので `style().group()`(カード)+ `group_hover(..)`(overlay)を使う。pressed は group に無いので、カードの `NodeRef` の pressed 状態を読む reactive style(UNRESOLVED: NodeRef に pressed 信号があるか。無ければ `use_button` の state から)。

### 23.6 Behaviour & a11y
- カード自体は role を持たず、隠し `<button>`(onClick 時)または `<a>`(href 時)が唯一の tab stop で、名前 = label。Enter / Space(button)・Enter(link)で動作。
- カード面クリック(useClickableContainer): クリック対象とカードの間に操作要素(a, button, input, [role=button] 等)がある場合は無視、テキスト選択中のクリック(ドラッグ選択)も無視。onClick も href も無ければ内部の interactive 要素にクリックを委譲。href は中クリック・Cmd/Ctrl クリックで新規タブ。
- disabled: `aria-disabled`、tab stop は維持、押しても何もしない(リンク先も外す)。disabled ↔ enabled 切替でフォーカスを失わないよう復元。
- 既定テキストなし(label 必須)。

### 23.7 Kivact 実装方針
- `soft::clickable_card(label, children)`: `soft::card(..)` を包み、カード全体を press 対象にする: `kivact::interaction::use_button` の part を **カード root** に付け、`Role::Button`(または `Role::Link`)+ name = label にする(native では隠し要素は不要。a11y ツリー上はカード = ボタン、子の操作要素はその子として残る)。UNRESOLVED: kivact で「ボタンの中にボタン」を a11y 的に許すか。許さない場合は Astryx と同様に「透明な押下層(absolute inset 0)をボタンにし、子の操作要素を z 上位に置く」構成にする。
- 入れ子の操作要素のクリックがカードに伝播しないこと: kivact の press は最も内側の press 対象が取る前提(要確認)。
- ドラッグ選択中のクリック無視: テキスト選択がある場合は on_press を呼ばない(UNRESOLVED: kivact のテキスト選択状態の取得)。

### 23.8 受け入れ確認
- [ ] role button(名前 = label)が 1 つの tab stop、Enter/Space で on_press。
- [ ] hover でカード全体に `#0000000D`(dark `#FFFFFF0D`)が重なり、default は枠が `#d4d4d4`(dark `#525252`)に 125ms で変わる。押下中 `#0000001A`。
- [ ] 中のボタンを押してもカードの on_press は呼ばれない。
- [ ] Tab でカード全体に角丸 12px のリング。
- [ ] disabled で半透明、押しても無反応、Tab では止まる。


## 24. SelectableCard

docs: https://astryx.atmeta.com/components/SelectableCard 。source: packages/core/src/SelectableCard/SelectableCard.tsx(+ docs, SelectableCard.test.tsx)。

**判定: RESTYLE `kivact::ui::checkbox` 相当の semantics + kit wrapper(Card)。** 各カードは checkbox(複数選択)。behaviour は `kivact::interaction::use_checkbox`(CheckboxApi)、見た目は Card + 選択リング。最も近い basic: crates/kivact-ui-basic/src/choice.rs(`toggle_group` の選択見た目)と card.rs。

### 24.1 概要
- 複数選択の選択肢(フィルタ chip、機能アドオン、オプショングリッド)。選択 ID は呼び出し側が Set で持つ。フォーカス中は Space / Enter で切替。
- ナビゲーションには使わない(→ ClickableCard)。単一選択グループにも使わない(各カードが checkbox なので、AT には「複数選択・カードごとに tab stop」と伝わる)(SelectableCard.doc.mjs bestPractices)。

### 24.2 Anatomy
```
Card (div, relative, cursor pointer, ::after overlay, focus-within ring, 選択で border + inset ring)   .astryx-selectable-card
├─ input[type=checkbox] (sr-only, aria-label=label, checked)   ← 唯一の tab stop
├─ ::after overlay (inset 0, radius inherit)
└─ children(入れ子の操作要素は独立)
```

### 24.3 API(SelectableCard.tsx:262-321)

| prop | type | default | 意味 | Kivact(`soft::selectable_card(label, selected, children)`) |
|---|---|---|---|---|
| label | string(必須) | — | checkbox 名 | 第 1 引数 |
| isSelected / onChange | boolean / (bool) => void | 必須 | 選択状態 | `selected: impl Into<Binding<bool>>` |
| isDisabled | boolean | false | 無効 | `.disabled(..)` |
| onClick / onMouseUp | — | なし | 追加ハンドラ | `.on_press(..)`(任意) |
| padding / variant / elevation / width / height / maxWidth / children | — | Card と同じ | — | Card と同じ |

### 24.4 Visual spec(SelectableCard.tsx:69-256 + Card)
- Card(22.4)+ `position:relative; cursor:pointer`。
- hover / pressed overlay: ClickableCard と同じ `::after`(`#0000000D` / `#FFFFFF0D`、押下 `#0000001A` / `#FFFFFF1A`、radius inherit 12px)。**hover で枠色は変えない**(ClickableCard と違う)。
- **selected**: `border-color` と `--_card-ring = inset 0 0 0 2px <色>`(Card の box-shadow 先頭に入る)。default / transparent / muted は `--color-accent`(light `#1b1b1b` / dark `#f1f1f1`)。default は border 1px も accent になるので外縁から **1px border + 2px inset = 3px** の accent 帯、transparent / muted は border 無しで 2px の inset のみ。色 variant は対応する `--color-border-<hue>` で ring(border は無いので 2px のみ):
  | variant | light | dark |
  |---|---|---|
  | blue | `#a1caff` | `#5aa0f8` |
  | cyan | `#85d5e9` | `#49adc4` |
  | gray | `#d4d4d4` | `#262626` |
  | green | `#aad4a9` | `#6ab26b` |
  | orange | `#ffc7a1` | `#df843f` |
  | pink | `#ffc0d7` | `#fc78b1` |
  | purple | `#f3bfff` | `#d885eb` |
  | red | `#ffaea7` | `#fa6762` |
  | teal | `#90d7c8` | `#4fb1a0` |
  | yellow | `#e3c36c` | `#c29900` |
  未知 variant は `--selectable-card-ring-color`(既定 accent)。
- チェックマーク等の **indicator は描かない**(リングだけが選択表示)。
- focus: checkbox が focus-visible でカード全体に標準リング(2px accent、offset 3px、radius 12)。
- disabled: opacity 0.5、overlay 無し。

```
 未選択                   選択 (default)
┌────────────────┐       ┏━━━━━━━━━━━━━━━━┓  1px border accent + inset 2px accent
│ Feature A      │       ┃ Feature A      ┃
└────────────────┘       ┗━━━━━━━━━━━━━━━━┛
```

### 24.5 Motion
- `box-shadow, border-color 125ms(--duration-fast) cubic-bezier(0.24, 1, 0.4, 1)`(選択リングのフェードイン/アウト)。
- overlay `background-color 125ms` 同 easing(タッチは 0.9 の 200ms release)。
- Kivact: inset shadow と border color を paint transition 125ms(kivact の paint transition は「最初の 2 つの shadow」を補間する: sheet.rs:151-157 → ring を shadow 配列の **先頭** に置くこと)。

### 24.6 Behaviour & a11y
- 唯一の tab stop は隠し checkbox(名前 = label、checked = isSelected、disabled は aria-disabled)。Space(checkbox 既定)と **Enter**(追加: SelectableCard.tsx:427-441)で切替。
- カード面クリックで切替(入れ子操作要素のクリックとテキスト選択は無視: useClickableContainer)。カード内から開いた layer(popover 等)内のクリックはカードの onClick の前に処理(SelectableCard.tsx:336-363, 461-473)。
- 既定テキストなし。

### 24.7 Kivact 実装方針
- `soft::selectable_card(label, selected, children)`: `soft::card(..)` の root に `kivact::interaction::use_checkbox(binding)` の part を付け、`Role::CheckBox` + name + checked。Enter でも toggle するよう key handler を追加。
- 見た目: `selected` state style(`style().selected(..)` は任意プロパティ可: sheet.rs:760-766)で border color と inset shadow(先頭)を設定。
- 入れ子操作要素の扱いは 23.7 と同じ課題(UNRESOLVED 同上)。

### 24.8 受け入れ確認
- [ ] role checkbox(名前 = label)、Tab で 1 つずつ止まる、Space / Enter で checked 切替。
- [ ] 選択で 1px + 2px の accent(light `#1b1b1b` / dark `#f1f1f1`)縁が 125ms で現れる。blue variant は `#a1caff`(dark `#5aa0f8`)の 2px。
- [ ] hover で `#0000000D` の重ね、枠色は変わらない。
- [ ] カード内のボタンを押しても選択は変わらない。
- [ ] disabled で半透明・切替不可。


## 25. Carousel

docs: https://astryx.atmeta.com/components/Carousel 。source: packages/core/src/Carousel/Carousel.tsx(+ Carousel.doc.mjs, Carousel.test.tsx, __tests__)。storybook: Carousel.stories.tsx, CarouselA11y.stories.tsx。

**判定: NEW kit component。** Astryx の Carousel は **ページングや物理シミュレーションを持たない「ネイティブ横スクロール strip」**(scroll-snap は任意、smooth scroll、自動送り無し)+ 端に浮く丸い前後ボタン + 4px のエッジフェード。packages/core/src/motion/paging.ts 等は使っていない。kivact-ui-basic の `basic::carousel`(crates/kivact-ui-basic/src/carousel.rs)は「1 枚ずつ translate する slide 型」なので **構造は流用しない**(ボタン・名前キーの作法のみ参考)。

### 25.1 概要
- 横に並ぶ項目(商品、メンバー、写真)を横スクロールで見せる。ギャラリーや商品リストのように各項目を開始端にぴったり止めたいときは `hasSnap`。小さな循環可能な集合(写真)は `hasLoop`。
- 中身を表す `aria-label` を必ず付ける。gap と項目幅を揃える。トラックパッドの横スワイプ、マウスは Shift + ホイールで送れる。
- 全員が見るべき内容を入れない。自動送りしない。入れ子にしない(Carousel.doc.mjs bestPractices)。
- WAI-ARIA APG carousel(自動回転なし)。

### 25.2 Anatomy
```
root (div role=region aria-roledescription="carousel" aria-label)   .astryx-carousel  (relative, flex, center, overflow clip)
├─ scroller (div role=group aria-label tabindex=0 if overflow)       .astryx-carousel-scroller
│   │   (flex, center, overflow-x auto, scrollbar 非表示, smooth, [snap x mandatory], gap, padding-inline, edge-fade mask)
│   └─ item (div role=group aria-roledescription="slide" aria-label="Slide N of M") (flex, shrink 0, snap-align start)
│       └─ slide child
└─ button overlay (layer: root と同寸で重なる, flex space-between, pointer-events none)
    ├─ pill start (popover 面の円, shadow med, translateX(-50%)) > ghost sm icon button "Scroll left" ‹
    └─ pill end   (translateX(+50%)) > ghost sm icon button "Scroll right" ›
```

### 25.3 API(Carousel.tsx:68-155)

| prop | type | default | 意味 | Kivact(`soft::carousel(name, slides)`) |
|---|---|---|---|---|
| children | ReactNode | 必須 | 各子が 1 スライド | `slides: Vec<View>` か `impl IntoView` |
| aria-label | string | "Carousel" | region / group 名 | 第 1 引数 |
| gap | `0 \| 0.5 \| 1 \| 1.5 \| 2 \| 3 \| 4` | `1`(4px) | 項目間 | `.gap(Space)`(S0〜S4 のみ) |
| hasButtons | boolean | true | 前後ボタン | `.buttons(bool)` |
| hasEdgeFade | boolean | true | 端のフェードマスク | `.edge_fade(bool)` |
| hasLoop | boolean | false | 端で反対端へ戻る | `.looping(bool)` |
| hasSnap | boolean | false | `scroll-snap-type: x mandatory`、項目 start 揃え | `.snap(bool)` |
| padding | SpacingStep | なし | scroller の padding-inline と scroll-padding-inline | `.padding(Space)` |
| handleRef | Ref<CarouselHandle> | なし | `scrollNext()`, `scrollPrev()`, `scrollTo(i)`, `canScrollNext()`, `canScrollPrev()` | `soft::CarouselHandle`(Copy handle を `.handle(h)` で渡す、または builder が返す) |
| xstyle / className / style / ref / data-testid | — | — | — | 破棄 / NodeBuilder |

### 25.4 Visual spec(Carousel.tsx:161-308)
- root: `position:relative; display:flex; align-items:center; min-width:0; max-width:100%; overflow:clip`(clip margin 1px)。背景・枠なし。
- scroller: `display:flex; align-items:center; overflow-x:auto; overflow-y:hidden; overscroll-x contain; scroll-behavior:smooth(reduced: auto); scrollbar-width:none`、下に 1px の padding/負 margin(フォーカスリング用)。
- gap: 0 / 2 / 4(既定)/ 6 / 8 / 12 / 16px。padding: SpacingStep(0.1)を inline と scroll-padding-inline に同値。
- item: `display:flex; flex-shrink:0; scroll-snap-align:start`(幅はスライド自身)。
- edge fade(溢れ側のみ、RTL 反転): `mask-image: linear-gradient(to right, transparent 0%, rgba(0,0,0,0.3) 2px, black 4px)`(開始側)。終了側は `to left`、両方なら `transparent 0 → 0.3 at 2px → black 4px … black calc(100% - 4px) → 0.3 at calc(100% - 2px) → transparent 100%`。**4px だけのごく細いフェード**。loop + overflow では常に両側。
- 前後ボタン: pill = 背景 `--color-background-popover`(`#ffffff` / `#1b1b1b`)、`border-radius: --radius-full`、`box-shadow: --shadow-med`(light `0 2px 4px rgba(0,0,0,.05), 0 4px 12px rgba(0,0,0,.10)` / dark `0 2px 4px rgba(0,0,0,.35), 0 4px 12px rgba(0,0,0,.50), inset 0 0 0 1px rgba(255,255,255,.12)`)。中身は ghost **sm 28×28px** icon-only、radius full(`--_button-radius` 上書き)、icon `chevronLeft` / `chevronRight` **xsm 12px**(RTL ミラー)、色 text primary。pill は root の左端 / 右端に **半分はみ出す**(`translateX(∓50%)`、RTL 反転)、上下中央。
- スクロールできない側の pill は `opacity 0; pointer-events none`(ボタンは disabled)。

```
      ╭──╮                                         ╭──╮
     (‹)│[slide 1] [slide 2] [slide 3] [slide 4]│(›)     ← pill 28px が端から半分はみ出す、shadow med
      ╰──╯←4px fade                     4px fade→╰──╯
```

### 25.5 Motion
- pill の表示/非表示: `opacity 125ms(--duration-fast) cubic-bezier(0.24, 1, 0.4, 1)`。
- ボタンによるスクロール: `scrollBy({left: ±max(clientWidth - firstItemWidth*0.5, firstItemWidth), behavior: smooth})`(ブラウザの smooth 曲線。値は UNRESOLVED: ブラウザ依存)。reduced motion は即時(`auto`)。loop で端なら `scrollBy(∓scrollWidth)` で反対端へ smooth に戻る。
- scrollTo(i): 項目の開始端を viewport 開始端に合わせる smooth scroll。
- edge fade の切替は即時(transition 指定なし)。
- Kivact: `NodeRef::scroll_by(delta, ScrollBehavior::Smooth(kivact::motion::Easing::EaseOut))`(crates/kivact-core/src/view/node_ref.rs:234)。kivact の smooth 曲線は engine 既定(UNRESOLVED: 時間)。snap は kivact の scroll に snap 機能が無ければ、ホイール/ドラッグ終了時に最寄り項目開始位置へ `scroll_to(.., Smooth)`(UNRESOLVED: scroll end イベントの有無)。

### 25.6 Behaviour & a11y
- root `role=region` + `aria-roledescription="carousel"` + 名前。scroller `role=group` + 同じ名前、**溢れているときだけ tabindex 0**(矢印キーでネイティブ横スクロール)。各項目 `role=group` + `aria-roledescription="slide"` + "Slide {n} of {total}"。
- Shift + 縦ホイール → 横スクロールに変換(溢れているときのみ、preventDefault)(Carousel.tsx:396-411)。
- 前後ボタンは tab 順に入る(名前 "Scroll left" / "Scroll right")。押したボタンがスクロール端に達して disabled になったら、フォーカスを scroller に移す(フォーカス消失を防ぐ: Carousel.tsx:524-566)。
- 自動送りなし、ページ indicator なし(必要なら Pagination dots を外に置く)。
- 既定テキスト(提案キー):
  - `soft.carousel.label` = "Carousel"
  - `soft.carousel.scroll_left` = "Scroll left"、`soft.carousel.scroll_right` = "Scroll right"
  - `soft.carousel.slide` = "Slide {current} of {total}"
  - roledescription の "carousel" / "slide" は Astryx でも訳さない固定文字列(kivact に roledescription 相当が無ければ省略。UNRESOLVED)。

### 25.7 Kivact 実装方針
- `soft::carousel(name, slides)`: `stack((scroll(row(items)).axes(Horizontal), buttons_overlay))`(kivact の `stack` = 重ね合わせ、最初の子がサイズを決める: crates/kivact-core/src/control/container.rs:177)。scrollbar 非表示は `scrollbar_width(ScrollbarWidth::None)`(sheet.rs:458)。
- 溢れ・端判定: 11 の `soft::use_scroll_overflow(scroller_ref)`。tab stop は `.focusable(move || overflow.any.get())`。
- edge fade: `mask(linear_gradient(..))`(sheet.rs:419-422)を overflow 状態に応じて切替。
- ボタン: `button(..).variant(Ghost).size(ControlSize::Small).icon(icon("chevron-left")).hide_label(true)` を円形 pill(popover 面 + shadow med)で包み、`translate_x(pct(-50))` で外へ。root の `overflow(Hidden)` で pill が切れないよう、root ではなく **外側の親に余白が要る**(Astryx も root は clip だが pill は layer(top layer)なので切れない)→ kivact では pill を root の兄弟の overlay(`kivact::view::overlay(anchor, ..)`)に出すか、root を clip しない(推奨: overlay)。
- Shift+ホイール: scroller にホイール handler(UNRESOLVED: kivact でホイールイベントを横スクロールに変換する API)。
- basic の carousel.rs のメッセージキー(`basic.carousel.*`)は流用せず soft キーを使う。

### 25.8 受け入れ確認
- [ ] region(roledescription carousel)名 = 指定ラベル、各項目 "Slide 2 of 6"。
- [ ] 溢れていると scroller が Tab で止まり ←→ でスクロール、収まると止まらない。
- [ ] 右端に半分はみ出した 28px の丸ボタン(白地 / dark `#1b1b1b`、shadow med)、押すと「1 画面 − 半項目」分 smooth に送る。左端到達で ‹ が 125ms でフェードアウト。
- [ ] 溢れている側だけ 4px の細いフェード。
- [ ] hasLoop で末尾から › を押すと先頭へ戻る。hasSnap で項目の開始端に止まる。
- [ ] RTL でボタンと矢印が反転。


## 26. Collapsible / CollapsibleGroup / useCollapsible

docs: https://astryx.atmeta.com/components/Collapsible (CollapsibleGroup は同ページ、useCollapsible は hook)。source: packages/core/src/Collapsible (Collapsible.tsx, CollapsibleGroup.tsx, CollapsibleGroupContext.tsx, useCollapsible.ts, docs, Collapsible.test.tsx, CollapsibleGroup.test.tsx, useCollapsible.test.tsx, __tests__)。

**判定: RESTYLE `kivact::ui::disclosure` + kit wrapper。** 標準 disclosure(crates/kivact-ui/src/disclosure.rs: trigger part が expanded / controls を公開、Enter・Space・AT の Expand/Collapse、`keep_mounted`、accordion は `Binding<Vec<K>>` の列)を soft の recipe(大きめ見出し trigger + chevron)で描き、kit の `soft::collapsible_group` が single / multiple・dividers・density・chevron 位置を context で配る。basic の対応: crates/kivact-ui-basic/src/disclosure.rs(`basic::collapsible`: 見た目なしの trigger attach 版)。

### 26.1 概要
- trigger(常に見える)と開閉する content。trigger には意味のある名前を(独自 trigger は VisuallyHidden テキストを足す)。
- FAQ 型のフラットな列は CollapsibleGroup の `hasDividers`(自前の線を引かない)。accordion を Card で区切る場合は dividers と併用しない。
- 1 つだけ開くなら `type="single"`(設定・FAQ)、比較させるなら `"multiple"`。初見で必要な内容は開いた状態で始める(**Collapsible の既定は open = true**)。
- 必須の内容を隠さない、3 階層以上入れ子にしない、短い 1 段落には使わない(Collapsible.doc.mjs bestPractices)。

### 26.2 Anatomy
```
[CollapsibleGroup wrapper (div, flex column)]   ← hasDividers のときだけ描画   .astryx-collapsible-group
└─ Collapsible root (div, width 100%, divided: 上辺 1px(先頭以外))            .astryx-collapsible
    ├─ trigger (button aria-expanded aria-controls)  (flex, space-between, full width)   .astryx-collapsible-trigger
    │   ├─ [chevron start ›]  (chevronPosition=start)
    │   ├─ label span (flex-grow 1, cap/alphabetic で leading を trim)
    │   └─ [chevron end ▾]    (既定)
    └─ content (div id, padding-top 4, display none when closed)                        .astryx-collapsible-content
        └─ children
```

### 26.3 API

Collapsible(Collapsible.tsx:228-304):

| prop | type | default | 意味 | Kivact(`soft::collapsible(trigger, open)`) |
|---|---|---|---|---|
| trigger | ReactNode(必須) | — | 常に見える見出し | 第 1 引数 `impl IntoText`(または `.trigger_view(|| ..)`) |
| children | ReactNode | なし | 開閉内容 | `.content(|| ..)`(標準 disclosure と同じく開くたび新 branch。`keep_mounted(true)` 可) |
| defaultIsOpen | boolean | **true** | 非制御の初期 | 第 2 引数 `open: impl Into<Binding<bool>>`(caller-owned。kit は `soft::collapsible_default(trigger, true)` のような非制御版も用意) |
| isOpen / onOpenChange | boolean / fn | — | 制御 | binding |
| isDisabled | boolean | false | trigger を無効(dim、aria-disabled、tab 順から外す)。開いている内容は閉じない | `.disabled(..)` |
| chevronPosition | `'start' \| 'end'` | `'end'`(group の値を継承) | chevron の位置 | `.chevron(ChevronPosition::Start \| End)` |
| value | string | なし | group 内の識別子(group が状態を持つ) | `.value(K)` |
| animatedLayout | AnimatedLayout | なし | useAnimatedLayout のハンドル。与えると開閉で後続内容が滑る | `.animate_layout(true)`(Kivact では常時アニメーション可にしてよい: 下記) |

CollapsibleGroup(CollapsibleGroup.tsx:55-133):

| prop | type | default | 意味 | Kivact(`soft::collapsible_group(open_keys, items)`) |
|---|---|---|---|---|
| type | `'single' \| 'multiple'` | `'single'` | 同時に開ける数 | `.mode(GroupMode::Single \| Multiple)` |
| defaultValue / value / onChange | string \| string[] | [] | 開いている項目 | `open_keys: impl Into<Binding<Vec<K>>>`(single は長さ 0/1) |
| hasDividers | boolean | false | 項目間に hairline、wrapper を描く、density 既定 balanced | `.dividers(bool)` |
| density | `'compact'\|'balanced'\|'spacious'` | dividers 時 balanced、他は無し(padding 0) | trigger / content の上下 padding | `.density(..)` |
| chevronPosition | `'start' \| 'end'` | `'end'` | 子の既定 | `.chevron(..)` |

useCollapsible({isCollapsible: boolean | {defaultIsOpen?, isOpen?, onOpenChange?}, value?}) → `{isEnabled, isOpen, toggle}`。group 内で value があれば group が状態を持つ。初期 open は true。Kivact: `soft::use_collapsible(config) -> CollapsibleState`(他コンポーネント(Table 行など)が「開閉可能かつ group 連携」を得るための helper)。

single の挙動: 開いている項目を押すと **閉じて 0 個** になれる(`openValues.includes(v) ? [] : [v]`: CollapsibleGroup.tsx:214-216)。onChange は single で `string`(空なら `''`)、multiple で `string[]`。

### 26.4 Visual spec(Collapsible.tsx:80-226)
- trigger: `box-sizing:border-box; display:flex; align-items:center; justify-content:space-between; width:100%`、背景透明・枠なし(reset)、`font-family body; font-size --text-large-size 17px; font-weight 600`; color `--color-text-primary`(`#000000` / `#ffffff`)、text-align start、`padding-block: 0`(density で上書き)、padding-inline 0。line-height は継承(label は `text-box-trim: trim-both; text-box-edge: cap alphabetic` で上下の leading を削る → 行の見た目の高さ ≈ cap height)。
  - density(group から): compact `padding-block 4px` / balanced 8px / spacious 12px。group 外は 0。
  - **hover 背景なし**。押下中だけ `--color-overlay-pressed`(light `#0000001A` / dark `#FFFFFF1A`)(`pressedBackgroundColor`: 0.9。radius は無いので矩形)。
  - disabled: `opacity 0.5`, cursor default, tabindex −1。
  - focus-visible: 標準リング(2px accent `#1b1b1b` / `#f1f1f1`、offset 3px、角丸なし)。
- chevron: Icon size sm 指定だが箱は `--text-large-size` = **17×17px**(font-size 17px なのでグリフも 17px)、色 `--color-icon-secondary`(light `#6a6a6a` / dark `#9e9e9e`)、shrink 0。
  - end(既定): `chevronDown`(Lucide `chevron-down`)、closed `rotate(0)`、open `rotate(180deg)`(上向き)。
  - start: `chevronRight`、`margin-inline-end: 8px`、closed `rotate(0)`(右 = 内容側を指す)、open `rotate(90deg)`(下向き)。RTL は `scaleX(-1)` を前置(閉で左向き)。
- content: `padding-top: --spacing-1` 4px、`14px / 400 / 20px`(`--text-body-*`)、`--color-text-primary`。density で `padding-bottom` 4 / 8 / 12px。閉じると `display:none`(DOM には残る。hidden="until-found" ではない: 検索でヒットしない)。
- divided(group hasDividers): 各項目に `border-top: 1px solid --color-border`(light `#00000014` / dark `#FFFFFF1A`)、**先頭項目は 0**。外周(上下端)の線は無い。
- テーマ target: `astryx-collapsible`(density, divided)、`astryx-collapsible-trigger`(density, chevronPosition, open, disabled)、`astryx-collapsible-content`(density, open)、`astryx-collapsible-group`(density)。

```
dividers + balanced
 Shipping                                   ▾   ← 17px / 600, padding-block 8, chevron 17px secondary
 ──────────────────────────────────────────────  1px border-top (2 項目目以降)
 Returns                                    ▴   ← open: chevron 180deg
   We accept returns within 30 days…            ← content 14/20, padding 4 top / 8 bottom
 ──────────────────────────────────────────────
 Warranty                                   ▾
```

### 26.5 Motion
- chevron: `transform 125ms(--duration-fast) cubic-bezier(0.24, 1, 0.4, 1)`(0 ↔ 180deg / 0 ↔ 90deg)。
- 内容の開閉: **既定はアニメーション無し**(display none ↔ block で即時。後続の内容も即座に詰まる)。`animatedLayout` を渡したときだけ、useAnimatedLayout が開閉前後の位置を測って後続要素を 0.11 の settle spring(stiffness 157.91, damping 25.13, ≈735ms)で滑らせ、閉じる内容は閉じ終わるまで保持(holdLayoutExit)し、chevron も同じ motion に乗る(Collapsible.tsx:376-458)。reduced motion: fine は即時、coarse は ≤200ms のクロスフェード。
- 押下 overlay: タッチの release フェード 200ms(0.9)。
- Kivact: 推奨は「soft の collapsible は常に高さアニメーション」= content を `overflow(Hidden)` の箱に入れ、高さを `animated(target_h, Transition::spring(Spring{stiffness:157.91, damping:25.13, mass:1.0}))` で 0 ↔ 内容高さ(`NodeRef::rect()` の計測値)に動かす。閉じ切ったら unmount(または `keep_mounted`)。Astryx 既定(即時)と合わせたい場合に備え `.animate_layout(false)` で即時にできるようにする(既定値は UNRESOLVED: 既定を即時にするか spring にするか、docsite の Collapsible 既定デモは即時)。

### 26.6 Behaviour & a11y
- trigger は button、`aria-expanded`、`aria-controls` = content id、disabled は `aria-disabled` + tabindex −1。Enter / Space / クリックで toggle(WAI-ARIA Disclosure)。group は roving などのキー操作を **追加しない**(各 trigger が独立した tab stop)。
- 閉じるときに content 内にフォーカスがあれば trigger へ移す(useFocusLeavingChange / moveFocusLeftBehind: Collapsible.tsx:465-482)。
- content は閉じている間 `display:none` なので AT からも隠れる。
- 既定テキストなし。

### 26.7 Kivact 実装方針
- `soft::collapsible(trigger, open)` = 標準 `kivact::ui::disclosure(title, open).content(..)` を soft の `DisclosureRecipes`(root / trigger / indicator / content の 4 recipe: crates/kivact-ui/src/disclosure.rs:1-8)で描く。indicator recipe が chevron(17px、125ms 回転、start/end)。trigger の label に任意 View を入れたい場合は kit wrapper 側で `kivact::interaction::use_disclosure` を直接使う。
- group: `provide(CollapsibleGroupCx{ mode, density, dividers, chevron })`、各項目の `open` binding を `open_keys` から作る(標準 disclosure の accordion 例と同じ: disclosure.rs:42-60)。single は「自分のキーだけ / 空」に置き換える。
- dividers の「先頭以外に上線」は group が index を知っているので builder 時に決める。
- leading-trim(cap/alphabetic)は kivact に無い → trigger の line-height を 1(17px)にして近似(UNRESOLVED: 見た目の差は数 px)。
- 高さアニメーションは上記 Motion の案。

### 26.8 受け入れ確認
- [ ] 既定で開いている(defaultIsOpen true)。trigger は button(expanded=true, controls)。
- [ ] trigger 17px/600 primary、右端に 17px の ▾(`#6a6a6a` / dark `#9e9e9e`)、押すと 125ms で 180deg 回って ▴、内容が消える。
- [ ] chevron start で › が左(8px 間隔)、開くと 90deg で下向き。RTL で ‹。
- [ ] group single で 1 つ開くと他が閉じる、開いている項目を押すと全部閉じる。multiple で複数開ける。
- [ ] hasDividers で 2 項目目以降の上に 1px 線、trigger 上下 8px・content 下 8px。
- [ ] disabled trigger は半透明、Tab で止まらない、開いた内容は残る。
- [ ] 閉じたとき content 内のフォーカスが trigger に移る。


## 27. useContainerReveal

docs: https://astryx.atmeta.com/components/useContainerReveal(hook ページ; URL は UNRESOLVED、hooks は /hooks/ 配下の可能性)。source: packages/core/src/hooks/useContainerReveal.ts, packages/core/src/hooks/containerReveal.stylex.ts。利用例: Thumbnail(削除ボタン)、TreeListItem、Markdown の見出しリンク。

**判定: kit の style ヘルパ(hook 相当)。** kivact の `style().group()` + `group_hover` + `group_focus`(crates/kivact-style/src/sheet.rs:767-812)でほぼ表現できる。

### 27.1 概要
- 「行にホバー/キーボードフォーカスが入ったら行内のアクションが現れる」パターンの headless 実装。React に hover 状態を持たず CSS だけで完結。入れ子の container は祖先を shadow する(内側の hover が外側に漏れない)。
- 隠れている間も display:none にしない(opacity + position)のでマウントされたまま、Tab で到達でき、フォーカスで現れる。
- 二次的なアフォーダンス専用。常に見えるべき必須操作を隠さない。長いリストの行には hoverDelay 100–250ms(カーソル通過で全行が光らないように)(useContainerReveal.doc.mjs bestPractices)。

### 27.2 Anatomy
```
container  (getContainerProps)      ← :hover / :focus-within / coarse pointer がトリガ
└─ content (getContentRevealProps)  ← reveal(既定) または conceal(isRevealInverted)
```

### 27.3 API(useContainerReveal.ts:21-102)
- `useContainerReveal({ isEnabled = true })` → `{ getContainerProps(opts), getContentRevealProps(opts) }`。isEnabled=false で全部空(常時表示)。
- `ContainerRevealOptions`: `hoverDelay?: number`(ms, 既定 0。ホバー開始から reveal 開始までの待ち)、`forceState?: 'active' | 'inactive'`(呼び出し側がトリガ状態を固定。inactive でもキーボードフォーカスと coarse pointer では現れる)。
- `ContentRevealOptions`: `isRevealInverted?`(hover で隠す)、`isLayoutPreserved?`(隠れている間も箱を確保し opacity だけ変える)、`forceVisibility?: 'shown' | 'hidden'`(この要素だけ固定。hidden でもフォーカスで現れる)。

Kivact 提案:
```rust
pub struct Reveal { delay_ms: f32, force: Option<RevealForce> }      // container 側
pub fn reveal_container(style: Style) -> Style;                       // style.group()
pub fn reveal_content(opts: RevealContent) -> impl Fn() -> Style;     // opacity/position + group_hover/group_focus
pub enum RevealForce { Active, Inactive }
pub struct RevealContent { inverted: bool, preserve_layout: bool, force: Option<Visibility> }
```

### 27.4 Visual spec(containerReveal.stylex.ts:30-269)
状態ごとの content の値:

| 状態 | reveal (既定) | reveal + layoutPreserved | conceal (inverted) |
|---|---|---|---|
| 休止 | opacity 0, `position:absolute`(箱を取らない) | opacity 0, 箱は確保 | opacity 1 |
| container hover(hover 可能端末, 非 disabled) | opacity 1, `position:static` | opacity 1 | opacity 0 |
| container :focus-within | opacity 1, static | opacity 1 | (変化なし = 1) |
| `any-pointer: coarse` 端末 | 常に opacity 1, static | 常に 1 | (1) |

- forceState active: 常に表示(reveal opacity 1/static、conceal opacity 0)。inactive: hover では出ないが focus-within と coarse では出る(stylex.ts:99-197)。
- forceVisibility shown: opacity 1, static, delay 0。hidden: opacity 0(absolute)、ただし自身の `:focus-within` で 1/static(stylex.ts:250-268)。
- disabled 判定: container が `:disabled` または `aria-disabled="true"` なら hover トリガ無効(stylex.ts:35)。

### 27.5 Motion
- reveal: `transition-property: opacity, position; duration: --duration-fast(neutral 125ms), 0s; easing: --ease-standard = cubic-bezier(0.24, 1, 0.4, 1); transition-behavior: allow-discrete`(stylex.ts:201-216)。
- delay(`--_reveal-delay`): 休止に戻るとき(hover 解除) = `0s, 125ms`(opacity は即フェード開始、position は 125ms 後に absolute へ = フェードアウトが終わってから箱を消す: REST_DELAY, stylex.ts:23)。hover 進入時 = `hoverDelay, hoverDelay`(既定 0)。focus-within / coarse = `0s, 0s`。
- layoutPreserved / conceal: opacity のみ 125ms、`--ease-standard`、delay = hover 進入時 hoverDelay、それ以外 0(stylex.ts:217-246)。
- reduced motion: duration 0s。ただし hoverDelay は維持(`--_reveal-delay-reduced` は hover 時だけ HOVER_DELAY: stylex.ts:64-71)。

### 27.6 Behaviour & a11y
- 隠れた content もフォーカス可能・a11y tree に残る(opacity のみ)。フォーカスが入ると即表示。
- キーマップなし。既定テキストなし。

### 27.7 Kivact 実装方針
- container: `style().group()`。content: `style().opacity(0.0).group_hover(style().opacity(1.0)).group_focus(style().opacity(1.0)).transition(TransitionProp::Paint, Transition::tween(125, Easing::CubicBezier(0.24, 1.0, 0.4, 1.0)))`。
- `position: absolute` ↔ `static` の切替は layout 変化で state style に置けない(state style は paint のみ: sheet.rs:760-766)。→ container の `NodeRef::hovered()` / `focus_within()`(crates/kivact-core/src/view/node_ref.rs:495, 541)を読む reactive style で `position` を切替え、hover 解除時は 125ms 遅らせる(`kivact::interaction::use_delayed` か `animated` の on_complete)。`preserve_layout` 版は position を触らないので `group_hover` だけで済む(推奨: kit 内部の利用はまず layoutPreserved で作る)。
- hoverDelay: `group_hover` には遅延が無いので、delay>0 のときは `hovered()` + `use_delayed`(crates/kivact-interaction/src/delayed.rs)で「hover が delay ms 続いたら active」の signal を作り、reactive style で opacity を切替える。
- coarse pointer 端末で常時表示: `kivact::platform::capabilities().touch` が true なら常に opacity 1 / static(crates/kivact-core/src/platform/env.rs:122-139)。
- 入れ子の shadow: kivact の `group_hover` は「最も近い group 祖先」を見る(sheet.rs:785-790)ので自然に一致。

### 27.8 受け入れ確認
- [ ] 行にマウスを乗せるとアクションが 125ms でフェードイン、外すとフェードアウト後に箱が消える(レイアウトが詰まる)。
- [ ] Tab で隠れたアクションにフォーカスすると即座に見える。
- [ ] `hover_delay(200)` の行をカーソルで素早く横切っても表示されない。
- [ ] reduced motion でフェード無し(即時)、hover delay は維持。


## 28. useMediaQuery(レスポンシブ・ブレークポイント)

docs: https://astryx.atmeta.com/components/useMediaQuery(hook; URL は UNRESOLVED)。source: packages/core/src/hooks/useMediaQuery.ts, packages/core/src/theme/themeAdaptations.ts:36-100。

**判定: kit の env ヘルパ(コンポーネントではない)。**

### 28.1 概要
- 任意の media query 文字列を購読して boolean を返す hook(`window.matchMedia` + `useSyncExternalStore`, useMediaQuery.ts:35-52)。SSR では `serverDefault`(既定 false)。
- Astryx コンポーネントは名前付きブレークポイント(テーマ解決)を使う。AppShell の mobile 判定、Neutral の型スケール適応などが利用者。

### 28.2 ブレークポイント一覧(すべて CSS px)

| 名前 / 用途 | 値 | 出典 |
|---|---|---|
| `sm` | 640 | themeAdaptations.ts:50-57 `DEFAULT_WIDTH_BREAKPOINTS` |
| `md` | 768 | 同上。**AppShell の mobile 既定**(`mobileNav.breakpoint ?? 'md'`: AppShell.tsx:599)。compact 判定の幅 |
| `lg` | 1024 | 同上。Neutral の型スケール適応「pointer coarse かつ width < lg → base 16px」 |
| `xl` | 1280 | 同上 |
| `2xl` | 1536 | 同上 |
| compact | `width < md(768)` **または** `height < 500` | themeAdaptations.ts:59-98 `compactDefinition`(画面サイズで提示形式を変えるコンポーネント用) |
| compact + coarse | 上記 and `(pointer: coarse)` | 同上 `coarsePointerQuery` |
| FormLayout horizontal-labels 崩し | `max-width: 480px` | FormLayout.tsx:37 |
| hover 可能 | `(hover: hover)` | nav item / card の hover 背景はすべてこの中 |
| coarse pointer | `(pointer: coarse)` / `(any-pointer: coarse)` | 押下 overlay、ContainerReveal 常時表示 |
| reduced motion | `(prefers-reduced-motion: reduce)` | 全 transition を 0s |
| forced colors | `(forced-colors: active)` | 枠・outline の代替 |

範囲の意味: `from: X` は「X 以上」、`below: X` は「X 未満」(themeAdaptations.ts:100-106)。AppShell の breakpoint 名は `'sm' | 'md' | 'lg' | 'xl' | '2xl' | 'none'`(none = 常に非 mobile)。テーマは width 点を上書きでき、コンポーネントは最寄り Theme の値で解決する(AppShell.tsx:135-136)。

### 28.3 API
`useMediaQuery(query: string, serverDefault = false): boolean`。

Kivact 提案(`soft::env`):
```rust
pub enum Breakpoint { Sm, Md, Lg, Xl, Xxl }           // 640/768/1024/1280/1536 (soft::Tokens.breakpoints から)
pub fn viewport_below(bp: Breakpoint) -> Memo<bool>;  // window_size().width < bp
pub fn viewport_from(bp: Breakpoint) -> Memo<bool>;
pub fn is_compact() -> Memo<bool>;                    // width < md || height < 500
pub fn container_below(node: NodeRef, px: f32) -> Memo<bool>; // コンテナ幅版(@container の代替)
```
任意の media 文字列パースは提供しない(Rust 側で条件を書く)。

### 28.4 Visual spec
該当なし(値の解決のみ)。

### 28.5 Motion
なし(ブレークポイント跨ぎで AppShell は MobileNav を出し入れする。その動きは 1 / 17 を参照)。

### 28.6 Behaviour & a11y
- ウィンドウのリサイズで即時に再評価。SSR の概念は無いので serverDefault は不要。
- `(hover: hover)` 相当: kivact はマウスポインタ由来の hover しか立てない前提で、条件分岐不要。
- `(pointer: coarse)`: `kivact::platform::capabilities().touch`(env.rs:122-139, App の生存中固定・非 tracked)で近似する。touch が true の環境を coarse とみなす。
- `prefers-reduced-motion`: `kivact::platform::reduced_motion()`(crates/kivact-core/src/platform/env.rs:54)。

### 28.7 Kivact 実装方針
- `kivact::platform::window_size()`(crates/kivact-core/src/platform/env.rs:100-121)は tracked read(リサイズと scale 変更に追従)なので `memo(move || window_size().width < 768.0)` でよい。doc コメント自体が `@media (max-width: ..)` の代替として例示している。
- コンテナクエリ(Layout の `100cqi` 等)は `NodeRef::rect()`(node_ref.rs:416)で代替。
- ブレークポイント値は `soft::Tokens` に入れ、テーマで上書き可能にする。

### 28.8 受け入れ確認
- [ ] ウィンドウ幅 767px で `viewport_below(Md)` が true、768px で false。
- [ ] 高さ 499px で `is_compact()` が true(幅が広くても)。
- [ ] リサイズ中に AppShell が md を跨いで mobile/desktop を切替える。

