# 作業指示書: `kivact-ui-soft`（Astryx の全コンポーネントを Kivact の公式 UI キットとして実装する）

- 発行: 2026-11-03（Claude Code が調査・作成）
- 実装担当: Codex（主エージェント）と、Codex が起動するサブエージェント
- ブランチ: `feat/kashiwade/add-soft-ui-component`
- 参照する Astryx の clone: `D:\w\meta\astryx`（commit `322c0d6e66`）
- 公開サイト: <https://astryx.atmeta.com/components>

この文書は**実装の順序・方針・決定済み事項・完了条件**を定める本体である。各コンポーネントの寸法・色・状態・アニメーションの数値は付録 `soft-ui-work-order/` にある。実装中は本書と付録の両方を読むこと。

---

## 目次

0. [この文書の読み方](#0-この文書の読み方)
1. [ゴール](#1-ゴール)
2. [範囲](#2-範囲)
3. [守る規則（Kivact の制約）](#3-守る規則kivact-の制約)
4. [決定済み事項](#4-決定済み事項)
5. [見た目の核心（Neutral テーマ）](#5-見た目の核心neutral-テーマ)
6. [モーションの核心](#6-モーションの核心)
7. [中核（kivact 本体）への追加](#7-中核kivact-本体への追加)
8. [作業計画とサブエージェントの分担](#8-作業計画とサブエージェントの分担)
9. [コンポーネント別の作業一覧](#9-コンポーネント別の作業一覧)
10. [見た目とアニメーションの調整手順](#10-見た目とアニメーションの調整手順)
11. [試験](#11-試験)
12. [カタログ・README・利用者ドキュメント](#12-カタログreadme利用者ドキュメント)
13. [完了条件](#13-完了条件)
14. [付録の一覧](#14-付録の一覧)

---

## 0. この文書の読み方

### 0.1 文書の構成

| 文書 | 内容 | いつ読むか |
|---|---|---|
| 本書 `SOFT_UI_WORK_ORDER.md` | 方針・決定事項・順序・完了条件 | 最初に全部。各フェーズの開始時にも該当の章を読む |
| `soft-ui-work-order/foundation/kivact-kit-architecture.md` | Kivact の公式キットの作り方。crate の骨格、テーマと recipe の API、標準部品の一覧、style とアニメーションでできること・できないこと、試験、カタログ、登録が必要なファイル | フェーズ 0 の前に全部。recipe を書く前に 2 章をもう一度 |
| `soft-ui-work-order/foundation/tokens.md` | Astryx の Neutral テーマの全トークン（色・文字・余白・角丸・寸法・影・フォーカス・状態・icon・motion の値）を light と dark で解決した値。13 章は `Tokens` 構造体の案 | `tokens.rs` を書くとき。部品で値に迷ったとき |
| `soft-ui-work-order/foundation/motion.md` | Astryx のモーションの全抽出。motion role、spring の式と定数、gesture の閾値、reduced motion、部品ごとのアニメーション、Kivact での実現方法と不足 | アニメーションを付けるとき |
| `soft-ui-work-order/components/01-action-feedback.md` 〜 `06-table-list-chat.md` | 部品ごとの詳細仕様。各部品に「概要・Anatomy・API・見た目・モーション・振る舞いと a11y・Kivact 実装方針・受け入れ確認」の 8 項目 | その部品を実装するサブエージェントが、担当の節を全部 |

### 0.2 何を正とするか

1. **Kivact の API と内部構造は、コードと rustdoc が正。** 付録の `path:line` は 2026-11-03 時点の値で、ずれていればコードを読み直す。
2. **見た目と振る舞いは、Astryx のソース（`D:\w\meta\astryx\packages\core\src`、`packages\themes\neutral`、`packages\richtext`）が正。** 付録は AI が抽出したもので、誤りがありうる。値に疑いがあれば、付録が示す `file:line` を開いて確かめ、**付録を同じ commit で直す**。付録の値をコードに写すときに、もっともらしい値へ勝手に丸めない。
3. 付録どうし、または付録と本書が矛盾したら、本書 → components 付録 → foundation 付録の順に優先する。ただし数値は 2. のとおり Astryx のソースで確かめる。
4. `UNRESOLVED` と書かれた項目は、本書 4 章・7 章の決定に従う。どこにも決定がない UNRESOLVED は、Astryx のソースを読んで決め、決めた内容と理由を付録の該当箇所に書き足す。それでも決められないものだけ作者（Kashiwade）に質問する。

### 0.3 Astryx の clone の扱い

- `D:\w\meta\astryx` は**読むだけ**にする。そこで `pnpm install`・`node`・build・test を実行しない（第三者のコードを実行することになるため）。参照用のスクリーンショットが必要になったら、作者に頼む（10.4 節）。
- Astryx のコードを Kivact へ**コピーしない**。Astryx は MIT だが、Kivact は TypeScript を写すのではなく、同じ見た目と振る舞いを Kivact の公開 API で作り直す。数値・配色・挙動の仕様を移すのはよい。

---

## 1. ゴール

`crates/kivact-ui-soft` という五つ目の公式 UI キットを作る。

1. **Astryx の公開サイトの `/components` にあるすべてのコンポーネント**（2 章の表）を、Kivact の公開 API だけで実装する。`kivact-ui-basic` が shadcn/ui の全コンポーネントを実装したのと同じやり方を取る:
   - Kivact の標準部品（`kivact::ui`）と意味が同じものは、**テーマの recipe で見た目を差し替える**。標準部品のコピーは作らない。
   - 標準部品にないものは、キットの部品として作る。振る舞いは `kivact::interaction` から組む。
   - 二つ以上のキットに共通する意味で、中核に口が足りないものは**中核に足す**（7 章）。
2. **見た目は Astryx の Neutral テーマ**（docsite のプレビューが使っているテーマ）を light と dark の両方で再現する。寸法・色・角丸・影・文字・余白・状態の見た目を、付録の値どおりにする。
3. **アニメーションも再現する。** トランジション・入退場・spring・gesture（drag、swipe、flick）・ループ（spinner、skeleton、progress）・reduced motion の挙動を含む。
4. 試験、カタログ、README、利用者ドキュメント（英語と日本語）、ワークスペースへの登録まで含めて、merge 前の検査（13 章）を通す。

名前の由来: `soft` は Astryx の Neutral テーマの柔らかい印象（無彩色、薄い境界線、影の弱い面、大きめの角丸、前のめりの ease-out）を表す。

---

## 2. 範囲

### 2.1 対象: 公開サイト `/components` のサイドバーにあるもの全部

公開サイトに載っているのは `@astryxdesign/core` の全部品と `@astryxdesign/richtext` の Rich Text Editor である。下の表の**すべての行**が対象で、子部品（`…Item`、`…Header` など）と hook は親部品の機能として実装する。「付録」の列はその部品の詳細がある付録のファイル。

| サイトのグループ | 部品・子部品・hook | 付録 |
|---|---|---|
| App Shell | App Shell, useAppShellMobile | 04 |
| Aspect Ratio | Aspect Ratio | 04 |
| Avatar | Avatar, Avatar Group, Avatar Group Overflow, Avatar Status Dot | 05 |
| Badge / Banner / Blockquote | Badge, Banner | 01 |
|  | Blockquote | 05 |
| Bottom Sheet | Bottom Sheet, Bottom Sheet Switcher | 03 |
| Breadcrumbs | Breadcrumbs, Breadcrumb Item | 04 |
| Button | Button, Button Group, Icon Button, Toggle Button, Toggle Button Group | 01 |
| Calendar | Calendar | 02 |
| Card | Card, Clickable Card, Selectable Card | 04 |
| Carousel | Carousel | 04 |
| Chat | Chat Composer, Chat Composer Drawer, Chat Composer Input, Chat Composer Token Element, Chat Dictation Button, Chat Layout, Chat Layout Scroll Button, Chat Message, Chat Message Bubble, Chat Message List, Chat Message Metadata, Chat Send Button, Chat System Message, Chat Tokenized Text, Chat Tool Calls | 06 |
| Checkbox | Checkbox Input, Checkbox List, Checkbox List Item | 02 |
| Citation / Code / Code Block | Citation, Code, Code Block | 05 |
| Collapsible | Collapsible, Collapsible Group, useCollapsible | 04 |
| Command Palette | Command Palette, …Empty, …Footer, …Group, …Input, …Item, …List | 03 |
| Context Menu | Context Menu, Context Menu Item | 03 |
| Date Input | Date Input, Date Range Input, Date Time Input | 02 |
| Dialog | Dialog, Dialog Header, Alert Dialog, useImperativeDialog, useImperativeAlertDialog | 03 |
| Divider | Divider | 04 |
| Drawer | Drawer, Drawer Header | 03 |
| Dropdown Menu | Dropdown Menu, …Checkbox Item, …Divider, …Group, …Item, …Radio Group, …Radio Item, …Submenu | 03 |
| Empty State | Empty State | 05 |
| Field | Field, Field Label, Field Status, Input Group, Input Group Text, File Input | 02 |
| Heading | Heading | 05 |
| Hover Card | Hover Card, useHoverCard | 03 |
| Icon | Icon | 05 |
| Indicator | Checkbox Indicator, Check Indicator, Radio Indicator | 01 |
| Item / Kbd | Item | 06 |
|  | Kbd | 01 |
| Layout | Center, Form Layout, Grid, Grid Span, H Stack, Layout, Layout Content, Layout Footer, Layout Header, Layout Panel, Section, Stack, Stack Item, V Stack | 04 |
| Lightbox / Link | Lightbox | 03 |
|  | Link | 01 |
| List | List, List Item | 06 |
| Markdown | Markdown | 05 |
| Metadata List | Metadata List, Metadata List Item | 06 |
| More Menu | More Menu | 01 |
| Navigation | Mobile Nav, Mobile Nav Toggle, Nav Heading Menu, Nav Icon, Side Nav, Side Nav Collapse Button, Side Nav Heading, Side Nav Item, Side Nav Section, Top Nav, Top Nav Heading, Top Nav Item, Top Nav Mega Menu, Top Nav Mega Menu Featured Card, Top Nav Mega Menu Item, Top Nav Menu | 04 |
| Number Input / Outline / Overflow List / Overlay / Pagination | Number Input | 02 |
|  | Outline, Pagination | 04 |
|  | Overflow List | 06 |
|  | Overlay | 03 |
| Popover | Popover, usePopover | 03 |
| Power Search / Progress Bar | Power Search | 02 |
|  | Progress Bar | 01 |
| Radio | Radio List, Radio List Item | 02 |
| Resizable | Resize Handle, useResizable | 04 |
| Rich Text Editor / Scrollable Area | Rich Text Editor | 02 |
|  | Scrollable Area | 04 |
| Segmented Control | Segmented Control, Segmented Control Item | 01 |
| Selector | Selector, Selector Option, Complex Selector, Multi Selector, useCombobox | 02 |
| Skeleton / Slider / Spinner / Status Dot | Skeleton, Spinner, Status Dot | 01 |
|  | Slider | 02 |
| Stepper | Stepper, Step | 04 |
| Switch | Switch | 02 |
| Table | Table, Table Body, Table Cell, Table Footer, Table Header, Table Header Cell, Table Row, Table Selection Toolbar, useTableColumnResize, useTableColumnSettings, useTableFiltering, useTableFilterState, useTableGroupedRows, useTablePagination, useTableRowExpansion, useTableRowIndex, useTableRowStatus, useTableSelection, useTableSelectionState, useTableSortable, useTableStickyColumns, useTableTreeData, useTableTreeState | 06 |
| Tabs | Tab List, Tab, Tab Menu | 04 |
| Text 〜 Timestamp | Text, Thumbnail, Timer, Timestamp | 05 |
|  | Text Area, Text Input, Time Input | 02 |
| Toast | Toast, useToast | 03 |
| Token / Tokenizer / Toolbar | Token, Toolbar | 01 |
|  | Tokenizer | 02 |
| Tooltip | Tooltip, useTooltip | 03 |
| Tree List | Tree List | 06 |
| Typeahead | Typeahead, Typeahead Item, Base Typeahead, useMultiCombobox | 02 |
| VisuallyHidden | VisuallyHidden | 05 |
| Utilities | Internationalization Provider, Layer Provider, Link Provider, Media Theme, Syntax Theme, Theme, use… の hook 群 | 05（Layer は 03、focus/press 系 hook は 01、layout 系 hook は 04） |

**hook の扱い:** Astryx の hook は React の部品を組み立てるための道具である。Kivact では次のどれかになる（各付録に判定がある）:
- 部品の機能になる（例: `useTableSortable` → soft の table の並べ替え）。
- 既存の Kivact の仕組みに対応する（例: `useMediaQuery` → `kivact::platform::window_size()` から計算、`useClipboard` → `kivact::platform::copy_text`）。
- キットの小さな helper になる。
- native では不要（例: `useMergedRefs`、`useDevWarning`）。不要と判定したものは、README の対応表に「不要（理由）」と書く。

### 2.2 範囲外

| 範囲外のもの | 理由 |
|---|---|
| `@astryxdesign/lab`・`@astryxdesign/charts`・`@astryxdesign/vega` の部品（Chart、Comments、DiffViewer、EmojiPicker、InfoTip、Tour、Stat、Schedule、TransferList など） | 公開サイトの `/components` に載っていない canary 専用の部品。作者が後で別に依頼する |
| Neutral 以外のテーマ（matcha、stone、gothic、chocolate、y2k、butter） | 対象は Neutral だけ。`tokens.md` 12 章は将来のテーマ切替の参考として残す |
| Web に固有のもの（`href` による遷移、SSR、`className`/`xstyle`、`asChild` 系、CSS 変数による外部からの上書き） | native では意味がない。対応は付録の API 表にある（`href` → `on_press` など） |
| Astryx の 29 言語の翻訳カタログ | 既定の文言は英語だけ。アプリが `App::messages` で差し替える（4.6 節） |

---

## 3. 守る規則（Kivact の制約）

詳細は `foundation/kivact-kit-architecture.md` の 0 章と `CONTRIBUTING.md` にある。とくに次を守る。

1. **キットの依存は公開 facade の `kivact` だけ**（`default-features = false`。dev-dependency は `features = ["test"]`）。`kivact-core`・`kivact-style`・`kivact-ui`・`kivact-interaction` などの内部 crate に直接依存しない。外部 crate（Markdown のパーサ、syntax highlighter、日付ライブラリなど）にも依存しない。
2. **標準部品をコピーしない。** `kivact::ui` の button、toggle、checkbox、switch、text_input、text_area、number_input、select、menu、dialog、tooltip、tabs、radio_group、slider、range_slider、toast_region、splitter、split、progress、disclosure、list、data_table、command_palette、tag_input、field は、`UiRecipes` の recipe で見た目だけを差し替える。
3. **キットが持つのはテーマと、標準部品にない部品だけ。** 二つ以上の公式キットに共通する意味で、中核に口が足りないものは中核に足す（7 章）。キットで回避策を複製しない。
4. **互換性のための alias・wrapper・旧 API を残さない。** 未公開の初期開発版である。中核の API を変えてよいが、変えたら同じ変更で全キット・例・ドキュメントを新しい形に直す。
5. **既存の四つのキット（basic、pop、creative、game）の部品・機能を減らさない。** 外部の利用者がいる。中核の変更で既存キットの見た目や挙動が変わる場合は、既存キットの recipe を直して元の見た目を保つ。
6. **state style（`.hover(..)`、`.pressed(..)`、`.focus_visible(..)` など）は paint と text の `color` しか変えられない。** layout の property を state style に書かない。text decoration は `state.input.hovered` で分岐する。
7. **transition で補間されるのは paint の一部**（opacity、単色の background、border の色、transform、最初の二つの shadow、filter の量）だけである。gradient、text の色、ring、radius、clip は補間されない。補間が要るときは 6 章と 7 章の方針に従う。
8. **native の text field（text_input、text_area、tag/number/palette の入力）は `Style::paint` を持てない。**
9. **recipe の表は `OnceLock` で一度だけ作る。** 表は `Arc::ptr_eq` で比較されるので、毎回作るとテーマが等しくならない。
10. コード規則: crate root とすべての module の先頭に `//! Role: <英語の 1 文>` を置く。子 module を持つ module には `mod` と `pub use` だけを置く。glob import を使わない（試験と例の `kivact::prelude::*` を除く）。library で `unwrap()` を使わない（`expect("invariant: …")`）。コメントと rustdoc は英語。lint の `allow` には `reason` を書く。ファイルごとの code 行の上限は `cargo xtask check` が検査する。上限に当たったら、役割で葉の module に分ける。行数を減らすためだけの分割や例外はしない。
11. **panic は不具合。** `catch_unwind` で隔離しない。
12. **試験は実ウィンドウの E2E（`kivact::test`）だけ。** golden、画素や時間の厳密な比較、過去の不具合の再現だけの試験を足さない。ウィンドウを開くコマンドは `dev/tools/window-lock.sh -- <command>` を通す。

---

## 4. 決定済み事項

付録の `UNRESOLVED` や「作者の判断」とされた点のうち、ここで決めたもの。付録と食い違ったら、ここに従う。

### 4.1 名前

| 項目 | 決定 |
|---|---|
| crate | `kivact-ui-soft`（`crates/kivact-ui-soft`） |
| ドキュメントでの import | `use kivact_ui_soft as soft;` |
| カタログ | `examples/kivact-ui-soft`、package 名 `kivact-ui-soft-catalog` |
| テーマの入口 | `soft::theme(mode: ThemeMode) -> Theme`。型は `fn(ThemeMode) -> Theme` そのもの（`theme_conformance.rs` が要求する） |
| 文言の key | `soft.<component>.<name>`（例 `soft.banner.dismiss`）。標準部品の key（`dialog.close` など）はそのまま |
| 部品の関数名 | Astryx の名前を snake_case にする（`SegmentedControl` → `segmented_control`、`EmptyState` → `empty_state`、`MetadataList` → `metadata_list`）。標準部品の restyle になるもの（Dialog、Tooltip、Switch など）はキットに関数を作らず、`kivact::ui::…` を使う。対応は README の表に書く |
| 部品の型名 | Astryx の名前（`SegmentedControl`、`EmptyState`）。variant の enum は `<Component><Axis>`（`BadgeColor`、`BannerStatus`、`AvatarSize`） |

### 4.2 Astryx の概念 → Kivact の概念

| Astryx | Kivact | 補足 |
|---|---|---|
| `variant="primary" / "secondary" / "ghost" / "destructive"`（Button） | `ButtonVariant::Primary / Secondary / Ghost / Danger` | `Outline` と `Link` は Astryx にない。Astryx の対応する見た目（secondary の枠線付き、Link 部品）に合わせて見た目を決める。値は付録 01 の Button 節 |
| `size="sm" / "md" / "lg"` | `ControlSize::Small / Medium / Large` | Astryx に xs はない。`ControlSize::ExtraSmall` は付録 01 の提案値（高さ 24px）で描く |
| `label`（必須の名前） | builder の第一引数（`button("Save")`） | icon だけの部品でも名前は必須。`.hide_label(true)` で隠す |
| `onClick` / `onChange` / `value` | `.on_press(..)`、caller の `Signal` / `Binding` | 状態は呼び出し側の signal に置く |
| `href` / `target` | `.on_press(..)` | 遷移はアプリが行う。Link 部品は `on_press` を取る |
| `isDisabled` | `NodeBuilder::disabled(..)` | disabled は**不透明度 50%**、cursor は変えない（Neutral） |
| `isLoading` | 標準 button の busy（`.loading(..)`） | spinner の周期は 7 章の C-03 |
| `status` / `color` | `Tone`（標準部品の variant）か、キットの enum | Badge の 14 色は `soft::BadgeColor` |
| slot props（`startContent`、`endContent`、`header`、`footer`…） | slot method（`impl FnOnce() -> V`） | basic と同じ規則 |
| `Theme` / `useTheme` / `mode` | `soft::theme(ThemeMode::Light / Dark)` を `App::theme` か `kivact::view::provide_theme` に渡す | `mode="system"`（OS の外観に追従）は 7 章の C-20 |
| `SizeContext`（親の size を子が継承） | 7 章の C-04 | 中核に足すまでは、子に同じ size を明示する |
| `LinkProvider` / `InternationalizationProvider` / `LayerProvider` | 不要、または `App::messages` / Kivact の layer | 付録 05 の 18 章 |

### 4.3 フォント

- 本文と見出しは **Figtree**（400 / 500 / 600 / 700。SIL Open Font License 1.1）。コードはシステムの等幅フォント（`tokens.md` 3.1 の stack。Windows では Consolas）。
- フォントは**アプリが登録する**（Kivact の方針）。キットは `Theme::with_text` で family を `"Figtree, sans-serif"` に、コードを等幅の stack に設定する。Figtree が登録されていなければ sans-serif で描かれる。
- カタログ（`examples/kivact-ui-soft/assets/fonts/figtree/`）は Figtree を同梱する。入手元は Google Fonts のリポジトリ（`github.com/google/fonts` の `ofl/figtree`）。basic のカタログと同じく `OFL.txt`、`SOURCE.md`（入手元と版）、`SHA256SUMS.txt` を置き、`THIRD-PARTY-NOTICES.md` の「Catalog font families」の行に Figtree を足す。

### 4.4 Icon

- Neutral は **Lucide** の 29 種を使う（`tokens.md` 10 章、付録 05 の Icon 節に一覧と、各部品が使う icon 名）。stroke width 2、24px グリッド。表示の大きさは 12 / 16 / 20 / 24px で、既定は 20px。
- SVG は basic と同じ **lucide-static 0.468.0** から取る。basic の `crates/kivact-ui-basic/assets/icons` にある同名のファイルはコピーしてよい（ISC。各 SVG の `@license` コメントを残す）。basic にない名前は同じ版の lucide-static から取る。`crates/kivact-ui-soft/assets/LICENSE-lucide.txt` を basic と同じ内容で置き、`THIRD-PARTY-NOTICES.md` の Lucide の行に soft を足す。
- 名前の集合は `soft::ICON_NAMES` として公開し、`soft::theme` がテーマの icon set にすべて入れる（basic の `icons.rs` と同じ形）。Astryx 固有の意味名（`success`、`error` など）は付録 05 の対応表に従って Lucide の名前に写す。

### 4.5 色・寸法の値

- **値は付録 `tokens.md` の 13 章を `soft::Tokens` の field として持つ。** 部品のコードに色や寸法の数値を直接書かない。部品が使う値はすべて tokens 経由で読む（basic の `theme_access.rs` と同じ形）。
- 計算で決まる値（hover の overlay、color-mix のホバー色、行高）は、付録の手計算の結果を定数として持つ。Kivact で同じ計算をしない。
- Neutral が core の既定を上書きしていない色（Slider と Spinner の track の `#CCD3DB` / `#5A5E66`、Toast の反転背景 `#0A1317` / `#FFFFFF`）は、**core の既定値をそのまま使う**。Neutral に合わせて灰色に「直さない」。
- display-3 の行高は **40px**（Neutral の type scale の計算値）を使う。

### 4.6 文言

- 部品が自分で表示する文言、または名前として使う文言は、英語の既定値で `kivact::i18n::message`（テンプレートは `message_with`）から読む。key は `soft.` で始める。
- 既定値は Astryx の `locales/en.json` の英語をそのまま使う。全 568 件の一覧と、付ける key の案が付録 05 の 18.3 節にある。**実装した部品が使う文言だけ**を登録する。README に key と英語の既定値の表を置く。
- ICU の複数形（plural）が要る文言は、Kivact の `message_with` で表せる形に分ける（例: `soft.table.selected_one` と `soft.table.selected_other`）。分け方は README に書く。

### 4.7 日付・時刻・数値

- タイムゾーンの解釈はアプリが行う。キットの日付部品（Calendar、Date Input、Date Range Input、Date Time Input、Time Input、Timestamp、Timer）は、**アプリが渡したローカル日時**（年・月・日・時・分・秒の値）を扱う。タイムゾーンのデータベースに依存しない。
- 曜日・月の名前、日付の書式は英語の既定値を `soft.*` の文言として持ち、アプリが差し替える。週の始まりは builder の option にする（既定は日曜。Astryx の en-US の既定）。
- 相対時刻（Timestamp の「5 minutes ago」）の規則は付録 05 の Timestamp 節どおり。現在時刻の取得と更新の間隔もそこに従う。

### 4.8 Code Block と Markdown

- syntax highlighting の tokenizer と Markdown のパーサは**キットの中に自前で書く**（外部 crate に依存しない）。basic の `code_block/` がやっている方法を手本にする。
- 対応する言語と Markdown の構文は、付録 05 の CodeBlock 節と Markdown 節に挙げたもの。色は `tokens.md` 2.10 節の `astryx-neutral` の syntax テーマ。
- 段落の途中のリンクや inline code（Markdown、Chat の tokenized text、Link の inline 表示）は 7 章の C-15 を見る。

### 4.9 クリップボード

- コピーの button を持つ部品（Code Block、Timestamp など）は、キットが `kivact::platform::copy_text(..)` で**クリップボードに書く**。そのうえで `.on_copy(..)` callback で結果を知らせる。basic の code block（callback だけでアプリが書く）とは違うことを README に書く。

### 4.10 Toast

- Astryx の Toast は**縦に並ぶ普通のリスト**（間隔 8px、最大 5 件、hover・focus・window の blur で**その toast だけ**が止まる）。basic の sonner 型の重なった束ではない。basic の toast recipe を流用しない。
- 標準の `kivact::ui::toast_region` と `Toaster` を使い、見た目は recipe で作る。リスト表示・per-toast の一時停止・key による重複排除・`on_hide` などで足りない口は 7 章の C-12 で中核に足す。

### 4.11 Drawer と Bottom Sheet

- Astryx の Drawer は**drag で閉じない**。標準の `kivact::ui::dialog` の `DialogKind::Drawer(side)` の restyle にする（basic にある swipe で閉じる `basic::drawer` を真似ない）。
- Bottom Sheet は soft の部品として作り、gesture（drag、flick、snap、rubber band、stack の後退）を付録 03 の 4 節の定数どおりに実装する。足りない gesture の基盤は 7 章の C-09。

### 4.12 タッチとポインタの適応

- Astryx は coarse pointer（タッチ）で文字を大きくし（base 16px）、押下のフィードバックやアニメーションを変える。Kivact の主対象はデスクトップなので、**既定はマウスの経路**を再現する。
- タッチの経路（type scale の適応、タッチ時の押下の遅延と fade、reduced motion 時の 200ms の fade）は、Kivact に「pointer が coarse か」を知る口がないため後回しにする。7 章の C-21 に載せ、口ができたら付録の値で実装する。

### 4.13 Reduced motion

- reduced motion が有効なら、**マウスの経路のとおり変化を即時にする**（`motion.md` 2 章）。例外は Spinner（3 秒で 1 回転に遅くする）と、付録が「方針から外れる」と挙げた約 15 の部品で、それぞれ Astryx の実装どおりにする。
- 試験では `RenderOptions::reduced_motion` で両方の経路を確かめる。

### 4.14 フォーカスリング

- Neutral のフォーカスリングは**accent 色の 2px の実線を、要素の外側 3px 離して描く**（キーボードによるフォーカスのときだけ）。text 入力は外側のリングを描かず、border を accent 色にする。
- Kivact の style には ring の offset がない。中核に offset を足し（7 章の C-01）、`theme_conformance.rs` の button のフォーカスの検査を「テーマが宣言した offset の外側に `Theme::focus_ring()` px の `Palette.focus`」に直す。既存の四つのキットは offset 0 のまま同じ見た目にする。

---

## 5. 見た目の核心（Neutral テーマ）

Codex は見た目の調整が苦手なので、**似せるために外してはいけない点**を先にまとめる。値の根拠は `tokens.md`。

| # | 特徴 | 具体的な値（light / dark） | よくある誤り |
|---|---|---|---|
| 1 | 灰色に色相がない（R=G=B） | 例: page `#f1f1f1` / `#1b1b1b`、surface `#ffffff` / `#262626` | 暖色や寒色の灰色（slate、zinc）を使う |
| 2 | **accent が青ではない**。primary button、checkbox の checked、slider、フォーカスリングは黒に近い色 / 白に近い色 | accent `#1b1b1b` / `#f1f1f1` | 青い primary、青いフォーカスリング |
| 3 | 文字は純黒 / 純白。二次の文字は灰色 | primary `#000000` / `#ffffff`、secondary `#474747` / `#9e9e9e` | 文字を `#111` などにする |
| 4 | 面どうしの差が小さい。dark では card と popover（`#1b1b1b`）が main surface（`#262626`）より**暗い** | `tokens.md` 2.2 | dark の card を明るくする |
| 5 | 境界線がとても薄い | 既定の 1px の線は black 7.8% / white 10%。入力欄の枠は `#d4d4d4` / `#525252` | 不透明な `#e5e5e5` の線 |
| 6 | hover と press は**半透明の層を重ねる** | hover は black / white 5.1%、press は 10.2% | hover で fill の色そのものを変える |
| 7 | primary button の押下の主なフィードバックは**縮小**（scale 0.98） | `motion.md` 3.4 | 押下で色だけ変える |
| 8 | checkbox・radio・switch・link の hover は色の **color-mix** | 未選択の枠 20%、背景 5%、accent 15%（`tokens.md` 9.3） | 6 の overlay を使う |
| 9 | 影は light で柔らかく（5〜15% の黒、二層）、dark で深く（25〜70%）、**dark では 1px の白い内側の縁**（8% / 12% / 15%）が付く | `tokens.md` 7.1 | dark で影をなくす、縁を付け忘れる |
| 10 | 角丸がやや大きい | 6 / 10 / 12 / 28px、badge は完全な丸 | 4px や 8px に揃える |
| 11 | 書体は Figtree。字間は全部 0。大文字だけのラベルはない | 本文 14/20、small 12/20、見出し 24/20/17/14/12/10px（h3・h4 は bold、他は semibold）、display 42/35/29px は weight 400 | Inter で代用、見出しに letter-spacing |
| 12 | 余白は 4px 単位。Neutral は card と section の padding を 12px に詰める | 制御の高さ 28 / 32 / 36px | 8px 単位、padding 16px |
| 13 | disabled は**常に不透明度 50%**、cursor は普通 | — | 灰色の別の色にする |
| 14 | フォーカスは accent の 2px のリングを 3px 外側に | 4.14 節 | 青いリング、offset なし |
| 15 | 色が出るのは状態色・badge・token・コードだけ | info `#0074e2` / `#6d9cfe`、success `#198100` / `#64af4c`、warning `#ffce2f`、error `#c9303a` / `#ff705d`。淡い fill は light で pastel、dark で 24% の半透明 | 装飾に色を足す |
| 16 | Neutral の destructive button は**淡い赤の fill に濃い赤の文字**、フォーカスリングも赤 | 付録 01 の Button 節 | 濃い赤の fill に白文字（shadcn 風） |

---

## 6. モーションの核心

詳細は `foundation/motion.md`。部品ごとの値は各付録の「モーション」の項。

### 6.1 Astryx のモーションの性格

1. **曲線は一つ。** ほとんどの transition が `--ease-standard` = `cubic-bezier(0.24, 1, 0.4, 1)` を使う。時間の 20% で 64%、50% で 95% まで進む前のめりの曲線で、体感の長さは名目の約半分になる。**Kivact の `Easing::EaseOut`（三次多項式）や CSS の `ease` で代用しない。** 必ず `Easing::CubicBezier(0.24, 1.0, 0.4, 1.0)` を使う。
2. **短く、重さで段がついた時間**（Neutral）: 小さなフィードバックは 95 / 125 / 165ms、入場とパネルは 225 / 300 / 400ms、ループは 525 / 700 / 935ms。hover と色の変化は 125ms、Dialog の入場は 400ms、anchor 付きの layer は 165ms。
3. **移動は小さく静か。** layer は 8px 離れたところから scale 0.95 で入る。Dialog は**開いた要素の方向から** 16px、scale 0.95 で入る。button の押下は scale 0.98。
4. **ほとんどの退場はアニメーションしない。** Tooltip、Popover、menu、Dialog、Lightbox は 1 frame で消える。退場が動くのは Drawer、Bottom Sheet、Mobile Nav、Toast、Top Nav の mega menu、Overlay だけ。Bottom Sheet だけは退場に別の曲線 `cubic-bezier(0.3, 0, 0.6, 0.6)` と、scrim の linear な fade を使う。
5. **飾りがない。** shimmer はない（Skeleton は段階的な不透明度の pulse）。check の描画アニメーションもない。Segmented Control と Tab の選択の印も滑らない。メッセージの入場アニメーションも View Transition もない。**付録にないアニメーションを足さない。**
6. **物理は少しだけ。** spring を使うのは Switch と Slider（タッチ時）、それにレイアウトの glide（Collapsible、Tree List、Table の行の展開、Side Nav、App Shell）だけ。glide は `springs.settle`（臨界減衰、k 157.914、c 25.133、約 735ms）。sheet と toast は CSS の時間指定で、速度の引き継ぎはない。
7. **reduced motion は厳格**（4.13 節）。

### 6.2 Kivact での実現の方針

| Astryx のパターン | Kivact での実現 | 不足（7 章） |
|---|---|---|
| paint の transition（色・不透明度・transform・影） | style の `TransitionProp::Paint` に `Easing::CubicBezier` と時間を付ける | text の色、gradient、radius、ring は補間されない → C-05 |
| 入場（mount 時） | `.enter(..)` style + paint の transition | — |
| 退場（unmount 時） | `.exit(..)` style | 退場中も layout の場所を取る。退場の完了を知る callback がない → C-07 |
| ループ（spinner、skeleton の pulse、progress の不定表示） | `kivact::motion` の `frame_loop` か keyframes | 標準 button の busy の周期が固定 → C-03 |
| spring（Switch・Slider の thumb） | `kivact::motion` の `Spring {stiffness, damping, mass}` | recipe の中で animated な値を作れない（setup 時だけ）→ C-02 |
| 高さの開閉（Collapsible、Toast の行、Side Nav） | disclosure のように手で高さを動かす | layout の property の transition がない → C-06 |
| FLIP のレイアウトアニメーション | 当面は使わない（Astryx でも既定ではない） | C-08 |
| gesture（drag、flick、snap、rubber band） | pointer の event から自前で計算 | 速度の追跡、drag の slop、decay、snap がない → C-09 |
| retarget（途中で逆向きに戻る） | spring は速度を保つ | tween は retarget で時間が最初からやり直しになる → C-10 |

**最初は「Astryx の既定（非 FLIP）」の形で全部品を出す。** そのうえで 7 章の中核の追加ができたものから、付録の正確な動きに差し替える。

---

## 7. 中核（kivact 本体）への追加

付録が「Kivact に口がない」とした点をまとめた。Kivact の方針（`docs/for-kivact-developer/strategy.md` の 5 章と 6 章）では、**部品の振る舞い・入力・アニメーションの口は中核が持ち、キットで複製しない**。このため、下の項目は中核（`kivact-core`、`kivact-style`、`kivact-motion`、`kivact-interaction`、`kivact-ui`）に足す。

### 7.1 中核を変えるときの規則

1. **項目ごとに commit を分ける**（conventional commits、例 `feat(style): add ring offset`）。キットの作業と同じ commit に混ぜない。
2. 公開 API を足したら、同じ変更で rustdoc、`dev/xtask/policy/workspace.toml`（必要なら）、利用者向けドキュメント（`docs/for-kivact-user/{en,jp}`、該当する how-to）を更新する。
3. 公開 API の E2E を `dev/tests/kivact-public-api/tests/e2e` に足す。利用者の契約を確かめる試験にする。
4. 既存の四つのキットの見た目と挙動を変えない（3 章の 5）。変わるなら、その kit の recipe を直して元の見た目を保つ。
5. 中核の変更は**サブエージェントに並行させない**。一つずつ主エージェントが順に統合する（8.3 節）。
6. 項目が大きすぎる、または設計の判断が要る（下の表で「要相談」）ときは、作者に説明して判断を仰ぐ。それまでは「代替」の列の方法でキットを進める。

### 7.2 追加する項目

優先度: **A** = フェーズ 1 で先に入れる（多くの部品の見た目に効く）。**B** = 該当する部品の実装の前に入れる。**C** = 後でよい。代替で出荷し、後で差し替える。

| ID | 項目 | 使う部品 | 優先度 | 代替（中核に入るまで） |
|---|---|---|---|---|
| C-01 | style に ring（outline）の **offset** を足す。`theme_conformance.rs` のフォーカスの検査を offset に対応させる | すべての focusable な部品 | A | なし（先に入れる） |
| C-02 | recipe・part から**状態に追従する animated な値**を使えるようにする（switch の thumb の移動、checkbox の印、slider の thumb の lift） | Switch、Checkbox、Radio、Slider、Segmented Control | A | thumb の位置を即時に変える |
| C-03 | 標準 button の busy の **spinner の周期と表示の遅延を recipe か theme から変えられる**ようにする（Neutral は 525ms で 1 回転、225ms 待ってから fade in） | Button、Icon Button、Toggle Button | B | 0.7s の固定周期、遅延なし |
| C-04 | **size の継承**（親の `ControlSize` を子が既定値として使う context。子の明示が勝つ） | Button Group、Toggle Button Group、Toolbar、Input Group、Field、Segmented Control | B | 子に同じ size を明示し、README とカタログにそう書く |
| C-05 | paint の transition で **text の色**を補間する | Link、Tab、Side Nav Item、menu の item、全 hover | B | 文字色は即時に切り替える |
| C-06 | **layout の property（高さ・幅）の transition**、または「内容の高さへ開く」の公式の手段 | Collapsible、Collapsible Group、Toast の行、Side Nav の section、Tree List、Table の行の展開、Code Block の折り畳み、Chat の tool call | B | disclosure と同じく手で高さを動かす |
| C-07 | 退場中の node を **layout から外す** option と、退場の完了の callback | Toast、Drawer、Bottom Sheet、Mobile Nav、Top Nav の mega menu | B | 退場の間は場所を残す |
| C-08 | FLIP のレイアウトアニメーション（測ってから変える） | useAnimatedLayout を使う部品（App Shell、Side Nav の開閉、Tokenizer の token の追加・削除など） | C（要相談） | 即時に配置を変える（Astryx の既定と同じ） |
| C-09 | **gesture の基盤**: drag の slop、軸の固定、速度の追跡（event の timestamp）、decay、snap、rubber band | Bottom Sheet、Toast の swipe、Carousel、Slider（タッチ） | B | キットの中で pointer の event から計算する（Bottom Sheet だけ）。後で中核へ移す |
| C-10 | tween の retarget で、残りの距離に応じた時間にする（CSS の transition の反転と同じ） | すべての hover の transition | C | そのまま |
| C-11 | **disabled でもフォーカスを受け、tooltip を出せる**（`aria-disabled` 相当） | Button、Icon Button、menu の item | B | disabled ではフォーカスできないまま |
| C-12 | toast の**リスト表示**（重ねない）、**その toast だけの一時停止**、key（uniqueID）での**重複排除**、`on_hide` の callback、行の高さのアニメーション | Toast | B | 標準の toast_region の今の動作で、recipe だけ合わせる |
| C-13 | **focus-within** を node 自身の style で使う（`group()` + `group_focus` で足りなければ） | Token、Input Group、Tokenizer、Overlay | B | `group()` + `group_focus` で作る |
| C-14 | **roving focus のコンテナ**（slot の中の focusable な子を矢印キーで移動。text 入力の caret の位置を考慮） | Toolbar、Segmented Control、Toggle Button Group | B | キットの `toolbar_item(view)` で包んだものだけを数える |
| C-15 | **段落の中の inline な view**（リンク、inline code、token、mention を文字の流れの中に置く） | Markdown、Link（inline 表示）、Chat の tokenized text、Citation | B（要相談） | 段落を inline の部品ごとに折り返す row で近似する |
| C-16 | layer の配置: **window の端から 16px の余白**で clamp する。Tooltip が anchor を覆わない規則。flip → span → slide の順の fallback | Tooltip、Popover、Hover Card、menu、Selector の listbox | A | 今の配置のまま |
| C-17 | submenu の **hover の時間**（開くまで 150ms、閉じるまで 200ms）と **safe triangle** | Dropdown Menu の submenu、Context Menu | B | 今の時間（140ms / 0） |
| C-18 | menu を**押したまま drag して離すと選ぶ**操作、長押しで開く | Dropdown Menu、More Menu、Context Menu | C | クリックで開く |
| C-19 | 支援技術への**任意の通知**（announce の API） | Toast、Table の選択、Tokenizer、Chat | B | 標準部品の live region だけ |
| C-20 | OS の**外観（light / dark）の取得**と変化の通知 | Theme の `mode="system"` | C | アプリが mode を選ぶ |
| C-21 | **pointer が coarse か**（タッチか）の取得 | タッチの経路全部（4.12 節） | C | マウスの経路だけ |
| C-22 | text が**切り詰められたか**の取得 | Text の `maxLines` で全文を tooltip に出す、Token、Breadcrumb Item | C | 切り詰めの可能性があるときは常に tooltip を付ける |
| C-23 | 画像の**読み込み失敗**の取得 | Avatar（イニシャルへの fallback）、Thumbnail、Lightbox | B | 画像がない場合だけ fallback |
| C-24 | 3D の `rotateX`（日付の wheel） | Date Time Input のタッチの wheel | C | wheel を作らない（デスクトップの経路だけ） |
| C-25 | 破線の下線（dashed underline） | Citation、Timestamp（hover card を持つもの） | C | 実線の下線 |
| C-26 | reduced motion の検出を Windows 以外でも行い、**アプリが上書き**できるようにする | 全アニメーション | C | Windows の検出だけ |
| C-27 | 直前にフォーカスしていた要素へ**フォーカスを戻す**公開の手段 | Banner の dismiss、Toast の close、Lightbox | B | 次の focusable へ移る |

> 7.3 節に続く。

### 7.3 付録 02・04・06 が見つけた追加の項目

| ID | 項目 | 使う部品 | 優先度 | 代替（中核に入るまで） |
|---|---|---|---|---|
| C-28 | 標準部品 **`kivact::ui::tree`**（`use_tree` の上。tree の recipe の表、ガイド線のための `is_last` と祖先の `is_last`）。game の `tree` と creative の `outliner` が同じ意味を重複して持っているので、それらも新しい標準部品の restyle に移す | Tree List | B | `use_tree` の上のキットの部品 |
| C-29 | `data_table` の機能: 静的な表の意味（`TableFocus::None`。grid にしない）、density、比率の幅と下限、揃え、折り返しと切り詰め、空の状態の slot、header と行の右クリックメニュー、行の index、複数列の並べ替え、列ごとの filter、行の展開、グループの行、tree の行、sticky な列、行の状態、列の順序、行ごとの選択の可否、page size と hasMore、列の幅の変更の拡張（Shift の大きな step、Home / End、列ごとの可否） | Table と全 hook | B（項目ごとに分ける） | 既存の機能だけで出し、README に足りない機能を書く |
| C-30 | `kivact::interaction::use_follow_end`（末尾への追従のスクロール。basic の `message_scroller` もこれに移す） | Chat Layout | B | キットの中で追従を作る |
| C-31 | role と状態の追加: `Role::Log`、`Role::Article`、busy、required、read only、has popup、current | Chat Message List、Chat Message、Field、menu の trigger、Side Nav Item、Breadcrumbs、Pagination | B | 説明の文と live region で代える |
| C-32 | **text_area の折り返し（soft wrap）**と、行数の範囲での自動の高さ。いまの native editor は折り返さない（`crates/kivact-core/src/primitive/text_editing.rs:164`） | Text Area、Chat Composer Input、Rich Text Editor | **A** | 折り返さない（横に scroll） |
| C-33 | 静的な item list の標準部品（`Role::List` / `ListItem`、marker、divider、density）。basic の `item_group` と soft の List が同じ意味を持つ | List、Item | C | キットの部品 |
| C-34 | layout はするが、描画・支援技術・フォーカスから除く node（幅を測るため） | Overflow List、Tokenizer の末尾の予約 | C | `node_ref().rect()` で測る |
| C-35 | `use_combobox`、`use_select`、`use_calendar_grid` のキー操作・範囲の anchor・フォーカスの規則で Astryx と違う点を、中核の hook の option にする | Selector 群、Typeahead、Tokenizer、Calendar、Date Range Input | B | 付録 02 の仕様のキットの engine。中核に入ったら engine を消して hook を使う |
| C-36 | 地域ごとの書式の表（数値の記号、時刻の hour cycle、月と曜日の名前、collation） | Number Input、Date / Time Input 群、Calendar、Timestamp | C | 英語（en-US）の表をキットに持つ |
| C-37 | scroll の snap、慣性、smooth scroll の曲線、Shift + wheel の横 scroll | Carousel、Date の wheel、Outline、Scrollable Area | C | 即時の scroll |
| C-38 | `display: contents` に当たる配置（Form Layout の横並びのラベル）と、cap height での文字の上下の trim | Form Layout、Text の `hasCapsize` | C | 行を分けて組む、trim しない |
| C-39 | hover の **safe triangle**（C-17 と同じ仕組みを Top Nav の menu と Side Nav の flyout にも使う） | Top Nav、Side Nav | B | C-17 と一緒に入れる |

注:
- 付録 04 などに「offset 付きのフォーカスリングを自前の paint で描く」案があるが、**それは採らず C-01 に従う**。
- 付録 02 の Switch・Checkbox・Radio の「paint の印は補間されない」問題は C-02 で解く。印を子 node にして recipe を足す案（付録 02 の 11.7 節）も C-02 の設計の候補として検討する。
- Tab List の手動の選択は、既存の `kivact::ui::tabs(..).activation(ListActivation::Manual)` を使う。

---

## 8. 作業計画とサブエージェントの分担

### 8.1 フェーズ

各フェーズの最後に 13.1 節の「フェーズの完了確認」を行い、commit してから次へ進む。

| フェーズ | 内容 | 担当 | 完了の目安 |
|---|---|---|---|
| **0. 準備** | 付録の foundation 三つを読む。`git submodule update --init`。`cargo xtask check` と、basic の consumer 試験を一度通して環境を確かめる | 主 | 検査が通る |
| **1. 中核 A** | 7.2 節と 7.3 節の優先度 A（C-01、C-02、C-16、C-32）を中核に入れる | 主（1 項目ずつ） | 公開 API の E2E が通り、既存キットの見た目が変わらない |
| **2. 骨格** | `crates/kivact-ui-soft` の骨格: `Cargo.toml`、`lib.rs`、`tokens.rs`（`tokens.md` 13 章の全 field、light と dark）、`color.rs`、`theme.rs`、`theme_access.rs`、`icons.rs` と SVG、`recipes.rs`（空の表）。カタログの骨格（ページの登録と light / dark の切替だけ）。ワークスペースへの登録（`kivact-kit-architecture.md` 1.11 節の全ファイル）。`theme_conformance.rs` に Soft を足す | 主 | `soft::theme` を App に付けて起動し、カタログの空のページが出る。`cargo xtask check` が通る |
| **3. 標準部品の restyle** | 標準部品の recipe を全部書く。button、toggle、checkbox、switch、radio_group、text_input、text_area、number_input、select、menu、dialog（Modal、AlertModal、Drawer、Popover）、tooltip、tabs、slider、range_slider、toast_region、splitter、split、progress、disclosure、list、data_table、command_palette、tag_input、field | サブエージェント（8.2 節の R1〜R4） | 標準部品だけで作ったカタログのページが、light / dark で付録の値どおりに見える。`theme_conformance.rs` が通る |
| **4. キットの部品** | 9 章の「NEW」と「RESTYLE + wrapper」の部品を、付録ごとに作る | サブエージェント（8.2 節の K1〜K6） | 各部品のカタログのページと consumer 試験 |
| **5. 中核 B と差し替え** | 7.2 節と 7.3 節の優先度 B の項目を中核に入れ、代替で作った部分を付録の正確な動きに差し替える | 主（中核）+ サブエージェント（差し替え） | 代替の印（`// TODO(soft-core: C-xx)`）が残らない（C の項目を除く） |
| **6. 仕上げ** | 10 章の見た目の調整を全ページで行う。README、利用者ドキュメント、`THIRD-PARTY-NOTICES.md`。13 章の merge 前の検査 | 主 + サブエージェント（ドキュメント） | 13 章の完了条件 |

### 8.2 サブエージェントの分担

1 つのサブエージェントに 1 つの担当を与え、**担当のファイルだけを編集させる**。共有のファイル（`lib.rs` の `mod` と `pub use`、`recipes.rs` の表の組み立て、`tokens.rs`、カタログのページ登録、README の表）は**主エージェントだけが編集する**。サブエージェントは、そこへ足す行を完了報告に書く。

| 担当 | 内容 | 読む付録 | 編集してよいファイル（例） |
|---|---|---|---|
| R1 | 標準部品の restyle: button、toggle、checkbox、switch、radio_group、slider、range_slider、progress | 01、02、`motion.md` | `src/recipes_controls*.rs` |
| R2 | 標準部品の restyle: text_input、text_area、number_input、tag_input、field、select | 02 | `src/recipes_field*.rs`、`src/recipes_select*.rs` |
| R3 | 標準部品の restyle: menu、dialog（4 種）、tooltip、toast_region、command_palette | 03 | `src/recipes_menu*.rs`、`src/recipes_dialog*.rs`、`src/recipes_popup*.rs` |
| R4 | 標準部品の restyle: tabs、disclosure、splitter、split、list、data_table | 04、06 | `src/recipes_collections*.rs` |
| K1 | 付録 01 のキットの部品（Badge、Banner、Button Group、Toggle Button Group、Segmented Control、Toolbar、Link、More Menu、Skeleton、Spinner、Status Dot、Indicator、Kbd、Token） | 01 | 各部品の `src/<component>.rs` |
| K2 | 付録 02 のキットの部品（Field の構成、Input Group、File Input、Checkbox List、Radio List、Selector 群、Typeahead、Tokenizer、Power Search、Calendar、Date / Time Input 群、Rich Text Editor） | 02 | 同上 |
| K3 | 付録 03 のキットの部品（Bottom Sheet、Bottom Sheet Switcher、Hover Card、Lightbox、Overlay、Dialog Header、Drawer Header、Alert Dialog の構成） | 03 | 同上 |
| K4 | 付録 04 のキットの部品（App Shell、Layout、Stack 群、Grid、Section、Form Layout、Aspect Ratio、Divider、Resize Handle、Scrollable Area、Breadcrumbs、Outline、Pagination、Side Nav、Top Nav、Mobile Nav、Stepper、Tab Menu、Card 群、Carousel、Collapsible） | 04 | 同上 |
| K5 | 付録 05 のキットの部品（Avatar 群、Blockquote、Citation、Code、Code Block、Empty State、Text、Heading、Icon、Markdown、Thumbnail、Timer、Timestamp、VisuallyHidden） | 05 | 同上 |
| K6 | 付録 06 のキットの部品（List、Item、Metadata List、Overflow List、Tree List、Table の機能、Chat 群） | 06 | 同上 |
| D | README、利用者ドキュメント（英語と日本語）、カタログの説明文 | 本書 12 章 | `crates/kivact-ui-soft/README.md`、`docs/for-kivact-ui-user/{en,jp}/soft/` |

- 同時に動かすサブエージェントは**最大 3 つ**にする。それぞれ別の git worktree で作業させ、主エージェントが一つずつ merge して検査する。
- ウィンドウを開く試験は、どの worktree からも `dev/tools/window-lock.sh` を通す（同時に開くとフォーカスを奪い合って止まる）。
- K2 と K6 は大きいので、さらに部品ごとに分けてよい（例: K2a = Selector 群と Typeahead と Tokenizer と Power Search、K2b = 日付の部品、K2c = Rich Text Editor）。
- 順序: フェーズ 3 の R1〜R4 を先に終える。キットの部品は標準部品の見た目（button、menu、popover など）を中に使うためである。K5 の Text・Heading・Icon は他の部品が使うので、フェーズ 4 の最初に行う。

### 8.3 サブエージェントへの指示の書き方

主エージェントは、サブエージェントを起動するとき、次をすべて渡す。

1. 担当の部品の一覧と、読む付録の**節の番号**。「付録の該当節を全部読み、8 項目をすべて満たすこと」と書く。
2. 本書の 3 章（規則）、4 章（決定済み事項）、5 章（見た目の核心）、6.1 節（モーションの性格）を読むこと。
3. 編集してよいファイルの一覧と、編集してはいけない共有のファイル。
4. 中核に口が足りないときの扱い: 自分で中核を変えず、7.2 節の「代替」の方法で作り、`// TODO(soft-core: C-xx)` の印を付け、完了報告に書く。表にない不足を見つけたら、新しい ID の案と理由を完了報告に書く。
5. 完了報告に書くこと: 作った部品と公開 API、共有ファイルに足す行、実装した試験とその結果、10 章の見た目の確認の結果（撮った画像の場所と、付録の値と比べた結果）、残した `TODO(soft-core: …)`、付録を直した箇所。
6. 「付録の値を推測で変えない。疑わしい値は Astryx のソースで確かめ、付録を直す」こと。

---

## 9. コンポーネント別の作業一覧

各部品の詳細な仕様は付録にある。ここでは**実装の種類**と、手本にする basic のファイルを示す。

- **RESTYLE**: 標準部品を recipe で restyle するだけ。キットに関数を作らない。
- **RESTYLE + wrapper**: 標準部品を restyle し、Astryx の構成（header、slot、子部品）を作る小さなキットの関数を足す。
- **NEW**: キットの部品として作る。
- **MERGED**: 別の部品の機能として実装する。

（表は付録の要約表を集めたもの。付録が別の判断を示していれば付録に従い、ここを直す。）

### 9.1 付録 01（Action と Feedback）

| Astryx | 種類 | Kivact | 手本（basic） |
|---|---|---|---|
| Button | RESTYLE | `kivact::ui::button` | `recipes_controls.rs` |
| Icon Button | RESTYLE | `button(name).icon(..).hide_label(true)` | 同上 |
| Button Group | NEW | `soft::button_group` | `button_group.rs` |
| Toggle Button | RESTYLE | `kivact::ui::toggle` | `choice.rs`、`recipes_controls.rs` |
| Toggle Button Group | NEW | `soft::toggle_button_group` | `choice.rs`（`toggle_group`） |
| Link | NEW | `soft::link` | `typography.rs` |
| More Menu | RESTYLE + wrapper | `soft::more_menu`（`kivact::ui::menu` を icon button に attach） | `recipes_menu.rs` |
| Segmented Control (+Item) | NEW | `soft::segmented_control` | `choice.rs` |
| Toolbar | NEW | `soft::toolbar` | `button_group.rs` |
| Badge | NEW | `soft::badge` | `badge.rs` |
| Banner | NEW | `soft::banner` | `alert.rs` |
| Progress Bar | RESTYLE | `kivact::ui::progress` | `recipes_controls.rs` |
| Skeleton | NEW | `soft::skeleton` | `primitives.rs` |
| Spinner | NEW | `soft::spinner` | `primitives.rs` |
| Status Dot | NEW | `soft::status_dot` | `badge.rs` |
| Indicator（Checkbox / Check / Radio） | NEW | `soft::checkbox_indicator` など | `recipes_controls.rs` の印 |
| Kbd | NEW | `soft::kbd` | `primitives.rs` |
| Token | NEW | `soft::token` | `badge.rs` |

### 9.2 付録 02（Form Controls）

| Astryx | 種類 | Kivact | 手本（basic） |
|---|---|---|---|
| Text Input | RESTYLE + wrapper | `kivact::ui::text_input` + `soft::text_input`（start icon、clear、spinner、status の icon） | `input_group.rs`、`recipes.rs` |
| Text Area | RESTYLE + wrapper | `kivact::ui::text_area` + `soft::text_area`（文字数、resize の grip）。折り返しは C-32 | `input_group.rs` |
| Number Input | NEW | `soft::number_input`（Astryx の parse と commit の規則。`kivact::ui::number_input` は recipe の restyle だけ） | `kivact-ui/src/number_input.rs` |
| Field（+Field Label） | RESTYLE + wrapper | `kivact::ui::field` + `soft::field`、`soft::field_label` | `field.rs`、`recipes_field.rs` |
| Field Status | NEW | `soft::field_status`（attached / detached / tooltip） | `alert.rs` |
| Input Group（+Input Group Text） | NEW | `soft::input_group`、`soft::input_group_text` | `input_group.rs`、`button_group.rs` |
| File Input | NEW | `soft::file_input`（input の形と dropzone の形） | `input_group.rs`、`attachment.rs` |
| Checkbox Input | RESTYLE + wrapper | `kivact::ui::checkbox` + `soft::checkbox_input` | `recipes.rs`（`draw_checkbox`） |
| Checkbox List（+Item） | NEW | `soft::checkbox_list`（行の箱は restyle した checkbox） | `field.rs`、`item.rs` |
| Radio List（+Item） | RESTYLE + wrapper | `kivact::ui::radio_group` + `soft::radio_list` | `recipes_controls.rs` |
| Switch | RESTYLE + wrapper | `kivact::ui::switch` + `soft::switch_field`。thumb の移動は C-02 | `recipes.rs`（`draw_switch`） |
| Slider | RESTYLE + wrapper | `kivact::ui::slider` / `range_slider` + `soft::slider`（marks、値の tooltip） | `recipes_controls.rs` |
| Selector（+Selector Option） | RESTYLE + NEW | `kivact::ui::select` の restyle + `soft::selector` | `combobox.rs`、`list_popup.rs` |
| Complex Selector | NEW | `soft::complex_selector`（`DialogKind::Popover` を中に使う） | `date_picker.rs` |
| Multi Selector | NEW | `soft::multi_selector`（作成の行、全選択、option の action、trigger の count / labels / badges） | `combobox.rs`、`list_popup.rs` |
| Typeahead（+Item）、Base Typeahead | NEW | `soft::typeahead`、Typeahead と Tokenizer が共有するキットの engine（`use_text_edit` + overlay） | `combobox.rs` |
| Tokenizer | RESTYLE + NEW | `kivact::ui::tag_input` の restyle + `soft::tokenizer` | `combobox.rs` |
| Power Search | NEW | `soft::power_search`（`soft::tokenizer` の上） | `combobox.rs`、`date_picker.rs` |
| Calendar | NEW | `soft::calendar`（`kivact::interaction::use_calendar_grid` と `kivact::i18n::CalendarDate`。キー操作が合わなければキットの engine。付録 02 の Calendar 節） | `calendar/` |
| Date Input | NEW | `soft::date_input`（入力欄 + Calendar の popover）。タッチの sheet と wheel は 4.12 節のとおり後回し | `date_picker.rs` |
| Date Range Input | NEW | `soft::date_range_input`（preset の sidebar + 範囲の Calendar） | `date_picker.rs` |
| Date Time Input | NEW | `soft::date_time_input`（日付と時刻の segment、時刻の listbox） | `date_picker.rs`、`list_popup.rs` |
| Time Input | RESTYLE + wrapper | `kivact::ui::text_input` + `soft::time_input` | `input_group.rs` |
| Rich Text Editor | NEW | `soft::rich_text_editor`、`soft::rich_text_toolbar`、`soft::rich_text_view`。**第 1 段階**は restyle した text_area 上の Markdown のソースの編集と、`kivact::control::rich_text` による表示。**第 2 段階**（WYSIWYG）は C-32 と C-15 の後（付録 02 の 25 節の段階計画） | `code_block/view.rs`、`typography.rs` |
| useCombobox、useMultiCombobox、useTypeahead、useInputContainer、useInputStatusIcon、useCollator / useLocale | MERGED / helper | 付録 02 の 26 章。`kivact::interaction` の `use_combobox` などは Astryx とキー操作が違うので、付録の仕様のキットの engine にする | — |

### 9.3 付録 03（Overlays と Menus）

| Astryx | 種類 | Kivact | 手本（basic） |
|---|---|---|---|
| Dialog (+Header, useImperativeDialog) | RESTYLE + wrapper | `kivact::ui::dialog(..).kind(DialogKind::Modal)` + `soft::dialog_header` | `recipes_dialog.rs` |
| Alert Dialog (+useImperativeAlertDialog) | RESTYLE + wrapper | `DialogKind::AlertModal` | `alert_dialog.rs` |
| Drawer (+Header) | RESTYLE + wrapper | `DialogKind::Drawer(side)` + `soft::drawer_header` | `recipes_dialog.rs`（`basic::drawer` は真似ない。4.11 節） |
| Bottom Sheet | NEW | `soft::bottom_sheet` | `drawer.rs`（swipe の部分） |
| Bottom Sheet Switcher | NEW | `soft::bottom_sheet_switcher` | — |
| Popover (+usePopover) | RESTYLE | `DialogKind::Popover` | `recipes_dialog.rs` |
| Hover Card (+useHoverCard) | NEW | `soft::hover_card` | `hover_card.rs` |
| Tooltip (+useTooltip) | RESTYLE | `kivact::ui::tooltip` | `recipes_popup.rs` |
| Toast (+useToast) | RESTYLE | `kivact::ui::toast_region` + `Toaster` | `recipes_popup.rs`（sonner の束は真似ない。4.10 節） |
| Command Palette（+Empty、Footer、Group、Input、Item、List） | RESTYLE | `kivact::ui::command_palette` | `recipes_command.rs` |
| Dropdown Menu（+全子部品） | RESTYLE | `kivact::ui::menu(..).attach(trigger)` | `recipes_menu.rs` |
| Context Menu (+Item) | RESTYLE | `kivact::ui::menu(..).attach_context(target)` | `recipes_menu.rs` |
| Lightbox | NEW | `soft::lightbox` | `carousel.rs` |
| Overlay | NEW | `soft::overlay`（media の上の hover / focus の層。modal の backdrop ではない） | — |
| Layer (+useLayer、LayerProvider) | MERGED | Kivact の layer（`kivact::ui` の overlay の仕組み） | `layers.rs` |
| useFocusTrap、useScrollLock、useAnnounce | MERGED | 標準 dialog のフォーカストラップ、C-19 | — |

### 9.4 付録 04（Navigation・Layout・Container）

| Astryx | 種類 | Kivact | 手本（basic） |
|---|---|---|---|
| App Shell (+useAppShellMobile) | NEW | `soft::app_shell`（中で Layout と Mobile Nav を使う。md 未満で mobile） | `sidebar.rs`（`sidebar_provider`） |
| Layout（+Header、Content、Footer、Panel） | NEW | `soft::layout` と region の関数（`column` / `row` / `scroll` + style。padding はキットの context で子へ伝える） | `card.rs`、`sidebar.rs` |
| Stack、H Stack、V Stack、Stack Item | helper | `soft::stack` / `hstack` / `vstack`（`row` / `column` に gap と揃えを付ける薄い builder） | — |
| Grid、Grid Span | helper | `soft::grid`（kivact の `grid` と `Track`） | — |
| Center | helper | `soft::center` | — |
| Section | NEW | `soft::section`（bleed と padding の context） | `card.rs` |
| Form Layout | NEW | `soft::form_layout`（ラベルの横並びは 480px 未満で縦に） | `field.rs` |
| Aspect Ratio | helper | `soft::aspect_ratio`（`Style::aspect_ratio`） | `primitives.rs` |
| Divider | NEW | `soft::divider`（label 付き、bleed） | `primitives.rs`（`separator`） |
| Resize Handle (+useResizable) | RESTYLE + wrapper | `kivact::ui::split` / `splitter` + `soft::resizable`（collapse、snap、px 指定） | `recipes_controls.rs` |
| Scrollable Area（+useScrollableArea、useScrollOverflow、useOverflow） | RESTYLE + wrapper | `kivact::control::scroll` + `soft::scrollable_area` | `scroll_area.rs` |
| Breadcrumbs (+Item) | NEW | `soft::breadcrumbs`（省略した項目の menu は `kivact::ui::menu`） | `breadcrumb.rs` |
| Outline | NEW | `soft::outline`（`use_list` + scroll spy） | `sidebar.rs` |
| Pagination | NEW | `soft::pagination`（中で button、number_input、select を使う） | `pagination.rs` |
| Side Nav（+Heading、Item、Section、Collapse Button） | NEW | `soft::side_nav`。折り畳みは幅を settle の spring で動かし、label と icon を進み 0.4 / 0.6 / 0.7 で cross fade | `sidebar.rs`、`sidebar/` |
| Top Nav（+Heading、Item、Menu、Mega Menu、Mega Menu Item、Mega Menu Featured Card） | NEW | `soft::top_nav`（hover の popup は `use_popup` + `use_delayed`。mega menu は 225ms の fade + 4px） | `navigation_menu.rs`、`menubar.rs` |
| Mobile Nav (+Toggle) | RESTYLE + wrapper | `DialogKind::Drawer(side)` + `soft::mobile_nav`（300ms の slide、backdrop の fade と 2px の blur） | `sidebar.rs`、`recipes_dialog.rs` |
| Nav Icon | NEW | `soft::nav_icon` | `avatar.rs` |
| Nav Heading Menu | NEW | `soft::nav_heading_menu`（`use_list` の menu） | `list_popup.rs`、`recipes_menu.rs` |
| Stepper (+Step) | NEW | `soft::stepper` | `questionnaire.rs` |
| Tab List（+Tab、Tab Menu） | RESTYLE + wrapper | `kivact::ui::tabs`（**選択は手動**: `ListActivation::Manual`。下線は滑らず、各 tab が 125ms で fade）+ `soft::tab_menu` | `line_tabs.rs` |
| Card | NEW | `soft::card`（枠線は default の variant だけ。Neutral の padding は 12px） | `card.rs` |
| Clickable Card | NEW | `soft::clickable_card`（Card + `use_button`） | `item.rs` |
| Selectable Card | RESTYLE + wrapper | `use_checkbox` の意味 + `soft::selectable_card` | `choice.rs`、`card.rs` |
| Carousel | NEW | `soft::carousel`（横の scroll の strip、端の 4px の fade、半分はみ出した丸い button。**paging も物理もない**。basic の carousel を流用しない） | `carousel.rs`（構造は流用しない） |
| Collapsible（+Group、useCollapsible） | RESTYLE + wrapper | `kivact::ui::disclosure` + `soft::collapsible_group`。**既定で開いている**。開閉は既定で即時（`animated_layout` を指定したときだけ glide） | `disclosure.rs` |
| useContainerReveal | helper | `group()` / `group_hover` / `group_focus` の style helper | — |
| useMediaQuery | helper | `window_size()` の memo。breakpoint は sm 640、md 768、lg 1024、xl 1280、2xl 1536。compact は幅 md 未満か高さ 500 未満 | — |

### 9.5 付録 05（Content）

| Astryx | 種類 | Kivact | 手本（basic） |
|---|---|---|---|
| Avatar（+Status Dot） | NEW | `soft::avatar` | `avatar.rs` |
| Avatar Group（+Overflow） | NEW | `soft::avatar_group` | `avatar.rs` |
| Thumbnail | NEW | `soft::thumbnail` | `avatar.rs`、`primitives.rs`（`aspect_ratio`） |
| Empty State | NEW | `soft::empty_state` | `empty.rs` |
| VisuallyHidden | NEW | `soft::visually_hidden`（名前と説明だけを支援技術に出す） | — |
| Text | NEW | `soft::text`（`TextType` の全 type） | `typography.rs` |
| Heading | NEW | `soft::heading`（level 1〜6、display 1〜3） | `typography.rs` |
| Blockquote | NEW | `soft::blockquote` | `typography.rs` |
| Citation | NEW | `soft::citation` | `hover_card.rs` |
| Code | NEW | `soft::code`（inline） | `typography.rs`（`inline_code`） |
| Icon | NEW | `soft::icon`（size と color を付けた `kivact::control::icon`） | `icons.rs` |
| Code Block | NEW | `soft::code_block` | `code_block/` |
| Markdown | NEW | `soft::markdown` | `typography.rs`、`code_block/` |
| Timer | NEW | `soft::timer` | — |
| Timestamp | NEW | `soft::timestamp` | `date_picker.rs`（書式） |
| Utilities（Theme、useTheme、MediaTheme、SyntaxTheme、InternationalizationProvider、useTranslator、useClipboard、useStreamingText など） | MERGED / helper / 不要 | 付録 05 の 18 章の判定どおり | — |

### 9.6 付録 06（Table・List・Chat）

| Astryx | 種類 | Kivact | 手本（basic など） |
|---|---|---|---|
| List (+List Item, useListFocus) | NEW | `soft::list`（静的な `Role::List`）。**`kivact::ui::list`（listbox）の restyle にしない**。listbox の見た目は Selector・Checkbox List・Radio List 用に別途 restyle する | `item.rs`、`typography.rs`（`bullet_list`） |
| Item | NEW | `soft::item`（menu・select・list・command_palette の item の recipe もこの寸法） | `item.rs` |
| Metadata List (+Item) | NEW | `soft::metadata_list` | `field.rs`、`item.rs` |
| Overflow List | NEW | `soft::overflow_list`（測定は `node_ref().rect()`） | `breadcrumb.rs`、`pagination.rs` |
| Tree List (+useTreeFocus) | NEW（暫定）→ RESTYLE | 中核に `kivact::ui::tree` を足し（C-28）、soft はその restyle にする。入るまでは `use_tree` の上のキットの部品 | game の `tree`、creative の `outliner` |
| Table（+Body、Cell、Footer、Header、Header Cell、Row） | RESTYLE + NEW | データで駆動する表は `kivact::ui::data_table` の restyle。子部品で組む静的な表は `soft::table` 系の NEW | `table.rs` |
| Table Selection Toolbar | NEW | data_table の `.toolbar(ctx)` slot に置く `soft::table_selection_toolbar` | `button_group.rs` |
| useTable* の 15 の hook | data_table の機能 | 既存の機能（単一列の並べ替え、全体の filter、前後のページ、checkbox の選択、列の幅の変更、列の表示切替）は restyle。足りない機能は C-29 | 付録 06 の 7 章の機能表 |
| Chat Layout (+Scroll Button) | NEW | `soft::chat_layout`。末尾への追従は C-30 | `message_scroller.rs` |
| Chat Message List | NEW | `soft::chat_message_list`（`Role::Log` は C-31） | `message_scroller.rs`、`chat.rs` |
| Chat Message（+Bubble、Metadata） | NEW | `soft::chat_message` | `chat.rs`（`message`、`bubble`） |
| Chat System Message | NEW | `soft::chat_system_message` | `primitives.rs`（`separator`） |
| Chat Tool Calls | NEW | `soft::chat_tool_calls`（開閉は `use_disclosure`） | `trace.rs`、`disclosure.rs` |
| Chat Tokenized Text | NEW | `soft::chat_tokenized_text`（行内の token は C-15） | `badge.rs` |
| useStreamingText | helper | `soft::use_streaming_text`（Astryx のコードの値: 1 tick に 10 文字） | — |
| Chat Composer | NEW | `soft::chat_composer` | `input_group.rs` |
| Chat Composer Input | RESTYLE + wrapper | `kivact::ui::text_area` + wrapper。行内の token は C-15（代替: 入力欄の上に chip の行） | `input_group.rs`、`combobox.rs` |
| Chat Composer Drawer | NEW | `soft::chat_composer_drawer`（`use_disclosure`） | `disclosure.rs`、`attachment.rs` |
| Chat Composer Token Element | NEW | `soft::chat_composer_token`（chip で代替） | `badge.rs` |
| Chat Dictation Button | NEW | `soft::chat_dictation_button`（音声の取得と認識はアプリ。button は状態の値だけを受け取る） | `button.rs` |
| Chat Send Button | RESTYLE + wrapper | `kivact::ui::button` + `soft::chat_send_button` | `button.rs` |

### 9.7 すべての部品に共通する作業

1. 付録の該当節の 8 項目（概要・Anatomy・API・見た目・モーション・振る舞いと a11y・Kivact 実装方針・受け入れ確認）をすべて満たす。
2. 公開する builder は `NodeBuilder` と `IntoView` を実装し、自分の style と semantics を最初に入れる（呼び出し側の modifier が勝つ）。option のない部品は `Node` を返す。
3. 役割・名前・状態・値を支援技術に出す（付録の「振る舞いと a11y」）。キーボードだけで操作できる。
4. 文言は `soft.*` の key で読む（4.6 節）。
5. 方向（RTL）で左右が入れ替わる部品は、`kivact::view::provide_direction` の方向に従う。
6. カタログのページ（12.1 節）と consumer 試験（11 章）を足す。
7. light と dark の両方で 10 章の確認を行う。

---

## 10. 見た目とアニメーションの調整手順

Codex は見た目の調整が苦手なので、**目で「似ている」と判断せず、数値で確かめる**手順を取る。

### 10.1 数値をコードに入れる規則

1. 付録の値は**そのまま** tokens の field、または部品の定数（名前付き）にする。値を丸めたり、「見やすい」値へ変えたりしない。
2. 一つの部品の中で、同じ意味の値を二回書かない（例: button の高さを recipe と部品の両方に書かない）。
3. 付録が state ごとの値を示していれば、**すべての state**（rest、hover、pressed、focus-visible、selected / checked、open、disabled、invalid、loading）を実装する。付録にない state を作らない。
4. light と dark で値が違うものは、両方を tokens に持つ。dark の値を light から計算しない。

### 10.2 撮影して確かめる（lookbook）

1. カタログに**撮影用の試験**（`examples/kivact-ui-soft/tests/lookbook.rs`）を作る。全ページを light と dark で開き、`screen.screenshot().save_png(..)` で `target/soft-lookbook/<page>-<mode>.png` に保存する。state を見せるため、hover・押下・フォーカスを `kivact::test` の操作で作った画像も撮る。
2. この試験は `#[ignore]` にし、**画像を比較しない**（golden を作らない。CONTRIBUTING の規則）。人と Codex が見るための道具である。撮った画像は commit しない。
3. 撮った画像で、次を確かめる。
   - **寸法**: 付録の高さ・padding・gap・角丸・線の太さ。`element.screenshot()` で要素だけを切り出すと測りやすい。
   - **色**: `shot.pixel(x, y)` で fill・線・文字の代表の画素を読み、付録の値と比べる（半透明の overlay は、下の面の色と合成した値になる。`tokens.md` 2.8 節）。
   - **文字**: Figtree が使われているか（登録漏れで sans-serif になっていないか）、行高、weight。
   - **dark**: 面の明暗の順（5 章の 4）、影の白い縁（5 章の 9）。
4. サブエージェントは、完了報告に「確かめた項目と結果」を書く。主エージェントは画像を開いて確かめる。

### 10.3 アニメーションを確かめる

1. 時間・曲線・移動量は、付録の数値を名前付きの定数にして、コードで直接確かめられるようにする。
2. `RenderOptions::frame_period_ns` と時間の進め方を使い、入場の**途中の frame**（例: 時間の 20% の時点）を撮る。`cubic-bezier(0.24, 1, 0.4, 1)` なら 20% の時点で約 64% 進んでいるはずである。これで曲線の取り違え（`EaseOut` の多項式など）を見つける。ただし試験として画素や時間を厳密に比べない（10.2 節の 2）。
3. reduced motion（`RenderOptions::reduced_motion`）で、即時になるもの、fade になるもの、Spinner が 3 秒になることを確かめる。
4. 付録にないアニメーションを足していないか確かめる（6.1 節の 5）。

### 10.4 Astryx との比較

- 参照のスクリーンショットが必要になったら、作者に「この部品のこの state の Astryx の画像」と具体的に頼む。Codex 自身は Astryx のコードを実行しない（0.3 節）。
- 作者が最後に、カタログと Astryx の docsite を並べて見比べる。指摘は付録の値の誤りか、実装の誤りかを切り分け、付録の誤りなら付録を直す。

---

## 11. 試験

`foundation/kivact-kit-architecture.md` の 9 章を読む。

1. `crates/kivact-ui-soft/tests/consumer.rs` と `tests/consumer/*.rs` に、**`kivact` と `kivact_ui_soft` だけを import する**実ウィンドウの E2E を書く。
2. 部品ごとに、付録の「受け入れ確認」のうち**利用者の契約**になるものを試験にする: 役割と名前で見つかる、キーボードで操作できる、状態（checked、expanded、selected、value）が支援技術に出る、caller の signal が更新される、callback が呼ばれる、文言の key を差し替えられる、テーマを light と dark に切り替えても再 mount せずに見た目が変わる。
3. レイアウト崩れ（文字のはみ出し、意図しない折り返し、縦位置のずれ）の検査を、文字を持つ部品のカタログのページにかける。
4. 足さないもの: golden、画素や時間の厳密な比較、過去の不具合の再現だけの試験。
5. `dev/tests/kivact-public-api/tests/e2e/theme_conformance.rs` に Soft を足す（`KIT_RECIPES` の長さを 5 に、`themes` に `("Soft", kivact_ui_soft::theme)`）。12 種の標準部品が既定の見た目と違うこと、button のフォーカスのリング（C-01 の後）を確かめる。
6. カタログの試験（`examples/kivact-ui-soft/tests/catalog.rs`）で、全ページが開くことと、light / dark の切替を確かめる。
7. ウィンドウを開くコマンドは `dev/tools/window-lock.sh -- <command>` を通す。

---

## 12. カタログ・README・利用者ドキュメント

### 12.1 カタログ（`examples/kivact-ui-soft`）

- basic のカタログ（`examples/kivact-ui-basic`）と同じ構成にする: `main.rs`、`pages.rs`、`page.rs`、`docs/`、`shell.rs`、`look.rs`、`model.rs`、`fonts.rs`、`tests/catalog.rs`。
- Astryx の docsite の分類（Action、Chat、Container、Content、Feedback & Status、Form Controls、Layout、Navigation、Overlay、Table & List、Utility）でページを並べる。
- 各ページに、Astryx の docsite のその部品のページにある主な example（variant・size・state の並び）を同じ構成で置く。各 example の下にソースを示す（basic と同じ）。
- light / dark の切替と、reduced motion の切替をシェルに置く。
- Figtree を同梱して登録する（4.3 節）。

### 12.2 README（`crates/kivact-ui-soft/README.md`）

basic の README と同じ節を持つ:
1. キットの説明（Astryx の Neutral テーマ）と最小の例。
2. 標準部品の restyle の一覧と、Astryx の variant・size との対応（4.2 節）。
3. **Astryx の部品 → soft の API の対応表**（2.1 節の全行。hook は「部品の機能」「Kivact の仕組み」「helper」「不要（理由）」のどれか）。
4. テーマ、tokens、フォント、icon の使い方。
5. 文言の key と英語の既定値の表（4.6 節）。
6. カタログと試験。
7. 中核に足りないため代替で実装している点（7.2 節の C の項目）と、その見た目の違い。

### 12.3 利用者ドキュメント

- `docs/for-kivact-ui-user/en/soft/` と `docs/for-kivact-ui-user/jp/soft/` に、basic と同じ構成で `getting-started/`（キットの設定、フォームを作る、dark mode とフィードバック）と `how-to-guide/`（forms、navigation、overlays、data-display、theme-and-tokens、motion）を作る。**英語と日本語を同じファイル名の対にし、同じ変更で両方を書く。**
- `docs/` の Rust のコードブロックは `dev/tests/docs` で doctest としてコンパイルされる。`dev/tests/docs/Cargo.toml` に `kivact-ui-soft` を足す。
- 次の既存の文書を更新する（`kivact-kit-architecture.md` 1.10 節と 1.11 節）: `docs/for-kivact-ui-user/{en,jp}/README.md`、`docs/for-kivact-user/{en,jp}/README.md`（「four kits」）、`docs/for-kivact-user/{en,jp}/how-to-guide/theming.md`、`README.md`、`README_JP.md`、`examples/README.md`、`THIRD-PARTY-NOTICES.md`。

---

## 13. 完了条件

### 13.1 フェーズの完了確認（各フェーズの最後）

```text
cargo fmt --all -- --check
cargo xtask check
cargo clippy --workspace --all-targets --all-features -- -D warnings
dev/tools/window-lock.sh -- cargo test -p kivact-ui-soft
dev/tools/window-lock.sh -- cargo test -p kivact-ui-soft-catalog
dev/tools/window-lock.sh -- cargo test -p kivact-public-api-tests --test e2e
```

中核を変えたフェーズでは、既存の四つのキットとそのカタログの試験も通す。

### 13.2 merge 前の検査（CONTRIBUTING.md）

```text
cargo fmt --all -- --check
cargo xtask check
git diff --quiet -- architecture
cargo clippy --workspace --all-targets --all-features -- -D warnings
dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --all-features
dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --no-default-features
dev/tools/check-features.sh
```

`architecture/` は `cargo xtask check` が生成するので、生成し直したものを commit する。

### 13.3 完了の定義

1. 2.1 節の表の**すべての行**が、README の対応表で soft の API（または「部品の機能」「Kivact の仕組み」「不要（理由）」）に対応している。
2. 各部品が付録の 8 項目を満たし、カタログのページと consumer 試験がある。
3. light と dark の両方で、10 章の確認を全ページで行った。結果（確かめた項目、付録の値との差、残した差とその理由）を PR の説明に書く。
4. 付録のアニメーションがすべて入っている。7.2 節の C の項目で代替にしたものは、README と PR の説明に書いてある。
5. `TODO(soft-core: C-xx)` の印は C の項目だけに残っている。
6. 13.2 節の検査がすべて通る。
7. 既存の四つのキットの試験が通り、見た目が変わっていない。
8. PR の説明に、具体的な問題、変更後の挙動、実施した検証、未実施の理由、残る制限を書く（CONTRIBUTING.md）。

### 13.4 commit

- 目的ごとに conventional commits で分ける（`feat(soft): …`、`feat(style): …`、`docs(soft): …`、`test(soft): …`）。
- 中核の変更とキットの変更を同じ commit に混ぜない。
- 本書と付録（`SOFT_UI_WORK_ORDER.md`、`soft-ui-work-order/`）は作業中に直してよい。merge の前に残すか消すかは作者が決める。

---

## 14. 付録の一覧

| ファイル | 内容 |
|---|---|
| `soft-ui-work-order/foundation/kivact-kit-architecture.md` | Kivact の公式キットの作り方（crate の骨格、登録するファイル、テーマと recipe、標準部品、style とアニメーションの能力、icon、文言、フォント、試験、カタログ、ドキュメント、命名、merge 前の検査、basic の shadcn の写し方） |
| `soft-ui-work-order/foundation/tokens.md` | Neutral テーマの全トークンの解決値（light / dark）と `Tokens` 構造体の案 |
| `soft-ui-work-order/foundation/motion.md` | モーションの全抽出（role、spring、gesture、reduced motion、部品ごとのアニメーション、Kivact での実現方法と不足） |
| `soft-ui-work-order/components/01-action-feedback.md` | Button、Button Group、Icon Button、Toggle Button（+Group）、Link、More Menu、Segmented Control、Toolbar、Badge、Banner、Progress Bar、Skeleton、Spinner、Status Dot、Indicator、Kbd、Token と、押下・フォーカス系の hook |
| `soft-ui-work-order/components/02-form-controls.md` | Text Input、Text Area、Number Input、Field 群、Input Group、File Input、Checkbox、Radio、Switch、Slider、Selector 群、Typeahead、Tokenizer、Power Search、Calendar、Date / Time Input 群、Rich Text Editor |
| `soft-ui-work-order/components/03-overlays.md` | Dialog、Alert Dialog、Drawer、Bottom Sheet、Popover、Hover Card、Tooltip、Toast、Command Palette、Dropdown Menu、Context Menu、Lightbox、Overlay、Layer、配置の規則 |
| `soft-ui-work-order/components/04-navigation-layout-container.md` | App Shell、Layout 群、Stack、Grid、Section、Form Layout、Aspect Ratio、Divider、Resizable、Scrollable Area、Breadcrumbs、Outline、Pagination、Side Nav、Top Nav、Mobile Nav、Stepper、Tabs、Card 群、Carousel、Collapsible |
| `soft-ui-work-order/components/05-content.md` | Avatar 群、Thumbnail、Empty State、VisuallyHidden、Text、Heading、Blockquote、Citation、Code、Icon、Code Block、Markdown、Timer、Timestamp、Utilities と全文言の一覧 |
| `soft-ui-work-order/components/06-table-list-chat.md` | List、Item、Metadata List、Overflow List、Tree List、Table と全 hook の機能、Chat 群 |
