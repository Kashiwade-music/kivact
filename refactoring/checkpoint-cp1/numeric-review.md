# Final CP1 numeric review — complete coverage, NOT PASS

Immutable comparison cp0/b6e7ff2 → cp1-after-boundary/97689c4. No source/tool changes or checks. This is the pause/resume boundary requested by root.

## Coverage

- All 8326 assigned records indexed:3024 same-f32-projection +4648 numeric-value +609 integer +45 sequence-length.
- 44 exact category/path/status groups; every record has paired trace line numbers, raw hashes, kind/node/role/name/frame context and source provenance references.
- Numeric display strings from peer review: 29 additional records, kept separate from assigned totals.
- No record is accepted solely by epsilon, ULP distance or final f32 projection.

## Findings

-24 of35 numerically large deltas compare DIFFERENT preceding FrameTick epochs (8 coast values plus16 touch-thumb rectangle fields). Another10 touch-thumb rectangle deltas share an epoch and still require intra-frame state occurrence alignment; they are NOT proven alignment artifacts. Baseline coast line823 value99.64193258730873 follows400000007ns; current paired819 value99.54805950495728 follows383333340ns. Current836 follows400000007ns and gives99.64193257876418. These are alignment defects, not evidence of a0.094 numeric regression or permission to ignore it.
- Remaining large delta is Splitter n17 / Resize handle value_max f32MAX→f64MAX. Shared semantic sink was already f64; source soft/resizable sentinel widened. Treat as domain change requiring explicit acceptance.
- All semantic numeric sinks were alreadyf64. JSON contains exact output-consistency checks for simple Color component storage / split fraction percentages where applicable, explicitly marked as inferred-input algebra, NOT full source-input replay.
- Integer609 records: revision494, runs44, turn36, writes19, ops13, events3. Sequence45: reruns42, notes3. These require a causal member/counter ledger from sequence review, not numeric tolerance.

## Classification counts

- source_mapped_precision_candidate_requires_operand_replay: 7441
- unresolved_exact_counter_delta_requires_sequence_causal_ledger: 609
- unchanged_f64_sink_requires_upstream_forward_replay: 200
- unresolved_rerun_or_note_sequence_delta_requires_exact_member_ledger: 45
- unresolved_node_or_event_identity: 6
- domain_sentinel_change_requires_explicit_acceptance: 1
- unresolved_frame_epoch_alignment: 24

## Resume packet

1. Correct semantic/LayoutChanged matching using preceding FrameTick now_ns context;24 of35 large deltas are paired against a different frame epoch. Keep same epoch candidates from far35 evidence, but require unique per-node state/caller occurrence before re-pair.
2. Re-run comparator and all four category coverage against corrected epoch-aware identity, retaining immutable inputs and prior outputs.
3. Replay unchanged-f64 semantic sink values from source input operands (scalar geometry/release velocity/color components/split shares), not output-inferred algebra. Records include exact output-consistency checks only and label them explicitly as non-proof.
4. Review f32MAX→f64MAX Resize handle n17 range as deliberate domain/sentinel change requiring explicit decision, separate from precision widening.
5. For remaining geometry/warnings, reproduce old f32 per-operation and new f64 computations plus native Taffy/Parley boundary policy. Source declaration mapping alone is insufficient numeric proof.
6. Have p2 exact-account integer609 counter deltas and sequence45 reruns/notes to inserted/deleted/changed reports/caller order; do not normalize them as timing noise.
7. Join29 numeric value_text strings from review_p0 to corresponding same-node/same-epoch sink records; display precision can be a real public change even when geometry precision is intentional.

## Artifact contents

`review-final-numeric.json` contains complete per-record coverage, exact-path groups, semantic sink algebra checks,35 full raw paired large-delta contexts with same-epoch current candidate records, and peer display-string records. `numeric-provenance.json` supplies pinned old/new declaration and calculation locations.

CP1 numeric review NOT PASS: complete record coverage, but exact forward replay and corrected frame alignment remain unresolved. No blanket epsilon/projection acceptance. No numeric counter or sequence difference accepted.
