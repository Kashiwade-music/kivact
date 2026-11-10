# P7 設計：K2 item text を ≤ 3.64 KiB にする追加設計（P3 の上に積む）

> **V2 付属書 D7**（設計担当 P7 の成果物）。正本は [SLIM_FAST_V2_DESIGN.md](../SLIM_FAST_V2_DESIGN.md)、手順は [SLIM_FAST_V2_WORK_ORDER.md](../SLIM_FAST_V2_WORK_ORDER.md)。
> - 本書の「8 章 設計者（人間）の判断が要る点」は V2 設計書 4 章（DEC-01〜DEC-23）で決定済み。実装者は 8 章を読んで選ばない。V2 設計書 5 章（付属書間の調整）が本書を上書きする箇所がある。
> - 本書の unit 名と V2 の unit ID の対応は V2 設計書 0.2 節。unit の実施順・lane・wave は V2 作業指示書だけが決める（本書 6 章の「依存」「並列可」は参考）。
> - 文中の P1〜P8 は付属書 D1〜D8（`design-P<n>.md` は `D<n>-*.md`）を指す。`W:\w\kv-prof`・`W:\w\kv-mem`・`W:\w\kv-size`・scratchpad の path は設計時の試作・計測の場所（設計した機械にだけある参考）で、実装に必要ではない。行番号は HEAD `73c36c7c` のもの。

担当：K2 item text（WS 差分÷3000、CPU）≤ 3.64 KiB。button / row の余裕の見積り。P3（design-P3.md、M2-1〜M2-15）は採用済みとして、その上に積む。
基点：db8d679b（= HEAD 73c36c7c の製品コード）。file:line は特記なき限り db8d679b（W:\w\kivact）の行。
ID は **M3-x**。P3 の項目を置き換えるものは「M2-x を置き換え」と明記する。

用語：**P1** = P3 の試作（W:\w\kv-mem、P3 の 4.1 節）。**P7-a/b/c/d** = P1 の上に本設計の一部を試作して計測した状態（1.1 節）。計測はすべて本機・CPU renderer・`kv_stress STILL=1`・3 s 待ち（run.sh K2 と同じ手順）。

---

## 1. 事実

### 1.1 計測（本機、WS 差分÷3000、KiB）

同じ 1 回の window-lock の中で取った値を並べる（**WS の絶対値は日によって ±0.25 KiB/item ずれる**：P3 の P1 text は 6.49、本日の P1 text は 6.23〜6.27。同じ exe・同じ手順。比較は同じ回の中の差で行う）。

| 状態 | 内容（累積） | text | button | row | heap text（B/item） |
|---|---|---|---|---|---|
| P1 | P3 の試作 | 6.23 / 6.27 | 35.26 / 35.41 | 46.33 / 46.39 | 6,257 |
| P7-a | + 葉の記録 reserve_exact（M2-13）、TextExtents と computed text の style 共有（M3-2）、text の既定 role を engine に（M3-1）、arvane の計測 cache に入れない（M3-3 の近似）、段落 cache 256→32（不採用案 F） | 3.88〜4.11 | — | — | 4,264 |
| P7-b | + 測定記録 32 B/entry（M3-4）、CoreNode の稀な field を box（M3-5、origins も rare に入れた版）、HostNode の送信済み報告を box（M3-6）、OwnerEntry の稀な field を box（M3-7） | 3.27 / 3.32 | 31.95〜32.11 | 38.56〜38.63 | 3,425 |
| P7-c | + NodeStyle を Rc で共有（M3-9）、performed を疎に（M3-10）、**段落 cache を元の 256 に戻す**（F を外す） | 3.42 / 3.46 | 31.15〜31.27 | 37.66〜37.89 | 3,355 |
| P7-d | + 段落 cache の 2 度目採用（M3-3 確定形）、origins を CoreNode 本体へ戻す（M3-5 確定形）、LeafMeasure の外側 box を外す（M3-2）、Semantics を Rc で共有（M3-8） | **2.93〜3.00**（同じ回の P1 6.26、P7-c 3.36） | **30.22〜30.44** | **36.91〜37.23** | **3,006** |

- 試作は計測のための近似で、確定設計（3 章）との差は 4.1 節に書く。試作の差分と生データは `W:\w\kv-mem\.p3-measure\` に P3 のもの、`W:\w\kv-mem\.p3-measure\p7\{p1,a,b,c,d}\`（ws.txt、heap-*.txt）と `p7\worktree-p7.diff`（P7-d 時点の worktree 全体の差分。試作の書換えで改行が LF になったファイルは全行差分に見える）に本設計のもの。計測の script は `p7\p7meas.sh`・`p7\wsonly.sh`。`W:\w\kv-mem-target` は削除済み。
- 試作で確認したこと：`cargo check --all-targets`（kivact-layout・runtime・engine・core・test）が通る、型の大きさ（`OwnerEntry` 88、`HostNode` 312、`CoreNode` 64、`TextExtents` 64 を size test で確認）、K2 の WS と heap。**E2E は試作で走らせていない**（e2e の debug build が 8 GB の上限を超え、W: の空き容量を一時使い切った。直ちに削除して回復）。
- WS の雑音：同じ回の中の 3 回で ±0.1 KiB（ws0 の初回だけ +0.9 MB 高く出る。中央値を使う）。

### 1.2 text 1 個の構成（kv_stress `MODE=text`）

`dev/bench/src/bin/kv_stress.rs:11-21`：`scroll(column(for_index((0..rows).collect::<Vec<u32>>(), move |i, _| text(format!("Row {i}")))))`。1 item は次を持つ。
- for_index の行（`crates/kivact-core/src/view/flow/list.rs:339-347`）：子 owner 1、行の値の signal（source）1、`rows` Vec の 1 要素（`(Branch, Signal<u32>)`）。
- host node 1（Text）：engine の slot（NodeStore・links・layout 列・paint 列・measure 列）、core の CoreNode（`IdMap`）、内容の String。
- 静的な text には effect も memo も無い（P1 の census：live_reactive は行あたり 1 = 行の source）。

### 1.3 型の大きさ（`size_of`、試作 worktree に置いた計測用 test で確認）

| 型 | B | 備考 |
|---|---|---|
| layout 列 1 node（`columns.rs:213-229` が push する全列） | 1,040 | ChildCursor 48、NodeLayoutState 56（calcs Vec 24 + Record 32）、**NodeStyle 264**、presence_scale 16、natural 12、**taffy::Cache 384**、unrounded（taffy::Layout）92、final_layout 96、**performed（Option<LayoutInput>）56**、touched 4、below 4、origin 8 |
| 葉の測定記録 1 件 `(MeasureRequest, LeafSize)` | 96 | f64 の 64 + 32。text 1 個は 4 件（=384 B の Vec） |
| `HostNode` | 464 | properties 240、**sent_layout 96・sent_scroll 56**・sent_image 1、scenes 16、scroll 16、place 8、shift 16、index 8 |
| `NodeStore` の 1 entry `Option<(NodeId, HostNode)>` | 472 | ＋ links（`NodeLinks`）40 |
| `TextExtents` | 520 | **style（ComputedTextStyle）464**、single 32、wrapped 24 |
| `ComputedTextStyle` | 464 | |
| `Semantics<FocusTarget>` | 416 | |
| `CoreNode` | 216 | `IdMap` の entry は 224 B、hashbrown は全 bucket に触れるので WS ≈ 1.37 倍 |
| `OwnerEntry` | 192 | arena の slot は 200 B |
| graph の slot（Source 等） | 160 | P3 M2-14 で 176 |
| `LeafMeasure` | 32 | Text は `Box<RefCell<Option<Box<TextExtents>>>>`（外側の box が 1 alloc） |
| paint：`Option<Ink>` 80、`Option<Look>` 72、`Indexed`（多数の子を持つ親の子 1 つ）112 | | |

### 1.4 text 1 個の生存 allocation（heap、B/item、`kv_heap MODE=text`、3000 − 0 の差）

| 区分（site） | P1 | P7-c | 何か |
|---|---|---|---|
| layout 列（`NodeLayoutColumns::push`） | 1,417 | 992 | 1.3 節の 1,040 B × 2 冪の容量（4096/3000） |
| 葉の測定（`LayoutPass::leaf`、2 alloc） | 904 | 192 | **TextExtents 520**（`measure/leaf.rs:144` が leaf に inline 展開）＋ 測定記録 384（cap 4 × 96） |
| NodeStore（`NodeStore::reserve`） | 698 | 490 | HostNode entry 472 + links 40、容量込み |
| **computed text**（`Measure::measure`、1 alloc） | 480 | 0 | `Rc<ComputedTextStyle>`（464+16）を **text ごとに** 新しく作る（7 章 1） |
| **Semantics**（`Writer::set ← semantics::bind`） | 416 | 0 | text の `role(Role::Text)` だけの Semantics の box（7 章 2） |
| CoreNode（`ViewNodes::create`） | 323 | 83 | |
| arena（owner 273 + source 215） | 488 | 346 | |
| **arvane の計測 cache**（`Generations::insert<TextKey, TextMetrics>`） | 285 | 0 | text ごとの key（String 込み）と値。TextExtents と重複（7 章 3） |
| **arvane の段落 cache**（`shape_inline_layout` 210・`Shaped::get_or_insert` 101・`layout_at` 97） | 408 | 408 | parley の段落 layout を 2 世代 × 256 = 最大 512 個保持（1 個 ≈ 2.4 KB）。3000 行では有界の ≈1.2 MB が item あたり 0.4 KiB に見える |
| paint 列（ink 80 + look 72） | 207 | 207 | |
| 多数の子を持つ親の子 index（`ChildIndex::build`＋sort） | 175 | 175 | Indexed 112 + starts/furthest 16 |
| 行の Vec と内容（`graph::run`） | 87 | 87 | `rows` Vec の 1 要素 55（`list.rs:309`）＋ 32 B の alloc 1 |
| measure 列 | 65 | 65 | leaf 32 + computed_text 16（＋text の外側 box 16） |
| `Model::sync`（1.13 alloc/item） | 63 | 63 | inline 展開のため site を特定できなかった（8 章 H-P7-5） |
| 行 owner の nodes Vec（`insert_node`） | 32 | 32 | |
| その他 | ~130 | ~130 | |
| **計** | **6,257** | **3,355** | |

P7-d（3,006 B）は P7-c から：段落 cache 408 → ≈ 38（M3-3：描画した段落と一度見た key の hash だけ）、CoreNode 83 → 100（origins を本体に戻した）、measure 列の外側 box −16 ほか。

### 1.5 text の計測値は 4 か所に保持されている

1. taffy の `Cache`（layout 列、node ごと 384 B）：layout 入力 → 出力。
2. 葉の測定記録 `Record`（`kivact-layout/src/columns.rs:31-48`、`tree.rs:107-128`）：`MeasureRequest → LeafSize`。内容が変わった時の `remeasures_same`（`compare.rs:48-60`）が使う。
3. `TextExtents`（`kivact-engine/src/measure/state.rs:131-136`）：node ごとの単行・折り返しの metrics と、それを測った style の**複製（464 B）**。
4. arvane-text の `measured`（`arvane-text/src/cache.rs:637`、2 世代 × 4096）と段落 `paragraphs`（`layouts.rs:27`、2 世代 × 256）：text と style の key ごと。

3 と 4 の `measured` は同じ答えを二重に持つ。段落 cache は「同じ段落を別の幅で測る・描く時に shaping をやり直さない」ためのもので、**一度しか測らない静的な text の段落まで保持**している。

---

## 2. 原因

1. **共有されない同一値**：text ごとの computed text style（464 B、MEM-5 の共有が親の未解決で働かない）、`TextExtents.style` の複製（464 B）、`role(Role::Text)` だけの Semantics（416 B）、同じ layout props から作る `NodeStyle`（264 B）を node ごとに持つ。
2. **稀な field の常駐**：observe される node・scroll する node・image だけが使う送信済み報告（152 B）を全 HostNode が持つ。NodeRef・退場・observation・presence・DevTools の記録（CoreNode）、object・定数・cleanup・context・error handler（OwnerEntry）を全 node・全 owner が持つ。葉では読まれない `performed`（56 B）を全 layout slot が持つ。
3. **重複 cache と一度きりの値の保持**：arvane の `measured` が node の `TextExtents` と同じ答えを持つ。段落 cache が一度しか使わない段落を保持する。測定記録が f64 で 96 B/件、Vec が 2 冪で伸びる。
4. **既定値の明示**：`text()` が engine の既定（Text 種の role は Text、`model/semantics.rs:37-50`）と同じ role を patch として書き、Semantics の box・意味論の bind・`semantic_origins` を作る。

---

## 3. 設計（確定版）

### 3.0 共通の部品：`kivact_types::cell::Recent<T>`（M3-11、P3 M2-5 の `Recent` を移す）

P3 M2-5 は `Recent<T>` を `kivact-engine/src/model/recent.rs` に置いた。本設計は kivact-layout（M3-9）と engine の measure（M3-2）でも使うため、**汎用部分を `crates/kivact-types/src/cell.rs` へ移す**（`cell` は「latch・cache・generation」の小さな部品の module で、`Sent` もここにある）。`model/recent.rs` には `RecentProps` と `SharedProp` だけを残し、`use kivact_types::cell::Recent;` にする。

`crates/kivact-types/src/cell.rs` の末尾に追加（`use std::rc::Rc;` は既存）：

```rust
/// The last few distinct values of one kind, shared behind `Rc` between the
/// owners that store an equal value (repeated components): a value equal to a
/// recent one is that one's `Rc`, any other value replaces the oldest.
/// Compared by value; an `Rc` handed out is never changed in place (a writer
/// that changes it clones it, `Rc::make_mut`).
pub struct Recent<T> {
    slots: [Option<Rc<T>>; RECENT],
    /// The slot the next new value takes (the oldest).
    // xtask-allow(state-fields): independent: the ring position; any value is a valid next slot
    next: usize,
    /// The slot of the last hit or insert, compared first.
    // xtask-allow(state-fields): cache: where the last equal value was; any value is a valid start
    last: usize,
}

const RECENT: usize = 8;

impl<T> Default for Recent<T> {
    fn default() -> Self {
        Self { slots: std::array::from_fn(|_| None), next: 0, last: 0 }
    }
}

// The derives would require `T: Clone` / `T: Debug`; the slots are `Rc`s.
impl<T> Clone for Recent<T> {
    fn clone(&self) -> Self {
        Self { slots: self.slots.clone(), next: self.next, last: self.last }
    }
}

impl<T> std::fmt::Debug for Recent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kept = self.slots.iter().flatten().count();
        f.debug_struct("Recent").field("kept", &kept).finish()
    }
}

impl<T: PartialEq> Recent<T> {
    /// `value` behind an `Rc`: an equal recent value's, else a new one.
    pub fn share(&mut self, value: T) -> Rc<T> {
        for step in 0..RECENT {
            let at = (self.last + RECENT - step) % RECENT;
            if let Some(found) = &self.slots[at]
                && **found == value
            {
                self.last = at;
                return Rc::clone(found);
            }
        }
        let rc = Rc::new(value);
        self.slots[self.next] = Some(Rc::clone(&rc));
        self.last = self.next;
        self.next = (self.next + 1) % RECENT;
        rc
    }
}
```

- 比較は直近の hit から逆順に走る（同じ部品が続く mount では 1 回の比較で当たる。P3 の版は先頭から 8 個まで比較した）。
- P3 M2-5 の `model/recent.rs` の `pub(crate) struct Recent<T>` とその impl を削除し、`RecentProps` の field 型はこの `Recent` を使う。`SLOTS` 定数も削除。
- 単体試験（`cell.rs` の `#[cfg(test)]`）：等しい値 2 つが `Rc::ptr_eq`、9 個の異なる値の後で最初の値は新しい `Rc`、等しくない値は共有しない。

### 3.1 M3-1 text の既定 role を engine に任せる（kivact-core、kivact-engine）

**理由**：engine は role の無い Text 種の node を Role::Text として公開する（`crates/kivact-engine/src/model/semantics.rs:37-50` `semantic_role`、名前は内容 `:71-76`）。`text()` の `.role(Role::Text)` はこれと同じ値を patch として書くだけで、Semantics の box（416 B）・`semantic_origins` の Vec・意味論の bind（P3 M2-7 の plan と publish）を text ごとに作る。

**変更**（`crates/kivact-core/src/view/elements/text.rs`）：
- `:24-28`（`text`）：`.role(Role::Text)` を削除。`Node::new(Content::Plain(content.into_text()), Some(Location::caller()))` を返す。
- `:34`（`string_child`）：`Node::new(Content::Plain(content.into_text()), None)`。
- `:74`（`rich_text`）：`Node::new(Content::Rich(spans.into_prop()), Some(Location::caller()))`。
- `:19`：`use crate::view::{IntoView, View};`（`Role` が未使用になる）。
- module の doc（`:1-12`）に 1 文足す：「A text node states no role: the engine exposes a `Text` node without one as `Role::Text`, named by its content (`kivact_engine` `semantic_role`).」

**DevTools の読み手**：DevTools の a11y tree（`crates/kivact-core/src/devtools/a11y.rs:162`）と role の query（`devtools/query.rs:150`）、node 詳細（`devtools/node.rs:40,88`）、overlay（`devtools/overlay.rs:329`）は engine の **生の** semantics（`Reader::semantics`）を読むため、何もしなければ text が DevTools の a11y tree と `GetByRole(Text)` から消える。`Reader::semantics` を「Text 種の node は role を補う」に変える（今まで core が書いていた値と同じものを返す）。

`crates/kivact-engine/src/host/write/reader.rs:115-119`：

```rust
/// The semantics of `id` as written, with a `Text` node's kind role filled
/// in when it states none (what its builder used to write; the platform
/// tree takes the same role from [`crate::model::semantics::semantic_role`]).
pub fn semantics(&self, id: NodeId) -> Option<kivact_types::Semantics<FocusTarget>> {
    self.with_model(id, |model| {
        let node = model.tree().get(id)?;
        match node.semantics() {
            Some(semantics) => Some(semantics.clone()),
            None if node.kind() == crate::contract::NodeType::Text
                && model.editors[node.index.0].is_none() =>
            {
                Some(kivact_types::Semantics {
                    role: Some(kivact_types::Role::Text),
                    ..Default::default()
                })
            }
            None => None,
        }
    })
    .flatten()
}
```
（`Model::editors` は `pub(crate)`（`model/aggregate.rs:33`）、`with_model` は `reader.rs:459-463`。）

**不変条件**：公開される a11y tree・試験の query（`kivact-test` は engine が作る a11y tree を読む）・DevTools の表示は今と同じ。`text(..).role(r)` を明示した場合は今どおり patch が書かれ、その role が勝つ。`text(..).hidden(..)` 等の他の modifier も今どおり（patch がある node は P3 M2-7 の bind を通る）。

### 3.2 M3-2 計算済み text style と TextExtents の style を共有する（kivact-engine、P3 M2-5b を置き換え）

**変更**（`crates/kivact-engine/src/measure/state.rs`）：

1. `:17-27` `LeafMeasure::Text` の外側の box を外す：`Text(RefCell<Option<Box<TextExtents>>>)`（`LeafMeasure` は Natural の 24 B が最大で 32 B のまま変わらない）。`leaf_for`（`:122-129`）は `LeafMeasure::Text(RefCell::default())`。`take_slot`（`:64-71`）の `extents.into_inner()`、`extents()`（`:90-95`）の `extents.borrow_mut()`、`forget_extents`（`:98-105`）はそのまま型が合う。
2. `:29-34` `MeasureColumns` に field を追加：

```rust
#[derive(Clone, Debug, Default)]
pub(crate) struct MeasureColumns {
    pub(crate) leaf: Vec<LeafMeasure>,
    computed_text: Vec<RefCell<Option<Rc<ComputedTextStyle>>>>,
    /// The last distinct text styles resolved or measured in this window:
    /// the nodes with an equal style share one (M3-2).
    recent_style: RefCell<kivact_types::cell::Recent<ComputedTextStyle>>,
}
```
（`Recent` の `Clone`・`Debug` は 3.0 節の手書きの impl。`RefCell<Recent<_>>` で `MeasureColumns` の derive が通る。）
3. 関数を追加（同じ impl）：

```rust
/// `style` shared with an equal style of this window's recent ones.
pub(crate) fn share_style(&self, style: ComputedTextStyle) -> Rc<ComputedTextStyle> {
    self.recent_style.borrow_mut().share(style)
}
```
4. `:131-136` `TextExtents.style: Rc<kivact_types::ComputedTextStyle>`。

`crates/kivact-engine/src/measure/leaf.rs:141-149`：

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
（`:152` の `narrows(&extents.style, ..)`、`:160` の `text_metrics(text, &extents.style, ..)` は deref で `&ComputedTextStyle` になり変更不要。）

`crates/kivact-engine/src/measure/inherited_text.rs:49-53` `computed_text`：

```rust
let resolved = match parent_kept {
    Some(parent_rc) if *parent_rc == value => parent_rc,
    _ => columns.share_style(value),
};
```
module doc（`:31-36`）の MEM-5 の文を「A style equal to its parent's resolved one shares the parent's `Rc`, any other one an equal recent style's of the window (`MeasureColumns::share_style`), so the text leaves of one list keep one allocation; ...」に直す。

**不変条件**：共有された `ComputedTextStyle` はその場で変えない（`computed_text` の書き手は `set_computed_text` で `Rc` を差し替えるだけ。`TextExtents` は新しい box を作る）。比較は値。ring は window の Model（`MeasureColumns`）と一緒に捨てる。

### 3.3 M3-3 arvane-text は 2 度目に見た段落だけを保持する（arvane-text）

**方針**：段落 cache（`layouts::Shaped`）と計測 cache（`measured`）に、**初めて shaping した段落を入れない**。初回の段落は 1 個だけの作業領域（scratch）に置き、key の hash を「一度見た」集合に記録する。同じ key を 2 度目に shaping する時（scratch から外れた後）に cache に入れる。計測 cache は段落が保持されていた（＝その text は前にも使われた）時だけ結果を入れる。
- 静的な text（一度測って、見える行だけ一度描く）：見えない行の段落は保持されない。見える行は描画で 2 度目になり保持される。
- 同じ label の部品（3000 個の "x"）：2 個目から段落が hit し、3 個目から計測 cache が hit する。
- 折り返す段落を窓の resize で測り直す：1 回目の resize で 2 度目になり保持され、以後は今と同じ（段落 cache の上限 512 の範囲で shaping し直さない）。

**変更**（`crates/arvane-text/src/layouts.rs`）：`:176-202` の `Shaped` を次に置き換える（`use std::collections::{HashMap, HashSet};`）。

```rust
/// Shaped layouts by [`Key`]; a glyph's brush is the index of its span
/// plus one (0: the paragraph's style).
///
/// A paragraph is kept from its second shaping on: the first time a key is
/// shaped its layout stays in `scratch` only (until another first-time
/// paragraph replaces it) and its hash is remembered in `seen`, so a text
/// measured once and never drawn keeps nothing but the hash.
#[derive(Default)]
pub(crate) struct Shaped {
    current: HashMap<Key, parley::Layout<u32>>,
    previous: HashMap<Key, parley::Layout<u32>>,
    /// The last paragraph shaped for the first time.
    // xtask-allow(state-fields): cache: a one-entry cache of the last first-time paragraph; absent before any
    scratch: Option<(Key, parley::Layout<u32>)>,
    /// The hashes of the keys shaped once and not kept: two generations of
    /// at most `SEEN` each, as the layouts' two generations.
    seen: [HashSet<u64>; 2],
}

/// Whether a layout was kept from an earlier call or shaped by this one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Found {
    Kept,
    Shaped,
}

/// First-time keys remembered per generation.
const SEEN: usize = 8192;

/// The hash `seen` keeps for `key` (one fixed hasher: the maps' own are
/// seeded per map and change when a generation is replaced).
fn key_hash(key: &Key) -> u64 {
    use std::hash::BuildHasher;
    std::hash::BuildHasherDefault::<std::collections::hash_map::DefaultHasher>::default()
        .hash_one(key)
}

impl Shaped {
    /// The layout of `key`: kept, the scratch's, or shaped by `shape` (kept
    /// when the key was shaped once before, else into the scratch).
    pub(crate) fn get_or_insert(
        &mut self,
        key: Key,
        shape: impl FnOnce() -> parley::Layout<u32>,
    ) -> (&mut parley::Layout<u32>, Found) {
        if self.current.contains_key(&key) {
            let layout = self.current.get_mut(&key).expect("invariant: present");
            return (layout, Found::Kept);
        }
        if let Some(layout) = self.previous.remove(&key) {
            return (self.keep(key, layout), Found::Kept);
        }
        if self.scratch.as_ref().is_some_and(|(kept, _)| *kept == key) {
            let (_, layout) = self.scratch.as_mut().expect("invariant: matched");
            return (layout, Found::Kept);
        }
        let hash = key_hash(&key);
        let layout = shape();
        if self.seen[0].remove(&hash) || self.seen[1].remove(&hash) {
            return (self.keep(key, layout), Found::Shaped);
        }
        if self.seen[0].len() >= SEEN {
            self.seen[1] = std::mem::take(&mut self.seen[0]);
        }
        self.seen[0].insert(hash);
        let (_, layout) = self.scratch.insert((key, layout));
        (layout, Found::Shaped)
    }

    /// Keep `layout` in the current generation (a full one becomes the previous).
    fn keep(&mut self, key: Key, layout: parley::Layout<u32>) -> &mut parley::Layout<u32> {
        if self.current.len() >= ENTRIES {
            self.previous = std::mem::take(&mut self.current);
        }
        self.current.entry(key).or_insert(layout)
    }
}
```
module doc（`:1-17`）の「Bounded: ...」の段落に「A paragraph is kept from its second shaping on (a text measured once and never drawn keeps only its key's hash, `Shaped::seen`).」を足す。

`crates/arvane-text/src/text_engine.rs`：
- `:457-480` `layout_at` の戻り値を `(&'a mut parley::Layout<u32>, layouts::Found)` にし、`let (layout, found) = shaped.get_or_insert(key, || {..});`、最後を `(layout, found)`。
- `:273`（`ellipsis_for`）・`:330`（`shape_cached`）：`let (layout, _) = layout_at(..);`。
- `:173-218` `measure_uncached` の戻り値を `(Result<TextMetrics, TextError>, layouts::Found)`：`line_height` の失敗・空の text・不正な span は `(…, layouts::Found::Shaped)`（安価なので計測 cache に入れない）、`:187` は `let (layout, found) = layout_at(..);`、MissingFont は `(Err(TextError::MissingFont), found)`、最後は `(Ok(metrics), found)`。
- `:151-166` `measure`：

```rust
/// Measure a paragraph, caching successes and errors by text, style and
/// width once the paragraph was used before (a text measured once is not
/// kept: its node keeps the answer, `kivact_engine` `TextExtents`).
pub fn measure(&mut self, text: RichText<'_>, style: &ComputedTextStyle, max_width: Option<f64>)
    -> Result<TextMetrics, TextError>
{
    let key = TextKeyRef::new(text, style, max_width, false);
    if let Some(result) = self.cache.measured.get(&key) {
        return result;
    }
    let (result, found) = self.measure_uncached(text, style, max_width);
    if found == layouts::Found::Kept {
        self.cache.measured.insert(key.to_owned(), result.clone());
    }
    result
}
```
- `crates/arvane/src/engine.rs:174-200` の doc（"Cached by text ..."）に「once the paragraph was used before」を足す（API は不変）。

**不変条件**：返す metrics・shaping 結果は cache の有無で同じ（cache は同じ入力の答えを保持するだけ）。`release_caches`（`text_engine.rs:140-149`）は `Cache` を作り直すので scratch と seen も消える。hash の衝突は「初回の段落を保持する」だけで結果は変わらない。

**時間**：初回の text は今と同じく 1 回 shaping する（cache への挿入と key の複製が減る）。見える text は描画で 2 回目の shaping になる（画面 1 枚分、40 行で ≈ 1〜2 ms、最初の frame だけ）。

**単体試験**（`layouts.rs` か `text_engine.rs` の `#[cfg(test)]`、`TextEngine::isolated` で）：(1) 別々の text を 1 回ずつ測ると `paragraphs` の `current.len()` が 0、(2) 同じ text を 2 回（間に別の text を挟んで）測ると 1、(3) 同じ text を 3 回測ると `measured` に 1 件、(4) 結果の metrics は cache の状態によらず等しい。

### 3.4 M3-4 葉の測定記録を taffy の f32 で 32 B/件にする（kivact-layout、P3 M2-13 を置き換え）

**理由**：layout の入出力は taffy の f32 である。記録は f64 の `MeasureRequest`（64 B）と `LeafSize`（32 B）を持つが、layout が読む値は f32 に丸めたものだけ。

**型**（`crates/kivact-layout/src/columns.rs:31-48` を置き換え。`use kivact_types::{Baseline, LeafSize, MeasureRequest, Size, ...}`）：

```rust
/// A measure request as the layout pass asks it: taffy's f32 values as
/// bits (known width, known height, available width, available height).
/// An absent known size and the content-based available spaces are bit
/// patterns no f32 arithmetic produces (NaNs with every payload bit set).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Ask([u32; 4]);

const UNKNOWN: u32 = u32::MAX;
const MIN_CONTENT: u32 = u32::MAX - 1;
const MAX_CONTENT: u32 = u32::MAX;

impl Ask {
    pub(crate) fn new(known: taffy::Size<Option<f32>>, available: taffy::Size<taffy::AvailableSpace>) -> Self {
        let known_bits = |v: Option<f32>| v.map_or(UNKNOWN, f32::to_bits);
        let space_bits = |s: taffy::AvailableSpace| match s {
            taffy::AvailableSpace::Definite(v) => v.to_bits(),
            taffy::AvailableSpace::MinContent => MIN_CONTENT,
            taffy::AvailableSpace::MaxContent => MAX_CONTENT,
        };
        Self([known_bits(known.width), known_bits(known.height), space_bits(available.width), space_bits(available.height)])
    }

    /// The request the leaf measurer is asked (the f32 values widened, as before).
    pub(crate) fn request(self) -> MeasureRequest {
        let known = |b: u32| (b != UNKNOWN).then(|| f64::from(f32::from_bits(b)));
        let space = |b: u32| match b {
            MIN_CONTENT => kivact_types::AvailableSpace::MinContent,
            MAX_CONTENT => kivact_types::AvailableSpace::MaxContent,
            b => kivact_types::AvailableSpace::Definite(f64::from(f32::from_bits(b))),
        };
        MeasureRequest {
            known_width: known(self.0[0]),
            known_height: known(self.0[1]),
            available_width: space(self.0[2]),
            available_height: space(self.0[3]),
        }
    }
}

/// What a leaf measured to, as the layout pass reads it (f32).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Measured {
    width: f32,
    height: f32,
    baseline: f32,
    from: BaselineFrom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BaselineFrom { None, Top, Center }

impl Measured {
    pub(crate) fn of(leaf: &LeafSize) -> Self {
        let (from, baseline) = match leaf.baseline {
            None => (BaselineFrom::None, 0.0),
            Some(Baseline::FromTop(y)) => (BaselineFrom::Top, y as f32),
            Some(Baseline::FromCenter(y)) => (BaselineFrom::Center, y as f32),
        };
        Self { width: leaf.size.width as f32, height: leaf.size.height as f32, baseline, from }
    }

    /// The leaf size these values are (widened back exactly).
    pub(crate) fn leaf(self) -> LeafSize {
        let y = f64::from(self.baseline);
        LeafSize {
            size: Size::new(f64::from(self.width), f64::from(self.height)),
            baseline: match self.from {
                BaselineFrom::None => None,
                BaselineFrom::Top => Some(Baseline::FromTop(y)),
                BaselineFrom::Center => Some(Baseline::FromCenter(y)),
            },
        }
    }
}

/// What a leaf measured to, by request: the last 16 requests (`sizes`), in
/// the f32 the layout reads; ... （既存の doc の続き）
#[derive(Clone, Debug, Default)]
pub(crate) struct Record {
    pub(crate) sizes: Vec<(Ask, Measured)>,
    pub(crate) state: RecordState,
}
```
（`Ask` 16 B + `Measured` 16 B = 32 B/件。`Size::new` が無ければ `Size { width, height }`。）

`crates/kivact-layout/src/tree.rs:102-134`（leaf の measure closure）：

```rust
let mut out = taffy::compute_leaf_layout(inputs, style, resolve, |known, avail| {
    let slot = match entry(t) {
        Entry::Viewport => return taffy::Size::ZERO,
        Entry::Slot(slot) => slot,
    };
    let ask = crate::columns::Ask::new(known, avail);
    let id = tree.id_at(slot).expect("invariant: layout node exists");
    let record = &mut records[index].measured;
    let cached = record.sizes.iter().find(|(r, _)| *r == ask).map(|(_, m)| m.leaf());
    let leaf = cached.unwrap_or_else(|| {
        let leaf = measure.measure(id, ask.request());
        if record.sizes.len() == 16 {
            record.sizes.remove(0);
            record.state = crate::columns::RecordState::Incomplete;
        }
        let measured = crate::columns::Measured::of(&leaf);
        // A leaf is measured under a few requests: keep no spare capacity.
        record.sizes.reserve_exact(1);
        record.sizes.push((ask, measured));
        measured.leaf()
    });
    baseline = leaf.baseline;
    taffy::Size {
        width: known.width.unwrap_or(leaf.size.width as f32),
        height: known.height.unwrap_or(leaf.size.height as f32),
    }
});
```
- 初回の結果も `Measured::of(&leaf).leaf()`（f32 に丸めた値）を使う。記録から読む 2 回目以降と**同じ値**を layout が見るため（f32→f64→f32 は恒等）。`from_taffy`（`tree.rs:358`）は使い手が `:110-111` だけなので削除する（`Ask::request` が置き換える）。
- `compare.rs:48-60` `remeasures_same`：`.all(|(ask, retained)| crate::columns::Measured::of(&measure.measure(id, ask.request())) == *retained)`。
- `compare.rs:66-89` `compare_measurements`：`for (ask, retained) in ..sizes { let request = ask.request(); let rebuilt = measure.measure(id, request); if crate::columns::Measured::of(&rebuilt) != *retained { errors.push(LayoutDifference { id, detail: format!("measurement {request:?}: retained {:?}, rebuilt {rebuilt:?}", retained.leaf()) }) } }`。
- module doc（`tree.rs:17-18`）に「recorded in the f32 the layout reads」を足す。

**意味の変化**：`remeasures_same` は「layout が読む f32 で等しい」かを比べる。f64 では違うが f32 では等しい内容の変化は、今は layout をやり直し、変更後はやり直さない。layout の結果は同じ（taffy は f32 しか読まない）なので正しい。

**単体試験**（`kivact-layout` の既存 test の隣）：`Ask::new(..).request()` が f32 を広げた値と等しい（known None / MinContent / MaxContent / Definite(−1.5) を含む）、`Measured::of(&l).leaf()` を再度 `Measured::of` すると等しい、text 1 個の記録の容量が件数と等しい。

### 3.5 M3-5 CoreNode の稀な field を box にする（kivact-core、P3 M2-8 の 6. を置き換え）

**型**（`crates/kivact-core/src/view/tree.rs:290-343`）：

```rust
pub(crate) struct CoreNode {
    /// Owner whose disposal removes the node.
    pub owner: Owner,
    /// （既存の `at` の doc）
    pub at: Option<&'static Location<'static>>,
    /// Event handlers, if any.
    pub handlers: Option<Box<Handlers>>,
    pub(crate) exit_layout: crate::view::ExitLayout,
    /// The effect that last set each dynamic prop (diagnostics::why).
    pub(crate) origins: Vec<(Prop, EffectId)>,
    // xtask-allow(state-fields): independent: per-node resources few nodes have; created by the first of them
    /// What only some nodes have (references, exit callback, observation, presence, semantic origins, DevTools layers).
    pub(crate) rare: Option<Box<CoreRare>>,
}

const _: () = assert!(std::mem::size_of::<CoreNode>() <= 64, "CoreNode grew: put the new field in CoreRare");

/// The per-node resources few nodes have (`CoreNode::rare`).
#[derive(Default)]
pub(crate) struct CoreRare {
    pub(crate) references: Vec<Signal<crate::view::ref_data::RefState>>,
    pub(crate) exit_complete: Option<kivact_runtime::Handler<()>>,
    // xtask-allow(state-fields): independent: a per-node resource; created only when the node has one
    /// The graph sources node references read (`ViewNodes::observe_node`).
    pub(crate) observation: Option<NodeObservation>,
    // xtask-allow(state-fields): independent: a per-node resource; created only when the node has one
    /// Created only when a reference observes this node's lifetime.
    pub(crate) presence: Option<Signal<kivact_types::Presence>>,
    pub(crate) semantic_origins: Vec<&'static Location<'static>>,
    /// The style layers the node was mounted with, for the DevTools.
    #[cfg(feature = "devtools")]
    pub(crate) style_layers: Vec<crate::view::style_layer::StyleLayer>,
}

impl CoreNode {
    /// The rare resources, created on first use.
    pub(crate) fn rare_mut(&mut self) -> &mut CoreRare {
        self.rare.get_or_insert_default()
    }
    pub(crate) fn observation(&self) -> Option<NodeObservation> {
        self.rare.as_ref().and_then(|r| r.observation)
    }
    pub(crate) fn presence(&self) -> Option<Signal<kivact_types::Presence>> {
        self.rare.as_ref().and_then(|r| r.presence)
    }
    pub(crate) fn semantic_origins(&self) -> &[&'static Location<'static>] {
        self.rare.as_ref().map_or(&[], |r| &r.semantic_origins)
    }
    #[cfg(feature = "devtools")]
    pub(crate) fn style_layers(&self) -> Vec<crate::view::style_layer::StyleLayer> {
        self.rare.as_ref().map_or_else(Vec::new, |r| r.style_layers.clone())
    }
}
```
`CoreNode::new`（`:326-342`）：`CoreNode { owner, at: None, handlers: None, exit_layout: Default::default(), origins: Vec::new(), rare: None }`。
`origins` は style の effect を持つ全 node が書く（ほぼ全 styled node）ので本体に残す。

**全ての読み手・書き手**（db8d679b の行）：

| 場所 | 変更 |
|---|---|
| `view/tree.rs:479-480` `observe_node` | `if let Some(observation) = node.observation() {` |
| `view/tree.rs:493` | `node.rare_mut().observation = Some(observation);` |
| `view/tree.rs:504-509` `presence` | `let owner = node.owner; Some(*node.rare_mut().presence.get_or_insert_with(|| owner.signal(initial, at)))` |
| `view/tree.rs:671` `set_scene` | `self.get(id).and_then(CoreNode::observation)` |
| `view/tree.rs:683-685` `set_style_layers` | `if !layers.is_empty() || node.rare.is_some() { node.rare_mut().style_layers = layers; }` |
| `view/tree.rs:715` `remove` | `for reference in record.rare.iter().flat_map(|r| r.references.iter()) {` |
| `view/tree.rs:753` `remove_engine_subtree` | `let callback = node.rare.as_deref_mut().and_then(|r| r.exit_complete.take());` |
| `view/tree.rs:792` `set_exit` | `let rare = node.rare_mut(); rare.exit_complete = Some(match rare.exit_complete.take() { .. });` |
| `view/node_ref.rs:170-171` | `.rare_mut().references.push(state.bound);` |
| `view/node_ref.rs:220`、`:621` | `.and_then(CoreNode::observation)`（`use crate::view::tree::CoreNode;`） |
| `view/node_ref.rs:736` | `ViewState::tree(&rt).get(target.node)?.observation()` |
| `view/mount/create.rs:82` | `if exit_complete.is_some() { n.rare_mut().exit_complete = exit_complete; }` |
| `view/mount/semantics.rs:56`（P3 M2-7 後の `bind` 内） | `node.rare_mut().semantic_origins = patches.iter().map(|p| p.at).collect();` |
| `client/dispatch/delivery.rs:50` | `.and_then(CoreNode::presence)` |
| `client/dispatch/delivery.rs:296`、`client/dispatch/text.rs:89` | `.and_then(CoreNode::observation)` |
| `client/dispatch/env.rs:113` | `node.observation()` |
| `devtools/styles.rs:230` | `Some((node.style_layers(), node.owner, node.at))` |
| `diagnostics/why.rs:111-112` | `n.origins.clone(), n.semantic_origins().to_vec(),` |

P3 M2-8 の 6.（`observation: Option<Box<NodeObservation>>` と `.as_deref().copied()`）は**行わない**（observation は `CoreRare` の中で box 済み）。M2-8 の 1.〜5. はそのまま。

**不変条件**：`rare` は最初の稀な資源で作られ、node の削除まで残る（空に戻っても消さない：再作成の繰り返しを避ける）。`remove` が references を Detached にする順序・`exit_complete` を取り出す時点は今と同じ。

### 3.6 M3-6 HostNode の送信済み報告を box にする（kivact-engine）

`crates/kivact-engine/src/tree/node.rs:20-38`：`sent_layout`・`sent_scroll`・`sent_image`（`:28-31`）を削除し、次に置き換える。

```rust
    /// What was last published about it: only an observed, scrolling or
    /// image node publishes (`Model::report_geometry`, `Model::report_images`).
    pub(crate) sent: Option<Box<SentReports>>,
```
```rust
/// The last values published to the client about one node.
#[derive(Clone, Debug, Default)]
pub(crate) struct SentReports {
    pub(crate) layout: kivact_types::cell::Sent<LayoutReport>,
    pub(crate) scroll: kivact_types::cell::Sent<ScrollReport>,
    /// The last published loading status of an `Image` node's asset.
    pub(crate) image: kivact_types::cell::Sent<kivact_types::ImageStatus>,
}
```
- `:40-43` の assert を `std::mem::size_of::<HostNode>() <= 312`（index 8・properties 240・scenes 16・scroll 16・sent 8・place 8・shift 16 = 312）。
- `:173-175` `HostNode::new`：`sent: None,`。
- `crates/kivact-engine/src/model/interact/geometry.rs:791`：`if n.sent.get_or_insert_default().scroll.update(state).is_some() {`。
- `geometry.rs:813`：`&& n.sent.get_or_insert_default().layout.update(crate::tree::node::LayoutReport(..)).is_some()`。
- `crates/kivact-engine/src/model/interact/sync.rs:299`：`if n.sent.get_or_insert_default().image.update(status).is_some() {`。
- `crates/kivact-engine/src/tree/node_tree.rs:439` `set_observed`：`if let Some(sent) = n.sent.as_deref_mut() { sent.layout = Default::default(); }`。

**不変条件**：報告の比較と送信は今と同じ（`Sent` の規則）。報告しない node は box を持たない。

### 3.7 M3-7 OwnerEntry の稀な field を box にする（kivact-runtime、P3 の (b)）

`crates/kivact-runtime/src/reactive/data.rs:319-337`：`objects`・`consts`・`cleanups`・`contexts`・`on_error` を削除し、`pub(crate) extra: Option<Box<OwnerExtra>>,` を `nodes` の後に置く。

```rust
/// What only some owners have: objects, constants, cleanups, contexts and
/// an error handler (`OwnerEntry::extra`, created by the first of them).
#[derive(Default)]
pub(crate) struct OwnerExtra {
    pub(crate) objects: Vec<Key>,
    pub(crate) consts: Vec<Key>,
    pub(crate) cleanups: Vec<Box<dyn FnOnce()>>,
    pub(crate) contexts: Vec<(TypeId, Rc<dyn Any>)>,
    pub(crate) on_error: Option<Rc<dyn Fn(UiError)>>,
}

impl OwnerEntry {
    /// The rare members, created on first use.
    pub(crate) fn extra_mut(&mut self) -> &mut OwnerExtra {
        self.extra.get_or_insert_default()
    }
}

const _: () = assert!(std::mem::size_of::<OwnerEntry>() <= 88, "OwnerEntry grew: put the new member in OwnerExtra");
```

**全ての使用箇所**：

| 場所 | 変更 |
|---|---|
| `owner.rs:69-78` 作成 | `nodes: Vec::new(), extra: None,`（5 つの初期化を削除） |
| `owner.rs:137` | `o.extra_mut().cleanups.push(f);` |
| `owner.rs:158-165` `provide_context` | `let contexts = &mut o.extra_mut().contexts; let old = contexts.iter().position(|(t, _)| *t == tid).map(|i| contexts.remove(i)); contexts.push((tid, value)); old` |
| `owner.rs:180` `find_context` | `if let Some((_, v)) = o.extra.as_ref().and_then(|x| x.contexts.iter().find(|(t, _)| *t == tid)) {` |
| `owner.rs:192` | `Some(o) => o.extra_mut().on_error.replace(handler),` |
| `owner.rs:259` | `Some(o) => (o.extra.as_deref_mut().map(|x| std::mem::take(&mut x.cleanups)).unwrap_or_default(), o.created_at),` |
| `owner.rs:279-280` | `o.extra.as_deref_mut().map(|x| std::mem::take(&mut x.objects)).unwrap_or_default(),` と同様に `consts` |
| `owner.rs:450` | `.map(|o| o.extra_mut().consts.push(key))` |
| `object.rs:44-47` | `let objects = &mut o.extra_mut().objects; objects.push(key); objects.len() as u32 - 1` |
| `object.rs:126-133` | `owners.get_mut(owner).and_then(|o| o.extra.as_deref_mut()).and_then(|x| { if index < x.objects.len() && x.objects[index] == key { x.objects.swap_remove(index); x.objects.get(index).copied() } else { None } })` |
| `report.rs:30` | `if let Some(h) = o.extra.as_ref().and_then(|x| x.on_error.as_ref()) {` |

`extra` は dispose まで残す（`take` は中身だけ）。dispose の順序（cleanups → nodes・objects・consts の解放の round）は変えない。

### 3.8 M3-8 Semantics を Rc で共有する（kivact-engine、(a) の確定形）

**(a)「疎な表現」を採らない理由**：`Semantics<R>` は 40 の pub field を持つ公開型で、arvane（a11y 変換）・kivact-style（canvas region）・engine・core・DevTools の 33 ファイルが field を直接読む。疎にすると全読み手が accessor に変わり、公開型の分かりやすさ（field を見れば分かる）を損なう。一方、K2 の部品では Semantics の値が**部品ごとに同じ**（text は M3-1 で無くなり、button は role・focusable・actions が全 button で等しい）。名前が行ごとに違う部品（`checkbox(format!("Row {i}"))` 等）でも 416 B は box 1 個で、node の他の固定費（layout 列 ≈ 0.7 KB）より小さい。よって公開型は変えず、engine の保存だけを共有にする（利用者の API は不変）。

**変更**（P3 M2-5 の後に、同じ仕組みへ 1 種類足す）：
- `crates/kivact-engine/src/contract/node.rs:58-60`：`pub semantics: Option<std::rc::Rc<kivact_types::Semantics<FocusTarget>>>,`（P3 M2-5 が他の 4 つを `Rc` にしたのと同じ）。
- P3 M2-5 の `model/recent.rs`：`RecentProps` に `pub(crate) semantics: Recent<kivact_types::Semantics<kivact_types::FocusTarget>>,`、`impl SharedProp for kivact_types::Semantics<kivact_types::FocusTarget> { fn recent(p: &mut RecentProps) -> &mut Recent<Self> { &mut p.semantics } }`。
- `crates/kivact-engine/src/host/write/typed.rs:45-47`：

```rust
fn semantics(p: &mut NodeProperties) -> &mut Semantics<FocusTarget> {
    // A shared value is copied before one field of it changes.
    std::rc::Rc::make_mut(p.semantics.get_or_insert_default())
}
```
- `typed.rs:254-256`：`Semantics: set_semantics(v: Semantics<FocusTarget>) => shared(Some(v), |p| Some(&mut p.semantics)), mut = Mut::NONE;`（P3 M2-5 が足す `@store … shared(..)`）。
- 読み手（`tree/node.rs:231` の `as_deref()`、`kivact-core/src/view/tree.rs:533` の `as_deref()`）は変更不要。

**注意**：field ごとの setter（`set_role`・`set_name` 等、`typed.rs:222-265`）は `set_prop` の比較の前に `semantics(p)` を呼ぶため、共有中の値を等しい値で書いても複製が起きる。これらの setter を使うのは試験（`kivact-test/src/screen/accessibility_tests.rs:55-56`）だけで、core は `Semantics` を丸ごと書く（P3 M2-7）。正しさは変わらない。

### 3.9 M3-9 layout の NodeStyle を Rc で共有する（kivact-layout、(c) の確定形 1）

**型**（`crates/kivact-layout/src/style/node.rs`）：`NodeDisplay`（`:34`）、`GridArea`（`:73`）、`NodeStyle`（`:80`）の derive に `PartialEq` を足す（calc の値は handle の pointer で比べられ、calc を持つ node 同士は共有されない。正しい）。

**列**（`crates/kivact-layout/src/columns.rs`）：
- `:110` `pub(crate) styles: Vec<Rc<NodeStyle>>,`（`use std::rc::Rc;`）。
- `NodeLayoutValues` に追加：
```rust
    /// The default style every new or reset slot shares.
    pub(crate) default_style: Rc<NodeStyle>,
    /// The last distinct styles built from layout props: equal ones are shared (M3-9).
    pub(crate) recent_styles: kivact_types::cell::Recent<NodeStyle>,
```
- `Default`（`:139-157`）：`let default_style = Rc::new(NodeStyle::default());` を作り `styles: vec![Rc::clone(&default_style)], default_style, recent_styles: Default::default(),`。
- `push`（`:218`）：`values.styles.push(Rc::clone(&values.default_style));`、`reset`（`:243`）：`values.styles[index] = Rc::clone(&values.default_style);`。

**書き手**（`crates/kivact-layout/src/tree.rs`）：
- `:674-676` `rebuild_style`：`let root = self.values.styles[index].position == ..;`（deref で可）、`self.values.styles[index] = self.values.recent_styles.share(style);`。
- `:993`：`if Rc::make_mut(&mut values.styles[index]).set_min_width_floor(floor as f32) {`。
- `:1005`：`let style = Rc::make_mut(&mut self.values.styles[index]);`。
- `:1028`：`Rc::make_mut(&mut self.values.styles[index]).set_size_bounds(min, max);`。
- `:1041`：`let style = Rc::make_mut(&mut values.styles[usize::from(root)]);`。
（`make_mut` は共有中なら複製してから変える。overlay・root の style は他と共有されていても、その node だけが変わる。）

**読み手**：
- `LayoutPass.styles`（`:61`）：`&'a [Rc<NodeStyle>]`。
- `:85`：`self.item_styles.get(&index).unwrap_or(&*self.styles[index])`。
- `:99`：`let style: &NodeStyle = &styles[index];`。
- `:376`、`:509`：`&*self.styles[usize::from(t)]`。`:406`、`:443`、`:531`、`:736`、`:1108` は field の読みで deref により変更不要（型の不一致が出たら `(*..)` を足す）。
- `:1066` `scale_item_styles(styles: &[Rc<NodeStyle>], ..)`、`:1082`：`.or_insert_with(|| (*styles[index]).clone())`。
- `:828-842` `with_pass` の分解（`let NodeLayoutValues { styles, .., never_rounded } = values;`）：全 field を列挙する形のまま、末尾に `default_style: _, recent_styles: _,` を足す。

**不変条件**：共有された `NodeStyle` はその場で変えない（全ての書換えは `Rc::make_mut` か `share` の差し替え）。`NodeLayoutColumns` の clone（比較用の独立した domain）は `Rc` を共有し、片方の書換えは `make_mut` で分かれる。

### 3.10 M3-10 `performed` を子を持つ node だけの疎な表にする（kivact-layout、(c) の確定形 2）

`performed`（最後の full layout の入力）を読むのは、absolute の子を持つ親を単独で layout し直す `relayout_parents`（`tree.rs:786`）と比較（`compare.rs:135`）だけで、どちらも**子を持つ node** の値しか読まない。

- `columns.rs:120`：`pub(crate) performed: SlotMap<taffy::LayoutInput>,` と型を定義：
```rust
/// Values of a few slots, by layout position (`slot + 1`).
pub(crate) type SlotMap<V> =
    std::collections::HashMap<u64, V, std::hash::BuildHasherDefault<kivact_types::IdHasher>>;
```
- `:149`：`performed: SlotMap::default(),`。`:224`（push）の行を削除。`:249`（reset）：`values.performed.remove(&(index as u64));`。
- `tree.rs:71`：`performed: &'a mut crate::columns::SlotMap<LayoutInput>,`。
- `tree.rs:472-474`：
```rust
// Only a parent is laid out again alone (`relayout_parents`): a leaf keeps none.
if inputs.run_mode == RunMode::PerformLayout && self.child_count(t) > 0 {
    self.performed.insert(usize::from(t) as u64, inputs);
}
```
- `tree.rs:786`：`let performed = self.values.performed.get(&(at as u64)).copied();`。
- `compare.rs:115`：`self.values.performed.clear();`、`compare.rs:135`：`if let Some(input) = before.performed.get(&(at as u64)).copied() {`。

### 3.11 採らない案（理由）

| 案 | 効果（text、KiB） | 採らない理由 |
|---|---|---|
| 段落 cache の上限を 256→32 に下げる（P7-a/b の F） | −0.34 | 有界 cache を K2 の割り算のために縮める。折り返す段落が 64 個を超える画面（チャット等）で resize ごとに全段落を shaping し直す。M3-3 が同じ効果を副作用なしで得る |
| 葉の taffy::Cache を持たない | −0.37 | 葉の多い親（3000 行の列）を layout し直すたびに全葉の `compute_leaf_layout` と記録の検索が走る（≈1〜1.5 ms／回、K5 row update の予算 16.96 ms を食う）。`clear_caches_up` の「既に空なら止まる」（`tree.rs:726`）も葉で特別扱いが要る。本設計で目標に届くので不要 |
| Semantics の疎な表現 | （M3-8 と同等以下） | 3.8 節 |
| arvane の計測 cache に node の text を一切入れない（P7-a/b の近似） | −0.28 | 同じ label の部品（3000 個の "x"）を毎回測り直す（mount +5〜9 ms）。M3-3 の「2 度目から」で同じ効果 |
| ChildCursor・NodeLayoutState・Ink の f32 化・圧縮 | 各 −0.02〜−0.06 | 効果が小さく、layout・paint の hot path を変える |
| 定数の list の行を owner・signal なしで mount | −0.25 | 一般化の価値はあるが K2 の測り方に寄せた最適化になる。目標に届くので不要 |

---

## 4. 見積り（目標との対比）

### 4.1 試作と確定設計の差

P7-d は M3-1〜M3-10 を試作した状態（近似：M3-8 は engine の Model に置いた ring、M3-11 の `Recent` は各所の `Vec<Rc<T>>`、M3-4 の encoding は符号 bit を落とした版）。P7-d に無いもの：P3 の M2-5（engine prop の共有）・M2-6・M2-7・M2-9〜M2-12・M2-14（P3 4.1 節のとおり P1 にも無い）。text に効く差は M2-14（graph の slot +16 B、行の source 1 個）だけで **+0.02 KiB**。

### 4.2 K2 item text

| | text（KiB、同じ回の計測） |
|---|---|
| P1（同じ回） | 6.23〜6.27 |
| P7-c（同じ回） | 3.42〜3.46 |
| P7-d（同じ回の P1 は 6.26） | **2.93〜3.00**（中央値 2.98） |
| 確定設計（P7-d + M2-14 の +0.02） | **≈ 3.0**（P1 = 6.26 の回の値） |
| 同上を P3 の回（P1 = 6.49）に換算（+0.23） | **≈ 3.2** |
| 目標 | ≤ 3.64（余裕 12〜17%） |

- B0 14.57 からの倍率：≈ 4.5〜4.9 倍（目標 4 倍）。
- 内訳（heap 3,006 B/item、P7-d）：layout 列 992（taffy::Cache 384・final 96・unrounded 92・NodeStyle の Rc 8 ほか、2 冪の容量込み）、NodeStore 490（HostNode 312 + links 40、容量込み）、arena 346（owner 88 + source 160、容量込み）、paint 207、葉の測定 192（TextExtents 64 + 記録 4 件 128）、多数の子の index 175、CoreNode 100、行の Vec と内容 87、measure 列 65、`Model::sync` 63、段落の 2 度目採用の hash と描画済み段落 ≈ 40、その他 ≈ 250。
- 誤差：同じ回の 3 回で ±0.04、日ごと ±0.25（H-P7-1）。

### 4.3 button / row（P3 4.2 節の見積りに積む）

P3 の確定設計の見積り：button ≈ 24.4、row ≈ 33.4（P1 実測に、P1 に無い P3 項目の見積りの削減を足したもの）。本設計の追加分は「P7-d と P1 の差（同じ日の計測：P1 button 35.26〜35.41、row 46.33〜46.39）」から、P3 が既に数えた重複分（M2-13・M2-5b・CoreNode：button −1.4、row −2.6、P3 4.2 節の値）を引いて求める。

| 項目 | P1 | P7-d | 差 | うち P3 が数えた分 | 本設計の追加 | 確定設計の見積り | 目標 | 余裕 |
|---|---|---|---|---|---|---|---|---|
| button | 35.3 | 30.3 | −5.0 | −1.4 | **−3.6** | 24.4 − 3.6 ≈ **20.8** | ≤ 24.75 | **16%**（P3 は 1.4%） |
| row | 46.4 | 37.0 | −9.4 | −2.6 | **−6.8** | 33.4 − 6.8 ≈ **26.6** | ≤ 34.45 | **23%**（P3 は 3%） |

- button の追加分の内訳（heap からの帰属）：NodeStyle 共有 3 node × 264 B ×1.37 ≈ −1.1、Semantics 共有（root）−0.4、label の text の M3-1・M3-2（computed text・Semantics）−0.9、HostNode の報告 3 × 152 B −0.45、performed の疎化・OwnerExtra・LeafMeasure −0.3、CoreRare（origins を本体に残した確定形）−0.2、計 ≈ −3.4（実測 −3.6）。
- row は button の分に加え、行の text（−2.0：M3-1〜M3-3）、row の node（NodeStyle・報告・CoreNode）−0.6、行の段落の 2 度目採用 −0.4。
- **3000 行 idle WS**（`MODE=row`、CPU）：P3 の見積り 136,400 KiB − 6.8 × 3000 ≈ **116,000 KiB**（CPU の目標 158,479 ✓）。default renderer はこの画面で CPU と同じ（P3 4.2 節）なので **default の目標 129,989 も ✓**（ws0 35.7 MB のまま：26.6 × 3000 + 35,700 ≈ 115,500）。P3 の H2 は本設計で解消する。
- 誤差：P3 の見積り部分（M2-5・M2-9〜M2-11・M2-14 等、未試作）の ±1.5 KiB と本設計の実測部分の ±0.3。

### 4.4 その他の KPI への影響

- K5 mount n3000：allocation が行あたり ≈ 12 個減る（Semantics・computed text・TextExtents の外箱・測定記録の再確保・計測 cache の key・CoreNode の Vec 群・OwnerEntry の Vec）。増えるもの：`Recent::share` の比較（直近の hit から比べるので同じ部品の連続では 1 回、NodeStyle 264 B・ComputedTextStyle 464 B の比較で ≈ 0.1 µs/node）、`performed` の hash 挿入（子を持つ node の PerformLayout ごと ≈ 20 ns）、最初の frame で見える text の 2 度目の shaping（≈ 1〜2 ms）。差し引き ±10 ms 以内と見る（P3 の見積り 780〜800 ms に対し目標 813.8）。checkpoint で測る。
- K3/K4/K5 の更新系：layout の hot path の変化は NodeStyle の 1 段の間接参照だけ（同じ部品の style は 1 つの領域を共有するので cache の局所性はむしろ良い）。text の内容変化は f32 の記録で比べる（計算量は同じ）。
- K6（binary size）：影響は無視できる。

---

## 5. 正しさ・リスク・E2E

| 項目 | リスク | 既存 E2E・試験での担保 | 追加すべき試験 |
|---|---|---|---|
| M3-1 text の role | DevTools・query が text を見失う（`Reader::semantics` の補完で防ぐ） | `view::announcements_publish_in_separate_persistent_live_regions`（`get_by_role(Role::Text, ..)`）、`kivact-ui-soft` consumer `foundation`（`get_by_role(Role::Text, "AM")`）、`devtools::a_devtools_client_inspects_drives_and_reviews_a_test_app` | engine 単体：`Reader::semantics` が role 無しの Text node に `Role::Text` を返し、editor を持つ node には返さない。E2E `view::plain_rich_and_string_texts_are_text_role_and_named_by_content`：`text("a")`・`rich_text(..)`・`"b"` の 3 つが `get_by_role(Role::Text, name)` で見つかり、`stats().live_reactive` が text 3 個で増えない |
| M3-2 style 共有 | 共有値の書換え（経路は無い） | `text_style::*`、`text_primitives::*`、`i18n::*`（family の解決） | 不要 |
| M3-3 段落の 2 度目採用 | 見える text の初回描画で shaping が 2 回（時間のみ）。ellipsis を持つ text は 1 回の計測で 2 度目になり計測 cache に入る（今と同じ量） | `text_primitives::a_paragraph_draws_the_pixels_of_the_text_it_measured`、`text_effects::*`、`launch_fonts::*`（font 登録で cache 作り直し） | arvane-text 単体（3.3 節の 4 つ） |
| M3-4 f32 の記録 | f32 で等しい内容変化で layout を省く（結果は同じ） | `layout::*`、`text_primitives::measured_inline_boxes_wrap_and_share_the_text_baseline_in_draw`、`layout_check::*`（`compare_measurements` の診断） | layout 単体（3.4 節） |
| M3-5 CoreRare | 削除時の reference の切り離し漏れ | `node_ref::*`、`focus::unmounted_node_is_not_focused_and_a_rebound_ref_follows_the_new_node`、`presence::*`、`presence_modes::*`、`diagnostics::*`、`devtools::*` | 不要 |
| M3-6 SentReports | observe の切替えで報告が出ない | `node_ref::scroll_state_follows_the_scroll_position`、`scroll_timeline::*`、`image_semantics::*`、`assets::*` | 不要 |
| M3-7 OwnerExtra | dispose の順序 | runtime 単体、`disposal::*`、`resource_reclaim::closing_three_thousand_rows_returns_the_live_counts_to_the_baseline`、`environment::*`（context） | runtime 単体：context・cleanup・on_error・object・const の各 1 つだけを持つ owner の dispose で全て解放 |
| M3-8 Semantics 共有 | field ごとの setter の複製（試験だけ） | `relations::*`、`interaction_toggle::*`、`kivact-test` の `accessibility_tests` | 不要 |
| M3-9 NodeStyle 共有 | overlay・root の style の書換えが他の node に漏れる | `overlay::*`（最小幅・fit）、`viewport::*`、`windows::*`、`layout_units::*`（calc） | layout 単体：同じ props の 2 node が `Rc::ptr_eq`、片方に `set_min_width_floor` 後に他方は不変 |
| M3-10 疎な performed | absolute の子の親の単独 layout | `layout::*` の absolute を含む試験、`presence::exit_size_collapses_the_node_before_it_is_removed` | 不要 |

**境界条件**：
- 削除・unmount：`CoreRare`・`SentReports`・`OwnerExtra` は持ち主と一緒に捨てる。layout slot の reset（`columns.rs:235-255`）は `styles` を既定の `Rc` に、`performed` から除く。
- window close：`Recent` の ring（measure 列・layout 列・engine の `RecentProps`）は window の Model と一緒に捨てる。
- 非表示・reduced motion：変更なし。
- font の登録：arvane の `Cache` は作り直し（scratch・seen も消える）、engine は全 text を測り直す（`sync.rs:127-149`、今と同じ）。
- DevTools：a11y tree・query・style 層の表示は今と同じ（3.1 節、`style_layers()`）。

---

## 6. unit 分割案

| unit | 内容 | 所有ファイル | 依存 | 完了時の確認 |
|---|---|---|---|---|
| U-T1 | M3-1 | `kivact-core/src/view/elements/text.rs`、`kivact-engine/src/host/write/reader.rs` | なし | `cargo check -p kivact-core`、`cargo test -p kivact-engine --lib semantics`、E2E（window-lock）`view::announcements_publish_in_separate_persistent_live_regions` |
| U-T2 | M3-11 | `kivact-types/src/cell.rs`、`kivact-engine/src/model/recent.rs` | **U-M3 の後に同じ lane**（recent.rs は U-M3 が作る） | `cargo test -p kivact-types --lib cell`、`cargo check -p kivact-engine` |
| U-T3 | M3-2（M2-5b を置き換え、U-M4 は実施しない） | `kivact-engine/src/measure/state.rs`、`measure/leaf.rs`、`measure/inherited_text.rs` | U-T2 | `cargo check -p kivact-engine`、`cargo clippy -p kivact-engine --all-targets -- -D warnings`、E2E `text_primitives::a_paragraph_draws_the_pixels_of_the_text_it_measured` |
| U-T4 | M3-3 | `arvane-text/src/layouts.rs`、`arvane-text/src/text_engine.rs`、`arvane/src/engine.rs`（doc のみ） | なし | `cargo test -p arvane-text`、`cargo clippy -p arvane-text --all-targets -- -D warnings` |
| U-T5 | M3-4・M3-9・M3-10（M2-13 を置き換え、U-M5 は実施しない） | `kivact-layout/src/columns.rs`、`tree.rs`、`compare.rs`、`style/node.rs` | U-T2 | `cargo test -p kivact-layout`、`cargo clippy -p kivact-layout --all-targets -- -D warnings`、E2E `overlay::` の 1 個（最小幅を使うもの） |
| U-T6 | M3-5（M2-8 の 6. を置き換え） | `kivact-core/src/view/tree.rs`、`view/node_ref.rs`、`view/mount/create.rs`、`view/mount/semantics.rs`、`client/dispatch/{delivery,env,text}.rs`、`devtools/styles.rs`、`diagnostics/why.rs` | **U-M7・U-M8 の後に同じ lane**（同じファイル） | `cargo check -p kivact-core --all-targets`、`cargo check -p kivact-core --features devtools`、E2E `node_ref::scroll_state_follows_the_scroll_position presence::exit_size_collapses_the_node_before_it_is_removed` |
| U-T7 | M3-6 | `kivact-engine/src/tree/node.rs`、`tree/node_tree.rs`、`model/interact/geometry.rs`、`model/interact/sync.rs` | なし | `cargo check -p kivact-engine --all-features`、E2E `node_ref::scroll_state_follows_the_scroll_position` |
| U-T8 | M3-7 | `kivact-runtime/src/reactive/data.rs`、`owner.rs`、`object.rs`、`report.rs` | **U-M11 の後に同じ lane**（data.rs） | `cargo test -p kivact-runtime` |
| U-T9 | M3-8 | `kivact-engine/src/contract/node.rs`、`host/write/typed.rs`、`model/recent.rs` | **U-M3・U-T2 の後に同じ lane** | `cargo check -p kivact-engine --all-features`、`cargo test -p kivact-test --lib accessibility`、E2E `relations::relations_reach_the_accessibility_tree_in_call_order` |

checkpoint（重い確認）：workspace の test と全 E2E、`dev/bench/run.sh` の K2（text・button・row・3000 行）と K5 mount を本機で base と同じ回に計測。`kv_heap MODE=text` で `Measure::measure`（computed text）・`Writer::set ← semantics`・`Generations::insert<TextKey, TextMetrics>` の site が 0 であること。

---

## 7. 既存欠陥（Slim-Fast 実装 79bbd409..db8d679b）

1. **MEM-5（computed text の共有）が text の葉で働かない**：`crates/kivact-engine/src/measure/inherited_text.rs:39-55`。共有は「親が解決済みで等しい」時だけで、親（列・行の Box）は text でも editor でもないので解決されない（`:47` の注記どおり、解決しない）。結果、text ごとに `Rc<ComputedTextStyle>`（480 B）を作る。再現：`kv_heap MODE=text ROWS=3000` で `Measure::measure` が 480 B/item・1 alloc/item。修正：3.2 節。
2. **text() が engine の既定と同じ role を patch で書く**：`crates/kivact-core/src/view/elements/text.rs:28,34,74`。engine は role の無い Text 種を Role::Text とする（`model/semantics.rs:42`）。text ごとに Semantics の box 416 B・`semantic_origins`・意味論の bind が増える。修正：3.1 節。
3. **text の計測値の二重保持**：`arvane-text/src/text_engine.rs:153-166` が node の `TextExtents`（`measure/state.rs:131-136`）と同じ答えを text ごとに保持（285 B/item）。`TextExtents` はさらに style（464 B）を複製（P3 M2-5b が指摘）。修正：3.2・3.3 節。
4. **報告しない node が送信済み報告を持つ**：`crates/kivact-engine/src/tree/node.rs:28-31`（152 B）。報告は scroller・observed・image だけ（`geometry.rs:777-782`、`sync.rs:286-301`）。MEM-7 の「稀な field は box」に反する。修正：3.6 節。
5. **葉の `performed` は書かれるが読まれない**：`crates/kivact-layout/src/tree.rs:472-474` は全 node の PerformLayout で書き、読み手（`:786`、`compare.rs:135`）は子を持つ node だけ。修正：3.10 節。

---

## 8. 設計者（人間）の判断が要る点

- **H-P7-1 WS の日ごとのずれ**：同じ exe・同じ手順で text が 6.23〜6.49 KiB とずれる（±0.25 KiB/item）。本設計の余裕（4.2 節、≈ 0.45〜0.65 KiB）はこのずれより大きいが、判定は checkpoint で base と**同じ回**に測って行う。届かなかった場合の追加策は効く順に：(1) 定数の list の行を owner・signal なしで mount（−0.25）、(2) 葉の taffy::Cache を持たない（−0.37、CPU と引換え）、(3) 段落 cache の上限を下げる（−0.34、resize の CPU と引換え）。
- **H-P7-2 f32 の測定記録**：`remeasures_same` が「layout が読む f32 で等しいか」を比べるようになる（3.4 節）。layout の結果は同じ。設計書の LAY 系の規則に一文足す（「葉の記録は taffy の f32 で持つ」）。
- **H-P7-3 段落の 2 度目採用**（M3-3）：見える text の初回描画で shaping が 1 回増える（画面 1 枚分）。代わりに、一度しか測らない text の段落を保持しない。
- **H-P7-4 Semantics は疎にせず共有**（3.8 節）：公開型は不変。名前が行ごとに違う部品では 416 B/node が残る。
- **H-P7-5 未特定の 63 B/item**：`Model::sync` の中の inline 展開された allocation（1.13 alloc/item、text・button・row 共通）。kv_heap の site 名では特定できなかった。text の予算の 1.7%。必要なら次の段階で `CHAIN` と debug build で調べる。

---

付記：試作の差分（P3 の P1 に本設計の M3-1〜M3-10 の近似を積んだもの、計測用の size test を含む）は `W:\w\kv-mem` の未追跡変更として残す。生データは `W:\w\kv-mem\.p3-measure\p7\`。
