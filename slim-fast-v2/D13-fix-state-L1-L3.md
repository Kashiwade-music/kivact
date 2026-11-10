# 監査 R5：W6（状態機械）lane L1〜L3 の全 unit

> **V2.1 修正付属書 D13**（再監査 R5 の成果物、HEAD `2a3b06b8`＝W1 の後に対して書いた）。範囲：W6 の lane L1〜L3（状態機械）。
> - **優先順位**：本書は付属書 D1〜D8 より優先する。本書の unit の節の「修正」は、作業指示書の行の「本書の追加の指示」と同じ扱い。本書と V2 設計書 4 章（DEC）が食い違う場合は V2 設計書が正（V2 設計書 4 章の DEC-18 の名前は `while_open(factory)`）。
> - 本書の「全体」節の wave・lane の組み直しは、作業指示書（V2.1）に反映済み。実施順・所有ファイルは作業指示書が正。
> - 文中の worktree（`W:/w/kv-audit-*`）・scratchpad の path は監査の試作の場所で、実装に必要ではない（試作の差分は参考で、実装は本書の文に従う）。

対象：作業指示書 9 章の L1（XT-1 → P8-POP → LY → ST・ED → P8-L1 → P8-XT）、L2（NA → P8-NA → AS → EN-1 → EN-2 → P8-EN → KT → P8-KT）、L3（RT → P8-RT → CO → P8-CO → P8-ARV → P8-GPU）。HEAD 2a3b06b8 のコード、付属書 D5（本文・付録 P5-A〜F）、D8（3 章・付録 A-G*・付録 B）、W2〜W5 の付属書（D1〜D4・D6・D7）の所有ファイルと手順を突き合わせた。

調べ方（再現できるように書く）：
- 付録 B の行（unit 列が L1〜L3 の P8 unit の 185 行）を機械的に取り出し、73c36c7c の marker 行の文が HEAD に残っているか、直後に field があるか、W1 で file が変わったかを照合した（`git diff --name-only 73c36c7c..HEAD`。73c36c7c..eef6dc9f は文書だけなので、コードの差は W1 の commit だけ）。**付録 B の全 916 行のうち、W1 で marker の文が消えた行は 0**。L1〜L3 の行で W1 で行がずれたのは 21 行（下の各節に書く）。
- W2〜W5 の unit の所有ファイル（D1〜D4・D6・D7 の 6 章と作業指示書 4〜8 章）と、L1〜L3 の unit の所有ファイルの交差を全部列挙し、交差した 25 ファイルについて W2〜W5 の手順がその unit の対象 field・行を変えるかを読んだ。
- 検証（worktree `W:/w/kv-audit-R5`、HEAD に detach。最後に削除）：(1) LY の追加 E2E `grid_items_justify_self_within_their_area` を HEAD のコードで実行、(2) XT-1 の lint の修正と試験を入れて `cargo test -p xtask state_fields` と全 crate の `state-fields` を実行、(3) P8-XT の kv_heap の書換えで `state-fields --crate kivact-bench`、ほか下の各節に書いたもの。結果は各節の「検証」に書く。

用語：「行を探す」＝ 作業指示書 0.2 の 10 のとおり、行番号でなく `owner.field`・関数名で探す。以下で「HEAD の行」と書いた行番号は 2a3b06b8 のもの（参考）。

---

## XT-1（付属書 D5 本文 3.4 X1・X2、付録 P5-A A2、付録 P5-C C5、W6 lane L1）

### 判定：修正あり（試験のコード片の確定と、確認コマンドの追加）

### 修正（作業指示書の行の「本書の追加の指示」）
- `dev/xtask/src/state_fields/classify.rs` の `is_setting_payload` の腕を `"Rc" | "Arc" | "Box" => single_type_argument(segment).is_some_and(is_fn_object),` にする（D5 3.4 X1 のとおり。HEAD のコードと一致を確認した）。
- `dev/xtask/src/state_fields/locals.rs` の `is_scalar` の Path の腕を `syn::Expr::Path(_) => is_none(expr),` にし、`is_none` を D5 付録 P5-C C5 の 3. のコードのとおりに置き換える（HEAD の `is_scalar`・`is_none` は C5 の「今」と一字一句同じ。`is_none` の他の呼出し元 `RefCell::new(None)` 系の 2 箇所はそのままでよい）。
- `dev/xtask/src/state_fields/tests.rs` の末尾に次の 2 本を足す。付録 P5-A A2 の `an_arc_callback_option_is_a_setting` は本体がコメントだけなので、**次の本体を使う**（同じファイルの `option_callbacks_are_settings_but_wrapped_options_count` の書き方に合わせた）：

```rust
#[test]
fn an_arc_callback_option_is_a_setting() {
    let errors = run(
        "struct Probe { a: Option<u8>, b: Option<Job>, f: Option<std::sync::Arc<dyn Fn(u32) -> bool + Send + Sync>> }",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("Probe combines state fields: a, b"),
        "{errors:?}"
    );
    assert!(!errors[0].contains(", f"), "{errors:?}");
}

#[test]
fn enum_variants_named_none_are_not_option_locals() {
    let text = "fn sample() {\nlet section = signal(0usize);\nlet overlay = signal(Overlay::None);\n}";
    assert!(run(text, "", &[]).is_empty());
    let text = "fn sample() {\nlet section = signal(0usize);\nlet empty = signal(Option::None);\n}";
    let errors = run(text, "", &[]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("function combines state locals: section, empty"));
}
```
- lint の修正の**後**に、`examples/kivact-ui-pop/src/pop/shell.rs` の `catalog_with_mode` の中の次の 1 行を削除する（`let selected = signal(0usize);` の直前の行）：
  `// xtask-allow(state-fields): independent: the scroll-spy section is unrelated to which overlay is open`
- 完了時の確認に次を足す（X1・X2 が他の crate に unused marker を作らないことの確認）：`cargo xtask check --no-write --only state-fields --crate kivact-core`、`--crate arvane-native`、`--crate arvane-shell`。期待：kivact-core は W1 の既知の error 1 件（`semantics.rs` の `Plan`）だけ、他は 0 件。

### 検証
- 上の変更を worktree に入れ、`cargo test -p xtask state_fields` と、`cargo xtask check --no-write --only state-fields`（全 crate）を実行した。結果は本節末の「検証結果」に記す。

### 所有ファイルの変更
なし。

### wave・lane・順序の変更
なし。

---

## P8-POP（付属書 D8 6 章 L1、付録 B の unit 列 P8-POP の 13 行、W6 lane L1）

### 判定：変更なし

### 修正
- なし。13 行（`components.rs` 4、`data.rs` 4、`mission.rs` 1、`patterns.rs` 2、`shell.rs` 2）はすべて HEAD で 73c36c7c と同じ行・同じ文で残っている（W1〜W5 はこれらのファイルを変えない）。
- XT-1 の X2 は `data.rs` の `table::sort`（`signal(None::<(usize, SortOrder)>)`）の数え方を変えない（path の segment は `None` 1 個で、`is_none` は引き続き真）。したがって marker はすべて「used」のまま。

---

## LY（付属書 D5 付録 P5-B B5、W6 lane L1）

### 判定：修正あり（W4 の T5 との結合）

### 修正
- **`GridArea` の derive は T5 が足した `PartialEq` を残す**。D5 B5 のコード片の `#[derive(Clone, Copy, Debug)]` ではなく、次にする（T5 = D7 3.9 M3-9 が `NodeDisplay`・`GridArea`・`NodeStyle` に `PartialEq` を足す。`GridArea` から落とすと `NodeStyle` の `PartialEq` の derive が compile しない）：

```rust
/// An item's place in its grid container: both axes and its row-axis
/// alignment (most nodes are auto-placed in flex parents and have none).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GridArea {
    pub(crate) column: GridLine,
    pub(crate) row: GridLine,
    /// CSS `justify-self` (`None`: the container's `justify_items`).
    pub(crate) justify_self: Option<AlignSelf>,
}
```
- T5 の後、`NodeLayoutValues.styles` は `Vec<Rc<NodeStyle>>` になっているが、LY が変える `NodeStyle` の定義・`Default`・`GridItemStyle::justify_self`・`convert.rs` の `taffy_style` は `Rc` に関係しない。D5 B5 のその他のコード片はそのまま使う（HEAD の `node.rs`・`convert.rs` の該当行と一致を確認）。
- 追加 E2E `grid_items_justify_self_within_their_area` は **HEAD のコードで通ることを監査で確認した**（下の検証結果）。D5 の「HEAD で通らなければ STOP」の手順は不要になったので行わない。変更の後に通ることだけを確認する。

### 検証
- worktree で D5 のコード片の E2E を `grid.rs` の末尾に足し、`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- grid::` を HEAD のコードで実行した。結果は末尾の「検証結果」。

---

## ST・ED（付属書 D5 付録 P5-B B10、付録 P5-A A4、W6 lane L1）

### 判定：変更なし

### 修正
- なし。`crates/kivact-style/src/sheet/fields.rs` の `StateFields.group`、`crates/kivact-editing/src/edit_state.rs` の `EditState`（`max_length`・`log`・`damage`）は HEAD で D5 のとおり。W1〜W5 はこの 2 ファイルを変えない。

---

## P8-L1（付属書 D8 6 章 L1、付録 A-G7 W1（P8-W18）、7 章 #4、付録 B の P8-L1 の 31 行、W6 lane L1）

### 判定：修正あり（確認コマンドだけ。コードの手順は変更なし）

### 修正
- `crates/kivact-layout/src/columns.rs` は W1 の F2 が変え、W4 の T5 が変える（`styles` の `Rc`、`performed` の `SlotMap`、`default_style`・`recent_styles`）。どちらも `Counted`・`ChildCursor::at_epoch` を変えない（HEAD で確認：`Counted` は 1 行下にずれただけ）。付録 A-G7 W1 の手順はそのまま行う。`traversal.rs` の `flow_child`・`filtered_flow_child`・`child_ids` も HEAD で付録の「今」と一致。
- 付録 B の P8-L1 の他の行（style・editing・i18n・devtools）は全て HEAD で同じ行に残っている。
- 7 章 #4 の doc（`FlowFields.display`）は `sheet/fields.rs` の `display` field の doc の「set by `grid(..)` only」の文で探す。
- **確認のコマンドを差し替える**：付属書の `cargo test -p kivact-layout` は、kivact-layout に試験が 1 本も無い（`#[test]` 0 件、`tests/` も無い）ので P8-W18 を確かめない（付録 A-G7 W1 の「overlay・fixed で子を除く layout の unit test」は存在しない）。`Counted` の `Filtered` の経路は overlay・fixed の子を持つ親で通るので、次を足す：`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- overlay:: interaction_dialog:: apg_menu:: grid::`。`cargo clippy -p kivact-layout --all-targets -- -D warnings` と 5 package の state-fields は付属書のとおり。

---

## P8-XT（付属書 D8 6 章 L1、付録 A-G7 W3（P8-W20）・W4（P8-W21）、付録 B の P8-XT の 8 行、W6 lane L1）

### 判定：修正あり（W1 の M12 と W5 の G7 との結合）

### 修正
- **kv_heap（W1 の M12 の後のコード）**：付録 A-G7 W3 の書換え表の `dump` の行は、M12 で `dump(rows, chain, top)` になった後のコードで次のとおり行う（`dump` の中の `let sites = SITES ... .collect();` の文だけを置き換え、それ以外の M12 のコード（`owner`、`by_fn`、`TOP`・`CHAIN`）は変えない）：

```rust
    let heap = HEAP
        .lock()
        .expect("invariant: the probe never poisons its lock");
    let sites: Vec<&Site> = heap
        .iter()
        .flat_map(|h| h.sites.values())
        .filter(|s| s.live > 0)
        .collect();
```
  `record`・`forget` と static の置換は付録 A-G7 W3 のコード片のとおり（HEAD の `record`・`forget` は M12 で変わっておらず、付録の「今」と一致）。static の 3 本の marker の文も付録のとおり（`TRACKING` と `INSIDE` の文は付録 B の行と一字一句同じ）。
- **`Source.library` の読み手**：付録 A-G7 W4 の表の読み手の行に加えて、W5 の G7（D6 C-12）が `dev/xtask/src/api_shape.rs` に足した規則の中の読みも直す。手順：`dev/xtask/src` で `\.library\b` を grep し、後ろに `(` が無い（field の読み）全ての箇所を `.library()` にする（G7 の後も対象は `api_shape.rs`・`canonical_path.rs`・`name_collision.rs`・`structural_options.rs`・`public_surface.rs`・`public_type_edges.rs`・`state_fields/check.rs`・`check_source.rs` の試験の中だけで、所有ファイルの外には無いことを HEAD で確認した）。
- 完了時の確認の kv_heap の build は次のとおりに確定する：`CARGO_TARGET_DIR=W:/kv-target/A-heap cargo build --release -p kivact-bench --bin kv_heap`、続けて `rm -rf W:/kv-target/A-heap`。加えて `cargo xtask check --no-write --only state-fields --crate kivact-bench` と `--crate xtask` を実行する（どちらも error 0）。

### 検証
- worktree で kv_heap の書換え（static・`record`・`forget`・`dump`）を入れて `state-fields --crate kivact-bench` を実行した。結果は末尾。

---

## NA（付属書 D5 付録 P5-A A1、W6 lane L2）

### 判定：修正あり（確認コマンド）

### 修正
- コードの手順は D5 A1 のとおり（`NativeInput.waker` を `ShellWaker` に、`NativeInput::new` の引数、`wake`、`port/winit/open.rs` の `Some(waker.clone())` → `waker.clone()`、`port/win32/window.rs` の `None` → `waker.clone()`）。HEAD のコードは付録の「今」と一致。
- **確認のコマンドは `--features embed` を付ける**。`crates/arvane-native/src/port.rs` は `#[cfg(all(windows, feature = "embed"))] pub(crate) mod win32;` で、既定 feature は空なので、付録の `cargo clippy -p arvane-native --all-targets` は `port/win32/window.rs` を compile しない。確定するコマンド：
  - `cargo clippy -p arvane-native --all-targets --features embed -- -D warnings`
  - `cargo xtask check --no-write --only state-fields --crate arvane-native`

---

## P8-NA（付属書 D8 6 章 L2、付録 A-G6 W1（P8-W10）・W2（P8-W11）、付録 B の P8-NA の 16 行、W6 lane L2）

### 判定：修正あり（確認コマンドと、付録の条件付きの文の確定）

### 修正
- **付録 A-G6 W1 の「`static RETAINED` を lint が単独で数える場合に限り marker を置く」は本文 3.4 の P8-O6 で確定済み：置かない**。`Retained` の 2 field（`mapping`・`reported`）の marker は付録の文のとおり置く。`#[expect(clippy::disallowed_types, …)]` は HEAD の `MAPPING` にある既存のものを `RETAINED` に移すだけで、新しい `#[expect]` ではない（作業指示書 0.2 の 5・9 に当たらない）。
- 付録 A-G6 W2 の「追加を推奨する E2E」は**足さない**（本文 3.4 の P8-O5。足すのは 5.2 の unit test `surrogate_pairs_and_lone_surrogates_decode_in_one_push` だけ）。
- `port/x11/dnd.rs`・`port/x11/notifier.rs` の行は marker の文（コメント）だけの変更なので、Linux の build の確認は要らない（作業指示書 0.6 の Linux の規則は「コードを変える unit」に当たらない）。`state-fields --crate arvane-native` が Windows でも x11 のファイルを読む。
- **確認のコマンドは `--features embed` を付ける**（NA と同じ理由。`Utf16Decoder` は `port/win32/state.rs` にあり、feature 無しでは試験が 0 件で「成功」になる）：
  - `cargo clippy -p arvane-native --all-targets --features embed -- -D warnings`
  - `cargo test -p arvane-native --features embed --lib surrogate_pairs`（出力に `1 passed` があること）
  - `cargo xtask check --no-write --only state-fields --crate arvane-native`
- `native_input/state.rs` の 3 行（`NativeInput.text`・`reduced_motion`・`locale`）は NA の後で行がずれるが field は残る。`owner.field` で探す。

---

## AS（付属書 D5 付録 P5-D F-3、W6 lane L2）

### 判定：修正あり（付録の手順に残る選択肢の確定、可視性）

### 修正
- W2 の S5 は `store/asset_store.rs` の `register_font` だけを変え（D4 の D3 の表）、`VariantKey` の構築・読みの箇所には触れない。
- **`start_raster` の署名は変えない**（F-3 の「`start_raster(asset, svg, key, size, crop, now)` に変える」は行わない）。先頭の let-else を次にして、今と同じく SVG の key でない時は何もしない：

```rust
    fn start_raster(&mut self, asset: &Asset, svg: Arc<Svg>, key: VariantKey, now: Stamp) {
        let (Some(entry), VariantSource::Svg { size, crop }) =
            (self.entries.get_mut(asset), key.source)
        else {
            return;
        };
        entry
            .variants
            .insert(key, Variant::Pending { requested: now });
        let generation = entry.generation;
        self.submit(Job::Raster {
            asset: asset.clone(),
            generation,
            svg,
            size,
            crop,
        });
    }
```
- `VariantKey::is_whole` は `asset_store.rs`（別 module）の `fallback` から呼ぶので **`pub(crate) fn is_whole(&self) -> bool`** にする（F-3 のコード片の `fn is_whole` は private で compile しない）。`region` の戻り値の型は今の `crate::fit::Region` の path で書く。
- `use` の追加：`asset_store.rs` の `use super::entry::{…}` に `VariantSource` を足す。
- 他は F-3 のとおり（`asset_store.rs` の 3 つの literal、`fallback` の `(!whole || k.is_whole())`、`evict.rs` の `trim_tints` の `key.same_shape(&base)`）。HEAD で `VariantKey` の field を読む箇所はこれで全部（grep で確認）。marker 3 本は消す。
- **確認のコマンドを差し替える**：付属書の `cargo test -p arvane-asset store` は、`arvane-asset` に unit test が 1 本も無い（`#[test]` 0 件）ので何も確かめない。代わりに次を行う：
  - `cargo clippy -p arvane-asset --all-targets -- -D warnings`
  - `cargo xtask check --no-write --only state-fields --crate arvane-asset`
  - `dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- assets:: images_animated:: svg_embedded_text:: offscreen:: canvas:: borders_backgrounds:: image_semantics::`（静止画・動画の frame・SVG・tint の経路）

### 検証
- worktree で上の形（`VariantSource`、`start_raster` の let-else、`pub(crate) fn is_whole`）を実装し、`cargo clippy -p arvane-asset --all-targets -- -D warnings` が警告 0 で通ることと、上の E2E を確認した（結果は末尾）。

---

## EN-1（付属書 D5 付録 P5-A A5・A6・A7・A9・A8・A10、W6 lane L2）

### 判定：変更なし

### 修正
- なし。`measure/editor.rs`、`model/pointer/{state,machine,shell}.rs`、`text_input/{edit_lines,shaping,chunks}.rs` は W1 で変わっておらず、W2〜W5 の unit も所有しない。付録の「今」の行・コードは HEAD と一致した（`Track::new` の呼出しは `state.rs` の `PointerState::default` と `machine.rs` の `begin_pointer` の 2 箇所だけ、`PointerSource {` の literal は `shell.rs` の 1 箇所だけ、`source.primary` の読みは `machine.rs` の 3 箇所だけ）。
- `Track::contact` の `debug_assert_ne!(source.kind, PointerKind::Mouse, …)` は、`shell.rs` の `contact` の前段（`(input.kind == PointerKind::Mouse) != (input.id == PointerId::MOUSE)` を拒む分岐）があるので到達しない。そのまま書く。
- A5 の `Model::report_sessions` は W2 の AH1 が `model/edit/inputs.rs` を変えた後も同名で残る（AH1 は同ファイルの `Mutation::LOCAL_TEXT` の 1 行だけを変える）。

---

## EN-2（付属書 D5 付録 P5-D F-2・F-4・F-5（engine）・F-7、本文 3.5 の `Host.app_motion`、W6 lane L2）

### 判定：修正あり（F-7 の遷移の手順の欠落、W2・W3 の変更との共存）

### 修正
- **W2・W3 の変更との共存**：`host/events.rs` は W2 の AH1（`AccessibilityDeactivated` の扱い、`a11y.node` → `a11y.payload`）、`host/window.rs`・`host/events.rs`・`host/pump.rs` は W3 の S3（`LastPresent::Shown { at_ns }`、`FRAME_STATE_KEEP_NS`、`Host::release_at`、`control` の候補、`wake` の解放）が変えている。EN-2 はそれらの行に触れない。EN-2 が変えるのは `HostWindow.environment` の marker、`Host` の `next_wake`・`app_motion`、`HostPhase`、`begin`（events.rs）の `app_motion` の 1 文、`window_changes.rs` の 3 箇所、`launch.rs` の初期化だけ。
- **F-7（`HostPhase` に `next_wake`）は次のコードで確定する**（付録の「書き手 :432」「読み手 :793」に加えて、`Starting → Running` の遷移で値を引き継ぐ手順が付録に無いので足す。引き継がないと最初の turn が求めた timer wake が消える）：
  - `crates/kivact-engine/src/host/pump.rs` の `HostPhase` を次にする（doc は今の各 variant の doc を残す）。derive は今のまま（`Option<u64>` は `Copy`・`Eq`）。

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HostPhase {
    /// （今の Starting の doc）
    Starting {
        /// The client's next turn deadline (`TurnOutput::Continue`).
        next_wake: Option<u64>,
    },
    /// Handling the shell's events.
    Running {
        /// The client's next turn deadline (`TurnOutput::Continue`).
        next_wake: Option<u64>,
    },
    /// （今の Exiting の doc）
    Exiting,
    Closed,
}

impl HostPhase {
    /// The client's next turn deadline: kept only while the host runs turns.
    pub(super) fn next_wake(self) -> Option<u64> {
        match self {
            Self::Starting { next_wake } | Self::Running { next_wake } => next_wake,
            Self::Exiting | Self::Closed => None,
        }
    }

    /// Record the client's next turn deadline; nothing once it exits or closes.
    pub(super) fn set_next_wake(&mut self, at: Option<u64>) {
        if let Self::Starting { next_wake } | Self::Running { next_wake } = self {
            *next_wake = at;
        }
    }
}
```
  - `Host` の `next_wake` field とその marker（`independent: the client's next turn deadline, valid in every host phase`）を削除する。variant の中の `next_wake` には marker を付けない（variant の数える field は 1 個で、付けると unused marker）。
  - `Host::enter`：`if self.phase == HostPhase::Starting { self.phase = HostPhase::Running; }` を `if let HostPhase::Starting { next_wake } = self.phase { self.phase = HostPhase::Running { next_wake }; }` にする。
  - `Host::turn`（pump.rs）：`TurnOutput::Continue { next_wake_ns } => self.next_wake = next_wake_ns,` を `TurnOutput::Continue { next_wake_ns } => self.phase.set_next_wake(next_wake_ns),` にする。
  - `Host::control`（pump.rs）：候補配列の `self.next_wake,` を `self.phase.next_wake(),` にする（S3 が足した `release_at` の候補はそのまま残す）。
  - `pump.rs` の `if self.phase == HostPhase::Starting || self.phase == HostPhase::Closed {` を `if matches!(self.phase, HostPhase::Starting { .. } | HostPhase::Closed) {` にする。
  - `host/launch.rs`：`phase: HostPhase::Starting,` を `phase: HostPhase::Starting { next_wake: None },` にし、`next_wake: None,` の行を削除する。
  - `== HostPhase::Closed`・`== HostPhase::Exiting`・`!= HostPhase::Closed`・`!= HostPhase::Exiting` の比較は compile するのでそのままにする（変えるのは `Starting`・`Running` を値として書いた箇所だけ。HEAD では上の 3 箇所だけ）。`crates/kivact-engine/src/host.rs` の module doc の `HostPhase::Starting` は path のままで doc link ではないので変えない。
- F-2：D5 のとおり `record_environment` を `window_changes.rs` の `impl Host` に足し、`start_window`（`.environment.update(native.clone())` の文）と `environment_changed`（`host_window.environment.update(now.clone())` の文）を置き換える。marker は `// xtask-allow(state-fields): projection: written only by Host::record_environment`（D5 のまま）。`start_window` の今の `.expect("invariant: a started window has host state")` は `record_environment` の `if let` に替わるが、同じ関数の次の文（`surface` の書込み）に同じ expect が残るので不変条件の検査は残る。
- 本文 3.5 の `report_app_motion`：D5 のとおり（`events.rs` の `begin` の `self.app_motion.update((info.surface_active, info.env.frame_period_ns));` を `self.report_app_motion(info.surface_active, info.env.frame_period_ns);`、`window_changes.rs` の `if self.app_motion.update((drawable, period)).is_some() {` を `if self.report_app_motion(drawable, period) {`）。marker は本文 3.5 の文。
- F-4・F-5（engine）：D5 のとおり。`paint/state.rs`（W1 の F4）と `paint/list/walk.rs`（W1 の F1）は変わったが、`Step` の定義・`opened`・`walk.rs` の `Step {` の literal は付録の「今」と一致（行は `walk.rs` で 4 行下）。`contract/host.rs`（W1 の G2）の `DrawableExport.generation`・`SetDrawableGeometry.generation` の marker は HEAD に残っている。

- **確認のコマンドを差し替える**：付属書の `cargo test -p kivact-engine paint` は、kivact-engine の unit test が `model/events.rs` にしか無い（`paint` に当たる試験 0 件）ので何も確かめない。代わりに次を行う：
  - `cargo clippy -p kivact-engine --all-targets --features testing -- -D warnings`
  - `cargo xtask check --no-write --only state-fields --crate kivact-engine`
  - `dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- paint:: compositor:: environment:: lifecycle:: windows::`（F-4 の clip・rounded の描画、S3 の後の present、F-2 の環境の変化、F-7 の exit、複数 window の environment）

### 検証
- worktree で `HostPhase` の variant に `next_wake` を持たせ `Host.next_wake` とその marker を消した形で `cargo xtask check --no-write --only state-fields` を実行し、kivact-engine に error が出ないことを確認した（variant の中の 1 field は数えても組合せにならない）。
- `pump.rs` は HEAD で 799 code 行。S3 と EN-2 の追加（各 20 行未満）で 1000 を超えない。`control` は 55 行で、S3 の 1 候補と上の置換で 80 を超えない。

---

## P8-EN（付属書 D8 6 章 L2、付録 A-G3 W1（P8-W03）・W2（P8-W04）、付録 A-G6 W5（P8-W14）、7 章 #2・#3、付録 B の P8-EN の 43 行、W6 lane L2）

### 判定：修正あり（W2 の AH1 の改名）

### 修正
- **`Mutation.local_text` の行（付録 B）は、W2 の AH1 が改名した `Mutation.payload_only` に置く**（D2 3.5 `mutation.rs`：field `local_text` を `payload_only` に改名し doc を書き換え、marker 行は文言そのまま残す）。P8-EN は `payload_only` の直前の marker 行を、付録 B の `Mutation.local_text` の行の「新しい marker 行」（他の 9 field と同じ文）で置き換える。AH1 が書いた doc は変えない。
- **P8-W03（`sync.rs` の 1 行の削除）**：対象は `Model::sync` の `if self.changes.fonts { … }` の塊の最後の `self.changes.paint = true;`（HEAD の 148 行。W2 の AH1 は同じ塊の `self.a11y.node(id);` を `self.a11y.payload(id);` に、W3 の T7 は同ファイルの image の報告を変えるが、この 1 行は変えない）。同ファイルの他の `self.changes.paint = true;`（`invalidate_paint`、`derive`、`report_images` の中）は消さない。
- W2〜W5 の変更と付録 B の行の対応（全て「field は残る。`owner.field` で探す」）：
  - `contract/node.rs` の `EditingData.value`：W3 の M3・T9 は `NodeProperties` の 5 field を `Rc` にするだけで `EditingData` を変えない。
  - `tree/node.rs` の `Place.transformed` と 7 章 #3 の `update_place` の doc：W3 の T7 は `HostNode` の `sent_*` を `sent: Option<Box<SentReports>>` に替えるだけ。T7 の `sent` は `HostNode` の唯一の数える field なので marker は要らない（P8-EN は足さない）。
  - `tree/node_tree.rs` の `NodeTree.pending_child_order`・`root`：W1 の F3 で 9 行下にずれた。W3 の T7 は `set_observed` の 1 文だけを変える。
  - `model/aggregate.rs` の `Model.window_request`：W1 の F1 で 1 行下にずれた。W2 の AH1・W3 の M3（`recent` field の追加）は変えない。
  - `lower/assets.rs` の `ArvaneAssets.pending`：W2 の S5 は `register_font` の呼出しだけを変える。
  - `host/window.rs` の `HostWindow.parent`：W3 の S3 は `LastPresent` に variant を足すだけ。
  - `model/pointer/state.rs` の `PenTrack.over`・`PointerSource.pressure`・`tilt`・`Pointer.file_target`：EN-1 の後も field は残る（`PointerSource.primary` は EN-1 が消すが付録 B に行は無い）。
- P8-W14（`ShellImeText.password`）：HEAD の書き手は付録のとおり `text_geometry.rs` 2 箇所・`session.rs` 2 箇所だけ、読み手は 0（`arvane-native`・`arvane-virtual`・`kivact-test`・`dev` を grep で確認）。

---

## KT（付属書 D5 付録 P5-A の既存欠陥 D1・D2、付録 P5-F の F-U5、W6 lane L2）

### 判定：変更なし

### 修正
- なし。`crates/kivact-test/src/screen/{model,input,disposal_tests,host_sync_tests,real_window_tests}.rs` と `pointer.rs` は W1〜W5 で変わらない（W1 の G2 と W5 の G8 は `kivact-test` の別のファイルを変える）。`Contact { .. }` の literal は `input.rs` の `contact_down` の 1 箇所だけ、`send_contact` の `is_primary: true` は 1 箇所。
- 既存の試験への影響：2 本以上の contact を同時に下ろす既存の試験は無い（`contact_down` を使う 10 ファイルを全部読んだ。`input.rs` の 2 本の例は ```` ```ignore ```` の doc）。期待値の変更は無い。
- 新しい E2E `a_second_touch_is_not_primary_and_leaves_hover_and_activation_to_the_first` が使う `Screen::hovered`（engine の `Model::hovered` は最初の空でない hover chain、mouse は未配置なので空）、`is_pressed`（press chain と hover chain の両方）、`render`（`FnOnce`）は HEAD の実装で付録の期待どおりに働く。

---

## P8-KT（付属書 D8 6 章 L2、付録 A-G7 W2（P8-W19）、付録 B の P8-KT の 6 行、W6 lane L2）

### 判定：修正あり（確認コマンドだけ。コードの手順は変更なし）

### 修正
- コードの手順はなし。`editing_drop_probe.rs`・`host_sync_tests.rs`・`facts.rs` は W1〜W5 で変わらない。付録 A-G7 W2 の「今」は HEAD と一致（`*reader.borrow_mut() = …` と `probe.armed.set(true)` は試験の末尾近く）。
- **確認のコマンドは window-lock を通す**：kivact-test の `render` は既定（`KIVACT_TEST_MIRROR` が未設定 = `auto`）で実 window の mirror を開くので、作業指示書 0.6 の規則により `dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-test --lib replacing_editing_drops_line_state_after_releasing_the_model` で実行する（出力に `1 passed`）。

---

## RT（付属書 D5 付録 P5-B B6〜B9・既存欠陥 D2、付録 P5-D F-5（runtime）・F-6・F-9・A-RT の Queue の doc、W6 lane L3）

### 判定：修正あり（marker の文の行番号、W1 の変更との突合せ）

### 修正
- `crates/kivact-runtime/src/reactive/data.rs` は W1 の M11・T8 が変えた。RT が変える field（`Runtime.seq`・`write_epoch`・`ext`、`MemoCell.eq`、`Frame.owner`・`pure`・`writer`・`tracking`）は全て HEAD に残り、型も同じ。`OwnerEntry`（T8 の `extra`。W1 の error）は RT では変えない（R1・R2 の担当）。
- **marker の文に行番号を書かない**（D8 3.3 の規則 3。D5 F-9 と F-5 の `write_epoch` の文は `frame.rs:79-104`・`graph.rs:560-621` を含み、W2 の S7 が `graph.rs` を変えるので既に食い違う）。次の文を一字一句使う：
  - `Frame.owner`：`// xtask-allow(state-fields): independent: callback_frame and handler_frame replace the owner under any frame kind`
  - `Frame.pure`：`// xtask-allow(state-fields): independent: pure_section sets it under any frame kind and owner`
  - `Frame.writer`：`// xtask-allow(state-fields): independent: inherited from the enclosing frame except in a non-user effect's own frame (tracking_frame)`
  - `Frame.tracking`：`// xtask-allow(state-fields): independent: untracked_frame and callback_frame clear it under any kind`
  - `Runtime.write_epoch`：`// xtask-allow(state-fields): identity: the write generation stamped on each notified source (SourceNode::changed_at) and compared by effects after their own writes`
  - 他の F-5（`MemoCell.eq`、`Runtime.seq`）、F-6（`Runtime.ext`）、B6・B7（`DetailWrite.cause`・`DetailRun.cause`）、B8（`Reported.seq`）、B9（`next_handler_id`）は D5 の文のまま（行番号を含まない）。
  - 名指した関数は HEAD に実在（`reactive/frame.rs` の `pure_section`・`untracked_frame`・`callback_frame`・`handler_frame`・`tracking_frame`）。
- B9 の改名と既存欠陥 D2（`ActionCell::begin_run`）は D5 のとおり。HEAD の `action.rs` の `last` の書き手は `start_latest` の closure と queue worker の loop の 2 箇所（D5 の「今」と一致）。`begin_run` の戻り値の型は `RunFn` の戻り値 `Pin<Box<dyn Future<Output = Result<T, UiError>>>>` と同じ。
- Queue の doc（A-RT の表）：`Queue.next_worker_id` の doc `/// Id of the last worker spawned: a worker whose id is not current returns.` を `/// Allocates worker ids; the current worker's id lives in \`Worker::Picking\`/\`Running\`.` にする（marker は置かない）。

---

## P8-RT（付属書 D8 6 章 L3、付録 B の P8-RT の 9 行、W6 lane L3）

### 判定：変更なし（行番号のずれだけ）

### 修正
- `data.rs` の 4 行（`Runtime.clock`、`SourceNode.changed_at`・`written`、`OwnerEntry.parent`）は W1 の M11・T8 で 1〜12 行ずれたが、field と marker の文は残る。新しい marker の文が名指す関数（`advance_clock`、`notify_subscribers`、`recheck_after_own_writes`、`create_owner`、`detach_owner`、`Runtime::pending`）は HEAD に実在する。
- `OwnerEntry.extra`（W1 の error）は R1・R2 の担当で、P8-RT は触れない。P8-RT は `OwnerEntry.parent` の marker 行だけを置き換える。

---

## CO（付属書 D5 付録 P5-A A3、付録 P5-D F-5（core）・F-8・A-CO の initial_render・本文 3.5 の SharedImage、W6 lane L3）

### 判定：修正あり（W3 の T6 が `CoreNode` を組み替えた後の marker）

### 修正
- **`view/tree.rs` の F-5（core）は W3 の T6（D7 3.5 M3-5）の後の形に書く**。T6 は `CoreNode` の `observation`・`presence`（と `references`・`exit_complete`・`semantic_origins`・`style_layers`）を新しい `CoreRare` に移し、`CoreNode.rare: Option<Box<CoreRare>>` を足す。D5 F-5 の 4 本（`CoreNode.at`・`handlers`・`observation`・`presence`）の代わりに、次の 5 本を一字一句置く（T6 が書いた `rare`・`observation`・`presence` の marker と、`at`・`handlers` の今の marker を置き換える）：
  - `CoreNode.at`：`// xtask-allow(state-fields): option: the builder's call site, recorded when the node is created; absent for the root, fragments and the wrappers a builder adds, which take their nearest located ancestor's location`
  - `CoreNode.handlers`：`// xtask-allow(state-fields): independent: the declared handlers recorded when the node is created, or an empty record ViewNodes::handlers_mut and ViewNodes::observe_node create for a later observer; a node with or without a location may have either`
  - `CoreNode.rare`：`// xtask-allow(state-fields): independent: created by the first rare resource the node gets (a reference, an exit callback, an observation, a presence source, semantic origins or DevTools style layers) and kept until removal; a node with or without a location or handlers may have one`
  - `CoreRare.observation`：`// xtask-allow(state-fields): independent: created by ViewNodes::observe_node when a reference first needs the node's geometry, text truncation, scroll or image status; the presence source beside it is created separately by ViewNodes::presence, so either exists without the other`
  - `CoreRare.presence`：`// xtask-allow(state-fields): independent: created by ViewNodes::presence when a reference first observes the node's lifetime; the observation beside it is created separately, so either exists without the other`
  - 理由（D5 から変えた点）：`handlers`・`observation`・`presence` は builder の入力ではなく、mount の後に `ViewNodes::handlers_mut`・`observe_node`・`presence` が作る（HEAD の `tree.rs` の `observe_node` は `handlers` も `get_or_insert_with` で作る）。D8 3.3 の「option は caller・builder が与え内部の event で書き換わらない値」に合わないので `independent:` にした。`at` は mount の時に builder の call site で決まる（`create.rs`・`builders.rs`）ので `option:` のまま。
- **F-8（`NodeObservation`）**：`text_truncated` は D5 の文のまま。`measured` は `option:` でなく次の文にする（`client/dispatch/env.rs` の `layout_changed` が報告ごとに書き、報告に measured layout が無ければ `None` を書くので、builder の値ではない）：
  `// xtask-allow(state-fields): independent: set from each layout report, None while the report carries no measured layout box; text_truncated is set from the same report on its own, so every pair occurs`
- A3（`devtools_state.rs` の `PropWrite.changed`）、A-CO の `ViewState.initial_render`（`view/state.rs`）、本文 3.5 の `SharedImage::replace_current`（`gpu_interop/share/app_side.rs`）は D5 のとおり。W2 の G3 は `app_side.rs` の `shared_image` の中の需要の行（`demand_gpu`）だけを消し、`renew`・`import`・`current` を変えない。W2 の M8 は `view/tree.rs` の `CoreNode` の型を変えない（作業指示書 4 章のとおり M2-8 の手順 6 は行わない）。

---

## P8-CO（付属書 D8 6 章 L3、付録 A-G4 W1（P8-W05）・W2（P8-W06）・W3（P8-W07）、付録 B の P8-CO の 28 行、W6 lane L3）

### 判定：修正あり（W1 の M7 の後の読み手、M6 の有無）

### 修正
- **P8-W07 の `mount/semantics.rs` の読み手は W1 の M7 の後の形で直す**。HEAD では `bind` の中の `let text_at = patches.iter().rev().find(|p| p.text).map_or(at, |p| p.at);`（付録の `:66 .find(|p| p.text)` は M7 の前の形）。これを
  `let text_at = patches.iter().rev().find(|p| matches!(p.input, SemanticInput::Text)).map_or(at, |p| p.at);`
  にする（`rev()` を残す。`SemanticInput` は同ファイルで import 済み）。同ファイルの `Plan::new` の `SemanticInput` の 2 つの `match` は `_` の腕を持つので変更不要。W1 の error（`Plan`）を R1・R2 の修正が変えた後に `_` の腕の無い `match` が `SemanticInput` にあれば、新しい 3 variant を「その他」の腕に入れる。`p.reactive` の読み（2 箇所）は変えない。
- P8-W07 の他の読み手：`builders.rs`（W1 の M1 が変えた）の `labelled_by_sources`・`described_by_sources` の `match patch.relation {` は HEAD に残っている（付録のとおり置き換える）。`container.rs` の `.find_map(|patch| patch.orientation.clone())`、`resolve.rs` の `semantics.iter().any(|p| p.text)` も残っている。
- `SemanticsPatch.reactive` の `projection:` の名指す `Common::semantic` は HEAD に実在し、`SemanticsPatch {` の literal は `modifiers.rs` の 1 箇所だけ、`.reactive` への代入は kivact-core に無い（X4 の package 範囲の検査に当たらない）。
- P8-W06（`KeyRoute`）：W1 の M1 で `overlays.rs` が 1 行ずれただけで、`KeyRoute` と `key_route` は付録の「今」と一致。
- P8-W05（`Selected`）：W2 の M9 は `part_input.rs` の `route_accessibility` だけを変え、`Selected`・`select`・`deliver_selected` を変えない。
- **M6（D3 M2-6、W1 で差戻し）との関係**：P8-CO の開始時に `grep -n "enum LocalPaint" crates/kivact-core/src/view/mount/facts.rs` を実行する。
  - 1 行が出た（M6 が W6 より前に merge された）場合：付録 B の `NodeFacts.local_paint`（`mount/facts.rs`）と `NodeStyle.local_paint`（`mount/style.rs`）の 2 行は、marker 行を**削除**する（field は enum `LocalPaint` になり数えない。残すと unused marker）。`DrawInput.local_paint`（`mount/draw.rs`、型は `Option<LocalPaintSource>` のまま）、`Resolved.draw`・`attached`（`mount/resolve.rs`）は付録 B のとおり。
  - 出ない場合：付録 B のとおり全行を置き換える。
- W3 の T6 は `mount/create.rs`（`exit_complete` の 1 文）と `mount/semantics.rs`（`semantic_origins` の 1 文）を変えるが、付録 B の `FirstWrite.observe_layout` の行と P8-W07 の読み手には触れない。

---

## P8-ARV（付属書 D8 6 章 L3、付録 A-G6 W3（P8-W12）・W4（P8-W13）・W8（P8-W17）、付録 B の P8-ARV の 14 行、W6 lane L3）

### 判定：変更なし（行番号のずれだけ）

### 修正
- なし。`arvane-render/src/draw/painter.rs` は W2 の S4 が `FontCache` と `self.fonts.get(&run.font, run.font_index)` の 2 箇所を変えるが、`GlyphRun { … hint, cached }` の 4 つの literal は変えない（`hint:`・`cached:` の行で探す）。`arvane-shell/src/window.rs` は W1 の F6 が `ShellWindow::present_with` の doc に 2 行足しただけで、`StepOutcome::Idle` の 3 行は 2 行下にずれた。`a11y/shared.rs`・`kivact-shader/src/entry.rs`・`assemble.rs`・`gpu/encode.rs`・`arvane-compose` の 2 ファイルは W1〜W5 で変わらない。`RunSelection` は名前が `Run` で終わらないので phase と数えない。

---

## P8-GPU（付属書 D8 6 章 P6 の lane、付録 A-G6 W6（P8-W15）・W7（P8-W16）、付録 B の P8-GPU の 17 行、作業指示書 9 章の追加の指示、W6 lane L3）

### 判定：修正あり（W1 の G1、W5 の G8 の後のコード）

### 修正
- **W1 の G1 の後の `crates/arvane/src/window.rs`**：G1 が `Windows.gpu`・`Driver.gpu` を消した（付録 B に行は無い）。残る付録 B の 6 行（`Windows.observer`・`Shell.prefer_gpu`・`Shell.observer`・`ShellSetup.prefer_gpu`・`ShellSetup.observer`・`Driver.observer`）は全て残っている。**`Shell.prefer_gpu`・`ShellSetup.prefer_gpu` の新しい文は G1 の後の振る舞い（`arvane/src/env.rs` の `RendererStart::resolve`：`ARVANE_RENDERER` が `gpu` でも `auto` でも `prefer_gpu = false` を上書きする）に合わせて次にする**（付録 B の「`ARVANE_RENDERER=gpu` overrides it」は `auto` を落としている）：
  - `Shell.prefer_gpu`：`// xtask-allow(state-fields): option: whether the application asked to draw on the GPU (\`Shell::prefer_gpu\`, default true); false draws on the CPU unless \`ARVANE_RENDERER\` is \`gpu\` or \`auto\``
  - `ShellSetup.prefer_gpu`：`// xtask-allow(state-fields): option: whether the host asks to draw on the GPU (default true); false draws on the CPU unless \`ARVANE_RENDERER\` is \`gpu\` or \`auto\``
  - 他の 4 行は付録 B の文のまま。
- `crates/arvane/src/presentation.rs` の `RendererChange::Gpu.configure`・`interop`：G1 が `GpuStarting { cause }` を足して 21 行下にずれた。field は残るので付録 B の文を置く。`GpuStarting` の `cause` は数える field ではない。
- **W5 の G8 の後の再公開**：G8（DEC-17）が `kivact-core/src/gpu_interop.rs` から `AdapterMatch`・`GpuBackend`・`GpuPower`・`GpuPreference`・`GpuStart` の再公開を消し、`kivact::launch` に移している。付録 A-G6 W6 の「`pub use kivact_types::AdapterMatch;` の隣に `AdapterIds`」は行えないので、**`AdapterIds` は `crates/kivact-core/src/gpu_interop.rs` の `pub use kivact_types::GpuAdapter;` の行の次に `pub use kivact_types::AdapterIds;` として足す**（doc は `/// How the graphics API identifies an adapter (\`GpuAdapter::ids\`).`）。`kivact::launch` には足さない（`AdapterIds` は `GpuAdapter` の一部で、`GpuAdapter` は `gpu_interop` に残る）。`kivact-types/src/lib.rs` の `pub use gpu::{…}` には付録のとおり足す。
- E2E と docs：G8 が `dev/tests/kivact-public-api/tests/e2e/gpu_interop.rs`・`drawable_export_gpu.rs` と `gpu-texture-sharing.md` の import の path を `kivact::launch::…` に変えた後に、P8-GPU が `adapter.backend` → `adapter.backend()` を行う（付録の行番号はずれているので `adapter.backend` の grep で探す。HEAD では `gpu_interop.rs` 2 箇所、`drawable_export_gpu.rs` 1 箇所、`arvane/tests/e2e/external.rs` 2 箇所、`gpu-interop-producer` の `producer.rs`・`wgpu_engine.rs` 各 1 箇所、`kivact-gpu-share` の `wgpu/device.rs` 2 箇所・`wgpu/pool.rs` 1 箇所。`d3d12/adapter.rs`・`vulkan/physical_device.rs` の `adapter.backend` は付録の書換えで関数ごと置き換わる）。
- W2 の G3 は `gpu_interop.rs`（`mod demand` の削除）・`import/image.rs`（需要の行の削除）・`gpu-texture-sharing.md` を変えるが、`ExternalImage.latest` の field と docs の `GpuAdapter` の段落（en の「A `GpuAdapter` names the graphics API, the adapter's LUID or」の文）は変えない。
- `GpuAdapter` の field（`luid`・`device_uuid`・`driver_uuid`・`backend`）を読む箇所は、HEAD では付録 A-G6 W6 の所有ファイルの中だけ（kivact-engine・kivact-test・arvane の本体・kivact-core は `GpuAdapter` を値として保持・比較するだけ、`generation` は残る）。W1〜W5 の設計にも新しい読みは無い（D6 を grep で確認）。
- 確認のコマンドは付録 D8 6 章の P8-GPU の行のとおり。

---

## 付属書 B の行のうち W1〜W5 で行がずれた・field が変わったもの（L1〜L3 の unit の分）

| 付録 B の行（73c36c7c） | 何が起きたか | 扱い（確定） |
|---|---|---|
| `kivact-engine/src/mutation.rs:123` `Mutation.local_text` | W2 の AH1 が `payload_only` に改名 | P8-EN が `Mutation.payload_only` の marker 行に、その行の新しい文を置く |
| `kivact-engine/src/model/aggregate.rs:54` `Model.window_request` | W1 の F1 で 1 行ずれ | `owner.field` で探す |
| `kivact-engine/src/tree/node_tree.rs:15,19` `NodeTree.pending_child_order`・`root` | W1 の F3 で 9 行ずれ | 同上 |
| `kivact-core/src/view/overlays.rs:517,520` `KeyRoute.stop`・`activates` | W1 の M1 で 1 行ずれ | P8-W06 で marker ごと消える |
| `arvane-shell/src/window.rs:272,274,277` `StepOutcome::Idle.*` | W1 の F6 で 2 行ずれ | `owner.field` で探す |
| `arvane/src/presentation.rs:38,42` `RendererChange::Gpu.configure`・`interop` | W1 の G1 で 21 行ずれ | 同上 |
| `arvane/src/window.rs:196,285,288,345,348,412` | W1 の G1 で `gpu` field が消え、行がずれた | `owner.field` で探す。`prefer_gpu` の 2 行は P8-GPU の節の文 |
| `kivact-layout/src/columns.rs:60,63` `Counted.*` | W1 の F2 で 1 行ずれ（W4 の T5 は `Counted` を変えない） | P8-W18 で marker ごと消える |
| `kivact-runtime/src/reactive/data.rs:87,208,211,320` | W1 の M11・T8 で 1〜12 行ずれ | `owner.field` で探す |
| `dev/bench/src/bin/kv_heap.rs:35,37,39,41` | W1 の M12 で 1 行ずれ（static は変わらない） | P8-W20 のコード片で置き換える |
| `kivact-core/src/view/mount/facts.rs:17`、`mount/style.rs:87` `local_paint` | M6 が W6 より前に入った場合だけ enum になる | P8-CO の節の grep で判定（入っていれば marker 削除） |

L1〜L3 の付録 B の行で、W1〜W5 で field が消えるものは無い（上の改名と条件付きの 2 行を除く）。

---

## 全体

### wave・lane の組み直し
- L1〜L3 の unit の並びと所有ファイルは変えない。L1〜L3 と L4〜L6 の間でファイルの重なりは無い（D8 6 章の確認に、本監査で足したファイル（`kivact-core/src/gpu_interop.rs` の `AdapterIds` の 1 行は P8-GPU の既存の所有）を加えても同じ）。
- crate をまたぐ compile の結合：
  - P8-EN の `ShellImeText.password` の削除（arvane-shell の公開 field）の書き手は全て P8-EN の所有（kivact-engine）、読み手は 0。
  - P8-GPU の `GpuAdapter` の変更の読み手は全て P8-GPU の所有。L2（kivact-engine）・L4〜L6 の crate は field を読まない。
  - KT の `Contact.primary` は `pub(super)`。他は全て crate 内（`pub(crate)` 以下）。
  - したがって W6 の L1〜L3 の後に compile が失敗してよい crate は無い。

### 他の担当（R1・R2 ほか）への申し送り（ファイルの重なりを避けるため）
- W1 の error のうち `Plan`（`kivact-core/src/view/mount/semantics.rs`）と `OwnerEntry`（`kivact-runtime/src/reactive/data.rs`）のファイルは、W6 では L3 の P8-CO と RT・P8-RT が所有する。R1・R2 の修正を W6 に置く場合は L3 の該当 unit（`Plan` は P8-CO、`OwnerEntry` は RT）の手順の最後に足す。W6 の他の lane に置かない。W5 以前に置く場合は本書の手順に影響しない（P8-CO の `SemanticInput` の `match` の注意だけ）。
- M6（D3 M2-6）を復活させる場合は W6 より前に置く（`mount/{resolve,style,draw,facts}.rs` は W6 で P8-CO が所有する）。W6 の後に置くと、P8-CO の 2 行の marker の扱いが逆になる。

### ユーザー（人間）にしか決められない項目
- なし。

---

## 検証結果

worktree `W:/w/kv-audit-R5`（HEAD 2a3b06b8 に detach）、`CARGO_TARGET_DIR=W:/kv-target/E`。

| 何を | 結果 |
|---|---|
| LY の追加 E2E `grid_items_justify_self_within_their_area` を HEAD のコードで実行（`window-lock.sh -- cargo test -p kivact-public-api-tests --test e2e -- grid::`） | **成功**（`grid::` 11 本すべて ok）。D5 の期待値は HEAD で成り立つ |
| XT-1 の lint の修正（X1・X2）と 2 本の試験を入れて `cargo test -p xtask state_fields` | **成功**（27 passed。新しい 2 本 `an_arc_callback_option_is_a_setting`・`enum_variants_named_none_are_not_option_locals` を含む） |
| XT-1（pop の marker 削除込み）＋P8-XT の kv_heap（`HEAP`・`TRACKING`・`INSIDE` の 3 本の marker）＋EN-2 の `HostPhase { next_wake }`（`Host.next_wake` と marker の削除）＋NA の `waker: ShellWaker`＋P8-NA の `Retained`・`RETAINED`（marker 無し）と `Utf16Decoder`（`pending` だけ、marker 無し）を入れて、`cargo xtask check --no-write --only state-fields`（全 crate） | **error は W1 の既知の 3 件だけ**（`Indexed`・`Plan`・`OwnerEntry`）。X1・X2 は他の crate に unused marker を作らず、P8-O6（`RETAINED` に marker を置かない）も正しい |
| AS の実装（`VariantSource`、`start_raster` の let-else、`pub(crate) fn is_whole`）で `cargo clippy -p arvane-asset --all-targets -- -D warnings` | **成功**（警告 0） |
| 同じ AS の実装で image・SVG の E2E（AS の節のコマンド） | **成功**（28 passed、0 failed） |
| 試験の有無 | `arvane-asset`・`kivact-layout`・`kivact-types`・`kivact-shader` は `#[test]` 0 件、`kivact-engine` は `model/events.rs` だけ。付属書の `cargo test -p arvane-asset store`・`cargo test -p kivact-engine paint`・`cargo test -p kivact-layout` は何も確かめないので、各節で E2E に差し替えた |
