//! Role: Answers whether a native window handle a test App gave is alive.

/// Whether a native window handle a test App gave to the caller is still
/// alive. A handle from a test App's virtual backend is answered from its
/// thread's registry; another handle is asked of the OS.
pub fn is_native_window_alive(window: kivact_types::NativeWindow) -> bool {
    arvane_virtual::is_native_window_alive(window)
        .unwrap_or_else(|| kivact_engine::testing::native_window_alive(window))
}
