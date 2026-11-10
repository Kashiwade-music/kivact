# V2 再監査 R1（target lane A）：F5 の差戻し、AH1・AH2・AH3・AH4・F7・T7・S3

> **V2.1 修正付属書 D9**（再監査 R1 の成果物、HEAD `2a3b06b8`＝W1 の後に対して書いた）。範囲：W1 の F5 の返戻し、W2〜W4 の AH1・AH2・AH3・AH4・F7・T7・S3（engine・a11y・hit）。
> - **優先順位**：本書は付属書 D1〜D8 より優先する。本書の unit の節の「修正」は、作業指示書の行の「本書の追加の指示」と同じ扱い。本書と V2 設計書 4 章（DEC）が食い違う場合は V2 設計書が正（V2 設計書 4 章の DEC-18 の名前は `while_open(factory)`）。
> - 本書の「全体」節の wave・lane の組み直しは、作業指示書（V2.1）に反映済み。実施順・所有ファイルは作業指示書が正。
> - 文中の worktree（`W:/w/kv-audit-*`）・scratchpad の path は監査の試作の場所で、実装に必要ではない（試作の差分は参考で、実装は本書の文に従う）。

HEAD `2a3b06b8`。静的解析に加え、detached worktree（`W:/w/kv-audit-R1`、target `W:/kv-target/A`、監査の後に削除）で **F5 の修正・AH1・AH2・AH3・T7・S3 を本書の修正どおりに試作し、F7・AH4・S3 の E2E を本書のコードどおりに書いて実行した**。結果は各節の「試作での確認」に書く。本書のコード片はその試作で compile・clippy（`-D warnings`）・state-fields を通したものである。E2E の file 全体のコードは `cargo fmt` 済み。差分の断片は実装者が最後に `cargo fmt` を掛ける（試作の `cargo xtask check --no-write --crate kivact-engine` の error は fmt 前の書式だけで、module 規則・行数・state-fields の error は 0）。

共通の確認（lint の規則、`dev/xtask/src/state_fields/`）：数えるのは `Option`（callback 以外）・`bool`・atomic・`Cell<bool|Option|整数>`・名前付き counter（`generation`・`next_*`・`*_count`・`pending_*` 等）・`pending*/dirty*/awaiting*/relayout*/stale*` で始まる名前の集合、と phase enum（名前が `State/Phase/Status/Stage/Connection/Lifecycle/Run/Present` で終わる workspace の enum）。`Sent`・`Generation` 等の helper は他の stored がある時だけ数える。tuple の variant は数えない。W7 の X3（FINAL-2）後は「数える field が 2 つ以上あり、そのうち 1 つでも無印」で error になるので、本書の新しい型は **数える field を 1 つ以下**にしてある。関数の行数の上限は clippy `too_many_lines`（80、`clippy.toml`）、引数は clippy の既定（7）。

---

## F5-fix（W1 の差戻し：付属書 D1 3.6 S-6、`arvane-compose/src/cache.rs:53` `Indexed`）

### 判定：修正あり（型で閉じる。marker は使わない）

`Indexed { bounds: Option<PixelRect>, group: Option<PixelRect> }` は「content 命令は `bounds` だけ、group opener は `group` だけ、範囲外は両方 None」という排他を 2 つの Option で表した暗黙の状態。両方 Some は到達しない（`plan/damage.rs` の `indexed()` が唯一の構築）。enum にする。等価（`hash` と「描く矩形」）と `damage()`（`bounds.or(group)`）の意味は今と同一。

### 修正（作業指示書の新しい行「F5-fix」の「本書の追加の指示」にそのまま入れる）

1. `crates/arvane-compose/src/cache.rs`：`/// One command of a chunk's index. A group opener ...` の doc から `impl Indexed { .. damage .. }` の終わりまで（`struct Indexed`、`impl PartialEq for Indexed`、`impl Eq for Indexed`、`impl Indexed`）を、次で丸ごと置き換える。

```rust
/// Where one command of a chunk's index reaches, in chunk px.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Reach {
    /// Nothing of it lands in the chunk.
    Nothing,
    /// A command that draws: it damages this rectangle when it changes or
    /// moves.
    Draws(PixelRect),
    /// A group opener (a clip, a transform, a group without blur): it draws
    /// nothing of its own and damages its group's bounds only when it
    /// changed itself (its hash), never because its content's extent moved
    /// (each content command damages its own place).
    Opens(PixelRect),
}

/// One command of a chunk's index: its hash and where it reaches.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Indexed {
    pub(crate) hash: u64,
    pub(crate) reach: Reach,
}

impl Indexed {
    /// The rectangle this command draws in (`None` for an opener or a
    /// command outside the chunk): with the hash, what makes two entries
    /// the same.
    fn drawn(&self) -> Option<PixelRect> {
        match self.reach {
            Reach::Draws(rect) => Some(rect),
            Reach::Opens(_) | Reach::Nothing => None,
        }
    }

    /// What a change of this command damages.
    pub(crate) fn damage(&self) -> Option<PixelRect> {
        match self.reach {
            Reach::Draws(rect) | Reach::Opens(rect) => Some(rect),
            Reach::Nothing => None,
        }
    }
}

impl PartialEq for Indexed {
    fn eq(&self, o: &Self) -> bool {
        self.hash == o.hash && self.drawn() == o.drawn()
    }
}

impl Eq for Indexed {}
```

2. `crates/arvane-compose/src/plan/damage.rs`：`use crate::cache::{ChunkKey, ChunkRecord, Indexed, LayerRecord, Retained};` を `use crate::cache::{ChunkKey, ChunkRecord, Indexed, LayerRecord, Reach, Retained};` に。`fn indexed(..)` の本体を次に置き換える（署名は不変）。

```rust
fn indexed(command: &RenderCmd, hash: u64, clipped: Option<PixelRect>) -> Indexed {
    let reach = match (command, clipped) {
        (_, None) => Reach::Nothing,
        (RenderCmd::PushClip(_) | RenderCmd::PushTransform(_), Some(rect)) => Reach::Opens(rect),
        (RenderCmd::PushGroup(g), Some(rect)) if g.blur.is_none() => Reach::Opens(rect),
        (_, Some(rect)) => Reach::Draws(rect),
    };
    Indexed { hash, reach }
}
```

   他の読み手（`diff` の `Indexed::damage`、`prev.index[..] == index[..]`）は変えない。

3. 完了時の確認：`cargo check -p arvane-compose --all-features`、`cargo clippy -p arvane-compose --all-targets --all-features -- -D warnings`、`cargo test -p arvane-compose --lib`、`cargo xtask check --no-write --only state-fields --crate arvane-compose`（error 0）。

### 所有ファイルの変更
`crates/arvane-compose/src/cache.rs`、`crates/arvane-compose/src/plan/damage.rs`（F5 と同じ）。

### wave・lane・順序の変更
新しい unit「F5-fix」を **W2 lane B の先頭**（S4 の cherry-pick・S5・F7 の前）に置く。W2〜W5 で arvane-compose のこの 2 ファイルを触る unit は無い（W6 の P8-ARV は `cpu/compositor.rs`）。F7 の `incremental_frames_match_a_fresh_render` がこの damage の正しさを画素で確かめるので、F7 より前に置く。

### 試作での確認
上の 2 つの置換だけで `cargo clippy -p arvane-compose --all-targets -- -D warnings` 成功、`state-fields --crate arvane-compose` exit 0（`cache.rs:53` の error が消える）、`cargo test -p arvane-compose --lib` 成功。F7 の `incremental_frames_match_a_fresh_render` と compositor の既存 E2E 全件が、この修正の上で成功。

---

## AH1（付属書 D2 3.1〜3.5、3.8、W2 lane A）

### 判定：修正あり（設計は付属書のまま。欠けているコード・誤った位置・所有ファイルの漏れを補う）

付属書のコードは HEAD（F1・F3・F4・M2 の後）の関数・型・field と全て照合した：`A11yState`・`Dirt`（HEAD は `LocalText`・`Geometry` を持つ）・`A11yPublish`（HEAD は `Published { root }`、`publish` は `pub(crate)` で `host/events.rs` だけが読み書き）・`Built`（HEAD の `metadata: Option<Metadata>`）・`build_semantics_for`・`build_semantics_tail`・`build_identity_metadata`・`build_dirty_nodes`・`build_node(id, offset, clipped, out, regions, engine)`・`semantics_focus(&BTreeSet)`・`scroll_above`・`semantic_children`・`list_rows`・`is_clipping_list`・`NodeTree::{depth, overlays, fixed, is_in_subtree, chain, parent_id}`・`Model::{repaint, repaint_text, scroll_to, changed, invalidate_semantics, invalidate_inherited_text, set_scene}`・`Mutation`・typed.rs の props 表・`change_focus` の `old_target`/`target`（`Option<FocusTarget>`）・`set_scroll`・`sync.rs` の font の loop と `poll_assets`。F1 が足した `index_sources`（sync.rs `derive` の末尾）・`forget_sources`（apply.rs）・`Model.sources` は AH1 の編集箇所と重ならず、付属書どおりに書けば消えない。M2・F3 は AH1 のファイルに触れていない。F4 は `model/interact/geometry.rs` の `layout()` に `moved_child` を足しただけで、AH1 が触る `set_scroll` とは別の関数。

新しい型の state-fields：`Clipping`（`BTreeMap`・`rows: BTreeSet`、数える field 0）、`Payloads`（`BTreeSet` 3 つ。名前が `pending`/`dirty` 等で始まらないので 0）、`Dirt::Metadata`（0）、`A11yPublish::Published { root, dirt }`（`Dirt` は phase の名前でない、0）、`A11yState`（`constructed`・`published`・`enumerated` は名前付き counter に当たらない、0）、`Metadata`（`modal: Option` の 1 つだけ）、`Built`・`MetadataUpdate`（0）。X3 の後も error にならない。`Clipping.rows` の doc の「Projection: …」は文で marker ではない（lint は `BTreeSet rows` を数えないので marker は書かない）。

### 修正（「本書の追加の指示」に入れる。付属書の各コードに加えて、次を行う）

1. **`a11y/state.rs` の完成形**。付属書 3.1 のコードの「今と同じ」「今のまま」を次の具体形にする（付属書のコード中の日本語の注釈 `// 旧 …` は書かない）。
   - `impl Default for A11yState` は HEAD のものから `dirt: Dirt::Full,` の行を消し、`published: 0,` の後に `#[cfg(feature = "testing")]` と `enumerated: 0,` を足す。
   - `node`・`subtree`・`request_publication` の本体は HEAD と同じ文：
     ```rust
     pub(crate) fn node(&mut self, id: NodeId) {
         if let Some((nodes, _, _)) = self.metadata_dirt() {
             nodes.insert(id);
         }
     }
     pub(crate) fn subtree(&mut self, id: NodeId) {
         if let Some((_, subtrees, _)) = self.metadata_dirt() {
             subtrees.insert(id);
         }
     }
     pub(crate) fn request_publication(&mut self, removed: impl Iterator<Item = NodeId>) {
         if let Some((_, _, gone)) = self.metadata_dirt() {
             gone.extend(removed);
         }
     }
     ```
   - `region_of`・`region_id`（`#[cfg(feature = "testing")]`）と `SemanticsIdAllocator`（marker 2 行を含む）は HEAD の文のまま残す。旧 method `local_text` は削除する。
2. **`a11y/publish.rs`**：`use super::build::Built;` → `use super::build::{Built, MetadataUpdate};`。`accessibility` の doc の最後の文を「IDs can be allocated before acceptance, but hashes, the identity metadata (or a payload build's new clipping) and the clean dirt commit only on acceptance.」にする。本体は付属書 3.2 のとおり。
3. **`a11y/build.rs`**：`pub(super) use dirty::Built;` → `pub(super) use dirty::{Built, MetadataUpdate};`。`mod local_text;` を消し、`mod node;` の次の行に `mod payload;`。`a11y/build/local_text.rs` を削除。
4. **`a11y/build/dirty.rs`**（付属書 3.3 の箇条を次で正確にする）：
   - import を `use crate::a11y::state::{Clipping, Dirt, Metadata, Payloads};` にし、`use std::rc::Rc;` を消す（`build_geometry_semantics` を消すと未使用になり clippy `-D warnings` で落ちる）。
   - `struct Built` の doc を「A semantics tree built for publication and what it changes in the accepted identity metadata.」にし、その前に付属書の `enum MetadataUpdate` を置く。
   - `build_semantics_for`：HEAD の `if let Dirt::LocalText ..`・`if let Dirt::Geometry ..`・`let empty = BTreeSet::new();`・旧 `let (own, subtrees, removed) = match dirt { .. }` の 4 つをまとめて次に置き換える（`let full = ..` と `let Some(root) = .. else { .. }` はその上に残す）：
     ```rust
     let empty = BTreeSet::new();
     let (own, subtrees, removed) = match dirt {
         Dirt::Clean => return self.build_payloads(root, &Payloads::default(), engine),
         Dirt::Payload(payloads) => return self.build_payloads(root, payloads, engine),
         Dirt::Metadata { nodes, subtrees, removed } => (nodes, subtrees, removed),
         Dirt::Full => (&empty, &empty, &empty),
     };
     ```
   - `let (metadata, order) = self.build_identity_metadata();` の直後に `#[cfg(feature = "testing")] { self.a11y.enumerated += order.len() as u64; }`。
   - `.clipped` の読み手は **3 か所**（付属書の「tail の 2 か所」は誤り）：(a) `build_semantics_for` の `metadata.clipped.symmetric_difference(&self.a11y.metadata.clipped)` → `metadata.clipping.rows().symmetric_difference(self.a11y.metadata.clipping.rows())`、(b) `build_semantics_tail` の `self.semantics_focus(&metadata.clipped)` → `self.semantics_focus(metadata.clipping.rows())`、(c) `build_dirty_nodes` の `self.build_node(.., &metadata.clipped, ..)` → `metadata.clipping.rows()`。
   - `build_semantics_for` と `build_empty_semantics` の `metadata: Some(..)` → `metadata: MetadataUpdate::Replace(..)`。
   - `build_identity_metadata`：`let clipped = self.clipped_rows();` → `let clipping = self.clipping_of_all();`、`Metadata { clipped, ..` → `Metadata { clipping, ..`。この関数の直前に doc comment（作業指示書 W2 AH1 の指示、DEC-06）を置く：
     ```rust
     /// Enumerate the identity and order metadata of the whole tree. A PERF-1
     /// exception (DEC-06): a structural change is checked against the
     /// accepted metadata by this complete comparison, so the published
     /// hierarchy stays right without trusting each writer's notice. It never
     /// runs for the K5 updates, pointer moves or wheels (payload-only dirt).
     ```
   - `fn expand_relation_dirt` を `pub(super) fn expand_relation_dirt` に。`build_geometry_semantics` を削除。
5. **`a11y/build/node.rs`**：付属書の `clipping_of_all` は型名 `Clipping` を使うので、file 先頭の `use` に `use crate::a11y::state::Clipping;` を足す。`clipped_rows` を付属書の `clipping_of_all` と `is_clipping_list_node` に置き換え、`fn list_rows(` を `pub(super) fn list_rows(` に。
   **さらに（付属書に無い設計の補正、必須）**：`build_node` の先頭の
   ```rust
   let rect = self.semantic_rect(id).unwrap_or_default();
   let bounds = types::Rect {
       x: rect.x + offset.x,
       y: rect.y + offset.y,
       ..rect
   };
   ```
   を次に置き換える：
   ```rust
   // A node without a box (not laid out, hidden, or a boxless group
   // with nothing shown) has empty bounds at the origin: like every
   // other content-space bounds they follow no scroll above it.
   let bounds = self
       .semantic_rect(id)
       .map_or_else(types::Rect::default, |rect| types::Rect {
           x: rect.x + offset.x,
           y: rect.y + offset.y,
           ..rect
       });
   ```
   理由：付属書 1.5 の前提「bounds は content 空間なので祖先の scroll で子孫の payload は変わらない」は、box を持つ node にしか成り立たない。HEAD は rect が無い node（`display: none` の node、何も表示しない `display: contents` の group）の bounds を「(0,0) + 祖先の scroll の和」にしているので、祖先が scroll すると bounds が変わる。AH1 の `a11y_scrolled` はその子孫を作り直さないので、試作では E2E `ui_chat_code::chat_thread_is_a_log_and_follows_the_end`（hidden の node の y が 360 と 180）と `ui_components::menu_renders_nested_levels_and_custom_trigger_keeps_its_style`（menu の空の group 14 個の y が 0 と 2）が self_check の a11y の不一致で失敗した。rect の無い node は bounds を原点の空の矩形にすれば scroll に依らなくなり、payload の経路のままで正しい（参照の Full 構築も同じ関数を使うので一致する）。この補正の後、E2E 全 635 本のうち失敗は HEAD で既に失敗している 1 本（「全体」節）だけ。公開される bounds が変わるのは box の無い node だけで、これを期待値に持つ既存試験は無い（全件で確認）。
6. **`a11y/build/payload.rs`**（新）：付属書 3.4 のコードのまま（試作で clippy の `too_many_lines` を通る。`build_payloads` は 75 行。`use` も過不足なし）。
7. **`a11y/build/geometry.rs`**：付属書 3.4 末尾の `invalidate_boxless_ancestors` のまま。
8. **`a11y.rs`**（**所有ファイルに追加**。module doc に旧 `local text` の記述が残るため）：Invariants の 5 番目の箇条（「Changes record one `Dirt` level …」で始まる 7 行）を次に置き換える：
   ```text
   //! - Changes record one `Dirt` level while a tree is published (clean,
   //!   payload-only with its nodes, moved subtrees and clip sources,
   //!   metadata with its own-node, subtree and removed sets, or full);
   //!   before the first tree and while no assistive technology reads the
   //!   window nothing is pending (`A11yPublish`). A payload publication
   //!   builds the dirty payloads over the accepted identity metadata; a
   //!   metadata publication enumerates identity/order metadata first. Both
   //!   compare complete fingerprints, and the build returns what it changes
   //!   in the metadata, which only window acceptance commits (with the
   //!   hashes). No accepted payload tree is retained in the engine. A
   //!   rejected update requests a complete resynchronization (`Dirt::Full`).
   ```
9. **model 側の書き手**：付属書 3.5 のとおり。HEAD での場所（関数名で探す）：`aggregate.rs` `repaint_text`・`repaint`（新設 `a11y_scrolled` は `repaint` の直後）、`apply.rs` `set_scene` の canvas drawable root の loop（`self.a11y.subtree(root)` → `moved`）、`scroll_to`（付属書の置換形）、`changed` の末尾の states_place（`subtree` → `moved`）、`invalidate_semantics`（付属書の置換形）、`invalidate_inherited_text` の 2 か所（`node` → `payload`）、`states.rs` `restyled`、`interact/sticky.rs` `stick`、`state_motion/steps.rs` の 1 か所、`layout_motion/steps.rs` `step_layout_motion`、`drawables/request.rs` の `SetDrawableGeometry`、`focus/navigation.rs` `change_focus` の **2 つ目**の `if old_target != target {`（`request_publication` の方。1 つ目の `if old_target != target && let Some(old) = ..` は変えない）、`interact/sync.rs` `sync` の font の loop と `poll_assets`、`edit/inputs.rs`、`mutation.rs`、`host/write/typed.rs` の 7 行、`host/events.rs` `accessibility_event`。
10. **`model/interact/geometry.rs` の `set_scroll` の 1 行を AH1 が行う**（AH3 から移す）：`self.a11y.moved(id);` → `self.a11y_scrolled(id);`。理由：AH1 だけで scroll の a11y が設計どおりになり、W2 の後に半端な状態（`scroll_to` は payload、wheel は subtree 全体）を残さない。W2 の他の lane はこのファイルを触らない。
11. `mutation.rs` の `payload_only` の上の `// xtask-allow(state-fields): independent: …` 行は文言を変えずに残す（付属書のとおり）。`WriteStats.a11y_enumerated` は AH1 が足す（`Writer::stats`・`Reader::stats` は `..stats` で作るので AH2 の前でも compile する）。
12. 完了時の確認の E2E の filter を `hover pointer focus layout_scroll overlay relations apg_listbox flows touch_scroll text_editing interaction_tooltip apg_tooltip transform drawables view:: image_semantics` にする（self_check の a11y の比較が全 settle で走る範囲を広げる。STOP 条件は作業指示書 0.4 のまま）。

### 所有ファイルの変更
付属書 P2-U1a の一覧に `crates/kivact-engine/src/a11y.rs`（doc だけ）と `crates/kivact-engine/src/model/interact/geometry.rs`（`set_scroll` の 1 行だけ）を足す。

### wave・lane・順序の変更
なし（W2 lane A の単独 unit）。

### 試作での確認
1〜11 を行った worktree で：`cargo check -p kivact-engine`（features なし・`testing`）、`cargo clippy -p kivact-engine --all-targets --features testing -- -D warnings` 成功、`state-fields --crate kivact-engine` exit 0。12 の filter の E2E 175 本が成功（self_check の a11y の不一致 0）。E2E の全件（635 本）では 5 の補正が無いと 2 本が失敗し、補正の後は HEAD で既に失敗している 1 本を除き全て成功（AH3・T7・AH2・S3 を重ねた状態、「全体」節）。

---

## AH2（付属書 D2 3.5「診断の counter の配線」、W3 lane B）

### 判定：修正あり（文言の確定だけ）

付属書どおりで compile する（`kivact_core::diagnostics::Stats` の構築は `launch/inspect.rs` の 1 か所だけ、`WriteStats` の構築は `..stats` 付きの 2 か所だけ。workspace を grep 済み）。W2 の T1 が `host/write/reader.rs` を変えるが、AH2 は W3 なので順序どおり。

### 修正
- `host/write/reader.rs`・`host/write/writer.rs` の `#[cfg(feature = "testing")] let stats = WriteStats { .. }` の `a11y_published: ..,` の次の行に：
  ```rust
  a11y_enumerated: app
      .windows
      .values()
      .map(|model| model.a11y.enumerated)
      .sum(),
  ```
- `crates/kivact-core/src/diagnostics/stats.rs`：`pub a11y_published: u64,` の次に
  ```rust
  /// Cumulative nodes the accessibility identity enumeration visited (a
  /// complete pass per structural change; payload-only changes visit none).
  pub a11y_enumerated: u64,
  ```
- `crates/kivact-core/src/launch/inspect.rs`：`a11y_published: ts.a11y_published,` の次に `a11y_enumerated: ts.a11y_enumerated,`。

### 試作での確認
`cargo clippy -p kivact-engine -p kivact-core --all-targets --features kivact-engine/testing -- -D warnings` 成功。AH4 の E2E がこの counter を読んで成功。

---

## AH3（付属書 D2 3.6、W3 lane B）

### 判定：修正あり

付属書の移動対象（`ContextPlane`、`hit`・`hit_any`・`hit_node`・`hit_in`・`backface_hidden_at`・`hit_box`・`hit_context`・`collect_planes`・`plane_point`）は HEAD の `model/interact/geometry.rs` に全てあり、署名も付属書の前提どおり。**F4 は ink の不変条件を保っている**：layout pass の位置だけの変化は `layout()` の `changes.laid_out(..)` が flat parent を `redrawn` に入れ、`PaintList::invalidate` → `forget_ink` が親と祖先の ink を捨てる。子自身の size・content の変化は `ink()` の cache 判定（`kept.size == layout.size` 等）が拾う。`ink`（`paint/list/ink.rs`、`pub(super)`）・`PaintState.list`・`PaintList.ink_gen`（`pub(super)`）は `paint::list::hit_reach` から見える。module 規則（`dev/xtask/policy/workspace.toml` の `[module_rules.kivact-engine.uses]`）は `paint → model` を許し、`model` は `paint` を使わないので違反しない。`HitScope { prune: bool, hosts: HashSet }` は数える field が 1 つで state-fields に当たらない。

### 修正
1. `set_scroll` の 1 行（`self.a11y.moved(id)` → `self.a11y_scrolled(id)`）は AH1 が行う（AH1 の 10）。AH3 は行わない。
2. `host/testing/self_check.rs`：付属書 3.6.4 のブロックを `self_check` の中に書くと `self_check` が 81 行になり clippy `too_many_lines`（80）で落ちる。代わりに、`errors.extend(model.text_self_check());` の次に 1 行
   ```rust
   compare_hits(model, self.mouse_position(kivact_types::WindowId::MAIN), &mut errors);
   ```
   を足し、file の末尾（`compare_accessibility` の後）に次の関数を足す（`ArvaneTestHost::mouse_position(WindowId)` は既存の公開 method）：
   ```rust
   /// The hit test leaves children out by their inks: it finds what the
   /// complete walk finds, under the mouse and at the window's center.
   fn compare_hits(
       model: &crate::model::Model,
       mouse: Option<kivact_types::Point>,
       errors: &mut Vec<String>,
   ) {
       let size = model.window();
       let center = kivact_types::Point {
           x: size.width / 2.0,
           y: size.height / 2.0,
       };
       for at in mouse.into_iter().chain([center]) {
           let (pruned, reference) = (model.hit(at), model.hit_reference(at));
           if pruned != reference {
               errors.push(format!(
                   "hit at ({}, {}): pruned {pruned:?}, reference {reference:?}",
                   at.x, at.y
               ));
           }
       }
   }
   ```
3. `model/interact/hit.rs` の `use` は次の 6 行（試作で過不足なし）：
   ```rust
   use crate::layer::Look;
   use crate::model::Model;
   use crate::tree::node::HostNode;
   use kivact_types::geometry3d::Matrix4;
   use kivact_types::{NodeId, Point, Rect};
   use std::collections::{HashMap, HashSet};
   ```
   file の並び：module doc → `use` → 移した `struct ContextPlane` → `trait HitInk` → `struct HitScope` → `const HIT_SLACK` → `impl Model { hit_scope, may_hit }` → `impl Model { 移した関数（付属書の変更を入れたもの） }`。`hit_box` の `let Some(cn) = self.tree.get(c) else { continue };` は付属書のとおり（HEAD の `is_none_or(..)` の if を置き換える）。
4. `model/interact/geometry.rs` の後始末（付属書の「警告どおり」を確定）：`use kivact_types::geometry3d::Matrix4;` を削除、`use std::collections::{HashMap, HashSet};` → `use std::collections::HashMap;`。module doc の 1 行目を `//! Role: Lay out the node tree and resolve window rects, overlay origins, layers and scroll offsets.` にし、2 段落目の「…, hit testing, scroll offsets …」から `hit testing, ` を消す。Invariants の 2 番目（「A node is hit inside its box's shape as painted …」）と 3 番目（「A node that is not hit testable …」）の箇条は **そのまま切り取って** `hit.rs` の Invariants の先頭に移す。
5. `model/drawables/hit.rs` の `drawable_hit`・`exported_hit` は `hit_node`（枝刈りあり）を呼ぶので、debug の参照の walk でも drawable の中は枝刈りされる。付属書の設計（drawable の host とその祖先は枝刈りしない）で正しさは保たれるので **このファイルは変えない**。代わりに `hit.rs` の `hit_node` の doc の最後に「It leaves children out by their inks (`HitScope`), also when the debug reference walks a drawable's subtree.」を足す。
6. **`paint/list/ink.rs` の `ink` の cache 判定（付属書に無い設計の補正、必須）**：`if let Some(kept) = host.paint.ink[n.index.0].get() && kept.generation == generation && kept.size == layout.size && kept.content == layout.content_size {` の条件の最後に 1 つ足す：
   ```rust
       && kept.content == layout.content_size
       // Kept as drawing nothing while it or an ancestor was hidden: once
       // drawn again it is looked at again (showing a node forgets the
       // inks of the changed box and its ancestors, not of those below).
       && (kept.rect.is_some() || !host.tree().drawn(id))
   ```
   理由：親の `ink_local` は隠れている子にも `ink` を呼び、`None` を子の cell に残す。後で祖先（特に `display: contents` の wrapper。`forget_ink` は `box_of` で flat parent から上しか消さない）の可視性が戻っても、子の cell の `None` は捨てられない（F4 の前からの性質。paint は自分の box が見えない時の subtree の culling にしか ink を使わないので表に出なかった）。hit の枝刈りはこの `None` で子を飛ばすので、試作では E2E `style_paint::visibility_hidden_keeps_layout_but_hides_the_node` が self_check の `hit at (45, 20): pruned Some(NodeId(2)), reference Some(NodeId(4))` で失敗した。上の 1 条件で、`None` の entry だけ O(深さ) の `drawn` を確かめ直す（`Some` の entry の費用は変わらない）。補正の後は成功。
7. 完了時の確認の E2E filter を `transform transform_3d pointer hover drawables layout_scroll overlay touch_scroll interaction_drag_edit style_paint` にする（付属書 5.1 が挙げる枝刈りの例外に関わる試験と、6 の再現の試験）。

### 所有ファイルの変更
付属書 P2-U2 の一覧の「`model/interact/geometry.rs`（hit の移動と 664 行）」を「`model/interact/geometry.rs`（hit の移動、import と module doc）」にし、`crates/kivact-engine/src/paint/list/ink.rs`（6 の条件 1 つだけ）を足す。W3 の他の lane は `paint/list/ink.rs` を触らない。

### wave・lane・順序の変更
なし（W3 lane B の先頭。T7 が同じ `geometry.rs` を後で変える）。

### 試作での確認
AH1 の上に 1〜6 を行い、clippy（`-D warnings`）成功、state-fields exit 0、E2E 全件で self_check の hit の不一致 0（6 の前は 1 本で不一致。下の AH4 の `hit_reaches_boxes_outside_their_parents` を含む）。

---

## AH4（付属書 D2 5.3、W4 lane A）

### 判定：修正あり（E2E を完全なコードで確定）

付属書の文章の手順を、試作（AH1〜AH3・AH2 の上）で 5 本とも成功した完全なコードにする。付属書の判断の余地を次のとおり決めた：kit の `button` は `Element` を実装するので `.test_id`・`.on_press` を使う。`Locator::scroll` は要素の中心へ mouse を動かしてから wheel を送るので、scroll の試験は先に `hover()` してから基準の stats を取る（hover の再描画を基準に含めない）。`with_node` は実在する `Element::with_node`。5 本目の行は `pressable(String)` に名前を渡すため helper の引数を `impl Into<String>` にした（`transform.rs` の `pressable` は `&'static str`）。sticky の見出しと 200 行は一つの scroll（`rows`）にまとめた。

### 修正
`dev/tests/kivact-public-api/tests/e2e/a11y_incremental.rs`（新）を下の内容にする。`main.rs` の先頭の `mod apg_breadcrumb;` の前の行に `mod a11y_incremental;` を足す（アルファベット順で最初）。STOP 条件は作業指示書 W4 のまま。

```rust
//! Role: Tests that accessibility publishes in proportion to what changed
//! and that the pruned hit test finds what the complete walk finds.
//!
//! Hover, press, focus, a text update and a scroll publish only the
//! payloads they change: the identity enumeration (`Stats::a11y_enumerated`)
//! does not run for them. What is published is checked against a complete
//! build at every settle by the debug self-check, which also compares the
//! hit under the mouse and at the window's center with the complete walk.

use std::cell::RefCell;
use std::rc::Rc;

use crate::paint::{at, canvas_of};
use crate::support::mount;
use kivact::prelude::*;
use kivact::style::{Display, Position};
use kivact::test::Screen;
use kivact::view::View;

type Log = Rc<RefCell<Vec<String>>>;

/// The `kv_latency` screen at 200 rows: a text `Other <n>`, the buttons
/// `Other` and `Bump`, and a 300 px scroll box `list` of rows (a label and a
/// button with the test id `x<i>`); with `pinned`, the list's content also
/// holds a 20 px fixed box `pin`.
fn latency_screen(pinned: bool) -> Screen {
    let (screen, ()) = mount(move || {
        let other = signal(0u32);
        let first = signal(String::from("Row 0"));
        let mut rows: Vec<View> = (0..200u32)
            .map(|i| {
                let label = if i == 0 {
                    text(move || first.get()).into_view()
                } else {
                    text(format!("Row {i}")).into_view()
                };
                row((label, button("x").test_id(format!("x{i}"))))
                    .style(style().gap(4))
                    .into_view()
            })
            .collect();
        if pinned {
            rows.push(
                node()
                    .test_id("pin")
                    .style(
                        style()
                            .position(Position::Fixed)
                            .left(10)
                            .top(10)
                            .size(20, 20),
                    )
                    .into_view(),
            );
        }
        let view = column((
            text(move || format!("Other {}", other.get())),
            button("Other").on_press(move || other.update(|n| *n += 1)),
            button("Bump").on_press(move || first.update(|l| l.push('!'))),
            scroll(column(rows))
                .test_id("list")
                .style(style().height(300).shrink(0.0)),
        ));
        ((), view)
    });
    screen
}

#[test]
fn hover_and_press_publish_without_enumerating() {
    let screen = latency_screen(false);
    screen.get_by_test_id("x3").hover();
    let s0 = screen.stats();
    screen.get_by_test_id("x4").hover();
    let s1 = screen.stats();
    assert_eq!(
        s1.a11y_enumerated, s0.a11y_enumerated,
        "a hover enumerates nothing"
    );
    assert!(
        s1.a11y_constructed - s0.a11y_constructed <= 4,
        "a hover builds the two buttons: {}",
        s1.a11y_constructed - s0.a11y_constructed
    );
    screen.get_by_test_id("x4").click();
    let s2 = screen.stats();
    assert_eq!(
        s2.a11y_enumerated, s1.a11y_enumerated,
        "a press and the first focus enumerate nothing"
    );
}

#[test]
fn a_text_update_publishes_only_the_text() {
    let screen = latency_screen(false);
    for name in ["Other", "Bump"] {
        let button = screen.get_by_role(Role::Button, name);
        button.click();
        let s0 = screen.stats();
        button.click();
        let s1 = screen.stats();
        assert_eq!(
            s1.a11y_enumerated, s0.a11y_enumerated,
            "{name}: a text update enumerates nothing"
        );
        assert!(
            s1.a11y_constructed - s0.a11y_constructed <= 6,
            "{name}: the button and the text: {}",
            s1.a11y_constructed - s0.a11y_constructed
        );
    }
}

#[test]
fn scrolling_publishes_the_container_and_its_layers() {
    let screen = latency_screen(true);
    let list = screen.get_by_test_id("list");
    list.hover();
    let s0 = screen.stats();
    list.scroll(0.0, 120.0);
    let s1 = screen.stats();
    assert_eq!(
        s1.a11y_enumerated, s0.a11y_enumerated,
        "a scroll enumerates nothing"
    );
    assert!(
        s1.a11y_constructed - s0.a11y_constructed <= 6,
        "the container and the fixed box: {}",
        s1.a11y_constructed - s0.a11y_constructed
    );
}

#[test]
fn a_list_clips_rows_by_scroll_and_focus() {
    let (screen, ()) = mount(|| {
        let options: Vec<Node> = (0..100)
            .map(|i| {
                node()
                    .role(Role::Option)
                    .name(format!("Option {i}"))
                    .focusable(true)
                    .style(style().height(20).shrink(0.0))
            })
            .collect();
        let view = scroll(column(options))
            .with_node(|n| n.role(Role::ListBox))
            .test_id("options")
            .style(style().height(200).shrink(0.0));
        ((), view)
    });
    let list = screen.get_by_test_id("options");
    list.hover();
    let enumerated = || screen.stats().a11y_enumerated;
    let start = enumerated();
    list.scroll(0.0, 400.0);
    assert_eq!(enumerated(), start, "scrolling a list enumerates nothing");
    screen.get_by_role(Role::Option, "Option 25").focus();
    assert_eq!(enumerated(), start, "focusing a row enumerates nothing");
    list.scroll(0.0, 1200.0);
    assert_eq!(
        enumerated(),
        start,
        "the focused row stays exposed out of view"
    );
    list.scroll(0.0, -1600.0);
    assert_eq!(enumerated(), start, "scrolling back enumerates nothing");
}

/// A button-like box named `name` that logs its activations.
fn pressable(name: impl Into<String>, log: &Log) -> Node {
    let name = name.into();
    let (log, said) = (log.clone(), name.clone());
    node()
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .on_press(move || log.borrow_mut().push(said.clone()))
}

/// What a click at the window point `(x, y)` activated.
fn clicked(screen: &Screen, log: &Log, x: f64, y: f64) -> Vec<String> {
    screen.click_at(x, y);
    std::mem::take(&mut *log.borrow_mut())
}

/// 200 pressable rows of 20 px in a 200 px scroll box at (450, 40) whose
/// first child is a sticky 20 px heading.
fn rows_with_heading(log: &Log) -> impl IntoView {
    let mut children: Vec<Node> = vec![
        pressable("heading", log).style(
            style()
                .position(Position::Sticky)
                .top(0)
                .height(20)
                .shrink(0.0)
                .background(rgb(0xdddddd)),
        ),
    ];
    children.extend(
        (0..200).map(|i| pressable(format!("row {i}"), log).style(style().height(20).shrink(0.0))),
    );
    scroll(column(children))
        .test_id("rows")
        .style(at(450, 40, 200, 200))
}

#[test]
fn hit_reaches_boxes_outside_their_parents() {
    let log: Log = Rc::default();
    let (screen, ()) = mount({
        let log = log.clone();
        move || {
            let view = canvas_of((
                node().style(at(20, 40, 100, 100)).child(
                    pressable("badge", &log).style(
                        style()
                            .position(Position::Absolute)
                            .left(90)
                            .top(-20)
                            .size(30, 30),
                    ),
                ),
                node()
                    .style(at(200, 40, 100, 100))
                    .child(pressable("moved", &log).style(style().size(40, 40).translate(150, 0))),
                pressable("under", &log).style(at(20, 200, 100, 100)),
                node()
                    .hit_testable(false)
                    .style(at(20, 200, 100, 100))
                    .child(pressable("ignored", &log).style(style().size(100, 100))),
                node()
                    .style(style().display(Display::Contents))
                    .child(pressable("contents", &log).style(at(200, 200, 60, 60))),
                rows_with_heading(&log),
            ));
            ((), view)
        }
    });
    assert_eq!(
        clicked(&screen, &log, 125.0, 30.0),
        ["badge"],
        "outside its parent, above it"
    );
    assert_eq!(
        clicked(&screen, &log, 370.0, 60.0),
        ["moved"],
        "translated out of its parent"
    );
    assert_eq!(
        clicked(&screen, &log, 70.0, 250.0),
        ["under"],
        "through a box that is not hit testable"
    );
    assert_eq!(
        clicked(&screen, &log, 230.0, 230.0),
        ["contents"],
        "in a display: contents wrapper"
    );
    screen.get_by_test_id("rows").scroll(0.0, 300.0);
    assert_eq!(
        clicked(&screen, &log, 550.0, 50.0),
        ["heading"],
        "the stuck heading over the rows"
    );
    assert_eq!(
        clicked(&screen, &log, 550.0, 130.0),
        ["row 18"],
        "a row scrolled into view"
    );
}
```

### 試作での確認
AH1〜AH3・AH2 の上で 5 本とも成功（`a11y_enumerated` は 1〜4 の各段で不変、`a11y_constructed` の差は上限内）。5 本目は HEAD でも成功する（枝刈りの前後で同じ結果であることの確認）。

### 所有ファイル・wave の変更
なし（W4 lane A）。

---

## F7（付属書 D1 5.2、W2 lane B）

### 判定：修正あり（E2E を完全なコードで確定。付属書の手順のままでは 3 本が失敗する）

付属書の表の手順を HEAD（F1〜F6 の後）で書いて実行した。付属書のとおり「状態を変えた直後から ticks を数える」と、変更の前に予約済みだった frame が 1 回 tick するため `hidden_gif…`・`reduced_motion…`・`display_shader_clock…` の 3 本が `left: 1, right: 0` で失敗した（設計どおりの振る舞いで、試験の手順の誤り）。変更の後に 100 ms 流してから数えると成功する。また次を決めた：Hide/Show・Bump は button の click ではなく signal の書込みで行う（press・hover・focus の遷移と focus ring が `frame_ticks` と fresh render との画素比較を乱さない）。`percent_and_absolute…` は `Locator::rect()`（layout の box）で幅を見る（付属書の `accessibility_bounds()` は a11y の公開を経由するので、layout の S-3 の確認には余計）。`duplicate_layout_id…` は `UiError::message()` に `"dup"` を含む報告の数を見る（`kivact-core/src/client/dispatch/diagnostics.rs` の文言は `duplicate layout_id {id}`）。

### 修正
1. `dev/tests/kivact-public-api/tests/e2e/images_animated.rs`：`fn gif() -> Vec<u8> {` を `pub(super) fn gif() -> Vec<u8> {` にする（この file の唯一の変更）。
2. `dev/tests/kivact-public-api/tests/e2e/main.rs`：`mod focus;` の次の行に `mod frame_sources;`。
3. 新 `dev/tests/kivact-public-api/tests/e2e/frame_sources.rs` を下の内容にする。
4. 確認：`cargo test -p kivact-public-api-tests --test e2e --no-run`、`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- frame_sources`。

### 試作での確認
HEAD の上で 7 本とも成功（F5-fix・AH1〜AH3 の上でも成功）。

### wave・lane・順序の変更
W2 lane B の最後（F5-fix の後）。所有ファイルは付属書どおり。

```rust
//! Role: Tests for the frame sources the host keeps in an index.
//!
//! An animated image and a display-rate shader clock ask for frames only
//! while their node is shown, attached and not under reduced motion (an
//! essential shader keeps its clock); a duplicate layout id is reported
//! once; frames and layouts built from partial updates are what a fresh
//! window draws and lays out.

use std::time::Duration;

use super::images_animated::gif;
use super::paint::{GREEN, RED, at, canvas_of, expect};
use crate::support::mount;
use kivact::draw::PaintShader;
use kivact::geometry::Rect;
use kivact::motion::FrameRate;
use kivact::prelude::*;
use kivact::test::{Screen, render};
use kivact::view::View;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// The frame ticks the screen counts while it advances by `d`.
fn ticks_over(screen: &Screen, d: Duration) -> u64 {
    let before = screen.frame_ticks();
    screen.advance(d);
    screen.frame_ticks() - before
}

/// A 40 × 40 GIF of three 100 ms frames (red, green, blue), hidden while
/// `hidden` holds.
fn hideable_gif(hidden: Signal<bool>) -> Node {
    canvas_of(
        image(gif())
            .test_id("gif")
            .hidden(hidden)
            .style(at(0, 0, 40, 40)),
    )
}

#[test]
fn hidden_gif_stops_and_restarts_from_its_first_frame() {
    let (screen, hidden) = mount(|| {
        let hidden = signal(false);
        (hidden, hideable_gif(hidden))
    });
    expect(&screen.screenshot(), 20, 20, RED, 2, "frame 0");
    screen.advance(ms(100));
    expect(
        &screen.screenshot(),
        20,
        20,
        GREEN,
        2,
        "frame 1 after 100 ms",
    );
    hidden.set(true);
    // The frame asked for before the change still ticks once.
    screen.advance(ms(100));
    assert_eq!(
        ticks_over(&screen, ms(300)),
        0,
        "a hidden GIF asks for no frame"
    );
    hidden.set(false);
    screen.run_until_idle();
    expect(
        &screen.screenshot(),
        20,
        20,
        RED,
        2,
        "shown again at frame 0",
    );
    screen.advance(ms(100));
    expect(
        &screen.screenshot(),
        20,
        20,
        GREEN,
        2,
        "frame 1 100 ms later",
    );
}

#[test]
fn removed_gif_stops_requesting_frames() {
    let (screen, on) = mount(|| {
        let on = signal(true);
        let view = canvas_of(show(on, || {
            image(gif()).test_id("gif").style(at(0, 0, 40, 40))
        }));
        (on, view)
    });
    assert!(
        ticks_over(&screen, ms(200)) > 0,
        "a playing GIF asks for frames"
    );
    on.set(false);
    screen.run_until_idle();
    screen.advance(ms(500));
    assert_eq!(
        ticks_over(&screen, ms(300)),
        0,
        "a removed GIF asks for no frame"
    );
}

#[test]
fn reduced_motion_toggle_stops_and_resumes_gif() {
    let screen = render(|| canvas_of(image(gif()).test_id("gif").style(at(0, 0, 40, 40))));
    screen.advance(ms(100));
    expect(
        &screen.screenshot(),
        20,
        20,
        GREEN,
        2,
        "frame 1 after 100 ms",
    );
    screen.set_reduced_motion(true);
    screen.advance(ms(100));
    expect(
        &screen.screenshot(),
        20,
        20,
        RED,
        2,
        "the first frame under reduced motion",
    );
    assert_eq!(
        ticks_over(&screen, ms(300)),
        0,
        "no frame under reduced motion"
    );
    expect(
        &screen.screenshot(),
        20,
        20,
        RED,
        2,
        "still the first frame",
    );
    screen.set_reduced_motion(false);
    assert!(
        ticks_over(&screen, ms(100)) > 0,
        "frames go on without reduced motion"
    );
}

/// A display-rate shader whose red channel follows its clock.
const CLOCK: &str = "fn paint(position: vec2<f32>, context: Context) -> vec4<f32> {\n    return vec4<f32>(fract(context.time), 0.0, 0.0, 1.0);\n}\n";

/// A 100 × 100 canvas filled with the clocked shader (`essential`: it runs
/// under reduced motion too), hidden while `hidden` holds.
fn clocked(hidden: Signal<bool>, essential: bool) -> Node {
    let shader = PaintShader::wgsl(CLOCK).expect("a valid paint shader");
    canvas_of(
        canvas(move |d, _| {
            let paint = shader.paint().animate(FrameRate::Display);
            let paint = if essential { paint.essential() } else { paint };
            d.fill_rect(Rect::new(0.0, 0.0, 100.0, 100.0), paint);
        })
        .hidden(hidden)
        .style(at(0, 0, 100, 100)),
    )
}

#[test]
fn display_shader_clock_follows_hidden_and_reduced_motion() {
    let (screen, hidden) = mount(|| {
        let hidden = signal(false);
        (hidden, clocked(hidden, false))
    });
    assert!(
        ticks_over(&screen, ms(100)) >= 4,
        "a display-rate clock ticks every display frame"
    );
    hidden.set(true);
    screen.advance(ms(100));
    assert_eq!(ticks_over(&screen, ms(100)), 0, "hidden: no clock");
    hidden.set(false);
    screen.run_until_idle();
    assert!(ticks_over(&screen, ms(100)) >= 4, "shown again: it ticks");
    screen.set_reduced_motion(true);
    screen.advance(ms(100));
    assert_eq!(
        ticks_over(&screen, ms(100)),
        0,
        "reduced motion stops a decorative clock"
    );

    let (screen, _) = mount(|| {
        let hidden = signal(false);
        (hidden, clocked(hidden, true))
    });
    screen.set_reduced_motion(true);
    assert!(
        ticks_over(&screen, ms(100)) >= 4,
        "an essential clock ticks under reduced motion"
    );
}

#[test]
fn duplicate_layout_id_names_the_later_node_once() {
    let screen = render(|| {
        canvas_of((
            node().layout_id("dup").style(at(0, 0, 20, 20)),
            node()
                .layout_id("dup")
                .test_id("second")
                .style(at(40, 0, 20, 20)),
        ))
    });
    let reports = |screen: &Screen| {
        screen
            .errors()
            .iter()
            .filter(|e| e.message().contains("dup"))
            .count()
    };
    assert_eq!(reports(&screen), 1, "{:?}", screen.errors());
    screen.advance(ms(100));
    screen.advance(ms(100));
    assert_eq!(reports(&screen), 1, "reported once: {:?}", screen.errors());
}

/// 50 rows of a label and a button in a 300 px scroll box; the first
/// label ends in `bumps` exclamation marks.
fn bumped_rows(bumps: Signal<usize>) -> View {
    let rows: Vec<View> = (0..50usize)
        .map(|i| {
            let label = move || {
                if i == 0 {
                    format!("Row 0{}", "!".repeat(bumps.get()))
                } else {
                    format!("Row {i}")
                }
            };
            row((text(label), button("x")))
                .style(style().gap(4))
                .into_view()
        })
        .collect();
    scroll(column(rows))
        .style(style().height(300).shrink(0.0))
        .into_view()
}

#[test]
fn incremental_frames_match_a_fresh_render() {
    let (screen, bumps) = mount(|| {
        let bumps = signal(0usize);
        (bumps, bumped_rows(bumps))
    });
    screen.screenshot();
    for n in 1..=20 {
        bumps.set(n);
        screen.screenshot();
    }
    let fresh = render(|| bumped_rows(signal(20)));
    assert_eq!(
        screen.screenshot().hash(),
        fresh.screenshot().hash(),
        "20 partial frames end in the pixels of a fresh frame"
    );
}

#[test]
fn percent_and_absolute_layouts_survive_parent_resize() {
    let (screen, wide) = mount(|| {
        let wide = signal(false);
        let view = row((
            node()
                .test_id("pct")
                .style(style().width(pct(50)).height(20).shrink(0.0)),
            node()
                .test_id("px")
                .style(style().width(100).height(20).shrink(0.0)),
        ))
        .style(move || style().width(if wide.get() { 600 } else { 400 }).height(20));
        (wide, view)
    });
    let width = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").width;
    for (on, parent) in [(true, 600.0), (false, 400.0)] {
        wide.set(on);
        screen.run_until_idle();
        assert_eq!(width("pct"), parent / 2.0, "50 % of a {parent} px row");
        assert_eq!(width("px"), 100.0, "100 px in a {parent} px row");
    }
}
```

---

## T7（付属書 D7 3.6 M3-6、W3 lane B）

### 判定：修正あり（1 か所、clippy）

HEAD との照合：`HostNode` の `sent_layout`・`sent_scroll`・`sent_image` は `tree/node.rs` にあり、書き手・読み手は `model/interact/geometry.rs` `report_geometry`（`n.sent_scroll.update(..)`、`n.sent_layout.update(..)`）、`model/interact/sync.rs` `report_images`（`n.sent_image.update(..)`）、`tree/node_tree.rs` `set_observed`（`n.sent_layout = Default::default()`）の 4 か所だけ（workspace を grep 済み）。どれも `n` は `&mut HostNode` なので `get_or_insert_default()` が使える。AH3 は `geometry.rs` の hit の塊だけを移し、`report_geometry` は geometry.rs に残るので、T7 の手順は AH3 の後のコードにそのまま当たる。size の上限 `<= 312` は「index 8・properties ≤240（`NodeProperties` の assert）・scenes 16・scroll 16・sent 8・place 8・shift 16」の和で、M3（W3 lane A、`Box` → `Rc` は同じ 8 byte）の前でも後でも成り立つ。state-fields：`HostNode` の数える field は `sent: Option<..>` の 1 つだけ（`Sent` は helper で、他に stored が無いので数えない）。`SentReports` の 3 つの `Sent` も数えない。

### 修正
- `tree/node_tree.rs` `set_observed`：付属書の形（`if changed && let Some(n) = .. { if let Some(sent) = .. { .. } }`）は clippy の `collapsible_if` で落ちる（試作で確認）。次の形にする：
  ```rust
  if changed
      && let Some(n) = self.nodes.get_mut(&id)
      && let Some(sent) = n.sent.as_deref_mut()
  {
      sent.layout = Default::default();
  }
  ```
- `geometry.rs` の `sent_layout` の書換えは rustfmt 後に次の形になる（参考）：`&& n.sent.get_or_insert_default().layout.update(crate::tree::node::LayoutReport(..)).is_some()`。
- `SentReports` は `tree/node.rs` の `LayoutReport` の doc の直前に置く。

### 試作での確認
AH1・AH3 の上で `cargo clippy -p kivact-engine --all-targets --features testing -- -D warnings` 成功（上の修正の後）、state-fields exit 0、`node_ref`・`scroll_timeline`・`image_semantics`・`assets` の E2E 成功。

### 所有ファイル・wave の変更
なし（W3 lane B、AH3 の後、AH2 の前）。

---

## S3（付属書 D4 3 章「D2」の host 側と 5.2 の E2E 3 本、W3 lane F）

### 判定：修正あり（コードと E2E を確定）

S2（W1 lane F、merge 済み）が `arvane::Window::forget_frame_state` と `Presenter`・`CpuCompositor` 側を入れてある（`crates/arvane/src/window.rs`、`presenter/frame_policy.rs`）。host 側の HEAD：`LastPresent { Settled, Skipped { retry_at_ns } }`（`host/window.rs`）、書き手は `events.rs` `redraw`（Shown・Skipped・Dropped・SurfaceLost）・`wake`（retry）・`lifecycle_event`（Focused）、読み手は `pump.rs` `retry_at`。`LastPresent` は名前が `Present` で終わる phase enum で、`HostWindow` の他の Option（`last_stats`・`parent`）は marker 済みなので、variant を足しても state-fields は変わらない（`Shown { at_ns: u64 }` の `at_ns` は数えない）。AH1（W2）は `events.rs` の `accessibility_event` だけを変えるので衝突しない。

付属書の「`wake` の先頭で解放」を、`pump.rs` の helper にして `wake` には 1 行だけ足す形に決めた（`wake` の行数を増やさない）。`paint_due` の window は解放しない（その redraw が `Shown` を書き直す）。

### 修正
1. `host/window.rs`：`enum LastPresent` の前に
   ```rust
   /// How long an idle window keeps the CPU frame state of its last shown
   /// frame (a later frame redraws only what changed); after it the state is
   /// released and the next frame redraws everything.
   pub(super) const FRAME_STATE_KEEP_NS: u64 = 1_000_000_000;
   ```
   `LastPresent` の `Settled` の doc と variant を次の 2 つにする（`Skipped` はそのまま）：
   ```rust
   /// Shown at `at_ns`: the window keeps its CPU frame state for the next
   /// partial frame until `at_ns + FRAME_STATE_KEEP_NS`.
   Shown { at_ns: u64 },
   /// Nothing to keep or retry: dropped, lost, retried, or released after
   /// idling (a change or a surface loss redraws).
   Settled,
   ```
2. `host/events.rs` `redraw` の `Ok(arvane::Presented::Shown { .. })` の腕：`.last_present = LastPresent::Settled;` → `.last_present = LastPresent::Shown { at_ns: windows.now_ns() };`。他の腕は変えない。`wake` の retry の `for` の直後（`self.tick_windows(now);` の前）に `self.release_idle_frames(now, windows);` を 1 行。
3. `host/pump.rs`：import を `use super::window::{FRAME_STATE_KEEP_NS, HostWindow, ImeAllowed, LastPresent, SoftKeyboard, Surface};` に。`retry_at` の `LastPresent::Settled => None,` を `LastPresent::Shown { .. } | LastPresent::Settled => None,` に。`retry_at` の後に：
   ```rust
   /// When window `id` releases the CPU frame state of its last shown
   /// frame ([`FRAME_STATE_KEEP_NS`] after it).
   fn release_at(&self, id: WindowId) -> Option<u64> {
       match self.windows.get(&id)?.last_present {
           LastPresent::Shown { at_ns } => Some(at_ns.saturating_add(FRAME_STATE_KEEP_NS)),
           LastPresent::Settled | LastPresent::Skipped { .. } => None,
       }
   }

   /// Release the CPU frame state of every window whose last shown frame
   /// is [`FRAME_STATE_KEEP_NS`] old at `now`, unless it has a frame due
   /// (that frame shows again). The window keeps showing its picture;
   /// its next frame redraws everything.
   pub(super) fn release_idle_frames(&mut self, now: u64, windows: &mut Windows<'_>) {
       for id in self.windows.keys().copied().collect::<Vec<_>>() {
           if self.release_at(id).is_none_or(|at| at > now) || self.paint_due(id) {
               continue;
           }
           if let Some(mut window) = windows.get(id) {
               window.forget_frame_state();
           }
           self.windows
               .get_mut(&id)
               .expect("invariant: the released window has host state")
               .last_present = LastPresent::Settled;
       }
   }
   ```
   `control` の `next` の配列で、`retry_at` の `.min(),` の次の要素に
   ```rust
   self.windows
       .keys()
       .filter_map(|id| self.release_at(*id))
       .min(),
   ```
4. `dev/tests/kivact-public-api/tests/e2e/compositor.rs` の末尾に下の 3 本を足す（既存の `grid`・`assert_same_pixels`・`W`・`H`・`mount`・`render` を使う。import の追加は不要）。付属書 5.2 の 1 の「`pixels_rastered` が window の物理 pixel 数に等しい」は、chunk の重なりで等しくならない構成があり得るので「以上」にした（新しい試験の期待値の決定で、既存の期待値を緩めるものではない）。
5. 確認：`cargo clippy -p kivact-engine --all-targets -- -D warnings`、`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- compositor interaction_toast launch_real_window`。

### 既存の試験への影響（試作で実測）
- 解放の wake は events の無い due な wake なので、client に turn を 1 回与える（`wake` の末尾、既存の規則）。`interaction_toast::a_pushed_toast_is_shown_and_goes_when_its_time_is_up_without_frame_polling` の `screen.stats().turns - turns <= 6` は HEAD で 5、S3 の後で 6（上限ちょうどで成功）。期待値は変えない。解放は idle の 1 s 後に 1 回だけなので性能の後退ではない。
- `drawable_export_gpu`（GPU）以外に、1 s 以上進めて `frame_stats` を読む試験は無い（grep 済み）。`kivact-test` の lib の試験は S3 の上で成功。

### 試作での確認
1〜4 で clippy 成功、compositor の既存の全件と新 3 本、toast、launch_real_window が成功。新 1 本目（`idle_window_redraws_whole_after_release`）は HEAD で失敗し S3 の後で成功する（試験が解放を検出している）。

### 所有ファイル・wave の変更
なし（W3 lane F、付属書 P4-U3 の所有ファイルのとおり）。EN-2（W6）は `host/pump.rs` の `control` を `HostPhase` に合わせて書き換えるので、EN-2 の指示に「`control` の `release_at` の要素と `LastPresent::Shown` を残す」を足すこと（「全体」節）。

#### S3 の修正 4 の E2E（`compositor.rs` の末尾に足す）

```rust

/// A window idle for a second releases its CPU frame state: the next
/// frame redraws the whole window, with the same pixels as a fresh one.
#[test]
fn idle_window_redraws_whole_after_release() {
    let (screen, hot) = mount(|| {
        let hot = signal(BLUE);
        (hot, grid(hot))
    });
    screen.screenshot();
    hot.set(RED);
    screen.screenshot();
    let damage = screen.frame_stats().damage_pixels;
    assert!(
        damage < u64::from(W * H) / 10,
        "a frame within the keep time is partial: {damage} px"
    );
    screen.advance(Duration::from_millis(1_100));
    hot.set(GREEN);
    let shot = screen.screenshot();
    let rastered = screen.frame_stats().pixels_rastered;
    assert!(
        rastered >= u64::from(W * H),
        "the first frame after the release rasters the whole window: {rastered} px"
    );
    let fresh = render(|| grid(signal(GREEN))).screenshot();
    assert_same_pixels(&shot, &fresh, 2, "the frame after the release");
}

/// Releasing the frame state draws no frame.
#[test]
fn release_draws_no_frame() {
    let (screen, _hot) = mount(|| {
        let hot = signal(BLUE);
        (hot, grid(hot))
    });
    screen.screenshot();
    let (ticks, stats) = (screen.frame_ticks(), screen.frame_stats());
    screen.advance(Duration::from_secs(2));
    assert_eq!(screen.frame_ticks(), ticks, "no frame tick");
    assert_eq!(screen.frame_stats(), stats, "no frame presented");
}

/// Frames less than a second apart keep the frame state: each is partial.
#[test]
fn frames_within_keep_time_stay_partial() {
    let (screen, hot) = mount(|| {
        let hot = signal(BLUE);
        (hot, grid(hot))
    });
    screen.screenshot();
    for (i, colour) in [RED, GREEN, RED, GREEN].into_iter().enumerate() {
        screen.advance(Duration::from_millis(500));
        hot.set(colour);
        screen.screenshot();
        let damage = screen.frame_stats().damage_pixels;
        assert!(
            damage < u64::from(W * H) / 10,
            "update {i}, 500 ms after the last frame, is partial: {damage} px"
        );
    }
}
```

---

## 全体

### wave・lane の組み直し（担当範囲。同じ wave の lane の間でファイルの重なりなし）

| wave | lane | unit の順 | 所有ファイル（本書の変更後） |
|---|---|---|---|
| W2 | A | AH1 | 付属書 P2-U1a ＋ `kivact-engine/src/a11y.rs`（doc）＋ `model/interact/geometry.rs`（`set_scroll` の 1 行） |
| W2 | B | **F5-fix** → S4（cherry-pick）→ S5 → F7 | F5-fix：`arvane-compose/src/{cache.rs, plan/damage.rs}`。F7：`e2e/{frame_sources.rs（新）, main.rs, images_animated.rs}` |
| W3 | B | AH3 → T7 → AH2 | AH3：付属書 P2-U2 ＋ `paint/list/ink.rs`（条件 1 つ）、`geometry.rs` は hit の移動・import・doc だけ。T7・AH2：付属書のとおり |
| W3 | F | G4 → S3 | S3：`kivact-engine/src/host/{window,events,pump}.rs`、`e2e/compositor.rs` |
| W4 | A | AH4 | `e2e/{a11y_incremental.rs（新）, main.rs}` |

重なりの確認：W2 で `model/interact/geometry.rs`・`host/events.rs` を触るのは AH1 だけ。W3 で `paint/list/ink.rs`・`host/testing/self_check.rs` を触るのは AH3 だけ、`host/{window,events,pump}.rs` は S3 だけ、`e2e/compositor.rs` は S3 だけ、`model/interact/sync.rs` は T7 だけ（M3 の所有に sync.rs は無い）。`e2e/main.rs` は W2 の F7 と W4 の AH4 だけ（別 wave）。

### 他の担当の unit に伝える結合（本書の変更が後の unit の付属書の前提を変えるもの）

1. **P8-EN（W6 L2、D8 付録の `crates/kivact-engine/src/mutation.rs:123 Mutation.local_text` の行）**：AH1 がこの field を `payload_only` に改名する。P8-EN はその行の marker の文言を `Mutation.payload_only` の上の marker に書く（文言は D8 のまま。typed.rs:268 の参照は `Mut::MEASURE.and(Mut::PAYLOAD_ONLY)` になる）。作業指示書 0.4・V2 設計書 5.4 の STOP に当たらないよう、P8-EN の追加の指示に 1 行足すこと。
2. **M3（W3 lane A、D3 M2-5 の typed.rs）**：`boxed(..)` → `shared(..)` だけを行い、AH1 が書いた `mut = Mut::PAINT.and(Mut::PAYLOAD_ONLY)`・`Mut::TEXT.and(Mut::PAYLOAD_ONLY)` を `mut = Mut::PAINT;` 等に戻さない（D3 の文は `mut =` に触れないが、行ごと置き換えると戻る）。同じく M3 の `aggregate.rs` の変更（M2-4）は AH1 が書き換えた `repaint` と新設の `a11y_scrolled` を残す。
3. **EN-2（W6 L2、D5 F-7 の `HostPhase` / `Host.next_wake`）**：S3 が `pump.rs` `control` の `next` の配列に `release_at` の要素を足し、`LastPresent` に `Shown { at_ns }` を足す。EN-2 は `control` を書き換えるときこの要素を残す（V2 設計書 5.3 の表の注意どおり）。

### 試作の総合結果

worktree に F5-fix・AH1・AH3・T7・AH2・S3 の本書どおりの変更と F7・AH4・S3 の E2E を全て入れ、`cargo clippy -p kivact-engine -p kivact-core -p arvane-compose --all-targets --features kivact-engine/testing -- -D warnings` 成功、`state-fields` は kivact-engine・arvane-compose で exit 0（kivact-core は M7 の `Plan` の既知の 1 件だけ）、`cargo test -p kivact-test` 成功、`cargo test -p kivact-public-api-tests --test e2e`（全 635 本、debug、self_check 有効）で失敗は次の 1 本だけ。試作の全差分は `C:\Users\ryo\AppData\Local\Temp\claude\fd3f2b4db3f6db24\76c64dca-56c5-4a94-ab86-dbee8a09d336\scratchpad\audit-R1-prototype.patch`（参考。実装は本書の文に従う）。

- **HEAD で既に失敗している E2E（本担当の外）**：`diagnostics::why_follows_the_prop_to_the_write_through_a_memo`（`dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:104`、期待の `node 5 Text "doubled: 4"` に対し実際は `node 4`）。HEAD `2a3b06b8` の素の worktree で失敗することを確かめた。W1 の M1（空の view が host node を作らない、D3 M2-1）で node id が 1 つ詰まった結果と推定。W1 の差戻しに足し、M1 の担当（または CP1 の fix）で期待値の node 番号を直すこと（id は振る舞いではなく番号なので、`node 5` → `node 4` の変更は期待値を緩めることに当たらない）。
- 付記：`cargo check -p kivact-engine`（feature なし）で `paint/state.rs` の `Indexed.size`・`content` が「never read」の warning（F4 の残り。`--all-features` の gate では出ない）。本担当の外。

### ユーザー（人間）にしか決められない項目
なし。本書の 2 つの設計の補正（AH1 の rect の無い node の bounds、AH3 の ink の cache 判定）は、付属書の前提（「bounds は scroll に不変」「可視性の変化で ink が捨てられる」）を実コードで成り立たせるための最小の変更で、性能を悪くせず（後者は `None` の entry だけ O(深さ)）、既存の期待値を変えない。
