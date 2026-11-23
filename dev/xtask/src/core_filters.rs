//! Role: Drop xtask-core findings that this workspace's policy exempts.
//!
//! A dev-dependency never breaks `visible_to`, and examples,
//! the app template and the docs tests are written as user code without
//! `//! Role:` lines.

use std::path::Path;

/// Directories, relative to the root, whose modules need no `//! Role:` line.
const ROLE_EXEMPT: [&str; 3] = ["examples/", "templates/", "dev/tests/docs/"];

pub(crate) fn apply(root: &Path, errors: &mut Vec<String>) {
    let root = root.to_string_lossy().replace('\\', "/");
    errors.retain(|error| !dev_visibility(error) && !role_exempt(&root, error));
}

fn dev_visibility(error: &str) -> bool {
    error.contains("` has a dev dependency")
}

fn role_exempt(root: &str, error: &str) -> bool {
    let error = error.replace('\\', "/");
    error.contains("missing module role")
        && ROLE_EXEMPT.iter().any(|directory| {
            error.contains(&format!("{root}/{directory}")) || error.starts_with(directory)
        })
}

#[cfg(test)]
mod tests {
    //! Role: Check which xtask-core findings the workspace policy drops.
    #[test]
    fn drops_dev_visibility_and_exempt_role_findings() {
        let mut errors = vec![
            "`a` has a dev dependency on `b`, which is visible only to [\"c\"]".to_owned(),
            "`a` has a normal dependency on `b`, which is visible only to [\"c\"]".to_owned(),
            "/repo/examples/kivact/counter/src/main.rs (crate): missing module role; add //! Role: <description>".to_owned(),
            "/repo/crates/kivact-core/src/lib.rs (crate): missing module role; add //! Role: <description>".to_owned(),
        ];
        super::apply(std::path::Path::new("/repo"), &mut errors);
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(errors[0].contains("normal dependency"));
        assert!(errors[1].contains("kivact-core"));
    }
}
