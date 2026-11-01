// Misuse: sending a UI handle to another thread (handles are !Send).
use kivact::signal;

fn setup() {
    let count = signal(1);
    std::thread::spawn(move || count.get_untracked());
}

fn main() {
    let _ = setup;
}
