# 付属書 X：全体の xtask の検査を通す（W11x、2026-11-22）

基点 `c9518946` では `cargo xtask check --no-write`（全体）が通っていた。W5a〜W10 の gate は `--only local` だったので、その間に全体の検査の error が 1160 件たまった（統合 `fab95ac1`。log は `V:/kv-target/logs-W11a/xtask-full.log`、基点は `xtask-full-base.log`）。CP3（5.2 節）は全体の検査が通ることを求めるので、この付属書の unit で 0 件にする。

- 文書の Rust の block（約 316 件）と `constant_value` の 2 件は、W11b（G02〜G11）で消える。この付属書の対象外。
- policy（`dev/xtask/policy/workspace.toml`）の module_rules は基点から変わっていない。b〜g は、改修が足した・組み替えた code による。
- lib と cfg(test) の target は、同じ module の error を 2 回数える（下の数の「×2」）。

## 0. 全体の規則

- **規則を緩めて通さない**（Role の 80 字の上限、non-leaf の規則は変えない）。policy を変えるのは、下に「policy」と書いた所だけ。そこは、新しい構造が設計どおりで、規則がそれを許すべきときに限る。理由を policy の comment に書く。
- 各 unit は、module を動かすたびに自分の crate の検査を走らせる（`cargo xtask check --no-write --crate <crate>`。動かすと新しい循環ができうる）。target は `W:/w/kivact/target` 以外にする。
- 公開の API（公開の path と名前）は変えない。動かすのは非公開の module の中身と `pub(crate)` の item だけ。
- 一つの unit は、自分の crate の file だけを触る。policy の file は U6 だけが共有の節を触り、各 unit は自分の crate の `[module_rules.<crate>]` の節だけを触る。

## 1. 種類ごとの原因と直し方

### d. `derive macro expansion is unsupported: Prop`（122 = 61 module ×2）
- 原因：`#[derive(Prop)]`（`kivact-macros/src/lib.rs:44` の `derive_prop`）は改修で足した derive。解析器（`vendor/xtask-core/crates/xtask-modules/src/cfg.rs:99`）は、信頼の一覧に無い derive を報告する。Prop は `impl IntoValue<T> for T` だけを作り、module の参照を足さない。
- 直し（policy、U6）：`[analysis] trusted_derives` に `"Prop"` と `"kivact::Prop"` を足す（path の文字列で照合するので両方）。

### e. 解決できない明示の局所の path（50 = 25 module ×2）
1. **`#[component]` が作る型（kivact-ui の 14 module と creative の 1）**：
   - `#[component] pub fn button` が作る `pub struct Button` を、解析器（`symbols.rs:87` の `declaration`）が知らない。
   - 対象の参照：
     - `crate::Button`：chat_composer ×2、code_block::view、data_table::tree、pagination::view、toast::view
     - `crate::TextInput`：chat_composer、date_input::view、number_input::view、tag_input::view
     - `crate::Calendar`：date_input::view、date_parts::sheet、date_range_input::view
     - `crate::tree::TreeRows`：tree::view
     - creative の `crate::Number`：color_wheel::fields::implementation
   - **直し（xtask-core、U6）**：解析器に設定を足す。`[analysis]` の新しい設定 `generated_type_attributes = ["component"]`（`AnalysisOptions`（`model.rs`）と `AnalysisSettings`（`xtask-workspace/src/policy.rs:95`））。その attribute の付いた fn について、`declaration()` とその呼出し（`resolve.rs:30,45,75,168`、`visitor.rs:438`）が、PascalCase の fn の名前を Type と Value の item としても宣言する。
   - 試験：xtask-modules の analysis の試験に 1 件。
   - xtask-core は別の repository。`vendor/xtask-core` の中で branch `fix/kashiwade/component-generated-type` を切って commit し、kivact の submodule の pointer をその commit にする。**xtask-core への push はしない**（設計者が行う）。
2. **`Decl` の再公開の鎖（kivact-style の 6）**：`Decl` は `macro_rules style_props!`（`sheet/props.rs:29`）が作る。直し：`super::props::Decl` か `crate::sheet::props::Decl` を直接 import する（`props` を `pub(crate)` か `pub(super)` にする）。
3. **thread_local の static（kivact-engine の 4）**：`paint/state.rs:170,178` の `DROP_HIT_CANDIDATE`・`KEEP_STALE_DRAW`。直し：`paint/state.rs` に `pub(crate)` の get・set の関数を置いて呼ぶ。
4. **core の facade の glob（1）**：`kivact-core/src/view/span.rs:13` の `use crate::style::Paint`。直し：持ち主の path（`kivact_style::Paint`）から import する。

### b. style/non-leaf-implementation（260）
- 規則：test 以外の子の module（`mod x;` か中身のある inline の `mod x {}`）を持つ file は、`mod`・`use`・`extern crate` の他を置けない。
- 直しの形（全部 code）：中身を葉の module に移し、親は `mod` と `pub use` だけにする。kivact-ui は、`dialog.rs`・`toggle_group` のように中身を `x/view.rs` に置く形にそろえる。
  - **kivact-ui（128）**：
    - 中身を `x/view.rs` に移す component（27）：alert 13、chat_composer 11、button 9、toggle 7、avatar・button_group・chat_message・progress・toggle_group 6、card 4、async_view・avatar_group・kbd・sparkline 3、announcer_outlet・description_list・toggle_button 2、badge・combobox・empty_state・heading・skeleton・spinner・stat・toolbar・typing_indicator 1。
    - **罠**：`look.rs` が、移した先（view）の item を名前で呼んでいると、look と view の循環が新しくできる。そういう共有の型は `look.rs` か葉に置く。
    - `theme.rs` の 27 の inline の `pub mod <component> { pub use ... }`：それぞれ file `theme/<component>.rs`（`//! Role:` の行と `pub use` の行）にし、`theme.rs` は `pub mod <component>;` にする（c の 27 件も同時に消える）。
  - **kivact-style**（`theme/vars.rs` の 20）：inline の `mod sealed { pub trait Sealed {} }`（53 行）を `theme/sealed.rs`（Role 付き）に移す。
  - **kivact-core**：`app.rs` の inline の `pub(crate) mod test_launch`（219 行）を兄弟の module に移し、lib.rs の 94 行の再公開を直す。`lib.rs:91` の inline の `pub mod __test {..}` は file `__test.rs` にする。
  - **kivact-interaction**（date_field 16、switch_drag 9、swipe_dismiss 8）、**kivact-gpu-share**（vulkan 11、d3d12 4）、**kivact-engine**（`host/write.rs` 1）：親の共有の型と helper を新しい葉（`types.rs` など）に移す。f の親子の循環も同時に消える。date_field は `Parse` の alias・`DatePanel`・`DatePicker`・`TypedText` などを、engine は `stats_of`（`write.rs:52`）を葉に移す。
  - **kivact-macros**：
    - `expand.rs`（13）：中身を葉（`expand/component.rs`）に移す。
    - `lib.rs:42` の `derive_prop`（1）：proc-macro の入口は crate の root に要る。policy の style の例外を、今の例外（workspace.toml の 392〜395 行）と同じ理由で 1 つ足す（`item = "derive_prop"`、U6）。
  - **example の catalog の crate**（soft 28、creative 5、game 5、pop 4、basic 3）：lib.rs の `static FONTS`・`fn theme`・`const THEME` などを葉（`theme.rs`・`consts.rs`）に移す。soft の `pages.rs`（24）は、1000 行の上限も越えている（1386 行）。page ごとの file に分ける。

### f. module の循環（44 = 22 の循環 ×2）
親子の循環が多い：親の中身を子が `super::` で使い、親は子を使う。直しは、共有の item を葉に移すこと（b と同じ move）。

- **kivact-core**
  1. `app`・`launch::runtime`・`launch::settings`：
     - `AppSettings::with_theme -> App` は `app.rs` の `impl App` に移す。
     - `runtime` が `App` を持つのも policy（launch は app を使えない）に反する。そこで launch が要る値を、launch の中の plain な struct（launch の設定と root の view の関数など、runtime が `App` から読んでいるもの）にする。`App::run` などが `app.rs` の側でそれを作って launch に渡す（**設計者の決め**）。
  2. 大きな循環（`app_state`・`client::*`・`devtools::state`・`gpu_interop::import::registry`・`platform::*`・`view::{devtools_hooks,state}`）：
     - 原因は、R05 の `AppState` の hub。各 state の module が `AppState::of(rt)` を呼ぶ。
     - 直し（**設計者の決め**）：各 spoke の型の accessor を、`app_state.rs` の中に spoke の型の inherent impl（`impl HostState { pub(crate) fn of(rt: ..) -> .. }`）として置く。spoke は hub を参照しない。
     - 別の辺：`view/state.rs:327` が `platform::close_request::Registration` を使う。`Registration` を `view::state` に移す（g の 1 件も消える）。
  3. `view::element_data`・`view::part`・`view::part_data`：`HeldInput`（`part.rs:35`）を `part_data.rs` に移す。
- **arvane-compose** 4. `gpu::{device,state,surface}`：`GpuDevice` の struct を `state.rs` に置き、method は `device.rs` の `impl GpuDevice` に残す。
- **kivact-engine** 5. `host::write` と reader・writer：`stats_of` を葉に移す。
- **kivact-gpu-share**
  - 6・7. `vulkan`・`d3d12` と子：親の `USAGE`・`usage` などを葉に移す。
  - 8. `device`・`wgpu::vulkan`：`open_vulkan` を `wgpu/vulkan.rs` に移す。
- **kivact-interaction** 9〜11. `date_field`・`swipe_dismiss`・`switch_drag`：b の move で消える。
- **kivact-runtime** 12. `reactive::*`・`task::*`・`motion::state` の 16 module：
  - 原因は、R05 の `Runtime` の hub（`reactive/data.rs:96,98,504`）が `crate::task::TaskState` と `crate::motion::MotionState` を field に持つこと。
  - 直し（**設計者の決め**）：`TaskState` と `MotionState`（とその field の data の型：`RemoteWake`・`Deliver`・`ClockEntry`・`TimerKey` など）を、`reactive` を参照しない葉の module（`task::state`・`motion::state`）にする。`Runtime` を要る部分は、別の module に分ける。
  - そのうえで、policy の `[module_rules.kivact-runtime]` に `reactive` が `task::state` と `motion::state` を使ってよいことを足す（理由：「the runtime hub holds the task and motion state; those modules hold data only and use nothing of reactive」）。g の runtime の 12 件も消える。
  - 規則を top-level の `task`・`motion` に広げることはしない。
- **kivact-style** 13. 26 module の循環：
  - (1) `Resolve` と `resolved()`（`resolve/compile.rs:95,62`）を、下の新しい module `crate::length`（theme を知る長さの解決）に移す。`Resolve::new(&Groups..)` は compile.rs の impl に残す。policy：`[module_rules.kivact-style]` の `length` に `theme` を許す（理由：「the theme-aware length resolution sits below geometry and paint」）。g の 7 件も消える。
  - (2) `StyledRange`（`text.rs:378`）を `sheet` の中（`sheet/edit_text.rs` など）に移す（g の `text -> sheet::decl` も消える）。
  - 移した後に `theme::value` の `crate::internal::try_use_theme`（555 行）を確かめる。
- **kivact-types** 14. `layout`・`length`・`paint`：
  - `Length::to_len`（`length.rs:186-193`）を `paint.rs` の `impl Length` に移す。
  - policy：`length = ["theme_var"]`（paint を外す）、`layout` と `paint` に `length` を足す（理由：「Length is the authoring type layout and paint are built from」）。
- **kivact-ui**
  - 15. `combobox`：`ComboboxMode` を `data.rs` か葉に移す。
  - 16. `dialog`：`DialogKind` を `look.rs` に移す（公開の path は `kivact::ui::DialogKind` のまま）。
  - 17. `sidebar::{look,recipes,state}`：`SidebarState` と、recipes が読む item と theme の型を `state.rs` に移し、`look -> recipes -> state` の一方向にする。
- **kivact-ui-creative**
  - 18. `color`・`options`：`Accent` か色の定数を移す。
  - 19. `numeric::{builder,view}`：`NumericKind`・`PropertyState`・`Setup` を `numeric/types.rs` に移す。
- **kivact-ui-game** 20. `theme` と `size`・`recipes_*`：
  - `Density` を `size.rs` に移す。
  - `sparkline_table` を theme の下に移す。
  - recipes_controls と recipes_controls_choice の共有の helper を葉に移す。
- **kivact-ui-soft** 21. `theme`・`recipes_feedback`：`recipes_feedback` は `crate::theme::theme(inverted)` を呼ばず、theme を引数で受ける。
- **examples/kivact-ui-game** 22. `support::{records,state}`：`Record` か関数を葉に移す。

### g. module_rules の違反（27）
- **kivact-core（6）**：
  - `client::dispatch::env -> launch::close`（`env.rs:161` の `begin_exit`）：`begin_exit` を、`client` と `launch` の両方が使ってよい下の module（policy で確かめる。`platform::lifecycle` など）に移す。そういう module が無ければ返す。
  - `launch::{runtime,settings} -> app`：f の 1。
  - `view::measured_leaf -> custom::measurement`：`Constraints`・`Measurement` など `measured_leaf` が使う item を `view` の側に移す（`custom` は再公開する）。
  - `view::node_ref_visibility -> platform::env`：policy に、`view::elements::viewport` と同じ理由の例外を足す（「visibility needs the window size」）。
  - `view::state -> platform::close_request`：f の 2 の `Registration`。
- **kivact-runtime（12）**：f の 12。
- **kivact-style（8）**：f の 13。
- **kivact-types（1）**：f の 14。

### c. Role（326 = 166 module。80 字を越える 136、無い 30）
- 規則：module ごとに `//! Role:` の行がちょうど一つ。80 字以内（基点は通っていた。上限は上げない）。
- 直し：Role の行を 80 字以内に縮め、長い説明は次の `//!` の行に移す。無い module には足す（kivact-ui の theme の 27 は b の move で、kivact-style の `sealed`、arvane-shell の `surface::tests`、arvane-compose の `cpu::compositor::tests` は inline の `mod tests` の中に `//! Role:` を書く）。
- crate ごとの一覧は `python <scratchpad>/role2.py v` で出る（設計者の scratchpad）。log の module の path からも分かる。

### その他（13）
- kivact-macros の 6：`syn::parse_macro_input`・`syn::parse_quote` を policy の `trusted_macros` に足す（U6）。
- file の大きさ 2：`crates/kivact-ui-soft/src/color.rs`（1064 行）を分ける（色の群ごとの file）。`examples/kivact-ui-soft/src/pages.rs` は b で分ける。
- style の 3：
  - `kivact-ui-soft/src/variants.rs:6`（import の粒度）
  - `kivact-ui/src/date_range_input/draft.rs:21`（`&Option<T>` を `Option<&T>` に）
  - `kivact-runtime/src/task/remote.rs:21`（poison-access）
  - それぞれの規則のとおりに直す。
- 使われない style の例外 3（workspace.toml の 446・452・476 行付近）：消す（U6）。

## 2. unit（W11x。W11b と並べてよい。file は重ならない）

| unit | 所有 | 内容 |
|---|---|---|
| X1 | `crates/kivact-ui` | b 128、c 34、e 13（U6 の後）、f 15〜17、style の date_range_input |
| X2 | `crates/kivact-ui-soft`、`crates/kivact-ui-pop`、`examples/kivact-ui-soft`、`examples/kivact-ui-pop` | c、b、f 21、color.rs の分割、pages.rs の分割、variants.rs の style |
| X3 | `crates/kivact-ui-basic`・`-creative`・`-game`、その `examples/*`、`examples/catalog-shell` | c、b、f 18〜20・22、e 1（U6 の後） |
| X4 | `crates/kivact-core`、`crates/kivact-engine`、`crates/kivact-gpu-share`、`crates/kivact-macros`、`crates/arvane*` | b、c、e、f 1〜8、g の core |
| X5 | `crates/kivact-runtime`、`crates/kivact-types`、`crates/kivact-style`、`crates/kivact-interaction` | b、c、e（Decl）、f 9〜14、g の runtime・style・types、remote.rs の style |
| X6（U6） | `dev/xtask/policy/workspace.toml` の共有の節、`vendor/xtask-core` | d の policy、`trusted_macros`、derive_prop の例外、使われない例外の削除、e の xtask-core（push しない） |

- 順序：X6 を最初か並べて行う（X1・X3 の e は X6 の後に確かめる）。X1〜X5 は並べてよい。
- policy の各 crate の `[module_rules.<crate>]` の節は、その crate の unit が触る。
- 完了：統合で `cargo xtask check --no-write` の error が、W11b の文書の分を除いて 0 件。W11b の後は全体で 0 件。
- **設計者の決め**と書いた所で、書いた形にできない理由が出たら返す。

## 3. 設計者の答え（2026-11-22、Orchestrator 5 の W11x の質問）

- **X4 の付属書との違い**：すべて受ける（`paint::state` の名前、`open_vulkan` の `vulkan/open.rs`、`view/measurement.rs`、`begin_exit` の `platform/lifecycle.rs`、`span.rs` の `kivact_style::paint::Paint` の import）。公開の path は変わらないので、意図どおり。
- **X1 の `SidebarTheme`**：`look.rs` に残してよい。条件は、sidebar の 3 module の循環が消えていて、辺が一方向であること。kivact-ui の theme の型は `look.rs` に置くのが他の component と同じ形（`DialogKind` も `look.rs`）。
- **X2 の `color.rs` の分割**：(a) に近い形にする。policy の `[kit_literal_color]` は変えない。
  - `color.rs`：theme の変数（`*_VAR` と `Color::var` の定数。名前と明るい外観の既定の値）だけを置く。
  - `tokens.rs`（新）：`set_colors` とその補助の関数（外観ごとの値）を移す。
  - 色の群ごとの file には分けない（basename の制約のため。群は file の中の節の見出しで示す）。
  - どちらの file も 1000 行に収まること。
- **X2 に足す（G11 の STOP (b)）**：
  - soft の `ListDividers`・`TableCellAlign`・`TableCellDivider`・`TableRowStriped` に `#[derive(Prop)]` を付ける（K28 の V7。`.variant(..)` が通ること）。
  - soft の `*_VAR` を `pub` にし、basic・pop と同じく crate の root から公開する（付属書 K の 4.4.5）。
- **X5 (1) runtime の hub（f12）**：付属書 1 章 f12 の「設計者の決め」を、次で置き換える。前の決めは、`TaskState` の field の型（`Executor`・`Scheduler`・`Pool`・`Drain`・`Deliver`・`ClockEntry`・`Owned`）と `MotionState` の field の型（`LoopCell`・`Anim`）が `reactive` を使うので、移すだけでは作れない。
  - 方針：`reactive` は task と motion を知らない。依存は `task -> reactive`・`motion -> reactive` の一方向にする（依存の逆転）。
  - `reactive::data::Runtime` から `tasks`・`motion`・`motion_env` の field を外す。代わりに、型で引く拡張の欄（`theme.ext::<XTheme>()` と同じ形）を持たせる。
    - 欄は `Runtime` を作るときに一度だけ埋め、後から足したり消したりしない（state の印を付ける）。
    - 埋めるのは、`reactive`・`task`・`motion` の上にある組み立ての module（今の `crate::host` か `crate::driver` の、`Runtime` を作る所）。
  - task と motion は自分の状態を、自分の module の accessor で引く（例：`TaskState::of(rt)`、`MotionState::of(rt)`。中で拡張の欄を引く）。`crate::host` の `tasks()`・`motion()` も、それを呼ぶだけにする。
  - turn が task と motion に頼む仕事（`has_ready`、`remote_wake` の確認と取り出し、他の thread からの便の配達、motion の環境）は、`reactive` が定める trait（turn に加わる subsystem の trait）を通す。今の `task::remote::Deliver` の役は、この trait に入れる。
  - 欄が無いのはプログラムの誤りなので、引けないときの扱いは crate の今の panic の規則に従う。
  - 前の決めの policy の追記（`reactive` が `task::state`・`motion::state` を使ってよい）は行わない。
  - `cargo test -p kivact-runtime` と、`kivact-core` の turn の test が通ること。
- **X5 (2) `StyledRange`（f13 (2)）**：`sheet` に移し、crate の root（`kivact_style::StyledRange`）と facade（`kivact::style::StyledRange`）から公開する。`kivact_style::text::StyledRange` の path は消してよい。同じ型が 2 つの path にあるのが API-10 の問題なので、root を正の path とする（workspace の中に `text::StyledRange` の利用者はいない）。`PR_DRAFT.md` の破壊的な変更の一覧に 1 行書く。
- **X5 (3) `draw` と `length`（f13 (1)）**：policy の `[module_rules.kivact-style]` の `draw` の uses に `length` を足す（理由：「draw resolves theme-aware lengths when it records」）。`length` は geometry と paint の下に置いた層なので、上の `draw` が使うのは方向どおり。
- **G11 の STOP (a) `ZoomPan`**：設計（`REWORK_DESIGN.md` の 5.12・API-10）どおり、コードを改名する。付属書 U の手順から改名が抜けていた。文書は直さない。
  - 新しい unit **X7**（X5 を統合に入れた後。所有は `crates/kivact-types`、`crates/kivact-interaction`、`ViewTransform` の全利用者（`crates/`・`examples/`・`dev/`・`templates/`））：
    - `kivact_types::ViewTransform` を `ZoomPan` に改名する（file `view_transform.rs` は `zoom_pan.rs`）。
    - `kivact::interaction::ZoomPan` が正の path。`ViewTransform` の名前と別名は残さない。
    - doc の文の「view transform」は、zoom-pan の状態という意味に直す。
  - 完了：`rg -w ViewTransform crates examples dev templates` が何も返さず、`interaction_zoom_pan` の e2e が通る。
