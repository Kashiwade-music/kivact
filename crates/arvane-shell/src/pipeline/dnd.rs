//! Role: Normalize file drag sessions and defer complete drop delivery.
use super::RawDrop;
use crate::ShellEvent;
use kivact_types::{FileDrop, FilePhase, Point};
use std::path::PathBuf;
#[derive(Default)]
pub enum DropSession {
    #[default]
    Idle,
    Hovering {
        files: Vec<PathBuf>,
        at: Option<Point>,
    },
    Dropped {
        files: Vec<PathBuf>,
        at: Option<Point>,
    },
}
impl DropSession {
    pub fn input(&mut self, raw: RawDrop) -> Vec<ShellEvent> {
        let mut out = Vec::new();
        match raw {
            RawDrop::Hover { files, at } => {
                // An unknown position keeps the one the drag already reached.
                let (phase, at) = match self {
                    Self::Hovering { at: last, .. } => (FilePhase::Over, at.or(*last)),
                    Self::Idle | Self::Dropped { .. } => (FilePhase::Enter, at),
                };
                if let Some(position) = at {
                    out.push(ShellEvent::File(FileDrop {
                        phase,
                        files: files.clone(),
                        position,
                    }));
                }
                *self = Self::Hovering { files, at };
            }
            RawDrop::Move(position) => match self {
                Self::Hovering { files, at } => {
                    *at = Some(position);
                    out.push(ShellEvent::File(FileDrop {
                        phase: FilePhase::Over,
                        files: files.clone(),
                        position,
                    }));
                }
                Self::Dropped { at, .. } => *at = Some(position),
                Self::Idle => {}
            },
            RawDrop::Drop => {
                *self = match std::mem::take(self) {
                    Self::Hovering { files, at } => Self::Dropped { files, at },
                    other => other,
                };
            }
            RawDrop::Leave => match std::mem::take(self) {
                Self::Hovering {
                    files,
                    at: Some(position),
                } => out.push(ShellEvent::File(FileDrop {
                    phase: FilePhase::Leave,
                    files,
                    position,
                })),
                other @ Self::Dropped { .. } => *self = other,
                _ => {}
            },
        }
        out
    }
    pub fn take_dropped(&mut self) -> Option<ShellEvent> {
        match std::mem::take(self) {
            Self::Dropped { files, at } => at.map(|position| {
                ShellEvent::File(FileDrop {
                    phase: FilePhase::Drop,
                    files,
                    position,
                })
            }),
            other => {
                *self = other;
                None
            }
        }
    }
}
