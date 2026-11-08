# Kivact 表現力パリティ 設計書

版: 1.5（2026-11-07。四回のレビュー `20261107_1305_CODEX_REVIEW.md`、`20261107_1333_CODEX_REVIEW.md`、`20261107_1402_CODEX_REVIEW.md`、`20261107_1425_CODEX_REVIEW.md` と、実装の中断の報告 `VISUAL_PARITY_STATUS.md` を反映）。対象: `main` @ f4e40cc。

本書は、Kivact の描画・アニメーション・HTML-in-Canvas 相当の表現力を Web プラットフォームと同等にするための設計である。実装は別の AI がサブエージェントを用いて行う。実装者は本書と `VISUAL_PARITY_WORK_ORDER.md`（以下「作業指示書」）だけを根拠に作業し、追加の設計や探索をしない。

---

## 0. 読み方と規則

### 0.1 優先順位

1. 本書
2. 作業指示書
3. 既存の `docs/for-kivact-developer/*.md`、`CONTRIBUTING.md`

既存文書と本書が矛盾する場合は本書に従い、既存文書は作業指示書の該当単位で本書に合わせて書き換える。

### 0.2 本書と実コードがずれていたとき

本書は `main` @ f4e40cc の調査に基づく。実コードとのずれは次の規則で処理する。設計判断をし直さない。

| ずれ | 処理 |
|---|---|
| 行番号、private な helper の名前、引数の並びが違う | 実コードに合わせる。意味は本書のとおり |
| 本書が「既存」と書いた型・関数が見つからない | 同じ意味の既存のものを使う。無ければ本書の定義どおりに作る |
| 本書の指定どおりにするとコンパイルできない（借用、trait bound など） | 意味を変えない最小の型調整（`Clone` の追加、`&` の付け外し、`Arc` 化）をする |
| 意味の衝突（本書の二つの節が矛盾する、既存の不変条件と衝突する） | オーケストレータが付録 A「既定の裁定」に従って決め、PR 説明の草稿に 1 行記録する |

### 0.3 共通の禁止事項

- 互換性のためのコードを書かない。旧名の alias、deprecated、二重経路、移行用 flag を作らない。変更した公開 API の利用箇所は、workspace 内（kit、example、template、試験、文書）をすべて同じ作業で移行する。
- CI 機能を追加しない（`.github/` を変更しない）。
- 新しい暗黙の状態機械を作らない（§13）。新しい state は本書が定義した enum だけを使う。
- 公開 API にバックエンドの型（vello、peniko、kurbo、wgpu、naga、resvg、tiny-skia、parley）を出さない。
- **コードの構造の規則（`cargo xtask check` が検査する。`dev/xtask/policy/workspace.toml` の `[style]` と module の検査）:** 新しく書く code と、本書が変える module は、次を守る。本書の module の一覧（§8.1 など）は、この規則に合わせてある。
  - **non-leaf の file:** 子 module を持つ file（`foo.rs` と `foo/` が並ぶもの、`lib.rs`）は、`mod` の宣言、`use`、再公開だけを持つ。型、関数、`impl`、定数を置かない。既存の leaf の module に子 module を足すときは、親の中身を `foo/<役割の名前>.rs` の leaf に移し、親は再公開する。
  - **module の循環を作らない:** module の参照（`use` と、本体の中の path）は循環してはいけない。次の形にする。
    - **passive state と処理を分ける:** 集約の型（`Model`、`GpuContext`、`Builder`、`CpuCompositor` など）の field になる型は、集約の型を参照しない leaf（例: `drawables/state.rs`）に置く。集約の型の method を実装する処理の leaf（例: `drawables/place.rs`）は、集約の型と passive state の leaf を参照してよい。
    - **集約の型は passive state を leaf の path で import する:** `use crate::model::drawables::state::Drawables` のように直接書き、親の再公開を経由しない（親が処理の leaf も再公開していると、親を経由した参照が循環になる）。
    - **処理の leaf 同士は path で参照しない:** 再帰（layer → item → 子 layer など）は、集約の型の method（`self.emit_items()`）で呼ぶ。複数の処理の leaf が使う free 関数と型は、処理を持たない helper の leaf（例: `plan/frame.rs`、`gpu/pass.rs`）に置く。
    - **子は兄弟を兄弟の path で参照する:** `use super::types::DrawOp` と書き、親の再公開（`use super::DrawOp`）を経由しない。
  - **Role:** すべての module に `//! Role: <説明>` を一行置く。説明は 80 文字以下（Unicode の文字数）にする。説明を補う文は、Role の行の後の別の段落に書く。inline の `mod x { … }` にも Role が要る（inline の module を作らず file にする）。
  - **その他の style の規則:** `#[rustfmt::skip]` を使わない（表は通常の書式で書く）。一つの `use` は一つの module から import する（波括弧の中に入れ子の path を書かない）。引数に `&Option<T>`、`&Vec<T>`、`&String`、`&Box<T>` を使わない（`Option<&T>`、`&[T]`、`&str`、`&T`）。
  - **依存の可視性:** crate の依存は `[visible_to]`（§4.4）に従う。
- 新しい `unsafe` は `kivact-gpu-share` と `arvane-compose/src/gpu/` の中だけに置く。どちらも既存の `undocumented_unsafe_blocks` 等の lint に従う。drawable の export の公開 API（kivact-core の `DrawableShare`）は unsafe を持たない（§8.12 の準備の合図）。

---

## 1. 目的・判断基準・決定済み事項

### 1.1 目的

Kivact で、HTML・CSS・JS と同等の視覚表現と、ゲーム風の UI 表現（マウスに追従するパララックス、3D 空間に置いた UI、shader による演出、アニメーションを多用する HUD）を実現する。ここでの「ゲーム風」はゲームのような見た目のアプリ UI を指す。Unity や Unreal Engine に組み込む UI は対象外のままである。

### 1.2 判断基準

設計上の選択は、次の順で判断した。実装中に付録 A で裁定するときも同じ順で考える。

1. Rust 未経験の React・SolidJS 開発者がスムーズに書ける公開 API である。
2. 公開 API がわかりやすく統一的である（一つの意味に一つの経路）。
3. 普段 JavaScript や Python を書く Rust 未経験者でも記述できる（CSS の名前と意味を優先する）。
4. Kivact 内部が統一的でシンプルな構造である。
5. 長期保守性。
6. `docs/for-kivact-developer/strategy.md` を簡単に遂行できる。

crate 分割はモジュール分割より優先する。ただし境界が明確な場合に限る（architecture.md B-1〜B-5）。

### 1.3 決定済み事項

以下はユーザーが決定した。実装中に再検討しない。

| ID | 決定 |
|---|---|
| D-01 | 前回の調査で「一部だけ」「ないもの」とした描画機能を、すべて「揃っているもの」相当にする。調査で見つかった小さな不足も含める |
| D-02 | 前回の調査で不足とされたアニメーション機能をすべて実現する |
| D-03 | HTML-in-Canvas に相当する機能を実現する |
| D-04 | 3D transform と HTML-in-Canvas は CPU 描画と GPU 描画で一致させる。E2E は CPU 描画で行う |
| D-05 | shader はユーザーが WGSL をそのまま書く形にする。GPU でだけ実行し、CPU 描画では宣言された代わりの見た目を描く |
| D-06 | 色は CSS Color 4 の色空間で指定・補間できる。出力は sRGB のままで、display の色域への出力と HDR は行わない（経路ごとの扱いは §16.4 の CONTRACT-COLOR。engine の state transition だけは sRGB に写した端点から補間する。この制限はユーザーが 2026-11-07 に承認した） |
| D-07 | 部分再描画と layer のキャッシュを実現する |
| D-08 | Vello は各 layer の 2D ラスタライザとして残す。その上に Kivact 自身の compositor を CPU 版と GPU 版の両方で作る。Skia への乗り換えはしない |
| D-09 | 進め方は foundation v3 と同じ（設計書＋作業指示書、別 AI がサブエージェントで実装、最大 6 並列、squash merge） |
| D-10 | 互換性は破壊してよい（非公開の α 版） |

### 1.4 対象外

- Unity・Unreal Engine 等のゲームエンジンへの組み込み。
- macOS、Android、iOS、Web のバックエンド（SCOPE-01 のまま）。
- display の色域への出力、HDR 出力、画像の ICC 色管理。
- CPU 描画での shader 実行、CPU 描画での外部 GPU frame の表示（`cpu` の代替 pixels がある場合を除く）。
- スクロールコンテナの layer 化（タイル分割が必要になるため）。スクロールは damage に基づく再ラスタライズで描く。
- `matrix3d()` のうち最下行が `(0, 0, 0, 1)` でないもの（§5.3）。遠近は `perspective()` 関数と `perspective` プロパティで表す。
- Path の boolean 演算（SCOPE-02 のまま）。

---

## 2. 用語

| 用語 | 意味 |
|---|---|
| layer（compositor layer） | `arvane-compose` が独立した描画先（texture または pixmap）として扱う単位。`kivact_types::render::CompositeLayer` |
| chunk（raster chunk） | layer の中で、Vello が一度に描く連続した 2D 描画命令の列。`LayerItem::Raster` |
| 描画計画（plan） | `Composition` から作る、backend に依存しない描画手順。`arvane_compose::plan::Plan` |
| 3D 描画コンテキスト | `transform-style: preserve-3d` によって同じ 3D 空間に並ぶ layer の集まり（CSS Transforms 2 の 3D rendering context） |
| 参加面 | 3D 描画コンテキストに参加し、奥行き順に並べ替えられる平面 |
| 平坦化 | 4×4 行列を z=0 平面の射影写像（3×3）に落とすこと |
| damage | 前のフレームから見た目が変わった可能性のある領域 |
| drawable | canvas の描画関数が描く、実 view の部分木（HTML-in-Canvas の drawable element） |
| export | drawable の描画結果を、アプリの描画エンジンへ CPU pixels または共有 GPU texture として渡すこと |
| decorative | 動きを減らす設定のときに止める、または即座に終える動き。既存の `Transition::decorative` と同じ意味 |

---

## 3. 全体構成

### 3.1 フレームの流れ

```text
kivact-style (Style, Draw)
   │ compile_style / record
   ▼
kivact-engine
   paint list の walk（保持・cull）
   → CompositionBuilder: layer 化（§6.1）、chunk 化、item 化
   ▼
kivact_types::render::Composition   ← 1 フレームの完全な記述（値）
   ▼
arvane::Window::present(&Composition, FramePlan)
   ▼
arvane-compose
   plan::build(&Composition, &Cache) → Plan（backend 非依存の Op 列、damage）
   ├─ CpuCompositor: Op を pixmap で実行（chunk は arvane_render::CpuRaster が Vello CPU で描く）
   └─ GpuCompositor: Op を wgpu で実行（chunk は arvane_render::GpuRaster が vello_hybrid で描く）
   ▼
window surface（CPU: softbuffer に damage 付きで提示 / GPU: swapchain）
   export: drawable layer → DrawableShare（pixels または共有 texture）
```

### 3.2 一つの事実の持ち主

| 事実 | 持ち主 |
|---|---|
| 何を描くか（layer 木、chunk、item、効果） | engine が毎フレーム作る `Composition`（値）。compositor は状態として保持しない |
| 前フレームのラスタ結果 | compositor の cache（純粋 cache。S-9）。`Composition` との比較だけで再利用を決める |
| node の画面上の写像（hit test、a11y、rect、pointer 座標） | engine の model（§7）。描画と同じ写像を使う（G-07） |
| drawable の配置 | canvas の `DrawList` から導出した engine の `Drawables`（§10.3）。書き手は `set_scene` だけ |
| アニメーションの時刻 | runtime の clock 一つ（既存） |

### 3.3 CPU と GPU の一致の作り方

CPU と GPU の差は、`Plan` の各 `Op` を実行する部分だけにする。layer の評価、raster scale、3D の並べ替え、damage、可視判定は `plan` モジュールが backend を知らずに行う。各 `Op` の画素の計算式は §8 で一つずつ定義し、CPU 版と GPU 版は同じ式を実装する。

一致の保証は、次の三段に分けて定める。

| 対象 | 保証 |
|---|---|
| 幾何（描く位置、hit test、a11y の bounds、rect） | CPU と GPU で完全に同じ。engine が一つの写像を持ち、backend は使わない |
| compositor の Op（合成、3D の射影、filter、mask、backdrop） | 各 channel ±2/255 以内（8 bit の量子化と浮動小数点の丸め） |
| chunk の中の Vello のラスタライズ（輪郭の AA、glyph、box shadow の blur） | 既存の Vello の CPU と GPU の差のまま。最大差は保証しない。比較は、既存の `kivact-test/src/screen/gpu_tests.rs` の分布の基準（平均の差、大きな差の画素の比率）を使う。この差を本書は縮めない（ユーザーが 2026-11-07 に承認した） |

試験は、比べる画素がどの段かを明記し、その段の許容差を使う（§17.1）。

GPU だけで描くもの（user shader、外部 GPU frame）は、CPU 版が `Op` の `fallback` を描く（§8.9、§8.10）。

---

## 4. crate 構成

### 4.1 変更の一覧

| package | 変更 | 担当（変更後） |
|---|---|---|
| `kivact-types` | 拡張 | 3D 行列と射影写像、色空間と gamut mapping、`Composition`、filter graph、shader program の値、新しい paint・text・scene の語彙、新しい motion の語彙 |
| `kivact-shader`（新規） | 新設 | WGSL の検証（naga）と、Kivact の shader 契約に合わせた完全な WGSL module の組み立て。device を持たない |
| `kivact-style` | 拡張 | 3D・mask・border・background・text・filter・shader・color の authored API と resolve |
| `kivact-runtime` | 拡張 | 新しい easing・spring・keyframe の stepper、`Animated` の再構成、`Timeline`、`Stagger`、frame loop の reduced motion |
| `kivact-engine` | 変更 | `Composition` の生成（layer 化）、射影写像による hit test・a11y、per-property transition、shared layout、drawable、animated image の再生 |
| `kivact-core` | 拡張 | drawable（HTML-in-Canvas）、`DrawableShare`、scroll timeline、presence mode、`layout_id`、公開 API の再公開 |
| `arvane-render` | 縮小 | `RenderScene` を CPU pixmap または GPU texture に描くラスタライザだけになる。GPU device、surface、外部 frame、mask・backdrop・filter の emulation を持たない |
| `arvane-compose`（新規） | 新設 | `Composition` の合成。GPU device と surface の所有、layer cache、damage、3D、効果、user shader、外部 frame の import、drawable の export、offscreen 読み戻し |
| `arvane` | 変更 | presenter が `arvane-compose` を使う。`Window::present(&Composition, FramePlan)`。CPU 提示に damage を渡す |
| `arvane-shell` | 変更 | `ShellWindow::present_with` に damage と buffer の鮮度を追加 |
| `arvane-native` | 変更 | damage 付き CPU 提示、Linux の reduced motion（portal）、X11 の refresh rate（RandR） |
| `arvane-virtual` | 変更 | 新しい `present_with` の契約 |
| `arvane-text` | 拡張 | font variation の全軸、font width、optical sizing |
| `arvane-asset` | 拡張 | GIF・WebP・APNG のアニメーション decode、SVG の `<text>` と埋め込み raster 画像 |
| `kivact-gpu-share` | 拡張 | window 側からの export（drawable 用）。window device を import と export の両方に使える形で開く |
| `kivact-interaction`、`kivact-ui`、五つの kit | 移行 | API 変更の追従。kit の README と catalog に新機能の見本を足す |
| `kivact` | 変更 | 新しい公開名の再公開、feature の中継 |

workspace は 40 package から 42 package になる。library は 25 から 27 になる。

### 4.2 新しい crate の境界

#### kivact-shader

- **役割:** `//! Role: Validate user WGSL and assemble Kivact shader modules.`
- **入力:** WGSL のソース文字列と `ShaderKind`。
- **出力:** `kivact_types::render::ShaderProgram`（完全な WGSL module のテキストと param のレイアウト）、または `ShaderError`（位置付きのメッセージ）。
- **依存:** `kivact-types`、`naga = { version = "=29.0.4", default-features = false, features = ["wgsl-in"] }`。wgpu の依存する naga と同じ版に固定する。
- **境界の理由:** naga の型と依存を一か所に閉じる。kivact-style が shader の構築時に検証してすぐ `UiError` を返せる（G-05）一方で、kivact-style に wgpu を持ち込まない。device を持たない純粋な変換なので、compositor とも独立している（B-1、B-2）。
- **visible_to:** `["kivact-style", "arvane-compose"]`。

#### arvane-compose

- **役割:** `//! Role: Composite layer trees on the CPU and the GPU with caches and damage.`
- **入力:** `&Composition`、`FramePlan`（大きさと scale）、提示先（CPU frame buffer または GPU surface）。
- **出力:** 提示結果、window の damage、export した drawable frame、見落とした外部 frame。
- **依存:** `kivact-types`、`arvane-render`、`kivact-shader`、`wgpu`（feature `gpu`）、`pollster`（feature `gpu`）、`raw-window-handle`、`bytemuck`、`kivact-gpu-share`（feature `gpu-interop`、features `["import", "wgpu"]`）。
- **境界の理由:** 2D ベクターのラスタライズ（Vello）と、raster 結果の合成（3D、効果、cache、shader、外部 texture）は入出力と変更理由が異なる。合成は wgpu の独自 pipeline と GPU 共有を持ち、ラスタライズは Vello を持つ。両者の間は「RenderScene を指定の描画先に描く」という少数の型で説明できる（B-1）。Chrome の Skia と cc の分担と同じ。
- **visible_to:** `["arvane", "kivact"]`（`kivact` は feature の中継だけに使う）。
- **feature:** `gpu = ["arvane-render/gpu", "dep:wgpu", "dep:pollster"]`、`gpu-interop = ["gpu", "arvane-render/gpu", "dep:kivact-gpu-share"]`。`arvane-render` の `gpu-interop` feature は削除する（外部 frame の import が compose へ移るため）。

### 4.3 移動する責務

| 責務 | 移動元 | 移動先 |
|---|---|---|
| `GpuContext`、`GpuEnv`、`GpuSurface`、`Presentation`、adapter の選択、swapchain、`DeviceLost` | `arvane-render/src/gpu/{state,context,surface}.rs` | `arvane-compose/src/gpu/{state,context,surface}.rs` |
| 外部 frame の import と frame ごとの acquire/release（`ExternalPass`） | `arvane-render/src/gpu/{external,shared_images}.rs` | `arvane-compose/src/gpu/{external,shared_images}.rs` |
| GPU offscreen の読み戻し | `arvane-render/src/gpu/offscreen.rs` | `arvane-compose/src/gpu/offscreen.rs` |
| mask、backdrop、layer filter、drop shadow の emulation | `arvane-render/src/draw/effects.rs`、`arvane-render/src/backdrop.rs` | 削除。compositor の効果（§8）に置き換える |
| command の root-space bounds の索引 | `arvane-render/src/backdrop.rs` の `Index` | `arvane-compose/src/plan/bounds.rs`（damage 用） |
| `GPU_BUILT` | `arvane-render` | `arvane-compose`（`arvane-render` は `RASTER_GPU_BUILT` を持たない。feature の有無は compose が答える） |

### 4.4 依存 policy の変更（`dev/xtask/policy/workspace.toml`）

```toml
[visible_to]
kivact-shader = ["kivact-style", "arvane-compose"]
arvane-compose = ["arvane", "kivact"]
arvane-render = ["arvane", "arvane-compose", "kivact"]
```

- `kivact-style` の Cargo.toml に `kivact-shader` を追加する。
- `arvane` の Cargo.toml に `arvane-compose` を追加する。`arvane` の `arvane-render` への依存は、`glyph_ink_bounds`、`glyph_outline`、`MAX_SCENE_DEPTH`、`raster_plan`、`RenderError` のために残す。`arvane` の dev-dependency `arvane-render = { features = ["gpu-interop"] }` は `arvane-compose = { features = ["gpu-interop"] }` に変える。
- `kivact` の Cargo.toml: `gpu = ["dep:arvane-compose", "arvane-compose/gpu"]`、`gpu-interop = ["gpu", "arvane-compose/gpu-interop", "kivact-core/gpu-interop", "kivact-test?/gpu-interop"]`。`arvane-render` への直接依存は削除する。
- `[style] api_packages` に `kivact-shader` は加えない（facade から到達しない）。
- `[os_branch] allowed_paths` に `crates/arvane-compose/src/gpu/` を追加し、`crates/arvane-render/src/gpu/` を削除する。
- `dev/tools/check-features.sh` に `arvane-compose/gpu` と `arvane-compose/gpu-interop` を追加し、`arvane-render/gpu-interop` を削除する。
- `[module_rules]` に `arvane-compose` を追加する:

```toml
[module_rules.arvane-compose.uses]
cache = []
plan = ["cache"]
cpu = ["plan", "cache"]
gpu = ["plan", "cache"]
```

### 4.5 workspace の依存の追加

```toml
[workspace.dependencies]
naga = { version = "=29.0.4", default-features = false, features = ["wgsl-in"] }
zbus = { version = "5.19.0", default-features = false, features = ["blocking-api", "async-io"] }
```

- `x11rb` の feature に `"randr"` を追加する（`arvane-native` の Linux 依存）。
- `resvg` の feature を `default-features = false, features = ["text", "raster-images"]` にする（`arvane-asset`）。`usvg` の `text` feature が fontdb を連れてくる。fontdb の system font 読み込みには `fontdb/fs` と `fontdb/memmap` を有効にする（`fontdb = { version = "0.23", default-features = false, features = ["fs", "memmap"] }` を `arvane-asset` に直接追加し、usvg と同じ版を使う。版は `cargo tree -p usvg` で usvg の fontdb の版に合わせる）。
- `image` の feature に `"png"`（既存）に加えて APNG のための追加は不要（`PngDecoder::apng` は既定で使える）。
- `THIRD-PARTY-NOTICES.md` に、§8.7 で移植する resvg の filter 実装（MIT OR Apache-2.0）と、追加した crate（naga は既存の推移依存、zbus は既存の推移依存、fontdb、rustybuzz）を追記する。

---

## 5. kivact-types の新しい語彙

kivact-types は外部依存を持たない（既存の方針）。以下の型は、特記がない限りすべて `Clone + Debug + PartialEq` を derive する（`Copy` は記載があるものだけ）。新しい module は `//! Role:` を持つ。`kivact-types/src/lib.rs` の re-export に追加する名前は、各節の末尾に書く。

### 5.1 3D 行列と射影写像（`src/geometry3d.rs`、新規）

```rust
//! Role: 3D matrices, plane projections and polygons for 3D transforms.

/// A 4×4 matrix on column vectors (x, y, z, w): p' = M · p, `m[row][col]`.
/// CSS `matrix3d(a1, …, a16)` lists the matrix column by column:
/// a1 = m[0][0], a2 = m[1][0], a3 = m[2][0], a4 = m[3][0], a5 = m[0][1], …
/// Positive z points toward the viewer; y points down (as in CSS).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix4 { pub m: [[f64; 4]; 4] }

impl Matrix4 {
    pub const IDENTITY: Matrix4;
    pub fn translate(x: f64, y: f64, z: f64) -> Matrix4;
    pub fn scale(x: f64, y: f64, z: f64) -> Matrix4;
    /// CSS rotateX: [[1,0,0],[0,c,-s],[0,s,c]] in the upper 3×3.
    pub fn rotate_x(deg: f64) -> Matrix4;
    /// CSS rotateY: [[c,0,s],[0,1,0],[-s,0,c]].
    pub fn rotate_y(deg: f64) -> Matrix4;
    /// CSS rotate / rotateZ: [[c,-s,0],[s,c,0],[0,0,1]] (clockwise on screen).
    pub fn rotate_z(deg: f64) -> Matrix4;
    /// CSS rotate3d(x, y, z, a): Rodrigues' formula about the normalized
    /// axis; a zero or non-finite axis gives the identity.
    pub fn rotate_axis(x: f64, y: f64, z: f64, deg: f64) -> Matrix4;
    /// Upper 3×3 [[1, tan(xy), tan(xz)], [0, 1, tan(yz)], [0, 0, 1]].
    pub fn skew(xy_deg: f64, xz_deg: f64, yz_deg: f64) -> Matrix4;
    /// CSS perspective(d): m[3][2] = -1/d. d <= 0 or non-finite gives the identity.
    pub fn perspective(distance: f64) -> Matrix4;
    pub fn from_affine(t: Transform) -> Matrix4;
    /// The plane map `p` with z passed through:
    /// [[h00, h01, 0, h02], [h10, h11, 0, h12], [0, 0, 1, 0], [h20, h21, 0, h22]].
    pub fn from_projective(p: Projective) -> Matrix4;
    /// `self` first, then `next` (= next · self), as `Transform::then`.
    pub fn then(self, next: Matrix4) -> Matrix4;
    pub fn determinant(&self) -> f64;
    /// None when |det| < 1e-12 or a result is not finite.
    pub fn invert(&self) -> Option<Matrix4>;
    pub fn transform_point(&self, p: [f64; 4]) -> [f64; 4];
    /// Some when row 2, row 3 and column 2 are those of the identity
    /// (within 1e-12): the matrix is a 2D affine map.
    pub fn as_affine(&self) -> Option<Transform>;
    /// The map of the plane z = 0: rows {0, 1, 3} × columns {0, 1, 3}.
    pub fn flatten(&self) -> Projective;
    /// CSS flattening into a flat parent: row 2 and column 2 become those
    /// of the identity (m[2][*] = m[*][2] = 0 except m[2][2] = 1).
    pub fn flattened(&self) -> Matrix4;
    /// Decompose a matrix whose bottom row is (0, 0, 0, 1) (§5.3). None
    /// for a singular upper 3×3 or another bottom row.
    pub fn decompose(&self) -> Option<Decomposed>;
    pub fn is_finite(&self) -> bool;
    /// The back face is toward the viewer: invertible and the inverse has
    /// m[2][2] < 0 (CSS Transforms 2). Mirrors (scale(−1, 1, 1)) show the front.
    pub fn shows_back(&self) -> bool;
}

/// translate · Rz · Ry · Rx · skew · scale, as §5.3 decomposes it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decomposed {
    pub translate: [f64; 3],
    /// Degrees about x, y and z; the rotation is Rz · Ry · Rx.
    pub rotate: [f64; 3],
    /// Degrees: xy (CSS skewX), xz, yz.
    pub skew: [f64; 3],
    pub scale: [f64; 3],
}

/// A projective map of the plane (a 3×3 homography on (x, y, 1)):
/// (x', y', w') = H · (x, y, 1), point (x'/w', y'/w'). Points with
/// w' <= W_EPS are behind the viewer and have no image.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projective { pub m: [[f64; 3]; 3] }
pub const W_EPS: f64 = 1e-6;

impl Projective {
    pub const IDENTITY: Projective;
    pub fn from_affine(t: Transform) -> Projective;
    /// Some when m[2] == [0, 0, 1] within 1e-12.
    pub fn as_affine(&self) -> Option<Transform>;
    pub fn then(self, next: Projective) -> Projective;
    pub fn determinant(&self) -> f64;
    pub fn invert(&self) -> Option<Projective>;
    pub fn apply(&self, p: Point) -> Option<Point>;
    /// The convex polygon `r` maps to, clipped to w' > W_EPS (§5.2).
    pub fn map_rect(&self, r: Rect) -> Option<Polygon>;
    /// Bounds of `map_rect`; None when nothing is in front of the viewer.
    pub fn bounds(&self, r: Rect) -> Option<Rect>;
    /// Largest ratio of a mapped edge length to its original length over
    /// the four edges of `r` and its two diagonals; 1.0 when `r` has no
    /// image or is empty.
    pub fn max_scale(&self, r: Rect) -> f64;
}

/// A convex polygon of at most 16 vertices, in order.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Polygon { pub points: Vec<Point> }
impl Polygon {
    pub fn bounds(&self) -> Option<Rect>;
    pub fn contains(&self, p: Point) -> bool;
}
```

`lib.rs` に `pub use geometry3d::{Decomposed, Matrix4, Polygon, Projective, W_EPS};` を追加する。`geometry.rs` の `Insets` に `From<f64>`、`From<i32>`、`From<(f64, f64)>`（縦, 横）、`From<(f64, f64, f64, f64)>`（上, 右, 下, 左）を追加する。

### 5.2 射影の切り取り

`Projective::map_rect` は次の手順で行う。

1. 4 頂点を同次座標 (x', y', w') に写す。
2. Sutherland–Hodgman 法で、半空間 w' > W_EPS に切り取る。辺 (a, b) が境界をまたぐときは、t = (W_EPS − w_a) / (w_b − w_a) として同次座標のまま線形補間し、交点を求める。
3. 残った頂点を w' で割り、平面の点にする。頂点が 3 未満なら None を返す。

compositor の 3D 合成（§8.4）も同じ関数を使う。

### 5.3 3D 行列の分解

`Matrix4::decompose` は次の手順で行う。最下行が (0, 0, 0, 1) でない場合、または上 3×3 の行列式の絶対値が 1e-12 未満の場合は None を返す。

1. translate = (m[0][3], m[1][3], m[2][3]) とする。
2. 上 3×3 の列を c0, c1, c2（x, y, z 軸の像）とし、Gram–Schmidt 法で分解する。
   - sx = |c0|、q0 = c0 / sx
   - d01 = q0·c1、u1 = c1 − d01·q0、sy = |u1|、q1 = u1 / sy
   - d02 = q0·c2、d12 = q1·c2、u2 = c2 − d02·q0 − d12·q1、sz = |u2|、q2 = u2 / sz
3. det(上 3×3) < 0 なら sx = −sx、q0 = −q0、d01 = −d01、d02 = −d02 とし、R を回転に保つ（R·U = L を保つため、U の第 1 行全体の符号を反転する）。
4. skew = (atan(d01 / sy), atan(d02 / sz), atan(d12 / sz)) を度に変換し、scale = (sx, sy, sz) とする。
5. R = [q0 q1 q2]（列）を R = Rz(γ) · Ry(β) · Rx(α) に分解する。
   - β = asin(clamp(−R[2][0], −1, 1))
   - |cos β| > 1e-9 なら α = atan2(R[2][1], R[2][2])、γ = atan2(R[1][0], R[0][0])
   - そうでなければ γ = 0、α = atan2(−R[1][2], R[1][1])
   - rotate = (α, β, γ) を度に変換する。

再構成は translate · Rz · Ry · Rx · skew · scale である。z = 0 のとき、この分解は 2D の既存の QR 分解（translate · rotate · skewX · scale）と一致する。

`Matrix4::decompose` の rustdoc に、分解して再構成すると元に戻る（各要素の差が 1e-9 以内）ことを確かめる assert 付きの例を書く。例の入力は次の 5 つとする。

- 上 3×3 が [[−1, 1, 0], [0, 1, 0], [0, 0, 1]]（負の scale と shear）
- rotate_x(30)·rotate_y(40)·rotate_z(50)·skew(10, 5, 3)·scale(2, −1, 0.5)
- scale(1e-6, 1, 1)（特異に近い）
- rotate_y(90)（gimbal）
- 単位行列

### 5.4 色空間（`src/color_space.rs`、新規）

```rust
//! Role: CSS Color 4 color spaces, conversion, interpolation and gamut mapping.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ColorSpace {
    #[default] Srgb, SrgbLinear, DisplayP3, A98Rgb, ProphotoRgb, Rec2020,
    XyzD50, XyzD65, Lab, Lch, Oklab, Oklch, Hsl, Hwb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum HueInterpolation { #[default] Shorter, Longer, Increasing, Decreasing }

/// How colors are interpolated (CSS `in <space> <hue>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ColorInterpolation { pub space: ColorSpace, pub hue: HueInterpolation }
impl ColorInterpolation {
    /// CSS's default for gradients, transitions and color-mix: Oklab.
    pub const DEFAULT: ColorInterpolation = ColorInterpolation { space: ColorSpace::Oklab, hue: HueInterpolation::Shorter };
    /// Canvas 2D gradients: premultiplied sRGB.
    pub const SRGB: ColorInterpolation = ColorInterpolation { space: ColorSpace::Srgb, hue: HueInterpolation::Shorter };
}
impl Default for ColorInterpolation { fn default() -> Self { Self::DEFAULT } }

/// A color in a space with alpha. Channel ranges are CSS's: sRGB family
/// 0..1, XYZ unbounded, Lab L 0..100, Lch C >= 0 and H in degrees, Oklab L
/// 0..1, Oklch C >= 0 and H in degrees, Hsl H degrees with S and L 0..1,
/// Hwb H degrees with W and B 0..1. Alpha 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpaceColor { pub space: ColorSpace, pub channels: [f64; 3], pub alpha: f64 }

impl SpaceColor {
    pub fn new(space: ColorSpace, channels: [f64; 3], alpha: f64) -> SpaceColor;
    pub fn from_rgba(c: Rgba) -> SpaceColor;              // space Srgb
    pub fn convert(self, to: ColorSpace) -> SpaceColor;
    /// sRGB in gamut by CSS Color 4 §13.2 (OKLCh chroma reduction).
    pub fn to_srgb(self) -> Rgba;
    /// Whether the hue channel is powerless (§5.4.3).
    pub fn hue_is_powerless(&self) -> bool;
}

/// CSS Color 4 §12 interpolation at `t` (not clamped).
pub fn interpolate(a: SpaceColor, b: SpaceColor, t: f64, how: ColorInterpolation) -> SpaceColor;
/// Interpolate two sRGB colors and map the result back to sRGB.
pub fn mix_rgba(a: Rgba, b: Rgba, t: f64, how: ColorInterpolation) -> Rgba;
/// Euclidean distance in Oklab.
pub fn delta_e_ok(a: SpaceColor, b: SpaceColor) -> f64;
```

`lib.rs` に `pub use color_space::{ColorInterpolation, ColorSpace, HueInterpolation, SpaceColor, delta_e_ok, interpolate, mix_rgba};` を追加する。

#### 5.4.1 変換

- すべての変換は XYZ D65 を経由する。
- D50 の空間（XyzD50、Lab、Lch、ProphotoRgb）は、Bradford 変換で D65 と行き来する。
- 極座標の空間（Lch、Oklch、Hsl、Hwb）は、直交座標の対応空間（それぞれ Lab、Oklab、sRGB、sRGB）を経由する。
- 行列は CSS Color 4 §18 のサンプルコードの値をそのまま使う（下表）。

| 変換 | 行列（行ごと） |
|---|---|
| linear sRGB → XYZ D65 | [0.41239079926595934, 0.357584339383878, 0.1804807884018343], [0.21263900587151027, 0.715168678767756, 0.07219231536073371], [0.01933081871559182, 0.11919477979462598, 0.9505321522496607] |
| XYZ D65 → linear sRGB | [3.2409699419045226, −1.537383177570094, −0.4986107602930034], [−0.9692436362808796, 1.8759675015077202, 0.04155505740717559], [0.05563007969699366, −0.20397695888897652, 1.0569715142428786] |
| linear P3 → XYZ D65 | [0.4865709486482162, 0.26566769316909306, 0.1982172852343625], [0.2289745640697488, 0.6917385218365064, 0.079286914093745], [0.0, 0.04511338185890264, 1.043944368900976] |
| XYZ D65 → linear P3 | [2.493496911941425, −0.9313836179191239, −0.40271078445071684], [−0.8294889695615747, 1.7626640603183463, 0.023624685841943577], [0.03584583024378447, −0.07617238926804182, 0.9568845240076872] |
| linear A98 → XYZ D65 | [0.5766690429101305, 0.1855582379065463, 0.1882286462349947], [0.29734497525053605, 0.6273635662554661, 0.07529145849399788], [0.02703136138641234, 0.07068885253582723, 0.9913375368376388] |
| XYZ D65 → linear A98 | [2.0415879038107465, −0.5650069742788596, −0.34473135077832956], [−0.9692436362808795, 1.8759675015077202, 0.04155505740717557], [0.013444280632031142, −0.11836239223101838, 1.0151749943912054] |
| linear Rec2020 → XYZ D65 | [0.6369580483012914, 0.14461690358620832, 0.1688809751641721], [0.2627002120112671, 0.6779980715188708, 0.05930171646986196], [0.0, 0.028072693049087428, 1.060985057710791] |
| XYZ D65 → linear Rec2020 | [1.716651187971268, −0.355670783776392, −0.25336628137366], [−0.666684351832489, 1.616481236634939, 0.0157685458139111], [0.017639857445311, −0.042770613257809, 0.942103121235474] |
| linear ProPhoto → XYZ D50 | [0.7977666449006423, 0.13518129740053308, 0.0313477341283922], [0.2880748288194013, 0.711835234241873, 0.00008993693872564], [0.0, 0.0, 0.8251046025104602] |
| XYZ D50 → linear ProPhoto | [1.3457868816471583, −0.25557208737979464, −0.05110186497554526], [−0.5446307051249019, 1.5082477428451468, 0.02052744743642139], [0.0, 0.0, 1.2119675456389452] |
| XYZ D65 → XYZ D50（Bradford） | [1.0479298208405488, 0.022946793341019088, −0.05019222954313557], [0.029627815688159344, 0.990434484573249, −0.01707382502938514], [−0.009243058152591178, 0.015055144896577895, 0.7518742899580008] |
| XYZ D50 → XYZ D65 | [0.9554734527042182, −0.023098536874261423, 0.0632593086610217], [−0.028369706963208136, 1.0099954580058226, 0.021041398966943008], [0.012314001688319899, −0.020507696433477912, 1.3303659366080753] |
| XYZ D65 → LMS（Oklab M1） | [0.8190224379967030, 0.3619062600528904, −0.1288737815209879], [0.0329836539323885, 0.9292868615863434, 0.0361446663506424], [0.0481771893596242, 0.2642395317527308, 0.6335478284694309] |
| ∛LMS → Oklab（M2） | [0.2104542683093140, 0.7936177747023054, −0.0040720430116193], [1.9779985324311684, −2.4285922420485799, 0.4505937096174110], [0.0259040424655478, 0.7827717124575296, −0.8086757549230774] |
| Oklab → ∛LMS | [1.0, 0.3963377773761749, 0.2158037573099136], [1.0, −0.1055613458156586, −0.0638541728258133], [1.0, −0.0894841775298119, −1.2914855480194092] |
| LMS → XYZ D65 | [1.2268798758459243, −0.5578149944602171, 0.2813910456659647], [−0.0405757452148008, 1.1122868032803170, −0.0717110580655164], [−0.0763729366746601, −0.4214933324022432, 1.5869240198367816] |

伝達関数（s = sign(v)、a = |v|）:

| 空間 | 符号化 → 線形 | 線形 → 符号化 |
|---|---|---|
| sRGB、Display P3 | a ≤ 0.04045 ? v / 12.92 : s·((a + 0.055) / 1.055)^2.4 | a > 0.0031308 ? s·(1.055·a^(1/2.4) − 0.055) : 12.92·v |
| A98 RGB | s·a^(563/256) | s·a^(256/563) |
| ProPhoto RGB | a ≤ 16/512 ? v / 16 : s·a^1.8 | a ≥ 1/512 ? s·a^(1/1.8) : 16·v |
| Rec2020（α = 1.09929682680944、β = 0.018053968510807） | a < β·4.5 ? v / 4.5 : s·((a + α − 1) / α)^(1/0.45) | a > β ? s·(α·a^0.45 − (α − 1)) : 4.5·v |

Lab（D50 白色点 [0.9642956764295677, 1.0, 0.8251046025104602]、κ = 24389/27、ε = 216/24389）:

- XYZ → Lab:
  - 各成分を白色点で割る。
  - f(v) = v > ε ? ∛v : (κ·v + 16) / 116 とする。
  - L = 116·f1 − 16、a = 500·(f0 − f1)、b = 200·(f1 − f2)。
- Lab → XYZ:
  - f1 = (L + 16) / 116、f0 = a / 500 + f1、f2 = f1 − b / 200。
  - x = f0³ > ε ? f0³ : (116·f0 − 16) / κ
  - y = L > κ·ε ? f1³ : L / κ
  - z = f2³ > ε ? f2³ : (116·f2 − 16) / κ
  - 最後に白色点を掛ける。

その他:

- 極座標: C = √(a² + b²)、H = atan2(b, a) を度にし、[0, 360) に正規化する。
- HSL と HWB: CSS Color 4 §7、§8 のサンプルコード（`rgbToHsl`、`hslToRgb`、`rgbToHwb`、`hwbToRgb`）と同じ式を使う。

#### 5.4.2 補間（CSS Color 4 §12）

1. 両方の色を補間空間へ変換する。
2. 直交座標の空間では、各 channel に alpha を掛ける（premultiply）。極座標の空間では、hue 以外の channel に alpha を掛ける。
3. 極座標の空間では、Δ = h2 − h1 として hue を補正する。
   - Shorter: Δ > 180 なら h1 += 360、Δ < −180 なら h2 += 360
   - Longer: 0 < Δ < 180 なら h1 += 360、−180 < Δ ≤ 0 なら h2 += 360
   - Increasing: Δ < 0 なら h2 += 360
   - Decreasing: Δ > 0 なら h1 += 360
4. 各 channel と alpha を線形補間する。
5. alpha が 0 でなければ、premultiply した channel を alpha で割り戻す。hue は [0, 360) に正規化する。

hue が powerless な色については次のように扱う。

- 一方の hue だけが powerless なら、もう一方の hue を両方に使う。
- 両方とも powerless なら、hue は 0 とする。

#### 5.4.3 powerless な hue

| 空間 | 条件 |
|---|---|
| Lch | C < 0.02 |
| Oklch | C < 0.00002 |
| Hsl | S < 1e-6、または L が 0 か 1（誤差 1e-6） |
| Hwb | W + B ≥ 1 − 1e-6 |

#### 5.4.4 gamut mapping（CSS Color 4 §13.2.2）

`to_srgb` は次の手順で行う。alpha は変換を通して保つ。

1. 色を sRGB に変換する。各 channel が [0, 1] にあれば、そのまま返す。
2. Oklch に変換する。L ≥ 1 なら白を、L ≤ 0 なら黒を返す。
3. JND = 0.02、EPS = 0.0001 とする。clip(c) は、sRGB の各 channel を [0, 1] に丸める関数とする。
4. current を Oklch の色とし、clipped = clip(current を sRGB に変換したもの) とする。delta_e_ok(clipped, current) < JND なら clipped を返す。
5. min = 0、max = C、min_in_gamut = true とし、max − min > EPS の間、次を繰り返す。
   1. chroma = (min + max) / 2 とし、current の C を chroma にする。
   2. min_in_gamut が true で、current が sRGB の範囲内なら、min = chroma として次の反復へ進む。
   3. clipped = clip(current) とし、E = delta_e_ok(clipped, current) とする。
   4. E < JND の場合:
      - JND − E < EPS なら clipped を返す。
      - そうでなければ min_in_gamut = false、min = chroma とする。
   5. E ≥ JND の場合は max = chroma とする。
6. 反復が終わったら clipped を返す。

### 5.5 ElementTransform と 3D の paint 語彙（`src/paint.rs`）

`ElementTransform` を次の形に置き換える。

```rust
/// Element transform: CSS individual transforms and `transform`, in one
/// canonical form. About its origin the box is scaled, skewed, rotated
/// (x, then y, then z), moved, then seen through `perspective()`:
/// T(o) · P · T(t) · Rz · Ry · Rx · K · S · T(−o).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElementTransform {
    /// The move in px (x, y, z) ...
    pub translate: [f64; 3],
    /// ... plus fractions of the box size, as before:
    /// x += per_size[0]·w + per_size[1]·h, y += per_size[2]·w + per_size[3]·h.
    pub per_size: [f64; 4],
    /// Degrees about x, y and z.
    pub rotate: [f64; 3],
    /// Degrees: xy (skewX), xz, yz.
    pub skew: [f64; 3],
    pub scale: [f64; 3],
    /// 1/d of the transform's `perspective(d)` function; 0 is none.
    pub perspective_inverse: f64,
    /// Against the box's width and height (CSS default 50% 50%).
    pub origin: [Len; 2],
    /// z of the origin in px (CSS transform-origin's third value).
    pub origin_z: f64,
}
```

- `NONE` は次の値とする。
  - translate 0、per_size 0、rotate 0、skew 0、scale 1、perspective_inverse 0
  - origin 50%/50%、origin_z 0
- `is_none()` は、origin と origin_z を無視して判定する。
- `pub fn matrix(&self, size: Size) -> Matrix4` は上式で行列を作る。各記号は次のとおり。
  - o = (origin[0].of(w), origin[1].of(h), origin_z)
  - t = (translate[0] + per_size[0]·w + per_size[1]·h, translate[1] + per_size[2]·w + per_size[3]·h, translate[2])
  - perspective_inverse > 0 なら P = Matrix4::perspective(1 / perspective_inverse)、そうでなければ P は単位行列
- `pub fn is_2d(&self) -> bool` は、次をすべて満たすとき true を返す。
  - translate[2] = 0
  - rotate[0] = rotate[1] = 0
  - skew[1] = skew[2] = 0
  - scale[2] = 1
  - perspective_inverse = 0、origin_z = 0

`PaintProps` に次の field を追加する。

```rust
pub struct PaintProps {
    // … existing fields …
    /// CSS `perspective` and `perspective-origin`: how this node's children are seen.
    pub perspective: Option<Perspective>,
    pub transform_style: TransformStyle,
    pub backface: BackfaceVisibility,
    pub will_change: WillChange,
    pub isolation: Isolation,
    pub border_image: Option<Box<BorderImage>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Perspective { pub distance: f64, pub origin: [Len; 2] }   // origin default 50% 50%
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TransformStyle { #[default] Flat, Preserve3d }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BackfaceVisibility { #[default] Visible, Hidden }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WillChange { #[default] Auto, Transform, Opacity }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Isolation { #[default] Auto, Isolate }
```

Style は perspective の距離と origin を別々に持つ（`perspective: Option<f64>`、`perspective_origin: Option<(Length, Length)>`）。resolve は、距離があるときだけ `Perspective { distance, origin: origin（無ければ 50% 50%） }` を作り、距離の無い origin は無視する。state style の delta は、base と state を merge した style から作った完全な `Option<Perspective>` を持つ。

`PaintDelta` には次の field を追加し、state style で 3D を切り替えられるようにする。

- `perspective: Option<Option<Perspective>>`
- `transform_style: Option<TransformStyle>`
- `backface: Option<BackfaceVisibility>`

`will_change`、`isolation`、`border_image` は state style では変えられない（`DELTA` に加えない）。

### 5.6 border・background・mask の語彙（`src/paint.rs`、`src/scene/primitives.rs`）

```rust
// scene/primitives.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub enum BorderStyle { #[default] Solid, Dashed, Dotted, Double, Groove, Ridge, Inset, Outset }

// paint.rs
pub struct Border { pub widths: Edges<f64>, pub colors: Edges<Rgba>, pub styles: Edges<BorderStyle> }

pub struct FillLayer {
    pub image: FillImage,
    pub size: FillSize,
    pub position: [Len; 2],
    pub repeat_x: bool,
    pub repeat_y: bool,
    /// CSS background-clip / mask-clip.
    pub clip: BackgroundClip,
    /// CSS background-origin / mask-origin; None: the context's default
    /// (backgrounds: PaddingBox, masks: BorderBox).
    pub origin: Option<BackgroundOrigin>,
    /// CSS background-blend-mode (masks ignore it).
    pub blend: BlendMode,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BackgroundClip { #[default] BorderBox, PaddingBox, ContentBox, Text }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BackgroundOrigin { BorderBox, #[default] PaddingBox, ContentBox }

pub enum FillImage {
    Solid(Rgba), Gradient(Gradient), Image(Asset), Coordinate(CoordinateGradient),
    /// A paint shader (§11.3).
    Shader(std::sync::Arc<ShaderSpec>),
}

/// Gradient stops keep the authored color (not yet mapped to sRGB):
/// pub struct GradientStop { pub color: SpaceColor, pub at: Option<Len> }
pub struct Gradient {
    pub kind: GradientKind,
    pub stops: Vec<GradientStop>,
    /// CSS color hint between stop i and stop i + 1; len = stops.len() − 1.
    pub hints: Vec<Option<Len>>,
    pub repeating: bool,
    pub interpolation: ColorInterpolation,
}

// CoordinateGradient (Canvas gradients) gains the same field, default ColorInterpolation::SRGB:
// pub struct CoordinateGradient { pub shape, pub stops, pub extend, pub interpolation: ColorInterpolation }

/// One CSS mask layer.
pub struct MaskLayer { pub fill: FillLayer, pub mode: MaskMode, pub composite: MaskComposite }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MaskMode { #[default] Alpha, Luminance }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MaskComposite { #[default] Add, Subtract, Intersect, Exclude }

/// CSS border-image (also the nine-slice of `Draw::border_image`).
pub struct BorderImage {
    /// Image, gradient or solid color (§9.2).
    pub source: FillImage,
    /// Insets into the source: px are source pixels (for gradients and
    /// solid colors, px of the border image area), pct of the source size.
    pub slice: Edges<Len>,
    pub fill: bool,
    /// None: the border widths. pct is of the border image area (§9.2).
    pub width: Option<Edges<Len>>,
    pub outset: Edges<f64>,
    /// Horizontal, vertical.
    pub repeat: [BorderImageRepeat; 2],
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BorderImageRepeat { #[default] Stretch, Repeat, Round, Space }

pub struct Effects {
    pub filter: Vec<FilterOp>,
    pub backdrop: Vec<FilterOp>,
    pub blend: BlendMode,
    /// Replaces `mask: Option<FillLayer>`.
    pub masks: Vec<MaskLayer>,
    pub clip_path: Option<ClipShape>,
}

pub enum FilterOp {
    Brightness(f64), Contrast(f64), Saturate(f64), Grayscale(f64), Sepia(f64),
    HueRotate(f64), Invert(f64), Opacity(f64), Blur(f64),
    DropShadow { offset: Point, blur: f64, color: Rgba },
    /// CSS `url(#filter)`: an SVG filter graph (§5.9), images as assets.
    Graph(std::sync::Arc<FilterGraph<Asset>>),
    /// A filter shader (§11.3).
    Shader(std::sync::Arc<ShaderSpec>),
}
```


### 5.7 text の語彙（`src/text/*.rs`、`src/render/text.rs`）

```rust
// text/properties.rs
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStroke { pub width: TextLen, pub color: Rgba }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PaintOrder { #[default] FillStroke, StrokeFill }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontVariation { pub tag: [u8; 4], pub value: f64 }

pub struct TextStyle {
    // … existing …
    pub stroke: Option<TextStroke>,
    pub paint_order: Option<PaintOrder>,
    pub variations: Option<Vec<FontVariation>>,
    pub optical_sizing: Option<bool>,
    /// CSS font-width (font-stretch) in percent, 50..=200.
    pub font_width: Option<f64>,
}
```

- `ComputedTextStyle` に次の field を追加する。五つとも CSS と同じく継承する。
  - `stroke: Option<TextStroke>`
  - `paint_order: PaintOrder`
  - `variations: Vec<FontVariation>`
  - `optical_sizing: bool`（既定 true）
  - `font_width: f64`（既定 100）
- `SpanProps` に `stroke`、`variations`、`font_width` を追加する（いずれも `Option`）。
- `render/text.rs` の `SpanStyle` に `variations: Option<Vec<FontVariation>>` と `font_width: Option<f64>` を追加する。

### 5.8 描画命令（`src/render/scene.rs`、`src/render/effect.rs`、`src/render/paint.rs`）

`RenderCmd` を次のように変える。

| 変更 | 内容 |
|---|---|
| 削除 | `External`（`LayerItem::External` へ移る）、`PushLayer`/`PopLayer`、`Backdrop`。`Image` と `ImageSampled` の `color_matrix`、`Pattern` の image の `color_matrix`、`Paint::filtered`、`Gradient::filtered`（色の filter は常に compositor の layer で掛けるため。arvane-render の `derived.rs` の filtered の cache も削除する） |
| 追加 | `PushGroup(Group)`、`PopGroup` |
| 変更 | `Glyphs { run, origin, fill: Option<Paint>, stroke: Option<GlyphStroke>, order: PaintOrder }`。fill が None なら `run.color` の単色で塗る |

```rust
/// A group of commands composited as one inside a chunk: clipped, faded and
/// blended, as a Vello layer. Effects that need the group's pixels (filters,
/// masks, backdrops, Porter–Duff operators, 3D) are compositor layers instead.
pub struct Group {
    pub clip: Option<Shape>,
    pub opacity: f64,
    pub blend: BlendMode,
    /// Gaussian blur of the group (std_dev in group space). Only text and
    /// box shadows use it; the `filter` property never lowers to it.
    pub blur: Option<f64>,
}
pub struct GlyphStroke { pub stroke: Stroke, pub paint: Paint }
```

その他の変更:

- `effect.rs` から `Layer`、`LayerFilter`、`Mask`、`Backdrop` を削除する。`BoxShadow` は残す。
- `render::Paint` から `LinearGradient` variant を削除する（`Gradient` で表す）。
- `render::Gradient` の stops は、従来どおり sRGB の `(f64, Rgba)` とし、interpolation を持たない。wire の `Gradient`（`SpaceColor` の stop、hint、`interpolation`）と `CoordinateGradient`（`SpaceColor` の stop、`interpolation`。既定は `ColorInterpolation::SRGB`）は、engine が §9.6 で密な sRGB の stop の列に展開して渡す。Vello は sRGB の stop の間を線形に補間するだけである。

### 5.9 filter graph（`src/effect/filter_graph.rs`、新規）

SVG Filter Effects 1 の filter primitive を型で表す。座標は layer の局所座標（論理 px、`primitiveUnits="userSpaceOnUse"` 相当）である。最後の primitive の結果が出力になる。

型の引数 `I` は Image の画像の表し方で、wire では `Asset`、`Composition` では `Pixels` である。`impl<I> FilterGraph<I> { pub fn map_images<J>(&self, f: impl FnMut(&I) -> J) -> FilterGraph<J>; }` で写す。`validate` は `I` によらない。色はどちらでも解決済みの `Rgba` である（theme の解決は style の resolve で済ませる）。

```rust
//! Role: SVG filter primitive graphs (CSS filter: url()) as values.

/// `I` is how an Image primitive names its image: `Asset` on the wire
/// (`FilterOp::Graph`), `Pixels` in a composition (`LayerFilter::Graph`).
pub struct FilterGraph<I> {
    pub primitives: Vec<Primitive<I>>,
    /// How far the filter region extends beyond the layer bounds; None:
    /// 10% of the bounds' width and height on each side (SVG's −10% / 120%).
    pub margin: Option<Insets>,
    /// color-interpolation-filters of primitives that do not set their own.
    pub color_space: FilterColorSpace,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FilterColorSpace { #[default] LinearRgb, Srgb }
/// Result(i): the output of primitives[i], i below the reading primitive's index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FilterInput { Source, SourceAlpha, Result(u32) }

pub struct Primitive<I> { pub op: PrimitiveOp<I>, pub color_space: Option<FilterColorSpace> }

pub enum PrimitiveOp<I> {
    Blend { top: FilterInput, bottom: FilterInput, mode: BlendMode },
    ColorMatrix { input: FilterInput, matrix: ColorMatrix },
    ComponentTransfer { input: FilterInput, r: TransferFn, g: TransferFn, b: TransferFn, a: TransferFn },
    Composite { top: FilterInput, bottom: FilterInput, op: CompositeOp },
    ConvolveMatrix { input: FilterInput, order: (u32, u32), kernel: Vec<f64>, divisor: f64, bias: f64, target: (u32, u32), edge: EdgeMode, preserve_alpha: bool },
    DiffuseLighting { input: FilterInput, surface_scale: f64, diffuse_constant: f64, color: Rgba, light: Light },
    SpecularLighting { input: FilterInput, surface_scale: f64, specular_constant: f64, specular_exponent: f64, color: Rgba, light: Light },
    DisplacementMap { input: FilterInput, map: FilterInput, scale: f64, x: Channel, y: Channel },
    DropShadow { input: FilterInput, offset: Point, std_dev: (f64, f64), color: Rgba },
    Flood { color: Rgba },
    GaussianBlur { input: FilterInput, std_dev: (f64, f64), edge: EdgeMode },
    Image { image: I, rect: Rect },
    Merge { inputs: Vec<FilterInput> },
    Morphology { input: FilterInput, op: MorphologyOp, radius: (f64, f64) },
    Offset { input: FilterInput, offset: Point },
    Tile { input: FilterInput, tile: Rect },
    Turbulence { base_frequency: (f64, f64), octaves: u32, seed: f64, kind: TurbulenceKind, stitch: bool },
}
pub enum CompositeOp { Over, In, Out, Atop, Xor, Lighter, Arithmetic { k1: f64, k2: f64, k3: f64, k4: f64 } }
pub enum TransferFn { Identity, Table(Vec<f64>), Discrete(Vec<f64>), Linear { slope: f64, intercept: f64 }, Gamma { amplitude: f64, exponent: f64, offset: f64 } }
pub enum Light {
    Distant { azimuth_deg: f64, elevation_deg: f64 },
    Point { position: [f64; 3] },
    Spot { position: [f64; 3], points_at: [f64; 3], exponent: f64, cone_deg: Option<f64> },
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)] pub enum EdgeMode { Duplicate, Wrap, #[default] None }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub enum Channel { R, G, B, A }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub enum MorphologyOp { Erode, Dilate }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub enum TurbulenceKind { FractalNoise, Turbulence }

impl<I> FilterGraph<I> {
    /// The same graph with each Image's `image` mapped by `f`.
    pub fn map_images<J>(&self, f: impl FnMut(&I) -> J) -> FilterGraph<J>;
    /// Errors (messages name the primitive index), the same limits for the CPU and the GPU:
    /// - a Result index that is not below its primitive's index; an empty graph; a non-finite number;
    /// - ConvolveMatrix: order 1..=16 on each axis, kernel length order.0 × order.1,
    ///   target.0 < order.0 and target.1 < order.1 (divisor 0 is allowed here; the plan
    ///   replaces it by the kernel sum, or 1 if the sum is 0, as SVG, once for both executors);
    /// - ComponentTransfer: Table and Discrete 1..=256 values;
    /// - GaussianBlur, DropShadow std_dev >= 0; Morphology radius >= 0; Turbulence
    ///   base_frequency >= 0 and octaves 0..=16; lighting surface_scale finite,
    ///   constants >= 0, specular_exponent 1..=128; Merge 1..=16 inputs;
    /// - at most 64 primitives.
    pub fn validate(&self) -> Result<(), String>;
}
```

`effect.rs` と `lib.rs` は、この節の型を名前の一覧で再公開する（style の `glob-imports` 規則により `*` を使わない）。

### 5.10 shader program（`src/effect/shader.rs`、新規）

```rust
//! Role: Validated shader programs and their uses as plain values.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShaderKind { Paint, Filter }

/// A complete WGSL module assembled by kivact-shader (§11.3).
pub struct ShaderProgram {
    pub kind: ShaderKind,
    pub module: std::sync::Arc<str>,
    pub params: Vec<ParamSlot>,
    /// Size of the params uniform buffer in bytes (a multiple of 16; 16 when there are no params).
    pub params_size: u32,
    /// FNV-1a 64 of `module` (pipeline cache key).
    pub hash: u64,
}
pub struct ParamSlot { pub name: String, pub ty: ParamType, pub offset: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParamType { F32, Vec2, Vec3, Vec4, I32, U32 }

/// How a shader's `time` advances (§11.3.5).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ShaderClock { #[default] Still, Hz(f64), Display }

/// A shader as authored on a node (paint wire): the engine turns it into a
/// `ShaderUse` for each frame (time, decoded images).
pub struct ShaderSpec {
    pub program: std::sync::Arc<ShaderProgram>,
    pub params: std::sync::Arc<[u8]>,
    /// Slots 0..=3 of the WGSL `image(slot, uv)` helper.
    pub images: [Option<Asset>; 4],
    pub clock: ShaderClock,
    /// Keeps the clock running under reduced motion.
    pub essential: bool,
    /// Paint shaders: what the CPU (and a failed pipeline) draws instead; None: nothing.
    /// Never `FillImage::Shader` (resolve reports and drops it).
    pub fallback: Option<FillImage>,
    /// Filter shaders: px the output may extend beyond the layer bounds.
    pub margin: f64,
}

/// A program with its parameter bytes, images and clock for one draw.
pub struct ShaderUse {
    pub program: std::sync::Arc<ShaderProgram>,
    /// Exactly `params_size` bytes, little endian, laid out by `params`.
    pub params: std::sync::Arc<[u8]>,
    /// Slots 0..=3 of the WGSL `image(slot, uv)` helper; None binds a 1×1 transparent texture.
    pub images: [Option<Pixels>; 4],
    /// Seconds of the shader clock (§11.3.5).
    pub time: f64,
    /// Filter shaders: px the output may extend beyond the layer bounds.
    pub margin: f64,
}
```

`ShaderClock` と `ShaderSpec` は `FillImage` を参照するので `src/paint.rs` に置く。`ShaderProgram`、`ParamSlot`、`ParamType`、`ShaderKind`、`ShaderUse` は `src/effect/shader.rs` に置く。module の向きは §5.14 に従う。

### 5.11 Composition（`src/render/composition.rs`、新規）

```rust
//! Role: Layer trees of one frame, handed from the engine to the compositor.

/// One frame: `layers[0]` is the window's root layer (window logical px).
/// Child layers are referenced by index from `LayerItem::Layer`. Layers
/// referenced only from `exports` are drawn into their export sinks.
pub struct Composition { pub layers: Vec<CompositeLayer>, pub exports: Vec<Export> }

/// Stable identity of a layer across frames (cache key).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayerKey {
    Root,
    Node(NodeId),
    /// A layer opened by a Draw recording: the node, its scene slot (0 draw,
    /// 1 draw_over) and the ordinal of the layer within the recording.
    Scene { node: NodeId, slot: u8, ordinal: u32 },
    Drawable(DrawableId),
}

pub struct CompositeLayer {
    pub key: LayerKey,
    /// Layer space → parent layer space (root: → window logical px).
    pub transform: Matrix4,
    /// Content bounds in layer space before filters (raster and effect extent).
    pub bounds: Rect,
    pub plane: Plane,
    pub backface: BackfaceVisibility,
    /// Where the surface's pixel grid sits (§8.3 step 3).
    pub grid: RasterGrid,
    pub effects: LayerEffects,
    pub items: Vec<LayerItem>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Plane { #[default] Flat, Preserve3d }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RasterGrid {
    /// Snap to the target's pixels: crisp, re-rastered when the fraction of
    /// the position changes.
    #[default] Target,
    /// Fixed in layer space: the raster survives any movement; the final
    /// draw places it with bilinear sampling (moving layers).
    Layer,
}

pub struct LayerEffects {
    pub opacity: f64,
    pub blend: BlendMode,
    pub composite: Composite,
    pub filters: Vec<LayerFilter>,
    /// Layer space.
    pub clip: Option<Shape>,
    pub masks: Vec<RenderMask>,
    pub backdrop: Option<RenderBackdrop>,
}
// Default: opacity 1, Normal, SourceOver, no filters, no clip, no masks, no backdrop.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Composite { #[default] SourceOver, SourceIn, SourceOut, SourceAtop, DestinationOver, DestinationIn, DestinationOut, DestinationAtop, Lighter, Copy, Xor }

pub enum LayerFilter {
    Blur { std_dev: f64 },
    DropShadow { offset: Point, std_dev: f64, color: Rgba },
    ColorMatrix(ColorMatrix),
    /// Images resolved to pixels by the engine (§11.2.1).
    Graph(std::sync::Arc<FilterGraph<Pixels>>),
    Shader(ShaderUse),
}

/// A mask layer drawn as a scene in layer space; its alpha (or luminance × alpha) masks.
pub struct RenderMask { pub scene: RenderScene, pub mode: MaskMode, pub composite: MaskComposite }

/// CSS backdrop-filter: what the layer covers in its parent, filtered and
/// clipped to `shape` (layer space).
pub struct RenderBackdrop { pub shape: Shape, pub filters: Vec<LayerFilter> }

pub enum LayerItem {
    /// Vello-drawn commands in layer space.
    Raster(RenderScene),
    /// A child layer, clipped by `clips` (parent layer space, all applied) when drawn into this layer.
    Layer { index: u32, clips: Vec<Shape> },
    External(ExternalItem),
    Shader(ShaderItem),
}

pub struct ExternalItem {
    pub frame: ExternalFrame,
    pub rect: Rect,
    pub source: Option<Rect>,
    /// 2D map of `rect` into layer space.
    pub transform: Transform,
    /// Layer space, all applied.
    pub clips: Vec<Shape>,
    pub opacity: f64,
    pub pixelated: bool,
    /// Pixels the CPU compositor draws instead (Draw::gpu_frame_with_pixels).
    pub cpu: Option<Pixels>,
}

pub struct ShaderItem {
    pub shader: ShaderUse,
    /// The shader's coordinate box (position 0..size in WGSL), in item space.
    pub rect: Rect,
    /// Item space → layer space.
    pub transform: Transform,
    /// White where the shader shows, as a scene in item space (shape, clips).
    pub coverage: RenderScene,
    /// What the CPU compositor draws instead, in item space.
    pub fallback: RenderScene,
    pub opacity: f64,
}

/// Draw `layers[layer]` into the export sink of `sink` at `size` device px
/// (`scale` device px per logical px), answering request `generation` (§8.12, §10.5).
pub struct Export {
    pub layer: u32,
    pub sink: DrawableId,
    pub size: (u32, u32),
    pub scale: f64,
    pub generation: u64,
    pub mode: ExportMode,
}

/// A rectangle of device pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PixelRect { pub x: u32, pub y: u32, pub width: u32, pub height: u32 }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExportMode { Pixels, Gpu }
/// An exported drawable frame (§8.12); the generation travels in ExportedFrame.
#[derive(Clone, Debug, PartialEq)]
pub enum ExportFrame { Pixels(Pixels), Gpu { slot: u32, value: u64 } }
/// One exported frame of the request `generation` of `sink`, from the
/// compositor through arvane and the engine to core unchanged.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportedFrame { pub sink: DrawableId, pub generation: u64, pub frame: ExportFrame }
/// A GPU export pool lost with the device (§8.12), of the request `generation`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportLoss { pub sink: DrawableId, pub generation: u64 }

/// What a presented frame produced besides pixels (§10.6).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct FrameReport {
    pub exports: Vec<ExportedFrame>,
    /// External items drawn without their frame.
    pub missed: Vec<ExternalFrame>,
    /// (program hash, message) of shader pipelines that failed, once each.
    pub shader_failures: Vec<(u64, String)>,
    /// Exports skipped this frame because no slot was released (§8.12).
    pub pending_exports: Vec<DrawableId>,
    pub stats: FrameStats,
}

/// Work a frame did (statistics, S-9), for diagnostics and tests (§8.15).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FrameStats {
    pub chunks_rastered: u32,
    pub pixels_rastered: u64,
    pub surfaces_reused: u32,
    pub damage_pixels: u64,
    pub targets: u32,
}
```

既存の id の置き場所（`NodeId` と同じ module）に `#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)] pub struct DrawableId(pub u64);` を追加する。

`render.rs` は §5.11 の型を名前の一覧で再公開する（glob を使わない）。`kivact-types::render` は state-fields の `value_modules` に既に入っているので、これらの値型に marker は要らない。

### 5.12 scene recording（`src/scene/recording.rs`）

```rust
pub struct DrawingLayer {
    pub clip: Option<DrawingShape>,
    pub area: Rect,
    pub opacity: f64,
    pub blend: BlendMode,
    pub composite: Composite,
    pub filter: Vec<FilterOp>,
    /// Replaces `mask: Option<FillLayer>`.
    pub masks: Vec<MaskLayer>,
    /// A non-affine (3D) transform of the layer about the recording origin;
    /// affine transforms stay PushTransform.
    pub transform: Option<Matrix4>,
    pub transform_style: TransformStyle,
    pub backface: BackfaceVisibility,
}

pub enum DrawCmd {
    // … existing: Pixels, External, ImagePart, Shape, PushLayer, PopLayer,
    // PushTransform, PopTransform, Rect, RoundedRect, StrokeRect, Line,
    // Image, PushClip, PopClip …
    Text(TextRun),
    Paragraph { shaped: ShapedTextSnapshot, origin: Point, fill: Fill, stroke: Option<(Stroke, Fill)> },
    /// A drawable's subtree at `at` under the open transforms (§10).
    Drawable { id: DrawableId, at: Point },
    /// A nine-slice image / border image over `rect`.
    BorderImage { rect: Rect, image: Box<BorderImage> },
}

pub struct TextRun {
    // … existing …
    /// Fill paint; None: `color`.
    pub fill: Option<Fill>,
    pub stroke: Option<(Stroke, Fill)>,
    pub variations: Vec<FontVariation>,
    pub font_width: f64,
}

pub struct Region {
    // … existing …
    /// Replaces `transform: Transform`: region space → recording space.
    pub transform: Projective,
}
```

`Paragraph` の `color: Rgba` は `fill: Fill`（単色なら `Fill::Solid`）に置き換える。

### 5.13 motion の語彙（`src/motion.rs`）

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StepPosition { JumpStart, JumpEnd, JumpNone, JumpBoth }

#[derive(Clone, Debug, PartialEq)]
pub enum Easing {
    Linear,
    /// CSS ease = cubic-bezier(0.25, 0.1, 0.25, 1).
    Ease,
    /// cubic-bezier(0.42, 0, 1, 1).
    EaseIn,
    /// cubic-bezier(0, 0, 0.58, 1).
    EaseOut,
    /// cubic-bezier(0.42, 0, 0.58, 1).
    EaseInOut,
    CubicBezier(f64, f64, f64, f64),
    Steps { count: u32, position: StepPosition },
    /// CSS linear(): (input, output) points with increasing input (§12.1).
    LinearPoints(std::sync::Arc<[(f64, f64)]>),
    BackIn, BackOut, BackInOut,
    ElasticIn, ElasticOut, ElasticInOut,
    BounceIn, BounceOut, BounceInOut,
}
impl Easing {
    pub fn steps(count: u32, position: StepPosition) -> Easing;
    /// Points with input None are spread evenly between their neighbours (CSS linear()).
    pub fn linear_points(points: impl IntoIterator<Item = (Option<f64>, f64)>) -> Easing;
    /// Output for progress t. t is clamped to 0..=1; the output may leave
    /// 0..=1 where the curve overshoots (back, elastic, cubic-bezier).
    pub fn apply(&self, t: f64) -> f64;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub stiffness: f64,
    pub damping: f64,
    pub mass: f64,
    /// Settled when, for every component, √((x − to)² + (v/ω0)²) ≤ rest·a
    /// (a: the run's amplitude, ω0 = √(stiffness/mass); §12.2).
    pub rest: f64,
}
impl Spring {
    // GENTLE, WOBBLY, STIFF, DEFAULT keep their values, with rest 1e-3.
    /// Time to settle and bounce (−1..1) (§12.2).
    pub fn with_duration(duration_ms: f64, bounce: f64) -> Spring;
    pub fn rest(self, rest: f64) -> Spring;
    /// Why this spring cannot run (stiffness ≤ 0, damping < 0, mass ≤ 0,
    /// rest ≤ 0, or a value not finite), if so. Callers report it (§12.2).
    pub fn problem(&self) -> Option<&'static str>;
}

pub enum Kind {
    Tween { ms: f64, easing: Easing },
    Spring(Spring),
    /// (offset 0..=1, components, easing of the segment that starts here)
    Keyframes { frames: Vec<(f64, Vec<f64>, Easing)>, ms: f64 },
}

/// One keyframe of `Transition::keyframes`.
pub struct Keyframe<T> { pub offset: f64, pub value: T, pub easing: Easing }
impl<T> Keyframe<T> {
    pub fn new(offset: f64, value: T) -> Keyframe<T>;    // easing Linear
    pub fn easing(self, easing: Easing) -> Keyframe<T>;
}
impl<T> From<(f64, T)> for Keyframe<T>;
```

その他の変更:

- `Transition::keyframes` の引数を `frames: impl IntoIterator<Item = impl Into<Keyframe<T>>>` にする。
- `Easing::steps_start` と `Easing::steps_end` は削除する。`steps(n, StepPosition::JumpStart)` などへ移行する。
- `Easing` は `Copy` でなくなる。`ScrollBehavior::Smooth(Easing)` など、`Copy` を derive していた型は `Clone` にする。

`lib.rs` の re-export は `pub use motion::{Easing, Keyframe, Kind, MotionClock, PlayDirection, Repeat, Retarget, Spring, SpringRun, StepPosition, Transition, TransitionProp, lerp};` とする。


### 5.14 kivact-types の module の向き

新しい module は、既存の module の向きの規則（`[module_rules.kivact-types.uses]`）に従い、互いに循環しないように置く。

| module | 中身 | uses |
|---|---|---|
| `geometry3d`（新規） | §5.1 | `["geometry"]` |
| `color_space`（新規） | §5.4 | `["color"]` |
| `pixel`（新規） | `Pixels`（`scene/recording.rs` から移す）、`ColorMatrix`（`render/color.rs` から移す）、`BlendMode`（`paint.rs` から移す。filter graph が使うため） | `["asset", "color"]` |
| `effect`（新規） | `effect/filter_graph.rs`（§5.9）、`effect/shader.rs`（§5.10）。`effect.rs` は宣言と再公開だけ | `["asset", "color", "geometry", "pixel"]` |
| `prop_interp`（新規） | §12.9.2 | `["color", "color_space", "geometry", "motion", "paint"]`（使うものだけ） |

既存の module の `uses` に次を加える。

- `paint`: `"color_space"`, `"effect"`, `"geometry3d"`, `"pixel"`
- `render`: `"color_space"`, `"effect"`, `"geometry3d"`, `"pixel"`
- `scene`: `"effect"`, `"geometry3d"`, `"pixel"`
- `text`: `"color_space"`

`Pixels`、`ColorMatrix`、`BlendMode` の正準の path は、root の `kivact_types::Pixels`、`kivact_types::ColorMatrix`、`kivact_types::BlendMode` だけにする。`kivact_types::scene::Pixels` と `kivact_types::render::ColorMatrix` の再公開は削除し、利用箇所の `use` を直す。

FilterGraph の `ColorMatrix` primitive は `ColorMatrix` 型をそのまま使う（`effect` から `pixel` を参照できる）。


### 5.15 その他の wire の変更

| 場所 | 変更 |
|---|---|
| `motion.rs` | `kivact-style` の `TransitionProp` をここへ移し、§12.9 の variant にする。`kivact-style` は `pub use kivact_types::TransitionProp;` で公開する |
| `paint.rs` `PaintProps` | `transition: Option<Arc<Transition>>` を `transitions: Option<Arc<[(TransitionProp, Transition)]>>` にする（`Size` は含めない。`LayoutProps::size_transition` のまま） |
| `paint.rs` `ImageData` | `playing: Option<bool>` を追加する（§9.8） |
| `paint.rs` `PaintVec` | 削除し、§12.9.2 の `prop_interp`/`apply_components`（`PropInterp`、`PropBasis`） に置き換える |
| `semantics`（`render/semantics.rs` の node の bounds） | `bounds: Rect` を `bounds: Option<Rect>` にする。None は「位置を持たない」（drawable の Unplaced など。§10.4）。`arvane-native/src/a11y/conversion.rs` は None のとき AccessKit の bounds を設定しない。virtual と kivact-test の要素の矩形の読み出し（`rect()`）は `Option` を返す既存の形のまま、None を返す |
| `contract`（engine）`LayoutChanged` | `layout: Option<Rect>`（自身の transform を掛けない window の矩形）を追加する（§12.8） |
| `ids` | `DrawableId`（§5.11） |
| `node` の properties（engine の `NodeProperties`） | `drawable: Option<DrawableRole>` を追加する（§10.2） |

---

## 6. engine: Composition の生成

### 6.1 layer にする node（layer 化の規則）

paint list を作るとき（`paint_list()` の再構築時）、engine は描画される各 node について「compositor layer を開くか」を決める。判定には effective paint（state style と motion を反映した値）を使う。

まず、その node 自身の性質から理由を判定する（下表の R1〜R7）。

| ID | 理由 | 条件 |
|---|---|---|
| R1 | 3D | 次のどれか: (a) node から親への写像（§7.1 の `to_parent`）を平坦化したものが affine でない。(b) 親の plane が `Preserve3d` で、node の `ElementTransform::is_2d()` が false、または node の `transform_style` が `Preserve3d`。(c) `backface` が Hidden で、`is_2d()` が false |
| R2 | 画素が要る効果 | `filter` が空でない（色の filter 関数も含む。filter は常に合成後の画素に掛ける）。または `masks` が空でない。または `backdrop` が空でない |
| R3 | 明示 | `will_change` が `Auto` でない |
| R4 | 動き | engine の motion がこの node の transform または opacity を動かしている（§12.9.3 の `StateMotion::moves_layer(id)`）、または layout motion が動いている |

**raster の格子:** layer の理由に R4 があるか、R3 の `will_change` が Transform を含む layer は `RasterGrid::Layer`、それ以外は `RasterGrid::Target` とする（§8.3 の手順 3）。動く layer は小数 px の移動でも raster をやり直さず、止まっている layer は輪郭をぼかさない。例外として、画面の端で切り取られている動く layer は、見える範囲が 256 device px の格子の境界を越えたときに raster をやり直す（§8.3 の手順 2）。
| R5 | 木の上の layer | overlay、fixed（既存の `HostNode::is_layer()`） |
| R6 | drawable | drawable の root（§10）。key は `LayerKey::Drawable(id)` |
| R7 | 3D コンテキストの構成員 | 自身の `transform_style` が `Preserve3d` で、grouping property（§6.4）で平坦にされない |

次に、子孫の性質から昇格させる（下表の P1〜P3）。この判定には、paint list の再構築時に行う一回の下から上への走査（`SubtreeFacts`）を使う。

| ID | 昇格 | 条件 |
|---|---|---|
| P1 | group の中の layer | inline group を作る node（§6.3 の group 条件、clip だけの group は除く）の部分木に、layer、`External` item、`Shader` item のどれかがある |
| P2 | backdrop root | node が CSS の Backdrop Root（opacity < 1、filter、masks、clip_path、blend ≠ Normal、backdrop のどれかを持つ）で、部分木に backdrop を持つ node がある |
| P3 | blend の isolation | §6.3 の isolation を作る node の部分木に blend ≠ Normal の node があり、その node が inline group で表せない（部分木に layer がある） |


`SubtreeFacts` は node ごとに `has_layer: bool`、`has_item: bool`、`has_backdrop: bool`、`has_blend: bool` を持つ。値は paint list の再構築ごとに作り直す純粋 cache（S-9）であり、`PaintList` の field `facts: HashMap<NodeId, SubtreeFacts>` に置く（全 field が false の node は入れない）。paint list の再構築を起こす `Look::stacking` の tuple に、blend、backdrop の有無、isolation、will_change、transform_style、backface、perspective を加える。

root window は常に `LayerKey::Root` の layer である。

### 6.2 layer の空間と変換

layer の局所座標は、その node の border box の左上を原点とする node 自身の座標（node の transform を掛ける前）である。`CompositeLayer::transform` は、layer の局所座標から親 layer の局所座標への 4×4 行列である。

```text
M = Mn.then(T(o)).then(Pp).then(A)
  Mn: node の ElementTransform::matrix(size) に layout motion の affine を合成したもの（§7.1）
  o:  親 layer の中で、node の box の原点がある位置（walk の origin。push 済みの affine の内側の座標）
  Pp: 親 node の perspective の行列 T(q)·perspective(d)·T(−q)。q = p_o + po で、p_o は親 node の box の原点の walk の空間での位置、po は親の box 座標での perspective origin（perspective が無ければ単位行列）
      親 node が layer の外（親 layer の中の、layer でない node）でも同じ
  A:  親 layer の中で開いている PushTransform の合成（affine）
```

layer を開くと、walk は子の origin を (0, 0) から始め、clip を layer の局所座標の `Rect::EVERYWHERE` 相当（後述の cull 規則）にする。

**cull の規則:**

- 写像が affine の layer の中は、既存の規則どおり、window の viewport を局所座標へ逆写像して cull する。
- 写像が affine でない layer（R1）の中は、layer の `bounds` だけで cull する。
- layer の `bounds` は、node の部分木の ink（§6.6）とする。

### 6.3 chunk への分割と inline の group

layer の中の描画は、`CompositionBuilder` が `LayerItem` の列にする。

```rust
pub(crate) struct CompositionBuilder {
    layers: Vec<CompositeLayer>,
    exports: Vec<Export>,
    open: Vec<OpenLayer>,            // innermost last
}
struct OpenLayer {
    index: u32,                      // into layers
    chunk: SceneOut,                 // the Raster item being built
    stack: Vec<Opened>,              // groups open inside this layer, innermost last
    affine: Vec<Transform>,          // the PushTransforms in `stack`, for item spaces
}
/// What an Open entry pushed; Close pops exactly these, innermost first.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Opened {
    Transform(Transform),
    Clip(Rect),
    /// A group with opacity 1, Normal blend and no blur: equivalent to its clip.
    ClipGroup(Shape),
    /// Any other group: never spans an item boundary (P1 guarantees it).
    Group(Group),
    Layer,
}
```

**chunk の分割:**

- 連続する Vello 描画は一つの `LayerItem::Raster` にまとめる。
- 子 layer、`External`、`Shader` の item を出すときは、現在の chunk を閉じる。
  - 開いている `Transform`、`Clip`、`ClipGroup` に対応する pop を chunk の末尾に足し、chunk が空でなければ `Raster` item として追加する。
  - 次に item を追加する。
  - 新しい chunk を始め、開いていた `Transform`、`Clip`、`ClipGroup` を同じ順で push し直す。
- `Group` が開いている間に分割が起きることはない。P1 の昇格がこれを保証するので、builder は debug assert で確かめる。

**item に渡す情報:**

- 子 layer、`External`、`Shader` の item には、開いている clip を `clips: Vec<Shape>` として渡す。
  - 各 clip を `affine` で layer の局所座標へ写した形にする。写像には `Shape::transformed` を使う。
  - `ShaderItem::coverage` は、開いている clip を含む scene として作る。
- `Tint` の alpha（group にしない opacity）は、item の opacity に掛ける。

**inline の group:** CSS の isolated group を作る node は、その部分木に blend ≠ Normal の node があるとき、内容を `Group { clip: None, opacity: 1, blend: Normal, blur: None }` で囲む。isolated group を作る node の条件は次のどれかである。

- opacity < 1
- filter を持つ
- masks を持つ
- clip_path を持つ
- blend ≠ Normal
- isolation Isolate
- transform が none でない
- perspective を持つ
- transform_style Preserve3d
- will_change ≠ Auto

次の node は従来どおり `Group` にする。

- 部分木を持つ opacity < 1 の node
- clip_path を持つ node
- blend ≠ Normal の node

色の filter を各色に掛ける既存の `Tint::matrix` は削除する（`Tint` は alpha だけを持つ）。clamp を含む color matrix は合成と交換できないので、filter は R2 のとおり常に layer の surface に掛ける。

`kivact-types` の `Shape` に `pub fn transformed(&self, t: Transform) -> Shape` を追加する。

- `Rect` は、軸に平行に保たれる affine なら `Rect`、そうでなければ `Polygon` にする。
- `RoundedRect` は、軸に平行な一様 scale と平行移動なら `RoundedRect`、そうでなければ `Path` にする。
- `Polygon` と `Path` は点を写す。

### 6.4 3D 描画コンテキスト

**plane の決まり方:**

- layer の `plane` は、node の `transform_style` が `Preserve3d` で、次の grouping property を一つも持たないとき `Preserve3d` になる。それ以外は `Flat`。
- grouping property は次のとおり（CSS Transforms 2）。
  - overflow の clip（`clip: true`）
  - opacity < 1
  - filter
  - clip_path
  - masks
  - blend ≠ Normal
  - backdrop
  - isolation Isolate

**参加面:**

- `Preserve3d` の layer の子のうち、layer であるもの（R1〜R7 のどれかに当たるもの）が参加面になる。R1 の (b) により、2D でない transform を持つ子と `Preserve3d` の子は必ず layer になる。
- **clip の内側の子は参加面にしない:** root と子の layer の間に clip がある子（`LayerItem::Layer { clips }` の clips が空でない子。layer でない祖先の overflow の clip、clip path、Draw の recording の clip の内側に開かれた layer）は、参加面にしない。その子は、root の平面の内容として、root の平面に平坦化して描く（clip はその平面の上の通常の 2D の clip である）。CSS で、clip を持つ要素が grouping property となって子孫を平坦にするのと同じ結果である。その子の plane が `Preserve3d` なら、その子が新しい 3D コンテキストの root になる。
  - engine の hit test（§7.3）と drawable の配置（§10.3）と a11y（§7.4）も、同じ規則で参加面を決める（clips が空かどうかは、engine が `Composition` を作るときの同じ事実である）。
- layer でない子（2D の transform だけを持つ子）は、親と同じ平面にあるので、親の平面の内容として親の chunk に DOM 順で描く。

**backface:**

- 参加面の `backface` は node の `backface` をそのまま使う。
- 参加面でない layer の `backface` も、node の値を使う（§8.5 で、自身の写像の向きで判定する）。

### 6.5 node の描画順（layer の中）

layer を開いた node の描画は、既存の `paint_open`/`paint_close` と同じ順序である。ただし次の効果は `LayerEffects` に移し、chunk には出さない。

| 既存の出力 | 移動先 |
|---|---|
| `PushTransform`（node 自身の transform） | `CompositeLayer::transform` |
| `PushLayer`（clip_path、mask、group opacity、blend、filter） | `LayerEffects::{clip, masks, opacity, blend, filters}` |
| `Backdrop` | `LayerEffects::backdrop`（shape = 角の丸めを含む border box） |

chunk に出すのは次のものである（局所座標で出す）。

- box shadow、背景、border、border image、focus ring
- presence clip、overflow clip
- Draw scene、text、image、input
- 子
- draw_over、scrollbar

overlay と fixed の layer（R5）は、root layer の item の末尾に、既存の `layers()` の順で追加する。

### 6.6 layer の bounds

`bounds` は、layer の内容の範囲を node 自身の座標で表したものであり、node 自身の効果の到達範囲（filter、backdrop）を含まない。

- **既存の `ink()`:** 次の順で求めている。
  1. box ∪ content ∪ scene の範囲を、text と overflow の範囲だけ広げる。
  2. 子の ink を合わせる。
  3. 自身の `look.fx`（filter の到達範囲）だけ広げる。
  4. 自身の transform で写す。
- **`ink_local(host, id, n, layout, generation)`:** `paint/list/ink.rs` に追加する。手順 3 と 4 を除いた値（自身の効果を含まない、変換前の範囲）を返す。子の ink は子の効果を含むので、そのまま合わせる。
- **書き換え:** `ink()` は `ink_local` に手順 3 と 4 を掛ける形に書き換える（一つの計算を二度持たない）。
- **値の使い分け:**
  - `CompositeLayer::bounds` には `ink_local` を使う。compositor は、§8.3 の手順 2 で効果の到達範囲を足す。
  - cull（§6.2）には、効果を含む `ink()` を使う。画面外の図形から画面内へ伸びる blur や shadow を消さないためである。

### 6.7 Draw recording の中の layer

`DrawCmd::PushLayer(DrawingLayer)` を lower するとき、次のどれかに当たれば `LayerKey::Scene { node, slot, ordinal }` の子 layer を開く。

- `transform` が Some（3D）
- filter を持つ
- `masks` が空でない
- `composite` が SourceOver でない
- 内容（push から対応する pop まで）に `DrawCmd::Drawable`、`DrawCmd::External`、shader の塗り（`FillImage::Shader`）のどれかを含む（Group の途中で chunk が分割されないようにする）

当たらない場合は `Group`（clip、opacity、blend）に lower する。ordinal は、その recording の中で `PushLayer` が現れた順番（0 から）である。

scene cache（`lower/scene_cache.rs`）は、子 layer、External、Shader の item のどれも生まない recording だけを cache する。それ以外の recording は毎回 lower する。

`DrawingLayer::transform_style` が `Preserve3d` なら、その layer の `plane` を `Preserve3d` にする。recording の中で入れ子になった 3D layer は参加面になる（§6.4 と同じ規則）。

### 6.8 外部 frame と shader paint

- **`DrawCmd::External`:** `LayerItem::External` にする。既存の `Purpose::{Window, Cpu}` は削除し、`cpu_pixels` は常に `ExternalItem::cpu` に入れる。`lower/context.rs` の `Purpose` と、`lower/scene/raster.rs` の分岐を削除する。
- **shader の fill（`FillImage::Shader`）:** 背景や Draw の shape の塗りとして現れたら、`LayerItem::Shader` にする。
  - `rect` は fill layer の配置矩形（§9.4 の background positioning area、Draw では shape の bounds）。
  - `coverage` は、shape を白で塗る scene（開いている clip を含む）。
  - `fallback` は、同じ shape を `ShaderSpec::fallback` の paint で塗る scene（§11.3.6）。fallback が None なら空の scene。

### 6.9 engine から arvane への受け渡し

| 変更前 | 変更後 |
|---|---|
| `Model::scene(engine) -> RenderScene` | `Model::composition(engine) -> Composition` |
| `paint_scene_list(engine, Purpose)` | `compose_list(engine, list: &PaintList, pass: PaintPass) -> Composition` |
| `Host::present` が `window.present(&scene, plan)` | `window.present(&composition, plan)` |

```rust
/// The parameters of one walk over a paint list; nothing in PaintState is
/// changed for its duration (replaces the temporary scale/list swaps).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PaintPass { pub(crate) scale: f64 }
```

- **snapshot（`paint/snapshot.rs`）:** `subtree_paint_list(id, rect)` で別の `PaintList` を作り、`compose_list(engine, &list, PaintPass { scale: 1.0 })` を呼ぶ。`self.paint.list` と `self.paint.scale` は書き換えない。
- **raster zoom:** `PaintState::raster: Cell<f64>` の一時的な書き換えも同様に廃止し、walk の引数で渡す。
- **export:** `Composition::exports` は §10.5 で engine が追加する。

---

## 7. engine: 射影写像による幾何（hit test、a11y、rect）

描画と同じ写像を hit test、pointer の局所座標、`NodeRef::rect`、a11y、caret と IME の位置に使う（G-07）。2D affine だけを前提にした `model/transform.rs` の関数を、以下で置き換える。

### 7.1 node から親への写像

```rust
// model/transform.rs
impl Model {
    /// The node's own 4×4 about its box: ElementTransform::matrix(size),
    /// then the layout motion's affine (as today, element then layout).
    pub(crate) fn local_matrix(&self, id: NodeId, size: Size) -> Option<Matrix4>;
    /// Node box space → parent box space:
    /// local_matrix.then(T(location − parent scroll + sticky shift)).then(parent perspective matrix).
    /// None for a non-finite or singular result (the node and its subtree are not drawn or hit).
    pub(crate) fn to_parent(&self, id: NodeId) -> Option<Matrix4>;
    /// Node box space → window logical px, flattened per CSS (§7.2).
    pub(crate) fn window_map(&self, id: NodeId) -> Option<Projective>;
    /// The same up to `upto` (exclusive), for scroll_into_view and overlay anchors.
    pub(crate) fn map_between(&self, id: NodeId, upto: Option<NodeId>) -> Option<Projective>;
    /// Window bounds of the mapped border box (replaces `bounds`).
    pub(crate) fn bounds(&self, id: NodeId) -> Option<Rect>;
    /// Window point → node box space (replaces `to_own`, `to_local`).
    pub(crate) fn to_local(&self, id: NodeId, at: Point) -> Option<Point>;
}
```

`to_parent` の親 perspective 行列は Pp = T(po)·Matrix4::perspective(d)·T(−po) である。po は親の `perspective.origin` を親の border box の大きさで解決した点で、d は `perspective.distance` である。親が perspective を持たなければ単位行列とする。

overlay と fixed の root は、既存のとおり window 原点から始まる。祖先の写像は掛けない。

### 7.2 平坦化と 3D コンテキストを含む合成

`window_map` は次のように合成する。

1. node から root（または overlay・fixed の root）まで、`to_parent` の列 [M_0（node 自身）, M_1, …] を作る。
2. 下から順に畳み込む。
   - acc4 を 4×4 の累積、acc を 3×3 の累積とする。
   - 段 k で、M_k の親の plane（§6.4。layer でない親は Flat）が `Preserve3d` なら、acc4 = acc4.then(M_k) とし、4×4 のまま続ける。
   - Flat なら、acc = acc.then(acc4.then(M_k).flatten()) とし、acc4 を単位行列に戻す。
3. 最後に acc を返す。

これは CSS Transforms 2 の accumulated 3D transformation matrix と、平坦化の規則に一致する。

### 7.3 hit test

`hit_node` は、点を親の box 座標で受け取り、node の box 座標へ写して判定する。

- **3D コンテキストの外:** p_local = to_parent(id).flatten().invert()?.apply(p_parent) で写す。写像が逆を持たない、または点が viewer の後ろにある（apply が None）なら、node とその部分木は当たらない。
- **3D コンテキストの root（plane が Preserve3d の layer の node）:** 次の手順で判定する。
  1. コンテキストの参加面をすべて集める（root 自身の平面を含む。§6.4 の規則で再帰する）。各参加面について、root の親の box 座標への 4×4 の M_p（root の to_parent を含み、平坦化しない）を作る。
  2. 各参加面について H = M_p.flatten() とし、p_local = H⁻¹(p) を求める。次の場合は候補にしない。
     - p_local が None
     - p_local が参加面の box（とその clip）の外
     - backface が Hidden で M_p.shows_back()（§8.5）
  3. 候補の奥行き z を求める。q = M_p · (p_local.x, p_local.y, 0, 1) とし、z = q.z / q.w とする。z の大きい順（viewer に近い順）に並べる。z が等しい候補は、描画順（DOM 順）の後のものを先にする。参加面が `MAX_CONTEXT_PLANES` を超えるコンテキストでは、z を使わず DOM 順の後のものから並べる（§8.14 の平坦な重なり）。
  4. 並べた順に、参加面の中を通常の 2D の hit test で探し、最初に当たった node を返す。参加面の中では、参加面でない子孫を通常どおり判定する。
- **backface:** backface が Hidden で、`to_parent(id).shows_back()`（3D コンテキストの参加面では M_p.shows_back()）の node は描画されず、hit もしない（§8.5 と同じ判定）。
- **clip:** clip path、overflow の clip、region は、写した局所座標で既存の判定を使う。

### 7.4 a11y の bounds

`a11y/build/node.rs` の `a11y_transform` は、次のように変える。

- **写像が affine の場合:** window_map(id).as_affine() が Some で、親の写像も affine なら、既存の規則どおり、親に対する相対 transform を AccessKit に渡す。
- **affine でない場合:** その node の `bounds` を、最も近い affine の祖先の座標で表した矩形として渡し、transform は単位行列にする。最も近い affine の祖先とは、window_map が affine の祖先である（無ければ root）。子孫も同じ規則に従う。各 node は自分の射影 bounds を持つ。

既存の非対称（layout motion を a11y だけが考慮しない）は、`window_map` が layout motion を含むことで解消する。

### 7.5 Region と drawable の写像

- `Region::transform` は `Projective` になる（§5.12）。canvas の局所座標から region の座標への判定は、その逆写像を使う。a11y の bounds は、region の矩形の射影 bounds である。
- region と drawable の写像の基準は、scene を描く canvas（padding box）の原点である。既存の `Reader::region_bounds` は border box の原点を基準にしていたので、padding box の原点に直す（観察済みのずれの解消）。

### 7.6 `Place.transformed` の一本化

- `Place.transformed` を書くのは `update_place`（`tree/node.rs`）だけにする。値は次のいずれかのとき true である。
  - base paint の transform が none でない
  - state delta のどれかが transform を持つ
  - state motion の現在値の transform が none でない
  - 親が perspective を持ち、node の transform が 2D でない
- state motion（`state_motion/steps.rs`）は、`n.place.transformed` を直接書かず、`self.tree.update_place(id)` を呼ぶ。
- `transform_between` は `map_between` に置き換える。

---

## 8. arvane-compose: 合成

### 8.1 module 構成と公開 API

```text
crates/arvane-compose/src/
  lib.rs              Role、module の宣言、公開 API の再公開だけ
  cache.rs            CacheIndex: 前フレームの chunk と layer の記録（純粋 cache）
  plan.rs             宣言と再公開だけ
  plan/ops.rs         Plan、Op、DrawOp、ShaderOp、Source、TargetId、Sampling、Scissor、Damage、Redraw、ChunkTask、PlanError（データだけ。§8.2）
  plan/frame.rs       処理の leaf が共有する型と free 関数（Dest、ItemCtx、Geometry、Part、px_map、area、coverage の scene）
  plan/builder.rs     Builder（集約の型）と build の入口
  plan/validate.rs    Composition の検査
  plan/layers.rs      layer の評価、surface、raster scale、可視領域（§8.3。impl Builder）
  plan/items.rs       item の評価（§8.3 の手順 6。impl Builder）
  plan/context3d.rs   3D コンテキストの収集と BSP（§8.4。impl Builder）
  plan/polygon.rs     同次座標の切り取り、凸多角形、BSP の分割
  plan/bounds.rs      RenderCmd の bounds の索引（arvane-render/src/backdrop.rs の Index を移す）
  plan/hash.rs        RenderCmd、LayerEffects、item の 64 bit hash（FNV-1a）
  plan/damage.rs      chunk の差分と window damage（§8.6。impl Builder）
  cpu.rs              宣言と再公開だけ
  cpu/compositor.rs   CpuCompositor、CpuOutput、Rendered（公開）と Op の振り分け
  cpu/surface.rs      Pixmap（premultiplied RGBA8）と標本化
  cpu/draw.rs         DrawOp の実行（標本化、coverage、blend、composite、color matrix）
  cpu/blend.rs        blend mode と Porter–Duff の式（§8.7）
  cpu/blur.rs         gaussian blur（§8.8.1）
  cpu/effects.rs      drop shadow、color matrix、blur の σ（filters と graph が共有する helper）
  cpu/filters.rs      Filter op の実行（graph と shader の振り分け）
  cpu/graph.rs        宣言と再公開だけ
  cpu/graph/apply.rs  filter graph の評価の入口と primitive の振り分け
  cpu/graph/*.rs      filter graph の primitive（resvg 0.46.0 の src/filter/ を移植。§8.8.4）
  cpu/mask.rs         mask と clip の coverage
  gpu.rs              （feature gpu）宣言と再公開だけ
  gpu/state.rs        GpuContext、GpuEnv、GpuSurface、Presentation（集約の型。passive state の leaf だけを参照する）
  gpu/context.rs      adapter と device（移す）
  gpu/surface.rs      swapchain（移す。最後の合成を swapchain に描く）
  gpu/external.rs     外部 frame の import（移す）
  gpu/shared_images.rs（移す）
  gpu/offscreen.rs    読み戻し（移す）
  gpu/targets.rs      surface と chunk の texture の pool（passive state）
  gpu/pipelines.rs    内部の pipeline の表（passive state）
  gpu/pass.rs         処理の leaf が共有する helper（render pass の開始、view、uniform、copy、unpremultiply、FrameTextures）
  gpu/exec.rs         Op の振り分け（impl GpuContext）
  gpu/draw.rs         DrawOp の pipeline（draw.wgsl）
  gpu/filters.rs      blur、drop shadow、color matrix の pipeline（impl GpuContext）
  gpu/graph.rs        filter graph の pipeline（graph/*.wgsl。impl GpuContext）
  gpu/shader_cache.rs user shader の pipeline の cache（passive state）
  gpu/shader.rs       user shader の描画（impl GpuContext）
  gpu/export_pool.rs  drawable の export pool の表（passive state。feature gpu-interop）
  gpu/export.rs       drawable の export の処理（impl GpuContext。feature gpu-interop）
  gpu/wgsl/*.wgsl     内部の shader（include_str!）
```

`gpu/state.rs` は passive state の leaf（`targets`、`pipelines`、`shader_cache`、`export_pool`、`external` の表の型）だけを import する。処理の leaf（`exec`、`draw`、`filters`、`graph`、`shader`、`export`）は、`state` と passive state の leaf と `pass` を import し、互いに path で参照しない。plan と cpu も同じ形にする（`plan/builder.rs` は `plan/ops.rs`、`plan/frame.rs`、`plan/validate.rs` だけを import する）。

`ChunkRecord::retain` と `LayerRecord::retain` は、hash に畳み込んだ pointer の寿命を保つために持つだけで、読まない。この二つの field には `#[expect(dead_code, reason = "held so the hashed pointers stay valid (§8.6)")]` を付ける。他の field と variant は、読まれない（strict な Clippy の `dead_code`）なら、本書の配線が未完了のしるしなので、配線して解消する。

module の一覧に `gpu_unbuilt.rs`（feature `gpu` が無いときの空の型。`arvane-render` から移す）を加える。

```rust
// lib.rs (re-exports only)
pub use arvane_render::RenderError as ComposeError;
pub use cpu::{CpuCompositor, CpuOutput, Rendered};
pub use plan::Damage;
pub use kivact_types::render::{ExportFrame, ExportedFrame, FrameReport, PixelRect};
#[cfg(feature = "gpu")] pub use gpu::{GpuContext, GpuEnv, GpuReport, GpuSurface, Presentation};
#[cfg(not(feature = "gpu"))] pub use gpu_unbuilt::{GpuContext, GpuEnv, GpuReport, GpuSurface, Presentation};
pub const GPU_BUILT: bool = cfg!(feature = "gpu");

// plan.rs
#[derive(Clone, Debug, PartialEq)]
pub enum Damage { Full, Rects(Vec<PixelRect>) }

// cpu.rs
pub struct CpuCompositor { /* raster: arvane_render::CpuRaster, cache: CacheIndex, surfaces, frame: Pixmap */ }
pub struct CpuOutput<'a> {
    /// The window frame: width × height premultiplied RGBA8, kept across frames.
    pub rgba: &'a [u8],
    pub width: u32,
    pub height: u32,
    pub damage: Damage,
    /// Exports, missed external frames, stats (shader_failures is always empty on the CPU).
    pub report: FrameReport,
}
pub struct Rendered { pub image: CpuFrame, pub report: FrameReport }

impl CpuCompositor {
    pub fn new() -> CpuCompositor;
    pub fn release_caches(&mut self);
    /// One window frame of physical `size` at `scale`.
    pub fn frame(&mut self, composition: &Composition, size: (u32, u32), scale: f64) -> Result<CpuOutput<'_>, ComposeError>;
    /// A one-off image (snapshot, Offscreen), with a fresh cache.
    pub fn render(&mut self, composition: &Composition, size: (u32, u32), scale: f64) -> Result<Rendered, ComposeError>;
    /// Whether the next frame must redraw everything (after a resize or a lost buffer).
    pub fn invalidate(&mut self);
}

/// = kivact_types::render::FrameReport (§10.6); the GPU fills all three fields.
pub type GpuReport = FrameReport;
// GpuContext keeps every existing method moved from arvane-render
// (new, release_caches, describe, interop_adapter, failure, check,
// configure, surface, surface_maker, read_back, present_pixels,
// import_external, drop_external, take_missed_external) and changes:
impl GpuContext {
    pub fn present(&mut self, surface: &mut GpuSurface, composition: &Composition, scale: f64, physical: (u32, u32))
        -> Result<(Presentation, std::time::Duration, GpuReport), ComposeError>;
    /// GPU offscreen render with readback (tests and GPU snapshots).
    pub fn render(&mut self, composition: &Composition, size: (u32, u32), scale: f64) -> Result<CpuFrame, ComposeError>;
    /// Without the `gpu-interop` feature: an error (arvane checks `GPU_INTEROP_BUILT` first).
    pub fn export_drawable(&mut self, sink: DrawableId, generation: u64, adapter: &GpuAdapter, size: (u32, u32)) -> Result<SharedTextures, ComposeError>;
    /// Without the `gpu-interop` feature: does nothing.
    pub fn drop_export(&mut self, sink: DrawableId);
}
```

`export_drawable` と `drop_export` は、feature の組み合わせ（`gpu` だけ、`gpu-interop`、どちらも無い）のすべてで同じ署名を持つ。`gpu-interop` の無い build では、中身を error を返す処理と何もしない処理にする。`gpu` の無い build の `gpu_unbuilt.rs` にも同じ署名の stub を置く（既存の外部 image の import の `shared_images` と `gpu_unbuilt` の形と同じ）。

**feature の有無の判定:** compose は既存の `GPU_BUILT` と並べて `pub const GPU_INTEROP_BUILT: bool`（feature `gpu-interop` のときだけ true）を公開する。arvane の presenter の `export_drawable` は、次の順で結果を決める（error の文字列で判定しない）。

1. `!arvane_compose::GPU_INTEROP_BUILT`: `Failure { kind: FailureKind::Unavailable, .. }`（core は `Failed` にする。feature が無いので待っても変わらない）。
2. presenter が GPU の device を持たない: `Failure { kind: FailureKind::NotReady, .. }`（core は `GpuWaiting` にする）。
3. それ以外: `GpuContext::export_drawable` を呼び、error は既存の `error::render` で `Failure` にする（core は `Failed` にする）。

### 8.2 描画計画の型（`plan.rs`、`cache.rs`）

CPU と GPU の executor は、次の型だけを見て描く。`plan` module は backend を知らない。cache の key と記録は `cache.rs` に置き、`plan` が `cache` を参照する（§4.4 の module rules）。

```rust
// cache.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ChunkRole {
    /// The n-th Raster item of the layer.
    Content(u32),
    /// Clips of the n-th item (LayerItem::Layer / External).
    Clip(u32),
    /// The layer's own clip (LayerEffects::clip).
    LayerClip,
    /// The n-th RenderMask.
    Mask(u32),
    /// The coverage and fallback of the n-th Shader item.
    Coverage(u32),
    Fallback(u32),
    /// The backdrop shape.
    Backdrop,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ChunkKey { pub layer: LayerKey, pub role: ChunkRole }
pub(crate) struct ChunkRecord { pub scale: f64, pub origin: Point, pub size: (u32, u32), pub index: std::sync::Arc<[(u64, Option<PixelRect>)]>, pub frame: u64 }
pub(crate) struct LayerRecord { pub signature: u64, pub own: u64, pub window_bounds: Option<Rect>, pub frame: u64,
    /// Arcs whose pointers the signature folds in outside chunks: FilterGraph Image
    /// pixels, ShaderUse images, ExternalItem::cpu (same rule as ChunkRecord::retain).
    pub retain: Vec<Retained> }
#[derive(Default)]
pub(crate) struct CacheIndex {
    pub chunks: HashMap<ChunkKey, ChunkRecord>,
    pub layers: HashMap<LayerKey, LayerRecord>,
    /// Frames drawn so far (cache stamps; statistics, not a phase).
    pub frame: u64,
}

// plan.rs
pub(crate) type TargetId = u32;   // index into Plan::targets; 0 = the output

pub(crate) struct Plan<'a> {
    pub targets: Vec<TargetSpec>,
    /// Chunks to make current before `ops` run (raster work of this frame).
    pub chunks: Vec<ChunkTask<'a>>,
    pub ops: Vec<Op<'a>>,
    pub damage: Damage,
    pub exports: Vec<(DrawableId, TargetId)>,
    /// Records to store into the cache after a successful frame.
    pub chunk_records: Vec<(ChunkKey, ChunkRecord)>,
    pub layer_records: Vec<(LayerKey, LayerRecord)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TargetKind {
    /// A scratch target of this frame.
    Temp,
    /// A layer surface kept across frames; `reuse`: its content is current
    /// and the ops that would draw it were left out.
    Kept { key: LayerKey, signature: u64, reuse: bool },
}
pub(crate) struct TargetSpec { pub size: (u32, u32), pub kind: TargetKind }

pub(crate) struct ChunkTask<'a> {
    pub key: ChunkKey,
    /// Borrowed from the Composition (Raster items, mask scenes) or built by
    /// the plan (clip, coverage, fallback and backdrop scenes).
    pub scene: std::borrow::Cow<'a, RenderScene>,
    /// Chunk px = (layer point − origin) × scale.
    pub origin: Point,
    pub scale: f64,
    pub size: (u32, u32),
    pub redraw: Redraw,
    /// Per-command (hash, chunk-px bounds), stored into the cache.
    pub index: std::sync::Arc<[(u64, Option<PixelRect>)]>,
}
pub(crate) enum Redraw { All, Rects(Vec<PixelRect>) }

pub(crate) enum Source<'a> {
    Chunk(ChunkKey),
    Target(TargetId),
    /// GPU: the imported frame; CPU: `cpu` pixels or nothing.
    External { frame: ExternalFrame, cpu: Option<&'a Pixels>, source: Option<Rect> },
    Pixels(&'a Pixels),
}

pub(crate) enum Op<'a> {
    /// Clear `rect` of `target` (None: all of it) to transparent.
    Clear { target: TargetId, rect: Option<PixelRect> },
    Draw(DrawOp<'a>),
    /// `output` = filter(`input`); both have the same size and space.
    Filter { input: TargetId, output: TargetId, filter: &'a LayerFilter, px_per_unit: f64, origin: Point },
    /// Multiply `target` by the combined mask of `masks` (bottom layer last).
    Mask { target: TargetId, masks: Vec<(ChunkKey, MaskMode, MaskComposite)> },
    /// Multiply `target` by the coverage of a chunk (clip paths).
    Clip { target: TargetId, coverage: ChunkKey },
    /// GPU only; the CPU executes `fallback` instead (§8.10).
    Shader(ShaderOp<'a>),
}

// xtask-allow(state-fields): One draw command's independent optional inputs, not lifecycle state.
pub(crate) struct DrawOp<'a> {
    pub target: TargetId,
    pub source: Source<'a>,
    /// Destination polygon in target px (convex).
    pub polygon: Polygon,
    /// Edges of `polygon` that are BSP splits (no antialiasing, top-left rule).
    pub split_edges: Vec<usize>,
    /// Target px → source space: chunk/target/Pixels px, or for External the
    /// item rect space (the executor turns it into texels by the frame size).
    pub map: Projective,
    pub opacity: f64,
    pub blend: BlendMode,
    pub composite: Composite,
    /// Smooth (bilinear) or Nearest.
    pub sampling: Sampling,
    /// Coverage multiplied in (item clips), in target px.
    pub coverage: Option<ChunkKey>,
    /// Off, or Rects(..): pixels of `target` outside them stay as they are (window damage).
    pub scissor: Scissor,
}
// Sampling { Smooth, Nearest }, Scissor { Off, Rects(Vec<PixelRect>) }; the
// build of an export plan carries Caching { Cached, Uncached } (no reuse, no record).

// xtask-allow(state-fields): One shader draw's independent optional inputs, not lifecycle state.
pub(crate) struct ShaderOp<'a> {
    pub target: TargetId,
    pub shader: &'a ShaderUse,
    pub polygon: Polygon,
    /// Target px → shader space (logical px from the shader rect's origin).
    pub map: Projective,
    pub size: Size,
    pub px_per_unit: f64,
    pub coverage: ChunkKey,
    /// The CPU draws this as a DrawOp with the same opacity.
    pub fallback: ChunkKey,
    /// Filter shaders read this target as `content`.
    pub input: Option<TargetId>,
    pub opacity: f64,
}

/// The plan's own error, independent of arvane-render; executors turn it
/// into `ComposeError::InvalidComposition(message)`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlanError { pub message: String }
pub(crate) fn build<'a>(composition: &'a Composition, size: (u32, u32), scale: f64, cache: &CacheIndex, gpu: bool) -> Result<Plan<'a>, PlanError>;
```

**plan が作る scene（coverage の scene）:**

- 各 shape を `RenderCmd::FillShape { paint: Solid(WHITE) }` で一つ塗る scene として作る。
- clip が複数あるときは、外側から順に `PushGroup(Group { clip: Some(shape), .. })` で入れ子にし、最も内側で全体を白で塗る（交わりになる）。
- `Coverage(n)` と `Fallback(n)` の scene は、`item.coverage` と `item.fallback` の全体を `PushTransform(item.transform)` で囲んだものとする。

**画素の受け渡しの規則:** `Pixels`（`ShaderUse::images`、`Source::Pixels`、`ExternalItem::cpu`、filter graph の `Image`）は straight alpha である。CPU も GPU も、読み込み時（GPU は upload 時）に premultiply してから使う。

**hash の規則（`plan/hash.rs`）:**

- 数値の f64 は `to_bits` で畳み込む。
- enum は discriminant の番号を最初に畳み込む。
- `Pixels`、`GlyphRunSnapshot`、`AssetBytes`（`Pixels::rgba`、`GlyphRun::font`）は、Arc の pointer と長さで畳み込む。pointer を identity として使えるように、`ChunkRecord` は、その chunk の command が参照したこれらの Arc の clone を `retain: Vec<Retained>` に保持する。記録が生きている間は、allocator が同じ address を別の内容に再利用しない。既存の `AtlasImage::_pixels` と同じ考え方である。

```rust
pub(crate) enum Retained { Bytes(AssetBytes), Glyphs(GlyphRunSnapshot) }
// ChunkRecord gains: pub retain: Vec<Retained>
```

- `Asset`（path または static、shared）は、既存の `Asset` の `Hash` を使う。

### 8.3 layer の評価（`plan/layers.rs`）

`build` は root layer を target 0 に評価する。target 0 の大きさは physical size で、layer 空間から target px への写像は scale(scale) である。

`emit(layer, target T, to_t: Matrix4)` は次の順で行う。`to_t` は layer 空間から T の px への写像である。

1. **表裏:** H = to_t.flatten() とする。`layer.backface` が Hidden で `layer.transform.shows_back()` なら何も出さない（§8.5。参加面は §8.4 の手順 2）。
2. **surface の範囲と可視判定:**
   - reach = layer.effects の filters と backdrop の到達距離の和とする（layer 空間）。各 filter の到達距離は次のとおり。
     - Blur: 3σ
     - DropShadow: |offset| + 3σ
     - Graph: margin の四辺の最大値（None のときは max(bounds.width, bounds.height) の 10%）
     - Shader: margin
   - surface_rect = layer.bounds を reach だけ広げたものとする。
   - **cull:** H.map_rect(surface_rect)（効果の到達範囲を含む）が None、または T の矩形と交わらなければ何も出さない。layer.bounds（効果を含まない）では cull しない。
   - H が逆を持ち、T の矩形を H⁻¹ で写した多角形がすべて viewer の前にあるなら、その bounds を reach だけ広げたものと surface_rect の交わりを surface_rect にする（見えない部分を描かない）。`layer.grid` が `RasterGrid::Layer` のときは、交わりの矩形を layer 空間の 256/s の格子へ外側に広げてから使う。これで、画面の端をまたいで少し動いても surface_rect は変わらず、格子の境界を越えたときだけ raster をやり直す（契約の例外として §6.1 の raster の格子に書く）。
3. **raster scale:**
   - s_raw = H.max_scale(surface_rect)（layer 空間 1 単位あたりの T px）とする。
   - H が軸に平行な affine（回転と skew を含まず、x と y の scale が正）なら、s = s_raw とする。さらに、`layer.grid` で次のように決める。
     - `RasterGrid::Target`: surface_rect.origin を、H で写した位置が T の整数 px になるように切り下げ（snap）、平行移動が整数 px なら手順 9 の Draw を `sampling: Sampling::Nearest` にする（overlay などの 2D の layer の内容をぼかさない）。
     - `RasterGrid::Layer`: surface_rect.origin を layer 空間の 1/s の格子に切り下げる（H によらない）。平行移動の小数部は手順 9 の Draw の写像に残し、bilinear で標本化する（整数 px のときだけ `sampling: Sampling::Nearest`）。H の平行移動だけが変わっても、chunk の origin、scale、size は変わらない。
   - それ以外は、s = 2^(⌈2·log2(s_raw)⌉ / 2)（√2 刻みで切り上げ）とする。
   - s を [1/16, 16] に収める。
   - surface の大きさ (⌈w·s⌉, ⌈h·s⌉) のどちらかが 8192 を超えるなら、超えなくなるまで s を √2 で割る。
   - root（target 0）では s = scale のまま。surface を作らず、T に直接描く。
4. **surface を作るか:** root 以外の layer は、常に自分の surface S（`TargetKind::Kept`）を持つ。cache の signature が一致すれば `reuse: true` とし、手順 5〜7 の op を出さない（§8.6）。ただし、自身または子孫の layer が `effects.backdrop` を持つ layer は reuse しない（backdrop の画素は、signature に含まれない下の内容に依存するため）。
5. **backdrop:** `effects.backdrop` があれば、次の op を出す。
   1. T の内容を S の座標へ写して読む `Draw { target: B, source: Target(T), map: (to_t ∘ Ms⁻¹), .. }`。B は S と同じ大きさの一時 target、Ms = scale(s)·T(−surface_rect.origin) である。
   2. backdrop の filters を順に `Filter` op で掛ける。
   3. backdrop shape の coverage chunk（`ChunkRole::Backdrop`）で clip した `Draw { target: S, source: Target(B') }`。
6. **item:** 順に出す。
   - `Raster(scene)`: chunk task（key = (layer.key, Content(n))、origin = surface_rect.origin、scale = s、size = S の大きさ）と、`Draw { target: S, source: Chunk, map: 単位, polygon: S 全体 }`。
     - item が一つの `Raster` だけで、effects がすべて既定の layer では、chunk をそのまま S として扱い、Draw を出さない（GPU も CPU も、Source::Chunk を S の代わりに参照する）。
   - `Layer { index, clips }`:
     - layer.plane が `Preserve3d` のとき: 子は参加面なので、ここでは出さない（手順 8）。
     - それ以外: emit(child, S, child.transform.then(Ms4)) とする（Ms4 は Ms を 4×4 にしたもの）。emit の中で平坦化される（手順 1 の H）。
     - clips があれば coverage chunk（`Clip(n)`）を作り、子の最後の Draw（手順 9）の `coverage` にする。
   - `External(item)`: `Draw { source: External { frame, cpu: item.cpu.as_ref(), source: item.source }, map: (S px → item.rect の空間), polygon: item.rect を item.transform と Ms で写した四角形, coverage: clips の coverage chunk, opacity, sampling: pixelated なら Nearest }`。executor は、item.rect の空間の点を、texture の大きさ（GPU は import した texture、CPU は `cpu` の大きさ）と `source`（None なら全体）で texel に変換する。
   - `Shader(item)`: coverage chunk（`Coverage(n)`、item.coverage を item.transform で layer 空間に置いた scene）と fallback chunk（`Fallback(n)`）を作り、`Shader(ShaderOp)` を出す。CPU は ShaderOp を、`Draw { source: Chunk(fallback), map: 単位, polygon, opacity, blend: Normal, composite: SourceOver }` として実行する。
7. **効果:** S に次の順で掛ける（CSS の順）。
   1. filters を順に `Filter` op で掛ける（S → S1 → S2 …。各段は同じ大きさの一時 target）。
   2. clip があれば、clip の coverage chunk（`Clip(u32::MAX)`）で `Clip` op を掛ける。
   3. masks があれば、`Mask` op を掛ける。
8. **3D コンテキスト:** layer.plane が `Preserve3d` なら、§8.4 で参加面を集め、手順 9 の代わりに参加面の多角形を T に直接描く。plane が `Preserve3d` の layer は grouping property を持たない（§6.4）ので、手順 7 の効果は既定値である。
9. **親への合成:** `Draw { target: T, source: Target(S_last), polygon: H.map_rect(surface_rect), map: (T px → S px), opacity, blend, composite }`。

### 8.4 3D コンテキスト（`plan/context3d.rs`）

**参加面の収集:**

- コンテキストの root R（plane が Preserve3d の layer）について、参加面を次のように集める。
  - R 自身の平面は、R の item の列を子 layer の item（`LayerItem::Layer`）の位置で区切った断片に分ける。子 layer の前、子 layer と子 layer の間、最後の子 layer の後の、空でない item の連続をそれぞれ参加面 P0_k とし、行列は to_t（R の空間から T px）とする。
    - 例: item が Raster 赤 → Layer 青（`scale_z(2)`）→ Raster 緑なら、P0_0（赤）、青の参加面、P0_1（緑）の三つになる。三つとも z = 0 の同じ平面なので、下の同一平面の規則で item の順に重なり、緑が上になる。
    - R の effects（filter、mask など）は、断片ではなくコンテキスト全体の平坦化の後に掛かる（§6.4 の grouping property。effects を持つ R は Preserve3d の root にならない）。
  - R の子 layer C（`LayerItem::Layer { index, clips }`）のうち、clips が空のものごとに、行列 C.transform.then(to_t)（平坦化しない）で参加面を作る。
  - clips が空でない子 C は参加面にしない（§6.4）。C は断片の item として扱い、その断片の surface に §8.3 の手順 6 の `Layer` の規則（flat）で描く。clips は、その surface の上の coverage chunk（`Clip(n)`）である。断片は、参加面になる子の位置だけで区切る。
  - C の plane も Preserve3d なら（clips が空のとき）、C の子も同じ規則で再帰的に参加面にする。
  - 断片の中の `External` と `Shader` の item の clips も、断片の surface の上の通常の coverage（§8.3 の手順 6）である。3D の扱いは要らず、plan はこれらを error にしない。
- 各参加面は、自分の平面の内容（参加面になる子を除く item と effects）を、§8.3 の手順 2〜7 で自分の surface に描く。raster scale は、行列を平坦化した H_p で求める。

**多角形と並べ替え:**

1. 各参加面の surface_rect の 4 頂点 (x, y, 0, 1) を行列 M_p で写す。§5.2 と同じ方法で w > W_EPS に切り取り、w で割った 3D の点 (x/w, y/w, z/w) の凸多角形にする。
2. backface が Hidden で `M_p.shows_back()` なら捨てる（§8.5）。
3. BSP 木を作る。
   - 最初の多角形の平面を分割面にする（法線 n = (b − a) × (c − a)、d = −n·a）。
   - 他の多角形の各頂点の符号付き距離 n·v + d（|n| で正規化）を、ε = 1e-4 で前・後・同一平面に分ける。
   - 両側にまたがる多角形は辺で分割する。分割で生じた辺は split edge として記録する。
4. 描画順は、viewer を +z の無限遠に置く（正射影の視線）として決める。
   - n_z ≥ 0 なら、後の部分木 → 同一平面（DOM 順）→ 前の部分木の順。
   - n_z < 0 なら逆順。
5. 各多角形について、`Draw { target: T, source: Target(参加面の surface), polygon: (x, y) の多角形, split_edges, map: (T px → surface px) }` を出す。map は H_p⁻¹ に surface の Ms を合成したもの。参加面には clips が無い（clips のある子は参加面にしない）。コンテキストの root R 自身の `LayerItem::Layer { clips }`（R の親の空間、T の平面）の clips は、T の px の coverage chunk として、すべての多角形の Draw の `coverage` に付ける。

同じ平面上の多角形は、参加面の順を保つ。順の key は、R から参加面までの item の位置の列（R の断片と子 layer は R の items の中の位置、さらに入れ子の参加面はその中の位置）の辞書順である。これは `Composition` の item の順であり、tree の DOM 順と Draw の命令の順の両方を表す。

**hit test と drawable の配置:** engine の hit test（§7.3）と drawable の配置（§10.3）の z が等しい候補の順も、同じ key（描画順）で決める。R の断片にある region や drawable は、その断片の位置で並ぶ。

### 8.5 backface

表裏の判定は、4×4 の行列で行う（CSS Transforms 2 の規則）。

```rust
impl Matrix4 {
    /// The plane shows its back face: the matrix is invertible and its
    /// inverse has m[2][2] < 0. scale(−1, 1, 1) shows the front;
    /// rotate_y(180) shows the back.
    pub fn shows_back(&self) -> bool;
}
```

- **判定に使う行列:**
  - 3D コンテキストの参加面: §8.4 の M_p（コンテキストの root の親の空間までの累積。平坦化しない）
  - それ以外の layer と node: 自身の親への 4×4（`CompositeLayer::transform`、engine の `to_parent`。親の perspective を含む）
- **表裏の結果:** backface が Hidden で `shows_back()` が true なら、描かず、hit もしない。
- **共有:** plan（§8.3 の手順 1、§8.4 の手順 2）と engine の hit test（§7.3）は同じ関数を使う。

射影の行列式の符号（鏡像でも負になる）は使わない。`Projective::shows_back` は定義しない（§5.1 から削除する）。

### 8.6 cache と damage（`cache.rs`、`plan/hash.rs`、`plan/damage.rs`）

**chunk:**

- `CacheIndex.chunks: HashMap<ChunkKey, ChunkRecord { scale: f64, origin: Point, size: (u32, u32), index: Arc<[(u64, Option<PixelRect>)]>, frame: u64 }>`。
- 今フレームの chunk の index（各 command の hash と chunk px の bounds）を作り、記録と比べる。
  - scale、origin、size のどれかが違えば `Redraw::All` とする。
  - index が等しければ chunk task を出さない。
  - それ以外は、差分の rect で `Redraw::Rects` とする。
- **差分:**
  1. 共通の先頭と末尾を除く。
  2. 残りを先頭から比べる。hash が違うときは、先 64 個以内で一致する位置を探す（old 側を進めて一致すれば削除、new 側を進めて一致すれば挿入）。見つからなければ両方を変更として一つ進める。
  3. 変更された command の old と new の bounds を集める。
  4. 17 個以上になったら、全体の bounds 一つにまとめる。
- bounds は `plan/bounds.rs` の索引で求める。group の push/pop の bounds は group 全体の bounds、blur を持つ group は 3σ 広げる。

**layer の signature:**

- signature は次を FNV-1a で畳み込んだ 64 bit 値である。
  - key、bounds、plane、backface
  - effects の各値（filter の数値、ShaderUse の program hash・params・time）
  - 各 item の値（Raster は index の hash の列、Layer は子の signature と transform の bit と clips、External は frame と矩形、Shader は ShaderUse と矩形と coverage の hash）
  - `LayerFilter::Graph` は primitive の値を畳み込み、Image の pixels は `Pixels::rgba` の data の pointer で畳み込む（graph の `Arc` の pointer は使わない）
  - surface の scale と surface_rect
- `CacheIndex.layers: HashMap<LayerKey, LayerRecord { signature: u64, window_bounds: Option<Rect>, frame: u64 }>`。
- signature が一致する layer の surface は再利用する（§8.3 の手順 4）。

**window damage（CPU のみ）:**

- layer の記録に、signature とは別に `own: u64`（自身の bounds、plane、backface、effects、window への写像の bit だけを畳み込んだ hash）を持つ。
- 次の layer については、前後の window bounds の和を damage とする。
  - `own` が変わった
  - 消えた（前フレームの記録にあって今フレームに無い）
  - 新しく出た
- 子の layer の変化は、その子自身の規則で加える（親の signature の変化だけで親全体を damage にしない）。
- 再描画した chunk の damage rect を、その layer の window への写像で写した bounds を加える。root の Raster chunk の damage rect はそのまま加える。
- External と Shader の item が変わったら、その item の window bounds を加える。
- **効果を通る伝播:** 上で求めた damage の rect が、次の layer の中で生じた場合は、その layer の効果の出力範囲全体（surface_rect に reach を足した範囲の window bounds）を damage に加える。
  - filters、masks、clip（effects が既定でない）を持つ layer の中の damage は、その layer の出力全体を変え得る。
  - 3D の参加面（写像が affine でない）の中の damage も、同じく出力全体を加える。
  - これを祖先へも同じ規則で繰り返す（保守的な伝播）。
- **backdrop の伝播:** backdrop を持つ layer の window bounds が、そのフレームの damage と交わる場合は、その layer の window bounds（backdrop の shape に reach を足した範囲）を damage に加える。その layer は §8.3 の手順 4 のとおり常に描き直す。
- この伝播は最初の実装として保守的に行う。より細かい伝播は、計測（§8.15）で必要と分かったときに設計を改める。
- damage が 17 個以上、または合計面積が window の 50% を超えたら `Damage::Full` とする。
- resize、`invalidate()`、前フレームの失敗の直後も `Damage::Full` とする。
- GPU は target 0 を毎フレーム全体で合成する。damage は計算するが使わない（`GpuReport` に含めない）。

**記録の掃除:** 2 フレーム続けて使われなかった chunk と layer の記録と texture を捨てる。CPU の surface の合計が 256 MiB、GPU が 512 MiB を超えたら、古いものから捨てる。

### 8.7 画素の演算（CPU と GPU で同一）

**値と標本化:**

- surface の画素は premultiplied RGBA8 である。演算は 0..1 の f32 で行い、書き戻すときは (v·255).round() とする。
- **bilinear 標本化:**
  - texel の中心を (i + 0.5, j + 0.5) とする。
  - 範囲外の texel は透明 (0, 0, 0, 0) として扱う。
  - premultiplied のまま補間する。
- **nearest 標本化:** floor(x)、floor(y) の texel を使う。

**多角形の coverage:**

- 多角形の頂点を反時計回りにそろえる（符号付き面積が負なら逆順）。
- 画素の中心 p について、各辺 e の内向きの符号付き距離 d_e（px）を求め、coverage = clamp(0.5 + min_e d_e, 0, 1) とする。
- split edge は min から除く。代わりに、top-left 規則で画素の所属を決める。d_e > 0、または d_e = 0 で辺が上辺か左辺なら内側とする。

**合成の順序:**

1. source を map で標本化した色 s を求める。
2. s に opacity × coverage × （coverage chunk の alpha）を掛ける。
3. blend と composite で dst と合成する。

**blend（W3C Compositing 1）:** cs, cb は premultiplied、Cs, Cb は unpremultiplied、αs, αb は alpha である。

- co = cs·(1 − αb) + cb·(1 − αs) + αs·αb·B(Cb, Cs)
- αo = αs + αb·(1 − αs)

B の定義は次のとおり。

| mode | B(Cb, Cs) |
|---|---|
| Normal | Cs |
| Multiply | Cb·Cs |
| Screen | Cb + Cs − Cb·Cs |
| Overlay | HardLight(Cs, Cb) |
| Darken | min(Cb, Cs) |
| Lighten | max(Cb, Cs) |
| ColorDodge | Cb = 0 → 0; Cs = 1 → 1; else min(1, Cb / (1 − Cs)) |
| ColorBurn | Cb = 1 → 1; Cs = 0 → 0; else 1 − min(1, (1 − Cb) / Cs) |
| HardLight | Cs ≤ 0.5 → Cb·2Cs; else Screen(Cb, 2Cs − 1) |
| SoftLight | Cs ≤ 0.5 → Cb − (1 − 2Cs)·Cb·(1 − Cb); else Cb + (2Cs − 1)·(D(Cb) − Cb)、D(x) = x ≤ 0.25 ? ((16x − 12)x + 4)x : √x |
| Difference | \|Cb − Cs\| |
| Exclusion | Cb + Cs − 2·Cb·Cs |
| Hue | SetLum(SetSat(Cs, Sat(Cb)), Lum(Cb)) |
| Saturation | SetLum(SetSat(Cb, Sat(Cs)), Lum(Cb)) |
| Color | SetLum(Cs, Lum(Cb)) |
| Luminosity | SetLum(Cb, Lum(Cs)) |
| PlusLighter | co = min(1, cs + cb)、αo = min(1, αs + αb)（上式を使わない） |

Lum、ClipColor、SetLum、Sat、SetSat は W3C Compositing 1 §10.2 の定義に従う。Lum(C) = 0.3R + 0.59G + 0.11B。

**composite（Porter–Duff、premultiplied）:** result = Fa·src + Fb·dst。

| op | Fa | Fb |
|---|---|---|
| SourceOver | 1 | 1 − αs |
| SourceIn | αb | 0 |
| SourceOut | 1 − αb | 0 |
| SourceAtop | αb | 1 − αs |
| DestinationOver | 1 − αb | 1 |
| DestinationIn | 0 | αs |
| DestinationOut | 0 | 1 − αs |
| DestinationAtop | 1 − αb | αs |
| Lighter | 1 | 1（結果を 1 に clamp） |
| Copy | 1 | 0 |
| Xor | 1 − αb | 1 − αs |

composite は多角形の内側（coverage > 0）にだけ掛かる（bounded）。blend ≠ Normal のときは、composite は SourceOver でなければならない（engine が保証する）。

**GPU の実装:**

- Normal と、PlusLighter 以外の Porter–Duff は、固定機能の blend state で表す（Fa, Fb の表をそのまま使う）。
- それ以外の blend mode と PlusLighter は、dst の該当範囲を一時 texture に copy し、fragment shader で上式を計算して書く。
- **標本化:** sampler を使わず、`draw.wgsl` が 4 回の `textureLoad` で bilinear を計算する。texture の外の texel は 0 とする（CPU と同じ）。nearest は 1 回の `textureLoad`。
- **coverage:** 多角形を 1 px 外へ広げた凸多角形（各辺を外向きに 1 px 平行移動した多角形。split edge は広げない）を三角形の fan で描く。各 fragment で、元の多角形の辺の符号付き距離から CPU と同じ式で coverage を計算する。split edge の top-left 規則も fragment shader で計算する。
- **書き込み:** target はすべて非 sRGB の view（`Rgba8Unorm`、swapchain は `Bgra8Unorm` または `Rgba8Unorm`）で書く。shader の中で sRGB の変換をしない。

### 8.8 filter

#### 8.8.1 gaussian blur

CPU と GPU で同じ手順で行う。std_dev σ は layer 空間の値で、σ_px = σ·px_per_unit とする。

1. σ_px < 0.5 なら何もしない。
2. k = 0 から始め、σ_px / 2^k > 4 の間 k を増やす（最大 6）。
3. 2×2 の box 平均（premultiplied）で k 回 1/2 に縮小する。
4. σ' = σ_px / 2^k、半径 r = ⌈3σ'⌉、重み w_i = exp(−i² / (2σ'²)) を [−r, r] で正規化する。
5. 横、縦の順に分離畳み込みをする。範囲外は透明とする。
6. bilinear で k 回 2 倍に拡大する。

縮小と拡大の画素の対応は、次のとおり（CPU と GPU で同じ）。

- 縮小: 大きさは (⌈w/2⌉, ⌈h/2⌉)。dst(i, j) は src(2i..2i+1, 2j..2j+1) の 4 texel の平均で、範囲外の texel は 0 とする。
- 拡大: 大きさは縮小前の大きさに戻す。dst(x, y) は、低解像度の texel 座標 ((x + 0.5) / 2 − 0.5, (y + 0.5) / 2 − 0.5) を中心にした bilinear で、範囲外は 0 とする。

filter graph の GaussianBlur（σx, σy が別）も、軸ごとに同じ手順で行う。

#### 8.8.2 drop shadow

1. 入力の alpha に color（premultiplied）を掛けた画像を作る。
2. それを §8.8.1 で blur し、offset·px_per_unit だけずらす。
3. 入力をその上に SourceOver で重ねる。

#### 8.8.3 color matrix

各画素を unpremultiply し、4×5 行列を掛けて clamp し、premultiply する。CSS の filter 関数は sRGB の値に掛ける（既存の `ColorMatrix` の定義どおり）。

#### 8.8.4 filter graph

**CPU:**

- resvg 0.46.0 の `src/filter/` を `cpu/graph/` に移植する。
  - 移植する: color_matrix.rs、component_transfer.rs、composite.rs、convolve_matrix.rs、displacement_map.rs、lighting.rs、morphology.rs、turbulence.rs、mod.rs のうち primitive の実行・subregion・linearRGB 変換・Merge・Offset・Tile・Flood・Image・Blend の部分。
  - 移植しない: box_blur.rs、iir_blur.rs。GaussianBlur と DropShadow は §8.8.1 と §8.8.2 を使う。
- 入出力は tiny-skia の Pixmap ではなく `cpu/surface.rs` の Pixmap（premultiplied RGBA8）にする。usvg の型は §5.9 の型に置き換える。
- filter region は layer bounds を margin だけ広げたものとする。各 primitive の subregion は filter region である。Tile は自身の `tile` を使う。
- 座標は layer 空間で、px_per_unit で画素に写す。primitive の数値（std_dev、radius、offset、scale、light の位置、base_frequency）は px_per_unit を掛けて画素の値にする。base_frequency だけは px_per_unit で割る。
- `color_space`（None なら graph の値）が LinearRgb なら、primitive の入力を sRGB → linear に変換し、出力を linear → sRGB に戻す（resvg と同じ）。

**GPU:**

- 同じ式を WGSL で実装する（`gpu/wgsl/graph_*.wgsl`）。
- Turbulence は、seed から作る格子と勾配の表（resvg の `init` と同じもの）を CPU で作って storage buffer で渡し、noise の評価を WGSL で行う。
- ConvolveMatrix の kernel、ComponentTransfer の table は uniform buffer で渡す（最大 256 要素）。
- Lighting は、Sobel の法線と光源の式を resvg と同じに実装する。

#### 8.8.5 shader filter

GPU は §11.3 の module を、入力 target を `content` として実行し、出力 target に描く。出力の範囲は surface に margin を足したもの（§8.3 の reach）である。CPU は入力をそのまま出力にする（filter を飛ばす）。

### 8.9 mask、clip、backdrop

**clip:** coverage chunk（clip shape を白で塗った scene）の alpha を S の各画素に掛ける。

**mask:**

1. 各 RenderMask の scene を mask chunk として S の空間で raster する。
2. 各画素の値 m を求める。
   - Alpha: m = α
   - Luminance: m = (0.2125·R + 0.7154·G + 0.0721·B)·α。R, G, B は unpremultiply した sRGB の値。
3. 一番下の layer（列の最後）から上へ合成する。acc は一番下の m から始め、上の layer ごとに次の式を使う。一番下の layer の composite は使わない。
   - Add: acc = m + acc·(1 − m)
   - Subtract: acc = m·(1 − acc)
   - Intersect: acc = m·acc
   - Exclude: acc = m·(1 − acc) + acc·(1 − m)
4. S の各画素に acc を掛ける。

**backdrop:** §8.3 の手順 5。backdrop が読むのは、CSS の Backdrop Root の内容である。

- **backdrop root:** 最も近い祖先の layer のうち、effects が既定でないもの（opacity < 1、filters、clip、masks、backdrop、blend ≠ Normal、composite ≠ SourceOver のどれかを持つ）、または root。P2 により、CSS の Backdrop Root になる node は必ず layer なので、この規則は CSS と一致する。
- **透過の layer:** effects が既定の layer（overlay、fixed、cache のための layer、3D の layer、drawable など）は、backdrop の読み取りでは透過である。
- **入力の作り方:** backdrop の入力は、一時 target B に次の順で描いて作る。
  1. backdrop root の target の現在の内容を、B の座標へ写して描く。
  2. その間にある透過の layer の surface の現在の内容（その layer の、この item より前の内容）を、外側から順に、それぞれの写像で描く。
- 読むのは実際の画素なので、別の backdrop が描いたものも見え、color matrix は CSS どおり合成後の画素に掛かる。

### 8.10 user shader の実行（GPU）

`gpu/shader.rs` は `ShaderProgram::hash` を key に render pipeline を cache する。module は §11.3.3 の契約（binding と入口）に従う。`ShaderOp` は次のように描く。

1. polygon を三角形の fan にして描く。vertex shader には target px を渡す。
2. fragment shader に、map で求めた shader 空間の位置 `position`、`Context { size, scale: px_per_unit, time }`、params、images、`content`（filter のとき）を渡す。
3. 出力は straight alpha の sRGB である。生成された入口関数が premultiply し、coverage と opacity を掛けたうえで、SourceOver で target に描く（§11.3.3）。

pipeline の作成に失敗した場合（driver のエラー）は、その program を失敗として記録する。`GpuReport::shader_failures` に program の hash とメッセージを一度だけ含め、以後その program は fallback chunk を描く。arvane はこれを diagnostics の `RendererChange::ShaderFailed { message }` として一度だけ記録する。

### 8.11 外部 frame

- GPU は、既存の `ExternalPass`（arvane-render から移す）で frame を acquire する。そのうえで `Source::External` の texture view を DrawOp の source にする。
- CPU は、`cpu` があればそれを `Source::Pixels` として描き、無ければ描かずに `missed` に入れる。
- 既存の 100 ms の ready 待ち、frame ごとの release、`take_missed_external` の意味は変えない。

### 8.12 drawable の export

`Export`、`ExportMode`、`ExportFrame`、`ExportedFrame`、`ExportLoss` の定義は §5.11 の一か所だけにある。

**評価:** plan は layers[layer] を、大きさ `size` の target に写像 scale(scale) で評価する。drawable の box の原点が target の (0, 0) になる。export する layer 自身の `CompositeLayer::transform` は単位行列として無視する。

**送るかの判定:**

- compositor は sink ごとに `ExportRecord { generation, mode, signature, size, scale }`（純粋 cache）を持つ。
- 今フレームの値と一つでも違えば（内容が静止していても、request や mode や大きさが変われば）export する。
- export に成功したら、記録を更新する。

**送り方:**

- **Pixels:** target を unpremultiply して `ExportFrame::Pixels` を返す（GPU では読み戻す）。
- **Gpu:**
  - その sink の export pool（`export_drawable` で作ったもの）から `Pool::try_acquire()` で slot を得る。
  - `try_acquire` は、consumer が release 済みの slot（release fence の CPU から読んだ完了値が、その slot の待ちの値以上）だけを返す。待ちの値は、frame を描いた slot ではその最後の ready 値、まだ描いていない slot では準備の合図の値 `PREPARED = 1` である。無ければ None を返し、GPU の queue に待ちを積まない。
  - None なら、このフレームの export を見送る（記録を更新しないので、次に Export を処理するフレームで再試行する）。window の提示は続ける。
  - slot が得られたら、target をその texture に copy し、ready fence を signal して `ExportedFrame { sink, generation, frame: ExportFrame::Gpu { slot, value } }` を返す。
  - **consumer の準備（準備の合図）:** import の完了は、API の呼び出しではなく、共有の release fence で知らせる。
    - import した側は、texture の準備（layout と所有権の受け渡し）を終えたら、release fence を `PREPARED = 1` に signal する。`kivact_gpu_share::import::Import::new` が、準備の barrier の submit の直後にこれを行う（アプリは何もしない）。
    - frame の ready の値は 2 から始める。だから release = 1 は、どの frame も解放しない「準備の完了」だけを表す。
    - window は、まだ描いていない slot を、release fence が 1 に達するまで `try_acquire` で返さない。consumer が import する前の export は「slot が無い」と同じく見送られ、`pending_exports` と再試行（§10.5）で、import の後に最初の frame が届く。新しい pool の fresh な slot へ import の前に書くことは、型と fence の両方から起きない。
    - pool ごとに fence が別なので、古い generation の import が新しい pool の合図になることはない。
  - **見送りの報告:** slot が無くて見送った sink は、`FrameReport::pending_exports` に入れる。CPU の compositor が描いたフレーム（GPU の device を保ったまま一フレームだけ CPU で描く `GpuLimit` の fallback を含む）で mode が Gpu の Export も、同じく `pending_exports` に入れる（次のフレームで GPU が再試行する）。engine は、その sink の export が成功するまで再試行の frame を要求する（§10.5 の `ExportRetry`）。
- **`kivact-gpu-share` の変更:**
  - wgpu の export の `Pool` に、`pub fn try_acquire(&mut self) -> Result<Option<Target<'_>>, Error>` を足す。既存の `acquire(timeout)` は、release を待たない slot の選び方に変えない（drawable の export は `try_acquire` だけを使う）。
  - 共通の `Slots`（`src/pool.rs`）: frame の値を 2 から振る（`next_value` の最初の値を 2 にする）。まだ描いていない slot の待ちの値（`Acquired::wait`）を `PREPARED`（1）にする（`SlotState::Fresh` の `value()` が `Some(PREPARED)` を返す）。これで、既存の producer（wgpu、D3D12、Vulkan の pool）は、最初の書き込みの前に GPU 側で release ≥ 1 を待つ。
  - `Import::new`（`src/import/pool.rs`）: 準備の submit（既存の barrier と、各 slot の `release`）の後、`idle` を待つ前に、`signal(SHARED_PREPARED)` を一度行う（queue の順で準備の後に signal される）。既存の逆方向（アプリが作った texture を Arvane が import する。`arvane-compose/src/gpu/external.rs` も `Import::new` を使う）にも同じ合図が出る。
  - `PREPARED` は kivact-types の `gpu.rs` に `pub const SHARED_PREPARED: u64 = 1;` として置き、両側がこれを使う。
- **`SharedTextures` の契約の変更（kivact-types `gpu.rs` の rustdoc）:**
  - 「Order」の段落の「writes a texture for the first time only after the import returned `Ok`」を、「writes a texture for the first time only after `release` reached `SHARED_PREPARED` (1): the importer signals it once the textures are prepared for the hand-off」に置き換える。
  - 「Frames」の段落に「frame values start at 2 (1 is the preparation signal)」を加える。
  - 既存の `ShareLink::provide` の `# Safety` の節の、「writing a texture only after `ShareLink::wait_ready` is `Ok`」を、この新しい順序の規則への参照に直す（provide が unsafe であることは変えない。texture の作り方など、確かめられない契約が残るため）。

**pool の寿命:**

- **作成:** `export_drawable(sink, generation, adapter, size)` が、その generation の pool を作る。
  - 同じ sink の古い pool があれば、`drop_export` と同じ手順で捨ててから作る。
- **破棄:** `drop_export(sink)` は、次の手順で pool を捨てる。
  1. pool を表から外し、以後の export に使わない。
  2. `wait_idle(DROP_WAIT)`（5 秒）を worker thread で待ってから、pool を drop する。window の thread を止めない。
- **pool の持ち主と喪失の通知:** pool の実体は `GpuContext` の中にあるが、どの (sink, generation) に pool があるかの記録と通知は、window と同じ寿命の arvane の presenter が持つ。

```rust
// arvane/src/presenter/exports.rs (new)
/// The GPU export pools the window has made, and what the engine has not yet
/// been told. Lives as long as the presenter (outlives any GpuContext).
pub(crate) struct ExportRegistry {
    pools: HashMap<DrawableId, u64>,   // sink → generation of its live pool
    notices: Vec<ExportNotice>,        // drained by Window::take_export_notices
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportNotice {
    /// The pool of this request is gone (device lost, GPU given up).
    Lost(ExportLoss),
    /// A GPU device became usable for exports (start finished, or a new
    /// device after a loss).
    Available,
}
```

  - `export_drawable` が成功したら `pools` に (sink, generation) を入れ、`drop_export` で外す。
  - presenter が `GpuContext` を捨てる前に（device の喪失、`fallback_to_cpu` による session の CPU 化、その他の理由のすべて。presenter の `State` を GPU の状態から置き換える箇所）、`pools` の全項目を `ExportNotice::Lost` として `notices` に移し、`pools` を空にする。
  - presenter の状態が、export が `GpuContext` を借りられる Ready に入ったとき（起動の完了、喪失の後の再起動の完了のどちらでも、Ready へ入る一箇所）に、`ExportNotice::Available` を一つ積む。
  - `notices` は、提示が成功したか失敗したかに関係なく、`Window::take_export_notices() -> Vec<ExportNotice>` で取り出せる（取り出すと空になる）。
- **`export_drawable` の失敗の区別:** presenter が GPU の device を持たないとき（`Starting` の間、CPU の session）は、kind が新しい `FailureKind::NotReady`（kivact-types の `platform.rs` に加える。"A capability exists but its backend is not ready yet; retry when it reports ready."）の `Failure` を返す（待てば変わり得る）。`gpu-interop` の無い build では kind が `FailureKind::Unavailable` の `Failure` を返す（変わらない）。device を保ったまま一フレームだけ CPU で描く fallback の間は、device があるので pool を作る。
- **engine と core:** engine は pump ごとに `take_export_notices` を読み、`Lost` を `HostEvent::DrawableExportLost(ExportLoss)`、`Available` を `HostEvent::GpuExportAvailable` にする。core の扱いは §10.5 の遷移表のとおり（`Lost` は generation が現在と一致するときだけ新しい generation の `GpuRequested` に戻し、`Available` は `GpuWaiting` を新しい generation の `GpuRequested` に戻す）。adapter の identity（`GpuAdapterChanged`）は export の可否の判定に使わない。

### 8.13 提示

**CPU（`frame`）:**

1. target 0 は window の大きさの永続 pixmap である。
2. Damage::Rects のときは、plan は target 0 への op の前に、damage の各 rect について `Clear { target: 0, rect: Some(rect) }` を出す。target 0 への DrawOp には scissor を付け、damage の外を書き換えない。
3. Damage::Full のときは、`Clear { target: 0, rect: None }` を出す。
4. scratch target（`TargetKind::Temp`）は、使う前に必ず全体を Clear する。Kept の surface を描き直すときも全体を Clear する。
5. 提示の変換（白の上に合成して 0x00RRGGBB にする。既存の `CpuFrame::to_softbuffer` と同じ式）は arvane が行う（§15）。

**GPU（`present`）:**

1. chunk を vello_hybrid で chunk texture に raster する。部分 redraw は、damage rect の大きさの一時 texture に平行移動した scene を描き、chunk texture の該当位置へ copy する。
2. op を実行する。
3. target 0 を swapchain の texture に描く。最初に白で clear し、premultiplied の SourceOver で描く。
4. 既存の direct の経路（vello_hybrid が swapchain に直接描く）は削除する。

**GPU の容量不足:** `RenderError::GpuLimit` は従来どおり CPU で描き直して `present_pixels` で上げる（arvane の規則のまま）。CPU の結果は `CpuCompositor::frame` を使う。

### 8.14 上限

上限を超えても、内容は捨てない。描いたものと hit・focus・a11y が食い違わないように、品質を落として描く。

- **target:** 1 フレームの target が 1024 を超える場合は、次の条件をすべて満たす layer を、`Composition` の順で後のものから、自分の surface を持たずに親の target へ直接描く。この形を inline と呼び、cache しない。
  - effects が既定
  - plane が Flat
  - 写像が affine
  - 子孫に backdrop を持たない
  - その layer の中（子の surface の中を除く）に、blend ≠ Normal の `Group` または `LayerEffects`、composite ≠ SourceOver の `LayerEffects` を持たない（surface を持つかどうかで合成の結果が変わる layer は inline にしない）

  それでも超える場合は、GPU は `RenderError::GpuLimit` を返す。presenter は既存の規則で、そのフレームを CPU で描く。CPU の executor は target の数の上限を持たない（大きさの上限 8192 だけを持つ）。
- **3D の多角形:** 多角形の数で描き方を変えない。代わりに、参加面が 256 を超える 3D コンテキストは、engine と compositor の両方が同じ規則で「平坦な重なり」として扱う。描画は参加面を DOM 順（後のものが上）に重ね、hit test も DOM 順（後のものが先）で判定する。参加面の数は、tree の node の layer、Draw の recording の 3D の layer（`DrawingLayer`。tree の node を増やさない）、R の空でない断片のすべてを数える（clips のある子は参加面ではないので数えない）。engine は `Composition` を作るときに、コンテキストごとの数を同じ規則で数えて hit test にも使うので、描画と hit が食い違わない。256 以下のコンテキストの BSP は分割の数に上限を持たない。
- **layer の数:** 上限を持たない。
- **診断:** inline、GpuLimit、平坦な重なりのどれかが起きたら、arvane は diagnostics に `RendererChange::LimitExceeded { what }` を一度だけ記録する。engine の平坦な重なりの判定（§7.3）は、同じ定数 `MAX_CONTEXT_PLANES = 256`（kivact-types の `render/composition.rs` に置き、`render` から再公開する。`render.rs` は non-leaf なので定数を置かない）を使う。


### 8.15 計測（`FrameStats`）

compositor は、フレームごとに `FrameStats`（§5.11）を数えて `FrameReport::stats` で返す。統計であり（S-9）、状態の owner ではない。

| field | 数え方 |
|---|---|
| chunks_rastered | Vello に raster させた chunk の数（部分 redraw も 1 と数える） |
| pixels_rastered | raster した画素の合計（部分 redraw は rect の面積） |
| surfaces_reused | `reuse: true` の surface の数 |
| damage_pixels | window の damage の面積（Full なら window 全体） |
| targets | このフレームの target の数 |

cache と部分再描画の試験（§17.2）は、この値で最適化が働いたことを確かめる。

---

## 9. engine と backend: 新しい描画語彙の lower

### 9.1 border の線種（`lower/border.rs`）

border は辺ごとに線種を持つ（`Border::styles`）。既存の辺ごとの台形の分割（対角線で分ける処理）は、すべての線種で使う。以下、w は辺の幅、c は辺の色である。

| 線種 | 描き方 |
|---|---|
| Solid、Dashed、Dotted | 既存のまま |
| Double | 外側の帯 [0, w/3] と内側の帯 [2w/3, w] を c で塗る。w < 3 なら Solid と同じ |
| Groove | 外側の半分 [0, w/2] と内側の半分 [w/2, w] に分ける。上辺と左辺は外側 dark・内側 light、下辺と右辺は外側 light・内側 dark |
| Ridge | Groove の dark と light を入れ替える |
| Inset | 上辺と左辺は dark、下辺と右辺は light |
| Outset | 上辺と左辺は light、下辺と右辺は dark |

**dark と light:**

- 色の最大 channel が 0.01 以上なら、dark(c) = (r·2/3, g·2/3, b·2/3, a)、light(c) = c とする。
- 最大 channel が 0.01 未満（黒）なら、dark = (0, 0, 0, a)、light = (0.5, 0.5, 0.5, a) とする。

帯の境界は、外側の角の丸めを内側へ指定の距離だけ縮めた rounded rect（既存の `BoxShape::inset`）で求める。

### 9.2 border image と nine-slice（`lower/border_image.rs`、新規）

CSS Backgrounds 3 §6 の手順で、9 つの領域に分けて描く。`Style::border_image` と `Draw::border_image` は、同じ関数 `lower_border_image(out, image: &BorderImage, border_box: Rect, borders: Edges<f64>, ctx)` を使う。

1. **border image area:** border box を outset だけ外へ広げた矩形 A。
2. **source の大きさ (sw, sh):** 画像なら natural size（読み込み前なら何も描かない）。gradient と単色なら A の大きさ。
3. **slice:** slice を px（pct は source の大きさに対する割合）で解決し、0 以上に丸める。左右の slice の和が sw を超える場合、中央の列は空になる。上下も同様。
4. **幅:** width を解決する（None なら borders、pct は A の大きさに対する割合）。f = min(A.w / (wl + wr), A.h / (wt + wb)) が 1 未満なら、四つの幅すべてに f を掛ける。
5. **四隅:** source の隅の矩形を、A の隅の矩形（幅 × 幅）に拡大縮小して描く。
6. **辺:** source の辺の矩形を、厚さ方向は幅に合わせて拡大縮小する。長さ方向は repeat に従って並べる。長さ方向の tile の長さ t = source の辺の長さ × (幅 / source の厚さ) とし、L を A の該当辺の長さとする。
   - Stretch: 1 枚を L に引き伸ばす。
   - Repeat: 長さ t の tile を、中心が辺の中心に来るように並べ、L で切る。
   - Round: n = max(1, round(L / t)) とし、tile を L / n の長さにして n 枚並べる。
   - Space: n = floor(L / t) とする。n = 0 なら描かない。そうでなければ余りを n + 1 等分した隙間を空けて並べる。
   - 1 辺あたりの tile が 256 を超えるなら Stretch で描く。
7. **中央:** fill が true のときだけ描く。横方向は上辺の倍率、縦方向は左辺の倍率で拡大縮小し、repeat も辺と同じ規則に従う。

**command への lower:**

- 画像: 各 tile を `ImageSampled { source: Some(source の矩形), rect: 描く矩形 }` にする。Round と Space の tile も一枚ずつ出す。
- gradient と単色: `PushClip(描く矩形)`、`PushTransform(source の矩形 → 描く矩形 の affine)`、`FillShape { shape: Rect(source の矩形), paint: A を基準に解決した gradient }`、pop の順に出す。

### 9.3 background の clip、origin、blend（`lower/fill.rs`）

- **positioning area:** layer の位置と大きさは origin の box（None なら padding box）を基準に解決する。既存の「border box を基準にする」から変わる。
- **clip:**
  - BorderBox、PaddingBox、ContentBox: 各 box の rounded rect で clip する。PaddingBox と ContentBox の角は、border box の角を border 幅（と padding）だけ縮めたもの。
  - Text: §9.4 の規則で clip する。
- **repeat の範囲:** repeat は clip の box 全体を覆うように並べる（CSS と同じ）。
- **blend:** 一つでも blend ≠ Normal の layer があれば、背景全体を `PushGroup(Group { clip: None, opacity: 1, blend: Normal, blur: None })` で囲む。その中で背景色を描き、layer を下から順に描く。blend ≠ Normal の layer は `PushGroup(Group { blend: layer.blend, .. })` で囲む。

### 9.4 background-clip: text

clip が Text の layer は、node 自身の shaped text のグリフの輪郭で clip する。

1. node が Text node（または input の表示 text）なら、その `ShapedText` の各 glyph run を `arvane::glyph_outline(run)` で path にする。glyph の位置と origin を足して、一つの `Shape::Path { rule: NonZero }` にまとめる。
2. その背景 layer を `PushGroup(Group { clip: Some(path), opacity: 1, blend: Normal, blur: None })` の中で描く。

**text の描画順:** CSS と同じく、背景は text より先に描く。背景を見せたいときは、text の色を透明にする（`color(Color::TRANSPARENT)`）のが CSS と同じ使い方である。

**対象:** 子孫の Text node の text は含めない。container に Text clip を指定した場合、その container 自身の text が無いので何も描かれない（§1.4 に加えず、ガイドに明記する）。

### 9.5 mask の lower（`lower/effects.rs`）

`Effects::masks` の各 `MaskLayer` を、一つの `RenderMask` にする。

- **scene:** 背景の layer と同じ規則（§9.3）で `fill` を描く。positioning area の既定は border box、clip の既定は border box である。
- **mode と composite:** そのまま写す。
- **source が shader の mask:** 対象外とする。描かず、style の unsupported に数えて報告する。

`DrawingLayer::masks` も同じ規則で lower する。基準の矩形は、`DrawingLayer::area` があればそれ、無ければ recording の大きさとする。

### 9.6 gradient の補間と hint（`lower/fill.rs`）

**展開:** gradient の補間、gamut mapping、hint は、すべて engine が lower の時に一つの関数 `expand_gradient(stops, hints, interpolation) -> Vec<(f64, Rgba)>` で行う。Vello には sRGB の stop の列だけを渡す（Vello の色空間の補間は使わない）。これで、単色と同じ色の gradient の結果が一致し、gamut mapping が CSS と同じになる。

区間 [p_i, p_{i+1}]（色 c_i、c_{i+1}、hint の相対位置 h。hint が無ければ h = 0.5 で重みは線形）ごとに、次の手順で展開する。

1. **重み:** 区間の中の相対位置 t（0..1）の重み w(t) = t^(ln 0.5 / ln h) とする。h は [0.001, 0.999] に収める。hint が p_i または p_{i+1} と等しい場合（丸める前の h が 0 または 1）は、その位置で色が切り替わる hard stop とし、展開しない。
2. **色:** t の色を C(t) = `interpolate(c_i, c_{i+1}, w(t), interpolation).to_srgb()` とする（元の色のまま補間してから gamut mapping する）。
3. **適応分割:** 区間を、最初に t = h で二つに分け、各部分 [t_a, t_b] を次の条件を満たすまで二等分する。
   - 条件: 中点 t_m で、C(t_m) と、両端の色 C(t_a)、C(t_b) を Vello と同じ方法（premultiplied の sRGB の線形補間）で補間した色との差が、どの channel も 0.5/255 以下。
   - 二等分の深さは最大 12 とする（1 区間あたり最大 4096 個の stop）。深さ 12 で条件を満たさない部分（重みの式の傾きが無限大に近い t = 0 の近くなど。幅は区間の 1/4096 以下）は、そのまま両端の線形補間にする。
   - 特別扱いの省略は無い。端点が sRGB の範囲の外にある区間は、mapping が非線形なので、この条件によって必ず分割される。端点が sRGB の範囲の内側で、sRGB の補間で hint が無い区間だけが、結果として 2 stop のままになる。
4. **stop の列:** 得られた t の列を位置 p_i + t·(p_{i+1} − p_i) に写し、C(t) を色とする stop の列にする。

**精度の範囲:** この展開が保証するのは、深さ 12 までに条件を満たした部分で、stop の列が gradient の式と 0.5/255 以内で一致することまでである。深さ 12 で打ち切った部分（幅は区間の 1/4096 以下）は保証しない。この幅は Vello の LUT の 1 項目以下なので、それ以上分割しても最終の画素は変わらない。Vello は gradient を有限の LUT（最大 4096 項目、補間なし）で標本化するので、色が数 px で大きく変わる箇所（hint が端に近い場合など）では、最終の画素が式から数階調ずれる。これは §3.3 の三段目（Vello の raster の差）に属し、本書は縮めない。

### 9.7 text の stroke、fill、variation

**stroke と fill（engine の `lower/glyphs.rs`）:**

- `ComputedTextStyle::stroke` が Some なら、`RenderCmd::Glyphs::stroke` を `GlyphStroke { stroke: Stroke::new(width_px).join(Round), paint: Solid(color) }` にする。
  - width_px は `resolve(width)` で求める。
  - stroke は輪郭の中心線に沿う（CSS の `-webkit-text-stroke` と同じ）。
- `order` は `paint_order` から決める。
- Draw の `TextRun::fill` と `TextRun::stroke` の `Fill` は、次の規則で `render::Paint` にする。
  - `Fill::Solid(c)`: `Paint::Solid(c)`
  - layer が一つの `Fill::Layers` で、image が Gradient または Coordinate: その gradient の `Paint::Gradient`（text の bounds を基準にする）
  - それ以外（image、shader、複数の layer）: unsupported に数え、`color`（stroke は stroke の色に相当する最初の単色、無ければ text の color）で描く

**arvane-render（`draw/painter.rs`）:** `Glyphs` を次の順に描く。

- `order` が FillStroke なら、`fill_glyphs` の後に `stroke_glyphs` を描く。StrokeFill なら逆順で描く。
- **fill:** Paint で塗る。Gradient と Pattern の座標は scene の座標である。既存の glyph cache（atlas）は Solid の fill だけに使う。
- **stroke:** `canvas.set_stroke(stroke)` してから、glifo の `stroke_glyphs` で描く。
- **Canvas trait:** `fn stroke_glyphs(&mut self, run: GlyphRun<'_>, glyphs: impl Iterator<Item = Glyph> + Clone)` を追加する。CPU と GPU の両方で、`glyph_run(..).stroke_glyphs(..)` を呼ぶ。

**glyph の輪郭（`arvane-render/src/outline.rs`、新規）:**

```rust
/// The outlines of `run`'s glyphs as path commands in run space (font size
/// applied, y down, each glyph moved to its x and y), with the run's
/// normalized coordinates. Unhinted.
pub fn glyph_outline(run: &GlyphRun) -> Vec<PathCmd>;
```

skrifa の `OutlineGlyphCollection` と `OutlinePen` で輪郭を読み、`MoveTo`/`LineTo`/`QuadTo`/`CubicTo`/`Close` にする。y は font 座標を反転する。`arvane` はこれを `pub use arvane_render::glyph_outline;` で再公開する。

**variation（arvane-text の `build.rs`）:**

- paragraph と span に `StyleProperty::FontVariations(FontSettings::List(..))` を push する。中身は `variations` の各 tag と値である。
- optical_sizing が true で、variations に `opsz` が無ければ、`opsz` = font_size（px）を加える。
- `font_width` は `StyleProperty::FontWidth(FontWidth::from_percentage(font_width as f32))` として push する。
- `cache.rs` の `TextKey`、`SpanBits`、`layouts.rs` の `SpanKey` と `Key` に、variations（tag と値の bit の列）、font_width の bit、optical_sizing を加える。
- `kivact-style/src/text.rs` の「weight は `wght` に直接渡す」という記述は残す。`wght` を variations で明示した場合は variations の値が勝つ（parley の規則のまま）。

### 9.8 アニメーション画像（`arvane-asset`、engine）

**decode（`arvane-asset/src/raster.rs`）:**

```rust
pub(crate) enum Decoded { Still(Pixels), Animated(Animation) }
pub struct Animation {
    pub frames: Vec<(Pixels, u32 /* delay ms */)>,
    pub repeat: kivact_types::Repeat,
}
```

- 次の形式は、すべての frame を decode する。
  - GIF: `image::codecs::gif::GifDecoder::into_frames`
  - WebP: `WebPDecoder::into_frames`。アニメーションでなければ Still
  - APNG: `PngDecoder::apng()`。`is_apng()` が false なら Still
- 各 frame は合成済みの全面画像（image crate の `Frame::into_buffer`）であり、EXIF の向きを全 frame に適用する。
- delay が 11 ms 未満の frame は 100 ms とする（ブラウザと同じ）。
- repeat は、GIF の loop count（0 = Forever）、WebP の loop count、APNG の `num_plays`（0 = Forever）から決める。
- **decode 中の予算:**
  - **次の frame を decode する前に**、その decode の後の同時の保持量を見積もる。見積り = (保存済みの frame 数 + 1 + W) × w × h × 4 とする。W は形式ごとの decoder の作業領域の frame 数で、image 0.25 の実装の割り当てから決めた値である。
    - GIF: W = 2（合成の canvas と、frame の raw buffer）
    - WebP: W = 2（合成の canvas と、frame の buffer。主な画像の buffer の見積りであり、bitstream によって決まる補助の領域（color cache、Huffman の画像など）は含まない）
    - APNG: W = 4（current と previous の 2 frame、raw frame の領域、出力の clone）
  - **1 枚目の前:** 見積り（保存 0 + 1 + W）が予算の 1/4 を超えるなら、アニメーションとしての decode をせず、既存の静止画の `decode`（その形式の最初の frame だけを読む。既存の予算と `OverBudget`/`Memory` の検査を持つ）で Still にする。`tracing::warn!` を一度出す。静止画の `decode` も失敗すれば、既存のとおり失敗として扱う。
  - **2 枚目以降の前:** 見積りが予算の 1/4 を超えるなら、その frame を decode せずに止め、それまでに decode した最初の frame だけの Still にする（2 枚目以降は捨てる）。`tracing::warn!` を一度出す。
  - **decoder の内部の上限:** decoder 自身の割り当ての扱いは、形式ごとに次のとおりである（固定版 image 0.25.10 の実装による）。
    - GIF: `GifDecoder::new` の後、`ImageDecoder::set_limits` で既存の `decode` と同じ `Limits`（`max_image_width`/`max_image_height` = `MAX_IMAGE_EDGE`、`max_alloc` = `MAX_DECODER_ALLOC`）を掛ける。超えたら既存の `RasterError::Memory` にする。
    - APNG: `PngDecoder::with_limits(reader, limits)`（同じ `Limits`）で作り、その後に `apng()` へ進む。`PngDecoder::new` の後の `set_limits` は png crate の内部の buffer に効かないので使わない。
    - WebP: image の WebP の wrapper は `set_limits` を上書きせず、寸法しか検査しない（`max_alloc` は効かない）。保証は上の外側の見積り（寸法 × W。主な画像の buffer だけ）である。本書は、WebP の decoder の内部の割り当ての総量に上限を約束しない。
  - 1 frame の大きさの検査は、既存の `decode` と同じく割り当ての前に行う。

**AssetStore:** `pub fn animation(&mut self, asset: &Asset) -> Option<AnimationInfo>` を追加する。`pixels`/`pixels_in` に引数 `frame: u32` を追加する。

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationInfo { pub delays_ms: std::sync::Arc<[u32]>, pub repeat: kivact_types::Repeat }
```

**engine の再生（`model/playback.rs`、新規）:**

```rust
/// Animated image playback, keyed by node and image slot
/// (0: the image content, 1 + i: background layer i).
#[derive(Default)]
pub(crate) struct Playback(pub(super) BTreeMap<(NodeId, u8), PlaybackState>);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlaybackState { Playing { start: u64 }, Finished }
```

- **再生する画像:** 次をすべて満たす画像だけが entry を持つ。
  - 描画されている
  - `AnimationInfo` が得られる
  - `ImageData::playing`（既定 true）が true
  - reduced motion でない
- **開始:** entry が無い画像を初めて描くときに、`Playing { start: now }` を入れる。
- **表示する frame:**
  - `Playing { start }` では、経過 e = now − start、1 周の長さ T = delays の和とし、e mod T が属する frame を表示する。
  - 周回数 ⌊e / T⌋ が repeat に達したら `Finished` にする。`Finished` は最後の frame を表示する。
- **entry を消す条件:** 次のどれかで entry を消し、frame 0 を表示する。
  - `playing` が false になった
  - reduced motion になった
  - 描画されなくなった
  - asset が変わった
- **frame の要求:** `Playing` の entry について、次の frame に切り替わる時刻に `frame.request(at)` を出す。毎フレームは要求しない。
- **Draw:** `Draw::image` と `Draw::image_with` は frame 0 を描く。

### 9.9 SVG の `<text>` と埋め込み画像（`arvane-asset/src/svg/*`）

- usvg の `Options` に `fontdb: Arc<fontdb::Database>` を渡す。
- database は、`AssetStore` が持つ `fonts: Arc<fontdb::Database>` を使う。
  - 作成時に `load_system_fonts()` する。
  - `AssetStore::register_font(bytes: Arc<[u8]>)` で `load_font_data` して作り直す（`Arc::make_mut`）。
- engine の `ArvaneAssets::register_font` は、text engine への登録に加えて `store.register_font` を呼ぶ。
- **font の更新:** usvg は parse の時に text を輪郭へ変換して tree に保持するので、raster だけを捨てても font の変更は反映されない。
  - SVG の entry の parse の key に、font の generation を含める。
  - font が増えたら、SVG の entry をすべて `Pending` に戻し、元の bytes から parse し直す（既存の再読み込みの経路を使う）。
  - 古い generation の parse と raster の job の結果は、届いても捨てる（既存の `Entry.generation` の比較を使う）。
- **SVG の予算:** 既存の tree の見積りに、次を加える。
  - 埋め込みの raster 画像ごとに、decode 後の大きさ（w × h × 4。header から読む）
  - text の node ごとに、文字数 × 512 bytes（輪郭へ変換した後の tree の保持量の見積りである。parse の途中の割り当ての上限ではない）
  - 見積りが予算の 1/4 を超える SVG は、既存の TooLarge として失敗させる。parse の途中の割り当ては、既存どおり入力の bytes の上限で抑える。
- `image_href_resolver` は `resolve_string` を None を返すまま保つ（外部 file を読まない）。`resolve_data`（data URL）は usvg の既定に戻す。resvg の `raster-images` feature で PNG/JPEG/GIF/WebP の data URL を描く。
- `svg.rs` の module 文書（text off、raster off）を新しい事実に書き換える。

### 9.10 色の解決（`kivact-style`）

- `Color` の内部表現 `Repr` に `Space(SpaceColor)` を加える。
- `Color::resolve_space(&self, theme) -> SpaceColor` で、gamut mapping をしない元の色を返す。`resolve` は `resolve_space(..).to_srgb()` とする。
- gradient の stop は `resolve_space` の値を wire に渡す（§5.6）。
- `Color::mix` と `Color::mix_in` は、両端の `resolve_space` の値を `interpolate`（`ColorInterpolation::DEFAULT`、Oklab）して `Repr::Space` の色を返す（gamut mapping は最後の `resolve` で一度だけ行う）。
- `Animatable for Color` は、元の色の Oklab の (L, a, b, alpha) を premultiply した 4 成分で補間する。
  - `to_components`: `resolve_space` → Oklab → (L·α, a·α, b·α, α)
  - `from_components`: unpremultiply → Oklab の `SpaceColor` を持つ `Repr::Space`
- engine の state transition（§12.9）は、wire の `PaintProps` の色が sRGB の `Rgba` なので、sRGB に写した両端から Oklab で補間する。広色域の端点の transition の途中の色は、CSS の補間と一致しないことがある（§16.4 の CONTRACT-COLOR）。
- theme token の解決は、現在の owner の theme（`theme_of(Owner::current())`）で行う。owner が無ければ `Theme::default()` で行う。

---

## 10. drawable（HTML-in-Canvas）

### 10.1 使い方

canvas の描画関数が、実 view の部分木（drawable）を任意の変換の下に描く。drawable の中身は通常の view なので、focus、キーボード、IME、支援技術、test の検索がそのまま働く。hit test と a11y の位置は、描いた変換から engine が自動で求める。この点で、ブラウザの HTML-in-Canvas（3D context では写像を手で渡す）より手間が少ない。

```rust
use kivact::prelude::*;
use kivact::draw::{Layer, Transform};
use kivact::view::drawable;

fn card_in_3d() -> impl IntoView {
    let angle = signal(20.0_f64);
    let card = drawable();
    canvas(move |d, size| {
        let at = Point::new((size.width - card.size().get().width) / 2.0, 40.0);
        d.layer(
            Layer::new().transform(Transform::new().perspective(800.0).rotate_y(angle.get())),
            |d| d.drawable(card, at),
        );
    })
    .drawable(card, || column((text("Settings"), button("Apply").on_press(|| ()))))
    .style(style().size(pct(100), 360))
}
```

### 10.2 core の API（`kivact-core/src/view/drawable.rs`、新規）

```rust
/// A view subtree a canvas draws (HTML-in-Canvas). Create it in setup, mount
/// its content with `Node::drawable` (or `Viewport::drawable` to export it),
/// draw it with `Draw::drawable`.
#[derive(Clone, Copy, PartialEq)]
pub struct Drawable { app: AppId, id: DrawableId, state: Owned<Rc<DrawableState>> }

#[track_caller]
pub fn drawable() -> Drawable;   // setup only (API-12 panic with a fix hint otherwise)

impl Drawable {
    /// The laid-out size of the content; zero until it is laid out.
    pub fn size(&self) -> Value<Size>;
    /// A thread-safe handle that exports frames of a viewport-hosted drawable (§10.5).
    pub fn share(&self) -> DrawableShare;
}

impl Node {
    /// Mount `content` as a drawable of this node. The content is laid out
    /// as an absolutely positioned child at (0, 0) of this node's padding box
    /// (its own style may move or size it) and is drawn only where this
    /// node's `draw`/`draw_over` scene calls `Draw::drawable`.
    pub fn drawable<V: IntoView>(self, drawable: Drawable, content: impl IntoViewFn<V>) -> Self;
}
impl Viewport {
    /// Mount `content` as an exported drawable: laid out like `Node::drawable`,
    /// never drawn on screen, rendered for `Drawable::share`.
    pub fn drawable<V: IntoView>(self, drawable: Drawable, content: impl IntoViewFn<V>) -> Self;
}
```

**module の配置（循環を作らない）:**

- `kivact-core/src/view/drawable.rs` は宣言と再公開だけにする。
- `view/drawable/handle.rs`: `Drawable`、`DrawableState`、`DrawableBinding`、`drawable()`、`Drawable::{size, share}`。builder も mount も参照しない。
- `view/drawable/mount.rs`: drawable の root の mount（binding の検査と `DrawableRole` の書き込み）と、`impl Node { pub fn drawable }`、`impl Viewport { pub fn drawable }`。`elements::builders::node` と `handle` を参照する。
- `view/elements/builders.rs` と `viewport.rs` は、drawable を参照しない（既存の `view/mount/drawable.rs` の中身は `drawable/mount.rs` へ移す）。
- `view/drawable_share.rs` は `handle` と `crate::sync` を参照する。

**binding（一つの drawable は同時に一か所にだけ mount される）:**

```rust
// kivact-core/src/view/drawable.rs
pub(crate) enum DrawableBinding {
    Unbound,
    /// Mounted: its root node and the node that hosts it.
    Bound { root: NodeId, host: NodeId },
}
// DrawableState { owner: Owner, binding: Cell<DrawableBinding>, share: … }
```

- `Node::drawable` と `Viewport::drawable` は、mount の前に、既存の `NodeRef::bind_target` と同じ順で検査し、当たれば中身を mount せずに報告する。
  - drawable が別の App のもの: `ErrorKind::WrongApp`（"`.drawable(d)` got a Drawable of another App; it was not mounted"）
  - drawable の owner が dispose 済み: `ErrorKind::Disposed`
  - binding が `Bound` で、その root が tree に残っている: `ErrorKind::Message`（"Drawable is already mounted"）
- 検査を通ったら mount し、`Bound { root, host }` にする。
- root を unmount したとき、binding の root がその root と一致するときだけ `Unbound` に戻す（後の mount の binding を消さない）。unmount の後は、別の場所に mount し直せる。
- engine の entry（§10.3）も、remove の時に entry の root が一致するときだけ消す。
- **`Draw::drawable` の host の検査:** engine は drawable を、その entry の host の `draw`/`draw_over` の scene の中でだけ配置する。他の node の scene の `DrawCmd::Drawable` は描かず、その node の region の errors に `"drawable {id} is not hosted by this node"` を一度報告する。

**mount の手順:**

- `Node::drawable` は、子として drawable root の node を足す。中身は `content` の view とし、style は `style().position(Position::Absolute).top(0).left(0)`、properties の `drawable` は `Some(DrawableRole { id, mode: DrawableMode::Canvas })` とする。
- `Viewport::drawable` は `mode: DrawableMode::Export` にする。

```rust
// kivact-engine contract (NodeProperties)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawableRole { pub id: DrawableId, pub mode: DrawableMode }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawableMode { Canvas, Export }
```

**style 側の API（kivact-style の draw）:**

```rust
/// What `Draw::drawable` accepts; implemented by `kivact::view::Drawable`.
// xtask-allow(internal-contract): The drawable handle lives in core; the recorder only needs its id.
pub trait DrawableSource { #[doc(hidden)] fn drawable_id(&self) -> DrawableId; }

impl Draw {
    /// Draw a drawable's content with its top-left corner at `at`, under the
    /// open layers and transforms. Drawing the same drawable twice in one
    /// recording draws only the first and reports the second (as region errors).
    pub fn drawable(&mut self, drawable: impl DrawableSource, at: Point);
}
```

`Draw::drawable` は `DrawCmd::Drawable { id, at }` を記録する。2 回目以降は記録せず、`Recording::region_errors` に `"drawable {id} drawn twice"` を加える。

### 10.3 engine の配置（`model/drawables.rs`、新規）

```rust
#[derive(Default)]
pub(crate) struct Drawables(pub(super) BTreeMap<DrawableId, DrawableEntry>);
pub(crate) struct DrawableEntry { pub(crate) root: NodeId, pub(crate) host: NodeId, pub(crate) placement: Placement }
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Placement {
    /// Canvas mode, not drawn by the host's current scenes.
    Unplaced,
    Drawn(DrawnPlacement),
    /// Export mode: the app's geometry (drawable box → host canvas space), if any.
    Exported { geometry: Option<Projective> },
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DrawnPlacement {
    /// Drawable box space → host canvas (padding box) space.
    pub(crate) map: Projective,
    pub(crate) slot: u8,
    pub(crate) order: u32,
    /// Clips open where it was drawn, each with the map from host canvas
    /// space to the space the clip was declared in; all apply.
    pub(crate) clips: Vec<(Projective, Shape)>,
    /// An enclosing Draw layer with backface Hidden shows its back (§8.5).
    pub(crate) back_hidden: bool,
}
```

**entry の登録と削除:** drawable root の node を mount したときに entry を作り、remove したときに消す。

**配置は一つの関数で決め、書き手を一つにする。**

- **Canvas mode:**
  - `fn drawable_placements(host: &HostNode) -> BTreeMap<DrawableId, DrawnPlacement>` は、host の二つの scene（slot 0、slot 1 の順）の DrawCmd を順に読み、次のように配置を作る。
    1. 開いている変換の stack を作る。`PushTransform` は affine、`PushLayer` の `transform: Some(m)` は 4×4 である。
    2. clip の stack を作る。`PushClip` の矩形と `DrawingLayer::clip` の shape を、宣言された空間で記録する。
    3. backface の stack を作る。各 `DrawingLayer` の `backface` と、その layer の行列（次の項の規則で選ぶ）を積む。
    4. `Drawable { id, at }` に着いたら、配置を作る。
       - map は translate(at) から始め、stack を内側から外側へ合成して平坦化する。Draw の layer の plane が Flat なら各段で平坦化する（§7.2 と同じ）。
       - clips は、その時点の clip の stack を、host canvas の空間からそれぞれの空間への写像と組にしたもの。
       - back_hidden は、backface の stack の各 Hidden の layer について、§8.5 と同じ行列の選び方で判定する。Flat の親の中の layer は自身の 4×4（その layer の transform に親の Draw layer の perspective を含めたもの。累積しない）、`Preserve3d` の親の中の layer はコンテキストの中の累積の 4×4 を使う。どれかが `shows_back()` なら true。compositor の `CompositeLayer` の判定と同じ関数 `Matrix4::shows_back` を、同じ行列に適用する。
  - 同じ drawable が host 全体で二度以上現れたら、最初のもの（slot 0 が先、同じ slot では order の小さいもの）だけを配置とし、残りを `Recording::region_errors` と同じ経路で一度報告する。
  - `Model::set_scene(host, slot, list)` は、scene を保存した後、この関数で host の全 drawable の配置を作り直して書く（片方の slot の更新で、もう片方の配置を失わない）。結果に無い drawable は `Unplaced` にする。
  - paint の lower（§10.4）も、同じ関数の結果で描く drawable を決める。描画と幾何は同じ結果を使う。
- **Export mode:** host request `SetDrawableGeometry { id, generation, geometry: Option<Matrix4> }`（§10.5）だけが書く。値は `Exported { geometry: geometry.map(|m| m.flatten()) }` とする。generation が現在の request と違う要求は捨てる。

**mask と hit:** mask は hit test に影響しない（CSS と同じ）。clip と backface だけを hit に反映する。

### 10.4 paint、hit test、a11y

**paint:**

- paint list の walk は、drawable root を通常の子として walk しない（`is_layer()` と同じ扱い）。
- walk の最後に、Canvas mode の各 drawable root を origin (0, 0) から walk し、`PaintList::drawables: HashMap<DrawableId, Range<usize>>` に entry の範囲を記録する。
- host の scene の lower で `DrawCmd::Drawable { id, at }` に着いたら、§10.3 の `drawable_placements` の結果で、それが配置の勝者かを確かめる。勝者でなければ描かない。
- 勝者なら、その範囲の entry で子 layer（`LayerKey::Drawable(id)`、R6）を作る。transform は、scene の現在の affine と translate(at) の合成である。それを `LayerItem::Layer` として出す（§6.3 の chunk の分割に従い、開いている clip は item の clips になる）。
- Export mode の drawable は §10.5 の export として描く。

**hit test:** host の node で、次の順に調べる。

1. slot 1（draw_over）に描かれた drawable（order の降順）
2. 子
3. slot 0（draw）に描かれた drawable（order の降順）
4. region

drawable の判定は次のとおり。

- back_hidden の drawable は当たらない。
- 点（host canvas の空間）が、clips のすべてに含まれる場合だけ候補にする。
- map の逆写像で点を drawable の box の座標へ写し、drawable root から通常の hit test をする。
- Export mode の drawable は、`Exported { geometry: Some(map) }` のときだけ、viewport の node の範囲で同様に調べる。
- `Unplaced` と `geometry: None` の drawable は当たらない。

**window_map:** drawable root の `to_parent` は `Matrix4::from_projective(map).then(T(border_left, border_top))` とする（map は host の padding box の空間へ写すので、host の border box の空間へ直す）。親は host である。これにより、drawable の中の node の rect、pointer の局所座標、caret、IME の位置は、描いた変換を通る。

**a11y:**

- drawable root は、host の a11y の子として出す（tree の順は mount 順）。
- bounds は §7.4 の規則で求める。clips で見えない部分は bounds から除かない（ブラウザと同じ）。
- `Unplaced`、back_hidden、`geometry: None` のときは bounds を付けない。node 自体は出す。そのために、意味情報の wire の bounds を `Option<Rect>` にする（§5.15）。

**layout:** drawable root は host の中で absolute に配置され、host の大きさに影響しない（taffy の absolute の規則）。

### 10.5 export（`DrawableShare`）

アプリの 3D エンジンが、drawable を自分の scene に貼るための経路である。frame の受け渡しは、CPU pixels と共有 GPU texture（feature `gpu-interop`）の二通りである。request と pool には generation（世代）を付け、古い request や pool に属する texture、frame、event を取り違えない（既存の逆方向の `ShareLink` の request token と同じ考え方）。

```rust
// kivact-core/src/view/drawable_share.rs
/// Send + Sync; usable from the app's render thread.
#[derive(Clone)]
pub struct DrawableShare { inner: std::sync::Arc<Inbox<ExportLink>> }

impl DrawableShare {
    /// Receive frames as CPU pixels (straight-alpha RGBA8). Starts a new generation.
    pub fn request_pixels(&self);
    /// Receive frames in textures shared with the app's device on `adapter`
    /// (the window's adapter: `kivact::gpu_interop::adapter()`). Starts a new generation.
    #[cfg(feature = "gpu-interop")]
    pub fn request_gpu(&self, adapter: GpuAdapter);
    /// Stop exporting (frees the window's pool). Starts a new generation.
    pub fn stop(&self);
    /// Device px per logical px of exported frames (default: the window's
    /// scale, followed when it changes).
    pub fn set_scale(&self, scale: f64);
    /// Where the app shows the drawable: drawable box (logical px) → the
    /// hosting viewport's local logical px; None hides it from hit testing.
    pub fn set_geometry(&self, geometry: Option<Matrix4>);
    /// The newest frame of the current generation, if any.
    pub fn latest(&self) -> Option<DrawableFrame>;
    /// Wait for what the consumer must handle next:
    /// - new shared textures not yet taken (after `request_gpu`, a resize or
    ///   a device change): import them (`kivact_gpu_share::import::Import::new`
    ///   tells the window when they are ready); GPU frames of that generation
    ///   come only after the import;
    /// - otherwise a frame of the current generation with `number > after`.
    ///   Frame numbers grow over the whole life of the drawable and never
    ///   restart, so `after` from an older generation returns the first frame
    ///   of the current one (pass 0 for "any frame").
    /// Errors: `Timeout`, `Closed` (disposed), `Failed` (the request failed,
    /// e.g. a build without GPU sharing). Never `Stale`.
    pub fn wait(&self, after: u64, timeout: std::time::Duration) -> Result<ShareUpdate, ShareError>;
    pub fn is_closed(&self) -> bool;
}

/// What `DrawableShare::wait` returns.
#[derive(Debug)]
pub enum ShareUpdate {
    #[cfg(feature = "gpu-interop")]
    Textures(DrawableTextures),
    Frame(DrawableFrame),
}

#[cfg(feature = "gpu-interop")]
#[derive(Debug)]
pub struct DrawableTextures { pub generation: u64, pub textures: SharedTextures }

#[derive(Clone, Debug)]
pub enum DrawableFrame {
    Pixels { generation: u64, number: u64, pixels: Pixels },
    /// The slot is ready at `value` on the pool's ready fence; signal the
    /// release fence with `value` when done sampling (SharedTextures contract).
    /// Use it only with the textures of the same `generation`.
    Gpu { generation: u64, number: u64, slot: u32, value: u64 },
}
```

**`ShareError` と `Inbox` の移動:**

- `ShareError` は、既存の `kivact::gpu_interop::ShareError` を `kivact-core::view` へ移し、両方の経路で使う（feature の外に置く。DEP-4）。
  - variant `Stale` は、既存の `ShareLink` の token 付きの操作だけが返す。`DrawableShare::wait` は Stale を返さない。待っている間に generation が変わったら、新しい generation の `Textures` か `Frame` を待ち続ける（§10.5 の consumer の繰り返しが Err で終わらない）。
- 既存の `Inbox`（`gpu_interop/import/inbox.rs`）は、feature の外の新しい module `kivact-core/src/sync.rs`（`//! Role: Thread-safe mailboxes shared by worker-thread handles.`）へ移し、`gpu_interop` と `view` の両方から使う。
  - `[module_rules.kivact-core.uses]` に `sync = []` を加え、`gpu_interop` と `view` の uses に `"sync"` を足す。

**状態（`ExportLink`、S-1 の enum）:**

```rust
pub(crate) enum ExportLink {
    Open(ExportOpen),
    /// The drawable was disposed.
    Closed,
}
pub(crate) struct ExportOpen {
    pub(crate) request: ExportRequest,
    pub(crate) geometry: Mailbox<Option<Matrix4>>,
    pub(crate) scale: Option<f64>,
    pub(crate) latest: Option<DrawableFrame>,
    /// Generation allocator (an ID counter, S-8).
    pub(crate) next_generation: u64,
    /// Frame number allocator over the drawable's life (an ID counter, S-8).
    pub(crate) next_number: u64,
}
pub(crate) enum ExportRequest {
    None { generation: u64 },
    Pixels { generation: u64 },
    /// The window has to make the pool; no GPU frames yet.
    GpuRequested { generation: u64, adapter: GpuAdapter },
    /// The window has no GPU device; retried on `GpuExportAvailable`.
    GpuWaiting { generation: u64, adapter: GpuAdapter },
    /// Textures made. `textures` is taken once by `wait` (as
    /// `ShareUpdate::Textures`). GPU frames flow once the consumer's import
    /// signals the pool's release fence (§8.12); the core does not track it.
    GpuProvided { generation: u64, adapter: GpuAdapter, textures: Option<SharedTextures>, size: (u32, u32) },
    Failed { generation: u64, error: UiError },
}
```

**`ExportOpen` の marker:** 各 field の直前に `xtask-allow(state-fields)` と、次の理由を書く。

- `geometry`: Mailbox である。
- `scale`: 利用者の設定値である。
- `latest`: 配送の記録（S-9）である。
- `next_generation`、`next_number`: ID の割り当てである。

**`ExportLink` の遷移の規則:**

- 状態の遷移は `ExportLink` の method だけが行う。
- `request_pixels`、`request_gpu`、`stop` は、`next_generation` を進めて新しい generation の状態にし、`latest` を捨てる。
- **generation の照合:** engine から届く `DrawableTextures` と `DrawableFrame` の event は、現在の generation と一致するものだけを受け付け、他は捨てる。
- **GPU の遷移表:** 次の表のとおりで、表に無い組の event は捨てる。g は現在の generation、「新 g」は `next_generation` を進めた値である。

| 状態 | event | 次の状態 |
|---|---|---|
| `GpuRequested { g }` | `DrawableTextures { g, Ok(t) }` | `GpuProvided { g, textures: Some(t) }` |
| `GpuRequested { g }` | `DrawableTextures { g, Err(kind NotReady) }` | `GpuWaiting { g }` |
| `GpuRequested { g }` | `DrawableTextures { g, Err(その他。Unavailable を含む) }` | `Failed { g }` |
| `GpuWaiting { g }` | `GpuExportAvailable` | 新 g の `GpuRequested` |
| `GpuProvided { g }` | `DrawableResized`、`DrawableExportLost { g }` | 新 g の `GpuRequested` |
| `GpuRequested { g }`、`GpuProvided { g }` | `GpuExportAvailable` | 変化なし |
| どの Gpu の状態でも | `request_pixels`、`request_gpu`、`stop` | 新 g の各状態 |

- **import の失敗と未接続:** アプリが import に失敗したか、import しない間は、`GpuProvided` のまま、pool の release fence が 1 にならない。window の提示は続き、export は見送られて再試行が続く（間隔は 100 ms まで伸びる）。GPU の frame は来ない。アプリは `stop` か `request_pixels` で抜ける。
- **`textures` の受け渡し:** `wait` が `take()` で一度だけ `ShareUpdate::Textures` として渡す。まだ取られていない textures があれば、`wait` は frame より先にそれを返す（import の前に frame を待つ循環が起きない）。
- **number:** core は、受け付けた frame に `next_number` を振る。number は generation をまたいで増え続ける。`wait(after)` は、現在の generation の frame で number > after のものを返す。after が古い generation の number でも、現在の generation の frame の number はそれより大きいので、待ち続けない。
- **consumer の標準の繰り返し:**

```rust
let mut seen = 0;
let mut imported = None; // the app's own imported textures
loop {
    match share.wait(seen, Duration::from_millis(100)) {
        Ok(ShareUpdate::Textures(t)) => {
            // kivact_gpu_share::import::Import; its success also tells the
            // window that these textures are ready for frames.
            imported = Some(Import::new(&device, &queue, t.textures)?);
        }
        Ok(ShareUpdate::Frame(frame)) => {
            seen = frame.number();
            draw_my_scene(imported.as_ref(), &frame);
        }
        Err(ShareError::Timeout) => continue,
        Err(error) => return Err(error.into()),
    }
}
```

  初回も resize も device の変更も、同じ繰り返しで扱える。この例は `docs/for-kivact-user/*/how-to-guide/drawables.md` にそのまま載せ、§17.2 の `documented_consumer_loop_survives_resize` で同じ形を試す。Pixels の consumer は `ShareUpdate::Frame` だけを受け取る。

  `DrawableFrame` に `number()` と `generation()` の method を加える。

**流れ:**

1. `DrawableShare` の method は、Inbox を更新して、`ThreadWaker` で UI thread を起こす。
2. core は turn の始めに各 drawable の Inbox を読み、次の host request を出す。
   - request の変化: `HostRequest::DrawableExport { id, generation, mode: ExportRequestMode, scale: Option<f64> }`。scale は `set_scale` の値で、None なら engine が window の scale を使う（window の scale が変わったら、engine は大きさの変化と同じく扱う）。core は window の scale を読まない（`view` は `platform` を参照しない。§0.3 の module の規則）。`ExportRequestMode` は `None`、`Pixels`、`Gpu(adapter)` である。`GpuRequested` と `GpuProvided` は `Gpu(adapter)`、`GpuWaiting` と `Failed` は `None` として送る。
   - geometry の Mailbox: `HostRequest::SetDrawableGeometry { id, generation, geometry }`
3. engine は mode が `Pixels` の drawable と、mode が `Gpu` で pool を作れた drawable について、毎フレーム `Composition::exports` に `Export { layer, sink: id, size: ⌈size × scale⌉, scale, generation, mode }` を加える。
4. GPU の mode の扱い:
   - **pool の作成:** engine は、`Gpu` の generation が変わったときに、`window.export_drawable(id, generation, adapter, size)` を呼ぶ。`Gpu` の間に大きさが変わったら、core に新しい generation を求める `HostEvent::DrawableResized { id }` を出し、core の新しい request を受けてから呼ぶ。consumer の import の前の export は compositor が見送る（§8.12）ので、engine は import を待たない。
   - **textures の返却:** 結果の textures は `HostEvent::DrawableTextures { id, generation, result }` で core へ返す。
   - **Gpu を離れるとき:** mode が Gpu から Pixels または None に変わったら、engine は `window.drop_export(id)` を呼ぶ。
5. compositor が返した `ExportedFrame` は、`Presented::Shown::exports`（§10.6）から `HostEvent::DrawableFrame(ExportedFrame)` のまま core へ返す（Pixels も Gpu も同じ envelope の generation を持つ）。core は、envelope の generation が現在と一致する frame だけで `latest` を更新し、number を振る。
6. **再試行（`ExportRetry`）:** `Presented::Shown::pending_exports`（§10.6）に入った sink について、engine は次の状態を持つ。

```rust
// kivact-engine/src/model/drawables.rs
pub(crate) enum ExportRetry {
    Idle,
    /// The last presented frame skipped this export. Times are the host
    /// clock's ns (`window.now_ns()`, virtual in tests), as the pump's deadlines.
    Pending { retry_at_ns: u64, interval_ns: u64 },
}
```

   - **遷移（提示ごとに確定する）:** `Presented::Shown` を受けるたびに、export 中の各 drawable の `ExportRetry` を、その提示の `pending_exports` だけから決める。
     - sink が `pending_exports` にある: `Idle` なら `Pending { retry_at_ns: now + period, interval_ns: period }`（period は display の 1 フレーム）。`Pending` なら interval を 2 倍（上限 100 ms）にして retry_at を更新する。
     - sink が `pending_exports` に無い: `Idle` にする（送れた場合も、内容が最後に送ったものへ戻って送る必要が無くなった場合も同じ）。
     - mode が `Gpu` でなくなったとき、drawable を破棄したときも `Idle` にする。
   - **frame の要求:** `Pending` の retry_at_ns を、pump の既存の deadline（timer と同じ ns の値）に加える。retry_at_ns に達したら、dirty が無くても frame を提示する。この frame の damage は空で、compositor は Export だけを処理する（画面の画素は変わらない）。`Instant` は使わない（仮想時計の試験で進まないため）。
   - consumer が止まっても、interval が 100 ms まで伸びるので busy loop にならない。
7. **喪失と復帰:** engine は pump ごとに `window.take_export_notices()` を読み、`Lost` を `HostEvent::DrawableExportLost(ExportLoss)`、`Available` を `HostEvent::GpuExportAvailable` で core へ返す（§8.12）。
8. drawable を dispose したら `Closed` にし、engine の export を `window.drop_export(id)` で止める。

**GPU の pool:**

- window の device から、`kivact-gpu-share` の `wgpu::Pool::new(device, adapter, PoolConfig { width, height, slots: 3, storage: false })` で作る。frame は `try_acquire`（§8.12）で取得する。consumer が止まっても、window の GPU の queue は待たない。
- window の device は、`gpu-interop` のとき export に必要な extension を持つよう、次のようにする。
  - Vulkan の `DEVICE_EXTENSIONS` は import と export で共通なので、`kivact_gpu_share::import::request_device` を `kivact_gpu_share::device::request_device` に移し、両方から使う（§4.1）。
- 共有 texture の契約（format、hand-off の状態、fence の意味）は、既存の `SharedTextures` の文書のとおりである。
  - ただし、export 側は window、import 側はアプリになる。
  - アプリは `kivact_gpu_share::import::Import` で取り込める。

**受入条件:** 次の場合に window の提示が止まらず、export だけが見送られるか作り直されることを、試験（§17.2）で確かめる。

- アプリが import しない（準備の合図が来ない）
- アプリが release しない（再試行の間隔が伸び、release の後に他の UI の変化が無くても最新の frame が届く）
- device を失った、window が CPU へ fallback した
- drawable を破棄した
- 大きさを変えた
- mode を切り替えた（Pixels → Gpu → Pixels → None）
- 静止した内容で request し直した

### 10.6 arvane を通る報告

compositor の報告（export した frame、見落とした外部 frame、shader の失敗）は、次のように engine へ届く。

- `arvane::Presented::Shown` に field `exports: Vec<ExportedFrame>` と `pending_exports: Vec<DrawableId>` を加える（§5.11 の kivact-types の型なので、engine から名前を書ける）。
- presenter は、`FrameReport` の残りを次のように処理する。
  - `missed`: 既存の `RendererChange::ExternalFrameMissed` で記録する（既存の振る舞いのまま）。
  - `shader_failures`: `RendererChange::ShaderFailed { message }` で、hash ごとに一度だけ記録する。
- engine の `Host::present` は、`Presented::Shown::exports` の各 frame を `HostEvent::DrawableFrame(frame)` にし、`pending_exports` で `ExportRetry` を更新する（§10.5）。
- `arvane::Window` に次の method を加える。feature の無い build では kind が `Unavailable` の `Failure` を返す（§8.12）。

```rust
impl Window<'_> {
    pub fn export_drawable(&mut self, sink: DrawableId, generation: u64, adapter: &GpuAdapter, size: (u32, u32)) -> Result<SharedTextures, Failure>;
    pub fn drop_export(&mut self, sink: DrawableId);
    /// Pool losses and GPU availability since the last call (§8.12).
    pub fn take_export_notices(&mut self) -> Vec<ExportNotice>;
}
```

- engine は §10.5 の流れでこれらを呼び、結果を `HostEvent::DrawableTextures { id: DrawableId, generation: u64, result: Result<SharedTextures, Failure> }` で core へ返す。
- `Presented::Shown` に `stats: FrameStats` も加える。CPU の presenter も、`CpuOutput::report`（§8.1）から同じ値を返す。
- `ExportNotice` は arvane が公開し、engine の `HostEvent` に写す（kivact-types には置かない。arvane と engine の間だけの値である）。
- **試験の制御（device の喪失）:** GPU の lane の E2E が device の喪失を起こせるよう、次の経路を加える。既存の `Screen::lose_surface` と同じ形の試験の制御である。
  - kivact-test: `Screen::lose_gpu_device(&self)`（GPU が無い run では何もしない）
  - engine: `ArvaneTestHost::lose_gpu_device(&mut self)`
  - arvane: `arvane::spi::lose_gpu_device(window: &mut Window<'_>)`。presenter の既存の `device_failed` の経路を、合成した `RenderError`（"test: device lost"）で通す。background の起動では既存どおり一度だけ再起動するので、`ExportNotice::Lost` の後に、再起動の完了で `ExportNotice::Available` が出る。
- **最後の統計の持ち主:** engine の `Host`（`host/pump.rs`）は field `last_stats: Option<FrameStats>`（S-9 の配送の記録。`xtask-allow(state-fields)` の理由に「最後に提示したフレームの報告。制御には使わない」と書く）を持ち、`Presented::Shown` のたびに上書きする。`FrameLog` が有効なら、同じ値を `FrameTimes` と並べて file にも書く。`last_stats` は FrameLog の有効・無効に依存しない（test host は `FrameLog::disabled()` を使うため）。
- **読み出し:** engine の `ArvaneTestHost`（`host/testing/test_host.rs`）に `pub fn frame_stats(&self) -> Option<FrameStats>` を加え、`Launch::Running` の `Host::last_stats` を返す（`Pending` なら None）。kivact-test の `Screen::frame_stats() -> FrameStats` はこれを読む（None なら panic し、「まだフレームを提示していない」と書く）（§8.15）。

---

## 11. 公開 API

名前は CSS の property 名と値を優先し、snake_case にする。動的な値は style の closure（`.style(move || …)`）で与える。setter は既存の macro（`setter!`、`color_setter!`、`text_setter!`）で書く。新しい Style の field は `resolve.rs` の `classify_properties!` に分類を足す（足さないとコンパイルできない）。

### 11.1 style（`kivact::style`、`kivact-style/src/sheet.rs`）

| setter | 引数 | wire |
|---|---|---|
| `rotate_x` | `impl Into<f64>`（度） | `ElementTransform::rotate[0]` |
| `rotate_y` | `impl Into<f64>`（度） | `rotate[1]` |
| `translate_z` | `impl Into<f64>`（px） | `translate[2]` |
| `transform_origin_z` | `impl Into<f64>`（px） | `origin_z` |
| `perspective` | `impl Into<f64>`（px） | `PaintProps::perspective`（origin は既定 50% 50%） |
| `perspective_origin` | `x: impl Into<Length>, y: impl Into<Length>` | `Perspective::origin` |
| `transform_style` | `TransformStyle` | `transform_style` |
| `backface_visibility` | `BackfaceVisibility` | `backface` |
| `will_change` | `WillChange` | `will_change` |
| `isolation` | `Isolation` | `isolation` |
| `border_top_style`、`border_right_style`、`border_bottom_style`、`border_left_style` | `BorderStyle` | `Border::styles` の各辺（既存の `border_style` は全辺） |
| `border_image` | `BorderImage` | `PaintProps::border_image` |
| `mask` | `impl Into<Mask>`（型を変更） | `Effects::masks` |
| `text_stroke` | `width: impl Into<TextLength>, color: impl ThemeColor` | `TextStyle::stroke` |
| `paint_order` | `PaintOrder` | `TextStyle::paint_order` |
| `font_variation_settings` | `impl IntoIterator<Item = impl Into<FontVariation>>` | `TextStyle::variations` |
| `font_optical_sizing` | `bool` | `TextStyle::optical_sizing` |
| `font_width` | `impl Into<f64>`（%、50..=200） | `TextStyle::font_width` |

**transform の合成:** style の個別 transform は、CSS と同じく translate → rotate → scale → transform の順に合成する。

- translate は (x, y, translate_z)。
- rotate は rotate（z）、rotate_y、rotate_x の順。rotate_x を最も内側にするので、Rz·Ry·Rx の正準形に合う。

**state style の `DELTA`:** 次を加える。

- rotate_x、rotate_y、translate_z、transform_origin_z
- perspective、perspective_origin
- transform_style、backface_visibility
- 4 辺の border style

**新しい公開名（`kivact-style/src/facade.rs` の明示リストに加える）:**

- 3D と合成: `TransformStyle`、`BackfaceVisibility`、`WillChange`、`Isolation`
- border image: `BorderImage`、`BorderImageRepeat`
- text: `PaintOrder`、`FontVariation`
- 色: `ColorSpace`、`HueInterpolation`、`oklch`、`oklab`、`lab`、`lch`、`hwb`

**値の型:**

```rust
// kivact-style/src/effect.rs
pub use kivact_types::{BackfaceVisibility, Isolation, TransformStyle, WillChange};

// kivact-style/src/paint/border_image.rs (new)
#[derive(Clone, Debug, PartialEq)]
pub struct BorderImage { /* source: PaintImage, slice: [Length; 4], fill, width: Option<[Length; 4]>, outset: [f64; 4], repeat: [BorderImageRepeat; 2] */ }
impl BorderImage {
    /// An image (Asset) or a Gradient as the source.
    pub fn new(source: impl Into<BorderImageSource>) -> BorderImage;
    pub fn slice(self, slice: impl Into<Insets>) -> BorderImage;            // px of the source
    pub fn slice_pct(self, slice: impl Into<Insets>) -> BorderImage;        // percent of the source
    pub fn fill(self) -> BorderImage;
    pub fn width(self, width: impl Into<Insets>) -> BorderImage;            // px; default: the border widths
    pub fn outset(self, outset: impl Into<Insets>) -> BorderImage;
    pub fn repeat(self, repeat: BorderImageRepeat) -> BorderImage;          // both axes
    pub fn repeat_xy(self, horizontal: BorderImageRepeat, vertical: BorderImageRepeat) -> BorderImage;
}
pub enum BorderImageSource { Image(Asset), Gradient(Gradient) }
// From<Asset>, From<&'static [u8; N]>, From<&'static str>, From<Gradient> for BorderImageSource.

// kivact-style/src/text.rs
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontVariation { /* tag: Option<[u8; 4]>, value: f64 */ }
impl FontVariation { pub fn new(tag: &str, value: impl Into<f64>) -> FontVariation; }
// From<(&str, f64)>, From<(&str, i32)>. A tag that is not 4 ASCII bytes is counted as unsupported (as FontFeature).
pub use kivact_types::PaintOrder;
```

**structural-options の検査:**

- `structural-options` の検査対象になる引数なしの builder flag は、`BorderImage::fill`、`Gradient::interpolate` のような値の setter であり、構造 option ではない。
- そのため `[structural_options].record_types` に `BorderImage`、`Mask`、`MaskLayer`、`ShaderPaint`、`ShaderFilter`、`FilterGraph` を加える。
- 引数なしの構造 flag の `Timeline::alternate`、`Timeline::essential`、`FrameLoop::essential`、`ShaderPaint::essential`、`ShaderFilter::essential` は、既存の `Transition::essential`、`Transition::alternate` と同じく `[structural_options].list` に加える。

### 11.2 draw（`kivact::draw`、`kivact-core/src/draw.rs`）

**Transform（`kivact-style/src/transform.rs`）に追加する method:**

| method | 意味 |
|---|---|
| `rotate_x(deg)`、`rotate_y(deg)` | CSS rotateX、rotateY |
| `rotate_3d(x, y, z, deg)` | CSS rotate3d |
| `translate_z(px)` | CSS translateZ |
| `translate_3d(x: impl Into<Length>, y: impl Into<Length>, z: impl Into<f64>)` | CSS translate3d |
| `scale_z(f)`、`scale_3d(x, y, z)` | CSS scaleZ、scale3d |
| `perspective(d)` | CSS の perspective() 関数。最初の関数でなければ unsupported に数え、最初にあるものとして扱う |
| `matrix_3d(m: [f64; 16])` | CSS matrix3d（列ごと）。最下行が (0, 0, 0, 1) でなければ unsupported に数え、最下行を (0, 0, 0, 1) にする |

**Transform の内部表現:**

- `translate: [f64; 3], per_size: [f64; 4], rotate: [f64; 3], skew: [f64; 3], scale: [f64; 3], perspective_inverse: f64, stage: u8` に置き換える。
- 正準形の段（stage）は translate(1)、rotate_z(2)、rotate_y(3)、rotate_x(4)、skew(5)、scale(6) である。
- 同じ段の関数のうち、translate（和）、同じ軸の rotate（角度の和）、scale（積）は成分に直接吸収する。後の段の関数も、その段の成分に直接書く。それ以外（前の段の関数、skew の重ね掛け、`matrix`、`matrix_3d`、`rotate_3d`）は、現在の値を行列にして積を取り、§5.3 で分解し直す。
- 百分率の translate が 2D でない関数の後に来た場合は、百分率の部分を先行する線形写像の x、y の行で写す。z への寄与は捨て、unsupported に数える。
- `Animatable for Transform` の成分は 17 個（translate 3、per_size 4、rotate 3、skew 3、scale 3、perspective_inverse 1）である。
- `to_wire` は `ElementTransform` を作る。

**Layer（`kivact-style/src/draw/layer.rs`）に追加する method:**

| method | 意味 |
|---|---|
| `transform(t: Transform)` | 既存。3D の Transform なら `DrawingLayer::transform: Some(matrix)`、2D なら従来どおり PushTransform |
| `transform_style(TransformStyle)` | `DrawingLayer::transform_style` |
| `backface_visibility(BackfaceVisibility)` | `DrawingLayer::backface` |
| `composite(Composite)` | Canvas の globalCompositeOperation の Porter–Duff 部分。blend と同時に指定したら、後の指定が勝つ（blend を指定すると composite は SourceOver に、composite を指定すると blend は Normal になる） |
| `mask(impl Into<Mask>)` | 型を変更 |

**Draw に追加する method:**

```rust
impl Draw {
    pub fn drawable(&mut self, drawable: impl DrawableSource, at: Point);
    pub fn border_image(&mut self, rect: Rect, image: &BorderImage);
    /// The text's glyph outlines as a path (Canvas has no equivalent; SVG does).
    pub fn text_path(&self, text: &str, pos: Point, style: &TextFormat) -> Path;
    pub fn paragraph_path(&self, paragraph: &TextParagraph, origin: Point) -> Path;
    /// Changed: any paint (was a theme color).
    pub fn paragraph(&mut self, paragraph: &TextParagraph, origin: Point, fill: impl Into<Paint>);
    pub fn stroke_paragraph(&mut self, paragraph: &TextParagraph, origin: Point, stroke: &Stroke, paint: impl Into<Paint>);
}
```

- **TextFormat に追加する field:**
  - `fill: Option<Paint>`: None なら `color` で塗る。
  - `stroke: Option<(Stroke, Paint)>`
  - `variations: Vec<FontVariation>`
  - `font_width: f64`（既定 100）
- **輪郭の取得:** `text_path` と `paragraph_path` は、hidden の text measurer bridge（`kivact_types::TextShaper`）に追加する `fn outline(&self, run: &GlyphRun) -> Vec<PathCmd>` を使う。engine の実装は `arvane::glyph_outline` を呼ぶ。
- **新しい公開名（`kivact::draw`）:** `Composite`、`Mask`、`MaskLayer`、`MaskMode`、`MaskComposite`、`BorderImage`、`BorderImageRepeat`、`FilterGraph`、`FilterInput`、`PaintShader`、`FilterShader`、`ShaderPaint`、`ShaderFilter`、`ShaderValue`、`DrawableSource`
- **filter graph の値の型:** `Light`、`Channel`、`TransferFn`、`CompositeOp`、`EdgeMode`、`MorphologyOp`、`TurbulenceKind` は `kivact::draw::filter` module から公開する（`kivact_types` の型をそのまま再公開する）。

### 11.2.1 FilterGraph の builder（`kivact-style/src/effect/filter_graph.rs`）

SVG の filter primitive を、結果の handle をつないで書く。primitive の番号を利用者が数える必要はない。最後に追加した primitive の結果が出力になる（SVG と同じ）。

```rust
use kivact::draw::{Filter, FilterGraph, FilterInput};
use kivact::draw::filter::Channel;

// A glitch: displace the element by noise.
let mut g = FilterGraph::new();
let noise = g.turbulence(0.02, 2, 3.0);
g.displacement_map(FilterInput::SOURCE, noise, 12.0, Channel::R, Channel::G);
let glitch = Filter::new().graph(g);

// A glow: blur the alpha, color it, put the element on top.
let mut g = FilterGraph::new();
let blurred = g.blur(FilterInput::SOURCE_ALPHA, 6.0);
let tinted = g.flood_in(blurred, ColorToken::Accent);
g.merge([tinted, FilterInput::SOURCE]);
let glow = Filter::new().graph(g);
```

```rust
/// A handle to a primitive's result, or the element itself. Copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilterInput { pub(crate) graph: u64, pub(crate) input: kivact_types::FilterInput }
impl FilterInput {
    /// Belong to no graph (graph 0): valid in every graph.
    pub const SOURCE: FilterInput;
    pub const SOURCE_ALPHA: FilterInput;
}

/// A filter graph being built. Not Clone: a handle names one graph, so a
/// copy would let handles of one graph slip into another unnoticed.
/// `Filter::graph` consumes it and keeps the built graph (which is Clone).
#[derive(Debug)]
pub struct FilterGraph { /* id: u64, primitives with style Colors, margin, color_space */ }
impl Default for FilterGraph { fn default() -> Self { FilterGraph::new() } }
impl FilterGraph {
    pub fn new() -> FilterGraph;
    /// Extend the filter region beyond the element (default: 10% of its size).
    pub fn margin(&mut self, margin: impl Into<Insets>) -> &mut FilterGraph;
    /// Compute in sRGB instead of linear RGB (SVG color-interpolation-filters).
    pub fn srgb(&mut self) -> &mut FilterGraph;
    pub fn blur(&mut self, input: FilterInput, std_dev: impl Into<f64>) -> FilterInput;
    pub fn blur_xy(&mut self, input: FilterInput, x: impl Into<f64>, y: impl Into<f64>) -> FilterInput;
    pub fn blend(&mut self, top: FilterInput, bottom: FilterInput, mode: BlendMode) -> FilterInput;
    pub fn color_matrix(&mut self, input: FilterInput, matrix: [f64; 20]) -> FilterInput;
    pub fn saturate(&mut self, input: FilterInput, amount: impl Into<f64>) -> FilterInput;
    pub fn hue_rotate(&mut self, input: FilterInput, deg: impl Into<f64>) -> FilterInput;
    pub fn luminance_to_alpha(&mut self, input: FilterInput) -> FilterInput;
    pub fn component_transfer(&mut self, input: FilterInput, r: TransferFn, g: TransferFn, b: TransferFn, a: TransferFn) -> FilterInput;
    pub fn composite(&mut self, top: FilterInput, bottom: FilterInput, op: CompositeOp) -> FilterInput;
    /// Divisor: the kernel sum (1 if 0); target: the center; edge: Duplicate.
    pub fn convolve(&mut self, input: FilterInput, order: (u32, u32), kernel: impl IntoIterator<Item = f64>) -> FilterInput;
    pub fn diffuse_lighting(&mut self, input: FilterInput, light: Light, color: impl ThemeColor) -> FilterInput;
    pub fn specular_lighting(&mut self, input: FilterInput, light: Light, color: impl ThemeColor, exponent: impl Into<f64>) -> FilterInput;
    pub fn displacement_map(&mut self, input: FilterInput, map: FilterInput, scale: impl Into<f64>, x: Channel, y: Channel) -> FilterInput;
    pub fn drop_shadow(&mut self, input: FilterInput, x: impl Into<f64>, y: impl Into<f64>, std_dev: impl Into<f64>, color: impl ThemeColor) -> FilterInput;
    pub fn flood(&mut self, color: impl ThemeColor) -> FilterInput;
    /// `color` where `input` has alpha (feFlood + feComposite in).
    pub fn flood_in(&mut self, input: FilterInput, color: impl ThemeColor) -> FilterInput;
    pub fn image(&mut self, image: impl Into<Asset>, rect: Rect) -> FilterInput;
    pub fn merge(&mut self, inputs: impl IntoIterator<Item = FilterInput>) -> FilterInput;
    pub fn erode(&mut self, input: FilterInput, radius: impl Into<f64>) -> FilterInput;
    pub fn dilate(&mut self, input: FilterInput, radius: impl Into<f64>) -> FilterInput;
    pub fn offset(&mut self, input: FilterInput, x: impl Into<f64>, y: impl Into<f64>) -> FilterInput;
    pub fn tile(&mut self, input: FilterInput, tile: Rect) -> FilterInput;
    pub fn turbulence(&mut self, base_frequency: impl Into<f64>, octaves: u32, seed: impl Into<f64>) -> FilterInput;
    pub fn fractal_noise(&mut self, base_frequency: impl Into<f64>, octaves: u32, seed: impl Into<f64>) -> FilterInput;
}
impl Filter { pub fn graph(self, graph: FilterGraph) -> Filter; }
```

**builder の規則:**

- 各 method は primitive を一つ追加し、その結果の handle を返す。`flood_in` だけは、Flood と Composite(In) の二つを追加し、Composite の handle を返す。
- **graph の id:** `FilterGraph::new` は、process の中で一意の id を `static NEXT_GRAPH: AtomicU64`（1 から。0 は「graph に属さない」）から取る。各 method は、返す handle に自身の id を入れる。
- **handle の検証:** handle は、graph が 0（`SOURCE`、`SOURCE_ALPHA`）か、それを作った graph の id と等しいときだけ有効である。
  - 別の graph の handle（id が違う）を渡すと、その入力を Source に置き換える。id が一致すれば、handle は必ずその graph の前の primitive を指す（自分より後を指す handle は作れない）。
  - 置き換えたことは、method の中で記録し、graph の検査の結果として報告する。
- **値の境界（三段）:**
  1. **作成中:** builder の `FilterGraph`（Clone でない。graph の id、`ThemeColor`、`Asset` を持つ）。
  2. **style の値:** `Filter::graph(self, graph)` は builder を消費し、id を落とした不変の `AuthoredGraph`（kivact-style の private な型。primitive の色は `ThemeColor`、画像は `Asset`）を `Arc` で持つ。`Filter` はこれで Clone できる。
  3. **wire:** style の resolve が、その時点の theme で色を `Rgba` に解決し、`kivact_types::FilterGraph<Asset>` にする（`FilterOp::Graph`）。
  4. **composition:** engine の lower（U51）が `map_images` で `Asset` を natural pixels（読み込み前は 1×1 の透明）に解決し、`FilterGraph<Pixels>` にする（`LayerFilter::Graph`）。フレームごとに作り直してよい。layer の signature は graph の `Arc` の pointer ではなく primitive の値と、Image の pixels の data の pointer（`Pixels::rgba` の Arc。§8.6）を畳み込むので、作り直しても cache は保たれる。
- **検査の時点:** `compile_style`（または Draw の recording）の時点で、§5.9 の `validate` を通す。
  - 失敗したら、その filter の graph の op だけを捨てる。
  - `UiError(ErrorKind::Message, "filter graph: primitive N: …")` を、style の場所に一度だけ報告する。


### 11.3 shader

#### 11.3.1 公開 API（`kivact-style/src/shader.rs`、新規。`kivact::draw` から公開）

```rust
// kivact-runtime/src/reactive/error.rs: a new variant
//     /// A shader that did not validate or a mismatched shader parameter;
//     /// the message starts with "line L, column C: " when it has a position.
//     Shader,
// Display: "shader: {message}".

#[derive(Clone, Debug, PartialEq)]
pub struct PaintShader(std::sync::Arc<ShaderProgram>);
impl PaintShader {
    /// Validate and assemble a paint shader. Errors are ErrorKind::Shader
    /// with "line L, column C: message" positions in `source`.
    #[track_caller]
    pub fn wgsl(source: &str) -> Result<PaintShader, UiError>;
    pub fn paint(&self) -> ShaderPaint;
}
#[derive(Clone, Debug, PartialEq)]
pub struct FilterShader(std::sync::Arc<ShaderProgram>);
impl FilterShader {
    #[track_caller]
    pub fn wgsl(source: &str) -> Result<FilterShader, UiError>;
    pub fn filter(&self) -> ShaderFilter;
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShaderPaint { /* program, params: Vec<(String, ShaderValue)>, images: [Option<Asset>; 4], clock, essential, fallback: Option<Paint> */ }
impl ShaderPaint {
    /// Set a field of the WGSL `Params` struct.
    pub fn param(self, name: &str, value: impl Into<ShaderValue>) -> ShaderPaint;
    /// Bind an image to `image(slot, uv)`; slot 0..=3.
    pub fn image(self, slot: u32, image: impl Into<Asset>) -> ShaderPaint;
    /// Advance `context.time` at `rate`.
    pub fn animate(self, rate: FrameRate) -> ShaderPaint;
    /// Keep the clock under reduced motion.
    pub fn essential(self) -> ShaderPaint;
    /// What CPU rendering (and a failed GPU pipeline) draws instead; default: nothing.
    pub fn fallback(self, paint: impl Into<Paint>) -> ShaderPaint;
}
impl From<ShaderPaint> for Paint;

#[derive(Clone, Debug, PartialEq)]
pub struct ShaderFilter { /* program, params, images, clock, essential, margin */ }
impl ShaderFilter {
    pub fn param(self, name: &str, value: impl Into<ShaderValue>) -> ShaderFilter;
    pub fn image(self, slot: u32, image: impl Into<Asset>) -> ShaderFilter;
    pub fn animate(self, rate: FrameRate) -> ShaderFilter;
    pub fn essential(self) -> ShaderFilter;
    /// px the output may extend beyond the element (glows); default 0.
    pub fn margin(self, px: impl Into<f64>) -> ShaderFilter;
}
impl Filter { pub fn shader(self, shader: ShaderFilter) -> Filter; pub fn graph(self, graph: FilterGraph) -> Filter; }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShaderValue { F32(f64), Vec2([f64; 2]), Vec3([f64; 3]), Vec4([f64; 4]), Color(Color), I32(i32), U32(u32) }
// From<f64>, From<f32>, From<i32>, From<u32>, From<(f64, f64)>, From<(f64, f64, f64)>,
// From<(f64, f64, f64, f64)>, From<Point>, From<Size>, From<Color>, From<ColorToken>.
```

- `FrameRate` は runtime の型なので、kivact-style からは `kivact_runtime::FrameRate` を使う（style は runtime に依存できる）。wire では `ShaderClock` に写す。
- param の検証は `compile_style` で行う。次の場合は、その param を捨てて `UiError(ErrorKind::Shader)` を一度報告する。
  - 名前が `Params` に無い
  - 型が合わない（Color は vec4 にだけ合う）
- `Params` の field に値を与えなかった場合は 0 とする。

#### 11.3.2 利用者が書く WGSL

```wgsl
// A paint shader. `position` is in logical px from the shader box's
// top-left; `context.size` is the box size. Return a straight-alpha sRGB color.
struct Params { speed: f32, tint: vec4<f32> }

fn paint(position: vec2<f32>, context: Context, params: Params) -> vec4<f32> {
    let uv = position / context.size;
    let wave = 0.5 + 0.5 * sin(uv.x * 12.0 + context.time * params.speed);
    return vec4<f32>(params.tint.rgb * wave, 1.0);
}
```

```wgsl
// A filter shader: the entry is `effect` (`filter` is a WGSL reserved word).
// `content(p)` reads the element's own pixels (straight alpha).
fn effect(position: vec2<f32>, context: Context) -> vec4<f32> {
    let c = content(position);
    let scan = 0.85 + 0.15 * step(0.5, fract(position.y * 0.5));
    return vec4<f32>(c.rgb * scan, c.a);
}
```

**規則:**

- **入口:**
  - paint shader は `fn paint(position: vec2<f32>, context: Context) -> vec4<f32>`、または第 3 引数 `params: Params` を持つ形を定義する。
  - filter shader は、同じ形の `fn effect(...)` を定義する（`filter` は WGSL の予約語なので入口名に使えない）。
- **`Params`:** 名前は `Params` 固定で、field の型は `f32`、`vec2<f32>`、`vec3<f32>`、`vec4<f32>`、`i32`、`u32` に限る。入れ子と配列は使えない。
- **使える helper:**
  - `image(slot: u32, uv: vec2<f32>) -> vec4<f32>`: straight alpha、bilinear、端で clamp
  - `content(position: vec2<f32>) -> vec4<f32>`: filter shader（`effect`）のみ。入力の straight alpha、範囲外は透明
  - `srgb_to_linear(c: vec3<f32>) -> vec3<f32>`
  - `linear_to_srgb(c: vec3<f32>) -> vec3<f32>`
- **書いてはいけないもの:** 違反は `wgsl()` の時点でエラーになる。
  - `@group`/`@binding` 付きの global
  - entry point（`@vertex`/`@fragment`/`@compute`）
  - `override`
  - `enable`
  - `kivact_` で始まる名前
  - `Context`、`image`、`content`、`srgb_to_linear`、`linear_to_srgb` の再定義
- **使える WGSL の範囲:** これら以外は WGSL のすべてを使える（関数、ループ、配列、`var<private>` 等）。

#### 11.3.3 組み立てる module（kivact-shader）

`kivact_shader::assemble(source: &str, kind: ShaderKind) -> Result<ShaderProgram, ShaderError>` は、次の順のテキストを連結して naga で parse・validate する。

1. prelude（下記。`kivact_params` の binding は、手順の 1 回目で `Params` struct が見つかったときだけ入れる）
2. 利用者のソース
3. 生成する入口のテキスト

parse は 2 回行う。

- 1 回目: prelude（`kivact_params` の行を除く）と利用者のソースだけを parse し、IR から次を読む。
  - `Params` struct の有無
  - `paint`/`effect` 関数の引数の数（2 か 3）
- 2 回目: 1 回目の結果で binding の行と ARGS を決め、完全な module を parse して validate する。
  - ARGS は引数が 3 のときだけ `, kivact_params` にする。
  - 引数が 3 で `Params` が無ければ、エラーにする。

```wgsl
// prelude
struct Context { size: vec2<f32>, scale: f32, time: f32 }
struct KivactDraw {
    map: mat3x3<f32>,              // target px → shader space (columns)
    target_size: vec2<f32>,
    content_origin: vec2<f32>,     // filter: shader-space point at content texel (0, 0)
    content_scale: f32,            // filter: content texels per shader unit
    opacity: f32,
}
@group(0) @binding(0) var<uniform> kivact_context: Context;
@group(0) @binding(1) var<uniform> kivact_params: Params;            // only with Params
@group(0) @binding(2) var kivact_sampler: sampler;
@group(0) @binding(3) var kivact_image0: texture_2d<f32>;
@group(0) @binding(4) var kivact_image1: texture_2d<f32>;
@group(0) @binding(5) var kivact_image2: texture_2d<f32>;
@group(0) @binding(6) var kivact_image3: texture_2d<f32>;
@group(0) @binding(7) var kivact_content: texture_2d<f32>;
@group(0) @binding(8) var kivact_coverage: texture_2d<f32>;
@group(0) @binding(9) var<uniform> kivact_draw: KivactDraw;
fn kivact_unpremul(c: vec4<f32>) -> vec4<f32> { if (c.a <= 0.0) { return vec4<f32>(0.0); } return vec4<f32>(c.rgb / c.a, c.a); }
fn image(slot: u32, uv: vec2<f32>) -> vec4<f32> {
    switch slot {
        case 0u: { return kivact_unpremul(textureSampleLevel(kivact_image0, kivact_sampler, uv, 0.0)); }
        case 1u: { return kivact_unpremul(textureSampleLevel(kivact_image1, kivact_sampler, uv, 0.0)); }
        case 2u: { return kivact_unpremul(textureSampleLevel(kivact_image2, kivact_sampler, uv, 0.0)); }
        default: { return kivact_unpremul(textureSampleLevel(kivact_image3, kivact_sampler, uv, 0.0)); }
    }
}
fn content(position: vec2<f32>) -> vec4<f32> {
    let texel = (position - kivact_draw.content_origin) * kivact_draw.content_scale;
    let size = vec2<f32>(textureDimensions(kivact_content));
    if (any(texel < vec2<f32>(0.0)) || any(texel > size)) { return vec4<f32>(0.0); }
    return kivact_unpremul(textureSampleLevel(kivact_content, kivact_sampler, texel / size, 0.0));
}
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> { return select(pow((c + 0.055) / 1.055, vec3<f32>(2.4)), c / 12.92, c <= vec3<f32>(0.04045)); }
fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> { return select(1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3<f32>(0.0031308)); }
```

```wgsl
// generated entry points (ENTRY = paint or effect; ARGS adds kivact_params when the entry takes params)
struct KivactVertex { @builtin(position) clip: vec4<f32>, @location(0) target_px: vec2<f32> }
@vertex fn kivact_vertex(@location(0) target_px: vec2<f32>) -> KivactVertex {
    let ndc = target_px / kivact_draw.target_size * 2.0 - 1.0;
    return KivactVertex(vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0), target_px);
}
@fragment fn kivact_fragment(v: KivactVertex) -> @location(0) vec4<f32> {
    let h = kivact_draw.map * vec3<f32>(v.target_px, 1.0);
    var c = clamp(ENTRY(h.xy / h.z, kivact_context ARGS), vec4<f32>(0.0), vec4<f32>(1.0));
    let cover = textureSampleLevel(kivact_coverage, kivact_sampler, v.target_px / kivact_draw.target_size, 0.0).a;
    let a = c.a * cover * kivact_draw.opacity;
    return vec4<f32>(c.rgb * a, a);
}
```

**ShaderError と位置:**

```rust
pub struct ShaderError { pub message: String, pub line: u32, pub column: u32 }
```

- naga の `ParseError::location(&text)` と `WithSpan` の span から位置を求める。prelude の行数を引いて、利用者のソースの行と列にする。
- 生成部分の位置になった場合は、`"internal: "` を前置する（起きないはずの不具合として扱う）。
- validator は `ValidationFlags::all()`、`Capabilities::empty()` とする。

**parse 後の IR で確かめること:**

- `module.entry_points` は生成した 2 つだけである。
- `module.overrides` は空である。
- `@binding` 付きの global は prelude のものだけである。
- `paint`/`effect` 関数の引数と戻り値の型が規則どおりである。
- `Params` の member の型が規則どおりである。

**ParamSlot のレイアウト:**

- `naga::proc::Layouter` で `Params` の member の offset を求める。
- params_size は struct の大きさを 16 の倍数に切り上げた値とする。`Params` が無ければ 16 とする。

**hash:** module テキストの FNV-1a 64 bit を `ShaderProgram::hash` にする。

#### 11.3.4 compositor の pipeline

- vertex buffer は `vec2<f32>`（target px）一つで、多角形の fan の頂点を入れる。
- paint shader:
  - color target: `Rgba8Unorm`
  - blend: `src = One, dst = OneMinusSrcAlpha`（premultiplied の SourceOver）
- filter shader:
  - blend なし（出力 target を上書きする）。
  - coverage は 1×1 の白とする。
  - content は入力 target とし、`content_origin` と `content_scale` は入力 target の写像から求める。
- `kivact_context.scale` は `px_per_unit`、`size` は shader box の大きさ（filter では layer の bounds の大きさ）とする。
- image の slot に何も無いときは 1×1 の透明 texture を bind する。
- sampler は linear、clamp-to-edge とする。

#### 11.3.5 clock

`context.time` は秒で、`ShaderClock` に従う。

| clock | time |
|---|---|
| Still | 常に 0 |
| Hz(r)、Display | engine の motion clock（surface が非 active の間は止まる、単調）の秒 |

**reduced motion:** `essential` でない shader では、reduced motion の間 time = 0 とする。

**frame の要求:**

- engine は paint list の再構築ごとに、`ShaderClocks: BTreeMap<NodeId, ClockNeed>`（純粋 cache）を作る。対象は、描画される node のうち、clock が Still でなく、reduced motion で止まらない shader を持つものである。

  ```rust
  #[derive(Clone, Copy, Debug, PartialEq)]
  pub(crate) enum ClockNeed { Display, Hz(f64) }
  ```

- node が複数の shader を持つ場合は、次のように集約する。
  - 一つでも Display なら Display。
  - そうでなければ、最大の Hz を使う。
- 期限は、前の時刻を保存せず、clock の原点（motion clock の 0）と tick の番号から求める。
  - Hz(r): next = ⌊now·r / 1e9⌋ + 1 番目の tick、すなわち (⌊now·r / 1e9⌋ + 1)·1e9 / r ns
  - Display: 毎 tick に次のフレームを要求する
- engine は tick ごとに、全 node の期限の最小値で frame を要求する。

#### 11.3.6 fallback

CPU の compositor と、pipeline の作成に失敗した GPU は、`ShaderItem::fallback`（`ShaderPaint::fallback` の paint で同じ shape を塗る scene）を描く。fallback が None なら何も描かない。filter shader の CPU の扱いは §8.8.5 のとおりである。

### 11.4 色（`kivact::style`）

```rust
pub fn oklch(l: impl Into<f64>, c: impl Into<f64>, h: impl Into<f64>) -> Color;   // L 0..1
pub fn oklab(l: impl Into<f64>, a: impl Into<f64>, b: impl Into<f64>) -> Color;
pub fn lab(l: impl Into<f64>, a: impl Into<f64>, b: impl Into<f64>) -> Color;     // L 0..100
pub fn lch(l: impl Into<f64>, c: impl Into<f64>, h: impl Into<f64>) -> Color;
pub fn hwb(h: impl Into<f64>, w: impl Into<f64>, b: impl Into<f64>) -> Color;     // w, b 0..1
impl Color {
    pub fn from_space(space: ColorSpace, c0: impl Into<f64>, c1: impl Into<f64>, c2: impl Into<f64>) -> Color;
    /// Parse a CSS color (§11.4.1). Errors are ErrorKind::Message naming the problem.
    #[track_caller]
    pub fn css(text: &str) -> Result<Color, UiError>;
}
impl Gradient {   // kivact::draw::Gradient
    pub fn interpolate(self, space: ColorSpace) -> Gradient;
    pub fn hue(self, hue: HueInterpolation) -> Gradient;
    /// A color hint between the previous and the next stop.
    pub fn hint(self, at: impl Into<Length>) -> Gradient;
}
impl CanvasGradient { pub fn interpolate(self, space: ColorSpace) -> Self; pub fn hue(self, hue: HueInterpolation) -> Self; }
```

#### 11.4.1 `Color::css` の文法

- **使える形:** 大文字と小文字は区別しない。
  - 名前の色: CSS Color 4 §6.1 の 148 色、`transparent`
  - 16 進: `#rgb`、`#rgba`、`#rrggbb`、`#rrggbbaa`
  - 関数: `rgb()`/`rgba()`、`hsl()`/`hsla()`（旧来のカンマ区切りと、新しい空白区切り・`/ alpha` の両方）、`hwb()`、`lab()`、`lch()`、`oklab()`、`oklch()`
  - `color(<space> c0 c1 c2 [/ alpha])`: space は `srgb`、`srgb-linear`、`display-p3`、`a98-rgb`、`prophoto-rgb`、`rec2020`、`xyz`、`xyz-d50`、`xyz-d65`
- **数値:** 数値、`%`、角度（`deg`、`rad`、`grad`、`turn`）を受ける。
- **使えないもの:** `none`、相対色の構文、`calc()`、`currentcolor` はエラーにする。
- **例:**
  - `hsl(120deg 50% 25% / 0.5)` は `Color::from_space(Hsl, 120, 0.5, 0.25).with_alpha(0.5)` と同じ。
  - `rgb(255 0 0 / 50%)` は `rgba(0xff0000, 0.5)` と同じ。

### 11.5 背景、mask、画像

```rust
impl Paint {   // kivact::draw::Paint (background layers)
    pub fn clip(self, clip: BackgroundClip) -> Paint;      // default BorderBox
    pub fn origin(self, origin: BackgroundOrigin) -> Paint; // default PaddingBox (masks: BorderBox)
    pub fn blend(self, mode: BlendMode) -> Paint;           // background-blend-mode
}
#[derive(Clone, Debug, PartialEq)]
pub struct MaskLayer { /* paint: Paint, mode, composite */ }
impl MaskLayer {
    pub fn new(paint: impl Into<Paint>) -> MaskLayer;
    pub fn mode(self, mode: MaskMode) -> MaskLayer;
    pub fn composite(self, op: MaskComposite) -> MaskLayer;
}
#[derive(Clone, Debug, PartialEq)]
pub struct Mask(pub(crate) Vec<MaskLayer>);   // first is on top
// From<Paint>, From<Gradient>, From<C: ThemeColor>, From<MaskLayer>, From<Vec<MaskLayer>>, tuples of 2..=4 Into<MaskLayer>.
impl From<Paint> for MaskLayer; impl From<Gradient> for MaskLayer;

impl Node {   // the image() builder
    /// Play an animated GIF, WebP or PNG (default true). Reduced motion shows the first frame.
    pub fn playing(self, playing: impl IntoValue<bool>) -> Self;
}
```

`playing` は、既存の `image_fit` と同じく reactive な property として engine へ書く。engine の property の表（`host/write/typed.rs`）に `ImagePlaying: set_image_playing(v: bool) => field(Some(v), |p| Some(&mut image(p)?.playing))` を加え、core の builder は値が変わるたびに `PropValue::ImagePlaying(v)` を書く。

`BackgroundClip`、`BackgroundOrigin`、`MaskMode`、`MaskComposite` は、`kivact_types` の型を `kivact::draw` からそのまま公開する。

### 11.6 prelude

prelude に次を加える（日常の利用に要る最小限）。

- `kivact::view::drawable`
- `kivact::motion::{animated, frame_loop, timeline, Transition, Easing, Spring}`
- `kivact::draw::Transform`

既存の prelude に motion の名前が無いため、motion の名前はここで初めて加える。prelude に入れる名前は、§16 の文書の例が `use kivact::prelude::*;` だけで書けるように選んだ。

---

## 12. アニメーション

すべての時刻は既存の runtime clock（engine の motion では engine の frame clock）である。新しい state はすべて enum で表す（§13）。

### 12.1 easing（`kivact-types/src/motion.rs`）

`Easing::apply(t)` は t を [0, 1] に clamp してから計算する。

| easing | 式 |
|---|---|
| Linear | t |
| Ease、EaseIn、EaseOut、EaseInOut、CubicBezier | cubic-bezier。既存の 30 回の二分法をそのまま使う |
| Steps { n, position } | 下記 |
| LinearPoints | 下記 |
| BackIn | c3·t³ − c1·t²（c1 = 1.70158、c3 = c1 + 1） |
| BackOut | 1 + c3·(t − 1)³ + c1·(t − 1)² |
| BackInOut | c2 = c1·1.525。t < 0.5 ? (2t)²·((c2 + 1)·2t − c2) / 2 : ((2t − 2)²·((c2 + 1)·(2t − 2) + c2) + 2) / 2 |
| ElasticIn | t = 0 → 0; t = 1 → 1; else −2^(10t − 10)·sin((10t − 10.75)·c4)（c4 = 2π/3） |
| ElasticOut | t = 0 → 0; t = 1 → 1; else 2^(−10t)·sin((10t − 0.75)·c4) + 1 |
| ElasticInOut | c5 = 2π/4.5。t = 0 → 0; t = 1 → 1; t < 0.5 ? −2^(20t − 10)·sin((20t − 11.125)·c5) / 2 : 2^(−20t + 10)·sin((20t − 11.125)·c5) / 2 + 1 |
| BounceOut | n1 = 7.5625、d1 = 2.75。t < 1/d1 → n1·t²; t < 2/d1 → n1·(t − 1.5/d1)² + 0.75; t < 2.5/d1 → n1·(t − 2.25/d1)² + 0.9375; else n1·(t − 2.625/d1)² + 0.984375 |
| BounceIn | 1 − BounceOut(1 − t) |
| BounceInOut | t < 0.5 ? (1 − BounceOut(1 − 2t)) / 2 : (1 + BounceOut(2t − 1)) / 2 |

**Steps（CSS Easing 1）:**

- step = ⌊t·n⌋ に、position が JumpStart または JumpBoth なら 1 を足す。
- jumps は position で決まる。
  - JumpNone: n − 1。n < 2 なら n = 2 として扱う。
  - JumpBoth: n + 1
  - JumpStart、JumpEnd: n
- 出力は clamp(step / jumps, 0, 1)。

**LinearPoints（CSS linear()）:**

1. 入力の位置の補完: 最初の点の入力が None なら 0、最後の点が None なら 1 とする。間の None は、前後の既知の入力の間に等間隔で割り振る。
2. 入力を非減少にそろえる（各入力 = max(入力, 前の入力)）。
3. t を含む区間で線形補間する。同じ入力が並ぶ区間は、後の点の出力を使う。

### 12.2 spring

**静止の判定（全 spring に共通）:** 閾値 `rest`（既定 1e-3）は、run の振幅に対する相対値である。位置と、速度を ω0 で割った値（位置と同じ単位になる）に、同じ閾値を使う。

- 成分 i の振幅を a_i = clamp(max(|to_i − from_i|, |v0_i| / ω0), 1e-9, 500) とする。v0 は run の開始の速度、ω0 = √(k/m) である。
  - 下限 1e-9: 振幅も速度も 0 の成分は、最初の step で静止する。0 → 0.0005 のような小さい振幅も、相対の判定なので 0 → 1 と同じ時間で静止する。
  - 上限 500: 振幅が 500 を超える run（長い scroll など）でも、最後に跳ぶ量が 0.5（px などの単位）を超えない。その代わり、振幅が 500 を超えると、静止までの時間が ln(a/500)/(ζω0) だけ伸びる（`with_duration` の保証の範囲は、振幅 500 以下である）。
- 成分ごとの大きさ E_i = √((x_i − to_i)² + (v_i / ω0)²) を使い、すべての成分で E_i ≤ rest·a_i を満たしたら静止とする。
- E は減衰振動の energy（½k·x² + ½m·v² = ½k·E²）の平方根なので、どの ζ でも時間とともに増えない（d/dt = −c·v² ≤ 0）。だから一度条件を満たせば、以後ずっと満たす。位置と速度が一瞬だけ小さくなる時刻で終わることはない。以後の |x − to| と |v|/ω0 も E 以下である。
- 速度を ω0 で割るので、duration を短くして ω0 が大きくなっても、判定が相対的に厳しくならない。

**step（解析解。`spring_step` を置き換える）:** 既存の半陰的 Euler 法（1 ms の substep）は、高い減衰（`with_duration(500, −0.99)` の ζ ≈ 100）で発散するので削除する。代わりに、各 step で、成分ごとに、前の step の位置と速度を初期値とする減衰振動の解析解で dt だけ進める。これは公開 API が受け付けるすべての係数と dt で安定である。

- 記号: m = max(mass, 1e-3)、k = stiffness、c = damping、x0 = x − to、v0 = v、ω0 = √(k/m)、ζ = c / (2√(k·m))。
- `Spring::problem` が Some の spring は、`spring_advance` が即座に完了として扱う（x = to、v = 0、true を返す）。`spring_advance` は報告しない純粋関数である。
- **報告の時点:** 問題を報告するのは、transition を受け取る側である。
  - kivact-style の `compile_style`: style の transition（`transition`、`layout_transition`、`enter`/`exit` の transition）に `Spring::problem` があれば、既存の style の報告と同じく `UiError(ErrorKind::Message, "spring: …")` を style の場所に一度報告する。
  - kivact-runtime: `animated(target, transition)` の作成と `Timeline` の区間の追加で、`ErrorKind::Message` を報告する（transition を後から変える API は無い）。
  - engine は報告しない（engine に届く transition は style が検査済みである）。
- ζ < 1 − 1e-6: ωd = ω0·√(1 − ζ²)、B = (v0 + ζω0·x0)/ωd、
  - x(t) = e^(−ζω0t)·(x0·cos ωd·t + B·sin ωd·t)
  - v(t) = e^(−ζω0t)·((v0)·cos ωd·t − (ζω0·B + ωd·x0)·sin ωd·t)
- |ζ − 1| ≤ 1e-6: x(t) = e^(−ω0t)·(x0 + (v0 + ω0·x0)·t)、v(t) = e^(−ω0t)·(v0 − ω0·(v0 + ω0·x0)·t)
- ζ > 1 + 1e-6: s = √(ζ² − 1)、r1 = −ω0 / (ζ + s)（桁落ちを避ける形）、r2 = −ω0·(ζ + s)、A = (v0 − r2·x0)/(r1 − r2)、B = x0 − A、
  - x(t) = A·e^(r1·t) + B·e^(r2·t)、v(t) = A·r1·e^(r1·t) + B·r2·e^(r2·t)
- 新しい位置は to + x(dt)、速度は v(dt) である。
- この関数は kivact-types の `motion.rs` に `fn spring_advance(s: Spring, x: &mut [f64], v: &mut [f64], to: &[f64], amplitude: &[f64], dt: f64) -> bool`（静止したら true）として置き、`Transition::step`、§12.6.1、§12.2 の τ の計算がすべてこれを使う。
- **振幅の計算（`SpringRun::start`）:** 振幅 a_i は run の開始で一度計算し、`SpringRun` に `pub amplitude: Vec<f64>` として保存する（純粋 cache）。既存の `SpringRun::restart(now_ns)` は、次に置き換える。

```rust
impl SpringRun {
    /// Start a run at `now_ns` from `from` toward `to`, keeping the velocity
    /// (an interrupted spring goes on with it), and compute the amplitude
    /// from `from`, `to`, the velocity and `transition`'s spring (empty when
    /// the transition is not a spring).
    pub fn start(&mut self, now_ns: u64, transition: &Transition, from: &[f64], to: &[f64]);
}
```

  呼び出し箇所はすべてこれに移す（既存の `restart` の利用: `kivact-runtime/src/motion/animated.rs`、`kivact-engine/src/model/state_motion/steps.rs`、`kivact-engine/src/model/layout_motion/steps.rs`）。`SpringRun { .. }` を直接作る箇所は、`SpringRun::default()` の後に `start` を呼ぶ形にする。反復の次の回（§12.3）は amplitude を変えない。

**`Spring::with_duration(duration_ms, bounce)`:** duration は「静止するまでの時間」の意味である（Framer Motion の duration と同じ。振幅 500 以下では振幅によらない）。bounce を [−0.99, 0.99] に収めてから、次の手順で値を求める。

1. ζ = bounce ≥ 0 ? 1 − bounce : 1 / (1 + bounce)
2. ω0 = 1 での正規化した静止時刻 τ(ζ) を、δ = 1e-3 として求める。τ は「それ以後ずっと、位置と速度/ω0 の両方が δ 以下」になる時刻である。
   - 0 → 1（x0 = −1、v0 = 0、ω0 = 1）の解析解（§12.2 の式）の x(t)、v(t) から E(t) = √(x(t)² + v(t)²) を作る。E は増えないので、E(t) ≤ δ となる最小の t を二分探索で求め、τ とする（上端は E ≤ δ になるまで t = 1 から倍にして探す。精度 1e-7）。ζ = 100 で τ ≈ 1381.5 程度になる。
3. ω0 = τ / (duration_ms / 1000)
4. stiffness = ω0²、damping = 2ζ·ω0、mass = 1、rest = 1e-3

**frame の間隔:** 静止の判定は各 frame の step の後で行う。E は増えないので、τ より前に静止することはなく、τ 以後の最初の frame で静止する。検出は [duration, duration + 1 frame] に入る（検算: duration 500 ms、bounce −0.99〜0.99 を 0.01 刻み、frame の間隔 1 ms・16 ms・100 ms で 500〜512 ms。振幅 500 以下、v0 = 0 のとき）。

### 12.3 keyframes

`Kind::Keyframes` の各 frame は、そこから始まる区間の easing を持つ。時刻の進み p（0..1）について、次の手順で値を求める。

1. p を含む区間 [o_i, o_{i+1}] を探す。
2. u = (p − o_i) / (o_{i+1} − o_i) とし、u' = easing_i.apply(u) とする。
3. 値は lerp(v_i, v_{i+1}, u') とする。

最後の frame の easing は使わない。

**反復の終わりの値（全経路で共通。`Transition::step` の変更）:** run が終わったときの値は、最後の反復の終わりの値とする。`PlayDirection::Alternate` で回数が偶数なら from、それ以外は to である（CSS の animation と Framer Motion の `repeatType: "reverse"` と同じ）。

- 既存の `Transition::step` と `sample_timed` は、終わりで to に snap していた。これを最後の反復の終わりの値に変える（`iter >= total` のとき、spring の最後の反復の完了のとき）。
- `Animated` の `finish(Completed)` は、to ではなく、この値を publish し、`current` に残す。target は `AnimPhase::Completed { target }` に残る（§12.4）。次の `set` は、この値から始まる。
- engine の state motion は §12.9.3 の `Held`、Timeline は §12.6 の区間の値、で同じ値を使う。
- **layout motion は反復しない:** layout motion（`layout_transition`、`layout_id`）は、node の box を goal へ運ぶ動きなので、transition の `repeat` と `direction` を使わない。engine は layout motion の run を `Repeat::Times(1)`、`PlayDirection::Normal` として扱い、完了で goal に着く（既存のとおり goal へ snap する）。kivact-style の `compile_style` は、`layout_transition` に `repeat` か `alternate` を指定した style に、`UiError(ErrorKind::Message, "layout_transition ignores repeat and alternate")` を一度報告する。`layout_transition` の rustdoc にも書く。

### 12.4 Animated の再構成と velocity（`kivact-runtime/src/motion/animated.rs`）

```rust
#[derive(Clone)]
pub(crate) enum AnimPhase {
    /// Not running; the target is `current`.
    Idle,
    /// The last run completed. `target` is the value last given to `set`;
    /// `current` may differ from it (alternate with an even count, §12.3).
    Completed { target: Vec<f64> },
    Running { start_ns: u64, run: Rc<Run> },
    Paused { start_ns: u64, at_ns: u64, run: Rc<Run> },
}
// PartialEq: same variant, equal numbers, Rc::ptr_eq on `run`.

/// One run: fixed endpoints and timing, and the spring's integration state.
pub(crate) struct Run {
    pub(crate) from: Vec<f64>,
    pub(crate) to: Vec<f64>,
    pub(crate) timing: Transition,
    pub(crate) spring: RefCell<SpringRun>,
}

pub(crate) struct AnimCell<T: 'static> {
    pub(crate) owner: Owner,
    pub(crate) value: Signal<T>,
    pub(crate) transition: Transition,
    /// The value now (always valid; published to `value`).
    pub(crate) current: RefCell<Vec<f64>>,
    pub(crate) phase: Signal<AnimPhase>,
}
```

既存の field `from`、`to`、`timing`、`spring` は `Run` に移す（S-3）。`retarget`、`begin`、`finish`、`step`、`shift_clock` は、phase の payload を読み書きする形に書き直す。

**`set_velocity` の遷移:**

| phase | `set_velocity(v)` の結果 |
|---|---|
| Running | 今の run の速度を v にする。run の repeat、direction、反復の回数はそのまま |
| Paused | 今の run の速度を v にする（resume の後にその速度から動く） |
| Idle、Completed | current から target へ、新しい run を始める。timing は cell の transition の spring を使い、`Repeat::Times(1)`、`PlayDirection::Normal`、delay 0 とする（速度を与えて target へ戻す一回の動きである） |

transition が spring でなければ、どの phase でも報告して無視する。

**target の持ち主:** cell の target は phase だけが持つ（別の field に写さない）。Idle では `current`、Completed では `target`、Running と Paused では `run.to` である。`get_target()` 相当の内部関数 `target(&self) -> Vec<f64>` は、この規則で phase から読む。

公開 API の追加は次のとおり。

```rust
impl<T: Animatable> Animated<T> {
    /// Give the spring a velocity (units per second, per component) now:
    /// a running spring keeps its target; an idle or completed one starts a
    /// run from its current value toward its target (for a completed
    /// alternate run, the value last given to `set`). Not a spring
    /// transition: reported (ErrorKind::Message) and ignored.
    pub fn set_velocity(&self, velocity: T);
}
```

### 12.5 frame loop と reduced motion

- `FrameLoop` に setup の builder `pub fn essential(self) -> Self` を加える（`kivact-runtime` と `kivact-core` の両方）。`LoopCell` に `motion: LoopMotion { Decorative, Essential }` を加える（既定は Decorative。構築の設定値）。
- `MotionState::on_frame_tick` と `wanted_frame` は、reduced motion の間、Decorative の loop を飛ばす（frame を要求しない）。
- reduced motion が true から false に変わったら（`MotionState::reduce_motion` の対になる `restore_motion(rt)` を `env_changed` から呼ぶ）、Running の Decorative な loop の run を `LoopRun { start_ns: now, index: 0, last_tick: None, ..run }` にする。これは surface の再開と同じ扱いである。

### 12.6 Timeline（`kivact-runtime/src/motion/timeline.rs`、新規）

```rust
#[derive(Clone, Copy)]
pub struct Timeline { cell: Owned<Rc<TimelineCell>> }
#[track_caller]
pub fn timeline() -> Timeline;                                   // setup only

/// Clone and Copy are implemented by hand (no `T: Copy` bound), as for Animated.
pub struct TimelineTrack<T: 'static> { timeline: Timeline, index: u32, value: Value<T> }
impl<T: Animatable> TimelineTrack<T> { pub fn get(&self) -> T; pub fn get_untracked(&self) -> T; }
impl<T: Animatable> IntoValue<T> for TimelineTrack<T>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum At {
    /// After the previously added segment ends, plus ms.
    AfterPrevious(f64),
    /// When the previously added segment starts, plus ms.
    WithPrevious(f64),
    /// At ms from the start.
    Time(f64),
    /// At a label, plus ms.
    Label(&'static str, f64),
}
#[derive(Clone, Copy)]
pub struct Segment { timeline: Timeline, index: u32 }
impl Segment { pub fn at(self, at: At) -> Segment; }             // setup only

impl Timeline {
    // building (setup only)
    pub fn track<T: Animatable>(&self, initial: T) -> TimelineTrack<T>;
    pub fn to<T: Animatable>(&self, track: TimelineTrack<T>, target: T, transition: Transition) -> Segment;
    pub fn label(&self, name: &'static str, at: At);
    pub fn call(&self, at: At, f: impl Into<Callback<()>>) -> Segment;
    pub fn repeat(self, repeat: Repeat) -> Timeline;
    pub fn alternate(self) -> Timeline;
    pub fn essential(self) -> Timeline;
    /// Drive time by a progress value 0..=1 instead of the clock (scroll-driven).
    pub fn drive(self, progress: impl IntoValue<f64>) -> Timeline;
    pub fn on_complete(self, f: impl Into<Callback<()>>) -> Timeline;
    // control (handlers and effects)
    pub fn play(&self);
    pub fn pause(&self);
    pub fn reverse(&self);
    pub fn restart(&self);
    pub fn finish(&self);
    pub fn seek(&self, ms: f64);
    pub fn seek_label(&self, name: &'static str);
    pub fn set_rate(&self, rate: f64);
    // reading
    pub fn time(&self) -> Value<f64>;
    pub fn progress(&self) -> Value<f64>;
    pub fn duration(&self) -> f64;
    pub fn is_playing(&self) -> Value<bool>;
}
```

**区間の開始時刻:** 区間を追加した時点で確定する（GSAP と同じ）。

| At | 開始時刻 |
|---|---|
| AfterPrevious(ms) | 直前に追加した区間の終わり + ms（区間がまだ無ければ 0 + ms） |
| WithPrevious(ms) | 直前の区間の開始 + ms |
| Time(ms) | ms |
| Label(name, ms) | label の時刻 + ms。label が無ければ報告して AfterPrevious(ms) として扱う |

- `.at()` は、その区間の開始時刻を変える。区間を追加した直後に呼ぶ（後に追加した区間の時刻は変わらない）。
- `label` と `call` の At も同じ規則で決まる。`call` は長さ 0 の区間として「直前の区間」になる。

**区間の長さと標本化:** 既存の `Transition::step` と同じ規則にする。

- 長さは delay_ms + 一回の長さ × 回数である。delay は最初の一回の前だけに入る。
  - 一回の長さ: Tween と Keyframes は ms、Spring は §12.6.1 の静止時刻
  - 回数: `Repeat::Times(n)` の n
- alternate の奇数回は、逆向きに標本化する。
- `Repeat::Forever` は区間には使えず、報告して Times(1) とする。

**timeline の長さ:** duration は、区間、label、call の時刻の最大値である。

**track の値:**

- 時刻 τ における track の値は、その track の区間を開始時刻の順（同じなら追加順）に見て決める。値は initial から始める。
- 開始 ≤ τ の各区間について、次の値を求める。
  - from: その区間の開始時刻での track の値（それより前の区間だけで決まる）
  - 値: from から target へ、区間の transition で τ − 開始 だけ進めた値（終わりを過ぎたら、§12.3 の反復の終わりの値）
- 最後に評価した区間の値を track の値とする。
- from は、setup の終わり（最初の評価の前）に一度だけ計算して区間に保存する（決定的な前計算。純粋 cache）。

**再生の状態:** 再生の設定と phase を分けて、一つの `TimelineCell` が持つ。

```rust
/// How playback proceeds; kept across every phase (not a mirror of it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Playback { pub(crate) rate: f64, pub(crate) direction: Direction }   // default rate 1, Forward
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Direction { Forward, Backward }
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TimelinePhase {
    /// Not playing, at `time` (also the state after setup, at 0).
    Idle { time: f64 },
    Playing { start_ns: u64, from_time: f64 },
    Paused { time: f64 },
    /// At an end after the last iteration.
    Finished { time: f64 },
    /// Time follows a progress value (drive).
    Driven,
}
// TimelineCell { playback: Cell<Playback>, phase: Signal<TimelinePhase>, … }
```

- **時刻:**
  - `Playing` の全体時刻は、t = from_time ± (now − start_ns) / 1e6 · rate（Backward は −）である。
  - 反復と alternate を含む全体の長さは、duration × 反復回数である。反復 k の中の時刻は t − k·duration とし、alternate の奇数回は duration − (t − k·duration) とする。

**操作と遷移:** 表の「Playing へ」は、Playing { start_ns: now, from_time: その時刻 } にすることを表す。

| 操作 | Idle / Paused | Playing | Finished | Driven |
|---|---|---|---|---|
| `play` | 現在の時刻から Playing へ | 変化なし | 始点（Forward は 0、Backward は全体の終わり）から Playing へ | 報告して無視 |
| `pause` | 変化なし | 現在の時刻で Paused へ | 変化なし | 報告して無視 |
| `reverse` | direction を反転 | direction を反転し、現在の時刻から Playing へ（取り直し） | direction を反転するだけ（次の play は新しい向きの始点から） | 報告して無視 |
| `seek(ms)` | 時刻を ms に | ms から Playing へ（取り直し） | Paused { time: ms } へ | 報告して無視 |
| `restart` | 始点から Playing へ | 始点から Playing へ | 始点から Playing へ | 報告して無視 |
| `finish` | 終点で Finished、`on_complete` を呼ぶ | 同左 | 変化なし | 報告して無視 |
| `set_rate(r)` | rate を r に | rate を r にし、現在の時刻から Playing へ（取り直し） | rate を r に | rate を r に（効果なし） |

その他の規則:

- **終点:** Forward は全体の終わり、Backward は 0 である。tick で終点に着いたら Finished { time: 終点 } にし、`on_complete` を呼ぶ。
- **rate:** `set_rate` の r は正の値でなければならない。0 以下は報告して無視する。
- **call:** Playing で時刻が call の時刻をまたいだときに一度呼ぶ（向きは問わない）。seek でまたいだときは呼ばない（GSAP の既定と同じ）。
- **drive:** time = clamp(progress, 0, 1)·duration とする。`on_complete` は呼ばない。
- **reduced motion:** essential でない timeline は、reduced motion の間に `play` または `restart` されると、すぐに終点の Finished になって `on_complete` を呼ぶ。Playing の間に reduced motion になったら、同じく終点の Finished にする。
- **owner の破棄:** frame の登録を外し、以後の操作を何もしない（既存の `Disposed` の扱い）。
- **frame:** Playing の timeline は `MotionState` の Anim として登録し、毎 tick で track の値を publish する。値が前と同じ track は publish しない。
- **`is_playing`:** phase の投影（memo）である。

#### 12.6.1 timeline の spring

timeline の中の spring は、速度 0 で 0 から 1 へ進む解析解 p(t) を使い、各成分を lerp(from, to, p) とする。

- ω0 = √(k/m)、ζ = c / (2√(k·m)) とする。
  - ζ < 1: ωd = ω0·√(1 − ζ²)、p = 1 − e^(−ζω0t)·(cos ωd·t + (ζω0/ωd)·sin ωd·t)
  - |ζ − 1| < 1e-6: p = 1 − e^(−ω0t)·(1 + ω0·t)
  - ζ > 1: r1,2 = −ω0·(ζ ∓ √(ζ² − 1))、p = 1 − (r2·e^(r1·t) − r1·e^(r2·t)) / (r2 − r1)
- 静止時刻は、§12.2 の `spring_advance` を 0 から 1 へ（振幅 1、v0 = 0）1 ms ずつ進め、静止した最初の t とする（上限 60 s。判定は §12.2 の位置と速度/ω0 の両方）。区間の中の p(t) は上の式で直接求める（`spring_advance` の値と一致する）。

### 12.7 Stagger（`kivact-runtime/src/motion/stagger.rs`、新規）

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Stagger { each_ms: f64, from: StaggerFrom, easing: Easing }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaggerFrom { Start, End, Center, Edges, Index(usize) }
impl Stagger {
    pub fn new(each_ms: impl Into<f64>) -> Stagger;      // from Start, easing Linear
    pub fn from(self, from: StaggerFrom) -> Stagger;
    pub fn easing(self, easing: Easing) -> Stagger;
    /// The delay in ms of item `index` of `count`.
    pub fn delay(&self, index: usize, count: usize) -> f64;
}
```

`delay(index, count)` は次のように求める。

1. 距離 d と最大距離 D を、from に応じて決める。
   - Start: o = 0
   - End: o = count − 1
   - Center: o = (count − 1) / 2
   - Index(i): o = i
   - 上の四つでは、d = |index − o|、D = 各 index の d の最大値とする。
   - Edges: d = min(index, count − 1 − index)、D = ⌊(count − 1) / 2⌋ とする。
2. D = 0 なら 0 を返す。
3. delay = easing.apply(d / D)·D·each_ms を返す。

`kivact::motion` から `Stagger`、`StaggerFrom` を公開する。リストの登場を時間差で始めるには、`for_index` の index を使い、`Transition::tween(..).delay(stagger.delay(i, n))` と書く（ガイドに例を載せる）。`kivact-ui/src/skeleton.rs` の手書きの遅延は `Stagger` に置き換える。

### 12.8 scroll-driven（`kivact-core/src/view/scroll_timeline.rs`、新規）

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewRange { Cover, Contain, Entry, Exit }
/// 0 at the start of `scroller`'s scroll range along `axis`, 1 at its end (0 if it cannot scroll).
#[track_caller]
pub fn scroll_progress(scroller: NodeRef, axis: Orientation) -> Value<f64>;
/// How far `target` has travelled through `scroller`'s viewport along `axis` (CSS view() timelines).
#[track_caller]
pub fn view_progress(target: NodeRef, scroller: NodeRef, axis: Orientation, range: ViewRange) -> Value<f64>;
```

**scroll_progress:** scroller の内部 `ScrollState { offset, viewport, content }` を使い、軸方向の offset / (content − viewport) を [0, 1] に収める。分母が 0 以下なら 0 とする。

**view_progress:** 自身の transform を掛けない window の矩形を使う（CSS の view timeline は transform を無視する）。この矩形は、`LayoutChanged::layout` から core の `NodeObservation` に足す `layout_rect: Signal<Option<Rect>>` で得る。

- 記号:
  - V = scroller の viewport の大きさ（軸方向）
  - T = target の大きさ
  - r = target の位置 − scroller の viewport の位置（どちらも layout_rect から。scroll すると減る）
- 式（結果は [0, 1] に収め、分母が 0 なら 0 とする）:

| range | progress |
|---|---|
| Cover | (V − r) / (V + T) |
| Contain | T ≤ V なら (V − r − T) / (V − T)、T > V なら −r / (T − V) |
| Entry | (V − r) / min(T, V) |
| Exit | (e − r) / (e + T)。e = T ≤ V ? 0 : V − T |

`scroll_progress`、`view_progress`、`ViewRange` は `kivact::view` から公開する（`NodeRef` と同じ module。core の `motion` は `view` を参照できないため）。`Timeline::drive` に渡すか、style の closure で直接使う。

### 12.9 per-property transition

#### 12.9.1 TransitionProp

```rust
// kivact-types/src/motion.rs (moved from kivact-style)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransitionProp {
    /// Every paint property without its own entry (CSS `transition: all`).
    All,
    Opacity, Transform, Background, BorderColor, BorderWidth, Radius,
    TextColor, Shadow, Filter, BackdropFilter,
    /// Presence size (enter_size / exit_size), as before.
    Size,
}
```

`Style::transition(prop, t)` は既存のまま、同じ prop の指定を置き換える。既存の `TransitionProp::Paint` は `All` に移行する。

#### 12.9.2 property ごとの補間の値（`PaintVec` を置き換える）

既存の固定長の `PaintVec` は削除する。代わりに、property ごとに補間の成分を取り出して書き戻す関数を、kivact-types の新しい top-level の module `prop_interp`（`src/prop_interp.rs`、leaf。`//! Role: Interpolate paint properties per transition property.`）に置く。`paint.rs` は leaf のまま保つ（子 module を持たせない）。`lib.rs` が `prop_interp`、`PropInterp`、`PropBasis`、`apply_components` を root から再公開する。`[module_rules.kivact-types.uses]` に `prop_interp = ["color", "color_space", "geometry", "motion", "paint"]`（使うものだけ）を加える。`paint` は `prop_interp` を参照しない。色の成分は、すべて Oklab の premultiplied（L·α, a·α, b·α, α）である。

```rust
/// One property's interpolation: the shape the numbers are written into
/// and the numbers at both ends. Fixed when a track starts.
#[derive(Clone, Debug, PartialEq)]
pub struct PropInterp { pub basis: PropBasis, pub from: Vec<f64>, pub to: Vec<f64> }

/// The non-numeric shape of an interpolated value.
#[derive(Clone, Debug, PartialEq)]
pub enum PropBasis {
    /// Numbers written into the property of the same name (Opacity, Transform,
    /// Background, BorderColor, BorderWidth, Radius, TextColor).
    Plain(TransitionProp),
    /// The padded shadow list (as long as the longer side; each entry's inset
    /// and its non-numeric fields fixed); numbers replace dx, dy, blur, spread, color.
    Shadows(Vec<Shadow>),
    /// The filter list being interpolated (function kinds and order from the
    /// non-empty side; Graph and Shader ops as they are); numbers replace each
    /// op's numeric arguments in order. `backdrop` picks the property.
    Filters { backdrop: bool, ops: Vec<FilterOp> },
}

impl PropBasis {
    /// Whether two bases interpolate the same components: the same variant
    /// and prop; for Shadows the same length and inset at each index; for
    /// Filters the same `backdrop`, length, function kind at each index and
    /// `==` Graph and Shader ops. The numbers in the basis are not compared.
    pub fn same_shape(&self, other: &PropBasis) -> bool;
}

/// The interpolation of `prop` between two paints, or None when they cannot
/// be interpolated (the property switches at once, as CSS discrete interpolation).
pub fn prop_interp(prop: TransitionProp, from: &PaintProps, to: &PaintProps, text: (Rgba, Rgba)) -> Option<PropInterp>;
/// Write `values` laid out in `basis` into `out` (a copy of the target paint):
/// the property is replaced by the basis filled with the values.
pub fn apply_components(basis: &PropBasis, values: &[f64], out: &mut PaintProps, text: &mut Rgba);
```

| prop | 成分 | 補間できない条件（None） |
|---|---|---|
| Opacity | opacity | — |
| Background | 単色の背景の色 4 | どちらかが単色でない |
| BorderColor | 4 辺の色 16 | — |
| BorderWidth | 4 辺の幅 4 | — |
| Radius | 4 隅 × (x.px, x.pct, y.px, y.pct) 16 | — |
| Transform | ElementTransform の 17（translate 3、per_size 4、rotate 3、skew 3、scale 3、perspective_inverse 1）と origin の 5 | — |
| Shadow | shadow ごとに (dx, dy, blur, spread, 色 4) | 二つの列の同じ位置で inset が違う。短い列は、相手の inset を持つ透明・0 の shadow で補う（CSS） |
| Filter、BackdropFilter | 位置ごとの数値の引数（Blur は 1、DropShadow は dx, dy, blur, 色 4、他は量 1） | 二つの列の長さが違い、どちらも空でない。または同じ位置の関数の種類が違う。または同じ位置の Graph・Shader が `==` でない。片方が空なら、相手の各関数を下の表の初期値にした列とみなす |
| TextColor | text の色 4 | — |

**filter の関数の初期値（CSS Filter Effects 1 の補間の初期値）:**

| 関数 | 初期値 |
|---|---|
| Blur | 0 |
| Brightness、Contrast、Saturate、Opacity | 1 |
| Grayscale、Sepia、Invert | 0 |
| HueRotate | 0（度） |
| DropShadow | offset (0, 0)、blur 0、color 透明の黒 |
| Graph、Shader | 初期値が無い。片方が空で相手に Graph か Shader を含む列は None（即座に切り替わる） |

**basis の作り方:**

- Filter: 二つの列が同じ長さなら、from の列を basis にする（種類は一致している）。片方が空なら、空でない側の列を basis にし、空の側の数値は上の表の初期値にする。
- Shadow: 長い側の長さにそろえ、短い側の不足分は、相手の同じ位置の shadow の inset と非数値の field を持つ、色が透明・数値が 0 の shadow で補う。basis は、各位置で、from に実在する shadow があればそれ、無ければ to の shadow を使う。
- `apply_components` は、basis の列に数値を書いたもので property を置き換える。track が終わったら、paint は target の値（filter が none なら空の列）そのものになる（§12.9.3）。
- `All` は、個別の指定が無い prop のすべてに同じ transition を使う。

#### 12.9.3 engine の state motion（`model/state_motion/state.rs`）

```rust
pub(crate) struct StateAnimation {
    pub(super) target: PaintProps,
    pub(super) target_text: Rgba,
    pub(super) tracks: Vec<PropTrack>,
    pub(super) colors: ColorChannels,
}
pub(crate) struct PropTrack {
    pub(super) prop: TransitionProp,
    pub(super) transition: std::sync::Arc<Transition>,
    pub(super) phase: TrackPhase,
}
pub(crate) enum TrackPhase {
    /// A state change is pending: `shown` is drawn; `from`/`to` are the
    /// interrupted run's endpoints (for `Retarget::Proportional`), equal to
    /// `shown` when nothing ran; `spring` keeps the interrupted velocity.
    Captured { interp: PropInterp, shown: Vec<f64>, spring: SpringRun },
    Running { interp: PropInterp, current: Vec<f64>, start: u64, spring: SpringRun },
    /// The run ended away from the target (alternate with an even count, §12.3):
    /// `value` is drawn until the next capture.
    Held { interp: PropInterp, value: Vec<f64> },
    /// At the target value.
    Settled,
}
```

- **track の作成:** node の `transitions` から、prop ごとに一つずつ作る。prop の transition は、個別の指定、`All` の指定、指定なしの順に探し、最初に見つかったものを使う。
  - 指定なしの prop、および `prop_interp` が None の prop は track を作らない（即座に target の値になる）。
- **capture（`capture_state_motion`）:** track ごとに次のように Captured にする。表示中の値をそのまま `shown` に移すので、途中の値と速度は失われない。
  - Running { interp, current, spring } → Captured { interp, shown: current, spring }
  - Held { interp, value } → Captured { interp, shown: value, spring: 既定 }
  - Settled、または track が無い → 表示中の paint（target）から `prop_interp(prop, 表示中, 表示中)` を作り、Captured { interp, shown: interp.to, spring: 既定 }（`prop_interp` が None なら track を作らない）
- **retarget（`retarget_states`）:** Captured { interp: old, shown, spring } について、新しい target の paint で次を行う。
  1. 表示中の paint を、古い target の写しに `apply_components(old.basis, shown)` を書いたものとする。
  2. new = `prop_interp(prop, 表示中の paint, 新しい target, …)` を作る。new.from は、表示中の値を新しい basis で表したものになる（shown を流用しない。basis の成分の数が変わっても整合する）。None なら、track を Settled にする（即座に切り替わる）。
  3. new.to が new.from と等しければ Settled にする。
  4. そうでなければ、basis が変わったかで次のように決める。
     - `old.basis.same_shape(&new.basis)` のとき: 成分が一対一に対応するので、timing = `transition.retargeted(old.from, old.to, new.from, new.to)` とし、速度を保つ。
     - shape が違うとき（shadow の数や inset、filter の関数の列が変わった）: 成分が対応しないので、新しい run として始める。timing は `transition` のまま（比例の retarget をしない）、速度は 0 にする。
  5. Running { interp: new, current: new.from, start: now, spring } にし、`spring.start(now, &timing, &new.from, &new.to)` を呼ぶ。
- **step:** Running の track を `Transition::step` で進める。終わったら、終わりの値（§12.3）が interp.to なら Settled、そうでなければ Held { value } にする。
- 全 track が Settled になったら、`StateAnimation` を map から消す。
- **現在の paint:** target の写しに、Settled でない各 track の値（Captured は shown、Running は current、Held は value）を `apply_components(interp.basis, …)` で書いたものとする。
- **layer の判定:** `StateMotion::moves_layer(id) -> bool` は、Opacity または Transform の track が Running のとき true を返す（§6.1 の R4）。Captured と Held は false である。

### 12.10 presence mode（`kivact-core/src/view/flow/branch.rs`）

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PresenceMode {
    /// The old content leaves while the new one enters (as today).
    #[default] Sync,
    /// The new content mounts after the old content's exit completes.
    Wait,
    /// The old content leaves out of layout (ExitLayout::Pop) while the new one enters.
    PopLayout,
}
impl Show { pub fn presence(self, mode: PresenceMode) -> Show; }
impl<T> ShowSome<T> { pub fn presence(self, mode: PresenceMode) -> ShowSome<T>; }
/// `dynamic` returns a builder (was View).
pub struct Dynamic<T> { /* … */ }
impl<T> Dynamic<T> { pub fn presence(self, mode: PresenceMode) -> Dynamic<T>; }
impl<T: PropValue> IntoView for Dynamic<T>;
```

**状態（Show の例。ShowSome と Dynamic も同じ形）:**

```rust
enum ShowArm {
    Pending,
    Child(Branch),
    Fallback(Option<Branch>),
    /// Wait mode: the old branch is leaving; `next` mounts when it has gone.
    Leaving { next: Next },
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Next { Child, Fallback }
```

- **Sync:** 既存のとおり。
- **PopLayout:** 古い branch の fragment を `remove_with_exit(id, ExitLayout::Pop, notify)` で外す。
- **Wait の手順:**
  1. 腕の切り替えで、古い branch を exit 付きで外す（`unmount_branch_with_exit(rt, b, on_exit)`）。外した branch の owner は、既存どおり直ちに dispose する。
  2. `Leaving { next }` にする。
  3. `on_exit` は core の `Leaving.on_exit`（既存の ExitComplete の経路）で呼ばれる。呼ばれたら、そのとき望まれている腕を mount する。
  4. Leaving の間に条件が変わった場合は、next だけを更新する。
- **exit の無い branch:** exit animation が無くても、ExitComplete は次の sync で届く（既存の性質）。

### 12.11 shared layout（`layout_id`）

```rust
impl<B: NodeBuilder> B {   // node_modifiers
    /// Animate from the last box of the node that had the same id (Framer Motion `layoutId`).
    /// Uses this node's `layout_transition`; without one nothing animates.
    pub fn layout_id(self, id: impl Into<String>) -> Self;
}
```

**engine への受け渡し:** `layout_id` は style ではなく node の属性である（Framer Motion の `layoutId` prop と同じ）。

- core: node の builder の共通の field `common.layout_id: Option<Arc<str>>` に入れ、mount のときに `PropValue::LayoutId(Some(id))` を一度書く（reactive ではない）。
- engine: `NodeProperties` に `pub layout_id: Option<std::sync::Arc<str>>` を加え、property の表（`host/write/typed.rs`）に `LayoutId: set_layout_id(v: Option<Arc<str>>) => field(v, |p| Some(&mut p.layout_id))` を加える。
- `LayoutProps` には加えない。

engine は `model/layout_motion/shared.rs`（新規）に次の型を置く。

```rust
/// Window layout boxes of removed nodes with a layout id, kept until the
/// next layout pass has had the chance to use them (a per-pass mailbox).
#[derive(Default)]
pub(crate) struct SharedLayouts { pub(super) departed: HashMap<Arc<str>, Rect> }
```

- **記録:** layout_id を持つ node が外されるとき、その node の window の layout 矩形（自身の transform を掛けない）を記録する。記録するかどうかは、既存の presence の状態だけで決める（新しい flag を足さない）。
  - `begin_exit`: node が Leaving に入る前（Mounted か Arriving）のときだけ記録する。すでに Leaving なら記録しない。
  - `remove_now`: node が Leaving なら記録しない（`begin_exit` で記録済み、または exit の間に記録が消費・破棄された）。Mounted か Arriving（exit を通らずに直接外される）のときだけ記録する。
  - 同じ id がすでにあれば上書きする（後に外された node が勝つ）。
- **消費:** `Model::layout` の中の `retarget_layout_motion` で、layout_id を持ち、この layout で初めて矩形を得た node に、同じ id の記録があれば取り出す。その矩形を from として、node の layout motion を始める。
- **破棄:** `Model::layout` の終わりで `departed` を空にする。remove の後の最初の layout pass が、消費の唯一の機会である（commit の末尾では消さない。layout は commit の後の settle で行われるため）。
- **id の重複:** 同じ id の node が同時に二つ生きている場合、後から作られた node が記録を使う。`UiError(ErrorKind::Message, "duplicate layout_id …")` を一度報告する。

### 12.12 layout motion の window 空間化と角の補正

`LayoutAnimation` の from、to、current を、node の window の layout 矩形（x, y, w, h。自身の transform を掛けない）にする。既存の親相対の矩形は使わない。

**window 空間の写像:** 矩形 a を矩形 b へ写す window 空間の affine を、次のように定める（`then` は既存の「self の後に next」）。

```text
box_map(a, b) = T(−a.x, −a.y).then(S(b.w / a.w, b.h / a.h)).then(T(b.x, b.y))
```

**node の局所の motion transform:**

- 記号:
  - g を、node の新しい window の layout 矩形（goal）とする。
  - A_c = box_map(g, current) とする。
  - V を、layout motion が動いている最も近い祖先の A_c（= box_map(goal_anc, current_anc)。同じフレームの current を使う）とする。そのような祖先が無ければ単位行列とする。
  - 理由: 各 node の補正は、その node に実際に掛かる window 空間の累積を A_c にそろえる。だから最も近い動いている祖先の累積は、その祖先の A_c そのものであり、それより外の祖先を掛け直してはいけない（掛け直すと補正が重複する）。動いていない祖先は、累積に何も足さない。
- 式:
  - O = A_c.then(V.invert())（祖先が既に掛ける V を打ち消す）
  - L = T(g.x, g.y).then(O).then(T(−g.x, −g.y))（window 空間から node の box の局所座標へ戻す）
- L を、既存の `layout_motion_transform` の値（node の box の左上を原点とする局所の transform）にする。

**検算:** 祖先なしで g = (x 100, w 100)、current = (x 0, w 200) とする。

- L(p) = −100 + 2p である。
- node の box の原点は window の 100 なので、局所の点 p は window の 100 + L(p) = 2p に描かれる。
- p = 0 は window の 0 になり、current の左端と一致する。

**検算（3 階層）:** root、parent、child の A_c がすべて T(10, 0) とする。

- root: V = I、O = T10。実際の累積は T10。
- parent: V = root の A_c = T10、O = T10·T−10 = I。実際の累積は T10。
- child: V = parent の A_c = T10、O = I。実際の累積は T10（期待どおり）。祖先の A_c の積 T20 を V にすると、child の累積は T0 になって誤る。

**角の補正:** layout motion の scale（sx = cur.w / goal.w、sy = cur.h / goal.h）が 1 でない間、node の effective paint の角の丸めの px 値を、x 方向は sx で、y 方向は sy で割る（`effective_paint` で行う）。拡大縮小しても、角が元の丸めに見えるようにするためである。

### 12.13 Animated<Color>

§9.10 のとおり、Oklab の premultiplied で補間し、theme token は現在の owner の theme で解決する。`docs/for-kivact-user/*/how-to-guide/animation.md` の「色は animated() で補間できない」という記述を、補間できる型の一覧（`f32`、`f64`、tuple、`Point`、`Size`、`Rect`、`Color`、`Transform`）に直す。

---

## 13. 状態機械

### 13.1 既存の暗黙の状態機械の解消

調査（`main` @ f4e40cc の state-fields の marker 全数と、lint が見ない形の grep）で見つけたものを、すべて次のように直す。marker が既に付いていて、理由が正しいもの（独立した構築 option、純粋 cache、Sent、Mailbox）は変えない。

| ID | 場所 | 問題 | 直し方 |
|---|---|---|---|
| SM-01 | `kivact-engine/src/model/presence/state.rs` の `Presences.arriving`/`leaving` と `tree/node.rs` の `HostNode.presence` | 同じ事実の二重保存（S-4）。`transition_presence` が二つを同期している | `HostNode.presence` を削除する。`NodeTree` に `presence: BTreeMap<NodeId, NodePresence>`（無い node は Mounted）を置いて唯一の持ち主にする。`drawn`/`shown`/`presence_of(id)` はこの map を引く。arriving と leaving の集合は、この map を variant で走査して得る（集合を別に持たない） |
| SM-02 | `tree/node.rs` の `Place.transformed` | 書き手が二つ（`update_place` と state motion）（S-5） | §7.6 のとおり、`update_place` だけが書く |
| SM-03 | `paint/list/walk.rs` の `closes()` と `paint/frame_scene.rs` の `paint_close` | Open で何を push したかを、Close 側が別の論理で推し量っている（S-4、S-6） | `Step` に `fn opened(&self) -> Vec<Opened>`（§6.3 の `Opened`）を一つだけ置く。Open はその順に push し、Close はその逆順に pop する。`closes()` は `!step.opened().is_empty()`、または step が draw_over か scrollbar を持つ、で判定する。`CompositionBuilder` の `Opened` stack（§6.3）は、この列から作る |
| SM-04 | `paint/state.rs` の `PaintState.scale`/`raster` と `paint/snapshot.rs` の list の差し替え | walk の間だけ state を書き換えて戻す（S-5、S-7） | §6.9 の `PaintPass` を引数で渡す。`raster: Cell<f64>` を削除する。snapshot は別の `PaintList` を作って渡す |
| SM-05 | `PaintState.dropped_reported: RefCell<BTreeSet<(NodeId, u8)>>`、`lower/assets.rs` の `reported_images: HashMap<NodeId, Asset>`、`reported_families: HashSet<String>` | 手作りの report-once（S-9） | `kivact_types::cell::ReportOnceSet` を使う。`reported_images` は `ReportOnceSet<(NodeId, Asset)>` |
| SM-06 | `measure/editor.rs` の `EditorNode.reported_session: Option<TextSession>` | 最後に送った値を `Sent` で表していない（S-9） | `Sent<TextSession>` にする |
| SM-07 | `kivact-runtime/src/motion/animated.rs` の `AnimCell` の `from`/`to`/`timing`/`spring` | 実行中だけ有効な値が phase の外にある（S-3） | §12.4 の `Run` を phase の payload にする |
| SM-08 | `model/state_motion/state.rs` の `StateAnimation` の `spring`/`current` と `StateAnimPhase` | spring の状態が phase の外にある（S-3） | §12.9.3 の `TrackPhase::Running` に spring を入れる |
| SM-09 | `paint/state.rs` の `PaintList.backdrop_reach`（単調に増え、Stale にする） | backdrop の replay のための補修値（S-5） | 削除する。backdrop は compositor が実画素で読む（§8.9） |
| SM-10 | `arvane-render/src/gpu/state.rs` の `GpuSurface.raster: Option<Texture>` と `keep: Retain` | 中間 texture の有無が別の field の値で決まる | compose へ移すときに削除する。swapchain へは常に target 0 から合成するので、中間 texture は要らない。`keep` は `Retain` のまま残す |
| SM-11 | `arvane-render/src/draw/painter.rs` の `Frame.backdrops: bool` と `backdrop::Index` | backdrop の replay の有無の flag | 削除する（SM-09 と同じ理由） |
| SM-12 | `kivact-engine/src/lower/context.rs` の `Purpose` | 同じ scene の二つの意味（Window/Cpu）を flag で切り替える | 削除する（§6.8）。CPU の代わりの pixels は常に `ExternalItem::cpu` に入れる |

### 13.2 新しい状態の一覧

新しいコードの状態は、次の型だけで表す。各行は、持ち主、書き換える event、有効な payload、破棄の時点を決めている。表に無い bool、Option、counter で状態を補わない。passive な record や純粋 cache の field には、`xtask-allow(state-fields)` と正確な理由を書く。

| 状態 | 定義 | 持ち主と書き手（event） | 破棄・作り直し |
|---|---|---|---|
| `AnimPhase`（payload の `Run`、Completed の `target`。target の唯一の持ち主） | §12.4 | `AnimCell`。retarget、step、pause、resume、jump_to、set_velocity、reduced motion | owner の dispose（既存の owner の破棄の処理で cell ごと消える。phase の variant ではない） |
| `Playback` と `TimelinePhase` | §12.6 | `TimelineCell`。§12.6 の操作の表と tick と reduced motion（`Playback` は phase をまたぐ設定であり、phase の写しではない） | owner の dispose（cell ごと消える） |
| `TrackPhase`（Captured、Running、Held、Settled。payload に `PropInterp` と表示中の値と `SpringRun`） | §12.9.3 | state motion。capture、retarget、step | 全 track が Settled で `StateAnimation` ごと消す |
| `LoopMotion` | §12.5 | 構築時の設定（`essential()`） | owner の dispose |
| `PlaybackState` | §9.8 | engine。paint list の再構築と tick | 再生しない条件で entry を消す |
| `Placement`、`DrawnPlacement` | §10.3 | engine。`set_scene`（Canvas。両 slot から作り直す）と `SetDrawableGeometry`（Export。generation を照合） | drawable root の remove で entry ごと消す |
| `ExportLink`、`ExportRequest`（generation 付き。GPU は Requested、Waiting、Provided） | §10.5 | core。`DrawableShare` の method と、generation を照合した host event。GPU は §10.5 の遷移表 | dispose で Closed |
| `DrawableBinding`（Unbound、Bound） | §10.2 | core。`Node::drawable`/`Viewport::drawable` の mount（検査の後）、root の unmount（root が一致するときだけ） | drawable の owner の dispose |
| `ExportRetry`（Idle、Pending） | §10.5 | engine。提示ごとに `pending_exports` から確定する。Pending の retry_at_ns は pump の deadline（host の ns の時計） | `pending_exports` に無い提示、mode が `Gpu` でなくなる、drawable の破棄で Idle |
| `ExportRegistry`（pool の記録と `ExportNotice` の queue） | §8.12 | arvane の presenter。`export_drawable`、`drop_export`、`GpuContext` を捨てる前、新しい `GpuContext` の準備の完了 | presenter と同じ寿命。notices は `take_export_notices` で空になる |
| `ExportRecord` | §8.12 | compositor。export に成功したとき（純粋 cache） | sink の drop_export、device の喪失 |
| `ShowArm::Leaving`、`Next` | §12.10 | Show、ShowSome、Dynamic の effect と exit の callback | branch の owner の dispose |
| `SharedLayouts` | §12.11 | engine。`begin_exit`（Leaving の前）と `remove_now`（Leaving でない）で記録、layout で消費 | `Model::layout` の終わり（layout pass ごとの Mailbox） |
| `CacheIndex`、`ChunkRecord`（`retain` 付き）、`LayerRecord` | §8.2、§8.6 | compositor。成功したフレームの後（純粋 cache） | 2 フレーム使われない、または予算の超過 |
| `SubtreeFacts`、`ShaderClocks` | §6.1、§11.3.5 | engine。paint list の再構築（純粋 cache） | 次の再構築で作り直す |

compositor の `Plan` と `FrameStats` は 1 フレームの値であり、状態ではない。

---

## 14. arvane-render の新しい API

```rust
//! Role: Arvane render: rasterises RenderScene into CPU pixmaps and GPU textures.

pub use error::RenderError;   // + variant InvalidComposition(String) (used by arvane-compose)
pub use cpu::CpuRaster;
#[cfg(feature = "gpu")] pub use gpu::{GpuRaster, RASTER_FORMAT};
pub use ink::glyph_ink_bounds;
pub use outline::glyph_outline;
pub use validate::{dimensions, raster_plan};
pub use limits::MAX_SCENE_DEPTH;

pub struct CpuRaster { /* vello_cpu RenderContext, Resources, Caches, ImageCache, Derived */ }
impl CpuRaster {
    pub fn new() -> CpuRaster;
    pub fn release_caches(&mut self);
    /// Rasterize `scene` mapped by scale(scale) · translate(−origin) into
    /// `out` (size.0 × size.1 premultiplied RGBA8, row-major, no padding).
    /// With `clip`, only that rect of `out` is drawn and the rest is left as
    /// it is; without it `out` is cleared to transparent first.
    pub fn raster(&mut self, scene: &RenderScene, origin: Point, scale: f64, size: (u16, u16), clip: Option<PixelRect>, out: &mut [u8]) -> Result<(), RenderError>;
}

#[cfg(feature = "gpu")]
pub const RASTER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
#[cfg(feature = "gpu")]
pub struct GpuRaster { /* vello_hybrid Renderer, Resources, Scene, Caches, Images */ }
#[cfg(feature = "gpu")]
impl GpuRaster {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<GpuRaster, RenderError>;
    pub fn release_caches(&mut self);
    /// Rasterize into `view` (RASTER_FORMAT, RENDER_ATTACHMENT, `size`), cleared to transparent.
    pub fn raster(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder,
        scene: &RenderScene, origin: Point, scale: f64, size: (u16, u16), view: &wgpu::TextureView) -> Result<(), RenderError>;
}
```

- `PixelRect` は §5.11 のとおり `kivact_types::render` に置く。
- **CPU の `clip`:** 引数 `clip` の範囲を小さな pixmap に描き、`out` の該当部分に書き写す。scene は translate(−clip.origin / scale) を前置して描く。
- **GPU の部分 redraw:** compose が一時 texture に `raster` してから copy する（§8.13）。
- **削除する処理:** `draw/effects.rs` の mask、backdrop、layer filter、drop shadow の emulation を削除する。box shadow と blurred rounded rect の描画は `draw/shadows.rs` に残す。
- **`PushGroup` の描画:** `canvas.push_layer(clip, blend, opacity, blur)` で描く。blur は vello_common の `FilterPrimitive::GaussianBlur` で表す（既存の text shadow の描き方のまま）。
- **外部 texture:** `Canvas::external_size` と `draw_external` を削除する。

---

## 15. shell、native、virtual、platform

### 15.1 CPU 提示の damage（`arvane-shell/src/window.rs`）

```rust
/// Whether an offered buffer still holds the frame presented last time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferAge { Previous, Unknown }

fn present_with(
    &mut self,
    width: u32,
    height: u32,
    damage: Option<&[kivact_types::render::PixelRect]>,
    draw: &mut dyn FnMut(&mut [u32], BufferAge) -> bool,
) -> Result<bool, ShellError>;
```

- **native（`arvane-native/src/present.rs`）:**
  - softbuffer の `buffer.age()` が 1 なら `Previous`、それ以外は `Unknown` を `draw` に渡す。
  - damage が Some なら `buffer.present_with_damage(rects)` を、None なら `present()` を呼ぶ。
  - softbuffer の `Rect` へは、そのまま写す。
- **virtual:** 前フレームの pixels を保持する。damage の内側だけを書き換え、`Previous` を渡す。
- **arvane の presenter:**
  - `Previous` なら damage の rect の範囲だけを、`CpuOutput::rgba` から 0x00RRGGBB に変換して書く（白の上の合成）。
  - `Unknown` なら全体を書く。damage は `CpuOutput::damage` をそのまま渡す（Full なら None）。

### 15.2 Linux の reduced motion（`arvane-native/src/port/linux_settings.rs`、新規）

`port.rs` に `#[cfg(target_os = "linux")] mod linux_settings;` を加える。`zbus` は `arvane-native` の `[target.'cfg(target_os = "linux")'.dependencies]` に `zbus = { workspace = true }` として加える。

- 起動時に thread `arvane-settings` を作り、zbus の blocking API で session bus の `org.freedesktop.portal.Desktop` に接続する。
- `org.freedesktop.portal.Settings` の `ReadOne`（無ければ `Read`）で、次の値を読む。
  - `org.gnome.desktop.interface` の `enable-animations`（bool）
  - `org.kde.kdeglobals.KDE` の `AnimationDurationFactor`（double）
- reduced motion の判定: enable-animations が false、または AnimationDurationFactor が 0 なら true とする。どちらも読めなければ false とする。
- 値の変化を知る方法: `SettingChanged` signal を購読する。値が変わったら共有の `AtomicBool` を更新し、event loop を起こして `ShellEvent::EnvironmentChanged` を出す（既存の環境変化の経路）。
- `native_input/system.rs` の `reduced_motion()` は、Linux ではこの `AtomicBool` を読む。
- portal に接続できない環境（session bus が無い等）では、thread を終えて false のままにする（`tracing::debug!` を一度出す）。

### 15.3 X11 の refresh rate（`arvane-native/src/port/x11/window.rs`）

- `frame_period_ns` は、RandR（x11rb の `randr` feature）で window の中心を含む CRTC の mode から求める。
  - 求め方: `get_screen_resources_current`、`get_crtc_info`、mode の dot_clock / (htotal·vtotal) で refresh rate を出し、1e9 / refresh rate を周期とする。
  - 値は `ConfigureNotify` のたびに取り直し、変化したら `EnvironmentChanged` を出す。
- RandR が使えないか値が 0 なら、既存の 16,666,667 ns とする。

---

## 16. 文書、kit、catalog

### 16.1 利用者向けガイド（英日の対で、同じ変更で書く）

| ファイル | 内容 |
|---|---|
| `docs/for-kivact-user/{en,jp}/how-to-guide/3d-and-depth.md`（新規） | 3D transform、perspective、preserve-3d、backface。`pointer_position()` と `animated` の spring によるパララックスの例。3D のカードの例 |
| `…/how-to-guide/effects.md`（新規） | filter（関数と filter graph）、mask、blend と isolation、backdrop、border image と nine-slice、background の clip と origin、gradient text |
| `…/how-to-guide/shaders.md`（新規） | WGSL の契約（§11.3.2）、param、image、animate、fallback、CPU 描画での見え方、GPU lane での試験 |
| `…/how-to-guide/drawables.md`（新規） | HTML-in-Canvas 相当の drawable、hit test と a11y の自動の対応、export（pixels と共有 texture）、アプリの 3D エンジンでの使い方 |
| `…/how-to-guide/timelines-and-scroll.md`（新規） | Timeline、Stagger、scroll_progress、view_progress、presence mode、layout_id |
| `…/how-to-guide/animation.md` | 新しい easing、`Spring::with_duration`、keyframe の easing、`set_velocity`、per-property transition、frame loop の reduced motion、補間できる型の一覧（§12.13） |
| `…/how-to-guide/canvas.md` | 3D の layer、composite、text の fill・stroke・path、border image、drawable への案内 |
| `…/how-to-guide/images-and-fonts.md` | アニメーション画像と `playing`、SVG の text と埋め込み画像、font variation、font width、optical sizing |
| `…/how-to-guide/theming.md` | 色空間の関数、`Color::css`、gradient の補間と hint |
| `…/how-to-guide/coming-from-react-or-solid.md` | 対応表に追加: CSS の新しい property と setter、Framer Motion（`layoutId`、`AnimatePresence mode`、`useScroll`）と GSAP（timeline、stagger）との対応 |
| `…/how-to-guide/testing.md` | shader の GPU lane（`RenderOptions::gpu()` と `KIVACT_TEST_GPU=1`）、CPU 描画での fallback |
| `…/how-to-guide/README.md` | 新しいページへの索引 |

文書の Rust の code block は `dev/tests/docs` で compile される。window を開く例は `no_run` とする（既存の規則）。shader のページの WGSL は ```` ```wgsl ```` の block とし、Rust の例では `include_str!` ではなく文字列リテラルで渡す（doctest で file を持たないため）。

### 16.2 kit と catalog

- **五つの kit:** API の変更に追従する（§17.3 の移行表）。kit の公開 API は縮めない（kit に実利用者がいるため）。
- **Game kit の catalog（`examples/kivact-ui-game/src/game/effects.rs`、新規）:** "Effects" のページを足し、次の見本を並べる。
  1. パララックスの HUD: `pointer_position()` と spring
  2. 3D の tilt card: perspective、rotate_x/y、backface
  3. drawable を回転させた 3D の設定パネル
  4. filter graph の glitch: turbulence と displacement
  5. shader の scanline の背景（fallback 付き）
  6. timeline で順に立ち上がる boot sequence と stagger
  7. scroll に連動する header
  - ページは既存の catalog の登録の仕組みに追加する。`examples/kivact-ui-game/tests/catalog.rs` に、このページを mount して主要な role（button、heading）が見つかることの試験を 1 つ足す。
- **game kit の文書:** `docs/for-kivact-ui-user/{en,jp}/game/how-to-guide/effects-and-depth.md`（新規）に、上の見本の作り方と、kit の tokens（色、角）との組み合わせ方を書く。
- **他の kit:**
  - `kivact-ui/src/skeleton.rs` の手書きの遅延は `Stagger` にする（§12.7）。

### 16.3 開発者向け文書

| ファイル | 変更 |
|---|---|
| `docs/for-kivact-developer/strategy.md` | §2 の対象外の行の「ゲーム内 UI」を「ゲームエンジン（Unity・Unreal Engine 等）に組み込むゲーム内 UI」に直し、ゲーム風のアプリ UI は対象であると 1 行足す。§5 の「2D 描画」の行を「描画（CSS の 2D と 3D transform、filter、mask、WGSL の shader（GPU のみ）、HTML-in-Canvas に相当する drawable）」に直す。「アプリが描いた絵の表示」の行に drawable の export を加える。約 200 行の上限を守るため、同じ量の記述を簡潔にする |
| `docs/for-kivact-developer/decisions.md` | §16.4 の行を加える |
| `docs/for-kivact-developer/architecture.md` | crate の表に `kivact-shader` と `arvane-compose` を加え、`arvane-render` の行を「RenderScene のラスタライズ（Vello）」に直す。package 数（42、library 27）を直す。module の向きの表に compose を加える。「追加・変更するときのチェックリスト」の「layout/paint の性質」の行に「画素が要る効果は compositor の LayerEffects（§6.1）」を加える。「turn、Stage と時間」の節に Timeline を 1 行加える |
| `CONTRIBUTING.md` | GPU lane の試験（shader、GPU export）の実行方法を 2 行加える |
| `docs/for-kivact-developer/manual-verification.md` | shader の見た目、Linux の reduced motion（GNOME と KDE）、X11 の refresh rate、drawable の GPU export を手動確認の一覧に加える |
| `THIRD-PARTY-NOTICES.md` | §4.5 |

### 16.4 decisions.md に加える行

| ID | 問い | 決定 | 理由 |
|---|---|---|---|
| SCOPE-04 | 3D transform（rotateX 等）を足すか（SCOPE-03 を置き換える） | CSS Transforms 2 の 3D（perspective、preserve-3d、backface）を CPU と GPU で一致させて実装する | ゲーム風の UI と Web と同等の表現を目標にしたため（2026-11-07 の作者の決定） |
| SCOPE-05 | user shader をどう提供するか | WGSL をそのまま書く fragment shader（paint と filter）。GPU だけで実行し、CPU は宣言された代わりの見た目を描く | 表現の自由度と、Kivact が抱えるもの（CPU の解釈器）の少なさを優先した。描画以外（hit、a11y）には影響しない |
| SCOPE-06 | 色空間をどこまで扱うか | CSS Color 4 の空間で指定・補間し、gamut mapping して sRGB で出力する | UI library として必要十分であり、ブラウザの UI の扱いと同じ |
| SCOPE-07 | `matrix3d` の遠近の行を扱うか | 扱わない（最下行が (0, 0, 0, 1) のものだけ）。遠近は perspective() と perspective property で表す | 正準形で補間でき、遠近の利用はこの二つで足りるため |
| BOUNDARY-COMPOSE | 合成をどこに置くか | `arvane-compose` を新設し、Vello は layer の 2D ラスタライズだけにする | 3D、cache、shader、外部 texture は、Vello の外で一つの合成として扱うほうが単純であるため（Chrome の Skia と cc の分担と同じ） |
| BOUNDARY-SHADER | WGSL の検証をどこに置くか | `kivact-shader`（naga だけに依存） | style が構築時に検証して UiError を返せ、style に wgpu を持ち込まないため |
| CONTRACT-ISOLATION | blend の isolated group | CSS と同じ。stacking context（z-index による分離を除く）が isolated group になる | CSS の利用者の期待と一致させるため |
| CONTRACT-FILTER-SURFACE | 色の filter を各色に掛ける最適化（Tint）を残すか | 残さない。filter はすべて合成後の layer の surface に掛ける | clamp を含む色の行列は合成と交換できず、CSS と結果が変わるため。経路を一つにして内部も単純になる |
| CONTRACT-COLOR | 広色域の色をどこまで元の空間のまま運ぶか | gradient の stop（engine が CSS の補間と gamut mapping で展開する）、style の `Color::mix`、`Animated<Color>` は元の色を保ち、最後に一度だけ CSS の gamut mapping で sRGB へ写す。engine の state transition だけは、sRGB に写した端点から Oklab で補間する | wire の `PaintProps` の色の型をすべて変える複雑さを避けるため。D-06 に対する制限で、ユーザーが 2026-11-07 に承認した |
| CONTRACT-SPRING-DURATION | `Spring::with_duration` の duration の意味 | 静止するまでの時間。振幅 500 以下では振幅によらない（静止の判定は、位置と速度/ω0 に同じ相対の閾値 `rest` を使い、振幅を 500 で頭打ちにする）。step は解析解 | Framer Motion の利用者の期待と一致させ、duration と frame の間隔によらず契約を守り、最後に跳ぶ量を 0.5 以下に抑え、すべての係数で安定させるため。振幅 500 を超える場合の遅れは、ユーザーが 2026-11-07 に承認した |
| CONTRACT-LAYOUT-REPEAT | layout motion に反復の transition を与えたとき | 反復と往復を使わず、一回で goal へ動く（報告する） | layout motion は box を goal へ運ぶ動きであり、往復して元の位置で終わると layout と描画が食い違うため。ユーザーが 2026-11-07 に承認した |
| CONTRACT-RASTER-GRID | 動く layer の raster の格子 | 動く layer（R4、`will_change(Transform)`）は layer 空間に固定した格子で raster し、小数 px の位置は最後の合成で bilinear にする。止まっている layer は target の px に snap する | 動きの間に raster をやり直さないため（Chrome の合成 layer と同じ）。動いている間だけ、小数 px の位置で輪郭がわずかにぼける。ユーザーが 2026-11-07 に承認した |
| CONTRACT-GRADIENT-PRECISION | gradient の画素の精度 | engine の展開は式と 0.5/255 以内。最終の画素は Vello の LUT（最大 4096 項目）の量子化を含む | 独自の gradient の rasterizer を持たず、CPU と GPU で同じ Vello の経路を使うため。ユーザーが 2026-11-07 に承認した |
| CONTRACT-REPEAT-END | 反復した run の終わりの値 | 最後の反復の終わりの値（alternate で偶数回なら from）。`Animated`、state transition、Timeline で共通 | CSS の animation と Framer Motion と同じにし、経路で終わりの値が違わないようにするため |
| CONTRACT-3D-CROWD | 参加面の多い 3D コンテキスト | 参加面が 256 を超えたら、描画も hit も DOM 順の平坦な重なりにする（engine と compositor が同じ定数で判定する） | 描画と hit の食い違いを作らずに、BSP の分割の量を有限にするため。ユーザーが 2026-11-07 に承認した |
| CONTRACT-EXPORT-ACK | GPU の export の開始 | consumer の import が共有の release fence を `SHARED_PREPARED`（1）に signal するまで、window は fresh な slot に書かない（frame の値は 2 から）。それまでの export と、slot が無いときの export は、backoff（上限 100 ms）で再試行する。公開 API に ACK も unsafe も持たない | 既存の `SharedTextures` の hand-off の契約を、確かめられない自己申告ではなく GPU の fence で守るため。import の完了を知らせる処理は kivact-gpu-share の safe な `Import::new` の中にあり、利用者は import するだけでよい。止まった consumer で window も CPU も止めない |
| CONTRACT-BACKDROP-ROOT | backdrop が読む範囲 | CSS の Backdrop Root（effects を持つ layer）または root。effects が既定の layer（overlay、fixed、cache、3D、drawable）は透過として、その中身を重ねて読む | 実画素を読む合成で、CSS の Backdrop Root と同じ見え方にするため |

日付はすべて 2026-11-07 とする。

---

## 17. 試験

### 17.1 方針

- 振る舞いは、公開 API の E2E（`dev/tests/kivact-public-api/tests/e2e/`）で確かめる。window を開かない単体試験は作らない（Q-01。純粋な遷移や数式も例外にしない）。本書の §17.2 の試験はすべて E2E である。rustdoc の例（doctest）は文書であり、この規則の対象外である。
- 画素の比較の許容差は §3.3 の三段に従う。
  - 一段目（幾何）: 位置は ±1 device px。
  - 二段目（compositor の Op の結果）: channel ごとに ±2/255。
  - 三段目（Vello の raster を含む画素。輪郭の AA、glyph、box shadow の blur、gradient の LUT）: 最大差を決めない。既存の `kivact-test/src/screen/gpu_tests.rs` の `assert_close` の分布の基準（平均の差 ≤ 1.0、差 > 64 の画素 ≤ 0.5%、差 ≤ 8 の画素 ≥ 99%）を使う。
  - この基準は、kivact-test の公開 API に一か所だけ置く: `Screenshot::raster_difference(&self, other: &Screenshot) -> RasterDifference`（`pub struct RasterDifference { pub mean: f64, pub far: f64, pub close: f64 }`）と `RasterDifference::within_raster_tolerance(&self) -> bool`（上の三つの値を定数として持つ）。既存の `assert_close` もこれを使う形に書き換える（U55）。E2E は試験に閾値を書き写さず、これを呼ぶ。
  - 試験は、比べる画素がどの段かを試験名の近くのコメントに書く。
- 色や数式の期待値は、実装の関数で計算しない。本書の式から求めた既知の数値、または性質（範囲の内側にある、L と H が保たれる、など）で確かめる。
- GPU の試験は `RenderOptions::from_env().gpu().mirror(MirrorMode::Required)` を使い、`gpu_unavailable()` が理由を返したら skip する（既存の `gpu_interop.rs` と同じ形）。
- 新しい topic module は `tests/e2e/main.rs` に登録する。

### 17.2 新しい試験

各 topic の試験名と、確かめる内容を示す。試験はこの名前で作る。

**`transform_3d.rs`**

| 試験名 | 確かめる内容 |
|---|---|
| `rotate_y_narrows_the_drawn_box` | 200×100 の赤い box を `rotate_y(60)`（親に perspective なし）。box の中心の画素は赤。中心から ±60 px の画素は背景色（cos60 = 0.5 で幅 100 px） |
| `perspective_makes_the_far_edge_shorter` | 親に `perspective(400)`、子に `rotate_x(45)`。子の上端に近い行の赤の幅が、下端に近い行より短い |
| `hit_test_follows_the_projection` | 投影された中心のクリックは button の `on_press` を呼ぶ。元の box の内側でも投影の外側のクリックは呼ばない |
| `rect_is_the_projected_bounds` | `NodeRef::rect()` が §5.2 の式で求めた bounds と ±0.5 px で一致する |
| `preserve_3d_orders_by_depth` | `transform_style(Preserve3d)` の親の中で、先の子を `translate_z(50)`、後の子を `translate_z(0)` にして重ねる。重なりの画素は先の子の色で、クリックも先の子に当たる |
| `backface_hidden_is_not_drawn_or_hit` | `rotate_y(180)` かつ `backface_visibility(Hidden)` の node は描かれず、クリックも当たらない |
| `mirrored_backface_hidden_is_still_drawn` | `scale_xy(-1, 1)` かつ `backface_visibility(Hidden)` の node は鏡像として描かれ、クリックが当たる |
| `a11y_bounds_of_3d_nodes_are_projected` | 3D の button の支援技術の bounds が、投影の bounds と ±1 px で一致する |
| `focus_ring_and_caret_follow_3d` | 3D の text input に focus したとき、caret の矩形（IME の cursor area）が投影の内側にある |

**`compositor.rs`**

| 試験名 | 確かめる内容 |
|---|---|
| `partial_redraw_matches_full_redraw` | 多数の node のうち一つの色を変えた後の screenshot が、最終状態を新しく mount した screenshot と一致する。そのフレームの `frame_stats()` の `damage_pixels` が window の 10% 未満である |
| `damage_reaches_through_blur_and_backdrop` | blur を持つ親の中の子の色を変える場合と、backdrop の下の内容を変える場合の両方で、screenshot が新しい mount と一致する（damage の伝播） |
| `translucent_move_leaves_no_trail` | 半透明の node を動かした後の screenshot が、新しい mount と一致する（damage の部分消去） |
| `cached_layer_moves_without_raster` | window の内側に収まる `will_change(Transform)` の node を、小数 px を含む translate（0 → 37.25 px を 16 ms ごとの値）で動かす。各時刻の位置が期待どおり（±1 device px）で、最終の screenshot が新しい mount と ±2/255 で一致する。動かしている間の `Screen::frame_stats()` の `chunks_rastered` が 0 である（`RasterGrid::Layer`。§8.15） |
| `backdrop_sees_backdrop` | 二枚重ねた backdrop blur の内側の画素が、一枚だけのときの画素と異なる |
| `backdrop_in_overlay_sees_the_app` | overlay の中の panel の backdrop blur が、overlay の下のアプリの内容をぼかす（panel の内側の画素が、下の内容の blur の値になる） |
| `destination_out_cuts_a_hole` | Draw の layer に `composite(DestinationOut)` で円を描くと、その内側の画素が下の背景色になる |
| `many_cache_layers_keep_content` | 5000 個の `will_change` の node の button がすべて描かれ（各 button の中心の画素が背景と異なる）、最後の button をクリックできる（上限の inline、§8.14） |
| `cull_keeps_blur_reaching_into_view` | window の左端の外 20 px に置いた幅 10 px の box に blur(10) を掛けると、window の x = 0〜5 の画素が、同じ scene を左に 40 px 広い window で描いて切り出した画素と ±2/255 で一致する（効果の到達範囲で cull する。§8.3） |
| `inline_keeps_isolation_and_blend` | 5000 個の `will_change` の node と同じ window で、赤い背景の上に、透明な `isolation(Isolate)` の親とその中の不透明な青の `blend_mode(Multiply)` の子を置くと、子の画素が青になる（inline にしない。§8.14） |
| `coplanar_draw_order_is_kept_in_3d` | `transform_style(Preserve3d)` の canvas の同じ矩形に、赤を塗る → `scale_z(2)` の Draw layer の中で青を塗る → 緑を塗る、と描くと、矩形の画素が緑になる。同じ構成で、三つの命令の位置に置いた region のクリックが、緑の region に当たる |
| `crowded_3d_context_hits_what_is_drawn` | 互いに交差する 300 枚の button を一つの `Preserve3d` の親に置く（`MAX_CONTEXT_PLANES` を超える）。重なった点のクリックが、その点に最後に描かれた（DOM 順で後の）button の `on_press` を呼び、その点の画素がその button の色である |

**`effects_filters.rs`**

| 試験名 | 確かめる内容 |
|---|---|
| `contrast_applies_to_the_composite` | 半透明で重なる二つの子を持つ node に `contrast(2)`。重なりの画素が、合成後に matrix を掛けた値と一致する |
| `brightness_clamps_after_compositing` | 不透明の灰色 0.8 の上に灰色 0.2・alpha 0.5 の子を重ね、親に `brightness(2)`。重なりの画素が 1.0（255）になる（各色に先に掛けると 0.7 になる誤りを検出する） |
| `filter_graph_displaces_the_source` | turbulence と displacement_map の graph の出力が、元の画像と異なり、同じ入力では毎回同じになる |
| `filter_graph_rejects_handles_of_other_graphs` | graph A の handle を graph B の primitive に渡すと、その入力が Source に置き換わり、一度報告される。`FilterInput::SOURCE` はどの graph でも有効である |
| `backdrop_drop_shadow_is_applied` | backdrop_filter の drop-shadow が描かれる |
| `luminance_mask_uses_brightness` | 白から黒への gradient の luminance mask。白い側の画素は不透明、黒い側は透明 |
| `mask_composite_subtract` | 二層の mask を subtract で合成した結果の画素 |

**`borders_backgrounds.rs`**

| 試験名 | 確かめる内容 |
|---|---|
| `border_styles_draw_their_bands` | double、groove、ridge、inset、outset の各辺の標本点の色が §9.1 のとおり |
| `border_styles_per_side` | 辺ごとに異なる線種 |
| `border_image_nine_slice` | 3×3 の色分けした 30×30 の画像を slice(10) で 120×60 に描く。四隅の色、辺と中央の色、Round の tile 数の境界の画素 |
| `background_clip_text_paints_glyphs_only` | gradient の背景と透明な文字色。glyph の内側の画素は gradient の色、glyph の外は背景 |
| `background_origin_and_blend` | content-box の origin で画像の位置が padding だけずれる。multiply の blend の色 |

**`text_effects.rs`**

| 試験名 | 確かめる内容 |
|---|---|
| `text_stroke_outlines_glyphs` | glyph の輪郭の外側 1 px に stroke の色が出る |
| `paint_order_stroke_first` | StrokeFill では、glyph の内側の画素が fill の色になる |
| `font_variation_wght_matches_weight` | 試験用の `InterVariable.ttf`（`crates/arvane/tests/fonts`）で、`font_variation_settings([("wght", 700.0)])` と `weight(700)` の screenshot が一致し、`wght` 400 と異なる |
| `optical_sizing_changes_large_text` | `font_optical_sizing(false)` と true で、48 px の text の画素が異なる（Inter の opsz 軸） |
| `draw_text_with_gradient_and_path` | `TextFormat::fill` に gradient を指定した描画と、`text_path` の bounds が text の metrics の内側にあること |

**`color_spaces.rs`**

| 試験名 | 確かめる内容 |
|---|---|
| `color_conversion_fixtures` | 実装に依存しない既知の値と一致する: sRGB の赤の Oklab = (0.627955, 0.224863, 0.125846)、白の Oklab = (1, 0, 0)、sRGB の赤の Lab（D50）= (54.2905, 80.8049, 69.8910)、`color(display-p3 1 0 0)` の未 clip の sRGB = (1.093066, −0.226742, −0.150135)（各 ±1e-4。§5.4.1 の行列と伝達関数から求めた値） |
| `oklch_out_of_gamut_maps_like_css` | `oklch(0.7, 0.4, 30)` の背景の画素が sRGB の範囲にあり、Oklch に戻した L と H が元と ±0.02 で一致し、C が元より小さい（CSS の gamut mapping の性質） |
| `wide_gamut_gradient_interpolates_source_colors` | `color(display-p3 0 1 0)` から `oklch(0.6, 0.3, 300)` の Oklab の gradient の中央の画素が、二色を元の値のまま Oklab で補間し最後に写した値（試験の中で式を直接書いて計算する）と ±3/255 で一致する |
| `css_color_parsing` | §11.4.1 の例と、エラーになる入力（`none`、`calc()`） |
| `gradient_interpolation_spaces` | 赤から青の gradient の中央の画素が、Srgb で (128, 0, 128) ±2、Oklab で (140, 83, 162) ±3 になる（§5.4.1 の行列で求めた既知値） |
| `gradient_hint_moves_the_midpoint` | 黒から白の sRGB の gradient（幅 4000 px）で、hint 25% と 0.5% のそれぞれについて、hint の位置の画素が 128 ± 2 になり、行の画素が左から右へ減らない（単調）。hint 99.9% では、x = 3990 の画素が 64 未満、右端（x = 3999、t = 0.999875）の画素が 200 以上である（式の値は約 234、LUT の値は約 215。LUT の量子化を許す性質の試験。§9.6 の精度の範囲） |
| `same_color_gradient_matches_solid` | `oklch(0.7, 0.4, 30)` の単色の背景と、両端が同じその色の gradient の背景の画素が一致する（どちらも CSS の gamut mapping。§9.6） |
| `wide_gamut_to_white_gradient_maps_each_point` | `oklch(0.7, 0.4, 30)` から白への sRGB の補間の gradient の中央の画素が、二色を元の値のまま補間して最後に CSS の gamut mapping した値（試験の中で式を書いて計算する）と ±3/255 で一致する（端点を先に mapping した値とは G が約 59 違う） |
| `transition_interpolates_in_oklab` | 色の transition の途中の値が Oklab の補間と一致する |

**`images_animated.rs`**

| 試験名 | 確かめる内容 |
|---|---|
| `gif_frames_advance` | 試験の中で `image` crate（dev-dependency）で作る 3 frame・各 100 ms の GIF。`screen.advance(100ms)` ごとに画素の色が変わる |
| `playing_false_shows_first_frame` | `playing(false)` では最初の frame のまま |
| `reduced_motion_shows_first_frame` | reduced motion では最初の frame のまま |
| `apng_and_webp_animate` | APNG と animated WebP も同様に進む |
| `svg_text_is_drawn` | 登録した font を使う `<text>` の SVG で、text の位置の画素が背景と異なる |
| `svg_data_url_image_is_drawn` | data URL の PNG を含む SVG で、画像の画素が出る |

**`shaders.rs`**

| 試験名 | lane | 確かめる内容 |
|---|---|---|
| `invalid_wgsl_reports_line_and_column` | CPU | 構文の誤りが `ErrorKind::Shader` で、行と列が利用者のソースの位置になる |
| `forbidden_bindings_are_rejected` | CPU | `@group(0) @binding(0)` の global や `@fragment` を書くとエラーになる |
| `wrong_entry_signature_is_rejected` | CPU | 入口の型が違うとエラーになる |
| `cpu_draws_the_fallback` | CPU | CPU 描画で fallback の色が描かれる |
| `param_type_mismatch_is_reported` | CPU | param の型の不一致が一度報告される |
| `gpu_paint_shader_output` | GPU | `return vec4(position / context.size, 0.0, 1.0)` の各標本点の画素 |
| `gpu_filter_shader_inverts` | GPU | `content()` を反転する filter で、画素が反転する |
| `gpu_shader_time_advances` | GPU | `animate(FrameRate::Display)` で、advance の後に出力が変わる |

**`drawables.rs`**

| 試験名 | 確かめる内容 |
|---|---|
| `drawable_is_drawn_where_the_canvas_draws_it` | translate と rotate の下に描いた drawable の画素の位置 |
| `drawable_button_is_clickable_through_3d` | 3D の layer の下の drawable の button を、投影された位置のクリックで押せる |
| `drawable_content_is_focusable_and_findable` | `get_by_role(Role::Button, "Apply")` で見つかり、Tab で focus が中へ入る |
| `drawable_a11y_bounds_are_projected` | 支援技術の bounds が、描いた変換の bounds と一致する |
| `drawable_mounted_twice_reports` | 同じ `Drawable` を二つの node の `.drawable(d, …)` に渡すと、二つ目は mount されず一度報告される。一つ目を unmount した後なら、別の node に mount できる。別の node の `Draw::drawable(d)` は描かれず報告される |
| `drawing_a_drawable_twice_reports` | 二度描くと一度報告され、最初の位置だけに描かれる |
| `unplaced_drawable_is_not_hit` | 描かない drawable はクリックが当たらず、a11y の bounds が無い（node は a11y の tree に残る） |
| `draw_over_update_keeps_draw_placement` | `draw` に drawable を描き、`draw_over` だけを更新しても、drawable の button がクリックでき、a11y の bounds が変わらない |
| `drawable_twice_across_slots_uses_the_first` | `draw` と `draw_over` の両方に同じ drawable を描くと、`draw` の位置だけに描かれ、そこだけに当たり、一度報告される |
| `clipped_drawable_is_not_hit_outside_the_clip` | Draw の clip の外に出た drawable の button の部分は描かれず、クリックも当たらない。clip の内側の部分は当たる |
| `backface_hidden_drawable_is_not_hit` | backface Hidden の 3D の Draw の layer の中で裏を向いた drawable は描かれず、当たらない |
| `nested_flat_backface_matches_compositor` | 外側の Flat の layer を `rotate_y(180)`（Visible）、内側の Flat の layer を `rotate_y(180)`・Hidden にして drawable を描くと、drawable は描かれず、クリックも当たらない。同じ二重の回転を `Preserve3d` の親の中で行うと（累積は表）、描かれてクリックが当たる |
| `pixels_frames_of_old_generation_are_dropped` | `request_pixels` の直後にもう一度 `request_pixels` を呼ぶと、`latest` と `wait` が返す frame は新しい generation のものだけである |
| `wait_cursor_survives_generation_change` | 古い generation の frame の number を `after` に渡した `wait` は、新しい generation の最初の frame を timeout せずに返す（number は単調に増える） |
| `exported_drawable_pixels_reach_another_thread` | `Viewport::drawable` の `share().request_pixels()` で、別の thread の `wait` が期待どおりの内容の frame を受け取る |
| `exported_drawable_geometry_enables_hits` | `set_geometry` の後、viewport の中の写像の位置のクリックが drawable の button に当たる |

**`drawable_export_gpu.rs`**（gpu-interop、Windows/Linux、GPU lane）

| 試験名 | 確かめる内容 |
|---|---|
| `gpu_export_matches_cpu_export` | `request_gpu(adapter())` で得た共有 texture を試験の wgpu device に import して読み戻した画素が、CPU export の画素と一致する |
| `stalled_consumer_does_not_stop_the_window` | アプリが import しない場合と、import して release しない場合のそれぞれで、drawable の内容を 10 回変えても window の提示が続き（`Presented::Shown` が続く）、他の node の変化が画面に出る |
| `export_generation_rejects_stale_frames` | resize の後、`wait` が新しい generation の `ShareUpdate::Textures` を返し、古い generation の frame は `latest` に出ない |
| `documented_consumer_loop_survives_resize` | §10.5 の consumer の繰り返しを、そのままの形で別 thread で動かす。初回の frame が届き、drawable の大きさを 2 回変えた後も、それぞれ新しい大きさの frame が届く |
| `export_mode_switch_resends_static_content` | 内容を変えずに Pixels → Gpu → Pixels → stop と切り替えると、各 mode で最初の frame が届き、stop の後は届かない |
| `gpu_frames_start_after_import` | `request_gpu` の後、`Import::new` で import するまで GPU の frame は来ず、window の提示は続く。import の後 200 ms 以内に frame が来る。resize の後、古い generation の textures を import しても、新しい generation の frame は来ない（新しい textures を import すると来る） |
| `released_slot_delivers_latest_without_other_updates` | consumer が release しないまま drawable の内容を一度だけ C に変え、その後 release すると、他の UI の変化が無くても 200 ms 以内に内容 C の frame が届く。release しない間の提示の間隔が 100 ms まで伸びる（busy loop にならない） |
| `gpu_export_recovers_after_device_loss` | GPU の export が流れている間に `Screen::lose_gpu_device()` を呼ぶと、window の提示は続き、`wait` が新しい generation の `ShareUpdate::Textures` を返す。import すると、新しい generation の frame が届く。古い generation の喪失は、その後の新しい request を戻さない |
| `retry_stops_when_content_returns` | release しない間に内容を A（最後に送った内容）→ C → A と変えると、A に戻った後の提示で再試行が止まり、以後 frame の要求が出ない（`ScreenDriver` の仮想時計で 1 秒進めても提示が増えない） |

**`motion_easing.rs`**

| 試験名 | 確かめる内容 |
|---|---|
| `css_named_easings` | Ease、EaseIn、EaseOut、EaseInOut の 25%/50%/75% での値（`animated` を `screen.advance` で読む）が、cubic-bezier の値と一致する |
| `linear_points_and_steps` | linear() の点と、jump-none、jump-both の段 |
| `back_elastic_bounce_shapes` | 各 easing の 50% の値（§12.1 の式） |
| `keyframe_segment_easing` | 区間ごとの easing |
| `spring_with_duration_settles_near_duration` | `with_duration(500, b)`（b = −0.99、−0.5、0、0.094、0.5、0.99）が、0→1、0→100、0→0.0005 のどれでも、frame の間隔 1 ms、16 ms、100 ms のそれぞれで、500〜600 ms で静止する（§12.2 の E の判定。bounce 0.094 は一瞬だけ小さくなる時刻がある場合） |
| `spring_extremes_stay_finite` | `with_duration(500, −0.99)` と `with_duration(500, 0.99)` の `Animated` の 0→1 が、各 frame で有限の値である。16 ms と 100 ms の frame 間隔で、同じ時刻（400 ms）の値が ±1e-6 で一致する（解析解の step） |
| `alternate_even_count_ends_at_from` | `tween(100).repeat(2).alternate()` の `Animated`、Timeline の区間、state transition の三つが、どれも from の値で終わる（§12.3）。`layout_transition` に同じ transition を与えると、報告が一度出て、layout motion は一回だけ goal へ動く |
| `set_velocity_after_alternate_resumes_the_target` | spring の `Animated` を 0 から `set(10)` し、`repeat(2).alternate()` で 0 に戻って完了した後、`set_velocity(5)` を呼ぶと 10 へ向かって動き、10 で静止する（target は `Completed` に残る） |
| `invalid_spring_is_reported` | stiffness 0 の spring を style の transition と `animated` に与えると、それぞれ一度報告され、値は即座に target になる |
| `set_velocity_moves_an_idle_spring` | 静止中の spring に速度を与えると動いて戻る |

**`timeline.rs`**

| 試験名 | 確かめる内容 |
|---|---|
| `segments_follow_positions` | AfterPrevious、WithPrevious、Time、Label の開始時刻 |
| `segment_delay_and_repeat_match_transition` | 100 ms、delay 50 ms、2 回の区間の長さが 250 ms（`Transition` と同じ）で、後続の区間がその直後に始まる |
| `playback_settings_persist_across_phases` | `set_rate(2)` の後の `play`、play → pause → play、reverse → finish → play が §12.6 の遷移表どおりの時刻と向きになる |
| `seek_reverse_and_rate` | seek の値、reverse で戻る、rate 2 で半分の時間 |
| `repeat_and_alternate` | 反復と往復 |
| `calls_fire_on_playback_not_seek` | call の発火の規則 |
| `drive_by_progress` | progress の値で track が決まる |
| `reduced_motion_finishes_decorative` | reduced motion の play で即座に Finished になり、`on_complete` が呼ばれる |
| `stagger_delays` | Start、End、Center、Edges と easing の遅延 |

**`scroll_timeline.rs`**

| 試験名 | 確かめる内容 |
|---|---|
| `scroll_progress_tracks_offset` | `scroll_to` の後の値 |
| `view_progress_ranges` | cover、contain、entry、exit の §12.8 の値 |

**`presence_modes.rs`**

| 試験名 | 確かめる内容 |
|---|---|
| `wait_mounts_after_exit` | Wait では、exit の 200 ms の後に新しい腕が現れる |
| `pop_layout_reflows_at_once` | PopLayout では、兄弟の位置がすぐ詰まる |
| `sync_keeps_both` | Sync では、両方が同時に描かれる |

**`shared_layout.rs`**

| 試験名 | 確かめる内容 |
|---|---|
| `layout_id_starts_from_the_departed_box` | 一覧の card から詳細へ切り替えると、詳細の node の rect が、最初は元の card の rect、終わりで自身の rect になる（記録は remove の後の最初の layout で消費される） |
| `layout_motion_maps_boxes_exactly` | goal (x 100, w 100) から current (x 0, w 200) への途中の 0 ms の描画で、node の左端の画素が window の x 0 にある。親原点が 0 でない場合と、拡縮と移動が同時の場合も同様（§12.12 の検算） |
| `nested_layout_motion_keeps_children_steady` | 親だけが大きさを変えるとき、layout_transition を持つ子の window の rect が、親の拡大縮小で歪まない |
| `three_level_layout_motion_moves_once` | root、parent、child がすべて layout_transition を持ち、すべて 10 px 右へ移るとき、0 ms の描画で child の左端の画素が移動前の位置にある（補正が重複しない）。親原点が 0 でない場合と、拡縮を含む場合も同様（§12.12 の検算） |
| `layout_id_exit_records_once` | exit animation を持つ layout_id の node を外して exit が終わった後、同じ id の node を新しく mount しても、古い box から動かない（記録は `begin_exit` の一度だけで、最初の layout で消える。§12.11） |
| `radius_is_corrected_while_scaling` | 拡大縮小の途中で、角の画素が元の丸めと一致する |

**既存の `motion.rs`**

| 試験名 | 確かめる内容 |
|---|---|
| `per_property_transition` | Opacity だけ transition を持つ node で、色は即座に変わり、opacity は時間をかけて変わる |
| `filter_to_none_interpolates` | `blur(4)` → none の 50% で blur が 2 の画素になり、終わりで filter の無い画素になる。`brightness(2)` → none の 50% は brightness 1.5 の画素である |
| `filter_identity_values` | none → `grayscale(1)` の 50% の画素が `grayscale(0.5)` の画素と一致する（初期値の表。§12.9.2） |
| `shadow_list_shortening_interpolates` | (外側の shadow、inset の shadow) → (外側の shadow) の途中で、inset の shadow が透明へ薄れながら inset のまま描かれる |
| `reversed_blur_keeps_its_velocity` | spring の transition で `blur(0)` → `blur(10)` の途中（blur ≈ 5）で戻すと、戻した直後の frame の blur の変化が直前の速度と連続する（同じ shape の retarget。§12.9.3） |
| `retarget_across_shadow_counts_starts_from_shown` | shadow 1 個から 2 個への transition の途中で 3 個へ retarget すると、retarget の直後の frame の 1 個目の shadow が直前の frame と連続し、2 個目以降は透明から始まる（どの shadow も即座に target へ跳ばない） |
| `interrupted_transition_continues_from_shown` | hover の transition の途中で hover を外すと、外した直後の frame の値が、直前の frame の値から 1 frame 分の変化しか動かない（旧始点へ跳ばない）。spring では速度も続く |
| `frame_loop_pauses_under_reduced_motion` | decorative な loop は止まり、essential な loop は動く |

### 17.3 API 変更の移行表（workspace 内の全利用箇所）

| 変更前 | 変更後 |
|---|---|
| `Easing::steps_start(n)` / `steps_end(n)` | `Easing::steps(n, StepPosition::JumpStart)` / `JumpEnd` |
| `StepPosition::{Start, End}` | `StepPosition::{JumpStart, JumpEnd}` |
| `Easing` の `Copy` | `Clone`（`.clone()` を足す） |
| `PaintVec` | 削除。`prop_interp`/`apply_components`（`PropInterp`、`PropBasis`）（§12.9.2） |
| `Tint::matrix`、`RenderCmd::{Image, ImageSampled}::color_matrix`、`Paint::filtered` | 削除。色の filter は layer に掛ける（§6.3、§5.8） |
| `Spring` の静止の判定 | 絶対値の位置と速度から、振幅に対する相対値の一つの閾値 `rest`（既定 1e-3。速度は ω0 で割って比べる）へ（§12.2）。新しい field `rest` を加える。`Spring { .. }` を直接書く箇所は `rest: 1e-3` を足す |
| `FailureKind` | variant `NotReady` を加える（§8.12）。`FailureKind` を match する箇所を追従させる |
| `SharedTextures` の契約の順序（import が `Ok` を返した後に最初の書き込み） | release fence が `SHARED_PREPARED`（1）に達した後に最初の書き込み。frame の値は 2 から（§8.12）。kivact-gpu-share の pool と `Import::new` は内部で従う。自前の graphics API のコードで texture を作る producer は、この順序に合わせる |
| `SpringRun::restart(now_ns)` | `SpringRun::start(now_ns, &transition, from, to)`（§12.2） |
| `layout_transition` の `repeat`/`alternate` | 使わない（報告する。§12.3） |
| spring の step（半陰的 Euler 法） | 解析解（§12.2）。spring の途中の値を固定値で比べる既存の試験は、期待値を解析解の値に更新してよい |
| alternate で回数が偶数の run の終わりの値（to に snap） | from（§12.3）。`Animated`、state transition、Timeline のすべて |
| `SpringRun` の field | `amplitude: Vec<f64>` を加える（§12.2） |
| 意味情報の `bounds: Rect` | `Option<Rect>`（§5.15） |
| `FilterOp` の `Copy`（`Graph` と `Shader` が `Arc` を持つ） | `Clone` |
| `TransitionProp::Paint` | `TransitionProp::All`（意味は同じ: 個別の指定の無い paint の property すべて。`Size` は含まない） |
| `kivact_style::TransitionProp` | `kivact_types::TransitionProp`（`kivact::style::TransitionProp` の path は同じ） |
| `Effects::mask: Option<FillLayer>` | `Effects::masks: Vec<MaskLayer>` |
| `Style::mask(impl Into<Paint>)` | `Style::mask(impl Into<Mask>)`（`Paint` からの変換があるので、多くの呼び出しは変更不要） |
| `Border::style` | `Border::styles: Edges<BorderStyle>` |
| `Layer::mask(impl Into<Paint>)` | `Layer::mask(impl Into<Mask>)` |
| `Draw::paragraph(p, origin, color)` | `Draw::paragraph(p, origin, fill)`（色は `Into<Paint>`） |
| `DrawCmd::Paragraph { color }` | `{ fill, stroke }` |
| `RenderCmd::{PushLayer, PopLayer, Backdrop, External}` | §5.8、§5.11 |
| `render::Paint::LinearGradient` | `render::Paint::Gradient` |
| `ElementTransform { translate: Point, rotate: f64, skew_x: f64, scale: Point, .. }` | §5.5 |
| `Region::transform: Transform` | `Projective` |
| `dynamic(..) -> View` | `dynamic(..) -> Dynamic<T>`（`IntoView`） |
| `kivact_types::scene::Pixels`、`kivact_types::render::ColorMatrix` | `kivact_types::Pixels`、`kivact_types::ColorMatrix` |
| `arvane_render::{CpuRenderer, GpuContext, GpuEnv, GpuSurface, Presentation, GPU_BUILT}` | `arvane_render::CpuRaster`、`arvane_compose::{GpuContext, GpuEnv, GpuSurface, Presentation, GPU_BUILT}` |
| `arvane::Window::present(&RenderScene, FramePlan)` | `present(&Composition, FramePlan)` |
| `ShellWindow::present_with(w, h, draw)` | `present_with(w, h, damage, draw)`（§15.1） |
| `kivact::gpu_interop::ShareError` | `kivact::view::ShareError` |
| `kivact_ui_creative::timeline`（timeline の編集の widget） | `kivact_ui_creative::timeline_editor`（prelude の `kivact::motion::timeline` と名前が衝突するため。A-4 で決めた。文書の `ui::timeline(..)` も直す） |
| background の positioning area（border box） | padding box（CSS の既定）。kit の背景画像の位置が border の幅だけずれる箇所は、`.origin(BackgroundOrigin::BorderBox)` で元の見た目に合わせる |

---

## 付録 A. 既定の裁定

オーケストレータが §0.2 の「意味の衝突」を裁定するときの既定である。上から順に当てはめ、最初に当たったものを使う。結果は PR 説明の草稿に 1 行で記録する。

| ID | 状況 | 裁定 |
|---|---|---|
| A-1 | 本書が CSS・SVG・Web Animations の細部を定めていない | その節が挙げた仕様に従う。仕様も決めていなければ、Chrome の挙動に合わせる |
| A-2 | 依存 crate（vello、wgpu、naga、parley、resvg、image、zbus、x11rb）の API の名前や引数が本書の想定と違う | 意味が同じ API を使う（名前、引数の並び、型の包み方の違いだけを吸収する）。同じ意味の API が無く、本書の挙動を実装できない場合は、その unit の該当部分を止め、オーケストレータがユーザーに報告して決定を待つ。近似の実装や、試験の期待値を近似に合わせることはしない。他の unit は続ける |
| A-3 | 単純さと性能が衝突する | 単純さを選ぶ。E2E が時間切れになる場合に限り、性能を選ぶ |
| A-4 | 新しい公開名が既存の名前と衝突する（name-collision、canonical-path の検査） | 本書の名前を保ち、衝突する kit 側の私的な名前を変える。facade の名前同士なら、本書の名前に domain の接頭辞を付ける |
| A-5 | state-fields の lint が新しい passive record を指摘する | 各 field の直前に、正確な理由を書いた `xtask-allow(state-fields)` を付ける。lint を黙らせるために意味を変えない |
| A-6 | 画素の試験が CPU と GPU、あるいは Windows と Linux で許容差を超えて揺れる | §3.3 の三段のどれに当たるかを確かめる。compositor の Op の差なら実装を直す。Vello の raster の段なら、その段の既存の差の範囲かを確かめる。どちらでもなければ A-2 と同じく止めて報告する |
| A-7 | 別の unit が担当する crate が、自分の変更で compile できなくなる | 意味を変えない最小の修正（import、型の追従）だけをし、PR 説明に書く |
| A-8 | 1 関数 80 行、1 file 1000 行の上限を超える | 意味のある helper または module に分ける。line packing はしない |

## 付録 B. 再開時の裁定（2026-11-07、オーケストレータ）

wave R1 で A-2 により止まった項目について、ユーザーの指示（判断はオーケストレータに委ねる）により、オーケストレータが次のように決めた。本文と食い違う場合は、この付録が優先する。

| 番号 | 対象（unit、節） | 裁定 |
|---|---|---|
| B-1 | R11 項目 2、§8.2、§8.8.5、§11.3.4 | filter shader は `Op::Filter`（`LayerFilter::Shader`）だけで実行する。入力 target を `content` とし、`kivact_context.size` は layer の bounds とする（`Op::Filter::bounds`）。`ShaderOp` は paint shader 専用とし、`ShaderOp::input` を削除する。`coverage` と `fallback` は `ChunkKey` のまま（paint shader には必ずある）。GPU の `ShaderOp::input` を bind する経路も削除する |
| B-2 | R11 項目 6、§8.1、§8.6 | `Retained` の payload は、`retain` と同じく pointer の寿命を保つためだけに持つ。`Retained` の enum にも `#[expect(dead_code, reason = "held so the hashed pointers stay valid (§8.6)")]` を付ける |
| B-3 | R11 項目 6、§8.2 | `ChunkTask::index` を削除する。chunk の index は `Plan::chunk_records` の記録だけが運ぶ（記録に index が無ければ、記録へ移す） |
| B-4 | R11 項目 6、§8.2、§8.11 | `Source::Pixels` を削除する。§8.11 の「`cpu` を `Source::Pixels` として描く」は、「`Source::External` の `cpu` の画素を描く」と読み替える |
| B-5 | R11、R12、§8.2 | GPU の executor だけが読む plan の field（`ShaderOp` の `shader`、`map`、`size`、`px_per_unit`、`Op::Filter::bounds` など）には `#[cfg_attr(not(feature = "gpu"), expect(dead_code, reason = "read only by the GPU executor"))]` を付ける |
| B-6 | R16 項目 3、§11.3.1、§0.3 | kivact-style の `paint::layer`↔`shader` の循環は依存の向きで解く。shader の program の data（program、param、clock など）は `Paint` を名指ししない shader の leaf に置く。`ShaderPaint`（`fallback: Option<Paint>` を持つ）は `Paint` と `PaintImage` の側（paint）に置く。`ShaderPaint` を作る builder は、paint に依存する別の shader の leaf に置く。`shader.rs` は宣言と再公開だけにし、公開の path は変えない |
| B-7 | R15 項目 2、§7.3、§6.4 | （ユーザー承認の案 (a)）layer の分類を kivact-engine の leaf `layer` に置く。`layer` は `Model` を名指ししない。分類の関数は、tree と必要な事実を引数で受け取る。`Model` の包み関数は model の側に置く。policy は `layer` の key（最小の uses）と、`model` と `paint` の uses への `layer` の追加。不要になった except（`model::interact::sync -> paint::state`）は削除する |
| B-8 | R14 項目 2、§9.3 | content box は `kivact_layout::NodeLayout::content_box`（公開済み）を使う。padding box は border box を computed の border 幅だけ縮めたもの。新しい layout の API は作らない |
| B-9 | R14 項目 3、§9.4 | box の描画は、node 自身の `ShapedText` を、text の描画と同じ cache から取る（未 shape なら先に shape する）。描画の順（背景が先、text が後）は変えない |
| B-10 | R14 項目 8、R21、§12.11 | engine は、id の重複を見つけたとき `HostEvent::DuplicateLayoutId { node, id }` を一度出す。kivact-core の dispatch（R21）が、node の owner に `UiError(ErrorKind::Message, "duplicate layout_id {id}")` を一度報告する |
| B-11 | R15、§12.11 | `kivact_types::LayoutProps::layout_id` を削除する（§12.11「LayoutProps には加えない」） |
| B-12 | R11、R12、§8.1、§4.4 | turbulence と色の表のように CPU と GPU の両方が使う表は、`cpu` でも `gpu` でもない leaf（`crate::tables`）に置く。policy の `cpu` と `gpu` の uses に `tables` を加える。`gpu` は `cpu` を参照しない |
| B-13 | R12、§8.1 | `gpu/mask.rs`（mask と clip の pass、§8.9）を §8.1 の module の一覧に加える |
| B-14 | R11、§8.1、§0.3 | `GpuReport` の alias と `GPU_BUILT` は leaf に置き、`lib.rs` は宣言と再公開だけにする（§0.3 が §8.1 の code block に優先する） |
| B-15 | CP8、§12.5、§17.2 | reduced motion の間、decorative な frame loop は tick しない（§12.5）。既存の試験 `motion::reduced_motion_completes_decorative_transitions_only` が、decorative な loop が回り続けることを期待していた部分は、設計の決定で意味が変わったものとして期待値を更新する。作業指示書 §10 CP8 の「期待値を更新してよいもの」に加える |
| B-16 | CP8、§0.3、§8.1 | 特定の feature（例: `testing`）のときだけ使う item は、その feature のときだけ存在させる（`#[cfg(feature = …)]`）。feature の組み合わせによって未使用になる item に `allow` を付けない |
| B-17 | U92、§17.2 | `color_conversion_fixtures` の変換関数は内部にあり公開 API から値を読めない。既知の Oklab と Lab（D50）の値が sRGB の赤・白として描かれること（±1/255）と、`color(display-p3 1 0 0)` が CSS の gamut mapping の値で描かれること（二段目）を画素で確かめる。±1e-4 の数値の照合は公開 API で観測できないので行わない（試験のための公開 API を足さない） |
| B-18 | U92、§17.2 | `oklch_out_of_gamut_maps_like_css` の H の許容差は ±1.0°とする（8 bit の画素からは 0.02° を分解できない）。L は設計どおり ±0.02 |
| B-19 | U93、U94、§10.5 | `DrawableShare::set_geometry(Option<Matrix4>)` を利用者が呼べるよう、`kivact_core::geometry`（facade の `kivact::geometry`）で `Matrix4` を再公開する |
| B-20 | U93、U95、§12.4、§16.1 | 区間ごとの easing（§12.4）を利用者が書けるよう、`kivact::motion` で `Keyframe` を再公開する（U93 の `fix(kivact-core)`）。U95 が A-2 で止めた `animation.md` の keyframe の easing と、U94 が書かなかった `drawables.md` の `set_geometry`（B-19）は、wave 9 の merge の後に文書の修正の unit で書き足す |
| B-21 | U91、§17.2、CONTRACT-BACKDROP-ROOT | `backdrop_drop_shadow_is_applied` は、不透明な backdrop が自身の drop shadow を隠す（CSS と同じ）ため、透明な背景の Backdrop Root（effects を持つ layer）の中に小さな不透明の図形を置き、図形を (dx, dy) ずらした位置の画素が shadow の色になることを確かめる |
| B-22 | U93、§10.5、CONTRACT-EXPORT-ACK | `drawable_export_gpu.rs` の consumer は、§10.5 の文書どおり `kivact-gpu-share` の `Import`（`Import::new`、`Import::release`）を使う。試験の crate は `kivact-gpu-share`（import と wgpu の feature）を dev-dependency に加えてよい。facade に consumer の API を足さない |
| B-23 | U93、§17.2 | `invalid_spring_is_reported` の「即座に target」は、報告の後の最初の frame で target になることとする。`presence_modes` の試験は、exit 中の node が test id の query に出ないため、新しい腕と兄弟の位置で確かめる |
