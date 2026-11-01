//! Role: `workbench` binary: opens the 100,000-record workbench.
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact_workbench::{TITLE, generate_records, workbench};

fn main() -> kivact::Result<()> {
    App::new(TITLE).run(|| workbench(generate_records(100_000)))
}
