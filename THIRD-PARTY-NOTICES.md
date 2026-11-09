# Third-party notices and provenance

Kivact is Copyright (c) 2026 Kashiwade <kashiwade@outlook.com> and is licensed under `MIT OR Apache-2.0` ([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)). The third-party materials below keep their own licenses and copyright notices, which must be retained.

| Material | Local license / provenance record |
|---|---|
| Embedded Windows IMM32 implementation | [ime.rs](crates/arvane-native/src/port/win32/ime.rs) references winit 0.30.13 `src/platform_impl/windows/ime.rs`, Apache-2.0 ([source](https://github.com/rust-windowing/winit/blob/v0.30.13/src/platform_impl/windows/ime.rs)); license terms in [LICENSE-APACHE](LICENSE-APACHE) |
| Tuffy font shipped by Arvane | [Tuffy-LICENSE.txt](crates/arvane/assets/fonts/Tuffy-LICENSE.txt): original public-domain release and fallback grant with the original named authors. Preserve the supplied text |
| InterVariable test font | [InterVariable-LICENSE.txt](crates/arvane/tests/fonts/InterVariable-LICENSE.txt): original copyright notice and SIL Open Font License |
| Tabler icons in Pop/Game/Creative | [Pop license](crates/kivact-ui-pop/assets/LICENSE-tabler-icons.txt), [Game license](crates/kivact-ui-game/assets/LICENSE-tabler-icons.txt), [Creative license](crates/kivact-ui-creative/LICENSE-tabler-icons.txt): supplied MIT terms and Paweł Kuna copyright notice |
| Lucide icons in Basic and Soft | [Basic license](crates/kivact-ui-basic/assets/LICENSE-lucide.txt), [Soft license](crates/kivact-ui-soft/assets/LICENSE-lucide.txt). The ISC terms of lucide-static 0.468.0 include Lucide Contributors and Feather (Cole Bemis, MIT) copyright notices. The SVG files retain their `@license` comments |
| Catalog font families | Shared families in `examples/shared-assets/fonts/` and the remaining catalog asset directories retain original OFL.txt notices, SOURCE.md and SHA256SUMS.txt. Inter, Barlow, Barlow Condensed, JetBrains Mono, Geist, Geist Mono, Noto Sans JP and [Figtree](examples/kivact-ui-soft/assets/fonts/figtree/SOURCE.md) come from Google Fonts |
| ICU4X compiled locale data and formatters | ICU4X 2.4.0, Copyright Unicode, Inc. and others, [Unicode License v3](https://github.com/unicode-org/icu4x/blob/icu%402.4.0/LICENSE). All locales from the compiled data packages are included |
| Project-made images | The image fixtures of the tests (PNG, JPEG, GIF, SVG), and the example images (`examples/kivact-ui-basic/assets/images`) are simple shapes and gradients made for Kivact. They are part of Kivact and fall under its license |

Test fixture and font references across packages are test inputs, not permission to expose internal crates through the public facade. License and provenance records must move with assets.

This inventory covers material in this repository. Licenses of other Cargo dependencies are declared in their own packages and are not repeated here; the crates listed under the next heading are recorded because this release adds them or makes them direct.

The internal `kivact-test` harness uses the same bundled Tuffy test font and upstream font notices as `kivact-engine`; this move adds no third-party assets.

## Shared catalog fonts

Basic, Pop, Game and Creative read byte-identical shared families from
[`examples/shared-assets/fonts`](examples/shared-assets/fonts). Inter, Geist,
Geist Mono and Noto Sans JP retain their original family OFL notices, combined
[provenance](examples/shared-assets/fonts/SOURCE.md) and
[SHA-256 records](examples/shared-assets/fonts/SHA256SUMS.txt). No font was upgraded.
Pop's Barlow, Game's Barlow Condensed and JetBrains Mono, and Soft's Figtree
remain beside their original notices in their catalog asset directories.

## resvg 0.46.0 filter port

`crates/arvane-compose/src/cpu/graph/` ports the filter driver, the color space conversions, the sRGB tables and the primitive filters of resvg 0.46.0 (`src/filter/`, Copyright 2018 the Resvg Authors), dual licensed under `MIT OR Apache-2.0`. The ported files keep their attribution headers. The gaussian blur and the drop shadow are Kivact's own and do not come from resvg.

## Cargo dependencies added or made direct in this release

The identifiers come from each crate's `license` field in the locked version (Cargo.lock). The full license texts ship in each crate package (`LICENSE`, or `LICENSE-MIT` and `LICENSE-APACHE`), and the copyright lines below are taken from those files.

| Crate (version) | License | Copyright and provenance |
|---|---|---|
| `naga` (29.0.4) | MIT OR Apache-2.0 | Copyright (c) 2025 The gfx-rs developers. Made a direct dependency of `kivact-shader` (WGSL validation, `wgsl-in` only). Full texts: `LICENSE.MIT` and `LICENSE.APACHE` |
| `zbus` (5.19.0) | MIT | Copyright (c) 2024 Zeeshan Ali Khan & zbus contributors. Already used by `arvane-native`; this work fixes its features (`blocking-api`, `async-io`). Full text: `LICENSE` |
| `fontdb` (0.23.0) | MIT | Copyright (c) 2020 Yevhenii Reizner. Comes with `usvg` (resvg's `text` feature); `arvane-asset` adds it directly with the `fs` and `memmap` features for system fonts. Full text: `LICENSE` |
| `rustybuzz` (0.20.1) | MIT | Copyright (c) HarfBuzz developers; Copyright (c) 2020 Yevhenii Reizner. Comes with `usvg` (resvg's `text` feature). Full text: `LICENSE` |
| `serde` (1.0.229) | MIT OR Apache-2.0 | Authors: Erick Tryzelaar and David Tolnay. Made a direct dependency of `kivact-devtools-protocol` (wire values). Full texts: `LICENSE-MIT` and `LICENSE-APACHE` |
| `serde_json` (1.0.151) | MIT OR Apache-2.0 | Authors: Erick Tryzelaar and David Tolnay. Made a direct dependency of the DevTools crates (JSON-RPC and values). Full texts: `LICENSE-MIT` and `LICENSE-APACHE` |
