//! Role: An append-only output scene that knows how many groups are open in it.

use kivact_types::render::{RenderCmd, RenderScene};

/// Draw commands being built in paint order. It is append-only, so it always knows how many
/// clips, transforms and layers are open at its end (a cached payload is appended under one
/// more transform only while that stays within Arvane's depth limit).
#[derive(Clone, Debug, Default)]
pub(crate) struct SceneOut {
    scene: RenderScene,
    open: usize,
    deepest: usize,
}

impl SceneOut {
    /// Append `command`.
    pub(crate) fn push(&mut self, command: RenderCmd) {
        use RenderCmd as D;
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
    pub(crate) fn extend(&mut self, commands: impl IntoIterator<Item = RenderCmd>) {
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
    pub(crate) fn into_scene(self) -> RenderScene {
        self.scene
    }
}
