# kivact-ui-pop catalog

Interactive styled component catalog for [kivact-ui-pop](../../crates/kivact-ui-pop/README.md). The shared catalog shell shows one page per component family; each page's demos are in `src/pages/`, and each demo's code is the code on screen.

Run from the repository root:

```sh
cargo run -p kivact-ui-pop-catalog
cargo run -p kivact-ui-pop-catalog --no-default-features
```

The GPU feature is enabled by default; CPU remains available. The catalog uses unchanged Inter bytes and a subset of Noto Sans JP from examples/shared-assets/fonts; Pop's Barlow files remain in assets/fonts. Each location retains provenance, OFL notices and exact SHA256 hashes.

The page walker of the shared shell and the stateful checks of the forms, the overlays and the conversation are in tests/catalog.rs. Execute tests with window mirroring serially through dev/tools/window-lock.sh.
