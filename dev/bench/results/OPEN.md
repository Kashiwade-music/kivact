# OPEN: 設計者への差戻し

- U23 `crates/kivact-core/src/view/mount/semantics.rs:61` — STOP: button ごとに意味論 patch の Vec が mount 後も effect に保持される（割当て元 `kivact_core::view::element_data::Common::append_semantics`、1280.0 B/item、M-R5）/ 要素を消してよいかの設計判断はできないため変更せず報告。影響する設計書の節: 4.7 節 MEM-4（M-R5）。
- CP1 K5 `wheel_n3000` — B0 mean 24.436 ms / p95 28.291 ms、CP1 mean 27.555 ms / p95 29.168 ms（mean +12.8%、p95 +3.1%）。K1・K4 とほかの K5 行は改善。関連 unit: U21（paint facts）・U22（a11y geometry）。CP1 の規則に従い記録して先へ進む（W7 の B1 で再判定）。
- CP1 環境メモ — Windows のリンカ PDB 上限とディスク不足のため、checkpoint のテストは `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` で実行した（debug assertions は有効のまま。試験の内容・期待値は変えていない）。
- U56 `dev/tests/kivact-public-api/tests/e2e/gpu_on_demand.rs` — STOP: `RenderOptions` で `GpuStart::OnDemand` を要求できず（gpu を要求しない設定は `GpuMode::Off` / `prefer_gpu = false` で GPU を起動せず）、`Screen` にも公開の renderer 取得が無いため、設計書 7 章の GPU OnDemand の E2E を公開 API だけで書けない。影響する設計書の節: 4.3、7 章。`kivact-test` に公開の renderer 観測（または OnDemand を選べる test host の option）が必要。

## CP2 KPI 未達（設計者への差戻し、2026-11-10）

B0 と CP2（`dev/bench/results/CP2.md`、どちらも有効）の比較。倍率は B0 ÷ CP2。設計書 1.4 節の「必達」に対して未達の行：

| KPI | 指標 | B0 | CP2 | 倍率 | 必達 |
|---|---|---|---|---|---|
| K2 | counter idle WS（default） | 136068 KiB | 35068 KiB | 3.88× | ≥ 5× |
| K2 | counter idle WS（CPU） | 37372 KiB | 35076 KiB | 1.07× | ≥ 1.4× |
| K2 | 3000 行 idle WS（default） | 649944 KiB | 275332 KiB | 2.36× | ≥ 5× |
| K2 | item box / text / button / row | 32.52 / 14.57 / 123.75 / 172.24 KiB | 11.33 / 9.56 / 62.55 / 79.80 KiB | 2.87× / 1.52× / 1.98× / 2.16× | ≥ 5× / ≥ 4× / ≥ 5× / ≥ 5× |
| K2 | 3000 行 idle WS（CPU） | 554676 KiB | 275880 KiB | 2.01× | ≥ 3.5× |
| K3 | 3000 行・毎 frame 更新の CPU | 99.37% | 99.95% | 1.00× | ≥ 5× |
| K4 | 3000 行 work p95 / p99 | 21.99 / 24.71 ms | 18.82 / 21.39 ms | 1.17× / 1.16× | ≥ 6× / ≥ 5× |
| K4 | 描画間隔 p99（3000 行） | 27.13 ms | 22.14 ms | — | ≤ 19.17 ms（1.15 × 16.67） |
| K5 | mount n=3000 | 1627.5 ms | 1459.3 ms | 1.12× | ≥ 2× |
| K5 | 無関係な 1 text 更新 n=3000 | 75.68 ms | 58.71 ms | 1.29× | ≥ 8× |
| K5 | 1 行の更新 n=3000 | 101.74 ms | 79.28 ms | 1.28× | ≥ 6× |
| K5 | pointer 移動 p95 n=3000 | 105.99 ms | 87.92 ms | 1.21× | ≥ 10× |
| K5 | wheel n=3000 | 24.44 ms | 22.93 ms | 1.07× | ≥ 1.5× |
| K6 | counter（default） | 24594432 B | 18978816 B | 1.30× | ≥ 1.7× |
| K6 | kv_startup（no-default-features） | 24590848 B | 15922176 B | 1.54× | ≥ 2.2× |
| K6 | basic catalog（default） | 54251520 B | 43090944 B | 1.26× | ≥ 1.7× |

達成した主な行：K1 startup 461.8 → 40.9 ms（11.3×、必達 5×）、K1 catalog spawn 654.0 → 196.8 ms（3.3×、必達 2.5×）、K4 first-frame raster 30.7 → 3.2 ms（9.6×、必達 6×）、K4 0 行 work p95 3.54 → 1.86 ms（悪化なし）、K5 counter click 2.29 → 1.78 ms（1.28×、必達 1.2×）、K2 counter（GPU 強制）136444 → 94704 KiB（悪化なし）、K3 アイドル 0 frame。

原因の調査と設計の見直しを設計者に差し戻す。実装者は目標を下げず、測定条件も変えない。AC-2 はこのままでは満たされないため、W7 で完了を宣言しない。

## W7-1 の判定（2026-11-10）

- AC-1 ✓（CP3 の全 gate が成功。期待値は緩めていない）
- AC-2 ✗（`RESULT.md` のとおり。K2・K3 の 3000 行・K4 の 3000 行・K5 の多く・K6 が必達未達）
- AC-3 ✗（`dev/xtask/state-audit.md` に 27 件の `OPEN`。U23 の M-R5 も残る）
- AC-4 ✗（U23 の STOP が K2 に影響。GPU OnDemand の E2E が未実施。撤退規則の適用は無し）
- 判定：受入れは満たされない。W7-2（書き戻し・後片付け）と W7-3（PR の下書き）は行わない。設計書・作業指示書・レビューは残す。

## V2 環境（2026-11-10）

- 機械：Windows 11、24 logical CPUs、137 GB RAM、GeForce RTX 3060 Ti（Vulkan）。
- ディスク：当初は C: 約 64.8 GB、W: 約 26.4 GB 空きだったため **3 つの共有 target**（`C:/Users/ryo/AppData/Local/Temp/opencode/kv-target/AB`・`CD`・`EF`）を使った。W1 の途中（2026-11-10 15:35）に C: 約 27 GB、W: 約 15 GB、E: 約 302 GB まで減り、C: の共有 target が増え続けるため、**W1 の残りの unit から target を E: の lane ごとに変える**：`E:/kv-target/<A〜F>`（1.1 節「70 GB 以上のドライブがあれば lane ごと」。E: は十分に空いている）。W6 の L1〜L6 は A〜F に読み替える。KPI の計測は `E:/kv-target/kpi`。旧の C: の共有 target は W1 の merge と確認の後に消す。
- 追記（ユーザーの指示、2026-11-10）：W: の空きを増やしたので、以後の target は **`W:/kv-target/<A〜F>`** にする。E: は ST4000DM004（HDD）で書き込みが遅いため、新しい unit では使わない。W: は ADATA SU650（USB 接続の SSD）。E: に作った target（W1 の途中で作った分）は、その lane の unit が終わったら消す。KPI の計測は `W:/kv-target/kpi`。
- すべての cargo に `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` を付ける（ディスクと PDB の上限。debug assertions は有効）。
- `B0-here`：V1 で同じ機械・同じ commit `79bbd409` に取った `dev/bench/results/B0.md` を `B0-here.md` として使う（機械・commit・条件が同じため再計測しない）。
- `B1-start`：V1 の `B1.md`（commit `0699af99`、コードは現在の HEAD と同一）を `B1-start.md` として使う。
- `fonttools`：`python -m pip install --user fonttools` 成功（4.68.0）。S9 を実施できる。
- V2 差戻し：この節の下に追記する。

## V2 差戻し（W1、2026-11-10）

- F5 `crates/arvane-compose/src/cache.rs:53`（commit `a8f6ad32`、付属書 D1 3.6）— `xtask check` の `state-fields`：`Indexed` が `bounds`・`group` の 2 つの `Option<PixelRect>` を持つため error。付属書は marker を指定していないので、marker を足さず型も変えていない。設計者の判断（marker の文言、または型の形）が要る。
- M7 `crates/kivact-core/src/view/mount/semantics.rs:14`（commit `a087054c`、付属書 D3 3.7）— `xtask check` の `state-fields`：`Plan` が `checked`・`indeterminate` を持つため error。同じく marker を足していない。設計者の判断が要る。
- S4（付属書 D4 3 章「D3」）— **W2 に回した**。元の commit `3c284b0b` は残し、W1 の lane F では `3224b48b`（revert）で打ち消した。理由：`kivact-engine/src/lower/assets.rs:256` の `register_font(bytes)` が `FontSource` を要求し、呼出しの直し（S5、P4-U5）が W2 の lane B にあるため、W1 の後に compile が通らない（本書 3 章「W1 の後に compile が失敗してよい crate：なし」）。本書 0.5 の「未 merge の変更を前提にする compile error は次の wave へ」に従った。W2 では S4 の commit を cherry-pick してから S5 を行い、同じ lane に並べる。S4 の xtask の後半（kivact-types 以降）の結果は未確認（報告で fmt 後の再実行が完了していない）。
- M2（commit `42c8db7a`）は完了。付属書の `V2 設計書 4.9（EVT-1）` と `M-R4` の文は旧い縮小動作を書いたままなので、W8 の書き戻し項目にする。
- M7 の作業中に、付属書が名前を挙げていない e2e 試験 1 件（`constant_semantics_publish_once_and_keep_no_publisher`）を取り除いた（本書 0.2 の 3）。退避先は `C:/Users/ryo/AppData/Local/Temp/claude/eab73ffa7cd53a85/c0105c61-711d-4a11-89c2-54c01c5acade/scratchpad/M7-leftover.patch`。
- M6（付属書 D3 3.6、lane D）— **commit なし、差戻し**。worktree は `a087054c` のまま clean。試みの全差分は `C:/Users/ryo/AppData/Local/Temp/claude/eab73ffa7cd53a85/c0105c61-711d-4a11-89c2-54c01c5acade/scratchpad/m6-attempt.patch`（fmt していない）。理由は 3 つ：(1) E2E `style_paint::paint_only_style_tracks_size_signals_theme_and_removal`（`dev/tests/kivact-public-api/tests/e2e/style_paint.rs:74`）が失敗する。付属書の InStyle の経路は、style の effect の中で paint の callback を記録するため、callback だけが読む `color.set(true)` で `.style(..)` の closure が再実行される。DEC-11 はこれを認めているが、試験は逆を期待している。設計者が試験の期待値を変えるか、設計を変えるかを決める必要がある。(2) `xtask check` の `state-fields`：`InStylePaint`（`crates/kivact-core/src/view/mount/style.rs:94`、`size`・`drawn`）に付属書が marker を指定していない。(3) 付属書と実装が合わない：`text_measurer` は `Option` ではなく `TextMeasurer` を返すため、付属書の `if let Some(m) = measurer` は compile しない（試みでは `d.set_text_measurer(measurer)` にした）。ほかに、`bind_style` が 86 行（上限 80）、`InStylePaint::record` が引数 8 個（上限 7）で、試みでは直している。W2 以降の M8・M9 が M6 の変更を前提にする場合は、その unit が STOP する。
- F4（commit `81db248b`）の副作用：`crates/kivact-engine/src/paint/list/invalidation.rs` の `invalidate` の doc が「layout pass の後に全 location（と変わった size）」と書いたまま古い。doc だけなので、W8 の書き戻しで直す（F4 の所有外）。
- M2 の副作用：付属書の `V2 設計書 4.9（EVT-1）` と `M-R4` の文が、旧い縮小動作を書いたままなので、W8 で直す（上の M2 の項目と同じ）。
- T8（付属書 D7 3.7、lane E、commit `636c4794`）— **差戻し**。`crates/kivact-runtime/src/reactive/data.rs:331` の `OwnerEntry` の `extra`（`Option<Box<OwnerExtra>>`）が `xtask check` の `state-fields` に「OwnerEntry combines state fields: extra」で当たる。付属書は marker を指定していないので足していない。設計者が marker の文言を決めるか、構造を変えるかを決める。M11 の残りの差分（`M11-leftover.patch`）は T8 の作業だったので、T8 が取り込んだ。
- G1（付属書 D6 3.1・3.2、lane F、commit `ef29066e`）：`arvane::RendererChange` に `GpuStarting { cause }` を足したため、`kivact-engine`（`host/testing/test_host.rs:199` ほか）と `kivact-test` の match が compile しない。直しは G2（P6-2、本来 W2）の範囲。W1 の終わりに compile が通る必要があるため、G2 を lane F の続きとして W1 に前倒しした（同じ lane の続き。他の lane の変更は前提にしない）。付属書の順と違う点はこれだけ。

## W1 の結果（2026-11-10、設計見直しのため W1 の後で停止）

- 統合：lane A〜F を `refactor/kashiwade/slim-fast` に merge（`b2177e0f`〜`0b5e0f80`）。`cargo fmt --all -- --check` 成功。`cargo check --workspace --all-targets --all-features` 成功（exit 0、エラー 0）。
- 完了した unit：F1（A）、F3・M2・F4（B）、F2・F5・F6（C）、M1・M7（D）、M11・T8・M10・M12（E）、S1・S2・G1・G2（F）。
- 完了しなかった unit：M6（D、差戻し、commit なし）。S4（F、revert して W2 へ移動。元の commit は `3c284b0b`）。
- xtask の `state-fields` が残る unit（設計者判断を待つ）：F5（`Indexed`）、M7（`Plan`）、T8（`OwnerEntry.extra`）。
- 付記：M12 の nogpu の K6 の数値は、S6（W3）で `dev/bench/Cargo.toml` の `gpu` が外れるまで正しくない。
- 付記：F4 の `invalidation.rs` の `invalidate` の doc、M2 の `EVT-1`・`M-R4` の文は、W8 で直す。
- 付記：W1 の終わりの時点で W2 以降の unit は始めていない。




## W2 の結果（2026-11-10）

- 統合：lane A〜F を `refactor/kashiwade/slim-fast` に `--no-ff` で merge（conflict なし）。統合 HEAD は W2-F merge の `7a907e69`。
- 完了した unit と commit（lane 順）：
  - A：AH1 `15c1aa50`
  - B：F5-fix `d22b2e12`、F4-fix `75cd6329`、F7 `0ec4a8c2`
  - C：S45 `edd34df2`、T4 `2dc7bf46`、S8 `c2815cf8`、S9 `2f41d9fa`
  - D：M7-fix `0aab46cd`、M6 `0b3d3ca7`、M8 `d815c624`、T1 `5a632ea3`、G3 `eca6a920`
  - E：T8-fix `bea69f63`、S7 `19309b7a`
  - F：M9 `01361150`
- 統合 tree（`W:/kv-target/int`）の確認：`cargo fmt --all -- --check` 成功、`cargo check --workspace --all-targets --all-features` 成功、`cargo clippy --workspace --all-targets --all-features -- -D warnings` 成功、`cargo xtask check --no-write` 成功（`analyzed 49 workspace crates`、ok。xtask の path は `W:\w\kivact`）。
- 統合 tree の E2E と unit：kivact の E2E（各 lane が名指したもの、`hover` から `constant_semantics_publish_once_and_keep_no_publisher` まで）216 件成功。arvane の E2E `text` 9 件成功。`kivact-runtime`・`arvane-text`・`kivact-ui-basic-catalog` の unit 成功。
- W1 の差戻しの解消：F5（`Indexed` の state-fields）は F5-fix で enum `Reach`（DEC-24）。M7（`Plan`）は M7-fix で `CheckFold`（DEC-24）。T8（`OwnerEntry.extra`）は T8-fix の marker（付属書の文言）。M6 は D10 の遅延の draw effect（DEC-11 の取消し）。S4 は S45 で完了。G1・G2 は W1 で完了。
- 確認を求める点（設計者）：
  1. T4：付属書 D10 の `xtask-allow(state-fields): cache:` marker は、xtask が「unused marker」の error を出したため削除した（allow は足していない）。試験は `arvane` の font を `include_bytes!("../../arvane/assets/fonts/Tuffy.ttf")` で読む cross-crate の参照。公開の前に見直す。
  2. S45：fontdb の版は lock file では 0.23.0、付属書は 0.24.0。0.23.0 のまま（compile・試験は通る）。
  3. S8：D11 の S8 節への 1 行の読み替え（D15 の末尾）は、所有外のため行っていない（build に影響しない）。
  4. G3：drawable の GPU export は device を要求しなくなり、App は `adapter()` を読む（D15 Q3）。既存の GPU の E2E（`gpu_on_demand` 系）は W2 では実行していない。W3 と CP1 の GPU の確認で確かめる。
- 作業中に起きた問題（記録）：
  1. 共有 target の衝突：W2 の途中で lane が AB・CD・EF の target を共有した。cargo は同じ相対 path の workspace crate に同じ metadata hash を付けるため、別の worktree の成果物を再利用し得る。また `cargo xtask` は、同じ target で別の worktree から build された binary を実行し、別の worktree を解析する（`CARGO_MANIFEST_DIR` が埋め込まれる）。これで確認が汚れた unit（M7-fix、M9、S45 の xtask など）は、専用 target での再確認と統合 tree の確認で確かめ直した。W3 以降は lane ごとの target（`W:/kv-target/<L>`）を使う。
  2. lane の subagent が自分の loop のプロセスを止めようとして `Stop-Process` を使い、自動判定で拒否された。以後の subagent への指示に「他の worktree のプロセスや cargo を止めない」を入れた。
- 次：W3（M3→T3、AH3→T7→AH2、T6→S6、G4→S3）。CP1 はその後。
