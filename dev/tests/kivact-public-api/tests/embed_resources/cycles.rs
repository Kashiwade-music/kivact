//! Role: Twenty real embedded open/close cycles for CPU and default GPU.
#![allow(
    unsafe_code,
    reason = "renderer environment is changed only while the isolated resource test mutex is held"
)]
use super::{RESOURCE_TEST, embed_resources};
#[path = "evidence.rs"]
mod evidence;
use evidence::{Evidence, Snapshot};
use kivact::style::Color;
use kivact::{App, prelude::*, test::TestParent};
use std::{
    ffi::OsString,
    path::Path,
    time::{Duration, Instant},
};

struct Environment(Vec<(&'static str, Option<OsString>)>);
impl Environment {
    fn set(cpu: bool, log: &std::path::Path) -> Self {
        let previous = ["ARVANE_RENDERER", "KIVACT_FRAME_LOG"]
            .into_iter()
            .map(|name| (name, std::env::var_os(name)))
            .collect();
        if cpu {
            // SAFETY: RESOURCE_TEST serializes tests; no App workers exist yet.
            unsafe { std::env::set_var("ARVANE_RENDERER", "cpu") };
        } else {
            // SAFETY: RESOURCE_TEST serializes tests; no App workers exist yet.
            unsafe { std::env::remove_var("ARVANE_RENDERER") };
        }
        // SAFETY: RESOURCE_TEST serializes tests; no App workers exist yet.
        unsafe { std::env::set_var("KIVACT_FRAME_LOG", log) };
        Self(previous)
    }
}
impl Drop for Environment {
    fn drop(&mut self) {
        for (name, value) in &self.0 {
            if let Some(value) = value {
                // SAFETY: All Apps have joined before restoring the serialized environment.
                unsafe { std::env::set_var(name, value) };
            } else {
                // SAFETY: All Apps have joined before restoring the serialized environment.
                unsafe { std::env::remove_var(name) };
            }
        }
    }
}
fn gpu_status(notes: &str) -> (bool, bool, bool, bool) {
    let mut seen_gpu = false;
    let mut active = false;
    let mut presented = false;
    let mut terminal_cpu = false;
    for line in notes.lines() {
        if line.contains("renderer=gpu adapter=") {
            seen_gpu = true;
            active = true;
            terminal_cpu = false;
        }
        if line.starts_with("# renderer=cpu") {
            terminal_cpu = !line.contains("gpu=starting");
            assert!(
                !seen_gpu || !terminal_cpu,
                "GPU resource cycle fell back to CPU after activation: {notes}"
            );
            active = false;
        }
        if active && line.starts_with("F ") {
            presented = true;
        }
    }
    (seen_gpu, active, presented, terminal_cpu)
}
fn cycle(log: &Path, gpu: bool) -> bool {
    let parent = TestParent::open(320, 240).expect("real parent opens");
    let app = App::new("Resource cycle")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), || {
            node().style(style().size(160, 100).background(Color::WHITE))
        })
        .expect("embedded App opens");
    let deadline = Instant::now() + Duration::from_secs(20);
    let actual_gpu = loop {
        parent.pump(Duration::from_millis(5));
        assert!(app.error().is_none(), "embedded failure: {:?}", app.error());
        let gpu_before_capture = if gpu {
            let notes = std::fs::read_to_string(log).unwrap_or_default();
            let (_, active, presented, _) = gpu_status(&notes);
            active && presented
        } else {
            false
        };
        let painted = parent
            .capture()
            .is_ok_and(|shot| shot.pixel(20, 20) == Some(0xffffff));
        let notes = std::fs::read_to_string(log).unwrap_or_default();
        let any_frame = notes.lines().any(|line| line.starts_with("F "));
        let (terminal, presented, actual_gpu) = if gpu {
            let (seen, active, gpu_frame, terminal_cpu) = gpu_status(&notes);
            (
                (active && gpu_frame) || (!seen && terminal_cpu),
                if seen { active && gpu_frame } else { any_frame },
                active && gpu_frame,
            )
        } else {
            (true, any_frame, notes.contains("renderer=gpu adapter="))
        };
        if terminal && painted && presented && (!gpu || !actual_gpu || gpu_before_capture) {
            break actual_gpu;
        }
        assert!(
            Instant::now() < deadline,
            "renderer never became ready: {notes}"
        );
    };
    assert!(app.error().is_none());
    if gpu && actual_gpu {
        let notes = std::fs::read_to_string(log).unwrap_or_default();
        let (_, active, presented, _) = gpu_status(&notes);
        assert!(
            active && presented,
            "GPU renderer was not active before drop: {notes}"
        );
    }
    drop(app);
    let deadline = Instant::now() + Duration::from_secs(5);
    while parent.capture().is_ok() {
        assert!(Instant::now() < deadline, "child remained after App drop");
        parent.pump(Duration::from_millis(5));
    }
    parent.close();
    assert!(parent.is_closed());
    actual_gpu
}
#[allow(
    clippy::print_stderr,
    reason = "report retained evidence paths and why a real GPU resource check was skipped"
)]
fn twenty(cpu: bool) {
    let _serial = RESOURCE_TEST.lock().expect("resource test lock");
    let warnings = embed_resources::warnings();
    let warmup_start = warnings.lock().expect("warning log").len();
    let mut evidence = Evidence::new(cpu);
    evidence.snapshot("cold", &Snapshot::read());
    eprintln!(
        "Retained embedded resource evidence: {}",
        evidence.directory.display()
    );
    let gpu = !cpu && cfg!(feature = "gpu");
    let warmup_log = evidence.directory.join("warmup.log");
    let warmed_gpu = {
        let _environment = Environment::set(cpu, &warmup_log);
        cycle(&warmup_log, gpu)
    };
    evidence.frames("warmup", &warmup_log);
    let warmup_end = warnings.lock().expect("warning log").len();
    // Native libraries may require process-lifetime retention on first use.
    // Preserve this evidence separately; ordinary cycles must add no pins.
    std::fs::write(
        evidence.directory.join("warmup-warnings.txt"),
        warnings.lock().expect("warning log")[warmup_start..warmup_end].join("\n"),
    )
    .expect("retain library warmup warnings");
    if gpu && !warmed_gpu {
        let unavailable = kivact::test::gpu_unavailable()
            .expect("GPU unexpectedly fell back despite a usable hardware adapter");
        std::fs::write(evidence.directory.join("SKIPPED.txt"), &unavailable)
            .expect("retain GPU skip reason");
        eprintln!("GPU unavailable: {unavailable}");
        return;
    }
    let baseline = Snapshot::read();
    evidence.snapshot("baseline", &baseline);
    for index in 0..20 {
        let label = format!("cycle-{index:02}");
        let before = Snapshot::read();
        evidence.snapshot(&format!("{label}-before"), &before);
        let log = evidence.directory.join(format!("{label}.log"));
        let used_gpu = {
            let _environment = Environment::set(cpu, &log);
            cycle(&log, gpu)
        };
        let after = Snapshot::read();
        evidence.snapshot(&format!("{label}-after"), &after);
        evidence.frames(&label, &log);
        assert!(!gpu || used_gpu, "GPU fell back on cycle {index}");
        assert_eq!(before, baseline, "resources changed before cycle {index}");
        assert_eq!(
            after, baseline,
            "App threads or GUI resources leaked on cycle {index}"
        );
    }
    evidence.snapshot("after", &Snapshot::read());
    let warnings = warnings.lock().expect("warning log");
    let cycles = &warnings[warmup_end..];
    assert_eq!(
        embed_resources::retention_warnings(cycles),
        0,
        "ordinary cycles add no module retention after native library warmup: {cycles:?}"
    );
    assert!(
        !cycles
            .iter()
            .any(|message| message.contains("detached") || message.contains("detaching")),
        "ordinary cycles join their workers: {cycles:?}"
    );
}
#[test]
fn twenty_cpu_embed_cycles_release_resources() {
    twenty(true);
}
#[test]
fn twenty_default_gpu_embed_cycles_release_resources() {
    twenty(false);
}
