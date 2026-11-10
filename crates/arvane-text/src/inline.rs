//! Role: Shape one paragraph around caller-measured in-flow objects.
use crate::text_engine::{TextEngine, crlf_as_one_break, crlf_spans};
use crate::{TextError, build, metrics, output, transform};
use kivact_types::ComputedTextStyle;
use kivact_types::render::{InlineBox, InlineBoxPlacement, RichText, ShapedInlineText, ShapedText};
use kivact_types::{Rect, TextAlign};
use parley::{Alignment, AlignmentOptions, InlineBoxKind, PositionedLayoutItem};
use std::collections::HashSet;

impl TextEngine {
    /// Shape text and measured inline objects together, using the same font
    /// shaping and line breaking as a paragraph. Each object's bottom edge
    /// aligns with its line's baseline. Objects at the same offset retain
    /// their insertion order before bidirectional reordering.
    ///
    /// This returns drawing geometry. The objects have no text offsets of
    /// their own and the returned text does not expose editing carets.
    /// Ellipsis and line clamping with objects are unsupported.
    pub fn shape_inline(
        &mut self,
        text: RichText<'_>,
        style: &ComputedTextStyle,
        max_width: Option<f64>,
        boxes: &[InlineBox],
    ) -> Result<ShapedInlineText, TextError> {
        if boxes.is_empty() {
            return Ok(ShapedInlineText {
                text: self.shape_glyphs(text, style, max_width)?,
                boxes: Vec::new(),
            });
        }
        let height = metrics::line_height(style, max_width)?;
        if !build::valid_spans(text.text, text.spans) {
            return Err(TextError::InvalidSpans);
        }
        if style.ellipsizes() {
            return Err(TextError::UnsupportedInlineStyle);
        }
        let boundaries = transform::boundaries(text.text, style.transform);
        let boxes = inline_boxes(text.text, boxes, boundaries.as_deref())?;
        let (converted, spans) = transform::apply(text.text, text.spans, style.transform);
        let spans = crlf_spans(&converted, spans);
        let converted = crlf_as_one_break(&converted);
        let mut layout = build::shape_inline_layout(
            &mut self.fonts,
            &mut self.layouts,
            &converted,
            &spans,
            style,
            height,
            &boxes,
        );
        layout.break_all_lines(max_width.map(|value| value as f32));
        layout.align(
            match style.align {
                TextAlign::Start => Alignment::Start,
                TextAlign::Center => Alignment::Center,
                TextAlign::End => Alignment::End,
                TextAlign::Justify => Alignment::Justify,
            },
            AlignmentOptions::default(),
        );
        let plan = output::plan(&layout, style, max_width, None);
        let drawn = output::draw(&layout, style, text.spans, &plan, None);
        if drawn.runs.is_empty()
            && text
                .text
                .chars()
                .any(|character| !character.is_whitespace())
        {
            return Err(TextError::MissingFont);
        }
        let lines = output::lines(&layout, &plan, style.align == TextAlign::Justify, |range| {
            if let Some(pairs) = &boundaries {
                transform::original(pairs, range.start)..transform::original(pairs, range.end)
            } else {
                range
            }
        });
        let placements = placements(&layout, boxes.len());
        Ok(ShapedInlineText {
            text: kivact_types::ShapedTextSnapshot::new(ShapedText {
                size: plan.size,
                baseline: plan.baseline,
                runs: drawn.runs,
                decorations: drawn.decorations,
                highlights: drawn.highlights,
                lines,
                ..ShapedText::default()
            }),
            boxes: placements,
        })
    }
}

/// Validate object identities and remap their original byte offsets.
fn inline_boxes(
    text: &str,
    boxes: &[InlineBox],
    boundaries: Option<&[(usize, usize)]>,
) -> Result<Vec<parley::InlineBox>, TextError> {
    let mut identities = HashSet::new();
    boxes
        .iter()
        .map(|item| {
            if !text.is_char_boundary(item.offset)
                || !item.size.width.is_finite()
                || item.size.width < 0.0
                || !item.size.height.is_finite()
                || item.size.height < 0.0
                || !identities.insert(item.id)
            {
                return Err(TextError::InvalidInlineBoxes);
            }
            let index = match boundaries {
                Some(pairs) => pairs
                    .binary_search_by_key(&item.offset, |&(_, original)| original)
                    .map(|at| pairs[at].0)
                    .map_err(|_| TextError::InvalidInlineBoxes)?,
                None => item.offset,
            };
            Ok(parley::InlineBox {
                id: item.id,
                kind: InlineBoxKind::InFlow,
                index,
                width: item.size.width as f32,
                height: item.size.height as f32,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

/// Collect visual inline-object rectangles after bidirectional layout.
fn placements(layout: &parley::Layout<u32>, capacity: usize) -> Vec<InlineBoxPlacement> {
    let mut placements = Vec::with_capacity(capacity);
    for (line, shaped) in layout.lines().enumerate() {
        for item in shaped.items() {
            if let PositionedLayoutItem::InlineBox(item) = item {
                placements.push(InlineBoxPlacement {
                    id: item.id,
                    line,
                    rect: Rect {
                        x: f64::from(item.x),
                        y: f64::from(item.y),
                        width: f64::from(item.width),
                        height: f64::from(item.height),
                    },
                });
            }
        }
    }
    placements
}
