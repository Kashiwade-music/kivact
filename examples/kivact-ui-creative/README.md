# kivact-ui-creative catalog

Interactive styled learning catalog for [kivact-ui-creative](../../crates/kivact-ui-creative/README.md), with the original shell, chapters, themes and stateful demonstrations.

Run from the repository root:

```sh
cargo run -p kivact-ui-creative-catalog
cargo run -p kivact-ui-creative-catalog --no-default-features
```

The GPU feature is enabled by default; CPU remains available. Read src/creative.rs and its modules alongside the kit's public component API. Fonts belong to this app; assets/fonts/SOURCE.md, OFL.txt and SHA256SUMS.txt record provenance, licensing and exact bytes.

Preserved public-driver regression tests are grouped in tests/catalog.rs and tests/catalog/. They retain optional KIVACT_CATALOG_GPU and KIVACT_CATALOG_EVIDENCE settings. Execute GUI regressions serially in an authorized verification session.
