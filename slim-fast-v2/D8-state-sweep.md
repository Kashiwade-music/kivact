# design-P8：CheckableState の排他を型にする／未確認だった state-fields marker の全数確認

> **V2 付属書 D8**（設計担当 P8 の成果物）。正本は [SLIM_FAST_V2_DESIGN.md](../SLIM_FAST_V2_DESIGN.md)、手順は [SLIM_FAST_V2_WORK_ORDER.md](../SLIM_FAST_V2_WORK_ORDER.md)。
> - 本書の「8 章 設計者（人間）の判断が要る点」は V2 設計書 4 章（DEC-01〜DEC-23）で決定済み。実装者は 8 章を読んで選ばない。V2 設計書 5 章（付属書間の調整）が本書を上書きする箇所がある。
> - 本書の unit 名と V2 の unit ID の対応は V2 設計書 0.2 節。unit の実施順・lane・wave は V2 作業指示書だけが決める（本書 6 章の「依存」「並列可」は参考）。
> - 文中の P1〜P8 は付属書 D1〜D8（`design-P<n>.md` は `D<n>-*.md`）を指す。`W:\w\kv-prof`・`W:\w\kv-mem`・`W:\w\kv-size`・scratchpad の path は設計時の試作・計測の場所（設計した機械にだけある参考）で、実装に必要ではない。行番号は HEAD `73c36c7c` のもの。

担当 P8。対象 HEAD 73c36c7c（= db8d679b + 引継ぎ文書）。リポジトリは読み取りだけで、編集していない。`cargo xtask` の audit は実行していない。

前提は P5（design-P5.md）を採用済みであること。P5 が人間の判断に残した 2 点（P5 8 章の 2 と 3）を、設計者の決定に従って確定設計にした。

1. **`CheckableState` の排他を型で表す。** doc に書いた不変条件では残さない（3.1、3.2）。
2. **P5 が抜き取らなかった marker を全数確認する。** workspace の全 `xtask-allow(state-fields)` marker から、P5 が確定させたものを除いた残りを全て判定した（3.3〜3.5、付録 A・B）。

本書の構成：
- 本文 1〜8 章
- 付録 A：誤（WRONG）の詳細設計。group ごとの監査結果を本文の番号 P8-W01〜W21 に対応づけたもの。
- 付録 B：全 917 行の判定表。確認済みの行も 1 行ずつ載せる。

本文と付録が食い違う場合は、本文が正しい（3.4 の上書きを参照）。

---

## 1 事実

### 1.1 `CheckableState`（`crates/kivact-ui/src/values.rs:290-324`）

```rust
pub struct CheckableState {
    pub input: PartState,
    pub checked: CheckedState,            // Unchecked / Checked / Indeterminate
    pub size: ControlSize,
    pub invalid: bool,                    // marker: projection（偽。P5-F）
    pub text: Option<Value<String>>,      // marker: projection（偽）
    pub switch_drag: Option<SwitchDragState>, // marker: projection（偽）
}
```

書き手は 3 箇所ある。

| 書き手 | control | checked | invalid | text | switch_drag |
|---|---|---|---|---|---|
| `toggle.rs:109-116`（Checkbox） | check box | 3 値（`.indeterminate(..)` があれば mixed） | caller の `.invalid(..)` | None | None（`no_gesture`） |
| `toggle.rs:109-116`（Switch） | switch | 2 値（`switch_behavior` は mixed を捨てる、`toggle.rs:183-186`） | caller の `.invalid(..)` | Some | 任意 |
| `radio_group.rs:236-245` | radio | `from_bool(selected)`（2 値） | **常に false** | None | None |
| `kivact-ui-soft/src/indicator.rs:122-129` | Checkbox / Radio / Check | Kind による | false | None | None |

型が表せるのに、実際には作られない組合せが 4 つある。

- (a) `Indeterminate` で、check box でないもの。
- (b) `text` が None なのに `switch_drag` が Some のもの。
- (c) radio で `invalid = true` のもの。
- (d) check box か radio で `text` が Some のもの。

P5-F F-T4 は (b) と (d) を `switch: Option<SwitchFacts>` で閉じた。(a) は doc の不変条件として残し、(c) は「radio item は false を報告する」と marker に書いていた。

(c) は既存の欠陥でもある（7 章 #1）。

- `RadioGroup` は `Element` なので、caller は `.invalid(..)` を付けられる。`field(..)` で包めば、自動で付く（docs/for-kivact-user/en/how-to-guide/forms.md:71）。
- 標準の radio look（`look_controls.rs:51`）には `else if state.invalid { … Danger }` の分岐がある。しかし書き手が常に false を入れるので、この分岐は一度も通らない。
- この形は 79bbd409（Slim-Fast 以前）から同じである。

### 1.2 `CheckableState` の読み手（HEAD、全数）

`git grep -n -E "\.checked\b|switch_drag|\.text\.map"` の結果から、`CheckableState`（とその `Look<..>`）を読む箇所だけを選んだ。`ToggleState.checked`（toggle_button.rs:206、toggle_group.rs:165、docs の custom-controls.md）と、試験の `Locator::checked()` は別物なので除いた。

| crate | file:line |
|---|---|
| kivact-ui | `look.rs:177,181,211`、`look_controls.rs:49,59`、`look_radio.rs:30`、`radio_group.rs:141`（doc）、`radio_group.rs:259`、`values.rs:318`（eq） |
| kivact-ui-basic | `recipes.rs:298,310,326`、`recipes_controls.rs:67` |
| kivact-ui-creative | `recipes.rs:199,200`、`recipes_controls.rs:53` |
| kivact-ui-game | `recipes_controls.rs:216,217,267,277`（text）、`recipes_extra.rs:341,359`、`choice.rs:73` |
| kivact-ui-pop | `recipes_controls.rs:21,29,48,68,79`、`recipes_extra.rs:297` |
| kivact-ui-soft | `recipes_controls_choice.rs:82,155,207,231,241`（drag）、`:283`（drag）、`:403`、`segmented_control.rs:58`、`indicator.rs:178` |
| dev/tests | `kivact-public-api/tests/e2e/ui_theming.rs:48` |
| docs・examples | 0 件（`CheckableState` の field を読むコード片は無い） |

合計は 39 箇所、18 ファイル。P5 の見積り「約 54」は、`ToggleState` と `Locator::checked()` を含めて数えた値である。

### 1.3 marker の全数

| 区分 | 数 |
|---|---|
| `git grep -n "xtask-allow(state-fields)"` の行（`.md`・`.txt` を除くコード） | 1210 |
| うち lint 自身の試験 fixture（`dev/xtask/src/state_fields/`） | 10（対象外） |
| P5 が確定したもの（P5 の 3.5 節の一覧、付録の file:line、型を置き換えた struct の全 field） | 283（`p8/covered-by-P5.tsv`） |
| **本書で判定したもの** | **917** |

917 行は、crate ごとに 8 つの group（G1〜G8）に分け、全ての行を判定した（抜き取りはしていない）。判定の方法：

1. 各行について、field の書き手を全て Grep で探す。
2. 同じ owner の数える field との組合せを、書き手の式から確かめる。
3. prefix ごとの条件（BRIEF：`p8/BRIEF.md`）で判定する。
   - independent：全ての組合せに到達できる。
   - option：caller・builder・theme が与える値である。
   - cache：正本と無効化がある。
   - identity：識別子である。
   - statistic：振る舞いを分岐させない。

設計者（本書）は、全ての誤（WRONG）と閉じ方の変更（KIND）の根拠を読み直した。うち 9 件は、コードを読み返して確かめた（5.3）。

| 判定 | 行数 | 意味 |
|---|---|---|
| 確認済み（OK） | 499 | 主張が正しく、理由文も事実と合う |
| 文言（WORDING） | 284 | 閉じ方は正しいが、理由文が事実と違うか、independent の「なぜ全ての組合せが有効か」が無い。新しい文を確定した |
| 閉じ方の変更（KIND） | 84 | prefix の変更だけで閉じる（コードは変えない）。主に independent → option |
| 誤（WRONG） | 49 行（設計 21 件） | 隠れた含意・排他、導出できる複写、偽の主張。型・導出に直す |
| P5 に従う | 1 | `examples/kivact-ui-game/src/game/controls.rs:67`。P5 の E-10 が行番号 64・66 として確定済み |

本書の上書き（3.4）を反映した後の数である。

## 2 原因

1. **`CheckableState` は 3 種類の control の snapshot を 1 つの struct にまとめている。**
   - control ごとの差（mixed は check box だけ、ON/OFF の文字と drag は switch だけ）が、Option と doc で表されている。
   - radio の書き手は、`invalid` を入力に結ばずに定数を入れた。
2. **marker の大半は、Slim-Fast の U61 で旧理由に接頭辞を機械的に付けたものである**（P5 の 2 章の 1）。
   - 次の 3 つの定型文が、事実と照合されないまま残った。
     - `option: … absent means the component's default`（実際の既定値が無い slot に多い）
     - `independent: … every combination is valid`
     - `option: prefer gpu`（field 名の繰り返し）
   - 書き手が 1 つしか無いのに、「独立」と主張している場合が多い。2 つの field を同じ式から作っていて、含意（SM-2）や同時性（SM-3）が隠れていた（3.5）。
3. **lint は marker の意味を検査しない。** P5 の X4・X5 で一部（projection と重複文）は機械化される。independent・option・cache の真偽は、今後も人の監査で確かめる。

---

## 3 設計（確定版）

### 3.1 `CheckableState` の新しい形（公開 API）

**決定：** control の種類を enum `CheckControl` にし、switch だけが持つ情報を variant の中に入れる。recipe が普段読むのは accessor（`is_checked()`、`is_mixed()`、`checked()`、`switch_text()`、`switch_drag()`）にする。分岐を全て書きたい theme は `match state.control` を書ける。

**accessor を主にする理由**（React/JS の開発者の書きやすさ）：
- recipe の 8 割は「on か」「mixed か」だけを見ている（1.2 の 39 箇所のうち 31 箇所）。これは Radix の `data-state="checked" | "indeterminate"` や、base-ui の `checked` / `indeterminate` prop と同じ読み方である。
- `state.is_checked()` は、`PartState` の `is_focused()` や、P5 FINAL-1 の `is_pressed()` と同じ形に揃う。
- `match` は enum の網羅性を活かす場合だけに使い、必須にしない。
- 型としては、排他が型で表される（(a)(b)(d) は作れない）。

**(c) の閉じ方：** radio group の `.invalid(..)` を各 radio item に配る。これで invalid は全ての control で入力から決まり、全ての組合せが到達する（独立）。型の分岐は増えない。base-ui の Radio が invalid な field の中で `data-invalid` を持つのと同じ振る舞いである。

`crates/kivact-ui/src/values.rs`。`:290-324`（`CheckableState` と `impl PartialEq`）を次で置き換える。

```rust
/// Which control a [`CheckableState`] describes, with the state only that
/// control has.
#[derive(Clone, Copy, Debug)]
pub enum CheckControl {
    /// A check box: on, off or mixed
    /// ([`Checkbox::indeterminate`](super::Checkbox::indeterminate)).
    CheckBox(CheckedState),
    /// A switch: on or off.
    Switch {
        // xtask-allow(state-fields): independent: the switch's bound value; the opted-in drag starts from either value and moves the thumb whichever one the binding holds
        /// Whether the switch is on.
        on: bool,
        /// The localized word for the state: `switch.on` ("ON") or
        /// `switch.off` ("OFF"). The handle is made at setup and stays the
        /// same for the mounted control; read it in the paint, which follows
        /// its text. It is not part of the state's equality.
        text: kivact_core::state::Value<String>,
        // xtask-allow(state-fields): independent: the live visual of the opted-in drag gesture, None when the switch has no drag or none is running; it runs while the switch is on or off
        /// Live visual state while the opted-in drag gesture moves the thumb.
        drag: Option<SwitchDragState>,
    },
    /// A radio item: whether it is the group's chosen option.
    Radio {
        /// Whether this option is chosen.
        selected: bool,
    },
}

impl PartialEq for CheckControl {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::CheckBox(a), Self::CheckBox(b)) => a == b,
            (
                Self::Switch { on: a, drag: drag_a, .. },
                Self::Switch { on: b, drag: drag_b, .. },
            ) => a == b && drag_a == drag_b,
            (Self::Radio { selected: a }, Self::Radio { selected: b }) => a == b,
            _ => false,
        }
    }
}

/// The state of a check box, a switch or a radio item for its recipe.
///
/// Most recipes read [`is_checked`](Self::is_checked) and
/// [`is_mixed`](Self::is_mixed); `match state.control` tells the three
/// controls apart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CheckableState {
    /// The actual input state of the control.
    pub input: PartState,
    /// Which control this is, with its checked state.
    pub control: CheckControl,
    /// The size the caller chose with `.size(..)` (default `Medium`).
    pub size: ControlSize,
    /// Whether the caller marked the value invalid: `.invalid(..)` on the
    /// check box or switch, or on the radio group around a radio item
    /// (a `field` with an error sets it too).
    pub invalid: bool,
}

impl CheckableState {
    /// The checked state as assistive technology reports it; only a check
    /// box is ever [`Indeterminate`](CheckedState::Indeterminate).
    pub fn checked(&self) -> CheckedState {
        match self.control {
            CheckControl::CheckBox(checked) => checked,
            CheckControl::Switch { on, .. } => CheckedState::from_bool(on),
            CheckControl::Radio { selected } => CheckedState::from_bool(selected),
        }
    }

    /// Whether it is on: a checked box, a switch that is on, or the chosen
    /// radio item.
    pub fn is_checked(&self) -> bool {
        self.checked().is_checked()
    }

    /// Whether a check box shows its mixed (partially checked) state.
    pub fn is_mixed(&self) -> bool {
        matches!(self.control, CheckControl::CheckBox(CheckedState::Indeterminate))
    }

    /// A switch's localized word for its state; `None` for a check box and
    /// a radio item.
    pub fn switch_text(&self) -> Option<kivact_core::state::Value<String>> {
        match self.control {
            CheckControl::Switch { text, .. } => Some(text),
            CheckControl::CheckBox(_) | CheckControl::Radio { .. } => None,
        }
    }

    /// The live visual of a switch's opted-in drag; `None` otherwise.
    pub fn switch_drag(&self) -> Option<SwitchDragState> {
        match self.control {
            CheckControl::Switch { drag, .. } => drag,
            CheckControl::CheckBox(_) | CheckControl::Radio { .. } => None,
        }
    }
}
```

- `values.rs:3-5` の module doc にある「(the real `CheckedState` plus size and invalid)」を、「(which control it is and its checked state, [`CheckControl`], plus size and invalid)」に変える。
- lint の扱い：
  - `CheckableState` で数える field は `invalid` の 1 個だけになる。`control` は数えない名前の enum で、`input` は今も数えていない。したがって marker は置かない（HEAD の 3 本は消す）。
  - `CheckControl::Switch` の数える field は `on` と `drag` の 2 個なので、上の 2 本の marker を置く。`Radio` の数える field は 1 個なので marker は不要。
- `crates/kivact-ui/src/lib.rs:188-191` の `pub use values::{…}` の列に `CheckControl` を足す（P5-F の `SwitchFacts` は作らない）。

### 3.2 書き手と読み手の書換え（機械的な置換表と、表に当てはまらない個別の指示）

**書き手 1：`crates/kivact-ui/src/toggle.rs`**
- macro の引数 `$text:expr` を `$control:ident` に替える（`:16`）。
- 呼出しを替える。
  - `:144`（Checkbox の呼出しの最後の引数）の `None` を `checkbox_control` にする。
  - `:158`（Switch の呼出しの最後の引数）の `Some(switch_text)` を `switch_control` にする。
- `into_view` の `:103-116` を次にする。

```rust
                let invalid = self.root.accessible_invalid();
                let size = self.size;
                let (control, drag) =
                    $gesture(behavior.control(), self.state, size, self.on_edit);
                let kind = $control(self.state, drag);
                let part = control.map_state(move |state: ToggleState| CheckableState {
                    input: state.input,
                    control: kind(state.checked),
                    size: size.get(),
                    invalid: invalid.as_ref().is_some_and(|invalid| invalid.get()),
                });
```

- `switch_text` の定義（`:189-196`）の直後に、次の 2 関数を足す。

```rust
/// A check box's control: its checked state as the behavior reports it,
/// mixed included.
fn checkbox_control(
    _: Binding<bool>,
    _: super::switch_drag::Visual,
) -> impl Fn(CheckedState) -> CheckControl + 'static {
    CheckControl::CheckBox
}

/// A switch's control: on or off, its localized word for the state (made at
/// setup) and the live visual of its opted-in drag.
fn switch_control(
    state: Binding<bool>,
    drag: super::switch_drag::Visual,
) -> impl Fn(CheckedState) -> CheckControl + 'static {
    let text = switch_text(state.into_value());
    move |checked| CheckControl::Switch {
        on: checked.is_checked(),
        text,
        drag: drag.and_then(|drag| drag.get()),
    }
}
```

- import を変える。`:9` は `use super::values::{CheckControl, CheckableState, ControlSize};` にし、`use kivact_core::view::CheckedState;` を足す。
- 補足：`Binding`・`Value`・`Visual` はどれも Copy で、今のコードでも `self.state` を 3 回使っている。

**書き手 2：`crates/kivact-ui/src/radio_group.rs`**
- `:180-190` の分解を `let RadioGroup { mut root, options, … } = self;` にする。その直後に `let invalid = root.accessible_invalid();` を置く（`with_base_recipe_style(root, …)` が root を消費する `:208` より前）。
- `:200-204` の `for_each` の closure を、`move |item| radio_option(group, item, size, invalid, &looks, slot.as_ref(), item_node.as_ref())` にする。
- `radio_option` の引数の `size: Value<ControlSize>,` の次に、`invalid: Option<Value<bool>>,` を足す。
- `:236-245` を次にする。

```rust
        CheckableState {
            input: state.input,
            control: CheckControl::Radio { selected: state.selected },
            size: size.get(),
            invalid: invalid.is_some_and(|invalid| invalid.get()),
        }
```

- import を変える。`:3` は `use super::values::{CheckControl, CheckableState, ControlSize};` にし、`:10` の use 列から `CheckedState` を除く（他に使っていない）。
- `:109` の doc「indicator recipes in [`CheckableState::size`]」は、そのままでよい。

**書き手 3：`crates/kivact-ui-soft/src/indicator.rs:110-130`**
- `let checked = self.checked;` の次に、`let kind = self.kind;` を置く（`Kind` は Copy）。
- literal を次にする。

```rust
            Look::new(CheckableState {
                input,
                control: match kind {
                    Kind::Radio => CheckControl::Radio { selected: checked.get().is_checked() },
                    Kind::Checkbox | Kind::Check => CheckControl::CheckBox(checked.get()),
                },
                size,
                invalid: false,
            })
```

- `Kind::Check`（単独の check mark）は、`check_indicator` が `from_bool` で 2 値だけを渡すので、`CheckBox` の Checked か Unchecked になる。
- import の `:15` を `use kivact::ui::{CheckControl, CheckableState};` にする。
- この file は P5 の FINAL-1 も変える（`input.disabled` の行）。FINAL-1 は本 unit の後に行うので、衝突しない。

**読み手の置換表**（`X` は `state`・`s`・`look`・`state.get()` などの受け手。`Look<CheckableState>` は Deref で method を呼べる）

| HEAD の式 | 置換後 |
|---|---|
| `X.checked.is_checked()` | `X.is_checked()` |
| `X.checked == CheckedState::Checked` | `X.is_checked()` |
| `X.checked == CheckedState::Indeterminate` | `X.is_mixed()` |
| `X.checked != CheckedState::Unchecked` | `X.checked() != CheckedState::Unchecked` |
| `X.text.map(…)` | `X.switch_text().map(…)` |
| `X.switch_drag` | `X.switch_drag()` |
| `X.invalid`、`X.input`、`X.size` | 変更なし |

**ファイルごとの適用**（全て上の表のどれかに当たる。行は HEAD のもの）

| file | 行と置換 | import の後始末 |
|---|---|---|
| `crates/kivact-ui/src/look.rs` | :177 `!= Unchecked`、:181 `== Indeterminate`、:211 `.checked.is_checked()` | `CheckedState` は :177 で使うので残す |
| `crates/kivact-ui/src/look_controls.rs` | :49、:59 | — |
| `crates/kivact-ui/src/look_radio.rs` | :30 | — |
| `crates/kivact-ui/src/radio_group.rs` | :141（doc 例）、:259 | 書き手 2 を参照 |
| `crates/kivact-ui-basic/src/recipes.rs` | :298 `!= Unchecked`、:310 `== Indeterminate`、:326 | 残す（:298） |
| `crates/kivact-ui-basic/src/recipes_controls.rs` | :67 | — |
| `crates/kivact-ui-creative/src/recipes.rs` | :199、:200 `== Indeterminate` | :17 `use kivact::view::CheckedState;` を消す |
| `crates/kivact-ui-creative/src/recipes_controls.rs` | :53 | — |
| `crates/kivact-ui-game/src/recipes_controls.rs` | :216、:217 `== Indeterminate`、:267、:277 `state.text.map(…)` | :10 `use kivact::view::CheckedState;` を消す |
| `crates/kivact-ui-game/src/recipes_extra.rs` | :341、:359 | — |
| `crates/kivact-ui-game/src/choice.rs` | :73 | — |
| `crates/kivact-ui-pop/src/recipes_controls.rs` | :21 `!= Unchecked`、:29 `== Checked`、:48 `== Indeterminate`、:68、:79 | 残す（:21） |
| `crates/kivact-ui-pop/src/recipes_extra.rs` | :297 | — |
| `crates/kivact-ui-soft/src/recipes_controls_choice.rs` | :82、:155 `!= Unchecked`、:207 `== Checked`、:231 `== Indeterminate`、:241 と :283 `s.switch_drag`、:403 | 残す（:155） |
| `crates/kivact-ui-soft/src/segmented_control.rs` | :58 | — |
| `crates/kivact-ui-soft/src/indicator.rs` | :178 | 書き手 3 を参照 |
| `dev/tests/kivact-public-api/tests/e2e/ui_theming.rs` | :48 | — |

表に当てはまらない個別の指示：
- `values.rs:315-323` の `impl PartialEq for CheckableState` は消す（derive に替わる）。
- 文書は変えない。docs の `state.checked.is_checked()` は `ToggleState`（custom-controls.md）の話で、本変更の対象ではない。

**P5 との関係：** 本節は P5-F F-T4 を**置き換える**。
- P5 の L5 の UI-3 は、F-T4（`SwitchFacts`、`switch: Option<SwitchFacts>`、`lib.rs` の `SwitchFacts` の export、`toggle.rs`・`radio_group.rs`・soft `indicator.rs`・game `recipes_controls.rs:277`・soft `recipes_controls_choice.rs:241,283` の F-T4 の書換え）を行わない。
- P5-F 2 章の `values.rs` 301・304・311 の行（CheckableState の marker）も、本書が置き換える。
- UI-3 は、同じ file にある F-T5 などの他の変更だけを行う。

**unit の形：** lane の中で行う。compile error を手掛かりに読み手を直す unit（FINAL-1 の形）にはしない。理由は 3 つある。
1. 読み手は全数を特定した（1.2）。
2. 読み手の file はどれも、P5 の他の lane が持っていない（6 章で確認した）。
3. 確認コマンドの `cargo check` が、`CheckableState` を読み得る全ての package（kivact-ui と 5 つの kit、`kivact-public-api-tests`、doc test）を覆う。

### 3.3 marker の監査：判定の規則（全 group 共通）

- **independent：**
  - 同じ owner の他の数える field と、含意（「A なら B」）も排他も無いこと。それを書き手の式から確かめた。
  - 理由文には「誰がどの入力で設定するか」と「なぜ他と組み合わせて成立するか」を書く。
  - 同じファイルの兄弟 field で文を共有するのは、X5 が許す。ただし本書では、field ごとに書き分けた（付録 B）。
- **option：**
  - caller・builder・theme・application が与え、内部の event では書き換わらない値に限る。
  - 実行時に書き換わるのに option としていた行は、KIND で independent・cache・statistic に移した。
  - 「absent means the component's default」は、実際の既定値があるときだけ使う。無い場合は「absent means no X」と書く（G2・G5・G8 の WORDING の大半）。
- **cache：** 正本と再計算の契機があること。cache key の成分（arvane-text の 67 行）は、それぞれ 1 つの constructor でだけ作られ、比較にだけ使われることを確かめた（OK）。
- **identity：** id の割当て、または incarnation の刻印に限る。走っているかを示す bool と並ぶ generation（SM-6）は無かった。
- **statistic：** 振る舞いを分岐させないこと。分岐させていた行（`lower/context.rs` の `Context.dropped`、`Runtime.clock`、試験の `downs`）は independent に移した（KIND）。
- **builder の設定どうしの依存は、WORDING にとどめた。** 例：`DataTable.expanded_rows` は `details` が無いと無視される。実行時の状態ではなく、型にすると公開 builder API の形が変わるため。依存は新しい marker 文に書いた（付録 A-G2 末尾）。
- **SM-5 の形で書き手が外部にあるものは、independent で閉じる。**
  - `kivact-i18n/src/numbers.rs:151,153` の `SymbolCapture` は、ICU が `DataProvider::load` で埋める sink で、constructor では渡せない。
  - ICU は 2 つの slot を独自の順で埋め、`NumberSymbols::new` が build の後に両方を読むので、build 中の組合せは全て有効である。
  - lint の変更は不要とした。
  - `kivact-test` の試験の slot も同じ扱いにした（P5 と同じ）。

**適用の規則（実装者向け）：**
1. 付録 B の各行は、HEAD の marker の file:line を示す。P5 の unit の後では行がずれるので、**`owner.field` で field を探し、その直前の marker 行を「新しい marker 行」の全文で置き換える**。doc comment より上の位置は変えない。
2. 「新しい marker 行」が空欄の行のうち、「確認済み」は変更しない。「誤」は付録 A の設計に従う。
   - field が消える行や、数えない型に変わる行は、marker も消す。消さないと `unused marker` の error になる。
   - 付録 A が「数える field が 1 個になるので marker を削除」と書いた行も消す。
3. marker の文に行番号を書かない（付録 B では除いてある）。
4. 各 unit の完了時には、`cargo xtask check --no-write --only state-fields --crate <package>` が error 0 であること。`--report-audit` は付けない。

### 3.4 統合での上書き（付録 A・B の該当行は、本節が正）

| # | 対象 | group の判定 | 確定 | 理由 |
|---|---|---|---|---|
| P8-O1 | `kivact-style/src/resolve/compile.rs:31,37,42` `CompiledStyle.retained_paint`・`text`・`states` | KIND（option → projection） | **WORDING（option のまま）**。新しい文は付録 B | `text`・`states` は `pub` の field で、X4 は package 全体を検査する。そのため `theme/value.rs:342` の `self.text = text` を「他の書き手」と誤検出する（P5 の 3.6 O2 と同じ問題）。値は authored な `Style` の option を `compile_style` が写したものなので、option で正しい |
| P8-O2 | `examples/kivact-ui-game/src/game/controls.rs:67` `filter_on` | WORDING | **P5 の E-10 に従う** | P5 の E-10 が書いた `controls.rs:64,66` は、CRLF の数え違いで、HEAD の 65・67 行の 2 つの marker を指している |
| P8-O3 | `kivact-ui-soft/src/timestamp.rs:171` `TimestampEntry.copyable` | WRONG（pub field を削除） | **WORDING（option、field を残す）**。8 章 #1 | 読み手が無く、copy 操作も未実装であることは事実。ただし pub field の削除は、kit の公開 API の縮小にあたる（方針 5）。V2 では field を残し、marker を事実どおりの option にする。実装するか削除するかは人間が判断する |
| P8-O4 | 付録 A-G3 W1 の追加 E2E の「実装者が確認すること」 | — | **追加しない** | `crates/kivact-test/src/screen/scene_cache_tests.rs:112` `registering_a_font_after_a_cached_canvas_frame_replaces_fallback_shaping` は、font の登録だけで描画が変わることを見ており、担保は十分 |
| P8-O5 | 付録 A の「追加推奨」の E2E | — | **6 章の表に書いたものだけを足す**。それ以外は足さない | 振る舞いを変えない型の整理で、旧 field と accessor・variant の対応は全単射。既存の試験で担保できる |
| P8-O6 | 付録 A-G6 W1 の「`static RETAINED` を lint が数えるなら marker を置く」 | 条件付き | **置かない** | static group に数える static が 1 個だけなので、組合せの error にならない。`Retained` の 2 field（Option と AtomicBool）は数えるので、付録の 2 本の marker は置く |
| P8-O7 | `lower/context.rs:57` `Context.dropped` | KIND（statistic → independent） | KIND で確定 | 制御に使う値なので statistic は誤り。ただし隠れた状態機械ではなく、prefix を直せば閉じる |

### 3.5 誤（WRONG）の一覧：確定した設計 21 件

詳細（事実、新しい型のコード、全ての書き手と読み手の書換え、E2E、所有ファイル）は付録 A にある。公開 API が変わるのは W08・W14・W15 の 3 件だけ。**振る舞いが変わるものは無い**。

| ID | 対象（HEAD） | 隠れていた形 | 閉じ方 | 公開 API | 付録 |
|---|---|---|---|---|---|
| P8-W01 | `kivact-ui/src/data_table/runtime.rs:36,38` `ModelPlan.page`・`page_size` | SM-2（`page_size` が Some なら `page` も Some） | `paging: Option<PagingPlan { page, size_choice }>` と accessor `page()` | なし | A-G2 W1 |
| P8-W02 | 同 `:40,45` `ModelPlan.groups`・`grouping_key` | SM-3（同時に Some） | `grouping: Option<GroupingPlan { groups, key }>` | なし | A-G2 W2 |
| P8-W03 | `kivact-engine/src/model/changes.rs:148,152` `Dispatched.paint`・`fonts` | 冗長な書込み（`sync.rs:148`）が「fonts なら paint」を作る | `sync.rs:148` の 1 行を消す（読み手の redraw は既に `paint \|\| fonts`） | なし | A-G3 W1 |
| P8-W04 | `kivact-engine/src/text_input/source.rs:22,25`、`records.rs:24,26` `multiline`・`wrap_width` | SM-2（wrap_width が Some なら multiline） | `enum Breaks { Single, Lines { wrap: Option<f64> } }` | なし | A-G3 W2 |
| P8-W05 | `kivact-core/src/view/mount/part_input.rs:39,41,43` `Selected` | 3 field で 3 つの状況を表す暗黙の enum | `enum Selected { Bubbled, Own(Rc<Target>), Unknown }` | なし | A-G4 W1 |
| P8-W06 | `kivact-core/src/view/overlays.rs:517,520` `KeyRoute.stop`・`activates` | stop が Some なら activates | `enum KeyRoute { Focused, Modal, Unfocused }` と accessor | なし | A-G4 W2 |
| P8-W07 | `kivact-core/src/view/semantics_data.rs:26,29,31` `SemanticsPatch.text`・`relation`・`orientation` | `input` を含めて、1 つの patch に 1 つだけ付く排他の tag（SM-4） | `SemanticInput::{Relation, Orientation, Text}` に統合 | なし | A-G4 W3 |
| P8-W08 | `kivact-ui-soft/src/layout.rs:21,24` `LayoutInset.own_outer_x`・`_y` | SM-3（常に同じ値） | `own_outer: Option<f64>` | **あり**（`LayoutInset` の pub field。repo 内の利用は 0） | A-G5 W1 |
| P8-W09 | 同 `:173,183` `LayoutSlots.root_ref`・`content_width` | root_ref は content_width がある時だけ意味を持つ | `content_width: Option<ContentWidth { root, width }>` | なし | A-G5 W2 |
| P8-W10 | `arvane-native/src/module.rs:21,22` `MAPPING`・`REPORTED` | REPORTED が true なら MAPPING は初期化済み。REPORTED は計数ではない | `static RETAINED: OnceLock<Retained { mapping, reported }>`（P8-O6） | なし | A-G6 W1 |
| P8-W11 | `arvane-native/src/port/win32/state.rs:31,32` `Utf16Decoder.tail` | 同じ WM_CHAR の中で必ず取り出す戻り値の退避。pending と排他 | `tail` を消し、`push` が `Option<String>` を返す | なし | A-G6 W2 |
| P8-W12 | `arvane-render/src/draw/canvas.rs:41,42` `GlyphRun.hint`・`cached` | cached なら hint | `enum GlyphRaster { Outline, Hinted { atlas } }` | なし | A-G6 W3 |
| P8-W13 | `arvane-shell/src/a11y/shared.rs:45,46` `PlatformUpdate.focus`・`all_runs` | focus は導出できる。all_runs なら runs は空（SM-2） | focus を導出にし、`enum RunSelection { All, Changed(&[..]) }`。accessor の signature は変えない | なし | A-G6 W4 |
| P8-W14 | `arvane-shell/src/ime.rs:49` `ShellImeText.password` | password なら composition は None（marker と逆）。読み手は 0 で、purpose は `set_ime_purpose` が別に渡す | field を削除 | **あり**（backend 契約の `ShellImeText`。代わりは `InputPurpose::Password`） | A-G6 W5 |
| P8-W15 | `kivact-gpu-share/src/common.rs:130-132`、`kivact-types/src/gpu.rs:139-141` adapter の luid・device_uuid・driver_uuid | 2 つの UUID は同時に Some（SM-3）。どちらの組かは backend が決める | `pub enum AdapterIds { Dx12 { luid }, Vulkan { device_uuid, driver_uuid, luid: Option } }`、`GpuAdapter { ids, name, generation }`、`backend()` は accessor。gpu-share の `Identity` は消す | **あり** | A-G6 W6 |
| P8-W16 | `kivact-gpu-share/src/pool.rs:133` `Slots.next` | `last` から導出できる | field を消し、`cursor()` で計算 | なし | A-G6 W7 |
| P8-W17 | `kivact-shader/src/entry.rs:135,136` `Interface` | takes_params なら params_declared | `enum Interface { NoParams, Params { taken } }` | なし | A-G6 W8 |
| P8-W18 | `kivact-layout/src/columns.rs:60,63` `Counted.unfiltered`・`at` | SM-2 | `enum Filtering { Unfiltered, Filtered { at } }` | なし | A-G7 W1 |
| P8-W19 | `kivact-test/src/screen/editing_drop_probe.rs:7,13`、`host_sync_tests.rs:356` | armed なら reader が Some（Drop で unwrap する） | `armed: RefCell<Option<(Reader, NodeId)>>` | なし（試験） | A-G7 W2 |
| P8-W20 | `dev/bench/src/bin/kv_heap.rs:35,37` `PTRS`・`SITES` | SM-3（組の索引。lock が分かれている） | `static HEAP: Mutex<Option<Heap { ptrs, sites }>>` | なし（bench） | A-G7 W3 |
| P8-W21 | `dev/xtask/src/check_source.rs:16,18` `Source.target`・`library` | library は target の kind から導出できる。到達しない組がある | `enum Target { Undeclared, Library(String), Binary(String) }` と accessor `library()` | なし（xtask） | A-G7 W4 |

KIND（84 行）の主な内訳：

| 変更 | 行数 | 例 |
|---|---|---|
| independent → option | 62 | soft の slot の有無（app_shell・Layout・SideNav・stack・link・tab_list）、basic の `attachment`・`chart` の builder 値、`Interactive.*`（compile の入力） |
| option → independent | 10 | 実行時に書き換わる値：`PointerSource.pressure`・`tilt`、`OwnerEntry.parent`、`NodeTree.root`、`Producer.shared`、x11 の `reasons`・dnd の `point`、`wake_reasons`、`RendererChange::Gpu.interop` |
| statistic → independent | 3 | `Context.dropped`、`Runtime.clock`、試験の `downs` |
| → projection | 3 | facts の `Node.kind`・`parent`（independent から）と `SemanticsPatch.reactive`（option から）。X4 の形で、可視範囲に名前の衝突が無いことを Grep で確かめた |
| cache → independent | 2 | `numbers.rs` の `SymbolCapture`（3.3） |
| その他 | 4 | `NativeInput.locale`（option → cache）、`RendererChange::Gpu.configure`（option → statistic）、`SourceNode.changed_at`（independent → identity）、`SourceNode.written`（independent → statistic） |

---

## 4 見積り（目標との対比）

| 指標 | P5 の後 | 本書の後 |
|---|---|---|
| 意味を誰も確認していない marker | 917 行（P5 の 3.3 末尾） | 0（全行を判定し、表に残した） |
| 偽の主張（WRONG）の marker | 49 行 | 0（21 件の型・導出） |
| 事実と違う理由文、または組合せの理由が無い independent | 284 行 | 0（全文を確定した） |
| prefix の誤り | 84 行 | 0 |
| `CheckableState` の型が表せるのに作られない組合せ | 1（P5 の後に残る Indeterminate の排他） | 0 |

- **KPI への影響は無視できる。**
  - `CheckableState` は、`CheckedState`（1 B）と 2 つの Option（約 24 B）が `CheckControl`（最大で `Value` と `Option<SwitchDragState>` と bool）に替わるだけで、大きさはほぼ同じ。
  - W04（`Breaks`）と W18（`Filtering`）は、hot path で分岐の数が同じ。
  - W03 は、font の登録時に `paint` の書込みが 1 回減るだけ。
  - W16 は、frame ごとに剰余の計算が 1 回増えるだけ。
  - 計算量は変わらない。
- **AC-3 との関係：** P5 の 3.7 の手順（OPEN 0）に加えて、「人間が audit を確認した」こと（設計書 6.4）を、本書の付録 B と P5 で、**全 marker** について満たす。
  - 残る人の作業は、checkpoint で生成した audit と付録 B を照合することだけ（行が消えた・増えた差分を見る）。

## 5 正しさ・リスク・E2E

### 5.1 振る舞いが変わるもの

| 変更 | 振る舞い | E2E（追加は太字） |
|---|---|---|
| 3.2 radio group の invalid | `radio_group(..).invalid(true)`、または error のある `field` の中で、各 radio item の recipe が `invalid = true` を受け取る。標準 look は、未選択の円を Danger の枠で描く（`look_controls.rs:51` の、今まで通らなかった分岐） | **`dev/tests/kivact-public-api/tests/e2e/ui_controls.rs` `an_invalid_radio_group_reports_invalid_to_its_option_recipes`**（下記） |

追加する E2E（`ui_controls.rs` の radio group の節。`sizes()`・`options()`・`mount_with` は同じファイルにある）：

```rust
#[test]
fn an_invalid_radio_group_reports_invalid_to_its_option_recipes() {
    let (screen, ()) = mount_with(options(), move || {
        let invalid = signal(true);
        let view = column((
            radio_group("Size", sizes(), signal(None::<&'static str>))
                .invalid(move || invalid.get())
                .item(|item, state| {
                    text(move || {
                        let mark = if state.get().invalid { "invalid" } else { "valid" };
                        format!("{} {mark}", item.get().label())
                    })
                }),
            button("Fix").on_press(move || invalid.set(false)),
        ));
        ((), view)
    });
    screen.find_by_text("Small invalid");
    screen.find_by_text("Large invalid");
    screen.get_by_role(Role::Button, "Fix").click();
    screen.find_by_text("Small valid");
    assert!(screen.query_by_text("Small invalid").is_none());
    assert!(screen.errors().is_empty());
}
```

（HEAD では「Small valid」が出て失敗する。これは 7 章 #1 の再現でもある。）

### 5.2 振る舞いを変えない変更の担保

| 対象 | 既存の担保 | 追加 |
|---|---|---|
| 3.1・3.2 `CheckableState` | `interaction_toggle.rs`、`ui_controls.rs`（radio）、`ui_theming.rs:41`（switch の thumb）、`theme_conformance.rs:692-705`、`binding_update.rs`、各 kit の consumer（`kivact-ui-game/tests/consumer/controls.rs`、`kivact-ui-pop/tests/consumer/controls.rs`、`kivact-ui-soft/tests/consumer/toggle_label.rs`）、game・soft の catalog の switch | なし |
| W01・W02 | `ui_data_table.rs:513,541`、`apg_grid.rs:325`、`kivact-ui-soft/tests/consumer/grouped_table.rs:49,131` | なし |
| W03 | `scene_cache_tests.rs:112` | なし（P8-O4） |
| W04 | `foundation.rs:138`（basic）、debug の `RefreshMode::Verify` | なし |
| W05〜W07 | `parts.rs:99,170`、`overlay.rs:230`、`interaction_dialog.rs:338`、`interaction_tooltip.rs:115`、`layout_scroll.rs:255`、`apg_slider.rs:8` | なし |
| W08・W09 | `kivact-ui-soft/tests/consumer/navigation.rs:91`（padding・content_width を指定しない経路だけ） | **`layout_padding_and_pixel_content_width_place_the_content`**（下記） |
| W10・W11 | `embed/windows.rs`、`embed/x11.rs` | **W11 の unit test**（下記） |
| W12 | `crates/arvane/tests/e2e/present.rs:88,186`、`transform.rs:19` | なし |
| W13 | `crates/arvane/tests/e2e/accessibility.rs:73`、`embed/uia.rs` | なし |
| W14 | `input_platform.rs:104,315`、`text_editing.rs:218` | なし |
| W15・W16 | `gpu_interop.rs`、`viewport_gpu_interop.rs`、`drawable_export_gpu.rs`、`crates/arvane/tests/e2e/external.rs` | **`GpuAdapter::matches` の doc test**（付録 A-G6 W6 の末尾。下記） |
| W17 | `shaders.rs`、kivact-shader の doc test | なし |
| W18〜W21 | `cargo test -p kivact-layout`、`cargo test -p kivact-test replacing_editing_drops_line_state_after_releasing_the_model`、kv_heap の短い実行、`cargo test -p xtask` | なし |

W08・W09 の追加試験（`crates/kivact-ui-soft/tests/consumer/navigation.rs`。同じファイルの `render_with` と `RenderOptions` の書き方に揃える）。**先に HEAD で通ることを確かめる。通らなければ、既存欠陥として STOP して報告する（期待値は変えない）。**

```rust
#[test]
fn layout_padding_and_pixel_content_width_place_the_content() {
    let screen = render_with(
        RenderOptions::with_theme(soft::theme(ThemeMode::Light)).window_size(800.0, 400.0),
        || {
            soft::layout()
                .padding(soft::Space::S6)
                .content_width(kivact::style::Length::Px(400.0))
                .content(soft::layout_content(text("Body text")))
        },
    );
    let body = screen.get_by_text("Body text").rect();
    // The layout's own padding (not the region fallback) is above the content.
    assert!(body.y >= soft::Space::S6.px() - 0.5, "{body:?}");
    // A pixel content width without side panels is centred by a gutter.
    assert!(body.x >= (800.0 - 400.0) / 2.0 - 0.5, "{body:?}");
    assert!(body.x + body.width <= (800.0 + 400.0) / 2.0 + 0.5, "{body:?}");
    assert!(screen.errors().is_empty());
}
```

（`Length` の path は、`Layout::content_width` の引数の型に合わせて import する。）

W11 の unit test（`crates/arvane-native/src/port/win32/state.rs` の末尾。cfg は同じファイルの cfg に従う）：

```rust
#[cfg(test)]
mod tests {
    use super::Utf16Decoder;

    #[test]
    fn surrogate_pairs_and_lone_surrogates_decode_in_one_push() {
        let mut d = Utf16Decoder::default();
        assert_eq!(d.push(0xd83d), None);
        assert_eq!(d.push(0xde00).as_deref(), Some("\u{1f600}"));
        assert_eq!(d.push(0xd83d), None);
        assert_eq!(d.push(u16::from(b'a')).as_deref(), Some("\u{fffd}a"));
        assert_eq!(d.push(0xd83d), None);
        assert_eq!(d.push(0xd83d).as_deref(), Some("\u{fffd}"));
        assert_eq!(d.push(0xde00).as_deref(), Some("\u{1f600}"));
    }
}
```

W15 の doc test（`kivact-types/src/gpu.rs` の `GpuAdapter::matches` の doc に置く）：

```rust
/// ```
/// use kivact_types::{AdapterIds, AdapterMatch, GpuAdapter};
/// let adapter = GpuAdapter { ids: AdapterIds::Dx12 { luid: [1; 8] }, name: String::new(), generation: 1 };
/// assert!(adapter.matches(&AdapterMatch::Luid([1; 8])));
/// assert!(!adapter.matches(&AdapterMatch::Uuid { device: [0; 16], driver: [0; 16] }));
/// ```
```

### 5.3 リスク

- **公開 API の変更**：`CheckableState`（全 theme の recipe）、`LayoutInset`（soft）、`ShellImeText`（backend の契約）、`GpuAdapter`・`AdapterIds`・`kivact_gpu_share::Identity`（GPU 共有）。
  - repo 内の利用は全て、同じ unit で直す。互換の alias は作らない（方針 5）。
  - 外部の theme 作者は、`state.checked` を `state.checked()`（または `is_checked()`）に直す必要がある。
- **X4 の誤検出：** 本書が新しく置く `projection:` は 3 種類（`SemanticsPatch.reactive`、facts の `Node.kind`・`parent`）。どれも、可視範囲に同名 field への代入が無いことを Grep で確かめた。`CompiledStyle` は、衝突するので projection にしなかった（P8-O1）。
- **group の監査の誤り：** 917 行は、8 つの group で並行して判定した。設計者は WRONG と KIND の根拠を全て読み直し、そのうち 9 件（W03、W07、W16、SymbolCapture、`CompiledStyle.text` の X4 衝突、facts の `Node` の X4 衝突、`TimestampEntry`、soft の `Indicator`、radio の invalid）はコードを確かめた。OK と WORDING は、約 15 行を抜き取って根拠の記述を読み、主張と矛盾しないことだけを確かめた（コードの再確認はしていない）。残る誤りは checkpoint の audit の照合で拾う（4 章）。

## 6 unit 分割案

P5 の 6 章の lane（L1〜L6）に P8 の unit を足す。各 lane の中では、表の順に行う。

- **lane の間でファイルは重ならない**（`p8/units.json` で機械的に確かめた）。
- P5 の unit と同じファイルを持つ P8 の unit は、**その P5 の unit の後に、同じ lane で**行う。
- P6 の unit と重なるファイルにも、依存を書いた。
- FINAL-1（PartState）と FINAL-2（X3〜X5）は、**P8 の全 unit の後**に行う。
  - FINAL-2 の X4 と X5 は、本書の新しい marker 文の上で検査する。
  - FINAL-1 は `indicator.rs` などの読み手を、本書の形の上で直す。

| lane | unit | 所有ファイル | 依存（同じ lane の前の unit） | 完了時の軽い確認 |
|---|---|---|---|---|
| L1 | **P8-POP**（marker 13 行） | `examples/kivact-ui-pop/src/pop/{components.rs, data.rs, mission.rs, patterns.rs, shell.rs}` | **P5 の XT-1 の後**（`shell.rs` は XT-1 が持つ） | `cargo check -p kivact-ui-pop-catalog`；`cargo xtask check --no-write --only state-fields --crate kivact-ui-pop-catalog` |
| L1 | **P8-L1**（W18 と marker：style・editing・i18n・devtools・layout） | `crates/kivact-style/src/{draw/layer.rs, draw/text_run.rs, paint/border_image.rs, paint/layer.rs, resolve/compile.rs, sheet/fields.rs}`、`crates/kivact-editing/src/{behavior.rs, line_colors.rs}`、`crates/kivact-i18n/src/{format.rs, numbers.rs}`、`crates/kivact-devtools/src/panels/elements/view.rs`、`crates/kivact-layout/src/{columns.rs, traversal.rs, lib.rs}` | **P5 の LY と ST・ED の後**（`sheet/fields.rs` は ST が持つ。本 unit は 7 章 #4 の doc だけを直す） | `cargo clippy -p kivact-layout --all-targets -- -D warnings`；`cargo test -p kivact-layout`；`cargo check -p kivact-style -p kivact-editing -p kivact-i18n -p kivact-devtools`；state-fields `--crate` を 5 package で |
| L1 | **P8-XT**（W20・W21 と marker） | `dev/xtask/src/{check_source.rs, api_shape.rs, canonical_path.rs, name_collision.rs, structural_options.rs, public_surface.rs, public_type_edges.rs, state_fields/check.rs}`、`dev/bench/src/bin/kv_heap.rs` | **P5 の XT-1 の後**、**P6-7 の後**（`api_shape.rs`）。FINAL-2 より前（`state_fields/check.rs:82` の 1 行だけ） | `cargo test -p xtask`；`cargo clippy -p xtask --all-targets -- -D warnings`；`cargo build -p kivact-bench --release --bin kv_heap`（別の CARGO_TARGET_DIR） |
| L2 | **P8-NA**（W10・W11 と marker） | `crates/arvane-native/src/{mirror.rs, module.rs, native_input/state.rs, port/win32/input.rs, port/win32/state.rs, port/winit/runtime.rs, port/winit/window.rs, port/winit/windows.rs, port/x11/dnd.rs, port/x11/notifier.rs}` | **P5 の NA の後**（`native_input/state.rs`） | `cargo clippy -p arvane-native --all-targets -- -D warnings`；`cargo test -p arvane-native surrogate_pairs`；state-fields `--crate arvane-native` |
| L2 | **P8-EN**（W03・W04・W14 と marker、7 章 #2・#3 の doc） | `crates/kivact-engine/src/{check/boxes.rs, contract/node.rs, host/window.rs, layer.rs, lower/assets.rs, lower/context.rs, measure/input.rs, model/aggregate.rs, model/changes.rs, model/drawables/place.rs, model/edit/session.rs, model/interact/sync.rs, model/interact/text_geometry.rs, model/pointer/state.rs, mutation.rs, text_input.rs, text_input/chunks.rs, text_input/edit_lines.rs, text_input/records.rs, text_input/refresh.rs, text_input/shaping.rs, text_input/source.rs, tree/node.rs, tree/node_tree.rs}`、`crates/arvane-shell/src/ime.rs` | **P5 の EN-1 と EN-2 の後**（`model/pointer/state.rs`、`text_input/{edit_lines,chunks,shaping}.rs`、`host/window.rs`） | `cargo clippy -p kivact-engine --all-targets -- -D warnings`；`cargo check -p arvane-shell -p arvane-native -p arvane-virtual -p kivact-test --all-targets`；state-fields `--crate kivact-engine`（arvane-shell も） |
| L2 | **P8-KT**（W19 と marker） | `crates/kivact-test/src/{facts.rs, screen/editing_drop_probe.rs, screen/host_sync_tests.rs}` | **P5 の KT の後**（`host_sync_tests.rs`）、P8-EN の後 | `cargo test -p kivact-test replacing_editing_drops_line_state_after_releasing_the_model`；state-fields `--crate kivact-test` |
| L3 | **P8-RT**（marker 9 行） | `crates/kivact-runtime/src/{reactive/data.rs, reactive/host_types.rs, task/worker_pool.rs}` | **P5 の RT の後**（`reactive/data.rs`） | `cargo check -p kivact-runtime`；state-fields `--crate kivact-runtime` |
| L3 | **P8-CO**（W05・W06・W07 と marker） | `crates/kivact-core/src/{client/dispatch/key.rs, client/trace.rs, view/devtools_state.rs, view/drawable_share.rs, view/elements/builders.rs, view/elements/container.rs, view/modifiers.rs, view/mount/create.rs, view/mount/draw.rs, view/mount/facts.rs, view/mount/part_input.rs, view/mount/parts.rs, view/mount/resolve.rs, view/mount/semantics.rs, view/mount/style.rs, view/node_modifiers.rs, view/overlays.rs, view/semantics_data.rs, view/state.rs}` | **P5 の CO の後**（`view/devtools_state.rs`、`view/state.rs`） | `cargo clippy -p kivact-core --all-targets --all-features -- -D warnings`；state-fields `--crate kivact-core`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- parts:: overlay::keys_go interaction_tooltip::the_overlay layout_scroll::orientation` |
| L3 | **P8-ARV**（W12・W13・W17 と marker） | `crates/arvane-compose/src/{gpu/state.rs, plan/ops.rs}`、`crates/arvane-render/src/{draw.rs, draw/canvas.rs, draw/painter.rs, gpu/encode.rs}`、`crates/arvane-shell/src/{a11y/shared.rs, clicks.rs, window.rs}`、`crates/kivact-shader/src/{entry.rs, assemble.rs}` | なし（P6-1 の `arvane-compose/src/{cpu/compositor.rs, gpu/exec.rs}` とはファイルが重ならない） | `cargo clippy -p arvane-render --all-targets --all-features -- -D warnings`；`cargo check -p arvane-compose -p arvane-shell -p kivact-shader --all-targets`；`cargo test -p kivact-shader --doc`；state-fields `--crate` を 4 package で |
| L4 | **P8-IN**（marker 16 行） | `crates/kivact-interaction/src/{dialog.rs, drag/handle.rs, menubar.rs, table/data.rs, table/model.rs, toaster/column.rs, toaster/data.rs}` | **P5 の IN-2 の後**（`table/data.rs`） | `cargo check -p kivact-interaction`；state-fields `--crate kivact-interaction` |
| L5 | **P8-CK**（3.1・3.2 `CheckableState`） | `crates/kivact-ui/src/{values.rs, toggle.rs, radio_group.rs, look.rs, look_controls.rs, look_radio.rs, lib.rs}`、`crates/kivact-ui-basic/src/{recipes.rs, recipes_controls.rs}`、`crates/kivact-ui-creative/src/{recipes.rs, recipes_controls.rs}`、`crates/kivact-ui-game/src/{recipes_controls.rs, recipes_extra.rs, choice.rs}`、`crates/kivact-ui-pop/src/{recipes_controls.rs, recipes_extra.rs}`、`crates/kivact-ui-soft/src/{recipes_controls_choice.rs, segmented_control.rs, indicator.rs}`、`dev/tests/kivact-public-api/tests/e2e/{ui_theming.rs, ui_controls.rs}` | **P5 の UI-3 の後**（UI-3 は F-T4 を行わない。3.2 末尾）。`indicator.rs` は、この後 FINAL-1 も変える | `cargo check -j 6 --all-targets -p kivact-ui -p kivact-ui-basic -p kivact-ui-pop -p kivact-ui-soft -p kivact-ui-game -p kivact-ui-creative`；`cargo check -p kivact-public-api-tests --tests`；`cargo clippy -p kivact-ui --all-targets -- -D warnings`；`cargo test -p kivact-ui --doc radio_group`；state-fields `--crate kivact-ui`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- ui_controls:: ui_theming:: interaction_toggle::` |
| L5 | **P8-UI5**（W01・W02 と marker） | `crates/kivact-ui/src/{code_block.rs, data_table/rows.rs, data_table/runtime.rs, data_table/types.rs, date_input/view.rs}` | **P5 の UI-1 と UI-3 の後**、P8-CK の後 | `cargo clippy -p kivact-ui --all-targets -- -D warnings`；state-fields `--crate kivact-ui`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- ui_data_table:: apg_grid::group`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-soft --test consumer -- grouped_table` |
| L6 | **P8-UI6**（marker 約 40 行） | `crates/kivact-ui/src/{calendar/navigation.rs, calendar/view.rs, carousel.rs, carousel_geometry.rs, combobox/data.rs, command_palette.rs, data_table_tokens.rs, date_input/machine.rs, date_range_draft.rs, dialog.rs, group_types.rs, hover_card.rs, item_row.rs, navigation_values.rs, pagination.rs, selection_field.rs, time_input/machine.rs}` | **P5 の UI-4 の後**（`carousel.rs`、`combobox/data.rs`、`navigation_values.rs`）。**P6-7 の後**（`calendar/view.rs`） | `cargo check -p kivact-ui`；state-fields `--crate kivact-ui` |
| L6 | **P8-SOFT**（W08・W09 と marker、P8-O3） | `crates/kivact-ui-soft/src/{alert_dialog.rs, app_shell.rs, bottom_sheet.rs, citation.rs, container_reveal.rs, dialog_header.rs, disclosure_appearance.rs, item_values.rs, layout.rs, link.rs, markdown.rs, markdown_parser.rs, mobile_nav.rs, nav_heading.rs, outline.rs, power_search.rs, recipes_metrics.rs, rich_block_doc.rs, side_navigation/items.rs, side_navigation/shell.rs, soft_chat_tools.rs, soft_tree.rs, stack.rs, tab_list.rs, timestamp.rs, top_nav.rs}`、`crates/kivact-ui-soft/tests/consumer/navigation.rs` | **P5 の SO の後**（`side_navigation/shell.rs`） | `cargo clippy -p kivact-ui-soft --all-targets -- -D warnings`；`cargo check -p kivact-ui-soft-catalog`；state-fields `--crate kivact-ui-soft`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-soft --test consumer -- layout_` |
| L6 | **P8-KITS**（marker：basic・creative・examples） | `crates/kivact-ui-basic/src/{attachment.rs, chart/model.rs, questionnaire.rs, sidebar/menu.rs, sidebar/panel.rs}`、`crates/kivact-ui-creative/src/{dock/model.rs, editors/arrangement/model.rs, editors/node/model.rs, editors/time/view.rs, outliner/view.rs, outliner_data.rs, picker.rs}`、`examples/kivact-ui-basic/src/docs/{data_table.rs, drawer.rs, dropdown_menu.rs, trace.rs}`、`examples/kivact-ui-creative/src/creative/layout.rs`、`examples/kivact-ui-game/src/game/{controls.rs, system.rs, window.rs}`、`examples/kivact-ui-soft/src/model.rs`、`examples/kivact/component/src/main.rs` | **P5 の EX の後**（`sidebar/menu.rs`、game の `controls.rs`・`window.rs`） | `cargo check -p kivact-ui-basic -p kivact-ui-creative -p kivact-ui-basic-catalog -p kivact-ui-creative-catalog -p kivact-ui-game-catalog -p kivact-ui-soft-catalog -p kivact-example-component`；state-fields `--crate` を各 package で |
| P6 の lane | **P8-GPU**（W15・W16 と、P6-1・P6-3 のファイルにある marker） | `crates/kivact-types/src/{gpu.rs, lib.rs}`、`crates/kivact-core/src/{gpu_interop.rs, gpu_interop/import/image.rs}`、`crates/kivact-gpu-share/src/{common.rs, lib.rs, device.rs, vulkan_raw.rs, import/dx12.rs, import/vulkan.rs, vulkan/physical_device.rs, d3d12/adapter.rs, wgpu/device.rs, wgpu/pool.rs, pool.rs}`、`crates/arvane-compose/src/gpu/context.rs`、`crates/arvane/src/{window.rs, presentation.rs}`、`crates/arvane/tests/e2e/external.rs`、`dev/tests/kivact-public-api/tests/e2e/{gpu_interop.rs, drawable_export_gpu.rs}`、`dev/tests/gpu-interop-producer/src/{d3d12.rs, vulkan.rs, producer.rs, wgpu_engine.rs}`、`docs/for-kivact-user/{en,jp}/how-to-guide/gpu-texture-sharing.md` | **P6-1・P6-3 の後**（承認されれば P6-8 の後も）。P6-1 が `arvane/src/window.rs` の field を消すか改名した場合、その field の行は不要。残る field にだけ付録 B の文を置く。P6 の新しいコードが `GpuAdapter` の field を読んでいれば、`adapter.backend` を `adapter.backend()` に、`luid` などを `adapter.ids` の match に直す（compile error で示される） | `cargo check -p kivact-types -p kivact-gpu-share --all-features --all-targets`；`cargo check -p arvane -p arvane-compose --all-targets --features gpu`；`cargo check -p kivact --features gpu-interop`；`cargo check -p kivact-public-api-tests --tests --features gpu-interop`；`cargo check -p gpu-interop-producer`；`cargo test -p kivact-types --doc GpuAdapter`；state-fields `--crate` を kivact-types・kivact-gpu-share・arvane で |
| — | FINAL-1（P5） | （P5 のとおり） | **P8 の全 unit の後** | （P5 のとおり） |
| — | FINAL-2（P5） | （P5 のとおり） | **P8 の全 unit の後** | P5 のとおり。加えて、X4・X5 が本書の marker で error 0 であること。error が出たら本書の誤りなので、STOP して報告する |

lane の重さ（P8 で足すファイル数）：
- L1：+28
- L2：+38
- L3：+33
- L4：+7
- L5：+26
- L6：+66（marker だけのファイルが大半）
- P6 の lane：+27

marker だけを書き換える unit（P8-POP、P8-RT、P8-IN、P8-UI6、P8-KITS）は、付録 B の文を写すだけなので、機械的な作業である。

## 7 既存欠陥

| # | file:line | 内容 | Slim-Fast で入ったか | 修正 |
|---|---|---|---|---|
| 1 | `crates/kivact-ui/src/radio_group.rs:241` | radio item の `invalid` が常に false。`radio_group(..).invalid(..)` も、error のある `field` も、item の recipe に届かない。標準 look の Danger 枠（`look_controls.rs:51`）は一度も描かれない | いいえ（5198b99d から） | 3.2 の書き手 2。E2E は 5.1 |
| 2 | `crates/kivact-engine/src/layer.rs:71-72` | `Look.filtered` の doc と marker は「filter, masks or a backdrop filter」と書くが、実装（`:175`）は filter だけを見ている | 文言は U61 で写された | doc を `/// It has a CSS filter (it needs a layer).` にする。marker は付録 B（P8-EN） |
| 3 | `crates/kivact-engine/src/tree/node.rs:186` | `update_place` の doc の「The only writer of `Place::transformed`」は、`node_tree.rs:402` の `raise_under_perspective` と矛盾する | 不明（doc のみ） | doc を「The writer that clears `Place::transformed` (`NodeTree::raise_under_perspective` only raises it).」にする（P8-EN） |
| 4 | `crates/kivact-style/src/sheet/fields.rs:128-129` | `FlowFields.display` の doc の「set by `grid(..)` only」が誤り。`Style::display`（`setters.rs:846`）もある | いいえ | doc を `/// The display mode: flex (default), grid or a transparent container (\`display(..)\`, \`grid(..)\`).` にする（P8-L1） |
| 5 | 3.5 の 21 件の位置の marker（49 行） | 偽の主張（「独立」「計数」「投影」）。設計書 SM-11 と SL-8 の違反 | はい（U61 の接頭辞の機械的な付与） | 3.5 と付録 A |
| 6 | 付録 B の WORDING 284 行 | 理由文が事実と違うか、組合せの理由が無い。例：`TreeDrop.target` の「the application's new parent」（実際は widget の `drop_plan` が計算する）、`table::page` の「separate controls」（実際は新しい sort が page を 0 に戻す） | はい（同上） | 付録 B の文 |

## 8 設計者（人間）の判断が要る点

1. **`TimestampEntry.copyable`（`crates/kivact-ui-soft/src/timestamp.rs:171-173`、公開）**
   - 事実：doc は「Whether the detail value offers a copy action.」と約束しているが、copy 操作は d4d65c10（Slim-Fast 以前）から実装されていない。読み手も 0 件である。
   - 本書の扱い：V2 では field を残し、marker を事実どおりの option にした（P8-O3）。
   - 選択肢（どちらにするか判断が要る）：
     - (a) 機能を実装する。details の各行に copy の button を置き、`code_block` と同じ `use_copy_feedback` で clipboard に書く。kit の機能を増やすことになる。
     - (b) pub field を削除する。kit の公開 API の縮小で、方針 5 に触れる。
   - 推奨は (a) を V2 の範囲外の kit の作業として起票すること。
2. **P8-GPU の時期**：P6-8（`GpuPreference` の移動、P6 の 8 章で承認待ち）が承認されなかった場合は、P6-1 と P6-3 の後だけでよい。どちらでも本書の設計は変わらない。

---
## 付録 A-G2：G2（kivact-ui） の WRONG 詳細

対応：W1 = P8-W01, W2 = P8-W02。本文 3.3・3.4 の上書きがある場合は本文が正。


対象は 2 件、どちらも `crates/kivact-ui/src/data_table/runtime.rs` の private struct `ModelPlan` にある。公開 API・振る舞いは変わらない（内部の型の整理だけ）。所有ファイルは `crates/kivact-ui/src/data_table/runtime.rs` の 1 ファイルだけ（W1・W2 共通）。

---

#### W1：ModelPlan.page と ModelPlan.page_size（SM-2：含意）

##### 事実
- `runtime.rs:177-192` `paging()`:
  - `page_size = table.page.and_then(|(index, _)| table.page_sizes.map(..))`。つまり **page_size が Some なら table.page は Some**。
  - `page = table.page.map(|(index, size)| (index, page_size.map_or(size, ..)))`。page の Some/None は table.page と同じ。
- 到達する組合せは (None, None)、(Some, None)、(Some, Some) の 3 つ。**(page=None, page_size=Some) は到達しない**。page_size は「page が有る時だけ意味のある Option」（SM-2）。
- 書き手：`model_plan` の `runtime.rs:151`（`let (page, page_size) = paging(table);`）と ModelPlan literal（`runtime.rs:165-174`）だけ。
- 読み手：
  - page：`runtime.rs:152-156`（model.page）、`:201`（table_paging）、`:292`（metadata::rows）、`:307`（sorting）、`:458`（row_indexes）、`:506`（grid_plan の Filters.page）。
  - page_size：`runtime.rs:205`（table_paging の Paging::Local）だけ。

##### 新しい型（runtime.rs、`ModelPlan` の直前に追加）
```rust
/// The page plan, with the size binding when the caller also offered page
/// sizes; that binding resets the page when a size is chosen.
#[derive(Clone, Copy)]
struct PagingPlan {
    page: super::types::PagePlan,
    size_choice: Option<Binding<usize>>,
}
```
（PagingPlan の数える field は size_choice 1 個なので marker 不要。`PagePlan = (Binding<usize>, Value<usize>)` と `Binding` は Copy なので derive できる。）

##### ModelPlan の変更（runtime.rs:32-47）
`page` と `page_size` の 2 field（とその marker）を削除し、次の 1 field に置換：
```rust
    // xtask-allow(state-fields): option: DataTable::page or remote_page with any page_sizes choice; absent shows every row with no pager
    paging: Option<PagingPlan>,
```
さらに読み手の書換えを小さくするため accessor を追加：
```rust
impl<R: 'static, K: 'static> ModelPlan<R, K> {
    /// The page shown and its size, when the table pages its rows.
    fn page(&self) -> Option<super::types::PagePlan> {
        self.paging.map(|paging| paging.page)
    }
}
```

##### paging() の置換（runtime.rs:177-192）
```rust
fn paging<R, K>(table: &DataTable<R, K>) -> Option<PagingPlan>
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    table.page.map(|(index, size)| {
        let size_choice = table.page_sizes.map(|(choice, _)| {
            binding(choice, move |next: usize| {
                choice.set(next.max(1));
                index.set(0);
            })
        });
        PagingPlan {
            page: (index, size_choice.map_or(size, |choice| choice.into_value())),
            size_choice,
        }
    })
}
```
（旧コードと同じ binding・同じ setter。`size.set(next.max(1)); index.set(0);` の順序を保つ。）

##### 書き手・読み手の書換え（runtime.rs）
| 行 | 現在 | 変更後 |
|---|---|---|
| 151 | `let (page, page_size) = paging(table);` | `let paging = paging(table);` |
| 152-156 | `if let Some((index, size)) = page && table.has_more.is_none() { model = model.page(index, size); }` | `if let Some(PagingPlan { page: (index, size), .. }) = paging && table.has_more.is_none() { model = model.page(index, size); }` |
| 165-174 literal | `page, page_size,` | `paging,` |
| 201-209 table_paging | `match (plan.page, has_more) { (Some((page, _)), Some(has_more)) => Remote{..}, (Some((page, size)), None) => Local { page, page_size: plan.page_size.unwrap_or_else(|| binding(move || size.get(), |_| ())) }, _ => Off }` | `match (plan.paging, has_more) { (Some(PagingPlan { page: (page, _), .. }), Some(has_more)) => Paging::Remote { page, has_more }, (Some(PagingPlan { page: (page, size), size_choice }), None) => Paging::Local { page, page_size: size_choice.unwrap_or_else(|| binding(move || size.get(), |_| ())) }, _ => Paging::Off }` |
| 292 | `plan.page,` | `plan.page(),` |
| 307 | `sorting(table, plan.model, plan.page, behavior)` | `sorting(table, plan.model, plan.page(), behavior)` |
| 458 | `let page = plan.page;` | `let page = plan.page();` |
| 506 | `page: plan.page.map(|(index, _)| index),` | `page: plan.page().map(|(index, _)| index),` |

`page_size` を読む箇所は 205 だけなので他に変更なし。`sorting()`・`metadata::rows()`・`row_indexes()` の引数型は `Option<PagePlan>` のまま。

##### 公開 API・振る舞い
- 公開 API：変更なし（private struct と private fn だけ）。
- 振る舞い：変更なし。binding の生成数・setter・Paging の分岐は同一。

##### 担保する既存 E2E／試験
- `dev/tests/kivact-public-api/tests/e2e/ui_data_table.rs:513` `row_numbers_follow_the_displayed_page_and_page_size_changes_reset_it`（page_sizes の size_choice 経路と page リセット）。
- `dev/tests/kivact-public-api/tests/e2e/ui_data_table.rs:541` `a_remote_page_does_not_slice_the_rows_again_and_uses_has_more`（Remote 分岐）。
- `crates/kivact-ui-soft/tests/consumer/grouped_table.rs:49,131`（page と group の併用）。
- 追加 E2E：不要（page_sizes 無しの Local は既存 page 試験が通る。page_sizes を page 無しで呼んで無視される経路は旧コードでも無視で、型上表現不能になるだけ）。

---

#### W2：ModelPlan.groups と ModelPlan.grouping_key（SM-3：同時）

##### 事実
- `runtime.rs:158` `grouping_key = table.grouping.as_ref().map(|g| g.key.clone())`
- `runtime.rs:159-162` `groups = table.grouping.take().map(|g| super::grouped::plan(page_rows, g))`
- 両方とも `table.grouping` の Some/None をそのまま写すので **常に同時に Some/None**（SM-3）。(Some, None)・(None, Some) は到達しない。
- 書き手：上記 2 行と ModelPlan literal（`runtime.rs:165-174`）だけ。
- 読み手：groups は `runtime.rs:99`（mount → regions）、`:163`（display）、`:361`（selection_policy）。grouping_key は `runtime.rs:291`（metadata::rows）だけ。
- `grouped::Plan<R>` は Copy（`grouped.rs:38-44`）。`Key<R, String>` は `Rc<dyn Fn(&R) -> String>`（`types.rs:19`）で Clone。

##### 新しい型（runtime.rs、`ModelPlan` の直前に追加）
```rust
/// The page's groups and the key that forms them; both exist exactly when
/// the caller called `DataTable::group_by`.
struct GroupingPlan<R: 'static> {
    groups: super::grouped::Plan<R>,
    key: super::types::Key<R, String>,
}
```
（Option field を持たないので marker 不要。）

##### ModelPlan の変更（runtime.rs:32-47）
`groups` と `grouping_key` の 2 field（とその marker）を削除し、次の 1 field に置換：
```rust
    // xtask-allow(state-fields): option: DataTable::group_by, as the page's groups and their key together; absent shows the page's rows ungrouped
    grouping: Option<GroupingPlan<R>>,
```

##### 書き手・読み手の書換え（runtime.rs）
| 行 | 現在 | 変更後 |
|---|---|---|
| 158-162 | `let grouping_key = ...; let groups = table.grouping.take().map(|grouping| super::grouped::plan(page_rows, grouping));` | `let grouping = table.grouping.take().map(|grouping| GroupingPlan { key: grouping.key.clone(), groups: super::grouped::plan(page_rows, grouping) });` |
| 163 | `let display = groups.map_or(page_rows, |plan| plan.rows());` | `let display = grouping.as_ref().map_or(page_rows, |grouping| grouping.groups.rows());` |
| 165-174 literal | `groups, ... grouping_key,` | `grouping,` |
| 99 | `plan.groups,` | `plan.grouping.as_ref().map(|grouping| grouping.groups),` |
| 291 | `plan.grouping_key.clone(),` | `plan.grouping.as_ref().map(|grouping| grouping.key.clone()),` |
| 361 | `if plan.groups.is_some() {` | `if plan.grouping.is_some() {` |

注意：GroupingPlan の literal では `key: grouping.key.clone()` を `groups: super::grouped::plan(page_rows, grouping)`（grouping を move する）より**先に**書く（Rust の field 初期化は記述順に評価されるため、clone を先に評価させる）。`regions()`・`metadata::rows()` の引数型（`Option<grouped::Plan<R>>`、`Option<Key<R, String>>`）は変えない。

##### 公開 API・振る舞い
- 公開 API：変更なし。
- 振る舞い：変更なし（同じ key の clone、同じ grouped::plan 呼出し）。

##### 担保する既存 E2E／試験
- `dev/tests/kivact-public-api/tests/e2e/apg_grid.rs:325` `group_disclosures_participate_in_grid_arrow_navigation`（grouping_key → metadata::rows の行位置）。
- `crates/kivact-ui-soft/tests/consumer/grouped_table.rs:49` `group_disclosure_preserves_selection_and_pages_and_skips_collapsed_rows`、`:131` `remote_pages_group_the_supplied_rows_without_slicing_them_again`（groups → selection_policy の page_keys、display）。
- 追加 E2E：不要。

---

#### 参考：WRONG にしなかった境界例（WORDING で依存を明記）
authored な builder 設定どうしの「片方が無いと無視される」依存は、実行時に変わる状態ではなく、型にすると公開 builder API の形を変えるため WORDING とした。
- `DataTable.expanded_rows` は `details` が無いと無視（runtime.rs:274, rows.rs:661-667）。`DataTable.resize_range` は `widths` が無いと無視（runtime.rs:308-312）。`Grid.expanded_rows` も同様。
- `Calendar.month`（default_month）は `displayed_month` が有ると無視（calendar/view.rs:105,303-305）。
- `Combobox.open` は mode=Inline で無視（combobox/render.rs:557-563）。`Combobox.matcher` は search 有りで使われない（render.rs:374）。
- `RowContents.description` は `description_content` が優先、`press` は `delegate` が優先（item_row.rs:195-199,265-269）。


## 付録 A-G3：G3（kivact-engine） の WRONG 詳細

対応：W1 = P8-W03, W2 = P8-W04。本文 3.3・3.4 の上書きがある場合は本文が正。


WRONG は 2 件（6 行）。

- W1：`Dispatched.paint` / `Dispatched.fonts`（model/changes.rs:148, :152）
- W2：`Source.multiline` / `Source.wrap_width` / `Key.multiline` / `Key.wrap_width`（text_input/source.rs:22, :25、text_input/records.rs:24, :26）

---

#### W1：Dispatched の fonts ⇒ paint（隠れた含意）

##### 事実
- `Model::sync`（crates/kivact-engine/src/model/interact/sync.rs:96-178）
  - :127 `if self.changes.fonts { … }` の中で、:148 `self.changes.paint = true;` を立てる。
  - :156 `let changes = self.changes.dispatch();` が paint・restack・fonts を `std::mem::take` で `Dispatched` に移す（changes.rs:218-233）。
  - :148 と :156 の間に `changes.paint` を下げる処理は無い。
- したがって `Dispatched` では **fonts=true ⇒ paint=true**。`(fonts=T, paint=F)` は到達しない。marker の「any set of them can be pending」は偽。
- `Dispatched.paint` の読み手は sync.rs:159 だけ：`let redraw = changes.paint || changes.fonts || moved || changes.any(...)`。fonts は既に redraw に入っているので、:148 で paint を立てる意味は無い（冗長な書込みが含意を作っている）。
- `Dispatched.fonts` の読み手は sync.rs:160（redraw）と :176（`outdate_paint(..., changes.fonts, ...)` → invalidation.rs:96 `forget_inks`）。
- `Dispatched` は `outdate_paint` → `PaintList::invalidate`（paint/list/invalidation.rs:20, :102）にも渡るが、そこは `paint`/`fonts` field を読まない（`marked`/`any` だけ）。record_sync（sync.rs:182）も field を読まない。
- `Changes.paint` の他の書き手（sync.rs:85 invalidate_paint、:273 images、layout_motion/steps.rs:208）は影響を受けない。`Changes` 自体では sync 前に (fonts=T, paint=F) が成立するので Changes の 3 field は independent のまま（WORDING）。

##### 閉じ方
書き手の削除（冗長な含意を作る 1 行を消す）。型の変更は不要。

##### コード変更
1. crates/kivact-engine/src/model/interact/sync.rs:148 の 1 行を削除する。

```rust
        if self.changes.fonts {
            let texts: Vec<NodeId> = /* … 現状のまま … */;
            for id in texts {
                /* … 現状のまま … */
            }
            // 削除: self.changes.paint = true;
        }
```

2. marker 行を差し替える（changes.rs）。
   - :148 `Dispatched.paint`
     `// xtask-allow(state-fields): independent: Changes::dispatch takes Changes::paint as raised by its own inputs (host repaints, images, layout motion); a font registration no longer raises it, so it pairs with any fonts and restack`
   - :150 `Dispatched.restack`（WORDING 行と同じ）
     `// xtask-allow(state-fields): independent: Changes::dispatch takes Changes::restack (paint-order writes and motions) as is; paint and fonts come from other inputs, so any set occurs`
   - :152 `Dispatched.fonts`
     `// xtask-allow(state-fields): independent: Changes::dispatch takes Changes::fonts (Model::fonts_added) as is; sync no longer raises paint with it, so it pairs with any paint and restack`
   - Changes の 3 行（:136, :138, :140）は G3.out.tsv の WORDING 行どおり。

##### 振る舞い
不変。`redraw = paint || fonts || …` なので fonts のときの redraw は従来どおり true。`outdate_paint` に渡る `fonts` も同じ。DevTools の sync record は paint field を記録しないので差は出ない。

##### 公開 API
影響なし（`Changes`/`Dispatched` は pub(crate)/pub(super)）。

##### E2E
- 既存：crates/kivact-test/src/screen/scene_cache_tests.rs:112 `registering_a_font_after_a_cached_canvas_frame_replaces_fallback_shaping`（font 登録後の再描画を確認）。
- 追加推奨：font 登録だけを行い（他の書込みなし）、次の frame でテキストの画素が変わることを確かめる実 window の E2E（paint の明示 invalidation 無しで redraw されることの担保）。上の既存試験が font 登録のみで再描画を見ていれば追加不要（実装者が確認すること）。

##### 所有ファイル
- crates/kivact-engine/src/model/interact/sync.rs
- crates/kivact-engine/src/model/changes.rs

---

#### W2：text input の multiline と wrap_width（SM-2：bool が true の時だけ意味のある Option）

##### 事実
- `Source` の構築は `refresh_lines`（crates/kivact-engine/src/measure/input.rs:48-108）の 1 か所だけ（他は text_reference.rs:21 の `..snapshot.source` 複写）。
  - :67 `let wrap = n.edit_line_layout().wrap;`
  - :91 `multiline: n.multiline(),`
  - :92 `wrap_width: width.filter(|width| wrap && width.is_finite() && *width > 0.0),`
- `HostNode::edit_line_layout`（crates/kivact-engine/src/tree/node.rs:399-407）は単一行なら `EditLineLayout::default()`（`wrap: false`、kivact-types layout.rs:276-281）を返す。よって **wrap_width.is_some() ⇒ multiline**。`(multiline=false, wrap_width=Some)` は到達しない。
- `Source.multiline` marker「any combination is valid」は偽。`Source.wrap_width` は option と称するが、値は authored ではなく refresh ごとの viewport 幅（input.rs:69-72）。
- `Key::of`（crates/kivact-engine/src/text_input/records.rs:33-56）が :52-53 で Source の 2 値をそのまま写すので、`Key` にも同じ含意がある。
- 読み手：
  - text_input/chunks.rs:76 `split(text, 0, text.len(), self.key.multiline, true)`
  - text_input/chunks.rs:77 `let Some(width) = self.key.wrap_width else {`
  - text_input/chunks.rs:145 `let multiline = self.key.multiline;`
  - text_input/refresh.rs:96 `let fits = self.key.wrap_width.is_none()`
  - text_input/shaping.rs:334 `if self.key.wrap_width.is_some() {`
  - `source.multiline` / `source.wrap_width` を直接読むのは records.rs:52-53 だけ。

##### 閉じ方
型：二つの field を一つの enum `Breaks` にまとめる（`Source` と `Key` の両方）。

##### 新しい型（crates/kivact-engine/src/text_input/source.rs に追加）

```rust
/// How the shown text breaks into lines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Breaks {
    /// One line: an unmasked LF or CR shows as a space.
    Single,
    /// Split at LF hard breaks; `wrap` is the content width the rows also
    /// soft-wrap at, when the field wraps.
    Lines { wrap: Option<f64> },
}

impl Breaks {
    /// Whether the text splits at LF hard breaks.
    pub(crate) fn multiline(self) -> bool {
        matches!(self, Breaks::Lines { .. })
    }

    /// The width the rows soft-wrap at, if they do.
    pub(crate) fn wrap_width(self) -> Option<f64> {
        match self {
            Breaks::Single => None,
            Breaks::Lines { wrap } => wrap,
        }
    }
}
```

（variant の field は `wrap` 1 個なので lint の 2 個以上には当たらない。`Breaks` は状態名の enum ではない。）

##### 書き手・読み手の全変更
1. crates/kivact-engine/src/text_input/source.rs
   - :22-27（`multiline` の marker・doc・field と `wrap_width` の marker・doc・field）を削除し、次に置き換える：
     ```rust
     /// Hard breaks at LF and soft wrapping (an unmasked single line shows
     /// LF/CR as spaces).
     pub(crate) breaks: Breaks,
     ```
   - 上の `Breaks` 定義を同ファイルに追加。
   - `damage`（:12）と `password`（:17）の marker は残り、G3.out.tsv の WORDING 行の文に差し替える（damage・password・breaks のうち数えられるのは damage と password の 2 個なので marker は必要）。
2. crates/kivact-engine/src/text_input.rs:119 `pub(crate) use source::Source;` → `pub(crate) use source::{Breaks, Source};`
3. crates/kivact-engine/src/measure/input.rs
   - :4 `use crate::text_input::{EditLines, RefreshError, RefreshMode, Source};` → `use crate::text_input::{Breaks, EditLines, RefreshError, RefreshMode, Source};`
   - :91-92 を置換：
     ```rust
     breaks: if n.multiline() {
         Breaks::Lines {
             wrap: width.filter(|width| wrap && width.is_finite() && *width > 0.0),
         }
     } else {
         Breaks::Single
     },
     ```
4. crates/kivact-engine/src/text_input/records.rs
   - :24-27（`multiline` と `wrap_width` の marker と field）を削除し、`pub(crate) breaks: crate::text_input::source::Breaks,` を置く（marker 不要：Key の数える field が無くなる）。
   - :52-53 `multiline: source.multiline, wrap_width: source.wrap_width,` → `breaks: source.breaks,`
   - :18-19 の doc「whether they break at line feeds」はそのままで可（breaks がそれを表す）。
5. crates/kivact-engine/src/text_input/chunks.rs
   - :76 `self.key.multiline` → `self.key.breaks.multiline()`
   - :77 `let Some(width) = self.key.wrap_width else {` → `let Some(width) = self.key.breaks.wrap_width() else {`
   - :145 `let multiline = self.key.multiline;` → `let multiline = self.key.breaks.multiline();`
6. crates/kivact-engine/src/text_input/refresh.rs:96 `self.key.wrap_width.is_none()` → `self.key.breaks.wrap_width().is_none()`
7. crates/kivact-engine/src/text_input/shaping.rs:334 `self.key.wrap_width.is_some()` → `self.key.breaks.wrap_width().is_some()`
8. 確認：`rg "\.multiline\b|wrap_width" crates/kivact-engine/src/text_input crates/kivact-engine/src/measure/input.rs` で残りが無いこと（measure/leaf.rs:275 の local `wrap_width` は別物で変更不要。`n.multiline()`（HostNode の accessor）も変更不要）。

##### 振る舞い
不変。`n.multiline()` が false なら従来も `wrap` が false で `wrap_width` は None だった（tree/node.rs:400-405）。true なら従来どおりの幅を `Lines { wrap }` に入れる。`Key` の等値比較（refresh.rs:91 `self.key != key`）も、到達可能な値の組では従来と同じ結果。

##### 公開 API
影響なし（すべて pub(crate)）。

##### E2E
- 既存：crates/kivact-ui-basic/tests/consumer/foundation.rs:138 `text_input_edits_its_binding_and_rings_on_focus`（multiline 入力の行数・高さに触れる、:162）。debug の `RefreshMode::Verify`（host/testing/text_reference.rs:15-）が保持行と再構築行を比較するので、Key の取り違えは試験時に検出される。
- 追加推奨：`line_layout(wrap: true)` の multiline 入力に長い 1 行を入れ、幅を変えると行（row）数が変わり、単一行入力では LF が空白で表示され改行しないことを確かめる実 window の E2E（kivact-ui-basic の consumer 試験に置く）。

##### 所有ファイル
- crates/kivact-engine/src/text_input/source.rs
- crates/kivact-engine/src/text_input.rs
- crates/kivact-engine/src/text_input/records.rs
- crates/kivact-engine/src/text_input/chunks.rs
- crates/kivact-engine/src/text_input/refresh.rs
- crates/kivact-engine/src/text_input/shaping.rs
- crates/kivact-engine/src/measure/input.rs

---

#### 参考（WRONG ではないが注意）
- lower/context.rs:57 `Context.dropped` は statistic と称しつつ制御に使われる（scene cache 挿入の分岐 scene/paint.rs:74、offscreen.rs:150 の Err）。隠れた状態機械ではなく、コードを変えずに prefix を independent にすれば閉じるため KIND とした。
- layer.rs:71 `Look.filtered` の doc comment（`/// It has a CSS filter, masks or a backdrop filter (each needs a layer).`）は実装（filter のみ）と違うので、marker と一緒に doc も `/// It has a CSS filter (it needs a layer).` に直すこと。
- tree/node.rs:186 `update_place` の doc「The only writer of `Place::transformed`」は node_tree.rs:402 `raise_under_perspective` と矛盾する。doc を「The writer that clears `Place::transformed` (`NodeTree::raise_under_perspective` only raises it).」に直すこと。


## 付録 A-G4：G4（kivact-core） の WRONG 詳細

対応：W1 = P8-W05, W2 = P8-W06, W3 = P8-W07。本文 3.3・3.4 の上書きがある場合は本文が正。


対象 HEAD 73c36c7c。3 件の owner（8 行）。どれも公開 API は不変（pub(crate) / private のみ）、振る舞いは不変。

---

#### W1. `Selected`（crates/kivact-core/src/view/mount/part_input.rs:37-46）— 行 65・66・67

##### 事実
- 書き手は `select`（part_input.rs:50-68）だけ。読み手は `deliver_selected`（part_input.rs:75-116）だけ。呼び出し元は key/key_up router（part_input.rs:163-165）と wheel router（part_input.rs:176-183）。
- `select` の場合分け（入力 `own_canvas`, `region`）:

| own_canvas | region | lookup | observed | target |
|---|---|---|---|---|
| false | 任意 | （しない） | None | Some(root) |
| true | None | （しない） | Some(root) | Some(root) |
| true | Some(id) | Some(t) | Some(t) | Some(t) |
| true | Some(id) | None | None | None |

- 到達しない組合せ: `observed=Some` かつ `target=None`、`observed=Some(a)` かつ `target=Some(b)`（a≠b）、`own_canvas=false` かつ `observed=Some`、`own_canvas=false` かつ `target≠root`。marker の「any combination with the targets is valid」は誤り。3 field は 3 通りの状況を表す暗黙の enum（SM-1/SM-4）。

##### 新しい型（part_input.rs:35-46 を置換）
```rust
/// Where one event goes.
enum Selected {
    /// The event did not come from this node's canvas (no origin, or another
    /// node's): the node's own target takes the defaults; no target observes it.
    Bubbled,
    /// The event came from this node's canvas: `0` (the node's own target, or
    /// the region the event names) observes it and takes the defaults.
    Own(Rc<Target>),
    /// The event names a region of this canvas the registry does not have:
    /// only the node's own entries run.
    Unknown,
}
```
（3 つの marker 行は削除。新しい enum は数える field を持たない。）

##### 書き手の変更（part_input.rs:48-68 `select`）
```rust
/// Region identifiers belong to their original canvas. Ancestors receive
/// bubbling through node defaults without retargeting.
fn select(registry: &Registry, own_canvas: bool, region: Option<RegionId>) -> Selected {
    if !own_canvas {
        return Selected::Bubbled;
    }
    match region {
        None => Selected::Own(registry.root.clone()),
        Some(id) => registry.lookup(id).map_or(Selected::Unknown, Selected::Own),
    }
}
```
（`Registry::lookup` は `Option<Rc<Target>>` を返す: regions.rs:53。）

##### 読み手の変更（part_input.rs:75-116 `deliver_selected`）
```rust
fn deliver_selected<E: Clone + 'static>(
    list: fn(&Handlers) -> &EventHandlers<E>,
    own: &EventHandlers<E>,
    root: &Target,
    selected: &Selected,
    event: &E,
) -> crate::Result<Handled> {
    // The target whose observers see the event, and the target whose defaults run.
    let (observed, target): (Option<&Target>, Option<&Target>) = match selected {
        Selected::Bubbled => (None, Some(root)),
        Selected::Own(target) => (Some(&**target), Some(&**target)),
        Selected::Unknown => (None, None),
    };
    let observing = || observed.is_some_and(Target::live);
    let target_ready = || target.is_some_and(Target::enabled);
    let fallback_ready = || target_ready() && root.enabled();
    let mut sources = vec![Source::new(own, &Layer::NODE, &open)];
    if let Some(observed) = observed {
        sources.push(Source::new(
            list(&observed.handlers),
            &[Layer::Observer],
            &observing,
        ));
    }
    if let Some(target) = target {
        sources.push(Source::new(
            list(&target.handlers),
            &[Layer::PartDefault],
            &target_ready,
        ));
        if matches!(selected, Selected::Own(_)) && target.region().is_some() {
            sources.push(Source::new(
                list(&root.handlers),
                &[Layer::RegionDefault],
                &fallback_ready,
            ));
        }
    }
    deliver(&sources, event)
}
```
- `Target::live` / `Target::enabled` / `Target::region` は element_data.rs:699-707 付近の `pub(super)`（view 配下から可視、現行コードも同じ関数を使う）。両方 `fn(&self) -> bool`（element_data.rs:706, 709）。
- 呼び出し元（part_input.rs:163-165, 176-183）は変更なし（`select` の引数・戻り値の使い方が同じ）。

##### 振る舞い
不変。旧 `Bubbled` 相当（own_canvas=false）は target=registry.root で、`registry.root` と `root` 引数は同じ `Rc<Target>`（part_input.rs:165, 181 で `&registry.root` を渡す）。region fallback 条件 `own_canvas && target.region().is_some()` は `Own(t)` かつ `t.region().is_some()` と同値（Bubbled の target は root で region() は None: element_data.rs:699-703）。

##### E2E
- 既存: dev/tests/kivact-public-api/tests/e2e/parts.rs の `node_parts_keep_customizations_first_and_observe_consumed_input`（:99）、`canvas_parts_have_independent_focus_actions_input_state_and_geometry_lifetimes`（:170）。
- 追加推奨（推測：現行 E2E が未登録 region と bubbled key を直接突いているか未確認）: 子 canvas の region から bubble した key が親 canvas node の observer に届かず親の part default だけが走ること、および削除済み region id を持つ key が node 自身の handler だけを走らせること。

##### 所有ファイル
- crates/kivact-core/src/view/mount/part_input.rs

---

#### W2. `KeyRoute`（crates/kivact-core/src/view/overlays.rs:512-525）— 行 84・85

##### 事実
- 書き手は `key_route`（overlays.rs:532-552）だけ。`KeyRoute { .. }` の literal は overlays.rs:545 のみ。
- 読み手は `client/dispatch/key.rs` のみ: `route.target`（key.rs:27）、`route.stop`（key.rs:60）、`route.activates`（key.rs:69）。
- `target = named.or(focused).filter(|t| modal.is_none_or(|m| within(t, m))).or(modal)`、`activates = target.is_some()`、`stop = modal`（overlays.rs:541-550）。modal が Some なら target は必ず Some（modal で補完）なので `stop=Some` ⇒ `activates=true`。`(stop=Some, activates=false)` は不到達。marker の「independent of where the chain stops」は誤り（含意 SM-2 型）。
- 到達する状況は 3 つ: modal なしで named/focused あり（chain は root まで、activate する）、modal あり（target は modal 内の named/focused か modal 自身、chain は modal で止まる、activate する）、modal も named/focused もない（App の view か root、activate しない）。

##### 新しい型（overlays.rs:512-525 を置換）
```rust
/// Where a key goes and where its handler chain stops (see [`key_route`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KeyRoute {
    /// The node the host named, else the focused node; no modal is open, so
    /// the chain runs to the root.
    Focused { target: NodeId },
    /// A modal is shown: the named or focused node inside it, else the modal
    /// itself; the chain stops at `modal`.
    Modal { target: NodeId, modal: NodeId },
    /// Nothing is named or focused and no modal is open: the App's view (else
    /// the root) sees the key, so window-level shortcuts work; an unhandled
    /// Enter or Space does not activate it.
    Unfocused { view: NodeId },
}

impl KeyRoute {
    /// The first node of the chain.
    pub(crate) fn target(self) -> NodeId {
        match self {
            Self::Focused { target } | Self::Modal { target, .. } => target,
            Self::Unfocused { view } => view,
        }
    }

    /// The topmost shown modal, where the chain stops.
    pub(crate) fn stop(self) -> Option<NodeId> {
        match self {
            Self::Modal { modal, .. } => Some(modal),
            Self::Focused { .. } | Self::Unfocused { .. } => None,
        }
    }

    /// Whether an unhandled Enter or Space activates the target: not when the
    /// key reached the App's view (or the root) only because nothing has the
    /// focus and no modal is open.
    pub(crate) fn activates(self) -> bool {
        !matches!(self, Self::Unfocused { .. })
    }
}
```
（2 つの marker 行は削除。）

##### 書き手の変更（overlays.rs:532-552 `key_route` の本体）
```rust
pub(crate) fn key_route(
    rt: &Runtime,
    window: kivact_types::WindowId,
    named: Option<NodeId>,
) -> KeyRoute {
    let focused = focus::focused(rt, window).map(|f| f.node);
    let tree = ViewState::tree(rt);
    let modal = tree.reader().active_modal(window);
    let chosen = named.or(focused);
    match modal {
        Some(modal) => KeyRoute::Modal {
            target: chosen.filter(|t| tree.is_within(*t, modal)).unwrap_or(modal),
            modal,
        },
        None => match chosen {
            Some(target) => KeyRoute::Focused { target },
            None => {
                let root = tree.root(window).unwrap_or(NodeId(1));
                KeyRoute::Unfocused {
                    view: tree.reader().children(root).first().copied().unwrap_or(root),
                }
            }
        },
    }
}
```

##### 読み手の変更（crates/kivact-core/src/client/dispatch/key.rs）
- key.rs:27 `let target = route.target;` → `let target = route.target();`
- key.rs:60 `if Some(id) == route.stop {` → `if Some(id) == route.stop() {`
- key.rs:69 `&& route.activates` → `&& route.activates()`
- overlays.rs:24 と :34 の doc link `[`KeyRoute::stop`]` / `[`KeyRoute::activates`]` はメソッドへ解決されるので変更不要。

##### 振る舞い
不変（旧式との対応: modal=None なら target=chosen・activates=chosen.is_some()・chosen=None で view/root へ補完、modal=Some なら target=modal 内の chosen か modal・activates=true）。

##### E2E
- 既存: dev/tests/kivact-public-api/tests/e2e/overlay.rs:230 `keys_go_to_an_open_modal_without_focus_and_stop_there`、interaction_dialog.rs:338 `a_modal_contains_tab_and_blocks_the_page_behind_it`、feedback.rs:68 `card_on_press_activates_with_the_keyboard`。
- 追加推奨: focus も modal も無い状態で Enter を送ると App の view の `on_key` は呼ばれるが、その view の `on_press` は走らないこと（`Unfocused` の activates=false を担保する E2E が見当たらない）。

##### 所有ファイル
- crates/kivact-core/src/view/overlays.rs
- crates/kivact-core/src/client/dispatch/key.rs

---

#### W3. `SemanticsPatch` の排他 tag（crates/kivact-core/src/view/semantics_data.rs:17-33）— 行 88・89・90

##### 事実
- `SemanticsPatch` の literal は `Common::semantic`（modifiers.rs:52-62）だけ。各 helper は `semantic` で新しい patch を push した直後に `last_mut()` へ **1 つだけ** tag を付ける:
  - `input`: set_role 等（modifiers.rs:113, 129, 139, 155, 171, 191, 211, 221, 233, 243, 253）
  - `relation`: semantic_relation（modifiers.rs:79-82）
  - `orientation`: semantic_orientation（modifiers.rs:89-94）
  - `text`: Node::semantics_text（node_modifiers.rs:778-783）
- よって `input≠None`・`relation=Some`・`orientation=Some`・`text=true` は互いに排他（SM-4）。型は 2^3×|input| 通りを許すが到達するのは「どれか 1 つ、または無し」。marker の「a patch the builder may set; absent leaves the prop unchanged」も事実と違う（prop の適用は `apply` が行い、これらは後から読み返すための tag）。
- 読み手: relation は builders.rs:106-123（labelled_by_sources / described_by_sources）、orientation は container.rs:131-136、text は resolve.rs:267 と mount/semantics.rs:66。`SemanticInput` を match する箇所は全て `_` 腕を持つ（builders.rs:73-136, modifiers.rs:116-261, parts.rs:116-120（matches!）, regions.rs:190-196, resolve.rs:116-121, mount/semantics.rs:21-25）。`SemanticInput` を使うのは kivact-core の 7 ファイルのみ（pub(crate)）。

##### 新しい型
semantics_data.rs: `SemanticInput` に 3 variant を足し、`SemanticsPatch` から `text`・`relation`・`orientation` を削除する。
```rust
/// A patch retains its declaration site and its place in the modifier sequence.
#[derive(Clone)]
pub(crate) struct SemanticsPatch {
    // xtask-allow(state-fields): option: set by apply_part on the patches a part declared, false on the caller's own; the *_source readers and part constraints tell the two apart
    pub(crate) part: bool,
    pub(crate) input: SemanticInput,
    pub(crate) layer: usize,
    pub(crate) at: &'static Location<'static>,
    // xtask-allow(state-fields): projection: written only by Common::semantic from whether the patch's source is fixed; the source itself is erased into `apply`
    pub(crate) reactive: bool,
    pub(crate) apply: Rc<dyn Fn(&mut Semantics<FocusTarget>)>,
}

/// What a patch keeps beside `apply` for later readers: the preserved input
/// of a state or name modifier, a relation, an orientation, or the
/// `semantics_text` mark. At most one per patch.
#[derive(Clone)]
pub(crate) enum SemanticInput {
    None,
    Role(kivact_types::Role),
    Name(Prop<String>),
    Description(Prop<String>),
    Selected(Prop<bool>),
    Disabled(Prop<bool>),
    Hidden(Prop<bool>),
    Invalid(Prop<bool>),
    Checked(Prop<bool>),
    Indeterminate(Prop<bool>),
    Expanded(Prop<bool>),
    Focusable(Prop<bool>),
    /// `described_by`, `controls` or `labelled_by`: the relation and its target.
    Relation(Relation, NodeRef),
    /// `.orientation(..)`: its source, which `scroll` compares with its axes.
    Orientation(Prop<Orientation>),
    /// `semantics_text`: the node's semantics are published after its draw layers.
    Text,
}
```
（`NodeRef` は Copy: builders.rs:108 が `patch.relation` を値で match している。`use super::node_ref::NodeRef;` と `Orientation` の import は既存。）

##### 書き手の変更
- modifiers.rs:52-62（`Common::semantic` の literal）: `text: false,`・`relation: None,`・`orientation: None,` の 3 行を削除。
- modifiers.rs:79-82（`semantic_relation`）: `.relation = Some((relation, target));` → `.input = SemanticInput::Relation(relation, target);`
- modifiers.rs:89-94（`semantic_orientation`）: `.orientation = Some(source);` → `.input = SemanticInput::Orientation(source);`（`self.semantic(source.clone(), ..)` はそのまま）
- node_modifiers.rs:778-783（`semantics_text`）: `.text = true;` → `.input = crate::view::semantics_data::SemanticInput::Text;`
- 行 86（part）と行 87（reactive）の marker は G4.out.tsv の新文言に置換（上の struct に反映済み）。

##### 読み手の変更
- builders.rs:105-113（labelled_by_sources）:
  ```rust
  .filter_map(|patch| match &patch.input {
      crate::view::semantics_data::SemanticInput::Relation(
          crate::view::semantics_data::Relation::LabelledBy,
          target,
      ) => Some(*target),
      _ => None,
  })
  ```
- builders.rs:115-123（described_by_sources）: 同形で `Relation::DescribedBy`。
- container.rs:131-136:
  ```rust
  .find_map(|patch| match &patch.input {
      crate::view::semantics_data::SemanticInput::Orientation(source) => Some(source.clone()),
      _ => None,
  });
  ```
- resolve.rs:267: `semantics.iter().any(|p| p.text)` → `semantics.iter().any(|p| matches!(p.input, crate::view::semantics_data::SemanticInput::Text))`
- mount/semantics.rs:66: `.find(|p| p.text)` → `.find(|p| matches!(p.input, SemanticInput::Text))`（このファイルは既に `SemanticInput` を import: semantics.rs:21 で使用）
- 既存の `SemanticInput` の match は全て `_` 腕があるので変更不要。

##### 振る舞い
不変。tag の読み手はそれぞれ自分の variant だけを見ており、旧来も 1 patch に 1 tag しか付かなかった。`semantic_inputs()`（builders.rs:46-70）の `apply` ポインタによる重複除去も影響なし。公開 API 影響なし（全て pub(crate)）。

##### E2E
- 既存: interaction_tooltip.rs:115 `the_overlay_is_a_named_tooltip_and_the_trigger_is_described_by_the_label`（described_by）、layout_scroll.rs:255 `orientation_on_a_scroll_container_is_reported_and_the_axes_win`（orientation）、apg_slider.rs:8 `slider_vertical_thumbs_expose_orientation`。
- 追加推奨: `Node::semantics_text` を直接使う E2E が dev/tests に見当たらない（grep で 0 件）。`semantics_text` に範囲外の selection を与えると `semantics_text is not published` が呼出し位置で `Unsupported` 報告され text が公開されないこと（mount/semantics.rs:63-75 の `Text` 判定を担保）。labelled_by の E2E も存在確認できず、必要なら追加。

##### 所有ファイル
- crates/kivact-core/src/view/semantics_data.rs
- crates/kivact-core/src/view/modifiers.rs
- crates/kivact-core/src/view/node_modifiers.rs
- crates/kivact-core/src/view/elements/builders.rs
- crates/kivact-core/src/view/elements/container.rs
- crates/kivact-core/src/view/mount/resolve.rs
- crates/kivact-core/src/view/mount/semantics.rs


## 付録 A-G5：G5（kivact-ui-soft） の WRONG 詳細

対応：W1 = P8-W08, W2 = P8-W09, W3 = （W3 は本文 3.4 の P8-O3 で上書き：実施しない）。本文 3.3・3.4 の上書きがある場合は本文が正。


対象 3 件（入力 5 行）。すべて `crates/kivact-ui-soft` 内で閉じる。振る舞いは 3 件とも変えない。

| ID | 行 | 閉じ方 |
|---|---|---|
| W1 | layout.rs:21 `LayoutInset.own_outer_x`／layout.rs:24 `LayoutInset.own_outer_y` | 常に同値の 2 つの Option を `own_outer: Option<f64>` 1 個に統合（SM-3） |
| W2 | layout.rs:173 `LayoutSlots.root_ref`／layout.rs:183 `LayoutSlots.content_width` | `root_ref` を削除し `content_width: Option<ContentWidth { root, width }>` に統合（型で表す） |
| W3 | timestamp.rs:171 `TimestampEntry.copyable` | 読み手の無い field を削除 |

W1 と W2 は同じファイル layout.rs を変える。1 人の実装者が続けて適用すること（下の行番号は HEAD 73c36c7c のもの）。

---

#### W1：`LayoutInset.own_outer_x`／`own_outer_y` → `own_outer`

##### 事実
- 書き手は repo 全体で次の 2 行だけ（Grep `own_outer`）:
  - layout.rs:210 `insets.own_outer_x = padding.map(Space::px);`
  - layout.rs:211 `insets.own_outer_y = insets.own_outer_x;`
- `from_padding`（layout.rs:30-40）と `Default` はどちらも `None` にする。app_shell.rs:605、card.rs:109、section.rs:145、mobile_nav.rs:520 は `from_padding`／`default` だけを使う。
- 読み手は layout.rs:451-452 の `region_insets` だけで、`inset.own_outer_x.or(inset.outer_x)` と `inset.own_outer_y.or(inset.outer_y)` の形。
- このため 2 つは常に同値になる（両方 `None`、または両方 `Some(p)` で同じ `p`）。`(Some(a), None)`、`(None, Some(b))`、`(Some(a), Some(b))`（a≠b）は到達しない。今の marker の independent（全組合せが有効）は誤り。

##### 新しいコード（layout.rs 先頭の `LayoutInset`）
```rust
/// Insets inherited by Layout regions from a parent surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutInset {
    // xtask-allow(state-fields): option: inline outer padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback (layout.rs:451).
    /// Inherited inline outer padding in logical pixels; None uses the region fallback.
    pub outer_x: Option<f64>,
    // xtask-allow(state-fields): option: block outer padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback (layout.rs:452).
    /// Inherited block outer padding in logical pixels; None uses the region fallback.
    pub outer_y: Option<f64>,
    // xtask-allow(state-fields): option: inline gap padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback (layout.rs:455).
    /// Padding between inline regions in logical pixels; None uses the region fallback.
    pub inner_x: Option<f64>,
    // xtask-allow(state-fields): option: block gap padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback (layout.rs:456).
    /// Padding between block regions in logical pixels; None uses the region fallback.
    pub inner_y: Option<f64>,
    // xtask-allow(state-fields): option: this Layout's padding from Layout::padding, written only in Layout::into_view (layout.rs:210); absent uses the inherited outer_x and outer_y.
    /// This layout’s outer padding override on both axes in logical pixels.
    pub own_outer: Option<f64>,
}
```
（`outer_x`〜`inner_y` の marker は G5.out.tsv の KIND 行と同じ文。marker の行番号の注記は実装後の行に合わせて直してよい。）

##### 書き手・読み手の変更（全部）
| 場所 | 現在 | 変更後 |
|---|---|---|
| layout.rs:21-26 | `own_outer_x`／`own_outer_y` の marker・doc・field（6 行） | 上の `own_outer` の 3 行（marker・doc・field）に置き換える |
| layout.rs:210-211 | `insets.own_outer_x = padding.map(Space::px);` と `insets.own_outer_y = insets.own_outer_x;` | `insets.own_outer = padding.map(Space::px);` の 1 行 |
| layout.rs:451 | `inset.own_outer_x.or(inset.outer_x).unwrap_or(fallback)` | `inset.own_outer.or(inset.outer_x).unwrap_or(fallback)` |
| layout.rs:452 | `inset.own_outer_y.or(inset.outer_y).unwrap_or(fallback)` | `inset.own_outer.or(inset.outer_y).unwrap_or(fallback)` |

`from_padding`（`..Self::default()`）は変えない。

##### 公開 API
- `kivact_ui_soft::LayoutInset`（lib.rs:158 で再公開）の pub field `own_outer_x`／`own_outer_y` を消し、pub field `own_outer: Option<f64>` を足す。互換 alias は作らない。
- repo 内（crates、examples、tests、docs）にこの 2 つの field を使う箇所は無い（Grep で確認済み）。
- 理由：Layout の padding（`Layout::padding(Space)`）は両軸で 1 つの値。theme 作者が読むなら値は 1 つの方が誤解が無い。

##### 不採用にした案
`own_outer` を `LayoutInset` から外して `LayoutSlots` に移す案は採らない。Layout の外で region（`layout_content` 等）を Card の中などに直接置くと、region は外側の Layout の `LayoutSlots` を読む一方で、`LayoutInset` は Card の `from_padding` を読む。このため外側の Layout の padding が漏れ、振る舞いが変わる。

##### 振る舞い
変わらない。唯一の書き手が 2 つに同じ値を書いていたため、読み手が受け取る値は同一。

##### E2E
- 既存：crates/kivact-ui-soft/tests/consumer/navigation.rs:91 `layout_components_mount_and_interact`（Card の中の Layout）。padding を指定していないため、`own_outer` が `None` の経路だけを担保する。
- 追加すべき：`soft::layout().padding(soft::Space::S6)` に header・content・start を置き、header の左右と上、および content の上（divider なしの場合）の inset が `Space::S6.px()` になることを rect で確かめる。同じ Layout を `soft::card` の中に置き、Card の padding ではなく Layout の padding が勝つことも確かめる。

---

#### W2：`LayoutSlots.root_ref` を `content_width` に統合

##### 事実
- `LayoutSlots`（layout.rs:171-185）の書き手は layout.rs:218-225 の 1 箇所だけ。ここで `root_ref` は常に `Some(root_ref)` になる。`None` になるのは、region が Layout の外にあり `try_use_context::<LayoutSlots>()` が無くて `unwrap_or_default()` した場合（layout.rs:374-376）だけで、その時は他の field もすべて既定値になる。
- `root_ref` の読み手は region_content_style（layout.rs:509-510）だけで、`(slots.root_ref, slots.content_width)` が両方 `Some`、かつ幅が `Length::Px` の時だけ使う。layout_row_style は `slots.root_ref` ではなく引数の `root_ref` を受け取る（layout.rs:253, :404, :417）。
- つまり `root_ref` は「content_width を中央寄せするための付属品」で、意味を持つのは `content_width` が `Some` の時だけ（SM-2：Option が別の Option の Some の時だけ意味を持つ）。marker の `option:`（caller 設定）も誤り。`root_ref` は Layout が内部で作る `node_ref()`（layout.rs:217）で、caller が与えるものではない。

##### 新しい型（layout.rs:171 付近、`LayoutSlots` の直前に追加）
```rust
/// A caller width limit with the layout root used to centre pixel widths.
#[derive(Clone, Copy)]
struct ContentWidth {
    root: NodeRef,
    width: Value<Length>,
}

#[derive(Clone, Copy, Default)]
struct LayoutSlots {
    // xtask-allow(state-fields): option: header divider resolved at into_view from LayoutRegion::divider or Layout::default_dividers (layout.rs:214); None means the layout has no header.
    header_divider: Option<bool>,
    // xtask-allow(state-fields): option: footer divider resolved at into_view from LayoutRegion::divider or Layout::default_dividers (layout.rs:214); None means the layout has no footer.
    footer_divider: Option<bool>,
    // xtask-allow(state-fields): option: whether the caller supplied Layout::start, captured once in into_view (layout.rs:222); false means no leading panel.
    has_start: bool,
    // xtask-allow(state-fields): option: whether the caller supplied Layout::end, captured once in into_view (layout.rs:223); false means no trailing panel.
    has_end: bool,
    // xtask-allow(state-fields): option: caller limit from Layout::content_width paired with the layout root ref used to centre pixel widths; absent means no width limit.
    content_width: Option<ContentWidth>,
}
```
`NodeRef` と `Value<Length>` はどちらも `Copy`（kivact-core/src/view/node_ref.rs:25、および既存の `LayoutSlots: Copy`）。`ContentWidth` は `Default` を持たないが、`Option` に包むので `LayoutSlots` の `#[derive(Default)]` はそのまま使える。

##### 書き手・読み手の変更（全部）
1. **layout.rs:217-225（書き手）**
   ```rust
   let root_ref = node_ref();
   let slots = LayoutSlots {
       header_divider: header.as_ref().and_then(|region| region.divider),
       footer_divider: footer.as_ref().and_then(|region| region.divider),
       has_start: start.is_some(),
       has_end: end.is_some(),
       content_width: content_width.map(|width| ContentWidth { root: root_ref, width }),
   };
   ```
   `root_ref` は layout.rs:244 の `root.node_ref(root_ref)` で引き続き使う。
2. **layout.rs:253**：`node.base_style(move || layout_row_style(slots, root_ref))` を `node.base_style(move || layout_row_style(slots))` にする。
3. **layout.rs:404-433 `layout_row_style`**：引数 `root_ref: NodeRef` を消し、match を次の形にする。
   ```rust
   fn layout_row_style(slots: LayoutSlots) -> kivact::style::Style {
       // direction と base は現行のまま
       if let Some(ContentWidth { root, width }) = slots.content_width {
           match width.get() {
               Length::Px(width) if !(slots.has_start && slots.has_end) => {
                   let gutter = center_gutter(root, width);
                   // 以下、現行 :418-429 の has_start / has_end の分岐をそのまま使う
               }
               width => {
                   base = base
                       .width(pct(100))
                       .max_width(width)
                       .margin_x(kivact::style::auto());
               }
           }
       }
       base
   }
   ```
4. **layout.rs:505-511 `region_content_style`**：
   ```rust
   if kind == Region::Content
       && !slots.has_start
       && !slots.has_end
       && let Some(ContentWidth { root, width }) = slots.content_width
       && let Length::Px(width) = width.get()
   {
       let gutter = center_gutter(root, width);
       // 現行 :513-516 のまま
   }
   ```
5. **layout.rs:518-525**：`&& let Some(width) = slots.content_width` を `&& let Some(ContentWidth { width, .. }) = slots.content_width` にする（`.max_width(width.get())` はそのまま）。
6. `root_ref` の marker 行（layout.rs:173）と field（:174）を削除する。

##### 公開 API
変わらない（`LayoutSlots` は private）。

##### 振る舞い
変わらない。`root_ref` は Layout の中では常に `Some` で、使われるのは `content_width` が `Some` の時だけだった。Layout の外（Default）では、今も変更後も `content_width` が `None` なので中央寄せは起きない。

##### E2E
- 既存：navigation.rs:91（content_width なし）。
- 追加すべき：`soft::layout().content_width(Length::Px(400.0)).content(...)` を幅 800 の window に置き、側 panel が無い時は content region の左右 padding に gutter（(root 幅 − 400)/2）が入ること、`start` がある時は row の leading 側に gutter が入ることを rect で確かめる。`Length::Percent` 等の時は max_width で中央寄せされることも確かめる。

---

#### W3：`TimestampEntry.copyable` の削除

**（本文 3.4 P8-O3 により実施しない。V2 では field を残し、marker を付録 B の option の文にする。以下は判断材料として残す。）**

##### 事実
- 定義は timestamp.rs:171-173（marker、doc「Whether the detail value offers a copy action.」、`pub copyable: bool`）。
- repo 全体で読み手も crate 内の書き手も無い（Grep `copyable`：この定義と、無関係な kivact-runtime/src/machine.rs:115 の doc だけ）。details の描画 `timestamp_details`（timestamp.rs:605-646）は zone・format・label だけを使い、copy 操作も clipboard 呼出しも無い。
- d4d65c10（field を足した commit）から一度も読まれていない。marker の「Copy permission is a presentation capability」は実装されていない機能を主張しており、事実と違う。lint の件数を減らすためだけの marker にあたる。

##### 変更
- timestamp.rs:171-173 の 3 行（marker、doc、field）を削除する。`TimestampEntry` の `#[derive(Default)]` は残す。
- 残る `zone`・`label` は 2 個の数える field のままで、既存の marker（OK 判定）をそのまま使う。

##### 公開 API
`kivact_ui_soft::TimestampEntry`（lib.rs:231）の pub field `copyable` を削除する。repo 内に利用箇所は無い。互換 alias は作らない。copy 操作が仕様として必要なら、それは別の機能追加として扱う（details の各行に copy ボタンを付け、その有無を入力として持つ）。値だけ先に持っておくことはしない。

##### 振る舞い
変わらない（値が読まれていなかったため）。

##### E2E
不要（読み手が無い）。`cargo check -p kivact-ui-soft --all-targets` と examples のビルドが通ることで足りる。

---

#### 所有ファイル一覧
- crates/kivact-ui-soft/src/layout.rs（W1、W2。G5 の KIND 行 layout.rs:9/12/15/18/175/177/179/181 の marker 書換えも同じファイル）
- crates/kivact-ui-soft/src/timestamp.rs（W3）
- 追加 E2E：crates/kivact-ui-soft/tests/consumer/navigation.rs（または repo の E2E 規約にある実 window の test）


## 付録 A-G6：G6（arvane 系・gpu-share・types・shader） の WRONG 詳細

対応：W1 = P8-W10, W2 = P8-W11, W3 = P8-W12, W4 = P8-W13, W5 = P8-W14, W6 = P8-W15, W7 = P8-W16, W8 = P8-W17。本文 3.3・3.4 の上書きがある場合は本文が正。


対象 HEAD 73c36c7c。行番号はすべてこの HEAD のもの。8 件（W1〜W8）、行数では 18 行。

| ID | 対象 | 閉じ方 | 公開 API |
|---|---|---|---|
| W1 | arvane-native `module.rs` MAPPING / REPORTED | type（一つの OnceLock の値にまとめる） | 変更なし |
| W2 | arvane-native `win32/state.rs` Utf16Decoder.pending / tail | derive（tail を戻り値にする） | 変更なし |
| W3 | arvane-render `draw/canvas.rs` GlyphRun.hint / cached | type（enum GlyphRaster） | 変更なし（pub(crate)） |
| W4 | arvane-shell `a11y/shared.rs` PlatformUpdate.focus / all_runs | derive + type（enum RunSelection） | 変更なし（field は private、accessor 維持） |
| W5 | arvane-shell `ime.rs` ShellImeText.password | derive（field 削除） | **変更あり**（pub field 削除） |
| W6 | kivact-gpu-share `common.rs` Identity、kivact-types `gpu.rs` GpuAdapter の luid / device_uuid / driver_uuid | type（enum AdapterIds） | **変更あり** |
| W7 | kivact-gpu-share `pool.rs` Slots.next | derive | 変更なし（pub(crate)） |
| W8 | kivact-shader `entry.rs` Interface | type（enum） | 変更なし（pub(crate)） |

---

#### W1 `crates/arvane-native/src/module.rs`：MAPPING と REPORTED（行 21・22）

##### 事実
- `MAPPING`（:15-21）は `retain_module` の `MAPPING.get_or_init(retain)`（:32）だけが書く。
- `REPORTED`（:22-24）は同じ関数の :35 `REPORTED.swap(true, AcqRel)` だけが書き、true のとき return して log（:38-45）を飛ばす。
- 書く順は常に MAPPING → REPORTED。到達する組は (未初期化, false)、(初期化済, false)、(初期化済, true)。(未初期化, true) は到達しない。REPORTED は MAPPING が決まった後にだけ意味を持つ（SM-2 の含意）。
- REPORTED は計数でなく一度きりの latch で、log を出すかどうかを分岐させる。`statistic:` は誤り。

##### 新しい型（:13-24 を置き換える）
```rust
#[cfg(any(windows, target_os = "linux"))]
use std::sync::atomic::{AtomicBool, Ordering};

/// What retaining the module gave, and whether it was reported.
#[cfg(any(windows, target_os = "linux"))]
struct Retained {
    // xtask-allow(state-fields): cache: the loader's answer for keeping this module mapped (`retain`); asking again gives the same answer, so it is asked once per process and never invalidated
    mapping: Option<&'static str>,
    // xtask-allow(state-fields): independent: set by the first `retain_module` call before it logs, whatever the loader answered; a reentrant call from the log subscriber sees it set and returns, so both answers occur with it unset and set
    reported: AtomicBool,
}

#[cfg(any(windows, target_os = "linux"))]
#[expect(
    clippy::disallowed_types,
    reason = "process-wide module mapping initialized once"
)]
static RETAINED: std::sync::OnceLock<Retained> = std::sync::OnceLock::new();
```

##### `retain_module`（:29-46）の書換え
```rust
pub fn retain_module() {
    #[cfg(any(windows, target_os = "linux"))]
    {
        let retained = RETAINED.get_or_init(|| Retained {
            mapping: retain(),
            reported: AtomicBool::new(false),
        });
        // Publish the report flag before entering subscriber code: a reentrant
        // call observes the completed mapping and must not log recursively.
        if retained.reported.swap(true, Ordering::AcqRel) {
            return;
        }
        let caller = std::panic::Location::caller();
        if let Some(mapping) = retained.mapping {
            /* :39-41 と同じ warn! */
        } else {
            /* :43-44 と同じ error! */
        }
    }
}
```
- module doc（:1-7）の「Reporting happens afterward, outside initialization」はそのまま成り立つ（log は get_or_init の外）。
- `static RETAINED` 自体を lint が単独で数える場合（`cargo xtask check --no-write --only state-fields --crate arvane-native` で確かめる）に限り、その直前に `// xtask-allow(state-fields): cache: the loader's answer and its report latch, made once per process by the first retain_module call` を置く。数えないなら marker は付けない。

##### 書き手・読み手
- 書き手・読み手は `module.rs` の中だけ（`retain_module` は session.rs:246, :298, win32/bridge.rs:11 から呼ばれるが中身に触れない）。

##### 振る舞い・E2E
- 振る舞いは不変（初回だけ retain、初回だけ log、再入時は log しない）。
- 既存 E2E：`dev/tests/kivact-public-api/tests/e2e/embed/windows.rs`・`embed/x11.rs`（embed session が retain_module を呼ぶ）。追加不要。

##### 所有ファイル
- `crates/arvane-native/src/module.rs`

---

#### W2 `crates/arvane-native/src/port/win32/state.rs`：Utf16Decoder.tail（行 31・32）

##### 事実
- `tail` は `push`（:65-80）の :78 `self.tail = chars.next()` だけが書く。読むのは `take_tail`（:81-83）で、呼ぶのは input.rs:152 だけ。input.rs:150-156 は `push` が Some を返した同じ WM_CHAR 処理の中で必ず `take_tail` を呼ぶ。したがって WM_CHAR と WM_CHAR の間で tail は常に None。tail は「第 2 unit を待つ文字」ではなく、`from_utf16_lossy(&[high, non-low])` が 2 文字（U+FFFD とその unit の文字）を返したときの 2 文字目を戻り値の代わりに退避しているだけ。marker の理由文は事実と違う。
- tail が Some になるのは pending を :72 で take した直後だけなので、pending と tail は排他（SM-4）。

##### 新しいコード（:57-84 を置き換える）
```rust
#[derive(Default)]
pub(super) struct Utf16Decoder {
    /// The unpaired high surrogate waiting for its pair.
    pending: Option<u16>,
}
impl Utf16Decoder {
    /// The text `unit` completes: `None` while a high surrogate waits for
    /// its pair; an unpaired surrogate decodes to U+FFFD.
    pub(super) fn push(&mut self, unit: u16) -> Option<String> {
        if (0xd800..=0xdbff).contains(&unit) {
            return self
                .pending
                .replace(unit)
                .map(|_| char::REPLACEMENT_CHARACTER.to_string());
        }
        Some(match self.pending.take() {
            Some(previous) => String::from_utf16_lossy(&[previous, unit]),
            None => String::from_utf16_lossy(&[unit]),
        })
    }
}
```
- `take_tail` を削除。数える field は pending だけになるので、pending の marker（:59）も削除する。

##### 読み手の書換え
- `crates/arvane-native/src/port/win32/input.rs:148-157`：
```rust
            let text = state.decoder.borrow_mut().push(unit);
```
  （以降 :158 の `if let Some(text) = text && !text.chars().all(char::is_control)` はそのまま）

##### 振る舞い・E2E
- 振る舞いは不変（同じ文字列を同じ時に commit する）。
- 既存 E2E：`dev/tests/kivact-public-api/tests/e2e/embed/windows.rs`（test_parent.rs:366 が WM_CHAR を unit ごとに post して文字を打つ試験）。
- 追加を推奨する E2E（Windows の embed）：surrogate pair（例 U+1F600 の 2 unit）を post すると 1 文字として commit される／孤立した high surrogate の後に 'a' を post すると "\u{FFFD}a" が commit される。

##### 所有ファイル
- `crates/arvane-native/src/port/win32/state.rs`、`crates/arvane-native/src/port/win32/input.rs`

---

#### W3 `crates/arvane-render/src/draw/canvas.rs`：GlyphRun.hint / cached（行 41・42）

##### 事実
- 書き手は painter.rs の 4 箇所だけ：
  - :698-706 stroke：`hint: false, cached: false`
  - :730-737 identity 変換：`hint: true, cached: solid`
  - :747-754 uniform scale：`hint: true, cached: solid`
  - :766-773 回転・skew・伸縮：`hint: false, cached: false`
- 到達する組は (true, true)、(true, false)、(false, false)。(false, true) は到達しない：cached ⇒ hint の含意（SM-2）。cached の doc（:81-82）自身が hint と同じ条件を述べている。
- 読み手：hint は canvas.rs:103（`apply` の `.hint(self.hint)`）、cached は gpu/encode.rs:301（`.atlas_cache(run.cached)`）。cpu.rs:247-255 は `apply` だけを使う。

##### 新しい型（canvas.rs:67-88 の struct を書換え）
```rust
/// How a run's glyphs are rasterized.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphRaster {
    /// Unhinted outlines where the shaping put them: stroked runs, and
    /// runs under a rotation, a skew or a non-uniform scale.
    Outline,
    /// Hinted vertically, under a translation and a uniform scale;
    /// `atlas`: the glyphs may come from a glyph atlas (a solid fill).
    Hinted { atlas: bool },
}

pub(crate) struct GlyphRun<'a> {
    pub(crate) font: &'a FontData,
    pub(crate) font_size: f64,
    pub(crate) coords: &'a [i16],
    pub(crate) embolden: glifo::FontEmbolden,
    /// Hinting, and whether the glyph atlas may be used.
    pub(crate) raster: GlyphRaster,
}

impl GlyphRun<'_> {
    /// Whether the glyphs may come from a glyph atlas.
    #[cfg_attr(
        not(feature = "gpu"),
        expect(dead_code, reason = "the GPU's glyph atlas")
    )]
    pub(crate) fn atlas(&self) -> bool {
        matches!(self.raster, GlyphRaster::Hinted { atlas: true })
    }
    // apply(): `.hint(self.hint)` → `.hint(matches!(self.raster, GlyphRaster::Hinted { .. }))`
}
```
- 二つの marker（:77, :80）は削除（数える field が無くなる）。`GlyphRaster` は `draw.rs:38` の `pub(crate) use canvas::{Canvas, GlyphRun}` に `GlyphRaster` を加えて painter から使う。

##### 書き手・読み手の書換え
- painter.rs:704-705 → `raster: GlyphRaster::Outline,`
- painter.rs:734-735 → `raster: GlyphRaster::Hinted { atlas: solid },`
- painter.rs:751-752 → `raster: GlyphRaster::Hinted { atlas: solid },`
- painter.rs:770-771 → `raster: GlyphRaster::Outline,`
- canvas.rs:103 → `.hint(matches!(self.raster, GlyphRaster::Hinted { .. }))`
- gpu/encode.rs:301 → `.atlas_cache(run.atlas())`

##### 振る舞い・E2E
- 振る舞いは不変（同じ hint・atlas の値を渡す）。
- 既存 E2E：`crates/arvane/tests/e2e/present.rs` の `a_glyph_run_inks_inside_its_shaped_box_on_its_baseline`（:88）と `the_gpu_frame_matches_the_cpu_frame_within_the_tolerance`（:186）、`crates/arvane/tests/e2e/transform.rs` の `a_transform_rotates_and_skews_what_it_encloses_until_it_is_popped`（:19、Outline の経路）。追加不要。

##### 所有ファイル
- `crates/arvane-render/src/draw/canvas.rs`、`crates/arvane-render/src/draw/painter.rs`、`crates/arvane-render/src/draw.rs`、`crates/arvane-render/src/gpu/encode.rs`

---

#### W4 `crates/arvane-shell/src/a11y/shared.rs`：PlatformUpdate.focus / all_runs（行 45・46）

##### 事実
- 構築は 2 箇所だけ：
  - :258-265（activate）：`focus: tree.focus()`, `runs: &[]`, `all_runs: true`
  - :305-339（pending_with）：`focus: tree.focus()`；`Unsent::Tree` → (`&[]`, true)、`Unsent::Nothing` → (`&[]`, false)、`Unsent::Changed` → merged なら (`&[]`, true)、そうでなければ (`&runs[..]`, false)
- focus は常に同じ struct の `tree`（`AcceptedTree::focus`, state.rs:63-66）の複写：導出できる値を保持している。
- all_runs = true のとき runs は常に空。runs は all_runs = false のときだけ意味を持つ（SM-2 の含意）。読み手 native.rs:216-221 もその前提で `if !selection.all_runs() { … changed_runs() … }` と書いている。

##### 新しい型（:153-162 を書換え）
```rust
/// Which text runs an update carries.
#[derive(Clone, Copy)]
enum RunSelection<'a> {
    /// Every text run of each selected node.
    All,
    /// Only the runs that changed (possibly none).
    Changed(&'a [Arc<SemanticsTextRun>]),
}

pub struct PlatformUpdate<'a> {
    tree: &'a state::AcceptedTree,
    nodes: Vec<&'a kivact_types::render::SemanticsNode>,
    runs: RunSelection<'a>,
    scale: f64,
}
```
（a11y/runs.rs に既に `Runs` 型があるので名前は `RunSelection` にする。）二つの marker（:155, :159）は削除。

##### accessor（:170-188）の書換え（公開の signature は不変）
```rust
pub fn focus(&self) -> Option<SemanticsId> {
    self.tree.focus()
}
pub fn all_runs(&self) -> bool {
    matches!(self.runs, RunSelection::All)
}
pub fn changed_runs(&self) -> impl Iterator<Item = &SemanticsTextRun> {
    let runs: &[Arc<SemanticsTextRun>] = match self.runs {
        RunSelection::All => &[],
        RunSelection::Changed(runs) => runs,
    };
    runs.iter().map(AsRef::as_ref)
}
```

##### 書き手の書換え
- :258-265 → `PlatformUpdate { tree, nodes: tree.nodes(), runs: RunSelection::All, scale: shared.scale }`
- :305-327 → タプルを `(nodes, runs)` にし、`Unsent::Tree => (tree.nodes(), RunSelection::All)`、`Unsent::Nothing => (Vec::new(), RunSelection::Changed(&[]))`、`Unsent::Changed { .. }` は `if *merged == Merge::Merged { RunSelection::All } else { RunSelection::Changed(&runs[..]) }`
- :333-339 → `PlatformUpdate { tree, nodes, runs, scale: shared.scale }`

##### 読み手
- `crates/arvane-native/src/a11y/native.rs:210-224` は accessor だけを使うので変更なし。

##### 公開 API・振る舞い・E2E
- 公開 API 不変（field は private、`focus()`・`all_runs()`・`changed_runs()` の signature は同じ）。振る舞い不変。
- 既存 E2E：`crates/arvane/tests/e2e/accessibility.rs` の `a_replaced_tree_takes_updates_of_the_nodes_that_changed`（:73）、`dev/tests/kivact-public-api/tests/e2e/embed/uia.rs`。追加不要。

##### 所有ファイル
- `crates/arvane-shell/src/a11y/shared.rs`

---

#### W5 `crates/arvane-shell/src/ime.rs`：ShellImeText.password（行 49。行 50 composition は W5 後に marker 削除）

##### 事実
- 書き手は kivact-engine の 4 箇所だけ：
  - `crates/kivact-engine/src/model/interact/text_geometry.rs:246-257`：`password: true`、text 空、selection (0,0)、caret 0、`composition: None`、rects は caret の 1 個
  - 同 :264-274：`password: false`
  - `crates/kivact-engine/src/model/edit/session.rs:138-143`：`password: true, rects, area, ..default()`（composition None）
  - 同 :147-148：`password: false`
- password = true ⇒ composition = None（かつ text 空）。marker の「a password field can still compose」は事実と逆。
- **読み手が無い**：arvane-native・arvane-virtual・kivact-test・dev/tests のどこも `ShellImeText::password` を読まない（Grep `\.password\b` で確認。kivact-test の `is_password` は a11y 側、results.rs:171-181 は text/rects_from/composition だけを写す）。password であることは別の経路 `ShellWindow::set_ime_purpose(InputPurpose::Password)`（winit/window.rs:351-353）で backend に渡っている。保護は「text を空にした snapshot」で既に実現している。
- したがって password は他から分かる（purpose）重複で、しかも誰も読まない。導出（削除）で閉じる。

##### 変更
- `crates/arvane-shell/src/ime.rs:18-20` の marker・doc・`pub password: bool` を削除。struct の doc（:5-15）に 1 文足す：「A password input gives an empty excerpt (no text, no selection, no composition) and only the caret's box; the window learns the purpose from `ShellWindow::set_ime_purpose`.」
- 数える field は composition だけになるので composition の marker（:31）も削除。
- 書き手：
  - text_geometry.rs:247 の `password: true,` と :265 の `password: false,` を削除。
  - session.rs:139 の `password: true,` と :148 の `password: false,` を削除。

##### 公開 API への影響
- `arvane_shell::ShellImeText`（arvane が `arvane::ShellImeText` として再公開、kivact-engine testing が `kivact_engine::testing::ShellImeText` として再公開）から pub field `password` が消える。利用者は backend 実装者と内部の試験だけで、theme 作者は触れない。代替は `set_ime_purpose` の `InputPurpose::Password`（既存）。互換 alias は作らない。

##### 振る舞い・E2E
- 振る舞いは不変（読み手が無い）。
- 既存 E2E：`dev/tests/kivact-public-api/tests/e2e/input_platform.rs` の `password_updates_and_ime_keep_published_text_protected`（:104、password の IME text が空で composition None であることを確かめる）、`text_editing.rs` の `ime_composition_commits_cancels_swallows_keys_and_ends_on_blur`（:218）、`input_platform.rs` の `raw_ime_composition_commits`（:315）。追加不要。

##### 所有ファイル
- `crates/arvane-shell/src/ime.rs`、`crates/kivact-engine/src/model/interact/text_geometry.rs`、`crates/kivact-engine/src/model/edit/session.rs`

---

#### W6 adapter の識別子：`kivact-gpu-share::Identity`（行 130-132）と `kivact_types::GpuAdapter`（行 139-141。行 142 generation は W6 後に marker 削除）

##### 事実
- Identity の書き手は 2 箇所だけ：
  - `crates/kivact-gpu-share/src/import/dx12.rs:53-58`：`backend: Dx12, luid: Some(luid), device_uuid: None, driver_uuid: None`
  - `crates/kivact-gpu-share/src/vulkan_raw.rs:112-117`：`backend: Vulkan, luid: (valid).then_some(..), device_uuid: Some(..), driver_uuid: Some(..)`
- GpuAdapter の書き手：`crates/arvane-compose/src/gpu/context.rs:209-216`（Identity を写す）、試験用 `dev/tests/gpu-interop-producer/src/d3d12.rs:170-177`（Dx12 と同じ組）、`dev/tests/gpu-interop-producer/src/vulkan.rs:201-210`（Vulkan と同じ組）。
- 到達する組：Dx12 ⇒ (luid Some, device None, driver None)、Vulkan ⇒ (luid 任意, device Some, driver Some)。device_uuid と driver_uuid は常に同時に Some/None（SM-3）で、どちらの組かは backend が決める（含意）。読み手もこれを前提にしている：`vulkan/physical_device.rs:76-79` は `expect("invariant: Vulkan reports a device UUID")`、:93-96 は (Some, Some) 以外を Mismatch、`d3d12/adapter.rs:18-20` は luid None を Mismatch。

##### 新しい型（kivact-types `crates/kivact-types/src/gpu.rs`、GpuAdapter の前に追加）
```rust
/// How an adapter's graphics API identifies it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AdapterIds {
    /// Direct3D 12: DXGI's adapter LUID ([`AdapterMatch::Luid`]'s bytes).
    Dx12 {
        /// The adapter LUID.
        luid: [u8; 8],
    },
    /// Vulkan: `VkPhysicalDeviceIDProperties`.
    Vulkan {
        /// `deviceUUID`.
        device_uuid: [u8; 16],
        /// `driverUUID`.
        driver_uuid: [u8; 16],
        /// `deviceLUID`, where the driver reports one (Windows).
        luid: Option<[u8; 8]>,
    },
}

impl AdapterIds {
    /// The graphics API.
    pub fn backend(&self) -> GpuBackend {
        match self {
            Self::Dx12 { .. } => GpuBackend::Dx12,
            Self::Vulkan { .. } => GpuBackend::Vulkan,
        }
    }
    /// The LUID, when the API reports one.
    pub fn luid(&self) -> Option<[u8; 8]> {
        match *self {
            Self::Dx12 { luid } => Some(luid),
            Self::Vulkan { luid, .. } => luid,
        }
    }
}
```
GpuAdapter（:84-107）を書換え：
```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GpuAdapter {
    /// How the graphics API identifies the adapter (and so which API it is).
    pub ids: AdapterIds,
    /// The adapter's name, for diagnostics.
    pub name: String,
    /// (doc は今の :101-103 のまま)
    pub generation: u64,
}

impl GpuAdapter {
    /// The graphics API of the device.
    pub fn backend(&self) -> GpuBackend {
        self.ids.backend()
    }
    /// Whether this is the adapter `identity` names.
    pub fn matches(&self, identity: &AdapterMatch) -> bool {
        match (identity, self.ids) {
            (AdapterMatch::Luid(luid), ids) => ids.luid().as_ref() == Some(luid),
            (
                AdapterMatch::Uuid { device, driver },
                AdapterIds::Vulkan { device_uuid, driver_uuid, .. },
            ) => device_uuid == *device && driver_uuid == *driver,
            (AdapterMatch::Uuid { .. }, AdapterIds::Dx12 { .. }) => false,
        }
    }
}
```
- marker：:93, :96, :99 を削除。数える field は generation だけになるので :102 の marker も削除。AdapterIds::Vulkan の `luid` は variant 内で唯一の数える field なので marker 不要。
- `AdapterIds` を再公開に加える：`crates/kivact-types/src/lib.rs:57` の `pub use gpu::{AdapterMatch, …, GpuAdapter, …}` の列、および `crates/kivact-core/src/gpu_interop.rs:103` 付近（`pub use kivact_types::AdapterMatch;` の隣。GpuAdapter を再公開している行があればその列）に `AdapterIds` を足す。
- 公開 API の選択理由：外部の engine 作者が `match adapter.ids { AdapterIds::Dx12 { luid } => …, AdapterIds::Vulkan { device_uuid, driver_uuid, .. } => … }` と書け、不可能な組（Dx12 に UUID、Vulkan に UUID 欠け）が型で表せなくなる。backend は variant から分かるので field ではなく accessor `backend()` にする（重複の保持をやめる）。

##### kivact-gpu-share
- `crates/kivact-gpu-share/src/common.rs:8-22` の `struct Identity` を削除し、`kivact_types::AdapterIds` を使う。`crates/kivact-gpu-share/src/lib.rs:186` `pub use common::{Error, Identity};` → `pub use common::Error;`（Identity は公開から消える。alias は作らない）。
- `device.rs:7, :21`：`pub fn identity(adapter: &::wgpu::Adapter) -> Option<AdapterIds>`。
- `import/dx12.rs:22, :47-59`：`Some(AdapterIds::Dx12 { luid })`。
- `vulkan_raw.rs:99-120`：戻り値を Vulkan の識別子を表す private な struct にする（Option を持たないので lint の対象外）：
  ```rust
  /// What `VkPhysicalDeviceIDProperties` and the API version say.
  pub(crate) struct VulkanIds {
      pub(crate) device_uuid: [u8; 16],
      pub(crate) driver_uuid: [u8; 16],
      pub(crate) luid: Option<[u8; 8]>,
      pub(crate) version: u32,
  }
  ```
  値は `VulkanIds { device_uuid: id.device_uuid, driver_uuid: id.driver_uuid, luid: (id.device_luid_valid == vk::TRUE).then_some(id.device_luid), version }`。
  （注：VulkanIds の数える field は luid の 1 個だけなので marker 不要。lint が数えるなら W6 の目的と同じく型の内側の唯一の Option であり marker は付けず報告する。）
- `import/vulkan.rs:32, :41-47`：戻り値型を `Option<AdapterIds>` にし、
  ```rust
  let ids = unsafe { crate::vulkan_raw::identity(instance, hal.raw_physical_device()) };
  Some(AdapterIds::Vulkan { device_uuid: ids.device_uuid, driver_uuid: ids.driver_uuid, luid: ids.luid })
  ```
- `vulkan/physical_device.rs:69-81`（identity）：`expect` 2 個を消し、
  ```rust
  let ids = unsafe { crate::vulkan_raw::identity(instance, physical) };
  (ids.device_uuid, ids.driver_uuid, ids.version)
  ```
- `vulkan/physical_device.rs:86-97`（uuids）：
  ```rust
  match adapter.ids {
      AdapterIds::Vulkan { device_uuid, driver_uuid, .. } => Ok((device_uuid, driver_uuid)),
      AdapterIds::Dx12 { .. } => Err(Error::Mismatch(format!(
          "the window's device is {:?}, not Vulkan", adapter.backend()))),
  }
  ```
- `d3d12/adapter.rs:11-21`（luid）：
  ```rust
  match adapter.ids {
      AdapterIds::Dx12 { luid } => Ok(luid),
      AdapterIds::Vulkan { .. } => Err(Error::Mismatch(format!(
          "the window's device is {:?}, not Direct3D 12", adapter.backend()))),
  }
  ```
  （「the adapter has no LUID」の分岐は到達不能になり消える）
- `wgpu/device.rs:28, :33`、`wgpu/pool.rs:226`：`adapter.backend` → `adapter.backend()`。
- `wgpu/d3d12.rs:35-36`、`wgpu/vulkan.rs:27-28`、`vulkan/pool.rs:199-202`、`d3d12/pool.rs:3` は luid()/uuids() 経由なので変更なし。

##### arvane-compose
- `crates/arvane-compose/src/gpu/context.rs:204-217` `gpu_adapter`：引数を `ids: kivact_types::AdapterIds` にし、`GpuAdapter { ids, name: adapter.get_info().name, generation }`。:187-190 の呼出しは `kivact_gpu_share::device::identity(adapter)` の戻り値をそのまま渡す。

##### 試験・docs
- `crates/arvane/tests/e2e/external.rs:49, :52`：`adapter.backend` → `adapter.backend()`。
- `dev/tests/kivact-public-api/tests/e2e/gpu_interop.rs:158, :161, :506, :510`、`drawable_export_gpu.rs:193`：同上。
- `dev/tests/gpu-interop-producer/src/producer.rs:105`、`wgpu_engine.rs:37`：同上。
- `dev/tests/gpu-interop-producer/src/d3d12.rs:170-177`：`GpuAdapter { ids: AdapterIds::Dx12 { luid }, name: …, generation: 1 }`。
- `dev/tests/gpu-interop-producer/src/vulkan.rs:201-210`：`GpuAdapter { ids: AdapterIds::Vulkan { device_uuid: id.device_uuid, driver_uuid: id.driver_uuid, luid: (id.device_luid_valid == vk::TRUE).then_some(id.device_luid) }, name: …, generation: 1 }`。
- `crates/kivact-core/src/gpu_interop/import/registry.rs:138` の doc link `[`GpuAdapter::backend`]` は method を指すようになるだけで変更不要（rustdoc で確かめる）。
- `docs/for-kivact-user/en/how-to-guide/gpu-texture-sharing.md:338-339`（jp も同じ箇所）：「A `GpuAdapter` names the graphics API, the adapter's LUID or Vulkan UUIDs」→「A `GpuAdapter`'s `ids` is an `AdapterIds`: Direct3D 12's LUID, or Vulkan's device and driver UUIDs (and the LUID on Windows); `backend()` names the graphics API」。jp は同じ内容を日本語で。

##### 振る舞い・E2E
- 振る舞いは不変（同じ adapter を選び、同じ Mismatch を返す。到達不能だった「no LUID」「no Vulkan UUIDs」の文言だけが消える）。
- 既存 E2E：`dev/tests/kivact-public-api/tests/e2e/gpu_interop.rs`、`viewport_gpu_interop.rs`、`drawable_export_gpu.rs`、`crates/arvane/tests/e2e/external.rs`。feature 組合せ `cargo check -p kivact --features gpu-interop` を必ず通す。
- 追加を推奨する試験：`GpuAdapter::matches` が `AdapterMatch::Uuid` を Dx12 の adapter に対して false を返すこと（kivact-types の doc test で足りる）。

##### 所有ファイル
- `crates/kivact-types/src/gpu.rs`、`crates/kivact-types/src/lib.rs`、`crates/kivact-core/src/gpu_interop.rs`
- `crates/kivact-gpu-share/src/{common.rs, lib.rs, device.rs, vulkan_raw.rs, import/dx12.rs, import/vulkan.rs, vulkan/physical_device.rs, d3d12/adapter.rs, wgpu/device.rs, wgpu/pool.rs}`
- `crates/arvane-compose/src/gpu/context.rs`
- `crates/arvane/tests/e2e/external.rs`、`dev/tests/kivact-public-api/tests/e2e/{gpu_interop.rs, drawable_export_gpu.rs}`、`dev/tests/gpu-interop-producer/src/{d3d12.rs, vulkan.rs, producer.rs, wgpu_engine.rs}`
- `docs/for-kivact-user/{en,jp}/how-to-guide/gpu-texture-sharing.md`

---

#### W7 `crates/kivact-gpu-share/src/pool.rs`：Slots.next（行 133。行 134 last は W7 後に marker 削除）

##### 事実
- `next` の書き手は `Slots::new`（:145-151、0）と `finish`（:180、`(next + 1) % len`）だけ。`last` の書き手も `new`（None）と `finish`（:178、`Some(next_value())`）だけ。値は FIRST_FRAME = 2 から 1 ずつ増える（:162-168）。
- finish の回数を n とすると last = None（n=0）または Some(n + 1)、next = n % len。よって `next == last.map_or(0, |l| l - 1) % len`：next は last から導出できる複写で、modulo 1 回を省く以外の性能上の理由も無い。

##### 新しいコード
- field `next`（:131-133）と marker を削除。数える field は last だけになるので last の marker（:134）も削除。
- 追加：
```rust
    /// The texture of the next frame: round robin over the frames finished so far.
    fn cursor(&self) -> usize {
        let finished = self
            .last
            .map_or(0, |last| last.get() - FIRST_FRAME.get() + 1);
        (finished % self.drawn.len() as u64) as usize
    }
```
- `new`（:148）：`next: 0,` を削除。
- `next()`（:153-158）：`let slot = self.cursor(); Acquired { slot: slot as u32, wait: self.drawn[slot].value() }`。
- `finish`（:172-187）：先頭で `let slot = self.cursor();`、assert は `acquired.slot as usize, slot`、`self.drawn[slot] = SlotState::Drawn(value);`、:180 の `self.next = …` を削除（last の更新で cursor が進む）。`self.last = Some(value)` は drawn の更新より後でも前でもよいが、slot は更新前に取ること。

##### 読み手
- `d3d12/pool.rs:284, :293, :320`、`vulkan/pool.rs:412, :509, :528` は `next()`・`finish()`・`next_value()` を呼ぶだけで変更なし。

##### 振る舞い・E2E
- 振る舞いは不変（同じ順で texture を巡る）。
- 既存 E2E：`dev/tests/kivact-public-api/tests/e2e/gpu_interop.rs`、`viewport_gpu_interop.rs`（pool の frame を複数回送る）。追加不要。

##### 所有ファイル
- `crates/kivact-gpu-share/src/pool.rs`

---

#### W8 `crates/kivact-shader/src/entry.rs`：Interface（行 135・136）

##### 事実
- 書き手は `read_interface`（:25-58）だけ。`takes_params` は `check_arguments`（:61-103）の戻り値で、第 3 引数があるのに `params_declared` が false なら :95-102 で Err を返す。よって takes_params = true ⇒ params_declared = true。(false, true) は到達しない（含意、SM-2）。marker の「every combination is valid」は誤り。
- 読み手：`assemble.rs:92`（`prefix(interface.params_declared)`）、`assemble.rs:95`（`entry_text(kind, interface.takes_params)`）。

##### 新しい型（:13-22 を置き換える）
```rust
/// What the first parse of a user source declares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Interface {
    /// No `Params` struct: the entry function takes two arguments.
    NoParams,
    /// The user declares a `Params` struct; `taken`: the entry function
    /// takes it as its third argument.
    Params { taken: bool },
}

impl Interface {
    /// Whether the user declares a `Params` struct.
    pub(crate) fn params_declared(self) -> bool {
        matches!(self, Self::Params { .. })
    }
    /// Whether the entry function takes `Params` as its third argument.
    pub(crate) fn takes_params(self) -> bool {
        matches!(self, Self::Params { taken: true })
    }
}
```
- 二つの marker を削除。

##### 書き手・読み手の書換え
- entry.rs:54-57 →
```rust
    Ok(if params_declared {
        Interface::Params { taken: takes_params }
    } else {
        Interface::NoParams
    })
```
  （params_declared が false のとき check_arguments は Ok(false) か Err なので、NoParams で takes_params が失われることはない）
- assemble.rs:92 → `prefix(interface.params_declared())`、:95 → `entry_text(kind, interface.takes_params())`。

##### 振る舞い・E2E
- 振る舞いは不変。
- 既存 E2E：`dev/tests/kivact-public-api/tests/e2e/shaders.rs`、kivact-shader の doc test（assemble.rs:80-84）。追加不要。

##### 所有ファイル
- `crates/kivact-shader/src/entry.rs`、`crates/kivact-shader/src/assemble.rs`


## 付録 A-G7：G7（layout・test・bench・xtask） の WRONG 詳細

対応：W1 = P8-W18, W2 = P8-W19, W3 = P8-W20, W4 = P8-W21。本文 3.3・3.4 の上書きがある場合は本文が正。


WRONG は 9 行・4 件（W1〜W4）。どれも公開 API に影響しない（`pub(crate)`・`pub(super)`・試験・dev ツールの中に閉じる）。振る舞いも変わらない。

---

#### W1 `kivact-layout` の `Counted.unfiltered` と `Counted.at`（SM-2）

対象の行：`crates/kivact-layout/src/columns.rs:60`（unfiltered）、`:63`（at）

##### 事実
- `Counted { epoch, count, unfiltered: bool, at: Option<(usize, usize)> }`（columns.rs:54-65）。
- `at` を `Some` にするのは `filtered_flow_child`（traversal.rs:165-190、:187 の `store`）だけ。`flow_child` は `counted.unfiltered` が true なら素の index を返し（traversal.rs:159-161）、`filtered_flow_child` を呼ばない。
- `at_epoch`（columns.rs:73-88）は数え直すたびに `at: None` で作る。viewport の cursor（traversal.rs:267-276）は `unfiltered = false` で作るが、viewport の位置は `state.fixed_cursor` が持つので `at` は書かれない。
- 組合せ：`(unfiltered=true, at=Some)` は到達しない。`at` は「一部の子が除かれている」時だけ意味を持つ。SM-2（bool と、その値の時だけ意味のある Option）の形。両方に `cache:` の marker を付けているが、cache の組が暗黙の状態（全部が流れの中 / 除かれた子があり、最後に引いた位置を持つ）を作っている。

##### 新しい型（columns.rs、`Counted` の直前に追加）
```rust
/// Which children of a counted node are in flow, as counted at its epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Filtering {
    /// Every child is in flow: a flow index is the raw index.
    Unfiltered,
    /// Some child is out of flow; `at` is the last (filtered, raw)
    /// position looked up (`None`: none yet at this epoch).
    Filtered { at: Option<(usize, usize)> },
}

/// A cursor as counted at one epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Counted {
    pub(crate) epoch: Epoch,
    /// The children in flow (the viewport's: its attached fixed boxes).
    pub(crate) count: usize,
    /// Whether some child is filtered out, and where the last lookup was.
    pub(crate) filtering: Filtering,
}

impl Counted {
    /// Whether no child is filtered out.
    #[inline(always)]
    pub(crate) fn unfiltered(self) -> bool {
        matches!(self.filtering, Filtering::Unfiltered)
    }
}
```
二つの `xtask-allow(state-fields)` marker（columns.rs:60, :63）は消す。`Filtering` は State/Phase 等の名前ではないので数えない。`Filtered { at }` は数える field が一つだけ。`epoch: Epoch` は helper で、同じ struct に Stored が無くなるので数えない。marker は要らない。

##### 書き手・読み手の変更
| file:line | 今 | 書換え |
|---|---|---|
| columns.rs:78-86 `ChildCursor::at_epoch` | `let (count, unfiltered) = count(); Counted { epoch, count, unfiltered, at: None }` | `let (count, unfiltered) = count(); let filtering = if unfiltered { Filtering::Unfiltered } else { Filtering::Filtered { at: None } }; Counted { epoch, count, filtering }` |
| traversal.rs:157-162 `flow_child` | `if counted.unfiltered { return …child_at(slot, index) } self.filtered_flow_child(slot, index, counted)` | `let Filtering::Filtered { at } = counted.filtering else { return taffy_id(self.tree.child_at(slot, index)); }; self.filtered_flow_child(slot, index, counted, at)` |
| traversal.rs:165-166 `filtered_flow_child` | 引数 `counted: Counted`、`let at = counted.at;` | 引数に `at: Option<(usize, usize)>` を足し、`let at = counted.at;` を消す |
| traversal.rs:187-190 | `Counted { at: Some((filtered, raw)), ..counted }` | `Counted { filtering: Filtering::Filtered { at: Some((filtered, raw)) }, ..counted }` |
| traversal.rs:258 `child_ids` | `self.flow_cursor(slot).unfiltered` | `self.flow_cursor(slot).unfiltered()` |
| traversal.rs:21 | `use crate::{ChildCursor, Counted, Flow, …}` | `Filtering` を足す |
| lib.rs:35 | `pub(crate) use columns::{ChildCursor, Counted, …}` | `Filtering` を足す |
| traversal.rs:12-14（モジュール doc） | 「whether it is unfiltered and the last position」 | 「whether some child is filtered out and, if so, the last position」 |

`count_flow`（traversal.rs:141-147）と viewport の closure（:267-275）は `(usize, bool)` を返すままでよい。`FlowChildren.unfiltered`（traversal.rs:36）は別の struct で、bool が一つだけなので変えない。

##### 振る舞い・試験
- 振る舞いは同じ（同じ分岐を enum の match で書く）。hot path（`#[inline(always)]`）の分岐の数も変わらない。
- 担保：`cargo test -p kivact-layout`（overlay・fixed で子を除く layout の unit test）と、overlay・fixed を含む既存の実窓 E2E。新しい E2E は要らない（内部の表現を変えるだけ）。

##### 所有ファイル
`crates/kivact-layout/src/columns.rs`、`crates/kivact-layout/src/traversal.rs`、`crates/kivact-layout/src/lib.rs`

---

#### W2 `kivact-test` の `Probe.armed` と `Probe.reader`（＋試験の local `reader`）（SM-2）

対象の行：`crates/kivact-test/src/screen/editing_drop_probe.rs:7`（armed）、`:13`（reader）、`crates/kivact-test/src/screen/host_sync_tests.rs:356`（local reader）

##### 事実
- `Probe { armed: Cell<bool>, drops, reentered, reader: Rc<RefCell<Option<(Reader, NodeId)>>> }`（editing_drop_probe.rs:6-15）。
- 試験は、全 probe が共有する slot `reader` を埋めてから（host_sync_tests.rs:396）、生きている probe の `armed` を立てる（:398-403）。
- `Drop`（editing_drop_probe.rs:16-27）は `armed` が false なら戻る。true なら `self.reader.borrow().as_ref().unwrap()` を読む。
- 組合せ：`armed=true ⇒ reader=Some`。armed=true で None なら panic する。marker の「armed は別の観測」「reader は一度設定され drop で読む」は、この含意を隠している。SM-2 の形。

##### 新しい型（editing_drop_probe.rs）
```rust
pub(super) struct Probe {
    // xtask-allow(state-fields): independent: the reader and the other live node the test arms this probe with; an unarmed probe skips the drop check, and the shared counters count drops whatever it holds
    pub(super) armed: RefCell<Option<(Reader, NodeId)>>,
    // xtask-allow(state-fields): statistic: a test counter of drops
    pub(super) drops: Rc<Cell<usize>>,
    // xtask-allow(state-fields): statistic: a test counter of re-entered drops
    pub(super) reentered: Rc<Cell<usize>>,
}
impl Drop for Probe {
    fn drop(&mut self) {
        let Some((reader, independent)) = self.armed.borrow_mut().take() else {
            return;
        };
        self.drops.set(self.drops.get() + 1);
        // This is a different still-live node: replacing the editor must
        // not invalidate the reference or hold the shared Model borrow.
        reader.with_node(independent, |node| assert!(node.is_some()));
        self.reentered.set(self.reentered.get() + 1);
    }
}
```
`reader` field と、その marker（editing_drop_probe.rs:13-14）は消す。import の `Cell`・`RefCell`・`Rc` は使い続ける。

##### 書き手・読み手の変更（host_sync_tests.rs、`replacing_editing_drops_line_state_after_releasing_the_model`）
| file:line | 今 | 書換え |
|---|---|---|
| :356-357 | marker と `let reader = Rc::new(RefCell::new(None));` | 消す |
| :359-364 | `let (d, e, read, seen) = (drops.clone(), reentered.clone(), reader.clone(), probes.clone());` | `let (d, e, seen) = (drops.clone(), reentered.clone(), probes.clone());` |
| :368-373 | `Probe { armed: Cell::new(false), drops: d.clone(), reentered: e.clone(), reader: read.clone() }` | `Probe { armed: RefCell::new(None), drops: d.clone(), reentered: e.clone() }` |
| :396 | `*reader.borrow_mut() = Some((writer.reader(), independent));` | 消す |
| :398-403 | `probe.armed.set(true);` | `probe.armed.replace(Some((writer.reader(), independent)));` |

`RefCell` の import（:349）は Probe の literal で使い続ける。`Reader` は `Clone` なので、probe ごとに `writer.reader()` を渡してよい。

##### 振る舞い・試験
- 試験の意味は同じ（arm した probe だけが drop の時に別 node を読み、counter を進める）。
- 担保：`cargo test -p kivact-test replacing_editing_drops_line_state_after_releasing_the_model`（:407-410 の assert：drops=1、reentered=1、同じ値の書込みでは drop しない）。

##### 所有ファイル
`crates/kivact-test/src/screen/editing_drop_probe.rs`、`crates/kivact-test/src/screen/host_sync_tests.rs`

---

#### W3 `dev/bench` kv_heap の static `PTRS` と `SITES`（SM-3）

対象の行：`dev/bench/src/bin/kv_heap.rs:35`（PTRS）、`:37`（SITES）

##### 事実
- `PTRS: Mutex<Option<HashMap<usize, (usize, u64)>>>`、`SITES: Mutex<Option<HashMap<u64, Site>>>`（kv_heap.rs:35-38）。
- `record`（:43-75）は同じ呼出しの中で SITES と PTRS を `get_or_insert_with` で `Some` にする。二つの `Option` は最初の record から同時に `Some` になる（SM-3）。PTRS の値 `(size, key)` は SITES の key を指す索引で、`forget`（:77-88）は二つを組で更新する。lock が分かれているので、二つの更新の間に別 thread から見える。
- marker の「allocator wrapper が唯一の書き手・読み手」は誤り。`dump`（:136-162）が SITES を読み、`main`（:168）と `dump`（:138）が TRACKING を書く。

##### 新しい型（kv_heap.rs:35-38 を置き換える）
```rust
/// What the probe recorded: the live allocations and their sites.
#[derive(Default)]
struct Heap {
    /// Each live allocation's size and site key, by address.
    ptrs: HashMap<usize, (usize, u64)>,
    /// The sites by key.
    sites: HashMap<u64, Site>,
}

// xtask-allow(state-fields): independent: the probe's record of live allocations and their sites, filled by record and drained by forget under one lock; tracking only gates new records and the guard only blocks reentry, so it pairs with either of them
static HEAP: Mutex<Option<Heap>> = Mutex::new(None);
// xtask-allow(state-fields): independent: main turns recording on before the App starts and dump turns it off; frees are forgotten whether or not it is on, so it pairs with any recorded contents
static TRACKING: AtomicBool = AtomicBool::new(false);
// xtask-allow(state-fields): independent: the per-thread reentrancy guard the allocator holds while it records or forgets, and dump keeps set; any thread can be inside with tracking on or off
thread_local! { static INSIDE: Cell<bool> = const { Cell::new(false) }; }
```
`Mutex<Option<…>>` のままにするのは、`HashMap::new()` が `const fn` ではないため。static は一つの static group として数え、3 個なので marker は残す。

##### 書き手・読み手の変更
| file:line | 書換え |
|---|---|
| :58-74 `record` | 二つの lock を一つにする：`if let Ok(mut heap) = HEAP.lock() { let heap = heap.get_or_insert_with(Heap::default); let site = heap.sites.entry(key).or_insert(Site { live: 0, count: 0, ips }); site.live += size; site.count += 1; heap.ptrs.insert(ptr, (size, key)); }` |
| :77-88 `forget` | `if let Ok(mut heap) = HEAP.lock() && let Some(heap) = heap.as_mut() && let Some((size, key)) = heap.ptrs.remove(&ptr) && let Some(site) = heap.sites.get_mut(&key) { site.live -= size; site.count -= 1; }` |
| :139-146 `dump` | `let heap = HEAP.lock().expect("invariant: the probe never poisons its lock"); let sites: Vec<&Site> = heap.iter().flat_map(\|h\| h.sites.values()).filter(\|s\| s.live > 0).collect();`（以降はそのまま） |

##### 振る舞い・試験
- 計測の意味は同じ。一つの lock の下で更新するので、PTRS と SITES がずれて見える瞬間が無くなる（測定値は同じか、より正確）。
- 担保：`cargo build -p kivact-bench --release --bin kv_heap` と、短い実行（`MODE=box ROWS=100 SETTLE_SECS=1`）で `H total_mib` と `H site` の行が出ること。E2E は対象外（bench の probe）。

##### 所有ファイル
`dev/bench/src/bin/kv_heap.rs`

---

#### W4 xtask の `Source.target` と `Source.library`（導出、型で表す）

対象の行：`dev/xtask/src/check_source.rs:16`（target）、`:18`（library）

##### 事実
- `Source { target: Option<String>, library: bool, … }`（check_source.rs:14-29）。
- 本番の書き手は `load` だけ（:174-181）。`walk_file` が push した直後に、同じ loop で `library` と `target` を同時に書き、target の文字列に kind を埋め込む（library なら crate 名、binary なら `package@bin:name`）。
- `walk_file`（:416-419）と `fixture`（:451-454）は `target: None, library: true` で作る。`library` だけ、または `target` だけを書く場所は他に無い（:176-177 だけ）。
- 組合せ：`(None, false)` は到達しない。`(Some(library 名), false)` と `(Some("…@bin:…"), true)` は不正。`library` は target の kind から導ける。marker の「target 名と並んでどの値も有効」は誤り。

##### 新しい型（check_source.rs）
```rust
/// The Cargo target a source file was reached through.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    /// Not reached through a declared target: a checker fixture, or a file
    /// `walk_file` read before `load` names its target. Checked as library code.
    Undeclared,
    /// The package's library, by its crate name.
    Library(String),
    /// One binary of the package, as `package@bin:name`.
    Binary(String),
}

pub(crate) struct Source {
    pub(crate) package: String,
    pub(crate) target: Target,
    pub(crate) module: String,
    // … 他の field はそのまま
}

impl Source {
    /// Whether the source is library code (a library target or an undeclared fixture).
    pub(crate) fn library(&self) -> bool {
        !matches!(self.target, Target::Binary(_))
    }

    pub(crate) fn identity_root(&self) -> String {
        match &self.target {
            Target::Library(name) | Target::Binary(name) => name.clone(),
            Target::Undeclared => self.package.replace('-', "_"),
        }
    }
}
```
二つの marker（:16, :18）は消す（`Target` は数えない名前の enum。Source に数える field は残らない）。

##### 書き手・読み手の変更
| file:line | 今 | 書換え |
|---|---|---|
| check_source.rs:32-36 `identity_root` | `self.target.clone().unwrap_or_else(…)` | 上の match |
| check_source.rs:175-181 `load` | `source.library = library; source.target = Some(if library {…} else {…});` | `source.target = if library { Target::Library(name.replace('-', "_")) } else { Target::Binary(format!("{}@bin:{}", package.replace('-', "_"), name)) };` |
| check_source.rs:418-419 `walk_file` | `target: None, library: true,` | `target: Target::Undeclared,` |
| check_source.rs:453-454 `fixture` | `target: None, library: true,` | `target: Target::Undeclared,` |
| check_source.rs:498（test） | `.filter(\|s\| s.library)` | `.filter(\|s\| s.library())` |
| api_shape.rs:19 | `if !source.library {` | `if !source.library() {` |
| canonical_path.rs:9 | 同上 | 同上 |
| name_collision.rs:53 | 同上 | 同上 |
| structural_options.rs:16 | 同上 | 同上 |
| public_surface.rs:56 | `.filter(\|s\| s.library && …)` | `.filter(\|s\| s.library() && …)` |
| public_type_edges.rs:24 | `.filter(\|s\| s.library)` | `.filter(\|s\| s.library())` |
| state_fields/check.rs:82 `is_value_module` | `if source.library {` | `if source.library() {` |

`Binding.target`（public_surface.rs:18）は別の struct の `String` で、名前が同じだけなので変えない。

##### 振る舞い・試験
- 振る舞いは同じ（library の判定は、load の後も fixture でも今と同じ値になる）。
- 担保：`cargo test -p xtask`（check_source.rs:490-505 の library 1 件・identity 5 件の test、各 rule の fixture test）と、`cargo xtask` の通常の check で結果が変わらないこと（`--report-audit` は使わない）。

##### 所有ファイル
`dev/xtask/src/check_source.rs`、`dev/xtask/src/api_shape.rs`、`dev/xtask/src/canonical_path.rs`、`dev/xtask/src/name_collision.rs`、`dev/xtask/src/structural_options.rs`、`dev/xtask/src/public_surface.rs`、`dev/xtask/src/public_type_edges.rs`、`dev/xtask/src/state_fields/check.rs`

---

#### 報告事項（WRONG ではない）
- `crates/kivact-i18n/src/numbers.rs:151,153` `SymbolCapture.symbols/digits`：SM-5 型の out-parameter slot。書き手は ICU の `DataProvider::load` で、constructor には渡せない（SM-5 の「どちらも無理なら停止して報告」に当たる）。`cache:` は誤りなので、KIND で `independent:` にした（理由文で ICU が書き手であることを明記）。lint 側でこの形をどう扱うか、判断が要る。
- `kivact-test` の試験 slot（`Rc<Cell<Option<…>>>` を render closure が一度だけ埋める形）は SM-5 型。P5（p5-part-D.md:245）と同じく、lint 側の課題として OK にした。


# 付録 B：全 917 行の判定表

列の意味：
- marker（HEAD）：HEAD の marker 行。
- 閉じ方：判定の後の prefix、または type／derive。
- unit：変更する行の担当 unit（6 章）。
- 根拠：確認済みの行は 140 字で切った。
- 新しい marker 行：その位置に置く全文。行番号は除いてある。

本文 3.4 の上書きは反映済み。

### B.1 G1

| marker（HEAD） | owner.field | 判定 | 閉じ方 | unit | 根拠 | 新しい marker 行 |
|---|---|---|---|---|---|---|
| `crates/kivact-style/src/draw/layer.rs:40` | Layer.area | 文言 | option | P8-L1 | 書き手は Layer::area のみ（layer.rs:86-89）。None は「無い」ではなく push_group で Draw::size 全体に置換（layer.rs:207）され group・clip 形状・mask の基準になる。理由文が事実と違う。 | `` // xtask-allow(state-fields): option: set by Layer::area; absent means the group, its clip shape and its mask use the whole canvas (Draw::size) `` |
| `crates/kivact-style/src/draw/layer.rs:42` | Layer.transform | 確認済み | option | — | 書き手は builder メソッドのみ（layer.rs:86-151）。pub(super) で draw 内に他の書き手なし（layer.rs:183,201 は Draw.transform）。読みは Draw::layer/push_group（layer.rs:177-… |  |
| `crates/kivact-style/src/draw/layer.rs:46` | Layer.clip_rect | 確認済み | option | — | 書き手は builder メソッドのみ（layer.rs:86-151）。pub(super) で draw 内に他の書き手なし（layer.rs:183,201 は Draw.transform）。読みは Draw::layer/push_group（layer.rs:177-… |  |
| `crates/kivact-style/src/draw/layer.rs:48` | Layer.clip | 確認済み | option | — | 書き手は builder メソッドのみ（layer.rs:86-151）。pub(super) で draw 内に他の書き手なし（layer.rs:183,201 は Draw.transform）。読みは Draw::layer/push_group（layer.rs:177-… |  |
| `crates/kivact-style/src/draw/layer.rs:54` | Layer.mask | 確認済み | option | — | 書き手は builder メソッドのみ（layer.rs:86-151）。pub(super) で draw 内に他の書き手なし（layer.rs:183,201 は Draw.transform）。読みは Draw::layer/push_group（layer.rs:177-… |  |
| `crates/kivact-style/src/draw/layer.rs:56` | Layer.shadow | 確認済み | option | — | 書き手は builder メソッドのみ（layer.rs:86-151）。pub(super) で draw 内に他の書き手なし（layer.rs:183,201 は Draw.transform）。読みは Draw::layer/push_group（layer.rs:177-… |  |
| `crates/kivact-style/src/draw/text_run.rs:44` | TextFormat.family | 確認済み | option | — | pub field の値型。書き手は呼出し側の struct literal（devtools/ui-basic/ui-creative 等で描画ごとに構築）と Default（text_run.rs:92-115）のみ。text_run（text_run.rs:126-167）… |  |
| `crates/kivact-style/src/draw/text_run.rs:50` | TextFormat.italic | 文言 | option | P8-L1 | 書き手は呼出し側 literal と Default（text_run.rs:101）のみ。bool で absent は無く、false は継承ではなく upright の face（TextRun.italic へ素通し text_run.rs:154）。理由文が事実と合わない。 | `` // xtask-allow(state-fields): option: set by the caller's format literal; false (the default) draws the upright face, true the family's italic face `` |
| `crates/kivact-style/src/draw/text_run.rs:53` | TextFormat.line_height | 確認済み | option | — | pub field の値型。書き手は呼出し側の struct literal（devtools/ui-basic/ui-creative 等で描画ごとに構築）と Default（text_run.rs:92-115）のみ。text_run（text_run.rs:126-167）… |  |
| `crates/kivact-style/src/draw/text_run.rs:71` | TextFormat.tabular_nums | 文言 | option | P8-L1 | 書き手は呼出し側 literal と Default（text_run.rs:107）のみ。bool で absent は無く、false は tnum を足さない（text_run.rs:135、text.rs:358-363）。継承でもない。 | `` // xtask-allow(state-fields): option: set by the caller's format literal; false (the default) keeps the font's digit widths, true adds the tnum feature `` |
| `crates/kivact-style/src/draw/text_run.rs:76` | TextFormat.decoration_color | 確認済み | option | — | pub field の値型。書き手は呼出し側の struct literal（devtools/ui-basic/ui-creative 等で描画ごとに構築）と Default（text_run.rs:92-115）のみ。text_run（text_run.rs:126-167）… |  |
| `crates/kivact-style/src/draw/text_run.rs:79` | TextFormat.fill | 確認済み | option | — | pub field の値型。書き手は呼出し側の struct literal（devtools/ui-basic/ui-creative 等で描画ごとに構築）と Default（text_run.rs:92-115）のみ。text_run（text_run.rs:126-167）… |  |
| `crates/kivact-style/src/draw/text_run.rs:82` | TextFormat.stroke | 確認済み | option | — | pub field の値型。書き手は呼出し側の struct literal（devtools/ui-basic/ui-creative 等で描画ごとに構築）と Default（text_run.rs:92-115）のみ。text_run（text_run.rs:126-167）… |  |
| `crates/kivact-style/src/grid.rs:263` | GridStyle.columns | 確認済み | option | — | 書き手は grid_setter!（grid_setters.rs:10-67、Style builder）と merge（grid.rs:285-304、authored style の重ね合わせ）のみ。pub(crate) で他になし。読みは grid::resolve（co… |  |
| `crates/kivact-style/src/grid.rs:265` | GridStyle.rows | 確認済み | option | — | 書き手は grid_setter!（grid_setters.rs:10-67、Style builder）と merge（grid.rs:285-304、authored style の重ね合わせ）のみ。pub(crate) で他になし。読みは grid::resolve（co… |  |
| `crates/kivact-style/src/grid.rs:267` | GridStyle.auto_columns | 確認済み | option | — | 書き手は grid_setter!（grid_setters.rs:10-67、Style builder）と merge（grid.rs:285-304、authored style の重ね合わせ）のみ。pub(crate) で他になし。読みは grid::resolve（co… |  |
| `crates/kivact-style/src/grid.rs:269` | GridStyle.auto_rows | 確認済み | option | — | 書き手は grid_setter!（grid_setters.rs:10-67、Style builder）と merge（grid.rs:285-304、authored style の重ね合わせ）のみ。pub(crate) で他になし。読みは grid::resolve（co… |  |
| `crates/kivact-style/src/grid.rs:271` | GridStyle.flow | 確認済み | option | — | 書き手は grid_setter!（grid_setters.rs:10-67、Style builder）と merge（grid.rs:285-304、authored style の重ね合わせ）のみ。pub(crate) で他になし。読みは grid::resolve（co… |  |
| `crates/kivact-style/src/grid.rs:273` | GridStyle.column | 確認済み | option | — | 書き手は grid_setter!（grid_setters.rs:10-67、Style builder）と merge（grid.rs:285-304、authored style の重ね合わせ）のみ。pub(crate) で他になし。読みは grid::resolve（co… |  |
| `crates/kivact-style/src/grid.rs:275` | GridStyle.row | 確認済み | option | — | 書き手は grid_setter!（grid_setters.rs:10-67、Style builder）と merge（grid.rs:285-304、authored style の重ね合わせ）のみ。pub(crate) で他になし。読みは grid::resolve（co… |  |
| `crates/kivact-style/src/grid.rs:277` | GridStyle.justify_items | 確認済み | option | — | 書き手は grid_setter!（grid_setters.rs:10-67、Style builder）と merge（grid.rs:285-304、authored style の重ね合わせ）のみ。pub(crate) で他になし。読みは grid::resolve（co… |  |
| `crates/kivact-style/src/grid.rs:279` | GridStyle.justify_self | 確認済み | option | — | 書き手は grid_setter!（grid_setters.rs:10-67、Style builder）と merge（grid.rs:285-304、authored style の重ね合わせ）のみ。pub(crate) で他になし。読みは grid::resolve（co… |  |
| `crates/kivact-style/src/paint/border_image.rs:61` | BorderImage.slice_pct | 文言 | option | P8-L1 | 書き手は slice()=false／slice_pct()=true（border_image.rs:86-97）のみ。slice の単位で false は px（border_image.rs:155）。継承・absent の概念が無く理由文が事実と違う。 | `` // xtask-allow(state-fields): option: set by slice_pct() and cleared by slice(); false means the slice insets are px of the source, true percent `` |
| `crates/kivact-style/src/paint/border_image.rs:63` | BorderImage.fill | 文言 | option | P8-L1 | 書き手は fill()（border_image.rs:100-103）のみ。bool で absent は無く、false は中央を塗らない（CSS 既定、border_image.rs:156）。継承ではない。 | `` // xtask-allow(state-fields): option: set by fill(); false (the CSS default) leaves the middle of the source unpainted `` |
| `crates/kivact-style/src/paint/border_image.rs:65` | BorderImage.width | 確認済み | option | — | private field。書き手は builder（border_image.rs:73-131）のみ。読みは wire（border_image.rs:134-166）。 None は border 幅（border_image.rs:105-106）＝既定。 |  |
| `crates/kivact-style/src/paint/layer.rs:38` | Paint.repeat.1 | 閉じ方の変更 | option | P8-L1 | 書き手は Paint::of=(true,true) と no_repeat/repeat_x/repeat_y（layer.rs:69,157-172）の builder のみで authored 値。4 組合せとも到達するが理由が禁止句のみ。builder 設定値なので option。 | `` // xtask-allow(state-fields): option: set by no_repeat, repeat_x and repeat_y (default (true, true)); false on an axis means one tile along that axis (CSS background-repeat) `` |
| `crates/kivact-style/src/paint/layer.rs:41` | Paint.origin | 確認済み | option | — | pub(crate)。書き手は Paint::of（layer.rs:64-74）と builder（origin layer.rs:86-89、repeat 系 layer.rs:157-172）のみ。 None は文脈の既定（背景 padding box、mask borde… |  |
| `crates/kivact-style/src/resolve/compile.rs:31` | CompiledStyle.retained_paint | 文言 | option | P8-L1 | P8 統合で上書き：projection にすると X4 の package 範囲検査で他 struct の同名 field と衝突する（3.6 O2 と同じ理由）。compile_style の出力で、authored な Style の option を写すので option のまま文言だけ直す。 元の判定：private。唯一の書き手は compile_style の literal（compile.rs:367-370,406-407）で Style.paint.local_paint の写し。authored な設定値ではなく compile 結果なので projection。 | `` // xtask-allow(state-fields): option: compile_style's copy of the style's base paint callback (Style::paint); absent when the style sets none `` |
| `crates/kivact-style/src/resolve/compile.rs:37` | CompiledStyle.text | 文言 | option | P8-L1 | P8 統合で上書き：`theme/value.rs:342` の `self.text = text` が X4（pub field の package 範囲）で「他の書き手」と誤検出される。option のまま文言を直す。 元の判定：pub だが書き手は compile_style（compile.rs:390-402,410）のみ（repo 全体 grep、bind_style は読むだけ style.rs:174）。has_text()\|\|root の投影で authored 値ではない。 | `` // xtask-allow(state-fields): option: compile_style's text properties, Some when the style sets one or the node is the root (which carries the theme's text defaults); absent inherits every text property `` |
| `crates/kivact-style/src/resolve/compile.rs:42` | CompiledStyle.states | 文言 | option | P8-L1 | P8 統合で上書き（同上）。 元の判定：pub だが書き手は compile_style→state_styles（compile.rs:305-355,388,411）のみ。読みは style.rs:177 の unwrap_or_default。Style.states の投影で authored 値ではない。 | `` // xtask-allow(state-fields): option: compile_style's host state deltas (state_styles), Some when the style sets a hover, pressed, focus, group or enter/exit style; absent means none `` |
| `crates/kivact-style/src/resolve/compile.rs:223` | Interactive.focusable | 閉じ方の変更 | option | P8-L1 | 唯一の構築は bind_style（style.rs:129-135）で毎回作り保持しない入力。root 節点は NodeStyle::default（root.rs:95-107）で (root,focusable)=(T,T) は到達せず「全組合せ」は偽。focus-stop は mount の事実。 | `` // xtask-allow(state-fields): option: a compile input bind_style sets from the mount's focus-stop fact; false means compile_style adds no theme focus ring or disabled opacity `` |
| `crates/kivact-style/src/resolve/compile.rs:228` | Interactive.disabled | 閉じ方の変更 | option | P8-L1 | bind_style が caller の disabled prop を毎回読む（style.rs:131）保持しない引数。root では reader が None で常に false（root.rs:105）→全組合せ到達は偽。DevTools 強制は compile 内（compile.rs:361-363）。 | `` // xtask-allow(state-fields): option: a compile input bind_style reads from the caller's disabled prop on every style run; false means the disabled state style is not merged and hover/pressed looks stay `` |
| `crates/kivact-style/src/resolve/compile.rs:232` | Interactive.selected | 閉じ方の変更 | option | P8-L1 | bind_style が caller の selected prop を毎回読む（style.rs:132）保持しない引数。root では常に false（root.rs:105）で全組合せ到達は偽。強制は compile.rs:364-366。 | `` // xtask-allow(state-fields): option: a compile input bind_style reads from the caller's selected prop on every style run; false means the selected state style is not merged `` |
| `crates/kivact-style/src/resolve/compile.rs:235` | Interactive.root | 閉じ方の変更 | option | P8-L1 | bind_style が kind==Root で設定（style.rs:133）。root=true なら focusable/disabled/selected は常に false（root.rs:95-107）で独立ではない。節点種別という caller 入力。 | `` // xtask-allow(state-fields): option: a compile input bind_style sets when the node kind is Root; false means unset text props are left for the host to inherit instead of filled from the theme `` |
| `crates/kivact-style/src/sheet/fields.rs:20` | Sides.top | 確認済み | option | — | 書き手は sides! macro（setters.rs:50-98、builder）と Sides::merge（fields.rs:31-36）のみ。読みは compile.rs:69-76。 |  |
| `crates/kivact-style/src/sheet/fields.rs:22` | Sides.right | 確認済み | option | — | 書き手は sides! macro（setters.rs:50-98、builder）と Sides::merge（fields.rs:31-36）のみ。読みは compile.rs:69-76。 |  |
| `crates/kivact-style/src/sheet/fields.rs:24` | Sides.bottom | 確認済み | option | — | 書き手は sides! macro（setters.rs:50-98、builder）と Sides::merge（fields.rs:31-36）のみ。読みは compile.rs:69-76。 |  |
| `crates/kivact-style/src/sheet/fields.rs:26` | Sides.left | 確認済み | option | — | 書き手は sides! macro（setters.rs:50-98、builder）と Sides::merge（fields.rs:31-36）のみ。読みは compile.rs:69-76。 |  |
| `crates/kivact-style/src/sheet/fields.rs:42` | SizeFields.width | 確認済み | option | — | 書き手は setter! macro（setters.rs:112-136、builder）と merge（fields.rs:65-85）のみ（_mut 代入を grep 済）。読みは compile.rs:78-131。 |  |
| `crates/kivact-style/src/sheet/fields.rs:44` | SizeFields.height | 確認済み | option | — | 書き手は setter! macro（setters.rs:112-136、builder）と merge（fields.rs:65-85）のみ（_mut 代入を grep 済）。読みは compile.rs:78-131。 |  |
| `crates/kivact-style/src/sheet/fields.rs:46` | SizeFields.min_width | 確認済み | option | — | 書き手は setter! macro（setters.rs:112-136、builder）と merge（fields.rs:65-85）のみ（_mut 代入を grep 済）。読みは compile.rs:78-131。 |  |
| `crates/kivact-style/src/sheet/fields.rs:48` | SizeFields.min_height | 確認済み | option | — | 書き手は setter! macro（setters.rs:112-136、builder）と merge（fields.rs:65-85）のみ（_mut 代入を grep 済）。読みは compile.rs:78-131。 |  |
| `crates/kivact-style/src/sheet/fields.rs:50` | SizeFields.max_width | 確認済み | option | — | 書き手は setter! macro（setters.rs:112-136、builder）と merge（fields.rs:65-85）のみ（_mut 代入を grep 済）。読みは compile.rs:78-131。 |  |
| `crates/kivact-style/src/sheet/fields.rs:52` | SizeFields.max_height | 確認済み | option | — | 書き手は setter! macro（setters.rs:112-136、builder）と merge（fields.rs:65-85）のみ（_mut 代入を grep 済）。読みは compile.rs:78-131。 |  |
| `crates/kivact-style/src/sheet/fields.rs:54` | SizeFields.basis | 確認済み | option | — | 書き手は setter! macro（setters.rs:112-136、builder）と merge（fields.rs:65-85）のみ（_mut 代入を grep 済）。読みは compile.rs:78-131。 |  |
| `crates/kivact-style/src/sheet/fields.rs:56` | SizeFields.aspect_ratio | 確認済み | option | — | 書き手は setter! macro（setters.rs:112-136、builder）と merge（fields.rs:65-85）のみ（_mut 代入を grep 済）。読みは compile.rs:78-131。 |  |
| `crates/kivact-style/src/sheet/fields.rs:58` | SizeFields.grow | 確認済み | option | — | 書き手は setter! macro（setters.rs:112-136、builder）と merge（fields.rs:65-85）のみ（_mut 代入を grep 済）。読みは compile.rs:78-131。 |  |
| `crates/kivact-style/src/sheet/fields.rs:60` | SizeFields.shrink | 確認済み | option | — | 書き手は setter! macro（setters.rs:112-136、builder）と merge（fields.rs:65-85）のみ（_mut 代入を grep 済）。読みは compile.rs:78-131。 |  |
| `crates/kivact-style/src/sheet/fields.rs:94` | SpacingFields.row_gap | 確認済み | option | — | 書き手は setter!（setters.rs:100-107）と merge（fields.rs:101-111）のみ。読みは compile.rs:107-108。 |  |
| `crates/kivact-style/src/sheet/fields.rs:96` | SpacingFields.column_gap | 確認済み | option | — | 書き手は setter!（setters.rs:100-107）と merge（fields.rs:101-111）のみ。読みは compile.rs:107-108。 |  |
| `crates/kivact-style/src/sheet/fields.rs:117` | FlowFields.direction | 確認済み | option | — | 書き手は setter!（setters.rs:108-217）・display等（setters.rs:68,821,847）・with_scroll_axes（setters.rs:917）・merge（fields.rs:152-178）のみ。scroll_axes の o… |  |
| `crates/kivact-style/src/sheet/fields.rs:119` | FlowFields.align | 確認済み | option | — | 書き手は setter!（setters.rs:108-217）・display等（setters.rs:68,821,847）・with_scroll_axes（setters.rs:917）・merge（fields.rs:152-178）のみ。scroll_axes の o… |  |
| `crates/kivact-style/src/sheet/fields.rs:121` | FlowFields.align_self | 確認済み | option | — | 書き手は setter!（setters.rs:108-217）・display等（setters.rs:68,821,847）・with_scroll_axes（setters.rs:917）・merge（fields.rs:152-178）のみ。scroll_axes の o… |  |
| `crates/kivact-style/src/sheet/fields.rs:123` | FlowFields.justify | 確認済み | option | — | 書き手は setter!（setters.rs:108-217）・display等（setters.rs:68,821,847）・with_scroll_axes（setters.rs:917）・merge（fields.rs:152-178）のみ。scroll_axes の o… |  |
| `crates/kivact-style/src/sheet/fields.rs:125` | FlowFields.align_content | 確認済み | option | — | 書き手は setter!（setters.rs:108-217）・display等（setters.rs:68,821,847）・with_scroll_axes（setters.rs:917）・merge（fields.rs:152-178）のみ。scroll_axes の o… |  |
| `crates/kivact-style/src/sheet/fields.rs:127` | FlowFields.display | 確認済み | option | — | 書き手は setter!（setters.rs:108-217）・display等（setters.rs:68,821,847）・with_scroll_axes（setters.rs:917）・merge（fields.rs:152-178）のみ。scroll_axes の o… |  |
| `crates/kivact-style/src/sheet/fields.rs:131` | FlowFields.visibility | 確認済み | option | — | 書き手は setter!（setters.rs:108-217）・display等（setters.rs:68,821,847）・with_scroll_axes（setters.rs:917）・merge（fields.rs:152-178）のみ。scroll_axes の o… |  |
| `crates/kivact-style/src/sheet/fields.rs:133` | FlowFields.grid | 確認済み | option | — | 書き手は setter!（setters.rs:108-217）・display等（setters.rs:68,821,847）・with_scroll_axes（setters.rs:917）・merge（fields.rs:152-178）のみ。scroll_axes の o… |  |
| `crates/kivact-style/src/sheet/fields.rs:136` | FlowFields.wrap | 確認済み | option | — | 書き手は setter!（setters.rs:108-217）・display等（setters.rs:68,821,847）・with_scroll_axes（setters.rs:917）・merge（fields.rs:152-178）のみ。scroll_axes の o… |  |
| `crates/kivact-style/src/sheet/fields.rs:138` | FlowFields.position | 確認済み | option | — | 書き手は setter!（setters.rs:108-217）・display等（setters.rs:68,821,847）・with_scroll_axes（setters.rs:917）・merge（fields.rs:152-178）のみ。scroll_axes の o… |  |
| `crates/kivact-style/src/sheet/fields.rs:140` | FlowFields.overflow | 確認済み | option | — | 書き手は setter!（setters.rs:108-217）・display等（setters.rs:68,821,847）・with_scroll_axes（setters.rs:917）・merge（fields.rs:152-178）のみ。scroll_axes の o… |  |
| `crates/kivact-style/src/sheet/fields.rs:142` | FlowFields.scroll_axes | 確認済み | option | — | 書き手は setter!（setters.rs:108-217）・display等（setters.rs:68,821,847）・with_scroll_axes（setters.rs:917）・merge（fields.rs:152-178）のみ。scroll_axes の o… |  |
| `crates/kivact-style/src/sheet/fields.rs:145` | FlowFields.scroll_snap | 確認済み | option | — | 書き手は setter!（setters.rs:108-217）・display等（setters.rs:68,821,847）・with_scroll_axes（setters.rs:917）・merge（fields.rs:152-178）のみ。scroll_axes の o… |  |
| `crates/kivact-style/src/sheet/fields.rs:147` | FlowFields.z_index | 確認済み | option | — | 書き手は setter!（setters.rs:108-217）・display等（setters.rs:68,821,847）・with_scroll_axes（setters.rs:917）・merge（fields.rs:152-178）のみ。scroll_axes の o… |  |
| `crates/kivact-style/src/sheet/fields.rs:185` | PaintFields.local_paint | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:187` | PaintFields.background | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:194` | PaintFields.ring | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:198` | PaintFields.shadows | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:200` | PaintFields.opacity | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:202` | PaintFields.isolation | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:204` | PaintFields.border_image | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:206` | PaintFields.scrollbar | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:208` | PaintFields.cursor | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:210` | PaintFields.clip | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:212` | PaintFields.filter | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:214` | PaintFields.backdrop_filter | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:216` | PaintFields.blend | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:218` | PaintFields.mask | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:220` | PaintFields.clip_path | 確認済み | option | — | 書き手は setter!（setters.rs:166-238,432-437）・ring/shadow/paint/scrollbar_*・merge（fields.rs:225-266）と compile 内の局所 copy への theme 既定（compile.rs:27… |  |
| `crates/kivact-style/src/sheet/fields.rs:275` | TransformFields.rotate | 確認済み | option | — | 書き手は setter!（setters.rs:199-209,385-428）・scale/transform_origin/perspective_origin（setters.rs:363-414）と merge（fields.rs:304-330）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:277` | TransformFields.scale | 確認済み | option | — | 書き手は setter!（setters.rs:199-209,385-428）・scale/transform_origin/perspective_origin（setters.rs:363-414）と merge（fields.rs:304-330）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:279` | TransformFields.transform | 確認済み | option | — | 書き手は setter!（setters.rs:199-209,385-428）・scale/transform_origin/perspective_origin（setters.rs:363-414）と merge（fields.rs:304-330）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:281` | TransformFields.transform_origin | 確認済み | option | — | 書き手は setter!（setters.rs:199-209,385-428）・scale/transform_origin/perspective_origin（setters.rs:363-414）と merge（fields.rs:304-330）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:283` | TransformFields.translate_z | 確認済み | option | — | 書き手は setter!（setters.rs:199-209,385-428）・scale/transform_origin/perspective_origin（setters.rs:363-414）と merge（fields.rs:304-330）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:285` | TransformFields.rotate_x | 確認済み | option | — | 書き手は setter!（setters.rs:199-209,385-428）・scale/transform_origin/perspective_origin（setters.rs:363-414）と merge（fields.rs:304-330）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:287` | TransformFields.rotate_y | 確認済み | option | — | 書き手は setter!（setters.rs:199-209,385-428）・scale/transform_origin/perspective_origin（setters.rs:363-414）と merge（fields.rs:304-330）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:289` | TransformFields.transform_origin_z | 確認済み | option | — | 書き手は setter!（setters.rs:199-209,385-428）・scale/transform_origin/perspective_origin（setters.rs:363-414）と merge（fields.rs:304-330）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:291` | TransformFields.perspective | 確認済み | option | — | 書き手は setter!（setters.rs:199-209,385-428）・scale/transform_origin/perspective_origin（setters.rs:363-414）と merge（fields.rs:304-330）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:293` | TransformFields.perspective_origin | 確認済み | option | — | 書き手は setter!（setters.rs:199-209,385-428）・scale/transform_origin/perspective_origin（setters.rs:363-414）と merge（fields.rs:304-330）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:295` | TransformFields.transform_style | 確認済み | option | — | 書き手は setter!（setters.rs:199-209,385-428）・scale/transform_origin/perspective_origin（setters.rs:363-414）と merge（fields.rs:304-330）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:297` | TransformFields.backface | 確認済み | option | — | 書き手は setter!（setters.rs:199-209,385-428）・scale/transform_origin/perspective_origin（setters.rs:363-414）と merge（fields.rs:304-330）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:299` | TransformFields.will_change | 確認済み | option | — | 書き手は setter!（setters.rs:199-209,385-428）・scale/transform_origin/perspective_origin（setters.rs:363-414）と merge（fields.rs:304-330）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:337` | TextFields.font_size | 確認済み | option | — | 書き手は setter!/color_setter!（setters.rs:241-283）・text_setter!（text_setters.rs）と merge（fields.rs:360-381）のみ。読みは compile.rs:163-187。None は親からの継承… |  |
| `crates/kivact-style/src/sheet/fields.rs:339` | TextFields.weight | 確認済み | option | — | 書き手は setter!/color_setter!（setters.rs:241-283）・text_setter!（text_setters.rs）と merge（fields.rs:360-381）のみ。読みは compile.rs:163-187。None は親からの継承… |  |
| `crates/kivact-style/src/sheet/fields.rs:341` | TextFields.color | 確認済み | option | — | 書き手は setter!/color_setter!（setters.rs:241-283）・text_setter!（text_setters.rs）と merge（fields.rs:360-381）のみ。読みは compile.rs:163-187。None は親からの継承… |  |
| `crates/kivact-style/src/sheet/fields.rs:343` | TextFields.line_height | 確認済み | option | — | 書き手は setter!/color_setter!（setters.rs:241-283）・text_setter!（text_setters.rs）と merge（fields.rs:360-381）のみ。読みは compile.rs:163-187。None は親からの継承… |  |
| `crates/kivact-style/src/sheet/fields.rs:345` | TextFields.text_wrap | 確認済み | option | — | 書き手は setter!/color_setter!（setters.rs:241-283）・text_setter!（text_setters.rs）と merge（fields.rs:360-381）のみ。読みは compile.rs:163-187。None は親からの継承… |  |
| `crates/kivact-style/src/sheet/fields.rs:347` | TextFields.text_align | 確認済み | option | — | 書き手は setter!/color_setter!（setters.rs:241-283）・text_setter!（text_setters.rs）と merge（fields.rs:360-381）のみ。読みは compile.rs:163-187。None は親からの継承… |  |
| `crates/kivact-style/src/sheet/fields.rs:349` | TextFields.font_family | 確認済み | option | — | 書き手は setter!/color_setter!（setters.rs:241-283）・text_setter!（text_setters.rs）と merge（fields.rs:360-381）のみ。読みは compile.rs:163-187。None は親からの継承… |  |
| `crates/kivact-style/src/sheet/fields.rs:351` | TextFields.italic | 確認済み | option | — | 書き手は setter!/color_setter!（setters.rs:241-283）・text_setter!（text_setters.rs）と merge（fields.rs:360-381）のみ。読みは compile.rs:163-187。None は親からの継承… |  |
| `crates/kivact-style/src/sheet/fields.rs:353` | TextFields.text | 確認済み | option | — | 書き手は setter!/color_setter!（setters.rs:241-283）・text_setter!（text_setters.rs）と merge（fields.rs:360-381）のみ。読みは compile.rs:163-187。None は親からの継承… |  |
| `crates/kivact-style/src/sheet/fields.rs:440` | MotionFields.enter_size | 確認済み | option | — | 書き手は enter_size/exit_size/layout_transition（setters.rs:793-809）と merge（fields.rs:450-464）のみ。読みは compile.rs:93-95。 |  |
| `crates/kivact-style/src/sheet/fields.rs:442` | MotionFields.exit_size | 確認済み | option | — | 書き手は enter_size/exit_size/layout_transition（setters.rs:793-809）と merge（fields.rs:450-464）のみ。読みは compile.rs:93-95。 |  |
| `crates/kivact-style/src/sheet/fields.rs:444` | MotionFields.layout_transition | 確認済み | option | — | 書き手は enter_size/exit_size/layout_transition（setters.rs:793-809）と merge（fields.rs:450-464）のみ。読みは compile.rs:93-95。 |  |
| `crates/kivact-style/src/sheet/fields.rs:491` | ScrollbarStyle.width | 確認済み | option | — | 書き手は scrollbar_width/scrollbar_color（setters.rs:219-232）と PaintFields::merge（fields.rs:249-253）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:493` | ScrollbarStyle.colors | 確認済み | option | — | 書き手は scrollbar_width/scrollbar_color（setters.rs:219-232）と PaintFields::merge（fields.rs:249-253）のみ。 |  |
| `crates/kivact-style/src/sheet/fields.rs:502` | Style.size | 確認済み | option | — | 書き手は *_mut()（fields.rs:549-572、builder setter 経由）と Style::merge（setters.rs:871-898）のみ。None は group_or_default! で既定 group として読まれる（compile.rs:… |  |
| `crates/kivact-style/src/sheet/fields.rs:504` | Style.spacing | 確認済み | option | — | 書き手は *_mut()（fields.rs:549-572、builder setter 経由）と Style::merge（setters.rs:871-898）のみ。None は group_or_default! で既定 group として読まれる（compile.rs:… |  |
| `crates/kivact-style/src/sheet/fields.rs:506` | Style.flow | 確認済み | option | — | 書き手は *_mut()（fields.rs:549-572、builder setter 経由）と Style::merge（setters.rs:871-898）のみ。None は group_or_default! で既定 group として読まれる（compile.rs:… |  |
| `crates/kivact-style/src/sheet/fields.rs:508` | Style.paint | 確認済み | option | — | 書き手は *_mut()（fields.rs:549-572、builder setter 経由）と Style::merge（setters.rs:871-898）のみ。None は group_or_default! で既定 group として読まれる（compile.rs:… |  |
| `crates/kivact-style/src/sheet/fields.rs:510` | Style.transform | 確認済み | option | — | 書き手は *_mut()（fields.rs:549-572、builder setter 経由）と Style::merge（setters.rs:871-898）のみ。None は group_or_default! で既定 group として読まれる（compile.rs:… |  |
| `crates/kivact-style/src/sheet/fields.rs:512` | Style.text | 確認済み | option | — | 書き手は *_mut()（fields.rs:549-572、builder setter 経由）と Style::merge（setters.rs:871-898）のみ。None は group_or_default! で既定 group として読まれる（compile.rs:… |  |
| `crates/kivact-style/src/sheet/fields.rs:514` | Style.states | 確認済み | option | — | 書き手は *_mut()（fields.rs:549-572、builder setter 経由）と Style::merge（setters.rs:871-898）のみ。None は group_or_default! で既定 group として読まれる（compile.rs:… |  |
| `crates/kivact-style/src/sheet/fields.rs:516` | Style.motion | 確認済み | option | — | 書き手は *_mut()（fields.rs:549-572、builder setter 経由）と Style::merge（setters.rs:871-898）のみ。None は group_or_default! で既定 group として読まれる（compile.rs:… |  |
| `crates/kivact-style/src/text.rs:262` | TextExtras.letter_spacing | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:264` | TextExtras.word_spacing | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:266` | TextExtras.kerning | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:268` | TextExtras.features | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:270` | TextExtras.tabular_nums | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:272` | TextExtras.decoration_line | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:274` | TextExtras.decoration_style | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:276` | TextExtras.decoration_color | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:278` | TextExtras.decoration_thickness | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:280` | TextExtras.underline_offset | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:282` | TextExtras.overflow | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:284` | TextExtras.line_clamp | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:286` | TextExtras.capsize | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:288` | TextExtras.transform | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:290` | TextExtras.shadows | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:292` | TextExtras.caret_color | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:294` | TextExtras.selection_color | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:296` | TextExtras.placeholder_color | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:298` | TextExtras.stroke | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:300` | TextExtras.paint_order | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:302` | TextExtras.variations | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:304` | TextExtras.optical_sizing | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:306` | TextExtras.font_width | 確認済み | option | — | 書き手は text_setter! 等（text_setters.rs:12-218、setters.rs:826-843）と merge（text.rs:312-345）のみ。読みは resolve_extras（text.rs:407-463）。 |  |
| `crates/kivact-style/src/text.rs:471` | SpanStyle.font_size | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:474` | SpanStyle.weight | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:477` | SpanStyle.color | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:480` | SpanStyle.family | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:483` | SpanStyle.italic | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:486` | SpanStyle.letter_spacing | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:489` | SpanStyle.word_spacing | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:492` | SpanStyle.kerning | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:495` | SpanStyle.features | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:498` | SpanStyle.tabular_nums | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:501` | SpanStyle.decoration_line | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:504` | SpanStyle.decoration_color | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:507` | SpanStyle.decoration_thickness | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:510` | SpanStyle.underline_offset | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:513` | SpanStyle.background | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:516` | SpanStyle.variations | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:519` | SpanStyle.font_width | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:522` | SpanStyle.stroke | 確認済み | option | — | pub だが書き手は Span builder（kivact-core span.rs:62-）と Default のみ。読みは compile_span_style（internal.rs:14-62）。struct doc（text.rs:465-468）も unset は段… |  |
| `crates/kivact-style/src/text.rs:575` | StyledRange.color | 確認済み | option | — | pub の値型。書き手は highlighter callback の返す literal（ui-basic highlight.rs:38、ui-soft code_syntax.rs:134、ui code_editing.rs:31,44）のみで保持状態でない。None は… |  |
| `crates/kivact-style/src/text.rs:577` | StyledRange.background | 確認済み | option | — | pub の値型。書き手は highlighter callback の返す literal（ui-basic highlight.rs:38、ui-soft code_syntax.rs:134、ui code_editing.rs:31,44）のみで保持状態でない。None は… |  |
| `crates/kivact-style/src/text.rs:579` | StyledRange.decoration | 確認済み | option | — | pub の値型。書き手は highlighter callback の返す literal（ui-basic highlight.rs:38、ui-soft code_syntax.rs:134、ui code_editing.rs:31,44）のみで保持状態でない。None は… |  |
| `crates/kivact-style/src/text.rs:587` | TextDecoration.color | 確認済み | option | — | pub の値型。書き手は呼出し側 literal のみ。読みは compile_text_decoration（internal.rs:65-73）：color None=文字色、thickness None=font の既定。 |  |
| `crates/kivact-style/src/text.rs:589` | TextDecoration.thickness | 確認済み | option | — | pub の値型。書き手は呼出し側 literal のみ。読みは compile_text_decoration（internal.rs:65-73）：color None=文字色、thickness None=font の既定。 |  |

### B.2 G2

| marker（HEAD） | owner.field | 判定 | 閉じ方 | unit | 根拠 | 新しい marker 行 |
|---|---|---|---|---|---|---|
| `crates/kivact-ui/src/calendar/navigation.rs:73` | SwipeSnapshot.rtl | 文言 | independent | P8-UI6 | writer: calendar/view.rs:422 swipe_navigation の snapshot closure のみ（use_direction）。at_min/at_max は bounds 由来で方向と無関係、sign()/shift() が全組合せを処理（navigation.rs:84-93）。理由文が「every combination is valid」の汎用句のみ。 | `` // xtask-allow(state-fields): independent: computed per event in swipe_navigation from the inherited reading direction; it only flips the drag sign, so it pairs with any at_min/at_max taken from the caller's bounds `` |
| `crates/kivact-ui/src/calendar/navigation.rs:76` | SwipeSnapshot.at_min | 文言 | independent | P8-UI6 | writer: calendar/view.rs:424 のみ（表示月 <= min の月）。min と max が同じ月なら at_max と同時に true で両方向に抵抗（navigation.rs:91）、rtl とは無関係。4 組合せ全て到達可能。理由文が汎用句のみ。 | `` // xtask-allow(state-fields): independent: computed per event in swipe_navigation as "shown month <= the caller's min month"; it is true together with at_max when min and max fall in the shown month, which resists drags both ways `` |
| `crates/kivact-ui/src/calendar/navigation.rs:79` | SwipeSnapshot.at_max | 文言 | independent | P8-UI6 | writer: calendar/view.rs:425 のみ（表示月 >= max の月）。at_min と同時 true は min/max 同月で到達、rtl と無関係（navigation.rs:91）。理由文が汎用句のみ。 | `` // xtask-allow(state-fields): independent: computed per event in swipe_navigation as "shown month >= the caller's max month"; it is set apart from the reading direction and is true with at_min only when both bounds fall in the shown month `` |
| `crates/kivact-ui/src/calendar/view.rs:32` | Calendar.month | 文言 | option | P8-UI6 | writer: Calendar::default_month（view.rs:236）のみ。absent→選択の月か今日（view.rs:305）。displayed_month が Some だと無視される（view.rs:105,303-305）優先関係が理由文に無い。builder の authored 値で実行時変化なし。 | `` // xtask-allow(state-fields): option: Calendar::default_month; absent starts at the selection's month or today, and a caller month binding (displayed_month) takes precedence over it `` |
| `crates/kivact-ui/src/calendar/view.rs:38` | Calendar.week_start | 確認済み | option | — | writer: Calendar::week_start（view.rs:268）のみ。absent→locale の週始め（view.rs:485 で未指定時は policy 既定）。実行時の書換え無し。 |  |
| `crates/kivact-ui/src/calendar/view.rs:42` | Calendar.displayed_month | 文言 | option | P8-UI6 | writer: Calendar::month（view.rs:184）のみ。absent→calendar 自身の signal（default_month から開始、view.rs:305）。「component's default」は制御/非制御の意味を述べていない。 | `` // xtask-allow(state-fields): option: Calendar::month binds the shown month to the caller; absent keeps it in the calendar's own signal seeded from default_month `` |
| `crates/kivact-ui/src/carousel.rs:157` | Layout.per_view | 文言 | option | P8-UI6 | writer: prepare（carousel.rs:217）が Carousel::items_per_view（78）を複写するのみ。absent→1 枚表示、さらに scroll mode では slide を自然幅にする（carousel.rs:377）。既定値以上の意味があり理由文が不足。 | `` // xtask-allow(state-fields): option: copied from Carousel::items_per_view; absent shows one slide per view and leaves scroll-mode slides at their natural size `` |
| `crates/kivact-ui/src/carousel.rs:159` | Layout.gap | 確認済み | option | — | writer: prepare（carousel.rs:218）が Carousel::gap（86）を複写するのみ。absent→theme の carousel_gap（carousel.rs:172）。実行時の書換え無し。 |  |
| `crates/kivact-ui/src/carousel.rs:161` | Layout.padding | 確認済み | option | — | writer: prepare（carousel.rs:219）が Carousel::padding（91）を複写するのみ。absent→theme の carousel_padding（carousel.rs:166）。実行時の書換え無し。 |  |
| `crates/kivact-ui/src/carousel_geometry.rs:6` | PageRequest.vertical | 文言 | independent | P8-UI6 | writer: carousel.rs:232 navigate の literal のみ（orientation 由来）。rtl は vertical 時に無視（carousel_geometry.rs:34,76）だが入力として全組合せ有効。理由文が汎用句のみ。 | `` // xtask-allow(state-fields): independent: the carousel orientation read when a page step is requested; the caller sets it apart from reading direction, step sign and looping, and page() handles every pair `` |
| `crates/kivact-ui/src/carousel_geometry.rs:8` | PageRequest.rtl | 文言 | independent | P8-UI6 | writer: carousel.rs:233 のみ（use_direction）。page() は水平軸でだけ読む（carousel_geometry.rs:34,76）。vertical と同時 true は到達し単に無視される。理由文が汎用句のみ。 | `` // xtask-allow(state-fields): independent: the inherited reading direction at the request; page() reads it only on the horizontal axis, so a vertical request with rtl set is valid and ignores it `` |
| `crates/kivact-ui/src/carousel_geometry.rs:10` | PageRequest.next | 文言 | independent | P8-UI6 | writer: carousel.rs:234（delta > 0）のみ。キー/ボタンの向きで、軸・方向・looping のどれとも組める（carousel_geometry.rs:35-45）。理由文が汎用句のみ。 | `` // xtask-allow(state-fields): independent: the sign of the key or button step (delta > 0); either direction can be asked on any axis, in either reading direction, with or without looping `` |
| `crates/kivact-ui/src/carousel_geometry.rs:12` | PageRequest.looping | 閉じ方の変更 | option | P8-UI6 | writer: carousel.rs:235 が Layout.looping（= setup-only Carousel::looping、carousel.rs:57,205）を複写するのみ。caller の authored 設定で実行時変化なし→option が正しい。 | `` // xtask-allow(state-fields): option: copied from the setup-only Carousel::looping; false stops a page step at either end instead of wrapping `` |
| `crates/kivact-ui/src/code_block.rs:188` | CodeSource.highlight | 確認済み | option | — | writer: CodeBlock::highlight（code_block.rs:97）→ 450 で複写のみ。absent→plain_highlight（code_block.rs:384）。既定の挙動で文言どおり。 |  |
| `crates/kivact-ui/src/code_block.rs:192` | CodeSource.max_height | 文言 | option | P8-UI5 | writer: CodeBlock::max_height（147）→ 453 複写のみ。absent→高さ上限なし（code_block.rs:401-404）。「component's default」は無制限の意味を述べていない。 | `` // xtask-allow(state-fields): option: CodeBlock::max_height; absent leaves the editor without a height cap `` |
| `crates/kivact-ui/src/collection_values.rs:41` | ToolbarState.gap | 確認済み | option | — | writer: toolbar.rs:133 のみ（Toolbar::gap, toolbar.rs:72 の値）。absent→recipe が theme の toolbar.gap を使う（toolbar.rs:114）。公開 recipe state で文言どおり。 |  |
| `crates/kivact-ui/src/combobox/data.rs:22` | Combobox.query | 文言 | option | P8-UI6 | writer: Combobox::query（handle.rs:69）のみ。absent→behavior 内部の query（render.rs:566-568）。「component's default」は非制御の意味を述べていない。 | `` // xtask-allow(state-fields): option: Combobox::query binds the editor text to the caller; absent keeps the query inside the combobox behavior `` |
| `crates/kivact-ui/src/combobox/data.rs:24` | Combobox.open | 文言 | option | P8-UI6 | writer: Combobox::open（handle.rs:64）のみ。absent→local signal（render.rs:556-563）。mode=Inline では無視される（render.rs:558）。理由文に両方とも無い。 | `` // xtask-allow(state-fields): option: Combobox::open binds popup visibility to the caller; absent uses a local signal, and inline mode ignores it because its list always shows `` |
| `crates/kivact-ui/src/combobox/data.rs:29` | Combobox.mode | 確認済み | option | — | writer: Combobox::mode（handle.rs:79）のみ。absent→Popup（render.rs:557 unwrap_or）。文言どおり。 |  |
| `crates/kivact-ui/src/combobox/data.rs:33` | Combobox.placeholder | 確認済み | option | — | writer: Combobox::placeholder（handle.rs:89）のみ。absent→combobox.placeholder メッセージ（render.rs:42）。文言どおり。 |  |
| `crates/kivact-ui/src/combobox/data.rs:35` | Combobox.empty | 確認済み | option | — | writer: Combobox::empty_text（handle.rs:94）のみ。absent→combobox.empty メッセージ（render.rs:79）。文言どおり。 |  |
| `crates/kivact-ui/src/combobox/data.rs:38` | Combobox.matcher | 確認済み | option | — | writer: Combobox::filter（handle.rs:104）のみ。absent→大小無視の label 部分一致（render.rs:375-380）。既定 matcher で文言どおり。 |  |
| `crates/kivact-ui/src/combobox/data.rs:40` | Combobox.search | 文言 | option | P8-UI6 | writer: Combobox::search（handle.rs:114）のみ。absent→options をローカルで絞込み、present→非同期取得で matcher を使わない（render.rs:499,571,374）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: Combobox::search; absent filters the caller's options locally, present fetches suggestions and skips the matcher `` |
| `crates/kivact-ui/src/command_palette.rs:28` | CommandPalette.filter | 確認済み | option | — | writer: CommandPalette::filter（command_palette.rs:116）のみ。absent→filter_commands（doc 108-113, 167）。文言どおり。 |  |
| `crates/kivact-ui/src/command_palette.rs:30` | CommandPalette.close_on | 文言 | option | P8-UI6 | writer: CommandPalette::close_on（125）のみ。absent→Escape だけで閉じる（169-171）。「component's default」は追加キー無しの意味を述べていない。 | `` // xtask-allow(state-fields): option: CommandPalette::close_on; absent means only Escape closes the palette from the field `` |
| `crates/kivact-ui/src/command_palette.rs:32` | CommandPalette.placeholder | 確認済み | option | — | writer: CommandPalette::placeholder（132）のみ。absent→palette.placeholder メッセージ（173）。文言どおり。 |  |
| `crates/kivact-ui/src/command_palette.rs:34` | CommandPalette.empty | 確認済み | option | — | writer: CommandPalette::empty_text（139）のみ。absent→palette.empty メッセージ（176）。文言どおり。 |  |
| `crates/kivact-ui/src/command_palette.rs:36` | CommandPalette.footer | 文言 | option | P8-UI6 | writer: CommandPalette::footer（147）のみ。absent→footer を描かない（200-211）。「component's default」は不在の意味を述べていない。 | `` // xtask-allow(state-fields): option: CommandPalette::footer; absent draws no footer under the command list `` |
| `crates/kivact-ui/src/data_table/filter_value.rs:23` | FilterField::Number.min | 確認済み | option | — | 公開 enum を app が構築して与える authored 値。crate 内の書き手なし、読み手は accepts（filter_value.rs:47-50）のみ。absent→下限なしで文言どおり。 |  |
| `crates/kivact-ui/src/data_table/filter_value.rs:25` | FilterField::Number.max | 確認済み | option | — | 同上。読み手 accepts（filter_value.rs:49-50）のみ。absent→上限なしで文言どおり。 |  |
| `crates/kivact-ui/src/data_table/rows.rs:71` | Grid.tree | 文言 | option | P8-UI5 | writer: grid_plan（runtime.rs:496）が ModelPlan.tree（DataTable::tree 由来）を複写するのみ。absent→平坦な行で展開制御なし。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the grid's tree plan, present when DataTable::tree was given; absent draws rows without expand controls `` |
| `crates/kivact-ui/src/data_table/rows.rs:77` | Grid.filters | 文言 | option | P8-UI5 | writer: runtime.rs:501-507 のみ（DataTable::typed_column_filters の map）。absent→header に typed filter を置かない。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the grid's typed filter editors, present when DataTable::typed_column_filters was given; absent leaves the headers without them `` |
| `crates/kivact-ui/src/data_table/rows.rs:81` | Grid.indexed | 文言 | option | P8-UI5 | writer: runtime.rs:509 row_indexes（DataTable::row_index の map, runtime.rs:461）のみ。absent→番号 gutter なし。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the grid's row numbers, built when DataTable::row_index was given; absent draws no number gutter `` |
| `crates/kivact-ui/src/data_table/rows.rs:86` | Grid.selection | 文言 | option | P8-UI5 | writer: runtime.rs:513（DataTable::selection 複写）のみ。absent→check box なし。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the grid's copy of the caller's DataTable::selection binding; absent draws no row check boxes `` |
| `crates/kivact-ui/src/data_table/rows.rs:88` | Grid.row_selectable | 確認済み | option | — | writer: runtime.rs:510（DataTable::row_selectable の take）のみ。absent→全行選択可で既定どおり。 |  |
| `crates/kivact-ui/src/data_table/rows.rs:90` | Grid.sticky | 文言 | option | P8-UI5 | writer: runtime.rs:491-494,514（DataTable::sticky_columns 由来）のみ。absent→固定列なし。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the grid's pinned-column measurements, present when DataTable::sticky_columns named a column; absent pins none `` |
| `crates/kivact-ui/src/data_table/rows.rs:92` | Grid.widths | 文言 | option | P8-UI5 | writer: runtime.rs:515（DataTable::resizable 複写）のみ。absent→resize handle なし。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the grid's copy of the DataTable::resizable widths; absent draws no resize handles `` |
| `crates/kivact-ui/src/data_table/rows.rs:94` | Grid.actions | 文言 | option | P8-UI5 | writer: runtime.rs:516（DataTable::row_actions の take）のみ。absent→actions 列なし。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the grid's row actions column from DataTable::row_actions; absent adds no actions column `` |
| `crates/kivact-ui/src/data_table/rows.rs:96` | Grid.row_context_menu | 文言 | option | P8-UI5 | writer: runtime.rs:517（take）のみ。absent→副クリックで menu を開かない。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the grid's menu from DataTable::row_context_menu; absent opens no menu on a row's secondary click `` |
| `crates/kivact-ui/src/data_table/rows.rs:99` | Grid.cells | 確認済み | option | — | writer: runtime.rs:519（DataTable::cell_controls の take）のみ。absent→列自身の cell view で既定どおり。 |  |
| `crates/kivact-ui/src/data_table/rows.rs:101` | Grid.details | 文言 | option | P8-UI5 | writer: runtime.rs:520（take）のみ。absent→行下に何も置かない（rows.rs:661-667）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the grid's row details view from DataTable::row_details; absent puts nothing under the rows `` |
| `crates/kivact-ui/src/data_table/rows.rs:103` | Grid.expanded_rows | 文言 | option | P8-UI5 | writer: runtime.rs:521 のみ。details が None なら無視（rows.rs:415,661-667）、details 有で absent→常時表示。依存関係が理由文に無い。 | `` // xtask-allow(state-fields): option: the grid's copy of DataTable::expanded_rows; absent shows details always, and without details it adds no disclosure `` |
| `crates/kivact-ui/src/data_table/rows.rs:110` | Grid.row_height | 確認済み | option | — | writer: runtime.rs:526 のみ。absent→row recipe の theme 高さ（rows.rs:693 で None を渡す）。既定どおり。 |  |
| `crates/kivact-ui/src/data_table/rows.rs:112` | Grid.row_status | 文言 | option | P8-UI5 | writer: runtime.rs:527（take）のみ。absent→status lane なし（rows.rs:441,1033）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the grid's status resolver from DataTable::row_status; absent reserves no status lane `` |
| `crates/kivact-ui/src/data_table/rows.rs:114` | Grid.row_color | 文言 | option | P8-UI5 | writer: runtime.rs:528（take）のみ。absent→行の色なし（rows.rs:692）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the grid's color function from DataTable::row_color; absent gives rows no edge color `` |
| `crates/kivact-ui/src/data_table/runtime.rs:34` | ModelPlan.tree | 文言 | option | P8-UI5 | writer: model_plan（runtime.rs:126-141）のみ（DataTable::tree 由来）。absent→tree 配置なし。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the tree arrangement made from DataTable::tree; absent keeps the model's rows flat in sorted order `` |
| `crates/kivact-ui/src/data_table/runtime.rs:36` | ModelPlan.page | 誤（付録の設計） | type（PagingPlan に集約） | P8-UI5 | paging()（runtime.rs:176-192）で page_size = table.page.and_then(..) のため page_size=Some ⇒ page=Some（SM-2 含意）。(None,Some) は到達不能。PagingPlan{page,size_choice} を Option で持つ。詳細は G2.wrong.md W1。 | `` // xtask-allow(state-fields): option: DataTable::page or remote_page with any page_sizes choice; absent shows every row with no pager `` |
| `crates/kivact-ui/src/data_table/runtime.rs:38` | ModelPlan.page_size | 誤（付録の設計） | type（PagingPlan.size_choice へ移動） | P8-UI5 | page が Some のときだけ Some（runtime.rs:180-186）。読み手は table_paging（runtime.rs:205）のみ。PagingPlan 内の唯一の数える field になり marker 不要。W1。 （field 削除。PagingPlan.size_choice は owner 内で 1 個のため marker 不要） |  |
| `crates/kivact-ui/src/data_table/runtime.rs:40` | ModelPlan.groups | 誤（付録の設計） | type（GroupingPlan に集約） | P8-UI5 | groups と grouping_key は共に table.grouping から作られ（runtime.rs:157-161）常に同時に Some/None（SM-3）。Option<GroupingPlan{groups,key}> にまとめる。W2。 | `` // xtask-allow(state-fields): option: DataTable::group_by, as the page's groups and their key together; absent shows the page's rows ungrouped `` |
| `crates/kivact-ui/src/data_table/runtime.rs:45` | ModelPlan.grouping_key | 誤（付録の設計） | type（GroupingPlan.key へ移動） | P8-UI5 | groups と常に同時 Some/None（runtime.rs:157-161）。読み手 runtime.rs:291 のみ。W2。 （field 削除。GroupingPlan.key は Option でないため marker 不要） |  |
| `crates/kivact-ui/src/data_table/types.rs:126` | DataTable.grouping | 文言 | option | P8-UI5 | writer: DataTable::group_by（builder.rs:151）のみ。absent→group header なし。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: DataTable::group_by key and collapsed binding; absent shows each page's rows without group headers `` |
| `crates/kivact-ui/src/data_table/types.rs:136` | DataTable.column_order | 確認済み | option | — | writer: DataTable::column_order（builder.rs:254）のみ。absent→宣言順（runtime.rs:214-233）。既定どおり。 |  |
| `crates/kivact-ui/src/data_table/types.rs:152` | DataTable.row_index | 文言 | option | P8-UI5 | writer: DataTable::row_index（builder.rs:344）のみ。absent→番号 gutter なし（runtime.rs:265,461）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: DataTable::row_index start number; absent draws no row number gutter `` |
| `crates/kivact-ui/src/data_table/types.rs:154` | DataTable.filter | 文言 | option | P8-UI5 | writer: DataTable::filter（builder.rs:352）のみ。absent→filter 欄なし・全行（runtime.rs:142）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the caller's DataTable::filter query; absent shows no filter field and keeps every row `` |
| `crates/kivact-ui/src/data_table/types.rs:156` | DataTable.column_filters | 文言 | option | P8-UI5 | writer: DataTable::column_filters（builder.rs:363）のみ。absent→列ごとの query なし（runtime.rs:145）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the caller's DataTable::column_filters queries; absent applies no per-column text query `` |
| `crates/kivact-ui/src/data_table/types.rs:158` | DataTable.typed_column_filters | 文言 | option | P8-UI5 | writer: builder.rs:305 のみ。absent→typed filter 編集なし（runtime.rs:148,501）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the caller's DataTable::typed_column_filters values; absent shows no typed filter editors `` |
| `crates/kivact-ui/src/data_table/types.rs:161` | DataTable.sticky | 文言 | option | P8-UI5 | writer: DataTable::sticky_columns（builder.rs:215、両側空なら None）のみ。absent→固定列なし。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: DataTable::sticky_columns, None when both edge lists are empty; absent pins no column `` |
| `crates/kivact-ui/src/data_table/types.rs:163` | DataTable.widths | 文言 | option | P8-UI5 | writer: DataTable::resizable（builder.rs:374）のみ。absent→resize handle なし（runtime.rs:308）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the caller's DataTable::resizable widths; absent gives the headers no resize handle `` |
| `crates/kivact-ui/src/data_table/types.rs:165` | DataTable.resize_range | 文言 | option | P8-UI5 | writer: builder.rs:381 のみ。absent→18..2048（runtime.rs:309-311）。widths が None なら読まれない依存が理由文に無い。 | `` // xtask-allow(state-fields): option: DataTable::resize_range; absent limits resizing to 18..2048 px, and it is unused without resizable `` |
| `crates/kivact-ui/src/data_table/types.rs:167` | DataTable.row_label | 確認済み | option | — | writer: builder.rs:389 のみ。absent→先頭列の text（runtime.rs:389-395）。既定どおり。 |  |
| `crates/kivact-ui/src/data_table/types.rs:169` | DataTable.actions | 文言 | option | P8-UI5 | writer: DataTable::row_actions（builder.rs:404）のみ。absent→actions 列なし（runtime.rs:279）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: DataTable::row_actions width and view; absent adds no last actions column `` |
| `crates/kivact-ui/src/data_table/types.rs:171` | DataTable.row_context_menu | 文言 | option | P8-UI5 | writer: builder.rs:236 のみ。absent→副クリック menu なし。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: DataTable::row_context_menu; absent leaves a row's secondary click without a menu `` |
| `crates/kivact-ui/src/data_table/types.rs:174` | DataTable.toolbar | 確認済み | option | — | writer: builder.rs:429 のみ。absent→既定 toolbar（runtime.rs:73-76）。既定どおり。 |  |
| `crates/kivact-ui/src/data_table/types.rs:176` | DataTable.footer | 確認済み | option | — | writer: builder.rs:439 のみ。absent→既定 footer（selection 要約と pager、runtime.rs:77-86）。既定どおり。 |  |
| `crates/kivact-ui/src/data_table/types.rs:179` | DataTable.details | 文言 | option | P8-UI5 | writer: DataTable::row_details（builder.rs:454）のみ。absent→行下 view なし。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: DataTable::row_details; absent builds no view under each row `` |
| `crates/kivact-ui/src/data_table/types.rs:181` | DataTable.expanded_rows | 文言 | option | P8-UI5 | writer: builder.rs:462 のみ。details 無しでは無視（runtime.rs:274, rows.rs:661-667）、details 有で absent→常時表示。依存が理由文に無い。authored 設定の依存で実行時状態ではないため WRONG にしない。 | `` // xtask-allow(state-fields): option: the caller's DataTable::expanded_rows keys; absent shows row details always with no disclosure, and it is unused without row_details `` |
| `crates/kivact-ui/src/data_table/types.rs:183` | DataTable.cells | 確認済み | option | — | writer: DataTable::cell_controls（builder.rs:477）のみ。absent→列自身の cell view。既定どおり。 |  |
| `crates/kivact-ui/src/data_table/types.rs:186` | DataTable.row_height | 確認済み | option | — | writer: builder.rs:497 のみ。absent→row recipe の theme 高さ（rows.rs:693）。既定どおり。 |  |
| `crates/kivact-ui/src/data_table/types.rs:188` | DataTable.row_status | 文言 | option | P8-UI5 | writer: builder.rs:507 のみ。absent→status lane なし（runtime.rs:260）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: DataTable::row_status; absent reserves no leading status lane `` |
| `crates/kivact-ui/src/data_table/types.rs:190` | DataTable.row_color | 文言 | option | P8-UI5 | writer: builder.rs:514 のみ。absent→行端の色なし。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: DataTable::row_color; absent draws no row edge color `` |
| `crates/kivact-ui/src/data_table_tokens.rs:45` | DataTableStickyPaint.shadow_transition | 文言 | option | P8-UI6 | writer: theme 作者（kivact-ui-soft/src/theme.rs:263）と Default（None）。absent→transition 無しで即時（rows.rs:341-345）。「theme's default」は事実と違う。 | `` // xtask-allow(state-fields): option: a theme author's tween for the pinned-column shadow; None switches the shadow without animation `` |
| `crates/kivact-ui/src/data_table_tokens.rs:47` | DataTableStickyPaint.overlay_transition | 文言 | option | P8-UI6 | writer: theme 作者（kivact-ui-soft/src/theme.rs:267）と Default。absent→hover/selected overlay が即時に変わる（sticky_paint.rs:17）。「theme's default」は事実と違う。 | `` // xtask-allow(state-fields): option: a theme author's tween for the pinned cells' hover overlay; None changes the overlay without animation `` |
| `crates/kivact-ui/src/date_input/machine.rs:84` | Snapshot.min | 文言 | option | P8-UI6 | writer: date_input/view.rs:353（caller の min Value を毎 event 読む）のみ。absent→下限なし（machine.rs:92）。「component's default」は事実と違う。 | `` // xtask-allow(state-fields): option: the caller's DateInput min bound read for each event; absent accepts any early date `` |
| `crates/kivact-ui/src/date_input/machine.rs:86` | Snapshot.max | 文言 | option | P8-UI6 | writer: date_input/view.rs:354 のみ。absent→上限なし（machine.rs:92）。min>max も caller 入力として有効。 | `` // xtask-allow(state-fields): option: the caller's DateInput max bound read for each event; absent accepts any late date `` |
| `crates/kivact-ui/src/date_input/view.rs:416` | EditorInputs.custom_format | 文言 | option | P8-UI5 | writer: DateInput::format_with（view.rs:99、format() が None に戻す 94）→ 196 複写のみ。absent→DateStyle 書式。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: DateInput::format_with; absent formats with the DateStyle set by DateInput::format `` |
| `crates/kivact-ui/src/date_input/view.rs:418` | EditorInputs.external_invalid | 文言 | option | P8-UI5 | writer: view.rs:174（root.accessible_invalid＝caller の invalid 指定）→197 のみ。absent→typed draft の検証だけ（view.rs:449）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: the caller's invalid flag on the DateInput node; absent leaves validity to the typed draft alone `` |
| `crates/kivact-ui/src/date_range_draft.rs:18` | Draft::Editing.start | 文言 | independent | P8-UI6 | writer: Policy::transition Event::Start（date_range_draft.rs:111-114、送信元 date_range_input.rs:450）のみ。end と別 editor から来るので 4 組合せ到達、start>end は order_invalid で報告（:51-54）。理由文が組合せを述べない。 | `` // xtask-allow(state-fields): independent: the last date the start DateInput published, None once it is cleared; the two editors publish separately, so any pair with end is a draft, and start after end shows order_invalid `` |
| `crates/kivact-ui/src/date_range_draft.rs:20` | Draft::Editing.end | 文言 | independent | P8-UI6 | writer: Event::End（date_range_draft.rs:116-119、送信元 date_range_input.rs:452）のみ。start と独立に到達、end<start は order_invalid。理由文が組合せを述べない。 | `` // xtask-allow(state-fields): independent: the last date the end DateInput published, None once it is cleared; it is set apart from start, and an end before start is kept and reported by order_invalid `` |
| `crates/kivact-ui/src/date_time_sheet.rs:13` | Inputs.picker_label | 確認済み | option | — | writer: date_time_input.rs:97（DateTimeInput::picker_label, :45 の複写）のみ。absent→date_time.open メッセージ（date_time_sheet.rs:46-47）。既定どおり。 |  |
| `crates/kivact-ui/src/date_time_sheet.rs:19` | Inputs.week_start | 確認済み | option | — | writer: date_time_input.rs:102（:71 の複写）のみ。absent→locale の週始め（date_time_sheet.rs:84-85）。既定どおり。 |  |
| `crates/kivact-ui/src/dialog.rs:38` | Dialog.align | 確認済み | option | — | writer: Dialog::align（dialog.rs:239）のみ。absent→overlay の既定（start）、Popover 以外は無視（doc 235-238, 503-506）。既定どおり。 |  |
| `crates/kivact-ui/src/dialog.rs:40` | Dialog.width | 確認済み | option | — | writer: Dialog::width（dialog.rs:175）のみ。absent→theme recipe 幅（doc 172, 466-468）。既定どおり。 |  |
| `crates/kivact-ui/src/dialog.rs:42` | Dialog.description | 文言 | option | P8-UI6 | writer: Dialog::description（dialog.rs:244）のみ。absent→説明を描かず関連付けもしない（dialog.rs:337）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: Dialog::description; absent draws no description and links none to the dialog `` |
| `crates/kivact-ui/src/dialog.rs:44` | Dialog.initial_focus | 確認済み | option | — | writer: Dialog::initial_focus（dialog.rs:248）のみ。absent→use_dialog の既定 focus（dialog.rs:275-277）。既定どおり。 |  |
| `crates/kivact-ui/src/dialog.rs:46` | Dialog.return_focus | 確認済み | option | — | writer: pub(crate) Dialog::return_focus（dialog.rs:105、呼出し date_input/view.rs:215, date_time_sheet.rs:57, time_input/view.rs:134）のみ。absent→us… |  |
| `crates/kivact-ui/src/dialog.rs:48` | Dialog.dismiss | 確認済み | option | — | writer: Dialog::dismiss（dialog.rs:252）のみ。absent→use_dialog の既定 dismiss（dialog.rs:278-280）。既定どおり。 |  |
| `crates/kivact-ui/src/group_types.rs:35` | (no audit row: pub rtl: bool,) | 文言 | independent | P8-UI6 | writer: button_group.rs:84,108,133（use_direction）のみ。数える sibling なし（audit 行なし）。理由文の「derived position or construction input」は事実と違い（方向は継承値）、組合せ理由も無い。（owner は ButtonGroupState.rtl） | `` // xtask-allow(state-fields): independent: the inherited reading direction at render; the caller's orientation and size are set apart from it, so a group of either axis and size may read right to left `` |
| `crates/kivact-ui/src/group_types.rs:43` | ButtonGroupMemberState.first | 文言 | independent | P8-UI6 | writer: button_group.rs:86（index == 0）のみ。子 1 個で first=last=true、中間は両 false、4 組合せ到達。理由文が汎用句のみ。 | `` // xtask-allow(state-fields): independent: index == 0 in ButtonGroup's member memo; a lone child is both first and last and a middle child neither, so all four pairs occur `` |
| `crates/kivact-ui/src/group_types.rs:45` | ButtonGroupMemberState.last | 文言 | independent | P8-UI6 | writer: button_group.rs:87（index + 1 == count）のみ。first と全組合せ到達。理由文が汎用句のみ。 | `` // xtask-allow(state-fields): independent: index + 1 == count in ButtonGroup's member memo; with first it marks a lone child (both), the two ends (one each) and middle children (neither) `` |
| `crates/kivact-ui/src/group_types.rs:65` | ToggleGroupState.spacing | 確認済み | option | — | writer: toggle_group.rs:183,204（caller の spacing を max(0) で投影）のみ。absent→theme の ToggleGroupTokens.gap。公開 recipe state で既定どおり。 |  |
| `crates/kivact-ui/src/group_types.rs:67` | ToggleGroupState.rtl | 文言 | independent | P8-UI6 | writer: toggle_group.rs:184,205（use_direction）のみ。spacing と無関係で全組合せ到達。理由文が事実と違い（位置でも構築入力でもない）汎用句のみ。 | `` // xtask-allow(state-fields): independent: the inherited reading direction at render; the caller's spacing is set apart from it, so either direction pairs with a set or default spacing `` |
| `crates/kivact-ui/src/group_types.rs:76` | ToggleGroupItemState.first | 文言 | independent | P8-UI6 | writer: toggle_group.rs:186-187（options 先頭の key と一致）のみ。option 1 個なら first=last=true、4 組合せ到達。理由文が汎用句のみ。 | `` // xtask-allow(state-fields): independent: whether this toggle's key is the first of the caller's options; a lone option is both first and last and a middle one neither, so all four pairs occur `` |
| `crates/kivact-ui/src/group_types.rs:78` | ToggleGroupItemState.last | 文言 | independent | P8-UI6 | writer: toggle_group.rs:188-189（options 末尾の key と一致）のみ。first と全組合せ到達。理由文が汎用句のみ。 | `` // xtask-allow(state-fields): independent: whether this toggle's key is the last of the caller's options; with first it marks a lone option (both), the two ends (one each) and middle ones (neither) `` |
| `crates/kivact-ui/src/hover_card.rs:43` | (no audit row: label: Option<Value<String>>,) | 文言 | option | P8-UI6 | writer: HoverCard::label（hover_card.rs:115）のみ（audit 行なし）。absent→Group role＋hover_card.label 名、present→Dialog role（hover_card.rs:182-187）。role も変わる点が理由文に無い。（owner は HoverCard.label） | `` // xtask-allow(state-fields): option: HoverCard::label; absent keeps the preview a Group named by the hover_card.label message, present makes it a named Dialog `` |
| `crates/kivact-ui/src/item_row.rs:18` | RowContents.title_content | 確認済み | option | — | writer: ItemRow::title_content（item_row.rs:83）のみ。absent→title text を title_lines 付きで描く（item_row.rs:256-262）。既定どおり。 |  |
| `crates/kivact-ui/src/item_row.rs:20` | RowContents.description | 文言 | option | P8-UI6 | writer: ItemRow::description（item_row.rs:79）のみ。absent→description_content が無ければ説明なし（item_row.rs:265-275）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: ItemRow::description text; absent shows no description unless description_content gives one `` |
| `crates/kivact-ui/src/item_row.rs:22` | RowContents.description_content | 文言 | option | P8-UI6 | writer: ItemRow::description_content（item_row.rs:87）のみ。present なら description text より優先（item_row.rs:265-269）。優先関係が理由文に無い。 | `` // xtask-allow(state-fields): option: ItemRow::description_content; present it replaces the plain description text, absent falls back to that text `` |
| `crates/kivact-ui/src/item_row.rs:24` | RowContents.media | 文言 | option | P8-UI6 | writer: ItemRow::media（item_row.rs:68）のみ。absent→media slot なし（item_row.rs:225-227）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: ItemRow::media; absent leaves the media slot out of the row body `` |
| `crates/kivact-ui/src/item_row.rs:26` | RowContents.actions | 文言 | option | P8-UI6 | writer: ItemRow::actions（item_row.rs:72）のみ。absent→actions slot なし（item_row.rs:228-230）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: ItemRow::actions; absent leaves the trailing actions slot out of the row body `` |
| `crates/kivact-ui/src/item_row.rs:28` | RowContents.header | 文言 | option | P8-UI6 | writer: ItemRow::header（item_row.rs:91）のみ。absent→本体上に何も描かない（item_row.rs:231-233）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: ItemRow::header; absent draws nothing above the row body `` |
| `crates/kivact-ui/src/item_row.rs:30` | RowContents.footer | 文言 | option | P8-UI6 | writer: ItemRow::footer（item_row.rs:95）のみ。absent→本体下に何も描かない（item_row.rs:234-236）。理由文が意味を述べていない。 | `` // xtask-allow(state-fields): option: ItemRow::footer; absent draws nothing below the row body `` |
| `crates/kivact-ui/src/item_row.rs:32` | RowContents.marker | 確認済み | option | — | writer: ItemRow::marker（item_row.rs:100）のみ。absent→list が生成する marker（doc 98, render_marker）。既定どおり。 |  |
| `crates/kivact-ui/src/item_row.rs:35` | RowContents.delegate | 文言 | option | P8-UI6 | writer: ItemRow::delegate_to（item_row.rs:109）のみ。present なら press より優先（item_row.rs:195-199）。優先関係と absent の意味が理由文に無い。 | `` // xtask-allow(state-fields): option: ItemRow::delegate_to; absent lets the row itself take on_press, present forwards activation to the target instead `` |
| `crates/kivact-ui/src/item_row.rs:37` | RowContents.position | 確認済み | option | — | writer: ItemRow::position（item_row.rs:114）のみ。absent→固定子の context position（item_row.rs:178-185）。既定どおり。 |  |
| `crates/kivact-ui/src/item_row.rs:39` | RowContents.title_lines | 確認済み | option | — | writer: ItemRow::title_lines（item_row.rs:119）のみ。absent→theme の item_list.title_lines（item_row.rs:211）。既定どおり。 |  |
| `crates/kivact-ui/src/item_row.rs:41` | RowContents.description_lines | 確認済み | option | — | writer: ItemRow::description_lines（item_row.rs:124）のみ。absent→theme の item_list.description_lines（item_row.rs:212-213）。既定どおり。 |  |
| `crates/kivact-ui/src/menu.rs:37` | Menu.offset | 確認済み | option | — | writer: Menu::offset（menu.rs:103）のみ。absent→Anchored 4.0 / Context 0.0（menu.rs:222-226）。既定どおり。 |  |
| `crates/kivact-ui/src/menu.rs:40` | Menu.align | 確認済み | option | — | writer: Menu::align（menu.rs:116）のみ。absent→overlay 既定（menu.rs:425-428）。既定どおり。 |  |
| `crates/kivact-ui/src/menu.rs:44` | Menu.item | 確認済み | option | — | writer: Menu::item（menu.rs:148）のみ。absent→既定の行内容。既定どおり。 |  |
| `crates/kivact-ui/src/menu.rs:276` | Level.align | 確認済み | option | — | writer: menu.rs:217（top level は Menu::align 複写）と 550（submenu は None）のみ。absent→overlay 既定（menu.rs:425-428）。実行時の書換え無し。 |  |
| `crates/kivact-ui/src/navigation_values.rs:23` | StepperItemState.interactive | 文言 | independent | P8-UI6 | writer: stepper.rs:314,322（activate.is_some()、全 step 共通）のみ。disabled/optional/collapsed と全組合せ到達、disabled の press は stepper.rs:348-351 が拒否。理由文が sibling（status は数えない）との組合せを述べない。 | `` // xtask-allow(state-fields): independent: whether Stepper::on_activate was given, the same for every step; a disabled, optional or collapsed step keeps it, and disabled alone refuses the press `` |
| `crates/kivact-ui/src/pagination.rs:484` | ControlContent.summary | 文言 | option | P8-UI6 | writer: render_pagination（pagination.rs:744、render_summary 421-443）のみ。caller の summary slot か Count/Compact mode で Some。input と同時 Some も到達し両方描く（587）。理由文が意味を述べない。 | `` // xtask-allow(state-fields): option: the summary built from the caller's summary slot, or the page count in Count and Compact modes; absent places no summary between the controls `` |
| `crates/kivact-ui/src/pagination.rs:486` | ControlContent.input | 文言 | option | P8-UI6 | writer: pagination.rs:723-729,745 のみ（mode == Input の時だけ Some）。caller の mode 設定の投影で実行時変化なし。理由文が意味を述べない。 | `` // xtask-allow(state-fields): option: the page field built only for PaginationMode::Input; absent in every other mode, and it may sit beside a caller summary `` |
| `crates/kivact-ui/src/pagination.rs:695` | PageExtras.summary | 確認済み | option | — | writer: pagination.rs:216（Pagination::summary, :122 の複写）のみ。absent→mode 既定の要約（render_summary 429-433）。既定どおり。 |  |
| `crates/kivact-ui/src/pagination.rs:697` | PageExtras.page_size | 文言 | option | P8-UI6 | writer: pagination.rs:217（Pagination::page_size, :84 の複写）のみ。absent→size select なし、件数は 1 件/頁で数える（pagination.rs:352,417,730）。理由文が意味を述べない。 | `` // xtask-allow(state-fields): option: the caller's Pagination::page_size binding; absent shows no size select and counts one item per page `` |
| `crates/kivact-ui/src/selection_field.rs:51` | FieldMachine.bounds | 文言 | independent | P8-UI6 | writer: selection_field.rs:190 の snapshot closure のみ（layout 計測）。disabled と 4 組合せ到達（初回 layout 前でも caller は disabled 可）。「separately from」は禁止された「別の field」型の理由。 | `` // xtask-allow(state-fields): independent: the trigger rect read from layout for each event, None until it is first laid out; a disabled trigger still has bounds and an unmeasured one may be enabled `` |
| `crates/kivact-ui/src/selection_field.rs:53` | FieldMachine.disabled | 文言 | independent | P8-UI6 | writer: selection_field.rs:191（caller の disabled を毎 event 読む）のみ。bounds 有無と無関係に true/false。理由文が「別の field」型。 | `` // xtask-allow(state-fields): independent: the caller's disabled flag read for each event; it is set apart from layout, so it is true or false whether or not the trigger has been measured `` |
| `crates/kivact-ui/src/time_input/machine.rs:37` | Snapshot.seconds | 閉じ方の変更 | option | P8-UI6 | writer: time_input/view.rs:194（caller の format の seconds flag、view.rs:54）のみ。caller が与える設定値なので option。min/max と全組合せ有効（parse で秒 0、machine.rs:92）。 | `` // xtask-allow(state-fields): option: the seconds flag of the caller's TimeInput format, read per event; false parses HH:MM with seconds left at zero `` |
| `crates/kivact-ui/src/time_input/machine.rs:39` | Snapshot.min | 文言 | option | P8-UI6 | writer: time_input/view.rs:195（caller の min を毎 event 読む）のみ。absent→下限なし。「component's default」は事実と違う。 | `` // xtask-allow(state-fields): option: the caller's TimeInput min bound read for each event; absent accepts any early time `` |
| `crates/kivact-ui/src/time_input/machine.rs:41` | Snapshot.max | 文言 | option | P8-UI6 | writer: time_input/view.rs:196 のみ。absent→上限なし。「component's default」は事実と違う。 | `` // xtask-allow(state-fields): option: the caller's TimeInput max bound read for each event; absent accepts any late time `` |

### B.3 G3

| marker（HEAD） | owner.field | 判定 | 閉じ方 | unit | 根拠 | 新しい marker 行 |
|---|---|---|---|---|---|---|
| `crates/kivact-engine/src/a11y/state.rs:219` | SemanticsIdAllocator.next | 確認済み | identity | — | 書き手は SemanticsIdAllocator::next（state.rs:228）のみ、u64::MAX から減算する単純な id 割当て。phase を隠さない。 |  |
| `crates/kivact-engine/src/a11y/state.rs:221` | SemanticsIdAllocator.empty_root | 確認済み | identity | — | 書き手は build_empty_semantics（a11y/build/dirty.rs:158-162）のみ。空 root 用 id を一度割当て、tree の node と衝突したときだけ取り直す。走行状態ではない。 |  |
| `crates/kivact-engine/src/check/boxes.rs:69` | Axes.x | 文言 | independent | P8-EN | 書き手は Axes::held_by（boxes.rs:78-84）のみ：x=!scroll_x&&!input、y=!scroll_y。overflow-x/y は別々に指定されるので4組とも到達。理由が「every combination is a valid limit」で「なぜ」を述べていない。 | `` // xtask-allow(state-fields): independent: Axes::held_by clears x when the node scrolls horizontally or is a text input, y when it scrolls vertically; overflow-x and overflow-y are authored apart, so all four pairs occur `` |
| `crates/kivact-engine/src/check/boxes.rs:71` | Axes.y | 文言 | independent | P8-EN | 同上（boxes.rs:78-84）。y は縦 scroll だけで決まり x とは別入力。理由文が定型句のみ。 | `` // xtask-allow(state-fields): independent: Axes::held_by clears y when the node scrolls vertically, x for horizontal scrolling or a text input; overflow-x and overflow-y are authored apart, so all four pairs occur `` |
| `crates/kivact-engine/src/contract/devtools.rs:106` | EngineEmulation.theme_hint | 確認済み | option | — | 書き手は DevTools の set_emulation（kivact-core devtools/emulation.rs:34-41）と clear（:51 default）のみ。利用者が与える設定値。読み手 host/devtools.rs:25-28。 |  |
| `crates/kivact-engine/src/contract/devtools.rs:108` | EngineEmulation.scale | 確認済み | option | — | 同上（emulation.rs:39 params.scale）。None で window 本来の scale（host/devtools.rs:29, emulated_viewport）。 |  |
| `crates/kivact-engine/src/contract/devtools.rs:110` | EngineEmulation.touch | 確認済み | option | — | 同上（emulation.rs:40 params.touch）。None で window 本来の pointer class（host/devtools.rs:32-37）。 |  |
| `crates/kivact-engine/src/contract/host.rs:107` | HostRequest::DrawableExport.scale | 確認済み | option | — | 唯一の構築は kivact-core view/drawable/requests.rs:57-62、値は app が開いた export の open.scale。None で window の scale（state.rs:38 scale_at）。 |  |
| `crates/kivact-engine/src/contract/host.rs:116` | HostRequest::SetDrawableGeometry.geometry | 確認済み | option | — | 構築は requests.rs:65-69 のみ、値は app API set_geometry(Option<Matrix4>)（drawable_share.rs:407）。None で非表示（request.rs:32-38）。 |  |
| `crates/kivact-engine/src/contract/node.rs:148` | BoxData.intrinsic | 確認済み | option | — | 書き手は props 表の Intrinsic setter（host/write/typed.rs:372-378）のみ＝app の authored 値。読み手 tree/node.rs:250, measure/leaf.rs:401。 |  |
| `crates/kivact-engine/src/contract/node.rs:159` | EditingData.behavior | 確認済み | option | — | 書き手は mount/editing.rs:36-40（app の behavior 関数）のみ。None で native 既定の編集（model/keys.rs:347,562）。 |  |
| `crates/kivact-engine/src/contract/node.rs:162` | EditingData.colors | 確認済み | option | — | 書き手は LineColors setter（typed.rs:289-291）のみ、app の highlight 束縛。None で着色なし＝既定（input_paint.rs:167）。 |  |
| `crates/kivact-engine/src/contract/node.rs:165` | EditingData.decorations | 確認済み | option | — | 書き手は Decorations setter（typed.rs:293-295）のみ＝app props。None で装飾なし（input_paint.rs:241）。 |  |
| `crates/kivact-engine/src/contract/node.rs:168` | EditingData.forward_keys | 確認済み | option | — | 書き手は ForwardKeys setter（typed.rs:313-315）のみ、元は custom/text_editing.rs:87。None で転送キーなし（keys.rs:489）。 |  |
| `crates/kivact-engine/src/contract/node.rs:171` | EditingData.value | 文言 | option | P8-EN | 書き手は Model::controlled_value（model/edit/inputs.rs:97）のみ＝app の InputValue 書込み。None は「editor 既定」ではなく最初の制御値前の空文書（inputs.rs:55, apply.rs:173 unwrap_or("")）。 | `` // xtask-allow(state-fields): option: the controlled value the app writes (InputValue, Model::controlled_value); absent until the first write, when the editor starts from empty text `` |
| `crates/kivact-engine/src/contract/node.rs:174` | EditingData.purpose | 確認済み | option | — | 書き手は InputPurpose setter（typed.rs:297-299）のみ＝app props（text_editing.rs:247）。None で通常テキスト。 |  |
| `crates/kivact-engine/src/contract/node.rs:177` | EditingData.read_only | 確認済み | option | — | 書き手は ReadOnly setter（typed.rs:301-303）のみ（text_editing.rs:252）。None は編集可（typed.rs:63）。 |  |
| `crates/kivact-engine/src/contract/node.rs:180` | EditingData.max_length | 確認済み | option | — | 書き手は MaxLength setter（typed.rs:305-307）のみ。None で無制限（tree/node.rs:320）。 |  |
| `crates/kivact-engine/src/contract/node.rs:183` | EditingData.placeholder | 確認済み | option | — | 書き手は Placeholder setter（typed.rs:309-311）のみ（text_editing.rs:263）。None で placeholder なし。 |  |
| `crates/kivact-engine/src/contract/node.rs:186` | EditingData.multiline | 確認済み | option | — | 書き手は Multiline setter（typed.rs:332-334, mount/editing.rs:56）のみ。None は単一行（tree/node.rs:396 unwrap_or(false)）。 |  |
| `crates/kivact-engine/src/contract/node.rs:189` | EditingData.line_layout | 確認済み | option | — | 書き手は EditLineLayout setter（typed.rs:336-338）のみ。None で既定 layout（tree/node.rs:399-406）。 |  |
| `crates/kivact-engine/src/contract/node.rs:192` | EditingData.report_text | 確認済み | option | — | 書き手は ReportText setter（typed.rs:318-320, mount/editing.rs:60）のみ＝app の購読指定。None で報告しない。 |  |
| `crates/kivact-engine/src/contract/node.rs:195` | EditingData.report_session | 確認済み | option | — | 書き手は mount/editing.rs:38 と ReportSession setter（typed.rs:323-325）のみ。None で session 報告なし。 |  |
| `crates/kivact-engine/src/host/testing/test_host.rs:31` | Counts.turns | 確認済み | statistic | — | 書き手は counted（test_host.rs:44）のみ、読み手は TestHost::turns（:965）→ 試験・Inspect（launch/inspect.rs:87）だけ。制御に使われない。 |  |
| `crates/kivact-engine/src/host/testing/test_host.rs:33` | Counts.frame_ticks | 確認済み | statistic | — | 書き手は counted（test_host.rs:49）のみ、読み手は frame_ticks()（:970）→ 試験の assert（real_window_tests.rs:237）だけ。 |  |
| `crates/kivact-engine/src/host/window.rs:71` | HostWindow.parent | 閉じ方の変更 | option | P8-EN | 書き手は HostWindow::new（window_changes.rs:27、request.parent）のみで以後不変。値は app が window を宣言した親 window（kivact-core window_view.rs:168）＝authored な値。読み手 events.rs:235。 | `` // xtask-allow(state-fields): option: the window the app declared this one in, given by its open request (window_changes.rs) and never changed; absent for a top-level window; modal input routing reads it `` |
| `crates/kivact-engine/src/layer.rs:60` | Look.grouped | 確認済み | cache | — | Look::of（layer.rs:174 effects::grouped＝blend≠Normal か clip_path）で paint から導出、Model::derive（sync.rs:200-220）が props 変化で置換。文言は事実どおり。 |  |
| `crates/kivact-engine/src/layer.rs:63` | Look.clip | 確認済み | cache | — | Look::of（layer.rs:175 p.clip）で導出、derive で更新。 |  |
| `crates/kivact-engine/src/layer.rs:68` | Look.rounded | 確認済み | cache | — | Look::of（layer.rs:177 !radius.is_zero()）で導出、derive で更新。 |  |
| `crates/kivact-engine/src/layer.rs:71` | Look.filtered | 文言 | cache | P8-EN | Look::of（layer.rs:178）は !e.filter.is_empty() だけ。理由文（と doc comment）の「masks or a backdrop filter」は誤り（それは masked/backdropped）。doc comment も同時に直すこと。 | `` // xtask-allow(state-fields): cache: Look::of derives it from a non-empty CSS filter list of the paint; Model::derive replaces the look when the props change `` |
| `crates/kivact-engine/src/layer.rs:74` | Look.masked | 文言 | cache | P8-EN | Look::of（layer.rs:179 !e.masks.is_empty()）で導出。理由が field 名の繰返し「masked」のみ。 | `` // xtask-allow(state-fields): cache: Look::of derives it from a non-empty mask list of the paint; Model::derive replaces the look when the props change `` |
| `crates/kivact-engine/src/layer.rs:76` | Look.backdropped | 文言 | cache | P8-EN | Look::of（layer.rs:180 !e.backdrop.is_empty()）で導出。理由が field 名の繰返し「backdropped」のみ。 | `` // xtask-allow(state-fields): cache: Look::of derives it from a non-empty backdrop filter list of the paint; Model::derive replaces the look when the props change `` |
| `crates/kivact-engine/src/layer.rs:78` | Look.blended | 確認済み | cache | — | Look::of（layer.rs:181 blend≠Normal）で導出、derive で更新。 |  |
| `crates/kivact-engine/src/layer.rs:81` | Look.isolated | 確認済み | cache | — | Look::of（layer.rs:182）で導出。 |  |
| `crates/kivact-engine/src/layer.rs:86` | Look.preserves | 確認済み | cache | — | Look::of（layer.rs:184 transform_style==Preserve3d）で導出。 |  |
| `crates/kivact-engine/src/layer.rs:89` | Look.flat | 確認済み | cache | — | Look::of（layer.rs:185 transform.is_2d()、kivact-types paint.rs:730）で導出。transformed と含意（!transformed⇒flat）があるが導出値なので cache として可。 |  |
| `crates/kivact-engine/src/layer.rs:92` | Look.transformed | 確認済み | cache | — | Look::of（layer.rs:186 transform≠NONE）で導出、derive で更新。 |  |
| `crates/kivact-engine/src/layer.rs:95` | Look.perspective | 確認済み | cache | — | Look::of（layer.rs:187 perspective.is_some()）で導出。 |  |
| `crates/kivact-engine/src/layer.rs:98` | Look.backface_hidden | 確認済み | cache | — | Look::of（layer.rs:188）で導出。 |  |
| `crates/kivact-engine/src/layer.rs:101` | Look.shader | 確認済み | cache | — | Look::of（layer.rs:189-192 background layer の Shader）で導出。 |  |
| `crates/kivact-engine/src/layer.rs:115` | SubtreeFacts.layer | 確認済み | cache | — | subtree_facts/node_facts（layer.rs:240-250, 273-279）で Look と子の facts から導出。PaintList.facts に保持、stale_fact→refresh_facts（walk.rs:472-520）で無効化、d… |  |
| `crates/kivact-engine/src/layer.rs:118` | SubtreeFacts.has_layer | 確認済み | cache | — | 同上（layer \|\| below.has_layer）。 |  |
| `crates/kivact-engine/src/layer.rs:121` | SubtreeFacts.has_item | 確認済み | cache | — | 同上（look.shader \|\| scene_item \|\| 子）。文言どおり。 |  |
| `crates/kivact-engine/src/layer.rs:124` | SubtreeFacts.has_backdrop | 確認済み | cache | — | 同上（look.backdropped \|\| 子）。 |  |
| `crates/kivact-engine/src/layer.rs:127` | SubtreeFacts.has_blend | 確認済み | cache | — | 同上（look.blended \|\| 子）。 |  |
| `crates/kivact-engine/src/lower/assets.rs:63` | ArvaneAssets.pending | 文言 | independent | P8-EN | 書き手は check_family（assets.rs:293 push）、取り出しは take_pending（:305、sync.rs:231）のみ。fonts/revision は別データの世代で queue を制御しない。理由「any content is valid on its own」は定型句。 | `` // xtask-allow(state-fields): independent: missing-family reports check_family queues during a layout pass and sync takes each commit; the font and store generations version other data and never gate the queue `` |
| `crates/kivact-engine/src/lower/context.rs:57` | Context.dropped | 閉じ方の変更 | independent | P8-EN | statistic と称するが制御に使う：succeeded_since（context.rs:144-146）で scene cache 挿入を分岐（scene/paint.rs:74）、offscreen.rs:150 で Err を返す。drawables（hosting で設定）とは独立。コード不変で prefix 変更のみ。 | `` // xtask-allow(state-fields): independent: drop_command counts each undrawable command; succeeded_since compares it to a checkpoint to keep a failed scene out of the cache and offscreen fails on it; canvas hosting or not, any count occurs `` |
| `crates/kivact-engine/src/lower/context.rs:60` | Context.drawables | 確認済み | option | — | 書き手は builder Context::hosting（context.rs:120）のみ、new では None（:108）。構築時に caller が与える。 |  |
| `crates/kivact-engine/src/lower/glyphs.rs:28` | Colors.glyphs | 確認済み | option | — | Colors は描画ごとに caller が style/tint から構築（scene/text.rs:75,126、paint/text.rs:151、glyphs.rs:89）。内部 event で書換えなし。 |  |
| `crates/kivact-engine/src/lower/glyphs.rs:31` | Colors.decorations | 確認済み | option | — | 同上。None で文字色（currentColor）。 |  |
| `crates/kivact-engine/src/lower/glyphs.rs:34` | Colors.outline | 確認済み | option | — | 書き手は構築と builder outlined（glyphs.rs:46、style.stroke）のみ。 |  |
| `crates/kivact-engine/src/measure/editor.rs:14` | EditorNode.lines | 確認済み | cache | — | 正本は editor の値と style、refresh_lines（measure/input.rs:73-100）が作成・照合、fonts で破棄（sync.rs:141）。 |  |
| `crates/kivact-engine/src/measure/editor.rs:18` | EditorNode.measure_lines | 確認済み | cache | — | refresh_lines の幅 probe 用（measure/input.rs:73-75）、正本同上。lines とは独立の cache。 |  |
| `crates/kivact-engine/src/model/aggregate.rs:54` | Model.window_request | 文言 | independent | P8-EN | 書き手は Writer::set_window（writer.rs:502）のみ、取り出しは finish_window（window_changes.rs:175 take）。modifiers は shell 入力の projection で無関係。理由が組合せの「なぜ」を述べていない。 | `` // xtask-allow(state-fields): independent: the title and minimum size Writer::set_window queues and finish_window takes after the turn; the modifiers projection follows shell key input, so either is set without the other `` |
| `crates/kivact-engine/src/model/changes.rs:136` | Changes.paint | 文言 | independent | P8-EN | 書き手 invalidate_paint（sync.rs:85、host events.rs:295 等）、sync.rs:148,273、layout_motion steps.rs:208。restack/fonts は別入力。全組合せ到達。理由が定型句。 | `` // xtask-allow(state-fields): independent: raised by Model::invalidate_paint (host environment and asset events) and by image and layout-motion steps; restack and fonts are raised by other inputs (paint-order writes, a font registration), so any set can be pending at a sync `` |
| `crates/kivact-engine/src/model/changes.rs:138` | Changes.restack | 文言 | independent | P8-EN | 書き手 apply.rs:64-533、motion/presence/transform 各所。paint・fonts とは別入力。理由が定型句。 | `` // xtask-allow(state-fields): independent: raised by writes and motions that change paint order (Model::apply, state and layout motion, presence); paint and fonts are raised by other inputs (host repaints, a font registration), so any set can be pending at a sync `` |
| `crates/kivact-engine/src/model/changes.rs:140` | Changes.fonts | 文言 | independent | P8-EN | 書き手は fonts_added（sync.rs:92、writer.rs:390 の font 登録）のみ。Changes 上では paint=F と並ぶ（sync 前）ので全組合せ到達。理由が定型句。 | `` // xtask-allow(state-fields): independent: raised only by Model::fonts_added when the app registers a font; paint and restack are raised by other inputs (host repaints, paint-order writes), so any set can be pending at a sync `` |
| `crates/kivact-engine/src/model/changes.rs:148` | Dispatched.paint | 誤（付録の設計） | independent（sync.rs:148 削除後） | P8-EN | sync（sync.rs:127-149）が fonts のとき paint=true を立ててから dispatch（:156）するので Dispatched で fonts⇒paint、(fonts=T,paint=F) 到達不能。読み手は sync.rs:159 の redraw=paint\|\|fonts\|\|… のみで冗長。詳細 G3.wrong.md。 | `` // xtask-allow(state-fields): independent: Changes::dispatch takes Changes::paint as raised by its own inputs (host repaints, images, layout motion); a font registration no longer raises it, so it pairs with any fonts and restack `` |
| `crates/kivact-engine/src/model/changes.rs:150` | Dispatched.restack | 文言 | independent | P8-EN | dispatch（changes.rs:231）で Changes.restack をそのまま取る。paint/fonts と含意なし（fonts でも restack は立たない）。理由が定型句。paint の WRONG 修正と同時に書換え。 | `` // xtask-allow(state-fields): independent: Changes::dispatch takes Changes::restack (paint-order writes and motions) as is; paint and fonts come from other inputs, so any set occurs `` |
| `crates/kivact-engine/src/model/changes.rs:152` | Dispatched.fonts | 誤（付録の設計） | independent（sync.rs:148 削除後） | P8-EN | Dispatched.paint と同じ含意 fonts⇒paint（sync.rs:148）。読み手 sync.rs:160,176（forget_inks）。修正は G3.wrong.md の1件にまとめる。 | `` // xtask-allow(state-fields): independent: Changes::dispatch takes Changes::fonts (Model::fonts_added) as is; sync no longer raises paint with it, so it pairs with any paint and restack `` |
| `crates/kivact-engine/src/model/drawables/place.rs:64` | LayerFrame.in_3d | 文言 | independent | P8-EN | 書き手は push_layer（place.rs:120 inside_3d＝最も近い親 layer の preserves）のみ。preserves は自 layer の transform_style（:139）で別入力、4組とも有効。理由が定型句。 | `` // xtask-allow(state-fields): independent: push_layer reads it from the nearest open layer's preserves (inside_3d); preserves is this layer's own transform_style, so a flat or preserving layer sits in a flat or 3D parent `` |
| `crates/kivact-engine/src/model/drawables/place.rs:69` | LayerFrame.preserves | 文言 | independent | P8-EN | 書き手は push_layer（place.rs:139）のみ、DrawingLayer の transform_style。in_3d は親側。理由が定型句。 | `` // xtask-allow(state-fields): independent: push_layer takes it from this layer's transform_style; in_3d is the parent layer's, so a flat or preserving layer sits in a flat or 3D parent `` |
| `crates/kivact-engine/src/model/drawables/state.rs:24` | ExportAsk.generation | 確認済み | identity | — | 書き手は apply_request（model/drawables/request.rs:14-22）のみ＝app の要求世代。比較 state.rs:152,183,261、export.rs:64。同 owner に走行中 flag なし。 |  |
| `crates/kivact-engine/src/model/drawables/state.rs:29` | ExportAsk.scale | 確認済み | option | — | 書き手は apply_request のみ、app の open.scale 由来（requests.rs:44）。None で window scale（state.rs:38）。 |  |
| `crates/kivact-engine/src/model/edit/state.rs:17` | InputSource.transform | 確認済み | cache | — | ime_changed（edit/ime.rs:125-134）で毎回構築し given と全体比較（:136-139）するだけの key 成分。 |  |
| `crates/kivact-engine/src/model/edit/state.rs:19` | InputSource.password | 確認済み | cache | — | 同上（n.password()、ime.rs:132）。 |  |
| `crates/kivact-engine/src/model/edit/state.rs:21` | InputSource.read_only | 確認済み | cache | — | 同上（n.read_only()、ime.rs:133）。 |  |
| `crates/kivact-engine/src/model/edit/state.rs:32` | SessionSource.area | 確認済み | cache | — | session_ime_changed（edit/session.rs:124-131）で構築し全体比較する key 成分。 |  |
| `crates/kivact-engine/src/model/edit/state.rs:34` | SessionSource.transform | 確認済み | cache | — | 同上（window_map、session.rs:123）。 |  |
| `crates/kivact-engine/src/model/pointer/state.rs:15` | Pointer.file_target | 文言 | independent | P8-EN | 書き手は file drag 処理（pointer/shell.rs:447、Drop で None）と forget_missing（state.rs:54-55）のみ。hover_sent/position_sent は mouse event の projection で別入力。理由が組合せの「なぜ」を欠く。 | `` // xtask-allow(state-fields): independent: the file-listening node under an OS file drag, set by the file drag events and cleared at Drop, Leave or its removal; the hover and position projections follow mouse events, so any pair occurs `` |
| `crates/kivact-engine/src/model/pointer/state.rs:114` | PenTrack.over | 文言 | independent | P8-EN | 書き手 set_over（track_hover machine.rs:248-250、forget_missing state.rs:439）。hover は primary の pen だけ Some（state.rs:241）、clear_hover（machine.rs:204）は chain だけ空にし over を残す。全組合せ到達。理由が設定者を述べない。 | `` // xtask-allow(state-fields): independent: the nearest pointer-listening ancestor of the hit, set by track_hover; clear_hover empties the hover chain but keeps it for Leave, and a non-primary pen has no chain, so every pair occurs `` |
| `crates/kivact-engine/src/model/pointer/state.rs:381` | PointerSource.pressure | 閉じ方の変更 | independent | P8-EN | authored 設定ではなく shell の各 sample で上書きされる入力値（pointer/shell.rs:59,296,348）。tilt とは platform が別々に報告。option の定義（構築時・caller の設定値）に合わない。 | `` // xtask-allow(state-fields): independent: the pressure of the pointer's last shell sample (shell.rs); platforms report pressure and tilt apart (a touch may have pressure and no tilt, a mouse neither), so every pair occurs `` |
| `crates/kivact-engine/src/model/pointer/state.rs:384` | PointerSource.tilt | 閉じ方の変更 | independent | P8-EN | 同上（shell.rs:60,297,349）。実行時に shell event で書き換わる値なので option ではない。 | `` // xtask-allow(state-fields): independent: the pen tilt of the pointer's last shell sample (shell.rs); platforms report tilt and pressure apart (a pen may lack either, a mouse has neither), so every pair occurs `` |
| `crates/kivact-engine/src/mutation.rs:95` | Mutation.style | 文言 | independent | P8-EN | props 表（host/write/typed.rs の mut=、例 :168,:367）が property ごとに定数を and で合成し、apply（apply.rs:202,412）が各 flag を別工程で読む。全和集合が有効。理由が定型句。 | `` // xtask-allow(state-fields): independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is valid `` |
| `crates/kivact-engine/src/mutation.rs:98` | Mutation.measure | 文言 | independent | P8-EN | 同上（typed.rs:268 等で MEASURE を合成）。 | `` // xtask-allow(state-fields): independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is valid `` |
| `crates/kivact-engine/src/mutation.rs:101` | Mutation.text | 文言 | independent | P8-EN | 同上（typed.rs:176、apply.rs:198）。 | `` // xtask-allow(state-fields): independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is valid `` |
| `crates/kivact-engine/src/mutation.rs:105` | Mutation.placed | 文言 | independent | P8-EN | 同上（typed.rs:367、apply.rs:236 で apply 自身も OR）。 | `` // xtask-allow(state-fields): independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is valid `` |
| `crates/kivact-engine/src/mutation.rs:108` | Mutation.restack | 文言 | independent | P8-EN | 同上（typed.rs:168 等、apply.rs:244,310）。 | `` // xtask-allow(state-fields): independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is valid `` |
| `crates/kivact-engine/src/mutation.rs:111` | Mutation.paint | 文言 | independent | P8-EN | 同上（typed.rs:172、apply.rs:220）。 | `` // xtask-allow(state-fields): independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is valid `` |
| `crates/kivact-engine/src/mutation.rs:114` | Mutation.states | 文言 | independent | P8-EN | 同上（typed.rs:180、apply.rs:220,228）。 | `` // xtask-allow(state-fields): independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is valid `` |
| `crates/kivact-engine/src/mutation.rs:117` | Mutation.read_only | 文言 | independent | P8-EN | 同上（typed.rs:279,302 READ_ONLY.when、apply.rs:212）。 | `` // xtask-allow(state-fields): independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is valid `` |
| `crates/kivact-engine/src/mutation.rs:120` | Mutation.subtree | 文言 | independent | P8-EN | 同上（typed.rs:184、apply.rs:412）。 | `` // xtask-allow(state-fields): independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is valid `` |
| `crates/kivact-engine/src/mutation.rs:123` | Mutation.local_text | 文言 | independent | P8-EN | 同上（typed.rs:268、apply.rs:398-406：他 flag と併存すれば local 扱いを外すだけで組合せは有効）。 | `` // xtask-allow(state-fields): independent: one flag of the invalidation set a property declares in the props table (host/write/typed.rs `mut =`); sets combine flags of distinct concerns with Mutation::and and Model::apply reads each flag in its own step, so any union is valid `` |
| `crates/kivact-engine/src/paint/state.rs:155` | Ink.rect | 確認済み | cache | — | 書き手は paint/list/ink.rs:98（layout と Look から再計算）のみ、generation/size/content 一致時だけ再利用（ink.rs:85-90）、forget_inks で破棄。 |  |
| `crates/kivact-engine/src/paint/state.rs:160` | Ink.generation | 確認済み | identity | — | cache の有効性刻印（ink.rs:86 比較）。同 owner に走行中 flag なし（rect は描画域の値）、SM-6 に当たらない。 |  |
| `crates/kivact-engine/src/paint/state.rs:341` | ChildIndex.axis | 確認済み | cache | — | 書き手は ChildIndex::sort（paint/list/index.rs:172）のみ、entries から導出し build/refresh ごとに再計算。 |  |
| `crates/kivact-engine/src/paint/state.rs:351` | ChildIndex.pass | 確認済み | identity | — | 書き手 build（index.rs:98,124）と refresh（:156）。layout pass の刻印で比較（:138）、phase を隠さない。 |  |
| `crates/kivact-engine/src/text_input/edit_lines.rs:11` | EditLines.damage | 文言 | cache | P8-EN | 理由「from the refresh that made these lines」は誤り。書き手 hold_damage（edit_lines.rs:32、flush_damage editor.rs:48）、取り出し refresh_lines（input.rs:87）。base 不一致・None なら全再構築（refresh.rs:95-101）＝性能 hint。 | `` // xtask-allow(state-fields): cache: the editor damage since these lines' revision, merged by hold_damage (EditorNode::flush_damage) and taken by refresh_lines; a hint only: absent or of another base, the lines are rebuilt from the text `` |
| `crates/kivact-engine/src/text_input/records.rs:24` | Key.multiline | 誤（付録の設計） | type（Breaks enum） | P8-EN | Key::of（records.rs:52-53）は Source からの写し。Source 側で wrap_width.is_some()⇒multiline（edit_line_layout は単一行で既定 wrap=false、tree/node.rs:399-406）。Source と同じ型で閉じる。詳細 G3.wrong.md。 |  |
| `crates/kivact-engine/src/text_input/records.rs:26` | Key.wrap_width | 誤（付録の設計） | type（Breaks enum） | P8-EN | 同上。読み手 chunks.rs:77、refresh.rs:96、shaping.rs:334。Breaks に統合し marker 削除。 |  |
| `crates/kivact-engine/src/text_input/source.rs:12` | Source.damage | 文言 | independent | P8-EN | 構築は refresh_lines（measure/input.rs:84-88）：幅 probe なら None、他は lines が保持した damage。password とは別入力で全組合せ有効。理由が組合せを述べない。Breaks 修正後も marker は残る。 | `` // xtask-allow(state-fields): independent: the damage refresh_lines takes from the visible lines (none for a width probe or before any edit); password is the node's purpose at this refresh, so edits of a masked or plain input give every pair `` |
| `crates/kivact-engine/src/text_input/source.rs:17` | Source.password | 文言 | independent | P8-EN | 構築は measure/input.rs:90（n.password()）のみ。damage とは独立。理由「any combination is valid」は定型句。Breaks 修正後も marker は残る。 | `` // xtask-allow(state-fields): independent: the node's password purpose (HostNode::password) at this refresh; damage comes from edits of any input, masked or not, so both pairs occur `` |
| `crates/kivact-engine/src/text_input/source.rs:22` | Source.multiline | 誤（付録の設計） | type（Breaks enum） | P8-EN | 「any combination is valid」は偽：wrap_width は wrap（edit_line_layout().wrap、単一行では常に false）のときだけ Some（measure/input.rs:67,92）なので (multiline=F, wrap_width=Some) 到達不能＝SM-2。詳細 G3.wrong.md。 |  |
| `crates/kivact-engine/src/text_input/source.rs:25` | Source.wrap_width | 誤（付録の設計） | type（Breaks enum） | P8-EN | option でもない：値は authored ではなく refresh ごとの viewport 幅（measure/input.rs:69-72,92）。かつ multiline=true のときだけ意味を持つ（SM-2）。Breaks::Lines{wrap} に入れる。 |  |
| `crates/kivact-engine/src/tree/child_order.rs:8` | Link.previous | 確認済み | independent | — | 書き手 Editing::insert/remove（child_order.rs:36-80）。4組＝唯一の子・先頭・末尾・中間で全て有効、理由が端での欠如を説明している。 |  |
| `crates/kivact-engine/src/tree/child_order.rs:10` | Link.next | 確認済み | independent | — | 同上。 |  |
| `crates/kivact-engine/src/tree/node.rs:125` | Place.transformed | 文言 | cache | P8-EN | 書き手は update_place（node.rs:202：paint・state style・走行中 motion・親 perspective）と raise_under_perspective（node_tree.rs:402、上げるだけ）。理由が motion と perspective を欠き不正確。update_place の doc「The only writer」も事実と違う。 | `` // xtask-allow(state-fields): cache: update_place derives it from the base paint and state style transforms, a running state motion and the parent's perspective (raise_under_perspective only raises it); the walks skip untransformed nodes by it `` |
| `crates/kivact-engine/src/tree/node.rs:130` | Place.z_states | 確認済み | cache | — | 書き手は update_place（node.rs:197,204、state style の z_index）のみ、props 変化で再導出。 |  |
| `crates/kivact-engine/src/tree/node_tree.rs:15` | NodeTree.pending_child_order | 文言 | independent | P8-EN | 書き手 insert/detach（node_tree.rs:458,489、is_editing 時）、remove_with（:529）、取り出し pack_child_orders（:500）。detached subtree も対象なので root の有無と独立。理由が定型句。 | `` // xtask-allow(state-fields): independent: parents whose child order insert or detach edited, drained by pack_child_orders before layout; detached subtrees edit orders too, so pending work occurs with or without a root `` |
| `crates/kivact-engine/src/tree/node_tree.rs:19` | NodeTree.root | 閉じ方の変更 | independent | P8-EN | authored 設定ではなく構造書込みで変わる実行時の値：install（node_tree.rs:357、Root 種の node）で設定、remove_with（:537）で None。option の定義に合わない。 | `` // xtask-allow(state-fields): independent: the Root-kind node, set when the client installs it and cleared when it is removed; detached subtrees queue child-order packing too, so pending work occurs with or without a root `` |

### B.4 G4

| marker（HEAD） | owner.field | 判定 | 閉じ方 | unit | 根拠 | 新しい marker 行 |
|---|---|---|---|---|---|---|
| `crates/kivact-core/src/client/trace.rs:23` | Sink.stage | 文言 | option | P8-CO | 書き手は Trace::new の literal (trace.rs:65) のみ、KIVACT_TRACE から一度だけ。読み手 trace.rs:109,150。全部 false なら sink=None (trace.rs:48) なので「any combination of the three」は事実と違う（全 false は不到達） | `` // xtask-allow(state-fields): option: whether KIVACT_TRACE names the `stage` stream, read once by Trace::new; false means stage lines are not written (a sink always has at least one stream) `` |
| `crates/kivact-core/src/client/trace.rs:25` | Sink.semantics | 文言 | option | P8-CO | 書き手は Trace::new (trace.rs:66) のみ。読み手 trace.rs:112。全 false は sink=None (trace.rs:48) で不到達なので「any combination」は誤り | `` // xtask-allow(state-fields): option: whether KIVACT_TRACE names the `semantics` stream, read once by Trace::new; false means semantic diffs are not written (a sink always has at least one stream) `` |
| `crates/kivact-core/src/client/trace.rs:27` | Sink.host_events | 文言 | option | P8-CO | 書き手は Trace::new (trace.rs:67) のみ。読み手 trace.rs:98。全 false は sink=None (trace.rs:48) で不到達なので「any combination」は誤り | `` // xtask-allow(state-fields): option: whether KIVACT_TRACE names the `host_events` stream, read once by Trace::new; false means host events are not written (a sink always has at least one stream) `` |
| `crates/kivact-core/src/custom/measurement.rs:23` | Constraints.known_width | 確認済み | independent | — | 書き手 measurement.rs:172 (r.known_width をそのまま)、源は layout tree.rs:108 の taffy known.width。高さとは軸ごとに独立に決まり (types layout.rs:238 で width だけ Some の… |  |
| `crates/kivact-core/src/custom/measurement.rs:26` | Constraints.known_height | 確認済み | independent | — | 書き手 measurement.rs:173、源は tree.rs:109 の taffy known.height。軸ごとに独立 (layout.rs:238-239)。理由文は事実どおり |  |
| `crates/kivact-core/src/gpu_interop/import/image.rs:31` | ExternalImage.latest | 文言 | independent | P8-GPU | 書き手は import_in の task (image.rs:211) のみ、publish が起点。status は registry が import 結果で書く。Pending 中の publish・Lost 後の残存も可、未 import の frame は描かれない (arvane-compose gpu/external.rs:8-15)。理由文に書き手が無い | `` // xtask-allow(state-fields): independent: set only by the import's task when the engine calls FramePublisher::publish; the window sets the status from the import result, and a frame kept after Lost or published before Ready simply draws nothing, so every pair is valid `` |
| `crates/kivact-core/src/launch/settings.rs:19` | AppSettings.locale | 確認済み | option | — | 書き手は AppOptions::new (settings.rs:47) と builder locale (settings.rs:76) のみ。実行時の書換えなし (into_view_settings で移動) |  |
| `crates/kivact-core/src/launch/settings.rs:23` | AppSettings.window_size | 確認済み | option | — | 書き手は new (settings.rs:50) と builder window_size (settings.rs:98) のみ。読み手 host_options (settings.rs:172) |  |
| `crates/kivact-core/src/launch/settings.rs:27` | AppSettings.min_window_size | 確認済み | option | — | 書き手は new (settings.rs:51) と builder min_window_size (settings.rs:110) のみ。読み手 runtime.rs:123,332 |  |
| `crates/kivact-core/src/launch/settings.rs:30` | AppSettings.blocking_threads | 確認済み | option | — | 書き手は new と builder blocking_threads (settings.rs:143) のみ。読み手 runtime.rs:179 → task/state.rs:295 で None なら既定値 |  |
| `crates/kivact-core/src/launch/settings.rs:35` | AppSettings.devtools_in_tests | 確認済み | option | — | 書き手は new (false) と kivact::test 用 builder devtools_in_tests (settings.rs:151) のみ。読み手 runtime.rs:170 |  |
| `crates/kivact-core/src/view/devtools_state.rs:64` | SessionData.next_override | 確認済み | identity | — | 書き手 styles.rs:104-105 (get+1 → set) と初期化 devtools_state.rs:210 のみ。id 割当てだけで phase を隠さない |  |
| `crates/kivact-core/src/view/devtools_state.rs:70` | SessionData.next_snapshot | 確認済み | identity | — | 書き手 perf.rs:91-92 と初期化 devtools_state.rs:215 のみ。snapshot id の割当てのみ |  |
| `crates/kivact-core/src/view/devtools_state.rs:73` | SessionData.sent_windows | 確認済み | cache | — | 書き手 tree.rs:161 (notify_windows_changed の Sent::update) のみ。正本は list_windows、Sent は S-9 の dedup cache (types cell.rs:69)。他 field と組の暗黙状態なし |  |
| `crates/kivact-core/src/view/devtools_state.rs:75` | SessionData.errors_sent | 文言 | cache | P8-CO | 書き手 console.rs:57 のみ。保持するのは最後に通知した error の seq (console.rs:46-47) で「count」ではない。正本 rt.reported()、S-9 Sent。理由文の事実誤り | `` // xtask-allow(state-fields): cache: the seq of the last reported error already notified (S-9 `Sent`); the next turn notifies only entries of `rt.reported()` with a larger seq `` |
| `crates/kivact-core/src/view/devtools_state.rs:93` | PropWrite.effect | 確認済み | statistic | — | 書き手 devtools_hooks.rs:41-46 (prop_written、recording 中のみ) のみ。読み手 updates.rs:321-324 で cause graph の辺を作るだけ (DevTools 表示)。制御分岐なし |  |
| `crates/kivact-core/src/view/drawable_share.rs:111` | ExportOpen.scale | 確認済み | option | — | 書き手は初期化 None (drawable_share.rs:338) と App の set_scale (drawable_share.rs:400) のみ。内部 event では書き換わらない |  |
| `crates/kivact-core/src/view/drawable_share.rs:113` | ExportOpen.next_number | 確認済み | identity | — | 書き手 on_frame (drawable_share.rs:306) と初期化 :339 のみ。frame 番号の単調割当て、wait の `number > after` 比較に使う |  |
| `crates/kivact-core/src/view/drawable_share.rs:125` | ExportRequest::Pixels.generation | 確認済み | identity | — | generation は restart (drawable_share.rs:176) で割当て、on_frame (:297) が current と比較。既に variant ごとに generation を持つ enum で SM-6 の Running{id} 形 |  |
| `crates/kivact-core/src/view/drawable_share.rs:127` | ExportRequest::Pixels.latest | 文言 | independent | P8-CO | 書き手 on_frame (drawable_share.rs:300-308) のみ、生成時 None (:367)。「any request phase may have one」は事実と違う (latest を持つのは Pixels/GpuProvided だけ) | `` // xtask-allow(state-fields): independent: set only by ExportOpen::on_frame with a frame of this request's generation; None until the first such frame, and any generation may still lack one `` |
| `crates/kivact-core/src/view/drawable_share.rs:145` | ExportRequest::GpuProvided.generation | 確認済み | identity | — | on_textures (drawable_share.rs:241) が GpuRequested の generation を引き継ぐ。on_frame/on_gpu_renew が current と比較。enum 内 id で SM-6 形 |  |
| `crates/kivact-core/src/view/drawable_share.rs:150` | ExportRequest::GpuProvided.latest | 文言 | independent | P8-CO | 書き手 on_frame (drawable_share.rs:303-308) のみ、生成時 None (:249)。「any request phase」は誤り (この variant の中だけの値) | `` // xtask-allow(state-fields): independent: set only by ExportOpen::on_frame with a GPU frame of this request's generation; None until the first such frame, and any generation may still lack one `` |
| `crates/kivact-core/src/view/editing_data.rs:97` | EditorProps.behavior | 確認済み | option | — | 書き手 TextEditing::new (text_editing.rs:207) と builder (text_editing.rs:110) のみ。mount/editing.rs:26 で消費。実行時の書換えなし |  |
| `crates/kivact-core/src/view/editing_data.rs:99` | EditorProps.highlight | 確認済み | option | — | 書き手 new と builder (text_editing.rs:144) のみ。editing.rs:41 で一度だけ bind |  |
| `crates/kivact-core/src/view/editing_data.rs:101` | EditorProps.decorations | 確認済み | option | — | 書き手 new と builder (text_editing.rs:157) のみ。editing.rs:41 で消費 |  |
| `crates/kivact-core/src/view/editing_data.rs:103` | EditorProps.forward_keys | 確認済み | option | — | 書き手 new と builder (text_editing.rs:87) のみ。editing.rs:58 |  |
| `crates/kivact-core/src/view/editing_data.rs:106` | EditorProps.multiline | 確認済み | option | — | 書き手 new (false) と builder multiline (text_editing.rs:228) のみ。editing.rs:54 で engine へ |  |
| `crates/kivact-core/src/view/editing_data.rs:108` | EditorProps.line_layout | 確認済み | option | — | 書き手 new と builder (text_editing.rs:234) のみ。editing.rs:55 で multiline と別 prop として bind |  |
| `crates/kivact-core/src/view/editing_data.rs:110` | EditorProps.report_text | 確認済み | option | — | 書き手 new (false) と builder (text_editing.rs:242) のみ。editing.rs:58-60 |  |
| `crates/kivact-core/src/view/editing_data.rs:112` | EditorProps.purpose | 確認済み | option | — | 書き手 new と builder (text_editing.rs:247) のみ |  |
| `crates/kivact-core/src/view/editing_data.rs:114` | EditorProps.read_only | 確認済み | option | — | 書き手 new と builder (text_editing.rs:252) のみ (node_modifiers.rs:617 は Semantics 側の別 field) |  |
| `crates/kivact-core/src/view/editing_data.rs:116` | EditorProps.max_length | 確認済み | option | — | 書き手 new と builder (text_editing.rs:258) のみ |  |
| `crates/kivact-core/src/view/editing_data.rs:119` | EditorProps.placeholder | 確認済み | option | — | 書き手 new と builder (text_editing.rs:263) のみ |  |
| `crates/kivact-core/src/view/element_data.rs:521` | Common.exit_layout | 確認済み | option | — | 書き手 builder (element_data.rs:312) のみ、finish_resolve (resolve.rs:239) で移動。Common の全 field は builder modifier が書き mount が take するだけ (確認済み) |  |
| `crates/kivact-core/src/view/element_data.rs:530` | Common.focusable_when_disabled | 確認済み | option | — | 書き手 builder (element_data.rs:361) のみ |  |
| `crates/kivact-core/src/view/element_data.rs:532` | Common.roving_focus | 確認済み | option | — | 書き手 builder (element_data.rs:376) のみ |  |
| `crates/kivact-core/src/view/element_data.rs:534` | Common.hit_testable | 確認済み | option | — | 書き手 builder (element_data.rs:343) のみ |  |
| `crates/kivact-core/src/view/element_data.rs:537` | Common.focus_on_press | 確認済み | option | — | 書き手 builder Node::focus_on_press (node_modifiers.rs:885) のみ。読み手 regions.rs:316, resolve.rs:259 |  |
| `crates/kivact-core/src/view/element_data.rs:542` | Common.text_editing | 確認済み | option | — | 書き手 Node::text_editing (text_editing.rs:307) のみ、resolve.rs:62 で take。ime と両方 Some でも engine が Native を優先 (edit/session.rs:44-47) し定義済み |  |
| `crates/kivact-core/src/view/element_data.rs:544` | Common.ime | 確認済み | option | — | 書き手 ime_session (node_modifiers.rs:747) のみ。text_editing と併存時は engine が Native 優先 (session.rs:44-47) |  |
| `crates/kivact-core/src/view/element_data.rs:553` | Common.part_region_events | 確認済み | option | — | 書き手 region_events (node_modifiers.rs:880) と apply_part の付替え (node_modifiers.rs:840-853) のみ。builder 時の値 |  |
| `crates/kivact-core/src/view/element_data.rs:558` | Common.part_modal | 確認済み | option | — | 書き手 modal (node_modifiers.rs:875) のみ。読み手 resolve.rs:123, regions.rs:185, overlays.rs:384 |  |
| `crates/kivact-core/src/view/element_data.rs:563` | Common.draw | 確認済み | option | — | 書き手 builder (element_data.rs:495) のみ。builders.rs:291, element_data.rs:129 は不適用時の破棄 (take) |  |
| `crates/kivact-core/src/view/element_data.rs:566` | Common.draw_over | 確認済み | option | — | 書き手 builder (node_modifiers.rs:789) のみ |  |
| `crates/kivact-core/src/view/element_data.rs:569` | Common.draw_with_layout | 確認済み | option | — | 書き手 builder (node_draw.rs:12) のみ |  |
| `crates/kivact-core/src/view/element_data.rs:571` | Common.intrinsic | 確認済み | option | — | 書き手 builder (element_data.rs:501) のみ |  |
| `crates/kivact-core/src/view/element_data.rs:574` | Common.image_fit | 確認済み | option | — | 書き手 builder (builders.rs:200) のみ |  |
| `crates/kivact-core/src/view/element_data.rs:577` | Common.monochrome | 確認済み | option | — | 書き手 builder (builders.rs:218) のみ |  |
| `crates/kivact-core/src/view/element_data.rs:580` | Common.image_playing | 確認済み | option | — | 書き手 builder (builders.rs:208) のみ |  |
| `crates/kivact-core/src/view/element_data.rs:583` | Common.layout_id | 確認済み | option | — | 書き手 builder (element_data.rs:240) のみ |  |
| `crates/kivact-core/src/view/element_data.rs:587` | Common.at | 確認済み | option | — | 書き手 located_at_caller (node_modifiers.rs:420) のみ。読み手 mount.rs:76, builders.rs:250, virtual_list.rs:254 |  |
| `crates/kivact-core/src/view/mount/create.rs:19` | FirstWrite.focusable_when_disabled | 確認済み | option | — | FirstWrite の書き手は finish_resolve (resolve.rs:276) のみで Common の値をそのまま運ぶ |  |
| `crates/kivact-core/src/view/mount/create.rs:21` | FirstWrite.roving_focus | 確認済み | option | — | finish_resolve (resolve.rs:276) が Common.roving_focus をそのまま運ぶ |  |
| `crates/kivact-core/src/view/mount/create.rs:23` | FirstWrite.hit_testable | 確認済み | option | — | finish_resolve (resolve.rs:276) が Common.hit_testable をそのまま運ぶ |  |
| `crates/kivact-core/src/view/mount/create.rs:28` | FirstWrite.intrinsic | 確認済み | option | — | finish_resolve (resolve.rs:276) が Common.intrinsic をそのまま運ぶ |  |
| `crates/kivact-core/src/view/mount/create.rs:30` | FirstWrite.observe_layout | 文言 | option | P8-CO | builder 入力ではなく finish_resolve が draw の下/上/layout 層の有無から計算 (resolve.rs:272-274)。一度だけ書かれ実行時不変なので option は可、文言が事実と違う | `` // xtask-allow(state-fields): option: set once by finish_resolve when the node has a draw, draw_over or draw_with_layout layer; false means the layout is not observed for a layer's size `` |
| `crates/kivact-core/src/view/mount/create.rs:33` | FirstWrite.editing | 確認済み | option | — | finish_resolve (resolve.rs:276) が Common.text_editing から取り出した EditorProps をそのまま運ぶ (resolve.rs:62-73) |  |
| `crates/kivact-core/src/view/mount/draw.rs:18` | DrawInput.below | 確認済み | option | — | DrawInput の書き手は prepare_draw (resolve.rs:216) のみ。below は Common.draw を take |  |
| `crates/kivact-core/src/view/mount/draw.rs:20` | DrawInput.over | 確認済み | option | — | prepare_draw (resolve.rs:218) が Common.draw_over を take |  |
| `crates/kivact-core/src/view/mount/draw.rs:22` | DrawInput.with_layout | 確認済み | option | — | prepare_draw (resolve.rs:219) が Common.draw_with_layout を take |  |
| `crates/kivact-core/src/view/mount/draw.rs:24` | DrawInput.local_paint | 文言 | option | P8-CO | 値は resolve が作る paint channel (resolve.rs:80-84：style 有りの Box で編集しない node) の clone。None は「local paint 不可」(style.rs:140 で報告)。「the style's local paint」は事実と違う | `` // xtask-allow(state-fields): option: the local paint channel resolve made for a styled Box that does not edit text; None means the node cannot take local paint `` |
| `crates/kivact-core/src/view/mount/draw.rs:26` | DrawInput.regions | 文言 | option | P8-CO | prepare_draw (resolve.rs:221) が attached の registry を写す。Some は part binding・region・part 制約のどれかがある時 (resolve.rs:152-157) で region 宣言が無くても Some。文言が不正確 | `` // xtask-allow(state-fields): option: the region registry attach_parts made when the node has parts, regions or part constraints; None means the draw effect paints no region `` |
| `crates/kivact-core/src/view/mount/facts.rs:13` | NodeFacts.focus_stop | 文言 | option | P8-CO | 書き手 resolve (resolve.rs:75-79) のみ：focusable source が固定 false でない、または native 編集。builder 入力の直接運搬ではない。一度だけ計算 | `` // xtask-allow(state-fields): option: set once by resolve: a focusable source not fixed to false, or native text editing; false means the theme's focus ring and disabled look do not apply `` |
| `crates/kivact-core/src/view/mount/facts.rs:17` | NodeFacts.local_paint | 文言 | option | P8-CO | 書き手 resolve (resolve.rs:80-84) のみ：style 有りの Box で編集しない時に空 channel を作る。中身は style 段 (style.rs:150-163) が書く。builder 入力ではない | `` // xtask-allow(state-fields): option: made once by resolve for a styled Box that does not edit text; None means the node cannot take local paint (reported by the style stage) `` |
| `crates/kivact-core/src/view/mount/part_input.rs:39` | Selected.observed | 誤（付録の設計） | type | P8-CO | 書き手 select (part_input.rs:52-67) のみ。observed=Some なら target も同じ値、own_canvas=false なら observed=None・target=root。(Some,None)・(Some(a),Some(b≠a))・(Some,_,false) は不到達。enum Selected へ |  |
| `crates/kivact-core/src/view/mount/part_input.rs:41` | Selected.target | 誤（付録の設計） | type | P8-CO | select (part_input.rs:59-63)：target=None は own canvas の未登録 region の時だけ、その時 observed も None。observed と同値か root 固定の 3 通りしかない |  |
| `crates/kivact-core/src/view/mount/part_input.rs:43` | Selected.own_canvas | 誤（付録の設計） | type | P8-CO | own_canvas=false ⇒ (observed None, target root) に固定 (part_input.rs:54-60)。「any combination is valid」は誤り。読み手 part_input.rs:109 のみ |  |
| `crates/kivact-core/src/view/mount/parts.rs:153` | ConstraintSources.disabled | 確認済み | option | — | 書き手 resolve.rs:161, regions.rs:200 (Common::disabled_source＝caller 自身の最後の patch)。None は constant(false) (parts.rs:176) |  |
| `crates/kivact-core/src/view/mount/parts.rs:155` | ConstraintSources.hidden | 確認済み | option | — | 書き手 resolve.rs:162, regions.rs:201 (hidden_source)。None は constant(false) (parts.rs:177) |  |
| `crates/kivact-core/src/view/mount/parts.rs:157` | ConstraintSources.focusable | 文言 | option | P8-CO | 書き手 resolve.rs:163, regions.rs:202。None は「focusable でない」ではなく part の要求次第 (focus_default=!required_focus.is_empty()、parts.rs:189-194) | `` // xtask-allow(state-fields): option: the caller's own last `focusable` modifier; absent lets the parts decide: focusable when a part requires focus, otherwise not `` |
| `crates/kivact-core/src/view/mount/resolve.rs:36` | Resolved.disabled | 確認済み | option | — | 書き手 finish_resolve (resolve.rs:235) が Common::disabled_source (caller の patch) を運ぶ |  |
| `crates/kivact-core/src/view/mount/resolve.rs:38` | Resolved.selected | 確認済み | option | — | 書き手 finish_resolve (resolve.rs:236) が selected_source を運ぶ |  |
| `crates/kivact-core/src/view/mount/resolve.rs:42` | Resolved.draw | 文言 | option | P8-CO | 値は prepare_draw (resolve.rs:212-223) が作る：draw 層か local paint channel がある時 Some。builder 入力の運搬ではない。mount 時に一度だけ | `` // xtask-allow(state-fields): option: made once by prepare_draw when the node has a draw layer or a local paint channel; None means no draw effect is bound `` |
| `crates/kivact-core/src/view/mount/resolve.rs:45` | Resolved.ime | 確認済み | option | — | finish_resolve (resolve.rs:250,301) が Common.ime を運ぶ |  |
| `crates/kivact-core/src/view/mount/resolve.rs:48` | Resolved.attached | 文言 | option | P8-CO | 値は attach_parts (resolve.rs:152-180) が作る：part binding・region・part 制約のどれかがある時 Some。builder 入力そのものではない | `` // xtask-allow(state-fields): option: made once by attach_parts when the node has part bindings, regions or part constraints; None means no part router, constraints or region registry `` |
| `crates/kivact-core/src/view/mount/semantics.rs:121` | SemanticProps.image_fit | 確認済み | option | — | 書き手 finish_resolve (resolve.rs:287) のみ、Common.image_fit を運ぶ |  |
| `crates/kivact-core/src/view/mount/semantics.rs:123` | SemanticProps.monochrome | 確認済み | option | — | finish_resolve (resolve.rs:287) が Common.monochrome を運ぶ |  |
| `crates/kivact-core/src/view/mount/semantics.rs:125` | SemanticProps.image_playing | 確認済み | option | — | finish_resolve (resolve.rs:287) が Common.image_playing を運ぶ |  |
| `crates/kivact-core/src/view/mount/semantics.rs:127` | SemanticProps.layout_id | 確認済み | option | — | finish_resolve (resolve.rs:287) が Common.layout_id を運ぶ |  |
| `crates/kivact-core/src/view/mount/semantics.rs:129` | SemanticProps.focus_on_press | 確認済み | option | — | finish_resolve (resolve.rs:287) が Common.focus_on_press を運ぶ |  |
| `crates/kivact-core/src/view/mount/style.rs:75` | NodeStyle.focusable | 確認済み | option | — | 書き手 bind_node_style (style.rs:60＝NodeFacts.focus_stop) と root.rs:105 (default false) のみ。bind_style の caller が与える値で実行時不変 |  |
| `crates/kivact-core/src/view/mount/style.rs:83` | NodeStyle.fix_layout | 確認済み | option | — | 書き手 style.rs:63 (mount_host_with の caller が渡す builder hook) と root.rs (default None) のみ |  |
| `crates/kivact-core/src/view/mount/style.rs:87` | NodeStyle.local_paint | 文言 | option | P8-CO | 書き手 style.rs:64 (NodeFacts.local_paint の clone) と root.rs (None)。中身の merged paint は style.rs:162 が後で書く channel で、None は local paint 不可 (style.rs:140 で報告)。文言が不正確 | `` // xtask-allow(state-fields): option: the local paint channel resolve made (a styled Box that does not edit text); None means local paint is reported as unsupported `` |
| `crates/kivact-core/src/view/overlays.rs:517` | KeyRoute.stop | 誤（付録の設計） | type | P8-CO | 書き手 key_route (overlays.rs:545-551) のみ。stop=modal が Some なら target は必ず Some (modal で補完) で activates=true。(Some,false) は不到達。enum KeyRoute へ |  |
| `crates/kivact-core/src/view/overlays.rs:520` | KeyRoute.activates | 誤（付録の設計） | type | P8-CO | activates=target.is_some() (overlays.rs:546)、modal があれば常に true。「independent of where the chain stops」は誤り。読み手 key.rs:69 のみ |  |
| `crates/kivact-core/src/view/semantics_data.rs:19` | SemanticsPatch.part | 文言 | option | P8-CO | 書き手 Common::semantic (modifiers.rs:53, false) と apply_part (node_modifiers.rs:844, true) のみ。patch の出所で「absent leaves the prop unchanged」は事実と違う。読み手 modifiers.rs:177 等, resolve.rs:115 | `` // xtask-allow(state-fields): option: set by apply_part on the patches a part declared, false on the caller's own; the *_source readers and part constraints tell the two apart `` |
| `crates/kivact-core/src/view/semantics_data.rs:24` | SemanticsPatch.reactive | 閉じ方の変更 | projection | P8-CO | 書き手 Common::semantic (modifiers.rs:51-58) のみ、source.fixed().is_none() の写し (source は apply に消去される)。builder が選ぶ値ではない。読み手 semantics.rs:54 | `` // xtask-allow(state-fields): projection: written only by Common::semantic from whether the patch's source is fixed; the source itself is erased into `apply` `` |
| `crates/kivact-core/src/view/semantics_data.rs:26` | SemanticsPatch.text | 誤（付録の設計） | type | P8-CO | text=true は semantics_text (node_modifiers.rs:783) が新 patch に付ける tag。relation (modifiers.rs:82)・orientation (:94)・input (:113-253) も各 helper が新 patch に 1 つだけ付ける排他 tag (SM-4)。SemanticInput の variant へ |  |
| `crates/kivact-core/src/view/semantics_data.rs:29` | SemanticsPatch.relation | 誤（付録の設計） | type | P8-CO | relation は semantic_relation (modifiers.rs:79-82) だけが付け、その patch の input=None・orientation=None・text=false。排他 tag (SM-4)。SemanticInput::Relation へ |  |
| `crates/kivact-core/src/view/semantics_data.rs:31` | SemanticsPatch.orientation | 誤（付録の設計） | type | P8-CO | orientation は semantic_orientation (modifiers.rs:90-94) だけが付け、他の tag とは排他 (SM-4)。読み手 container.rs:136 のみ。SemanticInput::Orientation へ |  |
| `crates/kivact-core/src/view/state.rs:50` | FocusRequest.visible | 文言 | independent | P8-CO | 書き手 request_focus の literal (requests.rs:44-49) のみで以後不変、stage は Queued→Sent。読み手 requests.rs:175。組合せは全て有効だが理由文が「誰が」を欠く | `` // xtask-allow(state-fields): independent: given once by the caller of request_focus and never changed; the stage moves from Queued to Sent on the host write whatever the indicator, so both values occur in both stages `` |
| `crates/kivact-core/src/view/state.rs:191` | Requests.next_request | 確認済み | identity | — | 書き手 request_id (state.rs:205-208) のみ。host request id の単調割当て |  |
| `crates/kivact-core/src/view/state.rs:197` | Requests.pending_activations | 文言 | independent | P8-CO | 書き手 request_activation (requests.rs:223-229) の push と take_activations (requests.rs:234) のみ。id counter とは無関係で全組合せ有効。理由文に書き手が無い | `` // xtask-allow(state-fields): independent: pushed by request_activation (NodeRef::activate) and taken whole by the client after each event; the id counter grows only on host requests, so any queue length goes with any id `` |
| `crates/kivact-core/src/view/state.rs:227` | ViewState.next_window | 確認済み | identity | — | 書き手 next_window() (state.rs:342-344) と初期化 Cell::new(1) (state.rs:312) のみ。window id 割当て |  |
| `crates/kivact-core/src/view/state.rs:268` | WindowRecord.min_size | 文言 | option | P8-CO | 書き手 runtime.rs:332 (App::min_window_size、main window) と set_window (window_view.rs:205、Window::min_size) のみ、実行時不変。理由文が main window の出所を欠く | `` // xtask-allow(state-fields): option: `Window::min_size`, or `App::min_window_size` for the main window; absent means the engine's minimum `` |
| `crates/kivact-core/src/view/tree.rs:76` | Handlers.secondary_context | 確認済み | option | — | 書き手 builder (node_modifiers.rs:201) のみ。merge_caller で合成されるが builder 時 |  |
| `crates/kivact-core/src/view/tree.rs:88` | Handlers.selection | 確認済み | option | — | 書き手 resolve.rs:71 (TextEditing::on_selection_change) と resolve.rs:207 (draw_with_layout 用の no-op) のみ、mount 時に一度 |  |
| `crates/kivact-core/src/view/window_view.rs:44` | Window.min_size | 確認済み | option | — | 書き手 window() (window_view.rs:30, None) と builder min_size (window_view.rs:67) のみ。WindowSlot へ移して open 時に読む |  |

### B.5 G5

| marker（HEAD） | owner.field | 判定 | 閉じ方 | unit | 根拠 | 新しい marker 行 |
|---|---|---|---|---|---|---|
| `crates/kivact-ui-soft/src/alert_dialog.rs:30` | AlertDialog.action | 確認済み | option | — | 書き手は alert_dialog.rs:52（None）と :63（AlertDialog::action）のみ。読み手 :113 は action ボタンの有無にだけ使う。open binding・loading と独立した caller 設定で absent＝ボタン無しが自… |  |
| `crates/kivact-ui-soft/src/alert_dialog.rs:32` | AlertDialog.cancel | 文言 | option | P8-SOFT | 書き手は :53（None）と :69（cancel_label）のみ。absent 時は :107 で i18n "Cancel" を使うが、理由文は誰が与えるか／absent の意味を述べていない。 | `` // xtask-allow(state-fields): option: caller label from AlertDialog::cancel_label; absent uses the localized "Cancel" message (soft.alert_dialog.cancel). `` |
| `crates/kivact-ui-soft/src/app_shell.rs:84` | MobileNavConfig.has_toggle | 閉じ方の変更 | option | P8-SOFT | 書き手は Default :97（true）と builder :109 のみ。実行時に書き換わらない caller 設定であり independent ではなく option。読み手は configure_mobile :369 で automatic と AND するだけ。 | `` // xtask-allow(state-fields): option: caller choice from MobileNavConfig::has_toggle (default true); false hides the automatic drawer toggle so the app opens the drawer through its own control or the open binding. `` |
| `crates/kivact-ui-soft/src/app_shell.rs:86` | MobileNavConfig.open | 文言 | option | P8-SOFT | 書き手は Default :98 と builder :114 のみ。absent 時は configure_mobile :354 が閉じた local signal を作る。理由文に absent の意味が無い。 | `` // xtask-allow(state-fields): option: caller binding from MobileNavConfig::open; absent makes configure_mobile create an initially closed local signal. `` |
| `crates/kivact-ui-soft/src/app_shell.rs:90` | MobileNavConfig.breakpoint | 文言 | option | P8-SOFT | 書き手は Default :101（Some(Md)）と builder :132 のみ。None は :349 で幅による mobile 切替を無効にする。理由文は absent(None) の意味を述べていない。 | `` // xtask-allow(state-fields): option: caller threshold from MobileNavConfig::breakpoint (default Md); None disables viewport-driven switching to the mobile layout. `` |
| `crates/kivact-ui-soft/src/app_shell.rs:329` | ShellPresentation.has_top | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手は into_view :253（top.is_some()）。caller が AppShell::top_nav で与えた slot の有無を一度だけ写した値で実行時に変化しない。independent の組合せ理由も無い→option。 | `` // xtask-allow(state-fields): option: whether the caller supplied AppShell::top_nav, captured once in into_view; false means the shell has no top navigation. `` |
| `crates/kivact-ui-soft/src/app_shell.rs:331` | ShellPresentation.has_side | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手は :254（side.is_some()）。AppShell::side_nav の有無の構築時の写しで、:531/:578 で表示分岐に読むだけ。independent の理由が無い→option。 | `` // xtask-allow(state-fields): option: whether the caller supplied AppShell::side_nav, captured once in into_view; false means the shell has no sidebar. `` |
| `crates/kivact-ui-soft/src/app_shell.rs:333` | ShellPresentation.has_banner | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手は :255（banner.is_some()）。AppShell::banner の有無の構築時の写し。:498 の header 可視判定に読むだけ。→option。 | `` // xtask-allow(state-fields): option: whether the caller supplied AppShell::banner, captured once in into_view; false means the shell has no banner. `` |
| `crates/kivact-ui-soft/src/bottom_sheet.rs:129` | BottomSheet.final_focus | 文言 | option | P8-SOFT | 書き手は :144（None）と :205（BottomSheet::final_focus）のみ。absent 時は modal overlay が opener へ戻す（:221 の注記）。理由文に absent の意味が無い。state-audit.md に行が無い（lint 再生成で要否確認）。 | `` // xtask-allow(state-fields): option: caller target from BottomSheet::final_focus; absent leaves focus restoration to the modal overlay's opener. `` |
| `crates/kivact-ui-soft/src/citation.rs:31` | CitationSource.title | 文言 | option | P8-SOFT | pub な application データで crate 内に書き手なし。読み手 citation.rs:107 は absent 時 url、さらに "Source N" へ fallback。理由文に absent の意味が無い。 | `` // xtask-allow(state-fields): option: application-supplied source title; absent falls back to the url, then to the numbered "Source N" label. `` |
| `crates/kivact-ui-soft/src/citation.rs:34` | CitationSource.url | 文言 | option | P8-SOFT | crate 内に書き手なし（application データ）。:121-122 で url が無いと on_open があっても非対話になる。理由文はこの意味を述べていない。 | `` // xtask-allow(state-fields): option: application-supplied destination; absent keeps the citation non-interactive even when on_open is set. `` |
| `crates/kivact-ui-soft/src/citation.rs:37` | CitationSource.image | 文言 | option | P8-SOFT | crate 内に書き手なし。citation_media :141 で image が icon より優先。"independent authored visual" は icon との優先関係を隠す。 | `` // xtask-allow(state-fields): option: application-supplied preview image; when present it is shown instead of icon. `` |
| `crates/kivact-ui-soft/src/citation.rs:40` | CitationSource.icon | 文言 | option | P8-SOFT | crate 内に書き手なし。:152 で image が absent の時だけ使う fallback。"independent" は事実と違う（image がある時は無視）。 | `` // xtask-allow(state-fields): option: application-supplied fallback glyph used only when image is absent; absent shows no media. `` |
| `crates/kivact-ui-soft/src/container_reveal.rs:36` | RevealContent.inverted | 閉じ方の変更 | option | P8-SOFT | pub な caller 構築値で crate 内に書き手なし。reveal_content :90/:116 で読むだけで実行時に変化しない→independent ではなく option。 | `` // xtask-allow(state-fields): option: caller policy in RevealContent; true reveals away from hover and keeps layout, false reveals on hover. `` |
| `crates/kivact-ui-soft/src/container_reveal.rs:39` | RevealContent.preserve_layout | 閉じ方の変更 | option | P8-SOFT | pub な caller 構築値、書き手なし。:117 で hidden 時の absolute 化を抑えるだけ（inverted 時は無関係）。→option。 | `` // xtask-allow(state-fields): option: caller policy in RevealContent; false lets hidden content leave normal flow once the fade ends. `` |
| `crates/kivact-ui-soft/src/container_reveal.rs:42` | RevealContent.force | 確認済み | option | — | pub な caller 構築値、書き手なし。:87-91 で None なら hover/focus/touch 判定。"override" で absent の意味が明らか。 |  |
| `crates/kivact-ui-soft/src/container_reveal.rs:45` | RevealContent.container_force | 確認済み | option | — | pub な caller 構築値、書き手なし。:81-84 で None なら遅延 hover を使う。override として正確。 |  |
| `crates/kivact-ui-soft/src/dialog_header.rs:37` | DialogHeader.subtitle | 確認済み | option | — | 書き手は :63（None）と :83（DialogHeader::subtitle）のみ、読み手 :160。once-mounted slot で absent＝無しが自明。 |  |
| `crates/kivact-ui-soft/src/dialog_header.rs:39` | DialogHeader.start | 確認済み | option | — | 書き手は :64 と :99（header_start）のみ。構築時 slot。 |  |
| `crates/kivact-ui-soft/src/dialog_header.rs:41` | DialogHeader.end | 確認済み | option | — | 書き手は :65 と :105（header_end）のみ。構築時 slot。 |  |
| `crates/kivact-ui-soft/src/dialog_header.rs:44` | DialogHeader.compensation | 閉じ方の変更 | option | P8-SOFT | 書き手は :67 と :111（header_end_compensation）のみの caller 設定→option。absent 時は :141-143 で close があれば All。 | `` // xtask-allow(state-fields): option: caller override from header_end_compensation; absent compensates both axes when a close button exists and nothing otherwise. `` |
| `crates/kivact-ui-soft/src/disclosure_appearance.rs:27` | DisclosureVariant.density | 文言 | option | P8-SOFT | pub な theme 作者設定、crate 内に書き手なし。absent は recipes_collections.rs:338 で dividers なら Balanced、そうでなければ padding 0。"component's default" は不正確。 | `` // xtask-allow(state-fields): option: theme-author padding choice; absent uses Balanced when dividers is set and no padding otherwise. `` |
| `crates/kivact-ui-soft/src/disclosure_appearance.rs:30` | DisclosureVariant.dividers | 文言 | option | P8-SOFT | pub な theme 作者設定、書き手なし。bool なので "absent" は誤り。false は区切り線なし（recipes_collections.rs:250）。 | `` // xtask-allow(state-fields): option: theme-author choice; false draws no separator between grouped panels. `` |
| `crates/kivact-ui-soft/src/item_values.rs:69` | ItemLook.density | 文言 | option | P8-SOFT | 書き手は Default :81 と builder :91 のみ。absent は soft_list.rs:6-13 で単独 ItemDensity variant→ListLook.density→Balanced の順に fallback。定型文は不正確。 | `` // xtask-allow(state-fields): option: theme-author row padding from ItemLook::density; absent falls back to an ItemDensity variant, then the ListLook density. `` |
| `crates/kivact-ui-soft/src/item_values.rs:71` | ItemLook.align | 文言 | option | P8-SOFT | 書き手は Default :82 と builder :96 のみ。absent は soft_list.rs:15-19 で ItemAlign variant→既定値へ fallback。 | `` // xtask-allow(state-fields): option: theme-author alignment from ItemLook::align; absent falls back to an ItemAlign variant, then ItemAlign::default. `` |
| `crates/kivact-ui-soft/src/item_values.rs:73` | ItemLook.layout | 文言 | option | P8-SOFT | 書き手は Default :83 と builder :101 のみ。absent は soft_list.rs:21-25 で ItemLayout variant→既定値へ fallback。 | `` // xtask-allow(state-fields): option: theme-author arrangement from ItemLook::layout; absent falls back to an ItemLayout variant, then ItemLayout::default. `` |
| `crates/kivact-ui-soft/src/layout.rs:9` | LayoutInset.outer_x | 閉じ方の変更 | option | P8-SOFT | 書き手は from_padding :34（AppShell/Card/Section の padding）と Default のみ。周囲の surface が与える context 値で実行時 phase なし→option。読み手 :451 は None で spacing.s4。 | `` // xtask-allow(state-fields): option: inline outer padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback. `` |
| `crates/kivact-ui-soft/src/layout.rs:12` | LayoutInset.outer_y | 閉じ方の変更 | option | P8-SOFT | 書き手は from_padding :35 と Default のみ。読み手 :452 は None で spacing.s4。→option。 | `` // xtask-allow(state-fields): option: block outer padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback. `` |
| `crates/kivact-ui-soft/src/layout.rs:15` | LayoutInset.inner_x | 閉じ方の変更 | option | P8-SOFT | 書き手は from_padding :36 と Default のみ。読み手 :455。→option。 | `` // xtask-allow(state-fields): option: inline gap padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback. `` |
| `crates/kivact-ui-soft/src/layout.rs:18` | LayoutInset.inner_y | 閉じ方の変更 | option | P8-SOFT | 書き手は from_padding :37 と Default のみ。読み手 :456。→option。 | `` // xtask-allow(state-fields): option: block gap padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback. `` |
| `crates/kivact-ui-soft/src/layout.rs:21` | LayoutInset.own_outer_x | 誤（付録の設計） | type（own_outer: Option<f64> 1 個に統合） | P8-SOFT | 唯一の書き手 layout.rs:210 が Layout.padding を写し、:211 で own_outer_y に同値を複写。2 つの Option が常に同値（SM-3）。G5.wrong.md W1。 | `` // xtask-allow(state-fields): option: this Layout's padding from Layout::padding, written only in Layout::into_view; absent uses the inherited outer_x and outer_y. `` |
| `crates/kivact-ui-soft/src/layout.rs:24` | LayoutInset.own_outer_y | 誤（付録の設計） | derive（own_outer に統合して削除） | P8-SOFT | 唯一の書き手 :211 が `insets.own_outer_y = insets.own_outer_x`。常に own_outer_x と同値（SM-3 より強い重複）。G5.wrong.md W1。 |  |
| `crates/kivact-ui-soft/src/layout.rs:173` | LayoutSlots.root_ref | 誤（付録の設計） | type（ContentWidth に統合） | P8-SOFT | 唯一の書き手 :219 は常に Some(node_ref)。None は context 不在の Default だけ。読み手 :510 は content_width Some の時だけ使う。caller 設定ではなく option は偽。G5.wrong.md W2。 |  |
| `crates/kivact-ui-soft/src/layout.rs:175` | LayoutSlots.header_divider | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :220。:214 で header があれば必ず Some に解決済み→None は header 無しを意味する caller 構成の写し。実行時変化なし→option。 | `` // xtask-allow(state-fields): option: header divider resolved at into_view from LayoutRegion::divider or Layout::default_dividers; None means the layout has no header. `` |
| `crates/kivact-ui-soft/src/layout.rs:177` | LayoutSlots.footer_divider | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :221、:214 で footer があれば Some。None は footer 無し。→option。 | `` // xtask-allow(state-fields): option: footer divider resolved at into_view from LayoutRegion::divider or Layout::default_dividers; None means the layout has no footer. `` |
| `crates/kivact-ui-soft/src/layout.rs:179` | LayoutSlots.has_start | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :222（start.is_some()）。caller の Layout::start の有無の構築時の写し→option。 | `` // xtask-allow(state-fields): option: whether the caller supplied Layout::start, captured once in into_view; false means no leading panel. `` |
| `crates/kivact-ui-soft/src/layout.rs:181` | LayoutSlots.has_end | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :223（end.is_some()）。caller の Layout::end の有無の写し→option。 | `` // xtask-allow(state-fields): option: whether the caller supplied Layout::end, captured once in into_view; false means no trailing panel. `` |
| `crates/kivact-ui-soft/src/layout.rs:183` | LayoutSlots.content_width | 誤（付録の設計） | type（Option<ContentWidth>） | P8-SOFT | caller 設定（Layout::content_width :144）自体は option で正しいが、root_ref と組でのみ意味を持つ（:415-417, :509-510）。W2 で ContentWidth{root,width} に統合し marker を書換え。 | `` // xtask-allow(state-fields): option: caller limit from Layout::content_width paired with the layout root ref used to centre pixel widths; absent means no width limit. `` |
| `crates/kivact-ui-soft/src/lightbox.rs:75` | Lightbox.zoom | 確認済み | option | — | 書き手は lightbox.rs:90（false）と :104（Lightbox::zoom）のみ。setup 後は不変で :253/:353 の分岐に読むだけ。 |  |
| `crates/kivact-ui-soft/src/link.rs:263` | LinkAppearance.size | 確認済み | option | — | 唯一の書き手 link.rs:195 が Link::text_size（:150）の値を写す。override で absent＝theme の size。 |  |
| `crates/kivact-ui-soft/src/link.rs:265` | LinkAppearance.weight | 確認済み | option | — | 唯一の書き手 :196 が Link::weight（:156）を写す。override で absent＝theme の weight。 |  |
| `crates/kivact-ui-soft/src/link.rs:267` | LinkAppearance.underline | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :197 が caller の Link::underline（:120）を into_view で一度 sample。実行時の内部 event で変化しない→option。 | `` // xtask-allow(state-fields): option: caller choice from Link::underline sampled once in into_view; false underlines only on hover of an interactive link. `` |
| `crates/kivact-ui-soft/src/link.rs:270` | LinkAppearance.standalone | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :199 が Link::standalone（:138）を一度 sample。:340 の typography fallback に読むだけ→option。 | `` // xtask-allow(state-fields): option: caller choice from Link::standalone sampled once in into_view; false inherits the surrounding typography. `` |
| `crates/kivact-ui-soft/src/link.rs:273` | LinkAppearance.interactive | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :182/:201（self.press.is_some()）。caller が on_press を与えたかの写しで不変→option。 | `` // xtask-allow(state-fields): option: whether the caller gave Link an activation callback; false paints a static link without pointer cursor or pressed state. `` |
| `crates/kivact-ui-soft/src/markdown.rs:807` | Marks.strong | 文言 | independent | P8-SOFT | 書き手 :794（inline_spans の heading 引数）と :849/:852（*** / ** / __）。入れ子で italic・deleted・code と全組合せが到達可能（**~~`x`~~**）。理由文が組合せの理由を述べていない。 | `` // xtask-allow(state-fields): independent: set by the caller's strong flag or a *** / ** / __ delimiter; nesting reaches it with any italic, deleted or code subset, and styled_span paints each mark separately. `` |
| `crates/kivact-ui-soft/src/markdown.rs:809` | Marks.italic | 文言 | independent | P8-SOFT | 書き手 :850/:853（*** / * / _）のみ。入れ子で他 mark と全組合せ到達。理由文は「独立」を言うだけ。 | `` // xtask-allow(state-fields): independent: set only by a *** / * / _ delimiter; delimiters nest in any order, so every combination with strong, deleted and code is parsed and painted. `` |
| `crates/kivact-ui-soft/src/markdown.rs:811` | Marks.deleted | 文言 | independent | P8-SOFT | 書き手 :854（~~）のみ。入れ子で他 mark と全組合せ到達。理由文が組合せの理由を述べていない。 | `` // xtask-allow(state-fields): independent: set only by a ~~ delimiter; it nests inside or around strong and italic and may wrap code, and styled_span adds the strike-through regardless of the other marks. `` |
| `crates/kivact-ui-soft/src/markdown.rs:813` | Marks.code | 文言 | independent | P8-SOFT | 書き手 :855（`）のみ。code 内は再帰しない（:857）が外側の strong/italic/deleted は保持→組合せは全て到達。理由文が不十分。 | `` // xtask-allow(state-fields): independent: set only by a backtick delimiter; code stops further parsing but keeps the enclosing strong, italic and deleted marks, so all those combinations are painted. `` |
| `crates/kivact-ui-soft/src/markdown_parser.rs:16` | ListEntry.number | 文言 | option | P8-SOFT | 唯一の書き手 :290（ordinal、:294 で兄弟ごとに +1）。Markdown 作者の `N.` から得る不変値。absent＝bullet を述べていない。 | `` // xtask-allow(state-fields): option: ordinal parsed from the Markdown author's `N.` marker and incremented per sibling; None means a bullet item. `` |
| `crates/kivact-ui-soft/src/markdown_parser.rs:18` | ListEntry.checked | 文言 | option | P8-SOFT | 唯一の書き手 :291（task() :170 の `[ ]`/`[x]`）。ordered/bullet いずれにも付く。absent の意味が無い。 | `` // xtask-allow(state-fields): option: task mark parsed from the author's `[ ]` / `[x]` prefix on ordered or bullet items alike; None means a plain item. `` |
| `crates/kivact-ui-soft/src/mobile_nav.rs:60` | MobileContext.auto_toggle | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 mobile_nav.rs:185 ← app_shell.rs:369（MobileNavMode::Auto && has_toggle）。構築時の caller 設定で machine state と無関係に不変→option。 | `` // xtask-allow(state-fields): option: fixed at AppShell construction as MobileNavMode::Auto with MobileNavConfig::has_toggle; false means the app supplies its own drawer control. `` |
| `crates/kivact-ui-soft/src/nav_heading.rs:45` | NavHeadingMenuItem.description | 文言 | option | P8-SOFT | 書き手は :57（None）と :67（description）のみ。absent は :474-476 で補足文なし。定型文は不正確。 | `` // xtask-allow(state-fields): option: caller supporting text from NavHeadingMenuItem::description; absent renders the label alone. `` |
| `crates/kivact-ui-soft/src/nav_heading.rs:48` | NavHeadingMenuItem.disabled | 文言 | option | P8-SOFT | 書き手は :59（false）と :77（disabled）のみ、読み手 :140。bool に "absent" は誤り。 | `` // xtask-allow(state-fields): option: caller availability from NavHeadingMenuItem::disabled; false keeps the entry selectable. `` |
| `crates/kivact-ui-soft/src/outline.rs:251` | Navigation.viewport | 文言 | option | P8-SOFT | 唯一の書き手 outline.rs:336 ← Outline::scroll_container（:142）。absent は :299 で target.scroll_into_view（window）。理由文に誰が与えるか／absent が無い。 | `` // xtask-allow(state-fields): option: caller viewport from Outline::scroll_container; absent scrolls the window through target.scroll_into_view. `` |
| `crates/kivact-ui-soft/src/outline.rs:254` | Navigation.scroll_on_press | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :338 が Outline::scroll_on_press（:152）を into_view で一度 sample。内部 event で変わらない caller 設定→option。 | `` // xtask-allow(state-fields): option: caller choice from Outline::scroll_on_press sampled once in into_view; false skips scrolling and reports navigation end immediately. `` |
| `crates/kivact-ui-soft/src/outline.rs:256` | Navigation.controlled | 確認済み | option | — | 唯一の書き手 :339（Outline の active binding）。navigate :273 で caller binding に書くだけで pending は ScrollMachine。理由文は事実どおり。 |  |
| `crates/kivact-ui-soft/src/power_search.rs:143` | FilterDraft.field | 文言 | independent | P8-SOFT | 書き手は Default :152、Edit :211、Field event :256 のみ。operator とは別の select で設定、Apply :220 が両方 Some を要求→欠けた組合せも編集中として有効。理由文が組合せの理由不足。 | `` // xtask-allow(state-fields): independent: set only by the Field select or seeded from the edited filter; operator is chosen separately and Apply requires both, so a draft missing either is a valid in-progress form. `` |
| `crates/kivact-ui-soft/src/power_search.rs:145` | FilterDraft.operator | 確認済み | independent | — | 書き手は Default :153、Edit :212、Operator event :257 のみ。「全 field が全 operator を許す」は field との組合せが全て有効な理由として具体的。 |  |
| `crates/kivact-ui-soft/src/recipes_metrics.rs:62` | MetadataAppearance.position | 文言 | option | P8-SOFT | 書き手は Default :71 と MetadataLook::label_position :103 のみ。absent は :145-148 で単一列 Start／複数列 Top。定型文は不正確。 | `` // xtask-allow(state-fields): option: theme-author placement from MetadataLook::label_position; absent places terms beside details in one column and above them otherwise. `` |
| `crates/kivact-ui-soft/src/recipes_metrics.rs:64` | MetadataAppearance.width | 文言 | option | P8-SOFT | 書き手は Default :72 と label_width :115 のみ。absent は :191 で max-content の term 列幅。 | `` // xtask-allow(state-fields): option: theme-author term column width from MetadataLook::label_width; absent sizes the term column to its max-content width. `` |
| `crates/kivact-ui-soft/src/rich_block_doc.rs:24` | Kind::List.number | 文言 | independent | P8-SOFT | 書き手は parse :315 と split :182（+1）。checked と全 4 組合せが source から到達可能だが理由文は意味の説明だけ。 | `` // xtask-allow(state-fields): independent: parsed from the `N.` marker and incremented on split; a task checkbox may sit on ordered or bullet items, so every number and checked pair comes from valid source. `` |
| `crates/kivact-ui-soft/src/rich_block_doc.rs:26` | Kind::List.checked | 文言 | independent | P8-SOFT | 書き手は parse :315、split :183（false へ）、Document::task :150（head の [x] と同時に書換え）。number と独立だが理由文が組合せの理由不足。 | `` // xtask-allow(state-fields): independent: parsed from the `[ ]` / `[x]` prefix and toggled only by Document::task, which rewrites the head text in the same step; numbering never constrains it. `` |
| `crates/kivact-ui-soft/src/side_navigation/items.rs:402` | ItemSlots.leading | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 items.rs:191 ← SideNavItem::icon（:105）。caller slot で "measured" は誤り→option。 | `` // xtask-allow(state-fields): option: caller icon from SideNavItem::icon, mounted once; absent means the row has no leading icon. `` |
| `crates/kivact-ui-soft/src/side_navigation/items.rs:404` | ItemSlots.selected_icon | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :192 ← SideNavItem::selected_icon（:111）。absent は :564-575 で leading を選択時も表示。→option。 | `` // xtask-allow(state-fields): option: caller icon from SideNavItem::selected_icon, mounted once; absent keeps the leading icon while selected. `` |
| `crates/kivact-ui-soft/src/side_navigation/items.rs:406` | ItemSlots.end | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :193 ← SideNavItem::end_content（:126）。caller slot→option。 | `` // xtask-allow(state-fields): option: caller content from SideNavItem::end_content, mounted once; absent means no trailing row content. `` |
| `crates/kivact-ui-soft/src/side_navigation/items.rs:408` | ItemSlots.actions | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :194 ← SideNavItem::actions（:131）。caller slot→option。 | `` // xtask-allow(state-fields): option: caller controls from SideNavItem::actions, mounted once; absent means no row action controls. `` |
| `crates/kivact-ui-soft/src/side_navigation/shell.rs:376` | SideNavSlots.header | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 shell.rs:248 ← SideNav::header（:147）。caller slot で "measured" は誤り→option。 | `` // xtask-allow(state-fields): option: caller content from SideNav::header, mounted once; absent means no sidebar header. `` |
| `crates/kivact-ui-soft/src/side_navigation/shell.rs:378` | SideNavSlots.top | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :249 ← SideNav::top_content（:152）。→option。 | `` // xtask-allow(state-fields): option: caller content from SideNav::top_content, mounted once; absent means nothing above the entries. `` |
| `crates/kivact-ui-soft/src/side_navigation/shell.rs:380` | SideNavSlots.footer | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :250 ← SideNav::footer（:157）。→option。 | `` // xtask-allow(state-fields): option: caller content from SideNav::footer, mounted once; absent means no footer content. `` |
| `crates/kivact-ui-soft/src/side_navigation/shell.rs:382` | SideNavSlots.icons | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :251 ← SideNav::footer_icons（:162）。→option。 | `` // xtask-allow(state-fields): option: caller actions from SideNav::footer_icons, mounted once; absent means no compact rail actions. `` |
| `crates/kivact-ui-soft/src/soft_chat_tools.rs:81` | ToolCall.target | 閉じ方の変更 | option | P8-SOFT | 書き手は new :97 と builder :109 のみ（application 記録）。status に関係なく :272-273 で表示/非表示→application が与える値で option。 | `` // xtask-allow(state-fields): option: application-supplied target text shown beside the tool name in every status; absent hides the target label. `` |
| `crates/kivact-ui-soft/src/soft_tree.rs:8` | TreeLook.density | 文言 | option | P8-SOFT | 書き手は Default と builder :16 のみ。absent は recipes_collections.rs:75 で Balanced。定型文は不正確。 | `` // xtask-allow(state-fields): option: theme-author row padding from TreeLook::density; absent uses ItemDensity::Balanced. `` |
| `crates/kivact-ui-soft/src/soft_tree.rs:10` | TreeLook.guides | 文言 | option | P8-SOFT | 書き手は Default と builder :21 のみ。absent は recipes_collections.rs:87 で guide を表示（Some(true) と同じ）。 | `` // xtask-allow(state-fields): option: theme-author choice from TreeLook::guides; absent shows ancestor guides like Some(true). `` |
| `crates/kivact-ui-soft/src/stack.rs:77` | Padding.all | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 stack.rs:151（padding_options! の padding）。builder の authored 値で実行時 phase なし→option。読み手 resolve :135 で最も弱い fallback。 | `` // xtask-allow(state-fields): option: caller value from the padding builder; absent leaves edges without x, y or edge values on the container default. `` |
| `crates/kivact-ui-soft/src/stack.rs:79` | Padding.x | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :156（padding_x）。resolve :135 で start/end の次、all より優先。→option。 | `` // xtask-allow(state-fields): option: caller value from padding_x; absent defers inline edges to padding, then the container default. `` |
| `crates/kivact-ui-soft/src/stack.rs:81` | Padding.y | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :161（padding_y）。→option。 | `` // xtask-allow(state-fields): option: caller value from padding_y; absent defers top and bottom to padding, then the container default. `` |
| `crates/kivact-ui-soft/src/stack.rs:83` | Padding.start | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :166（padding_start）。→option。 | `` // xtask-allow(state-fields): option: caller value from padding_start; absent defers the leading edge to padding_x, then padding, then the container default. `` |
| `crates/kivact-ui-soft/src/stack.rs:85` | Padding.end | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :171（padding_end）。→option。 | `` // xtask-allow(state-fields): option: caller value from padding_end; absent defers the trailing edge to padding_x, then padding, then the container default. `` |
| `crates/kivact-ui-soft/src/stack.rs:87` | Padding.top | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :176（padding_top）。→option。 | `` // xtask-allow(state-fields): option: caller value from padding_top; absent defers the top edge to padding_y, then padding, then the container default. `` |
| `crates/kivact-ui-soft/src/stack.rs:89` | Padding.bottom | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :181（padding_bottom）。→option。 | `` // xtask-allow(state-fields): option: caller value from padding_bottom; absent defers the bottom edge to padding_y, then padding, then the container default. `` |
| `crates/kivact-ui-soft/src/tab_list.rs:102` | Tab.icon | 確認済み | option | — | 書き手は tab_list.rs:119（None）と :130（Tab::icon）のみ。理由文が設定者と absent を述べている。 |  |
| `crates/kivact-ui-soft/src/tab_list.rs:104` | Tab.selected_icon | 確認済み | option | — | 書き手は :120 と :135 のみ。absent で resting icon を使う（:921）と一致。 |  |
| `crates/kivact-ui-soft/src/tab_list.rs:107` | Tab.end | 確認済み | option | — | 書き手は :122 と :145（end_content）のみ。 |  |
| `crates/kivact-ui-soft/src/tab_list.rs:109` | Tab.panel | 確認済み | option | — | 書き手は :123 と :150（Tab::panel）のみ。 |  |
| `crates/kivact-ui-soft/src/tab_list.rs:179` | TabMenuOption.disabled | 閉じ方の変更 | option | P8-SOFT | 書き手は :190（false）と :200（TabMenuOption::disabled）のみ、読み手 :719。caller 設定で independent の組合せ理由も無い→option。state-audit.md に行なし。 | `` // xtask-allow(state-fields): option: caller availability from TabMenuOption::disabled; false keeps the option selectable in the strip menu. `` |
| `crates/kivact-ui-soft/src/tab_list.rs:862` | TabRuntime.behavior | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :472-478 ← tabs_behavior :798（caller の TabPattern が Tabs の時だけ Some）。構築時の caller 設定の写し→option。 | `` // xtask-allow(state-fields): option: TabsApi created only when the caller's TabPattern is Tabs; None makes the strip a Navigation landmark without tab semantics. `` |
| `crates/kivact-ui-soft/src/tab_list.rs:865` | TabRuntime.scrollable | 閉じ方の変更 | option | P8-SOFT | 唯一の書き手 :474（overflow.get()==Scroll を一度 sample）。caller の TabOverflow 設定→option。 | `` // xtask-allow(state-fields): option: caller TabOverflow sampled once in into_view; false lets the strip overflow without a scrolling viewport. `` |
| `crates/kivact-ui-soft/src/timestamp.rs:163` | TimestampEntry.zone | 確認済み | option | — | pub な application 値、crate 内に書き手なし。:636 で absent は main zone。override として正確。 |  |
| `crates/kivact-ui-soft/src/timestamp.rs:168` | TimestampEntry.label | 確認済み | option | — | pub な application 値、書き手なし。:641-643 で absent は値のみ表示。authored text として正確。 |  |
| `crates/kivact-ui-soft/src/timestamp.rs:171` | TimestampEntry.copyable | 文言 | option | P8-SOFT | P8 統合で上書き：読み手が無い（copy 操作が未実装）のは事実。ただし pub field の削除は kit の公開 API の縮小にあたるので 8 章で人間の判断に回し、V2 では field を残して事実どおりの option にする。 元の判定：repo 全体で読み手が皆無（grep "copyable" は timestamp.rs:173 の定義のみ）。copy 操作は実装されておらず、marker の「Copy permission」は事実無根。G5.wrong.md W3。 | `` // xtask-allow(state-fields): option: the caller's flag for a copy action on this detail row; timestamp_details does not draw a copy action yet, so both values render alike `` |
| `crates/kivact-ui-soft/src/top_nav.rs:63` | Slots.heading | 文言 | option | P8-SOFT | 書き手は Default と TopNav::heading :95 のみ。absent は heading 無し。"component's default" は不正確。 | `` // xtask-allow(state-fields): option: caller content from TopNav::heading, mounted once; absent leaves the heading area empty. `` |
| `crates/kivact-ui-soft/src/top_nav.rs:65` | Slots.start | 文言 | option | P8-SOFT | 書き手は Default と TopNav::start :100 のみ。MobileBar では :140 で捨てる。 | `` // xtask-allow(state-fields): option: caller content from TopNav::start, mounted once and dropped in the mobile bar; absent leaves the start area empty. `` |
| `crates/kivact-ui-soft/src/top_nav.rs:67` | Slots.center | 文言 | option | P8-SOFT | 書き手は Default と TopNav::center :105 のみ。MobileBar では :145 で捨てる。 | `` // xtask-allow(state-fields): option: caller content from TopNav::center, mounted once and dropped in the mobile bar; absent leaves the center area empty. `` |
| `crates/kivact-ui-soft/src/top_nav.rs:69` | Slots.end | 文言 | option | P8-SOFT | 書き手は Default と TopNav::end :110 のみ。 | `` // xtask-allow(state-fields): option: caller content from TopNav::end, mounted once; absent leaves the end area empty. `` |

### B.6 G6

| marker（HEAD） | owner.field | 判定 | 閉じ方 | unit | 根拠 | 新しい marker 行 |
|---|---|---|---|---|---|---|
| `crates/arvane-asset/src/store/asset_store.rs:72` | AssetStore.next_id | 確認済み | identity | — | writers: asset_store.rs:106（初期 1）, :637-638（画像ごとの採番）のみ。単なる id 割当てで phase を持たない |  |
| `crates/arvane-asset/src/store/asset_store.rs:74` | AssetStore.next_generation | 確認済み | identity | — | writers: asset_store.rs:107, :200-201, :368-369 のみ。Entry.generation の採番器で照合は :447/:504/:519。同じ owner に走行中を表す bool/Option は無く SM-6 に当たらない |  |
| `crates/arvane-asset/src/store/job.rs:57` | Job::Raster.generation | 確認済み | identity | — | writer: asset_store.rs:428-434（start_raster が entry.generation を刻印）のみ。job の刻印で phase を隠さない |  |
| `crates/arvane-asset/src/store/job.rs:61` | Job::Raster.crop | 確認済み | option | — | writer: asset_store.rs:434 crop: key.crop（呼び手が要求した VariantKey の crop）のみ。job の入力値で内部 event による書換え無し |  |
| `crates/arvane-asset/src/store/job.rs:76` | Done::Raster.generation | 確認済み | identity | — | job.rs:136-146 で Job::Raster の generation を写すだけ。asset_store.rs:519 で entry と照合して古い結果を捨てる（entry 側に走行中 flag 無し） |  |
| `crates/arvane-asset/src/store/job.rs:79` | Done::Raster.crop | 確認済み | option | — | job.rs で Job::Raster の crop を写すだけ。asset_store.rs:520-525 で VariantKey を作り直すのに使う |  |
| `crates/arvane-compose/src/gpu/state.rs:28` | GpuEnv.backends | 文言 | option | P8-ARV | writer: state.rs:41 from_env のみ（WGPU_BACKEND を一度読む）、reader :61。option は正しいが理由が変数名だけで absent の意味を述べていない | `` // xtask-allow(state-fields): option: the backends the `WGPU_BACKEND` environment variable names, read once by `from_env`; absent when it is unset, and then the application's `GpuPreference::backend` decides `` |
| `crates/arvane-compose/src/gpu/state.rs:31` | GpuEnv.power | 文言 | option | P8-ARV | writer: state.rs:42 from_env のみ（WGPU_POWER_PREF）、reader :52。option は正しいが理由が変数名だけ | `` // xtask-allow(state-fields): option: the power preference the `WGPU_POWER_PREF` environment variable names, read once by `from_env`; absent when it is unset, and then the application's `GpuPreference::power` decides `` |
| `crates/arvane-compose/src/plan/bounds.rs:26` | Kind::Group.clip | 確認済み | option | — | writer: bounds.rs:96-99 group_kind のみ（scene の Group.clip から）。authored な scene の値で blur と独立 |  |
| `crates/arvane-compose/src/plan/bounds.rs:28` | Kind::Group.blur | 確認済み | option | — | writer: bounds.rs:96-99 group_kind のみ（scene の Group.blur から）。authored な scene の値で clip と独立 |  |
| `crates/arvane-compose/src/plan/ops.rs:145` | Source::External.cpu | 文言 | option | P8-ARV | writer: items.rs:93 のみ（ExternalItem.cpu = app が Draw::gpu_frame_with_pixels で与える, composition.rs:265）。『CPU が描くとき present』は誤り: GPU でも Some があり、CPU は None なら missed（cpu/draw.rs:259-265） | `` // xtask-allow(state-fields): option: the fallback pixels the application gave with the frame (`Draw::gpu_frame_with_pixels`), which only the CPU compositor draws; absent when it gave none, and the CPU compositor then reports the frame missed `` |
| `crates/arvane-compose/src/plan/ops.rs:148` | Source::External.source | 確認済み | option | — | writer: items.rs:94 のみ（ExternalItem.source、app が与える）。cpu と独立な authored 値 |  |
| `crates/arvane-native/src/mirror.rs:106` | Counters.created | 確認済み | statistic | — | writers: mirror.rs:449/456/554 の fetch_add のみ。reader: stats() :226 → display_probe.rs:14 → real_window_tests.rs:204-208（試験のみ）。振る舞いを分岐させない |  |
| `crates/arvane-native/src/mirror.rs:108` | Counters.reused | 確認済み | statistic | — | writers: mirror.rs:449/456/554 の fetch_add のみ。reader: stats() :226 → display_probe.rs:14 → real_window_tests.rs:204-208（試験のみ）。振る舞いを分岐させない |  |
| `crates/arvane-native/src/mirror.rs:110` | Counters.presented | 確認済み | statistic | — | writers: mirror.rs:449/456/554 の fetch_add のみ。reader: stats() :226 → display_probe.rs:14 → real_window_tests.rs:204-208（試験のみ）。振る舞いを分岐させない |  |
| `crates/arvane-native/src/mirror.rs:117` | MirrorBatch.frame | 確認済み | option | — | writer: mirror.rs:320 present() のみ（test が与える frame）、flush :549 で take。pending batch の値 |  |
| `crates/arvane-native/src/mirror.rs:119` | MirrorBatch.accessibility | 文言 | independent | P8-NA | writer: mirror.rs:214（tree 変更 callback）のみ。frame/title/resize/gpu は別の MirrorWindow 呼出し（:320/:338/:344/:368）。flush :528-555 は各々独立に適用。理由が定型文 | `` // xtask-allow(state-fields): independent: set only by the accessibility tree's change callback, while frame, title, resize and gpu come from other MirrorWindow calls on the test thread; flush applies each on its own, so any mix in one batch is valid `` |
| `crates/arvane-native/src/mirror.rs:122` | MirrorBatch.title | 確認済み | option | — | writer: mirror.rs:338 set_title のみ（test が与える）、flush :528 で適用 |  |
| `crates/arvane-native/src/mirror.rs:124` | MirrorBatch.resize | 確認済み | option | — | writer: mirror.rs:344 resize のみ（test が与える）、flush :531 で適用 |  |
| `crates/arvane-native/src/mirror.rs:126` | MirrorBatch.gpu | 文言 | independent | P8-NA | writer: mirror.rs:368 gpu_presents のみ。flush :541-547 で Use::Gpu に切替え、同じ batch の frame は保持されない（:550）。全組合せ有効だが理由が定型文 | `` // xtask-allow(state-fields): independent: set only by `MirrorWindow::gpu_presents`, while the other batch values come from other MirrorWindow calls; flush switches the window to GPU use on its own (a frame in the same batch is then not kept), so any mix is valid `` |
| `crates/arvane-native/src/module.rs:15` | (no audit row: clippy::disallowed_types,) | 誤（付録の設計） | type（W1） | P8-NA | W1。MAPPING は module.rs:32 で初期化、REPORTED は :35 で直後に set。REPORTED=true ⇒ MAPPING 初期化済の含意（未初期化+true は到達不能）。順序のある状態が二つの static に分かれている | `` // xtask-allow(state-fields): cache: the loader's answer for keeping this module mapped (`retain`); asking again gives the same answer, so it is asked once per process and never invalidated `` |
| `crates/arvane-native/src/module.rs:22` | (no audit row: static REPORTED: AtomicBool = AtomicBool::new(false);) | 誤（付録の設計） | type（W1） | P8-NA | W1。計数でなく一度きりの latch で、module.rs:35 の swap が log の分岐を制御するので statistic は誤り。MAPPING との含意あり | `` // xtask-allow(state-fields): independent: set by the first `retain_module` call before it logs, whatever the loader answered; a reentrant call from the log subscriber sees it set and returns, so both answers occur with it unset and set `` |
| `crates/arvane-native/src/native_input/state.rs:15` | NativeInput.text | 文言 | independent | P8-NA | writers: win32/ime.rs:165, winit/window.rs:486（ImePort::surrounding、app の IME text）のみ。reduced_motion/locale は OS 設定（windows.rs:239-242, window_events.rs:401）。理由が抽象的 | `` // xtask-allow(state-fields): independent: written only by the IME port's `surrounding` when the application focuses or edits a text input, while reduced_motion and locale come from the system's settings; neither source reads the other, so every pair occurs `` |
| `crates/arvane-native/src/native_input/state.rs:31` | NativeInput.reduced_motion | 文言 | cache | P8-NA | 正本は OS 設定。再読込: windows.rs:239（WM_SETTINGCHANGE）, runtime.rs:385-389（ENVIRONMENT wake）、初期 open.rs:67。cache は正しいが無効化の契機を書くべき | `` // xtask-allow(state-fields): cache: the system's reduce-motion preference as last read; read again on `WM_SETTINGCHANGE` (Windows) and on the settings portal's environment wake (Linux) `` |
| `crates/arvane-native/src/native_input/state.rs:36` | NativeInput.locale | 閉じ方の変更 | cache | P8-NA | writers: windows.rs:242（WM_SETTINGCHANGE）, window_events.rs:401（Focused(true)）で OS から再読込、初期 open.rs:69。内部 event で書き換わる実行時の値なので option は誤り。reduced_motion と同じ cache | `` // xtask-allow(state-fields): cache: the system's locale as last read (`system_locale`); read again on `WM_SETTINGCHANGE` and when the window gains focus; absent when the system reports none `` |
| `crates/arvane-native/src/native_input/state.rs:62` | ContactNote.canceled | 文言 | independent | P8-NA | writers: windows.rs:334（POINTER_FLAG_CANCELED）, :194（message の flag を OR）。tilt は :330 で pen のときだけ。cancel は touch/pen どちらにも起こり全組合せ到達。理由が具体性不足 | `` // xtask-allow(state-fields): independent: the platform's CANCELED flag for the contact's pointer message; tilt comes from the pen info of the same message, and a touch (no tilt) or a pen (with or without tilt) can each be canceled, so every pair occurs `` |
| `crates/arvane-native/src/native_input/state.rs:65` | ContactNote.tilt | 確認済み | option | — | writer: windows.rs:330-335（pen のとき GetPointerPenInfo の値）のみ。platform が与える event の値。参考: tilt Some ⇒ kind==Pen の含意があるが kind は数えない field |  |
| `crates/arvane-native/src/port/embed/child.rs:32` | Inner.generation | 確認済み | identity | — | writers: child.rs:77（0）, :210（queue_host_size で +1）のみ、照合 :228-229。走行状態は ChildStatus 型で別にあり、host size の epoch（同時に複数の交渉が飛ぶ）なので SM-6 に当たらない |  |
| `crates/arvane-native/src/port/linux_settings.rs:21` | (no audit row: static REDUCED_MOTION: AtomicBool = AtomicBool::new(false);) | 確認済み | cache | — | 正本は settings portal。writer: linux_settings.rs:96 refresh（SettingChanged ごと）のみ、reader :36。無効化は SettingChanged signal |  |
| `crates/arvane-native/src/port/linux_settings.rs:28` | (no audit row: static NEXT: AtomicU64 = AtomicU64::new(0);) | 確認済み | identity | — | writer: linux_settings.rs:55 fetch_add のみ。Watch の登録番号の採番 |  |
| `crates/arvane-native/src/port/win32/state.rs:59` | Utf16Decoder.pending | 誤（付録の設計） | type（W2 で marker 削除） | P8-NA | W2 の一部。pending 自体は正しい（:66-73）が tail と排他（tail を設定する時 pending は :72 で take 済）。W2 で tail を消すと数える field が 1 個になり marker は削除 |  |
| `crates/arvane-native/src/port/win32/state.rs:61` | Utf16Decoder.tail | 誤（付録の設計） | derive（W2） | P8-NA | W2。tail は state.rs:78 で設定され input.rs:152 で同じ WM_CHAR 処理内に必ず take。呼出し間は常に None の戻り値の退避。『第 2 unit を待つ文字』は事実と違う |  |
| `crates/arvane-native/src/port/winit/runtime.rs:123` | Runner.wake_reasons | 閉じ方の変更 | independent | P8-NA | writers: runtime.rs:504（waker が任意 thread で fetch_or）, :353（user_event が swap 0）のみ。内部 event で変わる実行時の集積で option は誤り。どの LoopPhase でも wake は来て drain される | `` // xtask-allow(state-fields): independent: bits OR-ed in by the ShellWaker on any thread and swapped to zero by `user_event`; wakes arrive in every LoopPhase (before the main window, while running, after a stop) and are drained in each, so every pair with phase occurs `` |
| `crates/arvane-native/src/port/winit/window.rs:34` | WinitWindow.subclass | 確認済み | option | — | writer: open.rs:78（構築時の Subclass::install）のみ、reader window.rs:447。以後書換え無し、drop 順のための保持 |  |
| `crates/arvane-native/src/port/winit/window.rs:52` | WinitWindow.monitor | 文言 | cache | P8-NA | writers: open.rs:75/88（open 時 current_monitor）, window_events.rs:409（Moved/ScaleFactorChanged）のみ。『最初の報告前は absent』は誤り: open 時に読み、None は platform が monitor を返さないとき | `` // xtask-allow(state-fields): cache: the monitor `current_monitor` gave at open and at the last move or scale change, kept to tell when frame_period_ns must be read again; absent when the platform cannot tell the monitor `` |
| `crates/arvane-native/src/port/winit/windows.rs:24` | WindowTable.pending_show | 文言 | independent | P8-NA | writers: shell_loop.rs:57 push（callback 中の open）, runtime.rs:187 take（callback 後）のみ。placement は runtime.rs:469 で構築時固定。組合せは有効だが理由が具体性不足 | `` // xtask-allow(state-fields): independent: filled when the handler opens a window during a callback and emptied by the runner once it returns; placement is fixed when the table is made, so windows wait to be shown on platforms with and without placement alike `` |
| `crates/arvane-native/src/port/winit/windows.rs:26` | WindowTable.placement | 確認済み | option | — | writer: windows.rs:33-38 new のみ（runtime.rs:469 で placement_supported から）。platform の能力で構築時固定 |  |
| `crates/arvane-native/src/port/x11/dnd.rs:48` | Drag::Hovering.accepts | 文言 | independent | P8-NA | writers: dnd.rs:397-425（enter で決定）, :466-469（position で引継ぎ）のみ。point は position ごとに受否と無関係に更新（拒否の status も送る :476-485）。4 組合せ到達 | `` // xtask-allow(state-fields): independent: decided once at XdndEnter from the protocol version and the offered types; point is filled by every XdndPosition whether or not we accept (a refusing status is still sent), so all four pairs occur `` |
| `crates/arvane-native/src/port/x11/dnd.rs:50` | Drag::Hovering.point | 閉じ方の変更 | independent | P8-NA | writers: dnd.rs:425（enter で None）, :470（position で Some）。XdndPosition で書き換わる実行時の値で option は誤り。accepts と全組合せ有効（drop は :347 で (true,Some) のみ受理） | `` // xtask-allow(state-fields): independent: the last XdndPosition, absent until the first one arrives (no status sent yet); it is filled whatever accepts says, and drop takes the URIs only for accepts with a point, so all four pairs occur `` |
| `crates/arvane-native/src/port/x11/notifier.rs:10` | (no audit row: pub(super) reasons: AtomicU8,) | 閉じ方の変更 | independent | P8-NA | writers: notifier.rs:27 request（任意 thread で fetch_or）, :32 take（UI thread で swap 0）のみ。実行時の集積で option は誤り | `` // xtask-allow(state-fields): independent: bits OR-ed in by `request` on any thread and swapped to zero by `take` on the UI thread; a request in any UiThread state only sets bits, which wait until the UI loop takes them `` |
| `crates/arvane-render/src/draw/canvas.rs:77` | GlyphRun.hint | 誤（付録の設計） | type（W3） | P8-ARV | W3。writers: painter.rs:704-705, :734-735, :751-752, :770-771。(hint,cached) は (true,solid) か (false,false) だけ: cached ⇒ hint の含意、(false,true) は到達不能 |  |
| `crates/arvane-render/src/draw/canvas.rs:80` | GlyphRun.cached | 誤（付録の設計） | type（W3） | P8-ARV | W3。同上。cached の理由文自身が hint と同じ条件（translation と uniform scale）を書いており含意を示している。reader encode.rs:301 |  |
| `crates/arvane-render/src/paint.rs:49` | PatternFill.repeat_x | 確認済み | option | — | writer: paint.rs:100-101（scene の Pattern から写すだけ）のみ。authored な paint の値で x/y 独立。readers cpu.rs:287-288, encode.rs:354-355/390-391 |  |
| `crates/arvane-render/src/paint.rs:52` | PatternFill.repeat_y | 確認済み | option | — | writer: paint.rs:100-101（scene の Pattern から写すだけ）のみ。authored な paint の値で x/y 独立。readers cpu.rs:287-288, encode.rs:354-355/390-391 |  |
| `crates/arvane-shell/src/a11y/shared.rs:155` | PlatformUpdate.focus | 誤（付録の設計） | derive（W4） | P8-ARV | W4。writers: shared.rs:260, :335 ともに focus: tree.focus()。同じ struct の tree（AcceptedTree::focus, state.rs:64）から導出できる複写 |  |
| `crates/arvane-shell/src/a11y/shared.rs:159` | PlatformUpdate.all_runs | 誤（付録の設計） | type（W4） | P8-ARV | W4。writers: shared.rs:263（true, runs=&[]）, :305-327（Tree→true/&[], Nothing→false/&[], Changed→merged/&[] か runs）。all_runs=true ⇒ runs 空の含意（SM-2）。reader native.rs:216-221 |  |
| `crates/arvane-shell/src/clicks.rs:40` | ClickCounter.last | 文言 | independent | P8-ARV | writer: clicks.rs mouse の Down（:139）のみ、tap は touch/pen の Down（:107）のみ。どちらの device が先でも各 None/Some が他と組み合わさる。理由が定型的 | `` // xtask-allow(state-fields): independent: written only by a mouse `Down`, while tap is written only by a touch or pen `Down`; a window may see either device first or only one of them, so each of None and Some occurs with any value of the other `` |
| `crates/arvane-shell/src/clicks.rs:46` | ClickCounter.tap | 文言 | independent | P8-ARV | writer: clicks.rs:107（touch/pen の Down）のみ。last は mouse の Down（:139）のみ。理由が具体性不足 | `` // xtask-allow(state-fields): independent: written only by a touch or pen `Down`, while last is written only by a mouse `Down`; a window may see either device first or only one of them, so each of None and Some occurs with any value of the other `` |
| `crates/arvane-shell/src/ime.rs:18` | ShellImeText.password | 誤（付録の設計） | derive（W5：field 削除） | P8-EN | W5。writers: text_geometry.rs:247（true, composition None, text 空）, :265, session.rs:139-148。password ⇒ composition None で理由文は事実と逆。読み手無し（purpose は set_ime_purpose で別に渡る winit/window.rs:351） |  |
| `crates/arvane-shell/src/ime.rs:32` | ShellImeText.composition | 確認済み | option | — | writers: text_geometry.rs:271（edit.marked() から）、他は None。engine が与える snapshot の値。W5 適用後は数える field が 1 個になり marker 削除 |  |
| `crates/arvane-shell/src/window.rs:272` | StepOutcome::Idle.next_deadline | 閉じ方の変更 | independent | P8-ARV | writer: virtual backend.rs:310 control.deadline() のみ（turn 後の実行時の値）。authored でないので option は誤り。redraw/woken とは別の源 | `` // xtask-allow(state-fields): independent: the loop control's earliest timer deadline when the turn ends; timers come from the handler, apart from window redraw flags and wakes, so it is present or absent with any mix of them `` |
| `crates/arvane-shell/src/window.rs:274` | StepOutcome::Idle.redraw_requested | 文言 | independent | P8-ARV | writer: backend.rs:311-315（turn 後に各 window の redraw flag を集計）のみ。『turn 中に要求した』は不正確: turn 終了時に未配達の要求があるか | `` // xtask-allow(state-fields): independent: whether any window's redraw flag is still set when the turn ends; windows set it from their own requests, apart from the timer deadline and pending wakes, so a turn may end with any mix `` |
| `crates/arvane-shell/src/window.rs:277` | StepOutcome::Idle.woken | 文言 | independent | P8-ARV | writer: backend.rs:316 woken.pending()（turn 後に未処理の wake があるか）のみ。『turn が wake を処理した』は事実と逆 | `` // xtask-allow(state-fields): independent: whether a wake is still pending when the turn ends (`WakePending::pending`); wakers set it from any thread, apart from the timer deadline and redraw flags, so a turn may end with any mix `` |
| `crates/arvane-text/src/cache.rs:115` | SpanBits.font_size | 確認済み | cache | — | SpanBits は span_bits（cache.rs:587-）だけが span style と drawn から作る layout cache の key 成分。equality/hash のみに使う（Equivalent :181-）。構築後の書換え無し |  |
| `crates/arvane-text/src/cache.rs:117` | SpanBits.weight | 確認済み | cache | — | SpanBits は span_bits（cache.rs:587-）だけが span style と drawn から作る layout cache の key 成分。equality/hash のみに使う（Equivalent :181-）。構築後の書換え無し |  |
| `crates/arvane-text/src/cache.rs:119` | SpanBits.color | 確認済み | cache | — | SpanBits は span_bits（cache.rs:587-）だけが span style と drawn から作る layout cache の key 成分。equality/hash のみに使う（Equivalent :181-）。構築後の書換え無し |  |
| `crates/arvane-text/src/cache.rs:121` | SpanBits.stroke | 確認済み | cache | — | SpanBits は span_bits（cache.rs:587-）だけが span style と drawn から作る layout cache の key 成分。equality/hash のみに使う（Equivalent :181-）。構築後の書換え無し |  |
| `crates/arvane-text/src/cache.rs:123` | SpanBits.family | 確認済み | cache | — | SpanBits は span_bits（cache.rs:587-）だけが span style と drawn から作る layout cache の key 成分。equality/hash のみに使う（Equivalent :181-）。構築後の書換え無し |  |
| `crates/arvane-text/src/cache.rs:125` | SpanBits.italic | 確認済み | cache | — | SpanBits は span_bits（cache.rs:587-）だけが span style と drawn から作る layout cache の key 成分。equality/hash のみに使う（Equivalent :181-）。構築後の書換え無し |  |
| `crates/arvane-text/src/cache.rs:127` | SpanBits.letter_spacing | 確認済み | cache | — | SpanBits は span_bits（cache.rs:587-）だけが span style と drawn から作る layout cache の key 成分。equality/hash のみに使う（Equivalent :181-）。構築後の書換え無し |  |
| `crates/arvane-text/src/cache.rs:129` | SpanBits.word_spacing | 確認済み | cache | — | SpanBits は span_bits（cache.rs:587-）だけが span style と drawn から作る layout cache の key 成分。equality/hash のみに使う（Equivalent :181-）。構築後の書換え無し |  |
| `crates/arvane-text/src/cache.rs:131` | SpanBits.kerning | 確認済み | cache | — | SpanBits は span_bits（cache.rs:587-）だけが span style と drawn から作る layout cache の key 成分。equality/hash のみに使う（Equivalent :181-）。構築後の書換え無し |  |
| `crates/arvane-text/src/cache.rs:133` | SpanBits.features | 確認済み | cache | — | SpanBits は span_bits（cache.rs:587-）だけが span style と drawn から作る layout cache の key 成分。equality/hash のみに使う（Equivalent :181-）。構築後の書換え無し |  |
| `crates/arvane-text/src/cache.rs:135` | SpanBits.variations | 確認済み | cache | — | SpanBits は span_bits（cache.rs:587-）だけが span style と drawn から作る layout cache の key 成分。equality/hash のみに使う（Equivalent :181-）。構築後の書換え無し |  |
| `crates/arvane-text/src/cache.rs:137` | SpanBits.font_width | 確認済み | cache | — | SpanBits は span_bits（cache.rs:587-）だけが span style と drawn から作る layout cache の key 成分。equality/hash のみに使う（Equivalent :181-）。構築後の書換え無し |  |
| `crates/arvane-text/src/cache.rs:139` | SpanBits.decoration | 確認済み | cache | — | SpanBits は span_bits（cache.rs:587-）だけが span style と drawn から作る layout cache の key 成分。equality/hash のみに使う（Equivalent :181-）。構築後の書換え無し |  |
| `crates/arvane-text/src/cache.rs:141` | SpanBits.highlight | 確認済み | cache | — | SpanBits は span_bits（cache.rs:587-）だけが span style と drawn から作る layout cache の key 成分。equality/hash のみに使う（Equivalent :181-）。構築後の書換え無し |  |
| `crates/arvane-text/src/cache.rs:151` | SpanBitsRef.font_size | 確認済み | cache | — | SpanBitsRef は span_bits_ref（cache.rs:556-）だけが span style から作る lookup 用の key 成分。hash と Equivalent（:180-）のみに使う |  |
| `crates/arvane-text/src/cache.rs:153` | SpanBitsRef.weight | 確認済み | cache | — | SpanBitsRef は span_bits_ref（cache.rs:556-）だけが span style から作る lookup 用の key 成分。hash と Equivalent（:180-）のみに使う |  |
| `crates/arvane-text/src/cache.rs:155` | SpanBitsRef.color | 確認済み | cache | — | SpanBitsRef は span_bits_ref（cache.rs:556-）だけが span style から作る lookup 用の key 成分。hash と Equivalent（:180-）のみに使う |  |
| `crates/arvane-text/src/cache.rs:157` | SpanBitsRef.stroke | 確認済み | cache | — | SpanBitsRef は span_bits_ref（cache.rs:556-）だけが span style から作る lookup 用の key 成分。hash と Equivalent（:180-）のみに使う |  |
| `crates/arvane-text/src/cache.rs:159` | SpanBitsRef.family | 確認済み | cache | — | SpanBitsRef は span_bits_ref（cache.rs:556-）だけが span style から作る lookup 用の key 成分。hash と Equivalent（:180-）のみに使う |  |
| `crates/arvane-text/src/cache.rs:161` | SpanBitsRef.italic | 確認済み | cache | — | SpanBitsRef は span_bits_ref（cache.rs:556-）だけが span style から作る lookup 用の key 成分。hash と Equivalent（:180-）のみに使う |  |
| `crates/arvane-text/src/cache.rs:163` | SpanBitsRef.letter_spacing | 確認済み | cache | — | SpanBitsRef は span_bits_ref（cache.rs:556-）だけが span style から作る lookup 用の key 成分。hash と Equivalent（:180-）のみに使う |  |
| `crates/arvane-text/src/cache.rs:165` | SpanBitsRef.word_spacing | 確認済み | cache | — | SpanBitsRef は span_bits_ref（cache.rs:556-）だけが span style から作る lookup 用の key 成分。hash と Equivalent（:180-）のみに使う |  |
| `crates/arvane-text/src/cache.rs:167` | SpanBitsRef.kerning | 確認済み | cache | — | SpanBitsRef は span_bits_ref（cache.rs:556-）だけが span style から作る lookup 用の key 成分。hash と Equivalent（:180-）のみに使う |  |
| `crates/arvane-text/src/cache.rs:169` | SpanBitsRef.features | 確認済み | cache | — | SpanBitsRef は span_bits_ref（cache.rs:556-）だけが span style から作る lookup 用の key 成分。hash と Equivalent（:180-）のみに使う |  |
| `crates/arvane-text/src/cache.rs:171` | SpanBitsRef.variations | 確認済み | cache | — | SpanBitsRef は span_bits_ref（cache.rs:556-）だけが span style から作る lookup 用の key 成分。hash と Equivalent（:180-）のみに使う |  |
| `crates/arvane-text/src/cache.rs:173` | SpanBitsRef.font_width | 確認済み | cache | — | SpanBitsRef は span_bits_ref（cache.rs:556-）だけが span style から作る lookup 用の key 成分。hash と Equivalent（:180-）のみに使う |  |
| `crates/arvane-text/src/cache.rs:175` | SpanBitsRef.decoration | 確認済み | cache | — | SpanBitsRef は span_bits_ref（cache.rs:556-）だけが span style から作る lookup 用の key 成分。hash と Equivalent（:180-）のみに使う |  |
| `crates/arvane-text/src/cache.rs:177` | SpanBitsRef.highlight | 確認済み | cache | — | SpanBitsRef は span_bits_ref（cache.rs:556-）だけが span style から作る lookup 用の key 成分。hash と Equivalent（:180-）のみに使う |  |
| `crates/arvane-text/src/cache.rs:229` | RichKey.decoration | 確認済み | cache | — | RichKey は RichKeyRef::to_owned（cache.rs:451-465）だけが作る key 成分。equality のみ |  |
| `crates/arvane-text/src/cache.rs:231` | RichKey.overflow | 確認済み | cache | — | RichKey は RichKeyRef::to_owned（cache.rs:451-465）だけが作る key 成分。equality のみ |  |
| `crates/arvane-text/src/cache.rs:233` | RichKey.max_lines | 確認済み | cache | — | RichKey は RichKeyRef::to_owned（cache.rs:451-465）だけが作る key 成分。equality のみ |  |
| `crates/arvane-text/src/cache.rs:235` | RichKey.capsize | 確認済み | cache | — | RichKey は RichKeyRef::to_owned（cache.rs:451-465）だけが作る key 成分。equality のみ |  |
| `crates/arvane-text/src/cache.rs:248` | TextKey.color | 確認済み | cache | — | TextKey は TextKeyRef::to_owned（cache.rs:341-360）だけが作る key 成分。equality/hash のみ |  |
| `crates/arvane-text/src/cache.rs:252` | TextKey.wrap | 確認済み | cache | — | TextKey は TextKeyRef::to_owned（cache.rs:341-360）だけが作る key 成分。equality/hash のみ |  |
| `crates/arvane-text/src/cache.rs:255` | TextKey.family | 確認済み | cache | — | TextKey は TextKeyRef::to_owned（cache.rs:341-360）だけが作る key 成分。equality/hash のみ |  |
| `crates/arvane-text/src/cache.rs:257` | TextKey.italic | 確認済み | cache | — | TextKey は TextKeyRef::to_owned（cache.rs:341-360）だけが作る key 成分。equality/hash のみ |  |
| `crates/arvane-text/src/cache.rs:262` | TextKey.optical_sizing | 確認済み | cache | — | TextKey は TextKeyRef::to_owned（cache.rs:341-360）だけが作る key 成分。equality/hash のみ |  |
| `crates/arvane-text/src/cache.rs:264` | TextKey.max_width | 確認済み | cache | — | TextKey は TextKeyRef::to_owned（cache.rs:341-360）だけが作る key 成分。equality/hash のみ |  |
| `crates/arvane-text/src/cache.rs:266` | TextKey.carets | 確認済み | cache | — | TextKey は TextKeyRef::to_owned（cache.rs:341-360）だけが作る key 成分。equality/hash のみ |  |
| `crates/arvane-text/src/cache.rs:269` | TextKey.rich | 確認済み | cache | — | TextKey は TextKeyRef::to_owned（cache.rs:341-360）だけが作る key 成分。equality/hash のみ |  |
| `crates/arvane-text/src/cache.rs:280` | TextKeyRef.color | 確認済み | cache | — | TextKeyRef は new（cache.rs:305-331）と with_carets（:335）だけが style から作る lookup 用の key 成分 |  |
| `crates/arvane-text/src/cache.rs:283` | TextKeyRef.wrap | 確認済み | cache | — | TextKeyRef は new（cache.rs:305-331）と with_carets（:335）だけが style から作る lookup 用の key 成分 |  |
| `crates/arvane-text/src/cache.rs:286` | TextKeyRef.family | 確認済み | cache | — | TextKeyRef は new（cache.rs:305-331）と with_carets（:335）だけが style から作る lookup 用の key 成分 |  |
| `crates/arvane-text/src/cache.rs:288` | TextKeyRef.italic | 確認済み | cache | — | TextKeyRef は new（cache.rs:305-331）と with_carets（:335）だけが style から作る lookup 用の key 成分 |  |
| `crates/arvane-text/src/cache.rs:292` | TextKeyRef.optical_sizing | 確認済み | cache | — | TextKeyRef は new（cache.rs:305-331）と with_carets（:335）だけが style から作る lookup 用の key 成分 |  |
| `crates/arvane-text/src/cache.rs:294` | TextKeyRef.max_width | 確認済み | cache | — | TextKeyRef は new（cache.rs:305-331）と with_carets（:335）だけが style から作る lookup 用の key 成分 |  |
| `crates/arvane-text/src/cache.rs:296` | TextKeyRef.carets | 確認済み | cache | — | TextKeyRef は new（cache.rs:305-331）と with_carets（:335）だけが style から作る lookup 用の key 成分 |  |
| `crates/arvane-text/src/cache.rs:298` | TextKeyRef.rich | 確認済み | cache | — | TextKeyRef は new（cache.rs:305-331）と with_carets（:335）だけが style から作る lookup 用の key 成分 |  |
| `crates/arvane-text/src/cache.rs:413` | RichKeyRef.decoration | 確認済み | cache | — | RichKeyRef は rich_key（cache.rs:426-447）だけが style から作る key 成分 |  |
| `crates/arvane-text/src/cache.rs:415` | RichKeyRef.overflow | 確認済み | cache | — | RichKeyRef は rich_key（cache.rs:426-447）だけが style から作る key 成分 |  |
| `crates/arvane-text/src/cache.rs:417` | RichKeyRef.max_lines | 確認済み | cache | — | RichKeyRef は rich_key（cache.rs:426-447）だけが style から作る key 成分 |  |
| `crates/arvane-text/src/cache.rs:419` | RichKeyRef.capsize | 確認済み | cache | — | RichKeyRef は rich_key（cache.rs:426-447）だけが style から作る key 成分 |  |
| `crates/arvane-text/src/layouts.rs:46` | SpanKey.font_size | 確認済み | cache | — | SpanKey は SpanKey::new（layouts.rs:69-95）だけが span style から作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-text/src/layouts.rs:48` | SpanKey.weight | 確認済み | cache | — | SpanKey は SpanKey::new（layouts.rs:69-95）だけが span style から作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-text/src/layouts.rs:50` | SpanKey.family | 確認済み | cache | — | SpanKey は SpanKey::new（layouts.rs:69-95）だけが span style から作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-text/src/layouts.rs:52` | SpanKey.italic | 確認済み | cache | — | SpanKey は SpanKey::new（layouts.rs:69-95）だけが span style から作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-text/src/layouts.rs:54` | SpanKey.letter_spacing | 確認済み | cache | — | SpanKey は SpanKey::new（layouts.rs:69-95）だけが span style から作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-text/src/layouts.rs:56` | SpanKey.word_spacing | 確認済み | cache | — | SpanKey は SpanKey::new（layouts.rs:69-95）だけが span style から作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-text/src/layouts.rs:58` | SpanKey.kerning | 確認済み | cache | — | SpanKey は SpanKey::new（layouts.rs:69-95）だけが span style から作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-text/src/layouts.rs:60` | SpanKey.features | 確認済み | cache | — | SpanKey は SpanKey::new（layouts.rs:69-95）だけが span style から作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-text/src/layouts.rs:62` | SpanKey.variations | 確認済み | cache | — | SpanKey は SpanKey::new（layouts.rs:69-95）だけが span style から作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-text/src/layouts.rs:64` | SpanKey.font_width | 確認済み | cache | — | SpanKey は SpanKey::new（layouts.rs:69-95）だけが span style から作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-text/src/layouts.rs:122` | Key.wrap | 確認済み | cache | — | Key は Key::new（layouts.rs:139-）だけが作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-text/src/layouts.rs:124` | Key.family | 確認済み | cache | — | Key は Key::new（layouts.rs:139-）だけが作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-text/src/layouts.rs:126` | Key.italic | 確認済み | cache | — | Key は Key::new（layouts.rs:139-）だけが作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-text/src/layouts.rs:130` | Key.optical_sizing | 確認済み | cache | — | Key は Key::new（layouts.rs:139-）だけが作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-text/src/layouts.rs:132` | Key.rich | 確認済み | cache | — | Key は Key::new（layouts.rs:139-）だけが作る shaping cache の key 成分。equality のみ |  |
| `crates/arvane-virtual/src/window.rs:134` | (no audit row: pub(crate) fail_next: RefCell<Option<ShellError>>,) | 確認済み | option | — | writer: backend.rs:430（script の FailEffect、test が与える）, 消費 window.rs:237 の take のみ。test の scripted な一度きりの値 |  |
| `crates/arvane/src/presentation.rs:38` | RendererChange::Gpu.configure | 閉じ方の変更 | statistic | P8-GPU | writers: gpu.rs:448/483（None）, :476（Some(背景での configure の所要時間)）。計測値で読み手は Display（presentation.rs:92-93）のみ。test_host.rs:201, support.rs:417 は .. で無視。authored ではない | `` // xtask-allow(state-fields): statistic: how long the first swapchain took when it was configured in the background, shown as `configure_ms`; absent when it was configured on the UI thread; nothing branches on it `` |
| `crates/arvane/src/presentation.rs:42` | RendererChange::Gpu.interop | 閉じ方の変更 | independent | P8-GPU | writer: gpu.rs:694 context.interop_adapter() のみ。feature off だけでなく adapter 非対応でも None で理由も不正確。device で決まり configure とは別の源で全組合せ | `` // xtask-allow(state-fields): independent: the adapter when the device can import shared textures (feature gpu-interop and an adapter that supports it), absent otherwise; it depends on the device alone, while configure depends on where the swapchain was configured, so all pairs occur `` |
| `crates/arvane/src/window.rs:196` | Windows.observer | 文言 | option | P8-GPU | writers: window.rs:424/457/493 で Driver.observer を clone するだけ。Driver は Shell::run（:338）と ShellSetup（:379）の両方から作られるので『from ShellSetup』は不正確 | `` // xtask-allow(state-fields): option: the renderer observer the application gave (`Shell::on_renderer_change` or `ShellSetup::observer`), cloned into every presenter's Notes; absent when it gave none `` |
| `crates/arvane/src/window.rs:285` | Shell.prefer_gpu | 文言 | option | P8-GPU | writers: window.rs:297（既定 true）, :309 builder のみ。理由が field 名の繰り返し | `` // xtask-allow(state-fields): option: whether the application asked to draw on the GPU (`Shell::prefer_gpu`, default true); false draws on the CPU unless `ARVANE_RENDERER=gpu` overrides it `` |
| `crates/arvane/src/window.rs:288` | Shell.observer | 文言 | option | P8-GPU | writers: window.rs:299（None）, :332 on_renderer_change のみ。理由が field 名のみ | `` // xtask-allow(state-fields): option: the renderer observer `Shell::on_renderer_change` gave; absent when the application observes no renderer change `` |
| `crates/arvane/src/window.rs:345` | ShellSetup.prefer_gpu | 文言 | option | P8-GPU | pub field、Default :354 で true、host が構築時に与え :377 で読むのみ。理由が field 名の繰り返し | `` // xtask-allow(state-fields): option: whether the host asks to draw on the GPU (default true); false draws on the CPU unless `ARVANE_RENDERER=gpu` overrides it `` |
| `crates/arvane/src/window.rs:348` | ShellSetup.observer | 文言 | option | P8-GPU | pub field、Default :356 で None、host が与え :379 で読むのみ。理由が field 名のみ | `` // xtask-allow(state-fields): option: the renderer observer the host gives, cloned into every presenter's Notes; absent when it observes no renderer change `` |
| `crates/arvane/src/window.rs:412` | Driver.observer | 文言 | option | P8-GPU | writer: window.rs:399 driver() のみ（Shell::run :338 と ShellSetup :379 から）。『from ShellSetup』は不正確 | `` // xtask-allow(state-fields): option: the renderer observer the application gave (`Shell::on_renderer_change` or `ShellSetup::observer`), cloned into every presenter's Notes; absent when it gave none `` |
| `crates/kivact-gpu-share/src/common.rs:13` | Identity.luid | 誤（付録の設計） | type（W6） | P8-GPU | W6。writers: import/dx12.rs:53-58（luid Some, uuid は共に None）, vulkan_raw.rs:112-117（uuid は共に Some, luid は任意）。device_uuid⇔driver_uuid が同時（SM-3）で backend が組を決める含意。physical_device.rs:76-79 は expect で不変条件扱い |  |
| `crates/kivact-gpu-share/src/common.rs:16` | Identity.device_uuid | 誤（付録の設計） | type（W6） | P8-GPU | W6。writers: import/dx12.rs:53-58（luid Some, uuid は共に None）, vulkan_raw.rs:112-117（uuid は共に Some, luid は任意）。device_uuid⇔driver_uuid が同時（SM-3）で backend が組を決める含意。physical_device.rs:76-79 は expect で不変条件扱い |  |
| `crates/kivact-gpu-share/src/common.rs:19` | Identity.driver_uuid | 誤（付録の設計） | type（W6） | P8-GPU | W6。writers: import/dx12.rs:53-58（luid Some, uuid は共に None）, vulkan_raw.rs:112-117（uuid は共に Some, luid は任意）。device_uuid⇔driver_uuid が同時（SM-3）で backend が組を決める含意。physical_device.rs:76-79 は expect で不変条件扱い |  |
| `crates/kivact-gpu-share/src/pool.rs:131` | Slots.next | 誤（付録の設計） | derive（W7） | P8-GPU | W7。writers: pool.rs:148（0）, :180（finish で +1 mod len）のみ。last も finish :178 でのみ +1（初期 None, 値は 2 から）。next == (last-1) % len で導出でき、性能の理由も無い複写 |  |
| `crates/kivact-gpu-share/src/pool.rs:134` | Slots.last | 確認済み | identity | — | writers: pool.rs:149（None）, :178（finish）のみ。frame の値の採番。W7 適用後は数える field が 1 個になり marker 削除 |  |
| `crates/kivact-shader/src/entry.rs:16` | Interface.params_declared | 誤（付録の設計） | type（W8） | P8-ARV | W8。writer: entry.rs:31-57 read_interface のみ。check_arguments :95 は params 引数があって未宣言なら Err → takes_params ⇒ params_declared の含意。(false,true) 到達不能で『every combination is valid』は誤り |  |
| `crates/kivact-shader/src/entry.rs:19` | Interface.takes_params | 誤（付録の設計） | type（W8） | P8-ARV | W8。writer: entry.rs:31-57 read_interface のみ。check_arguments :95 は params 引数があって未宣言なら Err → takes_params ⇒ params_declared の含意。(false,true) 到達不能で『every combination is valid』は誤り |  |
| `crates/kivact-types/src/gpu.rs:73` | GpuPreference.backend | 確認済み | option | — | pub、app が与える wish（gpu.rs:73-74）。reader state.rs:61 で env が無いときに使う。内部での書換え無し |  |
| `crates/kivact-types/src/gpu.rs:76` | GpuPreference.adapter | 確認済み | option | — | pub、app が与える wish（gpu.rs:76-79）。内部での書換え無し。absent の意味も正しい |  |
| `crates/kivact-types/src/gpu.rs:93` | GpuAdapter.luid | 誤（付録の設計） | type（W6） | P8-GPU | W6。writers: context.rs:209-216（Identity を写す）, gpu-interop-producer d3d12.rs:170, vulkan.rs:201。Identity と同じ SM-3 と backend の含意。physical_device.rs:93, d3d12/adapter.rs:18 は組違いを Mismatch 扱い |  |
| `crates/kivact-types/src/gpu.rs:96` | GpuAdapter.device_uuid | 誤（付録の設計） | type（W6） | P8-GPU | W6。writers: context.rs:209-216（Identity を写す）, gpu-interop-producer d3d12.rs:170, vulkan.rs:201。Identity と同じ SM-3 と backend の含意。physical_device.rs:93, d3d12/adapter.rs:18 は組違いを Mismatch 扱い |  |
| `crates/kivact-types/src/gpu.rs:99` | GpuAdapter.driver_uuid | 誤（付録の設計） | type（W6） | P8-GPU | W6。writers: context.rs:209-216（Identity を写す）, gpu-interop-producer d3d12.rs:170, vulkan.rs:201。Identity と同じ SM-3 と backend の含意。physical_device.rs:93, d3d12/adapter.rs:18 は組違いを Mismatch 扱い |  |
| `crates/kivact-types/src/gpu.rs:102` | GpuAdapter.generation | 確認済み | identity | — | writer: context.rs:215（next_generation :222-225 の process 内連番）のみ。device の incarnation の刻印。W6 適用後は数える field が 1 個になり marker 削除 |  |

### B.7 G7

| marker（HEAD） | owner.field | 判定 | 閉じ方 | unit | 根拠 | 新しい marker 行 |
|---|---|---|---|---|---|---|
| `crates/kivact-devtools/src/panels/elements/view.rs:21` | view::owners_on | 文言 | independent | P8-L1 | writers: Owners checkbox のみ (view.rs:55)。layout は details の checkbox (details.rs:105) で両 tree 下に常に表示、4 組合せ到達。理由文に書き手が無い | `` // xtask-allow(state-fields): independent: set only by the Owners checkbox; it swaps the element tree for the owner tree while the details pane and its layout overlay checkbox stay shown, so it pairs with either overlay value `` |
| `crates/kivact-devtools/src/panels/elements/view.rs:23` | view::layout | 文言 | independent | P8-L1 | writers: layout_toggle (view.rs:25-32) のみ、details.rs:105 の checkbox から。owners_on と無関係に表示、4 組合せ到達。理由文に書き手が無い | `` // xtask-allow(state-fields): independent: set only through layout_toggle by the details pane's layout overlay checkbox, which is shown under either tree, so it pairs with either value of owners_on `` |
| `crates/kivact-devtools/src/session.rs:20` | new::target | 確認済み | identity | — | var(target) は session.rs:21 の構築時のみ、set/update 無し。読み手 status_bar.rs:13, toolbar.rs:11, session.rs:106,236-268 のみ。接続先の識別 |  |
| `crates/kivact-editing/src/behavior.rs:40` | EditContext.multiline | 閉じ方の変更 | option | P8-L1 | 唯一の構築 keys.rs:605-612、値は node.multiline() (node.rs:395、TextEditing::multiline の authored prop)。内部 event で変わらない。'every combination is valid' は不可 | `` // xtask-allow(state-fields): option: the editor's authored multiline prop the engine passes per request; false means committed text keeps no line breaks `` |
| `crates/kivact-editing/src/behavior.rs:43` | EditContext.read_only | 閉じ方の変更 | option | P8-L1 | keys.rs:346/561 で node.read_only() (node.rs:316、read_only prop・reactive 可の authored 値)。内部 event で書かれない | `` // xtask-allow(state-fields): option: the editor's authored read_only prop the engine passes per request; false lets user input change the value `` |
| `crates/kivact-editing/src/behavior.rs:46` | EditContext.password | 閉じ方の変更 | option | P8-L1 | keys.rs:346/561 で node.password() = purpose()==Password (node.rs:301、purpose prop)。authored 値 | `` // xtask-allow(state-fields): option: whether the editor's authored input purpose is Password, passed per request; false lets copy and cut expose the value `` |
| `crates/kivact-editing/src/behavior.rs:49` | EditContext.layout | 文言 | independent | P8-L1 | engine の唯一の呼出 keys.rs:612 は常に Some(layout)、公開 EditContext::new の caller は None 可。flags とは別入力。理由文が定型 | `` // xtask-allow(state-fields): independent: the editor's laid-out lines for this request, which the engine always supplies and a caller without layout leaves out; the flags are authored props and do not decide whether lines exist `` |
| `crates/kivact-editing/src/line_colors.rs:68` | LineStyle.color | 閉じ方の変更 | option | P8-L1 | 書き手は line_styles hook の変換 text_editing.rs:351-357 (highlighter が返す range の color を theme で解決) と shaping.rs:381 の分割コピーのみ。authored 値 | `` // xtask-allow(state-fields): option: the highlighter's text color for the range, resolved against the theme; absent keeps the editor's own text color `` |
| `crates/kivact-editing/src/line_colors.rs:70` | LineStyle.background | 閉じ方の変更 | option | P8-L1 | text_editing.rs:355 で highlighter の background を解決、shaping.rs:453 で highlight 有無として読む。authored 値 | `` // xtask-allow(state-fields): option: the highlighter's background for the range, resolved against the theme; absent paints no highlight behind it `` |
| `crates/kivact-editing/src/line_colors.rs:72` | LineStyle.decoration | 閉じ方の変更 | option | P8-L1 | text_editing.rs:356-358 で highlighter の decoration を compile、shaping.rs:452 で読む。authored 値 | `` // xtask-allow(state-fields): option: the highlighter's text decoration for the range, resolved against the theme; absent draws none `` |
| `crates/kivact-i18n/src/format.rs:107` | PluralForms.zero | 文言 | option | P8-L1 | 書き手は Messages::with_plural に渡す caller (messages.rs:42) のみ。読み手 messages.rs:53-59 は None で other に fallback。'locale has no form' は不正確（author が省いた場合も None） | `` // xtask-allow(state-fields): option: the message author's template for the zero category; absent falls back to the other template `` |
| `crates/kivact-i18n/src/format.rs:110` | PluralForms.one | 文言 | option | P8-L1 | 書き手は Messages::with_plural に渡す caller (messages.rs:42) のみ。読み手 messages.rs:53-59 は None で other に fallback。'locale has no form' は不正確（author が省いた場合も None） | `` // xtask-allow(state-fields): option: the message author's template for the one category; absent falls back to the other template `` |
| `crates/kivact-i18n/src/format.rs:113` | PluralForms.two | 文言 | option | P8-L1 | 書き手は Messages::with_plural に渡す caller (messages.rs:42) のみ。読み手 messages.rs:53-59 は None で other に fallback。'locale has no form' は不正確（author が省いた場合も None） | `` // xtask-allow(state-fields): option: the message author's template for the two category; absent falls back to the other template `` |
| `crates/kivact-i18n/src/format.rs:116` | PluralForms.few | 文言 | option | P8-L1 | 書き手は Messages::with_plural に渡す caller (messages.rs:42) のみ。読み手 messages.rs:53-59 は None で other に fallback。'locale has no form' は不正確（author が省いた場合も None） | `` // xtask-allow(state-fields): option: the message author's template for the few category; absent falls back to the other template `` |
| `crates/kivact-i18n/src/format.rs:119` | PluralForms.many | 文言 | option | P8-L1 | 書き手は Messages::with_plural に渡す caller (messages.rs:42) のみ。読み手 messages.rs:53-59 は None で other に fallback。'locale has no form' は不正確（author が省いた場合も None） | `` // xtask-allow(state-fields): option: the message author's template for the many category; absent falls back to the other template `` |
| `crates/kivact-i18n/src/numbers.rs:151` | SymbolCapture.symbols | 閉じ方の変更 | independent | P8-L1 | SM-5 型の out-parameter slot：ICU が try_new_unstable 中に load (numbers.rs:157-162) で埋め、NumberSymbols::new (:61-71) が一度だけ取り出す。正本/無効化が無く cache ではない。constructor 渡し不可（書き手が ICU）→要報告 | `` // xtask-allow(state-fields): independent: the decimal symbols ICU loads through this provider while it builds the formatter; ICU loads the digits through the other slot in its own order, and NumberSymbols::new reads both once after the build `` |
| `crates/kivact-i18n/src/numbers.rs:153` | SymbolCapture.digits | 閉じ方の変更 | independent | P8-L1 | 同上 numbers.rs:165-170 が唯一の書き手、:68-71 で一度読む。cache ではない（正本の再計算・無効化なし）。SM-5 だが ICU が書き手で constructor 渡し不可→要報告 | `` // xtask-allow(state-fields): independent: the decimal digits ICU loads through this provider while it builds the formatter; ICU loads the symbols through the other slot in its own order, and NumberSymbols::new reads both once after the build `` |
| `crates/kivact-interaction/src/combobox/handle.rs:113` | ComboboxApi.input_state | 確認済み | independent | — | P5 確定の同形（combobox content_state 等、covered-by-P5）。書き手は part 生成関数だけ、None に戻す書き手なし：combobox/handle.rs:572 (input()) のみ |  |
| `crates/kivact-interaction/src/dialog.rs:64` | DismissOn.escape | 文言 | option | P8-IN | 書き手は DialogApi::dismiss (dialog.rs:212) の caller のみ、既定 true (:72)。読み手 dialog.rs:288 (Escape と WindowClose)。理由文に誰が与えるか・false の意味が無い | `` // xtask-allow(state-fields): option: the caller's rule from DialogApi::dismiss (default true); false keeps the dialog open on an unhandled Escape or a window-close request `` |
| `crates/kivact-interaction/src/dialog.rs:67` | DismissOn.outside | 文言 | option | P8-IN | 書き手は DialogApi::dismiss の caller、既定 true、AlertModal は false (dialog.rs:253-260)。読み手 :298。理由文に誰が与えるか・false の意味が無い | `` // xtask-allow(state-fields): option: the caller's rule from DialogApi::dismiss (default true, false for an alert modal); false keeps the dialog open on a pointer press outside it and its anchor `` |
| `crates/kivact-interaction/src/drag/handle.rs:111` | Step::Stop.owns_edit | 文言 | independent | P8-IN | 書き手 handle.rs:866 (active.edit==Owned の写し) のみ、読み handle.rs:666-672→stop()。update の有無は位置だけで決まり、edit の有無と全組合せ到達 | `` // xtask-allow(state-fields): independent: copied from the ended drag's EditOwnership (Owned when the drag began the edit); a drag with or without an edit can end with or without an update `` |
| `crates/kivact-interaction/src/drag/handle.rs:113` | Step::Stop.update | 文言 | independent | P8-IN | handle.rs:858 で不正座標は active.last に置換されるため 'invalid coordinates で None' は誤り。None は update() (:875) で start からの delta が非有限の時だけ | `` // xtask-allow(state-fields): independent: what on_end receives, absent when the delta from the drag's start is not finite; that depends on the positions alone, so it comes with either edit ownership `` |
| `crates/kivact-interaction/src/drag/handle.rs:146` | DragApi.observed | 確認済み | independent | — | P5 確定の同形（combobox content_state 等、covered-by-P5）。書き手は part 生成関数だけ、None に戻す書き手なし：drag/handle.rs:644 (source_with) のみ |  |
| `crates/kivact-interaction/src/drag/handle.rs:154` | DragApi.disabled | 確認済み | option | — | 書き手 DragApi::disabled setter (handle.rs:291-293) のみ、読み is_disabled (:432)。内部 event で書かれない |  |
| `crates/kivact-interaction/src/edit/handle.rs:85` | EditSession.observer | 確認済み | option | — | 書き手 edit/handle.rs:128 (observer setter) のみ、初期 None (:112)。内部 event で書かれない |  |
| `crates/kivact-interaction/src/list/state.rs:38` | State.container_state | 確認済み | independent | — | P5 確定の同形（combobox content_state 等、covered-by-P5）。書き手は part 生成関数だけ、None に戻す書き手なし：list/parts.rs:19 のみ |  |
| `crates/kivact-interaction/src/menu/level.rs:36` | Level.retain_disabled_focus | 確認済み | option | — | menu/level.rs:54-58 で theme の InteractionPolicy から構築時に一度だけ計算、以後不変。読み level.rs:74 |  |
| `crates/kivact-interaction/src/menu/state.rs:160` | ItemEntry.disabled | 確認済み | option | — | 書き手 MenuEntry::disabled 修飾子 (state.rs:317)、of_item 既定 false (:194)、map_key で写すのみ (:548) |  |
| `crates/kivact-interaction/src/menu/state.rs:163` | ItemEntry.disabled_reason | 確認済み | option | — | 書き手 with_disabled_reason (state.rs:325) のみ。doc で disabled と独立（理由だけでは disable しない） |  |
| `crates/kivact-interaction/src/menu/state.rs:165` | ItemEntry.icon | 確認済み | option | — | 書き手 with_icon (state.rs:293) のみ、map_key 写し |  |
| `crates/kivact-interaction/src/menu/state.rs:168` | ItemEntry.description | 確認済み | option | — | 書き手 with_description (state.rs:300) のみ、map_key 写し |  |
| `crates/kivact-interaction/src/menu/state.rs:171` | ItemEntry.destructive | 確認済み | option | — | 書き手 destructive 修飾子 (state.rs:334) のみ、描画だけが読む |  |
| `crates/kivact-interaction/src/menubar.rs:96` | use_menubar::focus | 文言 | independent | P8-IN | focus は use_list の Selection::Single binding (menubar.rs:99-101) で list だけが書く。menus は var で以後不変。理由文に書き手が無い | `` // xtask-allow(state-fields): independent: the trigger list's selection, written by use_list as the arrows and activation move along the bar; menus is fixed at setup, so any index or none pairs with it `` |
| `crates/kivact-interaction/src/menubar.rs:109` | use_menubar::menus | 確認済み | option | — | var(menus) (menubar.rs:110) 以後 set/update 無し、読みのみ (:112-128, :170) |  |
| `crates/kivact-interaction/src/scalar/track_data.rs:15` | (no audit row: pub(crate) bounds: Option<(Value<f64>, Value<f64>)>,) | 確認済み | option | — | 書き手 ScalarApi::target (scalar/handle.rs:239-246) で Edit.bounds (P5 確定 option) を写すのみ |  |
| `crates/kivact-interaction/src/select/handle.rs:135` | SelectApi.preview | 確認済み | option | — | 書き手 select/handle.rs:335 (preview setter) のみ、初期 None (:168) |  |
| `crates/kivact-interaction/src/space_pan/handle.rs:18` | SpacePanApi.observed | 確認済み | independent | — | P5 確定の同形（combobox content_state 等、covered-by-P5）。書き手は part 生成関数だけ、None に戻す書き手なし：space_pan/handle.rs:65 のみ |  |
| `crates/kivact-interaction/src/splitter/handle.rs:116` | SplitterApi.observed | 確認済み | independent | — | P5 確定の同形（combobox content_state 等、covered-by-P5）。書き手は part 生成関数だけ、None に戻す書き手なし：splitter/handle.rs:435 のみ |  |
| `crates/kivact-interaction/src/table/data.rs:16` | Header.sortable | 文言 | option | P8-IN | 唯一の書き手 ColumnDef::header (model.rs:203) が compare.is_some() を写す。理由文は意味だけで出所が無い | `` // xtask-allow(state-fields): option: whether the caller gave the column a ColumnDef::compare, copied by ColumnDef::header; false leaves the header unsortable `` |
| `crates/kivact-interaction/src/table/data.rs:18` | Header.width | 確認済み | option | — | 唯一の書き手 ColumnDef::header (model.rs:204) が caller の with_width を写す。absent の意味も正しい |  |
| `crates/kivact-interaction/src/table/data.rs:21` | Header.min_width | 文言 | option | P8-IN | 書き手 ColumnDef::header (model.rs:205)、読み table/handle.rs:511 で None は resize range 自身の min。理由文に absent の意味が無い | `` // xtask-allow(state-fields): option: the caller's ColumnDef::min_width, copied by ColumnDef::header; absent keeps the resize range's own minimum `` |
| `crates/kivact-interaction/src/table/model.rs:65` | ColumnDef.width | 文言 | option | P8-IN | 書き手 with_width (model.rs:137-140)、new で None。理由文 'width' は field 名の繰り返し | `` // xtask-allow(state-fields): option: the caller's starting width from with_width; absent leaves the column width to layout `` |
| `crates/kivact-interaction/src/table/model.rs:67` | ColumnDef.min_width | 文言 | option | P8-IN | 書き手 min_width (model.rs:143-146、有限・非負のみ)。読み handle.rs:511。理由文 'min width' は field 名の繰り返し | `` // xtask-allow(state-fields): option: the caller's resize lower bound from min_width, finite and non-negative only; absent keeps the resize range's own minimum `` |
| `crates/kivact-interaction/src/table/model.rs:69` | ColumnDef.compare | 文言 | option | P8-IN | 書き手 compare (model.rs:110-113) と without_ordering (:163) のみ。理由文 'compare' は field 名の繰り返し | `` // xtask-allow(state-fields): option: the caller's row ordering from compare; absent (or without_ordering) makes the column unsortable `` |
| `crates/kivact-interaction/src/table/model.rs:71` | ColumnDef.matches | 文言 | option | P8-IN | 書き手 filter (model.rs:118-121) のみ。理由文 'matches' は field 名の繰り返し | `` // xtask-allow(state-fields): option: the caller's text matcher from filter; absent leaves the column out of text filtering `` |
| `crates/kivact-interaction/src/table/model.rs:73` | ColumnDef.matches_value | 文言 | option | P8-IN | 書き手 filter_value (model.rs:124-127) のみ。理由文 'matches value' は field 名の繰り返し | `` // xtask-allow(state-fields): option: the caller's typed-filter matcher from filter_value; absent leaves the column out of typed filtering `` |
| `crates/kivact-interaction/src/table/model.rs:300` | DataTableModelApi.sort | 確認済み | option | — | 書き手 multi_sort (model.rs:443-446、sort から) のみ。setup only setter |  |
| `crates/kivact-interaction/src/table/model.rs:302` | DataTableModelApi.query | 確認済み | option | — | 書き手 DataTableModelApi::filter (model.rs:452-455) のみ |  |
| `crates/kivact-interaction/src/table/model.rs:304` | DataTableModelApi.column_queries | 確認済み | option | — | 書き手 column_filters (model.rs:463-466) のみ |  |
| `crates/kivact-interaction/src/table/model.rs:306` | DataTableModelApi.arrangement | 確認済み | option | — | 書き手 arrange_rows (model.rs:470-473) のみ |  |
| `crates/kivact-interaction/src/table/model.rs:308` | DataTableModelApi.typed_queries | 確認済み | option | — | 書き手 typed_column_filters (model.rs:478-480) のみ |  |
| `crates/kivact-interaction/src/table/model.rs:310` | DataTableModelApi.paging | 確認済み | option | — | 書き手 page (model.rs:489-498) のみ（setup only）、読み pages/page memo (model.rs:378-397) |  |
| `crates/kivact-interaction/src/table/model.rs:341` | use_data_table_model::sort | 確認済み | option | — | struct field と同じ signal の local、書き手は setter のみ。absent で caller 順 (model.rs:562, 583-) |  |
| `crates/kivact-interaction/src/table/model.rs:343` | use_data_table_model::query | 確認済み | option | — | 同上、空なら全行 (model.rs:556-569) |  |
| `crates/kivact-interaction/src/table/model.rs:345` | use_data_table_model::column_queries | 確認済み | option | — | 同上 (model.rs:561-569) |  |
| `crates/kivact-interaction/src/table/model.rs:347` | use_data_table_model::arrangement | 確認済み | option | — | 同上、None で並べ替え無し (model.rs:603-604) |  |
| `crates/kivact-interaction/src/table/model.rs:349` | use_data_table_model::typed_queries | 文言 | option | P8-IN | None は typed 空＝typed filter 無し (model.rs:565-569)。'searches every label' は事実と違う | `` // xtask-allow(state-fields): option: the typed column filters a setter supplies; absent applies no typed filter `` |
| `crates/kivact-interaction/src/table/model.rs:351` | use_data_table_model::paging | 確認済み | option | — | 同上、None で全行 (model.rs:378-397) |  |
| `crates/kivact-interaction/src/text_edit/handle.rs:499` | Data.observed | 確認済み | independent | — | P5 確定の同形（combobox content_state 等、covered-by-P5）。書き手は part 生成関数だけ、None に戻す書き手なし：text_edit/handle.rs:407 のみ |  |
| `crates/kivact-interaction/src/toaster/column.rs:41` | Effect::Dismiss.previous | 文言 | independent | P8-IN | 書き手 dismiss_for (toaster/api.rs:227-231)：focus が toast 内の時だけ Some。callback は on_dismiss 有無と表示中かで決まり 4 組合せ到達。理由文が組合せを述べない | `` // xtask-allow(state-fields): independent: the return target when the focus was inside the dismissed toast; a toast with or without an on_dismiss handler can hold the focus, so it pairs with either callback `` |
| `crates/kivact-interaction/src/toaster/column.rs:43` | Effect::Dismiss.callback | 文言 | option | P8-IN | column.rs:67-69：on_dismiss 有り かつ index<limit（表示中）の時だけ Some。'when there is none' は待機中 toast の None を述べていない | `` // xtask-allow(state-fields): option: the caller's on_dismiss handler, absent when the toast set none or was still waiting beyond the limit `` |
| `crates/kivact-interaction/src/toaster/data.rs:115` | Toast.body | 確認済み | option | — | 書き手 Toast::body builder (data.rs:165-168) のみ、公開 field の authored 値 |  |
| `crates/kivact-interaction/src/toaster/data.rs:120` | Toast.duration | 文言 | option | P8-IN | 既定 Some(5s) (data.rs:160)、Toast::duration (:182) で caller が None 可＝dismiss まで残る。'absent until it is dismissed' は意味が逆に読める | `` // xtask-allow(state-fields): option: the caller's display time from Toast::duration (default 5 s); absent keeps the toast until it is dismissed `` |
| `crates/kivact-interaction/src/toaster/data.rs:123` | Toast.action | 確認済み | option | — | 書き手 Toast::action builder (data.rs:197-204) のみ |  |
| `crates/kivact-interaction/src/toggle.rs:34` | ButtonApi.disabled | 確認済み | option | — | 書き手 ButtonApi::disabled setter (toggle.rs:70-72) のみ |  |
| `crates/kivact-interaction/src/zoom_pan/handle.rs:21` | ZoomPanApi.observed | 確認済み | independent | — | P5 確定の同形（combobox content_state 等、covered-by-P5）。書き手は part 生成関数だけ、None に戻す書き手なし：zoom_pan/handle.rs:270 のみ |  |
| `crates/kivact-layout/src/columns.rs:60` | Counted.unfiltered | 誤（付録の設計） | type | P8-L1 | SM-2：at は filtered_flow_child (traversal.rs:165-190) だけが書き、!unfiltered の時しか呼ばれない (:159)。unfiltered=true ⇒ at=None。詳細 G7.wrong.md W1 |  |
| `crates/kivact-layout/src/columns.rs:63` | Counted.at | 誤（付録の設計） | type | P8-L1 | 同上 W1：(unfiltered=true, at=Some) は到達不能、at は Filtered の時だけ意味を持つ |  |
| `crates/kivact-runtime/src/reactive/data.rs:87` | Runtime.clock | 閉じ方の変更 | independent | P8-RT | clock_now() (clock.rs:22) は menu press (press.rs:140)・toaster timer (api.rs:450)・drag velocity (handle.rs:55) が制御に使う→statistic は誤り。書き手 advance_clock (launch/runtime.rs:226) のみ | `` // xtask-allow(state-fields): independent: the App's monotonic time, written only by advance_clock from each turn's host time; timers and gestures read it, and every reading is valid in any turn or exit state `` |
| `crates/kivact-runtime/src/reactive/data.rs:208` | SourceNode.changed_at | 閉じ方の変更 | identity | P8-RT | 書き手 notify_subscribers (graph.rs:419) のみ（write と trigger/selector）。recheck_after_own_writes (:621) の比較用 epoch。走行中 flag は無く SM-6 に当たらない。written との組合せ理由は不成立 | `` // xtask-allow(state-fields): identity: the write epoch notify_subscribers stamps on every change or trigger; recheck_after_own_writes compares it with the epoch before an effect ran, and no running flag sits beside it `` |
| `crates/kivact-runtime/src/reactive/data.rs:211` | SourceNode.written | 閉じ方の変更 | statistic | P8-RT | 書き手 graph.rs:383 のみ、読みは node_snapshot (graph.rs:845) → why.rs:88 の診断だけ。written=Some ⇒ changed_at 更新済み（含意あり）なので independent は不成立 | `` // xtask-allow(state-fields): statistic: where and in which turn the last value-changing write happened; only the node diagnostics (why chains, DevTools) read it `` |
| `crates/kivact-runtime/src/reactive/data.rs:320` | OwnerEntry.parent | 閉じ方の変更 | independent | P8-RT | detach_owner (owner.rs:253) が disposal 開始時に None にする実行時の書換え→option は誤り、'root だけ None' も誤り。(None,Live)=root、(None,Disposing)=切離した disposal root、(Some,両 phase) 到達 | `` // xtask-allow(state-fields): independent: the creating owner, set by create_owner and cleared only by detach_owner when this owner's disposal starts; the root is live without one, a detached disposal root is disposing without one, and children keep theirs `` |
| `crates/kivact-runtime/src/reactive/error.rs:32` | ErrorKind::Disposed.created_at | 確認済み | option | — | 構築時のみ (graph.rs:295, handles.rs:435, object.rs:83/103, owner.rs:476, scope.rs:42) の不変な error 値。診断の位置 |  |
| `crates/kivact-runtime/src/reactive/error.rs:36` | ErrorKind::Disposed.disposed_by | 確認済み | option | — | 同上、不変な error 値 |  |
| `crates/kivact-runtime/src/reactive/error.rs:100` | Inner.at | 確認済み | option | — | 構築時のみ (error.rs:121-148,186-198)、不変 |  |
| `crates/kivact-runtime/src/reactive/error.rs:102` | Inner.source | 確認済み | option | — | 構築時のみ (error.rs:127,149,198)、不変 |  |
| `crates/kivact-runtime/src/reactive/host_types.rs:6` | Pending.user | 文言 | independent | P8-RT | 唯一の構築 Runtime::pending (reactive/host.rs:50-56) の snapshot。各 bit は別の queue。'four independent wake facts' は組合せの理由になっていない | `` // xtask-allow(state-fields): independent: a snapshot of the user-effect queue taken by Runtime::pending; user effects queue whether or not render effects, deferred reports or a sent wake exist `` |
| `crates/kivact-runtime/src/reactive/host_types.rs:9` | Pending.render | 文言 | independent | P8-RT | 同上 has_render_effects の snapshot。理由文が定型 | `` // xtask-allow(state-fields): independent: a snapshot of the render and internal effect queue taken by Runtime::pending; a render effect queues with or without user effects, deferred reports or a sent wake `` |
| `crates/kivact-runtime/src/reactive/host_types.rs:12` | Pending.later | 文言 | independent | P8-RT | 同上 has_later_reports の snapshot。理由文が定型 | `` // xtask-allow(state-fields): independent: a snapshot of the deferred error reports taken by Runtime::pending; layout can report an error whatever effects are queued and whether a wake was sent `` |
| `crates/kivact-runtime/src/reactive/host_types.rs:15` | Pending.turn_requested | 文言 | independent | P8-RT | 同上 TurnState==WakeSent (host.rs:55)。wake は task/timer でも送られ、Running 中は queue があっても false。理由文が定型 | `` // xtask-allow(state-fields): independent: a snapshot of TurnState::WakeSent taken by Runtime::pending; a wake is also sent for tasks and timers, and a running turn has queued work without one, so it pairs with any queue contents `` |
| `crates/kivact-runtime/src/task/state.rs:110` | TaskState.blocking_threads | 確認済み | option | — | 書き手 TaskState::new (state.rs:128-134) のみ、AppConfig.blocking_threads (host.rs:14,47) から。読み state.rs:295 |  |
| `crates/kivact-runtime/src/task/worker_pool.rs:39` | (no audit row: pub(crate) outstanding: Arc<AtomicUsize>,) | 文言 | statistic | P8-RT | 'shutdown が待つ' は誤り：Pool::shutdown (worker_pool.rs:131-138) は thread を join するだけ。読みは TaskState::stats (state.rs:180) → DevTools と test driver の settle (driver.rs:214) のみ | `` // xtask-allow(state-fields): statistic: the submitted blocking jobs not yet finished; only TaskState::stats reads it, for DevTools and the test driver's settle loop `` |
| `crates/kivact-test/src/facts.rs:9` | Node.kind | 閉じ方の変更 | projection | P8-KT | 唯一の書き手 Facts::new (facts.rs:42-47) が engine reader の node type を写す snapshot。'any combination is valid' は不可、全組合せの到達も未確認 | `` // xtask-allow(state-fields): projection: written only by Facts::new, which copies the engine reader's node type for each id `` |
| `crates/kivact-test/src/facts.rs:12` | Node.parent | 閉じ方の変更 | projection | P8-KT | 唯一の書き手 Facts::new (facts.rs:46) が reader.parent(id) を写す | `` // xtask-allow(state-fields): projection: written only by Facts::new, which copies the engine reader's parent for each id `` |
| `crates/kivact-test/src/screen/disposal_tests.rs:31` | a_cleanup_writes_signals_and_registers_on_its_disposing_owner::slots | 確認済み | independent | — | render closure (disposal_tests.rs:38) が一度だけ埋め、試験本体 (:63) が取り出す SM-5 型の test slot。同関数の他 local は観測用 signal |  |
| `crates/kivact-test/src/screen/editing_drop_probe.rs:7` | Probe.armed | 誤（付録の設計） | type | P8-KT | SM-2：試験は reader を埋めた後 (host_sync_tests.rs:396) に armed を立て (:400)、Drop は armed の時 reader を unwrap (editing_drop_probe.rs:21)。armed ⇒ reader=Some。詳細 W2 | `` // xtask-allow(state-fields): independent: the reader and the other live node the test arms this probe with; an unarmed probe skips the drop check, and the shared counters count drops whatever it holds `` |
| `crates/kivact-test/src/screen/editing_drop_probe.rs:9` | Probe.drops | 確認済み | statistic | — | Drop (editing_drop_probe.rs:20) が加算、assert (host_sync_tests.rs:407-410) だけが読む。W2 後も残る |  |
| `crates/kivact-test/src/screen/editing_drop_probe.rs:11` | Probe.reentered | 確認済み | statistic | — | Drop (editing_drop_probe.rs:25) が加算、assert (host_sync_tests.rs:408) だけが読む。W2 後も残る |  |
| `crates/kivact-test/src/screen/editing_drop_probe.rs:13` | Probe.reader | 誤（付録の設計） | type | P8-KT | W2：armed と組で 'armed なら Some' の含意。armed: RefCell<Option<(Reader, NodeId)>> に統合して削除 |  |
| `crates/kivact-test/src/screen/host_sync_tests.rs:24` | rejected_pointer_down_cannot_retain_an_ended_region_gesture::presses | 確認済み | statistic | — | on_press (host_sync_tests.rs:61) が加算、assert (:88,:100) だけが読む |  |
| `crates/kivact-test/src/screen/host_sync_tests.rs:27` | rejected_pointer_down_cannot_retain_an_ended_region_gesture::downs | 確認済み | statistic | — | on_pointer (host_sync_tests.rs:48-50) が加算、assert (:79) だけが読む。handler の分岐には使わない |  |
| `crates/kivact-test/src/screen/host_sync_tests.rs:30` | rejected_pointer_down_cannot_retain_an_ended_region_gesture::disabled | 確認済み | independent | — | render closure (host_sync_tests.rs:36) が一度だけ埋め、試験本体 (:90) が取り出す SM-5 型の test slot |  |
| `crates/kivact-test/src/screen/host_sync_tests.rs:112` | a_primary_down_of_the_gesture_pointer_releases_a_gesture_whose_up_was_lost::presses | 確認済み | statistic | — | on_press (host_sync_tests.rs:148) が加算、assert (:187) だけが読む |  |
| `crates/kivact-test/src/screen/host_sync_tests.rs:115` | a_primary_down_of_the_gesture_pointer_releases_a_gesture_whose_up_was_lost::downs | 閉じ方の変更 | independent | P8-KT | host_sync_tests.rs:132-138 で observed.get()==1 の時だけ Handled::Yes を返す＝handler の振る舞いを分岐させる→statistic は誤り | `` // xtask-allow(state-fields): independent: counts the Down events the canvas handler sees and lets only the first start a gesture; the press counter counts activations separately, so any pair of counts is valid `` |
| `crates/kivact-test/src/screen/host_sync_tests.rs:352` | replacing_editing_drops_line_state_after_releasing_the_model::drops | 確認済み | statistic | — | Probe の Drop が加算、assert (host_sync_tests.rs:407,410) だけが読む |  |
| `crates/kivact-test/src/screen/host_sync_tests.rs:354` | replacing_editing_drops_line_state_after_releasing_the_model::reentered | 確認済み | statistic | — | Probe の Drop が加算、assert (host_sync_tests.rs:408) だけが読む |  |
| `crates/kivact-test/src/screen/host_sync_tests.rs:356` | replacing_editing_drops_line_state_after_releasing_the_model::reader | 誤（付録の設計） | type | P8-KT | W2：この slot は armed と組の含意の片割れ。local ごと削除し、各 probe の armed に直接入れる |  |
| `crates/kivact-test/src/screen/model.rs:33` | (no audit row: pub(super) last_contact: Cell<u64>,) | 確認済み | identity | — | 書き手 input.rs:257-258 (新しい contact に +1 の id を割当て) のみ。単なる id 割当て |  |
| `crates/kivact-test/src/screen/real_window_tests.rs:87` | virtual_time_drives_sleep_and_frame_loops_exactly::slot | 確認済み | independent | — | render closure (real_window_tests.rs:91) が一度だけ埋め、:96 で take する SM-5 型 test slot |  |
| `crates/kivact-test/src/screen/real_window_tests.rs:137` | a_display_rate_loop_follows_the_window_to_a_faster_display::count | 確認済み | statistic | — | frame_loop callback が加算、replace/get (real_window_tests.rs:148,155) と assert だけが読む |  |
| `crates/kivact-test/src/screen/real_window_tests.rs:139` | a_display_rate_loop_follows_the_window_to_a_faster_display::slot | 確認済み | independent | — | render closure (real_window_tests.rs:143) が一度だけ埋め、:146 で take する test slot |  |
| `crates/kivact-test/src/screen/real_window_tests.rs:226` | ticks::count | 確認済み | statistic | — | frame_loop callback が加算、assert と戻り値 (real_window_tests.rs:237-238) だけが読む |  |
| `crates/kivact-test/src/screen/real_window_tests.rs:228` | ticks::slot | 確認済み | independent | — | render closure (real_window_tests.rs:232) が一度だけ埋め、:235 で take する test slot |  |
| `dev/bench/src/bin/kv_heap.rs:35` | (no audit row: static PTRS: Mutex<Option<HashMap<usize, (usize, u64)>>> = Mutex::new(None);) | 誤（付録の設計） | type | P8-XT | SM-3：PTRS と SITES は record (kv_heap.rs:58-74) が同時に Some にし、PTRS の key は SITES の site を指す（forget :77-88）。'allocator が唯一の書き手・読み手' も誤り（dump :139- が読む）。詳細 W3。新 marker は統合後の static HEAP 用 | `` // xtask-allow(state-fields): independent: the probe's record of live allocations and their sites, filled by record and drained by forget under one lock; tracking only gates new records and the guard only blocks reentry, so it pairs with either of them `` |
| `dev/bench/src/bin/kv_heap.rs:37` | (no audit row: static SITES: Mutex<Option<HashMap<u64, Site>>> = Mutex::new(None);) | 誤（付録の設計） | type | P8-XT | W3：PTRS と常に同時に Some、索引の組。一つの Mutex<Option<Heap>> に統合 |  |
| `dev/bench/src/bin/kv_heap.rs:39` | (no audit row: static TRACKING: AtomicBool = AtomicBool::new(false);) | 文言 | independent | P8-XT | 書き手は main (kv_heap.rs:168, true) と dump (:138, false)、allocator は読むだけ。'allocator が唯一の書き手' は誤り | `` // xtask-allow(state-fields): independent: main turns recording on before the App starts and dump turns it off; frees are forgotten whether or not it is on, so it pairs with any recorded contents `` |
| `dev/bench/src/bin/kv_heap.rs:41` | (no audit row: thread_local! { static INSIDE: Cell<bool> = const { Cell::new(false) }; }) | 文言 | independent | P8-XT | alloc/dealloc の再入防止 (kv_heap.rs:96-108) と dump (:137) が書く。'allocator が唯一' は誤り、全組合せの理由が定型 | `` // xtask-allow(state-fields): independent: the per-thread reentrancy guard the allocator holds while it records or forgets, and dump keeps set; any thread can be inside with tracking on or off `` |
| `dev/tests/gpu-interop-producer/src/producer.rs:27` | Producer.shared | 閉じ方の変更 | independent | P8-GPU | shared_textures (producer.rs:135-138) と signal_prepared (:164-168) が実行時に take する→option は誤り。draw は共有前後どちらでも可（GPU が prepare signal を待つ :157-160） | `` // xtask-allow(state-fields): independent: the pool's handles until shared_textures or signal_prepared takes them, once per engine; draw numbers frames before or after that, since the engine's first frames wait for the preparation signal `` |
| `dev/tests/gpu-interop-producer/src/producer.rs:30` | Producer.next | 確認済み | identity | — | 書き手 draw (producer.rs:150) の +1 のみ、:144 で順序検査。frame 番号の割当て |  |
| `dev/xtask/src/check_source.rs:16` | Source.target | 誤（付録の設計） | type | P8-XT | load (check_source.rs:176-181) が library と target を同時に設定し target 文字列に '@bin:' で kind を埋込む。target=None は walk_file/fixture で library=true 固定。詳細 W4 |  |
| `dev/xtask/src/check_source.rs:18` | Source.library | 誤（付録の設計） | type | P8-XT | W4：library は target から導出可能、(None,false) 到達不能・(Some(lib名),false) は不正。enum Target に統合し library() accessor に |  |
| `dev/xtask/src/public_surface.rs:19` | Binding.public | 文言 | independent | P8-XT | 書き手 source_bindings (public_surface.rs:277,289) と expand_globs (:344) のみ。'separate from' は理由にならない | `` // xtask-allow(state-fields): independent: the visibility of the use or item that made the binding (or of the glob that copied it); public and private items of every kind exist, so it pairs with either value `` |
| `dev/xtask/src/public_surface.rs:22` | Binding.value | 文言 | independent | P8-XT | 書き手 source_bindings (public_surface.rs:275,291-294) と expand_globs (:346) のみ。'separate from its visibility' は理由にならない | `` // xtask-allow(state-fields): independent: whether the binding resolves to a fn, const or static; values and types are both exported and kept private, so it pairs with either visibility `` |

### B.8 G8

| marker（HEAD） | owner.field | 判定 | 閉じ方 | unit | 根拠 | 新しい marker 行 |
|---|---|---|---|---|---|---|
| `crates/kivact-ui-basic/src/attachment.rs:73` | Cfg.has_media | 閉じ方の変更 | option | P8-KITS | writer: attachment.rs:245 into_view で self.media.is_some()（.media() builder:134）の一度だけ。内部 event で不変。読み: root_style:188,222。caller の authored 値なので option。 | `` // xtask-allow(state-fields): option: set once in Attachment::into_view from whether the caller called .media(...); false means no media slot, the wider text padding and the 96 px vertical width `` |
| `crates/kivact-ui-basic/src/attachment.rs:75` | Cfg.actionable | 閉じ方の変更 | option | P8-KITS | writer: attachment.rs:247 into_view で self.on_press.is_some()（builder:158）の一度だけ。読み: root_style:233（cursor/hover）。caller の authored 値なので option。 | `` // xtask-allow(state-fields): option: set once in Attachment::into_view from whether the caller called .on_press(...); false means the chip is not pressable and gets no pointer cursor or hover fill `` |
| `crates/kivact-ui-basic/src/chart/model.rs:104` | Frame.horizontal | 閉じ方の変更 | option | P8-KITS | writer: plot.rs:30 Options::frame が Options.horizontal（view.rs:113 Chart::orientation builder）を描画毎に読むだけ。ChartConfig 由来ではない（理由文が事実と違う）。caller 設定値。 | `` // xtask-allow(state-fields): option: the caller's Chart::orientation (Horizontal), read per paint in Options::frame; false lays bars out as columns `` |
| `crates/kivact-ui-basic/src/chart/model.rs:106` | Frame.grid | 閉じ方の変更 | option | P8-KITS | writer: plot.rs:31 Options::frame ← view.rs:145 Chart::grid（既定 true:90）。読み: cartesian.rs:176, radar.rs:64。caller 設定値。 | `` // xtask-allow(state-fields): option: the caller's Chart::grid (default on), read per paint in Options::frame; false skips the grid lines `` |
| `crates/kivact-ui-basic/src/chart/model.rs:108` | Frame.axis | 閉じ方の変更 | option | P8-KITS | writer: plot.rs:32 Options::frame ← view.rs:152 Chart::axis（既定 true:91）。読み: cartesian.rs:54,72,77,162, radar.rs:31,67。caller 設定値。 | `` // xtask-allow(state-fields): option: the caller's Chart::axis (default on), read per paint in Options::frame; false drops the axis labels and their gutter `` |
| `crates/kivact-ui-basic/src/questionnaire.rs:104` | QuestionnaireStep.description | 確認済み | option | — | writer: questionnaire.rs:116,126（None）と builder:134 のみ。読み: 333,384,396。caller 設定値で absent の意味も正確。 |  |
| `crates/kivact-ui-basic/src/questionnaire.rs:107` | QuestionnaireStep.optional | 閉じ方の変更 | option | P8-KITS | writer: questionnaire.rs:118,128（false）と builder QuestionnaireStep::optional:150 のみ。読み: 534（Skip 表示 548、blocked 564）。builder の authored 値なので option。 | `` // xtask-allow(state-fields): option: the caller's QuestionnaireStep::optional (default false); false means the step has no Skip button and Next waits for an answer `` |
| `crates/kivact-ui-basic/src/sidebar/menu.rs:25` | SidebarGroup.action | 文言 | option | P8-KITS | writer: menu.rs:46（None）, builder:54 のみ。読み: into_view:93 で map するだけ＝absent は何も出さない。「component's default」は存在せず文言が事実と違う。 | `` // xtask-allow(state-fields): option: the caller's SidebarGroup::action; absent means the label row has no end control `` |
| `crates/kivact-ui-basic/src/sidebar/menu.rs:27` | SidebarGroup.content | 文言 | option | P8-KITS | writer: menu.rs:47（None）, builder:60 のみ。読み: into_view:108 map。absent は本体なし（既定の content は無い）。 | `` // xtask-allow(state-fields): option: the caller's SidebarGroup::content; absent means the group shows only its label `` |
| `crates/kivact-ui-basic/src/sidebar/menu.rs:168` | SidebarMenuButton.icon | 文言 | option | P8-KITS | writer: menu.rs:203（None）, builder:214 のみ。読み: build_button:388 .child(icon)。absent は icon 無し（既定 icon は無い）。 | `` // xtask-allow(state-fields): option: the caller's SidebarMenuButton::icon; absent means the row has no leading icon `` |
| `crates/kivact-ui-basic/src/sidebar/menu.rs:171` | SidebarMenuButton.badge | 文言 | option | P8-KITS | writer: menu.rs:205（None）, builder:229 のみ。読み: build_button:367 map(menu_badge)。absent は件数表示なし。 | `` // xtask-allow(state-fields): option: the caller's SidebarMenuButton::badge; absent means the row shows no count `` |
| `crates/kivact-ui-basic/src/sidebar/menu.rs:173` | SidebarMenuButton.action | 文言 | option | P8-KITS | writer: menu.rs:206（None）, builder:235 のみ。読み: build_button:352（has_action→padding 320）,406。absent は末尾 control と 32px 余白なし。 | `` // xtask-allow(state-fields): option: the caller's SidebarMenuButton::action; absent means no end control and no 32 px end padding `` |
| `crates/kivact-ui-basic/src/sidebar/menu.rs:177` | SidebarMenuButton.sub | 文言 | option | P8-KITS | writer: menu.rs:209（None）, builder:258 のみ。読み: build_button:407 map(menu_sub)。absent は入れ子 list なし。 | `` // xtask-allow(state-fields): option: the caller's SidebarMenuButton::sub; absent means the row has no nested list `` |
| `crates/kivact-ui-basic/src/sidebar/menu.rs:184` | ButtonCfg.cx | 確認済み | option | — | writer: build_button:356 の current()（state.rs:74 try_use_context）一度だけ。読み: button_style:290。None は provider 外で文言どおり。 |  |
| `crates/kivact-ui-basic/src/sidebar/panel.rs:25` | Sidebar.header | 文言 | option | P8-KITS | writer: panel.rs:44（None）, builder:72 のみ。読み: build:184 section(header)=map。absent は上部 section なし（既定は無い）。 | `` // xtask-allow(state-fields): option: the caller's Sidebar::header; absent means the panel has no top section `` |
| `crates/kivact-ui-basic/src/sidebar/panel.rs:27` | Sidebar.content | 文言 | option | P8-KITS | writer: panel.rs:45, builder:78 のみ。読み: build:148 map(scroll)。absent は中央 scroll 部なし。 | `` // xtask-allow(state-fields): option: the caller's Sidebar::content; absent means the panel has no scrolling middle section `` |
| `crates/kivact-ui-basic/src/sidebar/panel.rs:29` | Sidebar.footer | 文言 | option | P8-KITS | writer: panel.rs:46, builder:84 のみ。読み: build:186 section(footer)。absent は下部 section なし。 | `` // xtask-allow(state-fields): option: the caller's Sidebar::footer; absent means the panel has no bottom section `` |
| `crates/kivact-ui-creative/src/dock/model.rs:56` | DockTab.icon | 確認済み | option | — | writer: model.rs:68（None）, builder DockTab::icon:75 のみ（pub field は app が直接書く）。読み: chrome.rs:190。absent=icon なしで正確。 |  |
| `crates/kivact-ui-creative/src/dock/model.rs:58` | DockTab.closable | 閉じ方の変更 | option | P8-KITS | writer: model.rs:69（true）, builder:82 のみ。crate 内に他の書き手なし。doc（:78-80）が layout 値内の定数と明記。読み: chrome.rs:391,398,403,624, model.rs:486,502。app の authored 値。 | `` // xtask-allow(state-fields): option: the application's DockTab::closable (default true), a constant inside the layout value; false hides the close button and makes the close commands skip the tab `` |
| `crates/kivact-ui-creative/src/editors/arrangement/model.rs:58` | TimelineTrack.muted | 文言 | independent | P8-KITS | writers: model.rs:77（new）, app が ArrangementRequest::TrackFlag を適用（例 arrangement_model.rs:113）。widget 内は TrackFlag::checked:16 で読むだけ。組合せ制約なし。現文は書き手のみで有効性の理由なし。 | `` // xtask-allow(state-fields): independent: the application's track mute, changed only on a TrackFlag::Mute request from its header toggle; a muted track may also be soloed, armed or reading automation, as on a mixer `` |
| `crates/kivact-ui-creative/src/editors/arrangement/model.rs:60` | TimelineTrack.solo | 文言 | independent | P8-KITS | writers: model.rs:78, app の TrackFlag::Solo 適用（arrangement_model.rs:114）のみ。他 flag を書かない。現文は有効性の理由なし。 | `` // xtask-allow(state-fields): independent: the application's track solo, changed only on a TrackFlag::Solo request from its header toggle; soloing never clears mute, arm or automation read, so it pairs with any of them `` |
| `crates/kivact-ui-creative/src/editors/arrangement/model.rs:62` | TimelineTrack.armed | 文言 | independent | P8-KITS | writers: model.rs:79, app の TrackFlag::Arm 適用（arrangement_model.rs:115）のみ。現文は有効性の理由なし。 | `` // xtask-allow(state-fields): independent: the application's record arm, changed only on a TrackFlag::Arm request from its header toggle; an armed track may be muted, soloed or reading automation `` |
| `crates/kivact-ui-creative/src/editors/arrangement/model.rs:64` | TimelineTrack.automation_read | 文言 | independent | P8-KITS | writers: model.rs:80, TimelineTrack::automation:87（true）, app の TrackFlag::AutomationRead 適用（arrangement_model.rs:116）。現文は builder の書き込み:87 を落としている。 | `` // xtask-allow(state-fields): independent: the application's automation read, set by TimelineTrack::automation and changed on a TrackFlag::AutomationRead request; the widget only shows it as a header toggle, so it pairs with any mute, solo or arm `` |
| `crates/kivact-ui-creative/src/editors/node/gesture.rs:250` | DragMode::Link.released | 確認済み | independent | — | writers: input.rs:303,317（None で開始）, input.rs:120-127（Up で socket か None）。読み: finish_fields:188（None は近傍探索）。source 両 variant で成立。注記: 数える fie… |  |
| `crates/kivact-ui-creative/src/editors/node/model.rs:46` | NodeData.preview | 確認済み | option | — | writer: model.rs:64（None）, builder NodeData::preview:105 のみ（app 供給の画像）。読み: paint.rs:181。正確。 |  |
| `crates/kivact-ui-creative/src/editors/node/model.rs:49` | NodeData.muted | 文言 | independent | P8-KITS | writers: model.rs:65, app が NodeRequest::ToggleMute（input.rs:241-253 の M key）を適用（node_model.rs:87）。読み: paint.rs:95 で暗くするだけ。「either value is valid」だけで理由不足。 | `` // xtask-allow(state-fields): independent: the application's node mute, flipped by the application on NodeRequest::ToggleMute (M key); it only dims the node in paint, so a muted node may have a preview image or not `` |
| `crates/kivact-ui-creative/src/editors/shared/lane.rs:17` | LaneExtensions.menu | 確認済み | option | — | writer: Default（None）と lane.rs:79 の builder macro（menu_factory）のみ。読み: graph/view.rs:177, time/view.rs:116。注記: 数える field 1 個で lint 非発火（check.… |  |
| `crates/kivact-ui-creative/src/editors/time/view.rs:28` | TimeEditor.lanes | 文言 | option | P8-KITS | writer: view.rs:57（None）, builder.rs:40 .lanes() のみ。読み: visible_lanes:256-265 で None は item の lane id 毎の無名 lane。「its own lanes」は曖昧。注記: 数える field 1 個で lint 非発火（check.rs:199,306）→marker 無効果、削除可。 | `` // xtask-allow(state-fields): option: the caller's .lanes(...) builder; absent means one unnamed lane per distinct item lane id (visible_lanes) `` |
| `crates/kivact-ui-creative/src/numeric/implementation.rs:458` | NumericView.timecode | 確認済み | option | — | writer: builder timecode:242 → 構築:408 で copy のみ。読み: 391,392,595（None は通常の数値書式）。正確。 |  |
| `crates/kivact-ui-creative/src/numeric/implementation.rs:461` | NumericView.accent | 確認済み | option | — | writer: builder:210 → NumericView 構築で copy のみ。読み: 763 で None は p.accent。正確。 |  |
| `crates/kivact-ui-creative/src/outliner/view.rs:232` | RowFactory.row_height | 確認済み | option | — | writer: builder Outliner::row_height:181 → 構築:500 copy のみ。読み: row_style:417 で None は p.row。正確。 |  |
| `crates/kivact-ui-creative/src/outliner/view.rs:236` | RowFactory.has_activation | 文言 | option | P8-KITS | writer: view.rs:467 activation.is_some()（builder on_activate:131）→:503 のみ。読み: 673 double click rename 分岐。bool なのに「absent」と書き、誰が与えるかが無い。 | `` // xtask-allow(state-fields): option: whether the caller set Outliner::on_activate; false means a double click renames the row instead of activating it `` |
| `crates/kivact-ui-creative/src/outliner_data.rs:12` | TreeItem.parent | 文言 | independent | P8-KITS | writers: TreeItem::new:37 と app（TreeDrop 適用）。widget は読むだけ（reveal.rs:119, drop_plan.rs:69,90）。他 flag と含意なし。現文は「any parent or none is valid」のみ。 | `` // xtask-allow(state-fields): independent: the application's hierarchy, given in TreeItem::new and changed by the application after a TreeDrop; a root or a child may carry any visibility, selectability, render or placeholder flag `` |
| `crates/kivact-ui-creative/src/outliner_data.rs:17` | TreeItem.visible | 文言 | independent | P8-KITS | writers: new:41 と app（Restriction{Visible} 提案、H key view.rs:653）。読み: view.rs:432 で label を暗くするだけ、controls.rs:103。他 flag と含意なし。現文は定型。 | `` // xtask-allow(state-fields): independent: the application's visibility, changed on a Restriction{flag: Visible} proposal (eye toggle or H); it only dims the label, so a hidden item can still be selectable, rendered or a placeholder `` |
| `crates/kivact-ui-creative/src/outliner_data.rs:19` | TreeItem.selectable | 文言 | independent | P8-KITS | writers: new:42 と app（Restriction{Selectable}）。読み: view.rs:729 disabled、controls.rs:104。含意なし。現文は定型。 | `` // xtask-allow(state-fields): independent: the application's selectability, changed on a Restriction{flag: Selectable} proposal; it only disables the tree node, so it pairs with any visibility, render or placeholder state `` |
| `crates/kivact-ui-creative/src/outliner_data.rs:21` | TreeItem.rendered | 文言 | independent | P8-KITS | writers: new:43 と app（Restriction{Rendered}）。読み: controls.rs:105 の toggle 表示のみ。含意なし。現文は定型。 | `` // xtask-allow(state-fields): independent: the application's render toggle, changed on a Restriction{flag: Rendered} proposal; the outliner only shows it in its toggle column, so it pairs with any visibility, selectability or placeholder state `` |
| `crates/kivact-ui-creative/src/outliner_data.rs:25` | TreeItem.placeholder | 文言 | independent | P8-KITS | writers: new:45, builder placeholder:57 のみ（app）。読み: view.rs:280,290 で italic/muted のみ、選択・control は不変（doc:54）。現文は他 field との関係を述べない。 | `` // xtask-allow(state-fields): independent: the application marks an unresolved item with TreeItem::placeholder; it only draws the label italic and muted, so the item keeps its parent and all restriction toggles `` |
| `crates/kivact-ui-creative/src/outliner_data.rs:72` | TreeDrop.target | 文言 | independent | P8-KITS | writer: drop_plan.rs:95-110 のみ（app ではなく widget が計算）。Into→Some(over)、Before/After→over の parent（root なら None）。after との 4 通り全て到達。現文「the application's new parent」は事実と違う。 | `` // xtask-allow(state-fields): independent: the new parent drop_plan computes from the row under the pointer; None makes a root, and both a root and a child drop may land first (after None) or after a sibling `` |
| `crates/kivact-ui-creative/src/outliner_data.rs:75` | TreeDrop.after | 文言 | independent | P8-KITS | writer: drop_plan.rs:98（Into: 最後の子 or None）,107-（After→Some(over)、Before→前の兄弟 or None）のみ。target の Some/None いずれとも成立。現文は組合せ理由なし。 | `` // xtask-allow(state-fields): independent: the sibling under target that drop_plan puts the item after; None puts it first, which a drop into a root list and into a parent both allow `` |
| `crates/kivact-ui-creative/src/picker.rs:60` | PickerItem.section | 確認済み | option | — | writer: new:86（None）, builder section:96 のみ。読み: 375,894,957。absent は見出しなしで正確。 |  |
| `crates/kivact-ui-creative/src/picker.rs:64` | PickerItem.description | 確認済み | option | — | writer: new（None）, builder description:102 のみ。正確。 |  |
| `crates/kivact-ui-creative/src/picker.rs:67` | PickerItem.shortcut | 確認済み | option | — | writer: new（None）, builder shortcut:108 のみ。正確。 |  |
| `crates/kivact-ui-creative/src/picker.rs:70` | PickerItem.icon | 確認済み | option | — | writer: new（None）, builder icon:115 のみ。正確。 |  |
| `crates/kivact-ui-creative/src/picker.rs:73` | PickerItem.enabled | 閉じ方の変更 | option | P8-KITS | writer: new:89（true）, builder enabled:123 のみ。doc:119-121 が items 値内の定数と明記。読み: 396,403,647,997。app の authored 値で実行時 event では変わらない。 | `` // xtask-allow(state-fields): option: the application's PickerItem::enabled (default true), a constant inside the items value; false keeps the row listed but never active or pickable `` |
| `crates/kivact-ui-creative/src/picker.rs:600` | Session.column | 確認済み | option | — | writer: picker.rs:673 multiple.then_some(columns)（builder column:223 は setup only）のみ。absent=単一列で正確。注記: 数える field 1 個で lint 非発火（check.rs:199,… |  |
| `examples/kivact-ui-basic/src/docs/data_table.rs:64` | payments::sort | 文言 | independent | P8-KITS | writers: data_table の header（kivact-ui runtime.rs:439-443、新しい sort で page=0）。sort は filter 後の行の並べ替えのみで page と含意なし。現文は page との関係を述べない。 | `` // xtask-allow(state-fields): independent: the table's sort binding, written by the data_table column headers; any sort can be shown on any page, since sorting only reorders the filtered rows `` |
| `examples/kivact-ui-basic/src/docs/data_table.rs:66` | payments::page | 文言 | independent | P8-KITS | writers: pager, filter 入力 data_table.rs:100（page.set(0)）, 新 sort（runtime.rs:442）。0 に戻った後は任意 page へ移動可。現文は sort との組合せ理由なし。 | `` // xtask-allow(state-fields): independent: the table's page binding, moved by the pager and reset to 0 by a new sort or filter text; every page is reachable afterwards under any sort `` |
| `examples/kivact-ui-basic/src/docs/drawer.rs:35` | default::open | 文言 | independent | P8-KITS | writers: drawer.rs:43 set(true), Submit/Cancel:79,82 set(false)。goal は閉じても保持。現文は goal との組合せ理由なし。 | `` // xtask-allow(state-fields): independent: the drawer's open binding, set by Open Drawer and cleared by Submit or Cancel; the goal keeps its value while the drawer is closed, so either open state pairs with any goal `` |
| `examples/kivact-ui-basic/src/docs/drawer.rs:37` | default::goal | 文言 | independent | P8-KITS | writers: drawer.rs:57,68 の ±10 のみ。閉じる時に reset しない。現文「any daily goal is valid」は値域のみ。 | `` // xtask-allow(state-fields): independent: the calorie goal, stepped by the minus and plus buttons in the drawer; nothing resets it on close, so every goal pairs with either drawer state `` |
| `examples/kivact-ui-basic/src/docs/dropdown_menu.rs:75` | checkboxes::status | 文言 | independent | P8-KITS | writer: on_activate:88 "status" の反転のみ。activity は別 entry:89。現文は 2 field で同一の定型（書き手のみ）。 | `` // xtask-allow(state-fields): independent: the Status Bar checkbox, flipped only by its own menu entry; the Activity Bar entry never touches it, so both, either or neither bar can be on `` |
| `examples/kivact-ui-basic/src/docs/dropdown_menu.rs:77` | checkboxes::activity | 文言 | independent | P8-KITS | writer: on_activate:89 の反転のみ。現文は status と同一の定型。 | `` // xtask-allow(state-fields): independent: the Activity Bar checkbox, flipped only by its own menu entry; the Status Bar entry never touches it, so both, either or neither bar can be on `` |
| `examples/kivact-ui-basic/src/docs/trace.rs:64` | steps::first | 文言 | independent | P8-KITS | writer: trace_collapsible(first,…):70 の header toggle のみ。second は別 binding。現文は同一定型。 | `` // xtask-allow(state-fields): independent: the open binding of the first trace collapsible, toggled only by its header; the second collapsible has its own binding, so both, either or neither can be open `` |
| `examples/kivact-ui-basic/src/docs/trace.rs:66` | steps::second | 文言 | independent | P8-KITS | writer: trace_collapsible(second,…):75 の header toggle のみ。現文は同一定型。 | `` // xtask-allow(state-fields): independent: the open binding of the second trace collapsible, toggled only by its header; the first collapsible has its own binding, so both, either or neither can be open `` |
| `examples/kivact-ui-creative/src/creative/layout.rs:20` | properties::tab | 文言 | independent | P8-KITS | writer: inspector_tabs:856 tab.set(i) のみ。display_* を触らない。現文「any tab is valid」は値域のみ。 | `` // xtask-allow(state-fields): independent: the inspector tab strip's selection, set only by inspector_tabs; it never touches the viewport display checkboxes, so any tab pairs with any display setting `` |
| `examples/kivact-ui-creative/src/creative/layout.rs:23` | properties::display_name | 文言 | independent | P8-KITS | writer: checkbox("Name", display_name):52 のみ。現文は「every other display flag と有効」の定型。 | `` // xtask-allow(state-fields): independent: bound only to the Name checkbox of Viewport Display; Axes and Wireframe have their own signals, so every on/off mix is a valid display `` |
| `examples/kivact-ui-creative/src/creative/layout.rs:25` | properties::display_axes | 文言 | independent | P8-KITS | writer: checkbox("Axes", display_axes):53 のみ。現文は定型。 | `` // xtask-allow(state-fields): independent: bound only to the Axes checkbox of Viewport Display; Name and Wireframe have their own signals, so every on/off mix is a valid display `` |
| `examples/kivact-ui-creative/src/creative/layout.rs:27` | properties::display_wireframe | 文言 | independent | P8-KITS | writer: checkbox("Wireframe", display_wireframe):54 のみ。現文は定型。 | `` // xtask-allow(state-fields): independent: bound only to the Wireframe checkbox of Viewport Display; Name and Axes have their own signals, so every on/off mix is a valid display `` |
| `examples/kivact-ui-game/src/game/controls.rs:67` | buttons::filter_on | P5 に従う | independent | — | P5 付録 E-10（`controls.rs:64,66` と書かれているのは HEAD の 65・67 行）が grid_on と同じ文言で確定済み。本表の文言は採らない。 元の判定：writer: button_toolbar:363 の反転のみ（grid は :357）。現文は grid_on と同一で組合せ理由なし。 |  |
| `examples/kivact-ui-game/src/game/controls.rs:164` | navigation::step | 文言 | independent | P8-KITS | writer: wizard_steps（stepper:913, back:922）のみ。page は pagination_controls:967。現文は page と同一定型。 | `` // xtask-allow(state-fields): independent: the wizard step, moved only by the stepper's controls in wizard_steps; the pagination specimen has its own page signal, so any step pairs with any page `` |
| `examples/kivact-ui-game/src/game/controls.rs:166` | navigation::page | 文言 | independent | P8-KITS | writer: pagination(12, page):967 のみ。現文は step と同一定型。 | `` // xtask-allow(state-fields): independent: the pagination specimen's page, moved only by its pagination control; the wizard has its own step signal, so any page pairs with any step `` |
| `examples/kivact-ui-game/src/game/system.rs:72` | feedback::info | 文言 | independent | P8-KITS | writer: alert .open(info):353 の閉じるのみ（true→false、戻さない）。4 alert とも別 binding で 16 通り全て到達。現文は 4 field 同一。 | `` // xtask-allow(state-fields): independent: the info alert's open binding, cleared only by its own close button and never set back; the other alerts have their own bindings, so any subset of the four can be dismissed `` |
| `examples/kivact-ui-game/src/game/system.rs:74` | feedback::success | 文言 | independent | P8-KITS | writer: alert .open(success) の閉じるのみ。現文は 4 field 同一。 | `` // xtask-allow(state-fields): independent: the success alert's open binding, cleared only by its own close button and never set back; the other alerts have their own bindings, so any subset of the four can be dismissed `` |
| `examples/kivact-ui-game/src/game/system.rs:76` | feedback::warning | 文言 | independent | P8-KITS | writer: alert .open(warning):402 の閉じるのみ。現文は 4 field 同一。 | `` // xtask-allow(state-fields): independent: the warning alert's open binding, cleared only by its own close button and never set back; the other alerts have their own bindings, so any subset of the four can be dismissed `` |
| `examples/kivact-ui-game/src/game/system.rs:78` | feedback::danger | 文言 | independent | P8-KITS | writer: alert .open(danger):412 の閉じるのみ。現文は 4 field 同一。 | `` // xtask-allow(state-fields): independent: the danger alert's open binding, cleared only by its own close button and never set back; the other alerts have their own bindings, so any subset of the four can be dismissed `` |
| `examples/kivact-ui-game/src/game/system.rs:92` | overlays::pinned | 文言 | independent | P8-KITS | writer: dropdown_menu on_activate:522 の反転のみ。archived:523, user_open は別。現文は archived と同一定型。 | `` // xtask-allow(state-fields): independent: the detail-column menu checkbox, flipped only by its own entry; archived and the user popover have their own signals, so it pairs with either of their states `` |
| `examples/kivact-ui-game/src/game/system.rs:94` | overlays::archived | 文言 | independent | P8-KITS | writer: on_activate:523 の反転のみ。現文は pinned と同一定型。 | `` // xtask-allow(state-fields): independent: the include-archived menu checkbox, flipped only by its own entry; pinned and the user popover have their own signals, so it pairs with either of their states `` |
| `examples/kivact-ui-game/src/game/system.rs:96` | overlays::user_open | 文言 | independent | P8-KITS | writer: user_popover の dialog(…, user_open):633（trigger と閉じる）のみ。checkbox を読み書きしない。現文は組合せ理由が弱い。 | `` // xtask-allow(state-fields): independent: the user popover's open binding, set by its own trigger and cleared by the popover; it never reads or writes the menu checkboxes, so it opens with any checkbox pair `` |
| `examples/kivact-ui-game/src/game/system.rs:259` | layout::notify | 文言 | independent | P8-KITS | writer: document_inspector の switch("", notify):821 のみ。opacity は slider。現文は同一定型。 | `` // xtask-allow(state-fields): independent: bound only to the inspector's notify switch; the opacity slider has its own signal, so notifications on or off pair with any opacity `` |
| `examples/kivact-ui-game/src/game/system.rs:261` | layout::opacity | 文言 | independent | P8-KITS | writer: slider("Inspector 不透明度", opacity):836 のみ。現文は同一定型。 | `` // xtask-allow(state-fields): independent: bound only to the inspector's opacity slider; the notify switch has its own signal, so any opacity pairs with notifications on or off `` |
| `examples/kivact-ui-game/src/game/window.rs:44` | frame::width | 文言 | independent | P8-KITS | writer: native_window の divider（explorer_range 140–420）のみ。mode/run/help と無関係。現文は値域のみ。 | `` // xtask-allow(state-fields): independent: the explorer width, written only by the native_window divider within explorer_range; it is layout only, so any width pairs with any editor mode, run state or About dialog `` |
| `examples/kivact-ui-game/src/game/window.rs:47` | frame::help | 文言 | independent | P8-KITS | writer: command "help":71 set(true) と dialog の閉じるのみ。mode/run/width を触らない。現文は書き手のみ。 | `` // xtask-allow(state-fields): independent: the About dialog's open binding, set by the Help menu's help command and cleared by the dialog; it never touches mode, run or the explorer width, so it opens over any editor state `` |
| `examples/kivact-ui-pop/src/pop/components.rs:457` | switch_controls::volume | 文言 | independent | P8-POP | writer: slider("BGM音量", volume):472 のみ。現文は 2 field 同一。 | `` // xtask-allow(state-fields): independent: bound only to the BGM volume slider; the difficulty slider has its own signal, so any volume pairs with any difficulty `` |
| `examples/kivact-ui-pop/src/pop/components.rs:459` | switch_controls::difficulty | 文言 | independent | P8-POP | writer: slider("難易度補正", difficulty):492 のみ。現文は同一定型。 | `` // xtask-allow(state-fields): independent: bound only to the difficulty slider; the volume slider has its own signal, so any difficulty pairs with any volume `` |
| `examples/kivact-ui-pop/src/pop/components.rs:515` | check_controls::notifications | 文言 | independent | P8-POP | writer: checkbox("通知を受け取る"):529 のみ。現文は同一定型。 | `` // xtask-allow(state-fields): independent: bound only to the notifications checkbox; the weekly report checkbox has its own signal, so both, either or neither can be checked `` |
| `examples/kivact-ui-pop/src/pop/components.rs:517` | check_controls::reports | 文言 | independent | P8-POP | writer: checkbox("週次レポート"):534 のみ。現文は同一定型。 | `` // xtask-allow(state-fields): independent: bound only to the weekly report checkbox; the notifications checkbox has its own signal, so both, either or neither can be checked `` |
| `examples/kivact-ui-pop/src/pop/data.rs:41` | display::next_chip | 文言 | independent | P8-POP | writer: badge_examples:670 の追加 button のみ。live は spawn loop:45-55。現文は同一定型「unrelated」。 | `` // xtask-allow(state-fields): independent: the index of the next suggested chip tag, advanced only by the badge specimen's add button; the progress loop never reads it, so any index pairs with any progress `` |
| `examples/kivact-ui-pop/src/pop/data.rs:43` | display::live | 文言 | independent | P8-POP | writer: spawn loop data.rs:45-55（900ms 毎）のみ。現文は同一定型。 | `` // xtask-allow(state-fields): independent: the demo progress value, advanced only by the 900 ms spawn loop; the chip suggestion never reads it, so any progress pairs with any chip index `` |
| `examples/kivact-ui-pop/src/pop/data.rs:73` | table::page | 文言 | independent | P8-POP | writers: pager, 新 sort（kivact-ui runtime.rs:442）、status/search（data.rs:518,528 page.set(0)）。「separate controls」は事実と違う（sort が page を書く）。 | `` // xtask-allow(state-fields): independent: the student table's page, moved by the pager and reset to 0 by a new sort, status filter or search; every page is reachable afterwards under any sort `` |
| `examples/kivact-ui-pop/src/pop/data.rs:75` | table::sort | 文言 | independent | P8-POP | writer: data_table header（runtime.rs:439-443）のみ。並べ替えは filter 後の行順だけで page と含意なし。現文は page と同一定型。 | `` // xtask-allow(state-fields): independent: the student table's sort binding, written by the column headers; sorting only reorders the filtered rows, so any sort can be shown on any page `` |
| `examples/kivact-ui-pop/src/pop/mission.rs:17` | Mission.progress_label | 確認済み | option | — | writer: patterns.rs:281,299,317 の Mission literal のみ。読み: or_message（None は mission.progress）。正確。 |  |
| `examples/kivact-ui-pop/src/pop/mission.rs:24` | Mission.action_label | 確認済み | option | — | writer: patterns.rs:286,304,322 の literal のみ。None は mission.open。正確。 |  |
| `examples/kivact-ui-pop/src/pop/mission.rs:28` | Mission.locked | 閉じ方の変更 | option | P8-POP | writer: patterns.rs:288,306,324 の literal（app の fixture）のみ、実行時に書き換えなし。読み: mission.rs:89,197（彩度）,206（lock icon）,215（disabled）。authored 値なので option。 | `` // xtask-allow(state-fields): option: the application's lock flag in each Mission literal (patterns.rs); true desaturates the card art, shows a lock icon and disables the action button `` |
| `examples/kivact-ui-pop/src/pop/patterns.rs:19` | feedback::dismissed_info | 文言 | independent | P8-POP | writer: maintenance_notice:370 set(true) のみ（戻さない）。success は :398。現文は同一定型。 | `` // xtask-allow(state-fields): independent: set only by the maintenance notice's close button and never cleared; the success notice has its own signal, so both, either or neither notice can be dismissed `` |
| `examples/kivact-ui-pop/src/pop/patterns.rs:21` | feedback::dismissed_success | 文言 | independent | P8-POP | writer: success_notice:398 set(true) のみ。現文は同一定型。 | `` // xtask-allow(state-fields): independent: set only by the success notice's close button and never cleared; the maintenance notice has its own signal, so both, either or neither notice can be dismissed `` |
| `examples/kivact-ui-pop/src/pop/shell.rs:638` | student_drawer::pin_student | 文言 | independent | P8-POP | writer: switch("メイン編成に固定", pin_student):664 のみ。現文は同一定型。 | `` // xtask-allow(state-fields): independent: bound only to the pin-to-main-team switch in the student drawer; the auto-skill switch has its own signal, so both, either or neither can be on `` |
| `examples/kivact-ui-pop/src/pop/shell.rs:640` | student_drawer::auto_skill | 文言 | independent | P8-POP | writer: switch("スキル自動使用", auto_skill):671 のみ。現文は同一定型。 | `` // xtask-allow(state-fields): independent: bound only to the auto-skill switch in the student drawer; the pin switch has its own signal, so both, either or neither can be on `` |
| `examples/kivact-ui-soft/src/model.rs:8` | CatalogState.dark | 文言 | independent | P8-KITS | writers: model.rs:22（caller の ThemeMode）, shell.rs:59 Toggle appearance の反転。「caller-controlled」は利用者 toggle を落としていて事実と違う。読み: shell.rs:29,56。 | `` // xtask-allow(state-fields): independent: the appearance, seeded from the caller's ThemeMode and flipped by the Toggle appearance button; it only picks the theme, so every page and query can be shown in light or dark `` |
| `examples/kivact-ui-soft/src/model.rs:10` | CatalogState.selected | 文言 | independent | P8-KITS | writers: model.rs:23（caller の slug）, shell.rs:117 nav button のみ。dark を触らない。現文は「independent of」の言い換えのみ。 | `` // xtask-allow(state-fields): independent: the shown page, seeded from the caller's slug and set by the navigation buttons; the theme toggle never writes it, so any page shows in light or dark `` |
| `examples/kivact/component/src/main.rs:33` | app::first | 文言 | independent | P8-KITS | writer: card on_close:40 set(false) のみ（戻さない）。second は :49。現文は同一定型。 | `` // xtask-allow(state-fields): independent: whether the First card is shown, cleared only by its own close button and never set back; the Second card has its own signal, so either card or both can be closed `` |
| `examples/kivact/component/src/main.rs:35` | app::second | 文言 | independent | P8-KITS | writer: card on_close:49 set(false) のみ。現文は同一定型。 | `` // xtask-allow(state-fields): independent: whether the Second card is shown, cleared only by its own close button and never set back; the First card has its own signal, so either card or both can be closed `` |
