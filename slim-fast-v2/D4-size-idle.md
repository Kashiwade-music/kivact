# P4 設計：K6 バイナリサイズと K2 counter idle WS の基礎分

> **V2 付属書 D4**（設計担当 P4 の成果物）。正本は [SLIM_FAST_V2_DESIGN.md](../SLIM_FAST_V2_DESIGN.md)、手順は [SLIM_FAST_V2_WORK_ORDER.md](../SLIM_FAST_V2_WORK_ORDER.md)。
> - 本書の「8 章 設計者（人間）の判断が要る点」は V2 設計書 4 章（DEC-01〜DEC-23）で決定済み。実装者は 8 章を読んで選ばない。V2 設計書 5 章（付属書間の調整）が本書を上書きする箇所がある。
> - 本書の unit 名と V2 の unit ID の対応は V2 設計書 0.2 節。unit の実施順・lane・wave は V2 作業指示書だけが決める（本書 6 章の「依存」「並列可」は参考）。
> - 文中の P1〜P8 は付属書 D1〜D8（`design-P<n>.md` は `D<n>-*.md`）を指す。`W:\w\kv-prof`・`W:\w\kv-mem`・`W:\w\kv-size`・scratchpad の path は設計時の試作・計測の場所（設計した機械にだけある参考）で、実装に必要ではない。行番号は HEAD `73c36c7c` のもの。

担当：K6（counter default ≤ 14,467,313 B、kv_startup no-default-features ≤ 11,177,658 B、basic catalog default ≤ 31,912,659 B）と K2 counter idle WS の基礎分（default ≤ 27,214 KiB、cpu ≤ 26,694 KiB）。

計測はすべてこの機械（Windows 11 10.0.26200、Rust 1.98.1、表示倍率 100 %）で、detached worktree `W:\w\kv-size`（基点 db8d679b）を `CARGO_TARGET_DIR=W:\w\kv-size-target`、`cargo build --release -j 8` で build した。worktree には計測のための一時変更だけを入れ（下記）、リポジトリは編集していない。

- 計測用の一時変更：`dev/bench/Cargo.toml` で `gpu` を bench の feature にした（`default = ["complex-scripts", "gpu"]`、`gpu = ["kivact/gpu"]`、`kivact` の `features = ["gpu"]` を外す）。これで `kv_startup` の 4 構成（gpu × complex-scripts）を同じ binary で比べた。`--no-default-features --features gpu` が正式 K6 行（B1 の 15,922,688 B）と byte 単位で一致することを確かめた。
- WS は `dev/bench/run.sh` と同じ条件（初回 present の後 3 s）で、image 名から PID を取る PowerShell + `QueryWorkingSet`/`VirtualQueryEx`/`GetMappedFileName`（scratchpad の `wsbreak.ps1`）で page 単位に分類した。実 window は全て `dev/tools/window-lock.sh` を通した。

---

## 1 事実

### 1.1 K6：構成ごとの実測（byte）

`kv_startup` は counter とほぼ同じ画面（B1 で counter 18,978,816 / kv_startup default 18,976,768）。ここの値は全て db8d679b のコード。

| 構成 | 大きさ | base との差 | 備考 |
|---|---:|---:|---|
| counter default（B1 profile） | 18,978,816 | — | B1 と一致（再現性あり） |
| kv_startup default（gpu + complex-scripts） | 18,976,768 | 0 | base |
| 同、strip=none + debug=line-tables-only | 18,976,768 | 0 | MSVC の exe は symbol を持たない。`strip` は exe の大きさに効いていない（効かせる余地がない） |
| 同、hot crate も全部 `opt-level="s"` | 18,114,048 | −862,720 | hot crate の opt 3 の費用は 0.86 MB |
| 同、cold crate を `opt-level="z"`（hot は 3 のまま） | 17,620,480 | −1,356,288 | |
| 同、wgpu の `dx12` なし | 18,407,424 | −569,344 | Windows は Vulkan のみになる |
| 同、wgpu の `vulkan` なし | 18,111,488 | −865,280 | Windows は DX12 のみになる |
| **`gpu` なし**（complex-scripts あり） | **14,406,656** | **−4,570,112** | GPU 一式は 4.57 MB |
| `complex-scripts` なし（gpu あり）＝正式 K6 行 | 15,922,688 | −3,054,080 | B1 と一致 |
| gpu なし・complex-scripts なし | 11,352,576 | −7,624,192 | |
| basic catalog default | 43,090,432 | — | B1 と一致 |
| B0 のコード（79bbd409）+ B1 の profile、counter | 19,009,024 | +30,208（counter 比） | §1.1.1 |

section（`llvm-size -A`、byte）：

| exe | .text | .rdata | .pdata | .reloc |
|---|---:|---:|---:|---:|
| kv_startup default | 12,458,942 | 6,071,180 | 382,128 | 42,148 |
| kv_startup gpu なし | 9,097,118 | 5,009,218 | 249,540 | 28,540 |
| kv_startup cs なし | 12,448,814 | 3,027,188 | 381,912 | 42,120 |
| kv_startup どちらもなし | 9,087,022 | 1,965,226 | 249,324 | 28,468 |
| basic catalog default | 21,855,486 | 20,165,908 | 939,948 | 96,828 |

- complex-scripts は .rdata だけ（ICU の辞書）で 3.05 MB。GPU 一式は .text 3.36 MB、.rdata 1.06 MB、.pdata 0.13 MB。
- catalog は counter より .text が +9.40 MB、.rdata が +14.09 MB。.rdata の内 10.64 MB は catalog が `include_bytes!` する font（`examples/kivact-ui-basic/src/fonts.rs:6-8`：NotoSansJP[wght].ttf 9,589,900 B、Inter 876,576 B、GeistMono 171,948 B）。

#### 1.1.1 B0 のコードに B1 の profile を当てた場合（設計 1.2 節の「14.5 MB」の検証）

worktree を 79bbd409 に切り替え、root `Cargo.toml` に B1 と同じ `[profile.release]`（hot crate の opt 3 を含む）を足し、R-0（`crates/kivact-engine/src/check/shaped_lines.rs:51-56` の `debug_assert_eq!`）だけを消して counter を build した。

| コード | profile | counter default |
|---|---|---:|
| B0（79bbd409） | B1 と同じ（fat、cgu 1、s、hot 3、strip） | 19,009,024 |
| B0（79bbd409） | 同、全 crate `s`（設計 1.2 節の「14.5 MB」と同じ指定） | 18,143,232 |
| B1（db8d679b） | B1 と同じ | 18,978,816 |
| B1（db8d679b） | 同、全 crate `s` | 18,114,048（kv_startup） |

**結論：設計 1.2 節の「fat LTO、cgu=1、opt-level="s"、strip で 14.5 MB」は再現しない**（同じ指定で B0 のコードは 18.14 MB）。Slim-Fast の実装はコードの量をほとんど変えていない（同 profile で −30 KB）。B1 が「見込みの 14.5 MB」より大きいのは、見込みの元の数字が誤っていたためで、実装の欠陥ではない。hot crate の opt 3 の費用は 0.87 MB（B0・B1 とも）。

### 1.2 K6：code の内訳（counter の PDB、`llvm-pdbutil dump --symbols` の S_*PROC32 の code size を crate ごとに集計）

fat LTO で object は 1 つなので section contribution は crate を区別しない。関数の code size（inline 先は呼出し側に入る）を、名前の最初の非 std の crate に帰属させた。合計 12,577,950 B（.text とほぼ一致）。

| 系統 | crate（code byte） | 計 |
|---|---|---:|
| GPU | naga 1,092,115（wgsl front 0.30 M、spv back 0.30 M、hlsl back 0.19 M、valid 0.17 M、proc 0.17 M）、wgpu_core 485,401、wgpu_hal 328,954（vulkan 0.20 M、dx12 0.17 M）、wgpu 158,388、vello_hybrid 115,336、`arvane_compose::gpu` 約 245,000、`arvane_render::gpu` 約 93,000、gpu_allocator 17,717 | 約 2.5 M（+ .rdata 約 1.0 M、.pdata 0.13 M） |
| SVG | usvg 340,932、tiny_skia 331,503（+ path 38,610）、rustybuzz 171,670、ttf_parser 116,957、resvg 107,307、fontdb 36,373、roxmltree 31,374、svgtypes 25,764 | 約 1.2 M |
| 画像の decode | zune_jpeg 165,614、image_webp 133,198、image 86,285、png 70,548、gif 約 20,800 | 約 0.48 M |
| CPU raster | fearless_simd 287,948（fallback・sse4.2・avx2 の 3 版）、vello_cpu 222,631、vello_common 114,842、glifo 71,119、kurbo 58,828 | 約 0.76 M |
| text | harfrust 331,306、skrifa 378,373（cff 0.14 M、autohint 0.11 M）、read_fonts 200,212、parley 113,167、fontique 49,039、arvane_text 79,996 | 約 1.15 M |
| layout | taffy 195,039（grid だけで約 0.20 M。`resolve_intrinsic_track_sizes` の 1 関数が 75,091 B）、kivact_layout 91,103 | 約 0.29 M |
| Kivact | kivact_engine 1,295,015（`model::aggregate` 528 K、`host::pump` 211 K、`host::write` 181 K）、kivact_core 426,264、kivact_runtime 266,614、kivact_style 195,620、arvane_render 175,850、arvane_native 125,836、kivact_types 119,374、kivact_ui 115,662 | 約 2.8 M |
| 窓・a11y | winit 96,680、accesskit 約 0.11 M（consumer 69,084、windows 36,791）、windows 53,309 | 約 0.26 M |

std/alloc の generic（`ptr` 555 K、`slice` 423 K、`fmt` 202 K、`collections` 198 K、`sync` 179 K 等）は上の crate の型の instantiation で、上の各系統に比例して付く。

### 1.3 K2：counter idle WS の内訳（kv_startup、800×600、倍率 1.0、既定 renderer＝CPU で GPU は起動しない）

3 回の WS：34,904 / 34,892 / 34,860 KiB（B1 の 34,912 と一致。この機械で B1 を再現できる）。

| 分類 | KiB | 内容 |
|---|---:|---|
| image（共有可能） | 19,628 | **exe 4,272**（.text 3,824、.rdata 444、他 8）、system DLL 15,356（ntdll 1,976、combase 2,008、KernelBase 1,008、msctf 756、DWrite 728、UIAutomationCore 664、shell32 660、rpcrt4 652、ucrtbase 568、user32 556、twinapi.appcore 540、ExplorerFrame 416 …） |
| mapped file | 3,704 | **segoeui.ttf 940、seguisb.ttf 952（どちらも file 全体＝全 page が WS）**、~FontCache-FontFace.dat 364、pagefile 由来の section 1,236、SortDefault.nls 92、locale.nls 76 |
| private | 11,544 | **1,876 × 4 個（＝800×600×4 B の frame buffer 4 枚、7,504）**、2,044（heap segment）、948（font の複製）、564、stack・TEB 等 約 600 |

- 4 枚の frame buffer（1,920,000 B ずつ、`VirtualAlloc` された 1,876 KiB の領域）の正体（コードから確定）：
  1. `CpuCompositor.output`（`crates/arvane-compose/src/cpu/compositor.rs:117`、`Pixmap::new(size)` 159 行）
  2. root layer の chunk（`Stores.chunks`、`raster_chunk` が 297 行で `Pixmap::new(w, h)`。layer 1 個＝chunk 1 個＝window 全体）
  3. `CpuRaster.pixmap`（`crates/arvane-render/src/cpu.rs:33`。vello_cpu の描画先。129 行で window 大に resize され、そのまま保持される。131-136 行で chunk へ全体を copy するだけの中間 buffer）
  4. softbuffer の DIB section（`softbuffer-0.4.8/src/backends/win32.rs:91` `CreateDIBSection`。window に出す最終 buffer。必須）
- font の複製：`crates/arvane-text/src/output.rs:472-474` の `Arc::from(run.font().data.as_ref())` が、system font の file 全体（fontique が mmap したもの）を heap へ copy する。counter は Segoe UI（text）と Segoe UI Semibold（button）を使うので 2 個（960,076 B + 974,760 B）。copy の読出しで mmap 側の全 page も WS に入る（上の mapped の 940 + 952 KiB）。
- 比較のための床：winit 0.30.13 + softbuffer 0.4.8 だけで 800×600 の白い window を 1 枚出す最小 app（同じ profile、source は scratchpad の `p4-tools/mini`）は WS 17,868 KiB（private 2,972、image 14,132）。winit の `with_drag_and_drop(false)` にすると 15,976 KiB（OLE の drop target の登録で system DLL が約 1.9 MB 増える）。
- exe の resident page（4,272 KiB）は Kivact 自身が起動時に実行する code の page（resident な code byte：kivact_engine 693 K、kivact_core 240 K、kivact_runtime 180 K、kivact_style 163 K、harfrust 156 K …）。GPU の code は 1 page も resident でない。binary を小さくしても WS はほとんど減らない：gpu なしの exe（−4.57 MB）で WS −370 KiB（34,508）、全 crate `s`（−0.86 MB）で −190 KiB（34,716）。
- 計測上の注意：build 直後の exe を初めて起動した回だけ、exe の image が file 全体分（16,568 KiB）WS に入る（2 回目以降は 4,272）。`run.sh` は K2 の前に K1 で 5 回起動するので影響しないが、単独で測るときは 1 回捨てること。

### 1.4 試作による WS の実測（worktree に一時的に入れた変更、§3 の D1〜D3 の要点だけ）

worktree に D1（`CpuRaster` が `out` へ直接描く）、D2 の idle 後の状態（試作では各 present の直後に `self.cpu.invalidate()` を呼ぶ＝「解放済み」の状態を常に作る）、D3（`GlyphRun.font` を mmap と共有する `FontSource`）の要点だけを入れた `kv_startup`（default、gpu + complex-scripts）を build し、同じ方法で測った（試作の patch：scratchpad の `p4-tools/proto.py`。build 一覧 `p4-tools/sizes.txt`、計測 script `wsbreak.ps1`・`p4-tools/ws.sh` も同じ所。`W:\w\kv-size-target` は削除済み）。

| | WS | image | mapped | private | 主な変化 |
|---|---:|---:|---:|---:|---|
| B1（3 回） | 34,860〜34,904 | 19,628 | 3,704 | 11,544 | |
| 試作 default（3 回） | **25,844〜25,872** | 19,696 | 2,152 | 3,984 | window 大 buffer 4→1 枚（DIB だけ）、font の mapped 940/952→164/164 KiB、heap 2,044+948+564→956+648 |
| 試作 `ARVANE_RENDERER=cpu`（2 回） | **25,856〜25,860** | 19,696 | 2,140 | 3,990 | |

差の内訳：window 大 buffer 3 枚 −5,628 KiB（D1 で 1 枚、D2 で 2 枚）、font の複製の解消 private −1,952 KiB、font file の未使用 page −1,560 KiB（D3）。合計 −9,030 KiB。目標（default ≤ 27,214、cpu ≤ 26,694）を両方満たす（余裕 1,342 / 838 KiB）。

---

## 2 原因

### 2.1 K6

1. **GPU 一式が既定 feature に入っている（4.57 MB）**。設計 1.2 節の「`gpu` なしで 13.7 MB（GPU 一式は約 0.9 MB）」は B1 のコードでは再現しない（実測 4.57 MB）。OnDemand 起動でも code は全部 link される（GPU を選ぶ経路は実行時の分岐で、LTO は消せない）。naga 単独で 1.09 MB の code があり、wgpu が使う 3 つの言語変換（WGSL の読込み、SPIR-V と HLSL の書出し）と検証が入る。
2. **K6 の目標の算出根拠と計測構成の不一致**。設計 1.4 節は kv_startup no-default を「2.3×（約 11 MB）、R-11 の complex-scripts」と見積もったが、その 11 MB は 1.2 節の「gpu なし、complex-scripts なし 10.6 MB」の行に由来する。一方、計測する `kv_startup --no-default-features` は bench の Cargo.toml が `kivact` に `features = ["gpu"]` を固定しているので **GPU 込み**（15.92 MB）。目標の構成（GPU なし）と計測の構成（GPU あり）が食い違っている。同じ構成なら B1 のコードで 11,352,576 B（目標 11,177,658 B との差 174,918 B）。
3. **hot crate の opt-level 3 は 0.86 MB**（全部 `s` との差）。設計の「opt 3 で 14 MB 前後」の見積りに対し、実費は小さい。主因ではない。
4. **strip は効いていない／効かせる余地がない**：MSVC の exe は symbol table を持たず、`strip="symbols"` は PDB を作らないだけ（line-tables 付きでも exe は同じ大きさ）。
5. **catalog は app の asset と kit の code が大きい**：font 10.64 MB（うち NotoSansJP 9.59 MB）と、kit＋catalog の code（.text +9.40 MB）。framework 側だけの削減では 31.9 MB に届かない（§4）。
6. **「旧分析の 14.5 MB 見込み」と B1 の 18.98 MB の差は、見込みの元の数字の誤り**（§1.1.1：B0 のコードに同じ profile を当てても 19.01 MB、全 `s` でも 18.14 MB）。新コード・feature・strip のどれも差の原因ではない。K6 の 3 つの必達値（B0÷1.7、÷2.2、÷1.7）は B0 の「profile なし」の値から算出され、「profile を足せば 14.5 MB」という誤った見込みで達成可能と判断された。profile の効果は B0 比で counter 1.30×（24.59→18.98）が実力で、残りは feature 構成（GPU 4.57 MB）か app の asset でしか埋まらない。

### 2.2 K2（counter の WS 基礎分）

1. **window 大の buffer が 4 枚**（7.33 MiB）。うち `CpuRaster.pixmap` は描画の中間で、chunk へ copy するためだけに window 大で常駐する（不要）。chunk と output は部分 raster・部分合成のために frame 間で保持されるが、idle の window でも保持され続ける。
2. **system font の file 全体の複製**（約 1.85 MiB の private）＋その読出しで mmap 全 page が resident（約 1.85 MiB の mapped）。登録 font（`App::fonts`）ではさらに複製が重なる（§7 既存欠陥 E-1）。
3. exe の resident page（4.2 MiB）と system DLL（15.0 MiB）は binary の縮小ではほとんど減らない（§1.3）。system DLL のうち OLE drop target 分（約 1.9 MiB）は winit の既定（`drag_and_drop = true`）による。

---

## 3 設計（確定版）

D1〜D3 は K2（WS）、D4〜D8 は K6。file:line は db8d679b（＝73c36c7c）。

### D1 CpuRaster は描画先へ直接 raster する（window 大の中間 buffer を持たない）

対象：`crates/arvane-render/src/cpu.rs`

- `CpuRaster` から field `pixmap: Pixmap`（33 行、Default の 50 行）を削除し、代わりに部分 raster 用の `scratch: Vec<u8>` を持つ（空で始める）。
- `raster()`（70-140 行）の 129-136 行を次に置き換える：

```rust
// 全体：vello_cpu は `out` へ直接描く（Replace で全 pixel を書く。clear 不要）。
if region.width == frame_width && region.height == frame_height {
    let target = vello_cpu::PixmapMut::new(width, height, out)
        .ok_or(RenderError::InvalidSize)?; // 長さは 84 行で検査済み。None は不変条件違反
    self.context.render(target, &mut self.resources);
    return Ok(());
}
// 部分：region の大きさの scratch に描いて行ごとに copy する。scratch は region 以上に
// 育たない（全体 raster は scratch を使わない）。
let len = usize::from(width) * usize::from(height) * 4;
self.scratch.resize(len, 0); // 長さだけ合わせる。内容は Replace が全部書く
let target = vello_cpu::PixmapMut::new(width, height, &mut self.scratch[..len])
    .ok_or(RenderError::InvalidSize)?;
self.context.render(target, &mut self.resources);
let row_bytes = usize::from(width) * 4;
for (y, row) in self.scratch[..len].chunks_exact(row_bytes).enumerate() {
    let start = ((region.y as usize + y) * frame_width as usize + region.x as usize) * 4;
    out[start..start + row_bytes].copy_from_slice(row);
}
Ok(())
```

- 新 method `pub fn forget_scratch(&mut self) { self.scratch = Vec::new(); }`（D2 から呼ぶ）。
- 不変条件：`render` の target が `RenderContext` の大きさ（`reset_and_resize(width, height)`）と同じなので `target_fully_covered` が真になり、vello_cpu は clear せずに全 pixel を packing で書く（`vello_cpu-0.2.0/src/render.rs:806-813`）。旧実装の「新しい pixmap に描いて copy」と出力の byte は同一。
- import：`vello_cpu::Pixmap` は不要になる（`PixmapMut` を使う）。`derived.rs` の `Pixmap` は別用途で変更なし。
- 効果：常駐の window 大 buffer が 1 枚減る（800×600 で −1,875 KiB）。全体 raster ごとの window 大 copy（1.9 MB）も消える（K4 の初回 raster が僅かに速くなる。悪化はない）。部分 raster の scratch は最大 damage 矩形の大きさ（kv_stress の 3000 行で約 8,000 px＝32 KB）。

### D2 idle の window は CPU の frame state を手放す（1 s）

**方針**：最後の present から `FRAME_STATE_KEEP_NS`（1 s）新しい present が無い window は、CPU compositor の frame 間 state（root を含む全 chunk の pixel、保持 layer surface、output、D1 の scratch）を解放する。次の frame は全体を raster する（cache が空なので `Damage::Full`）。window に出ている絵は softbuffer の DIB が保持するので見た目は変わらない。

**根拠**：desktop の UI はほとんどの時間 idle で、idle の window が window 大 buffer を 2〜3 枚（800×600 で 3.7〜5.5 MiB、1440×900 で 10〜15 MiB）抱える理由がない。戻す費用は全体 raster 1 回で、K4 の初回 frame raster（800×600、B1 3.3 ms）と同じ量。連続更新・hover 中・animation 中は present が 1 s 以内に続くので解放されない（K4 は影響を受けない）。

**状態の表し方**（暗黙の状態を作らない）：host の既存 enum `LastPresent`（`crates/kivact-engine/src/host/window.rs:12`）に variant を足す。frame state を持つかどうかは `LastPresent` からの導出で、別 field を持たない。

```rust
pub(super) enum LastPresent {
    /// Shown at `at_ns`: Arvane keeps the window's frame state for the
    /// next partial frame until `at_ns + FRAME_STATE_KEEP_NS`.
    Shown { at_ns: u64 },
    /// Nothing to keep or retry: dropped, lost, retried, or released after idle.
    Settled,
    /// (既存のまま) Skipped { retry_at_ns: u64 },
    Skipped { retry_at_ns: u64 },
}
```

`crates/kivact-engine/src/host/window.rs` に `pub(super) const FRAME_STATE_KEEP_NS: u64 = 1_000_000_000;`。

書き手と変更（全部）：

| 場所 | 変更 |
|---|---|
| `host/events.rs:252-260`（`redraw` の `Presented::Shown`） | `last_present = LastPresent::Shown { at_ns: windows.now_ns() }`（`Settled` から変更） |
| `host/events.rs:276-283`（Dropped）、`287-300`（SurfaceLost）、`315-321`（retry）、`397-403`（Focused） | 変更なし（`Settled`／既存のまま） |
| `host/pump.rs:678-683` `retry_at` | `LastPresent::Shown { .. } \| LastPresent::Settled => None` |
| `host/pump.rs` 新 fn `release_at(&self, id) -> Option<u64>` | `Shown { at_ns } => Some(at_ns.saturating_add(FRAME_STATE_KEEP_NS))`、他は `None` |
| `host/pump.rs:771-810` `control` | `next` の候補配列に `self.windows.keys().filter_map(\|id\| self.release_at(*id)).min()` を足す（events・effects がある時の `Some(now_ns)` 分岐はそのまま） |
| `host/events.rs:310` `wake` の先頭（retry 処理の直後） | 各 window について `release_at(id) <= now` かつ `!self.paint_due(id)` なら `windows.get(id)` の `forget_frame_state()` を呼び、`last_present = LastPresent::Settled`。`paint_due` なら何もしない（その redraw が `Shown` を書き直す） |
| `crates/arvane/src/window.rs:91` 付近 | `pub fn forget_frame_state(&mut self) { self.presenter.forget_frame_state(); }` |
| `crates/arvane/src/presenter/frame_policy.rs:70` 付近 | `pub(crate) fn forget_frame_state(&mut self) { self.cpu.invalidate(); }` |
| `crates/arvane-compose/src/cpu/compositor.rs:144` `invalidate` | 既存の 3 行に `self.raster.forget_scratch();` を足す（D1）。doc を「after a resize, a lost buffer, or when the window has been idle」に |

- `invalidate` は既に「次の frame は全部描き直す」ための唯一の入口（`frame_policy.rs:263` と `compositor.rs:158,166`）で、新しい経路を作らない。GPU の state は対象外（GPU の memory は WS の基礎分に入らず、GPU を使う window は GPU の都合で保持する）。
- 境界条件：
  - window close：host の window state ごと消えるので何もしない。
  - 最小化・`Surface::Unavailable`・`AppPresence::Suspended`：`drawable` でなくても `release_at` は効く（見えない window ほど手放してよい）。`wake` は `windows.get(id)` が `Some` の時だけ呼ぶ。
  - reduced motion・animation・frame loop：frame が続く間は `Shown { at_ns }` が毎 frame 更新されるので解放されない。
  - DevTools の overlay・frame inspector：解放後の最初の frame の `FrameStats` は全体 raster（`chunks_rastered`・`pixels_rastered` が大きい）。DevTools の表示はそのまま正しい。
  - embed（`port/win32` の `repaint`）：DIB から再表示するので影響なし。
  - 試験 driver（仮想時間）：`control` の deadline として現れるので `advance` が 1 s を越えるとその時刻で `wake` が走り、client の turn が 1 回入る（既存の「due な wake は events が無くても client に turn を与える」と同じ）。frame は出ない。
  - K3：解放は 1 回の wake だけで frame を出さない。最後の frame の 1 s 後で、K3 の 10 s の標本区間（3 s 後から）の外。
- 効果（kv_startup）：idle 時の window 大 buffer は DIB 1 枚になる（output と root chunk の 2 枚、−3,750 KiB）。D1 と合わせて −5,625 KiB。

### D3 font の bytes を複製しない（`FontSource`）

`kivact-types` に font file の bytes を「出所と共有する」型を足し、glyph run・登録 font・SVG の font database が同じ allocation／mapping を共有する。

`crates/kivact-types/src/render/shaped.rs`（`GlyphRun` の前）：

```rust
/// A font file's bytes, shared with where they came from (a system font's
/// mapping, embedded bytes, or bytes read at run time); never copied.
#[derive(Clone)]
pub struct FontSource(Arc<dyn AsRef<[u8]> + Send + Sync>);

impl FontSource {
    /// Share `source` (e.g. a text system's mapped font file).
    pub fn new(source: Arc<dyn AsRef<[u8]> + Send + Sync>) -> Self { Self(source) }
    /// Embedded bytes, without copying them.
    pub fn from_static(bytes: &'static [u8]) -> Self { Self(Arc::new(bytes)) }
    /// Bytes already shared, without copying them.
    pub fn from_shared(bytes: Arc<[u8]>) -> Self { Self(Arc::new(bytes)) }
    /// The shared source, for a backend that keeps it (vello, fontdb).
    pub fn source(&self) -> &Arc<dyn AsRef<[u8]> + Send + Sync> { &self.0 }
    /// The address of the source: its identity while it lives (cache key).
    pub fn id(&self) -> usize { Arc::as_ptr(&self.0).cast::<u8>() as usize }
}
impl std::ops::Deref for FontSource { type Target = [u8]; fn deref(&self) -> &[u8] { (*self.0).as_ref() } }
impl PartialEq for FontSource {
    // 同じ出所なら内容を比べない（旧 `Arc<[u8]>` の Eq と同じ意味）
    fn eq(&self, other: &Self) -> bool { self.id() == other.id() || **self == **other }
}
impl fmt::Debug for FontSource { /* "FontSource(<len> bytes)" */ }
```

`render.rs:28-29` の `pub use shaped::{...}` に `FontSource` を足す。`GlyphRun.font`（`shaped.rs:30`）の型を `crate::AssetBytes` → `FontSource` に変える（doc：「The font file, shared with its source」）。

変更する全箇所：

| 場所 | 変更 |
|---|---|
| `crates/arvane-text/src/output.rs:466-477` `glyph_run` | 引数 `fonts: &mut HashMap<..>` を削除。`font: FontSource::new(run.font().data.clone().into_raw_parts().0)`（`linebender_resource_handle::Blob::into_raw_parts`、copy なし。fontique の mmap／登録 bytes の `Arc` を共有） |
| 同 `output.rs:67` `Ellipsis::of`、`383-384` `draw` | 引数 `fonts` を削除し、呼出し先へ渡すのをやめる |
| `crates/arvane-text/src/text_engine.rs:24,53,146,307,353`、`inline.rs:70` | field `font_bytes` とその初期化・`release_caches` での初期化・引数渡しを削除。`release_caches` の doc から「copied font bytes」を外す |
| `crates/arvane-text/src/text_engine.rs:88` `register_font(bytes: Arc<[u8]>)` | `register_font(&mut self, font: FontSource)`；`Blob::new(Arc::clone(font.source()))` |
| `text_engine.rs:41` `isolated(fonts: &[Arc<[u8]>])` | 引数は変えない（試験の呼出し元 5 箇所を変えない）。内部で `Blob::new(Arc::new(bytes.clone()))` のまま（copy なし） |
| `crates/arvane/src/engine.rs:61` `Engine::register_font(bytes: Arc<[u8]>)` | `register_font(&mut self, font: FontSource)` |
| `crates/arvane-asset/src/font.rs:44` `FontBytes::to_shared` | 削除し `pub fn to_source(&self) -> FontSource`（`Static(b) => FontSource::from_static(b)`、`Shared(b) => FontSource::from_shared(Arc::clone(b))`）。**Static の copy が消える** |
| `crates/arvane-asset/src/store/asset_store.rs:173` `register_font(bytes: Arc<[u8]>)` | `register_font(&mut self, font: FontSource)`；`self.fonts.with_font(font)` |
| `crates/arvane-asset/src/svg/fonts.rs:11,26-33,48-50` | `app: Vec<FontSource>`；`with_font(&self, font: FontSource)`；db 構築は `fonts.load_font_source(usvg::fontdb::Source::Binary(Arc::clone(font.source())))`（`fontdb-0.24.0/src/lib.rs:205,868`。**`to_vec` と `clone` の 2 回の copy が消える**） |
| `crates/kivact-engine/src/lower/assets.rs:255-265` | `let font = file.bytes().to_source(); engine.register_font(font.clone()) ...; self.store.register_font(font);` |
| `crates/arvane-render/src/draw/painter.rs:56-68` `FontCache` | key を `(font.id(), index)`、値 `FontData::new(Blob::new(Arc::clone(font.source())), index)`。`get(&mut self, font: &FontSource, index)`。694・725 行の呼出しは `self.fonts.get(&run.font, run.font_index)` |
| `crates/arvane-render/src/validate/fonts.rs` | `Entry.bytes: Weak<dyn AsRef<[u8]> + Send + Sync>`；`glyphs(&mut self, font: &FontSource, index)`；一致判定は `entry.bytes.as_ptr().cast::<u8>() as usize == font.id() && entry.len == font.len()`；`let face = skrifa::FontRef::from_index(font, index)`；登録は `Arc::downgrade(font.source())` |
| `crates/arvane-render/src/validate/scene.rs:367` | `fonts.glyphs(&run.font, run.font_index)?` |
| `crates/arvane-render/src/ink.rs:20`、`outline.rs:23` | `FontRef::from_index(&run.font, ..)` → `FontRef::from_index(&*run.font, ..)` |

- 不変条件：`FontSource` の clone は `Arc` の clone だけ。`id()` は source が生きている間一意（cache は `FontData`／`Weak` で source を保持・照合するので、旧 `Arc<[u8]>` の address key と同じ保証）。
- 寿命：glyph run が source（fontique の mmap）を保持するので、`TextEngine::release_caches` の `source_cache.prune` 後も run は有効（旧 doc の「Returned glyph runs own their font copies independently」と同じ保証を、copy でなく共有で満たす）。file の mapping は最後の run が消えるまで残る（WS には触れた page だけが入る）。
- 効果：counter で private −1,860 KiB（2 個の複製）＋ mapped の未使用 page（実測は §1.4）。`App::fonts` で font を登録する app（catalog 等）では、登録 1 font につき最大 4 回の複製（Static→Arc、SVG 用 `to_vec`、fontdb 構築の `clone`、glyph run）が 0 回になる（catalog の NotoSansJP は 9.6 MB × 最大 4）。

### D4 `gpu` を既定 feature から外す（K6 counter・kv_startup に必須）【設計者の判断が要る：§8 Q1】

- `crates/kivact/Cargo.toml`：`default = ["complex-scripts"]`。feature `gpu` の定義はそのまま。
- 既定で GPU を使わない app の挙動：CPU で描く（今の OnDemand の既定 renderer と同じ。違いは「CPU が遅い 8 frame で GPU に切り替える」・GPU export・外部 GPU 画像の import・shader の GPU 実行が無いこと）。shader は既存の `ShaderPaint::fallback`／filter の fallback を CPU で描く（`crates/kivact-style/src/shader/paint_shader.rs:64`）。`import_external_image` は今も `GPU_BUILT` が偽なら「built without GPU support」を返す（`crates/arvane/src/presenter/frame_policy.rs:107`）。
- consumer の feature（`gpu` を既定に入れているもの）：
  - 既定から外す：`templates/basic/Cargo.toml:9`、`examples/kivact-ui-basic/Cargo.toml:10`、`examples/kivact-ui-pop`、`examples/kivact-ui-soft`、`examples/kivact-ui-creative`、`crates/kivact-devtools/Cargo.toml:10`（どれも shader を使わない：§1 の grep）。`gpu = ["kivact/gpu"]` の定義は残す。
  - 既定に残す：`examples/kivact-ui-game`（`src/game/effects.rs` が shader を使う）、`dev/tests/kivact-public-api`・`dev/tests/docs`（GPU の E2E・doctest を通常の test で回す）、`dev/bench`（K2 の GPU 強制行と K1・K4 の GPU 行。D5）。
- 文書：`README.md:179`（表の Default を `no`、説明に「Without it the build draws on the CPU only (about 4.6 MB smaller). Enable it for GPU shaders, GPU export, external GPU images and the automatic switch to the GPU when CPU frames are slow.」）、`README_JP.md` の同じ表、`docs/for-kivact-user/{en,jp}/README.md` の feature の説明、`docs/for-kivact-user/{en,jp}/how-to-guide/gpu-texture-sharing.md`（`gpu-interop` は `gpu` を含むので手順は変わらない。「既定で入る」等の記述があれば直す）、`docs/for-kivact-ui-user/{en,jp}/creative/how-to-guide/show-app-rendered-images.md`（`features = ["gpu"]` を書く）。
- API の統一性：feature 名は変えない（`gpu`、`complex-scripts` のまま）。「既定は最小の CPU 描画、能力は feature で足す」で `embed`・`tokio`・`devtools` と同じ形になる。
- 効果：counter default 14,406,656 B（kv_startup gpu なし の実測と同じ構成）。

### D5 計測定義の修正：`kv_startup --no-default-features` は GPU も外す

- `dev/bench/Cargo.toml`：`[features] default = ["complex-scripts", "gpu"]`、`gpu = ["kivact/gpu"]`、依存 `kivact = { path = "../../crates/kivact", default-features = false }`（`features = ["gpu"]` を外す）。
- 理由：K6 の目標（約 11 MB）は設計 1.2 節の「gpu なし、complex-scripts なし」構成から算出された（§2.1-2）。D4 の後は「`--no-default-features` ＝ facade の最小構成」で、目標の算出根拠と計測構成が一致する。`run.sh` の他の build（`kv_startup`・`kv_stress`・`kv_latency`・`kv_heap`）は bench の既定 feature（gpu を含む）で今と同じ。
- 注意：目標値（11,177,658 B）は変えない。構成を目標の根拠に合わせる修正なので、設計者の承認を得る（§8 Q2）。

### D6 実行時の generic の重複を減らす（kivact-runtime、余裕の確保）

catalog の code の最大は kivact_runtime 3.20 MB（counter 0.27 MB）。型・closure ごとの instantiation が大きい：

| 関数（generic を除いた名前） | instantiation | code |
|---|---:|---:|
| `kivact_runtime::machine::MachineHandle::send` | 78 | 779,613 B |
| `kivact_runtime::reactive::data::Value::with_impl` | 553 | 717,380 B |
| `kivact_core::view::flow::list::for_each::{closure}` | 21 | 315,301 B |
| `kivact_runtime::reactive::setup::memo::{closure}` | 558 | 227,235 B |
| `async_task::raw::RawTask::run` ＋ `spawn_local` の poll | 81 ＋ 76 | 236,788 B |
| `kivact_runtime::reactive::data::Signal::with_impl` | 164 | 110,447 B |

確定する変更（意味を変えない手作業の「非 generic 部分の切出し」）：

1. `Value::with_impl`（`crates/kivact-runtime/src/reactive/handles.rs:260-293`）：(T, R, F) ごとに `runtime()`・`read_or_panic`・downcast・memo の再計算が inline される。非 generic の `fn resolve_value(app, repr, track, at) -> Resolved`（`enum Resolved { Const(Rc<dyn Any>), Cell(Rc<dyn Any>) }`、`#[inline(never)]`）に切り出し、generic 側は downcast（T ごと）と `f` の呼出しだけにする。memo の値の取出しは `rt.memo_value::<T, R>(&cell, f, at)` を `rt.memo_cell::<T>(&cell, at) -> Rc<MemoCell<T>>`（T だけの generic）＋呼出し側での borrow と `f` に分ける。`Signal::with_impl`（30-51 行）も同じ形（`resolve` は `read_or_panic` まで）。
2. `MachineHandle::send`（`crates/kivact-runtime/src/machine.rs:166-226`）：timer の `async move { wait.await; handle.send(event) }` が M ごとに future 型（async_task の instantiation）を作る。timer を非 generic の `spawn_timer(owner, delay, fire: Box<dyn FnOnce()>, at) -> Task` に切り出し、`fire = Box::new(move || handle.send(event))` を渡す（future 型は 1 種類になる）。transition と effect 実行は M 固有の code なので残す。
3. `for_each` の closure（`crates/kivact-core/src/view/flow/list.rs:118-181`、`apply_list` 184-273 が inline される）：item 型に依存しない「消える行の除去と owner の破棄（手順 2）」と「新しい行・動いた行の配置（手順 4 の後半、`insert`／`move_before`）」を、`NodeId`・`Owner` の slice を受ける非 generic の `fn dispose_rows(rt, &[(NodeId, Owner)])` と `fn place_rows(rt, frag, nodes: &[NodeId], fresh: &[bool], moved: &[bool])`（`#[inline(never)]`）に切り出す。generic に残すのは item の更新・行の mount・`Row<K, T>` の組立てだけ。

見積り：1 は 553×(1.30→約 0.25 KB)＋型ごと 約 60×0.5 KB で −0.55 MB、2 は −0.2 MB、3 は 21×(15→約 6 KB) で約 −0.19 MB。catalog で約 −1.0 MB、counter で約 −0.05 MB（counter は instantiation が少ない）。call が 1 段増えるだけで K4/K5 への影響は測定誤差以下（読み出しは数十 ns）。D6 は D4・D5 で counter・kv_startup の目標を満たした後の余裕（後続の V2 の変更でコードが増えても目標を割らないため）と、catalog の不足分を埋める一部。

### D7 profile（確定）：hot crate は 3 のまま、Kivact の cold crate は `s` のまま、第三者の「起動・描画の毎 frame に通らない」crate だけ `z`

計測（kv_startup、byte）：

| 構成 | gpu+cs | gpu・cs なし |
|---|---:|---:|
| B1 profile | 18,976,768 | 11,352,576 |
| 全 cold crate `z`（hot は 3） | 17,620,480 | 10,478,080 |
| 下の一覧だけ `z` | 18,221,056（−755,712。計測時は wgpu・wgpu-core・wgpu-hal・wgpu-types も `z`） | **11,062,784**（−289,792。GPU の crate を含まない構成なので確定一覧と同じ） |

全 cold crate `z` の K5 への影響（`kv_latency`、同じ機械で base→z を交互に 2 回。他の build と並行で絶対値は B1 より大きいが比は比べられる）：wheel n1000 7.9/7.7 → 9.8/9.3 ms（+20 %）、mount n3000 1,779/2,059 → 2,109/2,250 ms（+9〜18 %）、pointer p95 n1000 37.6/31.8 → 40.6/35.4 ms（+8〜11 %）、unrelated n3000 68.3/78.9 → 77.2/82.0 ms（+4〜13 %）、counter click 2.47/2.39 → 2.41/2.26 ms（差なし）。**Kivact の crate（kivact-core・kivact-runtime・kivact-style・kivact-ui・kivact-interaction 等）を `z` にすると K5 が悪化するので採らない。**

確定する profile（root `Cargo.toml` の `[profile.release]` の後に追加。`templates/basic/Cargo.toml` の同じ節にも同じものを書く）：

```toml
# 起動と毎 frame の経路に無い第三者の crate は大きさを優先する（SVG の解析と
# 描画、画像の decode、accessibility の adapter、file dialog・clipboard、
# 日時・数の書式、GPU の shader 変換）。描画・shaping・layout の hot path（上）
# と Kivact の crate はこれまでどおり。
[profile.release.package.usvg]
opt-level = "z"
# 以下同じ形で：resvg、tiny-skia、tiny-skia-path、rustybuzz、ttf-parser、fontdb、
# roxmltree、svgtypes、simplecss、image、zune-jpeg、zune-core、image-webp、png、gif、
# fdeflate、miniz_oxide、flate2、accesskit、accesskit_consumer、accesskit_windows、
# accesskit_winit、rfd、arboard、naga、
# icu_datetime、icu_calendar、icu_decimal、icu_plurals、icu_collator、icu_time、
# icu_locale_core、kivact-i18n
```

- 性能への影響：これらは asset の worker thread（SVG・画像の decode。結果は cache）、a11y の公開（AccessKit の adapter。K5 の試験 driver は AccessKit を通らない）、GPU の pipeline 作成時の shader 変換（naga。GPU 起動時の 1 回）、i18n の書式で、K1（CPU 起動）・K4・K5 の測定経路に入らない。wgpu・wgpu-core・wgpu-hal は GPU で描く毎 frame の command 作成に入るので一覧に入れない（GPU を有効にした app の frame 時間を悪化させない）。SVG・画像の初回 decode は遅くなりうる（`z` は loop の展開と inline を減らす。最大 1.5 倍程度と見込む。未計測）。
- `opt-level="z"` は Kivact の reactive・view・style・engine の crate には使わない（上の K5 の実測）。kivact-i18n は日時・数・複数形の書式だけで毎 frame の経路に無いので一覧に入れる。hot crate の 3 は維持（−0.86 MB の代わりに K4/K5 を悪化させる理由がない）。

### D7 の補足：採らない案（計測した上で）

| 案 | 効果 | 採らない理由 |
|---|---:|---|
| `panic = "abort"` | 未計測 | embed の `catch_unwind`（`crates/arvane-native/src/port/win32/proc.rs:45`）が使えず DAW を落とす（設計 4.8）。決定どおり |
| wgpu の `dx12` を外す（Windows は Vulkan だけ） | −0.57 MB（GPU 込み） | Vulkan の無い Windows（driver の無い iGPU、VM、RDP、WARP）で GPU を使えなくなる。D4 の後は既定 build に GPU が無いので K6 に効かない |
| wgpu の `vulkan` を外す（Windows は DX12 だけ） | −0.87 MB（GPU 込み） | 旧分析で DX12 は初期化 1.7 s・WS 287 MiB と Vulkan より悪い（K2 の GPU 強制行が悪化）。Linux は Vulkan が要る |
| winit の `with_drag_and_drop(false)` | WS −1.9 MiB（最小 app の実測） | `on_file`／`dropzone`（`crates/kivact-core/src/view/node_modifiers.rs:225`、`crates/kivact-ui/src/dropzone.rs:62`）の hover 位置付き drop が使えなくなる。必要時だけ OLE の drop target を登録するには winit の外で `IDropTarget` を実装する必要があり（unsafe の COM）、規模に見合わない。K2 は D1〜D3 で届く |
| SVG の描画を resvg/tiny-skia から Arvane の painter へ、SVG の文字を usvg の text から parley へ移す | 約 −1.2 MB（tiny-skia・resvg・rustybuzz・ttf-parser・fontdb） | SVG の filter・mask・text の描画を作り直す大工事で、既存の SVG E2E の期待値を変えるおそれがある。K6 は D4〜D8 で届く。将来の課題として記録 |
| 画像の format（jpeg・webp・gif）を feature にする | 約 −0.45 MB | 公開 API（`Asset` が受ける形式）の縮小 |
| 既定の `complex-scripts` を外す | −3.05 MB | Q-07・SLIM-06 の決定で既定に残す |


### D8 basic catalog の埋込み font【設計者の判断が要る：§8 Q3】

catalog（`examples/kivact-ui-basic/src/fonts.rs:5-10`）は NotoSansJP[wght].ttf（9,589,900 B、約 17,000 glyph の可変 font）を丸ごと埋め込む（B0 も同じ）。catalog・kit・試験の source が使う U+2000 以上の文字は 884 種（`examples/kivact-ui-*`、`crates/kivact-ui-*`、`dev/tests/kivact-public-api` の .rs/.ftl/.json/.toml/.md の和集合）。framework 側の D4・D6・D7 だけでは catalog は約 37 MB で 31.9 MB に届かない（§4）。

確定案（推奨）：**subset した font を asset として commit し、catalog・各 kit catalog・試験はそれを埋め込む**。

- 生成：`dev/tools/subset-catalog-font.py`（新規、Python + fontTools。CI には足さない。手で実行して結果を commit）。入力は元の `NotoSansJP[wght].ttf`。文字集合は「上と同じ走査で集めた文字 ＋ U+0020-007E ＋ U+3000-30FF（記号・ひらがな・カタカナ全部）＋ U+FF00-FFEF」。`--layout-features='*'`、`fvar`/`gvar`/`STAT`/`hhea`/`OS/2` を保つ（wght 軸と metrics を変えない）。出力 `examples/shared-assets/fonts/notosansjp/NotoSansJP-subset[wght].ttf`、同じ場所の `SOURCE.md` に生成手順と文字集合の出所を書く（SIL OFL は subset を許す。Noto は Reserved Font Name を持たない）。
- 参照の置換：`examples/kivact-ui-{basic,pop,game,creative}/src/fonts.rs` の `include_bytes!` と `dev/tests/kivact-public-api/tests/e2e/layout_check.rs:17` を subset に変える。元の 9.6 MB の file は使う者がいなくなるので消す。
- 漏れの検出（「文字を足したら script を再実行する」という暗黙の約束を試験で表す）：`examples/kivact-ui-basic` の test に 1 本足す。catalog の全 page の `source`（`docs::PAGES` が `include_str!` で持つ）を含む表示文字列の全文字が `fonts()` のどれかの cmap にあることを、`kivact::test::render` で各文字を `text` に置いて missing glyph（`.notdef`）が描かれないことで確かめる。
- 見積り：可変 font の subset は glyph 数にほぼ比例する（約 1,200 / 17,000 glyph と共通の table）。**0.6〜1.2 MB、差 −8.4〜−9.0 MB**。fontTools がこの機械に無く、取得（pip）は許可の範囲外なので未計測。
- 代替（不採用）：NotoSansJP を埋め込まず system の日本語 font に任せる（−9.59 MB）。catalog の見た目が OS で変わり、`fonts()` を使う決定的な試験に別の読込み経路が要る。

---

## 4 見積り（目標との対比）

### 4.1 K6（byte。「実測」以外は §1 の実測の差分からの見積り）

| 行 | 目標 | B1 | D4 | ＋D5 | ＋D7 | ＋D6 | ＋D8 | 判定 |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| counter default | 14,467,313 | 18,978,816 | 14,406,656（kv_startup 同構成の実測） | 同 | 約 14,120,000（none の selz 差 −290 K を当てた見積り） | 約 14,070,000 | — | ✓（余裕 約 0.40 MB） |
| kv_startup no-default | 11,177,658 | 15,922,688 | 15,922,688 | 11,352,576（実測） | **11,062,784（実測）** | 約 11,010,000 | — | ✓（余裕 約 0.17 MB） |
| basic catalog default | 31,912,659 | 43,090,432 | 約 38,520,000 | 同 | 約 38,200,000 | 約 37,200,000 | **約 28,200,000〜28,800,000** | D8 ありで ✓、なしで ✗（約 5.3 MB 不足） |

- 余裕が小さいので、V2 の他の担当がコードを増やす場合は、checkpoint で K6 を測って確かめる。D6 は余裕の確保のための unit。
- GPU を既定に残す場合（Q1 を否とする場合）の最良：counter 約 18.98 −0.86（全 `s`）−0.76（D7、GPU 込みの実測）−0.57（dx12 なし）−約 0.05（D6）≈ 16.8 MB（重なりがあるので実際はこれ以上）で未達。

### 4.2 K2 counter idle WS の基礎分（KiB）

| | 目標 | B1 | D1+D2+D3（試作の実測） | 判定 |
|---|---:|---:|---:|---|
| default | ≤ 27,214 | 34,912（この機械 34,880） | **25,844〜25,872** | ✓（余裕 約 1,350） |
| cpu | ≤ 26,694 | 34,968 | **25,856〜25,860** | ✓（余裕 約 835） |

- 0 item の `kv_stress`（同じ 800×600、B1 35,796 KiB）も同じ基礎分を持つので約 −9 MiB になる見込み。3000 行の idle WS の目標（≤ 129,989 KiB、B1 292,776）は item あたりの削減（別担当）が主で、基礎分の削減は約 3 % 分。
- D4 は WS をほとんど変えない（§1.3：exe の resident page に GPU の code は無い）。
- 最終判定はこの機械で B0 を取り直す。B0 の default は GPU を常に起動したので 136 MB 前後と見込まれ、÷5 で約 27 MB。cpu は B0 37,372 KiB（÷1.4 = 26,694）がこの機械でも同程度なら、余裕 約 0.8 MiB で満たす。

---

## 5 正しさ・リスク・E2E

### 5.1 D1
- 出力の同一性：全体 raster は vello_cpu の `Replace` が target 全体を書く（clear 省略は `target_fully_covered` の時だけ）。部分 raster は旧実装と同じ「region の大きさに描いて行 copy」。既存の `dev/tests/kivact-public-api/tests/e2e/compositor.rs` の 6 本（部分と全体の pixel 一致）と `crates/arvane/tests/e2e/present.rs` で担保。
- `PixmapMut::new` は長さ不一致で `None`。`raster` の冒頭で `out.len()` を検査済み。

### 5.2 D2
- 正しさ：解放後の frame は cache が空なので plan が `Damage::Full` を返し、全 chunk を raster して output を作り直す（resize 後と同じ経路）。softbuffer の buffer age が 1 でも damage が Full なので全面を copy する（`frame_policy.rs:518-531`）。
- 性能：解放後の最初の frame だけ全体 raster（800×600 で約 3.3 ms、1440×900 の catalog で 10 ms 前後と見込む）。K4（連続更新）は解放されない。K5 の `kv_latency` は仮想時間を進めない（`advance` を呼ばない）ので解放が起きない。K3 は 1 回の wake だけで frame を出さない。
- 既存 E2E のうち仮想時間を 1 s 以上進めて `frame_stats` を見るのは `drawable_export_gpu.rs:828,846` だけで、GPU の window（CPU compositor の解放は GPU の damage 計算に関係しない）。他の `frame_stats` の試験は 16 ms 刻み。
- 追加する E2E（`dev/tests/kivact-public-api/tests/e2e/compositor.rs`）：
  1. `idle_window_redraws_whole_after_release`：`grid(hot)` を mount → screenshot → `hot.set(RED)` → screenshot で部分 raster（`damage_pixels < W*H/10`）を確認 → `screen.advance(Duration::from_millis(1_100))` → `hot.set(GREEN)` → screenshot。`frame_stats().pixels_rastered` が window の物理 pixel 数に等しいこと、pixel が `render(|| grid(signal(GREEN)))` と一致（`assert_same_pixels(.., 2, ..)`）すること。
  2. `release_draws_no_frame`：mount → screenshot → `frame_ticks()` と `frame_stats()` を記録 → `advance(2 s)` → どちらも変わらないこと。
  3. `frames_within_keep_time_stay_partial`：更新の間に `advance(500 ms)` を挟んでも部分 raster のままであること。
- 試験 driver の deadline：解放の deadline は `next_deadline` に現れる。frame loop の試験（`crates/kivact-test/src/screen/real_window_tests.rs:100`）は frame の deadline（16.7 ms）が先に来るので変わらない。`run_until_idle` は due でない deadline を待たない。
- 状態：`LastPresent` の 3 状態。Shown を書くのは `redraw`、Shown→Settled は `wake` の解放だけ、Skipped の扱いは既存のまま。新しい field を作らない。

### 5.3 D3
- 正しさ：glyph run の bytes は fontique が渡す Blob と同じもの（copy を共有に変えただけ）。描画・計測・SVG の結果は同一。
- 寿命：mmap は run の `Arc` で保たれる（旧 doc の「返した glyph run は font の copy を自分で持つ」と同じ保証を共有で満たす）。
- 追加する試験：
  1. `crates/arvane/tests/e2e/text.rs` に `glyph_runs_share_the_registered_font`：`Engine::with_fonts_only(&[font.clone()])`（`font: Arc<[u8]>`）で shape した 2 つの text の全 run で `run.font.as_ptr() == font.as_ptr()`（複製していないことを address で確かめる）。
  2. 同 `glyph_runs_of_one_system_font_share_one_source`：`Engine::new()` で同じ style の 2 text を shape し、`run.font.id()` が一致すること。
  3. 登録経路：`dev/tests/kivact-public-api/tests/e2e/launch_fonts.rs` の既存 2 本と、SVG の `<text>` の既存 E2E が新しい経路を通る。

### 5.4 D4・D5
- 挙動：既定の app は CPU だけで描く。今の既定（OnDemand）でも counter 規模は GPU を起動しないので見た目・性能は同じ。GPU の機能は `features = ["gpu"]` で今と同じ。shader は既存の fallback を CPU で描く。
- E2E：`dev/tests/kivact-public-api` は `gpu` を既定に残すので既存の GPU E2E は同じ構成で回る。checkpoint の「`cargo tree` の GPU 非混入」は既定 build（`cargo tree -p kivact`）に対して行うよう作業指示書を直す。
- U56（GPU OnDemand の E2E）は `gpu` 付きの試験 crate で書くので D4 と独立。

### 5.5 D6・D7
- D6 は同じ順で同じ検査を行い同じ値で `f` を呼ぶ分割で、意味を変えない。既存の `kivact-runtime` の試験と machine を使う kit の E2E で担保。
- D7 は性能の経路から外れた crate だけ（§D7）。K1・K4・K5 は checkpoint の `run.sh` で確かめる。

### 5.6 D8
- 同じ glyph・同じ metrics なので catalog の screenshot・layout_check の期待値は変わらない（変わったら subset の設定の誤り）。文字の漏れは §D8 の試験で検出する。

---

## 6 unit 分割案

| unit | 内容 | 所有ファイル | 依存 | 完了時の確認 |
|---|---|---|---|---|
| P4-U1 | D1 | `crates/arvane-render/src/cpu.rs` | なし | `cargo clippy -p arvane-render --all-targets -- -D warnings`；`dev/tools/window-lock.sh -- cargo test -p arvane --test e2e present` |
| P4-U2 | D2（Arvane 側） | `crates/arvane/src/window.rs`、`crates/arvane/src/presenter/frame_policy.rs`、`crates/arvane-compose/src/cpu/compositor.rs` | U1 | `cargo clippy -p arvane -p arvane-compose --all-targets -- -D warnings` |
| P4-U3 | D2（host 側）と E2E 3 本 | `crates/kivact-engine/src/host/{window,events,pump}.rs`、`dev/tests/kivact-public-api/tests/e2e/compositor.rs` | U2 | `cargo clippy -p kivact-engine --all-targets -- -D warnings`；`dev/tools/window-lock.sh -- cargo test -p kivact-public-api-tests --test e2e compositor` |
| P4-U4 | D3（型と text・render） | `crates/kivact-types/src/render.rs`、`render/shaped.rs`、`crates/arvane-text/src/{output,text_engine,inline}.rs`、`crates/arvane-render/src/{draw/painter,validate/fonts,validate/scene,ink,outline}.rs`、`crates/arvane/src/engine.rs`、`crates/arvane/tests/e2e/text.rs` | U1 の後（`arvane-render` を順に触る） | `cargo clippy -p kivact-types -p arvane-text -p arvane-render -p arvane --all-targets -- -D warnings`；`dev/tools/window-lock.sh -- cargo test -p arvane --test e2e text` |
| P4-U5 | D3（登録・SVG） | `crates/arvane-asset/src/{font,store/asset_store,svg/fonts}.rs`、`crates/kivact-engine/src/lower/assets.rs` | U4、U3 の後（`kivact-engine` を順に触る） | `cargo clippy -p arvane-asset -p kivact-engine --all-targets -- -D warnings`；`dev/tools/window-lock.sh -- cargo test -p kivact-public-api-tests --test e2e launch_fonts` |
| P4-U6 | D4・D5 | `crates/kivact/Cargo.toml`、`templates/basic/Cargo.toml`（features 節）、`examples/kivact-ui-{basic,pop,soft,creative}/Cargo.toml`、`crates/kivact-devtools/Cargo.toml`、`dev/bench/Cargo.toml`、`README.md`、`README_JP.md`、`docs/for-kivact-user/{en,jp}/README.md`、`docs/for-kivact-ui-user/{en,jp}/creative/how-to-guide/show-app-rendered-images.md` | §8 Q1・Q2 の承認 | `cargo check -p kivact`；`cargo check -p kivact --features gpu`；`cargo tree -p kivact -e normal` に `wgpu` が無いこと |
| P4-U7 | D6 | `crates/kivact-runtime/src/reactive/{handles,graph}.rs`、`crates/kivact-runtime/src/machine.rs`、`crates/kivact-core/src/view/flow/list.rs` | なし | `cargo clippy -p kivact-runtime -p kivact-core --all-targets -- -D warnings`；`cargo test -p kivact-runtime` |
| P4-U8 | D7 | root `Cargo.toml`（profile 節）、`templates/basic/Cargo.toml`（profile 節） | なし（V2 作業指示書では S8＝本 unit が W2、S6＝U6 が W3 の後） | `cargo metadata --format-version 1 > /dev/null` |
| P4-U9 | D8 | `dev/tools/subset-catalog-font.py`、`examples/shared-assets/fonts/notosansjp/*`、`examples/kivact-ui-{basic,pop,game,creative}/src/fonts.rs`、`dev/tests/kivact-public-api/tests/e2e/layout_check.rs`、`examples/kivact-ui-basic` の test | §8 Q3 の承認 | `dev/tools/window-lock.sh -- cargo test -p kivact-ui-basic-catalog` |

checkpoint（重い確認）：workspace の fmt・clippy・test、`dev/bench/run.sh` の K2・K6 の行（と K1・K4・K5 の悪化なし）。

---

## 7 既存欠陥

- **E-1（資源の無駄、確定）**：登録 font の bytes が最大 4 回複製される。`crates/kivact-engine/src/lower/assets.rs:255` `file.bytes().to_shared()`（`Asset::Static` を `Arc<[u8]>` へ copy：`crates/arvane-asset/src/font.rs:46`）、`crates/arvane-asset/src/store/asset_store.rs:174` `bytes.to_vec()`（SVG 用の常駐 copy）、`crates/arvane-asset/src/svg/fonts.rs:49` `font.as_ref().clone()`（SVG に text があると fontdb 用に copy）、`crates/arvane-text/src/output.rs:474`（描画時に glyph run 用に copy）。再現：`App::fonts` で font を登録する app（catalog の NotoSansJP 9.6 MB 等）。B1 の catalog idle WS 151,736 KiB の一因と推定（catalog の WS 内訳は未計測）。修正は D3。
- **E-2（資源の無駄、確定）**：`CpuRaster.pixmap`（`crates/arvane-render/src/cpu.rs:33,129-136`）が window 大の中間 buffer として常駐し、全体 raster のたびに window 大の copy をする。修正は D1。
- **E-3（設計書の誤り、確定）**：設計 1.2 節の K6 表の「fat LTO・cgu 1・s・strip で 14.5 MB」「gpu なしで 13.7 MB（GPU 一式約 0.9 MB）」は再現しない（§1.1.1：同指定で B0 のコード 18.14 MB、GPU 一式 4.57 MB）。この数字から K6 の必達値が達成可能と判断された。
- **E-4（計測定義の不一致、確定）**：K6 の kv_startup no-default の目標（約 11 MB）は GPU なしの構成から算出されたが、`dev/bench/Cargo.toml` が `kivact` に `features = ["gpu"]` を固定しているため計測は GPU 込み。修正は D5。
- **E-5（計測の注意、確定）**：build 直後の exe の初回起動だけ exe の image 全体（kv_startup で 16,568 KiB）が WS に入る（2 回目以降は 4,272 KiB）。`run.sh` は K1 の 5 回の起動の後に K2 を測るので正式値には影響しない。K2 だけを単独で測る時は 1 回目を捨てる（`dev/bench/README.md` に 1 行書く）。

---

## 8 設計者（人間）の判断が要る点

- **Q1（D4）`gpu` を facade の既定 feature から外すか**。K6 の counter は GPU 一式（4.57 MB）を既定に含めたままでは届かない（GPU 込みの最良は約 16.8 MB 以上）。外すと既定の app は CPU だけで描く（今の既定でも counter 規模は CPU で描いている）。GPU の機能は `features = ["gpu"]` で今と同じ。SLIM-02（GPU は必要時に起動）は「app が `gpu` を有効にし、かつ必要になった時」に読み替える。
- **Q2（D5）K6 の `kv_startup --no-default-features` を「GPU なし・complex-scripts なし」の構成で測るか**。目標値は変えず、目標の算出根拠に構成を合わせる修正。直さない場合この行は GPU を含み、D4 の有無に関わらず届かない（GPU 込みで全 `s`・D7・dx12 なしを重ねても約 14 MB 前後の見込み）。
- **Q3（D8）catalog の NotoSansJP を subset にするか**。framework の変更だけでは catalog は約 37 MB（目標 31.9 MB）。埋込み font は app の asset で B0 も同じものを含む。subset で −8.4〜−9.0 MB（未計測の見積り。fontTools の取得が要る）。
- **Q4（D2）idle の解放までの時間 1 s**。短いほど WS の基礎分が早く下がり、長いほど「止まってから再び触った最初の frame の全体 raster」が減る。K2 の計測（初回 present の 3 s 後）より短い必要がある。
