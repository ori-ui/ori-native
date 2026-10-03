use std::{any::Any, sync::Arc};

use ori::{Action, AnyView, Base, Message, Provider, Proxied, Proxy, Tracked, Tree};

use crate::{AnimateRequest, BoxedWidget, LayoutNode, LayoutTree, Platform, Resources};

/// The context of the [`View`](ori::View) tree.
pub struct Context<P>
where
    P: Platform,
{
    /// The [`Platform`].
    pub platform: P,

    /// The [`LayoutTree`].
    pub layout: LayoutTree<P>,

    resources: Resources,
    id_tree:   Tree,
}

impl<P> Context<P>
where
    P: Platform,
{
    /// Create a [`Context`] for a given [`Platform`].
    pub fn new(mut platform: P) -> Self {
        let proxy = Arc::new(platform.proxy());

        Self {
            platform,
            layout: LayoutTree::new(proxy),
            resources: Resources::new(),
            id_tree: Tree::new(),
        }
    }

    /// Request starting to animate.
    pub fn request_start_animating(&mut self, node: LayoutNode) {
        if let Some(root) = self.layout.get_root(node) {
            let message = Message::new(AnimateRequest::Start, root);
            self.platform.proxy().message(message);
        }
    }

    /// Request stopping animating.
    pub fn request_stop_animating(&mut self, node: LayoutNode) {
        if let Some(root) = self.layout.get_root(node) {
            let message = Message::new(AnimateRequest::Stop, root);
            self.platform.proxy().message(message);
        }
    }
}

/// Type erased [`Effect`](ori::Effect).
pub type BoxedEffect<P, T> = Box<dyn AnyView<Context<P>, T, ()>>;

impl<P> Base for Context<P>
where
    P: Platform,
{
    type Element = BoxedWidget<P>;
}

impl<P> Tracked for Context<P>
where
    P: Platform,
{
    fn tree(&mut self) -> &mut Tree {
        &mut self.id_tree
    }
}

impl<P> Proxied for Context<P>
where
    P: Platform,
{
    type Proxy = P::Proxy;

    fn proxy(&mut self) -> Self::Proxy {
        self.platform.proxy()
    }

    fn send_action(&mut self, action: Action) {
        self.platform.send_action(action);
    }
}

impl<P> Provider for Context<P>
where
    P: Platform,
{
    fn push<T: Any>(&mut self, resource: Box<T>) {
        self.resources.push(resource);
    }

    fn pop<T: Any>(&mut self) -> Option<Box<T>> {
        self.resources.pop()
    }

    fn get<T: Any>(&self) -> Option<&T> {
        self.resources.get()
    }

    fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.resources.get_mut()
    }
}
