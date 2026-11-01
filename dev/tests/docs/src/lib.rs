//! Role: Doctest harness for the Markdown guides in `docs/` and the READMEs.
//!
//! The build script includes every guide as the documentation of an empty
//! module under [`guides`], so `cargo test -p kivact-docs --doc` compiles
//! (and, unless marked `no_run`, runs) every ```` ```rust ```` block. Blocks
//! that open a window are `no_run`; blocks that are not Rust use
//! ```` ```text ````.
//!
//! A block that shows an app together with its `#[cfg(test)] mod tests` is
//! `test_harness`: rustdoc builds it with `--test`, so the tests compile and
//! run while `main` (which opens a window) is never called. Under `no_run`
//! the `cfg(test)` module would not even be compiled; a unit test of this
//! crate rejects such blocks.

/// One module per guide, mirroring the paths under `docs/`.
pub mod guides;

#[cfg(test)]
mod tests;
