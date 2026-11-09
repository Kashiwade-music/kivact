//! Role: Opens a source location in the user's editor.

use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

use kivact_devtools_protocol as p;

const DEFAULT_EDITOR: [&str; 3] = ["code", "--goto", "{file}:{line}:{column}"];

/// Run the editor command of `config` (blocking). `{file}` is `root` joined with the location's file.
pub(crate) fn open_in_editor(
    config: &p::DevtoolsConfig,
    root: &str,
    location: &p::SourceLocation,
) -> Result<(), String> {
    let template: Vec<String> = if config.editor.is_empty() {
        DEFAULT_EDITOR.iter().map(|s| (*s).to_string()).collect()
    } else {
        config.editor.clone()
    };
    let file = Path::new(root)
        .join(&location.file)
        .to_string_lossy()
        .into_owned();
    let args = expand(&template, &file, location.line, location.column);
    let Some((program, rest)) = args.split_first() else {
        return Err(hint("the editor command is empty"));
    };
    run(program, rest).map_err(|e| hint(&format!("could not start `{program}`: {e}")))
}

/// Fill `{file}`, `{line}` and `{column}` in every argument of `template`.
fn expand(template: &[String], file: &str, line: u32, column: u32) -> Vec<String> {
    template
        .iter()
        .map(|argument| {
            argument
                .replace("{file}", file)
                .replace("{line}", &line.to_string())
                .replace("{column}", &column.to_string())
        })
        .collect()
}

/// Run `program`. If it is not found, try `<program>.cmd` once: on Windows `code` is `code.cmd`,
/// and `Command` adds only `.exe` by itself.
fn run(program: &str, rest: &[String]) -> std::io::Result<()> {
    let status = match Command::new(program).args(rest).status() {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            Command::new(format!("{program}.cmd")).args(rest).status()?
        }
        other => other?,
    };
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!("exit status {status}")))
    }
}

fn hint(problem: &str) -> String {
    format!(
        "{problem}. Set \"editor\" in .kivact/devtools.json, for example [\"code\", \"--goto\", \"{{file}}:{{line}}:{{column}}\"]."
    )
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the editor command template.

    use super::*;

    #[test]
    fn expand_fills_file_line_and_column() {
        let template = vec![
            "code".to_owned(),
            "--goto".to_owned(),
            "{file}:{line}:{column}".to_owned(),
            "{file}".to_owned(),
        ];
        assert_eq!(
            expand(&template, "/work/src/main.rs", 10, 5),
            vec![
                "code".to_owned(),
                "--goto".to_owned(),
                "/work/src/main.rs:10:5".to_owned(),
                "/work/src/main.rs".to_owned(),
            ]
        );
    }
}
