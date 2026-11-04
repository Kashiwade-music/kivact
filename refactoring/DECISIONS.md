# Foundation v3 decisions

| Date | Unit | Question / derivation | Decision and reason |
|---|---|---|---|
| 2026-11-04 | P-0 | Design documents exist only on the starting docs branch | Created integration from main and fast-forwarded the docs branch before unit worktrees. |
| 2026-11-04 | All | Work order requires author approval for design gaps | User explicitly delegates all judgments; orchestrator decides using API consistency, React/Solid accessibility, simple internals, and maintainability. |
| 2026-11-04 | P-0 | First baseline compilation exhausted disk space | Remove only this run's generated target/debug cache and disable incremental/debug symbols for checks; keep runtime debug assertions and optimize-level policy unchanged. |
| 2026-11-04 | P-0 | Baseline GPU read_back aborted with heap corruption after 5/6 tests | Preserve baseline failure; rerun the unchanged full suite with RUST_TEST_THREADS=1 to check whether concurrent GPU tests cause the failure. No source/test suppression. |
| 2026-11-04 | CP0 onward | Unchanged GPU read_back succeeds 6/6 with serial test execution after default parallel heap corruption | Use RUST_TEST_THREADS=1 for checkpoint E2E. Preserve the full test selection, keep window-lock, and retain the baseline concurrency failure as a known existing issue. |
| 2026-11-04 | P-0 | Baseline xtask waited after all architecture outputs were written | Volta node shim left an ELK grandchild holding stderr open. Verified and stopped only that worker; xtask returned 0. Set XTASK_NODE to the bundled node executable for later gates; do not modify vendor. |
