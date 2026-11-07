//! Role: The output of one lowering: the commands of the innermost open
//! layer, and the layers and items the lowering added to the composition.
//!
//! Invariants:
//! - A command goes to the chunk of the innermost open layer. The groups
//!   and clips the chunk has open are kept with their payload ([`Opened`]),
//!   so a child layer or an item splits the chunk: the chunk closes with
//!   their pops, the layer or item follows, and the same groups are opened
//!   again, in the same order, in the next chunk (§6.3).
//! - A layer's pieces, in paint order, are its raster chunks, its items and
//!   its child layers. The composition numbers the layers when it is built.
//! - `space` is the transform from the open layer's space to the chunk's
//!   current space. Clips and items are kept in layer space.

use kivact_types::geometry3d::Matrix4;
use kivact_types::paint::BackfaceVisibility;
use kivact_types::render::{
    CompositeLayer, Composition, Group, LayerEffects, LayerItem, LayerKey, Plane, RasterGrid,
    RenderCmd as D, RenderScene,
};
use kivact_types::scene::Shape;
use kivact_types::{BlendMode, NodeId, Rect, Size, Transform};

/// A group or clip a chunk has open, with what opens it again.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Opened {
    /// A transform of the chunk's space.
    Transform(Transform),
    /// A rect clip, as pushed, and the same clip in layer space.
    Clip { local: Rect, layer: Shape },
    /// A group that only clips (opacity 1, normal blend, no blur), as
    /// pushed, and the same clip in layer space.
    ClipGroup { local: Shape, layer: Shape },
    /// Any other group.
    Group(Group),
}

impl Opened {
    /// The command that opens this group or clip.
    fn open(&self) -> D {
        match self {
            Opened::Transform(t) => D::PushTransform(*t),
            Opened::Clip { local, .. } => D::PushClip(*local),
            Opened::ClipGroup { local, .. } => D::PushGroup(Group {
                clip: Some(local.clone()),
                opacity: 1.0,
                blend: BlendMode::Normal,
                blur: None,
            }),
            Opened::Group(group) => D::PushGroup(group.clone()),
        }
    }

    /// The command that closes this group or clip.
    fn close(&self) -> D {
        match self {
            Opened::Transform(_) => D::PopTransform,
            Opened::Clip { .. } => D::PopClip,
            Opened::ClipGroup { .. } | Opened::Group(_) => D::PopGroup,
        }
    }
}

/// The compositing properties of one layer, as the composition holds them.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LayerProps {
    /// Stable identity of the layer.
    pub(crate) key: LayerKey,
    /// Layer space → parent layer space.
    pub(crate) transform: Matrix4,
    /// Content bounds in layer space.
    pub(crate) bounds: Rect,
    /// Flat or shared 3D context with the parent.
    pub(crate) plane: Plane,
    /// Back-face visibility.
    pub(crate) backface: BackfaceVisibility,
    /// Raster grid placement.
    pub(crate) grid: RasterGrid,
    /// Opacity, blend, filters, clip, masks and backdrop.
    pub(crate) effects: LayerEffects,
}

impl LayerProps {
    /// The root layer's properties: identity transform, no effects.
    pub(crate) fn root() -> Self {
        LayerProps {
            key: LayerKey::Root,
            transform: Matrix4::IDENTITY,
            bounds: Rect::default(),
            plane: Plane::Flat,
            backface: BackfaceVisibility::Visible,
            grid: RasterGrid::Target,
            effects: LayerEffects::default(),
        }
    }

    /// The composition layer of these properties with `items`.
    fn into_layer(self, items: Vec<LayerItem>) -> CompositeLayer {
        CompositeLayer {
            key: self.key,
            transform: self.transform,
            bounds: self.bounds,
            plane: self.plane,
            backface: self.backface,
            grid: self.grid,
            effects: self.effects,
            items,
        }
    }
}

/// One thing of a layer, in paint order.
#[derive(Debug)]
enum Piece {
    /// Vello-drawn commands.
    Raster(RenderScene),
    /// An external frame or a shader.
    Item(Box<LayerItem>),
    /// A child layer.
    Layer(Box<Layer>),
}

/// A layer under construction: its properties, the clips its parent
/// applies, its finished pieces and the open chunk.
#[derive(Debug)]
struct Layer {
    props: LayerProps,
    clips: Vec<Shape>,
    pieces: Vec<Piece>,
    chunk: RenderScene,
    stack: Vec<Opened>,
    space: Transform,
    deepest: usize,
    /// The groups of `stack` are to be opened again at the start of the
    /// next chunk (a split closed them).
    reopen: bool,
}

impl Layer {
    /// An empty open layer.
    fn new(props: LayerProps, clips: Vec<Shape>) -> Self {
        Layer {
            props,
            clips,
            pieces: Vec::new(),
            chunk: RenderScene::default(),
            stack: Vec::new(),
            space: Transform::IDENTITY,
            deepest: 0,
            reopen: false,
        }
    }

    /// Append `command` to the chunk, keeping the groups it opens or closes.
    fn push(&mut self, command: D) {
        self.reopen_pending();
        match &command {
            D::PushTransform(t) => self.open(Opened::Transform(*t)),
            D::PushClip(r) => self.open(Opened::Clip {
                local: *r,
                layer: Shape::Rect(*r).transformed(self.space),
            }),
            D::PushGroup(g) => self.open(group_opened(g, self.space)),
            D::PopTransform | D::PopClip | D::PopGroup => self.close(),
            _ => {}
        }
        self.chunk.commands.push(command);
    }

    /// Keep `opened` on the stack and move the space.
    fn open(&mut self, opened: Opened) {
        if let Opened::Transform(t) = opened {
            self.space = t.then(self.space);
        }
        self.stack.push(opened);
        self.deepest = self.deepest.max(self.stack.len());
    }

    /// Drop the innermost open group or clip (a pop without one is ignored:
    /// Arvane's scene validation reports the unbalanced command).
    fn close(&mut self) {
        if self.stack.pop().is_some() {
            self.space =
                self.stack
                    .iter()
                    .fold(Transform::IDENTITY, |space, opened| match opened {
                        Opened::Transform(t) => t.then(space),
                        _ => space,
                    });
        }
    }

    /// The clips open now, in layer space.
    fn clips(&self) -> Vec<Shape> {
        self.stack
            .iter()
            .filter_map(|opened| match opened {
                Opened::Clip { layer, .. } | Opened::ClipGroup { layer, .. } => Some(layer.clone()),
                _ => None,
            })
            .collect()
    }

    /// Close the open chunk into a piece (nothing when it is empty). When
    /// `reopen`, the groups it had open are opened again by the next command
    /// in a new chunk, which the next piece follows.
    fn split(&mut self, reopen: bool) {
        let mut chunk = std::mem::take(&mut self.chunk);
        if !chunk.commands.is_empty() {
            chunk
                .commands
                .extend(self.stack.iter().rev().map(Opened::close));
            self.pieces.push(Piece::Raster(chunk));
        }
        self.reopen = reopen && !self.stack.is_empty();
    }

    /// Open again, at the start of a new chunk, the groups a split closed.
    fn reopen_pending(&mut self) {
        if std::mem::take(&mut self.reopen) {
            self.chunk
                .commands
                .extend(self.stack.iter().map(Opened::open));
        }
    }
}

/// The group `g` as an open entry: a clip-only group keeps its clip
/// shape, in layer space through `space`.
fn group_opened(g: &Group, space: Transform) -> Opened {
    match &g.clip {
        Some(clip) if g.opacity == 1.0 && g.blend == BlendMode::Normal && g.blur.is_none() => {
            Opened::ClipGroup {
                local: clip.clone(),
                layer: clip.transformed(space),
            }
        }
        _ => Opened::Group(g.clone()),
    }
}

/// Draw commands and composition pieces being built in paint order. The
/// open layers form a stack; the root layer is never closed.
#[derive(Debug)]
pub(crate) struct SceneOut {
    layers: Vec<Layer>,
    owner: (NodeId, u8),
    ordinal: u32,
}

impl Default for SceneOut {
    fn default() -> Self {
        SceneOut {
            layers: vec![Layer::new(LayerProps::root(), Vec::new())],
            owner: (NodeId(0), 0),
            ordinal: 0,
        }
    }
}

impl SceneOut {
    /// The innermost open layer.
    fn top(&mut self) -> &mut Layer {
        match self.layers.last_mut() {
            Some(layer) => layer,
            None => unreachable!("invariant: the root layer is always open"),
        }
    }

    /// The innermost open layer, read.
    fn top_ref(&self) -> &Layer {
        match self.layers.last() {
            Some(layer) => layer,
            None => unreachable!("invariant: the root layer is always open"),
        }
    }

    /// Name the node and the draw layer (0 draw, 1 draw-over) whose child
    /// layers are numbered next.
    pub(crate) fn scope(&mut self, node: NodeId, slot: u8) {
        self.owner = (node, slot);
        self.ordinal = 0;
    }

    /// The key of the next child layer a Draw recording opens in the scope.
    pub(crate) fn next_scene_key(&mut self) -> LayerKey {
        let (node, slot) = self.owner;
        let ordinal = self.ordinal;
        self.ordinal += 1;
        LayerKey::Scene {
            node,
            slot,
            ordinal,
        }
    }

    /// Append `command` to the innermost open layer.
    pub(crate) fn push(&mut self, command: D) {
        self.top().push(command);
    }

    /// Append `commands` in order.
    pub(crate) fn extend(&mut self, commands: impl IntoIterator<Item = D>) {
        for command in commands {
            self.push(command);
        }
    }

    /// The groups and clips open in the innermost layer's chunk.
    pub(crate) fn open(&self) -> usize {
        self.top_ref().stack.len()
    }

    /// The most groups and clips that were open at once in one layer.
    pub(crate) fn deepest(&self) -> usize {
        self.top_ref().deepest
    }

    /// The space of the innermost layer's chunk (layer space → chunk space).
    pub(crate) fn space(&self) -> Transform {
        self.top_ref().space
    }

    /// The clips open now in the innermost layer, in layer space.
    pub(crate) fn clips(&self) -> Vec<Shape> {
        self.top_ref().clips()
    }

    /// Open a child layer with `props`; its transform is taken in the
    /// innermost layer's space. Its commands follow until `close_layer`.
    pub(crate) fn open_layer(&mut self, props: LayerProps) {
        let parent = self.top();
        let clips = parent.clips();
        let space = parent.space;
        parent.split(true);
        let mut props = props;
        props.transform = props.transform.then(Matrix4::from_affine(space));
        self.layers.push(Layer::new(props, clips));
    }

    /// Close the innermost child layer, which becomes a piece of its parent.
    pub(crate) fn close_layer(&mut self) {
        if self.layers.len() < 2 {
            debug_assert!(false, "invariant: only a child layer is closed");
            return;
        }
        let Some(mut child) = self.layers.pop() else {
            return;
        };
        child.split(false);
        self.top().pieces.push(Piece::Layer(Box::new(child)));
    }

    /// Add an item to the innermost layer, between its chunks.
    pub(crate) fn item(&mut self, item: LayerItem) {
        let top = self.top();
        top.split(true);
        top.pieces.push(Piece::Item(Box::new(item)));
    }

    /// `scene` (in the chunk's space) inside the groups and clips open now:
    /// the coverage or the fallback of an item.
    pub(crate) fn within_open(&self, scene: RenderScene) -> RenderScene {
        let top = self.top_ref();
        let mut commands: Vec<D> = top.stack.iter().map(Opened::open).collect();
        commands.extend(scene.commands);
        commands.extend(top.stack.iter().rev().map(Opened::close));
        RenderScene { commands }
    }

    /// The composition of the root layer of `size` logical pixels, with
    /// its child layers numbered in paint order.
    pub(crate) fn into_composition(mut self, size: Size) -> Composition {
        while self.layers.len() > 1 {
            self.close_layer();
        }
        let Some(mut root) = self.layers.pop() else {
            return Composition {
                layers: Vec::new(),
                exports: Vec::new(),
            };
        };
        root.props.bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: size.width,
            height: size.height,
        };
        root.split(false);
        let mut layers = vec![LayerProps::root().into_layer(Vec::new())];
        let items = flatten(root.pieces, &mut layers);
        layers[0] = root.props.into_layer(items);
        Composition {
            layers,
            exports: Vec::new(),
        }
    }

    /// The commands of a scene without items or child layers.
    pub(crate) fn into_scene(mut self) -> RenderScene {
        let Some(mut root) = self.layers.pop() else {
            return RenderScene::default();
        };
        root.split(false);
        let mut commands = Vec::new();
        for piece in root.pieces {
            match piece {
                Piece::Raster(scene) => commands.extend(scene.commands),
                _ => debug_assert!(false, "invariant: a plain scene has no items or layers"),
            }
        }
        RenderScene { commands }
    }
}

/// Number the layers below `pieces` into `layers` (a parent before its
/// children) and return the items of `pieces` in order.
fn flatten(pieces: Vec<Piece>, layers: &mut Vec<CompositeLayer>) -> Vec<LayerItem> {
    pieces
        .into_iter()
        .map(|piece| match piece {
            Piece::Raster(scene) => LayerItem::Raster(scene),
            Piece::Item(item) => *item,
            Piece::Layer(child) => {
                let Layer {
                    props,
                    clips,
                    pieces,
                    ..
                } = *child;
                let index = layers.len();
                layers.push(LayerProps::root().into_layer(Vec::new()));
                let items = flatten(pieces, layers);
                layers[index] = props.into_layer(items);
                LayerItem::Layer {
                    index: index as u32,
                    clips,
                }
            }
        })
        .collect()
}
