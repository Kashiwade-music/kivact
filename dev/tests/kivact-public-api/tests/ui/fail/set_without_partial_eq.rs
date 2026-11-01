// Misuse: `Signal::set` needs `PartialEq` (equal writes are cut off). Use
// `update` or `notify` for values without it.
use kivact::signal;

struct Player;

fn setup() {
    let p = signal(Player);
    p.set(Player);
}

fn main() {
    let _ = setup;
}
