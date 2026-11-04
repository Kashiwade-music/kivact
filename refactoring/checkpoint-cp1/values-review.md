# Final value and diagnostic review

Baseline `b6e7ff2`; final archive source `97689c4`. This report uses only comparison-final occurrences.

Coverage: 27,291 / 27,291 rows assigned once; 9,473 type names + 17,607 cause labels + 211 values. No field_removed category exists.

## Decisions

- canonical_type_path_or_f64: 9467
- debug_label_only_exact_identity: 17607
- diagnostic_location_line_only: 147
- numeric_string_pending_p1: 29
- surface_active_independent_fact_split: 6
- rerun_sequence_pending_p2: 35

All 17,607 cause labels preserve exact key/phase/node/region after the three explicit Debug spelling substitutions recorded by source app_client::describe. 9,467 type changes are exact canonical path or f64 substitutions; six HostEnv→bool writes are the independent surface_active fact introduced by V3, not a type rename.

147 layout note changes preserve every character except source line/column, including clipped amount and text. date_touch text caller moves 488→489 after accessor formatting; layout_check callers move +5 after exhaustive LayoutRule pattern.

35 rerun-kind values (one split test) remain sequence-review pending; 29 numeric strings remain numerical-review pending. Neither set is accepted by this report as equivalent. No new concrete behavioral regression established in the accepted name/location-only groups.
