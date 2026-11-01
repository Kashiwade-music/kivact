# dev

開発用の検査とテストです。手順は [CONTRIBUTING](../CONTRIBUTING.md) にあります。

| 場所 | 用途 |
|---|---|
| `tests/kivact-public-api/` | 公開 API の実ウィンドウ E2E とウィンドウなしの画像書き出しの試験 |
| `tests/docs/` | `docs/` と README の Rust コードブロックの doctest |
| `tests/gpu-interop-producer/` | GPU texture 共有の試験で使う描画エンジン |
| `tools/window-lock.sh` | ウィンドウを開くコマンドを worktree 間で一つずつ実行する lock |
| `tools/check-features.sh` | library の非既定 feature を一つずつコンパイルする |
| `xtask/` | `cargo xtask check`（行数の上限、`visible_to`、module の依存、`architecture/` の図） |
