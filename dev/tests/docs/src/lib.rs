//! Doctest harness for the English Markdown guides in `docs/` and `README.md`.
//!
//! The build script includes every English guide as the documentation of an
//! empty module under [`guides`], so `cargo test -p kivact-docs --doc` compiles
//! and runs every ```` ```rust ```` block that is not `no_run`. The `no_run`
//! blocks (which open a window) are compiled instead as the `checked` modules,
//! which `cargo check -p kivact-docs --all-targets` type-checks without linking.
//! Blocks that are not Rust use ```` ```text ````. The Japanese guides are kept
//! equal to the English ones by `cargo xtask check` (`docs-pairs`).
//!
//! A block that shows an app together with its `#[cfg(test)] mod tests` is
//! `test_harness`: rustdoc builds it with `--test`, so the tests compile and
//! run while `main` (which opens a window) is never called. A unit test of
//! this crate rejects such blocks under `no_run`.

/// One module per guide, mirroring the paths under `docs/`.
pub mod guides;

/// The `no_run` blocks of the guides, type-checked and never called.
#[cfg(test)]
#[allow(
    dead_code,
    unused,
    unreachable_pub,
    non_snake_case,
    unsafe_code,
    clippy::all,
    clippy::pedantic,
    clippy::todo,
    clippy::unimplemented,
    clippy::too_many_lines,
    clippy::unwrap_used,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::dbg_macro,
    reason = "guide examples are written for readers; they are compiled here only to type-check them"
)]
mod checked;

#[cfg(test)]
mod tests;
