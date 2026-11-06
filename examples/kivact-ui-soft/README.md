# kivact-ui-soft catalog

Searchable component catalog for [kivact-ui-soft](../../crates/kivact-ui-soft/README.md) (the Astryx Neutral theme). The top bar switches between the light and dark appearances, the left column lists every component by category with a search field, and the right side shows the selected component's page with its live examples.

Run from the repository root:

```sh
cargo run -p kivact-ui-soft-catalog
cargo run -p kivact-ui-soft-catalog --no-default-features
```

The GPU feature is enabled by default; `--no-default-features` builds the CPU renderer only.

## Pages and examples

`src/pages.rs` registers every page (`PageInfo` with its `slug`, `name` and `category`) in the order of the component list. `src/docs/` holds the examples, one module per category (`controls.rs`, `forms.rs`, `navigation.rs`, `overlays.rs`, `collections.rs`, `content.rs`, `feedback.rs`, `dates.rs`, `utilities.rs`); `src/docs/examples.rs` routes a page's slug to its category. Each example is a `specimen(title, view, code)`: the live view and the short code that builds it. To add a page:

1. Add a `PageInfo` to `PAGES` in `src/pages.rs` (and raise the array length).
2. Return the page's specimens from the `examples(slug)` function of the matching category module in `src/docs/`.

`src/shell.rs` holds the top bar and the searchable list, `src/page.rs` the page heading, and `src/model.rs` the selection, search and appearance signals.

## Fonts

The fonts belong to this app. `src/fonts.rs` embeds the Figtree variable fonts from `assets/fonts/figtree/`, and `SOURCE.md`, `OFL.txt` and `SHA256SUMS.txt` there record their provenance, license and exact bytes.

## Tests

`tests/catalog.rs` opens every page through the search in both appearances and checks that the appearance toggle keeps the open page, and exercises the stateful examples (carousel, table, file input, item lists). The tests open a window when mirroring is available (`KIVACT_TEST_MIRROR`, `auto` by default, `off` for windowless runs), so run them through `dev/tools/window-lock.sh`:

```sh
dev/tools/window-lock.sh -- cargo test -p kivact-ui-soft-catalog
```

`tests/lookbook.rs` is an ignored test that captures every page in both appearances as PNG files under `target/soft-lookbook` for visual review; `KIVACT_SOFT_LOOKBOOK_PAGES=<slug>,<slug>` limits it to some pages.

```sh
dev/tools/window-lock.sh -- cargo test -p kivact-ui-soft-catalog --test lookbook -- --ignored
```
