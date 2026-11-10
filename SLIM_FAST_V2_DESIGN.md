# Kivact Slim-Fast V2 設計書

版 2.0（2026-11-10）。作業ブランチ `refactor/kashiwade/slim-fast`、基点 HEAD `73c36c7c`（= `db8d679b` W7-1 の判定 + 引継ぎ文書）。

手順は [SLIM_FAST_V2_WORK_ORDER.md](SLIM_FAST_V2_WORK_ORDER.md)（V2 作業指示書）に書く。本書は「何を、なぜ、どの形にするか」と、付属書どうしの調整と、設計者の判断の結果を持つ。各変更の型・関数・書き手・呼出元・試験の詳細は付属書 `slim-fast-v2/D1〜D8` にある。実装者は本書と付属書の決定を変えない。

---

## 0. 本書の扱い

### 0.1 正本と優先順位

衝突したら上が優先する。

1. 本書（V2 設計書）。特に 4 章（決定）と 5 章（付属書間の調整）は、付属書の記述を上書きする。
2. 付属書 `slim-fast-v2/D1〜D8`（下の表）。付属書の「8 章 設計者（人間）の判断が要る点」は、すべて本書 4 章で決めた。付属書の 8 章の「推奨」「選択肢」を実装者が読み直して選ぶことはしない。
3. 旧設計書 [SLIM_FAST_DESIGN.md](SLIM_FAST_DESIGN.md)（版 1.1）。本書と付属書が触れない事項についてだけ有効（0.2 前提、1.1 計測の定義、1.4 必達、2.2 規則、5 章の公開 API、6.1〜6.3 の状態の規則、8 章の決定）。版 1.1 の節のうち本書が置き換えたものは 2.5 節の表にある。
4. `docs/for-kivact-developer/architecture.md`、`decisions.md`、`strategy.md`。

旧作業指示書 [SLIM_FAST_WORK_ORDER.md](SLIM_FAST_WORK_ORDER.md)・`20261109_2107_CODEX_REVIEW.md`・`20261110_1145_CLAUDE_CODE.md` は履歴として残す。手順には V2 作業指示書だけを使う。

### 0.2 付属書

| 付属書 | 範囲 | 主な KPI・AC | 付属書の中の unit の名前 → V2 の unit ID |
|---|---|---|---|
| [D1-frame.md](slim-fast-v2/D1-frame.md) | frame・turn・layout・paint・compositor・試験 driver の O(N) の除去（a11y を除く） | K3、K4、K5（counter click、更新の非 a11y 部分） | P1-U1〜U7 → F1〜F7 |
| [D2-a11y-hit.md](slim-fast-v2/D2-a11y-hit.md) | a11y の構築・公開を O(changed) に、hit test の枝刈り | K5（pointer、wheel、更新の a11y 部分） | P2-U1a・U1b・U2・U3 → AH1・AH2・AH3・AH4 |
| [D3-memory-mount.md](slim-fast-v2/D3-memory-mount.md) | ノード・部品ごとのメモリ、mount、U23/M-R5、kv_heap の診断 | K2（item、3000 行）、K5 mount、AC-4（U23） | U-M1〜U-M12 → M1〜M12（M4・M5 は取消し、5.1） |
| [D4-size-idle.md](slim-fast-v2/D4-size-idle.md) | バイナリサイズ、idle の WS の基礎分 | K6、K2（counter） | P4-U1〜U9 → S1〜S9 |
| [D5-state.md](slim-fast-v2/D5-state.md) | AC-3：OPEN 25 件、旧付録 A の照合、lint の穴（X1〜X6） | AC-3 | 6 章の lane の unit 名をそのまま使う（XT-1、LY、ST・ED、NA、AS、EN-1、EN-2、KT、RT、CO、IN-1〜3、UI-1〜4、SO、CR、EX、DT、FINAL-1、FINAL-2） |
| [D6-gpu-api.md](slim-fast-v2/D6-gpu-api.md) | GPU OnDemand の修正と E2E（U56）、`Windows.gpu`/`Driver.gpu`、公開 API のレビュー（C-1〜C-17） | AC-4（U56）、AC-3（2 件）、公開 API | P6-1〜P6-9 → G1〜G9 |
| [D7-text-memory.md](slim-fast-v2/D7-text-memory.md) | text item を ≤ 3.64 KiB にする追加設計（D3 の上に積む） | K2（text、button・row の余裕） | U-T1〜U-T9 → T1〜T9 |
| [D8-state-sweep.md](slim-fast-v2/D8-state-sweep.md) | `CheckableState` の排他の型化（`CheckControl`）、workspace の全 marker 917 行の判定 | AC-3 | P8-* の名前をそのまま使う |

付属書の行番号は HEAD `73c36c7c` のもの。前の unit がファイルを変えると行はずれる。実装者は関数名・型名・field 名で場所を探す（作業指示書 0.2）。

### 0.3 変わらない前提（版 1.1 の 0.2 を引き継ぐ）

- 互換は残さない（wrapper、alias、旧名、二重 mode、deprecated を作らない）。in-repo のすべての呼出し元（kit、catalog、例、試験、文書）を同じ変更で直す。
- CI の機能は追加しない（`dev/bench`・`dev/tools` の script は手動の道具）。
- 状態機械と振る舞いの検証は E2E（実 window、`kivact::test`）で行う。付属書が明示した engine・runtime・xtask の既存の単体試験の形（`#[cfg(test)]` の module に足す試験）だけは例外として足してよい。
- kit を縮小しない。既存の E2E の期待値を緩めない。
- モジュール分割よりクレート分割。ただし境界が明確な場合だけ。V2 は新しい crate を作らない（3 章）。

---

## 1. 現状と V2 の目的

| AC | 現状（W7-1、`dev/bench/results/RESULT.md`） | V2 で行うこと |
|---|---|---|
| AC-1 正しさ | CP3 の gate は成功 | 付属書の「既存欠陥」（6 章）を直し、V2 の最後に同じ gate を通す |
| AC-2 KPI | K2・K3/K4（3000 行）・K5 の大半・K6 が必達未達 | 2 章の原因を直す。見積りはすべての必達行を満たす（2.3 節） |
| AC-3 状態 | `dev/xtask/state-audit.md` に OPEN 27 件。lint の gate と audit が食い違い、偽の marker が多数 | OPEN 27 件を閉じ（D5・D6）、偽の marker 70 行と誤った理由文 284 行を直し（D8）、lint の穴を塞ぐ（D5 X1〜X6） |
| AC-4 未実施 | U23（M-R5）と U56（GPU OnDemand の E2E）が STOP | U23 は D3 M2-7（と M2-12）で確定。U56 は D6 で試験 API と E2E を確定し、GPU OnDemand の欠陥 D1〜D7 を直す |

AC の定義（版 1.1 の 1.6 節）は変えない。完了の判定は V2 作業指示書の W8。

---

## 2. KPI と計測

### 2.1 目標

必達の倍率は版 1.1 の 1.4 節のまま変えない。B0 と B1 は**同じ機械**で取る。V2 を実装する機械で、基点 `79bbd409749e49dfdc581c7c6497eac1ac52ca92` の B0 を取り直す（`B0-here`、作業指示書 W0）。元の機械の B0（`dev/bench/results/B0.md`）と B1（`B1.md`）は未達の証拠として残し、判定には使わない。

### 2.2 計測の定義の変更

| ID | 変更 | 理由 |
|---|---|---|
| MS-1 | K6 の `kv_startup --no-default-features` の行は、facade の最小構成（GPU なし・`complex-scripts` なし）で測る（D4 D5：`dev/bench/Cargo.toml` の `kivact` 依存から `features = ["gpu"]` を外し、bench の既定 feature に `gpu` を置く）。B0 の行は版 1.1 の表のとおり「B0 の同じ build の既定」 | 必達値（約 11 MB）は GPU なし・`complex-scripts` なしの構成から算出された。今の bench は GPU を固定して測っており、目標の根拠と食い違っていた（D4 E-4） |
| MS-2 | `run.sh` が `kv_heap` の PDB も複製する。`kv_heap` の site の選び方を直し、`kv_stress` と同じ row の定義にする（D3 M2-15） | site 名が空・誤帰属だった。診断値で、KPI の行ではない |
| MS-3 | K2 を単独で測るときは、build 直後の最初の起動を捨てる（`dev/bench/README.md` に 1 行、D4 E-5） | 初回だけ exe の image 全体が WS に入る。`run.sh` は K1 の 5 回の後に K2 を測るので正式値には影響しない |
| MS-5 | K6 の既定 feature の 2 行（counter と basic catalog）は、それぞれ 2 行にする（DEC-01）。(1) **GPU 抜きの build**：counter は `kv_startup`（counter と同じ画面）を `-p kivact-bench --no-default-features --features complex-scripts`、catalog は `-p kivact-ui-basic-catalog --no-default-features --features complex-scripts` で build したもの。必達は版 1.1 のまま 1.7 倍（B0 は既定の build：counter 24,594,432 B → ≤ 14,467,313 B、catalog 54,251,520 B → ≤ 31,912,659 B）。(2) **既定の build（`gpu` 込み）**：必達は「V2 の前より大きくしない」（≤ W0 で測る `B1-start` の値。元の機械の B1 は counter 18,978,816 B、catalog 43,090,432 B）。`run.sh` に (1) の 2 つの build と `K6 counter_nogpu`・`K6 basic-catalog_nogpu` の行を足す（M12） | 1.7 倍は誤った見積り（2.4 節）から決められ、`gpu` を既定に残すと届かない（GPU 一式 約 4.6 MB）。製品の既定を変えずに、Kivact 自身の大きさは 1.7 倍の目標で測り、GPU 込みの既定の build は悪化しないことを測る。B0 との比べ方は `kv_startup` の no-default の行と同じ（B0 は既定の build） |
| MS-4 | 必達に届かない行は同じ条件で 2 回測り直し、3 回中 2 回で判定する（版 1.1 の 1.6 AC-2 のとおり）。W7-1 の「再計測しない」は誤りだった | 雑音の扱い。WS は同じ exe でも回により item あたり ±0.25 KiB 動く（D7 H-P7-1） |

### 2.3 見積り（付属書の試作の実測と見積り）

B1 は元の機械の値。V2 の見積りは付属書のこの機械の試作の実測（「実測」）か、そこからの見積り。最終判定は W8 の `B0-here` と `B1-here`。

| KPI | 行 | 必達の上限（元の B0 から） | B1 | V2 の見積り | 根拠 |
|---|---|---|---|---|---|
| K2 | counter idle WS default / cpu | ≤ 27,214 / ≤ 26,694 KiB | 34,912 / 34,968 | 25,844〜25,872（実測） | D4 4.2（D1〜D3） |
| K2 | 3000 行 idle WS default / cpu | ≤ 129,989 / ≤ 158,479 KiB | 292,776 / 292,328 | 約 116 MB | D7 4 章（D3 + D7 + D4 の基礎分） |
| K2 | item box / text / button / row | ≤ 6.50 / 3.64 / 24.75 / 34.45 KiB | 12.39 / 10.48 / 66.59 / 85.44 | 約 5.4 / 3.0〜3.2 / 20.8 / 26.6 | D3 4.2、D7 4 章（text は試作で 2.93〜3.00 を実測） |
| K3 | 3000 行・毎 frame 更新の CPU | ≤ 19.87 % | 99.08 % | 13.4 %（実測） | D1 4.1 |
| K4 | 3000 行 work p95 / p99 | ≤ 3.67 / 4.94 ms | 19.28 / 21.51 | 2.61 / 2.92（実測） | D1 4.1 |
| K4 | 描画間隔 p99（3000 行） | ≤ 19.17 ms | 22.89 | 18.10（実測） | D1 4.1 |
| K5 | unrelated text 更新 n3000 | ≤ 9.46 ms | 57.53 | 約 2.5（非 a11y 1.7 + a11y ≤ 0.5 + hit ≤ 0.3） | D1 4.2、D2 4.2 |
| K5 | row 更新 n3000 | ≤ 16.96 ms | 78.98 | 約 8（非 a11y 7.4 + a11y ≤ 0.5） | D1 4.2、D2 4.2 |
| K5 | pointer p95 n3000 | ≤ 10.60 ms | 89.05 | 約 6.6 | D2 4.2（D1 の sync_shader・sync_playback の除去が前提） |
| K5 | wheel n3000 | ≤ 16.29 ms | 22.02 | 約 0.9 | D2 4.2 |
| K5 | mount n3000 | ≤ 813.8 ms | 1,507.4 | 約 780〜800 | D3 4.3（試作で 830〜837 を実測、残りは見積り） |
| K5 | counter click | ≤ 1.907 ms | 1.954 | 約 0.93 | D1 4.1 |
| K6 | counter 画面の GPU 抜き（MS-5 (1)） | ≤ 14,467,313 B | —（同じ構成の HEAD は 14,406,656 を実測） | 約 14,120,000 | D4 4.1（D7・D6） |
| K6 | counter default（`gpu` 込み、MS-5 (2)） | ≤ B1-start（元の機械 18,978,816） | 18,978,816 | 約 18,170,000 | D7 の実測（GPU 込みで −0.76 MB）と D6 |
| K6 | `kv_startup` no-default | ≤ 11,177,658 B | 15,922,688 | 11,062,784（実測） | D4 4.1（MS-1・D7） |
| K6 | basic catalog の GPU 抜き（MS-5 (1)） | ≤ 31,912,659 B | — | 約 28.2〜28.8 MB | D4 4.1（D7・D6・D8 の font の subset） |
| K6 | basic catalog default（`gpu` 込み、MS-5 (2)） | ≤ B1-start（元の機械 43,090,432） | 43,090,432 | 約 32.8〜33.4 MB | D4 4.1 に GPU 一式 4.57 MB を足したもの |
| K1・K3 idle・K4 0 行・K4 初回 raster | 達成済みの行 | — | — | 悪化させない | 各付属書の 4 章 |

余裕が小さい行：mount（2〜4 %）、K6 の counter 画面の GPU 抜き（約 0.35 MB）と `kv_startup`（約 0.17 MB）。K6 の catalog の GPU 抜きは D8（font の subset、`fonttools` が必要）が無いと約 37 MB で未達。CP1 で測り、未達なら作業指示書 7.3 節（CP2 では 11.3 節）の手順に従う。

### 2.4 版 1.1 の誤りの訂正

| 版 1.1 | 訂正 | 出典 |
|---|---|---|
| 1.2 節 K6 の「fat LTO・cgu 1・s・strip で 14.5 MB」「GPU 一式約 0.9 MB」 | 再現しない。同じ指定で B0 のコードは 18.14 MB、GPU 一式は 4.57 MB。MSVC の exe に strip はほぼ効かない。K6 の counter は `gpu` を既定 feature に含めたままでは届かない。`gpu` は既定に残し（DEC-01）、counter の行を MS-5 で定義し直した | D4 1.1、E-3 |
| 1.5 節「scroll 時の a11y payload の作り直し（絶対座標で公開しているため）」 | a11y の bounds は既に content 空間で、祖先の scroll では子孫の payload は変わらない。範囲外とした理由は誤り。V2 は scroll を O(changed) で公開する | D2 1.5、3.7 |
| 4.7 MEM-6「3000 行の live_reactive は B0 の 0.6 倍以下」 | B1 は 0.82 倍で未達だったが RESULT・OPEN に記録が無かった。V2 の見積りは約 0.30 倍 | D3 7 章 5 |
| 4.6 A11Y-1（Geometry 経路） | `Model::repaint` が `a11y.node` を呼ぶため、hover・press・focus で必ず Metadata 経路（O(N)）に入り、K5 では一度も効いていなかった。Geometry 経路自体も O(N) が残っていた | D2 2.1、2.2 |
| 4.4 LAY-1 | `never_rounded` に Contents の slot が永久に残り、毎 pass 全 tree を丸め直していた | D1 7 章 D1 |

### 2.5 版 1.1 の節のうち V2 が置き換えるもの

| 版 1.1 | V2 |
|---|---|
| 4.3 GPU-1 の「需要の伝達」（`HostRequest::DemandGpu` を 4 箇所から出す）、4.3.1 `Composition::needs_gpu` | D6 3.2〜3.5：起動の入口は `Gpu::begin`（AtLaunch のとき `Off` から）と `Gpu::demand`（`Deferred` のときだけ）。shader の需要は `FrameReport.shader_fallback`。core の需要は `gpu_interop::adapter()` の読取りだけ。`needs_gpu` は削除 |
| 4.3 の優先順位の表の「試験の host で GPU を要求するもの → AtLaunch」 | D6 3.1 の表（試験は `RenderOptions::gpu_on_demand()` で OnDemand を選べる。試験 host が `start` を黙って上書きする処理は削除） |
| 4.4 LAY-1 の丸めのフォールバック | D1 3.2 S-2 |
| 4.5 PNT-1 の Contents の印付け（`stale_fact` の flat 子への伝播） | D3 3.4 M2-4（構造変更は `stale_children_of`） |
| 4.6 A11Y-1 の `Dirt::Geometry` | D2 3.1 の `Dirt` の再設計（payload 経路） |
| 4.7 MEM-4 M-R5（STOP） | D3 3.7 M2-7（確定） |
| 4.7 MEM-5 `ComputedTextStyle` の共有 | D7 M3-2（葉にも効く共有、`Recent`） |
| 4.8 の profile | D4 D7（第三者の cold crate だけ `opt-level = "z"` を追加） |
| 7 章の GPU OnDemand の E2E の書き方 | D6 3.6（`gpu_on_demand.rs` の完全なコード） |

---

## 3. crate 構成

V2 は crate を追加・統合しない。調べた境界の候補と判断：

| 候補 | 判断 | 理由 |
|---|---|---|
| hit test を engine から分ける | 分けない | hit は engine の `Model` の tree・transform・scroll・paint の ink を読む。D2 は engine の中で module を分け（`model/interact/hit.rs`）、paint への依存は model が定義する trait `HitInk` で逆転させる（DEC-04） |
| a11y の構築を crate に | 分けない | 版 1.1 3.3 と同じ（`Model` の内部を共有する） |
| `Recent<T>`（直近値の共有）を独立の crate に | 分けない | 小さな値の型で、`kivact-types::cell` に置く（D7 M3-11）。engine と layout が使う |
| `FontSource` | `kivact-types` に置く | 描画・text・asset の 3 crate が共有する値の型（D4 D3） |

---

## 4. 決定（付属書の「人間の判断が要る点」の結論）

付属書の 8 章の項目は、すべてここで決めた。実装者は付属書の 8 章を読んでも選ばない。

| ID | 出典 | 問い | 決定 |
|---|---|---|---|
| DEC-01 | D4 Q1 | `gpu` を facade の既定 feature から外すか | **外さない**（ユーザーの決定、2026-11-10）。既定は版 1.1 のまま `default = ["gpu", "complex-scripts"]`（GPU は OnDemand で起動）。D4 の「D4」（既定 feature の変更、consumer の default、README の表）は行わない。K6 の counter の行は MS-5 で定義し直す。理由：既定から外すと、CPU の raster が重い画面で GPU への自動の切替えが無くなり、user shader が CPU の代わりの見た目になる。製品の既定の性能と機能を、誤った見積りから決めた目標のために変えない |
| DEC-02 | D4 Q2 | K6 の no-default を GPU なしで測るか | 測る（MS-1） |
| DEC-03 | D4 Q3 | catalog の NotoSansJP を subset にするか | する（D4 D8）。subset の生成に Python の `fonttools` が要る。Orchestrator が W0 で `python -m pip install --user fonttools` を行う。入れられない環境では S9 だけを STOP にして他を進める |
| DEC-04 | D2 8-1 | `HitInk` による model → paint の依存性逆転 | 採る。model が「描画側が hit の上界を答える」interface を定義し、paint が実装する |
| DEC-05 | D2 8-2 | 公開の診断 `Stats::a11y_enumerated` を足すか | 足す（AH2）。E2E で PERF-1 の後退を検出する唯一の観測点 |
| DEC-06 | D2 8-3 | Metadata 経路（構造・role・name 等の変化）の O(N) を残すか | V2 では残す。PERF-1 の例外として、`build_identity_metadata` の doc comment に理由（構造の変化の正しさを全比較で守る。K5 の 4 場面に現れない）を書く（AH1 の手順に含める）。増分化は後続（7 章） |
| DEC-07 | D2 8-4 | turn の固定費（約 2.7 ms）の担当 | V2 では扱わない。pointer の見積り 6.6 ms はこれを含んで必達内 |
| DEC-08 | D2 8-5、8-6 | 相対座標 + AccessKit transform、Mut::NONE の prop の細分 | どちらも採らない（後続の候補として 7 章に記録） |
| DEC-09 | D1 8-1〜8-4 | row 更新の余裕、compositor と試験 driver を D1 に含めること、hit の担当、残る O(N) | row 更新は a11y が ≤ 0.5 ms（D2）なので総額約 8 ms で必達内。arvane-compose・arvane-virtual は D1（F5・F6）が持つ。hit は D2（AH3）。D1 4.4 の残る O(N)（image 数・observed 数に比例するもの）は後続（7 章） |
| DEC-10 | D3 H1・H2・H3 | text item、3000 行の WS、button・row の余裕 | D7 で解決（text 約 3.0〜3.2、button 20.8、row 26.6、3000 行約 116 MB）。D3 H3 の追加策は行わない |
| DEC-11 | D3 H4 | `Style::paint` の callback を持つ box で style の compile も再実行されること | 許容する |
| DEC-12 | D3 H5 | `kv_heap` の `CHAIN`・`TOP` の出力 | 採る（診断の道具の変更、CI ではない） |
| DEC-13 | D4 Q4 | idle の window が CPU の frame state を手放すまでの時間 | 1 s（`FRAME_STATE_KEEP_NS`） |
| DEC-14 | D5 8-1 | 公開の recipe state の形の変更（PartState などを accessor に） | 採る（D5 で確定済み）。外部の theme 作者は `state.input.pressed` → `state.input.is_pressed()` のように直す。V2 の変更点の一覧（作業指示書 W8 の PR 下書き）に書く |
| DEC-15 | D5 8-2 | `CheckableState` の排他を型にするか | 型にする（D8 3.1・3.2 `CheckControl`）。D5 の F-T4（`SwitchFacts`）は行わない（5.2） |
| DEC-16 | D5 8-3 | 抜き取りしていない crate の marker を人が読むか | D8 で全 917 行を判定した。V2 の後、`state-audit.md` を人が確認することは版 1.1 6.1 の 5 のとおり AC-3 の条件に残す（W8） |
| DEC-17 | D6 8-1 | `GpuPreference`・`GpuPower`・`GpuBackend`・`GpuStart`・`AdapterMatch` の所在 | `kivact::launch` に移し、feature なしで公開する（G8）。`App::gpu_preference` と `RenderOptions::gpu_preference` の cfg を外す。feature `gpu` が無い build では無視されることを doc に書く。`gpu_interop` からの再公開は削除する |
| DEC-18 | D6 8-2 | ARG-6 の `show`/`hide` と既存の `MenuApi::show(factory)`・`CommandPaletteApi::show(factory)` の衝突 | content を mount する method を `content(factory)` に改名する（G9） |
| DEC-19 | D6 8-3 | API-3（handler を位置引数にしない）と `#[component]` の必須 callback の位置引数 | `#[component]` は「必須の props は位置引数」を優先する。architecture.md の API-3 を「kit の builder の任意 handler は setter で受ける」に限定して書き直す（W8 の書き戻し） |
| DEC-20 | D6 C-15 | prelude の `provide_theme` | prelude から外す（`provide_locale`・`provide_messages` と揃える。App の root で一度書くだけなので明示の import が分かりやすい）。G6 が行い、呼出し元の import を直す |
| DEC-21 | D7 H-P7-1〜5 | WS の回ごとの揺れ、f32 の測定記録と `remeasures_same`、最初の frame の追加の shaping 1 回、`Semantics` を疎にせず共有、`Model::sync` の 63 B/item | すべて D7 の推奨のとおり。判定は MS-4 |
| DEC-22 | D8 8-1 | `TimestampEntry.copyable`（読み手が無く copy 操作も未実装） | V2 では field を残し、marker を事実どおりの `option:` にする（P8-SOFT）。copy 操作の実装は V2 の範囲外の kit の作業（7 章） |
| DEC-23 | D8 8-2 | P8-GPU の時期 | G1・G3・G8 の後（作業指示書 W6） |

---

## 5. 付属書間の調整

### 5.1 D7 が D3 を置き換える部分

- D3 M2-5b、M2-13、M2-8 の手順 6 は行わない。代わりに D7 M3-2、M3-4・M3-9・M3-10、M3-5 を行う。
- そのため D3 の unit U-M4 と U-M5 は**取り消す**（V2 の unit に M4・M5 は無い）。
- D7 M3-11 は D3 M2-5 の `Recent<T>` を `kivact-types::cell` に移す。M3（U-M3）が `model/recent.rs` を作った後に T2 が移す。

### 5.2 D8 が D5 を置き換える部分

- D5 付録 P5-F の F-T4（`SwitchFacts`）は行わない。UI-3 は F-T4 を飛ばし、P8-CK が `CheckControl` を入れる。
- D8 3.3〜3.5 が D5 の marker の文言と異なる場合は D8 が正（D8 の付録の各表は「P8 統合で上書き」の列を含む）。D8 3.6 の 7 件の上書き（`CompiledStyle` を projection にしない等）を含む。
- FINAL-1（PartState）と FINAL-2（xtask X3〜X5）は D8 の全 unit の後。

### 5.3 同じファイルを変える付属書の順序

作業指示書の wave がこの順序を守る。設計の上の依存は次のとおり。

| ファイル（engine は `crates/kivact-engine/src/`） | 順序 | 注意 |
|---|---|---|
| `model/aggregate.rs`、`model/apply.rs`、`model/interact/sync.rs`、`model/layout_motion/steps.rs` | F1 → AH1 → M3 →（T7、sync.rs）→（P8-EN の marker） | F1 の索引の書き手（`index_sources`・`forget_sources`）を AH1・M3 が消さない |
| `model/interact/geometry.rs` | F4 → AH3 → T7 | AH3 が hit の塊を `model/interact/hit.rs` に移す。F4 の paint の ink の不変条件（drawing・geometry・scroll・可視性の変化で自分と祖先の ink を捨てる）を AH3 の枝刈りが前提にする |
| `tree/node_tree.rs` | F3 → T7 →（P8-EN） | M3 の M2-4 は `NodeTree::box_of` を使い、F3 の flat cache の形に依存しない |
| `host/write/typed.rs`、`contract/node.rs`、`model/recent.rs` | AH1 → M3 → T2 → T9 | `Recent<T>` は T2 で `kivact_types::cell::Recent` になる。T9 はその後 |
| `host/events.rs`、`host/window.rs`、`host/pump.rs` | AH1 → S3 → EN-2 →（P8-EN） | S3 の `LastPresent::Shown { at_ns }` と EN-2 の F-2 は別の field で衝突しない |
| `crates/arvane/src/window.rs`、`presenter/frame_policy.rs`、`arvane-compose/src/cpu/compositor.rs` | S2 → G1 →（P8-GPU、P8-ARV） | G1 は `Windows.gpu`・`Driver.gpu` を消す。S2 の `forget_frame_state` は残す |
| `crates/arvane-text/src/text_engine.rs`、`crates/arvane/src/engine.rs` | S4 → T4 | S4 の `FontSource` の上に T4 の paragraph の cache を書く |
| `crates/kivact-layout/src/{tree,columns}.rs` | F2 → T5 →（LY、P8-L1） | |
| `crates/kivact-core/src/view/mount/semantics.rs`、`view/tree.rs`、`view/node_ref.rs`、`client/dispatch/*` | M7、M8 → T6 →（CO、P8-CO） | |
| `crates/kivact-core/src/view/flow/list.rs` | M1 → S7 | |
| `crates/kivact-runtime/src/reactive/{data,graph}.rs` | M11 → T8、S7 →（RT、P8-RT） | |
| `crates/kivact-ui/src/button.rs` | M10 → UI-3 | `crates/kivact-interaction/src/toggle.rs` は M10 だけが変える（UI-3 が持つのは `crates/kivact-ui/src/toggle.rs`） |
| `crates/kivact-interaction/src/menu/handle.rs`、`crates/kivact-ui/src/calendar/view.rs`、`dev/xtask/src/api_shape.rs` | G7 →（IN-2、P8-UI6、P8-XT） | |
| `README.md`、`README_JP.md`、`docs/for-kivact-user/{en,jp}/README.md` | G3 → G6 | |
| `templates/basic/Cargo.toml` | S8 だけ | |

### 5.4 V2 の性能の変更の後に書く状態の unit

D5・D8 は HEAD のコードに対して書かれている。状態の unit（W6）は性能の unit（W1〜W4）の後に行うので、行番号がずれ、一部の field は性能の unit が既に変えている。規則（作業指示書 0.4 にも書く）：

- 対象の field・関数が同じ名前で残っていれば、付属書の手順をそのまま行う。
- 性能の unit が対象の field を消した・改名した・型を変えた場合は、その項目を STOP にする（推測で marker を付けない、推測で enum にしない）。ただし D8 P8-GPU の「P6-1 が field を消した場合はその行は不要」のように付属書が扱いを書いている場合はそれに従う。

### 5.5 DEC-01 による D4 の変更

- D4 3 章の「D4」（`gpu` を既定 feature から外す）と、5.4 節の D4 の部分、4.1 の表の「D4」の列は行わない。S6 は「D5」（`dev/bench/Cargo.toml`）だけを行う。
- D4 4.1 の K6 の見積りは MS-5 の行で読み替える：counter 画面の GPU 抜き 約 14.12 MB、counter の `gpu` 込みの既定 約 18.17 MB（B1 18.98 − D7 の実測 0.76 − D6 約 0.05）、catalog の GPU 抜き 約 28.2〜28.8 MB（D4 4.1 の値そのまま）、catalog の `gpu` 込みの既定 約 32.8〜33.4 MB。
- D4 D8（catalog の font の subset）は変えない（catalog の GPU 抜きの行に必要）。

---

## 6. 既存欠陥（Slim-Fast の実装 79bbd409..db8d679b）と直す unit

付属書の 7 章の確定した欠陥。推測のものは付属書に「推測」と書いてあり、直す unit の E2E で確かめる。

| 欠陥 | 位置 | 直す unit | 出典 |
|---|---|---|---|
| `never_rounded` が Contents を保持し毎 pass 全 tree を丸め直す | `kivact-layout/src/tree.rs`、`columns.rs` | F2 | D1 D1 |
| `sync_shader_clocks` が毎 composition で全ノード × 祖先を走査 | `model/frame/clocks.rs` | F1 | D1 D2 |
| `playing_images` が毎 composition で全ノード × 祖先を走査 | `model/playback/tick.rs` | F1 | D1 D3 |
| layout pass ごとの全走査 3 種、重複 layout id の報告順 | `model/layout_motion/*` | F1 | D1 D4 |
| push 命令の bounds を diff し scroll viewport 全体が damage | `arvane-compose/src/plan/damage.rs` | F5 | D1 D5 |
| 仮想 window が present ごとに全画素を 2 回複製 | `arvane-virtual/src/window.rs` | F6 | D1 D6 |
| ChildIndex を layout pass ごとに全部引き直す | `paint/list/index.rs` | F4 | D1 D7 |
| `repaint` が `a11y.node` を呼び hover 等が Metadata 経路に入る | `model/aggregate.rs` | AH1 | D2 7-1 |
| 幾何・payload だけの変化を Metadata にする呼出元、状態 motion 等が毎 tick O(N) | 複数 | AH1 | D2 7-2 |
| AT 停止後に dirt と cache が際限なく溜まる | `host/events.rs`、`a11y/publish.rs` | AH1 | D2 7-3 |
| Metadata 受理時の O(N) の retain、fingerprint の二重計算 | `a11y/publish.rs` | AH1 | D2 7-5 |
| M-R4（EVT-1）が働かない | `model/events.rs` | M2 | D3 7-1 |
| mount の O(N²)（透明な親への insert） | `model/apply.rs` | M3 | D3 7-2 |
| kv_heap の site 名が空・誤帰属、row の定義の不一致 | `dev/bench/run.sh`、`kv_heap.rs` | M12 | D3 7-3、7-6 |
| button が常に computed の disabled を付ける（U23 の保持の直接の原因） | `kivact-ui/src/button.rs`、`kivact-interaction/src/toggle.rs` | M10 | D3 7-4 |
| 葉の `ComputedTextStyle` が共有されない、text の冗長な `role(Role::Text)` | `measure/inherited_text.rs`、`view/elements/text.rs` | T3、T1 | D7 7 章 |
| 登録 font の bytes が最大 4 回複製される | `lower/assets.rs` ほか | S4、S5 | D4 E-1 |
| `CpuRaster.pixmap` が window 大で常駐 | `arvane-render/src/cpu.rs` | S1 | D4 E-2 |
| CPU だけの window でも需要で GPU が起動する（重大） | `arvane/src/presenter/gpu.rs` | G1 | D6 D1 |
| 毎 CPU frame で composition 全体を走査（`needs_gpu`） | `presenter/frame_policy.rs` | G1 | D6 D2 |
| 既定の OnDemand で低水準の import と drawable の GPU export が始まらない | `gpu_interop/import/image.rs`、`drawable/requests.rs` | G3 | D6 D3、D4 |
| `Viewport::gpu_frame` が builder の時に `Owner::current` を呼ぶ | `view/elements/viewport.rs` | G3 | D6 D5 |
| 試験 host が `GpuPreference.start` を黙って上書きする（U56 の直接の原因） | `host/testing/test_host.rs` | G2 | D6 D7 |
| GPU まわりの doc が存在しない API を参照 | `kivact-test`、`contract/host.rs` | G2 | D6 D6 |
| `#[component]` の optional の `Value`、属性の破棄、`async fn` 等の診断、callback の default の評価時期 | `kivact-macros` | G5 | D6 C-1〜C-4 |
| 文書の誤り・リンク切れ・改名漏れ・catalog の文字列 | docs、examples | G6 | D6 C-6〜C-11 |
| ARG-1 違反 2 件と api-shape の穴 | `calendar/view.rs`、`menu/handle.rs`、`api_shape.rs` | G7 | D6 C-12 |
| ARG-2/3/6 の未適用 | kivact-ui、kivact-interaction、kit | G9 | D6 C-13 |
| `icon_only` を `leading_icon` と無関係に立てる | `kivact-ui/src/toggle_group.rs` | UI-3 | D5 7-1 |
| game の Run・Pause 後に status の notice が出ない（退行） | `examples/kivact-ui-game/src/game/window.rs` | EX | D5 7-2 |
| alert の再表示時の stale restore、transition が binding を読む | `kivact-ui/src/alert.rs` | UI-2 | D5 7-3 |
| `send_contact` が常に primary | `kivact-test/src/screen/input.rs` | KT | D5 7-4 |
| 偽 independent・偽 projection・実在しない関数を名指す projection | 複数 | D5 の各 unit、D8 の各 unit | D5 7-5〜7-9、D8 7-5、7-6 |
| lint の誤分類 2 件と gate/audit の条件の不一致 | `dev/xtask/src/state_fields/*` | XT-1、FINAL-2 | D5 7-10 |
| `SidebarWidth::Collapsed` の重複状態、filter の Valid の重複 | `side_navigation/shell.rs`、`data_table/filters.rs` | SO、UI-2 | D5 7-12、7-13 |
| radio item の `invalid` が常に false（Slim-Fast 以前から） | `kivact-ui/src/radio_group.rs` | P8-CK | D8 7-1 |
| doc の誤り（`Look.filtered`、`update_place`、`FlowFields.display`） | engine、style | P8-EN、P8-L1 | D8 7-2〜7-4 |

---

## 7. 範囲外（V2 の後続に記録するもの）

V2 の完了後、W8 で `dev/bench/results/OPEN.md` の「後続」節に書く。V2 では行わない。

| 項目 | 理由 | 出典 |
|---|---|---|
| a11y の Metadata 経路（構造の変化）の増分化 | K5 の場面に現れない。writer の通知の完全性に依存する | D2 8-3、DEC-06 |
| turn の固定費（約 2.7 ms）の内訳と削減 | 必達内 | DEC-07 |
| image 数・observed 数に比例する `report_images`・`poll_assets`・`report_geometry` | K に効かない | D1 4.4 |
| 相対座標 + AccessKit transform、Mut::NONE の prop の細分 | 必要性が無い | DEC-08 |
| SVG の描画を Arvane の painter へ（約 −1.2 MB） | 大工事で、既存 SVG E2E の期待値を変えるおそれ | D4 D7 補足 |
| `TimestampEntry.copyable` の copy 操作の実装 | kit の機能追加 | DEC-22 |
| scissor の矩形の判定を行ごとに前計算（CPU 合成の高速化） | 任意 | D6 C-17 |

---

## 8. 新しい規則と決定（W8 で architecture.md・decisions.md に書き戻す）

| ID | 内容 |
|---|---|
| PERF-1 の補足 | PERF-1 の例外（全走査を許す処理）は、doc comment に理由を書いたものだけ：構造の初期化、font の登録、環境の変化、明示的な `Full`、a11y の Metadata 経路（DEC-06） |
| PERF-4 | frame ごとの処理が読む「特別な prop を持つノードの集合」（shader の clock、再生中の画像、layout motion 等）は、prop の書き手が更新する索引（単一の書き手の投影）で持つ。毎 frame の全走査で探さない（D1 S-1） |
| PERF-5 | hit test は paint の ink（描画の上界）で子を枝刈りする。ink は model が定義する `HitInk` で読む（DEC-04） |
| MEM-8 | 空の view・tuple・Vec は host node を作らない（D3 M2-1）。多くのノードが同じ値を持つ prop・style・text style は `kivact_types::cell::Recent` で直近値を共有する（D3 M2-5、D7 M3-2・M3-11） |
| MEM-9 | idle の window は 1 s 後に CPU compositor の frame state を手放す。状態は `LastPresent` の variant で表す（D4 D2） |
| SLIM-11 | facade の既定 feature は `gpu` と `complex-scripts` のまま（DEC-01）。K6 は Kivact 自身の大きさを GPU 一式を除く構成で測り、GPU 込みの既定の build は悪化させない（MS-5） |
| SLIM-12 | GPU の起動の入口は二つ（`begin`、`demand`）。CPU だけの window は需要があっても起動しない（D6 3.2） |
| SLIM-13 | 試験は `RenderOptions::gpu_on_demand()` と `Screen::renderer()` で GPU OnDemand を検証する（D6 3.4） |
| SLIM-14 | 公開の recipe state は accessor で読む。排他な事実は enum（`CheckControl` 等）で持つ（DEC-14、DEC-15） |
| SLIM-15 | `#[component]` の必須 props は位置引数（handler を含む）。API-3 は kit の builder の任意 handler に限る（DEC-19） |
| SLIM-16 | lint の gate と audit の条件は一致させる。`projection:` は名指す関数の実在と唯一の書き手であることを xtask が検査する。`independent:` の理由文はファイルをまたいで共有しない（D5 X3〜X5） |

---

## 9. 危険と対策

| 危険 | 対策 |
|---|---|
| a11y の payload 経路の分類の誤り（identity が変わる書込みを payload にする） | debug の `self_check` が全ての E2E の毎 settle で公開済みの木と Full の参照構築を比べる（D2 5.1）。不一致は STOP（作業指示書 0.4） |
| hit の枝刈りの漏れ（ink が hit の範囲を含まない） | sticky・非 affine・drawable を枝刈りから外す。debug の自己検査が枝刈り無しと比べる（D2 3.6.4） |
| 索引（`MotionSources`）の更新漏れ | 全 setter は `Model::set_prop` → `changed()` の最後で `WRITTEN` を立てる。既存の画像・shader・shared layout の E2E と新しい `frame_sources.rs`（D1 5.2） |
| damage の不足で古い画素が残る | `incremental_frames_match_a_fresh_render`（D1 5.2）が増分の frame と新しい render を画素で比べる |
| 余裕の小さい KPI（mount、K6） | CP1 で測る。未達なら作業指示書 7.3 節（CP2 では 11.3 節） |
| Haiku 相当の実装者が規則に無い判断をする | 作業指示書 0.4 の STOP。設計の判断は人間（設計者）が本書・付属書に書き足して行う。Orchestrator は設計の判断をしない |
| 状態の unit が性能の unit の後のコードと合わない | 5.4 節の規則 |
| `opt-level = "z"` が性能を落とす | 第三者の cold crate に限る（D4 D7 の実測で Kivact の crate は対象外）。CP1 の K1・K4・K5 で確かめる |
