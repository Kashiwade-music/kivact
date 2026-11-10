# V2 再監査 R3（target lane C）：S4・S5・S6・S7・S8・S9・G3・G4、K6 nogpu

> **V2.1 修正付属書 D11**（再監査 R3 の成果物、HEAD `2a3b06b8`＝W1 の後に対して書いた）。範囲：W2〜W3 の S45（S4＋S5）・S6・S7・S8・S9・T4 の順序・G3・G4（サイズ・GPU）。
> - **優先順位**：本書は付属書 D1〜D8 より優先する。本書の unit の節の「修正」は、作業指示書の行の「本書の追加の指示」と同じ扱い。本書と V2 設計書 4 章（DEC）が食い違う場合は V2 設計書が正（V2 設計書 4 章の DEC-18 の名前は `while_open(factory)`）。
> - 本書の「全体」節の wave・lane の組み直しは、作業指示書（V2.1）に反映済み。実施順・所有ファイルは作業指示書が正。
> - 文中の worktree（`W:/w/kv-audit-*`）・scratchpad の path は監査の試作の場所で、実装に必要ではない（試作の差分は参考で、実装は本書の文に従う）。

調べ方：HEAD `2a3b06b8` から detached worktree `W:/w/kv-audit-R3` を作り（`git worktree remove --force` で削除済み）、本書の修正をすべて実際に当てた。`cargo check --workspace --all-targets --all-features`、関係する crate の `cargo clippy ... -- -D warnings`、`cargo xtask check --no-write`（全体）、`cargo test -p kivact-runtime`、window-lock を通した実 window の試験（本書末尾の「検証の記録」）で確かめた。下に書くコード片は、その worktree で compile と lint が通ったものそのまま。

**横断の阻害要因（R3 の範囲外。親が振り分ける）**：HEAD で `cargo clippy -p <kivact-engine に依存する crate> --all-targets -- -D warnings` は、feature `testing` を付けないと全部失敗する。`crates/kivact-engine/src/paint/state.rs:326` `Indexed` の field `size`・`content` を読むのが `paint/list/self_check.rs`（`#[cfg(all(debug_assertions, feature = "testing"))]`）だけで、`dead_code` になる（F4 `81db248b` の残り）。`--all-features` を付けても、`-p kivact-runtime -p kivact-core` のように kivact-engine を `testing` 無しで build する組合せでは失敗する（実測）。CP の gate（`--workspace --all-features`）は通るので W1 では気付かれていない。S45・G3・S7 の付属書の確認コマンドはこれに当たるため、本書ではそれらの clippy に `--all-features` を付けた形と、当たる場合の扱いを書いた。根本の直しは D1（F4）担当の監査で決めること（推奨：`Indexed` から `size`・`content` を消し、self_check はそれを比べない。あるいは self_check が要る値を `Ink` から読む）。

---

## S45（S4＋S5 を統合。付属書 D4 3 章「D3」の全部、W2 lane C）

### 判定：修正あり

S4 の commit `3c284b0b` を HEAD に cherry-pick すると conflict 無しで当たり、壊れる呼出し元は **`crates/kivact-engine/src/lower/assets.rs:256` の 1 箇所だけ**（`cargo check --workspace --all-targets --all-features` で確認）。S5 の 4 ファイルを足すと workspace 全体が error・warning 0 で compile する。`FontSource` に触れる全ての場所（workspace の grep と compile で列挙）：

| crate | file | 変更 | 出所 |
|---|---|---|---|
| kivact-types | `src/render.rs`、`src/render/shaped.rs` | `FontSource` の定義、`GlyphRun.font: FontSource`、再公開 | 3c284b0b |
| arvane-text | `src/output.rs`、`src/text_engine.rs`、`src/inline.rs` | copy の削除、`register_font(FontSource)` | 3c284b0b |
| arvane-render | `src/draw/painter.rs`、`src/validate/fonts.rs`、`src/validate/scene.rs` | `FontCache`・検証 cache の key | 3c284b0b |
| arvane-render | `src/ink.rs`、`src/outline.rs` | **変更不要**（`FontRef::from_index(&run.font, ..)` は deref coercion で compile する。付属書の表の 2 行は行わない） | — |
| arvane | `src/engine.rs` | `Engine::register_font(FontSource)` | 3c284b0b |
| arvane | `tests/e2e/text.rs` | 試験 2 本（**3c284b0b に入っていない**。下で足す） | 本 unit |
| arvane-asset | `src/font.rs`、`src/store/asset_store.rs`、`src/svg/fonts.rs` | `to_source`、`register_font(FontSource)`、fontdb の共有 | S5 |
| kivact-engine | `src/lower/assets.rs` | 呼出し 2 行 | S5 |
| 試験 | `Engine::with_fonts_only(&[Arc<[u8]>])`・`TextEngine::isolated` の呼出し元（arvane e2e `support.rs:54`・`text_style.rs:26`、`kivact-engine/src/host/testing/test_host.rs:157-158`） | **変更不要**（署名を変えない、付属書どおり） | — |

`GlyphRun { .. }` を literal で作るのは `arvane-text/src/output.rs` の 1 箇所だけ（grep 済み）。`GlyphRun.font` の他の読み手（`arvane-render/src/cpu.rs:258,263`、`gpu/encode.rs:300,310` は `run.font` を渡さない別の型）は影響なし。

### 修正（作業指示書の行の「本書の追加の指示」）

作業指示書の W2 の表から lane B の S5 の行を消し、lane C の先頭に次の行を置く：`| C | S45 | D4 3 章「D3」 | P4-U4 と P4-U5 | 下の手順 1〜6 |`。手順：

1. `git cherry-pick 3c284b0b`（S4 の元の commit。conflict は起きない）。commit message はそのまま残す。
2. `crates/arvane-asset/src/font.rs`：`use kivact_types::Asset;` の次に `use kivact_types::render::FontSource;` を足し、`to_shared` を次で置き換える（`FontBytes` の他は変えない）。

```rust
    /// The bytes as a font source shared with them (never copied).
    pub fn to_source(&self) -> FontSource {
        match self {
            FontBytes::Static(b) => FontSource::from_static(b),
            FontBytes::Shared(b) => FontSource::from_shared(Arc::clone(b)),
        }
    }
```

3. `crates/arvane-asset/src/store/asset_store.rs` の `register_font`：署名を `pub fn register_font(&mut self, font: kivact_types::render::FontSource)` にし、1 行目を `self.fonts = Arc::new(self.fonts.with_font(font));` にする。doc の「Registers the font `bytes`」を「Registers the font `font`」にする。
4. `crates/arvane-asset/src/svg/fonts.rs`：`use resvg::usvg;` の前に `use kivact_types::render::FontSource;`。field を `app: Vec<FontSource>,`。`with_font` を `pub(crate) fn with_font(&self, font: FontSource) -> Self` にし、本体の push を `app.push(font);`、doc を「The same App fonts plus `font`; ...」。db 構築の `fonts.load_font_data(font.as_ref().clone());` を `fonts.load_font_source(usvg::fontdb::Source::Binary(Arc::clone(font.source())));` にする（戻り値は捨てる。`let _ =` は付けない：`TinyVec` は `must_use` でない）。
5. `crates/kivact-engine/src/lower/assets.rs` の `register_font`：`let bytes = file.bytes().to_shared();` → `let font = file.bytes().to_source();`、`engine.register_font(bytes.clone())` → `engine.register_font(font.clone())`、`self.store.register_font(bytes);` → `self.store.register_font(font);`。
6. 古くなる doc を直す（S4 が持つファイル）：
   - `crates/arvane/src/engine.rs` の `release_caches` の doc の「ellipses, font copies and shaping scratch」→「ellipses and shaping scratch」、「keep their shared font bytes and remain valid」→「keep the font files they share alive and remain valid」。
   - 同 `shape` の doc の「the returned snapshot owns its runs and font bytes independently of this engine and its caches.」→「the returned snapshot owns its runs, which share the font files they were shaped with (kept alive by the runs), independently of this engine and its caches.」
   - `crates/kivact-types/src/render/shaped.rs` の `GlyphRun` の doc「A shaped run retaining its font bytes; usable on any thread.」→「A shaped run sharing its font file (kept alive by the run); usable on any thread.」
7. `crates/arvane/tests/e2e/text.rs` の末尾に次の 2 本を足す（D4 5.3 の 1・2。この形で compile・clippy・実行を確認済み）。

```rust
#[test]
fn glyph_runs_share_the_registered_font() {
    let font = arvane::test_font();
    let mut engine =
        arvane::Engine::with_fonts_only(&[font.clone()]).expect("the test font is usable");
    for text in ["Shared bytes", "Another paragraph"] {
        let shaped = engine.shape(text, &style(16.0), None).expect("shape");
        assert!(!shaped.runs.is_empty(), "{text:?} is drawn");
        for run in &shaped.runs {
            assert_eq!(
                run.font.as_ptr(),
                font.as_ptr(),
                "{text:?}: the run shares the registered bytes"
            );
        }
    }
}

#[test]
fn glyph_runs_of_one_system_font_share_one_source() {
    let mut engine = arvane::Engine::new();
    let first = engine.shape("Alpha", &style(16.0), None).expect("shape");
    let second = engine.shape("Beta", &style(16.0), None).expect("shape");
    assert!(
        !first.runs.is_empty() && !second.runs.is_empty(),
        "both are drawn"
    );
    assert_eq!(
        first.runs[0].font.id(),
        second.runs[0].font.id(),
        "one system font, one shared source"
    );
}
```

完了時の確認（付属書 P4-U4・P4-U5 の行の代わり）：

```bash
cargo check --workspace --all-targets --all-features
cargo clippy -p kivact-types -p arvane-text -p arvane-render -p arvane -p arvane-asset -p kivact-engine --all-targets --all-features -- -D warnings
dev/tools/window-lock.sh --timeout 900 -- cargo test -p arvane --test e2e -- text
dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- launch_fonts svg
```

clippy が `crates/kivact-engine/src/paint/state.rs` の `fields size and content are never read` だけで失敗したら、それは本 unit の外（冒頭の横断の阻害要因）なので直さず、報告に `NOTE: engine dead_code (F4)` と書いて続ける（STOP にしない）。

state-fields：本 unit が足す・変える型に数える field は無い（`FontSource` は field 1 個、`validate/fonts.rs` の `Entry` は `Weak`・整数だけ、`SvgFonts` は既存の `Mutex<Option<..>>` 1 個）。全体の `xtask check` で新しい error が出ないことを確認済み。marker は書かない。

### 所有ファイルの変更

S45 の所有ファイル：`crates/kivact-types/src/{render.rs,render/shaped.rs}`、`crates/arvane-text/src/{output,text_engine,inline}.rs`、`crates/arvane-render/src/{draw/painter,validate/fonts,validate/scene}.rs`、`crates/arvane/src/engine.rs`、`crates/arvane/tests/e2e/text.rs`、`crates/arvane-asset/src/{font,store/asset_store,svg/fonts}.rs`、`crates/kivact-engine/src/lower/assets.rs`。`ink.rs`・`outline.rs` は外す（変更不要）。

### wave・lane・順序の変更

- S5 を W2 lane B から外し、S4 と統合して W2 lane C の先頭（S45）。lane B は F7 だけになる。
- W2 の他の lane とファイルは重ならない（lane A AH1・lane D の T1/M8/M9/G3 は `lower/assets.rs`・`arvane-*`・`kivact-types/src/render*` を持たない）。

---

## T4（D7 3.3）の順序（W2 lane C）

### 判定：修正あり（順序だけ）

- T4 は `crates/arvane-text/src/text_engine.rs`・`crates/arvane/src/engine.rs` を S4 と共有する（V2 設計書 5.3 の「S4 → T4」）。作業指示書の今の配置（S5 が lane B、T4 が lane C、S4 は revert 済み）では、lane C の worktree に `FontSource` が無く、merge で同じファイルが二つの lane に出る。
- 確定：**W2 lane C を S45 → T4 → S8 → S9 の順にする**。T4 の行の「S4 の `FontSource` の後のコードに書く」はこの順で満たされる。
- T4 が触る `layout_at`・`measure_uncached`・`measure`・`ellipsis_for`・`shape_cached` の本体のうち、S4 が変えるのは `ellipsis_for`（`Ellipsis::of(&layout)`）と `shape_cached`（`output::draw(..)` の引数）の呼出し行だけで、T4 の手順（`let (layout, _) = layout_at(..)`）と同じ行には当たらない。`engine.rs` の T4 の doc（`measure` の「once the paragraph was used before」）は S45 の doc の変更（`release_caches`・`shape`）と別の関数。T4 の付属書の本文は変えなくてよい（T4 自体の監査は D7 担当）。

---

## S6（D4 3 章「D5」だけ。W3 lane D）

### 判定：修正あり（所有ファイルの追加 1 件、確認の具体化）

- 作業指示書の指示どおり `dev/bench/Cargo.toml` を変えると（worktree で当てて確認）、`cargo tree -p kivact-bench --no-default-features -e normal | grep -c wgpu` は 0、`--no-default-features --features complex-scripts`（M12 の `kv_startup_nogpu`）も wgpu 0、`cargo tree ... -e features -i arvane-compose` に `"gpu"` が出ない。`naga` は残るが、これは `kivact-shader`（WGSL の検証、GPU と無関係）の依存で、D4 1.1 の「gpu なし 14,406,656 B」の実測にも含まれていた。GPU 抜きとして正しい。
- 既定 build（`kv_startup`・`kv_stress`・`kv_heap`・`kv_latency`）は bench の既定 feature に `gpu` があるので今と同じ（wgpu 18 行）。`dev/bench/src` に `gpu` の cfg・API の使用は無い（grep 済み）ので `--no-default-features` で compile する。
- **文書の誤りが生じる**：`docs/for-kivact-user/en/README.md` の「`kivact-bench` enables only `gpu` for its `--no-default-features` binary, which measures the smaller build.」と `jp/README.md` の同じ文は、S6 の後は事実と逆になる。S6 で直す。

### 修正

本書の追加の指示を次に置き換える：

> 所有ファイルは `dev/bench/Cargo.toml` と `docs/for-kivact-user/{en,jp}/README.md`（下の 1 文ずつだけ）。`dev/bench/Cargo.toml`：`[features]` を `default = ["complex-scripts", "gpu"]`、`complex-scripts = ["kivact/complex-scripts"]`、`gpu = ["kivact/gpu"]`、`latency = ["kivact/test"]`（コメント行は残す）にし、依存を `kivact = { path = "../../crates/kivact", default-features = false }` にする。`docs/for-kivact-user/en/README.md` の「`kivact-bench`
> enables only `gpu` for its `--no-default-features` binary, which measures the
> smaller build.」を「`kivact-bench` builds with neither feature for its `--no-default-features` binary, which measures the smallest build.」に、`docs/for-kivact-user/jp/README.md` の「`kivact-bench` は `--no-default-features` のバイナリで `gpu` だけを有効にし、小さくなったビルドを測ります。」を「`kivact-bench` の `--no-default-features` のバイナリはどちらの feature も持たず、最小のビルドを測ります。」にする。確認：`cargo check -p kivact-bench --all-targets`、`cargo check -p kivact-bench --no-default-features --features complex-scripts --bin kv_startup`、`cargo tree -p kivact-bench --no-default-features -e normal | grep -c wgpu` が 0、`cargo tree -p kivact-bench --no-default-features --features complex-scripts -e normal | grep -c wgpu` が 0。

### 所有ファイルの変更

`docs/for-kivact-user/{en,jp}/README.md`（各 1 文）を足す。W3 で他に docs の README を持つ unit は無い（G3 は W2、G6 は W5）。

---

## K6 の nogpu の build（MS-5、M12 の run.sh）

### 判定：変更なし（S6 の後に正しく GPU 抜きになる）

- `build kv_startup_nogpu kv_startup -p kivact-bench --no-default-features --features complex-scripts --bin kv_startup`：S6 の後 wgpu 0（上）。S6 の前は wgpu を含む（OPEN.md の付記どおり。CP1 は W4 の後なので問題ない）。
- `build catalog_nogpu kivact-ui-basic-catalog -p kivact-ui-basic-catalog --no-default-features --features complex-scripts`：catalog は既に `gpu = ["kivact/gpu"]` を自前の feature で持ち、依存の `kivact` は `default-features = false`、`kivact-ui-basic` も GPU を有効にしない。HEAD で既に wgpu 0（`cargo tree` で確認）。S6 と無関係に正しい。
- `kv_startup_min`（`--no-default-features` だけ）は S6 の後 GPU・complex-scripts とも無し（MS-1 の定義どおり）。

---

## S7（D4 3 章「D6」、W2 lane E）

### 判定：修正あり（付属書の文章をコードに確定）

付属書 D6 は形の説明だけで、Haiku が設計する余地が大きい。worktree で次を当て、`cargo clippy -p kivact-runtime --all-targets -- -D warnings`（engine の dead_code を除き 0）、`cargo check -p kivact-core --all-targets --all-features`、`cargo test -p kivact-runtime`（成功）を確認した。M11・T8 の後の `handles.rs`・`graph.rs` と M1 の後の `list.rs` に対して書いてある。付属書の `rt.memo_cell::<T>(&cell, at) -> Rc<MemoCell<T>>` の形は採らず、既存の `source_cell` と同じ自由関数にする（より単純、`Rc` の複製が要らない）。

### 修正（付属書 D4 3 章「D6」の 1〜3 を次の手順で置き換える）

1. `crates/kivact-runtime/src/reactive/handles.rs`
   - `use std::any::Any;` を足す。`use super::graph::{is_memo, source_cell};` → `use super::graph::{memo_cell, source_cell};`。
   - `// Signal` の区切りの前に次を足す：

```rust
// ---------------------------------------------------------------------------
// Reads before the value type is known
//
// The steps of a read that do not depend on the value type are compiled once
// here; the generic `with_impl`s keep only the downcast and the caller's `f`.

/// What a `Value` read found before its value type is known.
enum Resolved {
    /// A constant's value (an `Rc<T>` behind `dyn Any`).
    Const(Rc<dyn Any>),
    /// A source's or a memo's cell, read as `track` asked.
    Cell(Rc<dyn Any>),
}

/// Find what `repr` of `app` holds, read as `track` asks (a stale memo is
/// computed first); a located misuse when the App, the node or the
/// constant is gone.
#[inline(never)]
fn resolve_value(
    app: AppId,
    repr: ValueRepr,
    track: bool,
    at: &'static Location<'static>,
) -> Resolved {
    let rt = runtime(app, at);
    match repr {
        ValueRepr::Const(key) => {
            let data = rt.consts.borrow().get(key).map(|entry| Rc::clone(&entry.data));
            match data {
                Some(data) => Resolved::Const(data),
                None => rt.const_disposed_panic(key, at),
            }
        }
        ValueRepr::Node(key) => Resolved::Cell(rt.read_or_panic(key, track, "Value", at)),
    }
}

/// The cell of the source `key` of `app`, read as `track` asks; a located
/// misuse when the App or the node is gone.
#[inline(never)]
fn resolve_signal(
    app: AppId,
    key: Key,
    track: bool,
    at: &'static Location<'static>,
) -> Rc<dyn Any> {
    runtime(app, at).read_or_panic(key, track, "Signal", at)
}
```

   - `Signal::with_impl` の先頭 3 行（`let rt = runtime(..)`、`let cell = rt.read_or_panic(.., "Signal", at)`、`drop(rt)`）を `let cell = resolve_signal(self.app, self.key, track, at);` の 1 行にする（以降は変えない）。
   - `Value::with_impl` の本体を次にする：

```rust
        let cell = match resolve_value(self.app, self.repr, track, at) {
            Resolved::Const(data) => {
                let Ok(value) = data.downcast::<T>() else {
                    misuse_at(ErrorKind::WrongApp, "handle type mismatch", at)
                };
                return f(&value);
            }
            Resolved::Cell(cell) => cell,
        };
        if let Some(src) = source_cell::<T>(&cell) {
            let guard = match src.value.try_borrow() {
                Ok(g) => g,
                Err(_) => misuse_at(
                    ErrorKind::BorrowedWrite,
                    "value read while it is being updated",
                    at,
                ),
            };
            f(&guard)
        } else if let Some(memo) = memo_cell::<T>(&cell) {
            let value = memo.value.borrow();
            f(value
                .as_ref()
                .expect("invariant: read_node computed the memo (a self-read is a Cycle there)"))
        } else {
            misuse_at(ErrorKind::WrongApp, "handle type mismatch", at)
        }
```

2. `crates/kivact-runtime/src/reactive/graph.rs`：`Runtime::memo_value` を削除（呼出し元は上の 1 箇所だけ）。`pub(crate) fn is_memo` を次で置き換える：

```rust
/// The memo cell of `T` behind `cell`, if it is one (`read_node` computed
/// its value first; a memo that reads itself is a `Cycle` misuse there).
pub(crate) fn memo_cell<T: 'static>(cell: &Rc<dyn Any>) -> Option<&MemoCell<T>> {
    cell.downcast_ref::<MemoCell<T>>()
}
```

   不要になった import（`misuse_at` 等）が warning になれば消す。

3. `crates/kivact-runtime/src/machine.rs`：`/// Create a machine during setup.` の前に次を足し、`send` の timer の `if let Some(delay) = delay { .. }` の中身を下の 4 行にする。

```rust
/// Run `fire` after `delay` on a task of `owner`. Every machine's timers
/// share this one future type: the handle and the event are in `fire`.
#[inline(never)]
fn spawn_timer(
    owner: Owner,
    delay: Duration,
    fire: Box<dyn FnOnce()>,
    at: &'static Location<'static>,
) -> Task {
    let wait = sleep(delay);
    spawn_in(
        owner,
        async move {
            wait.await;
            fire();
        },
        at,
    )
}
```

```rust
                    if let Some(delay) = delay {
                        let handle = *self;
                        let fire: Box<dyn FnOnce()> = Box::new(move || handle.send(event));
                        let task = spawn_timer(cell.owner, delay, fire, at);
                        cell.timers.borrow_mut().insert(key, task);
                    }
```

4. `crates/kivact-core/src/view/flow/list.rs` の `apply_list`（`#[allow(clippy::too_many_arguments)]` は既存。新しい関数は引数 5 個以下）：
   - 手順 2 の `for (i, slot) in old.iter_mut().enumerate() { if !keep_mark[i] && let Some(r) = slot.take() { .. } }` を次にする（行は `owner` を dispose した後に捨てる。今と同じ順）：

```rust
    let gone: Vec<Row<K, T>> = old
        .iter_mut()
        .zip(&keep_mark)
        .filter(|(_, kept)| !**kept)
        .filter_map(|(slot, _)| slot.take())
        .collect();
    let parts: Vec<(NodeId, Owner)> = gone.iter().map(|r| (r.node, r.row.owner())).collect();
    dispose_rows(rt, &parts);
    drop(gone);
```

   - 末尾の `if live && (fresh.iter().any(..) || moved.iter().any(..)) { .. }` の塊を次にする：

```rust
    if live {
        let nodes: Vec<NodeId> = new_rows.iter().map(|r| r.node).collect();
        place_rows(rt, frag, &nodes, &fresh, &moved);
    }
```

   - `apply_list` の後に次の 2 関数を足す：

```rust
/// Remove each row's node from the tree and dispose its owner, in order
/// (the part of a list update that does not depend on the item type).
#[inline(never)]
fn dispose_rows(rt: &Rc<Runtime>, rows: &[(NodeId, Owner)]) {
    for &(node, owner) in rows {
        let removed = ViewState::tree_mut(rt).remove(node);
        removed.commit(rt);
        owner.dispose();
    }
}

/// Insert the `fresh` rows into `frag` and move the `moved` ones, right to
/// left (`nodes` in the new order; the rows are already created).
#[inline(never)]
fn place_rows(rt: &Rc<Runtime>, frag: NodeId, nodes: &[NodeId], fresh: &[bool], moved: &[bool]) {
    if !fresh.iter().any(|f| *f) && !moved.iter().any(|m| *m) {
        return;
    }
    let tree = ViewState::tree(rt);
    let mut next = None;
    for i in (0..nodes.len()).rev() {
        if fresh[i] {
            tree.engine().insert(frag, nodes[i], next);
        } else if moved[i] {
            tree.engine().move_before(nodes[i], next);
        }
        next = Some(nodes[i]);
    }
}
```

振る舞いの違いは一つだけ：`Value` の定数の型が合わない時（型付きの handle では起きない）の misuse が `Disposed` から `WrongApp` になる。試験の期待値には当たらない。

完了時の確認：`cargo clippy -p kivact-runtime -p kivact-core --all-targets --all-features -- -D warnings`（engine の dead_code だけなら S45 と同じく `NOTE` で続ける）、`cargo test -p kivact-runtime`、`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- flows:: disposal::`。

### 所有ファイルの変更・wave の変更

なし（`handles.rs`・`graph.rs`・`machine.rs`・`list.rs`）。W2 lane E のまま。

---

## S8（D4 3 章「D7」、W2 lane C）

### 判定：修正あり（コードの確定）

- 付属書の一覧の 35 crate はすべて `Cargo.lock` にある（`roxmltree`・`png`・`miniz_oxide`・`accesskit_consumer` は 2 版あるが、版なしの指定は両方に効き、cargo は警告を出さない。worktree で `cargo check --release -p kivact-bench` を実行して確認）。
- `templates/basic/Cargo.toml` の profile は workspace の中では無視される（既存の警告「profiles for the non root package will be ignored」）。単独の App のための推奨設定なので同じものを書く（付属書どおり）。

### 修正

root `Cargo.toml` の最後の `[profile.release.package.arvane-text]` `opt-level = 3` の後と、`templates/basic/Cargo.toml` の同じ位置（最後の `[profile.release.package.arvane-text]` の後）に、空行 1 行と次をそのまま足す（コメントは英語。付属書の日本語のコメントは使わない）：

```toml
# Third-party crates off the startup and per-frame paths favour size (SVG
# parsing and drawing, image decoding, the accessibility adapter, file dialogs
# and the clipboard, date and number formatting, GPU shader translation). The
# hot paths above and Kivact's own crates keep their levels.
[profile.release.package.usvg]
opt-level = "z"
```

続けて同じ 2 行の形で、この順に：`resvg`、`tiny-skia`、`tiny-skia-path`、`rustybuzz`、`ttf-parser`、`fontdb`、`roxmltree`、`svgtypes`、`simplecss`、`image`、`zune-jpeg`、`zune-core`、`image-webp`、`png`、`gif`、`fdeflate`、`miniz_oxide`、`flate2`、`accesskit`、`accesskit_consumer`、`accesskit_windows`、`accesskit_winit`、`rfd`、`arboard`、`naga`、`icu_datetime`、`icu_calendar`、`icu_decimal`、`icu_plurals`、`icu_collator`、`icu_time`、`icu_locale_core`、`kivact-i18n`（全 34 節）。確認：`cargo metadata --format-version 1 > /dev/null`、`cargo check --release -p kivact-bench --bin kv_startup 2>&1 | grep -E "did not match|ambiguous"` が何も出さないこと。

### 所有ファイル・wave の変更

なし。W2 lane C の 3 番目（S45 → T4 → S8 → S9）。

---

## S9（D4 3 章「D8」、W2 lane C）

### 判定：修正あり（文字集合・subset の設定・試験・文書をすべて確定）

付属書のままだと次の 4 点で壊れる（worktree で fontTools 4.68 を使って確認）：

1. **family 名が変わる**：pyftsubset の既定は name ID 0〜6 だけを残す。Noto Sans JP の typographic family（ID 16）は「Noto Sans JP」、ID 1 は「Noto Sans JP Thin」。既定のままでは family が「Noto Sans JP Thin」になり、`"Inter, 'Noto Sans JP', sans-serif"` 等の指定が当たらなくなる。`name_IDs = *` が必須。
2. **文字の欠落**：付属書の「U+2000 以上の走査結果」では、走査で見つかる U+0080〜U+1FFF の 13 文字（`¥§©«°±²·¹»×äé`）が落ちる。game・creative の catalog は Inter を持たないので、これらが Noto の外へ fallback する。走査は U+0080 以上にし、Latin-1 も範囲に入れる。ICU が出す ja-JP の日付の文字（pop の calendar）も明示で入れる（`午` は走査に無い）。
3. **漏れの検出の試験が初めから落ちる**：basic catalog の page source には、どの catalog font にも無い emoji 4 文字（U+1F389、U+1F440、U+1F44F、U+1F680）がある。試験は emoji（U+1F000 以上）を対象外にする。描画の比較で `.notdef` を判定する具体的な形を下に確定した（cmap を読む dependency は足せない）。
4. **再生成できない**：元の 9.6 MB を消すと script を再実行できない。script は元を git の履歴から取り出す（削除した commit の親）。SHA-256 で確かめる。

subset の結果（worktree で生成）：`NotoSansJP-subset[wght].ttf` 869,572 B（元 9,589,900 B、−8.72 MB）、2,189 glyph、1,521 code point、表は元と同じ 24 種（`BASE`・`fvar`・`gvar`・`avar`・`HVAR`・`STAT`・`vhea`・`vmtx` を含む）、name ID 16「Noto Sans JP」、`.notdef` の輪郭あり。同じ入力で 2 回生成して byte 一致（決定的）。SHA-256 `7f0943e76a1d732753d53bbf4ce017f01286e17b0ea9e51577dbe5fe6316aeff`（W2 の時点の source の走査。S9 を行う時の source で変わり得るので、値は実行結果を使う）。

### 修正（作業指示書の S9 の行の「本書の追加の指示」）

1. `dev/tools/subset-catalog-font.py` を次の内容で新規に作る（一字一句）。

（「付録 A」の全文をそのまま書く）

2. `python dev/tools/subset-catalog-font.py` を実行する（Git Bash。`-I` を付けない：fontTools は `--user` で入っている）。出力 `examples/shared-assets/fonts/notosansjp/NotoSansJP-subset[wght].ttf` ができ、大きさが 700,000〜1,200,000 B であること。外れたら STOP。
3. `git rm "examples/shared-assets/fonts/notosansjp/NotoSansJP[wght].ttf"`。
4. 次の 5 箇所の `notosansjp/NotoSansJP[wght].ttf` を `notosansjp/NotoSansJP-subset[wght].ttf` にする：`examples/kivact-ui-{basic,pop,game,creative}/src/fonts.rs`、`dev/tests/kivact-public-api/tests/e2e/layout_check.rs` の `static NOTO`（行が長くなるので最後に `cargo fmt`）。
5. `examples/shared-assets/fonts/SHA256SUMS.txt` の `notosansjp/NotoSansJP[wght].ttf` の行を、`sha256sum` と `wc -c` で得た subset の値で `<sha256>  <bytes>  notosansjp/NotoSansJP-subset[wght].ttf` に置き換える（区切りは既存の行と同じ 2 空白）。
6. `examples/shared-assets/fonts/SOURCE.md` の最初の段落（「Noto Sans JP was copied ...No font was upgraded.」）を次にする：

```markdown
Noto Sans JP is a subset of the Google Fonts file
[Noto Sans JP](https://github.com/google/fonts/tree/main/ofl/notosansjp)
`NotoSansJP[wght].ttf` (9,589,900 bytes, SHA-256
`c2f3b4d463500a2ddcd3849cded1fceeb9fd6d1c32e6cbecd568453ba50fc68f`), made by
`dev/tools/subset-catalog-font.py`: the characters above ASCII that the
catalogs, the kits and the public API tests use, ASCII, Latin-1, General
Punctuation, Hiragana and Katakana, CJK symbols, and the full-width forms. The
weight axis, metrics, layout features and names are kept. Run the script again
after adding text with new characters; it takes the original from the git
history. The SIL Open Font License allows subsets, and the family has no
Reserved Font Name. The family's original `OFL.txt` notice is preserved beside
the font. `SHA256SUMS.txt` records the hashes and byte sizes.
```

7. 「unchanged / original bytes」と書く文書を直す（文字列の置換）：
   - `examples/kivact-ui-pop/README.md`：「The catalog uses unchanged Inter and Noto Sans JP bytes from examples/shared-assets/fonts」→「The catalog uses unchanged Inter bytes and a subset of Noto Sans JP from examples/shared-assets/fonts」。
   - `examples/kivact-ui-pop/assets/fonts/SOURCE.md`：「Inter and Noto Sans JP use the unchanged canonical files and provenance in `examples/shared-assets/fonts/`.」→「Inter (unchanged) and Noto Sans JP (a subset) use the files and provenance in `examples/shared-assets/fonts/`.」。
   - `examples/kivact-ui-game/assets/fonts/SOURCE.md`：「with its original font bytes and OFL notice」→「as a subset, with its OFL notice」。
   - `examples/kivact-ui-creative/assets/fonts/SOURCE.md`：「Geist, Geist Mono and Noto Sans JP retain their original font bytes, licenses and hashes in examples/shared-assets/fonts/.」→「Geist and Geist Mono retain their original font bytes, and Noto Sans JP is a subset; their licenses and hashes are in examples/shared-assets/fonts/.」。
   - `examples/kivact-ui-basic/README.md`：「Inter、Geist Mono、Noto Sans JP を読み込み」→「Inter、Geist Mono、Noto Sans JP（catalog が使う文字の subset）を読み込み」。
8. `examples/kivact-ui-basic/tests/catalog.rs`：`use kivact::test::{LayoutCheck, ...};` の前に `use kivact::style::Position;` を足し、末尾に「付録 B」の全文を足す（試験名 `every_page_character_is_in_the_catalog_fonts`）。

完了時の確認：`dev/tools/window-lock.sh --timeout 1800 -- cargo test -p kivact-ui-basic-catalog`（新しい試験を含め全部成功）、`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- layout_check`、`cargo check -p kivact-ui-pop-catalog -p kivact-ui-game-catalog -p kivact-ui-creative-catalog --all-targets`（package 名は各 `Cargo.toml` の `name`）。

後の wave の注意（作業指示書 CP1・CP2 に足す 1 行）：「`every_page_character_is_in_the_catalog_fonts` が文字の名前を挙げて失敗したら、その文字を足した unit の lane の fix として `python dev/tools/subset-catalog-font.py` を実行し、subset と `SHA256SUMS.txt` の行を commit する（期待値の変更ではない）。」

### 所有ファイルの変更

付属書の所有ファイルに次を足す：`examples/shared-assets/fonts/{SOURCE.md,SHA256SUMS.txt}`、`examples/shared-assets/fonts/notosansjp/NotoSansJP-subset[wght].ttf`（新）と `NotoSansJP[wght].ttf`（削除）、`examples/kivact-ui-basic/tests/catalog.rs`、`examples/kivact-ui-{basic,pop}/README.md`、`examples/kivact-ui-{pop,game,creative}/assets/fonts/SOURCE.md`。付属書の「同じ場所の `SOURCE.md`」は `examples/shared-assets/fonts/SOURCE.md`（既存）を指すと確定する（`notosansjp/` に新しい SOURCE.md は作らない）。

### wave・lane

W2 lane C の 4 番目。W2 の他の lane と重ならない（F7 は e2e の `main.rs`・`frame_sources.rs`・`images_animated.rs`、G3 は root の README・docs）。

---

## G3（D6 3.5、W2 lane D）

### 判定：修正あり

HEAD の G1（`Gpu::demand` は `Deferred` の時だけ、`Presenter::demand_gpu(&mut self)`）と G2（`HostRequest::DemandGpu` の doc は G2 が既に直した）の上で、付属書 3.5 のコードは compile する（worktree で `kivact-core` の all-features・default、`kivact-public-api-tests` の 3 構成を確認）。直す点：

1. **marker を書かない**：付属書の `asked` の行の `// xtask-allow(state-fields): independent: ...` は、`asked: RefCell<BTreeSet<WindowId>>` が lint の数える field でない（`RefCell<BTreeSet>` は bool・Option・整数・`pending` 等の名前の集合のどれでもない）ため「unused marker」の error になる。`State` には数える field が一つも無い。marker の行を書かない。
2. **xtask の module 規則の例外が古くなる**：`view/drawable/requests.rs` が `gpu_interop` を参照しなくなるので、`cargo xtask check` が `module_rules.kivact-core.except: no reference needs view::drawable::requests -> gpu_interop` の error を出す（実測）。`dev/xtask/policy/workspace.toml` の `[module_rules.kivact-core]` の `except` から `"view::drawable::requests -> gpu_interop"` を消す。`"view::elements::viewport -> gpu_interop"` は残す（viewport は `GpuFrame` と `adapter()` を使い続ける）。
3. **型の doc が G1 で直っていない**：G1 の所有に `crates/kivact-types/src/gpu.rs`（doc）があったが ef29066e は触っていない。G3 で直す。
4. viewport の `content` の memo と `draw` の closure は `gpu_frame` が `Some` の時だけ `adapter()` を読む（`Option::filter`）。GPU frame は import か shared image からしか作れず、どちらも先に `adapter()` を読んで需要を出し済み（`asked`）なので、memo の中から effect が出ることは無い。付属書の設計のままでよい。

### 修正（付属書 3.5 に対する差分。他は付属書どおり）

- `registry.rs`：import は `use crate::view::state::{ViewState, current_window};`、`use kivact_engine::contract::HostRequest;`、`use kivact_types::{ExternalImageId, ExternalImport, GpuAdapter, RequestId, WindowId};`、`use std::collections::{BTreeMap, BTreeSet, HashMap};`。`State` の field は doc だけ付けて marker を付けない：

```rust
    /// The windows whose GPU device the App asked for, once each.
    asked: RefCell<BTreeSet<WindowId>>,
```

  `State::demand` は付属書のコードを `impl State { .. fn of(..) }` の同じ `impl` の中（`of` の後）に置く。`adapter()` の doc は、既存の「A new [`GpuAdapter::generation`] means a new device: ...」の段落の後、「Like `platform::window_size`, ...」の段落の前に、次の段落を足す：

```rust
/// Reading it asks the window to start its GPU device when the device
/// waits until it is needed (`GpuStart::OnDemand`, the default): it is
/// `None` until the device is ready.
///
```

- `gpu_interop.rs`：`mod demand;` と `pub(crate) use demand::demand_gpu;`（とその後の空行 1 行）を消す。`git rm crates/kivact-core/src/gpu_interop/demand.rs`。
- `import/image.rs` の `import_in`：コメント行と `crate::gpu_interop::demand::demand_gpu(rt, window);` の 2 行を消す（`window` はその後の `register` が使うので残す）。
- `share/app_side.rs` の `shared_image`：`let rt = owner.runtime();`、コメント 2 行、`demand_gpu` の行の 4 行を、次の 2 行のコメントにする：

```rust
    // Reading the adapter (below) asks for the window's device; the renewal
    // effect makes a GPU request once it is ready.
```

- `view/elements/viewport.rs` の `gpu_frame`：`#[track_caller]` とコメント・`owner`・`rt`・`demand_gpu` の 5 行を消し、`self.gpu_frame = frame.into_value(); self` だけにする（`#[cfg(feature = "gpu-interop")]` と doc は残す）。
- `view/drawable/requests.rs`：`use kivact_types::{DrawableId, WindowId};` → `use kivact_types::DrawableId;`、`send_requests` の引数 `window: WindowId,` を消し、`// The App exports as a GPU texture ...` と `if matches!(mode, ExportRequestMode::Gpu(_)) { .. }` の 4 行を消す。`view/drawable/handle.rs`：`let window = crate::view::state::current_window(owner);` を消し、`send_requests(app, id, window, &sending, &sent);` → `send_requests(app, id, &sending, &sent);`。
- `dev/xtask/policy/workspace.toml`：上の 2。
- `crates/kivact-types/src/gpu.rs` の `GpuStart::OnDemand` の doc を次にする：

```rust
    /// Frames are drawn on the CPU until the window needs the GPU: a user
    /// shader drawn, the App reading the window's adapter to share GPU
    /// textures, or CPU frames that keep missing half the display period
    /// (the default).
```

- 文書（付属書の「同じ語にする」を確定）：
  - `docs/for-kivact-user/en/how-to-guide/gpu-texture-sharing.md` の「a composition with a user shader, a GPU export, GPU texture sharing, or CPU
frames that keep missing half the display period.」→「a user shader drawn, the App reading `gpu_interop::adapter()` (a `shared_image` does, and an engine that imports textures or receives a drawable's export reads it first), or CPU frames that keep missing half the display period.」（改行位置は自由）。
  - `docs/for-kivact-user/jp/how-to-guide/gpu-texture-sharing.md` の「必要とするのは、ユーザーの shader を含む composition、GPU export、GPU texture の共有、または CPU の frame が表示周期の半分を続けて超えたときです。」→「必要とするのは、ユーザーの shader を描いたとき、App が `gpu_interop::adapter()` を読んだとき（`shared_image` が読みます。texture を import する engine や drawable の export を受け取る engine も先に読みます）、または CPU の frame が表示周期の半分を続けて超えたときです。」
  - `README.md` の `gpu` の行の「by default the device starts when the window needs it (`GpuStart::OnDemand`).」→「by default the device starts when the window needs it: a user shader drawn, or the App reading `gpu_interop::adapter()` (`GpuStart::OnDemand`).」
  - `README_JP.md` の「既定ではウィンドウが GPU を必要としたときに device を起動します (`GpuStart::OnDemand`)。」→「既定ではウィンドウが GPU を必要としたとき（ユーザーの shader を描いた、App が `gpu_interop::adapter()` を読んだ）に device を起動します (`GpuStart::OnDemand`)。」
  - `docs/for-kivact-user/en/README.md` の「(a user shader, a GPU export or GPU texture sharing; CPU frames
that keep missing half the display period start it too)」→「(a user shader drawn, or the App reading `gpu_interop::adapter()` to share GPU textures; CPU frames that keep missing half the display period start it too)」。
  - `docs/for-kivact-user/jp/README.md` の「（ユーザーの shader、GPU export、GPU texture の共有）」→「（ユーザーの shader を描いた、GPU texture の共有のために App が `gpu_interop::adapter()` を読んだ）」。

完了時の確認（付属書 P6-3 の行に足す）：`cargo check -p kivact-core --all-targets --all-features`、`cargo check -p kivact-core --all-targets`、`cargo check -p kivact-public-api-tests --tests --features gpu-interop`、`cargo xtask check --no-write --only state-fields --crate kivact-core`、`cargo xtask check --no-write --only module-rules --crate kivact-core`（規則名は `cargo xtask check --help` の一覧の module 規則のもの。分からなければ `--only` を付けずに `--crate kivact-core`）。

### 所有ファイルの変更

付属書 P6-3 の所有に `dev/xtask/policy/workspace.toml`（`except` の 1 要素だけ）と `crates/kivact-types/src/gpu.rs`（doc だけ）を足す。W2 で他にこの 2 ファイルを持つ unit は無い（D2 の AH1 は policy を変えない）。

---

## G4（D6 3.6、W3 lane F）

### 判定：修正あり（ファイルを 2 つに分ける、共有画像の大きさを直す）

HEAD の G2 の API（`kivact::test::Renderer`、`Screen::renderer`、`Screen::wait_for_gpu`、`RenderOptions::gpu_on_demand`、`gpu_unavailable`）は付属書と名前・形が一致し、付属書の E2E のコードは G3 の後にそのまま compile する（`--features gpu-interop`、既定、`--no-default-features` の 3 構成で clippy 0）。ただし次の 4 点を直す（付録 C・D が直した全文）：

1. **xtask の `style/non-leaf-implementation`**：付属書の `gpu_on_demand.rs` は中に `mod shared { .. }` を持つので、実装を含む非葉の module になり、11 件の error になる（実測）。共有の試験を兄弟の module `gpu_on_demand_shared.rs` に移し、`main.rs` で cfg を付けて宣言する（既存の `gpu_interop.rs` と同じ形）。共通の helper は `pub(crate)` にする。
2. **`main.rs` の位置**：付属書の「`mod focus;` の次」は rustfmt の並べ替え（`style/format` の error、実測）に反する。`mod gpu_interop;` の次に置く。
3. **共有画像の試験が必ず落ちる（実 GPU で実行して確認）**：付属書の `W = 128`、`H = 64` では `Producer::new` が `"the pattern does not fit 128 x 64"` で失敗する（`gpu-interop-producer` の `pattern::fits` は幅 200 以上・高さ 48 以上）。幅 200 ちょうどでは `pattern::check` の「clear column」（x = 68）が中央の block 群（x = 68 から）に重なり `Verdict::Torn` になる（実測）。既存の `gpu_interop.rs` の `W2 = 224`、`H2 = 96` と同じ大きさにし、window も同じ大きさにする（`mount` の先頭で `options.window_size(..)`）。この形で 4 本とも成功（`ARVANE_RENDERER=cpu` でも 4 本成功、続けて 2 回再実行して成功）。
4. `RendererChange::GpuStarting { cause }`・`GpuStartCause` は G1 で付属書どおりの名前・形で入っており、試験 host の `Observed` の写像（`Start { gpu: true } | GpuStarting { .. }` → `GpuStarting`）も付属書どおり。E2E の期待（`Cpu` → `GpuStarting`/`Gpu` → `wait_for_gpu` → `Gpu`）は HEAD の G1・G2 の振る舞いと一致した（実行で確認）。G4 のために G1・G2 を直す必要は無い。

### 修正（付属書 3.6 のコードと `main.rs` の指示を差し替える）

- `dev/tests/kivact-public-api/tests/e2e/main.rs`：次の 2 行（cfg 付きの方を含めて 3 行）を `#[cfg(...)] mod gpu_interop;` の直後に足す：

```rust
mod gpu_on_demand;
#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
mod gpu_on_demand_shared;
```

- `dev/tests/kivact-public-api/tests/e2e/gpu_on_demand.rs`：「付録 C」の全文（新規）。
- `dev/tests/kivact-public-api/tests/e2e/gpu_on_demand_shared.rs`：「付録 D」の全文（新規）。
- 試験名は付属書と同じ 4 本（`gpu_on_demand::a_shader_starts_the_gpu_on_demand`、`gpu_on_demand::a_cpu_window_never_starts_the_gpu_for_a_shader`、`gpu_on_demand_shared::a_shared_image_starts_the_gpu_on_demand`、`gpu_on_demand_shared::a_cpu_window_shares_pixels_without_a_device`）。作業指示書 CP1・CP2 のコマンドの filter `gpu_on_demand` は両方の module に当たるので変えない。
- 完了時の確認：`cargo fmt -p kivact-public-api-tests` の後に `cargo clippy -p kivact-public-api-tests --tests --features gpu-interop -- -D warnings`、同 `--tests`、同 `--tests --no-default-features`、`cargo xtask check --no-write --crate kivact-public-api-tests`。実行は付属書どおり checkpoint で行うが、GPU のある機械では unit の中でも `KIVACT_TEST_MIRROR=required dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --features gpu-interop --test e2e -- gpu_on_demand` を実行してよい（結果は「検証の記録」）。

### 所有ファイルの変更

付属書 P6-4 の所有に `dev/tests/kivact-public-api/tests/e2e/gpu_on_demand_shared.rs`（新規）を足す。W3 で e2e の `main.rs` を持つ他の unit：S3（lane F、同じ lane なので順に行う：S3 → G4 でも G4 → S3 でもよいが、表の順 G4 → S3 のまま）。

---

## 全体

### wave・lane の組み直し（担当範囲）

| wave | lane | 変更前 | 変更後 |
|---|---|---|---|
| W2 | B | S5 → F7 | F7 |
| W2 | C | T4 → S8 → S9 | **S45**（`git cherry-pick 3c284b0b` ＋ S5 ＋ 試験 2 本 ＋ doc）→ T4 → S8 → S9 |
| W2 | D | M8 → M9 → T1 → G3 | 同じ。G3 の所有に `dev/xtask/policy/workspace.toml`・`crates/kivact-types/src/gpu.rs` を追加 |
| W2 | E | S7 | 同じ（手順を本書の S7 のコードに置き換え） |
| W3 | D | T6 → S6 | 同じ。S6 の所有に `docs/for-kivact-user/{en,jp}/README.md` の 1 文を追加 |
| W3 | F | G4 → S3 | 同じ。G4 の所有に `gpu_on_demand_shared.rs` を追加 |

ファイルの重なりの確認（同じ wave の別 lane）：W2 lane C の所有（kivact-types render、arvane-text、arvane-render、arvane engine と e2e text、arvane-asset、kivact-engine `lower/assets.rs`、root `Cargo.toml` profile、templates/basic profile、catalog の fonts と文書、`layout_check.rs`）は、lane A（AH1：engine の model・a11y・host）、lane B（F7：e2e `main.rs`・`frame_sources.rs`・`images_animated.rs`）、lane D（M8・M9・T1・G3：kivact-core の view・client・gpu_interop、engine `host/write/reader.rs`、root README・docs、xtask policy、`kivact-types/src/gpu.rs`）、lane E（S7：runtime と `view/flow/list.rs`）と重ならない。W3 の S6 の docs README は W3 の他 lane に無い。

### ユーザー（人間）にしか決められない項目

なし。

付記（親への申し送り）：冒頭の engine の `dead_code`（F4 の残り）は R3 の unit の確認コマンドを止めるので、W2 の始めまでに D1 担当の監査で直し方を確定すること。

---

## 検証の記録（worktree `W:/w/kv-audit-R3`、HEAD 2a3b06b8 ＋ 本書の全修正）

target `W:/kv-target/C`、`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`。

| 確認 | 結果 |
|---|---|
| S4 だけ cherry-pick して `cargo check --workspace --all-targets --all-features` | error 1 件（`kivact-engine/src/lower/assets.rs:256:46` E0308）のみ |
| S4＋S5 で同上 | error・warning 0 |
| `cargo clippy -p kivact-types -p arvane-text -p arvane-render -p arvane -p arvane-asset -p kivact-engine --all-targets --all-features -- -D warnings` | 0 |
| `cargo check --release -p kivact-bench --bin kv_startup`（S8 の profile） | `did not match`・`ambiguous` の警告なし |
| S6 後の `cargo tree`（bench no-default、bench nogpu、catalog nogpu） | wgpu 0。`-e features -i arvane-compose` に `"gpu"` なし |
| G3＋G4：`cargo clippy -p kivact-public-api-tests --tests` を `--features gpu-interop`・既定・`--no-default-features` で | 0 |
| S7：`cargo test -p kivact-runtime`、`cargo check -p kivact-core --all-targets --all-features` | 成功 |
| S7：`cargo clippy -p kivact-runtime --all-targets -- -D warnings` | engine の `dead_code`（冒頭）だけで失敗。R3 の変更に起因する指摘は 0 |
| `cargo xtask check --no-write`（全体、全修正の後） | error は W1 からの 3 件（`arvane-compose/src/cache.rs:53`、`kivact-core/src/view/mount/semantics.rs:14`、`kivact-runtime/src/reactive/data.rs:331`）だけ。途中で見つけた G3 の module 規則・G4 の non-leaf/format の error は本書の修正で消えた |
| `window-lock -- cargo test -p arvane --test e2e -- text` | 9 passed（新しい 2 本を含む） |
| `KIVACT_TEST_MIRROR=required window-lock -- cargo test -p kivact-public-api-tests --features gpu-interop --test e2e -- gpu_on_demand`（RTX 3060 Ti） | 4 passed（3 回）。付属書の `W=128,H=64` では `a_shared_image_starts_the_gpu_on_demand` が失敗（G4 の 3） |
| 同、`ARVANE_RENDERER=cpu` | 4 passed |
| `window-lock -- cargo test ... --test e2e -- launch_fonts layout_check svg gpu_interop drawable_export_gpu viewport` | launch_fonts・layout_check（subset の font）・svg・viewport・drawable_export_gpu は成功。`gpu_interop::*_on_dx12` の 2 本は `KIVACT_TEST_MIRROR` を付けないと `Screen::start` で失敗し、`KIVACT_TEST_MIRROR=required` を付けると成功（R3 と無関係の既存の前提） |
| `window-lock -- cargo test ... --test e2e -- flows:: disposal:: machine resource_reclaim`（S7） | 18 passed |
| `window-lock -- cargo test -p kivact-ui-basic-catalog`（S9） | 4 passed（`every_page_character_is_in_the_catalog_fonts` を含む） |
| `subset-catalog-font.py` を 2 回 | 同じ byte（決定的） |
| S9 の試験の陰性確認：page source に subset に無い `鬱` を一時的に足して実行 | `no catalog font draws ['鬱']: run dev/tools/subset-catalog-font.py` で失敗（漏れを検出できる）。元に戻した |

---

## 付録 A：`dev/tools/subset-catalog-font.py`

```python
"""Subset Noto Sans JP to the characters the catalogs and their tests use.

Run by hand, never by the build or CI, after adding text with characters the
catalogs did not use before (`examples/kivact-ui-basic`'s test
`every_page_character_is_in_the_catalog_fonts` fails until then):

    python dev/tools/subset-catalog-font.py

It reads the original Google Fonts file NotoSansJP[wght].ttf from the
working tree, or, once it is deleted, from the commit before the one that
deleted it (`--source <file>` reads another copy). The original's SHA-256 is
checked. The output replaces
examples/shared-assets/fonts/notosansjp/NotoSansJP-subset[wght].ttf. Needs
fontTools (`python -m pip install --user fonttools`).
"""

import argparse
import hashlib
import io
import pathlib
import subprocess
import sys

from fontTools import subset

ROOT = pathlib.Path(__file__).resolve().parents[2]
ORIGINAL = "examples/shared-assets/fonts/notosansjp/NotoSansJP[wght].ttf"
OUTPUT = ROOT / "examples/shared-assets/fonts/notosansjp/NotoSansJP-subset[wght].ttf"
ORIGINAL_SHA256 = "c2f3b4d463500a2ddcd3849cded1fceeb9fd6d1c32e6cbecd568453ba50fc68f"
SCANNED = ["examples/kivact-ui-*", "crates/kivact-ui-*", "dev/tests/kivact-public-api"]
SUFFIXES = {".rs", ".ftl", ".json", ".toml", ".md"}
# Characters ICU writes for ja-JP dates and times (the catalogs show calendars
# in Japanese), whether or not a source uses them.
EXTRA = "年月日火水木金土曜午前後時分秒"
RANGES = [(0x0020, 0x007E), (0x00A0, 0x00FF), (0x2000, 0x206F), (0x3000, 0x30FF), (0xFF00, 0xFFEF)]


def git(*args):
    """The standard output of git run at the repository root."""
    return subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True).stdout


def original(source):
    """The original font's bytes: `source`, the working tree's, or git's."""
    if source is not None:
        return source.read_bytes()
    path = ROOT / ORIGINAL
    if path.exists():
        return path.read_bytes()
    deleted = git("log", "--diff-filter=D", "--format=%H", "-1", "--", f":(literal){ORIGINAL}")
    deleted = deleted.decode().strip()
    if not deleted:
        sys.exit(f"{ORIGINAL} is neither in the working tree nor in the history")
    return git("show", f"{deleted}^:{ORIGINAL}")


def scanned_characters():
    """Every character above ASCII in the scanned sources."""
    found = set()
    for pattern in SCANNED:
        for top in sorted(ROOT.glob(pattern)):
            for path in sorted(top.rglob("*")):
                if path.suffix in SUFFIXES and path.is_file():
                    text = path.read_text(encoding="utf-8")
                    found.update(ord(c) for c in text if ord(c) >= 0x80)
    return found


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--source", type=pathlib.Path)
    args = parser.parse_args()
    data = original(args.source)
    if hashlib.sha256(data).hexdigest() != ORIGINAL_SHA256:
        sys.exit(f"not the original NotoSansJP[wght].ttf (SHA-256 {ORIGINAL_SHA256})")
    codepoints = scanned_characters()
    codepoints.update(ord(c) for c in EXTRA)
    for first, last in RANGES:
        codepoints.update(range(first, last + 1))
    options = subset.Options()
    options.layout_features = ["*"]
    options.name_IDs = ["*"]
    options.name_languages = ["*"]
    options.notdef_outline = True
    options.prune_unicode_ranges = False
    options.drop_tables = [t for t in options.drop_tables if t != "BASE"]
    font = subset.load_font(io.BytesIO(data), options)
    subsetter = subset.Subsetter(options)
    subsetter.populate(unicodes=sorted(codepoints))
    subsetter.subset(font)
    subset.save_font(font, str(OUTPUT), options)
    print(f"{OUTPUT.relative_to(ROOT)}: {OUTPUT.stat().st_size} bytes, {len(codepoints)} code points")


if __name__ == "__main__":
    main()
```

## 付録 B：`examples/kivact-ui-basic/tests/catalog.rs` の末尾に足すコード

```rust
/// The side of a cell of `every_page_character_is_in_the_catalog_fonts`.
const CELL: u32 = 40;
/// Cells per row of that test.
const COLUMNS: u32 = 25;

/// Whether `c` of a page source must be drawn by a catalog font: a visible
/// character above ASCII. Emoji (U+1F000 and above) are left to the
/// system's emoji font, which the catalog does not bundle.
fn bundled(c: char) -> bool {
    !c.is_ascii()
        && !c.is_whitespace()
        && !c.is_control()
        && c < '\u{1F000}'
        && !matches!(c, '\u{200B}'..='\u{200F}' | '\u{2028}'..='\u{202E}' | '\u{2060}'..='\u{206F}' | '\u{FE00}'..='\u{FE0F}' | '\u{FEFF}')
}

/// Cell `index` of the grid: `c` at its top left.
fn glyph_cell(index: u32, c: char) -> View {
    let (column, row) = (index % COLUMNS, index / COLUMNS);
    node()
        .child(text(c.to_string()))
        .style(
            style()
                .position(Position::Absolute)
                .left((column * CELL) as i32)
                .top((row * CELL) as i32)
                .size(CELL as i32, CELL as i32),
        )
        .into_view()
}

/// The pixels of cell `index` of `shot`.
fn cell_pixels(shot: &kivact::test::Screenshot, index: u32) -> Vec<u32> {
    let (x0, y0) = ((index % COLUMNS) * CELL, (index / COLUMNS) * CELL);
    (0..CELL)
        .flat_map(|y| (0..CELL).map(move |x| (x0 + x, y0 + y)))
        .map(|(x, y)| shot.pixel(x, y).expect("the cell is in the window"))
        .collect()
}

/// Every visible character above ASCII of the catalog's pages is drawn by
/// one of the catalog's fonts: none of them draws the missing glyph, which a
/// character no bundled font has gets (the first font's). A character added
/// to a page without running `dev/tools/subset-catalog-font.py` again fails.
#[test]
fn every_page_character_is_in_the_catalog_fonts() {
    let mut chars: Vec<char> = pages()
        .iter()
        .flat_map(|page| page.source.chars())
        .filter(|c| bundled(*c))
        .collect();
    chars.sort_unstable();
    chars.dedup();
    // Cell 0: a character no font has, drawn as the missing glyph.
    let probe = '\u{10FFFD}';
    let cells: Vec<char> = std::iter::once(probe)
        .chain(chars.iter().copied())
        .collect();
    let rows = (cells.len() as u32).div_ceil(COLUMNS);
    let grid = cells.clone();
    let screen = render_with(
        RenderOptions::with_theme(kivact_ui_basic::theme(ThemeMode::Light))
            .window_size(f64::from(COLUMNS * CELL), f64::from(rows * CELL))
            .scale(1.)
            .fonts(TestFonts::Only(
                fonts().iter().map(|&bytes| bytes.into()).collect(),
            )),
        move || {
            let cells: Vec<View> = grid
                .iter()
                .enumerate()
                .map(|(index, c)| glyph_cell(index as u32, *c))
                .collect();
            node().child(cells)
        },
    );
    let shot = screen.screenshot();
    let missing_glyph = cell_pixels(&shot, 0);
    assert!(
        missing_glyph.iter().any(|p| *p != missing_glyph[0]),
        "the missing glyph is drawn"
    );
    let missing: Vec<char> = (1..cells.len() as u32)
        .filter(|index| cell_pixels(&shot, *index) == missing_glyph)
        .map(|index| cells[index as usize])
        .collect();
    assert!(
        missing.is_empty(),
        "no catalog font draws {missing:?}: run dev/tools/subset-catalog-font.py"
    );
}
```

## 付録 C：`dev/tests/kivact-public-api/tests/e2e/gpu_on_demand.rs`

```rust
//! Role: The GPU device a window starts when it needs it (`GpuStart::OnDemand`).
//!
//! A window that may draw on the GPU starts on the CPU and starts its device
//! only when something needs it: a user shader drawn (the CPU draws its
//! fallback meanwhile), or the App reading the window's adapter, as a
//! `gpu_interop::shared_image` does. A window on the CPU only (the test
//! default, `ARVANE_RENDERER=cpu`) never starts one. The tests run where the
//! window can have a GPU device and are skipped, saying why, elsewhere
//! (`gpu_unavailable`). Under `ARVANE_RENDERER=cpu` the on-demand tests check
//! that no device starts instead. The shared image's tests are in
//! `gpu_on_demand_shared`.
#![allow(
    clippy::print_stderr,
    reason = "report unavailable optional hardware explicitly"
)]

use std::time::Duration;

use super::paint::{BLUE, GREEN, RED, at, expect};
use super::support::mount_with;
use kivact::draw::{Paint, PaintShader};
use kivact::prelude::*;
use kivact::style::rgb;
use kivact::test::{MirrorMode, RenderOptions, Renderer, gpu_unavailable};

/// A paint shader that returns blue (its fallback in the tests is red).
const BLUE_PAINT: &str = "fn paint(position: vec2<f32>, context: Context) -> vec4<f32> {\n    return vec4<f32>(0.0, 0.0, 1.0, 1.0);\n}\n";

/// The real time a device that must not start is given to start.
pub(crate) const GRACE: Duration = Duration::from_millis(300);

/// Whether the GPU lane can run here; says why when it cannot.
pub(crate) fn gpu_lane(name: &str) -> bool {
    match gpu_unavailable() {
        Some(reason) => {
            eprintln!("SKIPPED {name}: {reason}");
            false
        }
        None => true,
    }
}

/// `ARVANE_RENDERER=cpu`: every window draws on the CPU only.
pub(crate) fn cpu_forced() -> bool {
    std::env::var("ARVANE_RENDERER").as_deref() == Ok("cpu")
}

/// A window that may draw on the GPU, its device started when needed.
pub(crate) fn on_demand() -> RenderOptions {
    RenderOptions::from_env()
        .window_size(200.0, 100.0)
        .gpu_on_demand()
        .mirror(MirrorMode::Required)
}

/// A real window on the CPU only (the test default).
pub(crate) fn cpu_only() -> RenderOptions {
    RenderOptions::default()
        .window_size(200.0, 100.0)
        .mirror(MirrorMode::Required)
}

/// A green box, and beside it, while `shaded`, a box painted by `shader`
/// (blue on the GPU, its red fallback on the CPU).
fn scene(shaded: Signal<bool>, shader: PaintShader) -> impl IntoView {
    node()
        .child((
            node().style(at(0, 0, 100, 100).background(rgb(GREEN))),
            show(shaded, move || {
                node().style(
                    at(100, 0, 100, 100).background(Paint::from(shader.paint().fallback(rgb(RED)))),
                )
            }),
        ))
        .style(style().size(200, 100).background(rgb(0xffffff)))
}

/// A shader drawn in a window started with `options` is drawn by its
/// fallback, and no device starts, given the time to.
#[track_caller]
fn shader_stays_on_cpu(options: RenderOptions) {
    let shader = PaintShader::wgsl(BLUE_PAINT).expect("a valid paint shader");
    let (screen, shaded) = mount_with(options, move || {
        let shaded = signal(false);
        (shaded, scene(shaded, shader))
    });
    shaded.set(true);
    screen.run_until_idle();
    std::thread::sleep(GRACE);
    screen.run_until_idle();
    assert_eq!(
        screen.renderer(),
        Renderer::Cpu,
        "a window on the CPU only starts no device"
    );
    expect(
        &screen.screenshot(),
        150,
        50,
        RED,
        2,
        "the CPU draws the fallback",
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A window waits on the CPU until it draws a user shader; the frame that
/// draws it first starts the device (drawn by the fallback meanwhile), and
/// the GPU then draws the shader.
#[test]
fn a_shader_starts_the_gpu_on_demand() {
    if !gpu_lane("a_shader_starts_the_gpu_on_demand") {
        return;
    }
    if cpu_forced() {
        return shader_stays_on_cpu(on_demand());
    }
    let shader = PaintShader::wgsl(BLUE_PAINT).expect("a valid paint shader");
    let (screen, shaded) = mount_with(on_demand(), move || {
        let shaded = signal(false);
        (shaded, scene(shaded, shader))
    });
    // Nothing needs the GPU, and an idle window's cheap CPU frames do not
    // start the device.
    screen.advance(Duration::from_millis(500));
    assert_eq!(
        screen.renderer(),
        Renderer::Cpu,
        "no device before the window needs one"
    );
    expect(&screen.screenshot(), 50, 50, GREEN, 2, "the CPU's frame");
    shaded.set(true);
    screen.run_until_idle();
    match screen.renderer() {
        Renderer::GpuStarting => expect(
            &screen.screenshot(),
            150,
            50,
            RED,
            2,
            "the CPU draws the fallback while the device starts",
        ),
        Renderer::Gpu => {}
        other => panic!("a drawn shader starts the device, not {other:?}"),
    }
    screen.wait_for_gpu();
    assert_eq!(screen.renderer(), Renderer::Gpu);
    let shot = screen.screenshot();
    expect(&shot, 150, 50, BLUE, 2, "the GPU draws the shader");
    expect(&shot, 50, 50, GREEN, 2, "and the rest of the window");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A window on the CPU only (as with `ARVANE_RENDERER=cpu`) draws a user
/// shader by its fallback and never starts a device.
#[test]
fn a_cpu_window_never_starts_the_gpu_for_a_shader() {
    if !gpu_lane("a_cpu_window_never_starts_the_gpu_for_a_shader") {
        return;
    }
    shader_stays_on_cpu(cpu_only());
}
```

## 付録 D：`dev/tests/kivact-public-api/tests/e2e/gpu_on_demand_shared.rs`

```rust
//! Role: The GPU device a `shared_image` starts when its window waits for one.
//!
//! `shared_image` reads the window's adapter: a window that waits starts
//! its device, and the engine's GPU frames arrive; a window on the CPU
//! only asks for pixels and starts none.
#![allow(
    unsafe_code,
    reason = "providing shared textures is an unsafe fn: the test engine keeps its contract"
)]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpu_interop_producer::{Producer, pattern};
use kivact::draw::ImageOptions;
use kivact::geometry::Rect;
use kivact::gpu_interop::{self, ShareMode, ShareStatus, SharedImage};
use kivact::prelude::*;
use kivact::test::{RenderOptions, Renderer, Screen};
use kivact::view::canvas;

use super::gpu_on_demand::{GRACE, cpu_forced, cpu_only, gpu_lane, on_demand};
use super::support::mount_with;

/// The image's size, and the window's: the producer's pattern and its check
/// fit it (`pattern::fits`, and wider than 200 for the check's clear column).
const W: u32 = 224;
const H: u32 = 96;
/// The longest a step of the exchange waits (real time).
const WAIT: Duration = Duration::from_secs(10);

/// A window of the image's size, started with `options`, that makes a
/// `shared_image` once `go` is set and draws its GPU frame at its own size.
#[track_caller]
fn mount(options: RenderOptions) -> (Screen, Signal<bool>, Rc<RefCell<Option<SharedImage>>>) {
    let options = options.window_size(f64::from(W), f64::from(H));
    let image = Rc::new(RefCell::new(None::<SharedImage>));
    let handed = image.clone();
    let (screen, go) = mount_with(options, move || {
        let go = signal(false);
        let view = show(go, move || {
            let shared = gpu_interop::shared_image(W, H);
            *handed.borrow_mut() = Some(shared.clone());
            canvas(move |d, _| {
                if let Some(f) = shared.frame() {
                    let rect = Rect::new(0.0, 0.0, f64::from(W), f64::from(H));
                    let options = ImageOptions {
                        pixelated: true,
                        ..ImageOptions::default()
                    };
                    d.gpu_frame(&f, rect, options);
                }
            })
            .style(style().size(W, H))
        });
        (go, view)
    });
    (screen, go, image)
}

/// A shared image in a window started with `options` asks for pixels,
/// and no device starts, given the time to.
#[track_caller]
fn shares_pixels_on_cpu(options: RenderOptions) {
    let (screen, go, image) = mount(options);
    go.set(true);
    screen.run_until_idle();
    std::thread::sleep(GRACE);
    screen.run_until_idle();
    let image = image.borrow().clone().expect("the window made its image");
    assert_eq!(
        image.status(),
        ShareStatus::Pixels,
        "a window on the CPU asks for pixels"
    );
    assert_eq!(screen.renderer(), Renderer::Cpu, "and starts no device");
    let request = image.link().poll_request().expect("the image asks");
    assert!(
        matches!(request.mode, ShareMode::Pixels),
        "{:?}",
        request.mode
    );
}

/// Making a shared image starts the waiting device; the image then asks
/// for textures on the window's adapter, and the engine's frame arrives
/// and is drawn by the GPU.
#[test]
fn a_shared_image_starts_the_gpu_on_demand() {
    if !gpu_lane("a_shared_image_starts_the_gpu_on_demand") {
        return;
    }
    if cpu_forced() {
        return shares_pixels_on_cpu(on_demand());
    }
    let (screen, go, image) = mount(on_demand());
    assert_eq!(
        screen.renderer(),
        Renderer::Cpu,
        "nothing needs the GPU yet"
    );
    go.set(true);
    screen.run_until_idle();
    assert!(
        matches!(screen.renderer(), Renderer::GpuStarting | Renderer::Gpu),
        "making a shared image starts the device, not {:?}",
        screen.renderer()
    );
    screen.wait_for_gpu();
    let image = image.borrow().clone().expect("the window made its image");
    let link = image.link();
    // A request of pixels may come first, while the device started.
    let start = Instant::now();
    let (request, adapter) = loop {
        screen.run_until_idle();
        if let Some(request) = link.poll_request()
            && let ShareMode::Gpu(adapter) = &request.mode
        {
            let adapter = adapter.clone();
            break (request, adapter);
        }
        assert!(
            start.elapsed() < WAIT,
            "the image asks for textures on the ready device"
        );
        std::thread::sleep(Duration::from_millis(5));
    };
    assert_eq!((request.width, request.height), (W, H));
    let mut producer = Producer::new(&adapter, Default::default(), W, H)
        .expect("the engine starts on the adapter");
    let shared = producer
        .shared_textures()
        .expect("the engine shares its pool");
    // SAFETY: the test engine keeps the contract of `SharedTextures`.
    unsafe { link.provide(&request, shared) }.expect("the newest request takes the pool");
    let start = Instant::now();
    while image.status() != ShareStatus::Ready {
        assert!(
            start.elapsed() < WAIT,
            "the pool is imported: {:?}",
            image.status()
        );
        std::thread::sleep(Duration::from_millis(5));
        screen.run_until_idle();
    }
    let frame = producer.draw(0).expect("the engine draws");
    link.publish(&request, frame.slot, frame.value);
    screen.run_until_idle();
    assert!(image.frame().is_some(), "the GPU frame arrived");
    let shot = screen.screenshot();
    assert_eq!(
        pattern::check(&shot.pixels, shot.width as usize, (0, 0), (W, H), 0, 0),
        pattern::Verdict::Ok,
        "the window draws the engine's frame"
    );
    assert_eq!(screen.renderer(), Renderer::Gpu);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    go.set(false);
    screen.run_until_idle();
    assert!(
        producer.wait_release(frame.value, Duration::from_secs(5)),
        "the dropped image releases its frame (release {})",
        producer.release_value()
    );
}

/// A shared image in a window on the CPU only asks for pixels and starts
/// no device (the App's demand is ignored on the CPU).
#[test]
fn a_cpu_window_shares_pixels_without_a_device() {
    if !gpu_lane("a_cpu_window_shares_pixels_without_a_device") {
        return;
    }
    shares_pixels_on_cpu(cpu_only());
}
```
