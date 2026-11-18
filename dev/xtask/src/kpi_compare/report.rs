//! Role: Read the measured values and status lines of one run.sh report.

/// One report: its lines as written.
pub(super) struct Report {
    lines: Vec<String>,
}

impl Report {
    /// The report, or an error when run.sh marked it INVALID.
    pub(super) fn parse(text: &str) -> Result<Self, String> {
        if text
            .lines()
            .any(|line| line.trim_start().starts_with("INVALID:"))
        {
            return Err("the report is INVALID; measure again".into());
        }
        Ok(Self {
            lines: text.lines().map(str::to_owned).collect(),
        })
    }

    /// Whether `run.sh --no-catalog` skipped the basic catalog.
    pub(super) fn skipped_catalog(&self) -> bool {
        self.lines
            .iter()
            .any(|line| line.trim_start().starts_with("SKIPPED: basic catalog"))
    }

    /// Whether the OS throttled the frames of the run that `line` names (`rows=.. renderer=..`).
    pub(super) fn warned(&self, line: &str) -> bool {
        let wanted = format!("WARNING {line}:");
        self.lines
            .iter()
            .any(|row| row.trim_start().starts_with(&wanted))
    }

    /// The value of `field` on the row that starts with `line` (`last` = the row's last word).
    pub(super) fn value(&self, line: &str, field: &str) -> Option<f64> {
        let row = self
            .lines
            .iter()
            .filter_map(|row| row.strip_prefix("- "))
            .find(|row| {
                row.strip_prefix(line)
                    .is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
            })?;
        if field == "last" {
            return row.split_whitespace().last()?.parse().ok();
        }
        row.split_whitespace().find_map(|word| {
            word.split_once('=')
                .filter(|(key, _)| *key == field)
                .and_then(|(_, value)| value.parse().ok())
        })
    }
}
