use std::{
    any::{self, Any, TypeId},
    sync::Arc,
};

use ori::{Action, AnyView, Base, Message, Provider, Proxied, Proxy, Tracked, Tree};

use crate::{AnimateRequest, BoxedWidget, LayoutNode, LayoutTree, Platform};

/// The context of the [`View`](ori::View) tree.
pub struct Context<P>
where
    P: Platform,
{
    /// The [`Platform`].
    pub platform: P,

    /// The [`LayoutTree`].
    pub layout: LayoutTree<P>,

    resources: Vec<Resource>,

    view_id_tree: Tree,
}

#[allow(dead_code)]
struct Resource {
    type_id:   TypeId,
    type_name: &'static str,
    value:     Box<dyn Any>,
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
            resources: Vec::new(),
            view_id_tree: Tree::new(),
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
        &mut self.view_id_tree
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

impl Resource {
    fn is<T: Any>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }

    unsafe fn downcast_unchecked<T: Any>(self) -> Box<T> {
        let ptr: *mut T = Box::into_raw(self.value).cast();
        unsafe { Box::from_raw(ptr) }
    }

    unsafe fn downcast_ref_unchecked<T: Any>(&self) -> &T {
        let ptr = self.value.as_ref() as *const _ as *const T;
        unsafe { &*ptr }
    }

    unsafe fn downcast_mut_unchecked<T: Any>(&mut self) -> &mut T {
        let ptr = self.value.as_mut() as *mut _ as *mut T;
        unsafe { &mut *ptr }
    }
}

impl<P> Provider for Context<P>
where
    P: Platform,
{
    fn push<T: Any>(&mut self, resource: Box<T>) {
        self.resources.push(Resource {
            type_id:   TypeId::of::<T>(),
            type_name: any::type_name::<T>(),
            value:     resource,
        });
    }

    fn pop<T: Any>(&mut self) -> Option<Box<T>> {
        let i = self.resources.iter().rposition(|r| r.is::<T>())?;

        let resource = self.resources.remove(i);
        let resource = unsafe { resource.downcast_unchecked::<T>() };
        Some(resource)
    }

    fn get<T: Any>(&self) -> Option<&T> {
        let resource = self.resources.iter().rev().find(|r| r.is::<T>())?;
        let resource = unsafe { resource.downcast_ref_unchecked::<T>() };
        Some(resource)
    }

    fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let resource = self.resources.iter_mut().rev().find(|r| r.is::<T>())?;
        let resource = unsafe { resource.downcast_mut_unchecked::<T>() };
        Some(resource)
    }
}
