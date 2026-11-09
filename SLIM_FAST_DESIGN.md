# Kivact Slim-Fast 設計書

版 1.1（2026-11-09、`20261109_2107_CODEX_REVIEW.md` の妥当な指摘を反映）。作業ブランチ `refactor/kashiwade/slim-fast`、基点 `main@31a7ebe`。

手順は [SLIM_FAST_WORK_ORDER.md](SLIM_FAST_WORK_ORDER.md)（作業指示書）に書く。本書は「何を、なぜ、どの形にするか」を決める。実装者は本書の決定を変えない。本書に無い判断が必要になったら作業指示書 0.4 の「停止して報告」に従う。

## 0. 本書の扱い

### 0.1 優先順位

1. 本書（Slim-Fast 設計書）
2. `docs/for-kivact-developer/architecture.md`、`decisions.md`、`strategy.md`

衝突したら本書が優先する。最終 wave（W7）で、本書の決定を architecture.md と decisions.md に書き戻し、本書と作業指示書を削除する。

### 0.2 変わらない前提

- 互換は残さない（wrapper、alias、旧名、二重 mode、deprecated を作らない）。in-repo のすべての呼び出し元（kit、catalog、例、試験、文書）を同じ変更で直す。
- CI の機能は追加しない（`dev/bench` は手動の道具であり CI に組み込まない）。
- 状態機械の検証は E2E で行う（Q-01）。window を開かない単体試験は追加しない。
- 既存の作者の決定（decisions.md の Q-01〜Q-08、APG-01〜04、SCOPE-01〜07、ADR-01〜52、CONTRACT-*）は変えない。本書が明示的に置き換えるものだけを置き換える（8.1 節）。
- kit は縮小しない（外部の利用者がいる）。

### 0.3 用語

| 用語 | 意味 |
|---|---|
| B0 | 変更前（本ブランチの W0 時点）に実装機で取った KPI。倍率の分母 |
| B1 | 最終 wave 後に同じ実装機で取った KPI |
| O(N) | window の全ノード数に比例する処理 |
| O(changed) | 今回変わったノード数（とその祖先の深さ）に比例する処理 |
| item | 計測の 1 行（`kv_stress` の 1 項目） |
| 既定 renderer | 何も指定しない App の描画経路 |

### 0.4 版 1.1 の変更（`20261109_2107_CODEX_REVIEW.md` の扱い）

指摘はすべてコードで確かめ、実装方針（公開 API の書きやすさと統一、内部の単純さ、保守性、strategy.md、境界の明確な crate 分割、暗黙の状態機械の解消）に照らして採否を決めた。

| 指摘 | 扱い | 本書での反映 |
|---|---|---|
| R01 Style の履歴の再生が merge で指数的 | 採用 | MEM-1 を群ごとの疎な struct に変えた（履歴を持たない） |
| R02 LAY-1/PNT-1 は完全な O(changed) ではない | 一部採用 | 計算量の説明を実態に合わせ（1.4、2.2、4.4、4.5 節）、同じ値の印付けと facts の全走査をやめた。アルゴリズム自体は維持（親の子の比較は Taffy の layout と同じ大きさで安価） |
| R03 flat cache の削除済み entry の残留 | 採用 | `remove_with` で個別に消す（4.5 節） |
| R04 display:contents の印の混入 | 採用 | 印の正規化（4.5 節） |
| R05 OnDemand と GPU 共有の循環 | 採用 | `HostRequest::DemandGpu` で需要を明示（4.3 節） |
| R06 埋込み SVG の文字 | 採用 | `has_text` が `svg+xml` の data URL も見る（4.1 節）、E2E を追加 |
| R07 children の再 mount の契約 | 採用 | `#[component]` に `ViewFn` の prop を足した（5.2 節） |
| R08〜R11 付録 A の誤った enum 化・偽の独立 | 採用 | 全行の書き手と到達する組合せを調べ直し、付録 A を確定した型で書き直した（「確かめる」を残さない） |
| R10 の AsyncCell | 一部採用 | ADR-21 の通知の粒度を保ち、単一の書き手 `publish` の投影（SM-12）として明示した |
| R12 lint だけでは「全て」を判定できない | 採用 | 監査を主にし（6.1、6.4 節）、閉じ方を 4 種類に限定、SL-8 で理由の書式を検査 |
| R13 pipeline の遅延生成が初回操作を止める | 採用 | GPU-2 を撤回（4.3 節） |
| R14 wave の依存の不整合 | 採用 | DevTools を W5 の U51 へ、A-EC を W6 へ置いたまま移行期の gate を定義、U45 から facade の check を外した |
| R15 complex-scripts の転送漏れ | 採用 | consumer に転送を足し（4.8 節）、`run.sh` は package を一つずつ build |
| R16 workspace の no-default は GPU なしの確認にならない | 採用 | package ごとの feature の組合せの確認（7 章） |
| R17、R18 計測が全行を出さない・失敗が良い値に化ける | 採用 | `run.sh` を作り直した（1.1 節。INVALID、小数の CPU、mount 後の heap、item あたりの WS 差分） |
| R19 未達でも完了できる手順 | 採用 | 受入れの条件（1.6 節）。満たさなければ W7 で止める |
| R20 macro が `'static` を要求する | 採用 | owner の要らない変換を呼出し時に行う（5.2 節） |
| R21 `Binding::update` の意味 | 採用 | Signal 由来と `binding(get, set)` 由来の意味を定義（5.4 節） |
| R22 Lightbox・SM-9・GPU 設定の優先順位 | 採用 | Lightbox は builder を変えず投影として扱う、SM-9 の初期値と書き戻し、優先順位の表（4.3 節） |
| R23 computed text の共有が深さの二乗 | 採用 | 既に cache にある親とだけ比べる（MEM-5） |
| R24 一時の Frame の縮小・EventSink の揺れ | 採用 | Frame を対象外にし、M-R4 に履歴の閾値を入れた |
| R25 撤退規則が chunk を不透明とみなす | 採用 | 撤退は高速経路を全部止める |
| R26 独立したアプリへ profile が届かない | 採用 | 雛形に profile、ビルド時間を記録（4.8 節） |
| 「維持すべき判断」（crate の分割、改名、ARG-1、既定値を保持しない、B0/B1 の比較、a11y の Geometry 経路） | 維持 | — |

---

## 1. KPI：現状の分析と改善目標

### 1.1 計測の定義

計測は新設の `dev/bench`（crate `kivact-bench`）で行う。`dev/bench/run.sh <label>` が K1〜K6 を一つの Markdown に書く。実装は別の機械で行うため、**目標はすべて同じ機械で取った B0 に対する倍率**で判定する。

| ID | KPI | 計測（`dev/bench`） | 判定に使う値 |
|---|---|---|---|
| K1 | 起動時間 | `kv_startup`（counter と同じ画面、800×600）：`T main`（`main` の先頭）から `KIVACT_FRAME_LOG` の最初の `F` 行（最初の present の完了）まで、5 回の中央値。basic catalog：起動から最初の `F` 行まで（catalog は `T main` を出さないので OS のプロセス生成を含む）、5 回の中央値 | ms |
| K2 | メモリ | アイドル 3 秒後の Working Set と private bytes（counter と同じ画面、basic catalog、3000 行の静止画面）。item あたり＝`kv_stress STILL=1 MODE=<種類>` の (WS(3000 個) − WS(0 個)) ÷ 3000（CPU） | KiB、KiB/item |
| K3 | CPU | アイドル 10 秒の CPU 時間（小数の ms/s と元の tick）とアイドル中の frame 数。`kv_stress`（毎 frame 1 つの text を書き換える）の CPU 使用率（小数の %） | ms/s、1 core に対する % |
| K4 | フレーム時間 | `kv_stress` の `KIVACT_FRAME_LOG`：frame の UI thread 時間（work）の p50/p95/p99、描画間隔の p99。CPU の全画面 frame＝`kv_startup`（CPU 強制）の最初の frame の raster 時間 | ms |
| K5 | UI 更新レイテンシ | `kv_latency`：試験 driver（CPU、`KIVACT_TEST_MIRROR=off`、a11y 公開あり）で、座標 click・pointer 移動・wheel の入力から描画と a11y 公開の完了まで | ms（p50/p95） |
| K6 | バイナリサイズ | release の実行ファイル：`kivact-example-counter`（既定 feature）、`kv_startup`（`--no-default-features`、U11 の後は `complex-scripts` なし）、`kivact-ui-basic-catalog`（既定 feature）。package は 1 つずつ build する（feature の統合で他の package の feature が混ざらないため） | byte |

`kv_heap`（割当て元ごとの生存 heap byte、0 個と 3000 個の差）は原因の診断用で、KPI の行には使わない（WS と allocator の生存 byte は別の量）。

注意：

- `spawn→main`（OS のプロセス生成）は K1 の counter に含めない。この機械では Hello World でも bash 経由で約 52 ms かかり、Kivact の責任ではない。
- Windows の計測は `wmic`（無ければ PowerShell の CIM）を image 名で引く。MSYS の `/proc/<pid>/winpid` は別プロセスを指すことがあり、CPU 時間を誤る（分析中に実際に誤った）。
- K5 の試験 driver は毎 settle で a11y の木を公開する。したがって K5 は「支援技術が有効なときの更新」に近い値になる。支援技術なしの本番経路の更新コストは K4 の work が表す。
- **有効な計測の条件**：`run.sh` は build の失敗、frame の出ない probe、`T main` の欠落、5 秒で 100 frame 未満の `kv_stress`、17 個の場面を出さない `kv_latency`、heap の失敗のどれかで `INVALID:` を書いて非 0 で終わる。INVALID の report は B0 にも B1 にも使わない。B0 は 1.4 節のすべての行を計算できる `run.sh` で取る（項目の欠けた B0 と比べない）。

### 1.2 ベースライン（分析機での実測）

分析機：Windows 11 Enterprise、Intel Core i9-14900K、32 GB、Intel UHD Graphics 770（wgpu は Vulkan を選んだ）、Rust 1.98.1。`main@31a7ebe` に release ビルドの不具合修正（1.3 節 R-0）だけを当てた release ビルド。

`dev/bench/results/analysis-machine.md` は、完成した `run.sh`（1.1 節のすべての行を出す版）を分析機で通しで実行した結果（参考値。下の表は分析中の個別の計測）。両者はおおむね一致する。ただしその実行の間、OS が window の表示を約 32 fps に間引いた（個別の実行では 64 fps）。frame の work 時間は有効だが、fps と描画間隔は使えない。`run.sh` は表示周期に対して fps が 8 割未満なら WARNING を書く。B0 と B1 は、他の描画を止め、画面を点けたまま同じ条件で取る。

#### K1 起動

| 画面 | 値 |
|---|---|
| counter（既定の window、1,555,200 px） spawn→初回 present | 約 490 ms（GPU 既定）、約 485 ms（CPU） |
| 内訳（main 以降） | event loop と window 作成 23 ms ／ **`AssetStore::new` 282 ms** ／ mount と layout 3〜13 ms ／ **初回 frame 100 ms（うち CPU raster 99 ms）** |
| `kv_startup` 800×600 の初回 frame | 72 ms（raster 69 ms） |
| 初回 raster の画素数依存 | 200×100：3.7 ms、800×600：69 ms、1600×1000：226 ms（約 63 ns/px） |
| basic catalog spawn→初回 present | 676〜896 ms（初回 frame 210 ms） |

#### K2 メモリ（アイドル）

| 画面 | Working Set / private（MiB） |
|---|---|
| counter（既定 = GPU） | 196 / 141 |
| counter（CPU） | 47 / 30 |
| basic catalog（GPU / CPU） | 426 / 294、176 / 117 |
| 3000 行（text + button、GPU / CPU） | 611 / 641、472 / 539 |

item あたり（CPU、3000 個、WS の差分）：スタイル付きの空 box **約 30 KB**、text 約 12 KB、button **約 117 KB**、checkbox 約 78 KB、text+button の行 約 141 KB。

`kv_heap` による割当て元（item あたりの生存 byte）：

| 対象 | 主な割当て元 |
|---|---|
| 空 box | core `view::mount::resolve::resolve` 7.9 KB、engine `set_states`（`StateStyles`）5.9 KB、layout 列の増設 2.7 KB、authored `Style` の closure 1.9 KB、`LayoutProps` 1.4 KB、`PaintProps` 0.7 KB、`ComputedTextStyle` 0.47 KB |
| button | `StateStyles` 17.7 KB、`resolve` 15.8 KB、reactive arena 13.6 KB（button 1 個に reactive node 約 40：source 16、`memo<bool>` 11、effect 約 8）、layout 列 10.9 KB、`LayoutProps` 6.5 KB、`Style` 5.7 KB |

内部の型の大きさ：`Style` 1,928 B、`StateStyles` 6,024 B、`PaintDelta` 752 B、`LayoutProps` 736 B、`PaintProps` 704 B、`TextStyle` 488 B、`HostNode` 464 B、`ComputedTextStyle` 464 B、`Semantics` 416 B、`taffy::Style` 552 B、`taffy::Cache` 384 B、`Node`（builder）1,064 B、`Common` 944 B、`Color` 40 B。

GPU を使うと +150〜250 MiB（この iGPU の Vulkan ドライバ）。DX12 を強制すると 287 MiB、GPU 初期化 1.7 s でさらに悪い。

#### K3 CPU

| 状況 | 値 |
|---|---|
| アイドル（counter、両 renderer） | 0 frame、約 0 ms/s（既に良い） |
| 毎 frame 1 text 更新、0 行 | 1 core の約 2.8% |
| 毎 frame 1 text 更新、3000 行 | 1 core の 22〜25% |

#### K4 フレーム時間（`kv_stress`、60 Hz）

| 行数 | work p50 / p95 / p99（ms） | 内訳 |
|---|---|---|
| 0（GPU） | 0.8〜1.0 / 2.0〜2.5 / 2.6〜3.2 | |
| 1000（GPU） | 3.1 / 4.6 / 5.6 | scene 2.2 |
| 3000（GPU） | 7.9〜8.2 / 8.9〜9.5 / 9.5〜10 | **scene 7.1**、raster 1.0 |
| 3000（CPU） | 8.1〜8.6 / 9.6〜10.5 / 10.1〜11.4 | scene 6.4、raster 2.1 |

描画間隔の p99 は 18.4〜19.5 ms（16.7 ms の vsync）。damage は約 8,000 px だけで、raster は部分描画できている。**画面に見えているのは 30 行程度なのに、1 つの text の更新が全 3000 行に比例して遅い。**

#### K5 UI 更新レイテンシ（試験 driver、CPU、a11y 公開あり）

| 操作 | n=100 | n=1000 | n=3000 |
|---|---|---|---|
| 無関係な 1 text の更新（click） | 3.4 ms | 20 ms | **50〜72 ms** |
| 1 行の label 更新（`for_each`） | 10.9 ms | 29.6 ms | 83〜113 ms |
| 2 行の入替え | 4.1 ms | 23.7 ms | 86 ms |
| pointer 移動 p50 / p95 | 0.3 / 2.9 ms | 3.8 / 18.3 ms | 14〜17 / **69〜95 ms** |
| wheel | 0.24 ms | 3.0 ms | 13〜16 ms |
| mount | 318 ms | 574 ms | 1,040〜1,140 ms |

counter の click は約 1.6 ms（小さい画面は既に十分速い）。

#### K6 バイナリ

| 構成 | counter | basic catalog |
|---|---|---|
| 現状（`[profile.release]` なし） | 24.6 MB | 54.3 MB |
| fat LTO、codegen-units=1、opt-level=3、strip | 22.1 MB | — |
| fat LTO、codegen-units=1、opt-level="s"、strip | 14.5 MB | — |
| 同上で `gpu` なし | 13.7 MB（GPU 一式は約 0.9 MB） | — |
| 同上で `gpu` なし、parley の `complex-scripts` なし | 10.6 MB（ICU の分かち書き辞書が約 3.1 MB） | — |

**`main` の release ビルドはコンパイルできない**（1.3 節 R-0）。release を誰も作っていなかったため、バイナリサイズと release の性能はこれまで測られていない。

### 1.3 根本原因

| ID | 原因（コードの事実） | 影響する KPI |
|---|---|---|
| R-0 | `kivact-engine/src/check/shaped_lines.rs` の `debug_assert_eq!` が、`cfg(any(debug_assertions, feature = "testing"))` でしか公開されない `measure::text_props` を参照する。release で E0425。release で `kivact-core/src/launch/runtime.rs` の `info`（112 行）と `trace_name`（161 行）が未使用の警告 | K6 ほか全部（release が作れない） |
| R-1 | `AssetStore::new`（`arvane-asset/src/store/asset_store.rs:83-84`）が SVG の `<text>` のためだけに `fontdb::Database::load_system_fonts()` を UI thread で同期実行する（この機械で 614 ファイル、697 MB の font を走査、282 ms）。parley（fontique）の走査とは別の二度目の走査 | K1、K2 |
| R-2 | CPU compositor の `paint`（`arvane-compose/src/cpu/draw.rs:273-311`）が、全画素で多角形の coverage、射影 map、f64→f32 の重み、f32 の Porter–Duff と blend を計算する汎用 loop しか持たない。平行移動だけ・不透明度 1・Normal/SrcOver の通常の合成にも同じ loop を使う（約 63 ns/px） | K1（初回 frame）、K4（CPU）、試験の速度 |
| R-3 | GPU を既定で起動時に初期化する（`arvane/src/window.rs` `Driver::start` → `start_gpu`）。GPU を必要としない画面でもドライバと 26 の pipeline のメモリを常に持つ | K2、K3 |
| R-4 | layout の 1 pass ごとに O(N) の処理：`taffy::round_layout` が全ノードを丸める（`kivact-layout/src/tree.rs:856`）、`set_final_layout` が全ノードを比較する、`scale_item_styles` が全 slot を走査する（`tree.rs:861-913`） | K3、K4、K5 |
| R-5 | paint list を作り直すたびに `layer_facts`（`kivact-engine/src/layer.rs:209-255`）が描画される全ノードを再帰で走査し、ノードごとに祖先を辿る（O(N·深さ)）。作り直しは layout の計算・scroll・状態 style を持つノードの hover・state motion の毎 tick で起きる（`model/interact/sync.rs:158-166`） | K4、K5 |
| R-6 | node の install のたびに flat-children の cache を全部捨てる（`tree/node_tree.rs:320`） | K5（mount、行の入替え） |
| R-7 | geometry が一つでも変わると a11y の dirt が `Metadata` になり、`build_identity_metadata` と `build_dirty_nodes` が全ノードを走査する（`a11y/build/dirty.rs`、`model/interact/geometry.rs:173-175`） | K5（支援技術が有効なとき） |
| R-8 | 1 ノードの style が何重にも保持される。authored `Style`（1,928 B の密な struct）を closure が保持、`StateStyles`（6,024 B）を状態 style が無いノードにも常に box で保持、`LayoutProps`/`PaintProps` を既定値でも保持、`ComputedTextStyle` を全ノードに `Rc` で個別に保持 | K2、K5（mount） |
| R-9 | `Rc<RefCell<Option<T>>>` の `T` が大きい（`Rc::new` は `None` でも `T` の大きさを確保する）。core の mount の stage 入力が node ごとに数 KB を残す | K2 |
| R-10 | 部品 1 つあたりの reactive node が多い（button に約 40、`memo<bool>` が 11） | K2、K5（mount） |
| R-11 | release の profile が無い（LTO なし、strip なし）。`complex-scripts` が常に有効。`arvane-compose` が使わない `kivact-shader` に依存。`arvane` だけ workspace 外の `tracing` 版指定で既定 feature を引く | K6 |

R-2 は特に大きい。初回 frame の 99 ms、CPU 描画時の全画面 frame（800×600 で 69 ms）、E2E 試験の各 frame（既定は CPU 800×600）のほとんどがこの loop である。

### 1.4 改善目標（B0 に対する倍率）

倍率は「B0 の値 ÷ B1 の値」（大きいほど良い）。「必達」は 1.6 節の受入れの条件である。「見込み」は設計から見積もった到達点で、判定には使わない。

| KPI | 指標 | 分析機の B0 | 必達 | 見込み | 主な手段 |
|---|---|---|---|---|---|
| K1 | main→初回 present（`kv_startup` 800×600、既定 renderer） | 約 390 ms | **≥ 5×** | 6〜8×（50〜65 ms） | R-1 遅延 font DB、R-2 CPU 合成の高速経路 |
| K1 | basic catalog の起動→初回 present（プロセス生成を含む） | 680〜900 ms | ≥ 2.5× | 3× | 同上 |
| K2 | counter のアイドル WS（既定 renderer） | 196 MiB | **≥ 5×** | 6〜7×（約 30 MiB） | R-3 GPU を必要時に起動、R-1、R-8 |
| K2 | counter のアイドル WS（CPU 強制） | 47 MiB | ≥ 1.4× | 1.5〜1.7× | R-1、R-8 |
| K2 | counter のアイドル WS（`ARVANE_RENDERER=gpu`） | 183〜196 MiB | 悪化させない（≤ 1.05×B0） | 1.1× | R-8（ドライバ分は対象外） |
| K2 | 3000 行のアイドル WS（既定 renderer） | 611 MiB | **≥ 5×** | 6〜7× | R-3、R-8、R-9、R-10 |
| K2 | item あたり：空 box / text / button / 行 | 30 / 12 / 117 / 141 KB | ≥ 5× / ≥ 4× / ≥ 5× / ≥ 5× | 7× / 5× / 6× / 6× | R-8、R-9、R-10 |
| K2 | 3000 行のアイドル WS（CPU 強制） | 472 MiB | ≥ 3.5× | 4× | R-8、R-9、R-10 |
| K3 | アイドル | 0 frame、約 0 ms/s | **維持**（0 frame、≤ 0.5 ms/s） | 同左 | 既存の挙動を壊さない |
| K3 | 3000 行・毎 frame 更新の CPU | 22〜25% | **≥ 5×** | 6〜8× | R-4、R-5 |
| K4 | 3000 行の work p95 / p99 | 9.5 / 10 ms | **≥ 6× / ≥ 5×** | 7〜9× | R-4、R-5 |
| K4 | 0 行の work p95 | 2.0〜2.5 ms | 悪化させない（≤ 1.05×B0） | 1.2× | R-2（CPU） |
| K4 | 描画間隔 p99（3000 行） | 18.4〜19.5 ms | ≤ 1.15 × frame 周期 | 同左 | 上記 |
| K4 | CPU の全画面 frame（`kv_startup` の最初の frame の raster、800×600、CPU 強制） | 69 ms | **≥ 6×** | 8〜12× | R-2 |
| K5 | 無関係な 1 text の更新、n=3000 | 50〜72 ms | **≥ 8×** | 10〜15× | R-4、R-5、R-7 |
| K5 | 1 行の更新、n=3000 | 83〜113 ms | ≥ 6× | 8〜10× | 同上 |
| K5 | pointer 移動 p95、n=3000 | 69〜95 ms | **≥ 10×** | 15〜25× | R-5 |
| K5 | wheel、n=3000 | 13〜16 ms | ≥ 1.5× | 2× | R-5（a11y の subtree の作り直しは残る、1.5 節） |
| K5 | mount、n=3000 | 1,040〜1,140 ms | ≥ 2× | 2.5〜3× | R-6、R-8、R-9、R-10 |
| K5 | counter の click | 1.6 ms | ≥ 1.2× | 1.3× | R-2（damage の合成） |
| K6 | counter（既定 feature） | 24.6 MB | **≥ 1.7×** | 1.7〜1.8×（約 14 MB） | R-11 |
| K6 | `kv_startup`（`--no-default-features`。B0 は同じ build の既定） | 24.6 MB 前後 | ≥ 2.2× | 2.3×（約 11 MB） | R-11 の `complex-scripts` |
| K6 | basic catalog（既定 feature） | 54.3 MB | ≥ 1.7× | 1.8× | R-11 |

目標の根拠：

- **K1**：B0 の 390 ms のうち 282 ms が R-1、約 69 ms が R-2 である。両方を除くと window 作成 23 ms、mount と layout 約 10 ms、最初の font 読込みと shaping、raster と present が残る。R-2 の高速経路で 800×600 の全画面 frame は 5〜10 ms に下がる（vello_cpu の塗りと行の複写）。合計 50〜65 ms。
- **K2**：既定 renderer のアイドル WS の大半は GPU ドライバである。GPU を必要時に起動する（R-3）と、counter のような画面は CPU の WS（47 MiB）になり、さらに R-1（system font の DB）と R-8 で約 30 MiB に下がる。item あたりは 1.2 節の割当て元から積み上げた。空 box は `StateStyles` 5.9 KB、`resolve` の残留、`Style` 1.9 KB、`ComputedTextStyle` 0.47 KB がほぼ消え、layout 列（約 1〜2.7 KB）と `LayoutProps` 0.7 KB と `HostNode` が残るので 4〜5 KB（6〜7×）。button は reactive node を 40 から 12 以下にし（R-10）、状態 style を box の delta だけにし（hover・pressed・focus_visible の `PaintDelta` 約 2.2 KB は残る）、resolve の残留と recipe の `Style` 3 層（5.7 KB）を除いて 18〜22 KB（5〜6×）。
- **K3/K4**：3000 行で scene が 7.1 ms、毎 frame の O(N) は R-4（layout の丸め・比較・走査）と R-5（`layer_facts`）である。これを変わったノードとその祖先だけの処理にすると、見えている 30 行程度の lower と raster（約 1 ms）が残る。ただし完全な O(changed) ではない：変わった祖先の子の一覧を一度ずつ見る比較（LAY-1）と、事実が変わった親の子の集計（PNT-1）は、その親の子の数に比例する。これは Taffy 自身がその親を layout し直すときの子の走査と同じ大きさで、1 子あたり数 ns の比較である。幅（兄弟の数）を増やした時の伸びは `kv_stress` の 0 行と 3000 行の差で確かめる。
- **K5**：試験 driver は a11y を毎回公開する。R-7 で geometry だけの変化は identity metadata を作り直さない経路にするが、走査自体（全ノードの stack 移動）は O(N) のまま残るため、無関係な更新でも 3000 行で 2〜4 ms は残る。wheel は scroll container の subtree の a11y payload を作り直すので、改善は小さい（1.5 節）。
- **K6**：profile だけで 14.5 MB（1.7×）。速度に効く crate だけ opt-level=3 に戻す（4.8 節）ので既定 feature で約 14 MB。`complex-scripts` を外せる構成で約 11 MB。

### 1.6 受入れの条件

次の 4 つをすべて満たしたときだけ、本 refactor を完了とする。どれかを満たさない場合は、作業指示書の W7 で完了を宣言せず、満たさない項目と原因を設計者（人間）に報告して止める。設計書と作業指示書は削除せず、architecture.md と decisions.md にも書き戻さない。

| ID | 条件 | 確かめ方 |
|---|---|---|
| AC-1 正しさ | CP3 の gate（fmt、xtask、clippy、workspace の test、kivact-docs、gpu-interop-producer、feature の組合せ）がすべて成功する。既存の E2E の期待値を緩めていない | 作業指示書 11 章 |
| AC-2 KPI | 有効な B0 と B1（1.1 節）で、1.4 節の「必達」の行がすべて満たされる。満たさない行は、同じ条件で 2 回測り直し、3 回中 2 回以上満たせば満たしたとする（雑音の扱い。未達を許す判断ではない） | `dev/bench/results/RESULT.md` |
| AC-3 状態 | 6.4 節の状態の監査で、未解決の項目が 0 件 | `cargo xtask check --only state-fields`（W6 の後）と `dev/xtask/state-audit.md` |
| AC-4 未実施 | 実装者の STOP で実施しなかった項目と撤退規則を適用した項目のうち、AC-1〜AC-3 と 5 章の公開 API に影響するものが 0 件（影響しない小さな最適化の撤退は記録して残してよい） | PR 説明の下書き |

Linux の確認は、実機（WSL を含む）があれば行い、無ければ「Linux 未確認」と報告する。未確認のまま Linux の品質を適合とは書かない。

### 1.5 目標に含めないもの（理由）

| 項目 | 理由と再検討の条件 |
|---|---|
| GPU を強制した時のメモリ | ドライバ依存（分析機の Vulkan で +150〜250 MiB、DX12 はさらに大きい）。Kivact が持つ texture は R-8 で減るが、倍率の目標にはしない。悪化（+5% 超）は不可（1.4 節の行） |
| scroll 時の a11y payload の作り直し | a11y の bounds を絶対座標で公開しているため、scroll container の子孫すべての payload が変わる。親に対する相対座標と AccessKit の transform で公開する設計が必要で、本 refactor の範囲外。支援技術が無効の本番経路には影響しない |
| programmatic な `scroll_to` の a11y | `model/apply.rs:138` の `scroll_to` は今も Metadata の dirt を出すので、A11Y-1 の Geometry の経路を通らない。wheel と programmatic scroll は別々に測り、scroll 全般が速くなったとは書かない |
| engine の lower（描画命令の生成）の node 単位の保持 | 見えている entry だけを毎 frame lower しており O(visible)。3000 行でも 30 行分で、計測では scene 7.1 ms のほぼすべてが R-4/R-5 だった。保持は SceneOut の作り直しを要し、危険に見合わない |
| GPU 合成の部分描画・部分 present | wgpu に部分 present が無い。GPU では全画面の合成でも 1 ms 未満 |
| CPU raster の命令ごとの culling | 部分 raster の矩形は小さく、命令の走査は 0.1 ms 程度 |
| JSX 風の `view!` macro | 診断と IDE 補完が悪くなる。現在の関数呼び出しの組み立ては React・Solid の利用者に十分読める（5 章） |
| ICU の locale data の縮小 | Q-07（全 locale の compiled data を含める）を維持する |

---

## 2. 方針と設計規則

### 2.1 実装方針の具体化

| 実装方針 | 本設計での具体化 |
|---|---|
| Rust 未経験の React・SolidJS 開発者がスムーズに書ける | `#[component]` で props の builder を生成し、手書きの builder（約 80 行）を不要にする（5.2 節）。React の「component は view を返す関数」と同じ形にする |
| わかりやすく統一的な公開 API | 「名前が一つの意味を持つ」ように改名する：`NodeBuilder`→`Element`、`component()`→`scope()`、試験の `Element`→`Locator`、trait `Component` は削除（5.3 節）。handler・開閉・選択・項目の引数を全部品で一つの規約に揃える（5.4 節） |
| JS・Python を書く人でも容易に記述できる | 既定値の props、名前付きの任意 props、`on_*` の closure の型推論を壊さない（5.1 節）。`Binding::update` を足し、`open.set(!open.get())` を不要にする |
| 内部が統一的でシンプル | 1 ノードの style を一か所にだけ持つ（4.7 節）。暗黙の状態機械を enum と Machine にする（6 章）。O(N) の再計算をやめ、「変わったものだけを処理する」という一つの規則にする（PERF-1） |
| 長期保守性 | 性能を `dev/bench` で測れるようにする。型の大きさの上限をコンパイル時に検査する（MEM-7）。状態の lint の抜け穴を塞ぐ（6.2 節） |
| strategy.md を簡単に遂行できる | 9 章の「実行時品質」の指標を K1〜K6 として定義し測れるようにする。P-05（全体の費用で最適化）を、測定に基づく原因（1.3 節）だけを直す形で守る。G-04（資源を測る）のうち、mount した画面の資源は K2 で、アンマウント後の回収は E2E の資源の検査（7 章）で確かめる |

### 2.2 新しい設計規則

本 refactor の後も守る規則。W7 で architecture.md に書き戻す。

| ID | 規則 |
|---|---|
| PERF-1 | frame・turn・layout pass・a11y 公開の処理は、変わったノードとその祖先（と、その祖先の子の一覧の安価な比較）だけを処理する。全ノードの走査は、構造の初期化・font の登録・環境の変化・明示的な全体の再構築（`Full`）だけに使う。新しい O(N) の処理を足すときは、その理由を関数の doc comment に書く |
| PERF-2 | 一度しか要らない重い初期化（system font の DB、GPU、pipeline、ICU の formatter）は、使う時に作る。起動の critical path（main→初回 present）に新しい同期処理を足さない |
| PERF-3 | CPU 合成は、平行移動だけ・不透明度 1・Normal と SrcOver・mask と cover なしの描画に整数の高速経路を使う。汎用 loop はそれ以外にだけ使う |
| MEM-1 | ノードごとに保持する値は、既定値なら保持しない（`None`）。読む側は `None` を既定値として扱う |
| MEM-2 | 64 byte を超える値を `Rc<RefCell<Option<T>>>`・`Rc<Cell<Option<T>>>`・`Option<T>` の field として多数のノードに持たせない。`Option<Box<T>>` にする |
| MEM-3 | 多数のノードが同じ値を持つ場合（継承した computed text style など）は、親と同じなら `Rc` を共有する |
| MEM-4 | authored `Style` は field の群ごとの疎な struct で持つ（使った群だけを確保する）。操作の履歴を保持しない |
| MEM-5 | 部品 1 つの reactive node は、その部品が実際に独立して変わる値の数までにする。同じ source から導く複数の `memo<bool>` は、`Copy` な struct を返す一つの `memo` にする |
| MEM-6 | ノードごとの主要な型に、`const` の大きさの上限を置く（4.7 節 MEM-7 の表）。上限を超える変更はコンパイルできない |
| DEP-5 | 重い任意機能（ICU の分かち書き辞書）は facade の feature にし、末端の crate が実装する（DEP-4 の形）。既定では有効 |

---

## 3. crate 構成

### 3.1 判断の基準

architecture.md の B-1〜B-5 に従う。crate に分けるのは「入出力を少数の型で説明でき、相手の内部状態を読まず、一つの変更理由を持つ」境界だけ。境界がはっきりしないものは分けない。

### 3.2 追加する crate

| crate | 場所 | 境界の理由 |
|---|---|---|
| `kivact-macros` | `crates/kivact-macros` | `#[component]` の proc-macro。proc-macro は Rust の規則で独立の crate でなければならない。入力は関数の構文木、出力は token だけで、Kivact の内部状態を読まない。facade `kivact` だけが依存し再公開する |
| `kivact-bench` | `dev/bench` | KPI の手動計測。公開 API（facade）だけを使う葉。誰も依存しない |

### 3.3 検討したが分けない・統合しないもの

| 候補 | 判断 | 理由 |
|---|---|---|
| `kivact-core::devtools`（5.1k 行）を crate に | 分けない | core の view state・runtime・engine Reader の内部を広く読む。分けると doc(hidden) の内部 bridge が大量に要り、B-1 を満たさない |
| `kivact-engine` の `a11y`・`text_input`・`host` | 分けない | `Model` の内部を共有する（ADR-10 と同じ理由） |
| `arvane-render` と `arvane-compose` の統合 | 統合しない | Vello を `arvane-render` に閉じる境界（BOUNDARY-COMPOSE）がはっきりしている |
| `kivact-shader` の統合 | 統合しない | naga を閉じる境界（BOUNDARY-SHADER）。ただし使っていない `arvane-compose` からの依存は削除する（4.8 節） |
| `arvane-text` と `arvane-asset` の font DB の統合 | 統合しない | resvg は fontdb を要求する。DB を遅延生成すれば二重走査は起動から消える（4.1 節） |
| 小さい crate（`kivact-editing`、`kivact-i18n`、`kivact-devtools-client`） | そのまま | それぞれ編集 buffer、ICU、二重実装の防止という明確な境界を持つ |

### 3.4 依存の変更

| 変更 | 内容 |
|---|---|
| root `Cargo.toml` | `[profile.release]` を追加（4.8 節）。workspace の `parley` から `complex-scripts` を外す。`members` に `crates/kivact-macros` と `dev/bench` を追加。`[workspace.dependencies]` に `syn = { version = "2", features = ["full"] }`、`quote = "1"`、`proc-macro2 = "1"` |
| `crates/kivact/Cargo.toml` | 依存 `kivact-macros`、optional 依存 `arvane-text`、feature `complex-scripts = ["dep:arvane-text", "arvane-text/complex-scripts"]`、`default = ["gpu", "complex-scripts"]` |
| `crates/arvane-text/Cargo.toml` | `[features] complex-scripts = ["parley/complex-scripts"]`（既定は空） |
| `crates/arvane/Cargo.toml` | `tracing = "0.1.41"` を `tracing = { workspace = true }` に |
| `crates/arvane-compose/Cargo.toml` | `kivact-shader` の依存を削除（source は参照していない） |
| `dev/xtask/policy/workspace.toml` | `visible_to` に `kivact-macros = ["kivact"]`、`kivact-bench = []`。`kivact-shader` の `visible_to` から `arvane-compose` を外す。`arvane-text` の `visible_to` に `kivact` を足す（feature の転送のため。facade は呼ばない） |

---

## 4. 性能の設計

### 4.1 起動：SVG の font DB を遅延生成する（ST-1、R-1）

`arvane-asset` に、SVG の `<text>` のための font DB を初めて必要になった時に作る型を置く。

```rust
// crates/arvane-asset/src/svg/fonts.rs（新規）
//! Role: The font database of SVG `<text>`, built on the first SVG that has text.

use std::sync::{Arc, Mutex};

/// System fonts plus the App's fonts, loaded the first time an SVG with
/// text is parsed (on the asset worker thread), never at startup.
pub(crate) struct SvgFonts {
    /// The App's font files, in registration order.
    app: Vec<Arc<Vec<u8>>>,
    /// The database once built (a pure cache of `app` and the system fonts).
    db: Mutex<Option<Arc<usvg::fontdb::Database>>>,
}

impl SvgFonts {
    /// No App fonts yet; nothing is loaded.
    pub(crate) fn new() -> Self {
        Self { app: Vec::new(), db: Mutex::new(None) }
    }

    /// The same App fonts plus `bytes`; the database is built again on its next use.
    pub(crate) fn with_font(&self, bytes: Vec<u8>) -> Self {
        let mut app = self.app.clone();
        app.push(Arc::new(bytes));
        Self { app, db: Mutex::new(None) }
    }

    /// The database for `svg`: an empty one when the document has no text
    /// element, else the system and App fonts (built on the first call).
    pub(crate) fn for_document(&self, svg: &[u8]) -> Arc<usvg::fontdb::Database> {
        if !has_text(svg) {
            return Arc::new(usvg::fontdb::Database::new());
        }
        let mut db = self.db.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        Arc::clone(db.get_or_insert_with(|| {
            let mut fonts = usvg::fontdb::Database::new();
            fonts.load_system_fonts();
            for font in &self.app {
                fonts.load_font_data(font.as_ref().clone());
            }
            Arc::new(fonts)
        }))
    }
}

/// Whether the SVG may draw text with these fonts: a text element (`<text`,
/// or a prefixed `:text`), or an embedded SVG image (a `data:` URL of type
/// `image/svg+xml`, base64 or percent-encoded, whose inner document uses
/// the same font database). A false positive only loads the fonts.
fn has_text(svg: &[u8]) -> bool {
    let has = |needle: &[u8]| svg.windows(needle.len()).any(|w| w.eq_ignore_ascii_case(needle));
    has(b"<text") || has(b":text") || has(b"svg+xml") || has(b"svg%2bxml")
}
```

`OnceLock`/`OnceCell` は `clippy.toml` の `disallowed-types` なので使わない。空の DB は毎回作る（`Database::new()` は空の `Vec` を作るだけ）。`db` は state の lint で数えられる stored の field が一つだけなので marker は要らない。

- `svg/document.rs` の `pub(crate) type FontDb = Arc<usvg::fontdb::Database>;` を `pub(crate) type FontDb = std::sync::Arc<SvgFonts>;` に変える。`parse(bytes, fonts, budget)` の `fontdb: Arc::clone(fonts)` を `fontdb: fonts.for_document(bytes)` に変える。
- `AssetStore` の `fonts` の初期化は `Arc::new(SvgFonts::new())`（`load_system_fonts` を呼ばない）。font の登録（`asset_store.rs:159`）は `self.fonts = Arc::new(self.fonts.with_font(bytes.to_vec()));`。
- 圧縮 SVG（SVGZ）は展開後の byte が `parse` に渡る（`job.rs:229-248`）ので、`has_text` は展開後に判定される。
- 埋込みの SVG（`<image href="data:image/svg+xml;..">`）の内側の文字も、外側と同じ font DB を使う（usvg 0.46 の `parser/image.rs`）。中身を decode せずに判定するため、`svg+xml` の data URL があれば保守的に DB を用意する。
- 結果：起動時の system font の走査が無くなる。text を含む SVG を最初に decode する worker thread の上で一度だけ走査する（試験の inline 実行では UI thread、許容する）。

### 4.2 CPU 合成の高速経路（CPU-1、R-2、PERF-3）

`arvane-compose/src/cpu/draw.rs` の `paint` の先頭で、次の条件をすべて満たすときは高速経路を使い、汎用 loop を通らない。

| 条件 | 確かめ方 |
|---|---|
| 写像が整数の平行移動 | `map.m == [[1,0,tx],[0,1,ty],[0,0,1]]` で、`tx` と `ty` が整数（`v.fract() == 0.0`、有限）。`texel(x,y)` は `dst(x,y)` に対し `src(x+tx, y+ty)` を読む |
| 形が画素に揃った軸平行の矩形 | `op.polygon.points` が 4 点で、すべて整数で、`(x0,y0)`・`(x1,y0)`・`(x1,y1)`・`(x0,y1)` の 4 つの異なる角がそろう（重複した角の三角形は除く） |
| 不透明度 1、coverage なし | `op.opacity == 1.0`、`op.coverage.is_none()`（`cover` が `None`） |
| Normal と SourceOver | `op.blend == BlendMode::Normal`、`op.composite == Composite::SourceOver`（`kivact-types/src/render/composition.rs:129` の variant 名） |
| 乗算済みの texture | `texture` が `Texture::Premul(_)` |
| 標本化 | `op.sampling` を問わない（整数の平行移動では Nearest と Linear が同じ画素を読む） |

高速経路の処理：

1. 描く範囲 = 形の矩形 ∩ `dst` の範囲。`Scissor::Rects` のときは、汎用 loop と同じく画素ごとに「どれかの矩形に入るか」を調べる（damage の矩形は重なることがあり、矩形ごとに描くと半透明の画素を二度合成するため）。範囲外を読む画素（texture の外）は透明として扱う（汎用 loop の `texel` の範囲外と同じ）。
2. 行ごとに、source の行の slice と dst の行の slice を取り、画素ごとに整数の src-over を計算する：`a = s[3]`。`a == 255` なら `d = s`。`a == 0` なら何もしない。それ以外は各 channel `d = min(255, s + (d * (255 - a) + 127) / 255)`（u16 で計算し u8 に戻す）。
3. `Pixmap` の内部表現が乗算済み RGBA8 であることを `cpu/surface.rs` で確かめてから使う。乗算済みでない場合は「停止して報告」。

結果の画素は汎用 loop と ±1 の丸め差で一致する（汎用 loop は f32、高速経路は整数）。既存の画素比較の試験の許容差（`Screenshot::raster_difference`）の範囲に入る。範囲を超える試験が出たら、許容差を変えずに「停止して報告」する。

### 4.3 GPU を必要時に起動する（GPU-1、R-3）

#### 公開 API

`kivact-types/src/gpu.rs` の `GpuPreference` に field を一つ足す。

```rust
/// When the window's GPU device is started.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GpuStart {
    /// Frames are drawn on the CPU until the window needs the GPU: a
    /// composition with a user shader or a GPU export, or CPU frames that
    /// keep missing half the display period (the default).
    #[default]
    OnDemand,
    /// Start the device with the window (frames are drawn on the CPU until
    /// it is ready), as before.
    AtLaunch,
}

pub struct GpuPreference {
    pub power: GpuPower,
    pub backend: Option<GpuBackend>,
    pub adapter: Option<AdapterMatch>,
    /// When the device is started.
    pub start: GpuStart,
}
```

- `App::gpu_preference` はそのまま（`GpuStart` も `kivact::gpu_interop` から再公開する）。
- 優先順位（上が強い。一つの表で決め、他の場所に別の規則を書かない）：

| 入力 | 結果 |
|---|---|
| `ARVANE_RENDERER=cpu`、または `Shell::prefer_gpu(false)` | GPU を使わない（今と同じ） |
| `ARVANE_RENDERER=gpu` | `AtLaunch` |
| `ARVANE_RENDERER=auto` | `OnDemand` |
| 環境変数なし | `App::gpu_preference` の `start`（既定 `OnDemand`） |
| 試験の host で GPU を要求するもの（`RenderOptions::gpu`、`GpuMode::Required`） | `AtLaunch`（GPU の lane の E2E）。OnDemand の経路は 7 章の別の E2E で確かめる |

#### 状態

GPU の起動は presenter の `Gpu`（`arvane/src/presenter/gpu.rs`）だけが持つ。`enum State` に variant を一つ足す。bool や counter を別の field に置かない（6 章）。

```rust
enum State {
    /// Not started and never will be (CPU only).
    Off,
    /// Not started yet: started on the first GPU demand (`demand`), or when
    /// `slow` consecutive CPU frames missed half the display period.
    Deferred { slow: u8, waker: ShellWaker },
    Background(Background),
    Failed,
}
```

- `Driver::start` と二つ目以降の window（`Windows::open`）：`AtLaunch` なら今どおり始める。`OnDemand` なら `State::Deferred { slow: 0, waker }`。CPU 指定なら `Off`。
- 起動の入口は二つだけで、どちらも `Gpu::start` を呼ぶ（`Off` と `Deferred` で始め、それ以外では何もしない。何度呼んでも同じ）：
  1. **需要**（`Presenter::demand_gpu`）。次の三つの場合に呼ばれる。
     - CPU の frame の composition が GPU を要する（4.3.1）。
     - core の GPU 共有の利用（下の「GPU の需要の伝達」）。
     - drawable の GPU の export の要求。
  2. **遅い CPU frame**（`Presenter::note_cpu_frame`）：`raster + present > period / 2` の frame が 8 回続いたら始める（`period / 2` 以下の frame で `slow = 0`）。
- `start` は今どおり device の作成と pipeline の compile を worker thread で行い、その間も CPU で描き続ける（UI thread を止めない）。GPU が Ready になってから GPU で描く。GPU から CPU に自動で戻ることはしない（失敗時の CPU fallback は今どおり）。

#### GPU の需要の伝達（GPU 共有の循環を断つ）

GPU 共有（`gpu-interop`）の利用は、GPU が Ready になる前には composition に現れない。

- `kivact-core/src/gpu_interop/import/registry.rs:135` の `adapter()` は Ready 前には `None` を返す。
- `share/app_side.rs:78`、`:99` は、adapter が無ければ Pixels を要求する。
- `view/elements/viewport.rs:131`、`:164` は、adapter が無ければ GPU frame を載せない。
- `kivact-engine/src/model/drawables/export.rs:52` は、pool が無ければ GPU export を出さない。

そのため「composition に GPU の処理が出たら始める」だけでは、GPU がいつまでも始まらない。core の側で需要を明示的に届ける：

- engine の `HostRequest` に `DemandGpu { window: WindowId }` を足す。engine の host はそれを受けて arvane の `Windows::demand_gpu(window)` を呼び、arvane はその window の presenter の `demand_gpu` を呼ぶ。
- core の次の場所で、その window について一度 `DemandGpu` を出す（同じ owner からの二度目以降は出さない。presenter の側も冪等）。
  - GPU 共有の import registry を作る時。
  - shared image を作る時（`share/app_side.rs`）。
  - viewport に `gpu_frame` を設定する時。
  - drawable の export を GPU の texture で要求する時。
- これで「需要 → GPU 起動 → adapter と pool の用意 → GPU frame/export」の順になる。重複した需要は `Gpu::start` の冪等性で吸収する。

#### 4.3.1 GPU を要する composition

`kivact-types/src/render/` の `Composition` に `pub fn needs_gpu(&self) -> bool` を足す。user shader を参照する layer の effect（`LayerFilter` の shader と paint shader）があれば true。外部の GPU texture と GPU export は上の需要の伝達で扱うので、ここでは判定しない。

#### pipeline の作成は今どおり worker で行う

pipeline を初回の draw まで遅らせる案（GPU-2）は採らない。UI thread の描画中に compile が走り、初回の shader・filter・mask の操作が止まるため。pipeline は今どおり GPU の起動時に worker thread で作る。`OnDemand` では起動そのものが遅れるので、使わない GPU の pipeline のメモリは GPU を使わない画面では発生しない。

### 4.4 layout の O(changed) 化（LAY-1、LAY-2、R-4）

#### LAY-1 差分の丸め

`taffy::round_layout(pass, root)`（`kivact-layout/src/tree.rs` の `compute_at`）を、変わった subtree だけを丸める `round_changed` に置き換える。

列の追加（`NodeLayoutValues`、`columns.rs`）：

| 列 | 型 | 意味 |
|---|---|---|
| `touched` | `Vec<u32>` | この slot の `set_unrounded_layout` が呼ばれた pass の番号 |
| `below` | `Vec<u32>` | この slot か子孫が touched になった pass の番号 |
| `origin` | `Vec<(f32, f32)>` | 最後に丸めたときの親までの累積座標（`cumulative_x`, `cumulative_y`）。新しい slot と reset した slot は `(f32::NAN, f32::NAN)` |

`NodeLayoutColumns` に pass の番号 `stamp: u32` を足す（`compute` の最初と、engine が直接呼ぶ `Layout::compute_overlay` の最初に `wrapping_add(1)`、0 は使わない）。

`LayoutPass::set_unrounded_layout` は値を保存し、**値が前と違うか、その slot がまだ一度も丸められていない（`origin` が NaN）ときだけ** `touched[i] = stamp` とし、`i` から親へ `below[j] = stamp` を、既に `stamp` の祖先に着くまで書く（親は `tree.parent(slot)`、viewport の slot 0 まで）。Taffy は子の cache が当たっても、親を layout し直すと各子の `set_unrounded_layout` を呼ぶので、同じ値で印を付けない。

```rust
/// Round the subtrees whose unrounded layout or position changed in this
/// pass; the others keep their final layout (taffy's `round_layout`, but
/// only where something changed).
fn round_changed(pass: &mut LayoutPass<'_, impl LayoutTree + ?Sized>, node: TId, cum: (f32, f32), stamp: u32) {
    let i = usize::from(node);
    let same_origin = pass.origin[i] == cum;   // NaN never compares equal
    if same_origin && pass.below[i] != stamp {
        return;
    }
    let unrounded = pass.unrounded[i];
    if !same_origin || pass.touched[i] == stamp {
        let layout = rounded(unrounded, cum);  // the body of taffy 0.14's round_layout_inner, unchanged
        pass.set_final_layout(node, &layout);
        pass.origin[i] = cum;
    }
    let child_cum = (cum.0 + unrounded.location.x, cum.1 + unrounded.location.y);
    for index in 0..pass.child_count(node) {
        let child = pass.get_child_id(node, index);
        round_changed(pass, child, child_cum, stamp);
    }
}
```

- `rounded(unrounded, cum)` は taffy 0.14 の `round_layout_inner`（`taffy-0.14.0/src/compute/mod.rs:219-259`）の `layout` の計算部分を、`cumulative_x/y` を `cum` に置き換えて写す（`content_size` の feature が有効なので `round_scrollable_overflow_rect` も写す。taffy の private 関数なので同じ計算を本 crate に書く）。
- 再帰は `TId` の子を `child_count`/`get_child_id`（`RoundTree` が要求する `TraversePartialTree`）で辿る。taffy と同じ順序。
- `compute_overlay` と `compute_at` の両方の `round_layout` を置き換える。overlay の root も root と同じ規則。
- `set_final_layout` は変えない（変わった時だけ `geometry_changed` に積む）。
- 正しさの理由：丸めの結果はノード自身の unrounded layout と累積座標だけで決まる。どちらも変わらない subtree は前回の結果と同じである。
- 計算量：丸めは、印の付いたノードとその祖先、および印の付いた祖先の子の一覧の比較（1 子あたり数 ns）に比例する。親の位置が本当に変わった場合は、その子孫すべての丸めが変わるので、その subtree の大きさに比例する（これは避けられない）。

#### LAY-2 presence の scale を疎に持つ

`scale_item_styles`（`tree.rs:861`）は `presence_scale` の全 slot を走査する。`NodeLayoutValues` に `scaled: BTreeSet<usize>`（`presence_scale[i].is_some()` の slot の集合）を足し、`Layout::set_presence_scale` と `reset` で同期する。`scale_item_styles` は `scaled` だけを走査する。`scaled` は `presence_scale` の投影の索引で、書くのは `set_presence_scale` と `reset` の 2 か所だけ（`BTreeSet` は state の lint で数えられないので marker は不要）。

### 4.5 paint list の O(changed) 化（PNT-1、PNT-2、R-5、R-6）

#### PNT-1 subtree の事実を保持して差分で更新する

`PaintList.facts: HashMap<NodeId, SubtreeFacts>`（`paint/list.rs`）は、今は作り直しのたびに `layer_facts()` で全部作り直す。これを保持し、変わったノードとその祖先だけを更新する。

- `PaintState` に `stale_facts: StaleFacts` を足す。

```rust
/// The nodes whose subtree facts must be derived again before the next
/// paint list (a work queue, not a phase).
#[derive(Default)]
pub(crate) enum StaleFacts {
    /// Every fact is derived again (first list, new root, layer roots changed).
    #[default]
    All,
    /// Only these nodes (and, through propagation, their ancestors).
    Nodes(BTreeSet<NodeId>),
}
```

- `layer.rs` に、子の事実を `facts` から読む非再帰の関数を足す。本体は `subtree_facts` と同じで、子の再帰の代わりに `facts.get(&child).copied().unwrap_or_default()` を使う。

```rust
/// The facts of `id` from its children's recorded facts (`facts`), given its parent's look.
pub(crate) fn node_facts(view: &impl LayerView, id: NodeId, parent: Option<&Look>, facts: &HashMap<NodeId, SubtreeFacts>) -> SubtreeFacts
```

- `Model::paint_list` の `list.facts = self.layer_facts();` を `self.refresh_facts(&mut list.facts)` に変える。

```text
refresh_facts(facts):
  match take(stale_facts):
    All          => *facts = layer_facts()          // 今と同じ全計算
    Nodes(set)   =>
      queue = set を深さの深い順に並べたもの（深さ = tree の祖先の数）
      while let Some(id) = queue.pop_deepest():
        parent_look = flat parent の drawn_look
        new = node_facts(view, id, parent_look, facts)
        old = facts.get(id).copied().unwrap_or_default()
        if new == default { facts.remove(id) } else { facts.insert(id, new) }
        if new != old:  id の flat parent（layer node なら止める）を queue に足す
```

- **印の正規化**：事実は flat な木（`display: contents` の透明なノードを飛ばした木）で計算する（`layer.rs:229`、`node_tree.rs:94`）。`stale_fact(id)` は、`id` が透明なノードなら、代わりに最も近い透明でない祖先（flat な親）に印を付ける。子を追加・移動したときは、変わった flat な親と、追加・移動した子自身の両方に印を付ける（子の `own_layer` は flat な親の look を読むため）。
- **削除**：削除した subtree の各 id の事実を `facts` から個別に消す（engine の `Removed` の id の列、ADR-29）。毎回 `facts` 全体を `retain` で走査しない。
- **計算量**：stale なノードごとに、そのノードの flat な子の事実を集計する（子の数に比例）。事実が変わったときだけ親へ伝える。事実が変わる場合（hover で filter が付く等）に幅の広い親を集計し直すのは、その親の子の数に比例する。
- 印を付ける場所（`StaleFacts::Nodes` に足す、`All` ならそのまま）：
  - `Model::derive`（`sync.rs`）で `Look` が変わったノード `id` と、その `flat_children` すべて（子の `own_layer` は親の look を読むため）。`Look` に `PartialEq` を derive する。
  - `sync` の `restyled`（状態 style を持つノードの再描画、`sync.rs:163-166`）の各ノードとその `flat_children`（hover などの状態の delta が `drawn_look` を変えるため）。
  - motion の tick（`ticks.rs:159` の `restack |= motion.is_active()`）：動いているノードの一覧が手元にあれば各ノードとその `flat_children`、無ければ `All`。
  - 子の追加・削除・並べ替えの親（PNT-2 と同じ箇所）と、追加・移動した子（印の正規化を通す）。
  - canvas・draw の scene が変わったノード（`scene_facts` の入力が変わる箇所）。
  - `hidden`・`visibility` の変化、`display: contents` の変化（`apply.rs` の reflatten）、root・overlay・fixed の集合の変化は `All`（まれで、子孫の事実が一度に変わるため）。
- `changes.restack = true` を書く既存の箇所はそのまま残す（list の作り直しは O(visible) の walk で、facts の更新とは別）。上のどれにも当たらない `restack` の箇所は、対象のノードが分かれば印を付け、分からなければ `All` にする。
- `refresh_facts` は `paint/list/walk.rs` に置く（engine の module の規則で `layer.rs` は `lower` しか使えない）。`node_facts` は `layer.rs` に置く（`LayerView` だけを使う）。
- `PaintState.stale_facts` は state の lint で数えられない（enum の名前が `State`・`Phase` などで終わらない）ので marker は付けない。
- 正しさの検査：debug build で `paint_list` の最後に `debug_assert_eq!(*facts, self.layer_facts())` を置く（release では消える、既存の self-check と同じ扱い）。debug の paint の self-check（`paint/list/self_check.rs`）は空の `PaintList` から `paint_list()` を呼んで参照の list を作るので、self-check の中では呼ぶ前に `stale_facts` を退避して `All` にし、終わったら元に戻す（退避しないと保留の印が消え、空の facts で検査が失敗する）。

#### PNT-2 flat-children の cache を部分的に捨てる

`NodeTree` の flat の cache の全消去は 4 か所ある（`node_tree.rs:111` の `forget_flat`、`:320` の `install`、`:415` の `insert`、`:446` の `detach`）。次を足し、`install`・`insert`・`detach` の全消去を置き換える。

```rust
/// Forget the cached flat lists that can contain `id`: its own and those of
/// its ancestors up to and including the nearest one that is not transparent.
pub(crate) fn forget_flat_around(&self, id: NodeId) {
    let mut flat = self.flat.borrow_mut();
    flat.remove(&id);
    let mut at = self.get(id).and_then(|n| self.parent_id(n));
    while let Some(parent) = at {
        flat.remove(&parent);
        let Some(node) = self.get(parent) else { break };
        if !node.is_transparent() {
            break;
        }
        at = self.parent_id(node);
    }
}
```

- `insert`（子を親につなぐ）と `detach`（子を親から外す）では、変わった親 `p` について `forget_flat_around(p)` を呼ぶ（`detach` は外す前に親を読んでおく）。移動では元の親と新しい親の両方。
- `install` で既存の id を置き換える場合（同じ id が既にある）は `forget_flat_around(id)`。新しい id の `install` では何もしない（まだどの親にもつながっていない）。
- ノードを tree から消す `remove_with`（`node_tree.rs:472`）は、今は `detach` の全消去が削除した subtree の entry の掃除も兼ねている。部分的な破棄にした後は、`remove_with` が削除する各 id の entry（その id の flat な子の一覧）を `flat` から消す。cache はノードの寿命と同時に終わる（mount と unmount を繰り返しても entry が増えない）。
- ノードの透明さ（`display: contents`）が変わる箇所の `forget_flat()`（全消去、`model/apply.rs`）は残す（まれ）。

### 4.6 a11y：geometry だけの変化（A11Y-1、R-7）

`a11y/state.rs` の `Dirt` に variant を足す。

```rust
pub(super) enum Dirt {
    Clean,
    LocalText { nodes: BTreeSet<NodeId> },
    /// Only geometry changed since the last accepted tree: the payloads of
    /// `nodes` and of every node under `subtrees` are built again; the
    /// accepted identity metadata is reused when the clipped rows are the same.
    Geometry { nodes: BTreeSet<NodeId>, subtrees: BTreeSet<NodeId> },
    Metadata { nodes: BTreeSet<NodeId>, subtrees: BTreeSet<NodeId>, removed: BTreeSet<NodeId> },
    Full,
}
```

- `A11yState` に `pub(crate) fn moved(&mut self, id: NodeId)` を足す。`Clean` → `Geometry { nodes: {}, subtrees: {id} }`。`LocalText { nodes }` → `Geometry { nodes, subtrees: {id} }`。`Geometry` → `subtrees` に追加。`Metadata` → `subtrees` に追加（今と同じ）。`Full` → そのまま。
- 既存の `node`・`subtree`・`request_publication` は、`Geometry { nodes, subtrees }` を `Metadata { nodes, subtrees, removed: {} }` に昇格してから今と同じ処理をする（`metadata_dirt` の `match` に `Geometry` の腕を足す）。
- `model/interact/geometry.rs` の geometry の変化（173-175 行、`self.a11y.subtree(change.id)`）と scroll（660 行）の 2 か所を `self.a11y.moved(..)` に変える。
- `build_semantics_for`（`a11y/build/dirty.rs`）に `Dirt::Geometry` の腕を足す。中身は今の `Metadata` の経路から「identity metadata を作り直して前回と比べる部分」を除いたもの：
  1. `let clipped = self.clipped_rows();`。`clipped != self.a11y.metadata.clipped` なら `Metadata { nodes, subtrees, removed: {} }` として今の経路で作る（自分を呼び直す）。
  2. 同じなら、accepted の metadata（`let metadata = Rc::clone(&self.a11y.metadata)`）を使い、`let order = self.semantic_order();`、`dirty = nodes.clone()`、`inherited` を今の経路の 76〜87 行と同じ loop で作る（`modal_changed` は false、`subtree_roots` は `subtrees`、親は `metadata.parents`）。
  3. 今の経路と同じく `expand_overlay_dirt(&order, &inherited, &mut dirty)`、`expand_relation_dirt(&metadata, &mut dirty)`、`invalidate_boxless_ancestors(&mut dirty)` を呼ぶ。
  4. `build_dirty_nodes(root, false, &dirty, &metadata, engine)`（関数は変えない）。focus は `semantics_focus(&metadata.clipped)`。`Built { tree, metadata: None }` を返す（`LocalText` と同じく accepted の metadata を再利用する）。
- 結果：geometry だけの変化で `build_identity_metadata`（全ノードの children の hash と BTreeMap への挿入、parents/children の全比較）を行わない。`semantic_order` と `build_dirty_nodes` の走査（軽い）は残る。

### 4.7 メモリの設計（MEM、R-8〜R-10）

#### MEM-1 疎な `Style`（kivact-style）

公開 API（`style().padding(24).gap(12)` などの builder の method 名と引数、`Style::merge`、`compile_style` 等の hidden bridge の署名）は変えない。内部表現だけを、**field の群ごとの疎な struct** に変える（Servo の style struct と同じ考え方）。操作の履歴は持たない。同じ field への上書きは値を置き換えるだけで、`merge` も今と同じ field ごとの合成である。

- 今の `Style` の約 75 field を、次の 8 つの群の struct に分ける。field 名・型・既定値・`merge` の規則は今のまま、所属だけが変わる。

| 群（`Style` の field） | struct | 入る field |
|---|---|---|
| `size` | `SizeFields` | width、height、min_width、min_height、max_width、max_height、basis、aspect_ratio、grow、shrink |
| `spacing` | `SpacingFields` | padding、margin、inset、row_gap、column_gap |
| `flow` | `FlowFields` | direction、align、align_self、justify、align_content、display、visibility、grid、wrap、position、overflow、scroll_axes、scroll_snap、z_index |
| `paint` | `PaintFields` | local_paint、background、border_width、border_color、border_style、ring、radius、shadows、opacity、isolation、border_image、scrollbar、cursor、clip、filter、backdrop_filter、blend、mask、clip_path |
| `transform` | `TransformFields` | translate、rotate、scale、transform、transform_origin、translate_z、rotate_x、rotate_y、transform_origin_z、perspective、perspective_origin、transform_style、backface、will_change |
| `text` | `TextFields` | font_size、weight、color、line_height、text_wrap、text_align、font_family、italic、text |
| `states` | `StateFields` | hover、group、group_hover、group_focus、pressed、focus、focus_visible、disabled、selected、enter、exit |
| `motion` | `MotionFields` | enter_size、exit_size、layout_transition、transitions |

```rust
/// An authored style: each group of properties is allocated only when one
/// of its properties is set. Values and merge rules are the dense style's.
#[derive(Clone, Debug, Default)]
pub struct Style {
    size: Option<Box<SizeFields>>,
    spacing: Option<Box<SpacingFields>>,
    flow: Option<Box<FlowFields>>,
    paint: Option<Box<PaintFields>>,
    transform: Option<Box<TransformFields>>,
    text: Option<Box<TextFields>>,
    states: Option<Box<StateFields>>,
    motion: Option<Box<MotionFields>>,
    /// Setter calls that replaced an earlier value (the diagnostics count).
    duplicates: u32,
}
```

- 各群の struct は `#[derive(Clone, Debug, Default, PartialEq)]` と `fn merge(&mut self, o: &Self)` を持つ。`merge` の本体は今の `Style::merge` と `merge_compound_properties`（`sheet.rs:1144`、`:1222`）の、その群の field の部分をそのまま移したもの。
- `Style` は群ごとに `fn size_mut(&mut self) -> &mut SizeFields { self.size.get_or_insert_with(Default::default) }` を持つ（8 つ）。setter は `self.width = ..` を `self.size_mut().width = ..` にする（duplicates の数え方も同じ式を群の field に対して行う）。
- `Style::merge(mut self, o: &Style) -> Style`：各群について、`o` がその群を持てば `self.<群>_mut().merge(o_group)`、持たなければ何もしない。`duplicates` の扱いは今の `merge` と同じ。
- 読み手（`resolve.rs`、`text.rs` など）は群ごとに読む。群が `None` なら既定値の群を読む：

```rust
let size_default;
let size = match style.size.as_deref() {
    Some(group) => group,
    None => {
        size_default = SizeFields::default();
        &size_default
    }
};
```

  この形を `macro_rules! group_or_default` に一度だけ書き、各読み手が使う。
- 入れ子の状態 style（`StateFields` の `hover`、`pressed`、`focus`、`enter`、`exit` など、型は `Option<Box<Style>>`）は新しい `Style` のまま持つ。
- 正しさの理由：dense な `Style` の各 field は、新しい `Style` の群の同じ field に一対一で対応する。群が無い状態は、その群のすべての field が既定値の状態と同じ。setter と `merge` は同じ式を同じ field に適用する。したがって `compile_style` が読む値は今と一致する。比較（`PartialEq`）は「群が無い」と「既定値の群がある」を区別するが、今の dense な比較と結果を一致させるため、`PartialEq` は derive せず「各群を、無ければ既定値として比べる」手書きの実装にする。
- 大きさ：`size_of::<Style>()` は 72 byte（8 つの box と `u32`）。box の大きさだけを設定する style は `SizeFields` 一つ（約 250 byte）を確保する。同じ field を何度上書きしても大きさは増えない。

#### MEM-2 `StateStyles` を必要な時だけ持つ（kivact-types、engine）

- `kivact-types/src/paint.rs` の `StateStyles` の各 field `Option<PaintDelta>` を `Option<Box<PaintDelta>>` にする（`hover`、`pressed`、`focus`、`focus_visible`、`group_hover`、`group_focus`、`enter`、`exit`）。`deltas()` は `.as_deref()` を通す。
- core は `tree.update(id, PropValue::States(..))` で状態 style を書き、engine の `model/states.rs:147` は受け取った値を常に `Some(states)` で保持する。engine 側で、`StateStyles::default()` と等しい値は `None` として保持する（`(states != StateStyles::default()).then_some(states)`）。engine はすでに `None` を「状態 style なし」として扱う（`n.states().is_some()`）。

#### MEM-3 既定値を保持しない（engine）

engine の `host/write/typed.rs:166-171` の `set_layout`・`set_paint` は `boxed(Some(v), ..)` で常に保持する。既定値と等しい値は `None` として保持する（`boxed((v != LayoutProps::default()).then_some(v), ..)`、Paint も同じ）。読み手が `None` を既定値として扱うこと（layout の `layout_props`、paint の `derive` は `PaintProps::default()` を使う）を、実装の unit（作業指示書 U34）が `grep` で確かめる。`None` を「まだ書かれていない」と別に扱う読み手があれば、その unit は STOP する。

#### MEM-4 大きな `Option` を box にする（MEM-2 規則の適用）

`kv_heap` の上位の割当て元に対して次の規則を機械的に適用する（作業指示書 U23、U34）：

| 規則 | 形 | 直し方 |
|---|---|---|
| M-R1 | `Rc<RefCell<Option<T>>>`、`Rc<Cell<Option<T>>>`、`RefCell<Option<T>>` で `size_of::<T>() > 64` | `Option<Box<T>>` を中に持つ |
| M-R2 | ノードごとの記録が既定値と等しい | 保持しない（`None`） |
| M-R3 | ノードごとに同じ不変の値（親から継承した `ComputedTextStyle` など） | 親の `Rc` を `Rc::clone` で共有する（親と等しいときだけ） |
| M-R4 | 長く生きる queue（event の queue など）で、mount の burst の後に大きな `capacity` が残る | drain の後、`capacity > 4 * max(len, 1024)` のときだけ `shrink_to(max(len, 1024))`（同じ規模の burst が続いても縮小と拡張を繰り返さない）。関数の中で作って捨てる一時の `Vec` には当てはめない |
| M-R5 | mount の後も全ノード分の要素を持ち続ける `Vec`（分析機では button 1 個あたり 14 KB の `Vec<SemanticsPatch>`） | 消してよいかは設計の判断なので、実装者は STOP で報告し、Orchestrator は未実施に記録する |

規則に当てはまらない上位の割当て元は「停止して報告」する。

#### MEM-5 `ComputedTextStyle` を共有する

`kivact-engine/src/measure/inherited_text.rs` の `computed_text` は全ノードに新しい `Rc<ComputedTextStyle>` を作る。`text_props(tree, id)`（祖先を一度だけ反復で辿る今の計算）の結果が、**既に cache にある**親の `computed_text` と等しければ、親の `Rc` を `Rc::clone` して保持する（M-R3）。親の cache が無ければ共有せず今どおり作る（親を再帰で計算しない。再帰すると深さ D の鎖で D² の計算と深い call stack になる）。等しさで判断するので、`capsize` のような継承しない規則があっても結果は変わらない。

#### MEM-6 部品の reactive node を減らす（R-10）

分析機の計測で、3000 行（各行に text と button）の画面は reactive node が 168,150（1 行に約 56）、`frame_loop` が 3,002 あった。主な原因は、button が busy を使わなくても `busy_animation`（`kivact-ui/src/button.rs:343`：`signal`、`memo` 2 つ、`frame_loop`、Machine と `feed` 2 つ）と `busy_indicator` を必ず作ることである。MEM-5 規則（実際に独立して変わる値の数まで）を適用し、`.busy(..)` が設定された button だけがそれを作るようにする（`Button.busy` を `Option<Value<bool>>` にする。作業指示書 U42）。part の状態はすでに一つの `PartState` の memo（`kivact-core/src/view/part.rs:72`）なので変えない。目標：busy でない button の `frame_loop` は 0、3000 行の `live_reactive` は B0 の 0.6 倍以下（`kv_latency` の `S stats_n3000` で測る）。

#### MEM-7 型の大きさの上限

各 crate の型の定義の直後に、コンパイル時の検査を置く。

```rust
const _: () = assert!(std::mem::size_of::<HostNode>() <= 384, "HostNode grew: box the new rare field");
```

| 型 | crate | B0 | 上限 | 意味 |
|---|---|---|---|---|
| `Style` | kivact-style | 1,928 | 72 | MEM-1 の結果を固定する |
| `StateStyles` | kivact-types | 6,024 | 80 | MEM-2 の結果を固定する |
| `HostNode` | kivact-engine | 464 | 464 | 増やさない（増やす field は `Option<Box<_>>`） |
| `NodeProperties` | kivact-engine | 240 | 240 | 同上 |
| `Look` | kivact-engine | 72 | 72 | 同上 |
| `NodeStyle` | kivact-layout | 264 | 264 | 同上 |

`HostNode` などは毎ノードに必要な field が大半で、inline の大きさを box で減らしても合計の memory は減らないため、上限は「今より増やさない」にとどめる。`LayoutProps`、`PaintProps`、`TextStyle`、`PaintDelta` は box で保持されるので上限を置かない。

### 4.8 バイナリ（BIN、R-11）

root `Cargo.toml` に次を足す。

```toml
[profile.release]
lto = "fat"
codegen-units = 1
opt-level = "s"
strip = "symbols"
# panic は unwind のまま: embed（DAW の plugin）は panic を host の外へ出さない。

# 描画・shaping・layout の hot path は速度を優先する。
[profile.release.package.vello_cpu]
opt-level = 3
[profile.release.package.vello_common]
opt-level = 3
[profile.release.package.fearless_simd]
opt-level = 3
[profile.release.package.kurbo]
opt-level = 3
[profile.release.package.skrifa]
opt-level = 3
[profile.release.package.read-fonts]
opt-level = 3
[profile.release.package.harfrust]
opt-level = 3
[profile.release.package.parley]
opt-level = 3
[profile.release.package.taffy]
opt-level = 3
[profile.release.package.kivact-layout]
opt-level = 3
[profile.release.package.kivact-engine]
opt-level = 3
[profile.release.package.arvane-render]
opt-level = 3
[profile.release.package.arvane-compose]
opt-level = 3
[profile.release.package.arvane-text]
opt-level = 3
```

- `panic = "abort"` にしない（embed の `catch_unwind` が使えなくなり、DAW を落とす）。
- `complex-scripts` は facade の既定 feature のまま（多言語対応の既定の挙動を変えない）。サイズを優先するアプリは `default-features = false, features = ["gpu"]` にできる。README と利用者ガイドに書く。
- **feature の転送**：`kivact` に `default-features = false` で依存し `gpu` を転送しているアプリ・雛形・試験の consumer（`templates/basic`、5 つの catalog、`dev/tests/docs`、`dev/tests/kivact-public-api`、`kivact-devtools`、`dev/bench`）は、`complex-scripts = ["kivact/complex-scripts"]` を足し、既定の feature に入れる（今の既定の分かち書きを保つ）。kit の library（`kivact-ui-*`）は feature を強制しない（選ぶのはアプリ）。
- **独立したアプリへの profile**：profile は workspace の root の manifest からしか読まれないので（`templates/basic/README.md:19`）、雛形の `Cargo.toml` に同じ `[profile.release]` を書き、配布のガイドに理由（サイズと速度、ビルド時間が延びること）を書く。fat LTO と codegen-units=1 の release ビルドの時間は `run.sh` が記録する（P-05 の全体の費用）。

### 4.9 その他の小さい改善

| ID | 内容 |
|---|---|
| TXT-1 | `arvane-text/src/cache.rs` の `TextKey` の検索で `String` と `Vec` を確保しない。`hashbrown` の `Equivalent` を使い、借用の `TextKeyRef<'a>`（`&str`、`&[..]`、`Option<&RichKey>`）で検索する。`TextKeyRef` は `TextKey` と同じ順の field を `#[derive(Hash)]` し、同じ hash を出す（`String` と `str`、`Vec<T>` と `[T]`、`Box<T>` と `T` の `Hash` は同じ）。確保は miss の時の insert だけ |
| EVT-1 | `EventSink` の queue（3000 ノードの mount 後に 3.8 MB の capacity が残った）に M-R4 を適用する。`model/drawables/place.rs:79` の `frames` は scene ごとの一時の `Vec` で残留しないので対象外 |

---

## 5. 公開 API

### 5.1 React・Solid との対応で守ること

| React / Solid | Kivact（本設計後） | 守ること |
|---|---|---|
| `function Card({ title, onClose, children })` | `#[component] fn card(title: Value<String>, children: View, #[prop(optional)] on_close: Option<Callback<()>>) -> impl IntoView` | props の struct と builder を手で書かない |
| `<Card title="x" onClose={...}>...</Card>` | `card("x", (..)).on_close(move \|\| ..)` | 必須の props は位置引数、任意の props は名前付きの setter |
| `useState` / `createSignal` | `signal(0)` | 変わらない |
| `onClick={() => ...}` | `.on_press(move \|\| ..)` | closure の引数の型を書かずに済む（payload ありの event も `\|e\| ..` で推論される） |
| `const [open, setOpen]` → `setOpen(o => !o)` | `open.update(\|o\| *o = !*o)`（`Signal` と `Binding` の両方） | `Binding::update` を足す |
| Testing Library の `getByRole` → locator | `screen.get_by_role(..)` は `Locator` を返す | 名前を Playwright に揃える |

handler の型は今の API-3 を維持する：payload のある event は `impl Fn(E) -> R + 'static`（`R: HandlerResult`）、payload の無い event は `impl Into<Callback<()>>`。一つの trait（marker 型を使う `IntoCallback<E, M>`）への統合は採らない。marker 型の trait 経由では closure の引数の型を推論できず（Bevy の `IntoSystem` と同じ制約）、`on_edit(|phase| ..)` に型注釈が要るようになり、React の利用者の書き心地が悪くなるため。代わりに、規則に従っていない部品（5.4 節の表）を直し、xtask の `api-shape` で `on_*` の署名を検査する。

### 5.2 `#[component]`（kivact-macros）

#### 書き方

```rust
use kivact::prelude::*;

/// A card with a title, an optional close button and its content.
#[component]
pub fn card(
    title: Value<String>,
    children: View,
    #[prop(optional)] on_close: Option<Callback<()>>,
    #[prop(default = 12.0)] gap: Value<f64>,
) -> impl IntoView {
    let close = on_close.map(|on_close| button("Close").on_press(on_close));
    column((row((text(title), close)), children)).style(move || style().gap(gap.get()))
}

// 使う側（必須は位置引数、任意は setter）
card("Settings", (text("A"), text("B"))).on_close(move || open.set(false))
card("About", text("Kivact"))   // on_close と gap は既定値
```

#### 展開の規則

1. 関数の位置にそのまま、同じ名前・同じ可視性の関数を生成する。引数は「`#[prop(..)]` の無い引数」を宣言順に並べたもの。戻り値は生成する struct。
2. struct の名前は関数名を UpperCamelCase にしたもの（`card` → `Card`、`user_row` → `UserRow`）。`#[must_use]` を付ける。関数の doc comment を struct にも写す。
3. 任意の props（`#[prop(optional)]` と `#[prop(default = expr)]`）は struct の method（setter）になる。名前は引数名。
4. 引数の宣言型と、生成する引数・setter の受け取り型・保持する型・変換。**owner の要らない変換は呼ばれた時に行い、owner の要る変換だけを mount の時（`scope` の中）に行う。** 通常の部品（`text`、`button` など）と同じ入力を受け取る（`&str` も受け取れる。`'static` を要求しない）。

| 宣言型 | 受け取る型 | 保持する型（呼ばれた時の変換） | mount の時 |
|---|---|---|---|
| `Value<String>` | `impl ::kivact::state::IntoText` | `Reactive<String>`（`Reactive::from_text(x)`、owner 不要） | `.into_value()` |
| `Value<T>`（T が String 以外） | `impl ::kivact::state::IntoValue<T>` | `Reactive<T>`（`Reactive::new(x)`、owner 不要） | `.into_value()` |
| `Binding<T>` | `impl ::kivact::state::IntoBinding<T>` | `Binding<T>`（`into_binding()`、owner 不要） | そのまま |
| `Callback<()>` | `impl Into<::kivact::state::Callback<()>> + 'static` | `Box<dyn FnOnce() -> Callback<()>>`（変換しない） | `f()`（ここで `into()`） |
| `Callback<E>`（E が `()` 以外） | `impl Fn(E) -> R + 'static`（setter に型引数 `R: HandlerResult`） | `Box<dyn FnOnce() -> Callback<E>>` | `f()`（ここで `callback(..)`） |
| `View`（一度 mount する slot） | `impl ::kivact::view::IntoView` | `View`（`into_view()`、owner 不要） | そのまま |
| `Vec<View>`（複数の子、KIT-5） | `impl ::kivact::view::IntoChildren` | `Vec<View>`（`into_views()`） | そのまま |
| `ViewFn`（開くたびに作り直す content、API-6） | `impl ::kivact::view::IntoViewFn<V>`（生成する関数・setter に型引数 `V: IntoView`） | `ViewFn<()>`（`into_view_fn()`） | そのまま |
| `Option<X>`（`#[prop(optional)]` のみ） | X の受け取る型 | `Option<X の保持する型>`、既定は `None` | X と同じ |
| その他の `T` | `impl Into<T>` | `T`（`into()`） | そのまま |

   callback だけは、closure 自体が `'static`（handler の規則）なので `'static` を付けても利用者の負担は増えない。
5. `#[prop(default = expr)]` の既定値は、上の表の「呼ばれた時の変換」に `expr` を通した値。`expr` は位置引数の関数が呼ばれた時に評価する。
6. 生成する関数と setter に `#[track_caller]` を付ける（API-12）。
7. `IntoView` の実装は次の形。

```rust
impl ::kivact::view::IntoView for Card {
    fn into_view(self) -> ::kivact::view::View {
        let Card { title, children, on_close, gap } = self;
        ::kivact::view::scope(move || {
            let title = ::kivact::state::IntoValue::into_value(title);
            let gap = ::kivact::state::IntoValue::into_value(gap);
            let on_close = on_close.map(|f| f());
            /* 元の関数本体（children は ViewFn のまま） */
        })
    }
}
```

8. 足す公開 API（runtime と core、W2 の U25）：`impl Reactive<String> { pub fn from_text(input: impl IntoText) -> Self }`（owner 不要、`into_text()` を包む）。`impl IntoViewFn<View> for ViewFn<()>`（と sealed の impl）。これで本体は受け取った `ViewFn` を `show(open, children)` にそのまま渡せる（`show` は `IntoViewFn` を受け取る、`view/flow/branch.rs:168`）。
9. 診断（`compile_error!`）：型引数や lifetime を持つ関数、`impl IntoView` 以外の戻り値、`self` を取る関数、`#[prop]` の未知の key、`optional` で `Option` でない型、`optional` と `default` の併用。各 message は修正方法を一文で示す（API-12 と同じ）。

#### 適用範囲

- アプリ、kit、catalog、例が使う。`kivact-ui`・`kivact-interaction` の内部は facade に依存しないので使わない（今の具体 builder を保つ）。
- 生成された struct は `Element`（下記）を実装しない。React の component と同じく、外から style を当てたいなら props で受ける。
- `examples/kivact/component` と `templates/basic` と利用者ガイドの「部品を作る」を `#[component]` で書き直す。`examples/kivact/component` の `card` は今 `show(open, children)` で開閉のたびに children を作り直し（`main.rs:22`）、Collapse の後の Expand を試験している（`main.rs:77`）。書き直しでは children を `ViewFn` で宣言し、この契約と試験をそのまま保つ。kit の既存の公開 builder は `#[component]` に書き換えない（kit の公開 API の形を本 refactor の規約以外の理由で変えないため）。

### 5.3 一つの名前に一つの意味（改名）

| 旧 | 新 | 理由 |
|---|---|---|
| trait `NodeBuilder` | trait `Element` | 「style・test id・意味情報を設定できる、view の要素」。React の element と同じ語。`node_mut` 等の method 名は変えない |
| trait `Component`（= `IntoView + NodeBuilder`） | 削除。`impl Component` は `impl Element`（`Element` の supertrait を `IntoView` にする） | 「component」は `#[component]` の意味に一本化する |
| 関数 `component(move \|\| ..)`（setup の境界） | `scope(move \|\| ..)` | `#[component]` と同名で別の意味になるため。owner の境界であることを名前で示す |
| `kivact::test::Element<'a>` | `kivact::test::Locator<'a>` | `Element` trait と衝突する。Playwright の語 |
| `kivact::view::IntoViewFn` | そのまま | API-6 の「作り直す content」 |

prelude：`Element`、`scope`、`component`（attribute macro）を入れ、`NodeBuilder`、`Component`、関数 `component` を除く。`ButtonVariant` を入れる（入門ガイドで使う）。

### 5.4 引数の規約の統一

#### 規約

| ID | 規約 |
|---|---|
| ARG-1 | payload の無い event の setter は `impl Into<Callback<()>>`、payload のある event は `impl Fn(E) -> R + 'static`（`R: HandlerResult`）。`impl Fn() -> R`、`impl Fn(K) + 'static`（`HandlerResult` なし）は使わない |
| ARG-2 | 項目の集まりの位置引数の名前は `items`（`options`、`entries`、`nodes` を使わない） |
| ARG-3 | 0 個か 1 個（か複数）を選ぶ部品の選択は位置引数 `selected: impl IntoSelection<K>`（単一だけの部品は `impl IntoBinding<Option<K>>`）。必ず一つを選ぶ部品（tabs、segmented）は位置引数 `value: impl IntoBinding<K>` |
| ARG-4 | 開閉。開いていない状態が無意味な部品（dialog、window、drawer、sheet）は位置引数 `open: impl IntoBinding<bool>`。開閉が任意の部品は setter `.open(impl IntoBinding<bool>)`。`visible` という名前で開閉を表さない |
| ARG-5 | 項目の活性化（menu の項目を選んで実行する、tree の行を開く等）の event は `on_activate`。`on_select`、`on_pick`、`on_item_activate` を使わない（選択は binding が正本で通知を足さない、既存の規則） |
| ARG-6 | 状態を変える命令の method（`self` を取り何かを開く）は動詞 `show`/`hide`。setter（binding を受ける）と同じ名前にしない |

#### 直す対象（API の調査で見つかったもの）

| 対象 | 今 | 直した後 |
|---|---|---|
| `kivact-ui` `item_row.on_press` | `impl Fn() -> R` | `impl Into<Callback<()>>`（ARG-1） |
| `kivact-ui` `breadcrumb.on_activate` | `impl Fn(K) + 'static` | `impl Fn(K) -> R + 'static`（ARG-1） |
| `kivact-ui` `navigation_menu.on_open_change`、`stepper.on_step_press` | `HandlerResult` なし | ARG-1 |
| `kivact-ui` `chat_composer.on_submit` | `Fn(String) -> R` | そのまま（payload あり、ARG-1 に合う）。`text_input.on_submit` は payload 無しで ARG-1 に合う |
| `kivact-ui` `select(label, options, selected)` | 引数名 `options` | `items`（ARG-2） |
| `kivact-ui` `menu(.., entries)`、`tree(.., nodes)` | 引数名 | `items`（ARG-2） |
| `kivact-ui` `radio_group(.., value: IntoBinding<Option<K>>)` | 引数名 `value` | `selected`（ARG-3。単一だけなので型はそのまま） |
| `kivact-ui` `list(label, items).selection(..)` | setter | 位置引数 `list(label, items, selected)`（ARG-3） |
| `kivact-ui` `alert.visible(binding)` | `visible` | `.open(binding)`（ARG-4。既存の `.open` と重なる場合は一つにまとめる） |
| `kivact-ui-game` `icon_menu.on_select` | | `on_activate`（ARG-5） |
| `kivact-ui-creative` `SplitButton.on_select`、`picker` の `on_pick`、`outliner` の `on_item_activate` | | `on_activate`（ARG-5） |
| `kivact-ui-soft` `LightboxHandle::open(self, index)`、`mobile_nav.open(self)` | 命令 | `show(index)`、`show()`（ARG-6）。`lightbox(open, media)` の builder（`open` は位置引数の bool の binding、ARG-4 に既に合う）と `LightboxState` の非公開は変えない |
| `kivact-ui-soft` `mobile_nav.width(IntoValue<f64>)` | | `IntoValue<Length>`（`layout.width` と同じ） |
| `Binding` | `update` が無い | `pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R`。`Signal` から作った binding は `Signal::update` に委ねる（その場で変更し、同じ値でも通知する。`handles.rs:148`）。`binding(get, set)` で作った binding は、`get_untracked()` の値を clone して `f` を適用し、`set` を一度呼ぶ（通知するかは `set` が決める。setter が正規の書込み口であることを保つ）。rustdoc にこの二つの違いを書く |

kit の他の公開 setter も ARG-1〜6 に照らし、合わないものは同じ表の形で直す（作業指示書 U51〜U55 の手順）。

#### 文書の誤りの修正

API の調査で見つかった、文書とコードの不一致を直す（W7）：`provide_*` の所在（theme は `kivact::style`、locale は `kivact::i18n`）、dialog の catalog の説明（`.content`/`.footer`）、`node_mut` の扱い（`Element` を実装するときに書く method として説明し、doc(hidden) の理由を併記）、prelude にある名前の重複 import、`.to_owned()` の不要な例、`kivact::view::InputPurpose::Password` の完全 path（prelude にある）。

---

## 6. 暗黙の状態機械の解消

### 6.1 方針

1. **監査が主、lint は補助。** 名前と型の構文を見る lint は、関数・object・crate をまたぐ意味の制約を網羅できない。そこで、状態の持ち主・不変条件・有効な組合せ・event・初期化・期限・破棄を、作業指示書の付録 A に**直した後の型まで設計者が確定した表**として持つ。実装者はその表を写すだけにし、確かめてから設計する手順（「確かめる、合わなければ STOP」）を残さない。
2. lint の抜け穴を塞ぐ（6.2 節）。移行中は新しい検査を `state-fields-strict` という別名の rule にし、各 wave の unit が自分の crate で 0 件にする。W6 で `state-fields` に統合し、`state-fields-strict` を消す。
3. 付録 A に無いもので lint が見つけたものは、6.3 節の規則のうち形がそのまま当てはまるものだけを実装者が直す。当てはまらないもの、組合せの意味を読まないと決められないものは STOP し、設計者（人間）が付録 A に行を足して確定する。STOP の項目は「未実施」として完了扱いにしない（1.6 節 AC-3）。
4. 閉じ方は四つだけ：**型で表す**（enum、単一の Machine）、**導出して保持しない**、**単一の書き手の投影**（SM-12）、**真に独立**（SM-11、すべての組合せが有効と示せるもの）。
5. 限界：静的な検査と台帳では、意味の上で隠れた状態を「全て」見つけたとは証明できない。完了の条件は「既知の項目が 0 件」と「監査の報告（6.4 節）を人間が確認した」ことである。
### 6.2 lint の変更（`dev/xtask/src/state_fields/`）

| ID | 変更 |
|---|---|
| SL-1 | `OnceCell<T>`、`OnceLock<T>`、`LazyCell`、`LazyLock` を stored に数える |
| SL-2 | phase として数える workspace の enum の名前の接尾辞を `State`・`Phase` から `State`・`Phase`・`Status`・`Stage`・`Connection`・`Lifecycle`・`Run`・`Present` に広げる（`Connection`、`ScriptRun`、`LastPresent` を捕まえる。`Align` のような値の enum は数えない） |
| SL-3 | named helper（`Generation`、`Sent`、`Revision`、`Epoch`）は、同じ struct に他の stored field があるときは stored に数える |
| SL-4 | role の接尾辞による除外（`Input`、`Report` など）は、`Cell`・`RefCell`・`Signal`・`Var`・`Mutex` を field に持つ struct には適用しない |
| SL-5 | `kivact-engine::contract` の除外をやめる（`bool` と `Option` を持つ struct を検査する） |
| SL-6 | 名前が `pending*`・`dirty*`・`awaiting*`・`relayout*`・`stale*` の `Vec`・`VecDeque`・`BTreeSet`・`HashSet` を stored に数える |
| SL-8 | marker の理由は `independent:`、`projection:`、`cache:`、`identity:`、`statistic:`、`option:` のどれかで始める（それ以外は error）。`independent:` は「全組合せが有効」の理由を、`projection:` は書き手の関数名を書く |
| SL-7 | 関数内の local（`signal(false)`、`signal(None)`、`signal(<整数>)`、`var(..)`、`Cell::new`、`RefCell::new(None)`、`Rc::new(Cell::new(..))`、`Rc::new(RefCell::new(None))`）が同じ関数に 2 個以上あるとき、`--report` だけでなく error にする。例外は local の直前の `// xtask-allow(state-fields): <理由>` |

### 6.3 変換規則

| ID | 見つかった形 | 直した形 |
|---|---|---|
| SM-1 | 二つ以上の `bool` が同時に true にならない | 一つの enum。variant 名は状態の名前（`Idle`、`Running`、`Paused` など） |
| SM-2 | `bool` と、その bool が true のときだけ意味のある `Option<T>` | `enum { Off, On(T) }`（名前は意味に合わせる） |
| SM-3 | 二つの `Option` がともに `Some` かともに `None` | `Option<(A, B)>` または意味のある struct の `Option` |
| SM-4 | 二つの `Option` が排他 | `enum { None, A(A), B(B) }` |
| SM-5 | 構築後に埋める slot（`OnceCell`、`Option` を後で `Some` にする） | constructor に渡す（S-7）。渡せない循環は `Rc::new_cyclic`。どちらも無理なら停止して報告 |
| SM-6 | 古い仕事を捨てるための `Generation`/`epoch` の比較 | 仕事の識別子を `Running { id }` の variant に持たせ、完了は id の一致で受ける |
| SM-7 | 期限の時刻を保存し、読む時に比べる（`Warm { until_ns }`） | Machine の `Step::after(期間, Event::Expire)` |
| SM-8 | effect が別の signal を補修する（`effect(move \|\| if !a.get() { b.set(None) })`） | 二つを一つの Machine の状態にし、effect を消す |
| SM-9 | caller の `open: Binding<bool>` と部品内の Machine が同じ開閉を持つ | Machine が正本。(1) Machine の初期状態は構築時の `open.get_untracked()` から作る（`feed` は最初の値を配らない、`machine.rs:237`）。(2) 以後の変化は `feed(open, Event::OpenRequested)` で入れる。今の状態と同じ値の event は `stay`。(3) Machine の開閉の遷移は `Effect::WriteOpen(bool)` を出し、effect は `open.get_untracked() != v` のときだけ `open.set(v)` を呼ぶ（書き戻しは feed で同じ値として戻り `stay` になるので再入しない）。(4) owner の破棄で Machine も破棄される。E2E：初期 `open = true` で開いて描かれる、外から binding を変えると開閉する、部品の操作で閉じると binding が false になる |
| SM-10 | drag の session と並べて、同じ gesture を別の signal で追う | drag の Start/Update/End/Cancel を一つの domain の Machine に送る |
| SM-11 | 本当に独立した事実（すべての組合せが有効） | 直さない。field の直前に `xtask-allow(state-fields): independent: <すべての組合せが有効である理由>` を書く。「排他でない」は独立の理由にならない（一方が他方を含意するなら SM-1・SM-2）。field 名の繰り返しは理由として認めない |
| SM-12 | 同じ事実の公開の投影（通知の粒度のために別の signal に持つ等） | 書き手を一つの private な関数（例：`publish(next)`）に集め、その関数だけが両方を書く。marker は `projection: written only by <関数名>` |

SM-11 の「本当に独立」は、「すべての組合せが有効である」ことを理由に書けるものに限る。

---

### 6.4 状態の監査（受入れ）

- `cargo xtask check --only state-fields --report-audit`（W6 の U61 が作る）は、state の lint が数える field を 2 つ以上持つ struct・variant・関数の local を、field と marker の理由と閉じ方（SL-8 の接頭辞）ごとに `dev/xtask/state-audit.md` に書き出す。
- CP3 で、この報告に「閉じ方が無い」行が 0 件で、付録 A の全行が実装済みであることを確かめる。人間（設計者）がこの報告を読んで確認するまで、1.6 節 AC-3 は満たされない。

## 7. 検証

| 段階 | 行うこと | 行わないこと |
|---|---|---|
| unit の完了 | `cargo fmt`、指定された crate の `cargo check`/`cargo clippy -p <crate> --all-targets -- -D warnings`、指定された `cargo xtask check --no-write --only <rule> --crate <crate>`、本書・作業指示書が名前を挙げた試験 | workspace 全体の test、E2E の一括実行 |
| checkpoint（CP1、CP2、CP3） | 作業指示書 9 章の gate と `dev/bench/run.sh` | — |
| 最終（CP3） | 上記に加えて feature の組合せ、WSL の Linux の gate（実機があれば）、1.6 節の受入れの判定 | — |

- 性能の変更で振る舞いは変わらない。既存の E2E がそのまま通ることを確かめる。期待値を緩めて通すことはしない。
- **feature の組合せ**：`cargo test --workspace --no-default-features` は、他の package の依存が facade の既定 feature を有効にするので GPU なしの確認にならない。package を一つずつ指定して確かめる：`cargo check -p kivact --no-default-features`、`cargo check -p kivact --no-default-features --features gpu`、`cargo check -p kivact --features gpu-interop`、`cargo test -p kivact-public-api-tests --no-default-features`（この package は facade の feature を転送する）。加えて `cargo tree -p kivact --no-default-features -e features -i arvane-compose` の出力に `"gpu"` が無いことを確かめる。
- **新しい E2E**（`dev/tests/kivact-public-api/tests/e2e/`）：
  - `#[component]`：必須・任意・既定値の props、`&str` の引数、`Binding`・`Vec<View>`・`ViewFn` の props、payload ありの callback の型推論、root の setup の外で作った builder を mount すること、`ViewFn` の children を閉じて開くと作り直されること、unmount で component の owner が破棄されること（`Screen::stats().live_owners` が元に戻る）。
  - `Binding::update`：`Signal` 由来と `binding(get, set)` 由来の両方。
  - **資源の回収**：3000 行の list を mount して unmount し settle した後、`Screen::stats()` の `live_reactive`・`live_owners`・`frame_loops`・`tree_nodes` が mount の前の値に戻ること（G-01/G-04、R-3 の flat cache と PNT-1 の facts の eviction を含む）。
  - **GPU の OnDemand**（GPU のある機械だけ、GPU の lane）：shader の composition で GPU が始まること、shared image（gpu-interop）を作ると GPU が始まり GPU frame が届くこと、`ARVANE_RENDERER=cpu` で始まらないこと。
  - **SVG の埋込み文字**：外側に `data:image/svg+xml` の image だけがあり、内側に text がある SVG を描き、text の無い同じ SVG と画素が異なること（`Screenshot::raster_difference`）。
- 型の大きさは MEM-7 の `const` 検査がコンパイル時に確かめる。

## 8. 決定、非目標、危険

### 8.1 本書が置き換える決定

| 旧 | 新 |
|---|---|
| CONTRACT-01（固有 option を持たない合成部品は `impl Component` を返す） | `impl Element` を返す。アプリの部品は `#[component]` で `impl IntoView` を返す |
| ADR-47（`view::component(FnOnce→IntoView)`） | 名前を `view::scope` に変える。意味は同じ |
| ADR-34（`NodeBuilder.node_mut`） | trait 名を `Element` に変える。意味は同じ |
| Arvane の既定（起動時に GPU を始める） | `GpuStart::OnDemand` を既定にする |

### 8.2 新しい決定（W7 で decisions.md に足す）

| ID | 問い | 決定 | 理由 |
|---|---|---|---|
| SLIM-01 | 性能の目標をどう表すか | K1〜K6 を `dev/bench` で測り、同じ機械の B0 に対する倍率で判定する | 実装する機械が分析した機械と異なるため |
| SLIM-02 | GPU をいつ起動するか | 必要時（shader、GPU export、CPU が周期の半分を 8 frame 続けて超える） | 業務画面では GPU のドライバのメモリが WS の大半を占め、CPU 合成は高速経路で十分速い |
| SLIM-03 | handler の型を一つの trait にするか | しない。API-3 を全部品に守らせる | marker 型の trait では closure の引数の型が推論されない |
| SLIM-04 | props の builder をどう作るか | `#[component]` が関数から生成する | 手書きの builder（約 80 行）は Rust 未経験者の障壁 |
| SLIM-05 | 名前の衝突をどう解くか | `NodeBuilder`→`Element`、`Component` 削除、`component()`→`scope()`、試験の `Element`→`Locator` | 一つの名前に一つの意味（P-02、6 章の 4） |
| SLIM-06 | `complex-scripts` を任意にするか | facade の既定 feature のまま任意にする | 多言語の既定の挙動を保ちつつ、3 MB を外せるようにする |
| SLIM-07 | release の profile | fat LTO、codegen-units=1、opt-level="s"、hot path の crate だけ 3、strip、panic は unwind。雛形にも同じ profile を書く | サイズと速度の両立。embed の安全。profile は root の manifest からしか読まれない |
| SLIM-08 | authored `Style` の表現 | 8 つの群の疎な struct（操作の履歴を持たない） | 履歴の再生は merge で指数的に増え、上書きで肥大する |
| SLIM-09 | GPU の需要をどう届けるか | shader は composition から、GPU 共有と GPU export は core が `HostRequest::DemandGpu` で明示する。起動は presenter の `Gpu` だけが持つ | GPU 共有は GPU の準備の後でないと composition に現れない |
| SLIM-10 | 完了の判定 | 1.6 節の AC-1〜AC-4 をすべて満たしたときだけ完了。未達・未実施は報告して止める | 「試して報告した」と「達成した」を区別する |

### 8.3 危険と対策

| 危険 | 対策 |
|---|---|
| 差分の丸め（LAY-1）が taffy の丸めと 1 px ずれる | 計算部分を taffy から写し、累積座標の扱いを同じにする。既存の layout の E2E と `assert_layout_clean` が検出する |
| facts の差分更新（PNT-1）で印の付け漏れ | debug build の `debug_assert_eq!(facts, layer_facts())` が試験で検出する。漏れが見つかった箇所は `All` にして構わない（正しさを優先） |
| GPU の遅延起動で shader の初回が遅れる | 初回の frame は CPU の fallback を描き、GPU の準備後に shader を描く。`GpuStart::AtLaunch` を選べる |
| CPU 合成の高速経路の丸め差 | 汎用 loop との差は ±1。画素比較の試験の許容差の範囲 |
| `#[component]` の診断が分かりにくい | `compile_error!` の文言で直し方を示す。展開の規則を利用者ガイドに書く |
| 改名の取りこぼし | W3 の一括置換の後に workspace 全体の `cargo check` を CP で行う |
| Haiku 相当の実装者が規則に無い判断をする | 作業指示書 0.4 の「停止して報告」。設計の判断は設計者（人間）が付録 A・本書に書き足して行い、Orchestrator は設計の判断をしない |
| OnDemand の初回 GPU の切替で操作が止まる | device と pipeline は worker で作り、その間は CPU で描き続ける。`renderer=gpu init_ms=..` を `run.sh` が記録する |
