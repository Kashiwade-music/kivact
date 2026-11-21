//! Role: The `icons!` list: each icon name written once, its svg embedded from a directory.

/// `[(name, bytes); N]` of the svg files `<dir>/<name>.svg`, with `<dir>`
/// relative to the calling crate's `Cargo.toml`:
/// `icons!("assets/icons", ["home", "bell"])`.
#[macro_export]
macro_rules! icons {
    ($dir:literal, [$($name:literal),* $(,)?]) => {
        [$((
            $name,
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $dir, "/", $name, ".svg"))
                as &'static [u8],
        )),*]
    };
}
