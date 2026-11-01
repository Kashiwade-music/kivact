// Misuse: a blocking closure capturing a UI handle. Read the handle outside
// `blocking` and pass the value in.
use kivact::signal;

fn setup() {
    let query = signal(String::new());
    let _ = kivact::task::blocking(move || query.get_untracked().len());
}

fn main() {
    let _ = setup;
}
