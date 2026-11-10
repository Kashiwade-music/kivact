# P1 設計：frame・turn・layout・paint 経路の O(N) 除去（a11y を除く）

> **V2 付属書 D1**（設計担当 P1 の成果物）。正本は [SLIM_FAST_V2_DESIGN.md](../SLIM_FAST_V2_DESIGN.md)、手順は [SLIM_FAST_V2_WORK_ORDER.md](../SLIM_FAST_V2_WORK_ORDER.md)。
> - 本書の「8 章 設計者（人間）の判断が要る点」は V2 設計書 4 章（DEC-01〜DEC-23）で決定済み。実装者は 8 章を読んで選ばない。V2 設計書 5 章（付属書間の調整）が本書を上書きする箇所がある。
> - 本書の unit 名と V2 の unit ID の対応は V2 設計書 0.2 節。unit の実施順・lane・wave は V2 作業指示書だけが決める（本書 6 章の「依存」「並列可」は参考）。
> - 文中の P1〜P8 は付属書 D1〜D8（`design-P<n>.md` は `D<n>-*.md`）を指す。`W:\w\kv-prof`・`W:\w\kv-mem`・`W:\w\kv-size`・scratchpad の path は設計時の試作・計測の場所（設計した機械にだけある参考）で、実装に必要ではない。行番号は HEAD `73c36c7c` のもの。

担当 KPI：K3（3000 行 CPU）、K4（work p95/p99・描画間隔 p99）、K5 の unrelated text 更新・row 更新・counter click の a11y 以外の部分。
基点：`db8d679b`（本流 HEAD `73c36c7c` と同一コード）。計測は `W:\w\kv-prof`（detached worktree、付録 C patch＋本担当の追加 span／試作スイッチ）で `CARGO_TARGET_DIR=W:\w\kv-prof-target`、`prof` profile（release 継承、LTO なし、cgu=16）。**この機械（以下「本機」）の prof 値であり、B1 そのものではない**。本機 prof の fix 前の値は B1（実装機 release）に対して unrelated 1.147 倍、row 1.109 倍、counter 1.02 倍だったので、予算は「実装機換算」に直して書く（4 章）。

---

## 1. 事実（計測・コード）

### 1.1 本機 prof の計測（fix 前、`kv_latency` n=3000 と `kv_stress`）

| 場面 | 合計 | a11y（`fw.a11y`、P2） | 非 a11y の主な内訳（1 回あたり） |
|---|---|---|---|
| counter click | 1.99 ms | 0.04 | raster 0.90（うち CPU 合成 0.58〜0.96、仮想 window の present 0.68〜0.74）、core.turn 0.07×2 |
| unrelated n3000 | 66.0 ms | 45.5 | `comp.sync_playback` 8.39、`comp.sync_shader` 8.11、`layout.capture_motion` 0.52、`layout.retarget` 0.20、raster 1.97、paint_list 0.28、compose 0.22 |
| row 更新 n3000 | 87.6 ms | 47.5 | layout 15.8（taffy 8.75、`finish_rounding` 6.90）、sync_playback 8.10、sync_shader 8.04、raster 5.92（damage 平均 196k px）、capture 0.50、retarget 0.28、paint_list 0.38 |
| stress rows=3000（GPU 既定＝CPU で起動） | work p50/p95/p99 18.46/20.23/21.47 ms、scene p95 18.2、interval p99 22.69、CPU 99.9 % | なし | scene＝sync_playback 8.86＋sync_shader 8.35＋compose 0.27。turn 側 layout.all 0.90（capture 0.51、retarget 0.20） |
| stress rows=0 | 1.69/2.12/3.15 ms、CPU 12.8 % | なし | raster 1.13〜2.0 |

- 付録 B の元機の stress では `sync_playback` が 0.8 µs だった。`playing_images` は reduced motion で即 return するので、元機は OS の「アニメーション」設定が切れていたと推定する。B1 機・本機では両方 O(N) が効き、**B1 の scene_p95 17.1 ms は sync_playback＋sync_shader（本機で 8.9＋8.4）でほぼ全部**。
- `scene`（`host/pump.rs:319-338` の `t_scene..t_raster`）に含まれるもの：DevTools overlay の clone、`Model::composition`（`paint/frame_scene.rs:42-56`）＝`paint_list`（refresh_facts・walk・ChildIndex refresh）、`sync_playback`、`sync_shader_clocks`、`prepare_canvas`、`compose_list`（見えている entry の lower）、`report_dropped_draws`、`append_exports`、`append_overlay_layer`。work には加えて `settle_window`（sync＋layout）、`export_pools`、`scroll_inputs`、raster、present。
- row 更新の taffy 内訳（`lay.miss_big.*` span）：3000 子の column（NodeId 22）が 1 回の更新で **ComputeSize 8 回＋PerformLayout 1 回**、毎回 3000 子を走査（子はすべて cache hit。子の cache miss は更新全体で 41 個だけ）。8 回のうち半分は **`parent_size` の x 成分だけが違う同じ問い**（`{800, None}` と `{None, None}`）。taffy 0.14 の `CacheKey` は `x_axis_parent_size` を key に含む（`taffy-0.14.0/src/tree/cache.rs:190-205`）。
- `finish_rounding` は毎 compute で全 tree を `round_all`：`never_rounded` に **21,000 個（1 行 7 個）の `display: contents` slot** が恒久的に残る（`nr.contents` span で計数）。
- row 更新の damage：`DMG` ログで、scroll の clip の `PushClip`（grouping の push 命令）の bounds が「中身の和集合」なので、row 0 の label が伸びるたびに push の bounds が変わり、**旧・新の push bounds（幅＝最も広い行、高さ＝scroll viewport 525 px）が 2 枚**damage になる（101k→480k px）。
- counter click の present 0.7 ms は `arvane-virtual/src/window.rs:494-518` が前 frame を `to_vec()`（1.9 MB）し、`pixels.into()`（Vec→Arc<[u32]> でもう 1 回 1.9 MB copy）しているため。
- 1 行 12 ノードのうち 7 個が Contents。row 更新の geometry 変化は Local 2・PositionOnly 1 だけ（engine 側の invalidation は局所的）。

### 1.2 本機 prof で試作スイッチ（`P1_FIX=1 P1_PS=1`）を入れた後

試作は本設計の近似（sync_* と capture/dup を丸ごと skip、Contents を never_rounded から除外、flat 子の slot cache、push bounds を diff から除外、damage の結合、parent_size の正規化）。仮想 window の present の in-place 化と ChildIndex の部分 refresh は未試作。

| 場面 | fix 前 | 試作後 | うち a11y |
|---|---|---|---|
| counter click | 1.99 | 1.60（present 0.69 が残る） | 0.04 |
| unrelated n3000 | 66.0 | 45.9（非 a11y 2.66） | 43.2 |
| row 更新 n3000 | 87.6 | 54.1（非 a11y 9.1：taffy 4.96、raster 1.98、present 0.70、paint_list 0.46、compose 0.19、core 0.49） | 45.0 |
| stress 3000 | p50/p95/p99 18.46/20.23/21.47、CPU 99.9 % | **1.90/2.61/2.92、interval p99 18.10、scene p95 0.44、CPU 13.4 %** | — |
| stress 0 | 1.69/2.12/3.15、12.8 % | 1.23/1.70/1.80、7.5 % | — |

taffy の段階別（row 更新）：8.75 → slot cache 7.75 → parent_size 正規化 4.96 ms。raster：5.92 → push bounds 除外 4.30（damage 15k px） → 結合 1.80 ms。

### 1.3 コードの O(N)（a11y 以外）

| # | 場所 | 頻度 | 内容 |
|---|---|---|---|
| N1 | `kivact-engine/src/model/frame/clocks.rs:22` `sync_shader_clocks` | 毎 composition | `tree.iter()` 全ノード×`tree.drawn(id)`（祖先を辿る）＝O(N·深さ)。doc は「at a paint list rebuild」だが呼出しは毎 frame（`frame_scene.rs:45`） |
| N2 | `model/playback/tick.rs:20,49` `sync_playback`→`playing_images` | 毎 composition（reduced motion 以外） | 同上 O(N·深さ)、静止画の image node でも毎 frame `assets.animation()` を引き直す |
| N3 | `model/layout_motion/steps.rs:77` `capture_layout_motion` | 毎 layout pass | `tree.iter()` 全走査 |
| N4 | `model/layout_motion/mailbox.rs:23` `report_duplicate_layout_ids` | 毎 layout pass（`retarget_layout_motion` 内） | 全走査＋HashMap 挿入 |
| N5 | `mailbox.rs:66` `take_arrivals` | departed が空でない pass | 全走査 |
| N6 | `kivact-layout/src/tree.rs:914-973` `finish_rounding`/`has_never_rounded` | taffy が compute した毎 pass | Contents slot が `never_rounded` から出ないため毎回全 root を `round_all`（O(N)）＋`retain` O(Contents 数) |
| N7 | taffy の flex 再走査 | 変わった子の親 | 親の子数に比例（LAY-1 で許容した部分）。ただし parent_size の違いで同じ問いを 2 回解いている |
| N8 | `kivact-engine/src/tree/layout_adapter.rs:37-50` `child_count`/`child_at` | taffy の子アクセスごと | 子 1 つ取るたびに `flat(id)`＝RefCell＋`BTreeMap<NodeId,Rc<[NodeId]>>` 探索＋Rc clone/drop＋`index_of`（hash） |
| N9 | `paint/list/index.rs:137-159` `ChildIndex::refresh` | layout pass 後の paint list 再構築 | `relaid` なら 3000 entry すべて `computed_layout` を引き直す（設計どおりだが不要） |
| N10 | `arvane-compose/src/plan/damage.rs:132-205` + `plan/bounds.rs:118-135` | 毎 frame | push 命令の bounds＝中身の和集合 → 中身の範囲が変わると group 全体が damage |
| N11 | `damage.rs:214-250` `finish_damage` | 毎 frame | damage 矩形を結合しない。CPU raster は矩形ごとに chunk の命令を走査し直す（矩形数に比例） |
| N12 | `arvane-virtual/src/window.rs:472-520` `present_with` | 試験 driver の毎 present | 全画素 copy 2 回 |
| 残 | `model/interact/sync.rs:244` settle 後の `report_images(全 image)`、`:261` poll 時の全 image、`report_geometry` の scrollers∪observed、`stale_fact`(透明ノード)の flat 子全部（`aggregate.rs:146`） | 稀／image 数・observed 数に比例 | 本 KPI には効かない（4.4 節に記録） |

### 1.4 旧設計 LAY-1／LAY-2／PNT-1／PNT-2 の実装確認

| 項目 | 結果 |
|---|---|
| LAY-1 `round_changed` | 意図どおり（`tree.rs:152-170`、touch/below/origin 列）。**ただし後付けの `finish_rounding` フォールバックが Contents を拾い続け O(N) に戻している（欠陥 D1）** |
| LAY-2 `scaled` | 意図どおり（`layout.rs:24-43`、`columns.rs:254` reset で除去） |
| PNT-1 `refresh_facts` | 意図どおり（`walk.rs:472-524`、`aggregate.rs:135` の正規化、削除時 `forget_fact`）。debug の全比較は debug のみ |
| PNT-2 `forget_flat_around` | 意図どおり（`node_tree.rs:118-130`、`remove_with` で entry 除去） |
| PERF-1 の「frame/turn は変わったノードだけ」 | N1〜N5 が違反（毎 frame／毎 pass の全走査） |

---

## 2. 原因

1. **frame ごとの全走査（N1・N2）**：shader clock と animated image の「候補」を毎 frame tree 全体から探している。候補は props から決まるのに、props の変化点で索引を持っていない。drawn かどうかと reduced motion を毎回同時に判定しているため全走査になる。
2. **layout pass ごとの全走査（N3〜N5）**：layout transition と layout id を持つノードを毎 pass 全体から探している。
3. **row 更新で taffy が大きくなる理由**：
   - dirty の伝播自体は正しい：row 0 の text が伸びると row 0 の max-content 幅が増え、row 0 が最も広い行になると column の max-content 幅（＝scroll・root column が flex の basis・cross size を決めるために問う値）が実際に変わる。したがって column → scroll → root column → root の cache が消えるのは必要。
   - 消えた column を、親（scroll）とその親（root column）が flex アルゴリズムの各段階で**異なる入力**で問う。taffy 0.14 の cache key は `x_axis_parent_size` を含むので、column 自身が % を持たないのに `parent_size.width` が 800 と None の問いが別 entry になり、**同じ答えを 2 回ずつ計算**（8 回中 4 回が重複）。
   - 子へのアクセスが重い（N8）。
   - それとは別に `finish_rounding` が全 tree を丸める（N6）。B1 の「compute 11.4 ms」の約半分はこれ。
   - unrelated では `Measure::measures_same` が "Other N" の幅が同じと判定し compute 自体が走らない（0.12 ms）。
4. **row 更新の raster**：damage の計算が push 命令の bounds（中身の和集合）を命令自身の描画範囲として比較している（N10）、かつ damage 矩形を結合しない（N11）。
5. **counter click**：試験 driver の仮想 window が毎 present で全画素を 2 回複製（N12）。

---

## 3. 設計（確定版）

### 3.1 S-1 frame 源（shader clock・animated image）と layout motion 源の索引 `MotionSources`

#### 型と所有

新規 `crates/kivact-engine/src/model/frame/sources.rs`（`model/frame.rs` に `pub(crate) mod sources;` を足す）。Model に field `pub(crate) sources: super::frame::sources::MotionSources`（`model/aggregate.rs:25-68` の struct と `:81` の `Model::new` に `sources: Default::default()`）。

```rust
//! Role: The nodes whose own props ask for frames or layout motion: an index over the props.
//!
//! Invariants:
//! - A projection of the node props, written only by `Model::index_sources`
//!   (each touched node, at `Model::sync` through `Model::derive`) and
//!   `Model::forget_sources` (each node `Model::remove_now` destroys). It
//!   records neither whether a node is drawn nor whether motion is reduced:
//!   the readers filter by those when they read.
//! - The frame sources (`Frame::shader_clocks` and the `Playback` entries)
//!   are derived from it at a paint list rebuild only
//!   (`Model::refresh_frame_sources`); everything that changes what that
//!   derivation reads restacks and redraws: an entry here changed, a node's
//!   drawn state (structure, `hidden`, `visibility`), reduced motion, a finished asset.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use kivact_types::{Asset, NodeId};

use super::state::ClockNeed;
use crate::tree::node::HostNode;

/// The clock a node's shaders need with and without reduced motion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ShaderNeed {
    /// Motion allowed: every running clock.
    pub(crate) moving: ClockNeed,
    /// Reduced motion: the essential shaders' clock only.
    pub(crate) reduced: Option<ClockNeed>,
}

#[derive(Default)]
pub(crate) struct MotionSources {
    shaders: BTreeMap<NodeId, ShaderNeed>,
    /// (node, slot) → the image that would play there (slot 0: content image).
    images: BTreeMap<(NodeId, u8), Asset>,
    layout_transitions: BTreeSet<NodeId>,
    layout_ids: BTreeMap<Arc<str>, BTreeSet<NodeId>>,
    /// The key each node is filed under in `layout_ids`.
    filed_ids: BTreeMap<NodeId, Arc<str>>,
}

impl MotionSources {
    /// File `id` from its node `n` (`None`: it is gone). Returns whether a
    /// frame source (a shader clock or an image) of it changed.
    pub(crate) fn file(&mut self, id: NodeId, n: Option<&HostNode>) -> bool {
        let shader = n.and_then(|n| {
            Some(ShaderNeed { moving: super::clocks::node_clock(n, false)?, reduced: super::clocks::node_clock(n, true) })
        });
        let mut changed = match shader {
            Some(s) => self.shaders.insert(id, s) != Some(s),
            None => self.shaders.remove(&id).is_some(),
        };
        let images = n.map(crate::model::playback::state::node_images).unwrap_or_default();
        let old: Vec<(u8, Asset)> = self.images.range((id, 0)..=(id, u8::MAX)).map(|(k, a)| (k.1, a.clone())).collect();
        if old != images {
            for (slot, _) in old { self.images.remove(&(id, slot)); }
            for (slot, asset) in images { self.images.insert((id, slot), asset); }
            changed = true;
        }
        if n.is_some_and(|n| n.layout().is_some_and(|l| l.layout_transition.is_some())) {
            self.layout_transitions.insert(id);
        } else {
            self.layout_transitions.remove(&id);
        }
        let key = n.and_then(|n| n.layout_id().cloned());
        if self.filed_ids.get(&id) != key.as_ref() {
            if let Some(old) = self.filed_ids.remove(&id)
                && let Some(set) = self.layout_ids.get_mut(&old)
            {
                set.remove(&id);
                if set.is_empty() { self.layout_ids.remove(&old); }
            }
            if let Some(key) = key {
                self.layout_ids.entry(key.clone()).or_default().insert(id);
                self.filed_ids.insert(id, key);
            }
        }
        changed
    }
    pub(crate) fn shaders(&self) -> impl Iterator<Item = (NodeId, ShaderNeed)> + '_ { self.shaders.iter().map(|(id, n)| (*id, *n)) }
    pub(crate) fn images(&self) -> impl Iterator<Item = ((NodeId, u8), &Asset)> + '_ { self.images.iter().map(|(k, a)| (*k, a)) }
    pub(crate) fn layout_transitions(&self) -> impl Iterator<Item = NodeId> + '_ { self.layout_transitions.iter().copied() }
    pub(crate) fn has_layout_transition(&self, id: NodeId) -> bool { self.layout_transitions.contains(&id) }
    /// For each layout id carried by two or more nodes: the second-lowest
    /// node (the first one created after the original).
    pub(crate) fn duplicate_layout_ids(&self) -> impl Iterator<Item = (NodeId, &Arc<str>)> + '_ {
        self.layout_ids.iter().filter_map(|(key, ids)| ids.iter().nth(1).map(|id| (*id, key)))
    }
    pub(crate) fn layout_id_nodes(&self, key: &str) -> impl Iterator<Item = NodeId> + '_ {
        self.layout_ids.get(key).into_iter().flatten().copied()
    }
}

impl crate::model::Model {
    /// File `id` again from its props (the only writer besides removal). A
    /// changed frame source restacks and redraws, so the next paint list refreshes them.
    pub(crate) fn index_sources(&mut self, id: NodeId) {
        if self.sources.file(id, self.tree.get(id)) {
            self.changes.restack = true;
            self.changes.paint = true;
        }
    }
    /// `id` is destroyed (`Model::remove_now`).
    pub(crate) fn forget_sources(&mut self, id: NodeId) {
        self.sources.file(id, None);
    }
    /// At a paint list rebuild: the drawn shader clocks and playing images.
    pub(crate) fn refresh_frame_sources(&mut self) {
        self.refresh_shader_clocks();
        self.refresh_playback();
    }
}
```

#### 書き手と呼出元の変更（全部）

| 変更 | 場所 |
|---|---|
| `fn node_clock` を `pub(super)` にする（本体はそのまま） | `model/frame/clocks.rs:80` |
| `sync_shader_clocks` を削除し `refresh_shader_clocks` に置き換え（下記）。module doc の Invariants を「at a paint list rebuild, from `MotionSources`」に直す | `clocks.rs:20-35` |
| `Frame.shader_clocks` の doc を「projection: written only by `Model::refresh_shader_clocks` at a paint list rebuild」に | `model/frame/state.rs:58-60` |
| `fn node_images(n: &HostNode) -> Vec<(u8, Asset)>` を足す（`playing_images` の per-node 本体：content image が `playing != Some(false)` なら `(0, asset)`、background の `FillImage::Image` を `(background_slot(index), asset)`、順序は slot 昇順）。`playback.rs` に `pub(crate) use`不要（`state` は既に `pub(crate) mod`） | `model/playback/state.rs` 末尾 |
| `sync_playback` を `refresh_playback` に改名し、`playing_images` を索引から作る（下記）。module doc の Invariant「An entry exists only for a drawn image that plays」に「as of the last paint list rebuild」を足す | `model/playback/tick.rs:16-74`、`model/playback.rs:9-11` |
| `derive(id)` の末尾で `self.index_sources(id);` | `model/interact/sync.rs:201-222`（`n` の借用が終わった後、関数の最後） |
| `poll_assets` の `if changed {` 内に `self.changes.restack = true;` を足す（`paint = true` は既存） | `sync.rs:262-274` |
| `remove_now` の `for gone in self.remove_slots(id)` の本体に `self.forget_sources(gone);` | `model/apply.rs:577-586` |
| `update_env`：`change(&mut env, ..)` の後、`if env.reduced_motion != self.frame.env.reduced_motion { self.changes.restack = true; self.changes.paint = true; }` を `self.frame.env = env.clone();` より前に | `model/frame/ticks.rs:88-119` |
| `paint_list(&mut self) -> bool`（rebuild したら true）。`let rebuilt = list.state != key;` を if の前に取り、最後に返す | `paint/list/walk.rs:415-469` |
| composition を `let rebuilt = self.paint_list(); if rebuilt { self.refresh_frame_sources(); }` に。`sync_playback()`・`sync_shader_clocks()` の 2 行を削除 | `paint/frame_scene.rs:42-46` |
| `paint/list/self_check.rs:19,33` の `self.paint_list();` は戻り値を無視（`let _ = ` ではなく `self.paint_list();` のままで警告が出るなら `#[must_use]` を付けない） | 変更なし |

```rust
// clocks.rs
pub(crate) fn refresh_shader_clocks(&mut self) {
    let reduced = self.frame.reduced_motion();
    let needs: BTreeMap<NodeId, ClockNeed> = self
        .sources
        .shaders()
        .filter(|(id, _)| self.tree.drawn(*id))
        .filter_map(|(id, need)| Some((id, if reduced { need.reduced? } else { need.moving })))
        .collect();
    self.frame.shader_clocks = needs;
    self.request_clock_frames(self.frame.now());
}
// tick.rs（sync_playback の本体は不変、playing_images だけ）
fn playing_images(&self) -> Wanted {
    if self.frame.reduced_motion() { return Wanted::new(); }
    self.sources
        .images()
        .filter(|((id, _), _)| self.tree.drawn(*id))
        .map(|(key, asset)| (key, asset.clone()))
        .collect()
}
```

#### layout motion を索引で読む

| 変更 | 場所 |
|---|---|
| `capture_layout_motion`：`self.sources.layout_transitions().filter_map(|id| Some((id, self.window_layout_rect(id)?))).collect()` | `model/layout_motion/steps.rs:77-85` |
| `report_duplicate_layout_ids`：`let dups: Vec<(NodeId, Arc<str>)> = self.sources.duplicate_layout_ids().map(|(n, k)| (n, k.clone())).collect();` → 既存どおり `reported_ids.note(id)` が初回なら `DuplicateLayoutId { node, id }` | `mailbox.rs:23-40` |
| `take_arrivals`：departed の key を `BTreeSet` に集め（出力を key 順に保つ）、各 key について `self.sources.layout_id_nodes(&key).filter(|id| !known.contains(id) && self.sources.has_layout_transition(*id)).max()`。見つかれば `departed.remove(&key)` して `(id, rect)` を push | `mailbox.rs:66-98` |

#### 境界条件

| 事象 | 経路 | 結果 |
|---|---|---|
| shader／image／`playing`／layout transition／layout id の追加・変更・削除 | prop の setter → `changed()` が `WRITTEN` → sync の `derive` → `index_sources` →（frame 源が変われば）restack＋paint | 次の composition で list 再構築 → `refresh_frame_sources` |
| canvas scene の shader（`set_scene`） | `apply.rs:78` が `WRITTEN` を付ける → 同上 | 同上 |
| 非表示（`hidden`／`visibility`） | Layout／Hidden の Mutation は `RESTACK` を含み、`WRITTEN` で redraw | refresh で drawn でなくなり除外。再表示で frame 0 から（現行と同じ） |
| 構造変化（insert・move・remove・overlay） | `insert`／`move_before`／`remove_now` が restack＝true | 同上 |
| unmount／削除 | `remove_now` → `forget_sources`（索引から除去）＋restack | playback entry と shader clock は次の refresh で落ちる。step_* が先に走っても `repaint` は存在しない id を無視（現行と同じ） |
| exit 中（Leaving） | drawn のまま（現行の `drawn` の定義どおり）。`remove_now` で除去 | 現行と同じ |
| reduced motion の切替 | `update_env` が restack＋paint | refresh で essential 以外の shader と全 image が止まる。戻すと再開 |
| asset の到着（animated の情報が Ready に） | `poll_assets` の changed → restack（paint は既存） | refresh で entry 作成、`next_switch` を request |
| shader の時間 | 変更なし：clock は `frame.shader_clocks` と `step_shader_clocks`（tick ごとに repaint と次の request）。時間の値は lower 時の `frame.now` | 現行と同じ |
| window close | Model ごと drop（索引も） | — |
| surface 無し | composition が走らない＝refresh も走らない。frame clock 自体が止まる | 現行と同じ |
| DevTools の prop 編集・emulation | 同じ setter／`update_env` を通る | 同上 |

#### 暗黙の状態機械にならない理由

`MotionSources` の各 map は props の**投影**で、書き手は `MotionSources::file`（呼ぶのは `Model::index_sources`＝`derive` からの 1 箇所と `Model::forget_sources`＝`remove_now` からの 1 箇所）だけ。drawn・reduced motion という別の事実は保持せず読むたびに判定する。新しい bool／enum の field は無い（再計算の契機は既存の `Changes.restack`／`paint` を使う）。`Frame::shader_clocks` と `Playback` は既存の状態で、書き手は従来の sync_* が名前を変えた refresh_*（＋既存の step_*）のまま。BTreeMap／BTreeSet は state lint の対象外（旧設計 LAY-2 と同じ扱い）なので marker は付けない。

### 3.2 S-2 layout：丸めのフォールバックを局所化（欠陥 D1 の修正）

`crates/kivact-layout/src/tree.rs:909-973`。

```rust
/// Round the live slots that take part in layout and that no pass has
/// rounded since they were allocated or reset: each such subtree from its
/// nearest rounded ancestor, never the whole tree. A Contents slot never takes
/// part (its children are its flat parent's) and leaves the set; a detached
/// slot stays (attaching it lays it out under a new parent, whose pass rounds
/// it: its origin is NaN).
fn finish_rounding<Tree: LayoutTree + ?Sized>(&mut self, tree: &Tree, state: &LayoutState, measure: &mut dyn LeafMeasurer) {
    let pending = self.pending_rounding(tree);
    if pending.is_empty() { return; }
    self.with_pass(tree, state, measure, |pass| {
        for index in pending {
            if !pass.never_rounded.contains(&index) { continue; } // rounded with an ancestor
            if let Some((top, cum)) = pass.unrounded_top(index) { pass.round_all(top, cum); }
        }
    });
}
/// The never-rounded slots that take part in layout, in index order.
fn pending_rounding<Tree: LayoutTree + ?Sized>(&mut self, tree: &Tree) -> Vec<usize> {
    self.values.never_rounded.retain(|&index| {
        index > 0
            && tree.id_at(index - 1).is_some()
            && !tree.layout_props(index - 1).0.is_some_and(|p| p.display == kivact_types::Display::Contents)
    });
    self.values.never_rounded.iter().copied().collect()
}
```

`LayoutPass` に足す：

```rust
/// The layout parent of taffy index `i` (`None`: a layout root or detached).
fn layout_parent(&self, i: usize) -> Option<usize> {
    let slot = i.checked_sub(1)?;                         // the viewport has none
    let id = self.tree.id_at(slot)?;
    match self.tree.flow(slot) {
        Flow::Overlay => None,
        Flow::Fixed => self.tree.parent(slot).map(|_| 0),  // under the viewport
        Flow::Normal if self.tree.root() == Some(id) => None,
        Flow::Normal => self.tree.parent(slot).map(|p| p + 1),
    }
}
/// The topmost never-rounded slot above (or at) `i` and its parent's cumulative
/// origin, or `None` when that slot is detached.
fn unrounded_top(&self, i: usize) -> Option<(TId, (f32, f32))> {
    let mut top = i;
    loop {
        match self.layout_parent(top) {
            Some(p) if self.origin[p].0.is_nan() => top = p,
            Some(p) => {
                let l = self.unrounded[p].location;
                return Some((TId::from(top), (self.origin[p].0 + l.x, self.origin[p].1 + l.y)));
            }
            None => {
                let slot = top.checked_sub(1);
                let root = top == 0
                    || slot.and_then(|s| self.tree.id_at(s)).is_some_and(|id| self.tree.root() == Some(id))
                    || slot.is_some_and(|s| self.tree.flow(s) == Flow::Overlay);
                return root.then(|| (TId::from(top), (0.0, 0.0)));
            }
        }
    }
}
```

`finish_rounding_at`（overlay、`tree.rs:950-966`）も `pending_rounding` を使い、`id` の subtree に属する pending だけを同じ手順で丸める（`round_all(taffy_id(slot), (0,0))` の全 subtree 走査は、pending が `id` の subtree にある時だけ）。`has_never_rounded`（`:968-973`）は削除。

計算量：通常の pass では pending が空になり O(1)（Contents は最初の retain で永久に抜ける）。新しい slot が丸められずに残る稀な場合だけ、その subtree の大きさに比例。

### 3.3 S-3 layout：`parent_size` を自分が使わないノードでは正規化する

taffy 0.14 で `LayoutInput.parent_size` を読むのは、そのノード自身の size／min／max／margin／padding／border（flex・grid・leaf の各 `compute_*` の冒頭、`taffy-0.14.0/src/compute/flexbox.rs:241-260,555-617`、`grid/mod.rs:64-84`、`leaf.rs:26-62`）と kivact の `compute_child_layout` の presence scale（`tree.rs:405-415`、size の % だけ）と `leaf()` の padding／border（`tree.rs:136-138`）。gap・track は自分の内寸で解決（`flexbox.rs:594,397-405`）。子へは自分の寸法を渡すので、**自分の上記 props が % も calc も持たなければ、出力は `parent_size` に依存しない**。そのノードでは `parent_size` を `Size::NONE` にしてから cache を引く。

- `crates/kivact-layout/src/columns.rs`：`NodeLayoutValues` に列 `pub(crate) relative: Vec<bool>`（doc：「Whether its own box resolves a length against its parent's size: a percentage or calc in its size, min/max size, flex basis, margin, padding, border or inset. Only then do its cache keys keep the parent's size. Written with `styles` only: `push`, `reset`, `rebuild_style`.」）。`Default` に `vec![false]`、`push` で `false`、`reset` で `false`。
- `tree.rs:650-684` `rebuild_style`：`self.values.styles[index] = style;` の直前に `self.values.relative[index] = uses_parent_size(&style);`。
- `tree.rs` に関数（`resolve_dimension` の上）：

```rust
fn relative_length(raw: taffy::style::CompactLength) -> bool {
    use taffy::style::CompactLength;
    matches!(raw.tag(), CompactLength::PERCENT_TAG | CompactLength::CALC_TAG)
}
/// Whether `s` resolves a length of its own box against its parent's size.
fn uses_parent_size(s: &NodeStyle) -> bool {
    let d = |v: taffy::Dimension| relative_length(v.into_raw());
    let a = |v: taffy::LengthPercentageAuto| relative_length(v.into_raw());
    let p = |v: taffy::LengthPercentage| relative_length(v.into_raw());
    d(s.size.width) || d(s.size.height) || d(s.flex_basis)
        || a(s.min_size.width) || a(s.min_size.height) || a(s.max_size.width) || a(s.max_size.height)
        || [s.margin.left, s.margin.right, s.margin.top, s.margin.bottom, s.inset.left, s.inset.right, s.inset.top, s.inset.bottom].into_iter().any(a)
        || [s.padding.left, s.padding.right, s.padding.top, s.padding.bottom, s.border.left, s.border.right, s.border.top, s.border.bottom].into_iter().any(p)
}
```

- `LayoutPass` に `relative: &'a [bool]`、`with_pass`（`tree.rs:814-873`）の分解に `relative` を足して渡す（分解は網羅的なので必須）。
- `compute_child_layout`（`tree.rs:396-400` の `PerformHiddenLayout` の判定の直後）：

```rust
// A box whose own lengths are absolute lays out the same at any parent size:
// its parent's size is left out of its cache keys (taffy keys them by it).
let inputs = if self.relative[usize::from(t)] { inputs } else { LayoutInput { parent_size: taffy::Size::NONE, ..inputs } };
```

  `performed[t]` にはこの正規化後の inputs が入る（`relayout_parents` の `caches[at].get(&inputs)` と一致）。
- overlay の `compute_overlay`（`set_size_bounds` で一時的に長さに置換）と root の `compute_at(fill)`（size を長さに置換）は `relative` を書き換えない：authored が % なら true のまま（保守的に正しい）、% でなければ置換後も長さなので false で正しい。
- module doc（`tree.rs:13-35`）の Invariants に 1 行：「A box whose own lengths are absolute is cached without its parent's size (`relative`).」

効果：row 更新で column の大きな再計算が 9 回 → 5 回（本機 taffy 8.75 → 4.96 ms、slot cache 込み）。

### 3.4 S-4 engine tree：flat 子の cache を slot 添字にする（N8）

`crates/kivact-engine/src/tree/node_tree.rs`。

```rust
/// One flat child: its id and its slot (both are fixed while the list is cached:
/// a child that leaves or is destroyed forgets its parent's list first).
#[derive(Clone, Copy, Debug)]
pub(crate) struct FlatChild { pub(crate) id: NodeId, pub(crate) slot: NodeIndex }

// field（:14 を置換）
flat: RefCell<Vec<Option<Rc<[FlatChild]>>>>,   // indexed by the parent's slot
```

- `fn flat_at(&self, slot: NodeIndex) -> Rc<[FlatChild]>`：`self.flat.borrow().get(slot.0)` が `Some(Some(list))` なら clone を返す。無ければ現行 `flat()`（`:84-108`）の手順で子を集め（各子の `FlatChild { id, slot: index_of(child) }`）、`Vec` を `slot.0 + 1` まで `resize(None)` して格納。
- `flat_children(id)`／`flat_child(id, index)`（`:69-80`）：`index_of(id)` → `flat_at` → `.id` を返す（返り値の型は今と同じで self を借用しない）。`flat()` は削除。
- layout 用に 2 関数：

```rust
/// The slot of the `raw`-th flat child of `slot` (layout's constant-time child access).
pub(crate) fn flat_child_slot(&self, slot: usize, raw: usize) -> usize {
    if let Some(Some(list)) = self.flat.borrow().get(slot) { return list[raw].slot.0; }
    self.flat_at(NodeIndex(slot))[raw].slot.0
}
pub(crate) fn flat_len(&self, slot: usize) -> usize { /* same, .len() */ }
```

- `forget_flat`（`:112`）：`self.flat.get_mut().clear()`。`forget_flat_around(id)`（`:118-130`）：id と祖先の **slot** の entry を `None` に。`remove_with`（`:528`）：`if let Some(e) = self.flat.borrow_mut().get_mut(slot.0) { *e = None; }`（slot 再利用で古い list が残らない）。`install`（`:350-354`）は現行どおり `forget_flat_around`。
- `tree/layout_adapter.rs:37-50`：`child_count` → `self.flat_len(slot)`、`child_at` → `self.flat_child_slot(slot, raw_child)`。

効果：本機 row 更新 taffy −0.8 ms（−10 %）。flat 子は paint walk・a11y・hit でも使うので全般に効く。

### 3.5 S-5 paint：ChildIndex を「動いた子だけ」更新（N9）

- `paint/state.rs`（`impl PaintState`）に：

```rust
/// `child` of `parent` moved or resized in a layout pass: its entry in the
/// parent's child index (if it has one) is looked at again.
pub(crate) fn moved_child(&mut self, parent: NodeId, child: NodeId) {
    if let Some(index) = self.list.indices.get_mut(&parent) { index.touch(child); }
}
```

- `model/interact/geometry.rs:172-190`（`layout()` の geometry 変化の loop）：各 `change` について `let parent = self.tree.get(change.id).and_then(|n| self.tree.flat_parent(n));` を先に `Vec<(NodeId, NodeId)>` に集め、`self.paint.moved_child(parent, change.id)` を呼ぶ（Local・PositionOnly の両方）。
- `paint/list/index.rs:137-159` `refresh`：`relaid` による全件走査を削除。`if self.touched.is_empty() { self.pass = pass; return; }`、touched の entry だけ `*e = Indexed::new(host, e.id, host.node(e.id), layouts.computed_layout(e.id).unwrap_or_default(), generation, e.order)`、最後に `self.pass = pass; self.sort();`。module doc の Invariants（`:16-20`）を「After a layout pass the entries of the children whose layout changed (`PaintState::moved_child`) and of the touched children are looked at again」に。
- 正しさ：entry の location/size/content は geometry 変化の列（`set_final_layout` が丸めた値の変化を全部積む）で必ず通知され、ink の変化は既存の `forget_ink` が通知する。`sort()` の O(子) は浮動小数の 1 走査（3000 で数 µs）で残す。

### 3.6 S-6 compositor の damage（N10・N11）

`crates/arvane-compose/src/plan/damage.rs`、`cache.rs`。

1. **push 命令は自分の変化でだけ damage する**。`cache.rs:63` の index の要素型を

```rust
/// One command of a chunk's index. A group opener (a clip, a transform, a group
/// without blur) draws nothing of its own: it damages its group's bounds only
/// when it changed itself (`hash`), never because its content's extent moved
/// (each content command damages its own place).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Indexed { pub(crate) hash: u64, pub(crate) bounds: Option<PixelRect>, pub(crate) group: Option<PixelRect> }
impl PartialEq for Indexed { fn eq(&self, o: &Self) -> bool { self.hash == o.hash && self.bounds == o.bounds } }
impl Eq for Indexed {}
impl Indexed { fn damage(&self) -> Option<PixelRect> { self.bounds.or(self.group) } }
```

   にし、`ChunkRecord.index: Arc<[Indexed]>`。`chunk()`（`damage.rs:142-154`）で `scene.commands` と zip し、`RenderCmd::PushClip(_) | RenderCmd::PushTransform(_)` と `RenderCmd::PushGroup(g) if g.blur.is_none()` は `Indexed { hash, bounds: None, group: clipped }`、それ以外は `Indexed { hash, bounds: clipped, group: None }`。`diff()`（`:75-125`）の型を `&[Indexed]` にし、`changed.extend(x.1)` を全て `changed.extend(x.damage())` に。`damage.rs:273` の record 生成も同じ型。blur 付き group は従来どおり（子の damage が blur で広がるため）。
2. **window damage を結合する**。`finish_damage`（`:239-249`）で `rects` を作った直後に `let rects = coalesce(rects);`：

```rust
/// Fewer, slightly larger rectangles: the CPU raster passes over a chunk's
/// commands once per rectangle. Overlapping rectangles merge; then all merge
/// into their union when it is at most twice their area.
fn coalesce(mut rects: Vec<PixelRect>) -> Vec<PixelRect> {
    if rects.len() > 64 { return union_px(&rects).into_iter().collect(); }
    'again: loop {
        for i in 0..rects.len() {
            for j in i + 1..rects.len() {
                if overlaps_px(rects[i], rects[j]) {
                    rects[i] = union_px(&[rects[i], rects[j]]).expect("two rects");
                    rects.swap_remove(j);
                    continue 'again;
                }
            }
        }
        break;
    }
    let total: u64 = rects.iter().map(area).sum();
    match union_px(&rects) { Some(u) if rects.len() > 1 && area(&u) <= 2 * total => vec![u], _ => rects }
}
```

   （`overlaps_px`：x・y の区間が重なる。`area`・`union_px` は既存。）以降の `total` と `FULL_AT`／半分超の判定は結合後の矩形で行う。

効果（本機、row 更新）：damage 196k → 15k px、raster 5.92 → 1.80 ms。counter click の raster 0.96 → 0.70 ms。GPU の damage にも同じ矩形が渡る。

### 3.7 S-7 試験 driver の仮想 window：前 frame を in place で更新（N12）

`crates/arvane-virtual/src/window.rs:472-520` `present_with`：

```rust
let n = width as usize * height as usize;
// With damage, the buffer is the frame shown last, updated in place (copied
// only while a mirror or a screenshot still holds it).
let (mut pixels, age) = match std::mem::replace(&mut self.shown, Shown::Nothing) {
    Shown::OnCpuSurface(kept) if damage.is_some() && (kept.width, kept.height) == (width, height) => (kept.pixels, BufferAge::Previous),
    other => { self.shown = other; (std::iter::repeat_n(0u32, n).collect::<Arc<[u32]>>(), BufferAge::Unknown) }
};
let drawn = draw(Arc::make_mut(&mut pixels), age);
if !drawn { /* contract: `draw` returns false only before writing */ self.shown = Shown::OnCpuSurface(Kept { width, height, pixels }); return Ok(false); }
if let Display::Mirrored(real) = &self.display { real.present(width, height, pixels.clone()); }
self.shown = Shown::OnCpuSurface(Kept { width, height, pixels });
Ok(true)
```

`ShellWindow::present_with` の `draw` の doc に「returns false only before it wrote anything」を足す（`arvane/src/presenter/frame_policy.rs:520-534` の唯一の呼出しは常に true）。`Shown::Kept`（GPU の read-back）からは従来どおり全描画。screenshot（`backend.rs:612`）が Arc を持っていれば `make_mut` が複製するので結果は同じ。

効果：counter click・unrelated・row 更新の present 0.70 ms → 数十 µs（本機見積り）。

### 3.8 採らなかった案

- **一般化した relayout boundary（子の全 cache entry を再計算して等しければ親を消さない）**：row 0 が最も広い行になると column の max-content 幅が実際に変わるので、このベンチでは止まらない。taffy cache の entry 追い出しで健全性の証明も要る。費用に見合わない。
- **scroll の既定 style を `flex-basis: 0` 等にする**：既存の layout の意味を変える（E2E の期待値に触れる）。
- **sync_* を毎 frame O(候補) で回す**：image 数が多い画面で O(images) が毎 frame 残る（PERF-1 違反）。

---

## 4. 見積り（目標との対比）

### 4.1 本機 prof の実測（試作）と最終見積り

| KPI | 必達（実装機） | B1（実装機） | 本機 fix 前 | 本機 試作後 | 本機 最終見積り（未試作の S-5・S-7 込み） | 実装機換算の見積り |
|---|---|---|---|---|---|---|
| K3 3000 行 CPU | ≤ 19.87 % | 99.08 % | 99.9 % | **13.4 %** | 13 % | 約 12〜14 %（rows=0 が B1 で 11.2 %） |
| K4 work p95／p99 | ≤ 3.67／4.94 ms | 19.28／21.51 | 20.23／21.47 | **2.61／2.92** | 2.6／2.9 | 約 2.2／2.6 |
| K4 描画間隔 p99 | ≤ 19.17 ms | 22.89 | 22.69 | **18.10** | 18.1 | ≈ rows=0 の 17.4 |
| K5 counter click（総額） | ≤ 1.907 ms | 1.954 | 1.99 | 1.60 | **0.95** | ≈ 0.93 |

K3・K4・counter click は P1 だけで必達を満たす見込み（a11y は無関係か 0.04 ms）。

### 4.2 K5 の予算（a11y と合わせた総額で判定する 2 行）

| 行 | 必達（総額、実装機） | **P1 の予算（非 a11y、実装機換算）** | P1 の見積り（実装機換算） | **P2（a11y）に残る額** | a11y の現状（本機 prof→実装機換算） |
|---|---|---|---|---|---|
| unrelated text 更新 n3000 | ≤ 9.46 ms | **≤ 2.5 ms** | 1.7 ms（本機 2.66 − present 0.70 = 1.96 ÷1.147） | ≤ 6.9 ms | 43.2 → 37.7 ms（5.5 倍の削減が必要） |
| row 更新 n3000 | ≤ 16.96 ms | **≤ 8.0 ms** | 7.4 ms（本機 9.1 − present 0.70 − ChildIndex 0.2 = 8.2 ÷1.109） | ≤ 8.9 ms | 45.0 → 40.5 ms（4.6 倍） |

row 更新の P1 の残り（本機 prof、最終見積り 8.2 ms）：taffy 5.0（column の 5 回の flex 走査＝子数比例、LAY-1 で許容した部分）、raster 1.8、paint_list 0.26、compose 0.19、core turn 0.49、test driver・その他 0.5。release（fat LTO、taffy・kivact-layout は opt-level 3）では prof より速いので、見積りは保守的。

### 4.3 計算量（変更後）

| 経路 | 変更後 |
|---|---|
| composition の frame 源 | list 再構築時だけ O(K·深さ)（K＝shader／image を持つノード数） |
| layout pass の motion | O(T＋I＋D)（T＝layout transition、I＝layout id、D＝departed） |
| 丸め | 変わった祖先の子の比較（LAY-1）＋未丸めの subtree（通常 0） |
| taffy | 変わった祖先ごとに、その祖先が問われる**異なる**入力の数 × 子数（parent_size の重複が消える） |
| ChildIndex | 動いた子数＋sort の 1 走査 |
| damage | 変わった命令の矩形（push は自分が変わった時だけ）、結合 O(r²)（r≤64） |

### 4.4 残る O(N)（本 KPI に効かない、記録のみ）

`settle` の `report_images(全 image)`（`sync.rs:244`）と `poll_assets` の全 image（`:261`）は O(image 数)：image 数千の画面で layout pass ごと／asset 到着ごとに残る。`report_geometry` は O(scrollers＋observed)。`stale_fact`（透明ノード）は flat 子全部に印（`aggregate.rs:146-149`）→ for_each の anchor 直下の行の挿入・削除で O(行数)（mount と行の入替えに効く、P1 の KPI 外）。これらは本 refactor の KPI に効かないので V2 の後続に回す（8 章）。

### 4.5 1 行 12 ノードについて（処理時間の観点）

計測で **12 個中 7 個が `display: contents`**（36,023 中 21,012）。P1 の変更後、frame・turn・layout の定常経路はノード総数に比例しなくなるので処理時間への影響は消える（S-2 で never_rounded からも抜ける）。残る影響は mount（create・insert・sync のノード単位）、a11y の走査（P2）、flat 子の導出（Contents の鎖を降りる）とメモリ（P3）。空の `Option<View>`／空の children／`show` の anchor がノードを作らないようにする判断は P3 に委ねる。

---

## 5. 正しさ・リスク・E2E

### 5.1 リスクと担保

| 変更 | リスク | 担保 |
|---|---|---|
| S-1 | 索引の更新漏れ（WRITTEN を付けない prop 変更） | 全 setter は `Model::set_prop`→`changed()` の最後で `WRITTEN`、`set_scene` も `WRITTEN`、`create` も `WRITTEN`。refresh の漏れ（restack しない drawn 変化）は 3.1 の表で網羅。既存 E2E：`images_animated.rs`（gif_frames_advance、playing_false_shows_first_frame、reduced_motion_shows_first_frame、apng_and_webp_animate）、`shaders.rs` gpu_shader_time_advances（GPU lane）、`shared_layout.rs` 全件、`motion*.rs` |
| S-1 重複 layout id | 報告されるノードが slot 順 → id 順（作成順）に変わる | doc の「the later node (the one created last)」に一致。新 E2E で固定 |
| S-1 capture の順序 | slot 順 → id 順 | `retarget_run` は node ごとに独立（runs map と印付け）。`shared_layout.rs` で担保 |
| S-2 | 丸められない slot が出る | `layout_self_check`（debug、試験 driver が settle ごとに全体再計算と比較）と既存 layout E2E。Contents は layout されないので丸め不要 |
| S-3 | % を見落として cache が誤ヒット | 判定は taffy 0.14 が `parent_size` を読む全 props（3.3）＋inset・flex_basis（読まないが保守的に含める）。`layout_self_check` が fresh 計算と比較する。既存の % layout E2E（layout*.rs） |
| S-4 | cache の取り残し | 子の detach・削除・install・Contents 化で必ず親の entry を消す（現行と同じ箇所）。slot 再利用時は `remove_with` で None |
| S-5 | entry の更新漏れ | 位置・寸法は geometry 変化列、ink は `forget_ink`。debug の `paint_self_check` が fresh walk と比較 |
| S-6 | damage 不足で古い画素が残る | push の変化（clip 矩形・transform・opacity）は hash が変わり group bounds を damage。中身の変化は中身の命令が damage。blur は従来どおり。新 E2E（増分 frame と fresh render の screenshot 一致）で担保 |
| S-7 | screenshot・mirror との共有 | `Arc::make_mut` が共有中は複製。`draw` の契約（false は書く前だけ） |

### 5.2 追加する E2E（実 window、`kivact::test`）

新規 `dev/tests/kivact-public-api/tests/e2e/frame_sources.rs`（`main.rs` の module 一覧に `mod frame_sources;` を 1 行、アルファベット順の位置に）。`canvas_of`・`at`・`expect`・`pixel`・`RED/GREEN/BLUE` は既に `super::paint` から使える（`images_animated.rs:12` と同じ `use`）。`images_animated.rs:42` の `fn gif()` だけを `pub(super) fn gif()` にして `super::images_animated::gif` で使う（本 unit が行う唯一の既存 file の変更）。

| test | 手順と期待 |
|---|---|
| `hidden_gif_stops_and_restarts_from_its_first_frame` | gif を `.hidden(hidden)` で置き、`button("Hide")`／`button("Show")` で signal を切替。初期 frame 0（RED）→ advance 100 ms で GREEN。Hide → `advance(500 ms)`（押下の遷移を流す）→ `let t = screen.frame_ticks()` → `advance(300 ms)` → `frame_ticks() == t`。Show → RED、100 ms 後 GREEN |
| `removed_gif_stops_requesting_frames` | `show(on, ..)` で gif を外す。外した後 500 ms 流し、その後 300 ms で `frame_ticks` が増えない |
| `reduced_motion_toggle_stops_and_resumes_gif` | 再生中に `screen.set_reduced_motion(true)` → RED のまま、ticks 停止。`false` → 100 ms で frame が進む |
| `display_shader_clock_follows_hidden_and_reduced_motion` | CPU（GPU 不要）：`canvas` で `shader.paint().animate(FrameRate::Display)` を描く node。advance 100 ms で `frame_ticks` が 4 以上増える。hidden → 停止。表示に戻す → 再開。`set_reduced_motion(true)` → 停止。同じ構成で `.essential()` を付けた shader は reduced motion でも増え続ける |
| `duplicate_layout_id_names_the_later_node_once` | 同じ `layout_id("dup")` の node を 2 つ（後のものに `test_id("second")`）。`screen.errors()` に "dup" を含む報告が 1 件だけ、2 回 advance しても増えない |
| `incremental_frames_match_a_fresh_render` | `kv_latency` と同じ list（50 行で十分、scroll 内の column に row(text, button)）で Bump を 20 回 click → `screen.screenshot()`。同じ最終状態（label に `!` 20 個）を初期値として新しい Screen を render → screenshot。`assert_eq!` で一致（damage と ChildIndex の部分更新の健全性） |
| `percent_and_absolute_layouts_survive_parent_resize` | % 幅の子と px 幅の子を持つ row を、親の幅が変わる button で 2 回切替。各回 `accessibility_bounds()` の幅が期待値（% は親×割合、px は不変） |

### 5.3 試験の注意

`frame_ticks` は FrameTick の数なので、click の押下・hover の状態遷移が終わるまで（500 ms）流してから基準を取る。shader の CPU fallback は描画されるが clock は renderer に依存しない（`node_clock`）。

---

## 6. unit 分割案

| unit | 内容 | 所有ファイル（他と重複なし） | 依存 | 完了時の軽い確認 |
|---|---|---|---|---|
| P1-U1 | S-1（frame 源と layout motion の索引） | 新 `crates/kivact-engine/src/model/frame/sources.rs`、`model/frame.rs`、`model/frame/clocks.rs`、`model/frame/state.rs`、`model/frame/ticks.rs`、`model/playback.rs`、`model/playback/state.rs`、`model/playback/tick.rs`、`model/aggregate.rs`、`model/interact/sync.rs`、`model/apply.rs`、`model/layout_motion/steps.rs`、`model/layout_motion/mailbox.rs`、`paint/frame_scene.rs`、`paint/list/walk.rs` | なし | `cargo check -p kivact-engine --all-features`、`cargo clippy -p kivact-engine --all-targets --all-features -- -D warnings` |
| P1-U2 | S-2・S-3（丸めの局所化、parent_size の正規化） | `crates/kivact-layout/src/tree.rs`、`columns.rs` | なし | `cargo check -p kivact-layout`、`cargo clippy -p kivact-layout --all-targets -- -D warnings`、`cargo test -p kivact-layout` |
| P1-U3 | S-4（flat 子の slot cache） | `crates/kivact-engine/src/tree/node_tree.rs`、`tree/layout_adapter.rs` | なし（U1 と別ファイル） | U1 と同じ check・clippy |
| P1-U4 | S-5（ChildIndex の部分更新） | `crates/kivact-engine/src/paint/list/index.rs`、`paint/state.rs`、`model/interact/geometry.rs` | なし（U1 は geometry.rs を触らない） | U1 と同じ |
| P1-U5 | S-6（damage） | `crates/arvane-compose/src/plan/damage.rs`、`cache.rs` | なし | `cargo check -p arvane-compose --all-features`、clippy、`cargo test -p arvane-compose --lib` |
| P1-U6 | S-7（仮想 window の in place present） | `crates/arvane-virtual/src/window.rs`（＋`draw` の doc を書く trait 定義の file：`arvane-virtual/src/window.rs:254` 付近の同 file 内なら追加なし） | なし | `cargo check -p arvane-virtual`、clippy |
| P1-U7 | E2E（5.2） | 新 `dev/tests/kivact-public-api/tests/e2e/frame_sources.rs`、`main.rs`（mod 1 行）、`images_animated.rs`（helper の可視性のみ） | U1〜U6 | `cargo test -p kivact-public-api-tests --test e2e --no-run`、`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- frame_sources` |

U1〜U6 は並列可。checkpoint（重い）：全 E2E（images_animated・shaders・shared_layout・motion・layout・compositor・paint の各 file）、`kv_stress`・`kv_latency` の正式計測。

---

## 7. 既存欠陥（79bbd409..db8d679b の実装）

| ID | file:line | 内容 | 再現 | 修正 |
|---|---|---|---|---|
| D1 | `kivact-layout/src/tree.rs:914-973`、`columns.rs:228,254` | `never_rounded` が Contents slot を永久に保持し、taffy が compute する毎 pass `finish_rounding` が全 root を `round_all`（O(N)）。LAY-1 の目的（変わった subtree だけ丸める）を打ち消す。PERF-1 違反 | kv_latency row 更新：`lay.finish_rounding` 6.9 ms／pass、`nr.contents` 21,000 | S-2 |
| D2 | `kivact-engine/src/model/frame/clocks.rs:20-35`、`paint/frame_scene.rs:45` | `sync_shader_clocks` の doc は「paint list rebuild 時」だが毎 composition で全ノード×祖先走査。PERF-1 違反 | stress 3000：8.4 ms／frame | S-1 |
| D3 | `model/playback/tick.rs:20,49`、`frame_scene.rs:44` | `playing_images` が毎 composition で全ノード×祖先走査（reduced motion 以外） | stress 3000：8.9 ms／frame | S-1 |
| D4 | `model/layout_motion/steps.rs:77`、`mailbox.rs:23,66` | layout pass ごとの全走査 3 種。加えて重複 id の報告が slot 順（slot 再利用で作成順と食い違う）で doc（「the one created last」）と不一致（推測：slot 再利用時のみ） | 0.7 ms／pass | S-1 |
| D5 | `arvane-compose/src/plan/damage.rs:142-170` ＋ `plan/bounds.rs:118-135` | push 命令の bounds（中身の和集合）を命令自身の描画範囲として diff するため、group 内の 1 行の幅の変化で group 全体（scroll viewport 全体）が damage | row 更新：damage 101k→480k px、raster 5.9 ms | S-6 |
| D6 | `arvane-virtual/src/window.rs:494-518` | 毎 present で 1.9 MB を 2 回複製（`to_vec` と `Vec→Arc<[u32]>`） | counter click の 35 %（0.7 ms） | S-7 |
| D7 | `paint/list/index.rs:137-159` | layout pass ごとに 3000 entry を全部引き直す（設計どおりだが O(子)） | row 更新 paint_list 0.46 ms | S-5 |
| 参考 | `model/aggregate.rs:146-149` | 透明ノードの `stale_fact` が flat 子全部に印 → for_each の anchor 直下で O(行数)。KPI 外（mount・行の入替え） | — | 後続（8 章） |

---

## 8. 設計者（人間）の判断が要る点

1. **row 更新の総額は a11y 次第**：P1 の見積り 7.4 ms（実装機換算）で予算 8.0 ms 内だが余裕は小さい。P2 が a11y を row 更新で ≤ 8.9 ms、unrelated で ≤ 6.9 ms に下げられない場合、さらに layout を削る手段は「taffy の flex に子の集約値（max-content 寄与の最大・合計）を持たせる fork」級になり、本 refactor の危険に見合うか判断が要る。P1 単独の到達点は row 更新 非 a11y ≈ 7.4 ms、unrelated ≈ 1.7 ms。
2. **compositor（arvane-compose）と試験 driver（arvane-virtual）を P1 の unit に含めること**：K5 の非 a11y の raster／present 部分はここにしかない。他の担当が同じ crate を触る場合は所有の調整が要る。
3. **担当外の観察（wheel／pointer）**：`wheel_n3000` 26〜31 ms は `model/interact/geometry.rs:511-558` `hit_box` が 3000 子それぞれに `shown()`（祖先走査）・`to_parent`・逆行列を行う O(子)。paint の ChildIndex（reach で二分探索）を hit に使う案があるが、ink が「描かれない hit 可能な box」を含むかの確認が要り、P1 では設計していない。wheel（≤ 16.29 ms）と pointer p95 の担当に割り当てること。
4. **4.4 節の残り（image 数・observed 数に比例、透明ノードの stale_fact）**は KPI に効かないので V2 では行わない、でよいか。
