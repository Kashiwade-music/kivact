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



