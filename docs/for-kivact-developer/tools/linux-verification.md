# Linux / WSLg の検証

実行ツールは [dev/tools/linux/test-wslg.sh](../../../dev/tools/linux/test-wslg.sh)。リポジトリrootから以下を実行します。自動の実 window 試験は `CONTRIBUTING.md` に従い共通の window lock を通します。実際の OS 入力を送る GUI harness・IME・支援技術の手動操作は対象と操作への明示許可が必要です。

```bash
dev/tools/window-lock.sh --timeout 5400 -- bash dev/tools/linux/test-wslg.sh [wayland|x11|all]
```

引数の既定値はall。WindowsのGit Bashからは `KIVACT_WSL_DISTRO`（既定Ubuntu）の同じ作業treeに入り、自分をWSL内で起動し直します。Linuxでは直接実行します。`CARGO_TARGET_DIR` の既定はdistribution ext4側の `$HOME/kivact-target`。Windows側targetと分け、出力/ロックの衝突を避けます。

clippy（`-D warnings`）、Arvane facadeの契約試験、workspace試験（`--no-fail-fast`）を実行します。WaylandはWSLgの環境、X11は `WAYLAND_DISPLAY` を外してXWaylandを選びます。音声の `kivact-music-player` は `libasound2-dev` が必要なため既存runnerの対象外です。`ARVANE_RENDERER` と `WGPU_*` は変更せず渡されます。

試験ケースは `--test-threads=1` で直列に実行します。埋め込みの実入力はX11のfocusを共有し、別の実窓を同時に開くとfocusや親のmap準備に干渉します。プロセス間のwindow-lockに加え、同じ試験プロセス内も直列にします。各ケースが明示的に作る複数Appやthreadの並行動作は引き続き検証します。FDの完全一致を測るArvaneの `embed_resources` は一般E2Eから分離したプロセスで実行します。

ログは `$CARGO_TARGET_DIR/test-wslg/<step>.log`。各失敗を残して最後に集計し、failureを成功に置き換えません。CPU/software adapterでの通過とhardware GPUの実行/skip/初期化失敗を分けて記録します。

WSLg Wayland/XWaylandだけの通過は、GNOME/KDE、fcitx/ibusのIME、Orca/AT-SPI、portal/clipboard、実hardware Vulkan GPUを保証しません。既知のXWayland/Weston切断とplatform除外は [現行設計](../design.md)を参照してください。GUI入力と性能測定をCargo等の重い検査と同時に実行しません。

## GPU texture 共有（実機）

WSLg は hardware の Vulkan GPU を持たないので、GPU texture 共有（実験 feature `gpu-interop`）は上の runner では確かめられません。ハードウェア GPU のある Linux 実機で [dev/tools/linux/verify-gpu-interop.sh](../../../dev/tools/linux/verify-gpu-interop.sh) を使います（[手動確認一覧](../manual-verification.md) の「GPU texture 共有」の節）。

## 埋め込み（実験）

`dev/tools/linux/test-wslg.sh x11` は feature embed の公開 API E2E も実行する。個別には `dev/tools/window-lock.sh -- cargo test -p kivact-public-api-tests --all-features --test e2e embed`。対象は WSLg の X11/XWayland で、Linux 実機 DAW・IME・screen reader は未実施。

## workspace 全体と埋め込み host の確認

一般化した [verify-workspace.sh](../../../dev/tools/linux/verify-workspace.sh) は WSL 内から既存の `test-wslg.sh x11`、architecture の読取り検査、`kivact-embed-host` の通常/二つの editor の smoke を実行します。Windows と WSL で PID が異なるため、**Windows の Git Bash 側で outer window lock を必ず取得**し、WSL 内で二重に lock を取得しません。Windows の Git Bash から、実際の作業 tree と distribution 名に置き換えて実行します。

```bash
dev/tools/window-lock.sh --timeout 5400 -- wsl.exe -d Ubuntu -- bash -lc 'cd /mnt/<drive>/<path-to>/kivact && bash dev/tools/linux/verify-workspace.sh target/linux-verification-20261031'
```

引数は `[log-directory] [--clean-workspace]`。既存の出力先では終了コード 2 で止めて証跡を保ちます。既定では clean しません。別 worktree の生成物が混ざる場合だけ `--clean-workspace` で workspace package を clean し、外部依存は残します。これは既存の build に影響するため、同じ target を使う検査を同時に動かしません。

`CARGO_TARGET_DIR` の既定は既存 runner と同じ `$HOME/kivact-target` です。Windows と共有せず、複数の worktree も別々の target を明示してください。`CARGO_PROFILE_DEV_DEBUG`・`CARGO_PROFILE_TEST_DEBUG`・`CARGO_INCREMENTAL`・`ARVANE_RENDERER`・`WGPU_*` は呼出元の値を尊重し、script 内で固定しません。runner の詳細ログは新しい出力先の `wslg/`（`KIVACT_WSL_LOG_DIR`）へ保存するため、既定の `test-wslg` ログも上書きしません。
