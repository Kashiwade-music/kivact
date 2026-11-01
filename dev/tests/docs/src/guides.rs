//! Role: Generated doctest modules, one per Markdown guide.

/// Includes the modules the build script generated.
macro_rules! generated_guides {
    () => {
        include!(concat!(env!("OUT_DIR"), "/guides.rs"));
    };
}

generated_guides!();
