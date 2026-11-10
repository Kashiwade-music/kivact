# P2 設計：a11y の構築・公開、pointer 移動・wheel の hit test

> **V2 付属書 D2**（設計担当 P2 の成果物）。正本は [SLIM_FAST_V2_DESIGN.md](../SLIM_FAST_V2_DESIGN.md)、手順は [SLIM_FAST_V2_WORK_ORDER.md](../SLIM_FAST_V2_WORK_ORDER.md)。
> - 本書の「8 章 設計者（人間）の判断が要る点」は V2 設計書 4 章（DEC-01〜DEC-23）で決定済み。実装者は 8 章を読んで選ばない。V2 設計書 5 章（付属書間の調整）が本書を上書きする箇所がある。
> - 本書の unit 名と V2 の unit ID の対応は V2 設計書 0.2 節。unit の実施順・lane・wave は V2 作業指示書だけが決める（本書 6 章の「依存」「並列可」は参考）。
> - 文中の P1〜P8 は付属書 D1〜D8（`design-P<n>.md` は `D<n>-*.md`）を指す。`W:\w\kv-prof`・`W:\w\kv-mem`・`W:\w\kv-size`・scratchpad の path は設計時の試作・計測の場所（設計した機械にだけある参考）で、実装に必要ではない。行番号は HEAD `73c36c7c` のもの。

担当 KPI：K5 pointer 移動 p95 n3000（≤ 10.60 ms）、K5 wheel n3000（≤ 16.29 ms）、K5 unrelated text 更新（≤ 9.46 ms）・row 更新（≤ 16.96 ms）の a11y 部分。
基点：HEAD 73c36c7c（= db8d679b + 引継ぎ文書）。**計測は行っていない**（静的解析のみ。数値は引継ぎ付録 B の prof 診断値と B1）。

表記：「prof」は引継ぎ付録 B の prof profile 値（元機）。「B1 機」は B1 と同じ元機の release 値。prof→B1 の換算比は、pointer（prof 推定 56.1 ms に対し B1 89.05 ms）と wheel（prof 13.49 ms に対し B1 22.02 ms）から **約 1.6 倍**とする（両者で 1.58 と 1.63）。

---

## 1. 事実

### 1.1 計測（引継ぎ 5.1 節・付録 B、pointer 場面 n3000、28 回の redraw／120 回の移動）

| span | mean | 内訳・意味 |
|---|---|---|
| `test.redraw` | 41.01 ms/回 | = `redraw.present` 12.44 + `a11y.build` 26.35 + `a11y.hash_submit` 2.19（残差 0.03） |
| `redraw.present` | 12.44 | `comp.sync_playback` 5.64 + `comp.sync_shader` 5.24 + `present.raster` 1.27 + `comp.paint_list` 0.15 + `comp.compose_list` 0.12 + `pl.*` ほか（P1 の領域） |
| `a11y.build` | 26.35 | **全 28 回が `a11y.path.metadata`**（`a11y.path.geometry`・`local_text` は 0 回）。`identity_metadata` 14.87 + `diff_metadata` 4.64 + `tail` 6.83（`semantic_order` 2.40 + `tail_rest` 4.42） |
| `a11y.hash_submit` | 2.19 | 後述 1.4：Metadata 経路の受理時の O(N) の retain と旧 Metadata の破棄 |
| `test.wake` / `turn.all` / `core.turn` | 1.66 / 1.65 / 1.65 ms（n=120、移動ごと） | core の turn。`core.dispatch` は 0.209 ms × 232 回（1 turn あたり約 0.40 ms）。残り約 1.25 ms/turn は P2 の領域外（8 章 4） |
| `wheel.hit`（wheel 場面） | 13.47 ms/入力 | `test.send` 13.49 のほぼ全部。`wheel.handle` 1.9 µs、`wheel.sync` 1.7 µs、wheel 後の `test.pump1` 0.7 µs、`test.redraw` 0 回 |

- **wheel 場面では scroll も redraw も起きていない**（`test.redraw` が出ず、`test.pump1` が 0.7 µs）。kv_latency の `list()` は `scroll(column(rows))` に高さの制約が無く、scroll container が内容の高さまで伸びるため、wheel は hit test だけで終わる。**wheel KPI ＝ hit test の時間**。
- pointer の移動点は `(40 + i%7, 80 + 3i)`（`dev/bench/src/bin/kv_latency.rs:94-99`）。ほとんどが 3000 行の scroll container の中。wheel は最後の移動点 (40, 437) で行う（同 :101-104、`crates/kivact-test/src/screen/input.rs:193-201`）。
- pointer の 1 回の移動は `mouse_to`（`input.rs:89-92`、ここで engine の `Model::mouse_moved` → `self.hit(at)`、`model/pointer/shell.rs:139`）と `run_until_idle`（`input.rs:206-207`）。prof の pointer 区間には hit の span が無いが、wheel と同じ点群なので hit は 1 回あたり約 13.5 ms（列の中の点）と推定する。よって **prof の redraw を伴う移動 ≈ hit 13.5 + wake 1.66 + redraw 41.0 = 56.1 ms**（B1 89.05 ms と比 1.59）。
- click（unrelated・row 更新）は `pointer_click`＝move・down・up の 3 回の hit（`input.rs:391-395`、`shell.rs:139,188`）。Other／Bump の button は scroll container の上にあり、`hit_box` は z 順の逆に子を見るので scroll container（clip する）を先に試すが、点が clip の外なので即座に返る（`model/interact/geometry.rs:527-529`）。**click の hit は既に安い**。click 場面の a11y は pointer と同じく Metadata 経路（2.1）。

### 1.2 a11y の dirt を立てる全ての呼出元（`A11yState` の 6 method、HEAD）

| 呼出元 file:line | 呼ぶ method | 何の変化 | 今の dirt |
|---|---|---|---|
| `model/aggregate.rs:169-174` `repaint_text` | `local_text` | 入力の text・session | LocalText |
| **`model/aggregate.rs:176-187` `repaint`** | **`node`**（+ states_place なら `subtree`） | paint・状態 style の再描画 | **Metadata** |
| `model/apply.rs:63,167,467,532,581` | `request_publication` | move・create・insert・remove | Metadata |
| `model/apply.rs:89` `set_scene` | `node` | scene（region） | Metadata |
| `model/apply.rs:105` | `subtree` | canvas drawable の root の配置 | Metadata |
| `model/apply.rs:144` `scroll_to` | `subtree` | programmatic scroll | Metadata |
| `model/apply.rs:283,354,464,468,495,507,596` | `node`/`subtree` | 平坦化・positioning・挿入・削除 | Metadata |
| `model/apply.rs:392` `changed` | `subtree` | states_place の書込み | Metadata |
| `model/apply.rs:397-415` `invalidate_semantics` | `local_text` か `node`、style/text/placed/subtree なら `subtree` | 全 typed setter | Content だけ LocalText、他は全部 Metadata |
| `model/apply.rs:436,441` | `node` | 継承 text style の下の text・image | Metadata |
| `model/interact/sync.rs:51,53` | `subtree`/`invalidate` | viewport | Metadata/Full |
| `model/interact/sync.rs:145` | `node` | font 登録 | Metadata |
| `model/interact/sync.rs:270` | `node` | image の natural size | Metadata |
| `model/interact/sticky.rs:58` | `subtree` | sticky の shift | Metadata |
| `model/interact/geometry.rs:175` | `moved` | layout の geometry 変化 | Geometry |
| `model/interact/geometry.rs:664` `set_scroll` | `moved` | wheel・drag・scroll_into_view | Geometry |
| `model/state_motion/steps.rs:322` | `subtree` | 状態 transform の毎 tick | Metadata |
| `model/states.rs:173` `restyled` | `subtree` | states_place の状態 style | Metadata |
| `model/frame/ticks.rs:110` | `invalidate` | 環境変化 | Full |
| `model/presence/transitions.rs:193` | `subtree` | 退場開始 | Metadata |
| `model/focus/navigation.rs:306-308` | `request_publication` | focus の移動 | Metadata |
| `model/drawables/request.rs:42` | `subtree` | drawable の geometry | Metadata |
| `model/layout_motion/steps.rs:187` | `subtree` | layout motion の毎 tick | Metadata |
| `host/events.rs:527` | `invalidate` | AT の起動 | Full |
| `a11y/publish.rs:84` | `invalidate` | 拒否 | Full |

`repaint` を呼ぶのは hover・pressed の反転（`model/pointer/machine.rs:181`）、group 依存（`states.rs:165` 経由）、focus 移動（`navigation.rs:270`）、状態 motion の capture（`state_motion/steps.rs:255`）、perspective の昇格（`transform.rs:187`）。

### 1.3 旧設計 4.6 A11Y-1 の実装状況

- 実装済み：`Dirt::Geometry`（`a11y/state.rs:32-35`）、`moved`（`state.rs:116-148`）、`build_geometry_semantics`（`a11y/build/dirty.rs:92-115`）、`geometry.rs:175,664` の `moved`。
- しかし (a) **Geometry 経路自体が O(N)**：`clipped_rows`（`a11y/build/node.rs:417-425`、全ノードの `tree.iter`）、`semantic_order`（`a11y/build/geometry.rs:354-362`）、`inherited` の loop（`dirty.rs:131-141`）、`expand_overlay_dirt` の `order` 走査（`dirty.rs:275`）、`build_dirty_nodes` の根からの全走査と 1 ノード 1 Vec の確保（`dirty.rs:306-343`）。(b) 同じ公開の間に 1.2 の Metadata を立てる呼出しが 1 つでもあれば Metadata に昇格する（`state.rs:188-194`）。**hover・press・focus・状態 motion は必ず `repaint` → `node` を通るため、K5 のどの場面でも A11Y-1 は効かない**。

### 1.4 公開の流れ（test driver）

- 公開は `finish_window` の末尾（`host/window_changes.rs:206-211`、paint が残っていない pump のたび）。dirt が `Clean` なら O(1) で戻る（`a11y/publish.rs:26-28`）。
- 受理時（`publish.rs:67-83`）、Metadata なら `hashes.retain(live)`（36,023 件の BTreeSet 照合）、`region_ids`・`native_regions`・`authored` の retain、`Rc::new(metadata)` による旧 Metadata（36,023 件の BTreeMap×3 と BTreeSet）の破棄。候補ノードは 2 個なので、`a11y.hash_submit` 2.19 ms の大半はこの O(N)（推定）。
- shell の適用（`arvane-shell/src/a11y/state.rs:103-178`）は変わったノード×深さ。virtual backend も同じ（`arvane-virtual/src/window.rs:370-375`）。
- **test driver 側の不要な O(N) は release には無い**：`TestApp::pump`（`crates/kivact-test/src/screen/driver.rs:159-184`）は `failure`・`exited`・`due`・`woken`・`pending`・`redraw_requested` の O(1) の判定だけ。O(N) の `self_check`（`driver.rs:179-180`、`host/testing/self_check.rs`）は `debug_assertions` のときだけで、KPI（release）には入らない。mirror は `KIVACT_TEST_MIRROR=off`。**`test.redraw` 41 ms の中身は present（P1）と a11y（P2）だけ**。

### 1.5 a11y の bounds は既に scroll に対して不変（旧設計 1.5 節の前提の訂正）

- `Model::rect`（`model/interact/geometry.rs:260-300`）は祖先の scroll を引いた window 座標。`build_node` は `bounds = rect + offset`、offset は祖先の scroll の和（`a11y/build/node.rs:73-78`、`dirty.rs:328-339`、`node.rs:38-47`）。よって **bounds は content 空間で、祖先の scroll が変わっても子孫の payload は変わらない**。変わるのは (1) scroll した container 自身の `scroll`、(2) その下の layer（overlay・fixed は window 原点に置かれるので content 空間では動く、`geometry.rs:269-281`）、(3) その container を anchor の祖先に持つ overlay、(4) clip する list の行の clip（`node.rs:432-474`）、(5) sticky の shift（`sticky.rs:54-59` が別に通知）。旧設計 1.5 の「絶対座標で公開しているため子孫すべての payload が変わる」は事実と異なる。今の `set_scroll` → `moved(id)` は subtree 全体を作り直す過剰な invalidation（hash で捨てられるが構築は O(subtree)）。

### 1.6 hit test（`model/interact/geometry.rs:447-646`）

- `hit` → layer → `hit_node(root)` → `hit_in`（`shown` は祖先を辿る O(深さ)、`to_parent`、逆行列、backface、`Look::now(..).plane_3d()`）→ `hit_box`（layout、clip path、`hits`、clip、`scrollbar_at`、`drawable_hit`、子を z 順の逆に全て `hit_in`、`exported_hit`）。**clip しない container では点が子の外でも子に降りる**ので、列の中の点では 36,023 ノード全部を訪れる（1 ノード約 370 ns → 13.47 ms）。
- paint list は各 box の **ink**（自分と子孫が描きうる範囲、親の content 空間で自分の layout location 基準、自分の transform で写したもの）を `PaintState.ink` に保持している（`paint/list/ink.rs:76-104`、無効化は `paint/list/invalidation.rs`）。描画の変化・geometry・scroll・可視性の変化で自分と祖先の ink が捨てられ（`forget_ink`）、必要時に再計算される。`drawn` でない box の ink は `None`。
- module 規則：`model` は `paint` を使えない（`dev/xtask/policy/workspace.toml:252-262`）。

---

## 2. 原因

### 2.1 pointer 移動（hover の変化）が Metadata 経路に入る理由

hover の反転 → `set_chain`（`machine.rs:126-198`）→ hover style を持つ button に `repaint(id)`（`machine.rs:181`）→ `self.a11y.node(id)`（`aggregate.rs:178`）→ `metadata_dirt()` が `Clean` を `Metadata` に上げる（`state.rs:174-180`）。**paint の再描画を「identity が変わりうる変化」として扱っていることが唯一の原因**。core 側の書込み（button の recipe style の再評価）は値が同じなので engine の `set_prop` が `false` を返し（`model/apply.rs:209-211`、`view/tree.rs:653`）、a11y を汚さない。

click（unrelated／row 更新）も同じ：down・up の pressed の反転 → `repaint` → Metadata。最初の click では focus 移動の `request_publication`（`navigation.rs:307`）も Metadata。text の `Content` は LocalText だが同じ公開に Metadata が混ざるので昇格する。

### 2.2 Metadata 経路の O(N)

`build_identity_metadata`（`dirty.rs:192-252`）：`prepare_region_ids`（`identity.rs:464-480`、全ノードの order と scene）、`clipped_rows`、`semantic_order`、全ノードの parents・children の hash（1 ノード 1 Vec）・live・relation・region・authored の BTreeMap/BTreeSet 挿入。続いて全ノードの parents/children の比較（`dirty.rs:72-82`）、`build_semantics_tail` の `semantic_order` 再計算と全走査（1.3）。受理時の retain と破棄（1.4）。

### 2.3 局所更新を妨げる invalidation（まとめ）

1. `repaint` → `node`（2.1）。
2. 幾何だけの変化なのに `subtree`（Metadata）を使う呼出元：`sticky.rs:58`、`state_motion/steps.rs:322`、`layout_motion/steps.rs:187`、`states.rs:173`、`apply.rs:105,144,392`、`aggregate.rs:183`、`drawables/request.rs:42`。
3. payload だけの書込み（Paint・TextStyle・States・TextSpans・Placeholder・Image）を `invalidate_semantics` が Metadata にする（`apply.rs:397-415`）。継承 text の下（`apply.rs:436,441`）、image の natural size（`sync.rs:270`）も同様。
4. focus 移動の `request_publication`（`navigation.rs:307`）。
5. Geometry 経路自体の全走査（1.3）。

### 2.4 wheel・pointer の hit test

hit_box が子を「点が子孫に届きうるか」で枝刈りしない（1.6）。

---

## 3. 設計（確定版）

方針：**identity metadata（親子・relation・region・authored run・modal）が変わらない変化は全て「payload の dirt」として、変わったノードと祖先・動いた subtree・影響する clip list の行だけを作る**（PERF-1）。Metadata 経路（構造の変化）と Full は今のまま残す（8 章 3）。hit test は paint の ink で子を枝刈りする。test driver は変えない（1.4 のとおり不要な O(N) が無い）。

### 3.1 状態の型（`crates/kivact-engine/src/a11y/state.rs`）

暗黙の状態「`publish` が Published でないときの `dirt`」を型で閉じる（dirt は Published の中だけに存在する）。これで AT の非活性中に dirt が溜まる既存欠陥（7 章 3）も消える。

```rust
//! Role: Accessibility identities, accepted fingerprints and pending invalidation.
//!
//! Payload ownership belongs exclusively to the shell. IDs may be allocated
//! before acceptance; fingerprints and pending dirt commit only after acceptance.
//! Pending dirt exists only while a tree is published: before the first tree
//! and while no assistive technology reads the window, the next publication is
//! a complete tree, so nothing is kept pending.
use kivact_types::{NodeId, RegionId, SemanticsId};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

/// The rows of the clipping list-like containers that lie clipped out of
/// them (hidden from assistive technology), by list. The rows of two lists
/// never meet: a list's rows end at the first node with a role, and a list
/// has one.
#[derive(Clone, Default, PartialEq)]
pub(super) struct Clipping {
    by_list: BTreeMap<NodeId, BTreeSet<NodeId>>,
    /// Projection: the union of `by_list`, written only by [`Clipping::set_list`].
    rows: BTreeSet<NodeId>,
}

impl Clipping {
    /// Every clipped row.
    pub(super) fn rows(&self) -> &BTreeSet<NodeId> {
        &self.rows
    }

    /// Whether `list` clips rows out.
    pub(super) fn has_list(&self, list: NodeId) -> bool {
        self.by_list.contains_key(&list)
    }

    /// Replace the clipped rows of `list` (empty: it clips none); the rows
    /// whose clipping changed.
    pub(super) fn set_list(&mut self, list: NodeId, clipped: BTreeSet<NodeId>) -> Vec<NodeId> {
        let old = if clipped.is_empty() {
            self.by_list.remove(&list)
        } else {
            self.by_list.insert(list, clipped.clone())
        }
        .unwrap_or_default();
        for row in old.difference(&clipped) {
            self.rows.remove(row);
        }
        self.rows.extend(clipped.difference(&old).copied());
        old.symmetric_difference(&clipped).copied().collect()
    }
}

#[derive(Clone, Default)]
pub(super) struct Metadata {
    pub clipping: Clipping,          // 旧 `clipped: BTreeSet<NodeId>` を置換
    pub parents: BTreeMap<NodeId, Option<NodeId>>,
    pub children: BTreeMap<NodeId, u64>,
    pub reverse: BTreeMap<NodeId, BTreeSet<NodeId>>,
    pub regions: BTreeSet<(NodeId, RegionId)>,
    pub authored: BTreeSet<(NodeId, u64)>,
    pub live: BTreeSet<u64>,
    pub modal: Option<NodeId>,
}

/// The payloads to build again over the accepted identity metadata.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Payloads {
    /// Nodes whose own payload changed (a text, a paint, state styles, a
    /// scroll offset, an image).
    pub(in crate::a11y) nodes: BTreeSet<NodeId>,
    /// Nodes that moved with their subtrees: every payload under them is
    /// built again (bounds are content-space and absolute).
    pub(in crate::a11y) subtrees: BTreeSet<NodeId>,
    /// Nodes that moved, scrolled, took or lost the focus or changed their
    /// clip: the clipping lists at or above them look at their rows again.
    pub(in crate::a11y) clip_sources: BTreeSet<NodeId>,
}

/// What changed since the accepted tree.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Dirt {
    /// Nothing.
    Clean,
    /// Only payloads: the accepted identity metadata (hierarchy, relations,
    /// regions, authored runs, modal) is reused; work in proportion to the
    /// payloads.
    Payload(Payloads),
    /// The identity and order metadata is enumerated again; besides what it
    /// finds, `nodes` (own payloads), `subtrees` (whole subtrees) and
    /// `removed` are dirty.
    Metadata {
        nodes: BTreeSet<NodeId>,
        subtrees: BTreeSet<NodeId>,
        removed: BTreeSet<NodeId>,
    },
    /// A complete replacement (a new root, a rejected publication, an
    /// environment change).
    Full,
}

/// Whether and what the window's platform tree holds.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum A11yPublish {
    /// No assistive technology reads the window: nothing is built or pending.
    Inactive,
    /// Activated: the next publication is a complete tree.
    AwaitingFirstTree,
    /// The platform holds the accepted tree of `root`; `dirt` is what
    /// changed since.
    Published { root: SemanticsId, dirt: Dirt },
}

#[derive(Clone)]
pub(crate) struct A11yState {
    pub(super) native_regions: BTreeMap<u64, (NodeId, RegionId)>,
    pub(super) region_ids: BTreeMap<(NodeId, RegionId), u64>,
    pub(super) ids: SemanticsIdAllocator,
    pub(super) authored: BTreeMap<(NodeId, u64), u64>,
    pub(super) hashes: BTreeMap<u64, u64>,
    pub(super) publish: A11yPublish,      // 旧 pub(crate)。外は method で触る
    pub(super) metadata: Rc<Metadata>,
    #[cfg(feature = "testing")]
    pub(crate) constructed: u64,
    #[cfg(feature = "testing")]
    pub(crate) published: u64,
    /// Nodes the identity enumeration visited (one per node per Metadata or
    /// Full build).
    #[cfg(feature = "testing")]
    pub(crate) enumerated: u64,
}
// `dirt` field は削除。Default は publish: Inactive、enumerated: 0（他は今と同じ）。
```

`impl A11yState`（旧 `invalidate`/`local_text`/`moved`/`node`/`subtree`/`request_publication`/`metadata_dirt`/`publication_pending` を置換）：

```rust
impl A11yState {
    /// Assistive technology started reading the window; whether it was not
    /// (the next publication is then a complete tree).
    pub(crate) fn activate(&mut self) -> bool {
        if self.publish != A11yPublish::Inactive {
            return false;
        }
        self.publish = A11yPublish::AwaitingFirstTree;
        true
    }

    /// Assistive technology stopped reading the window: nothing is kept
    /// pending or accepted (an activation hands over a complete tree).
    pub(crate) fn deactivate(&mut self) {
        self.publish = A11yPublish::Inactive;
        self.hashes.clear();
        self.metadata = Rc::default();
    }

    /// The next publication is a complete tree.
    pub(crate) fn invalidate(&mut self) {
        if let A11yPublish::Published { dirt, .. } = &mut self.publish {
            *dirt = Dirt::Full;
        }
    }

    #[cfg(all(debug_assertions, feature = "testing"))]
    pub(crate) fn publication_pending(&self) -> bool {
        !matches!(self.publish, A11yPublish::Published { dirt: Dirt::Clean, .. })
    }

    /// The pending dirt (`None`: a complete tree is pending, or none is
    /// published), raised from `Clean` to an empty payload dirt.
    fn pending(&mut self) -> Option<&mut Dirt> {
        let A11yPublish::Published { dirt, .. } = &mut self.publish else {
            return None;
        };
        if *dirt == Dirt::Clean {
            *dirt = Dirt::Payload(Payloads::default());
        }
        if *dirt == Dirt::Full {
            return None;
        }
        Some(dirt)
    }

    /// Only the payload of `id` changed: a text, a paint, state styles, an
    /// image, a placeholder (not its role, relations or regions).
    pub(crate) fn payload(&mut self, id: NodeId) {
        match self.pending() {
            Some(Dirt::Payload(p)) => {
                p.nodes.insert(id);
            }
            Some(Dirt::Metadata { nodes, .. }) => {
                nodes.insert(id);
            }
            _ => {}
        }
    }

    /// `id` moved with its subtree (layout, a transform, a sticky shift, a
    /// layout or state motion, a drawable's geometry).
    pub(crate) fn moved(&mut self, id: NodeId) {
        match self.pending() {
            Some(Dirt::Payload(p)) => {
                p.subtrees.insert(id);
                p.clip_sources.insert(id);
            }
            Some(Dirt::Metadata { subtrees, .. }) => {
                subtrees.insert(id);
            }
            _ => {}
        }
    }

    /// The scroll offset of `id` changed: its own payload (its offset) and
    /// the clipping lists at or above it; `Model::a11y_scrolled` adds the
    /// layers under it.
    pub(crate) fn scrolled(&mut self, id: NodeId) {
        match self.pending() {
            Some(Dirt::Payload(p)) => {
                p.nodes.insert(id);
                p.clip_sources.insert(id);
            }
            Some(Dirt::Metadata { nodes, .. }) => {
                nodes.insert(id);
            }
            _ => {}
        }
    }

    /// The clip of `id` may have changed (its paint's `clip`): the clipping
    /// lists at or above it look at their rows again.
    pub(crate) fn reclip(&mut self, id: NodeId) {
        if let Some(Dirt::Payload(p)) = self.pending() {
            p.clip_sources.insert(id);
        }
    }

    /// The focus moved from `old` to `new`: the focus is published, and the
    /// clipping lists around both look again (a focused row stays exposed).
    pub(crate) fn focus_moved(&mut self, old: Option<NodeId>, new: Option<NodeId>) {
        if let Some(Dirt::Payload(p)) = self.pending() {
            p.clip_sources.extend(old);
            p.clip_sources.extend(new);
        }
    }

    pub(crate) fn node(&mut self, id: NodeId) { /* 今と同じ（metadata_dirt） */ }
    pub(crate) fn subtree(&mut self, id: NodeId) { /* 今と同じ */ }
    pub(crate) fn request_publication(&mut self, removed: impl Iterator<Item = NodeId>) { /* 今と同じ */ }

    /// Raise the pending dirt to `Metadata` (keeping a payload dirt's nodes
    /// and subtrees; its clip sources are dropped: the metadata build looks
    /// at every clipping list); its sets, or `None` when a complete tree is
    /// pending or none is published.
    fn metadata_dirt(&mut self) -> Option<(&mut BTreeSet<NodeId>, &mut BTreeSet<NodeId>, &mut BTreeSet<NodeId>)> {
        let A11yPublish::Published { dirt, .. } = &mut self.publish else {
            return None;
        };
        match std::mem::replace(dirt, Dirt::Clean) {
            Dirt::Clean => {
                *dirt = Dirt::Metadata { nodes: BTreeSet::new(), subtrees: BTreeSet::new(), removed: BTreeSet::new() }
            }
            Dirt::Payload(Payloads { nodes, subtrees, .. }) => {
                *dirt = Dirt::Metadata { nodes, subtrees, removed: BTreeSet::new() }
            }
            other => *dirt = other,
        }
        match dirt {
            Dirt::Metadata { nodes, subtrees, removed } => Some((nodes, subtrees, removed)),
            _ => None,
        }
    }
    // region_of / region_id（testing）は今のまま
}
```

遷移（単一の enum `A11yPublish` の中）：`Inactive →(activate) AwaitingFirstTree →(受理) Published{Clean}`、`Published{d}` の d は `Clean → Payload → Metadata → Full` の単調上昇、受理で `Clean`、拒否で `Full`（AwaitingFirstTree は拒否後もそのまま）、`deactivate` で `Inactive`（dirt ごと消える）。

### 3.2 公開（`a11y/publish.rs`、`accessibility` を置換）

```rust
pub(crate) fn accessibility(&mut self, window: &mut arvane::Window<'_>, engine: &mut arvane::Engine) {
    let (root, dirt) = match &mut self.a11y.publish {
        A11yPublish::Inactive | A11yPublish::Published { dirt: Dirt::Clean, .. } => return,
        A11yPublish::AwaitingFirstTree => (None, Dirt::Full),
        A11yPublish::Published { root, dirt } => (Some(*root), std::mem::replace(dirt, Dirt::Clean)),
    };
    let dirt = if self.tree().root().is_some_and(|id| Some(kivact_types::SemanticsId(id.0)) != root) {
        Dirt::Full
    } else {
        dirt
    };
    let Built { tree: mut candidate, metadata } = self.build_semantics_for(engine, &dirt);
    // One fingerprint per built node, used for the comparison and the commit.
    let hashes: Vec<(u64, u64)> = candidate.nodes.iter().map(|node| (node.id.0, hash::fingerprint(node))).collect();
    let replace = dirt == Dirt::Full || root != Some(candidate.root);
    if !replace {
        let accepted = &self.a11y.hashes;
        let mut at = hashes.iter();
        candidate.nodes.retain(|_| at.next().is_some_and(|(id, h)| accepted.get(id) != Some(h)));
    }
    #[cfg(feature = "testing")]
    {
        self.a11y.published += candidate.nodes.len() as u64;
    }
    let (new_root, focus) = (candidate.root, candidate.focus);
    let accepted = if replace {
        window.replace_accessibility(&candidate)
    } else {
        window.update_accessibility(&kivact_types::render::SemanticsUpdate { root: new_root, focus, nodes: candidate.nodes })
    };
    if accepted.is_err() {
        // Published: a complete tree next; awaiting the first tree: still.
        self.a11y.invalidate();
        return;
    }
    self.a11y.hashes.extend(hashes);
    match metadata {
        MetadataUpdate::Keep => {}
        // The build used an Rc clone that it dropped: no copy is made.
        MetadataUpdate::Clipping(clipping) => std::rc::Rc::make_mut(&mut self.a11y.metadata).clipping = clipping,
        MetadataUpdate::Replace(metadata) => {
            self.a11y.hashes.retain(|id, _| metadata.live.contains(id));
            self.a11y.region_ids.retain(|key, _| metadata.regions.contains(key));
            self.a11y.native_regions.retain(|_, key| metadata.regions.contains(key));
            self.a11y.authored.retain(|key, _| metadata.authored.contains(key));
            self.a11y.metadata = std::rc::Rc::new(metadata);
        }
    }
    self.a11y.publish = A11yPublish::Published { root: new_root, dirt: Dirt::Clean };
}
```

不変条件：build 中に dirt を立てる呼出しは無い（build は region の id 割当てだけを変える）。よって受理時に `Clean` を書いても失われる dirt は無い。

### 3.3 構築の振分け（`a11y/build/dirty.rs`）

```rust
/// What a build changes in the accepted identity metadata, on acceptance.
pub(in crate::a11y) enum MetadataUpdate {
    /// Nothing (a payload build that clipped no row in or out).
    Keep,
    /// Only the clipping of some lists (a payload build).
    Clipping(Clipping),
    /// All of it (a metadata, complete or empty-tree build).
    Replace(Metadata),
}

pub(in crate::a11y) struct Built {
    pub(in crate::a11y) tree: kivact_types::render::SemanticsTree,
    pub(in crate::a11y) metadata: MetadataUpdate,
}
```

- `build_semantics_for`：先頭の `LocalText`・`Geometry` の 2 つの `if let`（`dirty.rs:44-52`）を削除し、次に置換。
  ```rust
  let (own, subtrees, removed) = match dirt {
      Dirt::Clean => return self.build_payloads(root, &Payloads::default(), engine),
      Dirt::Payload(payloads) => return self.build_payloads(root, payloads, engine),
      Dirt::Metadata { nodes, subtrees, removed } => (nodes, subtrees, removed),
      Dirt::Full => (&empty, &empty, &empty),
  };
  ```
  Metadata／Full の残りは今のまま。ただし `metadata.clipped.symmetric_difference(&self.a11y.metadata.clipped)` → `metadata.clipping.rows().symmetric_difference(self.a11y.metadata.clipping.rows())`、`build_identity_metadata` の直後に `#[cfg(feature = "testing")] { self.a11y.enumerated += order.len() as u64; }`、返り値は `metadata: MetadataUpdate::Replace(metadata)`。
- `build_geometry_semantics`（`dirty.rs:90-115`）は削除。
- `build_semantics_tail` の `&metadata.clipped` 2 か所 → `metadata.clipping.rows()`。
- `build_identity_metadata`（`dirty.rs:194`）：`let clipped = self.clipped_rows();` → `let clipping = self.clipping_of_all();`、`Metadata { clipping, .. }`。
- `build_empty_semantics`：`metadata: MetadataUpdate::Replace(Metadata { live: .., ..Default::default() })`。
- `expand_relation_dirt` を `pub(super)` にする（payload.rs が呼ぶ）。
- `build_dirty_nodes` は Metadata／Full 経路でだけ使うので今のまま。

### 3.4 payload の構築（新 `a11y/build/payload.rs`、旧 `local_text.rs` を削除して置換）

`a11y/build.rs` の `mod local_text;` を `mod payload;` に変える。

```rust
//! Role: Build the payloads of payload-only dirt over the accepted identity metadata.
//!
//! Invariants:
//! - Work in proportion to the dirt: the nodes written, the subtrees that
//!   moved (each node once, outer roots first), the overlays placed by them
//!   (to a fixed point), the relation sources of all of these, their boxless
//!   ancestors, and the rows whose clipping changed in the clipping lists at
//!   or above the clip sources. No walk of the whole tree.
//! - The accepted identity metadata is an immutable Rc snapshot that the
//!   build only reads; a changed clipping goes back with the build and is
//!   committed on acceptance. A modal that changed without metadata dirt
//!   (none should) enumerates again, as a metadata build.
//! - Only nodes of the accepted tree (`Metadata::live`) are built: a node
//!   outside it (a leaving subtree's) is published by a metadata build.
//! - Nodes are built parents first (depth, then id), each with the scroll
//!   offsets above it, as the full builder computes them.
use super::dirty::{Built, MetadataUpdate};
use crate::a11y::state::{Clipping, Payloads};
use crate::model::Model;
use arvane as a;
use kivact_types::{NodeId, Point};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

/// A node to build: its depth and the scroll offsets above it.
type Target = (usize, Point);

impl Model {
    pub(super) fn build_payloads(&mut self, root: NodeId, dirt: &Payloads, engine: &mut a::Engine) -> Built {
        let metadata = Rc::clone(&self.a11y.metadata);
        if self.modal() != metadata.modal {
            let promoted = crate::a11y::state::Dirt::Metadata {
                nodes: dirt.nodes.clone(),
                subtrees: dirt.subtrees.clone(),
                removed: BTreeSet::new(),
            };
            return self.build_semantics_for(engine, &promoted);
        }
        let mut targets: BTreeMap<NodeId, Target> = BTreeMap::new();
        self.collect_subtrees(dirt.subtrees.iter().copied(), &mut targets);
        let mut sources = dirt.clip_sources.clone();
        // An overlay is placed by its anchor: one anchored at or under a
        // dirty or moved node moves with it (to a fixed point: overlays
        // anchored in other overlays).
        loop {
            let anchored: Vec<NodeId> = self
                .tree
                .overlays
                .iter()
                .copied()
                .filter(|o| !targets.contains_key(o))
                .filter(|o| {
                    self.node(*o)
                        .and_then(|n| n.overlay())
                        .and_then(|overlay| overlay.anchor())
                        .is_some_and(|anchor| {
                            self.tree()
                                .chain(Some(anchor))
                                .iter()
                                .any(|a| dirt.nodes.contains(a) || targets.contains_key(a))
                        })
                })
                .collect();
            if anchored.is_empty() {
                break;
            }
            sources.extend(anchored.iter().copied());
            self.collect_subtrees(anchored.into_iter(), &mut targets);
        }
        let mut dirty: BTreeSet<NodeId> = dirt.nodes.clone();
        dirty.extend(targets.keys().copied());
        let clipping = self.reclip(&metadata.clipping, &sources, &mut dirty);
        self.expand_relation_dirt(&metadata, &mut dirty);
        self.invalidate_boxless_ancestors(&mut dirty);
        dirty.retain(|id| metadata.live.contains(&id.0));
        let mut order: Vec<(usize, NodeId, Point)> = dirty
            .into_iter()
            .map(|id| match targets.get(&id) {
                Some(&(depth, offset)) => (depth, id, offset),
                None => (self.tree().depth(id), id, self.scroll_above(id)),
            })
            .collect();
        order.sort_unstable_by_key(|&(depth, id, _)| (depth, id));
        let clipped = match &clipping {
            Some(c) => c.rows(),
            None => metadata.clipping.rows(),
        };
        let mut nodes = Vec::new();
        let mut regions = BTreeSet::new();
        for (_, id, offset) in order {
            self.build_node(id, offset, clipped, &mut nodes, &mut regions, engine);
        }
        #[cfg(feature = "testing")]
        {
            self.a11y.constructed += nodes.len() as u64;
        }
        let focus = self.semantics_focus(clipped);
        Built {
            tree: kivact_types::render::SemanticsTree { root: kivact_types::SemanticsId(root.0), focus, nodes },
            metadata: clipping.map_or(MetadataUpdate::Keep, MetadataUpdate::Clipping),
        }
    }

    /// Add every node under `roots` (each root included) with its depth and
    /// the scroll offsets above it; a root already added (inside an outer
    /// root, which goes first) is skipped.
    fn collect_subtrees(&self, roots: impl Iterator<Item = NodeId>, targets: &mut BTreeMap<NodeId, Target>) {
        let mut roots: Vec<(usize, NodeId)> = roots.map(|r| (self.tree().depth(r), r)).collect();
        roots.sort_unstable();
        for (depth, root) in roots {
            if targets.contains_key(&root) {
                continue;
            }
            let mut stack = vec![(root, depth, self.scroll_above(root))];
            while let Some((id, d, offset)) = stack.pop() {
                if targets.insert(id, (d, offset)).is_some() {
                    continue;
                }
                let Some(n) = self.node(id) else { continue };
                let s = n.scroll();
                let below = Point { x: offset.x + s.x, y: offset.y + s.y };
                stack.extend(self.semantic_children(id).into_iter().map(|c| (c, d + 1, below)));
            }
        }
    }

    /// The clipping after the clipping lists at or above `sources` look at
    /// their rows again (`None`: no such list, or no row changed); the rows
    /// whose clipping changed go into `dirty`.
    fn reclip(&self, accepted: &Clipping, sources: &BTreeSet<NodeId>, dirty: &mut BTreeSet<NodeId>) -> Option<Clipping> {
        let mut lists = BTreeSet::new();
        let mut seen = BTreeSet::new();
        for &source in sources {
            let mut at = Some(source);
            while let Some(x) = at {
                if !seen.insert(x) {
                    break; // its ancestors were looked at
                }
                if self.is_clipping_list_node(x) || accepted.has_list(x) {
                    lists.insert(x);
                }
                at = self.tree().get(x).and_then(|n| self.tree().parent_id(n));
            }
        }
        if lists.is_empty() {
            return None;
        }
        let mut clipping = accepted.clone();
        let mut changed = false;
        for list in lists {
            let clipped = self.list_rows(list).map(|(_, c)| c).unwrap_or_default();
            let rows = clipping.set_list(list, clipped);
            changed |= !rows.is_empty();
            dirty.extend(rows);
        }
        changed.then_some(clipping)
    }
}
```

`a11y/build/node.rs` の変更：
- `clipped_rows`（`node.rs:415-425`）を置換：
  ```rust
  /// The clipping of every shown clipping list-like container (a complete enumeration).
  pub(super) fn clipping_of_all(&self) -> Clipping {
      let mut clipping = Clipping::default();
      for (list, _) in self.tree().iter() {
          if let Some((_, clipped)) = self.list_rows(list) {
              clipping.set_list(list, clipped);
          }
      }
      clipping
  }

  /// Whether `id` is a clipping list-like container (shown or not).
  pub(super) fn is_clipping_list_node(&self, id: types::NodeId) -> bool {
      self.tree().get(id).is_some_and(|n| is_clipping_list(n, self.editors[n.index.0].is_some()))
  }
  ```
- `list_rows` を `pub(super)` にする（中身は不変）。

`a11y/build/geometry.rs` の `invalidate_boxless_ancestors`（`geometry.rs:391-405`）を、祖先ごとに一度だけ見る形に置換（結果は同じ。大きな subtree で O(dirty×深さ) → O(dirty+深さ)）：
```rust
pub(super) fn invalidate_boxless_ancestors(&self, dirty: &mut std::collections::BTreeSet<NodeId>) {
    let mut seen = std::collections::BTreeSet::new();
    for id in dirty.clone() {
        let mut at = self.tree().get(id).and_then(|n| self.tree().parent_id(n));
        while let Some(ancestor) = at {
            if !seen.insert(ancestor) {
                break; // it and its ancestors were looked at
            }
            if self.node(ancestor).is_some_and(|node| node.is_transparent()) {
                dirty.insert(ancestor);
            }
            at = self.tree().get(ancestor).and_then(|n| self.tree().parent_id(n));
        }
    }
}
```

### 3.5 書き手の変更（全呼出元）

**`model/aggregate.rs`**
- `repaint_text`（169-174）：`self.a11y.local_text(id)` → `self.a11y.payload(id)`。
- `repaint`（176-187）を置換：
  ```rust
  /// `id` is drawn again (its paint or state styles): only its own payload
  /// (its scene regions' padding box) can change; a state style that moves
  /// or reorders it moves its subtree.
  pub(crate) fn repaint(&mut self, id: NodeId) {
      if self.tree.contains(id) {
          self.a11y.payload(id);
          if self.node(id).is_some_and(|n| n.states().is_some_and(crate::tree::states_place)) {
              self.a11y.moved(id);
          }
          self.changes.mark(id, super::changes::Dirty::REDRAW);
      }
  }
  ```
- 新設（`repaint` の後）：
  ```rust
  /// The scroll offset of `id` changed: its own payload and the clipping
  /// lists at or above it. Every other payload under it keeps its
  /// content-space bounds, except the layers under it (an overlay, a fixed
  /// box), drawn at their window origins: they move in content space.
  /// Overlays anchored under it follow (`a11y::build::payload`).
  pub(crate) fn a11y_scrolled(&mut self, id: NodeId) {
      self.a11y.scrolled(id);
      let layers: Vec<NodeId> = self
          .tree
          .overlays
          .iter()
          .chain(self.tree.fixed.iter())
          .copied()
          .filter(|x| *x != id && self.tree.is_in_subtree(*x, id))
          .collect();
      for layer in layers {
          self.a11y.moved(layer);
      }
  }
  ```
  （`model` から `a11y` の処理を呼ばないよう、Model 側の aggregate に置く。`A11yState` の method 呼出しは今の慣行どおり。）

**`model/apply.rs`**
- 104-106：`self.a11y.subtree(root)` → `self.a11y.moved(root)`。
- `scroll_to`（137-151）を置換：
  ```rust
  pub(crate) fn scroll_to(&mut self, id: NodeId, offset: Point) {
      let Some(n) = self.tree.get_mut(id) else { return };
      let moved = n.scroll != offset;
      n.scroll = offset;
      if moved {
          // Like a wheel or a scroll by a delta, a new offset moves the
          // content (the paint list redraws positions on `moved`).
          self.changes.moved();
          self.changes.mark(id, Dirty::SCROLLED);
          self.a11y_scrolled(id);
      }
      self.changes.layout.insert(crate::model::changes::LayoutReasons::PASS);
  }
  ```
- 388-393：`self.a11y.subtree(id)` → `self.a11y.moved(id)`。
- `invalidate_semantics`（397-415）を置換：
  ```rust
  /// What a write can change in the accessibility tree. A layout style
  /// (hidden, visibility, display, overflow), an inherited disabled state,
  /// read-only and every write not marked payload-only (role, name, actions,
  /// relations, editing, a drawable role ..) may change identities, parents,
  /// regions or clipping: the metadata is enumerated again. A payload-only
  /// write (a text, a paint, state styles, a text style, an image, a
  /// placeholder) changes the node's own payload; a paint may change its clip;
  /// a new transform or placement moves its subtree.
  fn invalidate_semantics(&mut self, id: NodeId, mutation: Mutation) {
      if mutation.payload_only && !(mutation.style || mutation.subtree || mutation.read_only) {
          self.a11y.payload(id);
      } else {
          self.a11y.node(id);
      }
      if mutation.style || mutation.subtree {
          self.a11y.subtree(id);
      }
      if mutation.paint {
          self.a11y.reclip(id);
      }
      if mutation.placed {
          self.a11y.moved(id);
      }
  }
  ```
  （旧の `text` → `subtree` は無くなる：text style の変化は自分の payload と、`invalidate_inherited_text` の下の text の payload と、layout の `moved` で足りる。）
- 436、441：`self.a11y.node(x)` → `self.a11y.payload(x)`。
- それ以外（89, 283, 354, 464, 467, 468, 495, 507, 532, 581, 596）は変えない（構造・identity の変化）。

**`mutation.rs`**
- field `local_text` を `payload_only` に改名。doc：「Only the node's own accessibility payload can change: its exposure, hierarchy, relations, regions and clipping stay (a text, a paint, state styles, a text style, an image, a placeholder).」既存の xtask-allow 行は文言そのまま残す。
- `NONE` の初期化子、`LOCAL_TEXT` → `PAYLOAD_ONLY`（`payload_only: true`）、`and` の 1 行を改名。
- `WriteStats` に `pub a11y_enumerated: u64,`（doc：「Cumulative nodes the accessibility identity enumeration visited, excluding debug references.」）を `a11y_published` の後に追加。

**`host/write/typed.rs`**（`mut =` の行だけ）
| 行 | prop | 新しい `mut` |
|---|---|---|
| 172 | Paint | `Mut::PAINT.and(Mut::PAYLOAD_ONLY)` |
| 176 | TextStyle | `Mut::TEXT.and(Mut::PAYLOAD_ONLY)` |
| 180 | States | `Mut::STATES.and(Mut::PAYLOAD_ONLY)` |
| 268 | Content | `Mut::MEASURE.and(Mut::PAYLOAD_ONLY)` |
| 272 | TextSpans | `Mut::MEASURE.and(Mut::PAYLOAD_ONLY)` |
| 311 | Placeholder | `Mut::MEASURE.and(Mut::PAYLOAD_ONLY)` |
| 342 | Image | `Mut::MEASURE.and(Mut::PAYLOAD_ONLY)` |

他の prop は変えない（Mut::NONE の role・name・semantics 等は Metadata のまま。8 章 6）。

根拠（payload-only が identity を変えないこと）：identity metadata の入力は、親子（tree の link）、relation（`Semantics` prop）、region（scene と `semantic_role`）、authored run（`Semantics.text` と role）、modal（overlay prop と hidden）、clipping（layout の hidden・visibility・overflow、paint の `clip`、role、focus）。`semantic_role`（`model/semantics.rs:37-59`）は role prop・kind・editor の有無・name・actions・scene だけを読む。上の 7 prop はどれも読まれない。例外は paint の `clip`（`tree/node.rs:459-465` の `clips()`）で、これを `reclip` で扱う（state style の delta は `clips()` に入らない：base paint だけを読む）。

**`model/edit/inputs.rs:143`**：`Mutation::LOCAL_TEXT` → `Mutation::PAYLOAD_ONLY`。

**幾何だけの変化（`subtree` → `moved`）**
- `model/states.rs:173`、`model/interact/sticky.rs:58`、`model/state_motion/steps.rs:322`、`model/layout_motion/steps.rs:187`、`model/drawables/request.rs:42`（`m.a11y.subtree(root)` → `m.a11y.moved(root)`）。

**その他**
- `model/focus/navigation.rs:306-308`：`self.a11y.request_publication(std::iter::empty())` → `self.a11y.focus_moved(old_target.map(|t| t.node), target.map(|t| t.node))`。
- `model/interact/sync.rs:145`（font）と `:270`（image の natural size）：`self.a11y.node(..)` → `self.a11y.payload(..)`。
- `model/interact/geometry.rs:664`（`set_scroll`）：`self.a11y.moved(id)` → `self.a11y_scrolled(id)`（このファイルは P2-U2 の所有、6 章）。
- `host/events.rs:522-536`：
  ```rust
  ShellEvent::AccessibilityActivated => {
      if let Some(model) = self.app.borrow_mut().model_mut(id)
          && model.a11y.activate()
          && self.accessibility_noted.note()
      {
          self.frame_log.borrow_mut().note("accessibility=on");
      }
  }
  ShellEvent::AccessibilityDeactivated => {
      if let Some(model) = self.app.borrow_mut().model_mut(id) {
          model.a11y.deactivate();
      }
  }
  ```
- 変えないもの：`sync.rs:51,53`（viewport）、`frame/ticks.rs:110`（環境）、`presence/transitions.rs:193`（退場：logical children が変わる）、`apply.rs` の構造の呼出し。

**診断の counter の配線（`a11y_enumerated`）**
- `host/write/reader.rs:73-85` と `host/write/writer.rs:134-148` の `#[cfg(feature = "testing")]` の `WriteStats { .. }` に `a11y_enumerated: app.windows.values().map(|model| model.a11y.enumerated).sum(),` を追加。
- `crates/kivact-core/src/diagnostics/stats.rs`：`a11y_published` の後に `/// Cumulative nodes the accessibility identity enumeration visited (a complete pass per structural change; payload-only changes visit none).` `pub a11y_enumerated: u64,`。
- `crates/kivact-core/src/launch/inspect.rs:79-80` に `a11y_enumerated: ts.a11y_enumerated,`。

### 3.6 hit test の枝刈り

#### 3.6.1 依存の向き（module 規則）

`model` は `paint` を使えない。hit test は model の処理（pointer・wheel から呼ばれる）なので、**model が trait `HitInk` を定義し、paint がそれを `Model` に実装する**（依存性逆転。既存の `layer::LayerView` を Model が実装するのと同じ形）。paint は model を使ってよい。8 章 1 で人間の確認を求める。

#### 3.6.2 新 `model/interact/hit.rs`

`model/interact.rs` に `pub(crate) mod hit;` を追加。`model/interact/geometry.rs` から次を**そのまま移す**：`struct ContextPlane`（55-60）、`hit`（444-461 の doc 込み）、`hit_any`、`hit_node`、`hit_in`、`backface_hidden_at`、`hit_box`、`hit_context`、`collect_planes`、`plane_point`（447-646）。`shown_overlays`・`layers`・`overlay_of` は geometry.rs に残す。移した後、geometry.rs の不要になった `use`（`HashSet`、`Matrix4` 等、コンパイラの警告どおり）を消す。移した関数には次の変更を加える。

```rust
//! Role: Find the node under a window point: the layers top first, then the
//! root's tree, the deepest shown hit-testable box first.
//!
//! Invariants:
//! - A child's subtree is looked at only when the point lies in the child's
//!   ink (`HitInk`: everything the child and its subtree can draw on, which
//!   holds every box of the subtree as drawn), so a point inside a long list
//!   visits the rows around it, not the list. A child whose ink does not
//!   bound its hits is always looked at: a sticky box (its shift is not in
//!   its ink), one whose map to its parent is not affine (its ink leaves out
//!   a perspective), and a drawable host or an ancestor of one (a drawable is
//!   placed by its host's drawing, outside the inks).
//! - The ink is the paint list's (kept until something in the subtree changes
//!   what or where it draws, computed when missing), never stale between
//!   syncs. The debug self-check compares every settled hit with the complete
//!   walk (`hit_reference`).

/// The drawing's answer to where a subtree can be hit.
pub(crate) trait HitInk {
    /// The ink of the box `id` (`n`, laid out as `layout`): what it and its
    /// subtree can draw on, relative to its layout location in its parent's
    /// unscrolled content space, mapped by its own transform (`None`: it
    /// draws nothing; it or an ancestor is hidden).
    fn hit_ink(&self, id: NodeId, n: &HostNode, layout: &kivact_layout::NodeLayout) -> Option<Rect>;
}

/// What one hit test reads once.
pub(super) struct HitScope {
    /// Whether children are left out by their inks (`false`: the debug
    /// reference walks everything).
    prune: bool,
    /// The drawable hosts and their ancestors, never left out.
    hosts: HashSet<NodeId>,
}

/// Logical pixels an ink is grown by: it is computed relative to the
/// child's location and compared in the parent's space.
const HIT_SLACK: f64 = 0.5;

impl Model {
    fn hit_scope(&self, prune: bool) -> HitScope {
        let mut hosts = HashSet::new();
        for (_, entry) in self.drawables.iter() {
            let host = self.tree.get(entry.root).and_then(|n| self.tree.parent_id(n));
            for x in self.tree.chain(host) {
                if !hosts.insert(x) {
                    break;
                }
            }
        }
        HitScope { prune, hosts }
    }

    /// (doc は旧 hit のもの)
    pub(crate) fn hit(&self, p: Point) -> Option<NodeId> {
        self.hit_with(p, &self.hit_scope(true))
    }

    /// [`Model::hit`] by the complete walk: the debug self-check's reference.
    #[cfg(all(debug_assertions, feature = "testing"))]
    pub(crate) fn hit_reference(&self, p: Point) -> Option<NodeId> {
        self.hit_with(p, &self.hit_scope(false))
    }

    fn hit_with(&self, p: Point, scope: &HitScope) -> Option<NodeId> {
        // 旧 hit の本体。self.hit_node(x, p) を self.hit_in(x, p, None, scope) に置換。
    }

    pub(crate) fn hit_any(&self, p: Point) -> Option<NodeId> {
        let scope = self.hit_scope(true);
        // 旧本体。self.hit_node(x, p) を self.hit_in(x, p, None, &scope) に置換。
    }

    pub(crate) fn hit_node(&self, id: NodeId, p: Point) -> Option<NodeId> {
        self.hit_in(id, p, None, &self.hit_scope(true))
    }
    // hit_in / hit_box / hit_context に `scope: &HitScope` を最後の引数として足し、
    // 互いの呼出しに渡す（hit_in → hit_box、hit_in → hit_context、hit_context → hit_box）。
}
```

`hit_box` の子の loop（旧 `geometry.rs:540-549`）を置換：
```rust
let scroll = n.scroll();
for c in self.stacked(n).rev() {
    if skip.is_some_and(|skip| skip.contains(&c)) {
        continue;
    }
    let Some(cn) = self.tree.get(c) else { continue };
    if cn.is_layer() || cn.properties.drawable.is_some() {
        continue;
    }
    if scope.prune && !self.may_hit(c, cn, scroll, p, scope) {
        continue;
    }
    if let Some(h) = self.hit_in(c, p, skip, scope) {
        return Some(h);
    }
}
```
新設：
```rust
/// Whether `p` (in the box space of the parent, which scrolls its children
/// by `scroll`) can hit the subtree of its child `c` (see the module
/// invariants).
fn may_hit(&self, c: NodeId, cn: &HostNode, scroll: Point, p: Point, scope: &HitScope) -> bool {
    if cn.place.positioning == crate::tree::node::Positioning::Sticky || scope.hosts.contains(&c) {
        return true;
    }
    if (cn.place.transformed || self.layout_motion_transform(c).is_some())
        && self.to_parent(c).is_none_or(|m| m.flatten().as_affine().is_none())
    {
        return true;
    }
    let layout = self.computed_layout(c).unwrap_or_default();
    let Some(ink) = self.hit_ink(c, cn, &layout) else {
        return false; // not drawn, so not shown: nothing in it is hit
    };
    let x = ink.x + layout.location.x - scroll.x;
    let y = ink.y + layout.location.y - scroll.y;
    p.x >= x - HIT_SLACK
        && p.x <= x + ink.width + HIT_SLACK
        && p.y >= y - HIT_SLACK
        && p.y <= y + ink.height + HIT_SLACK
}
```
（`to_parent` が `None` は特異で `hit_in` も `None` を返すので、降りても結果は同じ。`ink` が `None` ⇒ `drawn` でない ⇒ `shown` でない（`tree/node_tree.rs:264-274`、drawn ⊇ shown）⇒ `hit_in` は `None`。hit 範囲 ⊆ 自分の箱（`lower/shape.rs:245-253`）⊆ reach（`ink.rs:28-61`、箱 ∪ content を非負の `own` で広げる）⊆ ink、子の ink は親の ink に含まれる（clip するなら子は clip の中でしか当たらない：`geometry.rs:527-529`）。ink の配置は `ink_local`（`ink.rs:125-131`）と同じ「`r - parent.scroll + location`」で、`to_parent`（`model/transform.rs:272-283`）の配置と一致する。）

#### 3.6.3 新 `paint/list/hit_reach.rs`

`paint/list.rs` の `mod` 一覧（87-93 行）に `mod hit_reach;` を追加。
```rust
//! Role: Answer the hit test's ink query from the paint list's kept inks.
use super::ink::ink;
use crate::model::Model;
use crate::model::interact::hit::HitInk;
use crate::tree::node::HostNode;
use kivact_layout::NodeLayout;
use kivact_types::{NodeId, Rect};

impl HitInk for Model {
    fn hit_ink(&self, id: NodeId, n: &HostNode, layout: &NodeLayout) -> Option<Rect> {
        ink(self, id, n, layout, self.paint.list.ink_gen)
    }
}
```
（`ink` は `paint::list` の `pub(super)`、`PaintState.list` と `PaintList.ink_gen` は `paint` の中で見える。hit 時に計算した ink は paint の cache に入り、次の paint list の構築がそのまま使う。`forget_ink` の「上は全部消えている」不変条件（`invalidation.rs:38-45`）は、欠けた ink を後から埋めても崩れない。）

#### 3.6.4 debug の自己検査（`host/testing/self_check.rs`）

`errors.extend(model.text_self_check());`（22 行）の後に：
```rust
// The hit test leaves children out by their inks: it finds what the
// complete walk finds, under the mouse and at the window's center.
let size = model.window();
let points = [
    self.probe.window(kivact_types::WindowId::MAIN).and_then(|probe| probe.mouse_position()),
    Some(kivact_types::Point { x: size.width / 2.0, y: size.height / 2.0 }),
];
for at in points.into_iter().flatten() {
    let (pruned, reference) = (model.hit(at), model.hit_reference(at));
    if pruned != reference {
        errors.push(format!("hit at ({}, {}): pruned {pruned:?}, reference {reference:?}", at.x, at.y));
    }
}
```
`KIVACT_SELF_CHECK=0` で全体と同じく止まる。

### 3.7 旧設計 1.5 の「相対座標＋AccessKit transform」案の評価

- **不要と判断**。wheel KPI は hit test だけ（1.1）。scroll の a11y は 1.5 の事実から O(container＋layer＋影響する clip 行) にできる（3.5 の `a11y_scrolled`）。pointer・click は payload だけ。
- 相対座標が効くのは「大きな subtree の祖先の位置が動く」場合（list の上の要素の高さが変わる等）で、K5 の 4 場面には現れない。変更範囲は `kivact-types` の bounds の意味、arvane-shell の AccessKit 変換、test driver の `accessibility_bounds`、bounds を読む既存 E2E に及び、期待値を変えずに移行できる保証が無い。8 章 5 に記録する。

### 3.8 境界条件

| 条件 | 扱い |
|---|---|
| 削除・unmount | `remove_now` → `request_publication`（Metadata、今のまま）。payload の dirt に残った id は `live` で落ちる。hit は tree に無い id を見ない |
| 非表示（hidden・visibility） | layout style の書込み → Metadata（全 clipping 再計算）。hit は ink が `None` で枝刈り（`set_prop` の可視性変化で ink が捨てられる） |
| 退場中（leaving） | 公開木に無いノードの payload は `live` で落ちる（旧 LocalText 経路の拒否を防ぐ、7 章 4）。hit は `shown` で弾く（ink は drawn なので降りる） |
| reduced motion | 状態・layout motion の tick は `moved`（payload 経路）。motion が止まれば dirt も出ない |
| window close | Model ごと破棄。dirt は Published の中だけ |
| AT の停止・再開 | `deactivate` で dirt・hashes・metadata を捨てる。再開は Full |
| 拒否 | Published は Full、AwaitingFirstTree はそのまま |
| modal の開閉 | overlay prop・hidden の書込み → Metadata。payload 経路は `modal()` の不一致で Metadata に昇格（安全網） |
| focus | `focus_moved` → payload＋clip source。focus の公開は毎回 `semantics_focus` |
| DevTools | emulation は viewport → Metadata。`pick_node` は `hit_any`（枝刈り込み）。DevTools の overlay は host が描くので tree の外 |
| drawable | host とその祖先は枝刈りしない。drawable root の配置変化は `moved(root)` |
| 3D・perspective | 子の写像が affine でなければ枝刈りしない。3D context の plane は `hit_context` が今どおり処理 |
| sticky | 枝刈りしない。shift の変化は `stick` の `moved` |
| scroll | container の payload、下の layer、anchor が下にある overlay、clip list。子孫の bounds は content 空間で不変（1.5） |
| display: contents | 平坦な子で ink を見る（contents は box を持たず子が flat child）。boxless 祖先の payload は `invalidate_boxless_ancestors` |
| pointer-events（hit_testable）| 枝刈りは superset なので結果を変えない |

---

## 4. 見積り（目標との対比）

単位：B1 機の ms（prof×1.6）。P2 の後の値は計算量と prof の 1 ノードあたりの費用からの推定。

### 4.1 P2 の部分

| 項目 | 今（prof） | P2 後（prof） | 根拠 |
|---|---|---|---|
| hover 1 回の a11y（build＋hash_submit） | 28.54 | ≤ 0.05 | 2 button の `build_node`（各 5〜10 µs：rect と window_map の深さ 8 の walk）、relation と boxless の walk、2 件の hash、shell 適用 2 件。identity の列挙・retain・Metadata の破棄が無くなる |
| click（unrelated）の a11y | ≈ 26〜28（Metadata、N 依存） | ≤ 0.05 | button＋text の payload、text の subtree 1 ノード。clip list は無い |
| row 更新の a11y | ≈ 26〜28 | ≤ 0.1 | Bump、row0 の text・button とその label の数ノード |
| 列の中の hit 1 回 | 13.47 | ≈ 0.5 | 根から scroll までの 4 段（子は 1〜4 個）、内側の column の 3,000 子の ink 判定（1 子 60〜80 ns ≈ 0.2〜0.25 ms）、hover の直後は内側 column の ink 再計算（3,000 子の union、≈ 0.15〜0.2 ms）、当たる 1〜2 行の 12 ノード |
| 列の上の hit（click） | 既に小さい | 同じ | 1.1 |

### 4.2 KPI ごとの内訳と予算（B1 機）

| KPI | 目標 | 今 | P2 の予算 | P2 以外（P1 ほか） | P2＋P1 後の見込み |
|---|---|---|---|---|---|
| wheel n3000 | ≤ 16.29 | 22.02 | **hit ≤ 1.0**、他 ≤ 0.1 | なし（scroll・redraw が起きない） | **≈ 0.9 ms：P2 単独で達成** |
| pointer p95 n3000 | ≤ 10.60 | 89.05 | **a11y ≤ 0.3、hit ≤ 1.0** | turn 2.7（prof 1.66、8 章 4）、present：P1 が sync_shader（5.24）・sync_playback（5.64）を除けば raster 1.27＋paint_list 0.15＋compose 0.12 ≈ prof 1.6 → 2.6 | ≈ 0.3＋1.0＋2.7＋2.6 ≈ **6.6 ms**（P1 が present を prof ≤ 2.5 にすれば 10.6 以下） |
| unrelated n3000 | ≤ 9.46 | 57.53 | **a11y ≤ 0.5**、hit（上部）≤ 0.3 | turn（1〜2 回）、layout（compute ≈ 0.12 prof、capture_motion 等）、present | P1 の present・layout の結果次第。a11y が抜けると残りは P1＋turn |
| row n3000 | ≤ 16.96 | 78.98 | **a11y ≤ 0.5**、hit ≤ 0.3 | layout（compute ≈ 11.4 prof：P1）、turn、present | 同上 |

- 「a11y ≤ 3 ms」の例に対し、本設計の a11y は 4 場面とも **≤ 0.5 ms**（N に依存しない。clip list の行数にだけ依存、4.3）。
- 4 場面の P2 の合計予算：pointer 1.3、wheel 1.1、unrelated 0.8、row 0.8。
- 注意：unrelated・row の場面別の prof 内訳は採取されていない。今の値（57.53、78.98）のうち a11y が占める割合は、pointer と同じ N 依存の Metadata 経路から 26〜28 ms（prof）と推定したが、場面の比 1.6 で換算すると present・turn と合わせて今の値を超えるので、比が場面で異なる可能性がある。**到達の確認は checkpoint の計測で行う**（6 章）。

### 4.3 残る N 依存

- clip list（list 系の role で clip する container）の行の再計算は、その list の**載っている行数**に比例（`list_rows`、1 行 ≈ 1 µs）。virtual list（30〜50 行）は無視できる。3,000 行を全部 mount した ListBox では行の移動・scroll ごとに ≈ 3 ms（B1 で ≈ 5 ms）。KPI の bench には該当しない（`scroll(column)` に role が無い）。
- Metadata 経路（構造・role・name 等の変化）は O(N) のまま（8 章 3）。K5 の 4 場面には現れない（最初の click の focus も payload 経路になる）。
- hit：枝刈り後も、通る container の子の数に比例（3,000 子で ≈ 0.25 ms）。paint の `ChildIndex` の二分探索は、index が古い間（`touched`・`pass`）は使えず、z 順の並べ替えとも合わせにくいため採らない。

---

## 5. 正しさ・リスク・E2E

### 5.1 担保

- **a11y**：debug の `self_check`（`host/testing/self_check.rs:44-75`）が、全ての E2E の毎 settle で「公開済みの木」と「Full の参照構築」を比較する。payload 経路の取りこぼしは既存 E2E の全件で検出される。
- **hit**：3.6.4 の比較を新設。既存 E2E の毎 settle で、mouse 位置と window 中央の hit を枝刈り無しと比較する。
- 既存 E2E で特に効くもの：`hover.rs`、`pointer.rs`、`transform.rs`（回転した箱の角・先端の hit）、`transform_3d.rs`、`drawables.rs`、`layout_scroll.rs`（`reaching_rows`：-300 px 移動した行、影の行）、`overlay.rs`、`interaction_tooltip.rs`・`apg_tooltip.rs`（anchor の overlay）、`apg_listbox.rs`・`flows.rs`（ListBox）、`focus.rs`、`touch_scroll.rs`、`interaction_drag_edit.rs`、`text_editing.rs`。

### 5.2 リスク

| リスク | 対策 |
|---|---|
| payload-only の分類の誤り（identity が変わる書込みを payload にした） | 3.5 の根拠。self_check が検出。分類は 7 prop に限定 |
| ink が hit の範囲を含まない場合（例外の漏れ） | sticky・非 affine・drawable を除外。3.6.4 の比較が検出。**P1 が paint の ink の不変条件（drawing・geometry・scroll・可視性の変化で自分と祖先の ink を捨てる）を変える場合は P2-U2 と調整**（8 章 1） |
| `HitInk` の依存性逆転が規則の回避と見なされる | 8 章 1 |
| clip list の行の再計算の漏れ（例：list の外の overlay が row） | anchor で動く overlay も clip source に入れる（3.4）。self_check |
| `Rc::make_mut` の複製 | build の Rc clone は build の終わりで捨てる。複製されても正しさは同じ |

### 5.3 追加する E2E（P2-U3）

新ファイル `dev/tests/kivact-public-api/tests/e2e/a11y_incremental.rs`（`main.rs` に `mod a11y_incremental;`）。fixture は kv_latency と同じ形（`column((text(Other n), button("Other"), button("Bump"), scroll(column(rows))))`、行は `row((text(label), button("x").test_id(format!("x{i}"))))`、200 行、scroll に `style().height(300).shrink(0.0)`）。行は `Vec<View>` で組む（`layout_scroll.rs:124-153` の書き方）。

1. `hover_and_press_publish_without_enumerating`：`x3` を `hover()`、`let s0 = screen.stats();`、`x4` を `hover()`、`let s1 = screen.stats();` → `assert_eq!(s1.a11y_enumerated, s0.a11y_enumerated)`、`assert!(s1.a11y_constructed - s0.a11y_constructed <= 4)`。続けて `x4` を `click()`（pressed と最初の focus）→ `a11y_enumerated` が不変。
2. `a_text_update_publishes_only_the_text`：Other を 1 回 click（focus を移す）、`s0`、もう 1 回 click、`s1` → enumerated 不変、constructed の差 ≤ 6。Bump の click でも同じ（row 0 の label）。
3. `scrolling_publishes_the_container_and_its_layers`：scroll の内容に `node().test_id("pin").style(style().position(Position::Fixed).left(10).top(10).size(20, 20))` を 1 つ入れる。`screen.get_by_test_id("list").scroll(0.0, 120.0)` の前後で enumerated 不変、constructed の差 ≤ 6（container と fixed）。published の正しさは self_check。
4. `a_list_clips_rows_by_scroll_and_focus`：`scroll(column(options)).with_node(|n| n.role(Role::ListBox)).style(style().height(200).shrink(0.0))`、option は `node().role(Role::Option).name(format!("Option {i}")).focusable(true).style(style().height(20).shrink(0.0))` を 100 個。`scroll(0, 400)`、option 25 に `focus()`、`scroll(0, 1200)`（focus の行が clip の外）、`scroll(0, -1600)`。各段で enumerated 不変（clip の hidden の正しさは self_check）。
5. `hit_reaches_boxes_outside_their_parents`（同ファイル）：`transform.rs:44-51` の `pressable` を写した helper で log を取る。(a) 100×100 の親の中に `position(Absolute).left(90).top(-20).size(30, 30)` の badge → 親の外 (親.x+105, 親.y-10) の click が badge、(b) `.translate(150, 0)` の子 → 親の外の click が当たる、(c) `.hit_testable(false)` の wrapper の中の pressable は当たらず、下の pressable が当たる、(d) `display(Display::Contents)` の wrapper の中の pressable が当たる、(e) 200 行の scroll を `scroll(0, 300)` した後、見えている行の中心の click がその行、(f) `position(Sticky).top(0)` の見出しを scroll 後に click。各 click の後の self_check が枝刈り無しと比較する。

### 5.4 checkpoint（重いもの、unit の外）

- `dev/tools/window-lock.sh --timeout 1800 -- cargo test -p kivact-public-api-tests --test e2e`（debug：self_check 有効）。
- `cargo xtask check --no-write`（module 規則・state-fields・行数）。
- KPI：W:\w\kv-prof-p2 等の別 worktree で kv_latency の pointer／wheel／unrelated／row（n3000）を取り、4.2 の予算と比べる。

---

## 6. unit 分割

| unit | 所有ファイル（重複なし） | 依存 | 完了時の軽い確認 |
|---|---|---|---|
| **P2-U1a** a11y の payload 経路と書き手 | `crates/kivact-engine/src/a11y/state.rs`、`a11y/publish.rs`、`a11y/build.rs`、`a11y/build/dirty.rs`、`a11y/build/local_text.rs`（削除）、`a11y/build/payload.rs`（新）、`a11y/build/node.rs`、`a11y/build/geometry.rs`、`model/aggregate.rs`、`model/apply.rs`、`model/states.rs`、`model/interact/sticky.rs`、`model/interact/sync.rs`、`model/state_motion/steps.rs`、`model/layout_motion/steps.rs`、`model/drawables/request.rs`、`model/focus/navigation.rs`、`model/edit/inputs.rs`、`mutation.rs`、`host/write/typed.rs`、`host/events.rs` | なし | `cargo check -p kivact-engine --features testing`；`cargo clippy -p kivact-engine --all-targets --features testing -- -D warnings`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- hover pointer focus layout_scroll` |
| **P2-U1b** 診断 counter の配線 | `host/write/reader.rs`、`host/write/writer.rs`、`crates/kivact-core/src/diagnostics/stats.rs`、`crates/kivact-core/src/launch/inspect.rs` | U1a（`WriteStats.a11y_enumerated`、`A11yState.enumerated`） | `cargo check -p kivact-engine --features testing`；`cargo check -p kivact-core`；`cargo clippy -p kivact-core --all-targets -- -D warnings` |
| **P2-U2** hit の枝刈り | `model/interact.rs`、`model/interact/hit.rs`（新）、`model/interact/geometry.rs`（hit の移動と 664 行）、`paint/list.rs`（`mod` 1 行）、`paint/list/hit_reach.rs`（新）、`host/testing/self_check.rs` | U1a（`a11y_scrolled`） | `cargo check -p kivact-engine --features testing`；clippy 同上；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- transform pointer drawables layout_scroll overlay` |
| **P2-U3** E2E | `dev/tests/kivact-public-api/tests/e2e/a11y_incremental.rs`（新）、`dev/tests/kivact-public-api/tests/e2e/main.rs`（`mod` 1 行） | U1b、U2 | `dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- a11y_incremental` |

- 統合順：U1a → (U1b ∥ U2) → U3 → checkpoint（5.4）。
- **P1 との調整**：P1 が `model/interact/geometry.rs`（`layout()`、capture_motion 等）・`paint/list/*`・`model/interact/sync.rs` を編集する場合は衝突する。提案：P2-U2 を P1 の geometry.rs の編集より先に統合する（hit の塊を別ファイルに出すので、以後の衝突が消える）。`sync.rs` は P2-U1a の 2 行（145、270）だけ。paint の ink の不変条件は P1 が維持する（5.2）。
- STOP 条件：U3 の 1〜3 で `a11y_enumerated` が増える場合、または self_check の a11y／hit の不一致が出た場合は、修正を試みずに、失敗した test 名・settle 直前の操作・self_check のメッセージを報告して止まる（分類または例外の追加は設計者が決める）。

---

## 7. 既存欠陥（79bbd409..db8d679b の実装）

1. **設計の見落とし（性能）**：`Model::repaint`（`model/aggregate.rs:176-187`）が `a11y.node` を呼ぶため、hover・pressed・focus・状態 motion の再描画で必ず Metadata 経路（O(N)）に入る。A11Y-1（Geometry 経路）は K5 で一度も効かない。再現：kv_latency の pointer n3000 の prof で `a11y.path.metadata` が 28/28 回（引継ぎ付録 B）。修正：3.5。
2. **性能**：幾何だけ・payload だけの変化を Metadata にする呼出元（2.3 の 2〜4）。状態 transform・layout motion・sticky は**毎 tick** O(N)（AT 有効時の K3/K4 に効く）。修正：3.5。
3. **資源の溜まり（静的解析）**：`AccessibilityDeactivated`（`host/events.rs:533-536`）は `publish` を `Inactive` にするだけで dirt を消さない。非活性中は公開が早期に戻る（`publish.rs:22`）ので dirt は消費されず、`request_publication` の `removed` に削除された id が際限なく溜まる（id は再利用されない）。`nodes`・`subtrees` も触れたノードの数まで増え、`hashes`・`metadata`（数 MB）も保持され続ける。再現条件：AT が一度有効になり停止した後（Linux の AT-SPI で screen reader を止める）、行の mount・unmount を繰り返す。修正：3.1（dirt を Published の中に移す）と `deactivate`。
4. **推測（未再現）**：旧 LocalText 経路（`a11y/build/local_text.rs:22-63`）は公開木に無いノード（退場中の subtree の text）を publish しうる。shell は `UnreachableNode` で拒否し（`arvane-shell/src/a11y/state.rs:224-233`）、次は Full（O(N)）になる。条件：退場 animation 中の text の更新。修正：3.4 の `live` による選別。
5. **性能**：Metadata の受理時の O(N) の retain と旧 Metadata の破棄（`a11y/publish.rs:67-79`）が毎回 ≈ 2.2 ms。payload 経路では行わない（3.2）。fingerprint の二重計算（`publish.rs:285-295`）も 3.2 で 1 回にする。
6. **文書の誤り**：`SLIM_FAST_DESIGN.md` 1.5 節「a11y の bounds を絶対座標で公開しているため、scroll container の子孫すべての payload が変わる」は誤り（1.5）。V2 設計書で訂正する。

---

## 8. 設計者（人間）の判断が要る点

1. **`HitInk` trait による model → paint の依存性逆転**（3.6.1）。代案：model が自前の「subtree の hit 範囲」cache を持つ（ink と同等の無効化を model 側に重複実装：layout・transform・可視性・子の変化・scroll・sticky・drawable）。重複と無効化漏れの危険が大きいので trait 案を推す。規則の趣旨（model は描画を知らない）に対し、「描画側が hit の上界を答える」という model 定義の interface であることを確認してほしい。
2. **公開の診断 `Stats::a11y_enumerated` の追加**（3.5）。E2E で PERF-1 の後退（payload の変化で列挙が起きる）を検出する唯一の観測点。追加しない場合、5.3 の 1〜4 は self_check の正しさだけを担保し、性能の後退は KPI 計測まで見えない。
3. **Metadata 経路の O(N) を残すこと**。構造の変化（行の追加・削除・入替え）と Mut::NONE の書込み（role・name・value・selected・actions 等）は O(N) の列挙のまま。PERF-1 の「変わったノードと祖先だけ」に反するが、K5 の 4 場面には現れない。増分化の案：`removed`・`subtrees`・`nodes` だけ parents／children hash／reverse（forward edge を別に保持）／regions／authored／live を更新し、`prepare_region_ids` を dirty だけにする。writer の通知の完全性に依存する（今は全比較が安全網）ので、V2 に入れるかを決めてほしい。
4. **turn の固定費（P2 の領域外）**：n3000 で 1 turn ≈ 1.65 ms（prof、B1 機 ≈ 2.7 ms）、うち dispatch ≈ 0.40 ms、残り ≈ 1.25 ms は core の turn（`kivact-core/src/launch/runtime.rs:223-300` の stabilize・`settle_user_effects`・commit）と推定。engine の commit（`host/write/writer.rs:561-573` の `prepare_commit`・`sync`）には静的に見て O(N) は無い。pointer の予算 4.2 はこれを 2.7 ms のまま見込んでいる。担当を決めて内訳を計測してほしい。
5. **相対座標＋AccessKit transform は採らない**（3.7）。将来、list の上の要素の高さ変化で list 全体の bounds が変わる場合の O(subtree) を減らしたいときの候補として記録する。
6. **Mut::NONE の prop の細分**：`Selected`・`Checked`・`Expanded`・`Value`・`Name`（text 以外）等は identity を変えないものが多いが、`semantic_role` が name・actions を読む（画像の装飾判定）ので一律の分類はしなかった。大きな list の選択変更（2 行の `Selected`）を O(changed) にするなら、prop ごとの分類を追加する。
