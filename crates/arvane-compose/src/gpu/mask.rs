//! Role: Records the masks and clips of a target.
//!
//! Mask values are folded into an accumulator and multiplied into the target;
//! clips multiply the target by their coverage.

use super::pass::{CLEAR, FrameTextures, view, with_pass};
use super::pipelines::Words;
use super::state::GpuContext;
use super::targets;
use crate::ComposeError;
use crate::cache::ChunkKey;
use kivact_types::paint::{MaskComposite, MaskMode};

impl GpuContext {
    /// Multiply `target` by the combined value of `masks` (the bottom layer last).
    pub(super) fn record_mask(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: u32,
        masks: &[(ChunkKey, MaskMode, MaskComposite)],
        frame: &FrameTextures<'_>,
    ) -> Result<(), ComposeError> {
        let texture = frame.target(target)?;
        let size = (texture.width(), texture.height());
        let Some(((bottom, bottom_mode, _), upper)) = masks.split_last() else {
            return Ok(());
        };
        // The accumulator starts from the bottom mask; each layer above folds in.
        let mut acc = targets::accumulator(&self.device, size);
        self.mask_pass(
            encoder,
            &acc,
            bottom,
            *bottom_mode,
            MaskComposite::Add,
            None,
        )?;
        for (key, mode, composite) in upper.iter().rev() {
            let next = targets::accumulator(&self.device, size);
            self.mask_pass(encoder, &next, key, *mode, *composite, Some(&acc))?;
            acc = next;
        }
        let acc_view = view(&acc);
        self.apply_to(encoder, texture, &acc_view, &self.pipelines.apply_acc)
    }

    /// Multiply `target` by the alpha of the coverage chunk `coverage`.
    pub(super) fn record_clip(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: u32,
        coverage: ChunkKey,
        frame: &FrameTextures<'_>,
    ) -> Result<(), ComposeError> {
        let texture = frame.target(target)?;
        let (cover, _) = self.chunk_view(&coverage)?;
        self.apply_to(encoder, texture, &cover, &self.pipelines.apply_alpha)
    }

    /// Fold one mask chunk into `out`: the first (bottom) mask starts the accumulator (`prev` is None).
    fn mask_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        out: &wgpu::Texture,
        key: &ChunkKey,
        mode: MaskMode,
        composite: MaskComposite,
        prev: Option<&wgpu::Texture>,
    ) -> Result<(), ComposeError> {
        let (mask, _) = self.chunk_view(key)?;
        let mode_index = match mode {
            MaskMode::Alpha => 0,
            MaskMode::Luminance => 1,
        };
        let composite_index = match composite {
            MaskComposite::Add => 0,
            MaskComposite::Subtract => 1,
            MaskComposite::Intersect => 2,
            MaskComposite::Exclude => 3,
        };
        let mut words = Words::default();
        words
            .u(mode_index)
            .u(composite_index)
            .u(u32::from(prev.is_none()))
            .u(0);
        let params = super::pass::uniform(&self.device, &self.queue, words.bytes());
        let prev_view = prev.map(view);
        let prev_ref = prev_view.as_ref().unwrap_or(&self.pipelines.dummy_acc);
        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Arvane mask"),
            layout: &self.pipelines.mask_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&mask),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(prev_ref),
                },
            ],
        });
        let out_view = view(out);
        with_pass(encoder, &out_view, CLEAR, |pass| {
            pass.set_pipeline(&self.pipelines.mask_step);
            pass.set_bind_group(0, &group, &[]);
            pass.draw(0..3, 0..1);
        });
        Ok(())
    }

    /// Draw `pipeline` over all of `target`, reading `source` (a multiplication, see the pipelines).
    fn apply_to(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::Texture,
        source: &wgpu::TextureView,
        pipeline: &wgpu::RenderPipeline,
    ) -> Result<(), ComposeError> {
        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Arvane apply"),
            layout: &self.pipelines.apply_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source),
            }],
        });
        let target_view = view(target);
        with_pass(encoder, &target_view, wgpu::LoadOp::Load, |pass| {
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.draw(0..3, 0..1);
        });
        Ok(())
    }
}
