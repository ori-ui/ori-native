use std::{marker::PhantomData, time::Duration};

use ori::{Element, ViewId};

use crate::{Context, LayoutNode, Platform, Widget, WidgetMut};

/// [`Widget`] with a callback on animate.
pub struct AnimateWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    contents:   W,
    on_animate: Box<dyn Fn(Duration)>,

    root:           Option<ViewId>,
    is_animating:   bool,
    should_animate: bool,

    marker: PhantomData<fn(P)>,
}

impl<P, W> AnimateWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    /// Create new [`AnimateWidget`].
    pub fn new(contents: W, on_animate: impl Fn(Duration) + 'static) -> Self {
        Self {
            contents,
            on_animate: Box::new(on_animate),

            root: None,
            is_animating: false,
            should_animate: false,

            marker: PhantomData,
        }
    }

    /// Set whether the widget should currently be animating.
    pub fn set_animating(&mut self, cx: &mut Context<P>, animating: bool) {
        self.should_animate = animating;

        if let Some(root) = self.root {
            self.is_animating = animating;

            match animating {
                true => cx.request_start_animating(root),
                false => cx.request_stop_animating(root),
            }
        }
    }

    /// Teardown returning contents.
    pub fn teardown(self) -> W {
        self.contents
    }

    /// Get mutable reference to contents.
    pub fn contents(&mut self) -> &mut W {
        &mut self.contents
    }
}

impl<P, W> Widget<P> for AnimateWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    fn widget_ref(&self) -> P::WidgetRef {
        self.contents.widget_ref()
    }

    fn layout_node(&self) -> LayoutNode {
        self.contents.layout_node()
    }

    fn layout(&mut self, cx: &mut Context<P>) {
        self.contents.layout(cx);
    }

    fn animate(&mut self, cx: &mut Context<P>, dt: Duration) {
        (self.on_animate)(dt);
        self.contents.animate(cx, dt);
    }

    fn set_root(&mut self, cx: &mut Context<P>, root: Option<ViewId>) {
        if root.is_none()
            && self.is_animating
            && let Some(root) = self.root
        {
            self.is_animating = false;
            cx.request_stop_animating(root);
        }

        if let Some(root) = root
            && !self.is_animating
            && self.should_animate
        {
            self.is_animating = true;
            cx.request_start_animating(root);
        }

        self.root = root;
        self.contents.set_root(cx, root);
    }
}

impl<P, W> Element for AnimateWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    type Mut<'a>
        = WidgetMut<'a, P, Self>
    where
        Self: 'a;
}
