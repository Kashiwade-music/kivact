# kivact-bench

Kivact の性能の指標（K1〜K6）を手動で測る probe と、それをまとめて実行する `run.sh`。CI では実行しない。

```bash
# 何も描画していない状態で、window-lock を通して実行する（release の build を 9 回行う）
dev/tools/window-lock.sh --timeout 5400 -- dev/bench/run.sh <label>
```

`run.sh` は `dev/bench/results/<label>.md` に K1〜K6 を書く（このディレクトリは commit しない）。改善や後退は、比べる二つの commit（例：変更前を `base`、変更後を `head`）を同じ機械・同じ条件で測った report の比で判定する。build の失敗、frame を出さない probe、`T main` の欠落、5 秒で 100 frame 未満の `kv_stress`、17 個の場面を出さない `kv_latency`、heap の計測の失敗のどれかがあると、report の末尾に `INVALID:` を書いて非 0 で終わる。INVALID の report は比較に使わない。表示周期に対して fps が 8 割未満なら `WARNING` を書く（OS が window の描画を間引いた。work の時間は有効だが、fps と描画間隔は使えない）。

| ID | 指標 | 測り方 | 単位 |
|---|---|---|---|
| K1 | 起動時間 | `kv_startup`（counter と同じ画面、800×600）：`T main`（`main` の先頭）から `KIVACT_FRAME_LOG` の最初の `F` 行（最初の present の完了）まで、5 回の中央値。basic catalog：起動から最初の `F` 行まで（catalog は `T main` を出さないので OS のプロセス生成を含む） | ms |
| K2 | メモリ | 3 秒の idle の後の working set と private bytes（counter と同じ画面、basic catalog、3000 行の静止画面）。item あたり＝`kv_stress STILL=1 MODE=<種類>` の (WS(3000 個) − WS(0 個)) ÷ 3000（CPU） | KiB、KiB/item |
| K3 | CPU | idle 10 秒の CPU 時間と idle 中の frame 数。`kv_stress`（毎 frame 1 つの text を書き換える）の CPU 使用率 | ms/s、1 core に対する % |
| K4 | フレーム時間 | `kv_stress` の frame の UI thread の時間（work）の p50/p95/p99 と描画間隔の p99。CPU の全画面 frame＝`kv_startup`（CPU）の最初の frame の raster 時間 | ms |
| K5 | 更新の遅延 | `kv_latency`：試験 driver（CPU、`KIVACT_TEST_MIRROR=off`、a11y の公開あり）で、座標の click・pointer の移動・wheel の入力から描画の完了まで | ms |
| K6 | バイナリの大きさ | release の実行ファイル。既定の feature（`gpu` 込み）の counter・`kv_startup`・basic catalog、GPU 一式を除く build（`--no-default-features --features complex-scripts`）の `kv_startup`・basic catalog、最小構成（`--no-default-features`）の `kv_startup`。Kivact 自身の大きさは GPU 一式を除く build で比べ、既定の build は大きくしないことを確かめる | byte |

package は一つずつ build する（feature の統合で他の package の feature が混ざらないため）。K5 の試験 driver は毎回 a11y の木を公開するので、K5 は支援技術が有効なときの更新に近い。支援技術なしの更新の費用は K4 の work が表す。OS のプロセス生成（spawn から `main` まで）は K1 の counter に含めない。

| binary | 内容 |
|---|---|
| `kv_startup` | K1・K6：`main` の先頭と root の setup で時刻を出す counter。`WIN_W`・`WIN_H` で window の大きさを変える |
| `kv_stress` | K2・K3・K4：`ROWS` 個の静的な item と、毎 frame 書き換わる 1 つの text。`MODE`=row/text/box/button/checkbox、`STILL=1` で frame loop を止める（K2 の大画面の idle） |
| `kv_latency` | K5（feature `latency`） |
| `kv_heap` | K2 の原因の診断（指標の行ではない）。計数 allocator で、root の mount から `SETTLE_SECS`（既定 3）秒後の生存 byte を割当て元の関数ごとに item あたりで出す。`CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=line-tables-only` で build する（release の profile は strip するため）。`CHAIN`（既定 1）で site に出す Kivact/Arvane 関数の数（` <- ` でつなぐ）、`TOP`（既定 40）で出力行数を変える |

Windows では `wmic`（無ければ PowerShell の CIM）を image 名で引く（MSYS の pid は Windows の pid と一致しない）。Linux では `/proc` を読む。

K2 を単独で測るときは、build 直後の最初の起動を捨てる（初回だけ exe の image 全体が WS に入るため。`run.sh` は K1 の後に K2 を測るので影響しない）。
