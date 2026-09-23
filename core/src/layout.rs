use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    sync::Arc,
};

use ori::{Message, Proxy, ViewId};

use crate::{
    Align, BorderStyle, Direction, FlexStyle, Justify, LayoutRequest, LayoutStyle, Length,
    Overflow, Position, Sides, Size,
};

/// A leaf in the layout tree.
pub trait Measurable<P>: 'static {
    /// Compute the size for the given constraints.
    fn measure(
        &mut self,
        platform: &mut P,
        known_size: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32>;
}

impl<P> Measurable<P> for Infallible {
    fn measure(
        &mut self,
        _platform: &mut P,
        _known_size: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
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
    ) -> Size<f32> {
        for cached_size in &self.cache {
            if cached_size.known_size == known_size
                && cached_size.available_space == available_space
            {
                return cached_size.size;
            }
        }

        let size = self.inner.measure(platform, known_size, available_space);

        self.cache.push(CachedSize {
            size,
            known_size,
            available_space,
        });

        size
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
pub struct LayoutNode {
    id: taffy::NodeId,
}

/// The layout tree of an application.
pub struct LayoutTree<P> {
    proxy:     Arc<dyn Proxy>,
    tree:      taffy::TaffyTree<Box<dyn Measurable<P>>>,
    nodes:     HashSet<LayoutNode>,
    roots:     HashMap<LayoutNode, ViewId>,
    requested: HashSet<ViewId>,
}

impl<P> LayoutTree<P> {
    /// Create new [`LayoutTree`].
    pub fn new(proxy: Arc<dyn Proxy>) -> Self {
        Self {
            proxy,
            tree: taffy::TaffyTree::new(),
            nodes: HashSet::new(),
            roots: HashMap::new(),
            requested: HashSet::new(),
        }
    }

    /// Insert a `root`.
    pub fn insert_root(&mut self, node: LayoutNode, view: ViewId) {
        self.roots.insert(node, view);
    }

    /// Remove a `root`.
    pub fn remove_root(&mut self, node: LayoutNode) {
        self.roots.remove(&node);
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
    }

    /// Get the `root` node of the tree containing `node`.
    pub fn get_root(&self, node: LayoutNode) -> Option<ViewId> {
        let mut current = node.id;

        while let Some(parent) = self.tree.parent(current) {
            current = parent;
        }

        let node = LayoutNode { id: current };
        self.roots.get(&node).copied()
    }

    /// Get the computed layout of a layout node.
    pub fn get_allocation(&self, node: LayoutNode) -> Option<Allocation> {
        // NOTE: this is here because results returned by taffy mean nothing,
        //       and `layout` will panic if `node` has been removed.
        if !self.nodes.contains(&node) {
            return None;
        }

        let layout = self.tree.layout(node.id).ok()?;

        Some(Allocation {
            x: layout.location.x,
            y: layout.location.y,

            size: Size {
                width:  layout.size.width,
                height: layout.size.height,
            },

            content_size: Size {
                width:  layout.content_box_width(),
                height: layout.content_box_height(),
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
    ) where
        P: 'static,
    {
        if let Some(root) = self.roots.get(&node) {
            self.requested.remove(root);
        }

        let _ = self.tree.compute_layout_with_measure(
            node.id,
            taffy::Size {
                width:  Self::into_available_space(space.width),
                height: Self::into_available_space(space.height),
            },
            |input, _node, context, style| {
                taffy::compute_leaf_layout(
                    input,
                    style,
                    |_, _| 0.0,
                    |known_size, available_space| match context {
                        Some(leaf) => {
                            let size = leaf.measure(
                                platform,
                                Size {
                                    width:  known_size.width,
                                    height: known_size.height,
                                },
                                Size {
                                    width:  Self::from_available_space(available_space.width),
                                    height: Self::from_available_space(available_space.height),
                                },
                            );

                            taffy::Size {
                                width:  size.width,
                                height: size.height,
                            }
                        }

                        None => taffy::Size::ZERO,
                    },
                )
            },
        );
    }

    /// Create a new layout node.
    pub fn add_node(&mut self, children: &[LayoutNode]) -> LayoutNode {
        let id = self
            .tree
            .new_with_children(taffy::Style::DEFAULT, &[])
            .expect("should never fail");

        for child in children {
            let _ = self.tree.add_child(id, child.id);
        }

        let node = LayoutNode { id };

        self.nodes.insert(node);

        node
    }

    /// Create a new layout leaf.
    pub fn add_leaf<T>(&mut self, measurable: T) -> LayoutNode
    where
        T: Measurable<P> + 'static,
    {
        let id = (self.tree)
            .new_leaf_with_context(
                taffy::Style::DEFAULT,
                Box::new(measurable),
            )
            .expect("should never fail");

        let node = LayoutNode { id };

        self.nodes.insert(node);

        node
    }

    /// Insert a child at `index` in a layout node.
    pub fn insert_child(&mut self, parent: LayoutNode, index: usize, child: LayoutNode) {
        self.request_layout(parent);
        let _ = self.tree.insert_child_at_index(parent.id, index, child.id);
    }

    /// Replace the child at `index` in a layout node.
    pub fn replace_child(&mut self, parent: LayoutNode, index: usize, child: LayoutNode) {
        self.request_layout(parent);
        let _ = self.tree.replace_child_at_index(parent.id, index, child.id);
    }

    /// Replace `node` with `other`.
    pub fn replace_node(&mut self, node: LayoutNode, other: LayoutNode) {
        if let Some(parent) = self.tree.parent(node.id) {
            let children = self
                .tree
                .children(parent)
                .expect("`parent` exists so its children should too");

            let index = children
                .iter()
                .position(|child| *child == node.id)
                .expect("`node` is a child of `parent`");

            let _ = self.tree.replace_child_at_index(parent, index, other.id);
        }

        self.request_layout(other);
    }

    /// Remove a layout node.
    pub fn remove_node(&mut self, node: LayoutNode) {
        self.request_layout(node);
        let _ = self.tree.remove(node.id);
        self.nodes.remove(&node);
        self.roots.remove(&node);
    }

    /// Remove the child at `index` from a layout node.
    pub fn remove_child(&mut self, node: LayoutNode, index: usize) {
        self.request_layout(node);

        if let Ok(id) = self.tree.remove_child_at_index(node.id, index) {
            let node = LayoutNode { id };
            self.nodes.remove(&node);
        }
    }

    /// Set the size of a node without requesting layout.
    pub fn set_size_without_request(&mut self, node: LayoutNode, size: Size<Option<Length>>) {
        if let Ok(mut layout) = self.tree.style(node.id).cloned() {
            layout.size = taffy::Size {
                width:  Self::into_dimension(size.width),
                height: Self::into_dimension(size.height),
            };

            let _ = self.tree.set_style(node.id, layout);
        }
    }

    /// Set the layout style of a layout node.
    pub fn set_layout(&mut self, node: LayoutNode, style: LayoutStyle) {
        let Ok(mut layout) = self.tree.style(node.id).cloned() else {
            return;
        };

        layout.position = Self::into_position(style.position);
        layout.align_self = style.align_self.map(Self::into_align);
        layout.flex_shrink = style.flex_shrink;
        layout.flex_grow = style.flex_grow;
        layout.flex_basis = Self::into_dimension(style.flex_basis);

        layout.margin = taffy::Rect {
            top:    Self::into_length_auto(style.margin.top),
            right:  Self::into_length_auto(style.margin.right),
            bottom: Self::into_length_auto(style.margin.bottom),
            left:   Self::into_length_auto(style.margin.left),
        };

        layout.inset = taffy::Rect {
            top:    Self::into_length_auto(style.inset.top),
            right:  Self::into_length_auto(style.inset.right),
            bottom: Self::into_length_auto(style.inset.bottom),
            left:   Self::into_length_auto(style.inset.left),
        };

        layout.size = taffy::Size {
            width:  Self::into_dimension(style.size.width),
            height: Self::into_dimension(style.size.height),
        };

        layout.min_size = taffy::Size {
            width:  Self::into_length_auto(style.min_size.width),
            height: Self::into_length_auto(style.min_size.height),
        };

        layout.max_size = taffy::Size {
            width:  Self::into_length_auto(style.max_size.width),
            height: Self::into_length_auto(style.max_size.height),
        };

        self.request_layout(node);
        let _ = self.tree.set_style(node.id, layout);
    }

    /// Set the border style of a layout node.
    pub fn set_border(&mut self, node: LayoutNode, style: BorderStyle) {
        let Ok(mut layout) = self.tree.style(node.id).cloned() else {
            return;
        };

        layout.border = taffy::Rect {
            top:    Self::into_length(style.width.top),
            right:  Self::into_length(style.width.right),
            bottom: Self::into_length(style.width.bottom),
            left:   Self::into_length(style.width.left),
        };

        self.request_layout(node);
        let _ = self.tree.set_style(node.id, layout);
    }

    /// Set the padding of a layout node.
    pub fn set_padding(&mut self, node: LayoutNode, padding: Sides<Length>) {
        let Ok(mut layout) = self.tree.style(node.id).cloned() else {
            return;
        };

        layout.padding = taffy::Rect {
            top:    Self::into_length(padding.top),
            right:  Self::into_length(padding.right),
            bottom: Self::into_length(padding.bottom),
            left:   Self::into_length(padding.left),
        };

        self.request_layout(node);
        let _ = self.tree.set_style(node.id, layout);
    }

    /// Set the overflow of a layout node.
    pub fn set_overflow(&mut self, node: LayoutNode, overflow: Size<Overflow>) {
        let Ok(mut layout) = self.tree.style(node.id).cloned() else {
            return;
        };

        layout.overflow = taffy::Point {
            x: Self::into_overflow(overflow.width),
            y: Self::into_overflow(overflow.height),
        };

        self.request_layout(node);
        let _ = self.tree.set_style(node.id, layout);
    }

    /// Set the flex parameters of a layout node.
    pub fn set_flex(&mut self, node: LayoutNode, flex: FlexStyle) {
        let Ok(mut layout) = self.tree.style(node.id).cloned() else {
            return;
        };

        layout.flex_direction = match flex.direction {
            Direction::Horizontal if flex.reverse => taffy::FlexDirection::RowReverse,
            Direction::Vertical if flex.reverse => taffy::FlexDirection::ColumnReverse,

            Direction::Horizontal => taffy::FlexDirection::Row,
            Direction::Vertical => taffy::FlexDirection::Column,
        };

        layout.flex_wrap = match flex.wrap {
            true => taffy::FlexWrap::Wrap,
            false => taffy::FlexWrap::NoWrap,
        };

        layout.gap = taffy::Size {
            width:  Self::into_length(flex.gap.width),
            height: Self::into_length(flex.gap.height),
        };

        layout.justify_content = flex.justify_content.map(Self::into_justify);
        layout.align_items = flex.align_items.map(Self::into_align);

        self.request_layout(node);
        let _ = self.tree.set_style(node.id, layout);
    }

    /// Set the measure of a layout.
    pub fn set_measure<T>(&mut self, node: LayoutNode, measure: T)
    where
        T: Measurable<P> + 'static,
    {
        self.request_layout(node);
        let _ = self.tree.set_node_context(node.id, Some(Box::new(measure)));
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
