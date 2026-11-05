# kivact-ui-pop catalog

Interactive styled learning catalog for [kivact-ui-pop](../../crates/kivact-ui-pop/README.md), with the original shell, chapters, themes and stateful demonstrations.

Run from the repository root:

```sh
cargo run -p kivact-ui-pop-catalog
cargo run -p kivact-ui-pop-catalog --no-default-features
```

The GPU feature is enabled by default; CPU remains available. Read src/pop.rs and its modules alongside the kit's public component API. The catalog uses unchanged Inter and Noto Sans JP bytes from examples/shared-assets/fonts; Pop's Barlow files remain in assets/fonts. Each location retains provenance, OFL notices and exact SHA256 hashes.

Preserved public-driver regression tests are grouped in tests/catalog.rs and tests/catalog/. They retain optional KIVACT_TEST_GPU and KIVACT_CATALOG_EVIDENCE settings. Window mirroring follows KIVACT_TEST_MIRROR (auto by default, off for windowless runs). Execute tests with window mirroring serially through dev/tools/window-lock.sh.
