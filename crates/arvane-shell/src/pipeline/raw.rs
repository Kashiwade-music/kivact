//! Role: Raw platform input vocabulary for shared normalization.
use kivact_types::{Key, KeyCode, Point};
use std::path::PathBuf;
#[derive(Clone, Debug, PartialEq)]
pub enum RawInput {
    Key(RawKey),
    Ime(RawIme),
    Drop(RawDrop),
}
#[derive(Clone, Debug, PartialEq)]
pub struct RawKey {
    pub key: PlatformKey,
    pub phase: RawKeyPhase,
}
#[derive(Clone, Debug, PartialEq)]
pub enum RawKeyPhase {
    Down {
        repeat: Option<bool>,
        text: Option<String>,
    },
    Up,
}
#[derive(Clone, Debug, PartialEq)]
pub enum PlatformKey {
    Vk {
        vk: u16,
        chr: Option<char>,
        code: KeyCode,
    },
    Keysym {
        keysym: u32,
        chr: Option<char>,
        keycode: u8,
    },
    Code {
        key: Key,
        code: KeyCode,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub enum RawIme {
    Preedit {
        text: String,
        caret: Option<(usize, usize)>,
    },
    Commit(String),
    End,
    Reconvert {
        selection: (usize, usize),
    },
}
#[derive(Clone, Debug, PartialEq)]
pub enum RawDrop {
    Hover {
        files: Vec<PathBuf>,
        at: Option<Point>,
    },
    Move(Point),
    Drop,
    Leave,
}
