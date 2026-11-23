//! Role: Judge one KPI row from its B0 and checkpoint values.

use super::config::Row;

/// The verdict of one row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Verdict {
    /// No regression, and the required improvement (if any) is reached.
    Ok,
    /// Worse than the allowance.
    Regressed,
    /// No regression, but the required improvement is not reached.
    ImprovementMissed,
    /// The OS throttled the frames in one of the reports.
    Unverified,
    /// `run.sh --no-catalog` skipped the row.
    Skipped,
}

/// One judged row: its ratio (none when the noise floor decides) and its verdict.
pub(super) struct Judged {
    pub(super) ratio: Option<f64>,
    pub(super) verdict: Verdict,
    pub(super) met_target: bool,
}

pub(super) fn judge(row: &Row, base: f64, after: f64) -> Judged {
    if base < row.floor || base <= 0.0 {
        let ok = after <= row.floor.max(0.0);
        let verdict = if ok { Verdict::Ok } else { Verdict::Regressed };
        return Judged {
            ratio: None,
            verdict,
            met_target: ok,
        };
    }
    let ratio = after / base;
    let verdict = if ratio > row.kind.allowance() {
        Verdict::Regressed
    } else if row.improve.is_some_and(|limit| ratio > limit) {
        Verdict::ImprovementMissed
    } else {
        Verdict::Ok
    };
    Judged {
        ratio: Some(ratio),
        verdict,
        met_target: ratio <= row.target,
    }
}
