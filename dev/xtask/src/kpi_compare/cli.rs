//! Role: The cargo xtask kpi-compare command: read two reports, print the table.

use super::config::{self, Row};
use super::judge::{Judged, Verdict, judge};
use super::report::Report;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Exit 0: every row passes; 1: a row regressed; 3: only required improvements are missed; 2: bad input.
pub(crate) fn run(root: &Path, arguments: &[String]) -> ExitCode {
    match compare(root, arguments) {
        Ok((table, code)) => {
            print!("{table}");
            ExitCode::from(code)
        }
        Err(error) => {
            eprintln!("kpi-compare: {error}");
            ExitCode::from(2)
        }
    }
}

fn compare(root: &Path, arguments: &[String]) -> Result<(String, u8), String> {
    let (base, after, config) = paths(root, arguments)?;
    let read = |path: &Path| {
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
    };
    let rows = config::parse(&read(&config)?)?;
    let base = Report::parse(&read(&base)?).map_err(|error| format!("B0: {error}"))?;
    let after = Report::parse(&read(&after)?).map_err(|error| format!("checkpoint: {error}"))?;
    table(&rows, &base, &after)
}

fn paths(root: &Path, arguments: &[String]) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    match arguments {
        [base, after] => Ok((base.into(), after.into(), root.join("dev/bench/kpi.toml"))),
        [base, after, flag, config] if flag == "--config" => {
            Ok((base.into(), after.into(), config.into()))
        }
        _ => Err(
            "usage: cargo xtask kpi-compare <B0 report> <checkpoint report> [--config <kpi.toml>]"
                .into(),
        ),
    }
}

/// The Markdown table of every row and the exit code.
pub(super) fn table(rows: &[Row], base: &Report, after: &Report) -> Result<(String, u8), String> {
    let mut out = String::from(
        "| 行 | B0 | checkpoint | 比 | 必達 | 改善の必達 | 目標 | 判定 |\n|---|---|---|---|---|---|---|---|\n",
    );
    let (mut regressed, mut missed) = (false, false);
    for row in rows {
        let (verdict, cells) = line(row, base, after)?;
        regressed |= verdict == Verdict::Regressed;
        missed |= verdict == Verdict::ImprovementMissed;
        out.push_str(&cells);
    }
    let code = if regressed {
        1
    } else if missed {
        3
    } else {
        0
    };
    Ok((out, code))
}

fn line(row: &Row, base: &Report, after: &Report) -> Result<(Verdict, String), String> {
    let improve = row
        .improve
        .map_or("-".to_owned(), |limit| format!("≤ {limit:.2}"));
    let limits = format!(
        "≤ {:.2} | {improve} | ≤ {:.2}",
        row.kind.allowance(),
        row.target
    );
    if row.catalog && after.skipped_catalog() {
        return Ok((
            Verdict::Skipped,
            format!("| {} | - | - | - | {limits} | skipped |\n", row.name),
        ));
    }
    let value = |report: &Report, which: &str| {
        report
            .value(&row.line, &row.field)
            .ok_or_else(|| format!("{which}: no `{}` on the row `{}`", row.field, row.line))
    };
    let (b, a) = (value(base, "B0")?, value(after, "checkpoint")?);
    let judged = if row.warning && (base.warned(&row.line) || after.warned(&row.line)) {
        Judged {
            ratio: None,
            verdict: Verdict::Unverified,
            met_target: false,
        }
    } else {
        judge(row, b, a)
    };
    let ratio = judged
        .ratio
        .map_or("floor".to_owned(), |ratio| format!("{ratio:.3}"));
    let verdict = match judged.verdict {
        Verdict::Ok if judged.met_target => "ok（目標）",
        Verdict::Ok => "ok",
        Verdict::Regressed => "後退",
        Verdict::ImprovementMissed => "改善未達",
        Verdict::Unverified => "未検証（WARNING）",
        Verdict::Skipped => "skipped",
    };
    Ok((
        judged.verdict,
        format!(
            "| {} | {b} | {a} | {ratio} | {limits} | {verdict} |\n",
            row.name
        ),
    ))
}
