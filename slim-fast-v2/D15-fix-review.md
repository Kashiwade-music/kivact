# V2.2 修正付属書 D15：実装予定の AI のレビュー（`20261110_1828_CODEX_REVIEW.md`）の検証と修正

> **V2.2 修正付属書 D15**。HEAD `6c812296` に対して、レビューの指摘を 6 つの担当（Q1〜Q6）が事実で検証し（多くは試作と実 window の試験で再現）、妥当なものの修正を確定した。
> - **優先順位**：本書は D9〜D14 と D1〜D8 より優先する（V2 設計書 0.1）。本書の節の「修正」は、作業指示書の行の「本書の追加の指示」と同じ扱い。本書と V2 設計書 4 章（DEC）が食い違えば V2 設計書が正。
> - 作業指示書の各 unit の行が、読むべき本書の節を名指す（例：「D15『Q1：R01』」）。節の見出しは `## Q<n>：<レビューの ID>`。
> - 文中の worktree（`W:/w/kv-rf-*`）・scratchpad の path は検証の試作の場所で、実装に必要ではない。試作の差分は参考で、実装は本書の文に従う。
> - 本書が「HEAD で既に失敗している」と書く `diagnostics::why_follows_the_prop_to_the_write_through_a_memo` は W2 の T1 が直す（DEC-26）。


---

# Q1：R01・R02・culling（W3 AH3、W4 AH4）

（元の題：Q1（target lane A）：R01・R02 の検証と修正設計）

対象：レビュー `20261110_1828_CODEX_REVIEW.md` の R01・R02。unit は W3 lane B の AH3（D2 3.6、D9「AH3」）と W4 lane A の AH4（D2 5.3、D9「AH4」）。HEAD `6c812296`。

試作：detached worktree `W:/w/kv-rf-Q1`（target `W:/kv-target/A`、終了後に削除）に D9 の試作の差分 `audit-R1-prototype.patch`（AH1〜AH3・T7・AH2・S3・F7・AH4 を本書の前の形で入れたもの）を当て、その上に本書の修正を入れた。本書の修正だけの差分（試作に対する差分、参考）：`C:\Users\ryo\AppData\Local\Temp\claude\fd3f2b4db3f6db24\76c64dca-56c5-4a94-ab86-dbee8a09d336\scratchpad\rf-Q1-over-prototype.patch`。実装は本書の文に従う。

---

## Q1：R01：非 affine な子孫を通常の祖先の ink で枝刈りする

### 判定：妥当（実際に起こる。実 window の E2E で再現）

事実：
- ink（`paint/list/ink.rs` の `ink`・`ink_local`）は、各 box の ink を自分の `local_transform` で写して親で union する。`local_transform`（`model/transform.rs:197-205`）は自分の 4×4 の `flatten().as_affine()` で、
  1. 親の perspective を含まない（`to_parent` は `step_in`（`transform.rs:283-296`）で親の perspective を掛けるが、ink は掛けない）、
  2. 自分の transform が perspective を含むと `as_affine()` が `None` で、ink は**写さない**（`ink.rs` の `map_rect(None, ..)`）、
  3. preserve-3d の context では 4×4 の合成の flatten ≠ 各段の flatten の合成（例：rotate_y(60) を 2 段重ねると flatten の合成は幅 ×0.25、真の写像は ×0.5）。
- D2 3.6.2 の `may_hit` の免除は「**その子自身の** `to_parent` が非 affine」だけなので、その子の祖先（perspective を持つ親、さらに上の通常の box、own perspective の子を持つ通常の親）は、投影が出ない ink で枝刈りされる。D9 の補正（`hidden` の `None` の再計算）はこれを直さない。
- レビューの「`tree/node.rs:195` の印も立たない」は、子の `Place::transformed` は立つ（`under_perspective && spatial`）ので文言は不正確だが、問題は親（祖先）の側なので結論は変わらない。

再現（D9 の試作のまま、下の AH4 の試験 `hit_reaches_projected_boxes_outside_their_parents`）：
- self_check が click の点で失敗：`hit at (230, 200): pruned Some(NodeId(2)), reference Some(NodeId(4))`（枝刈りは canvas、参照は card）。
- `KIVACT_SELF_CHECK=0` で 3 つの配置（親の perspective／通常の祖先を 1 段挟む／自分の transform に perspective）の click の結果を全て取ると `[[], [], []]`：**3 つとも外れる**（枝刈り無しの参照はどれも card を返す）。

### 修正（W3 lane B の AH3 と W4 lane A の AH4）

方針：ink を計算するとき（paint の既存の cache、同じ無効化）に、**「この subtree には ink の外に描かれる box がある」という 1 bit** を一緒に求めて保持する。hit はその bit が立つ子を枝刈りしない。新しい cache・per-hit の集合は作らない。drawable の免除（D2 の `HitScope.hosts`：hit のたびに全 drawable の祖先の `HashSet` を作る）も同じ bit に載せて消す。

- bit が立つ box：自分の現在の element transform が 2D でない（`!ElementTransform::is_2d()`：z 成分か perspective を含む。親の perspective・3D context・自分の perspective はこれ以外では効かない：perspective は z=0 の平面を動かさず、2D の合成は 2D）、または drawable の root を子に持つ（host の描画が置く）、または clip しない box で子のどれかの bit が立つ。
- clip する box で止める理由：`hit_box` は clip する box の subtree を clip の内（その box の箱 ⊆ その ink）でしか見ない（`in_child_clip` の判定が子の loop より前）。ink の union と同じ構造（`ink_local` は clip する box で子を union しない）。
- 3D でも drawable でもない subtree（K5 の 4 場面を含む通常の UI）の hit は試作と同じ枝刈り。bit の立つ subtree も、降りた先（その box の空間）で子は各自の ink で枝刈りされる。per-hit の費用は試作より減る（`hit_scope` の `HashSet` 構築と、transformed な子ごとの `to_parent` の 4×4 合成が無くなる）。ink の再計算に `element_transform`（変換の無い node は `place.transformed` で即 return）と bool の OR が増えるだけ。`Ink` の大きさは 80 byte のまま（`Option<Drawn>` は bool の niche を使う。同形の型で `size_of` を確認）。
- paint の振る舞いは変えない（paint は `ink()`・`ink_local()` の rect だけを読む。layer の bounds・culling は不変）。
- 無効化：bit は rect と同じ入力（element transform、子の集合と子の ink、clip）から同じ関数で求めるので、rect を捨てる既存の無効化（`forget_ink`、layout の `laid_out` の flat parent、generation・size・content の判定）がそのまま bit も捨てる。paint の self_check（`paint/list/self_check.rs` の `ink_value`）は `Drawn` ごと比べるので、bit の取りこぼしも既存 E2E の全件で検出される。

#### AH3（W3 lane B）の手順の変更（D9「AH3」に対して）

D9「AH3」の 1・2・4・7 はそのまま。3・5・6 を次で置き換え、8・9 を足す。

**3'（`model/interact/hit.rs`）**：D9 の 3 の `use` 6 行と並びはそのまま。ただし付属書 3.6.2 の `HitInk`・`HitScope`・`hit_scope`・`may_hit` と、移した関数の `scope: &HitScope` を次にする。

(a) module doc の Invariants の 1 番目と 2 番目の箇条を次に置き換える（D9 の 4 で geometry.rs から移す 2 箇条はその前に置く）：
```rust
//! - A child's subtree is looked at only when the point lies in the child's
//!   ink (`HitInk`: everything the child and its subtree can draw on, which
//!   holds every box of the subtree as drawn), so a point inside a long list
//!   visits the rows around it, not the list. A child whose ink does not
//!   bound its hits is always looked at: a sticky box (its shift is not in
//!   its ink), and one whose subtree holds a box drawn outside the ink
//!   (`HitReach::Anywhere`: a transform that is not 2D, which a perspective
//!   or a 3D rendering context projects, or a drawable, which its host's
//!   drawing places), with its ancestors up to a box that clips.
//! - The ink is the paint list's (kept until something in the subtree changes
//!   what or where it draws, computed when missing), never stale between
//!   syncs. The debug self-check compares every settled hit with the complete
//!   walk (`hit_reference`), drawables included.
```

(b) `trait HitInk` と、その後の型（`struct HitScope` は作らない）：
```rust
/// The drawing's answer to where a subtree can be hit.
pub(crate) trait HitInk {
    /// Where the box `id` (`n`, laid out as `layout`) and its subtree can
    /// be hit.
    fn hit_ink(&self, id: NodeId, n: &HostNode, layout: &kivact_layout::NodeLayout) -> HitReach;
}

/// Where a subtree can be hit (see [`HitInk`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum HitReach {
    /// Nowhere: it draws nothing (it or an ancestor is hidden).
    Nothing,
    /// Inside its ink: relative to its layout location in its parent's
    /// unscrolled content space, mapped by its own transform.
    Within(Rect),
    /// Anywhere: a box of it is drawn outside its ink.
    Anywhere,
}

/// How a hit test walks the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HitWalk {
    /// Children are left out by their inks.
    Pruned,
    /// Every child is looked at: the debug self-check's reference.
    #[cfg(all(debug_assertions, feature = "testing"))]
    Complete,
}
```
（`Complete` の `#[cfg]` は必須：`hit_reference` が同じ cfg なので、無いと feature 無しの build で `variant Complete is never constructed` の warning になる。release では `HitWalk` は 1 値になり、`walk == HitWalk::Pruned` の比較は消える。）

(c) `const HIT_SLACK` はそのまま。`impl Model { hit_scope, may_hit }` の代わりに（`hit_scope` は作らない）：
```rust
impl Model {
    /// Whether `p` (in the box space of the parent, which scrolls its children
    /// by `scroll`) can hit the subtree of its child `c` (see the module
    /// invariants).
    fn may_hit(&self, c: NodeId, cn: &HostNode, scroll: Point, p: Point) -> bool {
        if cn.place.positioning == crate::tree::node::Positioning::Sticky {
            return true;
        }
        let layout = self.computed_layout(c).unwrap_or_default();
        let ink = match self.hit_ink(c, cn, &layout) {
            HitReach::Nothing => return false, // not drawn, so not shown
            HitReach::Anywhere => return true,
            HitReach::Within(ink) => ink,
        };
        let x = ink.x + layout.location.x - scroll.x;
        let y = ink.y + layout.location.y - scroll.y;
        p.x >= x - HIT_SLACK
            && p.x <= x + ink.width + HIT_SLACK
            && p.y >= y - HIT_SLACK
            && p.y <= y + ink.height + HIT_SLACK
    }
}
```

(d) 移した関数：
- `hit`：`self.hit_with(p, HitWalk::Pruned)`。`hit_reference`：`self.hit_with(p, HitWalk::Complete)`（cfg は付属書のまま）。`fn hit_with(&self, p: Point, walk: HitWalk)`、本体の `hit_node(x, p)` は `self.hit_in(x, p, None, walk)`。
- `hit_any`：`let scope = ..` の行を作らず、2 箇所とも `self.hit_in(.., p, None, HitWalk::Pruned)`。
- `hit_node`：署名を `pub(crate) fn hit_node(&self, id: NodeId, p: Point, walk: HitWalk) -> Option<NodeId>`、本体 `self.hit_in(id, p, None, walk)`。doc の最後の文 `A 3D rendering context is hit by depth ([`Model::hit_context`]).` の後に ` `walk` is the one of the hit test that reaches `id` (a drawable root is reached from its host).` を足す（D9 の 5 の doc の 1 文は足さない）。
- `hit_in`・`hit_box`・`hit_context` の最後の引数は `walk: HitWalk`（値渡し）。互いの呼出しに `walk` を渡す。
- `hit_box` の子の loop の枝刈りの行：`if walk == HitWalk::Pruned && !self.may_hit(c, cn, scroll, p) { continue; }`。
- `hit_box` の drawable の 3 箇所：`self.drawable_hit(id, 1, p, walk)`、`self.drawable_hit(id, 0, p, walk)`、`self.exported_hit(id, p, walk)`。

**5'（`model/drawables/hit.rs`、R02 の修正。下の R02 節）**。

**6'（`paint/list/ink.rs`）**：D9 の 6 の 1 条件は下の `drawn` に含む。`use crate::paint::state::{Ink, Look};` を `use crate::paint::state::{Drawn, Ink, Look};` にし、`pub(super) fn ink(..)` の doc から `pub(super) fn ink_local(..)` の終わりの `}` までを次で置き換える（`ink`・`ink_local` の署名と返す rect は変えない。paint の呼出し元 `walk.rs`・`index.rs` は変えない）：
```rust
/// The ink of `id` (its node `n`, laid out as `layout`) relative to its
/// layout location (`None`: it draws nothing): its reach and, unless it
/// clips, its children's inks at their scrolled locations, mapped by its
/// transform. Kept in the node (see the module invariants).
pub(super) fn ink(
    host: &Model,
    id: NodeId,
    n: &HostNode,
    layout: &NodeLayout,
    generation: Generation,
) -> Option<Rect> {
    drawn(host, id, n, layout, generation).map(|d| d.rect)
}

/// [`ink`], and whether a box of the subtree lands outside it once drawn
/// ([`Drawn::beyond`]).
pub(super) fn drawn(
    host: &Model,
    id: NodeId,
    n: &HostNode,
    layout: &NodeLayout,
    generation: Generation,
) -> Option<Drawn> {
    if let Some(kept) = host.paint.ink[n.index.0].get()
        && kept.generation == generation
        && kept.size == layout.size
        && kept.content == layout.content_size
        // Kept as drawing nothing while it or an ancestor was hidden: once
        // drawn again it is looked at again (showing a node forgets the
        // inks of the changed box and its ancestors, not of those below).
        && (kept.rect.is_some() || !host.tree().drawn(id))
    {
        return kept.rect;
    }
    let drawn = host.tree().drawn(id).then(|| {
        let look = Look::now(host, id, n);
        let local = drawn_local(host, id, n, layout, generation);
        // Blurred by the node's own filters, then transformed by it.
        let rect = map_rect(
            local_transform(host, id, n, layout).as_ref(),
            grow(local.rect, look.fx),
        );
        // A transform that is not 2D is drawn projected (by its own
        // perspective, its parent's or its 3D rendering context's).
        let spatial = host.element_transform(id).is_some_and(|t| !t.is_2d());
        Drawn {
            rect,
            beyond: local.beyond || spatial,
        }
    });
    host.paint.ink[n.index.0].set(Some(Ink {
        rect: drawn,
        size: layout.size,
        content: layout.content_size,
        generation,
    }));
    drawn
}

/// The ink of `id` before its own filters and transform: its own reach
/// and, unless it clips, its children's inks at their scrolled locations
/// (each with its own effects and transform).
pub(super) fn ink_local(
    host: &Model,
    id: NodeId,
    n: &HostNode,
    layout: &NodeLayout,
    generation: Generation,
) -> Rect {
    drawn_local(host, id, n, layout, generation).rect
}

/// [`ink_local`], and whether a box of the subtree lands outside it once
/// drawn: a child's does, or a drawable is mounted on it.
fn drawn_local(
    host: &Model,
    id: NodeId,
    n: &HostNode,
    layout: &NodeLayout,
    generation: Generation,
) -> Drawn {
    let look = Look::now(host, id, n);
    let mut rect = reach_local(host, id, n, layout, &look);
    let mut beyond = false;
    if !n.clips() {
        let scroll = n.scroll();
        for child in host.tree().flat_children(host.tree().nodes.id_at(n.index)) {
            let Some(c) = host.node(child).filter(|c| !c.is_layer()) else {
                continue;
            };
            if c.properties.drawable.is_some() {
                // Placed by this node's drawing, not by its layout.
                beyond = true;
                continue;
            }
            let l = host.computed_layout(child).unwrap_or_default();
            if let Some(d) = drawn(host, child, c, &l, generation) {
                let r = d.rect;
                let mut at = Rect {
                    x: r.x - scroll.x + l.location.x,
                    y: r.y - scroll.y + l.location.y,
                    ..r
                };
                if c.place.positioning == crate::tree::node::Positioning::Sticky {
                    // Wherever its scrollport may stick it.
                    at = sticky_reach(n, layout, &l, at);
                }
                rect = rect.union(at);
                beyond |= d.beyond;
            }
        }
    }
    Drawn { rect, beyond }
}
```

**8（`paint/state.rs`、新しい所有）**：`struct Ink` の field `rect` の型を `Option<Rect>` から `Option<Drawn>` にする（名前・marker の行・doc の行はそのまま。D8 の監査表の `Ink.rect` の行と一致させるため改名しない）。`struct Ink` の閉じ `}` の直後（`/// The compositor layer one listed node is drawn in.` の前）に：
```rust
/// Where a drawn subtree draws (an [`Ink`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Drawn {
    /// Everything it draws on as the paint list places it: each box mapped
    /// by the plane map of its transform.
    pub(super) rect: Rect,
    /// A box in it lands outside `rect` once drawn: one whose transform is
    /// not 2D (a perspective or a 3D rendering context projects it) or a
    /// drawable (its host's drawing places it). Its hits are not bounded
    /// by `rect`.
    pub(super) beyond: bool,
}
```
state-fields：`Drawn` は bool 1 つで数えられない。`Ink` の Option は 1 つのまま（既存の `cache:` の marker）。新しい marker は書かない。

**9（`paint/list/self_check.rs`、新しい所有）**：`use crate::paint::state::{ChildIndex, Ink, PaintList, StaleFacts};` を `use crate::paint::state::{ChildIndex, Drawn, Ink, PaintList, StaleFacts};` にし、`fn ink_value` を次にする（bit も比べる）：
```rust
fn ink_value(ink: Ink) -> (Option<Drawn>, kivact_types::Size, kivact_types::Size) {
    (ink.rect, ink.size, ink.content)
}
```

**`paint/list/hit_reach.rs`（付属書 3.6.3 の内容を置き換え）**：
```rust
//! Role: Answer the hit test's ink query from the paint list's kept inks.
use super::ink::drawn;
use crate::model::Model;
use crate::model::interact::hit::{HitInk, HitReach};
use crate::tree::node::HostNode;
use kivact_layout::NodeLayout;
use kivact_types::NodeId;

impl HitInk for Model {
    fn hit_ink(&self, id: NodeId, n: &HostNode, layout: &NodeLayout) -> HitReach {
        match drawn(self, id, n, layout, self.paint.list.ink_gen) {
            None => HitReach::Nothing,
            Some(d) if d.beyond => HitReach::Anywhere,
            Some(d) => HitReach::Within(d.rect),
        }
    }
}
```

**AH3 の所有ファイル（D9 の「所有ファイルの変更」に対して）**：付属書 P2-U2 ＋ `paint/list/ink.rs`（6'）＋ **`paint/state.rs`（8：`Ink.rect` の型と `struct Drawn` だけ）、`paint/list/self_check.rs`（9：`use` と `ink_value` だけ）、`model/drawables/hit.rs`（5'）**。`HitWalk`・`hit_node` の署名変更の呼出し元は `model/interact/hit.rs` と `model/drawables/hit.rs` だけ（engine 全体を grep で確認済み）で、両方 AH3 の所有。`ink()`・`ink_local()` の署名は変えないので `walk.rs`・`index.rs` は触らない。

**AH3 の完了時の確認**：D9 の 7 の filter のまま。追加で `cargo check -p kivact-engine`（feature なし）で本 unit 由来の warning が無いこと（HEAD 由来の `Indexed.size`・`content` は W2 の F4-fix が消す）。

**閉じ方（state-fields）**：新しい数える field は無い（`HitScope` を作らないので D9 の「`HitScope { prune, hosts }` は 1 つ」の記述は不要になる）。`cargo xtask check --no-write --only state-fields --crate kivact-engine` が exit 0。

#### AH4（W4 lane A）の試験の追加（D9「AH4」のファイルに対して）

`dev/tests/kivact-public-api/tests/e2e/a11y_incremental.rs`：
- `use kivact::prelude::*;` の直前に 2 行：`use kivact::draw::Transform;` と `use kivact::geometry::Point;`。
- module doc の最後の文を `... which also compares the hit under the mouse (where the last click was) and at the window's center with the complete walk, drawables included.` にする。
- file の末尾（`hit_reaches_boxes_outside_their_parents` の後）に次を足す（`pressable`・`clicked`・`Log` は D9 のもの）：

```rust
/// Where a 200 px card turned 45° about x, seen through a perspective of
/// 400 about its centre, is hit outside its untransformed box: its box is
/// at `(x, y)` of the window and the projection spans x − 21.5 to x + 221.5
/// and y + 39.9 to y + 185.9 (`transform_3d::rect_is_the_projected_bounds`),
/// so `(x + 210, y + 180)` lies in its near edge, 10 px right of the box.
fn beyond_the_box(x: f64, y: f64) -> (f64, f64) {
    (x + 210.0, y + 180.0)
}

/// A 200 px card named `name`, turned 45° about x.
fn card(name: &'static str, log: &Log) -> Node {
    pressable(name, log).style(style().size(200, 200).rotate_x(45))
}

#[test]
fn hit_reaches_projected_boxes_outside_their_parents() {
    let log: Log = Rc::default();
    let (screen, ()) = mount({
        let log = log.clone();
        move || {
            let view = canvas_of((
                // The parent's perspective projects the card.
                node()
                    .style(at(20, 20, 200, 200).perspective(400))
                    .child(card("child", &log)),
                // The same, one plain box further down.
                node().style(at(280, 20, 200, 200)).child(
                    node()
                        .style(style().size(200, 200).perspective(400))
                        .child(card("grandchild", &log)),
                ),
                // The card's own transform holds the perspective.
                node().style(at(540, 20, 200, 200)).child(
                    pressable("own", &log).style(
                        style()
                            .size(200, 200)
                            .transform(Transform::new().perspective(400.0).rotate_x(45.0)),
                    ),
                ),
            ));
            ((), view)
        }
    });
    for (name, x) in [("child", 20.0), ("grandchild", 280.0), ("own", 540.0)] {
        let (cx, cy) = beyond_the_box(x, 20.0);
        assert_eq!(
            clicked(&screen, &log, cx, cy),
            [name],
            "{name}: projected out of its parent's box"
        );
    }
}

#[test]
fn hit_reaches_a_projected_box_inside_a_drawable() {
    let log: Log = Rc::default();
    let (screen, ()) = mount({
        let log = log.clone();
        move || {
            let deep = drawable();
            let view = canvas(move |d, _| d.drawable(deep, Point::new(100.0, 100.0)))
                .test_id("stage")
                .style(style().size(800, 600))
                .drawable(deep, move || {
                    node().style(style().size(200, 200)).child(
                        node()
                            .style(style().size(200, 200).perspective(400))
                            .child(card("deep", &log)),
                    )
                });
            ((), view)
        }
    });
    let stage = screen.get_by_test_id("stage").rect().expect("laid out");
    let (x, y) = beyond_the_box(stage.x + 100.0, stage.y + 100.0);
    assert_eq!(
        clicked(&screen, &log, x, y),
        ["deep"],
        "projected out of its parent's box inside a drawable"
    );
}
```

- 試験が見るもの：click の結果（log）に加え、各 click の settle の self_check が、**mouse の位置（＝click の点）**と window 中央で枝刈りあり／なしの hit を比べる（`ArvaneTestHost::mouse_position` は最後の pointer の位置。試作の失敗の文言 `hit at (230, 200)` が click の点であることで確認）。D9 の `compare_hits` はそのままで足りる（「window 中央だけ」というレビューの前提は、D9 で mouse の位置が既に入っているので当たらない）。
- 4 つの場面の対応：親の perspective（`child`）、通常の祖先を 1 段挟む（`grandchild`：canvas → 通常の box → perspective の box → card）、自分の transform に perspective（`own`）、drawable の内部（`deep`：drawable root → perspective の box → card）。
- AH4 の STOP 条件（作業指示書 W4）はそのまま：この 2 本が失敗したら AH3 の 5'・6'・8 の取りこぼしなので、修正せずに報告。

### 確かめたこと（試作・試験の結果）

- 修正前（D9 の試作のまま）：上の 2 本が失敗（R01：self_check `hit at (230, 200): pruned Some(NodeId(2)), reference Some(NodeId(4))`、self_check を止めると 3 配置とも `[]`。drawable：log が `[]` で self_check は**一致**＝R02）。
- 修正後：`cargo clippy -p kivact-engine --all-targets --features testing -- -D warnings` 成功。`cargo check -p kivact-engine`（feature なし）の warning は HEAD 由来の `Indexed.size`・`content` だけ。`cargo xtask check --no-write --only state-fields --crate kivact-engine` exit 0。`a11y_incremental` の 7 本（D9 の 5 本＋本書の 2 本）成功。
- E2E：D9 AH3 の filter（`transform transform_3d pointer hover drawables layout_scroll overlay touch_scroll interaction_drag_edit style_paint`）＋`a11y_incremental` の 101 本が成功。**E2E の全件 637 本**（debug、self_check 有効：hit の比較と paint の ink（`Drawn` ごと）の比較を含む）で、失敗は HEAD で既に失敗している `diagnostics::why_follows_the_prop_to_the_write_through_a_memo` の 1 本だけ（D9「全体」の記録どおり、W2 の T1 が直す）。
- `cargo xtask check --no-write --crate kivact-engine` exit 0（module 規則・行数・state-fields）。
- `Ink` の大きさ：同形の型で `size_of::<Ink>()`・`size_of::<Option<Ink>>()` が修正前後とも 80。

---

## Q1：R02：drawable 内部で参照計算も枝刈りされる

### 判定：妥当（実際に起こる。再現）

事実：`model/drawables/hit.rs` の `exported_hit`（HEAD :71）・`hit_placed`（:157）は `self.hit_node(root, ..)` を呼び、D2・D9 の `hit_node` は `hit_scope(true)` を作るので、`hit_reference` から来た「枝刈りなし」が drawable の root で枝刈りありに戻る。D9 の 5 は「host とその祖先を免除するので正しさは保たれる」としてこのファイルを変えないが、それは drawable 内の ink が正しいことの仮定で、参照の独立性（self_check が誤りを検出できること）にはならない。

再現：R01 の修正の前の試作で、`hit_reaches_a_projected_box_inside_a_drawable` の click が外れ（log `[]`）、self_check は不一致を出さない（枝刈りと参照の両方が drawable の中で同じ子を落とす）。

### 修正（W3 lane B の AH3、上の 5'）

**5'（`model/drawables/hit.rs`）**：探索の方針 `HitWalk` を drawable への再帰に渡す。
- `use crate::model::interact::visual;` の直前に `use crate::model::interact::hit::HitWalk;`。
- `drawable_hit`：
```rust
    /// The deepest drawable of `host` drawn in `slot` (0 or 1) under `p`,
    /// where `p` is in `host`'s box space, walked as `walk`. The latest
    /// drawn answers first.
    pub(crate) fn drawable_hit(
        &self,
        host: NodeId,
        slot: u8,
        p: Point,
        walk: HitWalk,
    ) -> Option<NodeId> {
```
  本体の最後の `find_map` は `self.hit_placed(*id, placed, canvas, p, walk)`。
- `exported_hit`：
```rust
    /// The exported drawable of `host` under `p` (`p` in `host`'s box space),
    /// walked as `walk`. Its root's own map (`export_step`) takes the point
    /// into its box.
    pub(crate) fn exported_hit(&self, host: NodeId, p: Point, walk: HitWalk) -> Option<NodeId> {
```
  本体の `self.hit_node(entry.root, p)` を `self.hit_node(entry.root, p, walk)`。
- `hit_placed`：doc の最後を `... (`to_parent`) takes it into the drawable's box, walked as `walk`.`、引数の最後に `walk: HitWalk,`、本体の最後を `self.hit_node(root, at, walk)`。
- 呼出し元は `model/interact/hit.rs` の `hit_box` の 3 箇所だけ（3' の (d)）。同じ unit。

これで `hit_reference`（`HitWalk::Complete`）は通常描画（`drawable_hit` の slot 0・1 の canvas 配置）・export（`exported_hit`）・3D の layer の中の canvas 配置（`hit_placed`）の全てで末端まで枝刈りしない。R01 の修正で drawable の host とその祖先の免除は ink の bit（drawable の子を持つ box）に移るので、D2 の `HitScope.hosts`（hit ごとの `HashSet`）は無くなる。

### 確かめたこと

- 「drawable 内の ink を過小にした fixture で参照側が対象を返し不一致を検出する」：R02 の修正（5'）を入れたまま R01 の修正の効き目だけを一時的に止め（`hit_reach.rs` で `beyond` を無視）、drawable 内に R01 の誤った上界を作った。結果、`hit_reaches_a_projected_box_inside_a_drawable` は self_check の `hit at (310, 280): pruned Some(NodeId(2)), reference Some(NodeId(6))` で失敗（参照は drawable の中の card を返す）。R02 の修正の前は同じ状況で self_check は一致していた。止めた変更は戻した。
- 修正後、`drawables` の E2E 18 本を含む上の filter と全件が成功（R01 の節）。

---

## Q1：全体

### 他の担当との結合
- **D2・D9 の文言の上書き**：D2 3.6.2 の `HitScope`・`hit_scope`・`may_hit`、3.6.3 の `hit_reach.rs`、3.8 の表の「drawable」「3D・perspective」の行、D9「AH3」の 3・5・6 は本書が優先する（3.8 の 2 行の意味は「host とその祖先」「子の写像が affine でなければ」→「2D でない transform か drawable を含む subtree は、clip する box まで枝刈りしない」）。
- **EN-2（W6 L2、D5 F-4）**：`paint/state.rs` に `struct Drawn`（13 行）が `Ink` の後に入るので、D5・D13 が引く `paint/state.rs` の行番号（`Step` 等）は 13 行ほど下にずれる。EN-2 は行番号ではなく型名・コード片で場所を探すこと（作業指示書の W6 L2 の追加の指示に 1 文足す）。
- **P8（D8 の監査表）**：`paint/state.rs:155 Ink.rect` の行は名前・marker とも変わらない（型だけ `Option<Drawn>`）。書き手は `paint/list/ink.rs` の `drawn`（旧 `ink` の本体）だけ、という根拠もそのまま。
- **T7・AH2（同じ lane の後続）**：触るファイルは重ならない（T7 は geometry.rs・sync.rs、AH2 は counter）。

### wave・lane への影響
- W3 lane B（AH3）の所有に `paint/state.rs`・`paint/list/self_check.rs`・`model/drawables/hit.rs` を足す。W3 の他の lane（A：M3・T3、D：T6・S6、F：G4・S3）の所有・付属書（D3・D7・D10・D11）にこの 3 ファイルは無い（grep で確認）。W2 の F4-fix は `paint/state.rs` の `Indexed` を変えるが W2 で先に終わる（`Ink` の近くではない）。
- W4 lane A（AH4）の所有は変わらない（同じ file に 2 本と import 2 行を足す）。
- 新しい wave・lane・crate は不要。

### 気付いた既存の問題（本担当の外、推測・未再現）
- paint の culling（`paint/list/walk.rs` の `node` の `!visible` の分岐）も同じ ink で subtree を捨てるので、perspective の親の箱が view（window）の完全な外にあり投影された子だけが window に入る場合、その子が描かれない可能性がある。hit とは別の、V2 以前からの振る舞い。本書の `Drawn::beyond` があれば `shows` に `|| beyond` を足す 1 行で直せるが、drawable の host を culling しなくなる副作用（画面外の canvas の描画）があるので、本書では変えない。

### 人間にしか決められない項目
なし。drawable の免除を per-hit の集合から ink の bit に移したのは、R01 の修正と同じ仕組みに載せて内部を一つにし、hit ごとの割当てを消すため（性能は同等以上、API は不変）。

---

## Q1：追加：culling

（「全体」の「気付いた既存の問題」の確定。本節は上の R01 節の「paint の振る舞いは変えない」「`ink`・`ink_local` の署名を変えない・`walk.rs`・`index.rs` は触らない」と、「全体」の「気付いた既存の問題」の段落より優先する。）

### 判定：妥当（実 window の E2E の画素で再現）

試作（D9 の試作＋本書の R01・R02 の修正）に下の E2E を足して実行した。親の箱（x −210〜−10）は window の外で、perspective 400 と rotate_x(45) の子の投影だけが x 11.5 まで window に入る：
- 修正前：`a few children: pixel (4, 202) is ffffff, want ff0000`（描かれない）。
- `walk.rs` だけ直した段階：最初の点は赤になり、子の多い container（40 子、`ChildIndex` を使う経路）で `an indexed container: pixel (4, 482) is ffffff` と失敗した。**walk と index の両方が要る**。
- 両方を直した後：成功。

原因：`paint/list/walk.rs` の `node` は、自分が見えない box を `ink` が clip と重ならなければ subtree ごと捨てる。`paint/list/index.rs` の `Indexed::new` は子の `ink` を `Extent::Within` にして二分探索で探す。どちらも投影の入らない ink を使う（R01 と同じ原因）。V2 以前からの振る舞い。

### 修正（W3 lane B の AH3 に足す）

R01 の 6' を次のとおり変える：
- `pub(super) fn ink(..)`（`drawn(..).map(|d| d.rect)` を返す関数）は**作らない**（呼出し元がすべて `drawn` に移り、残すと dead code の warning になる）。`drawn` の doc を次にする（R01 の 6' の `/// [`ink`], and whether ...` の 2 行の代わり）：
```rust
/// The ink of `id` (its node `n`, laid out as `layout`) relative to its
/// layout location (`None`: it draws nothing): its reach and, unless it
/// clips, its children's inks at their scrolled locations, mapped by its
/// transform; and whether a box of the subtree lands outside it once drawn
/// ([`Drawn::beyond`]). Kept in the node (see the module invariants).
```
  `ink` を呼んでいたのは `walk.rs` の 1 箇所と `index.rs` の 1 箇所だけ（grep で確認済み）で、どちらも下で同じ unit が直す。`ink_local` は残す（`walk.rs` の `plan` が使う）。

**10（`paint/list/walk.rs`、新しい所有）**：
- `use super::ink::{grow, ink, ink_local, local_transform, overlaps, placed, reach};` → `use super::ink::{drawn, grow, ink_local, local_transform, overlaps, placed, reach};`
- `fn node` の `if !visible {` の中の `let shows = ...;` を次にする：
```rust
            // Neither it nor anything in its subtree shows, unless a box of
            // the subtree is drawn outside its ink (projected, or placed by
            // a drawing).
            let shows = drawn(host, id, n, &layout, self.generation).is_some_and(|d| {
                let at = Rect {
                    x: d.rect.x + origin.x,
                    y: d.rect.y + origin.y,
                    ..d.rect
                };
                d.beyond || overlaps(at, clip)
            });
```

**11（`paint/list/index.rs`、新しい所有）**：
- `use super::ink::{grow, ink, overlaps};` → `use super::ink::{drawn, grow, overlaps};`
- `Indexed::new` の `let extent = ...;` を次にする（W2 の F4-fix が同じ関数の `size:`・`content:` の行を先に消すが、この文とは重ならない）：
```rust
        let extent = match n.and_then(|n| drawn(host, id, n, &layout, generation)) {
            Some(d)
                if !d.beyond
                    && [d.rect.x, d.rect.y, d.rect.width, d.rect.height]
                        .iter()
                        .all(|v| v.is_finite()) =>
            {
                Extent::Within(grow(d.rect, SLACK))
            }
            Some(_) => Extent::Anywhere,
            None => Extent::Nothing,
        };
```
- module doc の Invariants の 2 番目の箇条の終わり `a child whose ink is\n//!   not finite reaches everywhere.` を `a child whose ink is\n//!   not finite, or whose subtree draws outside its ink (`Drawn::beyond`),\n//!   reaches everywhere.` にする。

**8 に足す（`paint/state.rs`）**：`enum Extent` の `Anywhere` の doc `/// Its subtree may draw anywhere (its extent is not finite).` を `/// Its subtree may draw anywhere (its extent is not finite, or a box\n    /// of it is drawn outside its ink).` にする。

**性能**：culling を外れるのは `beyond` の立つ subtree（2D でない transform か drawable を含み、clip する box で止まる）の、根から投影・drawable の box までの経路だけ。その中の子は各自の判定で今どおり culling される。見えない box は `own = false` なので自分の描画は増えない（drawable の host も、host の scene は描かず子の判定に降りるだけ）。index は、`beyond` の子を 1 つでも持つ container だけが既存の非有限の extent と同じ扱い（軸の二分探索をやめ、entry ごとの `overlaps` の線形の filter）になる。K5 の場面と kit・bench は 3D の transform も drawable も使わない（grep で確認）ので変わらない。`Ink` の大きさは変わらない。

**AH3 の所有ファイル（最終）**：付属書 P2-U2 ＋ `paint/list/ink.rs`・`paint/state.rs`・`paint/list/self_check.rs`・`model/drawables/hit.rs` ＋ **`paint/list/walk.rs`・`paint/list/index.rs`**。W3 の他の lane（A・D・F）の所有と付属書にこの 2 ファイルは無い（grep で確認）。`walk.rs` を次に触るのは W6 L2 の EN-2（別の wave）で、`node` の culling の行は EN-2 の F-4 の `Step` の変更とは別の場所。

### AH4 に足す E2E

`a11y_incremental.rs`：`use crate::paint::{at, canvas_of};` を `use crate::paint::{RED, at, canvas_of, expect};` にし、file の末尾に足す：
```rust
/// A red 200 px card turned 45° about x under its parent's perspective of
/// 400, the parent's box at `(x, y)`.
fn projected_card(x: i32, y: i32) -> Node {
    node()
        .style(at(x, y, 200, 200).perspective(400))
        .child(node().style(style().size(200, 200).background(rgb(RED)).rotate_x(45)))
}

#[test]
fn a_projection_into_the_window_is_drawn_from_a_box_outside_it() {
    // Geometry tier (±1 px): the parents' boxes end at x = -10, left of the
    // window; the cards' near edges reach x = 11.5 (`beyond_the_box`), so
    // the pixel (4, y + 182) is in each projection.
    let (screen, ()) = mount(|| {
        let mut many: Vec<Node> = (0..39)
            .map(|i| node().style(at(300 + 10 * i, 0, 8, 8)))
            .collect();
        many.push(projected_card(-210, 0));
        let view = canvas_of((
            projected_card(-210, 20),
            // An indexed container: its children are found by their reach.
            node().style(at(0, 300, 800, 300)).child(many),
        ));
        ((), view)
    });
    let shot = screen.screenshot();
    expect(&shot, 4, 202, RED, 2, "a few children");
    expect(&shot, 4, 482, RED, 2, "an indexed container");
}
```
（40 子の container は `INDEXED`（32）以上なので `ChildIndex` の経路を通る。）

### 確かめたこと

walk・index の修正と `ink` の削除の後：`cargo clippy -p kivact-engine --all-targets --features testing -- -D warnings` 成功。`cargo check -p kivact-engine`（feature なし）の warning は HEAD 由来の `Indexed.size`・`content` だけ。`cargo xtask check --no-write --crate kivact-engine` と `--only state-fields` は exit 0。**E2E の全件 638 本**（本節の 1 本を含む。debug、self_check 有効：paint の list・ink・index と hit の比較）で、失敗は HEAD 由来の `diagnostics::why_follows_the_prop_to_the_write_through_a_memo` の 1 本だけ。差分（参考）`rf-Q1-over-prototype.patch` は本節の変更を含む形に更新した。試作の worktree は削除した。


---

# Q2：R03（W6 P8-UI5）・3.4（W6 IN-3）

（元の題：レビュー修正 Q2（target lane B）：R03（DataTable の builder 設定）と 3.4（破棄される part の状態））

対象 HEAD 6c812296。試作は `W:/w/kv-rf-Q2`（detached worktree）で行い、全差分を
`C:\Users\ryo\AppData\Local\Temp\claude\fd3f2b4db3f6db24\76c64dca-56c5-4a94-ab86-dbee8a09d336\scratchpad\rf-Q2-prototype.diff`
に保存した（下の本文のコードと同じもの。実装者は本文のコードを正とし、patch は照合用に使う）。

---

## Q2：R03：DataTable の排他・含意を WORDING だけで閉じている

### 判定：妥当（実際に setter が手で排他・含意を保っている。型で消せて、公開 API も性能も変わらない）

事実（HEAD）：
- `crates/kivact-ui/src/data_table/types.rs:143-149` の `DataTable` は `sort: Option<Sort>`、`multi_sort: Option<Binding<Vec<(usize, SortOrder)>>>`、`page: Option<PagePlan>`、`has_more: Option<Value<bool>>` を別々に持つ。
- `builder.rs:263-270` `sort()` は `multi_sort = None` も書き、`:276-283` `multi_sort()` は `sort = None` も書く。`(Some, Some)` は作れないが、表現からは消えていない（排他）。
- `builder.rs:287-295` `page()` は `has_more = None` も書き、`:318-327` `remote_page()` は `page` と `has_more` を同時に `Some` にする。`(page None, has_more Some)` は作れない（含意）。
- 読み手は 7 箇所：`runtime.rs:64`（`table_paging(&plan, table.has_more)`）、`:112`（`sortable`）、`:152-156`（`table.has_more.is_none()`）、`:182-192`（`paging()` が `table.page`）、`:293`（`table.has_more.is_some()`）、`:423-447`（`sorting()`）、`:459`（`row_indexes()`）。D8 W1 の `ModelPlan.page`／`page_size` と `table_paging()` も同じ事実（page が有るか、Remote か）を別の Option で運ぶ。
- 補足：レビューの「`.has_more(..)`」という公開 setter は存在しない。`has_more` は `.remote_page(page, size, has_more)` の第 3 引数だけである。公開 API は `.sort(..)`・`.multi_sort(..)`・`.page(..)`・`.remote_page(..)`・`.page_sizes(..)` の 5 つで、本修正はどれの署名も変えない。

D14 が E-9 を取り消した 3 つの理由は、本設計では全て成り立たない：
1. 「`page_sizes` を enum に入れると呼出し順の自由と remote での効きを壊す」→ `page_sizes` は enum の外の独立した `Option` のまま残す。`.page_sizes(..)` を先にも後にも呼べ、remote でも行番号の offset に効く（HEAD と同じ）。
2. 「読み手が 4 箇所」→ 7 箇所すべてと builder.rs を同じ unit（P8-UI5）の所有に入れる。
3. 「D8 3.3 の WORDING の規則」→ その規則は「片方が無いと無視される」設定（`expanded_rows` と `row_details` 等）のためのもの。ここは setter 自身が相手を消して排他・含意を保つ、という別の形で、型にすると setter の手の消去（4 行）が消えてコードが短くなる。

性能：変化なし（mount 時に一度だけ読む builder の値。binding の生成数・生成する箇所の条件は HEAD と同じ）。

### 修正（W6 lane L5 の P8-UI5 で行う）

#### 対象 unit と所有ファイル
- **P8-UI5（W6 L5、lane の最後）が行う。** 理由：P8-UI5 は既に `data_table/runtime.rs`・`types.rs` を持ち、D8 W1（`ModelPlan.page`／`page_size`）で同じ paging の事実を作り直す unit である。同じ事実を UI-1 と P8-UI5 で二度設計しない。
- **P8-UI5 の所有に足す**：`crates/kivact-ui/src/data_table/builder.rs`、`dev/tests/kivact-public-api/tests/e2e/ui_data_table.rs`（末尾に試験 6 本と helper 1 個を足すだけ）。
  - 重なりの確認：W5〜W7 のどの unit の所有にも `data_table/builder.rs` と `ui_data_table.rs` は無い（D5・D8・D9〜D14・作業指示書を grep。UI-1 は D14 で builder.rs を外している）。L5 の中でも P8-UI5 以外は持たない。
- **UI-1 は変えない**（D14 のとおり data_table のファイルを持たない）。ただし D14「UI-1 (a)」の E-9 の箇条書きは次の 1 文に置き換える：「E-9 は UI-1 では行わない。P8-UI5 が rf-Q2（R03）の型で行う。」
- D14「1. wave・lane・所有ファイルの組み直し」の P8-UI5 の行の「所有に足す」に、上の 2 ファイルを足す。D14 P8-UI5 節の「`data_table/builder.rs` は持たない（E-9 を行わない）」の文は削除する。

#### 手順（D14 P8-UI5 節の「W1・W2」の箇条書きと、marker 表の `sort`・`multi_sort`・`page`・`has_more`・`page_sizes` の 5 行を、次で置き換える。D8 付録 A-G2 の W1 のコード（`struct PagingPlan { page, size_choice }`）は使わない。W2（`GroupingPlan`）は D8 のとおりで、下の runtime.rs のコードに含めた）

**1. `crates/kivact-ui/src/data_table/types.rs`**

(a) `pub(super) type PagePlan = (Binding<usize>, Value<usize>);` の行の直後に足す：

```rust
/// The caller's sort keys, in priority order.
pub(super) type MultiSort = Binding<Vec<(usize, SortOrder)>>;

/// Whose sort the headers write. `DataTable::sort` and
/// `DataTable::multi_sort` each replace it, so the later call wins.
#[derive(Clone, Copy)]
pub(super) enum SortSetting {
    /// The table's own sort by one column (no sort with `on_reorder`).
    Own,
    /// The caller's sorted column, from `DataTable::sort`.
    Single(Sort),
    /// The caller's ordered keys, from `DataTable::multi_sort`.
    Multi(MultiSort),
}

/// How the rows are paged. `DataTable::page` and `DataTable::remote_page`
/// each replace it, so the later call wins; the page sizes the caller
/// offers stay their own setting, kept by either call.
#[derive(Clone, Copy)]
pub(super) enum PageSetting {
    /// Every row in one list.
    Off,
    /// The table slices the rows it holds.
    Local(PagePlan),
    /// The caller supplies the page's rows and whether another page follows.
    Remote {
        page: PagePlan,
        has_more: Value<bool>,
    },
}
```

(b) `pub struct DataTable` の次の 4 field とその直前の marker 4 行（計 8 行）と、`page_sizes` の直前の marker 1 行を消す：
`sort: Option<Sort>`、`multi_sort: Option<Binding<Vec<(usize, SortOrder)>>>`、`page: Option<PagePlan>`、`has_more: Option<Value<bool>>`。
その位置（`row_selectable` の次、`page_sizes` の前）に次の 3 行を置く：

```rust
    pub(super) sort: SortSetting,
    pub(super) paging: PageSetting,
    // xtask-allow(state-fields): option: DataTable::page_sizes; absent offers no size picker, and it is unused without page or remote_page
```

（`page_sizes` の field 行はそのまま。marker の文は D14 の表の文と同じ。）

**2. `crates/kivact-ui/src/data_table/builder.rs`**

- import（2 行目）を置き換える：
```rust
use super::types::{
    DataTable, DataTableContext, DataTableTreeIndent, Grouping, PageSetting, SortSetting,
    TreeConfig,
};
```
- `data_table(..)` の struct literal の `sort: None,`・`multi_sort: None,`・`page: None,`・`has_more: None,` の 4 行を次の 2 行に置き換える（`page_sizes: None,` は残す）：
```rust
        sort: SortSetting::Own,
        paging: PageSetting::Off,
```
- `sort()`：doc の最後の行 `/// [`on_reorder`](Self::on_reorder), which then keeps the caller's order.` の次に `/// It replaces an earlier [`multi_sort`](Self::multi_sort).` を足し、本体を次にする：
```rust
        self.sort = SortSetting::Single(sort.into_binding());
        self
```
- `multi_sort()`：doc の最後の行 `/// descending, and unsorted. Column indices refer to the original order.` の次に `/// It replaces an earlier [`sort`](Self::sort).` を足し、本体を次にする：
```rust
        self.sort = SortSetting::Multi(sort.into_binding());
        self
```
- `page()`：doc の最後の行 `/// page in `page`, with Previous and Next buttons under the table.` の次に `/// It replaces an earlier [`remote_page`](Self::remote_page).` を足し、本体を次にする：
```rust
        self.paging = PageSetting::Local((page.into_binding(), size.into_value()));
        self
```
- `remote_page()`：doc の最後の行 `/// page is available.` を `/// page is available. It replaces an earlier [`page`](Self::page).` にし、本体を次にする：
```rust
        self.paging = PageSetting::Remote {
            page: (page.into_binding(), size.into_value()),
            has_more: has_more.into_value(),
        };
        self
```
- `page_sizes()` は変えない（doc の「Applies with page or remote_page」は事実どおり）。

**3. `crates/kivact-ui/src/data_table/runtime.rs`**（D8 A-G2 W1・W2 の置換表の代わりに、次を行う。関数・行は名前で探す）

(a) 4 行目の import を置き換える：
```rust
use super::types::{
    DataTable, DataTableContext, PagePlan, PageSetting, Paging, RowName, SortSetting,
};
```

(b) `struct ModelPlan` の定義全体を、次の 4 つの定義で置き換える（`ModelPlan` の `page`・`page_size`・`groups`・`grouping_key` の 4 field と marker 4 行が消え、`paging`・`grouping` になる。`tree` の marker は D8 付録 B の文のまま P8-UI5 が置き換える）：

```rust
/// How the rows are paged once the offered page sizes are applied.
#[derive(Clone, Copy)]
enum PagingPlan {
    Off,
    /// The table slices its rows. `size` is the writable size the footer's
    /// picker shows: the `page_sizes` binding, which shows the first page
    /// when written, or else a read-only view of the size `page` gave.
    Local {
        page: PagePlan,
        size: Binding<usize>,
    },
    /// The caller's rows are the page.
    Remote {
        page: PagePlan,
        has_more: kivact_core::state::Value<bool>,
    },
}

impl PagingPlan {
    /// The page shown and its size, when the table pages its rows.
    fn page(self) -> Option<PagePlan> {
        match self {
            Self::Off => None,
            Self::Local { page, .. } | Self::Remote { page, .. } => Some(page),
        }
    }

    /// The paging the toolbar and footer slots see.
    fn public(self) -> Paging {
        match self {
            Self::Off => Paging::Off,
            Self::Local {
                page: (page, _),
                size,
            } => Paging::Local {
                page,
                page_size: size,
            },
            Self::Remote {
                page: (page, _),
                has_more,
            } => Paging::Remote { page, has_more },
        }
    }
}

/// The page's groups and the key that forms them; both exist exactly when
/// the caller called `DataTable::group_by`.
struct GroupingPlan<R: 'static> {
    groups: super::grouped::Plan<R>,
    key: super::types::Key<R, String>,
}

struct ModelPlan<R: 'static, K: 'static> {
    model: kivact_interaction::DataTableModelApi<R, K, usize>,
    // xtask-allow(state-fields): option: the tree arrangement made from DataTable::tree; absent keeps the model's rows flat in sorted order
    tree: Option<super::tree::Plan<K>>,
    paging: PagingPlan,
    // xtask-allow(state-fields): option: DataTable::group_by, as the page's groups and their key together; absent shows the page's rows ungrouped
    grouping: Option<GroupingPlan<R>>,
    display: kivact_core::state::Value<Vec<R>>,
    page_rows: kivact_core::state::Value<Vec<R>>,
    shown: kivact_core::state::Value<Vec<kivact_interaction::ColumnDef<R, usize>>>,
}
```

(c) `fn mount`：`let paging = table_paging(&plan, table.has_more);` を `let paging = plan.paging.public();` に。`regions(..)` の引数 `plan.groups,` を `plan.grouping.as_ref().map(|grouping| grouping.groups),` に。

(d) `fn model_plan`：
- `let sortable = table.sort.is_some() || table.multi_sort.is_some() || table.reorder.is_none();` を `let sortable = !matches!(table.sort, SortSetting::Own) || table.reorder.is_none();` に。
- `let (page, page_size) = paging(table);` から関数の終わり（`ModelPlan { .. }` の literal の閉じ括弧と `}`）までを次で置き換える：
```rust
    let paging = paging(table);
    if let PagingPlan::Local {
        page: (index, size),
        ..
    } = paging
    {
        model = model.page(index, size);
    }
    let page_rows = model.rows();
    let grouping = table.grouping.take().map(|grouping| GroupingPlan {
        key: grouping.key.clone(),
        groups: super::grouped::plan(page_rows, grouping),
    });
    let display = grouping
        .as_ref()
        .map_or(page_rows, |grouping| grouping.groups.rows());
    let shown = ordered_columns(model.columns(), table.column_order);
    ModelPlan {
        model,
        tree,
        paging,
        grouping,
        display,
        page_rows,
        shown,
    }
}
```
（`key: grouping.key.clone()` を `groups:` より先に書く。field 初期化は記述順に評価される。）

(e) `fn paging` と `fn table_paging`（とその doc `/// The public paging view of the internal page plan.`）を消し、次の 1 関数を置く：
```rust
/// The table's paging, with the size chosen from `page_sizes` in place of
/// the size `page` or `remote_page` gave.
fn paging<R, K>(table: &DataTable<R, K>) -> PagingPlan
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let chosen = table.page_sizes.map(|(size, _)| size);
    match table.paging {
        PageSetting::Off => PagingPlan::Off,
        PageSetting::Local((index, size)) => {
            let picked = chosen.map(|chosen| {
                binding(chosen, move |next: usize| {
                    chosen.set(next.max(1));
                    index.set(0);
                })
            });
            PagingPlan::Local {
                page: (index, picked.map_or(size, |picked| picked.into_value())),
                size: picked.unwrap_or_else(|| binding(move || size.get(), |_| ())),
            }
        }
        PageSetting::Remote {
            page: (index, size),
            has_more,
        } => PagingPlan::Remote {
            page: (index, chosen.map_or(size, |chosen| chosen.into_value())),
            has_more,
        },
    }
}
```
（HEAD と同じ振る舞い：Local の size の binding は `size.set(next.max(1)); index.set(0);` の順。Remote は HEAD でも picker を出さず、選ばれた size の値だけを page の size に使う。HEAD は Remote でも reset 付きの binding を作って値だけを読んでいたので、値は同じ。）

(f) `fn navigation_plan` の `super::metadata::rows(..)` の最後の 3 引数を次にする（`metadata.rs` は変えない）：
```rust
        plan.grouping.as_ref().map(|grouping| grouping.key.clone()),
        plan.paging.page(),
        matches!(plan.paging, PagingPlan::Remote { .. }),
```
同じ関数の `sorting(table, plan.model, plan.page, behavior)` を `sorting(table, plan.model, plan.paging.page(), behavior)` に。

(g) `fn selection_policy` の `if plan.groups.is_some() {` を `if plan.grouping.is_some() {` に。

(h) `fn sorting` の本体の `let sort: super::types::Sort = if let Some(multi_sort) = table.multi_sort {` から関数の終わりまでを次で置き換え、その後に `fn multi_sorting` を足す（コメント 2 行 `// The model sorts by ..` は残す）：
```rust
    let single = match table.sort {
        SortSetting::Own => signal(None).into_binding(),
        SortSetting::Single(sort) => sort,
        SortSetting::Multi(multi_sort) => {
            return multi_sorting(multi_sort, model, page, behavior);
        }
    };
    behavior = behavior.with_sort(binding(single, move |next| {
        single.set(next);
        if let Some((index, _)) = page {
            index.set(0);
        }
    }));
    model.sort(single);
    (behavior, single)
}
/// Sort by the caller's ordered keys; the toolbar, footer and row
/// reordering see the first key.
fn multi_sorting<R, K>(
    multi_sort: super::types::MultiSort,
    model: kivact_interaction::DataTableModelApi<R, K, usize>,
    page: Option<PagePlan>,
    mut behavior: kivact_interaction::DataTableApi<K, usize>,
) -> (
    kivact_interaction::DataTableApi<K, usize>,
    super::types::Sort,
)
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let multi_sort = binding(multi_sort, move |next| {
        multi_sort.set(next);
        if let Some((index, _)) = page {
            index.set(0);
        }
    });
    behavior = behavior.multi_sort(multi_sort);
    model.multi_sort(multi_sort.into_value());
    let primary = binding(
        move || multi_sort.with(|keys| keys.first().copied()),
        move |next: Option<(usize, SortOrder)>| multi_sort.set(next.into_iter().collect()),
    );
    (behavior, primary)
}
```
（`fn sorting` の署名は変えない。`table.sort` は `Copy` なので `&DataTable` から読める。）

(i) `fn row_indexes`：`let page = plan.page;` と `let has_more = table.has_more;` の 2 行を `let paging = plan.paging;` に。memo の中の `let offset = page.map_or(0, |(index, size)| { .. });` を次で置き換える：
```rust
            // A local page past the last one shows the last page.
            let offset = match paging {
                PagingPlan::Off => 0,
                PagingPlan::Local {
                    page: (index, size),
                    ..
                } => index
                    .get()
                    .min(pages.get().saturating_sub(1))
                    .saturating_mul(size.get().max(1)),
                PagingPlan::Remote {
                    page: (index, size),
                    ..
                } => index.get().saturating_mul(size.get().max(1)),
            };
```

(j) `fn grid_plan` の `page: plan.page.map(|(index, _)| index),` を `page: plan.paging.page().map(|(index, _)| index),` に。

(k) `cargo fmt -p kivact-ui`。

#### state-fields の閉じ方
- `DataTable`：`sort: SortSetting`・`paging: PageSetting` は数えない（`Option`/`bool` でなく、型名が phase の接尾辞 `State`/`Phase`/`Status`/`Stage`/… で終わらない）。marker 4 行（sort・multi_sort・page・has_more）が消える。`page_sizes` は `Option` のままで、上の文の marker を置く（D14 の文と同じ。「page も remote_page も無いと使われない」は authored 設定どうしの「片方が無いと無視される」依存で、`expanded_rows` と同じ扱い。呼出し順の自由を保つために enum に入れない）。
- `SortSetting`・`PageSetting`・`PagingPlan` の variant の field（`Binding`・`Value`・`PagePlan` の tuple）は数えない。`GroupingPlan` は `Option` を持たない。marker 不要。
- `ModelPlan`：数える field は `tree`・`grouping` の 2 個で、両方 marker 付き。D8 付録 B の `ModelPlan.page`（`option: DataTable::page or remote_page with any page_sizes choice; …`）の行は**置かない**（field が無い）。`page_size`・`grouping_key` の行も field が無い。
- D14 P8-UI5 の marker 表から `sort`・`multi_sort`・`page`・`has_more` の 4 行を消す。`page_sizes` の行は残す。
- `Paging`（公開 enum）と `DataTableContext.sort` は変えない。

#### 試験（`dev/tests/kivact-public-api/tests/e2e/ui_data_table.rs` の末尾に足す。既存の import・`people()`・`columns()`・`options()`・`names()` を使う）

```rust
/// Mount the people table as `configure` sets it up, with a page binding at
/// 0, a size of 2 and a remote `has_more` of true.
fn mount_paged(
    configure: impl FnOnce(
        kivact::ui::DataTable<Person, u32>,
        Signal<usize>,
        Signal<usize>,
        Signal<bool>,
    ) -> kivact::ui::DataTable<Person, u32>
    + 'static,
) -> (Screen, (Signal<usize>, Signal<usize>)) {
    mount_with(options(), move || {
        let page = signal(0usize);
        let size = signal(2usize);
        let has_more = signal(true);
        let table = data_table(people(), |p: &Person| p.id, columns());
        ((page, size), configure(table, page, size, has_more))
    })
}

#[test]
fn a_later_remote_page_replaces_page() {
    let (screen, _) =
        mount_paged(|table, page, size, more| table.page(page, size).remote_page(page, size, more));
    // The caller's rows are the page: nothing is sliced and the total is unknown.
    assert_eq!(names(&screen), ["Ada", "Grace", "Linus", "Alan", "Edsger"]);
    screen.get_by_text("Page 1");
    assert!(!screen.get_by_role(Role::Button, "Next").is_disabled());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_later_page_replaces_remote_page() {
    let (screen, (page, _)) = mount_paged(|table, page, size, more| {
        table
            .page(page, size)
            .remote_page(page, size, more)
            .page(page, size)
    });
    // The table slices its rows again and knows the page count.
    assert_eq!(names(&screen), ["Ada", "Grace"]);
    screen.get_by_text("Page 1 of 3");
    screen.get_by_role(Role::Button, "Next").click();
    assert_eq!(page.get_untracked(), 1);
    assert_eq!(names(&screen), ["Linus", "Alan"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn page_sizes_given_before_page_offer_the_same_picker() {
    let (screen, (page, size)) = mount_paged(|table, page, size, _| {
        table.page_sizes(size, vec![2, 5]).page(page, size)
    });
    page.set(1);
    screen.run_until_idle();
    screen.get_by_text("Page 2 of 3");
    screen.get_by_role(Role::ComboBox, "Rows per page").click();
    screen.get_by_text("5").click();
    assert_eq!(size.get_untracked(), 5);
    assert_eq!(page.get_untracked(), 0);
    assert_eq!(names(&screen), ["Ada", "Grace", "Linus", "Alan", "Edsger"]);
    screen.get_by_text("Page 1 of 1");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn page_sizes_set_the_size_of_a_remote_page_in_either_order() {
    let (screen, _) = mount_paged(|table, page, size, more| {
        let chosen = signal(10usize);
        page.set(3);
        table
            .page_sizes(chosen, vec![10, 20])
            .remote_page(page, size, more)
            .row_index(1)
    });
    // The chosen size (10), not the given one (2), numbers the rows of page 4.
    screen.get_by_text("31");
    assert!(screen.query_by_role(Role::ComboBox, "Rows per page").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_later_multi_sort_replaces_sort() {
    let (screen, (single, keys)) = mount_with(options(), || {
        let single = signal(Some((0, SortOrder::Descending)));
        let keys = signal(vec![(1, SortOrder::Ascending)]);
        (
            (single, keys),
            data_table(people(), |p: &Person| p.id, columns())
                .sort(single)
                .multi_sort(keys),
        )
    });
    // The keys sort the rows; the single binding is not read.
    assert_eq!(names(&screen), ["Linus", "Ada", "Alan", "Grace", "Edsger"]);
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    screen
        .get_by_role(Role::ColumnHeader, "Name")
        .press_key_with(Key::Enter, shift);
    assert_eq!(
        keys.get_untracked(),
        [(1, SortOrder::Ascending), (0, SortOrder::Ascending)]
    );
    assert_eq!(single.get_untracked(), Some((0, SortOrder::Descending)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_later_sort_replaces_multi_sort() {
    let (screen, (single, keys)) = mount_with(options(), || {
        let single = signal(None::<(usize, SortOrder)>);
        let keys = signal(vec![(1, SortOrder::Ascending)]);
        (
            (single, keys),
            data_table(people(), |p: &Person| p.id, columns())
                .sort(single)
                .multi_sort(keys)
                .sort(single),
        )
    });
    // The single binding sorts the rows; the keys are not read.
    assert_eq!(names(&screen), ["Ada", "Grace", "Linus", "Alan", "Edsger"]);
    screen.get_by_role(Role::ColumnHeader, "Name").click();
    assert_eq!(single.get_untracked(), Some((0, SortOrder::Ascending)));
    assert_eq!(keys.get_untracked(), [(1, SortOrder::Ascending)]);
    assert_eq!(names(&screen), ["Ada", "Alan", "Edsger", "Grace", "Linus"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```

対応：local→remote（`a_later_remote_page_replaces_page`）、local→remote→local（`a_later_page_replaces_remote_page`）、single→multi（`a_later_multi_sort_replaces_sort`）、single→multi→single（`a_later_sort_replaces_multi_sort`）、page size を先に（`page_sizes_given_before_page_offer_the_same_picker`。後に設定する形は既存の `row_numbers_follow_the_displayed_page_and_page_size_changes_reset_it`）、remote で page size を先に（`page_sizes_set_the_size_of_a_remote_page_in_either_order`。後は既存の `a_remote_page_does_not_slice_the_rows_again_and_uses_has_more` の形）。どれも HEAD の意味を固定する回帰試験で、HEAD でも型化の後でも通る（下の「確かめたこと」）。

#### 完了時の確認（D8 6 章 P8-UI5 の行に足す）
- `cargo check -p kivact-ui -p kivact-ui-basic -p kivact-ui-soft -p kivact-ui-pop -p kivact-ui-creative -p kivact-ui-game -p kivact-public-api-tests --all-targets`
- `cargo clippy --no-deps -p kivact-ui --all-targets -- -D warnings`（`--no-deps`：HEAD の `kivact-engine` に W1 由来の dead_code 警告 `Indexed.size`/`content` があり、`--no-deps` が無いと依存の警告で失敗する。W1 F5 の閉じ方の担当の件で、本件と無関係）
- `cargo xtask check --no-write --only state-fields --crate kivact-ui`
- `dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- ui_data_table:: apg_grid::group`
- `dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-soft --test consumer -- grouped_table`

### 確かめたこと（試作・試験の結果）
- 試作（`W:/w/kv-rf-Q2`、`CARGO_TARGET_DIR=W:/kv-target/B`）：`cargo clippy --no-deps -p kivact-interaction -p kivact-ui --all-targets -- -D warnings` 成功。`cargo xtask check --no-write --only state-fields --crate kivact-ui` 成功（exit 0、出力なし）。
- 実 window（`dev/tools/window-lock.sh` 経由）：`ui_data_table::`（既存 22 本＋新 6 本）、`apg_grid::group_disclosures_participate_in_grid_arrow_navigation` すべて成功。
- 初版の `a_later_sort_replaces_multi_sort` は Shift+Enter で単一 sort を期待して失敗した（単一 sort の table では Shift 付きの活性化は sort しない。HEAD と同じ振る舞い）。click に直した上の本文で成功。
- HEAD のソース（試作の crates/ の変更を stash し、試験だけを残して実行）でも新しい 6 本はすべて成功した。型化は振る舞いを変えない（試験は意味の固定）。
- 試作で public-API の E2E 全体（629 本）を実行：628 成功、失敗 1 本は `diagnostics::why_follows_the_prop_to_the_write_through_a_memo`（node 番号 4 と 5 の差）。この試験は HEAD のソースでも同じく失敗し、本件の変更（DataTable・interaction の part を使わない app）と無関係（既存の失敗。親へ申し送り）。`cargo test -p kivact-ui-soft --test consumer -- grouped_table` の 2 本も成功。

---

## Q2：3.4：破棄される content の状態を親が保持する疑い

### 判定：妥当（実際に起こる。実 window で `UiError(Disposed)` の panic を再現した）

事実：
- `crates/kivact-interaction/src/disclosure.rs:221-239` `content()` は `part(..)` で part の状態（memo）を**呼んだ時の owner** に作り、`:228` で `DisclosureApi.content_state`（disclosure の owner の signal）に `Some(part_state)` を入れる。doc（`:44-47`）が勧める `show(details.is_open(), move || node().part(details.content()))` では、その owner は `show` の枝で、閉じると破棄される。slot は `Some(破棄された Value)` のまま残る。
- `:103-113` `content_state()` の memo は `self.content_state.get().map(|state| state.get())` を読む。
- 試作の E2E（実 window、HEAD のコード）で 3 通りとも panic した：
  - setup で `let state = details.content_state();` を作り、text で読む（最も普通の使い方）→ **最初に閉じた時点で** `kivact misuse: UiError(Disposed { .. }): Value read after its owner was disposed at crates\kivact-interaction\src\disclosure.rs:107`。
  - content を閉じた後に、新たに `content_state()` を作って読む → 同じ panic（`:107`）。
  - `show` の中の trigger を unmount した後に `trigger_state()` を読む → 同じ panic（`:95`）。
- 同じ形（part を作る関数が、作った owner より長生きする `Signal<Option<Value<S>>>` に part の状態を入れ、外さない）は kivact-interaction の 21 箇所にある：`disclosure.rs`（trigger・content）、`popup/api.rs`（`base_trigger`・`content_with`）、`tooltip/api.rs`（trigger・content）、`combobox/handle.rs`（input・content）、`select/handle.rs`（content_in）、`menu/handle.rs`（level の content）、`dialog.rs`（content）、`toggle.rs`（2 箇所）、`zoom_pan/handle.rs`、`drag/handle.rs`、`splitter/handle.rs`、`space_pan/handle.rs`、`scalar/handle.rs`、`text_edit/handle.rs`、`list/parts.rs`（container）、`table/handle.rs`（grid）、`calendar/handle.rs`（grid）。popup・tooltip・menu・select・combobox・dialog の content は「開くたびに新しい owner に mount する」設計（`popup/api.rs:61-69` の doc）なので、親が `content_state()` を読めば閉じた時点で同じ panic になる（popup は下の試験で HEAD の失敗を確かめる）。
- 既に正しい前例：`list/parts.rs:197-205` と `calendar/handle.rs:198-204` は、項目ごとの状態を mount の `on_cleanup` で表から外している。slot 1 個の 21 箇所だけが外していない。
- D14 IN-1 の「P5-F 3 章の推測（content_state は unmount で None に戻らない）は W6 では変更しない」は、この再現により取り消す。

### 修正（W6 lane L4 の IN-3 で行う）

方針：slot は「その part を作った owner が生きている間だけ、その part の状態を持つ」。書き手は part を作る関数（set）と、同じ owner の終わり（clear）の 2 つで、どちらも同じ owner に属する。新しい field・flag は作らない（暗黙の状態を増やさない）。読み手（`content_state()` 等の memo）は変えず、clear で `None` になると既存の fallback（`shown`・`is_open()`・`default`）に戻る。性能：part を作るたびに `on_cleanup` を 1 個登録するだけ（mount 時のみ。frame ごとの処理なし）。

#### 対象 unit と所有ファイル
- **IN-3（W6 L4、IN-1 → IN-2 の後、P8-IN の前）が行う。**
- **IN-3 の所有に足す**（全て `crates/kivact-interaction/src/`）：`part_slot.rs`（新規）、`lib.rs`、`select/handle.rs`、`menu/handle.rs`、`dialog.rs`、`toggle.rs`、`zoom_pan/handle.rs`、`drag/handle.rs`、`splitter/handle.rs`、`space_pan/handle.rs`、`scalar/handle.rs`、`text_edit/handle.rs`、`list/parts.rs`、`table/handle.rs`、`calendar/handle.rs`。テスト：`dev/tests/kivact-public-api/tests/e2e/interaction_toggle.rs`（末尾に試験 3 本と helper 1 個を足すだけ）。
  - 重なりの確認：`select/handle.rs` は IN-1、`menu/handle.rs`・`lib.rs` は IN-2、`dialog.rs`・`drag/handle.rs` は P8-IN も持つが、全て L4 の中で、IN-1 → IN-2 → IN-3 → P8-IN の順に前の commit の上で別の行を変える（IN-3 が変えるのは各ファイルの `…set(Some(part_state))` の 1 行と `lib.rs` の `mod` 1 行。P8-IN は marker 行だけ）。L1〜L3・L5・L6 の所有に kivact-interaction のファイルは無い（D14 L4 の全体と D5・D8 6 章を grep）。`interaction_toggle.rs` を所有する W6 の unit は無い（UI-3・P8-CK は確認で走らせるだけ）。W5 の G7 は `menu/handle.rs` の別の hunk（W6 より前）。W7 の FINAL-1・G9 は W6 の後。
- D14「1. 組み直し」の表の IN-3 の行の「所有に足す」に上のファイルを足す。D14 IN-1 節の最後の箇条書き（「P5-F 3 章の推測（`content_state` は unmount で None に戻らない）は W6 では変更しない（試験も足さない。CP2 の全 E2E で見る）」）は、「この推測は rf-Q2（3.4）で実 window で再現した。IN-3 が直す。IN-1 は `select/handle.rs` の slot の書き方を変えない」に置き換える。

#### 手順（IN-3 の「本書の追加の指示」に足す。D14 IN-3 の marker の作業の前に行う）

1. 新しいファイル `crates/kivact-interaction/src/part_slot.rs`（全文）：
```rust
//! Role: The slot through which a handle reads the state of its attached part.
use kivact_core::state::{Signal, Value, on_cleanup};

/// Put the state of a part being made in `slot`, and empty the slot when the
/// owner making it ends.
///
/// The state is owned by that owner (the trigger's or the content's mount,
/// often a `show` branch), so the slot holds it exactly while it lives: a
/// handle that reads the slot then falls back to its own state after the
/// part unmounts, and reads the new part after it mounts again.
pub(crate) fn attach_part_state<S: 'static>(slot: Signal<Option<Value<S>>>, state: Value<S>) {
    slot.set(Some(state));
    on_cleanup(move || {
        // The handle's own owner may have ended first, with the slot.
        if slot.try_get_untracked().is_some() {
            slot.update(|attached| {
                if *attached == Some(state) {
                    *attached = None;
                }
            });
        }
    });
}
```
（`== Some(state)` の比較：新しい mount が先に slot を書き換えていたら、古い mount の終わりは新しい状態を消さない。`Value<T>: PartialEq` は `kivact-runtime/src/reactive/data.rs:670`。）

2. `crates/kivact-interaction/src/lib.rs`：`mod options_cell;` の次の行に `mod part_slot;`。

3. 次の 21 行を置き換える（各行の左の式を、同じ字下げで右の式に。行番号は HEAD のもの。名前で探す）：

| ファイル（`crates/kivact-interaction/src/`） | 関数 | HEAD の行 | 置き換え後 |
|---|---|---|---|
| `disclosure.rs` | `DisclosureApi::trigger` | `self.trigger_state.set(Some(part_state));`（208） | `crate::part_slot::attach_part_state(self.trigger_state, part_state);` |
| `disclosure.rs` | `DisclosureApi::content` | `self.content_state.set(Some(part_state));`（228） | `crate::part_slot::attach_part_state(self.content_state, part_state);` |
| `popup/api.rs` | `PopupApi::base_trigger` | `self.trigger_state.set(Some(part_state));`（421） | `crate::part_slot::attach_part_state(self.trigger_state, part_state);` |
| `popup/api.rs` | `PopupApi::content_with` | `self.content_state.set(Some(part_state));`（463） | `crate::part_slot::attach_part_state(self.content_state, part_state);` |
| `tooltip/api.rs` | `TooltipApi::trigger` | `self.trigger_state.set(Some(part_state));`（281） | `crate::part_slot::attach_part_state(self.trigger_state, part_state);` |
| `tooltip/api.rs` | `TooltipApi::content` | `self.content_state.set(Some(part_state));`（322） | `crate::part_slot::attach_part_state(self.content_state, part_state);` |
| `combobox/handle.rs` | `ComboboxApi::input` | `self.input_state.set(Some(part_state));`（572） | `crate::part_slot::attach_part_state(self.input_state, part_state);` |
| `combobox/handle.rs` | `ComboboxApi::content` | `self.content_state.set(Some(part_state));`（643） | `crate::part_slot::attach_part_state(self.content_state, part_state);` |
| `select/handle.rs` | `SelectApi::content_in` | `self.content_state.set(Some(part_state));`（598） | `crate::part_slot::attach_part_state(self.content_state, part_state);` |
| `menu/handle.rs` | `MenuApi::content` | `level.content_state.set(Some(part_state));`（474） | `crate::part_slot::attach_part_state(level.content_state, part_state);` |
| `dialog.rs` | `DialogApi::content` | `self.content_state.set(Some(part_state));`（305） | `crate::part_slot::attach_part_state(self.content_state, part_state);` |
| `toggle.rs` | `ButtonApi::control` | `self.observed.set(Some(part_state));`（83） | `crate::part_slot::attach_part_state(self.observed, part_state);` |
| `toggle.rs` | `Core::control` | `self.observed.set(Some(part_state));`（379） | `crate::part_slot::attach_part_state(self.observed, part_state);` |
| `zoom_pan/handle.rs` | `ZoomPanApi::viewport` | `self.observed.set(Some(part_state));`（270） | `crate::part_slot::attach_part_state(self.observed, part_state);` |
| `drag/handle.rs` | `DragApi::source_with` | `self.observed.set(Some(part_state));`（644） | `crate::part_slot::attach_part_state(self.observed, part_state);` |
| `splitter/handle.rs` | `SplitterApi::handle` | `self.observed.set(Some(part_state));`（435） | `crate::part_slot::attach_part_state(self.observed, part_state);` |
| `space_pan/handle.rs` | `SpacePanApi::control` | `self.observed.set(Some(part_state));`（65） | `crate::part_slot::attach_part_state(self.observed, part_state);` |
| `scalar/handle.rs` | `ScalarApi::part` | `self.edit.observed.set(Some(part_state));`（386） | `crate::part_slot::attach_part_state(self.edit.observed, part_state);` |
| `text_edit/handle.rs` | `TextEditApi::control` | `self.data.get().observed.set(Some(state));`（407） | `crate::part_slot::attach_part_state(self.data.get().observed, state);` |
| `list/parts.rs` | `container` | `state.container_state.set(Some(part_state));`（19） | `crate::part_slot::attach_part_state(state.container_state, part_state);` |
| `table/handle.rs` | `DataTableApi::grid` | `state.grid_state.set(Some(part_state));`（337） | `crate::part_slot::attach_part_state(state.grid_state, part_state);` |
| `calendar/handle.rs` | `CalendarGridApi::grid` | `self.state().grid_state.set(Some(part_state));`（122） | `crate::part_slot::attach_part_state(self.state().grid_state, part_state);` |

置き換えの後、`grep -rn "set(Some(part_state))\|observed.set(Some(state))" crates/kivact-interaction/src` が 0 件であること。他の `Signal<Option<Value<..>>>`（`toggle.rs`・`drag/handle.rs` の `disabled`、`table/model.rs` の `query`）は caller が setup で渡す値で、part の owner に属さないので変えない。

4. marker の文（D14 IN-3 の文のうち、「一度 mount したら残る」と読める 4 本を次に置き換える。他の IN-3・IN-1・IN-2 の文は事実どおりのままで変えない）：
- `disclosure.rs` `DisclosureApi.content_state`：`    // xtask-allow(state-fields): independent: set by DisclosureApi::content while the content part is mounted and emptied when that mount ends; a closed disclosure has its trigger and no content`
- `popup/api.rs` `PopupApi.content_state`：`    // xtask-allow(state-fields): independent: set by PopupApi::content_with while the surface is mounted and emptied when it unmounts; the trigger has attached by then or is absent`
- `popup/api.rs` `PopupApi.trigger_state`：`    // xtask-allow(state-fields): independent: set by PopupApi::base_trigger at the trigger's mount; while the popup is closed there is a trigger and no content`
- `tooltip/api.rs` `TooltipApi.trigger_state`：`    // xtask-allow(state-fields): independent: set by TooltipApi::trigger at mount; the hint content is mounted only while the tooltip shows`
（4 本とも workspace で一意。X5 に掛からない。X4 は independent の文の関数名を調べない。）

5. `cargo fmt -p kivact-interaction`。

#### 試験（`dev/tests/kivact-public-api/tests/e2e/interaction_toggle.rs` の末尾に足す。既存の `use crate::support::mount;`・`use kivact::prelude::*;`・`fn control(label: &str) -> Node` を使う）

```rust
/// The text a parent draws from a part state it reads through the handle.
fn open_text(label: &'static str, open: bool) -> String {
    format!("{label} open {open}")
}

#[test]
fn a_parent_reads_the_disclosure_content_state_across_unmount_and_remount() {
    use kivact::interaction::use_disclosure;
    let (screen, (open, late)) = mount(|| {
        let open = signal(true);
        let late = signal(false);
        let details = use_disclosure(open);
        // Read at setup, before the content first mounts.
        let early = details.content_state();
        (
            (open, late),
            column((
                control("Details").part(details.trigger()),
                show(details.is_open(), move || {
                    control("Body").part(details.content())
                }),
                text(move || open_text("early", early.get().open)),
                // Read for the first time after the content unmounted.
                show(late, move || {
                    let state = details.content_state();
                    text(move || open_text("late", state.get().open))
                }),
            )),
        )
    });
    screen.get_by_text(&open_text("early", true));
    open.set(false);
    screen.run_until_idle();
    screen.get_by_text(&open_text("early", false));
    late.set(true);
    screen.run_until_idle();
    screen.get_by_text(&open_text("late", false));
    open.set(true);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Details");
    screen.get_by_text(&open_text("early", true));
    screen.get_by_text(&open_text("late", true));
    open.set(false);
    screen.run_until_idle();
    screen.get_by_text(&open_text("late", false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_parent_reads_the_disclosure_trigger_state_after_the_trigger_unmounts() {
    use kivact::interaction::use_disclosure;
    let (screen, (open, has_trigger)) = mount(|| {
        let open = signal(false);
        let has_trigger = signal(true);
        let details = use_disclosure(open);
        let state = details.trigger_state();
        (
            (open, has_trigger),
            column((
                show(has_trigger, move || {
                    control("Details").part(details.trigger())
                }),
                text(move || open_text("trigger", state.get().open)),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Details").click();
    assert!(open.get_untracked());
    screen.get_by_text(&open_text("trigger", true));
    has_trigger.set(false);
    screen.run_until_idle();
    // Without a trigger the state falls back to the requested state.
    open.set(false);
    screen.run_until_idle();
    screen.get_by_text(&open_text("trigger", false));
    has_trigger.set(true);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Details").click();
    screen.get_by_text(&open_text("trigger", true));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_parent_reads_the_popup_content_state_across_closing_and_reopening() {
    use kivact::interaction::use_popup;
    use kivact::view::{Placement, overlay};
    let (screen, open) = mount(|| {
        let open = signal(true);
        let popup = use_popup(open);
        let state = popup.content_state();
        let anchor = node_ref();
        (
            open,
            column((
                control("Menu").node_ref(anchor).part(popup.trigger()),
                show(popup.is_open(), move || {
                    overlay(
                        anchor,
                        Placement::Below,
                        control("Surface").part(popup.content()),
                    )
                }),
                text(move || open_text("surface", state.get().open)),
            )),
        )
    });
    screen.get_by_text(&open_text("surface", true));
    open.set(false);
    screen.run_until_idle();
    screen.get_by_text(&open_text("surface", false));
    open.set(true);
    screen.run_until_idle();
    screen.get_by_text(&open_text("surface", true));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```

#### 完了時の確認（D14 IN-3 の「完了時の確認」を次で置き換える）
- `cargo check -p kivact-interaction -p kivact-ui -p kivact-ui-basic -p kivact-ui-soft -p kivact-ui-pop -p kivact-ui-game -p kivact-ui-creative -p kivact-public-api-tests --all-targets`
- `cargo clippy --no-deps -p kivact-interaction --all-targets -- -D warnings`
- `cargo xtask check --no-write --only state-fields --crate kivact-interaction`
- `dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- interaction_ apg_ ui_disclosure_surfaces presence`

### 確かめたこと（試作・試験の結果）
- 再現（HEAD のコード、実 window）：上の 3 通りの読み方の probe 試験がすべて `UiError(Disposed)` の panic（disclosure.rs:107 ×2、:95 ×1）。最も普通の使い方（setup で `content_state()`、`show(is_open)` で content）でも、最初に閉じた時点で panic する。
- 修正後（試作）：`cargo clippy --no-deps -p kivact-interaction -p kivact-ui --all-targets -- -D warnings` 成功、`cargo xtask check --no-write --only state-fields --crate kivact-interaction` 成功（exit 0）。上の試験 3 本と `interaction_toggle::` の既存 2 本が成功。
- HEAD のソース（試作の crates/ の変更を stash）では、上の試験 3 本がすべて `UiError(Disposed)` で失敗した（disclosure の content・trigger、popup の content。popup は `popup/api.rs:232` の読取り）。修正を入れると成功する。試験は修正の有無を区別する。
- 試作で public-API の E2E 全体（629 本）を実行：`interaction_*`・`apg_*`・`ui_*`・`presence` を含め、`diagnostics::why_follows_the_prop_to_the_write_through_a_memo` 以外はすべて成功（この 1 本は HEAD でも失敗する既存の不具合。R03 の節を参照）。cleanup の順序（owner の終わりで slot を空にする）による既存の振る舞いの変化は見つからなかった。

---

## Q2：全体

- **wave・lane への影響**：どちらも W6 の中で閉じる。R03 は L5 の P8-UI5（所有に `data_table/builder.rs`・`ui_data_table.rs` を足す）。3.4 は L4 の IN-3（所有に kivact-interaction の 14 ファイル・新規 `part_slot.rs`・`interaction_toggle.rs` を足す）。lane の間でファイルは重ならない（L4 は kivact-interaction と `interaction_toggle.rs`、L5 は kivact-ui と `ui_data_table.rs`）。lane 内の重なり（IN-1 の `select/handle.rs`、IN-2 の `menu/handle.rs`・`lib.rs`、P8-IN の `dialog.rs`・`drag/handle.rs`）は lane の順で解決し、変える行は重ならない。
- **署名の変更と呼出し元**：R03 で変わる署名は private な `paging()`（返り値）と `table_paging()` の削除、新しい private `multi_sorting()` だけで、呼出し元は全て同じ `runtime.rs`。`DataTable` の field は `pub(super)` で、読み書きは `data_table/builder.rs`・`runtime.rs` だけ（`chrome.rs`・`rows.rs`・`filters.rs`・`metadata.rs` は `DataTable` の field を読まない。`filters.rs:39` の `self.page` は `Filters` の field）。3.4 は公開・crate 内の署名を変えない（新しい `pub(crate) fn` を足すだけ）。
- **他の担当との結合**：
  - D8 W1 の `struct PagingPlan { page, size_choice }` と、その `ModelPlan.paging` の marker 文は使わない（本書の enum で置き換える）。D8 W2 は本書のコードに含めた。
  - W1 由来の `kivact-engine` の dead_code 警告（`paint/state.rs` の `Indexed.size`・`content`）のため、HEAD では `cargo clippy -p <crate> --all-targets -- -D warnings` が依存の警告で失敗する。本書の確認は `--no-deps` を付けた。W1 F5 の閉じ方を担当する修正で消えるなら、`--no-deps` は外してよい。
  - 既存の失敗：`diagnostics::why_follows_the_prop_to_the_write_through_a_memo` は HEAD 6c812296 でも失敗する（期待 `node 5`、実際 `node 4`。W1 で node の番号の振り方が変わった可能性）。CP の全 E2E の前に担当を決めること（本件の範囲外）。
  - 3.4 の修正は G9（W7）の改名（`MenuApi::show` の新しい名前）と独立（slot の書き方だけで、method 名に触れない）。FINAL-1 の `PartState` の変更とも独立（slot の型は `Value<S>` の汎用）。
- **人間にしか決められない項目**：無し。
  - R03 で `page_sizes` を enum の外に残すのは、呼出し順の自由と remote での効きを保つため（設計者の判断。レビューの修正案とも一致）。
  - 3.4 の修正を 21 箇所すべてに当てるのは、同じ owner の寿命の欠陥で、helper 1 個の 1 行置換で閉じられるため（disclosure だけを直すと、popup・tooltip・menu・select・combobox・dialog の親の読取りで同じ panic が残る）。


---

# Q3：R04・3.3（W2 G3、W3 G4）・R07（W5 G8）

（元の題：Q3（target lane C）：R04・3.3・R07 の検証と修正設計）

対象 HEAD 6c812296。試作は worktree `W:/w/kv-rf-Q3`（G3 の本書版＋G4 の付録 C・D＋本書の新 E2E＋R07）と `W:/w/kv-rf-Q3s`（R07 だけ、サイズ計測用）。target `W:/kv-target/C`、`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`。

---

## Q3：R04：G3 で足す要求済み集合 `asked` が window の close で消えない

### 判定：妥当（D6 3.5／D11 G3 のとおりに実装すると実際に起こる）

- `slim-fast-v2/D6-gpu-api.md:557` の `asked: RefCell<BTreeSet<WindowId>>` は `State::demand`（`:565-574`）で insert されるだけで、削除する経路はどの付属書にも無い。D11 G3（`:467`、`:477-478`）も field を残し marker だけを外す。
- close の経路（`crates/kivact-core/src/view/window_view.rs:216-231` の `WindowSlot::close` → `ViewState::remove_window`（`view/state.rs:337`））は gpu_interop の `State` を触らない。`adapter_changed`（`gpu_interop/import/registry.rs:186`）も adapter の表と import の状態しか書かない。
- `adapter()` は CPU だけの window でも常に `None` を返すので、CPU 専用の構成でも insert される。`next_window`（`view/state.rs:342`）は id を再利用しないので、子 window の開閉 N 回で N 件残る（1 件は小さいが上限が無い。監査の対象外の状態でもある）。

ただし根本は「寿命の誤り」より「不要な状態」である。**「window の device が需要を待っているか」という事実は、すでに arvane の window ごとの presenter が持っている**：
- `crates/arvane/src/presenter/gpu.rs:251-266` の `State::{Off, Deferred, Background, Failed}`。`Deferred` が「需要を待つ」。
- W1 の G1 で `Gpu::demand`（`gpu.rs:386-393`）は `Deferred` の時だけ起動し、他の状態では何もしない（冪等）。一度 `Deferred` を出た window は二度と戻らない（device の再生成は `GpuStartCause::Restart` で `Background` の中で行い、需要を使わない）。
- presenter は `Windows::close`（`arvane/src/window.rs:247-255`）で消え、engine の window の model（`model.effects` の未処理の需要を含む）も `AppModel::close_window`（`kivact-engine/src/model/app_model.rs:128-150`）で消える。閉じた window への `effect` は writer の `app.windows.get_mut(&window)` が `None` で捨てる（`host/write/writer.rs:451-455`）。閉じた window からの host event は core の `dispatch`（`client/dispatch/delivery.rs:11-15`）が捨てる。

core の `asked` は、この事実の二重の写しで、目的は「同じ window に二度目の `DemandGpu` を送らない」最適化だけである。二度目を送っても host は何もしない。

### 修正

**方針：core は需要の記録を持たない（`asked` を作らない）。** `adapter()` が `None` を返す読取りのたびに `HostRequest::DemandGpu` を送る。再要求の抑制は window の寿命に属する唯一の正本、arvane の `Gpu::State` が行う。状態が増えないので state-fields の閉じ方・監査の行は不要（G3 の後の `gpu_interop::State` の field は HEAD と同じ `imports`・`adapter` の 2 つで、どちらも lint が数えない）。

**window の寿命の表（確定。単一の入口：core は `registry::demand`（`adapter()` だけが呼ぶ private な関数）、host は `Gpu::demand`）**

| 出来事 | arvane `Gpu::State`（window ごと、presenter と同じ寿命） | core |
|---|---|---|
| 生成 | `RendererStart` で決まる：`Cpu`→`Off`、`GpuOnDemand`→`Deferred`、`GpuAtLaunch`→`Background`（`Gpu::begin`） | 何も持たない |
| 要求（`adapter()` が `None`） | client の turn の後に `Windows::demand_gpu(id)` → `Gpu::demand(App)`。`Deferred` なら起動（note `GpuStarting { cause: App }`）、他の状態では何もしない | `DemandGpu` を window の engine model の `effects` に積むだけ（記録なし） |
| 失敗 | `Failed`（その session は CPU）。以後の要求は何もしない | adapter は `None` のまま。読取りは要求を送り、捨てられる |
| device の再生成 | `Background` の中で再起動（`GpuStartCause::Restart`）。adapter は `None` → 新しい generation の `Some` | `None` の間の読取りの要求は捨てられる。import は `Lost`（既存） |
| close | presenter と engine model が消え、未処理の要求も消える。close 後に書かれた要求は writer が捨てる | 消すものが無い |

**費用**：`None` を返す読取り 1 回につき、request id の加算 1 回、engine model の `Vec` への push 1 回、turn の後の presenter の lookup 1 回。`rt.request_turn()` は turn の中（TurnState が Running）では何もしない（`kivact-runtime/src/reactive/clock.rs:14-19`）ので、余分な turn は生じない。読取りは reactive の再評価の時にだけ起こる（adapter の変化、読み手の依存の変化）。viewport の毎 frame の draw は下の 3.3 の修正で需要を出さない。性能の目的に反しない。

**対象 unit**：W2 lane D の **G3**（D11「G3」を次のとおり上書き。D11 の他の指示（module 規則の except、`kivact-types/src/gpu.rs` の `OnDemand` の doc、文書の文言）はそのまま行う）。

G3 の手順の差し替え（D6 3.5 と D11 G3 の `registry.rs`・`viewport.rs` の指示をこれで置き換える）：

1. `crates/kivact-core/src/gpu_interop/import/registry.rs`
   - `asked` field と `State::demand` は**作らない**。`State`・`State::new` は HEAD のまま。
   - import を次にする：
     ```rust
     use crate::view::state::{ViewState, current_window};
     use kivact_engine::contract::HostRequest;
     use kivact_runtime::Signal;
     use kivact_runtime::{ErrorKind, UiError};
     use kivact_runtime::{Owner, Runtime};
     use kivact_types::{ExternalImageId, ExternalImport, GpuAdapter, RequestId, WindowId};
     use std::cell::RefCell;
     use std::collections::{BTreeMap, HashMap};
     use std::panic::Location;
     ```
   - `pub fn adapter()` の doc の「A new [`GpuAdapter::generation`] means a new device: ...」の段落と「Like `platform::window_size`, ...」の段落の間に次の段落を足し（D11 の段落より長い。D11 の 3 行の段落は使わない）、`adapter()` の本体を置き換え、その後ろに 3 つの関数を足す：
     ```rust
     /// Reading it asks the window to start its GPU device when the device
     /// waits until it is needed (`GpuStart::OnDemand`, the default): it is
     /// `None` until the device is ready. The ask changes nothing the App
     /// reads and is answered by nothing but the adapter's change, so any
     /// code may read it, a memo, a draw closure or a text's closure included;
     /// a window on the CPU only, or whose device started or failed, ignores it.
     ///
     ```
     ```rust
     #[track_caller]
     pub fn adapter() -> Option<GpuAdapter> {
         let owner = Owner::ambient("adapter", Location::caller());
         let window = current_window(owner);
         let adapter = with_adapter(owner, window, |adapter| adapter.cloned());
         if adapter.is_none() {
             demand(&owner.runtime(), window);
         }
         adapter
     }

     /// Whether the window the call belongs to has an adapter ([`adapter`]),
     /// tracked, without asking for its device: a viewport draws a GPU frame
     /// only on the device it was imported into, which the App's own read of
     /// the adapter started.
     #[track_caller]
     pub(crate) fn has_adapter() -> bool {
         let owner = Owner::ambient("Viewport::gpu_frame", Location::caller());
         with_adapter(owner, current_window(owner), |adapter| adapter.is_some())
     }

     /// `f` of `window`'s adapter, tracked.
     fn with_adapter<R>(
         owner: Owner,
         window: WindowId,
         f: impl FnOnce(Option<&GpuAdapter>) -> R,
     ) -> R {
         let rt = owner.runtime();
         let signal = State::of(&rt).adapter;
         drop(rt);
         signal.with(|adapters| f(adapters.get(&window)))
     }

     /// Ask `window`'s host to start its GPU device (`HostRequest::DemandGpu`).
     /// The window's presenter keeps whether it waits for one: only a waiting
     /// device starts, and the request of a closed window is dropped, so the
     /// App keeps no record of it.
     fn demand(rt: &Runtime, window: WindowId) {
         let id = ViewState::of(rt).requests.borrow_mut().request_id();
         ViewState::tree(rt)
             .engine()
             .effect(RequestId(id), HostRequest::DemandGpu { window });
         rt.request_turn();
     }
     ```
     （`|adapter| adapter.cloned()` を `Option::<&GpuAdapter>::cloned` にすると「FnOnce is not general enough」の compile error になる。closure のまま書く。）
2. `crates/kivact-core/src/gpu_interop/import.rs`：`pub(crate) use registry::{State, adapter_changed, imported};` → `pub(crate) use registry::{State, adapter_changed, has_adapter, imported};`
3. `crates/kivact-core/src/gpu_interop.rs`：D11 のとおり `mod demand;`・`pub(crate) use demand::demand_gpu;`（と空行 1 行）を消し、`pub(crate) use import::{State, adapter_changed, imported};` → `pub(crate) use import::{State, adapter_changed, has_adapter, imported};`。`git rm crates/kivact-core/src/gpu_interop/demand.rs`。
4. `crates/kivact-core/src/view/elements/viewport.rs`：D11 のとおり `gpu_frame` を `self.gpu_frame = frame.into_value(); self` だけにする（`#[track_caller]` も消す）。さらに `content` の memo と `draw` の closure の 2 箇所の `.filter(|_| crate::gpu_interop::adapter().is_some())` を `.filter(|_| crate::gpu_interop::has_adapter())` にする（3.3 の修正）。
5. `import/image.rs`、`share/app_side.rs`、`view/drawable/requests.rs`・`handle.rs`、`dev/xtask/policy/workspace.toml`、`kivact-types/src/gpu.rs` の doc、README・docs の文言：D11 G3 のとおり。
6. `docs/for-kivact-developer/architecture.md`：「## turn、Stage と時間」の最後の段落（「reactive の frame kind（Handler/Setup/Computation）と ... 勝手に加えない。」）の直後に、次の 1 段落を足す（3.3 の契約）：
   > host の資源を使うことの宣言（`gpu_interop::adapter()` が `None` の読取りで送る `HostRequest::DemandGpu`）は、答えを返さず、App が読む状態を書かず、何度送っても一度と同じで、閉じた window の分は host が捨てる。render effect が engine の木を書くのと同じく、memo・render effect・draw の中で送ってよい。答えや UI の状態の変化を伴う platform request、signal の書込み、focus の移動は従来どおり `require_effectful` の対象とする。window ごとの「需要を待つか」は presenter の `Gpu::State` だけが持ち、core は記録しない。

G3 の所有ファイル：D11 の列に `docs/for-kivact-developer/architecture.md`（上の 1 段落だけ）を足す。W2 で他にこのファイルを持つ unit は無い（W8 の 12.2 の書戻しは後の wave）。

G3 の完了時の確認：D11 のとおり（`cargo check -p kivact-core --all-targets --all-features`、同 feature なし、`cargo check -p kivact-public-api-tests --tests --features gpu-interop`、state-fields と module 規則の xtask）。E2E は W2 では足さない（e2e の `main.rs` は W2 lane B の F7 の所有）。

**E2E（W3 lane F の G4 に足す）**：G4 の所有に `dev/tests/kivact-public-api/tests/e2e/gpu_on_demand_windows.rs`（新規）を足す。`main.rs` には D11 G4 の 3 行の直後に次の 2 行を足す（rustfmt の並びのとおり `gpu_on_demand_shared` の次）：

```rust
#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
mod gpu_on_demand_windows;
```

`gpu_on_demand_windows.rs` の全文（試験 2 本：`gpu_on_demand_windows::each_window_opened_again_starts_its_own_device_on_demand`、`gpu_on_demand_windows::cpu_windows_start_no_device_however_often_they_ask`。付録 C の `gpu_on_demand.rs` の `pub(crate)` の helper を使う）：

```rust
//! Role: The GPU device of windows opened and closed while the App runs.
//!
//! Reading `gpu_interop::adapter` asks the window the read belongs to for
//! its GPU device. Whether a device waits to start is the window's own (its
//! presenter's), so the ask lives and ends with the window: every window
//! opened again starts a device of its own when it reads its adapter, the
//! window that did not read it starts none, and asking again starts no
//! second device. A window on the CPU only starts none however often it is
//! opened and asks. The adapter is read in a text's closure, a pure
//! computation: asking for the device is allowed there.

use std::cell::RefCell;
use std::rc::Rc;

use kivact::gpu_interop;
use kivact::prelude::*;
use kivact::test::{RenderOptions, Renderer, Screen};

use super::gpu_on_demand::{GRACE, cpu_forced, cpu_only, gpu_lane, on_demand};
use super::support::mount_with;

/// How many times a test opens and closes the window.
const ROUNDS: u32 = 3;
/// How many times a round makes the window read its adapter again.
const READS: u32 = 3;

/// The generations of the adapters the child window's text read, `None`
/// for a read without one, in order.
type Seen = Rc<RefCell<Vec<Option<u64>>>>;

/// A main window that opens the window "Child" while `open`. The child's
/// text reads the child's adapter ("gpu" or "cpu"), again whenever `reads`
/// changes, and notes each read in `seen`.
#[track_caller]
fn mount(options: RenderOptions) -> (Screen, Signal<bool>, Signal<u32>, Seen) {
    let seen: Seen = Rc::default();
    let noted = seen.clone();
    let (screen, (open, reads)) = mount_with(options, move || {
        let open = signal(false);
        let reads = signal(0_u32);
        let view = column((
            text("main"),
            window("Child", open).content(move || {
                let noted = noted.clone();
                text(move || {
                    reads.get();
                    let adapter = gpu_interop::adapter();
                    noted
                        .borrow_mut()
                        .push(adapter.as_ref().map(|adapter| adapter.generation));
                    if adapter.is_some() { "gpu" } else { "cpu" }.to_owned()
                })
            }),
        ));
        ((open, reads), view)
    });
    (screen, open, reads, seen)
}

/// Make the child window read its adapter `READS` times more.
fn read_again(screen: &Screen, reads: Signal<u32>) {
    for _ in 0..READS {
        reads.set(reads.get_untracked() + 1);
        screen.run_until_idle();
    }
}

/// The generation of the adapter the child window read last.
#[track_caller]
fn last_generation(seen: &Seen) -> Option<u64> {
    *seen.borrow().last().expect("the child window read its adapter")
}

/// Windows opened and closed `ROUNDS` times, started with `options`, never
/// start a device, however often they read their adapter.
#[track_caller]
fn windows_stay_on_cpu(options: RenderOptions) {
    let (screen, open, reads, seen) = mount(options);
    for round in 0..ROUNDS {
        open.set(true);
        screen.run_until_idle();
        let child = screen.window("Child");
        read_again(&screen, reads);
        std::thread::sleep(GRACE);
        screen.run_until_idle();
        assert_eq!(
            child.renderer(),
            Renderer::Cpu,
            "round {round}: a window on the CPU only starts no device"
        );
        child.get_by_text("cpu");
        open.set(false);
        screen.run_until_idle();
        assert!(!child.is_open(), "round {round}: the window closes");
    }
    assert_eq!(screen.renderer(), Renderer::Cpu, "nor does the main window");
    assert!(
        seen.borrow().iter().all(Option::is_none),
        "no read has an adapter: {:?}",
        seen.borrow()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Each window opened again starts a device of its own when it reads its
/// adapter; reading it again, while the device starts and after, makes no
/// other device; the main window, which never reads its adapter, starts
/// none.
#[test]
fn each_window_opened_again_starts_its_own_device_on_demand() {
    if !gpu_lane("each_window_opened_again_starts_its_own_device_on_demand") {
        return;
    }
    if cpu_forced() {
        return windows_stay_on_cpu(on_demand());
    }
    let (screen, open, reads, seen) = mount(on_demand());
    let mut generations = Vec::new();
    for round in 0..ROUNDS {
        open.set(true);
        screen.run_until_idle();
        let child = screen.window("Child");
        assert!(
            matches!(child.renderer(), Renderer::GpuStarting | Renderer::Gpu),
            "round {round}: reading the adapter starts the new window's device, not {:?}",
            child.renderer()
        );
        read_again(&screen, reads);
        child.wait_for_gpu();
        child.get_by_text("gpu");
        let generation = last_generation(&seen).expect("the ready device has an adapter");
        read_again(&screen, reads);
        assert_eq!(child.renderer(), Renderer::Gpu, "round {round}");
        assert_eq!(
            last_generation(&seen),
            Some(generation),
            "round {round}: reading the adapter again makes no other device"
        );
        generations.push(generation);
        open.set(false);
        screen.run_until_idle();
        assert!(!child.is_open(), "round {round}: the window closes");
    }
    assert!(
        generations.windows(2).all(|pair| pair[0] < pair[1]),
        "each window opened made a device of its own: {generations:?}"
    );
    assert_eq!(
        screen.renderer(),
        Renderer::Cpu,
        "the main window never read its adapter: it starts no device"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Windows on the CPU only (as with `ARVANE_RENDERER=cpu`) start no device,
/// however often they are opened and read their adapter.
#[test]
fn cpu_windows_start_no_device_however_often_they_ask() {
    if !gpu_lane("cpu_windows_start_no_device_however_often_they_ask") {
        return;
    }
    windows_stay_on_cpu(cpu_only());
}
```

G4 の完了時の確認は D11 のとおり（filter `gpu_on_demand` が 3 module の 6 本に当たる。CP1・CP2 のコマンドも変えない）。

何を試験で示し、何を構造で示すか：「閉じた window の記録が残らない」は、core に記録の field が存在しないこと（型）で保証する（公開 API からは観測できない）。E2E は振る舞いを固定する：開き直した window が毎回自分の device を起動する（OnDemand）、需要は読んだ window だけに効く（main は Cpu のまま）、同じ live window の再要求は第二の device を作らない（generation が変わらない）、CPU 専用の window は何度開閉・再要求しても起動しない、pure な text の closure の中の読取りが error を出さない。

### 確かめたこと（試作・試験の結果）

- `W:/w/kv-rf-Q3`：本書の G3（＋G4 の付録 C・D、＋本書の E2E）で `cargo clippy -p kivact-core --all-targets --all-features -- -D warnings`、同 feature なし、`cargo clippy -p kivact-public-api-tests --tests --features gpu-interop -- -D warnings` が成功（kivact-core 単体の clippy は HEAD の engine の F4 の dead_code で止まるので、試作中だけ F4-fix 相当を当てて確認し、戻した）。
- `cargo xtask check --no-write --crate kivact-core`：state-fields は W1 からの既知の `view/mount/semantics.rs:14 Plan` の 1 件だけ（gpu_interop に新しい行は無い）。module 規則の error 無し（`view::drawable::requests -> gpu_interop` の except を消した後）。
- 実 window（`KIVACT_TEST_MIRROR=required`、`window-lock.sh`、GPU のある Windows 機）：`cargo test -p kivact-public-api-tests --features gpu-interop --test e2e -- gpu_on_demand --test-threads=1` で 6 本成功（付録 C の 2 本、付録 D の 2 本、本書の 2 本）、5.45 s。`ARVANE_RENDERER=cpu` の lane と、既存の `gpu_interop`・`drawable_export_gpu`・`drawables` の E2E の結果は末尾の「追記」。

---

## Q3：3.3：`adapter()` を memo・draw の中で読んで需要を出すことの是非、public な `GpuFrame::new`

### 判定：妥当（D11 G3 の 4 の安全性の説明は事実に反する）

- D11 `:470` は「GpuFrame は import か shared image からしか作れず、どちらも先に `adapter()` を読んでいるので memo の中から effect が出ることは無い」とする。しかし `crates/kivact-style/src/draw/image.rs:60` の `GpuFrame::new` は public で、`GpuFrame` は `Copy`。engine の crate は `kivact-gpu-share` の `pool.frame(image)`（`pool.rs:79`）の `ExternalFrame` から作れ、App は frame を別の window の viewport に渡せる。その window が adapter を読んでいなければ、D11 の viewport の memo・draw の `adapter()` は pure な文脈の中で、意図しない window の device を起動する需要を出す。
- runtime の purity の契約：`kivact_runtime::pure` の doc（`kivact-runtime/src/reactive/frame.rs:242-247`）は「a platform request」を pure の中の副作用に数え、platform service は `require_effectful` で守る（`kivact-core/src/platform/services.rs:45,101`）。一方で render effect（pure）は engine の木を書く（`view/mount/draw.rs` の `set_scene`、`window_view.rs` の `set_window`）。需要がどちらに当たるかは文書に無かった。

### 修正（R04 の G3 の手順に組み込み済み）

契約を次のとおり確定する（architecture.md の段落と `adapter()` の doc）：
- **需要は「host の資源を使うことの宣言」**：答えを返さない（pump は `continue`、`kivact-engine/src/host/pump.rs:197-200`）、App が読む状態を書かない、冪等（`Gpu::demand` は `Deferred` だけ）、閉じた window の分は捨てられる、turn の中では turn を要求しない。render effect が engine の木を書くのと同じ種類なので、**memo・render effect・draw・text の closure の中で出してよい**。`require_effectful` は答えや UI の状態変化を伴う request（beep、window size の要求、focus）に限る。public の `adapter()` を「どこで読んでもよい tracked getter」のまま保つ（`platform::window_size` と同じ。React/Solid の利用者は getter をどこで読むかを気にしない）。
- **内部の読取りは需要を出さない**：viewport は `has_adapter()`（需要なし、tracked）で「この window の device に描けるか」だけを見る。frame の出所（import、shared image、`GpuFrame::new`、別 window）は需要に関係しなくなり、D11 の「先に読んでいる」という前提が不要になる。別 window の frame・device の無い window の frame は、既存の契約どおり CPU の `pixels` が描かれる（`gpu_frame_with_pixels` に行かない）。
- `Viewport::gpu_frame` は builder の時に `Owner::current` を呼ばない（D6 7 章 D5 の修正。D11 どおり）。

検証の範囲：public constructor 経由・別 window の frame を E2E で直接作ることはしない。公開 API の試験 crate は `kivact-types` に依存せず `ExternalFrame` を作れない（facade は `ExternalFrame` を再公開しない）うえ、viewport の経路に需要の分岐が存在しないこと（`has_adapter` は `demand` を呼ばない）がコードで閉じているため。CPU 専用の構成と、pure な文脈の中の読取りは R04 の 2 本の E2E が通る。

### 確かめたこと

- 本書の G3 で viewport の 2 箇所を `has_adapter()` にして clippy・xtask 成功（上）。付録 D の `a_shared_image_starts_the_gpu_on_demand`（shared image の frame を canvas の `d.gpu_frame` で描く）と既存の GPU 系 E2E が成功（「追記」）。
- `request_turn` が turn の中で何もしないこと：`kivact-runtime/src/reactive/clock.rs:14-19`（Quiet の時だけ wake）。

---

## Q3：R07：G8 で gpu-interop の条件を外す adapter 指定が、通常の `gpu` 構成では黙って無視される

### 判定：妥当

`crates/arvane-compose/src/gpu/context.rs:155` の `#[cfg(feature = "gpu-interop")]` が adapter の検索を囲む（`GpuPreference.adapter` の検索は `kivact_gpu_share::device::identity` が要り、`kivact-gpu-share` は arvane-compose の feature `gpu-interop` の時だけの依存）。G8（DEC-17）の後は `App::gpu_preference` が feature なしで公開されるので、既定の `gpu` の build で `adapter: Some(..)` を書いても検索されず、doc（D12 G8 の「A wish: the device is made on another adapter ... when the one asked for is not there」）と食い違う。

### 修正：adapter の検索を `gpu` の build で有効にする（設定できるなら効く）

試作で事実として可能と確かめた（下）。`kivact-gpu-share` の identity の読取りは wgpu の hal（Direct3D 12 の `GetDesc2`、Vulkan の `vkGetPhysicalDeviceProperties2`）だけを使い、その依存（`wgpu`、`pollster`、Windows/Linux の `ash`・`windows`）は `gpu` の build が既に持つ（`kivact-gpu-share/Cargo.toml` の `ash`・`windows` は target 依存の常時依存、`import` feature は `wgpu`・`pollster`）。診断の追加は不要（効くので）。

**対象 unit**：W5 lane C の **G8**（D12「G8」の手順の最後に 9 として足す）。

9. adapter の指定を `gpu` の build で効かせる（R07）：
   - `crates/arvane-compose/Cargo.toml`：
     - `# Shared textures with external render engines (the import side of the device).` の 1 行を、次の 2 行にする：
       ```toml
       # The identity of adapters (`GpuPreference::adapter`), and shared textures with
       # external render engines (the import side of the device).
       ```
     - `[features]` の 2 行を次にする：
       ```toml
       gpu = ["arvane-render/gpu", "dep:wgpu", "dep:pollster", "dep:raw-window-handle", "dep:kivact-gpu-share"]
       gpu-interop = ["gpu"]
       ```
   - `crates/arvane-compose/src/gpu/context.rs`：
     - `fn adapter` の doc を次にする：
       ```rust
       /// The adapter of `backends` the device is made on: the one the
       /// application named, when it is there, else the one its power preference
       /// picks.
       ```
     - `fn adapter` の本体の先頭の `#[cfg(feature = "gpu-interop")]`（`if let Some(wanted) = env.preference.adapter {` の直前の行）を消す。
     - `fn gpu_adapter` の直前の `#[cfg(feature = "gpu-interop")]` を消す（`next_generation` と `open_device` の cfg は残す）。
   - `crates/kivact-types/src/gpu.rs` の `GpuPreference.adapter` の doc の 2 行（`/// The adapter (`None`: as `power` picks). Only builds with GPU` と `/// texture sharing look adapters up by identity; others ignore it.`）を次にする（marker の行は変えない）：
     ```rust
     /// The adapter (`None`: as `power` picks), looked up by its identity on
     /// Windows and Linux (Direct3D 12 and Vulkan report it); as `power`
     /// picks when no adapter has it.
     ```
   - 完了時の確認（D12 の 8 に加える）：`cargo clippy -p arvane-compose --all-targets --features gpu -- -D warnings`、同 `--features gpu-interop`、同 feature なし、`cargo clippy -p arvane-compose --features gpu --target x86_64-unknown-linux-gnu -- -D warnings`（できなければ 0.6 節の WSL の check）、`cargo xtask check --no-write --crate arvane-compose`。

G8 の所有ファイルに `crates/arvane-compose/{Cargo.toml, src/gpu/context.rs}` と `crates/kivact-types/src/gpu.rs`（`GpuPreference.adapter` の doc の 3 行だけ）を足す。W5 の他の lane（A：macros・component_macro・custom-components、B：calendar・menu・api_shape）とは重ならない。`kivact-types/src/gpu.rs` は W2 の G3（`OnDemand` の doc）と W6 の P8-GPU も持つが、wave が違う。

**W6 の P8-GPU への申し送り**（D13「P8-GPU」に足す 1 文）：G8 の後、`arvane-compose/src/gpu/context.rs` の `fn gpu_adapter` とその 2 つの呼出し（`fn adapter` の検索と `open_device`）は feature `gpu` の build でも compile される。付録の `gpu_adapter` の書換え（引数を `kivact_types::AdapterIds`）では、`fn adapter` の検索の `gpu_adapter(adapter, identity, 0)` も `identity(adapter)` の戻り値をそのまま渡すので変更は要らない。確認のコマンド `cargo check -p arvane -p arvane-compose --all-targets --features gpu` がこの経路を compile する。

### 確かめたこと（`W:/w/kv-rf-Q3s`、HEAD＋上の 2 ファイルの変更）

- `cargo clippy -p arvane-compose --all-targets -- -D warnings` を `--features gpu`、`--features gpu-interop`、feature なしで成功。`--features gpu --target x86_64-unknown-linux-gnu` の `cargo check` と `cargo clippy -- -D warnings` も成功。
- サイズ：`cargo build --release -p kivact-bench --bin kv_startup`（既定 = `gpu` 込み、K6 の counter default の行と同じ build）で、変更前 19,069,440 B → 変更後 19,081,216 B（**+11,776 B**）。K6 の `gpu` 込みの行の見積り（約 18.17 MB）と必達（≤ B1-start、元の機械 18,978,816 B）の余裕（約 0.8 MB）の中。GPU 抜きの K6 の行（`--no-default-features`）は `gpu` が無いので `kivact-gpu-share` を含まない（変化なし）。
- `cargo xtask check --no-write --crate arvane-compose`：「追記」。

---

## Q3：全体

### 他の担当との結合・wave・lane への影響

- G3（W2 lane D）：所有に `docs/for-kivact-developer/architecture.md`（1 段落）を追加。手順は D11 G3 の `registry.rs`・`viewport.rs` の指示を本書で置き換える。W2 の他の lane（A：engine、B：F5-fix・F4-fix・F7（e2e の `main.rs`）、C：types render・arvane-text 等、E：runtime・`view/flow/list.rs`、F：M9 の `view/mount/*`・`handler_layers.rs`）と重ならない。署名の変更（`send_requests` の `window` 引数の削除）の呼出し元 `handle.rs` は同じ unit（D11 どおり）。新しい `has_adapter` の呼出し元 `viewport.rs` も同じ unit。
- G4（W3 lane F）：所有に `gpu_on_demand_windows.rs` を追加、`main.rs` に 2 行。W3 で e2e の `main.rs` を持つのは同じ lane の S3 だけ。
- G8（W5 lane C）：所有に `arvane-compose/{Cargo.toml, src/gpu/context.rs}`、`kivact-types/src/gpu.rs`（doc 3 行）を追加。
- P8-GPU（W6 lane L3）：状態の追加が無いので marker・監査の行の追加なし。上の申し送り 1 文だけ。
- W6 の CO（D14 X4）：「G3 が足した `SharedImage.current` への書込み」は本書でも変わらない（G3 の `app_side.rs` の変更は D11 どおり）。

### 他の担当への申し送り（本書の試作で見つけた事実。Q3 の範囲外）

- **W2 lane B の F4-fix（作業指示書 4 章、DEC-25）は debug＋`testing` の build を壊す**：`crates/kivact-engine/src/paint/list/self_check.rs:126-127` が `after.size`・`after.content` を読む（`paint/list.rs:91` の `#[cfg(all(debug_assertions, feature = "testing"))] mod self_check;`）。試作で `Indexed` の 2 field を消すと `cargo xtask check` の build で `E0609 no field size/content on type &paint::state::Indexed` になった（feature なしの clippy の dead_code は、この cfg の時だけ読まれることによる）。DEC-25 の「読み手が無い」は事実に反する。F4-fix の担当の監査で直し方を確定すること（例：self_check の比較から 2 field を外すか、self_check が読む値を別に持つ）。
- core の adapter の表（`gpu_interop::State.adapter`、`Signal<BTreeMap<WindowId, GpuAdapter>>`）も閉じた window の要素を消さない（host は閉じた window に `GpuAdapterChanged(None)` を送らず、送っても core が捨てる）。レビューも R04 とは別の既存の問題としている。GPU の device を持った window を閉じるたびに 1 要素（`GpuAdapter` 1 つ）が残る。仕事は付かず、読まれもしない（id は再利用されない）。直すには adapter を window の record（`WindowRecord`）の `Signal<Option<GpuAdapter>>` に移す（`projection:` の marker が要る）必要があり、V2 の性能・状態の unit の範囲を広げるので、V2 では直さず「V2 の範囲外として記録だけするもの」に 1 行で足す（判断は設計者として確定。人間の決定は不要）。

### 人間にしか決められない項目

なし。

---

## Q3：追記：試験の結果（本書の各「確かめたこと」の続き）

- `ARVANE_RENDERER=cpu KIVACT_TEST_MIRROR=required window-lock.sh -- cargo test -p kivact-public-api-tests --features gpu-interop --test e2e -- gpu_on_demand --test-threads=1`：6 本成功（7.03 s）。本書の `each_window_opened_again_starts_its_own_device_on_demand` は `cpu_forced` の分岐（`windows_stay_on_cpu(on_demand())`）で成功。
- 既存の回帰：同じ構成で `-- gpu_interop drawable_export_gpu drawables`：32 本成功（20.33 s）。G3 の需要の経路の変更と R07（adapter の検索の cfg の削除。`gpu_interop.rs:222`・`:273` の `AdapterMatch` を使う試験を含む）で退行なし。
- 陰性対照：`adapter()` の `if adapter.is_none()` を一時的に `if adapter.is_none() && false` にする（需要を出さない）と、`each_window_opened_again_starts_its_own_device_on_demand` が `gpu_on_demand_windows.rs:126`（「reading the adapter starts the new window's device」）で失敗し、`cpu_windows_start_no_device_however_often_they_ask` は成功した。新しい試験は需要が失われたことを検出する（試作は元に戻した）。
- `cargo xtask check --no-write --crate arvane-compose`（R07 の後）：error は W1 からの既知の `arvane-compose/src/cache.rs:53 Indexed`（W2 の F5-fix が閉じる）の 1 件だけ。features・依存の規則に R07 による error は無い。


---

# Q4：R05（W7 FINAL-2）

（元の題：Q4（target lane D）：R05 の検証と修正設計）

対象：レビュー `20261110_1828_CODEX_REVIEW.md` の R05、W7 の FINAL-2（付属書 D5 3.4 X3〜X5、修正付属書 D14「FINAL-2」・付録 A `claims.rs`・付録 B の試験）。HEAD 6c812296。
試作：`W:/w/kv-rf-Q4`（detached worktree。終了時に削除済み）。cargo は `CARGO_TARGET_DIR=W:/kv-target/D`（後半 `D2`）。

## Q4：R05：FINAL-2 の projection の書き手検査が、別の型の同名 field を誤検出し、alias 経由の書込みを見逃す

### 判定：妥当（実際に起こる）。レビューの 2 例に加え、同じ原因の穴を 5 つ再現した

D14 付録 A の `claims.rs` 全文と付録 B の試験をそのまま worktree に置き（`state_fields.rs` に `mod claims;`、`local_checks.rs` に `verify` の呼出し）、次の fixture を `tests.rs` に足して `cargo test -p xtask state_fields` を実行した。D14 の 7 本（X4・X5）は全て成功し、次の 6 本は全て失敗した（＝検査器の欠陥が実在する）。

| 試験（再現用、成果物には残さない） | fixture の要点 | D14 の結果 |
|---|---|---|
| `r05_same_name_field_of_another_type` | 同じ module に `struct T { a: Cell<bool> }` と `fn touch(t: &T) { t.a.set(true); }` | 誤検出：`S.a is also written in touch`（T の field なのに） |
| `r05_alias_write_is_seen` | `fn other(s: &S) { let cell = &s.a; cell.set(false); }` | 見逃し（error 0） |
| `r05_machine_name_only` | 別の型の `impl Other { fn transition(&self) -> State { State::Open { previewed: Some(9), .. } } }` | 見逃し（`transition` という名前だけで通る） |
| `r05_custom_mutator_is_seen` | `self.a.toggle()`（WRITE_METHODS に無い method） | 見逃し |
| `r05_nested_field_write_is_seen` | `self.a.x = 2`（field の一部への代入。`target_field` は最後の `x` しか見ない） | 見逃し |
| `r05_passed_to_function_is_seen` | `flip(&self.a)`、`fn flip(c: &Cell<bool>) { c.set(false) }` | 見逃し |

加えて、コードを読んで確かめた穴（試験は修正版の試験に含めた）：
- **pattern で束縛した書込み**：`if let State::Open { previewed, .. } = &mut next { *previewed = key; }`（HEAD の `select/machine.rs:112-113`）は `ExprField` を含まないので、D14 の検査器には書込みが 1 つも見えない（D14 IN-1 の節自身が「X4 の書く式に当たらない」と書いている）。つまり D14 で `State::Open.previewed` が通るのは「Machine の例外」ではなく「書込みを 1 つも見ていない」から。
- **macro の中**：`syn::visit` は macro の token を辿らないので、`assert!(self.a.replace(true))` 等の書込みは見えない。

レビューの指摘「`transition` という関数名だけで enum の variant への書込みを通す」も事実（D14 付録 A `verify_claim` の `machine` 判定は関数名だけを見る）。
なお、現在の HEAD・W6 後の projection（下の照合表）で D14 の検査器が誤判定する行は無い（D14 の確認どおり）。問題は「検証できないものを成功扱いにする」性質と、同名 field を足すだけで gate が壊れる性質で、SLIM-16 の約束（唯一の書き手を xtask が検査する）に足りない。レビューの判定どおり直す。

### 修正の方針（確定。理由）

レビューの代案「Rust の可視性で範囲を絞り、範囲内で検証できないものは error にして構造を直させる」を採る。検査器に型推論を入れない（receiver の型は `self` の時だけ分かる、で止める）。代わりに、検査器が「書込み」「読込み」を確定できない形は全て error（未検証）にし、その直し方を構造側（private 化・書込み method を owner に置く・field の改名）に寄せる。

1. **範囲は Rust の可視性**（field の、enum variant は enum の可視性）。**crate 全体に見える projection field は error**（`pub`、`pub(crate)`、`pub(in crate)`、crate root の module の private field、top-level module の `pub(super)`）。理由：範囲が crate 全体だと、同名 field を足すだけで gate が壊れる（レビューの指摘）。private だけに限ると、W6 後の 17 本のうち 11 本（`Host`・`HostWindow`・`Pointer`・`AsyncCell`・`State` 等）が他の module からの構築・読込みのために大きな組直しを要する。`pub(super)` まで許すと範囲は 1 つの directory に収まり、組直しは 4 本（小さい変更）で済む。
2. **範囲の中では、名前で受け取る（型を推論しない）**。`x.f` は全て owner の field とみなす。例外は 1 つだけ：`self.f` が owner でない型の `impl` の中にあるとき（その型の field と確定する）。範囲の中の同名 field の衝突は「projection の field を module の中で一意な名前にする」で直す（error 文に書く）。
3. **書込み**（名指した関数の外なら error `is also written in`）：代入・複合代入（field の一部を含む：`x.f.g = ..`、`x.f[i] = ..`、`*x.f = ..`）、書込み method（`set`・`update`・`replace`・`take`・`swap`・`insert`・`push`・`clear`・`get_or_init`・`set_if_changed` と、可変の取っ手を渡す `borrow_mut`・`get_mut`・`as_mut`・`lock`・`write`）、owner の literal の非空の値、literal の `..rest`（空の初期値でないもの）。
4. **未検証**（名指した関数の外なら error `may be changed in`）：`&x.f`・`&mut x.f`（alias、別関数への受渡し）、既知の読込み method でも書込み method でもない method（独自 mutator）、owner の struct pattern で field を束縛すること（`_` は除く）、標準の値 macro（`assert!`・`format!` 等 18 個）以外の macro、または式として読めない macro の中の `.f`。
5. **読込み**（検査しない）：値としての使用、読込み method（`get`・`with`・`borrow`・`is_some`・`as_ref`・`clone`・`iter`・`len` 等 26 個）。`unwrap`・`expect` は通過（その外側の文脈で決まる）。
6. **名指した関数の中は何でも可**。ただし**名指した関数の戻り値の型に参照を含めない**（含むと呼出し元が書けるので error）。
7. **Machine の例外は削除**。Machine の state の field は、`Machine::transition`（`impl … for Machine` の self 型 `Machine` の `transition`）を書き手として名指す。一般の規則 2〜6 だけで、`State::Open { previewed: None, .. }`（空）・pattern 束縛の書込み（`transition` の中）が通り、範囲（`select` の subtree）の他の関数の literal・pattern 束縛は error になる。関数名だけの特例は無くなる。
8. 値全体の置換（`*w = HostWindow::new(..)`、`x.clone()`）は field の書込みとして数えない。根拠：新しい owner の値は literal（範囲の中でしか書けない＝private 以下の field を持つので literal は範囲内に限られ、検査される）、`derive(Default)`（空の初期値）、`Clone`（書き手が書いた値の複製）のどれかでしか作れない。

この形で、レビューの「検証できないものを成功扱いにしない」は満たす（範囲内の未分類の使用は全て error）。検査器は D14 の 659 行から 840 行（fmt 済み、関数は全て 80 行以内）。

### W6 の後の projection（D14 の照合表）がこの規則で通るか（試作で確認）

W6 の付属書どおりの marker・書き手の集約（CO の `replace_current`、EN-2 の `report_app_motion`・`record_environment`、EN-1・P8-CO・P8-KT・EX の marker、RT の `begin_run`、IN-1 の marker）を worktree に再現して新しい検査器を実行した結果：

| file | owner.field | 可視範囲 | 新しい規則の結果 | 直し（FINAL-2 の手順 0） |
|---|---|---|---|---|
| `kivact-core/src/gpu_interop/share/app_side.rs` | SharedImage.current | private（`gpu_interop::share::app_side`） | 通る | なし |
| `kivact-core/src/view/semantics_data.rs` | SemanticsPatch.reactive | `pub(crate)` | **error**（crate 全体） | F1：`pub(super)`（範囲 `view`。書き手 `view/modifiers.rs`、読み手 `view/mount/semantics.rs` はどちらも範囲内） |
| `kivact-engine/src/host/pump.rs` | Host.app_motion | `pub(super)`（`host`） | 通る | なし |
| `kivact-engine/src/host/window.rs` | HostWindow.environment | `pub(super)` | 通る | なし |
| 同 | HostWindow.soft_keyboard、ime_allowed | `pub(super)` | 通る | なし |
| 同 | HostWindow.adapter | `pub(super)` | **error**：`pump.rs` `export_pools` の `&want.adapter`（別の型の同名 field の借用）を区別できない | F5：field を `reported_adapter` に改名 |
| `kivact-engine/src/model/aggregate.rs` | Model.modifiers | `pub(crate)` | **error**（crate 全体） | F2：`pub(super)`（範囲 `model`。書き手 `model/keys.rs`、他の使用なし） |
| `kivact-engine/src/model/pointer/state.rs` | Pointer.hover_sent、position_sent | `pub(super)`（`model::pointer`） | 通る | なし |
| `kivact-engine/src/measure/editor.rs` | EditorNode.reported_session | `pub(crate)` | **error**（crate 全体。書き手は `model::edit::inputs` で `measure` の外なので `pub(super)` にできない） | F3：private にし、書込み method `EditorNode::report_session` を owner に置く |
| `kivact-interaction/src/select/machine.rs` | State::Open.previewed | enum `pub(super)`（`select`） | marker を `Machine::transition` にすれば通る（`track_preview` 名指しは Machine 例外の削除で error） | F6：marker の差替え |
| `kivact-runtime/src/task/action.rs` | ActionCell.last | private | 通る | なし |
| `kivact-runtime/src/task/async_state.rs` | AsyncCell.latest | `pub(super)`（`task`） | 通る（`action.rs`・`resource.rs` の `.latest.get()` は読込み） | なし |
| `kivact-test/src/facts.rs` | Node.kind、Node.parent | `pub(super)` の top-level module＝crate 全体 | **error** | F4：private にし accessor を置く。`kind` は `facts.rs` の `match &node.kind`（engine の `NodeProperties.kind` の借用）と区別できないので `node_type` に改名 |
| `kivact-ui-basic/src/sidebar/menu.rs` | ButtonCfg.has_action | private | 通る（`let ButtonCfg { size, active, .. } = cfg;` は `has_action` を束縛しない） | なし |

- 計 17 本（D14 の表の「19 行（M6・M8 を含め最大 21）」は数え違い。field 数は 17、M6・M8 を含めて 19）。**M6 の `InStylePaint.drawn` と M8 の `NodeRefState.observed` は D10 で取り消された**（D10 3 章は `InStyle`／`InStylePaint` を使わず、M8 は `observation: Cell<RefObservation>` で projection marker を置かない）ので、照合表から消す。
- F1〜F6 を当てた後、17 本とも error 0。残る X4 の error は W6 が付属書で型・option・independent に置き換える snapshot の marker（90 件。HEAD にあるもの）だけで、W6 の後は消える。

### 修正（確定。作業指示書 10 章 FINAL-2 の行の「本書の追加の指示」へ。D14 FINAL-2 の節のうち下の項目を置き換える）

#### 対象 unit・wave・lane・所有ファイル
- unit：**FINAL-2（W7。FINAL-1 → G9 → FINAL-2 の順のまま）**。W6 の unit の手順は変えない（W7 は直列なので、他の lane とファイルは重ならない）。
- 所有ファイルの追加（手順 0 の分。全て W6 の後の状態に当てる）：
  - `crates/kivact-core/src/view/semantics_data.rs`（F1）
  - `crates/kivact-engine/src/model/aggregate.rs`（F2）
  - `crates/kivact-engine/src/measure/editor.rs`、`crates/kivact-engine/src/model/edit/inputs.rs`（F3。呼出し元は inputs.rs の 1 箇所だけ）
  - `crates/kivact-test/src/facts.rs`、`crates/kivact-test/src/screen/query.rs`、`crates/kivact-test/src/screen/layout.rs`（F4。他に compile error が出た kivact-test の file があればそれも）
  - `crates/kivact-engine/src/host/window.rs`、`crates/kivact-engine/src/host/pump.rs`（F5。他に compile error が出た `host` の file があればそれも）
  - `crates/kivact-interaction/src/select/machine.rs`（F6。marker 行 1 本）
- D14 FINAL-2 の所有ファイル（`check.rs`、`locals.rs`、`claims.rs`、`tests.rs`、`state_fields.rs`、`local_checks.rs`）は変えない。

#### 手順 0（新設。X4 の前提の構造。X3〜X5 の実装の前に行い、この 0 だけで 1 commit にしてよい）

各手順の後で `cargo check -p <その crate> --all-targets`。compile error は、下に書いた置換を全ての error 箇所に同じ形で当てて直す（それ以外の変更をしない）。

- **F1** `crates/kivact-core/src/view/semantics_data.rs`：`SemanticsPatch` の `pub(crate) reactive: bool,` を `pub(super) reactive: bool,` にする（marker は P8-CO の文のまま）。
- **F2** `crates/kivact-engine/src/model/aggregate.rs`：`Model` の `pub(crate) modifiers: kivact_types::cell::Sent<Modifiers>,` を `pub(super) modifiers: kivact_types::cell::Sent<Modifiers>,` にする。
- **F3** `crates/kivact-engine/src/measure/editor.rs`：
  - `EditorNode` の `pub(crate) reported_session: kivact_types::cell::Sent<kivact_editing::TextSession>,` を `reported_session: kivact_types::cell::Sent<kivact_editing::TextSession>,`（private）にする。
  - その直前の marker 行を次の全文にする：
    `    // xtask-allow(state-fields): projection: the session last sent to the selection listener, written only by EditorNode::report_session`
  - `impl EditorNode {` の直後（`pub(crate) fn new` の前）に足す：
    ```rust
        /// Note `session` as the one last sent to the selection listener;
        /// whether it differs from the one before (the only writer of
        /// `reported_session`).
        pub(crate) fn report_session(&mut self, session: &kivact_editing::TextSession) -> bool {
            self.reported_session.update(session.clone()).is_some()
        }

    ```
  - `crates/kivact-engine/src/model/edit/inputs.rs` `Model::report_sessions`：`.is_some_and(|node| node.reported_session.update(session.clone()).is_some());` を `.is_some_and(|node| node.report_session(&session));` にする。振る舞いは不変。
- **F4** `crates/kivact-test/src/facts.rs`：
  - `Node` の `pub(super) kind: Option<NodeType>,` を `node_type: Option<NodeType>,`、`pub(super) parent: Option<NodeId>,` を `parent: Option<NodeId>,` にする（marker 行は P8-KT の文のまま。`children`・`at` は変えない）。
  - `pub(super) struct Facts {` の直前に足す：
    ```rust
    impl Node {
        /// The engine reader's node type; `None` when the reader has no
        /// properties for the id.
        pub(super) fn node_type(&self) -> Option<NodeType> {
            self.node_type
        }

        /// The engine reader's parent; `None` for the root.
        pub(super) fn parent(&self) -> Option<NodeId> {
            self.parent
        }
    }
    ```
  - `Facts::new`：`let kind = reader.with_node(id, |n| n.map(|n| n.kind.node_type()));` を `let node_type = reader.with_node(id, |n| n.map(|n| n.kind.node_type()));` に、literal の `kind,` を `node_type,` にする。
  - 読み手（compile error `field ... of struct Node is private` の全箇所）：`n.kind` を `n.node_type()`、`n.parent`（facts の `Node` の）を `n.parent()` にする。HEAD では `screen/query.rs` の `if n.kind == Some(NodeType::Text)` → `if n.node_type() == Some(NodeType::Text)`、`cur = n.parent;` → `cur = n.parent();`、`screen/layout.rs` の `cur = n.and_then(|n| n.parent);` → `cur = n.and_then(|n| n.parent());` の 3 箇所。
- **F5** `crates/kivact-engine/src/host/window.rs`：`HostWindow` の `pub(super) adapter: Sent<kivact_types::GpuAdapter>,` を `pub(super) reported_adapter: Sent<kivact_types::GpuAdapter>,` に、`HostWindow::new`（literal）の `adapter: Sent::new(),` を `reported_adapter: Sent::new(),` にする（marker 行は変えない）。compile error `no field adapter on type ...HostWindow` の全箇所で `.adapter` を `.reported_adapter` にする（HEAD では `host/pump.rs` `Host::adapter_changed` の 3 箇所：`host_window.adapter.update(..)`、`host_window.adapter.get()`、`host_window.adapter = Sent::new();`）。`want.adapter` 等、別の型の `.adapter` は変えない。
- **F6** `crates/kivact-interaction/src/select/machine.rs`：`State::Open` の `previewed` の直前の marker 行（IN-1 が置いた行）を次の全文にする：
  `        // xtask-allow(state-fields): projection: the option last reported to the preview callback, written only by Machine::transition on Event::Preview, which only track_preview sends (the hovered key, else the list's active key); Open starts it at None and Close reports None`
  （IN-1 の手順 5 の `track_preview` の doc 1 行は残す。）
- 確認：`cargo fmt --all`（F4 の `screen/query.rs` の行は長くなり rustfmt が折り返す。試作の全体 check で fmt の差分として出た）、続けて `cargo clippy -p kivact-core -p kivact-engine -p kivact-test -p kivact-interaction --all-targets -- -D warnings`（試作で warning 0）。振る舞いは全て不変（private 化・改名・同じ式の method への移動だけ）。E2E の追加は無い（既存の selection listener・GPU adapter・kivact-test の query の試験が担保）。

#### 手順 X4（D14 FINAL-2 の「2. X4・X5」を置き換える）

- `dev/xtask/src/state_fields/claims.rs` は **本書 付録 A の全文**（D14 付録 A は使わない）。D14 の「付属書の骨組みとの違い」の 1・2・4・5・6 はそのまま成り立つ。3 は「variant の field の可視性は enum の可視性」のまま。7 は次に置き換える：owner は `enum Owner { Struct(String), Variant { enumeration, variant } }`。可視範囲は `Visible` 型をやめ、`scope()` が返す module の path（`None`＝crate 全体＝error）。Machine の例外は無い。
- `dev/xtask/src/state_fields.rs`：`mod check;` の次に `mod claims;`、`pub(crate) use check::check;` の次に `pub(crate) use claims::verify;`。module doc の最後の行の次に 1 行（D14 の文を置き換える）：
  `//! A projection field is visible to one module subtree below the crate root, and only the one writer its marker names changes it there; an independent reason is not shared with another file (SL-9).`
- `dev/xtask/src/local_checks.rs`：D14 のとおり（`Scope::run` の source の `for` の直後に `crate::state_fields::verify(..)`）。
- 試験：`dev/xtask/src/state_fields/tests.rs` の末尾に、D14 付録 B の最初の 2 本（X3 の `one_unmarked_field_beside_a_marked_one_is_an_error`・`one_unmarked_local_beside_a_marked_one_is_an_error`）と、**本書 付録 B の全文**を足す（D14 付録 B の 3 本目以降は使わない）。
- 付属書 D5 3.4 X4 の規則 1〜4 は、上の「修正の方針」1〜7 に置き換わる（error の文言は付録 A のとおり。D5 3.4 の 3 つに次の 3 つが加わる）：
  - `projection field <Owner>.<f> is visible to the whole crate; make it private or pub(super) below the crate root so its writers can be checked`
  - `projection writer <名前> of <Owner>.<f> returns a reference; a borrow of the field would let its callers change it`
  - `<Owner>.<f> may be changed in <fn> at <path>:<line> (<理由>); the checker cannot prove it is a read, so change `<f>` only in <名前> and read it by value or with a read method (give the field a name unique in its module if this is another type's field)`

#### 手順 4（確認）の差替え（D14 FINAL-2 の「4. 確認」の表と error の扱い）

照合表（`git grep -n "xtask-allow(state-fields): projection:" -- '*.rs' ':!dev/xtask'` の結果と同じ行であること）：

| file | owner.field | 名指す書き手 |
|---|---|---|
| `crates/kivact-core/src/gpu_interop/share/app_side.rs` | SharedImage.current | SharedImage::replace_current（CO） |
| `crates/kivact-core/src/view/semantics_data.rs` | SemanticsPatch.reactive（`pub(super)`） | Common::semantic（P8-CO） |
| `crates/kivact-engine/src/host/pump.rs` | Host.app_motion | Host::report_app_motion（EN-2） |
| `crates/kivact-engine/src/host/window.rs` | HostWindow.environment | Host::record_environment（EN-2） |
| 同 | HostWindow.soft_keyboard、ime_allowed | Host::synchronize_ime |
| 同 | HostWindow.reported_adapter | Host::adapter_changed |
| `crates/kivact-engine/src/model/aggregate.rs` | Model.modifiers（`pub(super)`） | Model::report_modifiers |
| `crates/kivact-engine/src/model/pointer/state.rs` | Pointer.hover_sent、position_sent | Model::report_hover、Model::report_mouse_position |
| `crates/kivact-engine/src/measure/editor.rs` | EditorNode.reported_session（private） | EditorNode::report_session |
| `crates/kivact-interaction/src/select/machine.rs` | State::Open.previewed | Machine::transition |
| `crates/kivact-runtime/src/task/action.rs` | ActionCell.last | ActionCell::begin_run（RT） |
| `crates/kivact-runtime/src/task/async_state.rs` | AsyncCell.latest | AsyncCell::publish |
| `crates/kivact-test/src/facts.rs` | Node.node_type、Node.parent（private） | Facts::new（P8-KT） |
| `crates/kivact-ui-basic/src/sidebar/menu.rs` | ButtonCfg.has_action | build_button（EX） |

（M6・M8 の 2 行は削除：D10 で取り消し。）

error の扱い（D14 の表に足す。STOP にしない）：

| error の文言 | 扱い |
|---|---|
| `visible to the whole crate`（上の表の行） | 手順 0 の該当の F が漏れた。行う |
| `visible to the whole crate`（表に無い行） | D14 の「表に無い行」の手順（付属書の文に置き換える） |
| `<Owner>.<f> may be changed in <fn> at <path>:<line> (...)`（表の行） | その行を開く。(a) receiver が `self` でなく、owner と別の型の値（局所変数・引数・別の struct の field）なら、projection の field を `reported_<f>` に改名し F5 と同じ手順（宣言・literal・compile error の全箇所）で直す。(b) owner の field への alias・独自 method・pattern 束縛なら、その書込みを名指した関数へ移せないか見ず、`dev/bench/results/OPEN.md` の「V2 差戻し」に 1 行書いて残りを続ける |
| `returns a reference` | 差戻し（OPEN.md「V2 差戻し」に 1 行） |
| `is also written in`（表の行） | D14 のとおり |

### 確かめたこと（試作・試験の結果）

- D14 版の再現：上の 6 本が D14 の `claims.rs` で失敗（誤検出 1、見逃し 5）。D14 の X4・X5 の試験 7 本は成功。
- 修正版（付録 A・B）：`cargo test -p xtask` 81 本全て成功（既存 65＋本書 16。D14 付録 B の X3 の 2 本は X3 を試作していないので含まない）。`cargo clippy -p xtask --all-targets -- -D warnings` warning 0。`cargo fmt -p xtask` 済み。
- HEAD への実行（`cargo xtask check --no-write --only state-fields`）：X5 の 44 件は D14 と同じ。X4 は HEAD の snapshot marker の error（W6 で消える）と、表の行の error を出した。
- W6 の再現（上の付属書の marker と集約を worktree に当てた）＋手順 0（F1〜F6）：`cargo clippy -p kivact-core -p kivact-engine -p kivact-test -p kivact-runtime -p kivact-interaction -p kivact-ui-basic --all-targets -- -D warnings` が通り、表の 17 本の X4 の error は 0。
- 全体の `cargo xtask check --no-write`（W6 再現＋手順 0＋新しい検査器の worktree）：`dev/xtask` の file に style・行数・`Role:` の error は 0（18 の style 規則）。exit 1 の原因は、HEAD にもある state-fields の error（W1 の返戻し 3 件・snapshot marker・X5）と、試作で fmt を掛けなかった 2 file（再現した EN-2 の `window_changes.rs`、F4 の `screen/query.rs`）の rustfmt 差分だけ。

### 範囲外で見つけた事実（親への申し送り。他の担当の unit）

1. **EN-2（D5 3.5・D13 284 行）の `report_app_motion` は compile しない**。`host/window_changes.rs` `finish_motion` は `let mut app = self.app.borrow_mut();` を持ったまま `if self.report_app_motion(drawable, period) {` を呼ぶので E0502（`*self` の可変借用と `self.app` の不変借用）。試作で再現。直し（EN-2 の手順に足す 1 行）：`let mut app = self.app.borrow_mut();` を `let shared = std::rc::Rc::clone(&self.app);` と `let mut app = shared.borrow_mut();` の 2 行にする（振る舞い不変。試作で compile・clippy 通過）。
2. **CO（D5 3.5）の `renew` の書換えは clippy `collapsible_if` で `-D warnings` に落ちる**。付属書の `if let Some(old) = self.replace_current(None) { if Runtime::current().is_some() { old.owner.dispose(); } }` を、`if let Some(old) = self.replace_current(None)` ／ `    && Runtime::current().is_some()` ／ `{ old.owner.dispose(); }`（let chain）にする（`replace_current` は先に評価されるので振る舞い不変。試作で clippy 通過）。
3. D14 の照合表の M6・M8 の 2 行は D10 で無くなっている（上記）。

## Q4：全体

- **他の担当との結合**：本書が変えるのは W7 の FINAL-2 だけ（手順 0 の追加、claims.rs と試験の差替え、確認の表）。W6 の各 unit（IN-1・EN-1・EN-2・P8-CO・P8-KT・CO・RT・EX）の手順は変えない。FINAL-2 は W6 の後に F1〜F6 を当てるので、W6 の付属書がどう書いても（marker の文・field の可視性）FINAL-2 が最後に揃える。D14 の X3（`check.rs`・`locals.rs`）・X5 は変えない（X5 の `duplicate_reasons` は D14 と同一）。
- **wave・lane への影響**：なし。W7 は FINAL-1 → G9 → FINAL-2 の直列なので、手順 0 が触る kivact-core・engine・test・interaction の file は他の lane と重ならない。署名の変更（`EditorNode::report_session` の追加、facts の `Node` の accessor、`HostWindow.reported_adapter` の改名）の呼出し元は全て FINAL-2 の中で直す（compile error を全て直すまでが手順）。
- **G9 との関係**：G9 が `select/handle.rs` を変えても、F6 は `select/machine.rs` の marker 1 行だけで、`track_preview` の名前に依存しない（marker の prose に名前が残るだけ）。
- **範囲外の申し送り**：EN-2 の E0502 と CO の clippy（上記「範囲外で見つけた事実」1・2）は、それぞれ EN-2・CO の担当の監査で手順に足す必要がある（FINAL-2 の前提）。
- **人間にしか決められない項目**：なし。projection の field を crate 全体に見せない（`pub(super)` まで）規則、Machine の例外の削除、未検証を error にする規則は、設計者として確定した（理由は「修正の方針」）。

## Q4：付録 A：`dev/xtask/src/state_fields/claims.rs`（確定版の全文。`cargo fmt` 済み、clippy 済み、840 行）

```rust
//! Role: Verify what state-field markers claim within the module a field is visible in.

use crate::check_source::Source;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;

/// The rule name reported by these checks.
const RULE: &str = "state-fields";
/// The marker that explains a field.
const MARKER: &str = "// xtask-allow(state-fields):";
/// The phrase that names a projection's one writer (SL-9).
const WRITER: &str = "written only by ";
/// Method calls that write the value they are called on or hand out a
/// mutable handle to it.
const WRITE_METHODS: [&str; 15] = [
    "set",
    "update",
    "replace",
    "take",
    "swap",
    "insert",
    "push",
    "clear",
    "get_or_init",
    "set_if_changed",
    "borrow_mut",
    "get_mut",
    "as_mut",
    "lock",
    "write",
];
/// Method calls that only read the value they are called on.
const READ_METHODS: [&str; 26] = [
    "get",
    "get_untracked",
    "try_get",
    "try_get_untracked",
    "with",
    "with_untracked",
    "borrow",
    "is_some",
    "is_none",
    "is_some_and",
    "is_none_or",
    "as_ref",
    "as_deref",
    "clone",
    "cloned",
    "copied",
    "iter",
    "len",
    "is_empty",
    "contains",
    "contains_key",
    "map",
    "and_then",
    "unwrap_or",
    "unwrap_or_default",
    "to_owned",
];
/// Method calls that pass the value on; their own receiver decides.
const PASS_METHODS: [&str; 2] = ["unwrap", "expect"];
/// Standard macros whose arguments are plain expressions.
const VALUE_MACROS: [&str; 18] = [
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "format",
    "format_args",
    "print",
    "println",
    "eprint",
    "eprintln",
    "write",
    "writeln",
    "panic",
    "unreachable",
    "todo",
    "dbg",
];

/// One `projection:` marker and the writer it names.
struct Claim<'a> {
    source: &'a Source,
    line: usize,
    /// The struct, or `Enum::Variant`, whose field the marker explains.
    owner: Owner,
    field: String,
    /// The module whose subtree can name the field (never the crate root).
    scope: String,
    writer: Writer,
}

/// The item that declares a claimed field.
#[derive(Clone)]
enum Owner {
    Struct(String),
    Variant {
        enumeration: String,
        variant: String,
    },
}

/// The function a projection names: `fn` or `Type::fn`.
#[derive(Clone)]
struct Writer {
    ty: Option<String>,
    name: String,
}

/// What one use of the claimed field does.
enum Effect {
    /// A write: an assignment, a write method, a literal or `..rest`.
    Write,
    /// A use the checker cannot prove to be a read (an alias).
    Unverified(String),
}

/// One use of the claimed field other than a plain read.
struct Use {
    path: PathBuf,
    line: usize,
    /// The innermost function and its impl's self type, if any.
    function: Option<(String, Option<String>)>,
    effect: Effect,
}

/// Check every `projection:` claim and every `independent:` reason (X4, X5).
///
/// `all` is every loaded source; errors are reported only in the sources
/// `report` selects.
pub(crate) fn verify(all: &[Source], report: &dyn Fn(&Source) -> bool, errors: &mut Vec<String>) {
    duplicate_reasons(all, report, errors);
    for source in all.iter().filter(|source| report(source)) {
        for claim in claims(source, errors) {
            verify_claim(all, &claim, errors);
        }
    }
}

/// X5: an `independent:` reason shared with another file says nothing about
/// the item it closes.
fn duplicate_reasons(all: &[Source], report: &dyn Fn(&Source) -> bool, errors: &mut Vec<String>) {
    let mut paths: BTreeMap<String, BTreeSet<PathBuf>> = BTreeMap::new();
    let mut lines = Vec::new();
    for source in all {
        for (index, text) in source.text.lines().enumerate() {
            let Some(reason) = text.trim_start().strip_prefix(MARKER) else {
                continue;
            };
            let reason = reason.split_whitespace().collect::<Vec<_>>().join(" ");
            if !reason.starts_with("independent:") || !source.owns_line(index + 1) {
                continue;
            }
            paths
                .entry(reason.clone())
                .or_default()
                .insert(source.path.clone());
            if report(source) {
                lines.push((source, index + 1, reason));
            }
        }
    }
    for (source, line, reason) in lines {
        let Some(other) = paths[&reason].iter().find(|path| **path != source.path) else {
            continue;
        };
        source.error_at_line(
            RULE,
            line,
            &format!(
                "independent: reason repeats in {}; say why every combination of this item's fields occurs",
                other.display()
            ),
            errors,
        );
    }
}

/// The `projection:` markers on the struct and named-variant fields of `source`.
fn claims<'a>(source: &'a Source, errors: &mut Vec<String>) -> Vec<Claim<'a>> {
    let mut claims = Vec::new();
    for item in &source.items {
        // A variant's fields are as visible as their enum.
        let owners: Vec<(Owner, &syn::Fields, Option<&syn::Visibility>)> = match item {
            syn::Item::Struct(item) => {
                vec![(Owner::Struct(item.ident.to_string()), &item.fields, None)]
            }
            syn::Item::Enum(item) => item
                .variants
                .iter()
                .map(|variant| {
                    let owner = Owner::Variant {
                        enumeration: item.ident.to_string(),
                        variant: variant.ident.to_string(),
                    };
                    (owner, &variant.fields, Some(&item.vis))
                })
                .collect(),
            _ => continue,
        };
        for (owner, fields, enum_visibility) in owners {
            for field in fields {
                let visibility = enum_visibility.unwrap_or(&field.vis);
                if let Some(claim) = claim(source, &owner, field, visibility, errors) {
                    claims.push(claim);
                }
            }
        }
    }
    claims
}

/// The claim of one field's marker, when it is a well-formed `projection:`
/// on a field that is not visible to the whole crate.
fn claim<'a>(
    source: &'a Source,
    owner: &Owner,
    field: &syn::Field,
    visibility: &syn::Visibility,
    errors: &mut Vec<String>,
) -> Option<Claim<'a>> {
    let name = field.ident.as_ref()?.to_string();
    let line = source.allow_line(RULE, field.span())?;
    let reason = source.allow_reason(RULE, line)?;
    if !reason.starts_with("projection:") {
        return None;
    }
    let Some(writer) = named_writer(&reason) else {
        source.error_at_line(
            RULE,
            line,
            "projection marker must name one writer: \"written only by <fn>\"",
            errors,
        );
        return None;
    };
    let Some(scope) = scope(source, visibility) else {
        source.error_at_line(
            RULE,
            line,
            &format!(
                "projection field {}.{name} is visible to the whole crate; make it private or pub(super) below the crate root so its writers can be checked",
                owner.label()
            ),
            errors,
        );
        return None;
    };
    Some(Claim {
        source,
        line,
        owner: owner.clone(),
        field: name,
        scope,
        writer,
    })
}

impl Owner {
    fn label(&self) -> String {
        match self {
            Self::Struct(name) => name.clone(),
            Self::Variant {
                enumeration,
                variant,
            } => format!("{enumeration}::{variant}"),
        }
    }
}

/// The one `written only by <fn>` or `written only by <Type::fn>` in `reason`.
fn named_writer(reason: &str) -> Option<Writer> {
    let mut parts = reason.split(WRITER);
    parts.next()?;
    let rest = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let path: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == ':')
        .collect();
    let path = path.trim_end_matches(':');
    let segments: Vec<&str> = path.split("::").collect();
    let valid = |segment: &&str| {
        segment
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
    };
    match segments.as_slice() {
        [name] if valid(name) => Some(Writer {
            ty: None,
            name: (*name).to_owned(),
        }),
        [ty, name] if valid(ty) && valid(name) => Some(Writer {
            ty: Some((*ty).to_owned()),
            name: (*name).to_owned(),
        }),
        _ => None,
    }
}

/// The module whose subtree can name a field with `visibility`, declared in
/// `source`; `None` when that is the whole crate.
fn scope(source: &Source, visibility: &syn::Visibility) -> Option<String> {
    let parent = |module: &str| {
        module
            .rsplit_once("::")
            .map_or(String::new(), |(parent, _)| parent.to_owned())
    };
    let module = match visibility {
        syn::Visibility::Inherited => source.module.clone(),
        syn::Visibility::Public(_) => String::new(),
        syn::Visibility::Restricted(restricted) => {
            let path: Vec<String> = restricted
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            match path.first().map(String::as_str) {
                Some("crate") => path[1..].join("::"),
                Some("self") => source.module.clone(),
                Some("super") => path
                    .iter()
                    .filter(|segment| *segment == "super")
                    .fold(source.module.clone(), |base, _| parent(&base)),
                _ => String::new(),
            }
        }
    };
    (!module.is_empty()).then_some(module)
}

/// Whether `source` is in the subtree of `scope` within the target of `declared`.
fn in_scope(scope: &str, declared: &Source, source: &Source) -> bool {
    source.identity_root() == declared.identity_root()
        && (source.module == scope || source.module.starts_with(&format!("{scope}::")))
}

/// Rules 2 and 3 of SL-9 for one claim.
fn verify_claim(all: &[Source], claim: &Claim<'_>, errors: &mut Vec<String>) {
    let visible: Vec<&Source> = all
        .iter()
        .filter(|source| in_scope(&claim.scope, claim.source, source))
        .collect();
    let label = format!("{}.{}", claim.owner.label(), claim.field);
    let writer = match &claim.writer.ty {
        Some(ty) => format!("{ty}::{}", claim.writer.name),
        None => claim.writer.name.clone(),
    };
    let found: Vec<Found> = visible
        .iter()
        .filter_map(|source| definition(source, &claim.writer))
        .collect();
    let detail = if found.is_empty() {
        Some(format!(
            "projection writer {writer} of {label} does not exist where the field is visible"
        ))
    } else if found.iter().any(|found| found.returns_reference) {
        Some(format!(
            "projection writer {writer} of {label} returns a reference; a borrow of the field would let its callers change it"
        ))
    } else {
        None
    };
    if let Some(detail) = detail {
        claim
            .source
            .error_at_line(RULE, claim.line, &detail, errors);
        return;
    }
    let uses = visible
        .iter()
        .flat_map(|source| uses(source, &claim.owner, &claim.field));
    let mut reported = BTreeSet::new();
    for found in uses {
        if is_writer(&found, &claim.writer) {
            continue;
        }
        let function = found
            .function
            .as_ref()
            .map_or("no function", |(name, _)| name.as_str());
        let at = format!("{}:{}", found.path.display(), found.line);
        let detail = match &found.effect {
            Effect::Write => format!(
                "{label} is also written in {function} at {at}; a projection has one writer ({writer})"
            ),
            Effect::Unverified(why) => format!(
                "{label} may be changed in {function} at {at} ({why}); the checker cannot prove it is a read, so change `{}` only in {writer} and read it by value or with a read method (give the field a name unique in its module if this is another type's field)",
                claim.field
            ),
        };
        if reported.insert(detail.clone()) {
            claim
                .source
                .error_at_line(RULE, claim.line, &detail, errors);
        }
    }
}

fn is_writer(found: &Use, writer: &Writer) -> bool {
    found.function.as_ref().is_some_and(|(name, ty)| {
        *name == writer.name
            && writer
                .ty
                .as_ref()
                .is_none_or(|expected| ty.as_ref() == Some(expected))
    })
}

/// The definition of the function `writer` names in `source`, if any.
fn definition(source: &Source, writer: &Writer) -> Option<Found> {
    let mut finder = Functions {
        writer,
        impls: Vec::new(),
        found: None,
    };
    for item in &source.items {
        finder.visit_item(item);
    }
    finder.found
}

/// A found writer definition.
struct Found {
    returns_reference: bool,
}

/// Finds the definition of one named function.
struct Functions<'a> {
    writer: &'a Writer,
    impls: Vec<Option<String>>,
    found: Option<Found>,
}

impl Functions<'_> {
    fn note(&mut self, signature: &syn::Signature) {
        let returns_reference = match &signature.output {
            syn::ReturnType::Default => false,
            syn::ReturnType::Type(_, ty) => has_reference(ty),
        };
        self.found = Some(Found { returns_reference });
    }
}

impl<'ast> Visit<'ast> for Functions<'_> {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if node.content.is_none() {
            syn::visit::visit_item_mod(self, node);
        }
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        self.impls.push(type_name(&node.self_ty));
        syn::visit::visit_item_impl(self, node);
        self.impls.pop();
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if self.writer.ty.is_none() && node.sig.ident == self.writer.name {
            self.note(&node.sig);
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let ty = self.impls.last().cloned().flatten();
        if node.sig.ident == self.writer.name
            && self
                .writer
                .ty
                .as_ref()
                .is_none_or(|expected| ty.as_ref() == Some(expected))
        {
            self.note(&node.sig);
        }
        syn::visit::visit_impl_item_fn(self, node);
    }
}

/// Whether `ty` contains a reference type.
fn has_reference(ty: &syn::Type) -> bool {
    struct References(bool);
    impl<'ast> Visit<'ast> for References {
        fn visit_type_reference(&mut self, _: &'ast syn::TypeReference) {
            self.0 = true;
        }
    }
    let mut references = References(false);
    references.visit_type(ty);
    references.0
}

/// The uses of `field` of `owner` in `source` other than plain reads.
fn uses(source: &Source, owner: &Owner, field: &str) -> Vec<Use> {
    let mut visitor = Uses {
        source,
        owner,
        field,
        functions: Vec::new(),
        impls: Vec::new(),
        found: Vec::new(),
    };
    for item in &source.items {
        visitor.visit_item(item);
    }
    visitor.found
}

/// Collects the uses of one field other than plain reads.
struct Uses<'a> {
    source: &'a Source,
    owner: &'a Owner,
    field: &'a str,
    functions: Vec<(String, Option<String>)>,
    impls: Vec<Option<String>>,
    found: Vec<Use>,
}

impl Uses<'_> {
    fn record(&mut self, line: usize, effect: Effect) {
        if self.source.owns_line(line) {
            self.found.push(Use {
                path: self.source.path.clone(),
                line,
                function: self.functions.last().cloned(),
                effect,
            });
        }
    }

    /// Whether a struct literal's or pattern's path names the owner.
    fn names_owner(&self, path: &syn::Path) -> bool {
        let segments: Vec<String> = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        let this = self.impls.last().cloned().flatten();
        match self.owner {
            Owner::Struct(name) => match segments.as_slice() {
                [only] if only == "Self" => this.as_ref() == Some(name),
                [.., last] => last == name,
                [] => false,
            },
            Owner::Variant {
                enumeration,
                variant,
            } => match segments.as_slice() {
                [first, last] if first == "Self" => {
                    last == variant && this.as_ref() == Some(enumeration)
                }
                [.., last] => last == variant,
                [] => false,
            },
        }
    }

    /// Whether the place `expr` (a receiver, an assignee or a borrow) is the
    /// claimed field or a part of it. A variant's field has no `.field`
    /// access; `self.field` in another type's impl is that type's field.
    fn reaches(&self, expr: &syn::Expr) -> bool {
        let Owner::Struct(owner) = self.owner else {
            return false;
        };
        match expr {
            syn::Expr::Paren(inner) => self.reaches(&inner.expr),
            syn::Expr::Group(inner) => self.reaches(&inner.expr),
            syn::Expr::Unary(inner) if matches!(inner.op, syn::UnOp::Deref(_)) => {
                self.reaches(&inner.expr)
            }
            syn::Expr::Index(index) => self.reaches(&index.expr),
            syn::Expr::MethodCall(call)
                if PASS_METHODS.contains(&call.method.to_string().as_str()) =>
            {
                self.reaches(&call.receiver)
            }
            syn::Expr::Field(access) => {
                let named =
                    matches!(&access.member, syn::Member::Named(name) if name == self.field);
                let other_self = is_self(&access.base)
                    && self
                        .impls
                        .last()
                        .cloned()
                        .flatten()
                        .is_some_and(|this| this != *owner);
                (named && !other_self) || self.reaches(&access.base)
            }
            _ => false,
        }
    }

    /// Whether a macro's tokens name `.field` (a use the checker cannot read).
    fn mentions(&self, tokens: proc_macro2::TokenStream) -> bool {
        let mut dot = false;
        for token in tokens {
            match token {
                proc_macro2::TokenTree::Group(group) => {
                    if self.mentions(group.stream()) {
                        return true;
                    }
                    dot = false;
                }
                proc_macro2::TokenTree::Punct(punct) => dot = punct.as_char() == '.',
                proc_macro2::TokenTree::Ident(ident) => {
                    if dot && ident == self.field {
                        return true;
                    }
                    dot = false;
                }
                proc_macro2::TokenTree::Literal(_) => dot = false,
            }
        }
        false
    }
}

impl<'ast> Visit<'ast> for Uses<'_> {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if node.content.is_none() {
            syn::visit::visit_item_mod(self, node);
        }
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        self.impls.push(type_name(&node.self_ty));
        syn::visit::visit_item_impl(self, node);
        self.impls.pop();
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.functions.push((node.sig.ident.to_string(), None));
        syn::visit::visit_item_fn(self, node);
        self.functions.pop();
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let ty = self.impls.last().cloned().flatten();
        self.functions.push((node.sig.ident.to_string(), ty));
        syn::visit::visit_impl_item_fn(self, node);
        self.functions.pop();
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.functions.push((node.sig.ident.to_string(), None));
        syn::visit::visit_trait_item_fn(self, node);
        self.functions.pop();
    }

    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        if self.names_owner(&node.path) {
            let listed = node.fields.iter().find(
                |value| matches!(&value.member, syn::Member::Named(name) if name == self.field),
            );
            match (listed, &node.rest) {
                (Some(value), _) if !is_empty_initial(&value.expr) => {
                    self.record(value.span().start().line, Effect::Write);
                }
                (None, Some(rest)) if !is_empty_initial(rest) => {
                    self.record(rest.span().start().line, Effect::Write);
                }
                _ => {}
            }
        }
        syn::visit::visit_expr_struct(self, node);
    }

    fn visit_pat_struct(&mut self, node: &'ast syn::PatStruct) {
        if self.names_owner(&node.path) {
            for pattern in &node.fields {
                if matches!(&pattern.member, syn::Member::Named(name) if name == self.field)
                    && !matches!(*pattern.pat, syn::Pat::Wild(_))
                {
                    let why = "bound by a pattern".to_owned();
                    self.record(pattern.span().start().line, Effect::Unverified(why));
                }
            }
        }
        syn::visit::visit_pat_struct(self, node);
    }

    fn visit_expr_assign(&mut self, node: &'ast syn::ExprAssign) {
        if self.reaches(&node.left) {
            self.record(node.span().start().line, Effect::Write);
        }
        syn::visit::visit_expr_assign(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        if is_compound_assignment(&node.op) && self.reaches(&node.left) {
            self.record(node.span().start().line, Effect::Write);
        }
        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_expr_reference(&mut self, node: &'ast syn::ExprReference) {
        if self.reaches(&node.expr) {
            let why = if node.mutability.is_some() {
                "borrowed with &mut"
            } else {
                "borrowed with &"
            };
            self.record(node.span().start().line, Effect::Unverified(why.to_owned()));
        }
        syn::visit::visit_expr_reference(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method = node.method.to_string();
        let read = READ_METHODS.contains(&method.as_str());
        let pass = PASS_METHODS.contains(&method.as_str());
        if !read && !pass && self.reaches(&node.receiver) {
            let effect = if WRITE_METHODS.contains(&method.as_str()) {
                Effect::Write
            } else {
                Effect::Unverified(format!("calls `{method}`, not a known read"))
            };
            self.record(node.span().start().line, effect);
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        // A standard macro's arguments are expressions; any other macro may
        // write a place it is given.
        let known = node
            .path
            .segments
            .last()
            .is_some_and(|segment| VALUE_MACROS.contains(&segment.ident.to_string().as_str()));
        let parser = Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
        match node.parse_body_with(parser) {
            Ok(arguments) if known => {
                for argument in &arguments {
                    self.visit_expr(argument);
                }
            }
            _ => {
                if matches!(self.owner, Owner::Struct(_)) && self.mentions(node.tokens.clone()) {
                    let why = "inside a macro the checker cannot read".to_owned();
                    self.record(node.span().start().line, Effect::Unverified(why));
                }
            }
        }
    }
}

fn is_self(expr: &syn::Expr) -> bool {
    matches!(expr, syn::Expr::Path(path) if path.path.is_ident("self"))
}

fn is_compound_assignment(op: &syn::BinOp) -> bool {
    matches!(
        op,
        syn::BinOp::AddAssign(_)
            | syn::BinOp::SubAssign(_)
            | syn::BinOp::MulAssign(_)
            | syn::BinOp::DivAssign(_)
            | syn::BinOp::RemAssign(_)
            | syn::BinOp::BitXorAssign(_)
            | syn::BinOp::BitAndAssign(_)
            | syn::BinOp::BitOrAssign(_)
            | syn::BinOp::ShlAssign(_)
            | syn::BinOp::ShrAssign(_)
    )
}

/// A constructor's starting value: a literal, `None`, a default, an empty
/// call, or a `new`/`signal`/`var`/`default` call of an empty value.
fn is_empty_initial(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Lit(_) => true,
        syn::Expr::Paren(inner) => is_empty_initial(&inner.expr),
        syn::Expr::Group(inner) => is_empty_initial(&inner.expr),
        syn::Expr::Path(path) => path.path.segments.last().is_some_and(|s| s.ident == "None"),
        syn::Expr::Call(call) => {
            let name = match call.func.as_ref() {
                syn::Expr::Path(path) => path.path.segments.last().map(|s| s.ident.to_string()),
                _ => None,
            };
            call.args.is_empty()
                || (name.as_deref().is_some_and(is_constructor)
                    && call.args.first().is_some_and(is_empty_initial))
        }
        syn::Expr::MethodCall(call) => {
            is_constructor(&call.method.to_string())
                && (call.args.is_empty() || call.args.first().is_some_and(is_empty_initial))
        }
        syn::Expr::Block(block) => is_seeded_block(&block.block),
        _ => false,
    }
}

/// `{ let mut x = <empty>; x.update(<empty>); x }`: a cache seeded with
/// empty values in a constructor.
fn is_seeded_block(block: &syn::Block) -> bool {
    let Some((tail, statements)) = block.stmts.split_last() else {
        return false;
    };
    let syn::Stmt::Expr(syn::Expr::Path(_), None) = tail else {
        return false;
    };
    statements.iter().all(|statement| match statement {
        syn::Stmt::Local(local) => local
            .init
            .as_ref()
            .is_some_and(|init| init.diverge.is_none() && is_empty_initial(&init.expr)),
        syn::Stmt::Expr(syn::Expr::MethodCall(call), Some(_)) => {
            matches!(call.receiver.as_ref(), syn::Expr::Path(_))
                && call.args.iter().all(is_empty_initial)
        }
        _ => false,
    })
}

fn is_constructor(name: &str) -> bool {
    matches!(name, "new" | "signal" | "var" | "default")
}

/// The last segment of an impl's self type.
fn type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(path) => path.path.segments.last().map(|s| s.ident.to_string()),
        syn::Type::Reference(reference) => type_name(&reference.elem),
        syn::Type::Paren(paren) => type_name(&paren.elem),
        syn::Type::Group(group) => type_name(&group.elem),
        _ => None,
    }
}
```

## Q4：付録 B：`dev/xtask/src/state_fields/tests.rs` の末尾に足す試験（全文。D14 付録 B の X3 の 2 本の後に置く）

```rust
/// Run X4 and X5 over fixtures, one per `(module, text)`, each in its own file.
fn verify_sources(modules: &[(&str, &str)]) -> Vec<String> {
    let sources: Vec<crate::check_source::Source> = modules
        .iter()
        .map(|(module, text)| {
            let mut source = fixture(text);
            source.module = (*module).to_owned();
            source.path =
                format!("crates/kivact-ui-soft/src/{}.rs", module.replace("::", "/")).into();
            source
        })
        .collect();
    let mut errors = Vec::new();
    super::verify(&sources, &|_| true, &mut errors);
    errors
}

/// A struct whose field `a` carries the projection `reason`.
fn projected(reason: &str, visibility: &str) -> String {
    format!(
        "struct S {{\n// xtask-allow(state-fields): projection: {reason}\n{visibility} a: Cell<bool>,\n// xtask-allow(state-fields): independent: probe b\nb: Cell<bool>,\n}}\n"
    )
}

/// `S` with the writer `S::publish`, followed by `rest`.
fn published(rest: &str) -> String {
    projected("written only by S::publish", "")
        + "impl S { fn publish(&self) { self.a.set(true); } }\n"
        + rest
}

/// The errors of `published(rest)` alone in module `widgets::list`.
fn verify_published(rest: &str) -> Vec<String> {
    verify_sources(&[("widgets::list", &published(rest))])
}

#[test]
fn projection_needs_an_existing_single_writer() {
    let errors = verify_published("impl S { fn other(&self) { self.a.set(false) } }");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("S.a is also written in other"),
        "{errors:?}"
    );
}

#[test]
fn projection_writer_must_exist() {
    let text = projected("written only by missing", "")
        + "impl S { fn publish(&self) { self.a.set(true) } }";
    let errors = verify_sources(&[("widgets::list", &text)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("projection writer missing of S.a does not exist"),
        "{errors:?}"
    );
}

#[test]
fn projection_allows_empty_initial_values_in_constructors() {
    let text = projected("written only by S::publish", "")
        + "impl S {\nfn new() -> Self { Self { a: Cell::new(false), b: Cell::new(false) } }\nfn publish(&self) { self.a.set(true) }\n}\n\
           struct T {\n// xtask-allow(state-fields): projection: written only by T::report\nsent: Sent<Option<u8>>,\n// xtask-allow(state-fields): independent: probe c\nc: bool,\n}\n\
           impl T {\nfn new() -> Self { Self { sent: { let mut sent = Sent::new(); sent.update(None); sent }, c: true } }\nfn report(&mut self) { self.sent.update(Some(1)); }\n}";
    let errors = verify_sources(&[("widgets::list", &text)]);
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn projection_marker_must_name_one_writer() {
    let text = projected("readonly snapshot", "");
    let errors = verify_sources(&[("widgets::list", &text)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("must name one writer"), "{errors:?}");
}

#[test]
fn projection_fields_visible_to_the_whole_crate_are_rejected() {
    for (module, visibility) in [
        ("widgets::list", "pub(crate)"),
        ("widgets::list", "pub"),
        ("widgets", "pub(super)"),
        ("", ""),
    ] {
        let text = projected("written only by S::publish", visibility)
            + "impl S { fn publish(&self) { self.a.set(true) } }";
        let errors = verify_sources(&[(module, &text)]);
        assert_eq!(errors.len(), 1, "{module} {visibility}: {errors:?}");
        assert!(
            errors[0].contains("visible to the whole crate"),
            "{errors:?}"
        );
    }
}

#[test]
fn projection_fields_are_checked_in_the_module_their_visibility_allows() {
    let other = "struct T { a: Cell<bool> }\nfn touch(t: &T) { t.a.set(true) }";
    let private = published("");
    let errors = verify_sources(&[("widgets::list", &private), ("widgets::other", other)]);
    assert!(errors.is_empty(), "{errors:?}");
    let shared = projected("written only by S::publish", "pub(super)")
        + "impl S { fn publish(&self) { self.a.set(true) } }";
    let errors = verify_sources(&[("widgets::list", &shared), ("widgets::other", other)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("S.a is also written in touch"),
        "{errors:?}"
    );
}

#[test]
fn a_same_name_field_is_the_projection_unless_its_receiver_is_another_types_self() {
    let errors = verify_published(
        "struct T { a: Cell<bool> }\nimpl T { fn touch(&self) { self.a.set(true); } }",
    );
    assert!(errors.is_empty(), "{errors:?}");
    let errors = verify_published("struct T { a: Cell<bool> }\nfn touch(t: &T) { t.a.set(true); }");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("S.a is also written in touch"),
        "{errors:?}"
    );
}

#[test]
fn an_alias_of_the_field_is_unverified() {
    let errors = verify_published("fn other(s: &S) { let cell = &s.a; cell.set(false); }");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("S.a may be changed in other") && errors[0].contains("borrowed with &"),
        "{errors:?}"
    );
}

#[test]
fn a_reference_passed_to_another_function_is_unverified() {
    let errors = verify_published(
        "impl S { fn other(&mut self) { flip(&mut self.a); } }\nfn flip(cell: &mut Cell<bool>) { cell.set(false); }",
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("borrowed with &mut"), "{errors:?}");
}

#[test]
fn an_unknown_method_on_the_field_is_unverified_and_known_reads_pass() {
    let errors = verify_published(
        "impl S { fn read(&self) -> bool { self.a.get() && self.a.clone().get() } fn other(&self) { self.a.toggle(); } }",
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("may be changed in other") && errors[0].contains("calls `toggle`"),
        "{errors:?}"
    );
}

#[test]
fn writes_through_parts_handles_and_literals_are_writes() {
    let text = "struct S {\n// xtask-allow(state-fields): projection: written only by S::publish\na: RefCell<Inner>,\n// xtask-allow(state-fields): independent: probe b\nb: Cell<bool>,\n}\n\
                impl S {\nfn publish(&self) { self.a.borrow_mut().x = 1; }\nfn part(&mut self) { self.a.get_mut().x = 2; }\nfn handle(&self) { *self.a.borrow_mut() = Inner::default(); }\nfn copy(&self) -> Self { Self { b: Cell::new(true), ..self.clone() } }\n}";
    let errors = verify_sources(&[("widgets::list", text)]);
    assert_eq!(errors.len(), 3, "{errors:?}");
    for function in ["part", "handle", "copy"] {
        assert!(
            errors
                .iter()
                .any(|error| error.contains(&format!("S.a is also written in {function}"))),
            "{function}: {errors:?}"
        );
    }
}

#[test]
fn a_pattern_binding_of_the_field_is_unverified() {
    let errors = verify_published(
        "fn ignore(s: &S) { let S { a: _, .. } = s; }\nfn bind(s: &mut S) { let S { a, .. } = s; a.set(false); }",
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("may be changed in bind") && errors[0].contains("bound by a pattern"),
        "{errors:?}"
    );
}

#[test]
fn uses_inside_macros_are_checked() {
    let errors = verify_published(
        "impl S { fn checked(&self) { assert!(self.a.replace(true)); } fn opaque(&self) { let _ = vec![self.a.take(); 1]; } fn custom(&self) { flip!(self.a); } fn shown(&self) { println!(\"{}\", self.a.get()); } }",
    );
    assert_eq!(errors.len(), 3, "{errors:?}");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("S.a is also written in checked")),
        "{errors:?}"
    );
    for function in ["opaque", "custom"] {
        assert!(
            errors.iter().any(
                |error| error.contains(&format!("may be changed in {function}"))
                    && error.contains("inside a macro")
            ),
            "{function}: {errors:?}"
        );
    }
}

#[test]
fn a_writer_returning_a_reference_is_rejected() {
    let text = projected("written only by S::publish", "")
        + "impl S { fn publish(&self) -> &Cell<bool> { &self.a } }";
    let errors = verify_sources(&[("widgets::list", &text)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("returns a reference"), "{errors:?}");
}

#[test]
fn machine_state_projections_name_the_machine_transition() {
    let machine = "pub(super) enum State {\nClosed,\nOpen {\n// xtask-allow(state-fields): projection: written only by Machine::transition on Event::Preview\npreviewed: Option<u8>,\n// xtask-allow(state-fields): independent: probe hovered\nhovered: Option<u8>,\n},\n}\n\
                   pub(super) struct Machine;\n\
                   impl MachineTrait for Machine {\ntype State = State;\nfn transition(&self, state: &State, key: Option<u8>) -> State {\nlet mut next = State::Open { previewed: None, hovered: None };\nif let State::Open { previewed, .. } = &mut next { *previewed = key; }\nnext\n}\n}";
    let errors = verify_sources(&[("select::machine", machine)]);
    assert!(errors.is_empty(), "{errors:?}");
    let reset = "fn reset() -> State { State::Open { previewed: Some(1), hovered: None } }";
    let errors = verify_sources(&[("select::machine", machine), ("select::handle", reset)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("State::Open.previewed is also written in reset"),
        "{errors:?}"
    );
    let named = "fn transition() -> State { State::Open { previewed: Some(1), hovered: None } }";
    let errors = verify_sources(&[("select::machine", machine), ("select::handle", named)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("State::Open.previewed is also written in transition"),
        "{errors:?}"
    );
}

#[test]
fn independent_reasons_are_not_shared_across_files() {
    let text = "struct P {\n// xtask-allow(state-fields): independent: probe fact\na: bool,\n// xtask-allow(state-fields): independent: probe fact\nb: bool,\n}";
    let errors = verify_sources(&[("widgets", text)]);
    assert!(errors.is_empty(), "{errors:?}");
    let errors = verify_sources(&[("widgets", text), ("other", text)]);
    assert_eq!(errors.len(), 4, "{errors:?}");
    assert!(
        errors
            .iter()
            .all(|error| error.contains("reason repeats in")),
        "{errors:?}"
    );
}
```


---

# Q5：R06（W7 G9）・R08（W5 G7）・3.1（W5 G5）

（元の題：Q5（target lane E）：公開 API の指摘 R06・R08・3.1 の検証と修正設計）

対象 HEAD `6c812296`。試作は `W:/w/kv-rf-Q5`（detached、`CARGO_TARGET_DIR=W:/kv-target/E`）で、D12 の G5・G7 の試作差分（`<scratchpad>/r4-G5.patch`、`r4-G7.patch`）を当てた上に本書の修正を重ねて行った。試作の差分（参考。正は本書の文）：`<scratchpad>/q5/q5-R06.diff` と新規 file `<scratchpad>/q5/interaction_show_hide.rs`（R06）、`<scratchpad>/q5/q5-31-tests.diff`（3.1）、`<scratchpad>/q5/q5-R08-on-G7.diff`（G7 と R08 の api_shape.rs）、`<scratchpad>/q5/checkpoint3-all.diff`（G5・G7・R08・3.1・R06 の全部、新規 file を除く）。worktree `W:/w/kv-rf-Q5` は消した。`<scratchpad>` = `C:\Users\ryo\AppData\Local\Temp\claude\fd3f2b4db3f6db24\76c64dca-56c5-4a94-ab86-dbee8a09d336\scratchpad`。

---

## Q5：R06：G9 の開閉命令が公開 API 全体で揃っていない

### 判定：妥当（実際に起こる）。さらに、レビューの挙げていない不統一と潜在不具合が 3 件ある

事実（HEAD のコード）：
- `SelectApi::open_sheet`（`crates/kivact-interaction/src/select/handle.rs:385`）は、開いていれば `toggle()` で閉じる（反転）。`ComboboxApi::open_sheet`（`combobox/handle.rs:439`）は `Event::OpenSheet` を送るだけ（開く）。D12:501 は後者だけを `show_sheet` にする。同じ名前で意味が逆。
- D12:529 は `DisclosureApi::close`・`NavigationMenuApi::close`・`PopupApi::close(reason)`・`ToolWindowApi::close` を「同じ型に show が無い」という理由で残す。ARG-6（`SLIM_FAST_DESIGN.md:952`「状態を変える命令の method は動詞 show/hide」）と合わず、利用者は型ごとに open/close/show/hide を覚える必要がある。
- 追加 1（レビュー外）：「隠す」の第三の動詞 `dismiss` がある。`DelayedApi::dismiss`（`delayed/api.rs:132`）、`HoverSurfaceApi::dismiss`（`hover_corridor/api.rs:152`）、`HoverCardApi::dismiss`（`hover_card.rs:100`）。開く命令の別名 `NavigationMenuApi::pin(index)`（`navigation_menu.rs:267`）もある。
- 追加 2（レビュー外、意味の不具合）：`ComboboxApi::open` は開いていても `Event::Open` を送り、machine（`combobox/machine.rs:109`）が mode を `Browsing(Popover)` に戻す。つまり「開く」を 2 回呼ぶと、入力済みの query による絞込み（`Filtering`）が解除され、sheet も popover に戻る。内部でも Alt+ArrowDown（`combobox/handle.rs:593`）と touch の押下（`kivact-ui/src/combobox/render.rs:349`）がこれを呼ぶ。APG の combobox は Alt+Down を「開く（開いていれば何もしない）」と定める。
- 追加 3（レビュー外、潜在の panic）：`ComboboxApi::toggle`（`combobox/handle.rs:456`）は `self.is_open().get_untracked()` を呼ぶ。`is_open()` は closure から `into_value()` で値を作り、これは setup 専用（`SetupOnly { api: "IntoValue::into_value" }`）。handler から `toggle()` を呼ぶと panic する（試作で `show()` に同じ書き方をして、実 window の E2E で `kivact misuse: ... SetupOnly ... combobox/handle.rs:420` を再現した。`kivact-ui/src/combobox/render.rs:617` の `on_press(move || list.toggle())` はこの経路）。

### 規約（確定。設計者の決定）

ARG-6 を次の 5 行で具体化する。作業指示書 10 章の公開 API 変更の記録と、`crates/kivact-interaction/src/lib.rs` の crate doc に書く。

| ID | 規約 |
|---|---|
| SH-1 | 表示は `show`、非表示は `hide`、反転は `toggle`。open・close・dismiss・pin・open_sheet を命令の名前に使わない |
| SH-2 | `show`・`hide` は冪等：既にその状態なら何も変えない（surface・query・highlight も保つ）。`toggle` は反転し、2 回で元に戻る |
| SH-3 | 対象を選ぶ引数（`show(index)`、`show(key)`、`toggle(key)`）と、閉じる理由（`hide(reason)`）は付けてよい。引数で動詞を変えない |
| SH-4 | 面の種類は接尾辞にする（`show_sheet`）。`hide` は面によらず一つ |
| SH-5 | 状態の語は open/closed のまま（`is_open()`、setter `.open(binding)`（ARG-4）、`on_open_change`、`PopupCloseReason`）。React の `open`／`onOpenChange` と同じ語で、命令の動詞だけが show/hide（setter の `open` と名前が衝突しないため。ARG-6 の理由） |

理由：React・Solid の利用者は開閉を `open` の state と `setOpen(true/false)`・`setOpen(o => !o)` で書く。その命令形は Ariakit の store（`store.show()`／`hide()`／`toggle()`）や HTML の popover（`showPopover`／`hidePopover`／`togglePopover`）と同じ 3 語で、意味（冪等な show/hide と反転の toggle）も一致する。`hide(reason)` の理由は focus の戻し方（`FocusReturn` の `ReasonSet`）を決める意味のある引数で、外部の呼出し元は 1 件（`kivact-ui-creative/src/picker.rs:701`、`Selection`）しかないため、引数なしの別 method は作らない（API を増やさない）。

### 全数表（HEAD のコード。setter・query・factory は下の「表に入れないもの」に分けた）

数え方：`crates/kivact-interaction`、`crates/kivact-ui`、`crates/kivact-ui-{basic,soft,creative,game,pop}` の公開 type の公開 method で、`self` を取り、表示状態を変えるもの全部（`grep "pub fn (open|close|show|hide|toggle|dismiss|pin|expand|collapse|reveal|maximize|restore)\w*"` と各 `*Api`／`*Handle` の impl の通読）。kivact-ui・basic・game・pop には該当が 0 件（setter と factory だけ）。

| # | crate | HEAD | 新 | HEAD の意味 → 新の意味 | 呼出し元（compile error の実測） |
|---|---|---|---|---|---|
| C1 | interaction | `PopupApi::close(self, reason: PopupCloseReason)` | `hide(self, reason)` | 冪等 → 同じ | 13（interaction 内 12、creative `picker.rs` 1） |
| C2 | interaction | `DisclosureApi::close(self)` | `hide(self)` | 冪等 → 同じ | 0 |
| C3 | interaction | `MenuApi::open(self)` | `show(self)` | 冪等（開いていれば何もしない） → 同じ。`top_level("open")` → `top_level("show")` | 3（`menubar.rs`） |
| C4 | interaction | `MenuApi::close(self)` | `hide(self)` | 冪等 → 同じ。`top_level("close")` → `top_level("hide")` | 3（`menubar.rs`） |
| C5 | interaction | `ComboboxApi::open(self)` | `show(self)` | **開いていても mode を Browsing(Popover) に戻す → 開いていれば何もしない（意味の変更）** | 4 |
| C6 | interaction | `ComboboxApi::close(self)` | `hide(self)` | 冪等 → 同じ | 6 |
| C7 | interaction | `ComboboxApi::toggle(self)` | `toggle(self)` | 反転 → 同じ（handler からの panic を直す） | — |
| C8 | interaction | `ComboboxApi::open_sheet(self)` | `show_sheet(self)` | sheet で開いていても mode を Browsing(Sheet) に戻す → **sheet で開いていれば何もしない**（popover で開いていれば sheet へ移る、は同じ） | 1（`kivact-ui/src/combobox/render.rs:343`） |
| C9 | interaction | `SelectApi::open_sheet(self)` | `show_sheet(self)` | **開いていれば閉じる（反転） → 冪等（意味の変更）**。popover で開いていれば sheet へ移る | 1（同ファイル内の `trigger_with_sheet`。本体を書き換える） |
| C10 | interaction | （無し） | `SelectApi::show(self)`、`SelectApi::hide(self)` を新設 | 冪等。`toggle` と `show_sheet` が既にあるのに show/hide だけ無いため（Combobox と揃える） | — |
| C11 | interaction | `SelectApi::toggle(self)` | `toggle(self)` | 反転 → 同じ（本体を show/hide で書く） | — |
| C12 | interaction | `InlineEditApi::open(self, key: K)` | `show(self, key)` | 冪等（同じ key なら machine が stay） → 同じ（D12 G9 の順 4 と同じ） | 6（creative `numeric/implementation.rs` 3、`outliner/view.rs` 1、`dock/chrome.rs` 1、`e2e/ui_field.rs` 1） |
| C13 | interaction | `NavigationMenuApi::pin(self, index)` | `show(self, index)` | 冪等（`Pinned(index)` にする） → 同じ | 3（`kivact-ui/src/navigation_menu.rs` 2、soft `navigation_session.rs` 1） |
| C14 | interaction | `NavigationMenuApi::toggle(self, index)` | `toggle(self, index)` | 反転（`Pinned(index)` ⇔ `Closed`） → 同じ | — |
| C15 | interaction | `NavigationMenuApi::close(self)` | `hide(self)` | 冪等 → 同じ | 6（`kivact-ui/src/navigation_menu.rs` 5、soft `navigation_session.rs` 1） |
| C16 | interaction | `TreeApi::toggle(self, key: K)` | `toggle(self, key)` | 反転 → 同じ | — |
| C17 | interaction | `DelayedApi::dismiss(self)` | `hide(self)` | 冪等（`wanted` が false になるまで隠す） → 同じ | 2（`tooltip/api.rs`、`e2e/interaction_delayed.rs`） |
| C18 | interaction | `HoverSurfaceApi::dismiss(self)` | `hide(self)` | 冪等 → 同じ | 1（`hover_card.rs`、定義の変更で直す） |
| C19 | interaction | `HoverCardApi::dismiss(self)` | `hide(self)` | 冪等 → 同じ | 1（`kivact-ui/src/hover_card.rs:180`） |
| C20 | soft | `LightboxHandle::show(self, index: usize)` | 変えない | 冪等（同じ index） | — |
| C21 | soft | `LightboxHandle::close(self)` | `hide(self)` | 冪等 → 同じ（D12 G9 の順 5 と同じ） | 1（`lightbox.rs` 内） |
| C22 | soft | `AppShellMobile::show(self)`、`toggle(self)` | 変えない | 冪等／反転 | — |
| C23 | soft | `AppShellMobile::close(self)` | `hide(self)` | 冪等 → 同じ（D12 G9 の順 5 と同じ） | 4（`top_nav.rs` 3、`side_navigation/items.rs` 1） |
| C24 | creative | `ToolWindowApi::close(self)` | `hide(self)` | 冪等 → 同じ | 6（`feedback.rs` 1、`examples/kivact-ui-creative/src/creative/feedback.rs` 5） |

表に入れないもの（変えない。判断を残さないため明記）：
- factory（命令でない）：`MenuApi::show(factory)`・`CommandPaletteApi::show(factory)` は D12 G9 の順 1 のとおり `while_open(factory)`（呼出し元 11：`e2e/interaction_menu.rs` 3、`e2e/apg_menubar.rs` 2、`e2e/interaction_palette.rs` 1、`kivact-ui/src/menu.rs` 1、soft `tab_list.rs`・`nav_heading.rs`・`tests/consumer/navigation.rs` 各 1、basic `menubar.rs` 1）。`DialogApi::close_trigger(self) -> Part` は Part の factory。
- setter・option（binding か値を受けて `Self` を返す）：`.open(binding)`（kivact-ui の各部品、soft `app_shell`・`mobile_nav`・`media_overlay`）、`open_delay`・`close_delay`・`open_on_arrow`・`close_on`・`DialogApi::dismiss(DismissOn)`・`dismissible`・`show_clear`・`hide_label` など。
- query：`is_open`・`is_sheet_open`・`is_expanded`・`state()`・`phase()`。
- 開閉でない命令：`ToasterApi::dismiss(self, id)`（toast を待ち行列から取り除く。項目の削除で、Sonner の `toast.dismiss(id)` と同じ語）、`DelayedApi::pulse`・`HoverSurfaceApi::pulse`（見せてから隠す一回の合図）、creative `DockLayout::close(&mut self, id)`・`close_others`・`close_container`・`maximize`・`restore`（data の値の編集で、取り除いた tab を返す。`Vec::remove` に当たる）、`NavigationMenuApi::hover`・`leave`・`hover_after`・`leave_after`・`panel_removed`（kit が入力を machine に転送する口）。
- kivact-test の `Locator::toggle` 等（試験の操作）、kivact-core の `view::show(when, factory)`（Solid の `<Show>`）。

### 修正

#### 対象 unit と所有ファイル

- **W7 の G9**（W7 は 1 lane で FINAL-1 → G9 → FINAL-2 を順に行う）。D12「G9」の改名表の **順 2・3・4・5 を下の「G9-C 表」で置き換え**、順 1 の直後に行う（順 1 の `while_open` が先でないと `MenuApi::show` が二重になる）。順 4（InlineEdit）は G9-C の C-f で、D12 の順 4 の docs の欄もそこで行う。順 6 以降はそのまま。D12「G9」の「変えないもの」の文から「`DisclosureApi::close`・`NavigationMenuApi::close`・`PopupApi::close(reason)`・`ToolWindowApi::close`（同じ型に `show` が無い単独の命令）、」を削り、代わりに上の「表に入れないもの」の 4 項目の箇条を置く。
- 所有ファイル（D12 G9 の「compile error と D12 が示す全ての file」に加え、明示で）：新規 `dev/tests/kivact-public-api/tests/e2e/interaction_show_hide.rs`、`dev/tests/kivact-public-api/tests/e2e/main.rs`（mod 1 行）、下の「docs・rustdoc」の file。
- 一人の unit で行えるか：行える。実測で、定義の変更（C-a〜C-m）の後、`cargo check --workspace --all-targets --all-features` の compile error（E0599／E0061）が 6 巡で全呼出し元（70 箇所・30 file）を示した（crate の依存順に止まるので巡回が要る）。compile error 駆動で足りる。例外は docs と rustdoc の文（下の表。compile に出ない）と、`CommandPaletteApi::show(factory)` の呼出し（定義が無くなるので E0599 `no method named show` で出る。`MenuApi` は新しい `show(self)` と引数の数が違うので E0061 で出る）。

#### G9-C 表（上から順に行う）

各行：定義を変え、`cargo check --workspace --all-targets --all-features` の E0599／E0061 が示す呼出し元の method 名を同じ表どおりに直す（`close`→`hide`、`open`→`show`、`dismiss`→`hide`、`pin`→`show`、`open_sheet`→`show_sheet`）。**同名の別 type の method（`Path::close`、`DockLayout::close`、`ToasterApi::dismiss`、`Parent::close` 等）を grep で一括置換しない**。compile error の示す行だけを直す。

| 順 | 定義（file） | 変更 |
|---|---|---|
| C-a | `crates/kivact-interaction/src/popup/api.rs` の `pub fn close(self, reason: PopupCloseReason)` | 名前を `hide`。本体は同じ。直上の doc の 1 行目「Request closure and apply ..」は変えない |
| C-b | `crates/kivact-interaction/src/disclosure.rs` の `pub fn close(self)` | `/// Hide: write `false` to the binding.` と `pub fn hide(self)` |
| C-c | `crates/kivact-interaction/src/menu/handle.rs` の `pub fn open(self)`・`pub fn close(self)` | `pub fn show(self)`（本体 1 行目 `self.top_level("show");`）、`pub fn hide(self)`（`self.top_level("hide");`）。doc の「Open the menu as ..」「Close every level ..」の動詞は `Show`・`Hide` |
| C-d | `crates/kivact-interaction/src/combobox/handle.rs` の `open_sheet`・`open`・`close`・`toggle` | 下のコード A で置き換える |
| C-e | `crates/kivact-interaction/src/select/handle.rs` の `open_sheet`・`toggle`、`trigger_with_sheet` の `on_press` | 下のコード B で置き換える |
| C-f | `crates/kivact-interaction/src/inline_edit.rs` の `pub fn open(self, key: K)` | `pub fn show(self, key: K)`。docs：`docs/for-kivact-user/en/how-to-guide/forms.md:322` の `rename.open(id)` → `rename.show(id)`、同ファイル本文の `` `open(id)` ``（302 行付近）、jp `forms.md:255` 本文の `` `open(id)` `` と `:273` のコード（D12 の順 4 の docs の欄と同じ） |
| C-g | `crates/kivact-interaction/src/navigation_menu.rs` の `pin`・`close` | `/// Show the panel `index` pinned: it stays open when the pointer` `/// leaves. Nothing changes while it is pinned.` `pub fn show(self, index: usize)`、`/// Hide the open panel, after dismissal or choosing a link.` `pub fn hide(self)`。本体は同じ |
| C-h | `crates/kivact-interaction/src/delayed/api.rs` の `pub fn dismiss(self)` | `pub fn hide(self)`（doc はそのまま「Hide now. ..」） |
| C-i | `crates/kivact-interaction/src/hover_corridor/api.rs` の `HoverSurfaceApi::dismiss` | `pub fn hide(self) { self.delayed.hide(); }` |
| C-j | `crates/kivact-interaction/src/hover_card.rs` の `HoverCardApi::dismiss` | `pub fn hide(self)`、本体 2 行目 `self.surface.hide();` |
| C-k | `crates/kivact-ui-soft/src/lightbox.rs` の `LightboxHandle::close` | `/// Hide this gallery.` `pub fn hide(self)`（D12 G9 の順 5） |
| C-l | `crates/kivact-ui-soft/src/mobile_nav.rs` の `AppShellMobile::close` | `/// Hide the configured drawer; do nothing outside a shell.` `pub fn hide(self)`（D12 G9 の順 5） |
| C-m | `crates/kivact-ui-creative/src/feedback.rs` の `ToolWindowApi::close` | `/// Hide the window: its `open` value becomes `false`.` `pub fn hide(self)` |

コード A（`combobox/handle.rs`。`/// Open the same list in a touch sheet.` から `toggle` の閉じ括弧までを置き換える）：

```rust
    /// Show the list in a touch sheet; an open popover list moves to the
    /// sheet. Nothing changes while the sheet shows.
    pub fn show_sheet(self) {
        let in_sheet = self.session.state().with_untracked(|state| {
            matches!(
                state.mode,
                machine::ComboMode::Browsing(crate::surface::Surface::Sheet)
                    | machine::ComboMode::Filtering(crate::surface::Surface::Sheet)
            )
        });
        if !in_sheet {
            self.session.send(machine::Event::OpenSheet);
        }
    }

    /// Show a popup list. Nothing changes while the list shows (its query and
    /// surface stay), and nothing happens without [`popup`](Self::popup).
    pub fn show(self) {
        if !self.shown() {
            self.session.send(machine::Event::Open);
        }
    }

    /// Hide a popup list. Nothing changes while it is hidden, and nothing
    /// happens without [`popup`](Self::popup).
    pub fn hide(self) {
        if let PresentationMode::Popup(_) = self.mode.get() {
            self.session.send(machine::Event::Close);
        }
    }

    /// Show a hidden popup list and hide a shown one.
    pub fn toggle(self) {
        if let PresentationMode::Popup(_) = self.mode.get() {
            if self.shown() {
                self.hide();
            } else {
                self.show();
            }
        }
    }

    /// Whether the list shows now, read untracked: [`is_open`](Self::is_open)
    /// makes a value, which only setup may do, and the commands run in
    /// handlers.
    fn shown(self) -> bool {
        match self.mode.get() {
            PresentationMode::Inline => true,
            PresentationMode::Popup(_) => self
                .session
                .state()
                .with_untracked(|state| state.mode.is_open()),
        }
    }
```

（`ComboMode::is_open` は `pub(super)` で同じ crate の `combobox` module から見える。state-fields：field は足さない。`shown` は導出で、新しい状態は無い。）

コード B（`select/handle.rs`）：
1. `/// Open the options in a touch sheet, or close an already open select.` から `open_sheet` の閉じ括弧までを次で置き換える：
```rust
    /// Show the options in a touch sheet; an open popover list moves to the
    /// sheet. Nothing changes while the sheet shows. Use
    /// [`Self::trigger_with_sheet`] for a trigger that chooses the surface.
    pub fn show_sheet(self) {
        let in_sheet = self.session.state().with_untracked(|state| {
            matches!(
                state,
                machine::State::Open {
                    surface: machine::Surface::Sheet,
                    ..
                }
            )
        });
        if !in_sheet {
            self.session.send(machine::Event::OpenSheet);
        }
    }

    /// Show the options in the popover. Nothing changes while they show.
    pub fn show(self) {
        if !self.is_open().get_untracked() {
            self.open_binding().set(true);
        }
    }

    /// Hide the options as [`PopupCloseReason::Programmatic`]. Nothing
    /// changes while they are hidden.
    pub fn hide(self) {
        self.popup.hide(PopupCloseReason::Programmatic);
    }
```
（`SelectApi::is_open` は popup が持つ値を返すだけで、値を作らないので handler で読める。）
2. `toggle` を次にする：
```rust
    /// Show hidden options and hide shown ones.
    pub fn toggle(self) {
        if self.is_open().get_untracked() {
            self.hide();
        } else {
            self.show();
        }
    }
```
3. `trigger_with_sheet` の `.on_press(move || { let sheet = choose(); if sheet { self.open_sheet(); } else { self.toggle(); } })` の中身を次にする（chooser は毎回先に呼ぶ。振る舞いは HEAD と同じ：開いていれば閉じ、閉じていれば chooser の選んだ面で開く）：
```rust
                    let sheet = choose();
                    if self.is_open().get_untracked() {
                        self.hide();
                    } else if sheet {
                        self.show_sheet();
                    } else {
                        self.show();
                    }
```

#### docs・rustdoc（compile に出ない。必ず直す）

| file | 変更 |
|---|---|
| `crates/kivact-interaction/src/lib.rs:17-18` | 「The open query is `is_open()` and the commands are `open()`, `close()` and `toggle()` on a Copy `self`.」→「The open query is `is_open()` and the commands are `show()`, `hide()` and `toggle()` on a Copy `self`: `show` and `hide` change nothing when the surface already is so, `toggle` inverts, and a kind of surface is a suffix (`show_sheet()`).」 |
| `crates/kivact-interaction/src/menu.rs:30` | `[`MenuApi::open`] and [`MenuApi::close`]` → `[`MenuApi::show`] and [`MenuApi::hide`]` |
| `crates/kivact-interaction/src/delayed/api.rs:5` | `[`DelayedApi::dismiss`] hides it at once` → `[`DelayedApi::hide`] hides it at once` |
| `crates/kivact-interaction/src/tooltip/api.rs:95` | `[`DelayedApi::dismiss`](crate::DelayedApi::dismiss)` → `[`DelayedApi::hide`](crate::DelayedApi::hide)` |
| `crates/kivact-interaction/src/disclosure.rs:6` | `[`DisclosureApi::close`] writes` → `[`DisclosureApi::hide`] writes` |
| `crates/kivact-ui-creative/src/feedback.rs:295,346,354,374` | `[`close`](ToolWindowApi::close) closes` → `[`hide`](ToolWindowApi::hide) closes`；`[`ToolWindowApi::close`]`（2 箇所）→ `[`ToolWindowApi::hide`]`；doc の例 `window.close()` → `window.hide()` |
| `crates/kivact-ui-creative/README.md:212,224,231,315` | `whose `close()` closes` → `whose `hide()` closes`；`window.close()` → `window.hide()`；`` `ToolWindowApi::close` `` → `` `ToolWindowApi::hide` ``；`(`.close()`)` → `(`.hide()`)` |
| `docs/for-kivact-user/en/how-to-guide/custom-controls.md:29`、jp `:20` | `` `dismiss()` `` → `` `hide()` `` |
| `docs/for-kivact-user/{en,jp}/how-to-guide/navigation.md:41` | en「`hover_after`, `leave_after`, `pin`, `toggle` and `close` send transitions」→「`hover_after`, `leave_after`, `show`, `toggle` and `hide` send transitions」。jp も同じ 5 語の並びを `show`、`toggle`、`hide` に |
| `crates/kivact-ui-soft/README.md:328` | 改名表の行に `` `LightboxHandle::close()`, `AppShellMobile::close()` `` → `` `hide()` `` を足す（D12 の順 5 の docs の欄と同じ） |

#### 追加する試験（E2E。名前と本体）

`dev/tests/kivact-public-api/tests/e2e/main.rs` の `mod interaction_select;` の次の行に `mod interaction_show_hide;` を足し、新規 `dev/tests/kivact-public-api/tests/e2e/interaction_show_hide.rs` を次の内容で作る（`cargo fmt` 済みの形）。各 handler は命令を 2 回呼ぶ（ボタンの押下は surface の外の press なので、1 回ずつ押す形では外側の press が閉じた後に 2 回目が開き、反転でも通ってしまう。1 つの handler で 2 回呼ぶと冪等性だけを見る）。Combobox は入力欄の key handler から呼ぶ（focus が入力欄に残り、focus の離脱で閉じない）。

```rust
//! Role: Tests the commands that show and hide an interaction's surface.
//!
//! `show` and `hide` are idempotent: a second call changes nothing. `toggle`
//! inverts, so two calls return to the start. Each handler calls its command
//! twice, so a press outside the surface cannot hide what the first call did.
//! Through the public API only.

use crate::support::mount;
use kivact::interaction::{
    ListItem, MenuEntry, NavOpen, Selection, use_combobox, use_menu, use_navigation_menu,
    use_select, use_text_edit,
};
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::for_each;

fn colors() -> Vec<ListItem<&'static str>> {
    vec![
        ListItem::new("red", "Red"),
        ListItem::new("green", "Green"),
        ListItem::new("pink", "Pink"),
    ]
}

/// Press the button `name` once.
fn press(screen: &Screen, name: &str) {
    screen.get_by_role(Role::Button, name).click();
    screen.run_until_idle();
}

#[test]
fn select_show_twice_stays_open_and_toggle_twice_returns() {
    let (screen, (open, sheet)) = mount(|| {
        let selected = signal(None::<&'static str>);
        let select = use_select(colors(), selected);
        let view = column((
            node()
                .part(select.trigger())
                .name("Color")
                .child(text(select.label())),
            button("Show twice").on_press(move || {
                select.show();
                select.show();
            }),
            button("Hide twice").on_press(move || {
                select.hide();
                select.hide();
            }),
            button("Toggle twice").on_press(move || {
                select.toggle();
                select.toggle();
            }),
            button("Sheet twice").on_press(move || {
                select.show_sheet();
                select.show_sheet();
            }),
        ));
        ((select.is_open(), select.is_sheet_open()), view)
    });
    press(&screen, "Show twice");
    assert!(open.get_untracked(), "a second show does not hide");
    press(&screen, "Hide twice");
    assert!(!open.get_untracked(), "a second hide does not show");
    press(&screen, "Toggle twice");
    assert!(!open.get_untracked(), "two toggles return to hidden");
    press(&screen, "Sheet twice");
    assert!(open.get_untracked(), "a second show_sheet does not hide");
    assert!(sheet.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn combobox_show_keeps_a_shown_list_and_its_filter() {
    let (screen, (open, filtering)) = mount(|| {
        let value = signal(None::<&'static str>);
        let combo =
            use_combobox(colors(), Selection::Single(value.into_binding())).popup(signal(false));
        // Shortcuts on the field: the focus stays in it.
        let shortcuts = move |event: KeyEvent| {
            match event.key() {
                Key::F(5) => {
                    combo.show();
                    combo.show();
                }
                Key::F(6) => {
                    combo.hide();
                    combo.hide();
                }
                Key::F(7) => {
                    combo.toggle();
                    combo.toggle();
                }
                Key::F(8) => combo.toggle(),
                _ => return Handled::No,
            }
            Handled::Yes
        };
        let view = column((
            node()
                .name("Color")
                .on_key(shortcuts)
                .part(use_text_edit(combo.query()).control())
                .part(combo.input()),
            show(combo.is_open(), move || {
                column(for_each(
                    combo.visible(),
                    |key| *key,
                    move |key| {
                        let key = key.with(|key| *key);
                        node().part(combo.item(key)).child(text(key))
                    },
                ))
                .part(combo.content())
                .name("Colors")
            }),
        ));
        ((combo.is_open(), combo.is_filtering()), view)
    });
    let field = screen.get_by_role(Role::ComboBox, "Color");
    field.focus();
    field.type_text("p");
    screen.run_until_idle();
    assert!(
        open.get_untracked() && filtering.get_untracked(),
        "typing filters"
    );
    field.press_key(Key::F(5));
    assert!(open.get_untracked(), "show does not hide");
    assert!(filtering.get_untracked(), "show keeps the query's filter");
    field.press_key(Key::F(7));
    assert!(open.get_untracked(), "two toggles return to shown");
    field.press_key(Key::F(6));
    assert!(!open.get_untracked(), "a second hide does not show");
    field.press_key(Key::F(8));
    assert!(open.get_untracked(), "toggle shows a hidden list");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn menu_show_twice_stays_open() {
    let (screen, open) = mount(|| {
        let menu = use_menu(vec![MenuEntry::item("cut", "Cut")]);
        let view = column((
            node().part(menu.trigger()).name("Edit").child(text("Edit")),
            menu.while_open(move || {
                column(node().part(menu.item("cut")).child(text("Cut")))
                    .part(menu.content())
                    .name("Edit menu")
            }),
            button("Show twice").on_press(move || {
                menu.show();
                menu.show();
            }),
            button("Hide twice").on_press(move || {
                menu.hide();
                menu.hide();
            }),
        ));
        (menu.is_open(), view)
    });
    press(&screen, "Show twice");
    assert!(open.get_untracked(), "a second show does not hide");
    screen.get_by_text("Cut");
    press(&screen, "Hide twice");
    assert!(!open.get_untracked(), "a second hide does not show");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn navigation_menu_show_is_idempotent_and_toggle_twice_returns() {
    let (screen, state) = mount(|| {
        let menu = use_navigation_menu();
        let view = column((
            button("Show twice").on_press(move || {
                menu.show(0);
                menu.show(0);
            }),
            button("Hide twice").on_press(move || {
                menu.hide();
                menu.hide();
            }),
            button("Toggle twice").on_press(move || {
                menu.toggle(0);
                menu.toggle(0);
            }),
        ));
        (menu.state(), view)
    });
    press(&screen, "Show twice");
    assert_eq!(
        state.get_untracked(),
        NavOpen::Pinned(0),
        "a second show does not hide"
    );
    press(&screen, "Toggle twice");
    assert_eq!(
        state.get_untracked(),
        NavOpen::Pinned(0),
        "two toggles return"
    );
    press(&screen, "Hide twice");
    assert_eq!(
        state.get_untracked(),
        NavOpen::Closed,
        "a second hide does not show"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```

（意味を変えた C5・C9 の試験は、それぞれ `combobox_show_keeps_a_shown_list_and_its_filter` の「show keeps the query's filter」と `select_show_twice_stays_open_and_toggle_twice_returns` の「a second show_sheet does not hide」。C7 の panic の修正は同じ combobox の試験の F7・F8（handler からの `toggle`）が確かめる。）

#### 閉じ方（state-fields・xtask）

- 新しい field・enum・local の状態は無い（`shown`・`in_sheet` は machine の state の導出で、書き手は machine だけ）。state-fields の marker は不要。
- api-shape：新しい method は `self` を取り `()` を返す命令で、規則に当たらない（試作で `cargo xtask check --no-write --only api-shape` は 0 件）。

#### 完了時の確認（D12 G9 の行を次で置き換える）

- `cargo check --workspace --all-targets --all-features`
- `cargo test -p kivact-interaction --doc`、`cargo test -p kivact-ui-creative --doc`（`feedback.rs:374` の例）
- `cargo doc -p kivact-interaction -p kivact-ui-creative -p kivact-ui-soft --no-deps` の出力に、`unresolved link` で `show`・`hide`・`open`・`close`・`dismiss`・`pin`・`open_sheet`・`while_open` を含む行が無いこと。**D12 の「警告 0」は HEAD で既に満たせない**（HEAD の時点で `kivact-interaction` に 4 件（`calendar/handle.rs:17`、`combobox/handle.rs:592` の `Self::multiple`、`toggle.rs:232`、`tree.rs:351`）、`kivact-ui-creative` に 4 件の既存の警告がある）ので、この判定に替える。
- `cargo xtask check --no-write --only api-shape`
- `KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 1800 -- cargo test -p kivact-public-api-tests --test e2e -- interaction_show_hide interaction_select interaction_combobox apg_combobox interaction_menu apg_menu interaction_palette interaction_delayed interaction_tooltip interaction_dialog apg_tabs ui_toolbar apg_tree ui_field ui_navigation ui_components`
- `KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 3600 -- cargo test -p kivact-docs --all-features --doc -- --test-threads=1`
- `grep -rnE "\.(open_sheet|pin)\(|DelayedApi::dismiss|HoverCardApi::dismiss|ToolWindowApi::close|DisclosureApi::close|MenuApi::(open|close)\b" crates docs examples dev --include=*.rs --include=*.md` が 0 行。

### 確かめたこと（試作・試験の結果）

- 定義の変更（C-a〜C-m）の後、E0599／E0061 を行と列で直す script（`<scratchpad>/q5/r06_fix.py`）で 6 巡、手で 1 箇所（`interaction_palette.rs:73` の `CommandPaletteApi::show(factory)`）。合計 70 箇所・30 file。最後に `cargo check --workspace --all-targets --all-features` が error・warning 0。
- E2E（実 window）：`interaction_show_hide` の 4 本が成功。**意味の変更が検出されること**：`SelectApi::show_sheet` を HEAD の反転に、`ComboboxApi::show` を HEAD の「常に Open を送る」に戻すと、`select_show_twice_stays_open_and_toggle_twice_returns`（67 行「a second show_sheet does not hide」）と `combobox_show_keeps_a_shown_list_and_its_filter`（126 行「show keeps the query's filter」）が失敗した。
- 最初の試作で `show()` に `self.is_open().get_untracked()` を書いた時、既存の `interaction_combobox::the_arrows_open_..`・`apg_combobox` の 4 本が `SetupOnly { api: "IntoValue::into_value" }` で panic した（Alt+Down の handler から `show` を呼ぶため）。HEAD の `toggle` が同じ書き方であること（追加 3）の再現で、コード A の `shown()` で直り、全部成功した。
- 既存の E2E：`interaction_`（全 interaction の E2E）・`apg_combobox`・`apg_menu`（menu・menu_button・menubar）・`apg_listbox`・`ui_field`・`ui_navigation`・`ui_components`・`component_macro` の 122 本が成功（実 window）。
- `cargo doc` の新しい警告は `palette/handle.rs:208`（D12 G9 の順 1 で直す）、`tooltip/api.rs:95`、`feedback.rs:295,346,354` だけ（上の docs の表に全部ある）。`menu.rs:30`・`delayed/api.rs:5`・`disclosure.rs:6` は private module の doc なので警告に出ないが、表のとおり直す。

---

## Q5：R08：G7 の callback の lint が等価な書式で回避できる

### 判定：妥当（guard として不完全。現 consumer の違反は 0 件）

D12:295 の `unit_fn` は `syn::Type::ImplTrait` で `output` が `ReturnType::Default` のものだけを見る。`impl Fn() -> ()`（output が空の tuple）、`F: Fn()`（型引数、`<..>` 内も where 句も）、`impl Fn() -> R`（`R: HandlerResult`）は builder method（receiver あり・`-> Self`）でも通る。`on_` の method は `fn_bound_handler` が入力 0 個を拒むので既に全部の書式で検出される（隙間は `on_` で始まらない builder だけ）。

### 修正

- 対象：**W5 lane B の G7**。所有ファイルは変わらない（`dev/xtask/src/api_shape.rs`）。D12「G7」の修正 3 の「`fn handler_type_param` の前に 2 関数を足す」の `unit_fn` を使わず、次にする。D12 G7 の修正 3 の他の手順（`fn_bound_handler` の入力 0 個の拒否は下の `fn_output_handler` に含まれる、`check_signature` への `if` の追加、2 つの既存試験の期待値 6・4 の変更）は行う。
- 判定は「`Fn` の入力が 0 個」で行い、戻り値で handler と factory を分ける（factory `impl Fn() -> V` は値を返すので handler でない。D12 が実測した 13 件の誤検出を避けるため、戻り値を無視することはできない）。戻り値が無い・`()`・`HandlerResult` の型引数のとき handler。型の位置は `impl ..`、`<F: ..>`、`where F: ..` を一つの関数 `bounds_of` で集め、既存の `fn_output_handler`・`handler_type_param` も同じ関数で書き直す（重複していた型引数・where 句の走査を 1 箇所にする。`fn_bound_handler`・`has_handler_bound` は不要になり消す）。
- 手順：`dev/xtask/src/api_shape.rs` の `fn check_signature` の G7 の `if` の条件を `args.iter().any(|ty| unit_handler(ty, sig))` にする（`cargo fmt` 後の形）：
```rust
    if method
        && sig.receiver().is_some()
        && returns_self(sig)
        && args.iter().any(|ty| unit_handler(ty, sig))
    {
        source.error(
            "api-shape",
            sig.span(),
            "a builder method takes a handler without a payload as impl Into<Callback<()>> (ARG-1)",
            errors,
        );
    }
```
  次に、`/// `impl Fn(..) -> R` or a generic `F: Fn(..) -> R`, where `R` has a` の行から `fn check_impl(` の直前までを、次で置き換える（`fn_output_handler`・`fn_bound_handler`・`returns_self`・`unit_fn`・`handler_type_param`・`has_handler_bound` がこの範囲にある）：
```rust
/// `impl Fn(..) -> R` or a generic `F: Fn(..) -> R`, where `R` has a
/// `HandlerResult` bound.
fn fn_output_handler(ty: &syn::Type, sig: &syn::Signature) -> bool {
    bounds_of(ty, sig).into_iter().any(|bound| {
        fn_arguments(bound).is_some_and(|arguments| {
            !arguments.inputs.is_empty()
                && matches!(&arguments.output, syn::ReturnType::Type(_, output)
                    if handler_type_param(output, sig))
        })
    })
}

/// `-> Self`.
fn returns_self(sig: &syn::Signature) -> bool {
    matches!(&sig.output, syn::ReturnType::Type(_, ty)
        if matches!(ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("Self")))
}

/// A handler without a payload taken as a closure: an `Fn` bound with no
/// input that returns nothing (`Fn()`, `Fn() -> ()`) or a `HandlerResult`
/// (`Fn() -> R`), written as `impl ..` or as a type parameter bounded in
/// `<..>` or `where`. A factory `Fn() -> V` returns a value and is not one.
fn unit_handler(ty: &syn::Type, sig: &syn::Signature) -> bool {
    bounds_of(ty, sig).into_iter().any(|bound| {
        fn_arguments(bound).is_some_and(|arguments| {
            arguments.inputs.is_empty()
                && match &arguments.output {
                    syn::ReturnType::Default => true,
                    syn::ReturnType::Type(_, output) => {
                        matches!(output.as_ref(), syn::Type::Tuple(tuple) if tuple.elems.is_empty())
                            || handler_type_param(output, sig)
                    }
                }
        })
    })
}

/// The `(..) -> ..` of an `Fn` bound.
fn fn_arguments(bound: &syn::TypeParamBound) -> Option<&syn::ParenthesizedGenericArguments> {
    let syn::TypeParamBound::Trait(bound) = bound else {
        return None;
    };
    let segment = bound.path.segments.last()?;
    match &segment.arguments {
        syn::PathArguments::Parenthesized(arguments) if segment.ident == "Fn" => Some(arguments),
        _ => None,
    }
}

/// The bounds of an argument type: those of `impl ..`, or of the type
/// parameter it names, from `<..>` and from the `where` clause.
fn bounds_of<'a>(ty: &'a syn::Type, sig: &'a syn::Signature) -> Vec<&'a syn::TypeParamBound> {
    match ty {
        syn::Type::ImplTrait(ty) => ty.bounds.iter().collect(),
        syn::Type::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
            let name = &path.path.segments[0].ident;
            let inline = sig
                .generics
                .type_params()
                .filter(|parameter| parameter.ident == *name)
                .flat_map(|parameter| parameter.bounds.iter());
            let clause = sig
                .generics
                .where_clause
                .iter()
                .flat_map(|clause| clause.predicates.iter())
                .filter_map(|predicate| match predicate {
                    syn::WherePredicate::Type(predicate) if predicate.bounded_ty == *ty => {
                        Some(predicate.bounds.iter())
                    }
                    _ => None,
                })
                .flatten();
            inline.chain(clause).collect()
        }
        _ => Vec::new(),
    }
}

/// A type parameter with a `HandlerResult` bound.
fn handler_type_param(ty: &syn::Type, sig: &syn::Signature) -> bool {
    matches!(ty, syn::Type::Path(_))
        && bounds_of(ty, sig).into_iter().any(|bound| {
            matches!(bound, syn::TypeParamBound::Trait(bound)
                if bound.path.segments.last().is_some_and(|segment| segment.ident == "HandlerResult"))
        })
}
```
- 試験（`mod tests` の `fn follows_alias_fields_but_not_private_fields` の前に足す）。`a`〜`e` の 5 書式が error、factory（`slot`）・payload あり（`pick`）・`Into<Callback<()>>`（`shared`）・`-> Self` でない（`run`）は error でない：
```rust
    #[test]
    fn rejects_a_builder_handler_without_payload_in_every_spelling() {
        let mut source = crate::check_source::fixture(
            "pub struct Widget; impl Widget { pub fn a(self, f: impl Fn()) -> Self { self } pub fn b(self, f: impl Fn() -> () + 'static) -> Self { self } pub fn c<F: Fn()>(self, f: F) -> Self { self } pub fn d<F>(self, f: F) -> Self where F: Fn() -> () + 'static { self } pub fn e<R: HandlerResult>(self, f: impl Fn() -> R) -> Self { self } pub fn slot<V: IntoView>(self, f: impl Fn() -> V) -> Self { self } pub fn pick(self, f: impl Fn(u8)) -> Self { self } pub fn shared(self, f: impl Into<Callback<()>>) -> Self { self } pub fn run(self, f: impl Fn()) {} }",
        );
        source.module.clear();
        let surface = crate::public_surface::build(std::slice::from_ref(&source));
        let mut errors = Vec::new();
        super::check(&source, &surface, &mut errors);
        assert_eq!(errors.len(), 5, "{errors:?}");
        assert!(
            errors.iter().all(|error| error.contains("(ARG-1)")),
            "{errors:?}"
        );
    }
```
- 現 consumer の新しい違反：0 件（直し方は不要）。
- state-fields：関係なし（xtask の関数だけ）。行数：`api_shape.rs` は 399 行（G7＋本修正の後）。
- 完了時の確認（D12 G7 の行に加える）：`cargo test -p xtask api_shape`（5 本）。

### 確かめたこと

- G7 の試作差分の上に上のコードを入れ、`cargo test -p xtask`：66 本成功（api_shape は既存 4 本＋新規 1 本。既存 2 本の期待値 6・4 は変わらず成立）。
- `cargo xtask check --no-write --only api-shape`：exit 0、0 件。
- 規則が実際の workspace で効くこと：`crates/kivact-ui/src/calendar/view.rs` に一時的に `pub fn probe_a<F>(self, f: F) -> Self where F: Fn() -> () + 'static` を足すと、`calendar/view.rs:261 ... a builder method takes a handler without a payload as impl Into<Callback<()>> (ARG-1)` で exit 1（G7 の `unit_fn` では通る書式）。戻した。
- `cargo clippy -p xtask --all-targets -- -D warnings` 成功、`cargo fmt -p xtask` 済み。

---

## Q5：3.1：G5 の macro の C-2〜C-4 の回帰試験が無い

### 判定：妥当。ただし「compile-fail は依存無しでは書けない」は不当：rustdoc の `compile_fail` で、新しい依存なしに書ける

D12:96 は追加試験を C-1 の 1 本に限る。HEAD の `component_macro.rs` は callback の既定式の評価時点、属性の保持、拒否構文を固定しない。拒否（compile-fail）は trybuild なしでも、`dev/tests/kivact-public-api` の lib target の doc test の `compile_fail` で書ける（dev-dependency の `kivact` が doctest から使える。同じ手法の前例：`crates/kivact-core/src/draw/offscreen.rs:88`、`crates/arvane/src/engine.rs:16`）。後続に回すのは **error message の文言の検査だけ**（`compile_fail` は「compile できない」ことしか見ない。文言の固定には trybuild 相当の UI 試験基盤が要り、V2 の範囲外）。optional の Value は D12 G5 の `optional_value_props_are_none_or_reactive` が既にあり（`Option<Value<String>>`＝Text と `Option<Value<u32>>`＝Value の両方の kind）、足さない。

### 修正

- 対象：**W5 lane A の G5**。所有ファイルに `dev/tests/kivact-public-api/src/lib.rs` を足す（W5 の他の lane の所有と重ならない：lane B は `calendar/view.rs`・`menu/handle.rs`・`api_shape.rs`、lane C は `e2e/windows.rs`・`e2e/gpu_interop.rs`・`e2e/drawable_export_gpu.rs` ほか）。D12 G5 の修正 7 の「足す試験はこの 1 本だけ」を「この 1 本と、下の 2 本と doc test」にする。
- E2E：`dev/tests/kivact-public-api/tests/e2e/component_macro.rs` の `use kivact::test::render;` の次に `use std::sync::atomic::{AtomicU32, Ordering};` を足し、末尾（D12 G5 の `optional_value_props_are_none_or_reactive` の後）に次を足す：
```rust
/// How many times [`counted_dismiss`] ran: the default expression of
/// `notice`'s `on_dismiss`.
static DISMISS_DEFAULTS: AtomicU32 = AtomicU32::new(0);

/// The default handler of `notice`, counting each run of the expression.
fn counted_dismiss() -> impl Fn() + 'static {
    DISMISS_DEFAULTS.fetch_add(1, Ordering::SeqCst);
    || {}
}

/// A notice with a default dismiss handler.
#[component]
fn notice(
    message: Value<String>,
    #[prop(default = counted_dismiss())] on_dismiss: Callback<()>,
) -> impl IntoView {
    column((text(message), button("Dismiss").on_press(on_dismiss)))
}

#[test]
fn callback_default_runs_when_the_builder_is_called() {
    let (screen, made_by_call) = mount(|| {
        let before = DISMISS_DEFAULTS.load(Ordering::SeqCst);
        let view = notice("Saved");
        (DISMISS_DEFAULTS.load(Ordering::SeqCst) - before, view)
    });
    assert_eq!(made_by_call, 1, "the default runs in the builder call");
    assert_eq!(
        DISMISS_DEFAULTS.load(Ordering::SeqCst),
        1,
        "mounting does not run the default again"
    );
    screen.get_by_text("Saved");
    screen.get_by_role(Role::Button, "Dismiss").click();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A card kept for old callers, with the attributes `#[component]` keeps:
/// `allow` on every generated item, `deprecated` on the builder. The file's
/// `deny(unused_variables)` makes a dropped `allow` a compile error.
#[component]
#[allow(
    unused_variables,
    reason = "the unused name proves the allow reaches the body"
)]
#[deprecated(note = "use `card`")]
fn old_card(title: Value<String>) -> impl IntoView {
    let unused = 0;
    text(title)
}

#[test]
#[deny(unfulfilled_lint_expectations)]
#[expect(deprecated, reason = "#[component] keeps #[deprecated] on the builder")]
fn allow_and_deprecated_reach_the_generated_items() {
    let screen = render(|| old_card("Old"));
    screen.get_by_text("Old");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```
  さらに、同じファイルの先頭の module doc（`//! the call, and a `ViewFn` prop is created again on every opening.` の行）の直後に次の 3 行を足す（inner attribute は最初の item より前に置く）：
```rust
//! Unused variables are errors here: `old_card` proves that `#[component]`
//! keeps its `#[allow(..)]`.
#![deny(unused_variables)]
```
  （inline の `mod { .. }` で囲む形は、xtask の `style/non-leaf-implementation`（「mod を持つ module は宣言と再公開だけ」）がファイル全体を error にするので使わない。試作で実測。）
  仕組み：C-4（既定式は builder を呼んだ時に一度だけ評価する）は `made_by_call == 1` と mount 後も 1 で固定する。C-2 の `allow` が生成物に付かないと、ファイル全体の `#![deny(unused_variables)]` の下で `unused` が compile error になる（既存の試験に未使用の変数は無い）。`deprecated` が builder に付かないと、`#[expect(deprecated)]` が満たされず、`#[deny(unfulfilled_lint_expectations)]` で compile error になる（どちらも E2E の test binary の compile が失敗するので、`cargo test` で必ず気付く）。
- doc test（compile-fail）：`dev/tests/kivact-public-api/src/lib.rs` の全体（今は Role の 1 行だけ）を次にする：
```rust
//! Role: Public API consumer tests. The tests live in tests/ (e2e,
//! offscreen); code that must not compile is checked by the doc tests below.
//!
//! # What `#[component]` rejects
//!
//! A plain function with doc comments and `allow`, `warn`, `deny` and
//! `deprecated` compiles:
//!
//! ```
//! use kivact::prelude::*;
//! /// A card.
//! #[component]
//! #[allow(dead_code, reason = "only compiled")]
//! #[deprecated(note = "use another card")]
//! fn card(title: Value<String>) -> impl IntoView {
//!     text(title)
//! }
//! ```
//!
//! An `async` function does not:
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! async fn card(title: Value<String>) -> impl IntoView {
//!     text(title)
//! }
//! ```
//!
//! A `const` function does not:
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! const fn card(title: Value<String>) -> impl IntoView {
//!     text(title)
//! }
//! ```
//!
//! An `extern` function does not:
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! extern "C" fn card(title: Value<String>) -> impl IntoView {
//!     text(title)
//! }
//! ```
//!
//! `#[expect(..)]` does not (write `#[allow(..)]`):
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! #[expect(dead_code, reason = "only compiled")]
//! fn card(title: Value<String>) -> impl IntoView {
//!     text(title)
//! }
//! ```
//!
//! Any other attribute does not:
//!
//! ```compile_fail
//! use kivact::prelude::*;
//! #[component]
//! #[inline]
//! fn card(title: Value<String>) -> impl IntoView {
//!     text(title)
//! }
//! ```
```
  最初の block（compile する対の例）は、各 `compile_fail` block が拒否以外の理由（typo 等）で失敗していないことの対照。`unsafe fn` は workspace の `unsafe_code = "deny"` でも失敗し得て区別できないので入れない（`async`・`const`・`extern` で C-3 の同じ検査の経路を通る）。
- 完了時の確認（D12 G5 の修正 9 に加える）：`cargo test -p kivact-public-api-tests --doc`、`cargo clippy -p kivact-public-api-tests --all-targets -- -D warnings`。E2E の確認の filter `component_macro` はそのまま（新しい 2 本を含む）。
- state-fields・行数：試験だけ。`component_macro.rs` は約 230 行。

### 確かめたこと

- G5 の試作差分の上で：E2E `component_macro` 8 本（既存 5、D12 の 1、本書の 2）成功、doc test 6 本（compile 1、`compile_fail` 5）成功。
- **退行の検出**（各修正を一つずつ外して実測）：
  - C-4 を HEAD に戻す（`store(prop, quote! { #expr })`）：`callback_default_runs_when_the_builder_is_called` が `left: 0, right: 1`（"the default runs in the builder call"）で失敗。
  - `deprecated` を constructor に付けない：`component_macro.rs:220:10: error: this lint expectation is unfulfilled` で e2e の compile が失敗。
  - G5 の macro 全部を HEAD に戻す：`component_macro.rs:213: error: unused variable: unused`（`allow` が捨てられる）、C-1 の 2 件の compile error、doc test の `compile_fail` 5 本が全部 FAILED（HEAD は async・const・extern・`#[expect]`・`#[inline]` を黙って compile する）。

---

## Q5：全体

### 最終の確認（R06・R08・3.1 と D12 の G5・G7 を全部入れた試作）

- `cargo check --workspace --all-targets --all-features`：error・warning 0（HEAD 由来の `kivact-engine/src/paint/state.rs:326` の dead_code 警告は `--all-features` では出ない）。
- `cargo clippy -p kivact-interaction -p kivact-ui -p kivact-ui-soft -p kivact-ui-creative -p kivact-ui-basic -p kivact-public-api-tests -p xtask -p kivact-macros --all-targets -- -D warnings`：成功。`cargo fmt --all` 済み。
- `cargo xtask check --no-write`（全 rule）：error は HEAD 既存の state-fields 3 件（`arvane-compose/src/cache.rs:53`、`kivact-core/src/view/mount/semantics.rs:14`、`kivact-runtime/src/reactive/data.rs:331`。D12 の記録と同じ）だけ。api-shape・module 規則・行数の新しい error は無い（3.1 の最初の試作の inline `mod` は `style/non-leaf-implementation` で 16 件の error になったので、本書の形（ファイル先頭の `#![deny]`）に直し、0 件を確かめた）。
- `cargo test -p xtask`：66 本成功。`cargo test -p kivact-public-api-tests --doc`：6 本成功。
- 実 window の E2E：上の 122 本＋`interaction_show_hide` 4 本・`component_macro` 8 本の再実行がすべて成功。
- kit の consumer 試験：`cargo test -p kivact-ui-soft -p kivact-ui-creative --test consumer`（実 window）が 44 本・97 本とも成功（`navigation_session`・`top_nav`・`ToolWindowApi`・InlineEdit の呼出し元を含む）。

### 他の担当との結合、wave・lane への影響

- **W5**：lane A（G5）に 3.1 の 2 本の E2E と `dev/tests/kivact-public-api/src/lib.rs`（新しく所有）。lane B（G7）は api_shape.rs のコードを本書 R08 に替える（所有は同じ）。lane C（G6→G8）とは file が重ならない（実測。lane C の e2e は `windows.rs`・`gpu_interop.rs`・`drawable_export_gpu.rs`）。G7 の `menu/handle.rs`（`trigger_action`）は W7 の G9 が同じ file の `open`/`close`/`show` を変えるが、wave が違い順に行うので衝突しない。
- **W7**：G9 の改名表の順 2・3・5 と「変えないもの」の 1 文を本書 R06 の G9-C 表に置き換え、所有ファイルに新規 E2E・`e2e/main.rs`・docs の表の file を足す。W7 は 1 lane（FINAL-1 → G9 → FINAL-2）なので同じ wave の他 lane は無い。FINAL-1（`PartState` の accessor 化）は `kivact-ui/src/combobox/render.rs`・soft の `nav_heading.rs` 等を G9 より先に変えるが、G9 は compile error の示す行だけを変えるので順に行えば重ならない。
- **D12 の G9 の完了時の確認の欠陥**：「`cargo doc -p kivact-interaction --no-deps`（警告 0）」は HEAD で既に満たせない（既存 4 件）。本書の R06「完了時の確認」の判定に置き換える。
- 公開 API の変更の記録（作業指示書 12.3 の箇条）に「開閉の命令の統一（SH-1〜SH-5、G9-C 表）：`hide`・`show`・`show_sheet` への改名、`SelectApi::show`/`hide` の新設、`ComboboxApi::show`・`SelectApi::show_sheet` を冪等に」を足す。
- 他の担当との関係：R07（G8 の adapter 指定）・R05（FINAL-2）とは file も unit の手順も重ならない。

### 人間にしか決められない項目

なし。命令の語（show/hide/toggle、SH-1〜SH-5）は ARG-6（承認済み）の具体化で、`PopupApi::hide(reason)` の引数を残すこと、`ToasterApi::dismiss(id)`・`DockLayout::close*` を開閉の命令から外すことは、理由を上に書いて設計者として決めた。


---

# Q6：3.2（W2 S8）

（元の題：担当 Q6（target lane F）：レビュー 3.2「S8 の cold crate」）

対象：W2 lane C の S8（付属書 D4 3 章「D7」、修正付属書 D11「S8」）。基点 HEAD `6c812296`。試作・計測は detached worktree `W:/w/kv-rf-Q6`、`CARGO_TARGET_DIR=W:/kv-target/F`、`cargo build --release -j 6`（fat LTO・cgu 1）。

## Q6：3.2：S8 の「cold crate」一覧が UI thread の経路（AccessKit・naga・ICU）を含む

### 判定：妥当（AccessKit・naga・ICU／kivact-i18n・svgtypes について）。画像・SVG の decode と raster は不当（worker thread）

#### (1) AccessKit は支援技術（AT）が有効な間、UI thread で毎回の更新に通る（コードで確認）

- 毎回の pump の終わり（描く frame が無い時。描く時は frame の後の pump）に UI thread で `Model::accessibility` を呼ぶ：`crates/kivact-engine/src/host/window_changes.rs:202-211`。
- `crates/kivact-engine/src/a11y/publish.rs:16-66`：`A11yPublish::Inactive`（AT なし）なら即 return。AT がある時は変わった node だけ（D2・AH1 の O(changed)）を `window.update_accessibility`（全体なら `replace_accessibility`）へ渡す。scroll・focus・文字入力（caret・text run の変化）はすべてここを通る。
- `crates/arvane-native/src/port/winit/window.rs:158-175` → `crates/arvane-native/src/a11y/native.rs:153-184`（`update` → `publish` → `hand_over`）→ `accesskit_winit::Adapter::update_if_active(|| shared.pending_with(root, convert_update))`。
- closure の中（UI thread）で `convert_update`（`native.rs:210-225`）→ `conversion::update`（`a11y/conversion.rs:88-112`）と `node`（119-184、`accesskit::Node` の builder を node ごとに）。
- 続けて `accesskit_windows-0.33.1/src/adapter.rs:415-440`：tree の write lock を取り `accesskit_consumer`（0.37）の `Tree::update_and_process_changes` と `AdapterChangeHandler`（UIA の property・text の変化の判定）、`accesskit_winit-0.33.1/src/platform_impl/windows.rs:42-46` で `events.raise()`（UIA の event を UI thread で発行）。
- AT の起動（`request_initial_tree`、`native.rs:31-40`）は `WM_GETOBJECT` の処理中に UI thread で全 tree を変換する。
- AT からの問合せ（読み上げ・移動・text range）：provider は `ProviderOptions_ServerSideProvider` のみ（`accesskit_windows-0.33.1/src/node.rs:1002`、`UseComThreading` なし）なので UIA の thread から直接呼ばれ、`accesskit_consumer`・`accesskit_windows` のコードがそのまま AT の応答時間になり、UI thread の更新とは tree の `RwLock` で直列になる。
- K5 の driver（`arvane-virtual/src/window.rs:370-375`）は AccessKit を通らないので、KPI では退行を検出できない（レビューの指摘どおり）。
- リポジトリ自身も AccessKit を毎 frame の経路として扱っている：root `Cargo.toml` の dev profile（144-149 行のコメント「The engine and Arvane do the work of every frame (... the GPU and accessibility adapters)」、189-194 行で `accesskit`・`accesskit_consumer`・`accesskit_windows` を opt 3）。
- 結論：AccessKit は「起動と毎 frame の経路に無い」に当たらない。`z` は AT 利用者の scroll・focus・文字入力の応答と読み上げの応答を遅くし得る（G-03・P-04）。AT が無い利用者には影響しない（`Inactive` で return、`update_if_active` も何もしない）。

#### (3) 画像・SVG・naga・その他の一覧の crate

| crate | 実行される thread（根拠） | 判定 |
|---|---|---|
| 画像の decode（`image`・`zune-jpeg`・`zune-core`・`image-webp`・`png`・`gif`・`fdeflate`・`miniz_oxide`・`flate2`） | 実 App は `AssetMode::Worker`（`kivact-engine/src/host/launch.rs:126-129`）→ `AssetStore::new`（`arvane-asset/src/store/asset_store.rs:88-91`）の worker thread `arvane-asset`（`store/worker.rs:95-110`）。decode の入口は `store/job.rs:195` だけ。inline の store は試験の driver（`#[cfg(feature = "testing")]`）と `OffscreenRenderer`（`kivact-engine/src/offscreen.rs:71`。App の外の画像書出しと、明示の `NodeRef::snapshot`（`paint/snapshot.rs:50`）。どちらも呼び手が同期の CPU raster 全体を待つ操作） | worker の 1 回の decode（結果は cache）。`z` で残す |
| SVG の解析・描画（`usvg`・`resvg`・`tiny-skia`・`tiny-skia-path`・`rustybuzz`・`ttf-parser`・`fontdb`・`roxmltree`・`simplecss`） | 同じ worker の `Job::Load`（解析）と `Job::Raster`（device size ごとの raster）。SVG の font DB も worker で最初の text 付き SVG の時に作る（`arvane-asset/src/svg/fonts.rs:7-8,37-53`）。`tiny_skia` を使うのは `svg/document.rs` だけ（CPU の描画は vello_cpu） | worker。`z` で残す |
| `svgtypes` | usvg（worker）に加え、公開 API `Path::parse_svg`（`kivact-style/src/path/commands.rs:111`）が UI thread で path data を解析する（canvas の draw の中で呼べる） | UI thread。**外す** |
| `naga` | `PaintShader::wgsl`・`FilterShader::wgsl`（`kivact-style/src/shader/{paint_shader,filter}.rs:25`）→ `kivact-shader` の `assemble`（`assemble.rs:115-`、WGSL の parse と validate）を UI thread で、component の構築の度に実行（例 `examples/kivact-ui-game/src/game/effects.rs:285` は mount ごと）。GPU では組込みの pipeline は device の worker で作る（`arvane-compose/src/gpu/context.rs:89-90`）が、user shader の pipeline は最初に描く GPU frame で window の thread が作る（`gpu/shader.rs:185-193` → `shader_cache.rs:132-161` の `create_shader_module`。wgpu の naga が WGSL の解析・検証・HLSL/SPIR-V の出力をする） | UI thread を止める。**外す** |
| ICU（`icu_datetime`・`icu_calendar`・`icu_decimal`・`icu_plurals`・`icu_collator`・`icu_time`・`icu_locale_core`）・`kivact-i18n` | 書式・照合は view の値の計算で UI thread。`kivact-i18n` は書式だけでなく `Messages` の検索と `interpolate`（`kivact-core/src/view/boundary.rs:22,180-290`）も持ち、翻訳する App の全 text の mount と locale の切替で UI thread を通る（D4「kivact-i18n は日時・数・複数形の書式だけ」は誤り）。D4 の実測でも Kivact の crate を `z` にすると K5 が 4〜20 % 悪化した | UI thread。**外す** |
| `rfd`・`arboard` | file dialog・clipboard。時間は OS の dialog・clipboard の呼出しが支配し、利用者の明示の操作で 1 回 | `z` で残す |
| `accesskit`・`accesskit_consumer`・`accesskit_windows`・`accesskit_winit` | 上の (1) | **外す** |

#### (2) K6 の実測（byte、Windows 11、Rust 1.98.1、この機械）

worktree には S6 の `dev/bench/Cargo.toml`（D11「S6」の内容。nogpu・no-default の構成を GPU 抜きにする）を当てた。「S7」は D11「S7」のコードを worktree に当てたもの（`cargo check -p kivact-core` 成功）。S9（font の subset、−8,720,328 B＝9,589,900−869,572。D11「S9」の実測）は catalog の行に差し引きで当てた。

| 構成 | kv_startup no-default（≤ 11,177,658） | counter の GPU 抜き＝kv_startup `--features complex-scripts`（≤ 14,467,313） | catalog の GPU 抜き（≤ 31,912,659） |
|---|---:|---:|---:|
| D11 の一覧（34 節、AccessKit・naga・ICU を含む）、S7 なし | 11,149,312（余裕 28,346） | 14,203,904 | — |
| 同、AccessKit の 4 つだけ外す、S7 なし | **11,184,128（6,470 超過）** | 14,239,232（+35,328） | — |
| D11 の一覧 ＋ S7 | — | — | 37,156,352 |
| D11 の一覧から AccessKit の 4 つを外す ＋ S7 | — | — | 37,204,992（+48,640） |
| **確定の一覧**（AccessKit・svgtypes・naga・ICU・kivact-i18n を外す）＋ S7 | **11,165,696（余裕 11,962）** | **14,220,288（余裕 247,025）** | **37,217,792（+12,800）→ S9 後 28,497,464（余裕 3,415,195）** |
| 確定の一覧 ＋ decoder の補助 crate 6 つ（下）を `z` ＋ S7 | 11,166,208（+512） | 14,221,312（+1,024） | 37,216,768（−1,024） |
| 参考：`gpu`・`complex-scripts` 込みの既定 kv_startup（MS-5 (2)、≤ B1-start。この機械の B1 は 18,976,768（D4 1.1）） | — | 18,800,640（確定の一覧＋補助 6 つ＋S7。B1 より −176,128） | — |

- AccessKit を `z` にする効果は kv_startup で 34,816〜35,328 B、catalog で 48,640 B。svgtypes・naga・ICU・kivact-i18n を `z` にする効果は catalog で 12,800 B、kv_startup ではほぼ 0。
- no-default の行（`naga`・`icu_datetime` 等は link されない。下の PDB の内訳）では、naga・ICU・kivact-i18n・svgtypes を外す費用はほぼ 0。費用は AccessKit の 35 KB だけで、HEAD の余裕 28 KB を 6.5 KB 上回る。同じ wave（W2 lane E）の S7 が −18〜−20 KB（D4 の見積り −50 KB より小さい）で、合わせて目標内（余裕 11,962 B）。
- 補う手段として、worker でだけ動く decoder の依存 6 つ（`imagesize`・`xmlwriter`・`data-url`（usvg のみが使う）、`weezl`（gif）、`crc32fast`（png・flate2）、`simd-adler32`（fdeflate・miniz_oxide）。`cargo tree -i` で bench・catalog の両方で他の利用者が無いことを確認）も `z` にしてみたが、+512／+1,024／−1,024 B と効果が無い（section の align の揺れ）。一覧を長くするだけなので採らない。
- `gpu` 込みの既定（MS-5 (2)）は B1 より 176 KB 小さく、「V2 の前より大きくしない」を満たす（AccessKit・naga を外しても十分）。
- 他の候補（PDB の内訳、下）：残りの第三者の crate は hot（vello_cpu・skrifa・harfrust・taffy・winit・fontique・icu_segmenter・unicode_bidi 等）か、generic の instantiation（std の slice・ptr・collections）で、opt-level で性能を落とさずに削れるものは無い。補いは S7 だけで足りる。
- no-default の行は余裕が 11,962 B と小さい。これは S8 固有でない（HEAD で既に余裕 28 KB、D4 の時点の見積り 0.17 MB から W1 の変更で約 86 KB 増えている）。

PDB の内訳（`CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=line-tables-only` で同じ大きさ（+1,024 B）の exe と PDB を作り、`llvm-pdbutil dump --symbols` の `S_*PROC32` の code size を名前の最初の非 std の crate に帰属。AccessKit を外した no-default、計 8,942,657 B）の主な値：kivact_engine 1,323,686、kivact_core 432,599、skrifa 378,500、harfrust 331,306、usvg 272,766、kivact_runtime 271,845、fearless_simd 269,624、vello_cpu 251,315、zune_jpeg 169,969、tiny_skia 169,520、rustybuzz 148,227、image_webp 130,610、winit 96,680、accesskit_consumer 69,205、fontique 49,039、accesskit_windows 36,862、svgtypes 21,332、icu_segmenter 14,789、imagesize 13,626、kivact_i18n 41。`naga`・`icu_datetime`・`icu_decimal` 等は 0（link されない）。

### 修正

#### 対象 unit・wave・lane・所有ファイル

- S8（W2 lane C、S45 → T4 → **S8** → S9 の 3 番目）。所有ファイルは今と同じ（root `Cargo.toml` の profile 節、`templates/basic/Cargo.toml` の profile 節）。追加・変更なし。
- 他の lane とファイルは重ならない（W2 で `Cargo.toml` の profile 節を持つ unit は S8 だけ）。署名の変更は無い。
- S7（W2 lane E）は今のまま行う（no-default の行の余裕は S7 を前提にする。下の「全体」）。

#### D11「S8」の「### 修正」を次で置き換える（Haiku はこのとおり行う）

root `Cargo.toml` の最後の `[profile.release.package.arvane-text]` `opt-level = 3` の後と、`templates/basic/Cargo.toml` の同じ位置（最後の `[profile.release.package.arvane-text]` の `opt-level = 3` の後）に、空行 1 行と次の 47 行をそのまま足す（両方の file で一字一句同じ。並べ替えない・足さない）：

```toml
# Third-party crates that run only on the asset worker thread (SVG parsing
# and rasterizing, image decoding) or wait on the OS (file dialogs, the
# clipboard) favour size. Crates on the UI thread keep their level, even
# when they look cold: the accessibility adapter (AccessKit, every update
# while assistive technology is active), shader validation and translation
# (naga), date, number and message formatting (ICU, kivact-i18n) and SVG
# path data (svgtypes, `Path::parse_svg`).
[profile.release.package.usvg]
opt-level = "z"
[profile.release.package.resvg]
opt-level = "z"
[profile.release.package.tiny-skia]
opt-level = "z"
[profile.release.package.tiny-skia-path]
opt-level = "z"
[profile.release.package.rustybuzz]
opt-level = "z"
[profile.release.package.ttf-parser]
opt-level = "z"
[profile.release.package.fontdb]
opt-level = "z"
[profile.release.package.roxmltree]
opt-level = "z"
[profile.release.package.simplecss]
opt-level = "z"
[profile.release.package.image]
opt-level = "z"
[profile.release.package.zune-jpeg]
opt-level = "z"
[profile.release.package.zune-core]
opt-level = "z"
[profile.release.package.image-webp]
opt-level = "z"
[profile.release.package.png]
opt-level = "z"
[profile.release.package.gif]
opt-level = "z"
[profile.release.package.fdeflate]
opt-level = "z"
[profile.release.package.miniz_oxide]
opt-level = "z"
[profile.release.package.flate2]
opt-level = "z"
[profile.release.package.rfd]
opt-level = "z"
[profile.release.package.arboard]
opt-level = "z"
```

（全 20 節。D11 の 34 節から `svgtypes`・`accesskit`・`accesskit_consumer`・`accesskit_windows`・`accesskit_winit`・`naga`・`icu_datetime`・`icu_calendar`・`icu_decimal`・`icu_plurals`・`icu_collator`・`icu_time`・`icu_locale_core`・`kivact-i18n` の 14 節を除いたもの。`png`・`miniz_oxide`・`roxmltree` は `Cargo.lock` に 2 版あるが、版なしの指定は両方に効き警告は出ない。）

確認（今の D11 の確認と同じ）：
1. `cargo metadata --format-version 1 > /dev/null` が成功する。
2. `cargo check --release -p kivact-bench --bin kv_startup 2>&1 | grep -E "did not match|ambiguous"` が何も出さない。
3. `grep -c 'opt-level = "z"' Cargo.toml` が 20、`grep -c 'opt-level = "z"' templates/basic/Cargo.toml` が 20。
4. `grep -nE 'package\.(accesskit|naga|icu_|kivact-i18n|svgtypes)' Cargo.toml templates/basic/Cargo.toml` の出力に `profile.release` の行が無い（`profile.dev.package.accesskit*` の 3 節は既存のまま残す）。

state-fields・試験：対象外（profile だけ。コードと型を変えない）。新しい試験は足さない（性能は CP の `run.sh` の K1・K4・K5、大きさは K6）。

#### D4 3 章「D7」の本文の読み替え（D11 の S8 の節の末尾に 1 行足す）

「D4『D7』の一覧・コメント・『性能への影響』の段落は本節の一覧が置き換える。AccessKit は AT が有効な間 UI thread で毎回の更新（scroll・focus・文字入力）に通り、naga・ICU・kivact-i18n・svgtypes も UI thread で動くので `z` にしない（rf-Q6）。」

### 確かめたこと（試作・試験の結果）

- 上の表の 13 本の release build と PDB 付きの 1 本（同じ worktree・同じ target、fat LTO）。13 本の build の log のどれにも `did not match`・`ambiguous` の警告なし。
- 確定の一覧 ＋ S7 で：kv_startup no-default 11,165,696 B（目標 11,177,658、余裕 11,962）、counter の GPU 抜き 14,220,288 B（目標 14,467,313、余裕 247,025）、catalog の GPU 抜き 37,217,792 B（S9 の subset の −8,720,328 を引いて 28,497,464、目標 31,912,659、余裕 3,415,195）。3 行とも満たす。
- AccessKit を外した費用：35 KB（no-default で +34,816、GPU 抜きで +35,328）。HEAD のまま（S7 なし）では no-default が 6,470 B 超過、S7 で解消。
- 速度（AT 有効時の応答、`z` と `s` の差）は計測していない。外す側に倒したので計測は不要（`s` は HEAD と同じ）。
- worktree の変更（S6・S7・profile）はリポジトリに戻していない。worktree は削除した。

## Q6：全体

- **他の担当との結合**：no-default の行は S7（W2 lane E、D11「S7」）の −18〜−20 KB を前提にして目標内に入る。S7 を取りやめる・縮める担当があれば、この行は 6,470 B 超過になる（S7 は性能に中立なので取りやめる理由は無い）。S6（W3 lane D）が当たるまで no-default・GPU 抜きの行は GPU 込みで測られる（D11「K6 の nogpu の build」のとおり。CP1 は W4 の後）。
- **wave・lane**：変更なし（S8 は W2 lane C の 3 番目のまま、所有ファイルも同じ）。
- **CP での扱い**：no-default の行は余裕が 12 KB と小さい。CP1・CP2 で未達なら作業指示書 7.3／11.3 に従い、AccessKit・naga・ICU・kivact-i18n・svgtypes を `z` に戻すことでは補わない（支援技術の利用者と UI thread の応答を落とす。G-03・P-04・P-05）。大きくした unit を、上の PDB の内訳の方法（`CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=line-tables-only` で build → `llvm-pdbutil dump --symbols` の code size を crate ごとに集計し、B1 の内訳と比べる）で特定して `OPEN.md` に書く。
- **人間にしか決められない項目**：なし。
