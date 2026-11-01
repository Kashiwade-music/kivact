//! Role: Music player binary.
#![warn(redundant_imports)]

use kivact::prelude::*;

fn main() -> kivact::Result<()> {
    App::new("Kivact Music Player").run(kivact_music_player::player)
}
