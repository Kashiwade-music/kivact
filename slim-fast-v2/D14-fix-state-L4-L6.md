# V2 再監査 R6（target lane F）：W6 lane L4〜L6 と FINAL-2

> **V2.1 修正付属書 D14**（再監査 R6 の成果物、HEAD `2a3b06b8`＝W1 の後に対して書いた）。範囲：W6 の lane L4〜L6 と W7 の FINAL-2（状態機械）。
> - **優先順位**：本書は付属書 D1〜D8 より優先する。本書の unit の節の「修正」は、作業指示書の行の「本書の追加の指示」と同じ扱い。本書と V2 設計書 4 章（DEC）が食い違う場合は V2 設計書が正（V2 設計書 4 章の DEC-18 の名前は `while_open(factory)`）。
> - 本書の「全体」節の wave・lane の組み直しは、作業指示書（V2.1）に反映済み。実施順・所有ファイルは作業指示書が正。
> - 文中の worktree（`W:/w/kv-audit-*`）・scratchpad の path は監査の試作の場所で、実装に必要ではない（試作の差分は参考で、実装は本書の文に従う）。

対象 HEAD 2a3b06b8。担当：W6 の L4（IN-1 → IN-2 → IN-3 → P8-IN）、L5（UI-1 → UI-2 → UI-3 → P8-CK → P8-UI5）、L6（UI-4 → SO → CR → EX → DT → P8-UI6 → P8-SOFT → P8-KITS）、W7 の FINAL-2（xtask X3〜X5）。付属書 D5（本文・付録 P5-A〜F）、D8（3 章・付録 A-G*・付録 B）。

方法：リポジトリは読取りだけ。L4・L5・L6 前半・L6 後半・FINAL-2 の 5 つに分けて HEAD のコードと突き合わせた。cargo を実行したのは次の 2 つで、どちらも削除済みの一時 worktree で行った。
- FINAL-2：X3〜X5 を試作し、xtask の試験 74 件と clippy で確かめた。
- P8-SOFT：追加する試験を HEAD で実行した。

それ以外は静的解析である。各節の「修正」は、作業指示書の該当 unit の行の「本書の追加の指示」にそのまま入れる文である。

## R6 共通の確定事項（L4〜L6 の全 unit と FINAL-1 に適用）

### 公開の recipe state の命名規則（DEC-14・DEC-15 の統一）

kit の theme を書く人（JS 開発者に相当）が覚える規則を一つにする。W6 と FINAL-1 の後の名前は、すべて次の規則に合うように確定した。

1. **排他や含意を持つ事実**は、pub の enum の field に置く。
   - field の名前は、型の名前から owner の接頭辞を除いて小文字にしたものにする。
   - 例：`control: CheckControl`、`action: CardAction`、`display: MenuItemDisplay`、`collapse: CodeCollapse`、`availability: Availability`。
   - state の全体が排他な場合は、state そのものを enum にする。例：`BreadcrumbItemState::{Current, Label, Link(input)}`、`DataTableCheckState::{Header(..), Row { checked }}`。
2. **部分を表す enum の名前は `State` で終わらせない。**
   - state-fields の lint は、`State` で終わる enum を phase として数える。
   - 名前の上でも snapshot の struct と紛らわしい。
   - そのため D5 の `CardActionState` は `CardAction` にする。衝突する private の builder の enum は `CardKind` に改名する。
3. **常にある事実は pub の field のまま残す。** 例：`input`、`size`、`invalid`、`destructive`、`selected`。
   - 一部の種類にだけある事実と、他から導く事実は accessor にする。例：`CardState::input() -> Option<PartState>`、`BreadcrumbItemState::input()`、`CheckableState::switch_text()`。
4. **bool を返す accessor は `is_` か `has_` で始める。** それ以外を返す accessor は名詞にする。
   - FINAL-1 の例：`is_pressed()`・`is_hovered()`・`is_disabled()`・`is_hidden()`、既存の `is_focused()`。
   - D8 の例：`is_checked()`・`is_mixed()`・`checked()`・`switch_text()`・`switch_drag()`。
   - この規則で付属書から変えた名前は 4 つある。
     - `ButtonState`/`ToggleButtonState` の `icon_only()` → `is_icon_only()`（E-1）
     - `CodeBlockState` の `header_content` → `has_header_content`（F-T2）
     - creative の node editor の `Info::selected()` → `is_selected()`（C4。crate 内）
     - `AlertState` は `content: Option<Expansion>` に accessor の `has_content()`・`expansion()` を足す（F-T3）
5. **旧 field と同じ情報は、同じ語の accessor で読めるようにする。**
   - 外部の theme 作者の直し方は「`.x` → `.x()`／`.is_x()`」の一通りになる。
   - D5 の E-2 の variant は `MenuItemDisplay::{Action, Check(MenuCheck), Submenu { expanded }}` にする（付属書の `Plain`／`open` から変えた）。同じ module の `MenuItemKind::Action` と、accessor の `is_expanded()` に名前を揃えるため。

W8 の書き戻しでは、SLIM-14（V2 設計書 8 章）の文に上の 1〜4 を 1 行で足す。足す文は「全体」の節にある。

---

# R6 監査：W6 lane L4（IN-1 → IN-2 → IN-3 → P8-IN）

対象 HEAD 2a3b06b8。L4 の所有ファイルは全て `crates/kivact-interaction/src/` と kit の `recipes_menu.rs`。W1（`git log --stat eef6dc9f..HEAD`）が kivact-interaction で変えたのは M10 の `toggle.rs` だけで、L4 の所有ファイルとは重ならない。W2〜W4 の付属書（D1〜D4・D6・D7）に kivact-interaction の L4 ファイルを変える unit は無い。W5 の G7 が `menu/handle.rs` の `trigger_action` を変える（下の IN-2）。

lint の前提（`dev/xtask/src/state_fields/`）：`Value`・`Binding` は数えない。`bool`・`Option`・`Cell/RefCell/Signal/Var<bool|Option|整数>` は数える。名前が `State`・`Phase`・`Status`・`Stage`・`Connection`・`Lifecycle`・`Run`・`Present` で終わる workspace の enum は phase。L4 が足す enum `MenuItemDisplay` は phase にならない。

---

## IN-1（D5 付録 P5-B B1〜B4、P5-F 2 章(b) の `select/handle.rs` 行、W6 lane L4）

### 判定：修正あり（軽微。手順の確定と X4 への注記）

### 修正（作業指示書 9 章 L4 の「IN-1」に「本書の追加の指示」として入れる文）
- 対象のコードは HEAD で付属書のとおり（`ListItemState`、`TreeItemState`、`Edit`、`select::machine::State::Open`、`SelectApi.content_state`、`track_preview`）。行番号だけずれている。
- 手順は次の 6 つだけ。コードの変更は無い（marker と doc 1 文）。
  1. `crates/kivact-interaction/src/list/data.rs` の `pub struct ListItemState`：`pub selected: bool,` の直前に次の行を足し、`pub active: bool,` の直前の `// xtask-allow(state-fields): projection: …` 行を次の 2 行目で置き換える。
     - `    // xtask-allow(state-fields): independent: whether the selection holds the row; a multiple selection or a caller's binding write keeps rows selected while another row is active`
     - `    // xtask-allow(state-fields): independent: whether the row is the roving or highlighted candidate; arrows under Manual activation, a combobox highlight and Ctrl+arrow move it without selecting`
  2. `crates/kivact-interaction/src/tree.rs` の `pub struct TreeItemState`：同じく
     - `selected` の前に `    // xtask-allow(state-fields): independent: copied from the row's ListItemState; a multiple selection or a caller's binding write keeps rows selected while another row is active`
     - `active` の前の行を `    // xtask-allow(state-fields): independent: copied from the row's ListItemState; Ctrl+arrow moves the active row without selecting it` に置き換える。
  3. `crates/kivact-interaction/src/scalar/edit_state.rs` の `pub(crate) struct Edit`：`pub(crate) observed:` の直前に `    // xtask-allow(state-fields): independent: the state of the part the kit attached, absent until input() or thumb() attaches it; single and range thumbs (bounds None or Some) exist both before and after attaching` を足す。`bounds` の marker は変えない。
  4. `crates/kivact-interaction/src/select/machine.rs` の `enum State` の `Open` variant：`previewed` の doc comment `/// The option last reported to the caller's preview callback.` の**上**に `        // xtask-allow(state-fields): projection: written only by track_preview through Event::Preview (the hovered key, else the list's active key); Open starts it at None and Close reports None` を足す。
  5. `crates/kivact-interaction/src/select/handle.rs` の `fn track_preview` の doc comment の最後（`/// surface's owner.` の次の行）に `/// This is the only sender of machine::Event::Preview.` を足す。
  6. 同じ `select/handle.rs` の `pub struct SelectApi`：`content_state` の直前の marker 行を `    // xtask-allow(state-fields): independent: set by SelectApi::content_in when the option list or sheet mounts; the preview callback beside it is set by the caller at setup, before or without any surface` で置き換える（P5-F 2 章(b)）。
- lint の確認（静的）：`ListItemState`・`TreeItemState` は数える field が 2（`selected`、`active`。`PartState` は struct、`TreeBranch` は phase の名前でない、`level: u32` は名前付き counter でない）。`Edit` は 2（`observed`、`bounds`）。`State::Open` は 3（`highlight`、`hovered`、`previewed`）。`SelectApi` は 2（`content_state`、`preview: Var<Option<Callback<..>>>`）。全て marker 付きになる。
- X4（FINAL-2）：`State::Open.previewed` の projection は `State` が `pub(super)`（親 module `select` とその子孫）なので、`select/handle.rs` の free fn `track_preview` は可視範囲にある。可視範囲の書込みは `transition` の中の `State::Open { previewed: None, .. }`（空の初期値）と `*previewed = key.clone()`（`ExprField` が無いので X4 の「書く式」に当たらない）だけで、規則 3 は Machine の例外が無くても満たす。**FINAL-2 への条件**：syn の enum variant の field の `vis` は常に `Inherited` なので、X4 の可視範囲は variant の field については **enum の可視性**で決めること（field の `vis` を使うと `select::machine` だけが範囲になり、`track_preview` が「does not exist where the field is visible」の誤検出になる）。この 1 文を FINAL-2 の X4 の規則 2 に足す。
- X5：上の independent の文は全て workspace で一意（HEAD の全 marker と D5・D8 の全文を grep で確認）。
- 試験：振る舞いは不変。追加・期待値の変更は無い。
- 付属書の残った判断：P5-B 7 章 D3（推測：`ScalarApi::part` を 2 回呼ぶ）は「記録のみ」で確定（変更しない）。P5-F 3 章の推測（`content_state` は unmount で None に戻らない）は W6 では変更しない（試験も足さない。CP2 の全 E2E で見る）。

### 所有ファイルの変更
- なし（D5 6 章の行どおり：`list/data.rs`、`tree.rs`、`scalar/edit_state.rs`、`select/machine.rs`、`select/handle.rs`。全て `crates/kivact-interaction/src/`）。

### 完了時の確認
- `cargo check -p kivact-interaction --all-targets`；`cargo xtask check --no-write --only state-fields --crate kivact-interaction`。

---

## IN-2（D5 付録 P5-E E-2・E-4・E-6 の table の行、P5-F 2 章(b) の `menu/level.rs` 行、W6 lane L4）

### 判定：修正あり

### 修正
**(1) E-2 `MenuItemState`（公開 API。DEC-14）の確定形。** 付属書の形から 3 点を変える：
- variant 名を `Plain` でなく `Action` にする（同じ module の `MenuItemKind::{Action, Checkbox, Radio, Submenu}` と揃える。theme の書き手が二つの enum で別の語を覚えずに済む）。
- `Submenu { open: bool }` を `Submenu { expanded: bool }` にする（accessor `is_expanded()`・HEAD の field 名 `expanded`・ARIA の `aria-expanded` と揃える。同じ型の中で `open` と `expanded` が混ざらない）。
- `destructive` の marker は HEAD の行（`// xtask-allow(state-fields): option: the entry's authored destructive mark, read for a warning color`）を**変えずに残す**（変更後の `MenuItemState` で数える field は `destructive` だけ。marker は使われるので `unused marker` にならず、X3 の後も `total = 1` で error にならない）。

`crates/kivact-interaction/src/menu/state.rs`：`pub struct MenuItemState { .. }`（doc comment から閉じ括弧まで）を次で置き換える。

```rust
/// What kind of row a menu item is, with the state only that kind has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItemDisplay {
    /// A plain item ([`MenuEntry::item`]).
    Action,
    /// A check or radio item ([`MenuEntry::checkbox`], [`MenuEntry::radio`])
    /// and its checked state.
    Check(MenuCheck),
    /// An item that opens a submenu ([`MenuEntry::submenu`]).
    Submenu {
        /// Its submenu is open.
        expanded: bool,
    },
}

/// The state of a menu item for the caller's drawing.
///
/// Most recipes read [`check`](Self::check) and
/// [`is_expanded`](Self::is_expanded); `match state.display` tells the kinds
/// of row apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuItemState {
    /// Which row the tracked menu press is over; [`PressOver::None`] leaves
    /// highlighting to the ordinary focus and hover states.
    pub pressed_over: PressOver,
    pub input: PartState,
    /// What kind of row this is, with its kind's state.
    pub display: MenuItemDisplay,
    // xtask-allow(state-fields): option: the entry's authored destructive mark, read for a warning color
    /// The entry is marked destructive ([`MenuEntry::destructive`]), for a
    /// warning color.
    pub destructive: bool,
}

impl MenuItemState {
    /// The checked state of a check or radio item; `None` for other items.
    pub fn check(&self) -> Option<MenuCheck> {
        match self.display {
            MenuItemDisplay::Check(check) => Some(check),
            MenuItemDisplay::Action | MenuItemDisplay::Submenu { .. } => None,
        }
    }

    /// Whether this is a submenu item whose submenu is open.
    pub fn is_expanded(&self) -> bool {
        matches!(self.display, MenuItemDisplay::Submenu { expanded: true })
    }
}
```

（`MenuItemDisplay::Submenu` は数える field が 1 個なので marker は置かない。`MenuItemState` の旧 `check`・`expanded` の 2 本の projection marker は field と一緒に消える。）

**(2) 書き手 1：`crates/kivact-interaction/src/menu/level.rs`。** `Level::item` は HEAD で本体 77 行（clippy `too-many-lines-threshold = 80`、`too_many_lines = "deny"`）。付属書の `display: match … { … }` をその場に書くと rustfmt 後に 81 行以上になり clippy が落ちる。そこで helper に出す：
- ファイルの末尾（`impl<K> Level<K>` の外、private の free fn が並ぶ所）に次を足す。

```rust
/// What kind of row an item is: its checked state for a check or radio item,
/// whether its submenu is open for a submenu item.
fn item_display(check: Option<MenuCheck>, submenu: bool, expanded: bool) -> MenuItemDisplay {
    match check {
        Some(check) => MenuItemDisplay::Check(check),
        None if submenu => MenuItemDisplay::Submenu { expanded },
        None => MenuItemDisplay::Action,
    }
}
```
- `Level::item` の `member.map_state(move |state| MenuItemState { … })` の中の 2 行 `check: check.get(),` と `expanded: expanded.get(),` を 1 行 `display: item_display(check.get(), submenu, expanded.get()),` に置き換える（`check` と `expanded` の memo は `.checked(..)`・`.expanded(..)` の semantics でも使うので残す）。
- 先頭の `use super::state::{Edge, MenuCheck, MenuEntry, MenuItemKind, MenuItemState, Opened, Target, Want};` に `MenuItemDisplay` を足す（アルファベット順で `MenuEntry` の次）。
- 同じファイルの `pub(super) struct Level`：`content_state` の直前の marker 行を `    // xtask-allow(state-fields): independent: set by MenuApi::content when this level's surface mounts; a submenu level exists in its session before its surface first mounts` で置き換える（P5-F 2 章(b)）。`retain_disabled_focus` の marker は変えない。

**(3) 書き手 2：`crates/kivact-interaction/src/menu/handle.rs`（G7 の後のコードに書く）。**
- G7（W5 lane B、D6 C-12）は同じファイルの `pub fn trigger_action` の署名（`impl Into<Callback<()>>`）と本体だけを変える。IN-2 はその直前の `pub fn item_state` の fallback の literal と import だけを変えるので、hunk は重ならない。G7 の変更を戻さない・触らない。
- `pub fn item_state(self, key: K) -> Value<MenuItemState>` の `.unwrap_or(MenuItemState { … })` を次にする。

```rust
                .unwrap_or(MenuItemState {
                    input: PartState::default(),
                    pressed_over: PressOver::None,
                    display: MenuItemDisplay::Action,
                    destructive: false,
                })
```
- 先頭の `use super::state::{MenuEntry, MenuItemState, MenuOrigin, MenuRow, PressOver};` を `use super::state::{MenuEntry, MenuItemDisplay, MenuItemState, MenuOrigin, MenuRow, PressOver};` にする。
- `item_state` の doc（`/// Drawing state of the actual mounted menu item.`）の次に `/// Before the item mounts it reads as a plain item (`MenuItemDisplay::Action`).` を足す。

**(4) export。** `crates/kivact-interaction/src/menu.rs` の `pub use state::{ MenuCheck, MenuEntry, MenuItemKind, MenuItemState, … }` と `crates/kivact-interaction/src/lib.rs` の `pub use menu::{ … }` に `MenuItemDisplay` を足す（どちらも `MenuItemKind` の前、アルファベット順）。facade（`crates/kivact/src/lib.rs`）は `pub use kivact_interaction as interaction;` の module の別名なので**変えない**（付属書 E-U1 の「facade の re-export 行」は不要）。

**(5) 読み手（全数。Grep で `MenuItemState` の field を読む箇所はこの 4 つだけ）。** `Look<MenuItemState>` は Deref で method を呼べる。
- `crates/kivact-ui-basic/src/recipes_menu.rs`：`fn when_lit` の `if s.expanded {` → `if s.is_expanded() {`；`fn item` の `match (s.check.is_some(), look.bar)` → `match (s.check().is_some(), look.bar)`；`fn item` の末尾の `if s.expanded {` → `if s.is_expanded() {`。
- `crates/kivact-ui-soft/src/recipes_menu.rs`：`let base = if s.expanded {` → `let base = if s.is_expanded() {`。
- `when_lit` の `s.input.disabled` は**変えない**（W7 の FINAL-1 が compile error で直す）。
- `kivact-ui`・`-pop`・`-game`・`-creative`、`dev/tests`、`docs` に `MenuItemState` の `check`・`expanded` の読み手は無い（`entry.check()` は `MenuEntry::check` で別物。`ui_theming.rs` は `input.is_focused()` だけ）。

**(6) E-4 `inline_edit.rs`（`reported` を導出にする）の確定コード。**
- `struct InlineEdit<K>` と `impl Clone`・`impl PartialEq` を次で置き換える。

```rust
/// The rename request. The key last written to the caller's `editing`
/// binding is always the key of the phase, so it is not stored.
struct InlineEdit<K: ItemKey> {
    phase: Phase<K>,
}
impl<K: ItemKey> Clone for InlineEdit<K> {
    fn clone(&self) -> Self {
        Self {
            phase: self.phase.clone(),
        }
    }
}
impl<K: ItemKey> PartialEq for InlineEdit<K> {
    fn eq(&self, other: &Self) -> bool {
        self.phase == other.phase
    }
}
```
- `transition` の `Event::Request(key) => { … }` の腕を次にする（`Event::Request(Some(key)) if … => Step::stay(),` の腕はそのまま）。

```rust
            Event::Request(key) => {
                let write = state.key() != key;
                let next = InlineEdit {
                    phase: key
                        .clone()
                        .map_or(Phase::Idle, |key| Phase::Requested { key }),
                };
                let step = Step::to(next);
                if write { step.with(key) } else { step }
            }
```
- `Event::Ended(key) if …` の腕の本体を `Step::to(InlineEdit { phase: Phase::Idle }).with(None)` にする。
- `Mounted`・`Typed` の腕は変えない（`let mut next = state.clone(); next.phase = …;` のまま compile する）。
- `use_inline_edit` の `machine(..)` の第 1 引数を `InlineEdit { phase: initial.map_or(Phase::Idle, |key| Phase::Requested { key }) }` にし（`.clone()` と `reported: initial,` を消す）、第 3 引数と直前のコメントを次にする。

```rust
        // The machine is the one writer of the request it reports; it writes
        // only a key the binding does not already hold, and the echo of its
        // own write comes back as a request for the state it holds.
        move |_, key| {
            if editing.get_untracked() != key {
                editing.set(key);
            }
        },
```
- 振る舞い：変わらない（`reported` は全ての遷移で `phase.key()` と等しかった。effect の guard は同じ値の再書込みを省くだけ）。`InlineEdit` は数える field が `phase`（phase 1 個）だけになるので marker は無い（HEAD の `reported` の marker は field と一緒に消す）。

**(7) E-6 `table/data.rs` の marker（文言を差し替える）。** 付属書 E-6 の `active` の文「the selection does not follow it」は、`TableFocus::Rows(ListOptions)` で caller が Automatic activation を選んだ場合に事実と違う。次の 4 行で置き換える（`TableRowState`・`TableCellState` の各 field の直前の projection 行。doc comment がある field は doc の上の marker 行を置き換える）。
- `TableRowState.selected`：`    // xtask-allow(state-fields): independent: whether the table selection holds the row; Space, a pointer press and the caller's binding change it, and a multiple selection keeps rows selected while the tab stop is elsewhere`
- `TableRowState.active`：`    // xtask-allow(state-fields): independent: whether the row holds the tab stop (itself under TableFocus::Rows, one of its cells under TableFocus::Cells); arrows between cells and under Manual activation move it without changing the selection`
- `TableCellState.selected`：`    // xtask-allow(state-fields): independent: copied from the cell's row selection; Space selects the row of the focused cell, and arrows leave the selection where it is`
- `TableCellState.active`：`    // xtask-allow(state-fields): independent: whether the cell holds the tab stop, only under TableFocus::Cells; arrows move it between cells of selected and unselected rows alike`
- 型・書き手（`table/handle.rs`、`table/cells.rs`）は変えない。

**(8) X4・X5（FINAL-2）の確認。** IN-2 の後、L4 のファイルに `projection:` は残らない（`MenuItemState` の 2 本と `InlineEdit.reported` は消え、`TableRowState`・`TableCellState` は independent）。上の independent の文はどれも workspace で一意。

**(9) G9（W7）との関係。** IN-2 は `MenuApi::open/close/show`、`InlineEditApi::open`、hook の引数名に触れないので、G9 の改名表の対象を消さない。IN-2 が足す名前（`MenuItemDisplay`、`check()`、`is_expanded()`、`display`）は ARG-2/3/6 の改名の対象ではない（G9 の行への追記は不要）。

**(10) FINAL-1（W7）との関係。** IN-2 は `PartState` の field を新しく読まない・書かない（fallback の `PartState::default()` は HEAD のまま）。`kivact-ui-basic/src/recipes_menu.rs` の `s.input.disabled` は FINAL-1 の対象として残る。

### 所有ファイルの変更
- なし（D5 6 章の行どおり）。注意：行の `menu.rs`・`lib.rs` は `crates/kivact-interaction/src/menu.rs` と `crates/kivact-interaction/src/lib.rs`（L6 UI-4 が持つ `crates/kivact-ui/src/menu.rs` とは別）。作業指示書の IN-2 の行に path を全部書く。

### 完了時の確認（付属書の行を置き換える）
- `cargo check -p kivact-interaction -p kivact-ui -p kivact-ui-basic -p kivact-ui-soft -p kivact-ui-pop -p kivact-ui-game -p kivact-ui-creative --all-targets`
- `cargo check -p kivact-public-api-tests --tests`
- `cargo clippy -p kivact-interaction --all-targets -- -D warnings`（`Level::item` の行数）
- `cargo xtask check --no-write --only state-fields --crate kivact-interaction`
- `dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- interaction_menu apg_menu ui_field`（`apg_menu` は `apg_menu_button`・`apg_menubar` も含む。`ui_field` は inline rename）
- `dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-creative --test consumer -- dock`（dock の rename）

---

## IN-3（D5 付録 P5-F F-U4、D5 3.5 の calendar/table の grid_state、W6 lane L4）

### 判定：修正あり（文言 1 本の差替えと、取りこぼし 1 件の追加）

### 修正
- 次の marker 行を置き換える（各 field の直前の行。全て `crates/kivact-interaction/src/`）。
  - `combobox/handle.rs` `ComboboxApi.content_state`：`    // xtask-allow(state-fields): independent: set by ComboboxApi::content when the list surface attaches; the input part attaches separately through ComboboxApi::input, before the surface first mounts`（`input_state` の marker は変えない。D8 で確認済み）
  - `disclosure.rs` `DisclosureApi.trigger_state`：**付属書の文を採らず**次にする（付属書の「an always-open disclosure has content and no trigger」は誤り。`kivact-ui` の `Disclosure::always_open` も trigger を付けて disabled にするだけ）：`    // xtask-allow(state-fields): independent: set by DisclosureApi::trigger when the kit attaches a trigger; the content part attaches through its own call, so a disclosure may have either part without the other`
  - `disclosure.rs` `DisclosureApi.content_state`：`    // xtask-allow(state-fields): independent: set by DisclosureApi::content when the content part mounts; before the first opening a disclosure has its trigger and no content`
  - `popup/api.rs` `PopupApi.content_state`：`    // xtask-allow(state-fields): independent: set by PopupApi::content_with when the surface mounts, only once the popup first opens; the trigger has attached by then or is absent`
  - `popup/api.rs` `PopupApi.trigger_state`：`    // xtask-allow(state-fields): independent: set by PopupApi::base_trigger at the trigger's mount; until the popup first opens there is a trigger and no content`
  - `tooltip/api.rs` `TooltipApi.trigger_state`：`    // xtask-allow(state-fields): independent: set by TooltipApi::trigger at mount; the hint content mounts only once the tooltip first shows`
  - `tooltip/api.rs` `TooltipApi.content_state`：`    // xtask-allow(state-fields): independent: set by TooltipApi::content when the hint mounts after the show delay, after the trigger attached`
  - `calendar/state.rs` `State.grid_state`：`    // xtask-allow(state-fields): independent: the day grid part's state, set by calendar/handle.rs when the kit attaches the grid and absent before; the calendar's other state is made at setup and is valid both before and after the grid attaches`
  - `table/state.rs` `State.grid_state`：`    // xtask-allow(state-fields): independent: the table grid part's state, set by table/handle.rs when the kit attaches the grid and absent before; the label and multi-sort options beside it are fixed at setup`
- **取りこぼしの追加**：`crates/kivact-interaction/src/calendar/data.rs` の `pub struct CalendarDayState` の `disabled` の直前の行 `// xtask-allow(state-fields): projection: readonly snapshot of the calendar policy for recipes, written only by the cell part` を**削除する**（marker 行だけ。doc comment と field は残す）。
  - 理由：付属書 P5-E 4 節は `CalendarDayState` を「snapshot の書直し」の対象に挙げ、E-U6 で「OPEN 担当の unit に統合」としたが、D5 6 章のどの unit の所有にも `calendar/data.rs` が無く、D8 付録 B にも行が無い（D8 は P5 の担当として除いた）。残すと FINAL-2 の X4 で「written only by the cell part」の `the` を関数名と読み「does not exist」の error になる。
  - 削除でよい理由：数える field は `disabled` 1 個だけ（`selection: DaySelection`・`span: CalendarSpan` は phase の名前でない enum、`input` は struct）。組合せが無いので marker は不要で、削除しても `unused marker` にも X3 の error にもならない（`total = 1`）。書き手は `calendar/handle.rs` の 3 箇所（fallback・cell・mount）で、`projection` 自体が事実と違う。
- lint の確認（静的）：`ComboboxApi`・`DisclosureApi`・`PopupApi`・`TooltipApi` は数える field が 2（全て marker 付き）。calendar・table の `State` は数える field が 1（`grid_state`）で、marker は X5 の重複を消すために文を変えるだけ（使われるので `unused marker` にならない）。
- X5：HEAD で重複していた 3 種（「the observation of the attached content part …」6 ファイル、「… trigger part …」3 ファイル、「… grid part …」2 ファイル）は、IN-1（select）・IN-2（menu/level）・IN-3（残り）で全て別の文になる。L4 の後、kivact-interaction にファイルをまたぐ同一の independent 文は残らない（HEAD の全 marker を集計して確認）。
- 振る舞い・試験：変化なし。

### 所有ファイルの変更
- `crates/kivact-interaction/src/calendar/data.rs` を IN-3 の所有に足す（marker 1 行の削除だけ。L4 以外の lane・W5・W7 の unit はこのファイルを持たない）。

### 完了時の確認
- `cargo check -p kivact-interaction --all-targets`；`cargo xtask check --no-write --only state-fields --crate kivact-interaction`。

---

## P8-IN（D8 6 章 P8-IN、付録 B の unit 列が P8-IN の 16 行、W6 lane L4）

### 判定：変更なし（手順の補足だけ）

### 修正（補足）
- 16 行は全て HEAD に同じ owner・field で残っている（`DismissOn.escape`・`outside`、`Step::Stop.owns_edit`・`update`、`use_menubar::focus`、`Header.sortable`・`min_width`、`ColumnDef.width`・`min_width`・`compare`・`matches`・`matches_value`、`use_data_table_model::typed_queries`、`Effect::Dismiss.previous`・`callback`、`Toast.duration`）。どれも数える field・local で、marker は使われる。付録 B の「新しい marker 行」の全文で、各 field・local の直前の marker 行を置き換える（doc comment がある field は doc の上の marker 行）。
- `table/data.rs` は IN-2 の後（同じ lane の順で満たす）。`Header` の 2 行だけを変え、IN-2 が書いた `TableRowState`・`TableCellState` の行には触れない。
- X5：16 行の independent 文（`owns_edit`、`update`、`focus`、`previous`）は workspace で一意。projection は無い。

### 所有ファイルの変更
- なし。

### 完了時の確認
- D8 6 章の行のとおり（`cargo check -p kivact-interaction`；`cargo xtask check --no-write --only state-fields --crate kivact-interaction`）。

---

## L4 の全体

- **lane 内の順序**：IN-1 → IN-2 → IN-3 → P8-IN のままでよい。ファイルの重なりは `table/data.rs`（IN-2 → P8-IN）だけで、同じ lane の順で満たす。
- **他 lane・他 wave との重なり**：
  - L5・L6 とファイルは重ならない（L4 は `crates/kivact-interaction/src/**` と `crates/kivact-ui-basic/src/recipes_menu.rs`・`crates/kivact-ui-soft/src/recipes_menu.rs`。L5・L6 の所有に `recipes_menu.rs` は無い。`menu.rs`・`combobox/*`・`calendar/*` の名前は kivact-ui 側と同名なので、作業指示書には crate を含む path で書く）。
  - W5 G7：`crates/kivact-interaction/src/menu/handle.rs` は G7 →（W6）IN-2 の順で、hunk が重ならない（上の IN-2 (3)）。
  - W7 FINAL-1：L4 は `PartState` の field の読み書きを新しく足さない。`kivact-ui-basic/src/recipes_menu.rs` の `s.input.disabled` は FINAL-1 が直す。
  - W7 G9：L4 は G9 の改名対象を消さず、L4 が足す名前は G9 の対象外。**ただし G9 の改名表 1（`MenuApi::show(factory)` → `content(factory)`、`CommandPaletteApi::show(factory)` → `content(factory)`）は、両方の型に既に `pub fn content(self) -> Part<PopupState>`（surface の part。`crates/kivact-interaction/src/menu/handle.rs`・`palette/handle.rs`）があるため、同名の inherent method の重複で compile error（E0592）になる**。L4 の範囲外（G9 の担当の監査）だが、親に申し送る。推奨：content を mount する method の新しい名前を `mount(factory)` にする（doc の「Mount the caller's content …」と一致し、`content()` が Popup・Disclosure・Tooltip・Combobox と揃った「surface の part」の意味を保つ）。DEC-18 の意図（`show` を ARG-6 の命令に空ける）はこれで満たせる。
- **公開の recipe state の命名（DEC-14・15）**：L4 の確定形は「独立な事実は pub field（`selected`、`active`、`destructive`、`input`）、排他・導出は accessor（`check()`、`is_expanded()`）と種類の enum の field（`display: MenuItemDisplay`）」。bool の accessor は `is_*`、値は名詞（`check()` は `MenuEntry::check()` と同じ名前・型）で、D8 の `CheckableState`（`is_checked()`・`checked()`・`switch_text()`）、D5 の `PartState`（`is_pressed()`・`is_focused()`）と揃う。他 lane との整合の参考：種類を表す enum の field 名と型名が `CheckableState.control: CheckControl`（L5 P8-CK）、`CardState.action: CardActionState`（L5 UI-1。型名が `State` で終わるので lint の phase に数えられる）、`MenuItemState.display: MenuItemDisplay`（L4）と 3 通りある。L4 は `MenuItemKind` と並ぶ名前として `MenuItemDisplay` を保つ。→ R6 の統合で確定：冒頭の「公開の recipe state の命名規則」により、L5 UI-1 の型は `CardAction` に改名した（field 名は型名から owner の接頭辞を除いたもの。3 つとも同じ規則に合う）。
- **人間にしか決められない項目**：なし。


---

# R6 監査：W6 lane L5（UI-1 → UI-2 → UI-3 → P8-CK → P8-UI5）

対象 HEAD 2a3b06b8。行番号は HEAD のもの（`data_table/*`・`values.rs`・`toggle.rs` は 73c36c7c と同じ行。`button.rs` は M10 で `Button::finish` の中の行がずれた）。実装者は名前で探すこと。

## 0. L5 共通の確定事項（全 unit の「本書の追加の指示」の先頭に入れる）

### 0.1 公開の recipe state の accessor の命名規則（DEC-14・DEC-15 の統一。L5 の全 unit に適用）

- 型（enum）に置き換えた公開 field は、**pub の enum field ＋ accessor** の形にする（FINAL-1 の `PartState { availability, .. }` と `is_pressed()`、L6 F-T1 の `BreadcrumbItemState::is_current()`・`input()`、L4 E-2 の `MenuItemState::check()`・`is_expanded()` と同じ形）。
- **bool を返す accessor は必ず `is_` か `has_` で始める**。bool 以外を返す accessor は名詞（`checked()`、`expansion()`、`selection()`、`input()`、`switch_text()`）。
- 旧 field の情報は、同じ名前（bool は `is_`／`has_` を付けた名前）の accessor で読めるようにする（外部の theme 作者の直し方が「`.x` → `.x()`／`.is_x()`」の一通りになる）。
- この規則による付属書からの変更（確定）：
  1. `ButtonState::icon_only()`・`ToggleButtonState::icon_only()`（D5 E-1）→ **`is_icon_only()`**。builder の `ToggleGroup::icon_only(..)`（公開の setter）とも名前が分かれる。
  2. `CodeBlockState.header_content`（D5 F-T2）→ **`has_header_content`**（兄弟の `has_footer`・CardState の `has_media` と揃える）。
  3. `AlertState` は D5 F-T3 の `details: Option<Expansion>` ではなく **`content: Option<Expansion>`**（builder `Alert::content` と同じ名前）とし、旧 field と同名の accessor **`has_content()`・`expansion()`** を足す。
  4. CardState の公開 enum は D5 の `CardActionState` ではなく **`CardAction`**（`CheckControl`・`CodeCollapse` と同じく `State` を付けない。`State` で終わる enum は state-fields の phase に数えられ、名前でも snapshot と紛らわしい）。衝突していた `card.rs` の private な builder の enum `CardAction` は **`CardKind`** に改名する（private なので公開 API に影響なし）。
  5. `DataTableCheckState`（D5 F-T5 の enum）に旧 field と同じ情報の accessor **`is_header()`・`checked()`** を足す。
- 変えないもの：`ToggleButtonState.pressed`（bool の field。ARIA の `aria-pressed` と `toggle_button(label, pressed)` の引数名に合わせた名前で、型を変えないので accessor 化しない）。`CheckableState` の accessor（`checked()`・`is_checked()`・`is_mixed()`・`switch_text()`・`switch_drag()`）は D8 3.1 のまま（規則に合っている）。

### 0.2 lane 内の所有の組み直し（ファイルの重なりは L5 の中だけ。順序で解決）

| unit | 付属書（D5 6 章）の所有から外すもの | 足すもの |
|---|---|---|
| UI-1 | `data_table/types.rs`・`data_table/builder.rs`・`data_table/runtime.rs`（P8-UI5 へ）、`date_input/view.rs`・`date_range_input.rs`（E-8 は行わない） | `crates/kivact-ui/src/alert.rs`（AlertState の書き手 2 行だけ） |
| UI-2 | — | — |
| UI-3 | `toggle.rs`、`radio_group.rs`、`crates/kivact-ui-soft/src/indicator.rs`、`crates/kivact-ui-soft/src/recipes_controls_choice.rs`（F-T4 だけの所有だった。P8-CK が持つ） | — |
| P8-CK | — | — |
| P8-UI5 | — | `crates/kivact-ui/src/data_table/builder.rs` は**足さない**（E-9 は行わない。下記）。`data_table/types.rs` の E-6（DataTableContext）と P5-F (b) の `tree_expand_all` の marker を UI-1 から引き取る |

---

## UI-1（D5 6 章 UI-1：付録 P5-E 4 節 CardState、P5-F F-T3・2 章 component_values.rs、P5-E E-6 の DataTableContext 行、E-8、E-9。W6 lane L5）

### 判定：修正あり

### 修正

**(a) 行わないもの（確定）**

- **E-8（date_input／date_range の外部値を `feed` に）は行わない。** 理由：今の `effect(move || state.send(Event::External(value.get())))` の最初の実行は、`Event::External` の遷移が出す `Effect::Validity(..)` で、caller の `.valid(..)` binding に mount 時の有効性を書く（`date_input/machine.rs` の `Event::External(value) => Step::to(..).with(Effect::Validity(true))`、`date_range_draft.rs` の `Effect::Validity(value.is_none_or(..))`）。`feed`（`effect_on`）は最初の値を送らないので、この mount 時の公開が消え、`DateInput::valid` の doc「Publish whether the current typed draft is a valid optional date」に反する。state-fields とも無関係（暗黙の状態ではない）。`date_input/view.rs`・`date_range_input.rs` は触らない。
- **E-9（DataTable builder の `page`・`has_more` を enum に）は行わない。** 理由：(1) 付録の `Local(PagePlan, Option<page_sizes>)` は、`.page_sizes(..)` が `remote_page` でも効く（`runtime.rs` の `paging()` が remote でも size の binding で番号の offset を計算する）ことと、builder の呼出し順が自由（`.page_sizes(..)` を `.page(..)` より先に呼べる）ことの両方を壊す。(2) 読み手は付録の「一か所」ではなく `runtime.rs` の 4 箇所（`mount` の `table_paging(&plan, table.has_more)`、`model_plan` の `table.has_more.is_none()`、`navigation_plan` の `table.has_more.is_some()`、`row_indexes` の `table.has_more`）。(3) D8 3.3 の規則「builder の設定どうしの依存は WORDING にとどめる」（同じ形の `sort`／`multi_sort` の排他も WORDING）に揃える。代わりに P8-UI5 が `page`・`has_more`・`page_sizes` の marker を事実どおりの文にする（P8-UI5 の節）。
- E-6 の `data_table/types.rs` の DataTableContext の 4 行と、P5-F 2 章 (b) の `types.rs:132`（`tree_expand_all`）は **P8-UI5 が行う**（types.rs を一つの unit にまとめる）。

**(b) `crates/kivact-ui/src/component_values.rs`**

`AlertState`（今の `pub struct AlertState { tone, expansion, dismissible, has_icon, has_actions, has_description, has_content }` と marker 5 本）を次で置き換える：

```rust
/// Alert meaning and disclosure state supplied to recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlertState {
    pub tone: Tone,
    // xtask-allow(state-fields): independent: Alert::content fills the content slot (Some) and Alert::collapsible with the open binding choose its Expansion; it combines with any icon, actions, description or dismiss button
    /// The content slot ([`Alert::content`](super::Alert::content)): `None`
    /// without content, otherwise whether it can collapse and is open.
    pub content: Option<Expansion>,
    // xtask-allow(state-fields): independent: Alert::dismissible shows the dismiss button, with or without content, icon or actions
    pub dismissible: bool,
    // xtask-allow(state-fields): independent: Alert::icon fills the icon slot of a dismissible or fixed alert alike
    pub has_icon: bool,
    // xtask-allow(state-fields): independent: Alert::actions fills the action slot, beside or without a dismiss button
    pub has_actions: bool,
    // xtask-allow(state-fields): independent: Alert::description adds the description line, with or without content below it
    pub has_description: bool,
}

impl AlertState {
    /// Whether the alert has a content slot ([`Alert::content`](super::Alert::content)),
    /// collapsible or not.
    pub fn has_content(&self) -> bool {
        self.content.is_some()
    }
    /// Whether the content can collapse and, if so, whether it is open;
    /// [`NotExpandable`](Expansion::NotExpandable) without content or when
    /// it cannot collapse.
    pub fn expansion(&self) -> Expansion {
        self.content.unwrap_or(Expansion::NotExpandable)
    }
}
```

`CardState`（今の 6 field と marker 6 本）を次で置き換え、`Selectable` の定義の後に `CardAction` を足す：

```rust
/// What a card does when it is activated, with the actual input of the part
/// that does it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CardAction {
    /// A plain surface; nothing activates it.
    Plain,
    /// Pressing activates it ([`Card::on_press`](super::Card::on_press)).
    Press {
        /// The actual input state of the card's button part.
        input: kivact_core::custom::PartState,
    },
    /// It is a check box ([`Card::selected`](super::Card::selected)).
    Select {
        /// The actual input state of the card's check box part.
        input: kivact_core::custom::PartState,
        /// Whether the card is selected.
        selected: bool,
    },
}

/// Card semantics supplied to recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardState {
    /// What activating the card does; `match` it, or read
    /// [`input`](Self::input), [`is_interactive`](Self::is_interactive) and
    /// [`selection`](Self::selection).
    pub action: CardAction,
    // xtask-allow(state-fields): independent: the caller's Card::disabled value applies to any action
    pub disabled: bool,
    // xtask-allow(state-fields): independent: media and footer are separate optional slots; all four combinations render
    pub has_media: bool,
    // xtask-allow(state-fields): independent: media and footer are separate optional slots; all four combinations render
    pub has_footer: bool,
}

impl CardState {
    /// The actual input state of the card's button or check box part;
    /// `None` for a plain card.
    pub fn input(&self) -> Option<kivact_core::custom::PartState> {
        match self.action {
            CardAction::Plain => None,
            CardAction::Press { input } | CardAction::Select { input, .. } => Some(input),
        }
    }
    /// Whether the card can be pressed or selected.
    pub fn is_interactive(&self) -> bool {
        !matches!(self.action, CardAction::Plain)
    }
    /// Whether the card can be selected and, if so, whether it is.
    pub fn selection(&self) -> Selectable {
        match self.action {
            CardAction::Select { selected, .. } => Selectable::new(true, selected),
            CardAction::Plain | CardAction::Press { .. } => Selectable::NotSelectable,
        }
    }
}
```

`InputGroupState`・`InputGroupMemberState` の marker 9 本は D5 付録 P5-F 2 章「crates/kivact-ui/src/component_values.rs」の表の文のとおり（一字一句。変更なし）。

lint：`AlertState` は数える field が `content`・`dismissible`・`has_icon`・`has_actions`・`has_description` の 5 個で全部に marker。`CardAction` は名前が phase の接尾辞（State 等）でないので `CardState.action` は数えない。`CardState` は `disabled`・`has_media`・`has_footer` の 3 個に marker。`CardAction::Select` の数える field は `selected` 1 個なので marker なし。X5：上の independent の文はどれも他のファイルに無い（同じファイルの has_media・has_footer の共有は X5 が許す）。

**(c) `crates/kivact-ui/src/lib.rs`**：`pub use component_values::{AlertState, CardState, ..}` の列に `CardAction` を足す（アルファベット順の位置：`AlertState, CardAction, CardState, ..`）。

**(d) `crates/kivact-ui/src/card.rs`**

- private の builder の enum `enum CardAction { Plain, Press, Select(Binding<bool>) }` を `enum CardKind { .. }` に改名する（定義・`Card.action` の型・`card()`・`on_press`・`selected` の中の全ての `CardAction::` を `CardKind::` に）。
- `card_target` の直前に次を足す：

```rust
/// The behavior a card's root got, with the live input state of its part.
#[derive(Clone, Copy)]
enum CardInput {
    Plain,
    Press(Value<kivact_core::custom::PartState>),
    Select(Value<kivact_core::custom::PartState>, Binding<bool>),
}
```

- `card_target` の戻り値を `(Node, CardInput)` にし、腕を `CardKind::Plain => (root, CardInput::Plain)`、Press の腕の最後を `(root.part(part), CardInput::Press(input))`、Select の腕の最後の `Some(input),` を `CardInput::Select(input, selected),`（`selected` は `CardKind::Select(selected)` の binding。`use_checkbox(selected)` に渡した後も Copy なので使える）にする。
- `into_view` の `let selected = match self.action { .. };` と `let interactive = ..;` を消し、`let (root, input) = card_target(..)` を `let (root, input) = card_target(self.root, self.action, disabled);`（名前は同じ）のままにして、`state` を次にする：

```rust
        let state = move || CardState {
            action: match input {
                CardInput::Plain => CardAction::Plain,
                CardInput::Press(input) => CardAction::Press { input: input.get() },
                CardInput::Select(input, selected) => CardAction::Select {
                    input: input.get(),
                    selected: selected.get(),
                },
            },
            disabled: disabled.get(),
            has_media,
            has_footer,
        };
```

- import：`use crate::component_values::{CardAction, CardState};`（`Selectable` は使わなくなるので消す）。

**(e) 読み手**

- `crates/kivact-ui/src/look_feedback.rs` の `card_root`：`s.selection == Selectable::Selected` → `s.selection() == Selectable::Selected`、`if s.interactive {` → `if s.is_interactive() {`。
- `crates/kivact-ui-soft/src/card.rs`：`root` の中の `s.selection`（3 箇所：`let selected = ..`、`if s.selection != Selectable::NotSelectable`、`} else if s.selection == Selectable::NotSelectable`）を `s.selection()` に、`if s.interactive {` を `if s.is_interactive() {` に。`overlay` の `s\n        .input\n        .filter(..)` を `s.input().filter(..)` に（**付録の読み手一覧に無い。追加**。中の `input.disabled`・`input.pressed`・`input.hovered` は FINAL-1 が直すのでここでは変えない）。
- `crates/kivact-ui-soft/src/banner.rs`：`header` の `s.has_content && s.expansion != Expansion::Collapsed` → `s.has_content() && s.expansion() != Expansion::Collapsed`；`s.expansion != Expansion::NotExpandable` → `s.expansion() != Expansion::NotExpandable`；`control` の `let expanded = s.expansion == Expansion::Expanded;` → `let expanded = s.expansion() == Expansion::Expanded;`（**3 箇所目は付録に無い。追加**）。
- `crates/kivact-ui/src/alert.rs` の書き手（`Alert::into_view` の `let state = move || AlertState { .. }`）：`expansion: Expansion::new(has_details && collapsible.get(), details.get()),` と `has_content: has_details,` の 2 行を消し、`content: has_details.then(|| Expansion::new(collapsible.get(), details.get())),` を `tone` の次に置く。（付録は UI-2 に置いていたが、型を変える unit で直さないと `cargo check -p kivact-ui` が通らないので UI-1 が行う。UI-2 はこの closure を変えない。）
- kit の basic・pop・game の alert recipe は `tone`・`variant` だけを読む（Grep 済み）ので変更なし。

**(f) 文書**

- `docs/for-kivact-user/en/how-to-guide/feedback.md`：
  - 「Card recipes receive `input: Option<PartState>` from the actual button or checkbox part; an ordinary card receives `None`.」を「Card recipes read `state.input()`, the `PartState` of the actual button or checkbox part (`None` for an ordinary card); `state.action` tells a plain, pressable and selectable card apart.」に。
  - 「`AlertState.has_content` reports whether the details slot exists, independent of whether the alert is collapsible.」を「`AlertState::has_content()` reports whether the content slot exists, independent of whether the alert is collapsible; `AlertState::expansion()` tells whether it can collapse and is open.」に。
- `docs/for-kivact-user/jp/how-to-guide/feedback.md`：
  - 「Card の recipe には実際の button または checkbox の Part から `input: Option<PartState>` が渡されます。通常の card は `None` です。」を「Card の recipe は `state.input()` で実際の button または checkbox の Part の `PartState` を読みます（通常の card は `None`）。`state.action` で通常・押せる・選べる card を区別します。」に。
  - 「`AlertState.has_content` は、開閉できるかどうかとは独立に、詳細の slot が実際にあるかを表します。」を「`AlertState::has_content()` は、開閉できるかどうかとは独立に、内容の slot が実際にあるかを表します。開閉できるか・開いているかは `AlertState::expansion()` で読みます。」に。

**(g) 完了時の確認（付属書の行の代わり）**

`cargo check -p kivact-ui -p kivact-ui-soft -p kivact-ui-basic -p kivact-ui-pop -p kivact-ui-game --all-targets`；`cargo clippy -p kivact-ui --all-targets -- -D warnings`；`cargo xtask check --no-write --only state-fields --crate kivact-ui`（kivact-ui-soft も）；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- components feedback:: apg_alert`。

### 所有ファイルの変更

`crates/kivact-ui/src/component_values.rs`、`card.rs`、`look_feedback.rs`、`alert.rs`（AlertState の書き手の 2 行だけ）、`lib.rs`、`crates/kivact-ui-soft/src/card.rs`、`crates/kivact-ui-soft/src/banner.rs`、`docs/for-kivact-user/{en,jp}/how-to-guide/feedback.md`。外すもの：`data_table/types.rs`・`builder.rs`・`runtime.rs`（P8-UI5 へ）、`date_input/view.rs`・`date_range_input.rs`（E-8 は行わない）。

### wave・lane・順序の変更

なし（L5 の先頭）。

---

## UI-2（D5 付録 P5-C C1・C2・C3、W6 lane L5）

### 判定：修正あり（小）

### 修正

- **C1**（`crates/kivact-ui/src/alert.rs`）：付録 P5-C C1「3. 設計」のコードのとおり（HEAD のコードと一致を確認：`LocalBannerState`、`BannerState::{Local, Shared}`、`BannerEvent::{Visibility, ExternalFocus, Dismiss}`、`BannerMachine { visible }`、`phase.feed(..)`、`ExternalFocus` を送る `effect`）。付録の「削除」に加えて、`alert.rs` の import で使わなくなったもの（clippy の `unused_imports` が示すもの）を消す。`Binding`・`signal`・`IntoValue` は他の場所で使っているので残る。**F-T3 の書き手（AlertState の closure）は UI-1 が済ませている。UI-2 は触らない**。
- lint の確認（確定）：`BannerEvent::Observed { shown, focused, within }` は enum 名が role 接尾辞 `Event` なので数えない。`BannerState::Shown`・`BannerEffect::RequestDismiss` は数える field が 1 個。`BannerMachine.ownership` は `Ownership`（phase 名でない enum）で数えない。alert.rs の marker は 0 本になる（`:26` の marker は消す）。
- 追加の E2E（`dev/tests/kivact-public-api/tests/e2e/feedback.rs`、`controlled_alert_keeps_refused_visibility_and_notifies_after_acceptance` の直後）は付録のコードのとおり（`render_with_handle`・`options()`・`ui::button`・`.dismissible(..)`・`.dismiss_label(..)`・`.open(..)` は実在を確認）。
- **C2**（`crates/kivact-ui/src/calendar_values.rs`）：付録のとおり 3 行の marker を置き換える（一字一句）。
- **C3**（`crates/kivact-ui/src/data_table/filters.rs`）：付録のとおり（HEAD のコードと一致：`FilterDraft::Valid { text, value, sent }`、手書き `PartialEq`、`fn sent()`、`type Effect = (bool, Option<FilterValue>)`、`text_editor` の `Sent::new()`）。`use kivact_core::custom::Sent;` を消す（このファイルの他に使う所なし）。追加の consumer 試験（`crates/kivact-ui-soft/tests/consumer/typed_table_filters.rs` の末尾）は付録のコードのとおり（`Locator::value() -> Option<String>`、`type_text`、`column()`、`FilterUi::InlineCompact` は実在。HEAD でも通る回帰の固定）。
- D5 6 章の「F-T3 の書き手」は UI-1 へ移したので、UI-2 では行わない。

完了時の確認：付属書（D5 6 章 UI-2）の行のとおり。

### 所有ファイルの変更

`crates/kivact-ui/src/alert.rs` の AlertState の closure は UI-1 が先に変える（同じ lane の順なので衝突なし）。他は付属書のとおり。

### wave・lane・順序の変更

なし。

---

## UI-3（D5 付録 P5-E E-1、P5-F F-T2・F-T5・2 章 values.rs（Dialog・SelectTrigger・DataTableRow）、lib.rs の export。F-T4 は行わない。W6 lane L5）

### 判定：修正あり

### 修正

**(a) 範囲（確定）**：F-T4（`SwitchFacts`）と、`values.rs` の `CheckableState`（型・marker 3 本・`impl PartialEq`）には**一切触れない**（P8-CK が行う）。`toggle.rs`、`radio_group.rs`、soft `indicator.rs`、soft `recipes_controls_choice.rs`、game `recipes_controls.rs` は UI-3 では変更しない。

**(b) E-1（M10 の後の button.rs に対する確定版）**

`crates/kivact-ui/src/values.rs` の `ButtonState`・`ToggleButtonState` を次にする（doc は今の文を保ち、`icon_only` の doc を `label_hidden` 用に書き換える）：

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonState {
    /// (今の input の doc をそのまま)
    pub input: PartState,
    pub size: ControlSize,
    // xtask-allow(state-fields): independent: busy comes from the caller's busy value and is valid with any icon or label layout
    pub busy: bool,
    // xtask-allow(state-fields): option: None for a plain button, Some(open) when a popup or the caller publishes an expanded state
    /// (今の expanded の doc をそのまま)
    pub expanded: Option<bool>,
    // xtask-allow(state-fields): independent: the leading icon, the trailing icon and the hidden label are set by separate builder calls and all eight combinations are drawn
    /// Whether the button shows a leading icon ([`Button::icon`](super::Button::icon)).
    pub leading_icon: bool,
    // xtask-allow(state-fields): independent: the leading icon, the trailing icon and the hidden label are set by separate builder calls and all eight combinations are drawn
    /// Whether the button shows a trailing icon
    /// ([`Button::trailing`](super::Button::trailing)).
    pub trailing_icon: bool,
    // xtask-allow(state-fields): independent: the leading icon, the trailing icon and the hidden label are set by separate builder calls and all eight combinations are drawn
    /// Whether the label is hidden ([`Button::hide_label`](super::Button::hide_label));
    /// it stays the accessible name.
    pub label_hidden: bool,
}

impl ButtonState {
    /// Whether only the leading icon shows: the button has one and its
    /// label is hidden. A look draws it as a square of the button's height.
    pub fn is_icon_only(&self) -> bool {
        self.leading_icon && self.label_hidden
    }

    /// (今の new の doc をそのまま)
    pub fn new(input: PartState, size: ControlSize) -> Self {
        ButtonState {
            input,
            size,
            busy: false,
            expanded: None,
            leading_icon: false,
            trailing_icon: false,
            label_hidden: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleButtonState {
    /// The actual input state of the toggle.
    pub input: PartState,
    // xtask-allow(state-fields): independent: the pressed state follows the toggle and is valid with any icon or label layout
    /// Whether it is pressed (on).
    pub pressed: bool,
    /// The size the caller chose with `.size(..)` (default `Medium`).
    pub size: ControlSize,
    // xtask-allow(state-fields): independent: the leading icon and the hidden label are set separately; all four combinations are drawn
    /// Whether the toggle shows a leading icon ([`ToggleButton::icon`](super::ToggleButton::icon)).
    pub leading_icon: bool,
    // xtask-allow(state-fields): independent: the leading icon and the hidden label are set separately; all four combinations are drawn
    /// Whether the label is hidden ([`ToggleButton::hide_label`](super::ToggleButton::hide_label),
    /// or [`ToggleGroup::icon_only`](super::ToggleGroup::icon_only) for a group's item).
    pub label_hidden: bool,
}

impl ToggleButtonState {
    /// Whether only the leading icon shows: the toggle has one and its label
    /// is hidden. A look draws it as a square of the toggle's height.
    pub fn is_icon_only(&self) -> bool {
        self.leading_icon && self.label_hidden
    }
}
```

（`ToggleGroup` が crate root で公開されていることを確認すること。`super::ToggleGroup` の link が解決しなければ、その link の括弧を `(`toggle_group(..).icon_only(..)`)` の平文にする。）

書き手：
- `crates/kivact-ui/src/button.rs` の `Button::finish`（M10 の後の名前。HEAD で `let state = memo(move || ButtonState {`）：`icon_only: leading_icon && hide_label.is_some_and(|hide| hide.get()),` → `label_hidden: hide_label.is_some_and(|hide| hide.get()),`。同じファイルの `hide_label` の doc の `[`ButtonState::icon_only`]` → `[`ButtonState::is_icon_only`]`。M10 が入れた `if busy.is_some() || disabled.is_some()` の分岐と `busy_parts` には触れない。
- `crates/kivact-ui/src/toggle_button.rs`：`icon_only: leading_icon && hide.is_some_and(|hide| hide.get()),` → `label_hidden: hide.is_some_and(|hide| hide.get()),`。`hide_label` の doc の `[`ToggleButtonState::icon_only`]` → `[`ToggleButtonState::is_icon_only`]`。
- `crates/kivact-ui/src/toggle_group.rs`：`icon_only: icon_only.get(),` → `label_hidden: icon_only.get(),`（builder の `icon_only` field・`.icon_only(..)` の setter・`item_content` は変えない）。
- `crates/kivact-ui-creative/src/recipes.rs` の `toggle_button`：`state.icon_only = s.icon_only;` → `state.label_hidden = s.label_hidden;`。

読み手（`X.icon_only` → `X.is_icon_only()`）：`crates/kivact-ui/src/look.rs` の 2 箇所（`button` と `toggle_button` の `let s = if state.icon_only {`）、`crates/kivact-ui-basic/src/recipes.rs`（`let s = if state.icon_only {`）、`crates/kivact-ui-pop/src/recipes.rs`（`if !state.icon_only {`）、`crates/kivact-ui-soft/src/recipes_controls.rs` の 2 箇所（`action(&t, s.size, s.icon_only)`）、`crates/kivact-ui-creative/src/recipes.rs`（`if s.icon_only {`）。`segmented_control.rs`・`top_nav.rs`・basic `sidebar` の `icon_only` は別物で変えない。

追加の E2E（`dev/tests/kivact-public-api/tests/e2e/ui_toolbar.rs` の末尾。既存の import で足りる）：

```rust
#[test]
fn toggle_group_icon_only_keeps_iconless_items_out_of_the_square_look() {
    let (screen, ()) = mount(|| {
        (
            (),
            toggle_group(
                "Tools",
                vec![
                    ListItem::new("pen", "Pen").with_icon("dot"),
                    ListItem::new("text", "Text"),
                ],
                signal(None::<&'static str>),
            )
            .icon_only(true),
        )
    });
    let pen = screen.get_by_role(Role::Button, "Pen").rect().expect("laid out");
    assert!((pen.width - pen.height).abs() <= 0.5, "{pen:?}");
    let text = screen.get_by_role(Role::Button, "Text").rect().expect("laid out");
    assert!((text.width - text.height).abs() > 0.5, "{text:?}");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```

（既定の theme では Text は padding 12×2＋border 2 = 26 px 幅、高さ 28 px。HEAD では 28×28 で失敗する＝既存欠陥の再現。）

**(c) F-T2**：付録のコードのとおり。ただし field 名 `header_content` を **`has_header_content`** にする（0.1 の 2）。marker は `// xtask-allow(state-fields): independent: CodeBlock::title, language, copy_button or end_content fill the header; a collapsible block shows its header even when this is false`（付録の文のまま）。accessor `has_header()` の本体は `self.has_header_content || self.collapse != CodeCollapse::Fixed`。書き手は `crates/kivact-ui/src/code_block.rs` の `fn presentation` の `let state = memo(move || CodeBlockState {`（付録の行番号の位置から関数が変わっている）で、付録のコードの `header_content:` を `has_header_content:` に。読み手：`code_block.rs` の `.hidden(move || !state.get().has_header)` → `has_header()`、`crates/kivact-ui-basic/src/code_block/view.rs` の `s.has_header` → `s.has_header()`。`state.get().wrapped`（code_block.rs）・`numbered`（code_gutter.rs）・`s.wrapped`（look_chat.rs、basic view.rs）は field のまま。`expanded` の読み手は無い（Grep 済み）。

**(d) F-T5**：付録の enum に次の accessor を足す（0.1 の 5）：

```rust
impl DataTableCheckState {
    /// Whether this is the header's box, which checks the page's rows.
    pub fn is_header(&self) -> bool {
        matches!(self, Self::Header(_))
    }
    /// The box's checked state; only the header's is ever Indeterminate.
    pub fn checked(&self) -> CheckedState {
        match *self {
            Self::Header(checked) => checked,
            Self::Row { checked } => CheckedState::from_bool(checked),
        }
    }
}
```

書き手は付録のとおり（`data_table/chrome.rs` の `let state = move || DataTableCheckState {`、`data_table/rows.rs` の `row_check` の `move || DataTableCheckState {`）。`rows.rs` の `use kivact_core::view::{ButtonPress, CheckedState, ..}` から `CheckedState` を消す（他に使わない）。`chrome.rs` は `CheckedState` を使い続ける。

**(e) values.rs の marker**：P5-F 2 章「crates/kivact-ui/src/values.rs（F-T4・F-T5 以外）」の DialogState・SelectTriggerState・DataTableRowState の 6 行を表の文で置き換える。表の「301, 304, 311 CheckableState」の行は**行わない**（P8-CK）。

**(f) lib.rs**：`pub use code_values::CodeBlockState;` → `pub use code_values::{CodeBlockState, CodeCollapse};`。`SwitchFacts` は足さない。

lint（確定）：`ButtonState` の数える field は busy・expanded・leading_icon・trailing_icon・label_hidden の 5 個、`ToggleButtonState` は pressed・leading_icon・label_hidden の 3 個で、全部に marker。`CodeBlockState` は numbered・wrapped・copied・has_header_content・has_footer の 5 個（`collapse: CodeCollapse` は phase 名でない enum で数えない）。`DataTableCheckState` は enum になり、`Row` の数える field は 1 個で行が消える。X5：同じ文の共有は同じファイル（values.rs、code_values.rs）の中だけ。

完了時の確認：`cargo check -p kivact-ui -p kivact-ui-basic -p kivact-ui-pop -p kivact-ui-soft -p kivact-ui-creative --all-targets`；`cargo clippy -p kivact-ui --all-targets -- -D warnings`；state-fields `--crate kivact-ui`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- ui_toolbar interaction_toggle ui_chat_code ui_data_table`。

### 所有ファイルの変更

`crates/kivact-ui/src/values.rs`、`button.rs`、`toggle_button.rs`、`toggle_group.rs`、`look.rs`、`data_table/chrome.rs`、`data_table/rows.rs`、`code_values.rs`、`code_block.rs`、`lib.rs`、`crates/kivact-ui-basic/src/recipes.rs`、`crates/kivact-ui-basic/src/code_block/view.rs`、`crates/kivact-ui-pop/src/recipes.rs`、`crates/kivact-ui-soft/src/recipes_controls.rs`、`crates/kivact-ui-creative/src/recipes.rs`、`dev/tests/kivact-public-api/tests/e2e/ui_toolbar.rs`。D5 6 章の所有から `toggle.rs`、`radio_group.rs`、`crates/kivact-ui-soft/src/recipes_controls_choice.rs`、`crates/kivact-ui-soft/src/indicator.rs` を外す（F-T4 だけの所有だった）。

### wave・lane・順序の変更

なし（UI-1 の後、P8-CK の前）。

---

## P8-CK（D8 3.1・3.2、6 章 P8-CK、W6 lane L5）

### 判定：修正あり（小）

### 修正

- D8 3.1・3.2 のコードと置換表のとおり（HEAD で読み手 39 箇所・書き手 3 箇所・`toggle.rs` の macro の形・`radio_group.rs` の分解の形を確認済み。行番号も HEAD と一致）。UI-3 の後の `values.rs` では `CheckableState` は HEAD の形のまま残っている（UI-3 は触らない）ので、D8 3.1 の置き換えをそのまま行う。
- `crates/kivact-ui/src/lib.rs`：`pub use values::{..}` の列に `CheckControl` を足す（UI-3 が `code_values` の行を変えた後のファイル）。
- `toggle.rs` の macro：`$text:expr` を `$control:ident` に替えたら、`into_view` の `let text: Option<fn(Value<bool>) -> Value<String>> = $text;` と `let text = text.map(..);` の 2 行も消す（D8 の「:103-116 を次にする」に含まれる）。`switch_control` が `switch_text` を呼ぶので `switch_text` は残る。
- `radio_group.rs`：`radio_option` の引数は 7 個（上限ちょうど）になる。
- PartState との境界（FINAL-1）：書き手 3（soft `indicator.rs`）の `input`、`CheckableState.input` はそのまま渡すだけ。`indicator.rs` の `input.disabled |= ..`・`input.focus = ..`・`look.input.disabled`・`state.input.disabled` は**変えない**（FINAL-1 が P5-F 3 章のコードで直す）。
- D8 5.1 の追加 E2E `an_invalid_radio_group_reports_invalid_to_its_option_recipes`（`ui_controls.rs`）は付録のコードのとおり（`mount_with`・`options()`・`sizes()`・`find_by_text`・`query_by_text` は実在。`RadioGroup::item(|item, state| ..)` の `state` は `Value<CheckableState>` で `state.get().invalid` は field のまま）。HEAD では "Small valid" が出て失敗する（既存欠陥の再現）。
- 振る舞いの変化（D8 7 章 #1）：error のある `field` の中の radio group で、各 radio の recipe が `invalid = true` を受け、標準 look は未選択の円を Danger 枠で描く。既存の試験の期待値には当たらない（`radio_group` を error のある `field` に入れる試験・catalog の撮影は Grep で無し）。CP2 の kit catalog 試験で見る。
- 0.1 の命名規則に照らした確認：accessor は `checked()`・`is_checked()`・`is_mixed()`・`switch_text()`・`switch_drag()` で規則どおり。変更なし。

完了時の確認：D8 6 章 P8-CK の行のとおり。

### 所有ファイルの変更

なし（D8 6 章 P8-CK の行のとおり）。UI-3 と重なるファイル（`values.rs`、`look.rs`、`lib.rs`、`crates/kivact-ui-basic/src/recipes.rs`、`crates/kivact-ui-creative/src/recipes.rs`）は UI-3 → P8-CK の順で、変える行は重ならない（UI-3 は icon_only・code・DataTableCheckState、P8-CK は checked・text・switch_drag）。

### wave・lane・順序の変更

なし。

---

## P8-UI5（D8 付録 A-G2 W1・W2、付録 B の unit 列 P8-UI5 の 34 行、W6 lane L5）

### 判定：修正あり

### 修正

- **W1・W2**（`crates/kivact-ui/src/data_table/runtime.rs`）：付録 A-G2 のコードと置換表のとおり（HEAD の行 151〜209・291〜292・307・361・458・506 と一致を確認）。E-9 を行わない（UI-1 の節）ので、`paging()` は HEAD の `table.page`・`table.page_sizes` を読む付録のコードのままでよい。`table.has_more` の読み手（`mount` の `table_paging(&plan, table.has_more)`、`model_plan` の `table.has_more.is_none()`、`navigation_plan` の `table.has_more.is_some()`、`row_indexes` の `let has_more = table.has_more;`）は変えない。
- **付録 B の 34 行**：付録 B の「新しい marker 行」の文をそのまま置く（どれも `option:` で X4・X5 の対象外）。`ModelPlan.page` の行の文は新しい field `paging` の直前に、`ModelPlan.groups` の行の文は新しい field `grouping` の直前に置き、`page_size`・`grouping_key` の marker は field ごと消える。
- **UI-1 から引き取る `data_table/types.rs` の行**（D5 P5-E E-6、P5-F 2 章 (b)）：
  - `DataTableContext` の `selection`・`filter`・`column_filters`・`typed_column_filters` の 4 本：`// xtask-allow(state-fields): option: the caller's binding; None when the caller gave none`。
  - `DataTable.tree_expand_all`：`// xtask-allow(state-fields): option: DataTable::expand_all_control; false leaves the tree column header without the expand/collapse-all control`。
- **付属書のどれも文を決めていない `DataTable` の 8 行**（HEAD は `option: absent means the component's default`。既定値の無い slot なので D8 3.3 の規則で書き直す）：

| field | 新しい marker 行 |
|---|---|
| `tree` | `// xtask-allow(state-fields): option: DataTable::tree parent key and expanded binding; absent keeps the rows flat` |
| `selection` | `// xtask-allow(state-fields): option: the caller's DataTable::selection binding; absent draws no check box column and selects nothing` |
| `row_selectable` | `// xtask-allow(state-fields): option: DataTable::row_selectable; absent lets every row be selected` |
| `sort` | `// xtask-allow(state-fields): option: the caller's DataTable::sort binding; absent sorts in the table's own state (or not at all with on_reorder), and DataTable::multi_sort clears it` |
| `multi_sort` | `// xtask-allow(state-fields): option: the caller's DataTable::multi_sort keys; absent sorts by one column at most, and DataTable::sort clears it` |
| `page` | `// xtask-allow(state-fields): option: DataTable::page or DataTable::remote_page; absent shows every row with no pager` |
| `has_more` | `// xtask-allow(state-fields): option: DataTable::remote_page's has_more, set together with page; absent pages the held rows locally, and DataTable::page clears it` |
| `page_sizes` | `// xtask-allow(state-fields): option: DataTable::page_sizes; absent offers no size picker, and it is unused without page or remote_page` |

- lint（確定）：`PagingPlan` の数える field は `size_choice` 1 個、`GroupingPlan` は 0 個で marker なし。`ModelPlan` は tree・paging・grouping の 3 個に marker。
- `code_block.rs` の `CodeSource.max_height` と `date_input/view.rs` の `EditorInputs` の 2 行は付録 B のとおり（UI-3 が `code_block.rs` を変えた後のファイル。E-8 を行わないので `date_input/view.rs` はこの unit だけが変える）。

完了時の確認：D8 6 章 P8-UI5 の行のとおり。

### 所有ファイルの変更

D8 6 章の行のとおり（`code_block.rs`、`data_table/rows.rs`、`data_table/runtime.rs`、`data_table/types.rs`、`date_input/view.rs`）。`data_table/builder.rs` は持たない（E-9 を行わない）。

### wave・lane・順序の変更

なし（L5 の最後）。

---

## L5 の全体

- **lane 内の順序**：UI-1 → UI-2 → UI-3 → P8-CK → P8-UI5（変更なし）。L5 の中で同じファイルを持つ組は、`lib.rs`（UI-1 → UI-3 → P8-CK）、`alert.rs`（UI-1 → UI-2）、`values.rs`・`look.rs`・basic `recipes.rs`・creative `recipes.rs`（UI-3 → P8-CK）、`code_block.rs`・`data_table/rows.rs`（UI-3 → P8-UI5）。どれも前の unit の commit の上で行い、変える行は重ならない。
- **他の lane・wave との重なり**：
  - L4（IN-1〜3、P8-IN）・L6（UI-4、SO、CR、EX、DT、P8-UI6、P8-SOFT、P8-KITS）の所有ファイルと L5 の所有ファイルの重なりは無い（basic・soft の `recipes_menu.rs` は L4、soft の `side_navigation`・`lightbox`・P8-SOFT の 26 ファイル、basic の `breadcrumb.rs`・`sidebar/*` は L6。L5 の soft は `card.rs`・`banner.rs`・`recipes_controls.rs`・`recipes_controls_choice.rs`・`segmented_control.rs`・`indicator.rs` だけ）。
  - W1 の M10（`button.rs`、`kivact-interaction/src/toggle.rs`）：UI-3 は `button.rs` の `Button::finish` の state の 1 行と doc の 1 行だけを変え、M10 の分岐に触れない。`kivact-interaction/src/toggle.rs` は L5 の誰も変えない。
  - W5 の G6（`crates/kivact-ui/src/lib.rs` の C-10 の module doc、`docs/**` の C-11 の import の整理、C-15）：L5 は `lib.rs` の `pub use` 行と `feedback.md` の本文（コード片でない文）だけを変えるので、G6 の後のファイルにそのまま書ける。G6 への追記は不要。
  - W5 の G7（`calendar/view.rs`、`menu/handle.rs`）：L5 と重ならない。
  - W7 の FINAL-1（PartState → Availability）：L5 が新しく書く PartState の field の読み書きは無い。FINAL-1 の compile error が示す L5 由来の読み手は、soft `card.rs` の `overlay`（`s.input().filter(|input| !input.disabled)`、`input.pressed`、`input.hovered`）と soft `indicator.rs`（P5-F 3 章のとおり）。FINAL-1 の行は「compile error が示す全ての file」なので追記は不要。
  - W7 の G9：`toggle_group(.., options, selection)` の引数名の改名（表の 6）は関数の定義の引数名と rustdoc だけで、UI-3 が変える `toggle_group.rs` の行（ToggleButtonState の literal の 1 行）と重ならない。G9 への追記は不要。
- **他の lane への申し送り（親が統一すること）**：0.1 の命名規則（bool accessor は `is_`／`has_`、pub enum field ＋ accessor、`State` で終わらない部分 enum の名前）を L4 の IN-2（E-2 `MenuItemState`）と L6 の UI-4（F-T1 `BreadcrumbItemState`）にも当てはめる。E-2 の `check()`・`is_expanded()`、F-T1 の `is_current()`・`input()` は既に規則どおり。
- **人間にしか決められない項目**：なし。


---

# R6 監査：W6 lane L6 前半（UI-4 → SO → CR → EX）

対象 HEAD 2a3b06b8。行番号は HEAD のもの（付属書の 73c36c7c の行とずれる箇所は名前で示す）。

前提の確認（4 unit 共通）：
- W1（`git log --stat eef6dc9f..HEAD`）が変えた kit・example のファイルは `crates/kivact-ui/src/button.rs` と `crates/kivact-interaction/src/toggle.rs`（M10）だけ。L6 前半の所有ファイルとは重ならない。
- W2〜W4（D1〜D4・D6・D7）の所有ファイルにも L6 前半のファイルは無い（D4 S9 の `examples/*/src/fonts.rs` は別ファイル）。
- W5：G6 は `docs/**` を持つ（C-11 の重複 import の削除で `docs/for-kivact-user/{en,jp}/how-to-guide/navigation.md` の code block を変え得る）。W5 は W6 の前なので順に行われ、衝突しない。UI-4 が変える navigation.md の 47 行目は散文で、G6 の C-7〜C-11 の対象外。G7（`calendar/view.rs`、interaction の `menu/handle.rs`）は L6 前半と重ならない。
- W7：FINAL-1（PartState）と G9（改名）は L6 前半の後。重なりは各 unit の節に書いた（いずれも「後の unit が compile error に従って直す」で済み、L6 前半の手順は変わらない）。
- state-fields lint：L6 前半の後、L6 前半の所有ファイルに残る `projection:` は `ButtonCfg.has_action`（EX）の 1 本だけで、X4 を満たす（EX の節）。L6 前半が置く `independent:` の文は全て、workspace の他ファイルと D5・D8 の他の行に同じ文が無いことを `git grep -F` と付属書の grep で確かめた（X5 を満たす）。

---

## UI-4（付属書 D5 6 章 UI-4：P5-E 6 章 E-U6、P5-F 1 章 F-T1・F-T6、2 章の kivact-ui 分と (b) の kivact-ui 分、本文 3.5 の button_group、W6 lane L6）

### 判定：修正あり

### 修正（作業指示書 9 章 UI-4 の行の「本書の追加の指示」に入れる文）

- **範囲の確定**：UI-4 が行うのは次の全部で、それ以外は行わない。
  1. P5-F 1 章 F-T1（`BreadcrumbItemState`）と F-T6（slider の `Geometry`）。
  2. P5-F 2 章の表のうち、file が `chat_values.rs`、`collection_values.rs`、`disclosure_values.rs`、`indicator_values.rs`、`navigation_values.rs`、`table_values.rs`、`tree_values.rs` の行。
  3. P5-F 2 章 (b) の表のうち、file が `crates/kivact-ui/src/carousel.rs:143`、`combobox/data.rs:27,31,43`、`menu.rs:42,278`、`slider.rs:177,179`、`disclosure_tokens.rs:9,12`、`progress_tokens.rs:6,15` の行。
  4. D5 本文 3.5 の `crates/kivact-ui/src/button_group.rs`（`ButtonGroupMember.keep_size`）の行。
  - P5-F 2 章の `values.rs` の表は UI-3、`component_values.rs` の表と (b) の `data_table/types.rs:132` は UI-1、(b) の `kivact-interaction`・`kivact-test`・`kivact-devtools`・`examples` の行は IN-1／IN-2／IN-3／KT／DT／EX が行う。UI-4 は行わない。
- **F-T1 の書き手（`crates/kivact-ui/src/breadcrumb.rs` の `breadcrumb_entry`）**：付属書の「`:155` の直後に `let is_current = current;`」は使わない（HEAD ではその位置で `current` が既に `Option<CurrentItem>` に shadow されている）。代わりに次にする：
  - `if current || activate.is_none() {` の中の `let current = current.then_some(CurrentItem::Page);` の直後に、次の 1 行を足す：
    ```rust
    let state = if current.is_some() { BreadcrumbItemState::Current } else { BreadcrumbItemState::Label };
    ```
  - 同じ branch の 2 つの `move || BreadcrumbItemState { current: current.is_some(), input: None, }` を、どちらも `move || state` にする。
  - else の branch の `move || BreadcrumbItemState { current, input: Some(input.get()), }` を `move || BreadcrumbItemState::Link(input.get())` にする。
  - 型は付属書 F-T1 のコードのまま（`navigation_values.rs` の struct を置き換え、marker 2 本を消す）。`kivact_core::custom::PartState` は Copy・PartialEq・Eq なので derive は付属書のとおりでよい。
- **F-T1 の読み手**（4 ファイル。workspace の読み手はこれで全部。dev/tests・examples・docs のコード片に field を読むものは無いことを grep で確かめた）：
  - `crates/kivact-ui/src/look_navigation.rs` の `breadcrumb_item`：`.color(if s.current {` → `.color(if s.is_current() {`
  - `crates/kivact-ui-basic/src/breadcrumb.rs` の `item`：`.color(if s.current {` → `.color(if s.is_current() {`
  - `crates/kivact-ui-soft/src/breadcrumbs.rs` の `item`：`(s.current || s.input.is_none())` と `s.current || s.input.is_none()` の 2 箇所 → `s.input().is_none()`。`s.input.is_some_and(|input| input.hovered)` → `s.input().is_some_and(|input| input.hovered)`（`input.hovered` は field のまま書く。`is_hovered()` にするのは後の FINAL-1）。
- **F-T1 の文書**（`docs/for-kivact-user/{en,jp}/how-to-guide/navigation.md` の 47 行目。行の残りの文は変えない）：
  - en：`` Breadcrumb item recipes receive `BreadcrumbItemState { current, input }`; `input` comes from the actual link part and is absent for passive/current labels. `` を `` Breadcrumb item recipes receive `BreadcrumbItemState`: `Current`, `Label` or `Link(input)`. `is_current()` tells the current page, and `input()` is the actual link part's state, `None` for passive and current labels. `` に置き換える。
  - jp：`` Breadcrumb の項目の recipe は `BreadcrumbItemState { current, input }` を受け取ります。`input` は実際のリンクの Part から来る値で、操作できないラベルや現在ページのラベルでは `None` です。 `` を `` Breadcrumb の項目の recipe は `BreadcrumbItemState`（`Current`、`Label`、`Link(input)` のどれか）を受け取ります。`is_current()` は現在ページかどうか、`input()` は実際のリンクの Part の値で、操作できないラベルや現在ページのラベルでは `None` です。 `` に置き換える。
- **F-T6（`crates/kivact-ui/src/slider.rs`）**：付属書の型と読み手の置換はそのまま行う。書き手だけ次に変える（`Slider::into_view` は HEAD で 76 code 行あり、付属書の 7 行の if 式を literal に書くと clippy `too_many_lines`（80）を超えるため）：
  - `enum TrackFlow` の直後に次を足す：
    ```rust
    impl TrackFlow {
        /// The flow of a track with this axis, reading the inherited
        /// direction at setup.
        fn new(vertical: bool) -> Self {
            if vertical {
                Self::BottomToTop
            } else if use_direction().get() == ReadingDirection::Rtl {
                Self::RightToLeft
            } else {
                Self::LeftToRight
            }
        }
    }
    ```
  - `into_view` の `Geometry { .. }` の literal の `vertical,` と `rtl: !vertical && use_direction().get() == ReadingDirection::Rtl,` の 2 行を `flow: TrackFlow::new(vertical),` の 1 行にする。
  - `Geometry` の doc と `fn vertical(&self) -> bool` は付属書のとおり。`use_direction`・`ReadingDirection` の import は今の行のまま使う。
- **marker の文言の訂正（1 件）**：P5-F 2 章 (b) の `disclosure_tokens.rs:9`（`DisclosureMotion.animate`）の文は存在しない method `Disclosure::animate`・`.animate(true)` を名指している（実際は `Disclosure::animated`）。次の全文にする：
  `// xtask-allow(state-fields): independent: the theme's default for Disclosure::animated; a disclosure that opts in with .animated(true) uses transition even when this is false`
- **button_group**：`crates/kivact-ui/src/button_group.rs` の `ButtonGroupMember.keep_size` の marker を D5 本文 3.5 の全文 `// xtask-allow(state-fields): option: ButtonGroupMember::keep_size; false lets the group size the member like its siblings` にする（数える field は `keep_size` だけだが marker は使用済みになり `unused marker` にならない）。
- 上記以外の marker の文言は P5-F の表の全文のまま（名指す関数・method は全て HEAD に実在することを確かめた：`ChatMessage::{status,reactions,on_press,header,footer,avatar,recipe_state}`、`Toolbar::center`、`ItemRow::{description,description_content,on_press,position,title_lines,description_lines}`、`ItemListTokens::{title_lines,description_lines}`、`Badge::{on_press,icon,trailing_icon}`、`StepperItem::{disabled,optional}`、`Pagination::{total_items,has_more}`、`RowStatus::{icon,color}`、`TreeRow::is_last`、`Tree::marker_content`、`DisclosureRecipeState::from_part`、`Carousel::looping`、`Combobox::{free_text,chips,exclude_selected}`、`Menu::at_least_anchor_width`）。
- **lint の確認**（変更後）：`Geometry` は数える field が 0（marker 0）。`BreadcrumbItemState` は enum（tuple variant は数えない）。`StepperItemState` は `status: StepStatus`（名前が `Status` で終わる workspace enum ＝ phase）があるが、bool 4 本は全て marker を持つので error にならない（X3 の後も無印の stored が 0）。他の struct は全ての数える field に marker がある。UI-4 の後、UI-4 の所有ファイルに `projection:` は残らない（X4 の対象外）。
- **残っている文の扱い（変更しない）**：`chat_values.rs` の `ChatComposerState.stopped`・`status`（同じ文 `independent: copied from the caller's stop_shown and status` が 2 本）は P5・D8 どちらの表にも無いが、同じファイルの中の共有なので X5 に当たらず、caller の別々の入力を写す事実とも合う。UI-4 は変えない。
- **確認のコマンド**（付属書の行に加える）：`cargo check -p kivact-ui -p kivact-ui-basic -p kivact-ui-soft --all-targets`；`cargo clippy -p kivact-ui --all-targets -- -D warnings`；`cargo xtask check --no-write --only state-fields --crate kivact-ui`（kivact-ui-basic、kivact-ui-soft も）が error 0；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- apg_breadcrumb ui_navigation apg_slider interaction_slider`。

### 所有ファイルの変更（あれば）
- 変更なし（D5 6 章の UI-4 の所有ファイルのまま）。`crates/kivact-ui/src/lib.rs` は変えない（`BreadcrumbItemState` は同じ名前で export 済み、`TrackFlow` は private）。

### wave・lane・順序の変更（あれば）
- なし。同じ lane の後続との境界（同じファイルを持つもの）：
  - `carousel.rs`：UI-4 は `Layout.looping` の marker だけ。P8-UI6 は `Layout.per_view` の marker だけ（`gap`・`padding` は確認済みで変えない）。
  - `combobox/data.rs`：UI-4 は `free_text`・`chips`・`exclude_selected`。P8-UI6 は `query`・`open`・`search`。
  - `navigation_values.rs`：UI-4 は `StepperItemState.collapsed`・`disabled`・`optional`、`BreadcrumbItemState`（型）、`PaginationSummary.total_items`・`has_more`。P8-UI6 は `StepperItemState.interactive` の marker だけ。
- W7 との境界：FINAL-1 は `crates/kivact-ui-soft/src/breadcrumbs.rs` の `input.hovered` を `input.is_hovered()` に直す（E0609 が示す）。G9 は `kivact-ui/src/menu.rs`・`combobox` の呼出し元を compile error に従って直す（UI-4 の marker の行には当たらない）。

---

## SO（付属書 D5 6 章 SO：P5-E E-5・E-6 の lightbox の行、本文 3.5 と 3.6 O3、W6 lane L6）

### 判定：修正あり（小）

### 修正
- **E-5（`crates/kivact-ui-soft/src/side_navigation/shell.rs`）**：付属書のとおり。HEAD の位置と書き方を確定する：
  - `enum SidebarWidth` とその doc、`impl SidebarWidth` を消す。
  - `SideNav` の `into_view` の `let expanded_width = signal(SidebarWidth::Expanded(resize.as_ref().map_or(260.0, |resize| resize.default_width)));` を `let expanded_width = signal(resize.as_ref().map_or(260.0, |resize| resize.default_width));` にする（`signal(<method 呼出し>)` は locals の lint が数えない）。
  - `fn resize_grip` と `fn sidebar_width` の引数 `expanded_width: kivact::state::Signal<SidebarWidth>` を `expanded_width: kivact::state::Signal<f64>` にする。
  - `resize_grip` の getter の `expanded_width.get().expanded()` → `expanded_width.get()`。setter を次にする：
    ```rust
        move |size| {
            if size < 160.0 {
                if let Some(collapsed) = collapsed {
                    collapsed.set(true);
                }
            } else {
                if let Some(collapsed) = collapsed {
                    collapsed.set(false);
                }
                expanded_width.set(size.clamp(resize.min, resize.max));
            }
        },
    ```
  - `sidebar_width` の `expanded_width.get().expanded()` → `expanded_width.get()`。
  - 振る舞いは変わらない（`Collapsed { restore }` の restore は折り畳む前の幅で、折り畳み中は幅を変えないので同じ値）。
- **E-6・3.6 O3（`crates/kivact-ui-soft/src/lightbox.rs` の `Lightbox`）**：
  - `index` の marker：`// xtask-allow(state-fields): option: the caller's index binding from Lightbox::index; None keeps the gallery's own index from 0`
  - `view` の marker：`// xtask-allow(state-fields): option: absent keeps the gallery's own fit and zoom view; LightboxHandle::view passes a binding projected from its state`
  - `zoom` の marker は変えない（D8 付録 B で確認済み）。
  - 3 本とも `option:` で、`projection:` が残らないので X4 の対象外。`LightboxHandle::view` と `Lightbox::index` は HEAD に実在する。
- 確認のコマンド：`cargo check -p kivact-ui-soft --all-targets`；`cargo clippy -p kivact-ui-soft --all-targets -- -D warnings`；`cargo xtask check --no-write --only state-fields --crate kivact-ui-soft`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-soft --test consumer -- rail_navigation`（`rail_navigation_has_one_actual_panel_and_rearms_after_expansion` は HEAD の `crates/kivact-ui-soft/tests/consumer/navigation.rs` にある）。

### 所有ファイルの変更（あれば）
- なし。

### wave・lane・順序の変更（あれば）
- なし。`side_navigation/shell.rs` の境界：SO は `SidebarWidth`・`expanded_width`・`resize_grip`・`sidebar_width` だけを変え、P8-SOFT は `SideNavSlots` の 4 本の marker（`header`・`top`・`footer`・`icons`）だけを変える。`resize_grip` の `state.input.hovered` は FINAL-1 が直す。

---

## CR（付属書 D5 6 章 CR：P5-C C4、W6 lane L6）

### 判定：修正あり（小）

### 修正
- 型・書き手・読み手は付属書 C4 のとおり（`crates/kivact-ui-creative/src/editors/node/gesture.rs` の `Info`、`a11y.rs` の `semantic` と `link_info` の closure 2 つ、`view.rs` の `region_control`・`region_actions`）。HEAD の位置は付属書と同じ（`Info` は `gesture.rs` の `pub(crate) struct Info`、marker は `checked` と `range` の上の 2 本）。
- **accessor の名前の統一**：bool を返す accessor は `is_` を付ける（`is_current()`、`is_checked()`、`is_pressed()` と揃える）。付属書の `pub(crate) fn selected(&self) -> bool` を `pub(crate) fn is_selected(&self) -> bool` にし、`view.rs` の `node.part(control).selected(move || info.get().selected)` を `node.part(control).selected(move || info.get().is_selected())` にする。`checked()`（`Option<bool>`）と `range()` は付属書のまま。
- `Facet` は `#[derive(Clone, Copy, Debug, PartialEq)]`（`Range` に f64 があるので `Eq` は付けない）。`Info` の derive は今の `#[derive(Clone, PartialEq)]` のまま。
- lint：`Info` は数える field が 0、`Facet::Toggle`・`Facet::Link` は数える field が 1 個ずつ（行が出ない）、`Facet::Range` は f64 だけ（数えない）。marker 2 本を消す（残すと `unused marker`）。
- **追加の試験 `node_link_region_reports_its_selection`**（`crates/kivact-ui-creative/tests/consumer/node_editor.rs` の末尾）：付属書のコードのまま使える。helper（`test_node`、`crate::node_support::app_node_editor`、`crate::support::options()`、`kivact::test::render_with_handle`、`A11yAction`、`Locator::is_selected`）は全て HEAD に実在し、link の名前 `"Link Image output 0 to Mix input 0"` は `texts.rs` の既定文言 `"Link {from} output {output} to {to} input {input}"`（`output`・`input` は数字）と一致する。link の region は `a11y::targets` が両端の node のある link を全て登録するので存在し、`region_actions` の既定の腕が `A11yAction::Select` を `State::invoke(target, true)` → `selected_link.set(..)` に結ぶ。node の Select は `edit.rs` の `select` が `selected_link.set(None)` する。振る舞いを変えない unit なので、試験は HEAD でも通る。順序：試験を先に足して `cargo test` で通ることを確かめてから型を変える（通らなければ既存欠陥として STOP。期待値は変えない）。
- **D5 付録 P5-C「既存欠陥 5」の推測（`RegionInfo.checked`・`selected`）の確定（R6 が決める。CR が行う）**：`crates/kivact-ui-creative/src/editors/shared/region_parts.rs` の `RegionInfo.selected` は、`RegionInfo::new` の `selected: false` の他に書き手も読み手も無い（`regions.rs` は `info.checked` だけを書き、`attachment` は `checked` だけを TrackSwitch で読む。range 側の `range/implementation.rs` は `RegionInfo::new` だけを使う。Grep 済み）。死んだ field なので消す：
  - `pub(crate) selected: bool,` とその直前の marker 行 `// xtask-allow(state-fields): independent: the a11y selected output, recomputed from the domain for any node` を消す。
  - `RegionInfo::new` の `selected: false,` の行を消す。
  - `checked` の直前の marker 行 `// xtask-allow(state-fields): independent: the a11y checked output, recomputed from the domain for any node` も消す（`RegionInfo` で数える field は `checked` 1 個だけになり、組合せが無い。marker は無くても error にならない）。
  - 振る舞いは変わらない。試験の追加は無い。
- 確認のコマンド：付属書の C-CR の行のとおり（`cargo clippy -p kivact-ui-creative --all-targets -- -D warnings`；`cargo xtask check --no-write --only state-fields --crate kivact-ui-creative`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-creative --test consumer -- node_editor`）。

### 所有ファイルの変更（あれば）
- `crates/kivact-ui-creative/src/editors/shared/region_parts.rs` を CR の所有に足す（上の RegionInfo の 3 行の削除だけ。W6 の他の unit・W5・W7 の unit はこのファイルを持たない）。

### wave・lane・順序の変更（あれば）
- なし。FINAL-1 は `view.rs` の `input.disabled || input.hidden`（`region_control` の外、HEAD の 205 行付近）を `is_disabled()`／`is_hidden()` に直す。CR はその行を変えない。

---

## EX（付属書 D5 6 章 EX：P5-E E-3・E-7・E-10、P5-F 5 章 F-U7、本文 3.5 の `ButtonCfg.has_action`、W6 lane L6）

### 判定：修正あり

### 修正
- **E-3（`examples/kivact-ui-game/src/game/window.rs`）**：`frame` は HEAD で本体 70 code 行あり、付属書の「`"run" => { run.set(..); status.set(..); }`」を 2 本書くと rustfmt で 6 行増える。上限に近いので helper にする：
  - `impl Run` から `fn message` とその doc を消す。`enum Run` の doc を `/// What the editor chrome's run commands set; the status bar's activity word comes from it.` にする。
  - `frame` の `command` の `"run" => run.set(Run::Running),` と `"pause" => run.set(Run::Paused),` の 2 行を `"run" | "pause" => run_command(key, run, status),` の 1 行にする。
  - `fn window_status` の前に次を足す：
    ```rust
    /// Record a run command: the activity it sets and the notice it leaves in
    /// the status bar, where later commands such as Save replace the notice.
    fn run_command(key: &str, run: Signal<Run>, status: Signal<String>) {
        let (next, notice) = if key == "run" {
            (Run::Running, "Running")
        } else {
            (Run::Paused, "Paused")
        };
        run.set(next);
        status.set(notice.to_owned());
    }
    ```
  - `window_status` の `g::mono(move || { run.get().message().map_or_else(|| status.get(), str::to_owned) })` を `g::mono(move || status.get())` にする（`.test_id("game-window-status")` は残す）。`run` 引数は activity の表示で使うので残す。
  - lint：`frame` の local は `width`・`help` だけが数えられ（`signal(Run::Idle)` は path の末尾が `None` でないので数えない）、どちらも marker を持つ。`Run` は名前が phase（`Run` で終わる）だが struct の field ではないので関係しない。
- **E-3 の試験（付属書の手順は catalog の locale と合わないので次に差し替える）**：catalog は `locale()` が `ja-JP` で、tool button の名前は `message("game.window.run", "Run")` → 「実行」、`message("game.window.save", "Save")` → 「保存」になる。さらに「保存」は `game/system.rs` の別の `tool_button("保存")` にもあるので、toolbar の中で探す。`examples/kivact-ui-game/tests/catalog.rs` の末尾に次を足す（`Role` は既存の import、`mount`・`chapter` は同じファイルの helper）：
  ```rust
  #[test]
  fn window_status_shows_the_last_command_after_run() {
      let screen = mount();
      chapter(&screen, "layout");
      let tools = screen.get_by_role(Role::Toolbar, "Console tools");
      let status = || screen.get_by_test_id("game-window-status").text();
      tools.get_by_role(Role::Button, "実行").click();
      assert_eq!(status(), "Running");
      tools.get_by_role(Role::Button, "保存").click();
      assert!(status().starts_with("Saved "), "{}", status());
      assert!(screen.errors().is_empty(), "{:?}", screen.errors());
  }
  ```
  （HEAD では 2 つ目の assert が「Running」のままで失敗する。これが E-3 の退行の再現。）実行：`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-game-catalog --test catalog -- window_status`。
- **E-7（`examples/kivact-ui-creative/src/creative/jobs.rs`）は行わない（取消し）**。理由：付属書の「clock は machine より前に作る（今と同じ順序）」は事実と逆で、HEAD は machine（`job`）を先に作り、`frame_loop` の tick が `job.send(JobEvent::Finished)` を呼ぶ。machine の effect の handler が `clock` を使うと、clock の tick が machine を、machine の handler が clock を参照する循環になり、解くには `Rc<Cell<Option<FrameLoop>>>` のような新しい状態が要る（方針「新しい型はまず状態を持たない形」に反する）。今の `effect(move || if running.get() { clock.start(..) } else { clock.stop() })` は machine の唯一の状態から導いた `running` を外部の資源（frame loop）に写すだけで、二重の持ち主も lint の行も無い。`jobs.rs` は EX の所有から外す。
- **E-10**：
  - game：`examples/kivact-ui-game/src/game/controls.rs` の `grid_on` と `filter_on` の marker（HEAD の 65・67 行。P8-O2）を、付属書の文 `// xtask-allow(state-fields): independent: the grid toggle and the filter toggle are pressed separately and all four combinations are valid views` にする（同じファイルの 2 本の共有は X5 が許す）。
  - creative：付属書の bool 用の 1 文は事実と合わない（`search` は検索 button・F3・feedback の章の button の 3 つが開く。`viewport_grid` は compact・sidebar・search と無関係）。`examples/kivact-ui-creative/src/creative/shell.rs` の `CatalogControls` の field の marker と、`CatalogControls::new` の同名の local の marker を、どちらも次の全文にする（field と local で同じ文。同じファイルなので X5 に当たらない）：
    - `compact`：`// xtask-allow(state-fields): independent: flipped by the density button and Ctrl+Alt+D only; either row density shows with any chapter, sidebar and search overlay state`
    - `current`：`// xtask-allow(state-fields): independent: the chapter chosen in the sidebar list; every chapter is a reachable view in either density and with the sidebar or search overlay in any state`
    - `sidebar_open`：`// xtask-allow(state-fields): independent: the top bar's sidebar toggle; the sidebar hides or shows over any chapter and density, with the search overlay up or down`
    - `search`：`// xtask-allow(state-fields): independent: opened by the search button, F3 and the feedback chapter's button, and closed by the overlay; it covers any chapter, density and sidebar state`
    - `shading`：`// xtask-allow(state-fields): independent: the viewport shading mode chosen in the viewport toolbar or the pie menu; every mode draws with the viewport grid on or off`
    - `viewport_grid`：`// xtask-allow(state-fields): independent: the viewport grid flipped from the viewport toolbar or the pie menu; the grid draws over every shading mode`
- **F-U7**：P5-F 2 章 (b) の表の `examples/kivact-ui-creative/src/creative/audio_values.rs`（`TransportState.recording`・`cycling`）、`examples/kivact-ui-basic/src/docs/alert_dialog.rs`（local `open`・`deleted`）、`examples/kivact-ui-basic/src/docs/dialog.rs`（local `open`・`deleted`）の 6 行を、表の全文のまま置く（書き手は HEAD で確かめた：`audio_samples.rs` の record toggle と loop switch、`arrange.rs` の loop switch、`audio_values.rs` の `controls`、各 dialog の「Delete Chat」「Delete account」・Cancel・Delete）。`audio_values.rs` の `playing` の marker は変えない（`crates/kivact-ui-creative/src/audio/transport.rs` の文と違う）。
- **basic sidebar（本文 3.5）**：`crates/kivact-ui-basic/src/sidebar/menu.rs` の `ButtonCfg.has_action` の marker を `// xtask-allow(state-fields): projection: whether the button has an action view, written only by build_button` にする。X4：field は private なので可視範囲は `sidebar::menu` の module だけで、書く式は `build_button` の中の `ButtonCfg { .., has_action }` の literal だけ（`let ButtonCfg { size, active, .. } = cfg;` は pattern で書込みではない）。`fn build_button` は同じ module に実在する。X4 を満たす。
- **E-11（推測、creative の `TargetState.active`）は V2 で行わない**：D5 6 章のどの unit の行にも無く、試験の名前も無い（作業指示書 0.2 の 3）。
- 確認のコマンド（付属書の行に加える）：`cargo check -p kivact-ui-game-catalog -p kivact-ui-creative-catalog -p kivact-ui-basic-catalog -p kivact-ui-basic --all-targets`；`cargo clippy -p kivact-ui-game-catalog --all-targets -- -D warnings`；`cargo xtask check --no-write --only state-fields --crate <pkg>` を kivact-ui-game-catalog、kivact-ui-creative-catalog、kivact-ui-basic-catalog、kivact-ui-basic で（error 0）；上の `window_status` の試験。

### 所有ファイルの変更（あれば）
- `examples/kivact-ui-creative/src/creative/jobs.rs` を EX の所有から外す（E-7 取消し）。
- 残り：`examples/kivact-ui-game/src/game/window.rs`、`game/controls.rs`、`examples/kivact-ui-game/tests/catalog.rs`、`examples/kivact-ui-creative/src/creative/shell.rs`、`creative/audio_values.rs`、`examples/kivact-ui-basic/src/docs/alert_dialog.rs`、`docs/dialog.rs`、`crates/kivact-ui-basic/src/sidebar/menu.rs`。

### wave・lane・順序の変更（あれば）
- なし。同じ lane の P8-KITS との境界：
  - `game/window.rs`：EX は `Run`・`command`・`run_command`・`window_status`。P8-KITS は `frame` の local `width`・`help` の marker 2 本だけ（D8 の新しい文は EX の後も事実と合う）。
  - `game/controls.rs`：EX は `grid_on`・`filter_on`（65・67 行）。P8-KITS は `step`・`page`（164・166 行）。
  - `sidebar/menu.rs`：EX は `ButtonCfg.has_action` だけ。P8-KITS は `SidebarGroup.action`・`content`、`SidebarMenuButton.icon`・`badge`・`action`・`sub` の 6 本。`ButtonCfg.cx` は確認済みで誰も変えない。

---

## L6 前半の全体

### 後半の unit とのファイルごとの分担（境界の事実）
| ファイル | L6 前半 | L6 後半 |
|---|---|---|
| `crates/kivact-ui/src/carousel.rs` | UI-4：`Layout.looping` | P8-UI6：`Layout.per_view` |
| `crates/kivact-ui/src/combobox/data.rs` | UI-4：`free_text`・`chips`・`exclude_selected` | P8-UI6：`query`・`open`・`search` |
| `crates/kivact-ui/src/navigation_values.rs` | UI-4：Stepper の 3 本、Breadcrumb の型、Pagination の 2 本 | P8-UI6：`StepperItemState.interactive` |
| `crates/kivact-ui-soft/src/side_navigation/shell.rs` | SO：`SidebarWidth` 一式 | P8-SOFT：`SideNavSlots` の 4 本 |
| `crates/kivact-ui-basic/src/sidebar/menu.rs` | EX：`ButtonCfg.has_action` | P8-KITS：6 本 |
| `examples/kivact-ui-game/src/game/window.rs` | EX：`Run`・command・status | P8-KITS：`width`・`help` |
| `examples/kivact-ui-game/src/game/controls.rs` | EX：`grid_on`・`filter_on` | P8-KITS：`step`・`page` |

L6 前半どうし、および同じ wave の L1〜L5 の所有ファイルとの重なりは無い（L5 の `kivact-ui` のファイル群・kit の recipe・soft の `indicator.rs` 等と、L4 の interaction・`recipes_menu.rs` と照合した）。L6 前半のファイルは、L4 が変える `MenuItemState` の field も、L5 が変える `ButtonState`・`CheckableState`・`CardState` 等の field も読まない（`kivact-ui/src/menu.rs` は `MenuItemState` を recipe に渡すだけ）。

### W7 との境界
- FINAL-1 が直す L6 前半の範囲の読み手：`crates/kivact-ui-soft/src/breadcrumbs.rs`（`input.hovered`）、`side_navigation/shell.rs`（`state.input.hovered`）、`crates/kivact-ui-creative/src/editors/node/view.rs`（`input.disabled || input.hidden`）。L6 前半は PartState の field を新しく書かない（F-T1 の `Link(PartState)` は値を包むだけ）。
- G9 の改名表と L6 前半の変更は重ならない（`tree(..).selection`・`tab_list`・`gradient_editor` は L6 前半のファイルに無い）。

### DEC-14 の命名（L6 前半の範囲）
- 公開：`BreadcrumbItemState::is_current()`／`input()` は、PartState の `is_focused()`・FINAL-1 の `is_pressed()`、D8 の `is_checked()`・`is_mixed()`、UI-1 の `CardState::input()`・`is_interactive()` と同じ規則（bool は `is_*`、値は名詞）に揃っている。変更なし。
- crate 内：CR の `Info::selected()` を `is_selected()` にした（上記）。

### 範囲外で見つけた事実（親への申し送り。L6 前半では変えない）
- `crates/kivact-interaction/src/calendar/data.rs` の `CalendarDayState.disabled` の marker は `projection: … written only by the cell part` のままで、D5 E-U6 は「OPEN 担当の unit に統合」とするが、`calendar/data.rs` を所有する unit が L4 に無く、D8 付録 B にも行が無い。X4 は `the` を関数名と読み「存在しない」の error になる。L4（IN-3 か P8-IN）に足す必要がある。
- `crates/kivact-ui-creative/src/editors/shared/region_parts.rs` の `RegionInfo.checked`・`selected`（D5 P5-C 既存欠陥 5 の「推測」）は D8 付録 B に行が無い。→ R6 の統合で確定：CR が `selected`（死んだ field）と 2 本の marker を消す（CR の節）。
- 提案（任意）：`kivact-ui-soft` の `LightboxHandle::show(index)`／`close()` は ARG-6 の `show`／`hide` に対し `close` が残る。G9 の改名表に足すかは親の判断。

### 人間にしか決められない項目
- なし。


---

# R6：W6 lane L6 の後半（DT → P8-UI6 → P8-SOFT → P8-KITS）の再監査

前提の確認（全 unit 共通）：
- 4 つの unit が変えるファイル（DT 2、P8-UI6 17、P8-SOFT 26 と試験 1、P8-KITS 22）は、`git diff --stat 73c36c7c..HEAD` で W1 の変更が 0 件。したがって付属書 D5・D8 の `file:line` は HEAD 2a3b06b8 でもそのまま合う。付録 B の 182 行（P8-UI6 45、P8-SOFT 77、P8-KITS 60）は全部、示された行に marker があり、次の field・local の名前も一致する（script で照合済み）。
- W2〜W5 の unit で、これらのファイルを持つものは G6（W5、`examples/kivact/component/src/main.rs` の C-11 の `.to_owned()` の 1 箇所）と G7（W5、`crates/kivact-ui/src/calendar/view.rs` の `on_commit` の署名）だけ。どちらも本書の対象の marker・field を変えない（下の各節）。
- 同じ lane の前の unit（UI-4・SO・CR・EX）が同じファイルを先に変える：`carousel.rs`・`combobox/data.rs`・`navigation_values.rs`（UI-4）、`side_navigation/shell.rs`（SO）、`kivact-ui-basic/src/sidebar/menu.rs`・game の `controls.rs`・`window.rs`（EX）。付録 B の本書の行は、どれも前の unit が変える行と別の field で、前の unit の型の変更で消える・数えなくなる field は無い（UI-4 の F-T1 は `BreadcrumbItemState` の 2 行、SO の E-5 は `SidebarWidth`、EX は `ButtonCfg.has_action`・`filter_on`・`run`／`status` だけを変える）。行番号はずれるので「`owner.field` で探す」（D8 3.3 の適用の規則 1）。
- 本書の 4 unit が置く・残す marker に `projection:` は 1 本も無い（新しい文 0、残る文は UI-4 と EX が持つ `navigation_values.rs`・`sidebar/menu.rs` の行だけ）。X4 で落ちるものは無い。
- X5（独立の理由文がファイルをまたいで同一）：付録 B の全 917 行の新しい文で HEAD の marker を置き換えた集合を script で作り、`independent:` の重複を数えた。残る重複 12 種はすべて D5 が置換文を確定しているもの（P5-F 2 章(b)、C2、F-8、3.5 の 3 行）で、本書の範囲では DT の 1 種（`the last RPC failure is reported separately from the panel and its pop-outs`、`state.rs` と `session.rs`）だけ。DT の新しい 2 文は互いに異なり、他のどの文とも重ならない。P8-UI6・P8-SOFT・P8-KITS の新しい文は重複 0。

---

## DT（付属書 D5 付録 P5-F 2 章(b) の F-U6 の 2 行・5 章の F-U6、W6 lane L6）
### 判定：修正あり
### 修正（作業指示書の行の「本書の追加の指示」にそのまま入れる文）
- `crates/kivact-devtools/src/state.rs` の `Session` の field `status`（`pub(crate) status: Signal<Option<String>>`）の直前の marker 行を、P5-F 2 章(b) の文ではなく次の全文で置き換える（P5-F の文は書き手を `Session::report` だけと書くが、`Session::send` の失敗の腕 `self.status.set(Some(format!("{}: {}", M::NAME, error.message())))` も書く。事実に合わせる）：
  `// xtask-allow(state-fields): independent: the status bar's last RPC failure, set by Session::send when a call fails and by Session::report; it stays while connection moves to Lost, and a connected session may have none`
- `crates/kivact-devtools/src/session.rs` の `Session::new` の local `let status = signal(None);` の直前の marker 行を、P5-F 2 章(b) の文のまま次で置き換える：
  `// xtask-allow(state-fields): independent: becomes Session.status, empty at setup; the target var beside it is fixed for the session whether or not a failure is shown`
- 他の行は変えない（`let target = var(target);` の `identity:` の marker は付録 B で確認済み）。
- lint の確認（静的）：`Session` は `connection: Signal<Connection>`（phase）と `status`（数える field）を持つので `status` の marker は必要。`Session::new` の数える local は `target`（`var(..)`）と `status`（`signal(None)`）の 2 個で、両方に marker がある。
- 完了時の確認は D5 6 章の行のとおり：`cargo check -p kivact-devtools`、`cargo xtask check --no-write --only state-fields --crate kivact-devtools`（error 0）。
### 所有ファイルの変更
なし。
### wave・lane・順序の変更
なし。

---

## P8-UI6（付属書 D8 6 章の P8-UI6 の行、付録 B.2 の unit 列 P8-UI6 の 45 行、W6 lane L6）
### 判定：修正あり
### 修正
- 付録 B.2 の `crates/kivact-ui/src/navigation_values.rs:23` `StepperItemState.interactive` の新しい marker 行は、存在しない method `Stepper::on_activate` を名指している（`Stepper` の handler は `stepper.rs` の `pub fn on_step_press`。`step_row` が `let interactive = activate.is_some();` を書く）。次の全文で置き換える：
  `// xtask-allow(state-fields): independent: whether Stepper::on_step_press was given, the same for every step; a disabled, optional or collapsed step keeps it, and disabled alone refuses the press`
- G9（W7）の改名との整合のため、次の 3 行は付録 B の文の `options` を `items` にした全文で置く（G9 は `toggle_group(.., options, selection)`・`combobox(label, options, selection)` の引数を `items` に改名する。W6 の時点でも「the group's items」は事実どおり）：
  - `crates/kivact-ui/src/group_types.rs` `ToggleGroupItemState.first`：
    `// xtask-allow(state-fields): independent: whether this toggle's key is the first of the group's items; a lone item is both first and last and a middle one neither, so all four pairs occur`
  - 同 `ToggleGroupItemState.last`：
    `// xtask-allow(state-fields): independent: whether this toggle's key is the last of the group's items; with first it marks a lone item (both), the two ends (one each) and middle ones (neither)`
  - `crates/kivact-ui/src/combobox/data.rs` `Combobox.search`：
    `// xtask-allow(state-fields): option: Combobox::search; absent filters the caller's items locally, present fetches suggestions and skips the matcher`
- 他の 41 行は付録 B.2 の「新しい marker 行」の全文をそのまま置く。名指す method（`Calendar::default_month`・`Calendar::month`・`Carousel::items_per_view`・`Carousel::looping`・`Combobox::query`・`Combobox::open`・`CommandPalette::close_on`・`CommandPalette::footer`・`Dialog::description`・`HoverCard::label`・`ItemRow::{description, description_content, media, actions, header, footer, delegate_to}`・`Pagination::page_size`）は全て HEAD に実在する（script と grep で確認）。`Combobox::open` は kivact-ui の binding の builder（`combobox/handle.rs` の `pub fn open(mut self, open: impl IntoBinding<bool>)`）で、G9 が `show`/`hide` に改名する `ComboboxApi::open(self)`（kivact-interaction）とは別物。改名しない。
- 付録 B.2 の 2 行 `group_types.rs:35`（`ButtonGroupState.rtl`）と `hover_card.rs:43`（`HoverCard.label`）は owner.field 列が「(no audit row …)」になっているが、数える field が 1 個の struct の marker で、`count_fields` が使用済みにする（unused marker にならない）。marker 行を付録 B の文で置き換えるだけでよい。
- G7（W5 lane B）との関係（確定）：G7 は `Calendar::on_commit` の引数の型だけを `impl Into<Callback<()>>` に変え、field `on_commit: Option<Callback<()>>`（設定。数えない）は変えない。P8-UI6 が書き換える `Calendar.month`・`Calendar.displayed_month` の 2 行はそのまま書ける。marker の追加は不要。
- UI-4 との関係（確定）：UI-4 が先に `carousel.rs`（`Layout.looping`）、`combobox/data.rs`（`free_text`・`chips`・`exclude_selected`）、`navigation_values.rs`（`StepperItemState` の `collapsed`・`disabled`・`optional`、`BreadcrumbItemState`、`PaginationSummary`）を書き換える。P8-UI6 の行（`Layout.per_view`、`Combobox.query`・`open`・`search`、`StepperItemState.interactive`）とは別の field。
- X4・X5：P8-UI6 の新しい文に `projection:` は無く、`independent:` の文はファイルをまたいで重ならない。
- 完了時の確認は D8 6 章の行に次を足す：`cargo clippy -p kivact-ui --all-targets -- -D warnings`（marker だけなので失敗しないはずだが、L6 の他 unit と揃える）。
### 所有ファイルの変更
なし。
### wave・lane・順序の変更
なし（UI-4 の後、G7 の後。作業指示書 9 章の順で満たされる）。

---

## P8-SOFT（付属書 D8 6 章の P8-SOFT の行、3.4 P8-O3、付録 A-G5 W1・W2、付録 B.5 の unit 列 P8-SOFT の 77 行、DEC-22、W6 lane L6）
### 判定：修正あり
### 修正
1. **marker の置換（77 行）**：付録 B.5 の「新しい marker 行」の全文を置く。行番号の注記は付けない（付録 A-G5 のコード片には `(layout.rs:451)` 等の注記があるが、付録 B の文が正。D8 3.3 の規則 3）。名指す method（`AlertDialog::cancel_label`、`AppShell::{top_nav, side_nav, banner}`、`BottomSheet::final_focus`、`Layout::{padding, start, end, content_width, default_dividers}`、`LayoutRegion::divider`、`LayoutInset::from_padding`、`Link::{underline, standalone}`、`MobileNavConfig::{has_toggle, open, breakpoint}`、`NavHeadingMenuItem::{description, disabled}`、`Outline::{scroll_container, scroll_on_press}`、`SideNav::{header, top_content, footer, footer_icons}`、`SideNavItem::{icon, selected_icon, end_content, actions}`、`TabMenuOption::disabled`、`TopNav::{heading, start, center, end}`、`ItemLook::*`、`TreeLook::*`、`MetadataLook::*`、`Document::task`）は全て HEAD に実在する。
2. **W1（`LayoutInset.own_outer`）**：付録 A-G5 W1 のとおり。`crates/kivact-ui-soft/src/layout.rs` の `LayoutInset` は次の形にする（marker は付録 B.5 の文、doc は 1 行だけ直す）：
```rust
/// Insets inherited by Layout regions from a parent surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutInset {
    // xtask-allow(state-fields): option: inline outer padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback.
    /// Inherited inline outer padding in logical pixels; None uses the region fallback.
    pub outer_x: Option<f64>,
    // xtask-allow(state-fields): option: block outer padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback.
    /// Inherited block outer padding in logical pixels; None uses the region fallback.
    pub outer_y: Option<f64>,
    // xtask-allow(state-fields): option: inline gap padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback.
    /// Padding between inline regions in logical pixels; None uses the region fallback.
    pub inner_x: Option<f64>,
    // xtask-allow(state-fields): option: block gap padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback.
    /// Padding between block regions in logical pixels; None uses the region fallback.
    pub inner_y: Option<f64>,
    // xtask-allow(state-fields): option: this Layout's padding from Layout::padding, written only in Layout::into_view; absent uses the inherited outer_x and outer_y.
    /// This layout's own outer padding (`Layout::padding`) on both axes in
    /// logical pixels; when set it replaces `outer_x` and `outer_y`.
    pub own_outer: Option<f64>,
}
```
   - `Layout::into_view` の `insets.own_outer_x = padding.map(Space::px);` と `insets.own_outer_y = insets.own_outer_x;` の 2 行を `insets.own_outer = padding.map(Space::px);` の 1 行にする。
   - `region_insets` の 2 行を `inset.own_outer.or(inset.outer_x).unwrap_or(fallback)`、`inset.own_outer.or(inset.outer_y).unwrap_or(fallback)` にする。
   - 呼出し元の追従：workspace（crates・examples・dev・docs）に `own_outer_x`／`own_outer_y` の読み書きは `layout.rs` の外に 0 件（grep 済み）。`app_shell.rs`・`card.rs`・`section.rs`・`mobile_nav.rs` は `from_padding`／`default` だけを使うので変えない。
   - 名前（DEC-14 の観点）：`own_outer` は兄弟の `outer_x`・`outer_y`（親から受け継ぐ外側の余白）に対する「この Layout 自身の外側の余白」で、builder `Layout::padding` の 1 値と 1 対 1。field 名は付録どおり `own_outer` で確定し、上の doc で `Layout::padding` との対応と「outer_x/outer_y を置き換える」ことを書く（付録の doc「override on both axes」より読み手が迷わない）。
   - lint：`LayoutInset` の数える field は 5 個（全て `Option<f64>`）で、5 本の marker が全部使われる。
3. **W2（`LayoutSlots.content_width: Option<ContentWidth>`）**：付録 A-G5 W2 のとおり。ただし `layout_row_style` は付録の「以下、現行のまま」の省略をせず、次の全文にする（振る舞いは HEAD と同じ：Px かつ両側 panel でない時だけ gutter、他は max_width で中央寄せ）：
```rust
fn layout_row_style(slots: LayoutSlots) -> kivact::style::Style {
    let direction = if rtl() {
        Direction::RowReverse
    } else {
        Direction::Row
    };
    let mut base = style()
        .direction(direction)
        .grow(1)
        .min_height(0)
        .min_width(0);
    let Some(ContentWidth { root, width }) = slots.content_width else {
        return base;
    };
    match width.get() {
        Length::Px(width) if !(slots.has_start && slots.has_end) => {
            let gutter = center_gutter(root, width);
            if slots.has_start {
                base = if rtl() {
                    base.padding_right(gutter)
                } else {
                    base.padding_left(gutter)
                };
            } else if slots.has_end {
                base = if rtl() {
                    base.padding_left(gutter)
                } else {
                    base.padding_right(gutter)
                };
            }
        }
        width => {
            base = base
                .width(pct(100))
                .max_width(width)
                .margin_x(kivact::style::auto());
        }
    }
    base
}
```
   - `ContentWidth` は付録のとおり `LayoutSlots` の直前に `#[derive(Clone, Copy)] struct ContentWidth { root: NodeRef, width: Value<Length> }`（doc 1 行付き）。数える field は 0 個なので marker は置かない。`LayoutSlots` の数える field は `header_divider`・`footer_divider`・`has_start`・`has_end`・`content_width` の 5 個で、付録 B.5 の 5 本の marker が全部使われる。`root_ref` の marker 行と field は消す（残すと unused marker）。
   - `into_view` の `LayoutSlots { .. }` の literal は付録の手順 1 のとおり（`content_width: content_width.map(|width| ContentWidth { root: root_ref, width })`）。`let root_ref = node_ref();` と `root.node_ref(root_ref)` は残す。`node.base_style(move || layout_row_style(slots, root_ref))` は `node.base_style(move || layout_row_style(slots))` にする。
   - `region_content_style` は付録の手順 4・5 のとおり。let chain の中の `let Length::Px(width) = width.get()` は外側の `width` を shadow する（edition 2024 の let chain で compile する。既存コードも let chain を使っている）。
4. **W3（`TimestampEntry.copyable`）**：DEC-22・P8-O3 のとおり field を残し、marker は付録 B.5 の option の文にする。加えて、事実と違う doc「Whether the detail value offers a copy action.」を次にする（marker と doc が食い違わないため。公開 API の型は変えない）：
   `/// Reserved for a copy action on this detail row; the timestamp details do not draw one yet, so the value has no effect.`
5. **D8 5.2 の追加試験**（`crates/kivact-ui-soft/tests/consumer/navigation.rs` の末尾）：付録のコードは compile しない箇所が 2 つある。(a) `Locator::rect()` は `Option<Rect>` を返す（同じ consumer の `chat_scroll.rs`・`disabled_menu.rs` は `.rect().expect(..)`／`.unwrap()`）。(b) `soft::Space::S6.px()` は `use_tokens` を呼び、render の外（試験の本体）では theme の reactive scope が無い。期待値の s6 は theme から直接読む。次の全文で確定する（import は同じファイルの既存の `ThemeMode`・`RenderOptions`・`render_with`・`text`・`soft` を使う。新しい `use` は不要）：
```rust
#[test]
fn layout_padding_and_pixel_content_width_place_the_content() {
    let s6 = soft::theme(ThemeMode::Light)
        .ext::<soft::Tokens>()
        .spacing
        .s6;
    let screen = render_with(
        RenderOptions::with_theme(soft::theme(ThemeMode::Light)).window_size(800.0, 400.0),
        || {
            soft::layout()
                .padding(soft::Space::S6)
                .content_width(kivact::style::Length::Px(400.0))
                .content(soft::layout_content(text("Body text")))
        },
    );
    let body = screen
        .get_by_text("Body text")
        .rect()
        .expect("mounted body text");
    // The layout's own padding (not the region fallback) is above the content.
    assert!(body.y >= s6 - 0.5, "{body:?}");
    // A pixel content width without side panels is centred by a gutter.
    assert!(body.x >= (800.0 - 400.0) / 2.0 - 0.5, "{body:?}");
    assert!(body.x + body.width <= (800.0 + 400.0) / 2.0 + 0.5, "{body:?}");
    assert!(screen.errors().is_empty());
}
```
   - D8 5.2 の「先に HEAD で通ることを確かめる。通らなければ STOP」は消す。この全文を HEAD 2a3b06b8 に足して実行した結果：成功（2026-11-10、別 worktree で `dev/tools/window-lock.sh --timeout 1500 -- cargo test -p kivact-ui-soft --test consumer -- layout_padding_and_pixel layout_components_mount`：2 passed）。HEAD で通るので、P8-SOFT の W1・W2 の後も同じ期待値で通ることが振る舞い不変の担保になる。失敗したら W1・W2 の書換えの誤りとして直す（期待値は変えない）
   - 付録 A-G5 の「追加すべき」E2E（Card の中の Layout の padding、`start` がある時の row の gutter、`Length::Percent`）は足さない（P8-O5：6 章の表に書いたものだけ）。
6. **G9（W7）との関係（確定）**：P8-SOFT の文は G9 が改名する名前（`tab_list(binding)` の `binding`、`selection`、`options`、`open`/`close` の命令）を名指さない。`TabRuntime.behavior` の文の `TabsApi` と `TabPattern` は G9 の対象外。追従は不要。
7. **SO との関係（確定）**：SO（E-5）は `side_navigation/shell.rs` の `SidebarWidth` と `sidebar_width`・`resize_grip` だけを変え、`SideNavSlots` の 4 行（`header`・`top`・`footer`・`icons`）は変えない。P8-SOFT は SO の後のファイルで `SideNavSlots.<field>` を探して置き換える。
8. 完了時の確認は D8 6 章の行のとおり（`cargo clippy -p kivact-ui-soft --all-targets -- -D warnings`、`cargo check -p kivact-ui-soft-catalog`、`cargo xtask check --no-write --only state-fields --crate kivact-ui-soft`、`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-soft --test consumer -- layout_`）。`layout_` は `navigation::layout_components_mount_and_interact` と新しい試験の 2 本に当たる。
### 所有ファイルの変更
なし（D8 6 章の行のとおり。`layout.rs` は W1・W2 と G5 の KIND 行を同じ unit で書く）。
### wave・lane・順序の変更
なし。
### 提案（任意、方針 7）— R6 の統合で**不採用に確定**
- 理由：`LayoutInset` の field は pub で `Default` を持つので、kit の外の custom surface が `provide(LayoutInset { outer_x: Some(..), ..Default::default() }, ..)` で soft の Layout に余白を伝えられる（context の公開の拡張点）。可視性を縮めるとこの使い方が消えるため、V2 では pub のまま（上の 2 の確定形）とする。作業指示書 12.3 の文も変えない。以下は検討の記録。
- `LayoutInset` は kit の外から使う公開 API が無い（構築の `from_padding` は `pub(crate)`、受け取る公開関数も無く、repo 内の利用は soft の 5 ファイルだけ）。`pub struct` を `pub(crate) struct`（field も `pub(crate)`）にし、`lib.rs` の `pub use layout::{.., LayoutInset, ..}` から外せば、`own_outer` の名前は公開 API から消え、作業指示書 12.3 の「D8 の公開 API の変更 3 件」が 2 件になる。V2 の範囲で行うなら P8-SOFT の所有ファイルに `crates/kivact-ui-soft/src/lib.rs` を足し、12.3 の文から `LayoutInset.own_outer` を消す。行わない場合も設計の矛盾は無い（上の 2 のとおり pub のまま確定）。

---

## P8-KITS（付属書 D8 6 章の P8-KITS の行、付録 B の unit 列 P8-KITS の 60 行、W6 lane L6）
### 判定：変更なし（付録 B の文をそのまま置く。補足のみ）
### 修正（補足）
- 60 行とも付録 B の「新しい marker 行」の全文を置く。名指す method（`Attachment::into_view`、`Chart::{orientation, grid, axis}`、`Options::frame`、`QuestionnaireStep::optional`、`SidebarGroup::{action, content}`、`SidebarMenuButton::{icon, badge, action, sub}`、`Sidebar::{header, content, footer}`、`DockTab::closable`、`TimelineTrack::automation`、`Outliner::on_activate`、`TreeItem::{new, placeholder}`、`PickerItem::enabled`）は全て HEAD に実在する。
- `examples/kivact-ui-game/src/game/window.rs` の `frame::width` の文は「written only by the native_window divider …」を含むが、prefix は `independent:` なので X4（`projection:` だけを検査）の対象外。そのまま置く。
- EX との関係（確定）：EX は同じ `kivact-ui-basic/src/sidebar/menu.rs`（`ButtonCfg.has_action` の projection を `written only by build_button` に）と game の `window.rs`（E-3：`Run::message` を消し、`run`・`pause` の腕が `status` も書く）・`controls.rs`（E-10：`filter_on` の 2 行）を先に変える。P8-KITS の行（`SidebarGroup.*`・`SidebarMenuButton.*`、`frame::width`・`frame::help`、`navigation::step`・`page`）は別の field・local で、E-3 の後も `frame` の数える local は `width`・`help`（ほかは `signal(Some(..))`・`signal(String)`・`signal(Run::Idle)` で数えない）なので、marker の要否は変わらない。`ButtonCfg.has_action` の X4 は private struct の module 内の唯一の書き手 `build_button`（`let has_action = action.is_some();` の literal）で満たされる。
- G6（W5）との関係（確定）：G6 は `examples/kivact/component/src/main.rs` の `card` の `.to_owned()` だけを変える。P8-KITS の `app::first`・`app::second` の 2 行は変わらない。
- G9（W7）との関係：P8-KITS の文は G9 の改名対象（`gradient_editor(selection)` 等）を名指さない。`examples/kivact-ui-creative/src/creative/layout.rs` の `properties::tab` の文の「selection」は `inspector_tabs` の button 列（`tabs(..)` ではない）の選択を指す普通名詞で、追従は不要。
- 完了時の確認は D8 6 章の行のとおり。state-fields の `--crate` は `kivact-ui-basic`・`kivact-ui-creative`・`kivact-ui-basic-catalog`・`kivact-ui-creative-catalog`・`kivact-ui-game-catalog`・`kivact-ui-soft-catalog`・`kivact-example-component` の 7 package（package 名は各 Cargo.toml で確認済み）。
### 所有ファイルの変更
なし。
### wave・lane・順序の変更
なし。

---

## L6 後半の全体

### 所有ファイルの重なり（確認済み）
- L4（IN-1〜P8-IN）：kivact-interaction と `kivact-ui-basic/src/recipes_menu.rs`・`kivact-ui-soft/src/recipes_menu.rs`。本書の 4 unit と重ならない。
- L5（UI-1・UI-2・UI-3・P8-CK・P8-UI5）：kivact-ui の `values.rs`・`toggle*.rs`・`radio_group.rs`・`look*.rs`・`data_table/*`・`date_input/view.rs`・`code_*`・`alert.rs`・`calendar_values.rs`・`component_values.rs`・`card.rs`・`lib.rs`、soft の `card.rs`・`banner.rs`・`recipes_controls*.rs`・`segmented_control.rs`・`indicator.rs`、basic・pop・game・creative の recipes 系。本書の 4 unit（P8-UI6 の `date_input/machine.rs`・`data_table_tokens.rs`・`group_types.rs` を含む）と重ならない。P8-SOFT の提案（`soft/src/lib.rs`）を採っても L5 は soft の `lib.rs` を持たないので重ならない。
- L6 前半（UI-4・SO・CR・EX）：同じ lane の中で前に行うので、重なり（`carousel.rs`・`combobox/data.rs`・`navigation_values.rs`・`side_navigation/shell.rs`・`sidebar/menu.rs`・game の `controls.rs`・`window.rs`）は順序で解決済み。field は重ならない（上の各節）。
- W5：G6（`examples/kivact/component/src/main.rs`）、G7（`calendar/view.rs`）が先。本書の対象の行を変えない。
- W7：FINAL-1（PartState）は本書のファイルの読み手を compile error で直すだけで、本書の marker と field を変えない。G9 の改名は、P8-UI6 の 3 行を上のとおり「items」で書けば、本書の文に追従は要らない。X4 の `written only by <fn>` で G9 が改名する関数を名指す行は、本書の範囲に 0 件。
- FINAL-2（X3〜X5）：本書の範囲で X3 の OPEN（印付き 1＋無印 1）になる struct・関数は無い（付録 B の行の owner は全 field に marker がある。`ContentWidth` は数える field 0）。X4 の対象 0、X5 の重複は DT の 1 種だけで DT が解消する。

### 人間にしか決められない項目
なし（`LayoutInset` の可視性の縮小は任意の提案で、行わなくても設計は確定している）。


---

# R6 監査：FINAL-2（xtask X3〜X5）

対象：作業指示書 10 章 FINAL-2、付属書 D5 3.4（X3〜X6）・3.7・6 章 FINAL-2 の行、D8 6 章 FINAL-2 の行。HEAD 2a3b06b8。

検証の方法（静的＋試作）：detached worktree（削除済み）に X3・X4・X5 を下の確定コードで実装し、HEAD に対して `cargo xtask check --no-write --only state-fields`、`cargo test -p xtask`、`cargo clippy -p xtask --all-targets -- -D warnings`、`cargo xtask check --no-write` を実行した。結果：
- xtask の試験 74 件すべて成功（既存の試験の期待値は変わらない。新しい試験 9 件も成功）。clippy 警告 0。行数・`Role:` の規則の error 0（fmt の差分だけ。`cargo fmt` で消える）。xtask 自身の source に state-fields の error は出ない（`claims.rs` の struct はどれも数える field が 1 個以下）。
- HEAD での error：X3 で 28 件（D5 3.1 の OPEN 25 件と W1 の 3 件。GPU の 2 件は G1 で消えた。28 件はどれも HEAD の audit の OPEN 行と一致）、X4 で 107 件、X5 で 44 件。これらを付属書（D5 本文・付録、D8 付録 B）の書換えと 1 件ずつ突き合わせ、W6 の後に残るものを下の「W6 の後に残る error と直し」に挙げた。

## FINAL-2（付属書 D5 3.4・3.7・6 章、D8 6 章、W7）

### 判定：修正あり

### 修正（作業指示書 10 章 FINAL-2 の行の「本書の追加の指示」にそのまま入れる文、または付属書 D5 3.4 の差し替え）

#### 1. X3（`check.rs`、`locals.rs`）— 付属書のコード片を次で置き換える

- `Counting` から `phase` を**消し**、`total` を足す（付属書は `phase` を残しているが、X3 の後は誰も読まないので `dead_code` 警告になり clippy `-D warnings` が失敗する。`rows()` は `field.class` を見るので `phase` は要らない）：

```rust
/// The counted state of one struct, variant or static group.
struct Counting {
    stored: Vec<String>,
    /// Every counted field, marked or not (Stored, promoted Helper, Phase).
    total: usize,
    counted: Vec<Counted>,
}
```

- `count_fields`：`let mut phase = 0;` を `let mut total = 0;` に。`Class::Phase => { phase += 1; true }` を `Class::Phase => true,` に。`if counted_field {` の直後（`if let Some(line) = marker` の前）に `total += 1;`。最後の `Counting { stored, phase, counted }` を `Counting { stored, total, counted }` に。
- `check_struct` と `check_enum` の `report_combination(.., &counting.stored, counting.phase, errors)` の 2 箇所を `counting.total` に。
- `report_combination`（引数 6 個、上限 7 の内）：

```rust
fn report_combination(
    context: &RuleContext<'_>,
    span: proc_macro2::Span,
    name: &str,
    stored: &[String],
    total: usize,
    errors: &mut Vec<String>,
) {
    // A counted field without a closure beside any other counted field is an
    // implicit combination: the audit's OPEN row (design 6.4).
    if !stored.is_empty() && total >= 2 {
        context.source.error_at(
            RULE,
            span,
            &format!(
                "{name} combines state fields: {} (close each with a type, a derivation, `projection:` or `independent:`)",
                stored.join(", ")
            ),
            errors,
        );
    }
}
```

- 境界の確認（設計の変更なし）：新しい条件は audit の OPEN 行の条件（`counted.len() >= 2` かつ無印で Phase でない field がある）と同値。旧条件 `stored.len() >= 2 || (phase > 0 && !stored.is_empty())` は全て含まれる。Phase だけ 2 個（stored 0）は旧・新とも error なし。Helper は Stored が 1 個以上あるときだけ数える（promote）ので、「印付き bool ＋ 無印 `Sent`」は新しく error になる（audit でも OPEN なので正しい。HEAD に該当なし）。
- static group（`report_statics`）と exempt struct の扱いは**変えない**。static は audit の行を作らないので gate と audit の一致に影響しない。exempt struct は `count_fields` を通るので X3 がそのまま当たる。
- `locals.rs` `check`：付属書どおり `if open.is_empty() || function.locals.len() < 2 { continue; }`。
- 試験 2 本（付属書の 2 本、`tests.rs` の末尾）。下の「確定した試験」の最初の 2 本。

#### 2. X4・X5（新ファイル `dev/xtask/src/state_fields/claims.rs`）— 付属書の骨組みを次の確定版で置き換える

付属書の骨組みとの違い（すべて確定）：
1. 署名は本文の形 `pub(crate) fn verify(all: &[Source], report: &dyn Fn(&Source) -> bool, errors: &mut Vec<String>)`。骨組みの `verify(sources: &[&Source], ..)` は使わない。
2. `--crate`・`--module` を付けたとき：
   - X4 の claim（marker）は `report` が選んだ source の field だけから集める。書込みと関数の実在は、field の可視範囲にある**全ての**読込み済み source（`load` は常に workspace 全体を読む）で調べる。可視範囲が package（正確には Cargo target。`identity_root()` が同じもの）全体なので、`--crate` でも結果は全体の実行と同じ。
   - X5 は全 workspace の行で重複を数え、`report` が選んだ source の行にだけ error を出す。`--crate X` の実行でも、他の crate の同じ文との重複は X の行の error になる（X5 は FINAL-2 で初めて入るので、W6 の lane の確認には影響しない）。
3. enum の variant の field は、field の可視性（構文上は常に private）ではなく **enum の可視性**で可視範囲を決める。これが無いと B3 の `State::Open.previewed`（`pub(super) enum State`、`select/machine.rs`）の書き手 `track_preview`（`select/handle.rs`）が範囲外になり、X4 rule 2 の error になる。
4. 「空の初期値」に 1 形を足す：`{ let mut x = <空>; x.m(<空>, ..); x }`（`let` の初期値と、局所変数への method 呼出しの引数がすべて空の初期値で、最後が path のブロック）。HEAD の変更しない projection `Model.modifiers`（`model/aggregate.rs` の `Model::new`）と `Pointer.hover_sent`・`position_sent`（`model/pointer/state.rs` の `impl Default for Pointer`）は `{ let mut sent = Sent::new(); sent.update(<既定値>); sent }` で初期化しており、この形が無いと X4 の error になる（D5 3.5 は「変更なし」としているので、lint 側で受ける）。
5. owner の struct literal の path が `Self`（または `Self::Variant`）のときは、囲む `impl` の self 型で owner を判定する。
6. inline の `mod x { .. }` の中身は、親の source では辿らない（子の module が独自の `Source` として読まれるため。二重に数えない）。
7. `Claim` の `variant: bool` は使わず、owner を `enum Owner { Struct(String), Variant { enumeration, variant } }` で持つ。`Visible` の variant 名は `Package` でなく `Target`（`identity_root()` 単位）。

確定コード（`cargo fmt` 済み、clippy 済み、659 行。全文をそのまま置く）は付録 A。

`dev/xtask/src/state_fields.rs`：`mod check;` の次に `mod claims;`、`pub(crate) use check::check;` の次に `pub(crate) use claims::verify;`。module doc の最後の行 `//! The rule closes the known gaps (SL-1 to SL-8) for every crate.` の次に 1 行：
`//! A projection marker names its one writer and an independent reason is not shared with another file (SL-9).`

`dev/xtask/src/local_checks.rs` `Scope::run`：source の `for` の閉じ括弧の直後（rule の `for` の中）に：

```rust
            if rule == "state-fields" {
                crate::state_fields::verify(
                    self.sources,
                    &|source| selected_source(source, options),
                    errors,
                );
            }
```

（`Scope::run` は 80 行を超えない。clippy `too_many_lines` で確認済み。）

P8-XT（W6 L1）との関係：P8-XT は `Source.target`/`library` を `enum Target` と accessor `library()` にし、`state_fields/check.rs:82` を `source.library()` に変える。FINAL-2 のコードは `Source` の `module`・`path`・`text`・`items`・`owns_line`・`allow_line`・`allow_reason`・`error_at_line`・`identity_root()` だけを使い、`target`・`library` に触れないので、P8-XT の前後どちらの API でもそのまま compile する。試験の `fixture` は P8-XT の後も `module`・`path` を書き換えられる（どちらも `pub(crate)`）。

#### 3. 確定した試験（`dev/xtask/src/state_fields/tests.rs` の末尾に追加。付属書 3.4 の 2＋5 本を、実際の `fixture` で書ける形にし、Machine の例外と X5 の 2 本を足した 9 本）

付録 B の全文をそのまま末尾に足す。`fixture` は `crate::check_source::fixture`（既に import 済み）。

#### 4. 確認（作業指示書 10 章 FINAL-2 の行の「確認」を次で置き換える。STOP にしない）

```bash
cargo test -p xtask state_fields
cargo clippy -p xtask --all-targets -- -D warnings
git grep -n "xtask-allow(state-fields): projection:" -- '*.rs' ':!dev/xtask'
cargo xtask check --no-write --only state-fields; echo "exit=$?"
```

- 3 行目の結果は次の表と同じ行（file と、直後の field）でなければならない。M6・M8 の 2 行は、その unit が付属書 D3 のとおり入れた場合だけある。

| file | owner.field | 名指す書き手 |
|---|---|---|
| `crates/kivact-core/src/gpu_interop/share/app_side.rs` | SharedImage.current | SharedImage::replace_current（CO） |
| `crates/kivact-core/src/view/semantics_data.rs` | SemanticsPatch.reactive | Common::semantic（P8-CO） |
| `crates/kivact-core/src/view/ref_data.rs` | NodeRefState.observed | NodeRef::require_observation（M8） |
| `crates/kivact-core/src/view/mount/style.rs` | InStylePaint.drawn | InStylePaint::record（M6） |
| `crates/kivact-engine/src/host/pump.rs` | Host.app_motion | Host::report_app_motion（EN-2） |
| `crates/kivact-engine/src/host/window.rs` | HostWindow.environment | Host::record_environment（EN-2） |
| 同 | HostWindow.soft_keyboard、ime_allowed | Host::synchronize_ime |
| 同 | HostWindow.adapter | Host::adapter_changed |
| `crates/kivact-engine/src/model/aggregate.rs` | Model.modifiers | Model::report_modifiers |
| `crates/kivact-engine/src/model/pointer/state.rs` | Pointer.hover_sent、position_sent | Model::report_hover、Model::report_mouse_position |
| `crates/kivact-engine/src/measure/editor.rs` | EditorNode.reported_session | Model::report_sessions（EN-1） |
| `crates/kivact-interaction/src/select/machine.rs` | State::Open.previewed | track_preview（IN-1） |
| `crates/kivact-runtime/src/task/action.rs` | ActionCell.last | ActionCell::begin_run（RT） |
| `crates/kivact-runtime/src/task/async_state.rs` | AsyncCell.latest | AsyncCell::publish |
| `crates/kivact-test/src/facts.rs` | Node.kind、Node.parent | Facts::new（P8-KT） |
| `crates/kivact-ui-basic/src/sidebar/menu.rs` | ButtonCfg.has_action | build_button（EX） |

- 表に無い行が出たら：その行の file を最後に変えた W6 の unit（`git log -1 --format=%s -- <file>`）の付属書の文（D5 3.5・付録、D8 付録 B、本監査の申し送り）を開き、その行を付属書の文に直す。付属書がその field を projection にしていない（option・independent・型・削除）ので、付属書の文に置き換えれば消える。これは機械的な追従で、同じ unit（FINAL-2）の中で行ってよい（作業指示書 0.2 の 12 の例外として、この 1 種類だけ他 crate の marker 行の置換を許す）。
- 4 行目は `exit=0` で、error が 0 行であること。error が出たら次の表で扱う（STOP にしない）。

| error の文言 | 扱い |
|---|---|
| `<Owner> combines state fields: <f>`（X3） | 前の unit が付属書の marker を落とした。その file の付属書の文（D5・D8 付録 B）をその field の直前に置く。付属書に文が無い field なら `dev/bench/results/OPEN.md` の「V2 差戻し」に 1 行書いて残りを続ける（W8 で完了を宣言しない） |
| `independent: reason repeats in <path>`（X5） | 両方の行を、それぞれの付属書の文に戻す（付属書の文はファイルをまたいで重複しないことを本監査で確かめた）。付属書の文どうしが重複していたら差戻し |
| `projection marker must name one writer`、`projection writer .. does not exist`、`.. is also written in ..`（X4） | 上の表に無い行なら、表の手順で付属書の文に直す。表の行なら差戻し（本監査で書込みの位置を確かめたので、出るのは前の unit が書き手を集めなかった場合だけ：その unit の付属書の手順の書き手の集約を行う） |

- 続けて D5 3.7 の「OPEN 0 の確認」の 4 行を実行する（`exit=0`、`grep -c '| OPEN |'` が 0）。X3 の後は state-fields の error 0 と OPEN 0 は同値。最後に `git checkout -- dev/xtask/state-audit.md` で戻してから commit する（作業指示書 10 章のとおり）。

### 所有ファイルの変更

- FINAL-2：`dev/xtask/src/state_fields/check.rs`、`locals.rs`、`claims.rs`（新）、`tests.rs`、`dev/xtask/src/state_fields.rs`、`dev/xtask/src/local_checks.rs`（付属書どおり。変更なし）。上の「表に無い行」の置換に限り、その行の file。

### wave・lane・順序の変更

- なし（FINAL-1 → G9 → FINAL-2 のまま）。前提：下の「他 unit への申し送り」がすべて行われていること。

## W6 の後に残る error と直し（他 unit への申し送り。親が該当 unit の行に足す）

### X3

HEAD の 28 件の行き先：D5 3.1 の 25 件は W6 の各 unit（#1 NA、#2・#25 XT-1、#3 CO、#4 ED、#5〜#10 EN-1、#11〜#14 IN-1、#15 LY、#16〜#19 RT、#20 ST、#21〜#23 UI-2、#24 CR）が閉じる。残る 3 件は W1 の F5・M7・T8 の返戻し（`arvane-compose/src/cache.rs` `Indexed`、`kivact-core/src/view/mount/semantics.rs` `Plan`、`kivact-runtime/src/reactive/data.rs` `OwnerEntry.extra`）。この 3 件は X3 が無くても HEAD の gate の error で、FINAL-2 の前（CP1 の gate）に、その unit の担当の監査が確定した閉じ方で閉じていること（FINAL-2 の前提）。参考：`Indexed` は書き手 `plan/damage.rs` `indexed` が `bounds` と `group` の片方だけを Some にする排他（型で閉じられる）。

W2〜W5 の付属書（D1〜D4・D6・D7）と D5・D8 のコード片の struct を全て走査し、「印付き 1 ＋ 無印 1」になるものを探した。X3 に当たるものは無い。ただし次の 3 件は付属書どおりに書くと**既存の規則**（unused marker・2 個の無印）で、その unit の確認の時点で error になる：

| unit | 付属書 | 問題 | 直し（その unit の行に足す文） |
|---|---|---|---|
| G3（W2 lane D） | D6 3.5 `gpu_interop/import/registry.rs` `State.asked` | `RefCell<BTreeSet<WindowId>>` は lint が数えない型（Option・bool・整数を包まない、名前も `pending` 等でない）。付属書の `independent:` marker は `unused marker` の error になる | 「`State.asked` の直前に marker を置かない（数えない field）。doc comment `/// The windows whose GPU device the App asked for, once each.` だけを置く」 |
| T2（W3 lane A） | D7 3.0 `kivact_types::cell::Recent` の `last: usize` | `last` は数える名前の整数ではない（`next` だけが数える）。付属書の `cache:` marker は `unused marker` の error | 「`Recent.last` の直前の marker 行を置かない。`next` の marker（independent）は残す」 |
| T6（W3 lane D） | D7 3.5 `CoreNode` のコード片 | コード片は `at`・`handlers` の直前の marker を省いている。HEAD の 2 本（`independent: a per-node resource; created only when the node has one`）を落とすと `at`・`handlers` が無印 2 個で error | 「`CoreNode.at`・`handlers` の直前の HEAD の marker 行はそのまま残す（付属書のコード片は省略して書いている）」 |

### X4

W6 の後に残る projection は上の表の 19 行（M6・M8 を含めて最大 21 行）。各行について、名指す関数の実在と、可視範囲の全ての書込みがその関数の中にあることを、HEAD のコードと付属書の書換えから確かめた（app_motion・environment・SharedImage・ActionCell は付属書の集約の後、facts の Node は `kivact-test` の全 source で `Node {` の literal と `.kind`/`.parent` への書込みが `Facts::new` だけであること、`SemanticsPatch.reactive` は `kivact-core` で `SemanticsPatch {` の literal が `Common::semantic` だけで `.reactive` への代入が無いこと、`NodeRefState.observed` は `kivact-core` で `.observed` への書込みが無いこと）。G9 の改名（`MenuApi::show/open/close`、`ComboboxApi::open/close`、`InlineEditApi::open`、`.selection(..)`）を名指す projection は無い（`MenuItemState` は IN-2 の E-2 で型になり、`InlineEdit.reported` は IN-2 の E-4 で消える）。

付属書が拾っていない 1 件（**孤児**。どの unit も持っていない）：

| file | 現在の marker | 問題 | 直し |
|---|---|---|---|
| `crates/kivact-interaction/src/calendar/data.rs`（`CalendarDayState.disabled`） | `// xtask-allow(state-fields): projection: readonly snapshot of the calendar policy for recipes, written only by the cell part` | D5 付録 P5-E 4 節の snapshot 一覧に入っているが、E-U6 は「OPEN 担当の unit と同じファイルなのでその unit に統合」としており、`calendar/data.rs` を持つ unit が無い。X4 で `projection writer the of CalendarDayState.disabled does not exist` の error になる。書き手も `calendar/handle.rs` の 2 箇所（`:80`、`:190` 付近）で、projection 自体が偽 | **IN-3（L4）**の所有ファイルに `crates/kivact-interaction/src/calendar/data.rs` を足し、手順に「`CalendarDayState.disabled` の直前の marker 行を削除する（`CalendarDayState` で数える field は `disabled` の 1 個だけ。`input: PartState`・`selection: DaySelection`・`span: CalendarSpan` は数えない型なので、marker は不要）」を足す（本書 L4 の IN-3 の節に確定済み。P8-IN ではなく IN-3 が行う）。確認は IN-3 の state-fields `--crate kivact-interaction` |

CO（L3）への条件：G3（W2）が `gpu_interop/share/app_side.rs` を書き換えるので、CO は「G3 の後のコードで、`self.current.set(..)` の全ての呼出し（G3 が足したものを含む）を `replace_current` 経由にする」（X4 は `current` への書込みが `replace_current` の外に 1 つでもあれば error）。

### X5

HEAD でファイルをまたいで重複する `independent:` の理由は 12 種・44 行（付属書の「13 種」は数え方の違いで、行の集合は同じ）。全ての行が D5 の書換え（付録 P5-F 2 章 (b)、本文 3.5 の 3 行、P5-C C2、P5-D F-8）の対象で、各行の新しい文はファイルをまたいで重複しない。D8 付録 B の新しい文（WORDING・KIND の 378 行）と、変更しない HEAD の文、D5 の新しい文、D3・D6・D7 が足す文を合わせて調べ、W6 の後にファイルをまたぐ重複は 0（同じファイルの中の共有だけ：`ButtonState`/`ToggleButtonState`/`CardState` の E-1・E 4 節の文、`pointer/state.rs` の A6 の文、`view/tree.rs` の per-node resource、`kivact-ui-creative/src/audio/transport.rs` と example の文は EX が example 側を変えるので別）。D3 の `Recent` の文（`model/recent.rs`）と D7 の `Recent` の文（`kivact_types::cell`）は同じ文だが、T2 が移す（前の file から消す）ので重複しない（T2 の確認で `model/recent.rs` が残っていないこと）。

`dev/xtask/src/state_fields/tests.rs` の fixture の文字列（`"struct Facts {\n// xtask-allow(..."`）は、行頭（`trim_start()`）が marker でないので X5 は拾わない。`.md` の文書は `Source` として読まれないので拾わない（試作の実行で確認）。

## 付録 A：`dev/xtask/src/state_fields/claims.rs`（確定版の全文）

```rust
//! Role: Verify what state-field markers claim across the sources a field is visible in.

use crate::check_source::Source;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use syn::spanned::Spanned;
use syn::visit::Visit;

/// The rule name reported by these checks.
const RULE: &str = "state-fields";
/// The marker that explains a field.
const MARKER: &str = "// xtask-allow(state-fields):";
/// The phrase that names a projection's one writer (SL-9).
const WRITER: &str = "written only by ";
/// Method calls that write the cell or collection they are called on.
const WRITE_METHODS: [&str; 10] = [
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
];
/// Method calls that reach the field they are called on.
const THROUGH_METHODS: [&str; 7] = [
    "borrow_mut",
    "get_mut",
    "as_mut",
    "lock",
    "write",
    "unwrap",
    "expect",
];

/// One `projection:` marker and the writer it names.
struct Claim<'a> {
    source: &'a Source,
    line: usize,
    /// The struct, or `Enum::Variant`, whose field the marker explains.
    owner: Owner,
    field: String,
    visible: Visible,
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

/// The sources a field can be written from.
enum Visible {
    /// The module and its descendants, within one target.
    Module(String),
    /// Every module of the target.
    Target,
}

/// The function a projection names: `fn` or `Type::fn`.
#[derive(Clone)]
struct Writer {
    ty: Option<String>,
    name: String,
}

/// One write of a field name, with the function it is in.
struct Write {
    path: PathBuf,
    line: usize,
    /// The innermost function and its impl's self type, if any.
    function: Option<(String, Option<String>)>,
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

/// The claim of one field's marker, when it is a well-formed `projection:`.
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
    Some(Claim {
        source,
        line,
        owner: owner.clone(),
        field: name,
        visible: visible(source, visibility),
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

/// Where a field with `visibility`, declared in `source`, can be written.
fn visible(source: &Source, visibility: &syn::Visibility) -> Visible {
    let module = source.module.clone();
    let parent = |module: &str| {
        module
            .rsplit_once("::")
            .map_or(String::new(), |(parent, _)| parent.to_owned())
    };
    match visibility {
        syn::Visibility::Inherited => Visible::Module(module),
        syn::Visibility::Restricted(restricted) => {
            let path: Vec<String> = restricted
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            match path.first().map(String::as_str) {
                Some("crate") if path.len() == 1 => Visible::Target,
                Some("crate") => Visible::Module(path[1..].join("::")),
                Some("self") => Visible::Module(module),
                Some("super") => {
                    let mut base = module;
                    for segment in &path {
                        if segment == "super" {
                            base = parent(&base);
                        }
                    }
                    Visible::Module(base)
                }
                _ => Visible::Target,
            }
        }
        syn::Visibility::Public(_) => Visible::Target,
    }
}

impl Visible {
    fn contains(&self, declared: &Source, source: &Source) -> bool {
        if source.identity_root() != declared.identity_root() {
            return false;
        }
        match self {
            Self::Target => true,
            Self::Module(module) => {
                module.is_empty()
                    || source.module == *module
                    || source.module.starts_with(&format!("{module}::"))
            }
        }
    }
}

/// Rules 2 and 3 of SL-9 for one claim.
fn verify_claim(all: &[Source], claim: &Claim<'_>, errors: &mut Vec<String>) {
    let visible: Vec<&Source> = all
        .iter()
        .filter(|source| claim.visible.contains(claim.source, source))
        .collect();
    let label = format!("{}.{}", claim.owner.label(), claim.field);
    let writer = match &claim.writer.ty {
        Some(ty) => format!("{ty}::{}", claim.writer.name),
        None => claim.writer.name.clone(),
    };
    if !visible.iter().any(|source| defines(source, &claim.writer)) {
        claim.source.error_at_line(
            RULE,
            claim.line,
            &format!(
                "projection writer {writer} of {label} does not exist where the field is visible"
            ),
            errors,
        );
        return;
    }
    let writes: Vec<Write> = visible
        .iter()
        .flat_map(|source| writes(source, &claim.owner, &claim.field))
        .collect();
    let machine = matches!(claim.owner, Owner::Variant { .. })
        && writes.iter().all(|write| {
            write
                .function
                .as_ref()
                .is_some_and(|(name, _)| name == "transition")
        });
    if machine {
        return;
    }
    for write in writes {
        if is_writer(&write, &claim.writer) {
            continue;
        }
        let function = write
            .function
            .as_ref()
            .map_or("no function", |(name, _)| name.as_str());
        claim.source.error_at_line(
            RULE,
            claim.line,
            &format!(
                "{label} is also written in {function} at {}:{}; a projection has one writer ({writer})",
                write.path.display(),
                write.line
            ),
            errors,
        );
    }
}

fn is_writer(write: &Write, writer: &Writer) -> bool {
    write.function.as_ref().is_some_and(|(name, ty)| {
        *name == writer.name
            && writer
                .ty
                .as_ref()
                .is_none_or(|expected| ty.as_ref() == Some(expected))
    })
}

/// Whether `source` defines the function `writer` names.
fn defines(source: &Source, writer: &Writer) -> bool {
    let mut finder = Functions {
        writer,
        impls: Vec::new(),
        found: false,
    };
    for item in &source.items {
        finder.visit_item(item);
    }
    finder.found
}

/// Finds the definition of one named function.
struct Functions<'a> {
    writer: &'a Writer,
    impls: Vec<Option<String>>,
    found: bool,
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
            self.found = true;
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
            self.found = true;
        }
        syn::visit::visit_impl_item_fn(self, node);
    }
}

/// The writes of `field` of `owner` in `source`.
fn writes(source: &Source, owner: &Owner, field: &str) -> Vec<Write> {
    let mut visitor = Writes {
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

/// Collects the writes of one field name.
struct Writes<'a> {
    source: &'a Source,
    owner: &'a Owner,
    field: &'a str,
    functions: Vec<(String, Option<String>)>,
    impls: Vec<Option<String>>,
    found: Vec<Write>,
}

impl Writes<'_> {
    fn record(&mut self, line: usize) {
        if self.source.owns_line(line) {
            self.found.push(Write {
                path: self.source.path.clone(),
                line,
                function: self.functions.last().cloned(),
            });
        }
    }

    /// Whether a struct literal's path names the owner.
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
}

impl<'ast> Visit<'ast> for Writes<'_> {
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
            for value in &node.fields {
                if matches!(&value.member, syn::Member::Named(name) if name == self.field)
                    && !is_empty_initial(&value.expr)
                {
                    self.record(value.span().start().line);
                }
            }
        }
        syn::visit::visit_expr_struct(self, node);
    }

    fn visit_expr_assign(&mut self, node: &'ast syn::ExprAssign) {
        if target_field(&node.left).is_some_and(|name| name == self.field) {
            self.record(node.span().start().line);
        }
        syn::visit::visit_expr_assign(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        if is_compound_assignment(&node.op)
            && target_field(&node.left).is_some_and(|name| name == self.field)
        {
            self.record(node.span().start().line);
        }
        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method = node.method.to_string();
        if WRITE_METHODS.contains(&method.as_str())
            && target_field(&node.receiver).is_some_and(|name| name == self.field)
        {
            self.record(node.span().start().line);
        }
        syn::visit::visit_expr_method_call(self, node);
    }
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

/// The field an assignment or a write method reaches: the last field access
/// after unwrapping parentheses, borrows, derefs and THROUGH_METHODS.
fn target_field(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Paren(inner) => target_field(&inner.expr),
        syn::Expr::Group(inner) => target_field(&inner.expr),
        syn::Expr::Reference(inner) => target_field(&inner.expr),
        syn::Expr::Unary(inner) if matches!(inner.op, syn::UnOp::Deref(_)) => {
            target_field(&inner.expr)
        }
        syn::Expr::MethodCall(call)
            if THROUGH_METHODS.contains(&call.method.to_string().as_str()) =>
        {
            target_field(&call.receiver)
        }
        syn::Expr::Field(field) => match &field.member {
            syn::Member::Named(name) => Some(name.to_string()),
            syn::Member::Unnamed(_) => None,
        },
        _ => None,
    }
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

## 付録 B：`dev/xtask/src/state_fields/tests.rs` の末尾に足す試験（全文）

```rust
#[test]
fn one_unmarked_field_beside_a_marked_one_is_an_error() {
    let errors = run(
        "struct Probe {\n// xtask-allow(state-fields): independent: probe fact\na: bool,\nb: Option<Job>,\n}",
        "",
        &[],
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("Probe combines state fields: b"),
        "{errors:?}"
    );
}

#[test]
fn one_unmarked_local_beside_a_marked_one_is_an_error() {
    let text = "fn sample() {\n// xtask-allow(state-fields): independent: probe fact\nlet a = signal(false);\nlet b = signal(None);\n}";
    let errors = run(text, "", &[]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("function combines state locals: b"),
        "{errors:?}"
    );
}

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

#[test]
fn projection_needs_an_existing_single_writer() {
    let text = projected("written only by S::publish", "")
        + "impl S { fn publish(&self) { self.a.set(true) } fn other(&self) { self.a.set(false) } }";
    let errors = verify_sources(&[("widgets", &text)]);
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
    let errors = verify_sources(&[("widgets", &text)]);
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
    let errors = verify_sources(&[("widgets", &text)]);
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn projection_marker_must_name_one_writer() {
    let text = projected("readonly snapshot", "");
    let errors = verify_sources(&[("widgets", &text)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("must name one writer"), "{errors:?}");
}

#[test]
fn private_projection_fields_are_checked_in_their_module_only() {
    let other = "struct T { a: Cell<bool> }\nfn touch(t: &T) { t.a.set(true) }";
    let private = projected("written only by S::publish", "")
        + "impl S { fn publish(&self) { self.a.set(true) } }";
    let errors = verify_sources(&[("widgets", &private), ("other", other)]);
    assert!(errors.is_empty(), "{errors:?}");
    let shared = projected("written only by S::publish", "pub(crate)")
        + "impl S { fn publish(&self) { self.a.set(true) } }";
    let errors = verify_sources(&[("widgets", &shared), ("other", other)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("S.a is also written in touch"),
        "{errors:?}"
    );
}

#[test]
fn machine_variant_projections_name_the_event_sender() {
    let machine = "pub(super) enum State {\nOpen {\n// xtask-allow(state-fields): projection: written only by track_preview through Event::Preview\npreviewed: Option<u8>,\n// xtask-allow(state-fields): independent: probe hovered\nhovered: Option<u8>,\n},\n}\nfn transition(state: &State) -> State { State::Open { previewed: Some(1), hovered: None } }";
    let handle = "fn track_preview() {}";
    let errors = verify_sources(&[("select::machine", machine), ("select::handle", handle)]);
    assert!(errors.is_empty(), "{errors:?}");
    let errors = verify_sources(&[("select::machine", machine)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("does not exist"), "{errors:?}");
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


---

# 全体（R6 の担当範囲：W6 L4〜L6、W7 FINAL-2）

## 1. wave・lane・所有ファイルの組み直し（確定）

lane の中の unit の順は、作業指示書 9 章のまま変えない。変えるのは所有ファイルだけで、下の表のとおりにする。

| lane | unit | 所有から外す | 所有に足す | 理由 |
|---|---|---|---|---|
| L4 | IN-3 | — | `crates/kivact-interaction/src/calendar/data.rs` | どの unit も持っていない projection の marker（`CalendarDayState.disabled`）を消すため。消さないと X4 の error になる |
| L5 | UI-1 | `crates/kivact-ui/src/data_table/types.rs`・`builder.rs`・`runtime.rs`、`date_input/view.rs`、`date_range_input.rs` | `crates/kivact-ui/src/alert.rs`（AlertState の書き手の 2 行だけ） | E-8 と E-9 は行わない。types.rs の marker は P8-UI5 にまとめる。AlertState の型を変える unit が書き手も直す |
| L5 | UI-3 | `crates/kivact-ui/src/toggle.rs`、`radio_group.rs`、`crates/kivact-ui-soft/src/indicator.rs`、`recipes_controls_choice.rs` | — | これらは F-T4 だけの所有だった。F-T4 は行わない（DEC-15）。CheckableState は P8-CK だけが変える |
| L5 | P8-UI5 | — | （`data_table/types.rs` は元から所有している）E-6 の DataTableContext と `tree_expand_all` の marker を UI-1 から引き取る | types.rs を一つの unit にまとめる |
| L6 | CR | — | `crates/kivact-ui-creative/src/editors/shared/region_parts.rs` | D5 P5-C の「既存欠陥 5」の推測を確定する。死んだ field の `RegionInfo.selected` と、marker 2 本を消す |
| L6 | EX | `examples/kivact-ui-creative/src/creative/jobs.rs` | — | E-7 は取り消す（clock と machine が互いを参照する循環になる） |

重なりの確認：
- 足したファイル（`calendar/data.rs`、`alert.rs`、`region_parts.rs`）は、W6 の他の lane も、W5（G5〜G8）と W7（FINAL-1、G9）の所有も持っていない。
  - FINAL-1 と G9 の所有は「compile error が示す全ての file」で、この 3 ファイルは PartState の field も G9 の改名対象も読まない。
  - `alert.rs` は L5 の中だけで重なり、UI-1 → UI-2 の順で解決する。
- lane の間で重なるファイルは無い。L4 は kivact-interaction と、basic・soft の `recipes_menu.rs`。L5 と L6 の分担は各 lane の「全体」節に書いた。
- 同じ lane の中の重なりは、どれも前の unit の commit の上で別の field・別の行を変えるだけである。
  - L5：`lib.rs`、`values.rs`、`look.rs`、basic と creative の `recipes.rs`、`code_block.rs`、`data_table/rows.rs`
  - L6：`carousel.rs`、`combobox/data.rs`、`navigation_values.rs`、`side_navigation/shell.rs`、`sidebar/menu.rs`、game の `controls.rs`・`window.rs`
- W5 の G7 は、IN-2 の `menu/handle.rs` と P8-UI6 の `calendar/view.rs` を先に変える。G7 が変える hunk（`trigger_action` と `on_commit` の署名）と W6 が変える行は重ならない。

## 2. 作業指示書への反映（Orchestrator が W6 を始める前に行う）

- **9 章の L4〜L6 の各 unit の行**：「本書の追加の指示」に本書の各節の「修正」を入れる。所有ファイルは上の表で直す。
- **9 章の「UI-3 は F-T4 を行わない」の行**には、次の文を足す：「UI-3 は `CheckableState`（型、marker 3 本、`impl PartialEq`）、`toggle.rs`、`radio_group.rs`、soft の `indicator.rs`・`recipes_controls_choice.rs`、game の `recipes_controls.rs` の check の部分に触れない。」
- **10 章の FINAL-2 の行**：
  - 確認の「error が出たら付属書の誤りなので STOP」は、本書 FINAL-2 の節の「4. 確認」の表に置き換える（STOP にしない）。
  - FINAL-2 が使う claims.rs と試験は、本書 FINAL-2 の付録 A・B の全文を使う。
- **12.3 の PR の下書き**：「recipe state の accessor 化と `CheckControl`」を次の文にする。
  「recipe state の型と accessor の統一（`PartState` の `Availability` と `is_pressed()`／`is_hovered()`／`is_disabled()`／`is_hidden()`、`ButtonState`／`ToggleButtonState` の `label_hidden` と `is_icon_only()`、`MenuItemState.display: MenuItemDisplay` と `check()`／`is_expanded()`、`CardState.action: CardAction` と `input()`／`is_interactive()`／`selection()`、`AlertState.content` と `has_content()`／`expansion()`、`BreadcrumbItemState` と `DataTableCheckState` の enum 化、`CodeBlockState.collapse: CodeCollapse`、`CheckableState.control: CheckControl` と `is_checked()`／`is_mixed()`／`checked()`／`switch_text()`／`switch_drag()`）」
- **12.2 の書き戻し**：decisions.md の SLIM-14 の行に次の 1 文を足す。
  「排他な事実は pub の enum の field（名前は型名から owner の接頭辞を除いたもの。部分を表す enum の名前は `State` で終わらせない）、常にある事実は pub の field、一部の種類にだけある事実と導く事実は accessor。bool の accessor は `is_`／`has_` で始める。」

## 3. 担当外の unit への申し送り（親が該当する監査の担当に渡す）

担当外の unit の確定は、各担当の判断による。ここには事実と推奨だけを書く。

1. **G9（W7）の改名表 1 は compile error になる。**
   - 改名表 1 は `MenuApi::show(factory)` と `CommandPaletteApi::show(factory)` を `content(factory)` にする。
   - しかし両方の型には、既に `pub fn content(self) -> Part<PopupState>`（surface の part）がある。場所は `crates/kivact-interaction/src/menu/handle.rs` と `palette/handle.rs`。同名の inherent method が重複して compile できない（E0592／E0201）。
   - 推奨は `mount(factory)` である。
     - doc の「Mount the caller's content …」と一致する。
     - `content()` が Popup・Dialog・Select・Tooltip・Combobox と揃った「surface の part」の意味を保つ。
     - workspace に `pub fn mount` は無い。
   - DEC-18 の意図（`show` を ARG-6 の命令のために空ける）はこの形でも満たせる。作業指示書 12.3 の「`content(factory)`」も直す。
   - L4 の IN-2 と IN-3 が置く marker の文「set by MenuApi::content …」は、`content(self)` の part を指す。`mount` を採れば事実どおりになる。
2. **FINAL-1（W7、D5 付録 P5-D F-1）の derive が compile できない。**
   - `Availability` のコード片は `#[derive(.., Default, ..)]` と手書きの `impl Default` を両方持つ。重複した実装（E0119）になり、derive の側も `#[default]` が無いので error になる。
   - derive の列から `Default` を外し、手書きの impl だけを残す（`PartState` の derive の `Default` はそのままでよい）。
3. **W2・W3 の unit で、付属書どおりに書くと既存の規則で error になるもの**（FINAL-2 の監査で見つけた。詳細は FINAL-2 の節）：
   - G3（`State.asked`）と T2（`Recent.last`）：数えない field に marker を置かない。
   - T6（`CoreNode.at`・`handlers`）：HEAD の marker を残す。
4. **CO（W6 L3）への条件**：G3 の後のコードで、`SharedImage.current` への書込みを全て `replace_current` 経由にする。そうしないと X4 の error になる。
5. **FINAL-2 の前提**：W1 の 3 件（F5 の `Indexed`、M7 の `Plan`、T8 の `OwnerEntry.extra`）が、それぞれの担当の監査が確定した閉じ方で閉じていること。L1〜L3 の各 unit が D5・D8 の文のとおりに marker を置くこと。FINAL-2 の節の「W6 の後に残る projection」の表が照合の基準になる。
6. **任意の提案（G9 の範囲）**：`kivact-ui-soft` の `LightboxHandle::close()` は、ARG-6 の `show`/`hide` に揃わない。G9 の改名表に足すかは、G9 の担当が決める。R6 の範囲では変えない。

## 4. ユーザー（人間）にしか決められない項目

無い。設計者の判断に見えたものは、すべて本書で確定した。判断と理由を残しておく。
- E-7・E-8・E-9 を取り消した（理由は各節）。
- E-11 は V2 で行わない。どの unit の行にも無いため。
- `LayoutInset` は pub のままにする（P8-SOFT の節）。
- `RegionInfo.selected` を削除する（CR の節）。
- 命名規則（冒頭の節）。
