# Tooling and verification audit (prefix TL)

Method: read and grep only (no cargo run). All time numbers are estimates derived from counts, marked "est.".

## 0. Summary

1. **The full gate is a 9-command serial pipeline (about 8 compile configurations) with no fast tier.** An implementer has only "run the gate" or "run nothing". The `kivact-docs` doctests (748 Rust blocks, 611 of them `no_run`, en and jp compiled separately) and the two feature configurations (`--all-features`, `--no-default-features`) dominate. The whole thing sits behind one global `window-lock`, so parallel worktrees and agents queue behind a 90-minute lock even when the run opens no window.
2. **1182 `xtask-allow(state-fields)` markers (about 60% of all xtask-allow lines) are mostly noise.** 644 are `option:` and 342 are `independent:`. They document passive records (builders, Options, Recipes), not state machines. About 2.2k lines of xtask (claims, classify, check, locals, tests) maintain the rule. The rule is right in intent, but the unit is wrong: it flags every struct with 2 or more Option/bool fields, and it needs a marker per field.
3. **`architecture/` (121 SVG, 132,874 lines) is regenerated into 8 of the last 8 architecture-touching commits.** CI does not check it (`--no-write`). The local gate `git diff --quiet -- architecture` only forces the author to commit generated output. It is pure review noise and merge-conflict surface.
4. **The DevTools stack has 5 Rust crates plus a Python package: 15.6k Rust lines plus 3.97k Python lines (2860 of them generated), plus a 105 KB JSON schema and about 1.5k lines of codegen in xtask.** `-client` (617 lines) and `-agent` (1672) are not real boundaries. The Python client duplicates the MCP server's purpose. `kivact-core/src/devtools/` is 5144 lines, plus 1463 lines in `view/devtools_*.rs`, so DevTools is about 6.6k lines inside the core hot crate.
5. **The tests contradict their own rule.** CONTRIBUTING says "no tests that only reproduce past bugs, no pixel or time exact comparison". There are 15+ comments marked `// Regression:` in e2e, plus `*_tests.rs` regression files in `kivact-test`. The 5 kit `consumer` suites (13.8k lines, about 220 tests) overlap `e2e/ui_*`, `theme_conformance` and `apg_*` (20 files).

Target shape: a three-tier gate (a: 1-3 min per unit, b: per wave, c: checkpoints). DevTools becomes 3 crates (`-protocol` including client, `-agent` folded into `kivact-core` behind the feature or kept as a leaf, `kivact-devtools` GUI+MCP binaries) with Python deleted or generated from the schema only on release. state-fields is reduced to a rule on mutable state with no per-field markers for passive records. `architecture/` is untracked. Doctests run for the en tree only.

## 1. Current structure map

| Item | Lines (est.) | Role | Verdict |
|---|---|---|---|
| `crates/kivact-devtools-protocol` | 3320 (+105 KB schema JSON) | wire types, JSON-RPC, schema | real boundary (shared by agent, client, GUI, MCP, python) |
| `crates/kivact-devtools-agent` | 1672 | server inside the target app | thin; only used by core and the test |
| `crates/kivact-devtools-client` | 617 | discovery + connection | **fake boundary**: used by GUI, MCP, agent tests only |
| `crates/kivact-devtools-mcp` | 729 | MCP bridge (tools derived from the schema) | proportionate, keep |
| `crates/kivact-devtools` (GUI app) | 8230 | the DevTools application written in Kivact | large but it is also the dogfooding app |
| `python/kivact-devtools` | 3972 (2860 generated) | Python client | **disproportionate** |
| `dev/xtask` | 8090 | policy plus 11 checks plus codegen | see TL-04..07 |
| `dev/xtask/src/devtools_codegen*` | about 1500 | schema to Python generator | only serves the Python package |
| `crates/kivact-test` | 6485 (of which about 2600 is `*_tests.rs`) | `kivact::test` harness | keep, but see TL-12 |
| `dev/tests/kivact-public-api` | about 38k (152 e2e files, 755 `#[test]`) | public-API e2e, ONE binary `e2e` | keep, but ONE binary means a single slow link and no per-area parallelism |
| `crates/kivact-ui-*/tests` | 13.8k, about 220 tests | kit consumer tests | see TL-13 |
| `dev/tests/docs` | build.rs + 90 lines | one doctest module per `.md` (748 blocks) | see TL-03 |
| `dev/tests/gpu-interop-producer` | 6 tests | test-only GPU engine | keep (platform specific) |
| `dev/bench` | 248 + 194 (run.sh) | KPI probes, manual | keep; run.sh does 9 release builds |
| `dev/tools` | window-lock.sh, check-features.sh, subset-catalog-font.py | | see TL-14 |
| `architecture/` | 121 SVG + index.html, 132,874 lines in git | generated diagrams | **delete from git** (TL-02) |
| `assets/`, `templates/basic`, `vendor/xtask-core` (3.1 MB submodule) | small | | ok |
| `.github/workflows/ci.yml` | 40 | fmt + `xtask check --no-write` on ubuntu only | see TL-15 |

Test layout: 22 top-level integration test binaries, 755 e2e tests in one binary, 212 `#[test]` in `src`, 1123 `#[test]` counting tests dirs. No Windows or macOS CI. Windows is the primary target and the whole real-window gate is local-only.

## 2. Findings

### TL-01 The gate has no fast tier and the lock covers non-window work
- kind: PROCESS
- where: `CONTRIBUTING.md` section "merge 前の検査" (9 commands); `dev/tools/window-lock.sh:1-70`
- evidence: Every test command is wrapped in `KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test ...`, including `--workspace` runs that with `MIRROR=off` use the virtual host only. The lock is a mkdir in the git common dir shared by ALL worktrees, polled every 2 s. Timeout per command is 5400 s. The test configurations are default and `--no-default-features`, 3 commands each (workspace, producer, docs), plus clippy `--workspace --all-targets --all-features`, plus `xtask check` (it compiles the workspace again for its Google-style Clippy pass), plus 24 `cargo check` calls in `check-features.sh`.
- Estimated cost (est., cannot be run here): three full workspace compilations with different feature sets (clippy all-features, test all-features, test no-default-features) plus xtask's own Clippy build, plus about 1500 rustdoc compile+link steps (748 blocks × 2 configs, `--test-threads=1`). Doctests are the largest single item, likely over an hour per config. The 5400 s timeout shows the authors expect it. A single subagent wave that follows the CONTRIBUTING gate blocks every other agent in other worktrees.
- fix: define three tiers (exact commands below, section 3.1). Change `window-lock.sh` semantic: take the lock only when a command opens real windows (`KIVACT_TEST_MIRROR` is `auto`/`required` or the command runs an app / `foreign_window_run` / `gpu-interop-producer`). Add a `--if-mirror` mode: `window-lock.sh --if-mirror -- cmd` skips locking when `KIVACT_TEST_MIRROR=off`. Then tier (a) and (b) run without the lock and in parallel across worktrees. Keep the lock for tier (c).
- breaking: no   kpi: -   size: S
- confidence: high on structure, medium on timings

### TL-02 `architecture/` generated diagrams are committed
- kind: PROCESS / DEAD
- where: `architecture/` (121 SVG, `index.html`), `CONTRIBUTING.md` gate line `git diff --quiet -- architecture`; `.github/workflows/ci.yml` runs `--no-write`
- evidence: 132,874 lines in git. 8 of the last 8 commits that touched it were feature PRs (`0c7455b3`, `b3f6d5d9`, `7aed365a`, `31a5e1d1`, `302175b6`, `cfee0b70`, `31a7ebed`, `96e22b6b`). CI explicitly skips rendering ("diagrams are regenerated locally"), so nothing verifies them except the local `git diff --quiet`. They need Node.js. They are one reason a Haiku-implemented wave produces a 100+ file diff of SVG noise.
- fix: add `/architecture/` to `.gitignore`, `git rm -r architecture`, remove the gate line and the CONTRIBUTING text. Keep xtask's structure check of the diagram model (it is part of `--no-write`), but write the SVG only on `cargo xtask diagrams` (a separate subcommand, optional). If the diagrams must be browsable publicly, publish them as a CI artifact or GitHub Pages from CI, not git.
- breaking: no   kpi: -   size: S
- confidence: high

### TL-03 Doctests: 748 blocks, en and jp compiled twice, 611 are `no_run`
- kind: PROCESS / DUP
- where: `dev/tests/docs/build.rs` (`TREES = ["for-kivact-user", "for-kivact-ui-user"]`, plus README.md and README_JP.md); counts: en 373 blocks, jp 373 blocks, 611 of 748 are `no_run`/`ignore`
- evidence: Each doctest is a separate rustdoc compile and link of a program that depends on the whole facade. CONTRIBUTING requires `--test-threads=1` because of real memory exhaustion (OOM) from simultaneous compile/link. `no_run` blocks only prove "it type-checks", which is the cheapest thing to prove, yet each pays a full link. jp blocks are required to be identical to en blocks (files are paired by name), so compiling both proves nothing new.
- fix: (1) Compile only the `en` trees and `README.md` in `dev/tests/docs`. Add one xtask check `docs-pairs` that fails when the sequence of fenced Rust blocks of a jp file differs from its en twin (string compare, 40 lines). (2) Convert `no_run` blocks of one guide into a single doctest module where possible: group the `no_run` blocks of one `.md` file into one generated `#[test]`-less `fn` body module compiled once by `cargo check -p kivact-docs --all-targets` (type-check only, no link). Only blocks that actually run (137) stay as rustdoc doctests. build.rs already generates modules per file, so it should emit `#[allow(dead_code)] mod guide_x { fn block_1() { ... } }` for `no_run` blocks instead of doc attributes. (3) Run docs only in the no-default-features config once for the check and `--all-features` only for `gpu-texture-sharing.md`.
- Expected effect (est.): from about 1500 rustdoc links to about 140 links plus one cargo check. This is the largest single reduction of the gate.
- breaking: no   kpi: -   size: M
- confidence: medium (block counts measured, timings estimated)

### TL-04 state-fields: wrong unit, 1182 markers, 2.2k lines of checker
- kind: PROCESS / STATE (the rule's own state)
- where: `dev/xtask/src/state_fields/{claims.rs 840, classify.rs 396, check.rs 426, locals.rs 275, audit.rs 76, tests.rs 764}.rs`; policy text `architecture.md` S-1..S-6 and lines 94-96; markers by prefix: `option:` 644, `independent:` 342, `cache:` 112, `identity:` 34, `statistic:` 21, `projection:` 16. Marker density: kivact-ui 38 files, kivact-interaction 30, kivact-engine 29, kivact-ui-soft 28, kivact-core 25.
- evidence (typical markers):
  - `// xtask-allow(state-fields): option: one authored property; absent means the inherited or default value`
  - `// xtask-allow(state-fields): cache: one component of a cache key`
  - `// xtask-allow(state-fields): identity: the next image id, given to each image the store makes`
  - `// xtask-allow(state-fields): option: whether the caller gave Link an activation callback; false paints a static link ...`
  These describe passive configuration records or caches. The rule counts any two `bool`/`Option`/counter fields in one struct. Nothing is protected: the only content is a one-line prose sentence, which `independent:` claims are even checked for being non-duplicate across files, i.e. the tool measures that the author wrote different sentences, not that the combination is valid. Its own docs admit that "meaning errors" (claiming independent when exclusive) are not found, so it relies on a human reading `state-audit.md` anyway. The real implicit state machines the owner wants eliminated (e.g. the 170 findings of fix-a-lot) were found by review, not by markers.
- fix (decided): replace the per-field marker scheme with a struct-level rule on **mutable state only**:
  1. Rule S: a struct or enum variant that (a) has at least 2 fields of kind `bool | Option<_> | integer counter | Cell/RefCell/Signal/Var/Atomic*` AND (b) has at least one method taking `&mut self`, or interior-mutable fields (`Cell`, `RefCell`, `Signal`, `Atomic*`, `Mutex`) written outside the constructor, AND (c) is not an Api/Options/Recipe/Props/Report/Builder (name suffix list already exists in classify.rs) is an error unless it carries ONE struct-level marker `// xtask-allow(state): <one line: how combinations are closed, or "independent flags: a, b">`.
  2. Fields written only in `new`/`Default`/by-value builder methods (`fn x(mut self, ..) -> Self`) are not state: no marker, no check. This alone removes the 644 `option:` and most `cache:`/`identity:`/`statistic:` markers (builders, Options, Recipes, passive records).
  3. Keep the local-variable check (`locals.rs`, signal(false)+signal(None) pairs) since that is where the hidden state machines were found.
  4. Delete `claims.rs` `independent:`-uniqueness check and the `WRITE_METHODS/READ_METHODS` tables unless needed by (2). Delete `--report-audit`/`state-audit.md` (an uncommitted generated file kept for human reading).
  Target: about 150 struct-level markers, about 1.2k lines of xtask. If the owner prefers to keep the old rule, the cheaper alternative is to drop the per-field marker and accept a crate-level `[state_fields.passive]` list in `policy/workspace.toml` (types by path), but the struct-level marker is better because it sits next to the code.
- breaking: no (tooling)   kpi: -   size: L (mechanical removal of about 1000 marker lines, one xtask rewrite)
- confidence: high on counts, medium on how many markers disappear (est. 80% after rule 2)

### TL-05 `xtask-allow(internal-contract)` (61) and `(api-shape)` (41)
- kind: PROCESS
- where: `dev/xtask/src/check_source.rs:125`; samples: `// xtask-allow(internal-contract): Macro expansion support only; users consume IntoValue, not its representation.`, `// xtask-allow(api-shape): Diagnostic source positions retain Rust caller locations.` (repeated 3+ times with identical text)
- evidence: identical reasons repeated. `internal-contract` flags `#[doc(hidden)]` items exposing hidden types. Macro support items are legitimately hidden.
- fix: `internal-contract`: exempt any item inside a module named `__private`/`__macro` and any `#[doc(hidden)]` item whose crate is listed in policy `[macro_support]`; remove those markers. `api-shape`: the `Location`-based exemption (`#[track_caller]`/`&'static Location`) belongs in the rule itself (types `Location`, `SourcePosition` allowed), not in 41 markers. Keep the other rule outcomes.
- breaking: no   kpi: -   size: S
- confidence: medium

### TL-06 xtask rule inventory: keep / change / delete
- kind: PROCESS
- where: `dev/xtask/src/*.rs`
- decision (value per rule):

| Rule | Lines | Decision | Reason |
|---|---|---|---|
| code-line limit 1000/file | policy | keep | cheap, no markers, protects the Haiku wave from huge files |
| `visible_to` (crate dependency direction) | policy 553 lines | keep, shrink | it IS the boundary check. Lists have grown by test crates (`"kivact-test"` in 6 lists), so move "dev-dependency of `kivact-test`/`kivact-public-api-tests`" to a global default: dev-deps never violate `visible_to`. |
| `module_rules` | 165 | keep | one reason: direction inside core; `except` list must be reviewed with the core redesign |
| Google style (20 rules, runs rustfmt and Clippy) | vendor | keep, but run Clippy ONCE | xtask check re-runs Clippy on the workspace and the gate also runs `cargo clippy --workspace --all-targets --all-features -D warnings`. Either style-clippy passes the same flag set (then drop the extra gate line) or xtask's step is skipped via `--no-clippy` in tier (b). |
| `state-fields` | 2.2k | change (TL-04) | |
| `api-shape` (406 + public_surface 539 + public_type_edges 186 + canonical_path 61 + name_collision 125 + structural_options 116 = 1.4k lines, 17 API rules in architecture.md) | | keep `canonical_path`, `name_collision`, `api-shape` core (`f64` units, `FnOnce` lifecycle setters, slot factories); **delete `structural_options`** | `structural_options` freezes a list of builder flags in policy: every new flag edits a reviewed list in addition to the code. That is exactly the "process artefact". Replace by API rule "flag methods take no argument" already covered by api-shape. |
| `effect-writes` | 221 | keep (only 2 markers) | it enforces S-5 "effects do not write state", which is the principal anti-pattern |
| `os-branch` | 85 | keep (6 markers) | |
| `kit-literal-color` | 65 | keep | cheap, no markers |
| `devtools-codegen --check` | about 1500 lines | delete with Python (TL-09) | |
| `internal-contract` | in check_source | change (TL-05) | |
| `--report`, `--report-audit`, `--only`, `--crate`, `--module` | | keep `--only/--crate/--module` (they enable tier a), delete `--report-audit` |

- breaking: no   size: M
- confidence: medium

### TL-07 Policy duplication in `workspace.toml`
- kind: PROCESS / DUP
- where: `dev/xtask/policy/workspace.toml` lines 1-4 (top of the file are four loose English comment lines placed before the heading, belonging to deleted entries), `[visible_to]`, `[module_rules.*]`
- evidence: free-form comments about old entries ("ViewTransform is a canonical passive...", "ScrollSnap and release sampling...") sit at the top, outside any section, so they document nothing. `kivact-devtools-protocol = [...6 consumers...]` and each app crate has a `= []` entry (14 lines) only to be registered.
- fix: apps/tests/examples (`kivact-example-*`, `*-catalog`, `kivact-docs`, `xtask`, `kivact-bench`, `kivact-template-basic`) are registered by `[apps] publish = false` or by glob: delete the 14 `= []` lines; delete the 4 stray comment lines. When crates merge (TL-08) shrink `visible_to`.
- breaking: no   size: S   confidence: high

### TL-08 DevTools: crate split is partly fake
- kind: STRUCT
- where: `crates/kivact-devtools-{protocol,agent,client,mcp}`, `crates/kivact-devtools`, `crates/kivact-core/Cargo.toml:22-31`
- evidence: `-client` (617 lines: `discovery.rs`, `connection.rs`, `error.rs`) is used by the GUI, the MCP and, as a dev-dependency, by the agent tests. `-agent` (1672) is used only by `kivact-core` (feature `devtools`) and tests. Both are client/server halves of one protocol and change together when a method is added: one reason to change per pair, no consumer that wants one half without the protocol.
- fix (decided):
  1. Merge `kivact-devtools-client` into `kivact-devtools-protocol` under `protocol::client` (feature `client`, default off so the app-side build does not pull discovery). The boundary "wire vocabulary plus how to open a connection" is explainable by few types (`Connection`, `Target`, `discover()`).
  2. Keep `kivact-devtools-agent` as a crate because it pulls `png` into the target app only when the `devtools` feature is on and it has no reference to core internals (verified by Cargo.toml: depends only on protocol, serde_json, png, tracing). A real boundary: leave it.
  3. Keep `kivact-devtools-mcp` (729) as a separate binary crate: different artifact, different dependency set.
  4. `kivact-devtools` (GUI) stays; it is also the largest dogfooding app (8.2k lines, 40 tests).
  Result: 5 crates become 4 and `visible_to` loses one entry.
- breaking: no (not yet published; `docs` mention `cargo install kivact-devtools kivact-devtools-mcp`)   kpi: -   size: M
- confidence: medium-high

### TL-09 Python client and codegen are disproportionate
- kind: DEAD / PROCESS
- where: `python/kivact-devtools` (3972 lines, `_generated.py` 2860), `dev/xtask/src/devtools_codegen*` (about 1500 lines including `python.rs` 428, `read.rs` 506, tests 206), `crates/kivact-devtools-protocol/schema/kivact-devtools.schema.json` (105 KB), `docs/for-kivact-user/*/how-to-guide/devtools.md` (344 lines, section at line 289 uses Python)
- evidence: One commit touches `python/` (single `git log` entry, 2026-11-09). The MCP server already exposes every protocol method as a tool, generated from the same schema, and AI agents (the stated consumer in strategy.md line 90: "MCP・Python の client") use MCP. The Python path costs: generator in xtask, `--check` in every `xtask check`, a committed 2860-line generated file, `unittest` suite (391 lines) never run in CI or the gate (optional).
- fix: delete `python/`, delete `dev/xtask/src/devtools_codegen*`, keep the JSON schema generated by a protocol unit test or `cargo run -p kivact-devtools-protocol --example schema` and tested for equality with the committed file (the schema is the MCP's input). Remove the Python section from the user docs (en and jp) and strategy.md line 90 (`MCP・Python` becomes `MCP`). If an owner decision wants Python kept (see Q1), then move to `pip` package generated at release time only, not committed.
- breaking: no (devtools unpublished); user docs change   kpi: -   size: M
- confidence: high on size, medium on whether anyone uses it

### TL-10 DevTools in core: hot-crate weight
- kind: STRUCT / INEFF
- where: `crates/kivact-core/src/devtools/*.rs` (5144 lines, 21 files), `view/devtools_apply.rs`, `view/devtools_hooks.rs` (1463), 197 `devtools` references in core src, `client/dispatch/{a11y,delivery,key,pointer}.rs`
- evidence: feature-gated (`devtools = ["dep:kivact-devtools-agent", ...]`), so K6 is untouched when off, but core compile time and every `cfg` branch in dispatch paths carry it. In `cargo check --all-features` and the gate (`--all-features`) it is always compiled. In the pointer and key dispatch it adds branches that the structural reading of the core (reference client/dispatch) must understand.
- fix: boundary rule for the redesign: the devtools module in core exposes ONE trait object `DevtoolsHooks` set once at startup (`Option<Box<dyn DevtoolsHooks>>` on the App client), and dispatch files call only `hooks.on_pointer(..)`. Fold `view/devtools_*` into `devtools/` (the hook implementation) so `view/` has no `devtools_` file. Do not split into another crate: the module needs core internals (tree, styles, motion), a fake boundary.
- breaking: no   kpi: K6 (when enabled)   size: M   confidence: medium (did not read all files)

### TL-11 CI only runs fmt + xtask check on ubuntu
- kind: PROCESS
- where: `.github/workflows/ci.yml`
- evidence: `cargo fmt --check` and `cargo xtask check --no-write` on `ubuntu-latest`. No test, no Windows, no clippy beyond xtask's Google subset. The primary platform is Windows, the build of Windows-only code (`arvane-native` win32, UIA tests, embed) is never type-checked in CI. The CI comment says the full gate "runs locally before merge".
- fix: add job `windows-check` on `windows-latest`: `cargo check --workspace --all-targets` (no tests, no window opens, minutes with rust-cache) plus `cargo test -p kivact-public-api-tests --test e2e -- --test-threads=1` with `KIVACT_TEST_MIRROR=off` (virtual host, no focus problem). Keep local checkpoints for real windows. Do not add GPU tests to CI.
- breaking: no   size: S   confidence: medium

### TL-12 `kivact-test` crate carries tests of itself (about 2.6k lines in `src`)
- kind: STRUCT / CONTRA
- where: `crates/kivact-test/src/screen/{accessibility,context,disposal,gpu,host_sync,real_window,scene_cache,window}_tests.rs`, `editing_drop_probe.rs`; crate doc `lib.rs`: "There is no mock host, no host conformance suite and no unit test of internal modules."
- evidence: the crate doc says no internal unit tests, yet it holds `host_sync_tests.rs` (448), `gpu_tests.rs` (580), `scene_cache_tests.rs` (327), `real_window_tests.rs` (239). Several are described as regressions (`context_tests.rs`: "Real-window regressions for layer context entry", `scene_cache_tests.rs`: "...scene regressions"). A harness's own tests should live with `dev/tests/kivact-public-api` or `tests/` of the crate so that they do not compile in the library test target of the crate that every other crate dev-depends on.
- fix: move all `screen/*_tests.rs` to `crates/kivact-test/tests/` as integration tests (they use only public API anyway) or fold the ones that test public behavior into `dev/tests/kivact-public-api/tests/e2e/`. Delete the ones that assert internals (`host_sync_tests`, `scene_cache_tests` check core/engine internal caches). Then `kivact-test` lib compiles without `#[cfg(test)]` modules; update the crate doc (no contradiction).
- breaking: no   size: M   confidence: medium

### TL-13 Kit consumer tests duplicate e2e conformance
- kind: DUP
- where: `crates/kivact-ui-{basic 2726, pop 884, game 884, creative 4274, soft 5039}/tests/consumer/*.rs`; `dev/tests/kivact-public-api/tests/e2e/{theme_conformance 732, ui_*.rs, apg_*.rs ×20, interaction_*.rs ×13}`
- evidence: five kits each have a `consumer` binary. soft has 34 files with 102 tests (e.g. `selector_keyboard`, `selector_touch`, `selector_focus` for one control; `toast_motion`, `toast_swipe`, `tab_indicator_motion`, `slider_touch_motion`, `switch_touch`, `date_touch`), many of which are behaviors of the shared `kivact-interaction` behavior (focus, keyboard, touch) that the e2e `apg_*` and `interaction_*` already cover for one kit. pop and game (884 lines each) are the same shape as basic: parallel copies. "Motion" tests (`*_motion`) compare time-based visuals, which contradicts "no pixel or time exact comparison".
- fix: decide a per-kit contract: each kit keeps ONE `consumer` file of at most about 600 lines that proves "this kit wires behavior X": one test per control family, parametrised over kits through the existing `theme_conformance` harness (move the shared checks there: selector keyboard/touch/focus, disclosure, toast). Delete kit-level tests that exercise `kivact-interaction` behavior (`*_keyboard`, `*_touch`, `*_focus`) once the e2e `interaction_*` covers the same behavior. Keep kit-specific features (creative node editor, dock, color wheel, arrangement, numeric, bezier; soft chat, grouped table, tree reveal).
- breaking: no   kpi: -   size: L   confidence: medium (did not diff test bodies)

### TL-14 Tests of removed or contradictory behaviour; slow patterns
- kind: CONTRA / INEFF
- where regression comments: `e2e/flows.rs:250` ("Regression: errors raised inside an error_boundary's fallback..."), `e2e/runtime.rs:261,264,268,271`, `e2e/task.rs:375-391` (5), `e2e/view.rs:413-422` (4), `e2e/embed/windows.rs:1` ("UIA regressions"); `kivact-test/src/screen/{context,scene_cache}_tests.rs`
- evidence (sleep patterns): 12 real-time sleeps in tests: `gpu_on_demand.rs:89`, `gpu_on_demand_shared.rs:69,127,144`, `gpu_on_demand_windows.rs:87` (all `GRACE`, a real-time wait for the idle compositor release, `MEM-8` says 1 s), `embed/uia.rs:75` (25 ms polling), `devtools.rs:71`, agent tests `agent.rs:54,120`, `kivact-test/screen/artifacts.rs:312`. The GPU-on-demand tests are also the ones that open real windows and need a GPU. The tests run on a "virtual clock" yet sleep in real time for `GRACE`.
- fix: (1) remove the `// Regression:` label and keep only tests whose name states the contract; delete ones that are only a past bug (decide by reading the single assertion: if it asserts a public behavior keep it, rename it). (2) Replace `GRACE` sleeps by advancing the virtual clock with `Screen::advance` where the host exposes the idle timer on the virtual clock. If the presenter timer is real-time only, expose it on the virtual host's clock (kivact-engine testing). (3) Devtools and agent tests: replace 10 ms polling with the connection's blocking read with timeout.
- breaking: no   kpi: -   size: M   confidence: medium

### TL-15 Public-API e2e is a single 152-file binary
- kind: INEFF / STRUCT
- where: `dev/tests/kivact-public-api/Cargo.toml` `[[test]] name = "e2e" path = "tests/e2e/main.rs"`
- evidence: all 755 tests link as ONE binary with all five kits, wgpu (via `gpu-interop-producer` and `kivact-gpu-share` dev-deps), windows-sys, `kivact-devtools-*`. Any change in any crate recompiles and relinks the 38k-line crate before even one test runs. For tier (a) an implementer wants `cargo test --test e2e -- <module>` but pays the full link every time. `kivact-public-api-tests` is also a `visible_to` consumer of all kits.
- fix: split `e2e` into 3 test binaries along the real dependency axes: `e2e_core` (runtime, view, style, layout, text, input, a11y: needs only `kivact`), `e2e_ui` (`ui_*`, `apg_*`, `theme_conformance`, `interaction_*`: needs the kits), `e2e_native` (embed, gpu_*, foreign_window, devtools, viewport_gpu_interop, drawable_export_gpu: needs wgpu, windows crates, producer). Tier (a) then links only the binary that matches the changed crate. The split is a real boundary: they differ in dev-dependencies, not in size.
- breaking: no   kpi: -   size: M   confidence: high

### TL-16 Doc and script leftovers
- kind: DEAD
- where: `dev/xtask/state-audit.md` (checked out generated file, git-ignored but present), `dev/tools/subset-catalog-font.py`, `dev/README.md`, `docs/for-kivact-developer/manual-verification.md`, stray lines at top of `workspace.toml`
- evidence: `state-audit.md` is a generated, uncommitted file in the source tree. `subset-catalog-font.py` is referenced by `decisions.md`, `examples/shared-assets/fonts/SOURCE.md` and `examples/kivact-ui-basic/tests/catalog.rs`, so it is alive but only for font provenance. The manual-verification table (about 10 rows) is the only record of what is not automated; it is kept current (references to `embed/geometry.rs` etc.), keep.
- fix: delete `state-audit.md` generation (TL-04). Leave the font script. Make sure `dev/README.md` lists the three tiers (TL-01).
- size: S   confidence: medium

### TL-17 `check-features.sh`: serial `cargo check` of 24 feature entries
- kind: INEFF
- where: `dev/tools/check-features.sh:19-43`
- evidence: each entry runs `cargo check -p kivact -p kivact-ui [-p crate] --no-default-features --lib --features entry`, serially. Entries differ in the feature set so each re-resolves and recompiles dependents (kivact + kivact-ui + core + engine). Several entries are duplicates of one another (`kivact/devtools` and `kivact-core/devtools`; `kivact/gpu-interop` and `kivact-core/gpu-interop`; `kivact/tokio`, `kivact-core/tokio`, `kivact-runtime/tokio`; `kivact-test/devtools` and `kivact/test`).
- fix: keep only the facade features of public meaning (`kivact/{devtools,embed,gpu,gpu-interop,tokio,test}`, plus `arvane-render/gpu`, `arvane-native/embed`, `kivact-gpu-share/{export,import,wgpu}`): about 11 runs instead of 24. Move to tier (c).
- breaking: no   size: S   confidence: high

## 3. Target structure for this area

### 3.1 Tiers and exact commands

**(a) fast, per unit (target: 1-3 min, no window lock, no real window)**

```
cargo fmt --all -- --check
cargo check -p <changed crates> --all-targets
cargo xtask check --no-write --only state-fields,api-shape,effect-writes,canonical-path,name-collision,kit-literal-color,os-branch,module-rules,visible-to --crate <changed crate>
KIVACT_TEST_MIRROR=off cargo test -p <changed crate>                       # unit + crate tests
KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- <module filter>   # after TL-15; before it: --test e2e -- <module>
```

`--only` and `--crate` already exist; the list of rule names must be verified against `xtask --help` when written into the work order (I did not run it).

**(b) medium, per wave (target: tens of minutes, no lock)**

```
cargo fmt --all -- --check
cargo xtask check --no-write
cargo clippy --workspace --all-targets -- -D warnings         # default features only, once
KIVACT_TEST_MIRROR=off cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs
cargo check -p kivact-docs --all-targets                      # type-checks all no_run guide blocks (after TL-03)
```

**(c) full, only at checkpoints (before merge or when a native path changed), under the lock**

```
cargo fmt --all -- --check
cargo xtask check --no-write            # and `cargo xtask diagrams` is no longer part of the gate (TL-02)
cargo clippy --workspace --all-targets --all-features -- -D warnings
dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features
dev/tools/window-lock.sh --timeout 5400 -- cargo test -p gpu-interop-producer --all-features -- --test-threads=1
dev/tools/window-lock.sh --timeout 5400 -- cargo test -p kivact-docs --doc --all-features -- --test-threads=1    # en only, about 140 runnable blocks (TL-03)
KIVACT_TEST_MIRROR=off cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --no-default-features    # no lock needed (TL-01)
dev/tools/check-features.sh             # reduced to about 11 entries (TL-17)
```

The `--no-default-features` pass for `gpu-interop-producer` and `kivact-docs` is dropped: the producer has no behaviour without its default feature, docs need the features to compile. The window-lock gains `--if-mirror`; the (c) lines that really open windows (`KIVACT_TEST_MIRROR=auto|required`, GPU lane, `foreign_window_run`, manual-verification rows) keep the lock.

### 3.2 Layout after the rework

- `crates/kivact-devtools-protocol` (wire plus `client` feature), `-agent`, `-mcp`, `kivact-devtools` (GUI). Python and codegen deleted. Core's devtools module behind one hook trait.
- `dev/xtask`: size limit, `visible_to` (with global dev-dep rule), `module_rules`, `state` (struct-level), `api-shape` (plus canonical path and name collision), `effect-writes`, `os-branch`, `kit-literal-color`, style delegation to `vendor/xtask-core`. Deleted: `structural_options`, `devtools_codegen*`, `--report-audit`, most of `claims.rs`.
- `dev/tests`: `kivact-public-api` with three test binaries, `docs` en only plus a jp/en parity check, `gpu-interop-producer` unchanged.
- `architecture/` untracked; diagrams generated on demand.
- Tests of the harness move out of `kivact-test/src`.

## 4. Public API
This area exposes no public API except `kivact::test` (the E2E harness). It is fine as written (Playwright-like); the only friction is the contradiction in TL-12 and the `RenderOptions::mirror` vs `KIVACT_TEST_MIRROR` double control (env var default plus per-test override: keep both, document that env wins only when the test did not set it).

## 5. Open questions for the owner
1. **Python client**: delete (my recommendation: MCP covers AI use) or keep as release-time generated package? This decides about 5.5k lines (python 3972 plus codegen 1500).
2. **state-fields**: accept the struct-level mutable-state rule (TL-04) or keep per-field markers? Deleting the marker scheme also deletes `docs/for-kivact-developer/architecture.md` lines 94-96 and S-section phrasing.
3. **`architecture/` publishing**: untrack entirely, or publish via CI/GitHub Pages?
4. **Windows CI**: allowed to add a Windows runner (cost) for `cargo check --all-targets` and virtual-host e2e? Without it, Windows breakage is only caught locally.
5. **Kit test consolidation (TL-13)**: how strongly should kits be tested independently? The proposal removes kit-level tests of shared interaction behavior and moves the shared checks into `theme_conformance`.
