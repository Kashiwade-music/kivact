//! Role: Complete direct-dependency allowlists of kits, examples and GPU interop.

use std::path::Path;
use std::process::Command;

/// Each consumer's complete list of direct dependencies (normal, dev and build,
/// on every target, renamed declarations by package name). Official kits use
/// exactly the facade any consumer gets.
const ALLOWLIST: &[(&str, &[&str])] = &[
    ("kivact-embed-host", &["kivact"]),
    (
        "kivact",
        &["kivact-core", "kivact-interaction", "kivact-ui"],
    ),
    (
        "kivact-core",
        &[
            "kivact-engine",
            "kivact-editing",
            "kivact-wire",
            "kivact-reactive",
            "kivact-i18n",
            "kivact-task",
            "kivact-motion",
            "kivact-style",
            "tracing",
        ],
    ),
    ("kivact-i18n", &["kivact-reactive", "kivact"]),
    ("kivact-task", &["kivact-reactive", "async-task", "tracing"]),
    (
        "kivact-motion",
        &["kivact-reactive", "kivact-wire", "kivact"],
    ),
    (
        "kivact-style",
        &[
            "kivact-reactive",
            "kivact-wire",
            "kivact-motion",
            "kurbo",
            "kivact",
            "kivact-editing",
        ],
    ),
    ("kivact-interaction", &["kivact-core"]),
    (
        "kivact-ui",
        &["kivact-core", "kivact-interaction", "kivact"],
    ),
    ("arvane-shell", &["raw-window-handle", "kivact-types"]),
    // Keep the native OS integration boundary explicit when adding adapters.
    (
        "arvane-native",
        &[
            "arvane-shell",
            "kivact-types",
            "winit",
            "softbuffer",
            "raw-window-handle",
            "tracing",
            "accesskit",
            "accesskit_winit",
            "arboard",
            "rfd",
            "sys-locale",
            "accesskit_unix",
            "xim",
            "libc",
            "x11rb",
            "xkbcommon-dl",
            "rustix",
            "accesskit_windows",
            "windows-sys",
        ],
    ),
    (
        "kivact-text",
        &[
            "kivact-editing",
            "kivact-lower",
            "kivact-types",
            "arvane",
            "unicode-segmentation",
        ],
    ),
    ("kivact-lower", &["kivact-types", "kivact-wire", "arvane"]),
    ("kivact-layout", &["kivact-wire", "taffy"]),
    ("kivact-wire", &["kivact-types", "kivact-editing"]),
    ("kivact-editing", &["kivact-types", "unicode-segmentation"]),
    (
        "kivact-public-api-tests",
        &[
            "kivact",
            "kivact-ui-basic",
            "kivact-ui-game",
            "kivact-ui-pop",
            "kivact-ui-creative",
            "async-channel",
            "tracing",
            "trybuild",
            "gpu-interop-producer",
            "windows",
            "windows-sys",
        ],
    ),
    ("kivact-ui-basic", &["kivact"]),
    ("kivact-ui-pop", &["kivact"]),
    ("kivact-ui-game", &["kivact"]),
    // The GPU frame tests of `viewport_surface` (feature `gpu-interop`) run an engine of their own.
    ("kivact-ui-creative", &["kivact", "gpu-interop-producer"]),
    ("kivact-example-counter", &["kivact"]),
    ("kivact-example-form", &["kivact"]),
    ("kivact-example-list", &["kivact"]),
    ("kivact-example-async", &["kivact"]),
    ("kivact-example-component", &["kivact"]),
    ("kivact-template-basic", &["kivact"]),
    (
        "kivact-example-task-board",
        &["kivact", "serde", "serde_json"],
    ),
    (
        "kivact-example-mini-postman",
        &["kivact", "serde", "serde_json"],
    ),
    ("kivact-ext-apps", &["kivact", "kivact-external-component"]),
    ("kivact-ui-basic-catalog", &["kivact", "kivact-ui-basic"]),
    ("kivact-ui-pop-catalog", &["kivact", "kivact-ui-pop"]),
    ("kivact-ui-game-catalog", &["kivact", "kivact-ui-game"]),
    (
        "kivact-ui-creative-catalog",
        &["kivact", "kivact-ui-creative", "async-channel"],
    ),
    // GPU texture sharing: Arvane's unsafe code below wgpu, the engine's
    // side of the sharing and the tests' engine reach the native APIs
    // through these alone.
    (
        "kivact-gpu-share",
        &["kivact-types", "wgpu", "pollster", "ash", "windows"],
    ),
    (
        "gpu-interop-producer",
        &["kivact-types", "kivact-gpu-share", "ash", "windows", "wgpu"],
    ),
];

fn metadata() -> serde_json::Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the workspace root is three levels above this crate");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let output = Command::new(cargo)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--manifest-path",
        ])
        .arg(root.join("Cargo.toml"))
        .output()
        .expect("cargo metadata runs");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("cargo metadata prints JSON")
}

fn packages(metadata: &serde_json::Value) -> &Vec<serde_json::Value> {
    metadata["packages"]
        .as_array()
        .expect("metadata has a packages array")
}

#[test]
fn allowlisted_consumers_are_workspace_members() {
    let metadata = metadata();
    let names: Vec<&str> = packages(&metadata)
        .iter()
        .filter_map(|package| package["name"].as_str())
        .collect();
    let missing: Vec<&str> = ALLOWLIST
        .iter()
        .map(|(consumer, _)| *consumer)
        .filter(|consumer| !names.contains(consumer))
        .collect();
    assert!(
        missing.is_empty(),
        "allowlisted consumers missing from the workspace: {missing:?}"
    );
}

#[test]
fn consumers_depend_only_on_their_allowlist() {
    let metadata = metadata();
    let mut violations = Vec::new();
    for package in packages(&metadata) {
        let name = package["name"].as_str().expect("package has a name");
        let Some((_, allowed)) = ALLOWLIST.iter().find(|(consumer, _)| *consumer == name) else {
            continue;
        };
        let dependencies = package["dependencies"]
            .as_array()
            .expect("package has a dependencies array");
        for dependency in dependencies {
            let dependency_name = dependency["name"].as_str().expect("dependency has a name");
            if allowed.contains(&dependency_name) {
                continue;
            }
            let kind = dependency["kind"].as_str().unwrap_or("normal");
            violations.push(format!(
                "`{name}` has a {kind} dependency on `{dependency_name}`; allowed: {allowed:?}"
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "dependencies outside the allowlist:\n{}",
        violations.join("\n")
    );
}
