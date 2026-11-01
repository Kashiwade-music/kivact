// An unregistered custom type is not accepted as a property input.
use kivact::prelude::*;

struct Mood;

fn takes(_: impl IntoValue<Mood>) {}

fn main() {
    takes(Mood);
}
