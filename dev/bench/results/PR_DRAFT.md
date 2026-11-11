# PR の下書き：Slim-Fast V2（PR は作らない。下書きのみ）

## 概要

- V2 の目的：Slim-Fast（基点 `79bbd409`）の性能・メモリ・サイズの目標（KPI）を満たし、状態の監査（AC-3）を閉じる。
- 性能の原因（V2 設計書 2.4 と 6 章）：frame・turn・layout・paint の O(N) の全走査、a11y の O(N) の再構築、hit test の枝刈り不足、ノードごとのメモリ（mount・text・button・row の余剰）、GPU の需要が CPU だけの window でも起動していたこと、既定の OnDemand で import が始まらなかったこと、バイナリのサイズ（font と GPU 以外の依存）。
- 直したこと：
  - frame と layout の O(changed) 化（索引の導入、LayoutInset・ChildIndex の部分更新、damage の判定）。
  - a11y の payload 経路、hit test の ink による枝刈り（`beyond` の bit）、kv_heap の診断。
  - ノードの共有（`Recent<T>`）、CoreNode と CoreRare、text の style の共有、NodeRef の遅延 observation。
  - GPU の需要を adapter の読取りで行い、CPU の window では起動しない。試験は `RenderOptions::gpu_on_demand()` で検証。
  - バイナリのサイズ：`opt-level = "z"` を worker と OS 待ちの crate に限定、font の subset、dev/bench の gpu feature の分離。
  - 状態の監査：`CheckableState` などを型（enum）で閉じ、`projection:` の検査を可視性で範囲を限る。OPEN 27 → 0。
- 挙動の変更：scroll spy の購読の再報告、icon-only の toggle group の label（icon の無い item は label を残す）。

## KPI（`RESULT-V2.md` の全表。K6 は MS-5 の定義）

| KPI | 指標 | B0-here2 | B1-here | 必達 | 判定 |
|---|---|---|---|---|---|
| K2 | counter idle WS（default）KiB | 97,688 | 25,716 | ≤ 27,214 | 満たす |
| K2 | 3000 行 idle WS（default）KiB | 569,580 | 99,164 | ≤ 129,989 | 満たす |
| K2 | item box / text / button / row KiB | 28.64 / 10.58 / 108.84 / 146.06 | 3.60 / 2.90 / 18.58 / 24.21 | ≤ 6.50 / 3.64 / 24.75 / 34.45 | 満たす |
| K3 | 3000 行・毎 frame 更新の CPU（default）% | 100.30 | 12.12 | ≤ 19.87 | 満たす |
| K4 | 3000 行 work p95 / p99 ms | 22.2231 / 23.588 | 1.7832 / 2.0806 | ≤ 3.67 / 4.94 | 満たす |
| K5 | pointer p95 n3000 ms | 103.148 | 4.537 | ≤ 10.60 | 満たす |
| K5 | mount n3000 ms | 1,557.247 | 499.897 | ≤ 813.8 | 満たす |
| K6 | counter GPU 抜き（MS-5 (1)）B | 24,594,432 | 14,304,768 | ≤ 14,467,313 | 満たす |
| K6 | kv_startup no-default（MS-5 (1)）B | 24,590,848 | 11,250,176 | ≤ 11,250,176（改定後） | 満たす（必達は改定） |
| K6 | basic catalog GPU 抜き（MS-5 (1)）B | 54,251,520 | 29,166,080 | ≤ 31,912,659 | 満たす |
| K1 | main → first present（default）ms | 484.8 | 42.3 | 悪化させない | 満たす |

K6 は MS-5 の定義（GPU 抜きの build は `kivact-bench` の `--no-default-features` で測る。既定の build は `gpu` を含む）。K6 `kv_startup` no-default の必達は、設計者の判断で B1-here の実測値に改定した。

## 公開 API の変更

- 版 1.1 の改名（付録 B）と、ARG-1〜6 の適用（命令の名前と規約：`show`・`hide` は冪等、反転は `toggle`、閉じる理由を渡すものは `hide(reason)`）。
- `#[component]`：必須の props は位置引数（`handler` を含む）。`allow`・`warn`・`deny`・`deprecated` の属性を保つ。
- `Binding::update`。
- `GpuStart`、`complex-scripts` の feature。
- V2 の変更：
  - `GpuPreference` 一式は `kivact::launch` に移動し、feature なしで公開する。
  - `RenderOptions::gpu_on_demand`・`Screen::renderer`・`Screen::wait_for_gpu`。
  - `Stats::a11y_enumerated`。
  - recipe state の型と accessor の統一（下の文）。
  - G9 の改名と `while_open(factory)`（content を mount する method）。
  - prelude から `provide_theme` を削除（`provide_locale`・`provide_messages` と揃える）。
  - D8 の公開 API の変更 3 件：`LayoutInset.own_outer`、`ShellImeText.password` の削除、`GpuAdapter` の `AdapterIds`。
- recipe state の型と accessor の統一（`PartState` の `Availability` と `is_pressed()`／`is_hovered()`／`is_disabled()`／`is_hidden()`、`ButtonState`／`ToggleButtonState` の `label_hidden` と `is_icon_only()`、`MenuItemState.display: MenuItemDisplay` と `check()`／`is_expanded()`、`CardState.action: CardAction` と `input()`／`is_interactive()`／`selection()`、`AlertState.content` と `has_content()`／`expansion()`、`BreadcrumbItemState` と `DataTableCheckState` の enum 化、`CodeBlockState.collapse: CodeCollapse`、`CheckableState.control: CheckControl` と `is_checked()`／`is_mixed()`／`checked()`／`switch_text()`／`switch_drag()`）。

## 確認の状況

- CP2（統合 HEAD、`1f000b93` の上）：fmt、フルの xtask（architecture を生成、Node を `XTASK_NODE` で指定）、workspace の clippy（-D warnings）、feature の組み合わせ、`--no-default-features` の試験、workspace の全試験（失敗 0）、gpu-interop-producer、kivact-docs、GPU の `gpu_on_demand` は成功。
- state 監査：OPEN 0（`dev/xtask/state-audit.md`）。
- Linux：未確認（WSL が起動できなかった）。
- AC-3：人間の確認は 2026-11-11 にユーザーが承認（`AC3_REVIEW.md`）。
- 残るもの：`kivact-types` の `cargo doc` の警告 21 件（`geometry3d.rs`、設計者の判断で無視）。

## 付記：PR は作らない

この下書きは commit するだけ。PR の作成と main への merge は人間が行う（作業指示書 0.1）。
