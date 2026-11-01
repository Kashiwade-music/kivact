# Third-party notices and provenance

Kivact is Copyright (c) 2026 Kashiwade <kashiwade@outlook.com> and is licensed under `MIT OR Apache-2.0` ([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)). The third-party materials below keep their own licenses and copyright notices, which must be retained.

| Material | Local license / provenance record |
|---|---|
| Embedded Windows IMM32 implementation | [ime.rs](crates/arvane-native/src/embed/windows/ime.rs) references winit 0.30.13 `src/platform_impl/windows/ime.rs`, Apache-2.0 ([source](https://github.com/rust-windowing/winit/blob/v0.30.13/src/platform_impl/windows/ime.rs)); license terms in [LICENSE-APACHE](LICENSE-APACHE) |
| Tuffy font shipped by Arvane | [Tuffy-LICENSE.txt](crates/arvane/assets/fonts/Tuffy-LICENSE.txt): original public-domain release and fallback grant with the original named authors. Preserve the supplied text |
| InterVariable test font | [InterVariable-LICENSE.txt](crates/arvane/tests/fonts/InterVariable-LICENSE.txt): original copyright notice and SIL Open Font License |
| Tabler icons in Pop/Game/Creative | [Pop license](crates/kivact-ui-pop/assets/LICENSE-tabler-icons.txt), [Game license](crates/kivact-ui-game/assets/LICENSE-tabler-icons.txt), [Creative license](crates/kivact-ui-creative/LICENSE-tabler-icons.txt): supplied MIT terms and Paweł Kuna copyright notice |
| Lucide icons in Basic | [Basic license](crates/kivact-ui-basic/assets/LICENSE-lucide.txt): ISC terms of lucide-static 0.468.0 with the Lucide Contributors and Feather (Cole Bemis, MIT) copyright notices; the SVG files keep their `@license` comments |
| Catalog font families | Each catalog's font asset directory retains original OFL.txt notices, SOURCE.md and SHA256SUMS.txt. Inter, Barlow, Barlow Condensed, JetBrains Mono, Geist, Geist Mono and Noto Sans JP were sourced from the Google Fonts repository according to the supplied record. Checksums identify bytes; they do not replace license notices |
| Project-made images | The image fixtures of the tests (PNG, JPEG, GIF, SVG), and the example images (`examples/kivact-ui-basic/assets/images`) are simple shapes and gradients made for Kivact. They are part of Kivact and fall under its license |

Test fixture and font references across packages are test inputs, not permission to expose internal crates through the public facade. License and provenance records must move with assets.

This inventory covers material in this repository. Licenses of Cargo dependencies are declared in their own packages and are not repeated here.
