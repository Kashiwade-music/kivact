# Third-party notices and provenance

Kivact is Copyright (c) 2026 Kashiwade <kashiwade@outlook.com> and is licensed under `MIT OR Apache-2.0` ([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)). The third-party materials below keep their own licenses and copyright notices, which must be retained.

| Material | Local license / provenance record |
|---|---|
| Embedded Windows IMM32 implementation | [ime.rs](crates/arvane-native/src/port/win32/ime.rs) references winit 0.30.13 `src/platform_impl/windows/ime.rs`, Apache-2.0 ([source](https://github.com/rust-windowing/winit/blob/v0.30.13/src/platform_impl/windows/ime.rs)); license terms in [LICENSE-APACHE](LICENSE-APACHE) |
| Tuffy font shipped by Arvane | [Tuffy-LICENSE.txt](crates/arvane/assets/fonts/Tuffy-LICENSE.txt): original public-domain release and fallback grant with the original named authors. Preserve the supplied text |
| InterVariable test font | [InterVariable-LICENSE.txt](crates/arvane/tests/fonts/InterVariable-LICENSE.txt): original copyright notice and SIL Open Font License |
| Tabler icons in Pop/Game/Creative | [Pop license](crates/kivact-ui-pop/assets/LICENSE-tabler-icons.txt), [Game license](crates/kivact-ui-game/assets/LICENSE-tabler-icons.txt), [Creative license](crates/kivact-ui-creative/LICENSE-tabler-icons.txt): supplied MIT terms and Paweł Kuna copyright notice |
| Lucide icons in Basic and Soft | [Basic license](crates/kivact-ui-basic/assets/LICENSE-lucide.txt), [Soft license](crates/kivact-ui-soft/assets/LICENSE-lucide.txt). The ISC terms of lucide-static 0.468.0 include Lucide Contributors and Feather (Cole Bemis, MIT) copyright notices. The SVG files retain their `@license` comments |
| Catalog font families | Each catalog's font asset directory retains original OFL.txt notices, SOURCE.md and SHA256SUMS.txt. Inter, Barlow, Barlow Condensed, JetBrains Mono, Geist, Geist Mono, Noto Sans JP and [Figtree](examples/kivact-ui-soft/assets/fonts/figtree/SOURCE.md) come from Google Fonts |
| ICU4X compiled locale data and formatters | ICU4X 2.4.0, Copyright Unicode, Inc. and others, [Unicode License v3](https://github.com/unicode-org/icu4x/blob/icu%402.4.0/LICENSE). All locales from the compiled data packages are included |
| Project-made images | The image fixtures of the tests (PNG, JPEG, GIF, SVG), and the example images (`examples/kivact-ui-basic/assets/images`) are simple shapes and gradients made for Kivact. They are part of Kivact and fall under its license |

Test fixture and font references across packages are test inputs, not permission to expose internal crates through the public facade. License and provenance records must move with assets.

This inventory covers material in this repository. Licenses of Cargo dependencies are declared in their own packages and are not repeated here.

The internal `kivact-test` harness uses the same bundled Tuffy test font and upstream font notices as `kivact-engine`; this move adds no third-party assets.
