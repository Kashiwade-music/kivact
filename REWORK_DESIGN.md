# Kivact 抜本改修（Rework）設計書

版 1.0（2026-11-18）。作業ブランチ `refactor/kashiwade/fundamental-rework`（基点 `2b8df5ac` = main）。

本書は、Kivact の破壊的で根本的なリファクタリングと性能改善の**設計の正本**である。実装者は本書・付属書（`rework/*.md`）・作業指示書（[REWORK_WORK_ORDER.md](REWORK_WORK_ORDER.md)）だけを読み、設計や探索をせずに実装できることを目標に書いた。

## 0. 文書の構成と優先順位

| 文書 | 役割 | 読む人 |
|---|---|---|
| 本書 `REWORK_DESIGN.md` | 目的、KPI、目標の構成、公開 API の契約、内部の規則、所見の判定、決定 | Orchestrator、全 Implementer（自分の unit に関係する節） |
| 付属書 `rework/<X>-*.md` | 領域ごとの unit の手順（所有ファイル、変更内容、コード、確認のコマンド） | その unit の Implementer |
| 作業指示書 `REWORK_WORK_ORDER.md` | wave・lane の進め方、subagent の起動、merge、gate、checkpoint、STOP の扱い | Orchestrator（Implementer は 0 章の規則だけ） |
| 根拠 `rework/audit/*.md` | 監査報告（2b8df5ac 時点の file:line と証拠）。所見 ID（`API-01`、`RC-05` など）の出典 | 必要なときだけ |

食い違ったときの優先順位：**作業指示書の「追加の指示」 → 付属書 → 本書 → 監査報告**。ただし本書 5 章（公開 API の契約）と 6 章（内部の規則）は、付属書より優先する。付属書が契約と違うコードを書いていたら STOP する。

監査報告の行番号は `2b8df5ac` のものである。前の unit がファイルを変えるので、行番号ではなく関数名・型名で場所を探す。

## 1. 目的と受入れ条件

目的：最近の規律のない機能追加で増えた複雑さを根本から整理し、次の 6 つの基準で Kivact を作り直す。公開 API の互換は壊してよい。利用者は追従するが、利用者が本質的にしたいことは必ず実現できる経路を残す（互換層は残さない）。

1. Rust 未経験の React・SolidJS 開発者が滑らかに書ける公開 API
2. 分かりやすく統一的な公開 API（一つのことに一つの書き方）
3. 普段 JavaScript や Python を書く人が容易に書ける（Rust 特有の作法を利用者のコードに出さない）
4. Kivact 内部が統一的で単純
5. 長期の保守性
6. [docs/for-kivact-developer/strategy.md](docs/for-kivact-developer/strategy.md) を簡単に遂行できる

受入れ条件（AC）：

| AC | 条件 | 判定する時点 |
|---|---|---|
| AC-1 正しさ | 作業指示書の full gate（tier c）がすべて成功する | 最終 checkpoint |
| AC-2 性能 | 2 章の KPI の「必達」行をすべて満たす。「目標」行は満たさなくても完了を妨げないが、結果を記録する | 最終 checkpoint |
| AC-3 状態 | 6.1 節の新しい状態の検査が全 workspace で 0 件。監査報告の STATE 所見で「採用」としたものがすべて閉じている | 最終 checkpoint |
| AC-4 公開 API | 5 章の契約がすべて実装され、5.12 節の削除名が workspace に残っていない（`rg` で確かめる。作業指示書の検査コマンド） | 最終 checkpoint |
| AC-5 文書と移行 | 利用者向け文書・README・例・雛形が新しい API だけを使う。`docs/for-kivact-developer/` の 3 文書が新しい構成と規則を書く。利用者向けの移行ガイド `docs/for-kivact-user/{en,jp}/how-to-guide/migrating-to-rework.md` がある | 最終 wave |

## 2. KPI

指標の定義は [dev/bench/README.md](dev/bench/README.md) の K1〜K6 のまま使う（起動時間・メモリ・CPU・フレーム時間・更新の遅延・バイナリサイズ）。

- 基準 `B0` は、作業を始める前に**同じ機械**で基点 `2b8df5ac` を `dev/bench/run.sh B0` で測った値だけを使う（作業指示書 W0）。
- 公開 API が変わるので、`dev/bench` の probe（`kv_startup`、`kv_stress`、`kv_latency`、`kv_heap`）は API の移行 unit が一緒に書き換える。**測る画面と操作は変えない**（同じ要素の数、同じ入力の台本）。
- 比較は比で行う：`比 = 改修後 / B0`。時間の行は雑音があるので、必達に届かない行は同じ条件で 2 回測り直し、3 回中 2 回で判定する。

| KPI | 行 | 必達（AC-2） | 目標（記録） | 主な手段（所見 ID） |
|---|---|---|---|---|
| K1 | `kv_startup`（counter と同じ画面）の起動 | ≤ 1.00 | ≤ 0.85 | system font の走査を UI thread から外す（AR-12）、DevTools agent の非同期起動（RC-10）、env の memo の遅延（RC-41）、pipeline の遅延（AR-13）、asset worker の遅延（AR-12） |
| K1 | basic catalog の起動 | ≤ 1.00 | ≤ 0.80 | 上と同じ＋ recipe 表の一括構築の廃止（UI-03） |
| K2 | counter の idle WS | ≤ 1.00 | ≤ 0.90 | 診断の常時記録の廃止（RC-05、RC-06）、provenance の廃止（RC-22） |
| K2 | 3000 行の idle WS | ≤ 1.00 | ≤ 0.85 | 上と同じ＋ HostNode・taffy cache の縮小（EN-23） |
| K2 | item あたり（box / text / button / row） | ≤ 1.00 | ≤ 0.85 | RNode の縮小（RC-06）、per-item memo の削減（UI-01、UI-02）、draw cache は描画中の node だけ（EN-03） |
| K3 | idle の CPU | ≤ 1.00 | ≤ 1.00 | GPU window の idle wake の廃止（AR-09） |
| K3 | 3000 行・毎 frame 更新の CPU | ≤ 1.00 | ≤ 0.60 | draw cache（EN-03）、restack の削減（EN-04）、content_version（AR-03）、motion の索引（RC-08） |
| K4 | 3000 行の work p95 / p99 | ≤ 1.00 | ≤ 0.60 | 同上 |
| K4 | 描画間隔 p99（3000 行） | ≤ 1.00 | ≤ 0.95 | frame scope（AR-01）、damage の一括（AR-04） |
| K5 | pointer p95 / wheel / row 更新 / unrelated text 更新（n3000） | ≤ 1.00 | ≤ 0.70 | hit test の索引（EN-09）、PERF-1 違反の解消（EN-05〜EN-08、EN-11）、selector（UI-02） |
| K5 | mount n3000 | ≤ 1.00 | ≤ 0.90 | style の compile の削減（ST-06、RC-22）、境界の fragment 削減（RC-22） |
| K5 | counter click | ≤ 1.00 | ≤ 1.00 | — |
| K6 | GPU を除く build の `kv_startup` | ≤ 1.00 | ≤ 0.95 | naga を GPU build だけに（ST-25）、resvg・tiny-skia・fontdb の削除（AR-11） |
| K6 | 既定 build（`gpu` 込み）の counter と basic catalog | ≤ 1.00 | ≤ 0.95 | 同上＋ gpu-share の import/export を gpu-interop だけに（AR-18） |
| K6 | GPU を除く build の basic catalog | ≤ 1.00 | ≤ 0.90 | recipe 表の分割（UI-03）、plot の crate 化（KIT-15） |

「必達」はどの行も**後退しない**こと（≤ 1.00）とする。時間の行（K1・K3・K4・K5）は雑音を考えて 1.05 まで許す。大きさの行（K2・K6）は 1.02 まで許す。

## 3. 利用者の必須経路

利用者（非公開を含む 4 つのリポジトリ）は追従するが、次のことは改修後も公開 API で実現できなければならない。右の列が改修後の経路である。どれかを実現できなくする変更は、付属書に書いてあっても STOP する。

| ID | 利用者がしたいこと | 改修後の経路 |
|---|---|---|
| CAP-01 | window を開いて event loop を回す（大きさ・最小の大きさ・theme・font） | `App::new(title).window_size(..).min_window_size(..).theme(theme).run(root)`。font は theme が持つ（`Theme::with_fonts`、5.7 節） |
| CAP-02 | DAW plugin の editor として host の native window（Win32、X11）に埋め込む。非表示で始める、scale を受け取る、未処理のキーを host に返す、resize の要求に host が非同期に答える、外から表示・非表示・大きさ・scale を変える、停止・失敗・panic を知る、安全に drop する | feature `embed` の `App::embed(Parent, root) -> Result<Embedded>`。全 OS で compile でき、未対応の環境では `ErrorKind::Unsupported`。停止は `EmbedNotice::Stopped(EmbedStatus)` の一つの通知（5.10 節） |
| CAP-03 | 任意の thread とアプリの中から window を閉じる | `App::handle()`（run の前）と `platform::app_handle().close()` |
| CAP-04 | window の close 要求を止める（未保存の確認） | `platform::on_close_request(handler)`。呼んだ owner の window に効く（5.10 節） |
| CAP-05 | 他の thread から UI を起こし、reactive な状態にデータを入れる | `task::remote(initial) -> (Signal<T>, Remote<T>)` と `task::remote_trigger() -> (Trigger, RemoteTrigger)`（新設、5.5 節） |
| CAP-06 | 非同期の native file dialog（開く・保存・folder）と、型付きの失敗 | `platform::{pick_file, save_file, pick_folder, FileDialog}`（変更なし） |
| CAP-07 | 独自描画（`canvas`、`Draw`、`Path`、`Stroke`、`Shadow`、文字）、canvas 上の hit region、毎 frame の再描画 | `canvas`、`kivact::draw::*`、`region`、`frame_loop(..)`。CSS の値の型は `kivact::style` へ移る（5.8 節） |
| CAP-08 | payload・局所座標・追加の button・slop・修飾キー・取消し・編集の begin/commit/cancel を持つ drag | `interaction::use_drag`（変更なし）。event に対象の大きさ `target_size()` を足す |
| CAP-09 | すべての標準部品を作り替える design system kit を作る | `ThemeVar`（値）と `<Name>Theme`（形の recipe）、`Look<S>`、`Variant`、`Theme::with`、`Theme::with_icons`（5.7・5.9 節） |
| CAP-10 | 支援技術に対応した独自 control を作る | `#[component] fn x(..) -> impl Element` ＋ `node()` の modifier（role、name、value、focusable、a11y_action、on_key、on_pointer、part、paint）と `use_scalar` |
| CAP-11 | 標準画面（dialog、menu と menubar、tabs、split、tooltip、select、text 入力、input group、toggle group、switch、checkbox、radio group、toast、scroll、data table、tree） | `kivact::ui`（形は 5.9 節で統一） |
| CAP-12 | text field に focus があっても動く window 単位の shortcut | root の `.on_key(..)` と `Handled`（変更なし） |
| CAP-13 | plot と window なしの書き出し（PNG、SVG） | 新しい crate `kivact-plot`（KIT-15）と `kivact::draw::Offscreen`（新しい crate `kivact-offscreen` を facade が再公開。RC-29） |
| CAP-14 | 利用者と同じ入力で UI 全体を試験する | `kivact::test`（変更なし。`TestFonts::Only` は theme の font に置き換わる） |
| CAP-15 | reactive の中核（signal、memo、effect、context、spawn、sleep、show、dynamic、for_each、Copy な handle） | 5.2〜5.6 節 |
| CAP-16 | audio plugin の parameter control（knob、fader、value slider、number field と EditPhase、ScalarStep、drag の距離、InteractionPolicy） | `kivact-ui-creative`（残す）と `interaction::use_scalar` |
| CAP-17 | font と icon の登録 | `Theme::with_fonts`、`Theme::with_icons`、`IconSet::with(name, svg)` |
| CAP-18 | 実行中の theme の切替え | `provide_theme(move || ..., child)` |

利用者のための移行ガイド（AC-5）は、最終 wave の付属書 G が書く。利用者の Cargo.toml が Kivact の内部 crate 名を `[profile.*.package.*]` に並べている問題（crate の分割・統合で利用者の build が壊れる）は、ガイドで `[profile.dev.package."*"] opt-level = 3` の 1 行に置き換えるよう案内する（README の「Kivact を使う」節にも書く）。

## 4. 目標の構成

### 4.1 crate の増減

| 変更 | crate | 境界の理由（B-1：少数の型で入出力を説明でき、相手の内部を見ず、変更理由が一つ） | 所見 |
|---|---|---|---|
| 新設 | `arvane-protocol` | `kivact-types/src/render/*`（RenderScene、Composition、CpuFrame、SemanticsTree、描画用の Paint・Gradient・span）。作るのは engine だけ、使うのは arvane-* だけ。renderer との契約という一つの変更理由 | ST-20 |
| 新設 | `kivact-offscreen` | `kivact-core/src/draw/offscreen.rs`。入力は Theme・font・描画の closure、出力は `Pixels` か SVG の文字列。view と runtime を読まない。core から kivact-svg への依存が消える | RC-29 |
| 新設 | `kivact-plot` | `kivact-ui-basic/src/plot/*`。入力は `PlotModel`、出力は plot の view。色は theme 変数で受け、どの kit とも組み合わせられる。分析ツールという変更理由は kit の見た目と別 | KIT-15 |
| 新設（試験用） | `dev/tests/kivact-kit-tests`（`publish = false`、test binary `e2e_kits`） | kit に依存する公開 API の E2E（theme_conformance など）。cargo は一つの package のすべての test target に dev-dependency を compile するので、kit を `kivact-public-api-tests` から分けないと DEC-14 の間に `e2e_core` なども compile できなくなる | TL-15、DEC-14 |
| 新設（開発用） | `examples/catalog-shell`（`publish = false`） | 5 つの catalog の shell・page 一覧・検索・scroll-spy。入力は `Vec<Page>` と theme の関数 | KIT-12 |
| 削除 | `kivact-devtools-client` | `kivact-devtools-protocol` の `client` module（feature `client`）に統合する。client と protocol は同じ理由で同時に変わり、client だけを使う利用者がいない | TL-08 |
| 削除（crate ではない） | `python/` と xtask の `devtools_codegen*`、DevTools の GUI の Scripts パネルと、それだけが使う protocol の method | 利用者がいない。AI からの利用は MCP が同じ schema で担う。Scripts パネルは Python client が無いと動かない | TL-09、DEC-10 |
| 縮小 | `arvane` | facade の役割を捨て、window driver と presenter だけにする。他の crate の再公開をすべて消し、engine は arvane-text・arvane-compose・arvane-asset・arvane-shell を直接使う | AR-06 |
| 縮小 | `kivact-types` | render protocol を `arvane-protocol` へ、`prop_interp` を engine へ移す。`cell`・`process` は残し、crate の doc を「純粋な値と、std だけの小さな同期 helper」に直す | ST-20 |

**分割しないと決めたもの**（境界が明確でないため。B-3）：kivact-core（view・client・platform・devtools は node 表の内部を共有する）、kivact-engine（host が model の内部を直接読む）、engine の `lower`・`text_input`（狭い API ができるまで module のまま）、arvane-native の embed（a11y・services・入力・present を共有する）、arvane-render と arvane-compose、kivact-ui の「分野の部品」（calendar・chat など。2 つ以上の kit が使う部品は中核に置く、という strategy の規則どおり）、kivact-types の `cell`。

結果：50 package → 54 package。

### 4.2 依存の向き（変更後）

```
kivact-types ← arvane-protocol ← arvane-render ← arvane-compose ← arvane (driver)
kivact-types ← arvane-shell ← arvane-native, arvane-virtual
kivact-types ← arvane-text ← arvane-asset → arvane-render、arvane-protocol（asset は SVG の文字に text の font を、SVG を RenderScene にするのに protocol と render を使う）
kivact-engine → arvane, arvane-shell, arvane-native, arvane-virtual, arvane-text, arvane-render, arvane-compose, arvane-asset, arvane-protocol, kivact-layout, kivact-editing
kivact-runtime ← kivact-style, kivact-i18n, kivact-core
kivact-offscreen → kivact-runtime（`UiError` などの error 型だけ）, kivact-style, kivact-engine, kivact-svg, kivact-types
kivact-core → kivact-runtime, kivact-style, kivact-engine, kivact-editing, kivact-devtools-agent, kivact-devtools-protocol（kivact-svg への依存は消える）
kivact-interaction → kivact-core ;  kivact-ui → kivact-core, kivact-interaction
kivact (facade) → kivact-core, kivact-interaction, kivact-ui, kivact-offscreen, kivact-macros, kivact-test(feature)
kivact-ui-* (5 kits), kivact-plot → kivact だけ（例外：kivact-ui-basic → kivact-plot。basic の chart が目盛りの計算を共有し、basic の theme が plot の色の変数を set するため）
kivact-devtools (GUI) → kivact, kivact-devtools-protocol(client)   ※ kivact-ui-basic への依存を消す（KIT-18）
```

`dev/xtask/policy/workspace.toml` の `visible_to` をこの図に合わせる（付属書 T）。

### 4.3 消すもの（領域をまたぐ一覧）

- 生成物：`architecture/`（git から外し `.gitignore` に入れる。xtask の図の生成は `cargo xtask diagrams` の別の subcommand にし、gate から外す）、`dev/xtask/state-audit.md` の生成。
- 印：`xtask-allow(state-fields)` の 1182 行（6.1 節の新しい検査に置き換え、受動的な record の印は消す）、同じ理由文を繰り返す `xtask-allow(internal-contract)`・`xtask-allow(api-shape)`（規則の側で例外にする）、例と雛形の `#![warn(redundant_imports)]` と `//! Role:`（例・雛形・docs は Role 規則の対象外にする）。
- README の「Migrating from the previous release」の表と「Filled recipes」の一覧（5 つの kit）と、それを検査する xtask。
- 過去の不具合の再現だけの試験と `// Regression:` の札（TL-14）。

## 5. 公開 API の契約

この章は**変更後の公開 API の正本**である。付属書の unit はこの契約を実装する。名前・署名・意味をここから変えない。変えないと実装できない場合は STOP する。

### 5.1 部品の書き方は一つ：`#[component]`

アプリ・`kivact-ui`・5 つの kit・`kivact-plot` の部品は、すべて `#[component]` の関数で書く（API-01、USR-09）。手書きの builder struct と `impl Element for` / `impl IntoView for` は、kivact-core が定義する基本要素（`node`、`text`、`column`、`row`、`stack`、`grid`、`scroll`、`spacer`、`image`、`icon`、`canvas`、`window`、`overlay` など）だけに残す。

```rust
use kivact::prelude::*;

#[component]
pub fn card(
    title: Value<String>,              // 必須の props は位置引数（宣言順）
    children: Vec<View>,               // 子
    on_close: Option<Callback<()>>,    // Option<..> は任意の props。同名の setter になる
    #[prop(default = 1)] level: u8,    // 既定値つきの任意の props
) -> impl Element {                    // impl Element を返すと呼び出し側が .style(..) などを付けられる
    let open = signal(true);
    column((
        row((text(title), on_close.map(|close| button("Close").on_press(close)))),
        show(open, move || children.clone()),
    ))
}

// 呼び出し側
card("First", vec![text("body").into_view()])
    .on_close(move || first.set(false))
    .style(style().width(320))
```

規則：

1. 戻り値の型は `impl Element`（ui・kit・plot はこれだけ）か `impl IntoView`（アプリで、root に modifier を付けさせないもの）。
2. `impl Element` のとき、macro が生成する struct は隠した field `__root: Node`（空の `node()`）を持ち、`Element::node_mut` はそれを返す。`into_view` は本体を `scope` の中で一度だけ実行し、本体の root に `Node::absorb(__root)` で呼び出し側の modifier を移す。`absorb` の規則：呼び出し側の style の層を本体の層の後ろに足す、handler は本体の handler より前に足す（`prepend_key` と同じ順の規則）、node_ref は本体の ref に**追加**する（本体の ref を外さない）、name・description・disabled・test_id・variant は呼び出し側の値で上書きする。
3. props の種類は型の**字面の名前**で決まる：`Value<T>`（定数・closure・`Signal`・`Value` を受ける）、`Signal<T>`（双方向。`Signal` だけを受ける）、`Callback<()>`（`IntoHandler<(), _>` を満たすものを受ける）、`Callback<E>`（`E` が `()` 以外。`impl Fn(E) -> R + 'static` を受ける。5.3 節）、`View`（`IntoView` を受ける）、`ViewFn`（`IntoViewFn` を受ける）、`Vec<View>`（`IntoChildren` を受ける）。それ以外の型 `T` は `impl Into<T>` を受ける。別名（`type Title = Value<String>`）は「それ以外」として扱う。macro は認識した props ごとに、`::kivact::state::Value<T>` などの実在を確かめる式を出す（API-18）。
4. `Option<X>` だけで任意になる。`#[prop(optional)]` は消す。`#[prop(default = expr)]` は残す。
5. 引数なしの部品も `#[component]` で書く（`#[component] fn header() -> impl Element`）。
6. macro は自分の crate の path を `proc-macro-crate` で解決する：依存に `kivact-core` があれば `::kivact_core`（kivact-ui と kivact-interaction のため。この二つは `kivact` を dev-dependency に持ち、`proc-macro-crate` は dev-dependency を区別できないので、`kivact-core` を先に調べる）、無ければ `::kivact`。アプリと kit は `visible_to` により `kivact-core` に依存できないので `::kivact` になる。生成するコードは解決した path だけを使う。
7. `Callback<E>` の props は内部では runtime の `Handler<E>`（`Rc<dyn Fn>`）で持ち、本体には `Callback<E>`（Copy）として渡す。owner への登録は本体が子に渡すときだけ行う（RC-33）。
8. `scope(FnOnce)` は `kivact::custom` へ移す（kit 作者の逃げ道。prelude から外す）。
9. 部品の関数は型引数を持てる（`#[component] pub fn select<K: ItemKey>(items: Value<Vec<ListItem<K>>>, value: Signal<Option<K>>) -> impl Element`）。生成する struct は同じ型引数と bound（`where` を含む）を持ち、すべての型引数に `'static` を要求する。props の field に現れない型引数は `PhantomData<fn() -> K>` の隠した field で持つ。lifetime 引数と const 引数は受けない（compile error で「型引数だけ」と案内する）。

### 5.2 状態（`kivact::state`）

| 名前 | 意味 | React / Solid の対応 |
|---|---|---|
| `signal(v) -> Signal<T>` | 書ける状態 | `createSignal` / `useState` |
| `Signal<T>` | Copy。`get`、`get_untracked`、`with`、`with_untracked`、`try_get`、`set`、`update(f: impl FnOnce(&mut T))`（戻り値なし）、`optional() -> Signal<Option<T>>` | — |
| `binding(get, set) -> Signal<T>` | 読みと書きの関数から作る双方向の値（lens）。`Signal` と同じ型 | 制御された `value` + `onChange` |
| `bind!(s.a.b) -> Signal<U>` | struct の field への lens | — |
| `memo(f) -> Value<T>` | 派生値 | `createMemo` / `useMemo` |
| `Value<T>` | Copy の読み取り専用。`get`、`get_untracked`、`with`、`with_untracked`、`try_get` | accessor |
| `var(v) -> Var<T>` | 追跡しない Copy の箱。アプリの object を Copy な handle にするのにも使う | `useRef` |
| `callback(f) -> Callback<E>` | Copy の handler。`call`、`try_call` | `useCallback` |
| `trigger() -> Trigger` | 値を持たない変化の通知。`track`、`notify`（旧 `Notifier`） | `createTrigger` |
| `selector(source) -> Selector<K>` | `is(&k)` が key ごとに O(1) で追跡される | `createSelector` |
| `effect`、`effect_on`、`on_mount`、`on_cleanup`、`untrack` | 効果と寿命 | `createEffect`、`onMount`、`onCleanup` |
| `provide(value, child)`、`use_context::<T>() -> T`、`try_use_context::<T>() -> Option<T>` | context。`T: Clone`。reactive にしたいときは `Signal`・`Value` を入れる | `useContext` |

変更点（所見）：

- `Binding<T>` と `IntoBinding<T>` を消し、`Signal<T>` に統合する。`Signal` の内部表現は `enum SignalRepr { Source(Key), Lens { read: Key, write: Key } }`（API-02）。入力部品は `value: Signal<T>`（具体型）を受ける。行の source が `Signal<Vec<T>>` の `for_each` は行に `Signal<T>` を渡し、読み取り専用の source は `Value<T>` を渡す。`Signal::project` は `#[doc(hidden)]`（`bind!` の展開先）。
- `Signal::notify` を消す（`Trigger` を使う）。
- `set` と `update` は、破棄された signal に対して同じ動き（位置付きの `WriteAfterDispose` 診断を一度記録して何もしない）。`update` は値を返さない（`on_press(move || n.update(|v| *v += 1))` の closure の戻り値が `()` のまま handler になるため。計算した値が要るときは `update` の後に `with` で読む）。`Var::update` も同じ形にする（API-15。決定 DEC-15）。
- `use_context::<T>()` は `T` を返す（`Value<T>` で包まない）。runtime と core の二つの context の置き場を一つにする（API-04、RC-31）。
- `Notifier`・`notifier()` は `Trigger`・`trigger()` に改名する（USR-03）。
- `Callback::bind`、`Callback::to_fn` を消す。`Callback::call_if_alive` は `#[doc(hidden)]`（API-11）。
- 変換 trait は `IntoValue<T>`、`IntoText`、`IntoHandler<E, M>` の 3 つだけにし、prelude には入れない。`IntoReactive`（flow は `IntoValue<bool>` を受ける）、`custom::Reactive`（`Prop<T>` を `#[doc(hidden)]` で公開して置き換える）、`IntoValue<Rc<str>>` の impl を消す（RC-32）。
- 独自の型を props の定数にするには `#[derive(Prop)]` を付ける（`constant_value!` は `#[doc(hidden)]`。API-06）。
- `PropValue`、`ItemKey` は `#[doc(hidden)]`（診断用の bound）。`Owner`・`UiThreadOnly` は `kivact::custom` だけに置く。

### 5.3 event の handler（`IntoHandler`）

handler の受け方は二つだけ（API-03。決定 DEC-16）：

- **payload の無い handler**（`on_press` など、`#[component]` の `Callback<()>` props）は `impl IntoHandler<(), M>` を受ける。closure `Fn() -> R` と `Callback<()>` の両方を渡せる。
- **payload のある handler**（`on_key`、`on_activate`、`on_commit`、`on_edit`、`#[component]` の `Callback<E>` props）は `impl Fn(E) -> R + 'static` を受ける。rustc は closure の引数の型を `Fn` の bound からしか推論しないので、`|n| n.abs()` を型注釈なしで書けるようにするため。共有した `Callback<E>` を渡すときは `move |e| cb.call(e)` と書く（ガイドに一度書く）。

```rust
// kivact-runtime/src/reactive/handler.rs
pub trait IntoHandler<E, M>: sealed::Sealed<M> { fn into_handler(self) -> Handler<E>; }
// 実装：F: Fn() -> R（E = ()、M = Nullary）、Callback<()>（M = Handle）。R: HandlerResult
// Handler<E> = Rc<dyn Fn(E) -> Result<Handled, UiError>>（arena の object を作らない）
```

`HandlerResult` は `()`、`Handled`、`Result<(), UiError>`、`Result<Handled, UiError>` の 4 つ（変更なし）。`From<F> for Callback<()>` は消す。共有したい handler は `callback(f)` で作る。

handler の名前（UI-13）：payload のない押下は `on_press`、key のある項目の実行は `on_activate(Fn(K))`、確定は `on_commit(Fn(T))`（`on_draft_commit` を改名）、編集の段階は `on_edit(Fn(EditPhase))`。`on_step_press`、`Avatar::on_open`、`TagInput::{on_add, on_remove}`、`HoverCard::on_open_change`、`NavigationMenu::on_open_change` は消す（値は `Signal` で受ける）。

### 5.4 view と要素（`kivact::view`）

- `Element` trait に `fn variant<V: Variant>(self, v: impl IntoValue<V>) -> Self` を足す。variant は node が `Option<Box<Variants>>` で持つ（MEM-2）。ui と kit の 68 個の `variant` の写しは消す（UI-14、KIT-04）。
- `Element` の method と同じ名前の固有 method を、`Element` を実装する型に置かない（`Card::disabled` などを消す。API-13）。xtask の検査を足す。
- flow：`show(cond: impl IntoValue<bool>, f: impl IntoViewFn)`、`show_some(v: impl IntoValue<Option<T>>, f: impl Fn(Value<T>) -> V)`、`dynamic`、`for_each`、`for_index`、`virtual_list`、`error_boundary`。開いたときに作る content の引数はすべて `impl IntoViewFn<V>`（引数なし）か `impl Fn(Arg) -> V`（引数あり）。`provide` と `scope` は `FnOnce`（API-16）。
- `kivact::view::commit_focused_edit() -> impl Future<Output = ()>`（新設、USR-05）：focus を持つ control の「focus を失ったときの確定」を実行し、その結果が state に書かれたら完了する。アプリが focus を外すための見えない node と待ち時間を不要にする。
- `PointerEvent::target_size() -> Size` と `DragUpdate::target_size() -> Size`（新設、USR-16）：handler を付けた node の大きさ（`pos` と同じ座標系）。
- `spacer()` は `min_width(0).min_height(0)` を含む（USR-17）。

### 5.5 非同期（`kivact::task`）

- `resource(fetch)`、`spawn`、`blocking`、`sleep`、`timeout`、`on_tokio`（feature）は変更なし。`blocking` を App の外で呼ぶと位置付きの `WrongApp`（RC-38）。
- `action(f) -> Action<T, I>` は Latest（新しい実行が古いものを取り消す）。順に実行するものは `queued_action(f)`。`ActionMode` は消す。`Action::close` を消す（RC-37）。
- `on_complete(Fn(Result<T, UiError>))`。`ActionOutcome` を消す。`Retry` は `Callback<()>` にする（RC-35）。
- `remote<T: Send + 'static>(initial: T) -> (Signal<T>, Remote<T>)`（新設、USR-02）：`Remote<T>: Clone + Send + Sync` は `set(T)` と `update(impl FnOnce(&mut T) + Send)`。書込みはまとめられ（UI の turn ごとに最新の値を一度だけ届ける）、event loop を起こす。
- `remote_trigger() -> (Trigger, RemoteTrigger)`（新設）：`RemoteTrigger::notify()` は `Send + Sync`、決して block しない、まとめられる。
- `clock(every: Duration) -> Value<SystemTime>`（新設、KIT-17）：同じ `every` の呼出しは一つの timer を共有する。
- `async_view(source, ready).loading(f).fallback(f)` を非同期の表示の唯一の標準形にする（API-08）。

### 5.6 アプリと platform（`kivact::App`、`kivact::platform`）

- `App` は `Clone` を実装しない。`App::run(self, root)`。`launch::AppOptions` は非公開の field 型にする（RC-13）。
- `App::fonts` と `App::font` を消す。font は theme が持つ（`Theme::with_fonts(&'static [&'static [u8]])`。`App::theme(t)` と `kivact::test::RenderOptions::with_theme(t)` がその font を登録する）。`TestFonts::Only` を消す（USR-12）。
- GPU の起動の希望 5 型（`GpuPreference`、`GpuPower`、`GpuBackend`、`GpuStart`、`AdapterMatch`）は `kivact::platform` に移し、`kivact::launch` を消す（API-11）。`GpuStart::Never`（CPU だけ）を足し、`prefer_gpu` を全部消す（AR-16、ST-23）。環境変数 `ARVANE_RENDERER=cpu|gpu|auto` は `Never|AtLaunch|OnDemand` に対応する。
- `platform::on_close_request(handler: impl IntoHandler<(), M>)` は setup の中だけで呼べ、**呼んだ owner の window**（main window、`window()`、foreign window）に効く。main 以外の window では、handler が `Handled::Yes` を返さなければ従来どおり `on_dismiss(DismissReason::WindowClose)` が続く（RC-11）。
- `use_direction() -> ReadingDirection`（Value で包まない。RC-18）。
- embed（feature `embed`）：`kivact::embed` は全 OS で compile できる。未対応の環境の `App::embed` は `Err(ErrorKind::Unsupported)`。停止は `EmbedNotice::Stopped(EmbedStatus)`（Closed / Failed / Panicked）で一度だけ知らせる。`embed::Visibility` は `EmbedVisibility` に改名する（USR-15、API-10、ST-21）。

### 5.7 style と theme（`kivact::style`）

**一つの style の model**（ST-01、ST-05、ST-08、ST-22）：

1. `style()` は宣言の列を作る。後の宣言が勝つ。`.style(a).style(b)` も同じ規則で重なる。
2. 値は CSS の型：`Length`（数は px、`pct`、`em`、`+`/`-` による calc）、`Color`、CSS の値の builder。すべて `kivact::style` にある。
3. theme の値は **theme 変数** `ThemeVar<T>`。`color::ACCENT`、`space::MD`、`radius::MD`、`font::SIZE` のような定数として、値を置ける場所ならどこにでも置ける。closure は要らない。theme が変われば解決し直す（CSS の `var(--x)`）。
4. 状態は入れ子の style で、**すべての property を受ける**：`.hover(..)`、`.active(..)`、`.focus(..)`、`.focus_visible(..)`、`.disabled(..)`、`.selected(..)`、`.group_hover(..)`、`.group_focus(..)`、`.enter(..)`、`.exit(..)`。
5. 部品の見た目は `.variant(v)` で選ぶ。theme は部品ごとの recipe 表（`<Name>Theme`）で部品の形を作り替える。reactive な style（アプリの状態に依存するもの）だけ `.style(move || ..)` を使う。

```rust
// 変更前
.style(move || { let t = use_theme(); t.with(|t| style().padding(t.space().md).radius(t.radius().md)).background(ColorName::Primary) })
// 変更後
.style(style().padding(space::MD).border_radius(radius::MD).background(basic::PRIMARY))
```

theme 変数：

```rust
// kivact-types（純粋）
pub struct ThemeVar<T: 'static> { name: &'static str, fallback: T }
impl<T> ThemeVar<T> { pub const fn new(name: &'static str, fallback: T) -> Self }
// Color と Length は theme 変数を指す const の値を持てる（Color は Copy のまま）
impl Color { pub const fn var(v: &'static ThemeVar<Color>) -> Color }
impl Length { pub const fn var(v: &'static ThemeVar<f64>) -> Length }
// kivact-style
impl Theme {
    pub fn set<T: ThemeValue>(self, var: &'static ThemeVar<T>, value: impl Into<T>) -> Theme; // 値は別の変数でもよい（別名。深さ 8 まで。循環は Unsupported を一度報告して fallback）
    pub fn get<T: ThemeValue>(&self, var: &'static ThemeVar<T>) -> T;
}
// 中核の変数：kivact::style::{color, space, radius, font} の module に pub const を置く
pub mod color { pub static ACCENT_VAR: ThemeVar<Color> = ThemeVar::new("accent", rgb(0x2563eb)); pub const ACCENT: Color = Color::var(&ACCENT_VAR); /* SURFACE, TEXT, TEXT_MUTED, BORDER, ... の全一覧は付属書 S の 4 章 */ }
// kit：pub const PRIMARY: Color = Color::var(&PRIMARY_VAR); fn theme(mode) -> Theme { Theme::light().set(&PRIMARY_VAR, ..).set(&color::ACCENT_VAR, PRIMARY) }
```

- 解決する場所は `compile_style` のまま（O-1 は (a)。theme の切替えは style の effect の再実行になる）。
- 消すもの：`ThemeColor`、`Color::of`、`Color::token`、`ColorToken`、process 全体の `THEMED` 表、`Palette`、`Spacing`、`Radii`、`Typography` の大きさ、`Theme::{colors, space, radius, with_colors, with_space, with_radius, with_text}`、`use_tokens`、kit の `ColorName` と `Tokens` の色・大きさの field、ui の `UiTokens`。`ThemeExtension` は値ではない表（`<Name>Theme` の recipe、`InteractionPolicy`）にだけ使う。
- 部品の token（button の高さなど）も theme 変数にする：`kivact::ui::theme::button::HEIGHT_MD` のように部品の module に置く（UI-03 の「部品の Tokens」を theme 変数に統一する。決定 DEC-06）。
- `Theme` は `Arc<ThemeData>`。`PartialEq` は `Arc::ptr_eq` で先に判定する。`Theme::light()`・`dark()` は static の clone（ST-26）。`Theme::ext` の fallback は theme ごとに一度だけ作る（ST-07）。`Theme::with_fonts`、`Theme::with_icons`。kit の theme の作り直しを避ける `kivact::style::cached_theme(build: fn() -> Theme) -> Theme`（fn pointer を key にする。KIT-08）。
- `ThemeMode` は名前を変えない（`ThemeHint` を消して `ThemeMode` を kivact-types に置く）。
- CSS の名前に揃える（ST-22）：`weight`→`font_weight`、`italic(bool)`→`font_style(FontStyle)`、`radius*`→`border_radius*`、`ring*`→`outline*`（`outline_offset`）、`basis`/`grow`/`shrink`/`wrap`/`direction`→`flex_basis`/`flex_grow`/`flex_shrink`/`flex_wrap`/`flex_direction`、`pressed`→`active`、`Direction`→`FlexDirection`。
- CSS の値の型（`Shadow`、`Filter`、`Transform`、`Paint`、gradient、`Shape`、`CornerRadius`、`BlendMode`、`Mask*`、`BorderImage*`、`PaintShader`、`FilterShader`、`FilterGraph`）は `kivact::style` だけに置く。
- `text_decoration_style(v: TextDecorationStyle)`（`IntoValue` をやめる。ST-12）。
- `Color::components()`・`Color::mix()`（theme なし）を消し、`components(&Theme)`・`mix(other, t, &Theme)` にする。`Color::relative_luminance()`・`Color::contrast_ratio(other)` を足す（ST-03、KIT-17）。

### 5.8 描画（`kivact::draw`）

- `kivact::draw` は描画者の型だけ：`Draw`、`Path`、`PathContour`、`Layer`、`Mix`、`Stroke`、`LineCap`、`LineJoin`、`FillRule`、`TextFormat`、`TextBaseline`、`TextMetrics`、paragraph、`ImageOptions`、`Pixels`、`RegionId`、`Offscreen`。
- `Layer::mix(Mix)`（`enum Mix { Normal, Blend(BlendMode), Composite(Composite) }`）が `blend`・`composite` を置き換える（ST-17）。
- `Pixels` は premultiplied alpha であることを型の doc に書く（AR-10）。
- `Offscreen::render(width: f64, height: f64) -> Result<Pixels>` と `render_svg(width: f64, height: f64) -> Result<String>`、`.scale(f64)`（RC-29）。SVG は SVG 1.1 と CSS に同等のものがある機能（font-feature-settings、font-variation-settings、mix-blend-mode、文字の装飾、塗りと線）は書き出し、同等のものが無い機能だけを失敗にする（ST-15、O-3。決定 DEC-11）。

### 5.9 標準部品（`kivact::ui`）と behavior（`kivact::interaction`）

- `kivact::ui`：部品の関数（`#[component]`）、部品の variant の enum（`ButtonVariant`、`ControlSize` など）、`icon_button(icon, label)`（新設、USR-17）、`Slant`・`Striped`（kit から移す汎用の variant の印、KIT-04）、`SyntaxKind`（KIT-05）、`sidebar`（basic と soft から移す、KIT-10）、`tabs` の overflow（`TabOverflow::{Menu, Scroll}`、KIT-09）。
- `kivact::ui::theme`：部品ごとの `<Name>Theme`（recipe 表、`ThemeExtension`）、`<Name>State`（recipe が読む状態）、部品の theme 変数、`Look<S>`、`Variant`。`UiRecipes`・`UiTokens` と 46 個の look/values/tokens の別ファイルを消す（UI-03、UI-14、API-19）。recipe の上書きは `Fn(&Theme, &Look<S>, Style) -> Style`（3 つ目は前の recipe の結果。UI-24）：

```rust
theme.with(ButtonTheme::default().root(|_theme, _look, base| base.border_radius(0)))
```

- 見えない名前（accessible name）を持つ入力部品（`text_input`、`text_area`、`select`、`combobox`、`slider`、`number_input`、日付・時刻の入力）は、名前を位置引数で取らない：`text_input(value)` と `.label("Name")`。`field("Name", control)` が control に名前を付ける。focus できる control に名前が無いと、mount のときに位置付きの診断を出す（debug build と `kivact::test` では error）。見える label を持つ部品（`button`、`checkbox`、`switch`、`radio_group` の項目）は位置引数の label のまま（API-17。決定 DEC-05）。
- 開閉の語彙を一つにする（UI-12）：interaction の surface の Api（Disclosure、Popup、Dialog、Menu、Select、Combobox の popup、Palette、HoverCard、NavigationMenu）は `use_x(.., open: Signal<bool>)` で open を必須の位置引数に取り、`show()`・`hide()`・`toggle()`（引数なし）と `is_open() -> Value<bool>` を持つ。`trigger()`・`content()` の Part と、`surface(placement, content) -> View`（anchor・`show`・overlay・content の Part の配線を行う。UI-24）を持つ。`use_menu_with_open`、`on_open_change`、`DialogMode::Popover`、`MenuApi::while_open`、`CommandPaletteApi::while_open` を消す。ui の部品の規則は API-13 のまま（閉じていると意味の無い dialog・window・drawer・sheet は `open` を位置引数、それ以外は `.open(signal)` の setter。setter を呼ばなければ部品が局所の signal を作る）。
- 値を編集する部品はすべて `Signal<T>` を位置引数で受ける：`tag_input(tags: Signal<Vec<String>>)`、`stepper(items, current: Signal<usize>)`（UI-13）。
- part の状態を読む唯一の方法は `Part::state()`。`*_state()` の getter 34 個、part の slot、項目・行・cell の状態の map を消す（UI-01）。
- `toaster()`・`announcer()` → `use_toaster()`・`use_announcer()`。`interaction::ViewTransform` → `ZoomPan`。`Snap` は interaction だけ、`ComboboxMode` は ui だけ（API-10、API-19、UI-19）。
- interaction は view を返さない（`surface()` を除く）。ui は入力で動く `impl Machine` を持たない（UI-04。6.3 節で検査）。
- 公開をやめるもの（`pub(crate)`）：`use_hover_corridor`、`use_hover_surface` とその Api、`GridMove`、`DelayedPhase`、`DelayedApi::phase`、`HoverCardOpenedBy`、`DragVelocity`、`filter_commands`。`flatten_tree_rows` は ui の data table（別の crate）が使うので `#[doc(hidden)] pub` にする。
- 部品の module は `crates/kivact-ui/src/<name>.rs`（`#[component]` の関数と variant の enum）と `crates/kivact-ui/src/<name>/look.rs`（`<Name>State`、`<Name>Theme`、既定の recipe、部品の theme 変数）。workspace の clippy が `mod_module_files` を禁止しているので `mod.rs` は使わない。
- `Variant` trait の公開の path は `kivact::ui::theme::Variant` だけ（定義は core に置き、`kivact::view` からは再公開しない）。部品の状態の型（`CheckableState` など）は改名しない。DEC-05 の `.label()` は 12 の入力部品だけで、集まりの名前（`list`、`tabs`、`tree`、`radio_group`、`toggle_group`、`button_group`、`input_group`、`splitter`）は位置引数のまま。`selection_field` を消す（`select(..).trigger(view)` に統合）。`input_group_with` を消す（UI-20、UI-13）。

### 5.10 kit と plot

- 5 つの kit は一つの骨格に揃える（KIT-06。付属書 K の 3 章が骨格を正本として書く）：`theme(mode) -> Theme`、実際の option があるときだけ `ThemeOptions` と `theme_with(mode, options)`、kit の theme 変数（`<kit>::PRIMARY` など。`ColorName` は消す）、`recipes_<group>.rs` に recipe、`variants.rs` に `<Component>Variant`、分野の部品は `src/<domain>/`。kit 名の接頭辞（`GameCellVariant`）を付けない。`*Appearance`・`*Look`（kit の型名）は `*Variant` に改名する。中核に同じ名前の `<Component>Variant`（`ButtonVariant` など）がある場合、kit の追加の見た目は `<Component>Tone` と名付ける（API-10 の名前の衝突を避ける）。中核の部品はすべて `<Name>Theme`（recipe）を持つ。値だけで装飾する部品にも空でない既定の recipe を置き、kit が形を変えられるようにする。
- `kivact-plot`：`kivact_plot::{plot, PlotModel, PlotData, PlotSeries, PlotRange, PlotAxis, AxisScale, TickFormat, LineDash, PlotView}`。色は `kivact_plot::color::{SERIES_1..SERIES_5, GRID, AXIS}` の theme 変数で受け、既定値は中核の `color::*` から導く。kit は自分の色を set できる。basic の `chart` は basic に残し、目盛りと scale の計算を `kivact-plot` と共有する（KIT-10）。

### 5.11 prelude（完全な一覧）

```
App, UiError, component, Prop, bind,
Signal, Value, Var, Callback, Trigger, signal, memo, var, callback, binding, trigger,
effect, effect_on, on_mount, on_cleanup, untrack, provide, use_context,
IntoView, Element, View, ViewFn, Role, Key, KeyEvent, PointerEvent, PointerPhase, MouseButton, WheelEvent,
Handled, CheckedState, NodeRef, node_ref,
node, text, column, row, stack, grid, scroll, spacer, image, icon, canvas, window,
show, show_some, dynamic, for_each, for_index, virtual_list, error_boundary,
Style, style, Color, Theme, ThemeMode, Align, Justify, Position, Overflow, TextAlign, Cursor,
pct, em, rgb, color, space, radius, font, use_theme, provide_theme,
Draw, Path, Stroke, Shadow, Point, Size, Rect,
Resource, Action, AsyncState, resource, action, spawn, sleep,
button, icon_button, text_input, text_area, checkbox, switch, select, radio_group, slider, field,
dialog, menu, tabs, tooltip, list, async_view, toast_region, use_toaster,
ListItem, MenuEntry, Toast, ButtonVariant, ControlSize, message
```

ここに無い名前は正準の module から import する（`kivact::ui::data_table` など）。同じ名前を二つの module から出さない（API-10）。

### 5.12 削除・改名する公開名（AC-4 の検査対象）

作業指示書の最終 checkpoint は、次の名前が `crates/`、`examples/`、`templates/`、`docs/`、`dev/` に識別子として残っていないことを `rg -w` で確かめる（改名の旧名を含む）。

`Binding`、`IntoBinding`、`IntoReactive`、`Reactive`（`custom::Reactive`）、`Notifier`、`notifier`、`ActionOutcome`、`ActionMode`、`Retry`、`AsyncSnapshot`、`constant_value`（`#[doc(hidden)]` の定義と内部の利用だけ許す。付属書 R が許す場所を列挙）、`AppOptions`（公開の path として）、`ThemeColor`、`ColorToken`、`ColorName`、`Palette`、`Spacing`、`Radii`、`UiRecipes`、`UiTokens`、`use_tokens`、`ThemeHint`、`Dimension`、`TextLen`、`GridFlow`、`Insets`、`SpanStyle`（style 側）、`TextExtras`、`DecorationStyleValue`、`prefer_gpu`、`use_menu_with_open`、`on_open_change`、`on_step_press`、`on_draft_commit`、`selection_field`、`input_group_with`、`while_open`、`DialogMode::Popover`、`ViewTransform`（interaction）、`toaster`、`announcer`、`BadgeHue`、`ItemHue`、`StatHue`、`ProgressHue`、`WeekendHue`、`ButtonTone`（game のもの。pop の `ButtonTone` は残す）、`CodeTone`、`CodeColors`、`TokenKind`（soft のもの。DevTools の protocol の `TokenKind` は残す）、`GameInputGroupVariant`、`GameCellVariant`、`PopBubble`、`Composite`（`Layer::composite` として。`Mix::Composite` の中は許す）、旧 style setter（`.weight(`、`.italic(`、`.radius(`、`.ring(`、`.basis(`、`.grow(`、`.shrink(`、`.wrap(`、`.direction(`、`.pressed(`）。

## 6. 内部の規則（architecture.md を置き換える部分）

最終 wave（付属書 G）が `docs/for-kivact-developer/architecture.md` をこの章に合わせて書き直す。それまでは本章が優先する。

### 6.1 状態の規則と検査（S 規則と state-fields の置き換え、TL-04）

- 規則 S-1〜S-9 の内容（三つ以上の状態は enum、排他の事実は variant の payload、同じ事実を二度保存しない、遷移だけが状態を書く、effect で状態を補修しない、未完成の object を作って後から埋めない）は残す。
- **検査の単位を変える**：新しい xtask 検査 `state`（`state-fields` を置き換える）は、(a) `bool`・`Option<_>`・整数の counter・`Cell`/`RefCell`/`Signal`/`Var`/`Atomic*` の field を 2 つ以上持ち、(b) `&mut self` の method を持つか、内部可変の field を constructor の外で書く struct・enum variant を対象にする。(c) 名前が `Api`、`Options`、`Policy`、`Props`、`Report`、`Event`、`Phase`、`Reason`、`Recipes`、`Theme`、`Tokens`、`Builder`、`Key`、`Bits` で終わる型、`State` で終わる型のうち recipe の状態（kivact-ui と 5 つの kit の中だけ。engine の `PointerState` などは対象のまま）と、`Hash + Eq` を derive して map の key だけに使う型は対象外。対象の型は **struct 単位の印一つ** `// xtask-allow(state): <組合せの閉じ方を 1 行>` が無ければ error。
- constructor・`Default`・値渡しの builder（`fn x(mut self, ..) -> Self`）だけで書く field は状態ではない。印も検査も要らない。
- 関数の中の状態の local（`signal(false)` と `signal(None)` の組など）の検査（`locals.rs`）は残す。
- `independent:` の理由文の重複検査、`--report-audit`、`state-audit.md` を消す。

### 6.2 性能とメモリ（PERF・MEM の追加と変更）

PERF-1〜PERF-5、MEM-1〜MEM-8 は残し、次を加える。

- PERF-6：frame の描画は、描き直した node だけを lower し直す（per-node の draw cache、EN-03）。paint list の並べ直し（restack）は重なりの事実（stacking context、z、layer の事実、子の組の変化）が変わったときだけ（EN-04）。
- PERF-7：Arvane は engine が渡す layer ごとの `content_version` で変化を知る。command の hash は変化した chunk だけ（AR-03）。debug build では hash による照合で版の上げ忘れを検出する。
- PERF-8：GPU の device は process（Driver）に一つ。window ごとに持つのは frame の状態だけ（AR-02）。
- PERF-9：診断の記録（turn の trace、原因の文字列、node の provenance）は、DevTools・`kivact::test` の trace・debug の trace sink が有効にしたときだけ行う。release で常に持つのは counter だけ（RC-05、RC-06、RC-22）。
- PERF-10：起動の critical path（main から最初の present まで）で、system font の走査・DevTools agent の起動・asset worker の起動・使わない GPU pipeline の compile をしない（AR-12、AR-13、RC-10）。
- PERF-11：項目の多い collection（list、combobox、menu、select、tabs、tree、table、palette）は、collection ごとに一つの key の索引（`Index<K>`）と `selector` を使い、項目あたりの memo は最大 2 つ（UI-02）。例外：menu の項目は最大 5 つ（press-drag の間の pointer の移動ごとに、全行の effect の再実行と submenu の timer の再開を起こさないため。付属書 U の U03）。
- PERF-1 の Metadata の例外は残す。ただし Metadata の経路で semantic order を二度数えない（EN-10 の S の部分）。
- 遅延初期化の道具は一つ（DEC-13）：`clippy.toml` は `OnceCell`・`OnceLock`・`LazyCell`・`LazyLock` を禁止したままにする。使う時に作る重い資源（PERF-2、PERF-10：ICU の formatter、GPU の pipeline、system font、既定の theme など）は、`kivact_types::cell::Deferred<T>`（UI thread の中、`!Sync`）と `kivact_types::cell::SharedDeferred<T>`（`static` と thread をまたぐもの、`Sync`）だけで持つ。どちらも `const fn new()`、`get_or_init(&self, f: impl FnOnce() -> T) -> &T`、`get(&self) -> Option<&T>` を持つ。std の型を使うのはこの二つの実装の中だけ（`#[expect(clippy::disallowed_types, reason = "...")]` を各 1 か所）。状態の補修や「未完成の object を後で埋める」（S-7）には使わない。付属書 S の unit が W1a で作る。

### 6.3 crate・module の規則

- B-1〜B-5 は残す。`visible_to` は「dev-dependency は対象外」を全体の既定にする（TL-06）。
- interaction は `View` を返さない（`surface()` を除く）。ui は入力 event で動く `impl Machine` と、signal を書く `effect` を持たない。xtask の `module-rules` か新しい検査で確かめる（付属書 T が実装）。
- `impl Element for` / `impl IntoView for` を手で書けるのは kivact-core の `view` module だけ（5.1 節）。xtask で確かめる。
- `machine(()` と `type State = ()` を禁止する（UI-09）。
- 公開 API の 17 規則（API-1〜API-17）は残し、本書 5 章と矛盾する部分（API-13 の入力部品の label、API-3 の handler の形）は 5 章で置き換える。`structural-options` の検査は消す（TL-06）。

### 6.4 試験と gate

- 試験は App 全体の E2E のまま（実 window、`kivact::test`）。過去の不具合の再現だけの試験、画素や時間の厳密な比較、golden は足さない。
- 公開 API の E2E は 4 つの test binary に分ける：`e2e_core`（`kivact` だけ）、`e2e_ui`（ui・apg・interaction。`kivact` だけ）、`e2e_kits`（theme_conformance と kit の crate が要るもの。別 package `kivact-kit-tests`）、`e2e_native`（embed、GPU、foreign window、DevTools、GPU 共有）（TL-15。kit を分けるのは DEC-14 のため）。
- docs の doctest は英語の木だけを compile する。日本語の Rust の code block の列が英語と同じであることを xtask の `docs-pairs` 検査で確かめる（TL-03）。
- gate は 3 段（tier a：unit ごと、tier b：wave ごと、tier c：checkpoint）。正確なコマンドは作業指示書 0.6 節。

## 7. 監査所見の判定

ID は `rework/audit/` の報告のもの。「採用」は報告の fix をそのまま実装する。「変更して採用」は右の列のとおりに変える。「不採用」は実装しない。付属書は「採用」と「変更して採用」の全所見を、どれかの unit に割り当てる。

### 7.1 公開 API（`api.md`）

採用：API-01（5.1 節の規則で）、API-02、API-03、API-04、API-05（5.11 節の一覧で）、API-06、API-07、API-08、API-09、API-10（`embed::Visibility` は `EmbedVisibility`）、API-11、API-12、API-13、API-14、API-15、API-16、API-18、API-19（`kivact::ui::theme` に置く。`recipe` ではない）。

変更して採用：API-17（5.9 節の DEC-05）。

open question：Q1 は `Value<T>` の名前を変えない。Q2 は (b)。Q3 は必須（ui・kit も `#[component]`）。Q4 は `action(f)` と `queued_action(f)`。

### 7.2 runtime・core・macro・i18n（`runtime-core.md`）

採用：RC-01〜RC-31、RC-33〜RC-41（RC-41 の小項目もすべて）。

変更して採用：RC-32（`IntoText` は残し prelude から外す。`IntoReactive`・`Reactive`・`IntoValue<Rc<str>>` は消す。`Prop<T>` を `#[doc(hidden)]` で公開）。

open question：Q1 は release では counter だけ。Q2（DevTools を feature で compile から外す DEP-4 の例外）は不採用（実行時の `Option` の hook で費用を消す）。Q3 は採用（engine の変更記録。記録中だけ）。Q4 は `Action::close` を消す。Q5 は `use_context -> T`。Q6 は window ごと。

### 7.3 engine・layout・editing（`engine.md`）

採用：EN-01〜EN-25、EN-27〜EN-32。

付属書 E の判断（採用）：EN-30 は実装しない（`Client` に既定値が無く、`Launch::Starting` を消すと仮の値が要る）。EN-11 の Tab の押下ごとの O(N) の列挙は残す（人の操作の速さでしか起きない）。EN-12 の折り返す editor の splice は後続。EN-23 の kivact-layout の `unrounded` は残す（丸めの増分化が pass をまたいで祖先の値を読む）。

変更して採用：EN-26（`PointerSource` の写しだけを消す。`Admission` は engine に残し、architecture.md に「engine が持つ配送の受理」と書く）。EN-10（S の部分だけ：semantic order を一度だけ数え、`clipping_of_all` は attached な node だけを回る。Metadata の増分化は後続）。

open question：Q1（`lower` の crate 化）は不採用。Q2 は App 全体の clock 一つ。Q3 は 6.2 節のとおり例外を残す。Q4 は採用（draw cache は描画中の node だけ）。

### 7.4 style・types・shader・svg（`style-types.md`）

採用：ST-01（型名は `ThemeVar`。DEC-02）、ST-02（ST-01 で消える）、ST-03、ST-04、ST-05、ST-06、ST-07、ST-08、ST-09、ST-11、ST-12、ST-13、ST-14、ST-15、ST-16、ST-17、ST-18、ST-19、ST-21、ST-22、ST-23、ST-25、ST-26、ST-27、ST-28、ST-29、ST-30、ST-31。

変更して採用：ST-10（同一の組だけ統合：`Length`、`TextLength`、`GridAutoFlow`、`Edges<f64>`、`ThemeMode`、`AvailableSpace`。Color と Rgba の組は「作者の値」と「解決した値」という別の事実なので残す）。ST-20（`arvane-protocol` の新設と `prop_interp` の移動は採用。`kivact-cell` は作らない）。

不採用：ST-24（色を f32 にする。色の計算と試験の期待値への影響が大きく、得る K2 が小さい）。

open question：O-1 は (a)。O-2 は採用。O-3 は 5.8 節（DEC-11）。O-4 は採用。O-5 は CSS の名前に揃える。

### 7.5 interaction・ui（`interaction-ui.md`）

採用：UI-01〜UI-25（UI-03 と UI-14 は 5.7・5.9 節のとおり、部品の Tokens を theme 変数にし、recipe を `<Name>Theme` にする）。

open question：Q1 は分割しない（strategy.md の「標準部品」の行を「kit なしで実用画面を作れる数と、二つ以上の kit が共有する部品」に直す。付属書 G）。Q2 は theme 変数に統一（DEC-06）。Q3 は `Element::variant`。Q4 は 5.9 節。

### 7.6 kit（`kits.md`）

採用：KIT-01、KIT-02、KIT-03、KIT-04、KIT-05（中核は `SyntaxKind` と theme 変数の色だけを持つ。Rust の tokenizer は basic に残す）、KIT-06、KIT-08（`IconSet::shared` と `icons!`、theme は `cached_theme`）、KIT-09（soft の `tab_list` の overflow は中核の `tabs` へ）、KIT-10、KIT-11、KIT-12（catalog の文言は英語）、KIT-13、KIT-14、KIT-15、KIT-16（日本語の docs は全ページ残す。migration 表と recipe の一覧は消す）、KIT-17（時計は `task::clock`）、KIT-18、KIT-19、KIT-20。

変更して採用：KIT-07（`theme_colors!` macro ではなく theme 変数で `ColorName` ごと消す）。

open question：Q1 は soft に残す（使われていない関数だけ消す）。Q3 は `kivact-plot`。Q4 は英語。Q5 は両言語。Q6 は採用。

### 7.7 Arvane（`arvane.md`）

採用：AR-01〜AR-07、AR-09〜AR-26。

変更して採用：AR-17（arvane-text へ移すのは family の解決だけ。`glyph_outline`・`glyph_ink_bounds` は arvane-render、`load_font` は arvane-asset に残す。移すと render → text の辺と text ↔ asset の循環ができるため）。AR-08（embed の表示・非表示は `ShellEvent::Visibility(bool)` にするが、`Suspended`・`Resumed`・`SurfaceDestroyed`・`SurfaceCreated` などモバイルの event は残す。strategy.md は「モバイルと移植の設計境界は P3 へ先送りしない」）。

open question：Q1 は採用。Q2（stem darkening の bucket）は変えない（作者が #16 で決めた見え方）。Q3 は採用（`<filter>` で未対応の primitive は filter なしで描き、Unsupported を一度報告）。Q4 は採用（debug で hash と照合）。Q5 は不採用（AR-08 と同じ理由）。

### 7.8 tooling（`tooling.md`）

採用：TL-01〜TL-10、TL-12〜TL-17。

不採用：TL-11（Windows の CI の追加。費用の判断は作者が行う）。

open question：Q1 は Python を消す。Q2 は struct 単位の規則。Q3 は git から外すだけ（公開は後続）。Q4 は不採用。Q5 は採用。

### 7.9 利用者（非公開のため報告は commit しない）

採用：CAP の一覧（3 章）、`remote`/`remote_trigger`（USR-02）、`Trigger`（USR-03）、prelude の追加（USR-04）、`commit_focused_edit`（USR-05）、`#[component]` で手書き builder をなくす（USR-09）、font を theme に（USR-12）、embed の cfg と通知（USR-15）、`target_size`（USR-16）、`icon_button` と `spacer`（USR-17）、profile の案内と移行ガイド（USR-01、USR-13）。

不採用：drop target（USR-08 の Q2）、combobox の trigger 文字（USR-18）、audio control の kit の統一（USR-11。作者の判断）。

## 8. 設計者の決定（作者が覆してよい）

| ID | 問い | 決定 | 理由 |
|---|---|---|---|
| DEC-01 | 未決の問いを作者に聞いてから進めるか | 聞かずに本書で決める。作者は PR の review で覆せる | 実装者は設計の判断ができないため、全部を前もって決める必要がある |
| DEC-02 | theme 変数の型名 | `ThemeVar<T>` | 状態の `Var<T>`（useRef）と衝突させない |
| DEC-03 | ui・kit の部品も `#[component]` にするか | する（169 の手書き builder を移す） | 一つの書き方。利用者の部品 library も同じ形で書ける |
| DEC-04 | `Signal` と `Binding` | `Signal` に統合 | 書ける handle を一つにする |
| DEC-05 | 入力部品の見えない名前 | 位置引数をやめ `.label()` と `field()`。名前の欠落は実行時の診断 | JS の `<label>` と `<input>` の感覚。a11y の保証は compile 時から実行時と試験に移る |
| DEC-06 | 部品の token | theme 変数に統一し、`UiTokens` と kit の `Tokens` の値の field を消す。recipe（形）は `<Name>Theme` | 値の仕組みと形の仕組みを一つずつにする |
| DEC-07 | style の名前 | CSS の名前（`font_weight`、`border_radius`、`flex_grow`、`outline`、`active`） | CSS の利用者が推測できる |
| DEC-08 | GPU の device | process に一つ | 起動とメモリが window の数に比例しない |
| DEC-09 | SVG の asset | usvg で `RenderScene` にし、vello で device の大きさに描く（engine の画素の API は変えない）。resvg の描画・tiny-skia・fontdb の system font の走査を消す。fontdb と rustybuzz の crate は usvg の文字の処理が使うので残し、font は arvane-text から渡す | 2 つ目の rasterizer と system font の 2 度目の走査を持たない |
| DEC-10 | Python の DevTools client | 消す | 利用者がいない。MCP が同じ役目 |
| DEC-11 | SVG の書き出しで同等のものが無い機能 | SVG 1.1 と CSS に同等があれば書き、無いものだけ失敗にする | plot の書き出し（`tnum` など）が失敗しない |
| DEC-12 | 性能の必達 | 全行で後退しない（≤ 1.00、雑音の許容つき）。改善の目標は記録 | 未達で設計が止まらず、後退は確実に防ぐ |
| DEC-13 | 遅延初期化 | `Deferred<T>`・`SharedDeferred<T>` だけ。std の lazy 型の禁止は続ける | 性能の所見が求める「使う時に作る」と、S-7（未完成の object を作らない）の区別を一つの道具で表す |
| DEC-16 | payload のある handler の型 | `impl Fn(E) -> R`。`IntoHandler` は payload の無い handler だけ | closure の引数の型の推論は `Fn` の bound からしか働かない。`Callback<E>` は `Fn` を実装できない |
| DEC-15 | `update` の戻り値 | 返さない（`Signal`・`Var` とも） | 監査の提案（`Option<R>`）では、最も多い `on_press(move || x.update(..))` が handler の結果の型に合わなくなる |
| DEC-14 | W6〜W10 の kit | W6 の style の掃引から W9〜W10 の kit の移行まで、5 つの kit・その catalog・kit の試験・`e2e_kits` を compile の対象から外す | kit は最終の API に対して一度で移行する方が、途中の API への追従を二度するより安い |

## 9. 作業の段階（付属書の unit が従う枠）

詳細な手順は作業指示書。下の表は最初の枠である。付属書の間の所有ファイルの重なりと lane の上限を解いた**確定の配置は作業指示書 4 章**（W1a・W1b・W1c・W2a・W2b・W3a・W3b・W4・W4b・CP1・W5a・W5b・W6a・W6b・W7a・W7b・W8・CP2・W9・W10・W11a・W11b・W11c・CP3）で、本表と食い違うときは作業指示書が正しい。

| wave | 内容 | lane（最大 6） | 排他の規則 |
|---|---|---|---|
| W0 | 準備・B0 の計測・試験の名前の基準（Orchestrator） | — | — |
| W1 | 掃除：xtask の state 規則の置換と印の削除、`architecture/` の除外、e2e の分割、docs の英語だけ、window-lock、check-features、DevTools client の統合と Python の削除、kivact-test の自己試験の移動（付属書 T） | T1〜T6（crate の群ごとに印を消す） | 印の削除は crate の群で lane を分ける。同じファイルを 2 つの lane が持たない |
| W2 | `arvane-protocol` の新設（types・engine・arvane の `use` の書換え）を lane A が単独で。並行して runtime・core・style（types 以外）・interaction・kit の内部改善（公開 API を変えないもの） | A：protocol。R、C、S、U、K：内部 | protocol の unit が kivact-types・engine・arvane-* を所有する |
| W3〜W4 | engine の性能（E）、Arvane の性能（A）、runtime・core・style・interaction・kit の内部の続き | E、A、R、C、S、U | crate の群ごとに lane。公開 API を変えない |
| CP1 | gate（tier c）と KPI（Orchestrator） | — | — |
| W5 | 公開 API の掃引 1：状態・handler・context・macro・App（付属書 R と C の API unit）。runtime → core → interaction → ui → kit・plot・test・例・docs の順の鎖 | 鎖は 1 lane。E と A は自分の crate で並行 | 鎖の unit がその間 runtime〜docs を所有する |
| W6 | 公開 API の掃引 2：style（theme 変数、宣言の表、状態の style、CSS の名前、module の移動）。style → core → ui → kit の順の鎖 | 鎖は 1 lane。E と A は並行 | 同上 |
| W7〜W8 | ui の作り直し（部品ごとの module・`<Name>Theme`・theme 変数・`#[component]`・開閉の語彙・machine の移動） | U1〜U4（部品の群で分ける）、E、A の残り | 部品の directory は lane ごとに別。`lib.rs` の `mod` 行の conflict は両方残す |
| CP2 | gate と KPI | — | — |
| W9〜W10 | kit の移行（5 つの kit を骨格に揃え `#[component]` と theme 変数へ）、`kivact-plot`、`catalog-shell`、sidebar・tabs overflow の中核への移動 | K1〜K5（kit ごと）、K6（plot・catalog-shell） | kit ごとに lane |
| W11 | 文書：利用者向け docs（英日）、README、例、雛形、移行ガイド、architecture.md・decisions.md・strategy.md、残りの状態の検査 0 件（付属書 G） | G1〜G4 | 文書の directory で分ける |
| CP3 | 最終：full gate、KPI、AC-1〜AC-5 | — | — |

unit の書き方（全付属書で共通。付属書の書き手が守る）：

1. unit は一つの subagent が一度で終えられる大きさ（目安：手で書くコードは 600 行まで。機械的な置換は行数を問わないが、付属書が置換のコマンド（`sed`・`perl`・`rg`）を一字一句書く）。
2. 各 unit は「wave と lane」「所有するファイル・directory」「前提の unit」「手順（番号付き。変更後の型・関数の署名とコードを書く）」「消すもの」「確認のコマンド（tier a）」「完了条件」を持つ。
3. 型・関数の署名を変える unit は、同じ unit の中でその呼出し元を全部直す。呼出し元が別の lane の所有なら、その unit は掃引の鎖（W5・W6）に置く。
4. 新しく作る状態の型は、6.1 節の検査に対する閉じ方（enum にする・印の 1 行の文面）まで付属書が決めて書く。
5. 試験：付属書が名前を挙げた E2E だけを足す・直す。期待値を緩めない。消す試験は名前を列挙する。
6. 設計の判断を実装者に残さない。「検討する」「必要なら」と書かない。

## 10. 付属書の一覧

| 付属書 | 範囲 | 主な所見 |
|---|---|---|
| [rework/T-tooling.md](rework/T-tooling.md) | xtask、gate、試験の構成、DevTools の crate、Python、architecture、CONTRIBUTING | TL-*、6.1・6.3・6.4 節 |
| [rework/R-runtime.md](rework/R-runtime.md) | kivact-runtime、kivact-i18n と状態・handler・非同期の公開 API（5.2・5.3・5.5 節） | RC-01〜RC-08、RC-14〜RC-17、RC-23〜RC-26、RC-30〜RC-35、RC-37〜RC-39、API-02、API-03、API-06、API-15、USR-02、USR-03 |
| [rework/C-core.md](rework/C-core.md) | kivact-core、kivact-macros、facade、kivact-offscreen、kivact-test の API の追従、App・platform・embed の公開 API（5.1・5.4・5.6・5.11 節） | RC-09〜RC-13、RC-18〜RC-22、RC-27〜RC-29、RC-36、RC-40、RC-41、API-01、API-04、API-05、API-07〜API-11、API-13、API-14、API-16、API-18、USR-05、USR-12、USR-15〜USR-17 |
| [rework/E-engine.md](rework/E-engine.md) | kivact-engine、kivact-layout、kivact-editing | EN-* |
| [rework/S-style.md](rework/S-style.md) | kivact-types、arvane-protocol の新設、kivact-style、kivact-shader、kivact-svg（5.7・5.8 節） | ST-* |
| [rework/U-ui.md](rework/U-ui.md) | kivact-interaction、kivact-ui（5.9 節） | UI-*、API-19 |
| [rework/K-kits.md](rework/K-kits.md) | 5 つの kit、catalog、kivact-plot、catalog-shell（5.10 節） | KIT-* |
| [rework/A-arvane.md](rework/A-arvane.md) | arvane*、kivact-gpu-share | AR-* |
| [rework/G-guides.md](rework/G-guides.md) | 利用者向け docs、README、例、雛形、移行ガイド、開発者向けの 3 文書、bench の probe の最終確認 | API-07〜API-09、AC-5 |

## 11. 後続（本改修では行わない）

- a11y の Metadata の経路の増分化（EN-10 の M の部分）。
- 折り返す editor の行の splice（EN-12 の一部）。
- `lower`・`text_input` の crate 化（狭い API ができたら）。
- 色の f32 化（ST-24）。
- drop target、combobox の trigger 文字、audio control の kit の統一（作者の判断）。
- Windows の CI、`architecture/` の図の公開（CI の artifact や Pages）。
