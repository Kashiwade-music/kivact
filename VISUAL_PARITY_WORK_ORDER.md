# Kivact 表現力パリティ 作業指示書

版: 1.5（2026-11-07。`20261107_1305_CODEX_REVIEW.md`、`20261107_1333_CODEX_REVIEW.md`、`20261107_1402_CODEX_REVIEW.md`、`20261107_1425_CODEX_REVIEW.md` の指摘を反映）。設計: `VISUAL_PARITY_DESIGN.md`（以下「設計書」）。

この文書は、設計書を実装する手順を定める。実装はオーケストレータ（あなた）がサブエージェントに割り振って行う。サブエージェントは設計の判断をせず、指定された設計書の節に従って実装する。

---

## 0. 全体の規則

### 0.1 役割

| 役割 | 担当 |
|---|---|
| オーケストレータ | worktree の作成、unit の割り振り、merge、checkpoint の検査、設計書の付録 A による裁定、PR 説明の草稿、integration branch の push |
| サブエージェント | 一つの unit の実装。指定の節を読み、指定の file を変え、unit の完了条件の command を通して commit する |

**サブエージェントへの指示に必ず含めること:**

1. この文書の該当 unit の全文と §0 の全文。
2. 「設計書の指定の節だけを読む。追加の設計や探索をしない。指定と実コードがずれていたら設計書 §0.2 に従う。意味の衝突は、オーケストレータに報告して指示を待つ」。
3. 「互換性のためのコード（alias、deprecated、二重経路）を書かない。unit の範囲外の crate が自分の変更で compile できなくなったら、設計書 付録 A の A-7 のとおり、意味を変えない最小の修正（import、型の追従）だけをして commit message に書く。範囲外の crate の機能は実装しない」。
4. 「PR を作らない。main に merge・push しない。integration branch に push しない」。
5. 「設計書の挙動を、依存 crate の制約などで実装できないと分かったら、近似を実装せず、その部分を止めてオーケストレータに報告する（設計書 付録 A の A-2）」。

**止めた部分の扱い:** オーケストレータは、A-2 で止まった部分を、代替案と影響（公開 API、許容差、試験）とともにユーザーへ報告し、決定を待つ。その間、依存しない unit は進める。決定は設計書に書き足してから、その部分を再開する。

### 0.2 速度の規則

- **並列数:** 一度に動かすサブエージェントは 6 つまで。wave の unit はすべて同時に動かす。wave の分け方は、各 unit の完了条件が、wave の開始時の HEAD から作った worktree の中で満たせるように決めてある（同じ wave の unit の成果に依存しない）。
- **squash merge 前提:** PR は squash merge されるので、integration branch の途中の commit は、wave の範囲外の crate が compile できなくてもよい。
- **unit の中で実行してよいもの:**
  - `cargo fmt --all`
  - 完了条件の `cargo check`
  - 完了条件の `cargo xtask check --no-write --only <rules> --crate <crate>`
  - 完了条件に書かれた試験。window を開かない doctest（dev-dependency に `kivact` を持たない crate だけ）は、どの wave でもよい。window を開く試験は wave 9 と、CP8 の修正の unit だけ。
  - 単体試験（`cfg(test)`）は作らない（`docs/for-kivact-developer/decisions.md` の Q-01。設計書 §17.1）。新しい振る舞いは wave 9 の E2E で確かめる。
- **unit の中で実行しないもの:**
  - workspace 全体の `cargo test`
  - `cargo clippy`
  - 引数なしの `cargo xtask check`（rustfmt と Clippy を workspace 全体で走らせ、architecture を書き換えるため）
  - WSL の検査

  これらは checkpoint でオーケストレータが行う。
- **build の slot:** worktree ごとに `CARGO_TARGET_DIR=<repo>/target/slot-<n>` を使う。n は wave の表の行番号（1〜6）で、slot は wave をまたいで再利用する。
  - disk の空きが 60 GB 未満なら、slot を 3 つにして 2 unit ずつ共有する（その場合、同じ slot の build は順番待ちになる）。
  - checkpoint は repo の既定の `target/` を使う。
- **workspace の解決を壊さない:** cargo は、どの command でも workspace の全 manifest を解決する。そのため、次の変更をする unit は、workspace 内の全 `Cargo.toml` の参照を同じ commit で直す。
  - crate の追加
  - feature の削除・改名
  - 依存の削除

  新しい crate を path 依存に加えるのは、その crate が同じ worktree に存在するときだけにする。

### 0.3 git

- **integration branch:** `feat/kashiwade/visual-parity`（`main` @ f4e40cc から作る）。最初の commit で `VISUAL_PARITY_DESIGN.md` とこの文書を追加する。
- **worktree:** unit ごとに作る。各 wave の開始時の integration branch の HEAD から作る。

  ```text
  git worktree add ../kivact-wt/<unit-id> -b wt/<unit-id> feat/kashiwade/visual-parity
  ```

- **unit の完了:** サブエージェントは worktree で commit して終わる（commit の数は問わない）。
- **merge:** オーケストレータは、wave の unit を表の順に `git merge --no-ff wt/<unit-id>` で integration branch に取り込む。conflict は設計書に従って解く。
- **後片付け:** wave の checkpoint を通したら、integration branch を `git push origin feat/kashiwade/visual-parity` で push し、その wave の worktree と `wt/*` branch を消す。force push はしない。
- **終わり:** 最後に、PR 説明の草稿を repo の外の作業 dir の `PR_DRAFT.md` に書き、ユーザーに渡して止まる。PR の作成と squash merge はユーザーが行う。

### 0.4 unit の書式

各 unit は次の項目を持つ。

- **節:** 読む設計書の節
- **範囲:** 変える crate と主な file（新は新規）
- **内容:** 実装すること
- **完了条件:** unit が終える前に通す command

完了条件の `cargo check` は、特記が無ければ次の 2 回を行う（feature を持たない crate は 1 回）。

```text
cargo check -p <crate…> --all-targets --all-features
cargo check -p <crate…> --all-targets --no-default-features
```

`--all-targets` には、その crate の試験（`tests/` と inline の `cfg(test)`）を含む。試験の compile も unit の責任である。

例外: dev-dependency に facade の `kivact` を持つ crate（`kivact-runtime`、`kivact-i18n`、`kivact-style`、`kivact-ui`、五つの kit）は、`kivact` が compile できる wave 6 の終わりまで、`--all-targets` の代わりに `--lib` で check する。それらの試験の compile は、wave 7（kit）と CP8 で確かめる。

---

## 1. wave の一覧

| wave | 内容 | unit |
|---|---|---|
| 1 | kivact-types の語彙すべて | U11 |
| 2 | types だけに依存する crate | U21〜U26 |
| 3 | compose の CPU と GPU の中核、style | U31〜U33 |
| 4 | arvane の presenter と engine の基盤、compose の GPU の効果 | U41、U42 |
| 5 | engine の機能、core の基盤 | U51〜U55 |
| 6 | core の機能、interaction と ui | U61、U62、U64 |
| 7 | kit と example の移行 | U71〜U74 |
| 8 | 既存の試験と文書の compile、lint | U81、U82 |
| 9 | 新しい試験、文書、見本 | U91〜U96 |

checkpoint は各 wave の後に行う（§10）。CP8（既存の試験一式）と CP9（最終の全検査）が重い検査である。

### 1.1 共有する API の作成と利用

複数の unit が使う API は、利用する unit より前の wave で作る。同じ wave の中では、ほかの unit が作る API を使わない。

| API | 作る unit | 初めて使う unit | 使える checkpoint |
|---|---|---|---|
| kivact-types の全語彙（`Composition`、`Matrix4::shows_back`、`MAX_CONTEXT_PLANES`、`prop_interp`/`PropInterp`/`PropBasis`（`same_shape`）、`FailureKind::NotReady`、`spring_advance` を使う `Transition::step`、`Spring::rest`/`Spring::problem`、`SpringRun::start`、`FilterGraph<I>` と `map_images`、`RasterGrid`、`FrameReport`（`pending_exports`）、`FrameStats`、`ExportMode`/`ExportFrame`/`ExportedFrame`（generation の envelope）/`ExportLoss`、意味情報の `Option` の bounds） | U11 | wave 2 の全 unit | CP1 |
| `kivact_shader::assemble` | U22 | U33、U42 | CP2 |
| `arvane_render::{CpuRaster, GpuRaster, glyph_outline}` | U23 | U31、U32、U41 | CP2 |
| `ShellWindow::present_with`（damage と `BufferAge`） | U24 | U41 | CP2 |
| `arvane_compose::plan`、`cache`（crate 内部。`plan::build` の error は crate 内の `PlanError`。executor が `ComposeError::InvalidComposition` に変える） | U26 | U31、U32、U42 | CP2 |
| `CpuCompositor`（`CpuOutput::report`、`Rendered::report`）、`GpuContext::present`、`FrameReport` の値 | U31、U32 | U41 | CP3 |
| `arvane::Window::present(&Composition, ..)`、`Presented::Shown { exports, pending_exports, stats }`、`export_drawable`、`drop_export`、`take_export_notices`（`ExportNotice`、presenter の `ExportRegistry`）、`arvane::spi::lose_gpu_device` | U41 | U51、U62 | CP4 |
| engine の `Host::last_stats`、`ArvaneTestHost::frame_stats()`、`ArvaneTestHost::lose_gpu_device()` | U41 | U55 | CP4 |
| engine の `Model::{local_matrix, to_parent, window_map, map_between, bounds, to_local}`（§7.1、§7.2） | U41 | U51、U52、U62 | CP4 |
| engine の state motion の全体（`PropTrack`、`TrackPhase`、`StateMotion::moves_layer`。§12.9.3） | U41 | U51 | CP4 |
| `Pool::try_acquire`（kivact-gpu-share） | U42 | U62 | CP4 |
| kivact-core の再公開と `Screen::frame_stats`、`Screen::lose_gpu_device`、`Screenshot::raster_difference`/`RasterDifference` | U55 | U61、U62、U64、wave 9 | CP5 |
| `GpuContext::export_drawable`/`drop_export`（全 feature の組み合わせで同じ署名。U32 は stub、U42 が中身） | U32 | U41 | CP3 |
| `Drawable`、`DrawableShare`（`wait`、`ShareUpdate`）、drawable の engine の API | U62 | U64（prelude）、U93 | CP6 |

---

## 2. wave 1

| 行 | unit | 名前 |
|---|---|---|
| 1 | U11 | types |

### U11 types

- **節:** §5 のすべて。加えて次の節:
  - §6.3 の `Shape::transformed`
  - §9.10 のうち kivact-types に関わるもの（無し）
  - §11.2 の `TextShaper::outline`（`kivact-types/src/text_shaping.rs`）
  - §12.1〜§12.3、§12.9.1、§12.9.2（`PaintVec` を削除し、`prop_interp`/`apply_components`（`PropInterp`、`PropBasis`）を作る）
  - §8.5 の `Matrix4::shows_back`、§8.12 と §5.11 の `Export`/`ExportMode`/`ExportFrame`/`ExportedFrame`/`ExportLoss`、§8.15 と §5.11 の `FrameReport`/`FrameStats`、§5.15 の意味情報の `Option` の bounds、§8.14 の `MAX_CONTEXT_PLANES`
  - §12.2 の `spring_advance`（解析解）、`Spring` の `rest` と `problem`、`SpringRun::amplitude` と `SpringRun::start`（`restart` を置き換え、workspace の呼び出しを移す）、§12.3 の反復の終わりの値（`Transition::step` と `sample_timed`）
  - §12.9.2 の `PropBasis::same_shape`、§8.12 の `FailureKind::NotReady`（`FailureKind` の match の追従を含む）
  - §8.12 の `SHARED_PREPARED` と、`SharedTextures` の rustdoc の契約の順序と frame の値の変更（`kivact-core` の `ShareLink::provide` の `# Safety` の文言の追従は U55）
  - §5.9 の `FilterGraph<I>`、`Primitive<I>`、`PrimitiveOp<I>`、`map_images`（wire は `FilterGraph<Asset>`、composition は `FilterGraph<Pixels>`）、§5.11 の `RasterGrid` と `CompositeLayer::grid`
  - `render::Gradient` の stop は sRGB の `Rgba` のまま（§5 の gradient の段落）。展開は U53 が engine で行う
- **範囲:**
  - `crates/kivact-types/src/**`（新しい file: `geometry3d.rs`、`color_space.rs`、`pixel.rs`、`effect.rs`、`effect/{filter_graph,shader}.rs`、`render/composition.rs`）
  - `dev/xtask/policy/workspace.toml` の `[module_rules.kivact-types]`（§5.14）
- **内容:**
  - 設計書の型と関数を、式どおりに実装する。
  - 色の変換行列は §5.4.1 の値を一字一句使う。easing の式は §12.1 に従う。
  - 各公開 item に、1 行以上の英語の rustdoc を付ける。
  - `render/color.rs` は、中身を `pixel.rs` へ移した後に削除する。
  - glob の再公開を使わない（§5.9〜§5.11）。
  - `Matrix4::decompose` の rustdoc に、§5.3 の 5 つの再構成の例を assert 付きで書く。
- **完了条件:**
  - `cargo check -p kivact-types --all-targets`
  - `cargo test -p kivact-types --doc`
  - `cargo xtask check --no-write --only state-fields,api-shape,canonical-path --crate kivact-types`

---

## 3. wave 2

| 行 | unit | 名前 |
|---|---|---|
| 1 | U21 | runtime |
| 2 | U22 | shader-crate |
| 3 | U23 | render-raster |
| 4 | U24 | shell-native-virtual |
| 5 | U25 | text-asset-layout-editing |
| 6 | U26 | compose-plan |

merge の順: U23 → U26 → U22 → U21 → U24 → U25。

### U21 runtime

- **節:** §12.2（stepper の rest）、§12.3、§12.4、§12.5、§12.6、§12.6.1、§12.7、§11.3.1 の `ErrorKind::Shader`
- **範囲:**
  - `crates/kivact-runtime/src/**`（新しい file: `motion/timeline.rs`、`motion/stagger.rs`）
  - `crates/kivact-i18n/src/**`（compile の追従だけ）
- **内容:**
  - `Animated` を再構成し、`set_velocity` を足す。`finish(Completed)` は §12.3 の反復の終わりの値を publish し、target を `AnimPhase::Completed { target }` に残す（§12.4）。`set_velocity` は §12.4 の遷移表のとおり（Idle と Completed からの run は一回・通常の向き・delay 0）。
  - `animated` の作成と Timeline の区間の追加で `Spring::problem` を報告する（§12.2）。
  - frame loop に `essential` と reduced motion の扱いを足し、`restore_motion` を作る。
  - Timeline（`TimelineTrack`、`At`、`Segment`、`TimelinePhase`）と Stagger を作る。
  - `ErrorKind::Shader` を足す。
  - `kivact-runtime/src/motion.rs` から、新しい公開名を名前の一覧で再公開する。
- **完了条件:**
  - `cargo check -p kivact-runtime -p kivact-i18n --lib`
  - `cargo xtask check --no-write --only state-fields,effect-writes,api-shape --crate kivact-runtime`

### U22 shader-crate

- **節:** §4.2（kivact-shader）、§4.4、§4.5（naga）、§11.3.2、§11.3.3
- **範囲:**
  - `crates/kivact-shader/`（新）
  - root の `Cargo.toml`（members、`[workspace.dependencies]` の naga、`[profile.dev.package]` の `kivact-shader` に `opt-level = 3`）
  - `dev/xtask/policy/workspace.toml`（`visible_to`）
- **内容:**
  - `assemble` と `ShaderError` を作る。parse は 2 回行う（§11.3.3）。
  - 他に、IR の検査、位置の変換、`Layouter` による param のレイアウト、FNV-1a の hash を作る。
  - prelude と入口のテキストは、§11.3.3 の WGSL をそのまま使う。
  - rustdoc の例に、正しい paint shader（`fn paint`）、正しい filter shader（`fn effect`。§11.3.2 の例）、構文の誤りの例、`fn filter` と書いた誤りの例（予約語のエラーになり、位置が利用者のソースの行と列になる）を書く。
- **完了条件:**
  - `cargo check -p kivact-shader --all-targets`
  - `cargo test -p kivact-shader --doc`

### U23 render-raster

- **節:** §4.3、§9.7（arvane-render の部分と glyph の輪郭）、§13.1 の SM-11、§14
- **範囲:**
  - `crates/arvane-render/**`
  - `crates/arvane/Cargo.toml` の dev-dependency の行（feature `gpu-interop` の参照を消す）
  - `crates/kivact/Cargo.toml` の feature（`arvane-render/gpu-interop` への参照を消す）
  - `dev/tools/check-features.sh` の `arvane-render/gpu-interop` の行の削除
- **内容:**
  - `CpuRaster` と `GpuRaster` を作る。
  - 新しい `RenderCmd` を描く（`PushGroup`、`Glyphs` の fill・stroke・order）。gradient は sRGB の stop を既存どおり描く（色空間の補間は engine が §9.6 で展開済み）。
  - `glyph_outline` と `Canvas::stroke_glyphs` を作る。
  - `RenderError::InvalidComposition` を足す。
  - 次を削除する: mask・backdrop・filter の emulation、`backdrop.rs`、外部 texture の描画、`gpu-interop` feature、`kivact-gpu-share` への依存。
  - GPU の device、surface、外部 frame、offscreen の file と `gpu_unbuilt.rs` は、`git mv` で `crates/arvane-compose/src/gpu/` と `crates/arvane-compose/src/gpu_unbuilt.rs` へ移す。arvane-compose は、この時点では workspace の member ではない（U26 が作る crate に、U32 が組み込む）。移した file の中身は変えない。
  - `arvane/Cargo.toml` の dev-dependency は、この unit では `arvane-render = { path = "../arvane-render", features = ["gpu"] }` にする（U41 が compose に替える）。
- **完了条件:** `cargo check -p arvane-render`（`--features gpu` と `--no-default-features`）

### U24 shell-native-virtual

- **節:** §15.1、§15.2、§15.3、§4.5（zbus、x11rb の randr）
- **範囲:**
  - `crates/arvane-shell/**`
  - `crates/arvane-native/**`（新しい file: `src/port/linux_settings.rs`、変更する試験: `tests/smoke.rs`）
  - `crates/arvane-virtual/**`
  - root の `Cargo.toml`（`[workspace.dependencies]` の zbus）
- **内容:**
  - `present_with` に damage と `BufferAge` を足す。
  - native の damage 提示、Linux の reduced motion（portal）、X11 の refresh rate を作る。
  - `arvane-native/src/a11y/conversion.rs` は、意味情報の bounds が None のとき AccessKit の bounds を設定しない（§5.15）。
  - virtual は前フレームを保持する。
- **完了条件:**
  - `cargo check -p arvane-shell -p arvane-native -p arvane-virtual`
  - `cargo check -p arvane-native --features embed`
  - `cargo xtask check --no-write --only os-branch,state-fields --crate arvane-native`

### U25 text-asset-layout-editing

- **節:** §9.7（arvane-text）、§9.8（arvane-asset と AssetStore）、§9.9
- **範囲:**
  - `crates/arvane-text/**`
  - `crates/arvane-asset/**`
  - `crates/kivact-layout/**`（compile の追従）
  - `crates/kivact-editing/**`（compile の追従）
- **内容:**
  - font variation、font width、optical sizing と、cache の key を作る。
  - アニメーション画像の decode、`AnimationInfo`、`pixels(.., frame)` を作る。予算は、次の frame の decode の前に、形式ごとの作業領域 W を含めて見積もる。1 枚目の前に超えるなら既存の静止画の `decode` に回し、2 枚目以降で超えるなら最初の frame の Still にする。GIF は `set_limits`、APNG は `PngDecoder::with_limits` で作ってから `apng()` へ進む（WebP は外側の見積りだけ。§9.8）。
  - SVG の fontdb と、data URL の画像を有効にする。
- **完了条件:**
  - `cargo check -p arvane-text -p arvane-asset -p kivact-layout -p kivact-editing`
  - `cargo xtask check --no-write --only os-branch,state-fields --crate arvane-asset`

### U26 compose-plan

- **節:** §4.1、§4.2（arvane-compose）、§4.4、§8.2〜§8.6、§8.14、§3.3
- **範囲:**
  - `crates/arvane-compose/`（新。`Cargo.toml`、`src/lib.rs`、`src/plan.rs`、`src/plan/*.rs`、`src/cache.rs`）
  - root の `Cargo.toml`（members、`[profile.dev.package]` の `arvane-compose` に `opt-level = 3`）
  - `dev/xtask/policy/workspace.toml`（`visible_to`、`[module_rules.arvane-compose]`、`[os_branch] allowed_paths`）
- **内容:**
  - この unit の `Cargo.toml` の依存は `kivact-types` だけにする（`arvane-render` は U23、`kivact-shader` は U22 と並行なので、まだ参照しない。plan は `ShaderUse` などの kivact-types の型だけを使う）。
  - `lib.rs` は `plan` と `cache` を宣言する。
  - `build`（§8.3。3D と BSP は §8.4、cache と damage は §8.6）を実装する。
  - `bounds.rs` には、旧 `arvane-render/src/backdrop.rs` の `Index` の計算（`git show f4e40cc:crates/arvane-render/src/backdrop.rs`）を、新しい `RenderCmd` に合わせて移す。
  - 次も plan に含める: damage の効果を通る伝播と部分消去の `Clear`（§8.6、§8.13）、`ChunkRecord::retain` と `LayerRecord::retain`（§8.2）、効果の到達範囲での cull（§8.3）、上限の inline とその除外条件、参加面が `MAX_CONTEXT_PLANES` を超えるコンテキストの平坦な重なり（§8.14）、`FrameStats` のうち plan が決める値（§8.15）。
  - `plan::build` の error は crate 内の `PlanError`（§8.2）。`arvane-render` の型を使わない。
  - §8.4 の R の平面の断片（子 layer の位置で区切る）と、同一平面の順の key（item の位置の列）。
  - §8.3 の手順 2 の `RasterGrid::Layer` の 256/s の格子への切り取りの拡大、手順 3 の `RasterGrid` の二つの格子と、§8.6 の `LayerFilter::Graph` の signature の畳み込み（graph の Arc の pointer を使わない）。
  - feature と GPU の module は U32 が足す。
- **完了条件:** `cargo check -p arvane-compose --all-targets`

---

## 4. wave 3

| 行 | unit | 名前 |
|---|---|---|
| 1 | U31 | compose-cpu |
| 2 | U32 | compose-gpu-core |
| 3 | U33 | style |

merge の順: U31 → U32 → U33。

### U31 compose-cpu

- **節:** §8.1（CPU の API）、§8.7、§8.8（CPU）、§8.9、§8.10（CPU の fallback）、§8.11〜§8.13（CPU）、§4.5（resvg の移植）
- **範囲:**
  - `crates/arvane-compose/src/cpu.rs`
  - `crates/arvane-compose/src/cpu/**`（新）
  - `crates/arvane-compose/Cargo.toml`（`arvane-render` を加える）
  - `crates/arvane-compose/src/lib.rs`（cpu の宣言と再公開）
- **内容:**
  - `CpuCompositor` を作る。Op の実行と §8.7 の式、blur、drop shadow、color matrix、mask、clip、backdrop（透過の layer を含む）、外部 frame の CPU の扱い、CPU の export（`ExportedFrame` の envelope）、`FrameStats` の計数、`CpuOutput::report` と `Rendered::report`（§8.1）を実装する。mode が Gpu の Export は書き出さず `pending_exports` に入れる（§8.12）。`PlanError` を `ComposeError::InvalidComposition` に変える。
  - filter graph は、resvg 0.46.0 の `src/filter/`（`~/.cargo/registry/src/*/resvg-0.46.0/src/filter/`）を移植する。各 file の先頭に、出典と license を書く。
- **完了条件:**
  - `cargo check -p arvane-compose --all-targets`
  - `cargo xtask check --no-write --only state-fields --crate arvane-compose`

### U32 compose-gpu-core

- **節:** §4.2〜§4.4（compose の feature）、§8.1（GpuContext）、§8.7（GPU の実装）、§8.8.1〜§8.8.3、§8.9、§8.11、§8.13、§10.5 の device の段落、§13.1 の SM-10
- **範囲:**
  - `crates/arvane-compose/src/gpu.rs`
  - `crates/arvane-compose/src/gpu/**`（U23 が移した file を組み込み、中身を書き換える）
  - `crates/arvane-compose/src/gpu_unbuilt.rs`
  - `crates/arvane-compose/Cargo.toml`（feature `gpu`、`gpu-interop`、wgpu、pollster、raw-window-handle、bytemuck、kivact-gpu-share）
  - `crates/kivact-gpu-share/**`（`device::request_device`）
  - `crates/kivact/Cargo.toml`（`gpu`、`gpu-interop` を compose へ中継する。§4.4）
  - `dev/tools/check-features.sh`（compose の 2 行）
- **内容:**
  - GPU executor を作る。
    - target と chunk の pool
    - draw の pipeline（§8.7 の GPU の標本化と coverage）
    - blend と composite、blur、drop shadow、color matrix、mask と clip、backdrop
  - chunk の部分 redraw、swapchain への合成、外部 frame の `ExternalPass`、offscreen の読み戻し、`FrameStats` の計数を作る。target が 1024 を超えたら `RenderError::GpuLimit` を返す（§8.14）。`PlanError` を `ComposeError::InvalidComposition` に変える。
  - filter graph と user shader と export は U42 が作る。この unit では、Op に Graph・Shader・export が来たら、CPU と同じ fallback（入力をそのまま出す、fallback chunk を描く、読み戻しで pixels を返す）で実行する。
  - `GpuContext::export_drawable`/`drop_export` を、feature の組み合わせのすべて（`gpu` だけ、`gpu-interop`、`gpu_unbuilt.rs`）で同じ署名で作る。この unit では、どれも `ComposeError::External("built without the gpu-interop feature")` と何もしない処理にする（`gpu-interop` の版の中身は U42）。既存の外部 image の import の `shared_images.rs` と同じ形にする（§8.1）。
- **完了条件:**
  - `cargo check -p arvane-compose --all-targets --features gpu`
  - `cargo check -p arvane-compose --all-targets --features gpu-interop`
  - `cargo check -p arvane-compose --all-targets --no-default-features`
  - `cargo check -p kivact-gpu-share --features import,wgpu`
  - `cargo xtask check --no-write --only state-fields,os-branch --crate arvane-compose`

### U33 style

- **節:** §9.10、§10.2（`DrawableSource` と `Draw::drawable`）、§11.1〜§11.5、§11.3.5（wire の `ShaderClock`）、§12.9.1（style 側）、§12.13
- **範囲:**
  - `crates/kivact-style/**`（新しい file: `shader.rs`、`paint/{border_image,mask}.rs`、`draw/{drawable,border_image}.rs`、`effect/filter_graph.rs`）
  - `crates/kivact-style/Cargo.toml`（kivact-shader）
  - `dev/xtask/policy/workspace.toml`（kivact-style の `[module_rules]` の `shader` と `[structural_options]` の list と record_types。§11.1）
- **内容:** 次をすべて作る。
  - style の setter と resolve（`classify_properties!`、`DELTA`、wire への写し、param の検証）
  - Transform の 3D
  - 色（`Repr::Space`、各関数、`Color::css`、Oklab の `mix` と `Animatable`）
  - Paint の clip、origin、blend と、Mask、BorderImage、FontVariation
  - shader の builder、filter graph の builder（graph の id と handle の所属の検証。`FilterGraph` は Clone にしない。`Filter::graph` が `AuthoredGraph` にし、resolve が `kivact_types::FilterGraph<Asset>` にする。§11.2.1）、Draw の API の追加
  - `compile_style` での報告: transition の `Spring::problem`（§12.2）、`layout_transition` の `repeat`/`alternate`（§12.3）
  - facade の名前
- **完了条件:**
  - `cargo check -p kivact-style --lib`
  - `cargo xtask check --no-write --only state-fields,api-shape,canonical-path,structural-options --crate kivact-style`

  `--all-targets` は使わない。kivact-style の doctest は dev-dependency の `kivact` に依存するため、CP8 で確かめる。

---

## 5. wave 4

| 行 | unit | 名前 |
|---|---|---|
| 1 | U41 | presenter-engine-base |
| 2 | U42 | compose-gpu-effects |

merge の順: U41 → U42。

### U41 presenter-engine-base

- **節:** §3.1、§6.9（engine から arvane への受け渡し）、§8.12（pool の持ち主と喪失の通知）、§8.13、§10.6、§12.9.2、§12.9.3、§15.1（arvane の presenter）、§17.3（arvane と engine の行）
- **範囲:**
  - `crates/arvane/**`（`src/{window,presenter,presenter/*,rasterizer,presentation,availability,env,error,lib}.rs`、`Cargo.toml`、`tests/e2e/**`）
  - `crates/kivact-engine/**`（compile の基盤だけ）
- **内容（arvane）:**
  - `Window::present(&Composition, FramePlan)`、`Presented::Shown::{exports, pending_exports}`、`export_drawable`/`drop_export`/`take_export_notices` を作る（§10.6）。presenter に `ExportRegistry`（`presenter/exports.rs`）を置き、`GpuContext` を捨てる全箇所の前で `Lost` を積み、新しい context の準備の完了で `Available` を積む。`export_drawable` の失敗は、device が無いときは kind `NotReady`、feature の無い build では kind `Unavailable` にする（§8.12）。`Available` は presenter が Ready に入る一箇所で積む。`arvane::spi::lose_gpu_device` を作る（§10.6）。CPU の presenter も `CpuOutput::report` から同じ報告を返す。`RendererChange` に `ShaderFailed`、`LimitExceeded` を足す。
  - CPU の damage 提示を作る。Rasterizer は `CpuCompositor::render` を使う形にする。
  - dev-dependency を `arvane-compose = { features = ["gpu-interop"] }` にする。
  - `Presented::Shown` に `stats: FrameStats` を加える（§10.6）。
  - **`crates/arvane/tests/e2e` の扱い:** 利用者の機能の契約を確かめる試験は、削除しない。
    - 対象は、group opacity と個別の opacity の違い、blur の範囲と shadow の offset、backdrop が形の外や後の描画を巻き込まないこと、外部 frame の表示、mask などである。これらは、同じ観測を `Composition`（root layer と、効果を `LayerEffects` に持つ子 layer）で行う形に移す。
    - 削除してよいのは、削除した private な helper（replay の index、DestOut の emulation）の内部だけを確かめる試験に限る。
    - 移した試験と削除した試験の対応表を、commit message と PR 説明の草稿に書く。
- **内容（engine の基盤）:** kivact-engine 全体を、新しい arvane、types、layout、editing の API で compile できるようにする。
  - `Model::composition` は、この unit では「root layer に、今の paint list から作った Raster chunk を一つだけ置く」形にする。layer 化は U51 が作る。
  - 機械的な追従をする。
    - `PaintProps.transitions` への改名（state motion は下の「state motion」の項で §12.9.3 の最終の形にする）
    - `Easing` の `.clone()`
    - `TextShaper::outline` の実装（`arvane::glyph_outline` を呼ぶ）
    - `RenderCmd::Glyphs` の新しい field（fill None、stroke None）
    - `Purpose` の削除と `ExternalItem` の root への追加
  - offscreen と snapshot は `CpuCompositor` を使い、`PaintPass` を引数で渡す（SM-04）。
  - **幾何の基盤:** §7.1 の `local_matrix`、`to_parent`、`window_map`、`map_between`、`bounds`、`to_local` と、§7.2 の平坦化の畳み込みを、この unit で作る（既存の 2D の関数を置き換える）。hit test、a11y、region への適用は U52 が行う。
  - **state motion（§12.9.3 の全体）:** `PaintVec` は U11 で消えているので、engine の state motion を、この unit で §12.9.3 の最終の形に書き換える（`PropTrack`、`TrackPhase` の Captured・Running・Held・Settled、capture と retarget の規則、`apply_components` による現在の paint、`StateMotion::moves_layer`）。`SpringRun::start` を使う。U54 はこの部分を変えない。
  - `Host::last_stats` を作り、`Presented::Shown::stats` で上書きする。FrameLog が有効なら file にも書く。`ArvaneTestHost::frame_stats()` と `ArvaneTestHost::lose_gpu_device()` を作る（§10.6）。
  - `kivact-engine/tests/host_locale.rs` の compile を直す。
- **完了条件:**
  - `cargo check -p arvane --all-targets`
  - `cargo check -p kivact-engine --all-targets --all-features`
  - `cargo check -p kivact-engine --no-default-features`
  - `cargo xtask check --no-write --only state-fields,effect-writes --crate kivact-engine`

### U42 compose-gpu-effects

- **節:** §8.8.4（GPU）、§8.8.5、§8.10、§8.12（GPU）、§11.3.4
- **範囲:**
  - `crates/arvane-compose/src/gpu/{graph,shader,export}.rs`（新）
  - `crates/arvane-compose/src/gpu/wgsl/graph_*.wgsl`（新）
  - `crates/arvane-compose/Cargo.toml`（kivact-shader）
  - `crates/kivact-gpu-share/src/wgpu/pool.rs`（`try_acquire`）、`src/pool.rs`（`Slots`）、`src/import/pool.rs`（準備の合図）
- **内容:**
  - filter graph の各 primitive を WGSL で実装する。Turbulence の表は CPU で作る。
  - user shader の pipeline cache と bind group を作り、失敗は `FrameReport::shader_failures` で報告する。
  - drawable の export pool と frame の書き出しを作る（§8.12）。generation の envelope、`ExportRecord`、`try_acquire`、見送りの `pending_exports` を含める。pool の記録と喪失の通知は arvane の presenter（U41）が持つので、compose は通知を持たない。
  - `kivact-gpu-share` の wgpu の export の `Pool` に `try_acquire` を足す。共通の `Slots` の frame の値を 2 から振り、fresh な slot の待ちの値を `SHARED_PREPARED` にする。`Import::new` が準備の後に `signal(SHARED_PREPARED)` を行う（§8.12）。
  - U32 の fallback の経路と、`gpu-interop` の版の `export_drawable`/`drop_export` の stub を、これらで置き換える。
- **完了条件:**
  - `cargo check -p arvane-compose --all-targets --features gpu-interop`
  - `cargo check -p arvane-compose --all-targets --no-default-features`
  - `cargo check -p kivact-gpu-share --features wgpu`

---

## 6. wave 5

| 行 | unit | 名前 |
|---|---|---|
| 1 | U51 | engine-layering |
| 2 | U52 | engine-geometry-3d |
| 3 | U53 | engine-lower-vocab |
| 4 | U54 | engine-motion |
| 5 | U55 | core-base |

merge の順: U51 → U53 → U52 → U54 → U55。

**engine の file の持ち主:** wave 5 の engine の 4 unit は、次の file の持ち主を守る。持ち主でない file は、新しい関数の呼び出しを 1〜3 行足すことだけを許す。

| file | 持ち主 |
|---|---|
| `paint/compose.rs`、`paint/list/*`、`paint/{frame_scene,state,snapshot,text,text_input}.rs`、`lower/{effects,context,scene,scene_out,scene_cache}.rs`、`lower/scene/*` | U51 |
| `model/transform.rs`、`model/interact/geometry.rs`、`a11y/**`、`tree/**`、`model/presence/**`、`host/write/reader.rs` | U52 |
| `lower/{border,border_image,fill,glyphs,assets,shape}.rs`、`measure/editor.rs` | U53 |
| `model/layout_motion/**`、`model/playback.rs`、`model/frame/**`、`model/states.rs` | U54（`model/state_motion/**` は U41 が完成させたので、wave 5 では誰も変えない） |

### U51 engine-layering

- **節:** §6.1〜§6.9、§9.5、§13.1 の SM-03、SM-09、SM-12
- **内容:**
  - layer 化の規則と `SubtreeFacts` を作る。
  - `CompositionBuilder` を作る（chunk の分割、`Opened` stack、item の clip と color matrix）。
  - layer の空間と変換、cull、`ink_local`、Draw の recording の layer、外部 frame と shader の item、mask の lower を作る。
  - `ShaderSpec` から `ShaderUse` への変換は、この unit では images を asset の natural pixels で解決し、time を 0 とする。
  - `FilterOp::Graph` の `FilterGraph<Asset>` を `map_images` で `FilterGraph<Pixels>` にする（§11.2.1）。
  - layer の理由から `RasterGrid` を決める（§6.1 の raster の格子）。
- **完了条件:**
  - `cargo check -p kivact-engine --all-targets --all-features`
  - `cargo xtask check --no-write --only state-fields,effect-writes --crate kivact-engine`

### U52 engine-geometry-3d

- **節:** §7.1〜§7.6、§13.1 の SM-01、SM-02
- **内容:**
  - U41 が作った `to_parent` などの関数を使い、hit test（3D コンテキストの奥行きと backface、`MAX_CONTEXT_PLANES` を超えるコンテキストの DOM 順、z が等しい候補の順を §8.4 の同一平面の key にそろえること。§7.3、§8.4、§8.5、§8.14）、a11y の bounds（`Option` の bounds を含む。§7.4、§5.15）、region の写像（padding box の基準。§7.5）を作る。
  - `Place.transformed` の一本化と、presence の唯一の持ち主を作る。
- **完了条件:** `cargo check -p kivact-engine --all-targets --all-features`

### U53 engine-lower-vocab

- **節:** §9.1〜§9.4、§9.6、§9.7（engine の部分）、§9.9 の engine の部分（`register_font` を store にも渡す）、§13.1 の SM-05、SM-06
- **内容:** border の線種、border image、background の clip・origin・blend、background-clip: text、gradient の `expand_gradient`（補間、gamut mapping、hint の適応分割。§9.6）、glyph の stroke・fill・order を lower する。`ReportOnceSet` と `Sent` に直す。
- **完了条件:** `cargo check -p kivact-engine --all-targets --all-features`

### U54 engine-motion

- **節:** §9.8（engine の再生）、§11.3.5（ShaderClocks と time）、§12.3（layout motion は反復しない）、§12.11（engine）、§12.12
- **内容:**
  - layout motion の run は `Repeat::Times(1)`、`PlayDirection::Normal` として扱う（§12.3）。`SpringRun::start` を使う。
  - layout motion を window 空間にし、祖先の補正（V は最も近い動いている祖先の A_c。§12.12）と角の補正を入れる。
  - `SharedLayouts`（記録は presence の状態で一度だけ。§12.11）、アニメーション画像の `Playback`、`ShaderClocks` と frame の要求を作る。
  - `paint/compose.rs`（U51 の file）には、shader の time と image の frame を渡す呼び出しだけを足す。
- **完了条件:**
  - `cargo check -p kivact-engine --all-targets --all-features`
  - `cargo xtask check --no-write --only state-fields --crate kivact-engine`

### U55 core-base

- **節:** §4.4（kivact の feature）、§11（kivact-core の再公開）、§12.5（`restore_motion` の呼び出し）、§17.3
- **範囲:**
  - `crates/kivact-core/**`（compile の基盤と再公開）
  - `crates/kivact-test/**`
- **内容:**
  - kivact-core と kivact-test を、新しい style、engine、runtime の API で compile できるようにする。
  - `kivact::draw`、`kivact::style`、`kivact::motion` に新しい公開名を加える（drawable と scroll timeline の名前は U62 と U61 が足す）。
  - viewport の gpu_frame が External item になる部分、`env_changed` の `restore_motion`、`Inbox` の `sync.rs` への移動（§10.5）を行う。
  - kivact-test に `Screen::frame_stats() -> FrameStats` と `Screen::lose_gpu_device()` を足す（`ArvaneTestHost` の同名の関数を呼ぶ。§10.6）。
  - kivact-core の `ShareLink::provide` の `# Safety` の文言を、`SharedTextures` の新しい順序の規則への参照に直す（設計書 §8.12）。
  - kivact-test に `Screenshot::raster_difference` と `RasterDifference::within_raster_tolerance` を作り、`screen/gpu_tests.rs` の `assert_close` をそれを使う形に書き換える（設計書 §17.1）。意味情報の bounds の `Option` に追従する。
- **完了条件:**
  - `cargo check -p kivact-core -p kivact-test --all-targets --all-features`
  - `cargo check -p kivact-core --no-default-features`
  - `cargo xtask check --no-write --only state-fields,effect-writes,api-shape,canonical-path --crate kivact-core`

---

## 7. wave 6

| 行 | unit | 名前 |
|---|---|---|
| 1 | U61 | core-view-motion |
| 2 | U62 | drawables-and-export |
| 3 | U64 | interaction-ui-facade |

merge の順: U64 → U61 → U62。

drawable と export は互いの型（`Drawable::share`、drawable の登録）を使うので、一つの unit にする。

### U61 core-view-motion

- **節:** §12.8、§12.10、§12.11（core の builder）、§11.5（`playing`）、§5.15（`LayoutChanged::layout`）
- **範囲:**
  - `crates/kivact-core/src/view/scroll_timeline.rs`（新）
  - `crates/kivact-core/src/view/flow/branch.rs`
  - `crates/kivact-core/src/view/{node_modifiers,tree,node_ref,view}.rs`
  - `crates/kivact-core/src/view/elements/builders.rs` の `playing`
  - `crates/kivact-core/src/client/dispatch/delivery.rs`
  - engine の `contract/events.rs` と `model/interact/geometry.rs` の `LayoutChanged::layout` の 1 field
- **内容:** `scroll_progress`、`view_progress`、`PresenceMode` と `Dynamic`、`layout_id`、`playing` を作る。
- **完了条件:** `cargo check -p kivact-core --all-targets --all-features`

### U62 drawables-and-export

- **節:** §10.1〜§10.6
- **範囲:**
  - `crates/kivact-core/src/view/{drawable,drawable_share}.rs`（新）
  - `crates/kivact-core/src/view/elements/{builders.rs,viewport.rs}` の drawable の部分
  - `crates/kivact-core/src/view/mount/*`
  - `crates/kivact-core/src/gpu_interop/**`（`ShareError` の移動）
  - engine の `contract/{node,host,events}.rs`、`model/drawables.rs`（新）、`model/apply.rs`、`paint/list/walk.rs`、`paint/compose.rs`、`model/interact/geometry.rs`、`a11y/build/*`、`host/{pump,write/*}.rs`
- **内容:**
  - drawable の core の API を作る。
  - engine の配置を作る。両 slot から一つの関数で作り、clip と backface を持たせる。backface は compositor と同じ行列の選び方で判定する（§10.3）。
  - paint、hit test、a11y（bounds の None を含む）、layout を作る。
  - drawable の binding（`DrawableBinding`、WrongApp・Disposed・二重 mount の拒否と報告、root が一致するときだけの解除。§10.2）と、host 以外の node の `Draw::drawable` の報告を作る。
  - `DrawableShare` を作る（generation、`ExportLink`、`ExportRequest` と GPU の遷移表、ACK の API は作らない（準備は fence の合図で compositor が見る。§8.12）、`wait` が Stale を返さないこと、`wait` と `ShareUpdate`、生涯で単調な frame の number、host request と host event、毎フレームの export、GPU の textures と frame の受け渡し、mode の切替、resize、device が無い間の `GpuWaiting` と `GpuExportAvailable`、喪失の `DrawableExportLost`）。
  - engine の `ExportRetry`（host の ns の時計、提示ごとの確定、backoff と pump の deadline）と、`take_export_notices` の読み出しを作る（§10.5）。
- **完了条件:**
  - `cargo check -p kivact-core -p kivact-engine --all-targets --all-features`
  - `cargo check -p kivact-core --no-default-features`

### U64 interaction-ui-facade

- **節:** §16.2（skeleton の Stagger）、§17.3、§11.6
- **範囲:**
  - `crates/kivact-interaction/**`
  - `crates/kivact-ui/**`
  - `crates/kivact/src/{lib,prelude}.rs`
- **内容:**
  - interaction と ui を移行表のとおりに書き換える。
  - prelude に §11.6 の名前を加える。ただし `drawable` は U62 と並行なので、この unit では加えない。CP6 の前に、オーケストレータが prelude に `kivact::view::drawable` の 1 行を加える。
- **完了条件:**
  - `cargo check -p kivact-interaction --all-targets`
  - `cargo check -p kivact-ui --lib`
  - `cargo check -p kivact --lib`

---

## 8. wave 7: kit と example

| 行 | unit | 範囲 |
|---|---|---|
| 1 | U71 | `crates/kivact-ui-basic/**`、`crates/kivact-ui-pop/**`、`examples/kivact-ui-basic/**`、`examples/kivact-ui-pop/**` |
| 2 | U72 | `crates/kivact-ui-game/**`、`crates/kivact-ui-creative/**`、`examples/kivact-ui-game/**`、`examples/kivact-ui-creative/**` |
| 3 | U73 | `crates/kivact-ui-soft/**`、`examples/kivact-ui-soft/**` |
| 4 | U74 | `examples/kivact/**`、`templates/basic/**`、`dev/tests/gpu-interop-producer/**` |

**全 unit に共通の事項:**

- **節:** §17.3
- **内容:** 移行表のとおりに書き換える。見た目が変わる既存の利用（background の origin、`EaseIn` の曲線、Oklab の補間）は、移行表の注のとおり、元の見た目に合わせる。kit の公開 API は縮めない。
- **完了条件:** 範囲の package の `cargo check -p <pkg…> --all-targets`（`--all-features` と `--no-default-features`）。U71〜U73 は、kit の試験（`tests/consumer.rs`）の compile も含む。U74 は加えて `cargo check -p kivact-runtime -p kivact-i18n -p kivact-style -p kivact-ui --all-targets` を通す（§0.4 の例外で wave 6 まで見なかった試験の compile）

merge の順: U71 → U72 → U73 → U74。

---

## 9. wave 8: 既存の試験と文書の compile、lint

| 行 | unit | 範囲 |
|---|---|---|
| 1 | U81 | `dev/tests/kivact-public-api/**`（既存の試験の compile）、`dev/tests/docs/**` と既存の `docs/**` の Rust の例（compile） |
| 2 | U82 | lint: `cargo xtask check --no-write --only <rule> --crate <crate>` を、変更した全 crate で全 rule について通す（rule: state-fields、effect-writes、api-shape、canonical-path、name-collision、kit-literal-color、os-branch、structural-options）。直し方は設計書 付録 A の A-4、A-5 |

**完了条件:**

- U81:
  - `cargo check -p kivact-public-api-tests -p kivact-docs --all-targets --all-features`
  - `cargo check -p kivact-public-api-tests -p kivact-docs --all-targets --no-default-features`
- U82: 上記の command がすべて成功すること。

merge の順: U81 → U82。

---

## 10. checkpoint

各 wave の後、オーケストレータが実行する。

### CP1〜CP7（軽い検査）

各 CP で、次の command を、表の crate について実行する。

```text
cargo fmt --all -- --check
cargo check -p <A の crate…> --all-targets --all-features
cargo check -p <A の crate…> --all-targets --no-default-features
cargo check -p <L の crate…> --lib --all-features
cargo check -p <L の crate…> --lib --no-default-features
cargo clippy -p <その CP で A に加わった crate…> --all-targets --all-features -- -D warnings
cargo clippy -p <その CP で L に加わった crate…> --lib --all-features -- -D warnings
```

- **A:** `--all-targets` で check できる crate。
- **L:** dev-dependency に `kivact` を持ち、CP7 まで `--lib` で check する crate（§0.4 の例外）。

| CP | A に加わる crate | L に加わる crate |
|---|---|---|
| CP1 | kivact-types | — |
| CP2 | kivact-shader、arvane-render、arvane-shell、arvane-native、arvane-virtual、arvane-text、arvane-asset、kivact-layout、kivact-editing、arvane-compose | kivact-runtime、kivact-i18n |
| CP3 | kivact-gpu-share | kivact-style |
| CP4 | arvane、kivact-engine | — |
| CP5 | kivact-core、kivact-test | — |
| CP6 | kivact-interaction | kivact-ui、kivact |
| CP7 | 五つの kit、example、template、gpu-interop-producer、そして L の全 crate（kivact-runtime、kivact-i18n、kivact-style、kivact-ui、kivact） | —（L は空になる） |

CP では、それまでの CP の crate も毎回含める（合計の集合を check する）。

**WSL の検査:** CP2 と CP4 では、加えて WSL で Linux の code を check する。integration branch を、main の checkout（`C:\w\kashiwade\kivact`）に checkout した状態で行う。

```text
wsl -d Ubuntu-24.04 -- bash -lc 'cd /mnt/c/w/kashiwade/kivact && CARGO_TARGET_DIR=$HOME/kivact-target cargo check -p arvane-native -p arvane-compose -p kivact-gpu-share --all-targets --all-features'
```

### CP8（既存の試験一式）

```text
cargo fmt --all -- --check
cargo xtask check --no-write
cargo clippy --workspace --all-targets --all-features -- -D warnings
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p kivact-docs --all-features -- --test-threads=1
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p gpu-interop-producer --all-features -- --test-threads=1
dev/tools/check-features.sh
```

**失敗した試験の扱い:**

- 失敗した試験を原因の crate ごとにまとめ、修正の unit（`U8x-fix-<crate>`、最大 6 並列）に割り当てる。修正の unit は、指定された試験の binary だけを window lock の下で `--test-threads=1` で再実行して確かめる。
- 既存の試験の期待値を更新してよいのは、設計書の決定で意味が変わったもの（§17.3 の注、CSS の isolation、Oklab の補間、background の origin、`EaseIn` の曲線、色の filter の合成後の適用、共有 texture の frame の値が 2 から始まること（§8.12。GPU 共有の既存の試験の期待値）、spring の相対の静止判定（`rest` 一つ、速度は ω0 で割る）と解析解の step、alternate で偶数回の run の終わりの値、layout motion が反復しないこと）だけである。それ以外は実装を直す。
- 既存の試験を削除してよいのは、U41 の対応表に載った private な helper の試験だけである。

**再実行の範囲:** CP8 の修正の後は、失敗した試験の binary と、修正した crate に依存する crate の試験だけを再実行する。CP8 を最初からやり直さない。workspace 全体は CP9 で実行する。

CP8 を通したら push する。

---

## 11. wave 9: 新しい試験、文書、見本

| 行 | unit | 名前 | 範囲 |
|---|---|---|---|
| 1 | U91 | e2e-render | §17.2 の `transform_3d.rs`、`compositor.rs`、`effects_filters.rs`、`borders_backgrounds.rs` |
| 2 | U92 | e2e-text-color-image-shader | §17.2 の `text_effects.rs`、`color_spaces.rs`、`images_animated.rs`、`shaders.rs` |
| 3 | U93 | e2e-motion-drawable | §17.2 の `drawables.rs`、`drawable_export_gpu.rs`、`motion_easing.rs`、`timeline.rs`、`scroll_timeline.rs`、`presence_modes.rs`、`shared_layout.rs`、`motion.rs` への追加 |
| 4 | U94 | docs-render | §16.1 の `3d-and-depth.md`、`effects.md`、`shaders.md`、`drawables.md`（英日）。drawables.md が gpu-interop の例を含むなら、`dev/tests/docs/build.rs` の `FEATURE_GUIDES` に加える |
| 5 | U95 | docs-motion | §16.1 の残りのページ（英日）、how-to-guide の README の索引（英日。U94 のページも含める）、§16.2 の game kit の文書（英日） |
| 6 | U96 | catalog-devdocs | §16.2 の game catalog の Effects ページと catalog の試験、§16.3、§16.4、THIRD-PARTY-NOTICES.md（§4.5） |

### 試験の unit（U91〜U93）

- **file:**
  - `dev/tests/kivact-public-api/tests/e2e/<topic>.rs`（新）と、`main.rs` への登録。`main.rs` の conflict はオーケストレータが解く。
  - fixture は `tests/fixtures/` に置くか、試験の中で生成する。GIF、APNG、WebP は試験の中で生成する。
- **dev-dependency:** 生成に使う `image = { version = "0.25.10", default-features = false, features = ["png", "gif", "webp"] }` を、`dev/tests/kivact-public-api/Cargo.toml` の dev-dependencies に加える（U92 が加える）。
- **完了条件:** topic ごとに次を実行する。GPU lane の試験は、この unit では「skip されて成功」でよい（CP9 で GPU 付きで実行する）。

  ```text
  KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh -- cargo test -p kivact-public-api-tests --test e2e --all-features -- <topic>:: --test-threads=1
  ```

- **試験が通らないとき:** 試験が設計書どおりなら、実装の crate を直してよい（設計書の範囲を出ない最小の修正）。修正は `fix(<crate>):` の commit に分ける。

### 文書の unit（U94、U95）

- **規則:** 英語と日本語を同じ commit で書く。既存のガイドの書式に合わせる。
- **完了条件:**

  ```text
  KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh -- cargo test -p kivact-docs --all-features --doc -- for_kivact_user --test-threads=1
  ```

  U95 は加えて、`for_kivact_ui_user::en::game` と `for_kivact_ui_user::jp::game` も同じ形で実行する。

### U96

- **完了条件:**
  - `KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh -- cargo test -p kivact-ui-game-catalog --test catalog -- --test-threads=1`
  - `cargo xtask check --no-write --only state-fields,api-shape,name-collision,kit-literal-color --crate kivact-ui-game-catalog`

merge の順: U91 → U92 → U93 → U94 → U95 → U96。

### CP9（最終）

CONTRIBUTING.md の merge の前の検査を、すべて実行する。

```text
cargo fmt --all -- --check
cargo xtask check
git add architecture && git commit -m "chore: regenerate architecture" (変更があるときだけ)
git diff --quiet -- architecture
cargo clippy --workspace --all-targets --all-features -- -D warnings
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p gpu-interop-producer --all-features -- --test-threads=1
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p kivact-docs --all-features -- --test-threads=1
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --no-default-features
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p gpu-interop-producer --no-default-features -- --test-threads=1
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p kivact-docs --no-default-features -- --test-threads=1
dev/tools/check-features.sh
```

**GPU lane（GPU のある Windows で）:**

```text
KIVACT_TEST_GPU=1 KIVACT_TEST_MIRROR=required dev/tools/window-lock.sh --timeout 5400 -- cargo test -p kivact-public-api-tests --test e2e --all-features -- shaders:: drawable_export_gpu:: transform_3d:: compositor:: effects_filters:: --test-threads=1
```

**Linux（WSL Ubuntu-24.04）:** integration branch を main の checkout に checkout した状態で、Wayland と X11 を分けて実行し、結果を分けて記録する。

```text
# Wayland (WSLg)
wsl -d Ubuntu-24.04 -- bash -lc 'cd /mnt/c/w/kashiwade/kivact && export CARGO_TARGET_DIR=$HOME/kivact-target KIVACT_TEST_MIRROR=off && dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features'
# X11 (Xwayland of WSLg): the same with WAYLAND_DISPLAY unset
wsl -d Ubuntu-24.04 -- bash -lc 'cd /mnt/c/w/kashiwade/kivact && export CARGO_TARGET_DIR=$HOME/kivact-target KIVACT_TEST_MIRROR=off && env -u WAYLAND_DISPLAY dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features'
```

**失敗したとき:** 修正の unit を作って直し、CP9 をやり直す。

**全部通ったら:**

1. integration branch を push する。
2. `PR_DRAFT.md` を書く。内容は次のとおり。
   - 概要、設計書の決定 D-01〜D-10
   - 設計書が設けた制限（元の要求より狭い点。2026-11-07 にユーザーが承認済み）として記録する: CONTRACT-COLOR（engine の state transition は sRGB に写した端点から補間する）、§3.3 の三段目（Vello の raster の CPU と GPU の差は縮めない）、CONTRACT-3D-CROWD（参加面が 256 を超える 3D コンテキストは平坦な重なり）、CONTRACT-SPRING-DURATION（振幅 500 を超えると静止が少し遅れる）。加えて、2026-11-07 の第 3 回のレビューで加えた制限（同日にユーザーが承認済み）: CONTRACT-GRADIENT-PRECISION（最終の画素は Vello の LUT の量子化を含む）、CONTRACT-LAYOUT-REPEAT（layout motion は反復しない）、CONTRACT-RASTER-GRID（動いている layer は小数 px で bilinear）
   - wave と unit の一覧
   - 付録 A の裁定の記録
   - CP9 の結果（Windows、GPU lane、Wayland、X11）
   - 手動確認が要る項目（`manual-verification.md` に足した項目）
3. ユーザーに報告して止まる。
4. `VISUAL_PARITY_DESIGN.md` とこの文書は、ユーザーの指示があれば最後の commit で削除する（指示が無ければ残す）。

---

## 12. 割り当ての早見表

| wave | 行 1 | 行 2 | 行 3 | 行 4 | 行 5 | 行 6 |
|---|---|---|---|---|---|---|
| 1 | U11 | | | | | |
| 2 | U21 | U22 | U23 | U24 | U25 | U26 |
| 3 | U31 | U32 | U33 | | | |
| 4 | U41 | U42 | | | | |
| 5 | U51 | U52 | U53 | U54 | U55 | |
| 6 | U61 | U62 | U64 | | | |
| 7 | U71 | U72 | U73 | U74 | | |
| 8 | U81 | U82 | | | | |
| 9 | U91 | U92 | U93 | U94 | U95 | U96 |

各 wave は、前の wave の checkpoint を通してから始める。wave の中の unit は同時に始める。
