# Kits audit (KIT)

Scope: crates/kivact-ui-{basic,pop,game,creative,soft}, examples/kivact-ui-*, docs/for-kivact-ui-user.
Method: read + grep only. "Unused" judgements use the catalogs, kit tests, docs and `dev/tests` only. The four downstream repos (the four downstream apps) were NOT available, so every "no user" claim below is "no user in this repo"; the owner must veto anything those four use.

## 0. Summary

1. The five kits do not share a skeleton. `theme(mode)` is uniform, but ThemeOptions exists in 2 of 5, `Tokens` is flat in 4 and nested in soft, `ColorName` is a hand-mirror of `Tokens` in all 5, variant words are `Variant`/`Look`/`Appearance`/`Tone`/`Hue`, and recipes live in files not named `recipes_*`. Section 3 gives one skeleton a weak implementer can apply.
2. Duplication that should be deleted or hoisted: pop and game are near twins (Slant, Striped, ButtonTone, 20 same-named recipes files, 4 `XHue(Value<Hue>)` newtypes in pop); civil-date math exists 3 times (soft timestamp, kivact-i18n, devtools-protocol); sRGB luminance/contrast exists 4 times; two syntax-token taxonomies (basic CodeTone, soft TokenKind); two sidebars (basic 1.2k, soft 1.5k lines); five catalog shells and two copy-pasted scroll-spies.
3. Implicit state: soft `MobileMachine` mirrors the caller's `open` binding in a second place (`Sync` event); creative has four copies of `observe_inputs`, an effect that repairs gesture state when inputs vanish; game kpi runs a Machine fed by an effect to build a latch.
4. Soft is a 28k-line kit with 334 public items. About 25 public fns have no user in catalog/tests/docs, and it re-implements core pieces (tab strip with overflow, aspect_ratio, own month names that bypass i18n). It needs a "what is theme, what is a domain part, what belongs in core" cut.
5. Process residue: 210 `xtask-allow(state-fields)` comments in kit code (96 in soft, almost all the text "option: ..."), a "Migrating from the previous release" table in all 5 READMEs for a release that was never published, README "Filled recipes" lists that copy `recipes.rs`, 110 en/jp doc files that must stay in sync by hand.

Target shape: five kits, each = `Tokens` + `ColorName` (macro generated) + `theme`/`theme_with` + `recipes` grouped by `UiRecipes` entry + typed variants named `<Component>Variant` + domain parts in clearly named modules. Shared mechanics (recipe/icon identity cache, hue-as-variant, Slant/Striped markers, luminance/contrast, date math) move into core. One `examples/catalog-shell` crate holds `Page`/`Example`/snippet extraction/shell, and each of the five catalogs is only a list of pages plus a theme function.

## 1. Current structure map

| crate | src lines (tests) | pub items | role | depends on |
|---|---|---|---|---|
| kivact-ui-basic | 14.5k (2.7k) | 122 | shadcn Lyra look + chart (1.5k), plot (2.9k), sidebar (1.2k), questionnaire, code_block, chat recipes | kivact only |
| kivact-ui-pop | 4.8k (0.9k) | 36 | calm slate look + timeline, chips, notification menu, agenda, thread | kivact only |
| kivact-ui-game | 3.9k (0.9k) | 34 | hard-edged console look + kpi, hud/halo/cut_corner, icon_menu | kivact only |
| kivact-ui-creative | 24.2k (4.3k) | 127 | dense editor look + dock 2.5k, editors 9.5k, color_wheel 1.3k, numeric 1.7k, outliner 1.3k, gradient, bezier, pie, range, audio | kivact only |
| kivact-ui-soft | 28.1k (5.0k) | 334 | Astryx Neutral look + app_shell, top_nav, side_navigation, tab_list, markdown 1.3k, rich_blocks 1.4k, chat tools, timestamp, lightbox, layout/stack/grid | kivact only |
| examples/kivact-ui-{basic,pop,game,creative,soft} | 8.0k / 5.0k / 5.0k / 7.5k / 3.0k | - | catalog apps + `tests/catalog.rs` | kit + kivact |
| docs/for-kivact-ui-user | 15k lines, 55 en + 55 jp files | - | tutorial + how-to per kit | - |

Who depends on the kits: `crates/kivact-devtools` depends on `kivact-ui-basic` (app.rs:35, panels/performance.rs:7), `dev/tests/kivact-public-api` and `dev/tests/docs`, the five catalogs. Nothing else.

Boundary judgement:
- Five kits as five crates is a real boundary. No kit depends on another, all depend only on the public `kivact` facade, the input is `ThemeMode` (and options) and the output is a `Theme` plus builder types. Keep five crates. Do not merge pop and game: despite the twin structure, the values differ in every recipe (diff of same-named recipe files is 60-100% of lines).
- Fake boundary: `kivact-devtools` (a dev tool) links the whole `kivact-ui-basic` including chart and plot to draw a few panels (KIT-18).
- Missing boundary: basic's `plot` (2.9k lines, no shadcn counterpart, consumed by a downstream analyzer app) lives in a theme kit although its inputs are just `PlotModel` and the theme palette (KIT-15).
- Missing boundary: five catalogs each own a shell, a page registry and scroll-spy (KIT-12).

## 2. Findings

### KIT-01 Soft: mobile-nav Machine mirrors the caller's open binding
- kind: STATE
- where: crates/kivact-ui-soft/src/mobile_nav.rs:47-250 (`AppShellMobileState`, `MobileMachine`, `configured`), app_shell.rs:84-90
- evidence: state is `Desktop | Mobile{open}`; `Mobile{open}` copies `Binding<bool>`. The machine needs `Sync(bool)` (fed from `open.get()`), `SetOpen`, `Toggle` and an Effect that writes back to the binding, plus `MobileMachine{open}` re-reading the binding in `transition`. Two places own "is the drawer open" and an event exists only to reconcile them.
- fix: delete the Machine. `mobile: Value<bool>` = memo(viewport width < breakpoint) is the only derived fact; `open: Binding<bool>` is the single owner of openness. `is_open() = mobile.get() && open.get()`; `show/hide/toggle` write the binding (no-op when `!mobile`). The "leave mobile closes the drawer" rule is one `on_change(mobile)` that sets `open=false` when mobile turns false. `AppShellMobile` becomes `{mobile: Value<bool>, open: Binding<bool>, toggle: NodeRef, drawer: NodeRef, auto_toggle: bool}`. About 110 lines removed.
- breaking: no (types are internal; `AppShellMobile` methods keep their names)   kpi: K2   size: S
- confidence: high

### KIT-02 Creative: four copies of `observe_inputs`, an effect that repairs gesture state
- kind: STATE, DUP
- where: crates/kivact-ui-creative/src/editors/time/view.rs:221-247, arrangement/view.rs:293-325, graph/view.rs:283-310, node/view.rs:255-300; editors/shared/targets.rs:178 (`effect(|| controller.send(Synchronize(source())))`); picker.rs:715-775 (3 effects feeding `ColumnMachine`)
- evidence: each editor has an `effect` that reads focus, visibility, availability and the data, computes `missing` (does the item being dragged still exist) and calls `state.press.inputs_changed(focused, visible, enabled, !missing, cancel)`. This is "an effect that repairs state": the drag state can be invalid because the model changed under it, and a second system notices it.
- fix: make a gesture hold only ids and look them up at every pointer event (an id that is gone makes the event cancel the gesture: one `match`, in the gesture machine). Put the shared part in `editors/shared/gesture.rs` as `fn inputs_ok(focus, visible, availability) -> Value<bool>` (memo) fed to the gesture machine as an event, and delete the per-editor `missing` closures. The same rule for picker's `follow_active/follow_columns`: `ColumnMachine` takes the `lists` and `query` as machine inputs (`machine(.., || ColumnInput{lists, query}, ..)`), not three effects pushing events. Target: zero `effect(` in crates/kivact-ui-creative (now 10).
- breaking: no   kpi: K3 (each effect runs per data change per editor)   size: M
- confidence: high for time/arrangement (read end to end), medium for node/graph (same shape, read the head)

### KIT-03 Soft timestamp: third copy of date math, English month names bypass i18n, no-op option, Rc<RefCell> builder
- kind: DUP, CONTRA, DEAD, API
- where: crates/kivact-ui-soft/src/timestamp.rs:462-500 (`days_from_civil`, `civil_from_days`, MONTHS, WEEKDAYS table starting "Thu"), :171-173 (`copyable`), :215-310 (`Rc<RefCell<TimestampSettings>>`); crates/kivact-i18n/src/date.rs:64-100 and crates/kivact-devtools-protocol/src/time.rs:17 hold the same algorithm.
- evidence: `calendar_text` hardcodes "January".."December" and "AM/PM" and `"{month_name} {day}, {year} at {clock}"`, while `kivact::i18n` has `Locale::month_name`, `weekday_name`, `format_date`, `format_time`, `hour_cycle`. A Japanese App gets English dates from this kit. `TimestampEntry::copyable` is documented "has no effect". Builder methods mutate an `Rc<RefCell<TimestampSettings>>` to avoid storing fields (a Rust-ism for no gain). `timestamp` creates a Machine + memo + `SystemTime::now()` per instance.
- fix: (1) `TimestampValue` becomes `kivact::i18n` `Instant` (add `CalendarDate::from_unix_days`/`TimeOfDay` if missing: core owns date math; delete lines 462-500 and `parse_iso` if i18n has a parser, else move `parse_iso` into i18n). (2) all formatting goes through `Locale::format_date/format_time` with `DateStyle`; `TimestampFormat::{SystemDate,SystemDateTime,SystemTime,UnixSeconds}` stay as ISO strings built from `CalendarDate`. (3) delete `copyable`. (4) store settings as plain `Value` fields on `Timestamp` and pass them into the `into_view` closure; delete `Rc<RefCell>`. (5) one shared 1-second clock `Value<Instant>` in core (see KIT-17) instead of a machine per Timestamp.
- breaking: yes (TimestampValue, TimeZone, InstantFormat)   kpi: K2, K3   size: M
- confidence: high

### KIT-04 Pop/Game: `Hue`-wrapper newtypes and duplicated variant markers
- kind: DUP, API
- where: crates/kivact-ui-pop/src/recipes_badge.rs:22 (`BadgeHue(Value<Hue>)`), recipes_items.rs:9 (`ItemHue`), recipes_metrics.rs:12 (`StatHue`), recipes_small.rs:37 (`ProgressHue`), recipes_calendar.rs:14 (`WeekendHue`, an `Rc<dyn Fn>`), recipes_small.rs:34 (`impl Variant for Hue`), button.rs:11 (`ButtonTone`), button.rs:28 and game/recipes_controls.rs:25 (`Slant`), pop/recipes_small.rs:22 and game/recipes_small.rs:20 (`Striped`), game/recipes_controls.rs:15 (`ButtonTone { Dark }`, an enum with one case)
- evidence: four identical newtypes `struct XHue(pub Value<Hue>); impl XHue { fn new(..) } impl Variant for XHue {}` exist only because a component takes one variant per Rust type and the hue must be reactive. `Hue` itself also implements `Variant`, so a user has two ways: `.variant(Hue::Pink)` and `.variant(BadgeHue::new(Hue::Pink))`. `Slant` and `Striped` are identical empty marker structs defined in both pop and game; a user who switches the theme from pop to game with `.variant(pop::Slant)` silently gets a plain button, and vice versa.
- fix: (core, generic) let `.variant(..)` accept `impl IntoValue<V>` and let a recipe read `look.variant::<Hue>()` (the `Look` already holds a `Value`-resolved snapshot). Delete the four `XHue` newtypes and `WeekendHue`'s separate type (weekend tone = a plain `Fn(Weekday)->Option<Hue>` setter on the calendar's variant). Move `Slant` and `Striped` into `kivact::ui` as generic variant markers that any theme may honour (`kivact::ui::Slant`, `kivact::ui::Striped`); delete both kits' copies. `game::ButtonTone` (one case) becomes `kivact::ui::ButtonVariant::Dark`? No: add `.tone(Tone)` use; if that does not fit, delete it and let game use a plain `ButtonVariant::Primary` look (game recipes already treat Dark as navy). Decision: delete `ButtonTone` in game, make `ButtonVariant::Secondary` navy.
- breaking: yes (kit variant types; core gets two markers and an IntoValue variant)   kpi: -   size: M
- confidence: high

### KIT-05 Two syntax-highlight taxonomies and two color sets
- kind: DUP
- where: crates/kivact-ui-basic/src/code_block/{rust,lines,highlight,view}.rs (467 lines, `CodeTone`, `CodeColors`, only Rust, `rust_source`); crates/kivact-ui-soft/src/code_syntax.rs (240 lines, `TokenKind` 13 cases, `CodeLanguage::custom(tokenizer)`, `SyntaxColors`); core `kivact::ui::code_block(..).highlight(content, f)` (crates/kivact-ui/src/code_block.rs:89) is the actual seam.
- evidence: both kits tokenize into ranges, map to colors and call the same core callback. They disagree on kind names (`Macro/Lifetime/Attribute` vs `Tag/Property/Variable/Constant`).
- fix: core owns `kivact::ui::SyntaxKind` (superset of the 13 + Macro + Lifetime) and `trait Highlighter`-free fn type `fn(&str) -> Vec<(Range<usize>, SyntaxKind)>`; `UiTokens.code_block.syntax: SyntaxColors` is filled by each theme. Basic's Rust tokenizer (rust.rs, 234 lines) becomes the only built-in language in core (or stays in basic as a `rust_tokens()` fn if the owner wants core language-free). Delete `CodeTone`, `CodeColors`, `TokenKind`, `SyntaxColors` duplicates; one set of colors in `UiTokens`.
- breaking: yes   kpi: K6 (small)   size: M
- confidence: medium (I did not read core code_block's color handling)

### KIT-06 Kits do not share a structure
- kind: STRUCT, API
- where: see table in section 3
- evidence (deviations found):
  - ThemeOptions/theme_with only in game (`{density}`) and creative (`{accent, density}`). pop has none although its look has hue/density in tokens; basic and soft have none. Docs promise "every kit works the same way" (docs/.../README.md).
  - `Tokens`: flat colors in basic/pop/game/creative, nested (`tokens.colors.text_primary`, `tokens.spacing.s3`, `tokens.radius.element`) in soft with 3 parallel shapes (`Tokens`, `Colors`, `HueColors/DataColors/SyntaxColors`, color.rs 660 + color_name.rs 183 + tokens.rs 512). Pop stores `mode` in `Tokens` and game `mode` + `density`, a second source of truth next to `Theme::mode()`.
  - Variant vocabulary: basic 15 `*Variant` + 5 `*Look`; soft 18 `*Variant` + 10 `*Appearance` + 5 `*Look` + 1 `*Style`; pop 7 `*Hue/Tone`, 1 Variant; game prefixes the kit name (`GameInputGroupVariant`, `GameCellVariant`) while its own `AvatarVariant` does not; pop has `PopBubble`. Three words (Variant, Look, Appearance) for one idea.
  - Recipe code lives outside `recipes_*`: basic `chat.rs`, `message_scroller.rs`, `list_popup.rs`, `theme_access.rs`; soft `soft_list.rs`, `soft_tree.rs`, `recipe_style.rs`; pop `button.rs`, `choice.rs`, `form.rs`, `tabs.rs`; game `button.rs`, `choice.rs`, `choice_style.rs`, `display.rs`, `table.rs`.
  - `lib.rs` module docs: pop/game/creative describe usage; soft has the one line "Expose Astryx Neutral as an official Kivact UI kit." and 335 re-exports with no grouping.
  - Message-key prefix: basic `basic.`, pop `pop.` plus unprefixed core keys, soft `soft.`, game 1 key, creative `creative.`; README claims "prefixed with `pop.`".
- fix: adopt the skeleton in section 3. Rename every kit appearance enum to `<Component>Variant`, delete `*Appearance`/`*Look` as public names (keep `Look<S>` of core), remove kit-name prefixes (`game::InputGroupVariant`), move every recipe fn into `recipes_<group>.rs`.
- breaking: yes (renames)   kpi: -   size: L (mechanical)
- confidence: high

### KIT-07 `ColorName` is a hand-written mirror of `Tokens` in every kit
- kind: DUP
- where: crates/kivact-ui-{basic,pop,game,creative}/src/color.rs (143/80/68/136 lines), soft color_name.rs (183) + color.rs (660)
- evidence: each is `enum ColorName { Background, Surface, ... }` + `impl ThemeColor { match self { Background => p.background, ...} }`. Adding a token needs three edits (field, enum case, match arm) per kit; pop has a fourth list (`Hue`).
- fix: core macro `kivact::style::theme_colors!(Tokens { Background => background, SurfaceMuted => surface_muted, .. })` that expands to the enum + `ThemeColor` impl (decl-macro, ~30 lines in kivact-style). Each color.rs shrinks to the list. Alternative (better if soft's nested tokens allow): store colors in `[Color; N]` indexed by the enum so `Tokens` has no duplicated field list. Decision: the macro (smaller change).
- breaking: no   kpi: -   size: M
- confidence: high

### KIT-08 Recipe-table and icon-set identity caches are copy-pasted ten times; icon name lists are double-entered
- kind: DUP, STATE
- where: `recipes()` with `OnceLock` + `#[expect(clippy::disallowed_types)]` in all five (basic recipes.rs:31, pop recipes.rs:36, game recipes.rs:16, creative recipes.rs:28, soft recipes.rs); `icons::set()` the same in all five (pop icons.rs:328, soft icons.rs:112, ...); pop icons.rs:4 `NAMES` and :107 `IMAGES` and game icons.rs:4 `NAMES`.
- evidence: pop `set()` does `NAMES.iter().filter_map(|n| bytes(n).map(..))`, a linear scan per name, and silently drops a name that exists in NAMES but not in IMAGES (or the reverse). 25 of 314 SVG files are byte-identical across kits (md5), e.g. Tabler icons in both pop and game.
- fix: core: `UiRecipes::shared(build: fn() -> UiRecipes) -> UiRecipes` and `IconSet::shared(build: fn() -> IconSet) -> IconSet` (cache keyed by the fn pointer inside core, so kits have no OnceLock and no `expect(disallowed_types)`). Core macro `kivact::style::icons!(dir = "assets/icons", ["home", "bell", ..])` expanding to `[(name, include_bytes!(concat!(dir, "/", name, ".svg")))...]` so a name is written once. Shared Tabler icons: one `assets/tabler/` under a `kivact-ui` feature-free module? Decision: do not share files across kits (K6 gain is ~10 KB); only the macro.
- breaking: no   kpi: K1 (build per theme avoided), K6 (-10 KB)   size: S
- confidence: high

### KIT-09 Soft reimplements core pieces and carries public API with no user in the repo
- kind: DEAD, DUP, STRUCT
- where: crates/kivact-ui-soft/src/{container_reveal,media_query,soft_chat_tools,rich_text,link,scroll_observer,tab_list,aspect_ratio}.rs
- evidence: public fns that appear only in `lib.rs` re-export, their definition and the README (zero hits in catalog, kit tests, dev/tests, docs): `reveal_container`, `reveal_content`, `RevealContent/Force/Visibility` (container_reveal.rs, 124 lines), `viewport_below`, `viewport_from`, `container_below`, `is_compact` (media_query.rs, 62 lines), `streaming_text`, `use_streaming_text`, `StreamSpeed`, `chat_token`, `chat_tokenized_text`, `rich_text_toolbar`, `rich_text_view`, `use_scroll_state`, `use_scroll_overflow`, `use_overflow`, `link_with`, `icon_style`, `grid_span`, `flex_stack`, `layout_footer`. `aspect_ratio` (soft aspect_ratio.rs, 90 lines, shape/fit options) exists next to `style().aspect_ratio(..)` which basic's catalog uses: two ways. `tab_list` (966 lines, own Machine `TabRoving`, own tab styling, overflow menu) parallels `kivact::ui::tabs` + recipes, while the other four kits restyle core tabs through recipes only. 150 of 334 soft pub names have no mention in catalog/docs/dev-tests (many are types returned by builders, so the true dead count is the fn list above, about 25).
- fix: delete the fn list above (their types too if only they use them). Delete `aspect_ratio` (use the style). For `tab_list`: move "overflow into a menu" and "scroll the strip" to `kivact::ui::tabs` (`.overflow(TabOverflow::Menu | Scroll)`), keep only soft's tab *looks* as recipes; delete `TabList`, `TabEntry`, `TabLayout`, `TabPattern` public types except the look enums. Same question for `top_nav` (938), `side_navigation` (1.5k), `app_shell` (682): they are real domain parts (app frame) and stay, but share a core `sidebar` with basic (KIT-10).
- breaking: yes   kpi: K6   size: L
- confidence: medium for tab_list (read 400 of 966 lines), high for the zero-use list (grepped examples, tests, dev, docs)

### KIT-10 Two sidebars, two chart stacks, two chat recipe sets
- kind: DUP
- where: basic sidebar.rs + sidebar/*.rs (1180 lines) vs soft side_navigation.rs + side_navigation/*.rs (1511 lines); creative dock/panel_group; basic `chart/` (1.5k, bar/line/area/pie/radar) vs basic `plot/` (2.9k, XY/heatmap; own scale.rs, ticks.rs; shares only `nice.rs round_step`); basic chat.rs/chat_colors.rs vs pop recipes_chat.rs (242) vs soft chat_appearance.rs/soft_chat_tools.rs.
- evidence: sidebars implement open/collapsed, mobile drawer, groups, menu buttons, resize each in a kit-private way; `chart/scale.rs::text_width` estimates glyph widths with a table ("a generous estimate") because hit-testing cannot measure text, and `plot` solved the same layout again with `plot_area`.
- fix: (1) sidebar behaviour (collapse state, rail/flyout, mobile drawer = Machine) goes to `kivact::ui::sidebar` with `SidebarRecipes`, the same shape as `tabs`; basic and soft keep only recipes + their own extra parts (basic SidebarMenuButton sizes, soft SideNavHeading). Size: moves ~1.5k lines to core, deletes ~1k. (2) `chart` and `plot` merge into one `plot` model with `PlotKind::{Bar,Line,Area,Pie,Radar,Scatter,Heatmap}`? No: bar/pie/radar categorical charts and XY plots have different models; keep both, but both read the one `ticks.rs`/`scale.rs` and chart's `text_width` is replaced by a measured width the plot code already uses. (3) chat recipes stay per kit (they are looks).
- breaking: yes   kpi: K6   size: L
- confidence: medium (sidebar internals not compared line by line)

### KIT-11 Game kpi: Machine + effect for a latch, per-instance subscriptions
- kind: STATE, INEFF
- where: crates/kivact-ui-game/src/kpi.rs:152-183; surface.rs:17-44 (`HaloMotionPhase` Machine fed a bool)
- evidence: `reveal_once` creates a `machine(Reveal::Hidden, ..)`, a `memo`, reads `kivact::platform::window_size()` and runs an `effect` that re-runs on every scroll-rect change and window resize for every KPI tile, forever, although the latch fires once. `Animated` + `memo(format(value.get()))` per tile. The halo has a 2-state Machine whose `transition` is `if running {Running} else {Stopped}` with an Effect equal to the state (a Machine that is only a signal).
- fix: core provides `NodeRef::once_visible(fraction) -> Value<bool>` (latch implemented once in core with one visibility observation that disposes itself after firing); kpi uses it, deleting the Machine, the effect and the window_size dependency. Halo: `memo(|| running.get())` driving the rotation `Animated`; delete `HaloMotionInput`.
- breaking: no   kpi: K3, K2   size: S
- confidence: high

### KIT-12 Catalogs: five shells, five registries, two scroll-spies
- kind: DUP, STRUCT, CONTRA
- where: examples/kivact-ui-basic/src/{model.rs,docs.rs,docs/*.rs,shell.rs,page.rs}; examples/kivact-ui-soft/src/{pages.rs (582 lines of 113 `PageInfo{slug,name,category}` literals), docs/*.rs (slug string-match chain across 9 modules)}; examples/kivact-ui-pop/src/pop/shell.rs:321 and examples/kivact-ui-game/src/game/shell.rs:318 (`install_scroll_spy`, two different algorithms); examples/kivact-ui-creative/src/creative/editors.rs:256 (`fn mix` = `Color::mix`); examples/kivact-ui-{pop,game}/src/messages.rs (hand-kept Japanese table "for every key of the kit")
- evidence: basic has the good design (`Page{slug,name,description,usage,source:include_str!,examples}` + `// example: id` markers so shown code is the running code). Soft separates "the list of pages" from "the code for page X", linked by a string `match slug` in 9 files; a slug typo yields an empty page (`.unwrap_or_else(node)`, a silent default) and only a test notices. Pop/game are showcase scrolls (hero, sections) with bespoke shells. Basic descriptions are Japanese, soft's English, pop/game Japanese; catalogs duplicate `fonts.rs`, `main.rs` with different setup (`.locale/.messages` vs a `for font` loop). `dark`/`selected`/`query` signals each carry `xtask-allow(state-fields)` explaining why they are independent.
- fix: new dev-only crate `examples/catalog-shell` (publish=false): `Page`, `Example`, `Page::snippet`, `catalog(pages: Vec<Page>, theme: fn(ThemeMode)->Theme, fonts, locale) -> impl IntoView` (sidebar list, search, dark toggle, page area, toast region, command palette). Page order and category come from one `Vec<Page>` built in the kit catalog (no separate registry, no string match). One scroll-spy in the shell for showcase-type pages (pop/game become pages with sections). Each of the five catalogs becomes `pages.rs` (+ `docs/*.rs`) + `main.rs` of 10 lines. Delete soft `pages.rs`, `docs/examples.rs`, `specimen.rs`; delete `mix` in creative. Decision on language: catalog UI text English, example descriptions English (docs jp stay in docs/). Message tables: `messages()` generated by `Messages::from_keys_with(kit prefix)`: out of scope, drop the hand table and show the English defaults, keeping one Japanese demo in the shell only.
- breaking: no (dev crates)   kpi: -   size: L
- confidence: high

### KIT-13 Game: kit options missing, docs contradict the API
- kind: API, CONTRA
- where: crates/kivact-ui-game/src/theme.rs:37-40 (`ThemeOptions{density}`), tokens.rs:24-25 (`display_family`, `mono_family`), docs/for-kivact-ui-user/en/game/how-to-guide/theme-and-density.md:99-116, getting-started/02-build-the-panel.md:35; README line 35 / docs 01-set-up:83
- evidence: fonts can only be changed by mutating `Tokens` after building (`tokens.display_family = "Barlow Condensed".into()` in the doc), but `Tokens::palette()` is computed from the same tokens inside `theme_with`, so the doc path rebuilds half a theme by hand; the game catalog bypasses it and writes `.font_family("Barlow Condensed")` literally in data.rs:55,450,460 while `display_family` stays "sans-serif". Every game recipe sets `font_family(display_family)` explicitly, so an app that sets `Theme::with_text(family)` is ignored by buttons/avatars/badges. Docs write `ThemeOptions { density: Density::Comfortable, ..Default::default() }` for a one-field struct (clippy `needless_update`). `Tokens` doc says "Prototype color tables" (process wording). Pop has the same hard-coded family problem: recipes do not read the theme text family.
- fix: `ThemeOptions { density: Density, display_font: Option<String>, mono_font: Option<String> }` in game (and the same `display_font` idea nowhere else: other kits leave fonts to `Typography.family`). Recipes read `theme.text().family` unless the token is set. Fix the three docs and the catalog to use the options. Reword "Prototype" doc comments (also game/icons.rs:1, recipes_indicators.rs:1, pop/icons.rs:1 "from the reference prototype").
- breaking: yes   kpi: -   size: S
- confidence: high

### KIT-14 `Tokens::fallback` allocates on every `ext` call when the kit theme is not installed; recipes rebuild full Styles
- kind: INEFF
- where: crates/kivact-style/src/theme/value.rs:231 (`ext`: `unwrap_or_else(|| Arc::new(T::fallback(self)))`), kits' `fallback` = `Self::new(theme.mode())` (pop tokens.rs:144, soft tokens.rs:53, game tokens.rs:84); 559 `ext::<Tokens>()`/`use_tokens` call sites in kits (soft 382, basic 162, pop 103, game 88, creative 55); pop button.rs:32-110 builds gradients, shadows and filter styles on every call
- evidence: a subtree that is not under the kit's theme (a dialog from a plain theme, devtools showing a kit component) pays a full `Tokens::new` plus an Arc per read, and every style evaluation of a button (hover, press) re-creates `Paint::from(linear_gradient(..))`, `Shadow`, `Filter` values. A page with 500 buttons re-evaluates the recipe 500x on a theme change (necessary) but also on each state change per instance.
- fix: (core) memoize `ext::<T>()` for the fallback in the `Theme` (a `OnceCell` slot per extension type, so at most one fallback per theme); (kits) recipes read one `let p = t.ext::<Tokens>()` per recipe fn (already mostly true) and precompute gradient paints into `Tokens` (`tokens.button_primary: Paint`) at `Tokens::new`, so a style evaluation clones an Arc instead of building. Ask the core auditor whether recipe results are memoized per `(recipe, Look)`; if not, that is the larger K3 win (core, not kit).
- breaking: no   kpi: K3, K5   size: M
- confidence: medium (I did not run a profile)

### KIT-15 Basic `plot` is not a shadcn component and should be its own crate
- kind: STRUCT
- where: crates/kivact-ui-basic/src/plot/* (2,900 lines incl. model, input 460, paint 527, view 341), `kivact-svg` export; plot depends on basic only through `Tokens` chart colors and a colormap (plot/paint.rs:11,86,128,140)
- evidence: inputs are `PlotModel`/`PlotData`/`PlotSeries` (few types), output is a `Plot` view and `PlotView` readouts; reason to change differs from the Lyra look (analysis tooling vs kit style); a downstream analyzer app needs it with whatever kit it uses. Basic's README/doc place it in "Lyra".
- fix: new crate `kivact-plot` (depends on `kivact` only): `plot(model)` reads its colors from a `PlotTokens` theme extension with fallback derived from `ColorToken::{Accent,..}`; every kit may install its own `PlotTokens` (basic installs the chart-1..5 colors). Keep `chart/` (shadcn parity) in basic.
- breaking: yes (path of `basic::plot`)   kpi: K6 (basic users who do not plot save ~2.9k lines)   size: M
- confidence: medium (decision depends on whether the owner wants kit-neutral plotting)

### KIT-16 Process residue in READMEs, docs, comments
- kind: PROCESS, DEAD
- where: crates/kivact-ui-*/README.md ("Migrating from the previous release": basic README:170, soft README:283, creative README:309, game and pop tables of old->new names; 76 + 156 + 13 + 28 + 27 table rows), "Filled recipes" lists (`<!-- kivact-recipes -->`) copying `recipes.rs`; `xtask-allow(state-fields)`: soft 96, creative 33, basic 18; docs/for-kivact-ui-user en+jp 110 files
- evidence: kits are not published ("not on crates.io yet", docs README), so there is no previous release; the tables describe API of a repo state nobody depends on, and `no compatibility layers` is the owner's rule. The "Filled recipes" list is a third copy of `build()` (code, README, xtask check). Soft's 96 comments are all of the form `// xtask-allow(state-fields): option: caller content from X::header, mounted once; absent means no sidebar header.` on `Option<..>` builder fields: the rule flags a builder holding "absent or present" and then asks the author to explain why it is not a state machine. `pop/README` says "prefixed with `pop.`" for keys that are mostly unprefixed (`calendar.next`, `palette.hint`). Docs en/jp are 55+55 files; every API change needs 2 edits per file and docs compile-check has no jp/en link check.
- fix: delete all five "Migrating" sections and the "Filled recipes" lists (and the xtask that checks them); replace with a single "Components" table per README generated by `cargo doc` links, or nothing. Remove the state-fields rule for plain builder `Option` fields that are set once at construction (rule scope: fields mutated after construction) and delete the 147 kit comments in the same change. Keep jp docs only for tutorial (`getting-started`, 4 files per kit); how-to guides in English only (halves 55 files), or generate jp from en once at release.
- breaking: no   kpi: -   size: M
- confidence: high

### KIT-17 Color and clock helpers duplicated across crates
- kind: DUP
- where: soft theme_scope.rs:107-120 (`luminance`, `contrast`), kivact-core/src/devtools/a11y.rs:273-292, arvane-render/src/text.rs:99-105, kivact-types/src/pixel.rs; kits' own timers (soft timer.rs, timestamp.rs `SystemTime::now()` per instance)
- fix: `Color::relative_luminance()` and `Color::contrast_ratio(other)` in kivact-style (single sRGB formula); soft `media_theme` uses them; a core `use_clock(resolution) -> Value<Instant>` (one shared timer per resolution) replaces per-instance tick machines in soft timestamp/timer.
- breaking: no   kpi: K2, K3   size: S
- confidence: high

### KIT-18 `kivact-devtools` links the whole basic kit
- kind: STRUCT
- where: crates/kivact-devtools/Cargo.toml:16, src/app.rs:35, src/panels/performance.rs:7
- evidence: devtools only needs a theme and a few components (`muted`, buttons, tabs); depending on `kivact-ui-basic` adds chart, plot, sidebar, questionnaire to every build that enables devtools (K6 in debug-tooling builds, compile time).
- fix: devtools uses `kivact::ui` components plus a 100-line private theme (it already uses a kit theme for look); delete the dependency. If the owner wants the Lyra look, use `kivact-ui-basic` with `default-features = false` and put chart/plot behind features `chart`, `plot`. Decision: private theme.
- breaking: no   kpi: K6   size: S
- confidence: medium

### KIT-19 Soft theme/tokens are an unbounded surface
- kind: STRUCT, API
- where: crates/kivact-ui-soft/src/tokens.rs (512), color.rs (660), color_name.rs (183), theme.rs (372, 20 `kivact::interaction::*` policy imports)
- evidence: `Tokens` exposes `spacing.s1..s6`, `radius.{inner,element,container}`, `sizes.*`, `motion.*`, `typography.<role>.{size_px,line_height_px}`, `colors.{text_primary, bg_surface, overlay_hover, ...}` as a public nested struct used by 382 call sites; the theme also sets interaction policy (menu, slider, toast, switch) which the other kits leave at core defaults. Other kits show what a kit needs: a `Palette` + ~20 tokens.
- fix: keep nested tokens (they mirror the Astryx spec) but make the sub-structs private modules and public only `Tokens` fields that recipes outside the crate legitimately read (`colors`, `spacing`, `radius`); move the interaction-policy block into one `fn policy() -> InteractionPolicy` in `theme_policy.rs` so `theme.rs` is the same 40-line shape as the other kits. Delete `MotionTokens/TypeTokens/RadiusTokens/SpaceTokens` re-exports unless a catalog uses them (4 appear in the unreferenced list).
- breaking: yes   kpi: K1 (theme build cost)   size: M
- confidence: medium

### KIT-20 Catalog and kit unit tests mirror each other's coverage
- kind: DUP
- where: crates/kivact-ui-*/tests/consumer/* (2.7k+0.9k+0.9k+4.3k+5.0k lines), examples/kivact-ui-*/tests/catalog.rs (292/367/266/322/195+236 lines); pop and game consumer tests have the same file names (controls, forms, interactions, overlays)
- evidence: each catalog test walks every page and asserts no errors; kit tests render the same components. Not harmful, but pop/game test files are same-name twins like their recipes.
- fix: no change before the shell crate exists; after KIT-12, a single `catalog-shell` test iterating `Vec<Page>` replaces the five catalog walkers (about 600 lines).
- breaking: no   kpi: -   size: S
- confidence: medium

## 3. Target structure

Crates (unchanged count, boundaries justified above):
- `kivact-ui-basic`, `kivact-ui-pop`, `kivact-ui-game`, `kivact-ui-creative`, `kivact-ui-soft`: each depends on `kivact` only. One reason to change per crate: the kit's look and domain parts.
- new `kivact-plot` (KIT-15): inputs `PlotModel`, output `Plot`; reason to change: plotting features.
- new dev-only `examples/catalog-shell` (KIT-12): inputs `Vec<Page>` + theme fn, output the app view.
- core additions required (generic, small): `UiRecipes::shared`, `IconSet::shared`, `icons!` macro, `theme_colors!` macro, `.variant(impl IntoValue<V>)`, `kivact::ui::{Slant, Striped}`, `NodeRef::once_visible`, `Color::{relative_luminance, contrast_ratio}`, `use_clock`, `kivact::ui::sidebar`, `kivact::ui::SyntaxKind`, tabs overflow (KIT-09), i18n date formatting from unix instants.

One uniform kit skeleton (apply to every kit; a missing piece is created empty-free, not added as a stub):

```
crates/kivact-ui-<kit>/
  Cargo.toml            depends on kivact only
  README.md             what it is + the 6-line "use it" snippet + link to docs; no migration table, no recipe list
  assets/icons/*.svg    only if the kit has its own icons
  src/lib.rs            module doc: 1 paragraph + the example; then `mod ...;` grouped as below, `pub use` grouped with a blank line between groups
  src/theme.rs          pub fn theme(mode) -> Theme; [pub struct ThemeOptions + pub fn theme_with(mode, ThemeOptions) -> Theme only if the kit has real options]; body = Palette + Spacing + Radii + Typography + UiTokens + .with(Tokens) + .with(recipes::recipes()) + .with_icons(icons::set())
  src/tokens.rs         pub struct Tokens (flat, Copy-able fields; `fn new(mode[, options])`; impl ThemeExtension with fallback); no `mode` field
  src/color.rs          theme_colors! { Tokens { .. } } -> pub enum ColorName
  src/icons.rs          icons!(..) list + pub(crate) fn set() -> IconSet via IconSet::shared
  src/recipes.rs        pub(crate) fn recipes() -> UiRecipes via UiRecipes::shared(build); build() is a flat chain in UiRecipes entry order
  src/recipes_<group>.rs  one file per group of `UiRecipes` entries (controls, overlays, collections, navigation, feedback, metrics, chat, input, calendar, files); every `fn(&Theme,&Look<S>)->Style` lives here; max ~250 lines
  src/variants.rs       pub enum <Component>Variant (+ marker structs) the kit adds to standard components; `impl Variant`; nothing else
  src/<domain>/         domain parts only (builders over kivact::ui, no recipes); one module per component; name = component name, no kit prefix
  tests/consumer/*.rs   one file per recipe group + one per domain module
examples/kivact-ui-<kit>/
  src/main.rs (10 lines), src/lib.rs (pub fn pages() -> Vec<Page>), src/docs/<component>.rs (Page + `// example:` markers), assets/ only for kit-specific fonts
```

Naming rules for the skeleton: appearance enums are `<Component>Variant`; kit-specific color keys are `<Name>Hue`-free (use `Hue`/`Tone` of core); no `Game*`/`Pop*`/`Soft*` prefixes; domain components are `snake_case` fn + `PascalCase` builder.

Deletes: `Rc<RefCell<TimestampSettings>>`, soft `MobileMachine`, creative 4 `observe_inputs`, pop 4 `XHue` + `WeekendHue` type, pop/game `Slant/Striped`, game `ButtonTone`, all `OnceLock` blocks, the 5 README migration sections + recipe lists, 147 kit `xtask-allow` comments, soft dead fns of KIT-09, basic `CodeTone/CodeColors`, creative `mix` in catalog, soft `pages.rs` registry.

## 4. Public API (before / after)

Variant and hue:
```rust
// before (pop)
button("Go").variant(ButtonTone::Pink).variant(Slant)
badge("New").variant(BadgeHue::new(Hue::Mint))
stat(..).variant(StatHue::new(hue_signal))
progress(..).variant(ProgressHue(hue_signal.into_value()))
// after
button("Go").variant(Hue::Pink).variant(kivact::ui::Slant)
badge("New").variant(Hue::Mint)
stat(..).variant(hue_signal)
progress(..).variant(hue_signal)
```
Theme options (uniform spelling):
```rust
// before: game docs and README
game::theme_with(mode, ThemeOptions { density: Density::Comfortable, ..Default::default() })
t.ext::<game::Tokens>().display_family  // and mutate Tokens by hand to set fonts
// after
game::theme_with(mode, game::ThemeOptions { density: Density::Compact, display_font: Some("Barlow Condensed".into()), ..Default::default() })
```
Timestamp:
```rust
// before
timestamp(TimestampValue::parse_iso("2026-11-01T09:00Z").unwrap()).format(TimestampFormat::DateLong).time_zone(TimeZone{offset_seconds: 32400, label: Some("JST".into())})
// after (i18n-formatted, locale from App::locale)
timestamp(instant).format(TimestampFormat::DateLong)   // zone from the App/locale; ISO formats unchanged
```
Names to delete: `pop::{BadgeHue, ItemHue, StatHue, ProgressHue, WeekendHue, Slant, Striped}`, `game::{ButtonTone, Slant, Striped}`, `basic::{CodeTone, CodeColors}`, `soft::{TokenKind, CodeLanguage tokenizer duplicate, TimestampEntry::copyable, reveal_*, viewport_*, container_below, is_compact, streaming_text, use_streaming_text, chat_token, chat_tokenized_text, rich_text_toolbar, rich_text_view, use_scroll_state, use_scroll_overflow, use_overflow, link_with, icon_style, aspect_ratio, TabList family}`.
Names to rename: `game::GameInputGroupVariant -> InputGroupVariant`, `game::GameCellVariant -> CellVariant`, `pop::PopBubble -> ChatBubbleVariant`, soft `*Appearance -> *Variant`, basic `CardLook/CodeBlockLook -> *Variant`.

## 5. Open questions for the owner

1. Soft's theme-independent big parts (markdown 1.3k lines, rich_blocks/rich_text 1.4k, power_search, lightbox, thumbnail, stepper): keep inside `kivact-ui-soft`, or extract `kivact-markdown` / `kivact-rich-text` as kit-neutral crates? Default decision in this report: keep in soft, delete only the no-user fns.
2. Does any of the four downstream apps use a name on the KIT-09 delete list (reveal_*, viewport_*, streaming_text, chat_token*, rich_text_*, use_scroll_*, tab_list, aspect_ratio)? I could not check.
3. Is `plot` meant to stay inside basic (KIT-15) or become `kivact-plot`?
4. Catalog language: English everywhere (recommended) or keep Japanese descriptions in basic/pop/game?
5. jp docs: keep both languages for all 55 files, or jp for tutorials only (KIT-16)?
6. Sidebar into core (KIT-10) is the biggest move; confirm before scheduling, since it touches basic and soft public API at once.
