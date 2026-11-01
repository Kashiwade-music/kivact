//! Role: Mini Postman binary: the default history file and HTTP through curl.
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact_example_mini_postman::{curl, history_path, postman};

fn main() -> kivact::Result<()> {
    App::new("Kivact · Mini Postman").run(move || postman(history_path(), curl))
}
