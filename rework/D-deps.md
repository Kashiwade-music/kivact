# 付属書 D：依存の更新（設計書 12.2 節、DEC-18）

## 1. 範囲と前提

### 1.1 範囲

設計書 12.2 節の「いま上げる群」を unit にし、「待つ群」の再開の条件と、その確かめ方を書く（6 章。unit は作らない）。DEC-18 のとおり依存は fork しない。

| 群 | crate | 今（`Cargo.lock`） | 決定 | unit |
|---|---|---|---|---|
| いま | taffy | 0.14.0 | 0.15.1 に上げる | D01 |
| いま | accesskit・accesskit_winit・accesskit_windows・accesskit_unix | 0.24.1・0.33.1・0.33.1・0.22.1 | 0.25.2・0.34.2・0.35.2・0.24.1 に上げる | D02 |
| いま | rfd・pollster | 0.17.2・0.4.0 | 0.18.0・1.0.2 に**一緒に**上げる（rfd 0.17.2 は pollster ^0.4、rfd 0.18.0 は pollster ^1.0 を使う。片方だけだと pollster が二重になる） | D03 |
| いま | arboard・svgtypes・flate2 | 3.6.1・0.16.1・1.1.10 | 3.7.0・0.16.2・1.1.11 に上げる | D04 |
| いま | zbus | 5.19.0 | **上げない**（6.3 節）。最新の accesskit_unix 0.24.1 が `zbus ^5.19`、atspi 0.29 が zbus 5 を要るため、6 にすると zbus・zvariant・zbus_names・zbus_macros が二重になる（K6 の後退） | なし |
| いま | xim | 0.5.0 | **上げない**（6.3 節）。xim 0.6.0 は `x11rb ^0.14` を要る。winit 0.30.13（最新の安定版）・softbuffer 0.4.8・arboard 3.7.0・arvane-native 自身が x11rb 0.13 なので、x11rb と x11rb-protocol が二重になる | なし |
| 待つ | wgpu・naga、vello_cpu・vello_common・vello_hybrid・glifo、parley・fontique、skrifa・read-fonts、harfrust、usvg・resvg | 29.0.4、0.2.0×3・0.3.0、0.11.1、0.44.0・0.41.0、0.12.0、0.46.0 | 上げない。CP2・CP3 で条件を確かめて報告する | なし（6 章） |

各 crate の変更点は 2026-11-18 に CHANGELOG・release notes と crates.io の依存の表で確かめた。Kivact のコードに効く変更は次だけである（それ以外は追加か内部の修正）。

- **taffy 0.15**：`Position` の既定が `Static` になった。absolute・fixed の box は「containing block」へ持ち上げられ、自前の木（Kivact の `LayoutPass`）は layout の closure の中で `compute_oof_layout` を呼び、`LayoutContainingBlock` を実装しなければならない。`compute_root_layout` は root を margin と inset で動かし、absolute の root を shrink-to-fit にする。`RoundTree` に 3 つの method が増えた。`align_items`・`justify_items`・`align_content`・`justify_content` は `Option` でなくなった（`None` は `NORMAL`）。`LayoutOutput` と `Cache` は `Copy` でなくなった。`CompactLength` の tag の値が変わり、calc の handle は下位 3 bit に `0b100` の tag を持つ（`tag() == CALC_TAG` では calc を見分けられない。`is_plain_calc()` を使う）。定義済みの available space は border box のもの（親が子の margin を引く）。性能：Grid 約 60%、Flexbox 約 15〜20% 速い。
- **accesskit 0.25**：`accesskit::Tree` が `TreeInfo` に改名され、`Tree` は `#[deprecated]` の別名になった（Kivact は `accesskit::Tree::new` を 1 か所で使う）。adapter 3 つの公開 API の署名は変わらない。
- **rfd 0.18**：追加（`set_show_hidden_files`、`set_format_label`）と修正だけ。MSRV 1.92（toolchain は 1.98.1）。
- **pollster 1.0**：`FutureExt` が `IntoFuture` に実装されるようになっただけ。`block_on` の署名は同じ。1.0.2 は再入の deadlock の修正。
- **arboard 3.7、svgtypes 0.16.2、flate2 1.1.11**：追加と修正だけ。

### 1.2 前提（先の unit がしていること）

- W1b の T08（kivact-layout の `state-fields` の印の変換）、T15（E2E の binary の分割：`e2e_core`・`e2e_ui`・`e2e_native`）、W1c の T22 が終わっている。
- W2a の lane A（S02〜S06）が arvane-native の `use` と `Cargo.toml`（`arvane-protocol`）を直し終えている。D02〜D04 はその後のファイルを編集する（場所は文面で探す）。
- cargo は crates.io から新しい版を取りに行く（network が要る）。

### 1.3 共通の手順（全 unit）

コマンドはすべて worktree の root で実行する（PLAN-04）。root にいないときは `ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1`。

**DUP-BEFORE**（各 unit の手順 0。K6：二重の版を増やさないことの確認の基準を取る。ファイルは `.git` の中に置くので commit されない）：

```bash
awk '/^name = /{n=$3} /^version = /{print n}' Cargo.lock | LC_ALL=C sort | uniq -c | awk '$1>1{print $2" "$1}' > "$(git rev-parse --git-dir)/d-dup-before.txt"
```

**DUP-AFTER**（各 unit の確認の最後）：

```bash
awk '/^name = /{n=$3} /^version = /{print n}' Cargo.lock | LC_ALL=C sort | uniq -c | awk '$1>1{print $2" "$1}' > "$(git rev-parse --git-dir)/d-dup-after.txt"
LC_ALL=C comm -13 "$(git rev-parse --git-dir)/d-dup-before.txt" "$(git rev-parse --git-dir)/d-dup-after.txt"
LC_ALL=C comm -23 "$(git rev-parse --git-dir)/d-dup-before.txt" "$(git rev-parse --git-dir)/d-dup-after.txt"
```

- 1 つ目の `comm` は**何も出してはならない**（新しく二重になった crate も、版の数が増えた crate も無い）。何か出たら、その crate を `cargo tree -i <crate>@<版> --target all -e normal,build` で誰が引いているかを調べ、`STOP: <unit> Cargo.lock:<crate> 版が二重になった / <cargo tree -i の出力の要約>` で止める。
- 2 つ目の `comm` は二重が消えた crate を出す。unit ごとの期待は各 unit の完了条件に書く。
- 二重の一覧を目で見るときは `cargo tree -d --target all -e normal,build --depth 0` を使う（判定は上の `comm` で行う）。

**E2E の約束**（PLAN-01・PLAN-07）：E2E は一つの module を一つのコマンドで、表の binary に対して実行する。module と binary の対応は付属書 T の T15 の手順 7 のとおり（本書が使うものは各 unit に書いた）。window-lock は一つのコマンドに一度だけ。**filter が一つの試験も実行しなかったコマンド（`running 0 tests`）は失敗**とし、別の binary で探し直さない（STOP）。

```bash
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- <module>::
```

**期待値の扱い**：上げた版の修正で E2E・単体試験の期待値（座標、大きさ、画素）が変わって失敗したら、期待値を変えずに `STOP: <unit> <試験の file>:<試験の名前> <期待値と実際の値> / 依存の版を上げた直後に失敗` で止める（作業指示書 0.4）。

## 2. unit の一覧

| ID | wave | lane | 前提の unit | 所有するファイル・directory | 所見 | 規模 |
|---|---|---|---|---|---|---|
| D01 | W2b | D1 | T08・T15（W1b）、T22（W1c） | root `Cargo.toml` の `[workspace.dependencies]` の `taffy =` の 1 行、`Cargo.lock`、`crates/kivact-layout/src/tree.rs`、新 `crates/kivact-layout/src/tree/root.rs`、`crates/kivact-layout/src/style.rs`、`crates/kivact-layout/src/style/node.rs`、`crates/kivact-layout/src/style/convert.rs`、`crates/kivact-layout/src/columns.rs`、`crates/kivact-layout/src/pass_state.rs`、`crates/kivact-layout/src/layout/tests.rs`（試験 2 つ） | 設計 12.2（いま：taffy） | M |
| D02 | W2b | D2 | W2a の S02〜S06 | root `Cargo.toml` の `accesskit =`・`accesskit_winit =` の 2 行、`crates/arvane-native/Cargo.toml` の `accesskit_unix =`・`accesskit_windows =` の 2 行、`crates/arvane-native/src/a11y/conversion.rs`（1 行）、`Cargo.lock` | 設計 12.2（いま：accesskit） | S |
| D03 | W2b | D2 | D02 | root `Cargo.toml` の `pollster =` の 1 行、`crates/arvane-native/Cargo.toml` の `rfd =` の 1 行、`crates/arvane-native/src/port/win32/services.rs`（comment 1 行）、`Cargo.lock` | 設計 12.2（いま：rfd、pollster） | S |
| D04 | W2b | D2 | D03 | `crates/arvane-native/Cargo.toml` の `arboard =` の 1 行、`crates/kivact-style/Cargo.toml` の `svgtypes =` の 1 行、`crates/arvane-asset/Cargo.toml` の `flate2 =` の 1 行、`Cargo.lock` | 設計 12.2（いま：arboard、svgtypes、flate2） | S |

lane D1 と D2 と C（C01・C02）は同じ wave（W2b）で並ぶ。root `Cargo.toml` は行の単位で分ける：C02 は `members` の 1 行、D01 は `taffy =`、D02 は `accesskit =`・`accesskit_winit =`、D03 は `pollster =`。この 4 か所は互いに離れているので git の merge は衝突しない。`Cargo.lock` の衝突は作業指示書 1.3 節の手順で作り直し、その後に 4.2 節の確認を行う。

## 3. unit の手順

## D01：taffy を 0.15.1 に上げる

- **wave / lane / 前提**：W2b / D1 / T08・T15（W1b）、T22（W1c）
- **所有**：root `Cargo.toml` の `[workspace.dependencies]` の `taffy =` の行だけ、`Cargo.lock`、`crates/kivact-layout/src/tree.rs`、新 `crates/kivact-layout/src/tree/root.rs`、`crates/kivact-layout/src/style.rs`、`crates/kivact-layout/src/style/node.rs`、`crates/kivact-layout/src/style/convert.rs`、`crates/kivact-layout/src/columns.rs`、`crates/kivact-layout/src/pass_state.rs`、`crates/kivact-layout/src/layout/tests.rs`
- **目的**：設計 12.2 の taffy 0.14 → 0.15（Grid・Flexbox の layout の高速化。K3・K4・K5 の手段）。taffy 0.15 の破壊的変更に Kivact の自前の木を合わせ、**Kivact の layout の結果と能力を変えない**：root は 0.14 と同じく原点に置き（`compute_root_layout` を使わない）、すべての node を positioned（`Relative` か `Absolute`）にして absolute の box の containing block を常に layout の親にし、丸めは今の親からの相対の丸めを保つ。grid container の absolute の子は 0.14 と同じく grid の線の area に置く：taffy 0.15 は grid の詳しい情報（`DetailedGridInfo`）を `set_detailed_grid_info` で渡し、out-of-flow の pass が `get_detailed_layout_info` で読み返す。Kivact はそれを **absolute の子を持つ grid container の分だけ** 疎な表に保存し、普通の grid は何も保存しない（親の決定）。あわせて、calc の長さを tag で探して見落としていた所（0.14 から calc の handle の下位の byte が 0 でないと外れていた）を `is_plain_calc()` で直す。
- **手順**：
  0. 1.3 節の DUP-BEFORE を実行する。次に、**版を上げる前に**（taffy 0.14 のまま）`crates/kivact-layout/src/layout/tests.rs` の試験 `rounding_matches_taffy_at_negative_half_offsets` の後に次の 2 つの試験を足し、`cargo test -p kivact-layout` を実行して 2 つとも通ることを確かめる（taffy 0.14 での基準。TV-09 の不変条件の試験）。通らなければ `STOP: D01 crates/kivact-layout/src/layout/tests.rs:<試験の名前> taffy 0.14 で通らない / <assert の出力>`。この 2 つの期待値は、この後どの手順でも変えない。
     ```rust
     #[test]
     fn an_absolute_child_is_placed_from_its_parent_and_a_root_stays_at_the_origin() {
         let zero = kivact_types::Edges {
             top: Dimension::Px(0.0),
             right: Dimension::Px(0.0),
             bottom: Dimension::Px(0.0),
             left: Dimension::Px(0.0),
         };
         let mut tree = Two::new(10.0);
         tree.props[0].margin = kivact_types::Edges {
             top: Dimension::Px(9.0),
             left: Dimension::Px(7.0),
             ..zero
         };
         tree.props[1].margin = zero;
         tree.props[1].position = kivact_types::Position::Absolute;
         tree.props[1].inset = kivact_types::Edges {
             top: Dimension::Px(30.0),
             right: Dimension::Auto,
             bottom: Dimension::Auto,
             left: Dimension::Px(20.0),
         };
         let layout = laid_out(&tree);
         let root = layout.layout_at(0).expect("root laid out");
         assert_eq!((root.location.x, root.location.y), (0.0, 0.0));
         let child = layout.layout_at(1).expect("child laid out");
         assert_eq!((child.location.x, child.location.y), (20.0, 30.0));
         assert_eq!((child.size.width, child.size.height), (10.0, 10.0));
     }

     #[test]
     fn an_absolute_grid_child_is_placed_in_the_area_of_its_grid_lines() {
         use kivact_types::{
             Breadth, GridContainer, GridLine, GridPos, GridProps, GridTrack, TrackSize,
         };
         let track = |px: f64| {
             GridTrack::Single(TrackSize {
                 min: Breadth::Size(Dimension::Px(px)),
                 max: Breadth::Size(Dimension::Px(px)),
             })
         };
         let second = GridLine {
             start: GridPos::Line(2),
             end: GridPos::Line(3),
         };
         let mut tree = Two::new(10.0);
         tree.props[0].display = kivact_types::Display::Grid;
         tree.props[0].grid = Some(Box::new(GridProps {
             container: Some(GridContainer {
                 columns: vec![track(30.0), track(70.0)],
                 rows: vec![track(40.0), track(60.0)],
                 ..Default::default()
             }),
             ..Default::default()
         }));
         tree.props[1].margin = kivact_types::Edges {
             top: Dimension::Px(0.0),
             right: Dimension::Px(0.0),
             bottom: Dimension::Px(0.0),
             left: Dimension::Px(0.0),
         };
         tree.props[1].position = kivact_types::Position::Absolute;
         tree.props[1].inset = kivact_types::Edges {
             top: Dimension::Px(5.0),
             right: Dimension::Auto,
             bottom: Dimension::Auto,
             left: Dimension::Px(5.0),
         };
         tree.props[1].grid = Some(Box::new(GridProps {
             column: second,
             row: second,
             ..Default::default()
         }));
         let layout = laid_out(&tree);
         // The area of column line 2..3 and row line 2..3 starts at (30, 40);
         // the insets move the box 5 px into it.
         let child = layout.layout_at(1).expect("child laid out");
         assert_eq!((child.location.x, child.location.y), (35.0, 45.0));
         assert_eq!((child.size.width, child.size.height), (10.0, 10.0));
     }
     ```
     （`kivact_types::Edges`・`Dimension`・`GridLine` は `Copy` なので、`..zero` や `second` を二度使える。）
  1. root `Cargo.toml` の `[workspace.dependencies]` で `taffy = { version = "0.14.0", default-features = false, features = ["std", "flexbox", "grid", "calc", "content_size"] }` の `"0.14.0"` を `"0.15.1"` にする（features は変えない）。次を実行する。
     ```bash
     cargo metadata --format-version 1 >/dev/null
     cargo update -p taffy --precise 0.15.1
     awk '/^name = /{n=$3} /^version = /{print n, $3}' Cargo.lock | grep -E '^"taffy" '
     ```
     最後の行の出力は `"taffy" "0.15.1"` の 1 行だけであること。
  2. `crates/kivact-layout/src/style/node.rs`：
     1. module の doc の次の 1 行を探す。
        ```rust
        //! - The defaults are taffy's (`taffy::Style::DEFAULT`): a row flex box.
        ```
        それを次の 3 行にする。
        ```rust
        //! - The defaults are taffy's (`taffy::Style::DEFAULT`): a row flex box,
        //!   except `position`: `Relative`, because taffy's default `Static` is no
        //!   containing block and the layout pass needs every node to be one (`tree::root`).
        ```
     2. `use taffy::{ ... };` の並びに `OofItemStyle` を足す（アルファベット順の位置。`cargo fmt` が並べ直す）。
     3. `pub(crate) struct NodeStyle` の field の型を変える：`pub(crate) align_items: Option<AlignItems>,` → `pub(crate) align_items: AlignItems,`、`pub(crate) align_content: Option<AlignContent>,` → `pub(crate) align_content: AlignContent,`、`pub(crate) justify_content: Option<JustifyContent>,` → `pub(crate) justify_content: JustifyContent,`。この 3 つの field のそれぞれ直前の行が `// xtask-allow(` で始まる comment なら、その行を消す（W1b の後は通常は無い）。`align_self: Option<AlignSelf>` は変えない。
     4. `struct NodeStyle` の上に `// xtask-allow(state): legacy: ` で始まる行があり、その中に `align_items`・`align_content`・`justify_content` の語があれば、その 3 語と、それぞれの直後の `, ` を消す。語の直後が `, ` でなく `;` なら（field の列の最後の語）、語と直前の `, ` を消す。他の文字は変えない。行が無ければ何もしない。
     5. `impl Default for NodeStyle` の `position: d.position,` を `position: Position::Relative,` にする。`align_items: d.align_items,`、`align_content: d.align_content,`、`justify_content: d.justify_content,` はそのまま（taffy 0.15 の `TStyle::DEFAULT` の値は `NORMAL`）。
     6. `impl FlexboxContainerStyle for NodeStyle` の 3 つの method の戻り値の型を変える：`fn align_content(&self) -> Option<AlignContent>` → `fn align_content(&self) -> AlignContent`、`fn align_items(&self) -> Option<AlignItems>` → `fn align_items(&self) -> AlignItems`、`fn justify_content(&self) -> Option<JustifyContent>` → `fn justify_content(&self) -> JustifyContent`（本体は変えない）。
     7. ファイルの最後（`impl GridItemStyle for NodeStyle` の閉じ括弧の後）に次を足す。
        ```rust
        impl OofItemStyle for NodeStyle {
            #[inline(always)]
            fn align_self(&self) -> Option<AlignSelf> {
                self.align_self
            }

            #[inline(always)]
            fn justify_self(&self) -> Option<AlignSelf> {
                self.grid_area.as_deref().and_then(|area| area.justify_self)
            }

            // An absolute child of a grid container is placed in the area of
            // these lines (taffy reads the grid back through
            // `get_detailed_layout_info`), as in taffy 0.14.
            #[inline(always)]
            fn grid_row(&self) -> Line<GridPlacement<String>> {
                line(
                    self.grid_area
                        .as_deref()
                        .map_or(GridLine::default(), |a| a.row),
                )
            }

            #[inline(always)]
            fn grid_column(&self) -> Line<GridPlacement<String>> {
                line(
                    self.grid_area
                        .as_deref()
                        .map_or(GridLine::default(), |a| a.column),
                )
            }
        }
        ```
  3. `crates/kivact-layout/src/style/convert.rs`：
     - `fn taffy_style` の中：`align_items: l.align_items.map(align),` → `align_items: l.align_items.map_or(AlignItems::NORMAL, align),`、`justify_content: l.justify.map(content),` → `justify_content: l.justify.map_or(JustifyContent::NORMAL, content),`、`align_content: l.align_content.map(content),` → `align_content: l.align_content.map_or(JustifyContent::NORMAL, content),`。`align_self: l.align_self.map(align),` と `justify_self: g.justify_self.map(align),` は変えない。
     - `fn grid` の中：`style.justify_items = c.justify_items.map(align);` → `style.justify_items = c.justify_items.map_or(AlignItems::NORMAL, align);`。
     - `position: match l.position { ... }` は変えない（`Relative`・`Sticky` → `TPos::Relative`、`Absolute`・`Fixed` → `TPos::Absolute`。taffy の `Static`・`Fixed`・`Sticky` は使わない）。
  4. `crates/kivact-layout/src/style.rs`：doc の次の 1 行を探す。
     ```rust
     //! - An unset `justify` (or `align_content`) stays unset: taffy's `normal`,
     ```
     それを次の 1 行にする（次の行 `//!   the start in a flex box ...` はそのまま）。
     ```rust
     //! - An unset `justify` (or `align_content`) is taffy's `normal` (`NORMAL`),
     ```
  5. `crates/kivact-layout/src/pass_state.rs`：`impl Default for LayoutState` の `no_grid: TStyle::default(),` を次にする。
     ```rust
     // Positioned like every node (`NodeStyle::default`), so a grid
     // container without tracks is the containing block of its absolute children.
     no_grid: TStyle {
         position: taffy::Position::Relative,
         ..TStyle::default()
     },
     ```
  6. calc を tag で探す所を直す。まず `crates/kivact-layout/src/tree.rs` の `fn relative_length` の全体（`fn relative_length(raw: taffy::style::CompactLength) -> bool {` から対応する `}` まで、中の `use taffy::style::CompactLength;` と `matches!(...)` を含む）を次で置き換える。
     ```rust
     /// Whether `raw` is a percentage or a calc (taffy 0.15 keeps a tag in the low
     /// bits of a calc handle, so a calc is found by `is_plain_calc`, not its tag).
     fn relative_length(raw: taffy::style::CompactLength) -> bool {
         raw.tag() == taffy::style::CompactLength::PERCENT_TAG || raw.is_plain_calc()
     }
     ```
     次に残りの 3 か所（`tree.rs` の `fn resolve_dimension`、`columns.rs` の `fn scaled_dimension` と `fn scaled_length`）を置換する。
     ```bash
     perl -pi -e 's/CompactLength::CALC_TAG =>/_ if value.into_raw().is_plain_calc() =>/' crates/kivact-layout/src/tree.rs crates/kivact-layout/src/columns.rs
     rg -n "CALC_TAG" crates/kivact-layout/src
     ```
     `rg` は何も出さないこと。置換した腕はどれも `_ => ...` の腕より前にある（順は変えない）。
  7. 新しいファイル `crates/kivact-layout/src/tree/root.rs` を作る。
     ```rust
     //! Role: The layout of a root and the out-of-flow hooks of the layout pass.
     //!
     //! Taffy 0.15 leaves an absolute box to its containing block: a container's
     //! layout lists the box and the containing block places it in its own pass
     //! (`compute_oof_layout`). Every node here is positioned (`Relative` or
     //! `Absolute`; `NodeStyle::default` is `Relative`), so the containing block of
     //! an absolute box is always its layout parent: the box's location stays
     //! relative to the parent's border box, the incremental rounding reaches it as
     //! a child, and no hoisted list is kept.
     //!
     //! An absolute child of a grid container is placed in the area of its grid
     //! lines, as in taffy 0.14: taffy hands each full grid layout's track
     //! geometry to `set_detailed_grid_info` and reads it back in the out-of-flow
     //! pass (`get_detailed_layout_info`). It is kept (`NodeLayoutValues::grid_infos`)
     //! only for a grid container with an absolute child; any other grid keeps
     //! nothing, and `reset` and `rebuild_style` drop a slot's entry.
     //!
     //! A root is laid out as taffy 0.14 did: at the origin, its margins taken out
     //! of the space it lays out in. (Taffy 0.15's `compute_root_layout` would move
     //! a root by its margins and insets and size an absolute root to fit.)

     use super::LayoutPass;
     use crate::LayoutTree;
     use crate::style::node::{NodeStyle, TStyle};
     use taffy::{
         AvailableSpace, CoreStyle, DetailedGridInfo, DetailedLayoutInfo, Layout,
         LayoutContainingBlock, LayoutGridContainer, LayoutInput, LayoutOutput, LayoutPartialTree,
         Line, NodeId as TId, Point, Position, RequestedAxis, ResolveOrZero, RunMode, Size,
         SizingMode, TraversePartialTree,
     };

     /// Lay out the root `root` in `space` and store its unrounded layout.
     pub(super) fn layout_root<Tree: LayoutTree + ?Sized>(
         pass: &mut LayoutPass<'_, Tree>,
         root: TId,
         space: Size<AvailableSpace>,
     ) {
         let styles = pass.styles;
         let calcs = pass.calcs;
         let style: &NodeStyle = &styles[usize::from(root)];
         let resolve = |handle, basis| calcs.resolve(handle, basis);
         let parent_size = space.into_options();
         let margin = style.margin().resolve_or_zero(parent_size.width, resolve);
         let padding = style.padding().resolve_or_zero(parent_size.width, resolve);
         let border = style.border().resolve_or_zero(parent_size.width, resolve);
         let inputs = LayoutInput {
             run_mode: RunMode::PerformLayout,
             sizing_mode: SizingMode::InherentSize,
             axis: RequestedAxis::Both,
             known_dimensions: Size::NONE,
             known_dimensions_are_definite: Size {
                 width: true,
                 height: true,
             },
             parent_size,
             available_space: Size {
                 width: without(space.width, margin.left + margin.right),
                 height: without(space.height, margin.top + margin.bottom),
             },
             vertical_margins_are_collapsible: Line::FALSE,
         };
         let output = pass.compute_child_layout(root, inputs);
         debug_assert!(
             output.oof_candidates.is_empty(),
             "invariant: every node is positioned, so a parent places each absolute child"
         );
         pass.set_unrounded_layout(
             root,
             &Layout {
                 order: 0,
                 location: Point::ZERO,
                 size: output.size,
                 scrollable_overflow_rect: output.scrollable_overflow_rect,
                 scrollbar_size: Size::ZERO,
                 padding,
                 border,
                 margin,
             },
         );
     }

     /// `space` less `margins`: a definite available space is the border box's.
     fn without(space: AvailableSpace, margins: f32) -> AvailableSpace {
         match space {
             AvailableSpace::Definite(value) => AvailableSpace::Definite((value - margins).max(0.0)),
             other => other,
         }
     }

     /// Place the absolute children of `t` after a full layout of it (`run_mode`
     /// is `PerformLayout`); a measure must not place them.
     #[inline(always)]
     pub(super) fn place_out_of_flow<Tree: LayoutTree + ?Sized>(
         pass: &mut LayoutPass<'_, Tree>,
         t: TId,
         run_mode: RunMode,
         output: &mut LayoutOutput,
     ) {
         if run_mode == RunMode::PerformLayout {
             taffy::compute_oof_layout(pass, t, output);
         }
     }

     impl<Tree: LayoutTree + ?Sized> LayoutContainingBlock for LayoutPass<'_, Tree> {
         type OofItemStyle<'a>
             = &'a NodeStyle
         where
             Self: 'a;

         #[inline(always)]
         fn get_oof_item_style(&self, t: TId) -> Self::OofItemStyle<'_> {
             self.item_style(t)
         }

         // The containing block of an absolute box is its layout parent (see the
         // module docs): no hoisted list is kept.
         #[inline(always)]
         fn clear_hoisted_children(&mut self, _t: TId) {}

         #[inline(always)]
         fn add_hoisted_children(&mut self, _t: TId, _hoisted: &[TId]) {}

         /// The kept grid of `t` (see the module docs), else none.
         #[inline(always)]
         fn get_detailed_layout_info(&self, t: TId) -> &DetailedLayoutInfo<String> {
             self.grid_infos
                 .get(&usize::from(t))
                 .unwrap_or(&DetailedLayoutInfo::None)
         }
     }

     impl<Tree: LayoutTree + ?Sized> LayoutGridContainer for LayoutPass<'_, Tree> {
         type GridContainerStyle<'a>
             = &'a TStyle
         where
             Self: 'a;

         type GridItemStyle<'a>
             = &'a NodeStyle
         where
             Self: 'a;

         #[inline(always)]
         fn get_grid_container_style(&self, t: TId) -> Self::GridContainerStyle<'_> {
             self.styles[usize::from(t)]
                 .display
                 .grid()
                 .unwrap_or(self.no_grid)
         }

         #[inline(always)]
         fn get_grid_child_style(&self, t: TId) -> Self::GridItemStyle<'_> {
             self.item_style(t)
         }

         /// Keep the grid of `t` while it has an absolute child (its out-of-flow
         /// pass places that child by its grid lines); drop it otherwise.
         fn set_detailed_grid_info(&mut self, t: TId, info: DetailedGridInfo<String>) {
             let index = usize::from(t);
             let absolute = self
                 .child_ids(t)
                 .any(|child| self.item_style(child).position == Position::Absolute);
             if absolute {
                 self.grid_infos
                     .insert(index, DetailedLayoutInfo::Grid(Box::new(info)));
             } else {
                 self.grid_infos.remove(&index);
             }
         }
     }
     ```
  8. `crates/kivact-layout/src/tree.rs`：
     1. module の doc の `Invariants:` の最後の項目（次の 2 行）を探す。
        ```rust
        //! - A box whose own lengths are absolute is cached without its parent's size
        //!   (`relative`).
        ```
        その 2 行の後に、次の 4 行を足す。
        ```rust
        //! - Every node is positioned (`Relative` or `Absolute`), so an absolute
        //!   box's containing block is its layout parent: the parent places it in its
        //!   own full layout (`root::place_out_of_flow`), and a root is laid out at
        //!   the origin as taffy 0.14 did (`root::layout_root`).
        ```
     2. `use taffy::{ ... };` の並びから `RoundTree, ` を消す。その `use` の文の直後に次の 2 行を足す。
        ```rust
        mod root;
        use root::{layout_root, place_out_of_flow};
        ```
     3. `fn same_from_outside` の `..*after` を `..after.clone()` にする（`LayoutOutput` は `Copy` でなくなった）。
     4. `fn rounded` の doc の 4 行の最後の行（次の 1 行）を探す。
        ```rust
        /// with taffy's `round`).
        ```
        それを次の 3 行にする。
        ```rust
        /// with taffy's `round`). Taffy 0.15 rounds a location from the cumulative
        /// position instead; this crate keeps the parent-relative rounding, so the
        /// upgrade moves no rounded box.
        ```
     5. `impl<Tree: LayoutTree + ?Sized> LayoutPartialTree for LayoutPass<'_, Tree>` の `fn compute_child_layout` の closure の中で、`let mut output = match (display, pass.child_count(t) == 0) {` で始まり `(_, true) => pass.leaf(t, natural_inputs),` の次の `};` で終わる文の直後に、次の 1 行を足す。
        ```rust
        place_out_of_flow(pass, t, inputs.run_mode, &mut output);
        ```
     6. 行 `impl<Tree: LayoutTree + ?Sized> RoundTree for LayoutPass<'_, Tree> {` を `impl<Tree: LayoutTree + ?Sized> LayoutPass<'_, Tree> {` にし、その中の `#[inline(always)]` と `fn get_unrounded_layout(&self, t: TId) -> Layout {` から対応する `}` まで（4 行）を消す。残る `fn set_final_layout` の直前の `#[inline(always)]` の上に、次の doc の 1 行を足す（本体は変えない）。
        ```rust
        /// Store the rounded layout of `t` and report how its geometry changed.
        ```
     7. `fn compute_at` の `taffy::compute_root_layout(pass, root, space);` を `layout_root(pass, root, space);` にする。
     8. `impl<Tree: LayoutTree + ?Sized> LayoutGridContainer for LayoutPass<'_, Tree> {` から対応する `}` まで（`get_grid_container_style` と `get_grid_child_style` を持つ block の全体）を消す（手順 7 の `root.rs` に移した）。`use taffy::{ ... };` の並びから `LayoutGridContainer, ` を消す。
     9. `struct LayoutPass` の field の `never_rounded: &'a mut BTreeSet<usize>,` の行の直後に次の 2 行を足す。
        ```rust
        /// The kept grids of the grid containers with an absolute child (`root`).
        grid_infos: &'a mut BTreeMap<usize, taffy::DetailedLayoutInfo<String>>,
        ```
     10. `fn with_pass` の `let NodeLayoutValues { ... } = values;` の並びの `never_rounded,` の次に `grid_infos,` を足し、`let mut pass = LayoutPass { ... };` の `never_rounded,` の次に `grid_infos,` を足す。
     11. `fn rebuild_style` の `self.values.styles[index] = self.values.recent_styles.share(style);` の行の直後に次の 1 行を足す（style が変われば、次の full layout が grid を保存し直す）。
         ```rust
         self.values.grid_infos.remove(&index);
         ```
  9. `crates/kivact-layout/src/columns.rs`：
     1. `pub(crate) struct NodeLayoutValues` の field の `pub(crate) never_rounded: BTreeSet<usize>,` の後（struct の閉じ括弧の前）に次を足す。
        ```rust
        /// The track geometry of each grid container (`position + 1`) that has an
        /// absolute child, from its last full layout: the out-of-flow pass places
        /// that child by its grid lines (`tree::root`). Other slots have no entry.
        pub(crate) grid_infos: BTreeMap<usize, taffy::DetailedLayoutInfo<String>>,
        ```
     2. `impl Default for NodeLayoutValues` の `never_rounded: BTreeSet::new(),` の次に `grid_infos: BTreeMap::new(),` を足す。
     3. `pub(crate) fn reset(&mut self, slot: usize)` の `values.scaled.remove(&index);` の行の直後に `values.grid_infos.remove(&index);` を足す。
- **消すもの**：`impl RoundTree for LayoutPass`（inherent の `impl` にする）、`fn get_unrounded_layout`、`taffy::compute_root_layout` の呼出し、`tree.rs` の `use taffy::{...}` の `RoundTree` と `LayoutGridContainer`、`tree.rs` の `impl LayoutGridContainer for LayoutPass`（`root.rs` へ移す）、`CompactLength::CALC_TAG` の 4 か所。
- **状態の閉じ方**：新しい状態の型は無い。`NodeLayoutValues::grid_infos` は taffy の結果の派生の表（一つの書き手 `set_detailed_grid_info`、消すのは `reset`・`rebuild_style` と同じ関数の `else` の腕）で、6.1 節 (a) の種類の field ではない。`LayoutPass` に足す `grid_infos` は借用（`&mut`）である。`NodeStyle` の `Option` の field を 3 つ値にする（印の文面は手順 2 の 4 で 3 語を消すだけ。残る `aspect_ratio`・`align_self`・`grid_area` の扱いは CP1 の台帳で設計者が決める）。
- **試験**：`crates/kivact-layout/src/layout/tests.rs` に `an_absolute_child_is_placed_from_its_parent_and_a_root_stays_at_the_origin` と `an_absolute_grid_child_is_placed_in_the_area_of_its_grid_lines` を足す（手順 0。taffy 0.14 で通ることを先に確かめる）。他の試験は変えない。
- **確認（tier a）**：
  ```bash
  cargo fmt -p kivact-layout
  cargo check -p kivact-layout --all-targets
  cargo clippy -p kivact-layout --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-layout
  cargo test -p kivact-layout
  cargo check -p kivact-engine --all-targets --features testing
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-engine --features testing
  ```
  E2E（`e2e_core`。module ごとに 1 コマンド）：`layout`、`layout_check`、`layout_scroll`、`layout_units`、`grid`、`flows`、`overlay`、`presence`、`presence_modes`、`paint`、`viewport`、`environment`、`custom_text_field`、`a11y_incremental`、`transform`、`shared_layout`。
  ```bash
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- layout::
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- layout_check::
  # 以下同じ形で layout_scroll:: layout_units:: grid:: flows:: overlay:: presence:: presence_modes:: paint:: viewport:: environment:: custom_text_field:: a11y_incremental:: transform:: shared_layout:: を一つずつ
  ```
  E2E（`e2e_ui`。module ごとに 1 コマンド）：`interaction_dialog`、`interaction_slider`、`interaction_tooltip`、`interaction_menu`。
  ```bash
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_dialog::
  # 以下同じ形で interaction_slider:: interaction_tooltip:: interaction_menu:: を一つずつ
  ```
  最後に 1.3 節の DUP-AFTER。
- **完了条件**：
  - `awk '/^name = /{n=$3} /^version = /{print n, $3}' Cargo.lock | grep -E '^"taffy" '` が `"taffy" "0.15.1"` の 1 行。
  - `rg -n "CALC_TAG|compute_root_layout|RoundTree|get_unrounded_layout" crates/kivact-layout/src` が空。
  - `rg -n "Option<(AlignItems|AlignContent|JustifyContent)>" crates/kivact-layout/src` が空。
  - `rg -n "impl<Tree: LayoutTree \+ \?Sized> LayoutGridContainer" crates/kivact-layout/src` が `tree/root.rs` の 1 行だけ。
  - 手順 0 の 2 つの試験が、版を上げる前と後の両方で期待値を変えずに通った。
  - DUP-AFTER の 1 つ目の `comm` が空。2 つ目の `comm` も空（taffy 0.15 は新しい二重を作らず、消しもしない）。
  - 上の確認のコマンドがすべて成功し、どの E2E のコマンドも 1 件以上の試験を実行した。

## D02：accesskit を 0.25 に上げる

- **wave / lane / 前提**：W2b / D2 / W2a の S02〜S06
- **所有**：root `Cargo.toml` の `[workspace.dependencies]` の `accesskit =`・`accesskit_winit =` の 2 行、`crates/arvane-native/Cargo.toml` の `accesskit_unix =`・`accesskit_windows =` の 2 行、`crates/arvane-native/src/a11y/conversion.rs`、`Cargo.lock`
- **目的**：設計 12.2 の accesskit 0.24 → 0.25（`accesskit_winit` 0.34、`accesskit_windows` 0.35、`accesskit_unix` 0.24）。clone の buffer の再利用（accesskit 0.25 の性能の改善）と、`accesskit_consumer` の二重（今は 0.37 と 0.38）の解消（K6）。zbus は 5 のまま（6.3 節）。
- **手順**：
  0. 1.3 節の DUP-BEFORE を実行する。
  1. root `Cargo.toml` の `[workspace.dependencies]`：`accesskit = "=0.24.1"` → `accesskit = "=0.25.2"`、`accesskit_winit = "=0.33.1"` → `accesskit_winit = "=0.34.2"`。
  2. `crates/arvane-native/Cargo.toml`：`[target.'cfg(target_os = "linux")'.dependencies]` の `accesskit_unix = "=0.22.1"` → `accesskit_unix = "=0.24.1"`、`[target.'cfg(windows)'.dependencies]` の `accesskit_windows = "=0.33.1"` → `accesskit_windows = "=0.35.2"`。`zbus = { workspace = true }` は変えない。
  3. 次を実行する。
     ```bash
     cargo metadata --format-version 1 >/dev/null
     awk '/^name = /{n=$3} /^version = /{print n, $3}' Cargo.lock | grep -E '^"(accesskit[a-z_]*|zbus|phf)" '
     ```
     出力は次のとおりであること：`"accesskit" "0.25.2"`、`"accesskit_atspi_common" "0.21.` で始まる 1 行、`"accesskit_consumer" "0.39.` で始まる 1 行（**1 行だけ**）、`"accesskit_ios" "0.2.` で始まる 1 行、`"accesskit_macos" "0.27.` で始まる 1 行、`"accesskit_unix" "0.24.1"`、`"accesskit_windows" "0.35.2"`、`"accesskit_winit" "0.34.2"`、`"phf" "0.14.` で始まる 1 行、`"zbus" "5.19.0"`。
  4. `crates/arvane-native/src/a11y/conversion.rs` の `fn update` の中の `let mut metadata = accesskit::Tree::new(accesskit::NodeId(root.0));` を `let mut metadata = accesskit::TreeInfo::new(accesskit::NodeId(root.0));` にする（`accesskit::Tree` は 0.25 で `#[deprecated]` の別名）。`rg -n "accesskit::Tree\b" crates` が空になること。
- **消すもの**：`accesskit::Tree` の使用（1 か所）。
- **状態の閉じ方**：新しい型は無い。
- **試験**：足さない・変えない。
- **確認（tier a）**：
  ```bash
  cargo fmt -p arvane-native
  cargo check -p arvane-native --all-targets --all-features
  cargo check -p arvane-native --all-targets --all-features --target x86_64-unknown-linux-gnu
  cargo clippy -p arvane-native --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane-native
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p arvane-native
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p arvane --test e2e -- accessibility::
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native --features embed -- embed::uia::
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- a11y_incremental::
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- landmark_roles::
  ```
  3 つ目（Linux の target の check）が、arvane-native 以外の crate の build script（`cc`・`pkg-config`）の失敗か、target の未導入（error の文に `can't find crate for` と `std` がある）で失敗したときだけ、代わりに次を実行する（作業指示書 0.6）。WSL でも失敗したら `STOP: D02 crates/arvane-native:Linux の check が通らない / <error の最初の行>`。
  ```bash
  ROOT_WIN=$(cygpath -w "$(git rev-parse --show-toplevel)")
  wsl -d "${KIVACT_WSL_DISTRO:-Ubuntu-24.04}" -- bash -lc "cd \"\$(wslpath -a '$ROOT_WIN')\" && CARGO_TARGET_DIR=\$HOME/kv-target/rw-D2-linux CARGO_PROFILE_DEV_DEBUG=0 cargo check -p arvane-native --all-targets --all-features"
  ```
  `embed::uia::` は Windows の試験である（この作業の機械は Windows）。最後に 1.3 節の DUP-AFTER。
- **完了条件**：
  - 手順 3 の出力が手順 3 の列挙のとおり。`rg -n "accesskit::Tree\b" crates` が空。
  - DUP-AFTER の 1 つ目の `comm` が空。2 つ目の `comm` が `"accesskit_consumer" 2` の 1 行（`accesskit_consumer` の二重が消えた）。
  - 上の確認のコマンドがすべて成功し、どの E2E のコマンドも 1 件以上の試験を実行した。

## D03：rfd を 0.18 に、pollster を 1.0 に上げる

- **wave / lane / 前提**：W2b / D2 / D02
- **所有**：root `Cargo.toml` の `[workspace.dependencies]` の `pollster =` の行、`crates/arvane-native/Cargo.toml` の `rfd =` の行、`crates/arvane-native/src/port/win32/services.rs`（comment 1 行）、`Cargo.lock`
- **目的**：設計 12.2 の rfd 0.18 と pollster 1.0。rfd 0.18.0 の Linux の portal の経路は `pollster ^1.0`、rfd 0.17.2 は `pollster ^0.4` を使うので、**同じ unit で両方を上げる**（片方だけだと pollster が二重になる）。pollster 1.0.2 は再入の deadlock を直す。
  rfd 0.18.0 は macOS で `objc2-uniform-type-identifiers` 0.3 を直接使うので、lock に 0.3.2 が入り、既にある 0.2.2（iOS の `objc2-ui-kit` 0.2.2 の依存。winit 0.30 の objc2 0.5 系の経路）と lock の上で二重になる。この二重は**許容する**：2 つの版は別の target（0.3.2 は macOS、0.2.2 は iOS）でしか build されず、一つの target の binary に両方が入ることは無いので、K6（binary の大きさ）は後退しない。objc2 0.5 と 0.6 の系（`objc2`・`objc2-foundation`・`objc2-app-kit`・`block2` など）は既に同じ理由で二重で、winit が objc2 0.6 に移るまで残る（12 章の外）。
- **手順**：
  0. 1.3 節の DUP-BEFORE を実行する。
  1. root `Cargo.toml` の `[workspace.dependencies]`：`pollster = "0.4.0"` → `pollster = "1.0.2"`。
  2. `crates/arvane-native/Cargo.toml` の `[dependencies]`：`rfd = "=0.17.2"` → `rfd = "=0.18.0"`。
  3. 次を実行する。
     ```bash
     cargo metadata --format-version 1 >/dev/null
     cargo update -p pollster --precise 1.0.2
     awk '/^name = /{n=$3} /^version = /{print n, $3}' Cargo.lock | grep -E '^"(rfd|pollster)" '
     ```
     出力は `"pollster" "1.0.2"` と `"rfd" "0.18.0"` の 2 行だけであること。
  4. `crates/arvane-native/src/port/win32/services.rs` の `fn wait` の中の comment の行 `// rfd 0.17.2's async worker can finish its epilogue after sending the` を `// rfd's async worker (seen in 0.17.2) can finish its epilogue after sending the` にする（次の行と `crate::module::retain_module();` は変えない。0.18.0 の Windows の worker にこの点の変更の記録は無く、守りは残す）。
- **消すもの**：なし。
- **状態の閉じ方**：新しい型は無い。
- **試験**：足さない・変えない。
- **確認（tier a）**：
  ```bash
  cargo fmt -p arvane-native
  cargo check -p arvane-native --all-targets --all-features
  cargo check -p arvane-native --all-targets --all-features --target x86_64-unknown-linux-gnu
  cargo check -p arvane-compose --all-targets --features gpu
  cargo check -p kivact-gpu-share --all-targets --all-features
  cargo clippy -p arvane-native --all-targets --all-features -- -D warnings
  cargo clippy -p arvane-compose --all-targets --features gpu -- -D warnings
  cargo clippy -p kivact-gpu-share --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate arvane-native
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p arvane-native
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native -- gpu_on_demand::
  ```
  Linux の target の check の失敗の扱いは D02 と同じ（WSL の形。`rw-D2-linux` の target を使う）。`gpu_on_demand::` は GPU が無ければ `gpu_unavailable` の理由で skip する試験を含む（skip は成功に数える。0 件の実行は失敗）。最後に 1.3 節の DUP-AFTER。
- **完了条件**：
  - 手順 3 の出力が 2 行のとおり。
  - DUP-AFTER の 1 つ目の `comm` が `"objc2-uniform-type-identifiers" 2` の 1 行だけ（目的に書いた、許容する二重）。`cargo tree -i objc2-uniform-type-identifiers@0.3.2 --target all -e normal,build` が rfd 0.18.0 と `objc2-app-kit` 0.3.2 の経路だけを示す。それ以外の行が出たら 1.3 節のとおり STOP。2 つ目の `comm` は空。
  - 上の確認のコマンドがすべて成功した。

## D04：arboard、svgtypes、flate2 を上げる

- **wave / lane / 前提**：W2b / D2 / D03
- **所有**：`crates/arvane-native/Cargo.toml` の `arboard =` の行、`crates/kivact-style/Cargo.toml` の `svgtypes =` の行、`crates/arvane-asset/Cargo.toml` の `flate2 =` の行、`Cargo.lock`
- **目的**：設計 12.2 の arboard 3.7 と svgtypes・flate2 の patch。コードの変更は無い（arboard は text の `Clipboard::new`・`get_text`・`set_text` だけを使い、変わらない。svgtypes は `SimplifyingPathParser` と、usvg・resvg 0.46 が使う。flate2 は `read::GzDecoder` だけ）。
- **手順**：
  0. 1.3 節の DUP-BEFORE を実行する。
  1. `crates/arvane-native/Cargo.toml`：`arboard = { version = "=3.6.1", default-features = false, features = ["wayland-data-control"] }` の `"=3.6.1"` を `"=3.7.0"` にする（features は変えない）。
  2. `crates/kivact-style/Cargo.toml`：`svgtypes = "=0.16.1"` → `svgtypes = "=0.16.2"`。
  3. `crates/arvane-asset/Cargo.toml`：`flate2 = { version = "1.1.10", default-features = false, features = ["rust_backend"] }` の `"1.1.10"` を `"1.1.11"` にする。
  4. 次を実行する。
     ```bash
     cargo metadata --format-version 1 >/dev/null
     cargo update -p flate2 --precise 1.1.11
     awk '/^name = /{n=$3} /^version = /{print n, $3}' Cargo.lock | grep -E '^"(arboard|svgtypes|flate2|x11rb)" '
     ```
     出力は `"arboard" "3.7.0"`、`"flate2" "1.1.11"`、`"svgtypes" "0.16.2"`、`"x11rb" "0.13.2"` の 4 行だけであること（x11rb は一つの版のまま）。
- **消すもの**：なし。
- **状態の閉じ方**：新しい型は無い。
- **試験**：足さない・変えない。
- **確認（tier a）**：
  ```bash
  cargo check -p arvane-native --all-targets --all-features
  cargo check -p arvane-native --all-targets --all-features --target x86_64-unknown-linux-gnu
  cargo check -p kivact-style --all-targets
  cargo check -p arvane-asset --all-targets
  cargo clippy -p arvane-native --all-targets --all-features -- -D warnings
  cargo clippy -p kivact-style --all-targets -- -D warnings
  cargo clippy -p arvane-asset --all-targets -- -D warnings
  cargo xtask check --no-write --crate arvane-native
  cargo xtask check --no-write --crate kivact-style
  cargo xtask check --no-write --crate arvane-asset
  cargo test -p kivact-style
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p arvane-native
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p arvane --test e2e -- assets::
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- canvas::
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- offscreen_svg::
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- svg_embedded_text::
  ```
  Linux の target の check の失敗の扱いは D02 と同じ。最後に 1.3 節の DUP-AFTER。
- **完了条件**：
  - 手順 4 の出力が 4 行のとおり。
  - DUP-AFTER の 1 つ目と 2 つ目の `comm` が空（arboard 3.7 の `windows-sys >=0.52, <0.62` は既にある版で満たされ、新しい二重を作らない）。
  - 上の確認のコマンドがすべて成功し、どの E2E のコマンドも 1 件以上の試験を実行した。

## 4. 他の付属書との接点

### 4.1 W2b の所有の境界

| file | C（C01・C02） | D1（D01） | D2（D02・D03・D04） |
|---|---|---|---|
| root `Cargo.toml` | `members` の `"crates/kivact-offscreen",` の 1 行 | `taffy =` の行 | `accesskit =`・`accesskit_winit =`（D02）、`pollster =`（D03） |
| `Cargo.lock` | 新しい package の項目 | taffy の項目 | accesskit・rfd・pollster・arboard・svgtypes・flate2 の項目 |
| `crates/kivact-layout/**` | — | D01 | — |
| `crates/arvane-native/**` | — | — | D02〜D04 |
| `crates/kivact-style/Cargo.toml`、`crates/arvane-asset/Cargo.toml` | — | — | D04 |

C01・C02 の所有（core の DevTools、`kivact-offscreen`、core・kivact の `Cargo.toml`、facade、`workspace.toml`）と D の所有は file を共有しない（root `Cargo.toml` と `Cargo.lock` は上の行の単位）。W2b の lane の数は 3（C・D1・D2）。

### 4.2 作業指示書への追加の指示（提案。親が作業指示書 4 章の W2b の表と 1.3 節に反映する）

1. W2b の表に次の 2 行を足す。
   | lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
   |---|---|---|---|---|
   | D1 | D01 | D | root `Cargo.toml` の `taffy =` の行、`Cargo.lock`、`crates/kivact-layout/src/**` | — |
   | D2 | D02 → D03 → D04 | D | root `Cargo.toml` の `accesskit =`・`accesskit_winit =`・`pollster =` の行、`Cargo.lock`、`crates/arvane-native/{Cargo.toml,src/a11y/conversion.rs,src/port/win32/services.rs}`、`crates/kivact-style/Cargo.toml`、`crates/arvane-asset/Cargo.toml` | — |
   前提の確認に「D01：W1b の T08・T15、W1c。D02：W2a の lane A（S02〜S06）」を足す。索引の表に `D01 | W2b | D1`、`D02, D03, D04 | W2b | D2` を足す。
2. merge の順は C → D1 → D2。`Cargo.lock` の衝突は 1.3 節のとおり `git checkout --theirs Cargo.lock && cargo metadata --format-version 1 >/dev/null` で作り直す。この作り直しは、後から merge した lane の lock に無い版を、`Cargo.toml` の要求を満たす最新の版で選び直す。版を D の確かめた値に揃えるため、W2b の全 lane の merge の後、tier b の前に次を実行する。
   ```bash
   cargo metadata --format-version 1 >/dev/null
   cargo update -p taffy --precise 0.15.1
   cargo update -p pollster --precise 1.0.2
   cargo update -p flate2 --precise 1.1.11
   awk '/^name = /{n=$3} /^version = /{print n, $3}' Cargo.lock | grep -E '^"(taffy|accesskit|accesskit_winit|accesskit_windows|accesskit_unix|accesskit_consumer|rfd|pollster|arboard|svgtypes|flate2|xim|zbus|x11rb)" '
   ```
   期待：`"accesskit" "0.25.2"`、`"accesskit_consumer" "0.39.`（1 行）、`"accesskit_unix" "0.24.1"`、`"accesskit_windows" "0.35.2"`、`"accesskit_winit" "0.34.2"`、`"arboard" "3.7.0"`、`"flate2" "1.1.11"`、`"pollster" "1.0.2"`、`"rfd" "0.18.0"`、`"svgtypes" "0.16.2"`、`"taffy" "0.15.1"`、`"x11rb" "0.13.2"`、`"xim" "0.5.0"`、`"zbus" "5.19.0"`。違えば `cargo update -p <crate> --precise <期待の版>` で直し、変わった `Cargo.lock` を merge の commit に含める。

### 4.3 後の付属書が前提にしてよいこと（D01・D02 の後のコード）

- **付属書 E（E03・E23・E28 が kivact-layout を編集する）**：
  - taffy は 0.15.1。`taffy::LayoutOutput` と `taffy::Cache` は `Clone` だが `Copy` でない（`Cache::get` は 0.15.1 でも `&mut self`。E28 の `before.caches[at].clone().get(&input)` と `Option<Box<taffy::Cache>>` はそのまま compile する）。
  - `NodeStyle` の `align_items: AlignItems`、`align_content: AlignContent`、`justify_content: JustifyContent` は `Option` でない（未設定は `NORMAL`）。`align_self` と `GridArea::justify_self` は `Option` のまま。`NodeStyle::default().position` は `Position::Relative`。
  - `crates/kivact-layout/src/tree/root.rs` が新しくある：`pub(super) fn layout_root`（root の layout。`compute_at` が呼ぶ）、`pub(super) fn place_out_of_flow`（`compute_child_layout` の closure が呼ぶ）、`impl LayoutContainingBlock for LayoutPass`、`impl LayoutGridContainer for LayoutPass`（`tree.rs` から移した。`set_detailed_grid_info` を持つ）。
  - `NodeLayoutValues` に `grid_infos: BTreeMap<usize, taffy::DetailedLayoutInfo<String>>` があり、`LayoutPass` が `&mut` で借りる。slot を再利用する所（今の `reset`。E03 が `push`・`reset` を一か所にまとめる時もその中）で `grid_infos.remove(&index)` を必ず残す。消すと、再利用した slot の flex container が古い grid の線で absolute の子を置く。`impl RoundTree for LayoutPass` は無く、`set_final_layout` は `LayoutPass` の inherent の method。E28 の手順 2 の「`rg -n "caches\[" crates/kivact-layout/src` の全行」に `root.rs` の行は無い。
  - calc の長さは `CompactLength::is_plain_calc()` で見分ける（`CALC_TAG` は使わない）。新しい code も同じにする。
- **付属書 A（arvane-native を編集する unit：A07・A12・A14 など）**：accesskit は 0.25.2（`accesskit::TreeInfo`）、accesskit_winit 0.34.2、accesskit_windows 0.35.2、accesskit_unix 0.24.1、rfd 0.18.0、arboard 3.7.0、pollster 1.0.2。xim 0.5.0 と zbus 5.19.0 は変えない。adapter の API の署名は前と同じなので、A の手順の文面はそのまま当たる。A04 は usvg 0.46.0 の型（`FontResolver` など）を使い続ける（usvg は待つ群）。A16 の `dep:pollster` の feature の名前は変わらない。
- **付属書 S（S14 の掃引）**：`crates/kivact-layout/src/layout/tests.rs` の新しい試験は `kivact_types::{Dimension, Edges, Position}` を使う。S の改名の掃引の対象に入る（試験の期待値は変えない）。

### 4.4 taffy 0.15 で受け入れる挙動の違い（D01 の決定）

| 項目 | 0.14（今） | D01 の後 | 理由 |
|---|---|---|---|
| root の位置 | 原点。margin・inset で動かない | 同じ（`layout_root` が 0.14 の形を保つ） | Kivact の root は window を埋める。0.15 の `compute_root_layout` は root を margin と inset で動かし、absolute の root を shrink-to-fit にする |
| absolute の box の containing block | 親 | 親（全 node が positioned） | lib.rs の Limits のとおり。hoisted の list を持たない（node あたりの memory を増やさない。K2） |
| absolute の子の grid の線 | grid の線の間の area に置いた | 同じ（`grid_infos` に保存した grid を `get_detailed_layout_info` で返す） | 能力を黙って落とさない（親の決定）。保存は absolute の子を持つ grid container だけ。注：taffy 0.15 自身は full の grid layout ごとに `DetailedGridInfo` を作って `set_detailed_grid_info` に渡す（taffy の中の費用で、fork しないと避けられない。DEC-18）。Kivact は不要なものをその場で捨て、保存の確保をしない |
| 丸め | 位置は親からの相対で丸める | 同じ | 0.15 の `round_layout` は累積の座標で位置を丸めるが、Kivact は自前の差分の丸め（`round_changed`）を持つ。変えると丸めた画素が動く |
| calc の長さ | handle の下位の byte が 0 の時だけ calc と分かった（`relative`・presence の scale・`resolve_dimension` で見落とし） | `is_plain_calc()` で常に分かる | 不具合の修正。calc の box が親の大きさの変化で正しく layout し直される |
| 未設定の `align_items` など | `None` | `NORMAL` | 0.15 の CHANGELOG のとおり同じ意味 |

### 4.5 設計書への訂正（親が設計書 12.2 に反映済み）

- 設計書 12.2 の「待つ群」の `skrifa 0.50` は、2026-11-18 の crates.io では揃わない：最新の glifo 0.5.0・parley_engine 0.13.0（parley 0.13 の本体）・usvg 0.50.0 はいずれも `skrifa ^0.49` と（parley_engine・usvg は）`harfrust ^0.15` を要る。skrifa 0.50.0（read-fonts 0.47）と harfrust 0.16.0 はそれより新しい。待つ群の目標は「三つが共有する skrifa と harfrust の版（今は 0.49 と 0.15）」と読む（6 章の C4・C5）。
- 設計書 12.2 の「zbus 6（accesskit_unix の要求と揃える）」は、揃えた結果として zbus 5.19 のまま（6.3 節）。

## 5. 自己点検

- 設計 12.2 の「いま上げる群」の全 crate が unit か理由のある見送りにある：taffy（D01）、accesskit・accesskit_winit・accesskit_windows・accesskit_unix（D02）、rfd・pollster（D03）、arboard・svgtypes・flate2（D04）、zbus（見送り、6.3 節）、xim（見送り、6.3 節）。「待つ群」は 6 章（unit なし）。
- 各 unit は K6 の確認（DUP-BEFORE・DUP-AFTER）と、`Cargo.lock` の版の確認を持つ。二重を作る組合せ（pollster と rfd、xim と x11rb、zbus と accesskit_unix）は同じ unit にするか見送った。D03 の `objc2-uniform-type-identifiers`（0.2.2 は iOS、0.3.2 は macOS）は、別の target でしか build されない lock の上だけの二重なので許容した（D03 の目的）。
- 設計の判断を実装者に残していない：版、Cargo.toml の文字列、コードの差し替え、試験、E2E の module と binary、Linux の check の代わりの経路（作業指示書 0.6 の規則のとおり）をすべて書いた。D01 の挙動の違いは 4.4 節で決めた。
- 同じ wave の二つの unit が同じ file を持たない：D01（lane D1）と D02〜D04（lane D2）は crate の file を共有しない。root `Cargo.toml` と `Cargo.lock` だけは行・項目の単位で C・D1・D2 が分け、merge の後の版の確認を 4.2 節に書いた。lane D2 の中の三つは直列。
- 互換のコードは無い（`accesskit::Tree` の別名を使わず `TreeInfo` にする。`RoundTree` の空の実装を残さない）。新しい依存の crate は足さない（taffy 0.15 が引く `smallvec` は既に lock にある）。`#[allow]`・`#[expect]`・`xtask-allow` は足さない。
- file の大きさ：`tree.rs` は今 978 code 行。D01 の差し引きで増えるのは数行で 1000 未満。`impl LayoutGridContainer`（約 20 code 行）を `tree.rs` から出すので、`tree.rs` はむしろ減る。新しいコードの大部分は `tree/root.rs`（約 120 code 行）に置いた。`compute_child_layout` は 77 code 行から 1 行増える（clippy の上限 80 以内）。
- PLAN-04（固定の `cd` なし）、PLAN-07（binary ごとに 1 コマンド、0 件は失敗）、window-lock は一つのコマンドに一度だけ、を全 unit で守った。

## 6. 待つ群と見送り（unit なし。Orchestrator が CP2・CP3 で確かめて報告する）

### 6.1 待つ群：今の版と、塞いでいる依存の辺（2026-11-18、crates.io で確かめた）

今の `Cargo.lock`：wgpu・naga 29.0.4、vello_cpu・vello_common・vello_hybrid 0.2.0、glifo 0.3.0、parley・fontique 0.11.1、skrifa 0.44.0（read-fonts 0.41.0）、harfrust 0.12.0、usvg・resvg 0.46.0（rustybuzz 0.20.1、fontdb 0.23.0、tiny-skia 0.11.4 を引く）。

最新の版：wgpu・naga 30.0.2、vello_cpu・vello_common 0.4.0、**vello_hybrid 0.2.0（0.3 以降が無い）**、glifo 0.5.0、parley・fontique・parley_engine 0.13.0、skrifa 0.50.0、harfrust 0.16.0、usvg・resvg 0.50.0。

塞いでいる辺：

| 辺 | 要求 | 結果 |
|---|---|---|
| vello_hybrid 0.2.0 → wgpu | `^29.0.3` | wgpu 30 に上げると wgpu・wgpu-core・wgpu-hal・wgpu-types・naga が二重 |
| vello_hybrid 0.2.0 → vello_common | `^0.2.0` | vello_cpu 0.4.0 は `vello_common ^0.4.0`。vello_cpu だけ上げると vello_common が二重 |
| vello_hybrid 0.2.0 → glifo | `^0.3.0` | vello_cpu 0.4.0 は `glifo ^0.5.0` |
| glifo 0.3.0 → skrifa | `^0.44.0` | glifo 0.5.0・parley_engine 0.13.0・usvg 0.50.0 は `skrifa ^0.49.0`。parley や usvg だけを上げると skrifa（と read-fonts・font-types）が二重 |
| parley_engine 0.13.0 → harfrust / usvg 0.50.0 → harfrust | `^0.15.0` / `^0.15.0` | parley 0.11.1 は `harfrust ^0.12.0`。usvg だけを上げると harfrust が二重 |
| fontique 0.13.0 → read-fonts | `^0.46.0` | skrifa 0.49 の read-fonts 0.46 と揃う |

全部を揃えて上げられる組（vello_hybrid の新しい版が出たとき）：wgpu・naga 30、vello_cpu・vello_common・vello_hybrid（同じ vello_common の線）、glifo 0.5 以降、parley・fontique 0.13 以降、skrifa 0.49 の線（read-fonts 0.46）、harfrust 0.15 の線、usvg・resvg 0.50（rustybuzz が消え、fontdb 0.24・tiny-skia 0.12 になる）。

### 6.2 再開の条件と確かめ方

**再開の条件**（すべて満たす）：

- C1：最新の vello_hybrid と最新の vello_cpu が同じ vello_common の線（0.x の minor）を要る。
- C2：最新の vello_hybrid が wgpu 30 以降を要る。
- C3：最新の vello_hybrid と最新の vello_cpu が同じ glifo の線を要る。
- C4：最新の glifo・parley_engine・usvg が同じ skrifa の線を要る。
- C5：最新の parley_engine と usvg が同じ harfrust の線を要る。

**確かめ方**：CP2 と CP3 で、Orchestrator が worktree の root で次を実行する（crates.io の公開の情報を読むだけ。ファイルを作らず、何も上げない）。

```bash
python -I - <<'PY'
import json, re, time, urllib.request
UA = {"User-Agent": "generic-client/1.0"}
def get(path):
    time.sleep(1)
    with urllib.request.urlopen(urllib.request.Request("https://crates.io/api/v1/" + path, headers=UA)) as r:
        return json.load(r)
def newest(name):
    return get(f"crates/{name}")["crate"]["max_stable_version"]
def req(name, dep):
    version = newest(name)
    for d in get(f"crates/{name}/{version}/dependencies")["dependencies"]:
        if d["crate_id"] == dep and d["kind"] == "normal":
            return f"{name} {version} -> {dep} {d['req']}", d["req"]
    return f"{name} {version} -> {dep} (none)", None
def line(r):
    m = re.search(r"(\d+)\.(\d+)", r or "")
    if not m:
        return None
    major, minor = int(m.group(1)), int(m.group(2))
    return (major,) if major > 0 else (0, minor)
def same(*edges):
    return len({line(r) for _, r in edges}) == 1
hy_wgpu = req("vello_hybrid", "wgpu")
hy_common = req("vello_hybrid", "vello_common")
cpu_common = req("vello_cpu", "vello_common")
hy_glifo = req("vello_hybrid", "glifo")
cpu_glifo = req("vello_cpu", "glifo")
skrifa = [req("glifo", "skrifa"), req("parley_engine", "skrifa"), req("usvg", "skrifa")]
harfrust = [req("parley_engine", "harfrust"), req("usvg", "harfrust")]
zbus = req("accesskit_unix", "zbus")
x11rb = [req("xim", "x11rb"), req("winit", "x11rb"), req("softbuffer", "x11rb"), req("arboard", "x11rb")]
for text, _ in [hy_wgpu, hy_common, cpu_common, hy_glifo, cpu_glifo, *skrifa, *harfrust, zbus, *x11rb]:
    print("edge", text)
checks = [
    ("C1 vello_hybrid and vello_cpu share vello_common", same(hy_common, cpu_common)),
    ("C2 vello_hybrid accepts wgpu 30 or later", (line(hy_wgpu[1]) or (0,))[0] >= 30),
    ("C3 vello_hybrid and vello_cpu share glifo", same(hy_glifo, cpu_glifo)),
    ("C4 glifo, parley_engine and usvg share skrifa", same(*skrifa)),
    ("C5 parley_engine and usvg share harfrust", same(*harfrust)),
]
for name, ok in checks:
    print(("ok   " if ok else "FAIL ") + name)
print("wait group: " + ("READY" if all(ok for _, ok in checks) else "BLOCKED"))
print("zbus 6: " + ("READY" if (line(zbus[1]) or (0,))[0] >= 6 else "BLOCKED"))
print("xim (x11rb): " + ("READY" if same(*x11rb) else "BLOCKED"))
PY
```

2026-11-18 の出力（基準）：C1・C2・C3 が `FAIL`、C4・C5 が `ok`、`wait group: BLOCKED`、`zbus 6: BLOCKED`、`xim (x11rb): BLOCKED`。

**Orchestrator がすること**：出力の全文を `rework/PROGRESS.md` に `依存の待つ群（CP2）`・`依存の待つ群（CP3）` の見出しで写す。`READY` の行があれば、CP の報告（人間への報告と、CP3 では `rework/PR_DRAFT.md` の「残る制限」）に「設計書 12.2 の再開の条件を満たした：<READY の行>」と 1 行書く。**自分では上げない**（設計者が新しい付属書の unit で上げる）。`rework/OPEN.md` には書かない（完了を妨げない報告である）。通信できず script が失敗したら、`cargo search vello_hybrid --limit 1` と `cargo search vello_cpu --limit 1` の出力を代わりに写し、「依存の辺は未確認」と書く。

### 6.3 いま上げる群から見送ったもの

| crate | 見送る理由（2026-11-18） | 再開の条件（6.2 の script の行） |
|---|---|---|
| zbus 6 | 最新の accesskit_unix 0.24.1 が `zbus ^5.19`、その依存の atspi 0.29（atspi-common 0.13）が zbus 5 を要る。Kivact の arvane-native も zbus を直接使う（`port/linux_settings.rs`）。6 にすると zbus・zbus_macros・zbus_names・zvariant が二重になる | `zbus 6: READY`（最新の accesskit_unix が zbus 6 を要る）。そのとき accesskit の版上げと同じ unit で上げる |
| xim 0.6 | xim 0.6.0 が `x11rb ^0.14` を要る。winit 0.30.13（最新の安定版。0.31 は beta）、softbuffer 0.4.8、arboard 3.7.0、arvane-native 自身が x11rb 0.13 を要るので、x11rb・x11rb-protocol が二重になる | `xim (x11rb): READY`（xim・winit・softbuffer・arboard の最新が同じ x11rb の線を要る）。そのとき arvane-native の `x11rb` の行と一緒に上げる |
