//! Role: Query whether a hardware GPU context can be created.

pub fn gpu_unavailable() -> Option<String> {
    arvane_compose::GpuContext::new(&arvane_compose::GpuEnv::from_env(
        kivact_types::GpuPreference::default(),
    ))
    .err()
    .map(|error| error.to_string())
}
