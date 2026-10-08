use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    sync::Arc,
};

use ori::{Message, Proxy, ViewId};

use crate::{
    Align, BorderStyle, Direction, FlexStyle, Justify, LayoutRequest, LayoutStyle, Length,
    Overflow, Platform, Position, Sides, Size,
};

/// A leaf in the layout tree.
pub trait Measurable<P>: 'static {
    /// Compute the size for the given constraints.
    fn measure(
        &mut self,
        platform: &mut P,
        known_size: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> (Size<f32>, Option<f32>);
}

impl<P> Measurable<P> for Infallible {
    fn measure(
        &mut self,
        _platform: &mut P,
        _known_size: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> (Size<f32>, Option<f32>) {
        unreachable!()
    }
}

/// A [`Measurable`] that caches its measurements.
#[derive(Clone, Debug)]
pub struct CachedMeasurable<T> {
    inner: T,
    cache: Vec<CachedSize>,
}

impl<T> CachedMeasurable<T> {
    /// Create new [`CachedMeasurable`].
    pub const fn new(measurable: T) -> Self {
        Self {
            inner: measurable,
            cache: Vec::new(),
        }
    }

    /// Unwrap the inner measurable.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

#[derive(Clone, Debug)]
struct CachedSize {
    size:            Size<f32>,
    baseline:        Option<f32>,
    known_size:      Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
}

impl<P, T> Measurable<P> for CachedMeasurable<T>
where
    T: Measurable<P>,
{
    fn measure(
        &mut self,
        platform: &mut P,
        known_size: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> (Size<f32>, Option<f32>) {
        for cached_size in &self.cache {
            if cached_size.known_size == known_size
                && cached_size.available_space == available_space
            {
                return (cached_size.size, cached_size.baseline);
            }
        }

        let (size, baseline) = self.inner.measure(platform, known_size, available_space);

        self.cache.push(CachedSize {
            size,
            baseline,
            known_size,
            available_space,
        });

        (size, baseline)
    }
}

/// Available space in a given dimension.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum AvailableSpace {
    /// A specific length in logical pixels.
    Definite(f32),

    /// The minimum size of contents.
    MinContent,

    /// The maximum size of contents.
    MaxContent,
}

/// The computed size of a [`LayoutNode`].
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Allocation {
    /// The x coordinate relative to the parent.
    pub x: f32,

    /// The y coordinate relative to the parent.
    pub y: f32,

    /// The allocated size.
    pub size: Size<f32>,

    /// The size of the contents.
    pub content_size: Size<f32>,

    /// The margin around the node.
    pub margin: Sides<f32>,

    /// The border widths.
    pub border: Sides<f32>,
}

/// Id of a node in the [`LayoutTree`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LayoutNode(u64);

/// The layout tree of an application.
pub struct LayoutTree<P> {
    proxy:     Arc<dyn Proxy>,
    next:      u64,
    nodes:     HashMap<u64, Node<P>>,
    roots:     HashMap<u64, ViewId>,
    requested: HashSet<ViewId>,
}

struct Node<P> {
    measurable: Option<Box<dyn Measurable<P>>>,
    style:      taffy::Style,
    cache:      taffy::Cache,
    unrounded:  taffy::Layout,
    rounded:    taffy::Layout,
    parent:     Option<u64>,
    children:   Vec<u64>,
}

impl<P> LayoutTree<P> {
    /// Create new [`LayoutTree`].
    pub fn new(proxy: Arc<dyn Proxy>) -> Self {
        Self {
            proxy,
            next: 0,
            nodes: HashMap::new(),
            roots: HashMap::new(),
            requested: HashSet::new(),
        }
    }

    /// Insert a `root`.
    pub fn insert_root(&mut self, node: LayoutNode, view: ViewId) {
        self.roots.insert(node.0, view);
    }

    /// Remove a `root`.
    pub fn remove_root(&mut self, node: LayoutNode) {
        self.roots.remove(&node.0);
    }

    /// Request a layout.
    pub fn request_layout(&mut self, node: LayoutNode) {
        if let Some(root) = self.get_root(node)
            && self.requested.insert(root)
        {
            self.proxy.message(Message::new(
                LayoutRequest::Layout,
                root,
            ));
        }

        fn mark_dirty<P>(tree: &mut LayoutTree<P>, node: u64) {
            if let Some(node) = tree.nodes.get_mut(&node)
                && let taffy::ClearState::Cleared = node.cache.clear()
                && let Some(parent) = node.parent
            {
                mark_dirty(tree, parent);
            }
        }

        mark_dirty(self, node.0);
    }

    /// Get the `root` node of the tree containing `node`.
    pub fn get_root(&self, node: LayoutNode) -> Option<ViewId> {
        let mut current = node.0;

        while let Some(node) = self.nodes.get(&current)
            && let Some(parent) = node.parent
        {
            current = parent;
        }

        self.roots.get(&current).copied()
    }

    /// Get the computed layout of a layout node.
    pub fn get_allocation(&self, node: LayoutNode) -> Option<Allocation> {
        let node = self.nodes.get(&node.0)?;
        let layout = &node.rounded;

        Some(Allocation {
            x: layout.location.x,
            y: layout.location.y,

            size: Size {
                width:  layout.size.width,
                height: layout.size.height,
            },

            content_size: Size {
                width:  layout.scrollable_overflow_rect.right
                    - layout.scrollable_overflow_rect.left,
                height: layout.scrollable_overflow_rect.bottom
                    - layout.scrollable_overflow_rect.top,
            },

            margin: Sides {
                top:    layout.margin.top,
                right:  layout.margin.right,
                bottom: layout.margin.bottom,
                left:   layout.margin.left,
            },

            border: Sides {
                top:    layout.border.top,
                right:  layout.border.right,
                bottom: layout.border.bottom,
                left:   layout.border.left,
            },
        })
    }

    /// Compute the layout of a layout tree with `node` as its root.
    pub fn compute_layout(
        &mut self,
        platform: &mut P,
        node: LayoutNode,
        space: Size<AvailableSpace>,
        scale: f32,
    ) where
        P: Platform,
    {
        if let Some(root) = self.roots.get(&node.0) {
            self.requested.remove(root);
        }

        let available_space = taffy::Size {
            width:  Self::into_available_space(space.width),
            height: Self::into_available_space(space.height),
        };

        let mut view = LayoutView {
            scale,
            scale_inverse: 1.0 / scale,
            layout: self,
            platform,
        };

        taffy::compute_root_layout(
            &mut view,
            taffy::NodeId::new(node.0),
            available_space,
        );

        taffy::round_layout(&mut view, taffy::NodeId::new(node.0));
    }

    /// Create a new layout node.
    pub fn add_node(&mut self, children: &[LayoutNode]) -> LayoutNode {
        let node = self.next;
        self.next += 1;

        self.nodes.insert(
            node,
            Node {
                measurable: None,
                style:      Default::default(),
                cache:      Default::default(),
                unrounded:  Default::default(),
                rounded:    Default::default(),
                parent:     None,
                children:   Vec::new(),
            },
        );

        for (i, child) in children.iter().copied().enumerate() {
            self.insert_child(LayoutNode(node), i, child);
        }

        LayoutNode(node)
    }

    /// Create a new layout leaf.
    pub fn add_leaf<T>(&mut self, measurable: T) -> LayoutNode
    where
        T: Measurable<P> + 'static,
    {
        let node = self.next;
        self.next += 1;

        self.nodes.insert(
            node,
            Node {
                measurable: Some(Box::new(measurable)),
                style:      Default::default(),
                cache:      Default::default(),
                unrounded:  Default::default(),
                rounded:    Default::default(),
                parent:     None,
                children:   Vec::new(),
            },
        );

        LayoutNode(node)
    }

    /// Insert a child at `index` in a layout node.
    pub fn insert_child(&mut self, parent: LayoutNode, index: usize, child: LayoutNode) {
        self.request_layout(parent);

        if let Some(parent) = self.nodes.get_mut(&parent.0) {
            parent.children.insert(index, child.0);
        }

        if let Some(child) = self.nodes.get_mut(&child.0) {
            child.parent = Some(parent.0);
        }
    }

    /// Replace the child at `index` in a layout node.
    pub fn replace_child(&mut self, parent: LayoutNode, index: usize, child: LayoutNode) {
        self.request_layout(parent);

        if let Some(parent) = self.nodes.get_mut(&parent.0) {
            let prev = parent.children[index];
            parent.children[index] = child.0;

            if let Some(prev) = self.nodes.get_mut(&prev) {
                prev.parent = None;
            }
        }

        if let Some(child) = self.nodes.get_mut(&child.0) {
            child.parent = Some(parent.0);
        }
    }

    /// Swap the order of two children of `parent`.
    pub fn swap_children(&mut self, parent: LayoutNode, index_a: usize, index_b: usize) {
        self.request_layout(parent);

        let Some(node) = self.nodes.get_mut(&parent.0) else {
            tracing::error!(
                ?parent,
                index_a,
                index_b,
                "tried to swap children of invalid node",
            );
            return;
        };

        if index_a >= node.children.len() || index_b >= node.children.len() {
            tracing::error!(
                ?parent,
                index_a,
                index_b,
                len = node.children.len(),
                "tried to swap children with invalid indices",
            );

            return;
        }

        node.children.swap(index_a, index_b);
    }

    /// Replace `node` with `other`.
    pub fn replace_node(&mut self, node: LayoutNode, other: LayoutNode) {
        self.request_layout(node);

        if let Some(state) = self.nodes.get(&node.0)
            && let Some(parent) = state.parent
            && let Some(parent) = self.nodes.get_mut(&parent)
            && let Some(child) = parent.children.iter_mut().find(|child| **child == node.0)
        {
            *child = other.0;
        }
    }

    /// Remove a layout node.
    pub fn remove_node(&mut self, node: LayoutNode) {
        fn remove_node<T>(tree: &mut LayoutTree<T>, node: u64) {
            if let Some(node) = tree.nodes.remove(&node) {
                for child in node.children {
                    remove_node(tree, child)
                }
            }

            tree.roots.remove(&node);
        }

        self.request_layout(node);
        remove_node(self, node.0);
    }

    /// Remove the child at `index` from a layout node.
    pub fn remove_child(&mut self, node: LayoutNode, index: usize) {
        self.request_layout(node);

        if let Some(node) = self.nodes.get_mut(&node.0) {
            let child = node.children.remove(index);
            self.nodes.remove(&child);
        }
    }

    /// Set the layout style of a layout node.
    pub fn set_layout(&mut self, node: LayoutNode, style: LayoutStyle) {
        let Some(layout) = self.nodes.get_mut(&node.0) else {
            return;
        };

        layout.style.position = Self::into_position(style.position);
        layout.style.align_self = style.align_self.map(Self::into_align);
        layout.style.flex_shrink = style.flex_shrink;
        layout.style.flex_grow = style.flex_grow;
        layout.style.flex_basis = Self::into_dimension(style.flex_basis);
        layout.style.aspect_ratio = style.aspect_ratio;

        layout.style.margin = taffy::Rect {
            top:    Self::into_length_auto(style.margin.top),
            right:  Self::into_length_auto(style.margin.right),
            bottom: Self::into_length_auto(style.margin.bottom),
            left:   Self::into_length_auto(style.margin.left),
        };

        layout.style.inset = taffy::Rect {
            top:    Self::into_length_auto(style.inset.top),
            right:  Self::into_length_auto(style.inset.right),
            bottom: Self::into_length_auto(style.inset.bottom),
            left:   Self::into_length_auto(style.inset.left),
        };

        layout.style.size = taffy::Size {
            width:  Self::into_dimension(style.size.width),
            height: Self::into_dimension(style.size.height),
        };

        layout.style.min_size = taffy::Size {
            width:  Self::into_length_auto(style.min_size.width),
            height: Self::into_length_auto(style.min_size.height),
        };

        layout.style.max_size = taffy::Size {
            width:  Self::into_length_auto(style.max_size.width),
            height: Self::into_length_auto(style.max_size.height),
        };

        self.request_layout(node);
    }

    /// Set the border style of a layout node.
    pub fn set_border(&mut self, node: LayoutNode, style: BorderStyle) {
        let Some(layout) = self.nodes.get_mut(&node.0) else {
            return;
        };

        layout.style.border = taffy::Rect {
            top:    Self::into_length(style.width.top),
            right:  Self::into_length(style.width.right),
            bottom: Self::into_length(style.width.bottom),
            left:   Self::into_length(style.width.left),
        };

        self.request_layout(node);
    }

    /// Set the padding of a layout node.
    pub fn set_padding(&mut self, node: LayoutNode, padding: Sides<Length>) {
        let Some(layout) = self.nodes.get_mut(&node.0) else {
            return;
        };

        layout.style.padding = taffy::Rect {
            top:    Self::into_length(padding.top),
            right:  Self::into_length(padding.right),
            bottom: Self::into_length(padding.bottom),
            left:   Self::into_length(padding.left),
        };

        self.request_layout(node);
    }

    /// Set the overflow of a layout node.
    pub fn set_overflow(&mut self, node: LayoutNode, overflow: Size<Overflow>) {
        let Some(layout) = self.nodes.get_mut(&node.0) else {
            return;
        };

        layout.style.overflow = taffy::Point {
            x: Self::into_overflow(overflow.width),
            y: Self::into_overflow(overflow.height),
        };

        self.request_layout(node);
    }

    /// Set the flex parameters of a layout node.
    pub fn set_flex(&mut self, node: LayoutNode, flex: FlexStyle) {
        let Some(layout) = self.nodes.get_mut(&node.0) else {
            return;
        };

        layout.style.flex_direction = match flex.direction {
            Direction::Horizontal if flex.reverse => taffy::FlexDirection::RowReverse,
            Direction::Vertical if flex.reverse => taffy::FlexDirection::ColumnReverse,

            Direction::Horizontal => taffy::FlexDirection::Row,
            Direction::Vertical => taffy::FlexDirection::Column,
        };

        layout.style.flex_wrap = match flex.wrap {
            true => taffy::FlexWrap::Wrap,
            false => taffy::FlexWrap::NoWrap,
        };

        layout.style.gap = taffy::Size {
            width:  Self::into_length(flex.gap.width),
            height: Self::into_length(flex.gap.height),
        };

        layout.style.justify_content = flex.justify_content.map(Self::into_justify);
        layout.style.align_items = flex.align_items.map(Self::into_align);

        self.request_layout(node);
    }

    /// Set the measure of a layout.
    pub fn set_measure<T>(&mut self, node: LayoutNode, measure: T)
    where
        T: Measurable<P> + 'static,
    {
        self.request_layout(node);
        if let Some(node) = self.nodes.get_mut(&node.0) {
            node.measurable = Some(Box::new(measure));
        }
    }

    fn into_overflow(overflow: Overflow) -> taffy::Overflow {
        match overflow {
            Overflow::Visible => taffy::Overflow::Visible,
            Overflow::Hidden => taffy::Overflow::Hidden,
        }
    }

    fn into_available_space(space: AvailableSpace) -> taffy::AvailableSpace {
        match space {
            AvailableSpace::Definite(length) => taffy::AvailableSpace::Definite(length),
            AvailableSpace::MinContent => taffy::AvailableSpace::MinContent,
            AvailableSpace::MaxContent => taffy::AvailableSpace::MaxContent,
        }
    }

    fn from_available_space(space: taffy::AvailableSpace) -> AvailableSpace {
        match space {
            taffy::AvailableSpace::Definite(length) => AvailableSpace::Definite(length),
            taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
        }
    }

    fn into_length(length: Length) -> taffy::LengthPercentage {
        match length {
            Length::Length(x) => taffy::LengthPercentage::length(x),
            Length::Fract(x) => taffy::LengthPercentage::percent(x),
        }
    }

    fn into_dimension(length: Option<Length>) -> taffy::Dimension {
        match length {
            Some(Length::Length(x)) => taffy::Dimension::length(x),
            Some(Length::Fract(x)) => taffy::Dimension::percent(x),
            None => taffy::Dimension::auto(),
        }
    }

    fn into_length_auto(length: Option<Length>) -> taffy::LengthPercentageAuto {
        match length {
            Some(Length::Length(x)) => taffy::LengthPercentageAuto::length(x),
            Some(Length::Fract(x)) => taffy::LengthPercentageAuto::percent(x),
            None => taffy::LengthPercentageAuto::auto(),
        }
    }

    fn into_position(position: Position) -> taffy::Position {
        match position {
            Position::Relative => taffy::Position::Relative,
            Position::Absolute => taffy::Position::Absolute,
        }
    }

    fn into_align(align: Align) -> taffy::AlignItems {
        match align {
            Align::Start => taffy::AlignItems::START,
            Align::Center => taffy::AlignItems::CENTER,
            Align::End => taffy::AlignItems::END,
            Align::Baseline => taffy::AlignItems::BASELINE,
            Align::Stretch => taffy::AlignItems::STRETCH,
        }
    }

    fn into_justify(justify: Justify) -> taffy::AlignContent {
        match justify {
            Justify::Start => taffy::AlignContent::START,
            Justify::Center => taffy::AlignContent::CENTER,
            Justify::End => taffy::AlignContent::END,
            Justify::Stretch => taffy::AlignContent::STRETCH,
            Justify::SpaceBetween => taffy::AlignContent::SPACE_BETWEEN,
            Justify::SpaceEvenly => taffy::AlignContent::SPACE_EVENLY,
            Justify::SpaceAround => taffy::AlignContent::SPACE_AROUND,
        }
    }
}

struct ChildIter<'a>(std::slice::Iter<'a, u64>);

impl<'a> Iterator for ChildIter<'a> {
    type Item = taffy::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied().map(From::from)
    }
}

struct LayoutView<'a, P> {
    scale:         f32,
    scale_inverse: f32,
    platform:      &'a mut P,
    layout:        &'a mut LayoutTree<P>,
}

impl<P> taffy::TraversePartialTree for LayoutView<'_, P> {
    type ChildIter<'a>
        = ChildIter<'a>
    where
        Self: 'a;

    fn child_ids(&self, parent_node_id: taffy::NodeId) -> Self::ChildIter<'_> {
        let node = self
            .layout
            .nodes
            .get(&parent_node_id.into())
            .expect("`parent_node_id` should always be valid ");

        ChildIter(node.children.iter())
    }

    fn child_count(&self, parent_node_id: taffy::NodeId) -> usize {
        self.layout
            .nodes
            .get(&parent_node_id.into())
            .map_or(0, |node| node.children.len())
    }

    fn get_child_id(&self, parent_node_id: taffy::NodeId, child_index: usize) -> taffy::NodeId {
        let node = self
            .layout
            .nodes
            .get(&parent_node_id.into())
            .expect("`parent_node_id` should always be valid ");

        node.children
            .get(child_index)
            .copied()
            .expect("`child_index` should be valid")
            .into()
    }
}

impl<P> taffy::CacheTree for LayoutView<'_, P> {
    fn cache_get(
        &mut self,
        node_id: taffy::NodeId,
        input: &taffy::LayoutInput,
    ) -> Option<taffy::LayoutOutput> {
        let node = self.layout.nodes.get_mut(&node_id.into())?;
        node.cache.get(input)
    }

    fn cache_store(
        &mut self,
        node_id: taffy::NodeId,
        input: &taffy::LayoutInput,
        layout_output: taffy::LayoutOutput,
    ) {
        if let Some(node) = self.layout.nodes.get_mut(&node_id.into()) {
            node.cache.store(input, layout_output);
        }
    }

    fn cache_clear(&mut self, node_id: taffy::NodeId) {
        if let Some(node) = self.layout.nodes.get_mut(&node_id.into()) {
            node.cache.clear();
        }
    }
}

impl<P> taffy::LayoutPartialTree for LayoutView<'_, P>
where
    P: Platform,
{
    type CoreContainerStyle<'a>
        = &'a taffy::Style
    where
        Self: 'a;

    type CustomIdent = String;

    fn get_core_container_style(&self, node_id: taffy::NodeId) -> Self::CoreContainerStyle<'_> {
        let node = self
            .layout
            .nodes
            .get(&node_id.into())
            .expect("`node_id` should always be valid ");

        &node.style
    }

    fn set_unrounded_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        let node = self
            .layout
            .nodes
            .get_mut(&node_id.into())
            .expect("`node_id` should always be valid ");

        node.unrounded = *layout;
    }

    fn compute_child_layout(
        &mut self,
        node_id: taffy::NodeId,
        inputs: taffy::LayoutInput,
    ) -> taffy::LayoutOutput {
        if inputs.run_mode == taffy::RunMode::PerformHiddenLayout {
            return taffy::compute_hidden_layout(self, node_id);
        }

        taffy::compute_cached_layout(
            self,
            node_id,
            inputs,
            |tree, node_id, inputs| {
                let Some(node) = tree.layout.nodes.get_mut(&node_id.into()) else {
                    return taffy::LayoutOutput::HIDDEN;
                };

                if let Some(ref mut measurable) = node.measurable {
                    let mut baseline = None;
                    let mut output = taffy::compute_leaf_layout(
                        inputs,
                        &node.style,
                        |_, _| 0.0,
                        |known_size, available_space| {
                            let known_size = Size {
                                width:  known_size.width,
                                height: known_size.height,
                            };

                            let available_space = Size {
                                width:  LayoutTree::<P>::from_available_space(
                                    available_space.width,
                                ),
                                height: LayoutTree::<P>::from_available_space(
                                    available_space.height,
                                ),
                            };

                            let (measured_size, measured_baseline) = measurable.measure(
                                tree.platform,
                                known_size,
                                available_space,
                            );

                            baseline = measured_baseline;

                            taffy::Size {
                                width:  measured_size.width,
                                height: measured_size.height,
                            }
                        },
                    );

                    output.baselines.first = baseline;
                    return output;
                }

                match node.style.display {
                    taffy::Display::None => taffy::compute_hidden_layout(tree, node_id),
                    taffy::Display::Flex => taffy::compute_flexbox_layout(tree, node_id, inputs),
                    taffy::Display::Grid => taffy::compute_grid_layout(tree, node_id, inputs),
                }
            },
        )
    }
}

impl<P> taffy::LayoutFlexboxContainer for LayoutView<'_, P>
where
    P: Platform,
{
    type FlexboxContainerStyle<'a>
        = &'a taffy::Style
    where
        Self: 'a;

    type FlexboxItemStyle<'a>
        = &'a taffy::Style
    where
        Self: 'a;

    fn get_flexbox_container_style(
        &self,
        node_id: taffy::NodeId,
    ) -> Self::FlexboxContainerStyle<'_> {
        let node = self
            .layout
            .nodes
            .get(&node_id.into())
            .expect("`node_id` should always be valid ");

        &node.style
    }

    fn get_flexbox_child_style(&self, child_node_id: taffy::NodeId) -> Self::FlexboxItemStyle<'_> {
        let node = self
            .layout
            .nodes
            .get(&child_node_id.into())
            .expect("`child_node_id` should always be valid ");

        &node.style
    }
}

impl<P> taffy::LayoutGridContainer for LayoutView<'_, P>
where
    P: Platform,
{
    type GridContainerStyle<'a>
        = &'a taffy::Style
    where
        Self: 'a;

    type GridItemStyle<'a>
        = &'a taffy::Style
    where
        Self: 'a;

    fn get_grid_container_style(&self, node_id: taffy::NodeId) -> Self::GridContainerStyle<'_> {
        let node = self
            .layout
            .nodes
            .get(&node_id.into())
            .expect("`node_id` should always be valid ");

        &node.style
    }

    fn get_grid_child_style(&self, child_node_id: taffy::NodeId) -> Self::GridItemStyle<'_> {
        let node = self
            .layout
            .nodes
            .get(&child_node_id.into())
            .expect("`child_node_id` should always be valid ");

        &node.style
    }
}

impl<P> taffy::TraverseTree for LayoutView<'_, P> {}

impl<P> taffy::RoundTree for LayoutView<'_, P> {
    fn get_unrounded_layout(&self, node_id: taffy::NodeId) -> taffy::Layout {
        let node = self
            .layout
            .nodes
            .get(&node_id.into())
            .expect("`node_id` should always be valid ");

        let mut layout = node.unrounded;
        scale_taffy_layout(&mut layout, self.scale);
        layout
    }

    fn set_final_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        let node = self
            .layout
            .nodes
            .get_mut(&node_id.into())
            .expect("`node_id` should always be valid ");

        let mut layout = *layout;
        scale_taffy_layout(&mut layout, self.scale_inverse);
        node.rounded = layout;
    }
}

fn scale_taffy_layout(layout: &mut taffy::Layout, scale: f32) {
    fn scale_point(point: &mut taffy::Point<f32>, scale: f32) {
        point.x *= scale;
        point.y *= scale;
    }

    fn scale_size(size: &mut taffy::Size<f32>, scale: f32) {
        size.width *= scale;
        size.height *= scale;
    }

    fn scale_rect(rect: &mut taffy::Rect<f32>, scale: f32) {
        rect.top *= scale;
        rect.right *= scale;
        rect.bottom *= scale;
        rect.left *= scale;
    }

    scale_point(&mut layout.location, scale);
    scale_size(&mut layout.size, scale);
    scale_rect(
        &mut layout.scrollable_overflow_rect,
        scale,
    );
    scale_size(&mut layout.scrollbar_size, scale);
    scale_rect(&mut layout.border, scale);
    scale_rect(&mut layout.padding, scale);
    scale_rect(&mut layout.margin, scale);
}
