# 付属書 G：文書と最終の整合（W11）

本付属書は、改修の最後の wave（W11）で行う文書の書換えと、全体の最終確認の unit を定める。対象は利用者向けの文書（英日）、README、例・雛形の文書、開発者向けの文書、移行ガイド、`kivact-docs` の compile の再開、AC-4（削除名）の最終確認である。コードの API は W1〜W10 の付属書（T・R・C・E・S・U・K・A）がすでに最終形にしている前提で書く。

## 1. 範囲と前提

### 1.1 範囲

| 対象 | unit |
|---|---|
| `kivact-docs` の compile の再開、guide を絞って compile する `KIVACT_DOCS_ONLY` | G01 |
| `docs/for-kivact-user/{en,jp}/getting-started/*`（チュートリアル） | G02 |
| `docs/for-kivact-user/{en,jp}/how-to-guide/*`（37 ファイル × 2） | G03〜G08 |
| 新設 `docs/for-kivact-user/{en,jp}/how-to-guide/migrating-to-rework.md`、`docs/for-kivact-user/{en,jp}/README.md`、`how-to-guide/README.md`（索引） | G09 |
| `docs/for-kivact-ui-user/{en,jp}/**`（kit のガイド）と plot のガイドの移動 | G10、G11 |
| `README.md`、`README_JP.md`、`templates/basic/**` の文書、`examples/kivact/**` の README と doc comment、`crates/*/README.md` | G12 |
| `docs/for-kivact-developer/architecture.md` | G13 |
| `docs/for-kivact-developer/{decisions.md, strategy.md, manual-verification.md}` | G14 |
| `CONTRIBUTING.md` の残り、AC-4 の最終確認、報告だけの検査の一覧の削除、`kivact-docs` を gate に戻す、bench の probe の最終確認 | G15 |

担当する所見：API-07（文書の誤り）、API-08（非同期の表示は `async_view` だけ）、API-09（例と雛形の文書の部分）、KIT-16（kit の docs。日本語は全ページ残す）、TL-03（文書の部分：日本語の Rust の code block を英語と同一にする。docs-pairs）、TL-09（文書の部分：Python の節を消す）、AC-5、設計 3 章の profile の案内（USR-01、USR-13）、設計 5.12（AC-4）。

### 1.2 前の付属書が済ませていること（前提）

この付属書は、次のことが W1〜W10 で済んでいる前提で書く。前提が成り立たないと分かったら、その unit を STOP して Orchestrator に報告する（自分で API を作らない）。

1. **公開 API は設計 5 章のとおり実装済み**（付属書 R・C・S・U・K）。名前の正本は各付属書の 4 章の対応表である。本付属書のコードがそれと違う名前を使っていて compile できないときは、**その付属書の 4 章の名前に直す**（推測で別の名前を作らない）。4 章にも無いときは STOP。
2. **付属書 T（W1）**：
   - `dev/tests/docs/build.rs` は英語の木（`for-kivact-user/en`、`for-kivact-ui-user/en`）と `README.md` だけを doctest にする（TL-03）。
   - xtask の検査 `docs-pairs` がある：対になる日本語のファイル（`/en/` → `/jp/`、`README.md` → `README_JP.md`）の「info 文字列が `rust` で始まる fenced code block の列」が英語と文字列として同一であることを確かめる。
   - 6.3 節の新しい検査のうち W11 まで error にできないものは、`dev/xtask/policy/workspace.toml` の報告だけの一覧 `[rework.pending]` に入っている（T 4.4）。`state` と `docs-pairs` の行は T22（W1c）が消している。残りの行（`interaction-view`、`ui-machine`、`unit-machine`、`hand-element`、`element-shadow`）は U・C・K が通し、G15 が表と pending のコードを消す。
   - 公開 API の E2E は `kivact-public-api-tests` の `e2e_core`・`e2e_ui`・`e2e_native` と、新しい package `kivact-kit-tests`（`dev/tests/kivact-kit-tests`）の `e2e_kits` に分かれている（T15。workspace は 54 package）。`python/` と xtask の `devtools_codegen*` は消えている。`kivact-devtools-client` は `kivact-devtools-protocol` の feature `client` に統合済み。`architecture/` は git から外れ、図は `cargo xtask diagrams`。`state-fields` は `state` に置き換わっている。
   - `CONTRIBUTING.md` の「テスト」と「merge 前の検査」は付属書 T が書き直している（tier a/b/c、e2e の binary、window-lock、state の検査、Python と `--report-audit` と `git diff --quiet -- architecture` の削除）。G15 はそれを確かめるだけで、残っていれば直す。
   - 例と雛形の `#![warn(redundant_imports)]` は消えている。
3. **付属書 C（W5 の鎖）**：`examples/kivact/*/src/main.rs` と `templates/basic/src/main.rs` の**コード**は新しい API に移行済み。本付属書は doc comment と README だけを扱う。`kivact-docs` は W5 から gate の `--exclude kivact-docs` で compile の対象外にされている（root `Cargo.toml` の `members` は変えていない。作業指示書の全体の指示 4、INC-09）。
4. **付属書 K（W9〜W10）**：5 つの kit と `kivact-plot`、`examples/catalog-shell` は最終の API に移行済み。kit の旧名→新名の表は付属書 K の 4 章にある。
5. **付属書 S**：中核の theme 変数は次の名前とする（付属書 S の 4 章が正本。違えば S に合わせる）。

| 旧 | 新（値） | 新（変数の static） |
|---|---|---|
| `ColorToken::Surface` / `Palette.surface` | `color::SURFACE` | `color::SURFACE_VAR` |
| `ColorToken::SurfaceAlt` / `surface_alt` | `color::SURFACE_ALT` | `color::SURFACE_ALT_VAR` |
| `ColorToken::Text` / `text` | `color::TEXT` | `color::TEXT_VAR` |
| `ColorToken::TextMuted` / `text_muted` | `color::TEXT_MUTED` | `color::TEXT_MUTED_VAR` |
| `ColorToken::Accent` / `accent` | `color::ACCENT` | `color::ACCENT_VAR` |
| `ColorToken::AccentText` / `accent_text` | `color::ACCENT_TEXT` | `color::ACCENT_TEXT_VAR` |
| `ColorToken::Border` / `border` | `color::BORDER` | `color::BORDER_VAR` |
| `ColorToken::Focus` / `focus` | `color::FOCUS` | `color::FOCUS_VAR` |
| `ColorToken::Danger` / `danger` | `color::DANGER` | `color::DANGER_VAR` |
| `ColorToken::Success` / `success` | `color::SUCCESS` | `color::SUCCESS_VAR` |
| `ColorToken::Warning` / `warning` | `color::WARNING` | `color::WARNING_VAR` |
| `ColorToken::Overlay` / `overlay` | `color::OVERLAY` | `color::OVERLAY_VAR` |
| `ColorToken::Shadow` / `shadow` | `color::SHADOW` | `color::SHADOW_VAR` |
| `ColorToken::Selection` / `selection` | `color::SELECTION` | `color::SELECTION_VAR` |
| `ColorToken::Hover` / `hover` | `color::HOVER` | `color::HOVER_VAR` |
| `ColorToken::Pressed` / `pressed` | `color::PRESSED` | `color::PRESSED_VAR` |
| `ColorToken::Raised` / `raised` | `color::RAISED` | `color::RAISED_VAR` |
| `theme.space().xs` … `.xl` | `space::XS`、`SM`、`MD`、`LG`、`XL` | `space::MD_VAR` など |
| `theme.radius().sm` … `.lg` | `radius::SM`、`MD`、`LG` | `radius::MD_VAR` など |
| `theme.text().size`、`size_small`、`size_large` | `font::SIZE`、`SIZE_SMALL`、`SIZE_LARGE` | `font::SIZE_VAR` など |

6. **付属書 R**：変換 trait `IntoValue`・`IntoText`・`IntoHandler` は `kivact::state` から import できる。`IntoSelection`・`Selection` は `kivact::interaction`。`Signal::update` と `Var::update` は値を返さない（DEC-15）。handler は二つの形だけ（DEC-16）：payload の無い handler（`on_press` など、`Callback<()>` props）は `impl IntoHandler<(), M>`（closure `Fn() -> R` と `Callback<()>`）、payload のある handler（`on_key`、`on_activate`、`on_commit`、`on_edit`、`Callback<E>` props）は `impl Fn(E) -> R + 'static`。共有した `Callback<E>` を payload のある handler に渡すときは `move |e| cb.call(e)` と書く。
7. **`#[component]` は型引数を持てる**（設計 5.1 の規則 9）。lifetime 引数と const 引数は受けない。

### 1.3 後の段階に渡すもの

- AC-5 を満たす文書一式（利用者向けの英日 docs、README、例、雛形、移行ガイド、開発者向けの 3 文書）。
- `kivact-docs` が gate（tier c）に戻っていること、`[rework.pending]` が無く、全検査が error であること。
- AC-4 の検査の「出力の扱い」の規則（G15 の手順 4。作業指示書 2 章の AC-4 のコマンドに写す）。
- `REWORK_DESIGN.md`、`REWORK_WORK_ORDER.md`、`rework/` は**消さない**。作業指示書の指示どおり、PR が merge されるまで残す（G15 で確かめるだけ）。

### 1.4 文書の unit に共通の規則

1. 英語のファイルを先に直し、doctest で compile を確かめてから、日本語のファイルを直す。
2. 日本語のファイルの Rust の code block（info 文字列が `rust` で始まる fenced block）は、英語の同じ順番の block と**一字一句同じ**にする（コメント・文字列も英語のまま。docs-pairs）。手で写さず、3.0.2 の script で写す。日本語の本文（説明の文章）は、英語の新しい本文を日本語に訳して書く。既存の日本語の文体（です・ます、用語）に合わせる。
3. 文書の中の「部品」は 1 つの書き方だけを教える：`#[component]`。3.0.4 の規則で変換する。
4. 旧名（設計 5.12、3.0.3 の表の左の列）を、移行ガイド `migrating-to-rework.md` 以外の文書に残さない。
5. 内部の作業の言葉（付属書、unit、wave、AC、所見 ID、`REWORK_*`）を利用者向けの文書・README・rustdoc に書かない。非公開の利用者のリポジトリの名前や内容を書かない。
6. 公開リポジトリの文書なので、`[profile.*.package.<Kivact の内部 crate 名>]` を利用者に勧めない（3.0.3 の冒頭、設計 3 章）。

## 2. unit の一覧

W11 は SCHEDULE（作業指示書 4 章）のとおり三つの段に分ける（INC-30）：W11a（G01、G13、G14。付属書 K の K44 も同じ段）、W11b（G01 の後に始める利用者向けの文書の unit）、W11c（G15、単独。W11a・W11b の全 unit の後。全ファイルの残りを直すため、前の段の unit と所有が重なる）。

| ID | wave | lane | 前提の unit | 所有するファイル・directory | 所見 | 規模 |
|---|---|---|---|---|---|---|
| G01 | W11a | G1 | —（W1〜W10 の完了） | `dev/tests/docs/**`（root `Cargo.toml` は変えない） | TL-03、AC-5 | S |
| G02 | W11b | G1 | G01（W11a） | `docs/for-kivact-user/{en,jp}/getting-started/**` | API-08、AC-5 | M |
| G09 | W11b | G1 | G02 | `docs/for-kivact-user/{en,jp}/how-to-guide/migrating-to-rework.md`（新規）、`docs/for-kivact-user/{en,jp}/README.md`、`docs/for-kivact-user/{en,jp}/how-to-guide/README.md` | AC-5、USR-01、USR-13、TL-09 | M |
| G12 | W11b | G1 | G09 | `README.md`、`README_JP.md`、`templates/basic/README.md`、`templates/basic/src/main.rs`（doc comment だけ）、`examples/kivact/README.md`、`examples/kivact/*/README.md`、`examples/kivact/*/src/main.rs`（doc comment だけ）、`examples/README.md`、`crates/*/README.md` | API-07、API-09、KIT-16、AC-5 | M |
| G03 | W11b | G2 | G01（W11a） | `docs/for-kivact-user/{en,jp}/how-to-guide/{coming-from-react-or-solid,custom-components}.md` | API-01（文書）、API-07、AC-5 | M |
| G06 | W11b | G2 | G03 | `docs/for-kivact-user/{en,jp}/how-to-guide/{theming,i18n,animation,timelines-and-scroll,3d-and-depth,effects,shaders,canvas,drawables}.md` | AC-5 | L（機械的） |
| G08 | W11b | G2 | G06 | `docs/for-kivact-user/{en,jp}/how-to-guide/custom-controls.md` | AC-5 | M |
| G13 | W11a | G3 | — | `docs/for-kivact-developer/architecture.md` | 設計 4・6 章、AC-5 | M |
| G04 | W11b | G3 | G01、G13（W11a） | `docs/for-kivact-user/{en,jp}/how-to-guide/{forms,lists-and-tables,overlays,windows,layout,images-and-fonts}.md` | API-07、AC-5 | L（機械的） |
| G05 | W11b | G3 | G04 | `docs/for-kivact-user/{en,jp}/how-to-guide/{collections,navigation,keyed-suggestions,disclosure-and-previews,feedback,indicators,metrics-and-description,chat-and-code,slides-and-files}.md` | AC-5 | M |
| G14 | W11a | G4 | — | `docs/for-kivact-developer/{decisions.md,strategy.md,manual-verification.md}` | 設計 7・8 章、AC-5 | M |
| G07 | W11b | G4 | G01、G14（W11a） | `docs/for-kivact-user/{en,jp}/how-to-guide/{async-and-tasks,http-json,platform-services,testing,diagnostics,devtools,embed,foreign-windows,gpu-texture-sharing}.md` | API-08、TL-09、AC-5 | L |
| G10 | W11b | G4 | G07 | `docs/for-kivact-ui-user/{en,jp}/{basic,pop,game}/**`、新規 `docs/for-kivact-user/{en,jp}/how-to-guide/plot.md`（basic の plot.md を移す） | KIT-15（文書）、KIT-16、AC-5 | L |
| G11 | W11b | G4 | G10 | `docs/for-kivact-ui-user/{en,jp}/{creative,soft}/**`、`docs/for-kivact-ui-user/{en,jp}/README.md` | KIT-16、AC-5 | L |
| G15 | W11c | G1 | G01〜G14 全部 | `CONTRIBUTING.md`、`dev/README.md`、`dev/xtask/policy/workspace.toml` の `[rework.pending]`、`dev/xtask/src/{pending,local_checks,check_cli,main,policy}.rs` の pending の部分（T 4.4）、AC-4 の検査で見つかった残りの箇所（`crates/`、`examples/`、`templates/`、`docs/`、`dev/` の文書・コメント・文字列だけ） | AC-4、AC-5、TL-03 | M |

lane の順（同じ lane の中は上から順に一つずつ）：

- W11a：G1 = G01、G3 = G13、G4 = G14（K44 は lane K）
- W11b：G1 = G02 → G09 → G12、G2 = G03 → G06 → G08、G3 = G04 → G05、G4 = G07 → G10 → G11
- W11c：G1 = G15

同じ段の二つの unit が同じファイルを所有することはない（5 章で確かめた）。

## 3. unit の手順

### 3.0 文書の unit が共通に使う道具と表

#### 3.0.1 文書の unit の確認コマンド（tier a）

G01 が `KIVACT_DOCS_ONLY` を足した後は、自分の直したファイルだけを doctest にできる。値は `docs/` からの相対の path（ファイルか directory）をコンマで区切ったもの。root の README は `README.md` と書く。

```bash
# 例：G02（チュートリアル）
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_DOCS_ONLY=for-kivact-user/en/getting-started KIVACT_TEST_MIRROR=off \
  cargo test -p kivact-docs --doc --all-features -- --test-threads=1
KIVACT_DOCS_ONLY=for-kivact-user/en/getting-started cargo check -p kivact-docs --all-targets --all-features
cargo xtask check --no-write --only docs-pairs
```

- doctest が 1 つでも失敗したら、その block を直してから次へ進む。期待値（test の assert）を緩めない。
- `docs-pairs` は W1c から error である（付属書 T の T22 が pending から外す）。W11b では別の lane が同時に他の対を直しているので、出力に自分の所有するファイルの名前が出ないことを確かめる。
- 試験を実行する `cargo test` は、作業指示書の全体の指示（INC-15）のとおり必ず `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test ...` の形で実行する（環境変数は `env` の後に並べる）。`kivact-docs` は `--all-features` だけで実行し、`--no-default-features` では実行しない（docs は feature が無いと compile できない。T 4.1、INC-31）。

#### 3.0.2 日本語のファイルへ Rust の code block を写す script

repo の外（`/tmp`）に保存して使う。repo には commit しない。

```bash
cat > /tmp/g-sync-rust-blocks.pl <<'PERL'
#!/usr/bin/perl
# Usage: perl g-sync-rust-blocks.pl <en.md> <jp.md>
# Replaces the n-th fenced block whose info string starts with `rust` in the
# jp file by the n-th such block of the en file. Fails when the counts differ.
use strict;
use warnings;

sub read_file { my ($p) = @_; local $/; open my $f, '<:raw', $p or die "$p: $!"; my $t = <$f>; close $f; return $t; }
sub blocks { my ($t) = @_; my @b; while ($t =~ /^(```rust[^\n]*\n.*?^```[ \t]*\r?$)/msg) { push @b, $1; } return @b; }

my ($en, $jp) = @ARGV;
my $et = read_file($en);
my $jt = read_file($jp);
my @eb = blocks($et);
my @jb = blocks($jt);
die "block count differs: $en has " . scalar(@eb) . ", $jp has " . scalar(@jb) . "\n" if @eb != @jb;
my $i = 0;
$jt =~ s/^(```rust[^\n]*\n.*?^```[ \t]*\r?$)/$eb[$i++]/msge;
open my $o, '>:raw', $jp or die "$jp: $!";
print $o $jt;
close $o;
PERL
```

使い方（一つのファイル）：`perl /tmp/g-sync-rust-blocks.pl docs/for-kivact-user/en/how-to-guide/forms.md docs/for-kivact-user/jp/how-to-guide/forms.md`。

「block count differs」で止まったら、日本語のファイルの節の構成を英語の新しい構成に合わせて（節を足す・消す）から、もう一度実行する。

#### 3.0.3 旧 API → 新 API の置換表

この表は二つの用途を持つ：(1) 文書の unit が旧 API を書き換える規則、(2) G09 が書く移行ガイドの本文（英語のまま使う）。左の列の名前を `rg -n -w '<名前>'` で探し、右の列に書き換える。

````markdown
## Components

| Before | After |
|---|---|
| `fn header() -> impl Element { .. }`, `fn panel() -> impl IntoView { scope(move \|\| ..) }` | `#[component] fn header() -> impl Element { .. }` |
| `#[component] fn card(..) -> impl IntoView` (callers could not add `.style(..)`) | `#[component] fn card(..) -> impl Element`; the caller's `.style(..)`, `.name(..)`, `.test_id(..)`, `.disabled(..)` and handlers apply to the component's root |
| `#[prop(optional)] on_close: Option<Callback<()>>` | `on_close: Option<Callback<()>>`; an `Option<..>` prop is optional by itself and becomes a setter |
| a hand-written builder struct with `impl Element` and `impl IntoView` | a `#[component]` function |
| `constant_value!(Tone);` next to your own prop type | `#[derive(Clone, PartialEq, Prop)]` on the type |
| `scope(move \|\| view)` (in the prelude and `kivact::view`) | a `#[component]`; a component library that builds without the macro uses `kivact::custom::scope` |
| a helper taking `impl IntoText` and calling `.into_text_value()` | a `Value<String>` prop |
| a helper taking `impl IntoValue<T>` and calling `.into_value()` | a `Value<T>` prop |
| a helper taking `impl Fn() -> V` content | a `ViewFn` prop |
| `type Title = Value<String>;` used as a prop type | write `Value<String>`: the macro recognizes the literal names `Value`, `Signal`, `Callback`, `View`, `ViewFn` and `Vec<View>`, and an alias is a plain prop |

`App::run` takes any function that returns a view. Keep the root function plain when it is named `app`: `#[component]` would generate a type named `App`.

## State: `kivact::state`

| Before | After |
|---|---|
| `Binding<T>` | `Signal<T>` |
| a parameter `value: impl IntoBinding<T>` | `value: Signal<T>` |
| `binding(get, set)` returned `Binding<T>` | `binding(get, set)` returns `Signal<T>` |
| `task.project(\|t\| &t.done, \|t\| &mut t.done)` | `bind!(task.done)` |
| `for_each` over a `Signal<Vec<T>>` gave each row a `Binding<T>` | each row gets a `Signal<T>`; a readonly source still gives `Value<T>` |
| `task.get_untracked()` in a component body | `task.get()`: a component body does not track reads |
| `Notifier`, `notifier()` | `Trigger`, `trigger()` with `track()` and `notify()` |
| `signal.notify()` | a `Trigger` next to the signal: `changed.notify()` |
| `use_context::<T>()` returned a `Value<T>`: `session.get().user.get()` | it returns the `T`: `session.user.get()` |
| `try_use_context::<T>()` returned `Option<Value<T>>` | it returns `Option<T>` |
| `signal.update(f)` returned `R` and panicked on the signal of a removed component | it returns nothing: a write to the signal of a removed component is reported and does nothing; read a computed value with `with` after the update |
| `Callback::bind(..)`, `Callback::to_fn()` | pass the `Callback` itself: every `on_*` and every callback prop accepts it |
| `let c: Callback<()> = (move \|\| ..).into();` | `let c = callback(move \|\| ..);` |
| `kivact::custom::Reactive<T>` | store a `Value<T>` |
| `IntoReactive<bool>` in the bounds of `show`, `show_some`, `dynamic` | `impl IntoValue<bool>`; call sites do not change |
| `kivact::state::Owner` | `kivact::custom::Owner` |
| `IntoValue`, `IntoText`, `IntoBinding`, `PropValue`, `ItemKey`, `IntoSelection`, `Selection` from the prelude | not in the prelude: `IntoValue`, `IntoText` and `IntoHandler` are in `kivact::state`, `IntoSelection` and `Selection` in `kivact::interaction`; `IntoBinding` is gone |

## Event handlers

| Before | After |
|---|---|
| `on_press(impl Into<Callback<()>>)` | a handler without an event (`on_press` and every `Callback<()>` prop) takes a closure `move \|\| ..` or a `Callback<()>` |
| a handler with an event (`on_key`, `on_activate`, `on_commit`, `on_edit`, a `Callback<E>` prop) | unchanged: a closure `move \|event\| ..`; to pass a shared `Callback<E>`, write `move \|e\| cb.call(e)` |
| `.on_draft_commit(f)` | `.on_commit(f)` |
| `stepper(items, current).on_step_press(f)` | `stepper(items, current)` with `current: Signal<usize>`; read the signal |
| `tag_input(label, tags).on_add(f).on_remove(g)` | `tag_input(tags).label(label)` with `tags: Signal<Vec<String>>` |
| `avatar(..).on_open(f)` | removed: what the avatar opens takes a `Signal<bool>`; see the `avatar` rustdoc |
| `hover_card(..).on_open_change(f)`, `navigation_menu(..).on_open_change(f)` | `.open(signal)` and read the signal |
| a handler shared by two controls built with `Callback::from(..)` | `let save = callback(move \|\| ..);` and pass `save` to both |

## Views and elements: `kivact::view`

| Before | After |
|---|---|
| `dynamic(move \|\| r.state(), \|state\| match state { .. .into_view() })` for loading, error and ready | `async_view(r, ready).loading(\|\| ..).fallback(\|error, retry\| ..)` |
| `Retry` in `.fallback(\|error, retry: Retry\| ..)`, `retry.run()` | `retry` is a `Callback<()>`: `.on_press(retry)` or `retry.call(())` |
| `kivact::task::AsyncSnapshot` | `AsyncState` |
| `card(..).disabled(..)`, `card(..).on_press(..)` (methods of one component) | the same calls, now the `Element` methods every component has |
| `.variant(..)` defined by each component | the same call, now `Element::variant` |
| `provide_direction(dir, ..)` took a constant | it takes a constant, a signal or a closure |
| `use_direction()` returned `Value<ReadingDirection>` | it returns `ReadingDirection` |
| an invisible node and a timer to make the focused field commit its draft before saving | `kivact::view::commit_focused_edit().await` |
| `spacer()` could not shrink below its content | `spacer()` has a minimum size of 0 |
| — | `PointerEvent::target_size()` and `DragUpdate::target_size()`: the size of the node the handler is on |

## Standard components: `kivact::ui`

| Before | After |
|---|---|
| `text_input("Name", value)` | `text_input(value).label("Name")` |
| `field("Name", text_input("Name", value))` | `field("Name", text_input(value))`: `field` names the control |
| `text_area("Bio", value)`, `select("Color", items, value)`, `combobox("Colors", items, value)`, `slider("Volume", value)`, `number_input("Count", value)`, `date_input("Date", day)`, `date_range_input(..)`, `time_input(..)`, `date_time_input(..)` | the name moves to `.label("..")` or to the enclosing `field("..", ..)` |
| `button(label)`, `checkbox(label, value)`, `switch(label, value)`, `radio_group(name, items, value)` | unchanged: their label is visible |
| a focusable control without a name | reported when it mounts (an error in debug builds and in `kivact::test`) |
| `toaster()` | `use_toaster()` |
| `announcer()` | `use_announcer()` |
| `selection_field(label, value_label).content(f)` | `select(items, value).label(label).trigger(view)` |
| `input_group_with(label, \|\| children)` | `input_group(label, children)` |
| `dialog(title, open).mode(DialogMode::Popover)` | `kivact::interaction::use_popup(open)` and its `surface(placement, content)` |
| `MenuApi::while_open(f)`, `CommandPaletteApi::while_open(f)` | `api.surface(placement, content)`, or `show(api.is_open(), f)` |
| `use_menu_with_open(items, open)` | `use_menu(items, open)`; the `open` signal is required by every opening behavior |
| `api.control_state()`, `api.item_state(key)` and the other `*_state()` getters | `api.control().state()`, `api.item(key).state()`: `Part::state()` |
| `kivact::ui::Snap` | `kivact::interaction::Snap` |
| `kivact::interaction::ComboboxMode` | `kivact::ui::ComboboxMode` |
| `kivact::interaction::ViewTransform` | `kivact::interaction::ZoomPan` |
| `UiRecipes`, `theme.ext::<UiRecipes>()`, `recipes.button(\|previous\| ..)` | `kivact::ui::theme::ButtonTheme` and the other `<Name>Theme` tables: `theme.with(ButtonTheme::default().root(\|theme, look, base\| ..))` |
| `UiTokens` and the `*Tokens` of each component | the theme variables of each component, such as `kivact::ui::theme::button::HEIGHT_MD` |
| `*Recipes`, `*State`, `Look`, `Variant` in `kivact::ui` | `kivact::ui::theme` |
| — | `icon_button(icon, label)` |

## Async: `kivact::task`

| Before | After |
|---|---|
| `action(ActionMode::Latest, f)` | `action(f)`: a new run cancels the one in flight |
| `action(ActionMode::Queue, f)`, `action(ActionMode::QueueLatest, f)` | `queued_action(f)`: runs go one at a time, in order, and every run is kept |
| `action.close()` | gone: a queued action finishes with its owner |
| `.on_complete(\|outcome\| match outcome { ActionOutcome::Succeeded(v) => .., ActionOutcome::Failed(e) => .. })` | `.on_complete(\|result: Result<T, UiError>\| ..)` |
| a channel plus `spawn` to receive values from your thread | `let (value, sender) = kivact::task::remote(initial);`: `value` is a `Signal<T>`, `sender` is `Send` and has `set` and `update` |
| a capacity-1 channel to wake the UI from your thread | `let (changed, wake) = kivact::task::remote_trigger();`: `wake.notify()` never blocks |
| a timer per widget for a clock | `kivact::task::clock(every)`: one shared timer per period |
| `blocking(..)` outside a running App | reported as `WrongApp` at the call site |

## App and platform: `kivact::App`, `kivact::platform`

| Before | After |
|---|---|
| `App::new(..).fonts(fonts)` | `App::new(..).theme(Theme::light().with_fonts(FONTS))` with `const FONTS: &[&[u8]] = &[..];` |
| `kivact::launch::{GpuPreference, GpuPower, GpuBackend, GpuStart, AdapterMatch}` | `kivact::platform::{GpuPreference, GpuPower, GpuBackend, GpuStart, AdapterMatch}` |
| `prefer_gpu(false)`, `EmbedOptions.prefer_gpu` | `GpuStart::Never` in `GpuPreference` |
| `ARVANE_RENDERER=cpu` / `gpu` | `cpu`, `gpu` or `auto` = `GpuStart::Never`, `AtLaunch` or `OnDemand` |
| `app.clone()` | `App` is not `Clone`; take `App::handle()` before `run` to reach it from elsewhere |
| `kivact::AppOptions`, `kivact_core::launch::AppOptions` | the setters of `App` |
| `platform::on_close_request(f)` covered the main window only | it covers the window of the component that calls it (the main window, a `window(..)`, a foreign window) |
| `kivact::test::TestFonts::Only(fonts)` | `RenderOptions::with_theme(theme.with_fonts(FONTS))` |

## Embedding: `kivact::embed`

| Before | After |
|---|---|
| `kivact::embed::Visibility` | `kivact::embed::EmbedVisibility` |
| `EmbedNotice::Stopped`, then `embedded.status()` | `EmbedNotice::Stopped(EmbedStatus)`: `Closed`, `Failed(..)` or `Panicked(..)`, sent once |
| `#[cfg(..)]` around embed code on platforms without embedding | `kivact::embed` compiles everywhere; `App::embed` returns `ErrorKind::Unsupported` where embedding is not available |

## Styles and themes: `kivact::style`

| Before | After |
|---|---|
| `ColorToken::Accent`, `Color::token(ColorToken::Accent)` | `color::ACCENT` (also `SURFACE`, `SURFACE_ALT`, `TEXT`, `TEXT_MUTED`, `ACCENT_TEXT`, `BORDER`, `FOCUS`, `DANGER`, `SUCCESS`, `WARNING`, `OVERLAY`, `SHADOW`, `SELECTION`, `HOVER`, `PRESSED`, `RAISED`) |
| `ThemeColor`, `Color::of(..)` | `ThemeVar<Color>` and `Color::var(&VAR)` |
| `Palette { accent: .., ..base.colors().clone() }` and `Theme::with_colors(palette)` | `theme.set(&color::ACCENT_VAR, rgb(..))`, one call per variable |
| `Spacing`, `Theme::with_space`, `theme.space().md` | `space::MD` (`XS`, `SM`, `MD`, `LG`, `XL`); `theme.set(&space::MD_VAR, 12)` |
| `Radii`, `Theme::with_radius`, `theme.radius().md` | `radius::MD` (`SM`, `MD`, `LG`); `theme.set(&radius::MD_VAR, 6)` |
| `Theme::with_text`, `theme.text().size` | `font::SIZE` (`SIZE_SMALL`, `SIZE_LARGE`); `theme.set(&font::SIZE_VAR, 15)` |
| `theme.colors().accent` | `theme.get(&color::ACCENT_VAR)` |
| `use_tokens()` | theme variables |
| `.style(move \|\| { let t = use_theme().get(); style().padding(t.space().md) })` | `.style(style().padding(space::MD))`: no closure, the value follows the theme |
| a `ThemeExtension` that holds colors or sizes | your own `ThemeVar` statics; `ThemeExtension` stays for tables of functions |
| `ThemeHint` | `ThemeMode` |
| `.weight(700)` | `.font_weight(700)` |
| `.italic(true)` | `.font_style(FontStyle::Italic)` |
| `.radius(8)`, `.radius_top_left(8)` and the other `radius_*` | `.border_radius(8)`, `.border_radius_top_left(8)`, `border_radius_*` |
| `.ring(2, color)`, `.ring_offset(2)`, `.ring_none()` | `.outline(2, color)`, `.outline_offset(2)`, `.outline_none()` |
| `.basis(..)`, `.grow(1)`, `.shrink(0)`, `.wrap(..)`, `.direction(..)` | `.flex_basis(..)`, `.flex_grow(1)`, `.flex_shrink(0)`, `.flex_wrap(..)`, `.flex_direction(..)` |
| `Direction::Row` | `FlexDirection::Row` |
| `.pressed(style())` | `.active(style())` |
| `text_decoration_style(move \|\| ..)` | `text_decoration_style(TextDecorationStyle::Wavy)` takes a plain value; make the whole style reactive with `.style(move \|\| ..)` |
| `color.components()`, `a.mix(b, t)` | `color.components(&theme)`, `a.mix(b, t, &theme)`; new `relative_luminance()` and `contrast_ratio(other)` |
| `Dimension`, `TextLen` | `Length`, `TextLength` |
| `GridFlow` | `GridAutoFlow` |
| `Insets` | `Edges<f64>` |
| `SpanStyle`, `TextExtras`, `DecorationStyleValue` | `TextStyle`, the one set of text properties used by paragraphs, spans and editor ranges |
| a state style that accepted only paint properties | `.hover(..)`, `.active(..)`, `.focus(..)`, `.focus_visible(..)`, `.disabled(..)`, `.selected(..)`, `.group_hover(..)`, `.group_focus(..)` accept every property |
| `Shadow`, `Filter`, `Transform`, `Paint`, gradients, `Shape`, `CornerRadius`, `BlendMode`, `Mask*`, `BorderImage*`, `PaintShader`, `FilterShader`, `FilterGraph` from `kivact::draw` | from `kivact::style` |

## Drawing: `kivact::draw`

| Before | After |
|---|---|
| `Layer::new().blend(BlendMode::Multiply)` | `Layer::new().mix(Mix::Blend(BlendMode::Multiply))` |
| `Layer::new().composite(Composite::SourceIn)` | `Layer::new().mix(Mix::Composite(Composite::SourceIn))` |
| `Offscreen` from `kivact-core` | `kivact::draw::Offscreen` (now its own crate behind the facade): `render(width, height)`, `render_svg(width, height)`, `.scale(f64)` |
| SVG export failed on font features, variations, blend modes and text decorations | they are written; only features with no SVG 1.1 or CSS equivalent fail |

## Kits and plots

| Before | After |
|---|---|
| `kivact_ui_basic::{plot, PlotModel, PlotSeries, PlotAxis, PlotRange, AxisScale, TickFormat, LineDash}` | `kivact_plot::{..}` (the crate `kivact-plot`); it works with every kit |
| a kit's `ColorName` and `Tokens` fields (`theme.ext::<Tokens>().colors.primary`) | the kit's theme variables, such as `kivact_ui_basic::PRIMARY` |
| a kit's `*Appearance`, `*Look`, `*Hue`, `*Tone` and kit-prefixed variant types | the kit's `<Component>Variant` types; see the kit's guide |
````

#### 3.0.4 文書の中の関数を `#[component]` にする規則

文書の Rust の code block の中で、view を返す関数（戻り値が `impl IntoView` か `impl Element`）を次の規則で書き換える。

1. 名前が `app` の関数は `#[component]` にしない（生成される型 `App` が prelude の `App` と衝突する）。そのまま plain な関数にする。
2. それ以外の view を返す関数には `#[component]` を付ける。
3. 引数の型を変える：

| 旧 | 新 |
|---|---|
| `impl IntoText`、`impl IntoValue<String>` | `Value<String>` |
| `impl IntoValue<T>` | `Value<T>` |
| `impl IntoBinding<T>`、`Binding<T>` | `Signal<T>` |
| `impl IntoView` | `View` |
| `impl Fn() -> V`（開くたびに作る中身） | `ViewFn` |
| `impl IntoChildren` | `Vec<View>` |
| `impl Into<Callback<()>>`、`impl Fn() + 'static`（handler） | `Callback<()>` |
| `impl Fn(E) -> R + 'static`（handler） | `Callback<E>` |
| それ以外（`u32`、`Vec<Task>`、自分の struct） | そのまま |

4. 本体の `let x = x.into_text_value();`、`let x = x.into_value();`、`let x = x.into_binding();` の行を消す。
5. 本体を `scope(move || { ... })` で包んでいたら、包みを外して中身を本体にする。
6. 戻り値の型：本体の最後の式が `show(`、`show_some(`、`dynamic(`、`for_each(`、`for_index(`、`provide(`、`provide_theme(`、`error_boundary(`、tuple `(..)` のどれかなら `impl IntoView`、それ以外（`node()`、`text(..)`、`column(..)`、`row(..)`、部品の呼出し、`async_view(..)`）なら `impl Element`。
7. 関数が型引数を持つ（`fn x<K: ..>(..)`）なら、`#[component]` を付けて型引数と bound をそのまま残す（設計 5.1 の規則 9）。lifetime 引数や const 引数を持つ関数は文書に無い（あれば STOP）。
8. 書き換えた関数の呼出し側は変えない（位置引数の順は宣言の順）。`#[prop(optional)]` を消し、`Option<..>` の型だけ残す。
9. compile で「the name `X` is defined multiple times」や型の衝突が出たら、関数の名前に `_view` を足す（`fn toast()` → `fn toast_view()`）。
10. `.with_node(|n| n.role(..))` のように `with_node` で root の `Node` の method を呼んでいた builder は、本体の root が `node()` なら `.role(..)` を直接呼ぶ。部品の呼出しの後ろの `with_node` は残す。


#### 3.0.5 how-to のガイドを直す unit（G04〜G08）の共通の手順

各 unit は、所有する英語のファイルごとに次を行い、最後に日本語を直す。

1. Rust の block の中の view を返す関数に 3.0.4 の規則を当てる。
2. 3.0.6 のコマンドでファイルの旧名を列挙し、各行を 3.0.3 の表の右の列に書き換える。下の「ファイルごとの注」に書いた箇所は、注のとおりに書き換える（表より注が優先）。
3. 本文（説明の文章）で旧 API を説明している文を、新しい API の説明に書き換える。旧名を新名に置き換えるだけで意味が通る文は置き換えるだけにする。節を消すのは注に書いたものだけ。
4. 3.0.1 のコマンド（`KIVACT_DOCS_ONLY` にそのファイル）で doctest を通す。名前の違いは 1.2 の 1. の規則で直す。試験（`#[test]`）の期待値は変えない。試験の中で旧 API を使っている行（`text_input("Name", ..)` を探す `get_by_role(Role::TextBox, "Name")` など）は、探す名前を変えない（`.label("Name")` が同じ名前を付ける）。
5. 全部の英語のファイルが通ったら、日本語のファイルの本文を英語の新しい本文の訳にし、3.0.2 の script で Rust の block を写す。

#### 3.0.6 旧名の検査コマンド（文書用）

`<files>` に検査するファイルを並べる。どちらのコマンドも何も返さなければよい。ただし 2 つ目のコマンドに出る行のうち、`.composite(`・`.blend(` で受け手が `Layer` でないもの（`MaskLayer::composite`、filter graph の primitive）と、`.radius(`・`.direction(`・`.grow(` などで受け手が `Style` でないもの（`Draw`、`Shadow`、`Transition` などの値）は旧名ではないので残す。残した行は報告に行と理由を書く。

```bash
rg -n -w -e Binding -e IntoBinding -e IntoReactive -e Notifier -e notifier -e ActionOutcome -e ActionMode \
  -e AsyncSnapshot -e constant_value -e ThemeColor -e ColorToken -e ColorName -e Palette -e Spacing -e Radii \
  -e UiRecipes -e UiTokens -e use_tokens -e ThemeHint -e Dimension -e TextLen -e GridFlow -e Insets -e SpanStyle \
  -e TextExtras -e DecorationStyleValue -e prefer_gpu -e use_menu_with_open -e on_open_change -e on_step_press \
  -e on_draft_commit -e selection_field -e input_group_with -e while_open -e toaster -e announcer -e get_untracked \
  -e into_text_value -e BadgeHue -e ItemHue -e StatHue -e ProgressHue -e WeekendHue -e CodeTone \
  -e CodeColors -e GameInputGroupVariant -e GameCellVariant -e PopBubble <files>
rg -n -e '\.(weight|italic|radius|radius_[a-z_]+|ring|ring_offset|ring_none|basis|grow|shrink|direction|pressed|blend|composite|project)\(' \
  -e 'DialogMode::Popover' -e 'kivact::launch' -e '\bscope\(' -e '\bDirection::' -e 'TestFonts::Only' \
  -e 'kivact::custom::Reactive' -e '#\[prop\(optional\)\]' -e 'Color::token' -e 'Color::of' <files>
```

`ButtonTone` は game の kit の旧名だけ（pop の `ButtonTone` は残る。設計 5.12）なので、game の文書（`docs/for-kivact-ui-user/*/game`）だけで `rg -n -w ButtonTone` を探す。`TokenKind` は DevTools の protocol の型として残るので文書では探さない（kit の syntax の色の旧名は付属書 K の 4 章の表で探す）。

`.wrap(` は機械では判定しない。`style()` の鎖の `.wrap(..)` は `.flex_wrap(..)` にし、`text_area(..).wrap(true)` や code block の `.wrap(true)` のような部品の setter は残す。


## G01：`kivact-docs` の compile の再開と `KIVACT_DOCS_ONLY`

- **wave / lane / 前提**：W11a / G1 / W1〜W10 の全 unit（K44 と同じ段。G01 は K44 に依存しない）
- **所有**：`dev/tests/docs/**`（`Cargo.toml`、`build.rs`、`src/**`）。root `Cargo.toml` は変えない（`kivact-docs` は workspace の member のまま。gate が `--exclude kivact-docs` をやめるだけ。INC-09）
- **目的**：W5 から gate で外していた `kivact-docs` を compile できる状態に戻し、guide を絞って doctest にする手段を足す（TL-03、AC-5）。kit の plot が `kivact-plot` に移ったので依存を足す（KIT-15）。
- **手順**：
  1. root `Cargo.toml` は何もしない（作業指示書の全体の指示 4）。`kivact-docs` が member であることだけを `cargo metadata --no-deps --format-version 1 | grep -o '"name":"kivact-docs"'` で確かめる（出なければ STOP）。
  2. `dev/tests/docs/Cargo.toml` の `[dependencies]` に、`kivact-ui-soft = ...` の行の直後に次の行を足す（`rg -n kivact-plot dev/tests/docs/Cargo.toml` で既にあれば足さない）：
     ```toml
     kivact-plot = { path = "../../../crates/kivact-plot" }
     ```
  3. `dev/tests/docs/build.rs` に `KIVACT_DOCS_ONLY` を足す。付属書 T が build.rs を書き換えているので、次の三か所を関数名ではなく役割で探す。
     - (a) ファイルの末尾に次の二つの関数を足す：
       ```rust
       /// `KIVACT_DOCS_ONLY`: comma-separated paths under `docs/` (a guide such as
       /// `for-kivact-user/en/how-to-guide/forms.md`, or a directory such as
       /// `for-kivact-user/en/getting-started`) and `README.md` for the root README.
       /// When it is set, only those guides become doctest modules, so a change to
       /// a few guides is checked without compiling every example.
       fn only() -> Option<Vec<String>> {
           println!("cargo::rerun-if-env-changed=KIVACT_DOCS_ONLY");
           let value = std::env::var("KIVACT_DOCS_ONLY").ok()?;
           Some(
               value
                   .split(',')
                   .map(|path| path.trim().trim_end_matches('/').to_owned())
                   .filter(|path| !path.is_empty())
                   .collect(),
           )
       }

       /// Whether the guide or directory at `rel` (relative to `docs/`, `/`-separated,
       /// or `README.md`) is generated under the `KIVACT_DOCS_ONLY` filter.
       fn included(only: Option<&[String]>, rel: &str) -> bool {
           match only {
               None => true,
               Some(paths) => paths.iter().any(|path| {
                   rel == path
                       || rel.starts_with(&format!("{path}/"))
                       || path.starts_with(&format!("{rel}/"))
               }),
           }
       }
       ```
     - (b) `fn main` の先頭で `let only = only();` を作る。root の README を guide にする行（`for readme in [...]` の loop、または `README.md` を `guide(..)` に渡す行）を、`if included(only.as_deref(), readme) { ... }` で包む（`readme` は `"README.md"` などの文字列）。
     - (c) directory を辿る関数（`fn directory(code, name, dir)` に当たるもの）に引数 `rel: &str`（`docs/` からの相対の path）と `only: Option<&[String]>` を足す。中の loop で、子の path ごとに `let child_rel = format!("{rel}/{file}");` を作り、`if !included(only, &child_rel) { continue; }` を最初に置く。子の directory を辿る再帰呼出しには `&child_rel` と `only` を渡す。`main` から木（`for-kivact-user` など）を辿る呼出しには、`rel` として木の名前（`tree`）と `only.as_deref()` を渡し、その呼出しの前に `if !included(only.as_deref(), tree) { continue; }` を置く。
     - 関数が 80 行を超えたら、loop の中の 1 ファイルの処理を `fn entry(..)` に切り出す。
  4. `dev/tests/docs/src/lib.rs` の crate doc の「`cargo test -p kivact-docs --doc`」の段落の直後に、次の 1 段落を足す：
     ```rust
     //! Set `KIVACT_DOCS_ONLY` to a comma-separated list of paths under `docs/`
     //! (files or directories, or `README.md`) to generate only those guides.
     ```
- **消すもの**：なし
- **状態の閉じ方**：新しい型なし
- **試験**：なし（`dev/tests/docs/src/tests.rs` は変えない）
- **確認（tier a）**：
  ```bash
  cargo fmt -p kivact-docs
  cargo check -p kivact-docs --all-targets --all-features
  cargo clippy -p kivact-docs --all-targets --all-features -- -D warnings
  cargo xtask check --no-write --crate kivact-docs
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_DOCS_ONLY=README.md KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --doc --all-features -- --test-threads=1 --list
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_DOCS_ONLY=for-kivact-user/en/getting-started/01-create-a-project.md KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --doc --all-features -- --test-threads=1 --list
  ```
- **完了条件**：`cargo metadata --no-deps --format-version 1 | grep -o '"name":"kivact-docs"'` が 1 件出る。上の二つの `--list` が、それぞれ README と `_01_create_a_project` の doctest だけを列挙する（他の guide の test 名が出ない）。check と clippy が通る。

## G02：チュートリアル（getting-started）の書換え

- **wave / lane / 前提**：W11b / G1 / G01（W11a）
- **所有**：`docs/for-kivact-user/en/getting-started/**`、`docs/for-kivact-user/jp/getting-started/**`
- **目的**：チュートリアルを一つの部品の形（`#[component]`）、一つの書ける handle（`Signal`）、`.label()`、theme 変数、CSS の style の名前、`async_view` で教える（API-01 の文書、API-08、DEC-05、DEC-07、AC-5）。
- **手順**：
  1. 英語の各ファイルを次の内容にする。「全文」と書いたものはファイル全体をその内容で置き換える（外側の `````markdown` の 4 つの backtick の fence は付属書の囲みであり、ファイルには書かない）。「編集」と書いたものは示した箇所だけ変える。
  2. 日本語の各ファイル（同じファイル名）を、英語の新しい本文の訳にする。見出しの構成・節の順・表の行を英語と同じにする。日本語の既存の訳語（「コンポーネント」「シグナル」など）を使う。最後に 3.0.2 の script で Rust の block を写す：
     ```bash
     for f in README 01-create-a-project 02-components-and-layout 03-state 04-input 05-lists-and-conditions 06-style-and-accessibility 07-async 08-testing; do
       perl /tmp/g-sync-rust-blocks.pl docs/for-kivact-user/en/getting-started/$f.md docs/for-kivact-user/jp/getting-started/$f.md
     done
     ```
  3. compile できないとき：名前の違いは 1.2 の 1. の規則で直す。`update` は値を返さない（DEC-15）ので、`on_press(move || x.update(..))` の形はそのまま書く（`{ x.update(..); }` にしない）。

### 01-create-a-project.md（編集）

- 「Create the project」の節の `toml` block（`[dependencies]` の block）の直後に、次の段落と block を足す：

````markdown
While you develop, Kivact's layout and drawing code is too slow without
optimization. Add this to `Cargo.toml` too: it optimizes every dependency in
debug builds and leaves your own code easy to debug.

```toml
[profile.dev.package."*"]
opt-level = 3
```
````

- Rust の block は変えない。`app` の説明の段落（「`app` is the root **view builder**: ...」）の最後に次の 1 文を足す：「`app` is the one view function in this tutorial that is not a component; from the next step on, the parts of the screen are components.」

### 02-components-and-layout.md（全文）

````markdown
# Step 2: Components and layout

[日本語](../../jp/getting-started/02-components-and-layout.md)

In this step you lay out the screen with `column` and `row`, and split it into
small components: a header and a placeholder list of tasks.

## The program so far

```rust,no_run
use kivact::prelude::*;

#[component]
fn header() -> impl Element {
    column((
        text("Tasks"),
        text("Things to do today"),
    ))
    .style(style().gap(4))
}

#[component]
fn task_row(title: Value<String>) -> impl Element {
    row((text("-"), text(title))).style(style().gap(8))
}

#[component]
fn task_list() -> impl Element {
    column((
        task_row("Buy milk"),
        task_row("Write the tutorial"),
        task_row("Go for a walk"),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    column((header(), task_list())).style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

## What is new

**A component is a function marked `#[component]`.** `header`, `task_row` and
`task_list` are components. This is the only way to write one: the standard
components such as `button` and the components of the UI kits are written the
same way. A component returns `impl Element`, a view with one root that its
caller can change: `task_list().style(..)`, `.name(..)` or `.test_id(..)` apply
to the `column` inside it.

**Props are arguments.** `task_row` takes a `title: Value<String>`. A `Value`
prop accepts a fixed value (`"Buy milk"`, a `String`) or one that changes (a
signal, or a closure, from the next step on), so `task_row("Buy milk")` works.
Arguments are positional, in the order of the signature. An argument of type
`Option<..>` is optional and becomes a setter instead, such as
`.on_close(..)`; see [Write reusable components](../how-to-guide/custom-components.md).

**The body runs once.** Kivact runs a component's body once, when the
component appears on screen, and gives it its own lifetime: state created in the
body belongs to that one instance and is cleaned up when it disappears. When
state changes, only the small closures that read it run again (next step); the
body itself never runs again. This is how SolidJS works, not how a React render
function works.

```rust,no_run
use kivact::prelude::*;

#[component]
fn click_counter() -> impl Element {
    let count = signal(0);
    column((
        text(move || format!("Pressed {} times", count.get())),
        button("Press me").on_press(move || count.update(|n| *n += 1)),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    row((click_counter(), click_counter())).style(style().gap(24).padding(24))
}

fn main() -> kivact::Result<()> {
    App::new("Counters").run(app)
}
```

Each `click_counter` has its own `count`. Pressing one button updates only the
text next to it. You will use signals and these closures in the next step.

Create signals and memos in setup code: the body of a component, the root
function, and the code that `show`, `dynamic` and `for_each` run. Closures such
as `text(move || ..)` run again on changes, so they read values that setup
already created.

**The root is a plain function.** `App::new("Tasks").run(app)` calls `app`
once and shows what it returns. `run` takes any function that returns a view.
The root of this tutorial stays a plain function named `app`; everything below it
is a component.

**`column` and `row`** place their children vertically and horizontally. They
take their children as a tuple: `column((a, b, c))`. The extra pair of
parentheses is the tuple, so `column(a, b)` does not compile. The children can
have different types, and a tuple can hold up to 16 of them.

**`style()`** describes how a view looks and is laid out. You attach it with
`.style(..)`:

- `gap(8)` puts 8 pixels between the children of a `column` or `row`.
- `padding(24)` adds 24 pixels of space inside the view, around its children.

The names follow CSS: `gap`, `padding`, `flex_grow`, `border_radius`,
`font_weight` and so on. Lengths are logical pixels, so they scale with the
display. Style methods chain because each one returns the style again.

`text` accepts a string literal, a `&str`, a `String` or a `Value<String>`.

## Run it

```text
cargo run
```

You see the title "Tasks" with "Things to do today" below it, then three lines
starting with a dash, with spacing between them. Change `gap(8)` to `gap(24)`
and run again to see the lines move apart.

Next: [Step 3: State](03-state.md)
````

### 03-state.md（全文）

````markdown
# Step 3: State

[日本語](../../jp/getting-started/03-state.md)

In this step you add a counter of remaining tasks and a button that changes
it. The screen updates by itself when the state changes.

## The program so far

```rust,no_run
use kivact::prelude::*;

#[component]
fn header() -> impl Element {
    column((
        text("Tasks"),
        text("Things to do today"),
    ))
    .style(style().gap(4))
}

#[component]
fn task_row(title: Value<String>) -> impl Element {
    row((text("-"), text(title))).style(style().gap(8))
}

#[component]
fn task_list() -> impl Element {
    column((
        task_row("Buy milk"),
        task_row("Write the tutorial"),
        task_row("Go for a walk"),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    let completed = signal(0);
    let left = memo(move || 3 - completed.get());

    column((
        header(),
        task_list(),
        text(move || format!("{} tasks left", left.get())),
        button("Complete one")
            .disabled(move || left.get() == 0)
            .on_press(move || completed.update(|n| *n += 1)),
    ))
    .style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

The list is still a placeholder. The counter and the button are a stand-in
that lets you practice state before the real list arrives in step 5.

## What is new

**`signal(0)`** creates a piece of state that starts at `0`. It returns a
`Signal<i32>`, which is a small handle to the state, not the number itself.
The handle is `Copy`, so you can pass it to as many closures as you like
without cloning anything. `Signal` is the one handle you write through: the
inputs of step 4 and the checkboxes of step 5 take a `Signal` too.

You use a signal with a few methods:

- `completed.get()` reads the current value.
- `completed.set(5)` replaces the value.
- `completed.update(|n| *n += 1)` changes the value in place. Use it when the
  new value depends on the old one.
- `completed.with(|n| ..)` borrows the value without copying it. That is
  useful for large values such as a list, and you will use it in step 5.

**Closures that follow state.** `text("Tasks")` shows a fixed string.
`text(move || format!(..))` takes a closure instead. Kivact runs the closure,
notes which signals it read with `get()`, and runs it again whenever one of them
changes. Only that one piece of text is updated. The same works for properties
such as `.disabled(move || ..)`.

**`memo`** computes a value from other state: `left` is `3 - completed`. It
recalculates when `completed` changes, and the closures that read `left`
update only if the result is different. It returns a `Value<i32>`, a handle you
can read but not write. Use `memo` for a value that several places read; for a
single place, a closure is enough.

**`button(..).on_press(..)`** creates a button and runs a closure when the
user presses it, by click, or by the Enter or Space key when it has focus. The
button label doubles as its accessible name.

**Why `move`?** These closures are used long after `app` has returned, so they
cannot borrow local variables. `move` copies the handles into the closure.
Because a `Signal` is `Copy`, the same `completed` can go into several `move`
closures. Without `move` the compiler reports that the closure may outlive
the borrowed value.

**Setup runs once per mount.** `App::run` calls the root once, and each
component body runs once when the component appears. State changes run the
small closures that read that state again, not the setup. Create signals and
memos during setup; handlers update the handles already created there.

## React and Solid correspondences

If you know React or Solid, choose by the role a value plays rather than by
renaming one to one. The prelude already provides the common names:

| React / Solid concept | Kivact |
| --- | --- |
| `useState` / `createSignal` | `signal(initial)`; read with `get` or `with`, write with `set` or `update` |
| `useMemo` / `createMemo` | `memo(move \|\| ..)` for a shared derived value |
| A value prop or accessor | A `Value<T>` prop accepts a constant, signal, memo or closure |
| A controlled input | Pass a `Signal<T>`; `bind!(form.name)` edits one field of a struct in a signal |
| An event prop | A closure, or a shareable `Callback<E>` made with `callback(..)` |
| `useRef` | `var(initial)`: a box the screen does not watch |
| `useEffect` / `createEffect` | `effect(move \|\| ..)` synchronizes with something outside the UI when the state it reads changes; compute displayed values with `memo` instead |
| `useReducer` with effects and timers | `Machine` keeps related states and events together; a transition returns `Step` with optional effects and deadlines |

This counter needs only `signal` and `memo`. Use a signal for each independent
editable value, and a memo for a value derived from it. When several flags
describe stages of one operation, such as press-and-hold, waiting, running and
cancelling, use one Machine instead of synchronizing several signals. Machine
definitions use `kivact::state::{Machine, Step, machine}`. The
[React and Solid guide](../how-to-guide/coming-from-react-or-solid.md) shows
these patterns in more detail.

## Run it

```text
cargo run
```

You see "3 tasks left" and a "Complete one" button. Each press lowers the
number by one. At "0 tasks left" the button becomes disabled.

Next: [Step 4: Input](04-input.md)
````

### 04-input.md（全文）

````markdown
# Step 4: Input

[日本語](../../jp/getting-started/04-input.md)

In this step you add a text field and an Add button. The button stays disabled
until you type something, and it clears the field after adding.

## The program so far

```rust,no_run
use kivact::prelude::*;

#[component]
fn header() -> impl Element {
    column((
        text("Tasks"),
        text("Things to do today"),
    ))
    .style(style().gap(4))
}

#[component]
fn task_row(title: Value<String>) -> impl Element {
    row((text("-"), text(title))).style(style().gap(8))
}

#[component]
fn task_list() -> impl Element {
    column((
        task_row("Buy milk"),
        task_row("Write the tutorial"),
        task_row("Go for a walk"),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    let completed = signal(0);
    let left = memo(move || 3 - completed.get());
    let draft = signal(String::new());
    let last_added = signal(String::new());

    let add = move || {
        let title = draft.get().trim().to_owned();
        if !title.is_empty() {
            last_added.set(title);
            draft.set(String::new());
        }
    };

    column((
        header(),
        task_list(),
        text(move || format!("{} tasks left", left.get())),
        button("Complete one")
            .disabled(move || left.get() == 0)
            .on_press(move || completed.update(|n| *n += 1)),
        row((
            text_input(draft)
                .label("New task")
                .placeholder("What needs doing?")
                .on_submit(add)
                .style(style().width(240)),
            button("Add")
                .disabled(move || draft.with(|text| text.trim().is_empty()))
                .on_press(add),
        ))
        .style(style().gap(8)),
        text(move || {
            let title = last_added.get();
            if title.is_empty() {
                "Nothing added yet".to_owned()
            } else {
                format!("Added: {title}")
            }
        }),
    ))
    .style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

The new task does not appear in the list yet. For now the program only
remembers the last title you added and shows it at the bottom. Step 5 turns the
list into real data.

## What is new

**`text_input(draft)`** creates a single-line text field. Its argument is a
`Signal<String>`; the field and the signal are connected in both directions.
Typing updates `draft`, and calling `draft.set(..)` changes what the field
shows. That is how the program clears the field after adding. For a field
inside a larger struct, pass `bind!(profile.name)` to edit that one field of
the struct; step 5 uses the same pattern for task checkboxes.

**`.label("New task")`** gives the field its **name**. Screen readers announce
it, and tests use it to find the field. It is not drawn on screen, which is why
the field also has `.placeholder(..)`, the hint shown while the field is empty.
For a label that is drawn next to the field, write
`field("New task", text_input(draft))` instead: `field` shows the label and
gives it to the field as its name. Every control that can take focus needs a
name. If one has none, Kivact reports it when the control appears, and in a
debug build or a test that report is an error.

**`.on_submit(add)`** runs when you press Enter inside the field.
**`.on_press(add)`** runs when you press the button. Both use the same
closure. This works because `add` only captures `Copy` signal handles, so the
closure is `Copy` itself and can be passed twice.

**`.disabled(move || ..)`** takes a closure that reads state, like `text` does.
The Add button is disabled while the draft is empty or only spaces. Here
`draft.with(|text| ..)` looks at the string without copying it, whereas
`draft.get()` clones the current string.

The `add` closure also checks for an empty title, because Enter in the field
can run it even when the button is disabled.

## Run it

```text
cargo run
```

Type "Water the plants" in the field. The Add button becomes enabled. Press
Enter or click Add: the field empties and the bottom line reads "Added: Water
the plants". The Add button is disabled again.

Next: [Step 5: Lists and conditions](05-lists-and-conditions.md)
````

### 05-lists-and-conditions.md（全文）

````markdown
# Step 5: Lists and conditions

[日本語](../../jp/getting-started/05-lists-and-conditions.md)

In this step the placeholder list becomes real. Tasks live in a signal, each row has a checkbox and a Remove button, and an empty list shows a hint.

## The program so far

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
    done: bool,
}

#[component]
fn header() -> impl Element {
    column((
        text("Tasks"),
        text("Things to do today"),
    ))
    .style(style().gap(4))
}

#[component]
fn task_row(tasks: Signal<Vec<Task>>, task: Signal<Task>) -> impl Element {
    let id = task.get().id;
    row((
        checkbox(move || task.get().title, bind!(task.done)),
        button("Remove").on_press(move || tasks.update(|list| list.retain(|task| task.id != id))),
    ))
    .style(style().gap(8))
}

#[component]
fn task_list(tasks: Signal<Vec<Task>>) -> impl Element {
    column(for_each(
        tasks,
        |task| task.id,
        move |task| task_row(tasks, task),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    let tasks = signal(vec![
        Task { id: 1, title: "Buy milk".to_owned(), done: false },
        Task { id: 2, title: "Write the tutorial".to_owned(), done: false },
        Task { id: 3, title: "Go for a walk".to_owned(), done: false },
    ]);
    let next_id = var(4);
    let draft = signal(String::new());
    let left = memo(move || tasks.with(|list| list.iter().filter(|task| !task.done).count()));

    let add = move || {
        let title = draft.get().trim().to_owned();
        if !title.is_empty() {
            let id = next_id.get();
            next_id.set(id + 1);
            tasks.update(|list| list.push(Task { id, title, done: false }));
            draft.set(String::new());
        }
    };

    column((
        header(),
        row((
            text_input(draft)
                .label("New task")
                .placeholder("What needs doing?")
                .on_submit(add)
                .style(style().width(240)),
            button("Add")
                .disabled(move || draft.with(|text| text.trim().is_empty()))
                .on_press(add),
        ))
        .style(style().gap(8)),
        show(move || tasks.with(|list| !list.is_empty()), move || task_list(tasks))
            .fallback(|| text("Nothing to do. Add a task above.")),
        text(move || format!("{} left", left.get())),
    ))
    .style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

The stand-in counter and button from step 3 are gone. The remaining count now comes from the tasks.

## What is new

**A task is a struct.** `Task` holds an `id`, a `title` and a `done` flag. Kivact needs two derives on anything it keeps in a list: `Clone`, because it copies items into rows, and `PartialEq`, because it compares old and new items to find out what changed. `tasks` is a `Signal<Vec<Task>>`. You change it with `update`: `push` adds a task, and `retain` keeps only the tasks that match, which removes one.

**`var`** is a plain mutable box that the screen does not watch. `next_id` hands out unique ids. Changing it does not update anything, which is exactly what you want for a counter that only the `add` closure uses. Use `signal` for anything the screen shows.

**`for_each(items, key, build_row)`** builds one row per item.

- `items` is the signal that holds the list.
- `key` returns a value that identifies an item, here `task.id`. Rows are matched by key, so when you remove the second task, Kivact removes only that row and keeps the others, together with any state they hold.
- `build_row` is called once for each new key. It receives a `Signal<Task>` for that item: reading it gives the current item, and writing it writes into the list.

`column(for_each(..))` places the rows in a column. Passing a readonly
`Value<Vec<Task>>` instead of the signal gives each row a readonly `Value<Task>`;
the writable list signal is what allows the checkbox to edit the original item.
Do not use an item's current array index as its key, because removing or
reordering items changes that index.

**The row is a component with two props.** `task_row` takes the list and the
row's task. Its body runs once per row, and the body does not track what it
reads, so `task.get().id` there simply reads the id once for the Remove
handler. Reads inside closures, such as `move || task.get().title`, follow
later changes.

**Binding a checkbox to one field.** `checkbox(label, value)` shows a checkbox that edits a `Signal<bool>`. The row's `done` flag lives inside the list, so `bind!(task.done)` makes a `Signal<bool>` for just that field of the task. Clicking the checkbox writes into the list, and the `left` memo recounts.

`checkbox` takes a closure as its label, so the label follows the title. The label is drawn and is also what a screen reader announces.

**Why `task_row` takes `tasks`.** The Remove button must change the whole list, not just the row's item, so the row needs the list signal. A `Signal` is `Copy`, so passing it down costs nothing.

**`show(condition, view).fallback(other)`** shows `view` while the condition is true and `other` while it is false. The condition is a closure over a signal, like other changing properties. Here the list appears when there is at least one task, and the hint appears when there is none. When the branch switches, Kivact builds the new branch and discards the old one.

## Run it

```text
cargo run
```

You see the three tasks, each with a checkbox and a Remove button, and "3 left" at the bottom. Check "Buy milk": the count drops to "2 left". Add a task with the field: it appears at the end of the list. Remove every task: the list is replaced by "Nothing to do. Add a task above."

Next: [Step 6: Style and accessibility](06-style-and-accessibility.md)
````

### 06-style-and-accessibility.md（全文）

````markdown
# Step 6: Style and accessibility

[日本語](../../jp/getting-started/06-style-and-accessibility.md)

In this step you give the app a look: a larger title, muted secondary text, a card around the list and button variants. You also give every control a name that assistive technology can read, and add a dark mode switch.

## The program so far

```rust,no_run
use kivact::prelude::*;

const LOGO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect width="24" height="24" rx="6" fill="#336699"/><path d="M6 12.5l4 4 8-9" stroke="white" stroke-width="2.5" fill="none"/></svg>"##;

#[derive(Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
    done: bool,
}

#[component]
fn header(dark: Signal<bool>) -> impl Element {
    row((
        image(LOGO).name("Tasks logo").style(style().size(32, 32)),
        column((
            text("Tasks").role(Role::Heading).level(1).style(style().font_size(28).font_weight(700)),
            text("Things to do today").style(style().color(color::TEXT_MUTED)),
        ))
        .style(style().gap(2).flex_grow(1)),
        switch("Dark mode", dark),
    ))
    .style(style().gap(12).align_items(Align::Center))
}

#[component]
fn task_row(tasks: Signal<Vec<Task>>, task: Signal<Task>) -> impl Element {
    let id = task.get().id;
    row((
        checkbox(move || task.get().title, bind!(task.done)),
        button("Remove")
            .name(move || format!("Remove {}", task.get().title))
            .variant(ButtonVariant::Ghost)
            .on_press(move || tasks.update(|list| list.retain(|task| task.id != id))),
    ))
    .role(Role::ListItem)
    .style(style().gap(8).align_items(Align::Center).justify_content(Justify::SpaceBetween))
}

#[component]
fn task_list(tasks: Signal<Vec<Task>>) -> impl Element {
    column(for_each(
        tasks,
        |task| task.id,
        move |task| task_row(tasks, task),
    ))
    .role(Role::List)
    .name("Task list")
    .style(
        style()
            .gap(8)
            .padding(space::MD)
            .border_radius(radius::MD)
            .background(color::SURFACE_ALT)
            .border(1, color::BORDER),
    )
}

#[component]
fn tasks_screen(dark: Signal<bool>) -> impl Element {
    let tasks = signal(vec![
        Task { id: 1, title: "Buy milk".to_owned(), done: false },
        Task { id: 2, title: "Write the tutorial".to_owned(), done: false },
        Task { id: 3, title: "Go for a walk".to_owned(), done: false },
    ]);
    let next_id = var(4);
    let draft = signal(String::new());
    let left = memo(move || tasks.with(|list| list.iter().filter(|task| !task.done).count()));

    let add = move || {
        let title = draft.get().trim().to_owned();
        if !title.is_empty() {
            let id = next_id.get();
            next_id.set(id + 1);
            tasks.update(|list| list.push(Task { id, title, done: false }));
            draft.set(String::new());
        }
    };

    column((
        header(dark),
        row((
            text_input(draft)
                .label("New task")
                .placeholder("What needs doing?")
                .on_submit(add)
                .style(style().width(240)),
            button("Add")
                .disabled(move || draft.with(|text| text.trim().is_empty()))
                .on_press(add),
        ))
        .style(style().gap(8)),
        show(move || tasks.with(|list| !list.is_empty()), move || task_list(tasks))
            .fallback(|| text("Nothing to do. Add a task above.")),
        text(move || format!("{} left", left.get())).style(style().color(color::TEXT_MUTED)),
    ))
    .style(
        style()
            .padding(24)
            .gap(16)
            .width(pct(100))
            .height(pct(100))
            .background(color::SURFACE)
            .color(color::TEXT),
    )
}

fn app() -> impl IntoView {
    let dark = signal(false);
    provide_theme(
        move || if dark.get() { Theme::dark() } else { Theme::light() },
        move || tasks_screen(dark),
    )
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

## What is new

**Styles use CSS names.** `style()` has methods for text, color, size and borders, named like their CSS properties.

- `font_size(28)` and `font_weight(700)` set the size and weight of the text in a view and everything below it. `font_weight` is a CSS-style number, where 400 is regular and 700 is bold.
- `color(..)` sets the text color. `background(..)`, `border(1, ..)` and `border_radius(..)` style the box itself.
- `flex_grow(1)` lets a view take the free space in a `row`, which pushes the switch to the right. `align_items(Align::Center)` and `justify_content(Justify::SpaceBetween)` align children.

**Theme variables.** `color::SURFACE`, `color::TEXT`, `color::TEXT_MUTED`, `color::SURFACE_ALT`, `color::BORDER`, `color::ACCENT` and the rest are *theme variables*, like `var(--surface)` in CSS: the active theme decides what each one is. `space::MD` and `radius::MD` are the theme's medium spacing and corner radius. Put a variable anywhere a value goes; when the theme changes, every style that uses it follows, with no closure. Use theme variables for most colors, and the same code works in the light and the dark theme. When you need one fixed color, use `rgb(0x336699)`; plain numbers such as `padding(24)` are fixed lengths in logical pixels.

**Image and font bytes.** `image(LOGO)` uses the SVG bytes embedded in this
program. `include_bytes!("logo.svg").as_slice()` supplies a file embedded at
build time. For bytes produced at runtime, `kivact::view::AssetBytes::new(bytes)`
retains an owned `Vec<u8>`. Generated RGBA pixels use
`kivact::draw::Pixels::new(width, height, rgba)`, where `rgba` can also be an
owned vector containing four bytes per pixel. Clone an existing `AssetBytes`
when reusing unchanged bytes. Fonts belong to the theme: declare
`const FONTS: &[&[u8]] = &[include_bytes!("Inter.ttf")];` and start the app
with `App::new("Tasks").theme(Theme::light().with_fonts(FONTS))`.

**Button variants.** A button is the main action by default (`Primary`, filled with the accent color). `.variant(ButtonVariant::Ghost)` gives Remove a quiet look with no fill until you hover it. The other variants are `Secondary`, `Outline`, `Danger` and `Link`.

**Accessible names.** A screen reader announces a control by its role and name, and keyboard and test tools find controls the same way. So every control needs a name:

- Controls with a visible label already have one: the button's label, the checkbox's label, the switch's label. The text field got its name from `.label(..)` in step 4.
- `.name(..)` sets or overrides the name. The Remove buttons would all read "Remove", so each one is named after its task: "Remove Buy milk". The name may be a closure, so it follows the title.
- Elements with no text, such as an image, need `.name(..)` because it is their alternative text. An image without a name is treated as decoration and hidden from assistive technology. The logo here is named "Tasks logo".
- `.role(..)` says what kind of thing a view is. `text(..)` with `Role::Heading` marks the title as a heading. `Role::List` and `Role::ListItem`, together with a name on the list, let a screen reader announce "Task list" and count its items.

**Dark mode.** `provide_theme(theme, || ..)` applies a theme to everything inside it. The theme can be a closure: here it reads the `dark` signal, so the theme changes when the "Dark mode" switch does. Kivact restyles the existing views and rebuilds nothing. `provide_theme` draws nothing by itself, so the top view gets `background(color::SURFACE)` and `color(color::TEXT)`, the way a web page sets them on `body`.

If you do not need a switch, a fixed theme is one line: `App::new("Tasks").theme(Theme::dark())`.

**Text decorations and visibility.** Add `text_decoration(TextDecorationLine::UNDERLINE)` and `text_decoration_style(TextDecorationStyle::Wavy)` to draw a wavy underline. The default pattern is `Solid`; `Dashed` and `Dotted` are also available. To change the pattern with state, make the whole style a closure: `.style(move || style().text_decoration_style(..))`. `visibility(Visibility::Hidden)` preserves the subtree's layout space while excluding it from painting, assistive technology, focus and pointer hits. `Visibility::Visible` restores it. These types live in `kivact::style`. Text color changes interpolate with `TransitionProp::All`.

## Run it

```text
cargo run
```

The window shows the logo, a bold title with a gray subtitle, and a "Dark mode" switch on the right. The tasks sit in a bordered card, each with a checkbox and a quiet Remove button. Turn on the switch: the whole window changes to dark colors at once. Press Tab to move focus through the controls and Space to check a task.

Next: [Step 7: Async](07-async.md)
````

### 07-async.md（全文）

07 の Rust の block は、06 の Rust の block に次の変更を加えた**完全な**block とする（省略記号を残さない）：

- 冒頭の `use` を次の 3 行にする：
  ```rust
  use std::time::Duration;

  use kivact::prelude::*;
  ```
- `tasks_screen` の署名を `fn tasks_screen(dark: Signal<bool>, saved: Vec<Task>) -> impl Element`（`#[component]` は付けたまま）にし、本体の最初の 5 行（`let tasks = signal(vec![` から `let next_id = var(4);` まで）を次の 2 行に置き換える：
  ```rust
      let next_id = var(saved.iter().map(|task| task.id).max().unwrap_or(0) + 1);
      let tasks = signal(saved);
  ```
- `fn app()` の直前に、次の `load_tasks` と `loading_screen` を置く：
  ```rust
  /// Pretends to read the saved tasks from disk. The first attempt fails so that
  /// you can see the error state; every later attempt succeeds.
  async fn load_tasks(attempt: u32) -> kivact::Result<Vec<Task>> {
      kivact::task::sleep(Duration::from_millis(600)).await;
      if attempt == 1 {
          return Err(UiError::msg("Could not read the saved tasks"));
      }
      Ok(vec![
          Task { id: 1, title: "Buy milk".to_owned(), done: false },
          Task { id: 2, title: "Write the tutorial".to_owned(), done: false },
          Task { id: 3, title: "Go for a walk".to_owned(), done: false },
      ])
  }

  #[component]
  fn loading_screen(dark: Signal<bool>) -> impl Element {
      // Counts the loads, so that the fake loader can fail only the first one.
      let attempts = var(0);
      let saved = resource((), move |()| {
          attempts.update(|n| *n += 1);
          load_tasks(attempts.get())
      });
      async_view(saved, move |tasks| tasks_screen(dark, tasks))
          .loading(|| text("Loading tasks...").role(Role::Status).style(style().padding(24)))
          .fallback(|error, retry| {
              column((
                  text(format!("Error: {error}")).role(Role::Alert),
                  button("Try again").on_press(retry),
              ))
              .style(style().padding(24).gap(12))
          })
  }
  ```
- `fn app()` を次にする：
  ```rust
  fn app() -> impl IntoView {
      let dark = signal(false);
      provide_theme(
          move || if dark.get() { Theme::dark() } else { Theme::light() },
          move || loading_screen(dark),
      )
  }
  ```

07 の本文（全文。「(Rust の block)」の位置に上の block を ```` ```rust,no_run ```` で置く）：

````markdown
# Step 7: Async

[日本語](../../jp/getting-started/07-async.md)

In this step the app loads its saved tasks when it starts. Loading takes a moment, and it can fail, so the app shows a loading message, an error with a retry button, or the task list.

## The program so far

(Rust の block)

`tasks_screen` now receives the loaded tasks instead of creating them. The rest of the screen is unchanged from step 6.

## What is new

**`resource(source, load)`** starts an asynchronous job and keeps its progress as state.

- `source` is the input of the job. Kivact starts the job once at the beginning, and again whenever `source` changes; with a signal as the source, each change makes a new request and a request still running is discarded. Here the source is `()`, which never changes, so the job runs again only when you ask for it: `retry` in the error view below, or `saved.refetch()`.
- `load` receives the current source value and returns a future of `kivact::Result<T>`: `Ok(value)` on success or `Err(UiError::msg("..."))` on failure. Use `?` on an error implementing `std::error::Error + Send + Sync + 'static`, and it converts into a `UiError`.

`load_tasks` stands in for reading a file or calling a server. `kivact::task::sleep(..).await` waits without blocking the window, so the window stays responsive. Slow synchronous work, such as reading a big file, belongs in `kivact::task::blocking(..)`.

To let you see every state, the fake loader fails on the first attempt. `attempts` is a `var` (step 5) that the loader increments on every call. Remove the `if attempt == 1` check when you are done experimenting.

**`async_view(resource, ready)`** shows the state of the job, like `<Suspense>` together with an error boundary:

- `.loading(..)` builds the view shown while nothing has arrived yet.
- `ready` builds the view from the value once the job succeeds. Here it builds the whole task screen from the loaded tasks.
- `.fallback(|error, retry| ..)` builds the view shown when the job failed. `retry` is a `Callback<()>` that starts the job again; give it straight to `on_press`.

Without `.loading(..)` and `.fallback(..)`, `async_view` shows a standard loading indicator and an error message with a retry button.

The `ready` view lives as long as that result is shown. `tasks_screen` creates the `tasks` signal when the data arrives, and "Try again" starts a new load, which builds a fresh screen from the new result. Keep state that must survive a reload outside `async_view`, and pass its handles in.

When you need the states yourself, `saved.state()` returns an `AsyncState` (`Idle`, `Pending`, `Failed` or `Ready`); [Run async work from a component](../how-to-guide/async-and-tasks.md) shows how.

**Accessible roles for status messages.** `Role::Status` marks the loading message and `Role::Alert` marks the error, so a screen reader announces them when they appear.

## Run it

```text
cargo run
```

The window shows "Loading tasks..." for about half a second, then "Error: Could not read the saved tasks" and a "Try again" button. Press the button: the loading message appears again, and then the task list you built in step 6, with "3 left" at the bottom.

Next: [Step 8: Testing](08-testing.md)
````

### 08-testing.md（編集）

- 「The program with tests」の `rust,test_harness` block のうち、`#[cfg(test)]` の行より上を、07 の Rust の block と一字一句同じにする（07 の block を写し、その後ろに既存の `#[cfg(test)] mod tests { .. }` を変えずに続ける）。`use kivact::style::provide_theme;` の行は残さない。
- `mod tests` の中は変えない（試験の期待値を変えない）。本文も変えない。

### README.md（getting-started の索引、編集）

- 表の step 2 の「You will build」を `A header and a placeholder list, split into components.` にする。
- 表の step 6 の「You will build」を `Theme variables, CSS style names, button variants, accessible names and a dark mode switch.` にする。

- **消すもの**：チュートリアルの `scope(..)`、`Binding`、`project(..)`、`get_untracked`、`ColorToken`、`.weight(`、`.grow(`、`.radius(`、`dynamic(` による非同期の表示、`App::fonts`、`use kivact::style::provide_theme;`、`impl IntoText` の helper。
- **状態の閉じ方**：新しい型なし（docs は xtask の状態の検査の対象外）。
- **試験**：08 の `mod tests` の 5 つの試験（`the_first_load_fails_and_trying_again_shows_the_tasks`、`adding_a_task_clears_the_field`、`enter_adds_the_task_too`、`checking_a_task_lowers_the_count`、`removing_every_task_shows_the_hint`）が doctest として通ること。変えない。
- **確認（tier a）**：
  ```bash
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_DOCS_ONLY=for-kivact-user/en/getting-started KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --doc --all-features -- --test-threads=1
  cargo xtask check --no-write --only docs-pairs
  ```
- **完了条件**：上の doctest が全部通る。`rg -n -w -e Binding -e scope -e get_untracked -e ColorToken -e IntoText -e IntoBinding docs/for-kivact-user/*/getting-started` が何も返さない。`rg -n '\.weight\(|\.grow\(|\.radius\(|dynamic\(|\.project\(' docs/for-kivact-user/*/getting-started` が何も返さない。docs-pairs の出力に getting-started のファイルが出ない。

## G03：React・Solid からの対応表と「再利用する部品」のガイド

- **wave / lane / 前提**：W11b / G2 / G01（W11a）
- **所有**：`docs/for-kivact-user/{en,jp}/how-to-guide/coming-from-react-or-solid.md`、`docs/for-kivact-user/{en,jp}/how-to-guide/custom-components.md`
- **目的**：React・Solid の開発者に新しい API の全体（`#[component]`、`Signal`、`use_context -> T`、`async_view`、theme 変数、CSS の名前、`field()` と `.label()`、`remote`/`remote_trigger`、`Trigger`、app の object の `var`、`commit_focused_edit`）を教える。部品の書き方を一つにする（API-01・API-03・API-04・API-06・API-07・API-18 の文書、AC-5）。
- **手順**：
  1. `docs/for-kivact-user/en/how-to-guide/coming-from-react-or-solid.md` を下の「coming-from（全文）」にする。
  2. `docs/for-kivact-user/en/how-to-guide/custom-components.md` を下の「custom-components（全文）」にする。
  3. 3.0.1 の確認コマンド（`KIVACT_DOCS_ONLY=for-kivact-user/en/how-to-guide/coming-from-react-or-solid.md,for-kivact-user/en/how-to-guide/custom-components.md`）で doctest を通す。名前の違いは 1.2 の 1. の規則で直す。
  4. 日本語の 2 ファイルを英語の新しい本文の訳にし、3.0.2 の script で Rust の block を写す。

### coming-from（全文）

````markdown
# Map what you know from React or SolidJS to Kivact

[日本語](../../jp/how-to-guide/coming-from-react-or-solid.md)

If you know React or SolidJS, most Kivact concepts have a direct counterpart.
This guide maps them one to one, explains the single big difference (components
run once, like Solid's, not on every state change like React's), and covers the
Rust details that trip up developers who have not written Rust before.

It assumes you have done the [tutorial](../getting-started/README.md).

## The one idea: setup runs once

A Kivact component is a function marked `#[component]`. Its body runs **once**,
when the component appears on screen. It creates state, wires things together
and returns a view. After that, the body never runs again. When a signal
changes, only the pieces of the screen that read it are updated.

```tsx
// React: the whole function runs again on every click.
function Counter() {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(count + 1)}>count: {count}</button>;
}
```

```rust,test_harness
use kivact::prelude::*;

// Kivact: the body runs once; only the `text` closure runs again.
#[component]
fn counter() -> impl Element {
    let count = signal(0);
    column((
        text(move || format!("count: {}", count.get())),
        button("+1").on_press(move || count.update(|c| *c += 1)),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Counter").run(counter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_reading_closure_updates() {
        let screen = kivact::test::render(counter);
        screen.get_by_text("count: 0");
        screen.get_by_role(Role::Button, "+1").click();
        screen.get_by_text("count: 1");
    }
}
```

This is the model of SolidJS. Two consequences:

- **Pass a closure where a value changes.** `text(format!("count: {}", n))` reads
  `n` once while building and shows that text forever. `text(move || ...)` reads
  it inside a closure Kivact runs again when `n` changes. The same goes for
  `.disabled(..)`, `.style(..)`, `.name(..)` and every other property that can
  change: give them a closure, a signal or a memo.
- **There is no dependency array.** Anything you read with `.get()` or `.with()`
  inside a closure, memo or effect is tracked automatically.

## Concept map

| Idea | React | SolidJS | Kivact |
| --- | --- | --- | --- |
| Component | function returning JSX | function returning JSX | `#[component] fn card(..) -> impl Element`, with or without props |
| Props | props object | props object | arguments of the component: `Value<T>` for a value that may change, `Signal<T>` for a value the component writes, `View` for children, `Callback<E>` for handlers; required ones are positional, `Option<..>` ones become setters |
| Children | `children` | `props.children` | a `View` prop (one slot), `Vec<View>` (several), or `ViewFn` (built again on every opening) |
| Callback prop | `onSave` prop | `onSave` prop | a `Callback<()>` prop takes a closure or a `Callback<()>`; a `Callback<E>` prop takes a closure `move \|e\| ..` |
| Shared handler | `useCallback` | a function | `callback(f)` makes a `Copy` `Callback<E>` |
| State machine | `useReducer` plus effects | reducer plus effects | `Machine`: explicit states, events, effects and timers |
| State | `useState` | `createSignal` | `signal(value)`; `get`, `with`, `set`, `update` on one `Signal<T>` |
| Two-way value | `value` + `onChange` | `value` + `onInput` | pass a `Signal<T>`; `bind!(form.name)` for one field of a struct; `binding(get, set)` for a converted value |
| Derived value | compute in render, `useMemo` | `createMemo` | a closure for one use, `memo(..)` to share or cache |
| Effect | `useEffect` | `createEffect` | `effect(..)`; `effect_on(source, f)` to skip the first run |
| Cleanup | `useEffect` return value | `onCleanup` | `on_cleanup(..)` |
| Run once after mount | `useEffect(.., [])` | `onMount` | `on_mount(..)` |
| Mutable value, no re-render; an app object | `useRef` | a plain variable | `var(value)`: a `Copy` box the screen does not watch |
| A change with no value | a counter in state | `createTrigger` | `trigger()`: `track()` and `notify()` |
| Is this row selected? | `selected === id` in every row | `createSelector` | `kivact::state::selector(source)`, then `is(&key)` in each row |
| Element ref | `useRef` on a DOM node | `ref` | `node_ref()` and `.node_ref(r)`; `r.focus()` |
| Context | `createContext`, `useContext` | `createContext`, `useContext` | `provide(value, \|\| child)` and `use_context::<T>()`, which returns the value |
| Portal | `createPortal(child, el)`; `window.open` + a portal for a new window | `<Portal>` | `overlay(..)` in the same window; `window(title, open)` for an OS window |
| Conditional | `cond && <A/>` | `<Show when>` | `show(cond, \|\| a)`, `show_some(option, \|v\| a)` |
| Several branches | ternary or `switch` | `<Switch>` | `dynamic(source, \|v\| match v { .. })` |
| List | `items.map(..)` with `key` | `<For>` | `for_each(items, \|i\| key, \|item\| row)` |
| List by position | `key={index}` | `<Index>` | `for_index(items, \|i, item\| row)` |
| Key | `key` prop | the `For` identity | the key function you give `for_each` |
| Error boundary | `ErrorBoundary` | `ErrorBoundary` | `error_boundary(..)` |
| Data loading | libraries or `useEffect` | `createResource` | `resource(source, fetch)`; `action(f)` for user actions |
| Loading and error UI | `<Suspense>` and `<ErrorBoundary>` | the same | `async_view(resource, ready).loading(..).fallback(..)` |
| Data from another thread | `postMessage` from a worker | the same | `kivact::task::remote(initial)`: a `Signal` and a `Send` sender |
| Shared element between two places (Framer Motion `layoutId`) | `layoutId` (Framer Motion) | a library | `.layout_id("id")` on both nodes; the new one glides from the old box with a layout transition in its style |
| Old content leaves before new content enters (`AnimatePresence mode="wait"`) | `AnimatePresence mode="wait"` | a library | `show(cond, \|\| a).fallback(\|\| b).presence(PresenceMode::Wait)`; `Sync` and `PopLayout` are the other modes |
| A value that follows scrolling (`useScroll`) | `useScroll` | a scroll listener | `scroll_progress(scroller, Orientation::Vertical)`, or `view_progress(target, scroller, axis, ViewRange::Entry)` for one element |
| A sequence on one clock (GSAP `timeline`) | `gsap.timeline()` | a library | `timeline()`, `track(..)`, `to(track, target, transition).at(At::AfterPrevious(ms))`, `play()` |
| A delay per list item (GSAP `stagger`) | `stagger` | a library | `Stagger::new(ms).from(StaggerFrom::Center)` and `stagger.delay(index, count)` |
| CSS `var(--accent)` | `var(--accent)` | the same | `color::ACCENT`, `space::MD`, `radius::MD`: theme variables you put anywhere a value goes |
| CSS `:hover`, `:active`, `:focus-visible` | the same | the same | `style().hover(..)`, `.active(..)`, `.focus_visible(..)` |
| CSS `transform: perspective(..) rotateX(..)` | `style={{ transform }}` | `style={{ transform }}` | `style().transform(Transform::new().perspective(..).rotate_x(..))` |
| CSS `filter: blur(4px)` | `style={{ filter }}` | `style={{ filter }}` | `style().filter(Filter::new().blur(4))` (`kivact::style`) |
| CSS `mask`, `clip-path`, `mix-blend-mode` | `style={{ mask }}` and so on | the same | `style().mask(..)`, `.clip_path(shape)`, `.mix_blend_mode(BlendMode::Multiply)` |
| CSS `font-variation-settings`, `font-stretch` | `style={{ fontVariationSettings }}` | the same | `style().font_variation_settings([("wght", 650.0)])`, `.font_width(75.0)` |
| CSS `-webkit-text-stroke`, `paint-order` | `style={{ WebkitTextStroke }}` | the same | `style().text_stroke(width, color)`, `.paint_order(..)` |

The ordinary names in the table come with `use kivact::prelude::*;`, including
`callback`, `trigger`, `error_boundary` and `effect_on`. Import `selector` from
`kivact::state`, `remote` and `remote_trigger` from `kivact::task`, and, for a
state machine, `Machine`, `Step` and `machine` from `kivact::state`.

`Machine` is `useReducer` with effects and timers. Define explicit states and
events, return a `Step` from `transition`, and execute effects in the runner.
See [custom controls](custom-controls.md#write-behavior-with-a-state-machine).

## Props, children and callbacks

A component's props are its arguments; Kivact writes the props builder from the
signature.

```rust,test_harness
use kivact::prelude::*;

// `Value<String>`: a literal, a String, a signal or a closure that returns text.
// `Value<i32>`: a number, a signal, a memo or a closure.
#[component]
fn stat(label: Value<String>, value: Value<i32>) -> impl Element {
    row((text(label), text(move || value.get().to_string()))).style(style().gap(8))
}

// `View`: any view, including another component. `Option<View>`: an optional slot.
#[component]
fn panel(title: Value<String>, child: View, footer: Option<View>) -> impl Element {
    column((text(title), child, footer)).style(style().gap(8).padding(12))
}

// A callback prop: the caller passes a closure or a `Callback`.
#[component]
fn buttons(on_add: Callback<()>) -> impl Element {
    row((button("Add").on_press(on_add), button("Add again").on_press(on_add)))
}

fn app() -> impl IntoView {
    let hits = signal(0);
    panel(
        "Stats",
        column((
            stat("Fixed", 42),
            stat("Signal", hits),
            stat("Derived", move || hits.get() * 2),
            buttons(move || hits.update(|h| *h += 1)),
        )),
    )
    .footer(text("Counts start again when the app restarts."))
    .style(style().width(320))
}

fn main() -> kivact::Result<()> {
    App::new("Props").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_prop_type_takes_constants_signals_and_closures() {
        let screen = kivact::test::render(app);
        screen.get_by_text("42");
        assert_eq!(screen.all_by_text("0").len(), 2);
        screen.get_by_text("Counts start again when the app restarts.");
        screen.get_by_role(Role::Button, "Add again").click();
        screen.get_by_text("1");
        screen.get_by_text("2");
    }
}
```

- A prop that may change is `Value<T>`. The caller passes a constant, a
  signal, a memo or a closure, and the body gets a `Value<T>` to read with
  `.get()` or `.with(..)`.
- A callback prop is `Callback<E>`. For an action without input
  (`Callback<()>`), the caller passes a closure or a `Callback<()>` it already
  has. For a callback with input, the caller passes a closure; a shared
  `Callback<E>` goes in as `move |e| cb.call(e)`. The body gets a `Copy`
  handle, so both buttons can use the same action; hand a `Callback<()>` to any
  `on_press`, or call any callback with `.call(event)`.
- An `Option<..>` prop is optional: the caller sets it with a setter of the
  same name, `.footer(..)`. `#[prop(default = expr)]` gives a prop a default
  value instead.
- Wherever a closure returning `()` is accepted, a closure returning
  `Result<(), UiError>` is accepted too, so you can use `?` inside a handler.
  The error reaches the nearest error handler.
- A component returns `impl Element`, so its caller can add `.style(..)`,
  `.name(..)` or `.test_id(..)`, as `app` does on `panel`. The caller's style
  comes after the component's own and wins.
- A component can have type parameters (`fn picker<K: Clone + PartialEq + 'static>(..)`).
  Where React would take any child, take a `View`, a `Vec<View>` or a `Value<T>` prop.

## Effects, cleanup and mount

```rust,no_run
use kivact::prelude::*;

#[component]
fn tracker(query: Signal<String>) -> impl Element {
    // Runs once the screen is built, then again whenever `query` changes.
    effect(move || {
        println!("searching for {}", query.get());
    });
    // Runs once, after the nodes of this component exist.
    on_mount(|| println!("mounted"));
    // Runs when this component is removed (a branch closes, a row disappears).
    on_cleanup(|| println!("gone"));

    text(move || format!("query: {}", query.get()))
}

fn main() -> kivact::Result<()> {
    App::new("Effects").run(|| tracker(signal("rust".to_owned())))
}
```

- Use `effect` for side effects such as logging, saving or talking to something
  outside the UI. Do not use it to copy one signal into another to show a derived
  value: read the source inside a closure or a `memo` instead.
- `on_cleanup` called inside an effect runs before the effect runs again and
  when it is removed, like the function a React effect returns.
- State belongs to the nearest *boundary*: a component, the root, a `show`
  branch, a `for_each` row, a `provide` or an `error_boundary`. Signals created
  inside are freed when that boundary goes away.

## Conditions, branches and lists

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    let logged_in = signal(false);
    let user = signal(None::<String>);
    let names = signal(vec!["Ann".to_owned(), "Bob".to_owned()]);

    column((
        // `<Show when=.. fallback=..>`
        show(logged_in, || text("Welcome back")).fallback(|| text("Please sign in")),
        // Option: the factory gets a `Value<String>` for the current value.
        show_some(user, |name| text(move || format!("Hello, {}", name.get()))),
        // `<For>`: rows keep their state by key.
        column(for_each(names, |name| name.clone(), |name| text(move || name.get()))),
        // `<Index>`: rows keep their state by position.
        column(for_index(names, |i, name| text(move || format!("{i}: {}", name.get())))),
        button("Toggle").on_press(move || logged_in.update(|v| *v = !*v)),
        button("Sign in as Ann").on_press(move || user.set(Some("Ann".to_owned()))),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Branches").run(app)
}
```

`show(condition, factory)` builds the branch when the condition is true and
throws it away (with its state) when it turns false. `condition` is a signal, a
memo or a closure returning `bool`. The factory returns a view and runs each time
the branch opens. [Lists and tables](lists-and-tables.md) goes into keys and row
state in detail.

## `dynamic` and match arms

Rust requires every arm of a `match` to have the same type. The element
functions (`text`, `column`, `node`) all return `Node`, so arms made only of those
agree, but a component such as `button(..)` returns its own type, and mixing the
two is a type error: `match arms have incompatible types`. Inside `dynamic`, end
each arm with `.into_view()` to erase the type:

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
enum Mode {
    Read,
    Edit,
}

#[component]
fn editor() -> impl Element {
    let mode = signal(Mode::Read);
    column((
        button("Switch").on_press(move || {
            mode.update(|m| {
                *m = match m {
                    Mode::Read => Mode::Edit,
                    Mode::Edit => Mode::Read,
                }
            })
        }),
        // The source must be `Clone + PartialEq`. The view is rebuilt
        // when it changes to a different value.
        dynamic(mode, |mode| match mode {
            Mode::Read => text("Read only").into_view(),
            Mode::Edit => button("Save").into_view(),
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Modes").run(editor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_swaps_the_arms() {
        let screen = kivact::test::render(editor);
        screen.get_by_text("Read only");
        screen.get_by_role(Role::Button, "Switch").click();
        assert!(screen.query_by_role(Role::Button, "Save").is_some());
        assert!(screen.query_by_text("Read only").is_none());
    }
}
```

`.into_view()` is needed only for this shape: one `match` or `if` that returns
different kinds of view. Tuples (up to 16 children, each of its own type), `Vec`,
and the separate factories of `show` and `show_some` need no conversion. For two
branches prefer `show(..).fallback(..)`; reach for `dynamic` when you have three
or more or the kinds differ. For loading, error and ready, use `async_view`
rather than a `dynamic` over the state. `dynamic` rebuilds the view when the
source changes, so do not use it just to update a piece of text: use a closure.

## Context

`provide(value, || child)` makes `value` available to the child and everything
below it; `use_context::<T>()` returns the nearest one. `T` must be `Clone`, so
provide a struct of `Copy` handles, or an `Rc` for anything heavy.

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, Copy)]
struct Session {
    user: Signal<String>,
}

#[component]
fn greeting() -> impl Element {
    let session = use_context::<Session>(); // panics, naming the type, if nothing provided it
    text(move || format!("Signed in as {}", session.user.get()))
}

#[component]
fn settings() -> impl Element {
    let session = use_context::<Session>();
    button("Rename").on_press(move || session.user.set("ann2".to_owned()))
}

fn app() -> impl IntoView {
    let session = Session { user: signal("ann".to_owned()) };
    provide(session, || column((greeting(), settings())))
}

fn main() -> kivact::Result<()> {
    App::new("Context").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn children_share_the_provided_state() {
        let screen = kivact::test::render(app);
        screen.get_by_text("Signed in as ann");
        screen.get_by_role(Role::Button, "Rename").click();
        screen.get_by_text("Signed in as ann2");
    }
}
```

- `use_context` returns the provided value itself, like Solid's `useContext`.
  The value does not change; put signals inside it, as `Session` does, for
  state that changes.
- There are no default values. `use_context::<T>()` panics with the type name when
  nothing was provided. When the value is optional, use
  `kivact::state::try_use_context::<T>()`, which returns an `Option<T>`.
- `provide` only affects its child. Siblings built after it do not see the value.
- Themes, locale, messages and reading direction have their own `provide_*`
  functions: `provide_theme`, `kivact::i18n::provide_locale`,
  `kivact::i18n::provide_messages` and `kivact::view::provide_direction`
  ([Theming](theming.md), [Internationalization](i18n.md)).

## Refs and app objects

- `var(value)` is a mutable box that does **not** notify anything when it
  changes, like `useRef(value).current`. It is also how you keep an object of
  your app that is not `Copy` (a history, a parser, a connection) in a `Copy`
  handle that any number of closures can share.
- `node_ref()` makes a handle you attach with `.node_ref(r)`; after mount,
  `r.focus()` focuses the element and `r.rect()` reads its box. It is not a
  DOM object you can modify: Kivact owns the elements.

```rust,no_run
use kivact::prelude::*;

/// An object of the app that is not `Copy`.
struct History {
    entries: Vec<String>,
}

#[component]
fn search() -> impl Element {
    let query = signal(String::new());
    let input = node_ref();
    let history = var(History { entries: Vec::new() }); // changing it redraws nothing

    column((
        text_input(query).label("Search").node_ref(input),
        button("Search").on_press(move || {
            history.update(|h| h.entries.push(query.get()));
            input.focus();
        }),
        button("Print history").on_press(move || {
            history.with(|h| println!("{} searches", h.entries.len()));
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Refs").run(search)
}
```

## Values from another thread

A worker thread of your own (an audio engine, a file watcher, a network client)
cannot touch a `Signal`: signals live on the UI thread. `remote(initial)` gives
you a signal and a sender that can go to any thread.

```rust,no_run
use std::time::Duration;

use kivact::prelude::*;
use kivact::task::remote;

#[component]
fn ticks_from_a_thread() -> impl Element {
    // `ticks` is an ordinary signal; `sender` is `Send` and goes to the thread.
    let (ticks, sender) = remote(0u64);
    std::thread::spawn(move || {
        for n in 1.. {
            std::thread::sleep(Duration::from_millis(500));
            sender.set(n);
        }
    });
    text(move || format!("ticks: {}", ticks.get()))
}

fn main() -> kivact::Result<()> {
    App::new("Remote").run(ticks_from_a_thread)
}
```

Writes are merged: the UI sees the latest value once per turn, however often
the thread writes. When the thread only has to say "something changed",
`remote_trigger()` gives a `Trigger` for the UI and a `Send` sender whose
`notify()` never blocks. See
[Run async work from a component](async-and-tasks.md#receive-values-from-another-thread).

## Save what the user is still typing

A text field with `.on_commit(..)` writes its value when the user presses Enter
or leaves the field. A keyboard shortcut does not leave the field, so before
saving, ask the focused control to commit with `commit_focused_edit()`:

```rust,no_run
use kivact::prelude::*;
use kivact::view::commit_focused_edit;

#[component]
fn document() -> impl Element {
    let title = signal(String::from("Untitled"));
    let saved = signal(String::new());
    let save = action(move || async move {
        // Commit the field the user is still typing in, then read the state.
        commit_focused_edit().await;
        saved.set(title.get());
        Ok(())
    });
    column((
        text_input(title)
            .label("Title")
            .on_commit(move |value: String| title.set(value)),
        text(move || format!("Saved: {}", saved.get())),
    ))
    .on_key(move |event: KeyEvent| {
        if event.key() == Key::Char('s') && event.modifiers().ctrl {
            save.run(());
            Handled::Yes
        } else {
            Handled::No
        }
    })
}

fn main() -> kivact::Result<()> {
    App::new("Document").run(document)
}
```

## The Rust points

### 1. Closures need `move`

A closure that Kivact stores (an event handler, a text closure) lives longer than
the function that created it. A plain `|| count.get()` would only *borrow*
`count` from a function that is about to return, and the compiler refuses:

```text
error[E0373]: closure may outlive the current function, but it borrows `count`,
which is owned by the current function
help: to force the closure to take ownership of `count`, use the `move` keyword
```

Write `move || count.get()`. `move` takes the variables into the closure; it does
not copy big data by itself. Stored closures must be `'static`, meaning they hold
no references to local variables.

### 2. Handles are `Copy`; data is cloned on request

`Signal<T>`, `Value<T>`, `Var<T>`, `Callback<E>` and `Trigger` are small `Copy`
handles into the app's state. Copy as many as you like into `move` closures;
they all point at the same value. You never write `.clone()` on a handle.

The data inside is a different story. `signal.get()` returns a **clone** of the
value, so it needs `T: Clone`. To look at a `String` or `Vec` without cloning,
borrow it: `names.with(|n| n.len())`, then return the result you need. Do not
write to the same signal from inside `with`.

### 3. `Clone + PartialEq` bounds

`set` only notifies readers when the new value differs, so it needs
`T: PartialEq`. `get` needs `T: Clone`. `memo`, `for_each` items and a
`dynamic` source also compare values. For your own types, derive both:

```rust,no_run
#[derive(Clone, PartialEq)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}
# fn main() {}
```

`signal(value)` itself, and `with` and `update`, work for any type. To pass a
constant of your own type to a `Value<T>` prop, also derive `Prop`
(`#[derive(Clone, PartialEq, Prop)]`); see
[Write reusable components](custom-components.md#use-your-own-type-as-a-prop).

### 4. `Option<T>` for "no value"

There is no `null`. A value that may be absent is an `Option<T>`: `Some(x)` or
`None`. Handle it with `match`, `if let`, `.map(..)` or `.unwrap_or(..)`, never a
blind `.unwrap()`. For views use `show_some(option_signal, |v| ..)`; its factory
receives a `Value<T>` for the current contents. `select` and `radio_group` edit an
`Option<K>` (nothing chosen is `None`); `.optional()` turns a `Signal<T>` into a
`Signal<Option<T>>` for them.

### 5. `String` and `&str`

`String` owns its text; `&str` borrows it. Text arguments accept both, so
`text("Hello")` and `text(some_string)` work. A closure that returns text may
return a `&'static str` literal or a `String`:

```rust,no_run
use kivact::prelude::*;

#[component]
fn status(saved: Signal<bool>, name: Signal<String>) -> impl Element {
    column((
        text(move || if saved.get() { "Saved" } else { "Not saved" }),
        text(move || format!("Name: {}", name.get())),
        // Return an owned value out of `with`; a borrow cannot leave it.
        text(move || name.with(|n| n.to_uppercase())),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Strings").run(|| status(signal(false), signal("ann".to_owned())))
}
```

A closure can not return a `&str` that points into a signal, because the borrow
ends with `with`. Return an owned `String` (`n.clone()`, `format!(..)`).

### Closure arguments sometimes need a type

Rust infers closure argument types from the function that receives them. For
`on_key(move |event| ..)` and a `Callback<E>` prop that works. For `callback(..)`
and `action(..)`, where the argument type is chosen by the closure itself,
annotate it when you call a method on it:
`callback(move |word: String| len.set(word.len()))`. The compiler says
`type annotations needed` when you must.

### 6. A numeric closure must return the expected type

A constant integer can be accepted by a floating-point property, but a stored
closure must return that property's exact type. If `count` is a `Signal<i32>`,
`move || count.get()` cannot provide an `f64`. Cast inside the closure. When
the error names a trait, check the closure's return type first:

```rust,no_run
use kivact::prelude::*;

#[component]
fn count_slider(count: Signal<i32>) -> impl Element {
    let value = binding(
        move || count.get() as f64,
        move |next: f64| count.set(next.round() as i32),
    );
    slider(value).label("Count").range(0.0..=100.0)
}
```

`binding(get, set)` returns a `Signal<f64>` that reads and writes through the two
closures. For a plain numeric property, write `move || count.get() as f64`.
Casting the handle itself does not convert the values read later.

### 7. A local name can hide a prelude name

Rust uses the nearest binding for a name. After `let text = signal(String::new())`,
`text("Hello")` tries to call that signal and reports E0618, “expected function”.
Use names that describe the data: `label`, `rows`, `choices`, `name_field` or
`save_action`, leaving `text`, `row`, `list`, `field`, `message` and `window`
available as prelude functions. The theme variable modules `color`, `space`,
`radius` and `font` are prelude names too: `let color = ..` hides `color::ACCENT`,
so call it `tint`. `window` is a common local name (`let window = …`): call it
`settings_window` instead. Name a row callback's argument after its data too,
such as `item` or `todo`, rather than `row`. The same rule applies to an imported
type such as `Action`; avoid reusing it for another item. If a name must stay,
qualify it, such as `kivact::view::text("Hello")`,
`kivact::ui::field("Name", input)` or `kivact::task::Action`.

```rust,no_run
use kivact::prelude::*;

#[component]
fn heading_row(label: Value<String>) -> impl Element {
    row((text(label), button("Open")))
}
```

## See also

- [Tutorial](../getting-started/README.md): build an app step by step.
- [Lists and tables](lists-and-tables.md): keys, row state and large collections.
- [Forms](forms.md): the draft-and-validate pattern.
- [Run async work from a component](async-and-tasks.md): `resource`, `action` and `remote`.
- [Write reusable components](custom-components.md): writing components other people use.
- [Migrate an app to the reworked API](migrating-to-rework.md): old names and their replacements.
- Reference: `cargo doc -p kivact --open`.
````

### custom-components（全文）

````markdown
# Write reusable components

[日本語](../../jp/how-to-guide/custom-components.md)

Use this guide to turn a piece of UI you repeat into a component that other code
can reuse: a chip, a card, a notice with an action, a search box. It covers how to
take props, children and callbacks, how to ship a default look the caller can still
change, how to put your own colors on the theme and how to keep accessibility
intact when a component wraps another control. It assumes you have read the
getting-started tutorial.

A component is a function marked `#[component]`. It is the only way to write
one: your app, the standard components of `kivact::ui` and the official kits all
use it. Kivact writes the props builder from the signature, so you never write a
builder by hand.

| Return type | Use it when |
|---|---|
| `#[component] fn card(..) -> impl Element` | The usual case: the result has one root, and callers may add `.style(..)`, `.name(..)`, `.test_id(..)` and handlers to it. |
| `#[component] fn section(..) -> impl IntoView` | The result has no single root, such as a `show(..)` branch or a tuple. Callers cannot add modifiers. |

## Start with a component

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, Copy, PartialEq, Prop)]
pub enum ChipTone {
    Normal,
    Accent,
}

/// A reusable label with a reactive tone and an action.
#[component]
pub fn chip(
    label: Value<String>,
    on_press: Callback<()>,
    #[prop(default = ChipTone::Normal)] tone: Value<ChipTone>,
) -> impl Element {
    button(label).on_press(on_press).style(move || {
        style()
            .border_radius(12)
            .padding_x(10)
            .padding_y(4)
            .background(if tone.get() == ChipTone::Accent {
                color::ACCENT
            } else {
                color::SURFACE_ALT
            })
    })
}

fn app() -> impl IntoView {
    let highlighted = signal(false);
    chip("Kivact", move || highlighted.update(|value| *value = !*value))
        .tone(move || if highlighted.get() { ChipTone::Accent } else { ChipTone::Normal })
        .style(style().width(120))
}

fn main() -> kivact::Result<()> {
    App::new("Chips").run(app)
}
```

How the signature becomes the API:

- **A required prop is a positional argument**, in the order of the signature.
  `chip("Kivact", ..)` takes the label and the press handler by position.
- **An `Option<..>` prop and a `#[prop(default = expr)]` prop become setters.**
  The chip above can be written `chip("Kivact", ..)` with the tone left out, or
  `chip("Kivact", ..).tone(ChipTone::Accent)`. An optional callback is
  `on_close: Option<Callback<()>>`: the caller writes `.on_close(..)`, and the body
  receives an `Option<Callback<()>>`.
- **The body runs once, in its own setup owner.** Signals, effects, callbacks and
  cleanup that it creates belong to this instance and are disposed when the view
  is removed. The component adds no layout box; the body's root is the
  component's root.
- **The caller can change the root.** Because `chip` returns `impl Element`, the
  caller's `.style(..)` applies to the button inside. The caller's style comes
  after the body's own style and wins where both set the same property. The
  caller's handlers run before the body's, and the caller's `name`,
  `description`, `disabled`, `test_id`, `node_ref` and `variant` replace the
  body's.
- **Callbacks take closures.** A `Callback<()>` prop accepts a closure
  (`move || ..`) or a `Callback<()>` the caller already has. A `Callback<E>` prop
  with an input accepts a closure (`move |event| ..`), whose argument type Rust
  infers from `E`; forward a parent's `on_pick: Callback<i32>` as
  `picker(move |n| on_pick.call(n))`.

`Value<T>` takes a constant, a signal, a `Value` or a closure, so a prop can be
static or reactive without a second API. A `Value<String>` prop also accepts
`&str` and `String`. A value prop's type must be `Clone + PartialEq`, which is
what lets Kivact skip updates when the value did not change.

The macro recognizes the prop kinds by their written names: `Value<T>`,
`Signal<T>`, `Callback<E>`, `View`, `ViewFn` and `Vec<View>`. A type alias of one
of them is a plain prop; write the name itself.

## Use your own type as a prop

If a prop has a type of your own, derive `Prop` on it so a constant can be passed
directly:

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, Copy, PartialEq, Prop)]
pub enum Tone {
    Info,
    Warning,
}

/// A label with a tone that follows a signal.
#[component]
pub fn badge(label: Value<String>, tone: Value<Tone>) -> impl Element {
    node()
        .name(label)
        .style(move || match tone.get() {
            Tone::Info => style().background(color::ACCENT).color(color::ACCENT_TEXT),
            Tone::Warning => style().background(color::WARNING).color(color::TEXT),
        })
        .child(text(label))
}

fn app() -> impl IntoView {
    let online = signal(true);
    row((
        badge("Saved", Tone::Info),
        badge("Offline", move || if online.get() { Tone::Info } else { Tone::Warning }),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Badges").run(app)
}
```

Without `Prop`, `badge("Saved", Tone::Info)` fails to compile. Signals, `Value`s
and closures of `Tone` work either way.

## Take children and slots

A component that wraps content takes it as `View`. Several children are a tuple,
because a tuple of views is a view, and `()` means "nothing here", so it doubles
as an empty slot. A component with named slots takes one parameter per slot; an
`Option<View>` slot is optional.

```rust,no_run
use kivact::prelude::*;

/// A titled box. `body` is whatever the caller wants inside, `footer` may be `()`.
#[component]
pub fn card(title: Value<String>, body: View, footer: View) -> impl Element {
    column((
        text(title).style(style().padding_bottom(4)),
        body,
        footer,
    ))
    .style(
        style()
            .padding(space::MD)
            .gap(8)
            .border(1, color::BORDER)
            .border_radius(radius::MD),
    )
}

fn app() -> impl IntoView {
    column((
        card("Profile", (text("Ann"), text("Admin")), ()),
        card("Danger zone", text("Deleting cannot be undone"), button("Delete")),
    ))
    .style(style().gap(12).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Cards").run(app)
}
```

A `View` slot is consumed once, when the component is used. If your component
mounts its content more than once, for example each time a section opens, ask for
`ViewFn`: the caller passes a function, and the content is built again on every
opening.

```rust,no_run
use kivact::prelude::*;

/// A section that builds its content when it opens and drops it when it closes.
#[component]
pub fn collapsible(title: Value<String>, open: Signal<bool>, content: ViewFn) -> impl Element {
    column((
        button(title).on_press(move || open.update(|open| *open = !*open)),
        show(open, content),
    ))
}

fn app() -> impl IntoView {
    let open = signal(false);
    collapsible("Details", open, || text("Created by Ann"))
}

fn main() -> kivact::Result<()> {
    App::new("Collapsible").run(app)
}
```

A `Signal<bool>` prop is state that the component both reads and writes; the
caller hands over its own signal, or `bind!(form.details_open)` for one field of
a struct.

When each child needs its own wrapper, for example every pane of a split view,
take `Vec<View>`. The caller passes a tuple of up to 16 views or a `Vec` of one
kind of view, and the body receives each child as a `View` to wrap.

## Take callbacks

```rust,no_run
use kivact::prelude::*;

/// A chip with a remove button. Delete removes it too.
#[component]
pub fn removable_chip(label: Value<String>, on_remove: Callback<()>) -> impl Element {
    // `on_remove` is a Copy handle: the button and the key handler both use it.
    row((text(label), button("Remove").on_press(on_remove)))
        .name(label)
        .focusable(true)
        .on_key(move |event: KeyEvent| {
            if event.key() == Key::Delete {
                on_remove.call(());
                Handled::Yes
            } else {
                Handled::No
            }
        })
}

fn app() -> impl IntoView {
    let removed = signal(0);
    column((
        removable_chip("Rust", move || removed.update(|n| *n += 1)),
        text(move || format!("removed: {}", removed.get())),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Remove").run(app)
}
```

The body receives a `Copy` `Callback`: give it to any `on_*`, or call it with
`.call(argument)`; for a callback without input, the argument is `()`.

A press handler gets no event. To see whether a modifier key is held, as
`event.getModifierState("Control")` does in React, read `modifiers()`:

```rust,no_run
use kivact::prelude::*;
use kivact::view::modifiers;

/// Opens its panel; Ctrl+click (Cmd+click) opens only this one.
#[component]
pub fn open_button(open: Callback<bool>) -> impl Element {
    button("Open").on_press(move || {
        let held = modifiers().get();
        open.call(held.ctrl || held.meta)
    })
}

fn main() -> kivact::Result<()> {
    App::new("Open").run(|| open_button(|only| println!("only this panel: {only}")))
}
```

The caller's closure for a `Callback<E>` prop gets its argument type from `E`, so
`|only| ..` needs no annotation. If the caller's closure may fail, it may return
`Result<(), UiError>` instead of `()`; a failure is reported to the nearest
`error_boundary` (see [Find out why something failed or updated](diagnostics.md)).

## Ship a default look

A component owns its default style, and the caller can still change it:

- **Style the root in the body.** A style closure that reads a signal or a
  `Value` follows changes without a remount.
- **Use theme variables, not fixed colors.** `color::ACCENT`, `space::MD` and
  `radius::MD` follow the active theme, with no closure. A hard-coded color cannot
  follow a theme.
- **Expose meaning as props.** A tone, a size or a variant prop says what may
  change, and the component keeps the rest consistent. The caller's `.style(..)`
  is for the rest: it comes after the body's style.

## Put your own values on the theme

Hard-coded colors in a component cannot follow a theme. Declare a theme variable
for each value your component library needs. Whoever builds the theme can set it;
a theme that does not know it uses the fallback, which can be a core variable, so
your component still matches any theme.

```rust,test_harness
use kivact::prelude::*;
use kivact::style::ThemeVar;
use kivact::test::{RenderOptions, render_with};

/// The colors of `status_dot`. Without a value in the theme, they follow the
/// core success and muted colors.
pub static ONLINE_VAR: ThemeVar<Color> = ThemeVar::new("status.online", color::SUCCESS);
pub static OFFLINE_VAR: ThemeVar<Color> = ThemeVar::new("status.offline", color::TEXT_MUTED);
pub const ONLINE: Color = Color::var(&ONLINE_VAR);
pub const OFFLINE: Color = Color::var(&OFFLINE_VAR);

#[component]
pub fn status_dot(online: Value<bool>) -> impl Element {
    node()
        .role(Role::Status)
        // `Value` reads are tracked, so the name follows the signal.
        .name(move || if online.get() { "online" } else { "offline" })
        .style(move || {
            style()
                .size(12, 12)
                .border_radius(6)
                .background(if online.get() { ONLINE } else { OFFLINE })
        })
}

fn main() -> kivact::Result<()> {
    let theme = Theme::light().set(&ONLINE_VAR, rgb(0x00aa00)).set(&OFFLINE_VAR, rgb(0x888888));
    App::new("Status").theme(theme).run(|| status_dot(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dot_color(theme: Theme) -> u32 {
        let screen = render_with(RenderOptions::with_theme(theme), || {
            row(status_dot(true)).style(style().padding(8))
        });
        let dot = screen.get_by_role(Role::Status, "online").rect().expect("laid out");
        let shot = screen.screenshot();
        shot.pixel((dot.x + 6.0) as u32, (dot.y + 6.0) as u32).expect("inside the window")
    }

    #[test]
    fn the_theme_sets_the_color_and_a_plain_theme_falls_back() {
        let themed = Theme::light().set(&ONLINE_VAR, rgb(0x00aa00));
        assert_eq!(dot_color(themed), 0x00aa00);
        assert_ne!(dot_color(Theme::light()), 0x00aa00);
    }
}
```

- A theme variable is a `static` `ThemeVar` (a name and a fallback) and a
  `const` `Color` (or `Length` with `Length::var`) that points to it. Use the
  `const` anywhere a value goes; when the theme changes, every style that uses it
  follows, like `var(--x)` in CSS.
- `theme.set(&VAR, value)` gives the variable a value in that theme. The value
  may be another variable (`theme.set(&ONLINE_VAR, color::ACCENT)`).
- The name identifies the variable in diagnostics and DevTools; prefix it with
  the name of your library.
- `ThemeExtension` is for tables that are not values, such as the recipe tables
  of the standard components ([Theming](theming.md)).

To switch themes while the app runs, wrap the root in `provide_theme` with a
closure or a signal ([Switch the theme while the app runs](theming.md#switch-the-theme-while-the-app-runs)).

## Draw icons through the theme

Do not embed an image in a component. Name the icon, and let the theme decide what
the name looks like. A kit can then give your component its own glyphs, and an app
can replace any of them. For an ordinary icon button, use the standard
`icon_button(icon, label)`; write your own only for a different shape.

```rust,test_harness
use kivact::ErrorKind;
use kivact::prelude::*;
use kivact::style::Icon;
use kivact::test::{RenderOptions, render_with};

/// A square tool button that shows the icon named `name`.
#[component]
pub fn tool_button(name: Value<Icon>, label: Value<String>) -> impl Element {
    node()
        .role(Role::Button)
        .focusable(true)
        .name(label)
        .style(style().size(32, 32))
        .child(icon(name))
}

const SEARCH: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'>\
<circle cx='10' cy='10' r='6' fill='none' stroke='black' stroke-width='2'/>\
<path d='M15 15l6 6' stroke='black' stroke-width='2'/></svg>";

fn main() -> kivact::Result<()> {
    // The app (or a kit's theme) supplies the image for the name.
    let theme = Theme::light().with_icon("search", SEARCH);
    App::new("Icons").theme(theme).run(|| tool_button("search", "Search"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_theme_resolves_the_name() {
        let theme = Theme::light().with_icon("search", SEARCH);
        let screen = render_with(RenderOptions::with_theme(theme), || tool_button("search", "Search"));
        screen.get_by_role(Role::Button, "Search");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }

    #[test]
    fn an_unknown_name_is_reported_in_debug_builds() {
        let screen = render_with(RenderOptions::default(), || tool_button("nope", "Nope"));
        screen.run_until_idle();
        if cfg!(debug_assertions) {
            let errors = screen.errors();
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].kind(), &ErrorKind::Asset);
        }
    }
}
```

`Icon::named("search")` and a bare `"search"` are the same thing, so a
`Value<Icon>` prop accepts a name, an `Asset` (an image used as is) or a signal of
either. The standard set already has the marks the standard components draw, such
as `check` and `chevron-right`; add yours with `Theme::with_icon(name, image)`.
An unknown name draws nothing and, in debug builds, is reported once.

## Put the accessible name on the real control

A composite component, such as a search box made of an icon and a text input, has a
root that is not the control the user types into. The name and the disabled state
belong to that control: if they stay on the wrapper, a screen reader announces an
unnamed text box. `field(label, control)` shows a label and gives it to the
control as its name:

```rust,test_harness
use kivact::prelude::*;

/// A search box: an icon and a labelled text field that carries the caller's state.
#[component]
pub fn site_search(
    value: Signal<String>,
    #[prop(default = "Site search")] label: Value<String>,
    #[prop(default = false)] disabled: Value<bool>,
) -> impl Element {
    row((
        icon("search"),
        field(label, text_input(value).disabled(disabled)).style(style().flex_grow(1)),
    ))
    .style(style().gap(6).align_items(Align::Center))
}

fn app() -> impl IntoView {
    let query = signal(String::new());
    site_search(query)
}

fn main() -> kivact::Result<()> {
    App::new("Search").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_box_carries_the_name() {
        let screen = kivact::test::render(app);
        let input = screen.get_by_role(Role::TextBox, "Site search");
        input.type_text("kivact");
        assert_eq!(input.value().as_deref(), Some("kivact"));
    }

    #[test]
    fn pressing_the_label_focuses_the_text_box() {
        let screen = kivact::test::render(app);
        screen.get_by_text("Site search").click();
        assert!(screen.get_by_role(Role::TextBox, "Site search").is_focused());
    }
}
```

`field` accepts any element, including your own components that return
`impl Element`. A control that can take focus and has no name is reported when it
mounts; in debug builds and in `kivact::test` the report is an error.

Focus needs no forwarding: pressing a field's label activates the control
inside it, as the second test shows.

## Report misuse of your component

When a component gets an argument it cannot honor, fail where the developer can
see the cause. `kivact::custom::raise` panics with a located message, exactly
like Kivact's own misuse; it ends the App or fails the test:

```rust,no_run
use kivact::ErrorKind;
use kivact::custom::raise;
use kivact::prelude::*;

#[component]
pub fn progress_ring(done: u32, total: u32) -> impl Element {
    if total == 0 {
        raise(
            ErrorKind::Message,
            "progress_ring: `total` must be greater than zero",
        );
    }
    text(format!("{done} / {total}"))
}

fn main() -> kivact::Result<()> {
    App::new("Ring").run(|| progress_ring(3, 10))
}
```

Use it for programmer errors only. A problem the user can cause, such as a bad file,
is a `Result` that your component shows, not a misuse.

## What `#[component]` rejects

The macro reports a signature it cannot turn into a builder, with a message that
says what to change:

- a return type other than `impl Element` or `impl IntoView`;
- lifetime or const parameters (type parameters are accepted);
- a method: a `#[component]` is a free function;
- a destructuring pattern: each prop is a named argument;
- an unknown key in `#[prop(..)]`;
- an `async`, `const`, `unsafe` or `extern` function: a `#[component]` function is a plain `fn`;
- an attribute other than doc comments, `allow`, `warn`, `deny` and `deprecated` (write `#[allow(..)]` instead of `#[expect(..)]`).

## See also

- [Find out why something failed or updated](diagnostics.md) for what happens to a
  raised error and how to see it.
- [Build controls with your own look](custom-controls.md) for components that need
  standard behavior (selection, menus, dragging, text editing) with a look of your own.
- [Change the theme and the look of components](theming.md) for theme variables,
  component themes and icons from the app's side.
- [Lay out a screen](layout.md) for the style properties.
- `cargo doc -p kivact --open`: `kivact::view::Element`,
  `kivact::view::Node`, `kivact::component`, `kivact::style::{Theme, ThemeVar, IconSet}`.
````

- 「What `#[component]` rejects」の箇条書きは、付属書 C の 4 章が macro の拒否の一覧を書いていれば、その一覧に合わせる（型引数は受け、lifetime と const の引数は拒む。設計 5.1 の規則 9）。
- **消すもの**：両ファイルの `scope(..)`、`constant_value!`、`#[prop(optional)]`、`ThemeExtension` で色を持つ例、`use_theme().get().ext::<..>()`、`impl IntoText` と `.into_text_value()`、`Binding`、`ColorToken`、「When you need an `Element`」の節、「Give a plain function its own setup lifetime」の節。
- **状態の閉じ方**：新しい型なし。
- **試験**：coming-from の `only_the_reading_closure_updates`、`one_prop_type_takes_constants_signals_and_closures`、`dynamic_swaps_the_arms`、`children_share_the_provided_state`。custom-components の `the_theme_sets_the_color_and_a_plain_theme_falls_back`、`the_theme_resolves_the_name`、`an_unknown_name_is_reported_in_debug_builds`、`the_text_box_carries_the_name`、`pressing_the_label_focuses_the_text_box`。すべて doctest として通す。期待値を変えない（`one_prop_type_takes_constants_signals_and_closures` は footer の 1 行を足した。これは新しい setter の確認であり、既存の期待値は変えていない）。
- **確認（tier a）**：
  ```bash
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_DOCS_ONLY=for-kivact-user/en/how-to-guide/coming-from-react-or-solid.md,for-kivact-user/en/how-to-guide/custom-components.md KIVACT_TEST_MIRROR=off \
    cargo test -p kivact-docs --doc --all-features -- --test-threads=1
  cargo xtask check --no-write --only docs-pairs
  ```
- **完了条件**：上の doctest が全部通る。`rg -n -w -e Binding -e scope -e constant_value -e ColorToken -e IntoText -e into_text_value -e ThemeExtension docs/for-kivact-user/*/how-to-guide/{coming-from-react-or-solid,custom-components}.md` の結果が、custom-components の「`ThemeExtension` is for tables that are not values」の 1 行（英日）だけ。docs-pairs の出力にこの 2 組が出ない。
## G04：画面を組むガイド（forms、lists-and-tables、overlays、windows、layout、images-and-fonts）

- **wave / lane / 前提**：W11b / G3 / G01・G13（W11a）
- **所有**：`docs/for-kivact-user/{en,jp}/how-to-guide/{forms,lists-and-tables,overlays,windows,layout,images-and-fonts}.md`
- **目的**：画面を組む主なガイドを新しい API に直す（API-07 の forms の誤り、DEC-05、UI-12、UI-13、USR-12、AC-5）。
- **手順**：3.0.5 の手順。ファイルごとの注：
  - **forms.md**
    - `field("Name", text_input("Name", name)...)` の形はすべて `field("Name", text_input(name)...)` にする（`field` が名前を付ける）。`field` の中でない `text_input("X", v)` は `text_input(v).label("X")` にする。
    - 「Keep a draft apart from the saved model」の本文の「`Signal::field` makes a `Binding` to one field of the struct」（API-07：存在しない API）を「`bind!(draft.name)` makes a `Signal` for one field of the struct」にする。その節の code の `draft.project(|p| &p.x, |p| &mut p.x)` をすべて `bind!(draft.x)` にする（後ろの `.optional()` は残す）。`saved.get_untracked()` は `saved.get()` にする。
    - 「`draft.project(get, get_mut)` returns a `Binding<U>` for one field. The first ...」で始まる箇条を「`bind!(draft.field)` returns a `Signal<U>` for one field of the draft; writing it writes into the draft.」の 1 文にする（closure の説明の文を消す）。
    - `text_area("Bio", bio)` → `text_area(bio)`（`field("Bio", ..)` の中）。`.wrap(true)` は text_area の setter なので残す。`select(` の複数行の呼出しは最初の引数（名前の文字列）を消す（`field(..)` の中なので `.label` は付けない）。`slider("Volume", volume)` → `slider(volume)`（`field` の中）。
    - 「Pick the right control」の表：見出しの「Binding type」を「Value type」にし、各行を `text_input(Signal<String>)`、`text_area(Signal<String>)`、`radio_group(name, items, Signal<Option<K>>)`、`select(items, Signal<Option<K>>)`、`slider(Signal<f64>)`、`switch(label, Signal<bool>)`、`checkbox(label, Signal<bool>)` にする。3 列目の「`Signal<String>` or a field」を「a `Signal<String>` or `bind!(form.field)`」にする。表の直後に次の 1 文を足す：「Controls without a visible label (text fields, selects, sliders, date and time fields) get their name from `.label(..)` or from the `field(..)` around them; a focusable control without a name is reported when it mounts.」
    - 「Commit a single field with Enter or blur」：`on_draft_commit` を `on_commit` にする（本文も）。`item.get_untracked()` → `item.get()`。節の最後に次の段落を足す：「A keyboard shortcut does not leave the field, so a draft is not committed yet when the shortcut's handler runs. Call `kivact::view::commit_focused_edit().await` before reading the state to commit the focused field first; see [the React and Solid guide](coming-from-react-or-solid.md#save-what-the-user-is-still-typing).」
    - 「Calendar and date/time fields」：`ui::date_input("Date", day)` → `ui::date_input(day).label("Date")`、同様に `date_range_input("Period", range)`、`time_input("Time", time)`、`date_time_input("Appointment", appointment)`。「Style them through `UiTokens.calendar`, ... recipe tables of `UiRecipes`」の文を「Style them through the theme variables and the `<Name>Theme` tables of `kivact::ui::theme` (`CalendarTheme`, `DateInputTheme`, `TimeInputTheme`)」にする（型名は付属書 U の 4 章に合わせる）。
    - 「Group actions and toggles」の「`UiTokens` groups and `UiRecipes` tables; caller styles still override recipes.」を「the component theme variables and `<Name>Theme` tables of `kivact::ui::theme`; caller styles still override them.」にする。`item_node(.. target.with_node(..))` は残す。
  - **lists-and-tables.md**
    - `todo.get_untracked().id` → `todo.get().id`。
    - 「The row closure runs **once per row**, with one `Binding<Todo>` that ...」の `Binding<Todo>` を `Signal<Todo>` にする。
    - 「Write back to the list from a row」の「**A binding to one field.** The row's `todo` is already a `Binding<Todo>` ...」の箇条を「**A signal for one field.** The row's `todo` is a `Signal<Todo>`; `bind!(todo.done)` makes a `Signal<bool>` for its `done` field.」にする（`project` の説明を消す）。
    - 「a writable `Binding<T>` when the source is a `Signal` (as in the example below) or a `Binding`」を「a writable `Signal<T>` when the source is a `Signal` (as in the example below)」にする。
    - data table の表の `.sort(Binding<Option<(usize, SortOrder)>>)` を `.sort(Signal<Option<(usize, SortOrder)>>)` にする。
    - `ColorToken::Border` → `color::BORDER`。
  - **overlays.md**
    - dialog の mode の表から `Popover` の行を消す。
    - 「Open from a trigger, or as a popover」の節の code block を次にする（`use_popup` と `surface` の正確な署名は付属書 U の 4 章に合わせる）：
      ```rust
      use kivact::interaction::use_popup;
      use kivact::prelude::*;
      use kivact::view::Placement;

      #[component]
      fn help() -> impl Element {
          let open = signal(false);
          let popup = use_popup(open);
          row((
              button("Help").with_node(move |n| n.part(popup.trigger())),
              popup.surface(Placement::Below, || text("Press Tab to move between fields.")),
          ))
      }

      fn main() -> kivact::Result<()> {
          App::new("Help").run(help)
      }
      ```
      （info 文字列は元の `rust,no_run` のまま）。節の最初の段落を次にする：「`.attach(trigger)` makes the trigger open the dialog and returns the trigger and the dialog together. For a non-modal panel next to its trigger, use the popup behavior: `use_popup(open)` gives the trigger its Part, and `surface(placement, content)` places the panel at the trigger. A second press on the trigger closes it again.」
    - `fn rename(name: ..)` の中の `text_input("Name", draft)` → `text_input(draft).label("Name")`。
    - toast の節：`let toaster = toaster();` → `let toasts = use_toaster();`、`toaster.push(` → `toasts.push(`、`toast_region(toaster)` → `toast_region(toasts)`。本文の「`toaster()` returns the app's single shared queue」→「`use_toaster()` returns the app's single shared queue」、`toaster.max_visible(n)` → `toasts.max_visible(n)`、`toaster.dismiss(id)` → `toasts.dismiss(id)`。
  - **windows.md**
    - `use_context::<Draft>()` の後の `draft.get().dirty.get()` → `draft.dirty.get()`。
    - 「queued `ActionMode::Queue` runs drain」を「queued `queued_action` runs drain」にする。
    - `on_close_request` の説明（「Closing the main window ends the app unless an `on_close_request` handler keeps ...」の段落と、`platform::on_close_request` の節）に次の 2 文を足す：「`on_close_request` applies to the window of the component that calls it: the main window, a `window(..)` or a foreign window. In a window other than the main window, a handler that does not return `Handled::Yes` lets the close continue to the window's `on_dismiss(DismissReason::WindowClose)`.」
  - **layout.md**
    - `fn tags()` の style の `.wrap(..)` → `.flex_wrap(..)`。`.grow(1)` → `.flex_grow(1)`。`ColorToken::SurfaceAlt` → `color::SURFACE_ALT`、`ColorToken::Danger` → `color::DANGER`。
    - `fn form_grid()` の `text_input("Name", signal(String::new()))` → `text_input(signal(String::new())).label("Name")`、`"Email"` も同様。
  - **images-and-fonts.md**
    - `.radius(` → `.border_radius(`、`.weight(` → `.font_weight(`、`ColorToken::Warning` → `color::WARNING`。
    - 「Register your own fonts」の節の最初の段落を「Fonts belong to the theme. `Theme::with_fonts(fonts)` takes TTF, OTF or TTC files, and the App loads them before the first screen. After that you can use the font's **family name** (the name written in the font, which is often not the file name) in `style().font_family(..)`:」にする。
    - その節の `fn main` を次にする：
      ```rust
      fn main() -> kivact::Result<()> {
          // In an app, embed the files instead:
          // const FONTS: &[&[u8]] = &[include_bytes!("../assets/Inter-Regular.ttf"), include_bytes!("../assets/Inter-Bold.ttf")];
          // Bytes read at run time must live as long as the app: leak them once at startup.
          let regular: &'static [u8] = std::fs::read("assets/Inter-Regular.ttf").expect("font file").leak();
          let bold: &'static [u8] = std::fs::read("assets/Inter-Bold.ttf").expect("font file").leak();
          let fonts: &'static [&'static [u8]] = vec![regular, bold].leak();
          App::new("Fonts").theme(Theme::light().with_fonts(fonts)).run(app)
      }
      ```
    - 箇条：「Call `.font(..)` once per file. Regular and bold are usually separate files; register both and pick the face with `style().weight(700)`. `italic(true)` picks ...」を「Give every file to `with_fonts`. Regular and bold are usually separate files; register both and pick the face with `style().font_weight(700)`. `font_style(FontStyle::Italic)` picks an italic face when the family has one.」にする。「For one default font in the whole app, set the theme's text family: `theme.with_text(Typography { .. })` ...」の箇条を「For one default font in the whole app, set `font_family` on the root view: it is inherited by everything below.」にする。
- **消すもの**：上の各ファイルの旧名、overlays の Popover の mode の説明。
- **状態の閉じ方**：新しい型なし。
- **試験**：6 ファイルの doctest（既存の `#[test]` の名前と期待値はそのまま）。
- **確認（tier a）**：
  ```bash
  F=for-kivact-user/en/how-to-guide
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_DOCS_ONLY=$F/forms.md,$F/lists-and-tables.md,$F/overlays.md,$F/windows.md,$F/layout.md,$F/images-and-fonts.md KIVACT_TEST_MIRROR=off \
    cargo test -p kivact-docs --doc --all-features -- --test-threads=1
  cargo xtask check --no-write --only docs-pairs
  ```
- **完了条件**：doctest が全部通る。3.0.6 の二つのコマンドを `docs/for-kivact-user/{en,jp}/how-to-guide/{forms,lists-and-tables,overlays,windows,layout,images-and-fonts}.md` に当てて何も返らない。docs-pairs の出力にこの 6 組が出ない。

## G05：小さい部品のガイド（collections、navigation、keyed-suggestions、disclosure-and-previews、feedback、indicators、metrics-and-description、chat-and-code、slides-and-files）

- **wave / lane / 前提**：W11b / G3 / G04
- **所有**：`docs/for-kivact-user/{en,jp}/how-to-guide/{collections,navigation,keyed-suggestions,disclosure-and-previews,feedback,indicators,metrics-and-description,chat-and-code,slides-and-files}.md`
- **目的**：標準部品の個別のガイドを新しい API と `kivact::ui::theme` に直す（UI-03、UI-12、UI-13、UI-14、UI-20、DEC-05、DEC-06、AC-5）。
- **手順**：3.0.5 の手順。部品の theme の型・変数の名前（`<Name>Theme`、部品の theme 変数の module）は付属書 U の 4 章の対応表を正本とする。ファイルごとの注：
  - **collections.md**：「Install `UiTokens` to adjust `table`, `item_list`, and `tree` metrics. Override `UiRecipes::table`, `item_list`, or `tree` ...」の文を「Set the theme variables of the `table`, `item_list` and `tree` components (in `kivact::ui::theme`) to adjust their metrics, and wrap `TableTheme`, `ItemListTheme` or `TreeTheme` to style individual parts; each component accepts kit variants with `.variant(..)`.」にする（元の文の残りの部分はそのまま続ける）。
  - **navigation.md**：`stepper(items, current)` の `current` が signal でなければ `signal(..)` にする。「`.on_step_press` reports activation; the application ...」の文を「Pressing an enabled step writes its index to the `current` signal; the application decides what the new step shows.」にする。`interaction::use_menu_with_open(items, open_binding)` → `interaction::use_menu(items, open)`。
  - **keyed-suggestions.md**：冒頭の「`kivact::ui::combobox(label, items, selected)` edits ... The label names the editor.」を「`kivact::ui::combobox(items, selected)` edits ... `.label(..)` names the editor.」にする。code の `combobox("Colors", options.clone(), many)` → `combobox(options.clone(), many).label("Colors")`、`select("Color", options, one)` → `select(options, one).label("Color")`。「Themes supply `UiTokens::combobox` and `UiRecipes::combobox`.」→「Themes supply the combobox theme variables and `ComboboxTheme`.」。「For caller-owned chooser content, use `selection_field(label, value_label).content(factory)`. ...」の文を「For a chooser with your own trigger content, give `select` the trigger view with `.trigger(view)`; it keeps the select's surface, keys and recipes.」にする（その文の後ろの `.presentation(...)` の説明は残す）。
  - **disclosure-and-previews.md**：`on_open_change` の文を「Read the open state from the signal you give `.open(signal)`.」にする（hover card の節）。`UiTokens` の文を部品の theme 変数の文にする。
  - **feedback.md**：`project(..)` → `bind!(..)`。`ui::input_group_with("Website", || (` → `ui::input_group("Website", (`（閉じ括弧の数を合わせる）。`ui::text_input("Host", signal(String::new()))` → `ui::text_input(signal(String::new())).label("Host")`。「`input_group(label, children)` accepts already built children. `input_group_with(label, || children)` constructs children once in the inherited `.size(..)` scope, so explicit sizes on a child still win.」を「`input_group(label, children)` takes the children, built or as a closure that builds them; a closure runs once inside the group's `.size(..)`, so explicit sizes on a child still win.」にする。
  - **indicators.md**：`UiTokens::indicators`、`UiTokens::indicators.reduced_period_ms`、`UiTokens::indicators.skeleton_transition` を付属書 U の 4 章の対応する theme 変数か `<Name>Theme` の名前にする。「`UiRecipes` provides `badge`, `avatar`, ...」を「`kivact::ui::theme` provides `BadgeTheme`, `AvatarTheme`, `AvatarGroupTheme`, `SkeletonTheme`, `SpinnerTheme` and `KbdTheme`」にする（型名は U の 4 章に合わせる）。
  - **metrics-and-description.md**：「`.delta(impl IntoText)`」を「`.delta(text)`」、「`.trend(impl IntoValue<..>)`」を「`.trend(value)`, which takes a value, a signal or a closure」にする。`UiTokens`・`UiRecipes::heading`・`UiRecipes::stat`・`UiRecipes::description_list` を theme 変数と `HeadingTheme`・`StatTheme`・`DescriptionListTheme` にする。
  - **chat-and-code.md**：`color: Some(Color::token(ColorToken::TextMuted))` → `color: Some(color::TEXT_MUTED)`。`StyledRange` の field の形が変わっていたら（ST-11：`StyledRange { range, style: TextStyle }`）付属書 S の 4 章の形にする。`.wrap(true)` は code block の setter なので残す。「`Look`. `UiRecipes` has `chat_thread`, `chat_message`, `typing_indicator`, ...」と「`UiTokens::chat` sets typing timing, and `UiTokens::code_block` sets ...」の文を、`kivact::ui::theme` の `ChatThreadTheme`・`ChatMessageTheme`・`TypingIndicatorTheme`・`CodeBlockTheme` と各部品の theme 変数の文にする（名前は U の 4 章）。
  - **slides-and-files.md**：`UiTokens` の文を「Each component's theme variables in `kivact::ui::theme::{carousel, dropzone}` hold the optional overrides.」にする。
- **消すもの**：上の各ファイルの旧名。
- **状態の閉じ方**：新しい型なし。
- **試験**：9 ファイルの doctest。
- **確認（tier a）**：
  ```bash
  F=for-kivact-user/en/how-to-guide
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_DOCS_ONLY=$F/collections.md,$F/navigation.md,$F/keyed-suggestions.md,$F/disclosure-and-previews.md,$F/feedback.md,$F/indicators.md,$F/metrics-and-description.md,$F/chat-and-code.md,$F/slides-and-files.md KIVACT_TEST_MIRROR=off \
    cargo test -p kivact-docs --doc --all-features -- --test-threads=1
  cargo xtask check --no-write --only docs-pairs
  ```
- **完了条件**：doctest が通る。3.0.6 の二つのコマンドを 9 ファイル（英日）に当てて何も返らない。docs-pairs の出力にこの 9 組が出ない。

## G06：見た目と描画のガイド（theming、i18n、animation、timelines-and-scroll、3d-and-depth、effects、shaders、canvas、drawables）

- **wave / lane / 前提**：W11b / G2 / G03
- **所有**：`docs/for-kivact-user/{en,jp}/how-to-guide/{theming,i18n,animation,timelines-and-scroll,3d-and-depth,effects,shaders,canvas,drawables}.md`
- **目的**：theme 変数、`<Name>Theme`、CSS の名前、`kivact::style` に移った CSS の値の型、`Layer::mix`、`Offscreen` の新しい形を教える（ST-01、ST-05、ST-08、ST-17、ST-22、UI-03、RC-29、DEC-02、DEC-06、DEC-07、DEC-11、AC-5）。
- **手順**：3.0.5 の手順。ファイルごとの注：
  - **theming.md**
    - 「Switch the theme while the app runs」：`use kivact::style::provide_theme;` の行を消す（prelude にある）。`ColorToken::Surface`・`ColorToken::Text` → `color::SURFACE`・`color::TEXT`。
    - 「Read tokens in your own styles」の節を「Use theme variables in your styles」に改題し、code block を次にする（info 文字列は `rust,no_run`）：
      ```rust
      use kivact::prelude::*;

      #[component]
      fn card(body: View) -> impl Element {
          column(body).style(
              style()
                  .background(color::SURFACE_ALT)
                  .border(1, color::BORDER)
                  .border_radius(radius::MD)
                  .padding(space::MD),
          )
      }

      #[component]
      fn spaced(body: View) -> impl Element {
          // A theme variable needs no closure: the style follows a theme switch.
          column(body).style(style().padding(space::LG).gap(space::SM))
      }

      fn main() -> kivact::Result<()> {
          App::new("Theme variables").run(|| card(spaced(text("Hello"))))
      }
      ```
      節の最初の文を「Theme variables resolve against the theme they are drawn in, so they follow a switch for free:」にし、箇条を次の 3 つにする：
      - 「The color variables in `color`: `SURFACE`, `SURFACE_ALT`, `TEXT`, `TEXT_MUTED`, `ACCENT`, `ACCENT_TEXT`, `BORDER`, `FOCUS`, `DANGER`, `SUCCESS`, `WARNING`, `OVERLAY`, `SHADOW`, `SELECTION`, `HOVER`, `PRESSED` and `RAISED`. `space::XS` to `space::XL`, `radius::SM` to `radius::LG` and `font::SIZE`, `font::SIZE_SMALL` and `font::SIZE_LARGE` are lengths. `rgb(0x2563eb)` and `rgba(..)` give fixed colors.」
      - 「Other theme values are read with accessors: `theme.focus_ring()`, `theme.focus_ring_offset()`, `theme.disabled_opacity()` and `theme.control_height()`. `theme.get(&color::ACCENT_VAR)` reads the value of a variable, for example to draw on a canvas.」
      - 「`use_theme()` returns the theme in effect; read it inside a style closure when a style depends on more than variables.」
    - 「Change tokens」の節を「Set theme variables」に改題し、code block の `fn brand()` を次にする（`use kivact::style::theme::Palette;` を消す）：
      ```rust
      fn brand() -> Theme {
          Theme::light()
              .with_focus_ring(3.0)
              .with_focus_ring_offset(2.0)
              .set(&color::ACCENT_VAR, rgb(0x7c3aed))
              .set(&color::FOCUS_VAR, color::ACCENT)
              .with_disabled_opacity(0.4)
      }
      ```
      最初の文を「`set(&VAR, value)` returns a new theme with one variable changed and keeps the rest; the value may be another variable, as `FOCUS` follows `ACCENT` here:」にする。後ろの段落の「The same works for `with_space`, `with_radius`, `with_text` (type sizes and the default font family), `with_control_height` and `with_icons`.」を「The same works for the `space`, `radius` and `font` variables (`space::MD_VAR` and so on). `with_control_height` and `with_icons` set the other theme values.」にする。
    - 「Change how one standard component looks」：code block を次にする：
      ```rust
      use kivact::prelude::*;
      use kivact::ui::theme::ButtonTheme;

      fn square_buttons() -> Theme {
          // `base` is the style the theme's recipe made; return it changed.
          Theme::light().with(ButtonTheme::default().root(|_theme, _look, base| base.border_radius(0)))
      }

      fn main() -> kivact::Result<()> {
          App::new("Square").theme(square_buttons()).run(|| button("Save"))
      }
      ```
      節の最初の段落を「Theme variables change everything at once. To change one part of one component, wrap its **recipe**: a function from the theme, the part's look and the style made so far to a `Style`. Each standard component has a table in `kivact::ui::theme`, such as `ButtonTheme`, installed with `Theme::with`.」にする。箇条の 1 つ目を「A method named after a part (`root` and the other parts of the table) adds your function after the theme's recipe for that part: `base` is the style the theme made, and `look` holds the part's state and variant.」に、2 つ目を「Build the theme **once** (at start, or with `kivact::style::cached_theme(square_buttons)`, which builds it on first use and returns the same theme afterwards). A recipe table is compared by identity, so a theme rebuilt on every run restyles everything under it each time.」にする。checkbox・switch の parts の段落の recipe 名（`checkbox_check`、`checkbox_mixed`、`checkbox_indicator`、`switch_thumb`、`switch_track`、radio の `mark`）を `CheckboxTheme`・`SwitchTheme`・`RadioGroupTheme` の part の名前（付属書 U の 4 章）にする。
    - 「Variants」：`kivact::ui::Variant` → `kivact::ui::theme::Variant`。
    - 「Hover, pressed, focus and disabled looks」を「Hover, active, focus and disabled looks」に改題する。code の `ColorToken::*` → `color::*`、`.pressed(` → `.active(`、`.ring(2, ColorToken::Focus)` → `.outline(2, color::FOCUS)`。箇条の `pressed` を `active` に、`ring_none()` を `outline_none()` にする。「A state style changes paint only (background, border, ring, radius, opacity, shadow, transform, cursor, filters, color); anything else set in it is reported and ignored. `disabled` and `selected` can change any property.」を「A state style can set any property, including layout ones such as padding or width; the node is laid out again when the state changes.」にする。
    - 「Icons」：`ColorToken::Warning` → `color::WARNING`。
    - 「Add your own tokens」を「Add your own theme variables」に改題し、code block を次にする（info 文字列は `rust,test_harness`）：
      ```rust
      use kivact::prelude::*;
      use kivact::style::{Length, ThemeVar};

      static HIGHLIGHT_VAR: ThemeVar<Color> = ThemeVar::new("brand.highlight", rgb(0xff8800));
      static GUTTER_VAR: ThemeVar<f64> = ThemeVar::new("brand.gutter", 12.0);
      const HIGHLIGHT: Color = Color::var(&HIGHLIGHT_VAR);
      const GUTTER: Length = Length::var(&GUTTER_VAR);

      #[component]
      fn banner() -> impl Element {
          text("New").style(style().color(HIGHLIGHT).padding(GUTTER))
      }

      fn main() -> kivact::Result<()> {
          let theme = Theme::light().set(&HIGHLIGHT_VAR, rgb(0x0088ff)).set(&GUTTER_VAR, 16.0);
          App::new("Brand variables").theme(theme).run(banner)
      }

      #[cfg(test)]
      mod tests {
          use super::*;

          #[test]
          fn a_set_value_wins_and_an_unset_one_falls_back() {
              let themed = Theme::light().set(&GUTTER_VAR, 16.0);
              assert_eq!(themed.get(&GUTTER_VAR), 16.0);
              assert_eq!(Theme::dark().get(&GUTTER_VAR), 12.0);
          }
      }
      ```
      節の最初の文を「Share your own colors and sizes through the theme with your own theme variables:」にし、箇条を次の 3 つにする：「A variable is a `static` `ThemeVar` (a name and a fallback) and a `const` `Color` or `Length` that points to it. Use the `const` anywhere a value goes.」「`theme.set(&VAR, value)` gives it a value in one theme; a theme that never sets it uses the fallback, which may be another variable.」「Tables of functions, such as recipes, stay `ThemeExtension`s installed with `Theme::with`. See [Write reusable components](custom-components.md).」
    - 「See also」の「`Theme`, `Style` and `ColorToken`; `cargo doc -p kivact-ui --open` for `UiRecipes`」を「`Theme`, `Style` and `ThemeVar`; `cargo doc -p kivact-ui --open` for `kivact::ui::theme`」にする。
  - **i18n.md**：`use kivact::style::Direction;` → `use kivact::style::FlexDirection;`、`.direction(Direction::RowReverse)` → `.flex_direction(FlexDirection::RowReverse)`、本文の `Direction::RowReverse` → `FlexDirection::RowReverse`。`slider("Volume", signal(30.0))` → `slider(signal(30.0)).label("Volume")`。「read `kivact::view::use_direction().get()` once」→「read `kivact::view::use_direction()` once」。
  - **animation.md、timelines-and-scroll.md、3d-and-depth.md、shaders.md、drawables.md**：表の置換と 3.0.4 だけ（`ColorToken::*` → `color::*`、`.radius(` → `.border_radius(`）。shaders の「(`GpuStart::OnDemand`)」の文は残す（`GpuStart` は `kivact::platform` にある。path を書いていればそれを `kivact::platform::GpuStart` にする）。
  - **effects.md**：`.weight(` → `.font_weight(`、`.radius(` → `.border_radius(`、`ColorToken::*` → `color::*`。「Colors are theme-dependent: `flood` and `drop_shadow` take a `ThemeColor`, so a graph follows the active theme.」を「Colors may be theme variables: `flood` and `drop_shadow` take a `Color`, so `color::ACCENT` makes a graph follow the active theme.」にする。`.composite(..)` が `Layer` のものなら `.mix(Mix::Composite(..))` にする（filter graph の primitive の `composite` は名前を変えない）。`Filter` などの CSS の値の型の import を `kivact::draw::` から `kivact::style::` に直す。
  - **canvas.md**：`ColorToken::*` → `color::*`。`use kivact::draw::{BorderImage, Composite, Layer};` → `use kivact::draw::{Composite, Layer, Mix};` と `use kivact::style::BorderImage;`（`Composite` の正準の path は付属書 S の 4 章に合わせる）。`Layer::new().composite(Composite::SourceIn)` → `Layer::new().mix(Mix::Composite(Composite::SourceIn))`。本文の「`composite(Composite::…)` chooses how the layer combines with what is already ...」を「`mix(Mix::Composite(..))` chooses how the layer combines with what is already ...; `Mix::Blend(BlendMode::..)` blends it instead. One layer has one mix.」にする。`action(ActionMode::Latest, |(width, height): (u32, u32)| ..)` → `action(|(width, height): (u32, u32)| ..)`。その中の `Offscreen::new().render(width, height, artwork)` と「Write a drawing as SVG」の節の `Offscreen` の呼出しを付属書 C の 4 章の `Offscreen` の署名（`render(width: f64, height: f64)`、`render_svg(width, height)`、`.scale(f64)`）に合わせる。`Part::new(().into_value(), ..)` は付属書 U の 4 章の `Part` の作り方に合わせる。「Write a drawing as SVG」の本文で「unsupported」と説明している機能の一覧を「Features that SVG 1.1 and CSS can express (font features and variations, `mix-blend-mode`, text decorations, fills and strokes) are written; only a feature with no SVG or CSS equivalent fails the export with `Unsupported`.」にする（DEC-11）。
- **消すもの**：上の各ファイルの旧名と、theming の `Palette`・`UiRecipes`・`ThemeExtension`（色の例）の説明。
- **状態の閉じ方**：新しい型なし。
- **試験**：9 ファイルの doctest。theming の `a_set_value_wins_and_an_unset_one_falls_back` は旧 `an_installed_value_wins_and_a_missing_one_falls_back` の置き換え（同じ契約：設定した値が勝ち、無い theme は fallback）。
- **確認（tier a）**：
  ```bash
  F=for-kivact-user/en/how-to-guide
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_DOCS_ONLY=$F/theming.md,$F/i18n.md,$F/animation.md,$F/timelines-and-scroll.md,$F/3d-and-depth.md,$F/effects.md,$F/shaders.md,$F/canvas.md,$F/drawables.md KIVACT_TEST_MIRROR=off \
    cargo test -p kivact-docs --doc --all-features -- --test-threads=1
  cargo xtask check --no-write --only docs-pairs
  ```
- **完了条件**：doctest が通る。3.0.6 の二つのコマンドを 9 ファイル（英日）に当てて何も返らない。docs-pairs の出力にこの 9 組が出ない。

## G07：非同期・品質・実験的な機能のガイド（async-and-tasks、http-json、platform-services、testing、diagnostics、devtools、embed、foreign-windows、gpu-texture-sharing）

- **wave / lane / 前提**：W11b / G4 / G01・G14（W11a）
- **所有**：`docs/for-kivact-user/{en,jp}/how-to-guide/{async-and-tasks,http-json,platform-services,testing,diagnostics,devtools,embed,foreign-windows,gpu-texture-sharing}.md`
- **目的**：`action`/`queued_action`、`on_complete(Result)`、`remote`/`remote_trigger`、`async_view` の retry、`TestFonts::Only` の削除、Python の削除、embed の通知と名前、GPU の型の移動を教える（API-08、RC-35、RC-37、USR-02、USR-03、USR-12、USR-15、API-10、API-11、TL-09、AC-5）。
- **手順**：3.0.5 の手順。ファイルごとの注：
  - **async-and-tasks.md**
    - `action(ActionMode::Latest, |name: String| ..)` → `action(|name: String| ..)`。`text_input("Name", name)` → `text_input(name).label("Name")`、`"Search"`・`"Draft"` も同様。
    - 「The first argument of `action` says what a new `run` does while one is still going (see below); `ActionMode::Latest` cancels the old one.」を「A new `run` while one is still going cancels the old one (see below).」にする。
    - 「What happens when the user clicks twice」の表を次の 2 行にする：
      ```markdown
      | Constructor | Behaviour |
      |---|---|
      | `action(f)` | The new run starts at once and cancels the old one. |
      | `queued_action(f)` | Runs go one at a time, in order; every run is kept. |
      ```
      その前の文を「Choose the constructor by what a new `run` does to one that is still going:」にする。その節の code block を次にする（info 文字列はそのまま）：
      ```rust
      # use kivact::prelude::*;
      # use kivact::task::queued_action;
      #[component]
      fn drafts() -> impl Element {
          let draft = signal(String::new());
          let save = queued_action(|text: String| async move {
              kivact::task::sleep(std::time::Duration::from_millis(500)).await;
              Ok(text.len())
          });
          let save = save.on_complete(|result: Result<usize, UiError>| match result {
              Ok(count) => println!("saved: {count}"),
              Err(error) => eprintln!("save failed: {error}"),
          });
          column((
              text_input(draft).label("Draft"),
              // Pressing this ten times quickly saves ten times, one after another.
              button("Save draft").on_press(move || save.run(draft.get())),
          ))
      }
      # fn main() -> kivact::Result<()> { App::new("Drafts").run(drafts) }
      ```
      後ろの段落の「Use `Queue` or `QueueLatest` for work that must finish ...」を「Use `queued_action` for work that must finish even if the user moves on, such as writing a file.」にする（続く文は残す）。「`on_complete(|outcome| …)`」を「`on_complete(|result| …)`, which gets a `Result<T, UiError>`,」にする。
    - `remaining.get_untracked()`（countdown）→ `remaining.get()`（`spawn` の中の async block は追跡しないので同じ意味）。
    - 「Receive values from another thread」の節：最初の段落を「If your own code produces values in the background (a game or audio engine, a file watcher, a network client on its own runtime), give it the sender of a `remote` signal. `remote(initial)` returns a `Signal<T>` for the UI and a `Remote<T>` that is `Send`, so it can go to any thread.」にし、code block を次にする（info 文字列はそのまま）：
      ```rust
      use std::sync::Arc;
      use std::sync::atomic::{AtomicBool, Ordering};
      use std::time::Duration;

      use kivact::prelude::*;
      use kivact::task::remote;

      #[component]
      fn sensor() -> impl Element {
          let (reading, sender) = remote(0u32);
          let stop = Arc::new(AtomicBool::new(false));

          // Your own thread, started in setup.
          let stopped = stop.clone();
          std::thread::spawn(move || {
              for n in 0.. {
                  if stopped.load(Ordering::Relaxed) {
                      break;
                  }
                  sender.set(n);
                  std::thread::sleep(Duration::from_millis(500));
              }
          });
          // Tell the thread to end when this component is removed.
          on_cleanup(move || stop.store(true, Ordering::Relaxed));

          text(move || format!("reading: {}", reading.get()))
      }
      # fn main() -> kivact::Result<()> { App::new("Sensor").run(sensor) }
      ```
      その後ろの 2 段落（「The channel's waker asks the UI to run a turn, ...」と「Do not call a blocking `recv()` ...」）を次の 2 段落にする：「`set` wakes the UI, so values show up even when the user is not touching the window. Writes are merged: the UI sees the latest value once per turn, however often the thread writes. `update(|value| ..)` changes the value in place from the thread. A write after the component is gone does nothing, so stop the thread with your own flag, as `on_cleanup` does here.」「Do not block the UI thread waiting for a worker, and do not hold a `Mutex` that a worker also takes for a long time. For continuous pictures from your own renderer, see [Draw with a canvas](canvas.md#show-frames-from-your-own-renderer).」。「Futures that bring their own reactor ...」の段落は「... send the results through a `remote` sender as above.」で終える。
    - 「Wake the UI without queueing values」の節：最初の段落を「When the worker only has to say "something changed, read the latest", it does not need to send a value at all. `remote_trigger()` returns a `Trigger` for the UI and a `RemoteTrigger` for the worker: `notify()` never blocks, and several notifications before the next turn count as one.」にし、code block を次にする：
      ```rust
      use std::sync::{Arc, Mutex};
      use std::time::Duration;

      use kivact::prelude::*;
      use kivact::task::remote_trigger;

      #[component]
      fn level_meter() -> impl Element {
          let shared = Arc::new(Mutex::new(0u32));
          let (changed, wake) = remote_trigger();

          // Your own thread, started in setup.
          let latest = shared.clone();
          std::thread::spawn(move || {
              for n in 0.. {
                  *latest.lock().expect("invariant: the level lock is not poisoned") = n;
                  wake.notify();
                  std::thread::sleep(Duration::from_millis(100));
              }
          });

          text(move || {
              changed.track();
              let level = *shared.lock().expect("invariant: the level lock is not poisoned");
              format!("level: {level}")
          })
      }
      # fn main() -> kivact::Result<()> { App::new("Level").run(level_meter) }
      ```
      その後ろの段落を「`notify` never waits, so the producer never blocks on a busy UI thread. It wakes the UI's event loop, which is a system call on most platforms: from a real-time audio callback, set your own lock-free flag and notify from a non-real-time thread.」にする。
  - **http-json.md**：「`async_view` shows loading, the result, or an error with Retry.」を「`async_view` shows loading, the result, or an error with a retry button.」にする。43 行付近の `Retry` の言及も同様に `retry` の `Callback<()>` の説明にする。
  - **platform-services.md**：`.font(` を使う code（font_list）は G04 の images-and-fonts の書き方（`Theme::with_fonts`）に合わせる。`on_close_request` を説明していれば G04 の windows の 2 文と同じ説明を足す。
  - **testing.md**
    - `text_input("New task", draft)` → `text_input(draft).label("New task")`。
    - `fn stepper(value: ..)`（ガイドの中の独自の部品）は、名前が prelude の外の `kivact::ui::stepper` と衝突しないが、`#[component]` にすると型名 `Stepper` を作る。3.0.4 の 9. のとおり、compile で衝突したら `stepper_view` にする。`value.get_untracked()` → `value.get()`（試験の中で読むだけ）。
    - 「Configure the environment」の「set `.fonts(TestFonts::Only(vec![Asset::from(bytes)]))` with a font file you ship, or `TestFonts::System` to use the machine's fonts (layout then depends on the machine)」を「start the test with `RenderOptions::with_theme(theme.with_fonts(FONTS))` and the font files you ship, or set `.fonts(TestFonts::System)` to use the machine's fonts (layout then depends on the machine)」にする。
  - **diagnostics.md**：`button("Retry").on_press(move || reset.run())` と試験の `get_by_role(Role::Button, "Retry")` の文字列 `"Retry"` を両方 `"Try again"` にする（AC-4 の検査で旧名 `Retry` と区別するため。同じ試験の同じ文字列なので期待値の意味は変わらない）。`reset.run()` が `Callback<()>` になっていれば `reset.call(())`、または `.on_press(reset)` にする。「let one side read with `get_untracked`」の文を「let one side read with `untrack(|| ..)`」にする。
  - **devtools.md**（TL-09、DEC-10）
    - 冒頭の段落の「The same protocol is available to MCP tools and to Python scripts.」を「The same protocol is available to MCP tools.」にする。
    - 「Install the tools」の Python の 2 段落（「The Python client installs with pip. ...」の段落と、その sh block、「Only when you edit the Kivact repository itself, use `pip install -e python/kivact-devtools`.」）を消す。
    - 「Script DevTools with Python」の節を見出しごと全部消す（Scripts panel の段落を含む。Scripts panel そのものは付属書 T の T23 が消している）。ほかの節に Scripts panel の言及（`rg -n -i 'scripts panel\|\.kivact/devtools/scripts' docs/for-kivact-user/en/how-to-guide/devtools.md`）があれば、その文も消す。
    - その他の `Python`・`python` の言及（`rg -n -i python docs/for-kivact-user/en/how-to-guide/devtools.md`）を消すか「MCP」に置き換える。「the scope select」「`"scope"`」は DevTools の UI と JSON の言葉なので残す。
  - **embed.md**
    - 表の「the editor stopped (parent destroyed, failure, panic) | `Parent::on_notice` gives `EmbedNotice::Stopped`; read `status()`.」を「`Parent::on_notice` gives `EmbedNotice::Stopped(status)` once, with the `EmbedStatus` (`Closed`, `Failed` or `Panicked`).」にする。
    - `EmbedNotice::Stopped` に match している code を `EmbedNotice::Stopped(status)` にする（`status()` を読み直していた行は、payload の `status` を使う形にする）。`fn alive(status: EmbedStatus)` は残す。
    - `embed::Visibility` を使っていれば `embed::EmbedVisibility` にする。`#[cfg(any(windows, target_os = "linux"))]` で包んでいる doctest の行（`# #[cfg(..)]`）は消す（`kivact::embed` は全 OS で compile できる）。本文に「macOS does not compile」などの説明があれば「`App::embed` returns `ErrorKind::Unsupported` where embedding is not available (macOS, Wayland)」にする。
    - `slider("Gain", gain)` → `slider(gain).label("Gain")`。
    - `prefer_gpu` があれば `GpuPreference { start: GpuStart::Never, .. }` の説明にする。
  - **foreign-windows.md**：`child.get_untracked()` → `child.get()`。
  - **gpu-texture-sharing.md**：`use kivact::launch::{GpuBackend, GpuPower, GpuPreference, GpuStart};` → `use kivact::platform::{GpuBackend, GpuPower, GpuPreference, GpuStart};`。`.grow(` → `.flex_grow(`。`GpuStart` の説明の段落の最後に「`GpuStart::Never` keeps the window on the CPU. The environment variable `ARVANE_RENDERER` overrides the choice at run time: `cpu`, `gpu` and `auto` mean `Never`, `AtLaunch` and `OnDemand`.」を足す。
- **消すもの**：devtools の Python の節と pip の手順、上の各ファイルの旧名。
- **状態の閉じ方**：新しい型なし。
- **試験**：9 ファイルの doctest。async-and-tasks の `saving_shows_pending_then_the_result`、diagnostics の Retry を押す試験（文字列だけ `"Try again"` に変える）、testing の全試験。
- **確認（tier a）**：
  ```bash
  F=for-kivact-user/en/how-to-guide
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_DOCS_ONLY=$F/async-and-tasks.md,$F/http-json.md,$F/platform-services.md,$F/testing.md,$F/diagnostics.md,$F/devtools.md,$F/embed.md,$F/foreign-windows.md,$F/gpu-texture-sharing.md KIVACT_TEST_MIRROR=off \
    cargo test -p kivact-docs --doc --all-features -- --test-threads=1
  cargo xtask check --no-write --only docs-pairs
  ```
- **完了条件**：doctest が通る（gpu-texture-sharing は `--all-features` のときだけ compile される）。3.0.6 の二つのコマンドを 9 ファイル（英日）に当てて何も返らない。`rg -n -i python docs/for-kivact-user/*/how-to-guide/devtools.md` が何も返さない。`rg -n -w Retry docs/for-kivact-user` が何も返さない。docs-pairs の出力にこの 9 組が出ない。

## G08：独自の control のガイド（custom-controls）

- **wave / lane / 前提**：W11b / G2 / G06
- **所有**：`docs/for-kivact-user/{en,jp}/how-to-guide/custom-controls.md`
- **目的**：interaction の behavior から独自の control を `#[component]` で作る書き方に直す。part の状態は `Part::state()` だけ、開閉は `use_x(.., open)` と `surface`（UI-01、UI-12、UI-24、API-01、AC-5）。
- **手順**：3.0.5 の手順。interaction の API の正本は付属書 U の 4 章の対応表。
  1. 「The pattern: controller, Part, node」の `pill_switch` を次にする（`app` と試験はそのまま）：
     ```rust
     use kivact::interaction::use_switch;
     use kivact::prelude::*;
     use kivact::style::FlexDirection;

     /// A pill with a knob that slides to the right when on.
     #[component]
     pub fn pill_switch(label: Value<String>, on: Signal<bool>) -> impl Element {
         let api = use_switch(on);
         let control = api.control();
         let state = control.state();
         node()
             .part(control)
             .name(label)
             .style(move || {
                 let state = state.get();
                 let on = state.checked.is_checked();
                 style()
                     .flex_direction(FlexDirection::Row)
                     .justify_content(if on { Justify::End } else { Justify::Start })
                     .size(44, 24)
                     .padding(3)
                     .border_radius(12)
                     .background(if on { color::ACCENT } else { color::BORDER })
                     .opacity(if state.input.is_pressed() { 0.8 } else { 1.0 })
             })
             .child(node().style(style().size(18, 18).border_radius(9).background(color::SURFACE)))
     }
     ```
     本文の「`Binding`」を「`Signal`」に、「`api.control_state()` is a `Value` that follows the real target」を「`control.state()` is a `Value` that follows the real target」にする。三つの規則の 2. の「`api.control_state()`」を「`Part::state()`」にする。
  2. ファイル全体で `api.<x>_state(..)` を `api.<x>(..).state()` にする（`control_state()` → `control().state()`、`item_state(key)` → `item(key).state()`）。Part を node に付ける前に `state()` を取る（上の `pill_switch` の順）。
  3. `pub fn <name>(label: impl IntoText, ..: impl IntoBinding<T>) -> impl Element` の形の関数（`segmented`、`dropdown` など）を 3.0.4 で `#[component]` にする。generic な関数（`<K: ..>`）は 3.0.4 の 7. に従う。本体の `scope(|| ..)`・`scope(move || ..)` の包みを外す。`.with_node(|host| host.part(..))` は本体の root が `node()` なら `.part(..)` を直接呼ぶ。
  4. 「Open and close a list」の節：`use_select(options, value)` などの開閉の behavior を、付属書 U の 4 章の署名（`open: Signal<bool>` を位置引数に取る）にする。`MenuApi::while_open` の説明の文（「content with `MenuApi::while_open` instead of `show`」）を「content with the Api's `surface(placement, content)`, which wires the anchor, the opening and the content Part」にする。
  5. 「Drag to change a value」：`value.get_untracked()` を `value.get()` にする（`source_with` と `edit` の closure は handler の中で呼ばれ、追跡しない）。「Use `.source_with(move || value.get_untracked())`」の本文も同様にする。
  6. 「Edit text with your own frame」：`fn limited_input(label: impl IntoText, value: impl IntoBinding<String>, max: usize)` を `#[component] fn limited_input(label: Value<String>, value: Signal<String>, max: usize) -> impl Element` にする。`.grow(1)` → `.flex_grow(1)`。`text_area(` の呼出しがあれば `.label(..)` を付ける。「Paint native editing text」の本文の「Reactive content」は一般語なので「Content that changes」にする。
  7. 「Write behavior with a state machine」：`fn copy_button(text: impl IntoValue<String>) -> impl IntoView` を `#[component] fn copy_button(text: Value<String>) -> impl Element` にし、`let text = text.into_value();` と `scope(move || { .. })` の包みを外す。その後ろの本文の「its own mount lifetime, convert `text` once, then return `scope(move || { ... })`」の文を「A component body runs once per mount, so the Machine belongs to this instance.」にする。
  8. `ColorToken::*` → `color::*`、`.radius(` → `.border_radius(`、`Direction` → `FlexDirection`。
- **消すもの**：`scope(..)`、`IntoBinding`、`IntoText`、`into_value`、`into_text_value`、`*_state()` の getter、`while_open`、`ColorToken`。
- **状態の閉じ方**：新しい型なし。
- **試験**：custom-controls の全試験（`the_part_brings_the_role_the_keys_and_the_state`、`a_disabled_switch_ignores_input`、ほかファイル内の `#[test]` 全部）。期待値を変えない。
- **確認（tier a）**：
  ```bash
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_DOCS_ONLY=for-kivact-user/en/how-to-guide/custom-controls.md KIVACT_TEST_MIRROR=off \
    cargo test -p kivact-docs --doc --all-features -- --test-threads=1
  cargo xtask check --no-write --only docs-pairs
  ```
- **完了条件**：doctest が通る。3.0.6 の二つのコマンドを英日の custom-controls.md に当てて何も返らない。`rg -n '_state\(' docs/for-kivact-user/*/how-to-guide/custom-controls.md` が何も返さない。

## G09：移行ガイドと利用者向け文書の入口

- **wave / lane / 前提**：W11b / G1 / G02
- **所有**：`docs/for-kivact-user/en/how-to-guide/migrating-to-rework.md`（新規）、`docs/for-kivact-user/jp/how-to-guide/migrating-to-rework.md`（新規）、`docs/for-kivact-user/{en,jp}/README.md`、`docs/for-kivact-user/{en,jp}/how-to-guide/README.md`
- **目的**：利用者向けの移行ガイドを作る（AC-5、USR-01、USR-13）。入口の文書を新しい API、profile の案内、版の固定の案内に直す。how-to の索引から Python を消し、移行ガイドと plot のガイドを足す（TL-09、KIT-15）。
- **手順**：
  1. `docs/for-kivact-user/en/how-to-guide/migrating-to-rework.md` を作り、次の (A)、3.0.3 の表（`## Components` から `## Kits and plots` の表の終わりまで、そのまま）、(B) をこの順に書く。
     (A)：
     ````markdown
     # Migrate an app to the reworked API

     [日本語](../../jp/how-to-guide/migrating-to-rework.md)

     Kivact's public API was reworked in one breaking change. There is no
     compatibility layer: every old name below is gone, and each row gives what to
     write instead. The tables are grouped by the module you import from. The
     [tutorial](../getting-started/README.md) and the other guides already use the
     new API.

     ## Before you start

     **Pin every repository that uses Kivact to one revision.** Use the same `rev`
     in the git dependency of all your apps and component libraries, and update them
     together. Two revisions of Kivact in one build are two different crates, and
     their types do not mix.

     ```toml
     [dependencies]
     kivact = { git = "https://github.com/Kashiwade-music/kivact", rev = "<commit>" }
     ```

     **Optimize dependencies in debug builds with one line.** If your `Cargo.toml`
     lists crates of Kivact under `[profile.dev.package.<name>]`, delete those
     sections: the crates inside Kivact are split and merged when it changes. Write
     this instead:

     ```toml
     [profile.dev.package."*"]
     opt-level = 3
     ```

     Your own crates stay unoptimized and easy to debug; everything they depend on,
     Kivact and its renderer included, is optimized.

     **Work from the top of this page down.** Fix components first, then state,
     then styles: most compiler errors in the later tables disappear once the
     components have their new shape.
     ````
     (B)：
     ````markdown
     ## See also

     - [Write reusable components](custom-components.md): the one way to write a component.
     - [Map what you know from React or SolidJS to Kivact](coming-from-react-or-solid.md): the new API in one page.
     - [Change the theme and the look of components](theming.md): theme variables and component themes.
     ````
  2. `docs/for-kivact-user/jp/how-to-guide/migrating-to-rework.md` を作る：(A)・表・(B) の本文と表の「After」の説明の文を日本語に訳す。表の左右の列のコード（backtick の中）は英語と同じにする。見出しは「# 改修後の API へアプリを移行する」、言語の link は `[English](../../en/how-to-guide/migrating-to-rework.md)`。Rust の block は無い（toml だけ）。
  3. `docs/for-kivact-user/en/README.md`：
     - Rust の block の `fn counter() -> impl IntoView {` を `#[component]` の行と `fn counter() -> impl Element {` の 2 行にする（本体と `main` は変えない）。
     - 「> notice. Pin a commit if you need a stable build.」の文を「> notice. Pin every repository that uses Kivact to one revision (the same `rev`) if you need a stable build.」にする。
     - 「## Install」の最初の `toml` block の直後に、次を足す：
       ````markdown
       Kivact's layout and drawing code is too slow without optimization, so
       optimize your dependencies in debug builds. Do not list Kivact's own crates
       one by one: they change between revisions.

       ```toml
       [profile.dev.package."*"]
       opt-level = 3
       ```
       ````
     - GPU の段落の「To force CPU rendering at run time, set the environment variable `ARVANE_RENDERER=cpu`; `ARVANE_RENDERER=gpu` starts the device with the window.」を「The environment variable `ARVANE_RENDERER` chooses at run time: `cpu` never starts the GPU, `gpu` starts it with the window and `auto` starts it when needed, as by default. In code, `kivact::platform::GpuStart` (`Never`, `AtLaunch`, `OnDemand`) in `App::gpu_preference` makes the same choice.」にする。
     - 「The `[profile.release]` of `templates/basic/Cargo.toml` (...)」の文の `templates/basic/Cargo.toml` を `templates/basic/README.md` にする。
     - 「## Where to start」の表の how-to-guide の行の後に次の行を足す：`| [Migrating](how-to-guide/migrating-to-rework.md) | Old names and their replacements, for code written before the API rework. |`
  4. `docs/for-kivact-user/en/how-to-guide/README.md`：
     - 最初の表（React・Solid の行だけの表）に 2 行目として `| [Migrate an app to the reworked API](migrating-to-rework.md) | Every renamed and removed name with its replacement, and the Cargo settings to change |` を足す。
     - 「Look and language」の theming の行の説明を `Light and dark themes, theme variables, component themes, variants, state looks, icons and your own theme variables` にする。
     - 「Graphics」の表の最後に `| [Plot numeric data](plot.md) | XY plots the user can zoom and pan, for any kit, and exporting them as an image or SVG |` を足す（ファイルは G10 が作る）。
     - 「Quality」の devtools の行の「review export, MCP and Python」を「review export and MCP」にする。
  5. 日本語の `README.md` と `how-to-guide/README.md` に同じ変更を日本語で行い、Rust の block を 3.0.2 の script で写す（`docs/for-kivact-user/en/README.md` → `jp/README.md`）。
- **消すもの**：索引の「Python」、入口の README の「Pin a commit」の文。
- **状態の閉じ方**：新しい型なし。
- **試験**：`docs/for-kivact-user/en/README.md` の doctest。
- **確認（tier a）**：
  ```bash
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_DOCS_ONLY=for-kivact-user/en/README.md,for-kivact-user/en/how-to-guide/README.md,for-kivact-user/en/how-to-guide/migrating-to-rework.md KIVACT_TEST_MIRROR=off \
    cargo test -p kivact-docs --doc --all-features -- --test-threads=1
  cargo xtask check --no-write --only docs-pairs
  ```
- **完了条件**：doctest が通る。`migrating-to-rework.md` が英日にあり、3.0.3 のすべての表（Components、State、Event handlers、Views and elements、Standard components、Async、App and platform、Embedding、Styles and themes、Drawing、Kits and plots）を持つ。`rg -n -i python docs/for-kivact-user/*/how-to-guide/README.md` が何も返さない。設計 5.12 の名前が `rg -w` で一つずつ `docs/for-kivact-user/en/how-to-guide/migrating-to-rework.md` に見つかる（`Binding`、`IntoBinding`、`IntoReactive`、`Reactive`、`Notifier`、`notifier`、`ActionOutcome`、`ActionMode`、`Retry`、`AsyncSnapshot`、`constant_value`、`AppOptions`、`ThemeColor`、`ColorToken`、`ColorName`、`Palette`、`Spacing`、`Radii`、`UiRecipes`、`UiTokens`、`use_tokens`、`ThemeHint`、`Dimension`、`TextLen`、`GridFlow`、`Insets`、`SpanStyle`、`TextExtras`、`DecorationStyleValue`、`prefer_gpu`、`use_menu_with_open`、`on_open_change`、`on_step_press`、`on_draft_commit`、`selection_field`、`input_group_with`、`while_open`、`Popover`、`ViewTransform`、`toaster`、`announcer`、`Composite`、`weight`、`italic`、`radius`、`ring`、`basis`、`grow`、`shrink`、`wrap`、`direction`、`pressed`）。kit の型名（`BadgeHue` など）は「Kits and plots」の行の説明（`*Hue`、`*Tone`）で代表する。

## G10：kit のガイド（basic、pop、game）と plot のガイドの移動

- **wave / lane / 前提**：W11b / G4 / G07
- **所有**：`docs/for-kivact-ui-user/{en,jp}/basic/**`、`docs/for-kivact-ui-user/{en,jp}/pop/**`、`docs/for-kivact-ui-user/{en,jp}/game/**`、新規 `docs/for-kivact-user/{en,jp}/how-to-guide/plot.md`
- **目的**：3 つの kit のガイドを移行後の kit の API（付属書 K の 4 章）と中核の新しい API に直す。日本語は全ページ残す（KIT-16 の判定）。plot は `kivact-plot` に移ったので、ガイドを利用者向けの how-to に移す（KIT-15）。
- **手順**：
  0. 対象のファイル（英語。日本語は同じ相対 path の `jp/`。各 kit の root は `docs/for-kivact-ui-user/en/<kit>/`）：
     - basic：`getting-started/{README,01-set-up-the-kit,02-build-the-settings-form,03-dark-mode-and-feedback}.md`、`how-to-guide/{README,data-display,forms,navigation,overlays,theme-and-tokens}.md`（`how-to-guide/plot.md` は手順 2 で移す）
     - pop：`getting-started/{README,01-set-up-the-kit,02-build-the-form,03-validate-and-submit}.md`、`how-to-guide/{README,dates-and-calendars,messages-and-notifications,status-and-figures,tables,theme-and-tokens}.md`
     - game：`getting-started/{README,01-set-up-the-kit,02-build-the-panel,03-tabs-and-items}.md`、`how-to-guide/{README,choices-and-toolbars,effects-and-depth,item-lists-and-selection,meters-and-status,theme-and-density}.md`
  1. 付属書 K の 4 章の「kit ごとの旧名→新名」の表を開く。以後、kit の名前はこの表を正本とする。
  2. plot の移動：
     ```bash
     git mv docs/for-kivact-ui-user/en/basic/how-to-guide/plot.md docs/for-kivact-user/en/how-to-guide/plot.md
     git mv docs/for-kivact-ui-user/jp/basic/how-to-guide/plot.md docs/for-kivact-user/jp/how-to-guide/plot.md
     ```
     英語の `plot.md` を次のとおり直す：
     - 言語の link を `[日本語](../../jp/how-to-guide/plot.md)` にする。
     - 最初の段落の「with `kivact_ui_basic`」を「with the `kivact-plot` crate, which works with any kit and without one」にし、「It assumes you have done the [tutorial](../getting-started/README.md).」はそのまま（link 先が利用者のチュートリアルになる）。
     - 最初の段落の後に次を足す：
       ````markdown
       Add the crate next to `kivact`:

       ```toml
       [dependencies]
       kivact-plot = { git = "https://github.com/Kashiwade-music/kivact" }
       ```
       ````
     - Rust の block の `use kivact_ui_basic::{..}` をすべて `use kivact_plot::{..}` にする（中の名前は付属書 K の 4 章の kivact-plot の行に合わせる）。
     - 「A series without `color` takes the kit's chart color of its index」の文を「A series without `color` takes the series color of its index, the theme variables `kivact_plot::color::SERIES_1` to `SERIES_5`; a kit's theme sets them to its own chart colors.」にする。
     - 3.0.4・3.0.6 を当てる。
     - 日本語の `plot.md` も同じ変更を日本語で行い、言語の link を `[English](../../en/how-to-guide/plot.md)` にして、3.0.2 で Rust の block を写す。
     - `docs/for-kivact-ui-user/{en,jp}/basic/how-to-guide/README.md` の表から plot の行を消す。basic の他のページから `plot.md` への link があれば（`rg -n 'plot\.md' docs/for-kivact-ui-user`）、`../../../../for-kivact-user/en/how-to-guide/plot.md`（日本語は `.../jp/...`）に直す。
  3. kit ごとに（basic、pop、game の順）、英語の全ファイルに対して：
     - 付属書 K の 4 章の表の左の列の名前を `rg -n -w` で探し、右の列に直す。続いて 3.0.6 のコマンドで中核の旧名を探し、3.0.3 で直す。
     - 3.0.4 で view を返す関数を `#[component]` にする。
     - theme のページ（basic・pop は `how-to-guide/theme-and-tokens.md`、game は `how-to-guide/theme-and-density.md`）の、`Tokens` の field や `ColorName` を読む・書く節を、kit の theme 変数（`kivact_ui_basic::PRIMARY` などの `const` と `PRIMARY_VAR` などの `static`）を使う節にする：読むのは style に `kivact_ui_basic::PRIMARY` を置くだけ、変えるのは `kivact_ui_basic::theme(ThemeMode::Light).set(&kivact_ui_basic::PRIMARY_VAR, rgb(..))`。recipe を変える節は G06 の theming の `ButtonTheme` の形（`theme.with(<Name>Theme::default().<part>(|theme, look, base| ..))`）にする。kit の `ThemeOptions` と `theme_with(mode, options)` は、付属書 K の 4 章でその kit に残っているときだけ書く。ファイル名は変えない。
     - getting-started の「set up the kit」のページの `App::new(..).fonts(..)` があれば、kit の theme が font を持つので消す（K の 4 章が kit の font の扱いを書いていればそれに従う）。
     - kit の how-to の索引（`how-to-guide/README.md`）の説明の文の「tokens」を「theme variables」にする。
     - `KIVACT_DOCS_ONLY=for-kivact-ui-user/en/<kit>` で doctest を通す。
  4. 日本語の全ファイル（同じ相対 path）を英語の新しい本文の訳にし、3.0.2 で Rust の block を写す：
     ```bash
     for kit in basic pop game; do
       for en in $(git ls-files "docs/for-kivact-ui-user/en/$kit/*.md"); do
         perl /tmp/g-sync-rust-blocks.pl "$en" "${en/\/en\//\/jp\/}"
       done
     done
     ```
- **消すもの**：`docs/for-kivact-ui-user/{en,jp}/basic/how-to-guide/plot.md`（移動）。3 つの kit のガイドの kit の旧名と中核の旧名。
- **状態の閉じ方**：新しい型なし。
- **試験**：3 つの kit のガイドと plot.md の doctest。
- **確認（tier a）**：
  ```bash
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_DOCS_ONLY=for-kivact-ui-user/en/basic,for-kivact-ui-user/en/pop,for-kivact-ui-user/en/game,for-kivact-user/en/how-to-guide/plot.md KIVACT_TEST_MIRROR=off \
    cargo test -p kivact-docs --doc --all-features -- --test-threads=1
  cargo xtask check --no-write --only docs-pairs
  ```
- **完了条件**：doctest が通る。3.0.6 の二つのコマンドを `docs/for-kivact-ui-user/{en,jp}/{basic,pop,game}` と `docs/for-kivact-user/{en,jp}/how-to-guide/plot.md` に当てて何も返らない。付属書 K の 4 章の表の左の列の名前が、同じ範囲で `rg -w` に何も返らない。`rg -n 'kivact_ui_basic::.*plot|kivact_ui_basic::\{[^}]*Plot' docs` が何も返さない。`rg -n -w ButtonTone docs/for-kivact-ui-user/{en,jp}/game` が何も返さない（INC-39）。

## G11：kit のガイド（creative、soft）と kit の索引

- **wave / lane / 前提**：W11b / G4 / G10
- **所有**：`docs/for-kivact-ui-user/{en,jp}/creative/**`、`docs/for-kivact-ui-user/{en,jp}/soft/**`、`docs/for-kivact-ui-user/{en,jp}/README.md`
- **目的**：G10 と同じ（KIT-16、AC-5）。
- **手順**：
  0. 対象のファイル（英語。日本語は同じ相対 path の `jp/`）：
     - creative：`getting-started/{README,01-set-up-the-kit,02-edit-values,03-dock-and-outliner}.md`、`how-to-guide/{README,color-curves-and-gradients,knobs-and-faders,node-graph,panels-docks-and-outliners,show-app-rendered-images,theme-and-icons,timeline-and-arrangement}.md`
     - soft：`getting-started/{README,01-set-up-the-kit,02-build-the-settings-form,03-dark-mode-and-feedback}.md`、`how-to-guide/{README,data-display,forms,motion,navigation,overlays,theme-and-tokens}.md`
     - `docs/for-kivact-ui-user/en/README.md`
  1. G10 の手順 3・4 を creative と soft に対して行う。theme のページは creative が `how-to-guide/theme-and-icons.md`、soft が `how-to-guide/theme-and-tokens.md`。soft の theme のページの interaction の policy（`InteractionPolicy`）の説明は残し、`Tokens` の入れ子（`spacing`、`radius`、`colors`、`typography`）を読む節を soft の theme 変数の節にする（名前は付属書 K の 4 章）。creative の `show-app-rendered-images.md` の `ViewTransform`（interaction）は `ZoomPan` にする。
  2. `docs/for-kivact-ui-user/en/README.md`：「Before you start」以降で、kit の theme の説明に「tokens」「`ColorName`」があれば「theme variables」にする。Rust の block があれば 3.0.4・3.0.6 を当てる。plot について書いていれば「Plots are in the `kivact-plot` crate, which works with every kit: [Plot numeric data](../../for-kivact-user/en/how-to-guide/plot.md).」の 1 文にする（無ければ、kit の選び方の表の直後にこの 1 文を足す）。日本語も同様にし、3.0.2 で写す。
- **消すもの**：2 つの kit のガイドの kit の旧名と中核の旧名。
- **状態の閉じ方**：新しい型なし。
- **試験**：2 つの kit のガイドの doctest。
- **確認（tier a）**：
  ```bash
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_DOCS_ONLY=for-kivact-ui-user/en/creative,for-kivact-ui-user/en/soft,for-kivact-ui-user/en/README.md KIVACT_TEST_MIRROR=off \
    cargo test -p kivact-docs --doc --all-features -- --test-threads=1
  cargo xtask check --no-write --only docs-pairs
  ```
- **完了条件**：doctest が通る。3.0.6 の二つのコマンドと付属書 K の 4 章の旧名を `docs/for-kivact-ui-user` 全体に当てて何も返らない。docs-pairs の出力に `docs/for-kivact-ui-user` のファイルが出ない。

## G12：README、雛形、例の文書、crate の README

- **wave / lane / 前提**：W11b / G1 / G09
- **所有**：`README.md`、`README_JP.md`、`templates/basic/README.md`、`templates/basic/src/main.rs`（doc comment の行だけ）、`examples/README.md`、`examples/kivact/README.md`、`examples/kivact/*/README.md`、`examples/kivact/*/src/main.rs`（doc comment の行だけ）、`crates/*/README.md`
- **目的**：入口の文書を新しい API・54 package の構成・profile の案内に直す（API-07 の README の `scope`、API-09、KIT-16、設計 3 章、AC-5）。
- **手順**：
  1. `README.md`：
     - 最初の Rust の block を次にする：
       ```rust
       use kivact::prelude::*;

       #[component]
       fn counter() -> impl Element {
           let count = signal(0);
           column((
               text(move || format!("count: {}", count.get())),
               button("Increment").on_press(move || count.update(|n| *n += 1)),
           ))
           .style(style().gap(8).padding(16))
       }

       fn main() -> kivact::Result<()> {
           App::new("Counter").run(counter)
       }
       ```
       （info 文字列は `rust,no_run` のまま。）
     - その直後の段落（「The root setup runs once when the App mounts it. Use `scope(move || view)` ...」）を次にする：「A component is a function marked `#[component]`. Its body runs once when the component is mounted, and its state lives as long as it is on screen. Dynamic properties read signals inside closures. Runtime image inputs accept owned `Vec<u8>` or `AssetBytes`; keep an asset and clone it when reused so its cache identity stays stable. Fonts belong to the theme (`Theme::with_fonts`).」
     - 「## Features」の 1 つ目の箇条を「**Components and signals.** Components are functions marked `#[component]`. `signal`, `memo`, `effect` and keyed lists update only what changed, with no virtual DOM and no re-rendering of whole components. The standard components and the UI kits are written the same way.」にする。2 つ目の箇条の後に「**CSS names and theme variables.** Styles use CSS property names, and theme variables such as `color::ACCENT` and `space::MD` work like CSS custom properties.」を足す。
     - 「## Getting started」の最初の `toml` block の直後に次を足す：
       ````markdown
       Pin every repository that uses Kivact to the same revision (`rev = "<commit>"`
       in the git dependency) and update them together. Kivact's layout and drawing
       code is too slow without optimization, so optimize your dependencies in debug
       builds with one line instead of listing Kivact's crates:

       ```toml
       [profile.dev.package."*"]
       opt-level = 3
       ```

       Code written for the API before the rework: see
       [Migrate an app to the reworked API](docs/for-kivact-user/en/how-to-guide/migrating-to-rework.md).
       ````
     - 「## Workspace crates」の「The workspace contains 49 packages.」を「The workspace contains 54 packages.」にし、「see the [architecture guide](docs/for-kivact-developer/architecture.md) and [generated graphs](architecture/) for the full graph.」を「see the [architecture guide](docs/for-kivact-developer/architecture.md); `cargo xtask diagrams` generates the full crate and module graphs.」にする。mermaid の block を次にする：
       ````markdown
       ```mermaid
       flowchart TD
           Apps[Applications and kits] --> Facade[kivact]
           Plot[kivact-plot] --> Facade
           Facade --> UI[kivact-ui]
           UI --> Interaction[kivact-interaction]
           Interaction --> Core[kivact-core]
           Facade --> Core
           Facade --> Offscreen[kivact-offscreen]
           Offscreen --> Engine
           Core --> Runtime[kivact-runtime / kivact-style / kivact-i18n]
           Core --> Engine[kivact-engine]
           Engine --> Geometry[kivact-layout / kivact-editing]
           Engine --> Host[arvane]
           Engine --> Adapters[arvane-shell / arvane-text / arvane-asset / arvane-compose]
           Engine --> Native[arvane-native]
           Engine --> Virtual[arvane-virtual]
           Engine --> Protocol[arvane-protocol]
           Adapters --> Render[arvane-render]
           Render --> Protocol
           Facade -. test feature .-> Test[kivact-test]
           Test --> Core
           Test --> Virtual
           Runtime --> Types[kivact-types]
           Runtime --> Shader[kivact-shader]
           Protocol --> Types
       ```
       ````
     - crate の表：
       - `kivact-devtools-client` の行を消す。
       - `kivact-devtools-protocol` の Purpose を `Wire vocabulary shared by the DevTools agent, app and MCP server; the client feature adds discovery and typed calls.` にする。
       - `kivact-macros` の Purpose を `` `#[component]` and `#[derive(Prop)]` (re-exported by `kivact`). `` にする。
       - `kivact-style` の Purpose を `Styles with CSS names, themes, theme variables and the drawing vocabulary.` にする。
       - `kivact-ui` の Purpose を `Standard components and their component themes.` にする。
       - `kivact-svg` の Purpose を `` Writes a recorded drawing as an SVG 1.1 document (behind `Offscreen::render_svg`). `` のままにする。
       - `arvane` の Purpose を `Window driver and presenter; one GPU device per process.` にする。
       - `kivact-types` の Purpose を `Neutral values, geometry, events and small std-only helpers.` にする。
       - `xtask` の Purpose を `` Workspace checks; `cargo xtask diagrams` writes the dependency graphs. `` にする。
       - 次の 5 行を足す（`kivact-svg` の行の後、`arvane-asset` の行の後、`kivact-ui-soft` の行の後、`kivact-ui-soft-catalog` の行の後の順に）：
         - `` | `kivact-offscreen` | `crates/kivact-offscreen` | Drawing without a window: pixels and SVG documents (`kivact::draw::Offscreen`). | ``
         - `` | `arvane-protocol` | `crates/arvane-protocol` | The render scene and semantics the engine hands to the renderers. | ``
         - `` | `kivact-plot` | `crates/kivact-plot` | Numeric XY plots for any kit. | ``
         - `` | `kivact-catalog-shell` | `examples/catalog-shell` | The shared shell of the five kit catalogs. | ``
         - `` | `kivact-kit-tests` | `dev/tests/kivact-kit-tests` | Public-API E2E that needs the kits (`e2e_kits`). | ``（この行だけは `kivact-public-api-tests` の行の直後に置く。`kivact-public-api-tests` の Purpose は `Public consumer E2E (e2e_core, e2e_ui, e2e_native) and offscreen contracts.` にする）
       - crate の path と package 名を `cargo metadata --no-deps --format-version 1 | grep -o '"name":"[a-z0-9_-]*","version"'` の一覧と照らし、表の行が 54 package と一致することを確かめる（catalog の shell の package 名が `kivact-catalog-shell` と違えば、実際の名前を書く）。
     - 「## Cargo features of `kivact`」の表：`gpu` の行の「`ARVANE_RENDERER=cpu` forces the CPU renderer at run time;」を「`ARVANE_RENDERER=cpu|gpu|auto` chooses at run time (`GpuStart::Never`, `AtLaunch` or `OnDemand`);」にし、「choose `GpuStart::AtLaunch` with `App::gpu_preference`」を「choose with `kivact::platform::GpuStart` in `App::gpu_preference`」にする。`devtools` の行の「for the separate DevTools app, MCP server and Python client」を「for the separate DevTools app and MCP server」にする。`embed` の行を「Experimental. Open an app inside a host window, e.g. an audio plugin editor. Compiles on every platform; `App::embed` returns `ErrorKind::Unsupported` where embedding is not available.」にする。
     - 「## Repository layout」の表から `architecture/` の行を消す。
  2. `README_JP.md`：1. と同じ変更を日本語で行う（Rust の block は README.md と同一にする。3.0.2 の script で `perl /tmp/g-sync-rust-blocks.pl README.md README_JP.md`）。mermaid と toml の block は README.md と同じ内容にする。
  3. `templates/basic/README.md`：
     - 「## Debug-build performance」の節の `toml` block 全体を次にする：
       ```toml
       [profile.dev.package."*"]
       opt-level = 3
       ```
       その前の段落の「Kivact's workspace root optimizes them in debug builds, which covers this template while it is a member of Kivact's workspace.」の後に「Optimize every dependency, not a list of Kivact's crates: the crates inside Kivact change between revisions.」を足す。
     - 「## Release-build performance and size」の `toml` block から、package 名が `kivact-` または `arvane` で始まる `[profile.release.package.<name>]` の節（見出しの行と `opt-level` の行）をすべて消す。続いて `cargo tree -p kivact-template-basic -e normal --prefix none | sed 's/ v[0-9].*//' | sort -u > /tmp/g-template-deps.txt` を実行し、残った `[profile.release.package.<name>]` のうち `<name>` が `/tmp/g-template-deps.txt` に無いもの（resvg、tiny-skia、fontdb など、消えた crate）の節も消す。block の最初の comment と `[profile.release]` の節は残す。その段落の「opt-level 3 for the drawing, shaping and layout crates」を「opt-level 3 for the third-party drawing, shaping and layout crates」にする。
     - 3 行目以降の「The app uses `use kivact::prelude::*;` including the standard components of `kivact::ui`: ...」の段落に「The counter is a `#[component]`.」の 1 文を足す。
  4. `templates/basic/src/main.rs`：`counter` の doc comment（「/// A component is a function returning a view; `#[component]` also makes a builder for its props. The setup runs once; dynamic parts are closures over Copy handles.」の 3 行）を次の 2 行にする：
     ```rust
     /// A component: a function marked `#[component]`. Its body runs once;
     /// the parts that change are closures over Copy handles.
     ```
     先頭の `//! Role: ` で始まる行があれば `//! ` にする（文はそのまま）。コードは変えない。
  5. `examples/kivact/*/src/main.rs`：各ファイルの doc comment（`///`、`//!`）に 3.0.6 の旧名があれば新名に直す。`//! Role: ` で始まる行があれば `//! ` にする。component の例の `card` の doc comment は、実際の署名（`rg -n -A6 '#\[component\]' examples/kivact/component/src/main.rs` で確かめる）に合わせて「The body is a `ViewFn` ...」の説明を残す。コードは変えない。
  6. `examples/kivact/README.md`：表の async の行の「`resource` と `dynamic` による読込中・エラー・完了」を「`resource` と `async_view` による読込中・エラー・完了」にする。
  7. `examples/kivact/*/README.md`：
     - async：「`dynamic` が `AsyncState` に応じて読込中、エラー、完了を描き分けます。」を「`async_view` が読込中、エラー、完了を描き分けます。」にする。
     - component：本文を、実際の `card` の署名（手順 5 で確かめたもの）に合わせて次の形にする：「`#[component]` を付けた `card` は、題名（`Value<String>`）と開くたびに作り直す子（`ViewFn`）を位置引数で受け、任意の close の callback（`Option<Callback<()>>`）を `.on_close(..)` の setter で受けます。戻り値は `impl Element` なので、呼出し側が `.style(..)` を付けられます。本体は `show(open, content)` で折りたたみ、`app()` が別の子を渡して 2 回使います。"Close" は親の signal を更新して card を取り除きます。依存は `kivact` だけです。試験は `cargo test -p kivact-example-component` です。」
     - list：「行の checkbox は `binding` で一覧の signal に結びます。」を「行は `Signal<Todo>` を受け取り、checkbox は `bind!(todo.done)` で一覧の中の `done` に結びます。」にする（実際のコードが `bind!` を使っていることを `rg -n 'bind!' examples/kivact/list/src/main.rs` で確かめる。使っていなければ STOP：付属書 C の移行漏れ）。
  8. `examples/README.md`：変更は無い（確認だけ）。
  9. `crates/*/README.md`（5 つの kit の README）：
     - 「## Migrating from the previous release」の節が残っていれば、見出しから次の `## ` の見出しの直前まで消す（KIT-16）。
     - 「## Filled recipes」「## Filled standard recipes」の節（`<!-- kivact-recipes -->` の印を含む）が残っていれば同様に消す。
     - 残りの本文と Rust の block に、付属書 K の 4 章の旧名と 3.0.6 の旧名があれば直す。`Tokens`・`ColorName` を説明している文は kit の theme 変数の文にする。
     - xtask の検査が `<!-- kivact-recipes -->` を読んでいる場合（`rg -n kivact-recipes dev/ crates/ --glob '!*.md'`）、その検査が残っていれば STOP（付属書 K・T の削除漏れ）。
- **消すもの**：README の `scope(move || view)` の段落、`architecture/` の行、`kivact-devtools-client` の行、雛形の README の Kivact の内部 crate を並べた profile の節、kit の README の移行表と recipe の一覧。
- **状態の閉じ方**：新しい型なし。
- **試験**：README.md の doctest。雛形と例のコードは変えないので、その試験は変わらない。
- **確認（tier a）**：
  ```bash
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_DOCS_ONLY=README.md KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --doc --all-features -- --test-threads=1
  cargo check -p kivact-template-basic --all-targets
  cargo check -p kivact-example-counter -p kivact-example-form -p kivact-example-list -p kivact-example-async -p kivact-example-component --all-targets
  cargo xtask check --no-write --only docs-pairs
  ```
- **完了条件**：doctest と check が通る。`rg -n -e 'scope\(move' -e 'architecture/' -e 'devtools-client' -e 'Python' README.md README_JP.md` が何も返さない。`rg -n -e 'kivact-engine\]' -e 'arvane-render\]' -e 'kivact-layout\]' templates/basic/README.md` が何も返さない。`rg -n -e 'Migrating from the previous release' -e 'kivact-recipes' crates/*/README.md` が何も返さない。3.0.6 の二つのコマンドを `README.md README_JP.md templates/basic examples/kivact crates/*/README.md` に当てて何も返らない。

## G13：`docs/for-kivact-developer/architecture.md` の書換え

- **wave / lane / 前提**：W11a / G3 / —（W1〜W10 の完了）
- **所有**：`docs/for-kivact-developer/architecture.md`
- **目的**：設計 4 章（構成）と 6 章（内部の規則）を恒久の規則として書く。54 package の表、境界、新しい状態の規則（6.1）、PERF-6〜PERF-11、DEC-13、test binary、gate の段（CONTRIBUTING を参照）、公開 API の規則を 5 章に合わせる（AC-5）。
- **手順**：ファイルを上から順に、次の表のとおりにする。「そのまま」は節の本文を変えない。「置換」は節の本文（見出しの行は残す）を下の G13.x のテキストにする。「用語」は G13.9 の置換を当てる。

| 節（現在の見出し） | 処置 |
|---|---|
| 冒頭の段落 | そのまま |
| `## crate の境界` の最初の段落と表と表の下の段落 | 置換（G13.1） |
| `### 境界を作る規則（B）` | 置換（G13.2） |
| `### module の向き` | 置換（G13.3） |
| `## 一つの語彙（VOC）` | そのまま |
| `## 状態と事実の所有（S）` の表と、その下の 3 つの段落（「Options/Event/Phase/...」「印の理由は...」「Creative jobs の...」） | 置換（G13.4） |
| `### 事実の持ち主` | 用語 |
| `### Machine の書き方` | 用語。最後に 1 行足す（G13.4 の末尾） |
| `## 公開 API の 17 規則` の表 | 置換（G13.5） |
| `### 公開 module と利用者の入口` | 置換（G13.5） |
| `### handler の語彙と実行` | 置換（G13.5） |
| `### 構造 option と slot` | 置換（G13.5） |
| `### 共有データの境界` | そのまま |
| `## kit の規約（KIT）` の表と下の段落 | 置換（G13.6） |
| `### 日付・複合部品の共通 behavior` | 用語 |
| `## 性能とメモリ（PERF、MEM）` | 表に行を足し、表の下に段落を足す（G13.7） |
| `## feature と OS（DEP）` | 表の行を直し、GPU の段落を置換（G13.8） |
| `## turn、Stage と時間` | そのまま |
| `## 検証と保守の単位` | 置換（G13.10） |
| `## 追加・変更するときのチェックリスト` の表 | 行を直す（G13.11） |
| `### 最初に読む実装の場所` | 行を直す（G13.11） |
| `### 具体的な追加経路の机上確認` から最後まで | 消す（4.3 の 3：親の決定） |

### G13.1 crate の境界

````markdown
workspace は 54 package からなる。アプリと公式 kit は `kivact` facade を利用する。内部 crate を直接使わせることで公開 API の不足を埋めない。依存の許可と module の向きは `dev/xtask/policy/workspace.toml` が機械的に検査する。

| package | 担当と境界の理由 |
|---|---|
| `kivact` | 公開 facade。正準 module と prelude を再公開し、任意機能の入口をまとめる。同じ item を二つの module から出さない |
| `kivact-macros` | `#[component]` と `#[derive(Prop)]`。関数の署名から props の struct、constructor、`Option` と `#[prop(default)]` の setter、`IntoView`（戻り値が `impl Element` なら `Element` も）を生成する。自分を使う crate の path を `proc-macro-crate` で解決する（依存に `kivact` があれば `::kivact`、無く `kivact-core` があれば `::kivact_core`）。proc-macro は独立の crate でなければならない |
| `kivact-types` | workspace 共通の純粋な値、識別子、幾何、入力、意味情報と、std だけの小さな同期 helper（`cell::Recent`、`cell::Deferred`、`cell::SharedDeferred`、`process`）。ほかの workspace crate に依存しない |
| `arvane-protocol` | engine と renderer の契約（`RenderScene`、`Composition`、`CpuFrame`、`SemanticsTree`、描画用の paint・gradient・span）。作るのは engine だけ、使うのは arvane-* だけ |
| `kivact-runtime` | reactive、task、motion、Machine、turn の状態と wake。これらは同じ owner と実行順に依存するため一つに置く |
| `kivact-i18n` | Locale、messages、ICU4X による地域書式・日付計算。reactive context の所有は core |
| `kivact-style` | authored Style（CSS の名前と値の型）、Theme（`Arc` で共有する値）、theme 変数（`ThemeVar`）、Draw、Path と値の解決。外部描画 library の型を公開契約に出さない |
| `kivact-shader` | ユーザーの WGSL の検証と shader module の組み立て。naga だけに依存し、GPU の build だけが使う |
| `kivact-svg` | 記録した `DrawList` を SVG 1.1 の文書に書く。kivact-types と png だけに依存し、文字の baseline と画像の画素は `kivact-offscreen` が `Resources` で答える |
| `kivact-offscreen` | window なしの描画。入力は Theme・font・描画の closure、出力は `Pixels` か SVG の文字列。view と runtime を読まない。facade が `kivact::draw::Offscreen` として再公開する |
| `kivact-editing` | buffer、journal、editing session。window、描画、OS を知らない |
| `kivact-layout` | layout の計算と一つの Layout owner。Taffy は内部実装 |
| `kivact-core` | App、view、mount、context、入力配送、overlay、platform、diagnostics、embed。turn 全体の順序を所有する。view・client・platform・devtools は node 表の内部を共有するので分割しない |
| `kivact-interaction` | 描画しない behavior、Part と状態機械。view を返さない（`surface()` を除く）。UI の recipe を知らない |
| `kivact-ui` | 標準部品（`#[component]`）、部品の variant の enum、`kivact::ui::theme`（部品ごとの `<Name>Theme`・`<Name>State`・部品の theme 変数、`Look`、`Variant`）。kit なしで実用画面を作れる部品と、二つ以上の kit が共有する部品を持つ。入力で動く `impl Machine` と signal を書く `effect` を持たない |
| `kivact-test` | App 全体の E2E、Screen、Inspect。公開 view API と engine の試験境界を使う葉であり、view tree の内部を直接読まない |
| `kivact-devtools-protocol` | 3 者（agent・DevTools アプリ・MCP）が共有する通信の語彙と JSON Schema。I/O を持たない純粋な値と純粋関数。feature `client` が discovery・接続・型付きの呼び出し・通知の受信を足し、アプリと MCP が共有する |
| `kivact-devtools-agent` | socket・thread・file・process を扱う server。Kivact の内部を知らず、core は `take_incoming` で受け取った `Call` を処理して `Responder::send` で答える |
| `kivact-devtools` | DevTools の GUI。公開 API と標準部品だけで作り（P-07 の dogfooding）、私有の theme を持つ。kit に依存しない。パネルは module にする |
| `kivact-devtools-mcp` | MCP の仕様への追従と stdio。GUI を link しない |
| `kivact-engine` | window ごとの tree/model（`Model`）と App 全体の `AppModel`、計測、入力と配送の受理（admission）、paint と per-node の draw cache、a11y、host pump。lower と text_input は内部 module。多数の内部情報を共有する処理を偽の crate 境界で分けない |
| `arvane` | window driver と presenter。GPU の device は process（Driver）に一つで、window ごとには frame の状態だけを持つ。他の crate を再公開しない |
| `arvane-shell` | OS 非依存の window SPI、入力語彙と正規化 pipeline。winit、softbuffer、AccessKit を持ち込まない |
| `arvane-native` | Windows/Linux の実 window、OS port、入力 mapping、present、支援技術、services、embed、mirror |
| `arvane-virtual` | 台本、仮想 clock、window、surface と SteppedBackend。native mirror を任意に接続する |
| `arvane-text` | text shaping と計測。Parley を内部に閉じる |
| `arvane-render` | RenderScene のラスタライズ（Vello）。Vello と wgpu を内部に閉じる |
| `arvane-compose` | Composition の plan と、その CPU・GPU の合成。3D、cache、shader、外部 texture を Vello の外で一つの合成として扱う（Chrome の Skia と cc の分担と同じ） |
| `arvane-asset` | image、SVG、font の読み込みと decode。SVG は usvg で `RenderScene` にして vector で描き、2 つ目の rasterizer・shaper・font DB を持たない |
| `kivact-gpu-share` | UI に依存しない GPU texture の import/export 契約 |
| `kivact-ui-basic` / `kivact-ui-pop` / `kivact-ui-game` / `kivact-ui-creative` / `kivact-ui-soft` | 五つの公式 kit。共通の骨格（`theme(mode)`、kit の theme 変数、`recipes_<group>.rs`、`variants.rs`、分野の部品は `src/<domain>/`）。共通 behavior を複製しない |
| `kivact-plot` | 数値の XY plot（`PlotModel` を受けて view を返す）。色は `kivact_plot::color` の theme 変数で受け、どの kit とも組み合わせられる |
| `kivact-template-basic` | アプリの雛形 |
| `kivact-example-counter` / `kivact-example-form` / `kivact-example-list` / `kivact-example-async` / `kivact-example-component` | 公開 API の小さな利用例 |
| `kivact-ui-basic-catalog` / `kivact-ui-pop-catalog` / `kivact-ui-game-catalog` / `kivact-ui-creative-catalog` / `kivact-ui-soft-catalog` | 五つの kit の利用例と recipe の見本 |
| `kivact-catalog-shell` | 五つの catalog の shell・page 一覧・検索・scroll-spy（`examples/catalog-shell`、`publish = false`）。入力は page の一覧と theme の関数 |
| `kivact-public-api-tests` | facade の公開契約の試験。test binary は `e2e_core`・`e2e_ui`・`e2e_native` の 3 つ |
| `kivact-kit-tests` | kit に依存する公開 API の E2E（`e2e_kits`。theme_conformance と kit の crate が要る試験）。`dev/tests/kivact-kit-tests` |
| `kivact-docs` | 英語の文書と README の Rust のコンパイル。日本語は xtask の `docs-pairs` が英語と同じであることを確かめる |
| `gpu-interop-producer` | GPU 共有を検証する別の描画 producer |
| `kivact-bench` | 性能の指標（K1〜K6）を手動で測る probe と `run.sh`。facade の公開 API だけを使い、CI では実行しない |
| `xtask` | 依存、API、状態、style、文書の対の検査。図の生成は gate の外の `cargo xtask diagrams` |

library は 34、アプリは 2、雛形・小例・catalog と catalog の shell は 12、試験・道具は 6 package。検査用 package も workspace の数に含める。旧 wire、reactive、task、motion、text、lower の独立 crate と、DevTools の client だけの crate は復活させない。

次は境界が明確でないので分割しない（B-3）：kivact-core（view・client・platform・devtools は node 表の内部を共有する）、kivact-engine（host が model の内部を直接読む）、engine の `lower`・`text_input`（狭い API ができるまで module のまま）、arvane-native の embed（a11y・services・入力・present を共有する）、arvane-render と arvane-compose、kivact-ui の分野の部品（calendar・chat など）、kivact-types の `cell`。
````

（package の名前は `cargo metadata --no-deps --format-version 1 | grep -o '"name":"[a-z0-9_-]*","version"' | sort` の結果と一字ずつ照らす。数が 54 でなければ STOP。catalog の shell の名前が違えば実際の名前を書く。）

### G13.2 境界を作る規則（B）

````markdown
- B-1: 入出力を少数の型で説明でき、相手の内部状態を調べず、一つの変更理由を持つ単位にする。
- B-2: 明確な境界は crate にして依存、可視性、feature、最適化を制御する。
- B-3: 明確でない境界は統合する。将来の DAG や最適化だけを理由に crate を作らない。
- B-4: crate/module の先頭に英語の `//! Role:` と役割を書く。crate の境界の根拠も説明する。例・雛形・docs は対象外。
- B-5: `visible_to` にない利用者は追加しない。dev-dependency は `visible_to` の対象にしない（全体の既定）。新しい依存が必要なら境界の判断と policy を先に変更する。
````

### G13.3 module の向き

現在の節の最初の段落と表を残し、表を次のとおり直す：

- `core` の行：「`launch → client/devtools`、`app → launch`、`embed → app/launch/view`。`custom → view/draw`。`client` は `devtools` を参照せず、`view/devtools_hooks.rs` の hook を呼ぶ」を「`app → launch`、`embed → app/launch/view`。`custom → view`。`client` は `devtools` を参照せず、起動時に一度設定する `DevtoolsHooks`（trait object）だけを呼ぶ。`launch` は `kivact-test` だけが使う hidden な契約」にする。core の module の実際の名前と向きを `dev/xtask/policy/workspace.toml` の core の `[modules]` の表と照らし、違うところは policy に合わせる。
- `interaction` の行の最後に「view を返さない（`surface()` を除く）」を足す。
- 表の `interaction` の行の後に次の行を足す：`| ui | 部品ごとの module（view と `<Name>Theme`）。interaction の behavior を使い、入力で動く Machine と signal を書く effect を持たない。部品の module どうしは参照しない |`
- `engine` の行の「hit test（`model/interact/hit`）は ...」の文の後に「paint は描き直した node だけを lower し直す draw cache を持ち、restack は重なりの事実が変わったときだけ行う（PERF-6）。」を足す。
- 表の最後に次の行を足す：`| arvane | driver が process の GPU device を持ち、window ごとの presenter が frame の状態を持つ。engine は arvane-text・arvane-compose・arvane-asset・arvane-shell を直接使い、arvane の再公開を通さない |`

### G13.4 状態と事実の所有（S）

表と、その下の 3 つの段落を次にする：

````markdown
| 規則 | 内容 |
|---|---|
| S-1 | 三つ以上の状態、または phase により field の有効性が変わるものは enum にする |
| S-2 | 二つ以上の状態の bool/Option/counter、または Option と State/Phase enum の併置を検査する。独立した一つの bool を機械的に Machine にしない |
| S-3 | 同時に有効な値を variant の payload に入れる。依存する Some/None を並べない |
| S-4 | 同じ事実を二度保存しない。表示・readonly getter は正本の投影にする |
| S-5 | 遷移だけが状態を書く。effect による状態の補修、同期用 flag、epoch/counter を作らない |
| S-6 | phase の順序は型、状態、runtime guard で表す |
| S-7 | 必要な依存を constructor に渡す。未完成の object を作って後から slot を埋めない |
| S-8 | 通知には `Trigger` を使う。counter は件数・ID・統計の意味がある場合だけ |
| S-9 | ReportOnce、Sent、Generation、Mailbox、純粋 cache、統計を区別する。異なる事実を無理に一つの phase にしない |

**検査の単位は型（xtask の `state`）。** 次の (a) と (b) を満たす struct と enum variant が対象になる。

- (a) `bool`、`Option<_>`、整数の counter、`Cell`/`RefCell`/`Signal`/`Var`/`Atomic*` の field を二つ以上持つ。
- (b) `&mut self` の method を持つか、内部可変の field を constructor の外で書く。
- 名前が `Api`、`Options`、`Policy`、`Props`、`Report`、`Event`、`Phase`、`Reason`、`Recipes`、`State`（recipe の状態）、`Theme`、`Tokens`、`Builder`、`Key`、`Bits` で終わる型と、`Hash + Eq` を derive して map の key だけに使う型は対象外。

対象の型は、型の直前の一行の印 `// xtask-allow(state): <組合せの閉じ方>` が無ければ error になる。閉じ方は「すべての組合せが有効な理由」か「どの組合せをどの遷移が排除しているか」を一行で書く。constructor、`Default`、値渡しの builder（`fn x(mut self, ..) -> Self`）だけで書く field は状態ではなく、印も検査も要らない。関数の中の状態の local（`signal(false)` と `signal(None)` の組など）の検査は残る。例・雛形・docs は検査の対象外。

検査は意味の誤り（真に排他な組合せを独立と書いた等）までは見つけられない。新しい状態の型を足す変更では、印の一行が型の実際の遷移と合っていることを review で確かめる。

Creative jobs の run/stop は独立した一つの入力であり、progress/amount は数値の測定事実である。effect が実 FrameLoop の start/stop だけを行う場合、bool を置換するだけの第二の Machine は不要である。
````

`### Machine の書き方` の最後（6. の後）に次の段落を足す：「標準部品（kivact-ui）は入力で動く Machine を持たない。入力の状態機械は interaction の behavior に置き、ui はその Api と Part を使う。」

### G13.5 公開 API の規則

`## 公開 API の 17 規則` の表を次にする（設計 5 章が優先する部分を書き込んだ形）：

````markdown
| ID | 規則 |
|---|---|
| API-1 | 正準 module path と prelude だけを入口にする。constructor と型を同じ module に置く。behavior/data は interaction、部品と variant は ui、部品の theme（`<Name>Theme`、`<Name>State`、部品の theme 変数、`Look`、`Variant`）は `kivact::ui::theme`。同じ item を二つの path から出さない |
| API-2 | 動的な部品の option は `Value<T>` の props（`impl IntoValue<T>` を受ける）。構造を決める flag/mode、構築前の option 束、passive record は通常の値 |
| API-3 | `on_` は event だけ。payload の無い handler（`on_press` など、`#[component]` の `Callback<()>` props）は `impl IntoHandler<(), M>`（closure と `Callback<()>`）、payload のある handler（`on_key`、`on_activate`、`on_commit`、`on_edit`、`Callback<E>` props）は `impl Fn(E) -> R + 'static`（共有の `Callback<E>` は `move \|e\| cb.call(e)`）を受ける。戻り値は `HandlerResult`。lifecycle の登録は FnOnce。純粋 query は名詞と借用値。部品の必須の handler は位置引数、任意の handler は `Option<Callback<E>>` の setter |
| API-4 | 論理座標、寸法、連続値は入力も読み返しも f64。件数/index/level/font weight は整数。整数 literal の変換と異なり reactive closure は f64 を返す |
| API-5 | 見える text は `Value<String>`（`IntoText` を受ける）、ID/name は `Into<String>`、message key/default は static str。label は control の名前、title は見える container 見出し、Node.name は生の意味情報 |
| API-6 | 一度 mount する slot は `View`、作り直す content は `ViewFn`（または `Fn(Arg) -> V`）、項目は item ごとの factory。`provide` と `scope` は FnOnce。slot 名の語彙を揃える |
| API-7 | `use_` は owner に結び付いた behavior、context、App 全体の service の読み出し（`use_toaster`、`use_announcer`、`use_theme`、`use_context`）。signal/memo/machine/resource/action/animated/frame_loop/node_ref と app-wide environment query に `use_` を付けない。provide_x/use_x は対 |
| API-8 | Rc/Arc/Box dyn/Location/明示 lifetime を普通の公開署名に出さない。lifetime の例外は Constraints と Element の借用。実在する内部 cross-crate bridge だけ doc(hidden) にし、`__private`/`__macro` の module と policy の `[macro_support]` の crate は検査の側で例外にする |
| API-9 | backend の型、公開 path で名前を書けない型を漏らさない |
| API-10 | kit の名前は facade prelude、ui、標準 prelude と衝突させない。kit 名の接頭辞を型名に付けない |
| API-11 | conversion trait の診断を具体的にする。独自の型を props の定数にするのは `#[derive(Prop)]`（`constant_value!` は hidden で runtime と types の基本型だけ） |
| API-12 | setup/context の misuse は track_caller 付きで、修正方法の分かる panic にする |
| API-13 | 必須位置引数は見える label/title、data、選択、開閉、children。見えない名前を持つ入力部品（text_input、text_area、select、combobox、slider、number_input、日付・時刻の入力）は名前を位置引数で取らず、`.label(..)` か `field(..)` で受ける。focus できる control に名前が無いと mount で位置付きの診断（debug build と `kivact::test` では error）。項目の集まりは items。値を編集する部品は `Signal<T>` を位置引数で受ける（0 個以上を選ぶ部品は `IntoSelection`、単一は `Signal<Option<K>>`、必ず一つは `Signal<K>`）。閉じていると意味の無い部品（dialog、window、drawer、sheet）は `open: Signal<bool>` を位置引数、開閉が任意の部品は setter `.open(signal)`（呼ばなければ部品が局所の signal を作る）。一つの root は `impl Element` |
| API-14 | bool query は is_/has_、それ以外は名詞。get_/_value は使わず、更新は set/動詞、失敗可能は try_。略語は a11y/ime/id 以外を増やさない |
| API-15 | Api、State、Variant、Theme（部品の recipe 表）、Options、Policy、Mode、Kind、Event、Phase、Reason を役割どおり使う。InteractionPolicy の入れ子も Policy |
| API-16 | 開閉の behavior は `use_x(.., open: Signal<bool>)` で open を必須の位置引数に取り、命令 `show`/`hide`/`toggle`（引数なし、冪等）と `is_open() -> Value<bool>`、`trigger()`・`content()` の Part、`surface(placement, content) -> View` を持つ。開いている間だけ content を mount するのは `surface` か `show(api.is_open(), f)` |
| API-17 | 公開の recipe state は、排他な事実を pub の enum の field（型名は owner の接頭辞を除いた名前。部分を表す enum は State で終わらせない。例 CardAction、CheckControl）、常にある事実を pub の field、一部の種類にだけある事実と導出する事実を accessor で表す。bool の accessor は is_/has_ で始める。part の状態を読む唯一の方法は `Part::state()` |
````

`### 公開 module と利用者の入口` の本文を次にする：

````markdown
root は App、Result、UiError、ErrorKind と macro の `component`・`Prop`・`bind`。state、view、geometry、style、draw、motion、task、i18n、platform、diagnostics、interaction、ui、custom を意味ごとの正準 module とし、test/embed/gpu_interop は feature の入口とする。GPU の起動の希望（GpuPreference、GpuPower、GpuBackend、GpuStart、AdapterMatch）は feature なしで `platform` に置き、gpu feature の無い build では無視する。CSS の値の型（Shadow、Filter、Transform、Paint、gradient、Shape、CornerRadius、BlendMode、Mask*、BorderImage*、PaintShader、FilterShader、FilterGraph）は `style` だけ、描画者の型（Draw、Path、Stroke、Layer、Mix、Text*、Pixels、Offscreen）は `draw` だけに置く。prelude は設計で決めた一覧（日常の利用に必要な名前）を明示再公開し、変換 trait（IntoValue、IntoText、IntoHandler）を入れない。custom は kit 作者の正式な独自部品境界（Owner、Part、primitive、scope、raise、IME、文字編集）で、内部 engine の型を直接使わせない。

定数 prop は literal または `#[derive(Prop)]` を付けた型の値で扱う。reactive な値は `Value<T>`、書ける値は `Signal<T>`（lens も同じ型）、selection は single/multiple の選択、CalendarSelection は日付の Single/Range descriptor であり、異なる情報を一つの conversion trait に押し込めない。
````

`### handler の語彙と実行` の本文を次にする：

````markdown
HandlerResult は `()`、Handled、Result<(), UiError>、Result<Handled, UiError> の四つを受ける。Handled は Yes/No/PreventDefault を区別し、合成でも PreventDefault を失わない。`noop()` は伝播する。bubble しない callback は Handled を捨てるため、その戻り値で calendar の選択を拒否できない。拒否には実遷移前の activation_guard のような policy を使う。

handler は runtime の一つの `Handler<E>`（`Rc<dyn Fn(E) -> Result<Handled, UiError>>`、arena の object を作らない）に揃える。受け方は二つだけ（DEC-16）：payload の無い handler は sealed な `IntoHandler<(), M>`（closure `Fn() -> R`（M = Nullary）と `Callback<()>`（M = Handle））、payload のある handler は `impl Fn(E) -> R + 'static`。rustc は closure の引数の型を `Fn` の bound からしか推論しないので、payload のある handler を trait で包まない。共有したい handler は `callback(f)` で作り、payload のある handler には `move |e| cb.call(e)` で渡す。`#[component]` の `Callback<E>` props は内部で `Handler<E>` を持ち、本体には Copy の `Callback<E>` を渡す。owner への登録は本体が子に渡すときだけ行う。user handler、Part、既定動作の順序で伝播と既定動作の防止を区別する。

state の on_mount/on_cleanup は FnOnce の lifecycle 登録であり、node の実装用 on_attach(PartMount) と区別する。イベントは on_press（payload のない押下）、on_activate(key)（key のある項目の実行）、on_commit(value)（確定）、on_edit(EditPhase)（編集の段階）、on_submit、drag の on_start/on_update/on_end、on_files、on_dismiss(reason)、on_complete(Result)/on_exit_complete、on_error(UiError)、on_composition_change、on_active_change、on_reorder とする。項目の活性化は on_activate とし、on_select・on_pick・on_item_activate・on_step_press を使わない。値は Signal が正本で、通知のための on_select・on_add・on_remove・on_open_change を足さない。支援技術 action は a11y_action に登録する。
````

`### 構造 option と slot` の本文の第 3 段落（「ListItem、MenuEntry、...」）の「RenderOptions と AppOptions は private field と setter にまとめる」を「App と RenderOptions は private field と setter にまとめる（AppOptions は App の非公開の field の型）」にする。第 4 段落（「`view::scope(FnOnce → IntoView)` は ...」）と第 5 段落（「アプリ・kit・catalog・例の部品は `#[component]` で書く。...」）を次の 2 段落にする：

````markdown
部品は `#[component]` の関数だけで書く（アプリ、kivact-ui、五つの kit、kivact-plot、catalog、例）。必須の props は宣言順の位置引数、`Option<..>` と `#[prop(default = ..)]` は名前付きの setter。props の種類は型の字面の名前で決まる（`Value<T>`、`Signal<T>`、`Callback<E>`、`View`、`ViewFn`、`Vec<View>`。別名はそれ以外の props）。戻り値が `impl Element` のとき、生成する struct は隠した `__root: Node` を持ち、`into_view` は本体を `scope` の中で一度だけ実行して `Node::absorb` で呼出し側の modifier を本体の root に移す（呼出し側の style の層は本体の後ろ、handler は本体の前、name・description・disabled・test_id・node_ref・variant は呼出し側の値）。`impl Element for` / `impl IntoView for` を手で書けるのは kivact-core の view module の基本要素（node、text、column、row、stack、grid、scroll、spacer、image、icon、canvas、window、overlay など）だけで、xtask が確かめる。`Element` の method と同じ名前の固有 method を `Element` の実装型に置かない。

`kivact::custom::scope(FnOnce → IntoView)` は mount 時に一度だけ実行する setup の境界で、macro を使わずに部品を組む kit 作者の逃げ道である。新しい child owner と透明な fragment の下で hook/cleanup を保つ。eager に factory を呼ぶことや、`provide(())` で空の context を置いて owner を偽造することでは代用しない。再度開く content は必要な factory を保ち、捨てた editor owner を再利用しない。
````

API-11 の診断の段落（「API-11 の診断は blanket impl の ...」）はそのまま。API-8 の段落（「API-8 は診断と標準 Rust Error の ...」）の「IntoValue/IntoText の内部変換、Value/Binding の runtime/location」を「IntoValue/IntoText の内部変換、Value/Signal の runtime/location」にする。

### G13.6 kit の規約（KIT）

表と、その下の段落を次にする：

````markdown
| ID | 規約 |
|---|---|
| KIT-1 | `theme(mode) -> Theme`。実際の option があるときだけ `ThemeOptions` と `theme_with(mode, options)`。Theme の extension trait を作らない。theme の作り直しは `kivact::style::cached_theme(build)` で避ける |
| KIT-2 | kit の色・大きさは kit の theme 変数（`<kit>::PRIMARY` などの `const` と `PRIMARY_VAR` などの `static`）。`theme()` が中核の変数（`color::ACCENT_VAR` など）を kit の変数に向ける。`ColorName` と Tokens の値の field を持たない。色 literal は theme 変数の定義と `theme()` だけに置く |
| KIT-3 | 形の差は `recipes_<group>.rs` の `<Name>Theme` の上書き、値の差は theme 変数。全 recipe を無差別に上書きせず、上書きしない部品は中核の既定を使う |
| KIT-4 | 公開名は API-10 に従う。kit 名の接頭辞を型名に付けない（`GameCellVariant` としない） |
| KIT-5 | slot は API-6、children は `Vec<View>`。kit 私有の同義 slot/children trait を作らない |
| KIT-6 | 見た目の選択は `variants.rs` の `<Component>Variant` 一つにまとめ、`Element::variant` で渡す。`*Appearance`・`*Look`・`*Hue`・`*Tone` と呼ばない。標準 Element の disabled を隠さない |
| KIT-7 | kit の use_ は固有 domain に限る。共通 hook は core/interaction、状態は Machine |
| KIT-8 | 標準 message は component.key、kit 固有は kit.component.name。共通意味に複数 key を作らない |
| KIT-9 | kit の E2E は `crates/kivact-ui-<kit>/tests/consumer` の一つの binary（kit 固有の部品の契約）と、全 kit を通す `kivact-kit-tests` の `e2e_kits` の theme_conformance（共通部品の配線）。interaction の behavior の試験を kit に写さない |
| KIT-10 | 共通 font asset は一か所に置き、kit の theme が `Theme::with_fonts` で持つ。license も同じ asset と管理する |

共通部品へ移すときは移す元の kit の option と behavior の和集合を残す。単なる wrapper や再公開で kit に二つ目の入口を残さない。固有 domain は `src/<domain>/` に残す。例は Creative の keyframe glyph と column-follow picker、Pop の conversation-preview row、Soft の dictation/streaming/tool-call/token。catalog は削除せず、`kivact-catalog-shell` の shell の上で、その kit の recipe で標準部品を使う見本にする。catalog の文言は英語にする。
````

### G13.7 性能とメモリ

`## 性能とメモリ（PERF、MEM）` の表の PERF-5 の行の後に次の 6 行を足す：

````markdown
| PERF-6 | frame の描画は、描き直した node だけを lower し直す（per-node の draw cache、描画中の node だけが持つ）。paint list の並べ直し（restack）は、重なりの事実（stacking context、z、layer の事実、子の組の変化）が変わったときだけ行う |
| PERF-7 | Arvane は engine が渡す layer ごとの `content_version` で変化を知る。command の hash は変化した chunk だけ。debug build では hash による照合で版の上げ忘れを検出する |
| PERF-8 | GPU の device は process（Driver）に一つ。window ごとに持つのは frame の状態だけ |
| PERF-9 | 診断の記録（turn の trace、原因の文字列、node の provenance）は、DevTools・`kivact::test` の trace・debug の trace sink が有効にしたときだけ行う。release で常に持つのは counter だけ |
| PERF-10 | 起動の critical path（main から最初の present まで）で、system font の走査・DevTools agent の起動・asset worker の起動・使わない GPU pipeline の compile をしない |
| PERF-11 | 項目の多い collection（list、combobox、menu、select、tabs、tree、table、palette）は、collection ごとに一つの key の索引（`Index<K>`）と `selector` を使い、項目あたりの memo は最大 2 つ |
````

表の下（MEM-8 の行の後の空行の後）に次の 2 段落を足す：

````markdown
PERF-1 の Metadata の経路の例外は残す。ただし Metadata の経路で semantic order を二度数えず、clip の計算は attached な node だけを回る。

遅延初期化の道具は一つ：`clippy.toml` は `OnceCell`・`OnceLock`・`LazyCell`・`LazyLock` を禁止する。使う時に作る重い資源（PERF-2、PERF-10：ICU の formatter、GPU の pipeline、system font、既定の theme など）は、`kivact_types::cell::Deferred<T>`（UI thread の中、`!Sync`）と `kivact_types::cell::SharedDeferred<T>`（`static` と thread をまたぐもの、`Sync`）だけで持つ。どちらも `const fn new()`、`get_or_init(&self, f) -> &T`、`get(&self) -> Option<&T>` を持つ。std の型を使うのはこの二つの実装の中だけ。状態の補修や「未完成の object を後で埋める」（S-7）には使わない。
````

### G13.8 feature と OS（DEP）

表を次のとおり直す：

- `kivact` の行の「devtools は core と test の devtools（有効な release build は起動時に警告する）」の後に「。embed は全 OS で compile でき、未対応の環境では `App::embed` が `Unsupported` を返す」を足す。
- `kivact-test` の行の後に次の行を足す：`| kivact-devtools-protocol | client（discovery・接続・型付きの呼び出し・通知。DevTools アプリと MCP が使う） |`
- `docs` の行の「default は gpu と complex-scripts を facade に転送。」の後に「英語の文書だけを compile する。」を足す。
- `その他` の行の「五つの kit に gpu-interop 等の中継 feature を置かない」の後に「。kivact-plot も同じ」を足す。

表の下の 2 段落目の後の GPU の段落（「GPU の device は window ごとに必要時に起動する（既定は `GpuStart::OnDemand`）。...」）を次にする：

````markdown
GPU の device は process に一つで、必要時に起動する（既定は `GpuStart::OnDemand`）。起動の入口は presenter の `Gpu` の二つだけで、`begin`（`AtLaunch` のとき window の作成時）と `demand`（OnDemand で待っているとき）である。需要は、composition の user shader（CPU で代わりの見た目を描いた報告）、core の `gpu_interop::adapter()` の読取り（`HostRequest::DemandGpu`）、CPU の frame が表示周期の半分を 8 frame 続けて超えたこと、の三つ。`GpuStart::Never`、gpu feature の無い build、環境変数 `ARVANE_RENDERER=cpu` の window は需要があっても起動しない。`ARVANE_RENDERER` の `cpu`・`gpu`・`auto` は `Never`・`AtLaunch`・`OnDemand` に対応する。試験は `RenderOptions::gpu_on_demand()` で OnDemand を選び、`Screen::renderer()` で何が描いているかを確かめる。
````

### G13.9 用語の置換（「用語」の節に当てる）

次を左から右へ置き換える（節の文の中だけ。意味が変わらないことを確かめる）：

| 旧 | 新 |
|---|---|
| `caller binding`（事実の持ち主の表の行の見出し） | `caller の Signal` |
| `binding`（値の書込み先の意味の小文字の語） | `Signal` |
| `Notifier` | `Trigger` |
| `IntoBinding` | `Signal` |
| `UiTokens`・`Tokens` | 部品の theme 変数 |
| `UiRecipes`・`Recipes` | `<Name>Theme` |
| `ViewTransform` | `ZoomPan` |
| `App.fonts` | `Theme::with_fonts` |

### G13.10 検証と保守の単位

節の本文を次にする：

````markdown
一つの関数は 80 code 行、file は 1000 code 行を上限とする。長い処理は意味のある helper に分け、line packing、無意味な型/module、新しい state、抑制属性で件数だけを減らさない。依存、状態（`state`）、effect の書込み、公開署名、正準 path、名前衝突、kit の色、OS 分岐、module の規則（interaction は view を返さない、ui は入力で動く Machine と signal を書く effect を持たない、`impl Element for` は core の view だけ、`machine(()` と `type State = ()` の禁止、`Element` の method と同名の固有 method の禁止）、英日の文書の対（`docs-pairs`）を全 workspace で検査する。検査の例外は実際の境界と理由に限り、同じ理由を繰り返す印は規則の側の例外にする。

性能は `dev/bench` の K1（起動）、K2（メモリ）、K3（CPU）、K4（フレーム時間）、K5（更新の遅延）、K6（バイナリの大きさ）で測る。比べる二つの commit を同じ機械で `run.sh` にかけ、比で判定する（機械ごとに絶対値が違うため）。K6 は Kivact 自身の大きさを GPU 一式を除く build（`--no-default-features --features complex-scripts`）で比べ、既定の build（`gpu` 込み）は大きくしないことを確かめる。結果は commit しない。E2E の `Stats` の a11y の走査の件数（`a11y_enumerated`）と資源の回収（mount の前の `live_reactive`・`live_owners`・`tree_nodes` に戻ること）で PERF-1 と回収の後退を検出する。

試験は App 全体の E2E（実 window と `kivact::test`）である。過去の不具合の再現だけの試験、画素や時間の厳密な比較、golden は足さない。公開 API の E2E は依存の軸で 4 つの test binary に分ける：`kivact-public-api-tests` の `e2e_core`（`kivact` だけ）、`e2e_ui`（ui・apg・interaction。`kivact` だけ）、`e2e_native`（embed、GPU、foreign window、DevTools、GPU 共有）と、`kivact-kit-tests` の `e2e_kits`（theme_conformance と kit の crate が要るもの）。docs の doctest は英語の木と `README.md` だけを compile し、日本語の Rust の code block は `docs-pairs` が英語と同一であることを確かめる。gate は 3 段（tier a：変更ごと、tier b：まとまりごと、tier c：merge 前）で、正確なコマンドと window-lock・GPU の試験の直列化は [CONTRIBUTING](../../CONTRIBUTING.md) の「テスト」と「merge 前の検査」に書く。

変更後の意味情報も実 node から検証する。AsyncView の既定 pending は Status Loading、picker エラーは Alert、Contents は layout の透明性と a11y Group/論理階層を両立する。Desktop 操作を検証する date fixture は明示 Desktop、一般の Adaptive fixture は touch の Save を保つ。icon は実 install 済み glyph で確かめ、timeout や description relation 欠落を期待値の緩和で隠さない。

NodeRef::snapshot は poll 時の node を Host の request/reply へ送り、layout/paint と geometry report が settle してから CPU 描画を取得する。owner の寿命を応答時にも確認する。古い scene の同期 read や、別の geometry モデル、追加の FrameTick 待ちで取得順序を代用しない。
````

### G13.11 チェックリストと最初に読む場所

`## 追加・変更するときのチェックリスト` の表の行を次のとおり直す（他の行はそのまま）：

- `Role`：「types の Role、native の role mapping、protocol の `Role` と core devtools の `convert`、protocol の JSON Schema の再生成」にする。再生成のコマンドは CONTRIBUTING に付属書 T が書いたものを `rg -n -i schema CONTRIBUTING.md` で確かめ、その文字列を backtick で書く（`cargo xtask devtools-codegen` は書かない）。
- `text style（例 text_wrap_style）`：「types の `TextStyle`（宣言・computed 値・継承を一か所）、style の setter、arvane-text の 3 crate。relative unit/current color と native 精度を保つ」にする。
- `標準部品`：「interaction の behavior、ui の部品の module（`#[component]`、`<Name>Theme`、部品の theme 変数、variant）、移す元の kit の全 option の和集合、`e2e_ui` の公開 E2E、`kivact-kit-tests` の `e2e_kits` の theme_conformance、英日ガイド」にする。
- `DevTools の method`：「protocol の leaf の型と `lib.rs` の再公開、`protocol_methods!` の 1 行、JSON Schema の再生成、core の `devtools/` の振り分けと leaf、MCP の除外の一覧、ガイドの tool の一覧。版の規則（wire の型の field の削除・改名・型の変更、method・通知の削除・改名は `PROTOCOL_VERSION` を上げる。追加は上げない）」にする（Python と codegen を消す）。
- `アプリの描画表示`：「canonical ViewTransform」を「interaction の `ZoomPan`」にする。
- `E2E`：「App 全体を virtual backend と本番 presenter/driver で動かす。依存に合う binary（`kivact-public-api-tests` の `e2e_core`・`e2e_ui`・`e2e_native`、`kivact-kit-tests` の `e2e_kits`）に置く。mirror は任意。Screen の共通 helper と既存の数値・focus・Signal の assertion を保つ」にする。

`### 最初に読む実装の場所` の表の `標準部品` の行を「interaction の該当 behavior、ui の該当部品の module、`dev/tests/kivact-public-api/tests/e2e_ui/`、英日 guide」にする。表の下の段落はそのまま。

- **消すもの**：`### 具体的な追加経路の机上確認` の節（その下の `####` の節と「件数のまとめ」を含め、ファイルの最後まで）。`state-fields`、`--report-audit`、`state-audit.md`、`independent:`/`projection:` の印の説明、`kivact-devtools-client` の行、Python の言及、`CONTRACT-10` の形の handler の規則、`launch` の module の公開の説明。
- **状態の閉じ方**：新しい型なし。
- **試験**：なし（`docs/for-kivact-developer` は doctest の対象外）。
- **確認（tier a）**：
  ```bash
  rg -n -e 'state-fields' -e 'report-audit' -e 'state-audit' -e 'devtools-client' -e 'Binding' -e 'Notifier' -e 'UiRecipes' -e 'UiTokens' -e 'ColorName' -e 'ThemeColor' -e 'devtools-codegen' -e 'architecture/' -e '50 package' docs/for-kivact-developer/architecture.md
  rg -n -i python docs/for-kivact-developer/architecture.md
  ```
- **完了条件**：上の二つの rg が何も返さない。「54 package」の文と、54 行分の package（表の 1 行が複数の package を並べる行は、並べた数で数える）が表にある。PERF-6〜PERF-11 の 6 行、DEC-13 の段落、`xtask-allow(state)` の説明、4 つの test binary、CONTRIBUTING への参照がある。

## G14：decisions.md、strategy.md、manual-verification.md

- **wave / lane / 前提**：W11a / G4 / —（W1〜W10 の完了）
- **所有**：`docs/for-kivact-developer/decisions.md`、`docs/for-kivact-developer/strategy.md`、`docs/for-kivact-developer/manual-verification.md`
- **目的**：改修の決定（設計 8 章の DEC-01〜DEC-14 と、採用した主な所見）を decisions.md に記録し、置き換えられた行を消す。strategy.md は二か所だけ直す（設計 7.5 Q1、DEC-10）。manual-verification.md の link と名前を新しい構成に合わせる（AC-5）。
- **手順**：
  1. **decisions.md の行を消す**（後の決定が置き換えたもの。各行は ID で探す。「補足」の箇条も同じ ID のものを消す）：
     - 実装の決定の表：`ADR-27`、`ADR-37`、`ADR-47`、`ADR-52`、`ADR-64`、`ADR-65`、`ADR-67`、`ADR-72` の行。
     - 実装の決定の補足：`- ADR-27:` と `- ADR-37:` で始まる箇条。
     - 公開 API の契約の表：`CONTRACT-01`、`CONTRACT-05`、`CONTRACT-08`、`CONTRACT-10`、`CONTRACT-11`、`CONTRACT-12`、`CONTRACT-14`、`CONTRACT-15` の行。
     - 状態の決定の表：`STATE-AUDIT`、`STATE-CLAIMS` の行。
  2. **decisions.md の行を直す**（文の一部だけ）：
     - `ADR-21` の決定の列の「ActionPhase は close 後も queue を drain。」を消す（`Action::close` が無くなった）。
     - `ADR-40` の決定の列の「、公開 DragVelocity」を消す。
     - `ADR-51` の決定の列の「JSON Schema・Python・MCP をそこから生成する」を「JSON Schema と MCP をそこから生成する」にし、理由の列の「Rust・Python・MCP」を「Rust と MCP」にする。
     - `ADR-66` の決定の列の「線・塗り・余白は `UiTokens::sparkline`」を「線・塗り・余白は sparkline の theme 変数」にする。
     - `CONTRACT-02` の決定の列の「nearest_snap と DragVelocity は types の一つの純粋実装を interaction が再公開する」を「nearest_snap は types の一つの純粋実装を interaction が再公開する」にする。
     - `BOUNDARY-04` の決定の列の「Color/TextLength を入口とし Rgba/TextLen は内部解決値」を「Color と TextLength を入口とし、Rgba は内部の解決値」にする。
  3. **decisions.md に節を足す**：`## 作者の決定` の節の表と APG・非目標の小節の後、`## 実装の決定` の見出しの直前に、次の節を置く：

````markdown
## 改修の決定（2026-11-18）

API の抜本的な改修で行った決定。設計者が決め、作者が PR の review で覆せる（DEC-01）。ここに書いた決定と矛盾する先の行は削除した。

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-18 | DEC-01 | 未決の問いを作者に聞いてから進めるか | 聞かずに設計で決める。作者は PR の review で覆せる | 実装者は設計の判断ができないため、全部を前もって決める必要がある |
| 2026-11-18 | DEC-02 | theme 変数の型名 | `ThemeVar<T>` | 状態の `Var<T>`（useRef）と衝突させない |
| 2026-11-18 | DEC-03 | ui・kit の部品も `#[component]` にするか（CONTRACT-01・CONTRACT-08 を置き換える） | する。手書きの builder struct と `impl Element`/`impl IntoView` は core の基本要素だけに残す。`-> impl Element` の部品は呼出し側の modifier を `Node::absorb` で本体の root に移す | 一つの書き方。利用者の部品 library も同じ形で書け、`.style(..)` を付けられる |
| 2026-11-18 | DEC-04 | `Signal` と `Binding` | `Signal` に統合する（内部は Source と Lens の enum）。`binding(get, set)` と `bind!` も `Signal` を返す | 書ける handle を一つにする |
| 2026-11-18 | DEC-05 | 入力部品の見えない名前 | 位置引数をやめ `.label()` と `field()`。名前の欠落は mount の位置付きの診断（debug build と `kivact::test` では error） | JS の `<label>` と `<input>` の感覚。a11y の保証は compile 時から実行時と試験に移る |
| 2026-11-18 | DEC-06 | 部品の token（ADR-37 を置き換える） | theme 変数に統一し、`UiTokens` と kit の `Tokens` の値の field を消す。形（recipe）は `kivact::ui::theme` の `<Name>Theme` | 値の仕組みと形の仕組みを一つずつにする |
| 2026-11-18 | DEC-07 | style の名前 | CSS の名前（`font_weight`、`font_style`、`border_radius`、`flex_*`、`outline`、`active`、`FlexDirection`） | CSS の利用者が推測できる |
| 2026-11-18 | DEC-08 | GPU の device | process に一つ。window ごとには frame の状態だけ | 起動とメモリが window の数に比例しない |
| 2026-11-18 | DEC-09 | SVG の asset | usvg で `RenderScene` にして vector で描く。resvg・tiny-skia・fontdb を消す | 2 つ目の rasterizer・shaper・font DB を持たない |
| 2026-11-18 | DEC-10 | Python の DevTools client（ADR-51 の Python の部分を置き換える） | 消す。xtask の codegen も消し、JSON Schema は protocol から作る | 利用者がいない。MCP が同じ役目を持つ |
| 2026-11-18 | DEC-11 | SVG の書き出しで同等のものが無い機能（ADR-64 を置き換える） | SVG 1.1 と CSS に同等があれば書き（font-feature-settings、font-variation-settings、mix-blend-mode、文字の装飾、塗りと線）、無いものだけ Unsupported の error にする。書き出しは `kivact-offscreen` の `Offscreen::render_svg` と `kivact-svg` | plot の書き出し（`tnum` など）が失敗しない |
| 2026-11-18 | DEC-12 | 性能の必達 | 全 KPI で後退しない（≤ 1.00、時間の行は 1.05、大きさの行は 1.02 までの雑音を許す）。改善の目標は記録する | 未達で設計が止まらず、後退は確実に防ぐ |
| 2026-11-18 | DEC-13 | 遅延初期化 | `kivact_types::cell::Deferred<T>`（UI thread）と `SharedDeferred<T>`（static）だけ。std の lazy 型の禁止は続ける。ADR-17・ADR-21 の内部の OnceCell はこの二つに読み替える | 性能の規則が求める「使う時に作る」と、S-7（未完成の object を作らない）の区別を一つの道具で表す |
| 2026-11-18 | DEC-15 | `update` の戻り値 | 返さない（`Signal`・`Var` とも）。破棄された signal への `set` と `update` は位置付きの `WriteAfterDispose` を一度記録して何もしない | 監査の提案（`Option<R>`）では最も多い `on_press(move \|\| x.update(..))` が handler の結果の型に合わなくなる |
| 2026-11-18 | DEC-16 | payload のある handler の型 | `impl Fn(E) -> R`。`IntoHandler` は payload の無い handler だけ | closure の引数の型の推論は `Fn` の bound からしか働かない。`Callback<E>` は `Fn` を実装できない |
| 2026-11-18 | DEC-14 | 改修の途中の kit | style の改修から kit の移行までの間、5 つの kit・catalog・kit の試験を compile から外し、kit は最終の API に一度で移す | 途中の API への追従を二度するより安い |
| 2026-11-18 | RW-01 | handler の受け方（CONTRACT-10 を置き換える。DEC-16） | 二つだけ。payload の無い handler は sealed な `IntoHandler<(), M>`（closure と `Callback<()>`）、payload のある handler は `impl Fn(E) -> R + 'static`（共有の `Callback<E>` は `move \|e\| cb.call(e)`）。どちらも `Handler<E>`（`Rc<dyn Fn>`、arena の object を作らない）にする。`From<F> for Callback<()>`、`Callback::bind`・`to_fn` を消す | closure の引数の型の推論は `Fn` の bound からしか働かず、`Callback<E>` は `Fn` を実装できない。payload の無い press では `Callback<()>` を包み直す必要と press ごとの arena の object が消える |
| 2026-11-18 | RW-02 | context の読み出しの型 | `use_context::<T>() -> T`、`try_use_context -> Option<T>`。context の置き場は runtime と core の二つを一つにする | 定数の `Value` を二度 `.get()` する必要が無い。reactive にしたいときは中に `Signal` を入れる（Solid と同じ） |
| 2026-11-18 | RW-03 | prelude の範囲 | 設計で決めた約 120 の名前の明示の一覧。変換 trait（IntoValue、IntoText、IntoHandler）、kit 寄りの型、`scope` を入れない。`provide_theme`・`use_theme`・`color`・`space`・`radius`・`font` を入れる（CONTRACT-14 を置き換える） | 初めての利用者が見る名前を減らし、theme 変数を import なしで書けるようにする |
| 2026-11-18 | RW-04 | 非同期の action の形（CONTRACT-05 を置き換える） | `action(f)` は新しい実行が古いものを取り消す。順に実行するものは `queued_action(f)`。`ActionMode`・`ActionOutcome`・`Action::close` を消し、`on_complete` は `Result<T, UiError>` を受ける。`Retry` は `Callback<()>`。非同期の表示は `async_view` だけを教える | 既定の意味を一つにし、完了の結果を std の型で表す |
| 2026-11-18 | RW-05 | 他の thread から reactive な状態に値を入れる口 | `task::remote(initial) -> (Signal<T>, Remote<T>)` と `task::remote_trigger() -> (Trigger, RemoteTrigger)`。書込みは UI の turn ごとに一度にまとめ、event loop を起こし、block しない。`Notifier` は `Trigger` に改名する | channel と `spawn` の定型を利用者に書かせない |
| 2026-11-18 | RW-06 | font の持ち主（ADR-25 の font の部分を置き換える） | theme が持つ（`Theme::with_fonts(&'static [&'static [u8]])`）。`App::fonts` と `TestFonts::Only` を消し、`App::theme` と `RenderOptions::with_theme` が登録する | kit の theme と font を一緒に配れ、試験と本番が同じ経路を使う |
| 2026-11-18 | RW-07 | GPU の起動の希望の置き場（CONTRACT-11 を置き換える） | 5 つの型を `kivact::platform` に置き、`kivact::launch` を消す。`GpuStart::Never` を足し、`prefer_gpu` を消す。`ARVANE_RENDERER=cpu\|gpu\|auto` は `Never\|AtLaunch\|OnDemand` | 1 つの型のための module をなくし、CPU だけの起動を同じ型で表す |
| 2026-11-18 | RW-08 | close request の handler が効く window（CONTRACT-15 を置き換える） | `platform::on_close_request(f)` は呼んだ owner の window（main window、`window()`、foreign window）に効く。main 以外の window では handler が `Handled::Yes` を返さなければ `on_dismiss(DismissReason::WindowClose)` が続く。`AppHandle::close` と埋め込み App は handler を呼ばない | 別 window の未保存の確認も同じ口で書け、`on_dismiss` の規則と揃う |
| 2026-11-18 | RW-09 | 埋め込みの停止と名前（ADR-72 を置き換える） | `EmbedNotice::{ScaleChanged(f64), Stopped(EmbedStatus)}`。停止は `Closed`・`Failed`・`Panicked` を payload で一度だけ知らせる。`embed::Visibility` を `EmbedVisibility` に改名し、`kivact::embed` は全 OS で compile できる（未対応の環境の `App::embed` は `Unsupported`） | wrapper が通知の後に status を読み直す手間と、OS ごとの cfg を利用者から除く |
| 2026-11-18 | RW-10 | 埋め込みの表示・非表示の表し方（ADR-67 を置き換える） | `ShellEvent::Visibility(bool)` で表す。`Suspended`・`Resumed`・`SurfaceDestroyed`・`SurfaceCreated` などモバイルの event は残す | 表示の切替えを surface の喪失と別の事実にする。モバイルの設計境界は先送りしない（戦略 P3） |
| 2026-11-18 | RW-11 | theme の値（ADR-27 を置き換える） | theme の値は theme 変数 `ThemeVar<T>`（`color::ACCENT` など）。`Color::var`・`Length::var` で const の値に置ける。解決は `compile_style` で行い、theme の切替えは style の effect の再実行になる。`ThemeColor`、`ColorToken`、process 全体の `THEMED` 表、`Palette`・`Spacing`・`Radii` を消す。`Theme` は `Arc` で共有し、`PartialEq` は pointer を先に比べる | CSS の `var(--x)` と同じ一つの仕組みで、closure なしに theme に追従する。process の可変の表を持たない |
| 2026-11-18 | RW-12 | 状態の style | `.hover(..)`、`.active(..)`、`.focus(..)`、`.focus_visible(..)`、`.disabled(..)`、`.selected(..)`、`.group_hover(..)`、`.group_focus(..)`、`.enter(..)`、`.exit(..)` はすべての property を受け、layout の property が変われば layout をやり直す | CSS の擬似 class と同じ。paint だけという例外を利用者に覚えさせない |
| 2026-11-18 | RW-13 | layer の合成の指定 | `Layer::mix(Mix)`（`Mix::{Normal, Blend(..), Composite(..)}`）が `blend`・`composite` を置き換える。CSS の値の型は `kivact::style` だけに置く | 後から呼んだ方が黙って勝つ二つの setter をなくす。同じ型を二つの module から出さない |
| 2026-11-18 | RW-14 | window なしの描画の置き場 | `kivact-offscreen` を新設し、facade が `kivact::draw::Offscreen` として再公開する。core は kivact-svg に依存しない | 入力（theme・font・描画の closure）と出力（Pixels・SVG）が小さく、view と runtime を読まない明確な境界 |
| 2026-11-18 | RW-15 | 数値の plot の置き場（ADR-65 を置き換える） | 新しい crate `kivact-plot`。色は `kivact_plot::color` の theme 変数で受け、どの kit とも組み合わせられる。範囲の操作は ADR-65 のまま（PlotRange、Drag、EditSession、`on_edit(EditPhase)`）。basic の chart は basic に残し、目盛りと scale の計算を共有する | 分析ツールという変更理由は kit の見た目と別。kit なしでも使える |
| 2026-11-18 | RW-16 | 描画の protocol の置き場 | `arvane-protocol` を新設し、`kivact-types/src/render` を移す。`arvane` は window driver と presenter だけにして再公開をやめる | renderer との契約という一つの変更理由を持ち、作るのは engine、使うのは arvane-* だけ |
| 2026-11-18 | RW-17 | DevTools の client の crate | `kivact-devtools-client` を `kivact-devtools-protocol` の feature `client` に統合する | client と protocol は同じ理由で同時に変わり、client だけを使う利用者がいない |
| 2026-11-18 | RW-18 | 開閉の語彙（CONTRACT-12 を置き換える） | interaction の surface の Api は `use_x(.., open: Signal<bool>)` で open を必須の位置引数に取り、`show`・`hide`・`toggle`・`is_open`、`trigger()`・`content()` の Part、`surface(placement, content)` を持つ。`use_menu_with_open`、`on_open_change`、`DialogMode::Popover`、`while_open` を消す | 開閉の書き方を一つにする。`surface` が anchor・`show`・overlay・content の Part の配線をまとめる |
| 2026-11-18 | RW-19 | 部品の状態と recipe の形 | part の状態を読む唯一の方法は `Part::state()`。recipe の上書きは `Fn(&Theme, &Look<S>, Style) -> Style`（3 つ目は前の recipe の結果）。ui は入力で動く Machine を持たず、interaction は view を返さない（`surface()` を除く） | 状態の getter・slot・map の写しをなくし、上書きを前の結果の上に重ねられるようにする |
| 2026-11-18 | RW-20 | 状態の検査の単位（ADR-52・STATE-AUDIT・STATE-CLAIMS を置き換える） | 型の単位の検査 `state`：状態の field を二つ以上持ち、`&mut self` か内部可変の書込みを持つ型は、型の直前の印 `// xtask-allow(state): <閉じ方>` が無ければ error。constructor・`Default`・値渡しの builder だけで書く field は状態ではない。field ごとの印、`--report-audit`、`state-audit.md` を消す | field ごとの印は受動的な record に 1000 行以上の定型文を書かせ、意味の誤りは見つけられなかった。検査を可変の状態に絞り、閉じ方は型の横の一行に置く |
| 2026-11-18 | RW-21 | 公開 API の E2E と文書の compile | E2E を `kivact-public-api-tests` の `e2e_core`・`e2e_ui`・`e2e_native` と、新しい package `kivact-kit-tests` の `e2e_kits` の 4 つの binary に分ける。文書の doctest は英語だけを compile し、日本語の Rust の code block は `docs-pairs` で英語と同一であることを確かめる。生成した図（`architecture/`）は git から外し、`cargo xtask diagrams` で作る | 変更した crate に合う binary だけを link でき、日本語の同じ例の二度目の compile を省く |
| 2026-11-18 | RW-22 | kit の骨格 | 5 つの kit を一つの骨格に揃える：`theme(mode)`、option があるときだけ `theme_with`、kit の theme 変数、`recipes_<group>.rs`、`variants.rs` の `<Component>Variant`、分野の部品は `src/<domain>/`。kit 名の接頭辞を型名に付けない。kit の README の移行表と recipe の一覧を消す | kit の間の差を見た目と分野の部品だけにする。公開していない kit に旧版からの移行表は要らない |
| 2026-11-18 | RW-23 | 診断の記録 | turn の trace・原因の文字列・node の provenance は DevTools・`kivact::test` の trace・debug の trace sink が有効なときだけ記録し、release で常に持つのは counter だけにする。DevTools は feature で外さず、実行時の `Option` の hook で費用を消す | 常時の記録がメモリと起動の大半を占めていた。機能の有無で API を分けない |
| 2026-11-18 | RW-24 | engine の配送の受理 | pointer の source の写しを消し、配送の受理（admission）は engine が持つ | 入力の route と modal の受理の正本を一つにする |
````

  4. **strategy.md**（この三か所だけ。他は変えない）：
     - 「## 5. 所有範囲」の表の `| 標準部品 | 中核（少数） | kit なしで最初の実用画面（一覧・表・入力と検証・通知・非同期の状態）を作れる数。見た目はテーマの token で決まる |` の行を `| 標準部品 | 中核 | kit なしで実用画面（一覧・表・入力と検証・通知・非同期の状態）を作れる数と、二つ以上の kit が共有する部品。見た目はテーマの変数と部品の recipe で決まる |` にする。
     - 同じ表の「試験と診断」の行の「DevTools（別プロセスの agent と MCP・Python の client）」を「DevTools（別プロセスの agent と MCP の client）」にする。
     - 「## 6. 取り込みの規則」の 4. の「（定数と Binding の二つの mode、...）」の「定数と Binding の二つの mode」を「定数と書込み先の二つの mode」にする（親の決定。AC-4 の `Binding` を残さない）。
  5. **manual-verification.md**：
     - ファイルの中の相対 link（`](../../...)`）をすべて列挙し、link 先が存在するか確かめる：
       ```bash
       cd docs/for-kivact-developer
       grep -o '](\.\./\.\./[^)]*)' manual-verification.md | sed 's/^](//; s/)$//' | sort -u | while read p; do [ -e "$p" ] || echo "missing: $p"; done
       cd ../..
       ```
     - 無い link 先は、同じファイル名を `git ls-files | grep -F '/<ファイル名>'` で探し、見つかった path に直す（`dev/tests/kivact-public-api/tests/e2e/x.rs` が `tests/e2e_core/x.rs` などへ移った場合）。候補が二つ以上か 0 個なら、その link を含む行の「変更経路の参照」の列からその link だけを消し、報告に書く。
     - 「`Embedded::status` が `Panicked` を返し」の文の後に「、`Parent::on_notice` が `EmbedNotice::Stopped(EmbedStatus::Panicked(..))` を一度受け取り」を足す（ADR-72 の置換 RW-09）。
     - 3.0.6 の旧名があれば新名に直す。
- **消すもの**：手順 1 の行と補足の箇条。
- **状態の閉じ方**：新しい型なし。
- **試験**：なし。
- **確認（tier a）**：
  ```bash
  rg -n -e 'ADR-27 ' -e 'ADR-37 ' -e 'ADR-47 ' -e 'ADR-52 ' -e 'ADR-64 ' -e 'ADR-65 ' -e 'ADR-67 ' -e 'ADR-72 ' -e 'CONTRACT-01 ' -e 'CONTRACT-05 ' -e 'CONTRACT-08 ' -e 'CONTRACT-10 ' -e 'CONTRACT-11 ' -e 'CONTRACT-12 ' -e 'CONTRACT-14 ' -e 'CONTRACT-15 ' -e 'STATE-AUDIT' -e 'STATE-CLAIMS' docs/for-kivact-developer/decisions.md
  rg -n -i python docs/for-kivact-developer/strategy.md
  git diff --stat -- docs/for-kivact-developer/strategy.md
  ```
- **完了条件**：1 つ目の rg が、新しい節の「置き換える」の説明の中の ID（`（CONTRACT-10 を置き換える）` など）だけを返す（表の ID の列には無い）。2 つ目が何も返さない。strategy.md の diff が 3 行（標準部品の行、試験と診断の行、6 章 4. の行）だけ。decisions.md に DEC-01〜DEC-16 と RW-01〜RW-24 の行がある。manual-verification.md の link 先がすべて存在する。

## G15：最終の整合（W11c）

- **wave / lane / 前提**：W11c / G1 / G01〜G14 の全部（W11a・W11b の全 unit）。加えて、W8 の U と W10 の K の `--strict` の報告が `ok` であること（SCHEDULE の W11c の前提）
- **所有**：`CONTRIBUTING.md`、`dev/README.md`、`dev/xtask/policy/workspace.toml`（`[rework.pending]` の表の削除だけ）、`dev/xtask/src/{pending.rs,local_checks.rs,check_cli.rs,main.rs,policy.rs}`（pending の部分だけ。T 4.4 の最後の段落）、AC-4 の検査で見つかった残りの箇所（`crates/`、`examples/`、`templates/`、`docs/`、`dev/` の doc comment・comment・文書・文字列だけ。コードの API は変えない）
- **目的**：AC-4（設計 5.12 の削除名が残っていない）と AC-5 を確かめ、残りを直す。報告だけの検査の一覧を消して全検査を error にする。`kivact-docs` を gate に戻す。bench の probe を確かめる（TL-03、AC-4、AC-5）。
- **手順**：
  1. **CONTRIBUTING.md を確かめて直す。** 付属書 T が書き直した前提で、次の rg がそれぞれ何も返さないことを確かめる。返したら、右の文に直す。
     | rg | 直し方 |
     |---|---|
     | `rg -n 'scope\(move' CONTRIBUTING.md` | 「コード」の節の「一度だけ mount する slot は `impl IntoView`、開くたびに作り直す内容は `impl Fn() -> V` で受け取ります。子の mount 時に setup と cleanup owner を作る場合は `scope(move \|\| view)` を使います。」を「部品は `#[component]` の関数で書きます。一度だけ mount する slot は `View`、開くたびに作り直す内容は `ViewFn` の props で受け取ります。」にする |
     | `rg -n -i python CONTRIBUTING.md` | Python の行・文を消す（「テスト」の表の `python/kivact-devtools/tests` の行、`python -m unittest` の文） |
     | `rg -n -e 'state-fields' -e 'report-audit' -e 'state-audit' -e 'structural-options' CONTRIBUTING.md` | 検査の一覧の `state-fields` を `state` にし、`structural-options` を消す。`--report-audit` の説明の括弧を消す。例外の印の説明を「例外は対象の型や項目の直前の `// xtask-allow(<検査>): <理由>`（`state` は型の直前の一行）」にする |
     | `rg -n -e 'git diff --quiet -- architecture' -e '図と `architecture/index.html` を生成' CONTRIBUTING.md` | その行を消す。`cargo xtask check` の説明の「`architecture/` の図と `architecture/index.html` を生成します」を消し、「図は `cargo xtask diagrams` で生成します（gate の外）」を足す |
     | `rg -n 'devtools-codegen' CONTRIBUTING.md` | その文を消す |
     | `rg -n -e '--test e2e ' -e '--test e2e$' CONTRIBUTING.md` | `--test e2e` を、GPU の試験なら `--test e2e_native`、その他は付属書 T の 4 章の対応に従う binary 名にする |
     | `rg -n -e '--exclude kivact-docs' CONTRIBUTING.md` | 「merge 前の検査」の `cargo test --workspace` の行の `--exclude kivact-docs` はそのまま（kivact-docs は別の行で直列に実行する）。`kivact-docs` を `--test-threads=1` で実行する行が `--all-features` の一つだけあることを確かめる（`--no-default-features` の `kivact-docs` の行があれば消す。T 4.1、INC-31）。無ければ現在の CONTRIBUTING の書式で `--all-features` の一行を足す |
     続いて「ドキュメント」の節の「英語と日本語は同じファイル名で対にし、...」の段落を次にする：
     ````markdown
     英語と日本語は同じファイル名で対にし、同じ変更で両方を更新します。日本語のファイルの Rust のコードブロックは、英語の同じ順番のブロックと一字一句同じにします（コメントと文字列も英語のまま。`cargo xtask check` の `docs-pairs` が確かめます）。`docs/` の英語の木と `README.md` の Rust のコードブロックは `dev/tests/docs`（`kivact-docs`）の doctest としてコンパイルされます。新しいページの登録は不要です。直したガイドだけを確かめるには `KIVACT_DOCS_ONLY=<docs/ からの path をコンマ区切り>` を付けて `cargo test -p kivact-docs --doc` を実行します（root の README は `README.md`）。GPU texture 共有のガイドの例は `kivact-docs` の `gpu-interop` feature（`--all-features` で有効）のときだけコンパイルされます。公開 API を変えたときは、移行ガイド `docs/for-kivact-user/{en,jp}/how-to-guide/migrating-to-rework.md` には足さず、ガイドの本文を新しい使い方に揃えます。
     ````
     「ドキュメント」の表の `strategy.md` の行の後に `| docs/for-kivact-user/{en,jp}/how-to-guide/migrating-to-rework.md | 改修前の API からの移行の表（改修の時点で固定） |` を足す。
  2. **報告だけの検査の一覧と pending のコードを消す**（付属書 T の 4.4 の最後の段落のとおり）。
     1. 先に、pending の行の検査がすべて通っていることを確かめる：`cargo xtask check --no-write --only local --strict` が `ok`。`ok` でなければ、出た検査・package・file を一覧にして STOP し、Orchestrator に報告する（その検査を通す付属書 U・C・K の未完。ここで直さない）。
     2. `dev/xtask/policy/workspace.toml` の `[rework.pending]` の表を、直前の comment の行を含めて消す（見出しの行から次の `[` の見出しの直前まで）。
     3. `dev/xtask/src/pending.rs` を消し、`dev/xtask/src/main.rs`（または `lib.rs`）の `mod pending;` の行を消す。
     4. `dev/xtask/src/local_checks.rs` の `Scope::route` の pending の分岐（`self.pending.covers(..)` の `if`）を消し、常に `outcome.errors.extend(found)` にする。`Scope` の `pending` の field と、その構築を消す。`Outcome::notes` の field とその書込みを消す。
     5. `dev/xtask/src/check_cli.rs` の `strict` の field と `--strict` の引数の解析を消す。
     6. `dev/xtask/src/main.rs` の `report.notes.extend(local.notes)` の行を消す（`notes` の表示が他に無ければ、その表示も消す）。
     7. `dev/xtask/src/policy.rs` の `table.remove("rework")` の行を消す。
     8. `CONTRIBUTING.md` と `dev/README.md` の pending（`[rework.pending]`、`--strict`、「報告だけ」）の段落を消す（`rg -n -e 'rework.pending' -e '--strict' -e '報告だけ' CONTRIBUTING.md dev/README.md`）。
     9. 確かめる：`cargo fmt -p xtask`、`cargo clippy -p xtask --all-targets -- -D warnings`、`cargo test -p xtask`、`rg -n -e 'pending' -e 'strict' dev/xtask/src` が何も返さない（pending と無関係な語の行があれば報告に書く）。
  3. **全検査を走らせる。**
     ```bash
     cargo xtask check --no-write
     ```
     error が出たら分類する：(a) 文書（`docs-pairs`）の error は、対のファイルを 3.0.2 の script で直す。(b) コードの error（`state`、module の規則、`impl Element for` の検査、`Element` と同名の固有 method など）は、この unit では直さない：検査の名前・ファイル・型の名前を一覧にして STOP し、Orchestrator に報告する（その検査を所有する付属書の修正が要る。印を足して黙らせない）。
  4. **AC-4 の検査を走らせる。** 作業指示書 2 章の AC-4 のコマンド（一つだけ。付属書 T の 4.2 と INC-13 から作られたもの）をそのまま実行する。本付属書は自分の写しを持たない。出力の各行は、下の「出力の扱い」で分類する（作業指示書 2 章にも同じ文が写されている）。

     <!-- AC-4 出力の扱い（作業指示書 2 章へ写す block。ここから） -->
     **AC-4 の出力の扱い**

     - (i) 付属書 R の 4 章が `constant_value` を許すと列挙した場所（`#[doc(hidden)]` の定義と内部の利用）の行は残す。
     - (ii) `AppOptions` が `pub` でない型・field の名前として crate の内部にある行は残す（5.12 は公開の path だけを禁じる）。
     - (iii) `Composite` は `Mix::Composite(..)` と `Composite::<variant>` の中の行を対象にしない（`.composite(` だけを見る）。filter graph の primitive と `MaskLayer` の `.composite(` は残す（`Layer` の method ではない）。
     - (iv) `.wrap(`・`.direction(`・`.radius(`・`.grow(` などは、受け手が `Style`（`style()` の鎖、`Style` の値）のものだけが旧名。受け手が部品（`text_area(..).wrap(true)`、code block の `.wrap(true)`）、`Draw`、`Shadow`、`Transform` などの値のものは残す。
     - (v) 文字列の literal の中の英語の語（button の label の `"Retry"` など）と、Kivact の API を指さない英語の本文の語（文頭の「Reactive」、DevTools の「Palette」の UI 名など）は残す。
     - それ以外の行は旧名の残りである。文書・comment・doc comment・文字列の中なら、移行の置換表（`docs/for-kivact-user/en/how-to-guide/migrating-to-rework.md`）で新しい名前に直す。コードの識別子として残っていたら STOP し、ファイルと名前を Orchestrator に報告する（その名前を所有する付属書の削除漏れ）。
     - 残した行（(i)〜(v)）は、ファイル・行・理由を報告に一覧にする。

     <!-- AC-4 出力の扱い（ここまで） -->
  5. **文書の全体の compile。** `KIVACT_DOCS_ONLY` を付けずに：
     ```bash
     dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --all-features -- --test-threads=1
     ```
     失敗した doctest は、そのファイルを所有した unit の手順で直す（1.2 の 1. の規則）。
  6. **bench の probe を確かめる。**
     ```bash
     cargo check -p kivact-bench --all-targets
     ```
     4. の AC-4 のコマンドの出力に `dev/bench` の行があれば、4. の「出力の扱い」で扱う。
  7. **消さないもの。** `REWORK_DESIGN.md`、`REWORK_WORK_ORDER.md`、`rework/` は、作業指示書の指示（PR が merge されるまで残す）どおり残す。`git status --short REWORK_DESIGN.md REWORK_WORK_ORDER.md rework` で削除の印（`D`）が無いことを確かめる。
- **消すもの**：`[rework.pending]` の表、`dev/xtask/src/pending.rs`、`local_checks.rs` の pending の分岐と `Outcome::notes`、`check_cli.rs` の `--strict`、`main.rs` の notes の行、`policy.rs` の `table.remove("rework")`、CONTRIBUTING と `dev/README.md` の pending の段落、CONTRIBUTING の旧い記述（手順 1 の表）。
- **状態の閉じ方**：新しい型なし。
- **試験**：`kivact-docs` の全 doctest（`--all-features` だけ）。xtask の自分の試験（`cargo test -p xtask`）。
- **確認（tier a）**：
  ```bash
  cargo xtask check --no-write --only local --strict   # 手順 2 の前に
  cargo fmt -p xtask && cargo clippy -p xtask --all-targets -- -D warnings && cargo test -p xtask
  cargo xtask check --no-write
  # 作業指示書 2 章の AC-4 のコマンド
  dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --all-features -- --test-threads=1
  cargo check -p kivact-bench --all-targets
  ```
- **完了条件**：`cargo xtask check --no-write` が error 0（`state` の検査を含む全 workspace）。`[rework.pending]` の表が policy に無く、`dev/xtask/src/pending.rs` が無く、`rg -n -e '--strict' -e 'rework' dev/xtask/src CONTRIBUTING.md dev/README.md` が何も返さない。AC-4 のコマンドの出力が、報告に一覧にした (i)〜(v) の行だけ。`kivact-docs` の全 doctest が `--all-features` で通る。`kivact-bench` が check を通る。`REWORK_*.md` と `rework/` が残っている。

## 4. 他の付属書との接点

### 4.1 この付属書が他の付属書に頼る名前（本付属書のコードが使う前提）

本付属書の文書のコードは次の名前を使う。違う名前で実装されていたら、各付属書の 4 章の名前に文書を合わせる（1.2 の 1.）。

| 付属書 | 名前・署名 | 使う unit |
|---|---|---|
| R | `Signal<T>`（`get`、`with`、`set`、`update`（値を返さない。DEC-15）、`optional`）、`binding(get, set) -> Signal<T>`、`bind!(s.a)`、`var(v) -> Var<T>`（`get`、`with`、`set`、`update`）、`callback(f)`、`trigger() -> Trigger`（`track`、`notify`）、`use_context::<T>() -> T`、`try_use_context::<T>() -> Option<T>`、`untrack`、`kivact::state::selector`、`kivact::state::{IntoValue, IntoText, IntoHandler}` | G02、G03、G04〜G08 |
| R | `kivact::task::{remote, remote_trigger, queued_action, clock}`、`action(f)`（nullary と unary の closure）、`Action::on_complete(Fn(Result<T, UiError>))`、`Resource::refetch`、`resource((), f)`（`()` の source） | G02、G03、G07 |
| C | `#[component]` が `-> impl Element` と `-> impl IntoView` を受け、生成する struct の名前は関数名の UpperCamel（`task_row` → `TaskRow`）。`Option<X>` の props は setter。`#[prop(default = ..)]`。`#[derive(Prop)]`。prelude（設計 5.11）。`App::new(..).theme(..)`、`App::gpu_preference`、`App::handle`。`kivact::platform::{GpuPreference, GpuPower, GpuBackend, GpuStart, AdapterMatch}`。`kivact::view::commit_focused_edit`。`kivact::custom::{scope, raise, Owner}`。`kivact::draw::Offscreen`（`render(width, height)`、`render_svg(width, height)`、`.scale(f64)`）。`kivact::embed::{EmbedNotice::Stopped(EmbedStatus), EmbedVisibility}`。`kivact::test::RenderOptions::with_theme`、`TestFonts::System` | G02〜G12 |
| S | 1.2 の 5. の theme 変数の名前、`ThemeVar::new` が `const fn`、`rgb` が `const fn`（`static` の初期化子に置くため）、`Color::var`、`Length::var`（`kivact::style::Length`）、`Theme::{set, get, with_fonts, with_icon, with}`、`kivact::style::cached_theme`、`FlexDirection`、`FontStyle`、CSS の名前の setter、`Mix`、`TextStyle`（`StyledRange` の形） | G02、G03、G04、G06 |
| U | `kivact::ui::theme::{ButtonTheme, Variant, ..}` と部品ごとの `<Name>Theme` の part の名前、部品の theme 変数の module、`text_input(value).label(..)` などの setter、`field(label, control)`、`icon_button`、`use_toaster`、`use_popup(open)` と `surface(placement, content)`、`use_menu(items, open)`、`Part::state()`、`select(..).trigger(view)`、`input_group(label, children)` が closure を受けること、`stepper(items, current: Signal<usize>)`、`tag_input(tags)` | G04〜G08 |
| K | 付属書 K の 4 章の kit ごとの旧名→新名の表、kit の theme 変数（`<kit>::PRIMARY`、`PRIMARY_VAR`）、`kivact_plot` の公開名と `kivact_plot::color::SERIES_1..SERIES_5`、catalog の shell の package 名 | G10、G11、G12、G13 |
| T | `docs-pairs` の検査（`cargo xtask check --only docs-pairs`）、`dev/tests/docs/build.rs` が英語の木と `README.md` だけを生成すること、報告だけの検査の一覧 `[rework.pending]` と、それを読む xtask のコード（T 4.4。G15 が消す）、e2e の binary（`kivact-kit-tests` の `e2e_kits` を含む）、`cargo xtask diagrams`、CONTRIBUTING の「テスト」「merge 前の検査」の書換え、JSON Schema の再生成の手順 | G01、G13、G15 |

### 4.2 この付属書が後に渡すもの

- AC-4 のコマンドは作業指示書 2 章に一つだけ置く（INC-13）。除外は `migrating-to-rework.md`（英日）、`docs/for-kivact-developer/decisions.md`、`REWORK_*.md`、`rework/**`、`target/**`。`ButtonTone` は game の path だけ、`TokenKind` は DevTools の 3 path を除く（T 4.2 のまま）。出力の扱いは G15 の手順 4 の「AC-4 の出力の扱い」の block を写す。
- `KIVACT_DOCS_ONLY`（`dev/tests/docs/build.rs`）。CONTRIBUTING に書く。作業指示書の tier a の文書の確認に使える。
- 作業指示書の tier b・c のコマンドの `--exclude kivact-docs` は、G01 の完了から外す（Orchestrator。本付属書は作業指示書を編集しない）。`kivact-docs` は `--all-features` だけで実行する（INC-31）。G15 は CONTRIBUTING の「merge 前の検査」に `kivact-docs` の `--all-features` の 1 行があることを確かめる。

### 4.3 親が決めたこと（旧 要設計判断）

1. **`Signal::update` の戻り値。** 解決済み：DEC-15 で `update` は値を返さない。`on_press(move || x.update(..))` はそのまま handler になる。文書に `{ x.update(..); }` の書き方を使わない（INC-34）。
2. **strategy.md の 6 章 4. の `Binding`。** 直す（「定数と書込み先の二つの mode」）。G14 の手順 4 に入れた。
3. **architecture.md の「具体的な追加経路の机上確認」の節。** 消す（G13）。改修後の追加経路は「追加・変更するときのチェックリスト」の表が持つ。
4. **雛形の README の release profile。** Kivact の crate の行を消し、今も依存にある第三者の crate の行だけを残す（G12 の手順 3）。
5. **plot のガイドの置き場。** `docs/for-kivact-user/*/how-to-guide/plot.md` に移す（G10）。
6. **DevTools の Scripts panel。** 付属書 T の T23 が panel を消す。文書は G07 で Python の節と Scripts panel の言及を消す。
7. **`#[component]` と型引数。** 設計 5.1 の規則 9：型引数を受け、lifetime と const の引数は拒む（3.0.4 の 7.）。

## 5. 自己点検

### 5.1 担当の所見と unit

| 所見・条件 | unit |
|---|---|
| API-07（forms の `Signal::field`、README の `scope`、`kivact::view::Reactive` などの文書の誤り） | G04（forms）、G12（README）、G03（custom-components の誤りを含む全文の書換え）。rustdoc の誤り（`props.rs`、`element_data.rs`、`lib.rs`、`state.rs`）は付属書 R・C の担当（コードの doc comment）で、G15 の AC-4 の検査が残りを拾う |
| API-08（非同期の表示は `async_view`） | G02（07-async）、G03、G07、G12（例の README） |
| API-09（例・雛形の工程の痕跡） | G12（doc comment・README）。`#![warn(redundant_imports)]` と `xtask-allow` の削除は付属書 T・C |
| KIT-16（kit の docs。日本語は全ページ残す、移行表と recipe の一覧を消す） | G10、G11、G12（crate の README） |
| TL-03 の文書の部分（日本語の Rust の block を英語と同一にする） | 3.0.2 の script を全文書の unit が使う。G15 が全体を確かめる |
| TL-09 の文書の部分（Python の節） | G07（devtools.md）、G09（索引）、G12（README）、G14（strategy）、G15（CONTRIBUTING） |
| AC-4（5.12 の削除名） | G15 |
| AC-5（文書・README・例・雛形・移行ガイド・開発者向け 3 文書） | G02〜G14 |
| 設計 3 章の profile と版の固定の案内（USR-01、USR-13） | G02（01）、G09（移行ガイド・入口）、G12（README・雛形） |
| 設計 4 章・6 章を architecture.md へ | G13 |
| 設計 7・8 章を decisions.md へ（DEC-01〜DEC-14 と主な採用所見、置き換えた行の削除） | G14 |
| strategy.md の 2 か所（7.5 Q1、DEC-10） | G14 |
| `kivact-docs` を compile と gate に戻す | G01（workspace・依存）、G15（CONTRIBUTING・全体の doctest） |
| 報告だけの検査の一覧を消す | G15 |
| bench の probe の最終確認 | G15 |
| `REWORK_*.md` と `rework/` を消さない | G15 の手順 7 |

### 5.2 設計の判断を残していないか

- 各 unit の手順は、置き換える文・コードを全文か、検索できる元の文と置換後の文で書いた。名前が他の付属書で決まるもの（部品の theme の part 名、kit の名前、`Offscreen` の署名）は「付属書 X の 4 章に合わせる」と正本を一つに指定した（実装者が選ぶ余地は無い）。
- 条件付きの手順（G12 の README の節の有無、G15 の CONTRIBUTING の各 rg）は、すべて rg の結果で決まる。
- 設計の判断が要ったもの（7 件）は親が決め、4.3 にその答えと反映先を書いた。

### 5.3 同じ wave の unit が同じファイルを所有していないか

W11a（G01、G13、G14）と W11b（残りの G02〜G12）の所有（W11a の付属書 K の K44 は `dev/tests/kivact-kit-tests/` と kit の `tests/consumer/` だけで、下のどれとも重ならない）：

- G01：`dev/tests/docs/**`（root `Cargo.toml` は所有しない）
- G02：`docs/for-kivact-user/{en,jp}/getting-started/**`
- G03：how-to の 2 ファイル（coming-from、custom-components）
- G04：how-to の 6 ファイル（forms、lists-and-tables、overlays、windows、layout、images-and-fonts）
- G05：how-to の 9 ファイル（collections、navigation、keyed-suggestions、disclosure-and-previews、feedback、indicators、metrics-and-description、chat-and-code、slides-and-files）
- G06：how-to の 9 ファイル（theming、i18n、animation、timelines-and-scroll、3d-and-depth、effects、shaders、canvas、drawables）
- G07：how-to の 9 ファイル（async-and-tasks、http-json、platform-services、testing、diagnostics、devtools、embed、foreign-windows、gpu-texture-sharing）
- G08：how-to の 1 ファイル（custom-controls）
- G09：how-to の `migrating-to-rework.md`（新規）、how-to の `README.md`、`docs/for-kivact-user/{en,jp}/README.md`
- G10：`docs/for-kivact-ui-user/{en,jp}/{basic,pop,game}/**`、how-to の `plot.md`（新規）
- G11：`docs/for-kivact-ui-user/{en,jp}/{creative,soft}/**`、`docs/for-kivact-ui-user/{en,jp}/README.md`
- G12：`README.md`、`README_JP.md`、`templates/basic/{README.md,src/main.rs の doc comment}`、`examples/README.md`、`examples/kivact/**` の README と doc comment、`crates/*/README.md`
- G13：`docs/for-kivact-developer/architecture.md`
- G14：`docs/for-kivact-developer/{decisions,strategy,manual-verification}.md`

how-to の 37 ファイルは G03〜G09 にちょうど一度ずつ割り当てた（2 + 6 + 9 + 9 + 9 + 1 + README 1 = 37）。新規の 2 ファイル（`migrating-to-rework.md` は G09、`plot.md` は G10）も一つの unit だけが作る。G09 の索引は `plot.md` に link するが、そのファイルを所有しない。

W11c の G15 は W11a・W11b の全 unit の後に単独で動く。CONTRIBUTING.md、`dev/README.md`、policy、`dev/xtask/src` は前の段のどの G unit も所有しない。G15 が直す「残りの箇所」は前の段の unit が終わった後なので衝突しない。
