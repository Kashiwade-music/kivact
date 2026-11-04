# CP1 trace comparison adapter

This is diagnostic tooling, not a golden test and not a CP1 result. It reads raw
logs without rewriting, deleting or sorting their records. Generated comparison
artifacts live under `target/foundation-v3/cp1/` and are not committed. This
saved tool copy is committed for resuming the paused review. See
`../TRACE-RESUME.md` for the actual final comparison status and source commit.

## Run after CP1 raw traces are complete

```powershell
python refactoring/checkpoint-cp1/tools/compare_traces.py --baseline target/foundation-v3/traces/cp0 --current target/foundation-v3/traces/cp1-after-boundary --output target/foundation-v3/cp1/comparison-resume-new
python refactoring/checkpoint-cp1/tools/summarize_traces.py target/foundation-v3/cp1/comparison-resume-new
```

Use the actual complete CP1 raw directory in `--current`. An incomplete run
reports every missing file; it cannot pass. Exit 0 means exact equality of the
explicit common projection, allowing only documented schema additions/renames
and fully empty poll removal. Any numeric difference, source location drift,
record insertion/removal, unsupported schema or parse error returns 1 and
`REVIEW_REQUIRED`. The completed final comparison remains REVIEW_REQUIRED;
the review is paused with unresolved numeric and execution-order evidence.

`summary.json` contains exact category counts, per-file counts, raw SHA256,
raw/kept line counts and each removed poll's raw line span and original turn.
`details.jsonl` contains each difference's raw line numbers, structural path,
old/new values, every schema addition's actual value, and exact sequence spans.
The summary printer lists the first unresolved location per file.

## Parsing and ordering

A recursive parser reads Rust Debug structs, enum constructors, tuples, lists,
numbers, strings/chars, booleans and bounded ranges. The CP0 audit parsed all
915 files / 651,969 lines with zero parse or adapter errors. Unknown syntax
fails the whole file closed. Unknown fields/tags are preserved and compared,
except the explicit schema adapters below; a new Semantics or Routed field
fails closed. Named struct-field declaration order is not a semantic ordering;
fields are matched by name. Lists, causes, writes, reruns, notes, semantic
updates, host events and stages retain their recorded order.

Alignment first takes the identical discriminator prefix/suffix. A changed
middle of at most 4,000 records uses ordered sequence alignment. A larger
ambiguous middle produces `record_sequence_alignment_unresolved`, with exact
raw line lists/counts and the first five payloads. It is never considered a
match. Raw files retain the complete payloads for those reported spans.

## Explicit schema projections

1. **Semantics:** `Some(NodeSemantics)` is flattened with its optional
   `accessibility: Some(SemanticsProperties)`. `value` becomes `value_text`.
   An absent advanced property box means its former declared defaults (None,
   empty relations, SortDirection::None, Orientation::Unspecified). An entirely
   absent node semantics value means those same implicit defaults. Current
   `Some(Semantics)` projects onto precisely those old fields. New fields
   `disabled`, `hidden`, `focusable`, `protected` are individually reported as
   schema additions, with their actual values. `removed` is preserved.
   **No semantic record is dropped**, including a new record caused only by a
   newly logged field: those produce sequence differences requiring review.
2. **Finite:** the `Finite(number)` newtype wrapper is removed; the numeric
   value itself is compared. Signed-zero differences remain visible.
3. **Text IDs:** only `SemanticsTextRun.id: TextRunId(n)` projects to the old
   `NodeId(n)` constructor. Its number and run list ordering are preserved.
4. **Environment:** `Environment.scale_factor/window_size` project to
   `HostEnv.scale/window`; all other old environment fields are compared.
   `HostInfo.surface_active` is moved into its environment projection.
   `EnvChanged { environment, surface_active }` projects to the old tuple
   `EnvChanged(HostEnv { ..., surface_active })`. Surface availability is
   **always compared**, not ignored. New `keyboard_inset`, `pointer` and
   `fonts` are individually reported as schema additions with actual values.
5. **Routed pointer:** `target: Some(node)` and `region` map to the old host
   fields. `local` maps to PointerEvent.pos, sample.position to window_pos;
   all old pointer fields are retained. New captured/activation/time_ns/button/
   press are reported. `button` and `press` must agree with the sample's
   Down/Up phase; disagreement is unresolved. Activation must remain None
   during W1; Some is unresolved.
6. **Routed key:** target (including None), region and all KeyInput fields
   map to the old key event. Local/captured/activation are additions.
7. **Routed wheel:** target/region/local map to old node/region/pos, and
   delta/modifiers are preserved. New window position and routing metadata
   are reported. A missing pointer/wheel/file target is an adapter error.
8. **Routed file:** target/local/input.position map to old node/pos/window_pos;
   phase/files retain their order. Region and routing metadata are reported.
9. **Failure:** `Failure { kind, detail }` projects to its old diagnostic
   string; detail is compared exactly and kind is reported as a schema addition.
   No arbitrary error messages are normalized or discarded.

## Fully empty hardware poll rule

Only a complete numbered Begin-to-next-Begin block can be removed. It must
have one summary and one output, events/runs/ops zero, causes/writes/reruns/
notes empty, no receive/semantics/start or unknown records, all commit writes
zero, and `TurnOutput { next_wake_ns: None, exit: false }`. Additionally each
commit revision must equal the previously observed revision. Unknown revision
history prevents removal. Removed file names, raw spans and original turn
numbers are recorded. Only the numbered TurnRecord counter is offset by the
number of removed preceding polls; revision, time, counts and payloads are not
renumbered. The final baseline self-comparison identified 729 such polls.

## Numeric and other unresolved differences

Numbers retain their raw decimal spelling. Equal decimal values with different
spelling are reported as `numeric_spelling`; signed zero gets its own category.
Different floating values are classified as `numeric_same_f32_projection` only
when converting both to IEEE-754 binary32 yields identical bits; otherwise
`numeric_value_changed`. Integer changes are separate. This is a classification,
not acceptance: **all numeric differences require review**, including same-f32
projection. The heuristic does not assert a particular field was formerly f32;
old f64 semantic bounds also remain visible. No epsilon or rounding is used to
turn differences into a pass.

Source Location file/line/column drift is classified separately and still fails
the comparison. Source paths, diagnostic type strings, notes, ops, revisions,
write/rerun counts and stages are never discarded or generally renamed. These
may have intended W1 differences, which need explicit review. Schema addition
reports show newly observable data, not proof that their behavior is correct.
The trace itself samples semantics only at existing phase boundaries; this tool
cannot prove absence of unrecorded intermediate changes or pixel differences.

## Verification performed

- Complete CP0 parser audit: 915 files, zero parser/adapter errors.
- Complete CP0-versus-itself comparison: 915/915, zero differences.
- Synthetic adapter checks: HostInfo and EnvChanged surface split, pointer
  Routed fields, semantics flatten and implicit defaults, numeric categories,
  signed zero, and bounded alignment failing closed.
- Results: `baseline-audit/`, `self-comparison/`, `tooling-validation.json`.
- The baseline self-comparison is tooling validation only; it is not CP1 evidence.

## Follow-up: one validated artifact-name rename

The comparator recognizes exactly these six pairs:
`test__screen__window_tests__resize_dimensions_must_fit_finite_positive_f32`
versus the same explicitly named test ending in `f64`, with only the suffixes
`.txt`, `-2.txt`, `-3.txt`, `-4.txt`, `-5.txt`, `-6.txt`.
This is not a general `f32` -> `f64` filename substitution.

Before applying any pair it reads
`git show b6e7ff2:crates/kivact-core/src/test/screen/window_tests.rs` and the
current source, requiring exactly one old function and one new function in the
respective versions. Applied pairs go into `filename-alias-ledger.json` with
both raw names, both raw SHA256 hashes, source hashes, commit, functions and
line numbers. `validated-filename-aliases.json` is the preparation manifest:
its current raw hashes remain null until those traces exist. If both names
exist in the current directory, the comparison reports a conflict rather than
choosing one. All other missing/new filenames still require review. CP0
self-comparison applies no aliases and must remain 915/915 with zero differences.

## Optional source-location evidence (diagnostic only)

After running the actual comparison:

```powershell
python target/foundation-v3/cp1/classify_source_locations.py target/foundation-v3/cp1/comparison --repo . --baseline-commit b6e7ff2 --output target/foundation-v3/cp1/source-location-evidence
```

The script reads the original raw trace records referenced by each
`source_location_changed` difference. It reconstructs full old/new `Location`
(file/line/column) pairs, deduplicates repeated file/line/column leaf differences
for one occurrence, and aggregates each unique pair's exact occurrence count.
`pairs.jsonl` includes up to ten concrete trace examples per pair, both source
SHA256 hashes, exact target-line text, and alignment evidence. `summary.json`
contains pair and occurrence counts by classification.

Baseline source comes only from `git show b6e7ff2:<exact old path>`; current
source comes from the exact new path in the trace. Path separator normalization
is only for opening those paths. Absolute paths outside the explicit repo and
parent traversal are unresolved; no suffix/path-rename guessing is performed.
The current source must correspond to the binary that generated the CP1 trace;
its captured SHA256 makes the evidence reproducible but does not itself prove
that build association.

A pair is classified `unique_unchanged_source_context` only if at least three
contiguous source lines including the target are byte-for-byte equal as Unicode
line text (no whitespace normalization), the whole context occurs exactly once
in each file, its target offset maps the claimed old line to the claimed new
line, and the columns are equal. Context radii of 1/2/3/5/8/12 are tried; more
context may disambiguate repeated short lines. Different columns remain
`unchanged_context_column_changed`. Changed target text, repeated/insufficient
unchanged contexts, missing source, parse failure or unsupported paths remain
unresolved and are counted. CRLF/LF terminators are excluded from line matching;
raw source hashes preserve that difference.

**This classification never accepts or removes a comparison difference.** The
original comparison status, all payload/order/ops differences, and every actual
numeric difference remain unchanged. It only supplies exact evidence for a
human review of intended source-location shifts. Unpaired sequence differences
are not guessed into source-location pairs. Validation examples are stored in
`source-classifier-validation.json`; the no-difference self-report is in
`source-classifier-selfcheck/`.


## Ordered alignment and archived source association

`comparison-ordered` uses recursive unique exact-payload anchors, then unique stable event identity anchors, with an ordered longest increasing subsequence. Anchors are never used as payload equality: paired records still receive strict recursive comparison. Exact full-record prefixes/suffixes alone are trimmed. Bounded ambiguous spans use ordered SequenceMatcher; oversized unanchored spans remain unresolved. Lists inside payloads retain their original order. No sorted bags or event drops are used.

The archived first CP1 source association is `e95724c`. Use `--current-commit e95724c` with the source classifier; otherwise its default reads working-tree source and cannot establish the old binary association. `source-location-evidence-old-cp1` classifies 117 location pairs/18264 occurrences: 90 pairs/18168 occurrences have unique unchanged context; 3/35 have unchanged target but unresolved context; 24/61 have changed source text. All remain comparison differences.

Current trace source formats `node.map(|n| &n.semantics)`, versus baseline `node.and_then(|n| n.semantics.as_deref())`. Current `None` means node absent, `Some(None)` present with no semantics, `Some(Some(...))` present with semantics. The adapter records current presence as a schema addition and projects only the common semantic payload. Baseline `None` cannot distinguish the first two states; it is marked not observable rather than asserted equal. Raw current presence is retained in the ledger.

`layout-event-evidence.json` retains extra n13 layout event raw lines and hashes, same-f32-projection axis evidence, and an exact numeric endpoint reproduction: 30-step bezier bisection at zero yields scale0.9500000000698492 and width75.05000000551809 for width79. This is causal diagnostic evidence, not automatic acceptance. `key-alignment-evidence.json` proves the six-key subsequence in the sidebar example is equal despite the old coarse alignment's apparent field removal; it does not accept the entire file.
