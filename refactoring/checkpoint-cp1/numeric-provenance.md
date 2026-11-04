# CP1 numeric provenance: b6e7ff2 → 97689c4

Read-only source investigation. No checks, source or tool changes. No numeric delta accepted.

## Contract distinctions

- Point/Size/Rect and their math widen f32→f64; Taffy/Parley remain f32 at explicit import/export boundaries. Later f64 arithmetic may differ despite identical native f32 endpoints.
- `$.value.fields.amount` is LayoutWarning.amount (1132 deltas in the original review), not scroll displacement. ScrollChanged uses Point offset and Size viewport/content.
- Semantics value_min/max/now were already Option<f64>; SemanticSources remains Option<InputProp<f64>>. Option<Finite> is f64 validation/signed-zero normalization, not a precision expansion.
- Scalar track geometry widens while binding/range/fraction output were alreadyf64. This is a possible upstream reason for semantic value_now deltas requiring exact replay.
- Glyph positions/advances/font metrics widen; font/glyph identity and order must remain exact. Uncut native_inline_bounds intentionally computes edges in Parley f32 before importing; custom output math stays f64.
- f32MAX→f64MAX is an expanded sentinel/domain, not a rounding artifact.

## Exact declaration and producer evidence

### geometry — widened

Point/Size/Rect storage and operations change f32→f64. A f32 layout boundary does not force later sums, subtractions, transforms, intersection or scroll arithmetic back to f32.

- b6e7ff2 `crates/kivact-types/src/geometry.rs`: L10 `#[derive(Clone, Copy, Debug, Default, PartialEq)]`; L11 `pub struct Point {`; L12 `/// Horizontal position.`; L13 `pub x: f32,`; L14 `/// Vertical position.`; L15 `pub y: f32,`; L16 `}`; L19 `#[derive(Clone, Copy, Debug, Default, PartialEq)]`
- 97689c4 `crates/kivact-types/src/geometry.rs`: L5 `#[derive(Clone, Copy, Debug, Default, PartialEq)]`; L6 `pub struct Point {`; L7 `/// Horizontal position.`; L8 `pub x: f64,`; L9 `/// Vertical position.`; L10 `pub y: f64,`; L11 `}`; L14 `#[derive(Clone, Copy, Debug, Default, PartialEq)]`

### host_events — widened

Trace rect/size/offset payload is shared geometry. The trace amount group is LayoutWarning, NOT scroll amount: first review contains581 same-projection and551 changed LayoutWarning.amount records.

- b6e7ff2 `crates/kivact-wire/src/events.rs`: L118 `/// (`TextChanged`, `SelectionChanged`), the input method of a node with an`; L119 `/// IME session (`Ime`), geometry (`ScrollChanged`,`; L120 `/// `LayoutChanged`), assistive technology and dismissal (`A11y`), time and platform answers (`FrameTick`,`; L121 `/// `EffectResult`), the window and the platform (`EnvChanged`,`; L124 `/// own `gpu-interop` never sends them and ignores them) and reports`; L125 `/// (`HostFailure`, `TurnsCut`, `AssetFailed`, `DrawDropped`, `LayoutWarning`).`; L126 `#[derive(Clone, Debug, PartialEq)]`; L216 `},`
- 97689c4 `crates/kivact-engine/src/contract/events.rs`: L12 `/// (`TextChanged`, `SelectionChanged`), the input method of a node with an`; L13 `/// IME session (`Ime`), geometry (`ScrollChanged`,`; L14 `/// `LayoutChanged`), assistive technology and dismissal (`A11y`), time and platform answers (`FrameTick`,`; L15 `/// `EffectResult`), the window and the platform (`EnvChanged`,`; L18 `/// own `gpu-interop` never sends them and ignores them) and reports`; L19 `/// (`HostFailure`, `TurnsCut`, `AssetFailed`, `DrawDropped`, `LayoutWarning`).`; L20 `#[derive(Clone, Debug, PartialEq)]`; L82 `},`

### layout_findings — widened

LayoutFinding.amount and tolerances widen f32→f64; warning forwards this amount unchanged.

- b6e7ff2 `crates/kivact-wire/src/check.rs`: L39 `/// How far (px) a text may reach outside its limit.`; L40 `pub overflow_tolerance: f32,`; L41 `/// How far (px) the middle of a line may be off the middle of its`; L44 `/// always allowed.`; L45 `pub center_tolerance: Option<f32>,`; L46 `/// The roles whose text is a single line.`; L54 `LayoutCheck {`; L55 `overflow_tolerance: 1.0,`
- 97689c4 `crates/kivact-types/src/check.rs`: L32 `/// How far (px) a text may reach outside its limit.`; L33 `pub overflow_tolerance: f64,`; L34 `/// How far (px) the middle of a line may be off the middle of its`; L37 `/// always allowed.`; L38 `pub center_tolerance: Option<f64>,`; L39 `/// The roles whose text is a single line.`; L47 `LayoutCheck {`; L48 `overflow_tolerance: 1.0,`

### semantic_contract — unchanged_f64_with_wrapper

All three were Option<f64> already. New Option<Finite> wraps f64, rejects nonfinite, normalizes signed zero. A precision delta cannot be justified by sink widening because the sink did not widen.

- b6e7ff2 `crates/kivact-wire/src/semantics.rs`: L17 `/// Numeric lower bound.`; L18 `pub value_min: Option<f64>,`; L19 `/// Numeric upper bound.`; L20 `pub value_max: Option<f64>,`; L21 `/// Current numeric value.`; L22 `pub value_now: Option<f64>,`; L23 `/// Live announcement policy.`
- 97689c4 `crates/kivact-types/src/semantics/node.rs`: L22 `/// Numeric lower bound.`; L23 `pub value_min: Option<Finite>,`; L24 `/// Numeric upper bound.`; L25 `pub value_max: Option<Finite>,`; L26 `/// Current numeric value.`; L27 `pub value_now: Option<Finite>,`; L28 `/// Checked state; None for a node that is not a toggle.`; L90 `value_text: None,`

### semantic_sources — unchanged_f64

SemanticSources holds Option<InputProp<f64>> in both revisions; needs actual caller arithmetic provenance for changed numbers.

- b6e7ff2 `crates/kivact-core/src/view/common.rs`: L267 `pub(crate) struct SemanticSources {`; L268 `pub(crate) value_min: Option<InputProp<f64>>,`; L269 `pub(crate) value_max: Option<InputProp<f64>>,`; L270 `pub(crate) value_now: Option<InputProp<f64>>,`; L271 `pub(crate) live_region: Option<InputProp<kivact_wire::LiveRegion>>,`
- 97689c4 `crates/kivact-core/src/view/common.rs`: L267 `pub(crate) struct SemanticSources {`; L268 `pub(crate) value_min: Option<InputProp<f64>>,`; L269 `pub(crate) value_max: Option<InputProp<f64>>,`; L270 `pub(crate) value_now: Option<InputProp<f64>>,`; L271 `pub(crate) live_region: Option<InputProp<kivact_types::LiveRegion>>,`

### semantic_publish — validation_not_widening

Old source.get() directly published f64; new applies Finite::new. Existing finite nonzero f64 should preserve exact value at this layer.

- b6e7ff2 `crates/kivact-core/src/view/mount/semantics.rs`: L113 `SemanticsProperties {`; L114 `value_min: sources.value_min.as_ref().map(kivact_reactive::Prop::get),`; L115 `value_max: sources.value_max.as_ref().map(kivact_reactive::Prop::get),`; L116 `value_now: sources.value_now.as_ref().map(kivact_reactive::Prop::get),`; L117 `live_region: sources.live_region.as_ref().map(|s| s.get()),`
- 97689c4 `crates/kivact-core/src/view/mount/semantics.rs`: L133 `Semantics {`; L134 `value_min: sources`; L135 `.value_min`; L137 `.map(kivact_reactive::Prop::get)`; L138 `.and_then(Finite::new),`; L139 `value_max: sources`; L140 `.value_max`; L142 `.map(kivact_reactive::Prop::get)`

### shaped_text — widened

Glyph positions/advances/font metrics/normalized coordinates and shaped baseline widen f32→f64. Font/glyph identities and vector ordering remain separate exact invariants.

- b6e7ff2 `crates/kivact-types/src/protocol/shaped.rs`: L11 `#[derive(Clone, Copy, Debug, Default, PartialEq)]`; L12 `pub struct Glyph {`; L13 `/// Font-specific glyph identifier.`; L15 `/// Horizontal baseline position.`; L16 `pub x: f32,`; L17 `/// Vertical baseline position.`; L18 `pub y: f32,`; L19 `/// Logical advance.`
- 97689c4 `crates/kivact-types/src/render/shaped.rs`: L11 `#[derive(Clone, Copy, Debug, Default, PartialEq)]`; L12 `pub struct Glyph {`; L13 `/// Font-specific glyph identifier.`; L15 `/// Horizontal baseline position.`; L16 `pub x: f64,`; L17 `/// Vertical baseline position.`; L18 `pub y: f64,`; L19 `/// Logical advance.`

### scalar_track — mixed_contract_changed_upstream_math

Bindings/range endpoints/fraction return are f64 already. Geometry pos/thumb/length/from were f32 and become f64. Scalar value_now can differ due to widened geometry arithmetic upstream; caller output still requires replay.

- b6e7ff2 `crates/kivact-interaction/src/scalar/track.rs`: L1 `//! Role: Map the pointer along a slider's track to the value of a thumb.`; L2 ``; L11 ``; L12 `/// One thumb as the pointer moves it: its writer, range and element.`; L13 `#[derive(Clone, Copy)]`; L22 `impl Target {`; L23 `/// Normalize by the current range and invoke the writer, as`; L24 `/// [`Scalar::set`](super::Scalar::set) does.`
- 97689c4 `crates/kivact-interaction/src/scalar/track.rs`: L1 `//! Role: Map the pointer along a slider's track to the value of a thumb.`; L2 ``; L11 ``; L12 `/// One thumb as the pointer moves it: its writer, range and element.`; L13 `#[derive(Clone, Copy)]`; L22 `impl Target {`; L23 `/// Normalize by the current range and invoke the writer, as`; L24 `/// [`Scalar::set`](super::Scalar::set) does.`

### diagnostic_math — widened

Clipping max and center displacement operate on widened shaped bounds/geometry; amounts can differ even where Parley and Taffy stay f32.

- b6e7ff2 `crates/kivact-engine/src/check/findings.rs`: L17 `/// place.`; L18 `const PIXEL_ROUNDING: f32 = 0.5;`; L19 ``; L98 `let vp = visual_parent(host, &up, check);`; L99 `let mut push = |rule, amount, text_rect, limit_rect| {`; L100 `out.push(LayoutFinding {`; L102 `rule,`; L103 `amount,`
- 97689c4 `crates/kivact-engine/src/check/findings.rs`: L17 `/// place.`; L18 `const PIXEL_ROUNDING: f64 = 0.5;`; L19 ``; L98 `let vp = visual_parent(host, &up, check);`; L99 `let mut push = |rule, amount, text_rect, limit_rect| {`; L100 `out.push(LayoutFinding {`; L102 `rule,`; L103 `amount,`

### overflow_math — widened

Overflow uses max of edge subtractions; round each old f32 operation, not merely the final output, when reproducing old behavior.

- b6e7ff2 `crates/kivact-engine/src/check/boxes.rs`: L85 `/// How far `r` reaches outside `limit` across `axes` (at most 0 inside).`; L86 `pub(super) fn reach(r: Rect, limit: Rect, axes: Axes) -> f32 {`; L87 `let mut most = 0.0f32;`; L88 `if axes.x {`; L89 `most = most`; L90 `.max(limit.x - r.x)`; L91 `.max(r.x + r.width - (limit.x + limit.width));`; L92 `}`
- 97689c4 `crates/kivact-engine/src/check/boxes.rs`: L85 `/// How far `r` reaches outside `limit` across `axes` (at most 0 inside).`; L86 `pub(super) fn reach(r: Rect, limit: Rect, axes: Axes) -> f64 {`; L87 `let mut most = 0.0f64;`; L88 `if axes.x {`; L89 `most = most`; L90 `.max(limit.x - r.x)`; L91 `.max(r.x + r.width - (limit.x + limit.width));`; L92 `}`

### layout_boundary — native_f32_import

Taffy remains native f32. New boundary explicitly narrows inputs and imports native endpoint calculations via f64::from. Distinguish pure import equality from later f64 arithmetic.

- b6e7ff2 `crates/kivact-layout/src/tree.rs`: L80 `let mut baseline = None;`; L81 `let mut out = taffy::compute_leaf_layout(inputs, style, resolve, |known, avail| {`; L82 `let slot = match entry(t) {`; L83 `Entry::Viewport => return taffy::Size::ZERO,`; L84 `Entry::Slot(slot) => slot,`; L86 `let request = MeasureRequest {`; L87 `known_width: known.width,`; L88 `known_height: known.height,`
- 97689c4 `crates/kivact-layout/src/tree.rs`: L80 `let mut baseline = None;`; L81 `let mut out = taffy::compute_leaf_layout(inputs, style, resolve, |known, avail| {`; L82 `let slot = match entry(t) {`; L83 `Entry::Viewport => return taffy::Size::ZERO,`; L84 `Entry::Slot(slot) => slot,`; L86 `let request = MeasureRequest {`; L87 `known_width: known.width.map(f64::from),`; L88 `known_height: known.height.map(f64::from),`

### text_boundary — native_f32_import_and_widened_math

Parley input remains f32 via explicit conversion at the external library boundary.

- b6e7ff2 `crates/arvane-text/src/build.rs`: L13 `FontContext, FontFamily, FontFamilyName, FontStyle, FontWeight, GenericFamily, LayoutContext,`; L14 `LineHeight, StyleProperty,`; L15 `};`; L41 `let mut builder = contexts.ranged_builder(fonts, text, 1.0, true);`; L42 `builder.push_default(StyleProperty::FontSize(style.font_size));`; L43 `builder.push_default(StyleProperty::FontWeight(FontWeight::new(f32::from(`; L52 `if style.letter_spacing != 0.0 {`; L53 `builder.push_default(StyleProperty::LetterSpacing(style.letter_spacing));`
- 97689c4 `crates/arvane-text/src/build.rs`: L14 `FontContext, FontFamily, FontFamilyName, FontStyle, FontWeight, GenericFamily, LayoutContext,`; L15 `LineHeight, StyleProperty,`; L16 `};`; L42 `let mut builder = contexts.ranged_builder(fonts, text, 1.0, true);`; L43 `builder.push_default(StyleProperty::FontSize(style.font_size as f32));`; L44 `builder.push_default(StyleProperty::FontWeight(FontWeight::new(f32::from(`; L53 `if style.resolve(style.letter_spacing) != 0.0 {`; L54 `builder.push_default(StyleProperty::LetterSpacing(`

### text_output_math — widened_plus_explicit_native_endpoint_policy

New output widens native metrics before custom sums; uncut native inline bounds intentionally calculate left/right in native f32 first, then subtract imported f64 edges. This explicit endpoint policy is distinct from blanket arithmetic widening; source snippets document it.

- b6e7ff2 `crates/arvane-text/src/output.rs`: L86 `id: g.id,`; L87 `x: pen + g.x,`; L88 `y: g.y,`; L89 `advance: g.advance,`; L90 `};`; L151 `/// start.`; L152 `fn content_width(line: &parley::Line<'_, u32>) -> f32 {`; L153 `let m = line.metrics();`
- 97689c4 `crates/arvane-text/src/output.rs`: L45 `Self {`; L46 `ascent: f64::from(m.ascent),`; L47 `descent: f64::from(m.descent),`; L48 `underline_offset: f64::from(m.underline_offset),`; L49 `underline_size: f64::from(m.underline_size),`; L50 `strikethrough_offset: f64::from(m.strikethrough_offset),`; L51 `strikethrough_size: f64::from(m.strikethrough_size),`; L52 `}`

### geometry_reports — same_exact_equality_policy

sent_scroll and sent_layout compare exact tuples in both versions. Widening stored geometry may expose sub-f32 changes and add a report. Prove same node, successive state, changed component and producer calculation before accepting an extra report.

- b6e7ff2 `crates/kivact-engine/src/model/interact/geometry.rs`: L559 `};`; L560 `let state = (n.scroll, l.size, l.content_size);`; L561 `if n.sent_scroll != Some(state) {`; L562 `n.sent_scroll = Some(state);`; L563 `self.push(HostEvent::ScrollChanged {`; L564 `node: id,`; L579 `};`; L580 `let size = l.size;`
- 97689c4 `crates/kivact-engine/src/model/interact/geometry.rs`: L560 `};`; L561 `let state = (n.scroll, l.size, l.content_size);`; L562 `if n.sent_scroll != Some(state) {`; L563 `n.sent_scroll = Some(state);`; L564 `self.push(HostEvent::ScrollChanged {`; L565 `node: id,`; L580 `};`; L581 `let size = l.size;`

### soft_unbounded_range — widened_domain_not_rounding

Unbounded sentinel f32::MAX→f64::MAX changes numeric domain, not decimal formatting. The old layout_components value_max trace example needs source/node linkage to this bound before any classification; similarity of magnitudes alone is insufficient.

- b6e7ff2 `crates/kivact-ui-soft/src/resizable.rs`: L30 `let min = floor.unwrap_or(0.0);`; L31 `let max = ceil.unwrap_or(f32::MAX).max(min);`; L32 `(basis * value / 100.0).clamp(min, max)`; L40 `pub min_size: ResizeSize,`; L41 `pub max_size: ResizeSize,`; L42 `pub container: Option<NodeRef>,`; L55 `min_size: ResizeSize::Px(50.0),`; L56 `max_size: ResizeSize::Px(f32::MAX),`
- 97689c4 `crates/kivact-ui-soft/src/resizable.rs`: L30 `let min = floor.unwrap_or(0.0);`; L31 `let max = ceil.unwrap_or(f64::MAX).max(min);`; L32 `(basis * value / 100.0).clamp(min, max)`; L40 `pub min_size: ResizeSize,`; L41 `pub max_size: ResizeSize,`; L42 `pub container: Option<NodeRef>,`; L55 `min_size: ResizeSize::Px(50.0),`; L56 `max_size: ResizeSize::Px(f64::MAX),`

## Strict review method

1. Pin trace/input hashes and both source revisions. Re-run ordered comparison only after KeyUp pump and easing endpoint fixes; retain original artifacts as evidence, never silently replace them.
2. Align complete ordered semantic events first. Anchor node identity by stable node id within same tree revision plus role/name/test-id, creation site and ancestor topology; if ids change, require a unique structural identity map. Match event kind, phase, input ordinal, pointer id/button, region, rule, revision and cause. Never pair events solely by shape or numeric proximity.
3. Check complete key/text/click sequences, timer time, turn/stage boundaries, revision and event counts as exact contracts independently. Old matcher demonstrated178 false click changes; OTP demonstrated real KeyUp batching regression. A numeric tolerance cannot resolve either.
4. Assign each numeric leaf to its declaration and dataflow producer using mappings below. Finite Option wrappers and renamed records do not imply widening; semantic numeric fields were already f64.
5. Recover old f32 operands by parsing Rust shortest decimal strings back to binary32. Recover new f64 operands as binary64. Replay the exact source expression operation-by-operation, rounding at every old f32 operation and at documented Taffy/Parley boundaries. Comparing only f32(final_new) to old final is insufficient.
6. For unchanged-f64 sinks, first demand exact same inputs and computation. If an upstream geometry/color/drag expression intentionally widened, link that changed expression and replay both precision paths. Otherwise leave any finite nonzero value delta unresolved. Finite only justifies nonfinite filtering or signed-zero normalization, not arbitrary rounding.
7. For geometry imports, verify identical native Taffy/Parley f32 endpoints and glyph/font ids. Classify pure f32→f64 import separately from subsequent transform, subtraction, scroll or clipping. Current native_inline_bounds explicitly retains native f32 edge evaluation for uncut lines; custom ellipsis math is f64.
8. For a candidate extra LayoutChanged, compare consecutive per-node exact sent_layout tuples on both sides. Prove the old f32 tuples are equal while new f64 tuples differ and identify the changed leaf calculation. Then trace downstream writes/reruns/focus/scroll; additional behavior or an identity-transform endpoint error remains a regression, not precision acceptance.
9. Classify with evidence: exact equality; intentional declaration/calculation widening reproduced; explicit native-boundary policy reproduced; domain/sentinel change requiring separate approval; real behavioral change; unresolved alignment/provenance. Projection equality, small ULP distance, or blanket epsilon alone never accepts a difference.

JSON contains complete matched source snippets (exact git paths/line numbers/blob hashes), Finite implementation evidence and every original numeric trace path. Markdown excerpts are abbreviated; use JSON for full declarations and math.
