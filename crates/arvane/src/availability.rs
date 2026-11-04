//! Role: Query whether a hardware GPU context can be created.

pub fn gpu_unavailable() -> Option<String> {
    #[cfg(feature = "gpu")]
    {
        arvane_render::GpuContext::new(&arvane_render::GpuEnv::from_env(
            kivact_types::GpuPreference::default(),
        ))
        .err()
        .map(|error| error.to_string())
    }
    #[cfg(not(feature = "gpu"))]
    {
        Some("Arvane was built without GPU support".into())
    }
}
