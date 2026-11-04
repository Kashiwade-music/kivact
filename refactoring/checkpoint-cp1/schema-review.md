# Final schema-addition review

All schema additions accounted for and current-side defaults/context are consistent with inspected producers; no unexplained additions. This is NOT a baseline behavioral-equivalence or CP1 PASS finding.

Coverage: **531,150 records**, **20 exact paths**, **915 files**, all current-side. Every addition was matched to its same raw trace record. No source, comparator, or raw trace edits; no tests run.

Baseline `b6e7ff2`; current trace `97689c4`. Exact values, frequencies, type/tag distributions, per-file counts, example locations and contextual checks are in `review-final-schema.json`.

| Exact path | Count | Distinct exact values | Type/tag counts |
|---|---:|---:|---|
| `$.env.fonts` | 915 | 1 | {"Generation": 915} |
| `$.env.keyboard_inset` | 915 | 1 | {"Insets": 915} |
| `$.env.pointer` | 915 | 1 | {"PointerSettings": 915} |
| `$.environment.fonts` | 38 | 1 | {"Generation": 38} |
| `$.environment.keyboard_inset` | 38 | 1 | {"Insets": 38} |
| `$.environment.pointer` | 38 | 1 | {"PointerSettings": 38} |
| `$.input.button` | 11,989 | 4 | {"None": 5123, "Some": 6866} |
| `$.input.position` | 48 | 7 | {"Point": 48} |
| `$.input.press` | 11,989 | 45 | {"None": 5123, "Some": 6866} |
| `$.input.time_ns` | 11,989 | 85 | {"number": 11989} |
| `$.result.args[0].kind` | 3 | 2 | {"Backend": 2, "Resource": 1} |
| `$.routed.activation` | 17,742 | 1 | {"None": 17742} |
| `$.routed.captured` | 17,742 | 2 | {"bool": 17742} |
| `$.routed.local` | 5,696 | 211 | {"Point": 5696} |
| `$.routed.region` | 9 | 1 | {"None": 9} |
| `$.semantics.disabled` | 64,745 | 1 | {"bool": 64745} |
| `$.semantics.focusable` | 64,745 | 1 | {"bool": 64745} |
| `$.semantics.hidden` | 64,745 | 1 | {"bool": 64745} |
| `$.semantics.presence` | 192,104 | 3 | {"object": 192104} |
| `$.semantics.protected` | 64,745 | 1 | {"bool": 64745} |

## Verified current-side meaning

- Semantics flags: disabled/hidden/focusable/protected are each false in all64745 stored semantic payloads. Default values and raw NodeProperties trace provenance explain this; they are not the derived accessibility tree flags.
- Outer presence: existing node + semantics64745; existing node + no semantics33914; missing engine node93445. All192104 match the raw nested Option envelope.
- Environment: all915 starts and38 EnvironmentChanged observations have Insets(0,0,0,0), Generation(0), and the exact default PointerSettings:500000000ns; distance(4,4); wheel lines/chars3; slop4; primaryFine; fling0.998 and0.05.
- Routed:11989 Pointer,5696 Key,48 Wheel,9 File. Activation None17742; capture true7069/false10673. All button/press pairs match phase, timestamps are u64, all local/window points checked are finite; no-target/no-pointer key local is zero in1064 cases; all9 file regions are None.
- Two Up/captured:false observations are direct synthetic HostEvent injections in host_sync_tests.rs:57-98 (pointer IDs11/22), not violations of the engine send_pointer producer. All3433 other Up/Cancel observations have captured:true. Other phase capture values are recorded exactly; full capture state was not reconstructed.
- Failure.kind: Backend2 (platform open/reveal failures) andResource1 (unsupported embedded font). Each reproduces the raw Failure kind and agrees with the inspected producer categories.

## Baseline-unobservable limits

- Four bool flags were not in old NodeSemantics. Current raw stored props are false by default in all 64745 semantic payloads; this does not prove final native accessibility disabled/hidden/focusable/protected behavior. A11y publication derives those flags separately.
- Old semantics None collapsed engine node absent and semantics absent; exact outer presence (93445 absent nodes /33914 existing nodes without semantics /64745 with semantics) is now observable only in current traces. Cannot infer baseline outer node existence or prove old-equivalent lifetimes from these additions.
- No baseline captured or activation metadata. Current activation None agrees with routing until adoption transfer. Non-pointer captured false and pointer Up/Cancel true are source-derived consistency checks; exact capture selection for other phases cannot be reconstructed from added fields alone.
- Pointer time_ns was absent from baseline host listener payload. All11989 are unsigned u64, with85 exact values (11178 zero). Source copies normalized samples; these notes alone cannot prove source-clock identity or monotonicity across input-source resets.
- Key local5696 and wheel window position48 are new; all finite, and1064 key cases lacking pointer/target are zero. Exact transformed local coordinates and wheel source provenance cannot be independently recomputed from these metadata notes alone.
- Environment defaults (953 observations per added field) are consistent, but baseline did not expose keyboard insets, font generation, or the bundled pointer preferences. Non-default keyboard/font/primary-device behavior is unexercised here.
- Failure.kind is new. Backend2/Resource1 matches scoped producers and diagnostic details; previous string-only failures cannot prove categorical equivalence. Other failure categories are not exercised.

## Source evidence

- semantics: `crates/kivact-types/src/semantics/node.rs:35-43,97-101 (bool declarations and false defaults)`; `crates/kivact-engine/src/host/write/typed.rs:43 (get_or_insert_default)`; `crates/kivact-engine/src/tree/node.rs:121 (stored NodeProperties semantics)`; `crates/kivact-core/src/client/trace.rs:111 (node.map(|n| &n.semantics))`; `crates/kivact-core/src/view/tree.rs:486 and crates/kivact-engine/src/host/write/reader.rs:156 (engine node existence)`; `crates/kivact-engine/src/a11y/build/node.rs:125,143,223 (published accessibility state is derived separately)`
- environment: `crates/kivact-types/src/env.rs:59-75 (Environment defaults)`; `crates/kivact-types/src/env.rs:101-113 (PointerSettings defaults)`; `crates/kivact-types/src/cell.rs:98 (Generation default zero)`; `crates/arvane/src/test_display/virtual_window.rs:140-153 (explicit test options plus Environment default)`; `crates/arvane/src/test_display/shell.rs:207 (clone shared environment)`
- routing: `crates/kivact-engine/src/contract/routed.rs:7-29 (metadata semantics and adoption deferred)`; `crates/kivact-engine/src/model/pointer/machine.rs:33-56 (capture, phase-derived button/press, source timestamp)`; `crates/kivact-engine/src/model/pointer/shell.rs:57,273,324 (timestamp copied from sample)`; `crates/kivact-engine/src/model/keys.rs:162-170,421-429 (key local and no capture/activation)`; `crates/kivact-engine/src/model/pointer/machine.rs:550-561 (wheel window/local positions, no capture)`; `crates/kivact-engine/src/model/pointer/shell.rs:372-413 (file region None, no capture)`; `crates/kivact-core/src/test/screen/host_sync_tests.rs:57-98 (direct injected synthetic pointer metadata)`
- failure: `crates/arvane-shell/src/types.rs:15-21 (ShellError::new => Backend)`; `crates/arvane/src/error.rs:5-18 (resource font failure classification)`; `crates/kivact-types/src/platform.rs:66-94 (kind/detail contract)`

## Boundary

This ledger completes the assigned schema-addition category only. Numeric/value/record-sequence reviews and overall CP1 gates remain separate. Schema records are not silently accepted as baseline-equivalent. No new work started after the pause request.
