//! Role: The reporting mode of one state-fields run.

/// How one run reports: local notes on the console, and the audit file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Mode {
    // xtask-allow(state-fields): independent: all four combinations are valid because console notes and the audit file are separate requests
    pub(crate) report: bool,
    // xtask-allow(state-fields): independent: the audit file can be requested without console notes
    pub(crate) audit: bool,
}
