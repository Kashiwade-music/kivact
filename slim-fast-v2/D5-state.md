# design-P5：AC-3 暗黙の状態機械（OPEN 25 件、旧付録 A の照合、lint の穴）

> **V2 付属書 D5**（設計担当 P5 の成果物）。正本は [SLIM_FAST_V2_DESIGN.md](../SLIM_FAST_V2_DESIGN.md)、手順は [SLIM_FAST_V2_WORK_ORDER.md](../SLIM_FAST_V2_WORK_ORDER.md)。
> - 本書の「8 章 設計者（人間）の判断が要る点」は V2 設計書 4 章（DEC-01〜DEC-23）で決定済み。実装者は 8 章を読んで選ばない。V2 設計書 5 章（付属書間の調整）が本書を上書きする箇所がある。
> - 本書の unit 名と V2 の unit ID の対応は V2 設計書 0.2 節。unit の実施順・lane・wave は V2 作業指示書だけが決める（本書 6 章の「依存」「並列可」は参考）。
> - 文中の P1〜P8 は付属書 D1〜D8（`design-P<n>.md` は `D<n>-*.md`）を指す。`W:\w\kv-prof`・`W:\w\kv-mem`・`W:\w\kv-size`・scratchpad の path は設計時の試作・計測の場所（設計した機械にだけある参考）で、実装に必要ではない。行番号は HEAD `73c36c7c` のもの。

担当 P5。対象 HEAD 73c36c7c（= db8d679b + 引継ぎ文書）。リポジトリは読取りのみで、編集していない（`--report-audit` を 1 回実行。生成物の差分は無かった）。

本書の構成：本文 1〜8 章（統合した決定・lint の設計・手順・unit）と、件ごとの詳細を書いた付録 P5-A〜P5-F。付録は各 slice の設計で、**本文 3.6 節の「統合での上書き」を反映済み**。本文と付録が食い違って見える場合は本文が正。

---

## 1 事実

### 1.1 audit の実行結果（HEAD）

```
$ cargo xtask check --no-write --only state-fields --report-audit
state-audit: 27 open row(s) in dev/xtask/state-audit.md     # exit 1、約 13 秒
$ git status --short                                          # 差分なし（HEAD の audit と同一）
```

- 行数 1239。閉じ方の内訳：option 558、independent 342、cache 117、projection 101、type 40、identity 32、statistic 22、OPEN 27。
- 通常の gate（`--report-audit` なし）は 27 OPEN のまま成功する。理由は `dev/xtask/src/state_fields/check.rs` の二つの条件の食い違い：
  - error の条件（`report_combination`）：無印の stored が 2 個以上、または phase があって無印の stored が 1 個以上。
  - audit の OPEN（`Counting::rows`）：数える field が 2 個以上ある struct の無印の field。
  - したがって「印付き 1 個＋無印 1 個」は error にならず OPEN だけになる。27 件はすべてこの形。locals も同じ（`locals::check` は無印 2 個以上だけを error にする）。

### 1.2 marker の実態（audit と `git grep` から採取）

| 事実 | 数 | 証拠 |
|---|---|---|
| 理由文が複数の owner で同一 | 57 種、514 行 | audit の reason 列の集計 |
| `projection: readonly snapshot … written only by its render function` 等（関数名が無い／書き手が複数） | 86 行（kivact-ui 70、interaction 8、engine paint 5 ほか） | 旧 `passive record - X` の機械的な書換え（付録 P5-E 4 節） |
| `projection:` が名指す関数が実在しない、または他にも書き手がいる | 確定 6 件：`ListItemState.active`、`TreeItemState.active`（P5-B D1）、`HostWindow.environment`（P5-D F-2）、`ButtonCfg.has_action`（`SidebarMenuButton::build` は無い。実際は `build_button`）、`InlineEdit.reported`（「the machine's request transition」は関数でない）、`Host.app_motion`・`ActionCell.last`・`SharedImage.current`（書き手 2 つを列挙） | 本書 3.5 節 |
| `independent:` の理由が他ファイルと同一の定型文 | 13 種（`a construction option; every combination is valid` 7 ファイル、`the observation of the attached content part …` 6 ファイル ほか） | `git grep` の集計（3.4 節 X5） |
| 偽の `independent:`（実は含意・排他がある） | 確定 9 struct：`PartState`（P5-D F-1）、`ButtonState`/`ToggleButtonState`（P5-E E-1）、`MenuItemState`（E-2）、`CardState`（E 4 節）、`BreadcrumbItemState`・`CodeBlockState`・`AlertState`・`CheckableState`・`DataTableCheckState`（P5-F）、`VariantKey`（D F-3）、paint `Step`（D F-4）、slider `Geometry`（F-T6）、node editor `Info`（C4） | 各付録 |

### 1.3 lint の誤分類（数えるべきでない／数え方が誤り）

- `Option<Arc<dyn Fn ..>>` を設定（callback）と見なさない：`classify.rs` `is_setting_payload` が `"Rc" | "Box"` だけ。`Parent.on_resize_request` の OPEN の原因（P5-A A2）。
- `signal(Overlay::None)` を `signal(None)` と見なす：`locals.rs` `is_scalar`／`is_none` が最後の segment が `None` の path を全部 None と見る。`catalog_with_mode::overlay` の OPEN の原因（P5-C C5）。

## 2 原因

1. **lint が marker の意味を検査しない**。SL-8 は接頭辞の綴りだけを見る。`projection:` の関数の実在・唯一性、`independent:` の理由が組合せを述べているかは見ない。実装者は旧 marker に接頭辞を機械的に付け、件数だけを減らした（約 470 本、P5-D 2 章）。
2. **gate と audit の条件が違う**（1.1）。OPEN が残っても通常 gate が成功するため、27 件が CP3 まで残った。
3. **旧付録 A は「型にした後の marker」まで指示していない行があった**（`EditState.log` は型は実装済みで marker が無いだけ、P5-A A4）。また付録 A が想定しなかった二重の持ち主（soft `SidebarWidth`、P5-E E-5）や、写し替えでの退行（game window の status、E-3）が生じた。
4. **snapshot struct（recipe へ渡す値）の field 間の含意**を誰も読まずに「独立」とした（1.2 の偽の independent）。

## 3 設計（確定版）

### 3.1 OPEN 25 件の閉じ方（一覧）

詳細（全 field、全書き手 file:line、組合せ、変更後コード、E2E）は付録の該当節。

| # | audit 行（HEAD） | 閉じ方 | 要点 | 付録 |
|---|---|---|---|---|
| 1 | arvane-native `native_input/state.rs:42` NativeInput.waker | 型（Option を消す） | embed も手元の `ShellWaker` を渡す。`waker: ShellWaker` | A1 |
| 2 | kivact-core `embed.rs:40` Parent.on_resize_request | lint 修正（設定は数えない） | X1：`is_setting_payload` に `Arc` を足す | A2 |
| 3 | kivact-core `view/devtools_state.rs:95` PropWrite.changed | independent | effect 由来／直接の書込みのどちらも値を変える・変えないがあり、DevTools は 4 通りを表示 | A3 |
| 4 | kivact-editing `edit_state.rs:53` EditState.log | independent | base との関係は `Option<Log>` で型済み。log と damage は別の consumer が別々に消費し 4 通り到達 | A4 |
| 5 | kivact-engine `measure/editor.rs:24` EditorNode.reported_session | projection | `written only by Model::report_sessions`（3.6 O1 で cache から変更） | A5 |
| 6 | kivact-engine `model/pointer/state.rs:107` TouchTrack.hover | 型 | `role: ContactRole { Primary(Hover), Secondary }` | A6 |
| 7 | 同 `:117` PenTrack.hover | 型 | 同上 | A6 |
| 8 | 同 `:155` Route.press | independent | capture は任意の button の listener 宛先、press は primary button の activation。(Some,None)・(None,Some)・Released 全て到達 | A8 |
| 9 | 同 `:389` PointerSource.primary | 型（field 削除） | `Track::is_primary()` で導く | A6 |
| 10 | kivact-engine `text_input/edit_lines.rs:24` EditLines.colors | 型＋cache | `coloring: Option<Coloring { colors, colored }>`（colored は colors がある時だけ） | A10 |
| 11 | kivact-interaction `list/data.rs:305` ListItemState.selected | independent | Manual activation・combobox highlight・Ctrl+矢印・Multiple 選択で 4 通り到達。`active` の偽 projection も independent に | B1 |
| 12 | 同 `scalar/edit_state.rs:147` Edit.observed | independent | bounds（構築時）× attach 前後 | B2 |
| 13 | 同 `select/machine.rs:23` State::Open.previewed | projection | `written only by track_preview`（Event::Preview の唯一の送り手）※X4 は Machine の state の書き手を関数で追えないため 3.4 X4 の除外規則を参照 | B3 |
| 14 | 同 `tree.rs:331` TreeItemState.selected | independent | B1 の写し | B4 |
| 15 | kivact-layout `style/node.rs:111` NodeStyle.grid_area | 型 | `justify_self` を `GridArea` に移す（(None,Some) が作られない含意）。`grid_area` は option | B5 |
| 16 | kivact-runtime `reactive/detail.rs:59` DetailWrite.cause | independent | 3.6 O5 の判断 | B6 |
| 17 | 同 `:80` DetailRun.cause | independent | 同上 | B7 |
| 18 | 同 `reactive/diag.rs:32` Reported.seq | identity | 二つの ring が共有する通し番号、DevTools の paging key | B8 |
| 19 | 同 `task/action.rs:255` ActionCell.next_handler | identity | `next_handler_id` に改名 | B9 |
| 20 | kivact-style `sheet/fields.rs:390` StateFields.group | option | authored な flag | B10 |
| 21 | kivact-ui `alert.rs:25` BannerState::Shared.shown | 型（SM-9 の完成） | `BannerState { Shown { restore }, Hidden }`＋`Ownership`、snapshot event 1 本、transition を純粋に。stale restore の欠陥も直る | C1 |
| 22 | kivact-ui `calendar_values.rs:34` CalendarDayVisual.mark | independent | 8 通り到達（today×in_month×mark）。既存 2 本の汎用文も書換え | C2 |
| 23 | kivact-ui `data_table/filters.rs:240` FilterDraft::Valid.value | 導出（重複の削除） | `sent` は常に `value` と同値。`sent` を消し echo は `value` で判定 | C3 |
| 24 | kivact-ui-creative `editors/node/gesture.rs:60` Info.selected | 型 | `facet: Facet { Plain, Toggle{checked}, Range{..}, Link{selected} }` | C4 |
| 25 | kivact-ui-pop-catalog `pop/shell.rs:102` catalog_with_mode::overlay | lint 修正（既に enum） | X2：`X::None` を Option と見ない。その後 `:96` の marker を削除 | C5 |

GPU の 2 件（`arvane/src/window.rs:193` Windows.gpu、`:407` Driver.gpu）は P6。

### 3.2 旧作業指示書 付録 A（1195 行以降）の照合結果

全 8 節の全行を HEAD で確認した（A-DT の data panel の tasks・theme・styles は grep による抜取り、x11 の 2 行は読んだだけでビルド未確認）。

| 節 | 結果 | 誤／未の行（修正は付録） |
|---|---|---|
| A-RT | 13 行中 済 12、文言の誤り 1 | Runtime.ext の理由が指定文言と違う（D F-6）。DetailWrite/Run.cause は本書 #16/17 |
| A-EC | 全て済 | Host の同 struct に新しい暗黙状態 `next_wake`（D F-7） |
| A-PS | focus は済、**残り 4 field の marker が偽** | PartState → `Availability`（D F-1、最重要） |
| A-AR | 全て済 | 同ファイルに新しい暗黙状態 `VariantKey`（D F-3） |
| A-LE | 全て済 | EditState.log は本書 #4 |
| A-EN | 全て済 | 同 struct に偽 projection（HostWindow.environment、D F-2）と prefix 誤用（paint Step、D F-4） |
| A-CO | 全て済、文言の誤り 1 | ViewState.initial_render の理由が値の意味と逆（D 1 章 A-CO） |
| A-IN | 済。誤 1（軽） | inline_edit の `reported` は導出できる（E-4） |
| A-UI | 済。他 slice 3（#21・#23 と calendar） | alert の SM-9 は**未実装**だった（#21 で完成） |
| A-DT | 全て済 | — |
| A-KB / A-KP | 済 | pop shell は #25 |
| A-KG | 済。**退行 1** | game window の status が Run 後に出ない（E-3） |
| A-KC | 済。誤 1（軽） | creative jobs の clock が effect のまま（E-7） |
| A-KS | 済。誤 2 | SidebarWidth の二重の持ち主（E-5）、lightbox の marker（E-6、3.6 O3） |

### 3.3 lint が見落とした暗黙状態（Slim-Fast の差分から。重要度順）

| 順 | 対象 | 種類 | 閉じ方 | 付録 |
|---|---|---|---|---|
| 1 | kivact-core `PartState`（公開、全 kit が読む） | 偽 independent（disabled/hidden の間 pressed/hovered は常に false） | 型：`availability: Availability { Enabled{pressed,hovered}, Disabled, Hidden }`＋accessor。書き手 `view/part.rs:78-88`、`kivact-ui/src/combobox/render.rs:68`、**`kivact-ui-soft/src/indicator.rs:117-120`**（P5-F 3 章） | D F-1 |
| 2 | kivact-ui `ButtonState`/`ToggleButtonState.icon_only` | 偽 projection、**実害**（toggle_group が icon の無い item を正方形に描く） | 導出：`label_hidden` を保持し `icon_only()` を導く | E-1 |
| 3 | game catalog window の `run`/`status` | **退行**（Run 後に notice が出ない） | `Run::message` を消し status を独立に | E-3 |
| 4 | kivact-ui alert の restore | 欠陥（再表示時の stale restore、effect 順依存） | #21 | C1 |
| 5 | engine `HostWindow.environment` | 偽 projection（書き手 2） | `Host::record_environment` に集約 | D F-2 |
| 6 | runtime `ActionCell.last` | SM-12 未達（書き手 2） | `ActionCell::begin_run` に集約 | B D2 |
| 7 | arvane-asset `VariantKey` | SVG 専用 size/crop と raster 専用 frame が同居 | 型：`VariantSource { Frame(u32), Svg{size,crop} }` | D F-3 |
| 8 | engine paint `Step` | `clips`/`rounded` の暗黙の 3 値、projection 誤用 | 型：`ClipShape { None, Rect, Rounded }`、残りは option | D F-4 |
| 9 | interaction `MenuItemState` | check と expanded が排他 | 型：`MenuItemDisplay { Plain, Check(MenuCheck), Submenu{open} }` | E-2 |
| 10 | kivact-ui `CardState` | `interactive ⇔ input.is_some()` | 型：`CardActionState { Plain, Press{input}, Select{input,selected} }` | E 4 節 |
| 11 | kivact-ui `BreadcrumbItemState`、`CodeBlockState`、`AlertState`、`CheckableState`、`DataTableCheckState`、slider `Geometry` | 含意・排他 | 型（各 enum） | F-T1〜T6 |
| 12 | interaction inline_edit `reported` | 常に `phase.key()` と同値 | 導出 | E-4 |
| 13 | soft `SidebarWidth` | caller の binding と二重の持ち主 | `Signal<f64>` に戻す | E-5 |
| 14 | engine `Host.next_wake` | Exit 後も古い値 | 型：`HostPhase::{Starting,Running}{next_wake}` | D F-7 |
| 15 | creative catalog jobs | clock の start/stop が effect | Machine の Effect | E-7 |
| 16 | 接頭辞の誤用・汎用文（runtime/engine/core/kit） | 文言 | 文言を確定済み | D F-5/F-6/F-8/F-9、E-6/E-10、F 2 章 |
| 17 | `ToolStatus` 等その他 | — | 済 | — |

確認の範囲（正直な限界）：P5-D は runtime・engine・core・test・arvane*・layout・editing・devtools-agent、P5-E/F は interaction・ui・kit・devtools・examples を読んだ。`projection:` は全数、`independent:` は D が 123 本中の疑わしいもの、E/F が snapshot struct 全数と重複文を読んだ。arvane-text（約 67 本の marker）、kivact-types、kivact-i18n、arvane-compose/render/shell の marker の意味は抜取りしていない。X4・X5（3.4 節）が機械的に拾える範囲（偽 projection・重複定型文）は全 crate で検出される。それ以外の意味の誤り（例：真に排他なのに個別の理由を書いた independent）は人間の audit 確認（設計書 6.4）に残る。

### 3.4 xtask（`dev/xtask/src/state_fields/`）の修正設計

lint の目的は「件数を減らすだけの marker」を通さないこと。CI 機能は足さない（既存の rule `state-fields` の条件を直すだけ）。

#### X1 `Option<Arc<dyn Fn>>` は設定（`classify.rs`）

```rust
fn is_setting_payload(ty: &syn::Type) -> bool {
    // ...
    match segment.ident.to_string().as_str() {
        "Callback" | "Handler" | "ViewFn" => true,
        "Rc" | "Arc" | "Box" => single_type_argument(segment).is_some_and(is_fn_object),
        _ => false,
    }
}
```
試験は付録 P5-A A2（`an_arc_callback_option_is_a_setting`）。

#### X2 enum の variant `X::None` は Option の None ではない（`locals.rs`）

付録 P5-C C5 のコード（`is_scalar` の Path 分岐を `is_none(expr)` に、`is_none` は `None`・`Option::None`・`std|core::option::Option::None` だけ）と試験 `enum_variants_named_none_are_not_option_locals`。直した後に `examples/kivact-ui-pop/src/pop/shell.rs:96` の marker を削除する（同じ unit、この順）。

#### X3 gate と audit の条件を一致させる（`check.rs`、`locals.rs`）

OPEN の行が 1 本でもあれば通常の gate も error にする。

`check.rs`：
```rust
/// The counted state of one struct, variant or static group.
struct Counting {
    stored: Vec<String>,   // 無印の stored（今どおり）
    phase: usize,
    /// Every counted field, marked or not (Stored, promoted Helper, Phase).
    total: usize,
    counted: Vec<Counted>,
}
```
`count_fields` の `if counted_field { ... }` の中で `total += 1;`（`audit` に関係なく数える）。`report_combination` の呼出しを
```rust
report_combination(context, item.span(), &name, &counting.stored, counting.total, errors);

fn report_combination(context: &RuleContext<'_>, span: proc_macro2::Span, name: &str,
    stored: &[String], total: usize, errors: &mut Vec<String>) {
    // A counted field without a closure beside any other counted field is an
    // implicit combination: the audit's OPEN row (design 6.4).
    if !stored.is_empty() && total >= 2 {
        context.source.error_at(RULE, span,
            &format!("{name} combines state fields: {} (close each with a type, a derivation, `projection:` or `independent:`)", stored.join(", ")),
            errors);
    }
}
```
（今の `phase > 0 && !stored.is_empty()` は `total >= 2` に含まれる。）既存の試験は期待値が変わらない（`detects_wrapped_fields_and_inline_exceptions` 等を確認済み。印付き 1＋無印 1 を「error なし」と期待する試験は無い）。

`locals.rs` `check`：
```rust
        if open.is_empty() || function.locals.len() < 2 {
            continue;
        }
```
試験を 2 本足す（`tests.rs`）：
```rust
#[test]
fn one_unmarked_field_beside_a_marked_one_is_an_error() {
    let errors = run("struct Probe {\n// xtask-allow(state-fields): independent: probe fact\na: bool,\nb: Option<Job>,\n}", "", &[]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("Probe combines state fields: b"), "{errors:?}");
}

#[test]
fn one_unmarked_local_beside_a_marked_one_is_an_error() {
    let text = "fn sample() {\n// xtask-allow(state-fields): independent: probe fact\nlet a = signal(false);\nlet b = signal(None);\n}";
    let errors = run(text, "", &[]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("function combines state locals: b"), "{errors:?}");
}
```
**順序の制約**：X3 を入れると、閉じていない OPEN 行（P6 の GPU 2 件を含む）が通常 gate の error になる。X3 は全 lane と P6 の後（FINAL-2）に入れる。

#### X4 `projection:` の書き手を検査する（新ファイル `dev/xtask/src/state_fields/claims.rs`）

規則（SL-9 として `state_fields.rs` の doc に 1 行足す）：
1. 理由は `written only by <名前>` を**ちょうど一つ**含む。`<名前>` は `fn` か `Type::fn`（`[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)?`）。
2. その名前の関数が、field の**可視範囲**の source にある（`Type::fn` は `impl Type` または `impl … for Type` の中）。
3. 可視範囲の中で、その field を書く式は全てその関数の中（その中の closure を含む）にある。
   - 可視範囲：field が private（`Visibility::Inherited`）なら owner の module とその子孫。`pub(super)` は親 module とその子孫。`pub(in path)` はその path。`pub(crate)`・`pub` は同じ package（同じ `identity_root`）の全 source。
   - 「書く式」：(a) owner の struct literal（`Owner { field: e, .. }`、enum variant の owner は path の末尾が `Variant`）の `field: e` で、`e` が空の初期値でないもの。(b) `left = e` と複合代入（`+=` 等）で、`left` の対象 field がこの field。(c) method 呼出し `recv.m(..)`、`m ∈ {set, update, replace, take, swap, insert, push, clear, get_or_init, set_if_changed}` で、`recv` の対象 field がこの field。
   - 「対象 field」：Paren・Group・Reference・`*` を剥がし、`borrow_mut`・`get_mut`・`as_mut`・`lock`・`write`・`unwrap`・`expect` の method 呼出しは receiver へ進み、最後の `ExprField` の member 名。
   - 「空の初期値」：literal、path `None`、`Default::default()`、引数 0 個の呼出し（`Sent::new()` 等）、`new`・`signal`・`var`・`default` という名前の呼出し／method 呼出しで最初の引数が空の初期値のもの（`RefCell::new(None)`、`owner.signal(None, at)`）。
   - Machine の状態の例外：owner が enum の variant で、見つかった書込みが全て `transition` という名前の関数（`impl Machine` の遷移）の中にあるときは、3 を満たしたとみなす（Machine の状態は遷移だけが書くので、marker は遷移の入口＝event の送り手を名指す）。1（書式）と 2（名指した関数の実在）は通常どおり検査する。現在この例外に当たるのは #13 `State::Open.previewed`（名指す関数 `track_preview` は `select/handle.rs` に実在）の 1 件。
4. error の文言：
   - `projection marker must name one writer: "written only by <fn>"`
   - `projection writer <名前> of <Owner>.<field> does not exist where the field is visible`
   - `<Owner>.<field> is also written in <fn> at <path>:<line>; a projection has one writer (<名前>)`

骨組み：
```rust
//! Role: Verify what state-field markers claim across the sources a field is visible in.

use crate::check_source::Source;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use syn::spanned::Spanned;
use syn::visit::Visit;

const MARKER: &str = "// xtask-allow(state-fields):";
const WRITE_METHODS: [&str; 10] = ["set", "update", "replace", "take", "swap", "insert", "push", "clear", "get_or_init", "set_if_changed"];
const THROUGH_METHODS: [&str; 7] = ["borrow_mut", "get_mut", "as_mut", "lock", "write", "unwrap", "expect"];

/// One `projection:` marker and the writer it names.
struct Claim<'a> {
    source: &'a Source,
    line: usize,
    /// The struct, or `Enum::Variant`, whose field the marker explains.
    owner: Vec<String>,
    field: String,
    visible: Visible,
    writer: Writer,
    /// The owner is an enum variant (rule 3's Machine exception may apply).
    variant: bool,
}
enum Visible { Module(String), Package }
struct Writer { ty: Option<String>, name: String }

/// Check every `projection:` claim and every `independent:` reason (X4, X5).
pub(crate) fn verify(sources: &[&Source], errors: &mut Vec<String>) {
    duplicate_reasons(sources, errors);
    for claim in claims(sources, errors) {
        verify_claim(sources, &claim, errors);
    }
}
```
- `claims`：各 source の `syn::Item::Struct`／`Item::Enum`（named variant）の各 field について `source.allow_line("state-fields", field.span())` → `allow_reason` が `projection:` で始まるものを集める（`use_allow` は呼ばない）。書式が 1 に合わなければ error を出して除く。`variant` は owner が `Item::Enum` の variant のとき true。
- `verify_claim`：可視範囲の source を選び、`Writes` visitor（関数名の stack と impl の self 型の stack を持ち、`visit_item_fn`／`visit_impl_item_fn`／`visit_trait_item_fn`／`visit_item_impl`／`visit_expr_struct`／`visit_expr_assign`／`visit_expr_binary`（`*Assign` の op）／`visit_expr_method_call` を実装）で書く式を集め、2・3 を判定する。関数の外（const・static の初期化）の書込みも「他の書き手」として error。
- `local_checks.rs` の `Scope::run` で rule が `state-fields` のとき、source の loop の後に選ばれた source（`selected_source`）の参照を集めて `crate::state_fields::verify(&selected, errors)` を 1 回呼ぶ。可視範囲の判定には同じ package の**全** source が要るので、`verify` には `self.sources` 全体と「報告する source の集合」を渡す形にする：`verify(all: &[Source], report: &dyn Fn(&Source) -> bool, errors)`。`state_fields.rs` に `mod claims;` と `pub(crate) use claims::verify;`。

試験（`tests.rs` に追加。`fixture` を 2 個並べる helper `verify_sources(&[(&str module, &str text)])` を作る）：
```rust
#[test] fn projection_needs_an_existing_single_writer() {
    // struct S { // projection: written only by publish
    //   a: Cell<bool>, b: Cell<bool> /* independent */ }
    // impl S { fn publish(&self) { self.a.set(true) } fn other(&self) { self.a.set(false) } }
    // → 1 error: "S.a is also written in other"
}
#[test] fn projection_writer_must_exist() { /* written only by missing → 1 error */ }
#[test] fn projection_allows_empty_initial_values_in_constructors() {
    // fn new() -> S { S { a: Cell::new(false), b: .. } } → error なし（literal false は空の初期値）
}
#[test] fn projection_marker_must_name_one_writer() { /* "projection: readonly snapshot" → 1 error */ }
#[test] fn private_projection_fields_are_checked_in_their_module_only() {
    // 別 module の同名 field への `.a.set(..)` は数えない
}
```

#### X5 `independent:` の理由は他ファイルと共有しない（`claims.rs` `duplicate_reasons`）

- 全 source の、`owns_line` の行で `trim_start()` が `// xtask-allow(state-fields): independent:` で始まる行を集め、理由（空白を 1 個に詰めたもの）→ path の集合にする。
- 二つ以上の path に現れる理由は、その全ての行で error：`independent: reason repeats in <other path>; say why every combination of this item's fields occurs`。
- 同じファイルの中の共有は許す（同じ struct の兄弟 field、struct とその constructor の local が同じ事実を述べる場合）。
- 試験：同じ文の 2 source → 2 error、同じ source → 0。

HEAD で X5 に当たる 13 種は全て付録で置換文を確定済み：P5-F 2 章(b)（construction option・content/trigger part・theme・test signal・RPC・Transport・dialog）、P5-C C2（measured value の calendar 側）、P5-D F-8（measured value の core 側）、本書 3.5 節の残り 3 行（`button_group.rs:180`、`calendar/state.rs:26`、`table/state.rs:36`）。

#### X6 `audit.rs` の閉じ方の判定

変更なし（X3 で OPEN 行は必ず error になるので、audit は監査用の一覧として残す）。

### 3.5 `projection:` marker の最終一覧（X4 を満たす形）と追加の確定事項

| field（HEAD） | 決定 | 新しい marker／コード |
|---|---|---|
| `kivact-core/src/gpu_interop/share/app_side.rs:33` SharedImage.current（private） | projection。書き手を一つに集める | 下記コード。marker：`// xtask-allow(state-fields): projection: the import of the link's current request, written only by SharedImage::replace_current` |
| `kivact-engine/src/host/pump.rs:81` Host.app_motion | projection。書き手 2（`events.rs:65` begin、`window_changes.rs:245` finish_motion）を集める | 下記コード。marker：`// xtask-allow(state-fields): projection: the App motion last reported to the client, written only by Host::report_app_motion` |
| `host/window.rs:50` HostWindow.environment | projection（D F-2） | `written only by Host::record_environment` |
| `host/window.rs:56,60` soft_keyboard・ime_allowed | そのまま（書き手 `synchronize_ime` だけ、`pump.rs:705,719` を確認） | 変更なし |
| `host/window.rs:62` adapter | そのまま（`pump.rs:280,282` はともに `adapter_changed` の中） | 変更なし |
| `model/aggregate.rs:42` Model.modifiers | そのまま（`keys.rs:115` は `report_modifiers` の中） | 変更なし |
| `model/pointer/state.rs:18,20` hover_sent・position_sent | そのまま（`machine.rs:31` `report_hover`、`shell.rs:95` `report_mouse_position`） | 変更なし |
| `measure/editor.rs:24` EditorNode.reported_session | projection（3.6 O1） | `// xtask-allow(state-fields): projection: the session last sent to the selection listener, written only by Model::report_sessions` |
| `kivact-runtime/src/task/action.rs:250` ActionCell.last | projection（B D2） | `written only by ActionCell::begin_run` |
| `task/async_state.rs:67` AsyncCell.latest | そのまま（`publish` だけ、`:80`） | 変更なし |
| `kivact-interaction/src/inline_edit.rs:69` InlineEdit.reported | field 削除（E-4） | marker 削除 |
| `kivact-interaction/src/select/machine.rs:23` State::Open.previewed | projection（B3、X4 の Machine 例外） | B3 の文言 |
| `kivact-ui-basic/src/sidebar/menu.rs:189` ButtonCfg.has_action | projection。関数名を実在のものに | `// xtask-allow(state-fields): projection: whether the button has an action view, written only by build_button` |
| `kivact-ui-soft/src/lightbox.rs:74` Lightbox.index | option（E-6） | E-6 の文言 |
| `kivact-ui-soft/src/lightbox.rs:78` Lightbox.view | option（3.6 O3） | `// xtask-allow(state-fields): option: absent keeps the gallery's own fit and zoom view; LightboxHandle::view passes a binding projected from its state` |
| `kivact-ui/src/disclosure_values.rs:14` DisclosureRecipeState.open | independent（3.6 O2） | `// xtask-allow(state-fields): independent: copied from the attached part's DisclosureState::open by DisclosureRecipeState::from_part; standalone panels and accordion members (group Some) both open and close` |
| snapshot struct の残り（約 80 本） | type／independent／option | 付録 P5-B・D・E・F の表 |

SharedImage（`crates/kivact-core/src/gpu_interop/share/app_side.rs`）：
```rust
    /// Point the image at `next` and return the import it replaces: the
    /// only writer of `current`. Writing `None` over `None` does nothing.
    fn replace_current(&self, next: Option<Current>) -> Option<Current> {
        let old = self.current.try_get_untracked().flatten();
        if old.is_some() || next.is_some() {
            self.current.set(next);
        }
        old
    }
```
- `renew`（`:177-187`）：`if let Some(old) = self.current.try_get_untracked().flatten() { self.current.set(None); … }` を `if let Some(old) = self.replace_current(None) { if Runtime::current().is_some() { old.owner.dispose(); } }` に（`self.changed.notify()` は今どおり後）。
- `import`（`:212-216`）：`self.current.set(Some(Current { … }));` を `self.replace_current(Some(Current { token, owner: child, image: image.clone() }));` に（戻り値は捨てる：直前の `renew` が None にしている）。
- 振る舞い不変。既存 E2E：`dev/tests/kivact-public-api/tests/e2e/` の gpu_interop 系（`gpu-interop` feature、checkpoint）。

Host::report_app_motion（`crates/kivact-engine/src/host/window_changes.rs` の `impl Host` に追加）：
```rust
    /// Record the App motion inputs as reported to the client; whether they
    /// changed (the only writer of `app_motion`).
    pub(super) fn report_app_motion(&mut self, drawable: bool, frame_period_ns: u64) -> bool {
        self.app_motion.update((drawable, frame_period_ns)).is_some()
    }
```
- `events.rs:65-66`：`self.report_app_motion(info.surface_active, info.env.frame_period_ns);`
- `window_changes.rs:245`：`if self.report_app_motion(drawable, period) {`
- 振る舞い不変。

X5 の残り 3 行（付録に無いもの）：

| 行 | 新しい marker |
|---|---|
| `crates/kivact-ui/src/button_group.rs:180`（ButtonGroupMember.keep_size） | `// xtask-allow(state-fields): option: ButtonGroupMember::keep_size; false lets the group size the member like its siblings` |
| `crates/kivact-interaction/src/calendar/state.rs:26`（State.grid_state） | `// xtask-allow(state-fields): independent: the day grid part's state, set by calendar/handle.rs when the kit attaches the grid and absent before; the calendar's other state is made at setup and is valid both before and after the grid attaches` |
| `crates/kivact-interaction/src/table/state.rs:36`（State.grid_state） | `// xtask-allow(state-fields): independent: the table grid part's state, set by table/handle.rs when the kit attaches the grid and absent before; the label and multi-sort options beside it are fixed at setup` |

### 3.6 統合での上書き（付録は反映済み）

- **O1**：P5-A A5 `EditorNode.reported_session` は `cache:` でなく `projection:`（同じ「最後に送った値」の Sent が engine では全て projection で、X4 が書き手を検査できるため統一する）。
- **O2**：P5-F の `DisclosureRecipeState.open` は `projection:` でなく `independent:`（pub field の名前 `open` は kivact-ui の他 struct の `self.open = …` と衝突し、X4 の package 範囲の検査が誤検出するため。意味も「snapshot の写し」で投影の二重保持ではない）。
- **O3**：P5-E E-6 の `Lightbox.view` は `projection:` でなく `option:`（Some は handle の binding、None は内部の view。index と同じ閉じ方に揃える）。
- **O4**：P5-E E-10 の `CodeBlockState` 行は P5-F F-T2（型）で置き換える。`code_values.rs` の所有は E-U3 から lane 5 の E-U2 系に移る（6 章）。
- **O5**：P5-D の申し送り「DetailWrite/Run.cause は CauseSlot 相当にすべき」は**採らない**。record の `cause: Option<u32>` は「この write/run を結ぶ cause の node（記録されていれば）」という一つの意味で、唯一の読み手 `kivact-core/src/devtools/updates.rs:367` `cause_edge` は None を「edge を作らない」とだけ扱う。cap で捨てた件数は `TurnDetail.dropped` が別に持つ。builder 側の二義性は `CauseSlot` で解消済み（Dropped の後の write を前の cause に誤って結ばない）。よって B6/B7 の independent で閉じる。
- **O6**：P5-E E-U6 の「残りの snapshot marker の書直し」は P5-F で全行の文言・型が確定した。E-U6 は F の表を写す unit になる。
- **O7**：P5-D F-1（PartState）は**採用**（8 章で人間に通知）。書き手に P5-F が見つけた `kivact-ui-soft/src/indicator.rs:117-120` を含める。補足（確定）：
  - disabled と hidden は両方 true になり得る（`view/part.rs` の `mount.disabled`・`mount.hidden` は別の入力）。`Availability` では **Hidden が優先**し、hidden の間 `is_disabled()` は false。PartState の `hidden`／`disabled` を「hidden の間に disabled を区別して」読む読み手は workspace に無い（`grep` で `.hidden` の PartState 読み手は 0 件）ので、読める情報は減らない。doc に「Hidden wins: a hidden target is not drawn」と書く。
  - `indicator.rs` の今のコードは、外の part が disabled になっても `input.pressed`/`hovered` を残す（不変条件「disabled の間は false」を破る無効な組合せを作っている）。P5-F 3 章の書換えで `Availability::Disabled` になり pressed/hovered は消える。見た目の変化は「disabled にした瞬間に押下の look が残らない」だけで、既存 E2E の期待値には当たらない（soft の indicator の試験で checkpoint 確認）。

### 3.7 audit コマンドの正しい使い方と OPEN 0 の確認手順

`cargo xtask check --no-write --only state-fields --report-audit` は `--no-write` があっても **`dev/xtask/state-audit.md` を書き換える**（`local_checks.rs` `write_audit`。`--no-write` は他の生成物用）。exit code は「error があるか、OPEN が 1 以上」で 1。

unit の途中（自分の crate だけ確認する）：
```bash
cargo xtask check --no-write --only state-fields --crate <package>     # audit を書かない
```
（`--report-audit` を `--crate` と併用すると、選んだ crate の行だけの audit で**全体の audit を上書きする**ので、unit では併用しない。）

checkpoint（全体、OPEN 0 の確認）：
```bash
git diff --quiet -- dev/xtask/state-audit.md || { echo "audit already modified"; exit 1; }
cargo xtask check --no-write --only state-fields --report-audit; echo "exit=$?"
grep -c '| OPEN |' dev/xtask/state-audit.md        # 0 であること（grep の exit 1 は 0 件の意味）
git diff --stat -- dev/xtask/state-audit.md          # 変化した行を人間の確認用に残す
```
- 期待：`exit=0`、`OPEN` 0 件。標準エラーに `state-audit:` の行が出ないこと。
- 生成された audit は**コミットする**（設計書 6.4：人間が読む監査の報告）。V2 の CP で audit を更新するのはこの手順だけ。調査だけで実行した場合は `git checkout -- dev/xtask/state-audit.md` で戻す。
- X3 の後は通常 gate（`cargo xtask check --no-write`）も OPEN を error にするので、OPEN 0 は通常 gate でも担保される。X4・X5 は `--only state-fields` に含まれる（新しい rule 名は作らない）。
- 加えて人間の確認項目（AC-3）：audit の `projection` 行の関数名、`independent` 行の理由を読む。

## 4 見積り（目標との対比）

| 指標 | 現在 | 本書の後 |
|---|---|---|
| audit の OPEN | 27 | 2（P6 の GPU）→ P6 の後 0 |
| 偽 projection（関数が無い・書き手複数） | 確定 9 ＋ snapshot 約 86 | 0（X4 が検出し続ける） |
| ファイルをまたぐ同一の independent 文 | 13 種 | 0（X5 が検出し続ける） |
| 型で閉じる新しい enum | — | 約 20（ContactRole、Coloring、GridArea 移動、BannerState、Facet、Availability、VariantSource、ClipShape、MenuItemDisplay、CardActionState、BreadcrumbItemState、CodeCollapse、SwitchFacts、DataTableCheckState、TrackFlow ほか） |

KPI への影響：無視できる。`NodeStyle` は `Option<AlignSelf>`（2 B）が減り grid item だけ `GridArea` が 2 B 増える。`PartState` は 4 bool＋Focus（5 B）から Focus＋Availability（3 B 程度）。それ以外は marker と private 関数。計算量の変化は無い。

AC-3 の判定：OPEN 0（3.7 の手順）、旧付録 A の全行の実装（3.2 の誤・未を本書で全て設計）、人間による audit の確認。本書の範囲外で残るのは 3.3 末尾の「抜取りしていない crate の marker の意味」。

## 5 正しさ・リスク・E2E

振る舞いが変わるもの（意図した修正）と担保：

| 変更 | 振る舞い | E2E（追加は太字） |
|---|---|---|
| #21 alert（C1） | 再表示した controlled alert の dismiss は、再表示の時点の focus に戻る | **`dev/tests/kivact-public-api/tests/e2e/feedback.rs` `controlled_alert_shown_again_restores_focus_to_the_control_focused_while_hidden`**、既存 `alert_dismiss_restores_focus`・`controlled_alert_keeps_refused_visibility_and_notifies_after_acceptance`・`apg_alert.rs` |
| E-1 toggle_group | icon の無い item は正方形にならない | **`ui_toolbar.rs` `toggle_group_icon_only_keeps_iconless_items_out_of_the_square_look`** |
| E-3 game window | Run・Pause の後も Save 等の notice が出る | **`examples/kivact-ui-game/tests/catalog.rs` `window_status_shows_the_last_command_after_run`** |
| A 既存欠陥 D1（kivact-test） | 2 本目の contact が `is_primary: false` | **`pointer.rs` `a_second_touch_is_not_primary_and_leaves_hover_and_activation_to_the_first`**、既存の単一 contact の E2E（pointer・touch_scroll・interaction_slider・presence・ui_combobox）は期待値不変 |
| D F-7 Host.next_wake | Exiting 以降に timer wake を求めない | 既存の close→drain→exit の E2E |

振る舞いが変わらないもの（型・導出・marker）で、既存の E2E に穴があり追加するもの：

| 対象 | 追加する試験 |
|---|---|
| B5 justify_self | **`grid.rs` `grid_items_justify_self_within_their_area`**（HEAD でも通ることを先に確認。通らなければ既存欠陥として報告し期待値は変えない） |
| C3 filter の echo | **`crates/kivact-ui-soft/tests/consumer/typed_table_filters.rs` `inline_filter_keeps_typed_text_through_its_own_echo_and_follows_external_values`** |
| C4 link の選択 | **`crates/kivact-ui-creative/tests/consumer/node_editor.rs` `node_link_region_reports_its_selection`** |
| X1〜X5 | xtask の unit test（3.4） |

リスク：
- 公開の recipe state の形が変わる（PartState、ButtonState、ToggleButtonState、MenuItemState、CardState、BreadcrumbItemState、CodeBlockState、AlertState、CheckableState、DataTableCheckState）。kit・example・docs の読み手は全て同じ unit で直す。互換 alias は作らない（方針 5）。
- X4 は型を見ない近似。可視範囲を field の可視性で絞り、HEAD の全 projection（3.5）で誤検出が無いことを grep で確認した（`ActionCell.last` は private なので `motion/timeline/plan.rs` の `.last` は範囲外、`DisclosureRecipeState.open` は O2 で projection をやめた）。新しい誤検出が出たら、marker を `independent:` 等に変えるか関数を集めるかを設計者に戻す（lint を弱めない）。
- 本書の範囲外の推測（P5-F 3 章）：interaction の各 Api の `content_state` は unmount で None に戻らない。checkpoint の E2E で、開いて閉じた後に recipe の state を読む経路が error を出さないことを見る。

## 6 unit 分割案（6 lane＋最後の 2 unit）

所有ファイルは lane 間で重ならない。lane の中は上から順に行う。確認コマンドの E2E は名前を指定した数件（`--` の後に filter を並べる）。全 E2E・全 audit は checkpoint。

| lane | unit | 所有ファイル | 依存 | 完了時の軽い確認 |
|---|---|---|---|---|
| L1 | XT-1（X1・X2・pop の marker 削除） | `dev/xtask/src/state_fields/classify.rs`、`locals.rs`、`tests.rs`、`examples/kivact-ui-pop/src/pop/shell.rs` | なし | `cargo test -p xtask state_fields`；`cargo clippy -p xtask --all-targets -- -D warnings`；`cargo xtask check --no-write --only state-fields --crate kivact-ui-pop-catalog` |
| L1 | LY（B5） | `crates/kivact-layout/src/style/node.rs`、`style/convert.rs`、`dev/tests/kivact-public-api/tests/e2e/grid.rs` | なし | `cargo clippy -p kivact-layout --all-targets -- -D warnings`；`cargo xtask check --no-write --only state-fields --crate kivact-layout`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- grid::` |
| L1 | ST・ED（B10・A4） | `crates/kivact-style/src/sheet/fields.rs`、`crates/kivact-editing/src/edit_state.rs` | なし | `cargo check -p kivact-style -p kivact-editing`；`cargo xtask check --no-write --only state-fields --crate kivact-style`（kivact-editing も） |
| L2 | NA（A1） | `crates/arvane-native/src/native_input/state.rs`、`port/winit/open.rs`、`port/win32/window.rs` | なし | `cargo clippy -p arvane-native --all-targets -- -D warnings`；`cargo xtask check --no-write --only state-fields --crate arvane-native` |
| L2 | AS（D F-3） | `crates/arvane-asset/src/store/entry.rs`、`store/asset_store.rs`、`store/evict.rs` | なし | `cargo check -p arvane-asset --all-targets`；`cargo test -p arvane-asset store`；state-fields `--crate arvane-asset` |
| L2 | EN-1（A5・A6・A8・A10） | `crates/kivact-engine/src/measure/editor.rs`、`model/pointer/state.rs`、`model/pointer/machine.rs`、`model/pointer/shell.rs`、`text_input/edit_lines.rs`、`text_input/shaping.rs`、`text_input/chunks.rs` | なし | `cargo clippy -p kivact-engine --all-targets -- -D warnings`；state-fields `--crate kivact-engine` |
| L2 | EN-2（D F-2・F-4・F-5 engine・F-7、3.5 app_motion） | `crates/kivact-engine/src/host/window.rs`、`host/window_changes.rs`、`host/pump.rs`、`host/launch.rs`、`host/events.rs`、`paint/state.rs`、`paint/list/walk.rs`、`contract/host.rs`、`model/lifecycle.rs` | なし | 同上＋`cargo test -p kivact-engine paint` |
| L2 | KT（A 既存欠陥 D1・D2、F-U5） | `crates/kivact-test/src/screen/model.rs`、`screen/input.rs`、`screen/disposal_tests.rs`、`screen/host_sync_tests.rs`、`screen/real_window_tests.rs`、`dev/tests/kivact-public-api/tests/e2e/pointer.rs` | EN-1 | `cargo check -p kivact-test --all-targets`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- pointer::` |
| L3 | RT（B6〜B9・B D2・D F-5 runtime・F-6・F-9・Queue の doc） | `crates/kivact-runtime/src/reactive/detail.rs`、`reactive/diag.rs`、`reactive/data.rs`、`task/action.rs` | なし | `cargo clippy -p kivact-runtime --all-targets -- -D warnings`；state-fields `--crate kivact-runtime`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- task::action_ task::queue_` |
| L3 | CO（A3・D F-5 core・F-8・initial_render・SharedImage） | `crates/kivact-core/src/view/devtools_state.rs`、`view/tree.rs`、`view/state.rs`、`gpu_interop/share/app_side.rs` | なし | `cargo check -p kivact-core --all-features`；state-fields `--crate kivact-core` |
| L4 | IN-1（B1〜B4、F の select content_state） | `crates/kivact-interaction/src/list/data.rs`、`tree.rs`、`scalar/edit_state.rs`、`select/machine.rs`、`select/handle.rs` | なし | `cargo check -p kivact-interaction`；state-fields `--crate kivact-interaction` |
| L4 | IN-2（E-1 の外、E-2・E-4・E-6 table・F menu/level） | `crates/kivact-interaction/src/menu/state.rs`、`menu/level.rs`、`menu/handle.rs`、`menu.rs`、`lib.rs`、`inline_edit.rs`、`table/data.rs`、`crates/kivact-ui-basic/src/recipes_menu.rs`、`crates/kivact-ui-soft/src/recipes_menu.rs` | なし | `cargo check -p kivact-interaction -p kivact-ui-basic -p kivact-ui-soft`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- interaction_menu apg_menu` |
| L4 | IN-3（F-U4、3.5 calendar/table の grid_state） | `crates/kivact-interaction/src/combobox/handle.rs`、`disclosure.rs`、`popup/api.rs`、`tooltip/api.rs`、`calendar/state.rs`、`table/state.rs` | なし | state-fields `--crate kivact-interaction` |
| L5 | UI-1（E-U3＋F-T3＋InputGroup：CardState・AlertState・data_table・date の feed） | `crates/kivact-ui/src/component_values.rs`、`card.rs`、`look_feedback.rs`、`data_table/types.rs`、`data_table/builder.rs`、`data_table/runtime.rs`、`date_input/view.rs`、`date_range_input.rs`、`crates/kivact-ui-soft/src/card.rs`、`crates/kivact-ui-soft/src/banner.rs`、`docs/for-kivact-user/{en,jp}/how-to-guide/feedback.md` | なし | `cargo check -p kivact-ui -p kivact-ui-soft`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- components apg_date_picker` |
| L5 | UI-2（C1・C2・C3＋F-T3 の書き手） | `crates/kivact-ui/src/alert.rs`、`calendar_values.rs`、`data_table/filters.rs`、`dev/tests/kivact-public-api/tests/e2e/feedback.rs`、`crates/kivact-ui-soft/tests/consumer/typed_table_filters.rs` | UI-1 | `cargo clippy -p kivact-ui --all-targets -- -D warnings`；`… --test e2e -- feedback:: apg_alert`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-soft --test consumer -- typed_table_filters` |
| L5 | UI-3（E-1・E-U2＋F-T2・F-T4・F-T5、Dialog 等の marker、lib.rs の export） | `crates/kivact-ui/src/values.rs`、`button.rs`、`toggle_button.rs`、`toggle_group.rs`、`look.rs`、`toggle.rs`、`radio_group.rs`、`data_table/chrome.rs`、`data_table/rows.rs`、`code_values.rs`、`code_block.rs`、`lib.rs`、`crates/kivact-ui-basic/src/recipes.rs`、`crates/kivact-ui-basic/src/code_block/view.rs`、`crates/kivact-ui-pop/src/recipes.rs`、`crates/kivact-ui-soft/src/recipes_controls.rs`、`crates/kivact-ui-soft/src/recipes_controls_choice.rs`、`crates/kivact-ui-soft/src/indicator.rs`、`crates/kivact-ui-game/src/recipes_controls.rs`、`crates/kivact-ui-creative/src/recipes.rs`、`dev/tests/kivact-public-api/tests/e2e/ui_toolbar.rs` | UI-1（`crates/kivact-ui/src/lib.rs` は L5 の共有ファイル：UI-1 が `CardActionState` の export を、UI-3 が `SwitchFacts`・`CodeCollapse` の export を足す。同じ lane の中で順に行うので衝突しない） | `cargo check -p kivact-ui -p kivact-ui-basic -p kivact-ui-pop -p kivact-ui-soft -p kivact-ui-game -p kivact-ui-creative`；`… --test e2e -- ui_toolbar interaction_toggle ui_chat_code ui_data_table` |
| L6 | UI-4（E-U6＋F 2 章の kivact-ui 分、F-T1・F-T6、button_group） | `crates/kivact-ui/src/chat_values.rs`、`collection_values.rs`、`navigation_values.rs`、`indicator_values.rs`、`tree_values.rs`、`table_values.rs`、`disclosure_values.rs`、`breadcrumb.rs`、`look_navigation.rs`、`carousel.rs`、`combobox/data.rs`、`menu.rs`、`slider.rs`、`disclosure_tokens.rs`、`progress_tokens.rs`、`button_group.rs`、`crates/kivact-ui-basic/src/breadcrumb.rs`、`crates/kivact-ui-soft/src/breadcrumbs.rs`、`docs/for-kivact-user/{en,jp}/how-to-guide/navigation.md` | なし（`data_table/types.rs:132` の marker は UI-1 が行う） | `cargo check -p kivact-ui -p kivact-ui-basic -p kivact-ui-soft`；`… --test e2e -- apg_breadcrumb apg_slider interaction_slider` |
| L6 | SO（E-5・E-6 lightbox） | `crates/kivact-ui-soft/src/side_navigation/shell.rs`、`crates/kivact-ui-soft/src/lightbox.rs` | なし | `cargo check -p kivact-ui-soft`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-soft --test consumer -- rail_navigation` |
| L6 | CR（C4） | `crates/kivact-ui-creative/src/editors/node/gesture.rs`、`node/a11y.rs`、`node/view.rs`、`crates/kivact-ui-creative/tests/consumer/node_editor.rs` | なし | `cargo clippy -p kivact-ui-creative --all-targets -- -D warnings`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-creative --test consumer -- node_editor` |
| L6 | EX（E-3・E-7・E-10・F-U7、basic sidebar） | `examples/kivact-ui-game/src/game/window.rs`、`game/controls.rs`、`examples/kivact-ui-game/tests/catalog.rs`、`examples/kivact-ui-creative/src/creative/jobs.rs`、`creative/shell.rs`、`creative/audio_values.rs`、`examples/kivact-ui-basic/src/docs/alert_dialog.rs`、`docs/dialog.rs`、`crates/kivact-ui-basic/src/sidebar/menu.rs` | なし | `cargo check -p kivact-ui-game-catalog -p kivact-ui-creative-catalog -p kivact-ui-basic-catalog`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-game-catalog --test catalog -- window_status` |
| L6 | DT（F-U6） | `crates/kivact-devtools/src/state.rs`、`session.rs` | なし | `cargo check -p kivact-devtools`；state-fields `--crate kivact-devtools` |
| — | **FINAL-1 PS（D F-1 PartState）** | `crates/kivact-core/src/view/part_data.rs`、`view/part.rs`、`view.rs`（`pub use part_data::{Availability, Focus};`）、`crates/kivact-ui/src/combobox/render.rs`、`crates/kivact-ui-soft/src/indicator.rs`、および `cargo check --workspace --all-targets` が E0609/E0560/E0063 で示す全ての読み手（kit・examples・dev/tests・docs のコード片） | L1〜L6 の全 unit（kit の読み手と衝突するため最後に一人で行う） | `cargo check --workspace --all-targets`；`cargo clippy -p kivact-core --all-targets -- -D warnings`；state-fields `--crate kivact-core` |
| — | **FINAL-2 XT-2（X3・X4・X5）** | `dev/xtask/src/state_fields/check.rs`、`locals.rs`、`claims.rs`（新）、`tests.rs`、`dev/xtask/src/state_fields.rs`、`dev/xtask/src/local_checks.rs` | FINAL-1 と **P6**（GPU 2 件が閉じていること） | `cargo test -p xtask state_fields`；`cargo clippy -p xtask --all-targets -- -D warnings`；`cargo xtask check --no-write --only state-fields`（error 0 であること。出たら本書の未設計の誤りなので STOP して報告）；3.7 の OPEN 0 手順 |

lane の重さ（変更ファイル数の目安）：L1 9、L2 29、L3 8、L4 20、L5 37、L6 37、FINAL-1 は compile error 駆動（読み手 約 70 箇所）。L2・L5・L6 が重いので、L1・L3 の担当は先に終わったら FINAL-1 の下調べ（読み手の一覧を `cargo check` の出力から作るだけ）をしてよい。crate 単位の分割の例外：kivact-ui-soft・kivact-ui-basic・kivact-ui-game・kivact-ui-creative の読み手のファイルは、型の持ち主の unit（L4・L5）が持つ（読み手だけを別 lane に置くと compile が通らない）。lane 間でファイルは重ならない。

## 7 既存欠陥（Slim-Fast 実装の欠陥、確定のみ。推測は付録に明記）

| # | file:line | 内容 | 再現 | 修正 |
|---|---|---|---|---|
| 1 | `crates/kivact-ui/src/toggle_group.rs:169` | `icon_only` を `leading_icon` と無関係に立てる | `toggle_group(..).icon_only(true)` に icon の無い item | E-1 |
| 2 | `examples/kivact-ui-game/src/game/window.rs:63-64,236-239` | Run・Pause 後に status の notice が永久に出ない（79bbd409 からの退行） | catalog で Run → Save | E-3 |
| 3 | `crates/kivact-ui/src/alert.rs:359-376,394,435-447` | 再表示時の stale restore（effect 順依存）、transition が binding を読む（旧付録 A-UI 未実装） | C1 の E2E の手順 | C1 |
| 4 | `crates/kivact-test/src/screen/input.rs:377` | `send_contact` が常に `is_primary: true`。secondary contact の経路が E2E で一度も通らない | 2 本同時の contact | A 既存欠陥 D1 |
| 5 | `crates/kivact-core/src/view/part_data.rs:62-80` | 偽 independent（公開型） | — | D F-1 |
| 6 | `crates/kivact-interaction/src/list/data.rs:306`、`tree.rs:332` | projection が名指す関数が唯一の書き手でない | — | B1・B4 |
| 7 | `crates/kivact-engine/src/host/window.rs:49` | 同上（`start_window` も書く） | — | D F-2 |
| 8 | `crates/kivact-runtime/src/task/action.rs:249` | projection の書き手が 2 つ | — | B D2 |
| 9 | `crates/kivact-ui-basic/src/sidebar/menu.rs:188` | projection が実在しない関数 `SidebarMenuButton::build` を名指す | — | 3.5 |
| 10 | `dev/xtask/src/state_fields/classify.rs`（`is_setting_payload`）、`locals.rs`（`is_scalar`/`is_none`）、`check.rs`（`report_combination`） | lint の誤分類 2 件と、gate/audit の条件の不一致 | — | X1〜X3 |
| 11 | `dev/tests/kivact-public-api/tests/e2e/pointer.rs:9-10` | doc が存在しない試験 `multi_touch` を指す | — | A 既存欠陥 D2 |
| 12 | `crates/kivact-ui-soft/src/side_navigation/shell.rs:490-532` | `SidebarWidth::Collapsed` が読まれない重複状態で binding と矛盾し得る（表示への影響なし） | 外から `collapsed.set(false)` | E-5 |
| 13 | `crates/kivact-ui/src/data_table/filters.rs:240-242` | Valid の value と sent が常に同値の重複 | — | C3 |

## 8 設計者（人間）の判断が要る点

1. **公開の recipe state の形の変更**（PartState、ButtonState/ToggleButtonState、MenuItemState、CardState、BreadcrumbItemState、CodeBlockState、AlertState、CheckableState、DataTableCheckState）。方針 4（暗黙の状態機械を全て解消）・方針 5（互換 wrapper を作らない）から本書は**採用で確定**した。外部の theme 作者の recipe は `state.input.pressed` → `state.input.is_pressed()` のように直す必要がある。読み手の書き方は「値の field」から「accessor」に寄るが、既に `is_focused()` がある形に揃う。拒否する場合の代替は無い（偽の marker は残せない）。
2. **`CheckableState` に残る排他**「checked が Indeterminate なら switch は None」（P5-F F-T4）。型にするには `checked` を control ごとの enum にし、recipe の `.checked` の読み手 54 箇所に波及する。本書は doc の不変条件で残す案（F の推奨）を採ったが、方針 4 を厳密に適用するなら `CheckControl { CheckBox(CheckedState), Switch { on, facts }, Radio(bool) }` にする。どちらにするか判断が要る。
3. **3.3 末尾の未確認範囲**（arvane-text ほかの marker の意味）を V2 で人が読むか、X4/X5 の機械的な検出で AC-3 の完了とみなすか。設計書 6.1 の 5 は「人間が audit を確認したこと」を完了条件にしている。

---

# 付録（slice ごとの詳細設計。本文 3.6 節の上書きを反映済み。各付録末尾の unit 案は本文 6 章で置き換えた）

---

## 付録 P5-A：P5 part A：OPEN 10 件（arvane-native / kivact-core / kivact-editing / kivact-engine）

対象 HEAD 73c36c7c（= db8d679b + 文書）。行番号は HEAD で確認済み。audit の行番号は「field の span の開始行」＝doc comment があればその 1 行目。lint は marker を **その開始行の直前の行**から読む（`check_source.rs:70` `allow_line` → `marker_before`）。したがって marker は「audit の行番号の直前」に 1 行挿入する。

lint の数え方（`dev/xtask/src/state_fields/classify.rs`）の要点：`Option<T>`（T が `Callback`/`Handler`/`ViewFn`/`Rc<dyn Fn>`/`Box<dyn Fn>` 以外）、`bool`、atomic、`Cell/RefCell/Signal/Var` の bool・Option・整数、名前付き counter、`pending*` 等の collection、`OnceCell` 等、phase enum（接尾辞 State/Phase/Status/Stage/Connection/Lifecycle/Run/Present）。`Sent` は named helper で、同じ struct に他の stored があると stored に昇格。audit 行は「数える field が 2 つ以上」の struct/variant だけに出る。

---

### 1. 事実と設計（1 件ずつ）

#### A1. `crates/arvane-native/src/native_input/state.rs:42` `NativeInput.waker`

**(1) field と書き手・読み手**

`NativeInput` で数えられる field：`text: RefCell<Option<ShellImeText>>`（:17, independent marker）、`reduced_motion: Cell<bool>`（:33, cache）、`locale: RefCell<Option<String>>`（:38, option）、`waker: Option<ShellWaker>`（:46, cfg(windows), **OPEN**）。

`waker` の書き手は constructor だけ（`state.rs:77` `NativeInput::new(.., waker)`、`:94`）。以後変更されない（`&self` からしか触れず Cell でもない）。
- `port/winit/open.rs:64-73`：`Some(waker.clone())`（winit の top-level window。直後 `:77` で `Subclass::install(&window, &input)`）。
- `port/win32/window.rs:360-370`：`None`（embed の Win32 子 window。subclass を入れない。ただし同関数 `:355` に `let waker = wake_handle(&shared, &native);`（`ShellWaker`）が既にある）。

読み手は `NativeInput::wake`（`state.rs:101-105`）だけ。`wake` の呼出元は subclass procedure の 2 箇所だけ：`native_input/windows.rs:232`（WM_TIMER の file drag 位置）、`:245`（WM_SETTINGCHANGE）。subclass procedure は `Subclass::install`（`windows.rs:101`）からしか登録されず、install の呼出元は `winit/open.rs:77` だけ。

**(2) 組合せ**：`waker == None` ⇔ embed の Win32 子 window ⇔ subclass が無い ⇔ `wake` が一度も呼ばれない。`None` は「読み手が存在しない」状態を表しているだけで、Option である意味が無い。(None, subclass あり) は作られない。

**(3) 閉じ方：型で表す（Option を消す）**。embed も手元にある `ShellWaker` を渡し、field を `ShellWaker` にする。`ShellWaker` は lint が数えない型なので行が消える。embed で `wake` が呼ばれても（現状は呼ばれない）`wake_handle` は embed window の WAKE message を post するだけで正しい。

**(4) 変更後のコード**

`crates/arvane-native/src/native_input/state.rs`
```rust
    /// Wakes the event loop when the subclass noted something to deliver
    /// outside a window event (a settings change, the file drag position).
    /// Only the subclass procedure calls it; an embedded child installs no
    /// subclass, so its waker is never called from here.
    #[cfg(windows)]
    waker: ShellWaker,
```
```rust
    pub(crate) fn new(environment: Environment, #[cfg(windows)] waker: ShellWaker) -> Self {
        // 本体は不変（`waker,` のまま）
    }
```
```rust
#[cfg(windows)]
impl NativeInput {
    /// Wake the event loop.
    pub(crate) fn wake(&self, reasons: WakeReasons) {
        self.waker.wake(reasons);
    }
}
```
呼出元：
- `crates/arvane-native/src/port/winit/open.rs:72-73`：`#[cfg(windows)] Some(waker.clone()),` → `#[cfg(windows)] waker.clone(),`
- `crates/arvane-native/src/port/win32/window.rs:369`：`None,` → `waker.clone(),`（`:355` の `waker` を clone。`:375` の `Session::new(shared.clone(), waker)` はそのまま move）。

変更後の audit：`NativeInput` は text / reduced_motion / locale の 3 行（それぞれ independent / cache / option）。OPEN 0。

**(5) 振る舞い**：不変（winit window は従来どおり Some、embed は wake が呼ばれない）。E2E 追加不要。確認は `cargo check -p arvane-native`（Windows）。

---

#### A2. `crates/kivact-core/src/embed.rs:40` `Parent.on_resize_request`

**(1)** `Parent` の field：`window: Option<ParentWindow>`（:37, option）、`scale: Option<f64>`（:39, option）、`on_resize_request: Option<Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>`（:40, OPEN）。
書き手：`Parent::win32`（:58 `None`）、`Parent::x11`（:67 `None`）、builder `Parent::on_resize_request`（:79 `Some(Arc::new(f))`）。読み手：`Debug`（:47）、`App::embed` の変換（:263 `on_resize_request: parent.on_resize_request` → `arvane_shell::ChildHooks`）。

**(2)** これは状態ではなく **任意の callback 設定**。`window`・`scale` とのどの組合せも有効（callback の有無は host が resize 要求を受けるかどうかだけを決める）。

**原因**：lint は `Option<Callback|Handler|ViewFn|Rc<dyn Fn>|Box<dyn Fn>>` を設定として数えない（`classify.rs` `is_setting_payload`）が、`Arc<dyn Fn ..>` を漏らしている。同じ形の field はあと 3 つ（`arvane-native/src/port/win32/state.rs:102`、`port/x11/window.rs:36`、`arvane-shell/src/child.rs:59`）あり、いずれも marker 無し（その struct の数える field が 1 つ以下なので行が出ていないだけ）。

**(3) 閉じ方：lint の誤分類を直す（field は数えない callback）**。kivact-core のコードは変えない。`dev/xtask/src/state_fields/classify.rs` の `is_setting_payload` を次のように直す：
```rust
        "Rc" | "Arc" | "Box" => single_type_argument(segment).is_some_and(is_fn_object),
```
これで `Parent.on_resize_request` は `Class::None` になり行が消える。残る行は `Parent.window`・`Parent.scale`（どちらも option）。他 3 箇所は marker が無いので「unused marker」error は生じない（確認済み：直前行に marker なし）。

xtask の試験（`dev/xtask/src/state_fields/tests.rs`）に 1 件足す：
```rust
#[test]
fn an_arc_callback_option_is_a_setting() {
    // struct S { a: Option<u8>, f: Option<std::sync::Arc<dyn Fn(u32) -> bool + Send + Sync>> }
    // → error なし、audit 行なし（数える field は a の 1 つだけ）
}
```
（tests.rs の既存の書き方＝source 文字列を `check` に通して errors / rows を見る形に合わせる。既存の `Rc<dyn Fn>` の試験があればその隣に置く。）

**(5)** 振る舞い不変。E2E 不要（既存 `dev/tests/kivact-public-api/tests/e2e/embed/contract.rs` が on_resize_request を使う）。

---

#### A3. `crates/kivact-core/src/view/devtools_state.rs:95` `PropWrite.changed`

**(1)** `PropWrite`（:90-96）：`node: NodeId`、`prop: Prop`、`effect: Option<EffectId>`（:94, statistic）、`changed: bool`（:95, OPEN）。
書き手：`PropWrite` を作るのは `view/devtools_hooks.rs:41-46`（`prop_written(id, prop, writer, changed)`）だけ。作った後は変更されない（`Vec<PropWrite>` に push、`devtools_state.rs:46`）。`prop_written` の呼出元は `view/tree.rs:658` だけで、`changed = self.engine().set(id, value)`（:652、engine が値を比較した結果）、`writer` は呼出元の引数（effect から書いたときだけ Some）。
読み手：`devtools/updates.rs:314-349` `add_prop_changes`（`wasted: !write.changed`、changed なら update_counts と changed_nodes に入れ、そうでなければ `unchanged_props` を数える。effect は cause の edge）。

**(2)** 一つの書込みの記録（不変の値）。effect 由来の書込みも直接の書込みも、値が変わる場合と同じ値の場合の両方がある（`tree.rs:633-652`：writer の有無は origins の記録だけに使い、changed は engine の比較だけで決まる）。4 通りすべて有効で、DevTools はその 4 通りを区別して表示する（effect の edge と wasted）。

**(3) 閉じ方：真に独立**。`devtools_state.rs` の :94 と :95 の間に 1 行挿入：
```rust
    // xtask-allow(state-fields): independent: whether this one write changed the node's value (the engine's comparison); a write from an effect and a direct write can each change the value or leave it equal, and DevTools shows all four
```
（挿入先：`    pub(crate) effect: Option<EffectId>,` の次、`    pub(crate) changed: bool,` の直前。）

**(4)** コード変更は marker 1 行のみ。
**(5)** 振る舞い不変。

---

#### A4. `crates/kivact-editing/src/edit_state.rs:53` `EditState.log`

**(1)** `EditState`（:46-59）の数える field：`max_length: Option<usize>`（:51, option）、`log: Option<Log>`（:53-55, OPEN）、`damage: Option<Damage>`（:56-58, independent）。`text`・`revision`・`session` は数えない。
`log` の書き手（全て edit_state.rs 内）：`new` :90（None）、`splice` :285-292（Some に追加 / 新規 `Log { base: before, .. }`）、`start_log` :387-389（None）、`take_changes` :403-410（take）、`revert` :416（take）、`take` の `Taken::Recorded` :445-451（incoming の log を追加・採用）、`Taken::Whole` :464-472（追加 / 新規）。
`damage` の書き手：`damaged` :482-493（Some）、`take_damage` :497-499（take）。`damaged` の呼出元：`set_controlled` :222、`splice` :277、`revert` :421、`take` :441, :456。
読み手：`recorded_since` :393-395、`has_changes` :398-400、`take_changes`、`revert`；damage は `EditorNode::flush_damage`（kivact-engine `measure/editor.rs:44-50`）経由の `take_damage`。

**旧付録 A（A-LE）との照合**：A-LE は「`base` を消し `log: Option<Log>`、`struct Log { base, entries }`（entries は空にしない）」「`damage` は independent の marker」を指示した。**実装済み**（`Log` :63-66、`recorded_since` が `map_or(revision, |l| l.base)`、damage の marker :56）。OPEN が残る理由は、A-LE が `log` の閉じ方を「型」としたが `Option<Log>` は lint 上 stored に数えられ、marker の指示が無かったため。

**(2)** `log` と `revision` の関係（log 無し ⇔ log は revision から始まる）は `Option<Log>` の型で閉じている。残る組合せは `log` × `damage`（× `max_length`）：
- (Some, Some)：編集直後（`splice` が両方を書く）。
- (Some, None)：編集の後に editor が行 cache 用に `take_damage` した（kivact-engine の `flush_damage`）。
- (None, Some)：`set_controlled`（:214-226）が damage を付け `start_log` で log を消す。`take_changes` の後も damage は残る。
- (None, None)：新規、または両方 take された後。
2 つは別の consumer（log は kivact の差分の受け渡し、damage は行 cache）が別々に消費する。4 通りとも到達し有効。`max_length` は設定で全組合せ有効。

**(3) 閉じ方：真に独立**（base との関係は型で閉じ済み）。:52 と :53 の間（`max_length: Option<usize>,` の次、`/// The changes recorded since the log started; ...` の直前）に挿入：
```rust
    // xtask-allow(state-fields): independent: the edits kivact has not taken yet (take_changes, start_log, revert clear it); the line-cache damage is cleared only by take_damage, and set_controlled leaves damage without a log, so all four combinations occur
```
**(4)** marker 1 行のみ。**(5)** 振る舞い不変。

---

#### A5. `crates/kivact-engine/src/measure/editor.rs:24` `EditorNode.reported_session`

**(1)** `EditorNode`（:10-30）の数える field：`lines: RefCell<Option<EditLines>>`（:15, cache）、`measure_lines`（:19, cache）、`reported_session: Sent<TextSession>`（:24-25, OPEN；`Sent` は named helper、他の stored があるので昇格）。`controlled: ControlledValue`（enum, 接尾辞なし）・`scroll_x/y` は数えない。
`reported_session` の書き手：`EditorNode::new`（:38 `Sent::new()`）と `Model::report_sessions`（`model/edit/inputs.rs:213-215` の `update(session.clone())`）だけ。読み手も同じ `update`（前回と比べる）だけ。

**(2)** 「最後に selection listener へ送った session」で、送る判定のためだけの cache。lines の有無（最初の layout 前は None）とは無関係で全組合せ有効（layout 前に selection が報告されることも、layout 後に未報告のこともある）。

**(3) 閉じ方：projection**（本文 3.6 O1。engine の他の「最後に送った値」の Sent（`Pointer.hover_sent` 等）と同じ形で、X4 が書き手を検査する）。:23 と :24 の間（`pub(crate) controlled: ControlledValue,` の次、`/// Last session sent to the selection listener.` の直前）に挿入：
```rust
    // xtask-allow(state-fields): projection: the session last sent to the selection listener, written only by Model::report_sessions
```
**(4)** marker 1 行のみ。**(5)** 振る舞い不変。既存 E2E `dev/tests/kivact-public-api/tests/e2e/custom_text_field.rs` `custom_text_field_selection_uses_graphemes_and_byte_ranges`。

---

#### A6・A7・A9. `model/pointer/state.rs:107` `TouchTrack.hover`、`:117` `PenTrack.hover`、`:389` `PointerSource.primary`

3 件は同じ事実（「その contact が primary か」）の重複なので一つの設計で閉じる。

**(1) field と書き手・読み手**

- `TouchTrack`（:102-109）：`source: PointerSource`、`over: Option<NodeId>`（:105, independent）、`at: Point`、`hover: Option<Hover>`（:107, OPEN）、`gesture: TouchGesture`。`PenTrack`（:112-119）も同形で `route: Route`。
- `PointerSource`（:375-393）：`pressure: Option<f64>`（:382, option）、`tilt: Option<(f64,f64)>`（:385, option）、`primary: bool`（:389-392, OPEN）。`time_ns`、`id`、`kind`、`buttons` は数えない。
- `primary` の書き手：`PointerSource::MOUSE`（:406 `true`）と `model/pointer/shell.rs:290-300`（contact の Down で `input.is_primary`）だけ。`source_mut()` 経由の書換え（`shell.rs:58-61`、`:347-350`、`machine.rs:652`）は time/pressure/tilt/buttons だけで **primary は track の一生で変わらない**。
- Touch/Pen の `hover` の書き手：`Track::new`（`state.rs:234`, `:241` の `source.primary.then(Hover::default)`）だけが Some/None を決める。以後は `hover_mut()`（:288-293）経由で chain の中身だけを変える（`machine.rs:105` swap_chain、`state.rs:436` forget_missing）。Some⇔None は変わらない。
- `primary` の読み手：`machine.rs:82`（sample の `is_primary`）、`:244`（track_hover：primary だけ hover chain を設定）、`:290`（pointer_down：primary だけ `primary_press`）。
- `hover` の読み手：`Track::hover`（:279-285）、`chain`（:295-297）→ `report_hover`（machine.rs:23）、`set_chain` 等。

**(2) 組合せ**：Touch/Pen では `hover.is_some() ⇔ source.primary` が常に成り立つ（同じ事実の二重保持）。(primary, hover None)・(!primary, hover Some) は作られない。Mouse では `primary` は常に true（kind=Mouse かつ primary=false は作られない無効な組合せ）。

**(3) 閉じ方：型で表す**。`primary` を `PointerSource` から消し、Touch/Pen の track に `role: ContactRole`（`Primary(Hover)` / `Secondary`）を持たせる。Mouse は常に primary（型上 `hover: Hover` を必ず持つ、今どおり）。`is_primary` は track から導く。
- `ContactRole` は接尾辞が phase でないので数えない → `TouchTrack`・`PenTrack` は数える field が `over` 1 つになり audit 行が消える（`over` の marker は数えられた field に付くので「used」のまま、unused error にならない。MouseTrack と同じ状態）。
- `PointerSource` は pressure・tilt の 2 行（option）が残る。

**(4) 変更後のコード**

`crates/kivact-engine/src/model/pointer/state.rs`
```rust
/// A contact's part in hover: the primary contact of its kind (Pointer
/// Events' `isPrimary`) keeps a hover chain while it lasts; another
/// contact of the same kind has none.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ContactRole {
    Primary(Hover),
    Secondary,
}

impl ContactRole {
    fn of(primary: bool) -> Self {
        if primary { Self::Primary(Hover::default()) } else { Self::Secondary }
    }
    fn hover(&self) -> Option<&Hover> {
        match self { Self::Primary(hover) => Some(hover), Self::Secondary => None }
    }
    fn hover_mut(&mut self) -> Option<&mut Hover> {
        match self { Self::Primary(hover) => Some(hover), Self::Secondary => None }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TouchTrack {
    pub(crate) source: PointerSource,
    // xtask-allow(state-fields): independent: the pointer's target and the hover chain are cleared separately
    pub(crate) over: Option<NodeId>,
    pub(crate) at: Point,
    pub(crate) role: ContactRole,
    pub(crate) gesture: TouchGesture,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PenTrack {
    pub(crate) source: PointerSource,
    // xtask-allow(state-fields): independent: the pointer's target and the hover chain are cleared separately
    pub(crate) over: Option<NodeId>,
    pub(crate) at: Point,
    pub(crate) role: ContactRole,
    pub(crate) route: Route,
}
```
`impl Track`：`new` を二つに分ける。
```rust
impl Track {
    /// The mouse's track (always primary, always hovering).
    pub(crate) fn mouse() -> Self {
        Self::Mouse(MouseTrack {
            source: PointerSource::MOUSE,
            over: None,
            at: Point::default(),
            hover: Hover::default(),
            gesture: MouseGesture::default(),
        })
    }

    /// The track of a touch or pen contact that just went down; `primary`
    /// is the platform's `isPrimary` at its `Down`.
    pub(crate) fn contact(source: PointerSource, primary: bool) -> Self {
        debug_assert_ne!(source.kind, PointerKind::Mouse, "the mouse is not a contact");
        let role = ContactRole::of(primary);
        if source.kind == PointerKind::Pen {
            Self::Pen(PenTrack { source, over: None, at: Point::default(), role, route: Route::default() })
        } else {
            Self::Touch(TouchTrack { source, over: None, at: Point::default(), role, gesture: TouchGesture::default() })
        }
    }

    /// Whether this is the primary pointer of its kind (Pointer Events'
    /// `isPrimary`): the mouse always; a contact when it was the only one of
    /// its kind at its `Down`, until it ends.
    pub(crate) fn is_primary(&self) -> bool {
        match self {
            Self::Mouse(_) => true,
            Self::Touch(t) => matches!(t.role, ContactRole::Primary(_)),
            Self::Pen(t) => matches!(t.role, ContactRole::Primary(_)),
        }
    }

    pub(crate) fn hover(&self) -> Option<&Hover> {
        match self {
            Self::Mouse(t) => Some(&t.hover),
            Self::Touch(t) => t.role.hover(),
            Self::Pen(t) => t.role.hover(),
        }
    }

    pub(super) fn hover_mut(&mut self) -> Option<&mut Hover> {
        match self {
            Self::Mouse(t) => Some(&mut t.hover),
            Self::Touch(t) => t.role.hover_mut(),
            Self::Pen(t) => t.role.hover_mut(),
        }
    }
    // 他のメソッドは不変
}
```
`PointerSource`：`primary` field（:388-392 の doc 3 行と field）を削除。`PointerSource::MOUSE` の `primary: true,`（:406）を削除。
`PointerState::default`（:71-75）：`Track::new(PointerSource::MOUSE)` → `Track::mouse()`。

`crates/kivact-engine/src/model/pointer/machine.rs`
- `pointer_sample`（:51-86）：
  ```rust
  let track = self.pointer.state.track(pointer);
  let source = *track.source();
  let is_primary = track.is_primary();
  ...
  is_primary,
  ```
- `begin_pointer`（:88-93）：
  ```rust
  /// Follow a new contact, `source`, at the window position `at`; `primary`
  /// is the platform's `isPrimary` at its `Down`.
  pub(super) fn begin_pointer(&mut self, source: PointerSource, primary: bool, at: Point) {
      let mut track = Track::contact(source, primary);
      track.set_at(at);
      self.pointer.state.tracks.insert(source.id.0, track);
  }
  ```
- `:244`：`if self.pointer.state.track(pointer).source().primary {` → `if self.pointer.state.track(pointer).is_primary() {`
- `:290`：同上。

`crates/kivact-engine/src/model/pointer/shell.rs:290-302`
```rust
                self.begin_pointer(
                    PointerSource {
                        time_ns: input.time_ns,
                        id,
                        kind: input.kind,
                        pressure: input.pressure,
                        tilt: input.tilt,
                        buttons: input.buttons,
                    },
                    input.is_primary,
                    at,
                );
```
（`let primary = input.is_primary;` の行は削除。）

`use` の追加は不要（`ContactRole` は state.rs 内のみで使う。`touch.rs:8` の `TouchTrack` の pattern は全て `..` を使っているので変更不要：`touch.rs:201-208`）。

**(5) 振る舞い**：不変（primary は Down の値のまま、hover の有無も同じ）。既存 E2E：`dev/tests/kivact-public-api/tests/e2e/pointer.rs` `a_touch_tap_activates_a_button_and_leaves_no_hover`、`touch_scroll.rs` 全件、`interaction_slider.rs:185`、`presence.rs:171`、`ui_combobox.rs:62`（いずれも primary の単一 contact）。

**secondary contact の経路は E2E が無い**（kivact-test が常に `is_primary: true` を送るため。下の既存欠陥 D1）。D1 を直したうえで追加する E2E（`dev/tests/kivact-public-api/tests/e2e/pointer.rs` 末尾）：
```rust
/// A second finger down while the first is still down is not primary: it
/// neither hovers, shows pressed nor activates; the first finger does.
/// A finger that goes down after both lifted is primary again.
#[test]
fn a_second_touch_is_not_primary_and_leaves_hover_and_activation_to_the_first() {
    let log: Log = Rc::default();
    let (pa, pb) = (Rc::new(RefCell::new(0)), Rc::new(RefCell::new(0)));
    let (out_a, out_b, ca, cb) = (log.clone(), log.clone(), pa.clone(), pb.clone());
    let button = |id: &'static str, count: Rc<RefCell<i32>>, out: Log| {
        node().role(Role::Button).name(id).test_id(id).focusable(true)
            .on_press(move || *count.borrow_mut() += 1)
            .on_pointer(move |e: PointerEvent| {
                out.borrow_mut().push(format!("{id} {:?} primary={}", e.phase(), e.is_primary()));
                Handled::No
            })
            .style(style().size(60, 40).background(rgb(0x101010))
                .pressed(style().background(rgb(0xc00000))))
    };
    let screen = render(move || {
        column((button("a", ca, out_a), button("b", cb, out_b)))
            .style(style().align_items(Align::Start))
    });
    let (ax, ay) = center(&screen, "a");
    let (bx, by) = center(&screen, "b");
    let first = screen.contact_down(PointerKind::Touch, ax, ay);
    let second = screen.contact_down(PointerKind::Touch, bx, by);
    let color = |x: f64, y: f64| screen.screenshot().pixel(x as u32, y as u32);
    assert_eq!(color(ax, ay), Some(0xc00000), "the first finger presses a");
    assert_eq!(color(bx, by), Some(0x101010), "the second finger does not show pressed");
    assert_eq!(
        screen.hovered().and_then(|h| h.rect()),
        screen.get_by_test_id("a").rect(),
        "only the primary finger hovers"
    );
    screen.contact_up(second);
    assert_eq!(*pb.borrow(), 0, "a non-primary finger does not activate");
    screen.contact_up(first);
    assert_eq!(*pa.borrow(), 1, "the primary finger activates");
    let entries = std::mem::take(&mut *log.borrow_mut());
    assert!(entries.iter().any(|e| e.starts_with("a Down") && e.ends_with("primary=true")), "{entries:?}");
    assert!(entries.iter().any(|e| e.starts_with("b Down") && e.ends_with("primary=false")), "{entries:?}");
    let again = screen.contact_down(PointerKind::Touch, bx, by);
    assert_eq!(color(bx, by), Some(0xc00000), "after both lifted, a new finger is primary");
    screen.contact_up(again);
    assert_eq!(*pb.borrow(), 1);
}
```
（`Locator` は `PartialEq` を持たないので、hover の node は `rect()`（`crates/kivact-test/src/screen/input.rs:798`）で比較する。a と b は縦に並ぶので rect は異なる。）
また pointer.rs 冒頭 doc（:9-10）の「several pointers at once: `multi_touch`」を「several pointers at once: `a_second_touch_is_not_primary_and_leaves_hover_and_activation_to_the_first`」に直す（存在しない試験名を指している）。

---

#### A8. `model/pointer/state.rs:155` `Route.press`

**(1)** `Route`（:151-156）：`capture: Option<Capture>`（:154, independent）、`press: Option<Press>`（:155, OPEN）。
`press` の書き手：`machine.rs:343-357`（`primary_press`、primary pointer の **primary button** の Down のときだけ Some）、`machine.rs:475-480`（`pointer_up` で primary button の Up のとき take）、`state.rs:450-455`（forget_missing：chain と target の中身だけ）、`state.rs:362-370` `reset_gesture`（Route ごと default）。
`capture` の書き手：`machine.rs:296-299`（任意の button の Down で listener があれば Implicit）、`:489-492`（全 button が離れたら None）、`:563-573`（`set_pointer_capture`：Explicit / Released）、`state.rs:443-449`（target が消えたら None）、`reset_gesture`。
読み手：`Track::capture`/`pressed`（:349-360）、`pointer_up`（:460-466 Released の判定）、swap_chain（:111）。

**(2) 組合せ（全て到達）**：
- (None, None)：何も押していない。
- (Some, Some)：primary button の Down が listener の上。
- (None, Some)：primary press が pointer listener の無い node（`activatable` だけの node）の上、または press 中に capture の target が消えた（forget_missing）。
- (Some, None)：secondary button（右・中）の Down が listener の上（press は primary button だけ）、secondary contact の Down、primary button の Up の後に他の button が残る。
- (Released, Some)：listener が press 中に capture を解放（以後 hit-test、Up は hit の listener へ）。
capture は「listener event の宛先」（任意の button）、press は「primary button の activation の記録」で、決める event が異なる。

**(3) 閉じ方：真に独立**（両 field に具体的な理由を書く。capture の既存 marker は「independent lifetimes」で理由として弱いので書き換える）。
`state.rs:153` を置換：
```rust
    // xtask-allow(state-fields): independent: where the gesture's listener events go: set by any button's Down over a listener or by set_pointer_capture, cleared when no button is held; a secondary-button drag has it without a press
```
:154 と :155 の間に挿入：
```rust
    // xtask-allow(state-fields): independent: the primary button's pressed chain and activation target, set only by primary_press and taken by that button's Up; a press over a node without a pointer listener has no capture, and a listener may release the capture mid-press
```
また :150 の doc `/// Capture and primary press are independent facts of a routed gesture.` はそのまま。

**(4)** marker のみ。**(5)** 振る舞い不変。既存 E2E：`pointer.rs` `captured_pointer_keeps_moving_events_outside_the_node`、`release_capture_returns_to_hit_testing`、`enter_leave_click_counts_and_the_release_outside_the_window`。

---

#### A10. `crates/kivact-engine/src/text_input/edit_lines.rs:24` `EditLines.colors`

**(1)** `EditLines`（:9-28）の数える field：`damage: Option<Damage>`（:12, cache）、`colors: Option<LineColors>`（:24-25, OPEN）。`colored: usize`（:26）は lint が数えない（名前が counter 規則に該当しない）が、`colors` と組の状態。
`colors` の書き手：`new` :72（None）、`shaping.rs:320-323` `color`（違う colorizer なら Some に置換）、`shaping.rs:406-410` `uncolor`（take）。
`colored` の書き手：`new` :73（0）、`shaping.rs:352`（color の中で +1）、`:402`（color_wrapped_line）、`:417` `forget_colors`（0）、`chunks.rs:66`（rebuild で 0）、`chunks.rs:207`（`min(k)`）。
読み手：`color`（:320, :324-325）、`uncolor`。外部からの読み手は無い（`input_paint.rs` は line ごとの `kept.colors` を読む。別物）。

**(2) 組合せ**：`colors == None` のとき `colored == 0`（`colored` を増やすのは `color` だけで、その先頭で colors を Some にする。`uncolor` は colors を消すとき `forget_colors` で 0 にする）。(None, n>0) は作られない＝ **colored は colors が Some のときだけ意味を持つ（SM-2）**。`damage` とは独立（damage は次の shaping が適用する編集範囲、colors は行の色付けの cache の鍵。編集の有無と色付けの有無は全組合せ起こる）。

**(3) 閉じ方：型で表す（colored を colors の中へ）＋ 残る Option は cache**。
`coloring: Option<Coloring>`、`struct Coloring { colors: LineColors, colored: usize }`。(None, n>0) が型で消える。`EditLines` の数える field は damage と coloring の 2 つで、coloring に cache marker（damage と全組合せ有効）。

**(4) 変更後のコード**

`crates/kivact-engine/src/text_input/edit_lines.rs`（:24-26 を置換、struct を追加）
```rust
    // xtask-allow(state-fields): cache: the colorizer the kept line colors were made with and how many leading lines hold them; any pending damage can stand beside it
    /// The colors the lines were colored with, and how far.
    pub(crate) coloring: Option<Coloring>,
```
```rust
/// The colorizer of a text input's lines and how far it has colored them.
#[derive(Clone, Debug)]
pub(crate) struct Coloring {
    pub(crate) colors: LineColors,
    /// Lines `0..colored` hold colors made with `colors`.
    pub(crate) colored: usize,
}

impl EditLines {
    /// How many leading lines hold colors (0 without a colorizer).
    pub(crate) fn colored(&self) -> usize {
        self.coloring.as_ref().map_or(0, |c| c.colored)
    }

    /// Note that lines `0..n` hold colors; nothing without a colorizer.
    pub(crate) fn set_colored(&mut self, n: usize) {
        if let Some(c) = &mut self.coloring {
            c.colored = n;
        }
    }
}
```
`new`（:71-73）：`colors: None, colored: 0,` → `coloring: None,`

`crates/kivact-engine/src/text_input/shaping.rs`
```rust
    pub(crate) fn color(&mut self, text: &str, colors: &LineColors, upto: usize) {
        if self.coloring.as_ref().is_none_or(|c| c.colors != *colors) {
            self.forget_colors();
            self.coloring = Some(Coloring { colors: colors.clone(), colored: 0 });
        }
        while self.colored() < upto.min(self.len()) {
            let i = self.colored();
            // ...（:326-351 は不変）
            self.set_colored(i + 1);
        }
    }
```
（`:352 self.colored += 1;` → `self.set_colored(i + 1);`。`LineColors: PartialEq` は既存の `!= Some(colors)` 比較で成り立っている。）
- `color_wrapped_line` :402：`self.colored = last + 1;` → `self.set_colored(last + 1);`
- `uncolor`（:406-410）：
  ```rust
  pub(crate) fn uncolor(&mut self) {
      if self.coloring.take().is_some() {
          self.forget_colors();
      }
  }
  ```
- `forget_colors`（:412-419）：`self.colored = 0;` → `self.set_colored(0);`
- `use` に `crate::text_input::edit_lines::Coloring` を追加（shaping.rs の既存 use の並びに合わせる）。

`crates/kivact-engine/src/text_input/chunks.rs`
- :66：`self.colored = 0;` → `self.set_colored(0);`
- :207：`self.colored = self.colored.min(k);` → `let colored = self.colored().min(k); self.set_colored(colored);`

旧 `color` は「colors を Some にしてから forget（colored=0）」、新は「forget してから Some{colored:0}」で同値（forget_colors は line の色を消し colored を 0 にするだけで colors を読まない）。

**(5)** 振る舞い不変。既存 E2E：`dev/tests/kivact-public-api/tests/e2e/custom_text_field.rs` `custom_text_field_draws_line_numbers_syntax_and_an_inline_control`（syntax の色を画素で確認）。

---

### 2. 変更後の audit（この 10 件の範囲）

| 行 | 変更後 |
|---|---|
| NativeInput.waker | 行が消える（field は数えない型） |
| Parent.on_resize_request | 行が消える（xtask の Arc 修正後） |
| PropWrite.changed | independent |
| EditState.log | independent |
| EditorNode.reported_session | projection（本文 3.6 O1） |
| TouchTrack.hover / PenTrack.hover | 行が消える（`role: ContactRole`、struct の数える field が over の 1 つ） |
| Route.press | independent（capture の marker も文言更新） |
| PointerSource.primary | 行が消える（field 削除。pressure・tilt の option 2 行は残る） |
| EditLines.colors | `EditLines.coloring` = cache |

新しい OPEN 行は生じない（`ContactRole`・`Coloring` は phase 接尾辞でなく、bool/Option を新設しない。`Coloring` 内の `colored: usize` は数えない）。

### 3. 既存欠陥

- **D1（事実）**：`crates/kivact-test/src/screen/input.rs:377` `send_contact` が常に `is_primary: true` を送る。`contact_down` の doc（同ファイル :240-248 の例）は 2 本同時の contact を示しており、実 platform では 2 本目は `isPrimary=false`。結果、試験では 2 本目の指も hover chain を持ち `primary_press`（focus・pressed・activation）を行い、engine の secondary 経路（`state.rs` の hover None、`machine.rs:244/290` の分岐）が E2E で一度も通らない。
  修正：`crates/kivact-test/src/screen/model.rs:50-56` `Contact` に `pub(super) primary: bool` を足す。`input.rs` `contact_down`（:252-270）で push の前に
  ```rust
  let primary = !self.app.contacts.borrow().iter()
      .any(|(c, _)| c.window == self.window && c.kind == kind);
  let contact = Contact { id, window: self.window, kind, primary };
  ```
  `send_contact`（:377）を `is_primary: contact.primary,` にする。`Contact` を作る箇所は `input.rs:259` だけ（確認済み）。既存の contact を使う E2E はすべて 1 本ずつなので期待値は変わらない（checkpoint で pointer / touch_scroll / interaction_slider / presence / ui_combobox を実行）。
- **D2（事実）**：`dev/tests/kivact-public-api/tests/e2e/pointer.rs:9-10` の doc が存在しない試験 `multi_touch` を指す。A6 の新 E2E 名に直す。
- **D3（事実、lint）**：`classify.rs` `is_setting_payload` が `Arc<dyn Fn>` を設定扱いしない（A2）。

### 4. unit 分割案（この 10 件）

| unit | crate / 所有ファイル | 依存 | 軽い確認 |
|---|---|---|---|
| P5-A-xtask | `dev/xtask/src/state_fields/classify.rs`、`dev/xtask/src/state_fields/tests.rs` | なし（最初に入れる） | `cargo test -p xtask state_fields`、`cargo clippy -p xtask --all-targets -- -D warnings` |
| P5-A-native | `crates/arvane-native/src/native_input/state.rs`、`port/winit/open.rs`、`port/win32/window.rs` | なし | `cargo check -p arvane-native`、`cargo clippy -p arvane-native --all-targets -- -D warnings`、`cargo xtask check --no-write --only state-fields --crate arvane-native` |
| P5-A-core | `crates/kivact-core/src/view/devtools_state.rs` | なし | `cargo check -p kivact-core --features devtools`、`cargo xtask check --no-write --only state-fields --crate kivact-core` |
| P5-A-editing | `crates/kivact-editing/src/edit_state.rs` | なし | `cargo check -p kivact-editing`、`cargo xtask check --no-write --only state-fields --crate kivact-editing` |
| P5-A-engine | `crates/kivact-engine/src/measure/editor.rs`、`model/pointer/state.rs`、`model/pointer/machine.rs`、`model/pointer/shell.rs`、`text_input/edit_lines.rs`、`text_input/shaping.rs`、`text_input/chunks.rs` | なし | `cargo check -p kivact-engine`、`cargo clippy -p kivact-engine --all-targets -- -D warnings`、`cargo xtask check --no-write --only state-fields --crate kivact-engine` |
| P5-A-test | `crates/kivact-test/src/screen/model.rs`、`crates/kivact-test/src/screen/input.rs`、`dev/tests/kivact-public-api/tests/e2e/pointer.rs` | P5-A-engine | `cargo check -p kivact-test`、`dev/tools/window-lock.sh --timeout 600 -- cargo test -p kivact-public-api-tests --test e2e pointer::`（test target 名 `e2e` は Cargo.toml 確認済み）（名前指定の数件のみ。全 E2E は checkpoint） |

kivact-core と kivact-editing の unit は marker 1 行ずつなので、他の P5 の同 crate の unit に吸収してよい（所有ファイルが重ならないこと）。

---

## 付録 P5-B：P5 slice B：OPEN 10 件の閉じ方（kivact-interaction / kivact-layout / kivact-runtime / kivact-style）

前提：HEAD 73c36c7c（= db8d679b + 文書）。行番号は HEAD で確認済み。lint の marker は「field の span の開始行（doc comment があればその `///` 行）の直前」に置く（`dev/xtask/src/check_source.rs:47-65` `marker_before` は上に向かって `#[..]`・`///`・他の `// xtask-allow(` 行だけを飛ばす）。したがって doc comment 付きの field では **doc comment の上**に marker を置く。

audit の行番号は span の開始行（doc 行）。下表の「field 行」はコードの宣言行。

| # | audit 行 | field | 閉じ方（確定） | 振る舞い |
|---|---|---|---|---|
| B1 | list/data.rs:305 | ListItemState.selected | 真に独立（+ `active` の誤った projection marker を independent に書き換え） | 不変 |
| B2 | scalar/edit_state.rs:147 | Edit.observed | 真に独立 | 不変 |
| B3 | select/machine.rs:23 | State::Open.previewed | 単一の書き手の投影（`track_preview`） | 不変 |
| B4 | tree.rs:331 | TreeItemState.selected | 真に独立（+ `active` の marker 書き換え） | 不変 |
| B5 | style/node.rs:111 | NodeStyle.grid_area | **型で表す**：`justify_self` を `GridArea` に移し、`grid_area` は option | 不変（E2E 追加） |
| B6 | reactive/detail.rs:59 | DetailWrite.cause | 真に独立 | 不変 |
| B7 | reactive/detail.rs:80 | DetailRun.cause | 真に独立 | 不変 |
| B8 | reactive/diag.rs:32 | Reported.seq | identity | 不変 |
| B9 | task/action.rs:255 | ActionCell.next_handler | identity（`next_handler_id` に改名） | 不変 |
| B10 | sheet/fields.rs:390 | StateFields.group | option（authored な flag） | 不変 |

---

### 1 事実（field・書き手・読み手）

#### B1 `ListItemState`（crates/kivact-interaction/src/list/data.rs:301-308、公開・recipe の入力）

```rust
pub struct ListItemState {
    pub input: PartState,      // struct。lint は数えない
    pub selected: bool,        // :305 OPEN
    // xtask-allow(state-fields): projection: readonly snapshot of the row for recipes, written only by ListApi::item_state
    pub active: bool,          // :307 projection
}
```

`Copy` の値（snapshot）。構築箇所（＝書き手）は 4 つ。**marker の「written only by ListApi::item_state」は事実と違う**（既存欠陥 D1）。

| 書き手 | selected の出所 | active の出所 |
|---|---|---|
| `list/handle.rs:68` `ListApi::item_state`（未 mount の fallback） | `state.selected`（選択集合）に key が含まれるか | `state.active == Some(key)` |
| `list/parts.rs:91` `parts::item`（mount された行） | `selected(state.selection).contains(key)` の memo（:47-50） | `bound == Admitted && state.active == Some(key)`（:51-57） |
| `combobox/handle.rs:344` `ComboboxApi::item_state` fallback | `Selection::Single/Multiple` の値 | `self.active == Some(key)`（highlight） |
| `combobox/handle.rs:680` `ComboboxApi::item` | `item_selected(key)` | `is_active` memo |

select・menubar・palette は `ListApi`/`ComboboxApi` に委譲（`select/handle.rs:267,652-662`、`menubar.rs:140,197`、`palette/handle.rs:364,486`）。

読み手：kit の recipe（`kivact-ui/src/combobox/look.rs:56` `state.active`、`kivact-ui-basic/src/list_popup.rs:24,36` `state.active`、他は `selected`／未使用）。全一覧は Grep `ListItemState`（kivact-ui, -basic, -pop, -game, -creative, -soft）。今回は型を変えないので kit の修正は無い。

#### B2 `Edit`（crates/kivact-interaction/src/scalar/edit_state.rs:144-157、crate 内部）

数える field：`observed: Signal<Option<Value<ScalarState>>>`（:147、OPEN）、`bounds: Option<(Value<f64>, Value<f64>)>`（:156、option marker 済み）。`thumb_rect: Value<..>` は `Value` なので数えない。

- `observed` の書き手：`edit_state.rs:169`（`Edit::new` で `None`）、`scalar/handle.rs:386`（`ScalarApi::part` が kit の `input()`/`thumb()`（handle.rs:274,285）で part を作った時に `Some(part_state)`）。None に戻す書き手は無い。
- 読み手：`scalar/handle.rs:107-120` `control_state`（`None` なら論理値から作る fallback）。`range_scalar.rs:171` `thumb_state` 経由。
- `bounds` の書き手：構築時のみ。`scalar/handle.rs:97`（`None`、単一 slider）、`range_scalar.rs:91`（`Some(bounds)`、範囲 slider の各 thumb）。読み手：`handle.rs:245,395`、`track_data.rs:29`。
- 同じ形の先例：`toggle.rs:32-33` `ButtonApi.observed`（`independent: the observation of the attached part, absent until it attaches`）。

#### B3 `select::machine::State::Open`（crates/kivact-interaction/src/select/machine.rs:18-28、crate 内部）

数える field：`highlight: Option<K>`（independent 済み）、`hovered: Option<K>`（independent 済み）、`previewed: Option<K>`（:23 doc / :24 宣言、OPEN）。`typeahead: Typeahead`・`surface`・`opened_by` は phase 名の enum ではないので数えない。

`previewed` の書き手は `Machine::transition` だけ：
- `Event::Open`・`Event::OpenSheet`（:66-82）で `None`。
- `Event::Preview(key)`（:107-118）：Open の時だけ書き、値が変わった時だけ `Effect::Preview(key)`。
- `Event::Close`（:85-96）：状態ごと Closed に置き換え、`previewed.is_some()` なら `Effect::Preview(None)`。

`Event::Preview` を送るのは `select/handle.rs:791-801` `track_preview` だけ（`hovered.get().or_else(|| list.active().get())` を effect で送る。呼出し `handle.rs:625`、mount された surface の owner の下）。Effect の消費は `handle.rs:175-179`（caller の preview callback）。

旧作業指示書 A-IN 行「`select/handle.rs` `track_preview` → `State::Open { previewed }` と `Effect::Preview`」は**実装済み**（`reported: Var<Sent>`・on_cleanup は消えている）。

#### B4 `TreeItemState`（crates/kivact-interaction/src/tree.rs:327-336、公開）

```rust
pub struct TreeItemState {
    pub input: PartState,
    pub selected: bool,   // :331 OPEN
    // xtask-allow(state-fields): projection: readonly snapshot of the row for recipes, written only by TreeApi::item
    pub active: bool,     // :333 projection
    pub branch: TreeBranch,
    pub level: u32,
}
```

書き手：`tree.rs:525` `TreeApi::item_state`（`list.item_state(key)` の写し）と `tree.rs:704` `TreeApi::item`（`list.item(key).map_state(..)` の写し）。`selected`・`active` は B1 の `ListItemState` をそのまま写す。**marker の「written only by TreeApi::item」は事実と違う**（D1）。tree の `ListOptions`（`tree.rs:489-496`）は既定の Automatic activation、選択は caller の Single/Multiple binding、修飾 gesture（`tree.rs:545`）あり。読み手：`kivact-ui/src/tree.rs:84,122,256,301`、`kivact-ui/src/tree_values.rs:5,17`、`kivact-ui-creative/src/outliner/view.rs:243,614`（型を変えないので修正なし）。

#### B5 `NodeStyle`（crates/kivact-layout/src/style/node.rs:79-115、crate 内部）

`grid_area: Option<Box<GridArea>>`（:111 doc / :112 宣言、OPEN）、`justify_self: Option<AlignSelf>`（:113 marker / :114 宣言、option）。

- 書き手は `style/convert.rs:112-195` `taffy_style` だけ（`Default` の `node.rs:152-153` を除く）：`let item = l.grid.as_deref();`（:114）、`grid_area: item.map(|g| Box::new(GridArea { column: g.column, row: g.row }))`（:182-187）、`justify_self: item.and_then(|g| g.justify_self).map(align)`（:188）。
- 読み手：`node.rs:342-357` `GridItemStyle::grid_row/grid_column`、`node.rs:365-368` `justify_self()`。
- 出所 `kivact_types::GridProps`（`kivact-types/src/layout.rs:107-119`）は `container`・`column`・`row`・`justify_self` を一つにまとめた item の grid 設定。
- **隠れた含意**：`justify_self.is_some()` ⇒ `item.is_some()` ⇒ `grid_area.is_some()`。(grid_area = None, justify_self = Some) は作られない。CSS でも `justify-self` は grid item の配置の一部。

旧 A-LE 行の `NodeDisplay { None, Flex, Grid(Option<Box<TStyle>>) }` は**実装済み**（node.rs:30-68）。

#### B6・B7 `DetailWrite`・`DetailRun`（crates/kivact-runtime/src/reactive/detail.rs:55-90、`#[doc(hidden)]` 公開、core の DevTools 用）

`cause: Option<u32>`（DetailWrite :59 doc / :60、DetailRun :80 doc / :81、OPEN）、`changed: bool`（independent 済み）。

- 書き手：`DetailLog::write`（detail.rs:220-242、`cause: builder.cause.at()` :233）、`DetailLog::run`（:245-265、:257）。一度 push した後は変更されない record。
- `builder.cause`（`CauseSlot`、:136-158）の書き手：`DetailLog::cause`（:205-216）、`end_turn`（:317 で Empty）、`start`（:174）。旧 A-RT 行の `CauseSlot { Empty, At(u32), Dropped }` は**実装済み**。
- 読み手：`kivact-core/src/devtools/updates.rs:282,310` → `cause_edge(cause, id)`（:367、None なら edge を作らない）。

#### B8 `Reported`（crates/kivact-runtime/src/reactive/diag.rs:30-39、公開、DevTools 用）

`seq: u64`（:32 doc / :33、名前 `seq` の named counter として数える、OPEN）、`handled: bool`（independent 済み）。

- 書き手：`Runtime::reported`（diag.rs:279-294）だけ。`seq` は ring の要素 `(seq, error)` の写しで、その番号は `Diag::record_error`（:77-82）等が `next_seq` から割り当てる（errors と diagnostics で一つの counter）。
- 読み手：`kivact-core/src/devtools/console.rs:13,30,46,52`（`since` より大きいものを返す paging の key）、`kivact-core/src/devtools/input.rs:206`、`kivact-devtools/src/panels/console.rs:48`（virtual list の key）、`diag.rs:292`（sort）。

#### B9 `ActionCell`（crates/kivact-runtime/src/task/action.rs:242-257、crate 内部）

数える field：`last: RefCell<Option<Submitted<I>>>`（:250、projection 済み）、`next_handler: Cell<Generation>`（:255、`Generation` は named helper。同じ struct に stored の `last` があるので stored に昇格、OPEN）。`drained`・`on_complete` は `RefCell<Vec<..>>` で数えない。

- `next_handler` の書き手：`action.rs:385`（構築、`Generation::new()`）、`Action::on_complete`（:753-781）の :760-762（`get` → `next()` → `set`）だけ。値は `DoneHandler.id`（:68-74）に入り、登録 owner の cleanup（:774-779）が `retain(|h| h.id != id)` で照合する。それ以外の読み手は無い。
- 旧 A-RT 行 `ActionCell.last`（projection、型は変えない）は**marker の文言どおり実装済み**。ただし SM-12 の「一つの関数」になっていない（既存欠陥 D2）。
- 旧 A-RT 行 `Queue.worker_ids → next_worker_id` は改名済み（action.rs:112）。identity marker は無いが、`Queue` には他に stored が無く（`phase: RefCell<ActionPhase>` は phase）helper が昇格しないため lint の対象外。marker を足すと「unused marker」になるので、付けないのが正しい。

#### B10 `StateFields`（crates/kivact-style/src/sheet/fields.rs:383-409、crate 内部）

`group: bool`（:390、OPEN）。他の 11 field は全て `Option<Box<Style>>` で `option:` marker 済み。

- 書き手：`sheet/setters.rs:673-676` `Style::group()`（`true` にするだけ。false に戻す setter は無い）、`fields.rs:413` `merge`（`self.group |= o.group`）。
- 読み手：`resolve/compile.rs:310`（state style があるかの判定）、`:345`（`StateStyles.group` に写す）、`resolve/describe.rs:205`（DevTools の記述）。

---

### 2 原因・組合せの分析

#### B1 ListItemState（selected × active）
4 通り全て到達する：
- (T,T)：Automatic activation の list・tree で矢印が active を動かしつつ選択する。
- (F,T)：Manual activation（`ListOptions::listbox()`・`menu()`、data.rs:282-298）では矢印は active だけを動かす。combobox は highlight（active）が値（selected）と別に動く（`combobox/handle.rs:338-354` の doc）。修飾 gesture（Ctrl+矢印）も選択せずに動かす。
- (T,F)：Multiple selection で選択済みの行から active が離れる。caller が selection binding を外から書き換える。
- (F,F)：その他の行。
`selected` と `active` は別の持ち主（選択集合／roving の active key）からの読み取りで、互いを含意しない。`input.focus`（PartState）と `active` の関係は lint の対象外（PartState は struct）で、今回は扱わない。

#### B2 Edit（observed × bounds）
`bounds` は構築時に決まる config（単一 slider は None、範囲 slider の thumb は Some）。`observed` は kit が part を attach したか（attach 前は None）。単一・範囲どちらの thumb も attach 前・後の両方があるので 4 通り全て有効。`observed` 自体は SM-5 の「後で埋める slot」だが、part は kit が setup の後段で `input()`/`thumb()` を呼んで作るので constructor に渡せない（`ButtonApi`・`SplitterApi`・toggle の `Core` と同じ形。それらは independent で閉じている）。

#### B3 State::Open（highlight × hovered × previewed）
settle 後は `previewed == hovered.or(list.active)`（list.active は Machine の外の `ListApi` が持つ）。`Event::Hover` の直後、effect が走るまでの間だけ一致しない。したがって previewed は独立の事実ではなく、「preview callback に最後に知らせた key」＝ `hovered` と list の active からの投影。Machine に list.active を入れることはできない（list が正本）。dedup（変化した時だけ通知）と Close 時の `None` 通知に前回値が要るので保持は必要。書き手の入口は `track_preview` 一つに集まっている → 投影で閉じる。

#### B4 TreeItemState（selected × active）
B1 と同じ理由。tree では Multiple selection と修飾 gesture（`tree.rs:545`）があり、selection binding は caller が書けるので 4 通り全て到達する。

#### B5 NodeStyle（grid_area × justify_self）
(None, Some) は作られない。含意を型にする：`justify_self` は grid item の配置の一部なので `GridArea` に入れる。`grid_area` は「grid item としての配置（column・row・justify_self）。無ければ auto 配置で container の justify_items に従う」という authored な option になり、他の option 群（aspect_ratio・align_* 等）と同じ閉じ方になる。

#### B6・B7 DetailWrite/DetailRun（cause × changed）
`cause` は write／run の瞬間の「その turn で最後に記録された cause の index」（turn の最初の cause の前、または cap で cause を捨てた後は None）。`changed` は前後の値の比較。cause の無い write（turn の最初の cause の前の mount 中の write、cap 超過後）も、cause のある write も、値を変える・変えないの両方がある → 4 通り有効。両方とも一度 push される不変の record。

#### B8 Reported.seq
状態ではなく、errors と diagnostics の二つの ring が共有する通し番号（id）。DevTools は `since` で paging し、virtual list の key にする → identity。

#### B9 ActionCell.next_handler
on_complete 登録の id 割当て器。`last`（最後に始まった run の入力）とは無関係に、run の有無・回数に関係なく登録の度に一つ進む。現在の登録の id は各 `DoneHandler.id` が持ち、cleanup はその id で外す → identity（A-RT の `Queue.next_worker_id` と同じ形）。

#### B10 StateFields.group
`style().group()` で author が付ける flag（Tailwind の `group`）。`group_hover`/`group_focus` は「最も近い祖先の group」の状態を読むので、自分が group であることとは独立（入れ子の group で、自分が group かつ外側の group の group_hover を持つ組合せも有効）。ほかの 11 field と同じ「authored な property」で、`merge` は OR（false に戻す setter が無いので「後勝ち」と同値）。

---

### 3 設計（確定版）

#### B1 `crates/kivact-interaction/src/list/data.rs:301-308`

型は変えない。marker を 2 行（`selected` の前に追加、`active` の前の行を置換）：

```rust
/// Selection, active candidate and actual input state for an attached item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListItemState {
    pub input: PartState,
    // xtask-allow(state-fields): independent: whether the selection holds the row; a multiple selection or a caller's binding write keeps rows selected while another row is active
    pub selected: bool,
    // xtask-allow(state-fields): independent: whether the row is the roving or highlighted candidate; arrows under Manual activation, a combobox highlight and Ctrl+arrow move it without selecting
    pub active: bool,
}
```

#### B2 `crates/kivact-interaction/src/scalar/edit_state.rs:146-147`

```rust
pub(crate) struct Edit {
    // xtask-allow(state-fields): independent: the state of the part the kit attached, absent until input() or thumb() attaches it; single and range thumbs (bounds None or Some) exist both before and after attaching
    pub(crate) observed: Signal<Option<Value<super::data::ScalarState>>>,
```

（他の行は変更なし。`bounds` の marker はそのまま。）

#### B3 `crates/kivact-interaction/src/select/machine.rs:22-24`

doc comment の上に置く：

```rust
        // xtask-allow(state-fields): independent: the key the pointer is over in the open list
        hovered: Option<K>,
        // xtask-allow(state-fields): projection: written only by track_preview through Event::Preview (the hovered key, else the list's active key); Open starts it at None and Close reports None
        /// The option last reported to the caller's preview callback.
        previewed: Option<K>,
```

併せて、送り手が一つであることを固定するため `select/handle.rs:787-790` の doc の末尾に 1 文足す：`/// This is the only sender of machine::Event::Preview.`（コードの変更は無い）。

#### B4 `crates/kivact-interaction/src/tree.rs:329-334`

```rust
pub struct TreeItemState {
    pub input: PartState,
    // xtask-allow(state-fields): independent: copied from the row's ListItemState; a multiple selection or a caller's binding write keeps rows selected while another row is active
    pub selected: bool,
    // xtask-allow(state-fields): independent: copied from the row's ListItemState; Ctrl+arrow moves the active row without selecting it
    pub active: bool,
    pub branch: TreeBranch,
    pub level: u32,
}
```

（`TreeItemState { item: ListItemState, .. }` に入れ子にすれば lint の行は消えるが、recipe が `s.item.selected` と書くことになり公開 API の平坦さ（方針 1）を損ね、他の `*ItemState` とも揃わないので採らない。）

#### B5 `crates/kivact-layout/src/style/node.rs`、`style/convert.rs`

型で閉じる。

node.rs:70-77 を置換：

```rust
/// An item's place in its grid container: both axes and its row-axis
/// alignment (most nodes are auto-placed in flex parents and have none).
#[derive(Clone, Copy, Debug)]
pub(crate) struct GridArea {
    pub(crate) column: GridLine,
    pub(crate) row: GridLine,
    /// CSS `justify-self` (`None`: the container's `justify_items`).
    pub(crate) justify_self: Option<AlignSelf>,
}
```

（`GridArea` の数える field は `justify_self` 一つなので audit の行も error も出ない。marker は付けない＝付けると unused marker になる。）

node.rs:110-115 を置換（`justify_self` field を削除）：

```rust
    pub(crate) flex_shrink: f32,
    // xtask-allow(state-fields): option: the node's placement as a grid item (column, row, justify-self); absent means auto-placed and aligned by the container's justify_items
    /// Its place as a grid item (absent: auto-placed).
    pub(crate) grid_area: Option<Box<GridArea>>,
}
```

node.rs:152-153 の `Default`：`justify_self: d.justify_self,` の行を削除（`grid_area: None` は残す。taffy の `TStyle::DEFAULT.justify_self` は `None` なので振る舞いは同じ）。

node.rs:365-368 の読み手を置換：

```rust
    #[inline(always)]
    fn justify_self(&self) -> Option<AlignSelf> {
        self.grid_area.as_deref().and_then(|area| area.justify_self)
    }
```

convert.rs:182-188 を置換：

```rust
        grid_area: item.map(|g| {
            Box::new(GridArea {
                column: g.column,
                row: g.row,
                justify_self: g.justify_self.map(align),
            })
        }),
    };
```

（`justify_self: item.and_then(..)` の行を削除。`node.rs:116-119` の `size_of::<NodeStyle>() <= 264` は小さくなる方向なので変更不要。）

変更後の NodeStyle の audit 行：aspect_ratio・align_items・align_self・align_content・justify_content・grid_area（全て option）。justify_self の行は消える。

#### B6・B7 `crates/kivact-runtime/src/reactive/detail.rs`

DetailWrite（:58-60）：

```rust
    pub node: NodeId,
    // xtask-allow(state-fields): independent: the turn's last cause when the write happened (None before the turn's first cause or after the cap dropped one); a write with or without a cause may or may not change the value
    /// The index of the last cause before the write, if any.
    pub cause: Option<u32>,
```

DetailRun（:79-81）：

```rust
    pub node: NodeId,
    // xtask-allow(state-fields): independent: the turn's last cause when the run happened (None before the turn's first cause or after the cap dropped one); a run with or without a cause may or may not change its value
    /// The index of the last cause before the run, if any.
    pub cause: Option<u32>,
```

#### B8 `crates/kivact-runtime/src/reactive/diag.rs:31-33`

```rust
pub struct Reported {
    // xtask-allow(state-fields): identity: the report's number in the sequence the error and diagnostic rings share; DevTools pages by it (since) and keys rows by it
    /// The report sequence number (the rings share one counter).
    pub seq: u64,
```

#### B9 `crates/kivact-runtime/src/task/action.rs`

改名 `next_handler` → `next_handler_id`（private、3 箇所）。

:254-255：

```rust
    on_complete: RefCell<Vec<DoneHandler<T>>>,
    // xtask-allow(state-fields): identity: allocates on_complete registration ids; each DoneHandler keeps its own id and its owner's cleanup removes it by that id
    /// The id the next `on_complete` registration takes.
    next_handler_id: Cell<Generation>,
```

:385：`next_handler_id: Cell::new(Generation::new()),`

:760-762：

```rust
        let mut generation = cell.next_handler_id.get();
        let id = generation.next();
        cell.next_handler_id.set(generation);
```

併せて既存欠陥 D2 の修正（`last` の書き手を一つの関数に集める）をこの unit で行う（7 章）。

#### B10 `crates/kivact-style/src/sheet/fields.rs:389-390`

```rust
    pub(crate) hover: Option<Box<Style>>,
    // xtask-allow(state-fields): option: one authored flag (style().group()); false means the node is not a group, and group_hover/group_focus read the nearest ancestor group, not this flag
    pub(crate) group: bool,
```

---

### 4 見積り
状態の閉じ方のみ。性能への影響：B5 で `NodeStyle` から `Option<AlignSelf>`（2 B）が消え、`Box<GridArea>` が 2 B 増える（grid item のノードだけ）。他は marker と private の改名だけで、計算量・メモリは不変。AC-3：本 slice の OPEN 10 行が 0 になり、新しい OPEN 行は出ない（B5 で消える行 1、`GridArea` は数える field が 1 つで行を作らない）。

---

### 5 正しさ・リスク・E2E

| # | 振る舞い | 担保 |
|---|---|---|
| B1・B4 | 不変（marker のみ） | 既存：`dev/tests/kivact-public-api/tests/e2e/` の list/select/combobox/tree 系（変更不要） |
| B2 | 不変 | 既存の slider/range slider の E2E |
| B3 | 不変 | 既存の select の preview の E2E（変更不要） |
| B5 | 不変（同じ値を別の場所に持つだけ）。ただし **justify_self を見る E2E が無い**（Grep：`dev/tests` に `justify_self` 無し） | **追加**（下記） |
| B6・B7・B8 | 不変 | DevTools の既存試験 |
| B9 | 不変（private の改名） | 既存 `task::action_latest_wins_retry_cancel_and_multiple_done_handlers`（複数の on_complete）、`task::queue_orders_coalesces_recovers_and_drains`（`input()` :276） |
| B10 | 不変 | 既存の group_hover/group_focus の E2E |

#### 追加する E2E（B5）
`dev/tests/kivact-public-api/tests/e2e/grid.rs` の末尾に追加（実 window。`render_with`・`rect_of`・`assert_rect`・`window` は同 file で import 済み、`Align` は `kivact::prelude::*` に含まれる）：

```rust
#[test]
fn grid_items_justify_self_within_their_area() {
    let screen = render_with(window(800.0, 600.0), || {
        grid((
            node().test_id("start").style(style().height(10).width(50)),
            node()
                .test_id("end")
                .style(style().height(10).width(50).justify_self(Align::End)),
            node().test_id("center").style(
                style()
                    .height(10)
                    .width(50)
                    .justify_self(Align::Center)
                    .grid_column(1)
                    .grid_row(2),
            ),
            // A grid container placed as an item keeps its own placement.
            grid((node().test_id("inner").style(style().height(10)),))
                .test_id("nested")
                .style(
                    style()
                        .width(100)
                        .justify_self(Align::End)
                        .grid_column(2)
                        .grid_row(2)
                        .grid_template_columns(Track::fr(1)),
                ),
        ))
        .style(style().grid_template_columns((400, 400)))
    });
    assert_rect(rect_of(&screen, "start"), (0.0, 0.0, 50.0, 10.0), "default start");
    assert_rect(rect_of(&screen, "end"), (750.0, 0.0, 50.0, 10.0), "justify-self end");
    assert_rect(rect_of(&screen, "center"), (175.0, 10.0, 50.0, 10.0), "justify-self center");
    assert_rect(rect_of(&screen, "nested"), (700.0, 10.0, 100.0, 10.0), "nested grid end");
}
```

この test は変更前（HEAD）でも通ることを最初に確認し（回帰検出用）、変更後も通ることを確認する。HEAD で通らない場合は既存の欠陥なので、期待値を変えずに STOP して報告する。

---

### 6 unit 分割案

| unit | crate | 所有ファイル | 依存 | 完了時の確認 |
|---|---|---|---|---|
| UB-1 | kivact-interaction | `src/list/data.rs`、`src/tree.rs`、`src/scalar/edit_state.rs`、`src/select/machine.rs`、`src/select/handle.rs`（doc 1 文のみ） | なし | `cargo check -p kivact-interaction`；`cargo xtask check --no-write --only state-fields --crate kivact-interaction` |
| UB-2 | kivact-layout（+ E2E） | `crates/kivact-layout/src/style/node.rs`、`crates/kivact-layout/src/style/convert.rs`、`dev/tests/kivact-public-api/tests/e2e/grid.rs` | なし | `cargo clippy -p kivact-layout --all-targets -- -D warnings`；`cargo xtask check --no-write --only state-fields --crate kivact-layout`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e grid::` |
| UB-3 | kivact-runtime | `src/reactive/detail.rs`、`src/reactive/diag.rs`、`src/task/action.rs` | なし | `cargo clippy -p kivact-runtime --all-targets -- -D warnings`；`cargo xtask check --no-write --only state-fields --crate kivact-runtime`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e task::action_ task::queue_` |
| UB-4 | kivact-style | `src/sheet/fields.rs` | なし | `cargo check -p kivact-style`；`cargo xtask check --no-write --only state-fields --crate kivact-style` |

UB-1〜4 は互いに独立（並列可）。UB-4 は 1 行なので他の kivact-style の unit があればそこに合流してよい。

---

### 7 既存欠陥

**D1（規則違反、確定）projection marker の関数名が事実と違う。**
- `crates/kivact-interaction/src/list/data.rs:306`「written only by ListApi::item_state」：実際の書き手は `list/handle.rs:68`、`list/parts.rs:91`、`combobox/handle.rs:344`、`combobox/handle.rs:680` の 4 箇所。
- `crates/kivact-interaction/src/tree.rs:332`「written only by TreeApi::item」：実際は `tree.rs:525`（`item_state`）と `tree.rs:704`（`item`）。
- そもそも `active` は二つの独立した持ち主からの snapshot で、同じ事実の投影（SM-12）ではない。修正は 3 章 B1・B4（independent に書き換え）。lint は projection の関数名の実在・唯一性を検査していない（P5 本体の lint 抜け穴の節で扱う例）。

**D2（SM-12 からのずれ、確定）`ActionCell.last` の projection が一つの関数に集まっていない。**
`crates/kivact-runtime/src/task/action.rs:249` の marker は「(Latest make, worker pop)」と二つの書き手を挙げる。実際の書き手は `start_latest` の closure（:516）と queue worker の loop（:586）。修正：private な関数を一つ作り、両方をそれ経由にする。

```rust
impl<T: 'static, I: 'static> ActionCell<T, I> {
    /// Start one run of `submitted`: the only writer of `last`. `publish`
    /// updates the action's state after `last` names the run and before the
    /// run's future is made.
    fn begin_run(
        &self,
        submitted: &Submitted<I>,
        publish: impl FnOnce(&Self),
    ) -> Pin<Box<dyn Future<Output = Result<T, UiError>>>> {
        *self.last.borrow_mut() = Some(submitted.clone());
        publish(self);
        (self.run)(&submitted.input)
    }
}
```

- :515-518 を `|| self.begin_run(&submitted, |cell| cell.async_state.start())` に置換（今の順序「last → start → run」を保つ）。
- :586-588 を `cell.begin_run(&submitted, |cell| cell.queue_status())` に置換（今の順序「last → queue_status → run」を保つ。`queue.set_running(true)` は :585 のまま前に残す）。
- `Submitted<I>` が `Clone` でない場合は今の `submitted.clone()` と同じ実装を使う（今のコードが `.clone()` を呼んでいるので Clone は実装済み）。
- :249 の marker を `// xtask-allow(state-fields): projection: written only by ActionCell::begin_run (the input of the latest started run; None before the first run)` に置換。
- 担保：既存 E2E `task::action_latest_wins_retry_cancel_and_multiple_done_handlers`（`input()`・`attempt()`、:211）と `task::queue_orders_coalesces_recovers_and_drains`（:276 `fifo.input()`）。振る舞いは不変。

**D3（推測）`ScalarApi::part` を 2 回呼ぶと `observed` は後の part を指す。** `scalar/handle.rs:386` は上書きで、前の part の状態は `control_state` から見えなくなる。kit が同じ `ScalarApi` で `input()` と `thumb()` の両方を作る使い方は今の kit には無い（Grep で `.part(Role::` は handle.rs:274,285 の 2 つの公開関数のみ）ので、現時点の不具合ではない。修正不要、記録のみ。

---

## 付録 P5-C：P5 slice C：OPEN 5 件（alert / calendar / data_table filter / node editor Info / pop catalog）

対象（HEAD 73c36c7c で行番号を確認済み。`dev/xtask/state-audit.md` の行と一致）:

| # | audit 行 | 閉じ方（確定） |
|---|---|---|
| C1 | `crates/kivact-ui/src/alert.rs:25` `BannerState::Shared.shown` | 型で表す（`BannerState { Shown { restore }, Hidden }` ＋ snapshot event。旧付録 A-UI の SM-9 を実装） |
| C2 | `crates/kivact-ui/src/calendar_values.rs:34` `CalendarDayVisual.mark` | 真に独立（理由を具体化した marker。today / in_month の既存 marker の文言も具体化） |
| C3 | `crates/kivact-ui/src/data_table/filters.rs:240` `FilterDraft::Valid.value` | 導出して保持しない（重複の `sent` を削除し、echo 判定は `value` から導く） |
| C4 | `crates/kivact-ui-creative/src/editors/node/gesture.rs:60` `Info.selected` | 型で表す（`checked`・`selected`・`range` を target の種類ごとの `Facet` enum に） |
| C5 | `examples/kivact-ui-pop/src/pop/shell.rs:102` `catalog_with_mode::overlay` | 型で表す（既に `Overlay` enum。行は lint の誤検出：`Overlay::None` を `Option::None` と誤認）→ lint を直す |

変更後、この 5 件に由来する audit 行は全て消え（C2 だけは 3 行とも `independent` で残る）、新しい OPEN 行は出ない（各節末で確認）。

---

### 1 事実・2 原因・3 設計（件ごと）

#### C1 `BannerState::Shared.shown`（kivact-ui/src/alert.rs）

#### 1. field・書き手・読み手

`alert.rs:15-38`:

```rust
enum LocalBannerState { Shown { restore: Option<NodeToken> }, Dismissed }
enum BannerState {
    Local(LocalBannerState),
    Shared { shown: bool, /* marker independent */ restore: Option<NodeToken> },
}
```

lint が数える field: `Shared.shown`（bool、OPEN）、`Shared.restore`（Option、`independent` marker :26）。`LocalBannerState::Shown` は 1 field なので行なし。

書き手（全て `BannerMachine::transition` と初期値）:
- `alert.rs:359-361` `Visibility(shown)` → `Shared { shown, restore }`（restore を保持したまま shown だけ置換）
- `alert.rs:362-376` `ExternalFocus { focused: Some(t), within: false }` かつ `state.visible()` → restore = Some(t)（Local/Shared とも）
- `alert.rs:377-385` Local Shown + Dismiss → `Local(Dismissed)`
- `alert.rs:386-394` Shared{shown:true} + Dismiss、かつ `self.visible` の binding を **transition の中で `get_untracked()`**（:394）→ stay + `RequestDismiss { restore }`
- 初期値 `alert.rs:404-411`（Local は `Shown{None}`、Shared は `shown = visible.get_untracked()`, restore None）

入力経路:
- `alert.rs:435-437` `phase.feed(visible.into_value(), BannerEvent::Visibility)`（`feed` は `effect_on`、`kivact-runtime/src/machine.rs:239-246`）
- `alert.rs:438-447` 別の `effect` が `visible.get()` を**購読だけ**して `ExternalFocus { focused: owner.focused_node(), within }` を送る
- `alert.rs:256` dismiss button → `BannerEvent::Dismiss`

読み手: `BannerState::visible()`（:31-37）→ `alert.rs:190` `.hidden(move || !phase.state().get().visible())`。restore は Dismiss の transition だけが読む。

#### 2. 組合せ

`Shared`: (shown=T, restore=None/Some) は到達・有効。(shown=F, restore=Some) は **到達するが無意味**：`Visibility(false)` は restore を残し、隠れている間は `ExternalFocus` が無視される（`state.visible()` guard）。再表示時に `Visibility(true)` が**前回表示時の stale な restore** をそのまま持ち越す。再表示後の focus 記録は「`feed` の effect と `ExternalFocus` の effect のどちらが先に走るか」に依存する（`feed` が後なら restore は stale のまま。`focused_node()` が None なら必ず stale）。つまり shown と restore は独立ではなく「restore は shown の間だけ意味がある」= SM-2。さらに旧付録 A-UI `alert.rs:355-395` 行（SM-9「binding を input の snapshot に入れ、transition を純粋にする」）は**未実装**（:394 の transition が binding を読む）。

Local/Shared の別は構築時に `visible.is_some()` で決まり不変。state に持つ必要はない（machine の設定）。

#### 3. 設計（確定）

- state は表示中だけ restore を持つ 1 enum。所有（Local / Application）は `BannerMachine` の不変の設定。
- 可視性と focus を **1 つの snapshot event** にまとめ、1 つの effect から送る（effect の実行順に依存しない）。`feed` は削除。
- transition は純粋。binding を読む「拒否された書込みの判定」は effect handler 側（副作用の場所）に置く。

```rust
#[derive(Clone, Copy, PartialEq)]
enum BannerState {
    /// Visible; `restore` is the last focused node outside the banner while it was visible.
    Shown { restore: Option<NodeToken> },
    /// Dismissed by the user (local) or hidden by the application's binding.
    Hidden,
}
impl BannerState {
    fn visible(self) -> bool {
        matches!(self, Self::Shown { .. })
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Ownership {
    /// The alert owns its visibility; dismissal lasts until remount.
    Local,
    /// The application's `open` binding owns the visibility.
    Application,
}
enum BannerEvent {
    /// One snapshot: the application's visibility (None when the alert owns it)
    /// and the focused node with whether it lies inside the banner.
    Observed { shown: Option<bool>, focused: Option<NodeToken>, within: bool },
    Dismiss,
}
enum BannerEffect {
    RestoreFocus(NodeToken),
    NotifyDismiss,
    RequestDismiss { restore: Option<NodeToken> },
}
struct BannerMachine {
    ownership: Ownership,
}
impl Machine for BannerMachine {
    type State = BannerState;
    type Event = BannerEvent;
    type Effect = BannerEffect;
    fn transition(&self, state: &BannerState, event: BannerEvent)
        -> Step<BannerState, BannerEvent, BannerEffect>
    {
        match (*state, event) {
            (state, BannerEvent::Observed { shown, focused, within }) => {
                let outside = focused.filter(|_| !within);
                match (state, shown) {
                    (BannerState::Shown { .. }, Some(false)) => Step::to(BannerState::Hidden),
                    (BannerState::Hidden, Some(true)) => {
                        Step::to(BannerState::Shown { restore: outside })
                    }
                    (BannerState::Shown { .. }, _) if outside.is_some() => {
                        Step::to(BannerState::Shown { restore: outside })
                    }
                    _ => Step::stay(),
                }
            }
            (BannerState::Shown { restore }, BannerEvent::Dismiss) => match self.ownership {
                Ownership::Local => {
                    let step = Step::to(BannerState::Hidden);
                    let step = match restore {
                        Some(target) => step.with(BannerEffect::RestoreFocus(target)),
                        None => step,
                    };
                    step.with(BannerEffect::NotifyDismiss)
                }
                Ownership::Application => {
                    Step::stay().with(BannerEffect::RequestDismiss { restore })
                }
            },
            (BannerState::Hidden, BannerEvent::Dismiss) => Step::stay(),
        }
    }
}
fn banner_dismissal(
    reference: NodeRef,
    callback: Option<Callback<()>>,
    visible: Option<Binding<bool>>,
) -> MachineHandle<BannerMachine> {
    let initial = match visible {
        Some(visible) if !visible.get_untracked() => BannerState::Hidden,
        _ => BannerState::Shown { restore: None },
    };
    let ownership = if visible.is_some() { Ownership::Application } else { Ownership::Local };
    let phase = machine(
        initial,
        move || BannerMachine { ownership },
        move |_, command| match command {
            BannerEffect::RestoreFocus(target) => target.restore_focus(),
            BannerEffect::RequestDismiss { restore } => {
                let Some(visible) = visible else { return };
                // The binding may already be false before this snapshot arrived.
                if !visible.get_untracked() {
                    return;
                }
                visible.set(false);
                // A refused write keeps the alert visible: no focus move, no callback.
                if visible.get_untracked() {
                    return;
                }
                if let Some(target) = restore {
                    target.restore_focus();
                }
                if let Some(callback) = callback {
                    callback.call(());
                }
            }
            BannerEffect::NotifyDismiss => {
                if let Some(callback) = callback {
                    callback.call(());
                }
            }
        },
    );
    let owner = Owner::current();
    let within = reference.focus_within();
    effect(move || {
        phase.send(BannerEvent::Observed {
            shown: visible.map(|visible| visible.get()),
            focused: owner.focused_node(),
            within: within.get(),
        });
    });
    phase
}
```

削除: `LocalBannerState`（:15-19）、`BannerEvent::Visibility`・`ExternalFocus`、`BannerMachine.visible`、`phase.feed(..)`（:435-437）、marker 行 :26。`alert.rs:256`（`phase.send(BannerEvent::Dismiss)`）と `:190`（`visible()`）はそのまま。import の `Binding` は引数で使うので残す。

lint 後: `BannerState::Shown` は counted 1 field → 行なし。`BannerEvent` は role 接尾辞 `Event` で除外。`BannerEffect::RequestDismiss` は 1 field。`BannerMachine` は `Ownership`（phase 接尾辞でない enum）だけ → Class::None。**alert.rs の audit 行は 2 行（:25 OPEN、:27 independent）とも消える。**

#### 4. 呼出元
`banner_dismissal` の呼出元は `alert.rs` 内の 1 箇所のみ（`Alert` の render、シグネチャ不変）。kit（basic/pop/game/creative/soft）は `ui::alert(..)` の公開 API だけを使うため変更なし。公開 API 不変。

#### 5. 振る舞い・E2E
- 変わる点：隠れていた controlled alert が再表示されたとき、restore は「再表示時点で banner 外にある focus」になり、前回表示時の stale な target を使わない。再表示時の focus 記録が effect の実行順に依存しなくなる。その他（local dismiss、拒否された書込み、受理後の focus 復帰と callback）は不変。
- 既存で担保：`dev/tests/kivact-public-api/tests/e2e/feedback.rs:36` `alert_dismiss_restores_focus`、`:159` `controlled_alert_keeps_refused_visibility_and_notifies_after_acceptance`（再表示→dismiss→`outside` に復帰まで含む）、`:137` `alert_details_follow_the_open_binding`、`apg_alert.rs:8`。
- 追加する E2E（`dev/tests/kivact-public-api/tests/e2e/feedback.rs`、`controlled_alert_keeps_refused_visibility_and_notifies_after_acceptance` の直後）:

```rust
#[test]
fn controlled_alert_shown_again_restores_focus_to_the_control_focused_while_hidden() {
    let (screen, visible) = render_with_handle(options(), || {
        let visible = signal(true);
        (
            visible,
            column((
                ui::button("First"),
                ui::button("Second"),
                ui::alert("Controlled alert")
                    .open(visible)
                    .dismissible(true)
                    .dismiss_label("Dismiss controlled alert"),
            )),
        )
    });
    screen.get_by_role(Role::Button, "First").focus();
    visible.set(false);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Alert, "Controlled alert").is_none());
    let second = screen.get_by_role(Role::Button, "Second");
    second.focus();
    visible.set(true);
    screen.run_until_idle();
    screen
        .get_by_role(Role::Button, "Dismiss controlled alert")
        .click();
    assert!(!visible.get_untracked());
    assert!(second.is_focused(), "focus returns to the control focused when the alert reappeared");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```

---

#### C2 `CalendarDayVisual.mark`（kivact-ui/src/calendar_values.rs）

#### 1. field・書き手・読み手
`calendar_values.rs:24-35`（公開 struct、recipe への読取り専用 projection）。lint が数える: `today: bool`（:31、marker `independent: a measured value; any value is valid on its own`）、`in_month: bool`（:33、同文言）、`mark: Option<Tone>`（:34、OPEN）。`day: CalendarDayState` は phase 接尾辞の名前でない struct なので数えない。

書き手: **唯一** `crates/kivact-ui/src/calendar/panel.rs:187-194` の recipe state closure：
`today: day == today.get()`、`in_month: day.month_of() == month`、`mark: marks(day).or_else(|| weekend(day.weekday()))`。`marks`／`weekend` は caller の `Calendar::marks`（`calendar/view.rs:226-231`）／`Calendar::weekend`（`view.rs:151-156`）の callback（既定は常に None、`view.rs:86,88`）。

読み手: `crates/kivact-ui/src/look_calendar.rs:19-53`（`s.today`、`s.in_month`、`s.mark.is_some()`）、`crates/kivact-ui-soft/src/recipes_calendar.rs:59`（`s.mark.is_some()`）、`kivact-ui-basic/src/recipes_calendar.rs:42`・`kivact-ui-pop/src/recipes_calendar.rs:44`（Look 経由）。

#### 2. 組合せ
8 通り全て到達・有効。today は時計（`shared.today`）、in_month は表示月、mark は caller の callback が日付だけから決める。grid は前月・翌月の日も描くので「today かつ月外」も起きる（月末・月初）。caller はどの日付にも tone を付けられ、weekend は曜日だけで決まる。どの値も他の値を含意・排除しない。

#### 3. 設計（確定）：真に独立。型は変えない
`calendar_values.rs:30-34` を次に置き換える（marker は field の直前の行。既存 2 本の文言は汎用で理由になっていないので具体化する）:

```rust
    // xtask-allow(state-fields): independent: date == the clock's today; the grid shows today inside or outside the displayed month, with or without a tone
    pub today: bool,
    // xtask-allow(state-fields): independent: date lies in the displayed month; the grid's leading and trailing days are outside it whether or not they are today or toned
    pub in_month: bool,
    // xtask-allow(state-fields): independent: the tone from Calendar::marks, else Calendar::weekend, for this date; the caller's callbacks may tone any date, today or not, in the month or not
    pub mark: Option<kivact_interaction::Tone>,
```

lint 後: 3 行とも `independent`。OPEN 0。

#### 4. 呼出元: なし（コメントのみ）。
#### 5. 振る舞い: 不変。E2E 追加なし（コードの変更なし）。

（旧付録 A-IN `calendar/data.rs` の `DaySelection { None, Inside, Edge }` 行は実装済み：`kivact-interaction/src/calendar/data.rs:35-44`、`panel.rs:160`。）

---

#### C3 `FilterDraft::Valid.value`（kivact-ui/src/data_table/filters.rs）

#### 1. field・書き手・読み手
`filters.rs:236-247`:

```rust
enum FilterDraft {
    Valid { text: String, value: Option<FilterValue>, /* marker cache */ sent: Sent<Option<FilterValue>> },
    Invalid { text: String },
}
```

lint: `value`（Option → Stored、OPEN）、`sent`（named helper `Sent`、Stored があるので昇格、marker `cache:` :241）。

書き手（全て `FilterInput::transition` と初期値）:
- 初期 `filters.rs:335-343`：`sent.update(initial)`、`value: initial`
- `:298-307` `External(v)`（echo でない時）：`sent.update(v)`、`value: v`
- `:309-317` `Typed(text)` で parse 成功：`sent.update(v)`、`value: v`
- `:319` parse 失敗 → `Invalid { text }`（value・sent とも消える）

`Sent::update`（`kivact-types/src/cell.rs:90-96`）は値が等しければ据え置き、違えば置換するので、**全ての Valid で `sent.get() == Some(&value)` が常に成り立つ**。

読み手: `sent` は `:294-297` の echo 判定（`state.sent().is_some_and(|s| s.get() == Some(&value))`）のみ。`value` は手書き `PartialEq`（:248-266、sent を無視）のみ。`text()` は text。

#### 2. 組合せ
Valid の中で value と sent は常に等しい（同じ事実の重複保存）。「value ≠ sent」は到達しない無効な組合せ。

#### 3. 設計（確定）：導出して保持しない（`sent` を削除し、echo 判定は `value` から導く）

```rust
#[derive(Clone, PartialEq)]
enum FilterDraft {
    /// The text parses; `value` is the value last written to, or read from, the caller's binding.
    Valid { text: String, value: Option<FilterValue> },
    /// The text does not parse; nothing is written to the binding.
    Invalid { text: String },
}
impl FilterDraft {
    fn text(&self) -> &str {
        match self {
            Self::Valid { text, .. } | Self::Invalid { text } => text,
        }
    }
    /// Whether `incoming` is the value this draft already holds (its own write echoed back).
    fn holds(&self, incoming: &Option<FilterValue>) -> bool {
        matches!(self, Self::Valid { value, .. } if value == incoming)
    }
}
enum FilterEvent {
    External(Option<FilterValue>),
    Typed(String),
}
struct FilterInput {
    field: FilterField,
}
impl Machine for FilterInput {
    type State = FilterDraft;
    type Event = FilterEvent;
    /// The value to write to the caller's binding.
    type Effect = Option<FilterValue>;
    fn transition(&self, state: &FilterDraft, event: FilterEvent)
        -> Step<FilterDraft, FilterEvent, Self::Effect>
    {
        match event {
            FilterEvent::External(value) if state.holds(&value) => Step::stay(),
            FilterEvent::External(value) => Step::to(FilterDraft::Valid {
                text: format_value(value.as_ref()),
                value,
            }),
            FilterEvent::Typed(text) => match parse_value(&self.field, &text) {
                Ok(value) => Step::to(FilterDraft::Valid { text, value: value.clone() }).with(value),
                Err(()) => Step::to(FilterDraft::Invalid { text }),
            },
        }
    }
}
```

`text_editor`（`filters.rs:328-370`）の書換え:

```rust
    let initial = value.get_untracked();
    let input = field.clone();
    let draft = machine(
        FilterDraft::Valid {
            text: format_value(initial.as_ref()),
            value: initial,
        },
        move || FilterInput { field: input.clone() },
        move |_, next| value.set(next),
    );
    draft.feed(value.into_value(), FilterEvent::External);
    // 以降（valid memo・hint・binding・text_input）は変更なし
```

削除: 手書き `impl PartialEq for FilterDraft`（:248-266、derive に置換）、`fn sent()`（:274-279）、`let mut sent = Sent::new(); sent.update(..)`（:331-332）、`use kivact_core::custom::Sent;`（:5）、marker :241。`FilterValue` が `PartialEq` であることは現行の手書き impl が既に要求している。

lint 後: `FilterDraft::Valid` は counted 1 field（value）→ 行なし。:240・:242 の 2 行とも消える。

（旧付録 A-UI `data_table/filters.rs:294-312` 行「`FilterDraft::Valid { text, value, sent }`」は文字どおり実装されたが、その形は value と sent の重複を生む。V2 ではこの行を本設計で置き換える。）

#### 4. 呼出元: `text_editor` のみ（同ファイル :231）。公開 API 不変。
#### 5. 振る舞い: 不変（echo の比較対象が同値の別 field から value に変わるだけ）。外部効果 `(false, v)` は元々何もしない effect で、削除しても同じ。
- 既存で担保：`crates/kivact-ui-soft/tests/consumer/typed_table_filters.rs:31` `filter_popover_discards_unapplied_values_and_can_reset_a_typed_query`、`:90` `compact_inline_filter_commits_valid_values_without_a_popup`（kivact-test の実 window）。
- 追加（echo で打った文字列が整形し直されないことを固定する。同ファイル末尾）:

```rust
#[test]
fn inline_filter_keeps_typed_text_through_its_own_echo_and_follows_external_values() {
    let (screen, filters) = render_with_handle(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)).mirror(MirrorMode::Off),
        || {
            let filters = signal(HashMap::new());
            (
                filters,
                data_table(vec![("Ten", 10), ("Thirty", 30)], |row| row.1, vec![column()])
                    .typed_column_filters(filters)
                    .filter_ui(FilterUi::InlineCompact),
            )
        },
    );
    let input = screen.get_by_role(Role::TextBox, "Filter Score");
    input.type_text("21.50");
    assert_eq!(filters.get_untracked().get(&0), Some(&FilterValue::Number(21.5)));
    assert_eq!(input.value().as_deref(), Some("21.50"), "the echo of its own write keeps the typed text");
    filters.set(HashMap::from([(0, FilterValue::Number(30.0))]));
    screen.run_until_idle();
    assert_eq!(input.value().as_deref(), Some("30"));
    assert!(screen.query_by_role(Role::Row, "Ten").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```

（`filters` の key 型は既存試験 `filters.get_untracked().get(&0)` と同じ。`HashMap::from` の key は既存の `typed_column_filters` の型に合わせ整数 literal で推論させる。）

---

#### C4 `Info.selected`（kivact-ui-creative/src/editors/node/gesture.rs）

#### 1. field・書き手・読み手
`gesture.rs:51-63` `pub(crate) struct Info { id, target, name, value, role, checked: Option<bool> /*option marker :58*/, selected: bool /*OPEN*/, range: Option<(f64,f64,f64)> /*option marker :61*/ }`。

書き手（全て `crates/kivact-ui-creative/src/editors/node/a11y.rs`）:
- `:44-55` / `:122-133` の closure `info(target, name, value, role, checked, selected, range)`
- Item `:57-65` → (None, false, None)
- Socket `:66-77` → (None, false, None)
- Control: Number `:80-88` → (None, false, Some((min,max,value)))；Choice `:89-94` → (None, false, None)；Toggle `:95-97` → (Some(v), false, None)
- Link `:140-149` → (None, `selected_link == Some(link)`, None)

読み手（全て `crates/kivact-ui-creative/src/editors/node/view.rs`）:
- `:327-333` `initial.range.is_some()`、`info.get().range.map_or(..)`
- `:335-339` `initial.checked.is_some()`、`info.get().checked.unwrap_or(false)`
- `:341-344` Link のとき `info.get().selected`
- `:363` `match (initial.role, initial.checked.is_some())`

#### 2. 組合せ
3 つは target の種類で排他：checked は Toggle の時だけ Some、range は Number の時だけ Some、selected は Link の時だけ意味を持つ（他は常に false で読まれない）。(checked Some, range Some)、(selected true, checked Some) 等は到達しない。`option:` marker の 2 本も実体は「種類による排他」を隠している。

#### 3. 設計（確定）：型で表す

`gesture.rs:51-63` を置換:

```rust
/// What a region reports besides its name, value and role; decided by the target's kind.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Facet {
    /// Items, sockets and choice controls.
    Plain,
    /// A toggle control.
    Toggle { checked: bool },
    /// A number control.
    Range { min: f64, max: f64, now: f64 },
    /// A link between two sockets.
    Link { selected: bool },
}

#[derive(Clone, PartialEq)]
pub(crate) struct Info {
    pub(crate) id: RegionId,
    pub(crate) target: Target,
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) role: Role,
    pub(crate) facet: Facet,
}

impl Info {
    pub(crate) fn checked(&self) -> Option<bool> {
        match self.facet {
            Facet::Toggle { checked } => Some(checked),
            _ => None,
        }
    }
    pub(crate) fn range(&self) -> Option<(f64, f64, f64)> {
        match self.facet {
            Facet::Range { min, max, now } => Some((min, max, now)),
            _ => None,
        }
    }
    pub(crate) fn selected(&self) -> bool {
        matches!(self.facet, Facet::Link { selected: true })
    }
}
```

`a11y.rs` の書換え（`use super::gesture::{Facet, Info};`）:
- closure 2 箇所（:44、:122）: `let info = |target, name, value, role, facet| scene.id(&target).map(|id| Info { id, target, name, value, role, facet });`
- Item（:57-65）: `info(Target::Item(node.id), node.label.clone(), texts.selection(..), Role::Button, Facet::Plain)`
- Socket（:68-76）: 最後の 3 引数を `Facet::Plain` に
- Control（:79-107）:
```rust
            let (role, value, facet) = match &control.value {
                NodeControlValue::Number { value, min, max, .. } => (
                    Role::SpinButton,
                    format!("{value:.3}"),
                    Facet::Range { min: *min, max: *max, now: *value },
                ),
                NodeControlValue::Choice { options, selected } => (
                    Role::Button,
                    options.get(*selected).cloned().unwrap_or_default(),
                    Facet::Plain,
                ),
                NodeControlValue::Toggle(value) => {
                    (Role::Button, texts.toggle(*value), Facet::Toggle { checked: *value })
                }
            };
            result.extend(info(Target::control(node, index), format!("{} {}", node.label, control.label), value, role, facet));
```
- Link（:141-149）: `info(Target::link(link), texts.port_link(..), String::new(), Role::Button, Facet::Link { selected: selected_link == Some(link) })`

`view.rs` の書換え:
- :327 `if initial.range().is_some() {`、:330-332 `info.get().range().map_or(0.0, |v| v.0)`（.1／.2 も同様）
- :335 `if initial.checked().is_some() {`、:338 `info.get().checked().unwrap_or(false)`
- :343 `node.part(control).selected(move || info.get().selected())`
- :363 `match (initial.role, initial.checked().is_some()) {`

削除: marker :58・:61。

lint 後: `Info` は counted field なし。`Facet::Toggle`・`Facet::Link` は各 1 field（bool）で行なし。:59・:60・:62 の 3 行とも消える。

（旧付録 A-KC の NodeController 行「`Choose` と `update` を消し意味の event を足す」は実装済み：`gesture.rs:86-110` の `ModeEvent::{BeginConnect, OpenPopup, ClosePopup, UpdatePopupSlot, Cancel, Inputs}`、`Choose`/`update` は src に無い。）

#### 4. 呼出元: 上記 2 ファイルのみ（`Info` は crate 内 `pub(crate)`）。公開 API 不変。
#### 5. 振る舞い: 不変。
- 既存で担保：`crates/kivact-ui-creative/tests/consumer/node_editor.rs:134` `node_parameters_accept_finite_set_value_drag_and_toggle`（range・checked・Toggle/Activate）、`:23`（socket・link の Activate、item Select）。
- 追加（link の selected を固定。`node_editor.rs` 末尾、`typed_multiple_sockets_validate_reconnect_and_delete_nodes` と同じ 2 node と socket を使い、links の初期値を `vec![ui::NodeLink::new(1, 0, 2, 0)]` にする）:

```rust
#[test]
fn node_link_region_reports_its_selection() {
    let (screen, links) = kivact::test::render_with_handle(crate::support::options(), || {
        let socket = ui::NodeSocket::new;
        let data = signal(vec![
            test_node(1, "Image", 30.0, vec![], vec![socket("Image", "Image", kivact::style::rgb(0xffd25a))]),
            test_node(2, "Mix", 300.0, vec![socket("Image", "Image", kivact::style::rgb(0xffd25a))], vec![]),
        ]);
        let links = signal(vec![ui::NodeLink::new(1, 0, 2, 0)]);
        (
            links,
            app_node_editor(data, links, ui::NodeState::new())
                .style(style().height(300))
                .test_id("nodes"),
        )
    });
    let link = screen.get_by_role(Role::Button, "Link Image output 0 to Mix input 0");
    assert!(!link.is_selected());
    link.a11y(A11yAction::Select);
    assert!(link.is_selected());
    screen.get_by_role(Role::Button, "Image").a11y(A11yAction::Select);
    assert!(!link.is_selected(), "selecting a node clears the link selection");
    assert_eq!(links.get_untracked().len(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
```
（名前は `texts.rs:40-43` の既定文言 `"Link {from} output {output} to {to} input {input}"`。node の選択で link 選択が消えるのは `edit.rs:73-74`。`is_selected` は `kivact-test/src/screen/element.rs:250`。）

---

#### C5 `catalog_with_mode::overlay`（examples/kivact-ui-pop/src/pop/shell.rs）

#### 1. local・書き手・読み手
`shell.rs:92-170` `catalog_with_mode`。lint（`dev/xtask/src/state_fields/locals.rs`）が数える local:
- `selected = signal(0usize)`（:97、marker :96 `independent: the scroll-spy section is unrelated to which overlay is open`）
- `overlay = signal(Overlay::None)`（:102、OPEN）

`overlay` は `Signal<Overlay>`、`enum Overlay { None, Confirm, Form, Sheet, Command }`（`shell.rs:51-58`、旧付録 A-KP の SM-1 の形そのもの、実装済み）。書き手は `Overlay::open`（:62-67）の binding 経由のみ。`Option` ではない。

#### 2. 原因：lint の誤検出
`locals.rs` の `is_scalar` は `Expr::Path` の**最後の segment が `None`** なら `signal(None)` とみなす（`is_scalar` の Path 分岐、`is_none` も同じ判定）。`Overlay::None` という enum variant を `Option::None` と誤認し、`overlay` を「bool/Option の state local」として数えている。実体は型で閉じた 1 つの enum state。workspace で同じ形の local は他に無い（Grep `signal\(\s*[A-Z]\w*::None\s*\)`：他は `examples/kivact-ui-game/src/game/shell.rs:144` の struct field 初期化で local ではない。`RefCell::new(X::None)` は 0 件）。

#### 3. 設計（確定）：型で表す（既に enum）＋ lint の誤検出を直す
`dev/xtask/src/state_fields/locals.rs` の `is_scalar` と `is_none` を、`None`・`Option::None`・`std::option::Option::None`・`core::option::Option::None` の path だけを None とみなすように直す:

```rust
fn is_scalar(expr: &syn::Expr) -> bool {
    match strip(expr) {
        syn::Expr::Lit(literal) => matches!(
            literal.lit,
            syn::Lit::Bool(_) | syn::Lit::Int(_) | syn::Lit::Float(_)
        ),
        syn::Expr::Path(_) => is_none(expr),
        _ => false,
    }
}

/// Whether `expr` is `Option::None` (bare or qualified), not an enum variant named `None`.
fn is_none(expr: &syn::Expr) -> bool {
    let syn::Expr::Path(path) = strip(expr) else {
        return false;
    };
    let names: Vec<String> = path.path.segments.iter().map(|s| s.ident.to_string()).collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    matches!(
        names.as_slice(),
        ["None"] | ["Option", "None"] | ["std" | "core", "option", "Option", "None"]
    )
}
```

`dev/xtask/src/state_fields/tests.rs` に追加:

```rust
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

lint 修正の後、`catalog_with_mode` の state local は `selected` 1 個になり audit の対象外（audit は 2 個以上の関数だけ）。組合せが無くなるので `examples/kivact-ui-pop/src/pop/shell.rs:96` の marker 行（overlay との関係を理由にしたもの）を**削除**する。marker を消しても local 1 個では error にならない。**:97・:102 の 2 行とも audit から消える。**

順序：lint 修正 → marker 削除（同一 unit。逆順だと 2 個の無印 local で error になる）。

#### 4. 呼出元: 例の変更は marker 1 行の削除のみ。
#### 5. 振る舞い: 製品の振る舞いは不変。既存 E2E：`examples/kivact-ui-pop/tests/catalog.rs:264-301`（confirm・form dialog・drawer の開閉）。追加 E2E なし（lint は xtask の unit test で担保）。

---

### 既存欠陥

1. **alert の再表示時の stale restore と effect 順依存**（`crates/kivact-ui/src/alert.rs:359-376, 435-447`）。再現：`open(binding)` の alert を表示中に A へ focus → binding を false → 隠れている間に B へ focus → binding を true。`feed` の `effect_on` が `ExternalFocus` の effect より後に走る順だと restore は A のまま、`focused_node()` が None の場合は必ず A のまま。dismiss で既に関係のない A に focus が戻る。修正は C1。
2. **旧付録 A-UI の alert 行（SM-9「transition を純粋に」）が未実装**（`alert.rs:394` で transition が binding を `get_untracked()`）。修正は C1。
3. **`FilterDraft::Valid` の value と sent の重複**（`filters.rs:240-242`）：常に同値で、片方の `cache:` marker は実体のない理由。修正は C3（推測ではなく `Sent::update` の定義 `kivact-types/src/cell.rs:90-96` から確定）。
4. **lint `locals.rs` の `is_scalar`／`is_none` が enum variant `X::None` を `Option::None` とみなす誤検出**。修正は C5。
5. 文言のみ：`calendar_values.rs:30,32` の `independent: a measured value; any value is valid on its own` は field を問わない汎用文で、SM-11 の「全組合せが有効な理由」になっていない。修正は C2。同種の汎用文は `crates/kivact-ui-creative/src/editors/shared/region_parts.rs:26,28`（`RegionInfo.checked/selected`「recomputed from the domain for any node」）にもあり、C4 と同じく種類による排他の可能性がある（推測。本 slice の対象外なので親の付録 A 確認で扱うこと）。

---

### 6 unit 分割案（本 slice 分）

| unit | crate | 所有ファイル | 依存 | 完了時の軽い確認 |
|---|---|---|---|---|
| C-UI | kivact-ui（＋ public-api E2E 1 本、soft consumer 1 本） | `crates/kivact-ui/src/alert.rs`、`crates/kivact-ui/src/calendar_values.rs`、`crates/kivact-ui/src/data_table/filters.rs`、`dev/tests/kivact-public-api/tests/e2e/feedback.rs`、`crates/kivact-ui-soft/tests/consumer/typed_table_filters.rs` | なし | `cargo check -p kivact-ui`；`cargo clippy -p kivact-ui --all-targets -- -D warnings`；`cargo xtask check --no-write --only state-fields --crate kivact-ui`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e feedback::`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-soft --test consumer typed_table_filters::` |
| C-CR | kivact-ui-creative | `crates/kivact-ui-creative/src/editors/node/gesture.rs`、`.../node/a11y.rs`、`.../node/view.rs`、`crates/kivact-ui-creative/tests/consumer/node_editor.rs` | なし | `cargo check -p kivact-ui-creative`；`cargo clippy -p kivact-ui-creative --all-targets -- -D warnings`；`cargo xtask check --no-write --only state-fields --crate kivact-ui-creative`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-creative --test consumer node_editor::` |
| C-XT | xtask ＋ pop catalog | `dev/xtask/src/state_fields/locals.rs`、`dev/xtask/src/state_fields/tests.rs`、`examples/kivact-ui-pop/src/pop/shell.rs`（:96 の marker 削除のみ） | なし（lint 修正 → marker 削除の順で同一 unit） | `cargo test -p xtask state_fields`；`cargo clippy -p xtask --all-targets -- -D warnings`；`cargo xtask check --no-write --only state-fields --crate kivact-ui-pop-catalog` |

注意：`crates/kivact-ui-soft/tests/consumer/typed_table_filters.rs` は kivact-ui-soft lane と重なる可能性がある。親が soft crate を別 lane に割り当てる場合は、この試験追加を soft lane に移し C-UI の後に置く。C-XT の `locals.rs`／`tests.rs` は親の lint 抜け穴修正 unit と同じファイルなので、親がそちらへ統合してよい（統合する場合も shell.rs の marker 削除は lint 修正の後）。

---

## 付録 P5-D：P5 slice D：旧作業指示書 付録 A（A-RT / A-EC / A-PS / A-AR / A-LE / A-EN / A-CO）の実装確認と、lint が見ない暗黙状態

対象：HEAD 73c36c7c（= db8d679b + 文書）。差分は `git diff 79bbd409..db8d679b`。リポジトリは読取りのみ。

### 1. 行ごとの判定

判定の語：**済**（意図どおり）／**誤**（形は変えたが意図と違う）／**未**（未実装）／**対象外**（lint が数えない field なので marker は不要。型は意図どおり）。

#### A-RT（kivact-runtime）

| 行 | HEAD の位置 | 判定 | 備考 |
|---|---|---|---|
| AnimInfo running/paused → RunStatus | `motion/state.rs:85-123`。書き手 `animated.rs:238-240`、`timeline/api.rs:967-970`、`state.rs:174-178` | 済 | |
| DetailBuilder cause → CauseSlot | `reactive/detail.rs:133-150,162,212-214,325` | 済 | ただし `CauseSlot::at()` が Empty と Dropped を両方 `None` に潰し、`DetailWrite.cause`／`DetailRun.cause`（OPEN 行、他 slice 担当）に同じ二義性が再発している。他 slice へ申し送り（**本文 3.6 O5 で採らないと判断**：record の唯一の読み手 `cause_edge` は両者を区別しない） |
| Pool tx/handles → Workers | `task/worker_pool.rs:22-36,65-68,88,112,132` | 済 | |
| Queue worker_ids → next_worker_id | `task/action.rs:112,560-562` | 済（marker 不要） | `Queue` には Stored が無い（`phase` は Phase、`Generation` は helper で promote されない）ので marker を置くと unused marker になる。置かない。doc comment の「Id of the last worker spawned: a worker whose id is not current returns」は誤解を招くので「Allocates worker ids; the current worker's id lives in `Worker::Picking`/`Running`」に直す（低） |
| ActionCell last（projection） | `task/action.rs:249-251`、書き手 `:516`（Latest make）・`:586`（worker pop） | 済 | 書き手は 2 箇所のみを確認。理由文は付録 A 指定どおり |
| LoopCell rate（SM-5） | `motion/state.rs:439-449`（`rate: Value<..>` を setup で作る、`frame_loop.rs:33`） | 済 | |
| LoopRun index/last_tick → Cadence | `motion/state.rs:452-519` | 済 | |
| Runtime ext（SM-11） | `reactive/data.rs:74-78` | **誤（文言）** | 理由が「the write counter is unrelated」で、付録 A 指定の `independent: ADR-17: filled by the constructor before it returns; never observed empty` ではない。§2 F-6 |
| TaskCell task | `task/state.rs:89-96` | 対象外 | field 1 個で行が出ない |
| Runtime mount_revision/unstamped/awaiting | `reactive/data.rs:80-82` | 対象外 | `Cell<Revision>`・`RefCell<Vec>`（名前が pending 等でない）は数えられない |
| Runtime state/exit | `reactive/data.rs:84-85` | 対象外（type） | 両方 Phase。TurnState（Quiet/WakeSent/Running）と ExitState（Running/Draining/Exited）は Draining 中も turn が走るので全組合せが到達する |
| TurnReport | `reactive/host_types.rs:80-85` | 対象外 | `Report` 接尾辞で除外（mutable cell なし） |
| AsyncCell publish | `task/async_state.rs:63-94` | 済 | `latest` の書き手は `publish`（`:79`）のみ。読み手 `action.rs:678`、`resource.rs:123` |
| Frame | `reactive/data.rs:364-375` | 済（文言は付録 A どおり、弱い） | §2 F-9 で具体的な理由に差し替え |
| MemoCell value | `reactive/data.rs:173-174` | 済 | 同じ struct の `eq`（`:176-177`）の prefix が誤り（§2 F-8） |

#### A-EC（crate をまたぐもの）

| 行 | HEAD | 判定 | 備考 |
|---|---|---|---|
| TurnOutput → enum | `kivact-engine/src/contract/host.rs:25-42`、読み手 `host/pump.rs:431-434` | 済 | |
| Routed target/region/captured | `contract/routed.rs:7-31` | 済 | `captured` は削除済み（`contract/`・`kivact-test/src` に残存なし） |
| ImeStateInfo → enum | `contract/devtools.rs:126-143` | 済 | |
| Host close/phase（SM-11） | `host/pump.rs:52-82` | 対象外 | `CloseSignal` は数えられない。ただし同じ struct の `next_wake` に新しい暗黙状態（§2 F-7） |
| RawKey → RawKeyPhase | `arvane-shell/src/pipeline/raw.rs:11-22` | 済 | |

#### A-PS（kivact-core）

| 行 | HEAD | 判定 | 備考 |
|---|---|---|---|
| PartState focused/focus_visible → Focus | `view/part_data.rs:49-92`、書き手 `view/part.rs:78-88`、`kivact-ui/src/combobox/render.rs:68`、`tag_input.rs:222` | 済（focus） / **誤（残り 4 field の marker）** | 残る `pressed`/`hovered`/`disabled`/`hidden` に `independent: a public observation snapshot; any combination can be observed` が付いたが、同じ struct の doc（`:64-69`）と唯一の生成元 `part.rs:79-80` は「disabled か hidden の間は pressed/hovered は常に false」。偽の independent。§2 F-1（最重要） |

#### A-AR（arvane-*）

| 行 | HEAD | 判定 | 備考 |
|---|---|---|---|
| VirtualWindow 各 field | `arvane-virtual/src/window.rs:139-186` | 済 | soft_keyboard は `soft_keyboard_shown`（independent）／`soft_keyboard_requests`（statistic）に分割済み（`:163-168,425-428`）。`failing: u32` は数えられない |
| x11 Notifier → UiThread | `arvane-native/src/port/x11/notifier.rs:13-16` | 済（Linux、未ビルド確認） | |
| XimConnection Connection/Context/Keys | `arvane-native/src/port/x11/ime.rs:74-140` | 済（Linux、未ビルド確認） | |
| ImeDriver → ImeOffer | `arvane-shell/src/pipeline/ime.rs:13-31` | 済 | |
| asset Entry generation（identity） | `arvane-asset/src/store/entry.rs:50-51` | 済 | 同じファイルの `VariantKey` に新しい暗黙状態（§2 F-3） |
| AssetStore frame → FrameClock | `arvane-asset/src/store/asset_store.rs:42-69,104` | 済 | |
| asset Worker stop | `arvane-asset/src/store/worker.rs:73-77` | 済（構造変更あり、marker 不要） | thread と queue は `WorkerThread` enum に移った。Stored は `stop` だけ → 行が出ない |
| Drawn → enum | `arvane/src/presenter/frame_policy.rs:18-40` | 済 | `NoBuffer.renderer` は `#[expect(dead_code)]`。GPU は P6 担当 |
| CacheIndex → Records | `arvane-compose/src/cache.rs:92-111` | 済 | |
| Derived context | `arvane-render/src/derived.rs:47-55` | 対象外 | Stored は `context` 1 個で行が出ない |

#### A-LE

| 行 | HEAD | 判定 | 備考 |
|---|---|---|---|
| LayoutState changed/relayout → Pending | `kivact-layout/src/pass_state.rs:25-57`、`layout.rs:72-76`（root を先に見る順序に修正済み） | 済 | `epoch` は `Epoch` 型で数えられない（marker 不要） |
| NodeStyle display → NodeDisplay | `kivact-layout/src/style/node.rs:35-43,82` | 済 | `grid_area`（`:111`）は OPEN 行で他 slice 担当 |
| EditState base/log | `kivact-editing/src/edit_state.rs:46-66` | 型は済（`log: Option<Log>`、`Log { base, entries }`）。**OPEN 行（`log` に marker 無し）は slice A 担当** | |
| Agent listener → Listener | `kivact-devtools-agent/src/agent.rs:30-50` | 済 | |

#### A-EN（kivact-engine）

| 行 | HEAD | 判定 | 備考 |
|---|---|---|---|
| Place sticky/fixed → Positioning | `tree/node.rs:141-161,209-210` | 済 | |
| Editing/Children first/last → ends | `tree/child_order.rs:20,37-94,174-190` | 済 | |
| HostWindow last_stats/last_present | `host/window.rs:48,65-68` | 済 | 同じ struct の `environment` の projection marker が偽（§2 F-2） |
| Step own/grouped/group → Grouping | `paint/state.rs:193-231` | 済（grouping） | 同じ struct の他 field の `projection:` marker が誤用（§2 F-4） |
| Kept run_id/run → Run | `text_input/records.rs:70-94` | 済 | |
| MeasureColumns → LeafMeasure | `measure/state.rs:19-70,127`、`model/edit/inputs.rs:38-41`（editor 設置時に `set_measure(.., installed)` で None） | 済 | |
| MouseTrack at → Point | `model/pointer/state.rs:92-99` | 済 | `TouchTrack.hover`/`PenTrack.hover`（OPEN）は他 slice |
| ArvaneAssets/Walk/EventQueue counters | `lower/assets.rs:50-54`（identity）、`paint/list/walk.rs`、`model/events.rs:17-18` | 済 | Walk・EventQueue は数えられず marker 不要 |

#### A-CO（kivact-core・kivact-test）

| 行 | HEAD | 判定 | 備考 |
|---|---|---|---|
| NodeObservation rect/layout_rect → Measured | `view/tree.rs:264-275`、書き手 `client/dispatch/env.rs:118-123` | 済 | `measured` の marker 文言「any value is valid on its own」は弱い（§2 F-8 の一括修正に含める） |
| SessionData highlight/pick → Inspector | `view/devtools_state.rs:66`、`view/inspect_mode.rs:14-33` | 済 | |
| SharedImage current/pixels | `gpu_interop/share/app_side.rs:31-35`。`current` の書き手は `renew`（`:177-186`）と `import`（`:202-217`）だけ | 済 | |
| FocusKeys node/visible → KeyFocus | `view/state.rs:152-165` | 済 | |
| part_pointer Delivery/FinishReached | `view/mount/part_pointer.rs:236-247,331-338,432-464` | 済 | |
| ViewState initial_render | `view/state.rs:238-239`。初期値 `true`、書き手は `launch/runtime.rs:253` の `set(false)` だけ | 済（文言が逆） | 理由「the first frame was drawn」は値の意味と逆。`independent: one-way latch: true until the first frame is drawn (launch::runtime sets it false once)` に直す（低） |

確認の範囲：全行について HEAD のコードを読んだ。Linux 専用（x11）の 2 行はビルドを確かめていない（読んだだけ）。

### 2. 新しく見つけた暗黙状態・誤った marker（重要度順）

背景：差分で本 slice の crate の state-fields marker はほぼ全部（runtime 31、engine 142、core 113、arvane-text 67 など計約 470 本）が SL-8 の接頭辞に書き換えられた。多くは旧理由に `cache:`・`independent:` を機械的に付けたもの（例 `arvane-asset/src/store/entry.rs` の `cache key - …` → `cache: …`）。全数は見ていない。以下は抜取り（projection 16 本は全数、independent 123 本は一覧を読んで疑わしいものを開いた）で確定したもの。

#### F-1（高）PartState の偽の independent（kivact-core、公開型）

- 事実：`crates/kivact-core/src/view/part_data.rs:62-80`。`pressed`・`hovered`・`disabled`・`hidden` に `independent: a public observation snapshot; any combination can be observed`。doc は「Always `false` while the target is disabled or hidden」（`:64-69`）。生成元 `view/part.rs:79-80` は `pressed: !disabled && !hidden && …`、`hovered: !disabled && !hidden && …`。kit の生成元（`kivact-ui/src/combobox/render.rs:68-76`、`tag_input.rs:222-230`）は `..PartState::default()` で pressed/hovered は false。→ (pressed=true, disabled=true) 等は作られない。
- 閉じ方：**型で表す**。
  ```rust
  // crates/kivact-core/src/view/part_data.rs
  /// Whether a target takes input, and what input holds it when it does.
  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
  pub enum Availability {
      /// The target takes input.
      Enabled { pressed: bool, hovered: bool },
      /// A caller or attached behavior disables the target.
      Disabled,
      /// A caller or attached behavior hides the target (wins over Disabled:
      /// a hidden target is not drawn).
      Hidden,
  }
  impl Default for Availability → Enabled { pressed: false, hovered: false }（derive の #[default] は struct variant に付けられないので手書き）

  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
  pub struct PartState {
      pub focus: Focus,
      pub availability: Availability,
  }
  impl PartState {
      pub fn is_focused(&self) -> bool { self.focus != Focus::None }
      pub fn is_focus_visible(&self) -> bool { self.focus == Focus::FocusedVisible }
      pub fn is_pressed(&self) -> bool { matches!(self.availability, Availability::Enabled { pressed: true, .. }) }
      pub fn is_hovered(&self) -> bool { matches!(self.availability, Availability::Enabled { hovered: true, .. }) }
      pub fn is_disabled(&self) -> bool { self.availability == Availability::Disabled }
      pub fn is_hidden(&self) -> bool { self.availability == Availability::Hidden }
  }
  ```
  4 本の marker は消す（enum `Availability` は phase 名でないので数えられず、Stored が 0 になり行も出ない）。
- 書き手の書換え：`view/part.rs:78-88` を
  ```rust
  state(PartState {
      availability: if hidden { Availability::Hidden }
          else if disabled { Availability::Disabled }
          else { Availability::Enabled { pressed: input.press != Press::Idle, hovered: input.hover == Hover::Inside } },
      focus: /* 今のまま */,
  })
  ```
  `kivact-ui/src/combobox/render.rs:68-76` は `availability: if disabled.get() { Availability::Disabled } else { Availability::default() }`、`tag_input.rs:222-230` は変更不要（`..PartState::default()`）。再 export は `crates/kivact-core/src/view.rs:94` の `pub use part_data::Focus;` を `pub use part_data::{Availability, Focus};` にする（facade は `kivact_core` を再 export するので他に変更なし）。**書き手の追加（P5-F 3 章）**：`crates/kivact-ui-soft/src/indicator.rs:117-120` は PartState の field を直接書く。P5-F 3 章のコードに置き換える。
- 読み手：`state.pressed`／`.hovered`／`.disabled`／`.hidden` の読みを accessor に。手順は「field を上記の形に変える → `cargo check --workspace --all-targets` の E0609/E0560 を全部 accessor に直す」。`.pressed` を含むファイルは 64（他の型の field も含む）なので sed 一括はしない。kit（kivact-ui・ui-basic・ui-pop・ui-game・ui-creative・ui-soft）と examples・docs の README のコード片も対象。
- 振る舞い：変わらない（同じ値を同じ条件で作る）。既存 E2E（button の pressed/hovered/disabled の look を見るもの、`dev/tests/kivact-public-api/tests/e2e/` の part/recipe 系）がそのまま通ることで担保。追加 E2E は不要。
- **人間の判断**：公開 API の変更（kit 全体に波及）。代替は無い（真の理由を書ける independent は存在しない）。承認されない場合でも偽の marker は残せないので、最低限 doc の不変条件を理由に書いた `projection: written only by kivact_core::view::part (pressed/hovered are false while disabled or hidden)` は**不可**（kit にも生成元がある）。よって型で表す案を推奨。

#### F-2（中）HostWindow.environment の projection marker が偽（kivact-engine）

- 事実：`crates/kivact-engine/src/host/window.rs:49` は `projection: … written only by Host::environment_changed`。実際の書き手は `host/window_changes.rs:78-82`（`start_window`、native をそのまま）と `:134-136`（`environment_changed`、`fonts` を差し替えた値）の 2 つ。
- 閉じ方：**単一の書き手の投影**。`host/window_changes.rs` に
  ```rust
  impl Host {
      /// Record `environment` as window `id`'s last read environment
      /// (the only writer of `HostWindow::environment`).
      fn record_environment(&mut self, id: WindowId, environment: Environment) {
          if let Some(host_window) = self.windows.get_mut(&id) {
              host_window.environment.update(environment);
          }
      }
  }
  ```
  `start_window` の `:78-82` を `self.record_environment(id, native.clone());`、`environment_changed` の `:134-136` を `self.record_environment(id, now.clone());` に替える。marker を `// xtask-allow(state-fields): projection: written only by Host::record_environment` に。
- 振る舞い：不変。既存 E2E（環境変化・テーマ変更）で担保。

#### F-3（中）arvane-asset VariantKey：SVG 専用の size/crop と raster 専用の frame が同居（arvane-asset）

- 事実：`crates/arvane-asset/src/store/entry.rs:124-147`。`size: Option<PixelSize>`（SVG だけ）、`crop: Option<Crop>`（size があるときだけ意味を持つ。`region()` は size が None なら crop を読まない）、`frame: u32`（「0 for stills and SVGs」）。marker 3 本は `cache:`（差分で `cache key - …` に接頭辞を付けただけ）。(size=None, crop=Some) と (size=Some, frame≠0) は作られない（生成元 `asset_store.rs:280-285,293-298,522-527,681-684` を確認）。
- 閉じ方：**型で表す**。
  ```rust
  /// What a derived raster is made from.
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
  pub(crate) enum VariantSource {
      /// Frame `0` of a still, or frame `n` of an animation, of a raster source.
      Frame(u32),
      /// An SVG at a device size, or the part `crop` of it.
      Svg { size: PixelSize, crop: Option<Crop> },
  }
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
  pub(crate) struct VariantKey {
      pub(crate) source: VariantSource,
      /// The tint of the copy (`None`: untinted).
      pub(crate) tint: Option<Tint>,
  }
  impl VariantKey {
      pub(crate) fn region(&self) -> Region {
          match self.source {
              VariantSource::Svg { size, crop } => Crop::region(crop, size),
              VariantSource::Frame(_) => Region::WHOLE,
          }
      }
      /// Same picture apart from tint and animation frame (`trim_tints` counts these together).
      pub(crate) fn same_shape(&self, other: &Self) -> bool {
          match (self.source, other.source) {
              (VariantSource::Frame(_), VariantSource::Frame(_)) => true,
              (VariantSource::Svg { size: a, crop: c }, VariantSource::Svg { size: b, crop: d }) => a == b && c == d,
              _ => false,
          }
      }
      fn is_whole(&self) -> bool { !matches!(self.source, VariantSource::Svg { crop: Some(_), .. }) }
  }
  ```
  marker は全部消す（Stored は `tint` 1 個になり行が出ない。marker を置くと unused marker）。
- 書換え：`asset_store.rs:280-285` → `VariantKey { source: VariantSource::Frame(index), tint: None }`；`:293-298`・`:522-527` → `VariantKey { source: VariantSource::Svg { size, crop }, tint: None }`；`:428-435` の `crop: key.crop` → `start_raster` の引数で受けた `crop` をそのまま使う（`let VariantSource::Svg { crop, .. } = key.source else { unreachable!("invariant: start_raster is called for SVG keys") }` は使わず、呼出元 `:300` 付近から `crop` を渡す形に署名を `start_raster(asset, svg, key, size, crop, now)` に変える）；`:716` の `(!whole || k.crop.is_none())` → `(!whole || k.is_whole())`；`evict.rs:126-128` の `key.size == base.size && key.crop == base.crop` → `key.same_shape(&base)`。他に `.size`/`.frame`/`.crop` を読む箇所は `cargo check -p arvane-asset` の error で全部出る。
- 振る舞い：不変。試験は `cargo test -p arvane-asset`（store の unit test）と、画像/SVG の既存 E2E（`dev/tests/kivact-public-api/tests/e2e/` の image・svg 系）。

#### F-4（中）paint Step：`clips`/`rounded` の暗黙の 3 値と `projection:` の誤用（kivact-engine）

- 事実：`crates/kivact-engine/src/paint/state.rs:206-240`。`transform`・`clips`・`presence_clip`・`rounded`・`layer` に `projection: readonly snapshot … written only by the paint list walk`。`Step` は値で、生成は `paint/list/walk.rs:159-171` の 1 箇所。投影（同じ事実の二重保持）ではないので prefix の誤用。`rounded` は `clips` が true のときだけ読まれる（`state.rs:261-266`）。(clips=false, rounded=true) は意味を持たない。
- 閉じ方：
  1. `clips`+`rounded` を**型で表す**：
     ```rust
     #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
     pub(crate) enum ClipShape { #[default] None, Rect, Rounded }
     ```
     `Step.clip_shape: ClipShape`。`walk.rs:155-171` で `clip_shape: match (n.clips(), look.rounded) { (false, _) => ClipShape::None, (true, false) => ClipShape::Rect, (true, true) => ClipShape::Rounded }`。`opened()` は `match self.clip_shape { ClipShape::Rect => out.push(Opening::Clip), ClipShape::Rounded => out.push(Opening::ClipGroup), ClipShape::None => {} }`。`walk.rs` の `child_clip(.., clips, ..)` には `n.clips()` をそのまま渡す（局所変数 `clips` を残す）。他の `step.clips`／`step.rounded` 読みは compile error で全部出す。
  2. 残る `transform`/`presence_clip`/`layer`（Option）の marker は `option:` に直す：`option: absent means the node is drawn without its own transform`（transform）、`option: absent means no presence clip applies`（presence_clip）、`option: absent means the node is drawn in its parent's layer`（layer）。`own` の independent は付録 A どおり残す。
- 振る舞い：不変。既存の clip/rounded の描画 E2E・engine の paint unit test で担保。

#### F-5（中）lint が prefix の意味を検査しないことによる誤った prefix（複数 crate）

確定したもの（すべて文言の差し替えのみ、型は変えない）：

| file:line | 今 | 直す |
|---|---|---|
| `crates/kivact-engine/src/contract/host.rs:104` | `independent: the generation of the export request; …` | `identity: the export request's generation; the window answers with it and stale answers are dropped` |
| `crates/kivact-engine/src/contract/host.rs:114` | `independent: the generation of the placement request; …` | `identity: the placement request's generation; the window answers with it and stale answers are dropped` |
| `crates/kivact-core/src/view/tree.rs:298,306,311,314`（`CoreNode.at`/`handlers`/`observation`/`presence`） | `independent: a per-node resource; created only when the node has one` | 順に `option: absent means the node takes its nearest located ancestor's location`、`option: absent means the node has no handlers`、`option: absent means no node reference observes the node`、`option: absent means no reference observes the node's lifetime` |
| `crates/kivact-engine/src/model/lifecycle.rs:61,63`（`RemovedPayload`） | `independent: a value held only to be dropped …` | `option: absent means the node had no editor`／`option: absent means the node had no text extents` |
| `crates/kivact-runtime/src/reactive/data.rs:176`（`MemoCell.eq`） | `cache: part of the memo's cache entry …` | `option: absent means every recomputation notifies (no equality check)`（`graph.rs:51` が `None` をそう扱うことを確認） |
| `crates/kivact-runtime/src/reactive/data.rs:69`（`Runtime.seq`） | `statistic: the effect creation-order ID counter …` | `identity: allocates effect creation-order ids` |
| `crates/kivact-runtime/src/reactive/data.rs:71`（`Runtime.write_epoch`） | `independent: … valid at any value beside the extension set` | `identity: the write generation stamped on each notified source (SourceNode::changed_at) and compared by effects (graph.rs:560-621)` |

#### F-6（低）Runtime.ext の理由が付録 A と違う

`crates/kivact-runtime/src/reactive/data.rs:74` を `// xtask-allow(state-fields): independent: ADR-17: filled by the constructor before it returns; never observed empty` に（付録 A A-RT の指定文言）。

#### F-7（低）Host.next_wake は Exit 後に古い値を保持（kivact-engine）

- 事実：`crates/kivact-engine/src/host/pump.rs:61-62`（`independent: … valid in every host phase`）。書き手は `:432`（`Continue`）と `launch.rs:148`（初期 None）のみ。`TurnOutput::Exit` で `phase = Exiting` にしても `next_wake` は残る。読み手 `:793` は Exiting でも読む可能性がある（Exiting の return は shell に exit を求めるので実害は推測では無い）。「valid in every host phase」は事実と異なる（Exiting/Closed では意味が無い）。
- 閉じ方：**型で表す**。`HostPhase::Running { next_wake: Option<u64> }` に移す（`Starting` も `{ next_wake }` を持つ）。書き手 `:432` は `if let HostPhase::Starting { next_wake } | HostPhase::Running { next_wake } = &mut self.phase { *next_wake = next_wake_ns }`、読み手 `:793` は `match self.phase { Starting { next_wake } | Running { next_wake } => next_wake, _ => None }`。`HostPhase` の `==` 比較箇所（`self.phase == HostPhase::Closed` 等）は `matches!` に替える。全箇所は `cargo check -p kivact-engine` で出る。
- 振る舞い：Exiting 以降に timer wake を求めなくなる（推測：今も実質無い）。既存の exit 系 E2E（close→drain→exit）で担保。優先度は低いので V2 で時間が無ければ marker の文言を `independent: the client's next turn deadline; read only while the host runs turns` に直すだけでもよい——ただし方針 4 に照らすと型で表す案を採る。

#### F-8（低）弱い independent 文言（型は正しいが理由が「別の field」型）

`crates/kivact-core/src/view/tree.rs:265,267`（NodeObservation の `measured`/`text_truncated`）→ `measured` は `option:` に（`option: absent until the node's first layout`）、`text_truncated` は `independent: set from every layout report, including before any measurement is published`。他にも `kivact-test/src/screen/*_tests.rs` の「test slot … written once」（SM-5 型の遅延 slot）や `arvane-virtual` の定型文があるが、試験・仮想 backend なので V2 では対象外とし、lint 側の改善（P5 本体の xtask 節）に委ねる。

#### F-9（低）Frame の理由を具体化（kivact-runtime）

`crates/kivact-runtime/src/reactive/data.rs:366-373` の 4 本を次に：
- owner：`independent: callback_frame and handler_frame replace the owner under any frame kind (frame.rs:79-104)`
- pure：`independent: pure_section sets it under any frame kind and owner (frame.rs:72-77)`
- writer：`independent: inherited from the enclosing frame except in a non-user effect's own frame (frame.rs:55-67)`
- tracking：`independent: untracked_frame and callback_frame clear it under any kind (frame.rs:79-91)`

### 3. 他 slice への申し送り（本 slice では設計しない）

- `DetailWrite.cause`／`DetailRun.cause`（OPEN）：`CauseSlot::at()` が Empty と Dropped を同じ `None` にする。record 側も `CauseSlot` 相当（`Option<u32>` でなく `CauseRef { None, At(u32), Dropped }`）にすべき。 **→ 本文 3.6 O5 で採らない（B6/B7 の independent で閉じる）。**
- `EditState.log`（OPEN）：型は付録 A どおり。残りは閉じ方の確定のみ。
- GPU 関連の新 field（`arvane/src/env.rs` の `forced: Option<RendererChoice>`／`prefer_gpu: bool`／`renderer`）は P6 担当。

### 4. unit 分割案（slice D の修正分）

| unit | crate / 所有ファイル | 内容 | 依存 | 確認 |
|---|---|---|---|---|
| D-1 | kivact-core：`view/part_data.rs`、`view/part.rs`、`crates/kivact/src`（re-export の 1 ファイル）、および compile error で出る全読み手（kivact-ui・ui-basic・ui-pop・ui-game・ui-creative・ui-soft・examples・docs） | F-1（**人間の承認後**） | なし。ただし kit を所有する他 unit と衝突するので、kit 側の読み手修正はこの unit が全部持ち、他 unit は PartState の読みに触れない | `cargo check --workspace --all-targets`、`cargo clippy -p kivact-core --all-targets -- -D warnings`、`cargo xtask check --no-write --only state-fields --crate kivact-core` |
| D-2 | kivact-engine：`host/window.rs`、`host/window_changes.rs`、`host/pump.rs`、`host/launch.rs`、`paint/state.rs`、`paint/list/walk.rs`、`contract/host.rs`、`model/lifecycle.rs` | F-2、F-4、F-5（engine 分）、F-7 | なし | `cargo check -p kivact-engine --all-targets`、`cargo clippy -p kivact-engine --all-targets -- -D warnings`、`cargo test -p kivact-engine paint`、`cargo xtask check --no-write --only state-fields --crate kivact-engine` |
| D-3 | arvane-asset：`store/entry.rs`、`store/asset_store.rs`、`store/evict.rs` | F-3 | なし | `cargo check -p arvane-asset --all-targets`、`cargo test -p arvane-asset store`、`cargo xtask check --no-write --only state-fields --crate arvane-asset` |
| D-4 | kivact-runtime：`reactive/data.rs`、`task/action.rs`（doc comment のみ） | F-5（runtime 分）、F-6、F-9、Queue の doc | なし | `cargo check -p kivact-runtime`、`cargo xtask check --no-write --only state-fields --crate kivact-runtime` |
| D-5 | kivact-core：`view/tree.rs`、`view/state.rs`（initial_render の文言） | F-5（core 分）、F-8、A-CO initial_render | D-1 と同じ crate だがファイルは重ならない | `cargo check -p kivact-core`、`cargo xtask check --no-write --only state-fields --crate kivact-core` |

checkpoint：`cargo xtask check --no-write --only state-fields --report-audit` で OPEN が増えていないこと（生成物は `git checkout -- dev/xtask/state-audit.md` で戻す）、既存 E2E 一括。

---

## 付録 P5-E：P5 slice E：旧付録 A（A-IN / A-UI / A-DT / A-KB / A-KP / A-KG / A-KC / A-KS）の実装確認と lint が見ない暗黙状態

対象：`SLIM_FAST_WORK_ORDER.md` 付録 A の上記 8 節の全行、HEAD 73c36c7c（= db8d679b）。差分 `git diff 79bbd409..db8d679b` の kit 系 crate（kivact-interaction、kivact-ui、kivact-ui-basic/pop/game/creative/soft、kivact-devtools、examples/*）。読取りのみ。

確認の深さ：SM-9 / SM-8 / SM-7 / SM-10 の行はすべてコードを読んで確認した。SM-1〜4 / SM-11 / SM-12 の行も型定義と主な書き手を確認した。A-DT の「data の panel」6 件は animations・input・performance・accessibility だけを読み、tasks・theme・styles は `signal(None` が残っていないことを grep で確認しただけ（抜取り）。

### 1. 事実：行ごとの判定

判定：**済**＝意図どおり実装済み、**誤**＝形は変えたが意図と違う、または新しい欠陥がある、**未**＝未実装、**他**＝27 OPEN の行と重なる（P5 本体の担当なので一行だけ書く）。

#### A-IN（kivact-interaction）

| 行 | 判定 | 根拠（HEAD の file:line） |
|---|---|---|
| combobox SM-9 | 済 | `combobox/handle.rs:385-401`：初期値は `send(OpenRequested(open.get_untracked()))`、その後は `feed(open, OpenRequested)`、`FocusLeft{transferred}`。`machine.rs:117-146` が `WriteOpen` を出す。effect 側は `binding.get_untracked() != open` で守っている（`handle.rs:173-177`）。旧 `:386` の補修 effect は無い |
| palette SM-8 | 済 | `palette/handle.rs:334-336`。`combobox.follow_open(open)` が close の遷移で reset する。`effect_on(open)` は無い |
| menu watch_item SM-8 | 済 | `menu/level.rs:271-278`：一つの `effect` が `Event::Pointer { hovered, press: PressOver }` を送る |
| zoom_pan Space SM-1 | 済 | `zoom_pan/machine.rs:9-13,94-130`：`State::Idle { space }`、`Event::Space(bool)`。`handle.rs:65` の `armed` は memo |
| tooltip SM-7 | 済 | `tooltip/machine.rs:12-15,40-52`：`Warm { excluded }` と `.after(within, Event::Cool)`。`after` は同じ variant の timer を置き換える（`kivact-runtime/src/machine.rs:89-92`） |
| typeahead SM-7 | 済 | `list/machine.rs:100-107`、`select/machine.rs:127-130`、`menu/machine.rs:228,255-256` が `.after(TYPEAHEAD_WINDOW, Expire)` を使う。`list/typeahead.rs` に時刻は無い |
| inline_edit SM-8 | **誤（軽）** | `inline_edit.rs:66-73`。`reported` は Machine に入ったが、常に `phase.key()` と等しい（後述 E-4）。保持せずに導出できる。marker の関数名「the machine's request transition」は実在しない。`Ended`（`:158-163`）も書く。SM-9(3) の書き戻しの防御も無い（`:194`） |
| select track_preview | 他 | `select/machine.rs:23` `State::Open.previewed` は OPEN |
| calendar DaySelection | 他 | `CalendarDayVisual.mark` は OPEN（kivact-ui 側） |
| table HeaderSort SM-2 | 済 | `table/data.rs:85-103` |
| popup HostFocus SM-2 | 済 | `popup/machine.rs:81-107`、書き手は `popup/api.rs:150-156` だけ |
| text_edit next_control_id SM-11 | 済 | `text_edit/handle.rs:505-506`、marker は `identity:` |
| ListApi の 4 field SM-11 | 済（形が変わって lint の対象外） | `list/state.rs:56` の `typed: Cell<Binding<Typeahead>>`（Machine の投影の binding。select・menu が差し替える slot）、`pending` は MachineHandle、`roving.requested` は `roving/registry.rs:68`、anchor は options の binding |

#### A-UI（kivact-ui）

| 行 | 判定 | 根拠 |
|---|---|---|
| selection_field SM-9 | 済 | `selection_field.rs:29-103,172-202`：`FieldState { Closed{gesture}, Open{surface} }`、初期値は `get_untracked`、`feed`、`WriteOpen` を守って書く |
| date_input trigger の経路 | 済 | `date_input/view.rs:239`。両方の経路で `date_controller` を使う |
| date_input の単一 Machine SM-8 | 済（軽い注記あり） | `date_input/machine.rs:45-60` `DateField { typed, picker }`。ただし外部値は `effect(move \|\| state.send(Event::External(value.get())))`（`view.rs:179`）で入れている。`feed` の形ではない（E-8） |
| date_range picker SM-9 | 済（同上の注記） | `date_range_draft.rs:25-59`、`date_range_input.rs:429` も effect |
| data_table filters | 他 | `FilterDraft::Valid.value` は OPEN |
| drawer_swipe SM-10 | 済 | `drawer_swipe.rs:9-67` `Swipe { Resting, Pulling(f64) }` を Start/Update/End/Cancel で動かす |
| alert BannerState | 他 | `BannerState::Shared.shown` は OPEN |
| combobox `Query::Shared` SM-9 | 済（形は違うが目的は満たす） | `kivact-interaction/src/combobox/machine.rs:5-25`。binding は state に残る。ただし `transition` は binding の値を読まず（`reset` は variant を見るだけ）、値を読むのは handle の memo（`handle.rs:182`）だけ。transition は純粋なので**直さない**と判断した |
| data_table rows drag SM-10 | 済 | `data_table/rows.rs:901-928`、`row_drag.rs:6-35`。4 つの event を送る |
| component_values SM-4 | 済 | `component_values.rs:10,28,41-70` `Expansion`・`Selectable` |
| data_table Paging SM-4 | 済（context）、builder に残りあり | `data_table/types.rs:37-54,79`。builder（`types.rs:139-151`）の `page`・`has_more`・`page_sizes` は別々の Option のまま（E-9、軽） |
| SwitchDragState SM-1 | 済 | `values.rs:249-290` |
| ChatComposerState SM-11 | 済 | `chat_values.rs:100-105`。付録の文言のまま |
| hover_card CardOpen SM-4 | 済 | `hover_card.rs:26,42,93-99,131-132` |
| CodeBlockState SM-11 | 済（理由が弱い） | `code_values.rs:9-12`。付録の文言そのもの（「separate parts」は「別の field」型の理由。E-10 で文言を直す） |
| button の memo MEM-6 | 範囲外 | メモリ担当 |

#### A-DT（kivact-devtools）

| 行 | 判定 | 根拠 |
|---|---|---|
| accessibility A11yData SM-3 | 済 | `panels/accessibility.rs:14-30`、`resource(..)` |
| review CommentEdit SM-2 | 済 | `panels/review.rs:10-20,52-72,128-140` |
| override 行 SM-8 | 済 | `widgets/override_row.rs:11-128` `Override { Off{value}, On{id,value} }`。`Toggle(true)` 後の `id: None` は、`set_token` が reload し、`dynamic` が行を作り直すときに新しい id を受け取る（`panels/theme.rs:21-63,77-`）。effect は無い |
| scripts Runner SM-2 | 済 | `panels/scripts.rs:28-56,194-241` `ScriptRun::Running { script, child: Arc<Mutex<Child>> }` |
| session SM-11 | 済 | `session.rs:31-33`（status に marker）。panel・popped は counted 型ではない |
| data の panel SM-4 | 済（抜取り） | animations `:11`、input `:17`、performance `:27` が `resource`。tasks・theme・styles は `signal(None` が残っていない（`elements/styles.rs:117` の `signal(Some(Scope::Node))` は select の binding） |
| elements view・virtual_tree | 済 | highlight は `layout_toggle` の setter が送る（`elements/view.rs:25-32`、effect は無い）。`virtual_tree.rs:21` に `independent: one pending scroll request` |

#### A-KB・A-KP・A-KG

| 行 | 判定 | 根拠 |
|---|---|---|
| KB chart hover SM-2 | 済 | `kivact-ui-basic/src/chart/view.rs:194-215` |
| KB catalog dialog SM-11 | 済 | `examples/kivact-ui-basic/src/docs/dialog.rs:122-125` ほか |
| KP shell / patterns の overlay | 他 | `examples/kivact-ui-pop/src/pop/shell.rs:102` は OPEN。`patterns.rs:55-60` は今も 4 つの `Binding<bool>` を受ける。shell 側を直すときに一緒に直す |
| KP busy・verify・loading SM-7 | 済 | `components.rs:769-790` `SyncPhase` の Machine。upload の進捗（`:425-436`）は値の animation で、bool の flag ではない |
| KP 返信 SM-6 | 済 | `messaging.rs:31-73` `Reply { Idle, Waiting{id}, Typing{id} }` と `after` |
| KG halo SM-8 | 済 | `kivact-ui-game/src/surface.rs:28-44,102-121`：Machine の effect が clock を start/stop し、`feed(running)` で入れる |
| KG Overlay SM-1 | 済 | `examples/kivact-ui-game/src/game/state.rs:6-40` |
| KG Editing SM-8 | 済 | `game/system.rs:19-65`。title・confirmation は `editing(..)` で Machine から導く |
| KG busy SM-7 | 済 | `game/controls.rs:25-52` `Task` Machine と `Expire` |
| KG window `Run` SM-8 | **誤（退行）** | `game/window.rs:10-31,41,63-64,236-239`。後述 E-3 |

#### A-KC（kivact-ui-creative と catalog）

| 行 | 判定 | 根拠 |
|---|---|---|
| NodeController の意味 event | 済 | `editors/node/gesture.rs:86-108,180-230`。`Choose` と `update` は消えている |
| DecisionMachine の Press | 済 | `editors/shared/gesture.rs:22-130` |
| editor の active → TargetState | 済（推測の注記あり） | `editors/shared/targets.rs:82-179`。`Synchronize` は `active` を新しい table と照合しない（E-11、推測） |
| NumericMode SM-10 | 済 | `numeric/implementation.rs:431-446`（mode Machine。typing は inline_edit の投影） |
| dock DockGesture SM-4 | 済 | `dock/data.rs:22-28`、`dock/view.rs:147-156`。rename は `gesture` の投影の binding で、正本は `gesture` 一つ |
| dock next_rect / outliner generation | 済 | `dock/data.rs:35`、`outliner/view.rs:219-220`（`identity:`） |
| outliner plan SM-10 | 済 | `outliner/view.rs:258-262,384`。drag の payload／Machine に入る |
| tabs Reorder SM-10 | 済 | `tabs.rs:95-145`。Signal に enum を持ち、drag の 4 つの callback だけが書く |
| panel_group SM-10 | 済 | `panel_group/implementation.rs:165-195` |
| picker PickerPhase SM-8 | 済 | `picker.rs:439-523,550` |
| audio TransportState SM-11 | 済 | `audio/transport.rs:14-21` |
| feedback ToolWindow SM-8 | 済 | `feedback.rs:459-523` |
| EditorFocus SM-4 | 済 | `editors/shared/model.rs:20-31` |
| ChromeDisplay SM-1 | 済 | `dock/items.rs:30` |
| LinkSource SM-4 | 済 | `editors/node/gesture.rs:262` |
| catalog Txn SM-2 | 済 | `examples/kivact-ui-creative/src/creative/txn.rs` |
| catalog jobs SM-8 | **誤（軽）** | `jobs.rs:56-99`。Machine はあるが、clock の start/stop は `effect(running)`（`:93-99`）のまま。halo と同じ形（Machine の Effect）にする（E-7） |
| audio_values SM-11 | 済 | `audio_values.rs:14-18` |
| Grade SM-1 | 済 | `editors.rs:86-100` |
| shell SM-11 と category | 済（理由が弱い） | `shell.rs:810-856`。`Signal<Category>` も実装済み。marker の文言「either value is valid」は一つの field の値域しか述べておらず、組合せの理由になっていない（E-10） |

#### A-KS（kivact-ui-soft と catalog）

| 行 | 判定 | 根拠 |
|---|---|---|
| RichText `Write { composing }` | 済 | `rich_text.rs:62-92` |
| rich_blocks Command | 済（注記あり） | `rich_blocks.rs:191-214,251-264`。純粋な behavior が `Rc<Cell<Option<Command>>>` に置き、同じ key の `on_key` が取り出して `BlockEvent::Command` を送る。値を置くのは `EditRequest::Key` の時だけ（`:374`）。毎回上書きされるので、古い値が次の key まで残る経路は無い。一つの key の dispatch の中だけで使う受渡しなので許容する |
| SidebarWidth SM-2 | **誤** | `side_navigation/shell.rs:238-245,490-560,585-596`。後述 E-5 |
| bottom_sheet SM-10 | 済 | `bottom_sheet.rs:60,331-420` |
| ToolStatus SM-2 | 済 | `soft_chat_tools.rs:49-66` |
| rows_visible SM-8 | 済 | `soft_chat_tools.rs:737-740` は memo で導く |
| DictationState SM-2 | 済 | `chat_dictation.rs:18-30` |
| lightbox SM-12 | **誤（marker の文言）** | `lightbox.rs:73,77`。後述 E-6 |
| mobile_nav SM-9 | 済 | `mobile_nav.rs:74-170`：初期値は `get_untracked`、`feed(open, Sync)`、effect は守って書く |
| ItemBranch::Parent SM-11 | 済 | `side_navigation/items.rs:430-437` |
| SideNavCx NavSurface SM-4 | 済 | `shell.rs:241-243`（`surface: ..NavSurface`） |

### 2. 原因

- 実装者は付録の「直した形」をおおむね写した。誤りは三つの型に分かれる。
  1. 付録が想定していない、caller の binding と新しい enum の二重の持ち主（SidebarWidth）。
  2. 意味を読まずに旧 marker を一括で書き換えた（`passive record - X` → `projection: readonly snapshot ... written only by its render function`）。
  3. 状態を一つにしたときの文言の写し替えで、notice の上書きを壊した（game window）。
- SL-8 は marker の接頭辞の綴りしか検査しない。そのため、`projection:` の後の関数名が実在しない、他にも書き手がいる、といった誤りが通る（lint の穴は親の lint 節で扱う。ここでは kit 側の実例を挙げる）。

### 3. 設計（確定版）：重要度順の修正

#### E-1【高・既存欠陥】ButtonState / ToggleButtonState の `icon_only` が `leading_icon` を含意する暗黙状態。toggle_group が無効な組合せを作る

- 事実：`kivact-ui/src/values.rs:129-138,172-177` の rustdoc は「icon_only = leading icon を持ち、かつ label を隠す」と書いている。書き手は次の 5 箇所で、marker の「written only by its render function」は誤り。
  - `button.rs:250`：`icon_only: leading_icon && hide_label..`
  - `toggle_button.rs:209`：同じ式
  - **`toggle_group.rs:169`：`icon_only: icon_only.get()`（leading_icon を見ない）**
  - `values.rs:155`：`ButtonState::new`
  - `kivact-ui-creative/src/recipes.rs:183`：`state.icon_only = s.icon_only`
- 再現：`toggle_group(..).icon_only(true)` に icon の無い item を入れると、`ToggleButtonState { leading_icon: false, icon_only: true }` になる。recipe（`look.rs:265`、`kivact-ui-basic/src/recipes.rs:139`、`kivact-ui-soft/src/recipes_controls.rs:222`、creative `recipes.rs:172`）は icon の無い正方形を描く。
- 閉じ方：**導出して保持しない**。保持するのは独立な事実 `label_hidden`。
- 型（`values.rs`）：

```rust
pub struct ButtonState {
    pub input: PartState,
    pub size: ControlSize,
    // xtask-allow(state-fields): independent: busy comes from the caller's busy value and is valid with any icon or label layout
    pub busy: bool,
    // xtask-allow(state-fields): option: None for a plain button, Some(open) when a popup or the caller publishes an expanded state
    pub expanded: Option<bool>,
    // xtask-allow(state-fields): independent: the leading icon, the trailing icon and the hidden label are set by separate builder calls and all eight combinations are drawn
    pub leading_icon: bool,
    // xtask-allow(state-fields): independent: the leading icon, the trailing icon and the hidden label are set by separate builder calls and all eight combinations are drawn
    pub trailing_icon: bool,
    // xtask-allow(state-fields): independent: the leading icon, the trailing icon and the hidden label are set by separate builder calls and all eight combinations are drawn
    /// The label is hidden ([`Button::hide_label`](super::Button::hide_label)); it stays the accessible name.
    pub label_hidden: bool,
}
impl ButtonState {
    /// Whether only the leading icon shows: it has one and its label is hidden.
    /// A look draws it as a square of the button's height.
    pub fn icon_only(&self) -> bool { self.leading_icon && self.label_hidden }
    pub fn new(input: PartState, size: ControlSize) -> Self {
        ButtonState { input, size, busy: false, expanded: None,
            leading_icon: false, trailing_icon: false, label_hidden: false }
    }
}
pub struct ToggleButtonState {
    pub input: PartState,
    // xtask-allow(state-fields): independent: the pressed state follows the toggle and is valid with any icon or label layout
    pub pressed: bool,
    pub size: ControlSize,
    // xtask-allow(state-fields): independent: the leading icon and the hidden label are set separately; all four combinations are drawn
    pub leading_icon: bool,
    // xtask-allow(state-fields): independent: the leading icon and the hidden label are set separately; all four combinations are drawn
    pub label_hidden: bool,
}
impl ToggleButtonState {
    pub fn icon_only(&self) -> bool { self.leading_icon && self.label_hidden }
}
```

- 書き手の変更：
  - `button.rs:250`：`label_hidden: hide_label.is_some_and(|hide| hide.get()),`
  - `toggle_button.rs:209`：`label_hidden: hide.is_some_and(|hide| hide.get()),`
  - `toggle_group.rs:169`：`label_hidden: icon_only.get(),`
  - creative `recipes.rs:183`：`state.label_hidden = s.label_hidden;`
- 読み手の変更（`state.icon_only` → `state.icon_only()`）：
  - `kivact-ui/src/look.rs:82,265`
  - `kivact-ui-basic/src/recipes.rs:139`
  - `kivact-ui-pop/src/recipes.rs:121`
  - `kivact-ui-soft/src/recipes_controls.rs:122,222`
  - `kivact-ui-creative/src/recipes.rs:172`
- 変えないもの：`kivact-ui-soft/src/segmented_control.rs:22`・`top_nav.rs` の `icon_only` は builder / look の別物。`toggle_group.rs` の builder の `.icon_only(..)`（公開 API）と文書は変えない。
- 振る舞いの変化：icon の無い item だけ、正方形でなくなる。
- E2E（追加）：`dev/tests/kivact-public-api/tests/e2e/ui_toolbar.rs` に `toggle_group_icon_only_keeps_iconless_items_out_of_the_square_look` を足す。
  - mount：`toggle_group("Tools", vec![ListItem::new("pen","Pen").with_icon("dot"), ListItem::new("text","Text")], signal(None::<&str>)).icon_only(true)`
  - 確認：`screen.get_by_role(Role::Button,"Pen").rect()` の width と height が等しい（±0.5）。`"Text"` の rect の width と height は等しくない。`screen.errors().is_empty()`

#### E-2【中】MenuItemState の `check` と `expanded` は排他（SM-4）。projection marker で隠れている

- 事実：`kivact-interaction/src/menu/state.rs:64-80`。`check: Option<MenuCheck>` は check・radio の item だけが持ち、`expanded` は submenu の item だけが持つ。entry の種類は排他なので (Some, true) は作られない。書き手は `menu/level.rs:370-376` と `menu/handle.rs:178-184`（既定値）の二つ。
- 型：

```rust
/// What kind of row a menu item is, with its kind's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItemDisplay {
    /// An ordinary item.
    Plain,
    /// A check or radio item and its checked state.
    Check(MenuCheck),
    /// A submenu item; `open` while its submenu is open.
    Submenu { open: bool },
}
pub struct MenuItemState {
    pub pressed_over: PressOver,
    pub input: PartState,
    pub display: MenuItemDisplay,
    // xtask-allow(state-fields): independent: the authored destructive mark applies to any item kind
    pub destructive: bool,
}
impl MenuItemState {
    pub fn check(&self) -> Option<MenuCheck> { match self.display { MenuItemDisplay::Check(c) => Some(c), _ => None } }
    pub fn is_expanded(&self) -> bool { matches!(self.display, MenuItemDisplay::Submenu { open: true }) }
}
```

- 書き手：
  - `level.rs:370` の `check:`・`expanded:` を次の一つに置き換える：`display: match check.get() { Some(c) => MenuItemDisplay::Check(c), None if submenu => MenuItemDisplay::Submenu { open: expanded.get() }, None => MenuItemDisplay::Plain },`
  - `handle.rs:178`：`display: MenuItemDisplay::Plain`
  - `menu.rs`・`lib.rs` の pub use に `MenuItemDisplay` を足す（kivact facade の `interaction` の re-export 一覧も同じ）。
- 読み手：
  - `kivact-ui-basic/src/recipes_menu.rs:112,154`（`s.expanded` → `s.is_expanded()`）、`:127`（`s.check.is_some()` → `s.check().is_some()`）
  - `kivact-ui-soft/src/recipes_menu.rs:170`（`s.expanded` → `s.is_expanded()`）
- 振る舞いの変化：無い。既存 E2E `interaction_menu.rs`、`apg_menu.rs`、`apg_menu_button.rs`、`apg_menubar.rs` と kit catalog の試験が担保する。

#### E-3【中・既存欠陥（退行）】game catalog の window：Run・Pause の後は status の notice が出なくなる

- 事実：`examples/kivact-ui-game/src/game/window.rs:63-64` は `run.set(Running/Paused)` だけを書く。`window_status`（`:236-239`）は `run.get().message()` が `Some` の間 `status` を読まない。`Run` は Idle に戻らない（`run.set(Run::Idle)` は無い）。そのため Run の後の Save・New・Clear の notice（`"Saved …"` 等）が永久に隠れる。旧版（79bbd409）は run・pause が `status` も書くので、最後の操作の文言が出ていた。
- 閉じ方：`run`（RUNNING/IDLE の activity）と `status`（最後の操作の notice。どの run 状態とも組み合わさる）は真に独立。状態を導く関数は activity だけにする。
- コード：
  - `Run::message` を消す。
  - `"run" => { run.set(Run::Running); status.set("Running".to_owned()); }`、`"pause" => { run.set(Run::Paused); status.set("Paused".to_owned()); }`
  - `window_status` の `:236-239` は `g::mono(move || status.get())`。
  - activity 表示が `run.get().activity()` を読む箇所はそのまま。
- E2E（追加）：`examples/kivact-ui-game/tests/catalog.rs` に `window_status_shows_the_last_command_after_run` を足す。
  - mount は既存の `mount()`。`"Run"` の tool button（`get_by_role(Role::Button, "Run")`）を click する。
  - 確認：`screen.get_by_test_id("game-window-status")` の text が `"Running"`
  - 続けて `"Save"` を click する。text は `"Saved "` で始まる。
  - 実行：`dev/tools/window-lock.sh --timeout 600 -- cargo test -p kivact-ui-game-catalog --test catalog window_status`

#### E-4【中】inline_edit の `reported` は導出できる（SM-8 の形だけ）

- 事実：`kivact-interaction/src/inline_edit.rs:66-163`。`reported` の書き手は初期化（`:190`、`= initial`）、`Request`（`:124-126`。書かない分岐は `reported == key` のとき）、`Ended`（`:162`、`None`）。phase の key と常に等しい（`Mounted`・`Typed` は key を変えない）。
- 型：`struct InlineEdit<K> { phase: Phase<K> }` にする（`reported` を消す）。単一 field になるので `Clone`・`PartialEq` の手書き impl も簡単になる（`phase` の比較だけ）。
- 遷移 `Request(key)`：

```rust
Event::Request(Some(key)) if state.key().as_ref() == Some(&key) => Step::stay(),
Event::Request(key) => {
    let write = state.key() != key;
    let next = InlineEdit { phase: key.clone().map_or(Phase::Idle, |key| Phase::Requested { key }) };
    let step = Step::to(next);
    if write { step.with(key) } else { step }
}
```

- `Ended` は `Step::to(InlineEdit { phase: Phase::Idle }).with(None)`。
- effect（`:194`）は SM-9(3) にする：`move |_, key| if editing.get_untracked() != key { editing.set(key) }`
- marker（`:68`）は消す（field が一つになる）。
- 振る舞いの変化：無い。担保する既存 E2E は `dev/tests/kivact-public-api/tests/e2e/ui_field.rs`（rename）、`interaction_edit.rs`、creative dock の rename（`kivact-ui-creative` の試験）。

#### E-5【中】soft SidebarWidth：caller の `collapsed` と `SidebarWidth::Collapsed` の二重の持ち主。variant は誰も読まない

- 事実：`kivact-ui-soft/src/side_navigation/shell.rs`。
  - `:490-507` の `SidebarWidth::expanded()` は両方の variant で幅を返す。
  - 読み手の `sidebar_width`（`:594`）と `resize_grip` の getter（`:516-520`）は、折り畳みを `collapsed` binding と `cx.rail()` で判定する。variant は読まない。
  - 外から `collapsed.set(false)` されると variant は `Collapsed` のまま残る（矛盾した保存状態）。
  - 旧版（79bbd409 `:490,498,506`）は `Signal<f64>` だった。付録の SM-2 は caller の binding が残ることを想定していなかった。
- 閉じ方：折り畳みの正本は caller の `collapsed`（ARG-4 の binding）。幅は「展開時の幅」で、どの折り畳み状態とも組み合わさる独立な事実。
- コード：
  - enum `SidebarWidth` を消す。
  - `:238`：`let expanded_width = signal(resize.as_ref().map_or(260.0, |resize| resize.default_width));`
  - `sidebar_width`・`resize_grip` の引数は `Signal<f64>`。`:594`：`expanded_width.get()`、`:520`：`expanded_width.get()`
  - setter（`:522-532`）：`size < 160.0` なら `if let Some(c) = collapsed { c.set(true) }` だけ（幅は変えない）。それ以外は `if let Some(c) = collapsed { c.set(false) }` と `expanded_width.set(size.clamp(resize.min, resize.max))`。
- 振る舞いの変化：無い（variant は読まれていなかった）。担保する既存 E2E は `crates/kivact-ui-soft/tests/consumer/navigation.rs:515` `rail_navigation_has_one_actual_panel_and_rearms_after_expansion` と soft catalog。

#### E-6【低】marker の閉じ方の誤り（kit 側の実例）

| file:line | 今 | 直した marker（行に置く文言） |
|---|---|---|
| `kivact-ui/src/data_table/types.rs:63,69,72,75`（DataTableContext の selection・filter・column_filters・typed_column_filters） | `projection: … written only by the table's mount` | `// xtask-allow(state-fields): option: the caller's binding; None when the caller gave none` |
| `kivact-ui-soft/src/lightbox.rs:73`（`index`） | `projection: written only through LightboxHandle's state` | `// xtask-allow(state-fields): option: the caller's index binding from Lightbox::index; None keeps the gallery's own index from 0` |
| `kivact-ui-soft/src/lightbox.rs:77`（`view`） | 同上 | `// xtask-allow(state-fields): option: absent keeps the gallery's own fit and zoom view; LightboxHandle::view passes a binding projected from its state`（本文 3.6 O3） |
| `kivact-interaction/src/table/data.rs:109,111,121,124`（TableRowState・TableCellState の selected・active） | `projection: … written only by the row part / cell part` | `selected`：`// xtask-allow(state-fields): independent: the row selection and the tab stop move separately; a selected row may or may not hold the tab stop`。`active`：`// xtask-allow(state-fields): independent: the tab stop moves with the arrows and the selection does not follow it; all four combinations occur` |

#### E-7【低】creative catalog jobs：clock の start/stop を Machine の Effect にする

- `examples/kivact-ui-creative/src/creative/jobs.rs:30-99`。`JobMachine` の Effect を `enum JobEffect { Clock(bool) }` にする。
  - `Start`（Idle→Running）は `.with(JobEffect::Clock(true))`
  - `Stop | Finished`（Running→Idle）は `.with(JobEffect::Clock(false))`
- `machine(JobRun::Idle, || JobMachine, move |_, JobEffect::Clock(on)| if on { clock.start(FrameRate::Hz(24.0)) } else { clock.stop() })`
- `clock` は machine より前に作る（今と同じ順序）。`:93-99` の `effect` を消す。
- 振る舞いの変化：無い。担保は `examples/kivact-ui-creative/tests/catalog.rs` の jobs の章。

#### E-8【低】date_input / date_range の外部値は `feed` で入れる（SM-9 の形に揃える）

- `kivact-ui/src/date_input/view.rs:179`：`state.feed(value.into_value(), Event::External);`
- `kivact-ui/src/date_range_input.rs:429`：`draft.feed(value.into_value(), DraftEvent::External);`
- 初期値は既に `get_untracked` から作っている（`view.rs:345`、`date_range` の初期 state）。`feed` は最初の値を配らないので、重複した初回の External が無くなるだけ。
- 担保する既存 E2E は `apg_date_picker.rs`、`interaction_calendar.rs`、`ui_field.rs`。

#### E-9【低】DataTable builder の `page`・`has_more`

- `kivact-ui/src/data_table/types.rs:139-151`、`builder.rs:292-325`。`has_more` は `.remote_page(..)` の時だけ `page` と一緒に Some になる。(page None, has_more Some) は作られない。
- 型：builder の `page: Option<PagePlan>` と `has_more: Option<Value<bool>>` を `paging: PagingPlan` にする。

```rust
enum PagingPlan { Off, Local(PagePlan), Remote { plan: PagePlan, has_more: Value<bool> } }
```

- 書き手は `builder.rs:110-111` の既定値（`PagingPlan::Off`）、`:292-293`（`Local`）、`:324-325`（`Remote`）。
- 読み手は `DataTableContext.paging` を作る箇所（`data_table/runtime.rs` 内の `Paging::` の構築。`Grep "Paging::Local\|Paging::Remote" crates/kivact-ui/src/data_table` で一か所）。`page_sizes` は Local の時だけ意味を持つので `Local(PagePlan, Option<(Binding<usize>, Value<Vec<usize>>)>)` に入れる。
- 公開 API は変わらない。担保する既存 E2E は `interaction_table.rs` と basic catalog の data table。

#### E-10【低】「別の field」型の independent 理由の書き直し（文言だけ）

| file:line | 新しい文言 |
|---|---|
| `kivact-ui/src/code_values.rs:9,11` | **本文 3.6 O4 により P5-F F-T2（CodeBlockState の型）で置き換え。この行は行わない** |
| `examples/kivact-ui-creative/src/creative/shell.rs:810-828,836-855` | 各 bool：`independent: toggled only by its own control; every combination of compact rows, sidebar and search overlay is a reachable catalog view`。index・mode：`independent: chosen by its own control and valid with any toggle state` |
| `examples/kivact-ui-game/src/game/controls.rs:64,66` | `independent: the grid toggle and the filter toggle are pressed separately and all four combinations are valid views` |

#### E-11【推測・低】creative TargetState の `active` は Synchronize で検証されない

- `editors/shared/targets.rs:113-128`。`active()` の rustdoc は「any live target or none is valid」と書くが、target が消えても `active` は残る。
- `A` と `T` は別の型で、汎用には照合できない。読み手（各 editor の gesture）が `scene` に無い active を無視しているかは確認していない。
- V2 では確認の試験だけを追加する：creative の試験で、active な item を削除した後に key 操作で panic・error が出ないこと。

### 4. lint が見ない暗黙状態：recipe の snapshot struct の marker 一括書換え

- 事実：次の struct の field に `projection: readonly snapshot of … for recipes, written only by its render function` 等が約 80 個ある。旧 marker は `passive record - X`。
  - kivact-ui：ButtonState、ToggleButtonState、CheckableState、SelectTriggerState、DialogState、DataTableRowState、DataTableCheckState、CardState、AlertState、InputGroupState、InputGroupMemberState、ChatMessageState、CodeBlockState、ItemRowState、DisclosureRecipeState、BadgeState、PaginationSummary、StepperItemState、BreadcrumbItemState、RowStatus、TreeRowRecipeState
  - kivact-interaction：MenuItemState、TableRowState、TableCellState、TreeItemState、ListItemState、CalendarDayState
  - kivact-engine：`paint/state.rs` Step
- 判定：projection（SM-12）ではない。関数名が無く、複数の書き手がいる（E-1）。この書換えは「lint の件数を減らすだけの marker」にあたる。これらは各 component の memo が毎回作る値で、保存状態ではない。ただし field 間の含意を隠すものがあり、E-1 と E-2 が実例。
- 抜取りで見つけた含意は次の三つ。
  1. ButtonState / ToggleButtonState：E-1
  2. MenuItemState：E-2
  3. **CardState**（`component_values.rs:24-36`、書き手は `card.rs:97-107`、`card_target` は `:160-198`）：`interactive ⇔ input.is_some()`、`selection != NotSelectable ⇒ interactive`。型にする：

```rust
pub enum CardAction { Plain, Press { input: PartState }, Select { input: PartState, selected: bool } }
pub struct CardState {
    pub action: CardAction,
    // xtask-allow(state-fields): independent: the caller's disabled value applies to any action
    pub disabled: bool,
    // xtask-allow(state-fields): independent: media and footer are separate optional slots; all four combinations render
    pub has_media: bool,
    // xtask-allow(state-fields): independent: media and footer are separate optional slots; all four combinations render
    pub has_footer: bool,
}
impl CardState {
    pub fn input(&self) -> Option<PartState> { match self.action { CardAction::Plain => None, CardAction::Press { input } | CardAction::Select { input, .. } => Some(input) } }
    pub fn is_interactive(&self) -> bool { !matches!(self.action, CardAction::Plain) }
    pub fn selection(&self) -> Selectable { match self.action { CardAction::Select { selected, .. } => if selected { Selectable::Selected } else { Selectable::Unselected }, _ => Selectable::NotSelectable } }
}
```

  - 型名の衝突：builder 側の private enum `CardAction`（`card.rs`）がある。公開する方を `CardActionState` とする。
  - 書き手：`card.rs:97-107` は `card_target` が返す input（`Option<Value<PartState>>`）と action から作る。`card_target` の戻り値を `CardActionInput { Plain, Press(Value<PartState>), Select(Value<PartState>, Binding<bool>) }` にし、`state` の closure の中で `get()` する。
  - 読み手：`kivact-ui/src/look_feedback.rs:91,103`、`kivact-ui-soft/src/card.rs:164,175,182,191`（`s.selection` → `s.selection()`、`s.interactive` → `s.is_interactive()`）。`kivact-ui-basic/src/card.rs` は `has_media`・`has_footer` だけを読むので変更しない。
  - 担保する既存 E2E は `components.rs`（card）と各 kit catalog。

- 方針（確定）：
  1. 上の 3 struct は型にする（E-1、E-2、CardState）。
  2. 残りの snapshot struct の marker は、`projection:` を `independent:` に書き換える。理由には、その field を決める caller の入力と、他の field と組み合わせても描画が成立することを書く。例：`independent: set by the caller's .dismissible(); valid with or without icon, actions and description`。
  3. 一括の定型文は禁止する。親の lint 修正（`projection:` の関数名の実在と唯一の書き手の検査）が入ると、2 を行わない marker は error になる。
  4. 2 の作業量は約 70 行。struct ごとに書き手は一つの memo（各 component の render）なので、書き手の確認は grep 一回で済む。
  5. policy の `value_modules` に `*_values` を足す案は採らない。含意（E-1）が見えなくなるため。

### 5. 正しさ・リスク・E2E

| 修正 | 振る舞い | 担保・追加 |
|---|---|---|
| E-1 | toggle_group の icon の無い item が正方形でなくなる | 追加：`ui_toolbar.rs` の `toggle_group_icon_only_keeps_iconless_items_out_of_the_square_look`。既存：`ui_toolbar.rs`、各 kit catalog の button・toggle の撮影 |
| E-2 | 無し | 既存：`interaction_menu.rs`、`apg_menu*.rs` |
| E-3 | 退行を直す | 追加：game catalog `window_status_shows_the_last_command_after_run` |
| E-4 | 無し | 既存：`ui_field.rs`、`interaction_edit.rs`、creative dock |
| E-5 | 無し | 既存：soft `navigation.rs:515` |
| E-6・E-10 | 無し（文言だけ） | `cargo xtask check --no-write --only state-fields --crate <crate>` |
| E-7・E-8・E-9 | 無し | 既存：creative catalog、`apg_date_picker.rs`、`interaction_table.rs` |
| CardState | 無し | 既存：`components.rs`、kit catalog |

リスク：

- E-1・E-2・CardState は公開の recipe state の field を変える。kit（basic・pop・soft・creative）は同じ unit で直す。
- 互換の alias は作らない（方針 5）。外部の theme 作者への影響は CHANGELOG 相当の記録（あれば）に書く。

### 6. unit 分割案（所有ファイルは重ならない）

| unit | crate | 所有ファイル | 依存 | 軽い確認 |
|---|---|---|---|---|
| E-U1 | kivact-interaction（+ 読み手の kit） | `kivact-interaction/src/menu/state.rs`、`menu/level.rs`、`menu/handle.rs`、`menu.rs`、`lib.rs`（pub use）、`inline_edit.rs`、`table/data.rs`、`kivact-ui-basic/src/recipes_menu.rs`、`kivact-ui-soft/src/recipes_menu.rs`、facade の `crates/kivact/src/` の interaction re-export 行 | 無し | `cargo check -p kivact-interaction -p kivact-ui-basic -p kivact-ui-soft`、`cargo clippy -p kivact-interaction --all-targets -- -D warnings`、`dev/tools/window-lock.sh --timeout 600 -- cargo test -p kivact-public-api-tests --test e2e interaction_menu` |
| E-U2 | kivact-ui（button 系）+ kit の recipe | `kivact-ui/src/values.rs`、`button.rs`、`toggle_button.rs`、`toggle_group.rs`、`look.rs`、`kivact-ui-basic/src/recipes.rs`、`kivact-ui-pop/src/recipes.rs`、`kivact-ui-soft/src/recipes_controls.rs`、`kivact-ui-creative/src/recipes.rs`、`dev/tests/kivact-public-api/tests/e2e/ui_toolbar.rs` | 無し | `cargo check -p kivact-ui -p kivact-ui-basic -p kivact-ui-pop -p kivact-ui-soft -p kivact-ui-creative`、`… --test e2e ui_toolbar` |
| E-U3 | kivact-ui（card・table・date・marker） | `kivact-ui/src/component_values.rs`、`card.rs`、`look_feedback.rs`、`data_table/types.rs`、`data_table/builder.rs`、`data_table/runtime.rs`、`date_input/view.rs`、`date_range_input.rs`、`code_values.rs`、`kivact-ui-soft/src/card.rs` | 無し（E-U2 と同時可。`values.rs` は触らない） | `cargo check -p kivact-ui -p kivact-ui-soft`、`… --test e2e components`、`… --test e2e apg_date_picker` |
| E-U4 | kivact-ui-soft | `side_navigation/shell.rs`、`lightbox.rs` | 無し | `cargo check -p kivact-ui-soft`、`cargo test -p kivact-ui-soft --test consumer rail_navigation`（window-lock 経由） |
| E-U5 | examples（game・creative） | `examples/kivact-ui-game/src/game/window.rs`、`game/controls.rs`、`examples/kivact-ui-game/tests/catalog.rs`、`examples/kivact-ui-creative/src/creative/jobs.rs`、`creative/shell.rs` | 無し | `cargo check -p kivact-ui-game-catalog -p kivact-ui-creative-catalog`、`… cargo test -p kivact-ui-game-catalog --test catalog window_status` |
| E-U6 | snapshot marker の書直し（4 節 2） | 4 節の列挙のうち E-U1〜U3 が所有しない `*_values.rs`（`chat_values.rs`、`collection_values.rs`、`navigation_values.rs`、`indicator_values.rs`、`tree_values.rs`、`table_values.rs`、`disclosure_values.rs`）と `kivact-interaction/src/tree.rs`・`list/data.rs`・`calendar/data.rs` の marker 行。tree・list・calendar は OPEN 担当の unit と同じファイルなので、**その unit に統合**する | 親の lint 修正の後 | `cargo xtask check --no-write --only state-fields --crate kivact-ui`（kivact-interaction も同じ） |

- `kivact-ui-creative-catalog` の package 名は推測。`examples/kivact-ui-creative/Cargo.toml` の `name` を使う。
- recap：E-U1〜U5 はファイルが重ならず並行できる。E-U6 は lint 修正の後に行う。

### 7. 既存欠陥（まとめ）

1. E-1：`kivact-ui/src/toggle_group.rs:169` が `icon_only` を `leading_icon` と無関係に立てる。icon の無い item が正方形に描かれ、rustdoc の不変条件にも反する。
2. E-3：`examples/kivact-ui-game/src/game/window.rs:63-64,236-239`。Run・Pause の後、status の notice（Saved・New・Clear）が永久に出ない（79bbd409 からの退行）。
3. E-5：`kivact-ui-soft/src/side_navigation/shell.rs:490-532`。`SidebarWidth::Collapsed` は読まれない重複状態で、caller の binding と矛盾し得る（振る舞いへの影響は無し）。
4. 4 節：snapshot struct の約 80 個の `projection:` marker は、SL-8 の書式だけを満たす一括書換え。

### 8. 設計者（人間）の判断が要る点

- E-1・E-2・CardState は公開の recipe state の形を変える。kit 内の読み手は全部直すが、外部 theme 作者への影響を受け入れるか確認が要る。方針 1・5 からは受け入れる側を推奨する。

---

## 付録 P5-F：P5 slice F：recipe の snapshot に残る projection marker と、ファイルをまたいで重複する independent 理由の書直し

対象は HEAD 73c36c7c。リポジトリは読取りのみで、build はしていない。E-1、E-2、CardState、DataTableContext、lightbox、TableRow/Cell、ListItemState、TreeItemState、MenuItemState、paint の Step は B、D、E が担当済みなので、ここでは扱わない。

前提とする lint の変更（親が設計する）:

- **X4**：`projection:` は `written only by <名前>` を一つだけ持ち、その名前の fn が同じ crate にあって、その fn だけが書くこと。
- **X5**：同じ `independent:` の理由文を複数のファイルに置かないこと。

方針：

- 全ての snapshot struct は書き手の memo や closure（file:line）を読んで判断した。field どうしに含意や排他があるものは型を変え、それ以外は field ごとに「誰が設定するか」と「到達する具体的な組合せ」を marker に書く。
- marker は既存の marker 行と同じ位置（field の doc comment の上）に置き換える。表の「行」は HEAD での既存の marker 行。
- 文言は英語のまま全文を載せる。

### 1. 型を変えるもの（6 件、全て公開 API の変更あり。ただし 5 の slider だけは private）

#### F-T1【公開】`BreadcrumbItemState`：`current` と `input` は排他

- 事実：`crates/kivact-ui/src/breadcrumb.rs:155-209`。`current || activate.is_none()` の分岐は `input: None` を作り、それ以外の分岐は `current: false`（:203 の `current` は false の分岐にある）。そのため (current=true, input=Some) は作られない。到達するのは Current、Label、Link の 3 通り。
- 型（`crates/kivact-ui/src/navigation_values.rs:72-79` を置き換える。marker 2 本は消す）:

```rust
/// What a breadcrumb entry is, with its link's actual input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BreadcrumbItemState {
    /// The current page's label (`aria-current="page"`); not a link.
    Current,
    /// A label the caller gave no activation.
    Label,
    /// A link and the actual input of its activation part.
    Link(kivact_core::custom::PartState),
}
impl BreadcrumbItemState {
    /// Whether this is the current page's entry.
    pub fn is_current(&self) -> bool {
        matches!(self, Self::Current)
    }
    /// The link's actual input; `None` for the current page and plain labels.
    pub fn input(&self) -> Option<kivact_core::custom::PartState> {
        match self {
            Self::Link(input) => Some(*input),
            Self::Current | Self::Label => None,
        }
    }
}
```

- 書き手（`breadcrumb.rs`）:
  - `:155` の直後に `let is_current = current;` を置く（引数の bool を写す）。
  - `:161-164` と `:176-179`：`move || if is_current { BreadcrumbItemState::Current } else { BreadcrumbItemState::Label }`
  - `:203-206`：`move || BreadcrumbItemState::Link(input.get())`
- 読み手:
  - `crates/kivact-ui/src/look_navigation.rs:97`：`s.current` → `s.is_current()`
  - `crates/kivact-ui-basic/src/breadcrumb.rs:41`：同上
  - `crates/kivact-ui-soft/src/breadcrumbs.rs:61` と `:66`：`s.current || s.input.is_none()` → `s.input().is_none()`（Current の時も input は None なので同値）
  - 同 `:69`：`s.input.is_some_and(|input| input.hovered)` → `s.input().is_some_and(|input| input.hovered)`。D の F-1（PartState）が先に入っていれば `input.is_hovered()`。
- 文書：`docs/for-kivact-user/en/how-to-guide/navigation.md:47` と `jp/...:47` の `BreadcrumbItemState { current, input }` を書き換える。
  - en：`` `BreadcrumbItemState` (`Current`, `Label` or `Link(input)`); `input()` is the actual link part's state and is `None` for passive/current labels. ``
  - jp：同じ意味に訳す。
- 振る舞い：変わらない。既存の担保は `dev/tests/kivact-public-api/tests/e2e/apg_breadcrumb.rs` と `ui_navigation.rs`、soft の catalog。

#### F-T2【公開】`CodeBlockState`：`has_header` が false なら `expanded` は true

- 事実：`crates/kivact-ui/src/code_block.rs:203-236`。
  - `open` は `!can_collapse || expanded`。
  - `has_header` は `… || can_collapse`。
  - そのため「header が無い ⇒ 畳めない ⇒ expanded は true」で、(has_header=false, expanded=false) は作られない。
  - `expanded` を読む recipe は無い（Grep 済み）。`has_header` の読み手は `code_block.rs:337` と `crates/kivact-ui-basic/src/code_block/view.rs:92` の 2 箇所。
- 型（`crates/kivact-ui/src/code_values.rs` 全体を置き換える）:

```rust
//! Role: Read-only actual construction facts for source-code recipes.
/// Whether a code block can collapse and, if so, whether it shows every line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeCollapse {
    /// The block cannot collapse; every line shows.
    Fixed,
    /// The block can collapse and shows every line.
    Expanded,
    /// The block can collapse and shows its first lines only.
    Collapsed,
}
/// Rendered source facts supplied to code-block recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeBlockState {
    // xtask-allow(state-fields): independent: CodeBlock::line_numbers shows the gutter in every collapse state, wrapped or not
    pub numbered: bool,
    // xtask-allow(state-fields): independent: CodeBlock::wrap wraps long lines whether or not the gutter shows
    pub wrapped: bool,
    pub collapse: CodeCollapse,
    // xtask-allow(state-fields): independent: use_copy_feedback holds it for copy_feedback_ms after Copy; the feedback shows in any collapse state and Expand works while it shows
    pub copied: bool,
    // xtask-allow(state-fields): independent: CodeBlock::title, language, copy_button or end_content fill the header; a collapsible block shows its header even when this is false
    /// The header has a title, a language label, a copy button or end
    /// content, besides any collapse toggle.
    pub header_content: bool,
    // xtask-allow(state-fields): independent: CodeBlock::footer fills the footer slot, with or without header content
    pub has_footer: bool,
}
impl CodeBlockState {
    /// Whether every line shows.
    pub fn is_expanded(&self) -> bool {
        self.collapse != CodeCollapse::Collapsed
    }
    /// Whether the header row shows: it has content or a collapse toggle.
    pub fn has_header(&self) -> bool {
        self.header_content || self.collapse != CodeCollapse::Fixed
    }
}
```

- 書き手（`code_block.rs:225-235`）:

```rust
        let state = memo(move || CodeBlockState {
            numbered: numbered.get(),
            wrapped: wrapped.get(),
            collapse: match (can_collapse.get(), open.get()) {
                (false, _) => CodeCollapse::Fixed,
                (true, true) => CodeCollapse::Expanded,
                (true, false) => CodeCollapse::Collapsed,
            },
            copied: copied.get(),
            header_content: has_title
                || (has_language && language_label.get())
                || copy_button.get()
                || has_end,
            has_footer,
        });
```

- import：`code_block.rs:2` を `use crate::code_values::{CodeBlockState, CodeCollapse};` にする。`crates/kivact-ui/src/lib.rs:411` の `pub use code_values::CodeBlockState;` を `pub use code_values::{CodeBlockState, CodeCollapse};` にする。
- 読み手：`code_block.rs:337` は `!state.get().has_header` → `!state.get().has_header()`。`crates/kivact-ui-basic/src/code_block/view.rs:92` は `s.has_header` → `s.has_header()`。
- E-10 の CodeBlockState 行（`expanded`／`copied` の文言）は本節で置き換える（`expanded` の field は無くなる）。
- 振る舞い：変わらない。既存の担保は `dev/tests/kivact-public-api/tests/e2e/ui_chat_code.rs`（collapse・copy）と basic catalog。

#### F-T3【公開】`AlertState`：`expansion != NotExpandable` なら `has_content`

- 事実：`crates/kivact-ui/src/alert.rs:173-181`。`expansion = Expansion::new(has_details && collapsible, open)`、`has_content = has_details`。(has_content=false, Collapsed/Expanded) は作られない。
- 型（`crates/kivact-ui/src/component_values.rs:7-21` を置き換える。`Expansion` は残す）:

```rust
/// Alert meaning and disclosure state supplied to recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlertState {
    pub tone: Tone,
    // xtask-allow(state-fields): option: Alert::content gave details; Some(NotExpandable) shows them always, Some(Collapsed or Expanded) when Alert::collapsible is on
    /// The details slot: absent without details, otherwise whether it can
    /// collapse and is open.
    pub details: Option<Expansion>,
    // xtask-allow(state-fields): independent: Alert::dismissible shows the dismiss button, with or without details, icon or actions
    pub dismissible: bool,
    // xtask-allow(state-fields): independent: Alert::icon fills the icon slot of a dismissible or fixed alert alike
    pub has_icon: bool,
    // xtask-allow(state-fields): independent: Alert::actions fills the action slot, beside or without a dismiss button
    pub has_actions: bool,
    // xtask-allow(state-fields): independent: Alert::description adds the description line, with or without details below it
    pub has_description: bool,
}
```

- 書き手（`alert.rs:173-181`。**C-UI の unit が alert.rs を所有するので、その unit が行う**）: `expansion:` と `has_content:` の 2 行を `details: has_details.then(|| Expansion::new(collapsible.get(), details.get())),` に替える（右辺の `details` は既存の binding の local）。
- 読み手:
  - `crates/kivact-ui-soft/src/banner.rs:122`：`s.has_content && s.expansion != Expansion::Collapsed` → `s.details.is_some_and(|d| d != Expansion::Collapsed)`
  - 同 `:135`：`s.expansion != Expansion::NotExpandable` → `s.details.is_some_and(|d| d != Expansion::NotExpandable)`
  - 他に読み手は無い（Grep 済み）。
- 文書：`docs/for-kivact-user/en/how-to-guide/feedback.md:79` と `jp/...:61` の `AlertState.has_content` を `AlertState.details`（`None` は詳細の slot が無い）に書き換える。
- 振る舞い：変わらない。既存の担保は `feedback.rs`、`apg_alert.rs`、soft catalog の banner。

#### F-T4【公開】`CheckableState`：`text` と `switch_drag` は Switch のときだけ

- 事実：
  - `crates/kivact-ui/src/toggle.rs:105-116`：`text` は `$text`（Switch の時だけ Some）から、`switch_drag` は `$gesture`（Switch の `switch_gesture` だけが Some を返し得る）から作る。
  - `radio_group.rs:237-245` と `crates/kivact-ui-soft/src/indicator.rs:122-129` は両方 None。
  - 「(text=None, switch_drag=Some)」は作られない。
- 型（`crates/kivact-ui/src/values.rs:286-324` の該当部を置き換える）:

```rust
/// What only a switch reports to its recipe.
#[derive(Clone, Copy, Debug)]
pub struct SwitchFacts {
    /// The localized word for the state: `switch.on` ("ON") or `switch.off`
    /// ("OFF"). The handle is made at setup and stays the same for the
    /// mounted control; read it in the paint. It is not part of equality.
    pub text: kivact_core::state::Value<String>,
    /// Live visual state while the opted-in drag gesture moves the thumb.
    pub drag: Option<SwitchDragState>,
}
pub struct CheckableState {
    pub input: PartState,
    pub checked: CheckedState,
    pub size: ControlSize,
    // xtask-allow(state-fields): independent: the caller's .invalid(..) on a check box or switch (a radio item reports false); it holds in any checked state
    /// Whether the caller marked the value invalid (`.invalid(..)`).
    pub invalid: bool,
    // xtask-allow(state-fields): option: Some only for a Switch, with its ON/OFF word and its opted-in drag; a check box and a radio item report None
    /// The switch-only facts; `None` for a check box and a radio item.
    pub switch: Option<SwitchFacts>,
}
impl PartialEq for CheckableState {
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input
            && self.checked == other.checked
            && self.size == other.size
            && self.invalid == other.invalid
            && self.switch.map(|s| s.drag) == other.switch.map(|s| s.drag)
    }
}
```

- 書き手:
  - `toggle.rs:109-116`：`text, switch_drag: …` の 2 行を `switch: text.map(|text| SwitchFacts { text, drag: drag.and_then(|drag| drag.get()) }),` にする。
  - `radio_group.rs:243-244` と `kivact-ui-soft/src/indicator.rs:127-128`：`text: None, switch_drag: None` を `switch: None` にする。
- 読み手:
  - `crates/kivact-ui-game/src/recipes_controls.rs:277`：`state.text.map(|text| text.get())` → `state.switch.map(|switch| switch.text.get())`
  - `crates/kivact-ui-soft/src/recipes_controls_choice.rs:241` と `:283`：`let drag = s.switch_drag;` → `let drag = s.switch.and_then(|switch| switch.drag);`
- export：`crates/kivact-ui/src/lib.rs` で `CheckableState` を export している行に `SwitchFacts` を足す。
- 残る排他（設計者の判断）：「`checked == Indeterminate` なら `switch == None`」（Indeterminate になるのは check box だけ、`toggle.rs:184` で switch は mixed を無視する）は型に残らない。
  - これを型にするには、`checked` を control の種類ごとの enum にする必要がある。recipe の `.checked` の読み手は kit 全体で 54 箇所ある。
  - `CheckedState` は phase 型で lint の audit 上は type で閉じる。doc の不変条件（:293-294）で示すにとどめることを推奨する。親が全面的に型にすると決めた場合の型は `pub enum CheckControl { CheckBox(CheckedState), Switch { on: bool, facts: SwitchFacts }, Radio(bool) }`。
- 振る舞い：変わらない。既存の担保は `interaction_toggle.rs`、`ui_components.rs`、game・soft catalog の switch。

#### F-T5【公開】`DataTableCheckState`：行の box は Indeterminate にならない

- 事実：`crates/kivact-ui/src/data_table/rows.rs:565-568` は `header: false` と `CheckedState::from_bool(..)`。`data_table/chrome.rs:211-219` だけが `header: true` で Indeterminate を作る。読み手 `look_collections.rs:166` は値を読まない。
- 型（`values.rs:427-436` を置き換える。marker は消す）:

```rust
/// The state of the box around a check box of a
/// [`data_table`](super::data_table), for its recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataTableCheckState {
    /// The header's box, which checks the page's rows; Indeterminate while
    /// some of them are checked.
    Header(CheckedState),
    /// A row's box.
    Row { checked: bool },
}
```

- 書き手:
  - `chrome.rs:211-219`：`move || DataTableCheckState::Header(if all() { CheckedState::Checked } else if mixed() { CheckedState::Indeterminate } else { CheckedState::Unchecked })`
  - `rows.rs:565-568`：`move || DataTableCheckState::Row { checked: chosen() }`
- 振る舞い：変わらない。既存の担保は `ui_data_table.rs` と `interaction_table.rs`。

#### F-T6（private）slider の `Geometry`：`vertical` が true なら `rtl` は false

- 事実：`crates/kivact-ui/src/slider.rs:293-298` は `rtl: !vertical && …`。(vertical, rtl)=(T,T) は作られない。
- 型（`:174-181` を置き換える。marker 2 本は消す）:

```rust
/// The direction a slider's value grows along its track.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TrackFlow {
    /// A horizontal track in left-to-right text.
    LeftToRight,
    /// A horizontal track in right-to-left text.
    RightToLeft,
    /// A vertical track: the value grows upwards.
    BottomToTop,
}
struct Geometry {
    track: Value<Option<Rect>>,
    thumb: Value<Option<Rect>>,
    flow: TrackFlow,
}
impl Geometry {
    fn vertical(&self) -> bool {
        self.flow == TrackFlow::BottomToTop
    }
    // 既存のメソッドが続く
}
```

- 読み手：
  - `:189`、`:211`、`:225`、`:231`、`:246` の `self.vertical` → `self.vertical()`
  - `:194` の `else if self.rtl` → `else if self.flow == TrackFlow::RightToLeft`
  - `:238` の `self.vertical || self.rtl` → `self.flow != TrackFlow::LeftToRight`
- 書き手 `:293-298`：`flow: if vertical { TrackFlow::BottomToTop } else if use_direction().get() == ReadingDirection::Rtl { TrackFlow::RightToLeft } else { TrackFlow::LeftToRight },`
- `Geometry` は数える field が無くなり、行も消える。
- 振る舞い：変わらない。既存の担保は `apg_slider.rs` と `interaction_slider.rs`（vertical・RTL）。

### 2. marker の書直し（型は変えない）

各行の書き手は表に書いたとおりで、どれも単一の memo か closure。「新しい marker」の列は、その行に置く全文。

#### crates/kivact-ui/src/chat_values.rs（ChatMessageState。書き手 `chat_message.rs:155-183` `ChatMessage::recipe_state`）

| 行 | field | 決定 | 新しい marker |
|---|---|---|---|
| 77 | status | option | `// xtask-allow(state-fields): option: ChatMessage::status; absent for a message without a delivery state` |
| 82 | has_reactions | independent | `// xtask-allow(state-fields): independent: ChatMessage::reactions fills the reaction slot of a message with or without header, footer, avatar or press handler` |
| 84 | actionable | independent | `// xtask-allow(state-fields): independent: ChatMessage::on_press makes the bubble pressable, whichever of its slots are filled` |
| 86 | has_header | independent | `// xtask-allow(state-fields): independent: ChatMessage::header fills the slot above the bubble; header, footer and avatar are separate builder calls and each renders alone` |
| 88 | has_footer | independent | `// xtask-allow(state-fields): independent: ChatMessage::footer fills the slot below the bubble, with or without a header` |
| 90 | has_avatar | independent | `// xtask-allow(state-fields): independent: ChatMessage::avatar fills the avatar slot beside a bubble of any group position, with or without header or footer` |

#### crates/kivact-ui/src/collection_values.rs

| 行 | field | 書き手 | 決定 | 新しい marker |
|---|---|---|---|---|
| 43 | ToolbarState.centered | `toolbar.rs:98,130` | independent | `// xtask-allow(state-fields): independent: Toolbar::center gave a centre region; it is laid out in either reading direction` |
| 45 | ToolbarState.rtl | `toolbar.rs:95,130` | independent | `// xtask-allow(state-fields): independent: the reading direction from use_direction at mount; it mirrors the start and end regions with or without a centre region` |
| 82 | ItemRowState.has_description | `item_row.rs:201-215` | independent | `// xtask-allow(state-fields): independent: ItemRow::description (text or content) was given; pressable and passive rows both show one` |
| 84 | ItemRowState.actionable | 同上 | independent | `// xtask-allow(state-fields): independent: ItemRow::on_press or a delegated target makes the row pressable, with or without a description or a list position` |
| 86 | ItemRowState.position | 同上 | option | `// xtask-allow(state-fields): option: ItemRow::position, else the enclosing item list's numbering; absent for a row outside a numbered list` |
| 90 | ItemRowState.title_lines | 同上 | option | `// xtask-allow(state-fields): option: ItemRow::title_lines, else ItemListTokens::title_lines; absent means the title is not clamped` |
| 92 | ItemRowState.description_lines | 同上 | option | `// xtask-allow(state-fields): option: ItemRow::description_lines, else ItemListTokens::description_lines; absent means no clamp, and a row without a description ignores it` |

#### crates/kivact-ui/src/disclosure_values.rs（書き手は `DisclosureRecipeState::from_part` :20-29 だけ。Grep で `DisclosureRecipeState {` の literal は :24 のみ）

| 行 | field | 決定 | 新しい marker |
|---|---|---|---|
| 14 | open | independent（本文 3.6 O2） | `// xtask-allow(state-fields): independent: copied from the attached part's DisclosureState::open by DisclosureRecipeState::from_part; standalone panels and accordion members (group Some) both open and close` |
| 16 | group | option | `// xtask-allow(state-fields): option: the panel's place in an accordion; absent for a standalone disclosure` |

#### crates/kivact-ui/src/indicator_values.rs（BadgeState。書き手 `badge.rs:57-67`）

| 行 | field | 決定 | 新しい marker |
|---|---|---|---|
| 25 | actionable | independent | `// xtask-allow(state-fields): independent: Badge::on_press makes the badge pressable; its icons are added by separate calls` |
| 28 | leading_icon | independent | `// xtask-allow(state-fields): independent: Badge::icon fills the leading slot, with or without a trailing icon or press handler` |
| 30 | trailing_icon | independent | `// xtask-allow(state-fields): independent: Badge::trailing_icon fills the trailing slot, with or without a leading icon or press handler` |

#### crates/kivact-ui/src/navigation_values.rs

| 行 | field | 書き手 | 決定 | 新しい marker |
|---|---|---|---|---|
| 25 | StepperItemState.collapsed | `stepper.rs:315-331` `step_row` | independent | `// xtask-allow(state-fields): independent: copied from StepperState::collapsed, the whole stepper's responsive layout; disabled and optional steps appear in either layout` |
| 27 | StepperItemState.disabled | 同上 | independent | `// xtask-allow(state-fields): independent: StepperItem::disabled; a disabled step may also be optional and may have an activation handler on the stepper` |
| 29 | StepperItemState.optional | 同上 | independent | `// xtask-allow(state-fields): independent: StepperItem::optional only marks the label; it combines with disabled and either layout` |
| 75, 77 | BreadcrumbItemState | — | 型（F-T1） | marker を消す |
| 116 | PaginationSummary.total_items | `pagination.rs:406-420` `summary_source` | option | `// xtask-allow(state-fields): option: Pagination::total_items; absent when the caller does not know the total` |
| 119 | PaginationSummary.has_more | 同上 | independent | `// xtask-allow(state-fields): independent: Pagination::has_more for remote paging; the caller reports more pages with or without a known total` |

#### crates/kivact-ui/src/table_values.rs（RowStatus。書き手は builder `RowStatus::new`・`color`・`icon` :27-46）

| 行 | field | 決定 | 新しい marker |
|---|---|---|---|
| 19 | icon | option | `// xtask-allow(state-fields): option: RowStatus::icon; absent draws the tone's default mark` |
| 21 | color | option | `// xtask-allow(state-fields): option: RowStatus::color; absent uses the theme's color for the tone` |

#### crates/kivact-ui/src/tree_values.rs（TreeRowRecipeState。書き手 `tree.rs:298-313` `row_recipe`）

| 行 | field | 決定 | 新しい marker |
|---|---|---|---|
| 8 | is_last | independent | `// xtask-allow(state-fields): independent: the row is the last visible sibling at its level (TreeRow::is_last); last rows may be branches or leaves` |
| 10 | has_branches | independent | `// xtask-allow(state-fields): independent: some visible row of the whole tree has children; every row, last or not, reads the same value` |
| 12 | custom_marker | independent | `// xtask-allow(state-fields): independent: Tree::marker_content was given; it replaces the glyph on last and inner rows alike` |

#### crates/kivact-ui/src/values.rs（F-T4・F-T5 以外）

| 行 | field | 書き手 | 決定 | 新しい marker |
|---|---|---|---|---|
| 301, 304, 311 | CheckableState | — | 型（F-T4） | 上の型定義の 2 本に置き換える |
| 329 | DialogState.open | `dialog.rs:135-142` `Shape::state`、`drawer_swipe.rs:120-125` | independent | `// xtask-allow(state-fields): independent: the popup's open state (Shape::state) or the drawer's open binding; a dialog with or without media opens and closes` |
| 332 | DialogState.media | 同上 | independent | `// xtask-allow(state-fields): independent: Dialog::media gave a header view (the drawer's drag handle reports false); it is fixed while the dialog opens and closes` |
| 357 | SelectTriggerState.open | `select/handle.rs:501-521` `trigger_projection`、`selection_field.rs:217-222` | independent | `// xtask-allow(state-fields): independent: the select's popup or the selection field's FieldState::Open; the list opens with or without a selected option` |
| 360 | SelectTriggerState.placeholder | 同上 | independent | `// xtask-allow(state-fields): independent: no listed option is selected (trigger_projection; a selection field reports false); the trigger shows it while the list is open or closed` |
| 407 | DataTableRowState.color | `data_table/rows.rs:690-694` | option | `// xtask-allow(state-fields): option: DataTable::row_color for this row; absent keeps the theme's row background` |
| 412 | DataTableRowState.height | 同上 | option | `// xtask-allow(state-fields): option: DataTable::row_height (at least 18); absent uses the theme's row height` |
| 432 | DataTableCheckState.header | — | 型（F-T5） | marker を消す |

#### crates/kivact-ui/src/component_values.rs（AlertState は F-T3。CardState は E 担当）

InputGroupState の書き手は `input_group.rs:107-117`（memo）、`:244-248`（`input_addon` は Default）、`component_values.rs:160`（Default）。InputGroupMemberState の書き手は `input_group.rs:200-206` `GroupRender::member`。

| 行 | field | 決定 | 新しい marker |
|---|---|---|---|
| 94 | InputGroupState.focused | independent | `// xtask-allow(state-fields): independent: focus is within the group (focus_within of its root); invalid and any adornment combine with it` |
| 96 | InputGroupState.invalid | independent | `// xtask-allow(state-fields): independent: the group's .invalid(..) (accessible_invalid); a group is invalid whether focused or not` |
| 98 | has_start | independent | `// xtask-allow(state-fields): independent: InputGroup::start fills the inline start adornment, with or without an end one` |
| 100 | has_end | independent | `// xtask-allow(state-fields): independent: InputGroup::end fills the inline end adornment, with or without a start one` |
| 102 | has_block_start | independent | `// xtask-allow(state-fields): independent: InputGroup::block_start adds a row above the controls, beside any inline adornments` |
| 104 | has_block_end | independent | `// xtask-allow(state-fields): independent: InputGroup::block_end adds a row below the controls, beside any inline adornments` |
| 114 | InputGroupMemberState.first | independent | `// xtask-allow(state-fields): independent: index == 0 in GroupRender::member; a single member is both first and last` |
| 116 | InputGroupMemberState.last | independent | `// xtask-allow(state-fields): independent: index + 1 == count in GroupRender::member; the middle member of three is neither` |
| 118 | InputGroupMemberState.focused | independent | `// xtask-allow(state-fields): independent: focus is within this member (focus_within of its node), at any position in the group` |

#### (b) ファイルをまたぐ重複の解消

| file:行 | field | 書き手 | 決定 | 新しい marker |
|---|---|---|---|---|
| `crates/kivact-ui/src/carousel.rs:143` | Layout.looping | `carousel.rs:204`（`Carousel::looping` :56） | option | `// xtask-allow(state-fields): option: Carousel::looping; false stops navigation at the first and last slides` |
| `crates/kivact-ui/src/combobox/data.rs:27` | Combobox.free_text | `combobox/handle.rs:73-74` | option | `// xtask-allow(state-fields): option: Combobox::free_text; false accepts only listed options` |
| `combobox/data.rs:31` | Combobox.chips | `combobox/handle.rs:83-84` | option | `// xtask-allow(state-fields): option: Combobox::chips; false shows no chips for the selected keys` |
| `combobox/data.rs:43` | Combobox.exclude_selected | `combobox/handle.rs:123-124` | option | `// xtask-allow(state-fields): option: Combobox::exclude_selected; false keeps selected options in the list` |
| `crates/kivact-ui/src/data_table/types.rs:132` | DataTable.tree_expand_all | `data_table/builder.rs:186-187` | option | `// xtask-allow(state-fields): option: DataTable::expand_all_control; false leaves the tree column header without the expand/collapse-all control` |
| `crates/kivact-ui/src/menu.rs:42` | Menu.at_least_anchor_width | `menu.rs:122` | option | `// xtask-allow(state-fields): option: Menu::at_least_anchor_width; false sizes the surface to its content` |
| `menu.rs:278` | Level.at_least_anchor_width | `menu.rs:218`（最上位）、`:551`（submenu は false） | option | `// xtask-allow(state-fields): option: copied from Menu::at_least_anchor_width for the top level; submenu levels are false` |
| `slider.rs:177,179` | Geometry | — | 型（F-T6） | marker を消す |
| `crates/kivact-ui/src/disclosure_tokens.rs:9` | DisclosureMotion.animate | theme | independent | `// xtask-allow(state-fields): independent: the theme's default for Disclosure::animate; a disclosure that opts in with .animate(true) uses transition even when this is false` |
| `disclosure_tokens.rs:12` | DisclosureMotion.transition | theme | option | `// xtask-allow(state-fields): option: the theme's size transition for animating disclosures; absent uses a duration-based ease-out tween` |
| `crates/kivact-ui/src/progress_tokens.rs:6` | ProgressMotion.animate_initial | theme | independent | `// xtask-allow(state-fields): independent: whether the first determinate fill animates from zero; the moving-fill clock that reduced_period_ms drives is unaffected` |
| `progress_tokens.rs:15` | ProgressMotion.reduced_period_ms | theme | option | `// xtask-allow(state-fields): option: the moving-fill clock's period under reduced motion; absent leaves the indeterminate fill still` |
| `crates/kivact-interaction/src/combobox/handle.rs:115` | ComboboxApi.content_state | `ComboboxApi::content` :640-643 | independent | `// xtask-allow(state-fields): independent: set by ComboboxApi::content when the list surface attaches; the input part attaches separately through ComboboxApi::input, before the surface first mounts` |
| `crates/kivact-interaction/src/disclosure.rs:54` | DisclosureApi.trigger_state | `DisclosureApi::trigger` :205-208 | independent | `// xtask-allow(state-fields): independent: set by DisclosureApi::trigger when the kit attaches a trigger; an always-open disclosure has content and no trigger` |
| `disclosure.rs:56` | DisclosureApi.content_state | `DisclosureApi::content` :221-228 | independent | `// xtask-allow(state-fields): independent: set by DisclosureApi::content when the content part mounts; before the first opening a disclosure has its trigger and no content` |
| `crates/kivact-interaction/src/menu/level.rs:23` | Level.content_state | `MenuApi::content` `menu/handle.rs:461-474` | independent | `// xtask-allow(state-fields): independent: set by MenuApi::content when this level's surface mounts; a submenu level exists in its session before its surface first mounts` |
| `crates/kivact-interaction/src/popup/api.rs:103` | PopupApi.content_state | `PopupApi::content_with` :453-463 | independent | `// xtask-allow(state-fields): independent: set by PopupApi::content_with when the surface mounts, only once the popup first opens; the trigger has attached by then or is absent` |
| `popup/api.rs:105` | PopupApi.trigger_state | `PopupApi::base_trigger` :416-421 | independent | `// xtask-allow(state-fields): independent: set by PopupApi::base_trigger at the trigger's mount; until the popup first opens there is a trigger and no content` |
| `crates/kivact-interaction/src/select/handle.rs:131` | SelectApi.content_state | `SelectApi::content_in` :566-598 | independent | `// xtask-allow(state-fields): independent: set by SelectApi::content_in when the option list or sheet mounts; the preview callback beside it is set by the caller at setup, before or without any surface` |
| `crates/kivact-interaction/src/tooltip/api.rs:130` | TooltipApi.trigger_state | `TooltipApi::trigger` :274-281 | independent | `// xtask-allow(state-fields): independent: set by TooltipApi::trigger at mount; the hint content mounts only once the tooltip first shows` |
| `tooltip/api.rs:132` | TooltipApi.content_state | `TooltipApi::content` :315-322 | independent | `// xtask-allow(state-fields): independent: set by TooltipApi::content when the hint mounts after the show delay, after the trigger attached` |
| `crates/kivact-test/src/screen/disposal_tests.rs:35` | local open | 試験 | independent | `// xtask-allow(state-fields): independent: the show() condition the test flips to dispose the branch; the closed counter and the slot are written by other steps` |
| `disposal_tests.rs:37` | local closed | 試験 | independent | `// xtask-allow(state-fields): independent: the counter a late cleanup writes after the branch is gone; the test reads it whether open is true or false` |
| `crates/kivact-test/src/screen/host_sync_tests.rs:34` | local disabled | 試験 | independent | `// xtask-allow(state-fields): independent: the canvas's disabled input the test sets to reject a pointer down; the press and down counters move only on accepted input` |
| `crates/kivact-test/src/screen/real_window_tests.rs:118` | local shown | 試験 | independent | `// xtask-allow(state-fields): independent: set by the spawned sleep after 300 ms of virtual time; the frame-loop slot of the same test is filled at mount, before and after it flips` |
| `crates/kivact-devtools/src/state.rs:74` | Session.status | `Session::report` `session.rs:230-231` | independent | `// xtask-allow(state-fields): independent: the status bar's last RPC failure, set by Session::report; it stays while connection moves to Lost, and a connected session may have none` |
| `crates/kivact-devtools/src/session.rs:32` | local status（`Session::new`） | 同上 | independent | `// xtask-allow(state-fields): independent: becomes Session.status, empty at setup; the target var beside it is fixed for the session whether or not a failure is shown` |
| `examples/kivact-ui-creative/src/creative/audio_values.rs:16` | TransportState.recording | `audio_samples.rs:166`、`audio_values.rs:50`（`controls`） | independent | `// xtask-allow(state-fields): independent: the catalog's record toggle (audio_samples.rs) and the kit transport (controls) flip it; the stop button clears only playing and the loop switches only cycling` |
| `audio_values.rs:18` | TransportState.cycling | `arrange.rs:50`、`audio_samples.rs:144`、`audio_values.rs:51` | independent | `// xtask-allow(state-fields): independent: the loop switches in arrange.rs and audio_samples.rs and the kit transport set it; play, stop and record leave it` |
| `examples/kivact-ui-basic/src/docs/alert_dialog.rs:77` | local open | `:84`（Delete Chat）、footer の Cancel・Delete | independent | `// xtask-allow(state-fields): independent: "Delete Chat" opens the alert and Cancel or Delete closes it; it opens again after a deletion, so it is true or false with either value of deleted` |
| `alert_dialog.rs:79` | local deleted | footer の Delete（:102-106） | independent | `// xtask-allow(state-fields): independent: set by the alert's Delete button and never cleared; the muted line shows it while the alert is open or closed` |
| `examples/kivact-ui-basic/src/docs/dialog.rs:122` | local open | `:128`（Delete account）、footer の Cancel・Delete | independent | `// xtask-allow(state-fields): independent: "Delete account" opens the dialog and Cancel or Delete closes it; reopening after the deletion keeps deleted true` |
| `dialog.rs:124` | local deleted | footer の Delete（:143-147） | independent | `// xtask-allow(state-fields): independent: set by the dialog's Delete button and never cleared; the account line reads it with the dialog open or closed` |

kit の `crates/kivact-ui-creative/src/audio/transport.rs:18,20` は今の文言のままとする。example 側の文言が上の表で別になるので、X5 には当たらない。

`crates/kivact-ui/src/code_values.rs`、`slider.rs`、`navigation_values.rs:75,77` は 1 節の型の変更で marker が消える。

### 3. 他の担当への申し送り（本 slice では設計しない）

- **D の F-1（PartState → Availability）の見落とし（事実）**：`crates/kivact-ui-soft/src/indicator.rs:117-120` は `input.disabled |= actual.get().disabled;` と `input.focus = Focus::None` で PartState の field を**直接書く**。D は `PartState {` の literal だけを数えたので、この書き手が漏れている。F-1 を行う unit は、ここを次のように直す：
  ```rust
  let mut input = input.get();
  if actual.get().is_disabled() && !input.is_hidden() {
      input.availability = Availability::Disabled;
  }
  if input.is_disabled() {
      input.focus = Focus::None;
  }
  ```
- **推測**：`DisclosureApi`／`PopupApi`／`TooltipApi`／`SelectApi`／`MenuApi` の `content_state` は、content の unmount で None に戻らない（書き手は `.set(Some(..))` だけ）。閉じた後の `content_state()` は、破棄された part の `Value` を読み続ける可能性がある。読み手の memo は `is_open()` の fallback を使うので、表示の不具合は確認していない。checkpoint の E2E で、開いて閉じた後に recipe の state を読む経路に error が出ないことを見るよう勧める。

### 4. 変更後の audit（本 slice の範囲）

| 対象 | 変更後 |
|---|---|
| BreadcrumbItemState、DataTableCheckState、slider Geometry | 行が消える（enum、または数える field が無い） |
| CodeBlockState | numbered・wrapped・copied・header_content・has_footer が全て independent |
| AlertState | details は option、残り 4 本は independent（`has_content`・`expansion` の行は無くなる） |
| CheckableState | invalid は independent、switch は option、checked は type |
| その他 | 上表のとおり。新しい OPEN は無い。`projection:` は disclosure の `open` 1 本だけで、関数 `DisclosureRecipeState::from_part` は実在し、唯一の literal がそこにある |

### 5. unit 分割案（所有は crate 単位、重複は既存の案に合流させる）

| unit | 所有ファイル | 合流・依存 | 軽い確認 |
|---|---|---|---|
| **E-U2 に追加** | `crates/kivact-ui/src/values.rs`（F-T4、F-T5、Dialog・SelectTrigger・DataTableRow の marker）、`toggle.rs`、`radio_group.rs`、`data_table/chrome.rs`、`data_table/rows.rs`、`code_values.rs`・`code_block.rs`（F-T2。E-U3 から移す）、`lib.rs`（`SwitchFacts`・`CodeCollapse` の export）、`crates/kivact-ui-basic/src/code_block/view.rs`、`crates/kivact-ui-soft/src/indicator.rs`、`crates/kivact-ui-soft/src/recipes_controls_choice.rs`、`crates/kivact-ui-game/src/recipes_controls.rs` | E-U2 は既に `values.rs` を所有している。`code_values.rs` は E-U3 の所有から外す（E-10 の CodeBlock 行は本書が置き換える）。soft の `indicator.rs` は D-1（PartState）とも重なるので、D-1 は E-U2 の後に行う | `cargo check -p kivact-ui -p kivact-ui-basic -p kivact-ui-soft -p kivact-ui-game`；`cargo clippy -p kivact-ui --all-targets -- -D warnings`；`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e interaction_toggle ui_chat_code ui_data_table` |
| **E-U3 に追加** | `crates/kivact-ui/src/component_values.rs`（F-T3 と InputGroup の marker）、`crates/kivact-ui-soft/src/banner.rs`、`docs/for-kivact-user/{en,jp}/how-to-guide/feedback.md` | `alert.rs:173-181` の書き手の変更は **C-UI** が行う（C-UI は E-U3 の後） | `cargo check -p kivact-ui -p kivact-ui-soft`；`… --test e2e feedback apg_alert` |
| **E-U6（具体化）** | `crates/kivact-ui/src/chat_values.rs`、`collection_values.rs`、`navigation_values.rs`（F-T1 を含む）、`indicator_values.rs`、`tree_values.rs`、`table_values.rs`、`disclosure_values.rs`、`breadcrumb.rs`、`look_navigation.rs`、`carousel.rs`、`combobox/data.rs`、`menu.rs`、`slider.rs`（F-T6）、`disclosure_tokens.rs`、`progress_tokens.rs`、`crates/kivact-ui-basic/src/breadcrumb.rs`、`crates/kivact-ui-soft/src/breadcrumbs.rs`、`docs/for-kivact-user/{en,jp}/how-to-guide/navigation.md` | `data_table/types.rs:132` の marker 1 行は、`types.rs` を所有する E-U3 が行う。lint の X4/X5 の前でも書ける | `cargo check -p kivact-ui -p kivact-ui-basic -p kivact-ui-soft`；`cargo xtask check --no-write --only state-fields --crate kivact-ui`；`… --test e2e apg_breadcrumb apg_slider interaction_slider` |
| **F-U4（新）kivact-interaction の marker** | `crates/kivact-interaction/src/combobox/handle.rs`、`disclosure.rs`、`popup/api.rs`、`tooltip/api.rs` | `menu/level.rs:23` は E-U1、`select/handle.rs:131` は UB-1 が同じ文言で行う | `cargo check -p kivact-interaction`；`cargo xtask check --no-write --only state-fields --crate kivact-interaction` |
| **F-U5（新）kivact-test** | `crates/kivact-test/src/screen/disposal_tests.rs`、`host_sync_tests.rs`、`real_window_tests.rs` | P5-A-test（`screen/model.rs`・`input.rs`）とはファイルが重ならない | `cargo check -p kivact-test --all-targets`；`cargo xtask check --no-write --only state-fields --crate kivact-test` |
| **F-U6（新）kivact-devtools** | `crates/kivact-devtools/src/state.rs`、`session.rs` | なし | `cargo check -p kivact-devtools`；`cargo xtask check --no-write --only state-fields --crate kivact-devtools` |
| **F-U7（新）examples** | `examples/kivact-ui-basic/src/docs/alert_dialog.rs`、`docs/dialog.rs`、`examples/kivact-ui-creative/src/creative/audio_values.rs` | E-U5（creative の `jobs.rs`・`shell.rs`）とはファイルが重ならない | `cargo check -p kivact-ui-basic-catalog -p kivact-ui-creative-catalog`；`cargo xtask check --no-write --only state-fields --crate kivact-ui-basic-catalog` |

E2E の追加は不要（全て値を作る式と条件を変えないため、振る舞いは変わらない）。checkpoint では上に挙げた既存の E2E と、basic・soft・game・creative の catalog 試験で担保する。
