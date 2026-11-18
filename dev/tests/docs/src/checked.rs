//! The `no_run` blocks of the English guides as modules, type-checked and never run.

/// Includes the modules the build script generated.
macro_rules! generated_blocks {
    () => {
        include!(concat!(env!("OUT_DIR"), "/checked.rs"));
    };
}

generated_blocks!();
