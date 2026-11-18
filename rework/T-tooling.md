# 付属書 T：tooling（xtask・gate・試験の構成・DevTools の crate・Python・architecture・CONTRIBUTING）

## 1. 範囲と前提

### 1.1 範囲

この付属書は W1（W1a・W1b・W1c）の unit を書く。対象は次のとおり。

| 領域 | 所見・設計の節 |
|---|---|
| xtask の `state` 検査（`state-fields` の置換え） | TL-04、設計 6.1 |
| xtask の規則の変更（`structural-options` の削除、`api-shape`・`internal-contract` の例外を規則の側へ、`visible_to` の dev-dependency の既定、Role 規則の対象外、図の生成の分離、policy の整理） | TL-02、TL-05、TL-06、TL-07、設計 4.3・6.3 |
| 設計 6.3 の新しい検査 5 つ（`interaction-view`、`ui-machine`、`hand-element`、`unit-machine`、`element-shadow`）と、それを「報告のみ」にする `[rework.pending]` | 設計 6.3、UI-04、UI-09、API-13 |
| `docs-pairs` 検査、docs の doctest を英語だけにし `no_run` を型検査の module にする | TL-03、設計 6.4 |
| 約 1180 行の `xtask-allow(state-fields)` の変換・削除、繰り返しの `internal-contract`・`api-shape` の印、例と雛形の `#![warn(redundant_imports)]` と `//! Role:` | TL-04、TL-05、設計 4.3 |
| `kivact-devtools-client` を `kivact-devtools-protocol`（feature `client`）へ統合、`python/` と `dev/xtask/src/devtools_codegen*` の削除（schema の生成は protocol の試験へ） | TL-08、TL-09、DEC-10、設計 4.1 |
| DevTools の Scripts パネルと、それだけが使う protocol（`show_table`、`script_table`、`Topic::Scripts`、config の `python`・`scripts_dir`、`ClientKind::Python`）の削除、`PROTOCOL_VERSION` 1 → 2 | TL-09、DEC-10（親の決定：4.6 節の 3） |
| E2E の 4 binary への分割、`kivact-test` の自己試験の移動・削除、`// Regression:` の札、`GRACE` の実時間 sleep、devtools・agent の試験の polling | TL-12、TL-14、TL-15、設計 6.4 |
| `architecture/` を git から外す、`window-lock.sh --if-mirror`、`check-features.sh` の縮小 | TL-01、TL-02、TL-16、TL-17 |
| README の「Migrating from the previous release」と「Filled recipes」（5 kit）と、それを検査する試験 | KIT-16（設計 4.3 で本付属書へ） |
| `CONTRIBUTING.md`・`dev/README.md` の gate の 3 段 | TL-01、TL-16、設計 6.4 |

### 1.2 本付属書が扱わないもの（4 章で引き渡す）

- TL-10（core の DevTools を一つの hook trait にまとめる）：kivact-core の内部の再構成なので付属書 C に渡す（4.9 節。親が採用）。
- TL-13（kit の consumer 試験の統合）：kit の移行の後に行うので付属書 K に渡す（4.8 節）。
- TL-11（Windows の CI）：設計 7.8 で不採用。
- `docs/for-kivact-developer/{architecture,decisions,strategy}.md` の書き直し：付属書 G（W11a〜W11c）。本付属書は strategy.md の 1 行（TL-09 の指定）だけを直す。

### 1.3 前提

- 基点 `2b8df5ac`。W1 は他の付属書のどの unit よりも前に走る。W1a は付属書 S の `Deferred` の unit と同じ sub-wave で並ぶが、ファイルは重ならない（付属書 S は `crates/kivact-types/src/cell.rs` だけ、本付属書の W1a は `dev/xtask`・`python/`・DevTools の 5 crate と `crates/kivact-core/src/devtools{.rs,/scripts.rs,/target.rs}` だけ）。W1a の lane の順は T01 → T02 → T03 → T04 → T23（T23 は後から足したので番号が飛ぶ）。W1b のすべての unit は T23 の後に始まる。

### 1.4 後の付属書に提供するもの

- xtask の `state` 検査（struct 単位の印 `// xtask-allow(state): <1 行>`）と、全付属書が使う確認のコマンド `cargo xtask check --no-write --crate <crate> [--strict]`。
- `[rework.pending]`（6.3 の検査を「報告のみ」にする表）と `--strict`、`--only local`。
- E2E の binary：`kivact-public-api-tests` の `e2e_core`・`e2e_ui`・`e2e_native`、新しい package `kivact-kit-tests` の `e2e_kits`。共有の helper は `kivact_public_api_tests::{support, paint}`。
- gate の 3 段のコマンド（4.1 節）と AC-4 の検査のコマンド（4.2 節）。

## 2. unit の一覧

| ID | wave | lane | 前提の unit | 所有するファイル・directory | 所見 | 規模 |
|---|---|---|---|---|---|---|
| T01 | W1a | T0 | — | `dev/xtask/src/{state.rs,state/,state_fields.rs,state_fields/,local_checks.rs,check_cli.rs,check_source.rs,main.rs,pending.rs,policy.rs,public_surface.rs}`、`dev/xtask/policy/workspace.toml`、`dev/xtask/state-audit.md`（未追跡なら削除だけ） | TL-04、6.1 | L |
| T02 | W1a | T0 | T01 | `dev/xtask/src/{main.rs,check_cli.rs,local_checks.rs,check_source.rs,api_shape.rs,structural_options.rs,diagrams.rs,core_filters.rs}`、`dev/xtask/policy/workspace.toml` | TL-02、TL-05、TL-06、TL-07、4.3 | M |
| T03 | W1a | T0 | T02 | `dev/xtask/src/{main.rs,check_cli.rs,local_checks.rs,effect_writes.rs,interaction_view.rs,ui_machine.rs,hand_element.rs,unit_machine.rs,element_shadow.rs}`、`dev/xtask/policy/workspace.toml` | 6.3、UI-04、UI-09、API-13 | M |
| T04 | W1a | T0 | T03 | `dev/xtask/src/{main.rs,check_cli.rs,local_checks.rs,docs_pairs.rs,devtools_codegen.rs,devtools_codegen/}`、`dev/xtask/policy/workspace.toml`、`crates/kivact-devtools-protocol/{Cargo.toml,tests/schema_generated.rs,tests/schema_generated/}`、`python/` | TL-03（docs-pairs）、TL-09 | M |
| T23 | W1a | T0 | T04 | `crates/kivact-devtools-protocol/src/{scripts.rs,lib.rs,call.rs,notify.rs,app.rs,config.rs}`、`crates/kivact-devtools-protocol/schema/*.json`、`crates/kivact-devtools-protocol/tests/protocol.rs`、`crates/kivact-devtools-client/tests/connection.rs`、`crates/kivact-core/src/devtools.rs`、`crates/kivact-core/src/devtools/{scripts.rs,target.rs}`、`crates/kivact-devtools/src/{panels.rs,panels/scripts.rs,app.rs,state.rs,session.rs}`、`crates/kivact-devtools/tests/{app.rs,app/panels/scripts.rs}`、`crates/kivact-devtools-mcp/{src/tools.rs,tests/mcp.rs}` | TL-09、DEC-10（親の決定） | M |
| T05 | W1b | T1 | T23 | `crates/kivact-ui/src/**` | TL-04 | L |
| T06 | W1b | T1 | T05 | `crates/kivact-interaction/src/**` | TL-04 | M |
| T07 | W1b | T2 | T23 | `crates/kivact-core/src/**`、`crates/kivact/src/**`、`crates/kivact-macros/src/**` | TL-04、TL-05、TL-06 | L |
| T08 | W1b | T2 | T07 | `crates/kivact-engine/src/**`、`crates/kivact-layout/src/**`、`crates/kivact-editing/src/**` | TL-04、TL-05 | L |
| T09 | W1b | T3 | T23 | `crates/kivact-runtime/src/**`、`crates/kivact-i18n/src/**`、`crates/kivact-style/src/**`、`crates/kivact-types/src/**`（`cell.rs` を除く）、`crates/kivact-shader/src/**`、`crates/kivact-svg/src/**`、`crates/kivact-gpu-share/src/**` | TL-04、TL-05 | L |
| T10 | W1b | T3 | T09 | `crates/arvane/src/**`、`crates/arvane-*/src/**`、`dev/bench/src/**`、`dev/tests/gpu-interop-producer/src/**` | TL-04 | M |
| T11 | W1b | T4 | T23 | `crates/kivact-ui-{basic,pop,game,creative,soft}/src/**` | TL-04、KIT-16 | L |
| T12 | W1b | T4 | T11 | `examples/**`（`examples/shared-assets` を除く）、`templates/**` | TL-04、4.3 | M |
| T13 | W1b | T5 | T23 | `crates/kivact-devtools-client/`（削除）、`crates/kivact-devtools-protocol/**`、`crates/kivact-devtools-agent/**`、`crates/kivact-devtools-mcp/**`、`crates/kivact-devtools/**`、`Cargo.toml` の `members`、`Cargo.lock`、`dev/tests/kivact-public-api/Cargo.toml`、`dev/tests/kivact-public-api/tests/e2e/devtools.rs` | TL-08、TL-14(3)、TL-04 | M |
| T14 | W1b | T5 | T13 | `dev/tests/kivact-public-api/tests/e2e/{flows,runtime,task,view,gpu_on_demand,gpu_on_demand_shared,gpu_on_demand_windows,theme_conformance}.rs`、`.../e2e/embed/windows.rs`、`crates/kivact-ui-{basic,pop,game,creative,soft}/README.md` | TL-14、KIT-16 | M |
| T15 | W1b | T5 | T14 | `dev/tests/kivact-public-api/**`、`dev/tests/kivact-kit-tests/`（新設）、`Cargo.toml` の `members`、`Cargo.lock`、`dev/xtask/policy/workspace.toml` の `[[style.exceptions]]` 1 件 | TL-15、6.4 | L |
| T16 | W1b | T6 | T23 | `crates/kivact-test/**` | TL-12、TL-04、TL-05 | M |
| T17 | W1b | T6 | T16 | `dev/tests/docs/**`、`docs/for-kivact-user/jp/how-to-guide/{canvas,windows}.md` | TL-03 | M |
| T18 | W1b | T6 | T17 | `docs/for-kivact-user/{en,jp}/how-to-guide/{devtools,README}.md`、`README.md`、`README_JP.md`、`docs/for-kivact-developer/strategy.md` | TL-09、TL-08 | S |
| T19 | W1b | T6 | T18 | `dev/tools/window-lock.sh`、`dev/tools/check-features.sh` | TL-01、TL-17 | S |
| T20 | W1b | T6 | T19 | `architecture/`（git から外す）、`.gitignore`、`README.md`、`README_JP.md` | TL-02、TL-16 | S |
| T21 | W1b | T6 | T20 | `CONTRIBUTING.md`、`dev/README.md`、`docs/for-kivact-developer/manual-verification.md` | TL-01、TL-16、6.4 | M |
| T22 | W1c | T0 | T05〜T21 全部 | `dev/xtask/policy/workspace.toml`、`Cargo.toml` の `[workspace.lints.rust]`、`redundant_imports` の警告が出たファイル（workspace 全体） | TL-04、4.3 | M |

W1b の lane とファイルの排他：T1＝ui・interaction、T2＝core・facade・macros・engine・layout・editing、T3＝runtime・i18n・style・types・shader・svg・gpu-share・arvane*・bench・producer、T4＝5 kit の `src`・examples・templates、T5＝DevTools の crate・`dev/tests/kivact-public-api`・新しい `dev/tests/kivact-kit-tests`・kit の README、T6＝`kivact-test`・`dev/tests/docs`・docs・README・scripts・architecture・CONTRIBUTING。二つの lane が同じファイルを持つのは root の `Cargo.toml` の `members` 配列と `Cargo.lock` だけで、これは T5 の中だけ（T13 と T15。同じ lane の順）。README.md・README_JP.md は T6 だけが持つ。

W1b の後に W1c（T22、単独の lane）を置く。W1c は W1b のすべての lane の merge の後に走る（`state` を pending から外すため。4.4 節）。

## 3. unit の手順

### 共通手順 M：`state-fields` の印の変換（T05〜T13、T15〜T17 が使う）

各 unit の「対象 package」の一つずつに、次を行う。`<P>` は package 名、`<DIRS>` はその unit の所有する `src` の directory（空白区切り）。

**M1. 新しい規則が報告するものを一覧にする。**

```bash
cargo xtask check --no-write --only state --strict --crate <P> 2>&1 | grep "state:"
```

出力の各行は次のどれかである（`<path>:<line>` は型・関数の位置）。

| 種類 | 文面 |
|---|---|
| 型 | `state: <path>:<line> (<P>::<module>): <Type> holds state fields a, b, c and is mutated (<理由>); close the combinations ...` |
| enum | `state: <path>:<line> (...): <Enum> has variants with state fields V.a, V.b and is mutated (...); ...` |
| 関数 | `state: <path>:<line> (...): function <name> combines state locals a, b; ...` |
| 旧い印 | `state: <path>:<line> (...): obsolete \`xtask-allow(state-fields)\` marker; ...` |

**M2. 「型」「enum」「関数」の各行に、印を 1 行ずつ書く。** 旧い印の行は M3 で一括して消すので、ここでは触らない。

1. 対象の item を開く。型と enum は `struct <Type>` / `enum <Enum>` の行、関数は `fn <name>` の行。
2. 印の文面を次の規則で作る。
   - `L` = 報告の行にある field の列（`a, b, c` や `V.a, V.b`、関数なら local の列）をそのまま写す。
   - `R` = その item の本体（型なら `{` から `}` まで、関数なら本体の中）にある**最初の** `// xtask-allow(state-fields):` の理由。行頭の `independent:`・`option:`・`cache:`・`identity:`・`statistic:`・`projection:` のどれかとその後の空白を取り、末尾の `.` を取る。本体に旧い印が無ければ `R` は無し。
   - `W` = 本体の旧い印のうち `projection:` で始まり `written only by ` を含むものごとに、`<その印の直後の field 名> is written only by <"written only by " の後から最初の ; , . までの語>`。複数あれば `; ` で繋ぐ。無ければ無し。
   - 文面：`W` が無ければ `// xtask-allow(state): independent: L; R`、`W` があれば `// xtask-allow(state): W; independent: L; R`。`R` が無ければ `; R` を付けない。
   - 行全体（字下げを除く）が 180 文字を超えたら、`R` を 180 文字に収まる最後の空白の位置で切る（`...` は付けない）。
3. 印を置く場所：その item の**一番上の行**の直前。一番上の行とは、item の直前に続く `///` の doc の行と `#[` で始まる属性の行のうち最も上の行（無ければ `struct`・`enum`・`fn` の行そのもの）。字下げは item の行と同じにする。既に `// xtask-allow(...)` の別の印（`api-shape` など）がその上に並んでいれば、その印たちの直前でも直後でもよい（検査はどちらも認める）。

例（`kivact-ui/src/values.rs` の `ButtonState` が報告された場合。本体の最初の旧い印が `independent: busy comes from the caller's busy value and is valid with any icon or label layout`）：

```rust
// xtask-allow(state): independent: busy, expanded, leading_icon, trailing_icon, label_hidden; busy comes from the caller's busy value and is valid with any icon or label layout
/// The state a button's recipe reads.
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonState {
```

**M3. 旧い印を一括して消す。**

```bash
rg -l "xtask-allow\(state-fields\)" <DIRS> | xargs -r perl -ni -e 'print unless /^\s*\/\/ xtask-allow\(state-fields\):/'
```

**M4. 確かめる。**

```bash
cargo xtask check --no-write --only state --strict --crate <P>
```

`ok` が出るまで直す。残った行の扱い：

- `unused marker` → M2 で置いた場所が item の一番上の行の直前になっていない。置き直す。報告に無い item に印を書いていれば、その印を消す。
- 型・関数の行 → M2 の漏れ。M2 を行う。
- 上のどれでもない行 → STOP（`STOP: <unit> <path>:<item> state の報告が共通手順 M に無い形`）。

**M5.** `cargo fmt -p <P>`（印は comment だけなので fmt は字下げだけを直す）。

作業指示書 0.4 節の STOP の条件「xtask の `state` 検査が、付属書に無い型を報告した」は、共通手順 M を行う unit（T05〜T13、T15〜T17）には適用しない。これらの unit は報告されたすべての型・関数を M2 で閉じる。

---

## T01：xtask の `state` 検査（`state-fields` の置換え）と pending の仕組み

- **wave / lane / 前提**：W1a / T0 / —
- **所有**：`dev/xtask/src/state.rs`（新設）、`dev/xtask/src/state/`（新設：`check.rs`、`fields.rs`、`index.rs`、`locals.rs`、`tests.rs`）、`dev/xtask/src/state_fields.rs` と `dev/xtask/src/state_fields/`（削除）、`dev/xtask/src/pending.rs`（新設）、`dev/xtask/src/local_checks.rs`、`dev/xtask/src/check_cli.rs`、`dev/xtask/src/check_source.rs`、`dev/xtask/src/main.rs`、`dev/xtask/src/policy.rs`、`dev/xtask/src/public_surface.rs`、`dev/xtask/policy/workspace.toml`、`dev/xtask/state-audit.md`（未追跡の生成物。あれば消す）
- **目的**：設計 6.1 の `state` 検査を実装し、`state-fields`（field ごとの印、`independent:` の重複検査、`--report-audit`、`state-audit.md`）を消す（TL-04）。設計 6.3 の検査を「報告のみ」にする `[rework.pending]` と `--strict`、tier a を速くする `--crate`（局所の検査だけ）と `--only local` を入れる。
- **手順**：

1. 消す：`git rm -r -q dev/xtask/src/state_fields dev/xtask/src/state_fields.rs`、`rm -f dev/xtask/state-audit.md`。

2. `dev/xtask/src/check_source.rs`：
   - `impl Source` の `marker_line` と `allow_reason` の二つの method（doc comment ごと）を消す（`state_fields` だけが使っていた）。
   - ファイルの末尾の `pub(crate) fn type_names` の後に次を足す。

   ```rust
   /// The last path segment of an impl's self type (`Foo` for `crate::a::Foo<T>`).
   pub(crate) fn self_type_name(ty: &syn::Type) -> Option<String> {
       match ty {
           syn::Type::Path(path) => path
               .path
               .segments
               .last()
               .map(|segment| segment.ident.to_string()),
           syn::Type::Reference(reference) => self_type_name(&reference.elem),
           syn::Type::Paren(paren) => self_type_name(&paren.elem),
           syn::Type::Group(group) => self_type_name(&group.elem),
           _ => None,
       }
   }
   ```
   - `mod tests` の `loads_library_and_binary_targets_without_mixing_type_identities` を直す：fixture の文字列の中の `a: bool, b: bool` を 5 か所とも `a: Cell<bool>, b: Cell<bool>` にする（`src/lib.rs`、`src/main.rs`、bin 3 つの文字列）。`only: Some("state-fields".into())` を `only: Some("state".into())` にする。最後の `assert_eq!(..., 4)` の `4` を `5` にする（lib と main の `State`、bin の `Model` 3 つ。`impl Element for` はもう対象外にしない）。

3. `dev/xtask/src/state.rs` を新設する。

   ```rust
   //! Role: Find mutated types and functions that hold two or more state fields or locals.
   //!
   //! A struct, or the variants of an enum, with two or more fields of kind
   //! `bool`, `Option`, named counter, `Cell`, `RefCell`, `Mutex`, `RwLock`,
   //! `Atomic*`, `Signal` or `Var` needs one `// xtask-allow(state): <how the
   //! combinations are closed>` line before the type when the type is mutated:
   //! an impl of its package has a `&mut self` method, it has an
   //! interior-mutable field, or one of its methods writes a `Signal`/`Var`
   //! field. Fields written only by constructors, `Default` and by-value
   //! builders are a passive record and need nothing. Names ending in the
   //! policy's `exempt_suffixes`, `*State` in `recipe_state_packages`, and
   //! `Hash + Eq` keys are not checked. A function with two or more state
   //! locals needs the same line before the function (design 6.1).

   mod check;
   mod fields;
   mod index;
   mod locals;
   #[cfg(test)]
   mod tests;

   pub(crate) use check::check;
   pub(crate) use index::Index;
   ```

4. `dev/xtask/src/state/fields.rs` を新設する。

   ```rust
   //! Role: Classify one field's type as plain state, interior-mutable state, a handle or nothing.

   /// How one field takes part in the state of its type.
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub(super) enum Kind {
       /// A value, a collection, an enum or a callback setting.
       None,
       /// `bool`, `Option<_>` or a named integer counter.
       Plain,
       /// `Cell`, `RefCell`, `Mutex`, `RwLock` or `Atomic*`, written through `&self`.
       Interior,
       /// `Signal` or `Var`: state when a method of the type writes it.
       Handle,
   }

   /// One counted field: its label in reports, its field name and its kind.
   pub(super) struct Counted {
       pub(super) label: String,
       pub(super) field: String,
       pub(super) kind: Kind,
   }

   /// The counted fields of a struct or a variant; labels start with `prefix.` when it is not empty.
   pub(super) fn counted(fields: &syn::Fields, prefix: &str) -> Vec<Counted> {
       let mut out = Vec::new();
       for (index, field) in fields.iter().enumerate() {
           let name = field
               .ident
               .as_ref()
               .map_or_else(|| index.to_string(), ToString::to_string);
           let label = if prefix.is_empty() {
               name.clone()
           } else {
               format!("{prefix}.{name}")
           };
           if let syn::Type::Tuple(tuple) = &field.ty {
               for (element, ty) in tuple.elems.iter().enumerate() {
                   push(&mut out, format!("{label}.{element}"), &name, ty);
               }
           } else {
               push(&mut out, label, &name, &field.ty);
           }
       }
       out
   }

   fn push(out: &mut Vec<Counted>, label: String, field: &str, ty: &syn::Type) {
       let kind = kind(field, ty);
       if kind != Kind::None {
           out.push(Counted {
               label,
               field: field.to_owned(),
               kind,
           });
       }
   }

   /// The kind of a field named `name` of type `ty`.
   fn kind(name: &str, ty: &syn::Type) -> Kind {
       let Some(segment) = last_segment(peel(ty)) else {
           return Kind::None;
       };
       let outer = segment.ident.to_string();
       match outer.as_str() {
           "Cell" | "RefCell" | "Mutex" | "RwLock" => Kind::Interior,
           "Signal" | "Var" => Kind::Handle,
           "bool" => Kind::Plain,
           "Option" if single_argument(segment).is_some_and(is_setting) => Kind::None,
           "Option" => Kind::Plain,
           _ if outer.starts_with("Atomic") => Kind::Interior,
           _ if is_integer(&outer) && is_counter_name(name) => Kind::Plain,
           _ => Kind::None,
       }
   }

   /// `ty` without references, parentheses and `Rc`/`Arc`/`Box` wrappers.
   fn peel(ty: &syn::Type) -> &syn::Type {
       match ty {
           syn::Type::Reference(reference) => peel(&reference.elem),
           syn::Type::Paren(paren) => peel(&paren.elem),
           syn::Type::Group(group) => peel(&group.elem),
           syn::Type::Path(path) => match path.path.segments.last() {
               Some(segment)
                   if matches!(segment.ident.to_string().as_str(), "Rc" | "Arc" | "Box") =>
               {
                   single_argument(segment).map_or(ty, peel)
               }
               _ => ty,
           },
           _ => ty,
       }
   }

   fn last_segment(ty: &syn::Type) -> Option<&syn::PathSegment> {
       match ty {
           syn::Type::Path(path) => path.path.segments.last(),
           _ => None,
       }
   }

   fn single_argument(segment: &syn::PathSegment) -> Option<&syn::Type> {
       let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
           return None;
       };
       if arguments.args.len() != 1 {
           return None;
       }
       match arguments.args.first()? {
           syn::GenericArgument::Type(ty) => Some(ty),
           _ => None,
       }
   }

   /// A callback or a view factory: an `Option` of one is a setting, not state.
   fn is_setting(ty: &syn::Type) -> bool {
       match peel(ty) {
           syn::Type::TraitObject(object) => object.bounds.iter().any(|bound| {
               matches!(bound, syn::TypeParamBound::Trait(bound)
                   if bound.path.segments.last().is_some_and(|segment| {
                       matches!(segment.ident.to_string().as_str(), "Fn" | "FnMut" | "FnOnce")
                   }))
           }),
           ty => last_segment(ty).is_some_and(|segment| {
               matches!(
                   segment.ident.to_string().as_str(),
                   "Callback" | "Handler" | "ViewFn"
               )
           }),
       }
   }

   fn is_counter_name(name: &str) -> bool {
       matches!(
           name,
           "generation" | "epoch" | "serial" | "seq" | "token" | "next"
       ) || name.starts_with("next_")
           || name.ends_with("_count")
           || name.starts_with("pending_")
   }

   fn is_integer(name: &str) -> bool {
       matches!(
           name,
           "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize"
       )
   }
   ```

5. `dev/xtask/src/state/index.rs` を新設する。

   ```rust
   //! Role: Index which types of each package their own methods mutate after construction.
   //!
   //! Types are keyed by `package::Name`: two types of one name in one package
   //! share their facts (an approximation the rule accepts).

   use crate::check_source::{Source, self_type_name};
   use std::collections::BTreeSet;
   use syn::visit::Visit;

   /// Method calls that write the value they are called on.
   const WRITE_METHODS: [&str; 10] = [
       "set",
       "update",
       "replace",
       "take",
       "swap",
       "set_untracked",
       "update_untracked",
       "set_if_changed",
       "borrow_mut",
       "get_mut",
   ];

   /// The mutation facts of the workspace.
   #[derive(Default)]
   pub(crate) struct Index {
       /// `package::Type` of every type that an impl of its package gives a `&mut self` method.
       mutated: BTreeSet<String>,
       /// `package::Type::field` for every `self.field.<write>(..)` in a method of the type.
       written: BTreeSet<String>,
   }

   impl Index {
       pub(crate) fn build(sources: &[Source]) -> Self {
           let mut index = Self::default();
           for source in sources {
               for item in &source.items {
                   if let syn::Item::Impl(item) = item {
                       index.add_impl(&source.package, item);
                   }
               }
           }
           index
       }

       fn add_impl(&mut self, package: &str, item: &syn::ItemImpl) {
           let Some(name) = self_type_name(&item.self_ty) else {
               return;
           };
           let key = format!("{package}::{name}");
           for member in &item.items {
               let syn::ImplItem::Fn(method) = member else {
                   continue;
               };
               let Some(receiver) = method.sig.receiver() else {
                   continue;
               };
               if matches!(receiver.ty.as_ref(), syn::Type::Reference(reference) if reference.mutability.is_some())
               {
                   self.mutated.insert(key.clone());
               }
               if receiver.reference.is_none() && returns_self(&method.sig, &name) {
                   continue;
               }
               let mut writes = SelfWrites::default();
               writes.visit_block(&method.block);
               for field in writes.fields {
                   self.written.insert(format!("{key}::{field}"));
               }
           }
       }

       pub(crate) fn is_mutated(&self, package: &str, name: &str) -> bool {
           self.mutated.contains(&format!("{package}::{name}"))
       }

       pub(crate) fn is_written(&self, package: &str, name: &str, field: &str) -> bool {
           self.written.contains(&format!("{package}::{name}::{field}"))
       }
   }

   /// A by-value builder: `fn x(mut self, ..) -> Self`.
   fn returns_self(signature: &syn::Signature, name: &str) -> bool {
       let syn::ReturnType::Type(_, ty) = &signature.output else {
           return false;
       };
       self_type_name(ty).is_some_and(|returned| returned == "Self" || returned == name)
   }

   /// The fields of `self` that one method body writes.
   #[derive(Default)]
   struct SelfWrites {
       fields: BTreeSet<String>,
   }

   impl<'ast> Visit<'ast> for SelfWrites {
       fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
           if WRITE_METHODS.contains(&call.method.to_string().as_str())
               && let Some(field) = self_field(&call.receiver)
           {
               self.fields.insert(field);
           }
           syn::visit::visit_expr_method_call(self, call);
       }
   }

   /// `x` when `expr` is `self.x`, `self.x.y`, `&self.x` or `(self.x)`.
   fn self_field(expr: &syn::Expr) -> Option<String> {
       match expr {
           syn::Expr::Field(field) => match field.base.as_ref() {
               syn::Expr::Path(path) if path.path.is_ident("self") => match &field.member {
                   syn::Member::Named(name) => Some(name.to_string()),
                   syn::Member::Unnamed(index) => Some(index.index.to_string()),
               },
               base => self_field(base),
           },
           syn::Expr::Reference(reference) => self_field(&reference.expr),
           syn::Expr::Paren(paren) => self_field(&paren.expr),
           _ => None,
       }
   }
   ```

6. `dev/xtask/src/state/check.rs` を新設する。

   ```rust
   //! Role: Report mutated types and functions that combine state without a closing marker.

   use super::fields::{Counted, Kind, counted};
   use super::index::Index;
   use super::locals;
   use crate::check_policy::strings;
   use crate::check_source::Source;
   use syn::spanned::Spanned;

   /// The rule name and the name of its marker.
   const RULE: &str = "state";
   /// The marker of the replaced rule; every one left is an error.
   const OBSOLETE: &str = "// xtask-allow(state-fields):";

   /// Run the rule over one source.
   pub(crate) fn check(
       source: &Source,
       policy: &toml::Table,
       index: &Index,
       errors: &mut Vec<String>,
   ) {
       for item in &source.items {
           match item {
               syn::Item::Struct(item) => check_struct(source, policy, index, item, errors),
               syn::Item::Enum(item) => check_enum(source, policy, index, item, errors),
               _ => {}
           }
       }
       locals::check(source, errors);
       report_markers(source, errors);
   }

   fn check_struct(
       source: &Source,
       policy: &toml::Table,
       index: &Index,
       item: &syn::ItemStruct,
       errors: &mut Vec<String>,
   ) {
       let name = item.ident.to_string();
       if exempt(source, policy, &name, &item.attrs) {
           return;
       }
       let fields = counted(&item.fields, "");
       if fields.len() < 2 {
           return;
       }
       if let Some(why) = mutation(source, index, &name, &fields) {
           let what = format!("{name} holds state fields {}", labels(&fields));
           close(source, item.span(), &name, &what, &why, errors);
       }
   }

   fn check_enum(
       source: &Source,
       policy: &toml::Table,
       index: &Index,
       item: &syn::ItemEnum,
       errors: &mut Vec<String>,
   ) {
       let name = item.ident.to_string();
       if exempt(source, policy, &name, &item.attrs) {
           return;
       }
       let mut fields = Vec::new();
       for variant in &item.variants {
           let variant_fields = counted(&variant.fields, &variant.ident.to_string());
           if variant_fields.len() >= 2 {
               fields.extend(variant_fields);
           }
       }
       if fields.is_empty() {
           return;
       }
       if let Some(why) = mutation(source, index, &name, &fields) {
           let what = format!("{name} has variants with state fields {}", labels(&fields));
           close(source, item.span(), &name, &what, &why, errors);
       }
   }

   /// Why the type counts as mutated after construction; `None` for a passive record.
   fn mutation(source: &Source, index: &Index, name: &str, fields: &[Counted]) -> Option<String> {
       if index.is_mutated(&source.package, name) {
           return Some("a `&mut self` method".to_owned());
       }
       if let Some(field) = fields.iter().find(|field| field.kind == Kind::Interior) {
           return Some(format!("interior-mutable field `{}`", field.label));
       }
       fields
           .iter()
           .find(|field| {
               field.kind == Kind::Handle && index.is_written(&source.package, name, &field.field)
           })
           .map(|field| format!("its methods write `{}`", field.label))
   }

   fn close(
       source: &Source,
       span: proc_macro2::Span,
       name: &str,
       what: &str,
       why: &str,
       errors: &mut Vec<String>,
   ) {
       match source.allow_line(RULE, span) {
           Some(line) => source.use_allow(line),
           None => source.error_at(
               RULE,
               span,
               &format!(
                   "{what} and is mutated ({why}); close the combinations with an enum, or put one `// xtask-allow(state): <how the combinations are closed>` line before `{name}`"
               ),
               errors,
           ),
       }
   }

   fn labels(fields: &[Counted]) -> String {
       fields
           .iter()
           .map(|field| field.label.as_str())
           .collect::<Vec<_>>()
           .join(", ")
   }

   /// Passive roles by name (design 6.1 (c)) and map keys (`Hash + Eq`).
   fn exempt(source: &Source, policy: &toml::Table, name: &str, attrs: &[syn::Attribute]) -> bool {
       let suffix = strings(policy, "exempt_suffixes")
           .iter()
           .any(|suffix| name.ends_with(suffix.as_str()));
       let recipe_state = name.ends_with("State")
           && strings(policy, "recipe_state_packages").contains(&source.package);
       suffix || recipe_state || (derives(attrs, "Hash") && derives(attrs, "Eq"))
   }

   fn derives(attrs: &[syn::Attribute], wanted: &str) -> bool {
       attrs
           .iter()
           .filter(|attribute| attribute.path().is_ident("derive"))
           .any(|attribute| {
               let mut found = false;
               let _ = attribute.parse_nested_meta(|meta| {
                   if meta.path.segments.last().is_some_and(|segment| segment.ident == wanted) {
                       found = true;
                   }
                   Ok(())
               });
               found
           })
   }

   /// Report the replaced rule's markers and `state` markers no counted item used.
   fn report_markers(source: &Source, errors: &mut Vec<String>) {
       let used = source.used_allows();
       for (index, line) in source.text.lines().enumerate() {
           let number = index + 1;
           if !source.owns_line(number) {
               continue;
           }
           let line = line.trim_start();
           if line.starts_with(OBSOLETE) {
               source.error_at_line(
                   RULE,
                   number,
                   "obsolete `xtask-allow(state-fields)` marker; delete it (the `state` rule takes one marker per type)",
                   errors,
               );
           } else if line.starts_with("// xtask-allow(state):") && !used.contains(&number) {
               source.error_at_line(
                   RULE,
                   number,
                   "unused marker: no mutated state combination follows it",
                   errors,
               );
           }
       }
   }
   ```

7. `dev/xtask/src/state/locals.rs` を新設する。中身は削除前の `dev/xtask/src/state_fields/locals.rs`（`git show HEAD:dev/xtask/src/state_fields/locals.rs` で読める）を写し、次を変える。
   - 先頭の `//! Role:` を `//! Role: Find functions that keep two or more state locals.` にする。`use super::audit::{AuditRow, marker_closure};` を消す。
   - `Local` から `marker` field を消す：`pub(super) struct Local { pub(super) line: usize, pub(super) name: String }`（doc は `/// One detected local: its line and name.`）。
   - `Function` に field `/// The line of the \`// xtask-allow(state):\` marker before the function.\n pub(super) marker: Option<usize>,` を `name` の後に足す。
   - `pub(super) fn scan` を `fn scan` にする。`collect` と `audit` の二つの関数を消す。
   - `check` を次に置き換える。

   ```rust
   /// Report functions with two or more state locals and no marker; record the markers used.
   pub(super) fn check(source: &Source, errors: &mut Vec<String>) {
       for function in scan(source) {
           if function.locals.len() < 2 {
               continue;
           }
           match function.marker {
               Some(line) => source.use_allow(line),
               None => {
                   let names = function
                       .locals
                       .iter()
                       .map(|local| local.name.as_str())
                       .collect::<Vec<_>>()
                       .join(", ");
                   source.error_at_line(
                       "state",
                       function.locals[0].line,
                       &format!(
                           "function {} combines state locals {names}; close them with an enum, or put one `// xtask-allow(state): <how the combinations are closed>` line before the function",
                           function.name
                       ),
                       errors,
                   );
               }
           }
       }
   }
   ```
   - `visit_item_fn` と `visit_impl_item_fn` の `self.stack.push(Function { .. })` に `marker: self.source.allow_line("state", node.span()),` を足す。
   - `visit_local` の中の `let marker = self.source.allow_line("state-fields", local.span());` から `use_allow` までの 4 行を消し、`function.locals.push(Local { line, name, marker })` を `function.locals.push(Local { line, name })` にする。

8. `dev/xtask/src/state/tests.rs` を新設する。

   ```rust
   //! Role: Verify the state rule on passive records, mutated types, markers and locals.

   fn run(text: &str, policy: &str) -> Vec<String> {
       let source = crate::check_source::fixture(text);
       let index = super::Index::build(std::slice::from_ref(&source));
       let policy: toml::Table = policy.parse().expect("the fixture policy parses");
       let mut errors = Vec::new();
       super::check(&source, &policy, &index, &mut errors);
       errors
   }

   #[test]
   fn passive_records_need_no_marker() {
       let text = "struct Record { a: bool, b: Option<u8> }\nimpl Record { fn new() -> Self { Self { a: false, b: None } } fn a(mut self, a: bool) -> Self { self.a = a; self } }";
       assert!(run(text, "").is_empty());
   }

   #[test]
   fn a_mut_self_method_makes_fields_state() {
       let errors = run(
           "struct Model { open: bool, pending: Option<u8> }\nimpl Model { fn open(&mut self) { self.open = true; } }",
           "",
       );
       assert_eq!(errors.len(), 1, "{errors:?}");
       assert!(errors[0].contains("Model holds state fields open, pending"), "{errors:?}");
   }

   #[test]
   fn an_interior_mutable_field_makes_fields_state() {
       let errors = run("struct Cache { hit: Cell<bool>, last: Option<u8> }", "");
       assert_eq!(errors.len(), 1, "{errors:?}");
       assert!(errors[0].contains("interior-mutable field `hit`"), "{errors:?}");
   }

   #[test]
   fn a_signal_field_is_state_only_when_a_method_writes_it() {
       let written = "struct Handle { open: Signal<bool>, busy: Signal<bool> }\nimpl Handle { fn show(&self) { self.open.set(true); } }";
       assert_eq!(run(written, "").len(), 1);
       let read = "struct Handle { open: Signal<bool>, busy: Signal<bool> }\nimpl Handle { fn open(self) -> Signal<bool> { self.open } }";
       assert!(run(read, "").is_empty());
   }

   #[test]
   fn one_marker_before_the_type_closes_it_and_an_unused_one_is_an_error() {
       let closed = "// xtask-allow(state): independent: open, pending; probe\nstruct Model { open: bool, pending: Option<u8> }\nimpl Model { fn open(&mut self) {} }";
       assert!(run(closed, "").is_empty());
       let unused = run("// xtask-allow(state): independent: a; probe\nstruct Plain { a: bool }", "");
       assert_eq!(unused.len(), 1, "{unused:?}");
       assert!(unused[0].contains("unused marker"), "{unused:?}");
   }

   #[test]
   fn exempt_names_and_hash_keys_need_no_marker() {
       let text = "struct ListApi { a: Cell<bool>, b: Cell<bool> }\n#[derive(Clone, PartialEq, Eq, Hash)]\nstruct Slot { a: bool, b: Option<u8> }\nimpl Slot { fn f(&mut self) {} }";
       assert!(run(text, "exempt_suffixes = [\"Api\"]").is_empty());
   }

   #[test]
   fn recipe_states_are_exempt_only_in_recipe_packages() {
       let text = "struct ButtonState { a: Cell<bool>, b: Cell<bool> }";
       assert!(run(text, "recipe_state_packages = [\"kivact-ui-soft\"]").is_empty());
       assert_eq!(run(text, "").len(), 1);
   }

   #[test]
   fn enum_variants_need_one_marker_on_the_enum() {
       let errors = run("enum Mode { Open { a: Cell<bool>, b: Option<u8> }, Closed }", "");
       assert_eq!(errors.len(), 1, "{errors:?}");
       assert!(errors[0].contains("Mode has variants with state fields Open.a, Open.b"), "{errors:?}");
       let closed = "// xtask-allow(state): independent: Open.a, Open.b; probe\nenum Mode { Open { a: Cell<bool>, b: Option<u8> }, Closed }";
       assert!(run(closed, "").is_empty());
   }

   #[test]
   fn state_locals_need_one_marker_on_the_function() {
       let errors = run("fn sample() {\nlet a = signal(false);\nlet b = signal(None);\n}", "");
       assert_eq!(errors.len(), 1, "{errors:?}");
       assert!(errors[0].contains("function sample combines state locals a, b"), "{errors:?}");
       let closed = "// xtask-allow(state): independent: a, b; probe\nfn sample() {\nlet a = signal(false);\nlet b = signal(None);\n}";
       assert!(run(closed, "").is_empty());
   }

   #[test]
   fn the_replaced_marker_is_an_error() {
       let errors = run("struct Fact {\n// xtask-allow(state-fields): independent: probe\na: bool,\n}", "");
       assert_eq!(errors.len(), 1, "{errors:?}");
       assert!(errors[0].contains("obsolete"), "{errors:?}");
   }

   #[test]
   fn callback_options_and_values_are_not_state() {
       let text = "struct Row { on_press: Option<Callback<()>>, render: Option<Box<dyn Fn()>>, title: String, open: bool }\nimpl Row { fn f(&mut self) {} }";
       assert!(run(text, "").is_empty());
   }

   #[test]
   fn tuple_elements_count_one_by_one() {
       let errors = run("struct Pair { values: (bool, Option<u8>) }\nimpl Pair { fn f(&mut self) {} }", "");
       assert_eq!(errors.len(), 1, "{errors:?}");
       assert!(errors[0].contains("values.0, values.1"), "{errors:?}");
   }
   ```

9. `dev/xtask/src/pending.rs` を新設する。

   ```rust
   //! Role: Read the rework's pending table: rules whose findings in listed packages only print.
   //!
   //! `[rework.pending]` maps a local rule to the packages whose findings are
   //! notes until the units that fix them land; `--strict` makes them errors.
   //! The table exists only during the rework (REWORK_DESIGN.md); annex G's last
   //! unit deletes it with this module.

   use std::collections::{BTreeMap, BTreeSet};

   /// The packages each pending rule only reports on.
   #[derive(Default)]
   pub(crate) struct Pending {
       rules: BTreeMap<String, BTreeSet<String>>,
   }

   impl Pending {
       pub(crate) fn parse(policy: &toml::Table) -> Result<Self, String> {
           let mut pending = Self::default();
           let Some(rework) = policy.get("rework") else {
               return Ok(pending);
           };
           let rework = rework.as_table().ok_or("rework: expected a table")?;
           for (key, value) in rework {
               if key != "pending" {
                   return Err(format!("rework: unknown setting {key}"));
               }
               let table = value.as_table().ok_or("rework.pending: expected a table")?;
               for (rule, packages) in table {
                   if !crate::check_cli::RULES.contains(&rule.as_str()) {
                       return Err(format!("rework.pending: unknown rule {rule}"));
                   }
                   let expected = || format!("rework.pending.{rule}: expected an array of package names");
                   let names = packages
                       .as_array()
                       .ok_or_else(expected)?
                       .iter()
                       .map(|name| name.as_str().map(str::to_owned).ok_or_else(expected))
                       .collect::<Result<BTreeSet<_>, _>>()?;
                   pending.rules.insert(rule.clone(), names);
               }
           }
           Ok(pending)
       }

       pub(crate) fn covers(&self, rule: &str, package: &str) -> bool {
           self.rules
               .get(rule)
               .is_some_and(|packages| packages.contains(package))
       }
   }
   ```

10. `dev/xtask/src/check_cli.rs` を次の内容に置き換える。

   ```rust
   //! Role: Parse local check selection independently of the shared xtask CLI.

   /// The local rules, in report order.
   pub(crate) const RULES: [&str; 9] = [
       "state",
       "effect-writes",
       "api-shape",
       "canonical-path",
       "name-collision",
       "kit-literal-color",
       "os-branch",
       "structural-options",
       "devtools-codegen",
   ];

   /// The `--only` value that selects every local rule.
   const ALL: &str = "local";

   #[derive(Default, Debug)]
   pub(crate) struct Options {
       pub(crate) only: Option<String>,
       pub(crate) package: Option<String>,
       pub(crate) module: Option<String>,
       /// Print errors as notes and succeed.
       pub(crate) report: bool,
       /// Report pending rules as errors too (`[rework.pending]`).
       pub(crate) strict: bool,
   }

   impl Options {
       pub(crate) fn parse(arguments: Vec<String>) -> Result<(Self, Vec<String>), String> {
           let mut result = Self::default();
           let mut shared = Vec::new();
           let mut args = arguments.into_iter();
           while let Some(arg) = args.next() {
               match arg.as_str() {
                   "--only" | "--crate" | "--module" => {
                       let value = args
                           .next()
                           .filter(|v| !v.starts_with('-'))
                           .ok_or_else(|| format!("{arg} requires a value"))?;
                       let slot = match arg.as_str() {
                           "--only" => &mut result.only,
                           "--crate" => &mut result.package,
                           _ => &mut result.module,
                       };
                       if slot.replace(value).is_some() {
                           return Err(format!("duplicate {arg}"));
                       }
                   }
                   "--report" => result.report = true,
                   "--strict" => result.strict = true,
                   _ => shared.push(arg),
               }
           }
           if let Some(rule) = &result.only
               && rule != ALL
               && !rule.split(',').all(|name| RULES.contains(&name))
           {
               return Err(format!("unknown check: {rule}"));
           }
           if result.module.is_some() && result.package.is_none() {
               return Err("--module requires --crate".into());
           }
           Ok((result, shared))
       }

       pub(crate) fn selected(&self, rule: &str) -> bool {
           self.only
               .as_deref()
               .is_none_or(|only| only == ALL || only.split(',').any(|name| name == rule))
       }
   }

   #[cfg(test)]
   mod tests {
       //! Role: Verify local selectors do not escape into the shared parser.
       use super::Options;
       #[test]
       fn parses_selection_and_rejects_invalid_scopes() {
           let (options, rest) = Options::parse(
               [
                   "check", "--only", "state", "--crate", "demo", "--module", "state", "--report",
                   "--strict", "--no-write",
               ]
               .map(String::from)
               .to_vec(),
           )
           .unwrap();
           let (selected, _) =
               Options::parse(vec!["--only".into(), "api-shape,canonical-path".into()]).unwrap();
           assert!(selected.selected("api-shape"));
           assert!(selected.selected("canonical-path"));
           assert!(!selected.selected("state"));
           let (all, _) = Options::parse(vec!["--only".into(), "local".into()]).unwrap();
           assert!(all.selected("state") && all.selected("os-branch"));
           assert!(Options::parse(vec!["--only".into(), "api-shape,unknown".into()]).is_err());
           assert!(options.report);
           assert!(options.strict);
           assert_eq!(options.package.as_deref(), Some("demo"));
           assert_eq!(rest, ["check", "--no-write"]);
           assert!(Options::parse(vec!["--module".into(), "state".into()]).is_err());
           assert!(Options::parse(vec!["--only".into(), "unknown".into()]).is_err());
       }
   }
   ```

11. `dev/xtask/src/local_checks.rs`：ファイルの先頭から `fn run_rule` の終わりの `}` まで（`write_audit`、`selected`、`validate_policy`、`model_types`、`run_rule` を含む）を次に置き換える。その後にある `fn selected_source` と `mod tests` は残す（`selected_source` の本体は変えない）。

   ```rust
   //! Role: Run the workspace's local architectural checks under shared selectors.
   use crate::check_cli::{Options, RULES};
   use crate::check_policy::is_kit;
   use crate::pending::Pending;
   use std::path::Path;

   /// The findings of one run: errors fail it; notes are findings of a pending rule.
   pub(crate) struct Outcome {
       pub(crate) errors: Vec<String>,
       pub(crate) notes: Vec<String>,
   }

   pub(crate) fn check(root: &Path, policy: &Path, options: &Options) -> Result<Outcome, String> {
       let text = std::fs::read_to_string(policy).map_err(|e| format!("{}: {e}", policy.display()))?;
       let policy: toml::Table = text.parse().map_err(|e| format!("{e}"))?;
       validate_policy(&policy)?;
       let pending = Pending::parse(&policy)?;
       let sources = crate::check_source::load(root)?;
       if let Some(package) = &options.package
           && !sources.iter().any(|s| s.package == *package)
       {
           return Err(format!("unknown crate: {package}"));
       }
       let active = |rule: &str| {
           options.selected(rule)
               && sources.iter().any(|source| {
                   let kit_only = matches!(rule, "kit-literal-color" | "name-collision");
                   (!kit_only || is_kit(&source.package)) && selected_source(source, options)
               })
       };
       let mut outcome = Outcome {
           errors: Vec::new(),
           notes: Vec::new(),
       };
       if !RULES.iter().any(|rule| active(rule)) {
           return Ok(outcome);
       }
       let public_needed = [
           "api-shape",
           "canonical-path",
           "name-collision",
           "structural-options",
       ]
       .iter()
       .any(|rule| active(rule));
       let surface = crate::public_surface::build_for(&sources, public_needed);
       let index = crate::state::Index::build(&sources);
       let scope = Scope {
           sources: &sources,
           surface: &surface,
           index: &index,
           policy: &policy,
           pending: &pending,
       };
       scope.run(root, options, &mut outcome);
       for list in [&mut outcome.errors, &mut outcome.notes] {
           list.sort();
           list.dedup();
       }
       Ok(outcome)
   }

   /// The loaded workspace facts one check run shares across rules.
   struct Scope<'a> {
       sources: &'a [crate::check_source::Source],
       surface: &'a crate::public_surface::Surface,
       index: &'a crate::state::Index,
       policy: &'a toml::Table,
       pending: &'a Pending,
   }

   impl Scope<'_> {
       fn run(&self, root: &Path, options: &Options, outcome: &mut Outcome) {
           let empty = toml::Table::new();
           for rule in RULES {
               if !options.selected(rule) {
                   continue;
               }
               if let Some(package) = workspace_rule_package(rule) {
                   if options
                       .package
                       .as_deref()
                       .is_none_or(|selected| selected == package)
                   {
                       let mut found = Vec::new();
                       run_workspace_rule(rule, root, &mut found);
                       self.route(rule, package, found, options, outcome);
                   }
                   continue;
               }
               let settings = self
                   .policy
                   .get(&rule.replace('-', "_"))
                   .and_then(toml::Value::as_table)
                   .unwrap_or(&empty);
               for source in self.sources {
                   if !selected_source(source, options) {
                       continue;
                   }
                   let mut found = Vec::new();
                   self.run_rule(rule, source, settings, &mut found);
                   self.route(rule, &source.package, found, options, outcome);
               }
           }
       }

       /// File the findings of `rule` in `package` as notes while the rule is pending there.
       fn route(
           &self,
           rule: &str,
           package: &str,
           found: Vec<String>,
           options: &Options,
           outcome: &mut Outcome,
       ) {
           if !options.strict && self.pending.covers(rule, package) {
               outcome
                   .notes
                   .extend(found.into_iter().map(|finding| format!("pending: {finding}")));
           } else {
               outcome.errors.extend(found);
           }
       }

       fn run_rule(
           &self,
           rule: &str,
           source: &crate::check_source::Source,
           settings: &toml::Table,
           errors: &mut Vec<String>,
       ) {
           match rule {
               "state" => crate::state::check(source, settings, self.index, errors),
               "effect-writes" => crate::effect_writes::check(source, errors),
               "api-shape" => crate::api_shape::check(source, self.surface, errors),
               "canonical-path" => crate::canonical_path::check(source, self.surface, errors),
               "name-collision" => crate::name_collision::check(source, self.surface, errors),
               "kit-literal-color" => crate::kit_literal_color::check(source, settings, errors),
               "os-branch" => crate::os_branch::check(source, settings, errors),
               "structural-options" => {
                   crate::structural_options::check(source, settings, self.surface, errors)
               }
               _ => unreachable!("invariant: RULES contains only implemented checks"),
           }
       }
   }

   /// The package a workspace-wide rule reports for; `--crate` selects it by this name.
   fn workspace_rule_package(rule: &str) -> Option<&'static str> {
       match rule {
           "devtools-codegen" => Some("kivact-devtools-protocol"),
           _ => None,
       }
   }

   fn run_workspace_rule(rule: &str, root: &Path, errors: &mut Vec<String>) {
       match rule {
           "devtools-codegen" => errors.extend(crate::devtools_codegen::check(root)),
           _ => unreachable!("invariant: workspace_rule_package names only workspace rules"),
       }
   }

   /// Print one selected run's notes and errors and choose the exit code.
   pub(crate) fn selected(options: &Options, local: &Outcome) -> std::process::ExitCode {
       for note in &local.notes {
           println!("{note}");
       }
       for error in &local.errors {
           if options.report {
               println!("{error}");
           } else {
               eprintln!("error: {error}");
           }
       }
       if local.errors.is_empty() || options.report {
           println!("ok");
           std::process::ExitCode::SUCCESS
       } else {
           eprintln!("{} error(s)", local.errors.len());
           std::process::ExitCode::FAILURE
       }
   }

   fn validate_policy(policy: &toml::Table) -> Result<(), String> {
       for rule in RULES {
           if let Some(value) = policy.get(&rule.replace('-', "_")) {
               let table = value
                   .as_table()
                   .ok_or_else(|| format!("{rule}: expected a table"))?;
               for (key, value) in table {
                   if !matches!(
                       key.as_str(),
                       "exempt_suffixes"
                           | "recipe_state_packages"
                           | "record_types"
                           | "allowed_files"
                           | "allowed_paths"
                           | "list"
                   ) {
                       return Err(format!("{rule}: unknown setting {key}"));
                   }
                   if !value
                       .as_array()
                       .is_some_and(|items| items.iter().all(toml::Value::is_str))
                   {
                       return Err(format!("{rule}.{key}: expected an array of strings"));
                   }
               }
           }
       }
       Ok(())
   }
   ```

   `mod tests` の `crate_and_module_selectors_restrict_scope` を直す：`crates/demo/src/lib.rs` に書く文字列を `"mod state { pub struct Model { a: Cell<bool>, b: Option<Job> } mod child { pub struct Nested { a: Cell<bool>, b: bool } } } mod other { pub struct More { a: Cell<bool>, b: bool } }"` にし、`only: Some("state-fields".into())` を `only: Some("state".into())` にする（期待値 3・3・2・1 は変えない）。

12. `dev/xtask/src/main.rs`：
   - `mod state_fields;` を消し、`mod pending;` と `mod state;` をアルファベット順の位置に足す。
   - `println!("Local checks: --only <{}> [--crate <crate> [--module <path>]] [--report] [--report-audit]", ...)` を `println!("Local checks: --only <local|{}> [--crate <crate> [--module <path>]] [--strict] [--report]\n--crate runs only the local checks.", check_cli::RULES.join("|"));` にする。
   - `if options.only.is_some() {` を `if options.only.is_some() || options.package.is_some() {` にする。
   - `if options.report { report.notes.extend(local.errors); } else { ... }` の直前に `report.notes.extend(local.notes);` を足す。
   - `if local.audit_open > 0 { ... }` の塊（5 行）を消す。

13. `dev/xtask/src/public_surface.rs`：`struct Binding` の中の `// xtask-allow(state-fields): ...` の 2 行を消す。

14. `dev/xtask/policy/workspace.toml`：
   - `[state_fields]` の表（見出しと `role_suffixes`・`record_types`・`named_helpers`・value_modules の前の comment 行・`value_modules` の行）を次に置き換える。

   ```toml
   [state]
   # 設計 6.1 (c)：名前で決まる受動的な役割。`State` は recipe の状態を持つ package でだけ対象外。
   exempt_suffixes = ["Api", "Options", "Policy", "Props", "Report", "Event", "Phase", "Reason", "Recipes", "Theme", "Tokens", "Builder", "Key", "Bits"]
   recipe_state_packages = ["kivact-ui", "kivact-ui-basic", "kivact-ui-creative", "kivact-ui-game", "kivact-ui-pop", "kivact-ui-soft"]
   ```

   - ファイルの末尾に次を足す（`state` の列は xtask を除く全 package。W1c の T22 が消す）。

   ```toml
   # 改修（REWORK_DESIGN.md）の間だけ：ここに挙げた package での検査の結果は note として表示し、失敗にしない。
   # `--strict` で error に戻す。各検査を通す unit が入った後、付属書 G の最後の unit が表ごと消す。
   [rework.pending]
   state = [
       "arvane", "arvane-asset", "arvane-compose", "arvane-native", "arvane-render", "arvane-shell", "arvane-text", "arvane-virtual",
       "gpu-interop-producer", "kivact", "kivact-bench", "kivact-core", "kivact-devtools", "kivact-devtools-agent",
       "kivact-devtools-client", "kivact-devtools-mcp", "kivact-devtools-protocol", "kivact-docs", "kivact-editing", "kivact-engine",
       "kivact-example-async", "kivact-example-component", "kivact-example-counter", "kivact-example-form", "kivact-example-list",
       "kivact-gpu-share", "kivact-i18n", "kivact-interaction", "kivact-layout", "kivact-macros", "kivact-public-api-tests",
       "kivact-runtime", "kivact-shader", "kivact-style", "kivact-svg", "kivact-template-basic", "kivact-test", "kivact-types",
       "kivact-ui", "kivact-ui-basic", "kivact-ui-basic-catalog", "kivact-ui-creative", "kivact-ui-creative-catalog",
       "kivact-ui-game", "kivact-ui-game-catalog", "kivact-ui-pop", "kivact-ui-pop-catalog", "kivact-ui-soft", "kivact-ui-soft-catalog",
   ]
   ```

15. `dev/xtask/src/policy.rs` の `prepare`：`for rule in crate::check_cli::RULES { table.remove(&rule.replace('-', "_")); }` の直後に `table.remove("rework");` を足す（xtask-core は知らない表を拒むため）。

16. xtask 自身に共通手順 M を行う（`<P>` = `xtask`、`<DIRS>` = `dev/xtask/src`）。

- **消すもの**：`dev/xtask/src/state_fields.rs`、`dev/xtask/src/state_fields/`（`audit.rs`、`check.rs`、`claims.rs`、`classify.rs`、`locals.rs`、`mode.rs`、`tests.rs`）、`Mode`、`AuditRow`、`--report-audit`、`dev/xtask/state-audit.md`、`local_checks::model_types`、`Source::{marker_line, allow_reason}`、policy の `[state_fields]`。
- **状態の閉じ方**：`check_cli::Options`（`Option` 3 個と `bool` 2 個）は名前が `Options` で終わるので対象外（policy の `exempt_suffixes`）。`Index`・`Pending`・`Outcome`・`Scope`・`Counted`・`SelfWrites` は数える種類の field を 2 つ持たない。`Kind` は enum。
- **試験**：新設 `dev/xtask/src/state/tests.rs` の 12 個（上のとおり）。直す：`dev/xtask/src/check_cli.rs::tests::parses_selection_and_rejects_invalid_scopes`、`dev/xtask/src/local_checks.rs::tests::crate_and_module_selectors_restrict_scope`、`dev/xtask/src/check_source.rs::tests::loads_library_and_binary_targets_without_mixing_type_identities`。消す：`state_fields/tests.rs` の全試験（規則ごと消える）。
- **確認（tier a）**：

  ```bash
  cargo fmt -p xtask
  cargo check -p xtask --all-targets
  cargo clippy -p xtask --all-targets -- -D warnings
  cargo test -p xtask
  cargo xtask check --no-write --only local --strict --crate xtask
  cargo xtask check --no-write --only local
  ```

- **完了条件**：上のコマンドがすべて成功する。最後の 2 つは `ok` を出す（最後のものは他の package の `state` の結果を `pending:` の note として出す）。`rg -n "state-fields|state_fields|report-audit|state-audit" dev/xtask/src` が何も出さない。

---

## T02：xtask の規則の変更（図、例外、visible_to、structural-options、policy の整理）

- **wave / lane / 前提**：W1a / T0 / T01
- **所有**：`dev/xtask/src/main.rs`、`dev/xtask/src/check_cli.rs`、`dev/xtask/src/local_checks.rs`、`dev/xtask/src/check_source.rs`、`dev/xtask/src/api_shape.rs`、`dev/xtask/src/structural_options.rs`（削除）、`dev/xtask/src/diagrams.rs`（新設）、`dev/xtask/src/core_filters.rs`（新設）、`dev/xtask/policy/workspace.toml`
- **目的**：図は `cargo xtask diagrams` だけが書き、`check` は書かない（TL-02）。`api-shape` は `&'static Location` と不変の `Arc` を規則の側で認める、`internal-contract` は実装の crate では `#[doc(hidden)]` だけで足りる（TL-05）。dev-dependency は `visible_to` の対象外、例・雛形・`dev/tests/docs` は Role 規則の対象外（TL-06、設計 4.3・6.3）。`structural-options` を消す（TL-06）。policy の重複を消す（TL-07）。
- **手順**：

1. `dev/xtask/src/diagrams.rs` を新設する。

   ```rust
   //! Role: Write the architecture diagrams on request; `cargo xtask check` never writes them.

   use std::path::Path;
   use std::process::ExitCode;

   /// `cargo xtask diagrams [--root PATH] [--prune]`: run xtask-core with writing on (needs Node.js).
   pub(crate) fn run(root: &Path, arguments: &[String]) -> ExitCode {
       let mut shared = vec!["check".to_owned()];
       shared.extend(arguments.iter().cloned());
       let mut config = match xtask_core::parse_args(root, shared) {
           Ok(config) => config,
           Err(error) => {
               eprintln!("{error}");
               return ExitCode::from(2);
           }
       };
       config.no_write = false;
       if let Err(error) = crate::policy::prepare(&mut config) {
           eprintln!("error: {error}");
           return ExitCode::FAILURE;
       }
       match xtask_core::run(&config) {
           Ok(mut report) => {
               crate::core_filters::apply(&config.root, &mut report.errors);
               crate::report::print(&report)
           }
           Err(error) => {
               eprintln!("error: {error}");
               ExitCode::FAILURE
           }
       }
   }
   ```

2. `dev/xtask/src/core_filters.rs` を新設する。

   ```rust
   //! Role: Drop xtask-core findings that this workspace's policy exempts.
   //!
   //! A dev-dependency never breaks `visible_to` (design 6.3), and examples,
   //! the app template and the docs tests are written as user code without
   //! `//! Role:` lines (design 4.3).

   use std::path::Path;

   /// Directories, relative to the root, whose modules need no `//! Role:` line.
   const ROLE_EXEMPT: [&str; 3] = ["examples/", "templates/", "dev/tests/docs/"];

   pub(crate) fn apply(root: &Path, errors: &mut Vec<String>) {
       let root = root.to_string_lossy().replace('\\', "/");
       errors.retain(|error| !dev_visibility(error) && !role_exempt(&root, error));
   }

   fn dev_visibility(error: &str) -> bool {
       error.contains("` has a dev dependency")
   }

   fn role_exempt(root: &str, error: &str) -> bool {
       let error = error.replace('\\', "/");
       error.contains("missing module role")
           && ROLE_EXEMPT
               .iter()
               .any(|directory| error.contains(&format!("{root}/{directory}")) || error.starts_with(directory))
   }

   #[cfg(test)]
   mod tests {
       //! Role: Check which xtask-core findings the workspace policy drops.
       #[test]
       fn drops_dev_visibility_and_exempt_role_findings() {
           let mut errors = vec![
               "`a` has a dev dependency on `b`, which is visible only to [\"c\"]".to_owned(),
               "`a` has a normal dependency on `b`, which is visible only to [\"c\"]".to_owned(),
               "/repo/examples/kivact/counter/src/main.rs (crate): missing module role; add //! Role: <description>".to_owned(),
               "/repo/crates/kivact-core/src/lib.rs (crate): missing module role; add //! Role: <description>".to_owned(),
           ];
           super::apply(std::path::Path::new("/repo"), &mut errors);
           assert_eq!(errors.len(), 2, "{errors:?}");
           assert!(errors[0].contains("normal dependency"));
           assert!(errors[1].contains("kivact-core"));
       }
   }
   ```

3. `dev/xtask/src/main.rs`：
   - `mod core_filters;` と `mod diagrams;` をアルファベット順の位置に足し、`mod structural_options;` を消す。
   - `if arguments.first().map(String::as_str) == Some("devtools-codegen") { ... }` の塊の直後に次を足す。

   ```rust
       if arguments.first().map(String::as_str) == Some("diagrams") {
           return diagrams::run(root, &arguments[1..]);
       }
   ```
   - `let mut config = match xtask_core::parse_args(root, arguments) { ... };` の直後に次を足す。

   ```rust
       // The check validates the diagram model but never writes it; `cargo xtask diagrams` does (TL-02).
       config.no_write = true;
   ```
   - `let mut report = match xtask_core::run(&config) { ... };` の直後に `core_filters::apply(&config.root, &mut report.errors);` を足す。
   - help の `println!` の文字列の末尾に `\n\`cargo xtask diagrams\` writes architecture/ (needs Node.js).` を足す。
   - 先頭の crate doc の 1 行目を `//! Role: Run the shared xtask-core checks plus this workspace's local checks; draw diagrams on request.` にする。

4. `structural-options` を消す：`git rm -q dev/xtask/src/structural_options.rs`。`check_cli.rs` の `RULES` から `"structural-options"` を消し、配列の長さを `8` にする。`local_checks.rs` の `public_needed` の配列から `"structural-options"` を消し、`run_rule` の `"structural-options" => { ... }` の arm を消し、`validate_policy` の許す key から `"record_types"` と `"list"` を消す。

5. `dev/xtask/src/api_shape.rs` の `fn check_type`：

   ```rust
       let text = ty.to_token_stream().to_string();
       if names.contains("Rc")
           || shared_mutable_arc(&names, &text)
           || (names.contains("Box") && text.contains("dyn "))
       {
           source.error(
               "api-shape",
               ty.span(),
               "public signature exposes implementation ownership",
               errors,
           );
       }
   ```
   （削除前の条件の `names.contains("Arc")` と `(names.contains("Location") && text.contains("'static"))` を消し、文面の ` or caller Location` を消す。）`fn check_type` の後に次を足す。

   ```rust
   /// An `Arc` of shared mutation or of a trait object; an `Arc` of immutable data is a value (TL-05).
   fn shared_mutable_arc(names: &std::collections::BTreeSet<String>, text: &str) -> bool {
       names.contains("Arc")
           && (text.contains("dyn ")
               || names.iter().any(|name| {
                   matches!(name.as_str(), "Mutex" | "RwLock" | "Cell" | "RefCell")
                       || name.starts_with("Atomic")
               }))
   }
   ```
   `mod tests` に次を足す。

   ```rust
       #[test]
       fn accepts_caller_locations_and_immutable_arcs() {
           let mut source = crate::check_source::fixture(
               "pub struct Diag { pub at: &'static Location<'static>, pub data: Arc<[u8]>, pub text: Arc<str>, pub lock: Arc<Mutex<u8>>, pub shared: Arc<dyn Fn()> }",
           );
           source.module.clear();
           let surface = crate::public_surface::build(std::slice::from_ref(&source));
           let mut errors = Vec::new();
           super::check(&source, &surface, &mut errors);
           assert_eq!(errors.len(), 2, "{errors:?}");
       }
   ```

6. `dev/xtask/src/check_source.rs`：`pub(crate) fn internal_contract` を次にする。

   ```rust
       /// A `#[doc(hidden)]` item is an internal contract: in the implementation crates behind the
       /// facade without a marker (TL-05), elsewhere with `// xtask-allow(internal-contract): <reason>`.
       pub(crate) fn internal_contract(
           &self,
           attrs: &[syn::Attribute],
           span: proc_macro2::Span,
       ) -> bool {
           attrs.iter().any(|attribute| {
               attribute.path().is_ident("doc")
                   && attribute.meta.to_token_stream().to_string() == "doc (hidden)"
           }) && (INTERNAL_CONTRACT_PACKAGES.contains(&self.package.as_str())
               || self.allows("internal-contract", span))
       }
   ```
   `pub(crate) struct Source` の定義の前に次を足す。

   ```rust
   /// The implementation crates whose `#[doc(hidden)]` items need no `internal-contract` marker.
   const INTERNAL_CONTRACT_PACKAGES: [&str; 6] = [
       "kivact-types",
       "kivact-runtime",
       "kivact-style",
       "kivact-engine",
       "kivact-core",
       "arvane-protocol",
   ];
   ```
   `mod tests` に次を足す。

   ```rust
       #[test]
       fn hidden_items_of_implementation_crates_need_no_marker() {
           use syn::spanned::Spanned;
           let mut source = super::fixture("#[doc(hidden)]\npub fn bridge() {}");
           let syn::Item::Fn(item) = &source.items[0] else {
               panic!("the fixture is one function");
           };
           let (attrs, span) = (item.attrs.clone(), item.span());
           assert!(!source.internal_contract(&attrs, span));
           source.package = "kivact-runtime".into();
           assert!(source.internal_contract(&attrs, span));
       }
   ```

7. `dev/xtask/policy/workspace.toml`：
   - 先頭の 4 行（`# ViewTransform is a canonical passive ...` から `# TextDecorationStyle and Visibility remain ...` まで）を消す。
   - `[visible_to]` の次の行を置き換える。

   | 変更前 | 変更後 |
   |---|---|
   | `kivact-devtools-protocol = ["kivact-devtools-agent", "kivact-devtools-client", "kivact-core", "kivact-devtools", "kivact-devtools-mcp", "kivact-public-api-tests"]` | `kivact-devtools-protocol = ["kivact-devtools-agent", "kivact-core", "kivact-devtools", "kivact-devtools-mcp"]` |
   | `kivact-devtools-client = [...]` の行 | 消す |
   | `arvane-native = ["kivact-engine", "arvane-virtual", "arvane", "kivact"]` | `arvane-native = ["kivact-engine", "arvane-virtual", "kivact"]` |
   | `arvane-virtual = ["kivact-engine", "arvane", "kivact-test"]` | `arvane-virtual = ["kivact-engine", "kivact-test"]` |
   | `arvane-render = ["arvane", "arvane-compose", "kivact"]` | `arvane-render = ["arvane", "arvane-compose"]` |
   | `gpu-interop-producer = ["arvane", "kivact-public-api-tests"]` とその上の 2 行の comment | comment を `# 試験だけの engine（自前の device で frame を描いて共有する）。dev-dependency だけが使う（dev-dependency は visible_to の対象外）。` の 1 行にし、値を `[]` にする |

   - `# ----- アプリ・例・雛形・ツール: 誰も依存しない。` の行から `kivact-bench = []` の行まで（`# Public API tests consume ...` の comment を含む）を消す。
   - `[visible_to]` の見出しの直後に comment `# dev-dependency は visible_to の対象外（dev/xtask の core_filters.rs。設計 6.3）。` を足す。
   - `[structural_options]` の表（見出しから `list = [ ... ]` の閉じ括弧まで、その上の 2 行の comment を含む）を消す。

- **消すもの**：`dev/xtask/src/structural_options.rs`、policy の `[structural_options]`、`visible_to` の `kivact-devtools-client` と 17 個の `= []` の行、先頭の 4 行の comment。
- **状態の閉じ方**：新しい型は無い。
- **試験**：足す：`dev/xtask/src/core_filters.rs::tests::drops_dev_visibility_and_exempt_role_findings`、`dev/xtask/src/api_shape.rs::tests::accepts_caller_locations_and_immutable_arcs`、`dev/xtask/src/check_source.rs::tests::hidden_items_of_implementation_crates_need_no_marker`。
- **確認（tier a）**：

  ```bash
  cargo fmt -p xtask
  cargo clippy -p xtask --all-targets -- -D warnings
  cargo test -p xtask
  cargo xtask check --no-write --only local
  cargo xtask check --no-write
  ```

  最後の full の検査は W1a が単独の lane なので、この unit で一度だけ行う（workspace の style の Clippy が走るので時間がかかる）。`internal-contract`・`api-shape` の印はまだ残っているが、規則は印のある場所を許すので error にならない（印の削除は W1b）。
- **完了条件**：上のコマンドが成功し、最後の 2 つが `ok` を出す（`pending:` の note は出てよい）。`rg -n "structural" dev/xtask/src dev/xtask/policy` が何も出さない。`git status` に `architecture/` の変更が無い（`check` が図を書かない）。

---

## T03：設計 6.3 の新しい検査 5 つ

- **wave / lane / 前提**：W1a / T0 / T02
- **所有**：`dev/xtask/src/main.rs`、`dev/xtask/src/check_cli.rs`、`dev/xtask/src/local_checks.rs`、`dev/xtask/src/effect_writes.rs`、`dev/xtask/src/interaction_view.rs`・`ui_machine.rs`・`hand_element.rs`・`unit_machine.rs`・`element_shadow.rs`（新設）、`dev/xtask/policy/workspace.toml`
- **目的**：設計 6.3 の 5 つの規則を xtask の局所の検査として足し、今の違反を `[rework.pending]` で note にする。
- **手順**：

1. `dev/xtask/src/interaction_view.rs` を新設する。

   ```rust
   //! Role: Keep views out of kivact-interaction: only an Api's `surface` returns one (design 6.3, UI-04).

   use crate::check_source::{Source, type_names};
   use syn::spanned::Spanned;

   /// The type names that make a return value a view.
   const VIEW_TYPES: [&str; 4] = ["View", "ViewFn", "IntoView", "Element"];

   pub(crate) fn check(source: &Source, errors: &mut Vec<String>) {
       if source.package != "kivact-interaction" {
           return;
       }
       for item in &source.items {
           match item {
               syn::Item::Fn(item) => check_signature(source, &item.sig, errors),
               syn::Item::Impl(item) => {
                   for member in &item.items {
                       if let syn::ImplItem::Fn(method) = member {
                           check_signature(source, &method.sig, errors);
                       }
                   }
               }
               _ => {}
           }
       }
   }

   fn check_signature(source: &Source, signature: &syn::Signature, errors: &mut Vec<String>) {
       if signature.ident == "surface" {
           return;
       }
       let syn::ReturnType::Type(_, ty) = &signature.output else {
           return;
       };
       let names = type_names(ty);
       if VIEW_TYPES.iter().any(|name| names.contains(*name)) {
           source.error_at(
               "interaction-view",
               signature.span(),
               &format!(
                   "`{}` returns a view; kivact-interaction returns Apis and Parts, and only `surface` returns a view",
                   signature.ident
               ),
               errors,
           );
       }
   }

   #[cfg(test)]
   mod tests {
       //! Role: Check that only `surface` returns a view in kivact-interaction.
       #[test]
       fn only_surface_returns_a_view() {
           let mut source = crate::check_source::fixture(
               "pub fn outlet() -> View { loop {} }\nimpl Api { pub fn surface(self) -> View { loop {} } pub fn while_open<V: IntoView>(self) -> impl IntoView { loop {} } pub fn trigger(self) -> Part<State> { loop {} } }",
           );
           source.package = "kivact-interaction".into();
           let mut errors = Vec::new();
           super::check(&source, &mut errors);
           assert_eq!(errors.len(), 2, "{errors:?}");
       }
   }
   ```

2. `dev/xtask/src/hand_element.rs` を新設する。

   ```rust
   //! Role: Allow hand-written `impl Element` and `impl IntoView` only in kivact-core's view module (design 5.1, 6.3).

   use crate::check_source::Source;
   use syn::spanned::Spanned;

   pub(crate) fn check(source: &Source, errors: &mut Vec<String>) {
       if source.package == "kivact-core"
           && (source.module == "view" || source.module.starts_with("view::"))
       {
           return;
       }
       for item in &source.items {
           let syn::Item::Impl(item) = item else {
               continue;
           };
           let Some((_, path, _)) = &item.trait_ else {
               continue;
           };
           let Some(segment) = path.segments.last() else {
               continue;
           };
           if segment.ident == "Element" || segment.ident == "IntoView" {
               source.error_at(
                   "hand-element",
                   item.span(),
                   &format!(
                       "hand-written `impl {} for`; write a `#[component]` function (only kivact-core's view module defines elements)",
                       segment.ident
                   ),
                   errors,
               );
           }
       }
   }

   #[cfg(test)]
   mod tests {
       //! Role: Check where element impls may be written by hand.
       fn run(package: &str, module: &str) -> usize {
           let mut source = crate::check_source::fixture(
               "struct Card; impl Element for Card {} impl IntoView for Card {} impl Clone for Card {}",
           );
           source.package = package.into();
           source.module = module.into();
           let mut errors = Vec::new();
           super::check(&source, &mut errors);
           errors.len()
       }

       #[test]
       fn only_core_view_writes_element_impls() {
           assert_eq!(run("kivact-ui-soft", "widgets"), 2);
           assert_eq!(run("kivact-core", "view::elements"), 0);
           assert_eq!(run("kivact-core", "custom"), 2);
       }
   }
   ```

3. `dev/xtask/src/element_shadow.rs` を新設する。

   ```rust
   //! Role: Forbid inherent methods named like an `Element` method on a type that implements `Element` (design 5.4, API-13).

   use crate::check_source::{Source, self_type_name};
   use std::collections::BTreeSet;
   use syn::spanned::Spanned;

   /// The `Element` facts of the workspace.
   #[derive(Default)]
   pub(crate) struct Elements {
       /// The method names of kivact-core's `Element` trait.
       methods: BTreeSet<String>,
       /// `package::Type` of every type with an `impl Element for` in its package.
       types: BTreeSet<String>,
   }

   impl Elements {
       pub(crate) fn build(sources: &[Source]) -> Self {
           let mut elements = Self::default();
           for source in sources {
               for item in &source.items {
                   match item {
                       syn::Item::Trait(item)
                           if source.package == "kivact-core" && item.ident == "Element" =>
                       {
                           for member in &item.items {
                               if let syn::TraitItem::Fn(method) = member {
                                   elements.methods.insert(method.sig.ident.to_string());
                               }
                           }
                       }
                       syn::Item::Impl(item) if implements_element(item) => {
                           if let Some(name) = self_type_name(&item.self_ty) {
                               elements.types.insert(format!("{}::{name}", source.package));
                           }
                       }
                       _ => {}
                   }
               }
           }
           elements
       }
   }

   fn implements_element(item: &syn::ItemImpl) -> bool {
       item.trait_.as_ref().is_some_and(|(_, path, _)| {
           path.segments
               .last()
               .is_some_and(|segment| segment.ident == "Element")
       })
   }

   pub(crate) fn check(source: &Source, elements: &Elements, errors: &mut Vec<String>) {
       if source.package == "kivact-core" {
           return;
       }
       for item in &source.items {
           let syn::Item::Impl(item) = item else {
               continue;
           };
           if item.trait_.is_some() {
               continue;
           }
           let Some(name) = self_type_name(&item.self_ty) else {
               continue;
           };
           if !elements.types.contains(&format!("{}::{name}", source.package)) {
               continue;
           }
           for member in &item.items {
               if let syn::ImplItem::Fn(method) = member
                   && elements.methods.contains(&method.sig.ident.to_string())
               {
                   source.error_at(
                       "element-shadow",
                       method.sig.span(),
                       &format!(
                           "`{name}::{}` shadows the `Element` method of the same name; delete it and use the `Element` method",
                           method.sig.ident
                       ),
                       errors,
                   );
               }
           }
       }
   }

   #[cfg(test)]
   mod tests {
       //! Role: Check that element types do not shadow `Element` methods.
       #[test]
       fn inherent_methods_may_not_shadow_element_methods() {
           let mut core = crate::check_source::fixture(
               "pub trait Element { fn disabled(self) -> Self; fn style(self) -> Self; }",
           );
           core.package = "kivact-core".into();
           let ui = crate::check_source::fixture(
               "struct Card; impl Element for Card {} impl Card { pub fn disabled(self) -> Self { self } pub fn title(self) -> Self { self } }",
           );
           let sources = [core, ui];
           let elements = super::Elements::build(&sources);
           let mut errors = Vec::new();
           super::check(&sources[1], &elements, &mut errors);
           assert_eq!(errors.len(), 1, "{errors:?}");
           assert!(errors[0].contains("Card::disabled"), "{errors:?}");
       }
   }
   ```

4. `dev/xtask/src/unit_machine.rs` を新設する。

   ```rust
   //! Role: Forbid Machines without state: `machine(())` and `type State = ()` (design 6.3, UI-09).

   use crate::check_source::Source;
   use syn::spanned::Spanned;
   use syn::visit::Visit;

   pub(crate) fn check(source: &Source, errors: &mut Vec<String>) {
       let mut visitor = Units { source, errors };
       for item in &source.items {
           if !matches!(item, syn::Item::Mod(_)) {
               visitor.visit_item(item);
           }
       }
   }

   struct Units<'a> {
       source: &'a Source,
       errors: &'a mut Vec<String>,
   }

   impl<'ast> Visit<'ast> for Units<'_> {
       fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
           let named_machine = matches!(call.func.as_ref(), syn::Expr::Path(path)
               if path.path.segments.last().is_some_and(|segment| segment.ident == "machine"));
           let unit_argument = call
               .args
               .first()
               .is_some_and(|argument| matches!(argument, syn::Expr::Tuple(tuple) if tuple.elems.is_empty()));
           if named_machine && unit_argument {
               self.source.error_at(
                   "unit-machine",
                   call.span(),
                   "`machine(())` builds a Machine without state; write the condition inline",
                   self.errors,
               );
           }
           syn::visit::visit_expr_call(self, call);
       }

       fn visit_impl_item_type(&mut self, item: &'ast syn::ImplItemType) {
           if item.ident == "State" && matches!(&item.ty, syn::Type::Tuple(tuple) if tuple.elems.is_empty())
           {
               self.source.error_at(
                   "unit-machine",
                   item.span(),
                   "`type State = ()` declares a Machine without state; write the condition inline",
                   self.errors,
               );
           }
           syn::visit::visit_impl_item_type(self, item);
       }
   }

   #[cfg(test)]
   mod tests {
       //! Role: Check that Machines without state are reported.
       #[test]
       fn unit_state_machines_are_reported() {
           let source = crate::check_source::fixture(
               "impl Machine for Blur { type State = (); }\nfn f() { let m = machine(()); let n = machine(0u8); }",
           );
           let mut errors = Vec::new();
           super::check(&source, &mut errors);
           assert_eq!(errors.len(), 2, "{errors:?}");
       }
   }
   ```

5. `dev/xtask/src/effect_writes.rs`：
   - `struct Effects` に field `/// The rule that reports a write.\n rule: &'static str,` と `/// Report every write, ignoring \`xtask-allow\` markers.\n strict: bool,` を足す。
   - `pub(crate) fn check(source: &Source, errors: &mut Vec<String>) {` の本体を `fn scan(source: &Source, rule: &'static str, strict: bool, errors: &mut Vec<String>)` という新しい関数へ移し（本体はそのまま。`Effects { .. }` の初期化に `rule, strict,` を足す）、`check` と新しい `check_strict` を次にする。

   ```rust
   /// Effects in framework crates do not write signals. kivact-ui is checked by `ui-machine`.
   pub(crate) fn check(source: &Source, errors: &mut Vec<String>) {
       if source.package == "kivact-ui" {
           return;
       }
       scan(source, "effect-writes", false, errors);
   }

   /// The same scan under `rule`, where no marker excuses a write.
   pub(crate) fn check_strict(source: &Source, rule: &'static str, errors: &mut Vec<String>) {
       scan(source, rule, true, errors);
   }
   ```
   - `visit_expr_method_call` の `self.source.error("effect-writes", call.span(), "Signal/Var write inside an effect closure", self.errors);` を次にする。

   ```rust
               let detail = "Signal/Var write inside an effect closure";
               if self.strict {
                   self.source.error_at(self.rule, call.span(), detail, self.errors);
               } else {
                   self.source.error(self.rule, call.span(), detail, self.errors);
               }
   ```

6. `dev/xtask/src/ui_machine.rs` を新設する。

   ```rust
   //! Role: Keep input-driven Machines and signal-writing effects out of kivact-ui (design 6.3, UI-04).
   //!
   //! The policy's `allowed_types` lists the presentation timers that stay in
   //! kivact-ui (button busy reveal, progress motion, alert collapse, empty
   //! state changes) by definition path.

   use crate::check_policy::strings;
   use crate::check_source::{Source, self_type_name};
   use crate::public_surface::definition;
   use syn::spanned::Spanned;

   pub(crate) fn check(source: &Source, policy: &toml::Table, errors: &mut Vec<String>) {
       if source.package != "kivact-ui" {
           return;
       }
       let allowed = strings(policy, "allowed_types");
       for item in &source.items {
           let syn::Item::Impl(item) = item else {
               continue;
           };
           let machine = item.trait_.as_ref().is_some_and(|(_, path, _)| {
               path.segments
                   .last()
                   .is_some_and(|segment| segment.ident == "Machine")
           });
           let Some(name) = self_type_name(&item.self_ty) else {
               continue;
           };
           if machine && !allowed.contains(&definition(source, &name)) {
               source.error_at(
                   "ui-machine",
                   item.span(),
                   &format!(
                       "kivact-ui implements `Machine` for `{name}`; input-driven behavior belongs in kivact-interaction as a `use_*` Api"
                   ),
                   errors,
               );
           }
       }
       crate::effect_writes::check_strict(source, "ui-machine", errors);
   }

   #[cfg(test)]
   mod tests {
       //! Role: Check Machines and effect writes in kivact-ui.
       fn run(policy: &str) -> Vec<String> {
           let mut source = crate::check_source::fixture(
               "struct Drag; impl Machine for Drag { type State = u8; }\nfn setup(open: Signal<bool>) { effect(move || open.set(true)); }",
           );
           source.package = "kivact-ui".into();
           let policy: toml::Table = policy.parse().expect("the fixture policy parses");
           let mut errors = Vec::new();
           super::check(&source, &policy, &mut errors);
           errors
       }

       #[test]
       fn ui_has_no_machines_and_no_effect_writes() {
           assert_eq!(run("").len(), 2);
           assert_eq!(run("allowed_types = [\"kivact_ui::widgets::Drag\"]").len(), 1);
       }
   }
   ```

7. `dev/xtask/src/check_cli.rs`：`RULES` を次にする（長さ 13）。

   ```rust
   pub(crate) const RULES: [&str; 13] = [
       "state",
       "effect-writes",
       "api-shape",
       "canonical-path",
       "name-collision",
       "kit-literal-color",
       "os-branch",
       "interaction-view",
       "ui-machine",
       "hand-element",
       "unit-machine",
       "element-shadow",
       "devtools-codegen",
   ];
   ```

8. `dev/xtask/src/local_checks.rs`：
   - `Scope` に field `elements: &'a crate::element_shadow::Elements,` を足し、`check` の中で `let elements = crate::element_shadow::Elements::build(&sources);` を作って `Scope { .. }` に渡す。
   - `run_rule` の `match` に次の arm を足す（`_ =>` の前）。

   ```rust
               "interaction-view" => crate::interaction_view::check(source, errors),
               "ui-machine" => crate::ui_machine::check(source, settings, errors),
               "hand-element" => crate::hand_element::check(source, errors),
               "unit-machine" => crate::unit_machine::check(source, errors),
               "element-shadow" => crate::element_shadow::check(source, self.elements, errors),
   ```
   - `validate_policy` の許す key に `"allowed_types"` を足す。

9. `dev/xtask/src/main.rs`：`mod element_shadow;`、`mod hand_element;`、`mod interaction_view;`、`mod ui_machine;`、`mod unit_machine;` をアルファベット順の位置に足す。

10. `dev/xtask/policy/workspace.toml`：
   - `[os_branch]` の表の後に次を足す。

   ```toml
   [ui_machine]
   # kivact-ui に残す表示だけの timer（UI-04）。付属書 U が module を移したら、この path を同じ unit で直す。
   allowed_types = ["kivact_ui::button::BusyInput", "kivact_ui::progress::ProgressMotionInput", "kivact_ui::alert::BannerMachine", "kivact_ui::empty_state_changes::Controller"]
   ```
   - `[rework.pending]` の `state = [...]` の後に次を足す。

   ```toml
   interaction-view = ["kivact-interaction"]
   ui-machine = ["kivact-ui"]
   hand-element = ["kivact-core", "kivact-ui", "kivact-ui-basic", "kivact-ui-creative", "kivact-ui-game", "kivact-ui-pop", "kivact-ui-soft", "kivact-ui-game-catalog", "kivact-ui-pop-catalog"]
   unit-machine = ["kivact-interaction"]
   element-shadow = ["kivact-ui", "kivact-ui-basic", "kivact-ui-creative", "kivact-ui-game", "kivact-ui-pop", "kivact-ui-soft", "kivact-ui-game-catalog", "kivact-ui-pop-catalog"]
   ```

11. `cargo xtask check --no-write --only interaction-view,ui-machine,hand-element,unit-machine,element-shadow` を実行する。`error:` の行が出たら、その行の package を、その行の検査の名前の pending の列に足し（アルファベット順）、unit の報告に「pending に足した package：<rule> <package>」と書く。`ok` になるまで繰り返す。

- **消すもの**：なし。
- **状態の閉じ方**：`Effects` に足す field は `rule`（`&'static str`）と `strict`（`bool`）で、数える種類は `strict` だけ（`depth` は名前の付いた counter ではない）。`Elements` は集合 2 つ。印は要らない。
- **試験**：足す：`interaction_view::tests::only_surface_returns_a_view`、`hand_element::tests::only_core_view_writes_element_impls`、`element_shadow::tests::inherent_methods_may_not_shadow_element_methods`、`unit_machine::tests::unit_state_machines_are_reported`、`ui_machine::tests::ui_has_no_machines_and_no_effect_writes`。既存の `effect_writes::tests` の 4 つはそのまま通る（fixture の package は `kivact-ui-soft`）。
- **確認（tier a）**：

  ```bash
  cargo fmt -p xtask
  cargo clippy -p xtask --all-targets -- -D warnings
  cargo test -p xtask
  cargo xtask check --no-write --only local
  ```

- **完了条件**：上のコマンドが成功し、最後が `ok` を出す。`cargo xtask check --no-write --only interaction-view --strict --crate kivact-interaction` が `announcer`・`while_open` を含む error を出して失敗する（pending が効いている証拠。成功してはいけない）。

---

## T04：`docs-pairs` 検査、DevTools の schema の生成の移動、codegen と Python の削除

- **wave / lane / 前提**：W1a / T0 / T03
- **所有**：`dev/xtask/src/main.rs`、`dev/xtask/src/check_cli.rs`、`dev/xtask/src/local_checks.rs`、`dev/xtask/src/docs_pairs.rs`（新設）、`dev/xtask/src/devtools_codegen.rs` と `dev/xtask/src/devtools_codegen/`（削除）、`dev/xtask/policy/workspace.toml`、`crates/kivact-devtools-protocol/Cargo.toml`、`crates/kivact-devtools-protocol/tests/schema_generated.rs` と `tests/schema_generated/`（新設）、`python/`（削除）
- **目的**：日本語の guide の Rust の code block の列が英語と同じであることを確かめる `docs-pairs`（TL-03、設計 6.4）。Python の client と codegen を消し、JSON schema の生成と照合を protocol の試験にする（TL-09、DEC-10）。
- **手順**：

1. `dev/xtask/src/docs_pairs.rs` を新設する。

   ```rust
   //! Role: Check that each Japanese guide has the same Rust code blocks as its English twin.
   //!
   //! Comments, whitespace and the text of string literals may differ (they are
   //! translated or wrapped differently); the info string and everything else of
   //! each fenced Rust block must match (design 6.4, TL-03).

   use std::path::{Path, PathBuf};

   /// The guide trees with an `en` and a `jp` side.
   const TREES: [&str; 2] = ["docs/for-kivact-user", "docs/for-kivact-ui-user"];

   pub(crate) fn check(root: &Path, errors: &mut Vec<String>) {
       let mut pairs = vec![(root.join("README.md"), root.join("README_JP.md"))];
       for tree in TREES {
           let japanese = root.join(tree).join("jp");
           let mut files = Vec::new();
           markdown_files(&japanese, &mut files);
           for jp in files {
               if let Ok(relative) = jp.strip_prefix(&japanese) {
                   pairs.push((root.join(tree).join("en").join(relative), jp.clone()));
               }
           }
       }
       for (en, jp) in pairs {
           compare(&en, &jp, errors);
       }
   }

   fn markdown_files(directory: &Path, out: &mut Vec<PathBuf>) {
       let Ok(entries) = std::fs::read_dir(directory) else {
           return;
       };
       let mut paths: Vec<PathBuf> = entries.filter_map(Result::ok).map(|entry| entry.path()).collect();
       paths.sort();
       for path in paths {
           if path.is_dir() {
               markdown_files(&path, out);
           } else if path.extension().is_some_and(|extension| extension == "md") {
               out.push(path);
           }
       }
   }

   fn compare(en: &Path, jp: &Path, errors: &mut Vec<String>) {
       let Ok(english) = std::fs::read_to_string(en) else {
           errors.push(format!(
               "docs-pairs: {}: the English twin {} is missing",
               jp.display(),
               en.display()
           ));
           return;
       };
       let japanese = std::fs::read_to_string(jp).unwrap_or_default();
       let (english, japanese) = (rust_blocks(&english), rust_blocks(&japanese));
       if english.len() != japanese.len() {
           errors.push(format!(
               "docs-pairs: {}: {} Rust blocks, the English twin has {}",
               jp.display(),
               japanese.len(),
               english.len()
           ));
           return;
       }
       for (index, (a, b)) in english.iter().zip(&japanese).enumerate() {
           if a != b {
               errors.push(format!(
                   "docs-pairs: {}: Rust block {} differs from {} beyond comments, whitespace and string text",
                   jp.display(),
                   index + 1,
                   en.display()
               ));
           }
       }
   }

   /// The info string and the normalized code of every fenced block whose info string starts with `rust`.
   fn rust_blocks(text: &str) -> Vec<(String, String)> {
       let mut blocks = Vec::new();
       let mut open: Option<(String, String)> = None;
       for line in text.lines() {
           match (open.take(), line.strip_prefix("```")) {
               (None, Some(info)) => open = Some((info.trim().to_owned(), String::new())),
               (Some((info, body)), Some(_)) => {
                   if info.starts_with("rust") {
                       blocks.push((info, normalize(&body)));
                   }
               }
               (Some((info, mut body)), None) => {
                   body.push_str(line);
                   body.push('\n');
                   open = Some((info, body));
               }
               (None, None) => {}
           }
       }
       blocks
   }

   /// `code` without comments and whitespace, each string literal emptied.
   fn normalize(code: &str) -> String {
       let chars: Vec<char> = code.chars().collect();
       let mut out = String::new();
       let mut i = 0;
       while i < chars.len() {
           let c = chars[i];
           let next = chars.get(i + 1).copied();
           if c == '/' && next == Some('/') {
               while i < chars.len() && chars[i] != '\n' {
                   i += 1;
               }
           } else if c == '/' && next == Some('*') {
               i += 2;
               while i < chars.len() && !(chars[i] == '*' && chars.get(i + 1) == Some(&'/')) {
                   i += 1;
               }
               i += 2;
           } else if c == '"' {
               i = skip_string(&chars, i + 1);
               out.push_str("\"\"");
           } else if c == 'r' && matches!(next, Some('"' | '#')) && !after_word(&chars, i) {
               if let Some(end) = skip_raw_string(&chars, i + 1) {
                   i = end;
                   out.push_str("\"\"");
               } else {
                   out.push(c);
                   i += 1;
               }
           } else if c == '\'' && (next == Some('\\') || chars.get(i + 2) == Some(&'\'')) {
               let end = skip_char(&chars, i + 1);
               out.extend(&chars[i..end]);
               i = end;
           } else {
               if !c.is_whitespace() {
                   out.push(c);
               }
               i += 1;
           }
       }
       out
   }

   fn skip_string(chars: &[char], mut i: usize) -> usize {
       while i < chars.len() {
           match chars[i] {
               '\\' => i += 2,
               '"' => return i + 1,
               _ => i += 1,
           }
       }
       chars.len()
   }

   fn skip_raw_string(chars: &[char], mut i: usize) -> Option<usize> {
       let mut hashes = 0;
       while chars.get(i) == Some(&'#') {
           hashes += 1;
           i += 1;
       }
       if chars.get(i) != Some(&'"') {
           return None;
       }
       i += 1;
       while i < chars.len() {
           if chars[i] == '"' && (1..=hashes).all(|k| chars.get(i + k) == Some(&'#')) {
               return Some(i + 1 + hashes);
           }
           i += 1;
       }
       Some(chars.len())
   }

   fn skip_char(chars: &[char], mut i: usize) -> usize {
       while i < chars.len() {
           match chars[i] {
               '\\' => i += 2,
               '\'' => return i + 1,
               _ => i += 1,
           }
       }
       chars.len()
   }

   fn after_word(chars: &[char], i: usize) -> bool {
       i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_')
   }

   #[cfg(test)]
   mod tests {
       //! Role: Check what may and may not differ between a guide and its twin.
       use super::{normalize, rust_blocks};

       #[test]
       fn comments_whitespace_and_string_text_may_differ() {
           let en = "```rust\nlet name = \"Name\"; // the name\nfield(name).hint(\"Shown\")\n```\n";
           let jp = "```rust\nlet name = \"名前\"; // 名前\nfield(name)\n    .hint(\"表示\")\n```\n";
           assert_eq!(rust_blocks(en), rust_blocks(jp));
       }

       #[test]
       fn code_and_info_strings_must_match() {
           assert_ne!(rust_blocks("```rust\nfn don_t() {}\n```\n"), rust_blocks("```rust\nfn dont() {}\n```\n"));
           assert_ne!(rust_blocks("```rust,no_run\nfn a() {}\n```\n"), rust_blocks("```rust\nfn a() {}\n```\n"));
           assert!(rust_blocks("```text\nanything\n```\n").is_empty());
       }

       #[test]
       fn char_literals_and_raw_strings_are_not_strings() {
           assert_eq!(
               normalize("let q = '\"'; let r = r#\"a \"b\"\"#; // c"),
               "letq='\"';letr=\"\";"
           );
       }
   }
   ```

2. `dev/xtask/src/check_cli.rs`：`RULES` の最後の `"devtools-codegen"` を `"docs-pairs"` にする。

3. `dev/xtask/src/local_checks.rs`：`workspace_rule_package` の `"devtools-codegen" => Some("kivact-devtools-protocol"),` を `"docs-pairs" => Some("kivact-docs"),` にし、`run_workspace_rule` の `"devtools-codegen" => errors.extend(crate::devtools_codegen::check(root)),` を `"docs-pairs" => crate::docs_pairs::check(root, errors),` にする。

4. `dev/xtask/src/main.rs`：`mod devtools_codegen;` を消し、`mod docs_pairs;` を足す。`if arguments.first().map(String::as_str) == Some("devtools-codegen") { return devtools_codegen::run(root, &arguments[1..]); }` の塊を消す。

5. schema の生成を protocol の試験へ移す。

   ```bash
   mkdir -p crates/kivact-devtools-protocol/tests/schema_generated
   for f in json model read schema; do git mv dev/xtask/src/devtools_codegen/$f.rs crates/kivact-devtools-protocol/tests/schema_generated/$f.rs; done
   git mv dev/xtask/src/devtools_codegen/tests.rs crates/kivact-devtools-protocol/tests/schema_generated/reader_tests.rs
   git rm -r -q dev/xtask/src/devtools_codegen dev/xtask/src/devtools_codegen.rs
   perl -pi -e 's/\bcrate::devtools_codegen::/crate::/g' crates/kivact-devtools-protocol/tests/schema_generated/*.rs
   ```
   - `tests/schema_generated/read.rs`：`const SKIPPED_FILES: [&str; 7]` を `[&str; 8]` にし、列の最後に `"client.rs",` を足す（T13 が置く `client.rs` は wire の型を持たない）。
   - `tests/schema_generated/model.rs`：`pub(super) fn upper_snake` を doc ごと消す（Python だけが使っていた）。
   - `tests/schema_generated/reader_tests.rs`：1 行目を `//! Role: Verify reading protocol source and the schema built from it.` にする。`fn reads_records_choices_unions_aliases_and_methods` から `let python = super::python::python(&protocol);` の行と、`python.contains(` を含む `assert!` 文（1 行のもの 6 つと複数行のもの 2 つ）をすべて消す。`fn names_convert_to_snake_and_upper_snake` を `fn names_convert_to_snake_case` に改名し、`upper_snake` の `assert_eq!` の行を消す。
   - `crates/kivact-devtools-protocol/tests/schema_generated.rs` を新設する。

   ```rust
   //! Role: Generate the DevTools JSON schemas from the protocol source and compare them with the committed files.
   //!
   //! `KIVACT_WRITE_SCHEMA=1 cargo test -p kivact-devtools-protocol --test schema_generated`
   //! rewrites `schema/*.json` after a change of the wire types.

   mod committed;
   mod json;
   mod model;
   mod read;
   mod reader_tests;
   mod schema;
   ```
   - `crates/kivact-devtools-protocol/tests/schema_generated/committed.rs` を新設する。

   ```rust
   //! Role: Compare the committed schema files with the ones generated from the source.

   use std::path::Path;

   #[test]
   fn committed_schemas_match_the_source() {
       let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
       let protocol =
           super::read::read_protocol(&manifest.join("../..")).expect("the protocol source reads");
       let files = [
           ("schema/kivact-devtools.schema.json", super::schema::schema(&protocol)),
           ("schema/devtools-config.schema.json", super::schema::config_schema(&protocol)),
       ];
       let write = std::env::var_os("KIVACT_WRITE_SCHEMA").is_some();
       for (name, expected) in files {
           let path = manifest.join(name);
           if write {
               std::fs::write(&path, &expected).expect("the schema file is writable");
               continue;
           }
           let current = std::fs::read_to_string(&path)
               .expect("the committed schema exists")
               .replace("\r\n", "\n");
           assert!(
               current == expected,
               "{name} is out of date; run `KIVACT_WRITE_SCHEMA=1 cargo test -p kivact-devtools-protocol --test schema_generated`"
           );
       }
   }
   ```
   - `crates/kivact-devtools-protocol/Cargo.toml` に次を足す。

   ```toml
   [dev-dependencies]
   proc-macro2 = { workspace = true }
   syn = { workspace = true }
   ```
   - `cargo clippy -p kivact-devtools-protocol --all-targets -- -D warnings` が `dead_code`（Python だけが使っていた model の item・field）を報告したら、その item・field と、それを作る `read.rs` の箇所を消す。

6. `git rm -r -q python`。

7. `dev/xtask/policy/workspace.toml` の `[rework.pending]` の最後に `docs-pairs = ["kivact-docs"]` を足す（日本語の 2 ファイルの不一致を T17 が直し、T22 がこの行を消す）。

- **消すもの**：`dev/xtask/src/devtools_codegen.rs`、`dev/xtask/src/devtools_codegen/{check.rs,cli.rs,python.rs}`、`model::upper_snake`、`python/`（全体）、`cargo xtask devtools-codegen` の subcommand。
- **状態の閉じ方**：新しい型は無い（`docs_pairs` は関数だけ）。
- **試験**：足す：`dev/xtask/src/docs_pairs.rs` の 3 つ、`crates/kivact-devtools-protocol/tests/schema_generated/committed.rs::committed_schemas_match_the_source`。移す：`reader_tests.rs` の 8 つ（`names_convert_to_snake_and_upper_snake` は `names_convert_to_snake_case` に改名、Python の assert を消す）。
- **確認（tier a）**：

  ```bash
  cargo fmt -p xtask -p kivact-devtools-protocol
  cargo clippy -p xtask -p kivact-devtools-protocol --all-targets -- -D warnings
  cargo test -p xtask
  cargo test -p kivact-devtools-protocol
  cargo xtask check --no-write --only local
  cargo xtask check --no-write
  ```

- **完了条件**：上のコマンドが成功し、最後の 2 つが `ok`。`cargo xtask check --no-write --only docs-pairs --strict` は `canvas.md` と `windows.md` の 2 件だけを error にする（T17 が直す）。`rg -n "devtools-codegen|devtools_codegen" dev crates` が何も出さない。`python/` が無い。

---

## T23：DevTools の Scripts パネルと、それだけが使う protocol を消す

- **wave / lane / 前提**：W1a / T0 / T04（W1a の最後の unit。番号は追加の順で、W1a の中では T04 の次に行う）
- **所有**：
  - `crates/kivact-devtools-protocol/src/{scripts.rs（削除）,lib.rs,call.rs,notify.rs,app.rs,config.rs}`、`crates/kivact-devtools-protocol/schema/{kivact-devtools.schema.json,devtools-config.schema.json}`（生成し直す）、`crates/kivact-devtools-protocol/tests/protocol.rs`
  - `crates/kivact-devtools-client/tests/connection.rs`（T13 が後で移すファイル。ここでは 1 行だけ）
  - `crates/kivact-core/src/devtools.rs`、`crates/kivact-core/src/devtools/{scripts.rs（削除）,target.rs}`
  - `crates/kivact-devtools/src/{panels.rs,panels/scripts.rs（削除）,app.rs,state.rs,session.rs}`、`crates/kivact-devtools/tests/app.rs`、`crates/kivact-devtools/tests/app/panels/scripts.rs`（削除）
  - `crates/kivact-devtools-mcp/src/tools.rs`、`crates/kivact-devtools-mcp/tests/mcp.rs`
- **目的**：Python の client を消す（TL-09、DEC-10）と、Scripts パネル（`.py` を実行し `show_table` の表を出す）は使い道が無くなる。パネルと、それだけが使う protocol の method・通知・型・config の field・topic・client の種類を消し、`PROTOCOL_VERSION` を上げる（親の決定。protocol の版の規則：method・通知の削除と field の削除は版を上げる。`crates/kivact-devtools-protocol/src/app.rs` の `PROTOCOL_VERSION` の doc と `docs/for-kivact-developer/architecture.md` の「DevTools の method」の行）。
- **それだけが使うことの確認**（2b8df5ac で `rg` 済み。実装の前にもう一度 `rg -n "ShowTable|show_table|ScriptTable|script_table|Topic::Scripts|scripts_dir|SCRIPTS_DIR|ClientKind::Python|\.python\b|MAX_TABLES|\btables\b" crates dev --glob '!**/schema/*.json'` を実行し、次の表の外に結果があれば STOP）：

  | 名前 | 使う場所（全部消す） |
  |---|---|
  | method `show_table`（`Call::ShowTable`、`Reply::ShowTable`、`ShowTable`） | protocol `call.rs`、core `devtools/scripts.rs`、MCP の `EXCLUDED` と試験の除外の列 |
  | 通知 `script_table`（`Notification::ScriptTable`） | protocol `notify.rs`、core `devtools/scripts.rs`、GUI `session.rs`、GUI の試験 `tests/app/panels/scripts.rs` |
  | `ShowTableParams`（`scripts.rs`） | protocol `call.rs`・`notify.rs`、GUI `state.rs` の `tables` |
  | `Topic::Scripts` | protocol `app.rs`、core `devtools/scripts.rs`、GUI `session.rs` の購読 |
  | `DevtoolsConfig::{python, scripts_dir}`、`scripts_dir()`、`SCRIPTS_DIR` | protocol `config.rs`、GUI `panels/scripts.rs` |
  | `ClientKind::Python` | protocol `app.rs`、`crates/kivact-devtools-client/tests/connection.rs`（試験の値だけ） |
  | GUI の `Panel::Scripts`、`Session::tables`、`MAX_TABLES` | GUI `state.rs`・`app.rs`・`session.rs`・`panels.rs`、試験 `tests/app.rs` |

  残すもの：`push_capped`（turns・frames・errors も使う）、`Topic` の他の variant、`ClientKind::{Devtools, Mcp, Other}`。
- **手順**：
  1. protocol：
     - `git rm -q crates/kivact-devtools-protocol/src/scripts.rs`。`lib.rs` の `mod scripts;` と `pub use scripts::*;` を消す。
     - `call.rs`：`use crate::scripts::ShowTableParams;` と、`protocol_methods!` の中の `/// Show a table in the DevTools app (for scripts).` と `ShowTable => "show_table" (ShowTableParams) -> Empty;` の 2 行を消す。
     - `notify.rs`：`use crate::scripts::ShowTableParams;` と、`/// A script showed a table.` と `ScriptTable => "script_table" (ShowTableParams) in Scripts;` の 2 行を消す。
     - `app.rs`：`pub const PROTOCOL_VERSION: u32 = 1;` を `pub const PROTOCOL_VERSION: u32 = 2;` にする。`enum ClientKind` の `/// The Python client.` と `Python,` を消す。`enum Topic` の `/// Script output.` と `Scripts,` を消す。
     - `config.rs`：`DevtoolsConfig` の field `python`（doc `/// Path of the Python interpreter for scripts.` と `#[serde(..)]` を含む 3 行）と `scripts_dir`（同じく 3 行）、`const SCRIPTS_DIR`、`pub fn scripts_dir`（doc を含む）、`with_defaults` の中の `scripts_dir: Some(self.scripts_dir()),` を消す。
     - `tests/protocol.rs` の `methods_are_complete` の `assert_eq!(METHOD_NAMES.len(), 60);` を `59` にする。
  2. `crates/kivact-devtools-client/tests/connection.rs` の `kind: ClientKind::Python,` を `kind: ClientKind::Other,` にする。
  3. core：`git rm -q crates/kivact-core/src/devtools/scripts.rs`。`crates/kivact-core/src/devtools.rs` の `mod scripts;` を消す。`devtools/target.rs` の `const ROUTES: [Route; 14] = [` を `[Route; 13]` にし、列の `scripts::route,` を消す。
  4. GUI（`crates/kivact-devtools`）：
     - `git rm -q crates/kivact-devtools/src/panels/scripts.rs crates/kivact-devtools/tests/app/panels/scripts.rs`。`src/panels.rs` の `pub(crate) mod scripts;` を消す。
     - `src/state.rs`：`Session` の field `/// The \`show_table\` results.` と `pub(crate) tables: Signal<Vec<p::ShowTableParams>>,` を消す。`pub(crate) const MAX_TABLES: usize = 50;`（doc を含む）を消す。`enum Panel` の `/// Scripts and their tables.` と `Scripts,`、`Panel::ALL` の `Panel::Scripts,` を消して型を `[Panel; 11]` にし、`fn title` の `Panel::Scripts => "Scripts",` を消す。
     - `src/app.rs`：`Panel::Scripts => panels::scripts::view(session).into_view(),` を消す。
     - `src/session.rs`：`use` の列から `MAX_TABLES` を消す。`let tables = signal(Vec::new());` と `Session { .. }` の初期化の `tables,` を消す。`subscribe` の `p::Topic::Scripts,` を消す。`p::Notification::ScriptTable(table) => { ... }` の arm（4 行）を消す。
     - `tests/app.rs`：`#[path = "app/panels/scripts.rs"]` と `mod scripts;` の 2 行を消す。tab の題の列の `"Scripts",` を消す。
  5. MCP：`src/tools.rs` の `const EXCLUDED: [&str; 3] = ["initialize", "subscribe", "show_table"];` を `const EXCLUDED: [&str; 2] = ["initialize", "subscribe"];` にする。`tests/mcp.rs` の `!["initialize", "subscribe", "show_table"].contains(name)` を `!["initialize", "subscribe"].contains(name)` にする。
  6. schema を生成し直す：`KIVACT_WRITE_SCHEMA=1 cargo test -p kivact-devtools-protocol --test schema_generated`（T04 の試験）。`git diff --stat crates/kivact-devtools-protocol/schema` で 2 つの JSON が変わり、`"x-protocol": 2` になったことを確かめる。
  7. compile error・`dead_code`・unused import の警告に従って、上で消したものの残りの参照だけを消す（他のものは消さない）。
- **消すもの**：`crates/kivact-devtools-protocol/src/scripts.rs`、`ShowTableParams`、method `show_table`、通知 `script_table`、`Topic::Scripts`、`ClientKind::Python`、`DevtoolsConfig::{python, scripts_dir}`、`DevtoolsConfig::scripts_dir()`、`SCRIPTS_DIR`、`crates/kivact-core/src/devtools/scripts.rs`、`crates/kivact-devtools/src/panels/scripts.rs`、`Panel::Scripts`、`Session::tables`、`MAX_TABLES`、試験 `crates/kivact-devtools/tests/app/panels/scripts.rs`。
- **状態の閉じ方**：新しい型は無い。`Session` から field を 1 つ消すだけ。
- **試験**：消す：`crates/kivact-devtools/tests/app/panels/scripts.rs` の全試験（Scripts パネル）。直す：`crates/kivact-devtools-protocol/tests/protocol.rs::methods_are_complete`（60 → 59）、`crates/kivact-devtools/tests/app.rs` の tab の題の列を持つ試験（`"Scripts"` を消す）、`crates/kivact-devtools-mcp/tests/mcp.rs` の除外の列を持つ試験、`crates/kivact-devtools-client/tests/connection.rs` の `ClientKind` の値。`crates/kivact-devtools-protocol/tests/schema_generated/committed.rs::committed_schemas_match_the_source` は生成し直した schema で通る。
- **確認（tier a）**：

  ```bash
  cargo fmt -p kivact-devtools-protocol -p kivact-devtools-client -p kivact-core -p kivact-devtools -p kivact-devtools-mcp
  cargo clippy -p kivact-devtools-protocol -p kivact-devtools-client -p kivact-devtools-mcp --all-targets -- -D warnings
  cargo clippy -p kivact-core --all-targets --features devtools -- -D warnings
  cargo check -p kivact-devtools --all-targets
  cargo test -p kivact-devtools-protocol
  cargo test -p kivact-devtools-client
  cargo test -p kivact-devtools-mcp
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-devtools --test app
  cargo xtask check --no-write --only local
  ```

- **完了条件**：すべて成功。`rg -n "ShowTable|show_table|ScriptTable|script_table|Topic::Scripts|scripts_dir|SCRIPTS_DIR|ClientKind::Python|Panel::Scripts|MAX_TABLES" crates dev` が何も出さない（`docs/` は T18 が直す）。`rg -n '"x-protocol": 2' crates/kivact-devtools-protocol/schema/kivact-devtools.schema.json` が 1 行を出す。

---

## T05：`kivact-ui` の印の変換

- **wave / lane / 前提**：W1b / T1 / T23
- **所有**：`crates/kivact-ui/src/**`
- **目的**：`kivact-ui` の 208 行の `xtask-allow(state-fields)` を `state` 規則に移す（TL-04）。
- **手順**：共通手順 M を `<P>` = `kivact-ui`、`<DIRS>` = `crates/kivact-ui/src` で行う。`ButtonState` など `*State` で終わる型は `recipe_state_packages` で対象外なので報告されない。
- **消すもの**：`crates/kivact-ui/src` の全 `// xtask-allow(state-fields):` の行。
- **状態の閉じ方**：報告された型・関数ごとに共通手順 M2 の 1 行。
- **試験**：なし（comment だけの変更）。
- **確認（tier a）**：

  ```bash
  cargo xtask check --no-write --only state --strict --crate kivact-ui
  cargo fmt -p kivact-ui
  cargo check -p kivact-ui
  ```

- **完了条件**：1 つ目が `ok`。`rg -c "xtask-allow\(state-fields\)" crates/kivact-ui` が何も出さない。

## T06：`kivact-interaction` の印の変換

- **wave / lane / 前提**：W1b / T1 / T05
- **所有**：`crates/kivact-interaction/src/**`
- **目的**：81 行の印を移す（TL-04）。
- **手順**：共通手順 M を `<P>` = `kivact-interaction`、`<DIRS>` = `crates/kivact-interaction/src` で行う。`crates/kivact-interaction/src/menu/level.rs` の `// xtask-allow(effect-writes): ...` は残す（`effect-writes` の印。付属書 U が扱う）。
- **消すもの**：`crates/kivact-interaction/src` の全 `// xtask-allow(state-fields):` の行。
- **状態の閉じ方**：共通手順 M2。
- **試験**：なし。
- **確認（tier a）**：

  ```bash
  cargo xtask check --no-write --only state --strict --crate kivact-interaction
  cargo fmt -p kivact-interaction
  cargo check -p kivact-interaction
  ```

- **完了条件**：1 つ目が `ok`。`rg -c "xtask-allow\(state-fields\)" crates/kivact-interaction` が何も出さない。

## T07：`kivact-core`・`kivact`・`kivact-macros` の印の変換と繰り返しの印の削除

- **wave / lane / 前提**：W1b / T2 / T23
- **所有**：`crates/kivact-core/src/**`、`crates/kivact/src/**`、`crates/kivact-macros/src/**`
- **目的**：state の印を移し（TL-04）、規則の側の例外になった `internal-contract`・`api-shape`、消えた `structural-options` の印を消す（TL-05、TL-06）。
- **手順**：
  1. 共通手順 M を `kivact-core`（`crates/kivact-core/src`）、`kivact`（`crates/kivact/src`）、`kivact-macros`（`crates/kivact-macros/src`）の順に行う。
  2. 繰り返しの印を消す。

     ```bash
     rg -l "xtask-allow\((internal-contract|structural-options)\)" crates/kivact-core/src | xargs -r perl -ni -e 'print unless /^\s*\/\/ xtask-allow\((internal-contract|structural-options)\):/'
     rg -l "xtask-allow\(api-shape\)" crates/kivact-core/src | xargs -r perl -ni -e 'print unless /^\s*\/\/ xtask-allow\(api-shape\): [Dd]iagnostic source positions retain Rust caller locations\.\s*$/'
     ```
     （`view/owner.rs` の `on_cleanup` と `view/style_layer.rs` の `RecipeName` の `api-shape` の印は理由が違うので残る。）
  3. `crates/kivact-core/src/custom.rs` の `//! The E2E test is \`dev/tests/kivact-public-api/tests/e2e/primitive.rs\`.` の `tests/e2e/` を `tests/e2e_core/` にする（T15 が移す先）。
- **消すもの**：3 crate の全 `xtask-allow(state-fields)`、`kivact-core` の全 `xtask-allow(internal-contract)`（launch/settings.rs、state.rs、task.rs、view/element_data.rs、view/elements/viewport.rs ほか）、`view/node_modifiers.rs` の `xtask-allow(structural-options)`、`diagnostics/why.rs` の `api-shape` の 3 行。
- **状態の閉じ方**：共通手順 M2。
- **試験**：なし。
- **確認（tier a）**：

  ```bash
  cargo xtask check --no-write --only state --strict --crate kivact-core
  cargo xtask check --no-write --only state --strict --crate kivact
  cargo xtask check --no-write --only state --strict --crate kivact-macros
  cargo xtask check --no-write --only api-shape,canonical-path,name-collision --crate kivact-core
  cargo fmt -p kivact-core -p kivact -p kivact-macros
  cargo check -p kivact-core
  ```

- **完了条件**：xtask の 4 つが `ok`（4 つ目で error が出て、それが消した印の直後の行を指すなら STOP：`STOP: T07 <path> 消した印の場所で api-shape が error`）。`rg -n "xtask-allow\((state-fields|internal-contract|structural-options)\)" crates/kivact-core crates/kivact crates/kivact-macros` が何も出さない。

## T08：`kivact-engine`・`kivact-layout`・`kivact-editing` の印の変換

- **wave / lane / 前提**：W1b / T2 / T07
- **所有**：`crates/kivact-engine/src/**`、`crates/kivact-layout/src/**`、`crates/kivact-editing/src/**`
- **目的**：TL-04、TL-05。
- **手順**：
  1. 共通手順 M を `kivact-engine`、`kivact-layout`、`kivact-editing` の順に行う（`<DIRS>` は各 `crates/<P>/src`）。
  2. `rg -l "xtask-allow\(internal-contract\)" crates/kivact-engine/src | xargs -r perl -ni -e 'print unless /^\s*\/\/ xtask-allow\(internal-contract\):/'`（`contract.rs`、`host/write/reader.rs` の 11 行、`host/write/writer.rs` の 4 行）。
- **消すもの**：3 crate の全 `xtask-allow(state-fields)`、`kivact-engine` の全 `xtask-allow(internal-contract)`。
- **状態の閉じ方**：共通手順 M2。
- **試験**：なし。
- **確認（tier a）**：

  ```bash
  cargo xtask check --no-write --only state --strict --crate kivact-engine
  cargo xtask check --no-write --only state --strict --crate kivact-layout
  cargo xtask check --no-write --only state --strict --crate kivact-editing
  cargo xtask check --no-write --only api-shape,canonical-path --crate kivact-engine
  cargo fmt -p kivact-engine -p kivact-layout -p kivact-editing
  cargo check -p kivact-engine
  ```

- **完了条件**：xtask の 4 つが `ok`。`rg -n "xtask-allow\((state-fields|internal-contract)\)" crates/kivact-engine crates/kivact-layout crates/kivact-editing` が何も出さない。

## T09：runtime・i18n・style・types・shader・svg・gpu-share の印の変換

- **wave / lane / 前提**：W1b / T3 / T23
- **所有**：`crates/kivact-runtime/src/**`、`crates/kivact-i18n/src/**`、`crates/kivact-style/src/**`、`crates/kivact-types/src/**`（`crates/kivact-types/src/cell.rs` を除く。付属書 S の W1a の unit の所有）、`crates/kivact-shader/src/**`、`crates/kivact-svg/src/**`、`crates/kivact-gpu-share/src/**`
- **目的**：TL-04、TL-05（`kivact-style/src/sheet/fields.rs` の 91 行、`text.rs` の 46 行を含む）。
- **手順**：
  1. 共通手順 M を `kivact-runtime`、`kivact-i18n`、`kivact-style`、`kivact-types`、`kivact-shader`、`kivact-svg`、`kivact-gpu-share` の順に行う。
  2. 繰り返しの印を消す。

     ```bash
     DIRS="crates/kivact-runtime/src crates/kivact-style/src crates/kivact-types/src"
     rg -l "xtask-allow\(internal-contract\)" $DIRS | xargs -r perl -ni -e 'print unless /^\s*\/\/ xtask-allow\(internal-contract\):/'
     rg -l "xtask-allow\(api-shape\)" $DIRS | xargs -r perl -ni -e 'print unless /^\s*\/\/ xtask-allow\(api-shape\): ([Dd]iagnostic source positions retain Rust caller locations|Shared immutable payload, cloned without copying its data; one allocation is shared by all clones)\.\s*$/'
     ```
     （`runtime/src/reactive/error.rs` の「Standard Rust Error interop ...」の 2 行と `runtime/src/task/action.rs` の「sealed hidden contract ...」は理由が違うので残る。）
- **消すもの**：7 crate の全 `xtask-allow(state-fields)`、runtime・style・types の全 `xtask-allow(internal-contract)`、上の 2 種の理由の `xtask-allow(api-shape)`（runtime の約 20 行、types の paint.rs 6・motion.rs 1・effect/shader.rs 3・render/composition.rs 1）。
- **状態の閉じ方**：共通手順 M2。
- **試験**：なし。
- **確認（tier a）**：

  ```bash
  for p in kivact-runtime kivact-i18n kivact-style kivact-types kivact-shader kivact-svg kivact-gpu-share; do cargo xtask check --no-write --only state --strict --crate $p || break; done
  for p in kivact-runtime kivact-style kivact-types; do cargo xtask check --no-write --only api-shape,canonical-path --crate $p || break; done
  cargo fmt -p kivact-runtime -p kivact-i18n -p kivact-style -p kivact-types -p kivact-shader -p kivact-svg -p kivact-gpu-share
  ```

- **完了条件**：各コマンドが `ok`（api-shape の error が消した印の直後を指すなら STOP）。`rg -n "xtask-allow\((state-fields|internal-contract)\)" crates/kivact-runtime crates/kivact-i18n crates/kivact-style crates/kivact-types crates/kivact-shader crates/kivact-svg crates/kivact-gpu-share` が何も出さない。

## T10：Arvane・bench・producer の印の変換

- **wave / lane / 前提**：W1b / T3 / T09
- **所有**：`crates/arvane/src/**`、`crates/arvane-{native,shell,virtual,text,render,compose,asset}/src/**`、`dev/bench/src/**`、`dev/tests/gpu-interop-producer/src/**`
- **目的**：TL-04（`arvane-text/src/cache.rs` の 52 行を含む）。
- **手順**：共通手順 M を `arvane`、`arvane-asset`、`arvane-compose`、`arvane-native`、`arvane-render`、`arvane-shell`、`arvane-text`、`arvane-virtual`、`kivact-bench`（`dev/bench/src`）、`gpu-interop-producer`（`dev/tests/gpu-interop-producer/src`）の順に行う。
- **消すもの**：これらの全 `xtask-allow(state-fields)`。
- **状態の閉じ方**：共通手順 M2。
- **試験**：なし。
- **確認（tier a）**：

  ```bash
  for p in arvane arvane-asset arvane-compose arvane-native arvane-render arvane-shell arvane-text arvane-virtual kivact-bench gpu-interop-producer; do cargo xtask check --no-write --only state --strict --crate $p || break; done
  cargo fmt -p arvane -p arvane-asset -p arvane-compose -p arvane-native -p arvane-render -p arvane-shell -p arvane-text -p arvane-virtual -p kivact-bench -p gpu-interop-producer
  ```

- **完了条件**：すべて `ok`。`rg -c "xtask-allow\(state-fields\)" crates/arvane* dev/bench dev/tests/gpu-interop-producer` が何も出さない。

## T11：5 つの kit の印の変換

- **wave / lane / 前提**：W1b / T4 / T23
- **所有**：`crates/kivact-ui-{basic,pop,game,creative,soft}/src/**`
- **目的**：TL-04、KIT-16（soft の 96 行、creative の 33 行、basic の 18 行）。
- **手順**：共通手順 M を `kivact-ui-basic`、`kivact-ui-pop`、`kivact-ui-game`、`kivact-ui-creative`、`kivact-ui-soft` の順に行う（`<DIRS>` は各 `crates/<P>/src`）。
- **消すもの**：5 kit の `src` の全 `xtask-allow(state-fields)`。
- **状態の閉じ方**：共通手順 M2。
- **試験**：なし。
- **確認（tier a）**：

  ```bash
  for p in kivact-ui-basic kivact-ui-pop kivact-ui-game kivact-ui-creative kivact-ui-soft; do cargo xtask check --no-write --only state --strict --crate $p || break; done
  cargo fmt -p kivact-ui-basic -p kivact-ui-pop -p kivact-ui-game -p kivact-ui-creative -p kivact-ui-soft
  ```

- **完了条件**：すべて `ok`。`rg -c "xtask-allow\(state-fields\)" crates/kivact-ui-*/src` が何も出さない。

## T12：例・catalog・雛形の印、`//! Role:`、`#![warn(redundant_imports)]`

- **wave / lane / 前提**：W1b / T4 / T11
- **所有**：`examples/kivact/**`、`examples/kivact-ui-{basic,pop,game,creative,soft}/**`、`templates/**`
- **目的**：TL-04、設計 4.3（例と雛形の `#![warn(redundant_imports)]` と `//! Role:`。Role 規則の対象外は T02 の `core_filters.rs`）。
- **手順**：
  1. 共通手順 M を `kivact-example-counter`、`kivact-example-form`、`kivact-example-list`、`kivact-example-async`、`kivact-example-component`、`kivact-ui-basic-catalog`、`kivact-ui-pop-catalog`、`kivact-ui-game-catalog`、`kivact-ui-creative-catalog`、`kivact-ui-soft-catalog`、`kivact-template-basic` の順に行う。`<DIRS>` は各 package の directory（`examples/kivact/<name>`、`examples/kivact-ui-<kit>`、`templates/basic`）。catalog の docs のページ（`examples/kivact-ui-basic/src/docs/{alert_dialog,data_table,dialog,drawer,dropdown_menu,trace}.rs` など）の local の印は「関数」の報告になり、関数の前に 1 行を置く。
  2. 属性と Role を消す。

     ```bash
     rg -l "#!\[warn\(redundant_imports\)\]" examples templates | xargs -r perl -ni -e 'print unless /^\s*#!\[warn\(redundant_imports\)\]\s*$/'
     rg -l "^\s*//! Role: " examples templates | xargs -r perl -pi -e 's{^(\s*)//! Role: }{$1//! }'
     ```
     （Role の行は説明の doc として残し、`Role: ` の語だけを消す。）
- **消すもの**：examples・templates の全 `xtask-allow(state-fields)`、全 `#![warn(redundant_imports)]`（20 ファイル）、全 `//! Role: ` の語。
- **状態の閉じ方**：共通手順 M2。
- **試験**：なし。
- **確認（tier a）**：

  ```bash
  for p in kivact-example-counter kivact-example-form kivact-example-list kivact-example-async kivact-example-component kivact-ui-basic-catalog kivact-ui-pop-catalog kivact-ui-game-catalog kivact-ui-creative-catalog kivact-ui-soft-catalog kivact-template-basic; do cargo xtask check --no-write --only state --strict --crate $p || break; done
  cargo fmt -p kivact-example-counter -p kivact-example-form -p kivact-example-list -p kivact-example-async -p kivact-example-component -p kivact-ui-basic-catalog -p kivact-ui-pop-catalog -p kivact-ui-game-catalog -p kivact-ui-creative-catalog -p kivact-ui-soft-catalog -p kivact-template-basic
  cargo check -p kivact-example-counter -p kivact-template-basic --all-targets
  ```

- **完了条件**：すべて成功。`rg -n "xtask-allow\(state-fields\)|#!\[warn\(redundant_imports\)\]|//! Role:" examples templates` が何も出さない。

---

## T13：`kivact-devtools-client` を `kivact-devtools-protocol` の `client` へ統合

- **wave / lane / 前提**：W1b / T5 / T23
- **所有**：`crates/kivact-devtools-client/`（削除）、`crates/kivact-devtools-protocol/**`、`crates/kivact-devtools-agent/**`、`crates/kivact-devtools-mcp/**`、`crates/kivact-devtools/**`、root の `Cargo.toml` の `members`、`Cargo.lock`、`dev/tests/kivact-public-api/Cargo.toml`、`dev/tests/kivact-public-api/tests/e2e/devtools.rs`
- **目的**：client と protocol を一つの crate にする（TL-08、設計 4.1）。agent の試験の 10 ms の polling を wake の channel に、DevTools の E2E の 1 ms の sleep を `park_timeout` にする（TL-14(3)）。DevTools の 4 crate の state の印を移す（TL-04）。
- **手順**：
  1. 移す。

     ```bash
     mkdir -p crates/kivact-devtools-protocol/src/client
     for f in connection discovery error; do git mv crates/kivact-devtools-client/src/$f.rs crates/kivact-devtools-protocol/src/client/$f.rs; done
     git mv crates/kivact-devtools-client/tests/connection.rs crates/kivact-devtools-protocol/tests/client.rs
     git rm -r -q crates/kivact-devtools-client
     perl -pi -e 's/\bcrate::error::/super::error::/g; s/\bkivact_devtools_protocol::/crate::/g' crates/kivact-devtools-protocol/src/client/*.rs
     perl -pi -e 's/\bkivact_devtools_client::/kivact_devtools_protocol::client::/g' crates/kivact-devtools-protocol/tests/client.rs
     ```
  2. `crates/kivact-devtools-protocol/src/client.rs` を新設する。

     ```rust
     //! Role: Discover running targets and open a DevTools connection to one (feature `client`).

     mod connection;
     mod discovery;
     mod error;

     pub use connection::*;
     pub use discovery::*;
     pub use error::*;
     ```
  3. `crates/kivact-devtools-protocol/src/lib.rs`：`mod call;` の次の行に `#[cfg(feature = "client")]` と `pub mod client;` の 2 行を足す。crate doc の 1 行目を `//! Role: Wire vocabulary of the Kivact DevTools and, with feature \`client\`, the client connection.` にする。
  4. `crates/kivact-devtools-protocol/Cargo.toml`：`description` を `"Wire vocabulary of the Kivact DevTools (JSON-RPC methods, notifications, values) and the client connection (feature client)."` にし、次を足す。

     ```toml
     [features]
     # Discovery and the connection a DevTools client opens; the app-side agent does not need it.
     client = []

     [[test]]
     name = "client"
     path = "tests/client.rs"
     required-features = ["client"]
     ```
  5. 利用者を直す。

     | ファイル | 変更 |
     |---|---|
     | root の `Cargo.toml` | `members` から `"crates/kivact-devtools-client",` の行を消す |
     | `crates/kivact-devtools-agent/Cargo.toml` | `[dev-dependencies]` の `kivact-devtools-client = ...` を `kivact-devtools-protocol = { path = "../kivact-devtools-protocol", version = "0.1.0", features = ["client"] }` にする |
     | `crates/kivact-devtools-mcp/Cargo.toml`、`crates/kivact-devtools/Cargo.toml` | `kivact-devtools-client = ...` の行を消し、`kivact-devtools-protocol = { ... }` に `features = ["client"]` を足す |
     | `dev/tests/kivact-public-api/Cargo.toml` | `kivact-devtools-client = ...` の行を消し、`kivact-devtools-protocol = { path = "../../../crates/kivact-devtools-protocol", features = ["client"] }` にする |
     | `crates/kivact-devtools-mcp/src/target.rs`、`crates/kivact-devtools/src/{connection,source}.rs` | `use kivact_devtools_client as client;` を `use kivact_devtools_protocol::client;` にする |
     | `crates/kivact-devtools-agent/tests/agent.rs`、`dev/tests/kivact-public-api/tests/e2e/devtools.rs` | `use kivact_devtools_client::{` を `use kivact_devtools_protocol::client::{` にする |
  6. `crates/kivact-devtools-agent/tests/agent.rs` の polling を wake で待つ形にする。
     - `fn start_agent()` の後に次を足す。

     ```rust
     /// An agent whose wake sends on the returned channel, so a test waits for arrivals.
     fn start_agent_woken() -> (Agent, mpsc::Receiver<()>) {
         let (sender, receiver) = mpsc::channel();
         let agent = Agent::start(
             AgentConfig { app: sample_app() },
             Box::new(move || {
                 let _ = sender.send(());
             }),
         )
         .expect("invariant: the agent starts");
         (agent, receiver)
     }
     ```
     - `fn wait_closed` を次にする。

     ```rust
     fn wait_closed(agent: &mut Agent, wakes: &mpsc::Receiver<()>) -> usize {
         let deadline = Instant::now() + Duration::from_secs(10);
         loop {
             for incoming in agent.take_incoming() {
                 if let Incoming::Closed { remaining } = incoming {
                     return remaining;
                 }
             }
             let left = deadline.saturating_duration_since(Instant::now());
             assert!(wakes.recv_timeout(left).is_ok(), "no Closed arrived");
         }
     }
     ```
     - `wait_closed(&mut agent)` を呼ぶ試験（`closing_a_connection_reports_the_remaining_clients`）で、`start_agent()` を `start_agent_woken()` にして `let (mut agent, wakes) = start_agent_woken();` とし、呼出しを `wait_closed(&mut agent, &wakes)` にする。
     - `get_app_info_is_answered_on_the_ui_thread` の `let mut agent = start_agent();` を `let (mut agent, wakes) = start_agent_woken();` にし、`let started = Instant::now();` から `};`（`let info = loop { ... };` の終わり）までを次にする。

     ```rust
         let deadline = Instant::now() + Duration::from_secs(10);
         let mut answered = false;
         while !answered {
             let left = deadline.saturating_duration_since(Instant::now());
             assert!(wakes.recv_timeout(left).is_ok(), "the call did not arrive");
             for incoming in agent.take_incoming() {
                 match incoming {
                     Incoming::Call(Call::GetAppInfo(_), responder) => {
                         responder.send(Ok(Reply::GetAppInfo(agent.app().clone())));
                         answered = true;
                     }
                     Incoming::Call(_, responder) => {
                         responder.send(Err(RpcError::new(code::INTERNAL_ERROR, "unexpected call")));
                     }
                     Incoming::Closed { .. } => {}
                 }
             }
         }
         let info = receiver
             .recv_timeout(Duration::from_secs(10))
             .expect("the call was answered");
     ```
  7. `dev/tests/kivact-public-api/tests/e2e/devtools.rs` の `a_devtools_client_inspects_drives_and_reviews_a_test_app` の loop の `std::thread::sleep(Duration::from_millis(1));` を `std::thread::park_timeout(Duration::from_millis(16));` にする（agent の wake が driver を作った thread を unpark する）。
  8. 共通手順 M を `kivact-devtools-protocol`、`kivact-devtools-agent`、`kivact-devtools-mcp`、`kivact-devtools` の順に行う（`<DIRS>` は各 `crates/<P>/src`）。
- **消すもの**：`crates/kivact-devtools-client/`（全体）、`kivact_devtools_client` の名前。
- **状態の閉じ方**：新しい型は無い。移した `client::Connection` などが `state` で報告されたら共通手順 M2。
- **試験**：移す：`crates/kivact-devtools-client/tests/connection.rs` → `crates/kivact-devtools-protocol/tests/client.rs`（試験の名前は変えない）。直す：`crates/kivact-devtools-agent/tests/agent.rs` の `get_app_info_is_answered_on_the_ui_thread`、`closing_a_connection_reports_the_remaining_clients`（期待値は変えない）。
- **確認（tier a）**：

  ```bash
  cargo fmt -p kivact-devtools-protocol -p kivact-devtools-agent -p kivact-devtools-mcp -p kivact-devtools
  cargo clippy -p kivact-devtools-protocol -p kivact-devtools-agent -p kivact-devtools-mcp --all-targets --all-features -- -D warnings
  cargo check -p kivact-devtools -p kivact-public-api-tests --all-targets --all-features
  cargo test -p kivact-devtools-protocol --all-features
  cargo test -p kivact-devtools-agent
  cargo test -p kivact-devtools-mcp
  for p in kivact-devtools-protocol kivact-devtools-agent kivact-devtools-mcp kivact-devtools; do cargo xtask check --no-write --only state --strict --crate $p || break; done
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e --features devtools -- devtools
  ```

- **完了条件**：すべて成功。`rg -n "devtools-client|devtools_client" crates dev Cargo.toml` が何も出さない（`dev/xtask/policy/workspace.toml` は T02 で直してある）。

## T14：`// Regression:` の札、`GRACE` の sleep、kit の README の移行表と recipe の一覧

- **wave / lane / 前提**：W1b / T5 / T13
- **所有**：`dev/tests/kivact-public-api/tests/e2e/{flows,runtime,task,view,gpu_on_demand,gpu_on_demand_shared,gpu_on_demand_windows,theme_conformance}.rs`、`dev/tests/kivact-public-api/tests/e2e/embed/windows.rs`、`crates/kivact-ui-{basic,pop,game,creative,soft}/README.md`
- **目的**：過去の不具合の札を消す（TL-14(1)）。`GRACE` の実時間の sleep を仮想時計にする（TL-14(2)）。README の「Migrating from the previous release」と「Filled recipes」と、それを検査する試験を消す（KIT-16、設計 4.3）。
- **手順**：
  1. `// Regression:` の札を消す。flows.rs・runtime.rs・task.rs・view.rs の札の下にはもう試験が無い（commit `026e7d59` で消えた）。札の塊（`// ---` の区切りの行、`// Regression:` の行、続く `// ` の行、その後の空行 1 つ）を消す。

     ```bash
     cd dev/tests/kivact-public-api/tests/e2e
     perl -0pi -e 's{^// -{10,}\n// Regression:[^\n]*\n(?://(?!/)(?! -)[^\n]*\n)*\n?}{}mg' flows.rs runtime.rs task.rs view.rs
     cd -
     ```
     消した後の各ファイルの末尾が空行で終わっていれば `cargo fmt` が直す。
  2. `embed/windows.rs`：1 行目を `//! Role: Win32 embedding contracts: native hit testing, pointer capture during chords and system-key fallthrough to the host.` にする。`SERIAL.lock().expect("Windows screen regression lock")` を `SERIAL.lock().expect("Windows embed serial lock")` にする。試験は二つとも残す（名前が契約を述べている）。
  3. `GRACE`：
     - `gpu_on_demand.rs`：`/// The real time a device that must not start is given to start.` を `/// The virtual time a device that must not start is given to start.` にする。`fn shader_stays_on_cpu` の `std::thread::sleep(GRACE);` と直後の `screen.run_until_idle();` の 2 行を `screen.advance(GRACE);` の 1 行にする。
     - `gpu_on_demand_shared.rs` の `fn shares_pixels_on_cpu`、`gpu_on_demand_windows.rs` の `fn windows_stay_on_cpu`：同じ 2 行を `screen.advance(GRACE);` にする。
     - 根拠：GPU を起動するかの判断は UI thread の上で同期に決まり（`arvane/src/presenter/gpu.rs` の `RendererStart` と `demand`）、CPU だけの window は worker を作らない。`gpu_on_demand_shared.rs` の `a_shared_image_starts_the_gpu_on_demand` の 5 ms の polling（2 か所）は外部の producer の thread を待つ実時間の待ちなので変えない。
  4. README を消す（各ファイルで一度だけ実行）。

     ```bash
     perl -0pi -e 's/^## Migrating from the previous release\n.*?<!-- \/kivact-recipes -->\n\n?//ms' crates/kivact-ui-basic/README.md crates/kivact-ui-soft/README.md crates/kivact-ui-creative/README.md
     perl -0pi -e 's/^## Migrating from the previous release\n.*?(?=^## Documentation)//ms; s/^## Filled recipes\n.*?<!-- \/kivact-recipes -->\n?//ms' crates/kivact-ui-game/README.md
     perl -0pi -e 's/^## Filled recipes\n.*?(?=^## Documentation)//ms' crates/kivact-ui-pop/README.md
     ```
  5. `theme_conformance.rs`：`fn kit_readme_lists_the_filled_recipes`（`#[test]` ごと）と、それだけが使う `fn filled_recipe_keys`、`fn filled_extended_recipe_keys`、`fn filled_feedback_recipe_keys`、`fn filled_content_recipe_keys` を doc ごと消す。使われなくなった `use` を compile error（unused import の警告）に従って消す。
- **消すもの**：上の札 14 個、5 つの README の 2 種の節、試験 `kit_readme_lists_the_filled_recipes` と 4 つの helper。
- **状態の閉じ方**：新しい型は無い。
- **試験**：消す：`theme_conformance.rs::kit_readme_lists_the_filled_recipes`。直す（期待値は変えない）：`gpu_on_demand.rs::{a_shader_starts_the_gpu_on_demand, a_cpu_window_never_starts_the_gpu_for_a_shader}`、`gpu_on_demand_shared.rs::{a_shared_image_starts_the_gpu_on_demand, a_cpu_window_shares_pixels_without_a_device}`、`gpu_on_demand_windows.rs::{each_window_opened_again_starts_its_own_device_on_demand, cpu_windows_start_no_device_however_often_they_ask}`。
- **確認（tier a）**：

  ```bash
  cargo fmt -p kivact-public-api-tests
  cargo clippy -p kivact-public-api-tests --all-targets --all-features -- -D warnings
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e --all-features -- gpu_on_demand theme_conformance --test-threads=1
  ```

- **完了条件**：すべて成功（GPU の無い環境では GPU の試験は理由を出して skip する）。`rg -n "Regression" dev/tests/kivact-public-api/tests` と `rg -n "kivact-recipes|Migrating from the previous release|Filled (standard )?recipes" crates dev` と `rg -n "thread::sleep\(GRACE\)" dev` が何も出さない。

## T15：E2E を 4 つの binary に分ける

- **wave / lane / 前提**：W1b / T5 / T14
- **所有**：`dev/tests/kivact-public-api/**`、`dev/tests/kivact-kit-tests/`（新設）、root の `Cargo.toml` の `members`、`Cargo.lock`、`dev/xtask/policy/workspace.toml` の `[[style.exceptions]]`（1 件足す）
- **目的**：単一の `e2e` を `e2e_core`・`e2e_ui`・`e2e_native`（`kivact-public-api-tests`）と `e2e_kits`（新しい package `kivact-kit-tests`）に分ける（TL-15、設計 6.4、DEC-14）。`e2e_kits` を別の package にするのは、Cargo の dev-dependency が package のすべての test target に compile されるため（4.6 節の 2、親が採用）。
- **手順**：
  1. 共有の helper を package の lib へ移す。

     ```bash
     cd dev/tests/kivact-public-api
     git mv tests/e2e/support.rs src/support.rs
     git mv tests/e2e/support src/support
     perl -pi -e 's/^pub\(crate\) mod /pub mod /; s/^pub\(crate\) use /pub use /' src/support.rs
     perl -pi -e 's/\bpub\(crate\) /pub /g' src/support/*.rs
     cd -
     ```
  2. `dev/tests/kivact-public-api/src/paint.rs` を新設する（`tests/e2e/paint.rs` の helper を写す）。

     ```rust
     //! Role: Pixel helpers the E2E binaries share: colors, boxes on a canvas and comparisons.

     use kivact::prelude::*;
     use kivact::style::{Color, Position, Style};
     use kivact::test::Screenshot;

     pub const RED: u32 = 0xff0000;
     pub const BLUE: u32 = 0x0000ff;
     pub const GREEN: u32 = 0x00ff00;
     pub const WHITE: u32 = 0xffffff;
     pub const BLACK: u32 = 0x000000;

     /// A white 800 × 600 canvas holding `children`, each placed with [`at`].
     pub fn canvas_of(children: impl IntoView) -> Node {
         node()
             .child(children)
             .style(style().size(800, 600).background(Color::WHITE))
     }

     /// A box of `w × h` at `(x, y)` of the canvas.
     pub fn at(x: i32, y: i32, w: i32, h: i32) -> Style {
         style()
             .position(Position::Absolute)
             .left(x)
             .top(y)
             .size(w, h)
     }

     /// The color of a pixel (0xRRGGBB).
     #[track_caller]
     pub fn pixel(shot: &Screenshot, x: u32, y: u32) -> u32 {
         shot.pixel(x, y).expect("in the frame")
     }

     /// The largest channel difference between two colors.
     pub fn distance(a: u32, b: u32) -> u32 {
         (0..3)
             .map(|i| ((a >> (8 * i)) & 0xff).abs_diff((b >> (8 * i)) & 0xff))
             .max()
             .unwrap_or(0)
     }

     /// The pixel at `(x, y)` is `want`, within `tolerance` per channel.
     #[track_caller]
     pub fn expect(shot: &Screenshot, x: u32, y: u32, want: u32, tolerance: u32, what: &str) {
         let got = pixel(shot, x, y);
         assert!(
             distance(got, want) <= tolerance,
             "{what}: pixel ({x}, {y}) is {got:06x}, want {want:06x} (±{tolerance})"
         );
     }
     ```
     `tests/e2e/paint.rs` から、`pub(crate) const RED` の行から `pub(crate) fn expect` の終わりの `}` までを消す（3 つの試験と `gradient_fixture` は残す）。
  3. `dev/tests/kivact-public-api/src/lib.rs`：先頭の 2 行（`//! Role: Public API consumer tests. The tests live in tests/ (e2e,` と `//! offscreen); code that must not compile is checked by the doc tests below.`）を次の 3 行にし、ファイルの末尾に `pub mod paint;` と `pub mod support;` を足す。

     ```rust
     //! Role: Public API consumer tests and the E2E helpers their test binaries share.
     //! The tests live in tests/ (e2e_core, e2e_ui, e2e_native and offscreen); code
     //! that must not compile is checked by the doc tests below.
     ```
  4. helper の path を書き換える（`embed/` の中は自分の `support` を持つので除く）。

     ```bash
     cd dev/tests/kivact-public-api/tests/e2e
     rg -l "(crate|super)::(support|paint)::" . --glob '!embed/**' | xargs -r perl -pi -e 's/\b(?:crate|super)::(support|paint)::/kivact_public_api_tests::$1::/g'
     cd -
     ```
     `paint.rs`（試験の残り）には compile error に従って `use kivact_public_api_tests::paint::{...};` を足す。
  5. `canvas.rs` の試験 `snapshot_of_a_gpu_frame_is_unsupported`（直前の `#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]` と `#[test]` を含む）を切り取り、`gpu_interop.rs` の末尾に `#[test]` から貼る（`#[cfg(..)]` の行は貼らない。module が同じ cfg を持つ）。貼った中の `super::gpu_interop::unread_frame()` を `unread_frame()` にする。足りない `use` は compile error に従って足す。
  6. kit を使う試験を新しい package へ分ける。
     - `apg_grid.rs` から試験 `accessory_controls_share_one_tab_stop_and_remain_keyboard_operable` と `group_disclosures_participate_in_grid_arrow_navigation` を切り取り、`dev/tests/kivact-kit-tests/tests/e2e_kits/apg_grid_kit.rs` に置く。二つが呼ぶ `apg_grid.rs` の private な関数は、`apg_grid.rs` に残る試験も呼ぶなら写し、呼ばないなら移す。ファイルの先頭は `//! Role: APG grid behavior with a kit's theme installed.` と、`apg_grid.rs` の `use` のうち使うもの。
     - `presence.rs` から試験 `toast_swipe_dismiss_exits_from_the_drag_offset` と `toast_row_shrinks_while_it_exits` と、関数 `toast_options`、`top_where`、`red_top`、`tinted_top` を移し、`advance` を写して `dev/tests/kivact-kit-tests/tests/e2e_kits/presence_kit.rs` に置く（先頭は `//! Role: Toast presence with a kit's theme installed.`）。
     - `git mv dev/tests/kivact-public-api/tests/e2e/theme_conformance.rs dev/tests/kivact-kit-tests/tests/e2e_kits/theme_conformance.rs`、同じく `ui_standard.rs`。
     - 4 ファイルに手順 4 の perl を同じ形で行う（`crate::`・`super::` の `support`・`paint` を `kivact_public_api_tests::` に）。
     - `dev/tests/kivact-kit-tests/Cargo.toml` を新設する。

     ```toml
     [package]
     name = "kivact-kit-tests"
     version.workspace = true
     edition.workspace = true
     license.workspace = true
     publish = false
     autotests = false
     description = "Kit E2E tests: the five official kits against the standard components (e2e_kits)"

     [features]
     default = ["gpu", "complex-scripts"]
     complex-scripts = ["kivact/complex-scripts"]
     gpu = ["kivact/gpu"]

     [dependencies]
     kivact = { path = "../../../crates/kivact", default-features = false, features = ["test"] }

     [dev-dependencies]
     kivact-public-api-tests = { path = "../kivact-public-api" }
     kivact-ui-basic = { path = "../../../crates/kivact-ui-basic" }
     kivact-ui-creative = { path = "../../../crates/kivact-ui-creative" }
     kivact-ui-game = { path = "../../../crates/kivact-ui-game" }
     kivact-ui-pop = { path = "../../../crates/kivact-ui-pop" }
     kivact-ui-soft = { path = "../../../crates/kivact-ui-soft" }

     [[test]]
     name = "e2e_kits"
     path = "tests/e2e_kits/main.rs"

     [lints]
     workspace = true
     ```
     - `dev/tests/kivact-kit-tests/src/lib.rs`：`//! Role: Kit end-to-end tests; the tests live in tests/e2e_kits.` の 1 行だけ。
     - `dev/tests/kivact-kit-tests/tests/e2e_kits/main.rs`：

     ```rust
     //! Role: End-to-end tests of the five official kits through the public API.

     mod apg_grid_kit;
     mod presence_kit;
     mod theme_conformance;
     mod ui_standard;
     ```
  7. 残りの module を 3 つの directory に分ける。

     ```bash
     cd dev/tests/kivact-public-api/tests
     mkdir -p e2e_core e2e_ui e2e_native
     for m in a11y_incremental app_handle assets async_tests binding_update borders_backgrounds canvas canvas_worker close_request color_spaces component_macro components compositor custom_text_field diagnostics disposal drawables effects_filters environment flows focus frame_sources grid hover i18n image_semantics images_animated input_platform keyboard keys_unhandled landmark_roles launch_fonts launch_platform launch_real_window launch_support launch_window layout layout_check layout_scroll layout_units lifecycle machine motion motion_easing node_ref offscreen offscreen_svg overlay paint parts path_algebra platform_paths pointer presence presence_modes primitive relations resource_reclaim runtime scroll_timeline setup_functions shaders shared_layout style_paint svg_embedded_text task text_editing text_effects text_primitives text_style timeline touch_scroll transform transform_3d view viewport windows; do git mv e2e/$m.rs e2e_core/$m.rs; done
     for m in apg_accordion apg_alert apg_breadcrumb apg_carousel apg_combobox apg_date_picker apg_grid apg_listbox apg_menu apg_menu_button apg_menubar apg_radio_group apg_slider apg_spinbutton apg_tabs apg_toolbar apg_tooltip apg_tree_view apg_window_splitter interaction_calendar interaction_combobox interaction_delayed interaction_dialog interaction_drag_edit interaction_edit interaction_list_nav interaction_menu interaction_palette interaction_select interaction_show_hide interaction_slider interaction_splitter interaction_state interaction_table interaction_tabs interaction_toast interaction_toggle interaction_tooltip interaction_tree interaction_zoom_pan ui_calendar ui_carousel ui_chat_code ui_collections ui_combobox ui_command_palette ui_components ui_controls ui_data_table ui_disclosure_surfaces ui_dropzone ui_field ui_navigation ui_sparkline ui_theming ui_toolbar feedback indicators metrics_and_description reactive_props; do git mv e2e/$m.rs e2e_ui/$m.rs; done
     for m in devtools drawable_export_gpu embed foreign_window gpu_interop gpu_on_demand gpu_on_demand_shared gpu_on_demand_windows viewport_gpu_interop; do git mv e2e/$m.rs e2e_native/$m.rs; done
     git mv e2e/embed e2e_native/embed
     git rm -q e2e/main.rs
     ls e2e 2>/dev/null && echo "STOP: files left in tests/e2e"
     cd -
     ```
     最後の行が `STOP` を出したら、残ったファイルの名前を STOP の行に書く。
  8. 3 つの `main.rs` を新設する。
     - `tests/e2e_core/main.rs`：1 行目 `//! Role: End-to-end tests of the reactive core, views, style, layout, text, input and a11y through kivact.`、空行、手順 7 の 1 つ目の列の各名前を `mod <name>;` の行にしてアルファベット順に並べる。
     - `tests/e2e_ui/main.rs`：1 行目 `//! Role: End-to-end tests of kivact::ui, kivact::interaction and the APG patterns through kivact.`、2 つ目の列を同じく。
     - `tests/e2e_native/main.rs`：1 行目 `//! Role: End-to-end tests of embedding, GPU interop, foreign windows and DevTools.`、空行、次。

     ```rust
     #[cfg(feature = "devtools")]
     mod devtools;
     #[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
     mod drawable_export_gpu;
     #[cfg(feature = "embed")]
     mod embed;
     mod foreign_window;
     #[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
     mod gpu_interop;
     mod gpu_on_demand;
     #[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
     mod gpu_on_demand_shared;
     #[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
     mod gpu_on_demand_windows;
     #[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
     mod viewport_gpu_interop;
     ```
     （`embed` は top-level の module のまま。`embed/panic_boundary.rs` が `--exact "embed::panic_boundary::..."` で自分を起動し直すため。）
  9. `dev/tests/kivact-public-api/Cargo.toml`：
     - `description` を `"Public Kivact API consumer tests: E2E binaries e2e_core, e2e_ui and e2e_native, and the windowless Offscreen tests"` にする。
     - `kivact = { ... features = ["test"] }` を `[dev-dependencies]` から新しい `[dependencies]` の表へ移す（lib の helper が使う）。
     - `[dev-dependencies]` の kit 5 行とその上の comment（`# The standard components are drawn like a kit's own ...`）を消す。
     - comment の `tests/e2e/` を、`devtools.rs`・`gpu_interop.rs`・`drawable_export_gpu.rs` は `tests/e2e_native/`、`images_animated.rs` は `tests/e2e_core/` にする。
     - `[[test]] name = "e2e" path = "tests/e2e/main.rs"` を次の 3 つにする。

     ```toml
     [[test]]
     name = "e2e_core"
     path = "tests/e2e_core/main.rs"

     [[test]]
     name = "e2e_ui"
     path = "tests/e2e_ui/main.rs"

     [[test]]
     name = "e2e_native"
     path = "tests/e2e_native/main.rs"
     ```
  10. 文書の path：`tests/offscreen.rs` の `` `e2e/offscreen.rs` `` を `` `e2e_core/offscreen.rs` `` にする。`tests/e2e_core/disposal.rs` の `(Registering signals, effects and tasks on a disposing owner needs the reactive owner and is tested in \`kivact-core\`'s \`disposal_tests\`.)` を `(Registering signals, effects and tasks on a disposing owner needs the reactive owner, which the public API does not expose.)` にする（文が 2 行にまたがるので、2 行をまとめて置き換える）。
  11. root の `Cargo.toml` の `members` の `"dev/tests/kivact-public-api",` の次の行に `"dev/tests/kivact-kit-tests",` を足す。
  12. `dev/xtask/policy/workspace.toml`：`path = "dev/tests/kivact-public-api/*"` の `glob-imports` の例外の直後に次を足す。

     ```toml
     [[style.exceptions]]
     rule = "glob-imports"
     path = "dev/tests/kivact-kit-tests/*"
     reason = "Kit E2E tests are written from the user's side and import `kivact::prelude::*`."
     ```
  13. 共通手順 M を `kivact-public-api-tests` と `kivact-kit-tests` に行う。
- **消すもの**：`tests/e2e/`（directory）、`tests/e2e/main.rs`、`[[test]] name = "e2e"`、`kivact-public-api-tests` の kit の dev-dependency 5 つ。
- **状態の閉じ方**：新しい型は無い。
- **試験**：移すだけで名前を変えない（手順 5 の試験は `e2e_native::gpu_interop` へ、手順 6 の 4 つは `kivact-kit-tests` の `e2e_kits` へ）。`paint.rs` の 3 つの試験は `e2e_core` に残る。
- **確認（tier a）**：

  ```bash
  cargo fmt -p kivact-public-api-tests -p kivact-kit-tests
  cargo clippy -p kivact-public-api-tests -p kivact-kit-tests --all-targets --all-features -- -D warnings
  cargo test -p kivact-public-api-tests --all-features --no-run
  cargo test -p kivact-kit-tests --no-run
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- paint layout_units
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-kit-tests --test e2e_kits -- theme_conformance
  cargo xtask check --no-write --only state --strict --crate kivact-public-api-tests
  cargo xtask check --no-write --only state --strict --crate kivact-kit-tests
  ```

- **完了条件**：すべて成功。手順 1 の前（T14 の後の状態）に `cargo test -p kivact-public-api-tests --test e2e --all-features -- --list 2>/dev/null | grep -c ': test$'` で数えた数が、分割後の `cargo test -p kivact-public-api-tests --test e2e_core --test e2e_ui --test e2e_native --all-features -- --list 2>/dev/null | grep -c ': test$'` と `cargo test -p kivact-kit-tests --test e2e_kits -- --list 2>/dev/null | grep -c ': test$'` の合計と等しい（手順 1 の前に数を記録しておく）。`rg -n "\-\-test e2e\b|tests/e2e/" dev crates` が `dev/README.md`・`dev/tools/window-lock.sh`（T19・T21 が直す）、`crates/kivact-core/src/custom.rs`（T07 が直す）、`crates/kivact-test/src/lib.rs`（T16 が直す）、`crates/arvane`（Arvane 自身の `e2e`）の外で何も出さない。

---

## T16：`kivact-test` の自己試験の移動と削除

- **wave / lane / 前提**：W1b / T6 / T23
- **所有**：`crates/kivact-test/**`
- **目的**：crate の doc（「no unit test of internal modules」）と食い違う `src/screen/*_tests.rs` を、公開 API だけを使うものは `tests/` へ移し、内部を確かめるものは消す（TL-12）。印を移す（TL-04、TL-05）。
- **手順**：
  1. ファイルごとの扱い。

     | ファイル | 扱い | 理由 |
     |---|---|---|
     | `src/screen/accessibility_tests.rs` | 削除 | engine の `Writer` を直に使い、公開 API で作れない root のない host の publication を確かめる |
     | `src/screen/context_tests.rs` | `tests/pure_context.rs` へ移す | 公開の `render_with` と runtime の `pure`・`untrack`・`memo` だけを使う |
     | `src/screen/disposal_tests.rs` | 削除 | runtime の内部の `Owner::{current, signal, with_setup}` を使う。公開の範囲は `e2e_core/disposal.rs` が確かめる |
     | `src/screen/editing_drop_probe.rs` | 削除 | `host_sync_tests` だけが使う engine の reader の probe |
     | `src/screen/gpu_tests.rs` | `tests/gpu.rs` へ移す | GPU と CPU の frame の一致は公開の `RenderOptions::gpu` と `Screen::renderer` で確かめられる |
     | `src/screen/host_sync_tests.rs` | 削除 | 生の `HostEvent` の注入と同じ turn の queue 入力（非公開）で host の内部を確かめる |
     | `src/screen/real_window_tests.rs` | 削除 | 4 つの試験すべてが `with_host`（`set_now`・`set_display_period`・`redraw_requested`）か `display_stats` を使う |
     | `src/screen/scene_cache_tests.rs` | 削除 | scene・raster・font の cache の内部を `ArvaneTestHost` で直に確かめる |
     | `src/screen/window_tests.rs` | 削除 | `with_host(|h| h.min_window_size(..))` だけで確かめる（公開の getter が無い） |
     | `src/screen/support.rs` | 削除 | 上の試験だけが使う helper（`gpu.rs` は `button_box` を写す） |

     ```bash
     cd crates/kivact-test
     mkdir -p tests
     git mv src/screen/context_tests.rs tests/pure_context.rs
     git mv src/screen/gpu_tests.rs tests/gpu.rs
     git rm -q src/screen/{accessibility_tests,disposal_tests,editing_drop_probe,host_sync_tests,real_window_tests,scene_cache_tests,window_tests,support}.rs
     cd -
     ```
  2. `src/screen.rs`：`#[cfg(test)]` と `mod accessibility_tests;` … `mod window_tests;` の 10 組（20 行）を消す。
  3. `src/screen/driver.rs`：`/// The test driver (internal tests).`、`#[cfg(test)]` と `pub(crate) fn with_host<R>(...)` の関数全体を消す。`src/screen/input.rs`：`/// Input queued without settling (internal tests): ...` の doc の 3 行と、`#[cfg(test)] impl Screen { queue_click, queue_text, queue_host_failure }` の塊全体を消す。`cargo clippy -p kivact-test --all-targets -- -D warnings` が他に `#[cfg(test)]` の item の `dead_code` を報告したら、その item を消す。
  4. `tests/pure_context.rs`：1 行目を `//! Role: A pure context rejects writes, keeps memo tracking and reports memo cycles in a real window.` にし、`use crate::{MirrorMode, RenderOptions, render_with, render_with_handle};` を `use kivact_test::{MirrorMode, RenderOptions, render_with, render_with_handle};` にする。
  5. `tests/gpu.rs`：
     - 9 行目の `//! public \`RenderOptions::gpu\` and verify \`Screen::renderer\`; these internal` から `//! helpers additionally inspect the shell surface directly.` までを `//! public \`RenderOptions::gpu\` and verify \`Screen::renderer\`.` にする。
     - `use std::panic::Location;`、`use super::model::Screen;`、`use super::results::Screenshot;`、`use super::support::button_box;`、`use crate::{MirrorMode, RenderOptions};`、`use kivact_types::WindowId;` を消し、`use kivact_test::{MirrorMode, RenderOptions, Screenshot, render_with};` を足す。`use kivact_engine::testing::{Renderer, gpu_unavailable};` を `use kivact_test::{Renderer, gpu_unavailable};` にする。
     - `Screen::start(options, Vec::new(), Box::new(scene), Location::caller())` を `render_with(options, scene)` にする。`Screen::start(\n options,\n Vec::new(),\n Box::new(<closure>),\n Location::caller(),\n )` の 2 か所（`gpu_host_state_transition_matches_the_cpu_intermediate_frame` と `gpu_frames_follow_a_scale_change`）を `render_with(options, <closure>)` にする。
     - `screen.with_host(|host| host.renderer(WindowId::MAIN))` と `gpu.with_host(|host| host.renderer(WindowId::MAIN))` を `Some(screen.renderer())`・`Some(gpu.renderer())` にする（`Option` の比較はそのまま）。
     - `super::failure_dir()` を `kivact_test::failure_dir()` にする。
     - `include_bytes!("../../../../dev/tests/kivact-public-api/tests/fixtures/rgba.png")` と `.../shapes.svg` の `../../../../` を `../../../` にする。
     - ファイルの末尾に次を足す（足りない `use` は compile error に従って足す）。

     ```rust
     /// A focusable button-like box labelled `label` (as the deleted `screen/support.rs` made it).
     fn button_box(label: &'static str) -> Node {
         node()
             .role(Role::Button)
             .focusable(true)
             .name(label)
             .child(text(label))
     }
     ```
     `use kivact_core::view::{IntoView, column, node, row, text};` を `use kivact_core::view::{IntoView, Node, Role, column, node, row, text};` にする。
  6. `src/lib.rs` の `` `dev/tests/kivact-public-api/tests/e2e/layout_check.rs` `` を `` `dev/tests/kivact-public-api/tests/e2e_core/layout_check.rs` `` にする。
  7. `src/screen/results.rs` の `// xtask-allow(api-shape): existing test diagnostic contract reports the original builder location.` と `// xtask-allow(api-shape): semantic snapshots retain the nearest builder location for diagnostics.` の 2 行を消す（`&'static Location` は規則が認める。T02）。
  8. 共通手順 M を `kivact-test`（`crates/kivact-test/src`）に行う。
- **消すもの**：上の表の削除 8 ファイル、`Screen::with_host`、`Screen::{queue_click, queue_text, queue_host_failure}`、`results.rs` の `api-shape` の印 2 行。
- **状態の閉じ方**：新しい型は無い。
- **試験**：移す：`pure_setup_rejects_writes_even_through_untrack`、`pure_section_inside_a_memo_keeps_tracking`、`a_memo_cycle_through_copied_contexts_panics`（→ `tests/pure_context.rs`）、`gpu_frames_in_a_real_window_match_cpu_frames`、`gpu_host_state_transition_matches_the_cpu_intermediate_frame`、`gpu_frames_follow_a_scale_change`（→ `tests/gpu.rs`）。消す：`empty_real_empty_and_replaced_roots_publish_without_stale_nodes`、`a_cleanup_writes_signals_and_registers_on_its_disposing_owner`、`rejected_pointer_down_cannot_retain_an_ended_region_gesture`、`a_primary_down_of_the_gesture_pointer_releases_a_gesture_whose_up_was_lost`、`input_with_host_failure_keeps_host_core_and_binding_in_sync`、`input_with_host_failure_and_typing_after_a_click`、`host_edit_after_a_programmatic_reset_in_the_same_turn_is_kept`、`a_host_edit_that_overrides_the_only_pending_prop_sends_nothing`、`replacing_editing_drops_line_state_after_releasing_the_model`、`virtual_time_drives_sleep_and_frame_loops_exactly`、`a_display_rate_loop_follows_the_window_to_a_faster_display`、`a_minimized_window_asks_for_no_redraws_until_it_is_restored`、`failures_leave_artifacts_windows_are_pooled_and_tests_run_in_parallel`、`pending_canvas_assets_complete_after_a_presented_frame_and_remain_live_on_reuse`、`registering_a_font_after_a_cached_canvas_frame_replaces_fallback_shaping`、`a_successful_old_svg_raster_is_invalidated_when_the_worker_completes_its_replacement`、`a_depth_fallback_reports_a_dropped_local_command_only_once`、`the_minimum_window_size_reaches_the_window`。
- **確認（tier a）**：

  ```bash
  cargo fmt -p kivact-test
  cargo clippy -p kivact-test --all-targets --all-features -- -D warnings
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-test --test pure_context --test gpu
  cargo xtask check --no-write --only state,api-shape --strict --crate kivact-test
  ```

- **完了条件**：すべて成功（GPU が無ければ `gpu` の 3 つは理由を出して skip）。`rg -n "#\[cfg\(test\)\]" crates/kivact-test/src/screen.rs` が何も出さず、`ls crates/kivact-test/src/screen/*_tests.rs` が何も出さない。

## T17：docs の doctest を英語だけにし、`no_run` を型検査の module にする

- **wave / lane / 前提**：W1b / T6 / T16
- **所有**：`dev/tests/docs/**`、`docs/for-kivact-user/jp/how-to-guide/canvas.md`、`docs/for-kivact-user/jp/how-to-guide/windows.md`
- **目的**：英語の木と `README.md` だけを doctest にし、`no_run` の block は link しない型検査の module にする（TL-03、設計 6.4）。日本語の 2 ファイルを `docs-pairs` に合わせる。`dev/tests/docs` の `//! Role:` を消す（設計 4.3）。
- **手順**：
  1. `dev/tests/docs/build.rs` を次に置き換える。

     ```rust
     //! Generates the doctest modules and the type-checked `no_run` blocks of the English guides.
     //!
     //! Every `.md` file of the English user and kit guides, and `README.md`,
     //! becomes an empty module whose documentation is a copy of that file, so
     //! `cargo test -p kivact-docs --doc -- for_kivact_ui_user_en::basic` runs the
     //! blocks of one subtree. In the copy each `rust,no_run` block is marked
     //! `ignore`; its code goes instead into one module of `checked.rs`, which
     //! `cargo check -p kivact-docs --all-targets` type-checks without linking.
     //! The Japanese guides are not compiled: `cargo xtask check` (`docs-pairs`)
     //! keeps their Rust blocks equal to the English ones. A new guide needs no
     //! registration here.

     use std::fmt::Write as _;
     use std::fs;
     use std::path::{Path, PathBuf};

     /// The guide trees that hold Rust examples (English only).
     const TREES: [&str; 2] = ["for-kivact-user/en", "for-kivact-ui-user/en"];

     /// Guides (by file name) whose examples need a feature of this crate.
     const FEATURE_GUIDES: [(&str, &str); 1] = [("gpu-texture-sharing.md", "gpu-interop")];

     /// What the build writes: the doctest modules and the checked blocks.
     #[derive(Default)]
     struct Output {
         guides: String,
         checked: String,
     }

     /// Where the reader is among the fences of one guide.
     enum Fence {
         Outside,
         Other,
         NoRun(String),
     }

     fn main() {
         let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("set by Cargo"));
         let root = manifest.join("../../..");
         let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("set by Cargo"));
         let copies = out.join("guides");
         fs::create_dir_all(&copies).expect("OUT_DIR is writable");
         let mut output = Output::default();
         let readme = root.join("README.md");
         println!("cargo::rerun-if-changed={}", readme.display());
         guide(&mut output, &copies, "readme", "readme", &readme, None);
         for tree in TREES {
             let dir = root.join("docs").join(tree);
             println!("cargo::rerun-if-changed={}", dir.display());
             let name = module_name(tree);
             directory(&mut output, &copies, &name, &name, &dir);
         }
         fs::write(out.join("guides.rs"), output.guides).expect("OUT_DIR is writable");
         fs::write(out.join("checked.rs"), output.checked).expect("OUT_DIR is writable");
     }

     /// `name` is the nested module of `dir`; `prefix` its flat path for the checked blocks.
     fn directory(output: &mut Output, copies: &Path, name: &str, prefix: &str, dir: &Path) {
         let mut entries: Vec<PathBuf> = fs::read_dir(dir)
             .expect("guide directory is readable")
             .map(|entry| entry.expect("guide directory is readable").path())
             .collect();
         entries.sort();
         let label = dir
             .file_name()
             .and_then(|n| n.to_str())
             .expect("UTF-8 directory name");
         let _ = writeln!(
             output.guides,
             "#[doc = \"Guides in `{label}/`.\"]\npub mod {name} {{"
         );
         for path in entries {
             let file = path
                 .file_name()
                 .and_then(|n| n.to_str())
                 .expect("UTF-8 file name");
             if path.is_dir() {
                 let child = module_name(file);
                 let flat = format!("{prefix}_{}", child.trim_start_matches('_'));
                 directory(output, copies, &child, &flat, &path);
             } else if let Some(stem) = file.strip_suffix(".md") {
                 let feature = FEATURE_GUIDES
                     .iter()
                     .find(|(guide, _)| *guide == file)
                     .map(|(_, feature)| *feature);
                 let child = module_name(stem);
                 let flat = format!("{prefix}_{}", child.trim_start_matches('_'));
                 guide(output, copies, &child, &flat, &path, feature);
             }
         }
         output.guides.push_str("}\n");
     }

     /// Write the doctest module of one guide and the checked modules of its `no_run` blocks.
     fn guide(
         output: &mut Output,
         copies: &Path,
         name: &str,
         prefix: &str,
         path: &Path,
         feature: Option<&str>,
     ) {
         let text = fs::read_to_string(path).expect("guide is readable");
         let (copy, blocks) = split_blocks(&text);
         let copy_path = copies.join(format!("{prefix}.md"));
         fs::write(&copy_path, copy).expect("OUT_DIR is writable");
         let gate = feature
             .map(|feature| format!("#[cfg(feature = \"{feature}\")]\n"))
             .unwrap_or_default();
         let _ = writeln!(
             output.guides,
             "{gate}#[doc = include_str!(\"{}\")]\npub mod {name} {{}}",
             slash(&copy_path)
         );
         for (index, block) in blocks.iter().enumerate() {
             let _ = writeln!(
                 output.checked,
                 "{gate}mod {prefix}_block_{} {{\n{}\n}}",
                 index + 1,
                 checked_body(block)
             );
         }
     }

     /// The guide with each `rust,no_run` fence marked `ignore`, and the bodies of those blocks.
     fn split_blocks(text: &str) -> (String, Vec<String>) {
         let mut copy = String::new();
         let mut blocks = Vec::new();
         let mut fence = Fence::Outside;
         for line in text.lines() {
             let info = line.strip_prefix("```");
             fence = match (fence, info) {
                 (Fence::Outside, Some(info)) if is_no_run(info) => {
                     let _ = writeln!(copy, "```{}", info.replacen("no_run", "ignore", 1));
                     Fence::NoRun(String::new())
                 }
                 (Fence::Outside, Some(_)) => {
                     let _ = writeln!(copy, "{line}");
                     Fence::Other
                 }
                 (Fence::Other, Some(_)) => {
                     let _ = writeln!(copy, "{line}");
                     Fence::Outside
                 }
                 (Fence::NoRun(body), Some(_)) => {
                     blocks.push(body);
                     let _ = writeln!(copy, "{line}");
                     Fence::Outside
                 }
                 (Fence::NoRun(mut body), None) => {
                     let _ = writeln!(body, "{line}");
                     let _ = writeln!(copy, "{line}");
                     Fence::NoRun(body)
                 }
                 (state, None) => {
                     let _ = writeln!(copy, "{line}");
                     state
                 }
             };
         }
         (copy, blocks)
     }

     fn is_no_run(info: &str) -> bool {
         let attrs: Vec<&str> = info.split(',').map(str::trim).collect();
         attrs.first() == Some(&"rust") && attrs.contains(&"no_run")
     }

     /// The block as module items: hidden-line markers removed, and the code wrapped
     /// in `fn main` the way rustdoc wraps a block that has none.
     fn checked_body(block: &str) -> String {
         let lines: Vec<String> = block.lines().map(unhide).collect();
         let code = lines.join("\n");
         let has_main = lines.iter().any(|line| {
             let line = line.trim_start();
             line.starts_with("fn main(")
                 || line.starts_with("async fn main(")
                 || line.starts_with("pub fn main(")
         });
         if has_main {
             return code;
         }
         let returns_result = lines
             .iter()
             .rev()
             .map(|line| line.trim())
             .find(|line| !line.is_empty())
             .is_some_and(|last| last.starts_with("Ok::<"));
         if returns_result {
             format!(
                 "fn main() {{\n    fn _inner() -> core::result::Result<(), impl core::fmt::Debug> {{\n{code}\n    }}\n    let _ = _inner();\n}}"
             )
         } else {
             format!("fn main() {{\n{code}\n}}")
         }
     }

     /// A line without rustdoc's hidden-line marker (`# ` hides a line; `##` writes `#`).
     fn unhide(line: &str) -> String {
         let trimmed = line.trim_start();
         if trimmed == "#" {
             String::new()
         } else if let Some(rest) = trimmed.strip_prefix("# ") {
             rest.to_owned()
         } else if let Some(rest) = trimmed.strip_prefix('#').filter(|rest| rest.starts_with('#')) {
             rest.to_owned()
         } else {
             line.to_owned()
         }
     }

     /// A Rust identifier for a file or directory name (`01-first-app` → `_01_first_app`).
     fn module_name(file: &str) -> String {
         let stem = file.strip_suffix(".md").unwrap_or(file);
         let mut name: String = stem
             .chars()
             .map(|c| {
                 if c.is_ascii_alphanumeric() {
                     c.to_ascii_lowercase()
                 } else {
                     '_'
                 }
             })
             .collect();
         if name.starts_with(|c: char| c.is_ascii_digit()) {
             name.insert(0, '_');
         }
         name
     }

     fn slash(path: &Path) -> String {
         let path = fs::canonicalize(path).expect("guide path exists");
         let text = path.to_str().expect("UTF-8 path").replace('\\', "/");
         // `canonicalize` on Windows returns a verbatim `//?/` path; include_str! accepts the plain one.
         text.strip_prefix("//?/").unwrap_or(&text).to_string()
     }
     ```
  2. `dev/tests/docs/src/checked.rs` を新設する。

     ```rust
     //! The `no_run` blocks of the English guides as modules, type-checked and never run.

     /// Includes the modules the build script generated.
     macro_rules! generated_blocks {
         () => {
             include!(concat!(env!("OUT_DIR"), "/checked.rs"));
         };
     }

     generated_blocks!();
     ```
  3. `dev/tests/docs/src/lib.rs` を次に置き換える。

     ```rust
     //! Doctest harness for the English Markdown guides in `docs/` and `README.md`.
     //!
     //! The build script includes every English guide as the documentation of an
     //! empty module under [`guides`], so `cargo test -p kivact-docs --doc` compiles
     //! and runs every ```` ```rust ```` block that is not `no_run`. The `no_run`
     //! blocks (which open a window) are compiled instead as the `checked` modules,
     //! which `cargo check -p kivact-docs --all-targets` type-checks without linking.
     //! Blocks that are not Rust use ```` ```text ````. The Japanese guides are kept
     //! equal to the English ones by `cargo xtask check` (`docs-pairs`).
     //!
     //! A block that shows an app together with its `#[cfg(test)] mod tests` is
     //! `test_harness`: rustdoc builds it with `--test`, so the tests compile and
     //! run while `main` (which opens a window) is never called. A unit test of
     //! this crate rejects such blocks under `no_run`.

     /// One module per guide, mirroring the paths under `docs/`.
     pub mod guides;

     /// The `no_run` blocks of the guides, type-checked and never called.
     #[cfg(test)]
     #[allow(
         dead_code,
         unused,
         unreachable_pub,
         non_snake_case,
         clippy::all,
         clippy::pedantic,
         clippy::todo,
         clippy::unimplemented,
         clippy::too_many_lines,
         clippy::unwrap_used,
         clippy::print_stdout,
         clippy::print_stderr,
         clippy::dbg_macro,
         reason = "guide examples are written for readers; they are compiled here only to type-check them"
     )]
     mod checked;

     #[cfg(test)]
     mod tests;
     ```
  4. `dev/tests/docs/src/guides.rs` と `src/tests.rs` の 1 行目 `//! Role: ` の `Role: ` を消す（`perl -pi -e 's{^//! Role: }{//! }' dev/tests/docs/src/*.rs`）。`dev/tests/docs/Cargo.toml` の `description` を `"Compile checks for docs/: the English guides' Rust blocks as doctests, their no_run blocks type-checked"` にする。
  5. 日本語の 2 ファイルを英語に合わせる。
     - `docs/for-kivact-user/jp/how-to-guide/canvas.md`：`use kivact::draw::{Draw, Offscreen};` で始まる Rust の block の開始の fence ```` ```rust ```` を ```` ```rust,no_run ```` にする（英語の `canvas.md` の同じ block が `no_run`）。
     - `docs/for-kivact-user/jp/how-to-guide/windows.md`：`fn dont_save_exits_without_saving()` を `fn don_t_save_exits_without_saving()` にする。
  6. `cargo check -p kivact-docs --all-targets --all-features` を実行する。`checked` の中の error は、生成した module の名前（`<prefix>_block_<n>`）から guide のファイルと何番目の `no_run` の block かが分かる。block のコードが今の API で compile できないなら、英語と日本語の両方の guide のその block を同じに直す（rustdoc の `no_run` でも同じ error になるはずのもの）。直せない（API の不足）なら STOP。
- **消すもの**：日本語の木と `README_JP.md` の doctest、`dev/tests/docs` の `Role: ` の語。
- **状態の閉じ方**：`Output`（`String` 2 つ）、`Fence`（enum、variant の payload は `NoRun` の 1 つ）。build script は xtask の対象外。
- **試験**：残す：`dev/tests/docs/src/tests.rs::test_modules_in_guides_run_under_the_test_harness`。
- **確認（tier a）**：

  ```bash
  cargo fmt -p kivact-docs
  cargo check -p kivact-docs --all-targets --all-features
  cargo clippy -p kivact-docs --all-targets --all-features -- -D warnings
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --lib
  cargo xtask check --no-write --only docs-pairs,state --strict --crate kivact-docs
  dev/tools/window-lock.sh --timeout 1800 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --doc -- readme --test-threads=1
  ```

- **完了条件**：すべて成功。`cargo test -p kivact-docs --doc -- --list` に `jp` を含む名前が無い。

## T18：Python と client の記述を利用者の文書から消す

- **wave / lane / 前提**：W1b / T6 / T17
- **所有**：`docs/for-kivact-user/en/how-to-guide/devtools.md`、`docs/for-kivact-user/jp/how-to-guide/devtools.md`、`docs/for-kivact-user/en/how-to-guide/README.md`、`docs/for-kivact-user/jp/how-to-guide/README.md`、`README.md`、`README_JP.md`、`docs/for-kivact-developer/strategy.md`
- **目的**：TL-09（Python の節）、TL-08（client の crate の行）、Scripts パネルの削除（T23、親の決定）の利用者の文書の側。
- **手順**：
  1. 英語の `devtools.md`：
     - `The same protocol is available to MCP tools and to Python\nscripts.`（2 行）を `The same protocol is available to MCP tools.` にする。
     - `The Python client installs with pip.` の段落から `` `pip install -e python/kivact-devtools`. `` の行とその後の空行までを消す。
     - `` Every protocol method except `initialize`, `subscribe` and `show_table` is a `` を `` Every protocol method except `initialize` and `subscribe` is a `` にする（`show_table` は T23 が消した。行の折返しは変えない）。
     - `## Script DevTools with Python` の見出しから `## Limits` の直前までを消す（Python の client の例と Scripts パネルの段落。パネルは T23 が消した）。
  2. 日本語の `devtools.md`：
     - `同じ protocol は MCP の tool と Python の script からも使えます。` を `同じ protocol は MCP の tool からも使えます。` にする。
     - `Python の client は pip でインストールします。` の段落から `` Kivact のリポジトリー自体を編集するときだけ、`pip install -e python/kivact-devtools` を使ってください。 `` の行とその後の空行までを消す。
     - `` `initialize`・`subscribe`・`show_table` を除くすべての protocol の method が `` を `` `initialize`・`subscribe` を除くすべての protocol の method が `` にする。
     - `## Python で DevTools を操作する` の見出しから `## 制限` の直前までを消す。
  3. `docs/for-kivact-user/en/how-to-guide/README.md` の `review export, MCP and Python |` を `review export and MCP |` にする。日本語の README.md の `review の書き出し、MCP、Python |` を `review の書き出し、MCP |` にする。
  4. root の README：

     | ファイル | 変更前 | 変更後 |
     |---|---|---|
     | `README.md` | `\| Wire vocabulary shared by the DevTools agent, app, MCP server and Python client. \|` | `\| Wire vocabulary of the DevTools agent, app and MCP server, and the client connection (feature `client`). \|` |
     | `README.md` | `\| \`kivact-devtools-client\` \| ... \|` の行 | 消す |
     | `README.md` | `for the separate DevTools app, MCP server and Python client.` | `for the separate DevTools app and MCP server.` |
     | `README_JP.md` | `DevTools の agent・アプリ・MCP・Python client が共有する通信の語彙。` | ``DevTools の agent・アプリ・MCP が共有する通信の語彙と、client の接続（feature `client`）。`` |
     | `README_JP.md` | `\| \`kivact-devtools-client\` \| ... \|` の行 | 消す |
     | `README_JP.md` | `別アプリの DevTools、MCP server、Python client から使います。` | `別アプリの DevTools と MCP server から使います。` |
  5. `docs/for-kivact-developer/strategy.md` の `DevTools（別プロセスの agent と MCP・Python の client）` を `DevTools（別プロセスの agent と MCP の client）` にする。
- **消すもの**：利用者の文書の Python の client の記述、README の `kivact-devtools-client` の行。
- **状態の閉じ方**：該当なし。
- **試験**：なし。
- **確認（tier a）**：

  ```bash
  cargo xtask check --no-write --only docs-pairs --strict
  dev/tools/window-lock.sh --timeout 1800 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --doc -- readme for_kivact_user_en::how_to_guide::devtools --test-threads=1
  ```

- **完了条件**：すべて成功。`rg -n -i "python client|pip install|kivact-devtools-client|show_table|Scripts パネル|Scripts panel" README.md README_JP.md docs/for-kivact-user` が何も出さない。

## T19：`window-lock.sh --if-mirror` と `check-features.sh` の縮小

- **wave / lane / 前提**：W1b / T6 / T18
- **所有**：`dev/tools/window-lock.sh`、`dev/tools/check-features.sh`
- **目的**：TL-01（lock を window を開くコマンドだけに）、TL-17（feature の確認を公開の意味のある 11 件に）。
- **手順**：
  1. `dev/tools/window-lock.sh`：
     - 先頭の comment の `#   dev/tools/window-lock.sh -- cargo test -p kivact-public-api-tests --test e2e -- menu` を `#   dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- menu` にし、`# Usage:` の行を `# Usage: dev/tools/window-lock.sh [--timeout SECONDS] [--if-mirror] -- <command> [args...]` にし、その下に次の 3 行を足す。

     ```bash
     #   --if-mirror runs the command without the lock when KIVACT_TEST_MIRROR=off.
     #   Use it only for packages whose tests never force a mirror with
     #   RenderOptions::mirror(MirrorMode::Required) (see CONTRIBUTING.md).
     ```
     - `limit=900` から `shift`（`--` を外す行）までを次にする。

     ```bash
     limit=900
     if_mirror=0
     while [[ "${1:-}" == --timeout || "${1:-}" == --if-mirror ]]; do
       case "$1" in
         --timeout) limit="$2"; shift 2 ;;
         --if-mirror) if_mirror=1; shift ;;
       esac
     done
     if [[ "${1:-}" != -- ]]; then
       echo "usage: $0 [--timeout SECONDS] [--if-mirror] -- <command> [args...]" >&2
       exit 2
     fi
     shift

     run() {
       set +e
       timeout "$limit" "$@"
       local status=$?
       set -e
       if (( status == 124 )); then
         echo "window-lock: timed out after ${limit}s: $*" >&2
       fi
       return "$status"
     }

     if (( if_mirror )) && [[ "${KIVACT_TEST_MIRROR:-auto}" == off ]]; then
       run "$@" && exit 0 || exit $?
     fi
     ```
     - ファイルの末尾の `set +e` から `exit "$status"` までを `run "$@" && exit 0 || exit $?` の 1 行にする。
  2. `dev/tools/check-features.sh`：`entries=( ... )` を次にし、上の comment の `# Each entry is ...` の段落の前に `# Only features with a public meaning are checked (TL-17): the facade's, the GPU renderer, native embedding and the texture-sharing crate.` を足す。

     ```bash
     entries=(
       kivact/devtools
       kivact/embed
       kivact/gpu
       kivact/gpu-interop
       kivact/tokio
       kivact/test
       arvane-render/gpu
       arvane-native/embed
       kivact-gpu-share/export
       kivact-gpu-share/import
       kivact-gpu-share/wgpu
     )
     ```
- **消すもの**：`check-features.sh` の 13 件（`arvane-native/test`、`arvane-virtual/mirror`、`arvane-compose/{gpu,gpu-interop}`、`kivact-core/{embed,devtools,tokio,gpu-interop}`、`kivact-runtime/tokio`、`kivact-test/{devtools,embed,gpu-interop}`、`kivact-engine/testing`）。
- **状態の閉じ方**：該当なし。
- **試験**：なし。
- **確認（tier a）**：

  ```bash
  bash -n dev/tools/window-lock.sh
  KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --if-mirror --timeout 60 -- true
  dev/tools/window-lock.sh --timeout 60 -- true
  dev/tools/window-lock.sh --timeout 1 -- sleep 3; test $? -eq 124
  bash -n dev/tools/check-features.sh
  ```

- **完了条件**：すべて成功（3 つ目と 4 つ目で lock の directory が残らない：`ls "$(git rev-parse --git-common-dir)/kivact-window-tests.lock"` が失敗する）。

## T20：`architecture/` を git から外す

- **wave / lane / 前提**：W1b / T6 / T19
- **所有**：`architecture/`、`.gitignore`、`README.md`、`README_JP.md`
- **目的**：生成した図を commit しない（TL-02、設計 4.3）。図は `cargo xtask diagrams` で作る（T02）。
- **手順**：
  1. `git rm -r -q architecture`。
  2. `.gitignore`：
     - `python/kivact-devtools/build/` の行を消し、`# DevTools and Python artifacts` を `# DevTools artifacts and Python caches (dev/tools and example scripts)` にする。
     - `# Local reports: the KPI reports of dev/bench/run.sh and the state audit of` から `/dev/xtask/state-audit.md` までの 4 行を次にする。

     ```gitignore
     # Local reports: the KPI reports of dev/bench/run.sh.
     /dev/bench/results/

     # Generated architecture diagrams (`cargo xtask diagrams`).
     /architecture/
     ```
  3. `README.md`：`[generated graphs](architecture/) for the full graph.` を ``the graphs that `cargo xtask diagrams` generates in `architecture/` for the full graph.`` にする。`| \`xtask\` | \`dev/xtask\` | Workspace checks and generated architecture graphs. |` を ``| `xtask` | `dev/xtask` | Workspace checks; `cargo xtask diagrams` generates the architecture graphs. |`` にする。`| \`architecture/\` | Generated crate and module dependency graphs |` を ``| `architecture/` | Crate and module dependency graphs generated by `cargo xtask diagrams` (not committed) |`` にする。
  4. `README_JP.md`：`[生成した依存図](architecture/) を参照してください。` を ``` `cargo xtask diagrams` が `architecture/` に生成する依存図を参照してください。``` にする。`workspace 検査と architecture 図の生成。` を ``workspace 検査。`cargo xtask diagrams` が architecture 図を生成する。`` にする。`| \`architecture/\` | 生成した crate と module の依存関係の図 |` を ``| `architecture/` | `cargo xtask diagrams` が生成する crate と module の依存関係の図（commit しない） |`` にする。
- **消すもの**：git の `architecture/`（121 個の SVG と `index.html`）。
- **状態の閉じ方**：該当なし。
- **試験**：なし。
- **確認（tier a）**：

  ```bash
  git ls-files architecture | wc -l
  git check-ignore -q architecture/index.html
  cargo xtask check --no-write --only docs-pairs --strict
  ```

- **完了条件**：1 つ目が `0`、2 つ目が成功、3 つ目が `ok`。

## T21：`CONTRIBUTING.md`・`dev/README.md` を新しい gate と構成に直す

- **wave / lane / 前提**：W1b / T6 / T20
- **所有**：`CONTRIBUTING.md`、`dev/README.md`、`docs/for-kivact-developer/manual-verification.md`
- **目的**：gate の 3 段（設計 6.4、TL-01）、新しい xtask の検査と図の分離（TL-02、TL-04、TL-06）、E2E の binary（TL-15）、Python の削除（TL-09）を開発者の文書に書く。
- **手順**：
  1. `CONTRIBUTING.md`：
     - 「## コード」の箇条の「道具が確かめない規則」の 1 つ目を次にする：`- コードのコメントと rustdoc は英語で書きます。crate root と module（inline の \`mod {}\` を含む）の先頭に \`//! Role: <英語の 1 文>\` を置きます（xtask が図に表示します）。例（\`examples/\`）、雛形（\`templates/\`）、\`dev/tests/docs\` は利用者のコードとして書くので Role を置きません。`
     - 同じ箇条の最後の `- \`xtask\` は ... 図の生成には Node.js が要ります。` を `- \`xtask\` は \`vendor/xtask-core\` の submodule を使います。clone の後に \`git submodule update --init\` を実行します。図（\`architecture/\`、commit しない）は \`cargo xtask diagrams\` が書き、Node.js が要ります。\`cargo xtask check\` は図を書きません。` にする。
     - 「## ドキュメント」の `` `docs/` と README の Rust のコードブロックは ... コンパイルされます。`` の文を次にする：`` 英語の木（`docs/*/en/`）と `README.md` の Rust のコードブロックは `dev/tests/docs`（`kivact-docs`）がコンパイルします。`no_run` 以外は doctest として実行し、`no_run` は link しない型検査（`cargo check -p kivact-docs --all-targets`）にします。日本語の木と `README_JP.md` はコンパイルしません。代わりに `cargo xtask check` の `docs-pairs` が、日本語のファイルの Rust のコードブロックの列が英語と同じ（comment・空白・文字列の中身を除く）ことを確かめます。``
     - 「## テスト」の表を次にする。

     ```markdown
     | 場所 | 内容 |
     |---|---|
     | `dev/tests/kivact-public-api` | 公開 API の E2E：`e2e_core`（`kivact` だけ）、`e2e_ui`（ui・interaction・APG）、`e2e_native`（embed・GPU・foreign window・DevTools）。共有の helper は lib（`kivact_public_api_tests::{support, paint}`）。ウィンドウなしの画像書き出し（`tests/offscreen.rs`） |
     | `dev/tests/kivact-kit-tests` | 5 つの公式 kit が要る E2E（`e2e_kits`：theme の適合、kit の theme での標準部品） |
     | `crates/kivact-ui-*/tests` | 各キットの E2E |
     | `crates/arvane/tests` | Arvane の公開契約と描画・入力の E2E |
     | `crates/kivact-test` | App の共通起動経路を使う `kivact::test` ハーネス（自身の試験は `tests/`） |
     | `crates/arvane-virtual` | 仮想 backend、入力の台本、描画と任意の native mirror |
     | `dev/tests/docs` | 英語の guide と README のコードブロック |
     | `dev/tests/gpu-interop-producer` | GPU texture 共有の試験で使う描画エンジン |
     ```
     - GPU lane の文の `KIVACT_TEST_GPU=1 KIVACT_TEST_MIRROR=required dev/tools/window-lock.sh -- cargo test -p kivact-public-api-tests --test e2e --all-features -- --test-threads=1` を `dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_GPU=1 KIVACT_TEST_MIRROR=required cargo test -p kivact-public-api-tests --test e2e_core --test e2e_native --all-features -- --test-threads=1` にする。
     - `アプリを起動する試験と実 window を開くコマンドは、...` の段落を次にする。

     ```markdown
     `kivact::test` の試験は、`KIVACT_TEST_MIRROR=off` でも `RenderOptions::mirror(MirrorMode::Required)` を指定した試験が実 window を開きます。実 window の focus 競合と backend の同時利用を避けるため、次の package の試験は必ず `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test ...` の形で lock を通し、すべての worktree で同時に一つの process だけがウィンドウを開くようにします：`kivact-public-api-tests`、`kivact-kit-tests`、`kivact-test`、`kivact-docs`、`kivact-devtools`、5 つの kit、5 つの catalog、`catalog-shell`、`kivact-plot`、`kivact-example-*`、`kivact-template-basic`、`arvane`、`arvane-native`、`gpu-interop-producer`。それ以外の package の試験は window を開かないので lock は要りません。`window-lock.sh --if-mirror` は `KIVACT_TEST_MIRROR=off` のとき lock を省きますが、mirror を強制する試験の無い package にだけ使います。`gpu-interop-producer` と `kivact-docs` の doctest は `--test-threads=1` で直列にします（後者は多数の例の同時 compile/link によるメモリ不足を避けるため）。lock は時間の上限（既定 900 秒、`--timeout` で変更）も掛け、時間切れは終了コード 124 です。
     ```
     - 「## merge 前の検査」の節（見出しの後のコードブロックと、その後の 2 段落 `` `cargo xtask check` はファイルごとの ... `` と `` `cargo xtask devtools-codegen --check` は ... ``）を、見出しを「## 検査の 3 段」にして、4.1 節の「tier a」「tier b」「tier c」のコマンドの塊（W1a〜W4b・W11a 以降の形。W5a〜W10 の除外は書かない）と、次の段落に置き換える。

     ```markdown
     `cargo xtask check` はファイルごとの code 行の上限、crate の `visible_to`（dev-dependency は対象外）、module の依存の向き、Google Rust Style Guide の規則（rustfmt と Clippy を含む）と、局所の検査（`state`、`effect-writes`、`api-shape`、`canonical-path`、`name-collision`、`kit-literal-color`、`os-branch`、`docs-pairs`、`interaction-view`、`ui-machine`、`hand-element`、`unit-machine`、`element-shadow`）を行います。図は書きません（`cargo xtask diagrams`）。`--crate <crate>` を付けると局所の検査だけをその crate に行い（tier a）、`--only <検査,...>` で検査を選び（`--only local` はすべての局所の検査）、`--module <module path>` で範囲を絞ります。`--report` は違反を一覧にして成功で終了します。

     状態の規則（`state`）は、`bool`・`Option`・名前の付いた counter・`Cell`・`RefCell`・`Mutex`・`RwLock`・`Atomic*`・`Signal`・`Var` の field を 2 つ以上持ち、作った後に書かれる（`&mut self` の method、内部可変の field、自分の method が書く `Signal`/`Var`）型と、状態の local を 2 つ以上持つ関数を対象にします。組合せは enum で閉じ、閉じない組合せは型（関数）の直前の 1 行 `// xtask-allow(state): <組合せの閉じ方>` に書きます。constructor・`Default`・値渡しの builder だけで書く field は状態ではありません。

     改修の間だけ、`dev/xtask/policy/workspace.toml` の `[rework.pending]` に挙げた検査と package の組の結果は `pending:` の note になり、失敗になりません。`--strict` で error に戻ります。

     CI は fmt と `cargo xtask check --no-write` だけを実行します。`dev/tools/check-features.sh` は公開の意味を持つ feature を一つずつ有効にしてコンパイルします（tier c）。DevTools の JSON schema は protocol の型から `cargo test -p kivact-devtools-protocol --test schema_generated` が照合し、`KIVACT_WRITE_SCHEMA=1` を付けると書き直します。
     ```
  2. `dev/README.md` を次に置き換える。`<tier a>`・`<tier b>` の所には 4.1 節の対応するコマンドの塊（W1a〜W4b・W11a 以降の形）をそのまま入れる。

     ````markdown
     # dev

     開発用の検査とテストです。全体の手順は [CONTRIBUTING](../CONTRIBUTING.md)、
     境界と規則は [アーキテクチャ](../docs/for-kivact-developer/architecture.md) にあります。
     以下の shell コマンドはリポジトリのルートから実行します（Windows では Git Bash）。

     | 場所 | 用途 |
     |---|---|
     | `tests/kivact-public-api/` | 公開 API の E2E（`tests/e2e_core`、`tests/e2e_ui`、`tests/e2e_native`）、共有の helper（`src/`）、offscreen の描画契約 |
     | `tests/kivact-kit-tests/` | 5 つの公式 kit が要る E2E（`tests/e2e_kits`） |
     | `tests/docs/` | 英語の `docs/` と README の Rust コードブロック（doctest と `no_run` の型検査） |
     | `tests/gpu-interop-producer/` | GPU texture 共有の試験で使う独立した描画エンジン |
     | `../crates/kivact-test/` | 共通の App 起動経路を使う E2E ハーネス（公開入口は `kivact::test`） |
     | `../crates/arvane-virtual/` | 仮想時計・入力台本・共通描画 driver・任意の実 window mirror |
     | `tools/window-lock.sh` | 実 window を開くコマンドを worktree 間で一つずつ実行する lock（`--if-mirror`） |
     | `tools/check-features.sh` | 公開の意味を持つ feature を一つずつコンパイルする |
     | `tools/subset-catalog-font.py` | catalog の Noto Sans JP を使う文字の subset に作り直す（要 fontTools、原本は `--source`） |
     | `xtask/` | workspace の規則の検査（`cargo xtask check`）と依存図の生成（`cargo xtask diagrams`、要 Node.js） |
     | `bench/` | 性能の指標（K1〜K6）を手動で測る probe と `run.sh`（[README](bench/README.md)、CI では実行しない） |

     ## 検査の 3 段

     すべての cargo のコマンドに `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` を付けると、debug 情報を作らず build が速くなります（debug assertions は有効のまま）。

     tier a（変更ごと。数分。lock は E2E だけ）：

     ```text
     <tier a>
     ```

     tier b（まとまった変更の後）：

     ```text
     <tier b>
     ```

     tier c（merge の前、native の経路を変えたとき）は [CONTRIBUTING](../CONTRIBUTING.md) の「検査の 3 段」にあります。

     ## xtask の検査を絞る

     ```text
     cargo xtask check --no-write --crate kivact-ui-soft
     cargo xtask check --no-write --only state --strict --crate kivact-ui-soft --module navigation_session
     cargo xtask check --no-write --only effect-writes,state --crate kivact-ui-soft
     cargo xtask check --no-write --only local
     ```

     `--crate` は局所の検査だけを行います（xtask-core の行数・`visible_to`・module の向き・style は全体の `cargo xtask check` で行います）。`--strict` は `[rework.pending]` の note を error にします。

     ## E2E と doctest

     `kivact::test` の E2E は `arvane-virtual` の上で App 全体を動かします。
     `KIVACT_TEST_MIRROR=auto`（既定）は可能なら実 window に写し、`required` は実 window を必須にし、`off` は仮想 host だけにします。ただし試験が `RenderOptions::mirror(MirrorMode::Required)` を指定すると `off` でも実 window を開くので、E2E は lock を通します。

     ```text
     dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- menu
     dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-kit-tests --test e2e_kits
     dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft --test consumer
     dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p gpu-interop-producer --all-features -- --test-threads=1
     cargo check -p kivact-docs --all-targets
     dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --doc -- readme --test-threads=1
     ```

     `cargo test -p kivact-docs --doc -- readme` は root の `README.md` の例だけを、`-- for_kivact_ui_user_en::basic` は basic の guide だけを検査します。
     native 試験が並列実行中に異常終了した場合は、失敗した binary だけ `--test-threads=1` で再検証し、実行条件と症状を issue か PR の説明に記録します。

     ## feature の確認

     ```text
     cargo check -p kivact --no-default-features --lib
     cargo check -p kivact --no-default-features --features test --lib
     dev/tools/check-features.sh
     ```

     `kivact` の既定は `gpu` と `complex-scripts`（ICU の分かち書きの辞書）です。任意の `test` は `kivact-test`、`embed` は native 埋込み、
     `devtools` は DevTools の agent と protocol、`gpu-interop` は GPU texture 共有、`tokio` は非同期 task の連携を有効にします。
     `check-features.sh` は facade の feature と `arvane-render/gpu`・`arvane-native/embed`・`kivact-gpu-share` の feature を一つずつ確認します。
     ````
  3. `docs/for-kivact-developer/manual-verification.md` の link の `../../dev/tests/kivact-public-api/tests/e2e/` を次にする：`custom_text_field.rs`・`metrics_and_description.rs`・`parts.rs`・`viewport.rs` は `metrics_and_description.rs` だけ `tests/e2e_ui/`、他の 3 つは `tests/e2e_core/`。`embed/contract.rs`・`embed/panic_boundary.rs`・`viewport_gpu_interop.rs`・`foreign_window.rs` は `tests/e2e_native/`。
- **消すもの**：CONTRIBUTING の旧い「merge 前の検査」、`git diff --quiet -- architecture`、Python の試験の行と段落、`--report-audit` と `structural-options` と `devtools-codegen` の記述。
- **状態の閉じ方**：該当なし。
- **試験**：なし。
- **確認（tier a）**：

  ```bash
  rg -n "architecture/ の図と|git diff --quiet -- architecture|report-audit|structural-options|devtools-codegen|python/kivact-devtools|--test e2e\b|tests/e2e/" CONTRIBUTING.md dev/README.md docs/for-kivact-developer/manual-verification.md
  ```

- **完了条件**：上のコマンドが何も出さない。`CONTRIBUTING.md` と `dev/README.md` の tier のコマンドが 4.1 節と一字一句同じ（W5a〜W10 の除外の注を除く）。

---

## T22：W1 の締め（`state` を pending から外す、`redundant_imports` を workspace の lint に）

- **wave / lane / 前提**：W1c / T0 / T05〜T21 のすべての merge
- **所有**：`dev/xtask/policy/workspace.toml`、root の `Cargo.toml` の `[workspace.lints.rust]`、`redundant_imports` の警告が出たすべてのファイル（workspace 全体。W1c は単独の lane）
- **目的**：`state` を全 workspace で error にする（AC-3 の準備、TL-04）。例と雛形にあった `redundant_imports` を workspace の lint に移す（設計 4.3）。
- **手順**：
  1. `dev/xtask/policy/workspace.toml` の `[rework.pending]` から `state = [ ... ]`（閉じ括弧まで）と `docs-pairs = ["kivact-docs"]` の行を消す。残るのは T03 の 5 行。
  2. root の `Cargo.toml` の `[workspace.lints.rust]` に `redundant_imports = "warn"` を足す。
  3. `cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | grep -A6 "redundant"` が出すすべての箇所で、その `use`（または `use` の列のその名前）を消す。出なくなるまで繰り返す。
  4. `cargo xtask check --no-write` を実行する。`state` の error が出たら、その package の W1b の unit の lane の fix として共通手順 M を行う（作業指示書 1.3 節の fix）。
- **消すもの**：`[rework.pending]` の `state`・`docs-pairs`。
- **状態の閉じ方**：該当なし。
- **試験**：なし。
- **確認（tier a）**：

  ```bash
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  cargo xtask check --no-write
  rg -n "xtask-allow\(state-fields\)|#!\[warn\(redundant_imports\)\]" crates dev examples templates
  ```

- **完了条件**：1 つ目・2 つ目が成功（2 つ目は 6.3 の 5 つの検査の `pending:` の note だけを出し `ok`）。3 つ目が何も出さない。

---

## 4. 他の付属書との接点

### 4.1 gate の 3 段（作業指示書が引用する）

**すべての cargo のコマンドに次の環境変数を付ける**（debug assertions は有効のまま）：

```bash
CARGO_TARGET_DIR=<作業指示書 1.1 節の target> CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
```

**window-lock の規則**（作業指示書 0.6 節の「`KIVACT_TEST_MIRROR=off` のコマンドは W1 の後は lock なしでよい」を置き換える。4.6 節の 5、親が採用）：`kivact::test` の E2E は `KIVACT_TEST_MIRROR=off` でも、`RenderOptions::mirror(MirrorMode::Required)` を指定した試験（基点で 19 module）が実 window を開く。そこで次の package の `cargo test` は**必ず** `dev/tools/window-lock.sh` を通す（**lock の一覧**）：`kivact-public-api-tests`、`kivact-kit-tests`、`kivact-test`、`kivact-docs`、`kivact-devtools`、5 つの kit、5 つの catalog、`catalog-shell`、`kivact-plot`、`kivact-example-*`、`kivact-template-basic`、`arvane`、`arvane-native`、`gpu-interop-producer`。それ以外の package の `cargo test` は lock なしでよい（`--if-mirror` も不要）。

**書き方**（親の決定、INC-15・INC-43）：lock の一覧の package の試験のコマンドは、どの付属書でも次の形にする。環境変数は `env` で lock の内側に渡す。

```bash
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p <crate> <filter>
```

gate の workspace の試験（tier b・tier c）だけは `--timeout 5400` を使う。

**除外する package**（DEC-14 と作業指示書の枠）。**compile の対象外にする仕組みは gate の `--exclude` だけ**である（INC-09）。root の `Cargo.toml` の `members`・`exclude` は W1 の後どの wave でも変えない（kit・catalog・`kivact-kit-tests`・`kivact-docs` は workspace の member のまま。対象外の期間が終わったら gate が `--exclude` を付けなくなるだけ）。付属書の手順が「`members` から消す／戻す」と書いていたら何もしない。

- `XD`（W5a〜W10 の tier b、CP2）：`--exclude kivact-docs`
- `XK`（W6a〜W9 の tier b、CP2）：`--exclude kivact-ui-basic --exclude kivact-ui-pop --exclude kivact-ui-game --exclude kivact-ui-creative --exclude kivact-ui-soft --exclude kivact-ui-basic-catalog --exclude kivact-ui-pop-catalog --exclude kivact-ui-game-catalog --exclude kivact-ui-creative-catalog --exclude kivact-ui-soft-catalog --exclude kivact-kit-tests`
- `XT`（W10 の tier b）：`--exclude kivact-kit-tests`（W10 の tier b は kit と catalog を含め、`kivact-kit-tests` だけを外す。`kivact-kit-tests` を新しい API に直す K44 は W11a。INC-12）
- xtask：W1a〜W4b と W11a 以降は `cargo xtask check --no-write`（全体）。**W5a〜W10 は `cargo xtask check --no-write --only local`**（xtask-core の style の Clippy は `--workspace` を compile するので、compile の対象外の `kivact-docs`・kit があると必ず失敗する。4.6 節の 6、親が採用）。

#### tier a（unit ごと。付属書の「確認（tier a）」の雛形）

```bash
cargo fmt -p <crate>
cargo check -p <crate> --all-targets
cargo clippy -p <crate> --all-targets -- -D warnings
cargo xtask check --no-write --crate <crate>                      # 局所の検査だけ。pending を閉じる unit は --strict を付ける
cargo test -p <crate> <filter>                                     # lock の一覧に無い package
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p <crate> <filter>      # lock の一覧の package
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- <module>
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- <module>
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native --all-features -- <module>
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-kit-tests --test e2e_kits -- <module>
```

`e2e_kits` は `kivact-kit-tests` の binary（`kivact-public-api-tests` ではない）。作業指示書 0.6 節の `--test <e2e_core|e2e_ui|e2e_kits|e2e_native>` の行は、`e2e_kits` だけ `-p kivact-kit-tests` に直す。

#### tier b（wave の merge の後。W1a〜W4b・W11a 以降の形）

```bash
cargo fmt --all -- --check
cargo xtask check --no-write
cargo clippy --workspace --all-targets --all-features -- -D warnings
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs
cargo check -p kivact-docs --all-targets --all-features
rg -n "xtask-allow\(state-fields\)" crates dev examples templates     # W1c の後：何も出ない
```

W5a・W5b：xtask を `--only local` にし、clippy と test に `XD` を足し、`cargo check -p kivact-docs` を行わない。
W6a〜W9：W5a に加えて clippy と test に `XK` を足す。
W10：W5a に加えて clippy と test に `XT` を足す（kit と catalog は含める）。
W11a 以降：上の形のまま（何も除外しない。`kivact-docs` は G01、`kivact-kit-tests` は K44 が W11a で新しい API に直す）。

#### tier c（checkpoint：CP1・CP2・CP3。CP1 と CP3 の形）

```bash
cargo fmt --all -- --check
cargo xtask check --no-write
cargo clippy --workspace --all-targets --all-features -- -D warnings
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features
dev/tools/window-lock.sh --timeout 5400 -- cargo test -p gpu-interop-producer --all-features -- --test-threads=1
cargo check -p kivact-docs --all-targets --all-features
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --doc --all-features -- --test-threads=1
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --no-default-features
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_GPU=1 KIVACT_TEST_MIRROR=required cargo test -p kivact-public-api-tests --test e2e_core --test e2e_native --all-features -- --test-threads=1
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_GPU=1 KIVACT_TEST_MIRROR=required cargo test -p kivact-test --test gpu -- --test-threads=1
dev/tools/check-features.sh
```

CP2（W8 の後）：xtask を `--only local` にし、clippy と 2 つの workspace の test に `XD` と `XK` を足し、`kivact-docs` の 2 行を行わない。GPU lane の 2 行と `check-features.sh` はそのまま（kit を含まない）。加えて、pending の残りを記録するために次の 2 つを実行し、結果（error の行の数と検査の名前）を `rework/PROGRESS.md` に書く。**失敗しても先に進む**（W5a〜W10 の tier b は `--only local` で pending を note にするので、ここで残りを知る。INC-29）。`--crate` は 1 回しか書けないので 2 つに分ける。

```bash
cargo xtask check --no-write --only local --strict --crate kivact-ui
cargo xtask check --no-write --only local --strict --crate kivact-interaction
```

CP1（W4b の後）と CP3（最終、W11c の後）は上の形のまま。CP3 は加えて作業指示書 2 章の AC-4 の検査（4.2 節が基）を行う。CP3 の `cargo xtask check --no-write` は、G15 が `[rework.pending]` と `--strict` を消した後なので、すべての検査の結果が error になる（AC-3）。

`gpu-interop-producer` の `--no-default-features` と `kivact-docs` の `--no-default-features` は行わない（producer は既定の feature なしでは振る舞いを持たず、docs は feature が無いと compile できない。TL-01）。GPU lane は GPU の無い環境では `kivact::test::gpu_unavailable()` の理由を出して skip する。

### 4.2 AC-4 の検査（設計 5.12 の削除名）

**AC-4 のコマンドの正本は作業指示書 2 章（GATE-TIERS）の一つだけ**である（親の決定、INC-13）。作業指示書のコマンドはこの節を基にし、出力の扱い（付属書 G の G15 の手順 4 の (i)〜(v) の分類の規則）を加えたもの。この節と食い違ったら作業指示書が正しい。付属書の unit は作業指示書のコマンドを実行し、自分の写しを持たない。

最終 checkpoint（CP3）で次を実行し、**どれも何も出さない**ことを確かめる。除外：旧名を書く文書（`migrating-to-rework.md`、G14 が消した名前を記録する `decisions.md`）、改修の文書（`REWORK_*.md`、`rework/**`）、`target/`。

```bash
EXCL=(--glob '!docs/for-kivact-user/*/how-to-guide/migrating-to-rework.md' --glob '!docs/for-kivact-developer/decisions.md' --glob '!REWORK_*.md' --glob '!rework/**' --glob '!target/**')
rg -n -w "${EXCL[@]}" --glob '!dev/xtask/src/public_surface.rs' \
  -e Binding -e IntoBinding -e IntoReactive -e Reactive -e Notifier -e notifier \
  -e ActionOutcome -e ActionMode -e Retry -e AsyncSnapshot \
  -e ThemeColor -e ColorToken -e ColorName -e Palette -e Spacing -e Radii -e UiRecipes -e UiTokens -e use_tokens -e ThemeHint \
  -e TextLen -e GridFlow -e Insets -e TextExtras -e DecorationStyleValue -e prefer_gpu \
  -e use_menu_with_open -e on_open_change -e on_step_press -e on_draft_commit -e selection_field -e input_group_with -e while_open \
  -e BadgeHue -e ItemHue -e StatHue -e ProgressHue -e WeekendHue -e CodeTone -e CodeColors \
  -e GameInputGroupVariant -e GameCellVariant -e PopBubble \
  crates examples templates docs dev
rg -n -P "${EXCL[@]}" '(?<!taffy::)\bDimension\b' crates examples templates docs dev
rg -n -w "${EXCL[@]}" --glob '!crates/kivact-devtools-protocol/**' --glob '!crates/kivact-devtools/**' --glob '!crates/kivact-core/src/devtools/**' TokenKind crates examples templates docs dev
rg -n -w "${EXCL[@]}" ButtonTone crates/kivact-ui-game examples/kivact-ui-game docs/for-kivact-ui-user
rg -n -w "${EXCL[@]}" SpanStyle crates/kivact-style crates/kivact-core crates/kivact crates/kivact-ui crates/kivact-ui-basic crates/kivact-ui-pop crates/kivact-ui-game crates/kivact-ui-creative crates/kivact-ui-soft examples templates docs
rg -n "${EXCL[@]}" -e 'kivact::launch' -e 'launch::AppOptions' -e 'interaction::ViewTransform' -e 'DialogMode::Popover' crates examples templates docs dev
rg -n -P "${EXCL[@]}" '\b(toaster|announcer)\(' crates examples templates docs dev
rg -n -F "${EXCL[@]}" '.composite(' crates examples templates docs dev
rg -n -P 'pub fn (weight|italic|radius\w*|ring\w*|basis|grow|shrink|wrap|direction|pressed)\b' crates/kivact-style/src
```

加えて `rg -n -w constant_value crates examples templates docs dev` の各行が、付属書 R の 4 章が許す場所の一覧にあること。旧 style setter（`.weight(` など）は名前が style 以外の API（`.direction(FormDirection::..)` など）と重なるので、上の最後の行（`Style` に定義が無い）と tier c の compile（例・雛形・英語の docs を含む）で確かめる。

`Binding` の検査は `dev/xtask/src/public_surface.rs` の xtask 内部の型 `Binding` を除く。`TokenKind` は DevTools の wire の型（`kivact-devtools-protocol::TokenKind`）を除く。protocol の `TokenKind` は残す（親の決定。設計 5.12 の `TokenKind` は soft のもので、付属書 K の K41 が消す。INC-38）。`ButtonTone` は game の path だけを見る（pop の `ButtonTone` は残す。INC-39）。`Dimension` は taffy の型を除く。

### 4.3 xtask が他の付属書に提供する名前と書式

| 名前 | 意味 |
|---|---|
| `// xtask-allow(state): <1 行>` | 型（struct・enum）か関数の直前（doc と属性より上）に一つ。新しい状態の型を作る unit は、付属書がこの文面を書く（設計 9 章の規則 4） |
| `cargo xtask check --no-write --crate <crate>` | 局所の検査だけ（速い。tier a） |
| `--strict` | `[rework.pending]` の note を error にする。6.3 の検査を通す unit が使う |
| `--only local` | すべての局所の検査 |
| `cargo xtask diagrams` | 図を `architecture/` に書く（要 Node.js、commit しない） |
| `[rework.pending]` | 4.4 節 |
| `[ui_machine] allowed_types` | kivact-ui に残す表示だけの Machine の定義 path（`kivact_ui::<module>::<Type>`）。`button.rs`・`progress.rs`・`alert.rs`・`empty_state_changes.rs` の Machine を別の module に移すのは付属書 U。path の更新は W8 の U4 の **U52** にまとめる（W7b・W8 で二つの lane が `workspace.toml` を持たないため。INC-14） |
| `[state] exempt_suffixes`、`recipe_state_packages` | 設計 6.1 (c)。**付属書 K** の `kivact-plot` の新設の unit は `recipe_state_packages` に `"kivact-plot"` を足す（`<Name>State` の recipe の状態を持つなら） |
| `check_source::INTERNAL_CONTRACT_PACKAGES` | `#[doc(hidden)]` が印なしで内部の契約になる package。`arvane-protocol` を先に入れてある（**付属書 S** が `kivact-types/src/render/shaped.rs` などを移すため）。`kivact-offscreen` などに `#[doc(hidden)]` の契約を置く付属書は、この const に名前を足す |
| `visible_to` | 新しい crate（`arvane-protocol`、`kivact-offscreen`、`kivact-plot`）の `visible_to` の行は、その crate を作る unit（付属書 S・C・K）が足す。xtask-core は存在しない package の名前を拒むので、W1 では足さない。dev-dependency は対象外なので、試験の package を一覧に入れない |

### 4.4 `[rework.pending]`：どの unit が通し、誰が消すか

| 検査 | pending の package（T03 の時点） | 通す付属書（その unit は `--strict --only <検査> --crate <package>` で確かめる） |
|---|---|---|
| `state` | xtask 以外の全 package | 本付属書 T05〜T17。**T22（W1c）が行を消す** |
| `docs-pairs` | `kivact-docs` | 本付属書 T17。**T22 が行を消す** |
| `interaction-view` | `kivact-interaction` | 付属書 U（UI-04・UI-24：`Announcer::outlet`、`while_open`、`Toast::icon` を消し `surface()` だけにする）。W8 の各 U の lane の最後の unit（U54 など）が `--strict --only interaction-view,ui-machine,unit-machine --crate kivact-interaction` で確かめる |
| `ui-machine` | `kivact-ui` | 付属書 U（UI-04：入力で動く 13 の Machine を interaction の `use_*` へ。`combobox/labels.rs` などの signal を書く effect をなくす）。W8 の各 U の lane の最後の unit が `--strict --only interaction-view,ui-machine,unit-machine --crate kivact-ui` で確かめる。`[ui_machine] allowed_types` の path は U52 が直す |
| `unit-machine` | `kivact-interaction` | 付属書 U（UI-09：`scalar/focus_machine.rs`、`scalar/range_machine.rs`）。確かめ方は `interaction-view` と同じ |
| `hand-element` | `kivact-core`、`kivact-ui`、5 つの kit、`kivact-ui-game-catalog`、`kivact-ui-pop-catalog` | 付属書 C（C09 か C18：`kivact-core/src/custom/measured_leaf.rs` の `Primitive` を `view` に移し `--strict --only hand-element --crate kivact-core`）、付属書 U（U24：`--strict --only element-shadow,hand-element --crate kivact-ui`）、付属書 K（各 kit の W10 の最後の unit K35・K37・K38・K40・K43：`--strict --only hand-element,element-shadow --crate kivact-ui-<kit>`。game と pop は catalog の package も） |
| `element-shadow` | `kivact-ui`、5 つの kit、`kivact-ui-game-catalog`、`kivact-ui-pop-catalog` | 付属書 U（U24。API-13 の `Card::disabled` など）、付属書 K（`hand-element` と同じ unit） |

W5a〜W10 の tier b は `--only local` で pending を note にするので、残りは CP2 の記録（4.1 節）と、上の各 unit の `--strict` の確認で知る（INC-29）。

**G15（W11c、付属書 G の最後の unit）が行う pending の削除の手順**（INC-14。G15 の所有は `dev/xtask/policy/workspace.toml`、`dev/xtask/src/{pending.rs,local_checks.rs,check_cli.rs,main.rs,policy.rs}`、`CONTRIBUTING.md`、`dev/README.md` の該当の部分）：

1. 前の確認：`cargo xtask check --no-write --only local --strict` を実行し、`ok` であること。失敗したら、error の行の検査と package を書いて STOP（上の表の通す unit の残り。G15 は直さない）。
2. `dev/xtask/policy/workspace.toml`：`# 改修（REWORK_DESIGN.md）の間だけ：` で始まる comment の 2 行と、`[rework.pending]` の見出しとその下の行（T03 の 5 行）を消す。
3. `git rm -q dev/xtask/src/pending.rs`。`dev/xtask/src/main.rs` の `mod pending;` を消す。
4. `dev/xtask/src/local_checks.rs`：
   - `use crate::pending::Pending;` と `let pending = Pending::parse(&policy)?;` を消す。
   - `struct Scope` の field `pending: &'a Pending,` と、`Scope { .. }` の初期化の `pending: &pending,` を消す。
   - `Scope::run` の中の `self.route(rule, package, found, options, outcome);` と `self.route(rule, &source.package, found, options, outcome);` の 2 か所を `outcome.errors.extend(found);` にし、`fn route`（doc を含む）を消す。`run` の引数 `options` が使われなくなる場合は compile の警告に従って消し、呼出し `scope.run(root, options, &mut outcome)` も合わせる。
   - `struct Outcome` の field `notes`（doc の文の「notes are findings of a pending rule」も）を消し、`Outcome { errors: Vec::new(), notes: Vec::new() }` を `Outcome { errors: Vec::new() }` にする。`for list in [&mut outcome.errors, &mut outcome.notes] { list.sort(); list.dedup(); }` を `outcome.errors.sort();` と `outcome.errors.dedup();` の 2 行にする。
   - `fn selected` の `for note in &local.notes { println!("{note}"); }` を消す。
5. `dev/xtask/src/check_cli.rs`：`Options` の field `strict`（doc を含む 2 行）と、`parse` の `"--strict" => result.strict = true,` を消す。`mod tests` の引数の列から `"--strict",` を、`assert!(options.strict);` を消す。
6. `dev/xtask/src/main.rs`：`report.notes.extend(local.notes);` を消す。help の文字列の `[--strict] ` を消す。
7. `dev/xtask/src/policy.rs`：`table.remove("rework");` を消す。
8. `CONTRIBUTING.md`：`改修の間だけ、` で始まる段落（`[rework.pending]` と `--strict` の説明）を消す。`dev/README.md`：「xtask の検査を絞る」のコードブロックの `--strict ` を消し、その下の段落の `` `--strict` は `[rework.pending]` の note を error にします。`` の文を消す。
9. 確認：

   ```bash
   cargo fmt -p xtask
   cargo clippy -p xtask --all-targets -- -D warnings
   cargo test -p xtask
   cargo xtask check --no-write
   rg -n "Pending|rework\.pending|--strict|\.strict\b|\bnotes: Vec|local\.notes" dev/xtask CONTRIBUTING.md dev/README.md
   ```

   最後の `rg` が何も出さない（`xtask-core` の `report.notes` は残る：`main.rs` の `report.notes.extend(local.errors)`（`--report`）はそのまま）。

### 4.5 E2E の構成（全付属書が試験の名前を書くときに使う）

| binary | package | 中身 | dev-dependency |
|---|---|---|---|
| `e2e_core` | `kivact-public-api-tests` | runtime・view・style・layout・text・input・a11y・task・flows・offscreen など 82 module | `kivact`（lib の依存）、`async-channel`、`image` |
| `e2e_ui` | `kivact-public-api-tests` | `apg_*`（19）・`interaction_*`（21）・`ui_*`（16）・`feedback`・`indicators`・`metrics_and_description`・`reactive_props` | 同上 |
| `e2e_native` | `kivact-public-api-tests` | `embed`（feature `embed`）・`devtools`（feature `devtools`）・`gpu_interop`・`gpu_on_demand*`・`drawable_export_gpu`・`viewport_gpu_interop`（feature `gpu-interop`）・`foreign_window` | `kivact-devtools-protocol`（`client`）、`gpu-interop-producer`、`kivact-gpu-share`、`wgpu`、`windows`、`windows-sys` |
| `e2e_kits` | `kivact-kit-tests`（新設） | `theme_conformance`、`ui_standard`、`apg_grid_kit`、`presence_kit` | 5 つの kit、`kivact-public-api-tests`（helper） |

共有の helper：`kivact_public_api_tests::support::{mount, mount_with, random, color_reference}`、`kivact_public_api_tests::paint::{RED, BLUE, GREEN, WHITE, BLACK, canvas_of, at, pixel, distance, expect}`。新しい E2E の module は、その binary の `main.rs` に `mod` 行を足す（作業指示書 1.3 節の conflict の規則が `dev/tests/kivact-public-api/tests/<binary>/main.rs` を扱う）。`kivact-kit-tests` の `main.rs` も同じ規則で扱う。W5a 以降の API の掃引の鎖は `e2e_core`・`e2e_ui`・`e2e_native` を直し、W6a〜W10 は `kivact-kit-tests` を compile しない（K44 が W11a で直す）。

### 4.6 設計判断の結果（親が決定済み）

1 と 2 と 4〜7 は推奨どおり採用（設計 4.1・6.1・6.4 を改訂済み、作業指示書 0.6 節は親が直す）。3 は推奨を採らず、Scripts パネルとそれだけが使う protocol を消す（T23）。

1. **`State` の接尾辞の対象外**（採用）：設計 6.1 (c) は `State`（recipe の状態）を対象外とする。すべての `*State` を対象外にすると engine の `PointerState` などの本物の状態の型が検査から抜ける。**決定**：`State` は `recipe_state_packages`（`kivact-ui` と 5 kit。後で `kivact-plot`）の中でだけ対象外（T01 の実装）。
2. **`e2e_kits` を別の package にする**：Cargo の dev-dependency は package のすべての test target に compile されるので、`kivact-public-api-tests` の中に `e2e_kits` を置くと、W6a〜W9 に kit を compile から外せず `e2e_core` なども compile できなくなる（DEC-14）。feature で kit を optional にする方法は tier c の `--all-features` と衝突する。**決定（採用）**：新しい package `kivact-kit-tests`（`dev/tests/kivact-kit-tests`、`publish = false`）に `e2e_kits` を置く（T15）。package の数は設計 4.1 の 53 から 54 になる。
3. **DevTools の Scripts パネル**（親の決定：消す）：Scripts パネル、method `show_table`、通知 `script_table`、`ShowTableParams`、`Topic::Scripts`、`DevtoolsConfig::{python, scripts_dir}`、`ClientKind::Python` を消し、`PROTOCOL_VERSION` を 1 から 2 にし、MCP の除外の列から `show_table` を消す（T23、W1a）。`rg` で確かめた範囲では、これらを使うのは Scripts パネル・core の `devtools/scripts.rs`・MCP の除外の列・試験だけで、残す必要のあるものは無い（`push_capped` は他の列も使うので残す）。利用者の文書の側は T18。
4. **TL-10（core の DevTools の hook）**：設計 7.8 で採用だが kivact-core の内部の再構成（`client/dispatch/*` と `view/devtools_*.rs`）。**決定（採用）**：付属書 C が W3a〜W4b の C の lane の unit として持つ（`devtools` module が `Option<Box<dyn DevtoolsHooks>>` を一つ公開し、dispatch は `hooks.on_pointer(..)` などだけを呼ぶ。`view/devtools_*` を `devtools/` へ畳む）。
5. **window-lock**：TL-01 は「`KIVACT_TEST_MIRROR=off` なら lock が要らない」としたが、基点で 19 の E2E module が `RenderOptions::mirror(MirrorMode::Required)` で実 window を開く。**決定（採用）**：4.1 節の lock の一覧の package の `cargo test` は常に lock を通す。`--if-mirror` は実装する（T19）が gate では使わない。作業指示書 0.6 節の該当の行を 4.1 節の規則に置き換える。
6. **W5〜W10 の xtask**：xtask-core の全体の検査は style の Clippy で `--workspace --all-targets --all-features` を compile する。W5a から `kivact-docs`（古い API の `no_run` の型検査）、W6a から kit が compile できないので必ず失敗する。xtask-core（vendor の submodule）に package の除外は無い。**決定（採用）**：W5a〜W10 の tier b と CP2 は `cargo xtask check --no-write --only local` にする（行数の上限・`visible_to`・module の向き・style は CP1 と CP3 で確かめる）。
7. **AC-4 の旧 style setter**：`.direction(` などは style 以外の API にもあり、`rg` の字面では判定できない。**決定（採用）**：4.2 節のとおり、`kivact-style` に定義が無いこと（`rg`）と、tier c の compile（例・雛形・英語の docs を含む）で確かめる。

### 4.7 他の付属書が本付属書に依存すること

- すべての付属書の unit の「確認（tier a）」は 4.1 節の tier a の形を使い、E2E は `e2e_core`・`e2e_ui`・`e2e_native`（`-p kivact-public-api-tests`）と `e2e_kits`（`-p kivact-kit-tests`）の名前で書く。
- `crates/kivact-core/src/custom.rs` の E2E の path の doc は T07 が `tests/e2e_core/primitive.rs` に直す（T15 が実際に移す）。
- `docs/for-kivact-developer/architecture.md`（`state-fields` の S 規則の段落、`cargo xtask devtools-codegen`、`kivact-devtools-client` の行、`tests/e2e/` の path、Python）と `decisions.md`（ADR-51 の Python）は付属書 G が W11a（G13・G14）で書き直す。本付属書は触らない。
- `.github/workflows/ci.yml` は変えない（`cargo xtask check --no-write` はそのまま動く。設計 9 章の規則 2）。

### 4.8 付属書 K への引き渡し：TL-13（kit の consumer 試験の統合）

TL-13 は採用（設計 7.8）。kit の移行（W9・W10）の後に行う。**付属書 K** が、各 kit の移行の unit の中で、または K6 の最後の unit として、次を行う。

- 各 kit の `tests/consumer` を約 600 行の 1 ファイルにし、「この kit が behavior X を配線している」ことを control の族ごとに一つの試験で確かめる。
- `kivact-interaction` の behavior そのもの（`*_keyboard`、`*_touch`、`*_focus`、`selector_*`、`toast_swipe` など）を確かめる kit の試験は、`e2e_ui` の `interaction_*`・`apg_*` が同じ behavior を確かめていれば消す。共有の確認（selector の keyboard・touch・focus、disclosure、toast）は `kivact-kit-tests` の `theme_conformance` の kit の parametrised な試験に移す。
- kit 固有の機能（creative の node editor・dock・color wheel・arrangement・numeric・bezier、soft の chat・grouped table・tree reveal）の試験は残す。時間の見た目を比べる `*_motion` の試験は、設計 6.4 の「時間の厳密な比較を足さない」に従い、契約（終わった後の状態）を確かめる形に直すか消す。
- 消す試験は名前を列挙する（設計 9 章の規則 5）。

### 4.9 付属書 C への引き渡し：TL-10

4.6 節の 4 のとおり。付属書 C が採らない場合は、親が付属書 T の追補として W3a〜W4b の unit を書く。

## 5. 自己点検

- **所見の割当て**：TL-01（T19、T21、4.1）、TL-02（T02、T20、T21）、TL-03（T04、T17）、TL-04（T01、T05〜T17、T22）、TL-05（T02、T07〜T09、T16）、TL-06（T02、T07）、TL-07（T02）、TL-08（T13、T18）、TL-09（T04、T18、T23）、TL-10（4.9 で付属書 C へ。4.6 節の 4）、TL-11（不採用）、TL-12（T16）、TL-13（4.8 で付属書 K へ）、TL-14（T13、T14）、TL-15（T15）、TL-16（T01、T20、T21）、TL-17（T19）。設計 6.1（T01、T22）、6.3（T02、T03、4.4）、6.4（T15、T17、4.1）、4.1（T13、T04）、4.3（T04、T07〜T12、T14、T20、T22）、KIT-16 の README の部分（T14）。採用の所見はすべてどれかの unit か引き渡しにある。
- **設計の判断を残していないか**：共通手順 M の印の文面は報告の行と旧い印の理由から機械的に決まる。分割の module の行き先は T15 の 3 つの列で全部決めてある（列に無いファイルが残れば STOP）。設計の判断が要った 7 点は親が決定し（4.6 節）、本文はその決定で書いてある。T23 の削除の範囲は `rg` の結果の表で閉じており、表の外の利用が見つかれば STOP。
- **横断の検査（INCONSISTENCIES.md）への対応**：INC-09（除外は `--exclude` だけ、4.1）、INC-12（W10 は `XT`、4.1）、INC-13（AC-4 の正本は作業指示書 2 章、`EXCL` に `decisions.md`・`REWORK_*.md`・`rework/**`、4.2）、INC-14（4.4 の通す unit と G15 の削除の手順）、INC-15・INC-43（lock の一覧に `catalog-shell`・`kivact-plot`、全コマンドを `window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test` の形に）、INC-29（CP2 の `--strict` の記録、4.1）、INC-38（protocol の `TokenKind` は残す、4.2）、INC-45（wave の表記を SCHEDULE.md の sub-wave に。本付属書の unit の配置は SCHEDULE.md と同じ）。
- **W1a と W1b のファイルの重なり**：T23（W1a）は `crates/kivact-core/src/devtools*`・DevTools の 5 crate を変えるが、W1a は 1 lane で W1b より前に終わる。W1b で同じファイルを持つ T07（kivact-core）と T13（DevTools の crate）は T23 の結果の上で作業する。
- **同じ wave の二つの unit が同じファイルを持たないか**：W1a は 1 lane（T01 → T02 → T03 → T04 → T23 の順）。W1b の 6 lane は 2 章の末尾のとおり directory で分かれる。root の `Cargo.toml`（`members`）と `Cargo.lock` は T5 の lane の T13 と T15 だけ（同じ lane の順）。`dev/xtask/policy/workspace.toml` は W1b では T15 だけ（1 件の例外を足す）。`README.md`・`README_JP.md` は T6 の lane の T18 と T20 だけ（同じ lane の順）。`crates/kivact-ui-*/README.md` は T14（T5）、`crates/kivact-ui-*/src` は T11（T4）。`crates/kivact-core/src/custom.rs` の doc は T07（T2）が直し、T15 は触らない。`crates/kivact-types/src/cell.rs` は付属書 S の W1a の unit の所有で、T09 は除く。W1c は 1 lane。
- **状態の閉じ方**：新しい型の field は 1 章ごとに書いた。`check_cli::Options` は `Options` の接尾辞で対象外、`Fence` は enum。新しい `xtask-allow(state)` の印は本付属書の新しいコードには要らない。
- **試験**：足す試験・直す試験・消す試験・移す試験を名前で挙げた（T01〜T04、T13、T14、T16、T17）。期待値を緩めた試験は無い。`GRACE` の置換えは assert を変えない（仮想時計で同じ時間を与える）。過去の不具合の札は、試験の無い札だけを消した（試験は 026e7d59 で既に消えている）。
