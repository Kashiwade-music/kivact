# V2 再監査 R2（target lane B）：W1 の差戻し 3 件と D3・D7 の残り unit

> **V2.1 修正付属書 D10**（再監査 R2 の成果物、HEAD `2a3b06b8`＝W1 の後に対して書いた）。範囲：W1 の M7・T8・M6 の返戻し、W2〜W4 の M3（T2・T9 を統合）・M6・M8・M9・T1・T3・T5・T6（メモリ）。
> - **優先順位**：本書は付属書 D1〜D8 より優先する。本書の unit の節の「修正」は、作業指示書の行の「本書の追加の指示」と同じ扱い。本書と V2 設計書 4 章（DEC）が食い違う場合は V2 設計書が正（V2 設計書 4 章の DEC-18 の名前は `while_open(factory)`）。
> - 本書の「全体」節の wave・lane の組み直しは、作業指示書（V2.1）に反映済み。実施順・所有ファイルは作業指示書が正。
> - 文中の worktree（`W:/w/kv-audit-*`）・scratchpad の path は監査の試作の場所で、実装に必要ではない（試作の差分は参考で、実装は本書の文に従う）。

対象：HEAD `2a3b06b8`。読取りと、使い捨ての worktree（`W:/w/kv-audit-R2`、`W:/w/kv-audit-R2b`、どちらも最後に削除）での compile・lint・E2E の試し。

## 0. 本書で確かめたこと（以下の判定の根拠）

| 試したこと | 結果 |
|---|---|
| M7 の修正（`CheckFold`）・T8 の marker・M6 の再設計（下の節のコードそのまま）を HEAD に当てて `cargo check -p kivact-core --all-targets`、`cargo xtask check --no-write --only state-fields --crate kivact-core`／`--crate kivact-runtime`、`cargo xtask check --no-write --crate kivact-core` | すべて成功（error 0。行数・引数の上限も通る） |
| M7-fix・M6 だけを当てて E2E `style_paint::*`、`relations::*`、`interaction_toggle::each_kind_publishes_its_role_and_the_checked_state_without_a_value_text`、`environment::advanced_semantics_publish_roles_and_reactive_states` | 10 件すべて成功。**`style_paint::paint_only_style_tracks_size_signals_theme_and_removal` の「paint does not evaluate style again」も通る** |
| 続けて M8・M9・T1（本書の最終形）を当て、`cargo fmt`、`cargo xtask check --no-write --crate kivact-core`（error 0）、`cargo clippy -p kivact-core --all-targets -- -D warnings`（kivact-core は 0。engine の HEAD の警告は下記）、E2E `style_paint:: relations:: node_ref:: focus:: hover:: parts:: scroll_timeline:: view:: diagnostics:: devtools:: interaction_toggle:: ui_standard:: canvas snapshot` | 75 件中 74 件成功。失敗した 1 件 `diagnostics::why_follows_the_prop_to_the_write_through_a_memo` の実際の出力は、T1 の節の新しい `WHY` と一字一句同じ（期待値の変更で通る）。途中で見つけて本書に反映した誤り：M8 の `focus_value` の module 循環、`require_observation` の `request_turn` 漏れ、T1 の role の読み手 3 つ |
| HEAD に M3（+T2+T9）だけを当てて E2E `diagnostics:: view:: presence::exit_size_collapses_the_node_before_it_is_removed hover:: relations:: text_primitives:: text_style:: ui_theming:: interaction_toggle::` | 41 件中 40 件成功。失敗は `diagnostics::why_follows_the_prop_to_the_write_through_a_memo` だけで、**HEAD で既に失敗している**（W1 の M1 で node id が 5 → 4。T1 の節で期待値を直す） |
| M3（M2-4・M2-5）＋ T2（`Recent` を最初から `kivact_types::cell` に）＋ T9（Semantics の共有）を HEAD に当てて `cargo check -p kivact-engine --all-features --all-targets`、`cargo check -p kivact-core -p kivact-test --all-targets --all-features` | すべて成功。**`kivact-engine` の外の読み手は 1 行も直さずに compile する**（`as_deref()`・`as_ref()` と auto-deref）。`NodeProperties: Send` は要求されない（`Rc` のまま） |
| state-fields の lint の規則（`dev/xtask/src/state_fields/{classify,check,locals}.rs`、`dev/xtask/src/check_source.rs` の `marker_before`） | marker は **field の doc comment と `#[..]` より上**の行に置く（field の span は doc comment から始まり、`marker_before` はそこから上へ `///`・`#[..]`・他の marker だけを飛ばして探す）。数える field が 1 つしかない struct には marker を書かない（使われても audit の行にならない）。**数えない field の上の marker は `unused marker` の error**。試して確認：D7 3.0 の `Recent` の 2 本の marker（doc の下）はどちらも `unused marker`、`kivact-layout/src/columns.rs` の `Counted.unfiltered` の marker を doc の下へ動かすと `unused marker` |
| 行数（`vendor/xtask-core/crates/xtask-source/src/lines.rs`、token のある行を数える） | `crates/kivact-layout/src/tree.rs` は約 985 code 行（上限 1000）。T5 の変更は tree.rs で正味 −8 行前後なので通る（11 節） |

全 unit に共通の追加の指示（作業指示書 0.2 に足す）：
- **marker の位置**：`// xtask-allow(state-fields): ...` は、その field の `///` doc comment と `#[...]` の**上**に置く。付属書のコードで marker が doc comment の下にあるもの（D3 3.6 `InStylePaint`、D3 3.8 の `observed` など）は、本書で差し替えた。
- **付属書の 5 章「追加すべき E2E／試験」の扱い**：各 unit の節に「足す／足さない」を書いた。書いていない試験は足さない（作業指示書 0.2 の 3）。V2 設計書 0.3 のとおり、`#[cfg(test)]` の単体試験は engine・runtime・xtask にだけ足す。**kivact-types・kivact-layout には単体試験を足さない**（D7 3.0・3.4 の単体試験の指示は取り消す。担保は各 unit の E2E）。

---

## M7-fix（付属書 D3 3.7、新設、W2 lane D の先頭）

W1 の差戻し：`crates/kivact-core/src/view/mount/semantics.rs:14` `Plan`（checked, indeterminate）の state-fields error（commit `a087054c`）。

### 判定：修正あり

`checked` と `indeterminate` は「最後に見た `checked` の値」と「最後に見た `indeterminate` の値」で、fold の途中では独立に動く（後の `indeterminate(false)` が前の `checked` を見せる）。ただし結果に効く状態は 5 つしか無い（同値な組を畳むと：未設定／`indeterminate(false)` だけ／`checked(b)`／`indeterminate(true)`（下の `checked` が無い）／`indeterminate(true)`（下に `checked(b)`））。この 5 状態を enum にして型で閉じる。marker は使わない。`checked_state` 関数は不要になり削除する（呼出し元はこのファイルだけ）。

### 修正

`crates/kivact-core/src/view/mount/semantics.rs` を次のとおり変える（試して compile・lint を確認済み）。

1. `struct Plan` を次にする：

```rust
struct Plan {
    base: Semantics<FocusTarget>,
    check: CheckFold,
    rest: Box<[Rest]>,
}

/// What the `checked` and `indeterminate` patches fold to, in layer order:
/// an `indeterminate(true)` wins until a later `indeterminate(false)`
/// uncovers the last `checked` value.
#[derive(Clone, Copy, Default)]
enum CheckFold {
    /// Neither input yet: the patches' own `checked` stands.
    #[default]
    Unset,
    /// Only `indeterminate(false)`: no checked state.
    Cleared,
    /// The last `checked` value.
    Checked(bool),
    /// `indeterminate(true)`, over the last `checked` value if there was one.
    Indeterminate(Option<bool>),
}

impl CheckFold {
    /// The fold after a `checked(value)` patch.
    fn checked(self, value: bool) -> Self {
        match self {
            CheckFold::Indeterminate(_) => CheckFold::Indeterminate(Some(value)),
            CheckFold::Unset | CheckFold::Cleared | CheckFold::Checked(_) => {
                CheckFold::Checked(value)
            }
        }
    }

    /// The fold after an `indeterminate(value)` patch.
    fn indeterminate(self, value: bool) -> Self {
        match (self, value) {
            (CheckFold::Indeterminate(under), true) => CheckFold::Indeterminate(under),
            (CheckFold::Checked(checked), true) => CheckFold::Indeterminate(Some(checked)),
            (CheckFold::Unset | CheckFold::Cleared, true) => CheckFold::Indeterminate(None),
            (CheckFold::Indeterminate(Some(checked)) | CheckFold::Checked(checked), false) => {
                CheckFold::Checked(checked)
            }
            (CheckFold::Indeterminate(None) | CheckFold::Unset | CheckFold::Cleared, false) => {
                CheckFold::Cleared
            }
        }
    }

    /// The fold after `input`'s patch (other inputs leave it as it is).
    fn after(self, input: &SemanticInput) -> Self {
        match input {
            SemanticInput::Checked(source) => self.checked(source.get()),
            SemanticInput::Indeterminate(source) => self.indeterminate(source.get()),
            _ => self,
        }
    }

    /// Write the fold over the patches' `checked` (nothing while `Unset`).
    fn apply(self, semantics: &mut Semantics<FocusTarget>) {
        semantics.checked = match self {
            CheckFold::Unset => return,
            CheckFold::Cleared => None,
            CheckFold::Checked(checked) => Some(CheckedState::from_bool(checked)),
            CheckFold::Indeterminate(_) => Some(CheckedState::Indeterminate),
        };
    }
}
```

2. `Plan::new` の中：`let (mut checked, mut indeterminate) = (None, None);` を `let mut check = CheckFold::Unset;` に、`index < split` の枝の `match &patch.input { … }` を `check = check.after(&patch.input);` に、最後の構築を `Plan { base, check, rest: rest.into_boxed_slice() }` にする。
3. `Plan::resolve` の本体を次にする：

```rust
        let mut semantics = self.base.clone();
        let mut check = self.check;
        for patch in self.rest.iter() {
            (patch.apply)(&mut semantics);
            check = match &patch.input {
                RestInput::Checked(source) => check.checked(source.get()),
                RestInput::Indeterminate(source) => check.indeterminate(source.get()),
                RestInput::Other => check,
            };
        }
        check.apply(&mut semantics);
        semantics
```

4. `pub(super) fn checked_state(..)` を削除する。
5. 正しさ（旧の `(checked, indeterminate)` との対応）：`(None,None)`=Unset、`(None,Some(false))`=Cleared、`(Some(b),None)` と `(Some(b),Some(false))`=Checked(b)、`(None,Some(true))`=Indeterminate(None)、`(Some(b),Some(true))`=Indeterminate(Some(b))。遷移と出力（旧 `checked_state` と「どちらかが Some なら上書き」）は全て一致する。
6. **M7 が取り除いた E2E を戻す**（D3 5 章が名前を挙げた試験。M7 の実装者が作業指示書 0.2 の 3 を誤読して消した）。`dev/tests/kivact-public-api/tests/e2e/relations.rs` の末尾に次を足す（M7 の退避 patch と同じ内容）。失敗したら STOP（期待値を変えない）。

```rust
#[test]
fn constant_semantics_publish_once_and_keep_no_publisher() {
    let (screen, (plain, named)) = mount(|| {
        let plain = signal(false);
        let named = signal(false);
        (
            (plain, named),
            column((
                show(plain, || node()),
                show(named, || node().role(Role::Button).name("Save")),
            )),
        )
    });
    let before_plain = screen.stats().live_reactive;
    plain.set(true);
    screen.run_until_idle();
    let plain_cost = screen.stats().live_reactive - before_plain;
    plain.set(false);
    screen.run_until_idle();
    let before_named = screen.stats().live_reactive;
    named.set(true);
    screen.run_until_idle();
    let named_cost = screen.stats().live_reactive - before_named;
    assert_eq!(named_cost, plain_cost, "constant semantics add no reactive node");
    screen.get_by_role(Role::Button, "Save");
    assert!(screen.errors().is_empty());
}
```

（`relations.rs` の既存の `use` に `mount`・`show`・`signal` が無ければ足す：`use crate::support::mount;`。`kivact::prelude::*` で `show`・`signal`・`column`・`node`・`Role` は入る。）

7. 確認：`cargo check -p kivact-core --all-targets`、`cargo xtask check --no-write --only state-fields --crate kivact-core`（error 0）、`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- relations:: interaction_toggle::each_kind_publishes_its_role_and_the_checked_state_without_a_value_text environment::advanced_semantics_publish_roles_and_reactive_states`。

### 所有ファイルの変更
`crates/kivact-core/src/view/mount/semantics.rs`、`dev/tests/kivact-public-api/tests/e2e/relations.rs`。

### wave・lane・順序の変更
新しい unit「M7-fix」を **W2 lane D の最初**に置く（M8・T1・T6 より前。semantics.rs は 5.3 節の順 M7 → T6）。

---

## T8-fix（付属書 D7 3.7、新設、W2 lane E の先頭）

W1 の差戻し：`crates/kivact-runtime/src/reactive/data.rs:331` `OwnerEntry.extra` の state-fields error（commit `636c4794`）。

### 判定：修正あり（marker）

`OwnerEntry` は phase の field `phase: OwnerPhase`（名前が `Phase` で終わる enum）を持つため、数える field が `extra` 1 つでも「phase と stored の組合せ」で error になる。`extra` は稀な member の入れ物（box）で、最初の登録で作られ、dispose は中身を `take` するだけで box は残す（`owner.rs` の `dispose_one`）。Disposing 中の `provide_context`・`set_error_handler`・object の登録でも作られる（`extra_mut` は phase を見ない）。`phase` の 2 状態 × `extra` の有無の 4 通りは全て起きる。型の形で閉じる案（常に box を持つ・Runtime の別表にする）はメモリか hash の費用を増やすので採らない。D8 3.3 の規則（`option:` は caller・builder が与える値に限る）により `option:` ではなく `independent:` にする。

### 修正

`crates/kivact-runtime/src/reactive/data.rs` の `OwnerEntry` の `pub(crate) extra: Option<Box<OwnerExtra>>,` の直前の行に、次の 1 行を足す（この field に doc comment は無い）：

```rust
    // xtask-allow(state-fields): independent: the box of the rare members, created by the first object, constant, cleanup, context or error handler registered on the owner and kept until the entry is removed; either phase holds it or not (disposal empties it, never drops it)
```

確認：`cargo xtask check --no-write --only state-fields --crate kivact-runtime`（error 0、試して確認済み）、`cargo test -p kivact-runtime`。

### 所有ファイルの変更
`crates/kivact-runtime/src/reactive/data.rs`（この 1 行だけ）。

### wave・lane・順序の変更
新しい unit「T8-fix」を **W2 lane E の S7 の前**に置く（S7 の所有ファイル `reactive/{handles,graph}.rs`・`machine.rs`・`flow/list.rs` とは重ならない）。

---

## M6（付属書 D3 3.6、W2 lane D、W1 で未実施）

### 判定：修正あり（設計をやり直す。付属書 3.6 の `InStyle`／`InStylePaint` は使わない）

W1 の差戻しの 3 つの理由はすべて、付属書が「callback を持つ box の paint を style の effect の中で記録する」設計にしたことから来ている：(1) callback だけが読む signal の変化で `.style(..)` の closure 全体が再実行される（E2E `style_paint::paint_only_style_tracks_size_signals_theme_and_removal` の `assert_eq!(evaluations.get(), before, "paint does not evaluate style again")` に反する。性能でも style の compile の不要な再実行になる）、(2) `InStylePaint` の 2 つの Cell（state-fields）、(3) `text_measurer` の型の誤り。

**新しい設計（遅延の draw effect）**：「局所 paint の機構（source・size signal・layout observer・draw effect）を、paint callback を最初に持った時に作る」。作る機構は、今の HEAD が全 styled box に mount 時に作っているものと**全く同じ**（`draw::bind` をそのまま使う）。callback の読む signal・size・theme は今と同じく draw effect が追跡するので、style の closure は callback の signal で再実行されない（試験の期待どおり、DEC-11 の「style の compile の再実行」も起きない）。callback を一度も持たない styled box は notifier・size・draw effect・observer を持たない（D3 の目的のメモリ効果は同じ）。一度作った機構は node の owner と一緒に消える（callback が無くなっても残る。Draw layer は今と同じく空になる）。

DEC-11（「style の compile も再実行されることを許容」）はこの設計では不要になる。設計書 4 章の DEC-11 の行に「M6 は遅延の draw effect にしたため、callback の signal で style は再実行されない」と書き戻す（W8）。

付属書 3.6 の「InStyle の経路」「`InStylePaint`」「`report_unsupported` を `pub(super)`」「`theme_value`」の指示は**全て取り消す**。D3 5 章の追加 E2E `style_paint::a_paint_callback_acquired_later_draws_at_the_node_size` は**足さない**（既存の `paint_only_style_tracks_size_signals_theme_and_removal` が「callback 無しで始まり、後から callback を得て、node の大きさで描く」を既に検証する）。

### 修正（そのまま実装する。試して `cargo check`・lint・xtask 全規則が通ることを確認済み）

1. `crates/kivact-core/src/view/mount/facts.rs`：
   - import を `use std::cell::RefCell;`、`use std::panic::Location;`、`use std::rc::Rc;`、`use kivact_engine::contract::NodeType;`、`use kivact_runtime::{Notifier, Owner};`、`use kivact_style::CompiledStyle;` にする。
   - `NodeFacts.local_paint` の上の marker 行 `// xtask-allow(state-fields): option: a builder input carried to mount; absent means none` を**削除**し、field を `pub(super) local_paint: LocalPaint,` にする（数えない型になるので marker を残すと `unused marker`）。
   - 次を足す：

```rust
/// How a node records its style's paint callback (`Style::paint`).
#[derive(Clone, Default)]
pub(crate) enum LocalPaint {
    /// Not a styled box (or an editor): a paint callback is reported as unsupported.
    #[default]
    None,
    /// A draw effect of the node records the callback from this source: one
    /// made at mount for a styled box with its own draw layers or canvas
    /// regions (`draw::bind`), or by the first callback (`OnFirstPaint`).
    WithDraw(LocalPaintSource),
    /// A styled box without draw layers that no callback reached yet: the
    /// first one creates the source, the size observer and the draw effect
    /// (`draw::bind_style_paint`), and the node is `WithDraw` from then on.
    OnFirstPaint,
}

impl LocalPaintSource {
    /// A source of `owner` with no paint yet.
    pub(super) fn new(owner: Owner, at: &'static Location<'static>) -> Self {
        Self {
            value: Rc::new(RefCell::new(None)),
            revision: owner.notifier(at),
        }
    }
}
```

2. `crates/kivact-core/src/view/mount/resolve.rs` の `resolve`：
   - `use super::facts::{LocalPaint, LocalPaintSource, NodeFacts};` にし、使われなくなる `use std::cell::RefCell;` と `use std::rc::Rc;` を消す。
   - `let attached = attach_parts(&mut common, cx, id, at);` の直前に `// Read before `attach_parts` takes the regions.` と `let declares_regions = !common.regions.is_empty();` を足す。
   - `let local_paint = (..).then(|| LocalPaintSource { .. });` と `let draw = prepare_draw(.., local_paint.clone(), at);` を次に置き換える（`prepare_draw` と `NodeFacts { kind, focus_stop, local_paint }` は今のまま）：

```rust
    let styled_box = !common.style.is_empty() && kind == NodeType::Box && editing.is_none();
    let own_draw = common.draw.is_some()
        || common.draw_over.is_some()
        || common.draw_with_layout.is_some()
        || declares_regions;
    let local_paint = match (styled_box, own_draw) {
        (true, true) => LocalPaint::WithDraw(LocalPaintSource::new(cx.owner, at)),
        (true, false) => LocalPaint::OnFirstPaint,
        (false, _) => LocalPaint::None,
    };
    let draw_paint = match &local_paint {
        LocalPaint::WithDraw(source) => Some(source.clone()),
        LocalPaint::None | LocalPaint::OnFirstPaint => None,
    };
    let draw = prepare_draw(&mut common, cx, attached.as_ref(), draw_paint, at);
```

3. `crates/kivact-core/src/view/mount/draw.rs`：`fn report_unsupported(` の直前に次を足す（`bind` と `DrawInput` は変えない。`report_unsupported` は private のまま）：

```rust
/// Bind the draw effect of a styled box without draw layers when its style
/// first has a paint callback (`LocalPaint::OnFirstPaint`): the layout size
/// observer and the Draw layer's effect that `bind` makes at mount for a
/// box with draw layers. `local_paint` already holds the callback.
pub(super) fn bind_style_paint(
    cx: &Mcx<'_>,
    id: NodeId,
    local_paint: LocalPaintSource,
    at: &'static Location<'static>,
) {
    let size = cx.owner.signal(Size::new(0.0, 0.0), at);
    if let Some(handlers) = ViewState::tree_mut(cx.rt).handlers_mut(id) {
        handlers.observe_layout(move |_, s| size.set(s));
    }
    let input = DrawInput {
        below: None,
        over: None,
        with_layout: None,
        local_paint: Some(local_paint),
        regions: None,
        size,
    };
    bind(cx, id, input, at);
}
```

4. `crates/kivact-core/src/view/mount/style.rs`：
   - `use std::cell::RefCell;` を足し、`use super::facts::{LocalPaint, LocalPaintSource, NodeFacts};` にする。
   - `NodeStyle.local_paint` の marker 行と doc を消し、`/// How the style's paint callback is recorded.` と `pub local_paint: LocalPaint,` にする（`#[derive(Default)]` のまま。既定は `LocalPaint::None`。`root.rs` の `..NodeStyle::default()` は変えない）。
   - `bind_style` の `let owner = cx.owner;` の直後に `let window = cx.window;` と `let local_paint = RefCell::new(local_paint);` を足す。
   - effect の中の `let has_paint = resolved.has_retained_paint();` から `if let Some(source) = &local_paint { … }` の終わりまでを、次の 6 行に置き換える：

```rust
        let mcx = Mcx {
            rt: &rt,
            owner,
            window,
        };
        feed_local_paint(&mcx, id, &local_paint, &resolved, &reports, at);
```

   - `#[derive(Default)] struct StyleReports {` の直前に次の関数を足す：

```rust
/// Hand the style's paint callback to the node's draw effect (see
/// [`LocalPaint`]). The callback's own reads are tracked by that effect,
/// so a signal only the callback reads never runs the style again.
fn feed_local_paint(
    cx: &Mcx<'_>,
    id: NodeId,
    local_paint: &RefCell<LocalPaint>,
    resolved: &kivact_style::CompiledStyle,
    reports: &StyleReports,
    at: &'static Location<'static>,
) {
    let has_paint = resolved.has_retained_paint();
    if has_paint && matches!(*local_paint.borrow(), LocalPaint::OnFirstPaint) {
        let source = LocalPaintSource::new(cx.owner, at);
        *source.value.borrow_mut() = Some(Box::new(resolved.clone()));
        super::draw::bind_style_paint(cx, id, source.clone(), at);
        *local_paint.borrow_mut() = LocalPaint::WithDraw(source);
    }
    match &*local_paint.borrow() {
        LocalPaint::None => {
            if has_paint && reports.leaf_paint.note() {
                cx.rt.report(
                    Some(cx.owner),
                    UiError::new(
                        kivact_runtime::ErrorKind::Unsupported,
                        "local paint requires a box node; apply it to the containing node",
                        Some(at),
                    ),
                );
            }
        }
        LocalPaint::OnFirstPaint => {}
        LocalPaint::WithDraw(source) => {
            // A style can acquire paint later; observe only when it first does.
            // Never disable observation needed by a NodeRef or another consumer.
            if has_paint {
                ViewState::tree_mut(cx.rt).set_observe(id, true);
            }
            let changed = source
                .value
                .borrow()
                .as_ref()
                .map_or(has_paint, |old| !old.same_retained_paint(resolved));
            if changed {
                *source.value.borrow_mut() = has_paint.then(|| Box::new(resolved.clone()));
                source.revision.notify_from_render();
            }
        }
    }
}
```

   要点（実装者が変えないこと）：新しい source には draw effect を作る**前に**値を入れる（draw effect の最初の実行で描く。直後の `changed` は false になり notify しない）。`RefCell` の borrow は `bind_style_paint` の呼出しの間は持たない（上のコードのとおり `matches!(*local_paint.borrow(), ..)` は式の終わりで borrow を返す）。draw effect は `cx.owner`（node の owner）に作るので、style effect の再実行では消えない。
5. 不変条件（doc に書かなくてよい、確認用）：`Style::paint` の callback は、size・theme・callback の読む signal の変化で **draw effect だけが**記録し直す。callback が無くなれば style effect が source を空にして notify し、Draw layer が空になる。自前の draw／region を持つ node は今と同じ（mount 時に `WithDraw`）。
6. 確認：`cargo check -p kivact-core --all-targets`、`cargo clippy -p kivact-core --all-targets -- -D warnings`、`cargo xtask check --no-write --crate kivact-core`、`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- style_paint::`。

### 所有ファイルの変更
付属書のとおり `kivact-core/src/view/mount/{resolve,style,draw,facts}.rs`。

### wave・lane・順序の変更
W2 lane D の M7-fix の次、M8 の前。

---

## M8（付属書 D3 3.8・3.9、W2 lane D）

### 判定：修正あり

HEAD との食違い：
- 付属書は `NodeRefState` に `observed: Cell<bool>` を足すが、HEAD には既に `observes_layout: Cell<LayoutObservation>`（`NotRequested`／`Requested`、`rect()` が engine の layout 報告を要求したか）がある。付属書の形だと「observation の要求」と「layout 報告の要求」の 2 つの状態が並び、しかも `rect()` の時は両方が立つ（含意）。付属書の `projection:` marker も X4（FINAL-2）で `owned_by` の初期化を別の書き手と数えられうる。→ **1 つの enum `RefObservation { Unrequested, Sources, SourcesAndLayout }` にまとめて型で閉じる**（単調に上がるだけの要求）。
- `rect_value`／`scroll_value` の遅延作成の marker は、D8 3.3 の規則で `option:` は誤り（実行時に最初の読み手が作る）。`cache:` にする（正本は `bound` と node の observation、作成の契機は最初の読み手）。
- `text_truncated()`（`node_ref.rs`）と `scroll_state()`（`node_status.rs`）と `view_progress`（`scroll_timeline.rs`）は既に `rect()` を呼ぶので、付属書の「先頭で `require_observation`」は不要（`rect()` の中で行われる）。追加が要るのは `image_status()` と `scroll_progress()` だけ。
- 付属書 3.9 の `focus_value` は `focused()` の memo の前処理を複製する。前処理を関数 `focus_target` に出して両方から使う。また `focus_value(&self) -> Value<Focus>` を `node_ref.rs` に置くと `node_ref` ↔ `part_data` の module の循環（xtask の error）になる。`focus_memo(of)`（写しを呼出し側が渡す）にする。
- `hovered` の Region の枝：付属書の式は `within` が持っていた「region がまだ scene にある」（`bound_node` → `live_target`）の判定を落としている。`bound_node(app, node)` を通す形にして今と同値にする。
- 手順 6（`CoreNode.observation` の box）は行わない（V2 設計書 5.1。T6 が置き換える）。付属書 6 章の所有ファイルのうち `view/tree.rs`・`client/dispatch/*` は M8 では変えない。

### 修正（付属書 3.8・3.9 の代わりに、次をそのまま行う）

1. `crates/kivact-core/src/view/ref_data.rs`：
   - `NodeRefState` を次にする（`bound` の doc は今のまま）：

```rust
pub(crate) struct NodeRefState {
    pub(crate) owner: Owner,
    /// (今の doc のまま)
    pub(crate) bound: Signal<RefState>,
    /// What the readers called so far need on every node the ref binds to.
    pub(crate) observation: Cell<RefObservation>,
    // xtask-allow(state-fields): cache: the rect memo of the binding, created under the ref's owner by the first NodeRef::rect call; absent until then
    pub(crate) rect_value: Cell<Option<Value<Option<Rect>>>>,
    // xtask-allow(state-fields): cache: the scroll offset memo of the binding, created under the ref's owner by the first NodeRef::scroll_offset call; absent until then
    pub(crate) scroll_value: Cell<Option<Value<crate::geometry::Point>>>,
}
```

   - `enum LayoutObservation { NotRequested, Requested }` を削除し、次に置き換える：

```rust
/// What a reference's readers need on every node it binds to, from the first
/// such reader on (never lowered).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RefObservation {
    /// No reader of the node's geometry or status yet: a bound node gets no
    /// `NodeObservation` (a region binding always does).
    Unrequested,
    /// The node's graph sources (`NodeObservation`): scroll offset, image
    /// status, scroll progress.
    Sources,
    /// The sources and the engine's layout reports of the node (`NodeRef::rect`).
    SourcesAndLayout,
}
```

2. `crates/kivact-core/src/view/state.rs:28`：`pub(crate) use super::ref_data::{Bound, LayoutObservation, NodeRefState, RefState};` の `LayoutObservation` を `RefObservation` にする。`node_ref.rs:13` の import も同じ。
3. `crates/kivact-core/src/view/node_ref.rs`：
   - `owned_by`：`let rect_value = ..;`・`let scroll_value = ..;` を消し、`NodeRefState { owner, bound, observation: Cell::new(RefObservation::Unrequested), rect_value: Cell::new(None), scroll_value: Cell::new(None) }`。
   - `impl NodeRef` に追加：

```rust
    /// Keep what `need` asks on every node this ref binds to, from now on:
    /// the node's graph sources (`NodeObservation`) and, for
    /// `SourcesAndLayout`, the engine's layout reports of it. The node bound
    /// now gets them at once; a node bound later gets them in `bind_target`.
    pub(crate) fn require_observation(
        &self,
        need: RefObservation,
        at: &'static Location<'static>,
    ) {
        let (rt, s) = self.state(at);
        if s.observation.get() >= need {
            return;
        }
        s.observation.set(need);
        let Some(bound) = s.bound.try_get_untracked().and_then(RefState::bound) else {
            return;
        };
        let mut tree = ViewState::tree_mut(&rt);
        let _ = tree.observe_node(bound.node(), at);
        if need == RefObservation::SourcesAndLayout {
            tree.set_observe(bound.node(), true);
        }
        drop(tree);
        // The next layout pass reports the node (a call outside a turn).
        rt.request_turn();
    }
```

   （`rt.request_turn()` は必須：試験から mount の後に `rect()` を初めて呼ぶと、turn が来ず layout pass が走らないため `rect` が `None` のままだった。試して確認。`request_turn` は purity を検査しない。）

   - `bind_target` の `// One host-owned callback pair serves every ref, …` から `.push(state.bound);` までを次にする：

```rust
        // One host-owned callback pair serves every ref that reads the
        // observation; a region binding always needs it (`target_value` and
        // `focused` read its scene revision).
        let observation = state.observation.get();
        if observation > RefObservation::Unrequested || matches!(bound, Bound::Region { .. }) {
            if ViewState::tree_mut(rt).observe_node(id, at).is_none() {
                return;
            }
        } else if ViewState::tree(rt).get(id).is_none() {
            return;
        }
        if observation == RefObservation::SourcesAndLayout {
            ViewState::tree_mut(rt).set_observe(id, true);
        }
        ViewState::tree_mut(rt)
            .get_mut(id)
            .expect("invariant: the bound node exists")
            .references
            .push(state.bound);
```

   - `rect()` の本体：

```rust
        let at = Location::caller();
        self.require_observation(RefObservation::SourcesAndLayout, at);
        let (_, s) = self.state(at);
        if let Some(value) = s.rect_value.get() {
            return value;
        }
        let value = s.owner.with_setup(|| resolved_rect(self.app, s.bound));
        s.rect_value.set(Some(value));
        value
```

   - `scroll_offset()` の本体：

```rust
        let at = Location::caller();
        self.require_observation(RefObservation::Sources, at);
        let (_, state) = self.state(at);
        if let Some(value) = state.scroll_value.get() {
            return value;
        }
        let value = state.owner.with_setup(|| resolved_scroll(self.app, state.bound));
        state.scroll_value.set(Some(value));
        value
```

   - `text_truncated()` は変えない（先頭の `let _ = self.rect();` が observation を要求する）。
   - 自由関数を足す（`bound_node` の上）：

```rust
/// The bound target if it is still mounted, tracking the binding and, for a
/// region, its node's scene revision (the region may be gone).
fn focus_target(app: AppId, node: Signal<RefState>) -> Option<FocusTarget> {
    let bound = node.get().bound()?;
    let id = bound.target();
    if id.region.is_some()
        && let Some(rt) = Runtime::get(app)
        && let Some(observation) = ViewState::tree(&rt)
            .get(id.node)
            .and_then(|node| node.observation)
    {
        observation.scene_revision.track();
    }
    bound_node(app, node)
}
```

   - `focused()` の memo を `kivact_runtime::memo(move || focus_target(app, node).is_some_and(|id| keys.selector.is(&id)))` にする（前の setup 部分は今のまま）。
   - `impl NodeRef` に追加（`focused()` の後）。**`node_ref.rs` から `part_data::Focus` を名指さないこと**（`part_data` は `NodeRef` を使うので、名指すと xtask の「module reference cycle among: crate::view::node_ref, crate::view::part_data」の error になる。試して確認した。付属書 3.9 の `focus_value(&self) -> Value<Focus>` はこの理由で使えない）。値への写しは呼出し側（part.rs）が渡す：

```rust
    /// The focus of the bound target as one memo of `of(focus)`: `None`
    /// while the target is not focused, else whether the host asks for a
    /// visible indicator (`focused` and `focus_visible` in one, for a part's
    /// state). Setup only.
    #[track_caller]
    pub(crate) fn focus_memo<T: PartialEq + 'static>(
        &self,
        of: fn(Option<bool>) -> T,
    ) -> Value<T> {
        let at = Location::caller();
        let (rt, s) = self.state(at);
        require_setup("NodeRef::focus_memo");
        let window = current_window(s.owner);
        let keys = ViewState::of(&rt)
            .focus_keys(window)
            .expect("invariant: the node reference's window is open");
        let app = rt.id();
        let node = s.bound;
        drop(rt);
        kivact_runtime::memo(move || {
            let Some(id) = focus_target(app, node) else {
                return of(None);
            };
            if !keys.selector.is(&id) {
                return of(None);
            }
            // The indicator is read only while focused (as `focus_visible`).
            of(Some(keys.focus.get().is_some_and(|key| key.visible)))
        })
    }
```

   - `hovered()`：`let owner_hover = within(app, node, hover);` を消し、memo を次にする：

```rust
        kivact_runtime::memo(move || match node.get().bound() {
            Some(Bound::Region { .. }) => {
                bound_node(app, node).is_some_and(|target| hover.get() == Some(target))
            }
            Some(Bound::Node(_)) => is_within(app, node, hover),
            None => false,
        })
```

   - `fn within(..) -> Value<bool>` を `fn is_within(app: AppId, node: Signal<RefState>, source: Signal<Option<FocusTarget>>) -> bool` に変える：本体は今の memo の closure の中身そのまま（`kivact_runtime::memo(move || { … })` の包みを外す）。doc は「Whether the node `source` holds is the node a ref is bound to (through its node source `node`) or lies in its subtree; read inside a memo.」。
4. `crates/kivact-core/src/view/part.rs`（付属書 3.9 のとおり、具体形）：
   - `let focused = node.focused();` と `let focus_visible = node.focus_visible();` を次に：

```rust
    let focus = node.focus_memo(|focus| match focus {
        None => Focus::None,
        Some(false) => Focus::Focused,
        Some(true) => Focus::FocusedVisible,
    });
```
   - state の memo の `focus: match (focused.get(), focus_visible.get()) { … },` を `focus: focus.get(),` に。
   - mount effect の `let focused = focused.get();` を `let focused = focus.get() != Focus::None;` に。
   - `let mut common = input_observers(input, focused, enabled);` を `input_observers(input, focus, enabled)` に。
   - `fn input_observers` の引数 `focused: Value<bool>` を `focus: Value<Focus>` にし、`focused.get_untracked()` を `focus.get_untracked() != Focus::None` に。
5. `crates/kivact-core/src/view/node_status.rs` `image_status()`：`require_setup("NodeRef::image_status");` の次に `self.require_observation(crate::view::state::RefObservation::Sources, std::panic::Location::caller());`。
6. `crates/kivact-core/src/view/scroll_timeline.rs` `scroll_progress`：`require_setup("scroll_progress");` の次に `scroller.require_observation(crate::view::state::RefObservation::Sources, std::panic::Location::caller());`。（`view_progress` は `rect()` を呼ぶので変えない。）
7. 公開 API（`NodeRef::focused`・`focus_visible`・`rect`・`scroll_offset` の型と意味）は変えない。`rect()`・`scroll_offset()` は setup の外（handler・effect・memo の中・試験）から最初に呼ばれても、`with_setup` で ref の owner に memo を作る（`setup_frame` は追跡も pure も持たない frame なので、外側の memo・effect の依存に入らない。`observe_node` は signal を作るだけで書込みではない）。repo の kit には `rect()` を memo・handler の中で初めて呼ぶもの（`kivact-ui-game/src/surface.rs:114`、`kivact-ui-soft/src/grid.rs:134` など）があり、この経路で動く。
8. D3 5 章の追加 E2E **`node_ref::rect_called_after_binding_follows_the_node` を足す**（bind の後に初めて `rect()` を呼ぶ経路は新しく、既存の試験に無い）。`dev/tests/kivact-public-api/tests/e2e/node_ref.rs` の末尾：

```rust
#[test]
fn rect_called_after_binding_follows_the_node() {
    let (screen, (target, width)) = mount(|| {
        let target = node_ref();
        let width = signal(120.0);
        (
            (target, width),
            node()
                .node_ref(target)
                .style(move || style().width(width.get()).height(40).shrink(0)),
        )
    });
    // Bound at mount, before anything read its geometry.
    let rect = target.rect();
    screen.run_until_idle();
    assert_eq!(rect.get().map(|r| r.width), Some(120.0));
    width.set(160.0);
    screen.run_until_idle();
    assert_eq!(rect.get().map(|r| r.width), Some(160.0));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```

9. 確認：付属書の行の E2E に上の試験と `node_ref::image_status_reports_a_failed_image hover::hovered_follows_the_hit_through_listeners_capture_and_tree_changes focus::unmounted_node_is_not_focused_and_a_rebound_ref_follows_the_new_node parts::` を足す。`cargo xtask check --no-write --only state-fields --crate kivact-core` は error 0（`NodeRefState` は phase `bound` に cache の 2 つ、どちらも marker 付き）。

### 所有ファイルの変更
`kivact-core/src/view/{node_ref.rs, ref_data.rs, state.rs, part.rs, node_status.rs, scroll_timeline.rs}`、`dev/tests/kivact-public-api/tests/e2e/node_ref.rs`。付属書の `view/tree.rs`・`client/dispatch/{delivery,env,text}.rs` は外す（T6 が持つ）。

### wave・lane・順序の変更
なし（W2 lane D、M6 の後）。

---

## M9（付属書 D3 3.10・3.11 の core、W2）

### 判定：修正あり（小）

付属書の設計は HEAD と合う。直すのは書き方と marker：
- `EventHandlers.entries: Option<Rc<Vec<..>>>` は数える field がこれ 1 つなので、付属書の marker（`// xtask-allow(state-fields): option: absent means the list is empty`）は**書かない**。
- 取消しの effect の「定数か」の判定は `kivact_runtime::Prop::value(..).fixed()` を使う（`Value::constant_value` は runtime の `pub(crate)` で core から呼べない）。
- `route_accessibility` の「handler がある action だけ」の条件を具体形にする。

### 修正

1. `crates/kivact-core/src/view/mount/regions.rs`：
   - `impl Registry` に `pub(super) fn declares_regions(&self) -> bool { !self.state.borrow().groups.is_empty() }`（doc：「Whether the node declares canvas regions (a group per declaration).」）。
   - `prepare`：最初に `let declares = !declarations.is_empty();`。reconcile の effect（`registry.changed.track(); registry.reconcile();`）を `if declares { … }` で囲む（コメント「Drivers have lower creation order …」も中へ）。取消しの effect を次で囲む：

```rust
    // The node itself ends its gestures when it is disabled or hidden; constant
    // constraints never change after the mount, before any gesture.
    let constraints = registry.root.constraints;
    let fixed = kivact_runtime::Prop::value(constraints.disabled).fixed().is_some()
        && kivact_runtime::Prop::value(constraints.hidden).fixed().is_some();
    if !fixed {
        let weak = Rc::downgrade(&registry);
        cx.effect(EffectKind::Internal, at, move || { /* 今の本体のまま */ });
    }
```

2. `crates/kivact-core/src/view/mount/part_input.rs` `route_accessibility`：`let custom = …;` の次に `let declares = registry.declares_regions();`。`for action in [ … ] {` の直後に：

```rust
        // Without regions only the node's own handlers can run: route no other action.
        let offered = custom.iter().any(|(candidate, _)| *candidate == action)
            || registry.root.actions.a11y.iter().any(|(candidate, _)| *candidate == action);
        if !declares && !offered {
            continue;
        }
```

   （a11y の dispatch `client/dispatch/a11y.rs` は handler が無ければ何もしないので、外した action は今も何もしていなかった。公開する actions は `attach_parts` が route の前に決めるので変わらない。）
3. `crates/kivact-core/src/view/mount/parts.rs` `constraints`：付属書 3.11 の `ConstFlags` を同じファイルに置く（field の型 `values: Cell<[Option<Value<bool>>; 2]>` は lint が数えない）。`owner: kivact_runtime::Owner`、`Value` は `kivact_runtime::Value`、`use std::cell::Cell;` を足す。`let flags = ConstFlags { owner: cx.owner, at, values: Cell::new([None, None]) };` を `let value = …` の前に置き、`let value = |source: InputProp<bool>| match source.fixed() { Some(b) => flags.get(b), None => source.into_value(cx.owner, at) };`。
4. `crates/kivact-core/src/view/handler_layers.rs`：付属書 3.10 のとおり（marker なし）。置換は次の全 16 箇所：`register`・`caller` の `self.entries.partition_point` → `self.list().partition_point`、`Rc::make_mut(&mut self.entries)` →  `self.list_mut()`（`register`・`merge_caller`・`caller`・`replace_caller`）、`later.entries.iter()`（2 箇所）・`part.entries.iter()`（2 箇所）→ `.list().iter()`、`router` → `entries: Some(Rc::new(vec![..]))`、`only` → 集めた `Vec` を `entries: (!entries.is_empty()).then(|| Rc::new(entries))`、`is_empty` → `self.list().is_empty()`、`deliver` の `source.list.entries.iter()` → `source.list.list().iter()`。`Default` は `entries: None`。`list()` は `impl<E> EventHandlers<E>`、`list_mut()` は `impl<E: 'static> EventHandlers<E>` に置く。
5. 確認：付属書の行のとおり。加えて `cargo xtask check --no-write --crate kivact-core`。

### 所有ファイルの変更
なし（付属書のとおり）。

### wave・lane・順序の変更
**W2 lane F に移す**（W2 の lane F は G2 を W1 に前倒ししたので空き）。M9 の所有ファイルは lane D の M7-fix・M6・M8・T1・G3 のどれとも重ならず、署名の変更（`Registry::declares_regions` の追加、`EventHandlers` の private な field）は他の unit に影響しない。lane D を短くする。

---

## T1（付属書 D7 3.1、W2 lane D）

### 判定：修正あり

1. **role を「書かれた semantics」から読む所が 4 つある**：engine の `Reader::semantics`（DevTools の a11y tree・role の query・node 詳細・overlay・tree）、core の `ViewNodes::role`（`diagnostics::why` の見出し）、kivact-test の `facts.rs` の `role`（`Screen::snapshot()` の `SemanticNode.role` と `Element::role` の fallback、`Locator` の role）。付属書は `Reader::semantics` の「semantics が無い Text node」だけを補うので、(a) 他の patch（`.test_id(..)`・`.hidden(..)` など）を持つ text が Text でなくなり、(b) `snapshot()` と `why` が text を Text と言わなくなる。試すと E2E `view::for_each_keeps_row_owners_uses_minimal_moves_and_reports_duplicates`・`view::virtual_list_mounts_only_the_visible_window`（`snapshot()` の `role == Some(Role::Text)` で text を集める）と `diagnostics::why_follows_the_prop_to_the_write_through_a_memo`（見出しが `Some(Text)` になる）が失敗した。→ **「Text 種で editor でない node は、role を書いていなければ `Role::Text`」を engine の `NodeProperties::role()` 1 つにして、4 つの読み手がそれを使う**（engine の `semantic_role` と同じ規則。公開の a11y tree は今も engine が補う）。
2. **既存の E2E の期待値の衝突**：`diagnostics::why_follows_the_prop_to_the_write_through_a_memo` は **HEAD で既に失敗している**（W1 の M1 が tuple の fragment を無くし node id が 5 → 4 になった。HEAD に M3 だけを当てた worktree で確認：見出しが `node 4 Text "doubled: 4"`）。加えて T1 で text の role patch が無くなり「semantics patch declared at …」は `.test_id("doubled")` の 1 行になる。性能とメモリの目的（text ごとの Semantics box・patch・origins を無くす）を優先して**期待値を変える**（下の 4.）。
3. 付属書の完了の確認 `cargo test -p kivact-engine --lib semantics` は該当する試験が 0 件（engine の単体試験は `model/events.rs` だけ）。engine の単体試験の harness が無いので D7 5 章の engine 単体試験は**足さない**。代わりに E2E `view::plain_rich_and_string_texts_are_text_role_and_named_by_content` を**足す**。

### 修正（試して compile・E2E を確認済み。0 節）

1. `crates/kivact-core/src/view/elements/text.rs`：付属書 3.1 のとおり（3 箇所の `.role(Role::Text)` を削除、import から `Role` を外す、module doc に 1 文）。
2. `crates/kivact-engine/src/contract/node.rs` の `impl NodeProperties` の先頭に：

```rust
    /// The node's role: the one its semantics state, else `Role::Text` for a
    /// text node that is not a native editor (its kind's role, which the
    /// platform tree also gives it, `model::semantics::semantic_role`).
    pub fn role(&self) -> Option<kivact_types::Role> {
        self.semantics.as_deref().and_then(|s| s.role).or_else(|| {
            (matches!(self.kind, NodeKind::Text(_)) && self.editing.is_none())
                .then_some(kivact_types::Role::Text)
        })
    }
```

3. 読み手：
   - `crates/kivact-engine/src/host/write/reader.rs` の `pub fn semantics` を次にする（付属書のコードの代わり）：

```rust
    /// The semantics of `id` as written, with its role as
    /// [`NodeProperties::role`](crate::contract::NodeProperties::role) gives
    /// it (a `Text` node that states none has `Role::Text`).
    pub fn semantics(&self, id: NodeId) -> Option<kivact_types::Semantics<FocusTarget>> {
        self.with_model(id, |model| {
            let properties = &model.tree().get(id)?.properties;
            let role = properties.role();
            match properties.semantics.as_deref() {
                Some(semantics) => Some(kivact_types::Semantics {
                    role,
                    ..semantics.clone()
                }),
                None => role.map(|role| kivact_types::Semantics {
                    role: Some(role),
                    ..Default::default()
                }),
            }
        })
        .flatten()
    }
```

   - `crates/kivact-core/src/view/tree.rs` の `ViewNodes::role` の本体を `self.with_node(id, |node| node.and_then(NodeProperties::role))`（`NodeProperties` は import 済み）。
   - `crates/kivact-test/src/facts.rs` の `role` の本体を同じく `self.with_node(id, |node| node.and_then(NodeProperties::role))`（import 済み）。
   - `text(..).role(r)` を明示したものはその role のまま。M3 の後で `semantics` が `Rc` でも `as_deref()` で同じ。
4. `dev/tests/kivact-public-api/tests/e2e/diagnostics.rs` の `const WHY` を次にする（見出しの node id を 4 に、`semantics patch` の行を 1 行に。他の assert は変えない）：

```rust
const WHY: &str = "node 4 Text \"doubled: 4\"
  Content set by the effect created at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
    reads memo i32 created at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
      reads signal i32 created at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C, last written at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C (turn 4)
  semantics patch declared at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C";
```

5. `dev/tests/kivact-public-api/tests/e2e/view.rs` の import `use kivact::view::{Handled, KeyEvent, LiveRegion, RowHeight};` を `use kivact::view::{Handled, KeyEvent, LiveRegion, RowHeight, rich_text, span};` にし（`rich_text`・`span` は prelude に無い）、末尾に：

```rust
#[test]
fn plain_rich_and_string_texts_are_text_role_and_named_by_content() {
    let screen = render(|| {
        column((
            text("plain"),
            rich_text(|| vec![span("ri"), span("ch")]),
            "string",
            text("tagged").test_id("tagged"),
        ))
    });
    screen.get_by_role(Role::Text, "plain");
    screen.get_by_role(Role::Text, "rich");
    screen.get_by_role(Role::Text, "string");
    screen.get_by_role(Role::Text, "tagged");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```

6. 確認：`cargo check -p kivact-core -p kivact-engine -p kivact-test --all-targets --all-features`、`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- view:: diagnostics:: devtools:: snapshot`。

### 所有ファイルの変更
付属書の 2 つに `crates/kivact-engine/src/contract/node.rs`（`NodeProperties::role` の追加だけ）、`crates/kivact-core/src/view/tree.rs`（`ViewNodes::role` の本体だけ）、`crates/kivact-test/src/facts.rs`（`role` の本体だけ）、`dev/tests/kivact-public-api/tests/e2e/{diagnostics.rs, view.rs}` を足す。

### wave・lane・順序の変更
なし（W2 lane D、M8 の後）。足したファイルは W2 の他の lane が持たない（`contract/node.rs`・`reader.rs` は W3 の M3・AH2、`view/tree.rs` は W3 の T6 で、wave が違う）。M3 は `contract/node.rs` の `NodeProperties` の field の型を変えるが、`role()` は `as_deref()` を使うので変更不要（`Rc` でも compile する）。

---

## M3（付属書 D3 3.4・3.5 ＋ D7 3.0（T2）＋ D7 3.8（T9）、W3 lane A）

### 判定：修正あり（T2 と T9 を M3 に統合）

- D3 3.5 は `Recent` を `model/recent.rs` に作り、同じ lane の次の T2 がそれを `kivact_types::cell` に移し、さらに次の T9 が `RecentProps` と `typed.rs` と `contract/node.rs` をもう一度変える。同じファイルを 3 回書き直すだけなので、**M3 で最初から D7 3.0 の `Recent`（`kivact_types::cell`）を使い、`Semantics` の共有（D7 3.8）も同時に入れる**。T2・T9 の unit は無くなる。
- `Recent` の marker は**書かない**：`next: usize` は数える field（名前 `next`）が 1 つだけなので組合せの error にならず、`last: usize` は数えない field なので D7 3.0 の `cache:` marker は `unused marker` の error になる。D3 3.5 の `next` の marker も書かない。
- `NodeProperties: Send` の条件（D3 3.5「コンパイラが要求した場合に限り Arc」）は不要：`Rc` で engine・core・test が all-features・all-targets で compile することを確認した。
- 「`kivact-engine` の外で compile error が出たら STOP」（作業指示書 W3 の追加の指示）は残すが、出ないことを確認済み（外の読み手は `as_deref()`／`as_ref()`／auto-deref だけ：`kivact-core/src/client/dispatch/env.rs:72`、`view/tree.rs:533,601,607,735,845`、`view/elements/container.rs:195`、`client/trace.rs:128`、`kivact-test/src/facts.rs:62,126-168`）。
- 行番号：`apply.rs` の 3 箇所は HEAD で `:66`（`move_before`）、`:462`・`:508`（`insert`）。関数名と `self.stale_fact(parent)`／`self.stale_fact(previous)` の文で探す（W2 の AH1 が同じファイルを変えた後なので）。`:553` の `self.stale_fact(parent)`（`remove` の中、`flat_parent` に対するもの）は変えない。
- kivact-types への単体試験（D7 3.0 の最後の段落）は**足さない**（0 節）。

### 修正（そのまま実装する。HEAD に当てて compile を確認済み）

1. `crates/kivact-types/src/cell.rs` の末尾に D7 3.0 の `Recent<T>`・`RECENT`・`Default`・`Clone`・`Debug`・`share` を足す。**ただし `next` と `last` の上の `// xtask-allow(state-fields): …` の 2 行は書かない**（doc comment は残す）。
2. 新ファイル `crates/kivact-engine/src/model/recent.rs`：

```rust
//! Role: The last distinct values each kind of shared node property was
//! written with in one window, shared between the nodes that write an equal
//! value (repeated components).
use kivact_types::cell::Recent;
use kivact_types::{FocusTarget, LayoutProps, PaintProps, Semantics, StateStyles, TextStyle};

/// The recent values of the shared node properties of one window.
#[derive(Default)]
pub(crate) struct RecentProps {
    pub(crate) layout: Recent<LayoutProps>,
    pub(crate) paint: Recent<PaintProps>,
    pub(crate) text: Recent<TextStyle>,
    pub(crate) states: Recent<StateStyles>,
    pub(crate) semantics: Recent<Semantics<FocusTarget>>,
}

/// A node property stored shared ([`crate::contract::NodeProperties`]).
pub(crate) trait SharedProp: PartialEq + Sized {
    /// The recent values of this kind.
    fn recent(props: &mut RecentProps) -> &mut Recent<Self>;
}

impl SharedProp for LayoutProps {
    fn recent(props: &mut RecentProps) -> &mut Recent<Self> {
        &mut props.layout
    }
}
// PaintProps → props.paint、TextStyle → props.text、StateStyles → props.states、
// Semantics<FocusTarget> → props.semantics を同じ形で書く（5 つの impl）。
```

3. `crates/kivact-engine/src/model.rs`：`pub(crate) mod presence;` の次に `pub(crate) mod recent;`。
4. `crates/kivact-engine/src/model/aggregate.rs` `Model`：`measure` の次に `/// The recent values of the shared node properties (`Model::set_shared_prop`).` と `pub(crate) recent: super::recent::RecentProps,`。`Model::new` に `recent: Default::default(),`。
5. `crates/kivact-engine/src/model/apply.rs`：D3 3.5 の `set_shared_prop`（`set_boxed_prop` の後）と D3 3.4 の `stale_children_of`（`insert` の後）を足し、3 箇所を置き換える（上の行番号の注意）。
6. `crates/kivact-engine/src/contract/node.rs` `NodeProperties`：`layout`・`paint`・`text`・`states`・`semantics` の 5 つを `Option<std::rc::Rc<…>>` にする。marker と doc は今のまま。struct の doc の「the rest boxed」を「the rest boxed, or shared (`Rc`) when many nodes write equal values (`Model::set_shared_prop`)」にする。
7. `crates/kivact-engine/src/host/write/typed.rs`：
   - `fn semantics` の本体を `std::rc::Rc::make_mut(p.semantics.get_or_insert_default())`（上に `// A shared value is copied before one field of it changes.`）。
   - `props!` の `custom` の arm の前に `(@store $w:ident, $id:ident, $v:ident, $m:ident, shared($value:expr, $place:expr)) => { $w.record($id, |model| model.set_shared_prop($id, $value, $place, $m)) };`。表の doc（`store` の種類の説明）に `shared(value, place)` を足す。
   - `Layout`・`Paint`・`TextStyle`・`Semantics` の行の `boxed(` を `shared(` にする（`EditingData`・`ImeSession` の `boxed` は残す）。
8. `crates/kivact-engine/src/model/states.rs` の `set_boxed_prop(` → `set_shared_prop(`。
9. `crates/kivact-engine/src/model/presence/transitions.rs`：`Some(Box::new(layout))` → `Some(std::rc::Rc::new(layout))`。
10. 確認：付属書の U-M3 の行に、旧 U-T9 の `cargo test -p kivact-test --lib accessibility` と E2E `relations::relations_reach_the_accessibility_tree_in_call_order`、旧 U-T2 の代わりに `cargo clippy -p kivact-types --all-targets -- -D warnings` を足す。`cargo xtask check --no-write --crate kivact-types` と `--crate kivact-engine` が error 0。

### 所有ファイルの変更
U-M3 の所有ファイルに `crates/kivact-types/src/cell.rs` を足す（`model/recent.rs` は新規）。

### wave・lane・順序の変更
**T2・T9 を M3 に統合して削除**。W3 lane A は M3 → T3 の 2 unit になる。

---

## T2（付属書 D7 3.0、W3 lane A）

### 判定：修正あり（M3 に統合、unit 削除）
内容は M3 の節。kivact-types の単体試験は足さない。

---

## T9（付属書 D7 3.8、W3 lane A）

### 判定：修正あり（M3 に統合、unit 削除）
内容は M3 の節。D7 3.8 の「注意」（field ごとの setter が共有値を複製する。使うのは試験だけ）はそのまま。

---

## T3（付属書 D7 3.2、W3 lane A、M3 の後）

### 判定：修正あり（小）

- `Recent` は M3 で `kivact_types::cell::Recent` になっている（T2 の統合）。
- 行番号：`leaf.rs` の対象は `fn node_text` の `if extents.as_ref().is_none_or(|e| e.style != style) {`（HEAD `:143`）の if 文。比較は `*e.style != style` にする。
- 付属書のまま compile することを型で確認した：`LeafMeasure` は `Text(Box<RefCell<Option<Box<TextExtents>>>>)` → `Text(RefCell<Option<Box<TextExtents>>>)`、`leaf_for` は `LeafMeasure::Text(RefCell::default())`（`Box::default()` を置き換え）。`MeasureColumns` の derive `Clone, Debug, Default` は `RefCell<Recent<_>>` で通る（`Recent` は手書きの `Clone`・`Debug`）。`recent_style` は lint が数えない。

### 修正
付属書 3.2 のとおり。`measure/inherited_text.rs` の match の `_ => Rc::new(value)` を `_ => columns.share_style(value)` にする（付属書のコードのとおり）。

### 所有ファイルの変更
なし。

### wave・lane・順序の変更
なし（W3 lane A の M3 の次）。

---

## T5（付属書 D7 3.4・3.9・3.10、W4 lane C）

### 判定：修正あり

W1 の F2（`5a28427a`）が `kivact-layout/src/{columns,tree}.rs` を大きく変えたので、付属書の場所を次で読み替える（すべて HEAD `2a3b06b8` の行）：

| 付属書 | HEAD の場所 |
|---|---|
| `columns.rs:31-48` `Record` | `columns.rs:32-48`（`pub(crate) struct Record` と `RecordState`） |
| `columns.rs:110` `styles` | `columns.rs:111` `NodeLayoutValues.styles`。F2 が `relative: Vec<bool>`（`styles` と同時にだけ書く）を足した。`relative` は変えない |
| `columns.rs:120`・`:149`・`:224`・`:249`（performed） | `:126`（field）、`Default` の `performed: vec![None]`（`:157`）、`push` の `values.performed.push(None);`（`:233`）、`reset` の `values.performed[index] = None;`（`:259`） |
| `columns.rs:139-157` `Default`、`:218` push、`:243` reset | `Default for NodeLayoutValues`（`:146-164`）、`push` の `values.styles.push(NodeStyle::default());`（`:224`）、`reset` の `values.styles[index] = NodeStyle::default();`（`:252`） |
| `tree.rs:102-134`（leaf の closure） | `tree.rs:96-133` `fn leaf` の中の `compute_leaf_layout` の closure |
| `tree.rs:358` `from_taffy` | `tree.rs:417-423`。使い手は leaf の closure だけ。削除し、使われなくなる import（`tree.rs:49` の `AvailableSpace`・`MeasureRequest`）を外す |
| `tree.rs:61`（`LayoutPass.styles`） | `tree.rs:64` |
| `tree.rs:71`（`LayoutPass.performed`） | `tree.rs:75` |
| `tree.rs:85`（`item_style`） | `tree.rs:89` |
| `tree.rs:99` | `tree.rs:103`（`let style = &styles[index];` → `let style: &NodeStyle = &styles[index];`） |
| `tree.rs:376`、`:509`（`&self.styles[..]` を返す所） | `get_core_container_style`（`:435`）と同じ形の `:569`、`:591` 付近。型の不一致が出た所だけ `&*` を足す |
| `tree.rs:472-474`（performed の書込み） | `tree.rs:532-534`（`if inputs.run_mode == RunMode::PerformLayout { self.performed[usize::from(t)] = Some(inputs); }`） |
| `tree.rs:674-676`（`rebuild_style`） | `tree.rs:734-737`：`let root = self.values.styles[index].position == … && style.position == …;`、`self.values.relative[index] = uses_parent_size(&style);` の**後**で `self.values.styles[index] = self.values.recent_styles.share(style);` |
| `tree.rs:786`（`relayout_parents`） | `tree.rs:847` `let performed = self.values.performed[at];` |
| `tree.rs:993`、`:1005`、`:1028`、`:1041`（`make_mut`） | `tree.rs:1058`、`:1070`、`:1093`、`:1106` |
| `tree.rs:1066`、`:1082`（`scale_item_styles`） | F2 が `columns.rs:376-429` に移した。`styles: &[Rc<NodeStyle>]`、`:394` の `.or_insert_with(|| styles[index].clone())` → `.or_insert_with(|| (*styles[index]).clone())`、`:420` の `styles[parent + 1].flex_direction` は deref で変更不要 |
| `tree.rs:828-842`（`with_pass` の分解） | `tree.rs:889-904`：末尾に `default_style: _,` と `recent_styles: _,` を足す |
| `compare.rs:48-60`、`:66-89`、`:115`、`:135` | 同じ関数（`remeasures_same`、`compare_measurements`、`rebuild_all` の `self.values.performed.fill(None);`、`compare` の `if let Some(input) = before.performed[at] {`） |

その他：
- `Ask` の定数 `UNKNOWN` と `MAX_CONTENT` は同じ値（`u32::MAX`）だが、使う位置（known と available）が違うので衝突しない。付属書のまま。
- **単体試験は足さない**（kivact-layout には単体試験の module が無く、V2 設計書 0.3 の例外は engine・runtime・xtask だけ）。D7 3.4 の「単体試験」と 5 章の「layout 単体」を取り消す。担保は E2E。
- `kivact-layout/src/tree.rs` は約 985 code 行（上限 1000）。この unit の tree.rs の正味の増減は約 −8 行（`from_taffy` −7、leaf の closure −3、`with_pass` +2、`performed` の条件 ±0）なので超えない。もし超えたら（STOP ではなく）`fn leaf` の closure の中の記録の探索と追加を `columns.rs` の `impl Record { pub(crate) fn measured(&mut self, ask: Ask, measure: impl FnOnce() -> LeafSize) -> LeafSize }` に移す（中身は付属書の `cached.unwrap_or_else(..)` の部分そのまま）。
- `performed` の `child_count(t) > 0` は `LayoutPass` の `TraversePartialTree::child_count` を使う（既存の trait method）。

### 修正
付属書 3.4・3.9・3.10 を上の対応表で行う。完了の確認は付属書の行の `cargo test -p kivact-layout` を `cargo check -p kivact-layout --all-targets` に替え、E2E を `dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- overlay::fitted_overlays_flip_slide_and_respect_the_theme_margin overlay::overlays_flip_clamp_center_stay_out_of_flow_and_follow_their_anchor layout::display_contents_lays_out_children_in_the_parent layout_check:: text_primitives::measured_inline_boxes_wrap_and_share_the_text_baseline_in_draw presence::exit_size_collapses_the_node_before_it_is_removed` にする。

### 所有ファイルの変更
なし（付属書のとおり）。

### wave・lane・順序の変更
なし（W4 lane C。`Recent` は W3 の M3 が入れる）。

---

## T6（付属書 D7 3.5、W3 lane D）

### 判定：修正あり（小）

- `CoreNode` の既存の marker（`at`・`handlers` の上の `independent: a per-node resource; …`）は**残す**（付属書のコードは「（既存の `at` の doc）」としか書いていない）。
- `rare` の marker は doc comment の**上**に置き、文言を D8 3.3（理由と組合せ）に合わせて次にする：
  `// xtask-allow(state-fields): independent: the box of the per-node resources few nodes have (references, exit callback, observation, presence, semantic origins, DevTools layers), created by the first of them; set apart from the location and the handlers, so any combination with them occurs`
- `CoreRare` の `observation`・`presence` の marker は付属書のとおり（doc の上）。
- M8 の後のコードに合わせる：`node_ref.rs` の observation の読み手は `focus_target`（M8 が `focused()` から出した関数）、`target_value`、`fn observation` の 3 つ。`.and_then(|node| node.observation)` は全て `.and_then(CoreNode::observation)`、`?.observation` は `?.observation()`。`references` の追加は M8 の後の `bind_target` の `.expect("invariant: the bound node exists").references.push(state.bound);` → `.expect("invariant: the bound node exists").rare_mut().references.push(state.bound);`。
- M7-fix の後の `semantics.rs` の `bind` の `node.semantic_origins = …` → `node.rare_mut().semantic_origins = …`。
- M6 の `draw::bind_style_paint` は `handlers_mut`（`CoreNode.handlers`、本体に残る）だけを使うので変更不要。
- 付属書の表の行番号はずれている。field 名で全件を `grep -n "\.references\b\|exit_complete\|\.observation\b\|\.presence\b\|semantic_origins\|style_layers" crates/kivact-core/src` で探し、表の変更を当てる（`view/flow/branch.rs` の `self.presence` と `motion/frame_loop.rs` の `node.presence()` は別物なので変えない）。

### 修正
付属書 3.5 と上の差分。

### 所有ファイルの変更
なし（付属書のとおり）。

### wave・lane・順序の変更
なし（W3 lane D。W2 lane D の M7-fix・M6・M8 の後）。

---

## 全体

### wave・lane の組み直し（担当範囲）

| wave | lane | unit の順（変更後） | 変更点 |
|---|---|---|---|
| W2 | D | **M7-fix → M6 → M8 → T1 → G3** | M7-fix を新設、M6 を W1 から移す、M9 を lane F へ |
| W2 | E | **T8-fix → S7** | T8-fix を新設 |
| W2 | F | **M9** | G2 は W1 に前倒し済みで空きだった |
| W3 | A | **M3（+T2+T9）→ T3** | T2・T9 を M3 に統合 |
| W3 | D | T6 → S6 | 変更なし |
| W4 | C | T5 | 変更なし |

ファイルの重なりの確認（同じ wave の別 lane）：W2 の lane D（`kivact-core/src/view/mount/{semantics,resolve,style,draw,facts}.rs`、`view/{node_ref,ref_data,state,part,node_status,scroll_timeline}.rs`、`view/elements/text.rs`、`view/tree.rs`（`ViewNodes::role` だけ）、`kivact-engine/src/host/write/reader.rs`、`kivact-engine/src/contract/node.rs`（`NodeProperties::role` だけ）、`kivact-test/src/facts.rs`（`role` だけ）、e2e の `relations.rs`・`node_ref.rs`・`view.rs`・`diagnostics.rs`、G3 のファイル）と lane F の M9（`view/mount/{regions,part_input,parts}.rs`、`view/handler_layers.rs`）は重ならない。lane A の AH1 は `host/write/typed.rs`・`model/*`・`a11y/*` を持つが `reader.rs`・`contract/node.rs` は持たない（`reader.rs` は W3 の AH2）。lane E の T8-fix（`reactive/data.rs`）と S7（`reactive/{handles,graph}.rs`、`machine.rs`、`flow/list.rs`）は重ならない。lane B の F7 は e2e の新ファイル `frame_sources.rs`・`main.rs`・`images_animated.rs` で、lane D の e2e ファイルと重ならない。M7-fix・M8・T1 の試験は既存の module ファイルに足すので `e2e/main.rs` は変えない。

compile の結合（W2 の終わりに compile が通るか）：M6・M7-fix・M8・M9・T1 の署名の変更は全て kivact-core の中の `pub(crate)`／private で、他の lane の unit が使う関数・型を変えない（`Reader::semantics` は署名を変えない）。W3 の M3 の公開 field の型の変更（`NodeProperties`）は engine の外を直さずに compile する（試して確認）。

### W1 の unit との食違いの確認
- M1（`146868b4`、View の群・空、行の fragment）：M6 は `Mcx` を作り直すだけで View に触れない。T1 の E2E の `"string"` は M1 の後の `View::group` の中の `string_child` を通る。食違いなし。
- M7（`a087054c`）：M7-fix で閉じる。T6 の `semantic_origins` の書換えは M7-fix の後の `bind` に当てる。
- M10（`77ea681f`、button の定数の disabled）：M9 の `ConstFlags` は core の `constraints` の中だけで、button の `toggle.rs`・`button.rs` は触らない。食違いなし。
- M11（`642c36c3`、`Links`）・T8（`636c4794`）：T8-fix は marker 1 行だけ。M11 の `Links::Inline { len, items }` は数える field を持たない（lint は HEAD で error 0）。
- F2（`5a28427a`）：T5 の場所の対応表（T5 の節）。F3（`2e160b08`）：M3 の `stale_children_of` が使う `NodeTree::box_of`・`flat_parent` は F3 の後も同じ。

### 担当外だが HEAD で見つけたこと（親へ）
- `dev/tests/kivact-public-api/tests/e2e/diagnostics.rs` の `why_follows_the_prop_to_the_write_through_a_memo` は HEAD `2a3b06b8` で失敗する（M1 の後、見出しの node id が 4、semantics patch の行が 1 つ）。本書は T1 の unit で直す形にした。T1 より前の checkpoint は無いので wave の順では問題にならない。
- `cargo clippy -p kivact-engine -- -D warnings` は HEAD で失敗する：`crates/kivact-engine/src/paint/state.rs:326` `Indexed` の `size`・`content` が読まれない（dead_code）。W1 の F4（`81db248b`）または F5 の残り。W1 の確認は `cargo check` だけだったので見逃された。lane B（F4 の所有）か CP1 の fix の対象。

### 付属書に書き戻す点（W8）
- V2 設計書 4 章 DEC-11：M6 は遅延の draw effect で、callback の signal で style を再実行しない（「許容する」は不要になった）。D3 3.6・5 章の M2-6 の行を本書の M6 の節で置き換える。
- D3 3.5・D7 3.0 の `Recent` の marker（書かない）、D7 3.8 の unit（M3 に統合）。

### ユーザー（人間）にしか決められない項目
なし。本書で判断したもののうち、ユーザーの方針の解釈に当たるものは次の 2 つで、どちらも方針どおりの側を選んだ（変えたい場合だけ指示がほしい）：
1. kivact-types・kivact-layout の単体試験を足さない（V2 設計書 0.3「E2E で検証。単体試験の例外は engine・runtime・xtask」に従った。D7 の指示と食い違っていた）。
2. `diagnostics::why_follows_the_prop_to_the_write_through_a_memo` の期待値の 1 行を消す（T1 で text の冗長な role patch が無くなる結果。性能・メモリの目的を優先）。
