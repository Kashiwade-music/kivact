# kivact-ui-game catalog

The catalog of [kivact-ui-game](../../crates/kivact-ui-game/README.md): eleven
pages of its components, shown by the shared shell of
[catalog-shell](../catalog-shell).

Run from the repository root:

```sh
cargo run -p kivact-ui-game-catalog
cargo run -p kivact-ui-game-catalog --no-default-features
```

Each page is a file of `src/pages/`; its demo sits between `// example:` and
`// end example`, so the code on screen is the code that runs. The fonts belong
to this app: `assets/fonts/SOURCE.md`, `OFL.txt` and `SHA256SUMS.txt` record
their provenance, licensing and exact bytes.

The tests walk every page in both modes (`catalog_shell::check`) and drive the
controls of some pages through the public test driver. Execute them through
`dev/tools/window-lock.sh`, because the test driver opens windows.
