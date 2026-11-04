# Final source location review

Baseline b6e7ff2 -> final 97689c4. Diagnostic source identity only; comparison differences are not accepted, normalized away, or dropped.

Coverage: 23/23 required nontrivial pairs, 91 occurrences; 22 exact old/new-location + both full-source-SHA reviews reused, one independently reviewed final pair (#96). 91 unique unchanged-context pairs excluded from manual review; all of their columns match.

Actual lost Location::caller sites found: 0. Misaligned rerun pairs remain execution REVIEW_REQUIRED: 16 pairs / 37 occurrences. Their different source operations are not equivalent line movements.

| Pair line | Occurrences | Baseline -> final | Finding |
|---|---:|---|---|
| 16 | 32 | crates\kivact-core\src\view\state.rs:115:18 -> crates\kivact-core\src\view\state.rs:122:18 | retained_caller_ambiguous_context_resolved |
| 26 | 9 | crates\kivact-ui\src\split.rs:283:15 -> crates\kivact-ui\src\split.rs:280:15 | intentional_mechanical_expression_rewrite_same_caller |
| 36 | 2 | crates\kivact-ui-creative\src\editors\time\input.rs:248:18 -> crates\kivact-ui-creative\src\editors\time\input.rs:254:18 | retained_caller_ambiguous_context_resolved |
| 45 | 3 | crates\kivact-interaction\src\zoom_pan.rs:394:33 -> crates\kivact-interaction\src\zoom_pan.rs:387:33 | intentional_mechanical_expression_rewrite_same_caller |
| 48 | 1 | crates\kivact-interaction\src\zoom_pan.rs:413:29 -> crates\kivact-interaction\src\zoom_pan.rs:406:29 | intentional_mechanical_expression_rewrite_same_caller |
| 50 | 6 | crates\kivact-core\src\view\state.rs:115:18 -> crates\kivact-core\src\client\dispatch\env.rs:20:30 | intentional_surface_active_fact_split |
| 53 | 5 | crates\kivact-ui\src\split.rs:336:9 -> crates\kivact-ui\src\split.rs:272:20 | noncorresponding_rerun_sites_both_callers_retained |
| 54 | 6 | crates\kivact-ui\src\split.rs:275:20 -> crates\kivact-ui\src\split.rs:333:9 | noncorresponding_rerun_sites_both_callers_retained |
| 56 | 5 | crates\kivact-ui\src\split.rs:317:25 -> crates\kivact-ui\src\split.rs:223:17 | noncorresponding_rerun_sites_both_callers_retained |
| 57 | 6 | crates\kivact-ui\src\split.rs:336:9 -> crates\kivact-ui\src\split.rs:314:25 | noncorresponding_rerun_sites_both_callers_retained |
| 59 | 1 | crates\kivact-ui\src\split.rs:328:17 -> crates\kivact-ui\src\split.rs:333:9 | noncorresponding_rerun_sites_both_callers_retained |
| 60 | 1 | crates\kivact-ui\src\split.rs:226:17 -> crates\kivact-ui\src\split.rs:333:9 | noncorresponding_rerun_sites_both_callers_retained |
| 61 | 2 | crates\kivact-ui\src\split.rs:317:25 -> crates\kivact-ui\src\split.rs:325:17 | noncorresponding_rerun_sites_both_callers_retained |
| 62 | 1 | crates\kivact-ui\src\split.rs:336:9 -> crates\kivact-ui\src\split.rs:193:26 | noncorresponding_rerun_sites_both_callers_retained |
| 63 | 1 | crates\kivact-ui\src\split.rs:336:9 -> crates\kivact-ui\src\split.rs:223:17 | noncorresponding_rerun_sites_both_callers_retained |
| 64 | 1 | crates\kivact-ui\src\split.rs:328:17 -> crates\kivact-ui\src\split.rs:282:17 | noncorresponding_rerun_sites_both_callers_retained |
| 65 | 1 | crates\kivact-ui\src\split.rs:226:17 -> crates\kivact-ui\src\split.rs:272:20 | noncorresponding_rerun_sites_both_callers_retained |
| 66 | 1 | crates\kivact-ui\src\split.rs:193:26 -> crates\kivact-ui\src\split.rs:333:9 | noncorresponding_rerun_sites_both_callers_retained |
| 67 | 2 | crates\kivact-ui\src\split.rs:226:17 -> crates\kivact-ui\src\split.rs:314:25 | noncorresponding_rerun_sites_both_callers_retained |
| 68 | 1 | crates\kivact-ui\src\split.rs:285:17 -> crates\kivact-ui\src\split.rs:314:25 | noncorresponding_rerun_sites_both_callers_retained |
| 69 | 1 | crates\kivact-ui\src\split.rs:328:17 -> crates\kivact-ui\src\split.rs:314:25 | noncorresponding_rerun_sites_both_callers_retained |
| 70 | 2 | crates\kivact-ui\src\split.rs:317:25 -> crates\kivact-ui\src\split.rs:333:9 | noncorresponding_rerun_sites_both_callers_retained |
| 96 | 1 | crates\kivact-core\src\test\screen\host_sync_tests.rs:125:17 -> crates\kivact-core\src\test\screen\host_sync_tests.rs:141:17 | retained_caller_ambiguous_context_resolved |

## Pair #96 independently reviewed

Same gesture canvas part constructor at125:17 ->141:17, same enclosing named test and .part().region_events().on_pointer chain. The other repeated part constructor creates the action region and is not substituted. Earlier Routed sample construction/import edits account for the final shift; phase() accessor changed contiguous context. Raw turn Number(3) writes[0].created_at retains this constructor caller. Old125->135 evidence had a different final file SHA and was not reused.

## Remaining execution distinction

The16 cross-site split rerun pairs retain both callers at independently recorded counterpart sites, but their positional rerun entries refer to different operations (shares/position/range memo, pane node, drag behavior handle, resize node). They remain an ordering/sequence question for execution review. This artifact makes no decision that such reruns are behaviorally equivalent.

## Evidence integrity

pairs SHA256: e38f9bc7797837dbcf8f071679117b6629148b4f41fe81269b3bbfb4aee35fa6
prior review SHA256: 3c7e8c8cbf0f68e645b6f721e311b0f63c63cdd62325c106e2aa07d5da6df9c6
Full source blobs were read at the exact two commits and matched the classifier SHA256 and line text for every required pair. Reused source contexts/counterparts were revalidated byte-for-byte before their commit attribution was updated. No HEAD-source substitution, filename guess, whitespace normalization or changed raw trace is involved.
