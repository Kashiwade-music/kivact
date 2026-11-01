//! Role: Retained per-cycle native resource snapshots and frame timing evidence.
use super::embed_resources;
use std::{
    fs::{self, File},
    io::Write as _,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
#[derive(Debug, PartialEq)]
pub(super) struct Snapshot {
    threads: Vec<String>,
    gdi: u32,
    user: u32,
}
impl Snapshot {
    pub(super) fn read() -> Self {
        let (gdi, user) = embed_resources::gui_resources();
        Self {
            threads: embed_resources::owned_threads(),
            gdi,
            user,
        }
    }
}
pub(super) struct Evidence {
    pub(super) directory: PathBuf,
    summary: File,
}
impl Evidence {
    pub(super) fn new(cpu: bool) -> Self {
        let root = std::env::var_os("KIVACT_EMBED_RESOURCE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .ancestors()
                    .nth(3)
                    .expect("workspace root")
                    .join("target/embed-20261031/resources")
            });
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("wall clock")
            .as_nanos();
        let directory = root.join(format!(
            "{}-{}-{stamp}",
            if cpu { "cpu" } else { "gpu" },
            std::process::id()
        ));
        fs::create_dir_all(&directory).expect("create retained evidence directory");
        let mut summary =
            File::create(directory.join("summary.tsv")).expect("create timing summary");
        writeln!(summary, "cycle\tframes\tgpu_frames\twork_total_ns\tscene_total_ns\traster_total_ns\tpresent_total_ns\tpresent_max_ns\tgpu_present_total_ns\tgpu_present_max_ns\tgpu_init_ms\tgpu_configure_ms").expect("write timing header");
        Self { directory, summary }
    }
    pub(super) fn snapshot(&self, label: &str, value: &Snapshot) {
        fs::write(
            self.directory.join(format!("{label}.resources.txt")),
            format!(
                "unix_ns={}\n{value:#?}\n",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("wall clock")
                    .as_nanos()
            ),
        )
        .expect("retain raw resource snapshot");
        let names = embed_resources::all_threads();
        fs::write(
            self.directory.join(format!("{label}.threads.txt")),
            format!("{}\n", names.join("\n")),
        )
        .expect("retain raw process thread names");
    }
    pub(super) fn frames(&mut self, label: &str, log: &Path) {
        let notes = fs::read_to_string(log).expect("read retained frame log");
        let mut frames = 0u64;
        let mut gpu_frames = 0u64;
        let mut totals = [0u128; 4];
        let mut present_max = 0;
        let mut gpu_present_total = 0u128;
        let mut gpu_present_max = 0u128;
        let mut gpu = false;
        let mut init = String::new();
        let mut configure = String::new();
        let mut rows = File::create(self.directory.join(format!("{label}.frames.tsv")))
            .expect("create parsed frame evidence");
        writeln!(
            rows,
            "renderer\tend_unix_ns\twork_ns\tscene_ns\traster_ns\tpresent_ns"
        )
        .expect("write frame header");
        for line in notes.lines() {
            if line.starts_with("# ") {
                if line.contains("renderer=gpu adapter=") {
                    gpu = true;
                }
                if line.contains("renderer=cpu") {
                    gpu = false;
                }
                for field in line.split_whitespace() {
                    if let Some(value) = field.strip_prefix("init_ms=") {
                        init = value.into();
                    }
                    if let Some(value) = field.strip_prefix("configure_ms=") {
                        configure = value.into();
                    }
                }
            } else if let Some(frame) = line.strip_prefix("F ") {
                let values: Vec<u128> = frame
                    .split_whitespace()
                    .map(|field| field.parse().expect("numeric frame timing"))
                    .collect();
                assert_eq!(values.len(), 5, "frame_log.rs F format");
                frames += 1;
                gpu_frames += u64::from(gpu);
                for (total, value) in totals.iter_mut().zip(&values[1..]) {
                    *total += value;
                }
                present_max = present_max.max(values[4]);
                if gpu {
                    gpu_present_total += values[4];
                    gpu_present_max = gpu_present_max.max(values[4]);
                }
                writeln!(
                    rows,
                    "{}\t{}\t{}\t{}\t{}\t{}",
                    if gpu { "gpu" } else { "cpu" },
                    values[0],
                    values[1],
                    values[2],
                    values[3],
                    values[4]
                )
                .expect("retain parsed frame");
            }
        }
        assert!(frames > 0, "each cycle presents a real frame");
        writeln!(
            self.summary,
            "{label}\t{frames}\t{gpu_frames}\t{}\t{}\t{}\t{}\t{present_max}\t{gpu_present_total}\t{gpu_present_max}\t{init}\t{configure}",
            totals[0], totals[1], totals[2], totals[3]
        )
        .expect("retain timing summary");
        self.summary.flush().expect("flush timing summary");
    }
}
