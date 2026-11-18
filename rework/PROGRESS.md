# Rework 作業の進捗（Orchestrator が更新）

## 環境（W0 手順 3）
- 統合 branch：`refactor/kashiwade/fundamental-rework`（基点 `2b8df5ac` = main の code。以後の差は docs のみ）
- target：空きが 200 GB 以上の drive は V:（2026-11-18 時点 1.7 TB 空き）。よって lane ごとに `V:/kv-target/rw-<L>`。統合・基点の測定は `V:/kv-target/rw-main`。
- lane の worktree：`W:/w/kv-wt/rw-W<W>-<L>`（W: の空き 136 GB）。
- 計算機：24 論理 core、RTX 3060 Ti（driver 32.0.16.1088）、WSL2 の Ubuntu-24.04 あり（Linux check 用）。

## 進捗
- W0 完了（基点の測定と gate）
  - 基点の試験の一覧：`rework/baseline-tests.txt`（1644 件。生成物で commit しない）
  - B0：`rework/kpi/B0.md`（INVALID・WARNING なし。commit `c9518946` の code は基点 `2b8df5ac` と同じ）
  - 基点の gate（T 4.1 の「W0 の基点の gate」）：fmt・xtask・clippy・gpu-interop-producer・kivact-docs・check-features は成功。
    - workspace（all-features）と workspace（no-default-features）：`chat_keyboard` の 1 件が失敗（`--no-fail-fast` で確認。他に失敗なし）。
    - GPU E2E（`kivact-public-api-tests --test e2e`、751 件）：4 件失敗（`rework/OPEN.md` の「基点の失敗」）。
    - 失敗はすべて基点で既に起きていたもので、改修の失敗ではない。直していない。
  - gate のログ：`rework/baseline-gate/`（生成物で commit しない）

## W1a（進行中・ブロック中）
- worktree の作り方の注意：submodule `vendor/xtask-core` の初期化は、worktree では `git -c protocol.file.allow=always submodule update --init vendor/xtask-core` が要る（既定の file transport の禁止）。
- lane T0（`wt/rw-W1a-T0`、worktree `W:/w/kv-wt/rw-W1a-T0`、target `V:/kv-target/rw-T0`）：T01 `5beb3a38`、T02 `e2879bee`、T03 `0a0cc403`、T04 `a75087ab`、T23 `50333d86`。すべて commit 済み。いずれも差戻しが付く（`rework/OPEN.md` の「差戻し」）。
- lane S（`wt/rw-W1a-S`、worktree `W:/w/kv-wt/rw-W1a-S`、target `V:/kv-target/rw-S`）：S01 `b0e84c65` は clippy の `disallowed_types` で失敗（差戻し）。統合 branch には merge しない。
- 統合 branch へは W1a を merge していない。全 lane が完了し差戻しが解けるまで、merge と tier b は行わない（作業指示書 1.3）。
- 次の判断（設計者）：S01 の構成式の覆い方、T01 の rg 条件、T02 の visible_to の `kivact-devtools-client`、T04 の 2 件（`dev/README.md:41` と jp の docs-pairs 2 件）。解けたら W1a の merge → tier b → W1b へ進む。
