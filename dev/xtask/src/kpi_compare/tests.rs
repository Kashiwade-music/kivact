//! Role: Verify row parsing, value reading, verdicts and exit codes of kpi-compare.

use super::config::{Kind, Row, parse};
use super::judge::{Verdict, judge};
use super::report::Report;

fn row(field: &str, line: &str, kind: Kind, improve: Option<f64>, floor: f64) -> Row {
    Row {
        name: line.to_owned(),
        line: line.to_owned(),
        field: field.to_owned(),
        kind,
        target: 0.5,
        improve,
        floor,
        catalog: false,
        warning: false,
    }
}

const REPORT: &str = "# KPI x\n- rows=3000 renderer=cpu frames=300 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=12.50 ws_kib=1 work_ms p50=1.0 p95=2.0 p99=3.0 scene_p95=0.5 raster_p95=0.4 interval_p99=17.0\n- K6 counter default 1000\n- L pointer_move_n3000 n=50 mean=1.0 p50=0.9 p95=1.5 p99=2.0\n";

#[test]
fn reads_values_by_key_and_by_last_word() {
    let report = Report::parse(REPORT).unwrap();
    assert_eq!(report.value("rows=3000 renderer=cpu", "p95"), Some(2.0));
    assert_eq!(
        report.value("rows=3000 renderer=cpu", "interval_p99"),
        Some(17.0)
    );
    assert_eq!(
        report.value("rows=3000 renderer=cpu", "cpu_pct_of_core"),
        Some(12.5)
    );
    assert_eq!(report.value("K6 counter default", "last"), Some(1000.0));
    assert_eq!(report.value("L pointer_move_n3000", "p95"), Some(1.5));
    assert_eq!(report.value("rows=0 renderer=cpu", "p95"), None);
}

#[test]
fn judges_allowances_improvements_and_floors() {
    let timed = row("p95", "x", Kind::Time, Some(0.80), 0.0);
    assert_eq!(judge(&timed, 10.0, 7.9).verdict, Verdict::Ok);
    assert_eq!(
        judge(&timed, 10.0, 10.4).verdict,
        Verdict::ImprovementMissed
    );
    assert_eq!(judge(&timed, 10.0, 10.6).verdict, Verdict::Regressed);
    let sized = row("ws_kib", "x", Kind::Size, None, 0.0);
    assert_eq!(judge(&sized, 100.0, 102.0).verdict, Verdict::Ok);
    assert_eq!(judge(&sized, 100.0, 102.5).verdict, Verdict::Regressed);
    let floored = row("idle_cpu_ms_per_s", "x", Kind::Time, None, 1.0);
    assert_eq!(judge(&floored, 0.0, 0.5).verdict, Verdict::Ok);
    assert!(judge(&floored, 0.0, 0.5).ratio.is_none());
    assert_eq!(judge(&floored, 0.2, 1.5).verdict, Verdict::Regressed);
}

#[test]
fn invalid_warned_and_skipped_reports() {
    assert!(Report::parse("- a 1\n\nINVALID: build failed\n").is_err());
    let warned = Report::parse(
        "  WARNING rows=3000 renderer=cpu: 30 fps\n- rows=3000 renderer=cpu interval_p99=40.0\n",
    )
    .unwrap();
    assert!(warned.warned("rows=3000 renderer=cpu"));
    assert!(!warned.warned("rows=0 renderer=cpu"));
    let mut catalog = row(
        "last",
        "K6 basic-catalog no-gpu",
        Kind::Size,
        Some(0.97),
        0.0,
    );
    catalog.catalog = true;
    let base = Report::parse("- K6 basic-catalog no-gpu 1000\n").unwrap();
    let skipped = Report::parse("SKIPPED: basic catalog (--no-catalog)\n").unwrap();
    let (table, code) = super::cli::table(&[catalog.clone()], &base, &skipped).unwrap();
    assert!(table.contains("skipped"), "{table}");
    assert_eq!(code, 0);
    let missing = Report::parse("- K6 counter default 1\n").unwrap();
    assert!(super::cli::table(&[catalog], &base, &missing).is_err());
}

#[test]
fn exit_codes_follow_the_worst_row() {
    let rows = [row(
        "last",
        "K6 counter default",
        Kind::Size,
        Some(0.97),
        0.0,
    )];
    let base = Report::parse("- K6 counter default 1000\n").unwrap();
    let worse = Report::parse("- K6 counter default 1100\n").unwrap();
    let same = Report::parse("- K6 counter default 1000\n").unwrap();
    let better = Report::parse("- K6 counter default 900\n").unwrap();
    assert_eq!(super::cli::table(&rows, &base, &worse).unwrap().1, 1);
    assert_eq!(super::cli::table(&rows, &base, &same).unwrap().1, 3);
    assert_eq!(super::cli::table(&rows, &base, &better).unwrap().1, 0);
}

#[test]
fn the_committed_table_parses() {
    let text =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../bench/kpi.toml")).unwrap();
    let rows = parse(&text).unwrap();
    assert_eq!(rows.len(), 23);
    assert_eq!(rows.iter().filter(|row| row.improve.is_some()).count(), 6);
    assert_eq!(rows.iter().filter(|row| row.catalog).count(), 3);
}

/// A report as the BASE `dev/bench/run.sh` (2b8df5ac, before this unit) writes it: B0 is measured
/// with that script in W0 (headings, row lines and their order copied from its `log` calls).
const BASE_REPORT: &str = "# KPI B0 (2026-11-18T09:00Z, MINGW64_NT-10.0-26200, commit 2b8df5a)

release build of kivact-example-counter (cold for its own target dir): 95 s

## K1 startup (5 runs, median) and the first frame
- K1 main_to_first_present renderer=default median_ms=181.4 samples=[ 180.9 181.4 182.0 181.1 183.2 ]
- K4 first_frame_raster renderer=default median_ms=11.2 samples=[ 11.0 11.2 11.3 11.1 11.4 ]
- K1 main_to_first_present renderer=cpu median_ms=175.0 samples=[ 174.8 175.0 175.3 174.9 176.0 ]
- K4 first_frame_raster renderer=cpu median_ms=11.0 samples=[ 10.9 11.0 11.1 11.0 11.2 ]
- K1 catalog_spawn_to_first_present renderer=default median_ms=420.5 samples=[ 419.0 420.5 421.0 420.1 422.3 ] (includes OS process creation)

## K2/K3 idle (3 s settle, then 10 s sampled)
- counter renderer=default ws_kib=61234 private_kib=40210 idle_cpu_ms_per_s=0.150 idle_frames=0 cpu_ticks_100ns=15000 [renderer=gpu adapter=x]
- counter renderer=cpu ws_kib=52100 private_kib=35000 idle_cpu_ms_per_s=0.120 idle_frames=0 cpu_ticks_100ns=12000
- counter renderer=gpu ws_kib=70100 private_kib=48000 idle_cpu_ms_per_s=0.300 idle_frames=0 cpu_ticks_100ns=30000 [renderer=gpu adapter=x]
- basic-catalog renderer=default ws_kib=120000 private_kib=90000 idle_cpu_ms_per_s=0.200 idle_frames=0 cpu_ticks_100ns=20000
- rows3000-still renderer=default ws_kib=150000 private_kib=110000 idle_cpu_ms_per_s=0.100 idle_frames=0 cpu_ticks_100ns=10000
- basic-catalog renderer=cpu ws_kib=110000 private_kib=85000 idle_cpu_ms_per_s=0.180 idle_frames=0 cpu_ticks_100ns=18000
- rows3000-still renderer=cpu ws_kib=140000 private_kib=100000 idle_cpu_ms_per_s=0.090 idle_frames=0 cpu_ticks_100ns=9000

## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)
- item-box-0 renderer=cpu ws_kib=50000 private_kib=34000 idle_cpu_ms_per_s=0.080 idle_frames=0 cpu_ticks_100ns=8000
- item-box-3000 renderer=cpu ws_kib=52000 private_kib=35000 idle_cpu_ms_per_s=0.080 idle_frames=0 cpu_ticks_100ns=8000
- K2 per_item mode=box kib_per_item=0.67
- K2 per_item mode=text kib_per_item=1.20
- K2 per_item mode=button kib_per_item=2.10
- K2 per_item mode=row kib_per_item=3.40

## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)
- heap mode=box live_bytes_per_item=600
    [box] H site 120 allocs kivact_engine::tree::node::Node::new

## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)
- rows=0 renderer=default frames=300 fps=60.0 frame_period_ns=16666666 cpu_pct_of_core=5.00 ws_kib=60000 work_ms p50=0.5 p95=0.9 p99=1.2 scene_p95=0.2 raster_p95=0.3 interval_p99=17.1
  WARNING rows=3000 renderer=cpu: 40.0 fps for a 60.0 Hz display (the OS throttled the window; work times stay valid, fps and interval do not)
- rows=3000 renderer=cpu frames=200 fps=40.0 frame_period_ns=16666666 cpu_pct_of_core=35.20 ws_kib=150000 work_ms p50=4.0 p95=6.5 p99=8.0 scene_p95=2.0 raster_p95=3.0 interval_p99=30.0

## K5 update latency (kv_latency, test driver, CPU, mirror off)
- L mount_counter n=1 mean=20.000 p50=20.000 p95=20.000 p99=20.000
- L counter_click n=200 mean=0.800 p50=0.750 p95=1.100 p99=1.400
- L mount_n3000 n=1 mean=900.000 p50=900.000 p95=900.000 p99=900.000
- L unrelated_update_n3000 n=100 mean=1.000 p50=0.900 p95=1.500 p99=2.000
- L row_update_n3000 n=100 mean=1.200 p50=1.100 p95=1.800 p99=2.200
- L pointer_move_n3000 n=100 mean=2.000 p50=1.900 p95=2.600 p99=3.000
- L wheel_n3000 n=100 mean=3.000 p50=2.800 p95=3.900 p99=4.500
- S stats_n3000 Stats { nodes: 9000 }

## K6 release binary size (bytes)
- K6 counter default 9000000
- K6 kv_startup default 9100000
- K6 kv_startup no-default-features 5000000
- K6 basic-catalog default 15000000
- K6 kv_startup no-gpu 7000000
- K6 basic-catalog no-gpu 12000000
";

#[test]
fn reads_a_report_in_the_base_run_sh_format() {
    let text =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../bench/kpi.toml")).unwrap();
    let rows = parse(&text).unwrap();
    let base = Report::parse(BASE_REPORT).unwrap();
    for row in &rows {
        assert!(
            base.value(&row.line, &row.field).is_some(),
            "the base report has no `{}` on `{}`",
            row.field,
            row.line
        );
    }
    assert_eq!(
        base.value("K1 main_to_first_present renderer=default", "median_ms"),
        Some(181.4)
    );
    assert_eq!(
        base.value("counter renderer=cpu", "idle_cpu_ms_per_s"),
        Some(0.12)
    );
    assert_eq!(
        base.value("K6 basic-catalog no-gpu", "last"),
        Some(12_000_000.0)
    );
    assert!(base.warned("rows=3000 renderer=cpu"));
    assert!(!base.skipped_catalog());
    let (table, code) = super::cli::table(&rows, &base, &base).unwrap();
    assert!(table.contains("未検証（WARNING）"), "{table}");
    assert_eq!(
        code, 3,
        "the same report misses every required improvement:\n{table}"
    );
}
