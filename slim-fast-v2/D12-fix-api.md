# R4 監査：公開 API の unit（target lane D）

> **V2.1 修正付属書 D12**（再監査 R4 の成果物、HEAD `2a3b06b8`＝W1 の後に対して書いた）。範囲：W5 の G5〜G8、W7 の FINAL-1・G9（公開 API）。
> - **優先順位**：本書は付属書 D1〜D8 より優先する。本書の unit の節の「修正」は、作業指示書の行の「本書の追加の指示」と同じ扱い。本書と V2 設計書 4 章（DEC）が食い違う場合は V2 設計書が正（V2 設計書 4 章の DEC-18 の名前は `while_open(factory)`）。
> - 本書の「全体」節の wave・lane の組み直しは、作業指示書（V2.1）に反映済み。実施順・所有ファイルは作業指示書が正。
> - 文中の worktree（`W:/w/kv-audit-*`）・scratchpad の path は監査の試作の場所で、実装に必要ではない（試作の差分は参考で、実装は本書の文に従う）。

対象：W5 の G5・G6・G7・G8、W7 の FINAL-1・G9。HEAD `2a3b06b8`。

## 0. 調べ方と試作（前提）

- 静的な読解に加え、`git worktree add --detach W:/w/kv-audit-R4 HEAD`（target `W:/kv-target/D`）で次を試作し、compile・lint を実測した。worktree は最後に消す（本書の末尾）。
  - FINAL-1：`PartState` の 4 field に一時的に `#[deprecated]` を付けて `cargo check --workspace --all-targets --all-features` を実行し、**field の読み書きの全箇所**を warning として列挙した（compile error 駆動では crate が順に止まるため、一度では全部出ない）。続けて付属書どおりの `Availability` に変えて state-fields を実行した。
  - G5：付属書の C-1〜C-4 を書き、E2E を compile。C-1 の修正を外すと E2E が compile error になること（不具合の実在）を確かめた。`cfg` の扱いを実測した（下記 G5 の 2）。
  - G7：api-shape の新しい規則を書き、workspace 全体に掛けた。
  - G6 の C-15：prelude から `provide_theme` を外し、下記 2 ファイルに import を足して `cargo check --workspace --all-targets --all-features` が exit 0。
  - G8：下記の形で `kivact::launch` を作り、`cargo check -p kivact --no-default-features`／`--features gpu-interop`／`-p kivact-public-api-tests --tests --features gpu-interop`／`-p kivact-test --all-targets` が成功。
  - 上の G5・G7・G8・C-15 の試作を全部入れた状態で `cargo xtask check --no-write`（全 rule）：error は fmt の 3 件（`cargo fmt` で消える）と HEAD 既存の state-fields 3 件（`cache.rs:53`、`semantics.rs:14`、`data.rs:331`）だけ。canonical-path・api-shape・module 規則の新しい error は無い。`cargo clippy -p kivact-macros`・`-p xtask --all-targets -- -D warnings` も成功。
- 試作の差分（参考。正は本書の文）：`<scratchpad>/r4-G5.patch`、`r4-G7.patch`、`r4-C15.patch`。docs の重複 import の機械的な削除は `<scratchpad>/r4_dedupe.py`（本書 G6 に全文を載せる）。`<scratchpad>` = `C:\Users\ryo\AppData\Local\Temp\claude\fd3f2b4db3f6db24\76c64dca-56c5-4a94-ab86-dbee8a09d336\scratchpad`。

---

## G5（付属書 D6 9 章 C-1〜C-4、W5 lane A）

### 判定：修正あり

付属書の C-2 の前提の半分が誤り（`cfg` はマクロに届かない）、`expect` と `deprecated` を全 item に付けると generated code が warning/error になる。C-6 は G5 の所有ファイルにあるので G5 に移す。

### 修正（本書の追加の指示）

1. **C-1（実在を確認）**。`crates/kivact-macros/src/expand.rs` の `fn mount` の `match &prop.kind {` の直後、既存の `PropKind::Text | PropKind::Value(_) =>` の腕より**前に**次の 2 腕を足す（順序が逆だと効かない）。
   ```rust
   PropKind::Text if prop.is_optional() => Some(quote! {
       let #name = #name.map(|v| ::kivact::state::IntoValue::<::std::string::String>::into_value(v));
   }),
   PropKind::Value(ty) if prop.is_optional() => Some(quote! {
       let #name = #name.map(|v| ::kivact::state::IntoValue::<#ty>::into_value(v));
   }),
   ```
2. **C-2（付属書を差し替え）**。実測：rustc は `#[cfg]`・`#[cfg_attr]` を属性マクロの前に評価する（`#[component] #[cfg(false)] fn card..` は二つ目の `card` があっても衝突せず、`#[component] #[cfg(all())]` もマクロに `cfg` が渡らない）。よって `cfg`・`cfg_attr` は扱わない。マクロが黙って捨てているのは不活性な属性（`allow`・`warn`・`deny`・`deprecated`・`expect`・`must_use`・`inline`・`track_caller` など）。`expect` を 4 つの item 全部に付けると lint が出ない item で `unfulfilled_lint_expectations` になり、`deprecated` を `impl` に付けると無効または `useless_deprecated` になるので、次の規則にする。
   - `crates/kivact-macros/src/parse.rs` の `struct Model` の `docs` の次に足す：
     ```rust
     /// `allow`, `warn` and `deny`: put on every generated item.
     pub(crate) attrs: Vec<Attribute>,
     /// `deprecated`: put on the builder function, the name callers write.
     pub(crate) deprecated: Vec<Attribute>,
     ```
   - `fn component` の `let docs = function.attrs.iter().filter(..).cloned().collect();` を次で置き換え、`Ok(Model { .. })` に `attrs, deprecated,` を足す：
     ```rust
     let mut docs = Vec::new();
     let mut attrs = Vec::new();
     let mut deprecated = Vec::new();
     for attr in &function.attrs {
         let path = attr.path();
         if path.is_ident("doc") {
             docs.push(attr.clone());
         } else if path.is_ident("deprecated") {
             deprecated.push(attr.clone());
         } else if ["allow", "warn", "deny"].iter().any(|name| path.is_ident(name)) {
             attrs.push(attr.clone());
         } else {
             return Err(syn::Error::new_spanned(
                 attr,
                 "#[component] keeps doc comments and the allow, warn, deny and deprecated attributes; use #[allow(..)] instead of #[expect(..)] and remove any other attribute",
             ));
         }
     }
     ```
   - `expand.rs`：`fn component` の quote で struct の前と `impl #struct_name {` の前に `#(#attrs)*`（`let attrs = &model.attrs;`）。`fn constructor` の quote で `#(#docs)*` の前に `#(#attrs)* #(#deprecated)*`（`let (attrs, deprecated) = (&model.attrs, &model.deprecated);`）。`fn into_view` の quote で `impl ::kivact::view::IntoView` の前に `#(#attrs)*`。
   - 試作で `#[allow(dead_code, reason = "..")] #[deprecated(note = "..")]` を付けた component が clippy で warning 0 を確認済み。
3. **C-3**。`parse.rs` の `fn check_signature` の `let sig = &function.sig;` の直後に足す：
   ```rust
   if sig.constness.is_some()
       || sig.asyncness.is_some()
       || sig.unsafety.is_some()
       || sig.abi.is_some()
       || sig.variadic.is_some()
   {
       return Err(syn::Error::new_spanned(
           &sig.ident,
           "a #[component] function is a plain `fn`",
       ));
   }
   ```
4. **C-4**。`expand.rs` の `fn initializer` の `Required::Default(expr)` の腕を次にする（全 kind 共通。callback 以外は今と同じ振る舞い）：
   ```rust
   Required::Default(expr) => {
       let value = store(prop, quote! { __default });
       quote! { #name: { let __default = #expr; #value } }
   }
   ```
5. **C-6（G6 から移す）**。`docs/for-kivact-user/en/how-to-guide/custom-components.md` の「An optional callback is … because the conversion needs the component's setup.」の 2 文目以降を次にする：`` An optional callback is `#[prop(optional)] on_close: Option<Callback<()>>`: the body receives `Option<Callback<()>>`, already converted in the component's setup. `` jp（同 68 行付近）：「省略可能なコールバックは `#[prop(optional)] on_close: Option<Callback<()>>` と書きます。本体が受け取るのは、コンポーネントの setup で変換済みの `Option<Callback<()>>` です。」（`on_close.map(|f| f())` の括弧書きと「変換にはコンポーネントの setup が要るためです。」を消す）。
6. 同じ guide の「What `#[component]` rejects」の列の最後（`- an unknown key in `#[prop(..)]`.` の後）に 2 行を足す：
   - en：`` - an `async`, `const`, `unsafe` or `extern` function: a `#[component]` function is a plain `fn`; `` と `` - an attribute other than doc comments, `allow`, `warn`, `deny` and `deprecated` (write `#[allow(..)]` instead of `#[expect(..)]`). ``（直前の行末の `.` は `;` にする）
   - jp（「`#[component]` が受け付けないもの」）：`- `async`、`const`、`unsafe`、`extern` の関数: `#[component]` の関数は普通の `fn` です。` と `- doc コメント、`allow`、`warn`、`deny`、`deprecated` 以外の属性（`#[expect(..)]` の代わりに `#[allow(..)]` を書きます）。`
7. **E2E（C-1 の確認。足す試験はこの 1 本だけ）**。`dev/tests/kivact-public-api/tests/e2e/component_macro.rs` の末尾に足す：
   ```rust
   /// A heading with an optional subtitle and an optional level.
   #[component]
   fn heading(
       title: Value<String>,
       #[prop(optional)] subtitle: Option<Value<String>>,
       #[prop(optional)] level: Option<Value<u32>>,
   ) -> impl IntoView {
       let level = level.map(|level| text(move || format!("level={}", level.get())));
       column((text(title), subtitle.map(text), level))
   }

   #[test]
   fn optional_value_props_are_none_or_reactive() {
       let (screen, subtitle) = mount(|| {
           let subtitle = signal(String::from("Second"));
           let view = column((
               heading("Plain"),
               heading("Full").subtitle(subtitle).level(2_u32),
           ));
           (subtitle, view)
       });
       screen.get_by_text("Plain");
       screen.get_by_text("Full");
       screen.get_by_text("Second");
       screen.get_by_text("level=2");
       assert_eq!(screen.all_by_text("level=2").len(), 1);
       subtitle.set(String::from("Changed"));
       screen.run_until_idle();
       screen.get_by_text("Changed");
       assert!(screen.errors().is_empty(), "{:?}", screen.errors());
   }
   ```
   （`query_all_by_text` は無い。`all_by_text` を使う。`2` は `{integer}` の推論で `i32` になるので `2_u32`。）
8. state-fields：`Model` に足すのは `Vec` だけで数えられない。新しい local も `Vec` だけ。marker は不要。行数：`component` は約 45 行、`check_signature` は約 40 行で上限内。
9. 完了時の確認（付属書の行に加える）：`cargo clippy -p kivact-macros --all-targets -- -D warnings`、`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- component_macro`、`dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-docs --doc -- custom_components`。

### 所有ファイルの変更

- 付属書どおり（`crates/kivact-macros/src/{parse.rs,expand.rs}`、`dev/tests/kivact-public-api/tests/e2e/component_macro.rs`、`docs/for-kivact-user/{en,jp}/how-to-guide/custom-components.md`）。C-6 をこの unit が行う。

---

## G6（付属書 D6 9 章 C-6〜C-11・C-15、W5 lane C の 1 番目）

### 判定：修正あり

### 修正（本書の追加の指示）

順序：C-15 → C-11 の重複 import の削除（スクリプト）→ C-7〜C-11 の残り。C-6 は行わない（G5 が行う。`custom-components.md` は G5 の所有）。

1. **C-15**（DEC-20）。
   - `crates/kivact/src/prelude.rs` の `pub use crate::style::{..., pct, provide_theme, rgb, style};` から `provide_theme` だけを消す（他の `use` 行は統合しない）。
   - 呼出し元は実測で次の 2 ファイルと docs の 6 ブロックだけ（kit・examples・他の試験は明示の import か path を使っている）。各々で `use kivact::prelude::*;` の次の行に `use kivact::style::provide_theme;` を足す：
     - `crates/kivact-devtools/src/app.rs`（28 行の呼出し）
     - `dev/tests/kivact-public-api/tests/e2e/windows.rs`（87 行の呼出し）
     - `docs/for-kivact-user/{en,jp}/getting-started/06-style-and-accessibility.md`（9 行目から始まる rust ブロック）、`07-async.md`（9 行目から）、`08-testing.md`（20 行目から）。docs は doctest なので `cargo check` では検出されない。
   - W5 の他の lane（G5：macros・`component_macro.rs`・`custom-components.md`、G7：`calendar/view.rs`・`menu/handle.rs`・`api_shape.rs`）とは重ならない。
   - D6 C-15 の前半：`crates/kivact-test/src/lib.rs` の doc の `//! use kivact_core::view::{Element, text};` を `//! use kivact_core::view::text;`、`crates/kivact-test/src/render.rs` の `/// use kivact::view::{Element, text};` を `/// use kivact::view::text;` にする。
2. **C-7**（リンクと文言。行番号は HEAD）：
   - `crates/kivact-core/src/app.rs:23`、`embed.rs:213`：`` [`view::provide_theme`](crate::view::provide_theme) `` → `` [`style::provide_theme`](crate::style::provide_theme) ``
   - `app.rs:33`、`launch/settings.rs:73`：`` [`view::provide_locale`](crate::view::provide_locale) `` → `` [`i18n::provide_locale`](crate::i18n::provide_locale) ``
   - `app.rs:42`、`launch/settings.rs:83`：`` [`view::provide_messages`](crate::view::provide_messages) `` → `` [`i18n::provide_messages`](crate::i18n::provide_messages) ``
   - `launch/settings.rs:62` の `` `view::provide_theme` `` → `` `style::provide_theme` ``、`:188` の error 文字列の `use view::provide_theme` → `use style::provide_theme`（この文字列を assert する試験は無い）
   - `view/boundary.rs:264,297`：`` `kivact::view::provide_locale` `` → `` `kivact::i18n::provide_locale` ``。`:320`：`` `kivact::view::provide_messages` `` → `` `kivact::i18n::provide_messages` ``
   - `crates/kivact-style/src/theme/value.rs:198,495`、`crates/kivact-ui-creative/README.md:24`：`` `kivact::view::provide_theme` `` → `` `kivact::style::provide_theme` ``
   - `docs/for-kivact-user/en/how-to-guide/coming-from-react-or-solid.md:375-376` の文を次にする：`` - Themes, locale, messages and reading direction have their own `provide_*` functions: `kivact::style::provide_theme`, `kivact::i18n::provide_locale` and `kivact::i18n::provide_messages`, and `kivact::view::provide_direction` ([Theming](theming.md), [Internationalization](i18n.md)). `` jp:326：「テーマ、ロケール、文言、読み取りの向きには、それぞれ専用の `provide_*` 関数があります（`kivact::style::provide_theme`、`kivact::i18n::provide_locale`、`kivact::i18n::provide_messages`、`kivact::view::provide_direction`。[テーマ](theming.md)、[翻訳とロケール](i18n.md)）。」
3. **C-8**：次の行の `` `Element` `` を `` `Locator` `` にする（他の語は変えない）：en `how-to-guide/testing.md:104,179,336`、`windows.md:325`；jp `testing.md:88,153,287`、`windows.md:268`。
4. **C-9**：`examples/kivact-ui-soft/src/docs/overlays.rs` の表示用の文字列を次にする。
   - `:203` `"menu(\"Actions\", items).on_activate(on_activate).attach(button(\"Actions\"))"`
   - `:219` `"menu(\"Document actions\", items).on_activate(on_activate).attach_context(button(\"Document\"))"`
   - `:270` `"more_menu(items, on_activate)"`
   - 追加（同じ誤り）`:160` `"media_overlay(text(\"Document preview\")).attach(button(\"Cover\"))"`（`media_overlay` は closure でなく `impl IntoView` を取る）
5. **C-10**：`crates/kivact-ui/src/lib.rs:108-110` の 3 行を次の 4 行にし、`:114` の `//! use kivact::ui::ButtonVariant;` を消す。
   ```text
   //! Handlers are closures. One without a payload takes
   //! `impl Into<Callback<()>>` (a closure `move || ..` or a shared `Callback`);
   //! one with a payload takes `impl Fn(T) -> R`, where `R` is `()` or
   //! `Result<(), UiError>`. Bool options such as [`Button::hide_label`] take a constant or a reactive value.
   ```
6. **C-11**：
   - `examples/kivact/component/src/main.rs:22`：`{ "Collapse" } else { "Expand" }.to_owned())` → `{ "Collapse" } else { "Expand" })`（`IntoText` は `Into<String>` を返す closure を受ける）。
   - `examples/kivact/form/src/main.rs:23`：`kivact::view::InputPurpose::Password` → `InputPurpose::Password`。
   - `kivact::ui::ButtonVariant::` → `ButtonVariant::`：`docs/for-kivact-user/{en,jp}/getting-started/06-style-and-accessibility.md`（2）、`07-async.md`（1）、`08-testing.md`（1）、`how-to-guide/overlays.md`（2）、`theming.md`（5）。いずれも `use kivact::prelude::*;` を持つブロック。`docs/for-kivact-ui-user/**` は変えない（V2 の範囲外）。
   - 重複 import：次のスクリプトを worktree の外（scratchpad）に置き、`python -I <script> <worktree> --apply` で一度だけ実行する。規則は作業指示書の文と同じ（prelude が export する名前と**完全な path が同じ**ものだけを消す。名前だけ同じで path が違うものは残る。C-15 の後なので `provide_theme` は消さない）。実測で 36 ファイル・228 名（canvas.md 19、custom-controls.md 13、animation.md 11 ほか、en/jp 同数）。`custom-components.md` には該当が無い（G5 と重ならない）。
     ```python
     # r4_dedupe.py — usage: python -I r4_dedupe.py <repo root> [--apply]
     import glob, os, re, sys
     root = sys.argv[1]; apply = '--apply' in sys.argv
     prelude = open(os.path.join(root, 'crates/kivact/src/prelude.rs'), encoding='utf-8').read()
     exports = set()
     for m in re.finditer(r'pub use crate::([\w:]*?)(?:::)?\{([^}]*)\};|pub use crate::([\w:]+)::(\w+);|pub use crate::(\w+);', prelude, re.S):
         if m.group(2) is not None:
             mod = m.group(1)
             for n in m.group(2).split(','):
                 n = n.strip()
                 if n: exports.add(f'kivact::{mod}::{n}' if mod else f'kivact::{n}')
         elif m.group(3): exports.add(f'kivact::{m.group(3)}::{m.group(4)}')
         else: exports.add(f'kivact::{m.group(5)}')
     exports.discard('kivact::style::provide_theme')
     def split_names(body):
         out, depth, cur = [], 0, ''
         for ch in body:
             if ch == '{': depth += 1
             if ch == '}': depth -= 1
             if ch == ',' and depth == 0: out.append(cur.strip()); cur = ''
             else: cur += ch
         if cur.strip(): out.append(cur.strip())
         return out
     total = 0
     for path in sorted(glob.glob(os.path.join(root, 'docs/for-kivact-user/*/**/*.md'), recursive=True)):
         lines = open(path, encoding='utf-8').read().split('\n')
         blocks, start = [], None
         for i, line in enumerate(lines):
             s = line.strip()
             if s.startswith('```'):
                 if start is None and s.startswith('```rust'): start = i
                 elif start is not None: blocks.append((start, i)); start = None
         changed = False
         for a, b in blocks:
             if not any(l.strip() == 'use kivact::prelude::*;' for l in lines[a + 1:b]): continue
             for k in range(a + 1, b):
                 line = lines[k]
                 if line is None: continue
                 m = re.match(r'^(\s*)use (kivact::[\w:]*?)::\{(.*)\};\s*$', line)
                 single = re.match(r'^(\s*)use (kivact::[\w:]+)::(\w+);\s*$', line)
                 if m:
                     indent, base, inner = m.groups()
                     names = split_names(inner)
                     keep = [n for n in names if f'{base}::{n}' not in exports]
                     if len(keep) == len(names): continue
                     total += len(names) - len(keep); changed = True
                     if not keep: lines[k] = None
                     elif len(keep) == 1 and '{' not in keep[0]: lines[k] = f'{indent}use {base}::{keep[0]};'
                     else: lines[k] = f'{indent}use {base}::{{{", ".join(keep)}}};'
                 elif single and f'{single.group(2)}::{single.group(3)}' in exports:
                     total += 1; changed = True; lines[k] = None
         if changed and apply:
             open(path, 'w', encoding='utf-8', newline='').write('\n'.join(l for l in lines if l is not None))
     print('total names removed:', total)
     ```
     実行後に `--apply` なしで再実行し `total names removed: 0` を確かめる。
7. 完了時の確認（付属書の行を置き換える）：`cargo check --workspace --all-targets --all-features`；`cargo doc -p kivact-core --no-deps` と `cargo doc -p kivact-style --no-deps`（警告 0）；`cargo check -p kivact-ui-soft --all-targets`；`KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 3600 -- cargo test -p kivact-docs --all-features --doc -- --test-threads=1`（docs の 36 ファイルを変えるので全件。`gpu-texture-sharing.md` は feature `gpu-interop` の時だけ compile される）；`KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- windows::`。

### 所有ファイルの変更

- 追加：`crates/kivact-devtools/src/app.rs`、`dev/tests/kivact-public-api/tests/e2e/windows.rs`（C-15 の import 1 行ずつ）、`crates/kivact-test/src/{lib.rs,render.rs}`（doc の import だけ）。
- 除外：`docs/for-kivact-user/{en,jp}/how-to-guide/custom-components.md`（C-6 ごと G5）。

---

## G7（付属書 D6 9 章 C-12、W5 lane B）

### 判定：修正あり

付属書の「試験の期待を 3 件に」は誤り（4 件）。`impl Fn()` を拒む規則は「戻り値なし・receiver あり・`-> Self`」に絞らないと、slot の factory（`impl Fn() -> V`）など 13 件を誤検出する（試作で実測）。

### 修正（本書の追加の指示）

1. `crates/kivact-ui/src/calendar/view.rs` の `on_commit` を次で置き換える（呼出し元は workspace に 0 件）：
   ```rust
   pub fn on_commit(mut self, f: impl Into<Callback<()>>) -> Self {
       self.on_commit = Some(f.into());
       self
   }
   ```
   （`Callback` は同ファイルで import 済み。doc はそのまま。）
2. `crates/kivact-interaction/src/menu/handle.rs` の `trigger_action` を次にする（唯一の呼出し元 `crates/kivact-ui-soft/src/nav_heading.rs:147` の `.trigger_action(move || nav_panel.toggle())` は変更なしで compile する。`Callback` は `Copy`）：
   ```rust
   pub fn trigger_action(self, action: impl Into<Callback<()>>) -> Self {
       let action = action.into();
       self.session
           .options
           .update(|options| options.trigger_action = Some(action));
       self
   }
   ```
3. `dev/xtask/src/api_shape.rs`：
   - `fn fn_bound_handler` の `let syn::PathArguments::Parenthesized(arguments) = .. else { return false; };` の直後に `if arguments.inputs.is_empty() { return false; }` を足す（payload の無い handler は `impl Into<Callback<()>>` だけ）。
   - `fn check_signature` の `on_` の検査の `if` の後、`SLOTS` の検査の前に足す：
     ```rust
     if method && sig.receiver().is_some() && returns_self(sig) && args.iter().any(|ty| unit_fn(ty)) {
         source.error(
             "api-shape",
             sig.span(),
             "a builder method takes a handler without a payload as impl Into<Callback<()>> (ARG-1)",
             errors,
         );
     }
     ```
   - `fn handler_type_param` の前に 2 関数を足す：
     ```rust
     /// `-> Self`.
     fn returns_self(sig: &syn::Signature) -> bool {
         matches!(&sig.output, syn::ReturnType::Type(_, ty)
             if matches!(ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("Self")))
     }

     /// `impl Fn()`: a handler without a payload taken as a closure.
     fn unit_fn(ty: &syn::Type) -> bool {
         let syn::Type::ImplTrait(ty) = ty else {
             return false;
         };
         ty.bounds.iter().any(|bound| {
             let syn::TypeParamBound::Trait(bound) = bound else {
                 return false;
             };
             bound.path.segments.last().is_some_and(|segment| {
                 segment.ident == "Fn"
                     && matches!(&segment.arguments, syn::PathArguments::Parenthesized(arguments)
                         if arguments.inputs.is_empty()
                             && matches!(arguments.output, syn::ReturnType::Default))
             })
         })
     }
     ```
   - 試験 `checks_reachable_public_shapes`：fixture の `pub fn shared(self) -> Rc<Data> { loop {} }` の後に ` pub fn trigger(self, f: impl Fn()) -> Self { self }` を足し、`assert_eq!(errors.len(), 5);` → `assert_eq!(errors.len(), 6);`。
   - 試験 `accepts_arg1_callback_bounds_and_lifecycle_exceptions`：`assert_eq!(errors.len(), 2, "{errors:?}");` → `assert_eq!(errors.len(), 4, "{errors:?}");`（`on_inline` と `on_path` が新たに error。残りの assert はそのまま）。
4. 実測：1〜3 の後、`cargo xtask check --no-write --only api-shape` は 0 件（1・2 の前は、ちょうど `menu/handle.rs:190` と `calendar/view.rs:280` の 2 件）。`cargo test -p xtask api_shape` 4 件成功。`cargo check -p kivact-ui -p kivact-interaction -p kivact-ui-soft --all-targets` 成功。
5. 完了時の確認（付属書の行に加える）：`cargo check -p kivact-ui-soft --all-targets`、`cargo clippy -p xtask --all-targets -- -D warnings`。

### 所有ファイルの変更

なし（付属書どおり）。

---

## G8（付属書 D6 9 章 C-5・8 章 1、DEC-17、W5 lane C の 2 番目）

### 判定：修正あり（重大）

**`kivact::launch` は facade に存在しない**。facade（`crates/kivact/src/lib.rs`）は `kivact_core` の `custom, diagnostics, draw, geometry, i18n, motion, platform, state, style, task, view` だけを再公開する。`kivact_core::launch` は core と kivact-test の間の内部境界（`Launch`、`ClientMode`、`TestTiming(pub Rc<dyn Fn(u64)>)`、`LaunchHandle`、`Inspect`、`AppOptions`）で、丸ごと再公開すると内部型が公開され、api-shape も `TestTiming` の `Rc` で error になる。DEC-17 の path `kivact::launch::GpuPreference` を守るため、facade に GPU の 5 型だけを持つ `launch` module を作る。

### 修正（本書の追加の指示。付属書 8 章 1 の「推奨」の具体化）

1. `crates/kivact-core/src/launch.rs` の末尾に足す：
   ```rust
   /// An adapter's identity, as the graphics APIs report it.
   #[doc(inline)]
   pub use kivact_types::AdapterMatch;
   /// A graphics API the window's GPU device can use.
   #[doc(inline)]
   pub use kivact_types::GpuBackend;
   /// Which kind of adapter an App prefers ([`App::gpu_preference`](crate::App::gpu_preference)).
   #[doc(inline)]
   pub use kivact_types::GpuPower;
   /// What an App wishes of the window's GPU device ([`App::gpu_preference`](crate::App::gpu_preference)).
   #[doc(inline)]
   pub use kivact_types::GpuPreference;
   /// When the window's GPU device is started (`GpuPreference::start`).
   #[doc(inline)]
   pub use kivact_types::gpu::GpuStart;
   ```
2. `crates/kivact-core/src/gpu_interop.rs`：`AdapterMatch`、`GpuBackend`、`GpuPower`、`GpuPreference`、`gpu::GpuStart` の 5 つの `pub use`（各 3 行：doc・`#[doc(inline)]`・`pub use`）を消す。`GpuAdapter`、`SharedHandle`、`SharedTextureFormat`、`SharedTextures` は残す（別名は作らない）。
3. `crates/kivact-core/src/app.rs` と `launch/settings.rs` の `gpu_preference`：`#[cfg(feature = "gpu-interop")]` を消し、引数の型を `crate::launch::GpuPreference` にし、doc を次にする（private の `gpu_interop` への intra-doc link は feature なしの `cargo doc` で警告になるので、link にしない）：
   ```rust
   /// What the App wishes of the window's GPU device: when it starts
   /// ([`GpuStart`](crate::launch::GpuStart), default on demand), the kind of
   /// adapter, the graphics API and the adapter itself (default: Arvane's own
   /// choice). A wish: the device is made on another adapter or API when the
   /// one asked for is not there, and the environment variables
   /// `ARVANE_RENDERER`, `WGPU_BACKEND` and `WGPU_POWER_PREF` override it. A
   /// build without the feature `gpu` draws on the CPU and ignores it. Read the
   /// adapter the device is on with `kivact::gpu_interop::adapter` (feature
   /// `gpu-interop`).
   ```
4. 新規 `crates/kivact/src/launch.rs`：
   ```rust
   //! Role: What an App wishes of its window's GPU device at launch.
   //!
   //! Give a [`GpuPreference`] to `App::gpu_preference`. A build without the
   //! feature `gpu` draws on the CPU only and ignores it.

   #[doc(inline)]
   pub use kivact_core::launch::{AdapterMatch, GpuBackend, GpuPower, GpuPreference, GpuStart};
   ```
   `crates/kivact/src/lib.rs` の `/// Everyday names ..` と `pub mod prelude;` の前に `/// What an App wishes of its window's GPU device at launch ([`App::gpu_preference`]).` と `pub mod launch;` を足す。
5. `crates/kivact-test/src/options.rs` の `RenderOptions::gpu_preference`：`#[cfg(feature = "gpu-interop")]` を消し、引数を `kivact_core::launch::GpuPreference` にする（G2 の `TestGpu`・`into_host` の `start` の決め方・doc は変えない。G2 と整合：`start` は `gpu()`／`gpu_on_demand()` が決め、power・backend・adapter は `..initial.gpu_preference` で通る）。kivact-test の feature `gpu-interop` は facade から転送されるので残す。
6. 呼出し元の import（実測で全部）：
   - `dev/tests/kivact-public-api/tests/e2e/gpu_interop.rs`：`use kivact::gpu_interop::{ self, AdapterMatch, ExternalImage, FramePublisher, GpuAdapter, GpuBackend, GpuPreference, ImportStatus, ShareLink, ShareMode, ShareRequest, ShareStatus, SharedHandle, SharedImage, SharedTextureFormat, SharedTextures, };` → `use kivact::gpu_interop::{ self, ExternalImage, FramePublisher, GpuAdapter, ImportStatus, ShareLink, ShareMode, ShareRequest, ShareStatus, SharedHandle, SharedImage, SharedTextureFormat, SharedTextures, };` と `use kivact::launch::{AdapterMatch, GpuBackend, GpuPreference};`
   - `dev/tests/kivact-public-api/tests/e2e/drawable_export_gpu.rs`：`use kivact::gpu_interop::{self, GpuAdapter, GpuBackend, SharedTextures};` → `use kivact::gpu_interop::{self, GpuAdapter, SharedTextures};` と `use kivact::launch::GpuBackend;`
   - `docs/for-kivact-user/en/how-to-guide/gpu-texture-sharing.md:345`、jp `:278`：`use kivact::gpu_interop::{GpuBackend, GpuPower, GpuPreference, GpuStart};` → `use kivact::launch::{GpuBackend, GpuPower, GpuPreference, GpuStart};`。同 en:67 付近の「To choose, set `App::gpu_preference`」の文に変更は不要。
   - `gpu_on_demand.rs`（G4）、gpu-share、gpu-interop-producer は `kivact_types` を直接使うか本文の文字列だけなので変更なし。
7. W6 の P8-GPU への申し送り（D8 の P8-GPU の行の修正）：「`AdapterIds` を `crates/kivact-core/src/gpu_interop.rs:103` 付近の `pub use kivact_types::AdapterMatch;` の隣に足す」は、G8 の後は `AdapterMatch` が `launch` に移っているので、**`gpu_interop.rs` の `pub use kivact_types::GpuAdapter;` の隣**に足す（`AdapterIds` は `GpuAdapter` の field の型なので gpu_interop に置く）。
8. 完了時の確認（付属書の行に加える）：`cargo check -p kivact-test --all-targets`；`cargo doc -p kivact-core --no-deps`（警告 0）；`cargo xtask check --no-write --only canonical-path`；`cargo xtask check --no-write --only api-shape`；`KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 1800 -- cargo test -p kivact-docs --features gpu-interop --doc -- gpu_texture_sharing --test-threads=1`。

### 所有ファイルの変更

付属書の列を次に置き換える：`crates/kivact-core/src/{launch.rs,launch/settings.rs,app.rs,gpu_interop.rs}`、`crates/kivact/src/{lib.rs,launch.rs（新）}`、`crates/kivact-test/src/options.rs`、`dev/tests/kivact-public-api/tests/e2e/{gpu_interop.rs,drawable_export_gpu.rs}`、`docs/for-kivact-user/{en,jp}/how-to-guide/gpu-texture-sharing.md`（import 行だけ）。`crates/kivact-core/src/lib.rs` は変えない。

---

## FINAL-1（付属書 D5 6 章 FINAL-1、付録 P5-D F-1、W7）

### 判定：修正あり

1. **付属書の型は新しい state-fields error を出す**（実測）：`Availability::Enabled { pressed: bool, hovered: bool }` は variant の数える field が 2 つで `Availability::Enabled combines state fields: pressed, hovered`。二つは真に独立（activation key で押しつつ pointer は外、pointer が上にあって押していない、の両方が起きる）。variant を複数行にし、次の marker で閉じる（実測で error が消える。同じファイル内の同文は X5 で許される）：
   ```rust
   pub enum Availability {
       /// The target takes input.
       Enabled {
           // xtask-allow(state-fields): independent: a held activation key presses the target while the pointer is elsewhere, and the pointer rests over it unpressed, so every pair occurs
           /// A primary pointer or focused activation key is held down.
           pressed: bool,
           // xtask-allow(state-fields): independent: a held activation key presses the target while the pointer is elsewhere, and the pointer rests over it unpressed, so every pair occurs
           /// The pointer is over this exact target.
           hovered: bool,
       },
       /// A caller or attached behavior disables the target.
       Disabled,
       /// A caller or attached behavior hides the target (wins over Disabled:
       /// a hidden target is not drawn).
       Hidden,
   }
   ```
   `#[derive(Clone, Copy, Debug, PartialEq, Eq)]`（`Default` は derive しない）と手書きの `impl Default for Availability { fn default() -> Self { Self::Enabled { pressed: false, hovered: false } } }`。`PartState` の field の doc：`focus` は今の doc、`availability` は `/// Whether this target takes input; pressed and hovered exist only while it does.`。accessor は付録 P5-D F-1 のとおり（`is_focused`・`is_focus_visible` は残す）。
2. **付属書の事実の訂正**：D5 3.6 O7 の「`.hidden` の PartState 読み手は 0 件」は誤り。読み手は 9 箇所（`kivact-interaction` の `space_pan`・`zoom_pan`、creative の 7 箇所）。どれも `disabled || hidden` か、`(visible && !hidden, !disabled)` の組で読むので、Hidden 優先で `is_disabled()` が false になる差は hidden の間（描かれず hit もしない）にしか現れない。設計は変えない。
3. **書換えの規則（機械的）**：`<式>` は型が `PartState` の任意の式（`state.get()`、`input`、`s.input`、`state.input`、`look.input`、`input.get()`、`state.get().input`、`first.get()` など）。

   | 旧 | 新 |
   |---|---|
   | `<式>.pressed`（読み） | `<式>.is_pressed()` |
   | `<式>.hovered` | `<式>.is_hovered()` |
   | `<式>.disabled` | `<式>.is_disabled()` |
   | `<式>.hidden` | `<式>.is_hidden()` |
   | `PartState { focus: F, disabled: D, ..PartState::default() }`（`crates/kivact-ui/src/combobox/render.rs`） | `PartState { focus: F, availability: if D { Availability::Disabled } else { Availability::default() } }` |
   | `view/part.rs` の `state(PartState { .. })` | 付録 P5-D F-1 のコード |
   | `crates/kivact-ui-soft/src/indicator.rs` の `input.disabled \|= ..; if input.disabled { input.focus = .. }` | 付録 P5-F 3 章のコード（`actual.get().is_disabled() && !input.is_hidden()` で `Availability::Disabled`） |
   | `Availability` の import | kit・テストは `kivact::view::Availability`（core 内は `crate::view::part_data::Availability`） |

4. **読み手の全数（HEAD の行。W6 で行がずれるので関数名と式で探す）**。core の書き手（`view/part.rs`）を除き 43 ファイル・92 行。これに加えて `cargo check` が見ない docs の doctest 4 行。

   | file | 行（field） |
   |---|---|
   | `crates/kivact-interaction/src/space_pan/handle.rs` | 111(disabled,hidden) |
   | `crates/kivact-interaction/src/zoom_pan/handle.rs` | 284(disabled,hidden) 300(disabled) 315(disabled) |
   | `crates/kivact-ui-basic/src/badge.rs` | 97(hovered) |
   | `crates/kivact-ui-basic/src/button.rs` | 127(hovered,disabled) |
   | `crates/kivact-ui-basic/src/input_group.rs` | 103(disabled) |
   | `crates/kivact-ui-basic/src/list_popup.rs` | 36(hovered,disabled) |
   | `crates/kivact-ui-basic/src/recipes_combobox.rs` | 30(disabled) |
   | `crates/kivact-ui-basic/src/recipes_controls.rs` | 481(hovered,disabled) |
   | `crates/kivact-ui-basic/src/recipes_dialog.rs` | 229(pressed) |
   | `crates/kivact-ui-basic/src/recipes_menu.rs` | 114(disabled) |
   | `crates/kivact-ui-basic/src/sidebar/panel.rs` | 219(hovered) |
   | `crates/kivact-ui-creative/src/bezier/view/implementation.rs` | 250(disabled,hidden) |
   | `crates/kivact-ui-creative/src/color_wheel/view.rs` | 143(disabled,hidden) 368(disabled,hidden) |
   | `crates/kivact-ui-creative/src/editors/arrangement/view.rs` | 322(hidden) 323(disabled) |
   | `crates/kivact-ui-creative/src/editors/graph/view.rs` | 303(hidden) 304(disabled) |
   | `crates/kivact-ui-creative/src/editors/node/view.rs` | 205(disabled,hidden) |
   | `crates/kivact-ui-creative/src/editors/time/view.rs` | 243(hidden) 244(disabled) |
   | `crates/kivact-ui-creative/src/gradient/view.rs` | 125(disabled,hidden) |
   | `crates/kivact-ui-creative/src/range/implementation.rs` | 351(pressed) 398(pressed) |
   | `crates/kivact-ui-soft/src/breadcrumbs.rs` | 69(hovered) |
   | `crates/kivact-ui-soft/src/card.rs` | 203(disabled) 205(pressed) 207(hovered) |
   | `crates/kivact-ui-soft/src/indicator.rs` | 118-119（書き手、規則表）152(disabled) 187(disabled) |
   | `crates/kivact-ui-soft/src/link.rs` | 313(disabled,hovered) 316(disabled) |
   | `crates/kivact-ui-soft/src/nav_heading.rs` | 516(disabled,hovered) 637(hovered) |
   | `crates/kivact-ui-soft/src/recipes_collections.rs` | 208 281 383 393 444 452(disabled) 357 411(pressed) 599(hovered) |
   | `crates/kivact-ui-soft/src/recipes_controls.rs` | 129(hovered) |
   | `crates/kivact-ui-soft/src/recipes_controls_choice.rs` | 146(disabled,pressed) 156(hovered,disabled) 160(disabled) 245(hovered,disabled) 257(disabled) |
   | `crates/kivact-ui-soft/src/recipes_menu.rs` | 154(disabled) |
   | `crates/kivact-ui-soft/src/recipes_navigation.rs` | 158(disabled,hovered) |
   | `crates/kivact-ui-soft/src/recipes_popup_command.rs` | 126(disabled) |
   | `crates/kivact-ui-soft/src/recipes_select.rs` | 113(disabled) 123(disabled) |
   | `crates/kivact-ui-soft/src/segmented_control.rs` | 80(disabled) |
   | `crates/kivact-ui-soft/src/side_navigation/shell.rs` | 571(hovered) |
   | `crates/kivact-ui-soft/src/soft_list.rs` | 63(pressed) 65(hovered) |
   | `crates/kivact-ui-soft/src/thumbnail.rs` | 360(pressed) 362(hovered) |
   | `crates/kivact-ui/src/combobox/render.rs` | 74（書き手、規則表） |
   | `crates/kivact-ui/src/look.rs` | 120(hovered) |
   | `crates/kivact-ui/src/look_palette.rs` | 85(disabled,hovered) |
   | `crates/kivact-ui/src/switch_drag/handle.rs` | 45(disabled) |
   | `crates/kivact-ui/src/tree.rs` | 278(disabled) |
   | `dev/tests/kivact-public-api/tests/e2e/parts.rs` | 41 43 65 67 68 70 76 94 165(hovered) 75 79 92 143 148 159 163 191 192 194(pressed) |
   | `dev/tests/kivact-public-api/tests/e2e/pointer.rs` | 379 383 394(pressed) |
   | `examples/kivact-ui-basic/src/shell.rs` | 144(hovered) |
   | `docs/for-kivact-user/en/how-to-guide/custom-controls.md`（doctest） | 71(pressed) 231(disabled) |
   | `docs/for-kivact-user/jp/how-to-guide/custom-controls.md`（doctest） | 60(pressed) 195(disabled) |

   E2E の `parts.rs`・`pointer.rs` は式の形だけの変更で、期待値は同じ（緩めない）。
5. **compile error 駆動のままでよいか**：よいが、上の表を正とし、`cargo check` は確認に使う。理由：エラーは依存の順（core → interaction → ui → basic/soft/creative → examples・dev/tests）に crate ごとに止まり、一度に全部は出ない（実測）。docs の 4 行は compile error に出ない。W6 の unit（UI-1・UI-3 など）が同じファイルに `input.hovered` 等の新しい読みを足していれば、それも同じ規則で直す（`cargo check` が示す）。
6. 完了時の確認（付属書の行に加える）：`cargo xtask check --no-write --only state-fields --crate kivact-core` で `part_data.rs` の行が出ないこと；`KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-docs --doc -- custom_controls --test-threads=1`；`KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- parts:: pointer::`。

### 所有ファイルの変更

付属書の列（compile error が示す全ての file）に、`docs/for-kivact-user/{en,jp}/how-to-guide/custom-controls.md` を明示で足す。

---

## G9（付属書 D6 9 章 C-13・8 章 2、W7）

### 判定：修正あり（重大 1 件）

**DEC-18 の `content(factory)` は既存の method と衝突する**：`MenuApi::content(self) -> Part<PopupState>`（`menu/handle.rs:461`）と `CommandPaletteApi::content(self) -> Part<PopupState>`（`palette/handle.rs:420`）が既にあり、Rust は overload できない。`content()` は Popup・Dialog・Select・Tooltip・Disclosure・Combobox の全 API で「content の Part」を意味するので、そちらは変えられない。content を mount する method は **`while_open(factory)`** にする（意味：開いている間だけ factory の view を mount する。`show`/`hide`・`content` と衝突しない。React の `{open && <X/>}`、Solid の `<Show when>` に当たる語）。DEC-18 の意図（`show` を命令に空ける）はそのまま。

### 修正（作業指示書 10 章の「G9 の改名表」の差し替え）

規則：引数名だけの変更は呼出し元に影響しない。関数の定義の引数名と、本体でその引数を使う箇所（`let options = items.into_value();` のように**引数の使用箇所だけ**新しい名前にし、本体の local の名前は変えない。field の省略記法 `Foo { selection }` は `Foo { selection: selected }` にする）、直上の rustdoc で旧名を `` ` `` で囲んで書いた箇所を直す。method の改名は定義を変えて compile error（E0599／E0061）が示す全ての呼出し元を直す。docs（doctest と本文）は compile error に出ないので、各行の「docs」の欄を必ず直す。

| 順 | 対象（HEAD の署名） | 変更 | docs・rustdoc（compile で出ない） |
|---|---|---|---|
| 1 | `MenuApi::show<V>(self, factory: impl IntoViewFn<V>) -> View`、`CommandPaletteApi::show<V>(self, factory) -> View` | `while_open(factory)`。**2 より先** | `menu/handle.rs` の rustdoc（:30 の `[`show`](Self::show)`、:92 の例 `file.show(` → `file.while_open(`）、`palette/handle.rs`（:11 `[`CommandPaletteApi::show`]`、:208 `[`show`](Self::show)`、:255 の例、:494 の `[`MenuApi::show`](crate::MenuApi::show)`）、`MenuApi::while_open` の doc の「not with `show(menu.is_open(), ..)`」の文は残す。`docs/for-kivact-user/en/how-to-guide/custom-controls.md:403-404` の「The same pairing of `trigger()`, `content()` and `show` works for … and `use_menu`; only the Parts differ.」→「The same pairing of `trigger()`, `content()` and `show` works for `use_popup`, `use_disclosure`, `use_tooltip` and `use_dialog`; a menu mounts its content with `MenuApi::while_open` instead of `show`. Only the Parts differ.」（jp の対応文も同じ内容に） |
| 2 | `MenuApi::open(self)`／`close(self)`、`ComboboxApi::open(self)`／`close(self)` | `show(self)`／`hide(self)`。`MenuApi::top_level("open")`・`("close")` の文字列も `"show"`・`"hide"` | 呼出し元は compile で全部出る（`combobox/render.rs`、`menubar.rs`、`combobox/handle.rs` 内部など） |
| 3 | `ComboboxApi::open_sheet(self)`（追加：同じ型の命令） | `show_sheet(self)` | — |
| 4 | `InlineEditApi::open(self, key: K)`（**`close` は無い**。`is_open(&key)` は残す） | `show(self, key)` | `docs/for-kivact-user/en/how-to-guide/forms.md:322` の `rename.open(id)` → `rename.show(id)`、同ファイルの本文の `` `open(id)` ``（302 行付近）、jp `forms.md:255` 本文の `` `open(id)` `` と `:273` のコード |
| 5 | `LightboxHandle::close(self)`、`AppShellMobile::close(self)`（追加：同じ型が既に `show` を持つ） | `hide(self)` | `crates/kivact-ui-soft/README.md:328` の改名表の行に `` `LightboxHandle::close()`, `AppShellMobile::close()` `` → `` `hide()` `` を足す |
| 6 | `kivact-ui` `combobox(label, options: impl IntoValue<Vec<ListItem<K>>>, selection: impl IntoSelection<K>)` | 引数名 `items`、`selected` | `crates/kivact-ui-basic/README.md:193`、`crates/kivact-ui-soft/README.md:355`、`docs/for-kivact-ui-user/{en,jp}/soft/how-to-guide/forms.md:38` の署名の書き方を `combobox(label, items, selected)` に |
| 7 | `kivact-ui` `tabs(label, options: impl IntoValue<Vec<ListItem<K>>>, binding: impl IntoBinding<K>)` | `items`、`value` | `docs/for-kivact-ui-user/{en,jp}/basic/how-to-guide/navigation.md:74` の `tabs(label, options, current)` → `tabs(label, items, value)` |
| 8 | `kivact-ui` `toggle_group(label, options, selection: impl IntoSelection<K>)` | `items`、`selected` | `crates/kivact-ui-basic/README.md:220`、`crates/kivact-ui-soft/README.md:347`、`docs/for-kivact-ui-user/{en,jp}/soft/how-to-guide/forms.md:40`、`docs/for-kivact-ui-user/{en,jp}/creative/how-to-guide/theme-and-icons.md:11` |
| 9 | `kivact-ui` `disclosure_group(selection: impl IntoSelection<K>)` | `selected` | `crates/kivact-ui-soft/README.md:54,313` |
| 10 | `kivact-ui` `tree(label, items)` と setter `Tree::selection(self, impl IntoSelection<K>)`（引数名は既に `items`） | `tree(label, items, selected: impl IntoSelection<K>)`、setter を削除。struct の初期値 `kivact_interaction::IntoSelection::into_selection(signal(None))` を `selected.into_selection()` に。呼出し元：`.selection(x)` があれば消して第 3 引数に `x`、無ければ第 3 引数に `signal(None)`（`Signal<Option<K>>`。K は items から推論される） | code：`dev/tests/.../e2e/{apg_tree_view.rs,ui_components.rs}`、`crates/kivact-ui-game/tests/consumer/interactions.rs`、`crates/kivact-ui-soft/tests/consumer/{collections.rs,tree_reveal.rs}`、`examples/kivact-ui-game/src/game/system.rs:731`、`examples/kivact-ui-soft/src/docs/collections.rs:277`（と :294 の表示文字列 `"kivact::ui::tree(\"Files\", files, selected).expanded(expanded).reveal(2.0, transition)"`）。docs（doctest）：`docs/for-kivact-ui-user/en/game/how-to-guide/item-lists-and-selection.md:159`、jp `:141`。docs 本文：同 en:141・jp:123 と `soft/how-to-guide/data-display.md` en/jp:110 の `tree(label, nodes)` → `tree(label, items, selected)` と `.selection(..)` の言及の削除、`docs/for-kivact-user/{en,jp}/how-to-guide/collections.md:43` の `ui::tree(label, nodes).expanded(expanded).selection(selected)` → `ui::tree(label, items, selected).expanded(expanded)`、`crates/kivact-ui-game/README.md:86`、`crates/kivact-ui-soft/README.md:354` |
| 11 | `kivact-interaction` の hook（実際の一覧。下の 10 個だけ） | 下表 | `docs/for-kivact-user/en/how-to-guide/navigation.md:102` と jp `:82` の `use_menu_with_open(entries, open_binding)` → `use_menu_with_open(items, open_binding)` |
| 12 | `kivact-ui-soft` `tab_list(name: impl IntoText, binding: impl IntoBinding<K>)` | `tab_list(label, value)`（`name` も他の部品の `label` に揃える） | — |
| 13 | `kivact-ui-creative` `gradient_editor(stops, selection: impl IntoBinding<Option<usize>>)` | `selected` | `crates/kivact-ui-creative/README.md:277`、`docs/for-kivact-ui-user/{en,jp}/creative/how-to-guide/color-curves-and-gradients.md:131` |
| 14 | `kivact-ui-creative` `document_tabs(label, items, selected: impl IntoBinding<K>)`（追加：必ず一つを選ぶ tabs） | `value` | — |

順 11 の hook（`use_*` は 39 個。下の 10 個以外は引数名が ARG に合うので変えない）：

| hook（HEAD） | 変更後 |
|---|---|
| `use_combobox(options, selection: impl IntoSelection<K>)` | `use_combobox(items, selected)` |
| `use_disclosure_group(selection: impl IntoSelection<K>)` | `use_disclosure_group(selected)` |
| `use_list(items, selection: impl IntoSelection<K>, policy)` | `use_list(items, selected, policy)` |
| `use_menu(entries)` | `use_menu(items)`（ARG-2） |
| `use_menu_with_open(entries, open)` | `use_menu_with_open(items, open)` |
| `use_select(options, value: impl IntoBinding<Option<K>>)` | `use_select(items, selected)`（単一だけの選択は `selected`、ARG-3） |
| `use_tabs(options, selected: impl IntoBinding<K>)` | `use_tabs(items, value)`（必ず一つ、ARG-3） |
| `use_tree(nodes, expanded, selection)` | `use_tree(items, expanded, selected)`（ARG-2・3） |
| `use_data_table(rows, key, columns, selection, focus)` | `use_data_table(rows, key, columns, selected, focus)` |
| `use_calendar_grid(month, selection: CalendarSelection, policy)` | `use_calendar_grid(month, selected, policy)`（kivact-ui の `calendar(selected)` に揃える） |

変えないもの（判断を残さないため明記）：`data_table(rows, key, columns).selection(..)`（選択は任意で、付けると check box の列が出る機能の切替え）、`table_selection_toolbar(selection)`、`command_palette(commands, open)`／`use_command_palette(commands, open)`、`node_editor(nodes, links, state)`（node editor の領域の語）、`DisclosureApi::close`・`NavigationMenuApi::close`・`PopupApi::close(reason)`・`ToolWindowApi::close`（同じ型に `show` が無い単独の命令）、`toggle`、`is_open`、kivact-ui の setter `.open(binding)`（ARG-4）。

完了時の確認（作業指示書の行を置き換える）：`cargo check --workspace --all-targets --all-features`；`cargo test -p kivact-interaction --doc`（rustdoc の例は `no_run` で window を開かない）；`cargo doc -p kivact-interaction --no-deps`（警告 0。intra-doc link の切れを見る）；`cargo xtask check --no-write --only api-shape`；`KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 1800 -- cargo test -p kivact-public-api-tests --test e2e -- apg_combobox apg_tabs apg_menu interaction_menu interaction_palette ui_toolbar apg_tree ui_field ui_components`；`KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 3600 -- cargo test -p kivact-docs --all-features --doc -- --test-threads=1`。

### 所有ファイルの変更

作業指示書どおり（compile error と grep が示す全ての file）に、上表の「docs・rustdoc」の欄の file を明示で足す。

---

## 全体

### wave・lane の組み直し

- W5：lane A＝G5（C-6 を含む）、lane B＝G7、lane C＝G6 → G8。所有ファイルの重なりは無い（実測）：A は `crates/kivact-macros/**`・`e2e/component_macro.rs`・`custom-components.md`、B は `calendar/view.rs`・`menu/handle.rs`・`api_shape.rs`、C は docs（`custom-components.md` を除く）・prelude・core の doc・`kivact-devtools/src/app.rs`・`e2e/windows.rs`・`kivact-test/src/{lib.rs,render.rs,options.rs}`・`crates/kivact/src/{lib.rs,launch.rs}`・`e2e/{gpu_interop.rs,drawable_export_gpu.rs}`。G6 と G8 は同じ lane で順に行うので `app.rs`・`launch/settings.rs`・`gpu-texture-sharing.md` の共有は問題ない。
- W7：変更なし（FINAL-1 → G9 → FINAL-2）。FINAL-1 の marker（`Availability::Enabled`）は FINAL-2 の X5（他ファイルとの同文）に当たらない文言にしてある。
- W6 の P8-GPU：本書 G8 の 7 のとおり `AdapterIds` の再公開位置を直す（W6 の監査担当へ申し送り）。

### ユーザー（人間）にしか決められない項目

- 実質なし。次の 1 件は承認済みの決定（DEC-18）の名前が実装できないことによる置換えで、推奨で進めてよい：**`MenuApi`・`CommandPaletteApi` の content を mount する method の名前**。`content` は既存の `content() -> Part` と衝突するので不可。推奨 `while_open(factory)`。代案 `mount(factory)`（ただし Kivact の文書では mount を内部の動作の語として使う）。
- G8 の facade の `kivact::launch` は DEC-17 の path をそのまま実現する形（GPU の 5 型だけを持つ）で、判断は残らない。

### 公開 API の統一で V2 の範囲で直すもの（重要なものだけ。上の各 unit に組み込み済み）

- ARG-2/3/6 の漏れ：`use_tabs` の `selected`→`value`、`use_select` の `value`→`selected`、`use_menu`・`use_tree` の `entries`・`nodes`→`items`、`use_data_table`・`use_calendar_grid` の `selection`、creative `document_tabs` の `selected`→`value`、soft `tab_list` の `name`→`label`、`ComboboxApi::open_sheet`→`show_sheet`、同じ型に `show` がある `LightboxHandle`・`AppShellMobile` の `close`→`hide`（G9 に統合）。
- ARG-1：`impl Fn() -> R` の `on_*` と `impl Fn()` の builder method は G7 後に workspace で 0 件（api-shape が検出し続ける）。

### V2 の範囲外として記録だけするもの（直さない）

- `kivact::test` の `all_by_*` は Testing Library の `queryAllBy*`／`getAllBy*` と名前の形が違う（`get_by`・`query_by`・`find_by` は一致）。改名は V2 の後。
- `creative::transport(binding)` の引数名 `binding`。
- 試作中に `cargo check -p kivact --no-default-features` で `crates/kivact-engine/src/paint/state.rs:326` の `fields size and content are never read` の warning を見た（本書の unit は engine を変えていないので HEAD 由来。CP2 の clippy は `--all-features` なので出ない可能性がある。該当 lane の監査担当へ）。
