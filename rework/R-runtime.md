# 付属書 R：runtime・i18n と状態・handler・非同期の公開 API

版 1.0（2026-11-18）。基点 `2b8df5ac`。本書の行番号は書かない。場所は関数名・型名と、そのまま引用したコードの行で示す。

## 1. 範囲と前提

### 1.1 この付属書が扱うもの

- crate：`kivact-runtime`（全体）、`kivact-i18n`（全体）。
- 公開 API：設計書 5.2 節（状態）、5.3 節（handler）、5.5 節（非同期）。prelude（5.11 節）と macro の本体は付属書 C が持つ。本書は、本書が消す・改名する名前を prelude・macro・facade から外す最小の編集だけを行う。
- 所見：
  - runtime-core.md：RC-01〜RC-08、RC-14〜RC-17、RC-23〜RC-26、RC-30〜RC-35、RC-37〜RC-39、RC-41 のうち runtime と i18n の小項目（`mount_revision`、`report_now`、`Owner::ambient`/`Owner::current` の改名、i18n の依存。`notify_from_render` は付属書 C の求めで残す。5 章）。
  - api.md：API-02、API-03、API-06（runtime の部分）、API-11（`Callback` の部分）、API-15。
  - 利用者の調査：USR-02（`task::remote`・`task::remote_trigger`）、USR-03（`Trigger`）、KIT-17（`task::clock`）。
  - 設計書 6.2 節の PERF-9（診断の記録は記録者が有効にしたときだけ）。

### 1.2 前提（他の付属書が先に終えていること）

- 付属書 S の W1a の unit が `kivact_types::cell::{Deferred, SharedDeferred}` を作っている（`const fn new()`、`get_or_init(&self, f) -> &T`、`get(&self) -> Option<&T>`）。R01・R05 が使う。
- 付属書 T の W1a の unit が xtask の `state` 検査（設計書 6.1 節）を入れ、W1b が runtime・i18n・core の `xtask-allow(state-fields)` を `xtask-allow(state)` に変換または削除している。本書の「状態の閉じ方」は新しい `state` 検査に対して書く。
- 付属書 T の W1（T15）が e2e を `dev/tests/kivact-public-api/tests/{e2e_core,e2e_ui,e2e_native}/` と kit の試験の package `dev/tests/kivact-kit-tests` に分けている。本書が名前を挙げる試験のファイルは `dev/tests/kivact-public-api/tests/*/<topic>.rs` の形で書き（`<topic>` は `runtime`・`task` など）、場所は `rg -n 'fn <名前>' dev/tests` で確かめる。
- 窓を開く package（`kivact-public-api-tests`、`kivact-kit-tests` など。付属書 T 4.1 の一覧）の `cargo test` は、本書の「確認」の行のとおり `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test ...` で実行する（INC-15）。

### 1.3 後の付属書に渡すもの

- W5a の鎖の先頭（R05、R06、R08、R09、R20〜R31）が、状態・handler・非同期・診断の公開 API を最終形にし、全 crate の呼出し元を直す。付属書 C の鎖の unit は R31 の後に始まる。
- C が使う名前（4 章の表）：`Handler<E>`、`IntoHandler<E, M>`、`Callback::from_handler`、`Prop<T>`（hidden）、`AppParts`、`Runtime::app_state`、`Runtime::defer`、`Runtime::has_work`、`RecordBits`、`Cause`、`driver` module。

## 2. unit の一覧

wave と lane は設計書 9 章の枠に従う。R の lane は各 wave に一つだけで、同じ wave の R の unit は表の順に一つずつ実行する（前の unit が終わってから次を始める）。W5a の「鎖」は一つの lane で、R05、R06、R08、R09、R20〜R31 をこの順に実行し、その後に付属書 C の鎖の unit が続く。

| ID | wave | lane | 前提の unit | 所有するファイル・directory | 所見 | 規模 |
|---|---|---|---|---|---|---|
| R01 | W2a | R | S の W1a（Deferred） | `crates/kivact-i18n/`（`Cargo.toml`、`src/lib.rs`、`src/locale.rs`、`src/numbers.rs`、`src/regional.rs`、新規 `src/cache.rs`） | RC-14 | M |
| R02 | W2a | R | R01 | runtime `src/reactive/{selector.rs,owner.rs,frame.rs,data.rs,runtime.rs}`、`src/task/{state.rs,time.rs}` | RC-24、RC-25、RC-26 | M |
| R03 | W2a | R | R02 | runtime `src/reactive/{graph.rs,frame.rs,turn.rs,data.rs,runtime.rs,registry.rs,handles.rs}` | RC-07 | M |
| R04 | W2a | R | R03 | runtime `src/machine.rs`、`src/task/{blocking.rs,state.rs,time.rs}`、`src/reactive/{diag.rs,data.rs,mounts.rs,runtime.rs}`、`src/reactive.rs`、`src/lib.rs`、core `src/devtools/console.rs` | RC-17、RC-38、RC-39、RC-41（`mount_revision`）、C の R-REQ-9（`reported_since`） | M |
| R05 | W5a | 鎖（先頭） | CP1、R04、R07、S の W1a | runtime `src/host.rs`、`src/lib.rs`、`src/reactive/{data.rs,ext.rs,runtime.rs,registry.rs,selector.rs,owner.rs}`、`src/motion/**`、`src/task/**`、core `src/launch/{runtime.rs,inspect.rs,close.rs}`、`src/client/{state.rs,trace.rs,dispatch/env.rs}`、`src/platform/{app_handle.rs,close_request.rs,lifecycle.rs}`、`src/gpu_interop/import/registry.rs`、`src/view/{state.rs,foreign.rs,devtools_state.rs}`、`src/devtools/{perf.rs,motion.rs}`、新規 core `src/app_state.rs`、core `src/lib.rs`（`mod` 行） | RC-04 | L |
| R06 | W5a | 鎖 | R05 | runtime `src/host.rs`、`src/lib.rs`、`src/reactive/{host.rs,host_types.rs,clock.rs,turn.rs,reactive.rs}`、core `src/launch/{runtime.rs,inspect.rs,turn.rs(新規)}`、`src/launch.rs`、`src/view/{requests.rs,state.rs,node_ref.rs}`、`src/client/dispatch.rs`、`src/client/dispatch/pointer.rs`、`src/app_state.rs` | RC-01、RC-02、RC-03、RC-41（`report_now`） | L |
| R07 | W3a | R | R04 | runtime `src/motion/{state.rs,frame_loop.rs,animated.rs,timeline/api.rs}` | RC-08 | M |
| R08 | W5a | 鎖 | R06 | runtime `src/reactive/{diag.rs,detail.rs,data.rs,graph.rs,host.rs,runtime.rs,reactive.rs}`、`src/host.rs`、`src/lib.rs`、core `src/launch/runtime.rs`、`src/launch/turn.rs`、`src/client/trace.rs`、`src/devtools/lifecycle.rs` | RC-05（記録の opt-in。型は変えない） | M |
| R09 | W5a | 鎖 | R08 | runtime `src/reactive/{data.rs,graph.rs,turn.rs,owner.rs,arena.rs,selector.rs,frame.rs,handles.rs,host.rs,diag.rs,scope.rs,object.rs,trigger.rs}` | RC-06 | L |
| R10 | W3a | R | R07 | core `src/view/flow/virtual_list.rs`、新規 core `src/view/flow/virtual_list/heights.rs` | RC-23（`virtual_list` の部分） | M |
| R20 | W5a | 鎖 | R09 | runtime `src/reactive/{data.rs,handles.rs,setup.rs,props.rs,sealed.rs,reactive.rs}`、`src/lib.rs` | API-02、API-15、API-11（`Callback`）、RC-31（runtime の半分） | L |
| R21 | W5a | 鎖 | R20 | runtime `src/reactive/{handler.rs,handles.rs,data.rs,setup.rs,sealed.rs,reactive.rs}`、`src/lib.rs`、`src/motion/{animated.rs,timeline/api.rs,state.rs,frame_loop.rs}`、`src/task/action.rs`、`dev/xtask/src/api_shape.rs` | API-03、RC-33（runtime の部分） | M |
| R22 | W5a | 鎖 | R21 | runtime `src/task/**`、`src/task.rs`、`src/lib.rs`、`src/host.rs` | RC-15、RC-16、RC-35、RC-37 | L |
| R23 | W5a | 鎖 | R22 | runtime `src/reactive/{trigger.rs,setup.rs,owner_handle.rs,handles.rs,reactive.rs}`、`src/task/{remote.rs(新規),clock.rs(新規),resource.rs,state.rs}`、`src/task.rs`、`src/motion/{frame_loop.rs,state.rs,timeline/api.rs}`、`src/lib.rs` | USR-02、USR-03、KIT-17、RC-34（runtime） | M |
| R24 | W5a | 鎖 | R23 | runtime `src/reactive/{props.rs,sealed.rs,reactive.rs}`、`src/motion/animated.rs`、`src/lib.rs` | RC-32、API-06（runtime） | M |
| R25 | W5a | 鎖 | R24 | runtime 全体（`src/lib.rs`、新規 `src/driver.rs`、`src/reactive/{diag.rs,detail.rs,data.rs,graph.rs,host.rs,owner_handle.rs,scope.rs}`、`src/task/spawn.rs`、`src/machine.rs`、`src/motion/**` の `Owner::current` 呼出し） | RC-05（型の統合）、RC-30、RC-35（`spawn_in`）、RC-41（`Owner` の改名） | L |
| R26 | W5a | 鎖 | R25 | `crates/kivact-core/`（R20〜R22 の変更の追従）、`crates/kivact-macros/src/{expand.rs,parse.rs}`、`crates/kivact/src/prelude.rs` | R20〜R22 の呼出し元、RC-23（list の部分） | L |
| R27 | W5a | 鎖 | R26 | `crates/kivact-core/`（R23〜R25 の変更の追従）、`crates/kivact-macros/src/{expand.rs,parse.rs}`、`crates/kivact/src/{prelude.rs,lib.rs}`、`dev/xtask/policy/workspace.toml`（`except` の 1 項目）、`crates/kivact-i18n/`、`crates/kivact-style/` | R23〜R25 の呼出し元、RC-34（core） | L |
| R28 | W5a | 鎖 | R27 | `crates/kivact-interaction/` | 全 API の呼出し元 | L（機械的） |
| R29 | W5a | 鎖 | R28 | `crates/kivact-ui/` | 全 API の呼出し元 | L（機械的） |
| R30 | W5a | 鎖 | R29 | `crates/kivact-ui-basic/`、`crates/kivact-ui-soft/`、`crates/kivact-ui-pop/`、`crates/kivact-ui-game/`、`crates/kivact-ui-creative/`、`examples/kivact-ui-*/` | 全 API の呼出し元 | L（機械的） |
| R31 | W5a | 鎖 | R30 | `crates/kivact-devtools/`、`crates/kivact-test/`、`dev/tests/`、`dev/bench/`、`examples/kivact/`、`templates/` | 全 API の呼出し元、新しい E2E | L（機械的） |

同じ wave で二つの R の unit が同じファイルを持つことはあるが、lane が一つで順に実行するので衝突しない。他の付属書の lane との衝突を避けるため、R が W2〜W4 に持つ core のファイルを 4 章 4.3 に列挙する。

## 3. unit の手順

共通の約束（全 unit）：

- 場所は関数名・型名と引用した行で探す。見つからないときは推測で直さず STOP する。
- 「確認（tier a）」の `cargo xtask check` が `state` 検査で落ちたら、本書のその unit の「状態の閉じ方」に書いた印の 1 行だけを使う。それ以外の印・`#[allow]`・`#[expect]` を足さない。
- `perl` は Git Bash で実行する。ファイルの列挙は `git ls-files '<dir>/*.rs'`（git の pathspec の `*` は `/` をまたぐので再帰的に列挙される）。
- 「`cargo check` の誤りに従って直す」と書いた箇所は、本書がその誤りの種類ごとに書いた直し方だけを使う。種類が本書に無い誤りが出たら STOP する。

## R01：i18n の ICU object を locale ごとに一度だけ作る

- **wave / lane / 前提**：W2a / R / S の W1a（`Deferred`）。
- **所有**：`crates/kivact-i18n/Cargo.toml`、`crates/kivact-i18n/src/lib.rs`、`src/locale.rs`、`src/numbers.rs`、`src/regional.rs`、新規 `src/cache.rs`。
- **目的**：`format_date`・`parse_date`・`format_time`・`hour_cycle`・`month_name`・`weekday_name`・`plural_category`・`compare`・`format_number`・`parse_number`・`first_day_of_week` が呼出しごとに ICU の object と locale の解析を作り直すのをやめる（RC-14）。公開 API は変えない。

### 手順

1. `crates/kivact-i18n/Cargo.toml` の `[dependencies]` に次の 1 行を足す（`kivact-runtime` の行の直前）。設計書 6.2 節（DEC-13）が ICU の formatter を `Deferred` で持つと定めているので、この依存は設計が名指ししたものとして扱う。
   ```toml
   kivact-types = { path = "../kivact-types", version = "0.1.0" }
   ```
2. 新規 `crates/kivact-i18n/src/cache.rs` を次の内容で作る。
   ```rust
   //! Role: Per-thread caches of each locale's ICU objects, built on first use.
   //!
   //! A `Locale` is a tag; what ICU needs to format for it (the parsed
   //! locale, the formatters, the collator, the plural rules, the names) is
   //! built the first time it is asked for and kept in a per-thread cache of
   //! the last `CACHED_LOCALES` tags. ICU objects are not shared between
   //! threads, so a `Locale` stays `Send + Sync`.

   use std::cell::RefCell;
   use std::cmp::Ordering;
   use std::rc::Rc;
   use std::sync::Arc;

   use icu_calendar::Gregorian;
   use icu_datetime::fieldsets::{T, YMD};
   use icu_datetime::{FixedCalendarDateTimeFormatter, NoCalendarFormatter};
   use icu_decimal::DecimalFormatter;
   use kivact_types::cell::Deferred;

   use crate::numbers::NumberSymbols;
   use crate::{HourCycle, NameWidth, Weekday};

   /// How many locales one thread keeps (an App uses one or two).
   const CACHED_LOCALES: usize = 16;

   /// A display name kept by `LocaleCache::names`.
   #[derive(Clone, Copy, PartialEq, Eq)]
   pub(crate) enum NameKey {
       Month(NameWidth, u8),
       Weekday(NameWidth, Weekday),
   }

   /// The ICU objects of one locale, each built on first use.
   pub(crate) struct LocaleCache {
       pub(crate) tag: Arc<str>,
       icu: Deferred<icu_locale_core::Locale>,
       pub(crate) first_day: Deferred<Weekday>,
       pub(crate) hour_cycle: Deferred<HourCycle>,
       pub(crate) collator: Deferred<Box<dyn Fn(&str, &str) -> Ordering>>,
       pub(crate) plurals:
           Deferred<Box<dyn Fn(&icu_decimal::input::Decimal) -> icu_plurals::PluralCategory>>,
       pub(crate) symbols: Deferred<NumberSymbols>,
       /// Decimal formatters by grouping: `[Grouping::None, Grouping::Locale]`.
       pub(crate) decimals: [Deferred<DecimalFormatter>; 2],
       /// Date formatters by length: `[short, medium, long]`.
       pub(crate) dates: [Deferred<FixedCalendarDateTimeFormatter<Gregorian, YMD>>; 3],
       /// Time formatters: `[minutes, seconds]`.
       pub(crate) times: [Deferred<NoCalendarFormatter<T>>; 2],
       /// Month and weekday names already formatted.
       names: RefCell<Vec<(NameKey, String)>>,
   }

   impl LocaleCache {
       fn new(tag: Arc<str>) -> Self {
           LocaleCache {
               tag,
               icu: Deferred::new(),
               first_day: Deferred::new(),
               hour_cycle: Deferred::new(),
               collator: Deferred::new(),
               plurals: Deferred::new(),
               symbols: Deferred::new(),
               decimals: [Deferred::new(), Deferred::new()],
               dates: [Deferred::new(), Deferred::new(), Deferred::new()],
               times: [Deferred::new(), Deferred::new()],
               names: RefCell::new(Vec::new()),
           }
       }

       /// The parsed locale (a tag ICU cannot parse is `UNKNOWN`).
       pub(crate) fn icu(&self) -> &icu_locale_core::Locale {
           self.icu.get_or_init(|| {
               self.tag
                   .replace('_', "-")
                   .parse()
                   .unwrap_or(icu_locale_core::Locale::UNKNOWN)
           })
       }

       /// The name `key` formats to, computed by `make` the first time.
       pub(crate) fn name(&self, key: NameKey, make: impl FnOnce() -> String) -> String {
           if let Some((_, name)) = self.names.borrow().iter().find(|(k, _)| *k == key) {
               return name.clone();
           }
           let name = make();
           self.names.borrow_mut().push((key, name.clone()));
           name
       }
   }

   thread_local! {
       static CACHES: RefCell<Vec<Rc<LocaleCache>>> = const { RefCell::new(Vec::new()) };
   }

   /// The cache of `tag` on this thread; the oldest of `CACHED_LOCALES` is
   /// dropped when a new tag arrives.
   pub(crate) fn locale_cache(tag: &Arc<str>) -> Rc<LocaleCache> {
       CACHES.with(|caches| {
           let mut caches = caches.borrow_mut();
           if let Some(found) = caches.iter().find(|c| *c.tag == **tag) {
               return Rc::clone(found);
           }
           if caches.len() == CACHED_LOCALES {
               caches.remove(0);
           }
           let cache = Rc::new(LocaleCache::new(Arc::clone(tag)));
           caches.push(Rc::clone(&cache));
           cache
       })
   }

   #[cfg(test)]
   mod tests {
       //! Role: One locale's ICU objects are built once per thread.

       use std::rc::Rc;
       use std::sync::Arc;

       #[test]
       fn the_same_tag_reuses_one_cache() {
           let a = super::locale_cache(&Arc::from("ja-JP"));
           let b = super::locale_cache(&Arc::from("ja-JP"));
           assert!(Rc::ptr_eq(&a, &b));
       }
   }
   ```
3. `src/lib.rs`：既存の `mod` 行の並びに `mod cache;` を足す（アルファベット順の位置）。
4. `src/locale.rs`：
   - `pub struct Locale { tag: String, }` の field を `tag: std::sync::Arc<str>,` にする（derive の行 `#[derive(Clone, Debug, PartialEq, Eq)]` はそのまま。`Arc<str>` の比較は文字列の比較）。
   - `Locale::new` の本体を `Locale { tag: std::sync::Arc::from(tag.into()) }` にする。`tag()` の本体 `&self.tag` はそのまま。
   - `pub(crate) fn icu(&self) -> icu_locale_core::Locale { self.tag.replace('_', "-").parse()... }` を消し、次の二つを置く。
     ```rust
     /// This locale's ICU objects on this thread.
     pub(crate) fn cache(&self) -> std::rc::Rc<crate::cache::LocaleCache> {
         crate::cache::locale_cache(&self.tag)
     }

     /// The parsed ICU locale.
     pub(crate) fn icu(&self) -> icu_locale_core::Locale {
         self.cache().icu().clone()
     }
     ```
   - `first_day_of_week` の本体を次にする。
     ```rust
     let cache = self.cache();
     *cache.first_day.get_or_init(|| {
         let info = icu_calendar::week::WeekInformation::try_new(cache.icu().clone().into())
             .expect("invariant: compiled ICU week data includes locale fallback");
         Weekday::Monday.days_after((info.first_weekday as usize + 6) % 7)
     })
     ```
5. `src/numbers.rs`：
   - `struct NumberSymbols {` を `pub(crate) struct NumberSymbols {` にする（field は変えない）。
   - `fn decimal_formatter(&self, options: DecimalFormatterOptions) -> DecimalFormatter` を消す。`format_number` の `let mut options = DecimalFormatterOptions::default();` から `self.decimal_formatter(options).format(&decimal).to_string()` までを次にする。
     ```rust
     let cache = self.cache();
     let index = match style.grouping {
         Grouping::None => 0,
         Grouping::Locale => 1,
     };
     let formatter = cache.decimals[index].get_or_init(|| {
         let mut options = DecimalFormatterOptions::default();
         options.grouping_strategy = Some(match style.grouping {
             Grouping::None => GroupingStrategy::Never,
             Grouping::Locale => GroupingStrategy::Auto,
         });
         DecimalFormatter::try_new(cache.icu().clone().into(), options)
             .expect("invariant: compiled ICU decimal data includes locale fallback")
     });
     formatter.format(&decimal).to_string()
     ```
   - `parse_number` の本体（`let symbols = NumberSymbols::new(self); symbols.parse(text)`）を次にする。
     ```rust
     let cache = self.cache();
     cache.symbols.get_or_init(|| NumberSymbols::new(self)).parse(text)
     ```
   - `pub(crate) fn date_fields(locale: &Locale, text: &str, pattern: &str)` の `let symbols = NumberSymbols::new(locale);` を次の 2 行にする。以降の `symbols.` はそのまま。
     ```rust
     let cache = locale.cache();
     let symbols = cache.symbols.get_or_init(|| NumberSymbols::new(locale));
     ```
   - `NumberSymbols::new` の中の `locale.icu().into()` はそのまま（`icu()` が cache から clone する）。
6. `src/regional.rs`（各 method の `try_new` を cache に置き換える。引数と戻り値は変えない）：
   - `format_date`：`let formatter = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(self.icu().into(), fields).expect(..);` を次にする（直前の `let fields = match style { .. };` はそのまま）。
     ```rust
     let index = match style {
         DateStyle::Short => 0,
         DateStyle::Medium => 1,
         DateStyle::Long | DateStyle::Iso => 2,
     };
     let cache = self.cache();
     let formatter = cache.dates[index].get_or_init(|| {
         FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(cache.icu().clone().into(), fields)
             .expect("invariant: compiled ICU Gregorian date data includes locale fallback")
     });
     ```
   - `parse_date`：`let formatter = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(self.icu().into(), YMD::short(),).expect(..);` を次にする。
     ```rust
     let cache = self.cache();
     let formatter = cache.dates[0].get_or_init(|| {
         FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(cache.icu().clone().into(), YMD::short())
             .expect("invariant: compiled ICU Gregorian date data includes locale fallback")
     });
     ```
   - `format_time`：`let formatter = NoCalendarFormatter::try_new(self.icu().into(), T::medium().with_time_precision(precision),).expect(..);` を次にする。
     ```rust
     let cache = self.cache();
     let formatter = cache.times[usize::from(style.seconds)].get_or_init(|| {
         NoCalendarFormatter::try_new(
             cache.icu().clone().into(),
             T::medium().with_time_precision(precision),
         )
         .expect("invariant: compiled ICU time data includes locale fallback")
     });
     ```
   - `hour_cycle`：本体全体を `let cache = self.cache(); *cache.hour_cycle.get_or_init(|| { <今の本体> })` で包み、今の本体の `self.icu().into()` を `cache.icu().clone().into()` にする。
   - `month_name`：`if !(1..=12).contains(&month) { return String::new(); }` の後の残り全体を `self.cache().name(NameKey::Month(width, month), || { <今の残り> })` で包む（中の `self.icu()` はそのまま）。ファイルの先頭に `use crate::cache::NameKey;` を足す。
   - `weekday_name`：本体全体を `self.cache().name(NameKey::Weekday(width, day), || { <今の本体> })` で包む。
   - `plural_category`：`let rules = icu_plurals::PluralRules::try_new_cardinal(self.icu().into()).expect(..);` と `match rules.category_for(&operands) {` を次にする（`match` の腕はそのまま）。
     ```rust
     let cache = self.cache();
     let category = cache.plurals.get_or_init(|| {
         let rules = icu_plurals::PluralRules::try_new_cardinal(cache.icu().clone().into())
             .expect("invariant: compiled ICU plural rules include locale fallback");
         Box::new(move |operands: &icu_decimal::input::Decimal| rules.category_for(operands))
     });
     match category(&operands) {
     ```
   - `compare`：本体を次にする。
     ```rust
     let cache = self.cache();
     let compare = cache.collator.get_or_init(|| {
         let collator =
             icu_collator::Collator::try_new(cache.icu().clone().into(), Default::default())
                 .expect("invariant: compiled ICU collation data includes locale fallback");
         Box::new(move |a: &str, b: &str| collator.compare(a, b))
     });
     compare(a, b)
     ```
7. `cargo check` が `get_or_init` の closure の戻り値で E0308（`Box<closure>` と `Box<dyn Fn ..>` の不一致）を出したら、その closure の最後の式を `Box::new(..) as Box<dyn Fn(..) -> ..>` と `Deferred` の型引数どおりに cast する。これ以外の変更はしない。

### 消すもの

- `Locale::icu()` の毎回の `replace` + `parse`、`Locale::decimal_formatter`、各 method の毎回の `try_new`。

### 状態の閉じ方

- `LocaleCache`：`Deferred` は遅延初期化の道具で状態ではない（設計書 6.2 節 DEC-13）。内部可変の field は `names: RefCell<..>` の一つだけなので 6.1 節の検査の対象外。印は書かない。

### 試験

- 追加：`crates/kivact-i18n/src/cache.rs` の `the_same_tag_reuses_one_cache`。
- 既存の e2e の `i18n` module と kivact-i18n の doctest を変えずに通す。

### 確認（tier a）

```
cargo fmt -p kivact-i18n
cargo check -p kivact-i18n --all-targets
cargo clippy -p kivact-i18n --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-i18n
KIVACT_TEST_MIRROR=off cargo test -p kivact-i18n
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- i18n
```

### 完了条件

- `rg -n 'try_new' crates/kivact-i18n/src/regional.rs crates/kivact-i18n/src/numbers.rs crates/kivact-i18n/src/locale.rs` の各行が `get_or_init` の closure の中にある（`NumberSymbols::new` の中の `try_new_unstable` を除く）。
- 上の命令がすべて成功する。

## R02：selector の key node、timer の取消し、owner の破棄の割当て

- **wave / lane / 前提**：W2a / R / R01。
- **所有**：`crates/kivact-runtime/src/reactive/selector.rs`、`reactive/owner.rs`、`reactive/frame.rs`、`reactive/data.rs`、`reactive/runtime.rs`、`task/state.rs`、`task/time.rs`。
- **目的**：selector が追跡のない呼出しで node を作ることと、key の表を掃除（prune）で直すことをやめる（RC-24）。timer の取消しと poll を O(log N) にする（RC-25）。owner の破棄で解放する member を一つずつ box しない（RC-26）。

### 手順

1. `reactive/frame.rs`：`pub(crate) fn is_pure(&self) -> bool` の直後に次を足す。
   ```rust
   /// Whether a computation is collecting dependencies in the current frame.
   pub(crate) fn is_tracking(&self) -> bool {
       self.context().tracking.is_some()
   }
   ```
2. `reactive/selector.rs`：
   - `struct SelectorCache<K>` から field `prune: PruneSchedule,` を消す。`struct PruneSchedule`・その `impl`・`const SELECTOR_PRUNE_MIN` を消す。`create_selector` の `prune: PruneSchedule { at: SELECTOR_PRUNE_MIN, },` を消す。`use std::collections::{HashMap, HashSet};` を `use std::collections::HashMap;` にする。
   - `Selector::is` の本体を次にし、`fn prune` を消す。
     ```rust
     let at = Location::caller();
     let rt = runtime(self.app, at);
     let state = rt.object_or_panic::<SelectorState<K>>(self.key, "Selector", at);
     // The source was written earlier in this turn and the selector has
     // not caught up yet: bring it up to date first.
     if rt.needs_run(state.effect) {
         rt.run_effect(state.effect);
     }
     // Read outside a computation: the answer only, nothing to subscribe.
     if !rt.is_tracking() {
         return state.cache.borrow().current.as_ref() == Some(k);
     }
     // A reader outside a live owner (a disposing frame) reads the
     // selector's own owner: its node lives as long as the selector.
     let reader = rt
         .current_owner()
         .filter(|owner| rt.owner_live(*owner))
         .unwrap_or(state.owner);
     let existing = state.cache.borrow().nodes.get(k).and_then(|nodes| {
         nodes
             .iter()
             .rev()
             .find(|(owner, _)| *owner == reader)
             .map(|(_, node)| *node)
     });
     let node = match existing {
         Some(n) => n,
         None => {
             let n = rt.create_source(reader, (), at);
             state
                 .cache
                 .borrow_mut()
                 .nodes
                 .entry(k.clone())
                 .or_default()
                 .push((reader, n));
             // The reader's disposal forgets the pair (its node goes with it).
             let cache = Rc::downgrade(&state.cache);
             let key = k.clone();
             rt.add_cleanup(
                 reader,
                 Box::new(move || {
                     let Some(cache) = cache.upgrade() else { return };
                     let mut cache = cache.borrow_mut();
                     if let Some(nodes) = cache.nodes.get_mut(&key) {
                         nodes.retain(|(_, node)| *node != n);
                         if nodes.is_empty() {
                             cache.nodes.remove(&key);
                         }
                     }
                 }),
             );
             n
         }
     };
     rt.read_node(node, true, at);
     state.cache.borrow().current.as_ref() == Some(k)
     ```
   - module の doc の最後の文 `Unread key nodes are pruned as the key set grows.` を `A key node is forgotten when the owner that read it is disposed.` にする。
3. `reactive/selector.rs` の `mod tests` の `remounted_readers_do_not_accumulate_key_entries`：二か所の
   ```rust
   row.with_setup(|| {
       let _ = sel.is(&1);
   });
   ```
   をそれぞれ次にする（追跡のある読みにする）。`use crate::reactive::data::EffectKind;` を test module に足す。
   ```rust
   row.effect(EffectKind::Internal, move || { let _ = sel.is(&1); }, at);
   ```
   期待値（`assert_eq!(entries, 1, ..)`）は変えない。
4. `task/state.rs`：
   - `pub(crate) type TimerId = u64;` の下に `pub(crate) type TimerKey = (u64, TimerId);` を足す。
   - `pub(crate) struct CancelQueue(kivact_types::cell::LocalMailbox<Vec<TimerId>>);` を `pub(crate) struct CancelQueue(kivact_types::cell::LocalMailbox<Vec<TimerKey>>);` にする。`CancelQueue::push(&self, id: TimerId)` を `push(&self, key: TimerKey)`、本体の `pending.push(id);` を `pending.push(key);` にする。
   - `Scheduler::drain_cancel` の本体を次にする。
     ```rust
     while let Some(pending) = self.cancel.0.take() {
         for key in pending {
             self.timers.remove(&key);
         }
     }
     ```
5. `task/time.rs`：
   - `enum SleepState` の `Registered { id: super::state::TimerId },` を `Registered { key: super::state::TimerKey },` にする。
   - `Sleep::poll` の `Unregistered` の腕の `self.state = SleepState::Registered { id };` を `self.state = SleepState::Registered { key: (deadline, id) };` にする（`deadline` はその腕で `Unregistered` から取り出している値）。
   - `Registered` の腕の始め
     ```rust
     SleepState::Registered { id } => {
         let key = scheduler
             .timers
             .keys()
             .find(|(_, timer)| *timer == id)
             .copied();
         let Some(key) = key else {
     ```
     を次にする（その後の `self.state = SleepState::Fired; return Poll::Ready(()); }` と以降の `key.0`・`timers.remove(&key)`・`timers.get_mut(&key)` はそのまま）。
     ```rust
     SleepState::Registered { key } => {
         if !scheduler.timers.contains_key(&key) {
     ```
   - `impl Drop for Sleep` の本体を `if let SleepState::Registered { key } = self.state { self.cancel.push(key); }` にする。
6. `reactive/data.rs`：`pub(crate) struct Mount` の直前に次を足す。そして `Runtime` の field `later` の後に `pub(crate) graveyard: RefCell<Graveyard>,` を足す。
   ```rust
   /// What an owner's disposal released, dropped after every borrow ends
   /// (user `Drop` code may run). Kept on the runtime to reuse its capacity.
   #[derive(Default)]
   pub(crate) struct Graveyard {
       pub(crate) nodes: Vec<RNode>,
       pub(crate) data: Vec<Rc<dyn Any>>,
   }
   ```
   `reactive/runtime.rs` の `Runtime::with_id` の構造体式に `graveyard: RefCell::new(Graveyard::default()),` を足し、`use crate::reactive::data::{..}` に `Graveyard` を足す。
7. `reactive/owner.rs`：
   - `dispose_one` の `let mut graveyard: Vec<Box<dyn Any>> = Vec::new();` を `let mut graveyard = std::mem::take(&mut *self.graveyard.borrow_mut());` にする。
   - `dispose_one` の中の `drop(graveyard);`（三か所）をそれぞれ `self.bury(graveyard);` にする。`impl Runtime` に次を足す。
     ```rust
     /// Drop what a disposal released (no borrow is held: `Drop` code may
     /// dispose more), then keep the emptied lists for the next disposal.
     fn bury(&self, mut graveyard: Graveyard) {
         graveyard.nodes.clear();
         graveyard.data.clear();
         let mut slot = self.graveyard.borrow_mut();
         if slot.nodes.capacity() < graveyard.nodes.capacity() {
             *slot = graveyard;
         }
     }
     ```
   - `dispose_members` の引数 `graveyard: &mut Vec<Box<dyn Any>>,` を `graveyard: &mut Graveyard,` にする。本体の `graveyard.push(Box::new(node));` を `graveyard.nodes.push(node);`、`graveyard.push(Box::new(obj.data));` を `graveyard.data.push(obj.data);`、`graveyard.push(Box::new(entry.data));` を `graveyard.data.push(entry.data);` にする。
   - `dispose_members` の `let rank = |k: &Key| -> u8 { .. }; let mut nodes = nodes; nodes.sort_by_key(|k| rank(k)); for n in nodes {` を次にする。
     ```rust
     // Effects first (their cleanups may read memos and sources), then
     // memos, then sources. Ranks are read once; the sort is stable.
     let mut ranked: Vec<(u8, Key)> = {
         let g = self.graph.borrow();
         nodes
             .iter()
             .map(|k| {
                 let rank = match g.get(*k).map(|n| n.kind()) {
                     Some(Kind::Effect(EffectKind::User)) => 0,
                     Some(Kind::Effect(_)) => 1,
                     Some(Kind::Memo) => 2,
                     _ => 3,
                 };
                 (rank, *k)
             })
             .collect()
     };
     ranked.sort_by_key(|(rank, _)| *rank);
     for (_, n) in ranked {
     ```
   - 使われなくなった `use std::any::Any;` を消す（`cargo check` の unused の警告に従う）。`Graveyard` を `use crate::reactive::data::{..}` に足す。

### 消すもの

- `PruneSchedule`、`SELECTOR_PRUNE_MIN`、`Selector::prune`、`is` の中の `nodes.retain(..)`、`Sleep::poll` の `timers.keys().find(..)`、`dispose_members` の比較ごとの graph の borrow と `Box<dyn Any>`。

### 状態の閉じ方

- `SelectorCache<K>`：`current: Option<K>` と `nodes: HashMap` で、bool・Option・counter・Cell の field は一つだけ。検査の対象外。
- `Graveyard`：`Vec` 二つ。対象外。
- `Runtime`：field の追加だけ。T の W1b が置いた `Runtime` の印の文面は変えない。

### 試験

- 変更：`crates/kivact-runtime/src/reactive/selector.rs` の `remounted_readers_do_not_accumulate_key_entries`（手順 3。期待値は同じ）。
- 既存：e2e の `setup_functions::handle_constructors_create_under_the_setup_owner`（selector を effect の中で読む）、`task` module、`disposal` module。

### 確認（tier a）

```
cargo fmt -p kivact-runtime
cargo check -p kivact-runtime --all-targets
cargo clippy -p kivact-runtime --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-runtime
KIVACT_TEST_MIRROR=off cargo test -p kivact-runtime --lib
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- task disposal setup_functions
```

### 完了条件

- `rg -n 'PruneSchedule|SELECTOR_PRUNE_MIN|fn prune|keys\(\)\s*\.find|Vec<Box<dyn Any>>' crates/kivact-runtime/src` が何も返さない。
- 上の命令がすべて成功する。

## R03：再計算と stabilize の割当てをなくす

- **wave / lane / 前提**：W2a / R / R02。
- **所有**：`crates/kivact-runtime/src/reactive/{graph.rs,frame.rs,turn.rs,data.rs,runtime.rs,registry.rs,handles.rs}`。
- **目的**：memo の再計算ごとの `Vec` の collect、collector ごとの新しい `Vec`、`writer` を決めるための graph の borrow、stabilize ごとの `HashMap`、`take_*` の `BTreeSet` の clone、source の読みごとの `Rc<Runtime>` の clone をなくす（RC-07）。

### 手順

1. `reactive/data.rs`：
   - `pub(crate) struct EffectNode` に field `pub(crate) pass_runs: u8,` を `seq` の後に足す。`graph.rs` の `create_effect` と `Runtime::watch` の `EffectNode { .. cleanups: Vec::new(), }` に `pass_runs: 0,` を足す。
   - `Runtime` の field `scratch` の後に次の二つを足し、`runtime.rs` の `with_id` で両方 `RefCell::new(Vec::new())` で初期化する。
     ```rust
     /// Emptied dependency lists kept for the next collector.
     pub(crate) dep_pool: RefCell<Vec<Vec<Key>>>,
     /// The effects that ran in the current stabilize pass.
     pub(crate) pass_touched: RefCell<Vec<Key>>,
     ```
2. `reactive/frame.rs`：
   - `pub(crate) fn tracking_frame(&self, node: Key, owner: Key, pure: bool, effect: bool) -> FrameGuard<'_>` に引数 `writes_for_node: bool` を最後に足す。本体の `writer: if effect && !matches!(self.graph.borrow().get(node).map(|n| n.kind()), Some(..User..)) { Some(node) } else { self.context().writer },` を `writer: if writes_for_node { Some(node) } else { self.context().writer },` にする。
   - 同じ関数の `let slot = { let mut collectors = self.collectors.borrow_mut(); ...` の前に `let deps = self.dep_pool.borrow_mut().pop().unwrap_or_default();` を置き、`collectors.push(Collector { node, deps: Vec::new(), set: None });` を `collectors.push(Collector { node, deps, set: None });` にする。
   - `pub(crate) fn take_deps(&self) -> Vec<Key>` の後に次を足す。
     ```rust
     /// Return an emptied dependency list to the pool.
     pub(crate) fn recycle_deps(&self, mut deps: Vec<Key>) {
         deps.clear();
         self.dep_pool.borrow_mut().push(deps);
     }
     ```
3. `reactive/graph.rs` の `recompute`：
   - `let guard = self.tracking_frame(key, owner, pure, is_effect);` を次にする（User 以外の effect だけが自分の writer になる。今と同じ規則）。
     ```rust
     let writes_for_node = matches!(effect, Some(EffectKind::Render | EffectKind::Internal));
     let guard = self.tracking_frame(key, owner, pure, is_effect, writes_for_node);
     ```
   - `self.set_deps(key, deps);` を `self.set_deps(key, &deps); self.recycle_deps(deps);` にする。`fn set_deps(&self, node: Key, new: Vec<Key>)` を `fn set_deps(&self, node: Key, new: &[Key])` にし、本体の `n.deps().iter().zip(&new)` を `zip(new)`、`for src in new {` を `for &src in new {` にする。
   - `let subs = { let mut g = self.graph.borrow_mut(); ... NodeData::Memo(m) if changed => m.subs.iter().map(|(s, _)| *s).collect::<Vec<_>>(), _ => Vec::new(), } };` から、その後の `if is_effect && self.write_epoch.get() != epoch_before { self.recheck_after_own_writes(key, epoch_before); }` と `if !subs.is_empty() { .. }` までを次の一続きにする（effect には subscriber が無く、memo には recheck が無いので、順の入替えは結果を変えない）。
     ```rust
     let is_memo_change = {
         let mut g = self.graph.borrow_mut();
         let Some(n) = g.get_mut(key) else { return };
         if let Some(c) = n.computation_mut() {
             c.state = CompState::Clean;
         }
         changed && matches!(n.data, NodeData::Memo(_))
     };
     if is_effect && self.write_epoch.get() != epoch_before {
         self.recheck_after_own_writes(key, epoch_before);
     }
     if is_memo_change {
         self.mark_memo_subscribers(key);
     }
     ```
     そして `impl Runtime` に次を足す。
     ```rust
     /// Mark the subscribers of a memo whose value changed as dirty and queue
     /// the effects among them (no list is collected).
     fn mark_memo_subscribers(&self, key: Key) {
         let mut g = self.graph.borrow_mut();
         let mut sched = self.sched.borrow_mut();
         let count = g.get(key).map_or(0, |n| n.subs().len());
         for i in 0..count {
             let Some(s) = g.get(key).and_then(|n| n.subs().get(i)).map(|(s, _)| *s) else {
                 break;
             };
             let Some(n) = g.get_mut(s) else { continue };
             let queue = match &n.data {
                 NodeData::Effect(e) | NodeData::Watcher(e) => Some((e.kind, e.seq)),
                 _ => None,
             };
             let Some(c) = n.computation_mut() else { continue };
             // A running subscriber is pulling this memo right now and reads
             // the new value; do not re-mark it.
             if c.state.mark(Staleness::Dirty)
                 && let Some((kind, seq)) = queue
             {
                 sched.push(&mut c.state, kind, seq, s);
             }
         }
     }
     ```
4. `reactive/turn.rs`：
   - `Sched::take_render` と `Sched::take_user` を次にし、`fn unqueue` を消す（`pop_render` は `Self::unqueue(self, graph, key)` を使っているので、`pop_render` の本体を `let (_, key) = self.render.pop_first()?; Self::unmark(graph, key); Some(key)` にする）。
     ```rust
     pub(crate) fn take_render(&mut self, graph: &mut crate::reactive::arena::Arena<RNode>) -> Vec<Key> {
         let set = std::mem::take(&mut self.render);
         set.into_iter()
             .map(|(_, k)| {
                 Self::unmark(graph, k);
                 k
             })
             .collect()
     }

     pub(crate) fn take_user(&mut self, graph: &mut crate::reactive::arena::Arena<RNode>) -> Vec<Key> {
         let set = std::mem::take(&mut self.user);
         set.into_iter()
             .map(|(_, k)| {
                 Self::unmark(graph, k);
                 k
             })
             .collect()
     }

     /// An effect taken out of its queue set is stale again, not queued.
     fn unmark(graph: &mut crate::reactive::arena::Arena<RNode>, key: Key) {
         if let Some(NodeData::Effect(e) | NodeData::Watcher(e)) = graph.get_mut(key).map(|n| &mut n.data)
             && let CompState::Queued(stale) = e.run.state
         {
             e.run.state = CompState::Stale(stale);
         }
     }
     ```
   - `Runtime::stabilize` の `let mut own_runs: std::collections::HashMap<Key, usize> = std::collections::HashMap::new();` を消し、`let own = own_runs.entry(key).or_insert(0); *own += 1; if *own > CYCLE_LIMIT || runs > limit {` を次にする。
     ```rust
     let own = self.count_pass_run(key);
     if usize::from(own) > CYCLE_LIMIT || runs > limit {
     ```
     `loop` の後（`stabilize` の最後）に `self.reset_pass_runs();` を置く。`impl Runtime` に次を足す。
     ```rust
     /// Count one run of effect `key` in this stabilize pass; returns the count.
     fn count_pass_run(&self, key: Key) -> u8 {
         let mut g = self.graph.borrow_mut();
         let Some(NodeData::Effect(e) | NodeData::Watcher(e)) = g.get_mut(key).map(|n| &mut n.data) else {
             return 0;
         };
         if e.pass_runs == 0 {
             self.pass_touched.borrow_mut().push(key);
         }
         e.pass_runs = e.pass_runs.saturating_add(1);
         e.pass_runs
     }

     /// Forget the per-pass run counts of the effects that ran.
     fn reset_pass_runs(&self) {
         let mut touched = self.pass_touched.borrow_mut();
         let mut g = self.graph.borrow_mut();
         for key in touched.drain(..) {
             if let Some(NodeData::Effect(e) | NodeData::Watcher(e)) = g.get_mut(key).map(|n| &mut n.data) {
                 e.pass_runs = 0;
             }
         }
     }
     ```
     `CYCLE_LIMIT`（100）は `u8` に収まり、`saturating_add` で 255 を越えないので判定は今と同じ。
5. `reactive/registry.rs`：`pub(crate) fn try_runtime` の後に次を足す。
   ```rust
   /// Run `f` with the registered runtime of `app` without cloning its `Rc`.
   /// `f` must not register or unregister an App (the registry stays
   /// borrowed); only handle reads that run no user code use it.
   pub(crate) fn with_runtime<R>(
       app: AppId,
       at: &'static Location<'static>,
       f: impl FnOnce(&Runtime) -> R,
   ) -> R {
       check_not_foreign(app, at);
       REGISTRY.with(|r| {
           let reg = r.borrow();
           match reg.get((app.0 as usize).wrapping_sub(1)).and_then(Option::as_ref) {
               Some(rt) => f(rt),
               None => misuse_at(
                   ErrorKind::WrongApp,
                   "handle belongs to an App that no longer exists",
                   at,
               ),
           }
       })
   }
   ```
6. `reactive/handles.rs` の `fn resolve_signal`：本体の `runtime(app, at).read_or_panic(key, track, "Signal", at)` を `crate::reactive::registry::with_runtime(app, at, |rt| rt.read_or_panic(key, track, "Signal", at))` にする（source の読みは memo を計算しないので user code を実行しない）。`resolve_value` は変えない。

### 消すもの

- `stabilize` の `own_runs`、`Sched::unqueue`、`take_render`/`take_user` の `.clone()`、`recompute` の `collect::<Vec<_>>()`、`tracking_frame` の graph の borrow。

### 状態の閉じ方

- `EffectNode`：counter は `seq`（作成順の identity）と `pass_runs` の二つになる。`EffectNode` の定義の直前に次の 1 行を置く（T の W1b がこの型に別の印を置いていたら、その行をこの行で置き換える）。
  `// xtask-allow(state): seq is the creation-order identity and pass_runs the per-pass run count reset when each stabilize ends; run.state is the only phase`
- `Runtime`：field の追加だけ。印の文面は変えない。

### 試験

- 既存の runtime の unit 試験と e2e の `runtime` module（`memo_cutoff_stops_equal_values`、`diamond_is_glitch_free_and_runs_once`、`dynamic_dependencies_switch`）と `flows` module を変えずに通す。

### 確認（tier a）

```
cargo fmt -p kivact-runtime
cargo check -p kivact-runtime --all-targets
cargo clippy -p kivact-runtime --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-runtime
KIVACT_TEST_MIRROR=off cargo test -p kivact-runtime --lib
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- runtime flows
```

### 完了条件

- `rg -n 'own_runs|fn unqueue|\.clone\(\)\s*\.into_iter\(\)' crates/kivact-runtime/src/reactive/turn.rs crates/kivact-runtime/src/reactive/graph.rs` が何も返さない。
- 上の命令がすべて成功する。

## R04：Machine の純粋な遷移、blocking の pool 一本化、報告の種類、mount の版

- **wave / lane / 前提**：W2a / R / R03。
- **所有**：runtime `src/machine.rs`、`src/task/{blocking.rs,state.rs,time.rs}`、`src/reactive/{diag.rs,data.rs,mounts.rs,runtime.rs,reactive.rs}`、`src/lib.rs`、core `src/devtools/console.rs`（1 行）。
- **目的**：Machine の `transition` を純粋な section で実行し、発火した timer を表から消す（RC-17）。`blocking` は常に App の blocking pool を使い、App の外では位置付きの `WrongApp` にし、予備の thread の保持を一つにする（RC-38）。`Reported.handled` を `kind: ReportKind` にし、新しい報告だけを読む `Runtime::reported_since` を足す（RC-39、RC-09 の runtime の部分。付属書 C の R-REQ-9）。`Runtime.mount_revision` を消す（RC-41）。

### 手順

1. `src/machine.rs`：
   - `use crate::reactive::{IntoValue, Owned, Owner, Signal, Value, effect_on, require_effectful, untrack};` に `pure` を足す。`use kivact_types::cell::Generation;` と `use std::cell::Cell;`（`RefCell` と同じ行）を足す。
   - `struct MachineCell<M: Machine>` の `timers: RefCell<HashMap<Discriminant<M::Event>, Task>>,` を次の 2 行にする。
     ```rust
     timers: RefCell<HashMap<Discriminant<M::Event>, (Generation, Task)>>,
     next_timer: Cell<Generation>,
     ```
     `machine()` の構造体式に `next_timer: Cell::new(Generation::new()),` を足す。
   - `MachineHandle::send` の `let step = untrack(|| (cell.input)().transition(&cell.state.get_untracked(), current));` を次にする。
     ```rust
     let step = pure(|| untrack(|| (cell.input)().transition(&cell.state.get_untracked(), current)));
     ```
   - 同じ関数の timer の部分（`let key = discriminant(&event);` から `cell.timers.borrow_mut().insert(key, task);` の `}` まで）を次にする。
     ```rust
     let key = discriminant(&event);
     if let Some((_, task)) = cell.timers.borrow_mut().remove(&key) {
         task.cancel();
     }
     if let Some(delay) = delay {
         let mut generation = cell.next_timer.get();
         let id = generation.next();
         cell.next_timer.set(generation);
         let handle = *self;
         let fire: Box<dyn FnOnce()> = Box::new(move || {
             // A fired timer leaves the table unless a newer one replaced it.
             if let Some(cell) = handle.cell.try_get() {
                 let mut timers = cell.timers.borrow_mut();
                 if timers.get(&key).is_some_and(|(current, _)| *current == id) {
                     timers.remove(&key);
                 }
             }
             handle.send(event);
         });
         let task = spawn_timer(cell.owner, delay, fire, at);
         cell.timers.borrow_mut().insert(key, (id, task));
     }
     ```
2. `src/task/time.rs`：`fn timer_runtime(api: &'static str, at: &'static Location<'static>) -> Rc<Runtime>` を `pub(super) fn timer_runtime(..)` にする（本体は変えない）。
3. `src/task/blocking.rs`：
   - private の `struct Completion<R>` を `JobGuard<R>` に改名する：`perl -pi -e 's/\bCompletion\b/JobGuard/g' crates/kivact-runtime/src/task/blocking.rs`（このファイルは公開の `task::Completion` を import していない。import していたら STOP）。その doc comment の先頭を `/// Held by a running job (its guard):` にする。
   - `struct BlockingFuture<R>` の `pool: Option<Arc<Pool>>,` を `pool: Arc<Pool>,` にする。`poll` の `let payload = self.pool.as_ref().and_then(|pool| pool.take_panic(thread));` を `let payload = self.pool.take_panic(thread);` にする。
   - `fn start<R: Send + 'static>(f: impl FnOnce(CancelToken) -> R + Send + 'static) -> BlockingFuture<R>` を `fn start<R: Send + 'static>(f: impl FnOnce(CancelToken) -> R + Send + 'static, at: &'static Location<'static>) -> BlockingFuture<R>` にし、本体の `let runtime = crate::Runtime::current();` から `match pool { .. }` の終わりまでを次にする（`let cancel`、`let token`、`let slot`、`let s2` の 4 行はその前に残す）。
     ```rust
     let runtime = super::time::timer_runtime("blocking", at);
     let pool = super::TaskState::of(&runtime).blocking_pool();
     let outstanding = Arc::clone(&pool.outstanding);
     outstanding.fetch_add(1, Ordering::SeqCst);
     let job: Job = Box::new(move || {
         let guard = JobGuard {
             slot: s2,
             outstanding: Some(outstanding),
         };
         let r = f(token);
         lock(&guard.slot).outcome = Outcome::Done(r);
     });
     pool.submit(job);
     BlockingFuture { slot, cancel, pool }
     ```
     `JobGuard.outstanding: Option<Arc<AtomicUsize>>` は `Arc<AtomicUsize>` にし、`Drop for JobGuard` の `if let Some(o) = &self.outstanding { o.fetch_sub(1, Ordering::SeqCst); }` を `self.outstanding.fetch_sub(1, Ordering::SeqCst);` にする（上の `Some(outstanding)` も `outstanding` にする）。
   - `pub fn blocking<R, M, F>(f: F) -> impl Future<Output = R>` に `#[track_caller]` を付け、本体を `start(move |cancel| f.run(cancel), Location::caller())` にする。doc comment の最後に次の文を足す：`Outside a running App it raises a located \`WrongApp\`, as \`sleep\` does.`
   - `use std::panic::Location;` を足す。使われなくなった `kivact_types::process::DetachedThreads` の参照が消えたことを確かめる。
4. `src/task/state.rs`：
   - `TaskState` の `/// The blocking pool, created on first use.` と `fallback_threads: RefCell<Vec<std::thread::JoinHandle<()>>>,` を消し、`pub(super) pool: RefCell<PoolState>,` の直前に `/// The blocking pool, created on first use.` を置く。
   - `TaskState::new` の `fallback_threads: RefCell::new(Vec::new()),` を消す。`shutdown` の `worker_pool::wait_for_threads(self.fallback_threads.take());` を消す。`pub(crate) fn retain_fallback(..)` を消す。
5. `src/reactive/diag.rs`：
   - `pub struct Reported` の field `pub handled: bool,`（とその上の `xtask-allow` の行と doc）を次にする。
     ```rust
     /// Whether it is an error or a non-fatal diagnostic.
     pub kind: ReportKind,
     ```
     そして `Reported` の直前に次を足す。
     ```rust
     /// What a [`Reported`] entry is.
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub enum ReportKind {
         /// An error that reached the default sink (no `on_error` took it).
         Error,
         /// A non-fatal diagnostic (a write after dispose, a stale event, ...).
         Diagnostic,
     }
     ```
   - `Runtime::reported` の `handled: false,` を `kind: ReportKind::Error,`、`handled: true,` を `kind: ReportKind::Diagnostic,` にする。
   - `Runtime::reported` の後に次を足す（付属書 C の R-REQ-9。DevTools の console が新しいものだけを読む。O(新しいもの)：両方の ring を後ろから `seq` が `since` 以下になるまで読む）。
     ```rust
     /// Errors and diagnostics reported after sequence number `since`, in
     /// report order (bounded rings).
     pub fn reported_since(&self, since: u64) -> Vec<Reported> {
         let diag = self.diag.borrow();
         let mut reported = Vec::new();
         for (seq, error) in diag.errors.iter().rev().take_while(|(seq, _)| *seq > since) {
             reported.push(Reported { seq: *seq, kind: ReportKind::Error, error: error.clone() });
         }
         for (seq, error) in diag.diagnostics.iter().rev().take_while(|(seq, _)| *seq > since) {
             reported.push(Reported { seq: *seq, kind: ReportKind::Diagnostic, error: error.clone() });
         }
         reported.sort_by_key(|entry| entry.seq);
         reported
     }
     ```
   - `src/reactive.rs` の `pub use diag::Reported;` を `pub use diag::{ReportKind, Reported};` にし、`src/lib.rs` の `pub use reactive::{ .. Reported, .. }` に `ReportKind` を足す。
6. core `src/devtools/console.rs` の `fn error_entry` の `handled: entry.handled,` を次にする（protocol の `handled` は今と同じ値になる）。
   ```rust
   handled: entry.kind == kivact_runtime::ReportKind::Diagnostic,
   ```
7. `src/reactive/data.rs`：`Runtime` の `/// Last stamp remains meaningful after every acknowledged callback was removed.` と `pub(crate) mount_revision: Cell<kivact_types::Revision>,` を消す。`src/reactive/runtime.rs` の `with_id` の `mount_revision: Cell::new(kivact_types::Revision(0)),` を消す。
8. `src/reactive/mounts.rs` の `stamp_mounts` を次にする。
   ```rust
   pub(crate) fn stamp_mounts(&self, revision: Revision) {
       let mut awaiting = self.awaiting.borrow_mut();
       assert!(
           awaiting.back().is_none_or(|(last, _)| *last <= revision),
           "invariant: mount revisions only grow"
       );
       awaiting.extend(self.unstamped.borrow_mut().drain(..).map(|m| (revision, m)));
   }
   ```

### 消すもの

- `TaskState::fallback_threads`、`TaskState::retain_fallback`、`blocking::start` の pool の無い枝（thread の生成）、`Reported.handled`、`Runtime.mount_revision`。

### 状態の閉じ方

- `MachineCell<M>`：内部可変の field は `dispatch`（`RefCell<Dispatch>`）、`timers`（`RefCell<HashMap>`）、`next_timer`（`Cell<Generation>`）。`MachineCell` の定義の直前に次の 1 行を置く。
  `// xtask-allow(state): dispatch is the only phase (idle or running a queue); timers maps each pending timer variant to the generation and task that fire it, and next_timer only allocates those generations`
- `JobGuard<R>`：`slot` と `outstanding`（Option でなくなる）。対象外。
- `Reported`：`kind` は enum。対象外。

### 試験

- 既存：e2e の `machine` module、`task::a_blocking_panic_resumes_on_the_ui_thread`、`task::blocking_runs_on_the_pool_and_returns_on_the_ui_thread`、`devtools` の console の試験（`e2e_native`）。期待値は変えない。
- `rg -n 'blocking\(' dev/tests crates/*/tests crates/kivact-test/src` の結果がすべて App の中（`mount(..)`・`render(..)` の closure か、その中の task）にあることを確かめる。App の外の呼出しがあれば STOP。

### 確認（tier a）

```
cargo fmt -p kivact-runtime -p kivact-core
cargo check -p kivact-runtime --all-targets
cargo check -p kivact-core --all-targets
cargo clippy -p kivact-runtime -p kivact-core --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-runtime
cargo xtask check --no-write --crate kivact-core
KIVACT_TEST_MIRROR=off cargo test -p kivact-runtime --lib
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- machine task
```

### 完了条件

- `rg -n 'fallback_threads|retain_fallback|mount_revision|\.handled\b' crates/kivact-runtime/src crates/kivact-core/src/devtools/console.rs` が何も返さない。
- `rg -n 'struct Completion' crates/kivact-runtime/src/task/blocking.rs` が何も返さない。
- 上の命令がすべて成功する。

## R05：`Extensions` をやめ、runtime の状態を field に、core の状態を一つの struct にする

- **wave / lane / 前提**：W5a / 鎖（先頭）/ CP1、R04、R07、S の W1a（`Deferred`）。付属書 C の求め（core の `launch/runtime.rs` を W2〜W4 で共有しない）により、W5 の鎖の最初の unit にする。公開 API は変えない。
- **所有**：runtime `src/host.rs`、`src/lib.rs`、`src/reactive/{data.rs,ext.rs,runtime.rs,registry.rs,selector.rs,owner.rs,reactive.rs}`、`src/motion/**`、`src/task/**`、core `src/lib.rs`（`mod` 行）、新規 core `src/app_state.rs`、core `src/launch/{runtime.rs,inspect.rs,close.rs}`、`src/client/{state.rs,trace.rs,dispatch/env.rs}`、`src/platform/{app_handle.rs,close_request.rs,lifecycle.rs}`、`src/gpu_interop/import/registry.rs`、`src/view/{state.rs,foreign.rs,devtools_state.rs}`、`src/devtools/{perf.rs,motion.rs}`。
- **目的**：runtime が自分の `TaskState`・`MotionState` を TypeId の表に入れ、core が 10 個の拡張を同じ表に入れ、`ViewState::of` などが毎回線形の探索をする構造をやめる。runtime は `tasks`・`motion` を field に持ち、core の状態は一つの `AppState` を一度の downcast で読む。core が `MotionEnv` を入れ忘れると runtime の motion が panic する暗黙の契約もなくす（RC-04）。

### 手順

1. runtime `src/reactive/data.rs`：
   - `pub struct Extensions(..)`、その `impl Extensions`（`new`、`with`、`get`）を消す。
   - `Runtime` の `ext` field（`// xtask-allow(..)` の行、`#[expect(clippy::disallowed_types, ..)]` と `pub(crate) ext: std::cell::OnceCell<Extensions>,`）を消し、`later` の後に次を足す。
     ```rust
     /// Tasks, timers, the blocking pool and the exit drain of this App.
     pub(crate) tasks: crate::task::TaskState,
     /// Frame loops and animations of this App.
     pub(crate) motion: crate::motion::MotionState,
     /// What the host layer installs while `Runtime::new` runs: written once,
     /// before `Runtime::new` returns, and only read afterwards.
     pub(crate) app: kivact_types::cell::Deferred<AppSlot>,
     ```
   - `pub(crate) struct Mount` の前に次を足す。
     ```rust
     /// The host layer's part of the runtime (`AppParts`, stored).
     pub(crate) struct AppSlot {
         pub(crate) motion_env: crate::motion::MotionEnv,
         pub(crate) state: Box<dyn Any>,
     }
     ```
2. runtime `src/host.rs`：
   - `RuntimeConfig` の後に次を足す。
     ```rust
     /// What the host layer builds during `Runtime::new` (in the root's setup):
     /// its own per-App state, read back with [`Runtime::app_state`], and the
     /// environment values the motion system reads.
     pub struct AppParts {
         /// The host layer's per-App state (one value; it downcasts it).
         pub state: Box<dyn std::any::Any>,
         /// Reduced motion, drawability and the frame period.
         pub motion_env: MotionEnv,
     }
     ```
   - `Runtime::new` の `build: impl FnOnce(&Rc<Runtime>, Owner) -> Extensions,` を `build: impl FnOnce(&Rc<Runtime>, Owner) -> AppParts,` にし、本体を次にする。
     ```rust
     let tasks = TaskState::new(config.blocking_threads, waker);
     let rt = Runtime::register(host, config.start_ns, tasks);
     let registration = RegistrationGuard(rt.id());
     let root = rt.create_owner(None, config.at);
     assert_eq!(root, rt.root, "invariant: the root owns the first owner slot");
     let parts = {
         let _frame = rt.setup_frame(rt.root);
         build(&rt, rt.root())
     };
     rt.app.get_or_init(|| crate::reactive::AppSlot {
         motion_env: parts.motion_env,
         state: parts.state,
     });
     rt.state.set(TurnState::Quiet);
     std::mem::forget(registration);
     rt
     ```
     （`AppSlot` を `reactive` から `pub(crate) use` で見えるようにする：`src/reactive.rs` の `pub use data::{..}` の後に `pub(crate) use data::AppSlot;` を足す。）
   - `impl Runtime` に次の 4 つを足す。
     ```rust
     /// The host layer's per-App state given by `AppParts::state`.
     pub fn app_state(&self) -> &dyn std::any::Any {
         self.app
             .get()
             .expect("invariant: Runtime::new installs the app parts before it returns")
             .state
             .as_ref()
     }

     /// The motion environment given by `AppParts::motion_env`.
     pub(crate) fn motion_env(&self) -> &MotionEnv {
         &self
             .app
             .get()
             .expect("invariant: Runtime::new installs the app parts before it returns")
             .motion_env
     }

     /// The tasks, timers and blocking pool of this App.
     pub fn tasks(&self) -> &TaskState {
         &self.tasks
     }

     /// The frame loops and animations of this App.
     pub fn motion(&self) -> &MotionState {
         &self.motion
     }
     ```
   - `use crate::{.. Extensions, ..}` から `Extensions` を消し、`MotionEnv` を足す。
3. runtime `src/reactive/registry.rs`：`pub(crate) fn register(host: Rc<dyn Host>, start_ns: u64) -> Rc<Runtime>` を `pub(crate) fn register(host: Rc<dyn Host>, start_ns: u64, tasks: crate::task::TaskState) -> Rc<Runtime>` にし、本体の `Runtime::with_id(id, host, start_ns)` を `Runtime::with_id(id, host, start_ns, tasks)` にする。
4. runtime `src/reactive/runtime.rs`：`with_id(id: AppId, host: Rc<dyn Host>, start_ns: u64)` に引数 `tasks: crate::task::TaskState` を足し、構造体式の `ext: std::cell::OnceCell::new(),`（とその `#[expect]`）を次の 3 行にする。
   ```rust
   tasks,
   motion: crate::motion::MotionState::default(),
   app: kivact_types::cell::Deferred::new(),
   ```
5. runtime の試験の `Runtime::register(..)`：次の命令で第 3 引数を足す。
   ```
   git ls-files 'crates/kivact-runtime/*.rs' | xargs perl -pi -e 's/Runtime::register\(Rc::new\((\w+)\), 0\)/Runtime::register(Rc::new($1), 0, crate::TaskState::new(None, crate::ThreadWaker::new(|| {})))/g'
   ```
   `cargo check -p kivact-runtime --all-targets` で、他の形の `Runtime::register(` が残っていたら同じ第 3 引数を手で足す。
6. runtime `src/reactive/ext.rs` を削除し、`src/reactive.rs` の `mod ext;` と、doc の「`ext` (per-App extensions),」の部分を消す。`src/reactive.rs` と `src/lib.rs` の `pub use` から `Extensions` を消し、`src/lib.rs` の `pub use host::RuntimeConfig;` を `pub use host::{AppParts, RuntimeConfig};` にする。
7. `TaskState::of` と `MotionState::of` を消し、呼出しをすべて `tasks()`・`motion()` にする。
   - `src/task/state.rs` の `pub fn of(rt: &Runtime) -> &TaskState { rt.ext::<TaskState>() }`（`#[inline]` を含む）を消す。`src/motion/state.rs` の `pub fn of(rt: &Runtime) -> &MotionState { rt.ext::<MotionState>() }` を消す。
   - 呼出しを置き換える（runtime と core の全ファイル）：
     ```
     git ls-files 'crates/kivact-runtime/*.rs' 'crates/kivact-core/*.rs' | xargs perl -pi -e 's/(?:\b[\w]+::)*\bTaskState::of\(&?((?:[^()]|\((?:[^()]|\([^()]*\))*\))*)\)/$1.tasks()/g; s/(?:\b[\w]+::)*\bMotionState::of\(&?((?:[^()]|\((?:[^()]|\([^()]*\))*\))*)\)/$1.motion()/g'
     ```
     `super::TaskState::of(&rt)` → `rt.tasks()`、`kivact_runtime::MotionState::of(self.rt())` → `self.rt().motion()` になる。
   - `src/motion/state.rs` の三か所の `let v = rt.ext::<MotionEnv>().reduced_motion;`・`.surface_active;`・`.frame_period_ns;` を `let v = rt.motion_env().reduced_motion;` などにする。
   - `cargo check` で `unused import: TaskState`／`MotionState` が出たら、その import を消す。
8. 新規 core `src/app_state.rs`：
   ```rust
   //! Role: The core's per-App state: one struct in the runtime's app slot.
   //!
   //! `Launch::start` builds it in the root's setup and gives it to
   //! `Runtime::new` (`AppParts::state`); every subsystem reads its part
   //! with one downcast (`AppState::of`).

   use kivact_runtime::Runtime;

   use crate::client::state::HostState;
   use crate::view::state::ViewState;

   pub(crate) struct AppState {
       pub(crate) view: ViewState,
       pub(crate) foreign: crate::view::foreign::ForeignWindows,
       pub(crate) handle: crate::platform::app_handle::AppHandleExt,
       pub(crate) gpu: crate::gpu_interop::State,
       #[cfg(feature = "devtools")]
       pub(crate) devtools: crate::view::devtools_state::DevtoolsState,
       #[cfg(debug_assertions)]
       pub(crate) trace: crate::client::trace::Trace,
       pub(crate) lifecycle: crate::platform::lifecycle::LifecycleState,
       pub(crate) close: crate::platform::close_request::CloseRequestState,
       pub(crate) host: HostState,
   }

   impl AppState {
       /// The core's state of `rt`'s App.
       #[inline]
       pub(crate) fn of(rt: &Runtime) -> &AppState {
           rt.app_state()
               .downcast_ref::<AppState>()
               .expect("invariant: Launch::start gives Runtime::new an AppState")
       }
   }
   ```
   core `src/lib.rs` の `mod` 行の並びに `mod app_state;` を足す。上の型のうち `pub(crate)` でないもの（`ForeignWindows`、`AppHandleExt`、`gpu_interop::State`、`DevtoolsState`、`Trace`、`LifecycleState`、`CloseRequestState`、`HostState`、`ViewState`）があれば `pub(crate)` にする（中身は変えない）。
9. core の `ext` の読みを置き換える（各ファイルの関数の本体を次の 1 行にする）：
   - `client/state.rs` の `HostState::of`：`&crate::app_state::AppState::of(rt).host`
   - `client/trace.rs` の `Trace::of`：`&crate::app_state::AppState::of(rt).trace`
   - `platform/app_handle.rs` の `AppHandleExt::of`：`&crate::app_state::AppState::of(rt).handle`
   - `gpu_interop/import/registry.rs` の `State::of`：`&crate::app_state::AppState::of(rt).gpu`
   - `view/devtools_state.rs` の `DevtoolsState::of`：`&crate::app_state::AppState::of(rt).devtools`
   - `view/state.rs` の `ViewState::of`：`&crate::app_state::AppState::of(rt).view`
   - `view/foreign.rs` の二か所の `let windows = rt.ext::<ForeignWindows>();`：`let windows = &crate::app_state::AppState::of(rt).foreign;`
   - `platform/close_request.rs` の二か所の `.ext::<CloseRequestState>()`：その式を `crate::app_state::AppState::of(<同じ runtime の式>).close` にする（`rt.ext::<CloseRequestState>()` は `crate::app_state::AppState::of(rt).close`、`owner.runtime().ext::<CloseRequestState>()` の形なら `crate::app_state::AppState::of(&owner.runtime()).close`）。
   - `platform/lifecycle.rs` の `let state = rt.ext::<LifecycleState>();` を `let state = &crate::app_state::AppState::of(&rt).lifecycle;`（`rt` が `&Runtime` なら `&` を付けない）、`rt.ext::<LifecycleState>().registrations` を `crate::app_state::AppState::of(&rt).lifecycle.registrations` にする。
   - `rg -n 'ext::<' crates/kivact-core/src` が theme の `ext::<`（`kivact-style` の `Theme::ext`）以外を返さないことを確かめる。
10. core `src/launch/runtime.rs` の `Launch::start`：
    - `use kivact_runtime::{.. Extensions, ..}` の `Extensions` を `AppParts` にする。
    - `Runtime::new(..)` の build closure（`|_rt, root| { let view = ...; ... .with(HostState::default()) }`）を次にする。
      ```rust
      |_rt, root| {
          let view = ViewState::new(root, info, writer.clone(), at);
          let motion_env = install_main_window(&view, root, info, options, at);
          let state = crate::app_state::AppState {
              view,
              foreign: crate::view::foreign::ForeignWindows::default(),
              handle: AppHandleExt(handle.clone()),
              gpu: crate::gpu_interop::State::new(root, at),
              #[cfg(feature = "devtools")]
              devtools: crate::view::devtools_state::DevtoolsState::from_slot(
                  root,
                  slot.take().expect("invariant: the build runs once"),
                  at,
              ),
              #[cfg(debug_assertions)]
              trace: crate::client::trace::Trace::new(trace_name.as_deref()),
              lifecycle: crate::platform::lifecycle::LifecycleState::default(),
              close: crate::platform::close_request::CloseRequestState::default(),
              host: HostState::default(),
          };
          AppParts {
              state: Box::new(state),
              motion_env,
          }
      },
      ```
11. `dev/xtask/policy/workspace.toml` は変えない（`app_state` は `module_rules.kivact-core.uses` の key に無いので、どの module からも参照できる。4 章 4.3 で付属書 T に伝える）。

### 消すもの

- runtime：`Extensions`、`Runtime::ext`、`missing_extension`、`src/reactive/ext.rs`、`Runtime.ext`、`TaskState::of`、`MotionState::of`。
- core：`Extensions::new().with(..)` の連鎖。

### 状態の閉じ方

- `Runtime.app: Deferred<AppSlot>`：`Runtime::new` の中で一度だけ書き、`Runtime::new` が返った後は読むだけ。`Runtime` の印（T の W1b が置いた 1 行）の後ろに次の文を足す（同じ 1 行の中に `; ` で続ける）。
  `app is written once inside Runtime::new before it returns and only read afterwards`
  付属書 S・設計書 6.2 節は `Deferred` を「未完成の object を後で埋める」用途に使わないと定める。この field の使い方は親が決定済み（4 章 4.4 の 1）：`Runtime::new` の中で一度だけ書く。
- `AppSlot`、`AppParts`：`Box` と `MotionEnv`（Copy）。対象外。
- `AppState`：各 field は subsystem の状態で、互いに独立。field 自身の型がそれぞれの印を持つ。`AppState` に内部可変の field は無い。対象外。

### 試験

- 新しい試験は無い。e2e の `e2e_core` の `launch_window`・`motion`・`task`・`close_request`・`windows`・`app_handle` module と、`kivact-test` の自己試験を変えずに通す。

### 確認（tier a）

```
cargo fmt -p kivact-runtime -p kivact-core
cargo check -p kivact-runtime --all-targets
cargo check -p kivact-core --all-targets
cargo check -p kivact-core --all-targets --no-default-features
cargo clippy -p kivact-runtime -p kivact-core --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-runtime
cargo xtask check --no-write --crate kivact-core
KIVACT_TEST_MIRROR=off cargo test -p kivact-runtime --lib
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- launch_window motion task close_request windows app_handle
```

### 完了条件

- `rg -n '\bExtensions\b|\.ext::<|TaskState::of|MotionState::of|missing_extension' crates/kivact-runtime/src crates/kivact-core/src` が、kivact-style の `Theme` の `ext::<`（core の中の `theme.ext::<..>()` などの theme の呼出し）以外を返さない。
- 上の命令がすべて成功する。

## R06：turn を一本の順にし、仕事の有無を一つの述語にする

- **wave / lane / 前提**：W5a / 鎖 / R05。公開 API は変えない（core の `launch/runtime.rs` を持つので鎖に置く）。
- **所有**：runtime `src/host.rs`、`src/lib.rs`、`src/reactive/{host.rs,host_types.rs,clock.rs,turn.rs,data.rs}`、`src/reactive.rs`、core `src/launch.rs`、`src/launch/{runtime.rs,inspect.rs}`、新規 core `src/launch/turn.rs`、`src/view/{requests.rs,state.rs,node_ref.rs}`、`src/client/dispatch.rs`、`src/client/dispatch/pointer.rs`、`src/app_state.rs`。
- **目的**：「待っている仕事があるか」の四つの定義を一つの `Runtime::has_work` にし、core だけが知る activation の列を runtime の deferred の列にする（RC-02）。idle の近道が `WakeSent` のまま残る不具合を直す（RC-01、RC-41 の `report_now`）。settle の繰返しと入れ子の上限を一つの `TurnGuard::settle` にし、turn の順を `launch/turn.rs` の一つの関数にする（RC-03）。

### 手順

1. runtime `src/reactive/data.rs`：
   - `pub struct TurnGuard<'a>` に field `pub(crate) async_rounds: std::cell::Cell<u32>,` を足す。
   - `Runtime` の `later` の後に次を足す（`runtime.rs` の `with_id` で `RefCell::new(VecDeque::new())` で初期化）。
     ```rust
     /// Handlers deferred until the running one returns (`Runtime::defer`),
     /// each with the owner it runs for.
     pub(crate) deferred: RefCell<VecDeque<(Key, Box<dyn FnOnce()>)>>,
     ```
2. runtime `src/reactive/host_types.rs`：`pub struct Pending { .. }` を（4 つの field と印を含めて）削除する。
3. runtime `src/reactive/host.rs`：
   - `pub fn pending(&self) -> Pending` を削除し、次を足す。
     ```rust
     /// Whether work waits that a turn would run: queued render or user
     /// effects, deferred handlers, deferred error reports, or ready tasks.
     /// Mount callbacks are asked separately (`mounts_due`), with the
     /// revision the host acknowledged.
     pub fn has_work(&self) -> bool {
         self.has_render_effects()
             || self.has_user_effects()
             || !self.deferred.borrow().is_empty()
             || self.has_later_reports()
             || self.tasks.has_ready()
     }

     /// Whether a turn was requested of the host and has not started.
     pub fn turn_requested(&self) -> bool {
         self.state.get() == TurnState::WakeSent
     }

     /// Run `f` for `owner` once the running handler has returned (in the
     /// same turn): a handler frame of `owner`, skipped when the owner is
     /// gone. `NodeRef::activate` uses it.
     pub fn defer(&self, owner: Owner, f: Box<dyn FnOnce()>) {
         self.deferred.borrow_mut().push_back((owner.key, f));
         self.request_turn();
     }
     ```
   - `fn begin(&self, stage: Stage) -> TurnGuard<'_>` の構造体式に `async_rounds: std::cell::Cell::new(0),` を足す。
   - `TurnGuard::stabilize` はそのまま。`impl TurnGuard` に次を足す。
     ```rust
     /// Run the deferred handlers queued so far (those they defer wait for
     /// the next call). Returns whether any ran.
     pub fn run_deferred(&self) -> bool {
         let batch = std::mem::take(&mut *self.rt.deferred.borrow_mut());
         let ran = !batch.is_empty();
         for (owner, f) in batch {
             if self.rt.owner_alive(owner) {
                 let _frame = self.rt.handler_frame(Some(owner));
                 f();
             }
         }
         ran
     }
     ```
   - `Runtime::stabilize`（`turn.rs`）と `Runtime::run_user_effects`（`turn.rs`）を `pub(crate)` にする。`rg -n '\.stabilize\(\)|\.run_user_effects\(\)' crates --glob '!crates/kivact-runtime/**'` が core の `launch/runtime.rs` 以外を返したら STOP。
4. runtime `src/host.rs`：
   - `pub fn report_now(&self) -> TurnReport` を `fn exit_report(&self) -> TurnReport` に改名し（private）、本体の最初の 3 行（`if matches!(self.state.get(), TurnState::Running(_)) { self.state.set(TurnState::Running(Stage::Output)); }`）を消す。
   - `impl Runtime` に次を足す。
     ```rust
     /// End a host call that ran no turn (nothing to do): the App is quiet
     /// again and asks for a turn if work is queued. Every early return of
     /// the host's turn goes through it.
     pub fn end_idle(&self) -> TurnReport {
         self.state.set(TurnState::Quiet);
         let report = self.exit_report();
         if self.has_work() {
             self.request_turn();
         }
         report
     }
     ```
   - ファイルの先頭の `use` の後（module の段）に次の定数を置く。
     ```rust
     /// The tasks and timers rounds one turn may run (shared by all its settles).
     const ASYNC_ROUNDS: u32 = 1000;
     ```
     `TurnGuard::settle_async` を削除し、`impl TurnGuard<'_>` に次の二つを足す。
     ```rust
     /// One round of due timers and ready tasks. Returns whether any ran.
     fn poll_async_round(&self) -> bool {
         let rt = self.rt;
         if self.async_rounds.get() >= ASYNC_ROUNDS {
             return false;
         }
         self.stage(Stage::Async);
         let wakers = rt.tasks.take_due_timers(rt.now_ns());
         let woke = !wakers.is_empty();
         if woke {
             self.cause_with(CauseTag::Timer, "timer".into());
         }
         for w in wakers {
             w.wake();
         }
         let polled = rt.tasks.poll_tasks(rt);
         if polled > 0 {
             self.cause_with(CauseTag::Task, "task".into());
         }
         let ran = woke || polled > 0;
         if ran {
             let rounds = self.async_rounds.get() + 1;
             self.async_rounds.set(rounds);
             if rounds == ASYNC_ROUNDS {
                 rt.report(None, UiError::new(ErrorKind::Cycle { path: Vec::new() },
                     "tasks and timers did not settle within 1000 rounds of one turn; the rest runs in the next turn", None));
             }
         }
         ran
     }

     /// The only settle: effects, then deferred handlers, then one round of
     /// timers and tasks, until none of them has work.
     pub fn settle(&self) {
         loop {
             self.stabilize();
             let deferred = self.run_deferred();
             let polled = self.poll_async_round();
             if !deferred && !polled {
                 break;
             }
         }
     }
     ```
   - `TurnGuard::finish(self, ops: usize, client_pending: bool) -> TurnReport` を `finish(self, ops: usize) -> TurnReport` にし、本体を次にする。
     ```rust
     self.stage(Stage::Output);
     let report = self.rt.exit_report();
     self.rt.end_turn(ops, self.started.elapsed().as_nanos() as u64);
     self.rt.state.set(TurnState::Quiet);
     if self.rt.has_work() {
         self.rt.request_turn();
     }
     report
     ```
   - `TurnGuard::finish_setup` の `let pending = self.rt.pending(); if pending.user || pending.render || TaskState::of(..).has_ready() { .. }` を `if self.rt.has_work() { self.rt.request_turn(); }` にする。
   - `TurnGuard::check_draining` はそのまま残す（R22 が消す）。
5. runtime `src/reactive.rs`・`src/lib.rs`：`pub use host_types::{ExitState, Host, Pending, Stage, ThreadWaker, TurnReport};` などから `Pending` を消す。
6. core `src/client/dispatch/pointer.rs`：`pub(crate) fn run_requested_activations(rt: &Runtime) -> bool` を次の関数に置き換える。`src/client/dispatch.rs` の `pub(crate) use pointer::run_requested_activations;` を `pub(crate) use pointer::run_activation;` にする。
   ```rust
   /// Run the default action of `target` that `NodeRef::activate` asked for,
   /// as a click would, if it is still mounted and admitted by the open modal.
   pub(crate) fn run_activation(rt: &Runtime, target: kivact_types::FocusTarget) {
       let in_scope = ViewState::tree(rt).reader().in_modal_scope(target.node);
       if ViewState::tree(rt).get(target.node).is_none() || !in_scope {
           return;
       }
       activate(rt, target.node, target.region);
   }
   ```
7. core `src/app_state.rs`：`AppState` に field `pub(crate) activate: fn(&Runtime, kivact_types::FocusTarget),` を `host` の後に足す。`launch/runtime.rs` の build closure の `AppState { .. }` に `activate: crate::client::dispatch::run_activation,` を足す（view は client を参照できないので、launch が関数を渡す）。
8. core `src/view/node_ref.rs` の `NodeRef::activate`：`super::requests::request_activation(&rt, node);` を次にする。
   ```rust
   let app = self.app;
   rt.defer(
       rt.root(),
       Box::new(move || {
           if let Some(rt) = Runtime::get(app) {
               (crate::app_state::AppState::of(&rt).activate)(&rt, node);
           }
       }),
   );
   ```
9. core `src/view/requests.rs`：`request_activation`、`take_activations`、`has_activations` を削除する。`src/view/state.rs` の `pub(crate) pending_activations: Vec<kivact_types::FocusTarget>,` と、その初期化、`requests.pending_activations.shrink_to_fit();` を削除する。
10. 新規 core `src/launch/turn.rs` と `src/launch/runtime.rs` の変更：
    - `src/launch.rs` に `mod turn;` を足す。
    - `launch/runtime.rs` の `fn turn(&mut self, ..)`（doc comment を含む）、`fn settle_user_effects`、`fn phase_name`、`fn describe` を `launch/runtime.rs` から削除し、`fn output` と `fn commit` を `pub(super) fn output`・`pub(super) fn commit` にする。`const USER_EFFECT_ROUNDS` も `turn.rs` へ移す。
    - `launch/turn.rs` を次の内容で作る。`describe` と `phase_name` は `launch/runtime.rs` にあった本体をそのまま写す（下では本体を省いて `/* 写す */` と書いた）。
      ```rust
      //! Role: The order of one App turn.
      //!
      //! One turn: due mount callbacks and deferred reports, then each host
      //! event (dispatch, then settle), then commit rounds (commit, mounts,
      //! deferred reports, user effects, settle) until no work is left. The
      //! only settle is `TurnGuard::settle` (effects, deferred handlers,
      //! timers and tasks under one budget per turn).

      use kivact_engine::api::{Boundary, TurnOutput, Writer};
      use kivact_engine::contract::HostEvent;
      use kivact_runtime::{CauseTag, ErrorKind, Runtime, Stage, TurnGuard, UiError};
      use kivact_types::Revision;

      use super::runtime::{ClientMode, Launch, commit, output};
      use crate::client::state::HostState;
      use crate::view::state::ViewState;
      use crate::view::text_edits::TextEditsAhead;

      /// Commit rounds one turn may run before user effects count as a cycle.
      const USER_EFFECT_ROUNDS: usize = 100;

      impl Launch {
          /// One turn of `events` (see the module documentation). The
          /// boundaries among the events only decide when the `on_mount`
          /// callbacks of their revisions run.
          pub(super) fn turn(
              &mut self,
              events: &[HostEvent],
              boundaries: &[Boundary],
              now_ns: u64,
          ) -> TurnOutput {
              let rt = self.runtime().clone();
              let host = HostState::of(&rt);
              rt.advance_clock(now_ns);
              // A close requested from any thread since the last turn starts the
              // exit before the idle check, so a bare wake always counts.
              let embedded = matches!(self.mode, ClientMode::Embedded(_));
              super::close::take_close_request(&rt, embedded);
              #[cfg(feature = "devtools")]
              let incoming = crate::devtools::take_incoming(&rt);
              let tree = ViewState::tree(&rt).engine().clone();
              // A host call with no event, no due mount callback and no queued
              // work runs no turn.
              let idle = events.is_empty()
                  && boundaries.last().is_some_and(|b| !rt.mounts_due(b.revision))
                  && !rt.has_work();
              #[cfg(feature = "devtools")]
              let idle = idle && incoming.is_empty();
              if idle {
                  return output(rt.end_idle());
              }
              // The revisions done before the events: all but those whose
              // boundaries this turn delivers.
              let before = boundaries
                  .first()
                  .map_or(tree.stats().revision, |b| b.revision.0.saturating_sub(1));
              let mut stats = host.stats.get();
              stats.turns += 1;
              host.stats.set(stats);
              let turn = rt.begin_turn(events.len());
              if events.is_empty() && boundaries.last().is_some_and(|b| rt.mounts_due(b.revision)) {
                  turn.cause_with(CauseTag::Mount, "mounts".into());
              }
              ViewState::of(&rt).initial_render.set(false);
              turn.run_mounts(Revision(before));
              turn.deliver_reports();
              turn.settle();
              let edits = TextEditsAhead::begin(&mut ViewState::tree_mut(&rt), events);
              let mut boundaries = boundaries.iter().peekable();
              let mut acknowledge = |delivered: usize| {
                  while let Some(b) = boundaries.next_if(|b| b.events <= delivered) {
                      turn.run_mounts(b.revision);
                      turn.settle();
                  }
              };
              acknowledge(0);
              #[cfg(feature = "devtools")]
              crate::devtools::run_incoming(&rt, &turn, incoming);
              let mut delivered = 0;
              for event in events {
                  if matches!(&self.mode, ClientMode::Embedded(closing) if closing.is_set()) {
                      break;
                  }
                  turn.stage(Stage::Dispatch);
                  #[cfg(debug_assertions)]
                  crate::client::trace::event(&rt, format_args!("host_events receive {event:?}"));
                  edits.deliver(event);
                  turn.cause_with(CauseTag::Input, describe(event));
                  crate::client::dispatch::dispatch(&rt, event);
                  turn.settle();
                  delivered += 1;
                  acknowledge(delivered);
              }
              acknowledge(events.len());
              #[cfg(feature = "devtools")]
              crate::devtools::run_events(&rt);
              let done = Revision(tree.stats().revision);
              turn.check_draining();
              turn.settle();
              let writes = commit_rounds(&turn, &rt, &tree, done);
              let report = turn.finish(writes);
              #[cfg(feature = "devtools")]
              crate::devtools::after_turn(&rt);
              let output = output(report);
              #[cfg(debug_assertions)]
              {
                  crate::client::trace::record(&rt);
                  crate::client::trace::event(&rt, format_args!("host_events output {output:?}"));
              }
              output
          }
      }

      /// Commit, then run what the commit released (mount callbacks, deferred
      /// reports, user effects) until no work is left. Returns the writes.
      fn commit_rounds(turn: &TurnGuard<'_>, rt: &Runtime, tree: &Writer, done: Revision) -> usize {
          let mut writes = 0;
          let mut round = 0;
          loop {
              writes += commit(turn, rt, tree);
              let mounts = rt.mounts_due(done);
              // The commit may queue work too (a failed focus request reaching
              // an error boundary): it runs and commits in this turn.
              if !mounts && !rt.has_work() {
                  break;
              }
              round += 1;
              if round > USER_EFFECT_ROUNDS {
                  let abandoned = turn.abandon_user_effects();
                  let path: Vec<_> = abandoned
                      .iter()
                      .filter_map(|k| rt.node_info(*k).map(|n| n.at))
                      .collect();
                  rt.report(
                      None,
                      UiError::new(
                          ErrorKind::Cycle { path: path.clone() },
                          "user effects did not settle within 100 rounds",
                          path.first().copied(),
                      ),
                  );
                  break;
              }
              if mounts {
                  turn.run_mounts(done);
              }
              turn.deliver_reports();
              turn.run_user_effects();
              turn.settle();
          }
          writes
      }

      fn phase_name(phase: kivact_types::PointerPhase) -> &'static str {
          /* 写す */
      }

      /// Short cause label of an event for the structured trace (no values).
      fn describe(e: &HostEvent) -> String {
          /* 写す */
      }
      ```
      `launch/runtime.rs` の `ClientMode`、`Launch`、`output`、`commit` が `turn.rs` から見えること（`pub(super)` 以上）を確かめる。`Launch` の field `launch`・`mode` は `pub(super)` のまま。`kivact_engine::api::TurnOutput` などの import の path は `launch/runtime.rs` の今の import に合わせる。
    - `launch/runtime.rs` で使われなくなった import（`TextEditsAhead`、`requests` の一部、`CauseTag` など）を `cargo check` の警告に従って消す。
11. core `src/launch/inspect.rs` の `Inspect::pending` の本体を次にする。
    ```rust
    let rt = self.rt();
    rt.turn_requested() || rt.has_work()
    ```
    `use kivact_runtime::{Runtime, TaskState, UiError};` の `TaskState` が使われなくなったら消す。
12. `rg -n 'report_now|settle_async|client_pending|run_requested_activations|take_activations|has_activations|pending_activations|\.pending\(\)' crates/kivact-runtime/src crates/kivact-core/src` が何も返さないことを確かめる。`kivact-test` が `report_now` を呼んでいたら STOP（`rg -n report_now crates/kivact-test`）。

### 消すもの

- runtime：`Pending`、`Runtime::pending`、`Runtime::report_now`（`exit_report` に改名して private）、`TurnGuard::settle_async`、`finish` の `client_pending`。
- core：`run_requested_activations`、`requests::{request_activation, take_activations, has_activations}`、`ViewState.requests.pending_activations`、`settle_user_effects`、`Launch::turn` の `rt.stabilize()` の直接呼出し。

### 状態の閉じ方

- `TurnGuard<'a>`：`rt`、`started`、`async_rounds: Cell<u32>`。counter は一つ。`Guard` の名の型で 1 turn の間だけ生きる。対象外。
- `Runtime.deferred`：列。`Runtime` の印の文面は変えない。
- `AppState.activate`：fn pointer（constructor でだけ書く）。対象外。

### 試験

- 既存（変えずに通す）：e2e の `node_ref` module（`NodeRef::activate`）、`focus` module、`lifecycle`、`task`、`async_tests`、`runtime`、`flows`、`app_handle`、kivact-test の自己試験。
- 追加しない（RC-01 は過去の不具合の再現だけの試験になるため、設計書 6.4 節に従い足さない）。

### 確認（tier a）

```
cargo fmt -p kivact-runtime -p kivact-core
cargo check -p kivact-runtime --all-targets
cargo check -p kivact-core --all-targets
cargo check -p kivact-test --all-targets
cargo clippy -p kivact-runtime -p kivact-core --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-runtime
cargo xtask check --no-write --crate kivact-core
KIVACT_TEST_MIRROR=off cargo test -p kivact-runtime --lib
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- node_ref focus lifecycle task async_tests runtime flows app_handle
```

### 完了条件

- 手順 12 の `rg` が何も返さない。
- `crates/kivact-core/src/launch/turn.rs` があり、`fn turn` がそこにだけある（`rg -n 'fn turn\(' crates/kivact-core/src`）。
- 上の命令がすべて成功する。

## R07：motion を動いているものだけで回す

- **wave / lane / 前提**：W3a / R / R04。
- **所有**：runtime `src/motion/{state.rs,frame_loop.rs,animated.rs,timeline/api.rs}`。
- **目的**：commit と tick ごとに全 animation と全 frame loop を `Vec` に集めて走査するのをやめ、動いているものの集合（phase の遷移だけが書く）を回す。frame loop が tick ごとに signal を書くこと、Animated が step ごとに `Vec` を二つ作ることと、Animated ごとの `running` memo をやめる（RC-08）。

### 手順

1. `src/motion/state.rs` の `MotionState`：
   - 定義を次にする（`#[derive(Default)]` はそのまま）。`use std::collections::{BTreeMap, BTreeSet};` と `use std::rc::Weak;` を足し、`use kivact_types::cell::Registry;` を消す。
     ```rust
     pub struct MotionState {
         clock: RefCell<FrameClock>,
         /// Allocates the ids of loops and animations (creation order).
         next_id: Cell<u64>,
         /// Every live frame loop by id (DevTools, stats, the clock shift).
         loops: RefCell<BTreeMap<u64, Weak<LoopCell>>>,
         /// Every live animation and timeline by id.
         anims: RefCell<BTreeMap<u64, Weak<dyn Anim>>>,
         /// The loops that are started; written only by `LoopCell::set_run`.
         running_loops: RefCell<BTreeSet<u64>>,
         /// The animations whose phase is running; written only by their phase changes.
         running_anims: RefCell<BTreeSet<u64>>,
         /// How fast motion time runs for DevTools.
         time_scale: Cell<TimeScale>,
     }
     ```
   - `register_loop(&self, l: &Rc<LoopCell>)` と `register_anim(&self, a: &Rc<dyn Anim>)` を次の 5 つに置き換える。
     ```rust
     /// A new id for a loop or an animation.
     pub(crate) fn next_id(&self) -> u64 {
         let id = self.next_id.get() + 1;
         self.next_id.set(id);
         id
     }

     pub(crate) fn register_loop(&self, id: u64, l: &Rc<LoopCell>) {
         self.loops.borrow_mut().insert(id, Rc::downgrade(l));
     }

     pub(crate) fn register_anim(&self, id: u64, a: &Rc<dyn Anim>) {
         self.anims.borrow_mut().insert(id, Rc::downgrade(a));
     }

     /// A loop started (`true`) or stopped.
     pub(crate) fn set_loop_running(&self, id: u64, running: bool) {
         let mut set = self.running_loops.borrow_mut();
         if running { set.insert(id); } else { set.remove(&id); }
     }

     /// An animation's phase became running (`true`) or left it.
     pub(crate) fn set_anim_running(&self, id: u64, running: bool) {
         let mut set = self.running_anims.borrow_mut();
         if running { set.insert(id); } else { set.remove(&id); }
     }
     ```
   - `fn live_loops` と `fn live_anims` を次にし、動いているものだけを返す二つを足す。
     ```rust
     fn live_loops(&self) -> Vec<Rc<LoopCell>> {
         let mut loops = self.loops.borrow_mut();
         loops.retain(|_, l| l.strong_count() > 0);
         loops.values().filter_map(Weak::upgrade).collect()
     }

     fn live_anims(&self) -> Vec<Rc<dyn Anim>> {
         let mut anims = self.anims.borrow_mut();
         anims.retain(|_, a| a.strong_count() > 0);
         anims.values().filter_map(Weak::upgrade).collect()
     }

     /// The started loops, in creation order (ids of dropped loops are forgotten).
     fn running_loops(&self) -> Vec<Rc<LoopCell>> {
         let loops = self.loops.borrow();
         let mut running = self.running_loops.borrow_mut();
         let mut out = Vec::with_capacity(running.len());
         running.retain(|id| match loops.get(id).and_then(Weak::upgrade) {
             Some(l) => { out.push(l); true }
             None => false,
         });
         out
     }

     /// The running animations, in creation order.
     fn running_anims(&self) -> Vec<Rc<dyn Anim>> {
         let anims = self.anims.borrow();
         let mut running = self.running_anims.borrow_mut();
         let mut out = Vec::with_capacity(running.len());
         running.retain(|id| match anims.get(id).and_then(Weak::upgrade) {
             Some(a) => { out.push(a); true }
             None => false,
         });
         out
     }
     ```
   - `plan_frame` の先頭に次を足す。
     ```rust
     if self.running_loops.borrow().is_empty() && self.running_anims.borrow().is_empty() {
         return None;
     }
     ```
   - `wanted_frame` の `for l in self.live_loops()` を `for l in self.running_loops()`、`if self.live_anims().iter().any(|a| a.is_running())` を `if !self.running_anims().is_empty()` にする。
   - `on_frame_tick` の `for l in self.live_loops()` を `for l in self.running_loops()`、最後の `for a in self.live_anims() { if a.is_running() { a.step(now); } }` を `for a in self.running_anims() { a.step(now); }` にする。
   - `reduce_motion` の `self.live_anims()` を `self.running_anims()` にする。`restore_motion` の loop の走査を `self.running_loops()` にする。`apply_clock` は `live_anims()`・`live_loops()` のまま（一時停止中の animation も時計をずらす）。`animations()` と `stats()` は `live_*` のまま。
   - 以下、LoopCell の field の変更（手順 2）に合わせて、このファイルの `l.state.get_untracked()` を `l.run.get()` に、`LoopState::Running(run)` の pattern を `Some(run)` に、`LoopState::Stopped` を `None` にする（`apply_clock`、`on_frame_tick`、`wanted_frame`、`animations`、`restore_motion`）。`on_frame_tick` の `let frame = l.frames.get(); l.frames.set(frame + 1);` はそのまま。
2. `src/motion/state.rs` の `LoopCell` と `LoopState`：
   - `pub(crate) enum LoopState { Stopped, Running(LoopRun) }` を削除する。
   - `LoopCell` を次にする（`FrameInfo::frame` の意味（今までに走った回数）を保つため `frames` は残す。4 章 4.5 に記録）。
     ```rust
     // xtask-allow(state): run is the loop's phase (None when stopped); rate is its reactive projection, written only when the requested rate changes, never per tick; frames counts callbacks; gate and motion are setup options
     pub(crate) struct LoopCell {
         pub(crate) id: u64,
         pub(crate) owner: Owner,
         pub(crate) f: Handler<FrameInfo>,
         /// The running schedule (`None`: stopped), advanced at each tick.
         pub(crate) run: Cell<Option<LoopRun>>,
         /// The requested rate, for `FrameLoop::rate`.
         pub(crate) rate: Signal<Option<FrameRate>>,
         pub(crate) frames: Cell<u64>,
         pub(crate) gate: Cell<LoopGate>,
         pub(crate) motion: Cell<LoopMotion>,
     }
     ```
   - `LoopCell::set_run` を次にする。
     ```rust
     /// Start, change or stop (`None`) the loop.
     pub(crate) fn set_run(&self, run: Option<LoopRun>) {
         let was = self.run.get().is_some();
         self.run.set(run);
         self.rate.set(run.map(|r| r.rate));
         if was != run.is_some()
             && let Some(rt) = Runtime::get(self.owner.app())
         {
             MotionState::of(&rt).set_loop_running(self.id, run.is_some());
         }
     }
     ```
3. `src/motion/frame_loop.rs`：
   - `frame_loop` の `let state = owner.signal(LoopState::Stopped, at); let rate = crate::memo(..);` を `let rate = owner.signal(None, at);` にし、`let cell = Rc::new(LoopCell { owner, f, state, frames, gate, rate, motion })` を次にする。
     ```rust
     let id = MotionState::of(&rt).next_id();
     let cell = Rc::new(LoopCell {
         id,
         owner,
         f: crate::reactive::Handler::from_fn(f),
         run: Cell::new(None),
         rate,
         frames: Cell::new(0),
         gate: Cell::new(LoopGate::Always),
         motion: Cell::new(LoopMotion::Decorative),
     });
     MotionState::of(&rt).register_loop(id, &cell);
     ```
   - `start` の `let current = match cell.state.get_untracked() { LoopState::Stopped => None, LoopState::Running(run) => Some(run), };` を `let current = cell.run.get();` にする。
   - `rate()` の本体を `self.cell(Location::caller()).1.rate.into()` にする。
   - `use super::state::{..}` から `LoopState` を消す。
4. `src/motion/animated.rs`：
   - `pub(crate) struct AnimCell<T>` に field `pub(crate) motion_id: u64,` を足す。`create` の `let cell = Rc::new(AnimCell { owner, value, transition, current: .., phase: .. });` の前に `let motion_id = MotionState::of(&rt).next_id();` を置き、構造体式に `motion_id,` を足す。`MotionState::of(&rt).register_anim(&anim);`（今の行）を `MotionState::of(&rt).register_anim(motion_id, &anim);` にする。
   - `impl<T: Animatable> AnimCell<T>` に次を足し、このファイルの `self.phase.set(`・`cell.phase.set(` をすべて `self.set_phase(`・`cell.set_phase(` にする。`complete` の `self.phase.update(|phase| *phase = AnimPhase::Completed { target });` は `self.set_phase(AnimPhase::Completed { target });` にする。
     ```rust
     /// The only write of `phase`: keeps the motion system's running set.
     fn set_phase(&self, next: AnimPhase) {
         let was = self.is_running();
         self.phase.set(next);
         let now = self.is_running();
         if was != now
             && let Some(rt) = Runtime::get(self.owner.app())
         {
             MotionState::of(&rt).set_anim_running(self.motion_id, now);
         }
     }
     ```
     （`AnimPhase` は `PartialEq` を実装しているので `Signal::set` が使える。`set` は同じ値なら通知しない。今の `complete` は `update` で必ず通知していた。`Completed { target }` は前の値と違う phase から来るので通知は変わらない。）
   - `is_running`（`impl Anim`）を `self.phase.with_untracked(|p| matches!(p, AnimPhase::Running { .. }))` にする。
   - `fn publish(&self, c: &[f64])` の `*self.current.borrow_mut() = c.to_vec();` を次の 3 行にする。
     ```rust
     let mut current = self.current.borrow_mut();
     current.clear();
     current.extend_from_slice(c);
     drop(current);
     ```
   - `fn step(&self, now: u64)` を次にする（`current` をその場で進め、`Vec` を作らない）。
     ```rust
     fn step(&self, now: u64) {
         let Some((start_ns, run)) = self.phase.with_untracked(|p| match p {
             AnimPhase::Running { start_ns, run } => Some((*start_ns, Rc::clone(run))),
             _ => None,
         }) else {
             return;
         };
         let done = {
             let mut current = self.current.borrow_mut();
             let mut spring = run.spring.borrow_mut();
             run.timing
                 .step(&run.from, &run.to, start_ns, now, &mut *current, &mut spring)
         };
         if done {
             self.complete(&run);
         } else {
             let v = T::from_components(&self.current.borrow());
             self.value.update(move |slot| *slot = v);
         }
     }
     ```
   - `pub struct Animated<T>` の field `running: Value<bool>,` を `phase: Signal<AnimPhase>,` にする。`create` の `let phase = cell.phase; let running = crate::memo(..);` を `let phase = cell.phase;` にし、`Animated { app, cell: stored, value: Value::from(value), running }` を `Animated { app, cell: stored, value: Value::from(value), phase }` にする。`pub fn is_running(&self) -> bool` の本体を `self.phase.with(|p| matches!(p, AnimPhase::Running { .. }))` にする。手で書いた `Clone`・`Copy` はそのまま（`Signal` は Copy）。
5. `src/motion/timeline/api.rs`：
   - `pub(crate) struct TimelineCell` に field `pub(crate) motion_id: u64,` を足す。`TimelineCell::new(owner, at)` に引数 `motion_id: u64` を足して構造体式に入れ、`timeline()` の `let cell = Rc::new(TimelineCell::new(owner, at));` を次の 2 行にする。`register_anim(&anim)` を `register_anim(motion_id, &anim)` にする。
     ```rust
     let motion_id = MotionState::of(&rt).next_id();
     let cell = Rc::new(TimelineCell::new(owner, at, motion_id));
     ```
   - `impl TimelineCell` に、手順 4 と同じ形の `fn set_phase(&self, next: TimelinePhase)` を足す（`self.is_running()` は `impl Anim for TimelineCell` の `is_running`、`set_anim_running(self.motion_id, now)`。`self.owner` の名前が違えば TimelineCell の owner の field を使う）。このファイルの `self.phase.set(` と `cell.phase.set(`（`TimelinePhase` を書く行）をすべて `set_phase(` にする。`TimelinePhase` が `PartialEq` を実装していなければ STOP。

### 消すもの

- `LoopState`、`LoopCell.state`、`frame_loop` の `rate` memo、`Animated.running` memo、`MotionState` の `Registry` の 2 つの field、step ごとの `current.borrow().clone()` と `to_vec()`。

### 状態の閉じ方

- `MotionState`：内部可変の field が多い。定義の直前に次の 1 行を置く。
  `// xtask-allow(state): loops and anims hold every registration by id; running_loops and running_anims hold the ids whose phase is running and are written only by those phase changes; clock is the frame request phase; next_id allocates ids and time_scale is a DevTools setting`
- `LoopCell`：手順 2 の印。
- `AnimCell<T>`：`value`・`phase`（Signal）と `current`（RefCell）を持つので検査の対象。定義の直前に次の 1 行を置く（T の W1b が置いた印があれば、この行で置き換える）。
  `// xtask-allow(state): phase is the only phase and set_phase its only writer (it keeps the running set); current is the value now, stepped in place while running and published to value`
- `TimelineCell`：`motion_id` は constructor でだけ書く。T の W1b の印の文面の最後に `; set_phase is the only writer of phase` を足す。

### 試験

- 既存（変えずに通す）：e2e の `motion`、`motion_easing`、`timeline`、`node_ref`（`while_present`）、`presence` module、kivact-test の `real_window_tests` の frame loop の試験。

### 確認（tier a）

```
cargo fmt -p kivact-runtime
cargo check -p kivact-runtime --all-targets
cargo check -p kivact-core --all-targets
cargo clippy -p kivact-runtime --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-runtime
KIVACT_TEST_MIRROR=off cargo test -p kivact-runtime --lib
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- motion motion_easing timeline node_ref presence
```

### 完了条件

- `rg -n 'LoopState|Registry<|\.live\(\)|running: Value<bool>|current\.borrow\(\)\.clone\(\)' crates/kivact-runtime/src/motion` が何も返さない。
- 上の命令がすべて成功する。

## R08：診断の記録を記録者が有効にしたときだけにする

- **wave / lane / 前提**：W5a / 鎖 / R06。公開 API は変えない（core の `launch/runtime.rs` を持つので鎖に置く）。
- **所有**：runtime `src/reactive/{diag.rs,detail.rs,data.rs,host.rs,runtime.rs}`、`src/reactive.rs`、`src/host.rs`、`src/lib.rs`、core `src/launch/{runtime.rs,turn.rs}`、`src/client/trace.rs`、`src/devtools/lifecycle.rs`。
- **目的**：release の App が常に持っていた turn の記録（256 turn の ring、文字列の原因、書込みと再実行の列、stage の時間）を、DevTools・`kivact::test`・debug の trace sink のどれかが有効にしたときだけにする（RC-05 の内部の部分、PERF-9、RC-03 の「`record_stage` は記録中だけ」）。記録の型（`TurnRecord` など）はこの unit では変えない（型の統合は R25）。

### 手順

1. runtime `src/reactive/diag.rs` に次を足す（`pub struct Reported` の前）。
   ```rust
   /// Which recorders want the turn trace ([`Runtime::set_recorder`]). The
   /// trace (turn records, causes, writes, re-runs, stage times and the
   /// DevTools detail) is kept only while one of them is on; errors and
   /// diagnostics are always kept.
   #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
   pub struct RecordBits(u8);

   impl RecordBits {
       /// The DevTools agent records updates.
       pub const DEVTOOLS: RecordBits = RecordBits(1);
       /// A `kivact::test` screen (`Screen::trace`).
       pub const TEST: RecordBits = RecordBits(2);
       /// The debug comparison trace (`KIVACT_TRACE`).
       pub const TRACE_SINK: RecordBits = RecordBits(4);

       /// Whether any recorder is on.
       pub const fn any(self) -> bool {
           self.0 != 0
       }

       /// These bits with `bits` turned on or off.
       pub const fn with(self, bits: RecordBits, on: bool) -> RecordBits {
           if on {
               RecordBits(self.0 | bits.0)
           } else {
               RecordBits(self.0 & !bits.0)
           }
       }
   }
   ```
2. runtime `src/reactive/data.rs` の `pub(crate) struct Diag` に field `pub(crate) record: RecordBits,` を `detail` の前に足す（`Default` は `RecordBits(0)`）。`use crate::reactive::diag::RecordBits;` を足す。
3. runtime `src/reactive/diag.rs` の `impl Diag`：
   - `record_diagnostic`：`if self.current.notes.len() < PER_TURN { self.current.notes.push(..) }` を `if self.record.any() && self.current.notes.len() < PER_TURN { .. }` にする（ring への push と `tracing::debug!` はそのまま）。
   - `note`：`if self.current.notes.len() < PER_TURN {` を `if self.record.any() && self.current.notes.len() < PER_TURN {` にする。
   - `cause`、`record_write`、`record_run` の先頭に `if !self.record.any() { return; }` を足す（`record_run` の `self.current.runs += 1;` もその後にする）。
   - `push_record` の先頭に `if !self.record.any() { return; }` を足す。
4. runtime `src/reactive/diag.rs` の `impl Runtime`：
   - `pub fn set_detailed_trace(&self, on: bool)`（`#[doc(hidden)]` と `xtask-allow(internal-contract)` の行を含む）を削除し、次の二つを足す。
     ```rust
     /// Turn recorder `bits` on or off. While no recorder is on, no turn
     /// trace is kept: turning the last one off drops what was recorded.
     pub fn set_recorder(&self, bits: RecordBits, on: bool) {
         let mut diag = self.diag.borrow_mut();
         let was = diag.record.any();
         diag.record = diag.record.with(bits, on);
         match (was, diag.record.any()) {
             (false, true) => diag.detail.start(),
             (true, false) => {
                 diag.detail.stop();
                 diag.turns.clear();
             }
             _ => {}
         }
     }

     /// Whether a recorder is on (the turn trace is being kept).
     pub fn is_recording(&self) -> bool {
         self.diag.borrow().record.any()
     }
     ```
   - `record_cause_with` の先頭に `if !self.is_recording() { return; }` を足す。
   - `record_stage(&self, stage: Stage, elapsed: std::time::Duration)` を `record_stage(&self, stage: Stage, started: std::time::Instant)` にし、本体を次にする。
     ```rust
     let mut diag = self.diag.borrow_mut();
     if !diag.record.any() {
         return;
     }
     let elapsed = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
     diag.detail.stage(stage, self.now_ns(), elapsed);
     ```
     （`self.now_ns()` は `self.clock` を読むだけで diag を borrow しない。）
5. runtime `src/reactive/host.rs`：`fn begin` の `self.record_stage(stage, turn.started.elapsed());` を `self.record_stage(stage, turn.started);`、`TurnGuard::stage` の `self.rt.record_stage(stage, self.started.elapsed());` を `self.rt.record_stage(stage, self.started);` にする。
6. runtime `src/host.rs`：
   - `RuntimeConfig` に field を足す。
     ```rust
     /// The recorders on from the start (a test screen records its whole run).
     pub record: RecordBits,
     ```
   - `Runtime::new` の `let registration = RegistrationGuard(rt.id());` の直後に `rt.set_recorder(config.record, true);` を置く（`config.record` が空なら何も変わらない）。
   - `poll_async_round` の `self.cause_with(CauseTag::Timer, "timer".into());` と `self.cause_with(CauseTag::Task, "task".into());` をそれぞれ `if rt.is_recording() { .. }` で包む。
7. runtime `src/reactive.rs` の `pub use diag::{ReportKind, Reported};` を `pub use diag::{RecordBits, ReportKind, Reported};` にし、`src/lib.rs` の `pub use reactive::{..}` に `RecordBits` を足す。
8. core `src/launch/runtime.rs` の `Launch::start`：
   - `RuntimeConfig { start_ns: .., blocking_threads: .., at, }` に次の field を足す。
     ```rust
     record: if matches!(self.mode, ClientMode::Test { .. }) {
         RecordBits::TEST
     } else {
         RecordBits::default()
     },
     ```
   - `let rt = Runtime::new(..);` の直後に次を足す。
     ```rust
     #[cfg(debug_assertions)]
     if crate::client::trace::Trace::of(&rt).is_on() {
         rt.set_recorder(RecordBits::TRACE_SINK, true);
     }
     ```
   - `use kivact_runtime::{..}` に `RecordBits` を足す。
9. core `src/client/trace.rs` の `impl Trace` に次を足す。
   ```rust
   /// Whether this App writes a comparison trace.
   pub(crate) fn is_on(&self) -> bool {
       self.sink.is_some()
   }
   ```
10. core `src/devtools/lifecycle.rs` の `rt.set_detailed_trace(recording.updates);` を `rt.set_recorder(kivact_runtime::RecordBits::DEVTOOLS, recording.updates);` にする。
11. core `src/launch/turn.rs`：
    - `turn.cause_with(CauseTag::Mount, "mounts".into());` を `if rt.is_recording() { turn.cause_with(CauseTag::Mount, "mounts".into()); }` にする。
    - `turn.cause_with(CauseTag::Input, describe(event));` を `if rt.is_recording() { turn.cause_with(CauseTag::Input, describe(event)); }` にする（`describe` の `format!` は記録中だけ走る）。

### 消すもの

- `Runtime::set_detailed_trace`。記録者が無いときの turn の ring・原因・書込み・再実行・stage の記録。

### 状態の閉じ方

- `Diag`：T の W1b が置いた印の文面の最後に次を足す（同じ 1 行）：`; record says which recorders are on, and turns, current and detail stay empty while it is off`。
- `RecordBits`：`Bits` で終わる型（6.1 節の除外）。

### 試験

- 既存（変えずに通す）：e2e の `diagnostics::trace_records_causes_writes_reruns_ops_and_duration`、`focus::closing_a_modal_gives_the_focus_back_to_the_opener`（`Screen::trace` を読む。test の画面は `RecordBits::TEST` で記録する）、`e2e_native` の `devtools` module（DevTools の更新の記録）。

### 確認（tier a）

```
cargo fmt -p kivact-runtime -p kivact-core
cargo check -p kivact-runtime --all-targets
cargo check -p kivact-core --all-targets
cargo check -p kivact-test --all-targets
cargo clippy -p kivact-runtime -p kivact-core --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-runtime
cargo xtask check --no-write --crate kivact-core
KIVACT_TEST_MIRROR=off cargo test -p kivact-runtime --lib
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- diagnostics focus
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --features devtools --test e2e_native -- devtools
```

### 完了条件

- `rg -n 'set_detailed_trace' crates` が何も返さない。
- 上の命令がすべて成功する。

## R09：graph の node を 96 byte 以下にし、診断の情報を別の表に置く

- **wave / lane / 前提**：W5a / 鎖 / R08。公開 API は変えない。
- **所有**：runtime `src/reactive/{data.rs,graph.rs,turn.rs,owner.rs,arena.rs,selector.rs,frame.rs,handles.rs,host.rs,diag.rs,scope.rs,object.rs,trigger.rs,owner_handle.rs}`。
- **目的**：`RNode`（今は約 160 byte）から診断用の `at`・`ty`・`written` と、同じ allocation を指す二つ目の太い pointer をなくし、effect の `cleanups` を box に入れ、大きさを `const` の assert で守る（RC-06、MEM-6）。診断の情報は `Diag.node_meta` の表に置き、debug build か記録者が有効なときだけ書く。

### 手順

1. `src/reactive/data.rs` の型を次にする。
   - `pub(crate) trait Compute` に method を足す。
     ```rust
     /// The cell itself as `Any` (a memo's value is read through it).
     fn into_any(self: Rc<Self>) -> Rc<dyn Any>;
     ```
     `graph.rs` の `impl<T: 'static> Compute for MemoCell<T>`、`impl Compute for EffectCell`、`impl<T: 'static> Compute for WatcherCell<T>` に `fn into_any(self: Rc<Self>) -> Rc<dyn Any> { self }` を足す。
   - `RNode`、`NodeData`、`SourceNode`、`MemoNode`、`EffectNode` を次にし、`pub(crate) struct Computation` とその `impl` を削除する。
     ```rust
     pub(crate) struct RNode {
         pub(crate) owner: Key,
         pub(crate) owner_index: u32,
         /// Where a memo or an effect stands; a source is always `Clean`.
         pub(crate) state: CompState,
         pub(crate) data: NodeData,
     }

     const _: () = assert!(
         std::mem::size_of::<RNode>() <= 96,
         "RNode grew: diagnostics belong in Diag::node_meta, rare members in a box"
     );

     /// What a graph node is, with the data only that kind has.
     pub(crate) enum NodeData {
         /// A writable source (signal, var storage, trigger, selector key).
         Source(SourceNode),
         /// A cached derived value.
         Memo(MemoNode),
         /// An effect.
         Effect(EffectNode),
         Watcher(EffectNode),
     }

     pub(crate) struct SourceNode {
         /// The value (a `SourceCell<T>`).
         pub(crate) cell: Rc<dyn Any>,
         /// (subscriber, index of this node in subscriber.deps).
         pub(crate) subs: Links,
         /// Write generation of the last change.
         pub(crate) changed_at: kivact_types::cell::Generation,
     }

     pub(crate) struct MemoNode {
         /// The memo (a `MemoCell<T>`; its value is read through `into_any`).
         pub(crate) comp: Rc<dyn Compute>,
         /// (subscriber, index of this node in subscriber.deps).
         pub(crate) subs: Links,
         /// (source, index of this node in source.subs), in read order.
         pub(crate) deps: Links,
     }

     pub(crate) struct EffectNode {
         pub(crate) comp: Rc<dyn Compute>,
         /// (source, index of this node in source.subs), in read order.
         pub(crate) deps: Links,
         pub(crate) kind: EffectKind,
         /// Runs in the current stabilize pass (R03).
         pub(crate) pass_runs: u8,
         /// Creation order: queued effects run in it.
         pub(crate) seq: u64,
         /// Cleanups registered by the running effect (most effects have none).
         pub(crate) cleanups: Option<Box<Vec<Box<dyn FnOnce()>>>>,
     }
     ```
   - `impl RNode` を次にする（`kind`・`subs`・`subs_mut` は今の本体のまま、`computation`・`computation_mut` を消す）。
     ```rust
     /// Whether this node computes (a memo or an effect).
     pub(crate) fn is_computation(&self) -> bool {
         !matches!(self.data, NodeData::Source(_))
     }

     /// The computation of a memo or an effect.
     pub(crate) fn comp(&self) -> Option<&Rc<dyn Compute>> {
         match &self.data {
             NodeData::Source(_) => None,
             NodeData::Memo(m) => Some(&m.comp),
             NodeData::Effect(e) | NodeData::Watcher(e) => Some(&e.comp),
         }
     }

     /// What a memo or an effect read during its last run (empty for a source).
     pub(crate) fn deps(&self) -> &[(Key, u32)] {
         match &self.data {
             NodeData::Source(_) => &[],
             NodeData::Memo(m) => &m.deps,
             NodeData::Effect(e) | NodeData::Watcher(e) => &e.deps,
         }
     }

     pub(crate) fn deps_mut(&mut self) -> Option<&mut Links> {
         match &mut self.data {
             NodeData::Source(_) => None,
             NodeData::Memo(m) => Some(&mut m.deps),
             NodeData::Effect(e) | NodeData::Watcher(e) => Some(&mut e.deps),
         }
     }

     /// The cell a handle reads: a source's value or a memo's cell.
     pub(crate) fn cell(&self) -> Rc<dyn Any> {
         match &self.data {
             NodeData::Source(s) => Rc::clone(&s.cell),
             NodeData::Memo(m) => Rc::clone(&m.comp).into_any(),
             NodeData::Effect(e) | NodeData::Watcher(e) => Rc::clone(&e.comp).into_any(),
         }
     }
     ```
   - `pub struct NodeInfo` は変えない。その直前に次を足す。
     ```rust
     /// The diagnostics of one graph node (`Diag::node_meta`): kept in debug
     /// builds and while a recorder is on, never in the node itself.
     #[derive(Clone, Copy)]
     pub(crate) struct NodeMeta {
         pub(crate) at: &'static Location<'static>,
         pub(crate) ty: &'static str,
         /// Location and turn of the last value-changing write (sources).
         pub(crate) written: Option<(&'static Location<'static>, Turn)>,
     }
     ```
   - `pub(crate) struct Diag` に field `pub(crate) node_meta: std::collections::HashMap<Key, NodeMeta>,` を足す。
2. `src/reactive/arena.rs` の `pub(crate) struct Tomb` の `pub(crate) created_at: &'static Location<'static>,` を `pub(crate) created_at: Option<&'static Location<'static>>,` にする。次の命令で読む側を直し、`cargo check -p kivact-runtime` の残りの E0308 を「作る側は `Some(..)` で包む、読む側は `.and_then(|t| t.created_at)` にする」の二つの形だけで直す。
   ```
   git ls-files 'crates/kivact-runtime/*.rs' | xargs perl -pi -e 's/tomb\.map\(\|t\| t\.created_at\)/tomb.and_then(|t| t.created_at)/g'
   ```
   `owner.rs` の object と const の tomb（`Tomb { created_at, disposed_by: .. }`・`Tomb { created_at: at, disposed_by: .. }`）は `created_at: Some(created_at)`・`created_at: Some(at)` にする。
3. `src/reactive/graph.rs`：
   - `impl Runtime` に次を足す。
     ```rust
     /// Whether graph nodes keep their diagnostics (`Diag::node_meta`).
     fn keeps_node_meta(&self) -> bool {
         cfg!(debug_assertions) || self.diag.borrow().record.any()
     }

     /// Where node `key` was created, if its diagnostics are kept.
     pub(crate) fn node_at(&self, key: Key) -> Option<&'static Location<'static>> {
         self.diag.borrow().node_meta.get(&key).map(|m| m.at)
     }
     ```
   - `fn insert_node(&self, owner: Key, cell: Rc<dyn Any>, data: NodeData, at, ty)` を `fn insert_node(&self, owner: Key, data: NodeData, at: &'static Location<'static>, ty: &'static str) -> Key` にし、`RNode { owner, owner_index: 0, cell, at, ty, data }` を次にする。
     ```rust
     let state = if matches!(data, NodeData::Source(_)) {
         CompState::Clean
     } else {
         CompState::Stale(Staleness::Dirty)
     };
     let key = self.graph.borrow_mut().insert(RNode {
         owner,
         owner_index: 0,
         state,
         data,
     });
     if self.keeps_node_meta() {
         self.diag
             .borrow_mut()
             .node_meta
             .insert(key, NodeMeta { at, ty, written: None });
     }
     ```
   - `create_source`：`NodeData::Source(SourceNode { cell, subs: Links::default(), changed_at: Generation::new() })` を作り、`self.insert_node(owner, data, at, ty)` を呼ぶ（`written: None` を消す）。
   - `create_memo`：`let compute: Rc<dyn Compute> = cell.clone();` と `MemoNode { subs, run: Computation::new(compute) }` を `MemoNode { comp: cell, subs: Links::default(), deps: Links::default() }` にする（`cell` は `Rc<MemoCell<T>>`。`comp: cell` で `Rc<dyn Compute>` に unsize される）。
   - `create_effect`・`watch`：`EffectNode { kind, seq, run: Computation::new(compute), cleanups: Vec::new(), pass_runs: 0 }` を `EffectNode { comp: cell, deps: Links::default(), kind, pass_runs: 0, seq, cleanups: None }` にする。
   - `queue_effect`・`dequeue_effect`：次の形にする（`kind`・`seq` を先に写して `n.data` の borrow を終えてから `&mut n.state` を渡す）。
     ```rust
     fn queue_effect(&self, key: Key) -> bool {
         let mut graph = self.graph.borrow_mut();
         let Some(n) = graph.get_mut(key) else { return false };
         let (NodeData::Effect(e) | NodeData::Watcher(e)) = &n.data else { return false };
         let (kind, seq) = (e.kind, e.seq);
         self.sched.borrow_mut().push(&mut n.state, kind, seq, key)
     }
     ```
     `dequeue_effect` も同じ形で `self.sched.borrow_mut().remove(&mut n.state, kind, seq, key);`。
   - `read_node`：`(n.cell.clone(), memo)` の `memo` の計算を `let memo = matches!(n.data, NodeData::Memo(_)).then_some(n.state);`、cell を `n.cell()` にする。
   - `write_node`：`let cell = self.graph.borrow().get(key).map(|n| n.cell.clone());` を `let cell = self.graph.borrow().get(key).and_then(|n| match &n.data { NodeData::Source(s) => Some(Rc::clone(&s.cell)), _ => None });` にする。`if mutate(&cell) {` の中の `written` の部分を次にする。
     ```rust
     let turn = self.diag.borrow().turn();
     let mut diag = self.diag.borrow_mut();
     let meta = diag.node_meta.get_mut(&key).map(|m| {
         m.written = Some((at, turn));
         *m
     });
     if let Some(meta) = meta {
         diag.record_write(meta.ty, meta.at, at);
         diag.detail.write(NodeId(key), meta.ty, meta.at, at, true);
     }
     drop(diag);
     self.notify_subscribers(key);
     ```
     `else` の枝の `let written = self.graph.borrow().get(key).map(|n| (n.ty, n.at));` を `let written = diag.node_meta.get(&key).map(|m| (m.ty, m.at));` にする。
   - `notify_subscribers`：`if let NodeData::Source(s) = &mut n.data { s.changed_at = epoch; }` はそのまま。`NodeData::Memo(m) => { let was_clean = m.run.state == CompState::Clean; if !m.run.state.mark(color) {..} ..}` と `NodeData::Effect(e) | NodeData::Watcher(e) => { if !e.run.state.mark(color) {..} queued_any |= sched.push(&mut e.run.state, e.kind, e.seq, k); }` を、`n.state` を使う形にする（`match &n.data` で kind を判定して `kind`・`seq` を写してから `n.state.mark(color)` と `sched.push(&mut n.state, kind, seq, k)`。memo の subscriber は `n.subs()` から集める）。
   - `refresh`・`needs_run`：`n.computation().map(|c| c.state)` の形を `n.is_computation().then_some(n.state)` にする。
   - `recompute`：`n.computation().filter(|c| c.compute.busy()).map(|_| n.at)` を `n.comp().filter(|c| c.busy()).map(|_| ())`（`busy` は bool で十分）にし、`misuse_at(ErrorKind::BorrowedWrite, "memo re-read ..", at)` を次にする。
     ```rust
     crate::reactive::error::misuse(new_error(
         ErrorKind::BorrowedWrite,
         "memo re-read while its own `with` borrow is active after a dependency changed; read it after `with` returns",
         self.node_at(key),
     ));
     ```
     `let (owner, at) = (n.owner, n.at);` を `let owner = n.owner;` にし、`let Some(c) = n.computation_mut() else { return }; c.state = CompState::Running; (c.compute.clone(), effect, owner, at)` を次にする。
     ```rust
     let Some(compute) = n.comp().cloned() else { return };
     n.state = CompState::Running;
     (compute, effect, owner)
     ```
     `self.diag.borrow_mut().record_run(label, at);` と `diag.detail.run(NodeId(key), label, at, changed);` は、`self.node_at(key)` が `Some(at)` のときだけ呼ぶ（`if let Some(at) = self.node_at(key) { .. }`）。`c.state = CompState::Clean;`（R03 の `is_memo_change` のブロック）は `n.state = CompState::Clean;` にする。
   - `recheck_after_own_writes`：`Some(NodeData::Memo(m)) if m.run.state.staleness().is_some()` を `Some(NodeData::Memo(_)) if g.get(*d).is_some_and(|m| m.state.staleness().is_some())` にし、最後の `e.run.state.mark(stale);` を `n.state.mark(stale);`（`n` は `g.get_mut(key)`）にする。
   - `run_node_cleanups`：`std::mem::take(&mut e.cleanups)` を `e.cleanups.take()` にし、`if cleanups.is_empty()` の判定を `let Some(cleanups) = cleanups else { return };` にして `for c in cleanups.into_iter().rev()` で回す。
   - `set_deps`：`c.deps.push(..)` を `if let Some(deps) = g.get_mut(node).and_then(RNode::deps_mut) { deps.push((src, sub_index)); }` にする。
   - `remove_node`：`let at = g.get(key).map(|n| n.at)?;` を消し、`g.remove(key, Some(Tomb { created_at: at, disposed_by }))` を次にする。
     ```rust
     let created_at = self.diag.borrow_mut().node_meta.remove(&key).map(|m| m.at);
     g.remove(key, Some(Tomb { created_at, disposed_by }))
     ```
   - `fn unlink_deps`・`fn remove_dep_entry`：`computation_mut()` と `c.deps` の参照を `deps_mut()` にする。
   - `node_snapshot`：本体を次にする。
     ```rust
     let meta = self.diag.borrow().node_meta.get(&key).copied()?;
     let g = self.graph.borrow();
     let n = g.get(key)?;
     Some(NodeInfo {
         kind: n.kind(),
         ty: meta.ty,
         at: meta.at,
         written: meta.written,
         deps: n.deps().iter().map(|(d, _)| NodeId(*d)).collect(),
     })
     ```
   - `mark_memo_subscribers`（R03）：`n.computation_mut()` と `c.state` を `n.is_computation()` と `n.state` にする（`kind`・`seq` を先に写す）。
4. `src/reactive/turn.rs`：
   - `Sched::unmark`（R03）の `e.run.state` を `n.state` にする（`graph.get_mut(key)` の `n` の `n.state`、effect であることは `matches!(n.data, NodeData::Effect(_) | NodeData::Watcher(_))` で確かめる）。
   - `Runtime::stabilize` の cycle の `let at = self.graph.borrow().get(key).map(|n| n.at);` を `let at = self.node_at(key);` にする。
   - `abandon_render_effects`・`abandon_user_queue` の `g.get_mut(k).and_then(RNode::computation_mut)` と `c.state.abandon()` を `if let Some(n) = g.get_mut(k) && n.is_computation() { n.state.abandon(); }` にする。
5. `src/reactive/owner.rs`：
   - `add_effect_cleanup` の `e.cleanups.push(f)` を `e.cleanups.get_or_insert_with(Default::default).push(f)` にする。
   - `dispose_members` の `Some(std::mem::take(&mut e.cleanups))` を `e.cleanups.take()` にし、`if let Some(cleanups) = cleanups.filter(|c| !c.is_empty())` を `if let Some(cleanups) = cleanups` にする（`Option<Box<Vec<..>>>` を `into_iter().rev()` で回す）。
6. 残りの参照：`rg -n '\.at\b|\.ty\b|computation\(|computation_mut\(|\.run\.state|\.compute\b|n\.cell\b' crates/kivact-runtime/src/reactive` の各行を、上の規則（`at`・`ty` は `node_at`/`node_meta`、`computation` は `comp`/`deps`/`is_computation`、`run.state` は `n.state`、`n.cell` は `n.cell()`）で直す。`selector.rs` の `x.subs()` はそのまま。
7. `cargo check -p kivact-runtime` が `size_of::<RNode>() <= 96` の assert で失敗したら、次の一時的な試験を `data.rs` に足して `cargo test -p kivact-runtime --lib rnode_sizes -- --nocapture` で大きさを出し、その値を書いて STOP する（試験は消してから止める）。
   ```rust
   #[test]
   fn rnode_sizes() {
       eprintln!("RNode {} NodeData {} Memo {} Effect {} Source {} Links {}",
           std::mem::size_of::<RNode>(), std::mem::size_of::<NodeData>(),
           std::mem::size_of::<MemoNode>(), std::mem::size_of::<EffectNode>(),
           std::mem::size_of::<SourceNode>(), std::mem::size_of::<crate::reactive::links::Links>());
   }
   ```

### 消すもの

- `RNode.cell`、`RNode.at`、`RNode.ty`、`SourceNode.written`、`Computation`、`RNode::computation`/`computation_mut`、`EffectNode.cleanups` の inline の `Vec`、memo と effect の二重の `Rc`。

### 状態の閉じ方

- `RNode`：`state: CompState`（唯一の phase）と `owner_index`（位置の identity）。node は arena の payload で、`&mut` で書くのは遷移の関数だけ。定義の直前に次の 1 行を置く（T の W1b の印があれば置き換える）。
  `// xtask-allow(state): state is the node's only phase (always Clean for a source); owner_index is its position in the owner's node list`
- `EffectNode`：R03 の印を次の 1 行で置き換える。
  `// xtask-allow(state): seq is the creation-order identity, pass_runs the per-pass run count reset when each stabilize ends, cleanups the registered cleanups taken by each run; the phase is RNode::state`
- `NodeMeta`：`Copy` の記録。constructor と `write_node` の `written` の更新だけで書く。`written` の Option は一つ。対象外。
- `Diag`：field の追加。R08 の文面の最後に `; node_meta holds the diagnostics of nodes created while it is kept` を足す。

### 試験

- 既存（変えずに通す）：runtime の unit 試験、e2e の `runtime`、`diagnostics`（`why` の `last_write` を読む試験は test の画面＝debug build で動く）、`disposal`（`Disposed` の `created_at`）、`flows`、`lifecycle`。

### 確認（tier a）

```
cargo fmt -p kivact-runtime
cargo check -p kivact-runtime --all-targets
cargo check -p kivact-runtime --all-targets --release
cargo clippy -p kivact-runtime --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-runtime
KIVACT_TEST_MIRROR=off cargo test -p kivact-runtime --lib
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- runtime diagnostics disposal flows lifecycle
```

### 完了条件

- `rg -n 'struct Computation|fn computation|\bwritten: Option' crates/kivact-runtime/src/reactive/data.rs` が `NodeMeta` の `written` 以外を返さない。
- `const _: () = assert!(std::mem::size_of::<RNode>() <= 96` が `data.rs` にあり、上の命令（release の check を含む）がすべて成功する。

## R10：計測する `virtual_list` の scroll を O(log N) にする

- **wave / lane / 前提**：W3a / R / R07。
- **所有**：core `src/view/flow/virtual_list.rs`、新規 core `src/view/flow/virtual_list/heights.rs`。
- **目的**：`RowHeight::Measured` の list が scroll のたびに全行の高さを足す O(N) をやめ、行の高さの Fenwick 木で範囲を二分探索する。重複 key の検出を list の変化の pass に移し、scroll のたびに同じ `DuplicateKey` を報告するのをやめる（RC-23 の前半）。`Held` を `Option<K>` にする（RC-40 の `Held` の項。付属書 C の RC-40 の unit はこのファイルを扱わない。4 章 4.3）。

### 手順

1. 新規 `src/view/flow/virtual_list/heights.rs`：
   ```rust
   //! Role: Prefix sums of a virtual list's row heights (a Fenwick tree).

   /// The heights of a list's rows by index, with O(log N) prefix sums,
   /// point updates and position lookups.
   pub(super) struct Heights {
       /// The row heights.
       rows: Vec<f64>,
       /// The Fenwick tree over `rows` (1-based).
       tree: Vec<f64>,
   }

   impl Heights {
       /// The tree of `rows` (O(N)).
       pub(super) fn build(rows: Vec<f64>) -> Self {
           let n = rows.len();
           let mut tree = vec![0.0; n + 1];
           for i in 1..=n {
               tree[i] += rows[i - 1];
               let parent = i + (i & i.wrapping_neg());
               if parent <= n {
                   let value = tree[i];
                   tree[parent] += value;
               }
           }
           Heights { rows, tree }
       }

       /// An empty list.
       pub(super) fn empty() -> Self {
           Heights { rows: Vec::new(), tree: vec![0.0] }
       }

       /// The height of the rows before `index`.
       pub(super) fn prefix(&self, index: usize) -> f64 {
           let mut sum = 0.0;
           let mut i = index.min(self.rows.len());
           while i > 0 {
               sum += self.tree[i];
               i &= i - 1;
           }
           sum
       }

       /// The height of every row.
       pub(super) fn total(&self) -> f64 {
           self.prefix(self.rows.len())
       }

       /// Set the height of row `index`.
       pub(super) fn set(&mut self, index: usize, height: f64) {
           let Some(old) = self.rows.get_mut(index) else { return };
           let delta = height - *old;
           *old = height;
           let mut i = index + 1;
           while i < self.tree.len() {
               self.tree[i] += delta;
               i += i & i.wrapping_neg();
           }
       }

       /// The index of the row that contains position `y` (the last row
       /// for a `y` past the end, 0 for an empty list).
       pub(super) fn index_at(&self, y: f64) -> usize {
           let n = self.rows.len();
           if n == 0 {
               return 0;
           }
           let mut pos = 0;
           let mut rest = y;
           let mut step = n.next_power_of_two();
           while step > 0 {
               let next = pos + step;
               if next <= n && self.tree[next] <= rest {
                   pos = next;
                   rest -= self.tree[next];
               }
               step >>= 1;
           }
           pos.min(n - 1)
       }
   }

   #[cfg(test)]
   mod tests {
       //! Role: Prefix sums and lookups match a plain sum.

       use super::Heights;

       #[test]
       fn prefix_set_and_index_at_match_a_plain_sum() {
           let mut h = Heights::build(vec![10.0, 20.0, 30.0, 40.0, 50.0]);
           assert_eq!(h.prefix(3), 60.0);
           assert_eq!(h.total(), 150.0);
           assert_eq!(h.index_at(0.0), 0);
           assert_eq!(h.index_at(29.9), 1);
           assert_eq!(h.index_at(30.0), 2);
           assert_eq!(h.index_at(1000.0), 4);
           h.set(1, 5.0);
           assert_eq!(h.prefix(2), 15.0);
           assert_eq!(h.index_at(15.0), 2);
       }
   }
   ```
2. `virtual_list.rs` の先頭の `use` の後に `mod heights;` と `use heights::Heights;` を置く。
3. `virtual_list.rs` の `struct VState<K, T>` に次の 3 つの field を `measured` の後に足し、`VState { rows: HashMap::new(), measured: HashMap::new(), .. }` の構造体式に `heights: Heights::empty(), index_of: HashMap::new(), indexed: 0,` を足す。
   ```rust
   /// Row heights by list index (`Measured` lists), rebuilt when the list changes.
   heights: Heights,
   /// The list index of each key (its first occurrence), rebuilt when the list changes.
   index_of: HashMap<K, usize>,
   /// How many items `index_of` was built for.
   indexed: usize,
   ```
4. `Held` を消す：`#[derive(PartialEq)] enum Held<K> { Nothing, Holding { key: K } }` を削除し、`struct HeldState<K>` の `value: Held<K>,` を `value: Option<K>,` にする。`fn key(&self) -> Option<&K>` の本体を `self.value.as_ref()`、`fn set(&mut self, key: Option<K>)` の本体を次にする。構造体式の `value: Held::Nothing,` を `value: None,` にする。
   ```rust
   if self.value != key {
       self.value = key;
       self.changed.notify();
   }
   ```
   （`K: PartialEq` は `ItemKey` が含む。`impl<K: PartialEq> HeldState<K>` のまま。）
5. list の変化の pass：`impl IntoView for VirtualList` の `let seeker = Rc::new(Seeker { .. });` の直後（`bind_visible(..)` の前）に次を足す。この effect は `bind_visible` の effect より先に作るので、同じ変化では先に走る。
   ```rust
   // The list changed: index the keys (the first occurrence wins), report
   // duplicates once, rebuild the row heights and forget the measurements
   // of keys that left.
   let index_seeker = seeker.clone();
   let dup_owner = cx.owner;
   cx.effect(EffectKind::Internal, at, move || {
       let dups = index_seeker.reindex();
       if dups.is_empty() {
           return;
       }
       if let Some(rt) = Runtime::get(dup_owner.app()) {
           for h in dups {
               rt.report(
                   None,
                   UiError::new(
                       ErrorKind::DuplicateKey { key_hash: h },
                       "duplicate virtual_list key; later item dropped",
                       Some(at),
                   ),
               );
           }
       }
   });
   ```
   そして `impl<K: ItemKey, T: PropValue> Seeker<K, T>` に次を足す。
   ```rust
   /// Rebuild the key index (and the heights of a `Measured` list) from the
   /// current items (tracked). Returns the hashes of duplicate keys.
   fn reindex(&self) -> Vec<u64> {
       let estimate = match self.height {
           RowHeight::Measured { estimate } => Some(estimate),
           RowHeight::Fixed(_) => None,
       };
       kivact_runtime::pure(|| {
           self.items.with(|list| {
               let mut s = self.state.borrow_mut();
               let mut dups = Vec::new();
               let mut index_of = HashMap::with_capacity(list.len());
               for (i, it) in list.iter().enumerate() {
                   let k = (self.key)(it);
                   if index_of.contains_key(&k) {
                       dups.push(key_hash(&k));
                   } else {
                       index_of.insert(k, i);
                   }
               }
               if let Some(estimate) = estimate {
                   let rows = list
                       .iter()
                       .map(|it| s.measured.get(&(self.key)(it)).copied().unwrap_or(estimate).max(1.0))
                       .collect();
                   s.heights = Heights::build(rows);
                   s.measured.retain(|k, _| index_of.contains_key(k));
               }
               s.index_of = index_of;
               s.indexed = list.len();
               dups
           })
       })
   }
   ```
6. 計測の更新：`hd.observe_layout(move |_, size| { .. let prev = st.borrow_mut().measured.insert(key.clone(), size.height); if prev != Some(size.height) { measured_version.notify(); } })` を次にする。
   ```rust
   hd.observe_layout(move |_, size| {
       let Some(st) = st.upgrade() else { return };
       let mut s = st.borrow_mut();
       let prev = s.measured.insert(key.clone(), size.height);
       if prev != Some(size.height) {
           if let Some(&i) = s.index_of.get(&key) {
               s.heights.set(i, size.height.max(1.0));
           }
           drop(s);
           measured_version.notify();
       }
   });
   ```
7. `Seeker::visible` の `RowHeight::Measured { estimate } => { let mut y = 0.0f64; let mut seen = ..; for it in list { .. } drop(s); let mut sm = ..; if sm.measured.len() > .. { .. } y }` の腕全体を次にする。`visible` の戻り値の `dups` は `Measured` では常に空になる（報告は手順 5 の effect）。
   ```rust
   RowHeight::Measured { .. } => {
       let len = list.len().min(s.indexed);
       let first = s.heights.index_at(lo);
       let last = (s.heights.index_at(hi) + 1).min(len);
       for (i, it) in list.iter().enumerate().take(last).skip(first) {
           let k = (self.key)(it);
           if s.index_of.get(&k) != Some(&i) {
               continue;
           }
           let need = match s.rows.get(&k) {
               Some(r) => r.sig.with_untracked(|cur| cur != it),
               None => true,
           };
           visible.push((k, need.then(|| it.clone()), s.heights.prefix(i), None));
       }
       // The row with the focus stays mounted outside the range.
       if let Some(fk) = focused_key.as_ref()
           && let Some(&i) = s.index_of.get(fk)
           && (i < first || i >= last)
           && let Some(it) = list.get(i)
       {
           let need = match s.rows.get(fk) {
               Some(r) => r.sig.with_untracked(|cur| cur != it),
               None => true,
           };
           let top = s.heights.prefix(i);
           let at = visible.partition_point(|v| v.2 < top);
           visible.insert(at, (fk.clone(), need.then(|| it.clone()), top, None));
       }
       s.heights.total()
   }
   ```
8. `Seeker::fixed_visible`：`let mut seen = std::collections::HashSet::new();` と、`if !seen.insert(k.clone()) { dups.push(key_hash(&k)); continue; }` を `if s.index_of.get(&k) != Some(&i) { continue; }` にする。focus の行の `focused_key.filter(|fk| !seen.contains(*fk))` と `list.iter().enumerate().find(..)` を次にする（`first..last` の外にあるときだけ足す）。
   ```rust
   if let Some(fk) = focused_key
       && let Some(&i) = s.index_of.get(fk)
       && (i < first || i >= last)
       && let Some(it) = list.get(i)
   {
   ```
   `dups` の引数は使われなくなるので、`fixed_visible` と `visible` の戻り値・引数から `dups` を消し、`bind_visible` の `for h in dups { rt.report(..) }` を削除する。
9. `Seeker::run`（`scroll_to` の位置の計算）の `let place = kivact_runtime::pure(|| { self.items.with(|list| { .. for it in list { .. } .. }) });` を次にする。
   ```rust
   let place = kivact_runtime::pure(|| {
       self.items.with(|list| {
           let s = self.state.borrow();
           let i = *s.index_of.get(&want)?;
           match self.height {
               RowHeight::Fixed(h) => {
                   let h = h.max(1.0);
                   Some((i as f64 * h, h, list.len() as f64 * h))
               }
               RowHeight::Measured { estimate } => {
                   let top = s.heights.prefix(i);
                   let height = s.measured.get(&want).copied().unwrap_or(estimate).max(1.0);
                   Some((top, height, s.heights.total()))
               }
           }
       })
   });
   ```

### 消すもの

- `Held`、`visible` の全行の走査、`visible`・`fixed_visible` の `HashSet` の `seen`、`bind_visible` の scroll ごとの `DuplicateKey` の報告、`Seeker::run` の全行の走査。

### 状態の閉じ方

- `VState<K, T>`：`heights`・`index_of`・`indexed` は `reindex` だけが書く list の索引で、`measured` の cache と同じ事実を重ねない（`heights` は「計測値か推定値」を index 順に並べた派生の表で、`reindex` と計測の更新が同時に書く）。`VState` の定義の直前に次の 1 行を置く（T の W1b の印があれば置き換える）。
  `// xtask-allow(state): seek and held are the list's two phases; rows, measured, heights, index_of and indexed are caches written only by reindex, the layout observer and apply_visible`
- `HeldState<K>`：`value: Option<K>` が一つ。対象外。

### 試験

- 追加：`crates/kivact-core/src/view/flow/virtual_list/heights.rs` の `prefix_set_and_index_at_match_a_plain_sum`。
- 既存（変えずに通す）：e2e の `viewport`・`layout_scroll`・`flows` の `virtual_list` の試験（`rg -n 'virtual_list' dev/tests/kivact-public-api/tests` で列挙される試験）、`ui_data_table`。

### 確認（tier a）

```
cargo fmt -p kivact-core
cargo check -p kivact-core --all-targets
cargo clippy -p kivact-core --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-core
KIVACT_TEST_MIRROR=off cargo test -p kivact-core --lib heights
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- viewport layout_scroll flows
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_data_table
```

### 完了条件

- `rg -n 'enum Held|Held::|HashSet::new\(\)' crates/kivact-core/src/view/flow/virtual_list.rs` が何も返さない。
- 上の命令がすべて成功する。

## W5 の鎖の runtime の API unit（R20〜R25）の共通の約束

- R20〜R25 は kivact-runtime だけを変える。kivact-runtime の外の呼出し元は R26〜R31 が直す。その間 workspace は compile できない。
- kivact-runtime は dev-dependency に facade の `kivact` を持つので、R20〜R25 の確認は `--lib` だけで行う（`cargo test -p kivact-runtime` は R31 で走らせる）。
- 各 unit の確認（tier a）は次の形にする（unit ごとの追加は各節に書く）。
  ```
  cargo fmt -p kivact-runtime
  cargo check -p kivact-runtime --lib
  cargo clippy -p kivact-runtime --lib -- -D warnings
  cargo xtask check --no-write --crate kivact-runtime
  ```

## R20：`Binding` を `Signal` に統合し、`update` は値を返さない

- **wave / lane / 前提**：W5a / 鎖 / R09。
- **所有**：runtime `src/reactive/{data.rs,handles.rs,setup.rs,props.rs,sealed.rs,graph.rs,trigger.rs}`、`src/reactive.rs`、`src/lib.rs`、`src/motion/**` と `src/task/**` のうち `Signal` の field や `update` の戻り値を使う行。
- **目的**：書ける handle を `Signal<T>` 一つにし、内部表現を `SignalRepr { Source, Lens }` にする（API-02、DEC-04）。`Signal::update` と `Var::update` は `()` を返し、破棄後の `set`・`update` は位置付きの `WriteAfterDispose` 診断を一度記録して何もしない（API-15、DEC-15）。`use_context::<T>()` は `T` を返す（RC-31 の runtime の半分）。`Callback::bind`・`Callback::to_fn` を消し、`Callback::call_if_alive` を `#[doc(hidden)]` にする（API-11）。

### 手順

1. `src/reactive/data.rs`：
   - `pub struct Signal<T>` を次にする。
     ```rust
     /// A writable reactive value: a signal (`signal`) or a two-way view of
     /// another value (`binding`, `bind!`, `optional`). Writes change the
     /// value immediately and notify readers at the end of the turn.
     pub struct Signal<T: 'static> {
         pub(crate) app: AppId,
         pub(crate) repr: SignalRepr,
         pub(crate) _m: PhantomData<(fn() -> T, UiThreadOnly)>,
     }

     /// Where a [`Signal`] lives.
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub(crate) enum SignalRepr {
         /// A source node of the graph.
         Source(Key),
         /// A lens: `read` is the value it shows, `write` the `LensCell` object that writes it.
         Lens { read: ValueRepr, write: Key },
     }

     /// How a lens writes: replace the whole value, or edit the current one
     /// in place (both decide themselves whether readers are notified).
     pub(crate) struct LensCell<T> {
         pub(crate) owner: Key,
         pub(crate) set: Box<dyn Fn(T) -> Result<(), UiError>>,
         pub(crate) edit: Box<dyn Fn(&mut dyn FnMut(&mut T)) -> Result<(), UiError>>,
     }
     ```
   - `pub(crate) enum BindingRepr`、`pub struct Binding<T>`、`impl Clone for BindingRepr`、`impl Copy for BindingRepr`、`copy_handle!(Binding<T>);` を削除する。
   - `impl fmt::Debug for Signal<T>` の本体を `match self.repr { SignalRepr::Source(k) => write!(f, "Signal({}:{})", k.slot, k.generation), SignalRepr::Lens { write, .. } => write!(f, "Signal(lens {}:{})", write.slot, write.generation) }` にする。
2. `src/reactive/graph.rs` の `write_node`：戻り値を `bool`（node が生きていれば `true`）にする。破棄された node の枝（`let Some(cell) = cell else { .. record_diagnostic(err); self.request_turn(); return; };`）の中身を `self.write_after_dispose(at); return false;` にし、関数の最後で `true` を返す。`impl Runtime` に次を足す。
   ```rust
   /// A write to a removed signal: ignored, reported once as a located
   /// `WriteAfterDispose` diagnostic.
   pub(crate) fn write_after_dispose(&self, at: &'static Location<'static>) {
       let err = new_error(
           ErrorKind::WriteAfterDispose,
           "write to a disposed signal ignored",
           Some(at),
       );
       self.diag.borrow_mut().record_diagnostic(err);
       // Close a turn record so the note is visible in `trace()`.
       self.request_turn();
   }
   ```
3. `src/reactive/handles.rs` の `impl<T: 'static> Signal<T>` を次の内容に置き換える（`with_impl` の source の枝は今の本体を使う）。
   ```rust
   impl<T: 'static> Signal<T> {
       pub(crate) fn from_key(app: AppId, key: Key) -> Self {
           Signal { app, repr: SignalRepr::Source(key), _m: PhantomData }
       }

       /// The read-only view of this signal (the same node or lens value).
       pub(crate) fn as_value(&self) -> Value<T> {
           match self.repr {
               SignalRepr::Source(key) => Value::node(self.app, key),
               SignalRepr::Lens { read, .. } => Value { app: self.app, repr: read, _m: PhantomData },
           }
       }

       fn with_impl<R>(&self, track: bool, at: &'static Location<'static>, f: impl FnOnce(&T) -> R) -> R {
           let SignalRepr::Source(key) = self.repr else {
               return self.as_value().with_impl(track, at, f);
           };
           // 今の `with_impl` の本体（`self.key` を `key` にする）
       }

       // get, get_untracked, with, with_untracked：今のまま（with_impl を呼ぶ）。

       /// `None` if the signal was disposed (never panics for that reason).
       #[track_caller]
       pub fn try_get(&self) -> Option<T> where T: Clone {
           let SignalRepr::Source(key) = self.repr else {
               return self.as_value().try_get();
           };
           // 今の `try_get` の本体（`self.key` を `key` にする）
       }

       /// Read without subscribing and clone; `None` when the signal or its
       /// App is gone, or while the value is being updated. Never panics.
       pub fn try_get_untracked(&self) -> Option<T> where T: Clone {
           let SignalRepr::Source(key) = self.repr else {
               return crate::reactive::frame::untrack(|| self.as_value().try_get());
           };
           // 今の `try_get_untracked` の本体（`self.key` を `key` にする）
       }

       /// Change a source in place; `f` says whether the value changed.
       /// Returns whether the signal is still there.
       fn write_source(&self, key: Key, at: &'static Location<'static>, f: impl FnOnce(&mut T) -> bool) -> bool {
           let rt = runtime(self.app, at);
           rt.write_node(key, at, |cell| {
               let Some(src) = source_cell::<T>(cell) else { return false };
               let mut guard = match src.value.try_borrow_mut() {
                   Ok(g) => g,
                   Err(_) => misuse_at(ErrorKind::BorrowedWrite, "signal written while borrowed by `with`", at),
               };
               f(&mut guard)
           })
       }

       /// Run `op` on the lens object `write`, for its owner; a removed lens
       /// is reported like a removed signal.
       fn with_lens(&self, write: Key, at: &'static Location<'static>, op: impl FnOnce(&LensCell<T>) -> Result<(), UiError>) {
           let rt = runtime(self.app, at);
           let Some(lens) = rt.object::<LensCell<T>>(write) else {
               rt.write_after_dispose(at);
               return;
           };
           let result = {
               let _frame = if crate::reactive::execution::executing_app() == AppId::NONE {
                   rt.handler_frame(Some(lens.owner))
               } else {
                   rt.callback_frame(lens.owner)
               };
               op(&lens)
           };
           if let Err(err) = result {
               rt.route(Some(lens.owner), err);
           }
       }

       /// Set a new value; readers are notified only if it differs. A write
       /// to a removed signal is ignored and reported (`WriteAfterDispose`).
       #[track_caller]
       pub fn set(&self, value: T) where T: PartialEq {
           let at = Location::caller();
           match self.repr {
               SignalRepr::Source(key) => {
                   self.write_source(key, at, move |v| {
                       if *v == value { false } else { *v = value; true }
                   });
               }
               SignalRepr::Lens { write, .. } => self.with_lens(write, at, move |lens| (lens.set)(value)),
           }
       }

       /// Change the value in place with `f`; readers are always notified. A
       /// write to a removed signal is ignored and reported, as `set` does.
       /// To read what `f` computed, read the signal after it (`with`).
       #[track_caller]
       pub fn update(&self, f: impl FnOnce(&mut T)) {
           let at = Location::caller();
           match self.repr {
               SignalRepr::Source(key) => {
                   self.write_source(key, at, move |v| { f(v); true });
               }
               SignalRepr::Lens { write, .. } => {
                   let mut f = Some(f);
                   self.with_lens(write, at, move |lens| {
                       (lens.edit)(&mut |v: &mut T| {
                           if let Some(f) = f.take() { f(v) }
                       })
                   });
               }
           }
       }

       /// Change the value in place; `f` says whether it changed (lenses
       /// write back either way). Used by `project`.
       fn modify(&self, at: &'static Location<'static>, f: impl FnOnce(&mut T) -> bool) {
           match self.repr {
               SignalRepr::Source(key) => {
                   self.write_source(key, at, f);
               }
               SignalRepr::Lens { write, .. } => {
                   let mut f = Some(f);
                   self.with_lens(write, at, move |lens| {
                       (lens.edit)(&mut |v: &mut T| {
                           if let Some(f) = f.take() { f(v); }
                       })
                   });
               }
           }
       }

       /// Notify readers without changing the value (removed with `Trigger`, R23).
       #[track_caller]
       pub fn notify(&self) {
           self.modify(Location::caller(), |_| true);
       }

       /// A two-way view of one field (setup only): what `bind!` expands to.
       #[doc(hidden)]
       #[track_caller]
       pub fn project<U: crate::PropValue>(
           self,
           get_ref: impl Fn(&T) -> &U + 'static,
           get_mut: impl Fn(&mut T) -> &mut U + 'static,
       ) -> Signal<U> {
           let at = Location::caller();
           let rt = runtime(self.app, at);
           let owner = rt.require_setup("Signal::project", at);
           let this = self;
           let memo = rt.create_memo(
               owner,
               Box::new(move |_| this.with(|t| get_ref(t).clone())),
               Some(<U as PartialEq>::eq),
               at,
           );
           let get_mut = Rc::new(get_mut);
           let set_field = Rc::clone(&get_mut);
           let lens: Rc<LensCell<U>> = Rc::new(LensCell {
               owner,
               set: Box::new(move |u: U| {
                   this.modify(at, |t| {
                       let slot = (*set_field)(t);
                       if *slot == u { false } else { *slot = u; true }
                   });
                   Ok(())
               }),
               edit: Box::new(move |f: &mut dyn FnMut(&mut U)| {
                   this.modify(at, |t| { f((*get_mut)(t)); true });
                   Ok(())
               }),
           });
           let write = rt.insert_object(owner, lens, "Signal", None, at);
           Signal { app: self.app, repr: SignalRepr::Lens { read: ValueRepr::Node(memo), write }, _m: PhantomData }
       }
   }
   ```
   `use crate::reactive::data::{..}` の `Binding`、`BindingRepr` を `LensCell`、`SignalRepr` にする。`Value` の `with_impl` は `fn with_impl` のまま（同じ module から呼べる）。
4. `src/reactive/handles.rs`：
   - `impl<T: crate::PropValue> Binding<T> { .. }` の二つの block と `impl<T: crate::PropValue> Signal<T> { pub fn optional(self) -> Binding<Option<T>> { .. } }` を削除し、次を置く。
     ```rust
     impl<T: crate::PropValue> Signal<T> {
         /// This signal as a signal of `Option<T>`, for an input that edits an
         /// optional choice (`select`, `radio_group`): reading yields
         /// `Some(current)`, writing `Some(v)` writes `v`, writing `None` is
         /// ignored (setup only).
         #[track_caller]
         pub fn optional(self) -> Signal<Option<T>> {
             crate::reactive::setup::binding(
                 move || Some(self.get()),
                 move |v: Option<T>| {
                     if let Some(v) = v {
                         self.set(v);
                     }
                 },
             )
         }
     }
     ```
   - `impl<T: 'static> From<Signal<T>> for Value<T>` の本体を `signal.as_value()` にする。
   - `pub trait IntoBinding<T>`、`impl IntoBindingSeal for Signal/Binding`、`impl IntoBinding<T> for Signal/Binding` を削除する。
   - `impl<T: 'static> Var<T>` の `pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R` を `pub fn update(&self, f: impl FnOnce(&mut T))` にし、本体の `f(&mut g)` はそのまま（戻り値を返さない）。doc の「and return what `f` returns」を消す。`Var::set` の本体 `self.update(move |v| *v = value)` はそのまま。
   - `impl<E: 'static> Callback<E>`：`pub fn bind(..)` と `pub fn to_fn(..)` を削除する。`pub fn call_if_alive` の上に `#[doc(hidden)]` を置く。
5. `src/reactive/sealed.rs` の `pub trait IntoBindingSeal {}` を削除する。
6. `src/reactive/setup.rs`：
   - `pub fn binding<T: crate::PropValue, R: HandlerResult>(value: impl IntoValue<T>, set: impl Fn(T) -> R + 'static) -> Binding<T>` を次にする。
     ```rust
     /// A two-way value made of a value and the closure that writes it (a
     /// lens), for joining two separate pieces of state; it is a `Signal`.
     /// The value can be anything [`IntoValue`] takes; `set` is called with
     /// each new value and may return `()` or `Result<(), UiError>`.
     /// `update` reads the value, lets `f` edit a copy and calls `set` once.
     /// Setup only.
     ///
     /// ```ignore
     /// let celsius = signal(20.0);
     /// let fahrenheit = binding(move || celsius.get() * 1.8 + 32.0, move |f| celsius.set((f - 32.0) / 1.8));
     /// ```
     #[track_caller]
     pub fn binding<T: crate::PropValue, R: HandlerResult>(
         value: impl IntoValue<T>,
         set: impl Fn(T) -> R + 'static,
     ) -> Signal<T> {
         let at = Location::caller();
         let read = value.into_value();
         let (rt, owner) = setup_scope("binding", at);
         let set: Rc<dyn Fn(T) -> Result<(), crate::UiError>> =
             Rc::new(move |v| set(v).into_outcome().map(|_| ()));
         let write_whole = Rc::clone(&set);
         let lens: Rc<LensCell<T>> = Rc::new(LensCell {
             owner,
             set: Box::new(move |v| write_whole(v)),
             edit: Box::new(move |f| {
                 let mut v = read.get_untracked();
                 f(&mut v);
                 set(v)
             }),
         });
         let write = rt.insert_object(owner, lens, "Signal", None, at);
         Signal { app: rt.id(), repr: SignalRepr::Lens { read: read.repr, write }, _m: std::marker::PhantomData }
     }
     ```
     `use crate::reactive::data::{..}` の `Binding`、`BindingRepr` を `LensCell`、`SignalRepr` にする。
   - `use_context` と `try_use_context` を次にし、doc を書き換える。
     ```rust
     /// The nearest `T` provided above the current owner (Solid
     /// `useContext`), cloned. Provide a `Signal` or a `Value` (or a struct of
     /// them) when readers must see changes. Works in setup and in handlers,
     /// tasks and effects. Panics at the caller, naming `T`, if none is
     /// provided; use [`try_use_context`] when the context is optional.
     #[track_caller]
     pub fn use_context<T: Clone + 'static>() -> T {
         let at = Location::caller();
         let (rt, owner) = owner_scope("use_context", at);
         lookup_context::<T>(&rt, owner).unwrap_or_else(|| {
             misuse_at(
                 ErrorKind::Message,
                 format!("`use_context::<{}>()` requires context; wrap this component in `provide(value, || view)`", type_name::<T>()),
                 at,
             )
         })
     }

     /// The nearest `T` provided above the current owner, or `None`.
     #[track_caller]
     pub fn try_use_context<T: Clone + 'static>() -> Option<T> {
         let at = Location::caller();
         let (rt, owner) = owner_scope("try_use_context", at);
         lookup_context::<T>(&rt, owner)
     }
     ```
7. `src/reactive/props.rs`：
   - `impl<T: 'static> IntoValue<T> for Signal<T>` の本体を `Prop(PropRepr::Value(self.as_value()))` にする。
   - `impl<T: Clone + PartialEq + 'static> IntoValue<T> for Binding<T>`、`impl IntoReactive<T> for Binding<T>`、`impl IntoTextSeal for Binding<String>`、`impl IntoText for Binding<String>` を削除する。
   - doc comment の `Binding` の語（`a Signal, Value, Binding, ..`）を消す（`Signal` が lens も含む）。
8. `src/reactive.rs` と `src/lib.rs` の `pub use` から `Binding`、`IntoBinding` を消す。
9. `cargo check -p kivact-runtime --lib` の誤りを次の種類だけで直す。
   - `no field key on type Signal`：`trigger.rs` の `Signal::<()>::from_key(self.app, self.key)` はそのまま動く（`from_key` がある）。それ以外の `signal.key` は、読みなら `signal.as_value()`、source の key が要るなら `let SignalRepr::Source(key) = signal.repr else { unreachable!("invariant: <その場所の型> holds a source signal") };`。`owner.signal(..)` で作った signal だけがそこに来る場所に限る。それ以外の場所なら STOP。
   - `.update(..)` の戻り値を使う runtime の行：`update` の後に `with_untracked`／`get_untracked` で読む形にする。

### 消すもの

- `Binding`、`BindingRepr`、`IntoBinding`、`IntoBindingSeal`、`Binding::project`、`Binding::optional`、`Callback::bind`、`Callback::to_fn`、`Signal::update` と `Var::update` の戻り値、`Signal::update` の破棄時の panic。

### 状態の閉じ方

- `LensCell<T>`：closure の box 二つと owner。対象外。
- `SignalRepr`：Copy の enum。対象外。

### 試験

- runtime の unit 試験はこの unit では走らせない（共通の約束）。e2e の書換えは R31（4 章 4.6 の表）。

### 確認（tier a）

共通の形。

### 完了条件

- `rg -n '\bBinding\b|IntoBinding|BindingRepr|fn bind\(|fn to_fn' crates/kivact-runtime/src` が何も返さない。
- `rg -n 'pub fn update' crates/kivact-runtime/src/reactive/handles.rs` の 2 行（Signal と Var）がどちらも戻り値の型を持たない。

## R21：`IntoHandler` と `Handler`、`Callback` の格納

- **wave / lane / 前提**：W5a / 鎖 / R20。
- **所有**：runtime `src/reactive/{handler.rs,handles.rs,data.rs,setup.rs,sealed.rs}`、`src/reactive.rs`、`src/lib.rs`、`src/motion/{animated.rs,timeline/api.rs}`、`dev/xtask/src/api_shape.rs`。
- **目的**：payload の無い handler の入力を `IntoHandler<(), M>`（`Fn() -> R` と `Callback<()>`）にし、payload のある handler は `impl Fn(E) -> R` のまま受ける（API-03、決定 DEC-16）。`From<F> for Callback<()>` を消す。`Callback` は `Handler<E>` を持ち、`Callback::from_handler` で arena の object を一つだけ作る（RC-33 の runtime の部分）。

### 手順

1. `src/reactive/sealed.rs` に次を足す。
   ```rust
   /// Seal of `IntoHandler`: nullary closures and `Callback<()>` only (DEC-16).
   pub trait IntoHandlerSeal<E, M> {}
   ```
2. `src/reactive/handler.rs`：
   - `pub struct Handler<E>(Rc<dyn Fn(E) -> Result<Handled, UiError>>);` の doc を `/// An erased event handler: what every \`on_*\` method and \`Callback\` holds.` にし、`Handler::from_fn` に `#[doc(hidden)]` を付ける（kivact-core の内部が使う）。
   - 次を足す。
     ```rust
     /// What an `on_*` method without a payload takes (DEC-16): a closure
     /// `move || ..` returning `()`, `Handled` or a `Result` of them, or a
     /// shared [`Callback<()>`](crate::Callback). `M` only tells the two apart;
     /// it is inferred. Handlers with a payload take `impl Fn(E) -> R`.
     #[diagnostic::on_unimplemented(
         message = "`{Self}` cannot handle a `{E}` event",
         note = "pass `move || ..` (returning `()`, `Handled` or a `Result`), or a `Callback<()>`"
     )]
     pub trait IntoHandler<E, M>: crate::reactive::sealed::IntoHandlerSeal<E, M> {
         /// This handler, erased.
         fn into_handler(self) -> Handler<E>;
     }

     /// `IntoHandler` marker: a closure without the event.
     #[doc(hidden)]
     pub struct Nullary;
     /// `IntoHandler` marker: a `Callback`.
     #[doc(hidden)]
     pub struct Handle;

     impl<F, R> crate::reactive::sealed::IntoHandlerSeal<(), Nullary> for F
     where F: Fn() -> R + 'static, R: HandlerResult {}
     impl<F, R> IntoHandler<(), Nullary> for F
     where F: Fn() -> R + 'static, R: HandlerResult
     {
         fn into_handler(self) -> Handler<()> {
             Handler(Rc::new(move |()| self().into_outcome()))
         }
     }

     impl crate::reactive::sealed::IntoHandlerSeal<(), Handle> for crate::Callback<()> {}
     impl IntoHandler<(), Handle> for crate::Callback<()> {
         fn into_handler(self) -> Handler<()> {
             Handler(Rc::new(move |()| self.try_call(()).map(|()| Handled::No)))
         }
     }
     ```
3. `src/reactive/data.rs` の `pub(crate) struct CallbackFn<E> { owner: Key, f: Box<dyn Fn(E) -> Result<(), UiError>> }` の `f` を `pub(crate) f: crate::reactive::Handler<E>,` にする。
4. `src/reactive/handles.rs`：
   - `Callback::invoke` の `(cb.f)(e)` を `cb.f.call(e).map(|_| ())` にする。
   - `impl<F, R> From<F> for Callback<()>` を削除し、`impl<E: 'static> Callback<E>` に次を足す。
     ```rust
     /// A `Callback` of the running owner calling `handler` (one arena
     /// object; the handler is not boxed again). A `#[component]` turns its
     /// `Callback<E>` props into callbacks with it.
     #[doc(hidden)]
     #[track_caller]
     pub fn from_handler(handler: crate::reactive::Handler<E>) -> Callback<E> {
         let at = Location::caller();
         let (rt, owner) = super::scope::owner_scope("Callback::from_handler", at);
         let cb: Rc<CallbackFn<E>> = Rc::new(CallbackFn { owner, f: handler });
         Self::from_key(rt.id(), rt.insert_object(owner, cb, "Callback", None, at))
     }
     ```
5. `src/reactive/setup.rs` の `callback` の `f: Box::new(move |e| f(e).into_outcome().map(|_| ())),` を `f: crate::reactive::Handler::from_fn(f),` にする。
6. `src/reactive.rs`：`pub use handler::{Handled, Handler, HandlerResult};` を `pub use handler::{Handled, Handler, HandlerResult, IntoHandler};` にし、`src/lib.rs` の `pub use reactive::{..}` に `IntoHandler` を足す。
7. runtime の中の `impl Into<Callback<()>>`（3 か所）を `IntoHandler` にする。
   - `src/motion/animated.rs` の `pub fn on_complete(self, f: impl Into<crate::Callback<()>>) -> Self` を `pub fn on_complete<M>(self, f: impl crate::IntoHandler<(), M>) -> Self` にし、本体の `f.into()` を `crate::Callback::from_handler(f.into_handler())` にする。
   - `src/motion/timeline/api.rs` の `pub fn call(&self, at: At, f: impl Into<Callback<()>>) -> Segment` と `pub fn on_complete(self, f: impl Into<Callback<()>>) -> Timeline` に generic `<M>` を足し、引数を `f: impl crate::IntoHandler<(), M>` にし、本体の `f.into()` を `Callback::from_handler(f.into_handler())` にする。
   - `use crate::IntoHandler;` が要るファイルには足す（`into_handler` の method の解決のため）。
8. `dev/xtask/src/api_shape.rs`：
   - `fn into_unit_callback(ty: &syn::Type) -> bool` を次の関数に置き換え、`handler_callback` の本体を `into_handler(ty) || fn_output_handler(ty, sig)` にする。`callback_unit` が使われなくなったら消す。
     ```rust
     /// `impl IntoHandler<(), M>`, also with a qualified path or `+ 'static`.
     fn into_handler(ty: &syn::Type) -> bool {
         let syn::Type::ImplTrait(ty) = ty else {
             return false;
         };
         ty.bounds.iter().any(|bound| {
             let syn::TypeParamBound::Trait(bound) = bound else {
                 return false;
             };
             bound.path.segments.last().is_some_and(|s| s.ident == "IntoHandler")
         })
     }
     ```
   - 二つの誤りの文面の受け入れの形を「`impl IntoHandler<(), M>` か `impl Fn(E) -> R`」にする（`"on_ method requires a last argument of impl IntoHandler<(), M> or impl Fn(E) -> R with R: HandlerResult"`、`"a builder method takes a handler without a payload as impl IntoHandler<(), M> (API-3)"`）。
   - 同じファイルの `mod tests` の試験の入力の文字列のうち、受け入れる形として `impl Into<Callback<()>>` を書いたもの（`pub fn on_press(self, f: impl Into<Callback<()>>)` など）を `pub fn on_press<M>(self, f: impl IntoHandler<(), M>)` にする。期待する判定（error の有無と数）は変えない。

### 消すもの

- `impl<F, R> From<F> for Callback<()>`、`CallbackFn` の二重の box。

### 状態の閉じ方

- 新しい型は marker の struct と trait だけ。対象外。

### 試験

- 追加：e2e の `component_macro` module に R31 が `callback_prop_closure_infers_its_parameter` を足す（DEC-16 の compile 試験：payload のある `Callback<E>` の props は `impl Fn(E) -> R` を受け、closure の引数の型が推論される）。
- `cargo test -p xtask api_shape`を通す。

### 確認（tier a）

共通の形に次を足す。
```
cargo test -p xtask api_shape
```

### 完了条件

- `rg -n 'From<F> for Callback|Into<[\w:]*Callback<\(\)>>' crates/kivact-runtime/src dev/xtask/src` が何も返さない（xtask の試験の文字列で、拒否される形の例として残すものを除く）。

## R22：非同期の結果を `Result` に、Action を一つの状態と一つの drain にする

- **wave / lane / 前提**：W5a / 鎖 / R21。
- **所有**：runtime `src/task/**`、`src/task.rs`、`src/host.rs`、`src/lib.rs`。
- **目的**：`ActionOutcome` を消して `on_complete(Fn(Result<T, UiError>))` にし、`Retry` を `Callback<()>` にする（RC-35）。`action(f)` は Latest、順に実行するものは `queued_action(f)`、`ActionMode` を消す（設計書 5.5、Q4）。非同期の状態を一つの signal（`AsyncCellState`）にし、`AsyncSnapshot` を消す（RC-15）。Action の受付の状態を一つの `lifecycle` にする（RC-16）。`Action::close`・`CloseFuture`・drain の期限の仕組みを消し、所有者が破棄された queued action が終わるのを App の終了が一つの期限で待つだけにする（RC-37）。`ActionFn`・`BlockingWork` を `#[doc(hidden)]` にする。

### 手順

1. `src/task/async_state.rs`：`AsyncState<T>` とその method、`cancelled` はそのまま。`pub(crate) struct AsyncCell<T>` とその `impl` を削除し、次を置く。
   ```rust
   /// The run an action's state describes: its input and attempt number.
   pub(crate) struct RunInfo<I> {
       pub(crate) input: Rc<I>,
       pub(crate) attempt: u32,
   }

   impl<I> Clone for RunInfo<I> {
       fn clone(&self) -> Self {
           RunInfo { input: Rc::clone(&self.input), attempt: self.attempt }
       }
   }

   /// Where a resource or an action stands; a `Ready` value is `latest`.
   #[derive(Clone, Debug, PartialEq)]
   pub(crate) enum AsyncPhase {
       Idle,
       Pending,
       Ready,
       Failed(UiError),
   }

   // xtask-allow(state): phase is the only phase; latest is the last success (kept while Pending and Failed), run is the run the phase describes, waiting counts queued runs and publications counts writes; each is valid in every phase
   pub(crate) struct AsyncCellState<T, I> {
       pub(crate) phase: AsyncPhase,
       pub(crate) latest: Option<Rc<T>>,
       pub(crate) run: Option<RunInfo<I>>,
       pub(crate) waiting: usize,
       pub(crate) publications: u64,
   }

   /// The only publication path for resource and action state: one signal.
   pub(crate) struct AsyncCell<T: 'static, I: 'static> {
       state: crate::Signal<AsyncCellState<T, I>>,
   }

   impl<T: Clone + 'static, I: 'static> AsyncCell<T, I> {
       pub(super) fn new(owner: crate::Owner, at: &'static Location<'static>) -> Self {
           let initial = AsyncCellState { phase: AsyncPhase::Idle, latest: None, run: None, waiting: 0, publications: 0 };
           Self { state: owner.signal(initial, at) }
       }

       fn publish(&self, f: impl FnOnce(&mut AsyncCellState<T, I>)) {
           self.state.update(|s| {
               f(s);
               s.publications += 1;
           });
       }

       /// A run starts; `waiting` more wait behind it.
       pub(super) fn begin(&self, run: RunInfo<I>, waiting: usize) {
           self.publish(|s| {
               s.phase = AsyncPhase::Pending;
               s.run = Some(run);
               s.waiting = waiting;
           });
       }

       /// Work is pending (a resource fetch, or runs queued behind one).
       pub(super) fn pending(&self, waiting: usize) {
           self.publish(|s| {
               s.phase = AsyncPhase::Pending;
               s.waiting = waiting;
           });
       }

       /// A run ended with `result`; `waiting` runs still wait.
       pub(super) fn finish(&self, result: Result<T, UiError>, waiting: usize) {
           self.publish(|s| {
               s.waiting = waiting;
               match result {
                   Ok(value) => {
                       s.latest = Some(Rc::new(value));
                       s.phase = AsyncPhase::Ready;
                   }
                   Err(error) => s.phase = AsyncPhase::Failed(error),
               }
           });
       }

       /// The public state (tracked).
       pub(super) fn state(&self) -> AsyncState<T> {
           self.state.with(|s| match &s.phase {
               AsyncPhase::Idle => AsyncState::Idle,
               AsyncPhase::Pending => AsyncState::Pending,
               AsyncPhase::Ready => AsyncState::Ready(T::clone(
                   s.latest.as_deref().expect("invariant: a Ready state keeps its value in latest"),
               )),
               AsyncPhase::Failed(e) => AsyncState::Failed(e.clone()),
           })
       }

       pub(super) fn latest(&self) -> Option<T> {
           self.state.with(|s| s.latest.as_deref().cloned())
       }

       pub(super) fn error(&self) -> Option<UiError> {
           self.state.with(|s| match &s.phase {
               AsyncPhase::Failed(e) => Some(e.clone()),
               _ => None,
           })
       }

       pub(super) fn attempt(&self) -> u32 {
           self.state.with(|s| s.run.as_ref().map_or(0, |r| r.attempt))
       }

       pub(super) fn waiting(&self) -> usize {
           self.state.with(|s| s.waiting)
       }

       pub(super) fn run_untracked(&self) -> Option<RunInfo<I>> {
           self.state.with_untracked(|s| s.run.clone())
       }

       /// How many times the state was written (tracked): async views
       /// re-render on it without comparing values.
       pub(super) fn publications(&self) -> u64 {
           self.state.with(|s| s.publications)
       }
   }

   impl<T: Clone + 'static, I: Clone + 'static> AsyncCell<T, I> {
       pub(super) fn input(&self) -> Option<I> {
           self.state.with(|s| s.run.as_ref().map(|r| I::clone(&r.input)))
       }
   }
   ```
2. `src/task/async_publication.rs` を削除する。`src/task/async_source.rs` の trait を次にする。
   ```rust
   #[diagnostic::on_unimplemented(
       message = "async_view takes a Resource or Action",
       note = "pass the source handle directly; do not call state()"
   )]
   pub trait AsyncSource<T>: super::sealed::AsyncSourceSeal + Copy + 'static {
       /// The current state (tracked).
       fn state(&self) -> AsyncState<T>;
       /// A callback that runs the source again (refetch, or retry the last run). Setup or handler.
       fn retry(&self) -> crate::Callback<()>;
       /// How many times the state was published (tracked).
       #[doc(hidden)]
       fn publications(&self) -> u64;
   }
   ```
   `src/task/async_adapters.rs` の二つの impl を次にする。
   ```rust
   impl<T: Clone + 'static> AsyncSource<T> for Resource<T> {
       fn state(&self) -> AsyncState<T> { Resource::state(self) }
       fn retry(&self) -> crate::Callback<()> {
           let source = *self;
           crate::Callback::from_handler(crate::Handler::from_fn(move |()| source.refetch()))
       }
       fn publications(&self) -> u64 { Resource::publications(self) }
   }
   impl<T: Clone + 'static, I: 'static> AsyncSource<T> for Action<T, I> {
       fn state(&self) -> AsyncState<T> { Action::state(self) }
       fn retry(&self) -> crate::Callback<()> {
           let source = *self;
           crate::Callback::from_handler(crate::Handler::from_fn(move |()| source.retry()))
       }
       fn publications(&self) -> u64 { Action::publications(self) }
   }
   ```
3. `src/task/action.rs` を次の内容で置き換える（module の doc は今の文から `ActionMode`・`close`・`Closed` の説明を消し、`action` は最新の run が古い run を取り消す、`queued_action` は順に実行する、と書く。`ActionFn` の trait と 4 つの impl は今のものをそのまま残し、trait に `#[doc(hidden)]` を足す）。
   ```rust
   use std::cell::{Cell, RefCell};
   use std::collections::VecDeque;
   use std::future::Future;
   use std::panic::Location;
   use std::pin::Pin;
   use std::rc::Rc;

   use kivact_types::cell::Generation;

   use super::async_state::{AsyncCell, AsyncState, RunInfo, cancelled};
   use super::executor::RawTask;
   use super::latest::LatestRun;
   use super::state::Drain;
   use crate::{AppId, Handler, HandlerResult, Owned, Owner, Runtime, UiError, require_effectful};

   type RunFn<T, I> = Rc<dyn Fn(&I) -> Pin<Box<dyn Future<Output = Result<T, UiError>>>>>;

   struct DoneHandler<T: 'static> {
       id: Generation,
       owner: Owner,
       f: Handler<Result<T, UiError>>,
   }

   enum Runner<I: 'static> {
       /// `action`: a new run cancels the one in flight.
       Latest(Rc<LatestRun>),
       /// `queued_action`: one run at a time, in submission order.
       Queue(RefCell<Queue<I>>),
   }

   enum Worker {
       Stopped,
       Picking { task: RawTask, id: Generation },
       Running { task: RawTask, id: Generation },
   }

   struct Queue<I: 'static> {
       worker: Worker,
       waiting: VecDeque<RunInfo<I>>,
       next_worker_id: Generation,
   }

   /// Whether the action's owner still shows it; a detached queued action
   /// finishes its runs for the exit drain and publishes nothing.
   #[derive(Clone, Copy, PartialEq, Eq)]
   enum Lifecycle {
       Open,
       Detached,
   }

   // xtask-allow(state): lifecycle is the only phase (open, or detached after the owner's disposal); on_complete is a registration list and next_handler_id an id allocator, both valid in either phase
   pub(crate) struct ActionCell<T: 'static, I: 'static> {
       app: AppId,
       owner: Owner,
       run: RunFn<T, I>,
       runner: Runner<I>,
       async_state: AsyncCell<T, I>,
       lifecycle: Cell<Lifecycle>,
       on_complete: RefCell<Vec<DoneHandler<T>>>,
       next_handler_id: Cell<Generation>,
       at: &'static Location<'static>,
   }

   /// An async operation the user starts (`run`), with one public state.
   pub struct Action<T: 'static, I: 'static = ()> {
       handle: Owned<Rc<ActionCell<T, I>>>,
   }

   impl<T: 'static, I: 'static> Clone for Action<T, I> {
       fn clone(&self) -> Self {
           *self
       }
   }
   impl<T: 'static, I: 'static> Copy for Action<T, I> {}

   /// An action whose new run cancels the one in flight (the latest wins).
   /// `run` takes `|| async { .. }` or `|input: I| async move { .. }`
   /// returning `Result<T, UiError>`. Setup only.
   #[track_caller]
   pub fn action<T, M, F>(run: F) -> Action<T, F::Input>
   where
       T: Clone + 'static,
       F: ActionFn<T, M>,
   {
       create("action", Location::caller(), false, move |i: &F::Input| run.start(i))
   }

   /// An action whose runs wait for each other and run in submission order.
   /// When its component is removed, the runs already submitted still
   /// finish (App exit waits for them). Setup only.
   #[track_caller]
   pub fn queued_action<T, M, F>(run: F) -> Action<T, F::Input>
   where
       T: Clone + 'static,
       F: ActionFn<T, M>,
   {
       create("queued_action", Location::caller(), true, move |i: &F::Input| run.start(i))
   }

   // （ここに今の `ActionFn` trait と 4 つの impl をそのまま置く）

   fn create<T, I, Fut>(
       api: &'static str,
       at: &'static Location<'static>,
       queued: bool,
       f: impl Fn(&I) -> Fut + 'static,
   ) -> Action<T, I>
   where
       T: Clone + 'static,
       Fut: Future<Output = Result<T, UiError>> + 'static,
   {
       let owner = Owner::current(api, at);
       let rt = owner.runtime();
       let run: RunFn<T, I> = Rc::new(move |i| Box::pin(crate::untrack(|| f(i))));
       let runner = if queued {
           Runner::Queue(RefCell::new(Queue {
               worker: Worker::Stopped,
               waiting: VecDeque::new(),
               next_worker_id: Generation::new(),
           }))
       } else {
           Runner::Latest(LatestRun::new())
       };
       let cell = Rc::new(ActionCell {
           app: owner.app(),
           owner,
           run,
           runner,
           async_state: AsyncCell::new(owner, at),
           lifecycle: Cell::new(Lifecycle::Open),
           on_complete: RefCell::new(Vec::new()),
           next_handler_id: Cell::new(Generation::new()),
           at,
       });
       if queued {
           // Queued actions are listed for the exit drain.
           let drain: Rc<dyn Drain> = cell.clone();
           rt.tasks().queues.borrow_mut().insert(&drain);
       }
       let weak = Rc::downgrade(&cell);
       let on_dispose = move |rt: &Runtime| {
           let Some(cell) = weak.upgrade() else { return };
           // A `Latest` run is a task of the owner: the same disposal cancels it.
           if !matches!(cell.runner, Runner::Queue(_)) {
               return;
           }
           cell.lifecycle.set(Lifecycle::Detached);
           cell.on_complete.borrow_mut().clear();
           if cell.busy() {
               let drain: Rc<dyn Drain> = cell;
               rt.tasks().draining.borrow_mut().push(drain);
           }
       };
       Action { handle: owner.store_with("Action", cell, on_dispose, at) }
   }

   impl<T: Clone + 'static, I: 'static> Drain for ActionCell<T, I> {
       fn busy(&self) -> bool {
           match &self.runner {
               Runner::Latest(run) => run.is_running(),
               Runner::Queue(q) => {
                   let q = q.borrow();
                   !matches!(q.worker, Worker::Stopped) || !q.waiting.is_empty()
               }
           }
       }

       fn abandon(&self) -> usize {
           self.abandon_at(self.at)
       }
   }

   impl<T: Clone + 'static, I: 'static> ActionCell<T, I> {
       fn is_detached(&self) -> bool {
           self.lifecycle.get() == Lifecycle::Detached
       }

       fn waiting(&self) -> usize {
           match &self.runner {
               Runner::Latest(_) => 0,
               Runner::Queue(q) => q.borrow().waiting.len(),
           }
       }

       fn abandon_at(&self, at: &'static Location<'static>) -> usize {
           let n = match &self.runner {
               Runner::Latest(run) => usize::from(run.cancel()),
               Runner::Queue(q) => {
                   let mut q = q.borrow_mut();
                   let n = q.waiting.len() + usize::from(matches!(q.worker, Worker::Running { .. }));
                   q.waiting.clear();
                   let worker = std::mem::replace(&mut q.worker, Worker::Stopped);
                   drop(q);
                   drop(worker);
                   n
               }
           };
           if n > 0 && !self.is_detached() {
               self.async_state.finish(Err(cancelled(at)), 0);
           }
           n
       }

       fn begin_run(&self, run: &RunInfo<I>) -> Pin<Box<dyn Future<Output = Result<T, UiError>>>> {
           self.async_state.begin(run.clone(), self.waiting());
           (self.run)(&run.input)
       }

       fn finish(&self, result: Result<T, UiError>) {
           if self.is_detached() {
               if let (Err(e), Some(rt)) = (&result, Runtime::get(self.app)) {
                   rt.report(None, e.clone());
               }
               return;
           }
           let waiting = self.waiting();
           self.async_state.finish(result.clone(), waiting);
           if matches!(self.runner, Runner::Queue(_)) && self.busy() {
               self.async_state.pending(waiting);
           }
           let handlers: Vec<(Owner, Handler<Result<T, UiError>>)> =
               self.on_complete.borrow().iter().map(|h| (h.owner, h.f.clone())).collect();
           if Runtime::get(self.app).is_none() {
               return;
           }
           for (owner, f) in handlers {
               owner.run_handler(|| {
                   if let Err(err) = f.call(result.clone()) {
                       owner.report(err);
                   }
               });
           }
       }

       /// A detached queued action stopped: the exit drain forgets it.
       fn stopped(&self) {
           if self.is_detached()
               && let Some(rt) = Runtime::get(self.app)
           {
               rt.tasks().forget_drained();
           }
       }

       fn start_latest(self: &Rc<Self>, rt: &Runtime, run: &Rc<LatestRun>, info: RunInfo<I>) {
           let weak = Rc::downgrade(self);
           run.start(rt, self.owner, self.at, || self.begin_run(&info), move |result| {
               let Some(cell) = weak.upgrade() else { return };
               cell.finish(result);
           });
       }

       fn enqueue(self: &Rc<Self>, rt: &Runtime, info: RunInfo<I>) {
           let Runner::Queue(q) = &self.runner else {
               unreachable!("invariant: only a queued action enqueues")
           };
           q.borrow_mut().waiting.push_back(info);
           self.async_state.pending(self.waiting());
           self.ensure_worker(rt);
       }

       /// The next run for worker `id` (it becomes `Running`), or `None` when
       /// `id` is no longer the worker or nothing waits (the worker stops).
       fn pick(&self, id: Generation) -> Option<RunInfo<I>> {
           let Runner::Queue(q) = &self.runner else { return None };
           let mut q = q.borrow_mut();
           let current = match &q.worker {
               Worker::Picking { id: w, .. } | Worker::Running { id: w, .. } => *w == id,
               Worker::Stopped => false,
           };
           if !current {
               return None;
           }
           let Some(info) = q.waiting.pop_front() else {
               // The worker ends: its own task handle goes.
               let worker = std::mem::replace(&mut q.worker, Worker::Stopped);
               drop(q);
               drop(worker);
               self.stopped();
               return None;
           };
           q.worker = match std::mem::replace(&mut q.worker, Worker::Stopped) {
               Worker::Picking { task, id } | Worker::Running { task, id } => Worker::Running { task, id },
               Worker::Stopped => unreachable!("invariant: checked above"),
           };
           Some(info)
       }

       /// After a run of worker `id`: `None` when `id` is no longer the
       /// worker, else whether more runs wait (if not, the worker stops).
       fn after_run(&self, id: Generation) -> Option<bool> {
           let Runner::Queue(q) = &self.runner else { return None };
           let mut q = q.borrow_mut();
           let Worker::Running { id: w, .. } = &q.worker else { return None };
           if *w != id {
               return None;
           }
           if q.waiting.is_empty() {
               let worker = std::mem::replace(&mut q.worker, Worker::Stopped);
               drop(q);
               drop(worker);
               return Some(false);
           }
           q.worker = match std::mem::replace(&mut q.worker, Worker::Stopped) {
               Worker::Running { task, id } | Worker::Picking { task, id } => Worker::Picking { task, id },
               Worker::Stopped => unreachable!("invariant: checked above"),
           };
           Some(true)
       }

       fn ensure_worker(self: &Rc<Self>, rt: &Runtime) {
           let Runner::Queue(q) = &self.runner else { return };
           let id = {
               let mut q = q.borrow_mut();
               if !matches!(q.worker, Worker::Stopped) {
                   return;
               }
               q.next_worker_id.next()
           };
           let weak = Rc::downgrade(self);
           let task = rt.tasks().exec.spawn(Some(self.owner.id()), async move {
               loop {
                   let fut = {
                       let Some(cell) = weak.upgrade() else { return };
                       let Some(info) = cell.pick(id) else { return };
                       cell.begin_run(&info)
                   };
                   let result = fut.await;
                   let Some(cell) = weak.upgrade() else { return };
                   let Some(more) = cell.after_run(id) else { return };
                   cell.finish(result);
                   if !more {
                       cell.stopped();
                       return;
                   }
               }
           });
           q.borrow_mut().worker = Worker::Picking { task, id };
           rt.request_turn();
       }

       fn submit(self: &Rc<Self>, rt: &Runtime, info: RunInfo<I>) {
           match &self.runner {
               Runner::Latest(run) => self.start_latest(rt, run, info),
               Runner::Queue(_) => self.enqueue(rt, info),
           }
       }
   }

   impl<T: Clone + 'static, I: 'static> Action<T, I> {
       fn cell(&self, at: &'static Location<'static>) -> (Rc<Runtime>, Rc<ActionCell<T, I>>) {
           let c = self.handle.get_at(at);
           (c.owner.runtime(), c)
       }

       #[track_caller]
       fn submit(&self, input: Rc<I>, api: &'static str) {
           let at = Location::caller();
           let (rt, cell) = self.cell(at);
           require_effectful(api);
           cell.submit(&rt, RunInfo { input, attempt: 1 });
       }

       /// The state of the run the action describes (tracked).
       #[track_caller]
       pub fn state(&self) -> AsyncState<T> {
           self.cell(Location::caller()).1.async_state.state()
       }

       /// The last successful value, kept while a new run is pending or failed.
       #[track_caller]
       pub fn latest(&self) -> Option<T> {
           self.cell(Location::caller()).1.async_state.latest()
       }

       /// The error of the last run, if it failed.
       #[track_caller]
       pub fn error(&self) -> Option<UiError> {
           self.cell(Location::caller()).1.async_state.error()
       }

       /// The attempt number of the run `state` describes (0 before the first).
       #[track_caller]
       pub fn attempt(&self) -> u32 {
           self.cell(Location::caller()).1.async_state.attempt()
       }

       /// Runs waiting behind the running one (`queued_action`; 0 otherwise).
       #[track_caller]
       pub fn pending_len(&self) -> usize {
           self.cell(Location::caller()).1.async_state.waiting()
       }

       /// Run the last input again, as the next attempt.
       #[track_caller]
       pub fn retry(&self) {
           let at = Location::caller();
           let (rt, cell) = self.cell(at);
           require_effectful("Action::retry");
           if let Some(last) = cell.async_state.run_untracked() {
               cell.submit(&rt, RunInfo { input: last.input, attempt: last.attempt + 1 });
           }
       }

       /// Cancel the running run and the waiting ones; the state becomes
       /// `Failed` with a cancellation error and `on_complete` is not called.
       #[track_caller]
       pub fn cancel(&self) {
           let at = Location::caller();
           let (_, cell) = self.cell(at);
           // Checked before any state changes.
           require_effectful("Action::cancel");
           if cell.busy() {
               cell.abandon_at(at);
           }
       }

       /// Call `f` with the result of each run that ends (not for a
       /// cancellation). Several handlers run in registration order. Setup only.
       #[track_caller]
       pub fn on_complete<R: HandlerResult>(self, f: impl Fn(Result<T, UiError>) -> R + 'static) -> Self {
           let at = Location::caller();
           let (rt, cell) = self.cell(at);
           let owner = Owner::current("Action::on_complete", at);
           let mut generation = cell.next_handler_id.get();
           let id = generation.next();
           cell.next_handler_id.set(generation);
           cell.on_complete.borrow_mut().push(DoneHandler { id, owner, f: Handler::from_fn(f) });
           let weak = Rc::downgrade(&cell);
           drop(rt);
           owner.on_cleanup(move || {
               if let Some(c) = weak.upgrade() {
                   c.on_complete.borrow_mut().retain(|h| h.id != id);
               }
           });
           self
       }

       pub(super) fn publications(&self) -> u64 {
           self.cell(Location::caller()).1.async_state.publications()
       }
   }

   impl<T: Clone + 'static, I: Clone + 'static> Action<T, I> {
       /// Start a run with `input` (`()` for an action without input).
       #[track_caller]
       pub fn run(&self, input: I) {
           self.submit(Rc::new(input), "Action::run");
       }

       /// The input of the run `state` describes.
       #[track_caller]
       pub fn input(&self) -> Option<I> {
           self.cell(Location::caller()).1.async_state.input()
       }
   }
   ```
   `Generation::next(&mut self)` は進めた値を返す（今の `action.rs` の使い方と同じ）。`q.next_worker_id.next()` は `q` を `borrow_mut` したまま呼ぶので、`let mut q` で受ける。
4. `src/task/resource.rs`：
   - `async_state: AsyncCell<T>,` を `async_state: AsyncCell<T, ()>,` にする。
   - effect の中の `cell.async_state.start();` を `cell.async_state.pending(0);`、`cell.async_state.finish(r);` を `cell.async_state.finish(r, 0);` にする。
   - `state` の本体を `self.cell(Location::caller()).async_state.state()`、`latest` を `.async_state.latest()`、`error` を `.async_state.error()` にする。`snapshot` を削除し、`pub(super) fn publications(&self) -> u64 { self.cell(Location::caller()).async_state.publications() }` を足す。`cancel` の `cell.async_state.finish(Err(err));` を `cell.async_state.finish(Err(err), 0);` にする。
5. `src/task/state.rs`：
   - `pub(super) draining: RefCell<Vec<Rc<DrainEntry>>>,` を `pub(super) draining: RefCell<Vec<Rc<dyn Drain>>>,` にし、doc を `/// Queued actions of disposed owners that still run (App exit waits for them).` にする。`struct DrainEntry` とその `impl` を削除する。
   - `pub fn check_draining(&self, rt: &Runtime)` を削除し、次を足す。
     ```rust
     /// Forget the drained actions that have stopped.
     pub(crate) fn forget_drained(&self) {
         self.draining.borrow_mut().retain(|d| d.busy());
     }
     ```
   - `next_deadline` の本体を `self.time.borrow_mut().next_deadline()` にする。
   - `queues_busy` の `self.draining.borrow().iter().any(|entry| entry.action.busy())` を `self.draining.borrow().iter().any(|d| d.busy())` にし、`abandon_all_queues` の `.map(|entry| entry.action.clone())` を消す（`drain(..)` の要素がそのまま `Rc<dyn Drain>`）。
   - `TaskState` に field `pub(super) clocks: RefCell<Vec<Rc<super::clock::ClockEntry>>>,` を足すのは R23（この unit では足さない）。
6. `src/task/drain.rs` を削除し、`src/task.rs` の `mod drain;` と `pub use drain::DEFAULT_DRAIN_NS;` を消す。`src/host.rs` の `DEFAULT_DRAIN_NS` を、ファイルの先頭に置く次の定数に置き換える（`request_exit` の中の名前も `EXIT_DRAIN_NS` にする）。
   ```rust
   /// How long App exit waits for queued actions to finish (ns).
   const EXIT_DRAIN_NS: u64 = 3_000_000_000;
   ```
   `src/host.rs` の `TurnGuard::check_draining` を削除する。
7. `src/task/blocking.rs` の `pub trait BlockingWork` に `#[doc(hidden)]` を付ける。
8. `src/task.rs` の export を次にする（`AsyncSnapshot`・`Retry`・`ActionMode`・`ActionOutcome` とその `xtask-allow` の行を消す）。module の doc の「four async entries」の段落を、`resource`・`action`・`queued_action`・`spawn`・`blocking`・`remote`・`clock` の七つと書き直す（`remote`・`clock` は R23 で足す。この unit では五つと書き、R23 が七つに直す）。
   ```rust
   pub use action::{Action, ActionFn, action, queued_action};
   pub use async_source::AsyncSource;
   ```
   `src/lib.rs` の `pub use task::{..}` から `ActionMode`、`ActionOutcome`、`DEFAULT_DRAIN_NS`、`Retry`、`AsyncSnapshot` を消し、`queued_action` を足す。

### 消すもの

- `ActionMode`、`ActionOutcome`、`QueueMode`、`ActionPhase`、`QueueDrain`、`Acceptance`、`ActionCell.last`、`ActionCell.drained`、`Action::close`、`CloseFuture`、`drain.rs`、`DEFAULT_DRAIN_NS`、`DrainEntry`、`TaskState::check_draining`、`TurnGuard::check_draining`、`Retry`、`AsyncSnapshot`、`async_publication.rs`、`AsyncSource::snapshot`、`AsyncCell` の二つの signal、`queue_status`。

### 状態の閉じ方

- `AsyncCellState`・`ActionCell`：手順 1・3 の印の行。
- `Queue<I>`：`worker` は enum、`waiting` は列、`next_worker_id` は id の allocator（counter 一つ）。対象外。

### 試験

- 書換え・削除は R31（4 章 4.6）：`task::queue_orders_coalesces_recovers_and_drains`（`QueueLatest` の半分を消して `queue_orders_recovers_and_drains` に改名）、`task::action_latest_wins_retry_cancel_and_multiple_done_handlers`（`ActionOutcome` の match を消す）、`ActionMode` を使う e2e の試験。
- `rg -n 'fn .*drain|close\(' crates/kivact-runtime/src/task` で `Action::close` の試験が runtime に無いことを確かめる（2b8df5ac で無い）。

### 確認（tier a）

共通の形。

### 完了条件

- `rg -n 'ActionMode|ActionOutcome|CloseFuture|DEFAULT_DRAIN_NS|AsyncSnapshot|\bRetry\b|check_draining|QueueDrain|Acceptance' crates/kivact-runtime/src` が何も返さない。

## R23：`Trigger`、`remote`、`remote_trigger`、`clock`、`FrameLoop::while_present`

- **wave / lane / 前提**：W5a / 鎖 / R22。
- **所有**：runtime `src/reactive/{trigger.rs,setup.rs,owner_handle.rs,handles.rs}`、`src/reactive.rs`、新規 `src/task/remote.rs`、新規 `src/task/clock.rs`、`src/task/{resource.rs,state.rs,spawn.rs}`、`src/task.rs`、`src/motion/{frame_loop.rs,timeline/api.rs}`、`src/lib.rs`。
- **目的**：`Notifier`・`notifier()` を `Trigger`・`trigger()` に改名し（USR-03）、`Signal::notify` を消す（設計書 5.2）。`notify_from_render` は core が使い続けるので `Trigger::notify_from_render` として残す（付属書 C の R-REQ-6）。他の thread から UI に値と通知を届ける `task::remote`・`task::remote_trigger`（USR-02、CAP-05）と、周期で進む共有の時計 `task::clock`（KIT-17）を足す。`FrameLoop::while_present(impl IntoValue<bool>)` を公開し、`with_presence_gate` とその効かない effect を消す（RC-34 の runtime の部分）。

### 手順

1. 改名（runtime の全ファイル）：
   ```
   git ls-files 'crates/kivact-runtime/*.rs' | xargs perl -pi -e 's/\bNotifier\b/Trigger/g; s/\.notifier\(/.trigger(/g; s/(?<![\w.])notifier\(/trigger(/g; s/\bnotifier\b(?=\s*[,}])/trigger/g'
   ```
   その後 `src/reactive/setup.rs` の `pub fn trigger() -> crate::Trigger` の doc を `/// A dependency without a value (Solid \`createTrigger\`): computations that call \`track\` re-run after \`notify\`. Setup only.` にし、`src/reactive/owner_handle.rs` の `pub fn trigger(self, at) -> Trigger` の doc を `/// A [\`Trigger\`] of this owner, created at \`at\`.` にする。
2. `src/reactive/trigger.rs`：
   - module の doc の `Notifier` を `Trigger` に直す（手順 1 の命令が行う）。`notify_from_render` はそのまま残す。
   - `pub fn notify(self)` の本体 `Signal::<()>::from_key(self.app, self.key).notify();` を次にする。
     ```rust
     let at = Location::caller();
     runtime(self.app, at).write_node(self.key, at, |_| true);
     ```
     使われなくなった import（`Signal`）を消す（`cargo check` の unused の警告に従う）。
3. `src/reactive/handles.rs` の `Signal::notify` を削除する。runtime の中の `.notify()` で receiver が `Signal` のものは無い（`trigger.rs` の 1 か所は手順 2 で消えた）。
4. 新規 `src/task/remote.rs`：
   ```rust
   //! Role: Values and notifications sent to the UI from other threads.
   //!
   //! `remote(initial)` returns a `Signal` read on the UI thread and a
   //! `Remote` any thread may write. A write stores the value in a mailbox and
   //! wakes the App's task that delivers it: the waker schedules that task on
   //! the App's executor, whose `ThreadWaker` asks the host for a turn (the
   //! path a finished `blocking` job takes). Writes made before the task runs
   //! are merged: the signal is written once with the latest value (and the
   //! edits made after it, in order).

   use std::future::Future;
   use std::panic::Location;
   use std::pin::Pin;
   use std::sync::atomic::{AtomicBool, Ordering};
   use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
   use std::task::{Context, Poll, Waker};

   use crate::{Owner, Signal, Trigger};

   /// What other threads sent since the UI took it last.
   enum RemoteMail<T> {
       /// Nothing waits and the delivering task is not parked.
       Idle,
       /// Nothing waits; the delivering task waits for this waker.
       Parked(Waker),
       /// A new value replaces the signal's.
       Value(T),
       /// Edits to apply, in order, to the signal's value.
       Edits(Vec<Box<dyn FnOnce(&mut T) + Send>>),
   }

   fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
       m.lock().unwrap_or_else(PoisonError::into_inner)
   }

   /// The writing half of [`remote`]: `Clone + Send + Sync`, usable from any
   /// thread. Writes never wait for the UI thread.
   pub struct Remote<T> {
       mail: Arc<Mutex<RemoteMail<T>>>,
   }

   impl<T> Clone for Remote<T> {
       fn clone(&self) -> Self {
           Remote { mail: Arc::clone(&self.mail) }
       }
   }

   impl<T: Send + 'static> Remote<T> {
       /// Replace the value; the UI sees the latest one at its next turn.
       pub fn set(&self, value: T) {
           let parked = match std::mem::replace(&mut *lock(&self.mail), RemoteMail::Value(value)) {
               RemoteMail::Parked(waker) => Some(waker),
               _ => None,
           };
           if let Some(waker) = parked {
               waker.wake();
           }
       }

       /// Change the value with `f` at the UI's next turn (after the writes
       /// made before it). `f` runs on this thread when a value set from here
       /// is still waiting, else on the UI thread.
       pub fn update(&self, f: impl FnOnce(&mut T) + Send + 'static) {
           let parked = {
               let mut mail = lock(&self.mail);
               match std::mem::replace(&mut *mail, RemoteMail::Idle) {
                   RemoteMail::Idle => {
                       *mail = RemoteMail::Edits(vec![Box::new(f)]);
                       None
                   }
                   RemoteMail::Parked(waker) => {
                       *mail = RemoteMail::Edits(vec![Box::new(f)]);
                       Some(waker)
                   }
                   RemoteMail::Value(mut value) => {
                       f(&mut value);
                       *mail = RemoteMail::Value(value);
                       None
                   }
                   RemoteMail::Edits(mut edits) => {
                       edits.push(Box::new(f));
                       *mail = RemoteMail::Edits(edits);
                       None
                   }
               }
           };
           if let Some(waker) = parked {
               waker.wake();
           }
       }
   }

   /// The delivering task's wait for mail.
   struct Receive<T> {
       mail: Arc<Mutex<RemoteMail<T>>>,
   }

   impl<T> Future for Receive<T> {
       type Output = RemoteMail<T>;

       fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<RemoteMail<T>> {
           let mut mail = lock(&self.mail);
           match std::mem::replace(&mut *mail, RemoteMail::Idle) {
               RemoteMail::Idle | RemoteMail::Parked(_) => {
                   *mail = RemoteMail::Parked(cx.waker().clone());
                   Poll::Pending
               }
               ready => Poll::Ready(ready),
           }
       }
   }

   /// A signal other threads write through the returned [`Remote`] (setup
   /// only). Writes from other threads wake the event loop and are merged:
   /// each UI turn writes the signal at most once with what arrived since
   /// the last one. Disposing the owner stops the delivery; later writes are
   /// dropped.
   ///
   /// ```ignore
   /// let (progress, sender) = remote(0.0);
   /// std::thread::spawn(move || for p in 0..=100 { sender.set(f64::from(p) / 100.0) });
   /// ```
   #[track_caller]
   pub fn remote<T: Send + 'static>(initial: T) -> (Signal<T>, Remote<T>) {
       let at = Location::caller();
       let owner = Owner::current("remote", at);
       let rt = owner.runtime();
       let signal = owner.signal(initial, at);
       let mail = Arc::new(Mutex::new(RemoteMail::Idle));
       let inbox = Arc::clone(&mail);
       let _task = super::spawn::spawn_owned(
           &rt,
           owner,
           async move {
               loop {
                   match (Receive { mail: Arc::clone(&inbox) }).await {
                       RemoteMail::Value(value) => signal.update(move |slot| *slot = value),
                       RemoteMail::Edits(edits) => signal.update(move |slot| {
                           for edit in edits {
                               edit(slot);
                           }
                       }),
                       RemoteMail::Idle | RemoteMail::Parked(_) => {}
                   }
               }
           },
           at,
       );
       (signal, Remote { mail })
   }

   // xtask-allow(state): notified is the only pending fact; waker is the parked delivering task, taken by the notify that sets notified
   struct TriggerMail {
       notified: AtomicBool,
       waker: Mutex<Option<Waker>>,
   }

   /// The notifying half of [`remote_trigger`]: `Clone + Send + Sync`.
   #[derive(Clone)]
   pub struct RemoteTrigger {
       mail: Arc<TriggerMail>,
   }

   impl RemoteTrigger {
       /// Notify the trigger's readers at the UI's next turn. Never waits for
       /// the UI thread; notifications before that turn are one.
       pub fn notify(&self) {
           if self.mail.notified.swap(true, Ordering::AcqRel) {
               return;
           }
           let parked = lock(&self.mail.waker).take();
           if let Some(waker) = parked {
               waker.wake();
           }
       }
   }

   struct Notified {
       mail: Arc<TriggerMail>,
   }

   impl Future for Notified {
       type Output = ();

       fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
           if self.mail.notified.swap(false, Ordering::AcqRel) {
               return Poll::Ready(());
           }
           *lock(&self.mail.waker) = Some(cx.waker().clone());
           // A notify between the first check and parking took no waker.
           if self.mail.notified.swap(false, Ordering::AcqRel) {
               return Poll::Ready(());
           }
           Poll::Pending
       }
   }

   /// A [`Trigger`] other threads notify through the returned
   /// [`RemoteTrigger`] (setup only), for state the App keeps outside
   /// signals (a buffer a worker fills).
   #[track_caller]
   pub fn remote_trigger() -> (Trigger, RemoteTrigger) {
       let at = Location::caller();
       let owner = Owner::current("remote_trigger", at);
       let rt = owner.runtime();
       let trigger = owner.trigger(at);
       let mail = Arc::new(TriggerMail { notified: AtomicBool::new(false), waker: Mutex::new(None) });
       let inbox = Arc::clone(&mail);
       let _task = super::spawn::spawn_owned(
           &rt,
           owner,
           async move {
               loop {
                   (Notified { mail: Arc::clone(&inbox) }).await;
                   trigger.notify();
               }
           },
           at,
       );
       (trigger, RemoteTrigger { mail })
   }
   ```
5. 新規 `src/task/clock.rs`：
   ```rust
   //! Role: Wall-clock values shared by everything that ticks at one period.

   use std::cell::Cell;
   use std::panic::Location;
   use std::rc::Rc;
   use std::time::{Duration, SystemTime};

   use crate::{Owner, Runtime, Value};

   /// One shared clock: the owner that holds its signal and timer task, and
   /// how many live callers use it.
   pub(crate) struct ClockEntry {
       every_ns: u64,
       owner: Owner,
       value: Value<SystemTime>,
       users: Cell<u32>,
   }

   /// The wall-clock time, updated every `every` (setup only). Calls with the
   /// same period share one timer and one value; it stops when the last
   /// component that asked for it is removed.
   ///
   /// ```ignore
   /// let now = clock(Duration::from_secs(1));
   /// text(move || format_time(now.get()))
   /// ```
   #[track_caller]
   pub fn clock(every: Duration) -> Value<SystemTime> {
       let at = Location::caller();
       let caller = Owner::current("clock", at);
       let rt = caller.runtime();
       let every_ns = u64::try_from(every.as_nanos()).unwrap_or(u64::MAX).max(1);
       let found = rt.tasks().clocks.borrow().iter().find(|c| c.every_ns == every_ns).cloned();
       let entry = match found {
           Some(entry) => entry,
           None => {
               let owner = rt.root().child(at);
               let signal = owner.signal(SystemTime::now(), at);
               let period = Duration::from_nanos(every_ns);
               let _task = super::spawn::spawn_owned(
                   &rt,
                   owner,
                   async move {
                       loop {
                           super::time::sleep(period).await;
                           signal.update(|t| *t = SystemTime::now());
                       }
                   },
                   at,
               );
               let entry = Rc::new(ClockEntry { every_ns, owner, value: signal.into(), users: Cell::new(0) });
               rt.tasks().clocks.borrow_mut().push(Rc::clone(&entry));
               entry
           }
       };
       entry.users.set(entry.users.get() + 1);
       let value = entry.value;
       let app = rt.id();
       caller.on_cleanup(move || {
           let users = entry.users.get() - 1;
           entry.users.set(users);
           if users == 0
               && let Some(rt) = Runtime::get(app)
           {
               rt.tasks().clocks.borrow_mut().retain(|c| !Rc::ptr_eq(c, &entry));
               entry.owner.dispose();
           }
       });
       value
   }
   ```
   `src/task/state.rs` の `TaskState` に field `pub(super) clocks: RefCell<Vec<Rc<super::clock::ClockEntry>>>,` を足し、`TaskState::new` で `clocks: RefCell::new(Vec::new()),` を初期化する。`src/task/spawn.rs` の `pub(super) fn spawn_owned` はそのまま（`remote.rs`・`clock.rs` は同じ `task` module の子なので使える）。
6. `src/task.rs`：`mod clock;`・`mod remote;` を足し、`pub use clock::clock;` と `pub use remote::{Remote, RemoteTrigger, remote, remote_trigger};` を足す。module の doc の async の入口の段落を「`resource`、`action`、`queued_action`、`spawn`、`blocking`、`remote`（と `remote_trigger`）、`clock`」の七つにする。`src/lib.rs` の `pub use task::{..}` に `Remote, RemoteTrigger, clock, remote, remote_trigger` を足し、`pub use reactive::{..}` の `Notifier` と `notifier` が手順 1 で `Trigger`・`trigger` になったことを確かめる。
7. `src/motion/frame_loop.rs`：`pub fn with_presence_gate(self, gate: Value<bool>) -> Self`（`#[doc(hidden)]` を含む）を削除し、次を置く。
   ```rust
   /// Run callbacks only while `present` is true (for example
   /// `move || node.is_present()`), resuming the requested rate when it is
   /// true again; the start/stop state is kept. Setup only.
   #[track_caller]
   pub fn while_present(self, present: impl crate::IntoValue<bool>) -> Self {
       crate::require_setup("FrameLoop::while_present");
       let (_rt, cell) = self.cell(Location::caller());
       cell.gate.set(LoopGate::While(present.into_value()));
       self
   }
   ```

### 消すもの

- `Notifier`、`notifier`、`Owner::notifier`、`Signal::notify`、`FrameLoop::with_presence_gate` と、その中の `request_turn` だけの effect。

### 状態の閉じ方

- `RemoteMail<T>`：enum（4 状態）。対象外。
- `TriggerMail`：手順 4 の印。
- `ClockEntry`：`users: Cell<u32>` だけが内部可変（counter 一つ）。対象外。
- `TaskState`：field の追加。T の W1b の印の文面の最後に `; clocks lists the shared clocks and their users` を足す。

### 試験

- 追加は R31（4 章 4.6）：`task::remote_writes_from_another_thread_coalesce_into_one_update`、`task::remote_trigger_notifies_from_another_thread`、`task::clock_shares_one_timer_per_period`。

### 確認（tier a）

共通の形。

### 完了条件

- `rg -n '\bNotifier\b|\bnotifier\b|with_presence_gate' crates/kivact-runtime/src` が何も返さない。
- `rg -n 'pub fn notify' crates/kivact-runtime/src` が `Trigger::notify` と `RemoteTrigger::notify` だけを返す。

## R24：prop の変換 trait を `IntoValue`・`IntoText`・`IntoHandler` にする

- **wave / lane / 前提**：W5a / 鎖 / R23。
- **所有**：runtime `src/reactive/{props.rs,sealed.rs}`、`src/reactive.rs`、`src/motion/animated.rs`、`src/lib.rs`。
- **目的**：`IntoReactive`（flow は `IntoValue<bool>` を受ける）、`Reactive<T>`（`Prop<T>` を `#[doc(hidden)]` で公開して置き換える）、`IntoValue<Rc<str>>` の impl を消す（RC-32 の変更して採用）。`IntoText` は残し、prelude から外す（prelude は R27）。`constant_value!` を `#[doc(hidden)]` にする（API-06）。

### 手順

1. `src/reactive/props.rs`：
   - `pub struct Prop<T: 'static>(PropRepr<T>);` の上に `#[doc(hidden)]` を置き、doc を「A property input as a component stores it: a constant, a `Value`, or a closure. `#[component]` props and `#[derive(Prop)]` use it; applications pass inputs through `IntoValue`.」にする。
   - `impl<T: 'static> Prop<T>` に次を足す（`Reactive` の method を移す）。
     ```rust
     /// Describe `input` without creating a graph node (no setup needed).
     pub fn new(input: impl IntoValue<T>) -> Self {
         input.into_prop()
     }

     /// The current value, untracked.
     #[track_caller]
     pub fn get_untracked(&self) -> T
     where
         T: Clone,
     {
         crate::reactive::frame::untrack(|| self.get())
     }
     ```
     `impl Prop<String>` に `pub fn from_text(input: impl IntoText) -> Self { input.into_text() }` を足す。
   - `Reactive` の `PartialEq` と `Debug` の impl を `Prop` に移す（`impl<T: PartialEq + 'static> PartialEq for Prop<T>` と `impl<T: std::fmt::Debug + 'static> std::fmt::Debug for Prop<T>`。本体の `self.0.0`・`other.0.0` を `self.0`・`other.0`、文字列の `"Reactive({v:?})"` を `"Prop({v:?})"` にする）。
   - `pub struct Reactive<T>`、その `Clone`・`impl`・`impl Reactive<String>`・`PartialEq`・`Debug`、`impl IntoValue<T> for Reactive<T>`、`impl IntoTextSeal for Reactive<String>`、`impl IntoText for Reactive<String>` を削除する。
   - `pub trait IntoReactive<T>` と 4 つの impl（`Value`、`Signal`、`F`、と R20 で消えた `Binding`）を削除する。
   - `impl IntoValue<Rc<str>> for &str`、`impl IntoValue<Rc<str>> for String`、`constant_value!(Rc<str>);` とその上の comment を削除する。
   - `macro_rules! constant_value` の `#[macro_export]` の上に `#[doc(hidden)]` を置き、doc を「Implements `IntoValue<T> for T` for each listed type. Hidden: the crates below the facade use it for their own value types; application and kit types use `#[derive(Prop)]`.」にする。
   - 冒頭の module の doc と `IntoValue`・`IntoText` の doc から `Binding`、`Reactive`、`IntoReactive` の語を消す。
2. `src/motion/animated.rs` の `impl<T: Animatable> IntoReactive<T> for Animated<T> {}` を削除し、import の `IntoReactive` を消す。
3. `src/reactive.rs` の `pub use props::{IntoReactive, IntoText, IntoValue, ItemKey, Prop, PropValue, Reactive};` を `pub use props::{IntoText, IntoValue, ItemKey, Prop, PropValue};` にし、`src/lib.rs` の同じ名前を消す。

### 消すもの

- `IntoReactive`、`Reactive`、`IntoValue<Rc<str>>` の 3 つの impl。

### 状態の閉じ方

- 状態の型の変更は無い。

### 試験

- R31 で e2e の `reactive_props`・`flows`・`component_macro` を通す。

### 確認（tier a）

共通の形。

### 完了条件

- `rg -n 'IntoReactive|\bReactive\b|IntoValue<Rc<str>>|constant_value!\(Rc<str>\)' crates/kivact-runtime/src` が何も返さない。

## R25：host の driver の名前を `driver` にまとめ、記録の型を一つにし、`Owner` の二つの入口を改名する

- **wave / lane / 前提**：W5a / 鎖 / R24。
- **所有**：runtime 全体（`src/lib.rs`、新規 `src/driver.rs`、`src/reactive/{diag.rs,detail.rs,data.rs,graph.rs,host.rs,owner_handle.rs,scope.rs,reactive.rs}`、`src/host.rs`、`src/task/spawn.rs`、`src/task.rs`、`Owner::current`・`Owner::ambient` を呼ぶすべての runtime のファイル）。
- **目的**：
  - core↔runtime の turn の契約（`Runtime`、`TurnGuard`、`Stage`、`ExitState`、`TurnReport`、`Host`、`ThreadWaker`、`RuntimeConfig`、`AppParts`、`Live*`、`Kind`、`EffectKind`、`NodeInfo`、`RecordBits`、`TaskState`、`MotionState` など）を `kivact_runtime::driver`（`#[doc(hidden)]`）に移す（RC-30）。`spawn_in` を `driver` に移す（RC-35）。
  - 記録を一つの型 `TurnRecord` にし、原因を Copy の `Cause` にする。`TurnDetail`・`CauseEntry`・`DetailWrite`・`DetailRun`・`DetailEdge`・`DetailLog` を消す（RC-05 の型の部分）。
  - `Owner::current`（setup の owner）を `Owner::setup`、`Owner::ambient`（実行中の owner）を `Owner::running` に改名する（RC-41）。

### 手順

1. 新規 `src/driver.rs`：
   ```rust
   //! Role: The host driver's contract: what kivact-core and kivact-test use
   //! to create an App's runtime, run its turns and read its counters.
   //! Applications never use it.

   pub use crate::host::{AppParts, RuntimeConfig};
   pub use crate::motion::{AnimInfo, AnimKind, MotionEnv, MotionState, MotionStats, RunStatus};
   pub use crate::reactive::{
       EffectKind, ExitState, Host, Kind, LiveCounts, LiveObject, LiveOwner, LiveSnapshot, NodeInfo,
       OwnerInfo, RecordBits, Runtime, Stage, ThreadWaker, TurnGuard, TurnReport,
   };
   pub use crate::task::{TaskEntry, TaskState, TaskStats, spawn_in};
   ```
   `src/lib.rs` に次を足し、上の名前を `pub use` の列から消す（`lib.rs` の中で上の名前を使う行があれば `crate::driver::X` にする）。
   ```rust
   // xtask-allow(internal-contract): the host driver contract of kivact-core and kivact-test; applications use the facade
   #[doc(hidden)]
   pub mod driver;
   ```
   runtime の内部の `use crate::{.., Runtime, ..}` などは、`cargo check -p kivact-runtime --lib` の E0432 に従って `crate::driver::Runtime`（または `crate::reactive::Runtime`）にする。
2. 記録の型（`src/reactive/detail.rs` を次の内容で置き換え、`src/reactive/data.rs` の `TraceWrite`、`TraceRun`、`TurnRecord` を削除する。`Turn` は `data.rs` に残す）：
   ```rust
   //! Role: The turn trace: what each turn did, kept while a recorder is on.

   use std::panic::Location;

   use crate::reactive::host_types::Stage;
   use crate::reactive::identity::NodeId;
   use crate::reactive::data::Turn;

   /// The kind of work that started part of a turn.
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum CauseTag {
       /// Host input.
       Input,
       /// A due timer.
       Timer,
       /// A ready task.
       Task,
       /// Mount work with no host event.
       Mount,
       /// A DevTools call.
       Devtools,
   }

   /// What started part of a turn: a host event (its kind and node), a timer,
   /// a task, mounts or a DevTools call. Formatted only when read.
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub struct Cause {
       /// The kind of work.
       pub tag: CauseTag,
       /// What happened (`"Activate"`, `"timer"`, a DevTools method).
       pub event: &'static str,
       /// The host node the event was for, if any.
       pub node: Option<kivact_types::NodeId>,
   }

   impl std::fmt::Display for Cause {
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           match self.node {
               Some(node) => write!(f, "{} n{}", self.event, node.0),
               None => f.write_str(self.event),
           }
       }
   }

   /// One write of a signal in a turn.
   #[derive(Clone, Copy, Debug, PartialEq)]
   pub struct TraceWrite {
       /// The written node.
       pub node: NodeId,
       /// The index of the cause it followed, if any.
       pub cause: Option<u32>,
       /// Value type name.
       pub ty: &'static str,
       /// Where the signal was created.
       pub created_at: &'static Location<'static>,
       /// Where it was written.
       pub at: &'static Location<'static>,
       /// Whether the write changed the value.
       pub changed: bool,
   }

   /// One memo or effect run in a turn.
   #[derive(Clone, Copy, Debug, PartialEq)]
   pub struct TraceRun {
       /// The node that ran.
       pub node: NodeId,
       /// The index of the cause it followed, if any.
       pub cause: Option<u32>,
       /// `"memo"`, `"render"`, `"internal"` or `"effect"`.
       pub kind: &'static str,
       /// Where the memo or effect was created.
       pub at: &'static Location<'static>,
       /// Whether the run changed its value.
       pub changed: bool,
   }

   /// One dependency edge a write marked.
   #[derive(Clone, Copy, Debug, PartialEq)]
   pub struct TraceEdge {
       /// The changed source or memo.
       pub source: NodeId,
       /// The subscriber marked dirty.
       pub target: NodeId,
   }

   /// One turn stage and the time it took.
   #[derive(Clone, Copy, Debug, PartialEq)]
   pub struct StageSpan {
       /// The stage.
       pub stage: Stage,
       /// The App clock (ns) when it started.
       pub start_ns: u64,
       /// How long it took (ns).
       pub duration_ns: u64,
   }

   /// What one turn did.
   #[derive(Clone, Debug, Default, PartialEq)]
   pub struct TurnRecord {
       /// `Turn::Setup` for the notes of the App's setup, else the turn's number.
       pub turn: Turn,
       /// Host events delivered.
       pub events: usize,
       /// What started work, in order.
       pub causes: Vec<Cause>,
       /// Writes of signals.
       pub writes: Vec<TraceWrite>,
       /// Memo and effect runs.
       pub reruns: Vec<TraceRun>,
       /// Dependency edges marked by writes.
       pub edges: Vec<TraceEdge>,
       /// Stage spans.
       pub stages: Vec<StageSpan>,
       /// Number of memo and effect runs.
       pub runs: usize,
       /// Writes committed to the engine.
       pub ops: usize,
       /// Time spent in the turn (ns).
       pub duration_ns: u64,
       /// Notes (stale events, duplicate styles, diagnostics).
       pub notes: Vec<String>,
       /// Entries dropped at the per-list cap.
       pub dropped: u32,
   }
   ```
   `TurnRecord` の `Display`（今の `diag.rs` の impl）を次の形にする：1 行目は `"{turn}: {ops} op(s), {runs} re-run(s), causes [{c1}, {c2}, ..]"`（各 cause は `Display`）、続けて `changed` の書込みごとに `"\n  write {ty} (created at {created_at}) at {at}"`、note ごとに `"\n  note: {note}"`。
3. `src/reactive/data.rs` の `Diag`：`turns: VecDeque<TurnRecord>`（ring、256）、`current: TurnRecord`、`detail: DetailLog` を、`turns`・`current` と次の二つの field にする（`detail` を削除）。
   ```rust
   /// The index of the last cause recorded in the current turn.
   pub(crate) cause: Option<u32>,
   /// The stage being timed and the turn-relative ns it started at.
   pub(crate) open_stage: Option<(Stage, u64)>,
   ```
   `src/reactive/diag.rs` の `impl Diag` を次の規則で書き換える（記録者が無ければ何もしない規則は R08 のまま。1 turn の各列の上限は `CAP = 2000`、超えたら `dropped += 1`）：
   - `cause(&mut self, cause: Cause)`：`current.causes` に push し、`self.cause = Some(index)`。
   - `record_write(&mut self, node: NodeId, ty, created_at, at, changed: bool)`：`TraceWrite { node, cause: self.cause, ty, created_at, at, changed }` を push。
   - `record_run(&mut self, node: NodeId, kind, at, changed: bool)`：`runs += 1` と `TraceRun` を push。
   - `record_edge(&mut self, source: NodeId, target: NodeId)`：`TraceEdge` を push。
   - `stage(&mut self, stage, base_ns, elapsed_ns)`：今の `DetailLog::stage` の本体（`open_stage` を閉じて `StageSpan` を push、新しい stage を開く）。
   - `end_turn`：今の本体に、開いた stage を閉じる処理（今の `DetailLog::end_turn` の前半）と、`self.cause = None; self.open_stage = None;` を足す。
   - `graph.rs` の `diag.detail.write(..)` と `diag.record_write(ty, created_at, at)` の二つの呼出しを一つの `diag.record_write(NodeId(key), meta.ty, meta.at, at, changed)` に、`diag.detail.run(..)` と `record_run(label, at)` を一つの `record_run(NodeId(key), label, at, changed)`（`recompute` で `changed` が分かった後）に、`diag.detail.edge(a, b)` を `diag.record_edge(a, b)` にする。
4. `src/reactive/diag.rs` の `impl Runtime`：
   - `record_cause_with(&self, tag: CauseTag, label: String)` を `record_cause(&self, cause: Cause)` にする。
   - `take_turn_details` を削除し、次を足す。
     ```rust
     /// The newest turn record, if a recorder is on and a turn ended.
     pub fn last_turn(&self) -> Option<TurnRecord> {
         self.diag.borrow().turns.back().cloned()
     }
     ```
   - `turns()` はそのまま（`Vec<TurnRecord>` を返す）。
5. `src/reactive/host.rs` の `TurnGuard::cause_with(&self, tag: CauseTag, label: String)`（`#[doc(hidden)]` と `xtask-allow` の行を含む）を次にする。runtime の中の `cause_with` の呼出し（`src/host.rs` の timer と task）を `self.cause(Cause { tag: CauseTag::Timer, event: "timer", node: None })`・`CauseTag::Task, event: "task"` にする（`if rt.is_recording()` の判定は不要になるので消す）。
   ```rust
   /// Record what started the following work (kept only while recording).
   pub fn cause(&self, cause: Cause) {
       self.rt.record_cause(cause);
   }
   ```
6. `src/reactive.rs` の `pub use detail::{CauseEntry, CauseTag, DetailEdge, DetailRun, DetailWrite, StageSpan, TurnDetail};` と `pub use data::{.. TraceRun, TraceWrite, Turn, .. TurnRecord, ..}` を `pub use detail::{Cause, CauseTag, StageSpan, TraceEdge, TraceRun, TraceWrite, TurnRecord};` と `pub use data::Turn` にする。`src/lib.rs` の root の `pub use` は `Cause, CauseTag, StageSpan, TraceEdge, TraceRun, TraceWrite, Turn, TurnRecord, Reported, ReportKind` を持つ（`kivact::diagnostics` が再公開する）。`Stage` は `driver` と root の両方に置かない：`StageSpan.stage` の型のため root に `pub use reactive::Stage;` を置き、`driver` からは外す（手順 1 の `driver.rs` の `Stage` を消す）。
7. `Owner` の改名（runtime の全ファイル）：
   ```
   git ls-files 'crates/kivact-runtime/*.rs' | xargs perl -pi -e 's/\bOwner::ambient\(/Owner::running(/g; s/\bOwner::current\((?!\))/Owner::setup(/g; s/\bpub fn ambient\(/pub fn running(/; s/\bpub fn current\(api/pub fn setup(api/'
   ```
   `src/reactive/owner_handle.rs` の二つの method の doc を「The owner of the component setup that is running (setup only).」と「The owner of the code that is running: setup, handler, task, frame callback, cleanup, a running `Callback` or effect.」にする。
8. `src/task.rs` の `pub use spawn::{Task, spawn, spawn_in};` はそのまま（`driver` が `spawn_in` を再公開する）。`src/lib.rs` の root の `pub use task::{..}` から `spawn_in` を消す。`spawn_in` の doc の先頭に `Driver contract:` を足す。

### 消すもの

- `TurnDetail`、`CauseEntry`、`DetailWrite`、`DetailRun`、`DetailEdge`、`DetailLog`、`Runtime::take_turn_details`、`TurnGuard::cause_with`、`Runtime::record_cause_with`、`Owner::current`（runtime。core の 0 引数の `kivact_core::state::Owner::current()` は付属書 C が扱う）、`Owner::ambient`、root の driver の名前の `pub use`。

### 状態の閉じ方

- `Diag`：R09 の印の文面の `detail` を `cause, open_stage` に置き換える（`...; record says which recorders are on, and turns, current, cause and open_stage stay empty while it is off; node_meta holds ...`）。
- 記録の型は値の記録（constructor でだけ書く）。対象外。

### 試験

- R31 で `diagnostics::trace_records_causes_writes_reruns_ops_and_duration` と `focus` の `focus_results` を書き換える（4 章 4.6）。

### 確認（tier a）

共通の形。

### 完了条件

- `rg -n 'TurnDetail|CauseEntry|DetailWrite|DetailRun|DetailEdge|DetailLog|take_turn_details|cause_with|Owner::ambient|Owner::current\(' crates/kivact-runtime/src` が何も返さない。
- `rg -n '^pub use' crates/kivact-runtime/src/lib.rs` に `Runtime`、`TurnGuard`、`TaskState`、`MotionState`、`ThreadWaker`、`RuntimeConfig`、`spawn_in` が無い。

## W5 の鎖の掃引（R26〜R31）の共通の道具

R26 の最初の手順で、次の 4 つの perl の script を `target/rework/` に作る（`target/` は git の外。R27〜R31 も同じ script を使う）。中身は一字一句このとおりにする。

`target/rework/r_sweep.pl`（ファイル全体を一度に読む `-0` で使う。R20〜R25 の改名と削除のうち機械的なもの）：
```perl
# Binding -> Signal (R20)
s/(?:\b\w+::)*IntoBinding::into_binding\(((?:[^()]|\((?1)\))*)\)/$1/g;
s/\.into_binding\(\)//g;
s/\bimpl\s+((?:\w+::)*)IntoBinding</$1Signal</g;
s/\bIntoBinding\b/Signal/g;
s/\bBinding\b/Signal/g;
# use_context returns T (R20)
s/(try_use_context::<(?:[^<>]|<(?:[^<>]|<[^<>]*>)*>)+>\(\)\s*\.map\(\|(\w+)\|\s*)\2\.get\(\)/$1$2/g;
s/\.map\(\|(\w+)\|\s*\1\s*\)//g;
s/(\buse_context::<(?:[^<>]|<(?:[^<>]|<[^<>]*>)*>)+>\(\))\.get\(\)/$1/g;
# actions (R22)
s/(?:\b\w+::)*ActionOutcome::Succeeded\(/Ok(/g;
s/(?:\b\w+::)*ActionOutcome::Failed\(/Err(/g;
s/\baction\(\s*(?:\w+::)*ActionMode::Latest\s*,\s*/action(/g;
s/\baction\(\s*(?:\w+::)*ActionMode::Queue\s*,\s*/queued_action(/g;
# Trigger (R23)
s/\bNotifier\b/Trigger/g;
s/\.notifier\(/.trigger(/g;
s/(?<![\w.])notifier\(/trigger(/g;
s/\bnotifier\b(?=\s*[,}])/trigger/g;
# props traits (R24)
s/\bIntoReactive\b/IntoValue/g;
s/\bcustom::Reactive\b/custom::Prop/g;
s/\bReactive::/Prop::/g;
s/\bReactive</Prop</g;
s/\bstate::__private::Prop\b/custom::Prop/g;
# Owner entry points (R25)
s/\bOwner::ambient\(/Owner::running(/g;
s/\bOwner::current\((?!\))/Owner::setup(/g;
```

`target/rework/r_driver.pl`（`-0` で使う。R25 で `kivact_runtime::driver` に移った名前の path を直す）：
```perl
BEGIN { %d = map { $_ => 1 } qw(Runtime RuntimeConfig AppParts TurnGuard ExitState TurnReport Host ThreadWaker LiveCounts LiveObject LiveOwner LiveSnapshot OwnerInfo Kind EffectKind NodeInfo RecordBits TaskState TaskStats TaskEntry MotionState MotionEnv MotionStats AnimInfo AnimKind RunStatus spawn_in); }
s{use\s+kivact_runtime::\{([^{}]*)\};}{
    my @items = grep { /\S/ } map { s/^\s+|\s+$//gr } split /,/, $1;
    my @drv = grep { $d{ (split /\s+as\s+/, $_)[0] } } @items;
    my @rest = grep { !$d{ (split /\s+as\s+/, $_)[0] } } @items;
    my @out;
    push @out, "use kivact_runtime::{" . join(', ', @rest) . "};" if @rest;
    push @out, "use kivact_runtime::driver::{" . join(', ', @drv) . "};" if @drv;
    join("\n", @out)
}ge;
s{\bkivact_runtime::(\w+)\b}{ $d{$1} ? "kivact_runtime::driver::$1" : "kivact_runtime::$1" }ge;
```

`target/rework/r_handler_sig.pl`（行ごとの `-p` で使う。1 行に書いた `on_*` などの署名の `impl Into<..Callback<()>>` を `impl IntoHandler<(), M>` にし、関数に generic の `M` を足す）：
```perl
if (/\bfn\s+\w+/ && /impl\s+(?:::)?(?:std::convert::)?Into<(?:\w+::)*Callback<\(\)>>/) {
    s/\bfn\s+(\w+)(?:<([^()]*)>)?\s*\(/"fn $1<" . (defined $2 ? "$2, " : "") . "M>("/e;
    s/impl\s+(?:::)?(?:std::convert::)?Into<((?:\w+::)*)Callback<\(\)>>(?:\s*\+\s*'static)?/impl ${1}IntoHandler<(), M>/g;
}
```

`target/rework/r_handler_body.pl`（`-0` で使う。`r_handler_sig.pl` が変えた引数の `.into()` を `Callback::from_handler(..)` にする。`KV_ROOT` は呼び出す側が環境変数で渡す path の前置き：core は `kivact_runtime::`、interaction と ui は `kivact_core::state::`、それ以外は `kivact::state::`）：
```perl
BEGIN { $root = $ENV{KV_ROOT} // die "KV_ROOT" }
my %n;
while (/\b(\w+):\s*impl\s+(?:\w+::)*IntoHandler<\(\), M>/g) { $n{$1} = 1; }
for my $v (keys %n) {
    my $conv = "${root}Callback::from_handler(${root}IntoHandler::into_handler($v))";
    s/(?<![\w.])$v\.into\(\)/$conv/g;
    s/(?:::std::convert::|std::convert::)?\bInto::into\($v\)/$conv/g;
}
```

各 unit の掃引の命令は次の形（`<paths>` は unit が書く）：
```
git ls-files <paths> | grep '\.rs$' | xargs perl -0pi target/rework/r_sweep.pl
git ls-files <paths> | grep '\.rs$' | xargs perl -pi target/rework/r_handler_sig.pl
git ls-files <paths> | grep '\.rs$' | KV_ROOT=<root> xargs perl -0pi target/rework/r_handler_body.pl
```

script の後の `cargo check` の誤りは、次の種類だけを本書のとおりに直す（種類 F0〜F9。各 unit はこの表を引く）。

| 種類 | 誤り | 直し方 |
|---|---|---|
| F0 | E0252/E0254（`Signal` や `IntoValue` を二度 import） | 同じ `use` の二つ目の名前を消す |
| F1 | unused import（`Signal`、`Prop` など） | その名前を `use` から消す |
| F2 | E0405/E0425（`IntoHandler`・`Callback`・`Trigger`・`trigger`・`Prop`・`queued_action` が見つからない） | そのファイルが `Callback` や `Signal` を import している `use` に名前を足す（core は `kivact_runtime::`、interaction・ui は `kivact_core::state::`、他は `kivact::state::`。`queued_action` は `task::` の下） |
| F3 | E0308：`update(..)` の戻り値を使っている（`let x = s.update(f)`、式の値） | `s.update(|v| { <f の本体の副作用>; })` の後に `let x = s.with_untracked(|v| <取り出す式>)` と書く。`Generation::next` の形は `s.update(|g| { g.next(); }); let id = s.get_untracked();`（`Var` は `get()`） |
| F4 | E0599：`Callback` に `bind`・`to_fn` が無い | `cb.bind(e)` の結果の `Callback<()>` を呼ぶ所を `cb.call(e)` にする closure にし、`cb.to_fn()` は `move |e| cb.call(e)` にする |
| F5 | E0599：`Signal` に `notify` が無い | その signal を `trigger()` で作り直し、読む側の `.get()`（値を使っていない読み）を `.track()` にする |
| F6 | E0277：`impl IntoHandler<(), M>` が `Into<..>` を満たさない（script が拾わなかった `.into()`） | `Callback::from_handler(IntoHandler::into_handler(x))`（F2 の path） |
| F7 | E0061/E0308：`action(..)` の引数の数・`ActionMode::QueueLatest` | `ActionMode::QueueLatest` の使用は R31 の試験だけにある。他で出たら STOP |
| F8 | E0308：`try_use_context` の結果の型（script が拾わなかった `.map(|v| v.get())` の形） | `.map(..)` の closure の中の `v.get()` を `v` にし、残った `.map(|v| v)` を消す |
| F9 | E0599：`Owner::current`（2 引数）・`Owner::ambient` | `Owner::setup`・`Owner::running` |

複数行に分かれた署名の `impl Into<..Callback<()>>`（script は 1 行の署名だけを直す）は、`rg -n 'Into<[\w:]*Callback<\(\)>>' <paths>` で残りを探し、`r_handler_sig.pl` と同じ変更（fn に `<M>`、型を `impl ..IntoHandler<(), M>`）を手で行い、本体の `.into()` を F6 で直す。

## R26：core・macro・facade を R20〜R22 に合わせる

- **wave / lane / 前提**：W5a / 鎖 / R25。
- **所有**：`crates/kivact-core/`（全体）、`crates/kivact-macros/src/{expand.rs,parse.rs}`、`crates/kivact/src/prelude.rs`、`target/rework/*.pl`（作業用）。
- **目的**：R20（`Signal`・`update`・`use_context`）、R21（`IntoHandler`）、R22（非同期）を core・macro・facade で使えるようにする。core の `provide` が `T` をそのまま置く（RC-31 の core の一行。付属書 C と合意、4 章 4.2）。`for_each` の書ける行が list を写さずにその場で直す（RC-23 の後半）。`on_press` が arena の object を作らない（API-03）。

### 手順

1. 「共通の道具」の 4 つの script を作る。
2. core に script を当てる（R23〜R25 の改名もこの時点で当たるが、core の残りの直しは R27 で行う。この unit の確認は R27 の後にまとめて行う）。
   ```
   git ls-files 'crates/kivact-core/*.rs' | xargs perl -0pi target/rework/r_sweep.pl
   git ls-files 'crates/kivact-core/*.rs' | xargs perl -pi target/rework/r_handler_sig.pl
   git ls-files 'crates/kivact-core/*.rs' | KV_ROOT=kivact_runtime:: xargs perl -0pi target/rework/r_handler_body.pl
   ```
3. `crates/kivact-core/src/view/boundary.rs` の `provide`：`let value = owner.constant(value, at);` の行を消す（`owner.provide(value);` が `T` をそのまま置く）。doc の「each `use_context::<T>()` returns a `Value<T>`; read it with `get` or `with`.」を「each `use_context::<T>()` returns a clone of it; provide a `Signal` or a `Value` for readers that must see changes.」にする。
4. `crates/kivact-core/src/view/element_data.rs`：script の後の `on_press` と `on_exit_complete` の本体を次にする（`on_press` は arena の object を作らない）。
   ```rust
   fn on_exit_complete<M>(mut self, callback: impl IntoHandler<(), M>) -> Self {
       // The callback's owner is the caller's; an error goes to its boundary.
       let callback = Callback::from_handler(callback.into_handler());
       self.node_mut().common.exit_complete = Some(kivact_runtime::Handler::from_fn(move |()| {
           callback.call_if_alive(())
       }));
       self
   }

   fn on_press<M>(mut self, f: impl IntoHandler<(), M>) -> Self {
       let handler = f.into_handler();
       self.node_mut().common.set_press(move || handler.call(()));
       self
   }
   ```
   `use kivact_runtime::{..}` に `IntoHandler` を足す（F2）。`#[track_caller]` の属性はそのまま残す。
5. `crates/kivact-core/src/view/flow/list.rs`：
   - script で `impl<T: PropValue> ListSource<T> for Signal<Vec<T>>` が二つになる（元の `Binding<Vec<T>>` の impl）。二つ目（本体が `writable_list(self)`）を削除し、一つ目の本体を `writable_list(self)` に、`type Row = Signal<T>;` にする。doc と `on_unimplemented` の文から `Binding` の語を消す（script で `Signal` になった重複の語を直す）。
   - `fn writable_list` を次にする。
     ```rust
     fn writable_list<T: PropValue>(source: Signal<Vec<T>>) -> ListInput<T, Signal<T>> {
         ListInput {
             items: source.into_prop(),
             row: Rc::new(move |item, same_key| {
                 let initial = item.get_untracked();
                 // A row reads its own item, which the list keeps current. A
                 // write through the row edits that element of the list in
                 // place (the list is not copied) and sets the item, so the
                 // row reads it back at once.
                 kivact_runtime::binding(item, move |replacement: T| {
                     let found = source.with_untracked(|list| {
                         list.iter()
                             .position(|candidate| same_key(candidate, &initial))
                             .map(|index| (index, list[index] != replacement))
                     });
                     let Some((index, true)) = found else { return };
                     let stored = replacement.clone();
                     source.update(move |list| list[index] = stored);
                     item.set(replacement);
                 })
             }),
         }
     }
     ```
6. `crates/kivact-core/src/view/flow/error_boundary.rs`：
   - `use kivact_runtime::Retry;` を `use kivact_runtime::{Callback, Handler};` にする（既にある import と F0 で重ねない）。
   - `type FallbackFn = crate::view::ViewFn<(UiError, Retry)>;` を `type FallbackFn = crate::view::ViewFn<(UiError, Callback<()>)>;`、`pub fn fallback<V: IntoView>(mut self, f: impl Fn(UiError, Retry) -> V + 'static) -> Self` の `Retry` を `Callback<()>` にする。doc の `retry.run()` の例を `retry.call(())`（または `.on_press(retry)`）にする。
   - `let reset = Retry::new(move || shown.set(Shown::Child));` を `let reset = Callback::from_handler(Handler::from_fn(move |()| shown.set(Shown::Child)));` にする。
7. `crates/kivact-core/src/task.rs`：`pub use kivact_runtime::{Action, ActionFn, ActionMode, ActionOutcome, AsyncState, BlockingWork, CancelToken, Resource, Task, action, blocking, resource, sleep, spawn, timeout};` を `pub use kivact_runtime::{Action, AsyncState, CancelToken, Resource, Task, action, blocking, queued_action, resource, sleep, spawn, timeout};` にし、`pub use kivact_runtime::{AsyncSource, Retry};` を `pub use kivact_runtime::AsyncSource;` にし、`AsyncSnapshot` の `#[doc(hidden)]` の re-export と `xtask-allow` の行を消す。
8. `crates/kivact-core/src/state.rs`：`pub use kivact_runtime::{..}` から `Binding`（script で `Signal` になり重複する。F0）と `IntoBinding` を消し、`IntoHandler` を足す。`crates/kivact-core/src/custom.rs` の `pub use kivact_runtime::{Owned, clock_now, require_effectful, require_setup};` の後に次の 3 行を足す（付属書 C の R-REQ-2 の path `kivact::custom::Handler`）。
   ```rust
   // xtask-allow(internal-contract): #[component] expansion stores handler props as Handler<E>
   #[doc(hidden)]
   pub use kivact_runtime::Handler;
   ```
9. `crates/kivact-core/src/launch/turn.rs`：`turn.check_draining();` の行を消す。
10. `crates/kivact-macros/src/parse.rs`：`PropKind` の `/// \`Binding<T>\`: a two-way binding, kept as it is.` と `Binding(Type),` を `/// \`Signal<T>\`: a two-way value, passed as it is.` と `Signal(Type),` にし、型の判定の `"Binding" => match inner { Some(inner) => PropKind::Binding(inner.clone()), .. }` の key の文字列を `"Signal"`、variant を `PropKind::Signal` にする。
11. `crates/kivact-macros/src/expand.rs`（`#[component]` の handler の props。`Callback<()>` は `IntoHandler`、`Callback<E>`（E が `()` でない）は closure の引数の型推論のため `impl Fn(E) -> R` のまま受ける。決定 DEC-16）：
    - `fn r_ident` の後に次を足す。
      ```rust
      fn m_ident(prop: &Prop) -> Ident {
          format_ident!("M{}", upper_camel(&prop.name.to_string()), span = prop.name.span())
      }
      ```
    - `fn generic` に腕 `PropKind::Callback0 => { let marker = m_ident(prop); Some(quote! { #marker }) }` を足す。
    - `fn input_type`：`PropKind::Binding(ty) => quote! { impl ::kivact::state::IntoBinding<#ty> },` を `PropKind::Signal(ty) => quote! { ::kivact::state::Signal<#ty> },` に、`PropKind::Callback0 => { quote! { impl ::std::convert::Into<::kivact::state::Callback<()>> + 'static } }` を `PropKind::Callback0 => { let marker = m_ident(prop); quote! { impl ::kivact::state::IntoHandler<(), #marker> } }` にする。`PropKind::Callback(ty)` の腕は変えない。
    - `fn keep_type`：`PropKind::Binding(ty) => quote! { ::kivact::state::Binding<#ty> },` を `PropKind::Signal(ty) => quote! { ::kivact::state::Signal<#ty> },`、`PropKind::Callback0 => quote! { ::std::boxed::Box<dyn ::std::ops::FnOnce() -> ::kivact::state::Callback<()>> },` を `PropKind::Callback0 => quote! { ::kivact::custom::Handler<()> },`、`PropKind::Callback(ty) => quote! { ::std::boxed::Box<dyn ::std::ops::FnOnce() -> ::kivact::state::Callback<#ty>> },` を `PropKind::Callback(ty) => quote! { ::kivact::custom::Handler<#ty> },` にする。
    - `fn store`：`PropKind::Binding(_) => quote! { ::kivact::state::IntoBinding::into_binding(#input) },` を `PropKind::Signal(_) => quote! { #input },`、`PropKind::Callback0 => quote! { ::std::boxed::Box::new(move || ::std::convert::Into::into(#input)) },` を `PropKind::Callback0 => quote! { ::kivact::state::IntoHandler::into_handler(#input) },`、`PropKind::Callback(_) => quote! { ::std::boxed::Box::new(move || ::kivact::state::callback(#input)) },` を `PropKind::Callback(_) => quote! { ::kivact::custom::Handler::from_fn(#input) },` にする。
    - `fn mount`：`PropKind::Callback0 | PropKind::Callback(_) if prop.is_optional() => { Some(quote! { let #name = #name.map(|f| f()); }) }` を `Some(quote! { let #name = #name.map(::kivact::state::Callback::from_handler); })` に、`PropKind::Callback0 | PropKind::Callback(_) => Some(quote! { let #name = #name(); }),` を `Some(quote! { let #name = ::kivact::state::Callback::from_handler(#name); })` にする。
12. `crates/kivact/src/prelude.rs`：`Binding`・`IntoBinding`・`ActionMode` を消す（`pub use crate::task::ActionMode;` の行を削除し、`pub use crate::state::{IntoBinding, ItemKey, PropValue, bind};` を `pub use crate::state::{ItemKey, PropValue, bind};` にし、もう一つの `pub use crate::state::{..}` の `Binding` を消す）。それ以外の prelude の名前は付属書 C が 5.11 節のとおりにする。
13. この時点では core の R23〜R25 の部分（R27）が残るので、`cargo check` はまだ通らない。R27 に進む。

### 消すもの

- core の `Retry` と `AsyncSnapshot` の使用と re-export、`ListSource for Binding<Vec<T>>`、`writable_list` の list の clone、`provide` の `owner.constant`、`launch/turn.rs` の `check_draining`、macro の `IntoBinding`・`Binding`・`Box<dyn FnOnce() -> Callback>` の props の格納。

### 状態の閉じ方

- 新しい状態の型は無い。

### 試験

- 確認は R27 の後（R27 の「確認」で core・macro・facade を一緒に確かめる）。

### 確認（tier a）

```
cargo fmt -p kivact-core -p kivact-macros
cargo check -p kivact-macros
```

### 完了条件

- `rg -n '\bBinding\b|IntoBinding|into_binding|\bRetry\b|AsyncSnapshot|ActionMode|ActionOutcome|check_draining' crates/kivact-core/src crates/kivact-macros/src crates/kivact/src` が何も返さない（`"Retry"` の文字列を除く）。

## R27：core・facade・style・i18n を R23〜R25 に合わせ、core の `FrameLoop` の写しを消す

- **wave / lane / 前提**：W5a / 鎖 / R26。
- **所有**：`crates/kivact-core/`、`crates/kivact-macros/src/expand.rs`、`crates/kivact-macros/src/parse.rs`、`crates/kivact/src/{prelude.rs,lib.rs}`、`crates/kivact-style/`、`crates/kivact-i18n/`、`dev/xtask/policy/workspace.toml`（`module_rules.kivact-core` の `except` の 1 項目）。
- **目的**：`Trigger`・`remote`・`clock`、`Prop`（hidden）、記録の型、`driver` の path、`Owner::setup`/`running` を core・facade・style・i18n で使う。core の `motion::FrameLoop` の写し（`while_present(NodeRef)`）を消し、runtime の `FrameLoop` を公開する（RC-34）。

### 手順

1. style と i18n と core に script を当てる（core は R26 で `r_sweep.pl` を当てたので、`r_driver.pl` だけ）。
   ```
   git ls-files 'crates/kivact-style/*.rs' 'crates/kivact-i18n/*.rs' | xargs perl -0pi target/rework/r_sweep.pl
   git ls-files 'crates/kivact-core/*.rs' 'crates/kivact-style/*.rs' 'crates/kivact-i18n/*.rs' | xargs perl -0pi target/rework/r_driver.pl
   ```
2. `crates/kivact-core/src/state.rs`：
   - `pub use kivact_runtime::{..}` の `Notifier`・`notifier` が script で `Trigger`・`trigger` になったことを確かめる。`constant_value` をこの列から消し、次の行を足す。
     ```rust
     // xtask-allow(internal-contract): crates below the facade implement IntoValue for their value types with it; applications use #[derive(Prop)]
     #[doc(hidden)]
     pub use kivact_runtime::constant_value;
     ```
   - `pub mod __private;` と、その上の `xtask-allow` と `#[doc(hidden)]` の行を消し、`crates/kivact-core/src/state/__private.rs` を削除する。
3. `crates/kivact-core/src/custom.rs`：`pub use kivact_runtime::Reactive;` の行（とその上の `#[doc(inline)]` と doc）を次の 3 行にする（付属書 C の R-REQ-4 の path `kivact::custom::Prop`）。
   ```rust
   // xtask-allow(internal-contract): #[component] props and #[derive(Prop)] store inputs as Prop<T>
   #[doc(hidden)]
   pub use kivact_runtime::Prop;
   ```
4. `crates/kivact-core/src/task.rs`：`pub use kivact_runtime::{..}` に `Remote, RemoteTrigger, clock, remote, remote_trigger` を足す。
5. core の `FrameLoop`：
   - `crates/kivact-core/src/motion/frame_loop.rs` を削除し、`crates/kivact-core/src/motion.rs` の `mod frame_loop;` と `pub use frame_loop::{FrameLoop, frame_loop};` を消し、`pub use kivact_runtime::{..}` の列に `FrameLoop, frame_loop` を足す。
   - `dev/xtask/policy/workspace.toml` の `[module_rules.kivact-core]` の `except` から `"motion::frame_loop -> view::node_ref", ` を消す。
6. `crates/kivact-core/src/diagnostics.rs`：`pub use kivact_runtime::{TraceRun, TraceWrite, Turn, TurnRecord};` を `pub use kivact_runtime::{Cause, CauseTag, Stage, StageSpan, TraceEdge, TraceRun, TraceWrite, Turn, TurnRecord};` にする。module の doc の「a ring buffer of structured turn records」を「structured turn records (kept while a test screen, DevTools or the debug trace records)」にする。
7. `crates/kivact-core/src/launch/turn.rs`：
   - `fn describe(e: &HostEvent) -> String` と `fn phase_name` を削除し、次を置く。
     ```rust
     /// What a host event says for the turn trace (its kind and node, no values).
     fn cause_of(e: &HostEvent) -> Cause {
         use kivact_engine::contract::HostEvent as E;
         let (event, node) = match e {
             E::Pointer(route) => ("Pointer", route.target.map(|t| t.node)),
             E::Wheel(route) => ("Wheel", route.target.map(|t| t.node)),
             E::File(route) => ("File", route.target.map(|t| t.node)),
             E::Key(route) => ("Key", route.target.map(|t| t.node)),
             E::Activate { node, .. } => ("Activate", Some(*node)),
             E::FocusChanged { node, .. } => ("FocusChanged", *node),
             E::HoverChanged { target, .. } => ("HoverChanged", target.map(|t| t.node)),
             E::TextChanged { node, .. } => ("TextChanged", Some(*node)),
             E::SelectionChanged { node, .. } => ("SelectionChanged", Some(*node)),
             E::Ime { node, .. } => ("Ime", Some(*node)),
             E::ScrollChanged { node, .. } => ("ScrollChanged", Some(*node)),
             E::LayoutChanged { node, .. } => ("LayoutChanged", Some(*node)),
             E::A11y { node, .. } => ("A11y", Some(*node)),
             E::PresenceChanged { .. } => ("PresenceChanged", None),
             E::ExitComplete { .. } => ("ExitComplete", None),
             E::ModifiersChanged { .. } => ("ModifiersChanged", None),
             E::MousePosition { .. } => ("MousePosition", None),
             E::FrameTick { .. } => ("FrameTick", None),
             E::EffectResult { .. } => ("EffectResult", None),
             E::SnapshotResult { .. } => ("SnapshotResult", None),
             E::FocusResult { .. } => ("FocusResult", None),
             E::Lifecycle(_) => ("Lifecycle", None),
             E::EnvChanged { .. } => ("EnvChanged", None),
             E::HostFailure { .. } => ("HostFailure", None),
             E::TurnsCut { .. } => ("TurnsCut", None),
             E::ImageStatusChanged { .. } => ("ImageStatusChanged", None),
             E::AssetFailed { .. } => ("AssetFailed", None),
             E::DrawDropped { .. } => ("DrawDropped", None),
             E::DrawableNotHosted { .. } => ("DrawableNotHosted", None),
             E::DrawableDrawnTwice { .. } => ("DrawableDrawnTwice", None),
             E::LayoutWarning { .. } => ("LayoutWarning", None),
             E::DuplicateLayoutId { .. } => ("DuplicateLayoutId", None),
             E::CloseRequested { .. } => ("CloseRequested", None),
             E::WindowOpened { .. } => ("WindowOpened", None),
             E::WindowFailed { .. } => ("WindowFailed", None),
             E::Foreign { .. } => ("Foreign", None),
             E::InjectDelivered { .. } => ("InjectDelivered", None),
             E::DevtoolsPick { .. } => ("DevtoolsPick", None),
             E::DevtoolsKey { .. } => ("DevtoolsKey", None),
             E::AppMotion { .. } => ("AppMotion", None),
             E::ExternalImported { .. } => ("ExternalImported", None),
             E::GpuAdapterChanged { .. } => ("GpuAdapterChanged", None),
             E::DrawableFrame(_) => ("DrawableFrame", None),
             E::DrawableResized { .. } => ("DrawableResized", None),
             E::DrawableTextures { .. } => ("DrawableTextures", None),
             E::DrawableExportLost(_) => ("DrawableExportLost", None),
             E::GpuExportAvailable => ("GpuExportAvailable", None),
         };
         Cause { tag: CauseTag::Input, event, node }
     }
     ```
     `HostEvent` の variant が上と違う（付属書 E の unit が足した・消した）ときは、その variant の腕を同じ形で足す・消す。`FocusChanged` の `node` が `Option` でなければ `Some(*node)` にする。
   - `if rt.is_recording() { turn.cause_with(CauseTag::Mount, "mounts".into()); }` を `turn.cause(Cause { tag: CauseTag::Mount, event: "mounts", node: None });` に、`if rt.is_recording() { turn.cause_with(CauseTag::Input, describe(event)); }` を `turn.cause(cause_of(event));` にする。`use kivact_runtime::{..}` に `Cause` を足す。
8. `crates/kivact-core/src/devtools/turn.rs` の `turn.cause_with(CauseTag::Devtools, format!("devtools {}", call.method()));` を `turn.cause(kivact_runtime::Cause { tag: CauseTag::Devtools, event: call.method(), node: None });` にする。
9. `crates/kivact-core/src/devtools/updates.rs`：
   - `use kivact_runtime::{CauseEntry, CauseTag, DetailEdge, DetailRun, DetailWrite, Runtime};` を `use kivact_runtime::{Cause, CauseTag, TraceEdge, TraceRun, TraceWrite, Turn};` と driver の `Runtime`（R27 の `r_driver.pl` が直す形）にする。ファイルの中の `CauseEntry` を `Cause`、`DetailWrite` を `TraceWrite`、`DetailRun` を `TraceRun`、`DetailEdge` を `TraceEdge`、`kivact_runtime::TurnDetail` を `kivact_runtime::TurnRecord` にする。
   - `record_turn` の `let detail = rt.take_turn_details().pop();` を `let detail = rt.last_turn();`、`detail.filter(|detail| detail.turn != 0)` を `detail.filter(|detail| matches!(detail.turn, Turn::Number(_)))` にする。
   - `build_graph` の `graph.add_runs(i, &detail.runs);` を `graph.add_runs(i, &detail.reruns);`、`turn: detail.turn,` を `turn: match detail.turn { Turn::Number(n) => n, Turn::Setup => 0 },` にする。
   - `label: cause.label.clone(),` を `label: cause.to_string(),` にする。
10. `crates/kivact-core/src/client/trace.rs` の `if let Some(record) = rt.turns().last() {` を `if let Some(record) = rt.last_turn() {` にする。
11. `crates/kivact-macros/src/expand.rs`：`PropKind::Text => quote! { ::kivact::custom::Reactive<String> },` を `PropKind::Text => quote! { ::kivact::custom::Prop<String> },`、`PropKind::Value(ty) => quote! { ::kivact::custom::Reactive<#ty> },` を `PropKind::Value(ty) => quote! { ::kivact::custom::Prop<#ty> },`、`PropKind::Text => quote! { ::kivact::custom::Reactive::from_text(#input) },` を `PropKind::Text => quote! { ::kivact::custom::Prop::from_text(#input) },`、`PropKind::Value(_) => quote! { ::kivact::custom::Reactive::new(#input) },` を `PropKind::Value(_) => quote! { ::kivact::custom::Prop::new(#input) },` にする。`parse.rs` の doc の `kept as \`Reactive<String>\``・`kept as \`Reactive<T>\`` を `kept as \`Prop<String>\``・`kept as \`Prop<T>\`` にする。
12. `crates/kivact/src/prelude.rs`：`constant_value` を `pub use crate::state::{..}` の列から消す（隠した名前は prelude に置かない）。`crates/kivact/src/lib.rs` が `kivact_core::state` などを module ごと再公開していることを確かめる（個々の名前を列挙していれば、R20〜R25 で消えた名前を消し、`Trigger`・`trigger` を足す）。
13. F0〜F9 の直しで `cargo check` を通す。core の `.notify()` の receiver はすべて `Trigger`（R23 の前提で `notify_from_render` は無い）。

### 消すもの

- `crates/kivact-core/src/motion/frame_loop.rs`、`state/__private.rs`、`custom::Reactive`、`describe`・`phase_name`、`take_turn_details` の使用、policy の `motion::frame_loop -> view::node_ref` の例外。

### 状態の閉じ方

- 新しい状態の型は無い。

### 試験

- 既存の core の unit 試験と `kivact-macros` の試験を通す（e2e は R31）。

### 確認（tier a）

```
cargo fmt -p kivact-core -p kivact-macros -p kivact-style -p kivact-i18n
cargo check -p kivact-runtime -p kivact-i18n -p kivact-style --lib
cargo check -p kivact-core --lib
cargo check -p kivact-core --lib --no-default-features
cargo check -p kivact-macros --all-targets
cargo clippy -p kivact-runtime -p kivact-i18n -p kivact-style -p kivact-core --lib -- -D warnings
cargo xtask check --no-write --crate kivact-core
cargo xtask check --no-write --crate kivact-style
cargo xtask check --no-write --crate kivact-i18n
cargo xtask check --no-write --crate kivact-macros
```

### 完了条件

- `rg -n '\bNotifier\b|\bnotifier\b|\bReactive\b|IntoReactive|__private|Owner::ambient|TurnDetail|CauseEntry|cause_with|take_turn_details|with_presence_gate' crates/kivact-core/src crates/kivact-style/src crates/kivact-i18n/src crates/kivact/src` が何も返さない（`PointerObservation::Reactive` と `view/state.rs` の enum の variant `Reactive(..)`、doc の文の `Reactive` という英単語を除く）。
- 上の命令がすべて成功する。

## R28：kivact-interaction の呼出し元を直す

- **wave / lane / 前提**：W5a / 鎖 / R27。
- **所有**：`crates/kivact-interaction/`。
- **目的**：R20〜R25 の公開 API の変更に interaction を合わせる。

### 手順

1. `crates/kivact-interaction/src/selection.rs`：script の前に、`Binding<Option<K>>` と `Binding<Vec<K>>` の `IntoSelectionSeal` と `IntoSelection` の impl（`impl<K: 'static> crate::sealed::IntoSelectionSeal<K> for Binding<Option<K>> {}` など。`Signal<..>` の impl と対になるもの）を削除する。
2. script を当てる。
   ```
   git ls-files 'crates/kivact-interaction/*.rs' | xargs perl -0pi target/rework/r_sweep.pl
   git ls-files 'crates/kivact-interaction/*.rs' | xargs perl -pi target/rework/r_handler_sig.pl
   git ls-files 'crates/kivact-interaction/*.rs' | KV_ROOT=kivact_core::state:: xargs perl -0pi target/rework/r_handler_body.pl
   ```
3. `update` の戻り値（F3）：
   - `src/drag/handle.rs` の `let id = self.sources.update(Generation::next);`（`Var<Generation>`）を `self.sources.update(|g| { g.next(); }); let id = self.sources.get();` にする。
   - `src/toaster/api.rs` の `ToastId` を作る関数の `let id = ToastId(self.next.update(Generation::next));` を `self.next.update(|g| { g.next(); }); let id = ToastId(self.next.get_untracked());` にする。同じファイルの `pub fn hold_while(self, held: impl IntoValue<bool>)` の `let id = HoldId(self.next.update(Generation::next));` を `self.next.update(|g| { g.next(); }); let id = HoldId(self.next.get_untracked());` にする（`update` は値を返さない。DEC-15）。
   - `src/tooltip/api.rs` の `fn take_id(&self) -> TooltipId` の本体 `self.next.update(Generation::next)` を `self.next.update(|g| { g.next(); }); self.next.get_untracked()` にする（`Hub::take_id`。`update` の後に読む）。
4. `src/hover_corridor/api.rs`・`src/roving/registry.rs`：script で `notifier` が `trigger` になった。同じ scope に `trigger` という名の local や関数がある（E0428・E0308）なら、import を `use kivact_core::state::trigger as new_trigger;` ではなく、呼出しを `kivact_core::state::trigger()` の完全な path にして import から外す。
5. `src/scalar/range.rs`：script で `kivact_core::custom::Prop` になったことを確かめる（`__private` が無い）。
6. F0〜F9 で `cargo check` を通す。

### 消すもの

- `IntoSelection` の `Binding` の impl。

### 状態の閉じ方

- 状態の型の変更は無い。

### 試験

- interaction の unit 試験（あれば）を通す。e2e は R31。

### 確認（tier a）

```
cargo fmt -p kivact-interaction
cargo check -p kivact-interaction --lib
cargo clippy -p kivact-interaction --lib -- -D warnings
cargo xtask check --no-write --crate kivact-interaction
```

### 完了条件

- `rg -n '\bBinding\b|IntoBinding|into_binding|\bNotifier\b|\bReactive\b|IntoReactive|__private|Into<[\w:]*Callback<\(\)>>' crates/kivact-interaction/src` が何も返さない（doc の英単語を除く）。

## R29：kivact-ui の呼出し元を直す

- **wave / lane / 前提**：W5a / 鎖 / R28。
- **所有**：`crates/kivact-ui/`。
- **目的**：R20〜R25 の公開 API の変更に ui を合わせる。`async_view` を一つの signal（`publications`）と `Callback<()>` の retry にする（RC-15、RC-35）。

### 手順

1. script を当てる。
   ```
   git ls-files 'crates/kivact-ui/*.rs' | xargs perl -0pi target/rework/r_sweep.pl
   git ls-files 'crates/kivact-ui/*.rs' | xargs perl -pi target/rework/r_handler_sig.pl
   git ls-files 'crates/kivact-ui/*.rs' | KV_ROOT=kivact_core::state:: xargs perl -0pi target/rework/r_handler_body.pl
   ```
2. `src/async_view.rs` を次にする（`default_loading` と doc はそのまま）。
   ```rust
   use kivact_core::UiError;
   use kivact_core::state::{Callback, untrack};
   use kivact_core::task::{AsyncSource, AsyncState};
   use kivact_core::view::{Element, IntoView, View, dynamic};
   use std::rc::Rc;

   /// A view of a resource or action, with optional loading and error content.
   pub struct AsyncView<T> {
       publications: Rc<dyn Fn() -> u64>,
       state: Rc<dyn Fn() -> AsyncState<T>>,
       retry: Callback<()>,
       ready: Rc<dyn Fn(T) -> View>,
       loading: Rc<dyn Fn() -> View>,
       fallback: Rc<dyn Fn(UiError, Callback<()>) -> View>,
   }

   /// Display an async source. Failures include a retry button by default.
   pub fn async_view<T: Clone + 'static, V: IntoView>(
       source: impl AsyncSource<T>,
       ready: impl Fn(T) -> V + 'static,
   ) -> AsyncView<T> {
       AsyncView {
           publications: Rc::new(move || source.publications()),
           state: Rc::new(move || source.state()),
           retry: source.retry(),
           ready: Rc::new(move |value| ready(value).into_view()),
           loading: Rc::new(default_loading),
           fallback: Rc::new(default_fallback),
       }
   }

   impl<T> AsyncView<T> {
       // loading はそのまま。
       /// Replace the default error message and retry button.
       pub fn fallback<V: IntoView>(mut self, f: impl Fn(UiError, Callback<()>) -> V + 'static) -> Self {
           self.fallback = Rc::new(move |error, retry| f(error, retry).into_view());
           self
       }
   }

   impl<T: Clone + 'static> IntoView for AsyncView<T> {
       fn into_view(self) -> View {
           dynamic(
               move || (self.publications)(),
               move |_| match untrack(|| (self.state)()) {
                   AsyncState::Idle => ().into_view(),
                   AsyncState::Pending => (self.loading)(),
                   AsyncState::Ready(value) => (self.ready)(value),
                   AsyncState::Failed(error) => (self.fallback)(error, self.retry),
               },
           )
           .into_view()
       }
   }

   fn default_fallback(error: UiError, retry: Callback<()>) -> View {
       super::alert(format!("Error: {error}"))
           .tone(kivact_interaction::Tone::Danger)
           .actions(super::button("Retry").on_press(retry))
           .into_view()
   }
   ```
   （`dynamic` が `impl IntoValue<u64>` の closure を受けることを確かめる。`dynamic` の第 1 引数の bound が R24 で `IntoValue` になっている。）
3. `src/lib.rs` の module の doc の `` `impl Into<Callback<()>>` (a closure `move || ..` or a shared `Callback`); `` を `` `impl IntoHandler<(), M>` (a closure `move || ..` or a shared `Callback`); `` にする。
4. `src/date_wheel.rs`：script で `notifier()`・`Notifier` が `trigger()`・`Trigger` になったことを確かめ、手順 4（R28）と同じ規則で名前の衝突を直す。
5. F0〜F9 で `cargo check` を通す。

### 消すもの

- `AsyncSnapshot`・`Retry` の使用。

### 状態の閉じ方

- `AsyncView<T>`：closure と `Callback` の field。constructor と値渡しの builder だけで書く。対象外。

### 試験

- e2e は R31（`async_tests::async_view_shows_loading_then_ready`、`async_tests::async_view_fallback_retries`）。

### 確認（tier a）

```
cargo fmt -p kivact-ui
cargo check -p kivact-ui --lib
cargo clippy -p kivact-ui --lib -- -D warnings
cargo xtask check --no-write --crate kivact-ui
```

### 完了条件

- `rg -n '\bBinding\b|IntoBinding|into_binding|\bNotifier\b|\bReactive\b|IntoReactive|AsyncSnapshot|\bRetry\b|Into<[\w:]*Callback<\(\)>>' crates/kivact-ui/src` が何も返さない（`"Retry"` の文字列と doc の英単語を除く）。

## R30：5 つの kit と kit の例の呼出し元を直す

- **wave / lane / 前提**：W5a / 鎖 / R29。
- **所有**：`crates/kivact-ui-basic/`、`crates/kivact-ui-soft/`、`crates/kivact-ui-pop/`、`crates/kivact-ui-game/`、`crates/kivact-ui-creative/`（各 `src/` と、T15 の後に kit の crate に残る `tests/`。`dev/tests/kivact-kit-tests` に移った kit の試験は R31 が直す）、`examples/kivact-ui-basic/`、`examples/kivact-ui-soft/`、`examples/kivact-ui-pop/`、`examples/kivact-ui-game/`、`examples/kivact-ui-creative/`。
- **目的**：R20〜R25 の公開 API の変更に kit と kit の catalog を合わせる（W6 から kit は compile の対象から外れるが、W5 では compile する）。

### 手順

1. script を当てる。
   ```
   P="crates/kivact-ui-basic/*.rs crates/kivact-ui-soft/*.rs crates/kivact-ui-pop/*.rs crates/kivact-ui-game/*.rs crates/kivact-ui-creative/*.rs examples/kivact-ui-basic/*.rs examples/kivact-ui-soft/*.rs examples/kivact-ui-pop/*.rs examples/kivact-ui-game/*.rs examples/kivact-ui-creative/*.rs"
   git ls-files $P | xargs perl -0pi target/rework/r_sweep.pl
   git ls-files $P | xargs perl -pi target/rework/r_handler_sig.pl
   git ls-files $P | KV_ROOT=kivact::state:: xargs perl -0pi target/rework/r_handler_body.pl
   ```
2. `crates/kivact-ui-game/src/surface.rs` の `let clock = frame_loop(move |frame| time.set(frame.now.as_secs_f64())).while_present(node_ref);` を次にする。
   ```rust
   let presence = node_ref.presence();
   let clock = frame_loop(move |frame| time.set(frame.now.as_secs_f64()))
       .while_present(move || presence.get() == kivact::view::Presence::Present);
   ```
3. `crates/kivact-ui-creative/README.md` の `Binding` の 2 か所を `Signal` にする（`rg -n Binding crates/kivact-ui-creative/README.md`）。`crates/kivact-ui-basic/README.md` の `use_context` と `IntoSelection` の記述は、`use_context::<T>()` が `T` を返す形に合わせて例のコードの `.get()` を消す。
4. F0〜F9 で `cargo check` を通す。kit の `tests/consumer` の `.update(..)` の戻り値を使う箇所は F3。

### 消すもの

- kit の `Binding`・`IntoBinding`・`Reactive`・`Notifier`・`Into<..Callback<()>>`。

### 状態の閉じ方

- 状態の型の変更は無い。

### 試験

- 各 kit の `tests/consumer` を変えずに（掃引の機械的な変更だけで）通す。

### 確認（tier a）

```
cargo fmt -p kivact-ui-basic -p kivact-ui-soft -p kivact-ui-pop -p kivact-ui-game -p kivact-ui-creative
cargo check -p kivact-ui-basic -p kivact-ui-soft -p kivact-ui-pop -p kivact-ui-game -p kivact-ui-creative --all-targets
cargo check -p kivact-ui-basic-catalog -p kivact-ui-soft-catalog -p kivact-ui-pop-catalog -p kivact-ui-game-catalog -p kivact-ui-creative-catalog
cargo clippy -p kivact-ui-basic -p kivact-ui-soft -p kivact-ui-pop -p kivact-ui-game -p kivact-ui-creative --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-ui-basic
cargo xtask check --no-write --crate kivact-ui-soft
cargo xtask check --no-write --crate kivact-ui-pop
cargo xtask check --no-write --crate kivact-ui-game
cargo xtask check --no-write --crate kivact-ui-creative
```

### 完了条件

- `rg -n '\bBinding\b|IntoBinding|into_binding|\bNotifier\b|\bReactive\b|IntoReactive|Into<[\w:]*Callback<\(\)>>|ActionMode' crates/kivact-ui-basic crates/kivact-ui-soft crates/kivact-ui-pop crates/kivact-ui-game crates/kivact-ui-creative examples/kivact-ui-*` が何も返さない（doc の英単語を除く）。

## R31：devtools・test・試験・bench・例・雛形を直し、新しい E2E を足す

- **wave / lane / 前提**：W5a / 鎖 / R30。
- **所有**：`crates/kivact-devtools/`、`crates/kivact-test/`、`dev/tests/`（`dev/tests/kivact-public-api`、`dev/tests/kivact-kit-tests`、`dev/tests/gpu-interop-producer`。`dev/tests/docs` は W5 では compile しないので除く）、`dev/bench/`、`examples/kivact/`、`templates/`。
- **目的**：残りの crate を R20〜R25 に合わせ、新しい公開 API の契約の E2E を足し、workspace 全体を compile できる状態に戻す。

### 手順

1. script を当てる。
   ```
   P="crates/kivact-devtools/*.rs crates/kivact-test/*.rs dev/tests/kivact-public-api/*.rs dev/tests/kivact-kit-tests/*.rs dev/tests/gpu-interop-producer/*.rs dev/bench/*.rs examples/kivact/*.rs templates/*.rs"
   git ls-files $P | xargs perl -0pi target/rework/r_sweep.pl
   git ls-files $P | xargs perl -pi target/rework/r_handler_sig.pl
   git ls-files $P | KV_ROOT=kivact::state:: xargs perl -0pi target/rework/r_handler_body.pl
   git ls-files 'crates/kivact-test/*.rs' | xargs perl -0pi target/rework/r_driver.pl
   ```
2. `crates/kivact-devtools/src/session.rs` の `let main = self.inspected.update(|i| i.keep_window(&windows));`（F3）を次にする。
   ```rust
   let mut main = None;
   self.inspected.update(|i| main = Some(i.keep_window(&windows)));
   let main = main.expect("invariant: the session's inspected signal lives as long as the session");
   ```
3. 4 章 4.6 の表の試験の書換え・削除・追加を、表のとおりに行う。
4. F0〜F9 で `cargo check --workspace --all-targets` を通す（kivact-docs は W5 から W11 まで compile の対象外。docs の code block は直さない）。
5. kivact-runtime の unit 試験（R20〜R25 で走らせなかったもの）を走らせる。

### 消すもの

- 試験の `ActionMode`・`ActionOutcome`・`Callback::bind`・`Callback::to_fn`・`Signal::notify`・`QueueLatest` の使用。

### 状態の閉じ方

- 状態の型の変更は無い。

### 試験

- 4 章 4.6 の表。

### 確認（tier a）

```
cargo fmt --all
cargo check --workspace --all-targets
cargo check -p kivact-kit-tests --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask check --no-write
KIVACT_TEST_MIRROR=off cargo test -p kivact-runtime -p kivact-i18n
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- task runtime setup_functions diagnostics focus node_ref component_macro lifecycle windows app_handle async_tests binding_update reactive_props flows
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_
```
（`cargo xtask check --no-write` は workspace 全体の静的検査で、試験を走らせない。全体の試験は checkpoint が行う。）

### 完了条件

- 設計書 5.12 節のうち本付属書が消す名前（`Binding`、`IntoBinding`、`IntoReactive`、`Reactive`、`Notifier`、`notifier`、`ActionOutcome`、`ActionMode`、`Retry`、`AsyncSnapshot`）が `rg -w` で `crates/ examples/ templates/ dev/` に識別子として残っていない（`"Retry"` の文字列、`PointerObservation::Reactive` と `view/state.rs` の variant `Reactive`、`dev/xtask` の局所の型 `Binding`、arvane-native の private な `Notifier` を除く）。
- `constant_value` が 4 章 4.7 の許す場所の外に無い（付属書 C の `#[derive(Prop)]` の unit が終わった後に確かめる。R31 の時点では kit・ui・interaction に残っていてよい）。
- 上の命令がすべて成功する。

## 4. 他の付属書との接点

### 4.1 本付属書が作る名前（他の付属書が使う）

| 名前・署名 | path（facade） | unit |
|---|---|---|
| `pub struct Signal<T>`（`SignalRepr::{Source, Lens}`。lens も同じ型） | `kivact::state::Signal` | R20 |
| `binding<T: PropValue, R: HandlerResult>(value: impl IntoValue<T>, set: impl Fn(T) -> R + 'static) -> Signal<T>` | `kivact::state::binding` | R20 |
| `Signal::project(..) -> Signal<U>`（`#[doc(hidden)]`、`bind!` の展開先）、`Signal::optional(self) -> Signal<Option<T>>` | — | R20 |
| `Signal::update(&self, f: impl FnOnce(&mut T))`、`Var::update(&self, f: impl FnOnce(&mut T))`（戻り値なし。DEC-15） | — | R20 |
| `use_context<T: Clone + 'static>() -> T`、`try_use_context<T: Clone + 'static>() -> Option<T>` | `kivact::state` | R20（core の `provide` の 1 行は R26） |
| `pub trait IntoHandler<E, M>`（sealed、`fn into_handler(self) -> Handler<E>`）。実装：`F: Fn() -> R`（`E = ()`、`M = Nullary`）、`Callback<()>`（`M = Handle`）。payload のある handler は `impl Fn(E) -> R + 'static`（共有の `Callback<E>` は `move \|e\| cb.call(e)`）（設計 5.3、DEC-16） | `kivact::state::IntoHandler` | R21 |
| `pub struct Handler<E>`、`Handler::from_fn`（hidden）、`Handler::call(&self, E) -> Result<Handled, UiError>` | `kivact::custom::Handler`（hidden） | R21、R26 |
| `Callback::<E>::from_handler(handler: Handler<E>) -> Callback<E>`（hidden。実行中の owner に登録） | — | R21 |
| `action(run) -> Action<T, I>`（Latest）、`queued_action(run) -> Action<T, I>`、`Action::on_complete(Fn(Result<T, UiError>) -> R)` | `kivact::task` | R22 |
| `AsyncSource<T>::{state, retry -> Callback<()>, publications (hidden)}` | `kivact::task::AsyncSource` | R22 |
| `Trigger`、`trigger()`、`Owner::trigger(at)`、`Trigger::{track, notify, notify_from_render}` | `kivact::state` | R23 |
| `remote<T: Send + 'static>(initial: T) -> (Signal<T>, Remote<T>)`、`Remote::{set, update}`、`remote_trigger() -> (Trigger, RemoteTrigger)`、`RemoteTrigger::notify`、`clock(every: Duration) -> Value<SystemTime>` | `kivact::task` | R23、R27 |
| `FrameLoop::while_present(self, present: impl IntoValue<bool>) -> Self`（core の写しは消える） | `kivact::motion::FrameLoop` | R23、R27 |
| `Prop<T>`（hidden）：`constant`、`new(impl IntoValue<T>)`、`from_text(impl IntoText)`（`Prop<String>`）、`get`、`get_untracked`、`with`、`fixed`、`PartialEq`、`Debug`。`IntoValue::into_prop`（hidden） | `kivact::custom::Prop`（hidden） | R24、R27 |
| `constant_value!`（hidden） | `kivact::state::constant_value`（hidden） | R24、R27 |
| `kivact_runtime::driver::{Runtime, RuntimeConfig, AppParts, TurnGuard, ExitState, TurnReport, Host, ThreadWaker, Live*, OwnerInfo, Kind, EffectKind, NodeInfo, RecordBits, TaskState, TaskStats, TaskEntry, MotionState, MotionEnv, MotionStats, AnimInfo, AnimKind, RunStatus, spawn_in}` | — | R25 |
| `Owner::setup(api, at)`（旧 `Owner::current`）、`Owner::running(api, at)`（旧 `Owner::ambient`）（runtime の `Owner`） | — | R25 |
| `Cause { tag, event, node }`、`CauseTag`、`TurnRecord`（一つの記録の型）、`TraceWrite`、`TraceRun`、`TraceEdge`、`StageSpan`、`Stage`、`Runtime::last_turn()`、`TurnGuard::cause(Cause)` | `kivact::diagnostics` | R25、R27 |
| `RecordBits::{DEVTOOLS, TEST, TRACE_SINK}`、`Runtime::set_recorder(bits, on)`、`Runtime::is_recording()`、`RuntimeConfig.record` | driver | R08 |
| `Runtime::defer(owner, Box<dyn FnOnce()>)`、`Runtime::has_work()`、`Runtime::turn_requested()`、`Runtime::end_idle() -> TurnReport`、`TurnGuard::{settle, run_deferred}`、`TurnGuard::finish(ops)` | driver | R06 |
| core `src/launch/turn.rs`（`Launch::turn`、`commit_rounds`、`cause_of`） | — | R06、R27 |
| `AppParts { state: Box<dyn Any>, motion_env: MotionEnv }`、`Runtime::{app_state, tasks, motion}`、core `crate::app_state::AppState::of(rt)`（`ViewState::of`・`HostState::of` などは名前と戻りの型を変えない。C の R-REQ-11） | driver | R05 |
| `ReportKind::{Error, Diagnostic}`、`Reported.kind`、`Runtime::reported_since(since: u64) -> Vec<Reported>`（C の R-REQ-9） | `kivact::diagnostics` は C が決める | R04 |

付属書 C の 4.1 節の R-REQ-1〜R-REQ-14 への答え：

| C の ID | 本付属書の答え |
|---|---|
| R-REQ-1〜3 | R21（`IntoHandler`、`Handler`、`Callback::from_handler`）。`Handler` の facade の path は求めのとおり `kivact::custom::Handler`（R26） |
| R-REQ-4・5 | R24・R27（`kivact::custom::Prop`。`custom::Reactive`・`IntoReactive`・`state::__private` を消す） |
| R-REQ-6 | R23。`notify_from_render` は残す |
| R-REQ-7 | R25 が改名し、R26〜R31 の掃引（`r_sweep.pl`）が全 crate の呼出し元を直す。core の 0 引数の `kivact_core::state::Owner::current()` は改名しない（C の `Owner` の統合の範囲） |
| R-REQ-8 | R06 が `Runtime::end_idle()` を作る |
| R-REQ-9 | R04（W2a）が `Runtime::reported_since` を作る |
| R-REQ-10 | 受け入れた。core の turn・activation・`Extensions` の unit（R05、R06、R08、R09）を W5 の鎖の先頭に置いた。W2〜W4 で R が持つ core のファイルは 4.3 節の 3 つだけ |
| R-REQ-11 | R05 は `ViewState::of` などの名前と戻りの型を変えない |
| R-REQ-12 | 決定済み（INC-47）：R20・R26 が行う（4.4 節の 3） |
| R-REQ-13 | R10 |
| R-REQ-14 | R29 |
| 注意（closure の引数の推論） | 決定済み（DEC-16）：payload のある handler は `impl Fn(E) -> R`（4.4 節の 2） |

### 4.2 旧から新への対応（利用者と他の付属書の掃引のため）

| 旧 | 新 | unit |
|---|---|---|
| `Binding<T>`、`impl IntoBinding<T>` | `Signal<T>` | R20 |
| `x.into_binding()`、`IntoBinding::into_binding(x)` | `x` | R20 |
| `let r = s.update(f)` | `s.update(f); let r = s.with_untracked(..)` | R20 |
| `use_context::<T>().get()`、`try_use_context::<T>().map(\|v\| v.get())` | `use_context::<T>()`、`try_use_context::<T>()` | R20 |
| `Callback::bind(e)`、`cb.to_fn()` | `move \|\| cb.call(e)`、`move \|v\| cb.call(v)` | R20 |
| `impl Into<Callback<()>>`（引数） | `impl IntoHandler<(), M>`（関数に `<M>`） | R21 |
| `action(ActionMode::Latest, f)` | `action(f)` | R22 |
| `action(ActionMode::Queue, f)` | `queued_action(f)` | R22 |
| `action(ActionMode::QueueLatest, f)` | 無い（4.5 節） | R22 |
| `ActionOutcome::Succeeded(v)`、`ActionOutcome::Failed(e)` | `Ok(v)`、`Err(e)` | R22 |
| `Retry`、`retry.run()` | `Callback<()>`、`retry.call(())`（`.on_press(retry)`） | R22 |
| `AsyncSnapshot`、`source.snapshot()` | `source.publications()`（hidden） | R22 |
| `Action::close(deadline)` | 無い。`cancel()`、`state()`、`on_complete` | R22 |
| `Notifier`、`notifier()`、`owner.notifier(at)` | `Trigger`、`trigger()`、`owner.trigger(at)` | R23 |
| `signal(())` と `s.notify()` | `trigger()` と `t.notify()`、読む側は `t.track()` | R23 |
| `frame_loop(f).while_present(node_ref)` | `frame_loop(f).while_present(move \|\| presence.get() == Presence::Present)`（`let presence = node_ref.presence();`） | R23、R27 |
| `IntoReactive<T>` | `IntoValue<T>` | R24 |
| `custom::Reactive<T>`、`Reactive::new(x)`、`Reactive::from_text(x)` | `custom::Prop<T>`、`Prop::new(x)`、`Prop::from_text(x)` | R24 |
| `IntoValue<Rc<str>>` | `IntoValue<String>` | R24 |
| `kivact_runtime::Runtime` などの driver の名前 | `kivact_runtime::driver::Runtime` など | R25 |
| `Owner::current(api, at)`、`Owner::ambient(api, at)`（runtime） | `Owner::setup(api, at)`、`Owner::running(api, at)` | R25 |
| `TurnRecord.causes: Vec<String>`、`c.starts_with("Activate")` | `Vec<Cause>`、`c.event == "Activate"` | R25 |
| `TurnRecord.writes`（値を変えた書込みだけ） | すべての書込み、`w.changed` で値を変えたもの | R25 |
| `TurnDetail`、`take_turn_details()` | `TurnRecord`、`last_turn()` | R25 |
| `set_detailed_trace(on)` | `set_recorder(RecordBits::DEVTOOLS, on)` | R08 |

### 4.3 W2〜W4 に R が持つ他の付属書の領域のファイル

| wave | R の unit | ファイル | 他の付属書への求め |
|---|---|---|---|
| W2a | R04 | `crates/kivact-core/src/devtools/console.rs`（`error_entry` の 1 行） | C01（W2b）が `devtools/` を持つので、R04 は W2a に置く（INC-23）。C06 の W4 の編集は R04 の後。lane A の S04 の手順 10 の `match` の直しはこのファイルに及ばないこと |
| W3a | R10 | `crates/kivact-core/src/view/flow/virtual_list.rs`、新規 `crates/kivact-core/src/view/flow/virtual_list/heights.rs` | C・U は W3a にこのファイルを持たない。RC-40 の `Held` は R10 が行う（C の R-REQ-13） |
| W2a | R01 | `crates/kivact-i18n/Cargo.toml`（`kivact-types` の依存を足す） | 付属書 T：`dev/xtask/policy/workspace.toml` の `visible_to` で `kivact-types` が `kivact-i18n` から見えること（今は見える。T が狭める場合は `kivact-i18n` を含める） |

その他の接点：

- 付属書 S：W1a の `Deferred` を R01・R05 が使う。
- 付属書 T：新しい core の module `app_state`（R05）は `module_rules.kivact-core.uses` の key に入れない（入れると `view` から参照できなくなる）。`motion::frame_loop -> view::node_ref` の例外は R27 が消す。xtask の `api-shape` の handler の規則は R21 が「`impl IntoHandler<(), M>` か `impl Fn(E) -> R`」を受けるように直す。T が W1 で `api_shape.rs` を書き換える場合は、R21 の手順 8 と同じ受け入れの規則を入れること。
- 付属書 C：C の鎖の unit は R31 の後。C11 は `Runtime::end_idle()` を使う。C の `#[derive(Prop)]` の unit は `::kivact::custom::Prop::constant(self)` に展開し、kivact-interaction、kivact-ui、5 つの kit、例の `constant_value!` の呼出しを `#[derive(Prop)]` に置き換える（4.7 節の場所を除く）。C09 は macro の handler の props を DEC-16（4.4 節の 2）のとおりにする。
- 付属書 E：R27 の `cause_of` は `HostEvent` の全 variant を並べる。E が variant を足し・消すと R27 の時点の compile で分かる（R27 の手順 7 の規則で直す）。
- 付属書 U・K：W7〜W10 で書き直す ui と kit の部品は、本付属書の名前（4.1 節）と、payload のある handler の props の形（DEC-16、4.4 節の 2）に従う。

### 4.4 設計判断（すべて決定済み）

1. **決定済み：`Runtime.app` を `Deferred` で一度だけ書く（R05）。** core の状態（`ViewState` など）は root の owner の signal を持つので runtime を登録した後でしか作れず、runtime はそれを field に持つ必要がある。この循環は構造上避けられない。決定：`Runtime::new` の中で `get_or_init` で一度だけ書き、読む側は `expect` する（R05 のとおり。今の `ext: OnceCell` と同じ性質を、設計書 6.2 節の許す道具で書く）。親がこの形を受け入れた（`OnceCell` は残さない）。
2. **決定済み（DEC-16）：payload のある handler の型。** rustc は closure の引数の型を `Fn`・`FnMut`・`FnOnce` の bound からだけ推論し、`Callback<E>` は `Fn` を実装できない。そのため：
   - payload の無い handler（`on_press` など、`Callback<()>` の props）は `impl IntoHandler<(), M>`（closure と共有の `Callback<()>` の両方を受ける。R21 の impl はこの二つだけ）。
   - payload のある `Callback<E>` の props と payload のある `on_*`（`on_key`、`on_pointer` など）は `impl Fn(E) -> R + 'static`（`R: HandlerResult`）を受け、共有の `Callback<E>` は `move |e| cb.call(e)` で渡す。macro は内部で `Handler::from_fn` にする（RC-33 の割当ての削減はこちらでも得られる）。
   - R26 の macro の変更はこのとおり。R31 が e2e の `component_macro` に `callback_prop_closure_infers_its_parameter` を足して確かめる。
3. **決定済み（INC-47）：`use_context` の担当は R。** R20 が runtime の `use_context`・`try_use_context` を、R26 が core の `provide` の本体の 1 行（`owner.constant` を消す）を変え、R26〜R31 の掃引が呼出し元の `.get()` を消す。C08 は残り（`provide_direction` の形、`use_direction`、`Owner` の統合）だけを行う。

### 4.5 設計・監査の fix からの小さな違い（決めたこと）

- RC-08：`LoopCell.frames` は残す（`FrameInfo::frame` は「今までに走った回数」で、rate の変更で 0 に戻る `Cadence` の index とは意味が違う）。Animated の成分を `[f64; 4]` の固定の配列にしない（`Transform` の `Animatable` は 17 成分）。代わりに step はその場で `current` を進め、step ごとの `Vec` の割当てをなくす。
- 設計書 5.5 の `ActionMode` の削除で、`QueueLatest`（待ちの run を最新の一つにまとめる queue）は無くなる。使っていたのは e2e の試験だけ。
- RC-41 の `notify_from_render` の削除は行わない（C の R-REQ-6：core が使い続ける）。
- RC-37 の「`done()` を待つ」は足さない（設計書 5.5 の `Action` の API に無い）。終わりは `state()` と `on_complete` で知る。
- RC-15 の `with_state` は足さない（5.5 節に無い）。async の view は hidden の `AsyncSource::publications` で変化を知る。
- RC-06：`Disposed` の `created_at` は debug build と記録中だけ分かる（release では `None`）。
- RC-05：記録は DevTools と同じ一つの型になり、`TurnRecord.writes` は値を変えなかった書込みも `changed: false` で持つ。
- RC-23：書ける行の書込みは、行の key で list の位置を探し（O(N) の探索、list の複製は無い）、その要素だけを `update` で書く。
- RC-07：`with_runtime` は source の読み（user code を実行しない）にだけ使う。

### 4.6 試験の書換え・削除・追加（R31 が行う。場所は `rg -n 'fn <名前>' dev/tests` で探す）

| ファイル | 試験 | 行うこと |
|---|---|---|
| `dev/tests/kivact-public-api/tests/*/runtime.rs` | `small_handles_trigger_var_binding_callback_value` | `let trig = signal(());` を `let trig = trigger();`、memo の中の `trig.get();` を `trig.track();` にする。`let bumped = bump.bind(());` を消し、`bumped.call(());` を `bump.call(());` にする。`let f = setter.to_fn();` を消し、`.on_press(move \|\| f(9))` を `.on_press(move \|\| setter.call(9))` にする。`trig.notify();` はそのまま（`Trigger::notify`）。期待値は変えない |
| `dev/tests/kivact-public-api/tests/*/runtime.rs` | 追加 `writes_to_a_removed_signal_are_ignored` | 下のコード |
| `dev/tests/kivact-public-api/tests/*/setup_functions.rs` | `handle_constructors_create_under_the_setup_owner` | `let trigger = signal(());` を `let trigger = trigger();`、effect の中の `trigger.get();` を `trigger.track();` にする。期待値は変えない |
| `dev/tests/kivact-public-api/tests/*/setup_functions.rs` | `context_is_found_in_setup_and_in_handlers` | `r_sweep.pl` の変更だけ（`.get()` が消える） |
| `dev/tests/kivact-public-api/tests/*/task.rs` | `queue_orders_coalesces_recovers_and_drains` | 名前を `queue_orders_recovers_and_drains` にする。`let latest = action(ActionMode::QueueLatest, ..);` の文全体、`let (fifo, latest)` の `latest`（`let (screen, fifo) = mount(..)`、返す組を `(fifo, text(..))` にする）、`latest.run(1);`、ループの `latest.run(n);`、`assert_eq!(latest.pending_len(), 1, ..)`、最後の `latest_order` の 2 文を消す。`(la, lb)` の `lb` を消す。`matches!(r, kivact::task::ActionOutcome::Succeeded(_))`（掃引の後は `matches!(r, Ok(_))`）を `r.is_ok()` にする。fifo の期待値は変えない |
| `dev/tests/kivact-public-api/tests/*/task.rs` | `action_latest_wins_retry_cancel_and_multiple_done_handlers` | 掃引の後の `let result = match r { Ok(value) => Ok(value), Err(error) => Err(error), };` を `let result = r;` にする。期待値は変えない |
| `dev/tests/kivact-public-api/tests/*/task.rs` | 追加 `remote_writes_from_another_thread_coalesce_into_one_update`、`remote_trigger_notifies_from_another_thread`、`clock_shares_one_timer_per_period` | 下のコード |
| `dev/tests/kivact-public-api/tests/*/diagnostics.rs` | `trace_records_causes_writes_reruns_ops_and_duration` | `t.causes.iter().any(\|c\| c.starts_with("Activate"))` を `t.causes.iter().any(\|c\| c.event == "Activate")` にする。`t.writes` を読む前に `let writes: Vec<_> = t.writes.iter().filter(\|w\| w.changed).collect();` を置き、`t.writes.len()`・`t.writes.iter()`・`t.writes[i]` を `writes.len()`・`writes.iter()`・`writes[i]` にする（値を変えた書込みだけを数える。今の期待値と同じ事実）。後半の `t.writes.iter().all(..)` を `t.writes.iter().filter(\|w\| w.changed).all(..)` にする |
| `dev/tests/kivact-public-api/tests/*/focus.rs` | helper `focus_results` | `.filter(\|c\| c.starts_with("FocusResult"))` を `.filter(\|c\| c.event == "FocusResult")` にする |
| `dev/tests/kivact-public-api/tests/*/node_ref.rs` | `.while_present(target)` を使う試験 | `frame_loop(..).while_present(target)` を `{ let presence = target.presence(); frame_loop(..).while_present(move \|\| presence.get() == kivact::view::Presence::Present) }` の形にする（`presence()` は setup で呼ぶ） |
| `dev/tests/kivact-public-api/tests/*/component_macro.rs` | 追加 `callback_prop_closure_infers_its_parameter` | 下のコード |
| `dev/tests/kivact-public-api/tests/*/{app_handle,async_tests}.rs` | `ActionMode` を使う試験 | 掃引の変更だけ |
| `crates/kivact-runtime/src/reactive/selector.rs` | `remounted_readers_do_not_accumulate_key_entries` | R02 で書換え済み |
| `crates/kivact-i18n/src/cache.rs`、`crates/kivact-core/src/view/flow/virtual_list/heights.rs` | `the_same_tag_reuses_one_cache`、`prefix_set_and_index_at_match_a_plain_sum` | R01・R10 で追加済み |

追加する試験のコード（`task.rs` の先頭の `use` に `use kivact::task::{clock, remote, remote_trigger};` を、`runtime.rs` に `use std::cell::Cell;` と `use std::rc::Rc;` が無ければ足す）：

```rust
// runtime.rs
/// Writes to the state of a removed component are ignored and reported,
/// for `set` and `update` alike.
#[test]
fn writes_to_a_removed_signal_are_ignored() {
    let slot: Rc<Cell<Option<Signal<i32>>>> = Rc::new(Cell::new(None));
    let child_slot = slot.clone();
    let (screen, shown) = mount(move || {
        let shown = signal(true);
        let child_slot = child_slot.clone();
        (
            shown,
            show(shown, move || {
                child_slot.set(Some(signal(1)));
                text("child")
            }),
        )
    });
    screen.get_by_text("child");
    shown.set(false);
    screen.run_until_idle();
    let removed = slot.get().expect("the child was set up");
    removed.set(2);
    removed.update(|v| *v += 1);
    screen.run_until_idle();
    assert_eq!(removed.try_get_untracked(), None);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    let reported = screen
        .trace()
        .iter()
        .flat_map(|t| t.notes.iter())
        .filter(|n| n.starts_with("WriteAfterDispose"))
        .count();
    assert_eq!(reported, 2, "one diagnostic for each write");
}

// task.rs
/// Writes from another thread wake the App and arrive merged: the signal
/// is written once with the latest value and the later edits.
#[test]
fn remote_writes_from_another_thread_coalesce_into_one_update() {
    let (screen, (sender, runs)) = mount(|| {
        let (value, sender) = remote(0u32);
        let runs = var(0u32);
        effect(move || {
            value.get();
            runs.update(|r| *r += 1);
        });
        ((sender, runs), text(move || format!("v {}", value.get())))
    });
    screen.find_by_text("v 0");
    let before = runs.get();
    let worker = sender.clone();
    std::thread::spawn(move || {
        for n in 1..=100 {
            worker.set(n);
        }
        worker.update(|v| *v += 1);
    })
    .join()
    .expect("the writing thread ends");
    screen.find_by_text("v 101");
    assert_eq!(runs.get(), before + 1, "a hundred and one writes arrive as one update");
}

/// A trigger notified from another thread wakes the App; notifications
/// before a turn are one.
#[test]
fn remote_trigger_notifies_from_another_thread() {
    let (screen, (sender, count)) = mount(|| {
        let (changed, sender) = remote_trigger();
        let count = signal(0u32);
        effect(move || {
            changed.track();
            count.update(|c| *c += 1);
        });
        ((sender, count), text(move || format!("n {}", count.get())))
    });
    screen.find_by_text("n 1");
    let worker = sender.clone();
    std::thread::spawn(move || {
        worker.notify();
        worker.notify();
    })
    .join()
    .expect("the notifying thread ends");
    screen.find_by_text("n 2");
    screen.run_until_idle();
    assert_eq!(count.get_untracked(), 2);
}

/// Clocks of one period share one value and one timer, which ticks once per period.
#[test]
fn clock_shares_one_timer_per_period() {
    let (screen, (a, b, c, ticks)) = mount(|| {
        let a = clock(ms(1000));
        let b = clock(ms(1000));
        let c = clock(ms(500));
        let ticks = var(0u32);
        effect(move || {
            a.get();
            ticks.update(|t| *t += 1);
        });
        ((a, b, c, ticks), ())
    });
    screen.run_until_idle();
    assert_eq!(a, b, "one value for one period");
    assert_ne!(a, c);
    let before = ticks.get();
    for _ in 0..3 {
        screen.advance(ms(1000));
    }
    assert_eq!(ticks.get(), before + 3);
}

// component_macro.rs
#[component]
fn signed_picker(on_pick: Callback<i32>) -> impl IntoView {
    button("pick").on_press(move || on_pick.call(-3))
}

/// A closure passed to a `Callback<E>` prop gets its parameter type from
/// the prop, so it can call methods on it.
#[test]
fn callback_prop_closure_infers_its_parameter() {
    let (screen, seen) = mount(|| {
        let seen = signal(0);
        (seen, signed_picker(move |n| seen.set(n.abs())))
    });
    screen.get_by_role(Role::Button, "pick").click();
    assert_eq!(seen.get_untracked(), 3);
}
```

### 4.7 `constant_value!` を使ってよい場所（設計書 5.12 節）

`#[doc(hidden)]` の定義と、`#[derive(Prop)]` を使えない crate（facade の `kivact` と `kivact-core` の下にある crate、または自分自身）だけ：

- `crates/kivact-runtime/src/reactive/props.rs`（定義と基本の型）
- `crates/kivact-i18n/src/{date.rs,format.rs,locale.rs,messages.rs}`
- `crates/kivact-style/src/{color/value.rs,draw/image.rs,icon.rs,length.rs,sheet/fields.rs,theme/value.rs}`
- `crates/kivact-core/src/view/node_ref.rs`
- `crates/kivact-core/src/state.rs`（hidden の再公開の 1 行）

kivact-interaction、kivact-ui、5 つの kit、例、試験の呼出しは、付属書 C の `#[derive(Prop)]` の unit が置き換える（4.3 節）。

## 5. 自己点検

### 5.1 所見の割当て

| 所見 | unit |
|---|---|
| RC-01 | R06（`end_idle`、idle の条件の `has_work`、`report_now` を private の `exit_report` に） |
| RC-02 | R06（`Runtime::defer`、`has_work`、`Pending` の削除、activation の列の削除） |
| RC-03 | R06（`TurnGuard::settle` 一つ、`launch/turn.rs`）、R08（`record_stage` は記録中だけ）、R22（`check_draining` の削除） |
| RC-04 | R05 |
| RC-05 | R08（記録の opt-in）、R25（一つの記録の型、`Cause`、`describe` の削除は R27） |
| RC-06 | R09 |
| RC-07 | R03 |
| RC-08 | R07 |
| RC-14 | R01 |
| RC-15 | R22、R29（async view） |
| RC-16 | R22 |
| RC-17 | R04 |
| RC-23 | R10（`virtual_list`）、R26（書ける行） |
| RC-24、RC-25、RC-26 | R02 |
| RC-30 | R25、R27・R31（`r_driver.pl`） |
| RC-31 | R20（runtime）、R26（`provide` の 1 行）。INC-47 で決定済み |
| RC-32 | R24、R26・R27（macro と core） |
| RC-33 | R21、R26（macro） |
| RC-34 | R23、R27 |
| RC-35 | R22、R25（`spawn_in`）、R26（`ActionFn`・`BlockingWork` を facade から外す） |
| RC-37 | R22 |
| RC-38、RC-39 | R04 |
| RC-41（runtime・i18n） | `mount_revision`：R04。`report_now`：R06。`Owner` の改名：R25。`notify_from_render`：残す（4.5 節）。i18n の依存：`constant_value!` のために runtime への依存を残す（4.7 節。crate の移動は無い） |
| API-02 | R20 |
| API-03 | R21、R26（`on_press` が object を作らない）。DEC-16 |
| API-06（runtime） | R24、R27（`Prop` を hidden で公開、`constant_value!` を hidden に）。derive は付属書 C |
| API-11（`Callback`） | R20 |
| API-15 | R20（DEC-15：戻り値なし） |
| USR-02 | R23（`remote`、`remote_trigger`）、R31（E2E） |
| USR-03 | R23 |
| KIT-17 | R23（`clock`）、R31（E2E） |
| PERF-9 | R08、R09 |

### 5.2 設計の判断を実装者に残していないか

- 実装者が選ぶ箇所は無い。条件で分かれる手順は、compile の誤りの種類（3 章の F0〜F9、R09 の手順 7、R27 の手順 7）だけで、それぞれ直し方を一つに決めた。
- 親が決める問いは残っていない（4.4 節の 1〜3 はすべて決定済み）。

### 5.3 同じ wave の二つの unit が同じファイルを持たないか

- W2a：R の lane は一つ（R01→R02→R03→R04 の順）。R04 だけが core の `devtools/console.rs` を持つ。`devtools/` を持つ C01 は W2b（4.3 節、INC-23）。
- W3a：R07（runtime の `motion/`）と R10（core の `virtual_list`）。同じ lane で順に実行し、ファイルは重ならない。
- W3b・W4・W4b：R の unit は無い。
- W5a：鎖は一つの lane で、R05、R06、R08、R09、R20〜R31 の順に一つずつ実行する。付属書 C の鎖の unit（C07〜C18）は R31 の後。
