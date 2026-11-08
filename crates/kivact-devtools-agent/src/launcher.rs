//! Role: Start the DevTools app for a target.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use kivact_devtools_protocol::DevtoolsConfig;

const NOT_FOUND: &str = "DevTools app not found. Install it with `cargo install --git https://github.com/Kashiwade-music/kivact kivact-devtools` or set \"app\" in .kivact/devtools.json";

/// Start the first DevTools app that can be found, with `--target <discovery path>`. The child is
/// not waited for.
pub(crate) fn launch(
    config: &DevtoolsConfig,
    root: &Path,
    discovery_path: &Path,
) -> Result<(), String> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf));
    let env_app = std::env::var_os("KIVACT_DEVTOOLS_APP");
    for path in candidates(config, root, env_app, exe_dir.as_deref()) {
        let mut command = std::process::Command::new(&path);
        command.arg("--target").arg(discovery_path);
        match command.spawn() {
            Ok(_) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("cannot start {}: {error}", path.display())),
        }
    }
    Err(NOT_FOUND.to_owned())
}

/// In order: the `app` setting, `KIVACT_DEVTOOLS_APP`, next to this executable, `kivact-devtools`
/// on PATH.
pub(crate) fn candidates(
    config: &DevtoolsConfig,
    root: &Path,
    env_app: Option<OsString>,
    exe_dir: Option<&Path>,
) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if let Some(app) = &config.app {
        found.push(root.join(app));
    }
    if let Some(app) = env_app.filter(|app| !app.is_empty()) {
        found.push(root.join(app));
    }
    if let Some(dir) = exe_dir {
        found.push(dir.join(format!("kivact-devtools{}", std::env::consts::EXE_SUFFIX)));
    }
    found.push(PathBuf::from("kivact-devtools"));
    found
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the order in which the DevTools app is looked for.

    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use kivact_devtools_protocol::DevtoolsConfig;

    use super::candidates;

    fn configured() -> DevtoolsConfig {
        DevtoolsConfig {
            app: Some("tools/devtools".to_owned()),
            ..DevtoolsConfig::default()
        }
    }

    #[test]
    fn candidates_keep_the_documented_order() {
        let found = candidates(
            &configured(),
            Path::new("C:/work"),
            Some(OsString::from("env/devtools")),
            Some(Path::new("C:/bin")),
        );
        assert_eq!(found.len(), 4);
        assert_eq!(found[0], Path::new("C:/work").join("tools/devtools"));
        assert_eq!(found[1], Path::new("C:/work").join("env/devtools"));
        assert_eq!(
            found[2],
            Path::new("C:/bin").join(format!("kivact-devtools{}", std::env::consts::EXE_SUFFIX))
        );
        assert_eq!(found[3], PathBuf::from("kivact-devtools"));
    }

    #[test]
    fn without_an_app_setting_the_last_is_the_path_lookup() {
        let found = candidates(
            &DevtoolsConfig::default(),
            Path::new("C:/work"),
            Some(OsString::from("env/devtools")),
            Some(Path::new("C:/bin")),
        );
        assert_eq!(found.len(), 3);
        assert_eq!(found.last(), Some(&PathBuf::from("kivact-devtools")));
    }
}
