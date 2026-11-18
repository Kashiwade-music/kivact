# 付属書 A：Arvane（arvane・arvane-shell・arvane-native・arvane-virtual・arvane-text・arvane-render・arvane-compose・arvane-asset・kivact-gpu-share）

根拠：`rework/audit/arvane.md`（AR-01〜AR-26。file:line は 2b8df5ac）。契約：`REWORK_DESIGN.md` 4.1（arvane は driver と presenter だけ。再公開なし）、5.6（`GpuStart::Never`、`prefer_gpu` の削除、`ARVANE_RENDERER`）、5.8（`Pixels` は premultiplied）、6.2（PERF-7、PERF-8、PERF-10）、7.7、DEC-08、DEC-09。

## 1. 範囲と前提

### 1.1 所見と担当 unit

| 所見 | 判定 | unit |
|---|---|---|
| AR-01 frame の scope | 採用 | A01（CPU）、A03（GPU） |
| AR-02 GPU の device は Driver に一つ | 採用 | A11（compose）、A13（driver） |
| AR-03 content_version、hash の速度、inline の一括選択 | 採用 | A09（版。W5b の鎖）、A16（hash と inline） |
| AR-04 damage の矩形ごとの validate と submit | 採用（GPU は 4.5 節の制約つき） | A01（CPU）、A03（GPU） |
| AR-05 embed の注入入力の欠落 | 採用 | A12 |
| AR-06 facade の削除 | 採用 | A06（W5b の鎖） |
| AR-07 `ShellWindow` の縮小 | 採用 | A07（W5b の鎖） |
| AR-08 描けるかの判定、embed の表示 | 変更して採用（`ShellEvent::Visibility(bool)`。モバイルの event は残す） | A08（W5b の鎖） |
| AR-09 CPU の frame の三重保持 | 採用 | A08（engine の部分）、A15 |
| AR-10 画像の三重保持、`Pixels` は premultiplied | 採用 | A10（W5b の鎖） |
| AR-11 SVG の二つ目の font DB・shaper・rasterizer | 変更して採用（DEC-09 版 2、12 章 EA-15：resvg・tiny-skia の描画を保ち、usvg の font を arvane-text から渡して fontdb の system の走査だけを消す） | A04 |
| AR-12 起動の仕事を UI thread から外す | 採用 | A02（文字）、A04（asset の worker） |
| AR-13 pipeline の遅延 compile | 採用 | A03 |
| AR-14 export の変化の検出と pool の写し | 採用 | A10（CPU の記録の署名）、A13（registry） |
| AR-15 embed の子の寿命の所有者 | 採用（12 章 EA-04〜EA-06 の形） | A14 |
| AR-16 設定の運び手と handler | 採用 | A05（W5b の鎖） |
| AR-17 font の知識 | 採用（要設計判断 J-1 の推奨どおりに範囲を限る） | A02（family の解決）、A01（outline と ink の重複の統合） |
| AR-18 kivact-gpu-share の feature と module | 採用 | A16 |
| AR-19 release の面の試験用の口 | 採用 | A01（`dilation`）、A06（`test` feature）、A07（`frame_sink`）、A12（debug DPI） |
| AR-20 小さな dead・過剰 | 採用 | A03（compose の 3 件）、A05（`limits_noted`）、A06（`RefCell`）、A12（native の 4 件） |
| AR-21 code と食い違う doc | 採用 | A07（`ForeignChild`）、A12（native の lib doc）、architecture.md の文は付属書 G（4.9 節） |
| AR-22 foreign window の presenter | 採用 | A08 |
| AR-23 `State::Off` の三つの意味 | 採用 | A13 |
| AR-24 font 検証の cache | 採用 | A01 |
| AR-25 a11y の木の二度目の検証 | 採用 | A12 |
| AR-26 process の雑音 | 採用 | 付属書 T（W1 の state 規則と `architecture/` の除外）。本付属書に unit は無い |
| 6.2 DEC-13 の lazy 型 | 設計書 | A12（arvane-native・arvane-virtual の `OnceLock` 7 箇所）、A03（compose の `DeviceLost`、pipeline） |

### 1.2 前提（他の付属書が先に終えていること）

- W1（付属書 T）：xtask の `state` 規則（6.1 節）。`xtask-allow(state-fields)` の印は消え、対象の struct には T が `// xtask-allow(state): ...` を付けた。e2e は `e2e_core`・`e2e_ui`・`e2e_kits`・`e2e_native` に分かれ、ファイルは `dev/tests/kivact-public-api/tests/<binary>/<topic>.rs` にある。本付属書は試験を **topic（ファイル名）と関数名** で指す。binary は `ls dev/tests/kivact-public-api/tests/*/<topic>.rs` で確かめる。
- W1a（付属書 S の S01）：`kivact_types::cell::{Deferred, SharedDeferred}`。本付属書は `SharedDeferred::get_or_init` が std の `OnceLock::get_or_init` と同じく「他の thread が初期化している間は待つ」ことを前提にする（S01 は `OnceLock` で実装する）。
- W2（付属書 S の S02〜S06）：`arvane-protocol` ができ、`kivact_types::render::*` の大半は `arvane_protocol::*`（crate root に平らに再公開）へ移った。`FontSource`・`GlyphRun` などは `kivact_types::shaped::*`。`Composite`・`ExportFrame`・`ExportedFrame`・`FrameStats` は `kivact_types` に残る。本付属書のコードはこの W2 後の path で書く。S06 の改名（ST-21、ST-11）が型名を変えた場合は、S06 の改名表の新しい名前を使う。
- W5b の鎖：本付属書の鎖の unit（A05〜A10）は W5b に置き、W5a（付属書 R・C の鎖の unit（C18 まで）、付属書 E の鎖の unit（E15・E17・E18・E20）と E の lane（E12・E13・E16））の **後** に続く。C11（launch・`kivact-engine/src/embed.rs`）と C13（embed：`arvane/src/embed.rs`、`arvane-shell/src/child.rs`、`arvane-native/src/{port/embed/*,port/win32/*,port/x11/*}`、`kivact-types/src/embed.rs`、`embed::Visibility` → `EmbedVisibility`）が先に終わっている。本付属書の手順は名前で場所を指すので、C の変更の後でも当てはまる。C13 で `kivact_types::embed::Visibility` は `EmbedVisibility` になっている。

### 1.3 後の付属書に渡すもの

- `arvane_protocol::CompositeLayer::content_version`（A09）。版を安定させる engine の実装は付属書 E（4.2 節の契約）。
- `arvane::DriverOptions`、`arvane::run`、`arvane::start_stepped`、`arvane::embed_driver`（A05）。
- arvane の facade が無くなった後の engine の import の先（A06。4.3 節の表）。
- `arvane_shell::{Drawable, ImeRequest, WindowSurface, SurfacePresence}`、`ShellEvent::Visibility(bool)`、`Presented::NotDrawable`（A07・A08）。
- `kivact_types::Pixels` は premultiplied（A10）。
- 付属書 G へ：architecture.md の crate 表と arvane-render の文（4.9 節）。

### 1.4 wave と lane の置き方

- W3a・W4・W6b・W7a・W7b・W8（SCHEDULE.md の sub-wave）：lane A1（描画：render・compose・gpu-share）と lane A2（文字・asset・shell・native）。同じ wave の A1 と A2 は同じファイルを持たない（5 章で確かめた）。
- W5b：本付属書の鎖（A05〜A10）だけの wave。W5a（R・C の鎖と E の lane）が終わった後に置く（INC-27）。鎖の unit が arvane の全 crate と engine・core・types・test の該当ファイルを持つため、並行の lane は無い。
- W6b：A11・A12 は W6b（S の鎖の後半 S15〜S23・C19 と並行）。W6a の S13・S14 が engine・arvane の file を持つので、A の lane は W6a に置かない（INC-28）。W6b の S の鎖の unit は A11・A12 の所有ファイルを持たない（SCHEDULE.md の W6b の表）ので、W7 へ送る条件は起きない。A11・A12 は arvane-text と arvane-protocol に触らない。
- W7a：A13（A1）と A14（A2）。W7b：A15（A1）。W8：A16（A2）。A16 の前提に E14（W6b。`content_version` の stamp）を含める（INC-25）。
- 鎖の unit（A05〜A10）が engine・core・test のファイルを変えるのは、呼出し元を直すためだけである。

## 2. unit の一覧

| ID | wave | lane | 前提の unit | 所有するファイル・directory（要約。正確な一覧は各 unit の「所有」） | 所見 | 規模 |
|---|---|---|---|---|---|---|
| A01 | W3a | A1 | S02〜S06 | `arvane-render/src/{cpu.rs,draw.rs,draw/painter.rs,derived.rs,validate.rs,validate/fonts.rs,validate/scene.rs,text.rs,outline.rs,ink.rs,glyph.rs(新),lib.rs,gpu/raster.rs,gpu/encode.rs}`、`arvane-compose/src/cpu/compositor.rs` | AR-01（CPU）、AR-04（CPU）、AR-24、AR-19（`dilation`）、AR-17（outline と ink の重複） | M |
| A02 | W3a | A2 | S02〜S06 | `arvane-text/`、`arvane-asset/src/{font.rs,lib.rs}`、`arvane/src/{engine.rs,asset.rs}` | AR-12（文字）、AR-17（family の解決） | L |
| A03 | W4 | A1 | A01 | `arvane-render/src/{gpu.rs,gpu/raster.rs,gpu/encode.rs,lib.rs}`、`arvane-compose/src/`（`plan/` を除く）、`arvane-compose/Cargo.toml` | AR-01（GPU）、AR-04（GPU）、AR-13、AR-20（`ComposeError`、`DeviceLost`、`Retain`） | L |
| A04 | W4 | A2 | A02 | `arvane-asset/`、`Cargo.lock`、`dev/xtask/policy/workspace.toml`（`visible_to` の `arvane-text` の 1 行） | AR-11（DEC-09 版 2）、AR-12（asset の worker） | M |
| A05 | W5b | 鎖 | W5a の鎖と E の lane | `kivact-types/src/{gpu.rs,embed.rs}`、`arvane/src/{env.rs,window.rs,embed.rs,presentation.rs,presenter/frame_policy.rs,lib.rs}`、`arvane/tests/e2e/{support.rs,embed.rs,presenter.rs}`、`kivact-engine/src/{host/launch.rs,embed.rs,host/testing/test_host.rs}`、`kivact-core/src/embed/open.rs`、`kivact-test/src/options.rs`、`dev/xtask/policy/workspace.toml`（`arvane::ShellSetup` の 1 行） | AR-16、5.6、AR-20（`limits_noted`） | M |
| A06 | W5b | 鎖 | A05 | `arvane/`（facade の削除）、`arvane-text/src/{text_engine.rs,inline.rs,error.rs,lib.rs}`、`arvane-render/src/{error.rs,lib.rs}`、`kivact-engine/`（`use` と型 path の掃引、`Cargo.toml`）、`kivact-test/src/`（`kivact_engine::testing` 経由の名前だけ）、`Cargo.toml`（root の profile 1 行）、`templates/basic/README.md` の profile 1 行、`dev/xtask/policy/workspace.toml`、`dev/tools/check-features.sh` | AR-06、AR-19（`test` feature）、AR-20（`RefCell`） | L（機械的） |
| A07 | W5b | 鎖 | A06 | `arvane-shell/src/{window.rs,ime.rs,lib.rs}`、`arvane-native/src/port/{winit,win32,x11}/window.rs`、`arvane-native/src/port/winit/open.rs`、`arvane-native/tests/smoke.rs`、`arvane-virtual/src/window.rs`、`arvane/src/{window.rs,presenter/*}`、`kivact-types/src/platform.rs`、`kivact-engine/src/{host/pump.rs,host/window.rs,host/foreign.rs,host/write/writer.rs,contract/host.rs}`、`kivact-core/src/{platform/services.rs,view/foreign_window.rs}`、`arvane/tests/e2e/environment.rs` | AR-07、AR-19（`frame_sink`）、AR-21（`ForeignChild`） | L |
| A08 | W5b | 鎖 | A07 | `arvane-shell/src/{events.rs,window.rs,surface.rs(新),lib.rs}`、`arvane-native/src/port/embed/surface.rs`（削除）、`arvane-native/src/port/{win32,x11,winit}/*` の surface と visibility の箇所、`arvane-virtual/src/{surface.rs(削除),window.rs,backend.rs}`、`arvane/src/{window.rs,presenter/frame.rs,presenter/frame_policy.rs,lib.rs}`、`kivact-engine/src/host/{window.rs,events.rs,pump.rs,window_changes.rs}`、`arvane/tests/e2e/present.rs` | AR-08、AR-09（engine の解放）、AR-22 | L |
| A09 | W5b | 鎖 | A08 | `arvane-protocol/src/composition.rs`、`arvane-compose/src/{cache.rs,plan/damage.rs,plan/items.rs,plan/layers.rs}`、`kivact-engine/src/{lower/scene_out.rs,host/devtools.rs,lower/content_versions.rs(新),lower.rs}`、`arvane/tests/e2e/{support.rs,images.rs}` と `CompositeLayer {` を書く全試験 | AR-03（版） | M |
| A10 | W5b | 鎖 | A09 | 4.7 節の表のファイル（types・asset・render・compose・gpu-share・engine・core・svg・ui-basic・test・dev/tests・examples の該当箇所） | AR-10、AR-14（CPU の記録） | L |
| A11 | W6b | A1 | A03、A10 | `arvane-compose/src/{gpu.rs,gpu/*,gpu_unbuilt.rs,lib.rs}`、`arvane/src/{presenter/gpu.rs,presenter/frame_policy.rs,test_hooks.rs}`（手順 5：新しい compose の API への追従だけ） | AR-02（compose） | L |
| A12 | W6b | A2 | A08 | `arvane-native/src/{wake.rs(削除),lib.rs,mirror.rs,module.rs,native_input/windows.rs,port/win32/{dpi.rs,native.rs,window.rs,create.rs},port/x11/{poll.rs,thread.rs,notifier.rs,dnd.rs,window.rs},port/winit/{runtime.rs,window.rs,foreign/win32.rs},port/embed/session.rs}`、`arvane-native/Cargo.toml`、`arvane-shell/src/{types.rs,window.rs,a11y/shared.rs}`、`arvane-virtual/src/window.rs` | AR-05、AR-19（DPI）、AR-20（native 4 件）、AR-21（lib doc）、AR-25、DEC-13 | M |
| A13 | W7a | A1 | A11 | `arvane/src/{window.rs,presenter.rs,presenter/*,gpu_driver.rs(新),test_hooks.rs,lib.rs}`、`arvane/tests/e2e/presenter.rs`、`arvane-compose/src/gpu/{export.rs,device.rs,state.rs}`・`arvane-compose/src/{gpu.rs,lib.rs,gpu_unbuilt.rs}`（2 関数の追加だけ） | AR-02（driver）、AR-14（registry）、AR-23 | L |
| A14 | W7a | A2 | A12 | `arvane-native/src/port/embed/{child.rs,session.rs}`、`arvane-native/src/port/win32/{state.rs,window.rs,proc.rs,create.rs}`、`arvane-native/src/port/x11/{window.rs,thread.rs}` | AR-15（EA-04〜EA-06） | L |
| A15 | W7b | A1 | A13 | `arvane-compose/src/cpu/{compositor.rs,filters.rs,draw.rs}`、`arvane/src/presenter/frame_policy.rs` | AR-09（compose と presenter）、EA-07、EA-17 | M |
| A16 | W8 | A2 | A09、A11、E14 | `arvane-compose/src/plan/{hash.rs,builder.rs,layers.rs}`、`arvane-compose/Cargo.toml`、`arvane-compose/src/gpu/device.rs`（import 1 行）、`kivact-gpu-share/`、`dev/tests/gpu-interop-producer/src/{vulkan.rs,d3d12.rs}`（path だけ） | AR-03（hash、inline）、AR-18 | M |

A15（W7b）と A16（W8）は別の wave で、ファイルも重ならない（A15 は `cpu/*` と presenter、A16 は `plan/*`・`gpu/device.rs`・`Cargo.toml`）。

### 2.1 各 unit の共通の注意

- コメントと rustdoc は英語、本付属書の説明は日本語。新しい module には `//! Role: <英語 1 文>` を置く。
- 関数は 80 行、ファイルは 1000 行まで。新しいコードで超える場合は、手順が指定する分割に従う。
- `unwrap()` は使わない（`expect("invariant: ...")`）。
- 窓を開く試験は `dev/tools/window-lock.sh --` を前に付けて実行する（CONTRIBUTING）。下の確認のコマンドはそう書いてある。
- 「試験が通る」は `KIVACT_TEST_MIRROR=off` で通り、各 command（filter）が 1 件以上の試験を実行すること（0 件は失敗。PLAN-07）。commands は作業 tree の root（`git rev-parse --show-toplevel`）で実行する（PLAN-04）。GPU の試験は GPU を使えない機械では理由を表示して skip する（失敗ではない）。

### 2.2 E2E の試験の束（確認で名前を引く）

1 行が 1 つの command（1 つの binary）。module の binary は付属書 T の T15 の表のとおり（`e2e_core`・`e2e_native`）。filter が 1 件の試験も実行しない command は失敗として扱う（出力の `running 0 tests` は失敗。各 unit の完了条件）。

| 束 | command |
|---|---|
| `ARV` | `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p arvane --test e2e`（A06 の後は `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p arvane --features test --test e2e`） |
| `NATIVE-SMOKE` | `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p arvane-native --features test --test smoke` |
| `GPU-DEMAND` | `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e_native -- gpu_on_demand` |
| `GPU-SHARE` | `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e_native -- drawable_export_gpu gpu_interop` と `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e_core -- shaders` |
| `EMBED` | `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e_native -- embed` |
| `FOREIGN` | `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e_native -- foreign_window` と `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test foreign_window_run` |
| `ASSETS` | `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e_core -- assets images_animated svg_embedded_text offscreen_svg image_semantics` |
| `OFFSCREEN` | `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test offscreen` と `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e_core -- offscreen` |
| `TEXT` | `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e_core -- text_primitives text_style text_editing launch_fonts input_platform` |
| `DRAW` | `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e_core -- canvas drawables viewport compositor effects_filters borders_backgrounds paint style_paint` |
| `LIFE` | `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e_core -- lifecycle environment windows close_request resource_reclaim` |
| `DEVTOOLS` | `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e_native -- devtools` |
| `A11Y` | `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e_core -- a11y_incremental` |

GPU の機械で GPU の試験を実際に走らせるときは、同じ command の `env` の後に `KIVACT_TEST_GPU=1` を足し、`KIVACT_TEST_MIRROR=off` を `KIVACT_TEST_MIRROR=required` にし、末尾に `--test-threads=1` を足す（window-lock は 1 つのまま）。

## 3. unit の手順

## A01：CPU の frame の scope、damage の矩形の一括、font 検証の cache

- **wave / lane / 前提**：W3a / A1 / S02〜S06
- **所有**：`crates/arvane-render/src/{cpu.rs,draw.rs,draw/painter.rs,derived.rs,validate.rs,validate/fonts.rs,validate/scene.rs,text.rs,outline.rs(削除),ink.rs(削除),glyph.rs(新),lib.rs,gpu/raster.rs,gpu/encode.rs}`、`crates/arvane-compose/src/cpu/compositor.rs`
- **目的**：AR-01（CPU）：一つの window の frame のすべての chunk を一つの cache の frame の中で描く。AR-04（CPU）：damage の矩形が N 個でも validate と encode は 1 回。AR-24：font 検証の cache を HashMap にし、死んだ項目の除去は frame ごとに 1 回。AR-19：`dilation` の定数を引数で渡さない。AR-17 の一部：`outline.rs` と `ink.rs` の重複した座標の計算を一つの module にする。
- **手順**：
  1. `glyph.rs` を作る。`crates/arvane-render/src/outline.rs` の全文と `crates/arvane-render/src/ink.rs` の全文を一つのファイル `crates/arvane-render/src/glyph.rs` に移し、二つの元ファイルを `git rm` する。先頭は `//! Role: Glyph outlines and ink bounds read from a shaped run's font.` の 1 行にする（元の 2 つの `//! Role:` 行は消す）。両方の関数にある「run の variation の値から `Vec<NormalizedCoord>` を作る」数行（`NormalizedCoord::from_f32(*c as f32)` を含む行）を、次の関数 1 つにまとめ、`glyph_outline` と `glyph_ink_bounds` から呼ぶ：
     ```rust
     /// The run's variation coordinates, normalized as skrifa takes them.
     fn coords(run: &GlyphRun) -> Vec<NormalizedCoord> {
         // the body is the coordinate code both functions had, unchanged
     }
     ```
     `use` 行は二つのファイルのものを合わせ、重複を消す。`lib.rs` の `mod ink;` と `mod outline;` を `mod glyph;` 1 行にし、`pub use ink::glyph_ink_bounds;` と `pub use outline::glyph_outline;` を `pub use glyph::{glyph_ink_bounds, glyph_outline};` にする。`draw/painter.rs` の `use crate::outline::glyph_outline;` を `use crate::glyph::glyph_outline;` にする。
  2. `text.rs`：`fn dilation(size: f64, bucket: f64, d_full: f64, l_sat: f64, s_full: f64, s_none: f64) -> f64` を次に替える：
     ```rust
     /// `d(size, L) = D_FULL * min(1, L_q / L_SAT) * F(size)`: `bucket` is
     /// `L_q`, and `F` is 1 up to `S_FULL`, linear to 0 at `S_NONE`, and 0 above.
     fn dilation(size: f64, bucket: f64) -> f64 {
         let per_luminance = D_FULL * (bucket / L_SAT).min(1.0);
         let fade = ((S_NONE - size) / (S_NONE - S_FULL)).clamp(0.0, 1.0);
         per_luminance * fade
     }
     ```
     `darkening` の `let amount = dilation(size, bucket, D_FULL, L_SAT, S_FULL, S_NONE);` を `let amount = dilation(size, bucket);` にする。同じファイルの `#[cfg(test)]` の helper `fn model(size: f64, bucket: f64) -> f64` を消し、試験の中の `model(` をすべて `dilation(` にする（`perl -pi -e 's/\bmodel\(/dilation(/g' crates/arvane-render/src/text.rs` の後、helper の定義が消えたこと、`fn dilation(` の定義行が手順の形であることを目で確かめる）。期待値は変えない。
  3. `validate/fonts.rs` を次の全文に置き換える：
     ```rust
     //! Role: Glyph counts of the fonts scenes draw with, read once per font face.

     use crate::error::RenderError;
     use kivact_types::shaped::FontSource;
     use skrifa::raw::TableProvider;
     use std::collections::HashMap;
     use std::sync::{Arc, Weak};

     /// The most font faces remembered; past it the table starts again.
     const MAX_FACES: usize = 256;

     /// The glyph count of each font face drawn lately, by the address and
     /// length of its bytes and its index in the file.
     #[derive(Default)]
     pub(crate) struct Fonts {
         entries: HashMap<(usize, usize, u32), Entry>,
     }

     struct Entry {
         /// A face whose bytes were freed is pruned before its address can be
         /// reused by other bytes.
         bytes: Weak<dyn AsRef<[u8]> + Send + Sync>,
         glyphs: u32,
     }

     impl Fonts {
         /// Forget every face.
         pub(crate) fn clear(&mut self) {
             self.entries.clear();
         }

         /// Forget the faces whose bytes were freed (once per frame scope).
         pub(crate) fn prune(&mut self) {
             self.entries.retain(|_, entry| entry.bytes.strong_count() != 0);
         }

         /// The glyph count of face `index` of `bytes`.
         pub(super) fn glyphs(&mut self, bytes: &FontSource, index: u32) -> Result<u32, RenderError> {
             let key = (bytes.id(), bytes.len(), index);
             if let Some(entry) = self.entries.get(&key)
                 && entry.bytes.strong_count() != 0
             {
                 return Ok(entry.glyphs);
             }
             let glyphs = read_glyph_count(bytes, index)?;
             if self.entries.len() >= MAX_FACES {
                 self.prune();
                 if self.entries.len() >= MAX_FACES {
                     self.entries.clear();
                 }
             }
             self.entries.insert(key, Entry { bytes: Arc::downgrade(bytes.source()), glyphs });
             Ok(glyphs)
         }
     }
     ```
     `fn read_glyph_count(bytes: &FontSource, index: u32) -> Result<u32, RenderError>` は、今の `glyphs` の「miss のとき」の部分（`skrifa::FontRef::from_index(bytes, index)` から `font.maxp()?.num_glyphs()` まで、失敗を今と同じ `RenderError` の variant に写す）をそのまま移した private 関数。`bytes.id()`・`bytes.len()`・`bytes.source()` は今のファイルが使っている `FontSource` の method で、今の呼び方（`entry.bytes.as_ptr().cast::<u8>() as usize == bytes.id()` と `entry.len == bytes.len()`、`Arc::downgrade(bytes.source())`）のとおり。`Entry` の `Weak` の型は今の `Entry::bytes` の型をそのまま写す。
  4. `draw/painter.rs`：`draw::Caches` から `derived` を外す。`pub(crate) struct Caches` の field `pub(crate) derived: Derived` を消す（`#[derive(Default)]` はそのまま）。`pub(crate) fn draw(` の署名を次にする：
     ```rust
     pub(crate) fn draw(
         canvas: &mut impl Canvas,
         caches: &mut Caches,
         derived: &mut DerivedScope<'_>,
         scene: &RenderScene,
         frame: &Frame,
     )
     ```
     本体の `let Caches { fonts, derived, coords, .. } = caches;` を `let Caches { fonts, coords, .. } = caches;` にし、`derived.frame(|derived| {` と対応する `});` を消して closure の中身を外に出す（`Painter { canvas, fonts, derived, coords, .. }` の `derived` は引数の `derived`）。`pub(super) struct Painter<'a, C: Canvas>` の field `pub(super) derived: DerivedScope<'a>` を `pub(super) derived: &'a mut DerivedScope<'d>` にし、struct を `Painter<'a, 'd, C: Canvas>` にする。`Painter` の impl 行は lifetime を一つ足す（`impl<C: Canvas> Painter<'_, '_, C>` など）。`self.derived.gradient_tile(..)`・`self.derived.resampled(..)` は変えない。
  5. `cpu.rs`：`CpuRaster` に field `derived: crate::derived::Derived` を足し、`Default` の中で `derived: crate::derived::Derived::default()` を入れる（`Derived` に `Default` が無ければ、今まで `draw::Caches::default()` が `derived` に入れていた式を使う）。公開の `pub fn raster(&mut self, ...)` を消して、次の型と method にする：
     ```rust
     /// One window frame of rasters. Images converted and tiles derived for
     /// any raster of the frame stay cached for the next frame; what no raster
     /// of the frame drew is dropped when the frame ends.
     pub struct CpuFrameRaster<'a> {
         context: &'a mut RenderContext,
         resources: &'a mut Resources,
         caches: &'a mut draw::Caches,
         images: images::ImageScope<'a, ImageSource>,
         derived: DerivedScope<'a>,
         scratch: &'a mut Vec<u8>,
     }

     impl CpuRaster {
         /// Run `body` as one frame: every raster it makes shares the frame's
         /// image and derived-tile caches (AR-01).
         pub fn frame<R>(&mut self, body: impl FnOnce(&mut CpuFrameRaster<'_>) -> R) -> R {
             let CpuRaster { context, resources, caches, derived, images, scratch } = self;
             caches.validation.prune();
             images.frame(|images| {
                 derived.frame(|derived| {
                     body(&mut CpuFrameRaster { context, resources, caches, images, derived, scratch })
                 })
             })
         }
     }

     impl CpuFrameRaster<'_> {
         /// Draw `scene` (logical px, `origin` at the frame's top left, `scale`
         /// device px per logical px) into `out`, a premultiplied RGBA8 frame of
         /// `size` device px; only `clip` when given.
         pub fn raster(
             &mut self,
             scene: &RenderScene,
             origin: Point,
             scale: f64,
             size: (u16, u16),
             clip: Option<PixelRect>,
             out: &mut [u8],
         ) -> Result<(), RenderError> {
             let full = PixelRect { x: 0, y: 0, width: u32::from(size.0), height: u32::from(size.1) };
             let region = clip.unwrap_or(full);
             self.render(scene, origin, scale, size, region, &[region], out)
         }

         /// Draw `scene` into the damaged `rects` of `out` with one validation
         /// and one encode (AR-04): the rects' bounding box is rendered once and
         /// only the rects are copied.
         pub fn raster_rects(
             &mut self,
             scene: &RenderScene,
             origin: Point,
             scale: f64,
             size: (u16, u16),
             rects: &[PixelRect],
             out: &mut [u8],
         ) -> Result<(), RenderError> {
             let Some(bounds) = bounding(rects) else {
                 return Ok(());
             };
             self.render(scene, origin, scale, size, bounds, rects, out)
         }
     }

     /// The smallest rect holding every rect of `rects`; `None` when empty.
     fn bounding(rects: &[PixelRect]) -> Option<PixelRect> {
         let mut iter = rects.iter();
         let first = *iter.next()?;
         let (mut x0, mut y0) = (first.x, first.y);
         let (mut x1, mut y1) = (first.x + first.width, first.y + first.height);
         for r in iter {
             x0 = x0.min(r.x);
             y0 = y0.min(r.y);
             x1 = x1.max(r.x + r.width);
             y1 = y1.max(r.y + r.height);
         }
         Some(PixelRect { x: x0, y: y0, width: x1 - x0, height: y1 - y0 })
     }
     ```
     `PixelRect` の field の型が `u32` でなければ（`rg -n "pub struct PixelRect" -A10 crates/arvane-protocol/src`）、上の `u32::from(..)` をその型への変換に合わせる。
     `fn render(&mut self, scene: &RenderScene, origin: Point, scale: f64, size: (u16, u16), region: PixelRect, copy: &[PixelRect], out: &mut [u8]) -> Result<(), RenderError>`（private、`impl CpuFrameRaster<'_>` の中）は、消した `CpuRaster::raster` の本体を次のとおり写したもの（引数が 7 個を超えて clippy の `too_many_arguments` が出る場合は、`origin`・`scale`・`size` を `struct Target { origin: Point, scale: f64, size: (u16, u16) }`（private）にまとめて渡す）：
     - (a) 今の scale の検査・`validate::target(size)?`・`out.len()` の検査、今の `clip` の範囲の検査を `region` に対して。
     - (b) `at` と `draw::Frame` の計算は今のまま、`region` で。
     - (c) `validate::scene(scene, &mut self.caches.validation)?;`
     - (d) `self.context.reset_and_resize(width, height);`（`region` の大きさ）
     - (e) 今の `self.images.frame(|images| { let mut canvas = CpuCanvas { .. }; draw::draw(&mut canvas, &mut self.caches, scene, &frame); });` を、`let mut canvas = CpuCanvas { context: self.context, resources: self.resources, images: &mut self.images }; draw::draw(&mut canvas, self.caches, &mut self.derived, scene, &frame);` にする（`self.context` などは `&mut *self.context` と再借用する）。
     - (f) `self.context.flush();`
     - (g) `region` が全体で `copy` がその 1 つだけなら、今のとおり `out` に直接 render して `return Ok(())`。
     - (h) それ以外は、今のとおり `self.scratch` に `region` の大きさで render し、`copy` の各 rect について、scratch の中の位置（rect の左上 − region の左上）から `out` の位置（rect の左上）へ行ごとに `copy_from_slice` する（今の「scratch から out へ行を写す」loop を rect ごとに回す）。
     `struct CpuCanvas<'a>` の field `images: images::ImageScope<'a, ImageSource>` を `images: &'a mut images::ImageScope<'b, ImageSource>` にし、struct を `CpuCanvas<'a, 'b>` にする（impl 行は `CpuCanvas<'_, '_>`）。`fn source` の `images::converted(&mut self.images, image, ..)` を `images::converted(self.images, image, ..)` にする。
     `pub fn release_caches` と `pub fn forget_scratch` は変えない。`lib.rs` の `pub use cpu::CpuRaster;` を `pub use cpu::{CpuFrameRaster, CpuRaster};` にする。
  6. `gpu/raster.rs` と `gpu/encode.rs`（GPU の振る舞いはこの unit では変えない。frame の scope は A03）：`GpuRaster` に field `pub(super) derived: crate::derived::Derived` を足し、`GpuRaster::new` で手順 5 と同じ式で作る。`encode.rs` の `fn record` の `let GpuRaster { scene, resources, renderer, images, caches, textures, atlas } = self;` に `derived` を足し、`draw::draw(&mut canvas, caches, source, frame);` を `derived.frame(|mut derived| draw::draw(&mut canvas, caches, &mut derived, source, frame));` にする。`gpu/raster.rs` の `pub fn raster` の `validate::scene(scene, &mut self.caches.validation)?;` の直前に `self.caches.validation.prune();` を足す。
  7. `crates/arvane-compose/src/cpu/compositor.rs`：`fn run(` の `for task in &plan.chunks { raster_chunk(raster, task, &mut stores.chunks, &mut counts)?; }` を次にする：
     ```rust
     raster.frame(|frame| -> Result<(), ComposeError> {
         for task in &plan.chunks {
             raster_chunk(frame, task, &mut stores.chunks, &mut counts)?;
         }
         Ok(())
     })?;
     ```
     `fn raster_chunk(raster: &mut CpuRaster, ...)` を `fn raster_chunk(raster: &mut CpuFrameRaster<'_>, ...)` にし、`Redraw::Rects(rects)` の枝の `for rect in rects { raster.raster(scene, task.origin, task.scale, size, Some(*rect), &mut px.data)?; counts.pixels += area(rect); }` を次の 2 行にする：
     ```rust
     raster.raster_rects(scene, task.origin, task.scale, size, rects, &mut px.data)?;
     counts.pixels += rects.iter().map(area).sum::<u64>();
     ```
     `use arvane_render::{CpuRaster, RenderError};` を `use arvane_render::{CpuFrameRaster, CpuRaster, RenderError};` にする。全体を描く `raster.raster(scene, task.origin, task.scale, size, None, &mut px.data)?;` は変えない。
- **消すもの**：`CpuRaster::raster`、`crates/arvane-render/src/outline.rs`、`crates/arvane-render/src/ink.rs`、`draw::Caches::derived`、`dilation` の 4 つの定数引数と試験の helper `model`。
- **状態の閉じ方**：`Fonts` は field が 1 つで対象外。`CpuFrameRaster` は参照と scope だけで `bool`・`Option`・counter を持たない。`CpuRaster`・`GpuRaster` は field を 1 つ（`Derived`）足すだけで対象の field の数は変わらない（T の印があれば変えない）。
- **試験**：足す試験は無い。`crates/arvane-render/src/frame_cache.rs` と `text.rs` の単体試験がそのまま通る。
- **確認（tier a）**：
  ```
  cargo fmt -p arvane-render -p arvane-compose
  cargo check -p arvane-render -p arvane-compose --all-targets --all-features
  cargo clippy -p arvane-render -p arvane-compose --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane-render
  cargo xtask check --no-write --crate arvane-compose
  cargo test -p arvane-render -p arvane-compose --all-features
  ```
  E2E：`ARV`、`DRAW`、`OFFSCREEN`。
- **完了条件**：`rg -n "fn raster\(" crates/arvane-render/src/cpu.rs` が `CpuFrameRaster` の impl の中の 1 件だけ。`rg -w model crates/arvane-render/src/text.rs` が 0 件。`crates/arvane-render/src/outline.rs` と `ink.rs` が無い。上のコマンドと E2E が通る。

## A02：system font の走査を UI thread から外し、family の解決を arvane-text に集める

- **wave / lane / 前提**：W3a / A2 / S01、S02〜S06
- **所有**：`crates/arvane-text/`（`Cargo.toml` を含む）、`crates/arvane-asset/src/{font.rs,lib.rs}`、`crates/arvane/src/{engine.rs,asset.rs}`
- **目的**：AR-12（文字、12 章 EA-08・EA-09）：`TextEngine::new` は system font を走査しない。走査の worker は process に一つで、一度だけ起動し、`DetachedThreads` に登録する（埋め込みの host が module を保つ契約）。engine は launch と embed の最初に `arvane_text::start_system_font_scan()` を呼ぶ（A06）ので、走査は window と GPU の準備と重なり、最初の文字の計測は残りだけを待つ（PERF-10）。AR-17（family の解決）：family 名と総称 family の解決を arvane-text が fontique の総称の表で行い、arvane-asset の OS ごとの手書きの表（`Os`、`GenericFamily::candidates`）と facade のカンマの特例を消す。SVG の `<text>` 用に、UI thread の外で使える font の検索 `FontLookup` を作る（A04 が使う）。
- **手順**：
  1. 新しいファイル `crates/arvane-text/src/system_fonts.rs`（EA-08・EA-09：worker は process に一つ、一度だけ起動し、`DetachedThreads` に登録する）：
     ```rust
     //! Role: The process's system font scan, made once on a worker thread of its own.

     use kivact_types::cell::SharedDeferred;
     use kivact_types::process::DetachedThreads;
     use parley::fontique::{Collection, CollectionOptions};
     use std::sync::atomic::{AtomicBool, Ordering};
     use std::time::Instant;

     /// The system fonts, scanned once per process. The scan runs on the
     /// worker `start` spawns; a caller that needs it before it ends waits for
     /// the rest of it (`SharedDeferred` initializes once and makes later
     /// callers wait); a caller that comes before any worker ran scans itself.
     static SCAN: SharedDeferred<Collection> = SharedDeferred::new();

     /// Whether the scan worker was started (once per process).
     static STARTED: AtomicBool = AtomicBool::new(false);

     /// Start the system font scan on its worker, once per process (PERF-10:
     /// it overlaps window and GPU setup). The worker is detached: it is noted
     /// in `DetachedThreads`, so an embedding host keeps the module loaded
     /// while it may run (`arvane-native`'s module retention).
     pub fn start_system_font_scan() {
         if STARTED.swap(true, Ordering::AcqRel) {
             return;
         }
         let spawned = std::thread::Builder::new()
             .name("arvane-fonts".to_owned())
             .spawn(|| {
                 SCAN.get_or_init(scan);
             });
         if spawned.is_ok() {
             DetachedThreads::note();
         }
     }

     /// A collection of the system fonts, waiting for the rest of the scan.
     pub(crate) fn collection() -> Collection {
         let since = Instant::now();
         let collection = SCAN.get_or_init(scan).clone();
         tracing::debug!(
             target: "arvane_text::fonts",
             waited_us = since.elapsed().as_micros() as u64,
             "system fonts joined"
         );
         collection
     }

     fn scan() -> Collection {
         Collection::new(CollectionOptions { shared: false, system_fonts: true })
     }

     // The scan crosses threads.
     const _: fn() = || {
         fn shared<T: Send + Sync>() {}
         shared::<Collection>();
     };

     #[cfg(test)]
     mod tests {
         use super::*;

         #[test]
         fn the_font_scan_worker_starts_once_and_is_noted_as_detached() {
             let threads: Vec<_> = (0..4).map(|_| std::thread::spawn(start_system_font_scan)).collect();
             for thread in threads {
                 thread.join().expect("start does not panic");
             }
             assert!(STARTED.load(Ordering::Acquire));
             assert!(DetachedThreads::any());
             assert!(collection().family_names().next().is_some() || cfg!(not(any(windows, target_os = "linux"))));
         }
     }
     ```
     `lib.rs` に `mod system_fonts;` と `pub use system_fonts::start_system_font_scan;` を足す。`Cargo.toml` に `tracing = { workspace = true }` が無ければ足す（workspace の依存。新しい crate ではない）。最後の `const _` が compile できない（`Collection` が `Send + Sync` でない）場合は STOP して報告する。`DetachedThreads` の path は `kivact_types::process::DetachedThreads`（`note()` と `any()`）。`family_names` が `&mut self` を要るなら試験の中で `let mut c = collection();` にする。
  2. `text_engine.rs`：`#[derive(Default)] pub struct TextEngine` の `#[derive(Default)]` を消し、field を 1 つ足す：
     ```rust
     pub struct TextEngine {
         pub(super) cache: Cache,
         pub(crate) fonts: FontContext,
         pub(crate) layouts: LayoutContext<u32>,
         /// Whether the system fonts are joined into `fonts` yet.
         system: SystemFonts,
     }

     /// Where the engine's system fonts are.
     enum SystemFonts {
         /// The engine never sees system fonts (`TextEngine::isolated`).
         Isolated,
         /// The scan may still run; the fonts registered meanwhile are kept to
         /// be registered again into the scanned collection.
         Pending { registered: Vec<parley::fontique::Blob<u8>> },
         /// `fonts.collection` holds the system fonts.
         Joined,
     }
     ```
     `pub fn new() -> Self` を次にする（`impl Default for TextEngine { fn default() -> Self { Self::new() } }` も足す）：
     ```rust
     /// An engine that sees the system fonts. Their scan starts now on a
     /// thread of its own; the first call that needs fonts waits for it.
     pub fn new() -> Self {
         crate::system_fonts::start_system_font_scan();
         Self {
             cache: Cache::default(),
             fonts: FontContext {
                 collection: parley::fontique::Collection::new(parley::fontique::CollectionOptions {
                     shared: false,
                     system_fonts: false,
                 }),
                 source_cache: parley::fontique::SourceCache::default(),
             },
             layouts: LayoutContext::new(),
             system: SystemFonts::Pending { registered: Vec::new() },
         }
     }
     ```
     `isolated` の `let mut engine = Self { .. }` に `system: SystemFonts::Isolated,` を足す。次の private method を足す：
     ```rust
     /// Join the system fonts into the collection, waiting for their scan,
     /// and register again the fonts registered before. The text caches go:
     /// the answers may change with the system fonts.
     fn join_system_fonts(&mut self) {
         let SystemFonts::Pending { registered } = &mut self.system else {
             return;
         };
         let registered = std::mem::take(registered);
         let mut collection = crate::system_fonts::collection();
         for blob in registered {
             collection.register_fonts(blob, None);
         }
         self.fonts.collection = collection;
         self.cache = Cache::default();
         self.system = SystemFonts::Joined;
     }
     ```
     次の公開 method の本体の **最初の行** に `self.join_system_fonts();` を足す：`family_names`、`has_family`、`measure`、`shape`、`shape_glyphs`、`inline.rs` の `shape_inline`、手順 4 の `resolve_family`。`register_font` は join しない：`register_fonts` の呼出しの直後に、`SystemFonts::Pending { registered }` なら `registered.push(blob.clone())` を足す（`Blob::new(..)` を変数 `blob` に受けてから `register_fonts(blob.clone(), None)` に渡す）。
  3. `crates/arvane-text/src/families.rs` を作り、family の解決を置く（arvane-asset の `font.rs` と facade の `engine.rs` から移す）：
     ```rust
     //! Role: Which installed family a CSS family name or generic family draws with.

     /// What a host draws a family name with.
     #[derive(Clone, Debug, PartialEq, Eq)]
     pub enum FamilyChoice {
         /// This installed family (or a CSS family list, as written).
         Family(String),
         /// The system UI font.
         SystemUi,
     }

     /// A family name resolved: what to draw with, and whether the name was
     /// a family that is not installed (reported once by the host).
     #[derive(Clone, Debug, PartialEq, Eq)]
     pub struct FamilyResolution {
         /// What to draw with.
         pub choice: FamilyChoice,
         /// The name was a specific family that is not installed.
         pub missing: bool,
     }

     /// A generic family name (CSS) Arvane resolves, ASCII case-insensitive.
     pub(crate) fn generic(name: &str) -> Option<parley::GenericFamily> {
         let name = name.trim();
         [
             ("system-ui", parley::GenericFamily::SystemUi),
             ("sans-serif", parley::GenericFamily::SansSerif),
             ("serif", parley::GenericFamily::Serif),
             ("monospace", parley::GenericFamily::Monospace),
         ]
         .into_iter()
         .find(|(css, _)| css.eq_ignore_ascii_case(name))
         .map(|(_, generic)| generic)
     }
     ```
     `text_engine.rs` に次の公開 method を足す：
     ```rust
     /// Resolve a CSS family name: a family list or a quoted name is kept as
     /// written (shaping reads the list); `system-ui` and an empty name are
     /// the system UI font; `sans-serif`, `serif` and `monospace` are the
     /// first family of fontique's generic family, or the system UI font; any
     /// other name is that family when installed, else the system UI font,
     /// reported missing.
     pub fn resolve_family(&mut self, name: &str) -> FamilyResolution {
         self.join_system_fonts();
         let source = name.trim();
         let found = |choice| FamilyResolution { choice, missing: false };
         if source.contains(',') || source.starts_with(['\'', '"']) {
             return found(FamilyChoice::Family(source.to_owned()));
         }
         if source.is_empty() {
             return found(FamilyChoice::SystemUi);
         }
         if let Some(generic) = crate::families::generic(source) {
             if generic == parley::GenericFamily::SystemUi {
                 return found(FamilyChoice::SystemUi);
             }
             let collection = &mut self.fonts.collection;
             let first = collection.generic_families(generic).next();
             let name = first.and_then(|id| collection.family_name(id).map(str::to_owned));
             return found(name.map_or(FamilyChoice::SystemUi, FamilyChoice::Family));
         }
         if self.has_family(source) {
             return found(FamilyChoice::Family(source.to_owned()));
         }
         FamilyResolution { choice: FamilyChoice::SystemUi, missing: true }
     }
     ```
     `parley::GenericFamily` が `PartialEq` でない場合は `matches!(generic, parley::GenericFamily::SystemUi)` を使う。`lib.rs` に `mod families;` と `pub use families::{FamilyChoice, FamilyResolution};` を足す。
  4. `crates/arvane-text/src/font_lookup.rs` を作る（SVG の `<text>` 用。A04 が使う）：
     ```rust
     //! Role: Font faces for text drawn off the UI thread (SVG `<text>` on the asset worker).

     use parley::fontique::{Blob, Collection, FontStyle, FontWeight, FontWidth, GenericFamily, SourceCache};
     use std::sync::Arc;

     /// A family as SVG names it.
     #[derive(Clone, Copy, Debug)]
     pub enum FamilyQuery<'a> {
         /// A family by name.
         Named(&'a str),
         /// `serif`.
         Serif,
         /// `sans-serif`.
         SansSerif,
         /// `monospace`.
         Monospace,
         /// `cursive`.
         Cursive,
         /// `fantasy`.
         Fantasy,
     }

     /// One face of a font file: the file's bytes and the face's index in it.
     #[derive(Clone)]
     pub struct FontFace {
         /// The whole font file.
         pub data: Arc<dyn AsRef<[u8]> + Send + Sync>,
         /// The face's index in the file (0 unless a collection).
         pub index: u32,
     }

     /// The system fonts and the App's fonts, for one thread that is not
     /// the UI thread. Making it waits for the system font scan.
     pub struct FontLookup {
         collection: Collection,
         sources: SourceCache,
     }

     impl FontLookup {
         /// The system fonts plus `app` (the App's font files, in registration order).
         pub fn new(app: &[kivact_types::shaped::FontSource]) -> FontLookup {
             let mut collection = crate::system_fonts::collection();
             for font in app {
                 collection.register_fonts(Blob::new(Arc::clone(font.source())), None);
             }
             FontLookup { collection, sources: SourceCache::default() }
         }

         /// The face of the first of `families` that is installed, matched to
         /// `weight` (100–900), `italic` and `width` (1.0 is normal).
         pub fn face(&mut self, families: &[FamilyQuery<'_>], weight: f32, italic: bool, width: f32) -> Option<FontFace> {
             let style = if italic { FontStyle::Italic } else { FontStyle::Normal };
             for family in families {
                 let id = match family {
                     FamilyQuery::Named(name) => self.collection.family_id(name),
                     FamilyQuery::Serif => self.collection.generic_families(GenericFamily::Serif).next(),
                     FamilyQuery::SansSerif => self.collection.generic_families(GenericFamily::SansSerif).next(),
                     FamilyQuery::Monospace => self.collection.generic_families(GenericFamily::Monospace).next(),
                     FamilyQuery::Cursive => self.collection.generic_families(GenericFamily::Cursive).next(),
                     FamilyQuery::Fantasy => self.collection.generic_families(GenericFamily::Fantasy).next(),
                 };
                 let Some(info) = id.and_then(|id| self.collection.family(id)) else {
                     continue;
                 };
                 let Some(font) = info.match_font(FontWidth::from_ratio(width), style, FontWeight::new(weight), true) else {
                     continue;
                 };
                 let index = font.index();
                 if let Some(blob) = font.load(Some(&mut self.sources)) {
                     return Some(FontFace { data: Arc::new(blob), index });
                 }
             }
             None
         }

         /// Faces of the families SVG text falls back to when a character is
         /// missing: the first family of the generic sans-serif, system-ui,
         /// serif, monospace and emoji families, in this order.
         pub fn fallback_faces(&mut self) -> Vec<FontFace> {
             let generics = [
                 GenericFamily::SansSerif,
                 GenericFamily::SystemUi,
                 GenericFamily::Serif,
                 GenericFamily::Monospace,
                 GenericFamily::Emoji,
             ];
             generics
                 .into_iter()
                 .filter_map(|generic| self.collection.generic_families(generic).next())
                 .filter_map(|id| self.collection.family(id))
                 .filter_map(|info| info.default_font().cloned())
                 .filter_map(|font| {
                     let index = font.index();
                     font.load(Some(&mut self.sources)).map(|blob| FontFace { data: Arc::new(blob), index })
                 })
                 .collect()
         }
     }
     ```
     fontique 0.11.1 の名前：`FontWidth::from_ratio(f32)`、`FontWeight::new(f32)`、`FontStyle::{Normal, Italic}`、`FamilyInfo::match_font(width, style, weight, synthesize_style: bool) -> Option<&FontInfo>`、`FamilyInfo::default_font() -> Option<&FontInfo>`、`FontInfo::index() -> u32`、`FontInfo::load(Option<&mut SourceCache>) -> Option<Blob<u8>>`、`Blob<u8>: AsRef<[u8]> + Send + Sync`（`linebender_resource_handle` 0.1.1）。`FontWidth::from_ratio` と `FontWeight::new` は `parlance` の型で、名前が違えば `rg -n "pub fn from_ratio|pub const fn new|pub fn new" ~/.cargo/registry/src/*/parlance-0.1.0/src/*.rs` で同じ意味の constructor を使う。`lib.rs` に `mod font_lookup;` と `pub use font_lookup::{FamilyQuery, FontFace, FontLookup};` を足す。
  5. `crates/arvane-asset/src/font.rs`：`pub enum Os`（と `impl Os`）、`pub enum GenericFamily`（と `impl GenericFamily`：`parse`、`candidates`）、`pub enum FamilyChoice`、`pub struct FamilyResolution`、`pub fn resolve_family` と、それだけが使う private の関数・試験を消す。`FontBytes`・`FontFile`・`load_font` とその private の `families` は残す（要設計判断 J-1）。`lib.rs` の `pub use font::{FamilyChoice, FamilyResolution, FontBytes, FontFile, GenericFamily, Os, load_font, resolve_family};` を `pub use font::{FontBytes, FontFile, load_font};` にし、crate の doc の `[`resolve_family`]`・`[`GenericFamily`]` を述べる箇条（「`resolve_family` maps a family name ...」の文）を消す。
  6. facade（W5 の A06 で消えるまで動かす）：`crates/arvane/src/asset.rs` の `pub use arvane_asset::{.., FamilyChoice, FamilyResolution, ..}` から `FamilyChoice, FamilyResolution` を外し、別の行 `pub use arvane_text::{FamilyChoice, FamilyResolution};` を足す。`crates/arvane/src/engine.rs` の `pub fn resolve_font_family(&mut self, name: &str) -> asset::FamilyResolution` の本体を `self.text.get_mut().resolve_family(name)` 1 行にする（カンマの特例と `arvane_asset::resolve_family(name, arvane_asset::Os::current(), ..)` を消す）。戻り値の型は `arvane_text::FamilyResolution`。
- **消すもの**：`arvane_asset::{Os, GenericFamily, FamilyChoice, FamilyResolution, resolve_family}`、facade の `resolve_font_family` のカンマの特例。
- **状態の閉じ方**：`TextEngine` の新しい field は enum 1 つ（`SystemFonts`：Isolated / Pending / Joined の 3 状態を variant で表す）。`FontLookup` は `bool`・`Option` を持たない（対象外）。
- **試験**：`crates/arvane-asset/src/font.rs` の `resolve_family`・`candidates` の単体試験は関数と一緒に消す（名前は `rg -n "fn .*\(\)" crates/arvane-asset/src/font.rs` の `#[cfg(test)]` の中で `resolve_family` か `candidates` を呼ぶもの。消す前に名前を unit の報告に列挙する）。`crates/arvane-text` に単体試験を 1 つ足す（`text_engine.rs` の末尾の `#[cfg(test)] mod tests`、無ければ作る）：
  ```rust
  #[test]
  fn generic_and_quoted_family_names_resolve_without_reporting() {
      let font: std::sync::Arc<[u8]> = std::sync::Arc::from(&include_bytes!("../../arvane/assets/fonts/Tuffy.ttf")[..]);
      let mut engine = TextEngine::isolated(&[font]).expect("the test font is usable");
      assert_eq!(engine.resolve_family("system-ui").choice, FamilyChoice::SystemUi);
      assert!(!engine.resolve_family("\"Inter\", sans-serif").missing);
      assert!(engine.resolve_family("No Such Family").missing);
      assert!(matches!(engine.resolve_family("serif").choice, FamilyChoice::Family(_)));
  }
  ```
- **確認（tier a）**：
  ```
  cargo fmt -p arvane-text -p arvane-asset -p arvane
  cargo check -p arvane-text -p arvane-asset -p arvane -p kivact-engine --all-targets --all-features
  cargo clippy -p arvane-text -p arvane-asset -p arvane --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane-text
  cargo xtask check --no-write --crate arvane-asset
  cargo test -p arvane-text -p arvane-asset
  ```
  E2E：`ARV`、`TEXT`、`ASSETS`。
- **完了条件**：`rg -w "Os|candidates|resolve_family" crates/arvane-asset/src` が 0 件。`rg -n "contains\(','\)" crates/arvane/src` が 0 件。`TextEngine::new` の中に `CollectionOptions { .. system_fonts: true }` が無い。`the_font_scan_worker_starts_once_and_is_noted_as_detached` が実行され通る（`cargo test -p arvane-text --lib -- system_fonts` が 1 件以上を実行する）。測定（記録だけ、合否に使わない）：最初の文字の計測が待った時間は `tracing` の `arvane_text::fonts` の debug の `waited_us` に出る。Orchestrator は CP1 の K1 の測定で `kv_startup` をこの log を有効にして起動し、`waited_us` を K1 と一緒に記録する（4.12 節）。上のコマンドと E2E が通る。

## A03：GPU の frame の scope、chunk ごとに 1 回の submit、pipeline の遅延 compile、compose の小さな整理

- **wave / lane / 前提**：W4 / A1 / A01
- **所有**：`crates/arvane-render/src/{gpu.rs,gpu/raster.rs,gpu/encode.rs,lib.rs}`、`crates/arvane-compose/src/`（`plan/` を除く全ファイル。`ComposeError` の改名で触る）、`crates/arvane-compose/Cargo.toml`
- **目的**：AR-01（GPU）：atlas・texture・derived の cache の frame を window の frame に合わせる。AR-04（GPU）：damage の矩形ごとの texture の新規確保と submit をやめ、chunk ごとに 1 回の submit にする（4.5 節）。AR-13：使わない pipeline を compile しない。AR-20：`ComposeError` の別名、`DeviceLost` の取り出して戻す形、`GpuSurface` の「keep のときだけ last」を消す。
- **手順**：
  1. `crates/arvane-render/src/gpu/raster.rs`：`GpuRaster` の公開の `pub fn raster(&mut self, device, queue, encoder, scene, canvas, view)` を消し、次を足す：
     ```rust
     /// One window frame of GPU rasters: image textures, atlas slots and
     /// derived tiles that any raster of the frame draws stay for the next
     /// frame; atlas slots no raster drew are freed when the frame ends.
     pub struct GpuFrameRaster<'a> {
         device: &'a wgpu::Device,
         queue: &'a wgpu::Queue,
         renderer: &'a mut Renderer,
         resources: &'a mut Resources,
         scene: &'a mut vello_hybrid::Scene,
         caches: &'a mut draw::Caches,
         images: &'a mut Images,
         textures: ImageScope<'a, GpuImage>,
         atlas: ImageScope<'a, AtlasImage>,
         derived: DerivedScope<'a>,
     }

     impl GpuRaster {
         /// Run `body` as one frame (AR-01). Atlas slots no raster of the
         /// frame drew are destroyed after it, in one submission.
         pub fn frame<R>(
             &mut self,
             device: &wgpu::Device,
             queue: &wgpu::Queue,
             body: impl FnOnce(&mut GpuFrameRaster<'_>) -> R,
         ) -> R {
             let GpuRaster { renderer, resources, scene, caches, images, textures, atlas, derived } = self;
             caches.validation.prune();
             let mut evicted = Vec::new();
             let result = atlas.frame_evicting(
                 |atlas| {
                     textures.frame(|textures| {
                         derived.frame(|derived| {
                             body(&mut GpuFrameRaster {
                                 device, queue, renderer: &mut *renderer, resources: &mut *resources,
                                 scene, caches, images, textures, atlas, derived,
                             })
                         })
                     })
                 },
                 |_, slot| evicted.push(slot.value.id),
             );
             destroy_atlas_images(renderer, resources, device, queue, evicted);
             result
         }
     }
     ```
     `fn destroy_atlas_images(renderer: &mut Renderer, resources: &mut Resources, device: &wgpu::Device, queue: &wgpu::Queue, evicted: Vec<ImageId>)`（private）は、今の `encode.rs` の `fn record` の中で `frame_evicting` の後にある「evicted の id を `renderer.destroy_image(resources, encoder, id)` で消し、"Arvane atlas uploads" の encoder を `queue.submit` する」部分をそのまま移したもの（`evicted` が空なら何もしない）。
     `impl GpuFrameRaster<'_>` に `pub fn raster(&mut self, encoder: &mut wgpu::CommandEncoder, scene: &RenderScene, canvas: Canvas, view: &wgpu::TextureView) -> Result<(), RenderError>` を作る。本体は消した `GpuRaster::raster` の本体で、`self.record(device, queue, scene, &frame, size)` と `self.draw(..)` は `GpuFrameRaster` の method として移した `record`・`draw` を呼ぶ（`device`・`queue` は `self.device`・`self.queue`）。
     `GpuRaster::new` の warm-up の `raster.raster(device, queue, &mut encoder, &RenderScene::default(), canvas, &view)?;` を `raster.frame(device, queue, |frame| frame.raster(&mut encoder, &RenderScene::default(), canvas, &view))?;` にする。
  2. `crates/arvane-render/src/gpu/encode.rs`：`impl GpuRaster { pub(super) fn record(..) }` を `impl GpuFrameRaster<'_> { pub(super) fn record(&mut self, source: &RenderScene, frame: &draw::Frame, (width, height): (u16, u16)) }` にする。本体の `let GpuRaster { .. } = self;` と `atlas.frame_evicting(..)`・`textures.frame(..)`・A01 で入れた `derived.frame(..)` の包みを消し、包みの中身を直接書く：`HybridCanvas { scene: self.scene, resources: self.resources, renderer: self.renderer, device: self.device, queue: self.queue, images: self.images, textures: &mut self.textures, atlas: &mut self.atlas, uploads: None }` を作り、`draw::draw(&mut canvas, self.caches, &mut self.derived, source, frame);` を呼び、`canvas.uploads.take()` を今と同じく扱う（atlas の upload の encoder を submit する部分はここに残す。evicted の後始末は手順 1 の `destroy_atlas_images` へ移った）。`fn draw(&mut self, ..)`（vello の render を呼ぶもの、今 `gpu/raster.rs`）も `impl GpuFrameRaster<'_>` に移す。`struct HybridCanvas<'a>` の `textures: ImageScope<'a, GpuImage>` と `atlas: ImageScope<'a, AtlasImage>` を `&'a mut ImageScope<'b, GpuImage>`・`&'a mut ImageScope<'b, AtlasImage>` にし、struct を `HybridCanvas<'a, 'b>` にする。`images::converted(&mut self.textures, ..)` は `images::converted(self.textures, ..)`、`self.atlas.get_or_insert_if_room(..)` はそのまま。
  3. `crates/arvane-render/src/gpu.rs`：`pub use raster::{Canvas, GpuRaster, RASTER_FORMAT};` を `pub use raster::{Canvas, GpuFrameRaster, GpuRaster, RASTER_FORMAT};` に、`lib.rs` の `pub use gpu::{Canvas, GpuRaster, RASTER_FORMAT};` を `pub use gpu::{Canvas, GpuFrameRaster, GpuRaster, RASTER_FORMAT};` にする。
  4. `crates/arvane-compose/src/gpu/exec.rs`：`fn draw_planned` の `for task in &chunks { self.raster_chunk(task, &mut stats)?; }` を次にする：
     ```rust
     raster_chunks(
         &mut self.raster, &self.device, &self.queue, &mut self.chunks, &mut self.pool, &chunks, &mut stats,
     )?;
     self.check()?;
     ```
     method `fn raster_chunk(&mut self, ..)` と `fn raster_into(&mut self, ..)` を消し、ファイルの末尾に次の 2 つの free 関数を置く：
     ```rust
     /// Raster every chunk task of one frame inside one GPU raster frame.
     fn raster_chunks(
         raster: &mut GpuRaster,
         device: &wgpu::Device,
         queue: &wgpu::Queue,
         chunks: &mut HashMap<ChunkKey, wgpu::Texture>,
         pool: &mut Pool,
         tasks: &[ChunkTask<'_>],
         stats: &mut FrameStats,
     ) -> Result<(), RenderError> {
         raster.frame(device, queue, |frame| {
             for task in tasks {
                 raster_chunk(frame, device, queue, chunks, pool, task, stats)?;
             }
             Ok(())
         })
     }

     /// Raster one chunk with one submission (vello_hybrid writes its
     /// buffers through the queue for every render, so renders are not
     /// batched into one encoder). Damaged rects are drawn once, as their
     /// bounding box into a pooled texture, and only the rects are copied.
     fn raster_chunk(
         frame: &mut GpuFrameRaster<'_>,
         device: &wgpu::Device,
         queue: &wgpu::Queue,
         chunks: &mut HashMap<ChunkKey, wgpu::Texture>,
         pool: &mut Pool,
         task: &ChunkTask<'_>,
         stats: &mut FrameStats,
     ) -> Result<(), RenderError> {
         let (w, h) = task.size;
         let (texture, redraw) = match chunks.remove(&task.key) {
             Some(texture) if (texture.width(), texture.height()) == (w, h) => (texture, task.redraw.clone()),
             _ => (targets::target(device, (w, h)), Redraw::All),
         };
         let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Arvane chunk raster") });
         let mut part = None;
         match redraw {
             Redraw::All => {
                 frame.raster(&mut encoder, &task.scene, canvas(task.origin, task.scale, (w, h))?, &view(&texture))?;
                 stats.pixels_rastered += u64::from(w) * u64::from(h);
             }
             Redraw::Rects(rects) => {
                 if let Some(b) = bounding(&rects) {
                     let origin = Point::new(
                         task.origin.x + f64::from(b.x) / task.scale,
                         task.origin.y + f64::from(b.y) / task.scale,
                     );
                     let patch = pool.take(device, (b.width, b.height));
                     frame.raster(&mut encoder, &task.scene, canvas(origin, task.scale, (b.width, b.height))?, &view(&patch))?;
                     for r in &rects {
                         encoder.copy_texture_to_texture(
                             wgpu::TexelCopyTextureInfo { texture: &patch, mip_level: 0, origin: wgpu::Origin3d { x: r.x - b.x, y: r.y - b.y, z: 0 }, aspect: wgpu::TextureAspect::All },
                             wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d { x: r.x, y: r.y, z: 0 }, aspect: wgpu::TextureAspect::All },
                             wgpu::Extent3d { width: r.width, height: r.height, depth_or_array_layers: 1 },
                         );
                         stats.pixels_rastered += u64::from(r.width) * u64::from(r.height);
                     }
                     part = Some(patch);
                 }
             }
         }
         queue.submit([encoder.finish()]);
         if let Some(patch) = part {
             pool.give(patch);
         }
         stats.chunks_rastered += 1;
         chunks.insert(task.key, texture);
         Ok(())
     }

     /// The raster canvas of `size` device px at `origin`.
     fn canvas(origin: Point, scale: f64, size: (u32, u32)) -> Result<Canvas, RenderError> {
         let w = u16::try_from(size.0).map_err(|_| RenderError::InvalidSize)?;
         let h = u16::try_from(size.1).map_err(|_| RenderError::InvalidSize)?;
         Ok(Canvas { origin, scale, size: (w, h) })
     }

     /// The smallest rect holding every rect of `rects`; `None` when empty.
     fn bounding(rects: &[PixelRect]) -> Option<PixelRect> { /* the same body as A01's `bounding` in arvane-render cpu.rs */ }
     ```
     `use` に `arvane_render::GpuFrameRaster`、`crate::gpu::targets::Pool`（今の path）、`std::collections::HashMap` を足す（足りないものだけ）。`clippy::too_many_arguments` が出る場合は、`device`・`queue`・`pool` を `struct ChunkGpu<'a> { device: &'a wgpu::Device, queue: &'a wgpu::Queue, pool: &'a mut Pool }`（private）にまとめて渡す。
  5. `ComposeError` の別名を消す：
     ```
     rg -l '\bComposeError\b' crates/arvane-compose/src | xargs perl -pi -e 's/\bComposeError\b/RenderError/g'
     ```
     `crates/arvane-compose/src/lib.rs` の `pub use arvane_render::RenderError as RenderError;`（置換の結果）を `pub(crate) use arvane_render::RenderError;` にする。`cargo check -p arvane-compose --all-features` が `the name RenderError is defined multiple times`（E0252）を出すファイルでは、`use crate::RenderError;` の行を消す（そのファイルは `arvane_render::RenderError` を既に import している）。`--no-default-features`（`gpu_unbuilt.rs`）でも同じ確認をする。
  6. `crates/arvane-compose/src/gpu/state.rs`：
     - `DeviceLost` を次にする（`Mailbox` と `Mutex<()>` を消す）：
       ```rust
       /// The first device loss or uncaptured error, kept for the presenter.
       #[derive(Default)]
       pub(crate) struct DeviceLost(kivact_types::cell::SharedDeferred<String>);

       impl DeviceLost {
           /// Keep `message` unless one was kept before.
           pub(crate) fn record(&self, message: String) {
               self.0.get_or_init(|| message);
           }

           /// The kept message.
           pub(crate) fn message(&self) -> Option<String> {
               self.0.get().cloned()
           }
       }
       ```
       `SharedDeferred` に `Default` が無ければ `#[derive(Default)]` を消し、`impl Default for DeviceLost { fn default() -> Self { Self(kivact_types::cell::SharedDeferred::new()) } }` を書く。
     - `GpuSurface` の `pub(crate) keep: Retain, pub(crate) last: Option<wgpu::Texture>` を `pub(crate) kept: Retain` 1 つにし、`Retain` を次にする（`derive` は `Debug` だけにする。`PartialEq`・`Copy` は texture を持つので外す）：
       ```rust
       /// Whether a surface keeps its last presented frame for read-back.
       pub(crate) enum Retain {
           /// Presented frames go back to the pool.
           Drop,
           /// The last presented frame is kept (none yet: `None`).
           Keep(Option<wgpu::Texture>),
       }
       ```
       `gpu/surface.rs` の使う側を直す：`GpuSurface::new` の `keep: Retain::Drop, last: None` は `kept: Retain::Drop`；`pub fn keep_frames(&mut self)` は `if matches!(self.kept, Retain::Drop) { self.kept = Retain::Keep(None); }`；`read_back` の `surface.last ... .filter(|_| surface.keep == Retain::Keep)` は `match &surface.kept { Retain::Keep(Some(texture)) => Some(texture), _ => None }` に相当する形；`fn retain(&mut self, surface: &mut GpuSurface, output: wgpu::Texture)` は：
       ```rust
       match &mut surface.kept {
           Retain::Keep(last) => {
               if let Some(old) = last.replace(output) {
                   self.pool.give(old);
               }
           }
           Retain::Drop => self.pool.give(output),
       }
       ```
  7. `crates/arvane-compose/src/gpu/pipelines.rs`（AR-13）：
     - `Pipelines` の field を次にする（他の field は今のまま）：
       ```rust
       /// The 11 composites and the shader blend, each made on first use.
       draw: [SharedDeferred<wgpu::RenderPipeline>; DRAW_PIPELINES],
       draw_module: wgpu::ShaderModule,
       draw_pl: wgpu::PipelineLayout,
       filter: SharedDeferred<wgpu::RenderPipeline>,
       filter_pl: wgpu::PipelineLayout,
       graph: SharedDeferred<wgpu::RenderPipeline>,
       graph_pl: wgpu::PipelineLayout,
       mask_step: SharedDeferred<wgpu::RenderPipeline>,
       mask_pl: wgpu::PipelineLayout,
       ```
       `const DRAW_PIPELINES: usize = SHADER_BLEND + 1;` を足す。`SharedDeferred` は `kivact_types::cell::SharedDeferred`。field の初期値は `std::array::from_fn(|_| SharedDeferred::new())` と `SharedDeferred::new()`。
     - `fn draw_pipelines(..) -> Vec<wgpu::RenderPipeline>` を、slot 1 つを作る `fn draw_pipeline(device: &wgpu::Device, layout: &wgpu::PipelineLayout, module: &wgpu::ShaderModule, slot: usize) -> wgpu::RenderPipeline` にする（今の loop の 1 回分。`slot == SHADER_BLEND` は `("vs_draw","fs_blend")`、それ以外は `("vs_draw","fs_draw")` とその slot の blend の factor）。
     - filter・graph・mask_step の module と pipeline を作る今の式を、それぞれ private 関数 `fn filter_pipeline(device, layout: &wgpu::PipelineLayout) -> wgpu::RenderPipeline`、`fn graph_pipeline(..)`（今あるものの module の作成も中に入れる）、`fn mask_step_pipeline(device, layout) -> wgpu::RenderPipeline` に移す（module はその関数の中で `shader(..)` で作る。`GRAPH_WGSL` の module は graph を初めて使う時だけ作られる）。
     - accessor を足す：
       ```rust
       pub(crate) fn draw(&self, device: &wgpu::Device, slot: usize) -> &wgpu::RenderPipeline {
           self.draw[slot].get_or_init(|| draw_pipeline(device, &self.draw_pl, &self.draw_module, slot))
       }
       pub(crate) fn filter(&self, device: &wgpu::Device) -> &wgpu::RenderPipeline {
           self.filter.get_or_init(|| filter_pipeline(device, &self.filter_pl))
       }
       pub(crate) fn graph(&self, device: &wgpu::Device) -> &wgpu::RenderPipeline {
           self.graph.get_or_init(|| graph_pipeline(device, &self.graph_pl))
       }
       pub(crate) fn mask_step(&self, device: &wgpu::Device) -> &wgpu::RenderPipeline {
           self.mask_step.get_or_init(|| mask_step_pipeline(device, &self.mask_pl))
       }
       ```
       `graph_pipeline` の今の引数が上と違えば、今の引数のうち layout と device 以外を `Pipelines` の field に足して渡す。
     - `Pipelines::new` の終わりで、毎 frame 使う draw の slot を先に作る：`pipelines.draw(device, composite_slot(Composite::SourceOver));`（`composite_slot` の今の署名に合わせる）。`apply_acc`・`apply_alpha`・`clear`・各 layout・dummy は今のまま eager。
     - 呼出し元：`self.pipelines.draw[slot]`（`draw.rs`）を `self.pipelines.draw(&self.device, slot)`、`.filter`（`filters.rs` の pipeline として使う箇所）を `.filter(&self.device)`、`.graph`（`graph.rs`）を `.graph(&self.device)`、`.mask_step`（`mask.rs`）を `.mask_step(&self.device)` にする。`_layout` の field の参照は変えない。
- **消すもの**：`GpuRaster::raster`、`GpuContext::raster_chunk`・`raster_into`、`ComposeError`、`DeviceLost` の `Mailbox` と `access`、`GpuSurface::{keep, last}`、`fn draw_pipelines`。
- **状態の閉じ方**：`GpuSurface` は `Option` の field を 1 つ（`Retain::Keep` の payload）だけ持つ（S-3 の解消）。`Pipelines` は `SharedDeferred` の field を 15 個持ち、どれも「使う時に作る」以外に書かれない（DEC-13 の用途。`SharedDeferred` は 6.1 節の内部可変の field に当たるので、`Pipelines` の宣言の直前に次の 1 行を置く）：`// xtask-allow(state): each pipeline is made on first use and never written again; no combination of made and unmade pipelines is invalid`。`GpuFrameRaster` は参照と scope だけ。
- **試験**：足す試験は無い。
- **確認（tier a）**：
  ```
  cargo fmt -p arvane-render -p arvane-compose
  cargo check -p arvane-render -p arvane-compose --all-targets --all-features
  cargo check -p arvane-compose --all-targets --no-default-features
  cargo clippy -p arvane-render -p arvane-compose --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane-render
  cargo xtask check --no-write --crate arvane-compose
  cargo test -p arvane-render -p arvane-compose --all-features
  ```
  E2E：`ARV`、`GPU-DEMAND`、`GPU-SHARE`、`DRAW`。GPU の機械では `ARV` と `GPU-SHARE` を 2.2 節の GPU の形でも実行する。
- **完了条件**：`rg -w ComposeError crates/` が 0 件。`rg -n "Arvane chunk patch" crates/` が 0 件。`rg -n "draw_pipelines|Mailbox" crates/arvane-compose/src` が 0 件。上のコマンドと E2E が通る。

## A04：SVG の `<text>` の font を arvane-text から渡す（fontdb の 2 度目の走査をやめる）、asset の worker を初めての仕事で起こす

- **wave / lane / 前提**：W4 / A2 / A02
- **所有**：`crates/arvane-asset/`（`Cargo.toml` を含む）、`Cargo.lock`、`dev/xtask/policy/workspace.toml`（`visible_to` の `arvane-text` の 1 行）
- **目的**：AR-11（DEC-09 の版 2）：resvg・tiny-skia による SVG の描画は **今のまま保つ**（mask・pattern・filter・clipPath などの能力を落とさない。12 章 EA-15）。usvg の `<text>` の font は `usvg::Options::font_resolver` で arvane-text の `FontLookup`（A02）から渡し、`fontdb::Database::load_system_fonts`（system font の 2 度目の走査）と resvg の feature `system-fonts` を消す。AR-12（asset）：`AssetStore::new` は worker の thread を作らず、最初の仕事で作る（PERF-10）。
- **手順**：
  1. `crates/arvane-asset/Cargo.toml`：`resvg = { version = "0.46.0", default-features = false, features = ["text", "system-fonts", "raster-images"] }` の feature から `"system-fonts"` を消し、その上のコメントを次の 2 行にする。arvane-text の依存を足す：
     ```toml
     # `text` turns SVG `<text>` into paths with the fonts arvane-text finds (no
     # system font database of its own); `raster-images` draws data URL images.
     resvg = { version = "0.46.0", default-features = false, features = ["text", "raster-images"] }
     arvane-text = { path = "../arvane-text", version = "0.1.0" }
     ```
     `dev/xtask/policy/workspace.toml` の `visible_to` の `arvane-text = [...]` の一覧に `"arvane-asset"` を足す。`cargo check` が `Cargo.lock` を書き換えるのを受け入れる（fontdb の `fs`・`fontconfig`・`memmap` の feature と、それだけが引く crate が消える）。
  2. `crates/arvane-asset/src/svg/fonts.rs` を次の全文にする（`fn has_text` とその doc は今のファイルのものを末尾にそのまま残す）：
     ```rust
     //! Role: The fonts SVG `<text>` is turned into paths with.

     use arvane_text::{FamilyQuery, FontFace, FontLookup};
     use kivact_types::shaped::FontSource;
     use resvg::usvg;
     use resvg::usvg::fontdb;
     use std::sync::{Arc, Mutex};

     /// The App's font files for SVG text, in registration order. The system
     /// fonts come from arvane-text's process-wide scan when a document has text.
     pub(crate) struct SvgFonts {
         app: Vec<FontSource>,
     }

     impl SvgFonts {
         /// No App fonts yet.
         pub(crate) fn new() -> Self {
             Self { app: Vec::new() }
         }

         /// The same App fonts plus `font`.
         pub(crate) fn with_font(&self, font: FontSource) -> Self {
             let mut app = self.app.clone();
             app.push(font);
             Self { app }
         }

         /// The font resolver usvg uses for `svg`: no font when the document
         /// has no text, else the system and App fonts, loaded into usvg's
         /// database face by face as the text asks for them.
         pub(crate) fn resolver(&self, svg: &[u8]) -> usvg::FontResolver<'static> {
             if !has_text(svg) {
                 return usvg::FontResolver {
                     select_font: Box::new(|_, _| None),
                     select_fallback: Box::new(|_, _, _| None),
                 };
             }
             let lookup = Arc::new(Mutex::new(FontLookup::new(&self.app)));
             let fallback = Arc::clone(&lookup);
             usvg::FontResolver {
                 select_font: Box::new(move |font, db| {
                     let families: Vec<FamilyQuery<'_>> = font.families().iter().map(family).collect();
                     let italic = font.style() != usvg::FontStyle::Normal;
                     let face = lock(&lookup).face(&families, f32::from(font.weight()), italic, width(font.stretch()))?;
                     load(db, &face)
                 }),
                 select_fallback: Box::new(move |c, used, db| {
                     let default = usvg::FontResolver::default_fallback_selector();
                     if let Some(id) = default(c, used, db) {
                         return Some(id);
                     }
                     for face in lock(&fallback).fallback_faces() {
                         load(db, &face);
                     }
                     default(c, used, db)
                 }),
             }
         }
     }

     fn lock(lookup: &Mutex<FontLookup>) -> std::sync::MutexGuard<'_, FontLookup> {
         lookup.lock().expect("invariant: the SVG font lookup lives on one parse and never panics while held")
     }

     fn family(family: &usvg::FontFamily) -> FamilyQuery<'_> {
         match family {
             usvg::FontFamily::Serif => FamilyQuery::Serif,
             usvg::FontFamily::SansSerif => FamilyQuery::SansSerif,
             usvg::FontFamily::Cursive => FamilyQuery::Cursive,
             usvg::FontFamily::Fantasy => FamilyQuery::Fantasy,
             usvg::FontFamily::Monospace => FamilyQuery::Monospace,
             usvg::FontFamily::Named(name) => FamilyQuery::Named(name),
         }
     }

     /// CSS `font-stretch` as a width ratio.
     fn width(stretch: usvg::FontStretch) -> f32 {
         match stretch {
             usvg::FontStretch::UltraCondensed => 0.5,
             usvg::FontStretch::ExtraCondensed => 0.625,
             usvg::FontStretch::Condensed => 0.75,
             usvg::FontStretch::SemiCondensed => 0.875,
             usvg::FontStretch::Normal => 1.0,
             usvg::FontStretch::SemiExpanded => 1.125,
             usvg::FontStretch::Expanded => 1.25,
             usvg::FontStretch::ExtraExpanded => 1.5,
             usvg::FontStretch::UltraExpanded => 2.0,
         }
     }

     /// Load `face` into usvg's database and give its id there (a font
     /// collection's faces are loaded in index order).
     fn load(db: &mut Arc<fontdb::Database>, face: &FontFace) -> Option<fontdb::ID> {
         let ids = Arc::make_mut(db).load_font_source(fontdb::Source::Binary(Arc::clone(&face.data)));
         ids.get(face.index as usize).copied()
     }
     ```
     usvg 0.46.0 の型：`FontResolver { select_font: Box<dyn Fn(&Font, &mut Arc<fontdb::Database>) -> Option<fontdb::ID> + Send + Sync>, select_fallback: Box<dyn Fn(char, &[fontdb::ID], &mut Arc<fontdb::Database>) -> Option<fontdb::ID> + Send + Sync> }`、`FontResolver::default_fallback_selector()`、`Font::{families, style, stretch, weight}`、`FontFamily` は `svgtypes::FontFamily` の再公開（`usvg::FontFamily`）、`usvg::fontdb` の再公開。`FontLookup` が `Send` でないため closure が compile できない場合は STOP して報告する。
  3. `crates/arvane-asset/src/svg/document.rs` の `pub(crate) fn parse`：`usvg::Options { .. fontdb: fonts.for_document(bytes), .. }` の `fontdb: ..` の行を `font_resolver: fonts.resolver(bytes),` にする（`fontdb` は `usvg::Options::default()` の空の database のまま）。`fonts` の型 `FontDb = Arc<super::SvgFonts>` と、`resvg::render` による `rasterize` は変えない。`svg.rs` の crate の doc の「`<text>` is drawn with the store's font database (the system fonts and the fonts registered with it ...)」の文を「`<text>` is turned into paths with the system and registered fonts arvane-text finds (`svg/fonts.rs`); no second system font database is built」にする。`lib.rs` の doc の「no external type (image, resvg, skrifa)」はそのまま。
  4. worker を初めての仕事で起こす（AR-12）：`crates/arvane-asset/src/store/asset_store.rs` の `pub(super) enum Runner { Inline, Worker(Worker) }` に variant を足す：
     ```rust
     /// No job yet: the worker thread is made by the first job (PERF-10).
     Idle(Box<dyn Fn() + Send>),
     ```
     `pub fn new(waker: impl Fn() + Send + 'static) -> AssetStore` の本体を `AssetStore::with_runner(Runner::Idle(Box::new(waker)))` にする。`fn submit(&mut self, job: Job)` の先頭に次を足す：
     ```rust
     if let Runner::Idle(_) = &self.runner {
         let Runner::Idle(waker) = std::mem::replace(&mut self.runner, Runner::Inline) else {
             unreachable!("invariant: matched Idle just above");
         };
         self.runner = Worker::spawn(waker).map_or(Runner::Inline, Runner::Worker);
     }
     ```
     `Runner` を match する他の箇所（`rg -n "Runner::" crates/arvane-asset/src`）では、`Runner::Idle(_)` を「受け取る仕事が無い」として `Runner::Inline` と同じ arm に入れる。
- **消すもの**：resvg の feature `system-fonts`、`SvgFonts::for_document` と `SvgFonts.db`（`Mutex<Option<Arc<Database>>>`）、`fontdb::Database::load_system_fonts` の呼出し。resvg・tiny-skia・`rasterize` は消さない。
- **状態の閉じ方**：`Runner` は enum（Idle / Inline / Worker）。`SvgFonts` は `Vec` 1 つ。
- **試験**：足す試験は 2 つ（TV-09 の不変条件）。`crates/arvane-asset/src/svg/fonts.rs` の末尾の `#[cfg(test)] mod tests`：
  ```rust
  #[test]
  fn svg_text_uses_a_registered_font_without_a_system_font_database() {
      let tuffy: std::sync::Arc<[u8]> = std::sync::Arc::from(&include_bytes!("../../../arvane/assets/fonts/Tuffy.ttf")[..]);
      let fonts = SvgFonts::new().with_font(kivact_types::shaped::FontSource::from_shared(tuffy));
      let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><text x="0" y="15" font-family="Tuffy" font-size="12">Ab</text></svg>"#;
      let options = usvg::Options { font_resolver: fonts.resolver(svg), ..usvg::Options::default() };
      let tree = usvg::Tree::from_data(svg, &options).expect("valid SVG");
      assert!(tree.root().has_children(), "the text became paths");
      assert!(tree.fontdb().faces().all(|face| matches!(face.source, usvg::fontdb::Source::Binary(_))), "no font was read from the system font directories");
  }

  #[test]
  fn an_svg_without_text_loads_no_font() {
      let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><rect width="4" height="4"/></svg>"#;
      let options = usvg::Options { font_resolver: SvgFonts::new().resolver(svg), ..usvg::Options::default() };
      let tree = usvg::Tree::from_data(svg, &options).expect("valid SVG");
      assert_eq!(tree.fontdb().len(), 0);
  }
  ```
  `FontSource::from_shared` の名前は `rg -n "pub fn from_shared|pub fn from_static" crates/kivact-types/src` で確かめる（`FontBytes::to_source` が使っている constructor）。`fontdb::Database::len`・`faces()`・`FaceInfo::source` の名前は fontdb 0.23 のもの。`crates/arvane/tests/e2e/assets.rs` と `ASSETS`（`svg_embedded_text` を含む）の試験は期待値を変えずに通ること。
- **確認（tier a）**：
  ```
  cargo fmt -p arvane-asset
  cargo check -p arvane-asset -p arvane -p kivact-engine --all-targets --all-features
  cargo clippy -p arvane-asset --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane-asset
  cargo test -p arvane-asset
  cargo tree -p arvane-asset -e features -i fontdb
  ```
  最後の 1 つ：fontdb の feature に `fs`・`fontconfig`・`memmap` が無いこと。E2E：`ARV`、`ASSETS`、`OFFSCREEN`。
- **完了条件**：`rg -n "load_system_fonts|system-fonts" crates/arvane-asset` が 0 件。`rg -n "resvg::render" crates/arvane-asset/src` が 1 件（描画は残る）。上のコマンドと E2E が通り、各 filter が 1 件以上の試験を実行する。

## A05：`GpuStart::Never`、`prefer_gpu` の削除、`DriverOptions`

- **wave / lane / 前提**：W5b / 鎖 / W5a の鎖の全 unit（C13 を含む）と W5a の E の lane（E12・E13・E16）
- **所有**：`crates/kivact-types/src/{gpu.rs,embed.rs}`、`crates/arvane/src/{env.rs,window.rs,embed.rs,presentation.rs,presenter/frame_policy.rs,lib.rs}`、`crates/arvane/tests/e2e/{support.rs,embed.rs,presenter.rs}`、`crates/kivact-engine/src/{host/launch.rs,embed.rs,host/testing/test_host.rs}`、`crates/kivact-core/src/embed/open.rs`、`crates/kivact-test/src/options.rs`、`dev/xtask/policy/workspace.toml`（`"arvane::ShellSetup"` の行）
- **目的**：5.6 節と AR-16：GPU の起動の希望は `GpuPreference.start` 一つにし、CPU だけは `GpuStart::Never`。`prefer_gpu` をすべての運び手から消す。driver の設定を `DriverOptions` 一つにし、`Shell`・`ShellSetup`・`Embedder`・`Embedded` を消す。AR-20：`limits_noted` を bool にし、`RendererChange::LimitExceeded` を `GpuLimit` にする。
- **手順**：
  1. `crates/kivact-types/src/gpu.rs` の `enum GpuStart` を次にする（derive は今のまま）：
     ```rust
     /// When the window's GPU device is started. `ARVANE_RENDERER=cpu|gpu|auto`
     /// overrides it with `Never|AtLaunch|OnDemand`.
     pub enum GpuStart {
         /// Start the device when a window needs it (the first frames are drawn on the CPU).
         #[default]
         OnDemand,
         /// Start the device with the window (frames are drawn on the CPU until it is ready).
         AtLaunch,
         /// Never start a device: every frame is drawn on the CPU.
         Never,
     }
     ```
  2. `crates/kivact-types/src/embed.rs`：`EmbedOptions` の field `pub prefer_gpu: bool,` とその doc 1 行、`Default` の `prefer_gpu: true,` を消す。
  3. `crates/arvane/src/env.rs`：`fn resolve(forced: Option<&str>, prefer_gpu: bool, start: GpuStart) -> Self` を `fn resolve(forced: Option<&str>, start: GpuStart) -> Self` にし、本体を次にする：
     ```rust
     match forced {
         Some("cpu") => Self::Cpu,
         Some("gpu") => Self::GpuAtLaunch,
         Some("auto") => Self::GpuOnDemand,
         _ => match start {
             GpuStart::Never => Self::Cpu,
             GpuStart::AtLaunch => Self::GpuAtLaunch,
             GpuStart::OnDemand => Self::GpuOnDemand,
         },
     }
     ```
     `pub(crate) fn read(prefer_gpu: bool, preference: kivact_types::GpuPreference) -> Self` を `pub(crate) fn read(preference: kivact_types::GpuPreference) -> Self` にし、`RendererStart::resolve(forced.as_deref(), preference.start)` を呼ぶ。module と `RendererStart` の doc の `prefer_gpu` を述べる語句を「`GpuPreference::start`」に直す。
  4. `crates/arvane/src/presentation.rs`：`RendererChange::LimitExceeded { what: String }` を unit variant `GpuLimit` にし（doc：`/// A scene the GPU renderer could not draw (an atlas or texture limit) was drawn on the CPU, noted once per window.`）、`Display` の arm を `RendererChange::GpuLimit => write!(f, "renderer=gpu limit_exceeded what=gpu_limit")` にする（出力の文字列は今と同じ）。次の関数を足す：
     ```rust
     /// Make an [`Observer`] of `f`.
     pub fn observer(f: impl FnMut(WindowId, &RendererChange) + 'static) -> Observer {
         Rc::new(RefCell::new(f))
     }
     ```
  5. `crates/arvane/src/presenter/frame_policy.rs`：`Presenter` の field `limits_noted: HashSet<String>` を `/// The GPU limit was noted (once per window).` `gpu_limit_noted: bool` にし、`Presenter::new` で `false`。`fn note_limit(&mut self, what: &str)` を次にする：
     ```rust
     fn note_gpu_limit(&mut self) {
         if !self.gpu_limit_noted {
             self.gpu_limit_noted = true;
             self.notes.note(RendererChange::GpuLimit);
         }
     }
     ```
     `self.note_limit("gpu_limit");` を `self.note_gpu_limit();` にする。使われなくなった `HashSet` の import は、`shader_failed: HashSet<u64>` が残るので消さない。
  6. `crates/arvane/src/window.rs`：`pub struct Shell`、`impl Shell`（`new`・`prefer_gpu`・`gpu_preference`・`on_renderer_change`・`run`・`start_stepped`）、`pub struct ShellSetup`、`impl Default for ShellSetup` を消し、次を置く：
     ```rust
     /// How the driver draws: when the GPU device starts and on which adapter,
     /// and who is told of renderer changes.
     #[derive(Default)]
     pub struct DriverOptions {
         /// The GPU device's adapter, API and start (`GpuStart::Never`: CPU only).
         /// `ARVANE_RENDERER`, `WGPU_BACKEND` and `WGPU_POWER_PREF` override it.
         pub gpu: GpuPreference,
         /// Called on the window's thread with every renderer change, starting
         /// with the main window's `RendererChange::Start`.
         pub observer: Option<Observer>,
     }

     /// Run `backend`'s window loop with the driver until it exits. Construct
     /// and run on the application's main thread. A callback panic propagates.
     pub fn run(backend: Box<dyn Backend>, options: DriverOptions, handler: impl Handler) -> Result<(), ShellError> {
         let mut driver = driver(handler, options);
         backend.run(&mut driver)
     }

     /// Start the same driver on an explicitly stepped backend.
     pub fn start_stepped(
         backend: Box<dyn SteppedBackend>,
         options: DriverOptions,
         handler: impl Handler + 'static,
     ) -> Result<SteppedShell, ShellError> {
         backend.start(Box::new(driver(handler, options))).map(SteppedShell)
     }
     ```
     `pub(crate) fn driver<H: Handler>(handler: H, prefer_gpu: bool, gpu_preference: GpuPreference, observer: Option<Observer>) -> Driver<H>` を `pub(crate) fn driver<H: Handler>(handler: H, options: DriverOptions) -> Driver<H>` にし、`let env = crate::env::Env::read(options.gpu);`、`observer: options.observer`。`Windows` の field `observer: Option<Observer>` を `observer: &'a Option<Observer>` にし、`Driver` の 3 箇所の `observer: self.observer.clone(),` を `observer: &self.observer,` にする（`Windows::open` の `Notes::new(self.observer.clone(), id)` はそのまま。`Rc` の clone は window を開く時だけになる）。`Windows` を作る式で borrow が衝突する場合（`presenters: &mut self.presenters` と同時）は、`Driver` を destructure してから作る。
  7. `crates/arvane/src/embed.rs`：`pub struct Embedder<H>`・`impl Embedder`・`pub fn driver`・`pub struct Embedded`・`impl Embedded` を消し、次を置く（`pub use arvane_shell::{ChildStop, EmbedNotice, PanicSlot};`・`pub use arvane_shell::ChildHooks;`・`pub use kivact_types::EmbedOptions;` は A06 が消すまで残す）：
     ```rust
     /// The driver of an embedded window, made on the window's UI thread:
     /// `make` builds the handler and the optional renderer observer there.
     pub fn embed_driver<H: Handler + 'static>(
         gpu: kivact_types::GpuPreference,
         make: impl FnOnce() -> (H, Option<Observer>) + Send + 'static,
     ) -> MakeHandler {
         Box::new(move || {
             let (handler, observer) = make();
             Box::new(crate::window::driver(handler, crate::window::DriverOptions { gpu, observer }))
                 as Box<dyn arvane_shell::ShellHandler>
         })
     }
     ```
  8. `crates/arvane/src/lib.rs`：`pub use window::{Handler, Shell, ShellSetup, SteppedShell, Window, Windows};` を `pub use window::{DriverOptions, Handler, SteppedShell, Window, Windows, run, start_stepped};` に、`pub use presentation::Observer;` を `pub use presentation::{Observer, observer};` にし、`pub use embed::embed_driver;` を足す（`pub mod embed;` は残す）。
  9. engine：
     - `crates/kivact-engine/src/host/launch.rs` の `pub fn run_native` の、`let shell = Shell::new(..)` から `shell.run(handler)` までを次の形にする（backend の式と observer の closure の本体は今のもの）：
       ```rust
       let backend = arvane_native::backend(WindowRequest::main("Kivact", window_size))
           .map_err(|e| HostError::new(e.to_string()))?;
       let observer = arvane::observer(move |window, change| { /* the closure body given to on_renderer_change today */ });
       arvane::run(backend, arvane::DriverOptions { gpu: options.gpu_preference, observer: Some(observer) }, handler)
       ```
       `use arvane::{.., Shell, ..}` から `Shell` を外す。
     - `crates/kivact-engine/src/host/testing/test_host.rs` の `fn start_shell`：`Shell::start_stepped(Box::new(backend), ShellSetup { prefer_gpu: options.gpu == GpuMode::Required, gpu_preference: options.gpu_preference, observer }, shared)` を次にする：
       ```rust
       let gpu = if options.gpu == GpuMode::Required {
           options.gpu_preference
       } else {
           kivact_types::GpuPreference { start: kivact_types::gpu::GpuStart::Never, ..options.gpu_preference }
       };
       arvane::start_stepped(Box::new(backend), arvane::DriverOptions { gpu, observer }, shared)
       ```
       `fn renderer_observer` の `match` の `RendererChange::LimitExceeded { .. }` を `RendererChange::GpuLimit` にする。`use` から `Shell`・`ShellSetup` を外す。
     - `crates/kivact-engine/src/embed.rs` の `pub fn embed_native`：`let make = arvane::embed::driver(&options, move || { .. Embedder::new(host).on_renderer_change(..) });` を `let make = arvane::embed_driver(options.gpu_preference, move || { /* the same body up to building host */ (host, Some(arvane::observer(/* the closure given to on_renderer_change */))) });` にする。`arvane_native::embed::open(parent, options, hooks, make).map(Embedded::new)` の `.map(Embedded::new)` を消す。`struct EmbeddedHost` の `host: Embedded` を `host: Box<dyn arvane::spi::ShellChild>` にし、`EmbeddedHost` の method（`set_size`、`size`、`set_scale`、`scale`、`set_visibility`、`visibility`、`stop`）は `self.host.<同名>(..)` をそのまま呼ぶ（`ShellChild` に同名の method がある）。`use arvane::embed::{Embedded, Embedder};` を消す。
  10. `crates/kivact-core/src/embed/open.rs`：`prefer_gpu: true,` の行を消す。
  11. `crates/kivact-test/src/options.rs`：`TestGpu::Off | TestGpu::AtLaunch => kivact_types::gpu::GpuStart::AtLaunch,` を 2 行 `TestGpu::Off => kivact_types::gpu::GpuStart::Never,` と `TestGpu::AtLaunch => kivact_types::gpu::GpuStart::AtLaunch,` にする。
  12. arvane の試験：`crates/arvane/tests/e2e/support.rs` の `ShellSetup { prefer_gpu: gpu, gpu_preference: .., observer }` を、`DriverOptions { gpu: if gpu { <the gpu_preference expression> } else { GpuPreference { start: GpuStart::Never, ..<the gpu_preference expression> } }, observer }` にし、`Shell::start_stepped(` を `arvane::start_stepped(` にする。`crates/arvane/tests/e2e/embed.rs` の 3 箇所の `prefer_gpu: false,` を `gpu_preference: kivact_types::GpuPreference { start: kivact_types::gpu::GpuStart::Never, ..Default::default() },` にし（同じ式の中に `gpu_preference` が既にあればそれを置き換える）、`embed::driver(&options, || Embedder::new(h))` の形を `arvane::embed_driver(options.gpu_preference, || (h, None))` にする（observer を付けていた箇所は `Some(arvane::observer(..))`）。`presenter.rs` などで `RendererChange::LimitExceeded` を match していれば `RendererChange::GpuLimit` にする。
  13. `dev/xtask/policy/workspace.toml`：`"arvane::ShellSetup",` の行があれば `"arvane::DriverOptions",` に置き換える（行が無ければ何もしない。T が `structural-options` の検査を消していればその一覧ごと無い）。
- **消すもの**：`prefer_gpu`（すべて）、`Shell`、`ShellSetup`、`Embedder`、`Embedded`、`arvane::embed::driver`、`RendererChange::LimitExceeded`、`Presenter::limits_noted`、`note_limit`。
- **状態の閉じ方**：`DriverOptions` は名前が `Options` で終わる（対象外）。`Presenter` の `gpu_limit_noted: bool` は `Presenter` の唯一の bool で、他の `Option`・counter と組にならない（`shader_failed` は集合）。T が `Presenter` に印を付けていれば、その文面の `limits_noted` を `gpu_limit_noted` に直す。
- **試験**：足す試験は無い。期待値は変えない。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-types -p arvane -p kivact-engine -p kivact-core -p kivact-test
  cargo check --workspace --all-targets --all-features --exclude kivact-docs
  cargo clippy -p kivact-types -p arvane -p kivact-engine -p kivact-core -p kivact-test --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane
  cargo xtask check --no-write --crate kivact-types
  ```
  E2E：`ARV`、`EMBED`、`GPU-DEMAND`、`LIFE`。
- **完了条件**：`rg -w "prefer_gpu|ShellSetup|Embedder|LimitExceeded" crates/ dev/ examples/ templates/` が 0 件（`docs/` は付属書 G）。`rg -n "struct Shell\b" crates/arvane` が 0 件。上のコマンドと E2E が通る。

## A06：arvane の facade を消す（engine は arvane-text・arvane-asset・arvane-render・arvane-compose・arvane-shell を直接使う）

- **wave / lane / 前提**：W5b / 鎖 / A05
- **所有**：`crates/arvane/`（`Cargo.toml`・`src/`・`tests/`）、`crates/arvane-text/src/{text_engine.rs,inline.rs,error.rs,lib.rs}`、`crates/arvane-render/src/{error.rs,lib.rs}`、`crates/kivact-engine/`（`Cargo.toml` と、`arvane::` を書く全ファイル）、`dev/xtask/policy/workspace.toml`（`visible_to` の 4 行）、`dev/tools/check-features.sh`
- **目的**：AR-06（4.1 節）：arvane は window の driver と presenter だけにする。他の crate の再公開・転送（`Engine`、`Rasterizer`、`asset`、`spi`、facade の error の写し）を消す。AR-19：`test_font`・`gpu_unavailable`・`lose_gpu_device` を arvane の feature `test` の後ろに置く。AR-20：`RefCell<TextEngine>` が消える。
- **手順**：
  1. arvane-text の API を facade の形にする（engine の呼出しを機械的に移すため）：
     - `crates/arvane-text/src/error.rs`：`pub enum TextError` を `pub(crate) enum TextError` にし、次を足す（`crates/arvane/src/error.rs` の `text_kind` と `text` を移したもの）：
       ```rust
       /// The shared failure of a text error: invalid input is `Content`, a
       /// font that cannot be used is `Resource`.
       pub(crate) fn failure(error: TextError) -> kivact_types::Failure {
           let kind = match error {
               TextError::InvalidMetrics
               | TextError::InvalidSpans
               | TextError::InvalidInlineBoxes
               | TextError::UnsupportedInlineStyle => kivact_types::FailureKind::Content,
               TextError::InvalidFont | TextError::MissingFont => kivact_types::FailureKind::Resource,
           };
           kivact_types::Failure::new(kind, error.to_string())
       }
       ```
       `lib.rs` の `pub use error::TextError;` を消す。
     - `TextEngine` の公開 method の戻り値の `Result<T, TextError>` をすべて `Result<T, kivact_types::Failure>` にし、本体の最後で `.map_err(crate::error::failure)` を付ける（内部の private 関数は `TextError` のまま）。`measure`・`shape`・`shape_glyphs`・`shape_inline` の `text: RichText<'_>` 引数を `text: impl Into<RichText<'t>>`（lifetime 引数 `'t` を method に足す）にし、本体の先頭で `let text = text.into();`。`isolated` の戻り値も `Result<Self, kivact_types::Failure>`。
  2. `crates/arvane-render/src/error.rs` に次を足す（`crates/arvane/src/error.rs` の `render` を移したもの）：
     ```rust
     impl From<RenderError> for kivact_types::Failure {
         /// A device or surface problem keeps its kind; an invalid scene is invalid content.
         fn from(error: RenderError) -> Self {
             let kind = match &error {
                 RenderError::Backend(_) => kivact_types::FailureKind::Backend,
                 RenderError::NoAdapter(_) => kivact_types::FailureKind::Unavailable,
                 RenderError::Surface(_) => kivact_types::FailureKind::Surface,
                 _ => kivact_types::FailureKind::Content,
             };
             kivact_types::Failure::new(kind, error.to_string())
         }
     }
     ```
  3. arvane：
     - 消すファイル：`src/engine.rs`、`src/rasterizer.rs`、`src/asset.rs`、`src/spi.rs`、`src/availability.rs`、`src/test_font.rs`、`src/error.rs`、`src/embed.rs`。`embed.rs` の `embed_driver`（A05）は `src/window.rs` の末尾へ移す。
     - 新しいファイル `src/test_hooks.rs`（`#[cfg(feature = "test")]` の module）：
       ```rust
       //! Role: Test-only hooks of the driver: the bundled test font, a GPU probe and a device loss.

       use std::sync::Arc;

       /// The bundled test font (Tuffy, public domain; its license is
       /// `assets/fonts/Tuffy-LICENSE.txt`). It has no CJK glyphs.
       static TEST_FONT: &[u8] = include_bytes!("../assets/fonts/Tuffy.ttf");

       /// The bundled test font: an isolated text engine given it shapes,
       /// lays out and draws text the same on every machine.
       pub fn test_font() -> Arc<[u8]> {
           Arc::from(TEST_FONT)
       }

       /// Why no hardware GPU device can be made here, or `None` when one can.
       pub fn gpu_unavailable() -> Option<String> {
           arvane_compose::GpuContext::new(&arvane_compose::GpuEnv::from_env(kivact_types::GpuPreference::default()))
               .err()
               .map(|error| error.to_string())
       }

       /// Make the window's GPU device lost, as a device failure does: its
       /// exports are reported lost and the device restarts once per session.
       pub fn lose_gpu_device(window: &mut crate::Window<'_>) {
           window.lose_gpu_device();
       }
       ```
     - `Cargo.toml`：`[dependencies]` から `arvane-text`・`arvane-asset` を消し、`arvane-protocol = { path = "../arvane-protocol", version = "0.1.0" }` が無ければ足す。`[features]` を足す：
       ```toml
       [features]
       # Test hooks: the bundled test font, a GPU probe and a device loss (`kivact-engine/testing`).
       test = []
       ```
       `[[test]] name = "e2e"` に `required-features = ["test"]` を足す。`[dev-dependencies]` に `arvane-text = { path = "../arvane-text" }`、`arvane-asset = { path = "../arvane-asset" }` を足す。`description` を `"Arvane: the window driver and presenter behind kivact-engine"` にする。
     - `src/lib.rs` を次の形にする（crate の doc は 4.1 節の責務で書き直す。下は doc の後）：
       ```rust
       //! Role: Arvane's window driver: the shell's handler and every window's presenter.
       //!
       //! Responsibility: drives a shell backend (`arvane-shell`'s SPI,
       //! implemented by `arvane-native` and `arvane-virtual`) and presents
       //! each window's composition through `arvane-compose`, owning the
       //! renderers' life cycle and the frame policy. It re-exports no other
       //! crate: hosts use `arvane-shell`, `arvane-text`, `arvane-asset`,
       //! `arvane-render` and `arvane-compose` directly.
       #![forbid(unsafe_code)]

       mod env;
       mod presentation;
       mod presenter;
       #[cfg(feature = "test")]
       mod test_hooks;
       mod window;

       pub use presentation::{GpuStartCause, Observer, Renderer, RendererChange, observer};
       pub use presenter::{ExportNotice, FramePlan, Presented};
       #[cfg(feature = "test")]
       pub use test_hooks::{gpu_unavailable, lose_gpu_device, test_font};
       pub use window::{DriverOptions, Handler, SteppedShell, Window, Windows, embed_driver, run, start_stepped};
       ```
     - 残るファイルの中の `crate::Failure`・`crate::FailureKind` を `kivact_types::{Failure, FailureKind}` に、`crate::error::render(e)` を `kivact_types::Failure::from(e)` に、`crate::ShellWaker`・`crate::WakeReasons` などの再公開の名前を `arvane_shell::…` に、`crate::spi::…` を `arvane_shell::…` にする（`rg -n "crate::(Failure|FailureKind|ShellWaker|WakeReasons|ShellEvent|ShellError|ShellControl|EventTarget|InjectedInput|ShellImeText|AccessibilityError|SizeReply|spi|error)" crates/arvane/src` の各行）。
     - `src/window.rs` の `pub(crate) fn lose_gpu_device(&mut self)` に `#[cfg(feature = "test")]` を付ける（`presenter/frame_policy.rs` の `Presenter::lose_gpu_device` にも）。
  4. engine の `Cargo.toml`：`[dependencies]` に次を足す：
     ```toml
     arvane-text = { path = "../arvane-text", version = "0.1.0" }
     arvane-asset = { path = "../arvane-asset", version = "0.1.0" }
     arvane-render = { path = "../arvane-render", version = "0.1.0" }
     arvane-compose = { path = "../arvane-compose", version = "0.1.0" }
     ```
     `testing = [...]` の一覧に `"arvane/test"` を足す。`dev/xtask/policy/workspace.toml` の `visible_to` で、`arvane-text`・`arvane-asset`・`arvane-render`・`arvane-compose` の各一覧に `"kivact-engine"` を足す。`arvane` の一覧から外す名前は無い。
  5. engine の `arvane::` を移す。4.3 節の表が正本。手順：
     - 型と関数の path（`use` の group 以外）を置換する：
       ```
       rg -l 'arvane::' crates/kivact-engine/src | xargs perl -pi -e '
         s/\barvane::asset::(FamilyChoice|FamilyResolution)\b/arvane_text::$1/g;
         s/\barvane::asset::/arvane_asset::/g;
         s/\barvane::Engine\b/arvane_text::TextEngine/g;
         s/\barvane::(glyph_outline|glyph_ink_bounds|MAX_SCENE_DEPTH)\b/arvane_render::$1/g;
         s/\barvane::spi::lose_gpu_device\b/arvane::lose_gpu_device/g;
         s/\barvane::spi::/arvane_shell::/g;
         s/\barvane::embed::(ChildHooks|ChildStop|EmbedNotice|PanicSlot)\b/arvane_shell::$1/g;
         s/\barvane::(Failure|FailureKind)\b/kivact_types::$1/g;
         s/\barvane::(Accessibility|AccessibilityError|EventTarget|FileDialog|InjectedInput|ShellControl|ShellError|ShellEvent|ShellImeText|ShellWaker|SizeReply|WakeReasons)\b/arvane_shell::$1/g;
       '
       ```
       （作業 tree の root で実行する。`ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1` を先に。）
     - `use arvane::{..};` の group は perl で分けられないので手で分ける：group の中の名前のうち、**残す名前**（`Window`、`Windows`、`Handler`、`Presented`、`ExportNotice`、`FramePlan`、`Renderer`、`RendererChange`、`GpuStartCause`、`Observer`、`observer`、`DriverOptions`、`SteppedShell`、`run`、`start_stepped`、`embed_driver`、`test_font`、`gpu_unavailable`、`lose_gpu_device`）は `use arvane::{..}` に残し、それ以外は 4.3 節の表の crate の `use` に移す。対象の group（`rg -n "use arvane::\{" crates/kivact-engine/src` が全部を出す）：`host/events.rs`、`host/pump.rs`、`host/launch.rs`、`host/testing/test_host.rs`、`host/testing.rs`、`contract.rs`、`embed.rs`、`lower/assets.rs`。
     - `crates/kivact-engine/src/host/launch.rs` の `pub fn run_native` と `crates/kivact-engine/src/embed.rs` の `pub fn embed_native` の本体の最初の行に `arvane_text::start_system_font_scan();` を足す（EA-09：window と GPU の準備の前に走査を始める。二度目の呼出しは何もしない）。
     - `crates/kivact-engine/src/a11y/build/{payload.rs,node.rs,dirty.rs}` の `use arvane as a;` を消し、`a::Engine` を `arvane_text::TextEngine` にする（`perl -pi -e 's/\ba::Engine\b/arvane_text::TextEngine/g'` をその 3 ファイルに）。
     - facade の method 名の違いを直す（呼出し元は次の 7 箇所だけ。`TextEngine` が receiver のものだけを変える）：`lower/assets.rs` の `engine.resolve_font_family(name)` → `engine.resolve_family(name)`；`host/write/reader.rs` の `font_families()` の呼出し → `family_names()`（`TextEngine::family_names` は `&mut self`。reader が `&` しか持たないなら、その field の `Rc<RefCell<..>>` を `borrow_mut()` で取る）；`host/testing/test_host.rs` の `ArvaneTestHost::has_font_family` の中の `engine.has_font_family(family)` → `engine.has_family(family)`（`ArvaneTestHost::has_font_family` という engine 側の method 名は変えない）；`lower/glyphs.rs` と `measure/leaf.rs` の `.text_metrics(` → `.measure(`；`host/testing/test_host.rs` の `arvane_text::TextEngine::with_fonts_only(` → `arvane_text::TextEngine::isolated(`。
     - `crates/kivact-engine/src/offscreen.rs`：field `rasterizer: arvane::Rasterizer` を `compositor: arvane_compose::CpuCompositor` にし、`arvane::Rasterizer::new()` → `arvane_compose::CpuCompositor::new()`、`self.rasterizer.release_caches()` → `self.compositor.release_caches()`、`arvane::Rasterizer::check_size(width, height).is_ok()` → `arvane_render::dimensions(kivact_types::Size { width: f64::from(width), height: f64::from(height) }, 1.0).is_ok()`、`self.rasterizer.render(composition, size.0, size.1).map_err(OffscreenError::Failed)?` → `self.compositor.render(composition, (size.0, size.1), 1.0).map(|rendered| rendered.image).map_err(|e| OffscreenError::Failed(kivact_types::Failure::from(e)))?`。
     - `TextEngine` の method が `Failure` を返すようになったので、engine の `map_err` の型の不一致が出た箇所は `Failure` をそのまま受ける形にする（facade の `Engine` も `Failure` を返していたので、型は同じになる）。
  6. kivact-test：`kivact_engine::testing::{ShellEvent, ShellImeText}` は engine の `host/testing.rs` の `pub use arvane_shell::{ShellEvent, ShellImeText};`（手順 5 の結果）から来るので、kivact-test の変更は要らない。`cargo check -p kivact-test` が失敗する場合だけ、その名前の path を 4.3 節の表で直す。
  7. arvane の試験 `crates/arvane/tests/e2e/*.rs`：手順 5 の perl と同じ置換を、対象の path を `crates/arvane/tests/e2e` にして実行し、`use arvane::{..}` の group を同じ規則で分ける。`Engine::with_fonts_only(&[test_font()])` は `arvane_text::TextEngine::isolated(&[arvane::test_font()])`、`Engine::new()` は `arvane_text::TextEngine::new()`。`arvane::asset::{Asset, ..}` の `Asset` は `kivact_types::Asset`。
  8. `dev/tools/check-features.sh` の feature の一覧に `arvane/test` を足す（既存の行の書き方に合わせる）。
- **消すもの**：`arvane::{Engine, Rasterizer, asset, spi, embed, Failure, FailureKind}` と arvane の `pub use arvane_*` のすべて、`crates/arvane/src/{engine.rs,rasterizer.rs,asset.rs,spi.rs,availability.rs,test_font.rs,error.rs,embed.rs}`、`arvane_text::TextError` の公開。
- **状態の閉じ方**：新しい状態の型は無い。`RefCell<TextEngine>`（facade の `Engine`）は消える。
- **試験**：足す試験は無い。`crates/arvane/src/engine.rs` の compile_fail の doctest（`Engine` が `Send` でないこと）はファイルと一緒に消える。
- **確認（tier a）**：
  ```
  cargo fmt -p arvane -p arvane-text -p arvane-render -p kivact-engine
  cargo check --workspace --all-targets --all-features --exclude kivact-docs
  cargo check -p kivact-engine --no-default-features
  cargo clippy -p arvane -p arvane-text -p arvane-render -p kivact-engine --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane
  cargo xtask check --no-write --crate kivact-engine
  cargo test -p arvane-text
  ```
  E2E：`ARV`（`--features test`）、`TEXT`、`ASSETS`、`OFFSCREEN`、`EMBED`、`LIFE`。
- **完了条件**：`rg -n "arvane::(Engine|Rasterizer|asset|spi|embed|Failure|Shell[A-Z]|EventTarget|WakeReasons|InjectedInput|glyph_)" crates/ dev/ examples/` が 0 件。`rg -n "^pub use arvane_" crates/arvane/src` が 0 件。上のコマンドと E2E が通る。

## A07：`ShellWindow` を window の service に絞り、surface を `WindowSurface` に分ける

- **wave / lane / 前提**：W5b / 鎖 / A06
- **所有**：`crates/arvane-shell/src/{window.rs,ime.rs,lib.rs}`、`crates/arvane-native/src/port/winit/{window.rs,open.rs}`、`crates/arvane-native/src/port/win32/window.rs`、`crates/arvane-native/src/port/x11/window.rs`、`crates/arvane-native/tests/smoke.rs`、`crates/arvane-virtual/src/window.rs`、`crates/arvane/src/{window.rs,presenter/*}`、`crates/arvane/tests/e2e/environment.rs`、`crates/kivact-types/src/platform.rs`、`crates/kivact-engine/src/{host/pump.rs,host/window.rs,host/foreign.rs,host/write/writer.rs,contract/host.rs}`、`crates/kivact-core/src/{platform/services.rs,view/foreign_window.rs}`
- **目的**：AR-07：`ShellWindow` から使われない method（`reduced_motion`、`cpu_format`/`CpuFormat`、`focus`）と重複（`frame_period_ns`、`PlatformRequest::RequestWindowSize`）を消し、IME の 5 つの setter を `set_ime(Option<ImeRequest>)` 一つにし、presenter だけが使う surface の method を `trait WindowSurface` に分ける。AR-19：`keeps_frames`/`keep_frame` を `frame_sink` にする。AR-21：`PlatformRequest::ForeignChild` を消し `ShellWindow::foreign_child` にする。
- **手順**：
  1. `crates/arvane-shell/src/ime.rs` に次を足す：
     ```rust
     /// What the focused editor asks of the platform IME; `None` (in
     /// `ShellWindow::set_ime`) disables IME input for the window.
     #[derive(Clone, Debug, PartialEq)]
     pub struct ImeRequest {
         /// The editor's content purpose and keyboard hint.
         pub purpose: kivact_types::InputPurpose,
         /// The caret rectangle, logical window px.
         pub cursor_area: kivact_types::Rect,
         /// The surrounding text and selection, when the editor publishes them.
         pub text: Option<ShellImeText>,
         /// Ask for the software keyboard where the backend has one.
         pub soft_keyboard: bool,
     }
     ```
  2. `crates/arvane-shell/src/window.rs`：
     - `enum CpuFormat` を消す。
     - `trait ShellWindow` から `focus`、`cpu_format`、`keeps_frames`、`keep_frame`、`wait_until`、`recover_surface`、`shows_cpu_frame`、`release_cpu_surface`、`reduced_motion`、`set_ime_purpose`、`set_ime_allowed`、`set_soft_keyboard`、`set_ime_cursor_area`、`set_ime_text`、`frame_period_ns`、`present_with`、`with_gpu_target` を外す。`request_size` の default 本体（`perform(RequestWindowSize)` を呼ぶもの）を消し、必須の method にする。次を足す：
       ```rust
       /// Enable IME input for the focused editor as `ime` asks, or disable it (`None`).
       fn set_ime(&self, ime: Option<crate::ime::ImeRequest>);
       /// The native child window of a foreign-content window, made when the
       /// window opened; `None` for any other window.
       fn foreign_child(&self) -> Option<kivact_types::NativeWindow> {
           None
       }
       /// The window's presentation surface (the presenter's only).
       fn surface(&mut self) -> &mut dyn WindowSurface;
       ```
     - 新しい trait（同じファイル。外した method の署名と doc をそのまま移す）：
       ```rust
       /// The presentation side of a window, used by the presenter only.
       pub trait WindowSurface {
           fn present_with(&mut self, width: u32, height: u32, damage: Option<&[PixelRect]>, draw: &mut dyn FnMut(&mut [u32], BufferAge) -> bool) -> Result<bool, ShellError>;
           fn with_gpu_target(&self, make: Box<dyn FnOnce(crate::target::GpuTarget) + Send>) -> Result<(), ShellError>;
           fn recover_surface(&mut self) -> Result<(), ShellError>;
           fn shows_cpu_frame(&self) -> bool;
           fn release_cpu_surface(&mut self);
           fn wait_until(&mut self, done: &mut dyn FnMut() -> bool, limit: std::time::Duration) -> bool;
           /// Where the presenter hands frames it read back for inspection;
           /// `None` for windows that keep none (every native window).
           fn frame_sink(&mut self) -> Option<&mut dyn FrameSink>;
       }

       /// A keeper of read-back frames (test windows).
       pub trait FrameSink {
           /// Keep `frame`, a frame the presenter read back after showing it.
           fn keep(&mut self, frame: arvane_protocol::CpuFrame);
       }
       ```
     - `lib.rs`：`CpuFormat` の再公開を消し、`FrameSink`、`WindowSurface`、`ime::ImeRequest` を公開する（既存の `pub use window::{..}` と `pub use ime::{..}` の行に足す）。
  3. 各 backend（`port/winit/window.rs` の `WinitWindow`、`port/win32/window.rs` の `Window`、`port/x11/window.rs` の `Window`、`arvane-virtual/src/window.rs` の `VirtualWindow`）で同じ形に直す：
     - `impl ShellWindow for X` から手順 2 で外した method の実装を、`impl WindowSurface for X { .. }` に移す（本体はそのまま）。`keeps_frames` と `keep_frame` の代わりに `fn frame_sink(&mut self) -> Option<&mut dyn FrameSink>` を書く：native の 3 つは `None`。virtual は `if <the old keeps_frames body> { Some(self) } else { None }` と、`impl FrameSink for VirtualWindow { fn keep(&mut self, frame: CpuFrame) { <the old keep_frame body> } }`。
     - `impl ShellWindow for X` に `fn surface(&mut self) -> &mut dyn WindowSurface { self }` を足す。
     - `focus`・`reduced_motion`・`frame_period_ns` の実装は、impl の外の private の inherent method（同じ本体）にする。`perform(PlatformRequest::FocusWindow)` の中の `self.focus()` と、`environment()` の中の `self.reduced_motion()` は inherent を呼ぶ。`cpu_format` の実装は消す。
     - IME：5 つの setter の実装を、同じ名前の private の inherent method（`fn ime_purpose(&self, ..)`、`fn ime_allowed(&self, ..)`、`fn soft_keyboard(&self, ..)`、`fn ime_cursor_area(&self, ..)`、`fn ime_text(&self, ..)`。本体はそのまま）にし、trait の `set_ime` を次の順で呼ぶ形で書く：
       ```rust
       fn set_ime(&self, ime: Option<ImeRequest>) {
           match ime {
               None => {
                   self.soft_keyboard(false);
                   self.ime_allowed(false);
               }
               Some(request) => {
                   self.ime_allowed(true);
                   self.soft_keyboard(request.soft_keyboard);
                   self.ime_cursor_area(request.cursor_area);
                   self.ime_purpose(request.purpose);
                   self.ime_text(request.text);
               }
           }
       }
       ```
       winit の `set_soft_keyboard` が `self.set_ime_allowed(shown)` を呼んでいた箇所は `self.ime_allowed(shown)` にする。
     - `request_size`：winit は `perform` の `PlatformRequest::RequestWindowSize(size) => { .. request_inner_size .. }` の arm の本体を `fn request_size(&mut self, size: Size, request: RequestId) -> SizeReply { let _ = request; <the arm body>; SizeReply::Now(PlatformResult::Done) }` に移す（arm の `Ok(..)` の包みは外す）。virtual は `perform` の `if let PlatformRequest::RequestWindowSize(size) = request { self.requested_sizes.push(size); return PlatformResult::Done; }` を `fn request_size(&mut self, size: Size, _: RequestId) -> SizeReply { self.requested_sizes.push(size); SizeReply::Now(PlatformResult::Done) }` に移す。win32 と x11 は今 `request_size` を実装しているのでそのまま。各 `perform` の `PlatformRequest::RequestWindowSize(..)` の arm を消す。
     - `foreign_child`：winit は `perform` の `PlatformRequest::ForeignChild => match &self.foreign { .. }` の arm を消し、`fn foreign_child(&self) -> Option<NativeWindow> { self.foreign.as_ref().map(|f| f.native()) }` を書く。virtual は `if let PlatformRequest::ForeignChild = request { return self.foreign_child_result(); }` を消し、`fn foreign_child(&self) -> Option<NativeWindow> { match self.foreign_child_result() { PlatformResult::NativeWindow(child) => Some(child), _ => None } }` を書く。win32・x11 は `PlatformRequest::ForeignChild` の arm を消すだけ（trait の default の `None`）。
     - `crates/arvane-native/tests/smoke.rs` の `present_with` の呼出しは `window.surface().present_with(..)` にする。
  4. `crates/kivact-types/src/platform.rs`：`PlatformRequest` の variant `RequestWindowSize(Size)` と `ForeignChild`（と doc）を消す。
  5. engine と core の要求の経路（`RequestWindowSize` の代わり）：
     - `crates/kivact-engine/src/contract/host.rs` の `HostRequest` に variant を足す：`/// Ask the platform to resize window `window`'s content to `size` logical px; answered like a platform request.` `RequestWindowSize { window: WindowId, size: Size },`
     - `crates/kivact-engine/src/host/write/writer.rs`：`HostRequest::Platform { .. }` を `DeferredRequest::Platform(platform)` にする arm（`.push((request, DeferredRequest::Platform(platform)))`）の隣に、同じ形で `HostRequest::RequestWindowSize { window, size } => <the same push into window's deferred list> DeferredRequest::RequestSize(size)` の arm を足す（`window` の扱いは `Platform` の arm と同じ）。`DeferredRequest` の定義（`rg -n "enum DeferredRequest" crates/kivact-engine/src`）に `RequestSize(Size),` を足す。
     - `crates/kivact-engine/src/host/pump.rs` の `fn effects` の `DeferredRequest::Platform(PlatformRequest::RequestWindowSize(size)) => { .. }` の arm を `DeferredRequest::RequestSize(size) => { .. }`（本体は同じ）にする。
     - `crates/kivact-core/src/platform/services.rs` の `request_window_size` と `crates/kivact-core/src/view/foreign_window.rs` の `request_size` の `HostRequest::Platform { window, request: PlatformRequest::RequestWindowSize(size) }` を `HostRequest::RequestWindowSize { window, size }` にする（`request` などの他の field を同じ式で渡していれば、`HostRequest::RequestWindowSize` の形に合わせて落とす。返り値の待ち方は変えない）。
  6. engine の IME（`crates/kivact-engine/src/host/window.rs`・`host/pump.rs`）：
     - `HostWindow` の field `soft_keyboard: Sent<SoftKeyboard>` と `ime_allowed: Sent<ImeAllowed>` を消し、次の 2 つを足す。`enum SoftKeyboard` と `enum ImeAllowed` を消す：
       ```rust
       /// The IME request last sent to the window (`None`: IME disabled).
       pub(super) ime: Sent<Option<arvane_shell::ImeRequest>>,
       /// The editor's surrounding text as last published by the model.
       pub(super) ime_text: Option<arvane_shell::ShellImeText>,
       ```
       初期値は `Sent::new()` と `None`（他の `Sent` field と同じ作り方）。
     - `fn synchronize_ime(&mut self, id, window: &mut Window<'_>)` の本体を次の形にする（`caret`・`model.ime_purpose()`・`model.ime_changed(engine)`・`ImeUpdate::Set(text)` は今の式）：
       ```rust
       // 1. caret = the current caret expression (model.caret(engine).or_else(|| model.session_area()))
       // 2. if let ImeUpdate::Set(text) = model.ime_changed(engine) { host_window.ime_text = text; }
       let request = caret.map(|cursor_area| arvane_shell::ImeRequest {
           purpose: model.ime_purpose(),
           cursor_area,
           text: host_window.ime_text.clone(),
           soft_keyboard: true,
       });
       if host_window.ime.update(request.clone()).is_some() {
           window.set_ime(request);
       }
       ```
       `Sent::update` が初回に `None` を返す実装なら（`rg -n "fn update" -A12 crates/kivact-types/src/cell.rs`）、初回も送るために、今の `ime_allowed.update(..)` の使い方と同じ判定を使う。`fn defer` の `set_ime_purpose` と `set_ime_text` の 2 箇所は、`self.synchronize_ime(id, &mut window)` の 1 回の呼出しにする。
     - `crates/arvane/src/window.rs` の `Window` の `set_ime_purpose`・`set_ime_allowed`・`set_ime_cursor_area`・`set_ime_text`・`set_soft_keyboard` を消し、`pub fn set_ime(&self, ime: Option<arvane_shell::ImeRequest>) { self.shell.set_ime(ime); }` と `pub fn foreign_child(&self) -> Option<kivact_types::NativeWindow> { self.shell.foreign_child() }` を足す。
     - `crates/kivact-engine/src/host/foreign.rs` の `prepare_foreign`：`let PlatformResult::NativeWindow(child) = window.perform(PlatformRequest::ForeignChild) else { .. };` を `let Some(child) = window.foreign_child() else { .. };`（else の本体は今のまま）にする。
  7. presenter（`crates/arvane/src/presenter/*`・`window.rs`）：
     - `window.keeps_frames()` → `window.surface().frame_sink().is_some()`；`window.keep_frame(frame)` → `if let Some(sink) = window.surface().frame_sink() { sink.keep(frame); }`；`present_with`・`with_gpu_target`・`recover_surface`・`shows_cpu_frame`・`release_cpu_surface`・`wait_until` の呼出しは `window.surface().<同名>(..)`。`&dyn ShellWindow` を受ける関数（`Presenter::prepare`、`make_surface`）でこれらを呼ぶものは `&mut dyn ShellWindow` を受けるようにし、呼出し元を `&mut` で渡す。
     - `frame_period_ns` の重複を消す：`Presenter` に field `/// The display period, from the window's environment.` `frame_period_ns: u64` を足し、`Presenter::new` の後で（`Windows::open` の `open_window` の後と `Driver::start`）`presenter.set_frame_period(window.environment().frame_period_ns)` を呼ぶ（`pub(crate) fn set_frame_period(&mut self, period_ns: u64)`）。`Driver::event` の始めで、event が `ShellEvent::EnvironmentChanged`・`ShellEvent::Resized`・`ShellEvent::Focused(true)` で target が window のとき、その presenter に `set_frame_period(window.environment().frame_period_ns)` を呼ぶ。`present_native` の `let (now, period) = (window.now_ns(), window.frame_period_ns());` を `let (now, period) = (window.now_ns(), self.frame_period_ns);` にする。
  8. `crates/arvane/tests/e2e/environment.rs` の IME の setter の試験（`set_ime_purpose` などを呼ぶ箇所）を `window.set_ime(Some(arvane_shell::ImeRequest { purpose, cursor_area, text, soft_keyboard }))` と `window.set_ime(None)` に直す。期待する観測（virtual の IME の状態）は変えない。
- **消すもの**：`CpuFormat`、`ShellWindow::{focus, cpu_format, keeps_frames, keep_frame, reduced_motion, set_ime_purpose, set_ime_allowed, set_soft_keyboard, set_ime_cursor_area, set_ime_text, frame_period_ns}`（trait から）、`PlatformRequest::{RequestWindowSize, ForeignChild}`、engine の `SoftKeyboard`・`ImeAllowed`、`arvane::Window` の 5 つの IME の method。
- **状態の閉じ方**：`ImeRequest` は値（`&mut self` の method が無い）で対象外。`HostWindow` は `Sent` を 2 つ失い 1 つ得、`Option` の `ime_text` を得る。`ime_text` は `ime` と別の事実（model が最後に公開した文字列）で、`ime` はその写しを含む送った値。T が `HostWindow` に付けた印の文面に `soft_keyboard`・`ime_allowed` があれば、`ime`（送った IME の要求）と `ime_text`（model の最後の文字列）に直す。`Presenter` の `frame_period_ns: u64` は counter ではなく値。
- **試験**：足す試験は無い。`environment.rs` は手順 8 のとおり呼び方だけ直す。
- **確認（tier a）**：
  ```
  cargo fmt --all
  cargo check --workspace --all-targets --all-features --exclude kivact-docs
  cargo clippy -p arvane-shell -p arvane-native -p arvane-virtual -p arvane -p kivact-engine -p kivact-core -p kivact-types --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane-shell
  cargo xtask check --no-write --crate arvane-native
  cargo xtask check --no-write --crate arvane
  ```
  E2E：`ARV`、`NATIVE-SMOKE`、`EMBED`、`FOREIGN`、`TEXT`、`LIFE`。
- **完了条件**：`rg -n "set_ime_(purpose|allowed|cursor_area|text)|set_soft_keyboard|CpuFormat|frame_period_ns\(\)|RequestWindowSize\(|PlatformRequest::ForeignChild|keeps_frames|keep_frame" crates/ dev/` が 0 件（`frame_period_ns` の field は残る）。上のコマンドと E2E が通る。

## A08：描けるかの判定を `Drawable` 一つに、embed の表示を `ShellEvent::Visibility`、foreign window に presenter を作らない

- **wave / lane / 前提**：W5b / 鎖 / A07
- **所有**：`crates/arvane-shell/src/{events.rs,window.rs,surface.rs(新),lib.rs}`、`crates/arvane-native/src/port/embed/surface.rs`（削除）、`crates/arvane-native/src/port/embed.rs`、`crates/arvane-native/src/port/win32/{state.rs,window.rs}`、`crates/arvane-native/src/port/x11/{window.rs,thread.rs}`、`crates/arvane-native/src/port/winit/window.rs`、`crates/arvane-virtual/src/{surface.rs(削除),window.rs,backend.rs,lib.rs}`、`crates/arvane/src/{window.rs,presenter.rs,presenter/frame.rs,presenter/frame_policy.rs}`、`crates/kivact-engine/src/host/{window.rs,events.rs,pump.rs,window_changes.rs}`、`crates/arvane/tests/e2e/present.rs`
- **目的**：AR-08（変更して採用）：「この window は描けるか」を backend が `Drawable` で一度だけ答え、facade と engine の再計算（`no_surface`・`no_pixels`・`surface_of`）を消す。embed の表示・非表示は `ShellEvent::Visibility(bool)` で知らせ、GPU の surface は非表示の間も保つ（`SurfaceDestroyed`・`SurfaceCreated`・`Suspended`・`Resumed`・`SaveState`・`KeyboardInset`・`MemoryWarning` はモバイルの語彙として残す）。`Presented::NotDrawable(Drawable)` が「描かなかったが失敗ではない」を表す。AR-09（engine）：idle の frame の解放は CPU の frame の後だけ。AR-22：foreign の window に presenter を作らない。
- **手順**：
  1. `crates/arvane-shell/src/window.rs` に次を足し、`ShellWindow` の `fn surface_generation(&self) -> Option<u64>;` を `fn drawable(&self) -> Drawable;` に替える：
     ```rust
     /// Whether a window can show a frame now.
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub enum Drawable {
         /// Yes, on the surface of `generation` (a new surface, a new generation).
         Yes {
             /// The surface's generation; the first surface is 1.
             generation: std::num::NonZeroU64,
         },
         /// The window is hidden by its host (an embedded editor); its
         /// surface is kept.
         Hidden,
         /// The platform destroyed the window's surface (a mobile suspension).
         NoSurface,
         /// The window has no pixels (minimized, or sized 0).
         NoPixels,
     }
     ```
  2. `crates/arvane-shell/src/events.rs`：`ShellEvent` に `/// An embedded window was shown (`true`) or hidden by its host. Its surface is kept.` `Visibility(bool),` を足す（`SurfaceCreated` の次）。`EventTarget::of` は window の event として扱う（今の `_ => Window` の側。変更不要か確かめる）。
  3. `crates/arvane-shell/src/surface.rs`（新）：
     ```rust
     //! Role: The generation of a window's platform surface across platform destroys.

     use crate::events::ShellEvent;
     use std::num::NonZeroU64;

     /// A platform surface that the platform may destroy and make again
     /// (mobile suspension); each new surface has a new generation.
     #[derive(Clone, Copy, Debug)]
     pub enum SurfacePresence {
         /// The surface of `generation` (the first is 1).
         Live { generation: NonZeroU64 },
         /// Destroyed by the platform; the last one was of generation `last`.
         Destroyed { last: NonZeroU64 },
     }

     impl SurfacePresence {
         /// The first surface.
         pub fn new() -> Self {
             Self::Live { generation: NonZeroU64::MIN }
         }
         /// The live surface's generation; `None` while destroyed.
         pub fn generation(self) -> Option<NonZeroU64> {
             match self {
                 Self::Live { generation } => Some(generation),
                 Self::Destroyed { .. } => None,
             }
         }
         /// Apply a platform create (`live`) or destroy; the event to deliver when it changed.
         pub fn apply(&mut self, live: bool) -> Option<ShellEvent> {
             match (*self, live) {
                 (Self::Destroyed { last }, true) => {
                     *self = Self::Live { generation: last.saturating_add(1) };
                     Some(ShellEvent::SurfaceCreated)
                 }
                 (Self::Live { generation }, false) => {
                     *self = Self::Destroyed { last: generation };
                     Some(ShellEvent::SurfaceDestroyed)
                 }
                 _ => None,
             }
         }
     }

     impl Default for SurfacePresence {
         fn default() -> Self {
             Self::new()
         }
     }
     ```
     `lib.rs` に `mod surface;` と `pub use surface::SurfacePresence;`、`pub use window::Drawable;` を足す。
  4. backend の `drawable()`：
     - virtual（`crates/arvane-virtual/src/window.rs`）：field `surface: TestSurface` を `surface: arvane_shell::SurfacePresence` にし、初期値 `SurfacePresence::new()`。`fn drawable(&self) -> Drawable { match self.surface.generation() { None => Drawable::NoSurface, Some(_) if <width or height is 0, from the size the window uses for physical_size> => Drawable::NoPixels, Some(generation) => Drawable::Yes { generation } } }`。`present_with` の `if self.surface.generation().is_none() || width == 0 || height == 0` はそのまま。`crates/arvane-virtual/src/backend.rs` の `fn surface_event(window: &mut VirtualWindow, live: bool) -> Option<ShellEvent>` の本体を `window.surface.apply(live)` にする。`crates/arvane-virtual/src/surface.rs` と `mod surface;` を消す。
     - winit（`port/winit/window.rs`）：`fn drawable(&self) -> Drawable` を `match &self.surface { Surface::Destroyed { .. } => Drawable::NoSurface, Surface::Live { generation, .. } => if <physical_size() has a 0 edge> { Drawable::NoPixels } else { Drawable::Yes { generation: *generation } } }` で書く。
     - embed（win32・x11）：`ChildSurface`（`port/embed/surface.rs`）を消す（ファイルと `port/embed.rs` の `mod surface;`・`use`）。win32 の `State` の field `surface: Cell<ChildSurface>` と x11 の `Window` の field `surface: ChildSurface` を消す。`drawable()` は `match <applied visibility (win32: self.0.shared.applied().visibility; x11: self.shared.applied().visibility)> { EmbedVisibility::Hidden => Drawable::Hidden, EmbedVisibility::Shown if <physical size has a 0 edge> => Drawable::NoPixels, EmbedVisibility::Shown => Drawable::Yes { generation: NonZeroU64::MIN } }`（embed の surface は作り直さないので generation は常に 1）。
     - win32 の `apply_visibility`：末尾の `let mut surface = self.surface.get(); if let Some(event) = surface.apply(visibility) { self.surface.set(surface); self.event(event); }` を、`Shared::set_visibility` が変化を返す（`-> bool`）ので `if <the set_visibility call returned true> { self.event(ShellEvent::Visibility(visibility == EmbedVisibility::Shown)); }` にする（今の 2 つの `self.shared.set_visibility(visibility);` の戻り値を変数に受ける）。
     - x11 の `visibility_events`：`if let Some(event) = window.set_surface(visibility) { events.push(event); }` を `if window.set_visibility(visibility) { events.push(ShellEvent::Visibility(visibility == EmbedVisibility::Shown)); }` にし、`Window::set_surface` を `pub(super) fn set_visibility(&mut self, visibility: EmbedVisibility) -> bool` に改名して本体を「`let changed = self.shared.applied().visibility != visibility; if changed { self.shared.update(self.width, self.height, self.scale, visibility); } changed`」にする。
  5. presenter（`crates/arvane/src/`）：
     - `presenter/frame.rs`：`Presented` に variant `/// Not drawn: the window cannot show a frame now (`Drawable` says why). The change that makes it drawable asks for a frame.` `NotDrawable(arvane_shell::Drawable),` を足す。`no_surface` と `no_pixels` を消し、`presenter.rs` の `pub(crate) use frame::{no_pixels, no_surface};` を消す。
     - `window.rs` の `Window::present`：先頭の 2 つの検査を次にする：
       ```rust
       let drawable = self.shell.drawable();
       if !matches!(drawable, arvane_shell::Drawable::Yes { .. }) {
           return Ok(Presented::NotDrawable(drawable));
       }
       ```
       `Window::surface_generation` を `pub fn drawable(&self) -> arvane_shell::Drawable { self.shell.drawable() }` に替える。`Driver::event` の `window.surface_generation().is_some() && presenter.prepare(window)` を `matches!(window.drawable(), arvane_shell::Drawable::Yes { .. }) && presenter.prepare(window)` にする。`ShellEvent::SurfaceDestroyed` で `presenter.surface_destroyed()` を呼ぶ箇所はそのまま（`Visibility(false)` では GPU の surface を落とさない）。
     - AR-22：`Windows` の `presenters: &mut BTreeMap<WindowId, Presenter>` は変えず、`Windows::open` で `request.content` が `WindowContent::Drawn` のときだけ presenter を作って `begin` する。`Windows::get` は presenter の無い window にも `Window` を返すため、`Window` の field を `presenter: Option<&'a mut Presenter>` にし、presenter を使う method は次のとおりにする：`present` は presenter が無ければ `Ok(Presented::NotDrawable(arvane_shell::Drawable::NoSurface))`；`release_caches`・`forget_frame_state`・`drop_external_image`・`drop_export` は何もしない；`gpu_adapter` は `None`；`take_export_notices` は空の `Vec`；`import_external_image`・`export_drawable` は `Err(Failure::new(FailureKind::Unavailable, "a foreign-content window has no renderer"))`。`Windows::get` は `let presenter = self.presenters.get_mut(&id);` と `let shell = self.shell.window(id)?;` で作る。`Windows::ids` は `self.shell.windows()` を返す（presenter の無い window も含める）。`Driver::close`・`Windows::close` の shutdown は presenter のある window だけ（今の形のまま）。`WindowContent` の path は `rg -n "enum WindowContent" crates/kivact-types/src`。
  6. engine（`crates/kivact-engine/src/host/`）：
     - `window.rs`：`enum Surface`・`enum Unavailable`・`fn surface_of` を消す。`HostWindow` の field `surface: Surface` を `/// Whether the window can show a frame, as the window last said.` `drawable: arvane_shell::Drawable` にし、初期値 `arvane_shell::Drawable::NoSurface`。
     - `events.rs` の `fn refresh_surface`：`let surface = super::window::surface_of(window);` を `let drawable = window.drawable();` に、`host_window.surface = surface;` を `host_window.drawable = drawable;` にする。`window_changes.rs` の `start_window` の `let surface = surface_of(&window);` と `.surface = surface;` も同じに直す。
     - `pump.rs` の `fn drawable`：`matches!(window.surface, Surface::Drawable { .. })` を `matches!(window.drawable, arvane_shell::Drawable::Yes { .. })` にする。
     - `events.rs` の `fn lifecycle_event`：`ShellEvent::SurfaceCreated => { .. }` の arm に並べて `ShellEvent::Visibility(shown) => { self.report_drawable(id, windows); if shown { if let Some(model) = self.app.borrow_mut().model_mut(id) { model.invalidate_paint(); } } }` を足す。`lifecycle_event` へ event を振り分ける match（`ShellEvent::SurfaceDestroyed | ShellEvent::SurfaceCreated` を含む arm。`rg -n "SurfaceCreated" crates/kivact-engine/src/host`）に `| ShellEvent::Visibility(_)` を足す。
     - `events.rs` の `fn redraw` の match に `Ok(arvane::Presented::NotDrawable(_)) => { <the body of the Dropped arm today> }` を足す（`drawn()`、`drawables.dropped()`、`Settled`）。`Dropped(_)` の arm は残す（描けない scene の失敗）。
     - AR-09（engine）：`pump.rs` の `fn present` の `if let Presented::Shown { renderer, raster, present, stats, .. } = &presented` の中で、`last_present = LastPresent::Shown { at_ns }` を書く箇所（`events.rs` の `redraw` の `Shown` の arm）を、`renderer == arvane::Renderer::Cpu` のときだけ `LastPresent::Shown { at_ns }` にし、GPU のときは `LastPresent::Settled` にする（`release_idle_frames` は `Shown` だけを解放の対象にするので、GPU の frame の後に idle の wake を予約しない）。`redraw` の arm で `renderer` を取り出す：`Ok(arvane::Presented::Shown { renderer, .. })`。
  7. `crates/arvane/tests/e2e/present.rs`：`Presented::Dropped(Failure { kind: FailureKind::Content, .. })` か `FailureKind::Unavailable` で「pixels が無い」「surface が無い」を確かめている箇所（`rg -n "Dropped" crates/arvane/tests/e2e`）を、`Presented::NotDrawable(Drawable::NoPixels)`・`Presented::NotDrawable(Drawable::NoSurface)` に直す。確かめる事実（描かないこと）は変えない。
- **消すもの**：`ChildSurface`（`port/embed/surface.rs`）、`TestSurface`（`arvane-virtual/src/surface.rs`）、`ShellWindow::surface_generation`、`no_surface`、`no_pixels`、engine の `Surface`・`Unavailable`・`surface_of`。embed は `SurfaceCreated`・`SurfaceDestroyed` を出さなくなる（variant は残る）。
- **状態の閉じ方**：`Drawable` と `SurfacePresence` は enum。`Window` の `presenter: Option<&mut Presenter>` は「foreign の window には presenter が無い」という一つの事実で、`Window` は `&mut self` の method を持つが対象の field はこれ 1 つ（対象外）。`HostWindow` は `Surface` を `Drawable` に替えるだけ。
- **試験**：足す試験は 1 つ。`crates/arvane-shell/src/surface.rs` の末尾の `#[cfg(test)] mod tests` に：
  ```rust
  #[test]
  fn a_surface_made_again_has_a_new_generation() {
      let mut surface = SurfacePresence::new();
      assert_eq!(surface.apply(false), Some(ShellEvent::SurfaceDestroyed));
      assert_eq!(surface.generation(), None);
      assert_eq!(surface.apply(true), Some(ShellEvent::SurfaceCreated));
      assert_eq!(surface.generation().map(|g| g.get()), Some(2));
      assert_eq!(surface.apply(true), None);
  }
  ```
  `EMBED` の `visibility` の試験（`dev/tests/kivact-public-api/tests/*/embed/visibility.rs`）は期待値を変えずに通ること（App から見た表示・非表示の通知は engine の `report_drawable` が同じく出す）。
- **確認（tier a）**：
  ```
  cargo fmt --all
  cargo check --workspace --all-targets --all-features --exclude kivact-docs
  cargo clippy -p arvane-shell -p arvane-native -p arvane-virtual -p arvane -p kivact-engine --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane-shell
  cargo xtask check --no-write --crate arvane-native
  cargo xtask check --no-write --crate arvane-virtual
  cargo xtask check --no-write --crate arvane
  cargo test -p arvane-shell
  ```
  E2E：`ARV`、`EMBED`、`FOREIGN`、`LIFE`、`GPU-DEMAND`。
- **完了条件**：`rg -n "surface_generation|ChildSurface|TestSurface|no_pixels|no_surface|surface_of" crates/` が 0 件。上のコマンドと E2E が通る。

## A09：layer の `content_version`（PERF-7）

- **wave / lane / 前提**：W5b / 鎖 / A08
- **所有**：`crates/arvane-protocol/src/composition.rs`、`crates/arvane-compose/src/{cache.rs,plan/damage.rs,plan/items.rs,plan/layers.rs}`、`crates/kivact-engine/src/{lower/scene_out.rs,lower/content_versions.rs(新),lower.rs,host/devtools.rs}`、`CompositeLayer {` を書く全試験（`rg -l "CompositeLayer \{" crates dev`）
- **目的**：AR-03 (1)：engine が layer ごとに渡す `content_version` で、compose は版の変わらない layer の raster の chunk の hash と差分を省く。debug build では hash で照合し、版の上げ忘れを panic で知らせる（7.7 Q4）。この unit の engine の実装は「毎 frame 新しい版」で、振る舞いは今と同じ（正しいが省略は起きない）。版を安定させるのは付属書 E（4.2 節の契約）。
- **手順**：
  1. `crates/arvane-protocol/src/composition.rs` の `pub struct CompositeLayer` の `items` の前に field を足す：
     ```rust
     /// The version of `items`: two frames of one window that give a layer of
     /// the same `key` the same `content_version` promise the same `items`
     /// (see the contract in `rework/A-arvane.md` 4.2). The compositor skips
     /// hashing the raster chunks of such a layer.
     pub content_version: u64,
     ```
  2. engine：新しいファイル `crates/kivact-engine/src/lower/content_versions.rs`（先頭 `//! Role: Content versions of composition layers.`）に次を置き、`crates/kivact-engine/src/lower.rs` に `pub(crate) mod content_versions;` を足す：
     ```rust
     /// A content version never given before in this process: the layer is
     /// treated as changed. (Annex E's E14 replaces its use in `into_layer`
     /// with versions kept while a layer's items stay the same.)
     pub(crate) fn fresh_content_version() -> u64 {
         static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
         NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
     }
     ```
     `fn into_layer(self, items: Vec<LayerItem>) -> CompositeLayer` の `CompositeLayer { .. }` に `content_version: super::content_versions::fresh_content_version(),` を足す。`crates/kivact-engine/src/host/devtools.rs` の `append_overlay_layer` の `CompositeLayer { .. }` に `content_version: crate::lower::content_versions::fresh_content_version(),` を足す（`lower` module の宣言の範囲で path が見えなければ、`lower.rs` の宣言に合わせる）。
  3. 試験の `CompositeLayer { .. }`：`rg -l "CompositeLayer \{" crates/arvane/tests crates/arvane-compose dev/tests` の各ファイルで、`CompositeLayer {` の式に `content_version: <n>,` を足す。n は試験の中で layer ごとに一意で、同じ試験が同じ内容を 2 frame 描く箇所は同じ n を使う（`crates/arvane/tests/e2e/support.rs` の helper `fn layer(..)`・`root_layer(..)` は引数を足さず、`content_version: next_version()` とし、support.rs に `fn next_version() -> u64` を上の engine の関数と同じ本体で置く）。
  4. `crates/arvane-compose/src/cache.rs`：`ChunkRecord` に field `/// The layer's content version the chunk was rastered at (content chunks only).` `pub(crate) version: Option<u64>,` を足す。`ChunkRecord { .. }` を作る全箇所（`rg -n "ChunkRecord \{" crates/arvane-compose/src`）に `version` を足す（手順 5 の `chunk` の中では引数の `version`、`carry_reused` の `..prev.clone()` はそのまま）。
  5. `crates/arvane-compose/src/plan/damage.rs` の `pub(super) fn chunk(&mut self, key: ChunkKey, scene: Cow<'a, RenderScene>, grid: Grid, to_win: &Matrix4, cache_it: bool)` に引数 `version: Option<u64>` を足し、`let cached = cache_it && self.cacheable();` の直後に次を入れる：
     ```rust
     if let Some(version) = version
         && let Some(prev) = self.cache.chunk(&key).filter(|_| cached)
         && prev.version == Some(version)
         && prev.scale == grid.scale
         && prev.origin.x == grid.origin.x
         && prev.origin.y == grid.origin.y
         && prev.size == grid.size
     {
         #[cfg(debug_assertions)]
         check_version(&scene, prev, grid, key, version)?;
         self.chunk_records.push((key, ChunkRecord { frame: self.frame, ..prev.clone() }));
         return Ok(());
     }
     ```
     関数の残り（hash・index・差分・task・record）は今のまま、作る `ChunkRecord` に `version` を入れる。次の関数を同じファイルに足す：
     ```rust
     /// Debug builds: a kept content version must mean the same commands.
     #[cfg(debug_assertions)]
     fn check_version(scene: &RenderScene, prev: &ChunkRecord, grid: Grid, key: ChunkKey, version: u64) -> Result<(), PlanError> {
         let bounds = scene_bounds(scene)?;
         let mut retain = Vec::new();
         let index: Vec<Indexed> = hash::scene_hashes(scene, &mut retain)
             .into_iter()
             .zip(bounds)
             .zip(scene.commands.iter())
             .map(|((h, b), command)| indexed(command, h, b.and_then(|r| clip_px(r, grid.origin, grid.scale, grid.size))))
             .collect();
         assert!(
             prev.index[..] == index[..],
             "content_version {version} of {key:?} was kept though the layer's commands changed",
         );
         Ok(())
     }
     ```
     （`scene_bounds`・`indexed`・`clip_px` は今の `chunk` が使っている関数。`Grid` が `Copy` でなければ `&Grid` で受ける。）
  6. `self.chunk(` の全呼出し（`rg -n "self\.chunk\(" crates/arvane-compose/src/plan`）に `version` を渡す：`items.rs` の `fn emit_raster`（`ChunkRole::Content(..)` の chunk）は `Some(cx.layer.content_version)`；`layers.rs` の `build_surface` の `chunk_shortcut` の枝（`ChunkRole::Content(0)`）は `Some(layer.content_version)`；それ以外（`Clip`、`LayerClip`、`Mask`、`Coverage`、`Fallback`、`Backdrop` の chunk）は `None`（その scene は layer の effects から作られ、版の対象外）。
- **消すもの**：なし。
- **状態の閉じ方**：`ChunkRecord` は `Option` の `version` と counter の `frame` を持つが、`&mut self` の method を持たず、作る時にだけ書く（対象外）。
- **試験**：足す試験は無い（engine の版はこの unit では毎 frame 新しいので、省略の経路は付属書 E の unit が通る。E の unit が足す試験は 4.2 節）。
- **確認（tier a）**：
  ```
  cargo fmt -p arvane-protocol -p arvane-compose -p kivact-engine -p arvane
  cargo check --workspace --all-targets --all-features --exclude kivact-docs
  cargo clippy -p arvane-protocol -p arvane-compose -p kivact-engine -p arvane --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane-compose
  cargo test -p arvane-compose --all-features
  ```
  E2E：`ARV`、`DRAW`、`OFFSCREEN`（debug build なので照合が走る）。
- **完了条件**：`rg -n "content_version" crates/arvane-protocol/src crates/arvane-compose/src crates/kivact-engine/src` が field・engine の 2 箇所・compose の呼出しを出す。上のコマンドと E2E が通る。

## A10：`Pixels` は premultiplied（AR-10）、CPU の export の記録は署名で（AR-14）

- **wave / lane / 前提**：W5b / 鎖 / A09
- **所有**：4.7 節の表の全ファイル。
- **目的**：5.8 節・AR-10：`kivact_types::Pixels` を premultiplied alpha の RGBA8 と定め、作るところはすべて premultiplied で作り、読むところは変換しない。straight が要るのは PNG に書く時だけ。AR-14（CPU）：CPU の export の記録を毎 frame の画素の hash でなく plan の署名で比べる。
- **手順**：
  1. `crates/kivact-types/src/color.rs` の `pub fn unpremultiply` の隣に次を足す：
     ```rust
     /// Convert straight-alpha RGBA8 pixels to premultiplied alpha in place
     /// (each color channel times alpha / 255, rounded).
     pub fn premultiply(rgba: &mut [u8]) {
         for px in rgba.chunks_exact_mut(4) {
             let a = u16::from(px[3]);
             for c in &mut px[..3] {
                 *c = ((u16::from(*c) * a + 127) / 255) as u8;
             }
         }
     }
     ```
  2. 4.7 節の表の各行を、その「変更」の列のとおりに直す。
  3. doc の文：`rg -n -i "straight.alpha|straight \(not premultiplied\)" crates/ dev/tests` の各行のうち、`Pixels`・`AssetImage`・`OffscreenImage`・`SnapshotResult::Pixels`・`DrawableFrame::Pixels`・`request_pixels`・`snapshot`・gpu-share の `read_back` と pool の read-back を述べる文を「premultiplied alpha」に直す。WGSL の `image()` と shader の色（`kivact-style/src/shader/value.rs`）の文は変えない（そちらは今も straight）。
- **消すもの**：`crates/arvane-compose/src/gpu/pass.rs` の `fn unpremultiply`、`cpu/compositor.rs` の `fn unpremultiplied` と `fn content_hash`、`ExportRecord.content`、`cpu/draw.rs` の `Straight`（`Premultiplied` に改名）、`cpu/graph/apply.rs` の `straight_to_premul`、`gpu/graph.rs` の `upload_premultiplied` の変換、`gpu/shader.rs` の `premultiplied_texture` の変換、gpu-share の 3 つの `read_back` の `unpremultiply`。
- **状態の閉じ方**：新しい状態の型は無い。
- **試験**：4.7 節の表の試験の行（期待値の表し方を premultiplied に直すもの。確かめる事実は変えない）。足す試験は無い。
- **確認（tier a）**：
  ```
  cargo fmt --all
  cargo check --workspace --all-targets --all-features --exclude kivact-docs
  cargo clippy --workspace --all-targets --all-features --exclude kivact-docs -- -D warnings
  cargo xtask check --no-write --crate kivact-types
  cargo xtask check --no-write --crate arvane-asset
  cargo xtask check --no-write --crate arvane-render
  cargo xtask check --no-write --crate arvane-compose
  cargo test -p kivact-types -p arvane-asset -p arvane-render -p arvane-compose --all-features
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p gpu-interop-producer --all-features -- --test-threads=1
  ```
  E2E：`ARV`、`ASSETS`、`OFFSCREEN`、`DRAW`、`GPU-SHARE`、`DEVTOOLS`（PNG の screenshot）。
- **完了条件**：`rg -n "unpremultiply" crates/ dev/` が `kivact-types/src/color.rs` の定義と、4.7 節の表の「PNG に書く前」の 3 箇所だけ。`rg -n "ImageAlphaType::Alpha\b" crates/` が 0 件。上のコマンドと E2E が通る。

## A11：compose の GPU を `GpuDevice`（Driver）と `GpuContext`（window）に分ける

- **wave / lane / 前提**：W6b / A1 / A03、A10
- **所有**：`crates/arvane-compose/src/{gpu.rs,gpu/*,gpu_unbuilt.rs,lib.rs}`、`crates/arvane/src/{presenter/gpu.rs,presenter/frame_policy.rs,test_hooks.rs}`
- **目的**：AR-02（compose の側）：instance・adapter・device・queue・pipeline・vello_hybrid の raster・shader の cache・外部の画像の import を `GpuDevice` に集め、window ごとの `GpuContext` は frame の状態（damage の cache、texture の pool、chunk、kept、export）だけを持ち、`Rc<GpuDevice>` を共有する。surface の configure は `GpuDevice` から作る `SurfaceConfigurer`（`Send`）で行う。arvane の presenter は、この unit では window ごとに 1 つの `GpuDevice` を作る（共有は A13）。
- **手順**：
  1. `crates/arvane-compose/src/gpu/device.rs`（新）：
     ```rust
     //! Role: A driver's GPU device: adapter, queue, pipelines, rasterizer, shaders and imported images.

     /// One GPU device shared by every window of a driver (PERF-8). Made on a
     /// worker thread (`GpuDevice::new` blocks), then used on the UI thread.
     pub struct GpuDevice {
         pub(crate) instance: wgpu::Instance,
         pub(crate) adapter: wgpu::Adapter,
         pub(crate) device: wgpu::Device,
         pub(crate) queue: wgpu::Queue,
         pub(crate) interop: Option<kivact_types::GpuAdapter>,
         pub(crate) failure: std::sync::Arc<DeviceLost>,
         pub(crate) pipelines: Pipelines,
         pub(crate) raster: std::cell::RefCell<arvane_render::GpuRaster>,
         pub(crate) shaders: std::cell::RefCell<Shaders>,
         pub(crate) external: std::cell::RefCell<External>,
     }

     /// What configuring a window's surface needs, sendable to a worker thread.
     #[derive(Clone)]
     pub struct SurfaceConfigurer {
         adapter: wgpu::Adapter,
         device: wgpu::Device,
     }
     ```
     `GpuDevice` の method（本体は今の `GpuContext` の同名のものを移し、`self.raster`・`self.shaders`・`self.external` は `.borrow_mut()` を付ける）：`pub fn new(env: &GpuEnv) -> Result<GpuDevice, RenderError>`（今の `GpuContext::new` と `with_backends`。per-window の field（`cache`・`pool`・`kept`・`chunks`・`shader_failures`・`exports`）を作る行を消す）、`pub fn describe(&self) -> String`、`pub fn interop_adapter(&self) -> Option<kivact_types::GpuAdapter>`、`pub fn failure(&self) -> Option<String>`、`pub(crate) fn check(&self) -> Result<(), RenderError>`、`pub fn release_caches(&self)`（`self.raster.borrow_mut().release_caches()`）、`pub fn import_external(&self, ..)`、`pub fn drop_external(&self, ..)`、`pub fn take_missed_external(&self, ..)`（今の `GpuContext` の同名の署名の `&mut self` を `&self` にする）、`pub fn surface_maker<T>(&self) -> ..`（今の `GpuContext::surface_maker`）、`pub fn configurer(&self) -> SurfaceConfigurer`（`adapter`・`device` の clone）。`impl SurfaceConfigurer { pub fn configure(&self, surface: &mut GpuSurface, width: u32, height: u32) -> Result<std::time::Duration, RenderError> }` は今の `GpuContext::configure` の本体（`self.device`・`self.adapter` を使う部分）を移したもの。戻り値の型は今の `configure` に合わせる。`context.rs` の private 関数（`adapter`、`open_device`、`gpu_adapter`、`backend_order`、`next_generation`）は `device.rs` へ移す。
  2. `crates/arvane-compose/src/gpu/state.rs` の `pub struct GpuContext` を次にする：
     ```rust
     /// One window's GPU frame state on the shared device.
     pub struct GpuContext {
         pub(crate) gpu: std::rc::Rc<GpuDevice>,
         pub(crate) cache: CacheIndex,
         pub(crate) pool: Pool,
         pub(crate) kept: HashMap<LayerKey, wgpu::Texture>,
         pub(crate) chunks: HashMap<ChunkKey, wgpu::Texture>,
         pub(crate) shader_failures: Vec<(u64, String)>,
         pub(crate) exports: Exports,
     }
     ```
     `impl GpuContext { pub fn new(gpu: std::rc::Rc<GpuDevice>) -> GpuContext }`（他の field は空）と `pub fn device(&self) -> &std::rc::Rc<GpuDevice>` を `context.rs` に置く。`pub fn release_caches(&mut self)` は今のまま（per-window の cache だけ）。
  3. compose の GPU の module の中の field の参照を移す（`impl GpuContext` の中の本体。手順 1 で `GpuDevice` に移した method は除く）：
     ```
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     G=crates/arvane-compose/src/gpu
     FILES="$G/exec.rs $G/draw.rs $G/mask.rs $G/graph.rs $G/filters.rs $G/shader.rs $G/surface.rs $G/offscreen.rs $G/export.rs $G/external.rs $G/shared_images.rs $G/pass.rs $G/context.rs"
     perl -pi -e 's/\bself\.(instance|adapter|device|queue|interop|failure|pipelines)\b/self.gpu.$1/g' $FILES
     perl -pi -e 's/&mut self\.raster\b/&mut self.gpu.raster.borrow_mut()/g; s/\bself\.shaders\./self.gpu.shaders.borrow_mut()./g; s/\bself\.external\b/self.gpu.external/g' $FILES
     ```
     手で直すもの：(a) `device.rs` の中は `self.gpu.` でなく `self.`（perl の対象外のファイル）；(b) `exec.rs` の `std::mem::take(&mut self.external)` は `std::mem::take(&mut *self.gpu.external.borrow_mut())`、戻す行は `*self.gpu.external.borrow_mut() = ..`；(c) `self.gpu.shaders.borrow_mut()` が参照を返して borrow が文を越える箇所は、`let mut shaders = self.gpu.shaders.borrow_mut();` を先に作って使う；(d) 一つの文で同じ `RefCell` を 2 回 `borrow_mut` しない（compile は通っても実行で panic する）。`rg -n "borrow_mut\(\).*borrow_mut\(\)" crates/arvane-compose/src/gpu` が同じ cell を 2 回含む行を出したら、その行を分ける。
  4. `crates/arvane-compose/src/gpu.rs`・`lib.rs`：`pub use device::{GpuDevice, SurfaceConfigurer};` を足す。`gpu_unbuilt.rs` に同じ名前の空の型を足す：`pub enum GpuDevice {}`（`new` は `Err(RenderError::NoAdapter("built without the gpu feature".into()))`、他は `match *self {}`）と `#[derive(Clone)] pub enum SurfaceConfigurer {}`（`configure` は `match *self {}`）。`GpuContext::new(gpu: Rc<GpuDevice>)` は `match *gpu {}`。
  5. arvane の presenter（`crates/arvane/src/presenter/gpu.rs`）を新しい形に合わせる（共有はしない。window ごとに 1 つの device のまま）：
     - `type Started = Result<(Box<GpuContext>, Duration), RenderError>;` を `type Started = Result<(GpuDevice, Duration), RenderError>;` にし、`start_device` の worker の本体を `GpuDevice::new(&env).map(|device| (device, since.elapsed()))` にする。
     - `poll_start` が `Sent(Ok((device, init)))` を受けたところで `Box::new(GpuContext::new(std::rc::Rc::new(device)))` を作り、今と同じ `Step::Waiting { context, init }` にする。
     - `type Configured = (Box<GpuContext>, Result<(GpuSurface, Duration), RenderError>);` を `type Configured = Result<(GpuSurface, Duration), RenderError>;` にし、`Step::Configuring` に `context: Box<GpuContext>` を足す（worker は context を持たない）。`fn configure(wake, context, init, surface, size)` の worker は `let configurer = context.device().configurer();` を thread に move し、`configurer.configure(&mut surface, width, height).map(|took| (surface, took))` を返す。`advance` の `Configuring` の `Sent(..)` は `Step::Configuring` の `context` を使う。
     - `context.describe()`・`context.interop_adapter()`・`context.failure()`・`context.import_external(..)`・`context.drop_external(..)`・`context.take_missed_external(..)`・`context.surface_maker()` の呼出しは `context.device().<同名>(..)` にする（`crates/arvane/src/presenter/*`）。`context.configure(..)` は手順 5 の 3 つ目のとおり。
     - `crates/arvane/src/test_hooks.rs` の `gpu_unavailable` は `arvane_compose::GpuDevice::new(..)` を呼ぶ。
- **消すもの**：`GpuContext::{instance, adapter, device, queue, interop, failure, pipelines, raster, shaders, external}` の field、`GpuContext::new(env)`・`with_backends`・`configure`・`describe`・`interop_adapter`・`failure`・`import_external`・`drop_external`・`take_missed_external`・`surface_maker`（`GpuDevice`・`SurfaceConfigurer` へ移る）。
- **状態の閉じ方**：`GpuDevice` は `RefCell` を 3 つ持ち、`&self` の method がそれを書く（6.1 節の対象）。宣言の直前に次の 1 行を置く：`// xtask-allow(state): raster, shaders and external are independent caches of one device, each borrowed alone; any combination of their contents is valid`。`GpuContext` は `bool`・`Option` を持たない。`SurfaceConfigurer` は clone だけ。
- **試験**：足す試験は無い。
- **確認（tier a）**：
  ```
  cargo fmt -p arvane-compose -p arvane
  cargo check -p arvane-compose --all-targets --all-features
  cargo check -p arvane-compose --all-targets --no-default-features
  cargo check -p arvane -p kivact-engine --all-targets --all-features
  cargo clippy -p arvane-compose -p arvane --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane-compose
  cargo test -p arvane-compose --all-features
  ```
  E2E：`ARV`（`--features test`。GPU の機械では 2.2 節の GPU の形でも）、`GPU-DEMAND`、`GPU-SHARE`。
- **完了条件**：`rg -n "pub(crate)? (instance|adapter|device|queue|pipelines|raster|shaders|external):" crates/arvane-compose/src/gpu/state.rs` が 0 件。上のコマンドと E2E が通る。

## A12：native の小さな整理、embed の注入入力、a11y の二度目の検証、lazy の static

- **wave / lane / 前提**：W6b / A2 / A08
- **所有**：`crates/arvane-native/src/{wake.rs(削除),lib.rs,mirror.rs,module.rs,native_input/windows.rs}`、`crates/arvane-native/src/port/win32/{dpi.rs,native.rs,window.rs,create.rs,proc.rs}`、`crates/arvane-native/src/port/x11/{poll.rs,thread.rs,notifier.rs,dnd.rs,window.rs}`、`crates/arvane-native/src/port/winit/{runtime.rs,window.rs,foreign/win32.rs}`、`crates/arvane-native/src/port/embed/session.rs`、`crates/arvane-native/Cargo.toml`、`crates/arvane-shell/src/{types.rs,window.rs,a11y/shared.rs}`、`crates/arvane-virtual/src/window.rs`
- **目的**：AR-05：embed の window に注入した入力を捨てずに届ける。AR-20（native）：`wake.rs`、Win32 の DPI の `GetProcAddress`、X11 の timerfd、Win32 の embed の `Environment` の二重の組立てを消す。AR-19：debug の DPI の message を feature `test` の後ろに置く。AR-21：native の lib doc の Linux の文を直す。AR-25：engine の出した semantics の木の検証を debug だけにする。DEC-13：`OnceLock` の static 7 つを `SharedDeferred` にする。
- **手順**：
  1. AR-20 wake：`crates/arvane-shell/src/types.rs` の `WakeReasons` に足す：
     ```rust
     /// The reasons as bits (stable within a process), for atomic storage.
     pub const fn bits(self) -> u8 {
         self.0
     }
     /// The reasons of `bits` (from [`WakeReasons::bits`]).
     pub const fn from_bits(bits: u8) -> Self {
         Self(bits)
     }
     ```
     `crate::wake::encode(x)` を `x.bits()` に、`crate::wake::decode(x)` を `WakeReasons::from_bits(x)` にする（`port/winit/runtime.rs` の 2 箇所、`port/win32/native.rs` の 2 箇所、`port/x11/notifier.rs` の 2 箇所）。`crates/arvane-native/src/wake.rs` と `lib.rs` の `mod wake;` を消す。
  2. AR-20 DPI：`crates/arvane-native/src/port/win32/dpi.rs` の `type Proc`・`type SetContext`・`type GetContext`・`type GetDpi`・`fn api` を消し、`windows_sys::Win32::UI::HiDpi::{SetThreadDpiAwarenessContext, GetWindowDpiAwarenessContext, GetDpiForWindow}` を直接呼ぶ：`Guard::for_parent(parent)` は `let context = unsafe { GetWindowDpiAwarenessContext(parent) }; let old = unsafe { SetThreadDpiAwarenessContext(context) };`、`struct Restore { old: DPI_AWARENESS_CONTEXT }`（`set` の field は消す）、`Drop` は `unsafe { SetThreadDpiAwarenessContext(restore.old) };`、`scale(hwnd)` は `let dpi = unsafe { GetDpiForWindow(hwnd) };` を今と同じく 0 なら 96 にして `/ 96.0`。`old` が null のときは `Guard(None)`（今の `NonZeroIsize::new(old)` の判定と同じ。`DPI_AWARENESS_CONTEXT` の型が pointer なら `is_null()`、整数なら `== 0`）。各 `unsafe` に `// SAFETY:` を書く（今の文を使う）。`Cargo.toml` の `windows-sys` の feature に `Win32_UI_HiDpi` があることを確かめる（ある）。
  3. AR-20 X11 の待ち：`crates/arvane-native/src/port/x11/poll.rs` の `pub(super) struct Waiter { timer: OwnedFd }`・`Waiter::new` を消し、`Waiter::wait` を free 関数 `pub(super) fn wait(x: &impl AsFd, wake: &OwnedFd, remaining_ns: Option<u64>) -> Result<(), ShellError>` にする：`[x, wake]` の 2 つの fd を `rustix::event::poll(&mut fds, timeout)` に渡し、`timeout` は `remaining_ns.map(|ns| rustix::event::Timespec { tv_sec: (ns / 1_000_000_000) as i64, tv_nsec: (ns % 1_000_000_000) as i64 })` の参照（`None` は無期限）。EINTR の再試行と `ERR|HUP|NVAL` の error は今のまま。timer の drain を消す。`crates/arvane-native/src/port/x11/thread.rs` の `let waiter = Waiter::new()?;`（`run`）を消し、`waiter.wait(..)`（`event_loop`）を `super::poll::wait(..)` にする。`rustix::event::poll` の timeout の型が `Option<&Timespec>` でなく `i32`（ms）の版なら、`remaining_ns` を ms に切り上げて渡す（`rg -n "pub fn poll" ~/.cargo/registry/src/*/rustix-1*/src/event/poll.rs` で確かめる）。`Cargo.toml` の `rustix` の feature から `time` を外す（`rg -n "rustix::time" crates/arvane-native/src` が 0 件なら）。
  4. AR-20 Win32 の embed の `Environment`：`crates/arvane-native/src/port/win32/window.rs` の `State::new` の `input: NativeInput::new(kivact_types::Environment { scale_factor: .., theme_hint: .., reduced_motion: .., pointer: .., locale: .., ..Default::default() }, waker.clone())` の `Environment { .. }` を、`pub(super) fn environment(state: &State)` と同じ値の組を作る関数 `fn environment_of(shared: &Shared, frame_period_ns: u64) -> kivact_types::Environment` の呼出しに替え、`environment(state)` も `environment_of(&state.shared, frame_period(state.native.hwnd()))` を返す形にする（`environment_of` は今の `environment` の本体から `state.` を `shared.` に、`frame_period(..)` を引数にしたもの。`State::new` では hwnd がまだ無いので `crate::FRAME_PERIOD_NS`（今 `frame_period` が失敗時に返す定数）を渡す）。
  5. AR-19 DPI の debug message：`crates/arvane-native/src/native_input/windows.rs` の `debug_dpi_message`・`debug_dpi_change` とその dispatch の arm（`_ if cfg!(debug_assertions) && message == debug_dpi_message() => { .. }`）の条件を `cfg!(debug_assertions)` から `#[cfg(feature = "test")]`（関数に属性、arm は `#[cfg(feature = "test")]` を付けた `_ if message == debug_dpi_message() =>`）にする。module の doc の「Debug builds only」を「With the feature `test` only」にする。`debug_dpi_message` の `OnceLock<u32>` は手順 7 で `SharedDeferred` にする。
  6. AR-05 embed の注入：
     - `crates/arvane-shell/src/window.rs` の `trait ShellWindow` に足す：
       ```rust
       /// Resolve an assistive-technology request against the window's
       /// accepted tree, as the platform's own request would be.
       fn request_accessibility_action(&self, node: u64, request: kivact_types::A11yRequest) -> Option<crate::events::ShellEvent>;
       ```
       実装：winit `self.accessibility.request_action(node, request)`；win32 `self.0.a11y.request_action(node, request)`；x11 `self.accessibility.tree.request_action(node, request)`；virtual `self.accessibility().request_action(node, request)`。winit の `runtime.rs` の `injected_events` の `InjectedInput::Accessibility { node, request } => native.window.accessibility.request_action(node, request)..` は `native.window.request_accessibility_action(node, request)..` にする。
     - `crates/arvane-native/src/port/embed/session.rs`：`EmbedLoop` の field `injected: Vec<RequestId>` を `injected: Vec<(Vec<InjectedInput>, RequestId)>` にし、`fn inject(&mut self, _: WindowId, inputs: Vec<InjectedInput>, request: RequestId) { self.injected.push((inputs, request)); }` にする。`Session` に field `/// The input normalizer of injected input (DevTools); the platform's own input has its own.` `injected_input: RefCell<(arvane_shell::WindowInput, arvane_shell::pipeline::ImeDriver)>` を足し（`Session::new` で `RefCell::new((WindowInput::new(), ImeDriver::default()))`）、`fn deliver_injected` を次の形にする：
       ```rust
       fn deliver_injected(&self, slot: &mut HandlerSlot, lp: &mut EmbedLoop<'_>) {
           while !lp.injected.is_empty() && !self.shared.stopped() {
               for (inputs, request) in std::mem::take(&mut lp.injected) {
                   for event in self.injected_events(lp, inputs) {
                       if !self.deliver_one(slot, lp, event) {
                           return;
                       }
                   }
                   if !self.deliver_one(slot, lp, ShellEvent::Injected { request }) {
                       return;
                   }
               }
           }
       }

       /// The events `inputs` make, normalized like platform input.
       fn injected_events(&self, lp: &mut EmbedLoop<'_>, inputs: Vec<InjectedInput>) -> Vec<ShellEvent> {
           let now = lp.window.now_ns();
           let pointer = lp.window.environment().pointer;
           let double_click = (pointer.double_click_ns, pointer.double_click_distance);
           let mut state = self.injected_input.borrow_mut();
           let (input, ime) = &mut *state;
           let mut events = Vec::new();
           for injected in inputs {
               match injected {
                   InjectedInput::Accessibility { node, request } => {
                       events.extend(lp.window.request_accessibility_action(node, request));
                   }
                   other => events.extend(input.normalize(other, now, double_click, ime)),
               }
           }
           events
       }

       /// Deliver one event to the running handler; `false` once it stopped.
       fn deliver_one(&self, slot: &mut HandlerSlot, lp: &mut EmbedLoop<'_>, event: ShellEvent) -> bool {
           let HandlerSlot::Running { handler, .. } = slot else {
               return false;
           };
           let result = handler.event(EventTarget::of(&event, WindowId::MAIN), event, lp);
           self.accept(slot, result);
           !self.shared.stopped()
       }
       ```
       `PointerSettings` の field 名（`double_click_ns`、`double_click_distance` の型）は `rg -n "pub struct PointerSettings" -A12 crates/kivact-types/src` で確かめ、`WindowInput::normalize` の引数 `(u64, (f64, f64))` に合わせる。
  7. DEC-13：次の 7 つの `std::sync::OnceLock<T>` の static を `kivact_types::cell::SharedDeferred<T>` にする（`static X: std::sync::OnceLock<T> = std::sync::OnceLock::new();` → `static X: kivact_types::cell::SharedDeferred<T> = kivact_types::cell::SharedDeferred::new();`。`get_or_init`・`get` の呼出しは同じ名前のまま。`set` を使う箇所があれば `get_or_init(|| value)` にする。その static に付いた `#[expect(clippy::disallowed_types ..)]`・`#[allow(..)]` を消す）：`crates/arvane-virtual/src/window.rs`（`LIVE`）、`crates/arvane-native/src/mirror.rs`（`DISPLAY`）、`crates/arvane-native/src/module.rs`（`RETAINED`）、`crates/arvane-native/src/native_input/windows.rs`（`ID`）、`crates/arvane-native/src/port/x11/dnd.rs`（`NAME`）、`crates/arvane-native/src/port/winit/foreign/win32.rs`（`REGISTERED`）。`Cargo.toml` の `kivact-types` は既に依存にある。
  8. AR-21：`crates/arvane-native/src/lib.rs` の「Mouse settings and reduced motion are not read from the desktop: the defaults (500 ms, 4 px, 3 lines, no reduced motion).」を「Mouse settings are not read from the desktop: the defaults (500 ms, 4 px, 3 lines). Reduced motion comes from the XDG settings portal (`port/linux_settings.rs`), watched while the App runs.」にする。
  9. AR-25：`crates/arvane-shell/src/a11y/shared.rs` の `fn replace` の `let tree = validation::canonicalize(tree)?;` を次にする（engine が唯一の生産者なので、release では検証しない。`canonicalize` が並びを正す働きも持つなら、その正規化だけは release でも行う：`validation.rs` の `canonicalize` が「検証」と「正規化（並べ替え・複製）」のどちらを返すかを読み、正規化を含むなら関数を `validate(tree) -> Result<(), AccessibilityError>` と `canonical(tree) -> SemanticsTree` に分けて、`validate` だけを `#[cfg(debug_assertions)]` にする）：
     ```rust
     #[cfg(debug_assertions)]
     validation::canonicalize(tree)?;
     ```
     `AcceptedTree::apply` は残す（AT の最初の木の要求と action の対象の確かめに要る）。
- **消すもの**：`crates/arvane-native/src/wake.rs`、`dpi.rs` の `api`・`Proc` などの関数型、`Waiter`、Win32 の `Environment` の二つ目の組立て、`EmbedLoop.injected` の `Vec<RequestId>` の形、7 つの `OnceLock`。
- **状態の閉じ方**：`Session` に `RefCell` を 1 つ足す。`Session` は既に `RefCell` を複数持つので T が印を付けている。その文面の末尾に `; injected_input normalizes DevTools input apart from the platform's input and pairs with no other field` を足す。
- **試験**：足す試験は 1 つ。`EMBED` の `contract.rs`（`dev/tests/kivact-public-api/tests/*/embed/contract.rs`）に次を足す（既存の試験の作り方（embed の App を開く helper）を使う。名前：`injected_input_reaches_an_embedded_app`）：embed の App に `button("Press")` と押した回数の `signal` を置き、DevTools の注入の経路（`kivact::test` の embed の screen の `click`、無ければ `HostRequest::Inject` を出す既存の helper）で button を押し、回数が 1 になることを確かめる。embed の screen に注入の helper が無い場合は、この試験を足さずに STOP して報告する。
- **確認（tier a）**：
  ```
  cargo fmt -p arvane-native -p arvane-shell -p arvane-virtual
  cargo check -p arvane-native -p arvane-shell -p arvane-virtual -p arvane -p kivact-engine --all-targets --all-features
  cargo clippy -p arvane-native -p arvane-shell -p arvane-virtual --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane-native
  cargo xtask check --no-write --crate arvane-shell
  cargo test -p arvane-shell
  ```
  E2E：`NATIVE-SMOKE`、`EMBED`、`ARV`、`DEVTOOLS`、`A11Y`。
- **完了条件**：`rg -n "OnceLock|GetProcAddress|timerfd|wake::(encode|decode)" crates/arvane-native/src crates/arvane-virtual/src` が 0 件。上のコマンドと E2E が通る。

## A13：GPU の device を Driver に一つ、`State` の三つの意味を分け、export の registry の写しを消す

- **wave / lane / 前提**：W7a / A1 / A11
- **所有**：`crates/arvane/src/{window.rs,presenter.rs,presenter/*,gpu_driver.rs(新),test_hooks.rs,lib.rs}`、`crates/arvane/tests/e2e/presenter.rs`、`crates/arvane-compose/src/gpu/{export.rs,device.rs,state.rs}`、`crates/arvane-compose/src/{gpu.rs,lib.rs,gpu_unbuilt.rs}`（手順 5・6 の 2 関数 `export_losses`・`probe_adapter` の追加と公開だけ）
- **目的**：AR-02（driver の側。DEC-08、12 章 EA-10）：GPU の device の起動・再起動・失敗の状態機械を `Presenter` から `Driver` へ移し、Driver（一つの App の event loop）に device を一つにする。process に一つではない：同じ process の二つの埋め込み editor は Driver が二つなので device も二つで、片方の close や device の喪失は他方に及ばない。window の presenter は自分の surface と `FrameHealth` だけを持つ。AR-23：`State::Off` の三つの意味を variant に分ける。AR-14：device が window より長く生きるので、`ExportRegistry.pools` の写しを消す。
- **手順**：
  1. `crates/arvane/src/gpu_driver.rs`（新）に `presenter/gpu.rs` の `Gpu`・`State`・`Background`・`Step`・`Worker` 系・`start_device`・`cpu_session`・`poll_start`・`ready_note`・`cpu` を移し、次の形にする：
     ```rust
     //! Role: The driver's one GPU device and its start, restart and failure.

     /// The device's life, once per driver (AR-23).
     enum State {
         /// The driver has not begun.
         New,
         /// CPU only: `GpuStart::Never` or `ARVANE_RENDERER=cpu`.
         CpuOnly,
         /// Starts when a window needs it (`GpuStart::OnDemand`).
         Deferred { slow: u8, waker: ShellWaker },
         /// Starting, or ready, on a worker.
         Background(Background),
         /// The device failed past its restarts: CPU for the session.
         Failed,
         /// The driver closed.
         ShutDown,
     }

     /// The device's background steps. Surfaces are the presenters'.
     enum Step {
         Starting(Worker<Started>),
         Ready { device: std::rc::Rc<GpuDevice>, init: Duration },
     }

     pub(crate) struct GpuDriver {
         env: GpuEnv,
         state: State,
     }
     ```
     `GpuDriver` の method：`new(env)`（`State::New`）、`begin(&mut self, start: RendererStart, waker: ShellWaker, notes: &mut Notes)`（`debug_assert!(matches!(self.state, State::New))`；`Cpu` → `CpuOnly`、`GpuAtLaunch` → spawn、`GpuOnDemand` → `Deferred`）、`waits`、`demand(cause, notes)`、`note_cpu_frame(..)`、`advance(&mut self, notes: &mut Notes) -> Option<std::rc::Rc<GpuDevice>>`（`Starting` を poll して `Ready` にし、`Ready` なら device を返す）、`device(&self) -> Option<&std::rc::Rc<GpuDevice>>`、`failed(&mut self, error: &RenderError, notes: &mut Notes) -> Fallback`（今の `Gpu::failed`。`Surface` は presenter が扱うので `Fallback::Surface` を返すだけ）、`fallback_to_cpu`、`shutdown(&mut self)`（`State::ShutDown`。`ShutDown` の後の呼出しは `debug_assert!` で止める）、`adapter() -> Option<GpuAdapter>`（`device()?.interop_adapter()`）。restart の予算 `RESTARTS` は driver に一つ。
  2. `Presenter`（`presenter/frame_policy.rs`）の `gpu: Gpu` を `surface: WindowGpu` にする。`presenter/gpu.rs` に残すのは window の側だけ：
     ```rust
     /// One window's part of the GPU renderer.
     pub(crate) enum WindowGpu {
         /// No device yet, or CPU only.
         Off,
         /// The device is ready; the window's surface is being configured.
         Configuring { context: Box<GpuContext>, worker: Worker<Configured> },
         /// The window draws on the device (`surface` is made on the next frame when `Pending`).
         Ready { context: Box<GpuContext>, surface: DeviceSurface },
     }
     ```
     今の `Gpu` の `present`・`present_pixels`・`read_back`・`drop_surface`・`configuring`・`ready`・`context` は `WindowGpu` の method にする。`advance` は `fn advance(&mut self, device: Option<&std::rc::Rc<GpuDevice>>, make: MakeSurface<'_>, physical: (u32, u32), notes: &mut Notes)`：`Off` で `device` が `Some` なら `GpuContext::new(Rc::clone(device))` を作り、surface を作って configure を始める（今の `Waiting` の処理）。device を失ったときは driver が全 presenter の `WindowGpu` を `Off` にする（`fn lose_device(&mut self)`：context と surface を捨てる）。
  3. `Driver`（`window.rs`）に field `gpu: GpuDriver` を足し、`Windows` に `gpu: &'a mut GpuDriver` を持たせる（`Windows::get` が返す `Window` にも `gpu: &'a mut GpuDriver` を渡す）。`Driver::start` で `self.gpu.begin(self.env.start, waker.clone(), &mut notes)`（notes は main window の `Notes`）を 1 回だけ呼ぶ。`Windows::open` は presenter の `begin` を呼ばない（device は driver のもの）。`Driver::event` の `GPU_STEP` の wake では、まず `self.gpu.advance(..)` で device を進め、その後、今のとおり各 presenter の `prepare` を呼ぶ（`prepare` は `self.gpu.device()` を受ける）。`demand_gpu(id)` は `self.gpu.demand(GpuStartCause::App, ..)`。device の失敗（presenter が `Fallback::Cpu` を受けたとき）は driver の `gpu.failed(..)` を呼び、全 presenter の `WindowGpu::lose_device()` を呼ぶ（device の再起動は driver の予算から 1 回）。
  4. renderer の通知：device の通知（`Start`、`GpuStarting`、`Gpu{..}`、device の失敗の `Cpu{..}`）は driver が `WindowId::MAIN` の `Notes` で 1 回だけ出す（device は一つなので）。surface の失敗の `Cpu{..}` と `ExternalFrameMissed`・`ShaderFailed`・`GpuLimit` は今のとおり window の `Notes` で。`crates/arvane/tests/e2e/presenter.rs` で window ごとの `Start` の数を数える試験があれば、その期待を「device の通知は main の window に 1 回」に直す前に STOP して報告する（期待値の意味が変わるため）。
  5. AR-14：`presenter/exports.rs` の `ExportRegistry` の field `pools: HashMap<DrawableId, u64>` を消す。`lost_all` は「device を失った」ときに、その window の `GpuContext.exports.pools`（compose の `Exports`）の sink と generation から `ExportNotice::Lost` を作る：compose に `pub fn export_losses(&self) -> Vec<ExportLoss>`（`GpuContext` の method。`exports.pools` を sink の順に `ExportLoss { sink, generation }` へ）を足し（`crates/arvane-compose/src/gpu/export.rs`。この 1 関数だけ compose を変える。所有に含める）、presenter は `lose_device` の前にそれを `notices` に積む。`ExportRegistry::made`・`dropped` を消す。
  6. `crates/arvane/src/test_hooks.rs` の `gpu_unavailable` を、device を作らず adapter だけを求める形にする：compose に `pub fn probe_adapter(env: &GpuEnv) -> Result<String, RenderError>`（`GpuDevice` の `adapter(..)` の選び方と同じで、device を開かず、adapter の名前を返す。`device.rs` に置く。所有に含める）を足し、`gpu_unavailable()` は `arvane_compose::probe_adapter(&GpuEnv::from_env(GpuPreference::default())).err().map(|e| e.to_string())`。`lose_gpu_device(window)` は driver の device の失敗を起こす（`window.gpu.failed(&RenderError::Backend("test: device lost".into()), ..)` と全 window の `lose_device`）。
- **消すもの**：`Presenter.gpu`（`Gpu`）、`State::Off`、presenter の `begin`、`ExportRegistry.pools`・`made`・`dropped`、window ごとの device の起動。
- **状態の閉じ方**：`State` は 6 variant の enum（AR-23）。`WindowGpu` は 3 variant の enum。`GpuDriver` は field 2 つ（`env`、`state`）で対象外。`Presenter` は `bool` を 1 つ（`gpu_limit_noted`）だけ持つ。
- **試験**：足す試験は 2 つ（`crates/arvane/tests/e2e/presenter.rs`。GPU を使えない機械では `arvane::gpu_unavailable()` の理由で skip する、この binary の他の GPU の試験と同じ形）：
  - `two_windows_of_one_driver_share_one_gpu_device`：`GpuStart::AtLaunch` の driver（`arvane::start_stepped`）で window を 2 つ開き、両方が GPU で描いたとき（`Presented::Shown { renderer: Renderer::Gpu, .. }`）、observer が受けた `RendererChange::Gpu { .. }` が 1 回であること（device は Driver に一つ）。
  - `two_drivers_have_a_device_each_and_closing_one_leaves_the_other_drawing`：同じ thread で `arvane::start_stepped` を 2 回（二つの App、埋め込み editor 二つと同じ形）、それぞれの observer が `RendererChange::Gpu { .. }` を 1 回ずつ受けること（device は 2 つ）。一つ目の driver を閉じた（`SteppedShell` を drop した）後も、二つ目の window が `Renderer::Gpu` で `Presented::Shown` を返すこと。
- **確認（tier a）**：
  ```
  cargo fmt -p arvane -p arvane-compose
  cargo check -p arvane -p arvane-compose -p kivact-engine --all-targets --all-features
  cargo clippy -p arvane -p arvane-compose --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane
  ```
  E2E：`ARV`（GPU の機械では 2.2 節の GPU の形でも）、`GPU-DEMAND`、`GPU-SHARE`、`EMBED`、`LIFE`（`windows`）。
- **完了条件**：`rg -n "State::Off|fn begin" crates/arvane/src/presenter` が 0 件。`GpuDevice::new` を呼ぶのは `gpu_driver.rs` の 1 箇所だけ。上のコマンドと E2E が通る。

## A14：embed の子の寿命を一つの状態機械 `Child` に（貸出し・停止・片付けを明示する）

- **wave / lane / 前提**：W7a / A2 / A12
- **所有**：`crates/arvane-native/src/port/embed/{child.rs,session.rs}`、`crates/arvane-native/src/port/win32/{state.rs,window.rs,proc.rs,create.rs}`、`crates/arvane-native/src/port/x11/{window.rs,thread.rs}`
- **目的**：AR-15（12 章 EA-04〜EA-06）：子が App のコードを動かしてよいか、handler が callback に貸し出されているか、停止した子の片付けが残っているかを、`ChildLifecycle`・`HandlerSlot`・`StartSlot` の三つから導くのをやめ、UI thread の一つの enum `Child` で表す。callback は handler を `HandlerSlot::Calling` で借り、RAII（unwind を含む）で戻す。停止は `Stopped { cause, cleanup }` で、handler の `close` と native の資源の解放はちょうど一度。cross-thread の `Shared` の `ChildStatus { Running, Stopped { cause: Stop, window: NativeWindow } }` は和型のまま残す（window の消滅は停止の一つの形で、`window_gone: bool` のような組にしない）。x11 の `Window` の `width`・`height`・`scale` の重複を消す。
- **手順**：
  1. `crates/arvane-native/src/port/embed/session.rs` の `enum HandlerSlot` と `enum ChildLifecycle` を次の 3 つの型に置き換える。win32 の `crates/arvane-native/src/port/win32/state.rs` の `pub(super) enum StartSlot` と `State.start: RefCell<StartSlot>` を消す（`MakeHandler` は `Session::new` の引数で `Child::Opening` に入る）：
     ```rust
     /// The embedded child's life on its UI thread (AR-15): whether App code
     /// may run, whether its handler is lent to a callback, and what is left
     /// to clean up once it stopped.
     pub(in crate::port) enum Child {
         /// The window is being made; the handler is made at start.
         Opening { make: MakeHandler },
         /// The App runs. Its handler is in its slot or lent to a callback.
         Live { handler: HandlerSlot, control: ShellControl },
         /// The App stopped (`cause`, as `Shared` records it); the handler is
         /// still to be closed, and the native resources released, once.
         Stopped { cause: ChildStop, cleanup: Cleanup },
         /// A callback panicked: the handler was forgotten (never dropped)
         /// and the native resources are left as the panic left them.
         Leaked,
     }

     /// Where a live child's handler is.
     pub(in crate::port) enum HandlerSlot {
         /// In the slot, between callbacks.
         Idle(Box<dyn ShellHandler>),
         /// Lent to the callback running now (`Lent` gives it back).
         Calling,
     }

     /// What a stopped child still has to release.
     pub(in crate::port) enum Cleanup {
         /// The handler's `close` and the native resources (exactly once).
         Pending(Option<Box<dyn ShellHandler>>),
         /// Released.
         Done,
     }
     ```
     `Cleanup::Pending(None)` は「start の前に停止した（handler は作られていない）が、native の資源はまだ」。
  2. 遷移は次の表だけ。各行を `impl Child`（`session.rs`）の method として書き、`Session` の外から `Child` を直接書かない：

     | method（`Child`） | 前 | 後 | 呼ぶもの |
     |---|---|---|---|
     | `start(&mut self) -> bool` | `Opening { make }` | `Live { handler: Idle(make()), control: Wait }`、`true` | `Session::start`（win32 の `State::start_once`、x11 の `run`）。他の状態では何もせず `false` |
     | `lend(&mut self) -> Option<Box<dyn ShellHandler>>` | `Live { handler: Idle(h), .. }` | `Live { handler: Calling, .. }`、`Some(h)` | `Session::callback`（`deliver`・`start`・`deliver_injected` の前）。他の状態では `None`（callback を走らせない） |
     | `give_back(&mut self, h, stop: Option<ChildStop>)` | `Live { handler: Calling, control }` | `stop` が `None` なら `Live { handler: Idle(h), control }`、`Some(cause)` なら `Stopped { cause, cleanup: Pending(Some(h)) }` | `Lent::drop`（unwind でないとき）。`stop` は `shared.stop()` の写し |
     | `leak(&mut self, h)` | `Live { handler: Calling, .. }` | `Leaked`、`std::mem::forget(h)` | `Lent::drop`（`std::thread::panicking()` のとき） |
     | `stop_idle(&mut self, cause)` | `Opening { .. }` | `Stopped { cause, cleanup: Pending(None) }` | `Session::stopped_outside_callback`（`Shared` が callback の外で停止した時：host の先行破棄・`WM_DESTROY`・X11 の thread の終わり） |
     | 同上 | `Live { handler: Idle(h), .. }` | `Stopped { cause, cleanup: Pending(Some(h)) }` | 同上 |
     | `set_control(&mut self, control)` | `Live { control: _, .. }` | `Live { control, .. }` | `Session::accept`（Ok の結果） |
     | `take_cleanup(&mut self) -> Option<Option<Box<dyn ShellHandler>>>` | `Stopped { cleanup: Pending(h), cause }` | `Stopped { cleanup: Done, cause }`、`Some(h)` | `Session::dispose` |
     | 同上 | `Opening { .. }` | `Stopped { cause: ChildStop::Closed, cleanup: Done }`、`Some(None)` | 同上 |
     | 同上 | `Live { handler: Idle(h), .. }` | `Stopped { cause: ChildStop::Closed, cleanup: Done }`、`Some(Some(h))` | 同上 |
     | 同上 | `Live { handler: Calling, .. }` | 変えない、`None`、`debug_assert!(false, "invariant: disposal is deferred while a callback runs")` | 同上（win32 の `in_callback` の guard と x11 の loop が callback 中の破棄を遅らせる） |
     | 同上 | `Stopped { cleanup: Done, .. }`・`Leaked` | 変えない、`None` | 同上（二度目の破棄） |

     述語：`fn accepts_callbacks(&self) -> bool`（`Live { handler: Idle(_), .. }`）、`fn opening(&self) -> bool`、`fn live(&self) -> bool`（`Live { .. }`）、`fn active(&self) -> bool`（`Opening | Live`。今の `active()` と同じ意味：App の仕事を始めてよい）、`fn needs_cleanup(&self) -> bool`（`Opening | Live | Stopped { cleanup: Pending(_), .. }`）。
  3. `Session`：field `state: RefCell<ChildLifecycle>` を `child: RefCell<Child>` にする。`fn callback(&self)` は `Child::lend` を呼び、`Some(h)` なら次の guard を返す（`None` なら callback を走らせない）：
     ```rust
     /// A handler lent to one callback; dropping it gives the handler back
     /// (or leaks it when the callback panicked), so unwinding restores the
     /// child too.
     struct Lent<'a> {
         session: &'a Session,
         handler: Option<Box<dyn ShellHandler>>,
     }

     impl Drop for Lent<'_> {
         fn drop(&mut self) {
             let Some(handler) = self.handler.take() else {
                 return;
             };
             let mut child = self.session.child.borrow_mut();
             if std::thread::panicking() {
                 child.leak(handler);
             } else {
                 child.give_back(handler, self.session.shared.stop());
             }
         }
     }
     ```
     `Lent` は `fn handler(&mut self) -> &mut dyn ShellHandler`（`self.handler.as_deref_mut().expect("invariant: lent until dropped")`）を持つ。`deliver`・`start`・`deliver_injected`（A12）・`accept` は今の `Callback` の代わりに `Lent` を使い、`child` の `RefCell` の borrow を callback の間に持たない（`lend` と `give_back` の中だけ）。`accept` は Err・Exit で今のとおり `shared.fail(..)` を呼び、`Child` は書かない（停止は `Lent::drop` の `give_back` が `shared.stop()` を見て書く）。Ok は `set_control`。
     `pub(in crate::port) fn stopped_outside_callback(&self)`：`shared.stop()` が `Some(cause)` なら `child.stop_idle(cause)`。win32 の `State::stop`、`proc.rs` の `WM_DESTROY`・`WM_NCDESTROY` の後、x11 の `thread.rs` の `UiThreadExit::drop` と `run` の終わりで、`shared.window_gone()`・`shared.request_close()` の直後に呼ぶ。
     `pub(in crate::port) fn dispose(&self, window: &mut dyn ShellWindow) -> bool`：`let Some(cleanup) = self.child.borrow_mut().take_cleanup() else { return false; };`、`Some(handler)` なら今の `close` と同じく `handler.close(&mut lp)` を呼ぶ（`lp` は `EmbedLoop`）、`true` を返す。`true` は「呼出し元が native の資源をこの 1 回だけ解放する」。今の `close` と `forget` は `dispose` と `Child::leak` に統合する（`Session::leak`（win32 `proc.rs` の panic の後）は `Child` が既に `Leaked` なら何もしない）。
  4. 呼出し元の判定を直す：
     - win32 `window.rs` の破棄（`fn dispose`。今 `if !self.session.active() { return; }` で始まる）：`if !self.session.dispose(&mut window) { return; }` の後に、今の native の解放（CPU の surface、UIA の disconnect、`events.clear()`）を続ける（`dispose` が `true` を返した 1 回だけ）。
     - win32 の `State::stop` と `proc.rs` の `active()` の判定（App の callback を始めてよいか）は `accepts_callbacks()` を使う。`live()`・`opening()` の今の使い方は同じ名前の述語のまま。
     - x11 の `thread.rs` の終わりの `session.close(window)` を `if session.dispose(window) { <the native teardown that follows today> }` にする。
  5. `crates/arvane-native/src/port/embed/child.rs` の `ChildStatus` と `Stop` は変えない（`Stop::Panicked { failure }` の `failure` は panic の前に記録された失敗という別の事実なので残す）。`Shared::stop() -> Option<ChildStop>` が無ければ、`ChildStatus::Stopped { cause, .. }` を `ChildStop` に写す関数として足す（`Handle::stop` が使っている写しを共用する）。
  6. x11 の `Window` の field `width`・`height`・`scale` を消し、読む箇所は `self.shared.applied()` の `width`・`height`・`scale` を使う。書く箇所（`Window::new`、`resized`、`thread.rs` の `window.scale = scale`）は `self.shared.update(width, height, scale, visibility)` だけにする（`visibility` は `self.shared.applied().visibility`）。
- **消すもの**：`HandlerSlot::{Unstarted, Running}`（旧）、`ChildLifecycle`、`StartSlot`、`Callback`（旧の guard）、`Session::{close, forget}`、x11 の `Window::{width, height, scale}`。
- **状態の閉じ方**：`Child`・`HandlerSlot`・`Cleanup` は enum。`Session` の `child: RefCell<Child>` は今の `state: RefCell<ChildLifecycle>` を置き換えるだけで、`Session` の対象の field の数は変わらない（T・A12 の印の文面の `state` を `child` に直す）。`Lent` は `Option` を 1 つ持つ guard（`&mut self` の method は `handler` だけで、`handler` は書かない）。
- **試験**：`session.rs` の末尾の `#[cfg(test)] mod tests` に、`Child` の遷移の単体試験を足す（TV-09）。`make` は close の回数を数える handler を作る：
  ```rust
  struct Counted(std::rc::Rc<std::cell::Cell<u32>>);
  impl ShellHandler for Counted {
      fn start(&mut self, _: &mut dyn ShellLoop, _: ShellWaker) -> Result<ShellControl, ShellError> { Ok(ShellControl::Wait) }
      fn event(&mut self, _: EventTarget, _: ShellEvent, _: &mut dyn ShellLoop) -> Result<ShellControl, ShellError> { Ok(ShellControl::Wait) }
      fn close(&mut self, _: &mut dyn ShellLoop) { self.0.set(self.0.get() + 1); }
  }
  ```
  試験は handler を `take_cleanup` が返した数（= close される回数）で数える：
  - `a_child_stopped_before_start_cleans_up_natively_once`：`Opening` → `stop_idle(Closed)` → `take_cleanup()` が `Some(None)`、二度目は `None`。
  - `an_error_in_a_callback_keeps_the_handler_for_one_close`：`start` → `lend` → `give_back(h, Some(Failed(..)))` → `accepts_callbacks()` が false、`needs_cleanup()` が true、`take_cleanup()` が `Some(Some(_))`、二度目は `None`。
  - `exit_from_a_callback_stops_and_closes_once`：`give_back(h, Some(ChildStop::Closed))` の後の `take_cleanup` が 1 回だけ handler を返す。
  - `a_host_side_close_of_an_idle_child_closes_once`：`start` → `stop_idle(Closed)` → `take_cleanup` が handler を 1 回。
  - `a_panic_in_a_callback_leaks_the_handler`：`lend` → `leak(h)` → `take_cleanup()` が `None`、`needs_cleanup()` が false。
  - `no_callback_starts_after_the_window_is_gone`：`start` → `stop_idle(Closed)` → `lend()` が `None`。
  E2E：`EMBED` の `panic_boundary`・`contract`・`visibility`・`windows`・`x11` が期待値を変えずに通る。
- **確認（tier a）**：
  ```
  cargo fmt -p arvane-native
  cargo check -p arvane-native --all-targets --all-features
  cargo clippy -p arvane-native --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane-native
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p arvane-native --all-features --lib -- port::embed::session
  ```
  E2E：`EMBED`、`NATIVE-SMOKE`。Linux（WSLg）では `EMBED` の `x11` も（作業指示書 0.6 節の WSL の確認）。
- **完了条件**：`rg -n "ChildLifecycle|StartSlot|Unstarted" crates/arvane-native/src` が 0 件。上の 6 つの単体試験が実行され（filter が 6 件を実行する）通る。上のコマンドと E2E が通る。

## A15：CPU の compositor のメモリ（target の pool、filter の借用、GPU への切替えでの解放）

- **wave / lane / 前提**：W7b / A1 / A13
- **所有**：`crates/arvane-compose/src/cpu/{compositor.rs,filters.rs,draw.rs}`、`crates/arvane/src/presenter/frame_policy.rs`
- **目的**：AR-09：一時の target を大きさで再利用する（合計 64 MiB まで。idle の解放（MEM-8）で全部手放す。12 章 EA-17）、`Op::Filter` が入力の pixmap を clone しない、GPU が最初の frame を描いたら CPU の frame の状態を捨てる。root の chunk を `output` に直接描く最適化は行わない（12 章 EA-07：chunk の記録と画素の所有が食い違うため）。
- **手順**：
  1. `CpuCompositor` に field `/// Temporary targets kept for reuse by size, at most POOL_BYTES in all (EA-17).` `pool: Vec<Pixmap>` を足す。`fn make_targets` の `_ => Pixmap::new(spec.size.0, spec.size.1)` を `_ => take_target(pool, spec.size)` に、`run` の終わりで target 0 と kept 以外の target を `give_target(pool, px)` に返す：
     ```rust
     /// The most bytes of temporary targets kept between frames (EA-17).
     const POOL_BYTES: usize = 64 * 1024 * 1024;

     fn take_target(pool: &mut Vec<Pixmap>, size: (u32, u32)) -> Pixmap {
         match pool.iter().position(|px| px.size() == size) {
             Some(at) => {
                 let mut px = pool.swap_remove(at);
                 px.clear(None);
                 px
             }
             None => Pixmap::new(size.0, size.1),
         }
     }

     /// Keep `px` for reuse while the pool stays within `POOL_BYTES`; a target
     /// that does not fit is freed (a large window's targets are not kept).
     fn give_target(pool: &mut Vec<Pixmap>, px: Pixmap) {
         let kept: usize = pool.iter().map(|p| p.data.len()).sum();
         if kept + px.data.len() <= POOL_BYTES {
             pool.push(px);
         }
     }
     ```
     `invalidate`（`forget_frame_state` が呼ぶ idle の解放、MEM-8）と `release_caches`（memory の不足）で `self.pool = Vec::new();`（容量ごと手放す）。
  2. `Op::Filter`：`exec` の `let source = targets[*input as usize].clone(); targets[*output as usize] = filters::apply(filter, &source, ..)?;` を、`input != output` のときは `split_at_mut` で `&targets[input]` を借り、出力を pool の target に書く形にする：`filters::apply` の署名を `pub(crate) fn apply(filter: &LayerFilter, input: &Pixmap, out: &mut Pixmap, px_per_unit: f64, origin: Point) -> Result<(), RenderError>` にし、各 arm（`gaussian`、`drop_shadow`、`color_matrix`、`graph::apply`、`Shader` の写し）は戻り値の pixmap を `*out = ..` に入れる（内部の関数の割当ては変えない）。`input == output` のときだけ今のとおり clone する。
  3. `crates/arvane/src/presenter/frame_policy.rs` の `draw_native` の GPU の枝（`Some(Ok((presentation, present, report)))`）で、`window.surface().release_cpu_surface();` の直後に `self.cpu.invalidate();` を足す（GPU が描いた後に CPU の output・chunk を持たない）。
- **消すもの**：`Op::Filter` の入力の clone（`input != output` のとき）。
- **状態の閉じ方**：`CpuCompositor` は `Vec` を 1 つ足すだけ（対象の field は増えない）。
- **試験**：足す試験は 2 つ（TV-09。`crates/arvane-compose/src/cpu/compositor.rs` の末尾の `#[cfg(test)] mod tests`）：
  - `the_target_pool_keeps_at_most_64_mib`：`give_target` に 4096 × 2048（32 MiB）の `Pixmap` を 3 つ渡すと pool に 2 つだけ残り、`take_target` で同じ大きさを 2 回取ると両方が pool から出る（3 回目は新しく作る）。
  - `forget_frame_state_releases_every_pooled_target`：`CpuCompositor::frame` を 1 回描いた後（一時の target を使う composition：effect を持つ子の layer）、`invalidate()` の後に pool が空。
- **確認（tier a）**：
  ```
  cargo fmt -p arvane-compose -p arvane
  cargo check -p arvane-compose -p arvane --all-targets --all-features
  cargo clippy -p arvane-compose -p arvane --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane-compose
  cargo test -p arvane-compose --all-features
  ```
  E2E：`ARV`、`DRAW`、`OFFSCREEN`、`GPU-DEMAND`。
- **完了条件**：`rg -n "targets\[\*input as usize\]\.clone\(\)" crates/arvane-compose/src` が `input == output` の枝の 1 件だけ。`cargo test -p arvane-compose --all-features --lib -- cpu::compositor` が上の 2 つを含めて 1 件以上を実行し通る。上のコマンドと E2E が通る。

## A16：plan の hash を語の単位に、inline の選択を一括に、kivact-gpu-share の feature と module

- **wave / lane / 前提**：W8 / A2 / A09、A11、E14（W6b）
- **所有**：`crates/arvane-compose/src/plan/{hash.rs,builder.rs,layers.rs}`（`Fnv` を使うのは 2b8df5ac で `hash.rs`・`layers.rs` だけ）、`crates/arvane-compose/Cargo.toml`、`crates/arvane-compose/src/gpu/device.rs`（import の path）、`crates/kivact-gpu-share/`、`dev/tests/gpu-interop-producer/src/{vulkan.rs,d3d12.rs}`
- **目的**：AR-03 (2)(3)：byte ごとの FNV を 8 byte の語ごとの mix にし、target が多すぎるときの再計画を一括の選択にする。AR-18：adapter の識別を feature の無い `adapter` module に移し、compose の `gpu` は import・export を compile しない。device を開く 3 つの module と `*_contract`・`vulkan_raw` を統合する。
- **手順**：
  1. `crates/arvane-compose/src/plan/hash.rs`：`pub(crate) struct Fnv(u64)` を `pub(crate) struct WordHash(u64)` にし、定数と method の本体を次にする（他の method（`tag`、`len`、`f64`、`point` など）は `u64` を経由しているので本体を変えない）：
     ```rust
     const SEED: u64 = 0x243f_6a88_85a3_08d3;
     const K: u64 = 0x9e37_79b9_7f4a_7c15;

     impl WordHash {
         pub(crate) fn new() -> Self {
             Self(SEED)
         }
         pub(crate) fn u64(&mut self, word: u64) {
             let x = (self.0 ^ word).wrapping_mul(K);
             self.0 = x ^ (x >> 32);
         }
         pub(crate) fn bytes(&mut self, bytes: &[u8]) {
             let mut chunks = bytes.chunks_exact(8);
             for chunk in &mut chunks {
                 self.u64(u64::from_le_bytes(chunk.try_into().expect("invariant: chunks_exact gives 8 bytes")));
             }
             let rest = chunks.remainder();
             let mut last = [0u8; 8];
             last[..rest.len()].copy_from_slice(rest);
             self.u64(u64::from_le_bytes(last) ^ ((bytes.len() as u64) << 56));
         }
         pub(crate) fn finish(self) -> u64 {
             let mut x = self.0;
             x ^= x >> 33;
             x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
             x ^= x >> 33;
             x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
             x ^ (x >> 33)
         }
     }
     ```
     今の `Fnv` に `u8` 単位の method（`tag(u8)`）が `bytes(&[t])` を呼んでいれば、`self.u64(u64::from(t))` にする。改名：`rg -l '\bFnv\b' crates/arvane-compose/src | xargs perl -pi -e 's/\bFnv\b/WordHash/g'`。`OFFSET`・`PRIME` の定数を消す。
  2. `crates/arvane-compose/src/plan/builder.rs` の `pub(crate) fn build` の loop を次にする（候補の順は今と同じ「index の大きい順」）：
     ```rust
     validate::check(composition, scale)?;
     let mut plan = Builder::new(composition, cache, size, scale, gpu, HashSet::new()).run()?;
     if plan.targets.len() <= INLINE_TARGET_LIMIT {
         return Ok(plan);
     }
     let candidates: Vec<u32> = (1..composition.layers.len() as u32)
         .rev()
         .filter(|i| inline_candidate(composition, *i))
         .collect();
     let mut taken = 0;
     while plan.targets.len() > INLINE_TARGET_LIMIT && taken < candidates.len() {
         // Each inlined layer removes at least its own target: inline as many
         // as the targets over the limit at once, then plan again.
         taken = (taken + plan.targets.len() - INLINE_TARGET_LIMIT).min(candidates.len());
         let inline: HashSet<u32> = candidates[..taken].iter().copied().collect();
         plan = Builder::new(composition, cache, size, scale, gpu, inline).run()?;
     }
     Ok(plan)
     ```
  3. AR-18（kivact-gpu-share）：
     - `src/adapter.rs`（新、feature `adapter`。import・export の pool を含まない）：`device.rs` の `pub fn identity(adapter: &::wgpu::Adapter) -> Option<AdapterIds>` を移す。`import::dx12::identity` と `import::vulkan::identity` が使う hal の読み取り（`as_hal::<Dx12>` → `GetDesc2` → `luid_bytes`、`as_hal::<Vulkan>` → `vulkan_raw::identity`）を、`adapter.rs` の private 関数 `fn dx12(adapter) -> Option<AdapterIds>` と `fn vulkan(adapter) -> Option<AdapterIds>` に移し、`import/dx12.rs`・`import/vulkan.rs` の `identity` を消して `crate::adapter::identity` を使う。`lib.rs` に `#[cfg(all(feature = "adapter", any(windows, target_os = "linux")))] pub mod adapter;` を足し、`Cargo.toml` に feature `adapter = ["dep:wgpu"]` を足す（`import` と `wgpu` は `adapter` を含める：`import = ["adapter", "dep:wgpu", "dep:pollster"]`、`wgpu = ["adapter", "export", "dep:wgpu", "dep:pollster"]`）。`ash` と `windows` は、`adapter` だけのときに要る範囲（Vulkan の identity は `ash` を、Direct3D 12 の LUID は `windows` を使う）に留める（両方とも今 optional でないので Cargo.toml の依存は変えない）。
     - device を開く 3 つ：`src/wgpu_device.rs`（`open`）と `src/wgpu/device.rs`（`request_device`）と `src/device.rs`（`request_device`・`Opened`）を `src/device.rs` 一つにまとめる：`pub fn request_device(adapter: &::wgpu::Adapter, desc) -> Result<Opened, Error>`（今の `device.rs`）、`pub fn request_device_for(instance: &::wgpu::Instance, adapter: &GpuAdapter, desc) -> Result<(::wgpu::Adapter, ::wgpu::Device, ::wgpu::Queue), Error>`（今の `wgpu/device.rs` の `request_device`。`wgpu::request_device` の公開の path は `pub use crate::device::request_device_for as request_device;` を `wgpu.rs` に置く代わりに、呼出し元（`dev/tests`・docs）の `kivact_gpu_share::wgpu::request_device` を `kivact_gpu_share::device::request_device_for` に直す。`rg -n "wgpu::request_device" crates dev docs`）、`pub(crate) fn open_vulkan(..)`（今の `wgpu_device::open`）。`lib.rs` の `mod wgpu_device;` と `wgpu.rs` の `mod device;` を消す。`device` module の cfg は `#[cfg(any(feature = "import", feature = "wgpu"))]`。
     - `vulkan_raw.rs` を `vulkan/raw.rs` に統合し（`vulkan/raw.rs` は今 4 行の再公開）、`vulkan_contract.rs` の中身を `vulkan.rs` に、`d3d12_contract.rs` の中身を `d3d12.rs` に移す。`vulkan.rs`・`d3d12.rs` の module の cfg は、移した contract の項目（`API_VERSION`、`DEVICE_EXTENSIONS`、`HAND_OFF_LAYOUT`、`FORMAT`、`usage`、`HAND_OFF_STATE`、`resource_flags`、`luid_bytes`）が今の `*_contract` と同じ条件（feature なし、OS だけ）で見えるよう、`vulkan.rs` を `#[cfg(any(windows, target_os = "linux"))] pub mod vulkan;` にし、中の pool・read_back などの子 module に `#[cfg(feature = "export")]` を付ける（`d3d12` も同じ）。公開の path は `kivact_gpu_share::vulkan::{API_VERSION, DEVICE_EXTENSIONS, ..}`・`kivact_gpu_share::d3d12::{HAND_OFF_STATE, ..}` で、`vulkan_contract::`・`d3d12_contract::` を消す。crate 内の `crate::vulkan_contract::`・`crate::d3d12_contract::`・`crate::vulkan_raw::` の参照を `crate::vulkan::`・`crate::d3d12::`・`crate::vulkan::raw::` に直す（`rg -l "vulkan_contract|d3d12_contract|vulkan_raw" crates/kivact-gpu-share/src | xargs perl -pi -e 's/\bvulkan_contract::/vulkan::/g; s/\bd3d12_contract::/d3d12::/g; s/\bvulkan_raw::/vulkan::raw::/g'` の後、`mod` 宣言を手で直す）。
     - `crates/arvane-compose/Cargo.toml`：`kivact-gpu-share = { .., features = ["import", "wgpu"], optional = true }` を `features = ["adapter"]` にし、`gpu-interop = ["gpu"]` を `gpu-interop = ["gpu", "kivact-gpu-share/import", "kivact-gpu-share/wgpu"]` にする。`crates/arvane-compose/src/gpu/device.rs` の `kivact_gpu_share::device::identity(` を `kivact_gpu_share::adapter::identity(` にする（`gpu-interop` の枝の `kivact_gpu_share::device::request_device` はそのまま）。
     - `dev/tests/gpu-interop-producer/src/{vulkan.rs,d3d12.rs}` は `kivact_gpu_share::vulkan::{..}`・`d3d12::{..}` を使っており path は変わらない。compile が失敗した場合だけ path を直す。
- **消すもの**：`Fnv`、`OFFSET`・`PRIME`、build の 1 層ずつの再計画の loop、`kivact_gpu_share::{vulkan_contract, d3d12_contract, vulkan_raw, wgpu_device}`、`wgpu/device.rs`、`import::{dx12, vulkan}::identity`。
- **状態の閉じ方**：新しい状態の型は無い（`WordHash` は値）。
- **試験**：足す試験は無い。`crates/arvane-compose/src/plan/hash.rs` に hash の値そのものを固定した単体試験があれば、それは値の厳密な比較なので消し（名前を報告する）、「同じ入力は同じ hash、1 byte 違えば違う hash」の試験 `equal_input_hashes_equal_and_one_byte_differs` に置き換える。
- **確認（tier a）**：
  ```
  cargo fmt -p arvane-compose -p kivact-gpu-share
  cargo check -p kivact-gpu-share --all-targets --no-default-features
  cargo check -p kivact-gpu-share --all-targets --features adapter
  cargo check -p kivact-gpu-share --all-targets --all-features
  cargo check -p arvane-compose --all-targets --features gpu
  cargo check -p arvane-compose --all-targets --all-features
  cargo clippy -p arvane-compose -p kivact-gpu-share --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane-compose
  cargo xtask check --no-write --crate kivact-gpu-share
  cargo test -p arvane-compose --all-features
  cargo tree -p arvane-compose -e features --features gpu -i ash
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p gpu-interop-producer --all-features -- --test-threads=1
  ```
  `cargo tree .. -i ash` は Windows・Linux では `adapter` の Vulkan の識別のために ash を出す（ash は残る。pool・import は compile されない）。E2E：`ARV`、`DRAW`、`GPU-SHARE`、`GPU-DEMAND`。
- **完了条件**：`rg -n "\bFnv\b|vulkan_contract|d3d12_contract|vulkan_raw|wgpu_device" crates/ dev/` が 0 件。上のコマンドと E2E が通る。

## 4. 他の付属書との接点

### 4.1 本付属書が作り、他が使う名前

| 名前 | unit | 使う付属書 |
|---|---|---|
| `arvane_protocol::CompositeLayer::content_version: u64` | A09 | E（版の実装。4.2 節） |
| `arvane::{DriverOptions, run, start_stepped, embed_driver, observer, Observer}` | A05 | C（launch・embed の後続の変更があれば） |
| `kivact_types::gpu::GpuStart::Never` | A05 | C（`kivact::platform` への移動は C11。名前と意味は本付属書） |
| `arvane_shell::{Drawable, ImeRequest, WindowSurface, FrameSink, SurfacePresence}`、`ShellEvent::Visibility(bool)`、`ShellWindow::{set_ime, foreign_child, drawable, surface, request_accessibility_action}` | A07・A08・A12 | E（engine の host） |
| `arvane::Presented::NotDrawable(Drawable)` | A08 | E |
| `HostRequest::RequestWindowSize { window, size }`、`DeferredRequest::RequestSize(Size)` | A07 | C（`request_window_size`・foreign window の `request_size`）、E |
| `arvane_text::{TextEngine::resolve_family, FamilyChoice, FamilyResolution, FontLookup, FamilyQuery, FontFace, start_system_font_scan}` | A02 | E（`lower/assets.rs`、launch・embed の始め（A06）） |
| `arvane_render::{CpuFrameRaster, GpuFrameRaster}` | A01・A03 | 本付属書の中だけ |
| `impl From<arvane_render::RenderError> for kivact_types::Failure` | A06 | E（offscreen） |
| `kivact_types::color::premultiply` | A10 | S（kivact-types の所有者。W5b の鎖の間はこの unit が `color.rs` を持つ） |
| `kivact_types::Pixels` は premultiplied | A10 | C（kivact-offscreen の doc）、K（plot の heatmap は A10 が直す）、G（docs の文） |

### 4.2 `content_version` の契約（PERF-7、AR-03）

- **意味**：`CompositeLayer.content_version` は、その layer の `items`（raster の scene の命令、子の layer の参照と clip、外部の画像・shader の item、drawable の item）の版である。同じ window の連続する 2 frame で、同じ `key` の layer に同じ `content_version` を渡すことは、`items` が等しいことの約束である。`transform`・`bounds`・`plane`・`backface`・`grid`・`effects` は版の対象ではなく、compose が今のとおり比べる。
- **engine が上げる時（付属書 E の unit が実装する）**：layer の `items` を作る node のどれかが lower し直されて出力が変わり得るとき、子の layer の組（順・index・clip）が変わったとき、外部の frame・shader の item・drawable の出力が変わったとき。迷う場合は上げる（上げすぎは正しさを損なわず、hash の省略が減るだけ）。値は window ごとに単調に増える `u64` の counter から取り、一度使った値を同じ window の同じ `key` の別の内容に使わない。A09 の `fresh_content_version()`（`lower/content_versions.rs`。process で一意の値を毎回）は、この契約を常に満たす最小の実装で、E14 がこれを「変わらない layer は前の値を保つ」実装に替える。DevTools の overlay の layer は毎 frame 新しい値のままでよい。
- **compose が仮定してよいこと**：`(key, content_version)` が前の frame の chunk の記録と等しく、chunk の grid（scale・origin・size）が等しければ、その layer の `ChunkRole::Content(_)` の chunk の命令は等しい。その chunk は hash も差分もせず、記録を引き継ぎ、raster しない。clip・mask・coverage・fallback・backdrop の chunk（effects から作る scene）は版を使わず今のとおり hash する。
- **debug の照合**：`#[cfg(debug_assertions)]` では、版で省いた chunk も hash し、前の記録の index と比べ、違えば `content_version {v} of {key:?} was kept though the layer's commands changed` で panic する（`kivact::test` と debug の App で上げ忘れが見つかる）。release は照合しない。
- **E の unit（E14、W6b）**：`into_layer` の `fresh_content_version()` を layer の版の stamp に置き換え、DevTools の overlay（`append_overlay_layer`）は `fresh_content_version()` のまま（INC-04）。E14 が足す試験は 2 つ（engine の試験 `crates/kivact-engine/tests/content_version.rs`、`required-features = ["testing"]`。`kivact::test` から composition を見られないため）：`an_unchanged_layer_keeps_its_content_version_and_a_changed_one_bumps_it` と `a_changed_layer_bumps_its_version_and_a_sibling_layer_keeps_its_own`。二つは debug（照合あり）と release（照合なし）の両方で走らせ、照合の有無で版が変わらないことも確かめる（付属書 E の E14）。

### 4.3 facade の削除の対応表（A06。engine と試験の import の正本）

| 旧 | 新 |
|---|---|
| `arvane::Engine` | `arvane_text::TextEngine`（`text_metrics`→`measure`、`font_families`→`family_names`、`has_font_family`→`has_family`、`resolve_font_family`→`resolve_family`、`with_fonts_only`→`isolated`。`measure_text` は消える） |
| `arvane::asset::{AnimationInfo, AssetImage, AssetStore, AssetErrorKind, ImageState, PixelSize, Region, fit_image, load_font}` | `arvane_asset::…` |
| `arvane::asset::Asset` | `kivact_types::Asset` |
| `arvane::asset::{FamilyChoice, FamilyResolution}` | `arvane_text::…` |
| `arvane::{glyph_outline, glyph_ink_bounds, MAX_SCENE_DEPTH}` | `arvane_render::…` |
| `arvane::Rasterizer` | `arvane_compose::CpuCompositor::render` と `arvane_render::dimensions` |
| `arvane::{Accessibility, AccessibilityError, EventTarget, FileDialog, InjectedInput, ShellControl, ShellError, ShellEvent, ShellImeText, ShellWaker, SizeReply, WakeReasons}` | `arvane_shell::…` |
| `arvane::spi::{Backend, ChildHooks, MakeHandler, ShellChild, ShellError, ShellEvent, ShellHandler, ShellLoop, ShellWindow, Step, StepOutcome, SteppedBackend, SteppedLoop}` | `arvane_shell::…`（`CpuFormat` は A07 で消える） |
| `arvane::spi::lose_gpu_device` | `arvane::lose_gpu_device`（feature `test`） |
| `arvane::embed::{ChildHooks, ChildStop, EmbedNotice, PanicSlot}` | `arvane_shell::…` |
| `arvane::embed::EmbedOptions` | `kivact_types::EmbedOptions` |
| `arvane::{Failure, FailureKind}` | `kivact_types::…` |
| `arvane::{test_font, gpu_unavailable}` | 同じ名前（feature `test`） |
| 残る：`arvane::{Window, Windows, Handler, Presented, ExportNotice, FramePlan, Renderer, RendererChange, GpuStartCause, Observer, observer, DriverOptions, SteppedShell, run, start_stepped, embed_driver}` | — |

### 4.4 engine の host の変更（付属書 E への通知）

A05〜A10 は W5b の鎖で engine の次のファイルを変える。A の鎖は W5b、E の W5 の lane（E12・E13・E16）と E の鎖の unit（E15・E17・E18・E20）は W5a なので、同じファイルを同時に持たない（INC-27）。A09 の `fresh_content_version()` を置き換える E14 は W6b。

| unit | engine のファイル | 変更 |
|---|---|---|
| A05 | `host/launch.rs`、`embed.rs`、`host/testing/test_host.rs` | `arvane::run`・`start_stepped`・`embed_driver`、`GpuStart::Never`、`GpuLimit` |
| A06 | `Cargo.toml` と `arvane::` を書く全ファイル（約 60） | 4.3 節の表 |
| A07 | `host/pump.rs`、`host/window.rs`、`host/foreign.rs`、`host/write/writer.rs`、`contract/host.rs` | `set_ime`、`HostWindow.{ime, ime_text}`、`RequestWindowSize` の経路、`foreign_child` |
| A08 | `host/window.rs`、`host/events.rs`、`host/pump.rs`、`host/window_changes.rs` | `Drawable`、`Visibility`、`NotDrawable`、GPU の frame の後に idle の解放を予約しない |
| A09 | `lower/scene_out.rs`、`host/devtools.rs` | `content_version` |
| A10 | `offscreen.rs` と doc の 3 箇所 | premultiplied |

### 4.5 GPU の submit の制約（AR-04 の GPU の部分）

vello_hybrid 0.2.0 は render ごとに `queue.write_buffer_with` で自分の buffer を書く（`render/wgpu/mod.rs`）。queue の書込みは次の submit の **前** に実行されるので、複数の render を一つの encoder に録って 1 回で submit すると、前の render の buffer が後の render の値で上書きされる。したがって A03 は「frame の全 chunk を一つの encoder で 1 回の submit」（監査の案）をせず、**chunk ごとに 1 回の submit**（raster と damage の矩形の copy を同じ encoder に）にする。damage の矩形ごとの texture の新規確保と submit（今 1 矩形あたり 2 回）は無くなる。

### 4.6 SVG の扱い（AR-11、DEC-09 版 2、12 章 EA-15）

- SVG は今のとおり **resvg・tiny-skia で描く**。mask・pattern・filter・clipPath・入れ子の画像と text の能力は変えない（性能の改修のついでに能力を落とさない）。
- 変えるのは font だけ：usvg（resvg の `text` feature）の `<text>` の font は `usvg::Options::font_resolver` の `select_font`（`Fn(&usvg::Font, &mut Arc<fontdb::Database>) -> Option<fontdb::ID>`）と `select_fallback`（`Fn(char, &[fontdb::ID], &mut Arc<fontdb::Database>) -> Option<fontdb::ID>`）で、arvane-text の `FontLookup`（A02。process の一つの system font の走査の clone と App の font）が見つけた face の bytes を `Database::load_font_source(Source::Binary(..))` で読み込ませる。`fontdb::Database::load_system_fonts`（system font の 2 度目の走査）と resvg の feature `system-fonts` を消す。
- K6 の効果は小さい：resvg・tiny-skia・rustybuzz・fontdb の crate は残り、消えるのは fontdb の `fs`・`fontconfig`・`memmap` の feature と、それだけが引く依存。K1 の効果（文字のある最初の SVG の system の走査の 2 度目が無くなる）が主。設計書 2 章の K6 の行の「resvg・tiny-skia・fontdb の削除（AR-11）」は付属書 G が「fontdb の system の走査の削除」に直す（4.9 節）。

### 4.7 `Pixels` を premultiplied にするファイルの表（A10）

| ファイル | 場所 | 変更 |
|---|---|---|
| `crates/kivact-types/src/pixel.rs` | `pub struct Pixels` の doc と `rgba` の doc | 「Premultiplied-alpha RGBA8 pixels a scene draws as they are (each color channel already multiplied by alpha)」と「Tightly packed premultiplied RGBA8, ...」 |
| `crates/kivact-types/src/color.rs` | `premultiply`（新） | A10 手順 1 |
| `crates/arvane-asset/src/raster.rs` | `decode_still`、`collect_frames` | `Pixels` を作る直前に `kivact_types::color::premultiply(&mut rgba)`（`collect_frames` は各 frame の buffer に）。module の doc の straight を premultiplied に |
| `crates/arvane-asset/src/pixels.rs` | `fn tint` | `let p = \|c: u8\| ((u32::from(c) * alpha + 127) / 255) as u8; rgba.extend_from_slice(&[p(r), p(g), p(b), alpha as u8]);`。doc（7、12〜13、22、57 行の文）を premultiplied に |
| `crates/arvane-asset/src/svg/document.rs` | `rasterize` | `kivact_types::color::unpremultiply(&mut rgba);` の行を消す。`svg.rs` の doc の「converted to straight alpha」の文を消す |
| `crates/arvane-asset/src/lib.rs` | crate の doc | 「AssetImage are straight-alpha RGBA8」を「premultiplied RGBA8」に |
| `crates/arvane-render/src/cpu.rs` | `CpuCanvas::source` | `alpha_type: ImageAlphaType::Alpha` → `ImageAlphaType::AlphaPremultiplied`。コメントと doc を直す（vello はなお pixmap を作るので、変換の cache `images.rs` は残す） |
| `crates/arvane-render/src/derived.rs` | `raster`、`resampled` | `raster` の `kivact_types::color::unpremultiply(&mut rgba);` を消す。`resampled` の `ImageAlphaType::Alpha` → `AlphaPremultiplied`。doc（77〜78、106〜107、151 行） |
| `crates/arvane-render/src/gpu/encode.rs` | `fn premultiplied` | `ImageAlphaType::Alpha` → `AlphaPremultiplied`、関数名を `pixmap` に（呼出し 2 箇所も） |
| `crates/arvane-render/src/{lib.rs,images.rs,validate.rs,paint.rs}` | doc | straight の記述を premultiplied に |
| `crates/arvane-compose/src/cpu/draw.rs` | `Straight` と `impl Texels for Straight` | `Premultiplied` に改名し、texel を `[p[0] / 255, p[1] / 255, p[2] / 255, p[3] / 255]`（掛けない） |
| `crates/arvane-compose/src/cpu/graph/apply.rs` | `straight_to_premul` | `to_premul` に改名し、掛け算を消す。`stretch` の doc |
| `crates/arvane-compose/src/gpu/graph.rs` | `upload_premultiplied` | byte をそのまま upload（`bytes.push(..)` の掛け算の loop を `image.rgba` の参照に替える） |
| `crates/arvane-compose/src/gpu/shader.rs` | `premultiplied_texture` | 同上。doc「(straight alpha) premultiplied」を「premultiplied」に |
| `crates/arvane-compose/src/cpu/compositor.rs` | `export_frames`、`ExportRecord` | `ExportFrame::Pixels(unpremultiplied(px))` → `ExportFrame::Pixels(Pixels::new(px.size().0, px.size().1, px.data.clone()))`。`ExportRecord.content: u64` を `signature: u64` にし、`content: content_hash(px)` を `signature: planned.signature`（`plan::PlannedExport` の field）に（AR-14）。`content_hash` と `unpremultiplied` を消す |
| `crates/arvane-compose/src/gpu/export.rs`、`gpu/pass.rs` | `send_exports`、`fn unpremultiply` | `unpremultiply(&mut rgba);` と import を消し、`pass.rs` の関数を消す |
| `crates/kivact-gpu-share/src/{wgpu,vulkan,d3d12}/read_back.rs` | `read_back` | `unpremultiply(&mut rgba);` と import を消す。doc を「premultiplied RGBA8 (as `Pixels`)」に。`{wgpu,vulkan,d3d12}/pool.rs` の read-back の doc、`wgpu/vulkan.rs`・`wgpu/d3d12.rs` の doc、`lib.rs` の 86〜93・131〜132 行の文も |
| `crates/kivact-engine/src/offscreen.rs` | `render_composition` | `kivact_types::color::unpremultiply(&mut frame.rgba);` とそのコメントを消す。doc（10〜11、27、60 行）を premultiplied に |
| `crates/kivact-engine/src/{contract/host.rs,lower/scene.rs,lower/assets.rs}` | doc | `SnapshotResult::Pixels`、`DrawCmd::Pixels`、asset の画素の文を premultiplied に |
| `crates/kivact-svg/src/encode.rs` | `png_data_uri` | 先頭で `let mut straight = rgba.to_vec(); kivact_types::color::unpremultiply(&mut straight);` とし、`straight` を PNG に書く。doc「premultiplied RGBA8, written as a straight-alpha PNG」。`image.rs` の module の doc |
| `crates/kivact-devtools-agent/src/image.rs` | `encode_png` | 同上（copy を unpremultiply して書く）。`rg -n "encode_png\(" crates` の呼出し元が `Pixels` の bytes 以外を渡していれば STOP |
| `crates/kivact-offscreen/src/lib.rs`（C02 の後の場所） | doc | 「its premultiplied output is turned back into straight alpha」を「`Pixels` are premultiplied RGBA8」に |
| `crates/kivact-core/src/view/{drawable_share.rs,node_ref.rs}`、`crates/kivact-style/src/draw/image.rs` | doc | straight-alpha の文（drawable_share 60・361、node_ref 256）を premultiplied に。style の `GpuFrame` の文の「as `Pixels` and CSS colors are」を「as `Pixels` are」に |
| `crates/kivact-ui-basic/src/plot/heatmap.rs` | `color`（`[c.r, c.g, c.b, c.a]` を作るところ） | 返す前に rgb を `((c * a + 127) / 255)` で premultiply する（`kivact_types` は kit の依存に無いので式を書く） |
| `crates/arvane/tests/e2e/present.rs` | `representative` | `0, 255, 0, 128` → `0, 128, 0, 128`、`255, 255, 255, 64` → `64, 64, 64, 64`（同じ色の premultiplied の値） |
| `crates/arvane/tests/e2e/assets.rs` | `the_worker_decodes_png_jpeg_and_svg_by_their_content`、`rgba` の doc | 期待 `[255,0,0,255, 0,255,0,128, 0,0,255,255, 255,255,255,64]` を `[255,0,0,255, 0,128,0,128, 0,0,255,255, 64,64,64,64]`、message「premultiplied alpha, as stored」。SVG の点の `assert_eq!((r, g, b), (0, 0, 255), "straight color")` を `assert_eq!((r, g), (0, 0)); assert!(b.abs_diff(a) <= 1, "premultiplied blue equals alpha");` |
| `crates/arvane/tests/e2e/support.rs` | doc 71 行 | 「A premultiplied color」に |
| `dev/tests/kivact-public-api/tests/*/offscreen_svg.rs` | 51 行の `Pixels::new(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 128])` | `vec![255, 0, 0, 255, 0, 0, 128, 128]` |
| `dev/tests/kivact-public-api/tests/*/drawables.rs`、`drawable_export_gpu.rs` | doc（「straight-alpha RGBA」） | premultiplied に（値は opaque なので期待値は変えない） |
| `dev/tests/gpu-interop-producer/tests/read_back.rs` | 期待 `[199, 100, 0, 128, 0, 0, 0, 0]`、message「straight alpha on {}」 | `[100, 50, 0, 128, 7, 7, 7, 0]`、message「premultiplied, as written, on {}」（read-back は変換しない契約になる） |
| `dev/tests/gpu-interop-producer/src/{producer.rs,engine.rs}` | doc の「straight」 | premultiplied に |

### 4.8 依存の向きと `visible_to` の変更

- A04：`arvane-asset` → `arvane-text`（`FontLookup`）。`visible_to` の `arvane-text` に `arvane-asset` を足す。設計書 4.2 節の図に `arvane-text ← arvane-asset` を足す（付属書 G が architecture.md に書く）。`arvane-asset` は `arvane-protocol`・`arvane-render` に依存しない（INC-32 の追加は DEC-09 版 2 で不要になった）。
- A06：`kivact-engine` → `arvane-text`、`arvane-asset`、`arvane-render`、`arvane-compose`（4.2 節の図は engine → arvane-render を挙げていないが、`dimensions`・`glyph_outline`・`glyph_ink_bounds`・`MAX_SCENE_DEPTH`・`RenderError` を直接使うので足す）。`arvane` は arvane-text と arvane-asset に依存しなくなる。
- 付属書 T の W1 の `visible_to` の既定（dev-dependency は対象外）を前提にする（arvane の試験の dev-dependency）。

### 4.9 付属書 G へ（W11）

- architecture.md の crate の表：`arvane` は「window の driver と presenter（frame の方針、GPU の device は Driver に一つ）。他の crate を再公開しない」。`arvane-text` は「Parley の shaping と、family の解決・総称 family・system font の遅延の走査・SVG の font の検索」。`arvane-asset` は「decode（premultiplied）、resvg の SVG（font は arvane-text の `FontLookup`）、遅延して起こす worker」。`arvane-render` の「Vello と wgpu を閉じる」の文を「vello は内に閉じる。wgpu は device を持つ arvane-compose と共有する」に（AR-21）。
- decisions.md に：J-1〜J-3 の結論、4.5 節の submit の制約、4.6 節の SVG の font の経路。設計書 2 章の K6 の行の手段を「fontdb の system の走査の削除（AR-11）」に直す。
- 利用者の docs：`GpuStart::Never`（`ARVANE_RENDERER=cpu` と同じ）、`prefer_gpu` が無いこと、`Pixels` が premultiplied であること（canvas.md・drawables.md・gpu-texture-sharing.md・show-app-rendered-images.md の straight の記述。4.7 節の表の doc の行と同じ文に）。

### 4.10 設計判断の結果（親が決定済み）

- **J-1（AR-17 の範囲）**：決定済み（採用。設計書 7.7 節）。監査は `load_font`・`glyph_outline`・`glyph_ink_bounds` も arvane-text へ移すとしている。しかし (a) `glyph_outline` は arvane-render の painter が変形した glyph を描くのに使う（`draw/painter.rs`）ので、移すと arvane-render → arvane-text の依存ができる；(b) `load_font` はファイルを読む（`read::read`、上限、path を書いた error）ので、移すと arvane-text → arvane-asset の依存ができ、SVG の font のために足す arvane-asset → arvane-text と循環する。**推奨**：family の解決（重複していた知識）だけを arvane-text へ移し（A02）、`glyph_outline`・`glyph_ink_bounds` は arvane-render に一つの module として残し（A01 で重複を統合）、`load_font` は arvane-asset に残す。本付属書はこの推奨で書いた。
- **J-2（`prefer_gpu` の担当）**：決定済み（INC-07）。ST-23（`prefer_gpu` の削除と `GpuStart::Never`）は A05 が実装し、`kivact-types/src/{embed.rs,gpu.rs}` の変更は A05 の所有。付属書 S・C には手順が無い。
- **J-3（SVG）**：決定済み（DEC-09 版 2、12 章 EA-15）。SVG を `RenderScene` に変換する案は撤回。resvg・tiny-skia の描画を保ち、usvg の font だけを arvane-text から渡す（4.6 節、A04）。

### 4.12 Orchestrator への依頼（測定）

- CP1 の K1（`kv_startup`）の測定で、`arvane_text::fonts` の debug の log（`waited_us`：最初の文字の計測が system font の走査を待った時間）を有効にして 1 回起動し、K1 と一緒に記録する（12 章 EA-09。合否には使わない）。
- CP2 の K2（idle の WS）で、A15 の pool が idle の解放の後に空であることは単体試験で確かめてある（`forget_frame_state_releases_every_pooled_target`）。

### 4.11 他の付属書の wave との関係

- 付属書 S：W2a の S02〜S06 の後に A01・A02（W3a）。A11・A12 は W6b で、W6b の S の鎖の unit とファイルが重ならない（INC-28。W6a の S13・S14 とは wave が別）。A10 は W5b の鎖で `kivact-types/src/{pixel.rs,color.rs}` を持つ。
- 付属書 C：C13（W5a の鎖）が embed の native のファイルを持つので、embed の native の lane の unit（A12 の注入、A14）は W6b・W7a に置いた。A05・A07・A08 は C の鎖の後で、C13 の後の名前（`EmbedVisibility`）を使う。
- 付属書 E：W3〜W4 の E の lane は engine を持つので、本付属書は W3〜W4 で engine に触らない（A02 の facade の `engine.rs` は arvane の中）。4.2 節の版の実装は E14（W6b）。W5 の engine の変更は 4.4 節のとおり W5a（E）と W5b（A）に分かれる。
- 付属書 T：W1 の state の規則の後なので、本付属書の印は `// xtask-allow(state): ...` の struct 単位の 1 行だけを書く。
- 付属書 K：kivact-ui-basic の heatmap の 1 関数を A10 が W5b で直す（kit が compile の対象から外れる W6 より前）。

## 5. 自己点検

- **採用・変更して採用の全所見が unit にあるか**：AR-01（A01・A03）、AR-02（A11・A13）、AR-03（A09・A16）、AR-04（A01・A03）、AR-05（A12）、AR-06（A06）、AR-07（A07）、AR-08（A08、変更して採用のとおりモバイルの event を残す）、AR-09（A08・A15）、AR-10（A10）、AR-11（A04。DEC-09 版 2）、AR-12（A02・A04）、AR-13（A03）、AR-14（A10・A13）、AR-15（A14。EA-04〜EA-06 の形）、AR-16（A05）、AR-17（A01・A02、J-1）、AR-18（A16）、AR-19（A01・A06・A07・A12）、AR-20（9 項目：`limits_noted` A05、`wake.rs` A12、`ComposeError` A03、`DeviceLost` A03、`GpuSurface` A03、DPI A12、X11 の timerfd A12、`RefCell<TextEngine>` A06、Win32 の `Environment` A12）、AR-21（native の doc A12、architecture.md は G、`ForeignChild` A07、arvane の lib doc A06）、AR-22（A08）、AR-23（A13）、AR-24（A01）、AR-25（A12）、AR-26（T）。Q2（stem darkening）は変えない：A01 は `dilation` の引数の形だけを変え、定数と bucket は変えない。Q5 は不採用のとおり `Suspended` などを残す。
- **設計の判断を実装者に残していないか**：選択が要る箇所は 4.10 節の J-1〜J-3 に推奨つきで出した。実装者に残る「確かめる」手順は、名前の照合（`rg` で場所を出す）と、前提が崩れたときの STOP（Collection の `Send + Sync`、usvg の木の形、embed の注入の helper の有無、`encode_png` の呼出し元）だけ。
- **同じ wave の unit が同じファイルを持たないか**：
  - W3a：A01（arvane-render の列挙したファイル、compose の `cpu/compositor.rs`）と A02（arvane-text、arvane-asset の `font.rs`・`lib.rs`、arvane の `engine.rs`・`asset.rs`）。重ならない。
  - W4：A03（arvane-render の `gpu.rs`・`gpu/*`・`lib.rs`、compose の `plan/` 以外）と A04（arvane-asset、`Cargo.lock`、`workspace.toml` の `arvane-text` の 1 行）。重ならない。A03 は `Cargo.lock` を変えない（依存を足さない）。
  - W5b：本付属書の鎖 A05〜A10 だけ（順に一つずつ）。
  - W6b：A11（compose の `gpu*`・`lib.rs`、arvane の `presenter/gpu.rs`・`presenter/frame_policy.rs`・`test_hooks.rs`。2 章と A11 の所有も同じ）と A12（arvane-native、arvane-shell の `types.rs`・`window.rs`・`a11y/shared.rs`、arvane-virtual の `window.rs`）。重ならない。
  - W7a：A13（arvane の `window.rs`・`presenter*`・`gpu_driver.rs`・`test_hooks.rs`・`lib.rs`、compose の `gpu/{export.rs,device.rs,state.rs}`・`gpu.rs`・`lib.rs`・`gpu_unbuilt.rs`、arvane の試験 `presenter.rs`）と A14（arvane-native の embed・win32・x11、arvane-shell の `child.rs`）。重ならない。
  - W7b：A15（compose の `cpu/*`、arvane の `presenter/frame_policy.rs`）。A2 は無い。
  - W8：A16（compose の `plan/{hash.rs,builder.rs,layers.rs}`・`Cargo.toml`・`gpu/device.rs`、kivact-gpu-share、gpu-interop-producer の 2 ファイル）。A1 は無い。
- **新しい状態の型の閉じ方**：`SystemFonts`・`Runner`・`Drawable`・`SurfacePresence`・`State`（driver）・`WindowGpu`・`Child`・`HandlerSlot`・`Cleanup`・`Retain` は enum。印を書くのは `Pipelines`（A03）と `GpuDevice`（A11）の 2 つと、既存の印の文面の更新（`Presenter`・`HostWindow`・`Session`）。組合せの印で状態を通す箇所は無い（EA-06）。
- **試験**：足す試験は 16（`generic_and_quoted_family_names_resolve_without_reporting`、`the_font_scan_worker_starts_once_and_is_noted_as_detached`、`svg_text_uses_a_registered_font_without_a_system_font_database`、`an_svg_without_text_loads_no_font`、`a_surface_made_again_has_a_new_generation`、`injected_input_reaches_an_embedded_app`、`two_windows_of_one_driver_share_one_gpu_device`、`two_drivers_have_a_device_each_and_closing_one_leaves_the_other_drawing`、A14 の 6 つ、`the_target_pool_keeps_at_most_64_mib`・`forget_frame_state_releases_every_pooled_target`（A15）、`equal_input_hashes_equal_and_one_byte_differs`（A16））。期待値を変える試験は A10 の表の行だけで、確かめる事実は変えず premultiplied の表し方に直す。消す試験は A02 の `resolve_family`・`candidates` の単体試験と、A16 の hash の値を固定した単体試験（あれば。名前を報告する）。SVG の機能を落とす試験・baseline は無い（A04 は描画を変えない）。
