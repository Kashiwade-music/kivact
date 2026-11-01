# Kivact 開発用のアプリと検証

利用者向けの学習サンプルは [examples](../examples/README.md)、開発者向けの設計・要求・既知差は [開発者資料](../docs/for-kivact-developer/README.md) を参照してください。

| 場所 | 用途 |
|---|---|
| `apps/` | 複数の機能を組み合わせる回帰アプリ、外部部品の適合確認、制作 UI の回帰対象 |
| `benchmarks/hostbench/` | 実窓での描画・入力・レイアウト性能の測定 |
| `tests/kivact-public-api/` | `kivact::test` を使う公開 API の消費者試験と trybuild |
| `tests/docs/` | README と入門・手順の Rust コードを検証する doctest |
| `tests/workspace-policy/` | 依存の許可リストと公開名の一覧の試験 |
| `tests/fixtures/external-component/` | facade だけで実装した外部部品の fixture |
| `tests/fixtures/official-ui-consumer/` | 独立 workspace から3つの公式 kit を消費する試験 |
| `tests/fixtures/core-only-consumer/` | 独立 workspace から core facade だけを消費するコンパイル検証 |
| `xtask/` | xtask-core による規則の検査とアーキテクチャ図の生成（`cargo xtask check`） |
| `dev/tools/gui-harness/` | Windows SendInput / UI Automation の内部ハーネス |
| `dev/tools/linux/` | WSLg の Wayland / X11 検証 |

実窓の試験・ベンチマーク・OS 入力は利用中のデスクトップを妨げます。対象と時間を確認し、PC 利用者の許可を得てから実行します。検証コマンドは [CONTRIBUTING](../CONTRIBUTING.md) にあります。
