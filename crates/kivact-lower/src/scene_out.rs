//! Role: An append-only output scene that knows how many groups are open in it.

use kivact_types::protocol::{DrawCommand, Scene};

/// Draw commands being built in paint order. It is append-only, so it always knows how many
/// clips, transforms and layers are open at its end (a cached payload is appended under one
/// more transform only while that stays within Arvane's depth limit).
#[derive(Clone, Debug, Default)]
pub struct SceneOut {
    scene: Scene,
    open: usize,
    deepest: usize,
}

impl SceneOut {
    /// Append `command`.
    pub fn push(&mut self, command: DrawCommand) {
        use DrawCommand as D;
        match command {
            D::PushClip(_) | D::PushTransform(_) | D::PushLayer(_) => {
                self.open += 1;
                self.deepest = self.deepest.max(self.open);
            }
            // An unbalanced pop is the writer's bug; Arvane's scene validation reports it.
            D::PopClip | D::PopTransform | D::PopLayer => {
                self.open = self.open.saturating_sub(1);
            }
            _ => {}
        }
        self.scene.commands.push(command);
    }

    /// Append `commands` in order.
    pub fn extend(&mut self, commands: impl IntoIterator<Item = DrawCommand>) {
        for command in commands {
            self.push(command);
        }
    }

    /// The groups open at the end.
    pub(crate) fn open(&self) -> usize {
        self.open
    }

    /// The most groups that were open at once.
    pub(crate) fn deepest(&self) -> usize {
        self.deepest
    }

    /// The finished scene.
    pub fn into_scene(self) -> Scene {
        self.scene
    }
}
