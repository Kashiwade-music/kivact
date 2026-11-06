# kivact-ui-game catalog

Interactive styled learning catalog for [kivact-ui-game](../../crates/kivact-ui-game/README.md), with the original shell, chapters, themes and stateful demonstrations.

Run from the repository root:

```sh
cargo run -p kivact-ui-game-catalog
cargo run -p kivact-ui-game-catalog --no-default-features
```

The GPU feature is enabled by default; CPU remains available. Read src/game.rs and its modules alongside the kit's public component API. Fonts belong to this app; assets/fonts/SOURCE.md, OFL.txt and SHA256SUMS.txt record provenance, licensing and exact bytes.

Preserved public-driver regression tests are grouped in tests/catalog.rs and tests/catalog/. They retain optional KIVACT_TEST_GPU and KIVACT_CATALOG_EVIDENCE settings. Window mirroring follows KIVACT_TEST_MIRROR (auto by default, off for windowless runs). Execute tests with window mirroring serially through dev/tools/window-lock.sh.
