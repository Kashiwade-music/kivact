# Kivact と Arvane の設計

この文書は、crate をまたぐ規則と全体の契約・不変条件・理由をまとめる。
設計が変わったら同じ変更でここを直す。
一つの module の中で閉じる振る舞い（各部品の key 操作、text editing の細部、paint・Arvane の adapter の内部など）は、
それを実装する module の `//!` doc と公開 item の rustdoc に書き、ここには書かない。
実装経緯と検証記録は commit 履歴と PR を参照する。
アプリ作者向けの使い方は [利用者ガイド](../for-kivact-user/getting-started/README.md)、
コードの規則は [コーディング規則](coding-rules.md)、crate の依存の実際は自動生成の [アーキテクチャ図](../../architecture/crates.svg) にある。
公開 API と全体の設計は、まだ大きく変わりえます。

## 1. 全体像

Kivact は、SolidJS の実行モデル（setup は一度だけ、細粒度の signal、差分だけの更新）を proc macro なしの Rust で書く native GUI framework です。
core が retained tree を持ち、変わった prop だけを turn の中で engine（`kivact-engine`）の木へ書き込んで revision ごとに commit する。
engine はその木の上で layout・入力・編集・描画・accessibility を担い、単一の順序付き event の列で答える。
Arvane は、その engine の下で window・text・描画・accessibility の platform の違いを受け持つ自前の層です。

```text
アプリ: fn(args) -> impl IntoView（普通の関数。呼出し元の setup の中で一度だけ実行）
   │
任意の公式 UI（Basic/Pop/Game/Creative。kivact の公開 API だけで書く）
   │
kivact: 公開 facade（core・interaction・標準 UI を明示的に再公開する）
   │
kivact-ui: 標準 UI（core と interaction の公開 API だけを使う）
   │
   ▼
kivact-interaction: headless な振る舞い・semantics・非公開の純粋 policy
   │  公開 kivact-core だけに依存
   ▼
kivact-core: App・prelude・view / control / primitive / style / task / motion・engine への入口
   │  └ kivact-reactive（host を知らない反応の核）
   │  └ kivact-i18n（locale と messages。reactive だけに依存）
   │  └ kivact-task（reactive の上の UI thread の executor・resource・action・blocking・timer）
   │  └ kivact-motion（reactive と wire の上の frame loop・animated・transition）
   │  └ kivact-style（reactive・wire・motion・editing の上の style・theme・draw の語彙と compile の API）
   │
   │  engine::bridge の所有型 Writer（create/set_layout/set_paint/insert..、commit）・Reader
   │  ▲ HostEvent の順序付き列と commit の境界（Boundary）
   ▼
kivact-engine  node の木（prop の唯一の置き場）・focus・modal・pointer・編集・scene・accessibility・window の loop
   │  └ layout（構造の借用 adapter）→ kivact-layout（LayoutTree・LeafMeasure による計算と派生列）
   │  └ kivact-wire（独立 crate：型付き node・events・scene・measure・純粋 motion）
   │
   ▼
arvane（winit・text・render・asset）。shell の語彙と入力正規化は arvane-shell、共有の値は ui-types
```

## 2. crate と層

| crate | 役割 |
|---|---|
| `kivact` | 公開 facade。core の公開 module・`App`・handle・setup 関数を明示的に再公開する。`kivact-interaction` を `kivact::interaction`、`kivact-ui` を `kivact::ui` として再公開し、`crates/kivact/src` は `lib.rs` だけを持つ。`gpu`（既定）・`test`・実験的な `gpu-interop`（既定 off）は core へ転送する。実験的な `embed` は `core/embed` へ転送する |
| `kivact-core` | `App`、prelude と下の表の module。engine への唯一の入口。`gpu`（既定）は `engine/gpu`、`test` は `engine/testing`、`gpu-interop` は `engine/gpu-interop`（さらに `arvane/gpu-interop`）と `kivact-style/gpu-interop` へ転送する。実験的な `embed` は `engine/embed` へ転送する |
| `kivact-ui` | 標準 UI の部品（`kivact::ui`）。Button・Dialog・Menu・Select・Slider・Tabs・TextInput・List・DataTable などの標準部品と `UiRecipes`・`Look`。依存は公開 `kivact-core` と `kivact-interaction` だけで、`kivact` だけに見える。doctest だけが dev-dependency で `kivact` を使い、例を `use kivact::prelude::*;` と `kivact::ui::` の道で書く |
| `kivact-ui-basic`・`kivact-ui-pop`・`kivact-ui-game`・`kivact-ui-creative` | 独立した任意の styled kit。theme（`UiRecipes` を載せて標準部品の見た目を差し替える）と分野固有の部品を持つ。依存は公開 `kivact` だけ。kit なしでも `kivact::ui` の標準部品で最初の画面を書ける |
| `kivact-reactive` | arena・graph・owner・frame stack・turn・handle・setup の関数・error と境界・trace・App ごとの拡張枠・driver（時計と turn の要求の口）。host を知らない |
| `kivact-i18n` | `Locale`・`Messages`・`message`・`message_with`・`interpolate` と、部分木に locale と messages を与える口（`install_locale`・`install_messages`・`install_layered_messages`）。依存は `kivact-reactive` だけ。core の `i18n` module が利用者向けの名前だけを `kivact::i18n` として再公開し（`install_*` は再公開しない）、doctest だけが dev-dependency で `kivact` を使う |
| `kivact-task` | UI thread の executor・resource・action（queue の policy を含む）・blocking pool・host の時計の timer。依存は `kivact-reactive` と async-task だけ。host が App の turn を回すための driver API（`TaskState`・`TaskStats`・`Completion`・`spawn_in`・`DEFAULT_DRAIN_NS`）を持ち、core だけに見える |
| `kivact-motion` | frame loop・`Animated`・`Transition` と、App の frame の時計を持つ `MotionState`。依存は `kivact-reactive` と `kivact-wire`（`motion`）だけ。host が frame を回すための driver API（`MotionState`・`MotionStats`）を持ち、core だけに見える。doctest だけが dev-dependency で `kivact` を使う |
| `kivact-style` | `Style`・`Theme`・token・色・長さ・`Asset` と、`Draw` と描画の語彙（paint・shape・stroke・filter・transform・`Path`）。`Style::compile`（`Interactive` を受けて `CompiledStyle` を返す）、`Draw::record`（`Recording`。`set_text_measurer`・`declare_regions`）、`TextRun::from_style`、`SpanStyle::compile`、`Theme::install`・`Theme::of` が host 側の入口。依存は `kivact-reactive`・`kivact-wire`・`kivact-motion`・`kivact-editing` と kurbo（`Path` の演算。型は公開しない）で、engine・core を知らない。feature `gpu-interop`（`GpuFrame`）は style 自身の振る舞いだけを足す（wire は feature を持たない）。core の `style` module が `kivact::style` の名前だけを再公開し、`kivact::draw` の語彙は core の `draw` が再公開する。doctest だけが dev-dependency で `kivact` を使う |
| `kivact-interaction` | 見た目を持たない振る舞いと semantics（List/Tree/Drag/Popup などの controller と純粋 policy）。公開 primitive 契約で所有者・mount・focus を扱う。headless な handle は `DialogApi`・`MenuApi`・`SelectApi`・`TabsApi`・`TooltipApi`・`SplitterApi`・`ButtonApi` のように `Api` を付け、同名の `kivact::ui` の builder と区別する。app のデータ型（`ListItem`・`MenuEntry`・`Command`・`TreeNode`・`Toast`・`DialogKind`・`Side`・`Dismiss`・`EditPhase` など）は interaction が使うのでここで定義し、`kivact::ui` から再公開する（app 向けの文書と例は `kivact::ui` と prelude の path だけを使う） |
| `kivact-layout` | `LayoutTree` と `LeafMeasure` から flex/grid を計算し、slot に対応する派生列と中立な幾何差分を持つ。木・資源・engine の意味を所有しない |
| `kivact-text` | 借用した source/revision/damage から hard line を保ち、必要な行だけ shaping・編集幾何・IME 抜粋・bounded publication を作る。依存は editing・lower・ui-types・Arvane・unicode-segmentation。Editor と木を所有しない |
| `kivact-lower` | wire の値・局所 scene を中立な描画・文字 style・資源へ下げる。依存は kivact-wire・ui-types・arvane だけ。木・layout cache・window loop・公開順は持たない |
| `kivact-wire` | node・event・scene・measure と純粋 motion の独立した値の契約。依存は ui-types と kivact-editing だけ。engine と core が直接使う |
| `kivact-editing` | EditState・EditBehavior・EditLayout と、host が所有する Editor の純粋な編集契約。依存は ui-types と unicode-segmentation だけ。engine と core が直接使う |
| `kivact-engine` | `bridge`（6章）と、型付き node の木の上の layout・入力・描画・accessibility と Arvane の window の loop（7章）。実験的な `embed` は `arvane/embed` へ転送する |
| `ui-types` | Kivact にも Arvane にも属さない中立な値の crate（8章）。外部依存を持たない |
| `gpu-share` | UI を知らない中立な GPU texture 共有の契約と補助。既定（feature なし）は `Identity`・`Error` と Vulkan の版・extension・usage・format、Direct3D 12 の resource flag・LUID の規則を持つ。`export` は ash と Direct3D 12 の pool・timeline fence・barrier・adapter の照合・時間切れ付き待機・RGBA8 の読み戻しを公開する。`import` は `import` module に wgpu への取り込みと acquire/release を公開する。`wgpu` は `export` を含み、同じ版の wgpu（`=29.0.4`）で書いた engine に device・texture pool・submit・読み戻しを提供する。feature がなければ対応する API は存在しない。アプリの engine が直接依存でき、`kivact`・Arvane に依存しない |
| `arvane*` | 8章 |
| `kivact-public-api-tests` | 公開 API の統合試験と trybuild。公開 `kivact` と必要な公式kitの利用者であり、どの product crate も依存しない |
| `kivact-external-component`・`kivact-ext-apps`・`kivact-workbench`・例・雛形 | 公開 API だけで作った外部部品と、それを使うアプリ |
| `kivact-docs` | `docs/` と README の rust の code block を doctest にする |
| `xtask` | `cargo xtask check`（xtask-core によるコーディング規則と `architecture/` の図） |
| `kivact-workspace-policy-tests` | 依存の許可リストと公開名の一覧の試験 |

`kivact-core` の module（`src/<module>.rs` と `src/<module>/`。公開 path は facade が同じ形で再公開する）:

| module | 役割 |
|---|---|
| `i18n` | 再公開だけ。`kivact-i18n` の利用者向けの名前（`Locale`・`Messages`・`Weekday`・`locale`・`message`・`message_with`・`interpolate`）を `kivact::i18n` にする。部分木に与える `install_*` は再公開しない（利用者は `view::provide_locale`・`view::provide_messages` を使う） |
| `task` | 再公開だけ。`kivact-task` の利用者向けの名前（`spawn`・`blocking`・`sleep`・`timeout`・`resource`・`action` と `Resource`・`Action`・`AsyncState` など）を `kivact::task` にする。host の driver API は再公開しない |
| `motion` | 再公開だけ。`kivact-motion` の利用者向けの名前（`frame_loop`・`animated`・`Animated`・`FrameLoop`・`Transition`・`Easing`・`Spring` など）を `kivact::motion` にする。host の driver API は再公開しない |
| `style` | 再公開だけ。`kivact-style` の `kivact::style` の名前（`style`・`Style`・`Color`・`Theme`・`Length` など）と、`kivact::style::theme` の token 表を再公開する。`Style::compile`・`Draw::record` などの host 側の入口は再公開せず、core の client・view・control・primitive が `kivact_style` から直接使う。`kivact::draw` の語彙は `draw` が `kivact_style` から再公開する |
| `view` | 記述子と mount・retained tree・style の唯一の経路（`bind_style`）・App の root・`provide`・`provide_theme`・`provide_locale`・`provide_messages` の境界・component の `Owner`・`Role`/`A11yAction`（engine の型の再公開）・要素の型 `Node` と modifier の trait `NodeBuilder`（`view::node`。実装は `view` の `overlay`・`virtual_list`・`Part` が持つので `view` に定義し、`control` が再公開する）・`Span`・overlay の builder と modal の復帰要求（modal の判定は engine）・入力型・`NodeRef`・host への要求の状態（`view::requests`・`view::state`） |
| `view::flow`（非公開。`view` が再公開） | show/for_each/for_index/dynamic/error_boundary・`virtual_list` |
| `control` | 要素の型 `Node`（node・canvas・image・text・rich_text・container が返す）と、`Node` と部品の builder の modifier の trait `NodeBuilder` の公開 path（定義は `view::node`。公開 module が実装を持つ原則の、view の builder が `Node` を使うための唯一の例外）。`Node` を作る関数と container を置く。見た目を持つ部品は置かない |
| `primitive` | **実験的。** 部品作者の所有者・node query の契約（`Owner`・`Owned`・`NodeToken`・`NodeRefExt`）、native なテキスト編集の契約、計測の拡張口（`primitive()`・`Constraints`・`Measurement`）、modifier の trait `NodeExt`、部位 `part()` と `Part`・`PartState`・`PartMount`（定義は `view::part` にあり、公開は `primitive` だけ）、located diagnostic を投げる `raise` |
| `client`（非公開） | `AppClient` が具体的な `bridge::Client` を実装する。App の組立、host の時計と waker、turn、event と mount の配送 |
| `test`（feature `test`） | `kivact::test`。engine の試験 driver の上の、実 window の Testing Library（10章） |
| `draw` | 描画の語彙の再公開と、window なしで固定解像度の RGBA 画像へ描く `Offscreen`（engine の `bridge::Offscreen` への入口） |
| `gpu_interop`（feature `gpu-interop`） | **実験的。** 外部の render engine が共有した GPU texture の取り込み（低水準の `adapter`・`import`・`ExternalImage`・`FramePublisher`・`ImportStatus`・`GpuFrame`、高水準の `shared_image`・`SharedImage`・`ShareLink`・`ShareRequest`・`ShareMode`・`ShareStatus`・`ShareError`）と ui-types の語彙の再公開だけ。実装は `gpu_interop::import`（取り込み・状態・publisher）と `gpu_interop::share`（要求と再取り込み）に持ち、per-App の状態は client が runtime の extension に入れる。feature の cfg は `lib.rs`・`client`・`gpu_interop` と `kivact-style` の `draw`（`GpuFrame` と `Draw::gpu_frame`）に限る |
| `platform` | host の環境の読み取り（`platform::env`。`window_size`・`scale_factor`・`capabilities` など）と service の要求（`platform::services`。clipboard・file dialog・`open_path`・`set_fullscreen`）の実装、engine への入口 `run_native` |
| `diagnostics` | `why` の連鎖の実装（`diagnostics::why`）、`Stats`、reactive の trace の名前の再公開 |
| `prelude` | 再公開だけ |

共有する値と状態は、それを操作する実装より下に置く。
view の記述子と mount context は `descriptor` に、環境・focus・要求の記録は `state` に集める。
task・motion・i18n も状態や値の定義から操作側を参照しない。
`kivact-style` の中は、値（geometry・色・文字・shape・paint）を下に、`Theme` をその上に、`Style` と `Draw` をさらに上に置き、関心ごとに一つの module にまとめる（`[module_rules.kivact-style]`）。
値の engine の形への変換は値の module に置き、Theme で解決する変換も Theme が参照しない値（文字・paint・effect）ならそこに置く。
Theme が使う色の解決は `theme::value`、`Style` の setter は `sheet`、`Draw` の描画 method は `draw`、`Style` 全体の解決は `resolve` に置く。
別 module の型に対する inherent impl も操作側に置き、値の定義から上位の処理への依存を作らない。
kit も共有値と操作を分ける（Creative の timeline/outliner/editor の値は各 data module、dock の状態は dock/data）。
子は定義元を明示的に import し、親の glob や再公開を経由して循環を作らない。
公開 path は明示的な再公開で保ち、同じ処理を呼び直す互換 wrapper は置かない。

### 依存の向きと visible_to

`dev/xtask/policy/workspace.toml` の `visible_to` が「誰がその crate に依存してよいか」を決め、
`cargo xtask check` が `cargo metadata` の実際の辺（normal・dev・build）と照合する。
書いていない crate は誰からでも依存できる。意図は次のとおりです。

- **最小の例と雛形は `kivact` だけ、styled 部品が必要な consumer は公式 kit の公開 API へ依存する。** `kivact-reactive` は `kivact-task`・`kivact-i18n`・`kivact-motion`・`kivact-style`・`kivact-core` だけに、`kivact-task`・`kivact-i18n`・`kivact-motion`・`kivact-style` は `kivact-core` だけに見える。
- **公式 UI の `kivact-ui-basic`・`kivact-ui-pop`・`kivact-ui-game`・`kivact-ui-creative` は `kivact` だけに依存する。**
  `dev/tests/workspace-policy` の許可リストが、外部 package・normal/dev/build・target 別の直接依存も制限する。
  renderer はアプリが選び、部品の実装に engine/wire/layout/reactive・Arvane・private module を使わない。
  workspace 外の独立 consumer（`dev/tests/fixtures/*`）でも同じ公開 API を使って構築・操作する。
- **`kivact-interaction` は `kivact` と `kivact-ui` だけに見える。** アプリと公式 UI は公開 `kivact::interaction` を通す。
  通常依存は `kivact-core` だけで、公開 API のみを Cargo で強制する。
  内部の純粋 policy は独立した計算として保ち、focus、capture、modal と入力順序の既存 authority を複製しない。
- **`kivact-ui` は `kivact` だけに見える。** アプリと公式 kit は公開 `kivact::ui` を通す。依存は `kivact-core` と `kivact-interaction` の公開 API だけ。
- **`kivact-task` は `kivact-core` だけに見える。** 依存は `kivact-reactive` と async-task だけ。core の `task` module が利用者向けの名前だけを再公開し、host の driver API は core の client・view・test が `kivact_task` から直接使う。
- **`kivact-motion` は `kivact-core` だけに見える。** 依存は `kivact-reactive` と `kivact-wire` だけ。core の `motion` module が利用者向けの名前だけを再公開し、host の driver API は core の client・view・test が `kivact_motion` から直接使う。
- **`kivact-style` は `kivact-core` だけに見える。** 依存は `kivact-reactive`・`kivact-wire`・`kivact-motion`・`kivact-editing` と kurbo だけで、engine を知らない（text の測定は host が `Draw::set_text_measurer` で渡す）。core の `style` module が `kivact::style` の名前だけを再公開し、host 側の入口（`Style::compile`・`Draw::record`・`TextRun::from_style`・`SpanStyle::compile`・`Theme::install`・`Theme::of`）は core の client・view・control・primitive が `kivact_style` から直接使う。
- **`kivact-core` は `kivact`・`kivact-interaction`・`kivact-ui` だけに見える。** facade の再公開と headless な振る舞い・標準 UI 以外は core へ直接依存しない。
- **engine を知るのは `kivact-core` だけ**（`App::run`、window なしの `kivact::draw::Offscreen`、
  feature `test` で engine の試験 driver（feature `testing`）に立つ `kivact::test`）。
  具体的な書込み・読取りは `kivact_engine::bridge` に置く。
- **編集の契約は `kivact-editing`。** wire・text・style・engine・core に見え、core の primitive と view が従来の behavior-facing な名前を再公開する。Editor は host の所有型で、core は再公開しない。
- **wire と layout の計算は独立 crate。** core は kivact-wire の語彙と bridge の所有型 Writer/Reader を直接使う。kivact-wire は lower・layout・engine・motion・style・core に見える。engine の `layout` は構造を借りる adapter で、`kivact-layout` が計算と派生列の型を持ち、Model がその列を所有する。
- **`arvane` を使うのは `kivact-engine`、値変換の `kivact-lower`、文字の `kivact-text`。** Arvane の adapter（`arvane-native`・`arvane-text`・`arvane-render`・`arvane-asset`）は `arvane` だけに見え、
  lower は text と engine に見え、Arvane の型を engine の bridge に漏らさない。adapter 同士では依存せず、`ui-types` の値と `arvane-shell` の語彙・入力正規化を共有する。
  `arvane-shell` は `arvane` と `arvane-native` だけに見える。`arvane*` は Kivact の crate に依存しない。
  GPU texture 共有の `unsafe` は中立な `gpu-share` に閉じる。`arvane-render` はその `import` feature を使い、アプリの engine と試験用の `gpu-interop-producer` は `export`（wgpu の engine は `wgpu` も）を使う。
  `gpu-share` は visible_to を持たず、アプリの engine が直接依存できる。試験用の `gpu-interop-producer` は Arvane の契約の試験・公開 API の試験・creative kit の GPU frame の試験（dev-dependency）だけに見える。
- **共通の値は `ui-types` に一度だけ定義する**（8章）。`kivact-engine` はこの値の crate を直接参照する。
  `kivact-reactive` も `IntoValue` の定数実装だけに参照する（Rust の orphan rule により、別 crate の trait と共有値の両方に対する実装を core に置けないため）。
- **アプリ・ツールには誰も依存しない**（`visible_to = []`）。

`kivact-core` と facade の約束:

- module の依存は下向きに保つ。core 内は `kivact-reactive ◀ task, motion ◀ style ◀ view, draw ◀ control ◀ primitive ◀ prelude ◀ test`
  と `{task, motion, style, view} ◀ platform, diagnostics, gpu_interop ◀ client ◀ prelude`、crate 間は `kivact-reactive, kivact-wire ◀ kivact-motion`、`kivact-reactive, kivact-wire, kivact-motion, kivact-editing ◀ kivact-style`、`kivact-reactive ◀ kivact-task, kivact-i18n`、`{kivact-task, kivact-i18n, kivact-motion, kivact-style} ◀ kivact-core ◀ kivact-interaction ◀ kivact-ui ◀ kivact`。
  境界の関数は `view` が定義元の `view::boundary` から再公開する。
  i18n の値と context（`Locale`・`Messages`）は `kivact-i18n` に置き、部分木に与える境界（`provide_locale`・`provide_messages`）は view に置いて、`kivact-i18n` の `install_*` を呼ぶ。
  component の資源の所有者 `Owner`（reactive の `Owner` を包む newtype）と `app_state` は `view::owner` に置き、`primitive` が再公開する。
  `draw` は style の語彙を公開する入口。`platform`・`diagnostics`・`gpu_interop` は env・platform の要求・`why`・GPU の取り込みの実装を自分で持ち、`view::state`・`view::requests` の上に立つ（`view` はこれらを参照しない）。`prelude` は再公開だけで、中核の module からは参照しない。
- core 内の向きは policy の `[module_rules.kivact-core]` に最上位の全 module について書き、`cargo xtask check` が module の参照と照合する。
  `kivact-interaction` も部品ごとに組み合わせてよい部品を `[module_rules.kivact-interaction]` に書く
  （純粋な `policy` と `disclosure`・`list`・`drag`・`edit`・`focus_return`・`roving` などの上に `popup`・`menu`・`select`・`table`・`palette` を組む）。
  `kivact-engine` も `tree ◀ measure ◀ model（interact）◀ paint・a11y ◀ check ◀ host ◀ bridge・native` の向きを
  `[module_rules.kivact-engine]` に書く。`layout` は構造の借用 adapter、`mutation` は受動的な書込み記録とし、
  状態を持つ葉への参照だけを理由付きの `except` にする。`impl Model` の場所も syn の近似検査で制限する。
- keyed な一覧の差分は `kivact_reactive::keyed`（`diff`・`Rows`）に置き、`view::flow` の一覧と `view` の region の集まりが共有する。
  task と motion は互いに独立で、どちらも時計を `Runtime::now_ns` で読む。
- App ごとの状態は、それを持つ module の型（`TaskState`（`kivact-task`）・`MotionState`（`kivact-motion`）・`ViewState`・`HostState`）として拡張枠に置き、
  `Runtime::ext::<T>()` で取り出す。組み立てるのは `AppClient::new` の1か所だけ。
- `pub` は公開 API だけ。module の間の内部 API は `pub(crate)`（子 module も `pub(crate) mod`）で、
  `#[doc(hidden)]` の公開（封印した trait の method を除く）は使わない。公開の値型を定数 prop として登録する `constant_value!` だけは `#[macro_export]` の狭い例外とし、内部の modifier/helper macro は公開しない
  （modifier の method は macro を使わず、`NodeBuilder`・`NodeExt` の既定 method に一度だけ書く）。
  公開する型に内部のための `pub` method を足さない。`kivact::test` だけが読む内部の口は `#[cfg(feature = "test")]` にする。
- 公開 module が見せるのは、そこで定義した `pub` の item と明示の `pub use` だけ（glob で再公開しない）。
  facade prelude の core glob は、既存の単一定義をそのまま再公開する例外とする。
- 公開 API の統合試験と trybuild は `dev/tests/kivact-public-api`、個別の最小アプリは `examples/kivact` に置く。
  core から facade や公式 kit への dev-dependency は持たず、normal/dev/build の workspace 依存を同じ向きに保つ。

`dev/apps/embed-host` は手動確認用の consumer で、依存は facade の kivact（embed・test）だけ。native の型とプラグインの枠組みには依存しない。

### renderer feature の選択

`kivact` の直接利用では既定の `gpu` が選べる。
workspace の consumer はすべて `kivact` を `default-features = false` で参照し、
実行アプリ・雛形・docs・E2E が自分の既定 `gpu` feature を `kivact/gpu` へ転送する。
部品ライブラリ（公式 kit・外部部品）は renderer を選ばない。`--no-default-features` の CPU build に部品の dev 依存から GPU が混ざらない。
実行時の `ARVANE_RENDERER=cpu` は GPU を含む binary で CPU を選ぶ別の操作である。

実験 feature `gpu-interop`（GPU texture 共有）は既定で off で、`kivact` → `kivact-core` → `kivact-engine` → `arvane/gpu-interop` と転送し、どれも `gpu` を含む。
feature の無い build には `kivact::gpu_interop`・`App::gpu_preference`・`Draw::gpu_frame`・`RenderOptions::gpu_preference` と、engine の対応する書込み・event・scene の命令が一切ない（trybuild の `tests/ui/fail_without_gpu_interop`）。
Windows と Linux 以外では feature を有効にしても build は通り、`adapter()` は常に `None`、取り込みは `Unsupported` で失敗する（Arvane が取り込まない）。

## 3. 公開 API の形

### 実験 feature `embed`

`embed` は既定 off で、facade から core・engine・Arvane・native へ転送する。`testing` は有効にしない。
`App::embed` はホストの GUI thread から呼び、`Parent` に親の raw handle と任意の倍率を渡す。
`ui-types::ParentWindow` は `kivact-wire` が再公開し、core は wire の値を使う。
子 window の SPI は `arvane-shell::ShellChild`、UI thread で一度だけ handler を作る口は `MakeHandler`。
`App::run` と `App::embed` はサイズの検査と client の組立てを共有し、`App` は設定を再利用するため Clone。
共通契約、backend の境界、対応状況は[8章の埋め込み](#埋め込み)に置く。
Parent の resize callback は物理 px、platform::request_window_size は論理 px。wrapper は保存値の更新→ホストへの要求→拒否時の復元を守る。

### 公開名と prelude

facade の root・全公開 module とその公開の子 module の名前は、feature 条件の item も含めて `dev/tests/workspace-policy/public-names.golden.md` で固定する。method と From 実装は数えない。
数は上限ではなく、名前の増減は golden の差分で review する。
prelude は標準部品の必須引数の型（`ListItem`・`MenuEntry`・`ButtonVariant`）と向きの `Orientation` も含み、
`use kivact::prelude::*;` だけで各部品を呼べる（trybuild の `prelude_only.rs`）。
prelude は、例が頻繁に名指しする `Align`・`Justify`・`pct`・`icon`・`View` も含む。通常の部品は `impl IntoView` を返し、`View` は分岐の arm の型をそろえるときに使う。
examples・dev/apps・templates の crate は `redundant_imports` を warn にして、prelude で届く名前の再 import を防ぐ。
描画・専門サービス・部品作者の口は定義元の module から明示 import し、内部 ID/runtime の記録を公開 state に混ぜない。
`kivact::interaction` は部品作者の層。一覧と表は標準部品の `kivact::ui::list` と `kivact::ui::data_table` があるので、アプリ向けのガイドは `ui` の経路を案内し、`use_list`・`use_table`・`use_table_model` の説明は部品作者の手引きに置く。データ型は `kivact::ui` から使う。
app の課題で要る interaction の型（`DialogKind`・`Side`・`Dismiss`・`ScalarRange`・`ListActivation`・`EditPhase`・`Toast`・`Toaster`・`toaster`・`Command`・`TreeNode`）は `kivact::ui` が再公開する。

### 部品と callback（D5）

部品は引数から `impl IntoView` を返す普通の関数で、呼出元の setup で一度走る。
関数自体は owner を作らず、状態と文脈は frame stack の最寄り owner に属する（4章）。
公開 fn・method が callback を受ける口は、引数なしを `impl Fn() -> R + 'static`、引数つきを `impl Fn(T) -> R + 'static`（`R: HandlerResult`）に統一し、
`impl Into<Callback<_>>` や `Callback` を引数に取らない（`dev/tests/workspace-policy` の `closure_inputs` が `crates/*/src` を検査する）。
`Box<dyn Fn ..>`・`Rc<dyn Fn ..>`・`Arc<dyn Fn ..>` も引数に取らない（Arvane・engine・reactive の runtime の層どうしの受け渡しは除く）。
closure の引数は `T` から推論され、引数なしの通知は `|| ..` と書く。
`Callback` は closure を Copy で保持する型で、部品作者の保持や、アプリが同じ処理を複数の口へ共有するときに `callback(f)` で作る。`callback` は `Fn(T) -> R` の1つの形だけを受ける（引数の method を呼ぶ closure は `callback(|e: KeyEvent| e.key)` のように型が要る。rustc は `impl Fn(E)` を通して推論しない）。引数なしは `callback(|_| ..)`（`T = ()`、`.call(())`）と書く。
保持した `Callback` を別の口へ渡すときは `move |v| cb.try_call(v)` で包む。
handler の戻り値は `()`・`Result<(), UiError>`（key・wheel・pointer は `Handled` も）に封印する。`?` が注釈なしで `UiError` に推論される。

### setup-once と handle

新しい memo・callback・signal を作るもの（`binding`・`field`・`item`・`resource`・`action` ..）は setup で作る。
setup の外での生成は位置付きの `SetupOnly`（4章）。
handle（`Signal`・`Value`・`Var`・`Callback`・`Binding`・`Selector` ..）は `AppId` と arena の key（slot と generation。公開しない）で、
`Copy + !Send + !Sync`（`UiThreadOnly` marker）。
`AppId` は `kivact-reactive` が定義する不透明な32bit識別子で、整数からの構築・整数への変換はreactive内部に閉じる。
UI thread の外へ持ち出すとコンパイルで止まる。実際に状態へアクセスするときに App・世代・所有者の寿命を検査する（4章「arena と handle」）。
`binding(value, set)` は値を `IntoValue<T>`、書込みを closure で受け、別々の状態を結ぶ。
モデルの一つの field は `Signal::field`/`Binding::field`、keyed な一覧の一行は `Signal<Vec<T>>::item(key, id)` で書ける `Binding` にする。
`Binding<Option<K>>` を受ける入力（`select`・`radio_group` ..）へ、いつも値を持つ `Signal<K>`/`Binding<K>` を渡すときは `.optional()` を付ける（読むと `Some(現在の値)`、`Some(v)` の書込みは書き、`None` の書込みは無視する）。
非同期の入口は `resource`・`action`・`spawn`・`blocking` の 4 つ（4章「非同期と時間」）で、4 つとも prelude と `kivact::task` にある。`blocking` は `action` と同じ封印した形で、`|| ..` と `|cancel| ..` の両方を受ける。各 API の細則は item の rustdoc にある。
`Action::run(input)` は一つの method で、入力のない action の入力は `()`（`run(())`）。入力の型が `run` の呼出しで決まっても曖昧にならない。

### IntoValue と Reactive

`IntoValue<T>` は定数・Signal・Value・Binding・Animated・closure を受け、`into_value()` に統一する。
既存 handle は identity 変換、新規 source は setup を要する。
文字列 prop は setup で owned String に正規化し、動的 closure は static とする。
表示する文字列と accessible な文字列（`text`・`name`・`description`・部品の label・placeholder・hint）は `impl IntoText`（`kivact::view`、封印）で受ける。`&str`・`String`・`Rc<str>`・`Signal<String>`・`Value<String>`・`Binding<String>`・保存済みの `Reactive<String>`、および `String` に `Into` できる値を返す closure（`move || if on.get() { "on" } else { "off" }`）を、毎回 `Into<String>` で変換して受ける。部品は `into_text_value()` で `Value<String>` にする（`IntoValue::into_value` に当たる）。編集して model へ書く値や、絞り込みの検索語（`.filter(..)`）のようにデータとしての文字列は `IntoValue<String>` / `Binding<String>` のままにする。部品の値型は `view::constant_value!(T)` で定数入力にも対応し、登録のない型の定数は `view::constant(v)`（書込みのない `Value<T>`）で渡す。
入力の記述子は `kivact_reactive::Prop<T>` で、`IntoValue::into_prop` が組み立てる。`constant_value!` は通常公開の `Prop` と `IntoValue` を参照し、定数記述子の作成はgraph nodeを割り当てない。
`Reactive<T>` は setup 外のデータにも保存できる入力で、定数・closure・Value/Signal を `Reactive::new(..)` で受け、利用側の setup で owner に属する Value にする。
定数は値、反応値は source の同一性で比較する（`ListItem` と `MenuEntry` の label）。
`ListItem`・`MenuEntry`・`Command`・`Column`・`TreeNode` の field は private で、`label() -> String`・`<名前>_value()`・`is_<名前>()` の accessor で読む。
`IntoReactive<T>: IntoValue<T>` は変換 method のない eligibility marker で、入力の記述は既存の `into_prop` を使う。raw scalar の定数を含める blanket 実装は置かない。
分岐（`show`・`show_some`・`dynamic`）の条件は `IntoReactive<T>`（Signal・Value・Binding・Animated・closure）だけを受け、定数はコンパイルで拒む。
setup の外でも作れるデータ（`ListItem`・`MenuEntry`・`Toast`）は handle を新規に作らない。
`Toast::action` も toast が所有する `Rc` の closure で、setup の外（press handler など）でも作れ、toast と一緒に捨てる。
標準部品と kit の bool の option は `impl IntoValue<bool>` を受け、定数でなければならないもの（`Tabs::orientation` のように keyboard の policy や木の形を決めるもの、
`ListItem::disabled` のような data の値）は rustdoc に理由を書く。`Length` を受ける口は `IntoValue<Length>` で、数値は logical px の定数になる。
向きは `orientation(..)` で指定し（`vertical(bool)` や部品ごとの向きの enum は作らない）、意味は項目または pane が並ぶ方向。scroll も `scroll(child)`（既定は縦）に `.orientation(..)` を付けて向きを変え、縦横の両方は `.both_axes()` と書く。一覧の矢印キーの向きは `ListPolicy::axis: Option<Orientation>`（`None` は 4 方向）。
値を編集する部品の変化を受ける口は binding だけで（`on_change` は持たない）、別の処理は `binding(read, write)` の write に書く。`on_edit` は編集の区切りを知らせる。

### Node と IntoView

要素の型は `Node` 一つで、`node()`・container・`text`・`rich_text`・`image`・`canvas` はすべて `Node` を返す。
何の node か（box・text・image）は作った関数が決める。一つの種類の node だけが意味を持つ設定を他の node に付けると、mount 時に一度 `ErrorKind::Unsupported` を報告して捨てる。
日常の modifier（layout と `style`・`test_id`・`node_ref`・`name`・`description`・`disabled`・`invalid`・`on_*` の event・`draw` など）は `NodeBuilder` の既定 method で、macro なしの普通の Rust で一度だけ書く。
`Node` は `NodeBuilder` を実装するので、prelude の import だけで使える。
`disabled` と `invalid` は、どの入力部品にも付く状態なので `NodeBuilder` に置く（app が import なしで書ける）。それぞれ一つの定義だけで、部品は inherent method で上書きしない。部品は root の `NodeExt::accessible_disabled()`・`accessible_invalid()` を読んで、自分の振る舞いと見た目へ渡す。
部品が何であるかを決める modifier（`role`・`checked`・`expanded`・`selected`・`value_text`）と、part を付ける `part(part)` は `kivact::primitive::NodeExt` の既定 method である。
`Node` はこの 6 つを inherent method にも持つので、`node().role(..)` は import なしで書ける。app が使う builder のうち、`Scroll` は `role`、`Button` は `selected` と `expanded` を inherent の薄い転送として持つ（`NodeExt` と同じ）。prelude の import だけでは部品の builder に出ないので、app は `checkbox(..).checked(false)` のように部品の binding と食い違う状態を書けない。
部品作者向けのほかの modifier（`at`、追加の semantics、`*_default` handler、`on_a11y`・`on_a11y_event`、
`draw_over`）も `NodeExt` の既定 method とし、同じく一度だけ書く。
`NodeExt::base_style(..)` は部品が自分の既定の style を足す口で、呼ぶ順に関係なく利用者の `.style(..)` より下の層に積まれる（base 同士は呼んだ順）。部品は builder の設定が決まる `into_view` で呼ぶ。
部品の builder は root の `Node` を持ち `NodeBuilder` を実装する（`fn node_mut(&mut self) -> &mut Node` の1行）。
`Overlay`・`VirtualList` も modifier 用の `Node` を持って `NodeBuilder` を実装する（内容と子は使わない）。mount では、その `Node` の `Common` だけを取り、
`draw`・`draw_over`・`intrinsic_size` は `Unsupported` として一度報告して捨てる。
`on_press` を持つのに focusable でも role も持たない node（pointer でしか押せない node）は、debug build の mount で node ごとに一度 `ErrorKind::Unsupported` として報告する（node は作る）。意図して pointer だけの転送（`label` の対象への転送など）には `.role(Role::Generic)` を付ける。
`on_press` は後の呼出しが前のものを置き換える。part の既定の activation は caller の後に走る。
`Part` は semantic と input の部分集合を自分の inherent method に持ち、`NodeBuilder` と同じ `Common` の method に転送する。
modifier の中で field の書込み以上の処理を持つものは `view::modifiers` の `Common` の method に一つだけ書く。
構築関数は `#[track_caller]` で既定（role・focusable・既定 style）を先に置き、利用者の modifier が後から勝つ。
文字の性質（font・wrap・行数・揃え・overflow）は style の property で、CSS の継承する property と同じく、property ごとに独立に子孫の text へ継承する。
root は theme の大きさ・太さ・色・family・行の高さを持つ。
細則は `kivact-style` の `text` と engine の `wire::text` の rustdoc にある。
まれな semantics（関係・level・set_size・active descendant など）は一つの共通の box に置き、普通の node の割当てを増やさない。

### 誤用の固定と値の型

誤用のうちコンパイルで止められるもの（handle を別 thread へ、`blocking` が handle を捕捉、`PartialEq` のない型の `set`、
文字列の口に `Signal<i32>`、`f64` を編集する部品への `Signal<i32>`、分岐の定数の条件、入力のない action の `run()`、
factory へ prop を move）は trybuild（`dev/tests/kivact-public-api/tests/ui`）で固定している。
`#[diagnostic::on_unimplemented]` の note は条件で出し分けられない（stable に `on(..)` がない）ため、`IntoValue` の note は全ての `T` に正しい文にする。
公開 authoring 型（theme を含む style・draw builder・handler の region を持つ event）と、解決済み wire 値は責務が異なる。
同じ意味の値は `ui-types` の単一定義を再公開し、同義型の往復変換をしない。公開 path は `kivact::draw`・`view`・`style`・`control`・`platform` に保つ。
Arvane の backend object、Taffy、engine の内部 ID と木の型は `kivact` の公開 API に出さない。
値の検査（`LineHeight` の正で有限な値、色成分の範囲への制限、safe area の負数・非有限値）は host に残す。
core の `FontFeature` は不正なタグを診断する authoring 型として残し、検証後の wire は共有の値型を使う。

### 命名

`kivact::interaction` の構成関数は react-aria と同じく `use_*` と名付け（`use_dialog`・`use_menu`・`use_list` ..）、部品作者が使う。
アプリは `kivact::ui` と kit を使う。
部位の名前は Ark UI・Radix の anatomy に揃える。浮く部分と開閉する中身は `content()`、一覧の項目は `item(key)`、項目の容器は `list()`、
表は `grid()`/`header`/`row`/`cell`、zoom_pan の面は `viewport()`、slider は `thumb()`/`track(axis)`、number field は `input()`、
単一要素の部品は `control()`。
開閉の問合せは `shown()`、命令は `open()`/`close()`/`toggle()`、receiver は Copy の `self`。
caller が持てる状態はその名前の builder に `Binding` を渡し（`sort(binding)`・`widths(binding)`・`anchor(binding)`）、問合せは `<名前>_value()`。
表示は自由関数 `show(handle.shown(), factory)` で組み立て、表示中だけ新しい owner で中身を mount する。
標準部品の overlay は、trigger を取るものは `.attach(trigger)`、中身は `.content(..)`、dialog の footer は `.footer(..)` に揃え、
method 名の `show` は分岐の `show` だけに使う（kit の同じ役割の method も同じ名前にする）。
例外は `MenuApi::show` と `CommandPalette::show` で、menu は origin の種類が変わると中身を作り直し、palette は開閉の状態を dialog に持つため、中身はこの method で mount する。
dialog の popover は `dialog(..).kind(DialogKind::Popover).attach(trigger)` で trigger に付き、`anchor` の口は標準部品に持たない。
popover の開く側と揃えは menu と同じ `placement(Placement)`・`align(OverlayAlign)` で、title を見せない panel は `hide_title(..)` で名前だけを残す。
dialog の header の先頭の view（alert の icon など）は `media(..)` で、header の recipe は `DialogState::media` で並べ方を選ぶ。
`attach_context` の menu は pointer か target にそのまま開き（既定の offset は 0）、最上位の面を `context_surface` の recipe で描く。
`field(label, control)` は、見える label・control・`hint`・`error` を縦に並べる。label は control の accessible name（`labelled_by` で結び、control が持つ名前より優先する）で、`hint` と `error` の message は `described_by` で control に結ばれる。`error(impl IntoValue<Option<String>>)` は `Some` の間だけ message を出し、control を `invalid` にする（control 自身の `invalid` とは or を取る）。control は `NodeBuilder + IntoView` で、`text_input`・`checkbox`・`switch` の recipe は `NodeExt::accessible_invalid()` で読んだ値を state の `invalid` に渡す。
`tabs` は常に選択を持つので `Binding<K>`、select と radio_group は `Option<K>` を受ける。radio_group は名前（`radio_group(label, items, value)`）を必須にする。
表は標準部品の `data_table(rows, key, columns)` で、列は `kivact::ui::DataColumn<R>`（interaction の `ColumnDef<R, C>` の builder）一つ、interaction の列の型も `ColumnDef` 一つで、並べ替えられるかは `compare` を持つかどうかで決まる。`compare`・`filter` を渡すまで並べ替えも照合もしない（text で並べる近道は `sortable()`）。
選択・sort・page・filter・列幅は任意の builder で Binding を受け、渡した機能だけが付く。表の上と下の領域は `toolbar`・`footer`、行の下と cell の中身は `row_details`・`cell_controls` の slot で、kit は表を作らず `DataTableRecipes`（枠・上・下・header・行・cell・check の箱・並べ替えの印・頁送り）だけを持つ。
`list(label, items, selected)` も同じで、kit は `ListRecipes` を持つ。

### 標準部品と kit の外観：slot・recipe・variant

`kivact::ui` は button・toggle・checkbox・switch・text_input・text_area・field・select・dialog・menu・tooltip・tabs・radio_group・slider・range_slider・splitter・split・progress・disclosure・toast_region・list・data_table・command_palette・tag_input・number_input を
一つずつ実装し、振る舞いは interaction の部位に任せる。kit は標準部品を再実装・再公開せず、theme と domain の部品を持つ。
外観は三段に分ける。

- **token**：Theme の palette・寸法・`focus_ring`・`disabled_opacity`・`icons` と、kit の `ThemeExtension`（`Theme::with`、
  読むのは `theme.get::<T>()` だけ）。
- **recipe**：部位ごとの style。`pub type Recipe<S> = Arc<dyn Fn(&Theme, &Look<S>) -> Style + Send + Sync>` が唯一の形で、
  `UiRecipes`（これも `ThemeExtension`）を `Theme::with` で載せる。
  recipe は style だけを返し、hover・押下・focus・disabled の見た目は返す style の状態 variant で書く（`state.input` を読んで分岐しない）。
  focus ring と disabled の不透明度は、focusable な node と interaction の部位を持つすべての node（tab stop でない roving の項目を含む）に theme が既定で付ける。kit は `disabled_opacity` を手で付けない。
- **slot**：項目に何を表示するか（`Tabs::item`・`RadioGroup::item`・`Menu::item`・`Select::option`）。項目と状態の `Value` を受けて View を返す。
  recipe が任意の View を返す外観 callback は持たない。
- 部品の slot の method（`content`・`footer`・`hint`・`heading`・`actions` など）は view を作る closure を受ける。開くたびに作り直すものは `impl Fn() -> V`、mount のときに一度だけ作るものは `impl FnOnce() -> V` で、容器の子（`column((..))`・`panel(content)`）は view を直接受ける。

同じテーマの中の見た目の変種（segmented の radio_group、boxed の tabs）は、別部品や部分木の theme ではなく、
全標準部品の builder が持つ `.variant(v: impl Variant)` で `Variant` を実装した型の値（button の `ButtonVariant` も同じ）を渡し、recipe が `Look::variant::<T>()` で読む（知らないテーマは無視する）。
`Look<S>` は部位の実際の状態 `S` と、その per-instance の variant を持つ。
kit のテーマは `theme(mode: ThemeMode)` の一つの形で core の `Theme` を返し、`App::theme`・`provide_theme` にそのまま渡せる。
kit の選択肢は kit が `Theme` に足す trait（`ThemeOptions`）の method で受ける（`theme(mode).density(..)`）。method は `self` を受けて `Theme` を返し、`#[must_use]` を付ける。選択肢のない kit は持たない。
token の型と recipe を組み立てる theme constructor を分け、循環を作らない。
icon を描く経路は `control::icon(..)` だけで、kit の画像は kit の theme の `IconSet` に名前で入れる。
通知は core の `Toast::new(title).tone(Tone::Success).body(..)` で作り、kit は作る関数を持たない。意味は `kivact::ui::Tone`（Neutral・Info・Success・Warning・Danger・Accent）の一つで、toast の icon は `ToastRecipes` を持つ theme の `IconSet` から `toast-info`・`toast-success`・`toast-warning`・`toast-danger`・`toast-accent` の名前で引く。色相の名前（pop の `Hue`）は意味ではなく kit の外観の variant である。
各 State 型・icon の名前・表の読み書きの method は `kivact::ui` の rustdoc にある。

### 共通 interaction と公式 UI の境界

basic・pop・game・creative の公式 kit は描画・入力・キー操作・支援技術と view の状態を受け持ち、アプリのドメインの判断は持たない。境界を次の規則で守る。

1. アプリの項目列は `impl IntoValue<Vec<T>>` で読取専用に受ける。項目列を `Signal<Vec<T>>` や `Binding<Vec<T>>` として受け取る公開 API は作らず、kit は列へ書かない。
2. `Binding` で書けるのは、選択・zoom・pan・`ViewTransform`・tool・展開・開閉・page・sort の列と向き・改名中の行・列幅・dock/panel/split の layout と、
   playhead・loop range・cycling・数値・色など単一の編集値である。gradient の stop の追加・削除も一つの gradient 値の編集に含む。
3. 項目への変更は callback で要求する。creative editor は `on_edit(impl Fn(EditorEvent<R>))`、
   basic・pop・game の小さな部品は `on_add`・`on_remove` などの個別の closure を受ける。
   アプリが要求を反映した列がそのまま描画され、拒否した変更を kit 内部で代行しない。
4. 長さ・最短の長さ・接続の可否・ID・名前・再生クロック・ファイルの種類と大きさ・数値と日付の書式・sort の比較規則・色の意味は、アプリが値や関数で渡すか、要求を反映するときに決める。
   kit の定数は描画の寸法と view の既定値に限る。
5. 色や強調の意味は文字列から推測せず、`Tone` などで明示する。模擬データとアプリ固有の文言は catalog の側に置き、渡されない内容は描かない。
6. mount だけでアプリ全体の状態を変更しない。`toast_region` の利用に必要な `Toaster` の表示上限などはアプリが設定する。

値を書き換える部品は `on_edit` に `EditPhase` を出し、取消し（Escape・engine の Cancel・破棄）で編集前の値を書き戻す（`EditSession`）。
履歴はアプリが持つ。drag の取消しは interaction の `Drag::edit`・`Drag::edit_on_change`・`Scalar::scrub` が担い、kit は Escape を手で扱わない。
Creative の domain editor の transaction と lane 拡張は [Creative editor の編集と lane 拡張](architecture/creative-editors.md) にある。
Creative の実装は private な `editors/{time,graph,node,arrangement}` に分け、model・view_state・gesture・input・edit・layout・paint・a11y を責任ごとに置く。下位の `editors/shared` は domain の builder や view state を参照しない。`CanvasGesture` が押下の payload・保留した選択・移動の記録と終了順を共有し、pointer の捕捉と編集の確定・取消しは公開の `Drag`・`EditSession` が持つ。crate root の公開口は保ち、数値の面をドラッグする部品は `value_slider`、標準の track と thumb は `kivact::ui::slider` で区別する。

アプリの通常の button・checkbox・文字入力は `kivact::ui` の標準部品で作る。表示名・role・test id・参照・focus 復帰・disabled 条件はアプリが保つ。
主操作以外の button は Secondary/Outline/Ghost/Link を選ぶ。必要な寸法と伸縮以外の見た目は theme の recipe に任せる。
catalog の選択付き navigation・複合部品、workbench の外部 icon button の実証、Calibration・waveform など固有の描画操作は、標準部品とは契約が異なるため残す。
選択してコピーできる読取専用の文字（basic の `code_block`）は、`TextEditing` の read-only・multiline・`highlight`（行ごとの色と、行をまたぐ状態）と標準の key policy で作る。中核に code 用の口は足さず、色分けの規則（basic の `rust_lines`）と色の token（`Tokens::code`）は kit が持つ。

### 実験的な API と `unsafe fn`

`kivact::embed` と `App::embed` は feature `embed` の実験 API で、既定では存在しない。

実験的な口は、既定で off の feature の後ろに置く（`gpu-interop` の `kivact::gpu_interop`）。feature の無い build では存在しない。
外部の render engine の texture を取り込む `gpu_interop::import` と、その高水準の口 `ShareLink::provide` は公開 API で二つだけの `unsafe fn` で、`# Safety` は engine の約束（`SharedTextures` の rustdoc）を守ることを求める。
約束違反は graphics API（Vulkan と Direct3D 12）の未定義動作で、壊れた画、device の喪失、process の異常終了、resource の範囲外の GPU memory への access がありうる。Kivact と Arvane はその大部分を確かめられない。だから `unsafe fn` である。
Kivact の crate は `unsafe` のブロックを持たず、`kivact-core` はこの二つの宣言だけを `allow(unsafe_code)` にする。
wgpu・ash・windows の型は公開 API に出さず、語彙は ui-types の値（`GpuAdapter`・`SharedTextures`・OS の handle は std の `OwnedHandle`・`OwnedFd`）を明示的に再公開する。
取り込んだ資源の寿命は `Pixels` と違い値ではなく owner に従い、`ExternalImage` の clone が残っていても owner の破棄で外す。

API は二段にする。低水準の `import` は一つの pool を今取り込み、adapter と状態を追うのはアプリ。`ExternalImage::publisher()` が返す `FramePublisher`（`Send + Sync`）は engine の thread から frame を出し、`ExternalImage::latest_frame()`（追跡される）に届ける。
高水準の `shared_image(w, h)`（setup だけ）は、その上で取り込みを肩代わりする。UI の handle `SharedImage`（`!Send`）と、engine の thread に渡す `ShareLink`（`Send + Sync + Clone`）に分かれる。
- **要求。** 作った時点と、adapter の変化（`generation`、`Some` と `None` の切替え）・`set_size`・取り込みの `Lost` のたびに `ShareRequest` を出す。adapter があれば `ShareMode::Gpu(adapter)`、なければ `ShareMode::Pixels`。要求ごとに非公開の token が一つ増え、古い token の呼出しは `ShareError::Stale`。新しい要求を出した時点で UI は古い取り込みを外し、engine は古い pool を捨てる。
- **取り込み。** `provide` は最新の `Gpu` の要求だけを一度受け、UI thread が `import` する。`wait_ready` は `Ready` で `Ok`、失敗で `Failed(msg)`、置き換わりで `Stale`、時間切れで `Timeout`。engine は `Ok` の後に texture へ書く。
- **frame。** `publish` は最新・`Ready` の要求の frame だけを受け、最新のものを残す。読み飛ばした frame は、Arvane が release を「描いた値 − 1」に進めるので一緒に返る。`publish_pixels` はいつでも受け、最新を残す。`SharedImage::frame`・`pixels`・`status` は追跡される。
- **閉じる。** owner の破棄で link を閉じ（`is_closed`）、待ちはすぐ返り、`publish` は何もしない。取り込みは外れる。
- **起床。** engine の thread から UI への起床は、UI thread の executor の waker（host の `HostWaker`）だけを使う。worker が channel で UI の task を起こすのと同じ形で（canvas の how-to の worker の節）、Kivact は engine の thread を持たず、新しい起床の経路も作らない。engine の thread の待ちは、条件変数の時間付きの待ちで UI の変化を受ける。

### 中核への追加の判断

kit が中核の不足を自分で埋めるのではなく、汎用の口を中核に足す（[戦略](strategy/20261009_strategy.md)）。承認済みの例:
`TextInput`・`TextEdit`・`TextEditing` の `forward_keys`（指定した制御キーだけを native 編集より先に key handler へ渡す）、
`Part::region_events()`（部位の既定処理を同じ canvas の未処理 region 入力へ届ける）、`Handled::PreventDefault`（6章）、basic の shadcn の再現のための `Dialog::media`・`hide_title`・`placement`・`align` と menu の `context_surface`、accordion の高さの閉じる動きの間 content を見せたままにする `Disclosure::leaving`、button group が種類の違う子を `Node` にそろえるための `TextInput::into_node`（`Button::into_node` と同じ形）。

## 4. 反応と runtime

### arena と handle

App ごとに arena（graph node・owner・owner に属する object・定数）を持つ。arena の `Key` は `kivact-reactive` の外へ出さず、公開する識別子は不透明な `AppId`・`OwnerId`・`NodeId`（`Copy + Eq + Hash`、field は private、利用者が作る口はない）だけにする。
解放で generation を上げ、飽和した slot は退役するので、同じ `(slot, generation)` が二度有効になることはない。
墓標が `Disposed` の報告に作成位置と破棄位置を載せる。
handle の値の読出し・書込みと Callback の実行など、実際に状態へアクセスする操作で App と世代・所有者の寿命を検査する。
既存の handle の `IntoValue` identity 変換は node を作らず、検査もしない（実際の読み書きのときに検査する）。
App が drop 済みか別の App が実行中なら `WrongApp`、破棄済みなら `Disposed`。
「実行中の App」は frame を push するたびに設定する thread-local で、App を識別する可変 global 状態は `REGISTRY` とこの2つ。これに加えて、reactive が最初の誤用の報告のときに（`pub(crate)` の `install_quiet_hook` が）process 全体の panic hook を一度だけ置き換え、`MisusePanic` の出力を抑え、それ以外は元の hook へ渡す。
`kivact-reactive` の公開面は3つの読み手に分かれる。利用者と部品の作者は facade 経由の handle・setup 関数・`IntoValue`・`UiError`、
上の層（view・task・motion・style）は `Owner`・`Owned`・`Trigger`・`Prop`・`keyed`・`pure`・`require_setup`・`require_effectful`、
App の turn を動かす host は `Runtime`（`create_root`・`pending`・`mounts`・`turn`・`report`・`report_later` ..）と `Driver`（`now_ns`・`request_turn`）を使う。`raw` module と `#[doc(hidden)]` の公開はない。
`Trigger` は値を持たない通知の source で、`track` で購読し `notify` で購読者を無効にする（木のメタデータ・NodeRef・focus・virtual list・Action/Resource の revision など、旧 `Signal<()>` の revision）。
frame の外では登録済みの App を推測しない。現在の runtime は `None` になり、owner や時計を必要とする API は呼出し位置付きの `SetupOnly` / `WrongApp` を報告する。任意の focus 問合せは `None`、`clock_now` は 0 を返す。明示の handle は自身の App を選べる。
利用者のコードを実行している間は arena の表を借用しない（値は `Rc` を clone してから closure を呼び、破棄した値は借用が外れてから drop する）。

`Value<T>` は定数 arena の key または graph node の key を持ち、`T: Copy` / `Clone` を要求せず Copy にする。定数は owner とともに破棄され、墓標の作成位置は定数自身の作成地点、破棄位置は owner 側の地点を使う。定数を読んでも依存を集めず、graph node や effect を作らない。`Stats::live_reactive` は graph node、`live_constants` は定数を別々に数え、`arena_slots` は4つの arena の割当 slot をすべて数える。

実行境界では Copy の `Ctx`（effective owner・setup owner・pure・running effect・tracking slot）を1つ push し、guard が unwind と cross-App 呼出しを含めて元へ戻す。依存の Vec / HashSet は runtime の別の collector stack が所有する。新しい computation の guard だけが collector を回収し、pure の copy は同じ slot を使い、untrack / callback は追跡を止めても外側の collector を保持する。selector の prune は全 active collector の依存を保護し、cycle path は computation の collector だけから作る。

Setup は setup/effective owner を設定する。callback は effective owner だけを自身へ変え、呼出元の setup owner・pure・running effect を継承して untracked にするので、setup 中の割当は呼出元に、spawn/context は callback owner に属する。handler/task/on_mount は自身の owner と nonpure/untracked/nonsetup の文脈で走る。pure は Setup 中も必ず pure にし、書込みを拒む。

### graph（push-pull）

書込みは `Runtime::write_node` だけを通り、直下の購読者を Dirty、推移的な memo を Check にし、effect を queue に積む。
memo は lazy で、読むときに依存を確かめ、`PartialEq` で等しければ revision を上げない（cutoff）。
effect は生成順に走る。親の flow の effect が子の effect より先に走るので、破棄された子は走らない。memo を読めば必ず最新になるので glitch は見えない。
同じ effect が1回の安定化で 100 回を超えたら `Cycle` で打ち切る。
値だけを計算する closure（flow の条件、一覧の items と key、`dynamic` の source、resource の source、animated の target ..）は pure で走らせ、
その中の書込みと副作用（cancel・focus・platform 要求 ..）は状態を変える前に拒む。
一方、利用者の関数が走る次の文脈は常に untracked で、中の `get`・`with` は何も購読しない: event の handler（`on_*`）、`Callback::call`（呼出元が effect でも購読しない）、`on_mount`、task・action・resource の本体（`effect_on` の本体も同じ）。守りの `get_untracked` は要らない。

### owner と境界

owner は構造の境界ごとに作る: root・flow の分岐・一覧の行・`provide`/`provide_theme`・`error_boundary`。
reactive の `Owner`（`Copy`、App を保持しない）は子 owner・signal・trigger・effect・object・context・cleanup・error 受け口を作り、`Owned<T>` は owner の下に置いた `Rc<T>` を指す Copy handle で、`try_get` は破棄が始まった owner の下では `None` になる。
部品の作者に見せる `kivact::primitive::Owner` は core の newtype で、`current`・`store`・`store_with`・`on_cleanup`・`provide`・`context`・`report`・`root`・`focused_node`・`node_ref` だけを持つ（signal の作成と setup の実行はない）。
owner が生きているかは、setup で `store` した `Owned<()>` の `try_get` で調べる。同じ App かどうかは `Owner::root` の同値で調べる。
App 全体の状態（toaster・tooltip の hub）は `primitive::app_state` が root owner の setup として一度だけ作り、root の context に置く。
境界の子は子 owner の `Setup` frame と `catch_unwind` の中で setup し、setup の panic はその境界を空にする。
setup 中に作ったもの（signal・effect・node・task・action・context ..）は現在の setup owner に属する。setup の外での生成は位置付きの `SetupOnly`。
`spawn`・`context`・`kivact::platform` の関数・`style::current_theme` は frame stack で最も近い owner を使う
（handler は宛先 node の owner、task は task の owner、`Callback::call` は callback の owner）。
`context::<T>()` は未提供なら呼出し位置と型名を含む panic を報告し、任意の context は `try_context::<T>()` で読む。
破棄は owner ごとに 子 owner → cleanup（LIFO）→ user effect → render/internal effect → memo → source → object の順。
破棄中・破棄後の owner への登録は runtime が扱い、呼出し側は確かめない（G-01: 破棄した部品のものを残さない）。登録には必ず解除が対になる。
誤用は `MisusePanic(UiError)` として投げ、境界で捕捉して owner 木を上へたどり、最寄りの `error_boundary`（`on_error` → `fallback`）へ送る。
なければ既定の受け口（`tracing::error!` と ring buffer）。
handler を走らせてはいけない所（engine が自分の木を借用したまま呼ぶ primitive の measure 関数）で起きた error は `Runtime::report_later` が預かり、次の turn の始めに送る。
handler は常に engine の木を読み書きできる所で走る。

### turn

`AppClient::turn(events, boundaries, now)` が `start` の後の唯一の入口で、すべての turn が同じ規則で確定まで進みます。
`bridge::Client` の callback は `start` と `turn` の2つで、effect は起動時に受け取った所有型 Writer へ直接書きます。

```text
0. AfterCommit（前の turn までに境界を配った revision の on_mount）→ NextTurn（前の layout が預けた error〔report_later〕を handler へ送る）→ 安定化
1. event を順に配送（古い node・世代違いは捨てて trace に記録）。handler ごとに AfterHandler（要求された activation）→ 安定化。commit の境界に来るたびに、AfterCommit（その revision の on_mount）→ 安定化
2. 期限の来た timer と準備のできた task を poll → 安定化（1000 周で残れば Cycle を報告し、残りは次の turn）
3. AfterHandler（要求された activation があれば、安定化と 2 の timer・task の確定）→ BeforeCommit（frame の要求・focus の要求・消えた node の後始末・mount の token）→ engine に未 commit の書込みがあれば commit（revision + 1。なければ revision を上げない）
4. AfterCommit（layout 済みの revision を待つ on_mount）と User effect。その書込みは次の revision になり、3 へ戻る（最大100周 → Cycle）
5. 埋め込みでは default sink の最初の未処理 Panic を一度だけ確認し、元の UiError を保存して TurnOutput::stop に診断を入れる。次の wake 時刻を返す
```

保留の仕事（activation の要求・focus の要求・消えた node の後始末・on_mount・預けた error）は、置き場と規則が別々の列（core の `Requests`、runtime の mount と預けた error、`Tree` の remove など）に残し、`client::phases` の `drain(rt, Phase)` だけが、phase（AfterHandler・BeforeCommit・AfterCommit・NextTurn）ごとに決めた順で各列の drain を呼びます。
どの phase が何を、どの順で、なぜその時点で流すかは `client::phases` の module doc の表に書きます。

host は turn の前に必ず layout し、待っている event をすべて、その間の commit の境界（`Boundary`：何番目の event までがどの revision の書込みから生じたか）と一緒に1つの turn に渡す。
境界は on_mount を走らせてよい時点だけを決め、turn の規則は変えない。
client の書込みが engine の中で生んだ event（focus 要求の結果の `FocusChanged`・`FocusResult` など）は、
その revision の layout の後の turn に境界の前の event として届き、
その revision の on_mount はそれらの後に走る。layout が生む event（`LayoutChanged`・`ScrollChanged`・`LayoutWarning`）は境界の後の event として届く。
どの turn でも、handler の書込み、それが起こす effect と task、その commit は同じ turn の中で確定する（次の turn や wake を待たない）。
同じ layout の後に配る境界はまとめて1つの turn にするので、続いて起きた focus の移動は effect には最後の状態として見える。
event の配送は、次の event を先読みして決めることをしない。canvas の region の gesture が press を採用した事実（`Writer::adopt_pointer`）は engine が pointer ごとに覚え、click が生んだ `Activate { pointer: Some(id) }` は、core が配る直前に `Reader::take_adopted_activation` で尋ね、採用済みなら黙って捨てる。採用は、その pointer の次の primary `Down`・`Cancel`・`Activate` への答えで終わる（event を配る順に従うので、1つの batch の中の Down と Up と Activate でも決まる）。
1回の pump で 64 turn 続けて event が残れば、host は残りを次の wake に回し、`HostEvent::TurnsCut` で知らせ、core が `Cycle` として報告する。

木は1つ: node の prop と scene の置き場は engine の木だけ。
core の view の木は engine が持てないものだけを持つ: fragment を含む構造、owner・handler・作成位置・prop ごとの書込み元 effect。
rect・test id・layout の observe は engine の木にだけあり、core は Reader で読む。prop 値や書込み列の複製は持たない。
effect は比較つき setter でその場で書き、同値の書込みは記録しない。通常の callback は engine の借用を解放してから呼ぶ。

focus の正本は `view` の反応する source で、host の `FocusChanged` が turn の中で書く。
`NodeRef::{focused, focus_within}`・key の配送先・modal を閉じた後の復帰は、すべてこれを読む。
hover は host の `HoverChanged`（hover の chain の先頭）を core が覚え、`NodeRef::hovered` を最初に呼んだ App だけが graph の source を持つ。
復帰の要求（modal の close・Popup の close・`NodeRef::restore_focus`）は一つの経路に積み、送る時点で対象が生きて見え focusable で残る modal の中にあるかを、engine の `Reader::can_take_focus` で確かめる。
利用者の明示の focus 要求が勝ち、部品は anchor の生存・非表示・disabled を自分で検査しない。
nonmodal の popup が戻す先と理由は interaction の `FocusReturn`（`focus_return`）を popup が一つ持ち、menu・dialog の popover・scalar の thumb は target と理由を渡すだけにする。
modal の復帰だけは core の overlay の cleanup が積み、popup は重ねて戻さない。
list・table・menu の level は、mount 中の項目・最後に要求した項目・観測した focus・消えた項目の focus を interaction の `Roving<Id>`（`roving`）に一つ持つ。
他の手段で来た focus は tab stop を移し、消えた項目の focus は tab stop へ渡す。順序・有効判定・選択・typeahead・pending の要求・scroll は各部品が持つ。
modal の受理は engine だけが判断する（`kivact-engine` の `model::modal`）。core は modal の一覧も drag の記録も持たない。
core は event を配る直前に、event 1件ごとに1回だけ `Reader::admit_input` で尋ねる。同じ turn で先の event が開いた modal が、後の event にすぐ効く。
表示中の modal のうち木の順で最後のものの外を狙う入力（`Activate`・`A11y`・`Wheel`・`File`・`TextChanged`・`SelectionChanged`・`OverlayDismiss` と、pointer の `Down`・`Enter`・button を押さない `Move`）は受理されず、core は捨てて診断に数える。
modal を開く前に `Down` を受理した drag の `Move`・`Up`・`Cancel` は同じ node への限りその後も受理し、最後の離し、または `Cancel` がその受理を終える。別の node への却下はその drag を終えず、`Leave` は常に受理する。
key は modal で止める。その境界は `Reader::active_modal`、誰も処理しなかった Escape の宛先は `Reader::dismiss_target`（modal があればその中の、Dismiss を持つ一番上の overlay）で決める。
core に残る modal の仕事は、overlay の unmount で復帰の要求を積むことだけである。
key の開始先は host が名指す node、focus、一番上の modal、App の view、root の優先順とし、祖先へ bubble する。
focus も modal もない場合の App view 開始は、top-level の `on_key` を window 全体の shortcut に使うための契約とする（この fallback で届いた Enter・Space は activate しない）。
誰も処理しなかった Escape は、一番上の Dismiss を持つ overlay へ送る。

### engine の借用中に走る利用者の関数

engine の木や model を借用したまま同期で呼ばれる利用者の関数は `SyncHook`（`view::sync_hook`）だけである。今は leaf の measure、`TextEditing::behavior`、`TextEditing::highlight` の行の色の3つで、これから足す同種の関数も同じ型で包む。
provider（mount の owner の cleanup と、新しい content を書く束縛）が `Rc<Hooked { content, f, reported }>` を持つ。engine の callback は `Weak<Hooked>`・`Weak<Runtime>`・owner・呼出し位置を持つ `SyncHook` だけを持つ。engine が借用中に callback を捨てても利用者の値は drop されず、古い `Hooked` は view の木を借りる前に provider が捨てる。
`SyncHook::call` は `Weak` を upgrade し、owner の handler の frame に入ってから pure かつ untracked の frame で関数を走らせる。signal の書込みと副作用は `WriteInPureContext` として拒否され、読出しは何も購読しない。
panic（拒否された書込みを含む）は `catch_unwind` で捕まえて engine へは出さず、`Hooked` ごとに一度だけ `Runtime::report_later(Some(owner), ..)` で報告し、`call` は `None` を返す。呼出し側が自分の fallback（size 0×0、色なしの行、behavior の「処理しない」）を選ぶ。報告は engine が model を手放した後の次の turn の頭に届く。

### 非同期と時間

executor は UI thread 局所で `!Send` の future を走らせる。waker は host の `HostWaker` から作るので worker thread からも起こせる。
future は `CatchUnwind` で包み、`turn` から panic は出ない。
task は owner の object で、破棄で future を取り消す。blocking は App 所有の固定 pool。
`Resource` と `Action` は状態を1つの signal の `AsyncState<T> { Idle, Pending, Ready(T), Failed(UiError) }` に持ち、
読出しは `state()`・`latest()`・`error()` だけ（`AsyncState` も `is_ready`・`is_failed`・`value`・`error` を持つ）。
取消しは `ErrorKind::Cancelled` の `Failed`。`Action` の run の扱いは `ActionPolicy`（`Latest`・`Queue`・`QueueLatest`）が決め、
queue の policy の action だけが owner の破棄の後も detach して期限付きで処理を続ける。
時計は host の時刻で、client が driver として runtime に渡す。timer・frame loop・animation はすべて `Runtime::now_ns()` を読み、試験の host は仮想時計。
`kivact::platform` の service の要求は `Writer::effect` と `EffectResult` の `RequestId` で照合する。

### OS thread の起動と終了

非 test の framework source の thread 起動は次の9箇所。`async_task::Builder` は UI thread 上の future で、OS thread を作らない。`test_display`（feature testing）と rustdoc のアプリ例はこの数に含めない。

| 起動箇所 | 所有と終了 |
|---|---|
| `arvane/src/presenter/gpu.rs` の init と configure（2箇所） | Presenter が JoinHandle を保持。結果を受けると join。shutdown は window.wait_until で各最大2秒待ち、未完了なら警告して detach を記録し、埋め込みの破棄後に module を保持。遅れて届く surface/device の結果は receiver ごと保持して漏らし、worker 上の native surface 破棄を避ける |
| `arvane-asset/src/store/worker.rs` の Worker | store の Drop が停止を合図して最大2秒待ち、完了なら join、未完了なら警告して detach を記録し、埋め込みの破棄後に module を保持 |
| 同 file の inline helper | `spawn_scoped` の join と scope 内で同期終了。2秒上限の background worker ではない |
| `arvane-native/src/embed/x11/thread.rs` | 外部 Drop は Close を送り最大2秒待って join。上限超過時と自身の callback 内の Drop は未 join を記録し、その場で module を保持。callback が戻った後に外側 loop が終了と後始末 |
| `kivact-task/src/state/pool.rs` の pool と送信失敗時 fallback（2箇所） | pool が全 handle を保持し shutdown で各最大2秒待ち、完了なら join、残るものを警告して detach を記録し、埋め込みの破棄後に module を保持 |
| `kivact-task/src/blocking.rs` の pool がない経路 | runtime があれば TaskState が handle を保持して同じ終了待機。App の外では所有する TaskState がなく即 detach を記録。後の Embedded の Drop が module を保持 |
| `arvane-render/src/gpu/encode.rs` | scope.spawn の結果を同じ encode 呼出しで join。scope も終了を待つ |

detach は各 crate の static AtomicBool に保持し、kivact_task::detached_threads と arvane::detached_threads（presenter・asset・native の論理和）から確認する。Embedded の Drop は host の破棄後に確認し、残る thread があれば arvane-native::retain_module を呼ぶ。X11 の UI thread は上限超過と自己 callback からの Drop の時点で直接保持する。起動失敗で Embedded が返らない場合と callback 内の Drop による破棄延期も対象とするため、埋め込みの AppClient の Drop は task shutdown の後にも確認する。Arvane Driver の最後の field に置く埋め込み専用の completion guard は、handler（Host.model の asset store を含む）・presenter の field の破棄後に下位の detach の論理和を確認する。この guard の定義は handler/presenter を知らない shutdown module に置き、依存は下向きのまま保つ。

2秒は worker ごとの待機上限であり、App 全体の終了時間の保証ではない。複数の worker/fallback の待機は累積する。scoped な同期処理の実行時間にもこの上限は掛からない。
アプリが直接作った thread は framework の終了待機に含めない。mini-postman の pipe reader は通常終了・取消し時に join するが、POST 書込みや child.try_wait 等の早期 error 経路では detach し得る。creative catalog の gradient worker は bounded channel に1回送信して終わる短命 thread を detach する。これらを DLL の editor に移す場合は、アプリが停止と join を所有する必要がある。

### 診断

構造化 trace は turn ごとに原因・値を変えた書込み・再実行・engine の木への書込みの数・note を記録し（値は記録せず型名と位置だけ）、`tracing`（target `kivact`）にも流す。常に記録する。
default sink に届いた最初の `ErrorKind::Panic` は ring と独立に保持し、`Runtime::first_unhandled_panic` が O(1) で返す。owner の error boundary が回復した panic は含まない。埋め込みだけが start と turn の終わりにこれを確認し、start の戻り値 `Option<String>` または `TurnOutput::stop` で host に停止を伝える。host は `ShellError` を返し、後続の turn を配送しない。同じ turn の残りの effect は最後まで走る。埋め込みの handle を platform の callback（サイズ交渉など）の中で drop した場合は別で、EmbeddedHost と Host の内部で共有する closing atomic を Drop 冒頭で立て、platform effect の前後と新しい client turn・pump の入口で確認する。close 後の EffectResult で新しい turn や待機 task を再開しない。下位の native handle がまだ close を設定する前に UI thread が観測する場合もあるので、ShellControl::exit は返さず空の control で native の close に任せ、正常な close を App 自身の exit の失敗にしない。この確認は既に走っている client turn の途中には割り込まず、R5 の未処理 panic の turn を最後まで確定する規則は変えない。desktop の flag は None のまま。desktop は stop を常に None に保ち、従来の診断と回復を変えない。
`why(&NodeRef)` は動的な prop ごとに、effect の作成位置と読んだ memo・signal を再帰して返す。
host node は作った builder の呼出し位置を持ち、snapshot と診断はそれで node を示す。

### 多言語対応（`kivact::i18n`）

中核は App の locale と、部品の既定の文言を引く口を持つ。翻訳の資源と言語の切替はアプリが持つ（[戦略](strategy/20261009_strategy.md) の4章）。
`Locale` は BCP 47 の文字列で `App::locale` で与え、与えない App は host の locale に追従する（読めなければ `en-US`）。
部品は `message(key, default)` で文言を引く。`key` は `<部品>.<用途>`（`dialog.close`）、`default` は英語、戻りは `Value<String>` で、
部品は自分で effect を作らない。
文言を持つ部品は rustdoc に key と既定を載せ、`docs/for-kivact-ui-user` に kit ごとの key の表を置く。
`provide_locale`・`provide_messages` は `provide_theme` と同じ形の境界で、切替では何も作り直さず、読んでいる文言と `locale()` だけが更新される。
中核は翻訳の資源の管理も複数形の規則も持たない（複数形を要する文言は部品が key を分ける）。日付と数値の書式は後の単位。

## 5. 描画と表現の契約

語彙と解決の規則は `kivact-style`・`kivact-motion` の各 module と、core の `draw`・`flow`・`view::mount` と、engine の `model`・`paint` の rustdoc にある。
ここには層をまたぐ規則だけを書く。

- **style の唯一の経路。** `Style` は `Option` の field を平坦に並べた値で、node の style は `.style(..)` の層の列。
  node ごとに effect は1つで、theme と disabled・selected を読み、層を順に merge して `Style::compile` で engine の型付き props にする。
  theme の切替は paint を解決し直すだけで remount しない。
- **状態 style は engine が評価する。** hover・pressed・focus・focus_visible・group_hover・group_focus・enter・exit は `StateStyles` の差分として engine へ渡し、
  engine が pointer と focus の状態から重ねる（focus < focus_visible < group_focus < group_hover < hover < pressed < enter < exit）。
  core は pointer の Enter/Leave や signal を読まない。hover は CSS の `:hover` と同じく子孫の上も含むので、Style に `hovered` は作らない
  （`NodeRef::hovered` は同じ規則を反応的に読む口）。
- **既定の focus 表示。** focus stop は style がなくても style の effect を持ち、focus も focus_visible もなければ theme の `focus_ring`、
  disabled の間は `disabled_opacity` を足す。既定は semantics の disabled と focus の対象と同じ node に付くので、描画と意味がずれない（G-07）。
- **描画・hit・意味の一致（G-07）。** node の形・変形・sticky と fixed の位置・clip は、
  描画・hit 判定・`NodeRef::rect`・`LayoutChanged`・accessibility の bounds・
  overlay の anchor・scroll into view・試験の actionability で同じ値を使う。hit の形は node の基本の paint のもの（状態 style で変えても変わらない）。
- **局所描画。** 描く順は caller の `.draw`、`Style::paint`、子、`.draw_over`。描画用の proxy 子は作らない。
  `Style::paint` は box 専用で、text・image・overlay・native 編集を付けた葉と、状態 style の中の paint は `Unsupported` として報告する。
- **登場と退場。** `Style::enter`・`Style::exit` は paint の transition に乗せ、新しい animation の型は作らない。
  kivact が node を取り除く write は一つのままで、exit を見せる部分木は engine が leaving として残す。
  leaving の部分木は描くが、hit・focus・accessibility・modal の範囲・layout の観測からは即座に外れる。
  その上の scrollbar の drag と file の drag も、退場の始まりで終わる。
  leaving の部分木は消えるまで layout の流れに残り、後ろの兄弟は消えたときに詰める（CSS の exit transition と Framer Motion の `AnimatePresence` と同じ）。
  場所を持たずに退場させたい node は、流れの外に置く（`Position::Absolute`）。
- **時計は1つ。** frame で動くものの最も早い期限を `Writer::request_frame(at_ns)` で求め、固定の timer は使わない。
  描けない間（`HostEnv::surface_active` が false の間）は要求せず frame の時計も止める。
  時計の止まりは engine が一つの `MotionClock`（止まっていた時間の合計 `offset_ns`・最後に再開した時刻 `resumed_ns`）として持ち、
  `FrameTick` がその時点の写しを運ぶ（描けない間は tick を送らないので、止まりは再開後の最初の tick の `offset_ns` が増えることで伝わる）。
  `MotionState` は止まりを自分では持たず、tick の `MotionClock` を読む。`offset_ns` が増えたら animation を止まっていた分だけ送り、frame loop を tick の時刻から数え直す。
  reduced motion は tick ではなく、それを伝える `EnvChanged` の turn で適用する。動いている decorative な transition をその turn のうちに終え（`MotionState::reduce_motion`）、
  frame をもう一つ待たずに終わりの状態が見える。animation の開始や再 target の時は、環境の `reduced_motion` の memo を untracked で読む。処理中を伝える spinner（標準の button の loading、kit の spinner）は essential な動き（`Transition::decorative(false)`、または reduced motion で止めない `frame_loop`）として回り続ける。
- **canvas の描画。** `Draw` は wire の Scene を作り、変形は Arvane の `PushTransform` へ渡して core で平坦化しない。描けない値の命令は捨て、
  層ごとに一度 `DrawDropped` で数えて `Unsupported` として報告する。`Draw::region` は hit と a11y の leaf の幾何だけを与え、状態と入力は setup で付けた Part が持つ。
- **text の primitive（段落・IME session・a11y の text）。** native editor を使わずに `text_field` と同じ水準の editor を公開 API だけで作る口で、3つに分ける（`dev/tests/fixtures/text-primitives` が3つだけで 1 行の editor を作る）。
  - **段落。** `TextParagraph` は engine の text 経路が shaping した結果（`Arc<ShapedText>`）への `Rc` の handle で、`PartialEq` は handle の同一性。
    `Constraints::measure_paragraph`（measure 関数）、`Draw::layout_paragraph`（draw closure）、`primitive::shape_paragraph`（描画の外）が作り、すべて engine が lend する `TextShaper`（wire の trait。`measure` と `shape`）を通る。
    同じ text・style・幅は `TextEngine` の cache で一つの `Arc` を共有し、`Draw::paragraph` の `SceneCmd::Paragraph` は lower が同じ glyph の描画へ落とすので、測った glyph がそのまま描かれる（再 shaping はしない）。
    shaper は結果を `Arc` で返し、engine の借用は呼出しの中で終わる（利用者の関数へ借用を持ち出さない）。
    offset は UTF-8 byte（char 境界へ切り下げ）、幾何は段落の局所の論理 px。`caret`・`hit`（最も近い caret。行末の先は行の最後の caret）・`rects`（隣接 scalar caret の帯の和集合）・`lines` に答える。
    caret は scalar の境界ごとにあり、行を折り返す境界は次の行の先頭に属す。hit と `next_boundary`・`prev_boundary` は Unicode grapheme と shaping cluster の両方の境界に丸める。
    style の `align`・`baseline` は効かない（段落は先頭から置き、origin で置く）。
  - **IME session。** `NodeExt::ime_session(ImeSession)` は focus できる node（native editor 以外）に、確定済みの text・選択・caret（node の局所座標）を3つの `Value` として持たせる。
    core が1つの effect でまとめて `PropValue::ImeSession` として書き、engine は focus がある間だけ window の IME を許し、session の caret に候補窓を置き、session から `ShellImeText`（UTF-16）を答える。
    `Preedit`・`Commit`・`ImeSelection`・`ImeDisabled` は `HostEvent::Ime { node, event: ImeEvent }` として core に返り、`NodeExt::on_ime`（`Caller` の層）が受ける。
    composition は node が自分で表示する（session の text は確定済みで、IME に composition の範囲は渡さない）。native editor の経路は変えない。
  - **a11y の text。** `NodeExt::semantics_text(SemanticsText)` は node の text run を支援技術へ出す（native editor の `SemanticsText` と同じ publish の経路）。
    run の `id` は node の中での鍵で、engine が platform の識別子を割り当てて鍵ごとに保ち、変わらない run は同じ `Arc` で渡す。矩形は node の局所座標で、engine が window の座標へ移す。
    run がない・選択が run の外・矩形が有限でない値は `ErrorKind::Unsupported` として報告し publish しない（木全体を無効にしないため）。`TextParagraph::semantics_text` が段落から作る。
- **配置の語彙は CSS に揃える。** flex・grid・calc・sticky・fixed・transform・z-index は CSS の意味を基準にし、違いは11章の既知の制限に書く。
  border の幅は padding と同じく layout に入り（`box-sizing: border-box`）、色と形だけの変更は layout を起こさない。

### 文字の幾何の共有

`kivact-editing::TextGeometry` が `ShapedText` に対する caret、hit、選択帯と境界移動を一つの実装で計算します。`kivact-style::draw::TextParagraph` と `kivact-text::EditLines` はこれを呼び、native editor の行の chunk・cache・原点・alignment は `kivact-text` に残します。`ShapedText::cluster_boundaries` は Parley の cluster metadata に由来する scalar offset の整列済みの列で、0 と終端を含みます。移動の境界はこれと Unicode grapheme の積集合です。等しい caret 座標から cluster を推測しません。native editor の矢印キーの移動は `kivact-editing` の grapheme の規則（`prev_grapheme`・`next_grapheme`）のままで、hit と paragraph の移動は shaping cluster も守ります。この差は既知の残る制限です。

`primitive::shape_paragraph` は現在の theme の文字の既定値で App の Reader を介して shaping し、engine の借用を返却前に解放します。node 固有の継承値は持ちません。同じ実効入力は draw・measure と cache を共有します。

選択帯は native editor の隣接 scalar caret 間の帯を共有し、同じ行で接する帯を結合します。混在 bidi の真に分離した視覚範囲を表すための方向 metadata はなく、この制限は残します。


## 6. kivact と engine の境界（wire と bridge）

`kivact-wire` は node/event/scene/measure/純粋 motion と ID 用の hasher を定義する独立 crate で、engine と core が直接使う。
geometry・semantics・入力の語彙は ui-types で定義し、wire → core へ再公開する。bridge は共有値を再公開しない。共有値は window/renderer/Runtime に依存しない。
`bridge` の公開は Writer・Reader・Prop・Client・HostWaker・Boundary・WriteStats・Offscreen・OffscreenError・OffscreenImage・TextMeasurer だけに絞る。
wire は feature を持たない。外部 texture の scene/event variant と型は、どの build でも protocol の一部である。`gpu-interop` を持たない host と client は、それらを送らず、受け取っても無視する（feature はその crate の振る舞いだけを変える）。
`Writer`/`Reader` は同じ engine を `Rc` で所有する具体型で、各 API の間だけ model を借りる。書込み単位の backend trait や遅延 batch は置かない。
兄弟順の正本は engine の `NodeLinks` 一つで、構造の書込みはその場で反映する。通常は連続した子の配列を使い、
同じ revision の移動・途中の削除では配列を消費して一時的な連結順に切り替える。二つの順序を並行して保持しない。
commit・外部の geometry 読取り・layout の前で配列へまとめ、Taffy の子の添字アクセスは定数時間に保つ。
commit 前の focus・exit・hit の走査もその時点の正本を読む。`with_node` 内の入れ子の読取りは既存の共有 borrow のまま読める。
移動の書込み数、削除時の focus 移動、exit の間に占める場所は変えない。
起動の境界は engine の `run_native(Client, HostOptions)` で、Arvane の window loop を直接起動する。
App に backend 選択 API や第二の実装を加えない。

- **prop の表。** 型付き prop は `host/write/typed.rs` の `props!` の表の1行（`Prop` の variant、setter と値の型、保存先、無効化の `Mutation`）で宣言し、
  setter、bridge の `Prop`、型付きの値 `PropValue`（1 variant が1行の値の型）と `Writer::set(id, PropValue)`（setter を呼ぶだけ）を同じ表から生成する。
  core は `Tree::update(id, PropValue::X(v), writer)` で値を書き、`PropValue::prop` で書込み元の記録（`origins`）と `InputValue`・`Hidden` の特別扱いを決める。`Mutation` に既定値はない。
  setter は旧値比較後、変更時だけ変化を記録する。layout/paint/text/states/semantics は必要な node だけ Box で持つ。
- **commit。** focus の検査 → revision の更新 → event の境界の記録 → `Model::sync` の順。
  platform/window/frame の要求は通常の turn の後、font の登録は書込み時に処理する。
- **検証。** engine は core の書込みを信頼する。node ID の重複や0、親子の循環など局所的な不変条件だけを `debug_assert` で確かめ、別の検査用の木や invalid-write の報告経路は持たない。
- **HostEvent。** すべて単一の順序付きの列で、種類別の配列にしない。種類と field は wire の `HostEvent` の rustdoc にある。
  `Activate` は click と UIA Invoke だけを表す。`LayoutWarning` は debug build だけ。
- **HostInfo と能力。** 起動時に `HostEnv`（倍率・窓の大きさ・reduced motion・theme・display の周期・safe area・locale・fullscreen・描けるか）と時計と `Capabilities` を渡す。
  core は足りない能力を `tracing` に書き、`platform::capabilities` で App に見せる。
- **環境は一つの値 `HostEnv`。** engine は frame の group に `HostEnv` を一つ持ち、変更は `Model::update_env` だけを通す。
  値が変わったときだけ、全体を `HostEvent::EnvChanged(HostEnv)` として送る（倍率・大きさ・fullscreen・描けるかの変化も同じ event。`SurfaceLost`・`SurfaceRestored`・`FullscreenChanged` はない）。
  core は `Signal<HostEnv>` を一つだけ持ち、`platform` の accessor は field ごとの memo（`env_field`。accessor ごとに一つを共有）を読む。
  このため、ある field の変化で再実行されるのはその field の読み手だけになる。field を足すのは、wire の struct と accessor の2か所である。
  App が `App::locale` を指定しなかったときの locale は、`HostEnv::locale` の memo で、指定したときは定数である。
- **platform の要求の結果。** window の service の `Err` は `EffectResult` の `Failed(理由)` で `ErrorKind::Platform`、
  service がない（`ShellError::unsupported`）ときだけ `Unsupported` で `ErrorKind::Unsupported`。
- **GPU texture 共有（feature `gpu-interop`）。** canvas の `Draw::gpu_frame` は wire の `SceneCmd::External` で、paint が Arvane の `DrawCommand::External` に下げる（`Pixels` と同じ rect・source・opacity・pixelated。層の color filter は掛からない）。
  core は App ごとに一意な `ExternalImageId` を振り、`Writer::import_external(id, SharedTextures)` と `Writer::drop_external(id)` を書く。
  `SharedTextures` は OS の handle を持つので `Clone` できない値のまま model の列で運び、host が turn の後に書いた順で `Window::import_external_image`・`drop_external_image` に渡す（window は client の start から在る）。
  取り込みの答えは `HostEvent::ExternalImported`（`Ready { generation }`・Arvane の `Unavailable` は `Unavailable`・`Content` と `Backend` は `Failed`）で、core が `ImportStatus` の `Ready`・`Failed(Unsupported)`・`Failed(Platform)` にする。
  host は pump のたびに `Window::gpu_adapter` を読み（native と試験の window で同じ）、前に送った値と違えば `HostEvent::GpuAdapterChanged` を送る。core はそれを env の key と同じ追跡される値に持ち（起動時は `None`）、`Ready` の取り込みのうち世代の違うものを `Lost` にする。
  owner の破棄で core が `drop_external` を書く。描けなかった frame（`RendererChange::ExternalFrameMissed`）は frame log に残し、公開 API にしない。
  engine の thread と UI の間（`FramePublisher` と `ShareLink`）は、core の中の共有状態（mutex と条件変数と、UI の task の waker）だけで結ぶ。engine の thread の呼出しは状態を変えて UI の task を起こし、UI の task が turn の中で signal に書く（`latest_frame`・`frame`・`pixels`）か `import` を呼ぶ。bridge・wire・host には何も足さない。
  高水準の `shared_image` は、取り込みごとに image の owner の子 owner を作って `import` し、その子 owner の effect が `ImportStatus` を link の状態と `ShareStatus` に写す。新しい要求で子 owner を破棄するので、外す経路は低水準と同じ `drop_external` だけになる。

engine が守る入力の契約（`kivact::test` の E2E が本番の engine の経路で確かめ、OS の入力を通す部分は GUI harness が確かめる。10章）:

- **入力の handler の答え。** key・wheel・pointer の handler は `Handled` を返す。`Yes` は伝播と既定動作を止め、`No` はどちらも止めない。
  `PreventDefault` は DOM の `preventDefault()` に当たり、伝播は続けて既定動作だけを止める。
  既定動作は key の押下では Enter/Space の activate と Escape による overlay の Dismiss、wheel では core の `ScrollBy` である。
  Tab の focus 移動と native 編集は host が handler より先に処理する。ContextMenu キーは `Key::ContextMenu` を送るだけで、既定動作を持たない。
- **core の handler の層。** node の key・key release・pointer・wheel・file の handler は、event の種類ごとに一つの順序つきの列（`view::handler_layers` の `EventHandlers<E>`。要素は `(Layer, handler)`）で持つ。
  配送は `deliver` 一つで、`Layer` の順に（層の中は列の順に）呼ぶ。`Yes` は後の層を止め、`No` は次へ進み、`PreventDefault` は次へ進みつつ残り、既定の動作だけを止める。`Err` は止めて、今までどおり dispatch の境界で一度だけ報告する。

  | 層 | 中身 | 呼ばれる条件 |
  |---|---|---|
  | `Observer` | part の状態の観測（押下・hover・key の押下）。結果は返さず、consume しない | 選ばれた宛先（region の event なら region、そうでなければ node 自身。子孫から bubble した key は観測しない）が生きている間。後の層が `Yes` や `Err` を返しても必ず先に走る |
  | `Caller` | アプリの `on_*`。後に登録したものが先 | event が node に届いたとき（宛先の状態に関わらない） |
  | `NodeDefault` | node 自身の `on_*_default`。登録順 | `Caller` と同じ。canvas の region に向いた event も受ける。`part(part)` の前後どちらで登録しても part の既定より先 |
  | `PartDefault` | 付けた part の既定。付けた順 | 宛先が有効なとき（pointer は Cancel と Leave なら無効でも）。宛先は node か、event が持つ region |
  | `RegionDefault` | `Part::region_events()` を選んだ part の、未処理の region 入力への fallback | 宛先が region で、その region と node が有効なとき（pointer の Cancel は node が無効でも） |

  `file` は `Caller` だけを持ち、`on_file` は前の handler を置き換える。`activate`・text・`on_a11y` の action・selection・scroll・layout は一つの handler で、後の登録が置き換える（part の既定は付けた順に合成して、caller の後に呼ぶ）。
  part も canvas の region も同じ形の列を持つ。part の `on_*` は、node に付けたとき `PartDefault`（`region_events()` なら `RegionDefault` にも）、観測は `Observer` として node の列に入る。
  部品や region を持つ node は、mount で列を二つに分ける。`Caller` と `NodeDefault` は node の列に残り、`Observer`・`PartDefault`・`RegionDefault` は node 自身の宛先に移る。node は event ごとに宛先を選ぶ router を一つだけ登録する。
  pointer の hover の移り変わり（region から region へ動くときの `Leave` と `Enter`）は、それぞれ別の event として、観測と part の既定へ順に配る。移り変わりの `Yes` は次の移り変わりも元の event も止めず、region の fallback へ進むかどうかは元の event の答えだけで決める。capture と hover の状態機械は `part_pointer` が持つ。
- **wheel。** hit した node から最寄りの wheel listener の手前まで engine が scroll し、listener に届いたら `Wheel` を送る。
- **pointer。** capture は pointer ごとに持ち、必ず終わらせる（窓の外の `Up`、見逃した離しや非活性で `Cancel`）。
  listener の `PointerEvent` は、変わった button を `button`（`Down`・`Up` だけ）に、押されている button を `buttons` に分けて持つ。button を押した `Move` は drag、押していない `Move` は hover で、`Cancel` とその後の `Leave` は何も押していない。
  touch と pen の接触も同じ状態機械を通り、primary（Pointer Events の `isPrimary`）の pointer だけが hover・focus・click を作る。
- **Tab と focus。** Tab は engine が処理して `FocusChanged` を送る。
  focus 中の node が消えた・disabled・hidden になったら commit で1回だけ次の tab stop へ移す。
  Tab・host による focus 移動・`request_focus` は対象を scroll container の中へ入れる。
- **modal と Dismiss。** modal は表示中の最後のものが有効で、hit・探索・Tab・focus はその中に閉じる。leaving の部分木は modal の範囲にも数えない。
  入力の受理（`Reader::admit_input`。modal を開く前に始まった drag の継続を覚える表は `Model` にあり、呼出の間だけ借りる）・Escape の宛先（`dismiss_target`）・復帰先の可否（`can_take_focus`）も engine が答え、core は尋ねるだけにする。閉じたときの復帰の要求を積むのは core の役目。
  Dismiss を持つ overlay の部分木と anchor の外の pointer の `Down` では、press より先に `HostEvent::OverlayDismiss` を送る。
- **overlay の配置。** engine の `place_overlay` が anchor の外接矩形か窓の点に置き、入らなければ反対側へ回し、最後に window の中へ押し戻す。
- **古い event。** 古い node・世代違いの event は core が捨てる。

## 7. engine（`kivact-engine`）

kivact が turn の中で書く木を持ち、Arvane の window で動かす1つの crate です。振る舞いは6章の契約と、この章の規則です。

| module | 役割 |
|---|---|
| `bridge` | 所有型 Writer/Reader、Prop、具体的な Client と waker、commit 境界、統計、Offscreen と TextMeasurer（6章） |
| `native` | 初期設定 HostOptions（起動は root の run_native） |
| `tree` | 型付きの生 prop、scene、安定した NodeIndex、親子リンクと構造の index。派生の layout・paint・text の型は持たない |
| `measure` | 葉の資源計測、文字 extent と画像の natural size の列、native editor の型。木には計算結果を書き戻さない |
| `mutation` | Writes・WriteStats・Mutation の受動的な記録。Model や host の処理は参照しない |
| `model` | 単一の Model（NodeTree・派生列・assets・editor・EventQueue・Changes と状態 group）と、focus・pointer・編集・key・modal・dismiss・状態 style・semantics・event の意味 |
| `model::interact` | sync・visual・矩形・hit・scroll・overlay・sticky・文字/IME の geometry を処理する Model の層 |
| `text_measure` | client の文字計測。木と Editor に結び付く geometry は model::interact が処理し、行の保持と lazy shaping は kivact-text を使う |
| `layout` | 構造だけを借りる LayoutTree adapter。純粋な計算と列の型は kivact-layout、列の所有者と資源 measurer の組合せは Model |
| `paint` | paint の状態と一覧、木をたどる scene の組立て。値の変換・局所 scene は kivact-lower の Context を使う |
| `a11y` | dirty node の accessibility payload 構築と公開（受理済み fingerprint との比較） |
| `offscreen` | `bridge::Offscreen`: window なしの画像出力 |
| `check` | 文字の layout の崩れの検査と debug の lint |
| `host` | window の loop、shell の event の振分け、書込みの API と型付き prop の表、資源の完了と診断の配送、試験 driver |
| `frame_log` | `KIVACT_FRAME_LOG` の frame の時刻 |

公開するのは `run_native` と `native` の HostOptions、`bridge` と、feature `testing` の `testing` で、
ほかは `pub(crate)`。
layout 結果の意味付け・hit/scroll と scene の木の走査は同じ Model の node と位置を使うため engine にまとめる。BoxShape・effects・text の値変換と局所 scene の lowering は kivact-lower に置く。

- **意味は model に置く。** window・描画・shaping を知らない「木の意味」（focus・Tab・modal・Dismiss・pointer・native の `EditState`・key の解釈・状態 style・cursor・semantics の方針）は `model` に置き、
  `paint`・`a11y`・`host` はその状態を Arvane に写すだけで意味を決めない。
- **状態の group。** 状態は group ごとに、それを書き換える module の下の葉の `state` module に置く（`Focus`・`Pointer`・`Edits`・`StateMotion`・`Frame`・`PaintState`・`A11yState`・`Lint`）。
  field は `pub(super)` で、その group の module だけが直接書く。葉は処理側を参照しないので、module の参照に循環ができない（xtask が検査する）。
- **配送は一つ。** 未処理の変化を集めるのは `Changes`、layout/paint/a11y へ配るのは `Model::sync` だけ。asset の完了・入力・tick・resize も同じ配送を通る。
  `Changes` は node ごとの未処理を一つの map の `Dirty` の bit（`WRITTEN`・`CHILDREN`・`REDRAW`・`MEASURE`・`LAYOUT_STYLE`・`LAYOUT_CHAIN`・`SCROLLED`・`STUCK`）で持ち、木全体の変化（`moved`・`needs_layout`・`restack`・`fonts` など）は flag で持つ。
  node を取り除くときは、その node の bit を一度に捨てる。
  型付き prop から汎用の写しを作らない。`ink`・`paint_look`・`extents`・`natural`・`computed_text`・editor・layout の派生状態は
  Model が所有する NodeIndex の列に置き、生 prop と親子関係は NodeTree だけを正本にする。
  継承した文字の style（`computed_text`）は、node の最初の読み取りで祖先から一度だけ解決して持つ。
  `TextStyle` の書込みはその部分木の分を、挿入と移動は動かした部分木の分を、その書込みの中で捨てる（次の読み取りが解決し直す）ので、
  同じ revision の state motion の取込みも新しい style を読む。paint と測定は祖先をたどらずにこの列を読む。`outdate_paint` は
  tree・Changes・自身の列を受ける PaintState の操作で、PaintState から Model へ参照を戻さない。
- **仕事は変化と見えている範囲に比例させる。** 通常の1 node の変更で全木を走査しない。layout は内容の変化を `Measure::measures_same` で証明できたときだけ飛ばし、
  幅と高さから推測する別の経路は持たない。paint の一覧は窓と clip の外を ink で culling する。
  paint の一覧を作り直すのは、構造・可視・大きさ・overlay の書込み、移動（`moved`）、そして一覧の材料が変わったときだけにする。
  材料は node の `Place`（transform・z-index・sticky・fixed）、paint の `Look::stacking`（不透明度・layer・色の行列・描く物の数・角丸・影と ring のはみ出し・border・filter の届き・clip）、
  state style を持つ node の target paint の同じ事実、文字の届き（大きさと影）である。
  色だけの `Paint`・`TextStyle`・`States` の書込みは、その node を再描画の集合へ入れるだけで一覧を作り直さない。
  debug の差分と全部作り直しの比較（`paint_self_check`）が、この絞込みの正しさの網である。
  祖先の移動によるピクセル丸めで、同じ大きさの親の下でも子の位置や大きさは変わりうる。丸め後の差に応じ、大きさや content の範囲などが変わった node 自身、位置だけなら親を再描画の集合へ渡す。ink を祖先まで無効化し、child index は新しい位置と ink を反映する。
  absolute の箱は browser と同じく layout の root で、その style や中身の変化は親だけを前回の入力で計算し直し、親の大きさ・baseline・overflow の届く範囲が変わったときだけ祖先へ広げる（[kivact-layout の traversal](../../crates/kivact-layout/src/traversal.rs) の rustdoc）。
- **accessibility は一経路。** Changes が own-node・subtree・構造・focus・削除の dirt を記録する。layout は実際に変わった元 node ID を paint 用の親へ写す前に保持し、scroll・transform・hidden・modal と overlay anchor の依存を部分木へ広げる。公開は ID/親/primary children 順序の hash・clipping・逆 relation の metadata を列挙し、展開した dirt だけを同じ build_node で payload にする。通常の1文字編集で静的な他 node の payload を作り直さない。 plain Content と既存 native editor の content/selection/preedit は metadata_refresh が不要なとき、不変の Rc<Metadata> を薄く借用して dirty/reverse relation source だけを構築する。未知の prop・混在変更・構造/focus・geometry/subtree・scene/region・font/env では metadata を再列挙する。local path は全 preorder Vec、metadata の deep clone、全 hash prune を行わず、full refresh の受理時だけ snapshot を交換する。
  受理済み payload tree の所有者は arvane-shell::AcceptedTree だけで、engine は完全な payload fingerprint・root/focus・単調割当の region/run ID と dirty metadata だけを持つ。native editor/shaper の現行描画 cache は保持するが、authored text の前回受理 Arc mirror は持たない。fingerprint は全 field（float は bit、children/relation/run の順序と内容を含む）を比較する最適化 metadata であり、action の受理 authority ではない。
  window が受理したときだけ fingerprint を確定し dirt を消す。拒否は旧 fingerprint/dirt を保ち、次回を full resync にする。初回・再活性化・root 変更も full replacement。synthetic ID は拒否前に割当てても再試行で保持し、不要 metadata は受理後に除く。構造 metadata の列挙と clipping 判定は O(N) のまま残る。 root のない window も専用の安定 synthetic ID の Generic root 1個を同じ受理経路で公開し、古い AT node を残さない。engine はこの ID だけを保持し、payload は保持しない。空 window の viewport 変更と次の実 root も公開し、独立 debug 比較を省略しない。
- **platform を知らない。** OS・GPU・CPU の違い（renderer と GPU の fallback、frame の予算、display の周期、click の回数、隠れた window、
  環境変数）は Arvane が吸収する。
  engine に platform 別の分岐・renderer の上限・GPU の状態はない。engine は Arvane の `Window` を再抽象化せず、
  native と試験の window は同じ `Host::begin`/`handle` を通す。
- **描画の結果。** host は scene を `Window::present` に渡して結果（`Shown`・`Skipped`・`Dropped`・`SurfaceLost`）だけを受け、`Err` のときだけ失敗で終わる。
- **起動時の検証。** window の大きさは kivact の `App::run` が冒頭で1回だけ検証し、不正なら window を開かずに `ErrorKind::Message` を返す（platform に黙って丸めさせない）。
- **文字の layout の検査。** `check` は engine が持つ形の情報から、はみ出し・意図しない折返し・縦の中央のずれ・clip で切れた文字を判定する。
  試験 driver から呼び、`Screen::check_layout` が公開する（10章）。debug build では変わった文字だけを同じ規則で調べ、`HostEvent::LayoutWarning` を送る。



### 値の lowering と資源の所有

`kivact-lower` は tree や Model の再抽象化を持たず、wire の値・明示的な形・文字の style・局所 scene と Arvane の画像/文字 service だけを受ける。外側の木の走査・paint list・ink cache・editor の行・DrawDropped の node/layer ごとの再通知は engine が所有する。TextMeasurer と Offscreen の公開署名にも Arvane の型を出さない。

ArvaneAssets は Model.assets: RefCell<ArvaneAssets> が一つだけ所有し、store は private に保つ。 Model.assets は Model.paint の直前に宣言し、以前の PaintState.assets と同じく events・motion・presence の後、paint cache の前に AssetStore/Worker を破棄する。begin_frame は recency だけ、replace_store は store だけを置き換えて font と診断履歴を保存し、discard_failures は失敗列だけを消費する。poll の完了→eviction→失敗の消費順と、pixels の要求後に結果がなくても eviction を消費する順を維持する。画像は shared Asset を直接参照し、完了時は NodeTree.images の index から再測定対象を探す。

Context は資源と raster scale を借りる UI-local な一操作で、描画に必要だった最初の資源失敗と drop 数だけを記録する。window は資源失敗の周りを描き続け、Offscreen は drop→資源失敗→rasterizer の失敗の順で返す。診断の位置と配送は engine の責任に残す。

### slot の寿命と利用者の値の破棄

Model は構造 slot を reserve し、新規位置なら全ての派生列と Option<Box<EditorNode>> の列を一度だけ延ばしてから node を install する。
削除は subtree の値と slot を take し、calc handle・measure・paint・editor の列を reset してから free へ返す。再利用時に前の node の cache を読まない。
EditorNode は必要な node だけ Box を確保し、tree に editor や EditLines の例外を置かない。

構造削除は private な `NodeTree::remove_with` に中立な callback を渡す。Model は分割した列を reset し、raw node・editor・extents の所有 payload を返す。tree は同じ node の index を片付けて slot を free に戻し、その payload を以前と同じ親→子の各反復境界で drop する。Writer の統計記録より前で、Model の借用中であることも以前と同じ。構造削除の利用者 Drop による Reader 再入失敗は既存の未解決事項として残し、この機械的移動で変更しない。

`Editing` の差替えだけは別途検証済みの経路を保つ。`set_edit` / `editing_changed` が古い `Option<Box<EditorNode>>` を返し、Writer は Model guard を外してから setter が返る前にその一値を drop する。待ち行列や commit への遅延はない。

Model の終端 drop は元の slot 順で raw props/scenes → editor → extents をまとめて解放する。中立な残り slot の guard は一つの destructor が unwind しても残りの順序を保つ。もう一つの destructor が panic すれば通常の drop glue と同じく abort し得る。派生列の allocator/cache 解放時刻の完全な同値や性能は推測せず、実際の検証で確認する。


### 行 cache の source と整合性 checkpoint

kivact-text は Source の value/placeholder を借り、revision と UTF-8 byte の damage で hard line を更新する。Editor の文字列・変更履歴と EditorNode の所有は engine に残す。cache を try_borrow_mut できてから一度だけ take_damage し、同じ source/style snapshot を更新と参照計算に使う。借用中は None とし、damage を消費しない。font 登録後の fallback の変更は engine の既存の無効化で行を破棄する。

通常の measure/paint/key/IME/a11y は RefreshMode::Incremental。Verify は独立した利用者も batch/adoption 後に明示できる通常の更新契約で、debug build だけで全 split/prefix/scalar/UTF16/totals を比較して RefreshError を返す。unchanged revision の no-op 更新でも検査する。release は検査の code・走査・参照用 allocation を compile out し、Verify も通常の更新だけを行う。mode は source/style Key に入れず、検査失敗で本体を reset/heal しない。

Screen の既存の settled debug test-host 呼出しは、KIVACT_SELF_CHECK=0 の return 後だけ Verify を選び、node 文脈付きの証拠を通常の失敗経路へ渡す。通常 product debug の各編集に全走査を足さない。追加の normal-query 参照は retained の clone と fresh な disposable cache だけを使い、全行の range/count/boundary と bounded な geometry/neutral shape/publication を比較する。semantic ID は局所 allocator を使い、比較は ID/Arc allocation identity を除く値で行う。LineColors callback は再実行せず、新しい色付き glyph の参照比較は行わない。

byte は UTF-8、caret/公開 start/end は scalar、platform 変換は UTF-16 code unit を区別する。LF と直前の CR は終わる行に属し、最後の LF の後には空行がある。password は表示だけの mask で、raw excerpt/run を安全に隠す責任は engine が持つ。自然幅は最初の16KiBから始まる行、IME は周辺2048byte、公開はcaretと描画帯の周辺32行/32行stepを維持し、全段落の eager shaping はしない。

### native editor の所有権

`kivact-editing` の `Editor` は behavior に渡す `EditState` を所有し、host の controlled value・変更記録・damage の操作を受け持つ。記録の `log`・`base`・`damage` と編集時の最大長は `EditState` 内に一つだけ保持する。最大長は `Editor` の control accessor で設定し、behavior が `&mut EditState` を操作する間も同じ値を使う。core は `Editor` を再公開しない。

node の編集状態は `Option<Box<EditorNode>>` とし、Editor・行の cache・IME 中の保留・通知済み session・水平 text scroll・controlled value の revision をまとめる。通常の node は行の Vec を持たない。行の cache は Editor の外に置き、後の text crate との間に循環を作らない。clone は記録・damage・行の cache を独立に複製し、debug の参照計算が本体を変更しない。


### retained layout の境界

layout の計算と派生列は `kivact-layout` に置き、通常の公開契約 `LayoutTree` と `LeafMeasure` を通して実行する。
engine は唯一の木と資源を所有し、adapter・幾何の統合・full rebuild の検査を残す。
6b で crate を抽出した時点で `pub` に昇格し、engine から一時的な再公開はしない。
`LayoutTree` は唯一の node store の raw props と構造を借りる11操作であり、engine の
`NodeIndex` は構造上の位置の型として残す。境界では一つの整数 slot を変換するだけで、
子の列を複製せず、別の topology・ID map・unsafe cast を持たない。 通常の write 中の
Packed/Editing child order は engine の唯一の順序のまま保持し、semantic reader は active
iterator を使う。layout 開始時だけ通常の pack barrier を通し、adapter の child_at は
packed slice を定数時間で引く。これを毎回先頭から歩く iterator に置き換えない。fixed の候補は既存の
`BTreeSet<NodeId>` を借り、順序・range cursor と detached 候補の扱いを保つ。

`NodeLayoutColumns` が style・calc arena/所有 handle・measurement record・Taffy cache・
unrounded/rounded geometry・performed input・flow cursor を所有する。slot の remove/replace は
handle を解放して全派生列を reset してから node を落とし、free slot を再利用する。
`LayoutPass` が Taffy trait を実装し、content 測定は engine の adapter に委譲する。
親依存の invalidation、absolute parent の overflow reach による shortcut、fill root の
compact/grid 両 style と cache の更新、root→overlay→fixed の順序を保つ。

丸めた geometry の変化は `Local` / `PositionOnly` と NodeId の通常の結果として返す。
engine は local の ink を無効化し、位置だけなら layer を除く親の union だけを無効化する。
`Changes::laid_out` は変化列が空でも計算ごとに一度呼び、child index の位置更新を保つ。
参照比較は破棄可能な clone を raw style/hidden と topology から full rebuild し、geometry の
全フィールド（order 以外）と baseline を含む出力・測定を厳密に比べる。Taffy cache の
getter や testing-only public API を設けず、通常の opaque geometry/比較操作を使う。
既存の debug settle と opt-out、paint cache の完全な復元、text の Incremental/Verify の
cadence は変えない。dyn の前後比較で10%超の悪化候補が出たため、木の型を
`Tree: LayoutTree + ?Sized` として計算と iterator に通す実装を選ぶ。engine は具体的な
NodeTree のまま渡し、LeafMeasure は dyn の借用を保つ。木・子の列・ID map の所有や
計算本体は変えない。これは dyn だけを原因と断定するものではなく、指示された切替後に
正しさと性能を再検証する。6b の抽出後は caller 側でも実体化するため、engine と
layout の両方の開発時最適化を保つ。抽出前後の性能は受け入れた source を固定して比べる。

## 8. engine と Arvane の境界

Arvane facade は window・text・描画・shell の振る舞いを公開し、geometry・入力・描画命令などの値は `ui_types` から直接使う。`arvane::asset` は画像と font のサービスに必要な 11 名だけを明示的に再公開する。

Arvane は外部依存のない値（ui-types）、中立な shell の契約、4 adapter、公開 facade に分ける。

| crate | 役割 |
|---|---|
| `ui-types` | 論理 px の幾何、semantics、描画と文字の語彙、入力の語彙、Environment と Capabilities。外部依存・I/O・thread・global 状態を持たない |
| `arvane-shell` | shell の event・設定・error・wake・IME 周辺 text・dialog の語彙、click・contact の正規化、受理済みの accessibility の木と action 受理、window・handler・event loop と GPU の target の SPI（`ShellWindow`・`ShellHandler`・`ShellEventLoop`・`ShellChild`・`GpuTarget`）。依存は ui-types と raw-window-handle（0.6）、feature・unsafe・OS の実装を持たない。backend は native 入力を logical px の値に直して渡し、試験の window も同じ正規化を使う |
| `arvane-text` | Parley による shaping・bidi・改行・font fallback と、shaped snapshot・計測・family query の cache。登録成功時の無効化と cache 解放を一元管理する |
| `arvane-render` | CPU/GPU の描画（vello の共通の sparse strip pipeline。最後の pixel 塗りだけ vello_cpu と vello_hybrid に分ける）と scene の検証 |
| `arvane-asset` | 画像の形式判定・decode・SVG の raster・LRU と worker・font の検査 |
| `arvane-native` | winit の event loop と IME、softbuffer、AccessKit、clipboard、file dialog、path を開く操作、OS の設定と native 入力と accessibility の変換。arvane-shell の SPI を実装し、portable な語彙・入力正規化・受理済みの木は arvane-shell を使う。実験的な `embed` は親 window 内の子 window の入口（Windows と Linux X11 に対応） |
| `gpu-share`（feature `import`） | 共有の契約と検査を export 側と共有し、`import::{identity, request_device, Import}` で wgpu の下の GPU texture 取り込みを行う。共有 texture と timeline fence を取り込み、wgpu の queue に acquire と release を直接出す。`arvane-render` が使い、上面の facade に wgpu・ash・windows の型を出さない |
| `arvane` | adapter へ依存する唯一の公開窓口。`Engine`（text）、`Window` trait（native と試験の window）、presenter（renderer・GPU の寿命・fallback・frame の方針）、`Rasterizer`。実験的な `embed` は `arvane-native/embed` へ転送する |

- **API と SPI。** `ShellChild` はホストの loop 上の子 window の寿命と物理サイズ・倍率・失敗の口を持ち、drop で終了を要求して待機上限の下で後始末を進める。 API = `arvane` facade（engine が使う）、SPI = `arvane-shell`（backend が実装する）。二つ目の実装は headless の shell で検証する。native の backend（`arvane-native`）と、試験の window の headless な shell（`arvane` の `test_display`。実 display への mirror は任意）が同じ `ShellWindow` を実装し、facade の `NativeWindow` と presenter は具体型でなく trait を通して使う。scene・描画・readback・frame の方針は facade と render の責任で、shell は試験専用の API を持たない。`ShellEventLoop::run` は借用した `&mut dyn ShellHandler` を同期的に呼び、handler に `'static` を求めない。winit の event loop は process に一つという制約は変えず、試験の実 window の display は `arvane-native` に残す。
- **surface を作る thread。** `ShellWindow::with_gpu_target` は owned な callback を target の UI thread で実行する（display の thread の queue を経てもよい）。GPU の worker が native の handle を取り出す getter は設けない。`GpuTarget`（`Arc<dyn GpuWindow>`）は window の寿命を保つだけで、handle に触れる許可は与えない。生成の結果は surface maker と response channel で戻す。headless の shell は native の target がないことを unsupported の error で返し、facade は従来どおり offscreen の GPU と readback を使う。CPU の提示（`present_with`）は object safe な callback が bool で commit を決め、描画の error は呼出し側が保持する。backend の error と描画の error を混ぜない。

- **`unsafe`。** Arvane の `unsafe` は `arvane-native` と `gpu-share` に閉じる。`arvane-shell`・`arvane-render` と `arvane` は `forbid(unsafe_code)`。
- **ui-types。** Kivact にも Arvane にも属さない中立な値の crate で、第二の接続層（戦略の P-10）も Arvane に依存せずに同じ語彙を使える。
  幾何（`Point`・`Size`・`Rect`）・semantics（`Role`・`A11yAction`・`LiveRegion`）・cursor・画像 fit・描画の stroke と混色と gradient・文字の行高と装飾と折返しと省略と変換・
  platform の能力と safe area・入力の語彙（`Key`・`KeyCode`・`Modifiers`・`MouseButton`・`PointerId`・`PointerKind`・`PointerPhase`・`PointerButtons`・`PointerInput`）を一度だけ定義する。
  kivact-wire と core の公開窓口は同じ型を再公開し、一致変換を持たない。
  意味の違う値（CSS の spread/inset を含む `Shadow` と Arvane の `BoxShadow`、分解した `Transform` と行列）は分け、
  lower が下げる（`kivact-lower` の conversion・border・effects）。変形の意味と行列の組立ては engine の `model/transform.rs` に残す。

詳しい owner と無効化の契約は [保持した scene の描画](architecture/retained-paint.md) にあります。

- **glyph の共有。** shaped snapshot は immutable な `Arc<GlyphRun>` の列を持ち、draw command は run と論理 origin・任意の色 override だけを渡す。lower は glyph 座標や variation の配列を frame ごとに複製せず、CPU/GPU 共通の draw が配置を適用する。長文の shaping 合成による座標変更は新しい snapshot の構築時に終える。
- **local scene cache。** `lower::SceneCache` を各 Model の PaintState と engine Offscreen が1個所有し、短命な Context は第三の borrow として受け取る。128 entry の LRU は Weak scene 同一性・command 数・継承 text 全体・raster bits・Tint を照合し、成功した origin zero の lowering だけを再利用する。配置は外側の balanced transform とし、外部 frame は保持しない。asset poll の変更/失敗・store 交換・font 登録成功の revision で無効化し、hit ごとに natural/pixels を再解決して使用履歴と pixel 同一性・寸法を確かめる。pending・error・drop は保存しない。直接 Arvane Engine へ font を登録する lower caller は SceneCache を明示 clear する。
- **scene depth と配置。** `arvane::MAX_SCENE_DEPTH` が唯一の group 上限で、Kivact facade には公開しない。SceneCache の owner は新しい append-only output ごとに `begin_frame` を呼ぶ。cache は output cursor 以降だけを走査して親 group の open 数を追い、payload の最大深さと配置1段を加えて上限を超える場合、hit を含め通常の origin 付き lowering に戻す。これは最適化の可否だけを決め、構造の認証は render に残す。試行の error/drop は fallback 前に復元して二重報告しない。
- **font validation metadata。** render の各 CPU/GPU owner は font bytes の Weak 同一性・face index ごとに成功した glyph count だけを256 entry LRUで保持する。glyph 座標・variation・color・origin の合成は毎回検証する。MemoryWarning は Window→Presenter→renderer で metadata を解放し、live import/lease は残す。独立 engine Offscreen は通常の `release_caches` で scene/text/unused asset/validation を解放する。core draw Offscreen は既存どおり drop が解放境界で、global な memory warning を購読しない。
- **scene の境界検証。** `arvane-render` の `validate` を唯一の scene 境界検証者とする。lower の sanitize は CSS の長さ・色・半径などの意味を補正し、局所 command を従来の規則で穏当に drop するためのもの。完成 scene や font が描画可能であるという保証や第二の境界検証を提供しない。
- **所有。** OS 差は Arvane の Window/Shell が正規化し、GPU/CPU 差は facade の presenter と render adapter で終わる。
  Kivact の node・layout/hit/scroll・編集・style/motion・scene の意味は engine が決める。Arvane は wire を含む Kivact の型に依存しない。
  論理座標・直線 alpha の入力 pixel・premultiplied frame・text の scalar/glyph と編集 byte offset・Arc の同一性は型の doc に記す。
- **SPI の方針。** facade は外部 crate の型と renderer の寿命を公開しない。vello・peniko・kurbo・winit の型は出さない。
  試験の platform failure は本番と同じ Result の経路で返し、Window に試験専用の取出し API を足さない。
- **accessibility の受理と native の橋。** `arvane-shell::Accessibility` は木と差分を検証し、拒否した更新では階層・focus・text run を変えない。`AcceptedTree` は root・focus・node・parent・親から子の順の nodes の読取りを公開する。`PlatformTree` は活性化・停止・未送信の node/text run と action queue を所有する。window に結び付ける `Accessibility::of_window` も、platform の木の構築も通常の API で、testing feature を要らない。
  native の橋は adapter の update callback の内側で `PlatformTree` の変換 closure を実行する。adapter → shell の順で lock を取り、最新の受理木から native update を lock 内で作るため、活性化のあとに古い node を渡さない。未送信状態は変換が成功してから消し、panic 後の lock の回復でも未公開の変更を残す。変更通知と wake は lock の外で行う。停止中は差分をためず、再活性化で全木を渡す。
  action は node と全祖先が hidden・disabled でなく、宣言したものだけを受ける。focusable は `Focus` の宣言を兼ねる。`Activate`・`Toggle`・`Select` は既定の起動の宣言として同等で、UIA Invoke や Android ACTION_CLICK を含む native の既定の action も `Activate` として届く。native は OS の語彙と payload を変換し、注入と同じ中立な受理規則を通す。
- **GPU texture 共有（feature `gpu-interop`）。** アプリの render engine（自前の device。wgpu でなくてよい）が描いた texture を、CPU に読み戻さずに window が描く。
  対象は Windows の Direct3D 12 同士と Vulkan 同士、Linux の Vulkan 同士（OPAQUE_FD）で、API の違う組・DMA-BUF・macOS は対象外（他の OS では feature があっても何も取り込まない）。
  語彙は `ui-types`（`GpuPreference`・`GpuAdapter`・`SharedTextures`・`ExternalFrame`・`DrawCommand::External`）にあり、OS の handle は std の `OwnedHandle`・`OwnedFd` で渡す。
  adapter と backend はアプリが希望を出し（`Shell::gpu_preference`。`WGPU_BACKEND`・`WGPU_POWER_PREF` が優先）、Arvane が実際に使ったものを `RendererChange::Gpu` の `interop` と `Window::gpu_adapter` で知らせる。device を作り直すと世代が進み、取り込みはすべて無効になる。
  取り込み（`Window::import_external_image`）は window の device に属し、別の window の scene が同じ id を参照しても何も描かない。
  frame ごとに、ready fence を CPU で確かめて（最大 100 ms 待つ）、届いた frame だけを acquire して描き、submit の後で release する。release fence には、その window で最後に描いた値 − 1 を単調に signal する。
  共有する texture の RGBA8 は sRGB で符号化された値（premultiplied alpha）で、`Pixels` や CSS の色と同じ符号化として合成する。線形の値で描く engine は最後に sRGB へ符号化して渡す。sRGB の view では標本化しない。
  共有する texture は、engine が描画先・copy・sample に使える（Direct3D 12 は `ALLOW_RENDER_TARGET`、Vulkan の usage は `TRANSFER_SRC | TRANSFER_DST | SAMPLED | COLOR_ATTACHMENT`）。`SharedTextures::storage` が真のときは storage の書込みにも使える（`ALLOW_UNORDERED_ACCESS`、`STORAGE`）。
  Vulkan では、engine は作る前に、Arvane は取り込む前に、その usage と handle の種類で `vkGetPhysicalDeviceImageFormatProperties2` を呼んで対応を確かめる（Arvane は対応がなければ `Content` で断る）。
  Direct3D 12 では、Arvane が resource の flags を `storage` と照合する。Vulkan の memory は image の usage を持たないので、usage の食い違いを Arvane は検出できない。
  engine との約束（作成条件、受け渡しの状態、値の意味、時間切れ）は `ui_types::SharedTextures` の rustdoc、frame の手順は `arvane-render` の `gpu/external.rs`、`unsafe` の手順は `gpu_share::import` の module doc にある。
  engine 側の約束は `gpu-share` の feature `export` が肩代わりする。release は GPU 側で待ち（engine は window より先に描ける）、CPU では、その slot の前回の ready だけを時間切れ付きで待つ。届かない release は、pool が一巡した後に時間切れの error になる。手順は同 crate の crate doc にある。
  wgpu（Kivact と同じ 29.0.4）で書いた engine には、同 crate の feature `wgpu` が device の作成（Vulkan は共有の extension を足して開く）、`wgpu::Texture` の pool、1 frame の submit（生の submit による受け渡しの間に engine の command buffer を wgpu で submit し、wgpu の tracker を受け渡しの状態に戻す）を肩代わりする。生の submit は wgpu の queue に出すので、その間は queue を他の thread から使わない（`unsafe fn`）。
  同じ process の Direct3D 12 では、engine の wgpu の device も window の wgpu の device も同じ `ID3D12Device`（adapter ごとの singleton）を包み、共有はそのまま動く（試験で確認）。window の device ができた後に Direct3D 12 の debug layer を有効にすると、その device が削除される。そのため engine の `wgpu::Instance` は `InstanceFlags::DEBUG` と `VALIDATION` を付けずに作る。
- **GPU を好み、CPU へ戻る。** 既定の `gpu` feature では window の作成と同時に worker で GPU を始め、その間と使えないときは CPU で描く。
  GPU の失敗は frame を失わずに CPU で描き、session の終わりまで CPU に落ちることがある。どの renderer で描いたかは observer（`Shell::on_renderer_change`）で知らせる。
- **モバイル・touch・ライフサイクルの語彙は今ある。** 後から足すと上面を壊すので、Windows で使えなくても語彙を持つ：
  環境の snapshot と変化の event、能力の表、surface の世代、software keyboard の要求、App の `Suspended`/`Resumed`、
  surface の `SurfaceDestroyed`/`SurfaceCreated`、
  `MemoryWarning`、一つの pointer の語彙（id・`PointerKind::{Mouse, Touch, Pen}`・`PointerPhase`・圧力・傾き）。
- **pointer は一つの event。** mouse・touch・pen は Pointer Events と同じく `ShellEvent::Pointer(PointerInput)` 一つで届き、wheel だけが別の `ShellEvent::Wheel` である。
  `PointerInput` は id・kind・phase（`Down`・`Move`・`Up`・`Cancel`・`Enter`・`Leave`）・`buttons`（event の後に押されている button の集合）・`button`（`Down`・`Up` で変わった button。他の phase では `None`）・位置・圧力・傾き・click 数を持つ。
  phase は `ui-types` に一つだけあり、wire と core の `PointerEvent` も同じ型を使う。`Enter` は window が送らず、listener に入ったときに engine が作る。
  backend の mouse の message（位置のない button、何も持たない leave）は `arvane_shell::Mouse` が最後の位置と押されている button で補い、どの event も完全な snapshot にする。window の外へ出ても位置と button は残り、外での離しはその位置で届く。
  focus を失ったとき、button が押されていれば mouse の `Cancel` を一つだけ送り（押されていなければ送らない）、接触はすべて `Cancel` で終わらせてから `Focused(false)` を送る。
  click 数は button ごとに押したときの数を覚え、離しはその button の数を運ぶ（chord の中でも、別の button の押下で数が変わらない）。試験の window（`VirtualWindow`）も同じ `Mouse`・`Contacts`・`ClickCounter` を通す。
- **Linux。** Kivact は platform を知らず、Linux の対応はすべて Arvane の中にある（winit が Wayland と X11 を選び、font は fontconfig を dlopen する）。

platform ごとの能力（試験の window は、すべてを注入し記録できる platform として振る舞う。10章）:

| 能力 | Windows（winit 0.30） | Linux（X11・Wayland、実験的、WSLg で E2E） | Android（予定） | iOS（予定） |
|---|---|---|---|---|
| touch | `WM_POINTER*` から（synthetic な注入で確認。実機は未確認） | winit の touch（未確認） | winit の touch（未確認） | winit の touch（未確認） |
| pen | subclass が種類・傾きを読む（注入で確認）。hover・barrel button・eraser は渡さない | touch として届く（能力なし） | touch として届く（能力なし） | touch として届く（能力なし） |
| 接触の取消し | pointer と message の flag（注入では未確認） | winit の `Cancelled` | winit の `Cancelled` | winit の `Cancelled` |
| touch からの mouse の message | 署名で捨てる | — | — | — |
| App の suspend・resume | 起動で一度 resume するだけ | 同じ | `suspended`/`resumed`（surface も作り直す） | `suspended`/`resumed`（surface は残る） |
| memory warning | なし（能力なし） | なし | winit の `memory_warning` | winit の `memory_warning` |
| software keyboard | 何もしない（能力なし） | 何もしない | IME と一緒に出す | IME と一緒に出す |
| safe area | 0（能力なし） | 0 | 0（winit 0.30 は報告しない） | 0（同じ） |
| IME の再変換 | subclass が答える | なし | なし | なし |
| path を開く・file manager で示す | `explorer.exe`（示すは `/select,`） | `xdg-open`（示すは親の folder。未確認） | なし（`Unsupported`） | なし（`Unsupported`） |

macOS の path を開く・示す機能は対応が未規定のため `Unsupported` を返す（対応を仕様に加えるまで分岐を入れない）。

### lifecycle と touch scroll の authority

wire は `LifecycleEvent::{Suspended, Resumed, MemoryWarning}` と `HostEvent::Lifecycle` を持ち、旧 MemoryWarning event は置きません。現在の `surface_active` は `EnvChanged(HostEnv)` だけが運びます。host は実際の suspend/resume の重複を省き、suspension と drawable を更新してから lifecycle を並べます。surface の喪失・復旧や可視性は lifecycle を合成しません。

core の platform に受動的な owner 付き registry を置き、client は event を dispatch します。`on_lifecycle` は setup の登録と cleanup を対にし、登録順の snapshot を registry の借用の外で Callback に渡します。実行は登録 owner の中で untracked、error はその boundary へ送ります。実際に破棄済みの owner を呼ばず、callback が要求した条件分岐・fallback の変更は通常の event batching の後に反映します。callback ごとの再入する stabilize は行いません。

memory warning は先に host の text cache と AssetStore の未使用 entry/variant を解放し、pixel eviction を通常の take_evicted の経路へ残します。Arvane は device texture の別 cache を持たないため、lower の通常の poll が通知を消費します。現在・前 frame の asset と処理中の仕事、登録 font、再読込みに必要な source は保ち、通常の budget は変えません。core に第二の text/image cache はなく、tree/request の余った容量だけを縮め、live handle・environment memo・user asset を捨てません。

単指 touch の調停は engine の pointer track が初期点・直前点・candidate scroller を保持します。8 論理 px を超えたら主な軸の既存 scroll chain を調べ、移動できるときだけ元の press/capture を一度 Cancel してから scroll します。scroll_from と set_scroll が nested chaining・clamp・modal 境界の唯一の authority です。二本目の touch は調停を終えます。native takeover で Cancel 済みの接触には Down を合成せず、独自の gesture の再開は新しい接触から行います。対象の無効化・削除・modal 切替の検査が engine を借りられないときは Cancel を保留し、commit 境界または次の pointer input より前に通常の cancellation 経路で届けます。mouse と pen の規則は変えません。

### 埋め込み

DAW プラグインの editor（CLAP・VST3）は戦略の主対象。Windows と Linux X11 は実験的に動作し、正式対応には主要な DAW と両 OS の G-03（IME、accessibility）の実機確認を求める。macOS（NSView）は設計対象で、実機と保守者がそろったときに実装する。UI 作者は desktop と同じ画面を書き、Rust の wrapper 作者が規格の glue と入口を担当する。利用手順の正本は[埋め込みガイド](../for-kivact-user/how-to-guide/embed.md)。

#### 全 OS の公開契約

公開契約（thread、開く、setter、値の検査、単位、サイズの交渉、drop、失敗、unload の9項目）の正本は、ガイドの[全 OS で同じ契約](../for-kivact-user/how-to-guide/embed.md#全-os-で同じ契約)。ここでは書き写さず、以下でそれを実現する内部の流れを書く。backend の値の上限は Win32 が `i32::MAX`、X11 が `u16::MAX`。

#### 層と失敗の経路

依存は `kivact::embed` → `kivact-core::embed` → `kivact-engine::embed` → `arvane::embed` → `arvane-native::embed` → `arvane-shell`・`ui-types`。`ParentWindow` は wire が再公開し、core に native library や規格の型を入れない。`App::run` と `App::embed` は client の組立てとサイズ検査を共有し、engine の `Host` と Arvane の `Driver` は駆動者にかかわらず同じ経路で作る。waker は破棄後に App を生存させず、生存状態を見て無害に戻る。

`Runtime::first_unhandled_panic` は default sink に届いた最初の `ErrorKind::Panic` を ring と独立に保持し O(1) で返す。owner 境界で回復した panic は入らない。埋め込みの client だけが start と turn の終わりで一度確認し、元の `UiError` を共有状態に保存する。start は `Option<String>`、turn は `TurnOutput::stop` で診断を返し、`Host::begin`・`handle` が `ShellError` にして backend を止める。同じ turn の残りの phase は終える。desktop の stop は常に None で、診断・回復の挙動を変えない。

#### 中核と native backend

`embed` は window を所有するホストの GUI thread から開く。root は UI thread で一度だけ作る。Windows はホストの loop、Linux は独立した XCB 接続・eventfd・timerfd の thread を使う。外部からの drop は Close を送り、所有する thread ごとに最大2秒待ち、完了なら join する。上限超過では module を保持して戻り、未完了の後始末は後で続き得る。自身の UI callback 内の drop は自己 join をせず callback 後の外側 loop へ終了と後始末を委ねる。ホスト process の DPI awareness・COM apartment・signal・locale・X error handler は変えない。規格や枠組みの wrapper はアプリが持つ。

Windows の埋め込みの file/folder dialog は rfd 0.17.2 の AsyncFileDialog を使う。同期 FileDialog は呼出 thread で CoInitializeEx(STA) を実行するため、埋め込みでは使わない。async は library が持つ worker で COM を初期化し、UI thread は future を poll しながら、同じ thread で開いた modal dialog と同じく thread の message を取り出して配送する（ホストの画面と timer は動き続ける。ホストの top-level は dialog が無効にする。State.busy が editor 自身の event を queue に積み、App callback の再入を抑止する。WM_QUIT は投げ直してホストの loop へ残す）。worker の epilogue は future の完了後にも走り得るため、初回 poll 前に module を保持する。title・directory・file name・filter と親 HWND は同期経路と同じ公開 builder の値を渡す。OS dialog は自動試験で開かず、実 dialog と STA/MTA のホストでの確認は未実施の手動確認である。

埋め込みの中核は `arvane-native/src/embed/child.rs` と `session.rs` に置く。`Shared` は command の queue、適用済みの物理サイズと倍率、ホストのサイズ指定の generation、最初の失敗、生存状態を持つ。`Handle<B: Backend>` はホストの setter を検査して queue へ積み、backend の `wake` で UI thread に知らせる。setter は App の callback を同期に呼ばない。`Session` は handler と `ShellControl` を持ち、start・event の panic と Err、App の exit を最初の失敗へまとめ、以後の event を止める。次の安全な点で handler.close、handler の drop、native の資源、子 window の順に片付ける。破棄中の panic は残る資源を漏らして警告する。

backend は native identity、window の作成・破棄、event の変換、timer、command の適用と `ShellWindow` の OS の窓口を持つ。Windows の HWND・UI thread id・wake の重複防止は共通状態から分け、再入用の busy と event queue、`RefCell<Session>` は UI thread に閉じる。X11 の eventfd と join handle も backend が持ち、保持された async waker は弱参照だけを持つ。外部 drop の X11 join は 2 秒を上限とし、自身の callback 内では Close を queue に積んで戻る。

公開契約では、callback 外の drop は通常は App と native window の後始末を待つが、App が起動した各 thread は最大2秒とする。上限後の未完了cleanupは module を保持したまま続き得る。起動失敗で `Err` を返す場合も同じ待機上限で、root の元の panic 診断を返すことと、上限後も後始末が続き得ることを区別する。callback 内の drop は実行中の turn を終えてから後始末を始める。core の `Embedded::drop` が立てる flag で AppClient は同じ batch に残る event を配送せず、engine の closing flag が以後の turn を止める。turn の残りの effect や task は止めない（reactive と task に停止の検査を持ち込まない）。無条件の破棄完了保証と有限の待機は両立しないため、R9の待機上限と安全なmodule保持を優先する。

サイズの交渉は共通の `negotiate` が論理サイズを物理 px にし、generation を読んで callback を呼び、受理されたサイズを generation 付き command として積む。適用前にホストの setter が generation を進めたら、その交渉結果を捨てる。開いている途中の要求は engine が最初の event まで保留する。実サイズの変更は WM_SIZE と ConfigureNotify で確認し、変わったときだけ Resized を届ける。倍率の変更も layout の更新を知らせる。

| 子 window の能力 | Windows | Linux X11 |
|---|---|---|
| CPU/GPU・mouse・wheel・key | あり | あり（xkbcommon） |
| IME・accessibility | Imm32・UIA | XIM・AT-SPI |
| clipboard・file/folder dialog | あり | arboard・rfd |
| open・reveal | あり | あり |
| file drop | drop のみ | XDND v5、drop のみ |
| touch・pen・fullscreen | なし・なし・Unsupported | 同左 |
| frame wake | SetTimer（通常約64 Hz） | timerfd |

Windows の埋め込みの CPU present は softbuffer を使わず、State が最後の BGRA の Vec<u32> と寸法を保持する。GetDCEx の DCX_CACHE | DCX_CLIPSIBLINGS で借りた DC は z-order で上にある可視の兄弟を除いた領域だけを持つので、そのまま SetDIBitsToDevice で提示して ReleaseDC する。STATIC の CS_PARENTDC に依存する GetDC は使わない。WM_PAINT は BeginPaint/EndPaint で無効領域を確定して同じ経路で最後の buffer を再提示する。desktop の softbuffer は変更しない。CPU と実際の GPU frame の双方を、重なる兄弟を含むホスト client 領域の画面 capture の E2E で検証する。実行結果は改修の進捗記録に残す。

Windows の WM_DESTROY は UiaReturnRawElementProvider(hwnd, 0, 0, null) でこの子の provider を切り離してから adapter を解放する。明示的な close で先に dispose する経路も adapter の解放前に同じ切り離しを行う。accesskit_windows 0.33.1 の adapter は WM_GETOBJECT 応答で UiaReturnRawElementProvider を使うが、drop で UiaDisconnectProvider を呼ばない。UIA client が provider を保持し続けても unload 後に code が消えないよう、初回の木の activation の前に retain_module を呼ぶ。ホスト全体へ作用する UiaDisconnectAllProviders は使わない。
Linux の native service は App の UI thread に閉じる。`x11/services.rs` は desktop と同じ Services を使い、rfd に editor の XCB child raw handle を渡す。rfd 内部の zenity fallback は親指定を持たない既知の制限がある。arboard と rfd の初回利用前に module を保持する。`x11/a11y.rs` は lock と同じ accesskit_unix 0.22.1 の Adapter を使い、構築前に module を保持する。別 thread の activation/action/deactivation は PlatformTree と waker だけに触れ、App の callback は UI thread で配送する。child を root 座標に変換した物理 bounds は publish 時に更新する。

`x11/ime.rs` は xim 0.5.0 の x11rb-client と x11rb-xcb feature を使い、App の Arc<XCBConnection> と event queue を共有する。xim 0.6.0 は x11rb 0.14 を要求するため、0.13 と組める最新の 0.5.0 を選ぶ。hidden XIM window の selection/client message も先に filter し、server に返された key だけ XKB に戻す。preedit の部分編集は文字単位から shell の UTF-8 offset に変換する。`x11/dnd.rs` は XDND の URI selection を同じ接続で受け、local file URI を OS path に戻して Drop だけ届ける。


close は action を drain せず、App、Presenter、子 window の順で破棄する。Windows は system STATIC の子を subclass し、WM_NCDESTROY で元の wndproc に戻す。custom class は登録しない。初めは非表示で作り、UIA bridge と handler/start の初期化後に表示する。親が先に消えた場合も停止する。native 境界を越える panic は捕捉し、以後の handler を止める。owner の error boundary が回復しなかった Panic 診断は start と turn の終わりで回収し、元の UiError を保存して停止させる。root の未処理 panic は子の後始末を開始して App::embed が元診断を返し、起動失敗にも所有 thread ごとの待機上限と上限後の module 保持下での後始末継続を適用する。起動後は Embedded::error が元診断を優先して返す。破棄中の panic は残る資源を漏らして二重 panic を避け、警告する。GPU の結果は UI thread で破棄し、Windows の worker 待機は送信 message を配送する。

#### backend の完成条件

`embed/child.rs` は `Shared`、`Command::{SetSize { width, height, generation: Option<u64> }, SetScale(f32)}`、`Handle<B>` と `Backend` を持つ。`Shared` のサイズは AtomicU64、倍率は AtomicU32、generation は AtomicU64、失敗は Mutex と lock なしで読む AtomicBool、alive は AtomicBool、queue は mutex で保護する。queue は generation のないホストのサイズと倍率を最新の一つにまとめる。close は Command を使わず、stopping の flag と `wake`／同期の通知で UI thread に伝える。`Backend: Send + Sync + 'static` の契約は次のとおり。値の検査、失敗、交渉、破棄の共通処理を各 OS に複製しない。

| backend の口 | 必ず守ること |
|---|---|
| `MAX_DIMENSION` | native の物理寸法の上限を示す |
| `wake(&self)` | 近いうちに UI thread で queue を処理させる。呼出し元の stack で App を走らせない |
| `close(&self, shared)` | `request_close` の後に一度呼ばれる。UI thread 以外からは後始末の完了を最大2秒待ち（Windows は SendMessageTimeoutW、X11 は thread の終了）、上限後は thread を detach して module を保持して戻る。UI thread の callback 内では待たずに戻り、外側の loop が後始末する |
| native event・command | 変更後の実寸法を確認し、変更があったときだけ Resized を届ける |
| `Session`・`negotiate` | start/event の catch_unwind、最初の失敗、close の順序、generation による競合解決を共通実装に任せる |

`Session::close` は handler.close → handler の drop → native 資源（surface、adapter、timer、接続）→ native 子 window の順。`ShellControl::exit` も停止理由にする。macOS の完成条件はこの backend 契約の実装と、下記の共通12契約試験の通過。macOS の分岐は実機と保守者がそろうまで追加しない。

#### winit を埋め込みに使わない根拠

winit 0.30.13 の調査済みの制約により、desktop は winit を使い、埋め込みの子 window は native backend が所有する。ホスト process の DPI awareness、COM apartment、OleInitialize、timeBeginPeriod、signal、setlocale、X error handler、XInitThreads、process 全体の hook を変更しない。自分の module だけを保持する操作はこの禁止に含めない。

| 確認した実装・関数 | ホストとの共存を妨げる事実 |
|---|---|
| `EventLoopBuilder::build`、`EVENT_LOOP_CREATED` | static AtomicBool が一度 true になると戻らず、同じ process の二つ目は RecreationAttempt |
| Windows `pump_app_events` → `pump_events` → `dispatch_peeked_messages` | `PeekMessageW(hwnd = 0, PM_REMOVE)` でホストの window/thread の message まで取り出し、ホストの accelerator/dialog 処理を経ず配送する |
| Windows `EventLoop::new` → `become_dpi_aware` | process 全体の DPI awareness を既定で変更する |
| Windows `InitData::create_window_data` の `OleInitialize` | ホストが COM MTA の場合は初期化が失敗して panic する |
| Windows `register_window_class`・`create_event_target_window` の `RegisterClassExW` | 固定名 `Window Class`・`Winit Thread Event Target` を登録し、`UnregisterClass` を呼ばない |
| Windows `register_all_mice_and_keyboards_for_raw_input`、X11 `XConnection::new`・`EventLoop::new` | RAWINPUT の登録、XInitThreads・XSetErrorHandler・setlocale も process 全体に作用する |

baseview は git 配布の依存、IME の不足、Kivact と別の event 語彙を持つため採用しない。CLAP の `posix-fd-support`・timer と VST3 の `IRunLoop` は wrapper から得られる保証がないので、X11 は独立した XCB 接続と専用 UI thread を使う。Win32 は別 thread の子がホストの入力 queue を結合するのを避け、ホスト GUI thread の system STATIC を subclass する。winit の固定 class を複製せず、WM_NCDESTROY で元の wndproc を戻す。

外部事実の調査時の revision は CLAP `a47f6bad`（1.2.10）、VST3 pluginterfaces `4f547e8e`（3.8.1）、nih-plug `de421011`、clack `27ca2839`、baseview `4a274e35` / nih-plug 固定 `9a0b42c`。

#### 契約試験と手動確認

`kivact-public-api-tests` の `embed` は実親 window の描画・入力・倍率・resize・開閉を検証する。 `e2e/embed.rs` を入口とし、`embed/support.rs` が待機と描画の共通 fixture、`embed/contract.rs` が OS 条件を持たない12件の契約試験、`embed/windows.rs` と `embed/x11.rs` が native protocol に固有の試験を持つ。契約は開く・入力・root の一度の実行、入力値の検査、panic の元診断と owner 回復、非同期 setter と callback の非再入、サイズ交渉、通常の後始末待機／callback 内の drop、親の先行破棄、失敗後の資源の解放、同じ親の複数 editor、wrapper の gesture、clipboard の往復を検証する。callback 内の drop は実行中の turn を終えてから後始末し、次の入力を配送しないこと、panic の turn はその残りの phase を完了することを別々に確認する。process の thread・FD・module 保持の計測は `tests/embed_resources*` の専用 binary に残す。TestParent の parent_in は物理座標の試験用 container を作り、外側の親まで所有する。`kivact-embed-host` は公開 API だけで IME・Narrator の手動確認を行う。Linux の実行範囲は WSLg X11。DAW の正式対応に必要な確認は所有者の wrapper で行い、人の未実施項目を自動 E2E の結果で置き換えない。

| OS 固有の試験 | platform | 理由 |
|---|---|---|
| `dev/tests/kivact-public-api/tests/e2e/embed/windows.rs`（feature `embed`） | Windows | HWND の hit test・親の GUI thread・UIA・GDI/DXGI の兄弟 clip・capture・system key を直接検証する |
| `dev/tests/kivact-public-api/tests/e2e/embed/x11.rs`（feature `embed`） | Linux X11 | XIM の任意接続・XDND・AT-SPI は X11 の native protocol。AT-SPI client は bus と python3-pyatspi を要するため理由付き ignore とし、手動確認一覧に実行手順を置く |

共通契約は次の12件で、各試験関数に OS の cfg を置かない。macOS を追加しても同じ関数を実行する。

1. 開くと click・key・text が届き、root が一度だけ走る。
2. null の親を Platform、不正な倍率を Message で拒否する。
3. root の panic は元の診断を返し、owner 境界の回復は続行する。
4. 非同期 setter と getter の適用済み値、別 thread からの setter、callback の非再入。
5. 不正な setter 値を無視し App は続行する。
6. サイズ交渉の受理・拒否・callback なし・ホストの訂正・起動中の要求。
7. callback 外で通常の後始末を待つ drop と、press/cleanup 内で延期する drop。所有する thread の待機上限超過と module の保持は資源試験で分けて検証する。
8. 親が先に破棄されても安全に止まり、error は None。
9. 起動後の panic で資源を解放し、error に元の診断を残す。
10. 同じ親の二つの editor が独立して動く。
11. wrapper の Begin → binding の書込み → Commit/Cancel の順序。
12. clipboard を書いて読み戻す。

`TestParent` は公開 facade の feature `test` と `embed` で使う実親 window。`open`・`parent`・`parent_in`・`pump`・入力・`capture` から試験し、native の型を利用者に要求しない。資源比較は warm-up 後、名前が kivact-/arvane- の thread、GDI/USER または FD を専用 binary で数える。GPU driver の thread と既定 IME window は Kivact の thread と分ける。GPU configure の待機では送信 message を処理し、遅れて届く surface は worker で破棄しない。

#### 対応範囲と module の保持

Windows・Linux X11 は実験対応で、macOS・Wayland native・touch/pen は未対応。Linux の XIM は server がなければ XKB の通常入力を使い、reconversion は未対応。Windows と Linux の file drop は hover/cancel を持たない。未処理キーの返却と DAW のキー横取りの hook は提供しない。title はホストが管理し、fullscreen は Unsupported。最小サイズは plugin の request_window_size を丸めるが、host の set_size は wrapper の制約に従う。

Linux の on_edit 等は専用 thread で走る。VST3 gesture の UI thread 条件は wrapper が配送して守る。Cancel は元の binding 値の書戻し後なので wrapper は end だけを送る。永続 parameter は App の外に持つ。

worker ごとの終了待機は最大2秒。長い blocking・decode・GPU 初期化や X11 UI thread は、上限を超えると detach を記録し、埋め込みを閉じると code を含む module を process 終了まで保持する。Windows は関数 address から GetModuleHandleExW の FROM_ADDRESS | PIN、Linux は dladdr の path から dlopen の NOW | NOLOAD | NODELETE を使う。実行ファイルへの静的 link は元から process 終了まで mapped されるため、dlopen 成功とは区別して診断する。他 OS は何もしない。保持は冪等で、保持した事実・理由と caller を tracing（target kivact）の警告に一度だけ残す。

process に残る thread や provider を持つ library は初回利用前に保持する。Linux の arboard と accesskit_unix、残留 thread を作る file dialog、Windows の UIA provider が対象。ホストは editor を閉じた後に plugin を unload してよい。module の code は必要に応じて process 終了まで残る。アプリが直接作る thread の停止と join は引き続きアプリが管理する。残る見送りと再検討条件は[台帳](strategy/ledger.md)を参照する。

thread の join と detach の一覧は[4章の OS thread の起動と終了](#os-thread-の起動と終了)が正本。detach の static 記録は App の寿命をまたいで保持する。`Embedded::drop` の host 破棄後の検査に加え、起動失敗や callback 内の drop のため `AppClient::drop` は task shutdown 後に検査する。Arvane `Driver` の最後の field の completion guard は handler（asset store を含む）と presenter の field の drop が済んでから下位の detach を検査する。guard の定義はそれらを知らない shutdown module に置き、依存を下向きに保つ。

Linux の accesskit_unix 0.22.1 は process に残る executor と D-Bus 接続を持つため、adapter を作る通常起動でも module を保持する。arboard の serve thread と、rfd の結果通知後にも終了処理が残り得る zenity worker も利用前に保守的に保持する。rfd の同期 portal 経路だけでは worker 残留を意味しない。この通常起動での保持と、Kivact の thread の待機上限超過による保持を、資源計測では分けて記録する。XIM server がない場合は失敗とせず XKB に fallback する。server の消失・500 ms を超える応答遅延・不正な応答・保留キーの 1024 件超過も App を止めず、診断を出して XIM を無効にし、未配送のキーを順序どおり XKB の通常入力へ戻す。

## 9. 画像・SVG・フォント

`Asset` は `Path`・`Static`・`Shared` で、内容ではなく同一性で比べる。形式は内容で判定する（PNG・JPEG・GIF・WebP・SVG・SVGZ。GIF と WebP は最初の frame だけ）。
読込み・decode・raster 化は `arvane-asset` の store の worker thread で行い、終わると host が渡した waker が turn を起こす。
SVG は描く device pixel の寸法で raster にし直す。
`Asset` は ui-types の単一定義を core・wire・arvane-asset が再公開する。node と scene は同じ値を store に渡し、画像 source の写しや変換を持たない。
`TextProps::family` と `line_height` は祖先から継ぎ（root は theme の値）、`Writer::register_font` は書いたときに同期で処理する。
読めない画像は何も描かず、見つからない family は system UI の font で描き、それぞれ1回だけ `AssetFailed` を送る。
client は作成位置を付けた `ErrorKind::Asset` として既定の受け口だけに送り、`error_boundary` には届けない（画像1枚の失敗で部分木を置き換えないため）。
scene の画像（`Draw::image`）の失敗は報告しない。詳細は `arvane-asset` と [kivact-lower の assets](../../crates/kivact-lower/src/assets.rs)、[engine の資源完了処理](../../crates/kivact-engine/src/model/interact/sync.rs)、[kivact-layout の image](../../crates/kivact-layout/src/image.rs) にある。

## 10. 試験

### 方針

試験は E2E を優先する。Playwright のように、本物の layout・shaping・描画・pixel・accessibility を、実 window の上で仮想時計で確かめる。
mock host・host の適合 suite・内部 module の単体試験は作らない。
`kivact::test` の `Screen` は App を engine の試験 driver（本番の engine の `Host::begin`/`Host::handle`）の上で動かし、
入力は shell の入口に `ShellEvent` として注入する（OS の入力は使わない）。
試験の window は Arvane の `test_display` が、process に1本の UI thread と唯一の winit の event loop と window の pool で持つ。
そのため、試験の window を使う試験 binary は `App::run`・`Shell::new` を呼べない。
query・入力・settle・screenshot・option・失敗の証跡の細則は `kivact::test` の rustdoc、driver は engine の `testing`、
試験の window は `arvane::test_display` の rustdoc にある。

### 共通の描画 setup

`test::render_with_handle(options, setup)` は、root の setup が作った handle と `Screen` を返す公開 API で、handle の受け渡しと caller の失敗位置を core に集約する。kit の試験は generic な mount wrapper を持たず、theme と reduced motion を設定した options を明示して直接呼ぶ。domain 専用 fixture はこの API に委譲する。
`RenderOptions::from_env()` は `Default` の設定を保ち、`KIVACT_TEST_GPU` が正確に `1` のときだけ GPU、`KIVACT_TEST_HEADLESS` が正確に `1` のときだけ Headless、それ以外は Real を選ぶ。headless は開発者の手元の opt-in に限り、試験・CI・検証 script では設定しない。
`Screen::assert_layout_with(&LayoutCheck)` は既存の許容値や ignore 対象を保ち、通常の `assert_layout()` と同じ診断・失敗の証跡を残す。

4 kit × Light/Dark の標準部品の適合は `theme_conformance` の実 window 試験で検査する。role/name・選択・panel の関係・layout と、実 theme の focus token を使った Tab focus の外周 pixel を確かめる。標準の select/radio/tabs の振る舞いは public-api suite に置き、kit の試験には固有の variant・content・geometry・domain の回帰を残す。

### window-lock

window を開く自動試験（`kivact::test`、公開 API の e2e、kit の consumer と catalog、window を開く doctest）は、
すべての worktree で同時に1つの process だけが window を開くように、
`dev/tools/window-lock.sh -- <command>` を通して実行する（同時に開くと focus を奪い合い、試験が止まることがある）。
lock は時間の上限（既定 900 秒）も掛け、時間切れは終了コード 124 になる。

### 診断

失敗は query・全候補・focus・直近の操作と turn・error・公開された accessibility の木・core の semantics の木を1つの message にまとめ、
最後の frame と木を `<CARGO_TARGET_DIR または target>/kivact-test-failures/<試験名>.{png,txt}` に残す。
debug build では settle ごとに、core の host node が engine の木と同じ構造かを確かめる。さらに layout（rect・baseline・overflow）、paint の一覧（順序・ink・child index）、編集の行（chunk の分割・結合・文字数）、受理済みの accessibility の木を、全部作り直した結果と比べる。
accessibility の reference は A11yState metadata と各 mutable EditLines を独立に clone して全構築し、unwind 時も original を復元する。reference は production の ID/cache/fingerprint を補修せず、構築 counter にも加えない。shell の受理木と payload・children の読取り順・focus・relation・run ID を比較する。
Screen::stats の test feature 限定 a11y_constructed / a11y_published は u64 の累積補助 work 指標で、client Writes には含めない。初回 full 構築、全 turn の候補生成、拒否を含む window 送信を数え、独立 debug reference だけを除く。warmup 後の差を取り、後続 no-op のゼロで先の全構築を隠さない。

追加の自己検査は既定で有効で、`KIVACT_SELF_CHECK=0` で切れる。不一致は node と retained/reference の値を通常の Screen の失敗の証跡に出す。release には入らない。
`Screen::stats` の host 生成数と prop 書込み数は本体の変更のための内部性能診断で、公開 API の利用者には状態・表示・role/name を確かめる試験を案内する。
統計の大小だけで画面の正しさ・OS 入力・present・性能目標の達成は証明できない。

### 何を試験するか

公開 API（`kivact::test`）の E2E の suite は `dev/tests/kivact-public-api/tests/e2e/`・各公式 kit の `tests/`・`examples/*/tests/`・`dev/apps/*/tests/`・`templates/basic`。
`dev/tests/kivact-public-api/tests/e2e/` は1つの試験 binary（kivact・Arvane・wgpu を1回だけ link し、試験の window は1つの event loop を共有する）で、
確かめる振る舞い（runtime・focus・overlay・pointer・text 編集 ..）で module を分ける。
共有の `mount`・probe の fixture・random の driver は `support`。
`probe_matrix`・`probe_free_functions` は handle と自由関数をさまざまな文脈で呼んだ結果を golden の表で固定する。
Arvane の上面の契約は `crates/arvane/tests/e2e/`（Kivact の crate を使わず facade だけを試験の window の上で使う。module の分け方と決定性はその `main.rs` の doc）。
ほかに残すのは、trybuild のコンパイル試験（`dev/tests/kivact-public-api/tests/compile.rs`。
window を link しない別の小さな binary）、docs の doctest と公開 item の rustdoc の例、
release の性能の試験（`--ignored`）、公開 API では作れない `Screen` の試験（`crates/kivact-core/src/test/screen/*_tests.rs`）と xtask の試験。

- **host の契約の確かめ方。** 6章の契約と9章の資源の規則は、engine の本番の経路を通る E2E で確かめる。OS の入力・IME・UI Automation・clipboard を通す確認は GUI harness が受け持つ。
- **GPU の描画。** `crates/kivact-core/src/test/screen/gpu_tests.rs`（feature `gpu`）が、
  代表的な画面を実 window の GPU surface へ提示した frame を CPU の frame と許容差
  （channel の差の平均 1 以下・64 を超える pixel 0.5% 以下・8 以内が 99% 以上）で比べる。hardware の adapter がない環境でだけ、理由を書いて skip する。
- **E2E で確かめないもの。** 内部の仕事量（layout・shaping を飛ばしたか、cache 数）、GPU/CPU の画素単位の一致、renderer golden、AccessKit adapter の内部、
  OS の clipboard/file dialog。AccessKit の変換の内部は GUI harness の UI Automation で確かめ、platform node の取出し口を E2E のために公開しない。
  core が生成しない不正な書込みは engine の局所的な `debug_assert!` で検査する。必要になったら観測できる結果（pixel・accessibility の木・`stats()`）で E2E を足す。
- **性能。** e2e の `perf`・`scale` は release で `--ignored` を使い、1 text の変更は node 数によらず書込み1という契約を確かめる。
- **実 window の GUI harness（`dev/tools/gui-harness/`、Windows）。** SendInput で操作し UI Automation で状態を読む。
  6 つのアプリと workbench のアイコンの場面を1周として走らせ、
  証跡を `target/gui-evidence/` に残す。ユーザーの許可を得てから実行する。手順は [GUI harness](tools/gui-harness.md)、
  対象は [手動確認一覧](manual-verification.md)。
- **docs。** `cargo test -p kivact-docs --doc` が README と `docs/` の rust の code block をコンパイルし、走らせる。

### platform ごとの試験

Windows と Linux（WSLg の Wayland/X11、`dev/tools/linux/test-wslg.sh`、[Linux の検証](tools/linux-verification.md)）で同じ試験を通す。
Linux では frame は CPU・文字は同梱の Tuffy を使う。hardware adapter のない環境だけ理由付きで GPU 試験を skip する。
platform で限る試験は次の表に載せ、doc comment に理由を書く。`#[ignore]` で platform 差を黙って外さない。

| 試験 | 実行する platform | 理由 |
|---|---|---|
| `dev/tests/kivact-public-api/tests/e2e/assets.rs` の `emoji_draw_in_color_from_the_system_emoji_font` | Windows | system font の color emoji の font が要る。Linux の distribution にはないことがある（WSL の Ubuntu にはない） |
| `dev/apps/music-player` の試験 | Windows | 音声の依存（alsa-sys）が build に `libasound2-dev` を要る。WSLg の検査では package を入れられない |
| `crates/arvane/tests/e2e/external.rs`（feature `gpu-interop`） | Windows・Linux（`_on_dx12` は Windows） | GPU texture 共有は Windows と Linux だけが対象で、Direct3D 12 は Windows だけにある。取り込める hardware の adapter がない環境（WSLg の lavapipe など）では理由を書いて skip する |
| `dev/tests/kivact-public-api/tests/e2e/gpu_interop.rs`（feature `gpu-interop`） | Windows・Linux（`_on_dx12` は Windows） | 同上。CPU の window の試験は GPU を要らない |

### host の性能（hostbench）

engine と Arvane を同じ場面で測る bench です。`dev/benchmarks/hostbench` は公開 API だけで書いた1つの binary で、
host `arvane`（既定の GPU）と `arvane-cpu`（`ARVANE_RENDERER=cpu`）を測る。
場面・frame の時刻（`KIVACT_FRAME_LOG`）・runner（`dev/tools/gui-harness/hostbench.ps1`）の細則は hostbench の rustdoc にある。
pointer の drag と hover（`drag_backdrop`・`drag_slider`・`hover_tooltip`）は、公開 API が pointer の入力を送れないので、move が部品に起こす書込みを frame ごとに作って測る（hit test と drag の経路は含まない）。
runner は GUI の lock を持ち、ユーザーの許可を得て実行する。

```text
powershell -NoProfile -ExecutionPolicy Bypass -File dev\tools\gui-harness\hostbench.ps1 [-Runs 3] [-Hosts arvane,arvane-cpu] [-Scenarios text,idle] [-NoBuild]
```

性能比較で守ること:

- 基準 main の commit、候補の commit（未 commit なら source の記録）、Rust toolchain・release profile・exe の SHA256 を保存する。
- 同じ machine・renderer・viewport・前景・accessibility 接続条件で build と測定を分け、GPU/CPU ごとに3回以上の raw 値・中央値・ばらつきを残す。
- relayout の turn/layout/scene/raster/present、process CPU、private/working set、frame の遅延を別々に扱う。
  hostbench の update-to-frame と redraw の仕事は別の指標。
- 初回と追加の測定は別 batch で保持し、都合のよい結果で上書きしない。範囲の重なりだけで同等性を証明せず、測定差から変更固有の因果や普遍的な速度向上を断定しない。
- accessibility off の測定は AT 利用時の性能を測らない（UI Automation の client が動いていると host は木を保ったまま測る）。
- 生データ・exe の hash・実行条件は ignored の `target/` に保持し、commit しない。

## 11. 不変条件と既知の制限

### 守ること

1. setup は一度だけ。変化は signal → memo → 描画 effect → prop の経路だけで伝わる。root の再 compile も全 frame の送信もしない。
2. 出力を比べて差分だけを engine の木へ書く（同値の書込みで engine への書込み 0）。
3. engine は書込みを信頼して適用し（debug build でだけ確かめる）、物理的な失敗だけを報告する。
4. 登録には解除を対にする。
5. NodeId と generation は単調増加で再利用しない。
6. host からの event は単一の順序付きの列。
7. 標準部品（`kivact::ui`）と kit の部品と外部部品は同じ入口（`node()`、新しい基本部品は `primitive()`）と同じ公開 API で作る。
   `kivact::ui` は内部 module を使わず、`control`・`interaction`・`style`・`view` の公開 item だけを使う。
8. node の prop と scene の置き場は engine の木だけ。core は構造と owner・handler を持ち、effect は比較つき Writer へ直接書く。今の値は Reader で engine の木から読む。
9. focus・modal・pointer、native 編集の状態と制約、semantics の意味は engine の `model` に置く。
   標準の key/paste の規則は公開の behavior（`TextEditing::behavior`）を介して
   interaction の `text_edit_policy` が定義する。window・描画・shaping（`host`・`paint`・`a11y`）は model の状態を写し、別の authority を持たない。
10. backend の実装型を library の公開 API に出さない（taffy は `kivact-layout` に、winit・vello_hybrid・wgpu などはそれぞれの crate に閉じる）。
    共有の geometry・semantics などの値は ui-types の単一定義をそのまま公開する。
11. platform callback の中の panic は process を止めうるので、host の境界で必ず捕捉する。

### 既知の制限・未解決

誤用の報告は unwind（`panic_any` と `catch_unwind`）を前提とする。`panic = "abort"` の build では、誤用で process が終了する。

module に閉じる制限の詳細（vello_hybrid の glyph atlas、mask と合成の扱い、影と pattern の近似など）は、その module の rustdoc の Limits に書く。

- **API。** `on_press` と `on_a11y` は引数なし。描画領域を個別に操作する外部部品は `on_a11y_event` または `on_pointer` を使う。
  新しい基本部品（`kivact::primitive`）は実験的で、編集の子として独自描画や子 node を混ぜること、独自の hit 判定、ネイティブ widget の埋め込み、子を持ち自分で並べる基本部品は未対応。
  AT による範囲選択要求、混在 bidi の run 分割と語境界 metadata、表の行・列数の ARIA property の公開口は未対応。
- **要素の paint。** 背景の層の位置の基準は border box（CSS の既定は padding box）。枠の style は全辺に1つ。背景の画像の読込みの失敗は報告しない（`image(..)` の node だけ）。
  hover・pressed・focus などの状態 style の枠の幅は描画だけを変え、layout を動かさない（状態 style は engine の paint の差分）。
  hit の形は node の基本の style のもの。辺ごとに色の違う枠の角の境目は直線（CSS の曲線と少し違う）。
- **変形と重なり。** 2D の affine だけ（3D はない）。z-index はすべての container が stacking context（CSS の `isolation: isolate`）の扱いで、
  子を親の外の node の上へ出せない（overlay にする）。
  変形の補間は正準形の部分ごとで、順に外れる列は ±180° の中で回る。
- **配置。** grid の名前付きの線・`grid-template-areas`・subgrid・`order`・CSS の block と float はない。
  absolute の箱は親の箱に対して置く（CSS は最も近い positioned の祖先）。
  内容の大きさ（`Length::MinContent` ..）は幅・高さ・`basis`・track だけで、
  縦の flex の子の幅に書いた MinContent・FitContent の高さは折り返さない内容の高さになる（taffy の制約）。
  calc は `pct ± px` だけ。media query の文法と `vw`・`vh` はない（`platform::window_size()` を読む）。
  sticky の scrollport は最も近い scroll する祖先1つで、箱と scrollport の間の変形は見ない。
  fixed の箱は祖先の変形・不透明度・filter を受けず、overlay の外の fixed の箱はすべての overlay の下に描く。
- **文字。** span の decoration と features は段落のものを置き換える（CSS は祖先の decoration を子にも描く）。帯は span の glyph の advance だけ。
  decoration・`text_overflow`・`line_clamp` も他の文字の property と同じく継承する（CSS は継承しない）。
  `provide_theme` は node を作らないので、その下の文字の色は祖先から継ぐ。
  部分木の最上位の node に `color(ColorToken::Text)` を書く。
  省略記号は行の右で切る（RTL の行も）。
  `text_transform` は Text だけ。`::selection` の文字の色はない。inline の箱を段落に入れられない。
  `text-decoration-style`・`text-decoration-skip-ink`・`font-variant` の `tnum` 以外の値・`font-variation-settings` の任意の軸はない。
  文字は grayscale の antialias で、ClearType は使わない（透明な layer と変形で壊れるため）。
- **試験 API。** pointer の id・`clear`・scroll してから押す（scroll into view）がない。window の外の要素の click は失敗する。
- **touch と pen。** multi-touch の gesture は `zoom_pan` の二本指の pinch と pan だけで、二本指の scroll はなく、
  単指 touch は 8 論理 px を超えて native scroll に切り替わる。慣性と overscroll はない。
  pen の hover・barrel button・eraser と傾きは Kivact に渡さない。Windows の touch と pen は synthetic な注入で確かめただけで、実機と取り消された接触では確かめていない。
  Linux・Android・iOS の touch・ライフサイクル・software keyboard・memory warning は winit の仕様からの実装で、実機では確かめていない。
- **モバイル。** safe area は winit 0.30 が報告しないので常に 0。software keyboard が覆う領域も報告しない。
  memory warning は host の text と未使用 asset cache を解放し、App は `on_lifecycle` で応答できる。OS による process の終了後の状態の復元はない。
- **性能。** taffy は dirty な葉の祖先の container の子をすべて見直すので、layout が走ると一覧の長さに比例する（内容だけの変化は `measures_same` で飛ばす）。
  accessibility は実 geometry の変化と dirty 部分木だけを構築する。行の高さが変わり後続の全行が移動する場面では、必要な構築量も全 node に近付く。
  `virtual_list` の計測した行高と `for_each` の1件編集は O(N) の走査がある。
  GPU の経路で window の大きさを変えると frame ごとに swapchain を作り直す（driver の仕事が下限。理由は `arvane-render` の gpu の rustdoc）。
- **描画の近似。** GPU は device の texture の上限を超える画像を縮めて描く。backdrop は別の backdrop が描いたものを見ず、色の行列は replay する各 command の色に掛ける。backdrop が見るのは frame の中だけで、blur が frame の外を読む所では frame の中身を縁で鏡映して replay し直す（外の透明を縁に混ぜて黒く滲ませず、window の外の中身も混ぜない。CSS の backdrop root も viewport で終わる）。
  角の半径が違う影は角ごとの quarter の近似。大きな pattern の tile の境目は CPU と GPU で少し違う。
- **Linux。** WSLg（Ubuntu-24.04、Weston の上の Wayland と XWayland）で E2E を通しただけで、実際の desktop（GNOME・KDE）、fcitx・ibus の IME、
  AT-SPI の screen reader（Orca）、
  clipboard と portal の file dialog、GPU（Vulkan の hardware の adapter）では確かめていない。mouse の設定と reduced motion は desktop の設定を読まない。
  WSLg の X11 では Weston の window manager が assert で落ちて XWayland の接続が切れ、試験の process が終わることがある。
  `kivact-music-player` は ALSA の開発 package が要り、WSLg の検査では build しない。
- **platform の入力。** IME の再変換は Windows だけ（ほかの platform の IME の問い合わせは winit が渡さない）。
  OS の表示倍率そのものの変更、DPI の異なる monitor 間の移動、Windows 以外の実 window の DPI 変更は未検証。
  frame の周期は monitor の同一性が変わったときだけ読み直し、同じ monitor の refresh rate の変更は検出しない。
  precision touchpad の scroll は行単位の量で、pixel 単位の量と慣性はない。
  logoff・shutdown（`WM_QUERYENDSESSION`・`WM_ENDSESSION`）は受けないので、そのとき `CloseRequested` は届かない。
- **event loop。** winit は1つの process に event loop を一度しか作れない。試験の実 window を使う試験 binary は `App::run`・`Shell::new` を呼べない。
  macOS では winit の event loop を main thread の外に置けないので、試験の実 window は開かず headless だけになる。
- **資源。** GIF・WebP は動かない。読み込んだファイルの変更は監視しない。
- **GPU texture 共有。** `gpu-share` の共通契約と `Error` を import/export が共有する。`import`・`export`・`wgpu` の機能を持たない build には、対応する取り込み・pool・wgpu の API は存在しない。
  取り込んだ texture には画像のような最後の行と列の複製がないので、拡大縮小した bilinear で端の 1 texel が画像の描き方と違う（等倍なら一致）。
  ready が届かない frame は、frame ごとに CPU で最大 100 ms 待ってから諦め、その command を描かない（同じ値は二度待たない）。CPU の renderer と CPU で描く window は `DrawCommand::External` を何も描かない。
  device は window ごとにあり、取り込みはその window にだけ属する。複数 window で device を共有するのは native の複数 window に対応するときで、そのときは台帳を共有の device へ移す（公開 API は変えない見込み）。
  Vulkan では、取り込んだ memory の実際の寸法と image の usage を確かめる手段がないので、宣言と違う寸法や usage の texture は約束違反として未定義動作になる（Direct3D 12 は resource の記述で確かめて `Err` にする）。
  Direct3D 12 の debug layer を window の device ができた後に同じ process で有効にすると、window の device が失われる。engine の `wgpu::Instance` には `InstanceFlags::DEBUG` と `VALIDATION` を付けない。
  NVIDIA の Vulkan driver は、一度 export して閉じた memory の handle を後から export し直したものの取り込みを断ったので、engine は texture と fence を取り込みごとに一度だけ export する。
  Linux の実機（Surface Pro 6、Intel UHD Graphics 620、Mesa 26.2.5 の ANV、Debian sid、GNOME の Wayland）で、Vulkan の試験がすべて GPU で通った（2026-10-26）。その Khronos の validation layer（1.4.363）の下で、受け渡しの error は 0 件だった。ただし、window の描画の shader の作成（`vkCreateShaderModule`）で `VUID-StandaloneSpirv-None-10684`（Function の変数の struct 型が `Offset` の装飾を持つ）が出る。wgpu の shader compiler（naga）が書く SPIR-V の問題で、受け渡しとは関係しない（受け渡しの code は shader を作らない）。Windows の Vulkan の validation layer、API の違う組、別 process の engine では確かめていない。
  Kivact では、device を作り直すと（世代が変わる・CPU に落ちる）`Ready` の取り込みは `Lost` になる。低水準の `import` では、アプリが新しい adapter で texture を作って取り込み直す（自動では取り込み直さない。handle は一度しか渡せない）。高水準の `shared_image` は新しい `ShareRequest` を出し、engine が新しい pool を作って渡す。
  取り込みは App の1つの window の device に属する。`Lost` への遷移は device を作り直す試験の口がないので、E2E では確かめていない。

## 12. 外部依存

| crate | 使う crate | 用途 |
|---|---|---|
| async-task | kivact-task | UI thread の executor |
| kurbo（0.13.1、vello と同じ版） | kivact-style | `Path` の演算（変形・外接矩形・長さ・trim・線の輪郭。型は公開しない） |
| tracing | kivact-reactive | 構造化 trace と既定の error の受け口 |
| taffy（0.14） | kivact-layout | flex と grid の計算 |
| unicode-segmentation | kivact-editing・kivact-text | grapheme 単位の編集・行の処理・入力の境界 |
| image・resvg・skrifa・flate2 | arvane-asset（skrifa は arvane-render が glyph の検査にも使う） | 画像の decode、SVG の raster 化、フォントファイルの検査、SVGZ の展開 |
| winit・softbuffer・accesskit_winit・arboard・rfd | arvane-native | window・event・IME・提示・accessibility の接続・clipboard・file dialog |
| sys-locale（0.3） | arvane-native | OS の locale（BCP 47）の取得 |
| windows-sys（0.52、winit と同じ版） | arvane-native（Windows） | window の subclass（IME の問い合わせ `WM_IME_REQUEST`、button と wheel の位置、`WM_SETTINGCHANGE`、touch と pen の pointer の種類・取消し・傾き `WM_POINTER*`、touch から作られた mouse の message の破棄）と system の設定（reduced motion・double click・wheel） |
| png | kivact-engine（feature `testing`） | 試験 driver の screenshot |
| parley | arvane-text | shaping・bidi・改行・font fallback |
| fontique（parley と同じ版、Linux だけ） | arvane-text | feature `fontconfig-dlopen` を有効にするためだけ（fontconfig を実行時に読む） |
| vello_cpu（0.2）・vello_hybrid（0.2）・vello_common・wgpu（29）・pollster・raw-window-handle・bytemuck・glifo | arvane-render | CPU/GPU 描画（版の組合せと各 crate を使う理由は `arvane-render` の crate doc） |
| raw-window-handle（0.6、arvane-render と同じ版） | arvane-shell | `GpuTarget` の window と display の handle の契約 |
| accesskit | arvane-native | shell が受理した accessibility の木を native adapter へ変換 |
| ash（0.38、wgpu-hal と同じ版）・windows（0.62、wgpu-hal と同じ版、Windows だけ） | gpu-share・gpu-interop-producer（試験） | GPU texture 共有の共通の usage・flag と、Vulkan・Direct3D 12 の import/export（wgpu-hal の型と同じ版で渡す） |
| wgpu（`=29.0.4`、arvane-render と同じ版と features）・pollster | gpu-share（feature `import` または `wgpu`）・gpu-interop-producer（試験、feature `wgpu`） | 取り込みと wgpu の engine の補助。`as_hal`・`open_with_callback`・`create_texture_from_hal` に触れるので版を固定し、Arvane の wgpu と同時に上げる |
| trybuild | kivact-public-api-tests（dev） | コンパイル試験 |
| serde・serde_json・toml・rodio | 例・xtask | 保存・metadata の読込み・音声 |
| proc-macro2 | xtask | workspace の `macro_rules!` を呼ぶ path を token で探す（`trusted_macros` の自動化） |

複数 crate で使う外部依存の版と共通の feature は root の `[workspace.dependencies]` に一度だけ書く。各 manifest は `workspace = true` で参照し、用途ごとの追加 feature・optional の選択はその crate に残す。次の組は、接続する API と内部の型が合う版へ同時に上げる。

- wgpu・ash・windows（ash と windows は wgpu-hal が使う版にそろえる）
- vello_cpu・vello_hybrid・vello_common
- accesskit・accesskit_winit
- winit・windows-sys
- parley・fontique

Arvane の全 feature の依存は `cargo deny --manifest-path crates/arvane/Cargo.toml --config dev/tools/arvane-deny.toml check` で検査できる（手動）。
debug build でも、root の `Cargo.toml` で vello_cpu などの依存と taffy、engine（`kivact-engine`）と layout の計算（`kivact-layout`）と編集契約（`kivact-editing`）と値・motion（`kivact-wire`）、値の変換（`kivact-lower`）、Arvane の crate（受理済みの木の検証と差分を移した `arvane-shell` も含む）、wgpu・AccessKit・Parley を最適化する。
最適化しないと、frame ごとの layout・paint の一覧・accessibility・描画が debug build の drag と hover で 1 frame に収まらない（generic な layout は lower crate と engine の呼出側の両方で実体化し得るため、taffy の指定だけでは届かず、layout と engine の両方を指定する）。
独立した3つの consumer fixture も engine と layout の opt-level=3 を最初の Cargo より前に指定する。
App・core・kit の code は最適化せず、debugger で追える。engine を追うときは `--config 'profile.dev.package.kivact-engine.opt-level=0'`、layout を追うときは `--config 'profile.dev.package.kivact-layout.opt-level=0'` で build する。

## 13. 動かし方

```text
cargo run -p kivact-template-basic                            # GPU（使えなければ CPU）。ARVANE_RENDERER=cpu で CPU
cargo run -p kivact-workbench --bin workbench                       # 10万件の仮想一覧と外部部品
cargo run -p kivact-hostbench --release -- text                     # host の性能の1場面（KIVACT_FRAME_LOG で frame の時刻）
cargo test -p kivact-public-api-tests --release --test e2e -- --ignored --nocapture --test-threads=1 perf:: scale::
cargo test -p kivact-core --features test --lib real_window_tests                         # kivact::test の smoke（実 window）
cargo test -p kivact-core --features test --lib real_window_tests::cost -- --ignored    # 試験 1 件の費用（target/kivact-test-cost.txt）
dev/tools/linux/test-wslg.sh [wayland|x11|all]                              # Linux の検査（Windows の Git Bash から WSL の中で。下）

```

window を開く試験は `dev/tools/window-lock.sh -- <command>` を通す（10章）。
Linux の検査（`dev/tools/linux/test-wslg.sh`）は、Windows の Git Bash から動かすと WSL の distribution（`KIVACT_WSL_DISTRO`、
既定 `Ubuntu`）の中で同じ作業木に対して自分を動かし直す。
build の出力は `CARGO_TARGET_DIR`（既定 `$HOME/kivact-target`、distribution の ext4）に置き、Windows の `target/` と分ける。
clippy（`-D warnings`）の後、display server ごとに Arvane の契約の試験と workspace の試験（`--no-fail-fast`）を走らせ、
log を `$CARGO_TARGET_DIR/test-wslg/` に残す。
`kivact-music-player` は除く（`libasound2-dev` が要る）。sudo の要らない build のため、
Linux の開発 package は要らない（8章）。

標準の field は label を押すと control をクリックと同じに起動します。文字入力は focus し、checkbox と switch は切り替わります。disabled・非表示・利用できない control の起動は、共通の NodeRef activation が拒みます。

複合入力の主 editor は NodeExt の preserved source（name・invalid・labelled_by・described_by）を root から転送します。NodeRef の関係は consume せず追加し、参照先の寿命は共通の relation publication が追います。number_input と tag_input は主 editor を semantic/focus 順の先頭に置き、grid で既存の見た目の順を保ちます。label activation の対象探索は従来どおり最初の利用できる focus 対象です。
