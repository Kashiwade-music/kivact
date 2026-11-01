//! Role: Compile-time checks for public API usage and error messages.
//!
//! Compile checks: every usage example compiles against the public API;
//! the compile-time rows of the misuse table fail with the recorded errors.
//! Without the feature `gpu-interop`, its API does not exist (trybuild
//! builds the fixtures with the features of this build, so the check runs
//! only in builds without it).

#[test]
fn ui_fixtures() {
    // trybuild does not forward this harness's embed dev-dependency feature
    // to its generated kivact dependency. Check the handle with real features here.
    #[cfg(feature = "embed")]
    {
        fn send_sync<T: Send + Sync>() {}
        send_sync::<kivact::embed::Embedded>();
    }
    let t = trybuild::TestCases::new();
    #[cfg(not(feature = "embed"))]
    t.compile_fail("tests/ui/fail_without_embed/*.rs");
    t.pass("tests/ui/pass/*.rs");
    t.compile_fail("tests/ui/fail/*.rs");
    #[cfg(not(feature = "gpu-interop"))]
    t.compile_fail("tests/ui/fail_without_gpu_interop/*.rs");
}
