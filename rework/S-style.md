# 付属書 S：style・types・shader・svg と arvane-protocol

対象 crate：`kivact-types`、新設 `arvane-protocol`、`kivact-style`、`kivact-shader`、`kivact-svg`。根拠は `rework/audit/style-types.md`（ST-01〜ST-31）、契約は `REWORK_DESIGN.md` 5.7・5.8・6.2 節。

## 1. 範囲と前提

### 1.1 この付属書が実装する所見

| 所見 | 判定（設計書 7.4） | unit |
|---|---|---|
| ST-01 theme 変数 `ThemeVar` | 採用（DEC-02） | S15 |
| ST-02 `THEMED` 表 | 採用（ST-01 で消える） | S15 |
| ST-03 theme なしの解決 | 採用 | S09（足す API）、S16（消す・改名） |
| ST-04 palette の token が黒 | 採用（別名の解決で閉じる） | S15 |
| ST-05 状態はすべての property を受ける | 採用（O-2 採用） | S04（engine の報告）、S19（style・core） |
| ST-06 compile の複製 | 採用 | S18（style）、S19（core の `bind_style`） |
| ST-07 `Theme::ext` の fallback | 採用 | S09 |
| ST-08 宣言の表 | 採用 | S17、S18 |
| ST-09 状態名の列挙 | 採用 | S04（types・engine）、S18（style・core） |
| ST-10 同一の型の組 | 変更して採用（同一の組だけ） | S13 |
| ST-11 文字の style の型 | 採用 | S06（render 側の名前）、S22 |
| ST-12 `text_decoration_style` | 採用 | S17 |
| ST-13 SVG の gradient | 採用 | S05 |
| ST-14 SVG の塗りの範囲 | 採用 | S05 |
| ST-15 SVG が黙って落とすもの | 採用（DEC-11） | S11 |
| ST-16 SVG の幾何の重複 | 採用 | S05 |
| ST-17 `Mix` | 採用 | S03（wire）、S21（公開 API） |
| ST-18 `Draw` の後埋め | 採用 | S12 |
| ST-19 `LayoutProps.hidden` | 採用 | S13 |
| ST-20 `arvane-protocol` と `prop_interp` | 変更して採用（`kivact-cell` は作らない） | S02 |
| ST-21 同名の異義 | 採用 | S06（protocol・engine の名前）、S14（types の名前） |
| ST-22 CSS の名前と module | 採用（O-5 は CSS の名前） | S20 |
| ST-23 `prefer_gpu` | 採用 | **付属書 A の A05（W5 の鎖）が実装する**（4.7 節）。本付属書に unit は無い |
| ST-24 色の f32 化 | **不採用** | なし |
| ST-25 naga を GPU build だけに | 採用（O-4 採用） | S07 |
| ST-26 `Theme(Arc)` | 採用 | S09 |
| ST-27 `TypeMap` | 採用 | S10（types と style）。ui・core 側は 4.8 節 |
| ST-28 印の雑音 | 採用 | S23（style・types・svg・shader に残る印。規則の変更は付属書 T の W1a） |
| ST-29 doc の矛盾 | 採用 | S23 |
| ST-30 残骸 | 採用 | S08（内部）、S17（`duplicates` の比較）、S20（`ScrollbarWidth::Thin`） |
| ST-31 塗りの長さの 0 | 採用 | S18 |
| （DEC-13）`Deferred`・`SharedDeferred` | 設計書 6.2 | S01 |

### 1.2 前提（他の付属書が先に済ませること）

- W1a：付属書 T が xtask の `state` 規則を `state-fields` から置き換える（設計書 6.1）。本付属書の unit が書く `// xtask-allow(state): ...` は新しい規則の書式である。W1b で付属書 T が `xtask-allow(state-fields)` の印を消す。本付属書の unit は、W1b の後に残った `xtask-allow(state-fields)` を見つけたら、その行を消す（新しい規則では不要）。
- W1：付属書 T が e2e を `e2e_core`・`e2e_ui`・`e2e_kits`・`e2e_native` に分ける。ファイルは `dev/tests/kivact-public-api/tests/<binary>/<topic>.rs` に移る。本付属書は試験を **ファイル名（topic）と関数名** で指す。どの binary に入ったかは `ls dev/tests/kivact-public-api/tests/*/<topic>.rs` で確かめて、確認のコマンドの `--test` に使う。
- W5：付属書 R・C の鎖が runtime・core の公開 API を変える（`Binding`→`Signal`、`Notifier`→`Trigger`、`EffectKind` など）。W6 の unit の手順が runtime・core の API を名指すときは、**W5 の後の名前** を使う（付属書 R・C の 4 節の対応表）。本付属書のコードに出る runtime の名前（`Owner::effect`、`EffectKind::Render`、`ReportOnceSet`、`Prop`、`IntoValue`）は 2b8df5ac の名前であり、W5 で変わっていればその対応表の新しい名前に置き換える。
- W6 から W9〜W10 まで、5 つの kit・その catalog・`crates/kivact-ui-*/tests`・`e2e_kits` は compile の対象外（DEC-14）。W6 の unit はそれらを直さない。`kivact-docs`（`dev/tests/docs`）も W5 から付属書 G まで対象外。

### 1.3 後の付属書に渡すもの

- 付属書 U・K：theme 変数（`ThemeVar`、中核の `color`・`space`・`radius`・`font`、部品・kit の変数の宣言の型）、旧 `Palette`・`Spacing`・`Radii`・`Typography` からの対応表（4.2〜4.4 節）、`Theme::set/get`（すべての型の別名）、`kivact::style::control`、`Theme::with_fonts`（`Asset`）、`TypeMap`（4.8 節）、新しい setter の名前の対応表（4.5 節）、`kivact::draw` から `kivact::style` へ移る型の一覧（4.6 節）。
- 付属書 C：`kivact_style::internal::{StyleProperty, style_property}`（S17。C19 が S17 の直後に使う。INC-03）、`Theme::with_fonts`・`Theme::fonts`（App と `RenderOptions::with_theme` が font を登録する。USR-12）、`kivact_style::internal` の新しい入口、prelude に足す名前（4.7 節）。
- 付属書 E・A：`arvane-protocol` の型の一覧、W6 の鎖が触る engine・layout・arvane のファイルの一覧（4.10 節。W6 の間、E・A の lane はそのファイルを編集しない）。
- 付属書 G：移行ガイドに載せる旧名→新名（4.5・4.6 節）。

## 2. unit の一覧

| ID | wave | lane | 前提の unit | 所有するファイル・directory（詳細は各 unit の「所有」） | 所見 | 規模 |
|---|---|---|---|---|---|---|
| S01 | W1a | S | なし | `crates/kivact-types/src/cell.rs` | DEC-13 | S |
| S02 | W2a | A | S01 | `crates/kivact-types/`、新設 `crates/arvane-protocol/`、`crates/kivact-engine/`、`crates/arvane*/` の `use` 行、指定の 6 ファイル（core・editing・style・test）、`Cargo.toml`、`dev/xtask/policy/workspace.toml` | ST-20 | L（機械的） |
| S03 | W2a | A | S02 | types `pixel.rs`・`scene/recording.rs`、engine の 3 ファイル、svg `emit.rs`、style `draw/layer.rs` | ST-17（wire） | S |
| S04 | W2a | A | S03 | types 新 `style_state.rs`・`paint.rs`・`lib.rs`、engine の 9 ファイル、style `resolve/compile.rs`、core の 2 ファイル | ST-09、ST-05（engine） | M |
| S05 | W2a | A | S04 | types 新 `scene/resolve.rs`・新 `gradient_stops.rs`、engine の 4 ファイル、svg `shape.rs`・`fill.rs`・`emit.rs` | ST-13、ST-14、ST-16 | M |
| S06 | W2a | A | S05 | `crates/arvane-protocol/`、`crates/kivact-engine/`、`crates/arvane*/`（名前の置換） | ST-21（protocol・engine）、ST-11（render の span の名前） | M（機械的） |
| S07 | W2a | S | なし | style `Cargo.toml`・`shader/program.rs`・`shader/data.rs`、core `Cargo.toml`、kivact `Cargo.toml`、e2e `shaders.rs` | ST-25 | S |
| S08 | W2a | S | S07 | style `effect/filter_graph.rs`・`effect/filter.rs`・`icon.rs` | ST-30（内部） | S |
| S09 | W2a | S | S08 | style `theme/value.rs`、`color/value.rs`、新 `theme/cache.rs`、`theme.rs`、`lib.rs`、`facade.rs` | ST-26、ST-07、ST-03（足す API） | M |
| S10 | W3a | S | S09 | types 新 `type_map.rs`・`lib.rs`、style `theme/value.rs` | ST-27 | S |
| S11 | W3a | S | S10 | `crates/kivact-svg/src/`（全体）、e2e `offscreen_svg.rs` | ST-15 | M |
| S12 | W3a | S | S11 | style `draw/recorder.rs`・`draw/regions.rs`・`draw/paragraph.rs`・`draw/text_run.rs`・`internal.rs`、core `view/mount/draw.rs`、`crates/kivact-offscreen/src/lib.rs` | ST-18 | S |
| S13 | W6a | 鎖 | （W5b の終わり） | 4.10 節の S13 の一覧 | ST-10、ST-19 | L（機械的） |
| S14 | W6a | 鎖 | S13 | 4.10 節の S14 の一覧 | ST-21（types） | M（機械的） |
| S15 | W6b | 鎖 | S14 | style、types 新 `theme_var.rs`、W6 の掃引の範囲全体（1.2 節） | ST-01、ST-02、ST-04 | L |
| S16 | W6b | 鎖 | S15 | style `color/value.rs`・`theme/value.rs`、掃引の範囲 | ST-03 | S |
| S17 | W6b | 鎖 | S16 | style `sheet/`、新 `sheet/props.rs`・`sheet/decl.rs`・`sheet/shorthands.rs`、掃引の範囲（`text_decoration_style` の呼出し） | ST-08（前半）、ST-12、ST-30 | L |
| S18 | W6b | 鎖 | S17、C19 | style `resolve/`、`sheet/`、core `view/mount/style.rs`・`view/mount/facts.rs`・`view/mount/draw.rs`・devtools の 5 ファイル | ST-08（後半）、ST-06、ST-31、ST-09（style） | L |
| S19 | W6b | 鎖 | S18 | style `resolve/`、core 新 `view/node_states.rs`・`view/mount/style.rs`・`client/dispatch/delivery.rs`・`view/state.rs` | ST-05 | M |
| S20 | W6b | 鎖 | S19 | style・core `style.rs`・`draw.rs`・`view/span.rs`、掃引の範囲 | ST-22、ST-30（`Thin`） | L（機械的） |
| S21 | W6b | 鎖 | S20 | style `draw/layer.rs`・`draw.rs`、core `draw.rs`、掃引の範囲 | ST-17（公開） | S |
| S22 | W6b | 鎖 | S21 | types `text/`、style `text.rs`・`internal.rs`、core `view/span.rs`・`custom/text_editing.rs`、engine の 5 ファイル、arvane-text の 2 ファイル、ui の 2 ファイル、e2e の 3 ファイル | ST-11 | L |
| S23 | W6b | 鎖 | S22 | `crates/kivact-types/`、`crates/kivact-style/`、`crates/kivact-svg/`、`crates/kivact-shader/`、`crates/arvane-protocol/` の印と doc | ST-28、ST-29 | S |

wave は作業指示書 4 章（SCHEDULE）の sub-wave：S01 は W1a、S02〜S06 は W2a の lane A、S07〜S09 は W2a の lane S、S10〜S12 は W3a、S13・S14 は W6a、S15〜S23 は W6b の鎖（S17 の直後に付属書 C の C19 が入る）。W2a の lane A（S02〜S06）は付属書 S だけが持つ。W2a の間 lane A は `crates/kivact-types/`・`crates/kivact-engine/`・`crates/arvane*/`・`crates/kivact-svg/` の全体と、各 unit の「所有」に挙げた core・style・editing・test のファイルを所有する。W2a の lane S（S07〜S09）は kivact-types・engine・arvane・svg に触らない。付属書 C の C01・C02 は W2b（同じファイルを W2a と分けた。INC-23）。

## 3. unit の手順

共通の決まり（全 unit）：

- シェルは Windows の Git Bash。`perl -pi -e` を使う。`rg` が無い環境では `git grep -n -E` を使う（同じ正規表現）。
- 新しいファイルの先頭に `//! Role: <英語 1 文>` を書く（CONTRIBUTING）。コメントと rustdoc は英語。
- 手で書いたコードの後に `cargo fmt -p <crate>` を通す。sweep の後に未使用の `use` が残ったら `cargo fix --allow-dirty --allow-staged -p <crate> --all-targets` で消す（未使用の import の削除だけを目的に使う。ほかの提案が出たら適用せずに戻す）。
- 試験のコマンドは test binary ごとに一行で、filter は一行に一つ（作業指示書 PLAN-07）。filter が 0 件の試験しか実行しなかったら失敗として扱う（module の名前は付属書 T の T15 の表で確かめた。「上のコマンドが通る」は各行が 1 件以上の試験を実行して成功すること）。
- コマンドは worktree の root で実行する。固定の `cd` は書かない（場所が要るときは `ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1`）。
- 「確認（tier a）」の window を開きうる package の `cargo test` は `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test ...` の形で書く（INC-15・INC-43）。

---

## S01：`Deferred<T>` と `SharedDeferred<T>`

- **wave / lane / 前提**：W1a / S / なし
- **所有**：`crates/kivact-types/src/cell.rs`
- **目的**：遅延初期化の唯一の道具を作る（設計書 6.2、DEC-13）。std の `OnceCell`・`OnceLock` はこの二つの実装の中だけで使う。
- **手順**：
  1. `crates/kivact-types/src/cell.rs` の先頭の doc（`//! Role: Share diagnostic latches, ...`）の Role 行を次に置き換える：
     `//! Role: Share diagnostic latches, caches, generations, mailboxes and deferred values.`
  2. 同じファイルの末尾（`impl<T: PartialEq> Recent<T>` の後）に次を足す：

```rust
/// A value built on first use on the UI thread (PERF-2, PERF-10): an ICU
/// formatter, a GPU pipeline, a parsed table. Not for repairing state or
/// filling a half-built object later (S-7). `!Sync`: use
/// [`SharedDeferred`] in a `static` or across threads.
pub struct Deferred<T>(
    #[expect(
        clippy::disallowed_types,
        reason = "the one wrapper of std::cell::OnceCell the workspace allows (DEC-13)"
    )]
    std::cell::OnceCell<T>,
);

impl<T> Deferred<T> {
    /// Nothing built yet.
    #[expect(
        clippy::disallowed_types,
        reason = "the one wrapper of std::cell::OnceCell the workspace allows (DEC-13)"
    )]
    pub const fn new() -> Self {
        Self(std::cell::OnceCell::new())
    }

    /// The value, built by `f` on the first call; `f` runs at most once.
    pub fn get_or_init(&self, f: impl FnOnce() -> T) -> &T {
        self.0.get_or_init(f)
    }

    /// The value if it was built.
    pub fn get(&self) -> Option<&T> {
        self.0.get()
    }
}

impl<T> Default for Deferred<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A value built on first use and shared across threads (`Sync`), for a
/// `static`: `static TABLE: SharedDeferred<Table> = SharedDeferred::new();`.
/// The same rules as [`Deferred`].
pub struct SharedDeferred<T>(
    #[expect(
        clippy::disallowed_types,
        reason = "the one wrapper of std::sync::OnceLock the workspace allows (DEC-13)"
    )]
    std::sync::OnceLock<T>,
);

impl<T> SharedDeferred<T> {
    /// Nothing built yet.
    #[expect(
        clippy::disallowed_types,
        reason = "the one wrapper of std::sync::OnceLock the workspace allows (DEC-13)"
    )]
    pub const fn new() -> Self {
        Self(std::sync::OnceLock::new())
    }

    /// The value, built by `f` on the first call; a racing caller waits and
    /// then shares the one value.
    pub fn get_or_init(&self, f: impl FnOnce() -> T) -> &T {
        self.0.get_or_init(f)
    }

    /// The value if it was built.
    pub fn get(&self) -> Option<&T> {
        self.0.get()
    }
}

impl<T> Default for SharedDeferred<T> {
    fn default() -> Self {
        Self::new()
    }
}
```

  3. `#[expect]` が field の属性として受け付けられず compile の error（`unfulfilled_lint_expectations` を含む）になった場合は、`#[expect(...)]` を struct の宣言の直前（`pub struct Deferred<T>(` の前の行）に移す。どちらの位置でも文面は手順 2 のまま。
  4. 同じファイルの末尾に試験の module を足す：

```rust
#[cfg(test)]
mod deferred_tests {
    //! Role: Tests of the deferred values.
    use super::{Deferred, SharedDeferred};

    #[test]
    fn deferred_builds_once_on_first_use() {
        let calls = std::cell::Cell::new(0);
        let value = Deferred::new();
        assert!(value.get().is_none());
        assert_eq!(*value.get_or_init(|| { calls.set(calls.get() + 1); 7 }), 7);
        assert_eq!(*value.get_or_init(|| { calls.set(calls.get() + 1); 8 }), 7);
        assert_eq!(calls.get(), 1);
        assert_eq!(value.get(), Some(&7));
    }

    static SHARED: SharedDeferred<String> = SharedDeferred::new();

    #[test]
    fn shared_deferred_is_a_static() {
        assert_eq!(SHARED.get_or_init(|| "a".to_owned()), "a");
        assert_eq!(SHARED.get_or_init(|| "b".to_owned()), "a");
    }
}
```

- **消すもの**：なし。既存の `OnceCell`・`OnceLock`・`LazyLock` の利用（runtime、arvane-native、arvane-virtual、ui、kit、`examples/kivact-ui-basic`）は、それぞれの crate の付属書が自分の unit で `Deferred`・`SharedDeferred` に置き換える（4.11 節に一覧）。
- **状態の閉じ方**：どちらも field 一つ（6.1 の対象外）。
- **試験**：`crates/kivact-types/src/cell.rs` の `deferred_tests::deferred_builds_once_on_first_use`、`deferred_tests::shared_deferred_is_a_static` を足す。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-types
  cargo check -p kivact-types --all-targets
  cargo clippy -p kivact-types --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-types
  cargo test -p kivact-types cell
  ```
- **完了条件**：`rg -n "disallowed_types" crates/kivact-types/src` が `cell.rs` の 2 か所だけを返す。上のコマンドがすべて通る。

---

## S02：`arvane-protocol` の新設と `prop_interp` の engine への移動

- **wave / lane / 前提**：W2a / A / S01
- **所有**：
  - `crates/kivact-types/`（全体）
  - 新設 `crates/arvane-protocol/`
  - `crates/kivact-engine/`（全体）
  - `crates/arvane/`、`crates/arvane-shell/`、`crates/arvane-native/`、`crates/arvane-virtual/`、`crates/arvane-text/`、`crates/arvane-render/`、`crates/arvane-compose/`、`crates/arvane-asset/`（`use` 行・型の path・`Cargo.toml`）
  - `crates/kivact-core/src/view/drawable/events.rs`、`crates/kivact-core/src/view/drawable_share.rs`、`crates/kivact-editing/src/geometry.rs`、`crates/kivact-style/src/draw.rs`、`crates/kivact-style/src/draw/layer.rs`、`crates/kivact-test/src/screen/state.rs`、`crates/kivact-test/src/lib.rs`
  - root の `Cargo.toml`（`[workspace] members`）、`dev/xtask/policy/workspace.toml`
- **目的**：ST-20。host と Arvane の frame の契約を `arvane-protocol` に移し、engine だけが使う `prop_interp` を engine に移す。`kivact-types` を「純粋な値と、std だけの小さな同期 helper」に戻す。名前はこの unit では変えない（改名は S06）。
- **手順**：
  1. 移す・残すものを確定する（変更しない一覧）：
     - `crates/kivact-types/src/render/shaped.rs` は **types に残す**：`git mv crates/kivact-types/src/render/shaped.rs crates/kivact-types/src/shaped.rs`。中の型（`FontSource`、`Glyph`、`GlyphRun`、`GlyphRunSnapshot`、`InlineBox`、`InlineBoxPlacement`、`ShapedInlineText`、`ShapedText`、`ShapedTextSnapshot`、`TextDecorationRect`、`TextHighlight`、`TextLine`）は `kivact_types::shaped::*`。
     - `render/composition.rs` の `Composite`、`ExportFrame`、`ExportedFrame`、`FrameStats` は **types に残す**（手順 3）。
     - それ以外（`render/composition.rs` の残り、`render/effect.rs`、`render/paint.rs`、`render/scene.rs`、`render/semantics.rs`、`render/text.rs`）は `arvane-protocol` へ移す。
  2. crate を作る：
     ```
     mkdir -p crates/arvane-protocol/src
     git mv crates/kivact-types/src/render/composition.rs crates/arvane-protocol/src/composition.rs
     git mv crates/kivact-types/src/render/effect.rs crates/arvane-protocol/src/effect.rs
     git mv crates/kivact-types/src/render/paint.rs crates/arvane-protocol/src/paint.rs
     git mv crates/kivact-types/src/render/scene.rs crates/arvane-protocol/src/scene.rs
     git mv crates/kivact-types/src/render/semantics.rs crates/arvane-protocol/src/semantics.rs
     git mv crates/kivact-types/src/render/text.rs crates/arvane-protocol/src/text.rs
     git rm crates/kivact-types/src/render.rs
     ```
     `crates/arvane-protocol/Cargo.toml` を作る：
     ```toml
     [package]
     name = "arvane-protocol"
     authors.workspace = true
     version.workspace = true
     edition.workspace = true
     license.workspace = true
     repository.workspace = true
     description = "The frame protocol between the Kivact engine and Arvane: scenes, compositions, semantics and frame reports"

     [dependencies]
     kivact-types = { path = "../kivact-types", version = "0.1.0" }

     [lints]
     workspace = true
     ```
     `crates/arvane-protocol/src/lib.rs` を作る：
     ```rust
     //! Role: The frame protocol a Kivact host hands Arvane and what a window reports back.
     //!
     //! The engine produces these values and the Arvane crates consume them:
     //! one frame's composition and scenes, the semantics tree, the CPU frame
     //! and the frame report. It is a crate of its own because its one reason
     //! to change is the renderer contract; the values it builds on (geometry,
     //! colors, shaped text) are `kivact-types`.
     #![forbid(unsafe_code)]

     mod composition;
     mod effect;
     mod paint;
     mod scene;
     mod semantics;
     mod text;

     pub use composition::{
         CompositeLayer, Composition, Export, ExportLoss, ExportMode, ExternalItem, FrameReport,
         LayerEffects, LayerFilter, LayerItem, LayerKey, MAX_CONTEXT_PLANES, PixelRect, Plane,
         RasterGrid, RenderBackdrop, RenderMask, ShaderItem,
     };
     pub use effect::BoxShadow;
     pub use paint::{Gradient, GradientShape, Paint, Pattern, PatternSource};
     pub use scene::{CpuFrame, GlyphStroke, Group, RenderCmd, RenderScene, SHADOW_REACH};
     pub use semantics::{SemanticsNode, SemanticsTree, SemanticsUpdate};
     pub use text::{RichText, SpanStyle, TextSpan};
     ```
     root の `Cargo.toml` の `[workspace] members` の `"crates/kivact-types",` の次の行に `"crates/arvane-protocol",` を足す。
  3. `Composite`・`ExportFrame`・`ExportedFrame`・`FrameStats` を types に戻す：
     - `crates/arvane-protocol/src/composition.rs` から `pub enum Composite { ... }`（doc と derive を含む全体）を切り取り、`crates/kivact-types/src/pixel.rs` の末尾に貼る。
     - 同じファイルから `pub enum ExportFrame`、`pub struct ExportedFrame`、`pub struct FrameStats`（各 doc と derive を含む）を切り取り、新しいファイル `crates/kivact-types/src/export.rs` に置く。先頭は：
       ```rust
       //! Role: Exported drawable frames and frame statistics shared by the engine, Arvane and core.
       use crate::ids::DrawableId;
       use crate::pixel::Pixels;
       ```
     - `crates/arvane-protocol/src/composition.rs` の先頭の `use` に次を足す：`use kivact_types::export::{ExportedFrame, FrameStats};`、`use kivact_types::pixel::Composite;`。
  4. 移したファイルの path を直す（`crates/arvane-protocol/src/*.rs` だけに実行する）：
     ```
     perl -pi -e 's/\bcrate::render::/PROTO__::/g; s/\bcrate::/kivact_types::/g; s/PROTO__::/crate::/g; s/\bsuper::/crate::/g' crates/arvane-protocol/src/composition.rs crates/arvane-protocol/src/effect.rs crates/arvane-protocol/src/paint.rs crates/arvane-protocol/src/scene.rs crates/arvane-protocol/src/semantics.rs crates/arvane-protocol/src/text.rs
     ```
     手で直すもの：
     - `crates/arvane-protocol/src/scene.rs` の `use crate::shaped::GlyphRunSnapshot;`（手順の置換で `super::shaped` から変わったもの）を `use kivact_types::shaped::GlyphRunSnapshot;` にする。
     - `crates/arvane-protocol/src/text.rs` の doc の ``[`crate::ShapedText::highlights`]`` を ``[`kivact_types::shaped::ShapedText::highlights`]`` にする。
     - `crates/arvane-protocol/src/paint.rs` の doctest の `use kivact_types::render::{Gradient, GradientShape, Paint};` を `use arvane_protocol::{Gradient, GradientShape, Paint};` にする。
     - 型の `pub` の field・関数はそのまま（protocol の公開の契約）。
  5. types の中を直す：
     - `crates/kivact-types/src/shaped.rs`：変更なし（`use crate::...` はそのまま有効）。Role 行を `//! Role: Shaped text: glyph runs, lines, decoration and highlight rects, shared by the engine, Arvane and drawing.` にする。
     - `crates/kivact-types/src/text_shaping.rs`：`use crate::render::GlyphRun;` → `use crate::shaped::GlyphRun;`。
     - `crates/kivact-types/src/scene/recording.rs`：`use crate::render::Composite;` → `use crate::pixel::Composite;`。
     - `crates/kivact-types/src/geometry.rs` の doc の ``[`crate::render::RenderCmd::PushTransform`]`` と ``[`crate::render::RenderCmd::PopTransform`]`` を `` `arvane_protocol::RenderCmd::PushTransform` `` と `` `arvane_protocol::RenderCmd::PopTransform` ``（リンクではない code）にする。
     - `crates/kivact-types/src/lib.rs`：
       - `pub mod render;` を消し、`pub mod shaped;` と `pub mod export;` を `pub mod semantics_text;` の前（アルファベット順の位置）に足す。
       - `pub mod process;` の行はそのまま。`mod prop_interp;` と `pub use prop_interp::{PropBasis, PropInterp, apply_components, prop_interp};` を消す。
       - `pub use render::{ InlineBox, InlineBoxPlacement, ShapedInlineText, ShapedText, ShapedTextSnapshot, TextLine, };` を `pub use shaped::{InlineBox, InlineBoxPlacement, ShapedInlineText, ShapedText, ShapedTextSnapshot, TextLine};` にする。
       - `pub use pixel::{BlendMode, ColorMatrix, Pixels};` を `pub use pixel::{BlendMode, ColorMatrix, Composite, Pixels};` にする。
       - `pub use export::{ExportFrame, ExportedFrame, FrameStats};` を足す。
       - 先頭の doc を次にする：
         ```rust
         //! Role: Pure shared UI values and small std-only synchronization helpers.
         //!
         //! The vocabulary every Kivact and Arvane crate shares: geometry, colors,
         //! ids, input, semantics, the node props a style compiles to, recorded
         //! drawing and shaped text. `cell` and `process` hold the few std-only
         //! helpers with interior mutability (latches, mailboxes, deferred
         //! values, the detach tracker). The frame protocol of Arvane is
         //! `arvane-protocol`; engine-only computation lives in the engine.
         ```
  6. `prop_interp` を engine に移す：
     ```
     git mv crates/kivact-types/src/prop_interp.rs crates/kivact-engine/src/model/state_motion/interp.rs
     perl -pi -e 's/\bcrate::/kivact_types::/g; s/^pub fn /pub(crate) fn /; s/^pub struct /pub(crate) struct /; s/^pub enum /pub(crate) enum /; s/^    pub fn /    pub(crate) fn /' crates/kivact-engine/src/model/state_motion/interp.rs
     ```
     - `crates/kivact-engine/src/model/state_motion.rs` の `mod` 宣言の並びに `pub(crate) mod interp;` を足す。
     - `crates/kivact-engine/src/model/state_motion/steps.rs` の `use kivact_types::{NodeId, PaintProps, Rgba, apply_components, prop_interp};` を `use kivact_types::{NodeId, PaintProps, Rgba};` と `use super::interp::{apply_components, prop_interp};` の 2 行にする。
     - `crates/kivact-engine/src/model/state_motion/state.rs` の `use kivact_types::{NodeId, PaintProps, PropBasis, PropInterp, Rgba, apply_components};` を `use kivact_types::{NodeId, PaintProps, Rgba};` と `use super::interp::{PropBasis, PropInterp, apply_components};` にする。
     - `crates/kivact-engine/src/model/state_motion.rs` の doc の `` `kivact_types::prop_interp` `` を `` `interp::prop_interp` `` にする。
     - `interp.rs` の field が `pub` のままでも可（`pub(crate)` の型の中の `pub` field は `unreachable_pub` の対象）。`cargo clippy` が `unreachable_pub` を報告した field は `pub(crate)` にする（`perl -pi -e 's/^    pub ([a-z_]+:)/    pub(crate) $1/' crates/kivact-engine/src/model/state_motion/interp.rs`）。
  7. 利用者の path を書き換える。対象のファイルを集めて一度に置換する：
     ```
     rg -l "kivact_types::render::" crates dev --type rust | grep -v "^crates/kivact-types/" > /tmp/render_users.txt
     xargs perl -pi -e 's/kivact_types::render::(FontSource|Glyph|GlyphRun|GlyphRunSnapshot|InlineBox|InlineBoxPlacement|ShapedInlineText|ShapedText|ShapedTextSnapshot|TextDecorationRect|TextHighlight|TextLine)\b/kivact_types::shaped::$1/g; s/kivact_types::render::(ExportFrame|ExportedFrame|FrameStats)\b/kivact_types::export::$1/g; s/kivact_types::render::Composite\b/kivact_types::Composite/g; s/kivact_types::render::/arvane_protocol::/g' < /tmp/render_users.txt
     ```
     置換の後、中括弧の `use` で「残る名前」と「移る名前」が混ざった行を手で分ける。2b8df5ac で該当するのは次の行（置換の後は `arvane_protocol::{...}` の中に残る名前がある）：

     | ファイル | 分け方（`arvane_protocol::{..}` から出す名前 → 行き先） |
     |---|---|
     | `crates/arvane-text/src/text_engine.rs` の `use arvane_protocol::{RichText, ShapedText};` | `ShapedText` → `use kivact_types::shaped::ShapedText;` |
     | `crates/arvane-text/src/output.rs` の複数行の `use arvane_protocol::{ ... FontSource, Glyph, GlyphRun, TextDecorationRect, TextHighlight, TextLine, }` | 6 つすべて → `use kivact_types::shaped::{...};` |
     | `crates/arvane-text/src/inline.rs` | `InlineBox, InlineBoxPlacement, ShapedInlineText, ShapedText` → `kivact_types::shaped`、`RichText` は protocol |
     | `crates/arvane-compose/src/plan/ops.rs` | `Composite` → `kivact_types::Composite`、`FrameStats` → `kivact_types::export::FrameStats` |
     | `crates/arvane-compose/src/plan/layers.rs` | `Composite` → `kivact_types::Composite` |
     | `crates/arvane-compose/src/plan/items.rs` | `Composite` → `kivact_types::Composite` |
     | `crates/arvane-compose/src/plan/context3d.rs` | `Composite` → `kivact_types::Composite` |
     | `crates/arvane-compose/src/plan/builder.rs` | `FrameStats` → `kivact_types::export::FrameStats` |
     | `crates/arvane-compose/src/plan/bounds.rs` | `Glyph` → `kivact_types::shaped::Glyph` |
     | `crates/arvane-compose/src/lib.rs` の `pub use arvane_protocol::{ExportFrame, ExportedFrame, FrameReport, PixelRect};` | `ExportFrame, ExportedFrame` → `pub use kivact_types::export::{ExportFrame, ExportedFrame};` |
     | `crates/arvane-render/src/draw/painter.rs` | `FontSource` → `kivact_types::shaped::FontSource` |
     | `crates/arvane-compose/src/gpu/export.rs` | `ExportFrame, ExportedFrame` → `kivact_types::export` |
     | `crates/arvane-compose/src/gpu/exec.rs` | `ExportedFrame, FrameStats` → `kivact_types::export` |
     | `crates/arvane-compose/src/cpu/draw.rs` | `Composite` → `kivact_types::Composite` |
     | `crates/arvane-compose/src/cpu/compositor.rs` | `Composite` → `kivact_types::Composite`、`ExportFrame, ExportedFrame, FrameStats` → `kivact_types::export` |
     | `crates/kivact-engine/src/lower/scene/layers.rs` | `Composite` → `kivact_types::Composite` |
     | `crates/arvane/tests/e2e/text_style.rs` | `ShapedTextSnapshot` → `kivact_types::shaped` |
     | `crates/arvane/src/engine.rs` | `FontSource, InlineBox, ShapedInlineText` → `kivact_types::shaped`、`RichText` は protocol |
     | `crates/arvane/src/presenter/frame.rs` | 両方 → `kivact_types::export` |
     | `crates/kivact-core/src/view/drawable_share.rs` | （置換で `kivact_types::export::...` にならず `arvane_protocol::{ExportFrame, ExportedFrame}` になる）→ `use kivact_types::export::{ExportFrame, ExportedFrame};` |

     上の表に無い行で compile の error（`unresolved import arvane_protocol::X`）が出たら、X が手順 1 の「types に残す」名前なら同じ規則（shaped / export / `kivact_types::Composite`）で分ける。
  8. `kivact_types` の root から protocol の名前を使っていた engine の `testing` の再公開を直す：`crates/kivact-engine/src/host/testing.rs` に `pub use kivact_types::render::...` の形が手順 7 で `arvane_protocol::` に変わっていることを確かめる（`kivact-test` は `kivact_engine::testing::SemanticsNode` を使う。変更なし）。
  9. 依存を足す。次の各 `Cargo.toml` の `[dependencies]` の `kivact-types = ...` の次の行に `arvane-protocol = { path = "../arvane-protocol", version = "0.1.0" }` を足す（手順 7 の後で `rg -l "arvane_protocol" crates/<crate>` が 1 件以上ある crate だけ）：`crates/arvane`、`crates/arvane-shell`、`crates/arvane-native`、`crates/arvane-virtual`、`crates/arvane-text`、`crates/arvane-render`、`crates/arvane-compose`、`crates/kivact-engine`。`crates/arvane-asset` は `FontSource`（types に残る）だけなので足さない。`crates/arvane/tests` が使うので `crates/arvane/Cargo.toml` には必ず足す。
  10. `dev/xtask/policy/workspace.toml`：
      - `[visible_to]` の `arvane-asset = ["arvane"]` の次の行に足す：
        ```toml
        # ----- The frame protocol: the engine builds it and the Arvane crates consume it.
        arvane-protocol = ["arvane", "arvane-shell", "arvane-native", "arvane-virtual", "arvane-text", "arvane-render", "arvane-compose", "kivact-engine"]
        ```
        `"arvane-asset"` はこの行に書かない：付属書 A の A04 が arvane-asset から protocol を使う変更と同時に、この行への `"arvane-asset"` と `crates/arvane-asset/Cargo.toml` の依存を足す（二重の編集を避ける。INC-32）。
      - `[module_rules.kivact-types.uses]` の `prop_interp = [...]` と `render =[...]` の 2 行を消し、次の 2 行を足す：`export = ["ids", "pixel"]`、`shaped = ["color", "geometry", "semantics_text", "text"]`。`scene = [...]` の中の `"render"` を `"pixel"` に置き換える（`pixel` が既にあれば `"render"` を消すだけ）。`text_shaping = ["render", "scene", "text"]` を `text_shaping = ["scene", "shaped", "text"]` にする。
      - `value_modules = [...]` の `"kivact-types::render"` を `"kivact-types::shaped", "kivact-types::export"` に置き換える。
      - `module_rules.kivact-engine`（`[module_rules.kivact-engine.uses]` の表がある場合）で `model` が `state_motion` の子を使う規則に違反が出たら、`cargo xtask check --no-write --crate kivact-engine` の出力どおり `interp` を同じ module 内の子として許す（`model` の項目に追加は不要：`state_motion::interp` は `state_motion` の子）。
  11. `crates/kivact-types/Cargo.toml` の `description` を `"Pure shared values of Kivact and Arvane: geometry, colors, ids, input, semantics, node props, recorded drawing and shaped text (no external dependencies)"` にする。
- **消すもの**：`crates/kivact-types/src/render.rs`、`crates/kivact-types/src/render/`（directory ごと）、`crates/kivact-types/src/prop_interp.rs`、`kivact_types::{PropBasis, PropInterp, apply_components, prop_interp}` の再公開。
- **状態の閉じ方**：新しい型なし。
- **試験**：追加・削除なし。`crates/arvane/tests/e2e/*` は path の書換えだけ。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-types -p arvane-protocol -p kivact-engine -p arvane -p arvane-shell -p arvane-native -p arvane-virtual -p arvane-text -p arvane-render -p arvane-compose -p arvane-asset -p kivact-core -p kivact-editing -p kivact-style -p kivact-test
  cargo check --workspace --all-targets
  cargo clippy -p kivact-types -p arvane-protocol -p kivact-engine -p arvane -p arvane-compose -p arvane-render -p arvane-text -p arvane-shell --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-types
  cargo xtask check --no-write --crate arvane-protocol
  cargo xtask check --no-write --crate kivact-engine
  cargo test -p kivact-types
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p arvane --test e2e
  ```
- **完了条件**：`rg -n "kivact_types::render|crate::render" crates dev` が 0 件。`rg -n "prop_interp" crates/kivact-types` が 0 件。`ls crates/kivact-types/src/render` が存在しない。上のコマンドが通る。

---

## S03：`Mix`（wire の層の合成）

- **wave / lane / 前提**：W2a / A / S02
- **所有**：`crates/kivact-types/src/pixel.rs`、`crates/kivact-types/src/lib.rs`、`crates/kivact-types/src/scene/recording.rs`、`crates/kivact-engine/src/lower/scene/layers.rs`、`crates/kivact-svg/src/emit.rs`、`crates/kivact-style/src/draw/layer.rs`
- **目的**：ST-17 の wire の部分。`DrawingLayer` の `blend` と `composite`（排他の二つの field）を一つの `mix: Mix` にする。style の `Layer::blend`・`Layer::composite` の公開の署名はこの unit では変えない（S21 で `Layer::mix` に置き換える）。
- **手順**：
  1. `crates/kivact-types/src/pixel.rs` の末尾（S02 で移した `pub enum Composite` の後）に足す：

```rust
/// How a layer combines with what is beneath it: as it is, with a blend
/// mode (CSS `mix-blend-mode`), or with a Porter–Duff operator (Canvas
/// `globalCompositeOperation`). One value, so a blend and an operator
/// never stand side by side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mix {
    /// Source over, normal blending.
    #[default]
    Normal,
    /// A blend mode with what is beneath.
    Blend(BlendMode),
    /// A Porter–Duff operator with what is beneath.
    Composite(Composite),
}

impl Mix {
    /// `Blend(Normal)` and `Composite(SourceOver)` are `Normal`; any other
    /// value is itself.
    pub const fn normalized(self) -> Mix {
        match self {
            Mix::Blend(BlendMode::Normal) | Mix::Composite(Composite::SourceOver) => Mix::Normal,
            other => other,
        }
    }

    /// The blend mode (`Normal` unless this is a blend).
    pub const fn blend_mode(self) -> BlendMode {
        match self {
            Mix::Blend(mode) => mode,
            Mix::Normal | Mix::Composite(_) => BlendMode::Normal,
        }
    }

    /// The Porter–Duff operator (`SourceOver` unless this is an operator).
    pub const fn composite(self) -> Composite {
        match self {
            Mix::Composite(op) => op,
            Mix::Normal | Mix::Blend(_) => Composite::SourceOver,
        }
    }
}
```

     `BlendMode` は 2b8df5ac で `#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]`、`Composite` は `Eq` を derive している。
  2. `crates/kivact-types/src/lib.rs` の `pub use pixel::{BlendMode, ColorMatrix, Composite, Pixels};` を `pub use pixel::{BlendMode, ColorMatrix, Composite, Mix, Pixels};` にする。
  3. `crates/kivact-types/src/scene/recording.rs` の `pub struct DrawingLayer` の 2 つの field（`/// Blend with what is beneath.` `pub blend: BlendMode,` と `/// Porter–Duff operator with what is beneath.` `pub composite: Composite,`）を次の 1 field に置き換える：
     ```rust
     /// How the layer combines with what is beneath (normalized: never
     /// `Blend(Normal)` or `Composite(SourceOver)`).
     pub mix: Mix,
     ```
     同じファイルの `use crate::pixel::BlendMode;` と `use crate::pixel::Composite;` を `use crate::pixel::Mix;` にする（同じファイルのほかの場所で `BlendMode` を使っていればその `use` は残す）。
  4. `crates/kivact-engine/src/lower/scene/layers.rs`：
     - `fn needs_surface` の `|| layer.composite != Composite::SourceOver` を `|| layer.mix.composite() != Composite::SourceOver` にする。
     - `blend: layer.blend,`（2 か所：`Group` を作る式と `LayerEffects` を作る式）を `blend: layer.mix.blend_mode(),` にする。
     - `composite: layer.composite,` を `composite: layer.mix.composite(),` にする。
  5. `crates/kivact-svg/src/emit.rs` の `fn layer_attributes` の最初の検査（`if layer.blend != BlendMode::Normal { return Err(SvgError::unsupported("a blend mode on a layer")); }`）を次にする（blend の書出しは S11 で足す）：
     ```rust
     if layer.mix != Mix::Normal {
         return Err(SvgError::unsupported(
             "a blend mode or a composite operation on a layer",
         ));
     }
     ```
     `use` の `BlendMode` を `Mix` に替える（ほかで使っていれば両方を import する）。
  6. `crates/kivact-style/src/draw/layer.rs`：
     - `use kivact_types::Mix;` を足す（S02 で書き換えた `use kivact_types::Composite;` は残す）。
     - `pub struct Layer` の `pub(super) blend: BlendMode,` と `pub(super) composite: Composite,` を `pub(super) mix: Mix,` の 1 行にする。
     - `Layer::new` の `blend: BlendMode::Normal,` と `composite: Composite::SourceOver,` を `mix: Mix::Normal,` にする。
     - `pub fn blend(mut self, blend: BlendMode) -> Self` の本体を `self.mix = Mix::Blend(blend).normalized();` と `self` にする。
     - `pub fn composite(mut self, composite: Composite) -> Self` の本体を `self.mix = Mix::Composite(composite).normalized();` と `self` にする。
     - group が要るかの判定の `|| self.blend != BlendMode::Normal` と `|| self.composite != Composite::SourceOver` の 2 行を `|| self.mix != Mix::Normal` にする。
     - `p::DrawingLayer { ... }` の `blend: layer.blend,` と `composite: layer.composite,` を `mix: layer.mix,` にする。
- **消すもの**：`DrawingLayer::blend`、`DrawingLayer::composite`、style `Layer` の field `blend`・`composite`。
- **状態の閉じ方**：`Mix` は enum（排他を variant で表す。S-3）。
- **試験**：変更なし。公開 E2E の `compositor.rs`（`Layer::new().composite(Composite::DestinationOut)`）、`canvas.rs`（`Layer::new().blend(..)`）がそのまま通ること。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-types -p kivact-engine -p kivact-svg -p kivact-style
  cargo check -p kivact-types -p kivact-engine -p kivact-svg -p kivact-style -p kivact-core --all-targets
  cargo clippy -p kivact-types -p kivact-engine -p kivact-svg -p kivact-style --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-types
  cargo test -p kivact-svg
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- compositor
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- canvas
  ```
- **完了条件**：`rg -n "layer\.blend\b|layer\.composite\b" crates/kivact-engine/src/lower/scene crates/kivact-svg/src crates/kivact-style/src/draw` が 0 件。上のコマンドが通る。

---

## S04：`StyleState`・`StateSet`、状態の表の索引化、状態の報告

- **wave / lane / 前提**：W2a / A / S03
- **所有**：
  - `crates/kivact-types/src/style_state.rs`（新設）、`crates/kivact-types/src/lib.rs`、`crates/kivact-types/src/paint.rs`、`dev/xtask/policy/workspace.toml`（`[module_rules.kivact-types.uses]` の 2 行）
  - `crates/kivact-engine/src/contract/events.rs`、`crates/kivact-engine/src/model/states.rs`、`crates/kivact-engine/src/model/aggregate.rs`、`Model` を構築する式のあるファイル（手順 7）、`crates/kivact-engine/src/model/apply.rs`、`crates/kivact-engine/src/model/focus/navigation.rs`、`crates/kivact-engine/src/model/pointer/machine.rs`、`crates/kivact-engine/src/model/presence/transitions.rs`
  - `crates/kivact-style/src/resolve/compile.rs`
  - `crates/kivact-core/src/launch/runtime.rs` と、手順 10 の compile error が指す core・test・devtools の `match`
- **目的**：ST-09（状態の名前を一つの enum にし、状態の表を enum で索引する）と、ST-05 の engine の部分（style が求めた状態の変化を client に報告する `HostEvent::StatesChanged`）。core は S19 まで報告を使わない。
- **手順**：
  1. `crates/kivact-types/src/style_state.rs` を作る：

```rust
//! Role: The interaction states a style names, and sets of them.

/// A state a style can name (`.hover(..)`, `.active(..)`, ...). The host
/// evaluates the eight of [`StyleState::HOST`]; the core evaluates
/// `Disabled` and `Selected`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StyleState {
    /// The pointer is over the node or a node below it (CSS `:hover`).
    Hover,
    /// The node is pressed while hovered (CSS `:active`).
    Active,
    /// The node has the focus (CSS `:focus`).
    Focus,
    /// The node has the focus and the host shows a focus indicator (CSS `:focus-visible`).
    FocusVisible,
    /// The nearest group ancestor is hovered.
    GroupHover,
    /// The focus is on the nearest group ancestor or below it.
    GroupFocus,
    /// The node is disabled.
    Disabled,
    /// The node is selected.
    Selected,
    /// The paint the node enters from (CSS `@starting-style`).
    Enter,
    /// The paint a removed node leaves to.
    Exit,
}

impl StyleState {
    /// Every state, in declaration order.
    pub const ALL: [StyleState; 10] = [
        StyleState::Hover,
        StyleState::Active,
        StyleState::Focus,
        StyleState::FocusVisible,
        StyleState::GroupHover,
        StyleState::GroupFocus,
        StyleState::Disabled,
        StyleState::Selected,
        StyleState::Enter,
        StyleState::Exit,
    ];

    /// The host-evaluated states in the order their deltas stack, the
    /// lowest first: a later one overrides what it sets.
    pub const HOST: [StyleState; 8] = [
        StyleState::Focus,
        StyleState::FocusVisible,
        StyleState::GroupFocus,
        StyleState::GroupHover,
        StyleState::Hover,
        StyleState::Active,
        StyleState::Enter,
        StyleState::Exit,
    ];

    /// The index of a host-evaluated state in [`StyleState::HOST`].
    pub const fn host_slot(self) -> Option<usize> {
        match self {
            StyleState::Focus => Some(0),
            StyleState::FocusVisible => Some(1),
            StyleState::GroupFocus => Some(2),
            StyleState::GroupHover => Some(3),
            StyleState::Hover => Some(4),
            StyleState::Active => Some(5),
            StyleState::Enter => Some(6),
            StyleState::Exit => Some(7),
            StyleState::Disabled | StyleState::Selected => None,
        }
    }

    /// The name of the style method (`"hover"`, `"active"`, `"focus_visible"`).
    pub const fn name(self) -> &'static str {
        match self {
            StyleState::Hover => "hover",
            StyleState::Active => "active",
            StyleState::Focus => "focus",
            StyleState::FocusVisible => "focus_visible",
            StyleState::GroupHover => "group_hover",
            StyleState::GroupFocus => "group_focus",
            StyleState::Disabled => "disabled",
            StyleState::Selected => "selected",
            StyleState::Enter => "enter",
            StyleState::Exit => "exit",
        }
    }

    const fn bit(self) -> u16 {
        1 << (self as u16)
    }
}

/// A set of [`StyleState`]s.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct StateSet(u16);

impl StateSet {
    /// No state.
    pub const EMPTY: StateSet = StateSet(0);

    /// This set with `state` added.
    pub const fn with(self, state: StyleState) -> StateSet {
        StateSet(self.0 | state.bit())
    }

    /// Add `state`.
    pub fn insert(&mut self, state: StyleState) {
        self.0 |= state.bit();
    }

    /// Whether `state` is in the set.
    pub const fn contains(self, state: StyleState) -> bool {
        self.0 & state.bit() != 0
    }

    /// Whether no state is in the set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The states in both sets.
    pub const fn intersection(self, other: StateSet) -> StateSet {
        StateSet(self.0 & other.0)
    }

    /// The states in either set.
    pub const fn union(self, other: StateSet) -> StateSet {
        StateSet(self.0 | other.0)
    }

    /// The states in the set, in declaration order.
    pub fn iter(self) -> impl Iterator<Item = StyleState> {
        StyleState::ALL
            .into_iter()
            .filter(move |state| self.contains(*state))
    }
}
```

  2. `crates/kivact-types/src/lib.rs` に `pub mod style_state;` と `pub use style_state::{StateSet, StyleState};` を足す（`pub mod semantics_text;` の後）。`dev/xtask/policy/workspace.toml` の `[module_rules.kivact-types.uses]` に `style_state = []` を足し、`paint = [...]` の配列に `"style_state"` を足す。
  3. `crates/kivact-types/src/paint.rs` の `pub struct StateStyles { ... }` とその `impl StateStyles { pub fn deltas(..) }` を次に置き換える。`const _: () = assert!(std::mem::size_of::<StateStyles>() <= 80, ...)` はそのまま（8 個の `Option<Box<_>>` と bool と u16 で 72 byte）：

```rust
/// Host-evaluated state styles. `disabled` and `selected` are resolved by the
/// core because the core owns those states.
// xtask-allow(state): passive record of one compiled style: compile_style builds it with set_delta and the host only reads it
#[derive(Clone, Debug, PartialEq, Default)]
pub struct StateStyles {
    /// The paint deltas of the host states, indexed by
    /// [`StyleState::host_slot`](crate::StyleState::host_slot).
    pub deltas: [Option<Box<PaintDelta>>; 8],
    /// The node is a group for the `group_hover` and `group_focus` of the
    /// nodes below it.
    pub group: bool,
    /// The host states whose activation the host reports to the client
    /// (`HostEvent::StatesChanged`): the states whose style sets more than
    /// paint, which the core compiles into the node's style.
    pub report: crate::StateSet,
}

impl StateStyles {
    /// The paint delta of `state`, if the style sets one.
    pub fn delta(&self, state: crate::StyleState) -> Option<&PaintDelta> {
        state.host_slot().and_then(|i| self.deltas[i].as_deref())
    }

    /// Set the paint delta of a host state (`Disabled` and `Selected` are
    /// not host states; setting them does nothing).
    pub fn set_delta(&mut self, state: crate::StyleState, delta: Option<Box<PaintDelta>>) {
        if let Some(i) = state.host_slot() {
            self.deltas[i] = delta;
        }
    }

    /// Whether the host must evaluate `state` for this node: it has a delta,
    /// or the core asked for its reports.
    pub fn wants(&self, state: crate::StyleState) -> bool {
        self.delta(state).is_some() || self.report.contains(state)
    }

    /// Every delta it sets.
    pub fn deltas(&self) -> impl Iterator<Item = &PaintDelta> {
        self.deltas.iter().flatten().map(|d| &**d)
    }
}
```

  4. `crates/kivact-engine/src/contract/events.rs` の `pub enum HostEvent` の `HoverChanged { .. },` の後に足す：
     ```rust
     /// The host states of a node whose style asked for them
     /// (`StateStyles::report`) changed: the reported states active now.
     /// Sent once with the empty set when the node stops asking.
     StatesChanged {
         node: NodeId,
         states: kivact_types::StateSet,
     },
     ```
     enum の doc の「focus and hover (FocusChanged, FocusResult, HoverChanged)」の括弧の中に `StatesChanged` を足し、「focus and hover」を「focus, hover and the reported states」にする。
  5. engine の読み手を直す（各ファイルの先頭に `use kivact_types::StyleState;` を足す）：

     | ファイル（場所） | 変更前 | 変更後 |
     |---|---|---|
     | `model/focus/navigation.rs`（focus の対象を描き直す loop の `if`） | `(s.focus.is_some() \|\| s.focus_visible.is_some()) && crate::tree::states_place(s)` | `(s.wants(StyleState::Focus) \|\| s.wants(StyleState::FocusVisible)) && crate::tree::states_place(s)` |
     | `model/focus/navigation.rs`（2 か所） | `\|s\| s.group_focus.is_some()` | `\|s\| s.wants(StyleState::GroupFocus)` |
     | `model/states.rs`（`fn set_states`） | `\|s\| s.group_hover.is_some() \|\| s.group_focus.is_some()` | `\|s\| s.wants(StyleState::GroupHover) \|\| s.wants(StyleState::GroupFocus)` |
     | `model/pointer/machine.rs` | `\|s\| s.group_hover.is_some()` | `\|s\| s.wants(StyleState::GroupHover)` |
     | `model/pointer/machine.rs`（`for (id, hover, press) in flipped`） | `(hover && s.hover.is_some()) \|\| (press && s.pressed.is_some())` | `(hover && s.wants(StyleState::Hover)) \|\| (press && s.wants(StyleState::Active))` |
     | `model/presence/transitions.rs`（3 か所） | `s.enter.is_some()` / `s.exit.is_some()` | `s.delta(StyleState::Enter).is_some()` / `s.delta(StyleState::Exit).is_some()` |

     `s.group` の読み（`model/pointer/hover.rs`、`model/states.rs`）と `s.deltas()` の呼出しは変えない。
  6. `crates/kivact-engine/src/model/states.rs` の `fn deltas(&self, id: NodeId, entering: bool) -> Vec<&PaintDelta>` を次に置き換え、同じ `impl Model` に `active_states` と `report_states` を足す。ファイルの先頭に `use kivact_types::{StateSet, StyleState};` と `use crate::contract::HostEvent;` を足す：

```rust
    /// The state deltas of `id` that apply now, lowest first; with `entering`,
    /// its enter paint too.
    fn deltas(&self, id: NodeId, entering: bool) -> Vec<&PaintDelta> {
        let Some(s) = self.tree.get(id).and_then(|n| n.states()) else {
            return Vec::new();
        };
        let mut wanted = StateSet::EMPTY;
        for state in StyleState::HOST {
            if s.delta(state).is_some() {
                wanted.insert(state);
            }
        }
        let active = self.active_states(id, entering, wanted);
        StyleState::HOST
            .into_iter()
            .filter(|state| active.contains(*state))
            .filter_map(|state| s.delta(state))
            .collect()
    }

    /// The states among `wanted` that `id` is in now. Only those are
    /// evaluated (a group lookup walks the ancestors).
    pub(crate) fn active_states(&self, id: NodeId, entering: bool, wanted: StateSet) -> StateSet {
        let me = Some(kivact_types::FocusTarget {
            node: id,
            region: None,
        });
        let mut active = StateSet::EMPTY;
        for state in StyleState::HOST {
            if !wanted.contains(state) {
                continue;
            }
            let on = match state {
                StyleState::Focus => self.focused() == me,
                StyleState::FocusVisible => self.focus.visible() && self.focused() == me,
                StyleState::GroupFocus => self.is_group_focused(id),
                StyleState::GroupHover => self.is_group_hovered(id),
                StyleState::Hover => self.is_hovered(id),
                StyleState::Active => self.is_pressed(id),
                StyleState::Enter => entering,
                StyleState::Exit => self.tree.leaving(id),
                StyleState::Disabled | StyleState::Selected => false,
            };
            if on {
                active.insert(state);
            }
        }
        active
    }

    /// Tell the client the states `id` reports (`StateStyles::report`)
    /// when they differ from the ones told last (`Model::reported_states`).
    pub(crate) fn report_states(&mut self, id: NodeId) {
        let report = self
            .tree
            .get(id)
            .and_then(|n| n.states())
            .map_or(StateSet::EMPTY, |s| s.report);
        let now = if report.is_empty() {
            StateSet::EMPTY
        } else {
            self.active_states(id, false, report)
        };
        let told = self.reported_states.get(&id).copied().unwrap_or_default();
        if now == told {
            return;
        }
        if now.is_empty() {
            self.reported_states.remove(&id);
        } else {
            self.reported_states.insert(id, now);
        }
        self.events.push(HostEvent::StatesChanged { node: id, states: now });
    }
```

     同じファイルの `fn set_states` の最後（`changed` を返す直前、`for x in members { self.restyled(x); }` の後）に `self.report_states(id);` を足す。`pub(crate) fn restyled(&mut self, id: NodeId)` の本体の最後に `self.report_states(id);` を足す。
  7. `crates/kivact-engine/src/model/aggregate.rs` の `pub(crate) struct Model` の `pub(crate) motion: super::state_motion::state::StateMotion,` の次に足す：
     ```rust
     /// The reported states last told to the client per node
     /// (`HostEvent::StatesChanged`), the dedupe cache of S-9. A node is in
     /// it only while the set told is not empty.
     pub(crate) reported_states: std::collections::HashMap<NodeId, kivact_types::StateSet>,
     ```
     `Model` を構築する式（`rg -n "motion: " crates/kivact-engine/src/model --type rust` のうち `Model {` の中の行）に `reported_states: std::collections::HashMap::new(),` を足す。
  8. 報告を呼ぶ場所を足す：
     - `model/focus/navigation.rs`：`for n in [old_target, target].into_iter().flatten().map(|t| t.node) {` の中の `self.repaint(n);` の直後に `self.report_states(n);`。
     - `model/pointer/machine.rs`：`for (id, hover, press) in flipped {` の本体の先頭に `self.report_states(id);`。
     - `model/apply.rs` の `fn forget_sources(&mut self, id: NodeId)` の本体に `self.reported_states.remove(&id);`。
  9. `crates/kivact-style/src/resolve/compile.rs` の `fn state_styles` の `.then(|| StateStyles { ... })` を次に置き換える（その前の `(states.hover.is_some() || ...)` の条件は変えない）。`use kivact_types::{...}` に `StyleState` を足す：

```rust
    .then(|| {
        let mut out = StateStyles {
            group: states.group,
            ..StateStyles::default()
        };
        let delta = |h: &Style| Some(Box::new(paint_delta(s, h, theme)));
        out.set_delta(StyleState::Hover, states.hover.as_deref().and_then(delta));
        out.set_delta(StyleState::Active, states.pressed.as_deref().and_then(delta));
        out.set_delta(StyleState::Focus, states.focus.as_deref().and_then(delta));
        out.set_delta(StyleState::GroupHover, states.group_hover.as_deref().and_then(delta));
        out.set_delta(StyleState::GroupFocus, states.group_focus.as_deref().and_then(delta));
        out.set_delta(StyleState::Enter, states.enter.as_deref().and_then(delta));
        out.set_delta(StyleState::Exit, states.exit.as_deref().and_then(delta));
        // Over the focus state, as the host applies it.
        out.set_delta(
            StyleState::FocusVisible,
            states.focus_visible.as_deref().map(|h| {
                let focused = match &states.focus {
                    Some(focus) => s.clone().merge(focus),
                    None => s.clone(),
                };
                Box::new(paint_delta(&focused, h, theme))
            }),
        );
        out
    })
```

  10. （この手順の `match` の直しが `crates/kivact-core/src/devtools/console.rs`（付属書 R の R04）か `crates/kivact-devtools/`（付属書 K の K01）に及ぶ場合は STOP して報告する。INC-23）`cargo check -p kivact-core -p kivact-test -p kivact-devtools --all-targets` を実行する。`non-exhaustive patterns` の error が `HostEvent::StatesChanged` について出た各 `match` に arm を足す：`crates/kivact-core/src/launch/runtime.rs` の event を文字列にする `match` には `E::StatesChanged { node, states } => format!("StatesChanged n{} {states:?}", node.0),`。それ以外の `match` には `HostEvent::StatesChanged { .. } => {}` と、その直前の行に `// The core reads it from ST-05 on (annex S, S19).`。`_ =>` を持つ `match` は変えない。
- **消すもの**：`StateStyles` の field `hover`、`pressed`、`focus`、`focus_visible`、`group_hover`、`group_focus`、`enter`、`exit`。
- **状態の閉じ方**：`StateStyles` は手順 3 の印の 1 行。`StateSet` は field 一つ。`Model.reported_states` は map の 1 field（6.1 の数える種類ではない）。
- **試験**：新しい試験は S19 で足す（core が報告を使う時点で公開の契約になる）。engine の既存の試験と、公開 E2E の `hover.rs`・`focus.rs`・`pointer.rs`・`motion.rs` がそのまま通ること。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-types -p kivact-engine -p kivact-style -p kivact-core
  cargo check -p kivact-types -p kivact-engine -p kivact-style -p kivact-core -p kivact-test -p kivact-devtools --all-targets
  cargo clippy -p kivact-types -p kivact-engine -p kivact-style -p kivact-core --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-types
  cargo xtask check --no-write --crate kivact-engine
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-engine
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- hover
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- focus
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- pointer
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- motion
  ```
- **完了条件**：`rg -n "\.(hover|pressed|focus_visible|group_hover|group_focus)\.(is_some|as_deref)\(\)" crates/kivact-engine/src` が 0 件。上のコマンドが通る。

---

## S05：engine と SVG が共有する幾何と gradient の解釈

- **wave / lane / 前提**：W2a / A / S04
- **所有**：
  - `crates/kivact-types/src/scene/resolve.rs`（新設）、`crates/kivact-types/src/scene.rs`、`crates/kivact-types/src/gradient_stops.rs`（新設）、`crates/kivact-types/src/lib.rs`、`dev/xtask/policy/workspace.toml`（1 行）
  - `crates/kivact-engine/src/lower.rs`、`lower/shape.rs`、`lower/scene_shape.rs`（削除）、`lower/path.rs`（削除）、`lower/fill.rs`、`lower/scene/geometry.rs`、`lower/scene/layers.rs`、`tree/node.rs`、`model/drawables/place.rs`、`paint/frame_scene.rs`、`paint/list/ink.rs`（すべて `crates/kivact-engine/src/` の下）
  - `crates/arvane-protocol/src/paint.rs`
  - `crates/kivact-svg/src/shape.rs`、`crates/kivact-svg/src/fill.rs`、`crates/kivact-svg/src/emit.rs`、`crates/kivact-svg/src/document.rs`（試験）
- **目的**：ST-13・ST-14・ST-16。CSS の形の輪郭、角の半径、path の平行移動、塗りの範囲、gradient の stop の展開（補間の色空間と color hint）を types の純粋関数一つにし、engine と SVG の両方が呼ぶ。SVG の出力が window と同じ幾何・同じ stop になる。kivact-svg は crate のまま（入力 `DrawList`、出力 `String`）。
- **手順**：
  1. `crates/kivact-types/src/scene/resolve.rs` を作り、`crates/kivact-types/src/scene.rs` に `mod resolve;` を足す（中身は既存の型の `impl` なので再公開は不要）：

```rust
//! Role: Resolve recorded draw geometry into outlines and paint areas, shared by the engine and the SVG writer.

use crate::geometry::{Point, Rect};
use crate::paint::{ClipShape, Corners, Len};
use crate::scene::{DrawingShape, FillRule, PathCmd, Radius, RoundedRect, Shape, ShapePaint};

fn positive(v: f64) -> f64 {
    if v.is_finite() { v.max(0.0) } else { 0.0 }
}

fn rule(even_odd: bool) -> FillRule {
    if even_odd { FillRule::EvenOdd } else { FillRule::NonZero }
}

impl Corners {
    /// The four radii in `rect` (top left, top right, bottom right, bottom
    /// left): percentages of its width (horizontal) and height (vertical),
    /// never negative.
    pub fn radii_in(&self, rect: Rect) -> [Radius; 4] {
        self.each().map(|cr| Radius {
            x: positive(cr.x.of(rect.width)),
            y: positive(cr.y.of(rect.height)),
        })
    }

    /// The box `rect` with these corners; a plain rect when none is rounded.
    pub fn shape_in(&self, rect: Rect) -> Shape {
        if self.is_zero() {
            return Shape::Rect(rect);
        }
        let radii = self.radii_in(rect);
        if radii.iter().all(|r| r.x == r.y) {
            Shape::RoundedRect(RoundedRect::with_corners(rect, radii.map(|r| r.x)))
        } else {
            Shape::RoundedRect(RoundedRect { rect, radii })
        }
    }
}

impl ClipShape {
    /// The CSS basic shape in the box `rect` (`clip-path` geometry).
    pub fn in_rect(&self, rect: Rect) -> Shape {
        let at = |[x, y]: [Len; 2]| Point {
            x: rect.x + x.of(rect.width),
            y: rect.y + y.of(rect.height),
        };
        let ellipse = |c: Point, rx: f64, ry: f64| {
            Shape::RoundedRect(RoundedRect::ellipse(Rect {
                x: c.x - rx,
                y: c.y - ry,
                width: 2.0 * rx,
                height: 2.0 * ry,
            }))
        };
        match self {
            ClipShape::Path { cmds, even_odd } => Shape::Path {
                cmds: cmds.clone(),
                rule: rule(*even_odd),
            }
            .translated(Point { x: rect.x, y: rect.y }),
            ClipShape::Inset { edges, radius } => {
                let (left, top) = (edges.left.of(rect.width), edges.top.of(rect.height));
                let inner = Rect {
                    x: rect.x + left,
                    y: rect.y + top,
                    width: (rect.width - left - edges.right.of(rect.width)).max(0.0),
                    height: (rect.height - top - edges.bottom.of(rect.height)).max(0.0),
                };
                radius.shape_in(inner)
            }
            ClipShape::Circle { radius, center } => {
                let reference = rect.width.hypot(rect.height) / std::f64::consts::SQRT_2;
                let r = positive(radius.of(reference));
                ellipse(at(*center), r, r)
            }
            ClipShape::Ellipse { radius_x, radius_y, center } => ellipse(
                at(*center),
                positive(radius_x.of(rect.width)),
                positive(radius_y.of(rect.height)),
            ),
            ClipShape::Polygon { points, even_odd } => Shape::Polygon {
                points: points.iter().map(|p| at(*p)).collect(),
                rule: rule(*even_odd),
            },
        }
    }
}

impl Shape {
    /// The outline moved by `by`, as a path (arcs become curves).
    pub fn translated(self, by: Point) -> Shape {
        let at = |p: Point| Point { x: p.x + by.x, y: p.y + by.y };
        let rule = self.rule();
        Shape::Path {
            rule,
            cmds: self
                .outline()
                .into_iter()
                .map(|cmd| match cmd {
                    PathCmd::MoveTo(p) => PathCmd::MoveTo(at(p)),
                    PathCmd::LineTo(p) => PathCmd::LineTo(at(p)),
                    PathCmd::QuadTo(p, q) => PathCmd::QuadTo(at(p), at(q)),
                    PathCmd::CubicTo(p, q, r) => PathCmd::CubicTo(at(p), at(q), at(r)),
                    other => other,
                })
                .collect(),
        }
    }
}

impl DrawingShape {
    /// The shape as one outline in the recording's coordinates.
    pub fn to_shape(&self) -> Shape {
        match self {
            DrawingShape::Basic { shape, rect } => shape.in_rect(*rect),
            DrawingShape::Rounded { rect, corners } => corners.shape_in(*rect),
            DrawingShape::Path { path, rule } => Shape::Path {
                cmds: path.cmds.clone(),
                rule: *rule,
            },
        }
    }
}

impl ShapePaint {
    /// The rect a gradient or image paint of this shape spans, in the
    /// recording's coordinates: a CSS shape's reference box or a path's
    /// bounds, padded by half the stroke width where a stroked outline has
    /// no width or no height (a straight line).
    pub fn paint_area(&self) -> Rect {
        let mut area = match &self.shape {
            DrawingShape::Basic { rect, .. } | DrawingShape::Rounded { rect, .. } => *rect,
            DrawingShape::Path { .. } => self.shape.to_shape().bounds(),
        };
        if let Some(stroke) = &self.stroke {
            let pad = stroke.width.max(0.0) / 2.0;
            if area.width == 0.0 {
                area.x -= pad;
                area.width = 2.0 * pad;
            }
            if area.height == 0.0 {
                area.y -= pad;
                area.height = 2.0 * pad;
            }
        }
        area
    }
}
```

     `use crate::scene::{...}` の名前が `crate::scene` から公開されていない場合は、`crates/kivact-types/src/scene.rs` の `pub use` に書かれた元の module（`crate::scene::shape::Shape`、`crate::scene::recording::DrawingShape` など）の path にする。使う関数（`Shape::bounds`・`rule`・`outline`、`RoundedRect::with_corners`・`ellipse`、`Corners::each`・`is_zero`、`Len::of`）は 2b8df5ac で pub。
  2. `crates/kivact-types/src/gradient_stops.rs` を作り、engine の `crates/kivact-engine/src/lower/fill.rs` から次を **移す**（本体はそのまま。表の右の名前と公開にする）：

     | engine の元 | types の新しい形 |
     |---|---|
     | `fn fraction(l: Len, length: f64) -> f64` | private のまま |
     | `fn stops(g: &Gradient, length: f64) -> Vec<(f64, Rgba)>` | `impl Gradient { pub fn srgb_stops(&self, line_length: f64) -> Vec<(f64, Rgba)> }`（`g.` を `self.`、`length` を `line_length`） |
     | `pub(crate) fn expand_gradient(..)` | `pub fn expand_gradient(stops: &[(f64, SpaceColor)], hints: &[Option<f64>], how: ColorInterpolation) -> Vec<(f64, Rgba)>` |
     | `fn expand_interval`、`fn refine`、`fn within_step`、`const MAX_DEPTH` | private のまま |
     | `fn color_at(stops: &[(f64, Rgba)], t: f64) -> Rgba` | `pub fn color_at`（署名は同じ） |
     | `fn normalized(stops: &[(f64, Rgba)], lo: f64, hi: f64) -> Vec<(f64, Rgba)>` | `pub fn normalized_stops`（署名は同じ） |

     ファイルの先頭：
     ```rust
     //! Role: Expand a CSS gradient's authored stops into sRGB stops, as the engine and the SVG writer draw them.
     use crate::color::Rgba;
     use crate::color_space::{ColorInterpolation, SpaceColor, interpolate};
     use crate::geometry::{Point, Rect};
     use crate::paint::{Gradient, GradientKind, Len};
     ```
     同じファイルに linear の線を足す（engine の `GradientShape::linear_angle(rect, angle - 90.0)` と同じ計算）：
     ```rust
     impl Gradient {
         /// The line of a linear gradient over `rect` (CSS: 0 points up, 90
         /// right; long enough for the corners to take the end colors) as
         /// its start and end; `None` for another kind. A non-finite angle
         /// is 180 (downwards).
         pub fn line(&self, rect: Rect) -> Option<(Point, Point)> {
             let GradientKind::Linear { angle_deg } = self.kind else {
                 return None;
             };
             let angle = if angle_deg.is_finite() { angle_deg } else { 180.0 };
             let (dy, dx) = (angle - 90.0).to_radians().sin_cos();
             let span = rect.width * dx.abs() + rect.height * dy.abs();
             let (cx, cy) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
             Some((
                 Point { x: cx - dx * span / 2.0, y: cy - dy * span / 2.0 },
                 Point { x: cx + dx * span / 2.0, y: cy + dy * span / 2.0 },
             ))
         }
     }
     ```
     `crates/kivact-types/src/lib.rs` に `pub mod gradient_stops;` と `pub use gradient_stops::{color_at, expand_gradient, normalized_stops};` を足す。`dev/xtask/policy/workspace.toml` の `[module_rules.kivact-types.uses]` に `gradient_stops = ["color", "color_space", "geometry", "paint"]` を足し、`scene = [...]` に `"paint"` が無ければ足す（`resolve.rs` が `paint::ClipShape` を使う）。
  3. engine を直す：
     - `lower/fill.rs`：手順 2 で移したものを消す。`stops(g, X)` の 4 か所を `g.srgb_stops(X)`、`normalized(` を `kivact_types::normalized_stops(`、`color_at(` を `kivact_types::color_at(`、`expand_gradient(` を `kivact_types::expand_gradient(` にする。`fn gradient` の linear の分岐の `let angle = ...;` と `let GradientShape::Linear { start, end } = GradientShape::linear_angle(rect, angle - 90.0) else { return None; };` を `let Some((start, end)) = g.line(rect) else { return None; };` にし、`GradientKind::Linear { angle_deg }` を `GradientKind::Linear { .. }` にする。
     - `crates/arvane-protocol/src/paint.rs`：`impl GradientShape { pub fn linear_angle(..) }` を消す（`rg -n linear_angle crates` が 0 件になること）。
     - `lower/shape.rs`：`pub(crate) fn clip_path(shape: &ClipShape, rect: Rect) -> Shape` を消す。`fn in_clip_path` の `clip_path(shape, rect)` を `shape.in_rect(rect)` にする。`BoxShape::of` の `let radii = radius.each().map(|cr| Radius { x: positive(cr.x.of(rect.width)), y: positive(cr.y.of(rect.height)) });` を `let radii = radius.radii_in(rect);` にする。`paint/frame_scene.rs` の `.map(|s| shape::clip_path(s, bounds))` を `.map(|s| s.in_rect(bounds))` にする。
     - `git rm crates/kivact-engine/src/lower/scene_shape.rs crates/kivact-engine/src/lower/path.rs`、`lower.rs` の `pub(crate) mod path;` と `pub(crate) mod scene_shape;` を消す。呼出しの置換：`crate::lower::scene_shape::shape(X)` と `scene_shape::shape(X)` → `X.to_shape()`（X は `&DrawingShape`。`&` 付きの式 `&painted.shape` は `painted.shape.to_shape()`）、`crate::lower::path::shifted(S, P)` と `path::shifted(S, P)` → `S.translated(P)`。対象：`tree/node.rs`、`model/drawables/place.rs`、`paint/list/ink.rs`、`lower/scene/layers.rs`（2 か所）、`lower/scene/geometry.rs`。各ファイルの `use` の `scene_shape`・`path` を消す。
     - `lower/scene/geometry.rs` の `fn paint_shape` の `let mut area = match &painted.shape { ... };` から `if let Some(stroke) = &painted.stroke { ... }` の終わりまでを次に置き換える：
       ```rust
       let local = painted.paint_area();
       let area = Rect {
           x: origin.x + local.x,
           y: origin.y + local.y,
           width: local.width,
           height: local.height,
       };
       ```
  4. SVG を直す：
     - `crates/kivact-svg/src/shape.rs`：`fn outline_of` を次にし、`fn basic`、`fn rounded`、`fn ellipse`、`fn shift`、`fn bounds_of` を消す（`ARC_PIECE_DEG`、`fn arc_pieces`、`fn path_data` 以下と試験は残す）：
       ```rust
       /// The path commands and fill rule of a drawn shape, the outline the
       /// window fills (`DrawingShape::to_shape`).
       pub(super) fn outline_of(shape: &DrawingShape) -> (Vec<PathCmd>, FillRule) {
           let outline = shape.to_shape();
           let rule = outline.rule();
           match outline {
               Shape::Path { cmds, .. } => (cmds, rule),
               other => (other.outline(), rule),
           }
       }
       ```
     - `crates/kivact-svg/src/emit.rs`：`fn outline_clip` の `let (cmds, rule, _) = outline_of(clip);`（または bounds を捨てる形）を `let (cmds, rule) = outline_of(clip);` に。`fn shape(&mut self, shape: &ShapePaint)` の `let (cmds, rule, bounds) = outline_of(&shape.shape);` を `let (cmds, rule) = outline_of(&shape.shape);` と `let bounds = shape.paint_area();` にする（以後の `bounds` は塗りに使う）。
     - `crates/kivact-svg/src/fill.rs`：`fn css_stops` を消す。`fn gradient_paint` の `GradientKind::Linear { angle_deg } => { ... }` の分岐全体を次に置き換える：
       ```rust
       GradientKind::Linear { .. } => {
           let Some((start, end)) = gradient.line(bounds) else {
               return Err(SvgError::unsupported("a linear gradient without a line"));
           };
           let length = (end.x - start.x).hypot(end.y - start.y);
           let stops = gradient.srgb_stops(length);
           let (Some(first), Some(last)) = (stops.first(), stops.last()) else {
               return Ok(format!("{kind}=\"none\""));
           };
           let (lo, hi) = (first.0.min(0.0), last.0.max(1.0));
           let at = |t: f64| Point {
               x: start.x + (end.x - start.x) * t,
               y: start.y + (end.y - start.y) * t,
           };
           let stops = kivact_types::normalized_stops(&stops, lo, hi);
           let id = emitter.id("grad");
           linear_def(emitter, &id, at(lo), at(hi), Extend::Pad, &stops);
           Ok(format!("{kind}=\"url(#{id})\""))
       }
       ```
       radial の分岐の `let stops = css_stops(&gradient.stops, radius);` を次にする：
       ```rust
       let stops: Vec<(f64, Rgba)> = gradient
           .srgb_stops(radius)
           .into_iter()
           .map(|(t, c)| (t.clamp(0.0, 1.0), c))
           .collect();
       ```
       `let Rect { x, y, width: w, height: h } = bounds;` の束縛が linear で未使用になっても radial が使うので残す。未使用の `use`（`GradientStop` など）は `cargo fix` で消す。
- **消すもの**：engine の `lower/scene_shape.rs`、`lower/path.rs`、`lower/shape.rs` の `clip_path`、`lower/fill.rs` の `fraction`・`stops`・`expand_gradient`・`expand_interval`・`refine`・`within_step`・`color_at`・`normalized`・`MAX_DEPTH`；protocol の `GradientShape::linear_angle`；svg の `basic`・`rounded`・`ellipse`・`shift`・`bounds_of`・`css_stops`。
- **状態の閉じ方**：新しい型なし。
- **試験**：
  - `crates/kivact-engine/src/lower/fill.rs` の `#[cfg(test)]` に stop の展開の試験（`rg -n "#\[test\]" -A1 crates/kivact-engine/src/lower/fill.rs` で見つかるもののうち `stops(`・`expand_gradient(` を呼ぶもの）があれば、関数名を変えずに `crates/kivact-types/src/gradient_stops.rs` の `#[cfg(test)] mod tests`（`//! Role: Tests of gradient stop expansion.`）へ移す（`stops(&g, L)` は `g.srgb_stops(L)`）。
  - `crates/kivact-svg/src/document.rs` の `mod tests` に `fn a_gradient_interpolates_in_oklab_like_the_window()` を足す：stop 2 つ（赤 `Rgba{r:1,g:0,b:0,a:1}` と青 `Rgba{r:0,g:0,b:1,a:1}`、`interpolation: ColorInterpolation::DEFAULT`、`hints: vec![None]`）の linear `Gradient`（`angle_deg: 90.0`）で塗った `DrawCmd::Rect` を `write_list` で書き、出力の `"<stop "` の出現数が 2 より多いことを assert する（Oklab の展開で stop が足される。2b8df5ac の writer は 2 だった）。`Gradient` の他の field は `Gradient` の `Default` があれば `..Default::default()`、無ければ同じファイルの既存の試験と同じ組み立て方をする。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-types -p kivact-engine -p kivact-svg -p arvane-protocol
  cargo check -p kivact-types -p kivact-engine -p kivact-svg -p arvane-protocol -p kivact-core --all-targets
  cargo clippy -p kivact-types -p kivact-engine -p kivact-svg -p arvane-protocol --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-types
  cargo xtask check --no-write --crate kivact-engine
  cargo xtask check --no-write --crate kivact-svg
  cargo test -p kivact-types
  cargo test -p kivact-svg
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- offscreen_svg
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- paint
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- borders_backgrounds
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- canvas
  ```
- **完了条件**：`rg -n "scene_shape|lower::path|fn css_stops|fn clip_path\(|linear_angle" crates` が 0 件。上のコマンドが通る。

---

## S06：protocol と engine の内部の改名

- **wave / lane / 前提**：W2a / A / S05
- **所有**：`crates/arvane-protocol/`、`crates/kivact-engine/`、`crates/arvane*/`（`.rs` の名前の置換だけ）
- **目的**：ST-21 のうち protocol と engine の同名の異義と、ST-11 (b) の render 側の span の名前。公開の名前は変わらない。

  | 旧 | 新 |
  |---|---|
  | `arvane_protocol::Gradient` | `arvane_protocol::RasterGradient` |
  | `arvane_protocol::Paint` | `arvane_protocol::RasterPaint` |
  | `arvane_protocol::SpanStyle` | `arvane_protocol::ResolvedSpanStyle` |
  | `arvane_protocol::TextSpan` | `arvane_protocol::ResolvedSpan` |
  | engine の `Look`（`crates/kivact-engine/src/paint/list/look.rs` の型） | `LayerTraits` |

- **手順**：
  1. 次の perl script を作業ディレクトリの外（scratch）に `rename_protocol.pl` として保存する：

```perl
#!/usr/bin/perl
# usage: perl rename_protocol.pl FILE...
use strict;
use warnings;
my %map = (
    Gradient  => 'RasterGradient',
    Paint     => 'RasterPaint',
    SpanStyle => 'ResolvedSpanStyle',
    TextSpan  => 'ResolvedSpan',
);
for my $f (@ARGV) {
    local $/;
    open my $in, '<', $f or die "$f: $!";
    my $s = <$in>;
    close $in;
    my %bare;
    while ($s =~ /use\s+arvane_protocol::\{([^}]*)\}/g) {
        my $list = $1;
        for my $n (keys %map) { $bare{$n} = 1 if $list =~ /\b$n\b/; }
    }
    while ($s =~ /use\s+arvane_protocol::(\w+)\s*;/g) {
        $bare{$1} = 1 if exists $map{$1};
    }
    for my $n (keys %map) { $s =~ s/arvane_protocol::$n\b/arvane_protocol::$map{$n}/g; }
    for my $n (keys %bare) { $s =~ s/(?<!::)\b$n\b/$map{$n}/g; }
    open my $out, '>', $f or die "$f: $!";
    print $out $s;
    close $out;
}
```

  2. protocol の中（定義と利用）：
     ```
     perl -pi -e 's/(?<!::)\bGradient\b/RasterGradient/g; s/(?<!::)\bPaint\b/RasterPaint/g; s/(?<!::)\bSpanStyle\b/ResolvedSpanStyle/g; s/(?<!::)\bTextSpan\b/ResolvedSpan/g' crates/arvane-protocol/src/*.rs
     ```
  3. 利用者：
     ```
     rg -l "arvane_protocol" crates dev --type rust | grep -v "^crates/arvane-protocol/" | xargs perl <scratch>/rename_protocol.pl
     ```
     （`<scratch>` は手順 1 で保存した場所。）script は、ファイルが protocol から bare で import した名前だけをそのファイルの中で置き換える。同じファイルが `kivact_types` の同名の型（`Gradient`、`TextSpan`）を bare で使うことは Rust の名前の衝突で不可能なので、置換は正しい。
  4. engine の `Look`：
     ```
     rg -l "\bLook\b" crates/kivact-engine/src | xargs perl -pi -e 's/\bLook\b/LayerTraits/g'
     ```
     `git diff crates/kivact-engine | rg "^\+.*//.*LayerTraits"` でコメントの英単語 "Look" を置換した行があれば、型を指していない文だけ元に戻す。
  5. `cargo check --workspace --all-targets` を通す。
- **消すもの**：上の表の旧名。
- **状態の閉じ方**：新しい型なし。
- **試験**：名前の置換だけ。
- **確認（tier a）**：
  ```
  cargo fmt -p arvane-protocol -p kivact-engine -p arvane -p arvane-render -p arvane-compose -p arvane-text -p arvane-shell -p arvane-native -p arvane-virtual
  cargo check --workspace --all-targets
  cargo clippy -p arvane-protocol -p kivact-engine -p arvane -p arvane-render -p arvane-compose -p arvane-text --all-targets -- -D warnings
  cargo xtask check --no-write --crate arvane-protocol
  cargo xtask check --no-write --crate kivact-engine
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p arvane --test e2e
  ```
- **完了条件**：`rg -n "arvane_protocol::(Gradient|Paint|SpanStyle|TextSpan)\b" crates dev` が 0 件。上のコマンドが通る。

---

## S07：shader の検証を GPU の build だけに（`shader-validation`）

- **wave / lane / 前提**：W2a / S / なし
- **所有**：`crates/kivact-style/Cargo.toml`、`crates/kivact-style/src/shader/program.rs`、`crates/kivact-style/src/shader/data.rs`、`crates/kivact-shader/src/error.rs`、`crates/kivact-shader/src/assemble.rs`（doc の例）、`crates/kivact-shader/src/source.rs`（`ShaderError` の構築）、`crates/kivact-core/Cargo.toml`（`[features]`）、`crates/kivact/Cargo.toml`（`[features]`）、`dev/tests/kivact-public-api/tests/*/shaders.rs`
- **目的**：ST-25。GPU の無い build で naga を link しない（K6）。kivact-shader は境界として残す。`ShaderError` の位置を `Option` にする。
- **手順**：
  1. `crates/kivact-style/Cargo.toml`：`kivact-shader = { path = "../kivact-shader", version = "0.1.0" }` を `kivact-shader = { path = "../kivact-shader", version = "0.1.0", optional = true }` にし、`[features]` を足す：
     ```toml
     [features]
     # Validate and assemble user WGSL with naga (kivact-shader). The facade
     # enables it with `gpu`: without a GPU the CPU draws a shader's fallback,
     # so the source is kept as given and checked only for being empty.
     shader-validation = ["dep:kivact-shader"]
     ```
  2. `crates/kivact-core/Cargo.toml` の `[features]` に `shader-validation = ["kivact-style/shader-validation"]` を足す（DEP-4：facade と葉の間を core が転送する）。
  3. `crates/kivact/Cargo.toml` の `gpu = ["dep:arvane-compose", "arvane-compose/gpu"]` を `gpu = ["dep:arvane-compose", "arvane-compose/gpu", "kivact-core/shader-validation"]` にする。
  4. `crates/kivact-shader/src/error.rs` の `pub struct ShaderError` の `line: u32` と `column: u32` を次の 1 field にする：
     ```rust
     /// The 1-based line and byte column in the user's source, or `None`
     /// when the error has no place in it (a missing entry point).
     pub position: Option<(u32, u32)>,
     ```
     `ShaderError::new(message, line, column)` の本体を `position: Some((line, column))`、`ShaderError::whole(message)` を `position: None` にする。`impl fmt::Display` を `match self.position { None => write!(f, "{}", self.message), Some((line, column)) => write!(f, "{line}:{column}: {}", self.message) }` にする。`crates/kivact-shader/src/assemble.rs` の doc の例 `assert_eq!((error.line, error.column), (2, 30));` と `(1, 4)` の行を `assert_eq!(error.position, Some((2, 30)));` と `assert_eq!(error.position, Some((1, 4)));` にする。`crates/kivact-shader/src/source.rs` の `ShaderError` を作る所は `new`/`whole` を呼んでいれば変更なし（field を直接書いていれば同じ規則で直す）。
  5. `crates/kivact-style/src/shader/program.rs` の全体を次にする：

```rust
//! Role: Assembling a WGSL source into a shader program, and its errors.

use std::panic::Location;
use std::sync::Arc;

use kivact_runtime::{ErrorKind, UiError};
use kivact_types::{ShaderKind, ShaderProgram};

/// Validate and assemble `source` with naga (GPU builds).
#[cfg(feature = "shader-validation")]
#[track_caller]
pub(crate) fn compile(source: &str, kind: ShaderKind) -> Result<Arc<ShaderProgram>, UiError> {
    kivact_shader::assemble(source, kind)
        .map(Arc::new)
        .map_err(|err| {
            let message = match err.position {
                None => err.message,
                Some((line, column)) => format!("line {line}, column {column}: {}", err.message),
            };
            UiError::new(ErrorKind::Shader, message, Some(Location::caller()))
        })
}

/// Keep `source` as it is (CPU-only builds, which draw the fallback): only
/// an empty source is an error. Parameters are not known, so none is
/// checked (`data::param_problem`).
#[cfg(not(feature = "shader-validation"))]
#[track_caller]
pub(crate) fn compile(source: &str, kind: ShaderKind) -> Result<Arc<ShaderProgram>, UiError> {
    if source.trim().is_empty() {
        return Err(UiError::new(
            ErrorKind::Shader,
            "the shader source is empty",
            Some(Location::caller()),
        ));
    }
    Ok(Arc::new(ShaderProgram {
        kind,
        module: Arc::from(source),
        params: Vec::new(),
        params_size: 16,
        hash: fnv1a64(source),
    }))
}

/// FNV-1a over the source, the identity a program is cached by.
#[cfg(not(feature = "shader-validation"))]
fn fnv1a64(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
```

     `ShaderProgram` の field が 2b8df5ac の `kind`、`module`、`params`、`params_size`、`hash` 以外を持つ場合は、その field の型の `Default` 値を足す。
  6. `crates/kivact-style/src/shader/data.rs` の `fn param_problem(program: &ShaderProgram, name: &str, value: &ShaderValue) -> Option<String>` の本体の最初に次を足す：
     ```rust
     if !cfg!(feature = "shader-validation") {
         // Without validation the parameters are unknown; the CPU never runs the shader.
         return None;
     }
     ```
  7. `dev/tests/kivact-public-api/tests/*/shaders.rs`（W1 の分割の後の場所）の次の 5 つの試験関数の `#[test]` の次の行に `#[cfg(feature = "gpu")]` を足す（naga の検証が要る試験。GPU の無い build にはその検証が無い。O-4）：`invalid_wgsl_reports_line_and_column`、`forbidden_bindings_are_rejected`、`wrong_entry_signature_is_rejected`、`param_type_mismatch_is_reported`、`fallback_shader_problems_are_reported`。これらが使う helper（`shader_errors` など）が GPU なしで未使用になり warning が出る場合は、helper にも `#[cfg(feature = "gpu")]` を付ける。
- **消すもの**：`ShaderError::line`、`ShaderError::column`。
- **状態の閉じ方**：新しい型なし。
- **試験**：手順 7 の 5 関数に cfg を足す（期待値は変えない）。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-style -p kivact-shader
  cargo check -p kivact-style --all-targets
  cargo check -p kivact-style --all-targets --features shader-validation
  cargo check -p kivact --no-default-features --features complex-scripts
  cargo tree -p kivact --no-default-features --features complex-scripts -e normal | rg -c "naga"
  cargo clippy -p kivact-style -p kivact-shader --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate kivact-style
  cargo test -p kivact-shader
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- shaders
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core --no-default-features --features complex-scripts -- shaders
  ```
  5 行目は 0 を出す（GPU なしの build に naga が無い）。`shaders` が `e2e_native` にあれば `--test` をそれにする。
- **完了条件**：上の 5 行目が `0`。上のコマンドが通る。

---

## S08：filter graph の process 全体の counter と icon の enum の名前

- **wave / lane / 前提**：W2a / S / S07
- **所有**：`crates/kivact-style/src/effect/filter_graph.rs`、`crates/kivact-style/src/effect/filter.rs`、`crates/kivact-style/src/icon.rs`
- **目的**：ST-30 の内部の部分。graph の識別のための process 全体の `AtomicU64` をやめ、`FilterInput` を結果の番号だけにする。エラーは最初の一つだけ持つ。icon の private enum を意味の名前にする。
- **手順**：
  1. `crates/kivact-style/src/effect/filter_graph.rs`：
     - `use std::sync::atomic::{AtomicU64, Ordering};` と `static NEXT_GRAPH: AtomicU64 = AtomicU64::new(1);`（doc を含む）を消す。
     - `pub struct FilterInput { pub(crate) graph: u64, pub(crate) input: p::FilterInput }` を `pub struct FilterInput(pub(crate) p::FilterInput);` にし、doc を `/// A handle to a primitive's result, or to the element itself. Copy. A handle of a primitive the graph does not have yet is replaced by the element and reported.` にする。`SOURCE` と `SOURCE_ALPHA` の定数を `FilterInput(p::FilterInput::Source)`、`FilterInput(p::FilterInput::SourceAlpha)` にする。
     - `pub struct FilterGraph` の `id: u64,` を消し、`errors: Vec<String>,` を `error: Option<String>,` にする（「The first problem found while building; the later ones are not kept.」の doc を付ける）。`FilterGraph::new` から `id: NEXT_GRAPH.fetch_add(..)` を消し、`errors: Vec::new()` を `error: None` にする。型の doc の「A handle names one graph, so a copy would let handles of one graph slip into another unnoticed.」の文を「Not Clone: it is consumed by `Filter::graph`.」にする。module の doc の「The handles name their graph, so a handle of another graph is refused.」を消す。
     - `fn push` の戻り値を `FilterInput(p::FilterInput::Result(index))` にする。
     - `fn input(&mut self, h: FilterInput) -> p::FilterInput` の本体を次にする：
       ```rust
       if let p::FilterInput::Result(n) = h.0
           && n as usize >= self.prims.len()
       {
           self.report(format!(
               "primitive {}: an input names a result the graph does not have yet; the element is used",
               self.prims.len() + 1
           ));
           return p::FilterInput::Source;
       }
       h.0
       ```
       同じ `impl` に `fn report(&mut self, message: String) { if self.error.is_none() { self.error = Some(message); } }` を足し、ファイルの中の `self.errors.push(X)` をすべて `self.report(X)` にする。
     - `self.errors` を読む所（`Filter::graph` に渡す所、`AuthoredGraph` を作る所。`rg -n "errors" crates/kivact-style/src/effect` で探す）は `Vec<String>` を期待している。`AuthoredGraph`（または受け取る側）の field を `Option<String>` に変え、報告する所（`filter_problems` など）で `for e in &graph.errors` を `if let Some(e) = &graph.error` にする。
  2. `crates/kivact-style/src/effect/filter.rs` に `errors` の読みがあれば手順 1 と同じ規則で直す。
  3. `crates/kivact-style/src/icon.rs`：`enum InputProp` を `enum IconSource` に、`use kivact_runtime::{IntoValue, Prop as Input};` を `use kivact_runtime::{IntoValue, Prop};` にし、ファイルの中の `InputProp` を `IconSource`、`Input<Icon>` と `Input::constant` を `Prop<Icon>` と `Prop::constant` にする：
     `perl -pi -e 's/\bInputProp\b/IconSource/g; s/\bInput::constant\b/Prop::constant/g; s/\bInput<Icon>/Prop<Icon>/g; s/Prop as Input/Prop/' crates/kivact-style/src/icon.rs`
- **消すもの**：`NEXT_GRAPH`、`FilterInput::graph`、`FilterGraph::id`、`FilterGraph::errors`、`InputProp`。
- **状態の閉じ方**：`FilterGraph` は field の `Option` が `margin` と `error` の 2 つになる。`&mut self` の builder なので 6.1 の対象。印：`// xtask-allow(state): builder of one graph: margin is an authored option and error keeps the first build problem; each is written by its own method and both combinations occur`（`pub struct FilterGraph` の直前）。
- **試験**：style の filter graph の試験のうち「別の graph の handle」を確かめるものがあれば（`rg -n "another filter graph|other graph" crates dev`）、それは仕様の削除なので試験を消す（削除した試験の名前を作業報告に書く）。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-style
  cargo check -p kivact-style --all-targets
  cargo clippy -p kivact-style --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-style
  cargo test -p kivact-style
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- effects_filters
  ```
- **完了条件**：`rg -n "AtomicU64|NEXT_GRAPH|InputProp" crates/kivact-style/src` が 0 件。上のコマンドが通る。

---

## S09：`Theme(Arc)` と一つの編集の入口、fallback の cache、既定の theme の static、`with_fonts`、輝度

- **wave / lane / 前提**：W2a / S / S08
- **所有**：`crates/kivact-style/src/theme/value.rs`、`crates/kivact-style/src/theme.rs`、`crates/kivact-style/src/lib.rs`、`crates/kivact-style/src/facade.rs`
- **目的**：ST-26（`Theme` を `Arc<ThemeData>` にし、比較を `Arc::ptr_eq` から始める。`light()`・`dark()` は static の clone）、ST-07（拡張が無いときの fallback を theme の値ごとに一度だけ作る）、設計書 12 章 UI-S-01（`Theme` のすべての書込みは一つの編集の入口 `Theme::edit` を通り、拡張の fallback の cache を必ず空にする。`Arc` が共有でも一意でも同じ）、ST-03 のうち足す API（`Color::relative_luminance`・`Color::contrast_ratio`）、USR-12・API-13 の `Theme::with_fonts(impl IntoIterator<Item = impl Into<Asset>>)`。`cached_theme` は作らない（12 章 PF-02：関数の address は同一性を保証しない。kit は `thread_local!` の static で theme を保つ。4.4 節）。公開の署名は足すだけ（非破壊）。
- **手順**：
  1. `crates/kivact-style/src/theme/value.rs` の `pub struct Theme { extensions: ..., icons: IconSet, }` を、同じ field の private な `struct ThemeData` に改名し、field を 2 つ足して、`Theme` を包みにする：

```rust
/// The values of a [`Theme`], shared by its clones.
#[derive(Clone, Debug, PartialEq)]
struct ThemeData {
    extensions: Extensions,
    mode: ThemeMode,
    colors: Palette,
    space: Spacing,
    radius: Radii,
    text: Typography,
    focus_ring: f64,
    focus_ring_offset: f64,
    disabled_opacity: f64,
    control_height: f64,
    overlay_margin: f64,
    icons: IconSet,
    fonts: FontFiles,
    fallbacks: Fallbacks,
}

/// A complete theme. Replace with `App::theme` or `kivact::style::provide_theme`.
/// (keep the rest of the existing doc comment of `Theme` here)
///
/// A theme is a shared value: a clone costs a reference count, and two
/// clones compare equal at once.
#[derive(Clone, Debug)]
pub struct Theme(Arc<ThemeData>);

impl PartialEq for Theme {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0 == other.0
    }
}

// A theme is a `static` (`Theme::light`) and crosses to embed threads.
const _: () = {
    const fn shared<T: Send + Sync>() {}
    shared::<Theme>();
};

/// Font files a theme registers, as assets (a path, embedded bytes, owned
/// or shared bytes). `Asset` compares embedded and shared bytes by identity
/// (address and allocation), never byte by byte; the host registers each
/// asset once by that identity.
#[derive(Clone, Debug, Default, PartialEq)]
struct FontFiles(Vec<crate::asset::Asset>);

/// The fallback values of the extensions this theme lacks, built once per
/// theme value on the first read (ST-07). A cache: it is not part of the
/// theme's equality, and a changed copy of the theme starts empty.
#[derive(Default)]
struct Fallbacks(Mutex<Vec<(TypeId, Arc<dyn Any + Send + Sync>)>>);

impl Clone for Fallbacks {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for Fallbacks {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl fmt::Debug for Fallbacks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Fallbacks")
    }
}

impl Fallbacks {
    fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let list = self
            .0
            .lock()
            .expect("invariant: the theme fallback lock is not poisoned");
        let found = list
            .iter()
            .find(|(id, _)| *id == TypeId::of::<T>())
            .map(|(_, value)| Arc::clone(value));
        drop(list);
        found.and_then(|value| Arc::downcast::<T>(value).ok())
    }

    /// Keep `value` unless another reader stored one first; the one kept.
    fn insert<T: Send + Sync + 'static>(&self, value: Arc<T>) -> Arc<T> {
        let mut list = self
            .0
            .lock()
            .expect("invariant: the theme fallback lock is not poisoned");
        if let Some((_, kept)) = list.iter().find(|(id, _)| *id == TypeId::of::<T>())
            && let Ok(kept) = Arc::downcast::<T>(Arc::clone(kept))
        {
            return kept;
        }
        list.push((TypeId::of::<T>(), Arc::clone(&value) as Arc<dyn Any + Send + Sync>));
        value
    }
}
```

     `use std::sync::{Arc, Mutex};` は既にある。`use std::any::{Any, TypeId};` と `use std::fmt;` も既にある。
  2. **一つの編集の入口**（UI-S-01）：`impl Theme` に次の private な method を足し、`Theme` のすべての書込みをこれだけで行う：
     ```rust
     /// The theme's values to change. Every write goes through here: it
     /// copies shared values (`Arc::make_mut`) and empties the fallback
     /// cache, so a fallback that reads what changes is built again,
     /// whether this theme shared its values or not.
     fn edit(&mut self) -> &mut ThemeData {
         let data = Arc::make_mut(&mut self.0);
         data.fallbacks = Fallbacks::default();
         data
     }
     ```
     `impl Theme` の各 method を `self.0` 経由にする：読み取り（`self.mode` など）は `self.0.mode`、builder（`mut self` で field を書くもの）は `self.edit().<field> = ...;` にする。`Arc::make_mut(&mut self.0)` を `edit` の外で書かない（`rg -n "make_mut\(&mut self.0\)" crates/kivact-style/src/theme` が `edit` の 1 か所だけ）。`with_icon` は `let icons = self.0.icons.clone().with(name, image); self.edit().icons = icons;`。`with` は `self.edit().extensions.insert(Entry::new(value));`。`impl Color` の解決（`theme.colors.get(t)`）は `theme.0.colors.get(t)` にする。
  3. `pub fn ext<T: ThemeExtension>(&self) -> ThemeRef<T>` の本体を次にする：
     ```rust
     if let Some(value) = self.0.extensions.get::<T>() {
         return ThemeRef(value);
     }
     if let Some(value) = self.0.fallbacks.get::<T>() {
         return ThemeRef(value);
     }
     // Built outside the lock: a fallback may read other extensions.
     let built = Arc::new(T::fallback(self));
     ThemeRef(self.0.fallbacks.insert(built))
     ```
  4. `pub fn light() -> Theme` の本体の `Theme { ... }` の式を private な `fn light_theme() -> Theme { Theme(Arc::new(ThemeData { ..., fonts: FontFiles::default(), fallbacks: Fallbacks::default() })) }` に移し、`light`・`dark` を次にする：
     ```rust
     /// Light business-tool theme. A clone of one shared value.
     pub fn light() -> Theme {
         static LIGHT: SharedDeferred<Theme> = SharedDeferred::new();
         LIGHT.get_or_init(light_theme).clone()
     }

     /// Dark theme with the same metrics. A clone of one shared value.
     pub fn dark() -> Theme {
         static DARK: SharedDeferred<Theme> = SharedDeferred::new();
         DARK.get_or_init(dark_theme).clone()
     }
     ```
     `fn dark_theme() -> Theme` は元の `dark()` の本体（`Theme::light().with_mode(ThemeMode::Dark).with_colors(Palette { ... })`）。`use kivact_types::cell::SharedDeferred;` を足す。
  5. font を足す（`impl Theme` の中）：
     ```rust
     /// The theme with these font files added to the ones it has (a family
     /// in them is then usable by name in `font_family`). `App::theme` and
     /// `kivact::test::RenderOptions::with_theme` register them; a theme
     /// switched to later registers its own too.
     /// Any asset is a font file: `include_bytes!(..)`, a path, a `Vec<u8>`
     /// read at run time, shared bytes.
     #[must_use]
     pub fn with_fonts(
         mut self,
         fonts: impl IntoIterator<Item = impl Into<crate::asset::Asset>>,
     ) -> Theme {
         let added: Vec<crate::asset::Asset> = fonts.into_iter().map(Into::into).collect();
         self.edit().fonts.0.extend(added);
         self
     }

     /// The font files the theme registers, in the order added.
     pub fn fonts(&self) -> &[crate::asset::Asset] {
         &self.0.fonts.0
     }
     ```
  6. `crates/kivact-style/src/theme.rs` の Role 行の後の文の「`provide_theme` is in `kivact::view`」を「`provide_theme` is in `kivact::style`」に直す（ST-29）。
  7. 輝度（`crates/kivact-style/src/theme/value.rs` の `impl Color` の中）：

```rust
    /// The WCAG relative luminance of the color, from 0 (black) to 1
    /// (white). A theme token resolves in the theme in effect; the alpha
    /// is ignored.
    pub fn relative_luminance(self) -> f64 {
        let c = self.resolve(&ambient_theme()).sanitize();
        let linear = |v: f64| {
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(c.r) + 0.7152 * linear(c.g) + 0.0722 * linear(c.b)
    }

    /// The WCAG contrast ratio of two colors, from 1 to 21.
    pub fn contrast_ratio(self, other: Color) -> f64 {
        let (a, b) = (self.relative_luminance(), other.relative_luminance());
        let (high, low) = if a >= b { (a, b) } else { (b, a) };
        (high + 0.05) / (low + 0.05)
    }
```

     同じファイルの module の関数として：
     ```rust
     /// The theme a color resolves in where no theme is passed. S16 makes it
     /// the theme in effect (`try_use_theme`).
     fn ambient_theme() -> Theme {
         Theme::light()
     }
     ```
- **消すもの**：なし（公開の署名は増えるだけ）。
- **状態の閉じ方**：`ThemeData` は値の record で `&mut self` の method を持たない（書くのは `Theme::edit` だけ）。bool・Option・counter の field を持たないので 6.1 の (a) に当たらない。`Theme` は field 一つ（`Arc`）。`Fallbacks` は field 一つ（cache。書くのは `ext` の挿入と `edit` の空にすることだけ）。
- **試験**：`crates/kivact-style/src/theme/value.rs` の `mod tests` に足す：
  - `fn clones_share_one_value_and_compare_at_once()`：`let a = Theme::light(); let b = Theme::light(); assert!(a == b); assert!(Arc::ptr_eq(&a.0, &b.0));`
  - `fn a_missing_extension_falls_back_once_per_theme()`：`#[derive(Debug, PartialEq)] struct Probe(u32);`、`impl ThemeExtension for Probe { fn fallback(_: &Theme) -> Self { BUILT.fetch_add(1, Ordering::Relaxed); Probe(1) } }`（試験の中の `static BUILT: AtomicU32`）。同じ `Theme` で `ext::<Probe>()` を 2 回読んで `BUILT` が 1 であること。
  - UI-S-01 の不変条件（fallback が theme の値を読む型 `#[derive(Debug, PartialEq)] struct Height(f64); impl ThemeExtension for Height { fn fallback(t: &Theme) -> Self { Height(t.control_height()) } }`）：
    - `fn a_builder_on_a_unique_theme_rebuilds_the_fallback()`：`let t = Theme::light().with_control_height(30); assert_eq!(t.ext::<Height>().0, 30.0); let t = t.with_control_height(40); assert_eq!(t.ext::<Height>().0, 40.0);`（`t` の `Arc` は一意）。
    - `fn a_builder_on_a_shared_theme_rebuilds_the_fallback_and_keeps_the_clone()`：`let a = Theme::light().with_control_height(30); let _ = a.ext::<Height>(); let b = a.clone().with_control_height(40);` で `b.ext::<Height>().0 == 40.0` かつ `a.ext::<Height>().0 == 30.0`（旧い clone は変わらない）。
    - `fn installing_an_extension_rebuilds_a_fallback_that_reads_it()`：fallback が別の拡張を読む型（`struct Twice(u32); fallback(t) = Twice(t.ext::<Base>().0 * 2)`、`struct Base(u32)` の fallback は `Base(1)`）で、`let t = Theme::light(); assert_eq!(t.ext::<Twice>().0, 2); let t = t.with(Base(5)); assert_eq!(t.ext::<Twice>().0, 10);`。
  - `fn fonts_take_any_asset()`：`Theme::light().with_fonts([Asset::from(&b"x"[..])]).with_fonts(vec![Asset::from(vec![1u8, 2])]).fonts().len() == 2`、`Theme::light().with_fonts(Vec::<Asset>::new()).fonts().is_empty()`（`Asset` の `From` は 2b8df5ac の `kivact_types::asset` のもの。名前は `use crate::asset::Asset`）。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-style
  cargo check -p kivact-style --all-targets
  cargo clippy -p kivact-style --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-style
  cargo test -p kivact-style
  ```
- **完了条件**：`rg -n "pub struct Theme \{" crates/kivact-style/src` が 0 件。`rg -n "make_mut\(&mut self.0\)" crates/kivact-style/src/theme` が `fn edit` の 1 件だけ。`rg -n "cached_theme" crates` が 0 件。上のコマンドが通る。

---

## S10：`TypeMap`（型を key にする表を一つに）

- **wave / lane / 前提**：W3a / S / S09（W2a）
- **所有**：`crates/kivact-types/src/type_map.rs`（新設）、`crates/kivact-types/src/lib.rs`、`crates/kivact-style/src/theme/value.rs`、`dev/xtask/policy/workspace.toml`（`[module_rules.kivact-types.uses]` の 1 行）
- **目的**：ST-27。theme の拡張の表と ui の variant の表の二つの写しを、一つの型 `TypeMap`（共有用 `SharedTypeMap`・thread 内用 `LocalTypeMap`）にする。この unit は types と theme を移す。ui（と core に移る variant）の表は付属書 U・C がこの型に移す（4.8 節）。
- **手順**：
  1. `crates/kivact-types/src/type_map.rs` を作る：

```rust
//! Role: A map from a type to one value of that type, shared across threads or kept on one.

use std::any::{Any, TypeId};
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

/// Writes the two maps: the same code over `Arc` (Send + Sync values) and
/// `Rc` (UI thread values).
macro_rules! type_map {
    ($(#[$doc:meta])* $name:ident, $entry:ident, $ptr:ident, $dyn:ty, $($bound:tt)+) => {
        #[derive(Clone)]
        struct $entry {
            id: TypeId,
            value: $ptr<$dyn>,
            eq: fn(&dyn Any, &dyn Any) -> bool,
            debug: fn(&dyn Any, &mut fmt::Formatter<'_>) -> fmt::Result,
        }

        $(#[$doc])*
        #[derive(Clone, Default)]
        pub struct $name(Option<$ptr<Vec<$entry>>>);

        impl $name {
            /// An empty map; it allocates nothing until a value is added.
            pub const fn new() -> Self {
                Self(None)
            }

            /// The map with `value` in it, replacing a value of the same type.
            #[must_use]
            pub fn with<T: PartialEq + fmt::Debug + $($bound)+>(mut self, value: T) -> Self {
                fn eq<T: PartialEq + 'static>(a: &dyn Any, b: &dyn Any) -> bool {
                    a.downcast_ref::<T>() == b.downcast_ref::<T>()
                }
                fn debug<T: fmt::Debug + 'static>(
                    value: &dyn Any,
                    f: &mut fmt::Formatter<'_>,
                ) -> fmt::Result {
                    match value.downcast_ref::<T>() {
                        Some(value) => value.fmt(f),
                        None => Ok(()),
                    }
                }
                let entry = $entry {
                    id: TypeId::of::<T>(),
                    value: $ptr::new(value),
                    eq: eq::<T>,
                    debug: debug::<T>,
                };
                let entries = $ptr::make_mut(self.0.get_or_insert_with(|| $ptr::new(Vec::new())));
                match entries.iter_mut().find(|e| e.id == entry.id) {
                    Some(slot) => *slot = entry,
                    None => entries.push(entry),
                }
                self
            }

            /// The value of type `T`, borrowed.
            pub fn get<T: 'static>(&self) -> Option<&T> {
                self.find::<T>().and_then(|e| {
                    let value: &dyn Any = &*e.value;
                    value.downcast_ref::<T>()
                })
            }

            /// The value of type `T`, shared.
            pub fn get_shared<T: $($bound)+>(&self) -> Option<$ptr<T>> {
                self.find::<T>()
                    .and_then(|e| $ptr::downcast::<T>($ptr::clone(&e.value)).ok())
            }

            /// Whether the map holds no value.
            pub fn is_empty(&self) -> bool {
                self.0.as_ref().is_none_or(|entries| entries.is_empty())
            }

            fn find<T: 'static>(&self) -> Option<&$entry> {
                self.0.as_ref()?.iter().find(|e| e.id == TypeId::of::<T>())
            }

            fn entries(&self) -> &[$entry] {
                self.0.as_deref().map_or(&[], Vec::as_slice)
            }
        }

        impl PartialEq for $name {
            /// Equal when they hold equal values of the same types (a shared
            /// table is equal to itself at once).
            fn eq(&self, other: &Self) -> bool {
                if let (Some(a), Some(b)) = (&self.0, &other.0)
                    && $ptr::ptr_eq(a, b)
                {
                    return true;
                }
                let (a, b) = (self.entries(), other.entries());
                a.len() == b.len()
                    && a.iter().all(|x| {
                        b.iter().any(|y| {
                            x.id == y.id
                                && ($ptr::ptr_eq(&x.value, &y.value)
                                    || (x.eq)(&*x.value, &*y.value))
                        })
                    })
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                struct Value<'a>(&'a $entry);
                impl fmt::Debug for Value<'_> {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        (self.0.debug)(&*self.0.value, f)
                    }
                }
                f.debug_list().entries(self.entries().iter().map(Value)).finish()
            }
        }
    };
}

type_map!(
    /// A map from a type to one `Send + Sync` value, shared by its clones
    /// across threads (a theme's extensions).
    SharedTypeMap, SharedEntry, Arc, dyn Any + Send + Sync, Send + Sync + 'static
);

type_map!(
    /// A map from a type to one value, shared by its clones on one thread
    /// (a component's variants).
    LocalTypeMap, LocalEntry, Rc, dyn Any, 'static
);
```

     `(x.eq)(&*x.value, &*y.value)` の `&*x.value` は `&(dyn Any + Send + Sync)` で、`&dyn Any` に coerce される。compile が coerce しない場合は `(x.eq)(&*x.value as &dyn Any, &*y.value as &dyn Any)` と書く（`debug` の呼出しも同じ）。`crates/kivact-types/src/lib.rs` に `pub mod type_map;` と `pub use type_map::{LocalTypeMap, SharedTypeMap};` を足す。`dev/xtask/policy/workspace.toml` の `[module_rules.kivact-types.uses]` に `type_map = []` を足す。
  2. `crates/kivact-style/src/theme/value.rs`：`struct Entry`、`impl Entry`、`impl fmt::Debug for Entry`、`struct Extensions`、`impl PartialEq for Extensions`、`impl Extensions` を消す。`ThemeData` の `extensions: Extensions,` を `extensions: SharedTypeMap,` にし、`Theme::with` の本体を `let data = self.edit(); data.extensions = std::mem::take(&mut data.extensions).with(value); self` にする（S09 の編集の入口。fallback の cache が空になる）。`Theme::ext` の `self.0.extensions.get::<T>()` を `self.0.extensions.get_shared::<T>()` にする。`light_theme()` の `extensions: Extensions::default()` を `extensions: SharedTypeMap::new()` にする。`use kivact_types::SharedTypeMap;` を足す。module の doc の Invariants の「A theme with no extension allocates no table」の文はそのまま正しい。
- **消すもの**：`theme/value.rs` の `Entry`・`Extensions`。
- **状態の閉じ方**：`SharedTypeMap`・`LocalTypeMap` は field 一つ。
- **試験**：`crates/kivact-types/src/type_map.rs` に `#[cfg(test)] mod tests`（`//! Role: Tests of the type maps.`）：
  - `fn a_value_replaces_one_of_its_type()`：`SharedTypeMap::new().with(1_u32).with(2_u32)` の `get::<u32>()` が `Some(&2)`、`with("a")` を足しても `get::<u32>()` が 2。
  - `fn maps_with_equal_values_are_equal()`：`LocalTypeMap::new().with(1_u8).with("x")` と順が逆の map が `==`、値が違えば `!=`、空同士は `==`。
  - `fn get_shared_shares_the_value()`：`let m = SharedTypeMap::new().with(String::from("a")); assert!(Arc::ptr_eq(&m.get_shared::<String>().unwrap(), &m.get_shared::<String>().unwrap()));`
  - theme の既存の試験（`kivact-style` の `ThemeExtension` の doctest）がそのまま通ること。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-types -p kivact-style
  cargo check -p kivact-types -p kivact-style --all-targets
  cargo clippy -p kivact-types -p kivact-style --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-types
  cargo xtask check --no-write --crate kivact-style
  cargo test -p kivact-types type_map
  cargo test -p kivact-style
  ```
- **完了条件**：`rg -n "struct Extensions|struct Entry" crates/kivact-style/src` が 0 件。上のコマンドが通る。

---

## S11：SVG の書き出しの欠落をなくす（DEC-11）

- **wave / lane / 前提**：W3a / S / S10
- **所有**：`crates/kivact-svg/src/`（`emit.rs`、`text.rs`、`encode.rs`、`resources.rs`、`lib.rs` の doc、`document.rs` の試験）、`crates/kivact-offscreen/src/lib.rs`（`Resources` の実装の 1 関数）、`dev/tests/kivact-public-api/tests/*/offscreen_svg.rs`
- **目的**：ST-15 と DEC-11。SVG 1.1 と CSS に同等のものがある機能（`mix-blend-mode`、文字の塗り（gradient）と線、`font-stretch`、`font-kerning`、`font-feature-settings`、`font-variation-settings`、文字の装飾）は書き出す。同等のものが無い機能（層の Porter–Duff の operator、層の filter・mask・3D の transform）だけを失敗にする。黙って落とすものを無くす。
- **手順**：
  1. `crates/kivact-svg/src/encode.rs` に CSS の名前を足す：
     ```rust
     /// The CSS `mix-blend-mode` keyword of `mode`.
     pub(super) fn blend_name(mode: BlendMode) -> &'static str {
         match mode {
             BlendMode::Normal => "normal",
             BlendMode::Multiply => "multiply",
             BlendMode::Screen => "screen",
             BlendMode::Overlay => "overlay",
             BlendMode::Darken => "darken",
             BlendMode::Lighten => "lighten",
             BlendMode::ColorDodge => "color-dodge",
             BlendMode::ColorBurn => "color-burn",
             BlendMode::HardLight => "hard-light",
             BlendMode::SoftLight => "soft-light",
             BlendMode::Difference => "difference",
             BlendMode::Exclusion => "exclusion",
             BlendMode::Hue => "hue",
             BlendMode::Saturation => "saturation",
             BlendMode::Color => "color",
             BlendMode::Luminosity => "luminosity",
             BlendMode::PlusLighter => "plus-lighter",
         }
     }
     ```
  2. `crates/kivact-svg/src/emit.rs` の `fn layer_attributes` の最初の検査（S03 で書いた `if layer.mix != Mix::Normal { ... }`）を次に置き換える：
     ```rust
     let blend = match layer.mix {
         Mix::Normal => None,
         Mix::Blend(mode) => Some(mode),
         Mix::Composite(_) => {
             return Err(SvgError::unsupported("a composite operation on a layer"));
         }
     };
     ```
     関数の終わりの `Ok(attributes)` の前に足す：
     ```rust
     if let Some(mode) = blend {
         attributes.push_str(&format!(" style=\"mix-blend-mode:{}\"", blend_name(mode)));
     }
     ```
     関数の doc を「The attributes of the group of a layer: opacity, a clip and a blend mode (CSS `mix-blend-mode`). A composite operation, a filter, a mask and a 3D transform have no SVG 1.1 or CSS form.」にする。
  3. `crates/kivact-svg/src/text.rs` の `fn text` を次の規則で書き換える（module の doc の「Font features and text decorations have no SVG 1.1 form and are not supported.」を「Fill and stroke paints, font stretch, kerning, features, variations and decorations are written as SVG presentation attributes and CSS properties.」にする）：
     - `features` と `decoration` で `Err` を返す 2 つの `if` を消す。
     - 塗り：`run.fill` が `None` なら今の `fill="#hex"` と `opacity_attribute("fill", run.color)` のまま。`Some(fill)` なら `paint_attributes(self, "fill", fill, bounds)?` を使う。`bounds` は window が文字の gradient を塗る箱と同じ、run の原点と測った大きさの箱（engine の `lower/scene/text.rs` の `paint_run` は `Rect { origin, shaped.size }` で塗る）：`crates/kivact-svg/src/resources.rs` の `fn baseline(&self, run: &TextRun) -> f64` を `fn metrics(&self, run: &TextRun) -> TextMetrics`（doc：`/// The size of a run's line box and the distance from its top to the baseline, as the text engine measures it.`、`kivact_types::TextMetrics`）にし、`crates/kivact-offscreen/src/lib.rs` の実装を `self.measurer.measure(run)` にする。`text` の `let baseline = self.resources.baseline(run);` を `let metrics = self.resources.metrics(run); let baseline = metrics.baseline; let bounds = Rect { x: run.origin.x, y: run.origin.y, width: metrics.size.width, height: metrics.size.height };` にする。gradient は shape と同じ `gradientUnits="userSpaceOnUse"` で書く（px の stop・半径と canvas の座標の gradient がそのまま正しい）。`fill.rs` に単位の enum は足さない。
     - 線：`run.stroke` が `Some((stroke, paint))` なら `paint_attributes(self, "stroke", paint, bounds)?` と、emit.rs の既存の `fn stroke_attributes(stroke: &Stroke) -> String`（`stroke-width` などを書く。`pub(super)` にする）の結果を足す。
     - `run.font_width != 100.0` なら ` font-stretch="{number(run.font_width)}%"`。
     - 次の CSS を一つの `style="..."` にまとめる（空なら書かない）。各項目は `;` で区切る：
       - `run.kerning`：`FontKerning::Normal` → `font-kerning:normal`、`FontKerning::None` → `font-kerning:none`、`Auto` → 書かない。
       - `run.features` が空でなければ `font-feature-settings:` の後に各 feature を `'tttt' v`（`tttt` は 4 byte の tag を `String::from_utf8_lossy`、`v` は値）を `, ` で結ぶ。
       - `run.variations` が空でなければ `font-variation-settings:` の後に `'tttt' v`（`number(v)`）を `, ` で結ぶ。
       - `run.decoration.lines` が空でなければ：`text-decoration-line:` に `underline`・`overline`・`line-through` のうち `contains` するものを空白で結ぶ；`text-decoration-style:` に `solid`・`dashed`・`dotted`・`wavy`（`TextDecorationStyle` の variant）；`run.decoration.color` が `Some(c)` なら `text-decoration-color:` と `hex(c)`（alpha が 1 未満なら `rgba(r,g,b,a)` の CSS 形。encode.rs に `css_color(c: Rgba) -> String` を足して使う）；`thickness` が `Some(t)` なら `text-decoration-thickness:` と px（`TextLen::Px(v)` は `v`、`TextLen::Em(v)` は `v * run.font_size`。`number(..)` と `px`）；`underline_offset` が 0 でなければ `text-underline-offset:` と px（同じ換算）。
     - 属性の値の中の `'` と `"` は `escape` を通す。
  4. 既存の試験：`dev/tests/kivact-public-api/tests/*/offscreen_svg.rs` の `a_layer_svg_cannot_express_is_an_error` を、`Layer::new().blend(BlendMode::Multiply)` から `Layer::new().composite(Composite::DestinationOut)` に替える（`.expect_err` の文を `"a composite operation has no SVG 1.1 form"` に）。これは仕様の変更（DEC-11：blend は書き出せる）による期待値の更新で、緩めではない。`use kivact::draw::{...}` に `Composite` を足す。
- **消すもの**：`text.rs` の features と decoration の `Err`、`emit.rs` の blend の `Err`。
- **状態の閉じ方**：新しい型は無い。
- **試験**（`crates/kivact-svg/src/document.rs` の `mod tests` に足す。`write_list(vec![...])` の形で、`DrawCmd::PushLayer`・`DrawCmd::Text` などを既存の試験の組み立て方で作る）：
  - `fn a_blended_layer_writes_its_css_blend_mode()`：`mix: Mix::Blend(BlendMode::Multiply)` の層 → 出力に `mix-blend-mode:multiply`。
  - `fn a_composited_layer_is_unsupported()`：`mix: Mix::Composite(Composite::DestinationOut)` → `Err(SvgError::Unsupported(_))`。
  - `fn text_writes_its_gradient_fill_and_stroke()`：`fill: Some(Fill::Layers(..gradient..))` と `stroke: Some((Stroke 幅 2, Fill::Solid(..)))` の `TextRun` → 出力に `fill="url(#` と `stroke-width="2"` と `gradientUnits="userSpaceOnUse"`（試験の `Resources` の `metrics` は `TextMetrics { size: Size::new(80.0, 20.0), baseline: 0.0 }`）。
  - `fn text_writes_features_variations_and_stretch()`：`features: vec![FontFeature { tag: *b"tnum", value: 1 }]`、`variations: vec![FontVariation { tag: *b"wght", value: 650.0 }]`、`font_width: 75.0` → 出力に `font-feature-settings:'tnum' 1`、`font-variation-settings:'wght' 650`、`font-stretch="75%"`。
  - `fn text_writes_its_decoration()`：`decoration.lines = DecorationLines::UNDERLINE`、`style = Wavy` → `text-decoration-line:underline` と `text-decoration-style:wavy`。
  - `dev/tests/kivact-public-api/tests/*/offscreen_svg.rs` に `fn tabular_numbers_export_as_a_font_feature()` を足す：`Offscreen::new().render_svg(100.0, 40.0, |draw, _| { draw.text("12:34", Point::new(0.0, 0.0), &TextFormat::new().tabular_nums()) })`（`TextFormat` に `tabular_nums` が無ければ `font_feature_settings([("tnum", 1)])` の同等の method。`rg -n "pub fn (tabular_nums|font_feature_settings|features)" crates/kivact-style/src/draw/text_run.rs` で確かめる）が `Ok` で `font-feature-settings` を含む。DEC-11 の理由（plot の `tnum`）の公開の契約の試験。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-svg
  cargo check -p kivact-svg --all-targets
  cargo clippy -p kivact-svg --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-svg
  cargo test -p kivact-svg
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- offscreen_svg
  ```
- **完了条件**：`rg -n "OpenType font features on text|\"text decorations\"|a blend mode on a layer" crates/kivact-svg/src` が 0 件。上のコマンドが通る。

---

## S12：`Draw` を必要なものを揃えて作る（`RecordContext`）

- **wave / lane / 前提**：W3a / S / S11、C02（W2b。`crates/kivact-offscreen/` を作る）
- **所有**：`crates/kivact-style/src/draw/recorder.rs`、`crates/kivact-style/src/draw/regions.rs`、`crates/kivact-style/src/draw/paragraph.rs`、`crates/kivact-style/src/draw/text_run.rs`、`crates/kivact-style/src/draw.rs`、`crates/kivact-style/src/internal.rs`、`crates/kivact-core/src/view/mount/draw.rs`、`crates/kivact-offscreen/src/lib.rs`（付属書 C の C02 が W2b で `crates/kivact-core/src/draw/offscreen.rs` から移した場所）
- **目的**：ST-18（S-7）。`Draw` を「文字の計測器なし」で作って後から `set_text_measurer`・`declare_regions` で埋める呼出し順の約束をやめ、`record` が必要なものを一度に受け取る。
- **手順**：
  1. `crates/kivact-style/src/draw/recorder.rs`：
     - `pub struct Draw` の `pub(super) measurer: Option<Rc<dyn p::TextShaper>>,` を `pub(super) measurer: Rc<dyn p::TextShaper>,` にし、doc を `/// Measures and shapes text.` にする。
     - `pub fn set_text_measurer` を消す（`#[doc(hidden)]` と `xtask-allow(internal-contract)` の行を含む）。
     - `pub(super) fn new(theme: Theme, size: Size) -> Self` を `pub(super) fn new(cx: RecordContext, size: Size) -> Self` にし、本体で `measurer: cx.shaper, theme: cx.theme, declarations: cx.regions.into_iter().collect(),` とする。
     - 次の型を足す（`pub struct Recording` の前）：
       ```rust
       /// What a recording needs from its host, given at once: the theme its
       /// colors resolve in, the text engine and the regions the node's setup
       /// declared (empty for an offscreen drawing).
       pub struct RecordContext {
           /// The theme in effect.
           pub theme: Theme,
           /// Measures and shapes text.
           pub shaper: Rc<dyn p::TextShaper>,
           /// The declared regions, each drawable once per layer.
           pub regions: Vec<(RegionId, RegionSemantics)>,
       }
       ```
     - `pub fn record(size: Size, theme: &Theme, f: impl FnOnce(&mut Draw)) -> Recording` を `pub fn record(size: Size, cx: RecordContext, f: impl FnOnce(&mut Draw)) -> Recording` にし、本体の最初の行を `let mut draw = Draw::new(cx, size);` にする。doc の「Inside `f`, first install [`set_text_measurer`] and [`declare_regions`], then」の文を消し、「`f` runs the application's drawing, then the retained style paint」にする。`RegionSemantics` の doc の「handed to the painter with [`Draw::declare_regions`] before the drawing callbacks run」を「handed to [`record`] in its [`RecordContext`]」にする。
  2. `crates/kivact-style/src/draw/regions.rs`：`pub fn declare_regions(&mut self, ...)` を消す。
  3. `crates/kivact-style/src/draw/paragraph.rs` と `text_run.rs`：`self.measurer` の `Option` の扱いを外す：`let Some(m) = &self.measurer else { return Path::new(); };` を `let m = &self.measurer;` に；`.measurer.as_ref().and_then(|shaper| shaper.shape(&run, max_width))` を `.measurer.shape(&run, max_width)`（戻り値が `Option` なら続く `.unwrap_or_default()` はそのまま）に。他の `self.measurer` の `Option` 由来の分岐も同じ規則で直す（`rg -n "measurer" crates/kivact-style/src/draw`）。
  4. `crates/kivact-style/src/draw.rs` の `pub use recorder::{...}` に `RecordContext` を足す（`Draw`、`Recording`、`RegionSemantics` と並べる）。`crates/kivact-style/src/internal.rs` の `pub use crate::draw::recorder::record;` を `pub use crate::draw::recorder::{RecordContext, record};` にする。
  5. `crates/kivact-core/src/view/mount/draw.rs` の描画の effect：
     ```rust
     let measurer = weak
         .upgrade()
         .map(|rt| ViewState::tree_mut(&rt).reader().text_measurer(id));
     ```
     を
     ```rust
     let Some(measurer) = weak
         .upgrade()
         .map(|rt| ViewState::tree_mut(&rt).reader().text_measurer(id))
     else {
         return;
     };
     ```
     にし、`kivact_style::internal::record(sz, &theme, |d| { if let Some(measurer) = measurer { d.set_text_measurer(measurer); } if let Some(regions) = &region_paint { d.declare_regions(regions.resolve()); } ...` を次にする：
     ```rust
     let cx = kivact_style::internal::RecordContext {
         theme,
         shaper: std::rc::Rc::new(measurer),
         regions: region_paint.as_ref().map(|r| r.resolve()).unwrap_or_default(),
     };
     ... = kivact_style::internal::record(sz, cx, |d| {
         if let Some(f) = &f { f(d, sz); }
         ... (the rest of the closure unchanged)
     });
     ```
     （`theme` は `let theme = theme.map(|t| t.get()).unwrap_or_default();` で作った `Theme` の値。`ViewState` を借りた後で `rt` が無い場合の `return` は、元の `measurer` が `None` だった場合と同じく何も描かない。）
  6. `crates/kivact-offscreen/src/lib.rs`（C02 の後の Offscreen）の `fn record`：`kivact_style::internal::record(size, &self.theme, |draw| { draw.set_text_measurer(measurer.clone()); paint(draw, size); })` を次にする：
     ```rust
     let cx = kivact_style::internal::RecordContext {
         theme: self.theme.clone(),
         shaper: std::rc::Rc::new(measurer.clone()),
         regions: Vec::new(),
     };
     ... = kivact_style::internal::record(size, cx, |draw| paint(draw, size));
     ```
     `text_measurer(..)` の戻り値の型が `TextShaper` を実装し `'static` であること（2b8df5ac の `set_text_measurer(impl TextShaper + 'static)` に渡していた値）を確かめる。
- **消すもの**：`Draw::set_text_measurer`、`Draw::declare_regions`、`Draw.measurer` の `Option`。
- **状態の閉じ方**：`Draw` の `Option` field が 1 つ減る。`Draw` は `&mut self` の描画の命令を持つ記録器で、`Option`・counter の field（`unsupported: u32`、`region_errors` は Vec）を数えると対象になりうる。印：`// xtask-allow(state): recorder of one drawing: unsupported counts dropped commands and the region sets record what was drawn, each written by the command that drops or draws`（`pub struct Draw` の直前。付属書 T の W1b がすでに同等の印を書いていれば、そちらを残してこの行は足さない）。
- **試験**：変更なし。公開 E2E の `canvas.rs`、`canvas_regions.rs`（または region を使う topic）、`offscreen.rs`、`text_canvas.rs`（存在するもの）がそのまま通ること。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-style -p kivact-core
  cargo check -p kivact-style -p kivact-core --all-targets
  cargo clippy -p kivact-style -p kivact-core --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-style
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- canvas
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test offscreen
  ```
- **完了条件**：`rg -n "set_text_measurer|declare_regions" crates` が 0 件。上のコマンドが通る。

---

## W6 の鎖の共通事項（S13〜S23）

- W6 の鎖は 1 lane で、S13 → S14 → … → S23 の順に一つずつ実行する。各 unit は **W6 の掃引の範囲** の呼出し元をすべて直す。範囲は次のとおり（1.2 節。kit・catalog・kit の試験・`e2e_kits`・`kivact-docs` は含めない）：
  `crates/kivact-types`、`crates/arvane-protocol`、`crates/kivact-runtime`、`crates/kivact-i18n`、`crates/kivact-style`、`crates/kivact-shader`、`crates/kivact-svg`、`crates/kivact-core`、`crates/kivact-interaction`、`crates/kivact-ui`、`crates/kivact-test`、`crates/kivact-devtools`、`crates/kivact`、`crates/kivact-offscreen`（付属書 C の C02 が W2b で作る）、`dev/tests/kivact-public-api/tests` のうち `e2e_kits` 以外、`dev/bench`、`examples/kivact`、`templates/basic`。engine・layout・editing・arvane-* は、各 unit の「所有」が名指したファイルだけを変える（4.10 節。W6 の間、付属書 E・A の lane はそのファイルを編集しない）。
- 掃引の範囲に perl を当てる共通の書き方：
  ```
  W6_SCOPE="crates/kivact-types crates/arvane-protocol crates/kivact-runtime crates/kivact-i18n crates/kivact-style crates/kivact-shader crates/kivact-svg crates/kivact-core crates/kivact-interaction crates/kivact-ui crates/kivact-test crates/kivact-devtools crates/kivact crates/kivact-offscreen dev/bench examples/kivact templates/basic"
  W6_TESTS=$(ls -d dev/tests/kivact-public-api/tests/* | grep -v e2e_kits)
  ```
- 鎖の各 unit の確認（tier a）の共通部分：
  ```
  cargo fmt --all
  cargo check --workspace --all-targets --exclude kivact-ui-basic --exclude kivact-ui-pop --exclude kivact-ui-game --exclude kivact-ui-creative --exclude kivact-ui-soft --exclude kivact-ui-basic-catalog --exclude kivact-ui-pop-catalog --exclude kivact-ui-game-catalog --exclude kivact-ui-creative-catalog --exclude kivact-ui-soft-catalog --exclude kivact-kit-tests --exclude kivact-docs
  cargo clippy -p kivact-style -p kivact-types -p kivact-core -p kivact-ui --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-style
  ```
  （`--exclude` の一覧は作業指示書の W6 の除外と同じ。作業指示書が別の書き方（feature や workspace の一時的な除外）を定めていればそれに従う。以下この 3 行を「鎖の共通確認」と書く。）
- 生成した名前は W5 の後の runtime・core の API に合わせる（1.2 節）。

---

## S13：同一の型の組を一つにする（ST-10）と `LayoutProps.hidden`（ST-19）

- **wave / lane / 前提**：W6a / 鎖 / W5b の終わり
- **所有**：W6 の掃引の範囲、および 4.10 節の S13 の engine・layout・arvane のファイル
- **目的**：ST-10（変更して採用：同一の組だけ）。`Length`≡`Dimension`、`TextLength`≡`TextLen`、`GridAutoFlow`≡`GridFlow`、`Edges<f64>`≡`Insets`、`ThemeMode`≡`ThemeHint`、`AvailableSpace`（core）≡`AvailableSpace`（types）を、kivact-types の一つの定義にする。`Color` と `Rgba` の組は残す（作者の値と解決した値）。ST-19：常に false の `LayoutProps.hidden` を消す。
- **手順**：
  1. **Length**：
     - `git mv crates/kivact-style/src/length.rs crates/kivact-types/src/length.rs`。移したファイルで：Role 行を `//! Role: Lengths: logical px, percentages, calc sums and content sizes, as authored and as laid out.` にする；`pub(crate) fn to_wire(self) -> kivact_types::Dimension` を消す；`pub(crate) fn is_content` と `pub(crate) fn to_len` を `pub fn` にし、`to_len` の `kivact_types::Len` を `crate::paint::Len` にする；末尾の `kivact_runtime::constant_value!(Length);` と 4 つの `impl kivact_runtime::IntoValue<Length> for ...` を切り取って、手順の次の項に移す。
     - `crates/kivact-runtime/src/reactive/props.rs`（W5 の後の場所。`constant_value!(` に `kivact_types::Modifiers,` などを並べた呼出しがあるファイル）の、その `kivact_types::...` の並びに `kivact_types::Length,` と `kivact_types::TextLength,` を足し、切り取った 4 つの `impl IntoValue<Length> for i32/u32/f64/f32` を同じファイルの `impl IntoValue<f64> for i32` の前に置く（path は `kivact_types::Length`）。
     - `crates/kivact-types/src/layout.rs`：`pub enum Dimension { ... }` を消し、ファイルの中の `Dimension` を `Length` にする（`perl -pi -e 's/\bDimension\b/Length/g' crates/kivact-types/src/layout.rs`）。`use crate::length::Length;` を足す。
     - `crates/kivact-types/src/lib.rs`：`pub mod length;` と `pub use length::{Length, auto, pct};` を足し、`pub use layout::{...}` の `Dimension,` を消す。
     - `Dimension` の利用者を置き換える（D3D12 の `Dimension` を含む `kivact-gpu-share` と `gpu-interop-producer` は対象外）：
       ```
       perl -pi -e 's/\bDimension\b/Length/g' crates/kivact-layout/src/columns.rs crates/kivact-layout/src/image.rs crates/kivact-layout/src/layout/tests.rs crates/kivact-layout/src/style/convert.rs crates/kivact-layout/src/style/node.rs crates/kivact-layout/src/tree.rs crates/kivact-engine/src/model/interact/sticky.rs crates/kivact-engine/src/model/presence/transitions.rs crates/kivact-core/src/view/elements/container.rs crates/kivact-core/src/view/flow/virtual_list.rs crates/kivact-test/src/screen/scene_cache_tests.rs crates/kivact-style/src/resolve/compile.rs
       ```
       同じファイルが `Length` を別の path（`kivact_style::Length` など）からも import していて「defined multiple times」（E0252）になったら、どちらか一方の `use` を消す（同じ型になった）。
     - style：`crates/kivact-style/src/lib.rs` の `mod length;` を消し、`pub use length::{Length, auto, pct};` を `pub use kivact_types::{Length, auto, pct};` にする。style の中の `crate::length::` と `super::length::` を `kivact_types::` にする（`rg -l "length::" crates/kivact-style/src | xargs perl -pi -e 's/\b(crate|super)::length::/kivact_types::/g'`）。`.to_wire()` を `Length` に対して呼ぶ所（`compile.rs` の `fn dim` の `l.map(Length::to_wire)`、`grid.rs` の `Breadth::Size(l.to_wire())` など、compile の error「no method named `to_wire` found for enum `Length`」が指す所）は、`.to_wire()`・`.map(Length::to_wire)` を消して値をそのまま使う。
  2. **TextLength**：
     - `crates/kivact-types/src/text/properties.rs` の `pub enum TextLen { Px(f64), Em(f64) }` と `impl Default for TextLen` を、style の `crates/kivact-style/src/text.rs` の `pub enum TextLength`（doc を含む）・`impl Default for TextLength`・4 つの `impl From<i32/u32/f32/f64> for TextLength`・`pub fn em(..)` で置き換える（`TextLength::to_wire` は持ってこない）。`perl -pi -e 's/\bTextLen\b/TextLength/g' crates/kivact-types/src/text/properties.rs crates/kivact-types/src/text/computed.rs crates/kivact-types/src/text.rs crates/kivact-types/src/lib.rs crates/arvane-text/src/cache.rs crates/kivact-engine/src/lower/glyphs.rs crates/arvane/tests/e2e/text_style.rs`。`crates/kivact-types/src/lib.rs` と `text.rs` の再公開に `em` を足す（`pub use text::{..., TextLength, ...}` の隣に `em`）。
     - style の `text.rs` から移したものを消し、`pub use kivact_types::{TextLength, em};` を足す。style の中の `.map(TextLength::to_wire)` と `TextLength` の値への `.to_wire()` を消す（compile の error が指す所）。`crates/kivact-style/src/lib.rs` の `pub use text::{..., TextLength, ..., em}` はそのまま（text module が再公開する）。
  3. **GridAutoFlow**：`crates/kivact-style/src/grid.rs` の `pub enum GridAutoFlow`（doc と derive）を `crates/kivact-types/src/layout.rs` の `pub enum GridFlow { ... }` の位置に移して置き換え、`#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]` と `Row` に `#[default]` を付ける。`perl -pi -e 's/\bGridFlow\b/GridAutoFlow/g' crates/kivact-types/src/layout.rs crates/kivact-types/src/lib.rs crates/kivact-layout/src/style/convert.rs`。style の `grid.rs` に `pub use kivact_types::GridAutoFlow;` を置き、`GridAutoFlow::Row => w::GridFlow::Row,` などの 4 行の変換の `match` を値そのもの（`flow`）に置き換える。
  4. **Edges<f64>**：
     - `crates/kivact-types/src/geometry.rs`：`pub struct Insets` を消す。`impl Insets { pub fn sanitize }` を `impl Edges<f64> { pub fn sanitize(self) -> Self { ... } }` に、`impl From<f64> for Insets` などの 4 つの `From` を `for Edges<f64>` に書き換える（本体の `Insets {` を `Edges {`）。`pub struct Edges<T>` の derive に、`Insets` が持っていた derive（`Clone, Copy, Debug, Default, PartialEq`）のうち無いものを足す。
     - 置換（`use` 文の中は `Edges`、struct の式は `Edges {`、関連の path は `Edges::<f64>::`、型の位置は `Edges<f64>`）。次の script を `insets.pl` として scratch に保存して当てる：
       ```perl
       #!/usr/bin/perl -pi
       if (/^\s*(?:pub(?:\([^)]*\))?\s+)?use\b/ .. /;\s*$/) { s/\bInsets\b/Edges/g; next; }
       s/\bInsets\s*\{/Edges {/g;
       s/\bInsets::/Edges::<f64>::/g;
       s/\bInsets\b/Edges<f64>/g;
       ```
       ```
       rg -l "\bInsets\b" crates dev --type rust | grep -v -E "kivact-ui-(basic|pop|game|creative|soft)/" | xargs perl -pi insets.pl
       ```
       `use` に `Edges` が二度出て E0252 になった行は一つにする。`crates/kivact-core/src/geometry.rs` の `pub use kivact_types::{Edge, Insets, Matrix4, Point, Rect, Size};` は置換で `Edge, Edges, ...` になる（`kivact::geometry::Edges` が公開の名前）。
  5. **ThemeMode**：
     - `crates/kivact-types/src/env.rs` の `pub enum ThemeHint` を `pub enum ThemeMode` にし、derive に `Hash` を足し、doc を style の `ThemeMode` の doc（「Whether a theme is light or dark. Every kit's theme constructor takes it first ...」）にする。
     - `crates/kivact-style/src/theme/value.rs`：`pub enum ThemeMode { ... }`（doc・derive を含む）と `impl From<kivact_types::ThemeHint> for ThemeMode` を消し、`pub use kivact_types::ThemeMode;` を置く。
     - `rg -l "\bThemeHint\b" crates dev --type rust | grep -v -E "kivact-ui-(basic|pop|game|creative|soft)/" | xargs perl -pi -e 's/\bThemeHint\b/ThemeMode/g'`
     - `crates/kivact-core/src/platform.rs` と `crates/kivact-core/src/platform/env.rs` の再公開の一覧から `ThemeMode`（置換後の名前）を消す（`kivact::style::ThemeMode` だけが公開の path。API-10）。`platform/env.rs` の `pub fn theme_hint() -> Value<ThemeMode>` は名前を変えず、`use kivact_types::ThemeMode;` を private の import として残す。
     - `crates/kivact-devtools/src/app.rs` の `match` で `kivact::platform::ThemeMode::Dark => ThemeMode::Dark, kivact::platform::ThemeMode::Light => ThemeMode::Light,`（置換後）となった `match` を値そのもの（`theme_hint().get()` の値）に置き換える。
  6. **AvailableSpace**：`crates/kivact-core/src/custom/measurement.rs` の `pub enum AvailableSpace { ... }` を消し、`pub use kivact_types::AvailableSpace;` にする。core 側の型と types 側の型の間の変換（`From` の impl や `match` で variant を写す関数。`rg -n "AvailableSpace::" crates/kivact-core/src` で探す）を消し、値をそのまま渡す。2 つの enum の variant が同じであることを先に `rg -n "pub enum AvailableSpace" -A8 crates/kivact-core/src/custom/measurement.rs crates/kivact-types/src/layout.rs` で確かめる（違えば STOP して報告）。
  7. **ST-19**：`crates/kivact-types/src/layout.rs` の `LayoutProps` の `pub hidden: bool,`（doc を含む）と `Default` の `hidden: false,` を消す。`crates/kivact-style/src/resolve/compile.rs` の `hidden: false,` を消す。`crates/kivact-layout/src/style/convert.rs` の `if l.hidden || hidden {` を `if hidden {` にする。`crates/kivact-core/src/view/tree.rs` の `node.input.hidden == Some(true) || node.layout.as_ref().is_some_and(|layout| layout.hidden)` を `node.input.hidden == Some(true)` にする。`crates/kivact-core/src/devtools/node.rs` の `property("hidden", layout.hidden),` の行を消す。
- **消すもの**：`kivact_types::{Dimension, TextLen, GridFlow, Insets, ThemeHint}`、style の `Length`・`TextLength`・`GridAutoFlow`・`ThemeMode` の定義と `to_wire`、core の `AvailableSpace` の定義、`LayoutProps::hidden`。
- **状態の閉じ方**：新しい型なし。
- **試験**：名前の置換だけ。公開 E2E の `layout_units.rs`、`text_style.rs`、`environment.rs`、`input_platform.rs`、`layout_grid.rs`（存在するもの）と `kivact-layout` の試験が通ること。
- **確認（tier a）**：鎖の共通確認と次：
  ```
  cargo clippy -p kivact-layout -p kivact-engine -p arvane-text -p arvane-shell -p arvane-virtual -p arvane-native --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-types
  cargo test -p kivact-types
  cargo test -p kivact-layout
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- layout
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- text_style
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- environment
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- input_platform
  ```
- **完了条件**：`rg -n -w "Dimension|TextLen|GridFlow|Insets|ThemeHint" crates dev examples templates --type rust | rg -v "kivact-ui-|D3D12|d3d12|dx12"` が 0 件。`rg -n "\.hidden\b" crates/kivact-layout/src/style/convert.rs` が `hidden` の引数だけ。上のコマンドが通る。

---

## S14：kivact-types の同名の異義を改名する（ST-21）

- **wave / lane / 前提**：W6a / 鎖 / S13
- **所有**：W6 の掃引の範囲、`crates/kivact-engine/`、`crates/arvane*/`、`crates/kivact-layout/`、`crates/kivact-editing/`、`crates/kivact-gpu-share/`（名前の置換だけ。4.10 節）
- **目的**：ST-21 の types の名前。公開の名前（`kivact::draw::Transform`、`kivact::style::Shape`、`kivact::style::Gradient`、`kivact::style::FilterGraph`）は変わらない。

  | 旧（kivact-types） | 新 |
  |---|---|
  | `kivact_types::Transform`（2D の affine。`geometry.rs`） | `kivact_types::Affine` |
  | `kivact_types::scene::Shape`（輪郭） | `kivact_types::scene::Outline` |
  | `kivact_types::Gradient`（`paint.rs` の CSS gradient の wire） | `kivact_types::GradientSpec` |
  | `kivact_types::FilterGraph`（`effect/filter_graph.rs`） | `kivact_types::FilterGraphSpec` |
  | `kivact_types::motion::Kind` | `kivact_types::motion::TransitionKind` |

- **手順**：
  1. 次の perl script を scratch に `rename_imported.pl` として保存する（S06 の script を一般にしたもの。第 1 引数が import 元の path の正規表現、続く `OLD=NEW` が改名、`--` の後がファイル）：

```perl
#!/usr/bin/perl
# usage: perl rename_imported.pl 'PREFIX_REGEX' Old=New ... -- FILE...
use strict;
use warnings;
my $prefix = shift @ARGV;
my %map;
while (@ARGV && $ARGV[0] ne '--') { my ($o, $n) = split /=/, shift @ARGV; $map{$o} = $n; }
shift @ARGV;
for my $f (@ARGV) {
    local $/;
    open my $in, '<', $f or die "$f: $!";
    my $s = <$in>;
    close $in;
    my %bare;
    while ($s =~ /use\s+(?:$prefix)::\{([^}]*)\}/g) {
        my $list = $1;
        for my $o (keys %map) { $bare{$o} = 1 if $list =~ /\b$o\b/; }
    }
    while ($s =~ /use\s+(?:$prefix)::(\w+)\s*;/g) { $bare{$1} = 1 if exists $map{$1}; }
    for my $o (keys %map) { $s =~ s/((?:$prefix)::)$o\b/$1$map{$o}/g; }
    for my $o (keys %bare) { $s =~ s/(?<!::)\b$o\b/$map{$o}/g; }
    open my $out, '>', $f or die "$f: $!";
    print $out $s;
    close $out;
}
```

  2. types の中の定義と利用（types は自分の名前を bare で使う）：
     ```
     perl -pi -e 's/(?<!::)\bTransform\b/Affine/g' crates/kivact-types/src/*.rs crates/kivact-types/src/*/*.rs
     perl -pi -e 's/(?<!::)\bFilterGraph\b/FilterGraphSpec/g; s/(?<!::)\bGradient\b/GradientSpec/g' crates/kivact-types/src/*.rs crates/kivact-types/src/*/*.rs
     perl -pi -e 's/\bShape\b/Outline/g' crates/kivact-types/src/scene/*.rs crates/kivact-types/src/scene.rs
     perl -pi -e 's/\bKind\b/TransitionKind/g' crates/kivact-types/src/motion.rs
     ```
     その後 types の中で `scene::Shape` を path で使う所（`crate::scene::Shape`。`rg -n "scene::Shape" crates/kivact-types/src`）を `crate::scene::Outline` にする。`ClipShape`・`DrawingShape`・`CoordinateShape`・`GradientShape`・`GradientKind`・`GradientStop`・`CoordinateGradient` は `\b` で守られ、変わらないことを `git diff --stat crates/kivact-types` と `rg -n "Clip(Outline)|Drawing(Outline)|GradientSpecKind" crates/kivact-types`（0 件であること）で確かめる。
  3. 利用者（types の外）：
     ```
     FILES=$(rg -l "kivact_types|kivact_engine::testing" crates dev --type rust | grep -v "^crates/kivact-types/" | grep -v -E "kivact-ui-(basic|pop|game|creative|soft)/")
     perl rename_imported.pl 'kivact_types' Transform=Affine Gradient=GradientSpec FilterGraph=FilterGraphSpec -- $FILES
     perl rename_imported.pl 'kivact_types::scene' Shape=Outline -- $FILES
     perl rename_imported.pl 'kivact_types::motion' Kind=TransitionKind -- $FILES
     perl rename_imported.pl 'kivact_engine::testing' Transform=Affine -- $FILES
     ```
     手で直すもの：
     - path の別名で types を使う所（`use kivact_types as p;` の `p::Transform`、`types::`、`values::`、`w::` など）：`rg -n "\b(p|types|values|w|kt)::(Transform|Gradient|FilterGraph)\b|\bscene::Shape\b|motion::Kind\b" crates dev --type rust` の各行を新しい名前にする（style の `crate::transform::Transform` などは style 自身の型で、変えない）。
     - `crates/arvane-render/src/draw/painter.rs` と `crates/arvane-render/src/convert.rs` は kurbo の `Affine` も使う。この 2 ファイルでは手順 3 の置換の後に名前が衝突する（E0252 または型の不一致）。この 2 ファイルでは types の側を `kivact_types::Affine` の完全な path で書き、`use kivact_types::{..., Affine, ...}` から `Affine` を外す。
     - `crates/kivact-runtime/src/motion/transition.rs` の `pub(crate) use kivact_types::motion::Kind;` は手順 3 で `TransitionKind` になる。runtime の中で `Kind::` を使う所（`rg -n "\bKind\b" crates/kivact-runtime/src`）を `TransitionKind` にする。
     - engine の `testing` の再公開（`crates/kivact-engine/src/host/testing.rs` の `pub use kivact_types::...Transform...`）は手順 3 で `Affine` になる。
  4. `cargo check --workspace ...`（鎖の共通確認の除外つき）の error を、上の規則で直す。
- **消すもの**：上の表の旧名。
- **状態の閉じ方**：新しい型なし。
- **試験**：名前の置換だけ。
- **確認（tier a）**：鎖の共通確認と次：
  ```
  cargo clippy -p kivact-engine -p arvane -p arvane-render -p arvane-compose -p kivact-layout -p kivact-svg -p kivact-runtime --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-types
  cargo test -p kivact-types
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p arvane --test e2e
  ```
- **完了条件**：`rg -n "kivact_types::(Transform|Gradient|FilterGraph)\b|scene::Shape\b|motion::Kind\b" crates dev --type rust` が 0 件。上のコマンドが通る。

---

## S15：theme 変数（`ThemeVar`）と中核の変数、旧い色・大きさの表の削除

- **wave / lane / 前提**：W6b / 鎖 / S14（W6a）
- **所有**：W6 の掃引の範囲、`crates/kivact-layout/src/style/convert.rs`
- **目的**：ST-01（DEC-02）、ST-02、ST-04。theme の値を theme 変数 `ThemeVar<T>` にする（CSS の custom property）。`Color::var`・`Length::var`・`Theme::set/get`、中核の `color`・`space`・`radius`・`font` の module。`ThemeColor`・`Color::of`・`Color::token`・`ColorToken`・`THEMED`・`Palette`・`Spacing`・`Radii`・`Typography`・`Theme::{colors, space, radius, text, with_colors, with_space, with_radius, with_text}`・`use_tokens` を消し、呼出し元を変数の読みに置き換える。closure を消すのは付属書 U・K（この unit は同じ型の値を返す読みに機械的に置き換えるだけ）。
- **手順**：
  1. **types**：`crates/kivact-types/src/theme_var.rs` を作り、`lib.rs` に `pub mod theme_var;` と `pub use theme_var::{ThemeVar, ThemeVarId};`、policy の `[module_rules.kivact-types.uses]` に `theme_var = []` と `length = ["paint", "theme_var"]`（`length` の行が S13 で入っていれば `"theme_var"` を足す）を足す：

```rust
//! Role: Theme variables: a typed name with a fallback, which a theme may set or alias.

use std::fmt;
use std::hash::{Hash, Hasher};

/// A theme variable, CSS's custom property (`var(--name)`): a value a
/// theme sets, with the fallback used where it does not. Declare it as a
/// `static` (its address is its identity) and give its value form as a
/// `const` beside it:
///
/// ```
/// use kivact::style::{Color, Length, ThemeVar, radius, rgb};
/// pub static BRAND_VAR: ThemeVar<Color> = ThemeVar::new("brand", rgb(0x7c3aed));
/// pub const BRAND: Color = Color::var(&BRAND_VAR);
/// // A component's variable that follows the core radius unless set.
/// pub static FIELD_RADIUS_VAR: ThemeVar<f64> = ThemeVar::alias("field.radius", &radius::MD_VAR);
/// pub const FIELD_RADIUS: Length = Length::var(&FIELD_RADIUS_VAR);
/// ```
///
/// Two variables are equal only when they are the same `static`.
pub struct ThemeVar<T: 'static> {
    name: &'static str,
    fallback: VarValue<T>,
}

/// What a variable holds: a value, or another variable of the same type
/// (an alias, CSS `--a: var(--b)`), followed when it is read.
#[derive(Clone, Debug, PartialEq)]
pub enum VarValue<T: 'static> {
    /// A value.
    Value(T),
    /// The value of another variable.
    Var(&'static ThemeVar<T>),
}

impl<T: 'static> ThemeVar<T> {
    /// A variable named `name` (diagnostics and DevTools) with `fallback`.
    pub const fn new(name: &'static str, fallback: T) -> Self {
        Self { name, fallback: VarValue::Value(fallback) }
    }

    /// A variable whose fallback is the value of `of`: a theme that sets
    /// `of` changes this one too, unless it sets this one.
    pub const fn alias(name: &'static str, of: &'static ThemeVar<T>) -> Self {
        Self { name, fallback: VarValue::Var(of) }
    }

    /// The name.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// What the variable holds where a theme does not set it.
    pub const fn fallback(&self) -> &VarValue<T> {
        &self.fallback
    }
}

impl<T: 'static> From<&'static ThemeVar<T>> for VarValue<T> {
    fn from(var: &'static ThemeVar<T>) -> Self {
        VarValue::Var(var)
    }
}

/// The values of the pure types a variable may hold, from what converts
/// to them (`Theme::set(&radius::MD_VAR, 20)`).
macro_rules! value_from {
    ($t:ty: $($src:ty),*) => {$(
        impl From<$src> for VarValue<$t> {
            fn from(v: $src) -> Self {
                VarValue::Value(<$t>::from(v))
            }
        }
    )*};
}
value_from!(f64: f64, f32, i32, u32);
value_from!(u16: u16);
value_from!(crate::LineHeight: crate::LineHeight);

impl<T: 'static> PartialEq for ThemeVar<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl<T: 'static> Eq for ThemeVar<T> {}

impl<T: 'static> Hash for ThemeVar<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::from_ref(self).hash(state);
    }
}

impl<T: 'static> fmt::Debug for ThemeVar<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "var(--{})", self.name)
    }
}

/// The identity of a variable (its address), the key a theme stores its
/// value under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ThemeVarId(usize);

impl ThemeVarId {
    /// The identity of `var`.
    pub fn of<T: 'static>(var: &'static ThemeVar<T>) -> ThemeVarId {
        ThemeVarId(std::ptr::from_ref(var).cast::<()>() as usize)
    }
}
```

     `lib.rs` の再公開は `pub use theme_var::{ThemeVar, ThemeVarId, VarValue};` にする。`LineHeight` が `From<f64>` を持つなら（`rg -n "impl From<f64> for LineHeight" crates/kivact-types/src`）、`value_from!(crate::LineHeight: crate::LineHeight, f64);` にする（`line_height(1.5)` と同じ書き方で `set` できる）。

  2. **Length：変数・`em`・式（calc）**（12 章 UI-S-03）：`crates/kivact-types/src/length.rs` の `pub enum Length` を次にし（doc は今の段落に「`em(1.5)` is 1.5 times the font size this style sets, or the theme's base size (`font::SIZE`) when it sets none. A sum with a theme variable is resolved again when the theme changes. A sum the host cannot take (auto or a content size in `+`/`-`, or two different variables) is `Invalid` and reported as `Unsupported` where it is used; it is never silently `Auto`.」を足す）、演算を書き直す：

```rust
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Length {
    #[default]
    Auto,
    Px(f64),
    Percent(f64),
    /// A multiple of the font size (see the type's doc).
    Em(f64),
    /// A theme variable (`space::MD`), resolved where the style is compiled.
    Var(&'static crate::ThemeVar<f64>),
    /// A sum (CSS `calc(..)`): px, a percentage of the reference, an `em`
    /// multiple and at most one theme variable times a factor.
    Calc {
        percent: f64,
        px: f64,
        em: f64,
        var: Option<(&'static crate::ThemeVar<f64>, f64)>,
    },
    /// (`MinContent`, `MaxContent`, `FitContent`: the docs as today)
    MinContent,
    MaxContent,
    FitContent,
    /// A sum the host cannot take; reported as `Unsupported` where used.
    Invalid,
}

/// One term list of a sum: px, percent, em and one variable with its factor.
type Terms = (f64, f64, f64, Option<(&'static crate::ThemeVar<f64>, f64)>);

impl Length {
    /// The theme variable `v` as a length.
    pub const fn var(v: &'static crate::ThemeVar<f64>) -> Length {
        Length::Var(v)
    }

    fn terms(self) -> Option<Terms> {
        match self {
            Length::Px(v) => Some((v, 0.0, 0.0, None)),
            Length::Percent(p) => Some((0.0, p, 0.0, None)),
            Length::Em(e) => Some((0.0, 0.0, e, None)),
            Length::Var(v) => Some((0.0, 0.0, 0.0, Some((v, 1.0)))),
            Length::Calc { percent, px, em, var } => Some((px, percent, em, var)),
            Length::Auto | Length::MinContent | Length::MaxContent | Length::FitContent | Length::Invalid => None,
        }
    }

    fn from_terms((px, percent, em, var): Terms) -> Length {
        match (percent != 0.0, em != 0.0, var) {
            (false, false, None) => Length::Px(px),
            (true, false, None) if px == 0.0 => Length::Percent(percent),
            (false, true, None) if px == 0.0 => Length::Em(em),
            (false, false, Some((v, k))) if px == 0.0 && k == 1.0 => Length::Var(v),
            _ => Length::Calc { percent, px, em, var },
        }
    }

    /// `self + sign * rhs`, or `Invalid` (see the type's doc).
    fn combine(self, rhs: Length, sign: f64) -> Length {
        let (Some((px1, p1, e1, v1)), Some((px2, p2, e2, v2))) = (self.terms(), rhs.terms()) else {
            return Length::Invalid;
        };
        let var = match (v1, v2) {
            (None, None) => None,
            (Some(a), None) => Some(a),
            (None, Some((v, k))) => Some((v, sign * k)),
            (Some((a, k1)), Some((b, k2))) if std::ptr::eq(a, b) => Some((a, k1 + sign * k2)),
            (Some(_), Some(_)) => return Length::Invalid,
        };
        Length::from_terms((px1 + sign * px2, p1 + sign * p2, e1 + sign * e2, var))
    }
}

impl<T: Into<Length>> std::ops::Add<T> for Length {
    type Output = Length;
    /// The sum (CSS `calc(a + b)`); see the type's doc for what it keeps.
    fn add(self, rhs: T) -> Length {
        self.combine(rhs.into(), 1.0)
    }
}

impl<T: Into<Length>> std::ops::Sub<T> for Length {
    type Output = Length;
    /// The difference (CSS `calc(a - b)`).
    fn sub(self, rhs: T) -> Length {
        self.combine(rhs.into(), -1.0)
    }
}

/// `v` times the font size (CSS `em`; see [`Length`]).
pub fn em(v: impl Into<f64>) -> Length {
    Length::Em(v.into())
}
```

     元の `fn parts`、`fn from_parts` と元の `impl Add`・`impl Sub` を消す。`pub fn is_content` は `Calc` の腕を `Length::Calc { .. }` に直す。`pub fn to_len(self) -> Len` は `Px`・`Percent`・`Calc { percent, px, .. }` を写し（`em` と `var` は解決の後には 0）、ほかは `Len::ZERO`（S18 が数える）。doc の例の `assert_eq!(pct(100) - 20, Length::Calc { percent: 100.0, px: -20.0 });` を `assert_eq!(pct(100) - 20, Length::Calc { percent: 100.0, px: -20.0, em: 0.0, var: None });` にする。`crates/kivact-types/src/lib.rs` の `pub use length::{Length, auto, pct};` を `pub use length::{Length, auto, em, pct};` にし、S13 で `text/properties.rs` に移した `pub fn em(..) -> TextLength` と、その再公開（`text.rs`・`lib.rs` の `em`）を消す（`em` は `Length` を返すものだけ）。`TextLength` は wire の型として残す（style の公開から外す：手順 2b）。
     - `crates/kivact-layout/src/style/convert.rs` の `Length` の各 `match`（S13 で `Dimension` から変わったもの）に腕を足す：`Length::Calc { percent, px, .. }` の形にし（`em`・`var` は compile が解決して 0）、`Length::Em(_) | Length::Var(_) | Length::Invalid => <Auto と同じ処理>` を `Length::Auto =>` の腕の次に足す。コメント：`// compile_style resolves em, variables and invalid sums; none reaches layout.`
     2b. **文字の長さ**：style の文字の長さの入力（`letter_spacing`、`word_spacing`、`text_decoration_thickness`、`text_underline_offset`、`text_stroke` の幅。`sheet/text_setters.rs`）の引数を `impl Into<TextLength>` から `impl Into<Length>` にし、`TextExtras` の同じ field の型を `Length` にする。同じ変更を `crates/kivact-style/src/text.rs` の `SpanStyle`・`TextDecoration`（作者の装飾）、`crates/kivact-style/src/draw/text_run.rs` の `TextFormat`（`letter_spacing`・`word_spacing` など `TextLength` の field）、`crates/kivact-core/src/view/span.rs` の `Span` の setter に行う。`crates/kivact-style/src/text.rs` に足し、wire の `TextLength` を作る所（`resolve_extras`、`compile_span_style`、`compile_text_decoration`、`text_run`）でこれを使う：
       ```rust
       /// The text length of an authored `length`: px (a number or a theme
       /// variable) or em; a percentage, a content size, `auto` or a sum with
       /// both px and em counts in `dropped` and is `None`.
       pub(crate) fn text_length(length: Length, theme: &Theme, dropped: &mut u32) -> Option<p::TextLength> {
           match length {
               Length::Px(v) => Some(p::TextLength::Px(v)),
               Length::Em(e) => Some(p::TextLength::Em(e)),
               Length::Var(v) => Some(p::TextLength::Px(theme.get(v))),
               Length::Calc { percent: 0.0, px, em, var } if em == 0.0 => {
                   Some(p::TextLength::Px(px + var.map_or(0.0, |(v, k)| k * theme.get(v))))
               }
               Length::Calc { percent: 0.0, px: 0.0, em, var: None } => Some(p::TextLength::Em(em)),
               _ => {
                   *dropped += 1;
                   None
               }
           }
       }
       ```
       `Draw` の `text_run` では `dropped` に `&mut self.unsupported` を渡す。`crates/kivact-style/src/lib.rs`・`facade.rs` の公開から `TextLength` を消す（setter は `Length` を受け、`em` は `Length` を返す）。
（新しい `color` module のための場所を空ける）：
     ```
     git mv crates/kivact-style/src/color.rs crates/kivact-style/src/color_value.rs
     git mv crates/kivact-style/src/color crates/kivact-style/src/color_value
     rg -l "crate::color::|super::color::|super::super::color::" crates/kivact-style/src | xargs perl -pi -e 's/\b(crate|super|super::super)::color::/$1::color_value::/g'
     ```
     `crates/kivact-style/src/lib.rs` の `mod color;` を `mod color_value;` に、`pub use color::{...}` を `pub use color_value::{...}` にする。
  4. **Color の表現**：`crates/kivact-style/src/color_value/value.rs`：
     - `pub enum ColorToken { ... }` を消す。`enum Repr` の `Token(ColorToken, f64)` と `Themed(u32, f64)` を消し、`/// A theme variable, with the alpha applied to what it resolves to.` `Var(&'static ThemeVar<Color>, f64),` を足す。
     - `pub fn rgb` と `pub fn rgba` を `pub const fn` にする（`Rgba::from_hex` を `crates/kivact-types/src/color.rs` で `pub const fn` にする。本体の `alpha.clamp(0.0, 1.0)` は const で使える）。
     - `impl Color` の `pub fn token(..)` を消し、`pub const fn var(v: &'static ThemeVar<Color>) -> Color { Color(Repr::Var(v, 1.0)) }` を足す。`with_alpha` の `Repr::Token`・`Repr::Themed` の腕を `Repr::Var(v, a) => Color(Repr::Var(v, (a * alpha).clamp(0.0, 1.0))),` の 1 腕にする。`impl From<ColorToken> for Color` を消す。
     - `Color` は `Copy` のまま（`&'static ThemeVar<Color>` は Copy）。`PartialEq` は derive のまま：`Var` は変数の同一性と alpha で比べる。`Hash` は実装しない（f64 を持つため 2b8df5ac でも無い）。
  5. **ThemeValue と変数の表**：`crates/kivact-style/src/theme/vars.rs` を作り、`theme.rs` に `mod vars;` と `pub use vars::{FontFamily, ThemeValue};`、`pub(crate) use vars::{VarSlot, VarTable};` を足す：

```rust
//! Role: Theme variables a theme sets: their values, storage and resolution.

use std::fmt;
use std::sync::Arc;

use kivact_types::{LineHeight, ThemeVar, ThemeVarId, VarValue};

use super::value::Theme;
use crate::color_value::Color;

/// How many aliases a lookup follows.
const MAX_ALIASES: usize = 8;

/// A font family list of a theme (`font::FAMILY_VAR`): a CSS family list,
/// or the system UI font.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FontFamily(Option<Arc<str>>);

impl FontFamily {
    /// The system UI font.
    pub const SYSTEM: FontFamily = FontFamily(None);

    /// The family list, `None` for the system UI font.
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

impl From<&str> for FontFamily {
    fn from(list: &str) -> Self {
        FontFamily(Some(Arc::from(list)))
    }
}

impl From<String> for FontFamily {
    fn from(list: String) -> Self {
        FontFamily(Some(Arc::from(list)))
    }
}

impl From<FontFamily> for VarValue<FontFamily> {
    fn from(v: FontFamily) -> Self {
        VarValue::Value(v)
    }
}

impl From<&str> for VarValue<FontFamily> {
    fn from(v: &str) -> Self {
        VarValue::Value(FontFamily::from(v))
    }
}

impl From<String> for VarValue<FontFamily> {
    fn from(v: String) -> Self {
        VarValue::Value(FontFamily::from(v))
    }
}

impl From<Color> for VarValue<Color> {
    fn from(v: Color) -> Self {
        VarValue::Value(v)
    }
}

mod sealed {
    pub trait Sealed {}
}

/// What a theme stores for a variable.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq)]
pub enum VarSlot {
    Color(VarValue<Color>),
    Number(VarValue<f64>),
    Weight(VarValue<u16>),
    LineHeight(VarValue<LineHeight>),
    Family(VarValue<FontFamily>),
}

/// A type a [`ThemeVar`] holds: [`Color`], `f64` (lengths in px), `u16`
/// (font weights), [`LineHeight`] and [`FontFamily`]. Every one may be an
/// alias of another variable of its type, as a set value and as a fallback.
pub trait ThemeValue: Clone + PartialEq + fmt::Debug + Send + Sync + 'static + sealed::Sealed {
    #[doc(hidden)]
    fn into_slot(value: VarValue<Self>) -> VarSlot;
    #[doc(hidden)]
    fn from_slot(slot: &VarSlot) -> Option<VarValue<Self>>;
    /// A value that is itself a variable (a `Color::var`), with its alpha.
    #[doc(hidden)]
    fn as_alias(&self) -> Option<(&'static ThemeVar<Self>, f64)> {
        None
    }
    /// The value with the alphas of the aliases followed applied.
    #[doc(hidden)]
    fn with_alias_alpha(self, _alpha: f64) -> Self {
        self
    }
    /// The value of a chain that does not end and whose first fallback is an alias.
    #[doc(hidden)]
    fn on_cycle() -> Self;
}

macro_rules! plain_value {
    ($($t:ty => $variant:ident, $cycle:expr;)*) => {$(
        impl sealed::Sealed for $t {}
        impl ThemeValue for $t {
            fn into_slot(value: VarValue<Self>) -> VarSlot {
                VarSlot::$variant(value)
            }
            fn from_slot(slot: &VarSlot) -> Option<VarValue<Self>> {
                match slot {
                    VarSlot::$variant(v) => Some(v.clone()),
                    _ => None,
                }
            }
            fn on_cycle() -> Self {
                $cycle
            }
        }
    )*};
}

plain_value! {
    f64 => Number, 0.0;
    u16 => Weight, 400;
    LineHeight => LineHeight, LineHeight::Normal;
    FontFamily => Family, FontFamily::SYSTEM;
}

impl sealed::Sealed for Color {}

impl ThemeValue for Color {
    fn into_slot(value: VarValue<Self>) -> VarSlot {
        VarSlot::Color(value)
    }
    fn from_slot(slot: &VarSlot) -> Option<VarValue<Self>> {
        match slot {
            VarSlot::Color(v) => Some(v.clone()),
            _ => None,
        }
    }
    fn as_alias(&self) -> Option<(&'static ThemeVar<Self>, f64)> {
        self.var_target()
    }
    fn with_alias_alpha(self, alpha: f64) -> Self {
        self.with_alpha(alpha)
    }
    fn on_cycle() -> Self {
        Color::BLACK
    }
}

/// What `var` holds in `theme`: the value it sets, else its fallback.
fn held<T: ThemeValue>(theme: &Theme, var: &'static ThemeVar<T>) -> VarValue<T> {
    theme
        .slot(ThemeVarId::of(var))
        .and_then(T::from_slot)
        .unwrap_or_else(|| var.fallback().clone())
}

/// The next variable an alias leads to, with the alpha it applies.
fn next_of<T: ThemeValue>(held: &VarValue<T>) -> Option<(&'static ThemeVar<T>, f64)> {
    match held {
        VarValue::Var(next) => Some((next, 1.0)),
        VarValue::Value(v) => v.as_alias(),
    }
}

/// The value of `var` in `theme`, aliases followed up to `MAX_ALIASES`. A
/// chain that does not end takes the first variable's fallback when that
/// is a plain value, else `T::on_cycle()`; `Theme::var_problems` names it.
pub(crate) fn lookup<T: ThemeValue>(theme: &Theme, var: &'static ThemeVar<T>) -> T {
    let mut at = var;
    let mut alpha = 1.0;
    for _ in 0..=MAX_ALIASES {
        let value = held(theme, at);
        match next_of(&value) {
            Some((next, a)) => {
                alpha *= a;
                at = next;
            }
            None => match value {
                VarValue::Value(v) => return v.with_alias_alpha(alpha),
                VarValue::Var(_) => unreachable!("invariant: next_of follows every Var"),
            },
        }
    }
    match var.fallback() {
        VarValue::Value(v) if v.as_alias().is_none() => v.clone(),
        _ => T::on_cycle(),
    }
}

/// Whether the chain starting at `start` reaches a plain value within
/// `MAX_ALIASES` steps.
fn ends<T: ThemeValue>(theme: &Theme, start: &VarValue<T>) -> bool {
    let mut value = start.clone();
    for _ in 0..=MAX_ALIASES {
        let Some((next, _)) = next_of(&value) else {
            return true;
        };
        value = held(theme, next);
    }
    false
}

/// The variables a theme sets, sorted by identity.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct VarTable(Vec<(ThemeVarId, &'static str, VarSlot)>);

impl VarTable {
    pub(crate) fn insert(&mut self, id: ThemeVarId, name: &'static str, value: VarSlot) {
        match self.0.binary_search_by_key(&id, |(k, _, _)| *k) {
            Ok(i) => self.0[i] = (id, name, value),
            Err(i) => self.0.insert(i, (id, name, value)),
        }
    }

    pub(crate) fn get(&self, id: ThemeVarId) -> Option<&VarSlot> {
        self.0
            .binary_search_by_key(&id, |(k, _, _)| *k)
            .ok()
            .map(|i| &self.0[i].2)
    }

    /// The names of the variables (of every type) whose aliases do not
    /// end within `MAX_ALIASES` steps (a cycle or too long a chain).
    pub(crate) fn problems(&self, theme: &Theme) -> Vec<&'static str> {
        self.0
            .iter()
            .filter(|(_, _, slot)| match slot {
                VarSlot::Color(v) => !ends(theme, v),
                VarSlot::Number(v) => !ends(theme, v),
                VarSlot::Weight(v) => !ends(theme, v),
                VarSlot::LineHeight(v) => !ends(theme, v),
                VarSlot::Family(v) => !ends(theme, v),
            })
            .map(|(_, name, _)| *name)
            .collect()
    }
}
```

     （`theme.rs` の再公開は `pub(crate) use vars::{VarSlot, VarTable, lookup};` も可。`Theme::get` は `super::vars::lookup` を呼ぶ。）`crates/kivact-style/src/color_value/value.rs` の `impl Color` に足す：`pub(crate) fn var_target(self) -> Option<(&'static ThemeVar<Color>, f64)> { match self.0 { Repr::Var(v, a) => Some((v, a)), _ => None } }`。
  6. **Theme**：`crates/kivact-style/src/theme/value.rs`：
     - `ThemeData` の `colors: Palette, space: Spacing, radius: Radii, text: Typography,` を `vars: VarTable,` にし、`disabled_opacity`・`control_height`・`overlay_margin` の 3 field を消す（`light_theme()` の同じ 3 行も消す。値は `control` の変数の fallback）。`impl Theme` から `disabled_opacity`、`with_disabled_opacity`、`control_height`、`with_control_height`、`overlay_margin`、`with_overlay_margin` を消す。S09 の試験（`Height` の fallback が `t.control_height()` を読む 3 つ）を `t.get(&crate::control::HEIGHT_VAR)` と `.set(&crate::control::HEIGHT_VAR, 30)`／`40` に書き換える（期待値は同じ）。`resolve/compile.rs` の `fn interactive_defaults` の `theme.disabled_opacity()` を `theme.get(&crate::control::DISABLED_OPACITY_VAR)` にする。
     - `impl Theme` から `colors`、`space`、`radius`、`text`、`with_colors`、`with_space`、`with_radius`、`with_text` を消し、次を足す：
       ```rust
       /// The theme with `var` set to `value`: a value, or another variable
       /// of the same type (an alias, CSS `--accent: var(--primary)`:
       /// `.set(&color::ACCENT_VAR, PRIMARY)`, `.set(&FIELD_RADIUS_VAR, &radius::MD_VAR)`),
       /// followed up to 8 steps; a cycle uses the fallback and is reported
       /// once (`ErrorKind::Unsupported`) where the theme is provided.
       #[must_use]
       pub fn set<T: ThemeValue>(mut self, var: &'static ThemeVar<T>, value: impl Into<VarValue<T>>) -> Theme {
           self.edit()
               .vars
               .insert(ThemeVarId::of(var), var.name(), T::into_slot(value.into()));
           self
       }

       /// The value of `var` in this theme: the value it sets, else the
       /// variable's fallback, aliases followed.
       pub fn get<T: ThemeValue>(&self, var: &'static ThemeVar<T>) -> T {
           super::vars::lookup(self, var)
       }

       pub(crate) fn slot(&self, id: ThemeVarId) -> Option<&VarSlot> {
           self.0.vars.get(id)
       }

       /// The names of the variables of this theme whose aliases do not
       /// end (reported by `install_theme`).
       #[doc(hidden)]
       pub fn var_problems(&self) -> Vec<&'static str> {
           self.0.vars.problems(self)
       }
       ```
     - `fn light_theme()` の `colors: Palette { ... }`、`space: Spacing { ... }`、`radius: Radii { ... }`、`text: Typography { ... }` の 4 つの field を `vars: VarTable::default(),` にする（値は変数の fallback：手順 7）。
     - `fn dark_theme()` を次にする（値は 2b8df5ac の `Theme::dark()` の `Palette`）：
       ```rust
       fn dark_theme() -> Theme {
           use crate::color::*;
           Theme::light()
               .with_mode(ThemeMode::Dark)
               .set(&SURFACE_VAR, rgb(0x111827))
               .set(&SURFACE_ALT_VAR, rgb(0x1f2937))
               .set(&TEXT_VAR, rgb(0xf9fafb))
               .set(&TEXT_MUTED_VAR, rgb(0x9ca3af))
               .set(&ACCENT_VAR, rgb(0x3b82f6))
               .set(&ACCENT_TEXT_VAR, rgb(0xffffff))
               .set(&BORDER_VAR, rgb(0x374151))
               .set(&FOCUS_VAR, rgb(0x60a5fa))
               .set(&DANGER_VAR, rgb(0xf87171))
               .set(&SUCCESS_VAR, rgb(0x4ade80))
               .set(&WARNING_VAR, rgb(0xfbbf24))
               .set(&OVERLAY_VAR, rgba(0x000000, 0.5))
               .set(&SHADOW_VAR, rgba(0x000000, 0.5))
               .set(&SELECTION_VAR, rgba(0x3b82f6, 0.3))
               // Text at 7.5% over Surface is close to SurfaceAlt; a press doubles it.
               .set(&HOVER_VAR, rgba(0xf9fafb, 0.075))
               .set(&PRESSED_VAR, rgba(0xf9fafb, 0.15))
               // Shadows vanish on a dark page, so a floating surface is lighter.
               .set(&RAISED_VAR, rgb(0x1f2937))
       }
       ```
       （glob import が style の規則で禁止なら、`use crate::color::{SURFACE_VAR, ..., RAISED_VAR};` と 17 個を列挙する。）
     - `impl Color` の解決：`pub fn resolve` の `Repr::Token(..)` と `Repr::Themed(..)` の腕を `Repr::Var(v, a) => theme.get(v).resolve(theme).with_alpha_mul(a),` にし、`resolve_space` も同様に `Repr::Var(v, a) => { let mut c = theme.get(v).resolve_space(theme); c.alpha *= a; c }` にする（`theme.get` は変数でない色を返すので再帰は一段で終わる）。`fn resolve_concrete`、`fn resolve_space_concrete`、`pub fn of`、`trait ThemeColor` と 2 つの impl、`trait Themed` と impl、`static THEMED`、`fn themed_index`、`fn themed_value` を消す。
     - `pub fn use_tokens` を消す。
     - `pub fn install_theme(owner: Owner, theme: Value<Theme>)` に `#[track_caller]` を付け、本体の `owner.provide(ThemeCtx(theme));` の後に足す：
       ```rust
       let at = std::panic::Location::caller();
       let reported = kivact_types::cell::ReportOnceSet::new();
       owner.effect(
           kivact_runtime::EffectKind::Render,
           move || {
               for name in theme.with(Theme::var_problems) {
                   if reported.note(name) {
                       owner.report(UiError::new(
                           ErrorKind::Unsupported,
                           format!(
                               "theme variable `{name}` is an alias that does not end within 8 steps (a cycle?); its fallback is used"
                           ),
                           Some(at),
                       ));
                   }
               }
           },
           at,
       );
       ```
       （runtime の名前は W5 の後のものにする。1.2 節。）
     - module の doc の Invariants の「`Color::components_in(theme)` and ...」の行はそのまま（S16 で直す）。`ThemeExtension` の doc の「its own colors, sizes and fonts beside the core's palette and scales (tokens)」を「its tables beside the theme's variables (recipes, policies)」にし、例の `Brand { highlight: Color }` はそのまま。
     - 試験の module の `Kit`・`Other` と 4 つの試験（`color_and_token_stay_themselves` など。`ThemeColor` を使うもの）を消し、手順「試験」の試験を書く。
  7. **中核の変数の module**：`crates/kivact-style/src/color.rs`、`space.rs`、`radius.rs`、`font.rs` を作る。名前・型・fallback（2b8df5ac の `Theme::light()` の値）は 4.2 節の表の通り。`color.rs`：

```rust
//! Role: The core color variables of a theme (`color::ACCENT`), with the light theme's values as fallbacks.

use crate::color_value::{Color, rgb, rgba};
use kivact_types::ThemeVar;

/// Declares each color variable (`X_VAR`, the `static`) and its value form
/// (`X`, the `const` a style takes).
macro_rules! color_vars {
    ($($(#[$doc:meta])* $var:ident / $value:ident = $name:literal, $fallback:expr;)*) => {$(
        $(#[$doc])*
        pub static $var: ThemeVar<Color> = ThemeVar::new($name, $fallback);
        $(#[$doc])*
        pub const $value: Color = Color::var(&$var);
    )*};
}

color_vars! {
    /// The page background.
    SURFACE_VAR / SURFACE = "surface", rgb(0xffffff);
    /// A secondary background (a header, a well).
    SURFACE_ALT_VAR / SURFACE_ALT = "surface_alt", rgb(0xf3f4f6);
    /// Body text.
    TEXT_VAR / TEXT = "text", rgb(0x111827);
    /// Secondary text.
    TEXT_MUTED_VAR / TEXT_MUTED = "text_muted", rgb(0x6b7280);
    /// The accent (primary actions, the selection).
    ACCENT_VAR / ACCENT = "accent", rgb(0x2563eb);
    /// Text on the accent.
    ACCENT_TEXT_VAR / ACCENT_TEXT = "accent_text", rgb(0xffffff);
    /// Borders and dividers.
    BORDER_VAR / BORDER = "border", rgb(0xd1d5db);
    /// The focus ring.
    FOCUS_VAR / FOCUS = "focus", rgb(0x2563eb);
    /// Destructive actions and errors.
    DANGER_VAR / DANGER = "danger", rgb(0xdc2626);
    /// Success.
    SUCCESS_VAR / SUCCESS = "success", rgb(0x16a34a);
    /// Warnings.
    WARNING_VAR / WARNING = "warning", rgb(0xd97706);
    /// The scrim behind a modal.
    OVERLAY_VAR / OVERLAY = "overlay", rgba(0x000000, 0.35);
    /// Shadows.
    SHADOW_VAR / SHADOW = "shadow", rgba(0x000000, 0.2);
    /// The band behind selected text.
    SELECTION_VAR / SELECTION = "selection", rgba(0x2563eb, 0.15);
    /// A tint over a surface under the pointer; translucent, so it works on any background.
    HOVER_VAR / HOVER = "hover", rgba(0x111827, 0.05);
    /// A tint over a pressed surface, stronger than `HOVER`.
    PRESSED_VAR / PRESSED = "pressed", rgba(0x111827, 0.10);
    /// The background of a menu, a popover or a tooltip.
    RAISED_VAR / RAISED = "raised", rgb(0xffffff);
}
```

     `space.rs`（`use kivact_types::{Length, ThemeVar};`、同じ形の `macro_rules! length_vars` で `pub static $var: ThemeVar<f64> = ThemeVar::new($name, $fallback); pub const $value: Length = Length::var(&$var);`）：`XS_VAR / XS = "space.xs", 2.0`、`SM_VAR / SM = "space.sm", 4.0`、`MD_VAR / MD = "space.md", 8.0`、`LG_VAR / LG = "space.lg", 12.0`、`XL_VAR / XL = "space.xl", 16.0`。
     `radius.rs`（同じ形）：`SM_VAR / SM = "radius.sm", 3.0`、`MD_VAR / MD = "radius.md", 6.0`、`LG_VAR / LG = "radius.lg", 10.0`。
     `font.rs`：`SIZE_VAR / SIZE = "font.size", 14.0`、`SIZE_SMALL_VAR / SIZE_SMALL = "font.size_small", 12.0`、`SIZE_LARGE_VAR / SIZE_LARGE = "font.size_large", 18.0`（`Length`）と、値の形を持たない 4 つ：
     ```rust
     /// The weight of body text.
     pub static WEIGHT_VAR: ThemeVar<u16> = ThemeVar::new("font.weight", 400);
     /// The weight of strong text (a heading, a selected tab).
     pub static WEIGHT_STRONG_VAR: ThemeVar<u16> = ThemeVar::new("font.weight_strong", 600);
     /// The line height of text that sets none (the root's text style
     /// carries it and every text below inherits it).
     pub static LINE_HEIGHT_VAR: ThemeVar<LineHeight> = ThemeVar::new("font.line_height", LineHeight::Scale(1.35));
     /// The family list of text that sets none.
     pub static FAMILY_VAR: ThemeVar<FontFamily> = ThemeVar::new("font.family", FontFamily::SYSTEM);
     ```
     `control.rs`（`//! Role: The core control metrics of a theme: control height, disabled opacity and overlay margin.`。親の決定：2b8df5ac の `Theme` の field `control_height`・`disabled_opacity`・`overlay_margin` を変数にする）：`length_vars!` で `HEIGHT_VAR / HEIGHT = "control.height", 28.0` と `OVERLAY_MARGIN_VAR / OVERLAY_MARGIN = "control.overlay_margin", 0.0`、それに値の形を持たない `/// The opacity a disabled focusable element is drawn with, unless its style's disabled state sets one.` `pub static DISABLED_OPACITY_VAR: ThemeVar<f64> = ThemeVar::new("control.disabled_opacity", 0.5);`。`focus_ring`・`focus_ring_offset` は `Theme` の field のまま（style の一部。親の決定）。
     `macro_rules! length_vars` は `space.rs` に置いて `pub(crate) use length_vars;` で `radius.rs`・`font.rs`・`control.rs` から使う（同じ macro を何度も書かない）。`crates/kivact-style/src/lib.rs` に `pub mod color; pub mod control; pub mod font; pub mod radius; pub mod space;` を足し、`pub use kivact_types::ThemeVar;`、`pub use theme::{FontFamily, ThemeValue};` を足す。`crates/kivact-style/src/facade.rs` の一覧に `ThemeVar, ThemeValue, VarValue, FontFamily, color, control, font, radius, space` を足し（`pub use kivact_types::VarValue;` も `lib.rs` に足す）、`ColorToken`、`ThemeColor`、`use_tokens` を消す。`crates/kivact-style/src/lib.rs` の `pub use color_value::{Color, ColorToken, ...}` から `ColorToken` を、`pub use theme::value::{...}` から `ThemeColor`、`use_tokens` を消す。`crates/kivact-style/src/theme.rs` の `pub use tokens::{Palette, Radii, Spacing, Typography};` と `mod tokens;` を消し、`git rm crates/kivact-style/src/theme/tokens.rs`。
  8. **style の中の利用**：
     - setter と builder の色の引数：`rg -l "ThemeColor" crates/kivact-style/src | xargs perl -pi -e 's/impl ThemeColor\b/impl Into<Color>/g; s/\b(\w+): ThemeColor\b/$1: Into<Color>/g; s/Color::of\((\w+)\)/$1.into()/g; s/\.map\(Color::of\)/.map(Into::into)/g'`。残った `Color::of(`（引数が式のもの。`rg -n "Color::of\(" crates/kivact-style/src`）は `Color::from(<式>)` にする。`use crate::theme::value::ThemeColor;` の行は `cargo fix` で消す。
     - `crates/kivact-style/src/resolve/compile.rs`：`ColorToken::Focus` を `crate::color::FOCUS` に。root の text の既定（`if state.root { ... }`）を次にする：
       ```rust
       t.font_size = t.font_size.or(Some(theme.get(&crate::font::SIZE_VAR)));
       t.weight = t.weight.or(Some(theme.get(&crate::font::WEIGHT_VAR)));
       t.color = t.color.or(Some(crate::color::TEXT.resolve(theme)));
       t.family = t.family.or_else(|| theme.get(&crate::font::FAMILY_VAR).as_str().map(str::to_owned));
       t.line_height = t.line_height.or(Some(theme.get(&crate::font::LINE_HEIGHT_VAR)));
       ```
     - `crates/kivact-style/src/resolve/paint.rs` の `fn border` の `.unwrap_or(theme.colors().border)` を `.unwrap_or(crate::color::BORDER)` に。
     - `Length` を解決する（変数・`em`・式。UI-S-03）：`crates/kivact-style/src/resolve/compile.rs` に足す：
       ```rust
       /// `l` with its theme variables and `em` resolved (`em_px` is the font
       /// size `em` is of); an `Invalid` sum counts in `misplaced` and is `Auto`.
       pub(crate) fn resolved(l: Length, theme: &Theme, em_px: f64, misplaced: &mut u32) -> Length {
           match l {
               Length::Var(v) => Length::Px(theme.get(v)),
               Length::Em(e) => Length::Px(e * em_px),
               Length::Calc { percent, px, em, var } => {
                   let px = px + em * em_px + var.map_or(0.0, |(v, k)| k * theme.get(v));
                   if percent == 0.0 {
                       Length::Px(px)
                   } else {
                       Length::Calc { percent, px, em: 0.0, var: None }
                   }
               }
               Length::Invalid => {
                   *misplaced += 1;
                   Length::Auto
               }
               other => other,
           }
       }
       ```
       `compile_style` の中で `em_px` を一度作る：その style が設定する `font_size`（`Px`、変数、`em`（基準は `font::SIZE`）の解決）があればその px、無ければ `theme.get(&crate::font::SIZE_VAR)`。`fn dim(l, default)` を `fn dim(l: Option<Length>, default: Length, cx: &mut Resolve<'_>) -> Length { l.map(|l| resolved(l, cx.theme, cx.em_px, cx.misplaced)).unwrap_or(default) }` にし、`struct Resolve<'a> { theme: &'a Theme, em_px: f64, misplaced: &'a mut u32 }` を `compile.rs` に置いて `fn layout`・`fn edges` に `&mut Resolve` を渡す（`misplaced` は `CompiledStyle.misplaced_content` に足す。S18 が `misplaced_lengths` に改名する）。`Length::to_len` を呼ぶ所（paint.rs、geometry.rs の `corner`、paint/*.rs の位置・大きさ。`rg -n "to_len\(\)" crates/kivact-style/src`）は `resolved(l, theme, em_px, misplaced).to_len()` にする（theme・`em_px`・`misplaced` を引数に持たない関数は、その関数に `cx: &mut Resolve<'_>` を足して呼出しを直す）。
  9. **掃引**（範囲の全体。style 自身は `crate`、core と kivact-test は `kivact_style`、interaction と ui は `kivact_core::style`、それ以外は `kivact::style` を prefix にする）。次の script を scratch に `theme_vars.pl` として保存する：

```perl
#!/usr/bin/perl
# usage: perl theme_vars.pl PREFIX FILE...
use strict;
use warnings;
my $p = shift @ARGV;
sub up { my $c = shift; return uc join('_', $c =~ /([A-Z][a-z0-9]*)/g); }
for my $f (@ARGV) {
    open my $in, '<', $f or die "$f: $!";
    my @lines = <$in>;
    close $in;
    for (@lines) {
        my $q = /^\s*\/\/[\/!]/ ? 'kivact::style' : $p;
        s/Color::token\(\s*(?:\w+::)*ColorToken::(\w+)\s*\)/"${q}::color::".up($1)/ge;
        s/(?:\b\w+::)*\bColorToken::(\w+)/"${q}::color::".up($1)/ge;
        s/\.colors\(\)\.(surface_alt|text_muted|accent_text|surface|text|accent|border|focus|danger|success|warning|overlay|shadow|selection|hover|pressed|raised)\b/".get(&${q}::color::".uc($1)."_VAR)"/ge;
        s/\.space\(\)\.(xs|sm|md|lg|xl)\b/".get(&${q}::space::".uc($1)."_VAR)"/ge;
        s/\.radius\(\)\.(sm|md|lg)\b/".get(&${q}::radius::".uc($1)."_VAR)"/ge;
        s/\.text\(\)\.(size_small|size_large|size|weight_strong|weight|line_height)\b/".get(&${q}::font::".uc($1)."_VAR)"/ge;
        s/\buse_tokens::<([^>]+)>\(\)/"${q}::use_theme().with(${q}::Theme::ext::<$1>)"/ge;
        s/\.control_height\(\)/".get(&${q}::control::HEIGHT_VAR)"/ge;
        s/\.disabled_opacity\(\)/".get(&${q}::control::DISABLED_OPACITY_VAR)"/ge;
        s/\.overlay_margin\(\)/".get(&${q}::control::OVERLAY_MARGIN_VAR)"/ge;
        s/\.with_control_height\(/".set(&${q}::control::HEIGHT_VAR, "/ge;
        s/\.with_disabled_opacity\(/".set(&${q}::control::DISABLED_OPACITY_VAR, "/ge;
        s/\.with_overlay_margin\(/".set(&${q}::control::OVERLAY_MARGIN_VAR, "/ge;
    }
    open my $out, '>', $f or die "$f: $!";
    print $out @lines;
    close $out;
}
```

     ```
     perl theme_vars.pl crate $(rg -l "ColorToken|\.colors\(\)|\.space\(\)|\.radius\(\)|\.text\(\)\.|use_tokens|control_height|disabled_opacity|overlay_margin" crates/kivact-style/src)
     perl theme_vars.pl kivact_style $(rg -l "ColorToken|\.colors\(\)|\.space\(\)|\.radius\(\)|\.text\(\)\.|use_tokens|control_height|disabled_opacity|overlay_margin" crates/kivact-core/src crates/kivact-test/src)
     perl theme_vars.pl kivact_core::style $(rg -l "ColorToken|\.colors\(\)|\.space\(\)|\.radius\(\)|\.text\(\)\.|use_tokens|control_height|disabled_opacity|overlay_margin" crates/kivact-interaction/src crates/kivact-ui/src)
     perl theme_vars.pl kivact::style $(rg -l "ColorToken|\.colors\(\)|\.space\(\)|\.radius\(\)|\.text\(\)\.|use_tokens|control_height|disabled_opacity|overlay_margin" crates/kivact-devtools crates/kivact/src $W6_TESTS dev/bench examples/kivact templates/basic)
     ```
     置換は同じ型を返す読みに置き換える（`theme.space().md`（f64）→ `theme.get(&space::MD_VAR)`（f64）、`theme.colors().accent`（Color）→ `theme.get(&color::ACCENT_VAR)`（Color）、`ColorToken::Accent`（`impl Into<Color>` に渡す）→ `color::ACCENT`（Color））。`git diff` で `.text().` を置換した行のうち、受け手が `Theme` でないもの（`Locator::text()` など。変数名が `theme`・`t`・`th` 以外のもの）があれば戻す。
  10. 手で直す残り（`rg -n "ColorToken|ThemeColor|Color::of|Color::token|Palette|Spacing|Radii|Typography|with_colors|with_space|with_radius|with_text|\.colors\(\)|\.text\(\)\.family|use_tokens" $W6_SCOPE $W6_TESTS --type rust` が返す行。`CommandPalette` などの別の名前は除く）。規則：
      - `theme.with_colors(Palette { a: X, b: Y, ..theme.colors().clone() })` → `theme.set(&color::A_VAR, X).set(&color::B_VAR, Y)`（変えた field ごとに一つ）。
      - `theme.with_space(Spacing { md: X, .. })` → `.set(&space::MD_VAR, X)`；`with_radius` も同じ。
      - `let text = Typography { family: Some(F), ..theme.text().clone() }; theme.with_text(text)` → `theme.set(&font::FAMILY_VAR, F)`（`F` が `String` なら `FontFamily::from(F)` は `impl Into` で通る）。size などの field も対応する変数に。
      - `theme.text().family.clone()`（`Option<String>`）→ `theme.get(&font::FAMILY_VAR).as_str().map(str::to_owned)`。2b8df5ac の該当：`crates/kivact-core/src/custom/text_shaping.rs`、`crates/kivact-offscreen/src/lib.rs`。
      - `crates/kivact-core/src/style/theme.rs` を `git rm` し、`crates/kivact-core/src/style.rs` の `pub mod theme;` を消す。
      - `crates/kivact-core/src/devtools/apply.rs`（付属書 C の C01 が W2b で `view/devtools_apply.rs` から移した場所）の色・大きさの token の関数（`apply_token`、`apply_space`、`apply_radius`、`apply_text`、`set_palette`、`COLOR_TOKENS`、`color_token`、`token_list`）を、変数の表で書き直す：
        ```rust
        const COLOR_TOKENS: [(&str, &'static ThemeVar<Color>); 17] = [
            ("surface", &color::SURFACE_VAR), ("surface_alt", &color::SURFACE_ALT_VAR),
            ("text", &color::TEXT_VAR), ("text_muted", &color::TEXT_MUTED_VAR),
            ("accent", &color::ACCENT_VAR), ("accent_text", &color::ACCENT_TEXT_VAR),
            ("border", &color::BORDER_VAR), ("focus", &color::FOCUS_VAR),
            ("danger", &color::DANGER_VAR), ("success", &color::SUCCESS_VAR),
            ("warning", &color::WARNING_VAR), ("overlay", &color::OVERLAY_VAR),
            ("shadow", &color::SHADOW_VAR), ("selection", &color::SELECTION_VAR),
            ("hover", &color::HOVER_VAR), ("pressed", &color::PRESSED_VAR),
            ("raised", &color::RAISED_VAR),
        ];
        const LENGTH_TOKENS: [(&str, &'static ThemeVar<f64>); 14] = [
            ("disabled_opacity", &control::DISABLED_OPACITY_VAR), ("control_height", &control::HEIGHT_VAR),
            ("overlay_margin", &control::OVERLAY_MARGIN_VAR),
            ("space.xs", &space::XS_VAR), ("space.sm", &space::SM_VAR), ("space.md", &space::MD_VAR),
            ("space.lg", &space::LG_VAR), ("space.xl", &space::XL_VAR),
            ("radius.sm", &radius::SM_VAR), ("radius.md", &radius::MD_VAR), ("radius.lg", &radius::LG_VAR),
            ("text.size", &font::SIZE_VAR), ("text.size_small", &font::SIZE_SMALL_VAR),
            ("text.size_large", &font::SIZE_LARGE_VAR),
        ];
        const WEIGHT_TOKENS: [(&str, &'static ThemeVar<u16>); 2] = [
            ("text.weight", &font::WEIGHT_VAR), ("text.weight_strong", &font::WEIGHT_STRONG_VAR),
        ];
        ```
        DevTools の token の名前（`"surface"`、`"space.md"`、`"text.size"`、`"text.line_height"`、`"focus_ring"` など）と protocol の `TokenKind` は変えない（protocol を変えない。親の決定。soft の `TokenKind` は付属書 K の K41 が消す。INC-38）。`apply_token` は：色の名前なら `theme.set(var, color)`、`LENGTH_TOKENS` の名前なら `theme.set(var, number)`（`disabled_opacity`・`control_height`・`overlay_margin` も、今の `apply_scalar` から外してこの表で扱う）、`WEIGHT_TOKENS` なら `theme.set(var, number as u16)`、`"text.line_height"` なら `theme.set(&font::LINE_HEIGHT_VAR, value)`、それ以外は今の `apply_scalar`。`token_list` は同じ表を回して `theme.get(var)` を読む（色は `hex(theme.get(var).components_in(theme))`。`components_in` は S16 で `components` に改名される）。`Color::token(token)` を返していた所（`fn color` の token 名の解釈）は `Color::var(var)` を返す。
      - `crates/kivact-ui/src/data_table/sticky_paint.rs` の `fn over` は `.resolve(theme)` を使い続ける（S16 で直す）。
  11. **prelude**：`crates/kivact/src/prelude.rs` の `pub use crate::style::{...}` から `ColorToken`、`ThemeColor`、`use_tokens` を消し（付属書 C の W5 がすでに消していればそのまま）、`color, font, radius, space` を足す（設計書 5.11 の一覧）。
- **消すもの**：`ColorToken`、`Color::token`、`Color::of`、`ThemeColor`、`Themed`、`THEMED`、`Repr::Token`、`Repr::Themed`、`Palette`、`Spacing`、`Radii`、`Typography`（`theme/tokens.rs`）、`Theme::{colors, space, radius, text, with_colors, with_space, with_radius, with_text}`、`use_tokens`、core の `style::theme` module。
- **状態の閉じ方**：`ThemeVar` は field 二つ（名前と fallback、どちらも constructor だけが書く）：6.1 の対象外。`VarTable` は field 一つ。`FontFamily` は field 一つ。
- **試験**：
  - `crates/kivact-style/src/theme/value.rs` の `mod tests` に：`a_variable_takes_the_theme_value_or_its_fallback`（`Theme::light().get(&color::ACCENT_VAR) == rgb(0x2563eb)`、`.set(&color::ACCENT_VAR, rgb(0xff0000))` の後は `rgb(0xff0000)`）、`an_alias_resolves_through_another_variable`（試験の中の `static PRIMARY_VAR: ThemeVar<Color> = ThemeVar::new("test.primary", rgb(0x00ff00));`、`Theme::light().set(&color::ACCENT_VAR, Color::var(&PRIMARY_VAR)).get(&color::ACCENT_VAR).resolve(&..)` が緑）、`alpha_multiplies_through_an_alias`（`Color::var(&PRIMARY_VAR).with_alpha(0.5)` を設定し、解決の alpha が 0.5）、`a_cycle_takes_the_fallback_and_is_named`（`static A_VAR`（fallback `rgb(0x010203)`）と `B_VAR` を互いに別名にし、`get(&A_VAR)` が `rgb(0x010203)`、`var_problems()` が `"test.a"` と `"test.b"` を含む）、`dark_sets_the_core_colors`（`Theme::dark().get(&color::SURFACE_VAR) == rgb(0x111827)`）。
  - UI-U-02（すべての型の別名）：同じ `mod tests` に `fn every_type_aliases_as_a_value_and_as_a_fallback()`：試験の中の `static FIELD_VAR: ThemeVar<f64> = ThemeVar::alias("test.field", &radius::MD_VAR);`、`static WEIGHT_ALIAS_VAR: ThemeVar<u16> = ThemeVar::alias("test.weight", &font::WEIGHT_STRONG_VAR);`、`static FAMILY_ALIAS_VAR: ThemeVar<FontFamily> = ThemeVar::new("test.family", FontFamily::SYSTEM);`。`Theme::light().get(&FIELD_VAR) == 6.0`；`Theme::light().set(&radius::MD_VAR, 20).get(&FIELD_VAR) == 20.0`（共通の値が届く）；`Theme::light().set(&radius::MD_VAR, 20).set(&FIELD_VAR, 4).get(&FIELD_VAR) == 4.0`（個別の値が勝つ）；`Theme::light().get(&WEIGHT_ALIAS_VAR) == 600`；`Theme::light().set(&FAMILY_ALIAS_VAR, &font::FAMILY_VAR).set(&font::FAMILY_VAR, "Inter").get(&FAMILY_ALIAS_VAR).as_str() == Some("Inter")`（set の値としての別名）；`LineHeight` も `set(&X, &font::LINE_HEIGHT_VAR)` で同じ。`fn a_number_cycle_takes_the_fallback_and_is_named()`：`f64` の A・B を互いの別名にした theme で、A の fallback が値なら `get(&A)` がその値、`var_problems()` が A と B の名前を含む。
  - UI-S-03（式）：`crates/kivact-types/src/length.rs` に `#[cfg(test)] mod tests`（`//! Role: Tests of length sums.`）：`fn a_sum_keeps_its_variable()`（試験の中の `static V: ThemeVar<f64> = ThemeVar::new("t.v", 8.0);` で `Length::var(&V) + 2 == Length::Calc { percent: 0.0, px: 2.0, em: 0.0, var: Some((&V, 1.0)) }`）、`fn sums_of_like_terms_collapse()`（`pct(50) - pct(50) + 3 == Length::Px(3.0)`、`em(1) + em(0.5) == Length::Em(1.5)`）、`fn unsupported_sums_are_invalid()`（`Length::Auto + 2 == Length::Invalid`、`Length::var(&V) - Length::var(&W) == Length::Invalid`（別の変数）、`Length::MaxContent - 1 == Length::Invalid`）。`crates/kivact-style/src/resolve/compile.rs` の `mod tests`（無ければ作る：`//! Role: Tests of style compilation.`）に `fn a_sum_with_a_variable_resolves_in_each_theme()`（`style().width(space::MD + 2)` を `Theme::light()` で compile した `layout.width == Length::Px(10.0)`、`Theme::light().set(&space::MD_VAR, 20)` で `Length::Px(22.0)`）、`fn em_is_of_the_font_size_the_style_sets()`（`style().font_size(10).width(em(2))` の `layout.width == Length::Px(20.0)`、`style().width(em(2))` は `Length::Px(28.0)`（`font::SIZE` 14））、`fn an_invalid_sum_is_counted()`（`style().width(Length::Auto + 2)` の `misplaced_content == 1` で `layout.width == Length::Auto`）。
  - 公開 E2E：`dev/tests/kivact-public-api/tests/e2e_core/theme_vars.rs`（binary の `main.rs` に `mod theme_vars;` を足す）に 2 つ：
    - `fn a_theme_variable_restyles_on_a_theme_switch()`：`let theme = signal(Theme::light());` の `provide_theme(move || theme.get(), node().style(style().size(20, 20).background(color::ACCENT)))` を描き、`screen.screenshot()` の (10, 10) が `rgb(0x2563eb)`（`shaders.rs` の `expect(.., tolerance 2, ..)` と同じ helper）；`theme.set(Theme::light().set(&color::ACCENT_VAR, rgb(0xff0000)))` の後 settle して赤。style に closure を使わないこと（ST-01 の契約）。
    - `fn a_variable_cycle_is_reported_once()`：A と B を互いの別名にした theme を provide し、`screen.errors()` のうち `ErrorKind::Unsupported` で message が `theme variable` を含むものが、theme の再描画（signal を同じ値で 2 回 set）の後も 2 件（A と B）であること。
    - `fn a_sum_with_a_variable_follows_the_theme()`：`node().test_id("box").style(style().height(10).width(space::MD + 2))` を `provide_theme(move || theme.get(), ..)` の中に描き、box の幅が 10；`theme.set(Theme::light().set(&space::MD_VAR, 20))` の後 22（box の大きさは同じ binary の `layout_units.rs` の既存の試験の読み方で読む）。
    - `fn an_invalid_sum_is_reported()`：`style().width(Length::Auto + 2)` の node を描き、`screen.errors()` に `ErrorKind::Unsupported` で message が `length` を含むものが 1 件。
- **確認（tier a）**：鎖の共通確認と次：
  ```
  cargo clippy -p kivact-core -p kivact-ui -p kivact-interaction -p kivact-devtools -p kivact-test -p kivact-layout --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-types
  cargo test -p kivact-style
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- theme_vars
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native -- devtools
  ```
- **完了条件**：`rg -n -w "ColorToken|ThemeColor|Palette|Spacing|Radii|Typography|use_tokens|THEMED" $W6_SCOPE $W6_TESTS --type rust` が 0 件（`CommandPalette` などの別の識別子の一部は `-w` で除かれる）。上のコマンドが通る。

---

## S16：theme を受ける色の計算（ST-03）

- **wave / lane / 前提**：W6b / 鎖 / S15
- **所有**：`crates/kivact-style/src/theme/value.rs`、`crates/kivact-style/src/color_value/value.rs`、`crates/kivact-style/src/internal.rs`、`crates/kivact-style/src/lib.rs`、W6 の掃引の範囲（`components`・`mix`・`resolve` の呼出し）
- **目的**：ST-03。theme なしで既定の light の theme で解決する `Color::components()`・`Color::mix()` を消し、`components(&Theme)`・`mix(other, t, &Theme)` にする。`Animatable for Color` と輝度は、その時の theme（`try_use_theme`）で変数を解決する。`Color::resolve` は公開をやめる（ST-22）。
- **手順**：
  1. `crates/kivact-style/src/theme/value.rs` の `impl Color`：
     - `pub fn components(self) -> [f64; 4]`（theme なし）と `pub fn mix(self, other, amount)`（theme なし）を消す。
     - `pub fn components_in(self, theme: &Theme)` を `pub fn components(self, theme: &Theme) -> [f64; 4]` に、`pub fn mix_in(self, other: Color, amount: impl Into<f64>, theme: &Theme)` を `pub fn mix(self, other: Color, amount: impl Into<f64>, theme: &Theme) -> Color` に改名する。doc を「Components in sRGB, resolved in `theme`.」「Mix in Oklab (premultiplied) after resolving both colors in `theme`; `amount = 0` is self, `1` is other. The result keeps its space until it is painted.」にする。
     - `pub fn resolve` を `pub(crate) fn resolve` にし、`resolve_space` も `pub(crate)` にする。
     - `crates/kivact-style/src/internal.rs` に足す：`/// The color as the engine paints it, for core's text editing and custom paint bridges.` `pub fn resolve_color(color: Color, theme: &Theme) -> kivact_types::Rgba { color.resolve(theme) }`。
  2. ambient の theme：同じファイルに足す：
     ```rust
     /// The theme in effect for the current owner, if there is one: like
     /// [`use_theme`] but never a misuse and untracked (animation and
     /// luminance read it outside setup).
     pub fn try_use_theme() -> Option<Theme> {
         let owner = Owner::try_ambient()?;
         theme_of(owner).map(|theme| kivact_runtime::untrack(|| theme.get()))
     }
     ```
     `Owner::try_ambient` が runtime に無ければ、`Owner::ambient` の失敗しない版として `crates/kivact-runtime/src/reactive/owner_handle.rs` に `pub fn try_ambient() -> Option<Owner>`（`ambient` の本体のうち、owner が無いときに misuse を記録する部分を `None` を返すようにしたもの）を足す（W6 の鎖は runtime を所有する）。S09 の `fn ambient_theme()` の本体を `try_use_theme().unwrap_or_else(Theme::light)` にする。`impl kivact_runtime::Animatable for Color` の `to_components` の `self.resolve_space(&Theme::default())` を `self.resolve_space(&ambient_theme())` にする。`crates/kivact-style/src/lib.rs` と `facade.rs` の公開に `try_use_theme` を足す。
  3. 呼出しを直す：
     ```
     rg -l "components_in\(|mix_in\(" $W6_SCOPE $W6_TESTS --type rust | xargs perl -pi -e 's/\.components_in\(/.components(/g; s/\.mix_in\(/.mix(/g'
     ```
     theme なしの `.components()`（`rg -n "\.components\(\)" $W6_SCOPE $W6_TESTS`。2b8df5ac では `crates/kivact-devtools` に 2 か所）は、その場の theme（`use_theme().get()` か、関数が持つ `theme`）を渡す形 `.components(&theme)` にする。theme なしの `.mix(a, b)`（2 引数）は `.mix(a, b, &theme)` にする（`rg -n "\.mix\([^,()]*,[^,()]*\)" ...`）。
     `.resolve(&theme)`・`.resolve(theme)` を style の外で呼ぶ所：`crates/kivact-core/src/custom/text_editing.rs` は `kivact_style::internal::resolve_color(c, &theme)` に。`crates/kivact-ui/src/data_table/sticky_paint.rs` の `fn over` を次にする：
     ```rust
     fn over(theme: &Theme, overlay: Color, surface: Color) -> Color {
         let [tr, tg, tb, ta] = overlay.components(theme);
         let [br, bg, bb, _] = surface.components(theme);
         let mix = |a: f64, b: f64| a * ta + b * (1.0 - ta);
         Color::from_rgba_f64(mix(tr, br), mix(tg, bg), mix(tb, bb), 1.0)
     }
     ```
     ほかに style の外の `Color::resolve` があれば（`rg -n "\.resolve\((&)?theme\)" $W6_SCOPE $W6_TESTS` のうち `Color` の値のもの）、core なら `resolve_color`、それ以外は `components(theme)` を使う。
  4. module の doc（Invariants）の「`Color::components_in(theme)` and `mix_in(other, amount, theme)` resolve tokens in the given theme; `components` and `mix` use the default theme.」を「`Color::components(theme)` and `mix(other, amount, theme)` resolve variables in the given theme; animation and luminance resolve them in the theme in effect (`try_use_theme`).」にする。
- **消すもの**：theme なしの `Color::components`・`Color::mix`、`components_in`・`mix_in` の名前、公開の `Color::resolve`・`Color::resolve_space`。
- **状態の閉じ方**：新しい型なし。
- **試験**：`crates/kivact-style/src/theme/value.rs` の `mod tests` に `fn components_and_mix_resolve_in_the_given_theme()`（`color::SURFACE.components(&Theme::dark())` が `rgb(0x111827)` の成分、`color::SURFACE.mix(color::TEXT, 0.0, &Theme::dark())` の `components(&Theme::dark())` が同じ色。Oklab の往復の丸めがあるので、各成分の差が `1e-9` 以下で比べる）。公開 E2E の `motion.rs` の色の animation の試験がそのまま通ること。
- **確認（tier a）**：鎖の共通確認と次：
  ```
  cargo test -p kivact-style
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- motion
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- theme_vars
  ```
- **完了条件**：`rg -n "components_in|mix_in" $W6_SCOPE $W6_TESTS` が 0 件。`rg -n "pub fn resolve\(self" crates/kivact-style/src` が 0 件。上のコマンドが通る。

---

## S17：宣言の列と一つの property の表（ST-08 の前半）、`text_decoration_style`（ST-12）

- **wave / lane / 前提**：W6b / 鎖 / S16
- **所有**：`crates/kivact-style/src/internal.rs`、`crates/kivact-style/src/sheet.rs`、`crates/kivact-style/src/sheet/`（全体。新設 `decl.rs`・`props.rs`・`shorthands.rs`・`edit_text.rs`、削除 `setters.rs`・`text_setters.rs`・`grid_setters.rs`・`style.rs`）、`crates/kivact-style/src/resolve/compile.rs`、`crates/kivact-style/src/resolve/describe.rs`、`crates/kivact-style/src/resolve/paint.rs`、`crates/kivact-style/src/text.rs`、`crates/kivact-style/src/grid.rs`、W6 の掃引の範囲（`text_decoration_style` と `font_size` の呼出しの型の直し）
- **目的**：ST-08。`Style` を「宣言（`Decl`）の列」にし、property の一覧を `sheet/props.rs` の一つの表から生成する（enum `Decl`、setter、名前、分類）。後の宣言が勝ち、`merge` は列の連結。ST-12：`text_decoration_style(v: TextDecorationStyle)`（`IntoValue` をやめる）。ST-30：`duplicates` を `PartialEq` から外す。解決（resolve）はこの unit では宣言の列を旧い group の record に **畳んで（fold）** から読む（S18 が性能の形に直す）。setter の名前はこの unit では変えない（S20 が CSS の名前にする）。`font_size` は `Length` を受ける（`font::SIZE` を値として置けるように。数は今まで通り px）。
- **手順**：
  1. **property の表**（正本）。`crates/kivact-style/src/sheet/props.rs` に、次の表の行をそのまま macro の行として書く。列：`Decl` の variant と値の型、setter（この unit の名前。`—` は setter を生成しない longhand で、shorthand（手順 4）だけが作る）、setter の引数と値の式、分類（`Delta`：host が状態の差分として塗れる。`Rest`：それ以外）、group（describe の並びと `has_text` に使う）、畳む先（S17 の `fold` が書く旧い group の field）。

     | variant(型) | setter(引数) = 値 | class | group | 畳む先 |
     |---|---|---|---|---|
     | `Width(Length)` | `width(v: impl Into<Length>) = v.into()` | Rest | Size | `size.width` |
     | `Height(Length)` | `height(..)` 同 | Rest | Size | `size.height` |
     | `MinWidth(Length)` | `min_width(..)` 同 | Rest | Size | `size.min_width` |
     | `MinHeight(Length)` | `min_height(..)` 同 | Rest | Size | `size.min_height` |
     | `MaxWidth(Length)` | `max_width(..)` 同 | Rest | Size | `size.max_width` |
     | `MaxHeight(Length)` | `max_height(..)` 同 | Rest | Size | `size.max_height` |
     | `FlexBasis(Length)` | `basis(v: impl Into<Length>) = v.into()` | Rest | Size | `size.basis` |
     | `AspectRatio(f64)` | `aspect_ratio(v: impl Into<f64>) = v.into()` | Rest | Size | `size.aspect_ratio` |
     | `FlexGrow(f64)` | `grow(v: impl Into<f64>) = v.into()` | Rest | Size | `size.grow` |
     | `FlexShrink(f64)` | `shrink(v: impl Into<f64>) = v.into()` | Rest | Size | `size.shrink` |
     | `PaddingTop(Length)` | `padding_top(v: impl Into<Length>) = v.into()` | Rest | Spacing | `spacing.padding.top` |
     | `PaddingRight(Length)` | `padding_right(..)` 同 | Rest | Spacing | `spacing.padding.right` |
     | `PaddingBottom(Length)` | `padding_bottom(..)` 同 | Rest | Spacing | `spacing.padding.bottom` |
     | `PaddingLeft(Length)` | `padding_left(..)` 同 | Rest | Spacing | `spacing.padding.left` |
     | `MarginTop(Length)` | `margin_top(..)` 同 | Rest | Spacing | `spacing.margin.top` |
     | `MarginRight(Length)` | `margin_right(..)` 同 | Rest | Spacing | `spacing.margin.right` |
     | `MarginBottom(Length)` | `margin_bottom(..)` 同 | Rest | Spacing | `spacing.margin.bottom` |
     | `MarginLeft(Length)` | `margin_left(..)` 同 | Rest | Spacing | `spacing.margin.left` |
     | `Top(Length)` | `top(..)` 同 | Rest | Spacing | `spacing.inset.top` |
     | `Right(Length)` | `right(..)` 同 | Rest | Spacing | `spacing.inset.right` |
     | `Bottom(Length)` | `bottom(..)` 同 | Rest | Spacing | `spacing.inset.bottom` |
     | `Left(Length)` | `left(..)` 同 | Rest | Spacing | `spacing.inset.left` |
     | `RowGap(Length)` | `row_gap(..)` 同 | Rest | Spacing | `spacing.row_gap` |
     | `ColumnGap(Length)` | `column_gap(..)` 同 | Rest | Spacing | `spacing.column_gap` |
     | `FlexDirection(Direction)` | `direction(v: Direction) = v` | Rest | Flow | `flow.direction` |
     | `AlignItems(Align)` | `align_items(v: Align) = v` | Rest | Flow | `flow.align` |
     | `AlignSelf(Align)` | `align_self(v: Align) = v` | Rest | Flow | `flow.align_self` |
     | `JustifyContent(Justify)` | `justify_content(v: Justify) = v` | Rest | Flow | `flow.justify` |
     | `AlignContent(Justify)` | `align_content(v: Justify) = v` | Rest | Flow | `flow.align_content` |
     | `Display(Display)` | `display(v: Display) = v` | Rest | Flow | `flow.display` |
     | `Visibility(Visibility)` | `visibility(v: Visibility) = v` | Rest | Flow | `flow.visibility` |
     | `FlexWrap(bool)` | `wrap(v: bool) = v` | Rest | Flow | `flow.wrap` |
     | `Position(Position)` | `position(v: Position) = v` | Rest | Flow | `flow.position` |
     | `Overflow(Overflow)` | `overflow(v: Overflow) = v` | Rest | Flow | `flow.overflow` |
     | `ScrollAxes(ScrollAxes)` | —（`with_scroll_axes` が作る） | Rest | Flow | `flow.scroll_axes` |
     | `ScrollSnap(ScrollSnap)` | `scroll_snap(v: ScrollSnap) = v` | Rest | Flow | `flow.scroll_snap` |
     | `ZIndex(i32)` | `z_index(v: i32) = v` | Delta | Flow | `flow.z_index` |
     | `GridTemplateColumns(Tracks)` | `grid_template_columns(v: impl Into<Tracks>) = v.into()` | Rest | Flow | `flow.grid.columns`（`grid` は `get_or_insert_with(Default::default)`） |
     | `GridTemplateRows(Tracks)` | `grid_template_rows(..)` 同 | Rest | Flow | `flow.grid.rows` |
     | `GridAutoColumns(Tracks)` | `grid_auto_columns(..)` 同 | Rest | Flow | `flow.grid.auto_columns` |
     | `GridAutoRows(Tracks)` | `grid_auto_rows(..)` 同 | Rest | Flow | `flow.grid.auto_rows` |
     | `GridAutoFlow(GridAutoFlow)` | `grid_auto_flow(v: GridAutoFlow) = v` | Rest | Flow | `flow.grid.flow` |
     | `GridColumn(Box<GridPlacement>)` | `grid_column(v: impl Into<GridPlacement>) = Box::new(v.into())` | Rest | Flow | `flow.grid.column` |
     | `GridRow(Box<GridPlacement>)` | `grid_row(..)` 同 | Rest | Flow | `flow.grid.row` |
     | `JustifyItems(Align)` | `justify_items(v: Align) = v` | Rest | Flow | `flow.grid.justify_items` |
     | `JustifySelf(Align)` | `justify_self(v: Align) = v` | Rest | Flow | `flow.grid.justify_self` |
     | `Background(Background)` | `background(v: impl Into<Background>) = v.into()` | Delta | Paint | `paint.background` |
     | `BorderTopWidth(f64)` | — | Rest | Paint | `paint.border_width[0]` |
     | `BorderRightWidth(f64)` | — | Rest | Paint | `paint.border_width[1]` |
     | `BorderBottomWidth(f64)` | — | Rest | Paint | `paint.border_width[2]` |
     | `BorderLeftWidth(f64)` | — | Rest | Paint | `paint.border_width[3]` |
     | `BorderTopColor(Color)` | — | Delta | Paint | `paint.border_color[0]` |
     | `BorderRightColor(Color)` | — | Delta | Paint | `paint.border_color[1]` |
     | `BorderBottomColor(Color)` | — | Delta | Paint | `paint.border_color[2]` |
     | `BorderLeftColor(Color)` | — | Delta | Paint | `paint.border_color[3]` |
     | `BorderTopStyle(BorderStyle)` | `border_top_style(v: BorderStyle) = v` | Delta | Paint | `paint.border_style[0]` |
     | `BorderRightStyle(BorderStyle)` | `border_right_style(..)` 同 | Delta | Paint | `paint.border_style[1]` |
     | `BorderBottomStyle(BorderStyle)` | `border_bottom_style(..)` 同 | Delta | Paint | `paint.border_style[2]` |
     | `BorderLeftStyle(BorderStyle)` | `border_left_style(..)` 同 | Delta | Paint | `paint.border_style[3]` |
     | `RadiusTopLeft(Box<CornerRadius>)` | `radius_top_left(v: impl Into<CornerRadius>) = Box::new(v.into())` | Delta | Paint | `paint.radius[0]`（`Some(**v)`） |
     | `RadiusTopRight(Box<CornerRadius>)` | `radius_top_right(..)` 同 | Delta | Paint | `paint.radius[1]` |
     | `RadiusBottomRight(Box<CornerRadius>)` | `radius_bottom_right(..)` 同 | Delta | Paint | `paint.radius[2]` |
     | `RadiusBottomLeft(Box<CornerRadius>)` | `radius_bottom_left(..)` 同 | Delta | Paint | `paint.radius[3]` |
     | `RingWidth(f64)` | — | Delta | Paint | `paint.ring_width` |
     | `RingOffset(f64)` | — | Delta | Paint | `paint.ring_offset` |
     | `RingColor(Color)` | — | Delta | Paint | `paint.ring_color` |
     | `Opacity(f64)` | `opacity(v: impl Into<f64>) = f64::clamp(v.into(), 0.0, 1.0)` | Delta | Paint | `paint.opacity` |
     | `Isolation(Isolation)` | `isolation(v: Isolation) = v` | Rest | Paint | `paint.isolation` |
     | `BorderImage(Box<BorderImage>)` | `border_image(v: BorderImage) = Box::new(v)` | Rest | Paint | `paint.border_image` |
     | `ScrollbarWidth(ScrollbarWidth)` | `scrollbar_width(v: impl Into<ScrollbarWidth>) = v.into()` | Delta | Paint | `paint.scrollbar_width` |
     | `ScrollbarThumbColor(Color)` | — | Delta | Paint | `paint.scrollbar_thumb` |
     | `ScrollbarTrackColor(Color)` | — | Delta | Paint | `paint.scrollbar_track` |
     | `Cursor(Cursor)` | `cursor(v: Cursor) = v` | Delta | Paint | `paint.cursor` |
     | `Clip(bool)` | `clip(v: bool) = v` | Rest | Paint | `paint.clip` |
     | `Filter(Box<Filter>)` | `filter(v: Filter) = Box::new(v)` | Delta | Paint | `paint.filter` |
     | `BackdropFilter(Box<Filter>)` | `backdrop_filter(v: Filter) = Box::new(v)` | Delta | Paint | `paint.backdrop_filter` |
     | `MixBlendMode(BlendMode)` | `mix_blend_mode(v: BlendMode) = v` | Delta | Paint | `paint.blend` |
     | `Mask(Box<Mask>)` | `mask(v: impl Into<Mask>) = Box::new(v.into())` | Delta | Paint | `paint.mask` |
     | `ClipPath(Box<Shape>)` | `clip_path(v: Shape) = Box::new(v)` | Delta | Paint | `paint.clip_path` |
     | `LocalPaint(LocalPaint)` | —（`paint(f)` が作る） | Rest | Paint | `paint.local_paint` |
     | `TranslateX(Length)` | `translate_x(v: impl Into<Length>) = v.into()` | Delta | Transform | `transform.translate[0]` |
     | `TranslateY(Length)` | `translate_y(..)` 同 | Delta | Transform | `transform.translate[1]` |
     | `TranslateZ(f64)` | `translate_z(v: impl Into<f64>) = v.into()` | Delta | Transform | `transform.translate_z` |
     | `Rotate(f64)` | `rotate(..)` 同 | Delta | Transform | `transform.rotate` |
     | `RotateX(f64)` | `rotate_x(..)` 同 | Delta | Transform | `transform.rotate_x` |
     | `RotateY(f64)` | `rotate_y(..)` 同 | Delta | Transform | `transform.rotate_y` |
     | `Scale((f64, f64))` | —（`scale`、`scale_xy` が作る） | Delta | Transform | `transform.scale` |
     | `Transform(Box<Transform>)` | `transform(v: Transform) = Box::new(v)` | Delta | Transform | `transform.transform` |
     | `TransformOrigin(Box<(Length, Length)>)` | —（`transform_origin(x, y)`） | Delta | Transform | `transform.transform_origin` |
     | `TransformOriginZ(f64)` | `transform_origin_z(v: impl Into<f64>) = v.into()` | Delta | Transform | `transform.transform_origin_z` |
     | `Perspective(f64)` | `perspective(..)` 同 | Delta | Transform | `transform.perspective` |
     | `PerspectiveOrigin(Box<(Length, Length)>)` | —（`perspective_origin(x, y)`） | Delta | Transform | `transform.perspective_origin` |
     | `TransformStyle(TransformStyle)` | `transform_style(v: TransformStyle) = v` | Delta | Transform | `transform.transform_style` |
     | `BackfaceVisibility(BackfaceVisibility)` | `backface_visibility(v: BackfaceVisibility) = v` | Delta | Transform | `transform.backface` |
     | `WillChange(WillChange)` | `will_change(v: WillChange) = v` | Rest | Transform | `transform.will_change` |
     | `FontSize(Length)` | `font_size(v: impl Into<Length>) = v.into()` | Rest | Text | `text.font_size` |
     | `FontWeight(u16)` | `weight(v: u16) = v` | Rest | Text | `text.weight` |
     | `Color(Color)` | `color(v: impl Into<Color>) = v.into()` | Delta | Text | `text.color` |
     | `LineHeight(LineHeight)` | `line_height(v: impl Into<LineHeight>) = v.into()` | Rest | Text | `text.line_height` |
     | `TextWrap(bool)` | `text_wrap(v: bool) = v` | Rest | Text | `text.text_wrap` |
     | `TextAlign(TextAlign)` | `text_align(v: TextAlign) = v` | Rest | Text | `text.text_align` |
     | `FontFamily(String)` | `font_family(v: impl Into<String>) = v.into()` | Rest | Text | `text.font_family` |
     | `Italic(bool)` | `italic(v: bool) = v` | Rest | Text | `text.italic` |
     | `LetterSpacing(Length)` | `letter_spacing(v: impl Into<Length>) = v.into()` | Rest | Text | `text.text.letter_spacing`（`text` は `get_or_insert_with(Default::default)`。以下 `x.` と書く） |
     | `WordSpacing(Length)` | `word_spacing(..)` 同 | Rest | Text | `x.word_spacing` |
     | `FontKerning(FontKerning)` | `font_kerning(v: FontKerning) = v` | Rest | Text | `x.kerning` |
     | `FontFeatureSettings(Vec<FontFeature>)` | —（`font_feature_settings(iter)`） | Rest | Text | `x.features` |
     | `TabularNums(bool)` | —（`tabular_nums()`） | Rest | Text | `x.tabular_nums` |
     | `TextDecorationLine(TextDecorationLine)` | `text_decoration(v: TextDecorationLine) = v` | Rest | Text | `x.decoration_line` |
     | `TextDecorationStyle(TextDecorationStyle)` | `text_decoration_style(v: TextDecorationStyle) = v` | Rest | Text | `x.decoration_style` |
     | `TextDecorationColor(Color)` | `text_decoration_color(v: impl Into<Color>) = v.into()` | Rest | Text | `x.decoration_color` |
     | `TextDecorationThickness(Length)` | `text_decoration_thickness(v: impl Into<Length>) = v.into()` | Rest | Text | `x.decoration_thickness` |
     | `TextUnderlineOffset(Length)` | `text_underline_offset(..)` 同 | Rest | Text | `x.underline_offset` |
     | `TextOverflow(TextOverflow)` | `text_overflow(v: TextOverflow) = v` | Rest | Text | `x.overflow` |
     | `LineClamp(u32)` | `line_clamp(v: u32) = v` | Rest | Text | `x.line_clamp` |
     | `TextCapsize(bool)` | `text_capsize(v: bool) = v` | Rest | Text | `x.capsize` |
     | `TextTransform(TextTransform)` | `text_transform(v: TextTransform) = v` | Rest | Text | `x.transform` |
     | `TextStrokeWidth(Length)` | — | Rest | Text | `x.stroke_width` |
     | `TextStrokeColor(Color)` | — | Rest | Text | `x.stroke_color` |
     | `PaintOrder(PaintOrder)` | `paint_order(v: PaintOrder) = v` | Rest | Text | `x.paint_order` |
     | `FontVariationSettings(Vec<FontVariation>)` | —（`font_variation_settings(iter)`） | Rest | Text | `x.variations` |
     | `FontOpticalSizing(bool)` | `font_optical_sizing(v: bool) = v` | Rest | Text | `x.optical_sizing` |
     | `FontWidth(f64)` | `font_width(v: impl Into<f64>) = v.into()` | Rest | Text | `x.font_width` |
     | `CaretColor(Color)` | `caret_color(v: impl Into<Color>) = v.into()` | Rest | Text | `x.caret_color` |
     | `SelectionColor(Color)` | `selection_color(..)` 同 | Rest | Text | `x.selection_color` |
     | `PlaceholderColor(Color)` | `placeholder_color(..)` 同 | Rest | Text | `x.placeholder_color` |
     | `EnterSize(Orientation)` | `enter_size(v: Orientation) = v` | Rest | Motion | `motion.enter_size` |
     | `ExitSize(Orientation)` | `exit_size(v: Orientation) = v` | Rest | Motion | `motion.exit_size` |
     | `LayoutTransition(Box<Transition>)` | `layout_transition(v: Transition) = Box::new(v)` | Rest | Motion | `motion.layout_transition` |

     `CornerRadius` は `Length` 二つ（S15 の後 96 byte）なので `Box` に入れる（`Decl` の大きさ 64 byte 以下を保つ）。文字の長さ（`LetterSpacing` など）は `Length` を受け、S18 が `text_length`（S15 手順 2b）で wire の `TextLength` にする。
     表の外の 5 つ（`sheet/decl.rs` に手で書く。手順 3）：`BoxShadow(Vec<Shadow>)`（Delta、Paint、`paint.shadows`）、`TextShadow(Vec<Shadow>)`（Rest、Text、`x.shadows`）、`Transition(TransitionProp, Box<Transition>)`（Rest、Motion、`motion.transitions` の同じ prop を置き換えて push）、`State(StyleState, Box<Style>)`（Rest、States）、`Group`（Rest、States、`states.group = true`）。

  2. `crates/kivact-style/src/sheet/props.rs` の macro と表：

```rust
//! Role: The one table of style properties: their declarations, setters, classes and groups.

use kivact_runtime::Transition;
use kivact_types::{
    Align, BackfaceVisibility, Cursor, Display, FlexDirection as Direction, Isolation, Justify,
    Length, Orientation, Overflow, Position, ScrollSnap, TextAlign, TransformStyle, TransitionProp,
    Visibility, WillChange,
};

use super::decl::{PropClass, PropGroup};
use super::Style;
use crate::color_value::Color;
use crate::draw::LocalPaint;
use crate::effect::{BlendMode, BorderStyle, Filter, Shadow, Shape};
use crate::geometry::CornerRadius;
use crate::grid::{GridAutoFlow, GridPlacement, Tracks};
use crate::paint::{Background, BorderImage, Mask};
use crate::text::{
    FontFeature, FontKerning, FontVariation, LineHeight, PaintOrder, TextDecorationLine,
    TextDecorationStyle, TextLength, TextOverflow, TextTransform,
};
use crate::transform::Transform;

/// Declares the property table: `Decl` (with the hand-written variants of
/// `sheet::decl`), one setter per row that has one, and each declaration's
/// name, class and group.
macro_rules! style_props {
    (
        setters { $( $(#[$doc:meta])* $setter:ident ( $v:ident : $arg:ty ) => $variant:ident ( $ty:ty ) = $value:expr , $class:ident , $group:ident ; )* }
        longhands { $( $lvariant:ident ( $lty:ty ) = $lname:literal , $lclass:ident , $lgroup:ident ; )* }
    ) => {
        /// One declaration of a style: a property and its value.
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) enum Decl {
            $( $variant($ty), )*
            $( $lvariant($lty), )*
            BoxShadow(Vec<Shadow>),
            TextShadow(Vec<Shadow>),
            Transition(TransitionProp, Box<Transition>),
            State(kivact_types::StyleState, Box<Style>),
            Group,
        }

        impl Decl {
            /// The name of the property (the setter's, or the longhand's).
            pub(crate) fn name(&self) -> &'static str {
                match self {
                    $( Decl::$variant(_) => stringify!($setter), )*
                    $( Decl::$lvariant(_) => $lname, )*
                    other => other.extra_name(),
                }
            }

            /// Whether the host can apply it as a state's paint delta.
            pub(crate) fn class(&self) -> PropClass {
                match self {
                    $( Decl::$variant(_) => PropClass::$class, )*
                    $( Decl::$lvariant(_) => PropClass::$lclass, )*
                    other => other.extra_class(),
                }
            }

            /// The group it belongs to.
            pub(crate) fn group(&self) -> PropGroup {
                match self {
                    $( Decl::$variant(_) => PropGroup::$group, )*
                    $( Decl::$lvariant(_) => PropGroup::$lgroup, )*
                    other => other.extra_group(),
                }
            }

            /// The `Debug` of its value (diagnostics, DevTools).
            pub(crate) fn value_debug(&self) -> String {
                match self {
                    $( Decl::$variant(v) => format!("{v:?}"), )*
                    $( Decl::$lvariant(v) => format!("{v:?}"), )*
                    other => other.extra_value_debug(),
                }
            }
        }

        impl Style {
            $(
                $(#[$doc])*
                pub fn $setter(self, $v: $arg) -> Self {
                    self.push(Decl::$variant($value))
                }
            )*
        }
    };
}

style_props! {
    setters {
        /// (the `///` doc of the 2b8df5ac setter `width`, copied as it is)
        width(v: impl Into<Length>) => Width(Length) = v.into(), Rest, Size;
        // ... one line per row of the table in step 1 that has a setter, in the table's order
    }
    longhands {
        ScrollAxes(ScrollAxes) = "scroll_axes", Rest, Flow;
        // ... one line per row of the table in step 1 whose setter is "—"
    }
}
```

     表の行を macro の行に書く規則：setter のある行は `setter(v: 引数) => Variant(型) = 値, Class, Group;`、setter の無い行は `Variant(型) = "名前", Class, Group;`（名前は variant の snake case：`BorderTopWidth` → `"border_top_width"`、`RingWidth` → `"ring_width"`、`ScrollbarThumbColor` → `"scrollbar_thumb_color"`、`TextStrokeWidth` → `"text_stroke_width"`、`FontFeatureSettings` → `"font_feature_settings"`、`TabularNums` → `"tabular_nums"`、`Scale` → `"scale"`、`TransformOrigin` → `"transform_origin"`、`PerspectiveOrigin` → `"perspective_origin"`、`FontVariationSettings` → `"font_variation_settings"`、`LocalPaint` → `"paint"`、`ScrollAxes` → `"scroll_axes"`）。setter の行の上には、2b8df5ac の同じ名前の setter の `///` doc を写す（`setters.rs`・`text_setters.rs`・`grid_setters.rs` にある）。`font_size` の doc には「A number is px; `font::SIZE` and the other font size variables of the theme are lengths too.」を足す。`text_decoration_style` の doc は「CSS `text-decoration-style`: the pattern of the decoration lines.」にする（ST-12）。
  3. `crates/kivact-style/src/sheet/decl.rs`（手で書く部分）：

```rust
//! Role: The style declaration list: how declarations are pushed, merged, folded and named.

use kivact_types::StyleState;

pub(crate) use super::props::Decl;

/// Whether a declaration can be a host-evaluated paint delta of a state
/// (`Delta`) or must be compiled into the node's style (`Rest`, ST-05).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PropClass {
    Delta,
    Rest,
}

/// The group a declaration belongs to (diagnostics order, text detection).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PropGroup {
    Size,
    Spacing,
    Flow,
    Paint,
    Transform,
    Text,
    States,
    Motion,
}

impl Decl {
    pub(super) fn extra_name(&self) -> &'static str {
        match self {
            Decl::BoxShadow(_) => "shadow",
            Decl::TextShadow(_) => "text_shadow",
            Decl::Transition(..) => "transition",
            Decl::State(state, _) => state.name(),
            Decl::Group => "group",
            _ => unreachable!("invariant: the table names every other declaration"),
        }
    }

    pub(super) fn extra_class(&self) -> PropClass {
        match self {
            Decl::BoxShadow(_) => PropClass::Delta,
            _ => PropClass::Rest,
        }
    }

    pub(super) fn extra_value_debug(&self) -> String {
        match self {
            Decl::BoxShadow(list) | Decl::TextShadow(list) => format!("{list:?}"),
            Decl::Transition(prop, t) => format!("{prop:?} {t:?}"),
            _ => String::new(),
        }
    }

    pub(super) fn extra_group(&self) -> PropGroup {
        match self {
            Decl::BoxShadow(_) => PropGroup::Paint,
            Decl::TextShadow(_) => PropGroup::Text,
            Decl::Transition(..) => PropGroup::Motion,
            _ => PropGroup::States,
        }
    }

    /// Whether setting it twice in one style is a duplicate (states nest,
    /// shadows stack, transitions are per property group, a group flag
    /// repeats harmlessly).
    fn counts_twice(&self) -> bool {
        !matches!(
            self,
            Decl::State(..) | Decl::BoxShadow(_) | Decl::TextShadow(_) | Decl::Transition(..) | Decl::Group
        )
    }
}

/// A style value: an ordered list of declarations, the later winning.
/// Build with [`style()`](crate::style).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    pub(crate) decls: Vec<Decl>,
}

const _: () = assert!(
    std::mem::size_of::<Decl>() <= 64,
    "a declaration stays small: large values are boxed"
);

impl Style {
    /// Append `decl` (constant time: a setter never scans the list, PF-01).
    pub(crate) fn push(mut self, decl: Decl) -> Self {
        self.decls.push(decl);
        self
    }

    /// Overlay `o` on `self`: its declarations come after, so they win.
    pub fn merge(mut self, o: &Style) -> Style {
        self.decls.extend(o.decls.iter().cloned());
        self
    }

    /// How many declarations set a property an earlier declaration of this
    /// style sets (debug diagnostics, counted only when asked: core asks
    /// once per style layer in debug builds).
    pub(crate) fn duplicates(&self) -> u32 {
        let mut seen = std::collections::HashSet::new();
        self.decls
            .iter()
            .filter(|d| d.counts_twice())
            .filter(|d| !seen.insert(std::mem::discriminant(*d)))
            .count() as u32
    }

    /// The style of `state`: every `state` style of this style merged in
    /// order; `None` when it sets none.
    pub(crate) fn state(&self, state: StyleState) -> Option<Style> {
        self.decls
            .iter()
            .filter_map(|d| match d {
                Decl::State(s, style) if *s == state => Some(&**style),
                _ => None,
            })
            .fold(None, |acc: Option<Style>, s| {
                Some(acc.unwrap_or_default().merge(s))
            })
    }

    /// This style without its `state` styles; the removed ones merged.
    pub(crate) fn take_state(&mut self, state: StyleState) -> Option<Style> {
        let taken = self.state(state);
        self.decls
            .retain(|d| !matches!(d, Decl::State(s, _) if *s == state));
        taken
    }

    /// Whether it sets a text property.
    pub(crate) fn has_text(&self) -> bool {
        self.decls.iter().any(|d| d.group() == crate::sheet::decl::PropGroup::Text)
    }

    /// The retained paint callback the style sets last (`Style::paint`).
    pub(crate) fn local_paint(&self) -> Option<crate::draw::LocalPaint> {
        self.decls.iter().rev().find_map(|d| match d {
            Decl::LocalPaint(p) => Some(p.clone()),
            _ => None,
        })
    }

    /// Remove every paint callback, in its state styles too.
    pub(crate) fn strip_local_paint(&mut self) {
        self.decls.retain(|d| !matches!(d, Decl::LocalPaint(_)));
        for d in &mut self.decls {
            if let Decl::State(_, s) = d {
                s.strip_local_paint();
            }
        }
    }
}
```

     `crate::sheet::decl::PropGroup` の path は `PropGroup` で書いてよい（同じファイル）。
  4. `crates/kivact-style/src/sheet/shorthands.rs`：表から生成しない setter を、2b8df5ac の doc を写して次の本体で書く（`impl Style { ... }`）。`Color` の引数は `impl Into<Color>`（S15）：
     - `padding(v)`：`let v = v.into(); self.padding_top(v).padding_right(v).padding_bottom(v).padding_left(v)`。`padding_x`：left と right。`padding_y`：top と bottom。`margin`・`margin_x`・`margin_y`・`inset`（`top`/`right`/`bottom`/`left`）も同じ形。
     - `gap(v)`、`single_line()`、`size(w, h)`、`translate(x, y)`、`place_items(a)`、`backdrop_blur(px)`：2b8df5ac の本体のまま（ほかの setter を呼ぶ）。
     - `scale(v: impl Into<f64>)`：`let v: f64 = v.into(); self.push(Decl::Scale((v, v)))`。`scale_xy(x, y)`：`self.push(Decl::Scale((x.into(), y.into())))`。
     - `transform_origin(x, y)`：`self.push(Decl::TransformOrigin(Box::new((x.into(), y.into()))))`。`perspective_origin(x, y)` も同じ形。
     - `border(width, color)`：`let (w, c) = (width.into(), color.into()); self.push(Decl::BorderTopWidth(w)).push(Decl::BorderRightWidth(w)).push(Decl::BorderBottomWidth(w)).push(Decl::BorderLeftWidth(w)).push(Decl::BorderTopColor(c)).push(Decl::BorderRightColor(c)).push(Decl::BorderBottomColor(c)).push(Decl::BorderLeftColor(c))`。`border_top(width, color)` などの 4 つ：その辺の width と color の 2 つ。`border_width(width)`：4 辺の width。`border_color(color)`：4 辺の color。`border_style(s)`：4 辺の style（`self.border_top_style(s).border_right_style(s)...`）。
     - `radius(r)`：`let r = r.into(); self.radius_top_left(r).radius_top_right(r).radius_bottom_right(r).radius_bottom_left(r)`。`radius_corners(tl, tr, br, bl)`：4 つの setter。
     - `ring(width, color)`：`self.ring_with_offset(width, 0, color)`。`ring_with_offset(width, offset, color)`：`self.push(Decl::RingWidth(width.into())).push(Decl::RingOffset(offset.into())).push(Decl::RingColor(color.into()))`。`ring_none()`：`self.ring(0, Color::TRANSPARENT)`。
     - `scrollbar_color(thumb, track)`：`self.push(Decl::ScrollbarThumbColor(thumb.into())).push(Decl::ScrollbarTrackColor(track.into()))`。
     - `shadow(s: Shadow)`：この style の最後の `Decl::BoxShadow` があればその `Vec` に push、無ければ `self.push(Decl::BoxShadow(vec![s]))`：
       ```rust
       pub fn shadow(mut self, s: Shadow) -> Self {
           if let Some(Decl::BoxShadow(list)) =
               self.decls.iter_mut().rev().find(|d| matches!(d, Decl::BoxShadow(_)))
           {
               list.push(s);
               return self;
           }
           self.push(Decl::BoxShadow(vec![s]))
       }
       ```
       `text_shadow(s)` も `Decl::TextShadow` で同じ形。
     - `text_stroke(width, color)`：`self.push(Decl::TextStrokeWidth(width.into())).push(Decl::TextStrokeColor(color.into()))`。
     - `font_feature_settings(features)`：`self.push(Decl::FontFeatureSettings(features.into_iter().map(Into::into).collect()))`。`font_variation_settings` も同じ形。`tabular_nums()`：`self.push(Decl::TabularNums(true))`。
     - `transition(prop, t)`：`self.push(Decl::Transition(prop, Box::new(t)))`。
     - 状態：`hover(s)` → `self.push(Decl::State(StyleState::Hover, Box::new(s)))`；`pressed` → `StyleState::Active`；`focus` → `Focus`；`focus_visible` → `FocusVisible`；`group_hover` → `GroupHover`；`group_focus` → `GroupFocus`；`disabled` → `Disabled`；`selected` → `Selected`；`enter` → `Enter`；`exit` → `Exit`。`group()` → `self.push(Decl::Group)`。
     - `paint(f)`：`self.push(Decl::LocalPaint(LocalPaint::new(f)))`。
     - free 関数 `pub fn with_scroll_axes(style: Style, axes: ScrollAxes) -> Style { style.push(Decl::ScrollAxes(axes)) }`。
     - `pub fn style() -> Style { Style::default() }`（`sheet/style.rs` から移す）。
     関数が 80 行を超えないように、状態の 11 個の setter は `shorthands.rs` の末尾にまとめ、ファイルが 1000 code 行を超えるなら状態の setter を `sheet/state_setters.rs` に分ける。
  5. **fold（旧い group の record に畳む）**：`crates/kivact-style/src/sheet/fields.rs` を次のように変える：
     - 各 `impl XFields { pub(super) fn merge(..) }`、`impl Sides { fn merge }`、`impl StateFields { fn merge }`、`impl MotionFields { fn merge }` を消す（`TextExtras::merge` も `text.rs` から消す）。
     - `pub struct Style { size: Option<Box<SizeFields>>, ... duplicates: u32 }`、`const _: () = assert!(size_of::<Style>() <= 72, ..)`、`fn group_eq`、`impl PartialEq for Style`、`impl Style { fn size_mut .. motion_mut }`、`kivact_runtime::constant_value!(Style);`、`mod eq_tests` を消す（`Style` は `decl.rs`。`constant_value!(Style)` は `decl.rs` の末尾に移す）。
     - `PaintFields` の `ring: Option<(f64, f64, Color)>` を `ring_width: Option<f64>, ring_offset: Option<f64>, ring_color: Option<Color>,` に、`scrollbar: Option<Box<ScrollbarStyle>>` を `scrollbar_width: Option<ScrollbarWidth>, scrollbar_thumb: Option<Color>, scrollbar_track: Option<Color>,` にし、`struct ScrollbarStyle` を消す。`TextExtras`（`text.rs`）の `stroke: Option<(TextLength, Color)>` を `stroke_width: Option<TextLength>, stroke_color: Option<Color>,` に、`decoration_style: Option<DecorationStyleValue>` を `decoration_style: Option<TextDecorationStyle>` にする。`TextFields.font_size` を `Option<Length>` に、`FlowFields.grid` を `Option<Box<GridStyle>>` のままにする。
     - 次の型と関数を足す：
       ```rust
       /// A style's declarations folded into one record, the later winning:
       /// what resolution reads.
       #[derive(Clone, Debug, Default)]
       pub(crate) struct Groups {
           pub(crate) size: SizeFields,
           pub(crate) spacing: SpacingFields,
           pub(crate) flow: FlowFields,
           pub(crate) paint: PaintFields,
           pub(crate) transform: TransformFields,
           pub(crate) text: TextFields,
           pub(crate) states: StateFields,
           pub(crate) motion: MotionFields,
       }
       ```
       と `impl Style { pub(crate) fn fold(&self) -> Groups { let mut g = Groups::default(); for d in &self.decls { g.apply(d); } g } }`、`impl Groups { fn apply(&mut self, d: &Decl) { match d { ... } } }`。`apply` の各腕は手順 1 の表の「畳む先」に `Some(値)`（`Copy` でない値は `.clone()`）を書く。例：`Decl::Width(v) => self.size.width = Some(*v),`、`Decl::PaddingTop(v) => self.spacing.padding.top = Some(*v),`、`Decl::GridColumn(v) => self.flow.grid.get_or_insert_with(Default::default).column = Some((**v).clone()),`、`Decl::LetterSpacing(v) => self.text.text.get_or_insert_with(Default::default).letter_spacing = Some(*v),`、`Decl::Transform(v) => self.transform.transform = Some((**v).clone()),`、`Decl::TransformOrigin(v) => self.transform.transform_origin = Some(**v),`。表の外の 5 つ：
       ```rust
       Decl::BoxShadow(list) => self.paint.shadows = Some(list.clone()),
       Decl::TextShadow(list) => self.text.text.get_or_insert_with(Default::default).shadows = Some(list.clone()),
       Decl::Transition(prop, t) => {
           self.motion.transitions.retain(|(p, _)| p != prop);
           self.motion.transitions.push((*prop, (**t).clone()));
       }
       Decl::Group => self.states.group = true,
       Decl::State(state, s) => self.states.merge_state(*state, s),
       ```
       `StateFields` に `fn merge_state(&mut self, state: StyleState, s: &Style)` を足す：`StyleState` に対応する field（`Hover`→`hover`、`Active`→`pressed`、`Focus`→`focus`、`FocusVisible`→`focus_visible`、`GroupHover`→`group_hover`、`GroupFocus`→`group_focus`、`Disabled`→`disabled`、`Selected`→`selected`、`Enter`→`enter`、`Exit`→`exit`）を `Some(Box::new(match slot.take() { Some(x) => (*x).merge(s), None => s.clone() }))` にする。`apply` が 80 行を超えるので、group ごとの関数（`fn apply_size`、`apply_spacing`、`apply_flow`、`apply_paint`、`apply_transform`、`apply_text`、`apply_rest`）に分け、`apply` は `match d.group()` で振り分ける（各関数の `match` の最後の腕は `_ => {}`）。
     - `crates/kivact-style/src/sheet/style.rs` の `group_or_default!` を次に置き換えて `sheet.rs` に置き、`style.rs` を消す（呼出しの形 `group_or_default!(s.size, SizeFields)` は変わらない。S18 で消す）：
       ```rust
       /// The group `$group` of `$style`, its declarations folded (S17's
       /// bridge; S18 folds once per compilation instead).
       macro_rules! group_or_default {
           ($style:ident . $group:ident, $ty:ident) => {
               std::borrow::Cow::<$ty>::Owned($style.fold().$group)
           };
       }
       pub(crate) use group_or_default;
       ```
     - `sheet.rs`：`mod fields; mod grid_setters; mod setters; mod style; mod text_setters;` を `mod decl; mod fields; mod props; mod shorthands;`（と分けた場合の `mod state_setters;`）にし、`pub use fields::{ScrollbarWidth, Style};` を `pub use decl::Style; pub use fields::ScrollbarWidth;`、`pub use setters::with_scroll_axes; pub use style::style;` を `pub use shorthands::{style, with_scroll_axes};` にする。`pub(crate) use fields::{...}` に `Groups` を足し、`Sides` 以下はそのまま、`pub(crate) use decl::{Decl, PropClass, PropGroup};` を足す。`git rm crates/kivact-style/src/sheet/setters.rs crates/kivact-style/src/sheet/text_setters.rs crates/kivact-style/src/sheet/grid_setters.rs crates/kivact-style/src/sheet/style.rs`。module の doc の段落のうち state の説明（「A state style changes paint only ...」）はこの unit では残す（S19 で直す）。
  6. **resolve を新しい storage に合わせる**（`group_or_default!` を通さない直接の読み書き）：
     - `crates/kivact-style/src/resolve/compile.rs`：
       - `compile_style` の `let retained_paint = s.paint.as_deref().and_then(|paint| paint.local_paint.clone());` と `strip_state_paint(&mut s);` の 2 行を消し、`fn strip_state_paint` を消す。代わりに、`s = merge_forced(s, state.forced);` の **後** に `let retained_paint = s.local_paint();` と `s.strip_local_paint();` を置く（12 章 UI-S-02：選ばれた `selected`・`disabled`・forced の状態の style が持つ paint の callback は、merge された後で選ばれる。状態の callback を先に消さない）。
       - `if state.selected && let Some(v) = s.states_mut().selected.take() { s = s.merge(&v); }` を `if state.selected && let Some(v) = s.take_state(StyleState::Selected) { s = s.merge(&v); }` に。disabled も同じ。
       - `fn merge_forced`：`s.states_mut().focus.take()` などの 4 か所を `s.take_state(StyleState::Focus)`、`FocusVisible`、`Hover`、`Active` にする。
       - `fn interactive_defaults` を次にする：
         ```rust
         fn interactive_defaults(s: &mut Style, theme: &Theme, disabled: bool) {
             if disabled {
                 s.take_state(StyleState::Hover);
                 s.take_state(StyleState::Active);
             }
             let has_focus = s.state(StyleState::Focus).is_some() || s.state(StyleState::FocusVisible).is_some();
             if !has_focus {
                 let ring = Style::default().ring_with_offset(theme.focus_ring(), theme.focus_ring_offset(), crate::color::FOCUS);
                 *s = std::mem::take(s).focus_visible(ring);
             }
             let sets_opacity = s
                 .state(StyleState::Disabled)
                 .is_some_and(|d| d.decls.iter().any(|x| matches!(x, Decl::Opacity(_))));
             if !sets_opacity {
                 *s = std::mem::take(s).disabled(Style::default().opacity(theme.disabled_opacity()));
             }
         }
         ```
         （disabled の state が複数の `State(Disabled, ..)` 宣言になっても `state()` が merge する。）
       - `s.has_text()` はそのまま（`decl.rs` の新しい実装）。
     - `crates/kivact-style/src/resolve/describe.rs` を次の内容に置き換える（`DELTA`、`classify_properties!`、`set_properties` 以下の表、`PADDING`・`MARGIN`・… の名前の配列、`split_sides`・`options_value`・`sides_value` を消す）：
       ```rust
       //! Role: Describe the properties a `Style` sets, for diagnostics.

       use crate::sheet::{Decl, PropClass, Style};

       /// The properties `s` sets (not its states), the last value of each,
       /// by name, with the `Debug` of its value, in the order first set.
       pub fn describe_style(s: &Style) -> Vec<(&'static str, String)> {
           let mut out: Vec<(&'static str, String)> = Vec::new();
           for d in &s.decls {
               if matches!(d, Decl::State(..) | Decl::Group) {
                   continue;
               }
               let value = d.value_debug();
               match out.iter_mut().find(|(name, _)| *name == d.name()) {
                   Some(slot) => slot.1 = value,
                   None => out.push((d.name(), value)),
               }
           }
           out
       }

       /// The properties of each state style `s` sets, by state name.
       pub fn describe_state_styles(s: &Style) -> Vec<(&'static str, Vec<(&'static str, String)>)> {
           kivact_types::StyleState::ALL
               .into_iter()
               .filter_map(|state| s.state(state).map(|st| (state.name(), describe_style(&st))))
               .collect()
       }

       /// The properties set in the host states of `s` that a state cannot
       /// change as paint (`CompiledStyle::unsupported_in_states`, until ST-05).
       pub(super) fn unsupported_in_states(s: &Style) -> Vec<String> {
           let mut out = Vec::new();
           for state in kivact_types::StyleState::HOST {
               if let Some(st) = s.state(state) {
                   out.extend(
                       st.decls
                           .iter()
                           .filter(|d| d.class() == PropClass::Rest)
                           .map(|d| format!("{}.{}", state.name(), d.name())),
                   );
               }
           }
           out
       }
       ```
       値の文字列は `Decl::value_debug`（macro が生成する。variant の中身の `Debug`：`Px(8.0)` の形で、2b8df5ac の describe の値と同じ）を使う。
     - `crates/kivact-style/src/resolve/paint.rs`：`paint.ring.map(|(width, offset, c)| Ring { width, offset, color: c.resolve(theme) })` を `paint.ring_width.map(|width| Ring { width, offset: paint.ring_offset.unwrap_or(0.0), color: paint.ring_color.unwrap_or(crate::color::FOCUS).resolve(theme) })` に。`fn scrollbar` の `let style = paint.scrollbar.as_ref()?;` 以下を `paint.scrollbar_width`・`scrollbar_thumb`・`scrollbar_track` を読む形にする（三つとも `None` なら `None` を返す。thumb と track は片方だけなら他方を `ScrollbarProps::default()` の値にする）。`paint_delta` の `ring: st_paint.ring.and(full.ring)` を `ring: (st_paint.ring_width.is_some() || st_paint.ring_offset.is_some() || st_paint.ring_color.is_some()).then_some(full.ring).flatten()` にする。`scrollbar` の delta も同じ規則（三つのどれかが設定されていれば `ScrollbarDelta { width: style.scrollbar_width.map(|_| full.width), colors: (thumb か track が Some).then(|| (full.thumb, full.track)) }`）。
     - `crates/kivact-style/src/text.rs` の `fn resolve_extras`：`t.stroke = x.stroke.map(|(width, c)| ...)` を `t.stroke = x.stroke_width.map(|width| p::TextStroke { width, color: x.stroke_color.unwrap_or(crate::color::TEXT).resolve(theme) });` に、`t.decoration_style = x.decoration_style.as_ref().map(DecorationStyleValue::get);` を `t.decoration_style = x.decoration_style;` にし、`enum DecorationStyleValue` とその 4 つの impl を消す。
     - `crates/kivact-style/src/resolve/compile.rs` の `fn text` の `font_size: text.font_size,` を `font_size: text.font_size.and_then(|l| match resolved(l, theme, theme.get(&crate::font::SIZE_VAR), misplaced) { Length::Px(px) => Some(px), _ => { *misplaced += 1; None } }),` にする（`em` は theme の基準の大きさ `font::SIZE` の倍。百分率と内容の大きさは数えて報告する。`fn text` に引数 `misplaced: &mut u32` を足し、`compile_style` の `misplaced` を渡す）。
  7. **掃引**：`text_decoration_style(` の呼出しで closure や `Signal` を渡している所（`rg -n "text_decoration_style\(" $W6_SCOPE $W6_TESTS`）は、`.style(move || style().text_decoration_style(sig.get()))` の形にする（reactive な style は closure。設計書 5.7 の 5）。`font_size(` に `f32` 以外の数値の式を渡す所は変更なし（`impl Into<Length>`）。`font_size(x)` の `x` が `Option<f64>` などで compile error になれば、その場で `Length::Px(..)` にする。
  8. **DevTools の property の表**（付属書 C の S-REQ-2。C19 がこの unit の直後に使う。INC-03）：
     - `crates/kivact-style/src/sheet/edit_text.rs` を作る（`sheet.rs` に `mod edit_text;` と `pub(crate) use edit_text::{EditText, STYLE_PROPERTIES, StyleProperty};`）：

```rust
//! Role: The DevTools text form of each style property: how its value is shown and parsed.

use kivact_types::{Display, Length, TextAlign, ThemeVar};

use super::Style;
use crate::color_value::Color;

/// One style property as DevTools edits it: its name, its value in the
/// style as override text (`12`, `50%`, `auto`, `#2563eb`, a theme variable
/// name), and the style that text sets.
pub struct StyleProperty {
    pub name: &'static str,
    pub describe: fn(&Style) -> Option<String>,
    pub parse: fn(&str) -> Result<Style, String>,
}

/// A value DevTools writes and reads as text.
pub(crate) trait EditText: Sized {
    fn to_edit_text(&self) -> String;
    fn from_edit_text(text: &str) -> Result<Self, String>;
}

const COLOR_VARS: [&ThemeVar<Color>; 17] = [
    &crate::color::SURFACE_VAR, &crate::color::SURFACE_ALT_VAR, &crate::color::TEXT_VAR,
    &crate::color::TEXT_MUTED_VAR, &crate::color::ACCENT_VAR, &crate::color::ACCENT_TEXT_VAR,
    &crate::color::BORDER_VAR, &crate::color::FOCUS_VAR, &crate::color::DANGER_VAR,
    &crate::color::SUCCESS_VAR, &crate::color::WARNING_VAR, &crate::color::OVERLAY_VAR,
    &crate::color::SHADOW_VAR, &crate::color::SELECTION_VAR, &crate::color::HOVER_VAR,
    &crate::color::PRESSED_VAR, &crate::color::RAISED_VAR,
];

const LENGTH_VARS: [&ThemeVar<f64>; 11] = [
    &crate::space::XS_VAR, &crate::space::SM_VAR, &crate::space::MD_VAR, &crate::space::LG_VAR,
    &crate::space::XL_VAR, &crate::radius::SM_VAR, &crate::radius::MD_VAR, &crate::radius::LG_VAR,
    &crate::font::SIZE_VAR, &crate::font::SIZE_SMALL_VAR, &crate::font::SIZE_LARGE_VAR,
];

impl EditText for Length {
    fn to_edit_text(&self) -> String {
        match self {
            Length::Px(v) => format!("{v}"),
            Length::Percent(v) => format!("{v}%"),
            Length::Auto => "auto".to_owned(),
            Length::MinContent => "min-content".to_owned(),
            Length::MaxContent => "max-content".to_owned(),
            Length::FitContent => "fit-content".to_owned(),
            Length::Em(v) => format!("{v}em"),
            Length::Calc { percent, px, em, var } => {
                let var = var.map_or(String::new(), |(v, k)| format!(" + {k} * {}", v.name()));
                format!("calc({percent}% + {px}px + {em}em{var})")
            }
            Length::Var(v) => v.name().to_owned(),
            Length::Invalid => "invalid".to_owned(),
        }
    }
    fn from_edit_text(text: &str) -> Result<Self, String> {
        let text = text.trim();
        match text {
            "auto" => return Ok(Length::Auto),
            "min-content" => return Ok(Length::MinContent),
            "max-content" => return Ok(Length::MaxContent),
            "fit-content" => return Ok(Length::FitContent),
            _ => {}
        }
        if let Some(var) = LENGTH_VARS.into_iter().find(|v| v.name() == text) {
            return Ok(Length::var(var));
        }
        if let Some(em) = text.strip_suffix("em") {
            return em.trim().parse::<f64>().map(Length::Em).map_err(|_| expected("a length", text));
        }
        if let Some(percent) = text.strip_suffix('%') {
            return percent.trim().parse::<f64>().map(Length::Percent).map_err(|_| expected("a length", text));
        }
        text.trim_end_matches("px").parse::<f64>().map(Length::Px).map_err(|_| expected("a length", text))
    }
}

impl EditText for Color {
    fn to_edit_text(&self) -> String {
        match self.var_target() {
            Some((var, a)) if a == 1.0 => var.name().to_owned(),
            _ => {
                let [r, g, b, a] = self.components(&crate::Theme::light());
                let c = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                if a < 1.0 {
                    format!("#{:02x}{:02x}{:02x}{:02x}", c(r), c(g), c(b), c(a))
                } else {
                    format!("#{:02x}{:02x}{:02x}", c(r), c(g), c(b))
                }
            }
        }
    }
    fn from_edit_text(text: &str) -> Result<Self, String> {
        let text = text.trim();
        if let Some(var) = COLOR_VARS.into_iter().find(|v| v.name() == text) {
            return Ok(Color::var(var));
        }
        Color::css(text).map_err(|_| expected("a color", text))
    }
}

impl EditText for Display {
    fn to_edit_text(&self) -> String {
        match self { Display::Flex => "flex", Display::Grid => "grid", Display::Contents => "contents" }.to_owned()
    }
    fn from_edit_text(text: &str) -> Result<Self, String> {
        match text.trim() {
            "flex" => Ok(Display::Flex),
            "grid" => Ok(Display::Grid),
            "contents" => Ok(Display::Contents),
            other => Err(expected("flex, grid or contents", other)),
        }
    }
}

impl EditText for TextAlign {
    fn to_edit_text(&self) -> String {
        match self {
            TextAlign::Start => "start",
            TextAlign::Center => "center",
            TextAlign::End => "end",
            TextAlign::Justify => "justify",
        }
        .to_owned()
    }
    fn from_edit_text(text: &str) -> Result<Self, String> {
        match text.trim() {
            "start" => Ok(TextAlign::Start),
            "center" => Ok(TextAlign::Center),
            "end" => Ok(TextAlign::End),
            "justify" => Ok(TextAlign::Justify),
            other => Err(expected("start, center, end or justify", other)),
        }
    }
}

/// Numbers and flags: written as Rust writes them, parsed by `str::parse`.
macro_rules! number_text {
    ($($t:ty),*) => {$(
        impl EditText for $t {
            fn to_edit_text(&self) -> String { format!("{self}") }
            fn from_edit_text(text: &str) -> Result<Self, String> {
                text.trim().parse::<$t>().map_err(|_| expected(stringify!($t), text))
            }
        }
    )*};
}
number_text!(f64, u16, u32, i32, bool);

/// Values DevTools shows (their `Debug`) but does not parse.
macro_rules! shown_only {
    ($($t:ty),*) => {$(
        impl EditText for $t {
            fn to_edit_text(&self) -> String { format!("{self:?}") }
            fn from_edit_text(text: &str) -> Result<Self, String> {
                Err(format!("{} is not editable as text: {text}", stringify!($t)))
            }
        }
    )*};
}

impl<T: EditText> EditText for Box<T> {
    fn to_edit_text(&self) -> String { (**self).to_edit_text() }
    fn from_edit_text(text: &str) -> Result<Self, String> { T::from_edit_text(text).map(Box::new) }
}

fn expected(what: &str, text: &str) -> String {
    format!("expected {what}, got {text}")
}
```

       `shown_only!` を、手順 1 の表の値の型のうち上で実装しなかったものすべてに当てる（`shown_only!(Align, Justify, kivact_types::FlexDirection, Position, Overflow, Visibility, ScrollSnap, ScrollAxes, Orientation, Tracks, GridAutoFlow, GridPlacement, Background, BorderStyle, CornerRadius, Isolation, BorderImage, ScrollbarWidth, Cursor, Filter, BlendMode, Mask, Shape, LocalPaint, (f64, f64), (Length, Length), Transform, TransformStyle, BackfaceVisibility, WillChange, LineHeight, String, FontKerning, Vec<FontFeature>, Vec<FontVariation>, TextDecorationLine, TextDecorationStyle, TextLength, TextOverflow, TextTransform, PaintOrder, Transition);`。`String` は `shown_only` でなく `impl EditText for String { to: self.clone(), from: Ok(text.trim().to_owned()) }` にする（font family の override）。compile が「`EditText` が実装されていない」と言う型は、この規則で `shown_only!` に足す）。`Debug` を持たない型（`LocalPaint` など）は `to_edit_text` が `"<callback>"` を返す impl を手で書く。
     - `crates/kivact-style/src/sheet/props.rs` の macro の展開に表を足す（`impl Style { .. }` の後）：
       ```rust
       /// Every property of the table in DevTools form (`internal::style_property`).
       pub(crate) static STYLE_PROPERTIES: &[StyleProperty] = &[
           $( StyleProperty {
               name: stringify!($setter),
               describe: |s: &Style| s.decls.iter().rev().find_map(|d| match d {
                   Decl::$variant(v) => Some(EditText::to_edit_text(v)),
                   _ => None,
               }),
               parse: |text: &str| <$ty as EditText>::from_edit_text(text)
                   .map(|v| Style::default().push(Decl::$variant(v))),
           }, )*
           $( StyleProperty {
               name: $lname,
               describe: |s: &Style| s.decls.iter().rev().find_map(|d| match d {
                   Decl::$lvariant(v) => Some(EditText::to_edit_text(v)),
                   _ => None,
               }),
               parse: |text: &str| <$lty as EditText>::from_edit_text(text)
                   .map(|v| Style::default().push(Decl::$lvariant(v))),
           }, )*
       ];
       ```
       （`props.rs` の `use` に `super::edit_text::{EditText, StyleProperty};` を足す。）
     - `crates/kivact-style/src/internal.rs` に足す：
       ```rust
       pub use crate::sheet::StyleProperty;

       /// The property `name` (a setter's or a longhand's name) in DevTools form.
       pub fn style_property(name: &str) -> Option<&'static StyleProperty> {
           crate::sheet::STYLE_PROPERTIES.iter().find(|p| p.name == name)
       }
       ```
       （`sheet.rs` の `pub(crate) use edit_text::{..}` の `StyleProperty` は `pub use` にする：`internal` が再公開するため。）
     - 試験（`crates/kivact-style/src/sheet/decl.rs` の `mod tests`）：`fn style_properties_round_trip()`：`(style_property("width").unwrap().parse)("50%")` の style の `describe` が `Some("50%")`、`"space.md"` は `Some("space.md")`、`(style_property("color").unwrap().parse)("#2563eb")` の describe が `Some("#2563eb")`、`style_property("nope")` が `None`。
- **消すもの**：`sheet/setters.rs`、`sheet/text_setters.rs`、`sheet/grid_setters.rs`、`sheet/style.rs`、旧い `Style` の group の field と `*_mut`、`group_eq`、各 group の `merge`、`ScrollbarStyle`、`DecorationStyleValue`、`describe.rs` の `DELTA`・`classify_properties!`・名前の配列、`strip_state_paint`。
- **状態の閉じ方**：`Style` は `Vec<Decl>` の field 一つ（6.1 の対象外）。重複は field に持たず、`Style::duplicates()` が求める（PF-01）。`Groups` は値の record（`&mut self` の `apply` を持つ。`Option` の field は各 group の中）：印 `// xtask-allow(state): fold of one style's declarations: apply writes each authored property once per declaration and resolution only reads the result`（`pub(crate) struct Groups` の直前）。各 `XFields` は W1b で付属書 T が印を処理した状態のまま（6.1 の新しい規則では、`&mut self` の method を持たない値の record は対象外。この unit で `merge` を消すので対象外になる）。
- **試験**：
  - `crates/kivact-style/src/sheet/decl.rs` に `#[cfg(test)] mod tests`（`//! Role: Tests of the declaration list.`）：`fn the_later_declaration_wins()`（`style().width(10).width(20).fold().size.width == Some(Length::Px(20.0))`）、`fn merge_appends_and_the_overlay_wins()`（`style().width(10).merge(&style().width(30))` の fold が 30、`.duplicates()` が 1（同じ style の中の二つ目。core は層ごとに数えるので層の上書きは数えない））、`fn a_repeated_setter_counts_a_duplicate()`（`style().width(1).width(2).duplicates() == 1`、`style().width(1).height(2).duplicates() == 0`、`style().hover(style()).hover(style()).duplicates() == 0`）、`fn shadows_stack_in_one_style_and_a_later_layer_replaces_them()`、`fn nested_state_styles_merge()`（`style().hover(style().width(1)).hover(style().height(2))` の `state(StyleState::Hover)` が width と height の両方を持つ）。
  - 公開 E2E の style の試験（`style_paint.rs`、`text_style.rs`、`layout_*.rs`、`hover.rs`、`motion.rs`、`devtools.rs` の style の表示）がそのまま通ること。
- **確認（tier a）**：鎖の共通確認と次：
  ```
  cargo test -p kivact-style
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native -- devtools
  ```
- **完了条件**：`ls crates/kivact-style/src/sheet` が `decl.rs edit_text.rs fields.rs props.rs shorthands.rs`（と `state_setters.rs`）だけ。`rg -n "DecorationStyleValue|ScrollbarStyle|classify_properties|DELTA" crates/kivact-style/src` が 0 件。上のコマンドが通る。

---

## S17b：DevTools の text の追補（2026-11-21 の設計者の回答、C19 の差戻し）

- **wave / lane / 前提**：W6b / 鎖（lane X）/ C19 の commit `79358493` の上。この unit の後に C19 の確認をやり直し、S18 へ進む。
- **所有**：`crates/kivact-style/src/sheet/edit_text.rs`、`crates/kivact-style/src/sheet/props.rs`（macro の `value_debug` だけ）、`crates/kivact-style/src/sheet/decl.rs`（`extra_value_debug` と `mod tests`）、`crates/kivact-style/src/resolve/describe.rs`（`describe_style` だけ）、`crates/kivact-core/src/devtools/styles.rs`・`apply.rs`（下の 4 の掃除だけ）
- **目的**：(1) DevTools の override で `background`・`line_height`・`radius` を text から読めるように保つ（2b8df5ac の `override_paint`・`override_text` が読めた形）。(2) DevTools の style の一覧（base の層も state の層も）の値を、`Debug` の文字列でなく表の text の形にする。
- **手順**：
  1. `edit_text.rs`：`shown_only!` の一覧から `Background`、`CornerRadius`、`LineHeight` を外し、次の 3 つの impl を足す（`Length`・`Color` の impl の後）：
     - `impl EditText for Background`：`to_edit_text` は `self.solid()` が `Some(c)` なら `c.to_edit_text()`、`None`（gradient・複数の層）なら `format!("{self:?}")`。`from_edit_text` は `Color::from_edit_text(text).map(|c| Background::from(Paint::from(c)))`（gradient は text から作らない。既存の `From<C: Into<Color>> for Paint` と `From<Paint> for Background` を使う）。
     - `impl EditText for LineHeight`：`to_edit_text` は `Normal` → `"normal"`、`Scale(v)` → `format!("{v}")`、`Px(v)` → `format!("{v}px")`。`from_edit_text` は `text.trim()` が `"normal"` なら `Normal`、末尾が `px` ならその前を `f64` で読んで `Px`、それ以外は `f64` で読んで `Scale`。読めなければ `expected("a line height", text)`。
     - `impl EditText for CornerRadius`：`to_edit_text` は `x == y` なら `x.to_edit_text()`、違えば `format!("{} {}", x.to_edit_text(), y.to_edit_text())`。`from_edit_text` は空白で分けて、1 つなら `Length::from_edit_text` を `x` と `y` の両方に、2 つなら順に `x`・`y`、それ以外は `expected("one or two lengths", text)`。
     （`Box<CornerRadius>` は既存の `impl<T: EditText> EditText for Box<T>` で済む。）
  2. `props.rs` の macro：`pub(crate) fn value_debug(&self) -> String` を `pub(crate) fn edit_text(&self) -> String` に改名し、doc を「Its value in the DevTools text form (`EditText`).」にする。本体の 2 行を `$( Decl::$variant(v) => EditText::to_edit_text(v), )*` と `$( Decl::$lvariant(v) => EditText::to_edit_text(v), )*` にし、`other => other.extra_edit_text(),` にする。`decl.rs` の `extra_value_debug` を `extra_edit_text` に改名する（本体は変えない：shadow と transition は `Debug` のまま）。
  3. `describe.rs` の `describe_style`：`d.value_debug()` を `d.edit_text()` にし、doc の「with the `Debug` of its value」を「with its value in the DevTools text form」にする。`describe_state_styles` は変えない（state の層も同じ形になる）。
  4. core：C19 の `styles.rs`・`apply.rs` に、`Debug` の文字列を前提にした処理（`Px(..)` の形を読み直す、など）があれば消し、`describe_style`・`describe_state_styles` の値をそのまま使う。無ければ変えない。
- **状態の閉じ方**：新しい状態は無い。
- **試験**：`decl.rs` の `fn style_properties_round_trip()` に足す：`background` に `"#2563eb"` を parse した style の describe が `Some("#2563eb")`、`line_height` の `"1.5"` が `Some("1.5")`、`"20px"` が `Some("20px")`、`"normal"` が `Some("normal")`、`radius_top_left` の `"8"` が `Some("8")`、`"8 4"` が `Some("8 4")`、`radius_top_left` の `"a b c"` が `Err`。`describe_style` の値の文字列を期待する既存の試験（kivact-style の unit 試験、公開 E2E の devtools の style の表示）があれば、期待の値を text の形（`Px(8.0)` → `8` など）に直してよい（設計者の決まり。0.5 の差戻しに当たらない）。
- **確認（tier a）**：鎖の共通確認と次：
  ```
  cargo test -p kivact-style
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native -- devtools
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core
  ```
- **完了条件**：`rg -n "value_debug" crates/kivact-style/src` が 0 件。`rg -n "Background, |CornerRadius, |LineHeight," crates/kivact-style/src/sheet/edit_text.rs` の `shown_only!` の行に 3 つが無い。上のコマンドが通る。

---

## S18：解決を一度の畳み込みで行う（ST-08 の後半、ST-06、ST-31、ST-09 の style 側）

- **wave / lane / 前提**：W6b / 鎖 / C19（付属書 C。鎖の中で S17 の直後）
- **所有**：`crates/kivact-style/src/resolve/`、`crates/kivact-style/src/sheet.rs`、`crates/kivact-style/src/sheet/fields.rs`、`crates/kivact-style/src/geometry.rs`、`crates/kivact-style/src/text.rs`、`crates/kivact-style/src/paint/`（`to_len` の呼出し）、`crates/kivact-style/src/internal.rs`、`crates/kivact-core/src/view/mount/style.rs`、`crates/kivact-core/src/view/mount/facts.rs`、`crates/kivact-core/src/view/mount/draw.rs`、`crates/kivact-core/src/view/devtools_state.rs`、`crates/kivact-core/src/view/devtools_hooks.rs`、`crates/kivact-core/src/devtools/convert.rs`、`crates/kivact-core/src/devtools/styles.rs`、`crates/kivact-core/src/devtools/node.rs`
- **目的**：ST-08 の後半：`group_or_default!` の毎回の畳み込みをやめ、`compile_style` が宣言を **一度だけ** 畳んで、その record から layout・paint・text・state を解決する。ST-06：style の複製と再計算を減らす（`compile_style` が `Style` を値で受ける、theme の既定を `Style` を作らずに足す、state の delta は base の record を一度だけ作って使う、core が `CompiledStyle` の部品を move する、`LocalPaintSource` が `CompiledStyle` 全体でなく callback だけを持つ、`merge_layers` が最初の層から始める）。ST-31：塗りの長さの `auto`・内容の大きさ・変数でない非 px 値を数えて報告する。ST-09（style 側）：`ForcedStates` を `kivact_types::StateSet` にする。
- **手順**：
  1. `crates/kivact-style/src/sheet.rs` の `group_or_default!` を消す。resolve の中の `group_or_default!(s.X, XFields)` をすべて、関数の引数の `g: &Groups` の `&g.X` にする：
     - `crates/kivact-style/src/resolve/compile.rs`：`fn layout(s: &Style, theme)` → `fn layout(g: &Groups, theme: &Theme) -> LayoutProps`、`fn misplaced_content(s)` → `fn misplaced(g: &Groups) -> u32`（手順 4）、`fn text(s, theme)` → `fn text(g: &Groups, theme: &Theme) -> (TextStyle, u32)`、`fn state_styles(s, theme)` → `fn state_styles(s: &Style, g: &Groups, theme: &Theme)`。
     - `crates/kivact-style/src/resolve/paint.rs`：`border_widths`、`border`、`corners`、`effects`、`transform_of`、`element_transform`、`sets_transform`、`perspective`、`paint`、`paint_transitions`、`reports`、`scrollbar` の第 1 引数 `s: &Style` を `g: &Groups` にし、本体の `let paint = group_or_default!(s.paint, PaintFields);` などを `let paint = &g.paint;` にする。
  2. `pub fn compile_style(s: &Style, theme: &Theme, state: Interactive) -> CompiledStyle` を `pub fn compile_style(mut s: Style, theme: &Theme, mut state: Interactive) -> CompiledStyle` にし（ST-06：呼出し側の merge の結果を move で受け、`s.clone()` をしない）、本体を次の順にする：
     ```rust
     state.forced 由来の disabled / selected（今のまま）
     if state.focusable { interactive_defaults(&mut s, theme, state.disabled); }
     selected / disabled の state を take_state して merge（今のまま）
     s = merge_forced(s, state.forced);
     let retained_paint = s.local_paint();   // after the merged states (UI-S-02)
     s.strip_local_paint();
     let g = s.fold();                       // the one fold of this compilation
     let states = state_styles(&s, &g, theme);
     let mut misplaced = misplaced(&g);
     let mut unsupported_text = 0;
     let text = (s.has_text() || state.root).then(|| { let (mut t, dropped) = text(&g, theme); unsupported_text = dropped; root の既定（今のまま）; t });
     let mut paint = paint(&g, theme, &mut misplaced);
     paint.transitions = paint_transitions(&g);
     CompiledStyle { retained_paint, layout: layout(&g, theme), paint, text, states, misplaced_lengths: misplaced, unsupported_text, unsupported_in_states: unsupported_in_states(&s), reports: reports(&g, theme) }
     ```
     `CompiledStyle` の field `duplicates` を消す（PF-01：重複は core が層ごとに数える。手順 6）。`CompiledStyle` の field `misplaced_content` を `misplaced_lengths` に改名する（doc：「Lengths a property cannot take (a content size outside width, height and basis; `auto`, a content size or a non-px value in a paint position or radius), counted and reported as `Unsupported`.」）。
  3. state の delta（ST-06）：`fn paint_delta(base: &Style, st: &Style, theme: &Theme) -> PaintDelta` を `fn paint_delta(base: &Style, st: &Style, theme: &Theme) -> PaintDelta` のまま残すが、本体の `let merged = base.clone().merge(st); let full = paint(&merged, theme);` の `paint(..)` を `paint(&merged.fold(), theme, &mut 0)` にし、`st` の group を読む所は `let sg = st.fold();` を一度だけ作って `&sg.paint` などを読む。`state_styles` の focus_visible の `s.clone().merge(focus)` はそのまま（state を持つ style だけの費用）。`fn interactive_defaults` は S17 の形のまま（`Style` を作るのは focusable な node で focus の look が無いときだけ）。
  4. ST-31：`crates/kivact-types/src/length.rs` の `pub fn to_len(self) -> Len` を `pub fn to_len(self, misplaced: &mut u32) -> Len` にする：`Px`・`Percent`・`Calc { percent, px, .. }` は写し、`Auto`・`MinContent`・`MaxContent`・`FitContent`・`Var`・`Em`・`Invalid` は `*misplaced += 1;` して `Len::ZERO`（`Var`・`Em`・`Invalid` は呼出し側が先に `resolved(..)` するので、ここに来るのは解決し忘れ）。文字の長さ（`TextFields`・`TextExtras` の `Length` の field）は `crate::text::text_length(l, theme, &mut dropped)`（S15）で wire の `TextLength` にする。style の `to_len()` の呼出し（`rg -n "to_len\(" crates/kivact-style/src`：`geometry.rs` の `corner`、`resolve/paint.rs` の `element_transform`・`perspective`、`paint/*.rs` の gradient の中心・stop、背景の大きさと位置、mask、clip path の点、border image）に `&mut misplaced` を渡す。`misplaced` は `paint(g, theme, misplaced: &mut u32)` の引数で受け、その中で呼ぶ関数（`fill`、`fill_layer`、`corners`、`effects` の `clip_shape`、mask の `wire`、border image の `wire`）に `&mut u32` の引数を足して渡す。`Draw`（`draw/*.rs`）が `to_len` を呼ぶ所は `&mut self.unsupported` を渡す（canvas の命令の未対応の数え方と同じ）。`fn misplaced(g: &Groups) -> u32` は旧 `misplaced_content` の本体（内容の大きさを最小・最大・padding・margin・inset・gap に置いた数）。
     `crates/kivact-core/src/view/mount/style.rs` の報告の文（`"{} content size(s) (Length::MinContent, ...) outside width, height and basis; they are auto there"`）を `"{} length(s) the property cannot take: a content size outside width, height and basis, or auto, a content size or a variable left unresolved in a paint position or radius; they are 0 or auto there"` にし、`resolved.misplaced_content` を `resolved.misplaced_lengths` にする。
  5. ST-09（style・core）：`crates/kivact-style/src/resolve/compile.rs` の `pub struct ForcedStates(u8)` とその impl を消し、`Interactive.forced` の型を `kivact_types::StateSet` にする。`state.forced.contains(ForcedStates::DISABLED)` を `state.forced.contains(StyleState::Disabled)`、`SELECTED` → `Selected`、`FOCUS` → `Focus`、`FOCUS_VISIBLE` → `FocusVisible`、`HOVER` → `Hover`、`PRESSED` → `Active` にする。`crates/kivact-style/src/resolve.rs` と `internal.rs` の再公開から `ForcedStates` を消す。core：
     - `crates/kivact-core/src/view/devtools_state.rs`、`crates/kivact-core/src/view/devtools_hooks.rs`：`ForcedStates` を `kivact_types::StateSet` に（`use kivact_style::internal::ForcedStates;` を `use kivact_types::StateSet;` に、型名を置換）。`ForcedStates::default()` は `StateSet::EMPTY`。
     - `crates/kivact-core/src/devtools/convert.rs` の `p::InteractiveState::Hovered => ForcedStates::HOVER,` などの 6 行を `StyleState::Hover`、`Active`、`Focus`、`FocusVisible`、`Disabled`、`Selected` にし、関数の戻り値の型を `StyleState` にする。呼出し側で `states.insert(x)` していた所は `StateSet::insert(StyleState)` のまま通る。
     - `crates/kivact-core/src/devtools/styles.rs` の `"hover" => ForcedStates::HOVER,` などの 6 行も同じ。`let mut states = ForcedStates::default();` は `StateSet::EMPTY`。
     - `crates/kivact-core/src/devtools/node.rs` の `forced` の読み（`all.get(&id)` などで `ForcedStates` を `contains` で調べる所）を `StateSet::contains(StyleState::..)` に。
  6. core の ST-06：
     - `crates/kivact-core/src/view/mount/style.rs` の `pub(crate) fn merge_layers(layers: &[StyleLayer]) -> Style` を次にする（PF-01：重複は debug build でだけ、層ごとに数える）：
       ```rust
       /// The layers merged in order, and in debug builds how many
       /// declarations repeat a property within their own layer.
       pub(crate) fn merge_layers(layers: &[StyleLayer]) -> (Style, u32) {
           let mut duplicates = 0;
           let mut out: Option<Style> = None;
           for layer in layers {
               let s = (layer.f)();
               if cfg!(debug_assertions) {
                   duplicates += kivact_style::internal::duplicates(&s);
               }
               out = Some(match out {
                   Some(acc) => acc.merge(&s),
                   None => s,
               });
           }
           (out.unwrap_or_default(), duplicates)
       }
       ```
       `crates/kivact-style/src/internal.rs` に `pub fn duplicates(style: &Style) -> u32 { style.duplicates() }` を足す。`bind_style` の `hook.apply(merge_layers(&layers))` を `let (merged, duplicates) = merge_layers(&layers); let s = hook.apply(merged);` にし、`StyleReports::report` に引数 `duplicates: u32` を足して `if resolved.duplicates > 0 && cfg!(debug_assertions)` を `if duplicates > 0 && cfg!(debug_assertions)`（文の `resolved.duplicates` も `duplicates`）にする。`merge_layers` のほかの呼出し（`rg -n "merge_layers" crates/kivact-core/src`）は `.0` を使う。
     - `bind_style` の effect：`kivact_style::internal::compile_style(&s, theme, state)` を `compile_style(s, theme, state)`（move）に。`resolved` を作った後の `tree.update(id, PropValue::Paint(resolved.paint.clone()), writer);` などの 4 行の前で `let CompiledStyle { layout, paint, text, states, .. } = resolved;` と分解できないのは `feed_local_paint` と `reports.report` が `&resolved` を使うため。順を次にする：`feed_local_paint(&mcx, id, &local_paint, &resolved, &reports, at);` と `reports.report(&rt, owner, id, &resolved, at);` を先に呼び（report は tree を借りない）、その後 `let kivact_style::CompiledStyle { mut layout, paint, text, states, .. } = resolved;` と分解して `tree.update(id, PropValue::Layout(layout), writer)`（`fix_layout` は分解の後の `layout` に当てる）、`PropValue::Paint(paint)`、`PropValue::TextStyle(text)`、`PropValue::States(states.unwrap_or_default())` を move で渡す。`CompiledStyle` に `pub(crate)` な field があり分解できない場合は、`CompiledStyle` に `pub fn into_props(self) -> (LayoutProps, PaintProps, Option<TextStyle>, Option<StateStyles>)` を足して使う。
     - `crates/kivact-core/src/view/mount/facts.rs` の `LocalPaintSource` の `value: RefCell<Option<Box<CompiledStyle>>>` を `value: RefCell<Option<kivact_style::draw::LocalPaint>>` にし、`feed_local_paint`（mount/style.rs）の `Some(Box::new(resolved.clone()))` を `resolved.retained_paint()`（`CompiledStyle` に `pub fn retained_paint(&self) -> Option<LocalPaint> { self.retained_paint.clone() }` を足す）に、`!old.same_retained_paint(resolved)` を `Some(old) != resolved.retained_paint().as_ref()` の比較（`LocalPaint` の `PartialEq` は identity）にする。`crates/kivact-core/src/view/mount/draw.rs` の `compiled.paint_retained(d, sz)` を `paint.draw(d, sz)`（`let paint = source.value.borrow().clone(); if let Some(paint) = paint { paint.draw(d, sz); }`）にする。`LocalPaint` が `kivact_style::draw` から公開されていなければ、`crates/kivact-style/src/draw.rs` の `pub use local_paint::LocalPaint;` を `#[doc(hidden)] pub use` で足す（core の mount だけが使う内部の橋。理由のコメント `// Core's mount keeps the retained paint callback; not a user API.`）。`CompiledStyle::same_retained_paint` と `paint_retained` は使われなくなるので消す。
  7. `crates/kivact-style/src/sheet/fields.rs` のファイル名を中身に合わせる：`git mv crates/kivact-style/src/sheet/fields.rs crates/kivact-style/src/sheet/folded.rs`、`sheet.rs` の `mod fields;` を `mod folded;`、`fields::` を `folded::` に。Role 行を `//! Role: A style's declarations folded into one record of authored values, which resolution reads.` にする。各 `XFields` の doc の「The `size` group of a [`Style`]」の形の文を「The size properties of a folded style.」の形にする。
- **消すもの**：`group_or_default!`、`ForcedStates`、`CompiledStyle::duplicates`、`CompiledStyle::misplaced_content` の名前、`CompiledStyle::{same_retained_paint, paint_retained}`、`LocalPaintSource` の `CompiledStyle`。
- **状態の閉じ方**：`CompiledStyle` は compile の結果の値の record（`&mut self` の method なし）：対象外。`Interactive` は compile の入力（名前は対象外の接尾辞ではないが `&mut self` の method なし）：対象外。
- **試験**：
  - `crates/kivact-style/src/resolve/compile.rs` に `#[cfg(test)] mod tests`（`//! Role: Tests of style compilation.`）：`fn a_paint_position_that_is_auto_is_counted()`（`style().border_radius(Length::Auto)` は S20 の後の名前。この unit では `style().radius(Length::Auto)` を `compile_style(.., Interactive::default())` して `misplaced_lengths == 8`（4 隅の x と y。数は長さごと））、`fn forced_states_merge_their_styles()`（`Interactive { forced: StateSet::EMPTY.with(StyleState::Hover), .. }` で `hover(style().opacity(0.5))` の opacity が base の paint に入る）。
  - 公開 E2E の `diagnostics.rs`・`devtools.rs`（forced state、style の表示）、`hover.rs`、`style_paint.rs` がそのまま通ること。
- **確認（tier a）**：鎖の共通確認と次：
  ```
  cargo test -p kivact-style
  cargo test -p kivact-types
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native -- devtools
  ```
- **完了条件**：`rg -n "group_or_default|ForcedStates|misplaced_content|same_retained_paint" crates` が 0 件。`rg -n "s\.clone\(\)" crates/kivact-style/src/resolve/compile.rs` が focus_visible の 2 か所だけ。上のコマンドが通る。

---

## S19：状態の style がすべての property を受ける（ST-05）

- **wave / lane / 前提**：W6b / 鎖 / S18
- **所有**：`crates/kivact-style/src/resolve/`、`crates/kivact-style/src/sheet.rs`（module の doc）、`crates/kivact-style/src/sheet/shorthands.rs`（状態の setter の doc）、`crates/kivact-core/src/view/node_states.rs`（新設）、`crates/kivact-core/src/view.rs`（`mod` 宣言）、`crates/kivact-core/src/view/state.rs`、`crates/kivact-core/src/view/mount/style.rs`、`crates/kivact-core/src/client/dispatch/delivery.rs`、`dev/tests/kivact-public-api/tests/e2e_core/`（新しい試験のファイルと `main.rs`、`style_paint.rs` の 1 試験）
- **目的**：ST-05（O-2 採用）。`hover`・`active`・`focus`・`focus_visible`・`group_hover`・`group_focus` の style は、host が差分として塗れる property（分類 `Delta`）を今まで通り host の paint delta にし、それ以外（分類 `Rest`：layout、文字の大きさ・装飾、塗りの callback など）を、engine が報告する状態（S04 の `HostEvent::StatesChanged`）を読んで core が style を compile し直すときに base に merge する。`enter`・`exit` は CSS の `@starting-style` と同じく、遷移で動く property（paint）だけが見え方に効き、それ以外は受け付けて効果を持たない（報告もしない）。`disabled`・`selected` は今まで通り core が merge する。「状態が変えられない property」の報告（`unsupported_in_states`）を消す。
- **手順**：
  1. style（`crates/kivact-style/src/resolve/compile.rs`）：
     - `Interactive` に field を足す：doc `/// The host states the engine reported active for this node (HostEvent::StatesChanged): their non-paint declarations are compiled into the style.` を付けた `pub active: StateSet,`。
     - `compile_style` の `s = merge_forced(s, state.forced);` の直後（S18 の `let retained_paint = s.local_paint(); s.strip_local_paint();` より **前**。12 章 UI-S-02：状態の report は callback を含む元の宣言から決め、選ばれた状態の Rest（callback を含む）を合成してから callback を選ぶ）に足す：
       ```rust
       // What the host must report: decided from the declarations as written,
       // paint callbacks included, before anything is stripped.
       let report = report_of(&s);
       // The host states that set more than paint, in the order their deltas stack.
       for host in StyleState::HOST {
           if state.active.contains(host)
               && !matches!(host, StyleState::Enter | StyleState::Exit)
               && let Some(st) = s.state(host)
           {
               s = s.merge(&st.rest());
           }
       }
       ```
       `crates/kivact-style/src/sheet/decl.rs` の `impl Style` に `pub(crate) fn rest(&self) -> Style { Style { decls: self.decls.iter().filter(|d| d.class() == PropClass::Rest && !matches!(d, Decl::State(..) | Decl::Group)).cloned().collect() } }` を足す（`LocalPaint` は `Rest` なので、active な状態の callback は base に入り、その後の `s.local_paint()` が最後のもの（`StyleState::HOST` の順で後の状態が勝つ）を選ぶ）。
     - 同じファイルに足す：
       ```rust
       /// The host states whose style sets more than paint (a paint callback
       /// included): the engine reports them for this node (ST-05).
       fn report_of(s: &Style) -> StateSet {
           let mut report = StateSet::EMPTY;
           for host in StyleState::HOST {
               if matches!(host, StyleState::Enter | StyleState::Exit) {
                   continue;
               }
               if s.state(host).is_some_and(|st| !st.rest().decls.is_empty()) {
                   report.insert(host);
               }
           }
           report
       }
       ```
       `fn state_styles` に引数 `report: StateSet` を足し、作る `StateStyles` に `report` を入れる（`.then(|| { ... })` の中で `out.report = report;`）。`state_styles` の `.then` の条件に `|| !report.is_empty()` を足す（callback だけの状態でも表を作る）。
     - `CompiledStyle` の `unsupported_in_states` を `nested_in_states: Vec<String>`（doc：「The states and groups set inside a host state style, as `state.name`; they are ignored.」）に改名し、`describe.rs` の `pub(super) fn unsupported_in_states` を次の `nested_in_states` に置き換える（状態の style の中の状態と `group()` は意味を持たないので、報告を残す。2026-11-21 の設計者の回答）：
       ```rust
       /// The states and groups set inside the host state styles of `s`
       /// (`CompiledStyle::nested_in_states`); they are ignored.
       pub(super) fn nested_in_states(s: &Style) -> Vec<String> {
           let mut out = Vec::new();
           for state in kivact_types::StyleState::HOST {
               if let Some(st) = s.state(state) {
                   out.extend(
                       st.decls
                           .iter()
                           .filter(|d| matches!(d, Decl::State(..) | Decl::Group))
                           .map(|d| format!("{}.{}", state.name(), d.name())),
                   );
               }
           }
           out
       }
       ```
       `compile.rs` の呼出しと `use` も新しい名前にする。
     - `crates/kivact-style/src/resolve/paint.rs` の `paint_delta` の `sets_border` を、border の色か line style が設定されているときだけにする（幅は `Rest` で、状態が active のとき base に入る）：`let sets_border = st_paint.border_color.iter().any(Option::is_some) || st_paint.border_style.iter().any(Option::is_some);`。
  2. style の doc：`crates/kivact-style/src/sheet.rs` の module の doc の「Interaction states:」の最初の項を次にする：
     「`hover`, `active`, `focus`, `focus_visible`, `group_hover` and `group_focus` accept every property. The paint ones (background, border colors and line styles, outline, radii, opacity, shadows, transforms, z-index, scrollbar, cursor, filters, mask, clip path, text color) are deltas the host draws at once; the others (sizes, spacing, border widths, text properties, a paint callback) are compiled into the node's style while the host reports the state, so they lay out again like CSS. `enter` and `exit` are where a paint transition starts and ends (CSS `@starting-style`); properties a transition does not move have no effect there. `disabled` and `selected` are merged when the style is compiled.」
     同じ文の要旨を、`hover`・`pressed`（S20 で `active`）・`focus`・`focus_visible`・`group_hover`・`group_focus` の setter の doc の「A state style changes paint only ... reported as `Unsupported` and ignored.」の文の代わりに書く（「Every property applies; see the module doc.」の 1 文でよい）。
  3. core：`crates/kivact-core/src/view/state.rs` の `pub(crate) struct ViewState` に field を足し、`ViewState` を作る式で `RefCell::new(HashMap::new())` にする：
     ```rust
     /// The reported host states of the nodes whose style asked for them
     /// (`view::node_states`), by node.
     pub(crate) node_states: RefCell<std::collections::HashMap<NodeId, Signal<kivact_types::StateSet>>>,
     ```
  4. `crates/kivact-core/src/view/node_states.rs` を作り、`crates/kivact-core/src/view.rs` に `pub(crate) mod node_states;` を足す：

```rust
//! Role: The host states a node's style asked for, as one reactive source per node (ST-05).
//!
//! The engine reports them (`HostEvent::StatesChanged`) only for a node whose
//! compiled state styles set more than paint; the node's style effect reads
//! the source and compiles those declarations in.

use std::panic::Location;

use kivact_runtime::{Owner, Runtime, Signal};
use kivact_types::{NodeId, StateSet};

use crate::view::state::ViewState;

/// The source of `id`'s reported states, made under `owner` (the node's
/// owner) on the first call and dropped with it.
pub(crate) fn source(rt: &Runtime, id: NodeId, owner: Owner, at: &'static Location<'static>) -> Signal<StateSet> {
    if let Some(sig) = ViewState::of(rt).node_states.borrow().get(&id).copied() {
        return sig;
    }
    let sig = owner.signal(StateSet::EMPTY, at);
    ViewState::of(rt).node_states.borrow_mut().insert(id, sig);
    let app = rt.id();
    owner.on_cleanup(move || {
        if let Some(rt) = Runtime::get(app) {
            ViewState::of(&rt).node_states.borrow_mut().remove(&id);
        }
    });
    sig
}

/// The host reported `states` for `node`: set its source (a node without
/// one, or unmounted already, is ignored).
pub(crate) fn states_changed(rt: &Runtime, node: NodeId, states: StateSet) {
    let sig = ViewState::of(rt).node_states.borrow().get(&node).copied();
    if let Some(sig) = sig {
        rt.enter(|| sig.set(states));
    }
}
```

     （`Runtime::get(app)`・`rt.id()`・`rt.enter`・`Owner::on_cleanup`・`Owner::signal` は 2b8df5ac の名前。W5 で変わっていれば付属書 R の対応表の名前にする。`Signal` が `Copy` でない場合は `.copied()` を `.cloned()` にする。）
  5. `crates/kivact-core/src/view/mount/style.rs` の `bind_style`：
     - effect の前に `let reported: std::cell::Cell<Option<Signal<StateSet>>> = std::cell::Cell::new(None);` を作り、effect の closure に move する。
     - effect の中の `Interactive { ... }` に `active: reported.get().map(|sig| sig.get()).unwrap_or_default(),` を足す（tracked：報告が変わると effect が再実行される）。
     - `let resolved = ...;` の後に足す：
       ```rust
       let wants = resolved.states.as_ref().is_some_and(|s| !s.report.is_empty());
       if wants && reported.get().is_none() {
           let sig = crate::view::node_states::source(&rt, id, owner, at);
           // Track it from now on: the engine reports when the node's state changes.
           let _ = sig.get();
           reported.set(Some(sig));
       }
       ```
     - `struct StyleReports` の `states: ReportOnce` field は残す。`fn report` の `if !resolved.unsupported_in_states.is_empty() && self.states.note() { ... }` の塊を、条件を `!resolved.nested_in_states.is_empty() && self.states.note()` に、文を `format!("state styles set a state or group inside a state, ignored: {}; set each state and group() on the base style", resolved.nested_in_states.join(", "))` にする（`ErrorKind::Unsupported` のまま）。
  6. `crates/kivact-core/src/client/dispatch/delivery.rs`：S04 で足した `HostEvent::StatesChanged { .. } => {}`（と直前のコメント）を `HostEvent::StatesChanged { node, states } => crate::view::node_states::states_changed(rt, *node, *states),` にする（`dispatch_host` に S04 で arm を足していない場合は、`dispatch_host` の `match host_event {` の `HostEvent::DuplicateLayoutId` の腕の前に足す）。
- **消すもの**：`CompiledStyle::unsupported_in_states`、`describe::unsupported_in_states`（`nested_in_states` に置き換え）と「state styles set properties their state cannot change」の報告。
- **状態の閉じ方**：`bind_style` の effect の local の `Cell<Option<Signal<StateSet>>>` は一つ（`locals` の検査は二つ以上が対象）。`ViewState.node_states` は map の field。
- **試験**：`dev/tests/kivact-public-api/tests/e2e_core/state_styles.rs` を作り、`main.rs` に `mod state_styles;` を足す。hover・press・focus の操作と大きさの読み方は、同じ binary の `hover.rs`・`pointer.rs`・`focus.rs` の既存の試験と同じ helper（`screen.get_by_test_id(..)` の `hover()`／`press()`／`focus()` と、box の大きさの取得）を使う：
  - `fn hover_applies_a_layout_property()`：`node().test_id("box").style(style().width(40).height(20).hover(style().width(80)))` を描き、hover の前の幅が 40、hover の後の幅が 80、pointer が離れた後に 40。
  - `fn active_applies_a_layout_property()`：`.active(..)`（S20 の後の名前。S19 の時点では `.pressed(..)`）で `height(40)`。press して離す前に高さ 40、離した後に 20。
  - `fn focus_applies_a_text_property()`：focusable な `node()` の `.focus(style().font_size(30))` の子の text の高さが focus の前より大きい。
  - `fn a_paint_only_state_stays_a_host_delta()`：`hover(style().background(rgb(0xff0000)))` だけの node を hover すると、screenshot の node の中心が赤になり、`screen.errors()` が空。
  - UI-S-02（状態の paint callback）。同じ `state_styles.rs` に、`node().test_id("box").style(style().size(40, 40).<状態>(style().paint(|d, size| d.fill_rect(size.to_rect(), rgb(0xff0000)))))` の形で、screenshot の box の中心で確かめる：
    - `fn hover_draws_its_paint_callback()`：hover の前は赤でない、hover の間は赤、離れた後は赤でない。
    - `fn active_paint_replaces_hover_paint()`：`hover` の callback が青、`active` の callback が赤。hover で青、press の間は赤、release で青（後の状態が勝つ）。
    - `fn focus_draws_its_paint_callback()`：focusable な node（`node().focusable(true)` など focus を取れる形）を focus すると赤、blur で赤でない。
    - `fn disabled_and_selected_draw_their_paint_callbacks()`：`disabled` の callback が赤、`selected` の callback が緑の node を、node の disabled・selected の modifier（`bind_states` が読むもの）に `signal(false)` を渡して描き、disabled を true にすると赤、false に戻して selected を true にすると緑、両方 false で赤でも緑でもない。
    - `fn a_base_paint_comes_back_after_the_state()`：base の `paint` が青、`hover` の callback が赤。hover で赤、離れて青（置き換えと復帰）。
  - 既存の `dev/tests/kivact-public-api/tests/e2e_core/style_paint.rs` の `fn a_property_its_state_cannot_change_is_reported` を `fn a_state_inside_a_state_is_reported` に改名し、style は同じ（`hover(style().width(80).opacity(0.5).hover(style().opacity(1.0)))`）のまま、期待を「`errors` が 1 件で `ErrorKind::Unsupported`、message が `hover.hover` を含み、`hover.width` を含まない」にする（幅は ST-05 で効く。報告の試験はこの 1 件だけを直す）。
  - `crates/kivact-style/src/resolve/compile.rs` の `mod tests` に `fn a_state_paint_callback_asks_for_reports()`（`style().hover(style().paint(|_, _| {}))` の `states.unwrap().report.contains(StyleState::Hover)` で、hover の外の compile（`active` 空）の `has_retained_paint()` が false、`Interactive { active: StateSet::EMPTY.with(StyleState::Hover), .. }` の compile で true）と `fn only_states_with_more_than_paint_ask_for_reports()`：`style().hover(style().background(rgb(0xff0000)))` を compile した `states.unwrap().report` が空、`style().hover(style().width(10))` では `report.contains(StyleState::Hover)`。
- **確認（tier a）**：鎖の共通確認と次：
  ```
  cargo test -p kivact-style
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- state_styles
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- hover
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- pointer
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- focus
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- motion
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- presence
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui
  ```
- **完了条件**：`rg -n "unsupported_in_states|state cannot change" crates` が 0 件。`state_styles` の 4 試験が通る。上のコマンドが通る。

---

## S20：CSS の名前と、CSS の値の型の `kivact::style` への移動（ST-22）

- **wave / lane / 前提**：W6b / 鎖 / S19
- **所有**：`crates/kivact-style/`、`crates/kivact-core/src/style.rs`、`crates/kivact-core/src/draw.rs`、`crates/kivact-core/src/view.rs`（`IntrinsicSize` の再公開）、`crates/kivact-core/src/view/span.rs`、`crates/kivact-types/src/text/primitives.rs`（`FontStyle`）、`crates/kivact-types/src/text.rs`・`lib.rs`（再公開）、`crates/kivact/src/prelude.rs`、W6 の掃引の範囲
- **目的**：ST-22・DEC-07（O-5：CSS の名前）。setter を CSS の名前にし、CSS の値の型を `kivact::style` だけに置く（`kivact::draw` は描画者の型だけ。設計書 5.8）。ST-30：`ScrollbarWidth::Thin` を消す。
- **手順**：
  1. **表の名前**（`crates/kivact-style/src/sheet/props.rs` の setter と longhand の名前、`shorthands.rs` の関数名）を 4.5 節の表どおりに変える。表の行の setter 名を変え、variant 名も CSS に揃える（`FontWeight` はそのまま、`Italic(bool)` → `FontStyle(FontStyle)`、`RadiusTopLeft` → `BorderTopLeftRadius` など 4 つ、`RingWidth`/`RingOffset`/`RingColor` → `OutlineWidth`/`OutlineOffset`/`OutlineColor`、longhand の名前の文字列も `"outline_width"` などに）。新しい setter（`outline_offset(v: impl Into<f64>)`、`outline_width(v: impl Into<f64>)`、`outline_color(v: impl Into<Color>)`）を表の setter の行として足す（今までは shorthand だけが作った longhand）。`shorthands.rs`：`ring(width, color)` → `outline(width, color)`（`OutlineWidth` と `OutlineColor` を push。offset は push しない）、`ring_with_offset` を消す、`ring_none()` → `outline_none()`（`self.outline(0, Color::TRANSPARENT)`）、`radius(r)` → `border_radius(r)`、`radius_corners(..)` → `border_radius_corners(..)`。状態の `pressed(s)` → `active(s)`。
  2. **FontStyle**：`crates/kivact-types/src/text/primitives.rs` に足す：
     ```rust
     /// The face of a font family to use (CSS `font-style`).
     #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
     pub enum FontStyle {
         /// The upright face.
         #[default]
         Normal,
         /// The italic face, when the family has one.
         Italic,
     }
     ```
     `crates/kivact-types/src/text.rs` と `lib.rs` の再公開に `FontStyle` を足し、`crates/kivact-runtime/src/reactive/props.rs` の `constant_value!(kivact_types::...)` の並びに `kivact_types::FontStyle,` を足す。表の `italic(v: bool) => Italic(bool) = v` を `font_style(v: FontStyle) => FontStyle(FontStyle) = v` にし、fold の `Decl::FontStyle(v) => self.text.italic = Some(*v == FontStyle::Italic),` にする。style の公開に `FontStyle` を足す（`lib.rs`・`facade.rs`。`pub use kivact_types::FontStyle;`）。
  3. **Direction**：`crates/kivact-style/src/sheet.rs` の `pub use kivact_types::FlexDirection as Direction;` を `pub use kivact_types::FlexDirection;` にし、`lib.rs`・`facade.rs` の公開の `Direction` を `FlexDirection` にする。
  4. **ScrollbarWidth::Thin**：`crates/kivact-style/src/sheet/folded.rs` の `ScrollbarWidth` の `Thin` の variant（doc を含む）を消し、`resolve/paint.rs` の `ScrollbarWidth::Auto | ScrollbarWidth::Thin => 6.0` を `ScrollbarWidth::Auto => 6.0` にする。呼出し元の `ScrollbarWidth::Thin`（`rg -n "ScrollbarWidth::Thin" $W6_SCOPE $W6_TESTS`）は `ScrollbarWidth::Auto` にする（同じ 6 px）。
  5. **Span**（core の rich text の builder。4.5 節の AC-4 の名前の検査のため）：`crates/kivact-core/src/view/span.rs` の `pub fn weight(mut self, v: u16) -> Self` を `pub fn font_weight(..)` に、`pub fn italic(mut self) -> Self` を `pub fn font_style(mut self, v: FontStyle) -> Self`（`v == FontStyle::Italic` を今の `italic` の field に入れる）にする。
  6. **呼出し元の改名**（掃引の範囲全体）：
     ```
     FILES=$(rg -l "\.(weight|radius|radius_corners|radius_top_left|radius_top_right|radius_bottom_right|radius_bottom_left|ring|ring_none|basis|grow|shrink|direction|pressed)\(" $W6_SCOPE $W6_TESTS --type rust)
     perl -pi -e 's/\.weight\(/.font_weight(/g; s/\.radius_corners\(/.border_radius_corners(/g; s/\.radius_top_left\(/.border_top_left_radius(/g; s/\.radius_top_right\(/.border_top_right_radius(/g; s/\.radius_bottom_right\(/.border_bottom_right_radius(/g; s/\.radius_bottom_left\(/.border_bottom_left_radius(/g; s/\.radius\((?!\))/.border_radius(/g; s/\.ring_none\(\)/.outline_none()/g; s/\.ring\(/.outline(/g; s/\.basis\(/.flex_basis(/g; s/\.grow\(/.flex_grow(/g; s/\.shrink\(/.flex_shrink(/g; s/\.direction\((?!\))/.flex_direction(/g; s/\.pressed\(/.active(/g' $FILES
     rg -l "\bDirection\b" $W6_SCOPE $W6_TESTS --type rust | xargs perl -pi -e 's/(?<![A-Za-z_])Direction\b/FlexDirection/g'
     ```
     2 行目の後、`sheet.rs` に `FlexDirection as FlexDirection` ができていれば手順 3 の形に直す。置換してはいけない所を戻す（2b8df5ac の調査。行番号は目安で、名前で探す）：
     - `.border_radius(` になった `Gradient::radius(r)`（`radial_gradient().radius(20)` の chain。`dev/tests/kivact-public-api/tests/*/paint.rs` と `crates/kivact-style/src/paint/gradient.rs` の doc）→ `.radius(` に戻す。
     - `.font_weight(` は `Span` の改名と一致するのでそのまま（手順 5）。
     - `.outline(` になった free 関数 `ring(s)` の呼出しは無い（`.ring(` は method だけ）。
     - `.flex_shrink(` が arvane の `GpuContext::shrink` に当たらないこと（掃引の範囲外）。
     手で直すもの：
     - `.ring_with_offset(a, b, c)`（2b8df5ac で `crates/kivact-ui/src/look.rs`、`look_feedback.rs`、`look_input.rs`。`rg -n "ring_with_offset" $W6_SCOPE $W6_TESTS`）→ `.outline(a, c).outline_offset(b)`。
     - `.wrap(bool)` の Style の 4 か所（2b8df5ac：`crates/kivact-ui/src/combobox/look.rs`、`look_controls.rs`、`look_input.rs`、`tag_input.rs`。`style()` の chain の `.wrap(true)`／`.wrap(false)`）→ `.flex_wrap(..)`。`CodeBlock::wrap`、`TextInput::wrap`、`TextEditApi::wrap`、`RowRender::wrap` は変えない。
     - `.italic(` の Style の呼出し（2b8df5ac で 0 件）と `Span::italic()` の呼出し（`rg -n "\.italic\(\)" $W6_SCOPE $W6_TESTS`）→ `.font_style(FontStyle::Italic)`（`FontStyle` を import）。
     - DevTools の style の override の property の名前は、C19（W6b、S17 の直後）の後は S17 の `style_property` の表（`Decl` の名前）から来るので、手順 1 の表の改名で新しい名前（`"flex_grow"`、`"border_radius"`、`"font_weight"` など）になる。`crates/kivact-devtools` と試験が旧い名前の文字列を送っていれば（`rg -n "\"grow\"|\"shrink\"|\"radius\"|\"weight\"|\"basis\"|\"direction\"|\"wrap\"|\"ring\"" crates/kivact-devtools dev`）新しい名前にする。
  7. **CSS の値の型の移動**：
     - `crates/kivact-style/src/facade.rs` に足す（`kivact::style` の公開）：`pub use crate::effect::{BlendMode, CornerRadius, Filter, Shadow, Shape};`、`pub use crate::paint::{BackgroundClip, BackgroundOrigin, CanvasGradient, Gradient, GradientExtend, Mask, MaskComposite, MaskLayer, MaskMode, Paint, RadialExtent, conic_gradient, linear_gradient, radial_gradient};`（`BorderImage`・`BorderImageRepeat` は既にある）、`pub use crate::transform::Transform;`、`pub use crate::draw::{FilterGraph, FilterInput, FilterShader, PaintShader, ShaderFilter, ShaderPaint, ShaderValue};` と `pub use crate::draw::filter;`（各型が実際に定義されている module の path で書く。`crates/kivact-core/src/draw.rs` の今の `pub use kivact_style::...` の行が正しい path）。
     - `crates/kivact-core/src/draw.rs` を次の公開だけにする（module の doc の「The paints (...) are the painter's vocabulary and live here; element styles take the same values」の文を「CSS values (paints, gradients, shadows, filters, transforms, shapes, masks) are in `kivact::style`; drawing takes them from there.」にする）：`Draw, FillRule, ImageOptions, InlineBox, InlineBoxPlacement, InlineParagraph, Layer, LineCap, LineJoin, Path, PathContour, Pixels, RegionId, Stroke, TextBaseline, TextFormat, TextMetrics, TextParagraph` と `Offscreen`（`kivact_offscreen::Offscreen` の再公開。C02 の形のまま）。`TextRun`・`text_run`・`IntrinsicSize`・`Composite` を消す（`Composite` は S21 で `Mix` と一緒に戻す）。
     - `IntrinsicSize`：`crates/kivact-core/src/view.rs` の公開に `pub use kivact_types::IntrinsicSize;` を足す（`node().intrinsic_size(..)` の値。`kivact::view::IntrinsicSize`）。
     - 呼出し元の path を直す。次の script を scratch に `move_draw.pl` として保存する（第 1 引数が root：`kivact`、`kivact_core`、`crate`）：

```perl
#!/usr/bin/perl
# usage: perl move_draw.pl ROOT FILE...
use strict;
use warnings;
my $root = shift @ARGV;
my %style = map { $_ => 1 } qw(BlendMode CornerRadius Filter Shadow Shape BackgroundClip
    BackgroundOrigin CanvasGradient Gradient GradientExtend Mask MaskComposite MaskLayer MaskMode
    Paint RadialExtent conic_gradient linear_gradient radial_gradient Transform BorderImage
    BorderImageRepeat FilterGraph FilterInput FilterShader PaintShader ShaderFilter ShaderPaint
    ShaderValue filter);
my %view = (IntrinsicSize => 1);
sub home { my $n = shift; $n =~ s/^\s+|\s+$//g; my ($head) = $n =~ /^(\w+)/;
    return exists $style{$head} ? 'style' : exists $view{$head} ? 'view' : 'draw'; }
for my $f (@ARGV) {
    local $/;
    open my $in, '<', $f or die "$f: $!";
    my $s = <$in>;
    close $in;
    $s =~ s{(pub\s+)?use\s+\Q$root\E::draw::\{([^{}]*)\};}{
        my ($vis, $list) = ($1 // '', $2);
        my %by;
        for my $n (grep { /\S/ } split /,/, $list) { (my $t = $n) =~ s/^\s+|\s+$//g; push @{$by{home($t)}}, $t; }
        join("\n", map { "${vis}use ${root}::$_\::{" . join(', ', @{$by{$_}}) . "};" } sort keys %by);
    }ge;
    $s =~ s{\b\Q$root\E::draw::(\w+)}{ "${root}::" . home($1) . "::$1" }ge;
    open my $out, '>', $f or die "$f: $!";
    print $out $s;
    close $out;
}
```

       ```
       perl move_draw.pl kivact $(rg -l "kivact::draw::" crates/kivact-devtools crates/kivact/src $W6_TESTS dev/bench examples/kivact templates/basic crates/kivact-style/src crates/kivact-core/src crates/kivact-ui/src crates/kivact-interaction/src --type rust)
       perl move_draw.pl kivact_core $(rg -l "kivact_core::draw::" crates/kivact-ui/src crates/kivact-interaction/src --type rust)
       perl move_draw.pl crate $(rg -l "crate::draw::" crates/kivact-core/src --type rust)
       ```
       （1 行目は doc の例の `kivact::draw::...` も直す。`crates/kivact-core/src/draw/` の中の `crate::draw::` は core の draw module 自身の子なので、3 行目の結果で compile が通らない行があれば戻す。）同じ module から二つの `use` 行ができて import が重複した場合は一つにまとめる（`cargo fmt` は行をまとめない）。
  8. ST-29 の doc：`crates/kivact-style/src/text.rs` の module の doc の「- `Draw`'s `TextStyle` has the same vocabulary.」を「- `Draw`'s `TextFormat` has the same vocabulary.」に、「CSS's property names in snake case」の文はそのまま（名前が CSS になった）。`crates/kivact-style/src/paint.rs` の doc の「except `Background` and `BorderStyle`, which only a style takes」の文を「`kivact::style` exports them all; `Draw` takes the paints too.」にする。`crates/kivact-style/src/lib.rs` の doc の「The implementation behind `kivact::style` and `kivact::draw`: ... paints, shapes, strokes, filters, blending and transforms that styles share with it (they live in `effect`, `paint`, `draw` and `transform`)」を「The implementation behind `kivact::style` (the `Style` sheet, the CSS values, the theme and its variables) and `kivact::draw` (the `Draw` painter and `Path`).」にし、`/// Visibility of a layout box and its subtree.` を `/// Whether a layout box and its subtree are painted and take input (CSS \`visibility\`).` にする。
  9. prelude（`crates/kivact/src/prelude.rs`）：`pub use crate::draw::Transform;` を消す（設計書 5.11 の一覧に `Transform` は無い）。`Shadow`・`Stroke`・`Path`・`Draw` の prelude の path は 5.11 のとおり（`Shadow` は `crate::style::Shadow`、`Draw`・`Path`・`Stroke` は `crate::draw`）。付属書 C が W5 で 5.11 の prelude を書いていれば、ここでは `Shadow` の path だけを直す。
- **消すもの**：旧 setter（`weight`、`italic`、`radius`、`radius_corners`、`radius_top_left` ほか 3、`ring`、`ring_with_offset`、`ring_none`、`basis`、`grow`、`shrink`、`wrap`（Style）、`direction`、`pressed`）、`Direction`（style の別名）、`ScrollbarWidth::Thin`、`Span::{weight, italic}`、`kivact::draw` の CSS の値の型と `TextRun`・`text_run`・`IntrinsicSize`。
- **状態の閉じ方**：新しい型（`FontStyle`）は field の無い enum。
- **試験**：名前の置換だけ。公開 E2E の全体（`e2e_core`、`e2e_ui`、`e2e_native`）が通ること。
- **確認（tier a）**：鎖の共通確認と次：
  ```
  cargo test -p kivact-style
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test offscreen
  ```
- **完了条件**：`rg -n "\.(weight|italic|ring|ring_none|ring_with_offset|basis|grow|shrink|pressed)\(|\.radius\((?!\))|\.direction\((?!\))" -P $W6_SCOPE $W6_TESTS --type rust` が、`Gradient::radius` の chain と 4.13 節に挙げた Style でない受け手だけを返す。`rg -n "kivact::draw::(Shadow|Filter|Transform|Paint|Shape|BlendMode|CornerRadius|Mask|BorderImage|FilterGraph|PaintShader|FilterShader)\b" $W6_SCOPE $W6_TESTS` が 0 件。上のコマンドが通る。

---

## S21：`Layer::mix`（ST-17 の公開 API）

- **wave / lane / 前提**：W6b / 鎖 / S20
- **所有**：`crates/kivact-style/src/draw/layer.rs`、`crates/kivact-style/src/draw.rs`、`crates/kivact-core/src/draw.rs`、`crates/kivact-test/src/screen/gpu_tests.rs`、`dev/tests/kivact-public-api/tests/*/canvas.rs`・`compositor.rs`・`offscreen_svg.rs`
- **目的**：ST-17。`Layer::blend` と `Layer::composite`（最後の方が勝つ暗黙の排他）を `Layer::mix(Mix)` 一つにする。
- **手順**：
  1. `crates/kivact-style/src/draw/layer.rs`：`pub fn blend(..)` と `pub fn composite(..)` を消し、次を足す：
     ```rust
     /// How the layer combines with what is beneath it: `Mix::Blend(mode)`
     /// (CSS `mix-blend-mode`) or `Mix::Composite(op)` (Canvas
     /// `globalCompositeOperation`); `Mix::Normal` by default.
     pub fn mix(mut self, mix: Mix) -> Self {
         self.mix = mix.normalized();
         self
     }
     ```
     型の doc の「opacity, blend or composite, filter, mask, shadow」を「opacity, mix, filter, mask, shadow」にする。doc の例はそのまま（`blend` を使っていない）。
  2. `crates/kivact-style/src/draw.rs` の `pub use kivact_types::Composite;`（S02 の後の行）の隣に `pub use kivact_types::Mix;` を足す。`crates/kivact-core/src/draw.rs` の公開に `Composite`、`Mix` を足す（`kivact::draw::{Mix, Composite}`。`Mix::Composite(Composite::DestinationOut)` を書けるように）。
  3. 呼出し元（2b8df5ac の調査）：
     - `crates/kivact-test/src/screen/gpu_tests.rs` の `Layer::new() ... .blend(BlendMode::Screen)` → `.mix(Mix::Blend(BlendMode::Screen))`。
     - `dev/tests/kivact-public-api/tests/*/canvas.rs` の `Layer::new().blend(BlendMode::PlusLighter)` → `Layer::new().mix(Mix::Blend(BlendMode::PlusLighter))`。
     - `dev/tests/kivact-public-api/tests/*/compositor.rs` の `Layer::new().composite(Composite::DestinationOut)` → `Layer::new().mix(Mix::Composite(Composite::DestinationOut))`。
     - `dev/tests/kivact-public-api/tests/*/offscreen_svg.rs` の `Layer::new().composite(Composite::DestinationOut)`（S11 で変えた行）と `Layer::new().blend(..)` → 同じ規則。
     - ほかに `Layer` の chain の `.blend(`・`.composite(` があれば（`rg -n "\.blend\(|\.composite\(" $W6_SCOPE $W6_TESTS` のうち受け手が `Layer` のもの）同じ規則。`Paint::blend(..)`（背景の層の blend）と `MaskLayer::composite(..)` は変えない。
     各ファイルの import に `Mix` を足す（`kivact::draw::{.., Mix}`）。
- **消すもの**：`Layer::blend`、`Layer::composite`。
- **状態の閉じ方**：変更なし（`Layer.mix` は S03）。
- **試験**：上の 4 ファイルの書換えだけ（期待値は変えない）。
- **確認（tier a）**：鎖の共通確認と次：
  ```
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- canvas
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- compositor
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- offscreen_svg
  ```
- **完了条件**：`rg -n "pub fn (blend|composite)\(" crates/kivact-style/src/draw/layer.rs` が 0 件。上のコマンドが通る。

---

## S22：rich text の span と編集の range を一つの文字の style にする（ST-11）

- **wave / lane / 前提**：W6b / 鎖 / S21
- **所有**：`crates/kivact-types/src/text/properties.rs`、`crates/kivact-types/src/text.rs`・`lib.rs`、`crates/kivact-style/src/text.rs`、`crates/kivact-style/src/internal.rs`、`crates/kivact-style/src/lib.rs`・`facade.rs`、`crates/kivact-core/src/view/span.rs`、`crates/kivact-core/src/custom/text_editing.rs`、`crates/kivact-engine/src/lower/conversion.rs`、`crates/kivact-engine/src/measure/leaf.rs`、`crates/kivact-engine/src/text_input/shaping.rs`、`crates/kivact-engine/src/host/write/typed.rs`、`crates/kivact-engine/src/tree/node.rs`、`crates/kivact-ui/src/code_block.rs`、`crates/kivact-ui/src/code_editing.rs`、`dev/tests/kivact-public-api/tests/*/custom_text_field.rs`・`text_editing.rs`・`ui_chat_code.rs`
- **目的**：ST-11。rich text の span の style、編集の range の style、段落の style を一つの語彙にする：作者の値は `kivact_style::Style`（文字の property と background）、解決した値は `kivact_types::TextStyle`。`SpanStyle`（style）、`CompiledSpanStyle`、`compile_span_style`、`compile_text_decoration`、`SpanProps`（types）を消す。span の装飾の line style が常に実線になる不具合を直す。`TextFormat`（`Draw` の文字の書式）は変えない（4.13 節）。
- **手順**：
  1. types：`crates/kivact-types/src/text/properties.rs` の `pub struct SpanProps` を消し、`pub struct TextSpan` を次にする：
     ```rust
     /// A byte range of a rich text's `Content` and what it sets over the
     /// paragraph's text style (`None` fields are the paragraph's).
     #[derive(Clone, Debug, PartialEq)]
     pub struct TextSpan {
         pub range: Range<usize>,
         /// The text properties it sets.
         pub style: TextStyle,
         /// Filled behind the span's glyphs, one band per line (a `<mark>`).
         pub background: Option<Fill>,
     }
     ```
     `text.rs`・`lib.rs` の再公開から `SpanProps` を消す。
  2. engine：`SpanProps` の読み手を `TextStyle` の field に置き換える（`rg -n "SpanProps|\.style\.(decoration|background)" crates/kivact-engine/src`）。`crates/kivact-engine/src/lower/conversion.rs` の span を protocol の `ResolvedSpanStyle` に写す関数で：
     - 同じ名前の field（`font_size`、`weight`、`color`、`family`、`italic`、`kerning`、`stroke`、`variations`、`font_width`）は `span.style.<field>` を読む（`stroke` の型が違えば今の変換の式を使う）。`letter_spacing`・`word_spacing` は `TextLength` の px への今の換算をそのまま使う。
     - `features`：段落の features と同じ規則（`tabular_nums == Some(true)` なら `tnum` を足す）で作る。engine が段落の `TextStyle` に対してこれをしている関数（`rg -n "tabular_nums" crates/kivact-engine/src`）を呼ぶ。
     - `decoration`：`let s = &span.style; let decorated = s.decoration_lines.is_some() || s.decoration_color.is_some() || s.decoration_thickness.is_some() || s.underline_offset.is_some() || s.decoration_style.is_some();` が真なら `Some(TextDecoration { lines: s.decoration_lines.unwrap_or_default(), style: s.decoration_style.unwrap_or_default(), color: s.decoration_color, thickness: s.decoration_thickness, underline_offset: s.underline_offset.unwrap_or_default() })`、偽なら `None`（span の装飾の style が届く。ST-11 の BUG）。
     - background は `span.background`。
     ほかの engine のファイル（`measure/leaf.rs`、`text_input/shaping.rs`、`host/write/typed.rs`、`tree/node.rs`）は型の名前の置換だけ（`SpanProps` → `TextStyle`）で通ることを確かめる。
  3. style：`crates/kivact-style/src/text.rs` の `pub struct SpanStyle`、`pub struct CompiledSpanStyle`、`pub struct StyledRange`、`pub struct TextDecoration`（style の作者の装飾の値）、`fn decoration`（`TextDecorationStyle::default()` を固定で入れていたもの）を消す。`crates/kivact-style/src/internal.rs` の `compile_span_style` と `compile_text_decoration` を消し、次を足す：
     ```rust
     /// A span's or an editing range's style compiled against `theme`: its
     /// text properties, its background and how many declarations a span
     /// cannot take (not text and not `background`; reported as
     /// `Unsupported`).
     pub fn compile_span(style: &Style, theme: &Theme) -> (p::TextStyle, Option<p::Fill>, u32) {
         let rejected = style
             .decls
             .iter()
             .filter(|d| d.group() != PropGroup::Text && !matches!(d, Decl::Background(_)))
             .count() as u32;
         let g = style.fold();
         let (text, dropped) = crate::resolve::text_of(&g, theme);
         let background = g.paint.background.as_ref().map(|b| crate::resolve::fill_of(b, theme));
         (text, background, rejected + dropped)
     }
     ```
     `crate::resolve::text_of` は S18 の `fn text(g: &Groups, theme) -> (TextStyle, u32)` を `pub(crate) fn text_of` として公開したもの、`fill_of` は `resolve/paint.rs` の `fn fill(b: &Background, theme) -> Fill` を `pub(crate) fn fill_of` にしたもの（`&mut misplaced` を取るなら `&mut 0` を渡す）。
     新しい公開の `StyledRange`：`crates/kivact-style/src/text.rs` に
     ```rust
     /// The style of a UTF-8 byte range within one editing line: its text
     /// properties (color, decoration, weight, ...) and `background`, as a
     /// [`Style`](crate::Style).
     #[derive(Clone, Debug, PartialEq)]
     pub struct StyledRange {
         pub range: std::ops::Range<usize>,
         pub style: crate::Style,
     }
     ```
     `lib.rs`・`facade.rs` の公開から `SpanStyle`・`TextDecoration`（style の）を消す（`StyledRange` は残る）。
  4. core：
     - `crates/kivact-core/src/view/span.rs` の `pub struct Span { .., style: SpanStyle }` を `style: kivact_style::Style` にし、各 setter の本体を Style の setter に委ねる：`font_size(v)` → `self.style = std::mem::take(&mut self.style).font_size(v); self`、`font_weight(v)` → `.font_weight(v)`、`color(c)` → `.color(c)`、`family`／`font_family`（今の名前のまま）→ `.font_family(v)`、`font_style(v)` → `.font_style(v)`、`letter_spacing`、`word_spacing`、`font_kerning`、`font_feature_settings`、`tabular_nums`、`text_decoration`（線）、`text_decoration_color`、`text_decoration_thickness`、`text_underline_offset`、`text_decoration_style`（今あれば。無ければ足す：span の装飾の style を設定できなかった BUG の公開の入口）、`background(p)` → `.background(p)`、`font_variation_settings`、`font_width`、`text_stroke`。span を compile する所（`compile_span_style(&span.style, theme)` の呼出し）を `kivact_style::internal::compile_span(&span.style, theme)` にし、`TextSpan { range, style, background }` を作る。`unsupported` の報告は今の `CompiledSpanStyle.unsupported` の報告の所でそのまま使う。
     - `crates/kivact-core/src/custom/text_editing.rs`：`StyledRange` の `color`・`background`・`decoration` を読んで line の paint を作っている 2 か所を、`compile_span(&r.style, &theme)` の結果から作る（`color` は `text.color`、`background` は `Option<Fill>` の solid の色（`Fill::Solid(c)` なら `c`、層なら最初の層の solid、それ以外は `None`）、装飾は手順 2 と同じ規則で `text` の decoration の field から作る）。
  5. 呼出し元（ui と試験）：`StyledRange { range, color: Some(c), background: Some(b), decoration: .. }` を `StyledRange { range, style: style().color(c).background(b).text_decoration(lines)... }` にする（2b8df5ac：`crates/kivact-ui/src/code_block.rs`、`crates/kivact-ui/src/code_editing.rs`、`dev/tests/kivact-public-api/tests/*/custom_text_field.rs`、`text_editing.rs`、`ui_chat_code.rs`。`rg -n "StyledRange \{|SpanStyle|TextDecoration \{" $W6_SCOPE $W6_TESTS`）。
- **消すもの**：`kivact_types::SpanProps`、style の `SpanStyle`・`CompiledSpanStyle`・`TextDecoration`（作者の装飾の値）・旧 `StyledRange` の field、`internal::{compile_span_style, compile_text_decoration}`、`text.rs` の `fn decoration`。
- **状態の閉じ方**：`TextSpan`・`StyledRange` は値の record（`&mut self` の method なし）。
- **試験**：
  - `crates/kivact-engine/src/lower/conversion.rs` に `#[cfg(test)] mod tests`（`//! Role: Tests of the span conversion.`）を作り、`fn a_span_keeps_its_decoration_style()` を足す：`TextSpan { range: 0..1, style: TextStyle { decoration_lines: Some(DecorationLines::UNDERLINE), decoration_style: Some(TextDecorationStyle::Wavy), ..TextStyle::default() }, background: None }` を手順 2 の関数で写した `ResolvedSpanStyle` の `decoration` が `Some` で、その `style` が `Wavy`（2b8df5ac では span の装飾は常に `Solid` だった）。
  - 既存の `text_editing.rs`・`custom_text_field.rs`・`ui_chat_code.rs` が通ること。
- **確認（tier a）**：鎖の共通確認と次：
  ```
  cargo clippy -p kivact-engine --all-targets -- -D warnings
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-engine
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- text_style
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- text_editing
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- custom_text_field
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_chat_code
  ```
- **完了条件**：`rg -n -w "SpanProps|SpanStyle|CompiledSpanStyle|compile_span_style|compile_text_decoration" crates dev --type rust` が `ResolvedSpanStyle` 以外で 0 件。上のコマンドが通る。

---

## S23：印と doc の後始末（ST-28、ST-29）

- **wave / lane / 前提**：W6b / 鎖 / S22
- **所有**：`crates/kivact-types/`、`crates/arvane-protocol/`、`crates/kivact-style/`、`crates/kivact-svg/`、`crates/kivact-shader/`（コメントと doc だけ）
- **目的**：ST-28 のうち、この付属書の crate に残る情報の無い印と定型の doc を消す。ST-29 の残りの doc の矛盾を直す。規則の変更（`state` の検査、`api-shape` が `Arc<[T]>` を受けること、`internal-contract` の重複）は付属書 T の W1a・W1b。
- **手順**：
  1. `rg -n "xtask-allow\(state-fields\)" crates/kivact-types crates/arvane-protocol crates/kivact-style crates/kivact-svg crates/kivact-shader` の行を消す（6.1 の新しい規則では field 単位の印は使わない。W1b の後に残ったもの）。
  2. `rg -n "xtask-allow\((api-shape|internal-contract)\)" crates/kivact-types crates/arvane-protocol crates/kivact-style` の各行を一行ずつ消し、`cargo xtask check --no-write --crate <crate>` が通ることを確かめる。通らなくなった行は戻す（付属書 T の規則がその場合を例外にしていない印は必要な印である）。
  3. 定型の doc を消す：`crates/kivact-types/src/color.rs` の `/// Rgba.`、`/// R.`、`/// G.`、`/// B.`、`/// A.`、`/// With alpha mul.` の行。field の doc を書き直す：`r`・`g`・`b` は `/// Red (green, blue) in 0..=1, straight alpha.` の形、`a` は `/// Opacity in 0..=1.`、`with_alpha_mul` は `/// The color with its alpha multiplied by \`factor\` (clamped to 0..=1).`。
  4. ST-29：`crates/kivact-style/src/theme.rs` の Role の後の文が S09 で直っていること、`crates/kivact-style/src/text.rs` の「`Draw`'s `TextFormat`」（S20）、`crates/kivact-style/src/sheet.rs` の module の doc の、setter の doc を繰り返すだけの段落（「Layout vocabulary:」以下の箇条のうち、各 setter の doc と同じことを書いた項）を消し、残す段落は「Interaction states」（S19 の文）と「Entry points that look alike but differ」だけにする。
  5. 設計書 5.12 の削除名 `TextExtras` を残さない：`rg -l -w "TextExtras" crates/kivact-style/src | xargs perl -pi -e 's/\bTextExtras\b/TextRareFields/g'`。その struct の doc を「The rarer text properties of a folded style, boxed so a style without them stays small.」にする。
- **消すもの**：上の印と定型の doc、`TextExtras` の名前。
- **状態の閉じ方**：なし。
- **試験**：なし。
- **確認（tier a）**：
  ```
  cargo fmt --all -- --check
  cargo xtask check --no-write --crate kivact-types
  cargo xtask check --no-write --crate arvane-protocol
  cargo xtask check --no-write --crate kivact-style
  cargo xtask check --no-write --crate kivact-svg
  cargo xtask check --no-write --crate kivact-shader
  cargo doc -p kivact-style --no-deps
  ```
- **完了条件**：`rg -n "xtask-allow\(state-fields\)" crates/kivact-types crates/arvane-protocol crates/kivact-style crates/kivact-svg crates/kivact-shader` が 0 件。上のコマンドが通る。

## 4. 他の付属書との接点

### 4.1 `ThemeVar` の API と、部品・kit の変数の宣言の型（付属書 U・K・G）

S15 の後の API（`kivact::style` の公開）：

```rust
// kivact-types（kivact::style が再公開）
pub struct ThemeVar<T: 'static> { /* name, fallback: VarValue<T> */ }
pub enum VarValue<T: 'static> { Value(T), Var(&'static ThemeVar<T>) }
impl<T: 'static> ThemeVar<T> {
    pub const fn new(name: &'static str, fallback: T) -> Self;                       // 値の fallback
    pub const fn alias(name: &'static str, target: &'static ThemeVar<T>) -> Self;    // fallback = target の解決した値
    pub const fn name(&self) -> &'static str;
    pub const fn fallback(&self) -> &VarValue<T>;
}
impl<T> From<&'static ThemeVar<T>> for VarValue<T>;  // set の値としての別名
// 値の From：VarValue<f64> ← f64/f32/i32/u32、VarValue<u16> ← u16、VarValue<LineHeight> ← LineHeight、
// VarValue<Color> ← Color、VarValue<FontFamily> ← FontFamily/&str/String
// PartialEq / Eq / Hash は同一性（static の address）。Debug は "var(--name)"。

impl Color  { pub const fn var(v: &'static ThemeVar<Color>) -> Color; }   // alpha は with_alpha で掛けられる
impl Length { pub const fn var(v: &'static ThemeVar<f64>) -> Length; }   // + と - の式（calc）に入る
pub fn em(v: impl Into<f64>) -> Length;                                    // style が設定する font の大きさ、無ければ font::SIZE の倍

// kivact-style
pub trait ThemeValue: Clone + PartialEq + Debug + Send + Sync + 'static { /* sealed */ }
// 実装：Color、f64、u16、LineHeight、FontFamily。すべて別名を持てる
impl Theme {
    pub fn set<T: ThemeValue>(self, var: &'static ThemeVar<T>, value: impl Into<VarValue<T>>) -> Theme;
    pub fn get<T: ThemeValue>(&self, var: &'static ThemeVar<T>) -> T;
    pub fn with_fonts(self, fonts: impl IntoIterator<Item = impl Into<Asset>>) -> Theme;   // S09
    pub fn fonts(&self) -> &[Asset];                                                      // S09
}
pub fn try_use_theme() -> Option<Theme>;                                  // S16
pub struct FontFamily; // FontFamily::SYSTEM, From<&str>, From<String>, as_str()
```

**別名の書き方は一つ**（親の決定）：fallback は `ThemeVar::alias(name, &TARGET_VAR)`、theme での設定は `theme.set(&VAR, &TARGET_VAR)`（値なら `theme.set(&VAR, value)`）。`set_alias` は作らない。色は `Color::var(&TARGET_VAR)` を値として置いても同じく別名として辿る（`Color` は値の中に変数を持てるため。新しい書き方を勧めるときは `&TARGET_VAR` の形）。

宣言の型（部品の module、kit、`kivact-plot` が同じ形で書く。**変数は必ず `static`**。`const` で宣言すると使う所ごとに別の address になり、theme の値が届かない）：

```rust
// kivact::ui::theme::button（付属書 U の部品の module）
use kivact::style::{Color, Length, ThemeVar, color, control, radius};

/// The height of a medium button: the theme's control height unless set.
pub static HEIGHT_MD_VAR: ThemeVar<f64> = ThemeVar::alias("button.height_md", &control::HEIGHT_VAR);
pub const HEIGHT_MD: Length = Length::var(&HEIGHT_MD_VAR);

/// A button's corner radius: the theme's medium radius unless set.
pub static RADIUS_VAR: ThemeVar<f64> = ThemeVar::alias("button.radius", &radius::MD_VAR);
pub const RADIUS: Length = Length::var(&RADIUS_VAR);

/// A button's fill; by default the theme's accent.
pub static FILL_VAR: ThemeVar<Color> = ThemeVar::alias("button.fill", &color::ACCENT_VAR);
pub const FILL: Color = Color::var(&FILL_VAR);

// kit（付属書 K）
pub static PRIMARY_VAR: ThemeVar<Color> = ThemeVar::new("basic.primary", rgb(0x171717));
pub const PRIMARY: Color = Color::var(&PRIMARY_VAR);
fn light() -> Theme {
    Theme::light()
        .set(&PRIMARY_VAR, rgb(0x171717))
        .set(&color::ACCENT_VAR, &PRIMARY_VAR) // alias: --accent: var(--primary)
}
// 12 章 PF-02：theme は thread_local! の static で一度だけ作る（cached_theme は無い）
thread_local! {
    static LIGHT: Theme = light();
    static DARK: Theme = dark();
}
pub fn theme(mode: ThemeMode) -> Theme {
    match mode {
        ThemeMode::Light => LIGHT.with(Theme::clone),
        ThemeMode::Dark => DARK.with(Theme::clone),
    }
}
```

規則：
- 名前は `"<領域>.<名前>"`（中核の色だけは DevTools の token と同じ短い名前。4.2 節）。名前は診断と Debug にだけ使い、同一性は address。
- **すべての型の変数**（`Color`・`f64`・`u16`・`LineHeight`・`FontFamily`）が、値としても fallback としても別の同じ型の変数を指せる（12 章 UI-U-02）。8 段まで辿り、循環は最初の変数の fallback（値なら）を使い、`install_theme`（`App::theme` と `provide_theme` が呼ぶ）が名前ごとに一度 `ErrorKind::Unsupported` を報告する。
- 部品と kit の変数の既定は、意味の同じ中核の変数への別名にする（`ThemeVar::alias(.., &radius::MD_VAR)`）。共通の値（`theme.set(&radius::MD_VAR, 20)`）がすべての部品に届き、部品の変数を set すればそれが勝つ。中核に同じ意味の変数が無い値だけ `ThemeVar::new(.., value)`。
- 値の形（`const`）を持つのは `Color` と `Length`（`f64` の変数）。`u16`（font の太さ）・`LineHeight`・`FontFamily`・不透明度（`control::DISABLED_OPACITY_VAR`）の変数は `static` だけで、recipe（`Fn(&Theme, &Look<S>, Style) -> Style`。付属書 U）が `theme.get(..)` で読む。
- style の setter のうち変数をそのまま受けるもの：色の引数（`impl Into<Color>`）すべて、`Length` の引数（`width`・`padding`・`gap`・`top`・`font_size`・`translate_x`・`letter_spacing` など `impl Into<Length>`）すべて、`border_radius`（`impl Into<CornerRadius>`、`Length` から変換）。`Length` の `+`・`-` は変数・`em`・`pct`・px の式として保ち theme ごとに解決し直す（`space::MD + 2`）。受けられない式（`auto` や内容の大きさとの和、二つの別の変数）は `Unsupported` を報告する（12 章 UI-S-03）。`f64` の引数（`opacity`、`flex_grow`、`border` の幅、`outline` の幅）は変数を受けない（CSS の `<number>`。部品は recipe で `theme.get(..)` を読む）。

### 4.2 中核の theme 変数（S15。`kivact::style::{color, space, radius, font, control}`）

fallback は 2b8df5ac の `Theme::light()` の値、dark は `Theme::dark()` が set する値（空欄は light と同じで set しない）。

| module::定数（`static` は `<定数>_VAR`） | 型 | 名前 | fallback（light） | dark |
|---|---|---|---|---|
| `color::SURFACE` | Color | `surface` | `rgb(0xffffff)` | `rgb(0x111827)` |
| `color::SURFACE_ALT` | Color | `surface_alt` | `rgb(0xf3f4f6)` | `rgb(0x1f2937)` |
| `color::TEXT` | Color | `text` | `rgb(0x111827)` | `rgb(0xf9fafb)` |
| `color::TEXT_MUTED` | Color | `text_muted` | `rgb(0x6b7280)` | `rgb(0x9ca3af)` |
| `color::ACCENT` | Color | `accent` | `rgb(0x2563eb)` | `rgb(0x3b82f6)` |
| `color::ACCENT_TEXT` | Color | `accent_text` | `rgb(0xffffff)` | `rgb(0xffffff)` |
| `color::BORDER` | Color | `border` | `rgb(0xd1d5db)` | `rgb(0x374151)` |
| `color::FOCUS` | Color | `focus` | `rgb(0x2563eb)` | `rgb(0x60a5fa)` |
| `color::DANGER` | Color | `danger` | `rgb(0xdc2626)` | `rgb(0xf87171)` |
| `color::SUCCESS` | Color | `success` | `rgb(0x16a34a)` | `rgb(0x4ade80)` |
| `color::WARNING` | Color | `warning` | `rgb(0xd97706)` | `rgb(0xfbbf24)` |
| `color::OVERLAY` | Color | `overlay` | `rgba(0x000000, 0.35)` | `rgba(0x000000, 0.5)` |
| `color::SHADOW` | Color | `shadow` | `rgba(0x000000, 0.2)` | `rgba(0x000000, 0.5)` |
| `color::SELECTION` | Color | `selection` | `rgba(0x2563eb, 0.15)` | `rgba(0x3b82f6, 0.3)` |
| `color::HOVER` | Color | `hover` | `rgba(0x111827, 0.05)` | `rgba(0xf9fafb, 0.075)` |
| `color::PRESSED` | Color | `pressed` | `rgba(0x111827, 0.10)` | `rgba(0xf9fafb, 0.15)` |
| `color::RAISED` | Color | `raised` | `rgb(0xffffff)` | `rgb(0x1f2937)` |
| `space::XS` | Length（`f64`） | `space.xs` | 2 | |
| `space::SM` | Length | `space.sm` | 4 | |
| `space::MD` | Length | `space.md` | 8 | |
| `space::LG` | Length | `space.lg` | 12 | |
| `space::XL` | Length | `space.xl` | 16 | |
| `radius::SM` | Length | `radius.sm` | 3 | |
| `radius::MD` | Length | `radius.md` | 6 | |
| `radius::LG` | Length | `radius.lg` | 10 | |
| `font::SIZE` | Length | `font.size` | 14 | |
| `font::SIZE_SMALL` | Length | `font.size_small` | 12 | |
| `font::SIZE_LARGE` | Length | `font.size_large` | 18 | |
| `font::WEIGHT_VAR`（static だけ） | `u16` | `font.weight` | 400 | |
| `font::WEIGHT_STRONG_VAR`（static だけ） | `u16` | `font.weight_strong` | 600 | |
| `font::LINE_HEIGHT_VAR`（static だけ） | `LineHeight` | `font.line_height` | `LineHeight::Scale(1.35)` | |
| `font::FAMILY_VAR`（static だけ） | `FontFamily` | `font.family` | `FontFamily::SYSTEM` | |
| `control::HEIGHT` | Length（`f64`） | `control.height` | 28 | |
| `control::OVERLAY_MARGIN` | Length（`f64`） | `control.overlay_margin` | 0 | |
| `control::DISABLED_OPACITY_VAR`（static だけ） | `f64` | `control.disabled_opacity` | 0.5 | |

`Theme` に field として残るもの（変数にしない。builder と accessor は今のまま）：`mode`（`ThemeMode`）、`focus_ring`、`focus_ring_offset`（focus の ring は style の一部。親の決定）、`icons`、`fonts`、拡張（`with`／`ext`。recipe の表と `InteractionPolicy` だけに使う）。

### 4.3 旧い名前から theme 変数への対応表（付属書 U・K・G が使う）

| 旧（2b8df5ac） | 新（S15 の後） |
|---|---|
| `ColorToken::Surface`、`Color::token(ColorToken::Surface)`、`theme.colors().surface` を値として | `color::SURFACE` |
| `ColorToken::SurfaceAlt` / `TextMuted` / `AccentText` | `color::SURFACE_ALT` / `color::TEXT_MUTED` / `color::ACCENT_TEXT`（CamelCase を大文字の snake case に） |
| `ColorToken::<X>`（ほかの 14 個） | `color::<X の大文字>`（`Text`→`TEXT`、`Accent`→`ACCENT`、`Border`→`BORDER`、`Focus`→`FOCUS`、`Danger`→`DANGER`、`Success`→`SUCCESS`、`Warning`→`WARNING`、`Overlay`→`OVERLAY`、`Shadow`→`SHADOW`、`Selection`→`SELECTION`、`Hover`→`HOVER`、`Pressed`→`PRESSED`、`Raised`→`RAISED`、`Surface`→`SURFACE`） |
| `theme.colors().<field>`（`Color` の値が要る所） | `theme.get(&color::<FIELD>_VAR)` |
| `theme.colors().get(token)` | `theme.get(var)`（token の代わりに `&'static ThemeVar<Color>` を持つ） |
| `theme.space().xs` … `xl`（f64） | `theme.get(&space::XS_VAR)` …（f64）。style には `space::XS` をそのまま渡す |
| `theme.radius().sm` … `lg` | `theme.get(&radius::SM_VAR)` … / `radius::SM` |
| `theme.text().size` / `size_small` / `size_large` | `theme.get(&font::SIZE_VAR)` … / `font::SIZE` … |
| `theme.text().weight` / `weight_strong` | `theme.get(&font::WEIGHT_VAR)` / `theme.get(&font::WEIGHT_STRONG_VAR)` |
| `theme.text().line_height` | `theme.get(&font::LINE_HEIGHT_VAR)` |
| `theme.text().family`（`Option<String>`） | `theme.get(&font::FAMILY_VAR).as_str()` |
| `theme.with_colors(Palette { accent: X, .. })` | `theme.set(&color::ACCENT_VAR, X)`（変えた field ごと） |
| `theme.with_space(..)` / `with_radius(..)` / `with_text(Typography { .. })` | `.set(&space::.._VAR, ..)` / `.set(&radius::.._VAR, ..)` / `.set(&font::.._VAR, ..)` |
| `use_tokens::<T>()` | `use_theme().with(Theme::ext::<T>)`（recipe の表だけ。値の token は変数にする：付属書 U・K） |
| `impl ThemeColor for ColorName`（kit） | kit の `pub static X_VAR: ThemeVar<Color>` と `pub const X: Color`（`ColorName` は消す。付属書 K） |
| `Color::of(x)` | `x`（`Color`）か `Color::var(&X_VAR)` |
| `Color::components()`（theme なし） / `components_in(theme)` | `components(&theme)` |
| `Color::mix(other, t)`（theme なし） / `mix_in(other, t, theme)` | `mix(other, t, &theme)` |
| `Color::resolve(theme)`（style の外） | core：`kivact_style::internal::resolve_color(c, theme)`、それ以外：`c.components(theme)` |
| `theme.control_height()` / `.with_control_height(x)` | `theme.get(&control::HEIGHT_VAR)` / `.set(&control::HEIGHT_VAR, x)`。style には `control::HEIGHT` |
| `theme.disabled_opacity()` / `.with_disabled_opacity(x)` | `theme.get(&control::DISABLED_OPACITY_VAR)` / `.set(&control::DISABLED_OPACITY_VAR, x)` |
| `theme.overlay_margin()` / `.with_overlay_margin(x)` | `theme.get(&control::OVERLAY_MARGIN_VAR)` / `.set(&control::OVERLAY_MARGIN_VAR, x)`。style には `control::OVERLAY_MARGIN` |
| `UiTokens` の `Option<f64>` の token で `None` のとき中核の値を使っていたもの（`tokens.field_radius.unwrap_or(t.radius().md)`） | 部品の変数を中核の変数の別名で宣言する（`ThemeVar::alias("combobox.field_radius", &radius::MD_VAR)`。4.1 節。12 章 UI-U-02） |
| `cached_theme(build)`（2b8df5ac には無い。旧版の本付属書の案） | kit の `thread_local! { static LIGHT: Theme = light(); }` と `LIGHT.with(Theme::clone)`（12 章 PF-02） |
| `kivact::platform::ThemeHint` | `kivact::style::ThemeMode`（S13） |
| `kivact::geometry::Insets` | `kivact::geometry::Edges<f64>`（S13） |

W6 の S15 は、ui・core・devtools の呼出しを上の表で **同じ型の読み** に機械的に置き換える（closure は残る）。closure を消して `style().padding(space::MD)` の形にするのは付属書 U（ui）・K（kit）。

### 4.4 部品の token と recipe（付属書 U への申し送り）

- `UiTokens` の値の field は、部品の module の theme 変数（4.1 節の `HEIGHT_MD` の形）にする（DEC-06）。fallback は 2b8df5ac の `UiTokens::default()`（または `fallback(theme)`）が作る値。`fallback(theme)` が palette から作っていた値は、fallback に中核の変数を置く（`ThemeVar::new("x", color::BORDER)`）。
- recipe の上書き `Fn(&Theme, &Look<S>, Style) -> Style` の中の値の読みは `theme.get(..)`。
- `Theme::ext` は表だけ（recipe、`InteractionPolicy`）。S09 で、拡張が無い theme の fallback は theme の値ごとに一度だけ作られ、theme のどの書込み（`set`・`with`・builder）でも作り直される（12 章 UI-S-01）。
- 部品の token のうち、旧い `UiTokens` で `None` のとき theme の値（`t.radius().md` など）を使っていたものは、その中核の変数への別名で宣言する（4.1 節。値の数に固定しない。12 章 UI-U-02）。
- 付属書 K への申し送り（12 章 PF-02）：kit の `theme(mode)` は `thread_local!` の static（4.1 節の例）で theme を一度だけ作る。`cached_theme` は無い。

### 4.5 setter の名前（S17・S20）と AC-4 の検査（付属書 G・U・K・作業指示書）

| 旧 | 新 |
|---|---|
| `.weight(u16)` | `.font_weight(u16)`（`Span` も） |
| `.italic(bool)`（Style）、`Span::italic()` | `.font_style(FontStyle)`（`FontStyle::{Normal, Italic}`、`kivact::style::FontStyle`） |
| `.radius(r)` | `.border_radius(r)` |
| `.radius_corners(tl, tr, br, bl)` | `.border_radius_corners(tl, tr, br, bl)` |
| `.radius_top_left(r)` / `top_right` / `bottom_right` / `bottom_left` | `.border_top_left_radius(r)` / `border_top_right_radius` / `border_bottom_right_radius` / `border_bottom_left_radius` |
| `.ring(w, c)` | `.outline(w, c)` |
| `.ring_with_offset(w, o, c)` | `.outline(w, c).outline_offset(o)` |
| `.ring_none()` | `.outline_none()` |
| （新） | `.outline_width(w)`、`.outline_color(c)`、`.outline_offset(o)` |
| `.basis(l)` / `.grow(f)` / `.shrink(f)` | `.flex_basis(l)` / `.flex_grow(f)` / `.flex_shrink(f)` |
| `.wrap(bool)`（Style） | `.flex_wrap(bool)` |
| `.direction(d)`、型 `Direction` | `.flex_direction(d)`、型 `FlexDirection` |
| `.pressed(style)` | `.active(style)` |
| `.font_size(f64)` | `.font_size(impl Into<Length>)`（数は今まで通り px。`font::SIZE` を受ける） |
| `.text_decoration_style(impl IntoValue<_>)` | `.text_decoration_style(TextDecorationStyle)`（reactive は `.style(move || ..)`） |
| `ScrollbarWidth::Thin` | `ScrollbarWidth::Auto`（同じ 6 px） |
| `Layer::new().blend(m)` / `.composite(op)` | `Layer::new().mix(Mix::Blend(m))` / `.mix(Mix::Composite(op))`（S21） |

**AC-4 の旧 setter の検査について**（解決済み。INC-37）：AC-4 の検査のコマンドは作業指示書 2 章だけにある（付属書 T 4.2 と INC-13 の解決から作る。旧い setter は `kivact-style` に定義が無いことを tier c で compile して確かめる形）。本付属書は検査を持たない。2b8df5ac で W6 の範囲に残る Style でない同名の method（`Gradient::radius(r)`、`CodeBlock::wrap`、`TextInput::wrap`、`TextEditApi::wrap`、`RowRender::wrap`、`Grid::direction()`、`switch_drag::Axis::direction()`、`GpuContext::shrink`、engine の `Track::pressed()`）は対象外。`TextInput::wrap`・`CodeBlock::wrap` は付属書 U で props の `wrap` のまま。`Span` の `weight`・`italic` は S20 で CSS の名前にする。kit の同名の method は付属書 K。

### 4.6 `kivact::draw` と `kivact::style` の分担（S20・S21。付属書 C・G・U・K）

- `kivact::style` に移るもの：`Shadow`、`Filter`、`Transform`、`Paint`、`Gradient`、`CanvasGradient`、`GradientExtend`、`RadialExtent`、`linear_gradient`、`radial_gradient`、`conic_gradient`、`Shape`、`CornerRadius`、`BlendMode`、`Mask`、`MaskLayer`、`MaskMode`、`MaskComposite`、`BorderImage`、`BorderImageRepeat`、`BackgroundClip`、`BackgroundOrigin`、`PaintShader`、`FilterShader`、`ShaderPaint`、`ShaderFilter`、`ShaderValue`、`FilterGraph`、`FilterInput`、module `filter`、`FontStyle`（新）、`FlexDirection`（`Direction` の改名）、`ThemeVar`・`VarValue`・`ThemeValue`・`FontFamily`・`try_use_theme`・module `color`・`control`・`space`・`radius`・`font`（新）。
- `kivact::draw` に残るもの：`Draw`、`Path`、`PathContour`、`Layer`、`Mix`、`Composite`（`Mix::Composite` の中身。設計書 5.8 の一覧に無いが、`Mix::Composite(Composite::DestinationOut)` を書くのに要る）、`Stroke`、`LineCap`、`LineJoin`、`FillRule`、`TextFormat`、`TextBaseline`、`TextMetrics`、`TextParagraph`、`InlineParagraph`、`InlineBox`、`InlineBoxPlacement`、`ImageOptions`、`Pixels`、`RegionId`、`Offscreen`。
- `kivact::view` に移るもの：`IntrinsicSize`（`node().intrinsic_size(..)` の値）。
- 公開から消えるもの：`kivact::draw::{TextRun, text_run}`（engine の wire の型）。core の内部は `kivact_style::draw::text_run` を使い続ける。

### 4.7 付属書 C への申し送り

- **prelude**（設計書 5.11）：`color`・`space`・`radius`・`font` の module は W6 の S15 で生まれる。`control` の module（S15）は prelude に入れない（`kivact::style::control` から import する）。付属書 C が W5 で prelude を書くときは、この 4 つを除いた 5.11 の一覧を書き、S15 が足す。`ColorToken`・`ThemeColor`・`use_tokens` は W5 で prelude から外してよい（名前は S15 まで `kivact::style` に残る）。`Transform` は 5.11 に無い（S20 が prelude から消す）。`Shadow` は S20 の後 `kivact::style::Shadow`。
- **font の登録**（USR-12・12 章 API-13）：S09 が `Theme::with_fonts(impl IntoIterator<Item = impl Into<Asset>>)` と `Theme::fonts() -> &[Asset]` を W2a で足す（path・`Vec<u8>`・共有 bytes・`include_bytes!` を同じ入口で受ける。`.leak()` は要らない）。登録済みかどうかは `Asset` の同一性（`PartialEq`：static と共有 bytes は address と allocation、path は値）で判定し、同じ font を二度読まない。`App::theme(t)` と `RenderOptions::with_theme(t)` は `t.fonts()` を登録する（`App::fonts`・`TestFonts::Only` の削除は付属書 C）。`provide_theme` で後から切り替えた theme の font の登録も付属書 C（`install_theme` と同じ場所）。
- **style の内部の橋**（`kivact_style::internal`）の S12・S16・S18・S22 の後の形：`record(size, RecordContext { theme, shaper, regions }, f) -> Recording`、`compile_style(s: Style /*値で*/, theme, Interactive { focusable, disabled, selected, root, forced: StateSet, active: StateSet }) -> CompiledStyle`、`resolve_color(color, theme) -> Rgba`、`compile_span(&Style, &Theme) -> (TextStyle, Option<Fill>, u32)`、`describe_style`・`describe_state_styles`、`install_theme`・`theme_of`、`with_scroll_axes`。`ForcedStates` は `kivact_types::StateSet` になる（S18）。
- **Offscreen の crate 化**（RC-29）：付属書 C の C02（W2b）が `crates/kivact-core/src/draw/offscreen.rs` を `crates/kivact-offscreen/src/lib.rs` に移している。S12（W3a）はその場所を直す。S15・S16 も同じ path（`crates/kivact-offscreen/src/lib.rs` の `self.theme.text().family` など。付属書 C の S-REQ-4）を直す。
- **ST-23（`prefer_gpu` と `GpuStart::Never`）**：付属書 A の A05（W5 の鎖）が実装する。`crates/kivact-types/src/{embed.rs, gpu.rs}` の変更は A05 の所有。本付属書に unit は無い。
- **ST-21 の `embed::Visibility` → `EmbedVisibility`**：設計書 5.6 のとおり付属書 C（API-10・USR-15）。本付属書は扱わない。
- **`platform::theme_hint()`**：S13 で戻り値の型が `Value<ThemeMode>` になる（関数名は変えない）。

### 4.8 `TypeMap`（S10。付属書 U・C）

`kivact_types::{SharedTypeMap, LocalTypeMap}`：`new()`、`with<T>(self, value: T) -> Self`（同じ型を置き換える）、`get<T>(&self) -> Option<&T>`、`get_shared<T>(&self) -> Option<Arc<T>>`（Local は `Rc<T>`）、`is_empty()`、値の `PartialEq`（共有の表は ptr で即座に等しい）と `Debug`。theme の拡張は S10 で `SharedTypeMap` に移る。部品の variant の表（2b8df5ac の `crates/kivact-ui/src/variants.rs` の `Extension`・`Extensions`。設計書 5.4 で node の `Option<Box<Variants>>` に移る）は、それを移す付属書（C の `Element::variant`、または U）が `LocalTypeMap` で書く。`Variant: PartialEq + Debug + 'static` の値を `with` で入れる。

### 4.9 `Mix` と `StateSet`・`StyleState`（付属書 E・A・C）

- `kivact_types::Mix { Normal, Blend(BlendMode), Composite(Composite) }`（S03）。`DrawingLayer.mix` は正規化済み（`Blend(Normal)` と `Composite(SourceOver)` は来ない）。protocol の `LayerEffects` は今の `blend` と `composite` の 2 field のまま（engine が `mix.blend_mode()` と `mix.composite()` で写す）。付属書 A が `LayerEffects` を `mix` にする場合は、engine の `lower/scene/layers.rs` の 2 行を同じ unit で直す。
- `kivact_types::{StyleState, StateSet}`（S04）。`StateStyles { deltas: [Option<Box<PaintDelta>>; 8], group, report }`、`delta(state)`、`set_delta`、`wants(state)`、`deltas()`。`HostEvent::StatesChanged { node, states }`（engine → core）。engine の `Model::reported_states` と `report_states(id)`。付属書 E が engine の状態の評価（`model/states.rs` の `deltas`、hover・focus の変化の経路）を変えるときは、`report_states(id)` の呼出し（`restyled`、`set_states`、focus の変化、pointer の flipped）を保つ。

### 4.10 W6 の鎖が触る engine・layout・editing・arvane のファイル（付属書 E・A）

作業指示書 4 章（INC-28）：S13・S14 は W6a に単独で置かれ、W6a の間 S14 が engine・arvane・layout・editing・gpu-share の全体を所有する（E・A の lane は W6a に無い）。W6b では E19・E21・E24・A11・A12 が並行するが、次の S15・S22 のファイルとは重ならない（E20 は W5a、E22・E23 は W7a に移された）。

| unit | ファイル |
|---|---|
| S13 | `crates/kivact-layout/src/{columns.rs, image.rs, layout/tests.rs, style/convert.rs, style/node.rs, tree.rs}`、`crates/kivact-engine/src/{model/interact/sticky.rs, model/presence/transitions.rs, host/testing.rs, host/testing/test_host.rs, host/testing/test_options.rs, host/launch.rs, model/frame/state.rs, contract/devtools.rs, lower/glyphs.rs, measure/leaf.rs}`、`crates/arvane-text/src/cache.rs`、`crates/arvane-shell/src/events.rs`、`crates/arvane-native/src/{port/winit/window.rs, native_input/windows.rs}`、`crates/arvane-virtual/src/{script.rs, window.rs, backend.rs}`、`crates/arvane/tests/e2e/{environment.rs, text_style.rs}`（`Dimension`・`TextLen`・`GridFlow`・`Insets`・`ThemeHint`・`AvailableSpace` を名指す行。正確な一覧は S13 の手順の `rg -l` の結果） |
| S14 | `crates/kivact-engine/`、`crates/arvane*/`、`crates/kivact-layout/`、`crates/kivact-editing/`、`crates/kivact-gpu-share/` のうち、`Transform`・`Gradient`・`FilterGraph`（kivact_types の）・`scene::Shape`・`motion::Kind` を名指すファイル（`rg -l` の結果。型の名前の置換だけ） |
| S15 | `crates/kivact-layout/src/style/convert.rs`（`Length::Var` の腕） |
| S22 | `crates/kivact-engine/src/{lower/conversion.rs, measure/leaf.rs, text_input/shaping.rs, host/write/typed.rs, tree/node.rs}` |

W2 の lane A（S02〜S06）は kivact-types・engine・arvane-*・svg の全体を所有する（付属書 E・A の lane は W2 に無い）。

### 4.11 std の遅延初期化の型の利用者（DEC-13。各付属書が自分の unit で `Deferred`・`SharedDeferred` に置き換える）

2b8df5ac の利用：`crates/kivact-runtime/src/reactive/{data.rs, runtime.rs}`・`task/{spawn.rs, state.rs}`（付属書 R）、`crates/arvane-native/src/{mirror.rs, module.rs, native_input/windows.rs, port/x11/dnd.rs, port/winit/foreign/win32.rs}`、`crates/arvane-virtual/src/window.rs`（付属書 A）、`crates/kivact-ui/src/recipes.rs`（付属書 U）、5 つの kit の `recipes.rs`・`icons.rs`、`examples/kivact-ui-basic/src/docs.rs`（付属書 K）。置換の形：`static X: OnceLock<T> = OnceLock::new();` → `static X: SharedDeferred<T> = SharedDeferred::new();`（`#[expect(clippy::disallowed_types, ..)]` を消す）、`OnceCell<T>` の field → `Deferred<T>`。`OnceLock::set`／`OnceCell::set` を使う所（後から値を入れる）は S-7 に当たるので、その付属書が constructor で値を渡す形にする（`Deferred` に `set` は無い）。

### 4.12 `Color` の表現（brief の確認事項）

`Color(Repr)`、`enum Repr { Rgba(f64, f64, f64, f64), Var(&'static ThemeVar<Color>, f64), Space(SpaceColor) }`（S15）。`Copy`：三つの variant の中身はすべて `Copy`（`&'static` の参照を含む）。alpha は `f64`（2b8df5ac の `Token` と同じ精度。`f32` にしても `Rgba`・`Space` の variant が大きさを決めるので `Color` は小さくならない）。`PartialEq`：derive。`Var` は変数の同一性（`ThemeVar` の `PartialEq` は `std::ptr::eq`）と alpha の比較、`Rgba`・`Space` は値の比較。`Var` と `Rgba` は、同じ色に解決しても等しくない（theme で変わりうるため）。`Hash`：実装しない（f64 を持つ。2b8df5ac と同じ）。`ThemeVar` 自身は `Hash`（address）を持つ。`Color` を含む `ThemeVar<Color>` を `static` にできるのは、`Color: Sync`（参照先の `ThemeVar<Color>` も `Sync`）であるため。

### 4.13 設計判断の結果（親が決定済み）

1. （解決済み）AC-4 の旧 setter の名前の検査は作業指示書 2 章（INC-37。4.5 節）。
2. 決定済み（採用）：**`TextFormat`（`Draw` の文字の書式）を ST-11 の「`style: TextStyle` を持つ形」にしない**。理由：`TextStyle` は解決した値（`Rgba`）で、`TextFormat` は作者の値（`Color`、theme 変数を含む）の FRU の公開 struct。設計書 7.4 の ST-10 の判定（作者の値と解決した値の組は別の事実として残す）と同じ理由で、作者の書式に解決した型を埋め込まない。S22 は span と編集の range を一つの語彙（作者は `Style`、解決は `TextStyle`）にし、span の装飾の style の不具合を直す。推奨：この形で確定。
3. 決定済み（採用）：**`Typography` を全部消す**（設計書 5.7 は「`Typography` の大きさ」を消すと書く）。family・太さ・行の高さも `font::*` の変数にした（`with_text` が消えると残りの field を設定する道が無く、変数の方が一つの書き方になるため）。`Theme::text()` も消える。推奨：この形で確定。
4. `Length` の `em`（12 章 UI-S-03）：layout・塗りの長さの `em` は、その style が設定する `font_size` の倍、設定しなければ theme の `font::SIZE` の倍（compile の時点で解決する。先祖の style の font の大きさは継がない）。`font_size(em(x))` は `font::SIZE` の倍。文字の長さ（`letter_spacing` など）の `em` は従来どおり engine が文字の font の大きさで解決する（wire の `TextLength::Em`）。

## 5. 自己点検

- **採用・変更して採用の全所見がどれかの unit にある**：ST-01（S15）、ST-02（S15）、ST-03（S09・S16）、ST-04（S15）、ST-05（S04・S19）、ST-06（S18）、ST-07（S09）、ST-08（S17・S18）、ST-09（S04・S18）、ST-10（S13。同一の組の 6 つだけ）、ST-11（S06・S22。`TextFormat` は 4.13 の 2）、ST-12（S17）、ST-13（S05）、ST-14（S05）、ST-15（S11）、ST-16（S05）、ST-17（S03・S21）、ST-18（S12）、ST-19（S13）、ST-20（S02。`kivact-cell` は作らない）、ST-21（S06・S14。`EmbedVisibility` は付属書 C、4.7 節）、ST-22（S20。`Color::resolve` の非公開は S16）、ST-23（付属書 A の A05、4.7 節）、ST-25（S07）、ST-26（S09）、ST-27（S10。ui 側は 4.8 節）、ST-28（S23。規則の変更は付属書 T）、ST-29（S09・S20・S23）、ST-30（S08・S17・S20）、ST-31（S18）、DEC-13（S01）。ST-24 は不採用（unit なし）。
- **設計書 5.7・5.8 の契約**：`ThemeVar<T>`（S15）、`Color::var`・`Length::var`（S15）、`Theme::set/get`（S15）、中核の `color`・`space`・`radius`・`font`（S15）、全状態が全 property を受ける（S19）、CSS の名前（S20）、CSS の値の型は `kivact::style` だけ（S20）、`Theme(Arc)`（S09）、`Theme::with_fonts`（S09）、`Theme::with_icons`（2b8df5ac に既にある。変更なし）、`cached_theme` は作らない（12 章 PF-02。4.4 節）、`Theme` の一つの編集の入口（S09。UI-S-01）、すべての型の別名（S15。UI-U-02）、`Length` の式（S15。UI-S-03）、状態の paint callback（S17・S18・S19。UI-S-02）、`kivact::style::control`（S15）、`components(&Theme)`・`mix(.., &Theme)`（S16）、`relative_luminance`・`contrast_ratio`（S09）、`ThemeMode` を types に（S13）、`kivact::draw` は描画者の型（S20・S21）、`Mix`（S03・S21）、`Pixels` の premultiplied の doc は付属書 A（AR-10）、SVG の方針 DEC-11（S11）、`text_decoration_style(v: TextDecorationStyle)`（S17）。5.12 の削除名のうち本付属書が消すもの：`ThemeColor`、`ColorToken`、`Palette`、`Spacing`、`Radii`、`use_tokens`（S15）、`ThemeHint`、`Dimension`、`TextLen`、`GridFlow`、`Insets`（S13）、`SpanStyle`（S22）、`TextExtras`（S17 で作者の構造ではなくなり、畳んだ record の中の private な型になる。S23 の手順 5 で `TextRareFields` に改名する）、`DecorationStyleValue`（S17）、`Composite` を `Layer::composite` として（S21）、旧 style setter（S20）。
- **unit に設計の判断を残していない**：4.13 節の 3 点は親が決定済み。`em` の基準（style が設定する font の大きさ、無ければ `font::SIZE`）は S15 に書いた。各 unit の「手順」は名前・署名・コード・コマンドを与え、compile の error に応じる手直しは規則（どの error に何をするか）で与えた。
- **同じ wave の二つの unit が同じファイルを所有しない**：W1a は S01 だけ。W2 は lane A（S02〜S06。順番に実行）と lane S（S07〜S09。順番に実行）で、lane S は kivact-types・engine・arvane・svg のファイルを持たない（S07 の `kivact-shader`、`kivact-style` の shader、core・kivact の `Cargo.toml`、e2e の `shaders.rs`；S08 の style の effect と icon；S09 の style の theme）。lane A が持つ style のファイルは `draw.rs`・`draw/layer.rs`（S02・S03）と `resolve/compile.rs`（S04）で、lane S の unit はこれらを持たない。lane A が持つ core のファイル（`view/drawable/events.rs`、`view/drawable_share.rs`、`launch/runtime.rs`、S04 の手順 10 の `match`）と、S02 の root `Cargo.toml`・`workspace.toml`、S07 の core・kivact の `Cargo.toml` は、付属書 C の C01・C02 と重なるので、C01・C02 は W2b に置かれた（作業指示書 4 章。INC-23）。W3a の S の lane は S10〜S12（順番）。W6a は S13・S14、W6b は S15〜S23 と C19 の鎖（順番）。
- **新しい状態の型の閉じ方**：`StateStyles`（S04：印の 1 行）、`FilterGraph`（S08：印の 1 行）、`ThemeData`（S09：対象外、印の文面も用意）、`Groups`（S17：印の 1 行）、`Draw`（S12：印の 1 行）、`Mix`・`StyleState`・`Units`・`FontStyle`（enum）、`StateSet`・`TypeMap`・`VarTable`・`FontFamily`・`ThemeVar`・`Style`（field 一つか constructor だけが書く）。
- **unit の大きさ**：手で書くコードが 600 行を超えうるのは S17（表の行と shorthand。表の行は機械的な転記）と S15（変数の module と `vars.rs`、devtools の表）。どちらも残りは perl の置換。

