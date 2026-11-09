//! Role: The Kivact DevTools app binary.

use kivact::prelude::*;

fn main() -> kivact::Result<()> {
    let args = match kivact_devtools::args::Args::parse(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(text) => return Err(kivact::UiError::msg(text)),
    };
    App::new("Kivact DevTools")
        .window_size(1280, 800)
        .fonts(kivact_devtools::fonts::fonts().iter().copied())
        .blocking_threads(6)
        .run(move || kivact_devtools::app::root(args))
}
