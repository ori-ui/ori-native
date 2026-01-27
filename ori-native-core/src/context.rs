use ori::{Action, AnyView, Base, Message, Proxied, Proxy, ViewId};

use crate::{AnyShadow, Platform, views::WindowMessage};

pub trait LayoutLeaf<P> {
    fn measure(
        &mut self,
        platform: &mut P,
        known_size: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
    ) -> taffy::Size<f32>;
}

pub struct Context<P> {
    pub platform:      P,
    layout_tree:       taffy::TaffyTree<Box<dyn LayoutLeaf<P>>>,
    layout_controller: Option<ViewId>,
}

impl<P> Context<P> {
    pub fn new(platform: P) -> Self {
        Self {
            platform,
            layout_tree: taffy::TaffyTree::new(),
            layout_controller: None,
        }
    }

    pub fn new_layout_node(
        &mut self,
        style: taffy::Style,
        children: &[taffy::NodeId],
    ) -> taffy::NodeId {
        self.layout_tree
            .new_with_children(style, children)
            .expect("should never fail")
    }

    pub fn new_layout_leaf<T>(&mut self, style: taffy::Style, leaf: T) -> taffy::NodeId
    where
        T: LayoutLeaf<P> + 'static,
    {
        self.layout_tree
            .new_leaf_with_context(style, Box::new(leaf))
            .expect("should never fail")
    }

    pub fn insert_layout_child(
        &mut self,
        parent: taffy::NodeId,
        index: usize,
        child: taffy::NodeId,
    ) -> taffy::TaffyResult<()> {
        self.layout_tree.insert_child_at_index(parent, index, child)
    }

    pub fn replace_layout_child(
        &mut self,
        parent: taffy::NodeId,
        index: usize,
        child: taffy::NodeId,
    ) -> taffy::TaffyResult<()> {
        self.layout_tree
            .replace_child_at_index(parent, index, child)
            .map(|_| ())
    }

    pub fn remove_layout_node(&mut self, node: taffy::NodeId) -> taffy::TaffyResult<()> {
        self.layout_tree.remove(node).map(|_| ())
    }

    pub fn set_layout_style(
        &mut self,
        node: taffy::NodeId,
        style: taffy::Style,
    ) -> taffy::TaffyResult<()> {
        self.layout_tree.set_style(node, style)
    }

    pub fn set_layout_leaf<T>(&mut self, node: taffy::NodeId, leaf: T) -> taffy::TaffyResult<()>
    where
        T: LayoutLeaf<P> + 'static,
    {
        self.layout_tree
            .set_node_context(node, Some(Box::new(leaf)))
    }

    pub fn get_computed_layout(&self, node: taffy::NodeId) -> taffy::TaffyResult<&taffy::Layout> {
        self.layout_tree.layout(node)
    }

    pub fn compute_layout(
        &mut self,
        node: taffy::NodeId,
        available_space: taffy::Size<taffy::AvailableSpace>,
    ) -> taffy::TaffyResult<()>
    where
        P: Platform,
    {
        self.layout_tree.compute_layout_with_measure(
            node,
            available_space,
            |known_size, available_space, _node, context, _style| match context {
                Some(leaf) => leaf.measure(
                    &mut self.platform,
                    known_size,
                    available_space,
                ),

                None => taffy::Size::ZERO,
            },
        )
    }

    pub fn relayout(&mut self)
    where
        P: Proxied,
    {
        if let Some(layout_controller) = self.layout_controller.take() {
            self.platform.proxy().message(Message::new(
                WindowMessage::Relayout,
                layout_controller,
            ));
        }
    }

    pub fn with_layout_controller<T>(
        &mut self,
        window: ViewId,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let previous = self.layout_controller.replace(window);
        let output = f(self);
        self.layout_controller = previous;
        output
    }
}

pub type BoxedEffect<P, T> = Box<dyn AnyView<Context<P>, T, ()>>;

impl<P> Base for Context<P> {
    type Element = AnyShadow<P>;
}

impl<P> Proxied for Context<P>
where
    P: Proxied,
{
    type Proxy = P::Proxy;

    fn proxy(&mut self) -> Self::Proxy {
        self.platform.proxy()
    }

    fn send_action(&mut self, action: Action) {
        self.platform.send_action(action);
    }
}
