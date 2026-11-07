# Kivact 表現力パリティ 実装 現状レポート

- 対象ブランチ: `feat/kashiwade/visual-parity`（push 済み、最新 `ca77956b`）
- 作業指示書: `VISUAL_PARITY_WORK_ORDER.md`（版 1.5）／設計書: `VISUAL_PARITY_DESIGN.md`
- 作成日: 2026-11-07

## 1. 要約

- wave 1〜8 のユニットを実装し、チェックポイント CP1〜CP7 を通過しました（CP2 と CP4 は Linux の WSL 検査も通過）。
- wave 8 の lint 8 rule（state-fields、effect-writes、api-shape、canonical-path、name-collision、kit-literal-color、os-branch、structural-options）は workspace 全体で 0 件です。
- **未完了**: CP8（既存試験の一式、ウィンドウ試験を含む）、wave 9（新規 E2E 試験・文書・catalog）、CP9（最終検査）、`PR_DRAFT.md`。
- 作業指示書 CP8 が要求する `cargo xtask check`（全体）は **315 件のエラー**で未通過です。内訳は §5 を参照してください。
- 実行時の検証（試験の実行、描画の確認）は一度も行っていません。compile と lint の検査だけです。

## 2. コミットの対応表（ブランチ上の順）

| 区分 | 内容 | マージ commit / 主な commit |
|---|---|---|
| 設計と作業指示書 | 設計書、作業指示書、Codex レビュー | `3088cff4` 〜 `e9430d93`（f4e40cc 以降の先頭） |
| wave 1 | U11 kivact-types | `567a37e7`（CP1 の clippy 修正を含む） |
| wave 2 | U21 runtime、U22 shader、U23 render-raster、U24 shell/native/virtual、U25 text/asset、U26 compose-plan | `d062e94f` 他、CP2 修正 `5b20cabe` |
| wave 3 | U31 compose-cpu、U32 compose-gpu-core、U33 style、CP3 修正、F31（CPU シェーダ fallback と external 写像の修正） | `4a1758b4`（U32 の衝突解決）、`f7dec656`、`29edbf9d` |
| wave 4 | U41 presenter/engine base、U42 compose-gpu-effects、CP4 修正（承認済み unsafe の hand-off を含む） | `f1e590ca`、`e5fed289` |
| wave 5 | U51 layering、U53 lower vocab、U52 geometry/3D、U54 motion、U55 core base、CP5 修正 | `aaa1b73b`、`fce472d9`、`7a7ed85c` |
| wave 6 | U64 interaction/ui（完全）、U61 view motion（一部）、U62 drawables（一部）、CP6 修正、prelude の `drawable` | `bfed104a`、`c44b8364`、`3509725b`、`21fbe5a4` |
| wave 7 | U71 kit basic/pop、U72 kit game/creative、U73 kit soft（一部）、U74 examples/template/producer、StepPosition の再エクスポート | `03fc3c81`、`4c205c5f`、`fc942f2a`、`2d5e6b58` |
| wave 8 | U81 既存試験・docs の compile、U82 lint、F53（承認済み BorderStyle の 5 バリアントと `ReportOnceSet::retain`） | `7da9283d`、`ca77956b` |

## 3. 承認いただいた判断と、その反映

1. **`unsafe` の例外（承認済み）**: `arvane-compose/src/gpu/export.rs` の hand-off 関数 1 つに限定。`#[allow(unsafe_code, reason = ...)]` を関数単位で付け、`SAFETY` コメントを付けた。他の crate は deny のまま。
2. **`BorderStyle` の Double、Groove、Ridge、Inset、Outset を追加（承認済み）**: `kivact-types/src/scene/primitives.rs`。engine の `lower/border.rs` も対応済み。
3. **`ReportOnceSet::retain` を追加（承認済み）**: `kivact-types/src/cell.rs`。engine の `reported_images`（SM-05）の pruning に使用。
4. **export の送信判定は plan の署名を採用（承認済み、解釈）**: U42 の方式（`ExportRecord` の generation・mode・signature・size・scale）を採用。U31 の画素ハッシュ方式は使わない。
5. **`TransitionProp::Paint` → `All` は §17.3 の表どおり（承認済み）**: 設計書に `All` と `Paint` の挙動差の記載は無いため、E2E で確認が必要。

## 4. 作業指示書からの逸脱（記録）

- **clippy の `-A dead_code`**: CP2〜CP7 では、後続の wave で配線される dead code を除外して `-D warnings` を実行。CP8/CP9 の strict clippy（dead code を含む）は未通過（§5-2）。
- **WSL の検査**: 作業指示書のパス `C:\w\kashiwade\kivact` は存在しないため、`/mnt/w/w/kivact` を使用。Ubuntu-24.04 には Rust が無いため、`Ubuntu` ディストリビューションで実行。
- **公開 API の変更（作業指示書 §17.3 に無いもの）**: `kivact_ui_creative::timeline` を `timeline_editor` に改名（U82、A-4 に基づく。prelude の `timeline` と衝突していたため）。docs の `ui::timeline(...)` の参照は未修正。
- **lint 例外（A-5 の範囲）**: U82 が `xtask-allow(state-fields)` の理由付き例外を 2 構造体・8 フィールドに付与（`LayerFrame`、`Interface`）。U11 は `api-shape` の例外を Arc の共有ペイロード 12 箇所に付与（設計書が Arc を明記しているため）。U21 ほかにも A-5 の例外あり。
- **ワーカーの方式**: サブエージェントは全て Haiku 5.5（effort high）で実行。
- **衝突の解決**: U32（wave 3）の衝突を手動で解決（`plan.rs` は U31 の enum 化を採用、`lib.rs` は両方の公開面を統合）。
- **`DrawOp` の enum 化**: `scissor`、`nearest`、`cacheable` を enum（`Scissor`、`Sampling`、`Caching`）へ変更（U31 の lint 対応）。

## 5. 未完了・既知の問題

### 5-1. `cargo xtask check`（CONTRIBUTING の pre-merge 検査）: 315 件

- `style/non-leaf-implementation`: 210 件。モジュールファイルに glue code がある。移動するか理由付き例外が必要（rule が求める対応）。
- モジュールの参照循環: `arvane-compose`（`cpu::filters`↔`cpu::graph`、`gpu::{exec,export,filters,shader,state}`、`plan` 系）、`arvane-render`（`gpu`↔`gpu::encode`）、`kivact-core`（`view::elements::builders`↔`view::mount::drawable`）。設計レベルの構造変更が必要。
- `Role` 行: 80 文字超過、または `lib.rs` に無い（`arvane-compose` ほか）。
- `style/rustfmt-skip` 2 件、`style/import-granularity` 1 件、`style/flexible-references` 1 件。
- policy 違反: `kivact-core` の `view` が `platform` を使用（`dev/xtask/policy/workspace.toml` の `[module_rules.kivact-core.uses]` に未登録）。
- 件数の内訳（crate 別）: `arvane-compose` 147、`kivact-types` 59、`kivact-style` 30、`kivact-runtime` 19、`kivact-engine` 19、`kivact-ui-creative` 18、`kivact-core` 10、`kivact-ui-basic` 9、`arvane-render` 9、`kivact-ui-pop` 6、`kivact-ui-game` 4、`kivact-shader` 4、`arvane-native` 2、`arvane` 2。

### 5-2. strict clippy（`-D warnings`、dead code を含む）

- `arvane-compose` で 8 件（未読のフィールド `retain`、`index`、`0`、`projective` メソッド、構築されていないバリアント `Pixels`）。plan の配線が未完了であることを示す。

### 5-3. ユニット別の未完了項目

- **U26**: 3D 参加面へのクリップが `PlanError`（coverage 空間の設計上の衝突、A-2 で停止中）。
- **U31**: resvg 移植の THIRD-PARTY-NOTICES 追記（§4.5）が未実施。
- **U32**: shader ops の GPU 側は U42 で実装済み。`bytemuck` 未追加。§8.13 は簡略化（A-3）。`kivact-gpu-share` の device は再エクスポート止まり。
- **U33**: `kivact-core` での style reports の転送（`CompiledStyle::reports`）、`Node::playing`、`DrawableSource` の実装（U62 側）、`SpanStyle` の stroke・variations・font_width。
- **U42**: convolve の表を uniform ではなく storage buffer で渡している（値は同一）。filter shader のサイズが layer の境界ではなく target px / ppu。`ShaderOp.input` は未対応。export のエラーは `ComposeError::Backend`（`FailureKind::Unavailable` ではない）。`NotReady` は U41 側の課題。
- **U51**: R6（drawable レイヤー、U62 へ）、§6.2 の perspective（Pp が子に伝わらない）、3D レイヤーのカリングが不完全、§6.8 の shader fill（shape path の fill は未対応）、§10.5 の exports が空、§11.2.1（画像未準備）の扱い、shader mask の黙殺。
- **U52**: 3D コンテキストの hit test 分岐（参加面、深度順、`MAX_CONTEXT_PLANES`）が未実装。`a11y` の `Option` bounds（§7.4）が未実装。
- **U53**: §9.1 は F53 で完了。§9.2 の border image は未配線（`paint/` 側のラッパーが U51 の所有）。§9.3 の origin と clip（padding box）未実装。§9.4 background-clip: text 未実装。§9.7 の text stroke・fill・order は未実装（`push_text` と `ShapedText` の拡張が必要）。
- **U54**: ShaderClocks と shader time 未実装（consumer がまだ無い）。背景レイヤーの再生（スロット 1 以降）未配線。`layout_id` の重複診断が未報告。
- **U55**: `dev/xtask/policy/workspace.toml` の `[module_rules.kivact-core.uses] sync = []` と `gpu_interop` の uses（§10.5）が未実施（xtask 検査で検出済み、§5-1）。
- **U61**: `layout_id`（`kivact-style` の `Style` にフィールドが必要）、`playing`（`Common` フィールドとエンジンの `PropValue::Playing` の setter が必要）が未実装。
- **U62**: canvas の drawable 描画（`DrawCmd::Drawable` がまだ skip）、`a11y` の `Option` bounds（`kivact-types` の wire 変更が必要）、region のエラー（"not hosted by this node"、"drawn twice"）、テスト API（`ArvaneTestHost::frame_stats`、`Screen::frame_stats`、`lose_gpu_device`）が未実装。
- **U81**: docs の散文が旧 API のまま（`docs/for-kivact-user/{en,jp}/getting-started/06-style-and-accessibility.md` の `TransitionProp::Paint`、`docs/for-kivact-user/en/how-to-guide/animation.md` の `steps_end`/`steps_start`）。`docs/for-kivact-ui-user/{en,jp}/creative/how-to-guide/timeline-and-arrangement.md` の `ui::timeline` を `timeline_editor` に改名する必要あり。
- **U73**: 完了（`StepPosition` の再エクスポートは fc942f2a で追加）。
- **U74**: 完了（producer の frame 検査を `SHARED_PREPARED` に合わせて修正）。

### 5-4. 未実施の作業指示書の項目

- **CP8**: 既存試験の一式（`cargo test --workspace` と、ウィンドウ試験を `dev/tools/window-lock.sh` 経由で実行）、`dev/tools/check-features.sh`。ウィンドウ試験は未実行。
- **wave 9**: U91〜U93（新規 E2E 試験、§17.2 の一覧）、U94〜U95（英日の文書、§16.1）、U96（catalog の試験、§16.2〜§16.4、THIRD-PARTY-NOTICES）。未着手。
- **CP9**: 全検査、GPU lane（GPU 付き Windows）、Linux（WSL）の Wayland と X11 の試験。未実施。
- **`PR_DRAFT.md`**: 未作成（PR の作成と squash merge はユーザーが行う）。

## 6. 検証の状態

| 項目 | 状態 |
|---|---|
| `cargo fmt --all -- --check` | 通過（ca77956b 時点） |
| 各 checkpoint の compile（both feature sets） | CP1〜CP7 通過 |
| clippy（`-D warnings -A dead_code`、CP ごとの A/L 集合） | CP1〜CP7 通過 |
| clippy（strict、workspace 全体） | 未通過（dead code 8 件、§5-2） |
| WSL（Linux）の check | CP2、CP4 で通過 |
| lint 8 rule（workspace 全体） | 0 件 |
| `cargo xtask check`（全体） | 未通過（315 件、§5-1） |
| 単体・doc 試験（`kivact-types` の doctest） | 通過（12 件、1 件 ignore） |
| 既存の試験の実行（`cargo test`、ウィンドウ試験） | **未実施** |
| E2E 試験、GPU 試験 | **未実施** |
| 描画結果の目視確認 | **未実施** |

## 7. 環境メモ

- 作業のため `vendor/xtask-core`（submodule）を各 worktree に複製して使用。commit には含めていません（gitlink のまま）。
- 古いサブエージェントの worktree が `.claude/worktrees/agent-*` に 3 件残っています（作業とは無関係の残骸。削除は未実施）。
- ディスクの空きは約 55 GB（60 GB 未満のため、ビルドスロットを共有する運用）。`target/` の一時的なキャッシュは整理済み。

## 8. 次の選択肢（判断待ち）

- **A. 継続**: (1) `xtask` の style・policy 残件（§5-1）を crate 単位で並列に解消、(2) `arvane-compose` の dead code を plan の配線で解消（§5-2）、(3) 参照循環の構造変更、(4) 残りの未完了項目（§5-3）を順に実装、(5) CP8（既存試験）、wave 9、CP9、`PR_DRAFT.md`。
- **B. ここで区切る**: 本レポートを基に、未完了項目と判断事項をユーザー側で整理してから再開する。
