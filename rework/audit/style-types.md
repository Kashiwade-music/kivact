# Style / types / shader / svg audit (prefix ST)

Scope: `crates/kivact-types` (12.6k lines), `crates/kivact-style` (12.3k), `crates/kivact-shader` (0.7k), `crates/kivact-svg` (1.7k).
Read-only audit of `refactor/kashiwade/fundamental-rework` = 2b8df5ac. No build was run.

## 0. Summary

1. **There is no single styling model.** A color can be named six ways (`Color`, `ColorToken`, `Color::token`, `Color::of`, `theme.colors().x`, kit `ColorName` through `ThemeColor` with a process-global interning `Mutex`). Theme-dependent sizes need a `.style(move || use_theme()...)` closure (268 such closures). Interaction looks use two mechanisms (state styles and recipes that read `PartState`) because state styles change paint only (ST-01, ST-02, ST-05).
2. **Every value exists twice or three times.** 16 public type names exist in both `kivact-types` and `kivact-style`. `Length` and `Dimension` are identical, and so are `TextLength`/`TextLen`, `GridAutoFlow`/`GridFlow`, `Insets`/`Edges<f64>` and `ThemeMode`/`ThemeHint`. Rich text has three span types, two named `TextSpan` and two named `SpanStyle`. Same name, different meaning: `Visibility`, `Transform`, `Shape`, `Gradient`, `Paint`, `FilterGraph`, `Look` (ST-10, ST-11, ST-21).
3. **Colors resolve against the wrong theme**, in `Animatable for Color`, `Color::components` and `Color::mix` (they use the default light theme). A palette entry that is a token resolves to black. The SVG export draws something different from the window: gradient interpolation, paint areas, layer composite, and text fill/stroke (ST-03, ST-04, ST-13, ST-14, ST-15).
4. **Style compilation is heavier than needed:** whole-style clones, a full `paint()` per state, defaults rebuilt on every run, `CompiledStyle` cloned into the engine, a theme-extension fallback allocated on every read, and a deep compare of the fat `Theme` (ST-06, ST-07, ST-26).
5. **`kivact-types` is a dumping ground, not a vocabulary crate.** It holds the Arvane render protocol (1.3k lines), engine-only interpolation (`prop_interp`, 507 lines), interior-mutable helpers and process-global atomics (`cell`, `process`), next to the pure values (ST-20). The Style property list is restated in about 8 places and the 10 state names in about 9 places (ST-08, ST-09). There are 174 `xtask-allow(state-fields)` markers in style alone (ST-28).

**Target shape.**
- *Style* is one CSS-like value: a list of declarations, generated from one property table. Every state accepts every property.
- *Theme* is a flat set of variables (CSS custom properties). Core and kit variables are typed constants, usable directly in any style value without closures.
- *Value types* exist once, in `kivact-types`, and are re-exported by the facade.
- `kivact-types` keeps only the vocabulary. The Arvane protocol, engine-only code and sync helpers move out.
- `kivact-svg` stays a leaf crate but reuses one shared, pure interpretation of `DrawCmd` geometry and gradients (in types) with the engine.

## 1. Current structure map

| Crate / module | Responsibility | LOC | Used by | Boundary verdict |
|---|---|---|---|---|
| kivact-types | everything shared | 12.6k | 19 crates | **dumping ground**: 5 unrelated groups below |
| ├ geometry, geometry3d, pixel, color, color_space, ids, asset | pure values | ~3.0k | all | good vocabulary |
| ├ input, env, platform, window, embed, gpu, semantics*, image_status, presence, scroll, view_transform, drag_velocity | interaction / platform vocabulary | ~2.9k | core, engine, arvane-*, ui | good. `drag_velocity` is a stateful estimator (fine), and `embed::Visibility` collides with `layout::Visibility` |
| ├ layout, paint, text/*, motion, animatable | node-props contract style→engine (LayoutProps, PaintProps, StateStyles, TextStyle) | ~3.3k | style (producer), engine, layout | real boundary, but duplicated by style's authored types (ST-10) |
| ├ scene/*, effect/* | DrawList recording (Draw→engine/svg) and filter/shader wire | ~2.0k | style, engine, svg, arvane-compose | real boundary. Lacks the shared interpretation helpers that engine and svg both re-implement (ST-16) |
| ├ render/* | host↔Arvane frame protocol (RenderScene, Composition, ShapedText, SemanticsTree, CpuFrame) | 1.3k | engine + arvane-* only; style/core use 4 names | **missing boundary**: belongs in its own crate |
| ├ prop_interp | paint-transition interpolation | 507 | kivact-engine only | **wrong crate** |
| ├ cell, process | ReportOnce, Mailbox, Registry, Recent (Mutex/RefCell), process-global DetachedThreads atomic | 346 | all | **not values**: interior mutability and global state in a "pure values" crate |
| └ check, defaults, text_shaping, text_utf, semantics_text | layout-check findings, constants, TextShaper trait, UTF helpers | ~0.4k | engine, core, test, editing | acceptable |
| kivact-style | authored Style, Theme, Draw, Path, resolve | 12.3k | kivact-core, kivact-test | boundary core↔style is real (input Style+Theme+Interactive → CompiledStyle; Draw → DrawList). Internally the property list is restated 8× (ST-08) |
| ├ sheet/* | Style groups, setters, merge | 1.97k | | |
| ├ resolve/* | compile_style, paint deltas, describe | 1.3k | | heavy per run (ST-06) |
| ├ theme/*, color/* | Theme, ThemeExtension, ThemeColor, Palette, Color | 1.5k | | 6 ways to name a color (ST-01) |
| ├ draw/*, path/*, paint/*, effect/*, transform, text, grid, length, icon, shader/* | painter and CSS value types | ~7.5k | | 16 names duplicate types (ST-10) |
| kivact-shader | WGSL validation + assembly (naga) | 734 | kivact-style | **clean boundary** (source+kind → ShaderProgram\|ShaderError). Linked even without `gpu` (ST-25) |
| kivact-svg | DrawList → SVG 1.1 text | 1.69k | kivact-core (Offscreen::render_svg) | **clean dependency boundary** (types + png, `Resources` trait). **Duplicated interpretation**: shape outlines, paint area, gradient stops re-implemented from engine::lower, with divergences (ST-13, ST-14, ST-16) |

## 2. Findings

### ST-01 Theme-dependent values: six ways to name a color, closures for sizes, three definitions per kit color
- kind: API / DUP
- where: crates/kivact-style/src/theme/value.rs:537 (`ThemeColor`), :573 (`THEMED`), :659 (`Color::of`); crates/kivact-style/src/color/value.rs:6 (`ColorToken`); crates/kivact-style/src/theme/tokens.rs:20 (`Palette`); crates/kivact-ui-basic/src/color.rs:14,97 (`ColorName` + match); crates/kivact-ui-basic/src/theme.rs:18 (Palette re-mapped from kit Tokens)
- evidence:
  - Ways to name one theme color: `ColorToken::Accent`, `Color::token(..)`, `Color::from(..)`, `Color::of(..)`, `theme.colors().accent`, `theme.colors().get(..)`.
  - Every kit color is defined three times: a `Tokens` field, a `ColorName` variant, and a `ThemeColor` match arm. This is repeated in all 5 kits (325 `ColorName::` uses).
  - Each kit theme then copies its tokens into the core `Palette`: the same fact stored twice (S-4).
  - Theme-dependent sizes (`theme.space().md`, `UiTokens` fields) are only reachable inside a reactive closure: 126 `use_theme()` calls and 268 `.style(move ||` closures.
- fix: replace the palette, the kit tokens' color/size fields, `ThemeColor` and `ColorName` with **theme variables**:
  - In `kivact-types::theme` (pure), define `pub struct Var<T: 'static> { name: &'static str, fallback: T }`.
  - Add `pub const fn Color::var(v: &'static Var<Color>) -> Color` (`Repr::Var(&'static Var<Color>, f32 alpha)` keeps `Color: Copy`) and `pub const fn Length::var(v: &'static Var<f64>) -> Length` (`Length::Var(&'static Var<f64>)`).
  - The core ships `kivact::style::color::{SURFACE, TEXT, ACCENT, ..}` and `kivact::style::space::{XS..XL}`, `radius::{SM,MD,LG}`, `font::{SIZE, SIZE_SMALL, ..}`. Each is a `pub const X: Color = Color::var(&X_VAR)`, with `static X_VAR: Var<Color> = Var::new("accent", rgb(0x2563eb))`; `rgb` becomes a `const fn`.
  - `Theme` stores `vars: Arc<VarTable>`, a sorted `Vec<(ptr, Value)>` keyed by the `&'static Var` address.
  - API: `Theme::set(&'static Var<T>, impl Into<T>)` and `Theme::get(&'static Var<T>) -> T`. A value may itself be a var (alias, CSS `--accent: var(--primary)`), resolved with a depth limit of 8, plus a diagnostic.
  - Kits declare `pub const PRIMARY: Color = Color::var(&PRIMARY_VAR)` once and set the values in `theme(mode)`.
  - Delete: `ThemeColor`, `Color::of`, the `THEMED` table, `Repr::Themed`, `ColorToken`, `Palette`/`Spacing`/`Radii` (and the `Typography` sizes), `Theme::with_colors/with_space/with_radius/with_text`, and every kit `ColorName` with its color fields in kit `Tokens`.
  - `ThemeExtension` remains only for non-value tables (recipe tables, `InteractionPolicy`).
- breaking: yes   kpi: K3 (no closure re-runs, no mutex), K1   size: L
- confidence: high

### ST-02 Global `THEMED` interning table: mutex, leak, linear downcast scan on every set and every resolve
- kind: INEFF / STATE
- where: crates/kivact-style/src/theme/value.rs:573-592, :603-608
- evidence: `Color::of` → `themed_index` locks a process-global `Mutex<Vec<&'static dyn Themed>>` and scans all entries with `downcast_ref`, then `Box::leak`s new values. Every `resolve` of such a color locks the mutex again (`themed_value`) and calls `theme.ext::<Tokens>()` (a linear TypeId search, or a fallback allocation, see ST-07).
- fix: deleted by ST-01. If ST-01 is not done, store `&'static dyn Themed` directly in `Repr` (still `Copy`) and drop the table.
- breaking: no   kpi: K3   size: S
- confidence: high

### ST-03 Tokens resolve against the default light theme in animation, `components()` and `mix()`
- kind: BUG / CONTRA / INEFF
- where: crates/kivact-style/src/theme/value.rs:671 (`components`), :684 (`mix`), :713 (`Animatable::to_components`); used e.g. crates/kivact-ui-basic/src/button.rs (`p.secondary.mix(p.foreground, 0.05)`)
- evidence:
  - `to_components` calls `self.resolve_space(&Theme::default())`. An `animated(..)` color holding a token interpolates the **light** theme's value under a dark or kit theme, and the result is a concrete `Repr::Space`, so the theme link is lost.
  - `Theme::default()` → `Theme::light()` builds `IconSet::standard()` (a 19-entry map allocation) on every call.
  - This contradicts the module invariant "colors resolve in the theme they are painted in".
- fix:
  - Delete `Color::components()` and `Color::mix()`. Rename `components_in` → `components(&Theme)` and `mix_in` → `mix(other, t, &Theme)`.
  - `impl Animatable for Color` resolves `Var`/token colors with the ambient theme: `kivact_style::try_use_theme()` (new, non-panicking; untracked read), falling back to a `static DEFAULT_THEME: LazyLock<Theme>`.
  - `Theme::light()`/`dark()` return clones of `LazyLock` statics (cheap after ST-26).
- breaking: yes (2 fns removed)   kpi: K3   size: S
- confidence: high

### ST-04 A palette entry that is itself a token resolves to opaque black, silently
- kind: BUG (latent) / CONTRA
- where: crates/kivact-style/src/theme/value.rs:603 (`theme.colors.get(t).resolve_concrete()`), :630-642
- evidence: `resolve_concrete` maps `Repr::Token`/`Repr::Themed` to `Rgba{0,0,0,1}`. `Palette` fields are public `Color`s, so `Palette { hover: ColorToken::Text.into(), .. }` compiles and draws black, with no diagnostic (G-05). The kits avoid this today only by convention.
- fix: ST-01's alias resolution with a depth limit. A cycle or too deep a chain reports `ErrorKind::Unsupported` once per var and uses the var's fallback.
- breaking: no   kpi: -   size: S
- confidence: high

### ST-05 State styles have two semantics, so kits fall back to recipes that read `PartState`
- kind: API / CONTRA
- where: crates/kivact-style/src/sheet.rs:16-25 (doc), crates/kivact-style/src/sheet/setters.rs:659-747, crates/kivact-style/src/resolve/compile.rs:377-388, crates/kivact-style/src/resolve/describe.rs:10 (`DELTA`), crates/kivact-ui-basic/src/button.rs:127
- evidence:
  - `hover/pressed/focus/focus_visible/group_*/enter/exit` are host-evaluated paint deltas: a layout or text property other than `color` is reported `Unsupported` and ignored.
  - `disabled/selected` are merged at compile time and accept any property.
  - CSS developers expect `:hover { text-decoration: underline }` to work. The Basic kit's link button therefore reads `input.is_hovered()` in a recipe to add the underline: a second mechanism for the same thing.
- fix: one rule, "every state accepts every property".
  - `compile_style` splits each state into a paint delta (host-evaluated, unchanged fast path: `PaintDelta` gains the text paint props `text_decoration_lines/color/style`) and a *structural rest* (layout or text-metric props).
  - If any host state has a structural rest, `CompiledStyle` sets `needs: StateSet` (bit set of `StyleState`, ST-09). `bind_style` (crates/kivact-core/src/view/mount/style.rs:126) then reads the matching `NodeRef`-equivalent signals for those states only and merges the rest like `disabled/selected`.
  - Delete `unsupported_in_states`, `delta_misuse` and the `DELTA` list.
- breaking: no (only widens)   kpi: K5 (recompile only on nodes that opt in)   size: M
- confidence: high

### ST-06 `compile_style` and `bind_style` clone and recompute per run
- kind: INEFF
- where: crates/kivact-style/src/resolve/compile.rs:371 (`s.clone()`), :261 (default ring `Style` built per run), :268-275 (disabled default `Box<Style>` per run), :332 (focus_visible clones `s` again); crates/kivact-style/src/resolve/paint.rs:319-320 (`base.clone().merge(st)` then a full `paint()` per state); crates/kivact-core/src/view/mount/style.rs:20 (`merge_layers` folds from `Style::default()`), :150-157 (`resolved.paint.clone()`, `.text.clone()`, `.states.clone()`), :192/:212 (`Box::new(resolved.clone())`: the whole `CompiledStyle` incl. `Vec<UiError>` and `Vec<String>` is cloned to keep a paint callback)
- evidence: for a focusable button with hover and pressed, one style run does about 5 Style clones, 4 full `paint()` resolutions (each allocating `shadows` Vecs and filter/mask/effects) and 4 PropValue clones. Every theme switch or signal change reruns this for each node.
- fix:
  - `compile_style(s: Style /*owned*/, ..)` takes the merged style by value.
  - The theme defaults become two `static` `PaintDelta`-producing functions applied to the delta map, with no `Style` allocation.
  - `paint_delta(base_props: &PaintProps, st: &Style, theme)` resolves only the groups `st` sets and copies the rest from `base_props`, computed once.
  - Core destructures `CompiledStyle { layout, paint, text, states, retained_paint, .. }` and moves the parts into `tree.update`. `LocalPaintSource` stores `Option<LocalPaint>` (the `Rc` callback), not `CompiledStyle`.
  - `merge_layers` starts from the first layer instead of `Style::default()`.
- breaking: no   kpi: K3, K5   size: M
- confidence: high

### ST-07 `Theme::ext` allocates the fallback on every read when a theme lacks the extension
- kind: INEFF
- where: crates/kivact-style/src/theme/value.rs:231-237; readers: 73 `ext::<UiTokens>()`/`use_tokens::<UiTokens>()` sites in crates/kivact-ui/src, crates/kivact-ui/src/recipes.rs:744 (`UiRecipes::fallback` = clone of the whole static table)
- evidence: `.unwrap_or_else(|| Arc::new(T::fallback(self)))`. An app on `Theme::light()` (no kit) builds `UiTokens::default()` (about 30 nested token structs) plus an `Arc` on every component style run. The same happens for `UiRecipes`, `InteractionPolicy`, and every kit `ColorName` resolve under a foreign theme.
- fix: `Theme` (after ST-26: `Arc<ThemeData>`) gets `fallbacks: OnceMap<TypeId, Arc<dyn Any+Send+Sync>>` (a `Mutex<Vec<Entry>>`, excluded from `PartialEq`). `ext::<T>()` fills it once per theme value. After ST-01 the token structs mostly disappear anyway.
- breaking: no   kpi: K1, K3   size: S
- confidence: high

### ST-08 The Style property list is restated in about 8 places, so adding a property touches 14 files
- kind: STRUCT / DUP
- where: crates/kivact-style/src/sheet/fields.rs (8 group structs + 8 `merge` lists), sheet/setters.rs + text_setters.rs + grid_setters.rs (≈165 setters, `duplicates` logic copied per macro), resolve/describe.rs:10 (`DELTA`) + `classify_properties!` tables, sheet/setters.rs:896 (`has_text`), resolve/paint.rs:174 (`sets_transform`), :48 / :99 / :321-328 (the `set` checks of border/effects, written twice), resolve/compile.rs:307-315; architecture.md:449 ("3 crate / 14 file" for one text property)
- evidence: the field list of `PaintFields` appears in the struct, `merge`, `paint()`, `paint_delta()`, `effects()`, the `describe` tables and `DELTA`. Some setters skip the duplicate counting (`scroll_snap`, `scrollbar_*`, `visibility`, `display`, all state setters, `shadow`). The sides are encoded three ways (`Sides`, `[Option<f64>;4]` with comments "top, right, bottom, left", `[Option<CornerRadius>;4]`).
- fix: Style becomes a declaration list generated from **one** table in `kivact-style/src/sheet/props.rs`:
  ```rust
  style_props! {
      // name        value type               class    wire target (fn in resolve)
      width:         Length            => Layout,  layout.width;
      background:    Background        => Paint,   paint.background;
      text_decoration_line: TextDecorationLine => TextPaint, text.decoration_lines;
      ...
  }
  ```
  - The macro generates `enum Decl { Width(Length), .. , State(StyleState, Box<Style>) }`, `pub struct Style(SmallVec<[Decl; 4]>)` (last wins; `merge` = append), all setters, `Decl::class()` (used by ST-05), `Decl::name()` and `describe`.
  - The `apply(&mut LayoutProps/PaintProps/TextStyle, theme)` match is the only hand-written part.
  - `duplicates` is counted by one generic `push` (same discriminant already present).
  - Delete `fields.rs` groups, `group_or_default!`, `Sides`, `describe.rs` tables and `has_text`/`sets_transform`.
  - The size check moves to `size_of::<Decl>() <= 64`.
- breaking: no (setter names unchanged unless ST-22)   kpi: K2 (only set props stored), K3 (merge = Vec append)   size: L
- confidence: high

### ST-09 The ten state names are enumerated in about 9 places
- kind: DUP
- where: sheet/fields.rs:386-433 (`StateFields` + merge array), resolve/compile.rs:191-216 (`ForcedStates` bits), :280-302, :305-355, :421-443 (`strip_state_paint` list); resolve/describe.rs:342-367; crates/kivact-types/src/paint.rs:923-978 (`StateStyles` fields + `deltas()`)
- fix:
  - Define `#[derive(Copy, Eq, Hash)] pub enum StyleState { Hover, Pressed, Focus, FocusVisible, GroupHover, GroupFocus, Disabled, Selected, Enter, Exit }` once, in kivact-types, with `const ALL: [StyleState; 10]`.
  - `StateStyles { deltas: [Option<Box<PaintDelta>>; 8], group: bool }` is indexed by state, and `ForcedStates` becomes `StateSet(u16)` over the same enum.
  - The style side stores `Decl::State(StyleState, Box<Style>)`.
  - Every list above becomes a loop over `StyleState::ALL`.
- breaking: no (DevTools protocol names derive from `StyleState::name()`)   kpi: -   size: M
- confidence: high

### ST-10 Authored types and wire types are the same value twice (VOC-1 violated)
- kind: DUP
- where (style ↔ types):
  - `Length` (kivact-style/src/length.rs:26) ≡ `Dimension` (kivact-types/src/layout.rs:485)
  - `TextLength` (style/text.rs:120) ≡ `TextLen` (types/text/properties.rs:18)
  - `GridAutoFlow` (style/grid.rs:249) ≡ `GridFlow` (types/layout.rs:180)
  - `CornerRadius` (style/geometry.rs:13, `Length`) vs (types/paint.rs:382, `Len`)
  - `Shadow` (style/effect/shadow.rs:20) vs (types/paint.rs:801), differing only in `Color` vs `Rgba`
  - `FontFeature`, `FontVariation`, `TextDecoration`, `BorderImage`, `MaskLayer`, `Gradient`, `GradientKind`, `RadialExtent`: the same Color/Rgba split
  - Within types: `Insets` (geometry.rs:150) ≡ `Edges<f64>` (:163)
  - `ThemeMode` (style) ≡ `ThemeHint` (types/env.rs:16)
  - `AvailableSpace` (kivact-core/src/custom/measurement.rs:8) ≡ (types/layout.rs:211)
- evidence: 16 identical public type names exist in both crates (script: `comm` of `pub struct|enum` names), each with a hand-written `to_wire()`/`wire()` conversion.
- fix:
  - **Identical pairs:** keep one definition in kivact-types and re-export it from style. Delete `Dimension` (`Length` moves to types), `TextLen` (`TextLength` moves to types), `GridFlow` (keep the name `GridAutoFlow`), `Insets` (use `Edges<f64>`), `ThemeHint` (keep `ColorScheme`, matching CSS `prefers-color-scheme`, in types; `ThemeMode` becomes an alias-free rename), and core's `AvailableSpace` (re-export types').
  - **Color/Rgba pairs:** after ST-01, `Color` (with `Var`) is a pure value and moves to `kivact-types::color`. The wire structs (`Shadow`, `Gradient`, `MaskLayer`, `BorderImage`, `TextDecoration`, `TextStroke`, `TextShadow`) then hold `Color`, and the engine's lowering resolves them with the node's theme (O-1). If O-1 is declined, keep the pairs but generate the wire twin by macro from one declaration.
  - `CornerRadius` keeps one type with `Length` x/y. A non-px/% value reports Unsupported (see ST-31).
- breaking: yes (names)   kpi: K6 (less monomorphized conversion code)   size: L
- confidence: high

### ST-11 Rich text and text-style types: about 10 types, two `TextSpan`, two `SpanStyle`; spans cannot set a decoration style
- kind: DUP / BUG
- where:
  - style/text.rs:470 `SpanStyle` (authored) → types/text/properties.rs:151 `SpanProps` + :174 `TextSpan` → types/render/text.rs `SpanStyle` + `TextSpan` (render)
  - Text style types: `TextExtras` (style/text.rs:261), types `TextStyle` (:107), `ComputedTextStyle`, `TextFormat` (style/draw/text_run.rs:35), `TextRun` (types/scene/recording.rs:77), `StyledRange` + `TextDecoration` (style/text.rs:573,585)
  - style/internal.rs:31-37: `decoration(..)` → style/text.rs:376 hard-codes `style: TextDecorationStyle::default()`
- evidence: a span's decoration is always solid, while `Style::text_decoration_style` works on paragraphs (BUG, high confidence). `StyledRange.background` is `Option<Color>` while `SpanStyle.background` is `Option<Paint>`. style/text.rs:59 documents "`Draw`'s `TextStyle`", but the type is `TextFormat` (CONTRA).
- fix: one text-property set.
  - (a) kivact-types `TextStyle` (declared, `Option` fields) is the single declared form. Paragraph, span (`TextSpan { range, style: TextStyle }`), editing range (`StyledRange { range, style: TextStyle }`) and `TextFormat` all use it. A span-illegal property (wrap, align, clamp, …) reports Unsupported.
  - (b) The render-protocol span becomes `arvane_protocol::ResolvedSpan` (px, Rgba), the only second form.
  - (c) Delete style `SpanStyle`, `CompiledSpanStyle`, `compile_span_style`, `compile_text_decoration`, `TextExtras` (absorbed by ST-08) and types `SpanProps`. `TextFormat` keeps only its painter-only fields (`align`, `baseline`, `fill`, `stroke`) plus `style: TextStyle`.
- breaking: yes   kpi: K6   size: M
- confidence: high

### ST-12 `text_decoration_style` is the only reactive (`IntoValue`) property inside `Style`
- kind: API / STATE
- where: crates/kivact-style/src/sheet/setters.rs:826-843, crates/kivact-style/src/text.rs:533-571 (`DecorationStyleValue` with `Rc<Prop>`, identity equality)
- evidence: every other setter takes a plain value. This one reads a `Prop` inside the style effect (`DecorationStyleValue::get`), making `Style` non-pure and giving two ways to make a style reactive.
- fix: `pub fn text_decoration_style(self, v: TextDecorationStyle)`. Delete `DecorationStyleValue`. Reactivity stays `.style(move || ..)`.
- breaking: yes (signature)   kpi: -   size: S
- confidence: high

### ST-13 SVG export ignores gradient interpolation space and color hints
- kind: BUG / DUP
- where: crates/kivact-svg/src/fill.rs:177-215 (`css_stops`), versus crates/kivact-engine/src/lower/fill.rs:523 (`stops`) + :581 (`expand_gradient`)
- evidence: `Gradient` has `hints` and `interpolation` (CSS default Oklab; types/paint.rs:229-241). The engine expands each interval in that space and adds hint stops. `css_stops` uses `stop.color.to_srgb()` and never reads `hints` or `interpolation`, so the SVG gradient interpolates in sRGB and drops hints. `render_svg` promises "the same drawing".
- fix:
  - Move `stops` + `expand_gradient` + `fraction` from engine/lower/fill.rs into `kivact-types/src/paint.rs` as `impl Gradient { pub fn srgb_stops(&self, line_length: f64) -> Vec<(f64, Rgba)> }`.
  - Add `GradientShape::linear_angle` (already in types/render/paint.rs:97) as `Gradient::line(rect) -> (Point, Point)` in types.
  - Engine and svg both call them. Delete `css_stops` and the angle math in svg fill.rs:79-105.
- breaking: no   kpi: -   size: S
- confidence: high

### ST-14 SVG computes a different paint area than the window for shapes
- kind: BUG
- where: crates/kivact-svg/src/shape.rs:18-27, :156 (`bounds_of`; `ArcTo` contributes only its end point); crates/kivact-engine/src/lower/scene/geometry.rs:135-150
- evidence: the engine fills a `DrawingShape::Basic/Rounded` gradient over the shape's reference `rect`, a path over `shape.bounds()` (arcs converted to cubics), and pads a zero-width/zero-height stroke by half the stroke width. The SVG writer uses the outline's point bounds for every shape and pads nothing. A `circle(pct(30))` or `polygon` filled with a gradient, a path with `ArcTo`, or a straight stroked line with a gradient renders differently. This hits the plot export (lines with gradient strokes).
- fix: add `impl ShapePaint { pub fn paint_area(&self) -> Rect }` and `impl DrawingShape { pub fn to_shape(&self) -> scene::Shape }` in kivact-types/src/scene/recording.rs, moved from engine/lower/scene/geometry.rs:135-150 and engine/lower/scene_shape.rs. Engine and svg both call them.
- breaking: no   kpi: -   size: S
- confidence: high

### ST-15 SVG silently drops layer composite and text fill, stroke, variations, width and kerning
- kind: BUG (G-05)
- where: crates/kivact-svg/src/emit.rs:175-197 (`layer_attributes` checks blend/filter/masks/transform but not `composite`); crates/kivact-svg/src/text.rs:17-58 (reads neither `run.fill`, `run.stroke`, `run.variations`, `run.font_width` nor `run.kerning`)
- evidence: a `Layer::composite(Composite::DestinationOut)` (an eraser) is written as a plain group. Gradient-filled or outlined text is written as solid `run.color`. Other unsupported features (blend, features, decorations) fail the export, so the writer is inconsistent with itself.
- fix:
  - In `layer_attributes`: `if layer.composite != Composite::SourceOver { return Err(unsupported("a composite operation on a layer")) }` (after ST-17: `layer.mix != Mix::Normal`).
  - In `text`: write `fill` through `paint_attributes` (gradients are supported), `stroke` via `stroke=`/`stroke-width=` + `paint-order`, `font-stretch` from `font_width`, and `font-variation-settings` in a `style` attribute.
  - Add one test per field.
- breaking: no   kpi: -   size: S
- confidence: high

### ST-16 kivact-svg re-implements engine geometry (outline of CSS shapes, shift, bounds)
- kind: DUP / STRUCT
- where: crates/kivact-svg/src/shape.rs:31-151 (`basic`, `rounded`, `ellipse`, `shift`), crates/kivact-engine/src/lower/shape.rs:170-240 (`clip_path`, `BoxShape`), crates/kivact-engine/src/lower/path.rs (`shifted`)
- evidence: the same `ClipShape::Circle` reference-radius formula (`hypot/√2`), inset and polygon mapping are written twice. The crate boundary itself is clean (types + png + `Resources`); the duplication exists because the pure interpretation lives in engine::lower.
- fix: move `clip_path`, `BoxShape::of(..).shape()` and `shifted` into kivact-types as `ClipShape::in_rect(&self, rect) -> Shape`, `Corners::shape(rect) -> Shape` and `Shape::translated(p)`. kivact-svg/shape.rs keeps only `path_data` (SVG `d` writing) and the arc splitting. **Keep kivact-svg as a crate.** Its input is `DrawList` + `Resources`, its output is `String`, and it has one reason to change. Do not switch it to `RenderScene`: that would lose text-as-text.
- breaking: no   kpi: K6 (small)   size: S
- confidence: high

### ST-17 `Layer` (and `DrawingLayer`) keep `blend` and `composite` as an implicit exclusive pair
- kind: STATE
- where: crates/kivact-style/src/draw/layer.rs:51-52, :124-136; crates/kivact-types/src/scene/recording.rs:49-51
- evidence: `blend()` resets `composite` to SourceOver and `composite()` resets `blend` to Normal ("the last wins"). Two fields hold one fact. Canvas `globalCompositeOperation` is one property covering both.
- fix: add `pub enum Mix { Normal, Blend(BlendMode), Composite(Composite) }` in kivact-types (pixel.rs), with `Layer::mix(Mix)` replacing `blend`/`composite` and the field `DrawingLayer.mix`. `BlendMode::Normal` stays, for the element style `mix_blend_mode`. A layer uses `Mix::Normal`, and `Mix::Blend(BlendMode::Normal)` is normalized to `Mix::Normal` by the constructor.
- breaking: yes   kpi: -   size: S
- confidence: high

### ST-18 `Draw` is built incomplete and filled in afterwards (measurer, regions), a call-order protocol
- kind: STATE (S-7)
- where: crates/kivact-style/src/draw/recorder.rs:26 (`measurer: Option<..>` "None before the engine is there"), :47 (`set_text_measurer`, `#[doc(hidden)] pub`), :108-127 (`record` doc: "first install set_text_measurer and declare_regions, then …"); `Draw::theme: Theme` cloned per record (:116)
- fix:
  - `pub fn record(size: Size, cx: RecordContext<'_>, f: impl FnOnce(&mut Draw)) -> Recording`, with `pub struct RecordContext<'a> { pub theme: &'a Theme, pub shaper: &'a dyn TextShaper, pub regions: &'a HashMap<RegionId, RegionSemantics> }`.
  - `Draw` borrows them (`Draw<'a>`, hidden lifetime: closures take `&mut Draw`, so users never write it).
  - Delete `set_text_measurer`, `declare_regions` and the `Option`.
- breaking: no (hidden API)   kpi: K3 (no Theme clone per draw)   size: S
- confidence: high

### ST-19 `LayoutProps.hidden` is always false: a second "display: none" fact
- kind: DEAD / STATE (S-4)
- where: crates/kivact-types/src/layout.rs:432, written only as `hidden: false` at crates/kivact-style/src/resolve/compile.rs:126; read by crates/kivact-layout/src/style/convert.rs:306 (`l.hidden || hidden`), crates/kivact-core/src/view/tree.rs:911, crates/kivact-core/src/devtools/node.rs:417
- evidence: the real fact is the engine `Hidden` prop (engine/src/host/write/typed.rs:191), and nothing ever writes `true` to the layout field.
- fix: delete the field and the three reads (convert.rs uses only the `hidden` parameter).
- breaking: no   kpi: K2 (tiny)   size: S
- confidence: high

### ST-20 `kivact-types` mixes five concerns; split it along its consumers
- kind: STRUCT
- where: crates/kivact-types/src/render/* (1.3k, consumers: engine + arvane-* only, plus 4 names in style/core), prop_interp.rs (507, engine only), cell.rs (320: `Mutex`/`RefCell` helpers), process.rs (26: process-global `AtomicUsize`), lib.rs:122 (crate doc says "pure values")
- fix:
  - **New crate `arvane-protocol`**: `render/{composition,scene,paint,effect,semantics}.rs` + `render/text.rs` (as `ResolvedSpan`/`RichText`). The engine produces it and arvane-* consume it. Inputs and outputs are a handful of types (`RenderScene`, `Composition`, `SemanticsUpdate`, `CpuFrame`, `FrameReport`), with one reason to change.
  - `ShapedText`/`TextLine`/`InlineBox(Placement)`/`ExportFrame`/`ExportedFrame` stay in types (they are public through `kivact::draw` and drawables). `Composite` folds into `Mix` (ST-17).
  - **Move `prop_interp.rs`** to `kivact-engine/src/model/state_motion/interp.rs`.
  - **New leaf crate `kivact-cell`** (std only) for `cell.rs` + `process.rs`: helpers with interior mutability and process-global state, used by both Kivact and Arvane crates. They are not vocabulary and they make types non-pure.
  - Re-export nothing of them from the facade.
- breaking: no (internal)   kpi: K6/compile time (arvane does not rebuild on prop changes and vice versa)   size: M
- confidence: medium (crate dependency graph of arvane-* not fully traced)

### ST-21 Same name, different meaning (VOC-3)
- kind: CONTRA / API
- where:
  - `Visibility`: types/layout.rs:513 vs types/embed.rs:8
  - `Transform`: types geometry affine vs style/transform.rs CSS list (public as `kivact::draw::Transform`)
  - `Shape`: types/scene/shape.rs, style/effect/shape.rs, engine
  - `Gradient`: types/paint.rs:229, types/render/paint.rs, style/paint/gradient.rs
  - `Paint`: style vs types/render
  - `FilterGraph`/`FilterInput`: types/effect vs style/effect
  - `Look`: kivact-engine/src/layer.rs, kivact-ui/src/variants.rs:199, kivact-ui-basic/src/button.rs:11
  - `SpanStyle`/`TextSpan`: ST-11
  - root-level `Kind` (motion)
- fix (renames inside types/engine so the public names stay):
  - `embed::Visibility` → `EmbedVisibility`
  - types `Transform` → `Affine`
  - types `scene::Shape` → `Outline`
  - types `Gradient` → `GradientSpec`, render `Gradient` → `arvane_protocol::RasterGradient`, render `Paint` → `RasterPaint`
  - types `FilterGraph` → `FilterGraphSpec`
  - engine `Look` → `LayerTraits`
  - `motion::Kind` → `TransitionKind`
- breaking: no (internal names), yes for kivact-test users of `kivact_types` paths   kpi: -   size: S
- confidence: high

### ST-22 The public style vocabulary is split between `kivact::style` and `kivact::draw`, with non-CSS names
- kind: API
- where: crates/kivact-style/src/facade.rs, crates/kivact-core/src/draw.rs:9-35, crates/kivact/src/prelude.rs:30-37; crates/kivact-style/src/paint.rs:20-22 (doc); crates/kivact-style/src/theme/value.rs:599 (`Color::resolve` returns `Rgba`, which the facade does not export)
- evidence:
  - `style().shadow(Shadow::new(..)).background(linear_gradient(90)..).transform(Transform::new()..).clip_path(Shape::..)` needs imports from both modules.
  - `BorderImage`/`BorderImageRepeat` are exported from both.
  - Setter names mix CSS, Tailwind and own names: `weight`, `italic`, `radius`, `ring`, `basis`, `grow`, `shrink`, `wrap`, `direction`, `pressed`, next to `font_size`, `font_family`, `mix_blend_mode`. style/text.rs:7 claims "CSS's property names in snake case".
- fix:
  - All CSS value types (`Shadow`, `Filter`, `Transform`, `Paint`, gradients, `Shape`, `CornerRadius`, `BlendMode`, `Mask*`, `BorderImage*`, `PaintShader`, `FilterShader`, `FilterGraph`) live in `kivact::style`. `kivact::draw` keeps only painter types (`Draw`, `Path`, `PathContour`, `Layer`, `Mix`, `Stroke`, `LineCap`, `LineJoin`, `FillRule`, `TextFormat`, `TextBaseline`, `TextMetrics`, paragraphs, `ImageOptions`, `Pixels`, `RegionId`, `Offscreen`).
  - Renames: `weight`→`font_weight`, `italic(bool)`→`font_style(FontStyle::Italic)`, `radius*`→`border_radius*`, `ring*`→`outline*` (`outline_offset`), `basis/grow/shrink/wrap/direction`→`flex_basis/flex_grow/flex_shrink/flex_wrap/flex_direction`, `pressed`→`active`.
  - `Color::resolve` becomes `pub(crate)`. Kits use `components(&Theme)`.
- breaking: yes   kpi: -   size: M (mechanical, many call sites)
- confidence: high (see O-5 for the rename churn)

### ST-23 `EmbedOptions.prefer_gpu` plus `GpuPreference.start`: two fields decide the renderer
- kind: STATE
- where: crates/kivact-types/src/embed.rs:55, crates/kivact-types/src/gpu.rs:50-60, crates/arvane/src/env.rs:32-61
- evidence: `RendererStart::resolve(forced, prefer_gpu, start)`: a bool and an enum combine into one decision.
- fix: add `GpuStart::Never` (CPU only) and delete `prefer_gpu` (embed and `Shell::prefer_gpu`).
- breaking: yes (embed option)   kpi: -   size: S
- confidence: high

### ST-24 Colors are 4×f64 everywhere
- kind: INEFF
- where: crates/kivact-types/src/color.rs:6-16 (`Rgba` 32 B), crates/kivact-style/src/color/value.rs:34-45 (`Color` ≈ 48 B), types/paint.rs:298 (`Border` = 4 Rgba = 128 B of colors), types/text/properties.rs:107 (`TextStyle`: 6 × `Option<Rgba>`)
- evidence: per-node `PaintProps`, `TextStyle`, `StateStyles` deltas and every `DrawCmd` carry f64 colors that the renderers convert to f32 or u8.
- fix: `Rgba { r: f32, g: f32, b: f32, a: f32 }` (16 B), `SpaceColor` channels f32, and `Color` ≤ 24 B (the `Var` pointer variant is 8 B + f32 alpha). Color math in `color_space.rs` stays f64 internally (convert at its boundary).
- breaking: yes (public `Color::from_rgba_f64` stays, takes f64)   kpi: K2   size: M
- confidence: medium (sizes computed by hand, not measured)

### ST-25 naga is linked into builds without GPU; minor shader nits
- kind: INEFF (K6) / STATE
- where: crates/kivact-style/Cargo.toml (`kivact-shader` unconditional), crates/kivact/Cargo.toml (`gpu` feature), crates/kivact-style/src/shader/program.rs:13 (`err.line == 0` as "no position")
- evidence: with `default-features = false`, a CPU-only app still links naga (wgsl-in + validator) to validate shaders it can never run (the CPU draws `fallback`).
- fix:
  - kivact-style feature `shader-validation = ["dep:kivact-shader"]`, enabled by `kivact/gpu`. Without it, `PaintShader::wgsl` runs only `kivact_shader`-free checks (non-empty source) and returns a program whose `module` is the source; the CPU path draws the fallback (O-4).
  - `ShaderError { position: Option<(u32, u32)> }`.
  - **kivact-shader itself is a clean boundary; keep it.**
- breaking: no   kpi: K6   size: S
- confidence: medium (binary delta not measured)

### ST-26 `Theme` is a fat value: cloned per draw, compared deeply, rebuilt from scratch
- kind: INEFF
- where: crates/kivact-style/src/theme/value.rs:204-218 (17 `Color`s ≈ 0.8 KB, a `String` family, `IconSet`, extensions), :396-449 (`light()` builds `IconSet::standard()` per call), crates/kivact-style/src/icon.rs:157; crates/kivact-style/src/draw/recorder.rs:116
- fix: `pub struct Theme(Arc<ThemeData>)`. `PartialEq` short-circuits on `Arc::ptr_eq`. Builders use `Arc::make_mut`. `light()`/`dark()` clone `static LazyLock<Theme>`s.
- breaking: no   kpi: K3, K1   size: S
- confidence: high

### ST-27 Two copies of a TypeId-keyed heterogeneous map
- kind: DUP
- where: crates/kivact-style/src/theme/value.rs:110-185 (`Entry`/`Extensions`, `Arc`), crates/kivact-ui/src/variants.rs:37-108 (`Extension`/`Extensions`, `Rc`)
- fix: one `pub struct TypeMap<P>` (with `eq`/`debug` fn pointers; `P` = `Arc` or `Rc` chosen by two type aliases `SharedTypeMap`/`LocalTypeMap`) in `kivact-types::type_map`. Both use it.
- breaking: no   kpi: -   size: S
- confidence: high

### ST-28 Rules generate noise in these crates
- kind: PROCESS
- where:
  - 174 `xtask-allow(state-fields)` in kivact-style, about 160 of them the same sentence "option: one authored property; absent means the inherited or default value" (sheet/fields.rs, text.rs).
  - 11 identical `xtask-allow(api-shape): Shared immutable payload…` in kivact-types (paint.rs, motion.rs, effect/shader.rs, render/composition.rs).
  - 8 near-identical `internal-contract` notes in render/shaped.rs:224-291.
  - Boilerplate docs to satisfy missing_docs: color.rs:6-15 (`/// Rgba.`, `/// R.`).
- evidence: the state-fields rule (architecture.md:94) targets lifecycle state, but authored value records (every CSS property is an `Option`) need a marker per field. That is about 400 lines of comments with zero information.
- fix:
  - xtask `state-fields` skips a struct whose doc carries `#[doc = "kivact: passive"]`; Style itself disappears with ST-08.
  - `api-shape` accepts `Arc<[T]>`/`Arc<str>` fields without a marker.
  - Delete the one-word doc comments and rely on crate-level `#![allow(missing_docs)]` for `kivact-types` items re-exported with their own facade docs.
- breaking: no   kpi: -   size: S
- confidence: high

### ST-29 Docs contradict the code
- kind: CONTRA
- where:
  - crates/kivact-style/src/text.rs:59 ("`Draw`'s `TextStyle`": the type is `TextFormat`).
  - crates/kivact-style/src/theme.rs:4 ("`provide_theme` is in `kivact::view`": it is exported from `kivact::style`, crates/kivact-core/src/style.rs:15).
  - crates/kivact-style/src/paint.rs:20-22 ("except `Background` and `BorderStyle`, which only a style takes": the style facade also exports `BorderImage*`).
  - crates/kivact-style/src/text.rs:7 (CSS names, ST-22).
  - crates/kivact-style/src/lib.rs:64 ("Visibility of a layout box" vs the `embed::Visibility` homonym).
- fix: fix each sentence in the same change as ST-11/ST-22. Prefer deleting module-level prose that restates setter docs (sheet.rs:1-89 duplicates the setter docs).
- breaking: no   kpi: -   size: S
- confidence: high

### ST-30 Leftovers and over-generalization
- kind: DEAD
- where:
  - `ScrollbarWidth::Auto` and `::Thin` both resolve to 6.0 (resolve/paint.rs:368).
  - `FilterGraph` handle validation uses a process-global `AtomicU64` (style/effect/filter_graph.rs:24) and stores `errors: Vec<String>` in the builder.
  - `Style.duplicates` is part of `PartialEq` (sheet/fields.rs:546), so two equal styles differ by a diagnostic counter and memo dedupe fails.
  - Icon's private enum is named `InputProp` and `kivact_runtime::Prop` is aliased `as Input` (style/icon.rs:5,32).
- fix:
  - Delete `ScrollbarWidth::Thin`.
  - `FilterInput` becomes a plain `u32` result index (plus `SOURCE`/`SOURCE_ALPHA`). The builder checks only `index < len` and reports one error. Delete the global counter and the graph id: catching a handle from a different graph is not worth a process-global counter.
  - Exclude `duplicates` from `PartialEq` (it vanishes with ST-08).
  - Rename the icon enum to `IconSource`.
- breaking: no   kpi: -   size: S
- confidence: high

### ST-31 Paint lengths silently turn `auto` and content sizes into 0
- kind: BUG (G-05)
- where: crates/kivact-style/src/length.rs:136-139 (`to_len`: `parts().unwrap_or((0.0, 0.0))`), used for radii, gradient centers and stops, background size and position, clip paths, transform origins
- evidence: `radius(auto())` or `.position(Length::MaxContent, 0)` compiles to 0 with no diagnostic, while layout lengths report `misplaced_content`.
- fix: `fn to_len(self, report: &mut u32) -> Len` counts non-px/% values. `CompiledStyle.misplaced_content` (renamed `misplaced_lengths`) covers paint positions too. One `Unsupported` per node, as today.
- breaking: no   kpi: -   size: S
- confidence: high

## 3. Target structure for this area

```
kivact-types        (~9k) vocabulary only: geometry (Affine, Matrix4, Edges<T>, Corners), color (Rgba f32,
                    ColorSpace, SpaceColor, Color incl. Var), theme (Var<T>, VarTable, Theme value, ColorScheme),
                    length (Length, TextLength, Len), ids, asset, input, env/platform/window/embed/gpu,
                    semantics, StyleState/StateSet, node-props contract (LayoutProps, PaintProps, PaintDelta,
                    StateStyles, TextStyle, ComputedTextStyle), motion values, scene recording (DrawList,
                    DrawCmd, Mix) + pure interpretation helpers (DrawingShape::to_shape, ShapePaint::paint_area,
                    ClipShape::in_rect, Gradient::srgb_stops/line), effect specs, TypeMap, TextShaper trait,
                    shaped-text snapshots and InlineBox, LayoutCheck.
                    Boundary: pure data + pure functions, no interior mutability, no globals, no deps.
arvane-protocol     (new, ~1.3k from types/render) RenderScene/RenderCmd, Composition, CpuFrame, SemanticsTree,
                    RasterPaint/RasterGradient, ResolvedSpan/RichText. Producer: kivact-engine. Consumers: arvane-*.
                    Boundary: the frame protocol; one reason to change (renderer contract).
kivact-cell         (new, ~350, std only) ReportOnce(Set), Sent, Generation, Mailbox, LocalMailbox, Registry,
                    Recent, DetachedThreads. Boundary: sync/latch helpers with interior mutability used on both
                    sides; nothing else may go in.
kivact-style        Style = declaration list from one property table (ST-08), state styles (ST-05), theme access
                    (use_theme, try_use_theme, install_theme, provide via core), compile (Style → props) with
                    var resolution, Draw painter (RecordContext), Path (kurbo), shader wrappers (feature
                    shader-validation), CSS value builders (Shadow, Filter, Transform, Paint, gradients, Shape,
                    Mask, BorderImage, FilterGraph builder). Re-exports types' values; defines no wire twins.
kivact-shader       unchanged (optional dep of style).
kivact-svg          unchanged crate; shape.rs shrinks to path-data writing; fill.rs uses Gradient::srgb_stops;
                    emit/text cover composite + text fill/stroke.
kivact-engine       gains prop_interp (from types); loses lower/shape clip_path, lower/fill stops (moved to types).
```

Deleted: `THEMED`, `ThemeColor`, `Color::of`, `ColorToken`, `Palette`, `Spacing`, `Radii`, the size half of `Typography`, `Dimension`, `TextLen`, `GridFlow`, `Insets`, `ThemeHint`, core `AvailableSpace`, style `SpanStyle`/`CompiledSpanStyle`/`TextExtras`/`DecorationStyleValue`, types `SpanProps`, sheet `fields.rs` groups and `group_or_default!`, `describe.rs` tables and `DELTA`, `unsupported_in_states`, `LayoutProps.hidden`, `ScrollbarWidth::Thin`, `EmbedOptions.prefer_gpu`, `Layer::blend/composite` pair, `Draw::set_text_measurer`, `NEXT_GRAPH`, `Color::components()/mix()` (theme-less), kit `ColorName` enums and kit `Tokens` color fields.

## 4. Public API

**One styling model, explained in five lines to a CSS/Tailwind developer:**
1. `style()` builds a declaration list. Later declarations win, and `.style(a).style(b)` layers merge the same way (the cascade).
2. Values are CSS-typed: `Length` (number = px, `pct`, `calc` via `+`/`-`), `Color`, and the CSS value builders, all from `kivact::style`.
3. Theme values are **variables**: `color::ACCENT`, `space::MD`, `radius::MD`, `basic::PRIMARY`. They go anywhere a value goes, re-resolve when the theme changes, and need no closure (`var(--x)`).
4. States are nested styles that accept every property: `.hover(..)`, `.active(..)`, `.focus(..)`, `.focus_visible(..)`, `.disabled(..)`, `.selected(..)`, `.group_hover(..)`, `.group_focus(..)`, `.enter(..)`, `.exit(..)`.
5. A component look is picked with `.variant(v)`. A theme restyles components through recipe tables (`fn(&Look<S>) -> Style`; ui-owned). Reactive styles are `.style(move || ..)`, for app state only.

Before / after:

```rust
// BEFORE: theme size needs a closure, kit color needs the ThemeColor enum
.style(move || {
    let t = use_theme();
    t.with(|t| style().padding(t.space().md).radius(t.radius().md))
        .background(ColorName::Primary)
})
// AFTER
.style(style().padding(space::MD).border_radius(radius::MD).background(basic::PRIMARY))
```

```rust
// BEFORE (kit): one color = Tokens field + ColorName variant + match arm + Palette copy
pub struct Tokens { pub primary: Color, .. }
pub enum ColorName { Primary, .. }
impl ThemeColor for ColorName { fn resolve(self, t: &Theme) -> Color { match self { ColorName::Primary => t.ext::<Tokens>().primary, .. } } }
let colors = Palette { accent: p.primary, .. };
// AFTER
static PRIMARY_VAR: Var<Color> = Var::new("basic.primary", rgb(0x171717));
pub const PRIMARY: Color = Color::var(&PRIMARY_VAR);
pub fn theme(scheme: ColorScheme) -> Theme {
    Theme::light().set(&PRIMARY_VAR, rgb(if dark { 0xe5e5e5 } else { 0x171717 }))
                  .set(&color::ACCENT_VAR, PRIMARY)          // alias, like --accent: var(--primary)
}
```

```rust
// BEFORE: hover underline is impossible in a state style; recipe reads PartState
if input.is_hovered() && !input.is_disabled() { s.text_decoration(TextDecorationLine::UNDERLINE) } else { s }
// AFTER
s.hover(style().text_decoration_line(TextDecorationLine::UNDERLINE))
```

```rust
// BEFORE: two modules for one style
use kivact::style::{style, pct}; use kivact::draw::{Shadow, linear_gradient, Transform};
// AFTER
use kivact::style::{style, pct, Shadow, linear_gradient, Transform};
```

```rust
// BEFORE: Draw layer
Layer::new().blend(BlendMode::Multiply).composite(Composite::DestinationOut) // last wins, silently
// AFTER
Layer::new().mix(Mix::Composite(Composite::DestinationOut))
```

Public names:
- **delete:** `ThemeColor`, `ColorToken`, `Color::of`, `Color::token`, `Color::components` (theme-less), `Color::mix` (theme-less), `Color::resolve` (pub), `theme::{Palette, Spacing, Radii}`, `Theme::{colors, space, radius, with_colors, with_space, with_radius}`, `use_tokens` (vars replace token reads; `Theme::ext` remains for tables), `SpanStyle`, `StyledRange.decoration`'s separate `TextDecoration` (→ `TextStyle`), `ScrollbarWidth::Thin`, `Layer::blend`, `Layer::composite`, `draw::Composite` (→ `Mix`), `draw::text_run` (internal), `draw::TextRun` (wire type leak), duplicate `kivact::style::{BorderImage, BorderImageRepeat}` or `kivact::draw::` ones (keep style), `EmbedOptions.prefer_gpu`.
- **add:** `Var<T>`, `Color::var`, `Length::var`, `Theme::{set, get}`, modules `style::{color, space, radius, font}` of core vars, `Mix`, `ColorScheme`, `StyleState`, `try_use_theme`.
- **rename:** `ThemeMode`→`ColorScheme`, `weight`→`font_weight`, `italic`→`font_style`, `radius*`→`border_radius*`, `ring*`→`outline*`, `basis/grow/shrink/wrap/direction`→`flex_*`, `pressed`→`active`, `Direction`→`FlexDirection`, `GridFlow`/`GridAutoFlow`→`GridAutoFlow`, `components_in`→`components`, `mix_in`→`mix`.
- **move:** CSS value types from `kivact::draw` to `kivact::style` (ST-22).

## 5. Open questions for the owner

- **O-1 Where theme variables resolve.** (a) In `compile_style`, as today: a theme switch reruns every style effect, which is simplest. (b) In engine lowering: the wire props keep `Color` with `Var`, and the theme is an engine-inherited node prop. A theme switch then becomes a repaint with zero reactive work, and the Color/Rgba type pairs (ST-10) disappear completely, but it touches the engine and arvane lowering (L). Recommendation: (b), but it needs the engine auditor's agreement.
- **O-2 State styles with layout properties (ST-05).** Is the hover-triggered recompile on opted-in nodes acceptable (CSS behaviour), or should non-paint properties in host states be rejected?
- **O-3 SVG unsupported policy.** Fail the whole export (current for blend, features, decorations), or write the closest drawing and report each drop as `Unsupported`? Tabular numbers (`tnum`) in plot labels currently fail the whole export.
- **O-4 CPU-only builds without naga (ST-25).** Shader syntax errors would surface only in GPU builds. Accept?
- **O-5 CSS renames (ST-22).** They touch every kit and the downstream apps (mechanical). Confirm the full CSS naming, or keep the Tailwind-style short names (`ring`, `grow`) and only fix the docs?
