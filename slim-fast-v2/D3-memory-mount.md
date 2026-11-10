# P3 設計：ノード・部品ごとのメモリ、mount 時間、U23/M-R5、kv_heap 診断

> **V2 付属書 D3**（設計担当 P3 の成果物）。正本は [SLIM_FAST_V2_DESIGN.md](../SLIM_FAST_V2_DESIGN.md)、手順は [SLIM_FAST_V2_WORK_ORDER.md](../SLIM_FAST_V2_WORK_ORDER.md)。
> - 本書の「8 章 設計者（人間）の判断が要る点」は V2 設計書 4 章（DEC-01〜DEC-23）で決定済み。実装者は 8 章を読んで選ばない。V2 設計書 5 章（付属書間の調整）が本書を上書きする箇所がある。
> - 本書の unit 名と V2 の unit ID の対応は V2 設計書 0.2 節。unit の実施順・lane・wave は V2 作業指示書だけが決める（本書 6 章の「依存」「並列可」は参考）。
> - 文中の P1〜P8 は付属書 D1〜D8（`design-P<n>.md` は `D<n>-*.md`）を指す。`W:\w\kv-prof`・`W:\w\kv-mem`・`W:\w\kv-size`・scratchpad の path は設計時の試作・計測の場所（設計した機械にだけある参考）で、実装に必要ではない。行番号は HEAD `73c36c7c` のもの。

担当：K2 item box/text/button/row・3000 行 idle WS、K5 mount n3000、AC-4 の STOP U23（M-R5）、kv_heap の site 名。
基点：db8d679b（= HEAD 73c36c7c の製品コード）。計測は本機（この引継ぎ先の機械）で行った。

用語：**本機 base** = db8d679b を本機で計測した値。**P1** = 下記 3 章の一部を worktree `W:\w\kv-mem` に試作して計測した値（試作は計測のためだけの近似で、本書の確定設計とは細部が違う。差は 4.1 節に書く）。
数値の単位：K2 の item は WS 差分÷3000（KiB、run.sh と同じ「3 s 待ち・CPU renderer・`kv_stress STILL`」）。heap は `kv_heap` の生存 byte（B、診断値）。

---

## 1. 事実

### 1.1 計測（本機）

| 指標 | B1（元機） | 本機 base | P1（試作） | 目標上限 |
|---|---|---|---|---|
| item box | 12.39 | 11.99 | **6.31** | 6.50 |
| item text | 10.48 | 10.41 | 6.49 | 3.64 |
| item button | 66.59 | 66.69 | 35.84 | 24.75 |
| item row | 85.44 | 85.48 | 46.71 | 34.45 |
| 3000 行 idle WS（CPU、`MODE=row`） | 292328 | 292060 | 176296 | 158479（default は 129989） |
| K5 mount n3000（`kv_latency` と同じ画面、試験 driver） | 1507.4 ms | 1543〜1651 ms | 942〜975 ms | 813.8 ms |
| 同上 + 3.4 節 MNT（insert の O(N²) 除去）を試作に追加 | — | — | **830〜837 ms** | 813.8 ms |

- 本機 base は B1 とほぼ一致する（box 11.99 / text 10.41 / button 66.69 / row 85.48）。B1 の比率で議論してよい。
- P1 で局所 paint の機構を戻すと（環境変数で切替）box +1.74、button +3.06、row +4.68 KiB。observation を戻すと button +1.27、row +1.50 KiB。
- WS は各 1 回の計測（雑音 ±0.3 KiB 程度）。mount は 3 回（他の担当の計測と CPU を共有した状態で、絶対値に ±5% の雑音）。

### 1.2 1 項目あたりの node・reactive・allocation（`kv_census` = 試験 driver の stats、100 行、for_each）

| 項目 | host node | live reactive | constants | objects | 生存 alloc（heap） |
|---|---|---|---|---|---|
| 空 `node()`（行の器だけ） | 2 | 1 | 0 | 1 | — |
| box `node().style(h20,w200)` | 2 | 5 | 0 | 1 | 33 |
| text（静的） | 2 | 1 | 0 | 1 | 16 |
| button `button("x")` | **9** | **41** | 7 | 2 | **227**（B1 は 262） |
| row `row((text, button))` | **12** | 45（動的 text で 46） | 7 | 2 | 261 |
| P1 button | 3 | 21 | 10 | 2 | 170 |
| P1 row（rowlat） | 5 | 23 | 10 | 2 | 192 |

行の器（for_each/for_index の 1 行）は owner 1、行の値の source 1、Callback object 1、**行の fragment node 1** を持つ。stats_n3000 の 12 node/行 = 行 fragment + row + row 子の tuple fragment + text + button 8。

**button の 8 host node**（`crates/kivact-ui/src/button.rs:211-286`）：root、content row、label text の 3 つと、**空・群の fragment 5 つ**：
- `row((icon.map(slot), label, trailing.map(slot), children))` の tuple → fragment（`description.rs:206-224`）
- `icon: None` → `View::empty` → mount 時 fragment（`description.rs:139-144`）
- `trailing: None` → 同上
- `children: Vec<View>`（空）→ `IntoView for Vec` が fragment（`description.rs:193-204`）
- `root.child(content).child(indicator)` の `indicator: None` → fragment（`builders.rs:261-264` が空 view も mount する）

各 fragment は `Display::Contents` の Box で、`Mcx::fragment`（`description.rs:30-44`）が `LayoutProps`（736 B の box）・`HitTestable`・`Focusable` を書く完全な host node である（engine の node slot・layout 列・paint/measure 列・CoreNode・event 等を持つ）。

**button の reactive 40**（`kv_census`、種類と作成箇所）：

| 由来 | source | memo | effect | 計 |
|---|---|---|---|---|
| `use_button()`（`toggle.rs:48-53`）：observed、disabled | 2 | | | 2 |
| `node_ref()`（`node_ref.rs:53-67`）：bound と、**常に作る** rect・scroll memo | 1 | 2 | | 3 |
| `part()`（`part.rs:60-132`）：input、availability、`focused` ×2（`focus_visible()` が内部でもう一度 `focused()` を作る、`node_ref.rs:634-647`）、`focus_visible`、PartState、selector の key source | 3 | 4 | | 7 |
| part の mount hook：`hovered()` の within memo と hovered memo、Internal effect（`part.rs:89-118`） | | 2 | 1 | 3 |
| button の `state` memo（`button.rs:243`） | | 1 | | 1 |
| button の `.disabled(move \|\| busy… \|\| disabled…)`（`button.rs:225-227`、**busy も disabled も無くても付く**）→ constraints の caller_disabled memo（依存 0）と disabled・eligible・invert の memo（`parts.rs:160-216`） | | 4 | | 4 |
| `observe_node`（`tree.rs:472-495`、NodeRef の bind で**常に**作る）：measured、text_truncated、scroll_geometry、image_status、scene_revision、editing_revision | 6 | | | 6 |
| root の局所 paint（`resolve.rs:80-85`, `211-224`）：notifier、size signal、draw effect | 2 | | 1 | 3 |
| root の style effect、semantics effect（reactive：disabled が computed のため） | | | 2 | 2 |
| region の無い part node にも作る Registry（`regions.rs:331-373`）：revision・changed notifier、reconcile・cancel の Internal effect | 2 | | 2 | 4 |
| content row（`button.rs:274`）：局所 paint（notifier、size、draw effect）と style effect | 2 | | 2 | 4 |
| label（`helpers.rs:28`）：style effect | | | 1 | 1 |
| 計 | 18 | 13 | 9 | 40 |

constants 7：constraints の 3（`parts.rs:174-216`）、`phase`・`reveal_pending` の 2（`button.rs:270`）、label・size の 2。

### 1.3 heap の内訳（`kv_heap`、symbol 付き、button 3000 個 − 0 個、B/item）

本機 base（合計 67,039 B/item、B1 の 67,039 と一致）。site は「最初の kivact/arvane 関数 ← 呼出元 ←…」。

| 区分 | 主な site | B/item |
|---|---|---|
| node ごとの列（9 node 分、2 冪の容量込み） | `NodeLayoutColumns::push` 11451、`NodeStore::reserve` 5591、`ViewNodes::create`（CoreNode の HashMap）2473、`PaintState::push_slot` 1660、`MeasureColumns::push` 524、その他 ~700 | ~22,400 |
| event queue の残留容量 | `VecDeque<HostEvent>::grow ← model::events` | 5,242 |
| reactive の arena と deps/subs | `Arena::insert` 6990、`Runtime::set_deps` 2002（38 alloc）、cell/closure ~1,500 | ~10,500 |
| engine の prop の box | `Writer::set ← Mcx::fragment`（fragment の LayoutProps 736 B ×6）4416、`← bind_style`（Layout/Paint/Text/…）3472、`← semantics::bind`（Semantics 416 ×2）832 | 8,720 |
| 状態 style の複製 | `StateStyles` の `PaintDelta` 752 ×3 = 2256、その中の box 448 ×2 = 896 | 3,152 |
| mount 時の closure の保持 | `mount_host_with` 1736（31 alloc）、button builder 1712（29）、`mount_host` 928、`root_target` 728、`bind_node_style` 624、`push_style` 560、`bind_props` 560、`route_accessibility` 384、`EventHandlers::only` 320 … | ~8,500 |
| **semantics patch の Vec（U23 / M-R5）** | `Common::append_semantics`（1 alloc、1280 B） | 1,280 |
| text の測定 | `LayoutPass::leaf`（測定記録の Vec）2056、`Measure::measure`（`TextExtents`）480、`Ellipsis::of` 325 | ~2,900 |

P1（36,211 B/item）で残る主なもの：layout 列 5677、`Writer::set` 4304（Layout ×3・Paint・Text・Semantics ×2）、`Arena::insert` 3885、`NodeStore` 2795、`StateStyles` 3152、`LayoutPass::leaf` 2056、builder の closure 1452、`mount_host_with` 1448、CoreNode 1244、deps 1138、part routing ~1,900。

text（P1、6,257 B/item、host node 1）：layout 列 1417、`LayoutPass::leaf` 904（2 alloc）、`NodeStore` 698、`Measure::measure`（`TextExtents`、`ComputedTextStyle` 464 B の複製を含む）480、Semantics 416、CoreNode 323、owner 273、text cache ~690。

### 1.4 mount の内訳（P1 に prof 計測 patch を当てた prof build、n3000、ms）

| span | P1 | P1 + 3.4 MNT |
|---|---|---|
| `start.host_new`（= mount + commit + 最初の layout） | 785 | 643〜657 |
| ├ `core.mount_window_root` | 592 | 432〜437 |
| │ ├ `w.insert`（15011 回） | **199（13.3 µs/回）** | **31〜35（2.0〜2.3 µs/回）** |
| │ ├ `m.bind_style`（12010 node、style effect の初回実行込み） | 161（13.4 µs/node） | 163 |
| │ ├ `m.resolve` / `m.sem_bind_props` / `w.create` / `w.set_prop`（99076 回） | 47 / 25 / 25 / 47 | 同程度 |
| │ └ `mount_in.setup`（行の builder 実行） | 42 | 同程度 |
| ├ `core.commit` | 27 | 35〜37 |
| └ `layout.compute`（最初の layout、text の shaping 含む） | 138 | 146〜152 |
| `start.settle`（a11y の公開 130〜139 と最初の raster） | 174 | 180〜187 |
| 合計 `L mount` | 960 | **830〜837** |

`w.insert` 13.3 µs/回の原因は 2.4 節（O(N²)）。

### 1.5 U23 / M-R5 の現状

- `crates/kivact-core/src/view/mount/semantics.rs:46-104` `bind`：patch を layer で並べ、`reactive` な patch が 1 つでもあれば `publish` closure（**patch の Vec 全体と `previous: RefCell<Option<Semantics>>`（416 B + 文字列）を所有**）を Render effect として保持する。button では reactive の原因は `disabled` だけ（1.2 節：button と use_button の両方が必ず computed の `.disabled(..)` を付ける）。
- engine は `Semantics`・`Disabled`・`Hidden`・`Focusable` の書込みで旧値と比べ、等しければ何もしない（`model/apply.rs:191-211` `set_prop`、`host/write/typed.rs:254` `set_semantics` は `boxed`）。`previous` は engine の比較と重複している。

### 1.6 kv_heap の site 名が空になる原因（確定、実験済み）

- rustc は MSVC で `/PDBALTPATH:%_PDB%` を使い、exe には PDB の**ファイル名だけ**（`kv_heap.pdb`）を埋め込む（`grep -a` で確認：exe 内の PDB 参照は `kv_heap.pdb` のみ）。dbghelp は exe と同じフォルダ等でこの名前を探す。
- `dev/bench/run.sh:30-33` の `build()` は exe だけを `$target/kpi/` へ複製する。
- 実験（同じ exe、`MODE=button ROWS=10`）：`target/release/kv_heap.exe` → site 名あり／PDB の無いフォルダへ複製 → 全 site が空（system DLL の export 名 `CallWindowProcW` だけ解決）／PDB を同じフォルダへ複製 → site 名あり。
- 加えて `kv_heap.rs` の `owner()`（`kv_heap.rs:122-138`）は「名前に `kivact` を含む最初の frame」を選ぶため、`alloc::raw_vec::RawVec::grow_one<kivact_engine::…::SyncRecord>` のような**generic 引数に kivact を含む std の関数**を site にする。release の ICF（同一 code の畳み込み）で別型の `grow_one` が 1 つの名前に畳まれるので、B1 の「`grow_one<SyncRecord>` 1.5 KB/button」は実際には layout の葉の測定記録（`LayoutPass::leaf`）だった（chain 表示で確認）。

### 1.7 旧設計 MEM-1〜MEM-7 の実装状況

| 項目 | 実装 | 効果が出なかった理由 |
|---|---|---|
| MEM-1 疎な `Style` | ✓（`kivact-style/src/sheet/fields.rs:501-523`、上限 72 B） | authored `Style` は effect の実行ごとに作って捨てる一時値で、保持量の主因ではなかった |
| MEM-2 `StateStyles` の box、既定値は保持しない | ✓（`kivact-types/src/paint.rs:923-958`、`model/states.rs:151-154`） | button は hover・pressed・focus_visible を実際に持ち、node ごとに 752 B の密な `PaintDelta` を 3 つ複製する（3.1 KB/button） |
| MEM-3 既定の Layout/Paint を保持しない | ✓（`typed.rs:166-172`） | fragment は `display: Contents` という**既定でない** LayoutProps を書く（736 B × 5〜7/行） |
| MEM-4 M-R4 queue の縮小 | 実装したが無効（`model/events.rs:32-43`） | 縮小は「次の drain」でしか起きず、mount の burst の後に event が来ないアイドルでは drain が無い（2.3 節） |
| MEM-4 M-R5 | 未実施（STOP） | 3.7 節で確定 |
| MEM-5 computed text の共有 | ✓（`measure/inherited_text.rs:30-49`） | — |
| MEM-6 busy の部品は busy の時だけ | ✓（`button.rs:268-273`、frame_loops=0） | 目標「live_reactive ≤ 0.6×B0」（≤ 100,890）は未達（B1 138,130 = 0.82×）。残りの 40/button の由来（1.2 節）を対象にしていなかった |
| MEM-7 型の大きさの上限 | ✓（`HostNode` ≤464、`NodeProperties` ≤240 など） | — |

旧設計の分析（`kv_heap` の上位 site）は、2 冪の容量と ICF で畳まれた名前に引きずられ、**構造の原因**（空・群の fragment、行の fragment、event queue の残留、styled box ごとの局所 paint、NodeRef/observation/Registry の即時生成、node ごとの同一 prop の複製）を見落とした。

---

## 2. 原因

1. **透明な fragment が host node になる**（1 行 12 node のうち 7 つ）。空の view（`None`、`()`、空の `Vec`）と tuple/Vec の群がそれぞれ完全な host node を作り、node あたり ~3 KB（engine の slot・layout 列 ~1 KB・CoreNode・LayoutProps 736 B・event）を使う。for_each/for_index/virtual_list の行も行ごとに fragment で包む。mount・commit・layout・a11y の時間も node 数に比例する。
2. **event queue が mount の burst の容量を持ち続ける**（box 1.3、button 5.2、row 10.5 KB/item、heap）。M-R4 の縮小は次の drain に依存し、アイドルでは来ない。
3. **styled Box ごとの局所 paint の機構**：`style().paint(..)` を使う node は稀なのに、style を持つ全 Box が notifier・size signal・draw effect・layout observer（`Handlers` の box）を作る（P1 で計測 box 1.74、button 3.06 KiB）。
4. **部品の即時生成**：NodeRef の rect/scroll memo、bind 時の observation（source 6 + scroll observer）、`focus_visible` の重複 `focused`、region の無い part node の Registry（effect 2・notifier 2）と a11y action 11 個の handler、何も disable しないのに computed の disabled（constraints の memo 4、semantics を reactive にして patch を保持＝U23 の直接の原因）。
5. **node ごとの同一 prop の複製**：3000 個の同じ button が同じ `LayoutProps`・`PaintProps`・`TextStyle`・`StateStyles` をそれぞれ box で持つ（P1 で 4.3 + 3.2 KB/button）。text の `TextExtents` も `ComputedTextStyle`（464 B）を node ごとに複製する。
6. **小さな Vec の容量**：葉の測定記録（`Record::sizes`、`Vec<(MeasureRequest, LeafSize)>`）が 4→8 と伸び ~1 KB/葉。reactive node の deps/subs がそれぞれ heap の Vec（button で 22〜38 alloc）。
7. **mount の O(N²)**：`Model::insert`（`model/apply.rs:451-512`）の最後の `self.stale_fact(parent)`（`:508`）は、parent が透明（list の fragment）だと `flat_children(parent)` を全部集める（`aggregate.rs:135-150`）。insert ごとに `forget_flat_around(parent)`（`node_tree.rs:469`）がその cache を捨てるので、3000 行の list では insert ごとに O(行数)。move（`:66`）と再親付け（`:462`）も同じ。
8. **WS と heap の差**：heap は要求 byte（Vec の容量）を数え、WS は触れた page を数える。巨大な列（arena、layout 列、NodeStore、CoreNode の表）の 2 冪の未使用部分は WS に入らない（row で heap 108.9 KB > WS 85.4 KiB の主因、36,023 node → 容量 65,536）。逆に小さな allocation の header と断片化は WS に入る（button 227 alloc × ~16 B）。したがって**目標の判定と見積りは WS の実測差分**（P1）で行い、heap は内訳の帰属にだけ使う。allocation の数を減らすことは WS にも効く。

---

## 3. 設計（確定版）

ID は M2-x。各項目に「変更箇所（file:line は db8d679b）」「型・関数」「全ての書き手・呼出元」「不変条件」を書く。

### 3.1 M2-1 空の view と群を親へ平坦化する（kivact-core）

**型**（`crates/kivact-core/src/view/descriptor/description.rs:114-121`）：

```rust
type MountFn = Box<dyn FnOnce(&Mcx<'_>) -> NodeId>;

/// What an unmounted view is: nothing, one node, or a group of views mounted
/// in order into the parent (a tuple or a `Vec`).
enum ViewRepr {
    Empty,
    One(MountFn),
    Group(Vec<View>),
}

/// A type-erased, unmounted view. Mounting it creates nodes and effects
/// exactly once.
pub struct View {
    repr: ViewRepr,
}
```

**関数**（同ファイル）：

```rust
impl View {
    fn new(f: impl FnOnce(&Mcx<'_>) -> NodeId + 'static) -> Self {
        View { repr: ViewRepr::One(Box::new(f)) }
    }
    pub fn empty() -> Self {
        View { repr: ViewRepr::Empty }
    }
    /// A group of `views` (empty ones dropped): nothing for none, the view
    /// itself for one, a group otherwise.
    pub(crate) fn group(views: Vec<View>) -> Self {
        let mut views: Vec<View> = views.into_iter().filter(|v| !v.is_empty()).collect();
        match views.len() {
            0 => View::empty(),
            1 => views.pop().expect("invariant: one view"),
            _ => View { repr: ViewRepr::Group(views) },
        }
    }
    pub(crate) fn is_empty(&self) -> bool {
        matches!(self.repr, ViewRepr::Empty)
    }
    /// Mount as one node (a fragment for nothing or for a group).
    fn mount(self, cx: &Mcx<'_>) -> NodeId {
        match self.repr {
            ViewRepr::Empty => cx.fragment(),
            ViewRepr::One(f) => f(cx),
            ViewRepr::Group(views) => {
                let frag = cx.fragment();
                for v in views {
                    v.mount_into(cx, frag);
                }
                frag
            }
        }
    }
    /// Mount into `parent`, in order: nothing for nothing, each member of a
    /// group directly under `parent` (no fragment).
    pub(crate) fn mount_into(self, cx: &Mcx<'_>, parent: NodeId) {
        match self.repr {
            ViewRepr::Empty => {}
            ViewRepr::One(f) => {
                let child = f(cx);
                cx.append(parent, child);
            }
            ViewRepr::Group(views) => {
                for v in views {
                    v.mount_into(cx, parent);
                }
            }
        }
    }
}
```

- `IntoView for Vec<V>`（`:193-204`）：`View::group(self.into_iter().map(IntoView::into_view).collect())`。
- `tuple_into_view!`（`:206-224`）：本体を `View::group(vec![$($name.into_view()),+])` にする。
- `IntoView for Option<V>`・`()` は今のまま（`View::empty()`）。

**呼出元の変更**：
- `crates/kivact-core/src/view/elements/builders.rs:261-264`：`for child in children { child.mount_into(cx, id); }`。未使用になる `mount` の import（`:28`）を外す。
- `crates/kivact-core/src/view/overlays.rs:440-441`：`child.mount_into(cx, id);`（Overlay は host node なので平坦化してよい）。
- 単一の node が要る所は `mount(v, cx)` のまま（`View::mount` は Empty・Group に fragment を作る）：`boundary.rs:40`（`mount_in`）、`container.rs:166`（Scroll は root の child として渡すので builders 側で平坦化される）、`container.rs:191`（stack の layer は fragment の Contents を見て子を取り出す既存の処理のまま）、`drawable/mount.rs:83`、`custom/measured_leaf.rs:160`。

**不変条件**：
- 群の member は、群の位置に、宣言の順で親の子になる（fragment は Contents で、試験の query・snapshot・layout・a11y は Contents の子を親の子として扱うため、外から見える結果は変わらない）。
- 空の view は host node を作らない（`View::mount` で単一 node を求められた時だけ fragment）。
- node の作成順（id の増加順）は保たれる（fragment の id が割り当てられなくなるだけ）。

**効果**：button の host node 8 → 3（root・content・label）、`row((text, button))` は row の tuple fragment が消える。

### 3.2 M2-2 list・virtual_list の行を fragment で包まない（kivact-core）

- `crates/kivact-core/src/view/flow/list.rs:238-254`（for_each の `Plan::New`）：

```rust
Plan::New(item) => {
    let Some(keyed) = keyed::Row::new(parent, k.clone(), item, at) else {
        continue;
    };
    let item = keyed.item();
    // The row's own node: its view's root, or a fragment for an empty view
    // or a group (`View::mount`).
    let node = mount_in(rt, keyed.owner(), window, || row.call((k, item)));
    new_rows.push(Row { row: keyed, node });
    fresh.push(true);
    moved.push(false);
}
```
  （`let cx = Mcx { .. }` と `cx.fragment()`・`cx.append(node, child)` を削除。）
- `list.rs:341-349`（for_index）：`mount_branch(&rt, owner, window, frag, || row(i, sig.into_value()).into_view())`（内側の `view_fn` と fragment を削除）。
- `crates/kivact-core/src/view/flow/virtual_list.rs:533-535`：`let content = mount_in(rt, owner, window, || row.call((k.clone(), sig))); cx.append(wrapper, content);`（fragment を削除。wrapper は高さ観測のため残す）。

**不変条件**：行の node は、その行の view の根（単一 node の view）か、空・群の時の fragment。行の移動（`move_before`）、削除（`unmount_branch` / `remove`）、退場の paint（engine は削除された node の subtree の exit を見る）は、どの node でも同じに働く。行の作成順（「Creates are emitted in id order」）は変わらない。

### 3.3 M2-3 event queue は buffer ごと渡す（kivact-engine）

`crates/kivact-engine/src/model/events.rs:32-43` `EventQueue::drain` を次にする（M-R4 の縮小と注釈を削除）：

```rust
/// Take every queued event in order, with the commit boundaries among them:
/// the number of events before each boundary and its revision. The queue
/// hands its buffers over, so it keeps no capacity of a burst.
fn drain(&mut self) -> (Vec<HostEvent>, Vec<(usize, Revision)>) {
    let events: Vec<HostEvent> = std::mem::take(&mut self.events).into();
    let boundaries: Vec<(usize, Revision)> = std::mem::take(&mut self.boundaries).into();
    (events, boundaries)
}
```

`VecDeque → Vec` の変換は buffer を再利用する（O(1)〜O(n) の回転、再確保なし）。受け手（`host/pump.rs:507`）は turn の後に Vec を捨てる。設計書 4.9 EVT-1 と規則 M-R4 は「queue は drain で buffer ごと渡す」に置き換える（V2 の規則表）。

単体試験（同ファイルに `#[cfg(test)] mod tests`）：5000 event を push → `drain` → `events.capacity() == 0` かつ返した Vec の長さ 5000、順序が push 順。

### 3.4 M2-4 構造変更の facts を O(1) にする（kivact-engine、mount の O(N²) 除去）

`crates/kivact-engine/src/model/apply.rs` に追加：

```rust
/// What a structural change among the children of `id` (one inserted,
/// detached or moved) outdates: the facts of the box that lays them out,
/// `id` itself or its nearest ancestor that is not a Contents node (the
/// root's when `id` is detached). The other children keep their facts: a
/// child's facts come from its own subtree, never from its siblings.
fn stale_children_of(&mut self, id: NodeId) {
    if let Some(at) = self.tree.box_of(id).or_else(|| self.tree.root()) {
        self.paint.stale_fact(at);
    }
}
```

置換（3 箇所、どれも「子が出入りした親」に対する呼出し）：
- `apply.rs:66`（`move_before` の detach 後）`self.stale_fact(parent)` → `self.stale_children_of(parent)`
- `apply.rs:462`（`insert` の再親付けの detach 後）`self.stale_fact(previous)` → `self.stale_children_of(previous)`
- `apply.rs:508`（`insert` の最後）`self.stale_fact(parent)` → `self.stale_children_of(parent)`

動いた node 自身の `self.stale_fact(id)`（`:74`, `:80`, `:466`）は残す（透明な node なら自分の flat の子を stale にする、その数は自分の子の数）。Contents の切替え（`interact/sync.rs:168-171`）・transform（`transform.rs:186`）など他の `stale_fact` は変えない。

`NodeTree::box_of`（`tree/node_tree.rs:194`）は既存（node 自身が Contents でなければ自身、Contents なら最も近い非 Contents の祖先）。

**効果**（P1 で試作・計測）：`w.insert` 13.3 → 2.0〜2.3 µs/回、mount n3000 960 → 830〜837 ms。keyed list の並べ替え（move）も O(移動数)。

### 3.5 M2-5 engine の prop を直近値で共有する（kivact-engine）

**新ファイル** `crates/kivact-engine/src/model/recent.rs`（`model.rs` の mod 一覧に `pub(crate) mod recent;`）：

```rust
//! Role: The last distinct values each kind of node property was written with,
//! shared between the nodes that write an equal value (repeated components).
use std::rc::Rc;

use kivact_types::{LayoutProps, PaintProps, StateStyles, TextStyle};

const SLOTS: usize = 8;

/// The last `SLOTS` distinct values of one property kind.
pub(crate) struct Recent<T> {
    slots: [Option<Rc<T>>; SLOTS],
    // xtask-allow(state-fields): independent: the ring position; any value is a valid next slot
    next: usize,
}

impl<T> Default for Recent<T> {
    fn default() -> Self {
        Self { slots: std::array::from_fn(|_| None), next: 0 }
    }
}

impl<T: PartialEq> Recent<T> {
    /// `value` behind an `Rc`: an equal recent value's (shared), else a new
    /// one that replaces the oldest recent value.
    pub(crate) fn share(&mut self, value: T) -> Rc<T> {
        if let Some(found) = self.slots.iter().flatten().find(|rc| ***rc == value) {
            return Rc::clone(found);
        }
        let rc = Rc::new(value);
        self.slots[self.next] = Some(Rc::clone(&rc));
        self.next = (self.next + 1) % SLOTS;
        rc
    }
}

/// The recent values of the shared node properties of one window.
#[derive(Default)]
pub(crate) struct RecentProps {
    pub(crate) layout: Recent<LayoutProps>,
    pub(crate) paint: Recent<PaintProps>,
    pub(crate) text: Recent<TextStyle>,
    pub(crate) states: Recent<StateStyles>,
}

/// A node property stored shared ([`NodeProperties`](crate::contract::NodeProperties)).
pub(crate) trait SharedProp: PartialEq + Sized {
    fn recent(props: &mut RecentProps) -> &mut Recent<Self>;
}
impl SharedProp for LayoutProps { fn recent(p: &mut RecentProps) -> &mut Recent<Self> { &mut p.layout } }
impl SharedProp for PaintProps { fn recent(p: &mut RecentProps) -> &mut Recent<Self> { &mut p.paint } }
impl SharedProp for TextStyle { fn recent(p: &mut RecentProps) -> &mut Recent<Self> { &mut p.text } }
impl SharedProp for StateStyles { fn recent(p: &mut RecentProps) -> &mut Recent<Self> { &mut p.states } }
```

**型の変更**（`crates/kivact-engine/src/contract/node.rs:44-55`）：`layout: Option<Rc<LayoutProps>>`、`paint: Option<Rc<PaintProps>>`、`text: Option<Rc<TextStyle>>`、`states: Option<Rc<StateStyles>>`（`use std::rc::Rc;`）。`semantics` は `Box` のまま（setter が field ごとに書き換えるため）。derive（Clone・Debug・PartialEq）は Rc でも値で比べる。コンパイラが `NodeProperties: Send` を要求した場合に限り 4 つを `std::sync::Arc` にする（API は同じ）。

**書き手**：
- `crates/kivact-engine/src/model/aggregate.rs:25`（`Model`）に `pub(crate) recent: super::recent::RecentProps,`、`Model::new`（`:81-118`）に `recent: Default::default(),`。
- `crates/kivact-engine/src/model/apply.rs:289-305` `set_boxed_prop` の隣に追加（`set_boxed_prop` は Semantics などで残る）：

```rust
/// [`Model::set_boxed_prop`] for a property stored shared: an equal recent
/// value of the window is reused instead of allocating another (M2-5).
pub(crate) fn set_shared_prop<T: super::recent::SharedProp>(
    &mut self,
    id: NodeId,
    value: Option<T>,
    field: impl Fn(&mut NodeProperties) -> Option<&mut Option<std::rc::Rc<T>>>,
    mutation: Mutation,
) -> bool {
    let unchanged = self
        .tree
        .get_mut(id)
        .and_then(|node| field(&mut node.properties))
        .is_some_and(|old| old.as_deref() == value.as_ref());
    if unchanged {
        return false;
    }
    let value = value.map(|v| T::recent(&mut self.recent).share(v));
    self.set_prop(id, value, field, mutation)
}
```
- `crates/kivact-engine/src/host/write/typed.rs:154-161`（`props!` の `@store`）に `(@store $w:ident, $id:ident, $v:ident, $m:ident, shared($value:expr, $place:expr)) => { $w.record($id, |model| model.set_shared_prop($id, $value, $place, $m)) };` を追加し、`:166-176` の Layout・Paint・TextStyle を `boxed(..)` → `shared(..)` にする。
- `crates/kivact-engine/src/model/states.rs:151` `set_boxed_prop` → `set_shared_prop`。
- `crates/kivact-engine/src/model/presence/transitions.rs:256` `Some(Box::new(layout))` → `Some(std::rc::Rc::new(layout))`。
- 読み手は `as_deref()`・`as_ref()` のままで変更不要（`tree/node.rs:336-348`、`kivact-core/src/client/dispatch/env.rs:72`、`view/tree.rs:735,845`、`view/elements/container.rs:195`、`kivact-test/src/facts.rs:62`）。コンパイルで型の不一致が出た読み手だけ `as_deref()` に直す。

**M2-5b `TextExtents` の style を共有**（`crates/kivact-engine/src/measure/state.rs`、`measure/leaf.rs`）：
- `state.rs:132-136` `TextExtents.style: Rc<kivact_types::ComputedTextStyle>`。
- `state.rs:30-34` `MeasureColumns` に `recent_style: RefCell<crate::model::recent::Recent<kivact_types::ComputedTextStyle>>`（Default）と `pub(crate) fn share_style(&self, style: ComputedTextStyle) -> Rc<ComputedTextStyle> { self.recent_style.borrow_mut().share(style) }`。
- `leaf.rs:141-149`：

```rust
let mut extents = self.cache.extents(node.index.0)?;
if extents.as_ref().is_none_or(|e| *e.style != style) {
    let single = self.engine.text_metrics(text, &style, None).ok();
    *extents = Some(Box::new(TextExtents {
        style: self.cache.share_style(style),
        single,
        wrapped: Vec::new(),
    }));
}
```
  （`&extents.style` を渡している `:150`, `:159` は deref で `&ComputedTextStyle` になる。）

**不変条件**：共有された値は不変（engine は書換えに新しい値を set する。共有値をその場で変える経路は無い：`transitions.rs:256` も新しい Rc を作る）。比較は値（`PartialEq`）で、Rc の同一性に意味を持たせない。ring は window の Model と一緒に捨てる。

### 3.6 M2-6 局所 paint を必要な node だけに作る（kivact-core）

**型**（`crates/kivact-core/src/view/mount/facts.rs:17-27`）：

```rust
/// How a styled node records its style's paint callback (`Style::paint`).
#[derive(Clone, Default)]
pub(crate) enum LocalPaint {
    /// Not a box, an editor, or no style: a paint callback is reported as unsupported.
    #[default]
    None,
    /// A box with its own draw layers or canvas regions: its draw effect
    /// records the callback with them (`draw::bind`).
    WithDraw(LocalPaintSource),
    /// Any other styled box: the style effect records the callback itself,
    /// observing the node's size only once a callback appears.
    InStyle,
}
```
`NodeFacts.local_paint: LocalPaint`。`LocalPaintSource` は今のまま。

**resolve**（`crates/kivact-core/src/view/mount/resolve.rs:57-102`）：
```rust
let declares_regions = !common.regions.is_empty();          // attach_parts の前に読む
let attached = attach_parts(&mut common, cx, id, at);
...
let styled_box = !common.style.is_empty() && kind == NodeType::Box && editing.is_none();
let own_draw = common.draw.is_some() || common.draw_over.is_some() || common.draw_with_layout.is_some();
let local_paint = match (styled_box, own_draw || declares_regions) {
    (true, true) => LocalPaint::WithDraw(LocalPaintSource {
        value: Rc::new(RefCell::new(None)),
        revision: cx.owner.notifier(at),
    }),
    (true, false) => LocalPaint::InStyle,
    (false, _) => LocalPaint::None,
};
let draw_paint = match &local_paint { LocalPaint::WithDraw(s) => Some(s.clone()), _ => None };
let draw = prepare_draw(&mut common, cx, attached.as_ref(), draw_paint, at);
```
`prepare_draw`（`:198-224`）の条件 `(observe_layout || local_paint.is_some())` は今のまま（WithDraw の時だけ `Some`）。

**style**（`crates/kivact-core/src/view/mount/style.rs`）：`NodeStyle.local_paint: LocalPaint`（`#[derive(Default)]` のまま、既定 None）。`bind_node_style`（`:58-66`）は `local_paint: facts.local_paint.clone()`。`root.rs:95` は `..NodeStyle::default()` で None。`bind_style` の effect（`:126-175`）の `local_paint` の扱いを次にする：

```rust
match &local_paint {
    LocalPaint::None => {
        if has_paint && reports.leaf_paint.note() { /* 今の Unsupported 報告（:140-149） */ }
    }
    LocalPaint::WithDraw(source) => { /* 今の :150-165 のまま */ }
    LocalPaint::InStyle => in_style.record(&rt, owner, id, &resolved, has_paint, &theme_value, at),
}
```
effect が捕まえる状態（style.rs に追加）：

```rust
/// The style paint of a box without draw layers, recorded by its style effect.
#[derive(Default)]
struct InStylePaint {
    /// The node's layout size, observed once a callback first appears.
    // xtask-allow(state-fields): option: created with the first paint callback; absent means the size is not observed
    size: Cell<Option<Signal<Size>>>,
    /// A callback's scene is on the node's Draw layer.
    // xtask-allow(state-fields): projection: written only by InStylePaint::record
    drawn: Cell<bool>,
    reported: kivact_types::cell::ReportOnce,
}

impl InStylePaint {
    fn record(&self, rt: &Rc<Runtime>, owner: Owner, id: NodeId, resolved: &CompiledStyle,
              has_paint: bool, theme: &Theme, at: &'static Location<'static>) {
        if !has_paint {
            if self.drawn.replace(false) {
                let empty = kivact_style::internal::record(Size::new(0.0, 0.0), theme, |_| {}).scene;
                ViewState::tree_mut(rt).set_scene(id, SceneLayer::Draw, empty);
            }
            return;
        }
        let size = match self.size.get() {
            Some(size) => size,
            None => {
                let size = owner.signal(Size::new(0.0, 0.0), at);
                let mut tree = ViewState::tree_mut(rt);
                if let Some(handlers) = tree.handlers_mut(id) {
                    handlers.observe_layout(move |_, s| size.set(s));
                }
                tree.set_observe(id, true);
                drop(tree);
                self.size.set(Some(size));
                size
            }
        };
        let sz = size.get(); // tracked: a resize records again
        let measurer = ViewState::tree_mut(rt).reader().text_measurer(id);
        let Recording { scene, unsupported, .. } = kivact_style::internal::record(sz, theme, |d| {
            if let Some(m) = measurer { d.set_text_measurer(m); }
            resolved.paint_retained(d, sz); // signals the callback reads are tracked by this effect
        });
        ViewState::tree_mut(rt).set_scene(id, SceneLayer::Draw, scene);
        self.drawn.set(true);
        super::draw::report_unsupported(rt, owner, id, unsupported, &self.reported, at);
    }
}
```
- `draw.rs:114` `report_unsupported` を `pub(super)` にする。
- `theme_value`：effect の中で `theme_sig.with(|t| ..)` の外で `let theme_value = theme_sig.get();`（既に依存として追跡済み）。
- 呼出しは `drop(tree)` の後、`reports.report(..)` の前（tree の借用を持たない位置）。

**不変条件**：`Style::paint` の callback は、size・theme・callback が読む signal の変化で記録し直され、callback が無くなれば Draw layer が空になる（`style_paint::paint_only_style_tracks_size_signals_theme_and_removal` が担保）。自前の draw/region を持つ node は今と同じ経路（`merged_paint_runs_between_caller_draw_and_children_and_clears_independently`）。callback を持たない styled box は notifier・size・draw effect・layout observer を持たない。

### 3.7 M2-7 意味論の plan（U23 / M-R5 の確定設計、kivact-core）

`crates/kivact-core/src/view/mount/semantics.rs` の `resolve`（`:14-31`）と `bind`（`:46-104`）を次に置き換える。`checked_state`（`:33-44`）・`bind_late`・`SemanticProps`・`bind_props` は今のまま。

```rust
/// What a node's semantic publisher keeps after mounting (M-R5): the fold of
/// the patches before the first reactive one, and the patches from it on.
/// A node with only constant patches keeps nothing: it publishes once.
struct Plan {
    base: Semantics<FocusTarget>,
    checked: Option<bool>,
    indeterminate: Option<bool>,
    rest: Box<[Rest]>,
}

/// One patch from the first reactive one on, in layer order.
struct Rest {
    apply: Rc<dyn Fn(&mut Semantics<FocusTarget>)>,
    input: RestInput,
}

/// The preserved input a later patch publishes through the checked state.
enum RestInput {
    Other,
    Checked(Prop<bool>),
    Indeterminate(Prop<bool>),
}

impl Plan {
    /// `patches` sorted by layer.
    fn new(patches: Vec<SemanticsPatch>) -> Self {
        let split = patches.iter().position(|p| p.reactive).unwrap_or(patches.len());
        let mut base = Semantics::default();
        let (mut checked, mut indeterminate) = (None, None);
        let mut rest = Vec::with_capacity(patches.len() - split);
        for (index, patch) in patches.into_iter().enumerate() {
            if index < split {
                (patch.apply)(&mut base);
                match &patch.input {
                    SemanticInput::Checked(source) => checked = Some(source.get()),
                    SemanticInput::Indeterminate(source) => indeterminate = Some(source.get()),
                    _ => {}
                }
            } else {
                let input = match patch.input {
                    SemanticInput::Checked(source) => RestInput::Checked(source),
                    SemanticInput::Indeterminate(source) => RestInput::Indeterminate(source),
                    _ => RestInput::Other,
                };
                rest.push(Rest { apply: patch.apply, input });
            }
        }
        Plan { base, checked, indeterminate, rest: rest.into_boxed_slice() }
    }

    /// The canonical value: the base, then each later patch in order.
    fn resolve(&self) -> Semantics<FocusTarget> {
        let mut semantics = self.base.clone();
        let (mut checked, mut indeterminate) = (self.checked, self.indeterminate);
        for patch in self.rest.iter() {
            (patch.apply)(&mut semantics);
            match &patch.input {
                RestInput::Checked(source) => checked = Some(source.get()),
                RestInput::Indeterminate(source) => indeterminate = Some(source.get()),
                RestInput::Other => {}
            }
        }
        if checked.is_some() || indeterminate.is_some() {
            semantics.checked = checked_state(checked, indeterminate);
        }
        semantics
    }
}

fn bind(cx: &Mcx<'_>, id: NodeId, mut patches: Vec<SemanticsPatch>, at: &'static Location<'static>) {
    if patches.is_empty() {
        return;
    }
    patches.sort_by_key(|p| p.layer);
    if let Some(node) = ViewState::tree_mut(cx.rt).get_mut(id) {
        node.semantic_origins = patches.iter().map(|p| p.at).collect();
    }
    let text_at = patches.iter().rev().find(|p| p.text).map_or(at, |p| p.at);
    let reactive = patches.iter().any(|p| p.reactive);
    let plan = Plan::new(patches);
    let weak = cx.weak();
    let owner = cx.owner;
    let publish = move || {
        let mut props = plan.resolve();
        let Some(rt) = weak.upgrade() else { return };
        if let Some(Err(why)) = props.text.as_ref().map(kivact_types::check_text) {
            props.text = None;
            rt.report(Some(owner), UiError::new(ErrorKind::Unsupported,
                format!("semantics_text is not published: {why}"), Some(text_at)));
        }
        let writer = rt.running_effect();
        let mut tree = ViewState::tree_mut(&rt);
        // These facts also drive native input and layout; the engine ignores equal values.
        tree.update(id, PropValue::Disabled(props.disabled), writer);
        tree.update(id, PropValue::Hidden(props.hidden), writer);
        tree.update(id, PropValue::Focusable(props.focusable), writer);
        tree.update(id, PropValue::Semantics(props), writer);
    };
    if reactive {
        cx.effect(EffectKind::Render, at, publish);
    } else {
        publish();
    }
}
```

**正しさ**：今の `resolve` は全 patch を layer 順に fold する。`split` より前の patch はそれぞれ自分の source が `fixed()`（定数）で、`apply` は定数を読む純粋な関数なので、mount 時に一度 fold した結果は毎回の fold の先頭部分と同じになる（Checked/Indeterminate の最後の値も同じ）。`split` 以降は今と同じ順で適用する。`previous`（旧値の複製）は engine の書込み比較（`set_prop` の `*old == value`、`ViewNodes::update` の hidden・live region の変化判定）と重複するので持たない：等しい値の再書込みは engine で何もしない。

**M-R5 の規則の置き換え**（V2 の規則表へ）：「mount の後に node が保持する意味論は、最初の reactive な patch より前を fold した `Semantics` 1 つと、それ以降の patch だけ。reactive な patch が無い node は何も保持しない」。OPEN.md の U23 はこの節で閉じる。

**button での効果**：M2-12 で button の意味論は全て定数になるため effect も plan も残らない（P1 で effect が消えたことを確認）。reactive な名前・checked 等を持つ部品では、patch の Vec（~128 B × n + closure）と `previous`（416 B + 文字列）の代わりに base 1 つ（416 B）と残りの patch だけを持つ。

### 3.8 M2-8 NodeRef の memo と observation を使う時に作る（kivact-core）

1. `crates/kivact-core/src/view/ref_data.rs:42-51` `NodeRefState`：
   - `rect_value: Cell<Option<Value<Option<Rect>>>>`、`scroll_value: Cell<Option<Value<crate::geometry::Point>>>`（`// xtask-allow(state-fields): option: created by the first reader; absent until NodeRef::rect / scroll_offset is called`）。
   - 追加 `observed: Cell<bool>`（`// xtask-allow(state-fields): projection: written only by NodeRef::require_observation`）：この ref の node に `NodeObservation` が要る。
2. `crates/kivact-core/src/view/node_ref.rs:53-67` `owned_by`：`resolved_rect`・`resolved_scroll` を呼ばない（`Cell::new(None)`、`observed: Cell::new(false)`）。
3. 新しい関数（node_ref.rs）：

```rust
/// Keep a `NodeObservation` on every node this ref binds to, from now on
/// (the readers of geometry, scroll, text truncation and image status).
pub(crate) fn require_observation(&self, at: &'static Location<'static>) {
    let (rt, s) = self.state(at);
    if s.observed.replace(true) {
        return;
    }
    if let Some(bound) = s.bound.try_get_untracked().and_then(RefState::bound) {
        let _ = ViewState::tree_mut(&rt).observe_node(bound.node(), at);
    }
}
```
4. 呼出し（各公開 API の先頭、setup または handler の文脈。memo の中では呼ばない）：
   - `rect()`（`:572-580`）：`self.require_observation(at)`。その後 `rect_value` が無ければ `s.owner.with_setup(|| resolved_rect(self.app, s.bound))` を作って保存し返す。
   - `scroll_offset()`（`:318-322`）：同様に `require_observation` と `resolved_scroll` の遅延作成。
   - `text_truncated()`（`:553`）、`image_status()`（`node_status.rs:28`）、`scroll_progress`・`view_progress`（`scroll_timeline.rs:32`, `:47`、対象の ref ごと）：先頭で `require_observation`。`scroll_state()` は `rect()` を呼ぶので追加不要。
5. `bind_target`（`node_ref.rs:159-163`）：

```rust
// One host-owned callback pair serves every ref that reads the observation.
if state.observed.get() || matches!(bound, Bound::Region { .. }) {
    if ViewState::tree_mut(rt).observe_node(id, at).is_none() {
        return;
    }
} else if ViewState::tree(rt).get(id).is_none() {
    return;
}
```
   （region への bind は `focused`・`target_value` が `scene_revision` を読むので今どおり作る。）
6. `CoreNode.observation`（`crates/kivact-core/src/view/tree.rs:312-313`）を `Option<Box<NodeObservation>>` にする。`observe_node`（`:472-495`）は `Some(Box::new(observation))` を格納し `Some(observation)`（Copy）を返す。読み手は `.as_deref().copied()`：`client/dispatch/delivery.rs:296`、`client/dispatch/env.rs:113`、`client/dispatch/text.rs:89`、`view/node_ref.rs:220`, `:621`, `:736`、`view/tree.rs:479`, `:671`。

**不変条件**：observation を読む API を一度でも呼んだ ref は、その後に bind する全ての node に observation を作る（再 bind も）。読み手の memo は作成時点で observation が存在する（`require_observation` が memo 作成の前に走る）。host からの report（dispatch）は observation の無い node では捨てる（今と同じ）。

### 3.9 M2-9 part の focus と hover の memo を 1 つずつにする（kivact-core）

- `node_ref.rs` に `pub(crate) fn focus_value(&self) -> Value<crate::view::part_data::Focus>` を追加。中身は `focused()`（`:601-631`）の memo と同じ前処理で、最後を `if !keys.selector.is(&id) { return Focus::None } if keys.focus.get().is_some_and(|k| k.visible) { Focus::FocusedVisible } else { Focus::Focused }` にする（`keys.focus` は focus 中だけ読む：今の `focus_visible` の短絡と同じ依存）。
- `part.rs:63-64` を `let focus = node.focus_value();` に、`:81-85` を `focus: focus.get(),`、`enabled`・`input_observers`・mount effect の `focused.get_untracked()` / `focused.get()` を `focus.get_untracked() != Focus::None` / `focus.get() != Focus::None` にする（`input_observers` の引数型は `Value<Focus>`）。
- `NodeRef::hovered`（`:708-733`）：`within`（`:783-800`）の memo を作らず、hovered の memo 1 つで判定する：

```rust
kivact_runtime::memo(move || match node.get().bound() {
    Some(Bound::Region { .. }) => node.get().bound().is_some_and(|b| hover.get() == Some(b.target())),
    Some(Bound::Node(_)) => is_within(app, node, hover),  // `within` の memo 本体を関数にしたもの
    None => false,
})
```
  `within` 関数は `fn is_within(app, node: Signal<RefState>, source: Signal<Option<FocusTarget>>) -> bool` に変える（呼出元は hovered だけ）。Region の場合、hover が region の target に等しければ node の subtree の中なので、今の「within かつ等しい」と同じ。
- 公開の `NodeRef::focused()`・`focus_visible()` は変えない（利用者が呼んだ時だけ memo を作る）。

### 3.10 M2-10 region の無い part node の Registry を軽くする（kivact-core）

`crates/kivact-core/src/view/mount/regions.rs:324-374` `prepare`：
- `let declares = !declarations.is_empty();`（ループの前）。reconcile の effect（`:357-363`）は `if declares { .. }` の中だけで作る。正しさ：宣言が無ければ collection は登録されず、`changed` は誰も notify しない。reconcile の初回実行は空の state で何もしない。
- 取消しの effect（`:365-373`）：`let fixed = Prop::value(constraints.disabled).fixed().is_some() && Prop::value(constraints.hidden).fixed().is_some();` が真なら作らない。正しさ：定数なら初回（gesture はまだ無い）の後に再実行されない。

`crates/kivact-core/src/view/mount/part_input.rs:245-290` `route_accessibility`：`registry` が region を宣言していない時（`registry.state.borrow().groups.is_empty()` を `Registry::declares_regions()` として regions.rs に追加）は、caller の `a11y` か `registry.root.actions.a11y` に handler がある action だけを登録する（region の lookup は常に None なので、他の action の wrapper は何もしない）。宣言がある node は今どおり 11 個。

`crates/kivact-core/src/view/handler_layers.rs:57-182` `EventHandlers`（`entries` は private で、このファイルの外からは触れない）：

```rust
pub(crate) struct EventHandlers<E> {
    /// `None`: no handler (nothing allocated).
    // xtask-allow(state-fields): option: absent means the list is empty
    entries: Option<Rc<Vec<(Layer, Handler<E>)>>>,
}
impl<E> Default for EventHandlers<E> { fn default() -> Self { Self { entries: None } } }
impl<E> EventHandlers<E> {
    fn list(&self) -> &[(Layer, Handler<E>)] {
        self.entries.as_deref().map_or(&[], Vec::as_slice)
    }
}
impl<E: 'static> EventHandlers<E> {
    fn list_mut(&mut self) -> &mut Vec<(Layer, Handler<E>)> {
        Rc::make_mut(self.entries.get_or_insert_with(Rc::default))
    }
}
```
  置換規則（同ファイル内の全 20 箇所）：読み（`self.entries.partition_point`・`.iter()`・`.is_empty()`、`deliver` の `source.list.entries.iter()`、`absorb_part` の `part.entries.iter()`、`merge_caller` の `later.entries.iter()`）は `list()` 経由、`Rc::make_mut(&mut self.entries)` は `self.list_mut()`。`router` は `Some(Rc::new(vec![..]))`。`only` は集めた Vec が空なら `None`、そうでなければ `Some(Rc::new(v))`。`Clone` は `entries.clone()` のまま。Handlers の既定（`tree.rs:64` の `#[derive(Default)]`）で 6 個の空 `Rc<Vec>` を作らなくなり、`only` も空の結果で alloc しない。

### 3.11 M2-11 定数を共有する（kivact-core、kivact-ui）

- `crates/kivact-core/src/view/mount/parts.rs:160-216` `constraints`：

```rust
/// The two constant flags one mount needs at most, created on first use.
struct ConstFlags {
    owner: Owner,
    at: &'static Location<'static>,
    values: Cell<[Option<Value<bool>>; 2]>,
}
impl ConstFlags {
    fn get(&self, value: bool) -> Value<bool> {
        let mut values = self.values.get();
        let slot = &mut values[usize::from(value)];
        let v = *slot.get_or_insert_with(|| self.owner.constant(value, self.at));
        self.values.set(values);
        v
    }
}
```
  `let value = |source: InputProp<bool>| match source.fixed() { Some(b) => flags.get(b), None => source.into_value(cx.owner, at) };`（constraints の 6 値が全て定数なら constant は 2 つ）。`Owner` は `kivact_runtime::Owner`、`Value` は Copy。
- `crates/kivact-ui/src/button.rs:268-286`：`let busy_parts: Option<(Value<f64>, Value<bool>)> = busy.map(|_| busy_animation(state));`。content の recipe の state：`state.busy &= !busy_parts.is_some_and(|(_, reveal)| reveal.get());`、indicator：`busy_parts.map(|(phase, _)| busy_indicator(state, &variants, phase))`。`0.0.into_value()`・`false.into_value()` を作らない。

### 3.12 M2-12 何も disable しない button は定数の意味論にする（kivact-interaction、kivact-ui）

- `crates/kivact-ui/src/button.rs:225-227`：

```rust
let root = if busy.is_some() || disabled.is_some() {
    root.disabled(move || {
        busy.is_some_and(|busy| busy.get()) || disabled.is_some_and(|d| d.get())
    })
} else {
    root
};
```
- `crates/kivact-interaction/src/toggle.rs:79-91` `ButtonApi::control` の part の apply：

```rust
let host = host.part(input).role(Role::Button).focusable(true);
if caller_disabled.is_none() && disabled.get_untracked().is_none() {
    host
} else {
    host.disabled(move || {
        caller_disabled.is_some_and(|value| value.get())
            || disabled.with(|d| d.is_some_and(|d| d.get()))
    })
}
```
  apply は mount 時（setup の後）に走る。`ButtonApi::disabled` は「Setup only」と文書化済みで、repo の全呼出し（`card.rs:168`、`breadcrumb.rs:186`、`navigation_link.rs:59` 他）は `control()` の前に呼ぶ。`ButtonApi::disabled`（`:66-71`）の先頭に `kivact_core::custom::require_setup("ButtonApi::disabled");` を足して契約を強制する（setup 外の呼出しは今も「Setup only」違反）。

**効果**：button の constraints が全て定数になり、constraints の memo 4 個と semantics の effect が消え、U23 の保持が button では 0 になる。

### 3.13 M2-13 葉の測定記録の容量（kivact-layout）

`crates/kivact-layout/src/tree.rs:122-126`：push の前に `record.sizes.reserve_exact(1);`。記録は最大 16 で、測定（text の shaping）より再確保の方が十分安い。text 1 つで 0.6 KiB 前後。

### 3.14 M2-14 reactive node の deps/subs を 2 つまで inline にする（kivact-runtime）

**新ファイル** `crates/kivact-runtime/src/reactive/links.rs`：

```rust
//! Role: The links of one graph node (what it read, or who reads it): up to two inline.
use crate::reactive::arena::Key;

pub(crate) type Link = (Key, u32);

/// A small vector of links: two inline, more on the heap.
pub(crate) enum Links {
    Inline { len: u8, items: [Link; 2] },
    Heap(Vec<Link>),
}
const NONE: Link = (Key { slot: 0, generation: 0 }, 0);
impl Default for Links { fn default() -> Self { Links::Inline { len: 0, items: [NONE; 2] } } }
impl std::ops::Deref for Links {
    type Target = [Link];
    fn deref(&self) -> &[Link] {
        match self { Links::Inline { len, items } => &items[..usize::from(*len)], Links::Heap(v) => v }
    }
}
impl std::ops::DerefMut for Links {
    fn deref_mut(&mut self) -> &mut [Link] {
        match self { Links::Inline { len, items } => &mut items[..usize::from(*len)], Links::Heap(v) => v }
    }
}
impl Links {
    pub(crate) fn push(&mut self, link: Link) {
        match self {
            Links::Inline { len, items } if usize::from(*len) < 2 => { items[usize::from(*len)] = link; *len += 1; }
            Links::Inline { items, .. } => { let mut v = Vec::with_capacity(4); v.extend_from_slice(&items[..]); v.push(link); *self = Links::Heap(v); }
            Links::Heap(v) => v.push(link),
        }
    }
    pub(crate) fn swap_remove(&mut self, index: usize) -> Link {
        match self {
            Links::Inline { len, items } => { let last = usize::from(*len) - 1; items.swap(index, last); *len -= 1; items[last] }
            Links::Heap(v) => v.swap_remove(index),
        }
    }
    pub(crate) fn remove(&mut self, index: usize) -> Link {
        match self {
            Links::Inline { len, items } => { let n = usize::from(*len); let link = items[index]; items.copy_within(index + 1..n, index); *len -= 1; link }
            Links::Heap(v) => v.remove(index),
        }
    }
}
impl IntoIterator for Links {
    type Item = Link;
    type IntoIter = std::vec::IntoIter<Link>;
    fn into_iter(self) -> Self::IntoIter {
        match self { Links::Inline { len, items } => items[..usize::from(len)].to_vec().into_iter(), Links::Heap(v) => v.into_iter() }
    }
}
```
- `crates/kivact-runtime/src/reactive/data.rs`：`SourceNode.subs`・`MemoNode.subs`・`Computation.deps` を `Links` に。`subs_mut()` の戻り値を `Option<&mut Links>` に。`subs()`・`deps()` は `&[Link]`（deref）のまま。
- `graph.rs` 他の使用は `iter`・`len`・`get`・`get_mut`・`push`・`swap_remove`・`remove`・`is_empty`・`std::mem::take`・by value の `for`（1.7 節の棚卸し：deps 22 箇所、subs 20 箇所）で、上の API で全て足りる。`Key` は `pub(crate)` で `slot`・`generation` は `pub(crate)`（`arena.rs:18-24`）。
- 大きさ：`Links` 32 B（Vec 24 B より +8）。arena の slot 160 → 最大 176 B。deps/subs が 2 以下の node（大半）で heap の alloc を 1〜2 個省く。

### 3.15 M2-15 kv_heap と run.sh（dev/bench）

- `dev/bench/run.sh:27-33` `build()`：exe の複製の後に次を足す（exe が埋め込む PDB 名は `$bin.pdb`。strip する他の build には PDB が無い）：

```bash
  # Windows: the exe names its PDB by file name only (/PDBALTPATH:%_PDB%) and
  # the symbol resolver looks beside the exe, so the PDB travels with it.
  if [ -n "$exe" ] && [ -f "$target/release/$bin.pdb" ]; then
    cp "$target/release/$bin.pdb" "$kpi/$bin.pdb" || invalid "missing PDB $bin.pdb after: cargo build --release $*"
  fi
```
- `run.sh:100-104`（kv_heap の結果）：各 mode の最初の `H site` 行の名前が空なら INVALID にする：`echo "$out3" | grep -m1 '^H site' | grep -qE 'allocs [^ ]' || invalid "kv_heap mode=$mode: allocation sites unresolved (kv_heap.pdb beside $heap?)"`。
- `dev/bench/src/bin/kv_heap.rs:122-138` `owner()`：「名前に kivact を含む」を「**関数の path が `kivact`/`arvane` で始まる**（`n.starts_with("kivact") || n.starts_with("arvane")`、`kivact_bench` を除く）」に変える（generic 引数に型名を含む std の関数を site にしない）。環境変数 `CHAIN`（既定 1）で最初の N 個の kivact/arvane 関数を ` <- ` でつないだ名前を site にする（同じ名前の連続は 1 つ）。`TOP`（既定 40）で出力行数を変える。README の kv_heap の行に `CHAIN`・`TOP` を 1 文足す。
- `kv_heap.rs:183` の row を `kv_stress` と同じ `row((..)).style(style().gap(4))` にする（WS の行と heap の行を同じ定義にする）。

---

## 4. 見積り（目標との対比）

### 4.1 試作 P1 と確定設計の差

P1 に入っているもの：M2-1（View の群・空）、M2-2（for_each・for_index の行、virtual_list は無し）、M2-3、M2-8 の rect/scroll の遅延と「bind で observation を作らない」（readers の require は無し）、M2-9 の focus の半分（`focus_visible` が `focused` を再利用、memo 1 個減）、M2-12、局所 paint を**全く作らない**近似（M2-6 は callback のある node には作るが、計測の画面に callback は無いので同じ）。3.4 MNT は mount の計測だけ追加。P1 に無いもの：M2-4（WS には無関係）、M2-5、M2-7（button では M2-12 で不要）、M2-9 の残り、M2-10、M2-11、M2-13、M2-14、CoreNode.observation の box。

### 4.2 K2 item（WS KiB、CPU）

| 項目 | 本機 base | P1 実測 | P1 からの追加の削減（見積り、根拠は P1 の heap） | 確定設計の見積り | 目標 |
|---|---|---|---|---|---|
| box | 11.99 | 6.31 | M2-5 box の Layout 共有 −0.7、CoreNode −0.1、M2-10 Handlers −0.1 | **≈ 5.4** | ≤ 6.50 ✓ |
| text | 10.41 | 6.49 | M2-13 −0.6、M2-5b −0.45、CoreNode −0.1 | **≈ 5.3** | ≤ 3.64 ✗（8 章 H1） |
| button | 66.69 | 35.84 | M2-5 −6.1（Layout×3・Paint・Text 3.0 KB と StateStyles 3.15 KB）、M2-10 −1.5（reconcile・cancel effect、a11y handler 11→2、only/router の空 Rc）、M2-9 残り −0.6（memo 2）、M2-11 −0.6（定数 6）、M2-13 −0.6、M2-5b −0.45、CoreNode −0.35、M2-14 −1.2 | **≈ 24.4** | ≤ 24.75 ✓（余裕 1.4%） |
| row | 85.48 | 46.71 | button の分 −11.4、row の Layout 共有 −0.7、text の M2-13・M2-5b −1.0、CoreNode −0.2 | **≈ 33.4** | ≤ 34.45 ✓（余裕 3%） |

- button の根拠（heap、P1）：`Writer::set` 4304 のうち Semantics 2×416 を除く 3,472 B と `StateStyles` の 3,152 B は、3000 個とも同じ値なので M2-5 で各 1 個になる。M2-10 は `root_target` 728・`route_accessibility` 384＋closure・`EventHandlers::only` 320・`router` 160 と effect 2 個（~600）から。reactive 1 個 ≈ 300〜350 B（arena 160 B＋2 冪の余り、cell・closure、deps）。M2-14 は deps/subs の alloc 22〜38 個の大半を省き、slot の +16 B を差し引く。
- 誤差：各項目 ±1.5 KiB と見る（P1 の WS は 1 回測定、allocation の数の減少が断片化を通して WS をさらに減らす方向の不確かさもある）。button と row は**目標の境界**にある。届かなかった場合の追加策を 8 章 H3 に用意した。
- 3000 行 idle WS（CPU、`MODE=row`）：P1 176,296 − 13.3 KiB × 3000 ≈ **136,400 KiB**（≤ 158,479 ✓）。default renderer は GPU を起動しない画面で CPU と同じ値になる（B1 で 292,776 vs 292,328）ため ≈ 136 MB で、目標 129,989 には ws0（ROWS=0 の 35.7 MB）が **≈ 29.8 MB 以下**になる必要がある（counter idle の削減は他の担当。8 章 H2）。
- live_reactive（stats_n3000）：B1 138,130 → P1 69,081 → 確定設計 ≈ 51,000（1 行 ≈ 17）。MEM-6 の目標（≤ 0.6×B0 = 100,890）を満たす。tree_nodes 36,023 → 15,012（P1 実測、1 行 5）。

### 4.3 K5 mount n3000

- 本機 base 1543〜1651 ms（B1 1507、B0/B1 = 1.08 なので本機の B0 ≈ 1,670、目標の本機相当 ≈ 835 ms）。
- P1 942〜975 ms、P1 + M2-4 **830〜837 ms**（実測）。
- 残りの削減（見積り）：reactive 1 行 −6 個（M2-9・M2-10・M2-11 の memo・effect・定数）と allocation 1 行 −40 個前後（M2-5・M2-14・M2-10）。P1 で reactive −69k・node −21k が −585 ms（+MNT）だったことから比例で −30〜50 ms。M2-5 で prop の box の確保が消える分（`w.set_prop` 47 ms の一部）。→ **≈ 780〜800 ms**（目標 813.8 ✓、余裕 2〜4%）。
- 他に O(N²) は無いことを確認した：`ChildOrder::insert` は linked の O(1)（`tree/child_order.rs:35-66`）、`NodeTree::insert`（`node_tree.rs:446-471`）は O(深さ)、`Model::insert` の `text_props`・`forget_text_below` は O(深さ + 挿入した subtree)、`pack_child_orders` は commit ごとに一度 O(子)。残る大きな固定費は `m.bind_style`（style の compile、13.4 µs/styled node、163 ms）、最初の `layout.compute`（146 ms）と test driver の a11y の全構築（130〜139 ms）で、node 数に比例して既に減っている。

### 4.4 その他の KPI への影響

- K3/K4/K5 の更新系：node 数 −58% は layout・paint list・a11y の全走査がある経路を比例で軽くする（全走査そのものの除去は他の担当）。M2-4 は keyed list の並べ替えを O(移動数) にする。
- K6：コードの増減は小さく、影響は無視できる。
- GPU：無関係。

---

## 5. 正しさ・リスク・E2E

| 項目 | リスク | 既存 E2E での担保 | 追加すべき E2E |
|---|---|---|---|
| M2-1 平坦化 | 群の子が親の直下に来ることを前提にしない処理（stack の layer は `View::mount` の fragment を見るので不変）。DevTools の要素ツリーから匿名の Contents が減る | `layout::display_contents_lays_out_children_in_the_parent`、`flows::*`、`ui_standard::a_button_is_a_named_button_that_enter_space_click_and_invoke_press`、試験の query は Contents を投影する（`kivact-test/src/screen/query.rs:18`） | `view::groups_and_empty_views_mount_no_wrapper`：`row((text("a"), None::<View>, vec![text("b"), text("c")], ()))` を mount し、`stats().tree_nodes` が 4 増え（row・a・b・c）、children の順が a, b, c |
| M2-2 行の fragment | 行の根に exit・layout_id がある時の削除と移動 | `flows::for_each_rows_are_plain_functions_and_keep_state_by_key`、`flows::index_each_rows_follow_their_index`、`flows::virtual_list_*`、`presence::*`、`resource_reclaim::closing_three_thousand_rows_returns_the_live_counts_to_the_baseline` | `flows::reordered_rows_with_exit_styles_leave_and_move_by_their_own_node`：exit style を持つ row の並べ替えと削除 |
| M2-3 queue | なし（受け渡しの順序は同じ） | 全ての E2E | engine 単体試験（3.3 節） |
| M2-4 facts | Contents の子の facts が兄弟に依存していた場合の描画漏れ | `style_paint::*`、`transform_3d::*`、`effects_filters::*`、`flows::*` | `flows::inserting_into_a_contents_list_redraws_the_new_row`（3D・blend を持つ行を for_each の途中に挿入し、paint list に現れる） |
| M2-5 共有 | 共有値をその場で変える経路（無いことを確認済み）。`Send` 要求 | `presence::exit_size_collapses_the_node_before_it_is_removed`（`transitions.rs:256`）、`hover::*`（状態 style）、`ui_theming::*` | 不要（値の意味は不変） |
| M2-6 局所 paint | callback が読む signal の変化で style effect 全体が再実行（重いが正しい） | `style_paint::paint_only_style_tracks_size_signals_theme_and_removal`、`style_paint::merged_paint_runs_between_caller_draw_and_children_and_clears_independently` | `style_paint::a_paint_callback_acquired_later_draws_at_the_node_size`：最初は callback 無し、signal で callback を足し、node の大きさで描かれる |
| M2-7 plan | 前半の patch が実は reactive な値を読んでいた場合（`reactive` は自分の source で判定するため無い） | `relations::*`、`reactive_props::reactive_component_props_update_without_remounting`、`interaction_toggle::each_kind_publishes_its_role_and_the_checked_state_without_a_value_text`、`environment::advanced_semantics_publish_roles_and_reactive_states` | `relations::constant_semantics_publish_once_and_keep_no_publisher`：定数の role/name だけの node で `live_reactive` が増えない |
| M2-8 遅延 | API を呼ぶ前に bind した node への observation の後付け | `node_ref::scroll_state_follows_the_scroll_position`、`node_ref::image_status_reports_a_failed_image`、`scroll_timeline::*`、`focus::unmounted_node_is_not_focused_and_a_rebound_ref_follows_the_new_node` | `node_ref::rect_called_after_binding_follows_the_node`：mount 後の handler で `rect()` を初めて呼び、次の layout で値が入る |
| M2-9 | focus_visible の判定 | `focus::focused_follows_tab_click_and_focus_requests`、`hover::hovered_follows_the_hit_through_listeners_capture_and_tree_changes`、`parts::*` | 不要 |
| M2-10 | region 無しで reconcile を省く。DevTools の `handler_ran` に無処理の a11y action が出なくなる | `parts::*`、`canvas`（region）、`devtools::*` | 不要 |
| M2-11/12 | `ButtonApi::disabled` を setup 外で呼ぶ利用者（repo には無い） | `ui_standard::a_disabled_button_is_disabled_and_ignores_every_way_of_pressing_it`、`kivact-ui-soft` の consumer `button_busy` | 不要 |
| M2-13/14 | runtime の依存の整合（swap_remove の index の付け直し） | runtime の既存単体試験、`machine::*`、`reactive_props::*`、`disposal::*` | 不要（既存の graph 試験で担保） |

境界条件：
- 削除・unmount：行の node が群の fragment から根の node に変わっても `unmount_branch` は node と owner を消す。observation・局所 paint の size signal は node の owner に属し、owner の破棄で消える。
- 非表示・reduced motion：変更なし（busy の reduced motion は M2-11 で経路が変わらない）。
- window close：`RecentProps` は Model と一緒に捨てる。
- DevTools：style layers の記録（`set_style_layers`）は不変。要素ツリーから匿名の fragment が減る。

---

## 6. unit 分割案

所有ファイルは unit 間で重複しない。確認は軽いものだけ（E2E は名前で 1〜3 個、window-lock 経由）。

| unit | 内容 | 所有ファイル | 依存 | 完了時の確認 |
|---|---|---|---|---|
| U-M1 | M2-1、M2-2 | `kivact-core/src/view/descriptor/description.rs`、`view/elements/builders.rs`、`view/overlays.rs`、`view/flow/list.rs`、`view/flow/virtual_list.rs` | なし | `cargo check -p kivact-core`、`cargo clippy -p kivact-core --all-targets -- -D warnings`、`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- flows::for_each_rows_are_plain_functions_and_keep_state_by_key layout::display_contents_lays_out_children_in_the_parent` |
| U-M2 | M2-3 | `kivact-engine/src/model/events.rs` | なし | `cargo test -p kivact-engine --lib model::events` |
| U-M3 | M2-4、M2-5（engine の prop） | `kivact-engine/src/model/apply.rs`、`model/aggregate.rs`、`model/recent.rs`（新）、`model.rs`、`model/states.rs`、`contract/node.rs`、`host/write/typed.rs`、`model/presence/transitions.rs` | なし | `cargo check -p kivact-engine --all-features`、`cargo clippy -p kivact-engine --all-targets -- -D warnings`、E2E `presence::exit_size_collapses_the_node_before_it_is_removed hover::hover_covers_the_children_and_group_hover_styles_a_child_by_its_groups_hover` |
| U-M4 | M2-5b | `kivact-engine/src/measure/state.rs`、`measure/leaf.rs` | U-M3（`recent.rs`） | `cargo check -p kivact-engine`、E2E `text_style::` の 1 個（`text_primitives::` でも可） |
| U-M5 | M2-13 | `kivact-layout/src/tree.rs` | なし | `cargo test -p kivact-layout` |
| U-M6 | M2-6 | `kivact-core/src/view/mount/resolve.rs`、`mount/style.rs`、`mount/draw.rs`、`mount/facts.rs` | U-M1 の後（core の衝突回避のため順に） | `cargo check -p kivact-core`、E2E `style_paint::paint_only_style_tracks_size_signals_theme_and_removal style_paint::merged_paint_runs_between_caller_draw_and_children_and_clears_independently` |
| U-M7 | M2-7 | `kivact-core/src/view/mount/semantics.rs` | なし | E2E `relations::relations_reach_the_accessibility_tree_in_call_order interaction_toggle::each_kind_publishes_its_role_and_the_checked_state_without_a_value_text` |
| U-M8 | M2-8、M2-9 | `kivact-core/src/view/node_ref.rs`、`view/ref_data.rs`、`view/part.rs`、`view/tree.rs`、`view/node_status.rs`、`view/scroll_timeline.rs`、`client/dispatch/delivery.rs`、`client/dispatch/env.rs`、`client/dispatch/text.rs` | なし | E2E `node_ref::scroll_state_follows_the_scroll_position scroll_timeline::scroll_progress_tracks_offset focus::focused_follows_tab_click_and_focus_requests` |
| U-M9 | M2-10、M2-11（core） | `kivact-core/src/view/mount/regions.rs`、`mount/part_input.rs`、`mount/parts.rs`、`view/handler_layers.rs` | なし | E2E `parts::node_parts_keep_customizations_first_and_observe_consumed_input parts::canvas_parts_have_independent_focus_actions_input_state_and_geometry_lifetimes` |
| U-M10 | M2-11（button）、M2-12 | `kivact-interaction/src/toggle.rs`、`kivact-ui/src/button.rs` | なし | E2E `ui_standard::a_disabled_button_is_disabled_and_ignores_every_way_of_pressing_it`、`cargo test -p kivact-ui-soft --test consumer -- button_busy` |
| U-M11 | M2-14 | `kivact-runtime/src/reactive/links.rs`（新）、`reactive/data.rs`、`reactive/graph.rs`、`reactive.rs`（mod）と deps/subs を触る runtime 内の他ファイル | なし | `cargo test -p kivact-runtime` |
| U-M12 | M2-15 | `dev/bench/run.sh`、`dev/bench/src/bin/kv_heap.rs`、`dev/bench/README.md` | なし | `cargo build --release -p kivact-bench --bin kv_heap`（PDB 設定込み）のうえ、kpi フォルダへ exe と PDB を複製して `MODE=button ROWS=10 SETTLE_SECS=1` で site 名が出ること（window-lock 経由） |

checkpoint（重い確認）：workspace の test と全 E2E、`dev/bench/run.sh` の K2・K5（本機で base と比較）、`kv_census` 相当の stats（`kv_latency` の `S stats_n3000`）で tree_nodes ≈ 15,012・live_reactive ≈ 51,000。

---

## 7. 既存欠陥（Slim-Fast 実装 79bbd409..db8d679b）

1. **M-R4（EVT-1）が働かない**：`crates/kivact-engine/src/model/events.rs:32-43`。縮小は drain の中だけで、条件は「容量 > 4×max(今回の長さ, 1024)」。mount の burst を drain する時は長さが大きいので縮小せず、その後のアイドルでは `has_events()` が偽で drain されない（`host/pump.rs:488-511`）。再現：`kv_heap MODE=box ROWS=3000` で `VecDeque<HostEvent>::grow ← model::events` が 1,309 B/item 生存（button 5,242、row 10,485）。修正：3.3 節。
2. **mount の O(N²)**：`crates/kivact-engine/src/model/apply.rs:508`（と `:66`、`:462`）。透明な parent への insert ごとに parent の flat の子を全部集める。再現：n=3000 の for_each で `w.insert` 平均 13.3 µs（P1）。修正：3.4 節（2.0〜2.3 µs に低下を確認）。
3. **kv_heap の site 名が空・誤帰属**：`dev/bench/run.sh:32`（PDB を複製しない、実験で確定）と `dev/bench/src/bin/kv_heap.rs:135`（generic 引数の部分一致で std 関数を site にし、ICF で別型の名前になる）。修正：3.15 節。
4. **button と use_button が常に computed の disabled を付ける**：`crates/kivact-ui/src/button.rs:225-227`、`crates/kivact-interaction/src/toggle.rs:85-89`。何も disable しない button でも constraints が reactive になり、memo 4 個と semantics の effect（patch の Vec 1,280 B と `previous` を保持）が残る＝U23 の保持の直接の原因。修正：3.12 節。
5. **MEM-6 の目標未達の未報告**：設計書 4.7 MEM-6 の「3000 行の live_reactive は B0 の 0.6 倍以下」は B1 で 0.82 倍（138,130 / 168,150）だが、RESULT・OPEN に記録が無い。本設計で ≈ 0.30 倍。
6. **kv_heap と kv_stress の row の定義が違う**：`kv_heap.rs:183` は `row((..))`、`kv_stress.rs:15-17` は `.style(style().gap(4))` 付き（heap の診断値が WS の行と別物）。修正：3.15 節。
7. （推測、軽微）`NodeRef::focus_visible()`（`node_ref.rs:634-647`）は呼ぶたびに `focused()` の memo をもう 1 つ作る。公開 API の利用者が両方を呼ぶと memo が 3 個になる。本設計は part 内部だけを直す（3.9 節）。公開 API は今のまま（同じ値の memo の重複でメモリだけの問題）。

---

## 8. 設計者（人間）の判断が要る点

- **H1 text item（≤ 3.64 KiB）は本設計では届かない**：見積り ≈ 5.3 KiB（B0 14.57 から 2.7×、目標 4×）。残りは 1 host node の固定費（layout 列 ≈ 1.0 KB：うち `taffy::Cache` ≈ 0.5 KB、`HostNode` 472 B、CoreNode、paint/measure 列）、`Semantics` 416 B（name が行ごとに違うので共有できない）、行の owner（`OwnerEntry` 200 B）と行の source、text の shaping cache ≈ 0.7 KB。さらに削るには (a) `Semantics` を疎な表現にする（`kivact_types` の公開型の変更、−0.3 KiB）、(b) `OwnerEntry` の稀な Vec を box にする（−0.15）、(c) engine の node 列の再設計（`taffy::Cache` を必要な node だけに持つ等、kivact-layout の taffy 統合の変更、−0.4〜0.6）が要り、全部でも ≈ 4.3〜4.5 KiB と見積もる。目標を維持して (a)〜(c) を次の段階の設計にするか、目標を見直すかの判断を仰ぐ。
- **H2 3000 行 idle WS（default ≤ 129,989 KiB）**：本設計の行 ≈ 33.4 KiB/行では ws0（ROWS=0、本機 35.7 MB）が ≈ 29.8 MB 以下である必要がある。counter idle WS の目標（≤ 27,214 KiB）が他の担当で達成されれば満たす見込み。CPU の目標（≤ 158,479）は本設計だけで満たす。
- **H3 button・row の余裕が小さい（1〜3%）**：checkpoint で未達だった場合の追加策を順に：(1) 固定の `Style`（`helpers::label_style`・`single_line_text`）を thread_local の `Rc<Style>` で共有し layer の closure が clone を返す（−0.3 KiB/button）、(2) `Semantics` を疎にする（H1 (a)、button は 2 node で −0.6）、(3) style effect の compile 結果を「同じ `Style`・theme・状態なら直近の結果を共有」（M2-5 と同じ ring、mount −10〜15%、メモリへの直接効果は小）。どこまで先に入れるか。
- **H4 M2-6 の性能の振舞いの変更**：自前の draw を持たない box の `Style::paint` callback は style effect が記録するため、callback が読む signal の変化・resize で style の compile も再実行される（callback を持つ node は稀という前提）。許容するか。
- **H5 M2-15 の `CHAIN`・`TOP`**：計測器の診断出力の追加で、CI 機能ではない。kv_heap の既定の出力（`CHAIN=1`）は site の選び方を直すので B1 の site 名とは一致しない（B1 の heap 診断は site 名が空だったので比較の意味は失われない）。

---

付記：計測に使った試作の差分（計測用の env 切替や prof span を含み、製品の実装ではない）と生データは `W:\w\kv-mem` の未追跡フォルダ `.p3-measure\`（`worktree.diff`、`heapout\`・`hp1\`（kv_heap）、`census\`、`ws-*.txt`）に残した。worktree は残し、`W:\w\kv-mem-target` は削除した。
