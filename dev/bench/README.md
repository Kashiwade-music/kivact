# kivact-bench

KPI の手動計測（`SLIM_FAST_DESIGN.md` 1 章）。CI には組み込まない。

```bash
# 何も描画していない状態で、window-lock を通して実行する
dev/tools/window-lock.sh --timeout 1800 -- dev/bench/run.sh B0
```

`dev/bench/results/<label>.md` に K1〜K6 を書く。改善率は同じマシンで取った `B0`（変更前）との比で判定する。

| binary | 計測 |
|---|---|
| `kv_startup` | K1: `T main` から `KIVACT_FRAME_LOG` の最初の `F` 行（present 完了）まで。800×600。counter と同じ画面 |
| `kv_stress` | K3/K4: `ROWS` 個の静的な行と、毎 frame 書き換わる 1 つの text。`MODE`=row/text/box/button/checkbox、`STILL=1` で frame loop を止める（K2 の大画面アイドル） |
| `kv_heap` | K2: 計数 allocator で、マウント 6 秒後の生存 byte を割当て元の関数ごとに item あたりで出す。`CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=line-tables-only` で build する（release の profile は strip するため）。`CHAIN`（既定 1）で site に出す Kivact/Arvane 関数の数（` <- ` でつなぐ）、`TOP`（既定 40）で出力行数を変える |
| `kv_latency` | K5: 試験 driver（`KIVACT_TEST_MIRROR=off`、CPU、a11y 公開あり）で、座標 click・pointer 移動・wheel から描画完了までの時間 |

Windows では `wmic` を image 名で引く（MSYS の pid は Windows の pid と一致しない）。Linux では `/proc` を読む。

K2 を単独で測るときは、build 直後の最初の起動を捨てる（初回だけ exe の image 全体が WS に入るため。`run.sh` は K1 の後に K2 を測るので正式値には影響しない）。
