use std::{
    any::Any,
    sync::{Arc, Mutex},
    time::Duration,
};

use ori::{Element, Split, Teleportable, ViewId};

use crate::{
    Allocation, BoxedWidget, Context, LayoutNode, Parent, Platform, Widget, native::NativeGroup,
    widget::WidgetMut,
};

impl<P> Teleportable for Context<P>
where
    P: Platform,
{
    type Left = SplitWidget<P>;
}

impl<P, T> Split<T> for Context<P>
where
    P: Platform,
    T: Widget<P>,
    T: for<'a> Element<Mut<'a> = WidgetMut<'a, P, T>>,
{
    type Right = SplitWidget<P>;

    fn split(cx: &mut Self, widget: T) -> (Self::Left, Self::Right) {
        let boxed: BoxedWidget<P> = Box::new(widget);
        let layout = boxed.layout_node();

        let mut group = P::Group::build(&mut cx.platform);
        group.insert_child(&mut cx.platform, 0, boxed.widget_ref());

        let widget = group.widget_ref();
        let inner = SplitWidgetInner {
            group,
            boxed,
            allocation: None,
        };

        let shared = Arc::new(Mutex::new(Some(inner)));

        let left = SplitWidget {
            inner: shared.clone(),
            widget: widget.clone(),
            layout,
        };

        let right = SplitWidget {
            inner: shared,
            widget,
            layout,
        };

        (left, right)
    }

    fn with_mut<U>(
        right: &mut Self::Right,
        cx: &mut Self,
        f: impl FnOnce(&mut Self, T::Mut<'_>) -> U,
    ) -> U {
        let inner = &mut *right.inner.lock().expect("locking should not fail");
        let inner = inner.as_mut().expect("should be some until teardown");

        let mut parent = SplitWidgetParent {
            group:  &mut inner.group,
            layout: inner.boxed.layout_node(),
        };

        let contents = <dyn Any>::downcast_mut(inner.boxed.as_mut())
            .expect("split widget was created with widget of type `T`");

        let widget = WidgetMut::new(&mut parent, contents);

        f(cx, widget)
    }

    fn teardown(right: Self::Right, cx: &mut Self) -> T {
        let inner = &mut *right.inner.lock().expect("locking should not fail");
        let inner = inner.take().expect("should be some until teardown");
        inner.group.teardown(&mut cx.platform);

        *Box::<dyn Any>::downcast(inner.boxed)
            .expect("split widget was created with widget of type `T`")
    }
}

pub struct SplitWidget<P>
where
    P: Platform,
{
    inner:  Arc<Mutex<Option<SplitWidgetInner<P>>>>,
    widget: P::WidgetRef,
    layout: LayoutNode,
}

struct SplitWidgetInner<P>
where
    P: Platform,
{
    group: P::Group,
    boxed: BoxedWidget<P>,

    allocation: Option<Allocation>,
}

impl<P> Widget<P> for SplitWidget<P>
where
    P: Platform,
{
    fn widget_ref(&self) -> P::WidgetRef {
        self.widget.clone()
    }

    fn layout_node(&self) -> LayoutNode {
        self.layout
    }

    fn layout(&mut self, cx: &mut Context<P>) {
        let mut inner = self.inner.lock().expect("locking should not fail");

        if let Some(inner) = inner.as_mut() {
            if let Some(allocation) = cx.layout.get_allocation(inner.boxed.layout_node())
                && inner.allocation != Some(allocation)
            {
                inner.allocation = Some(allocation);
                inner.group.set_child_layout(
                    &mut cx.platform,
                    0,
                    0.0,
                    0.0,
                    allocation.size.width,
                    allocation.size.height,
                );
            }

            inner.boxed.layout(cx);
        }
    }

    fn animate(&mut self, cx: &mut Context<P>, dt: Duration) {
        let mut inner = self.inner.lock().expect("locking should not fail");

        if let Some(inner) = inner.as_mut() {
            inner.boxed.animate(cx, dt);
        }
    }

    fn set_root(&mut self, cx: &mut Context<P>, root: Option<ViewId>) {
        let mut inner = self.inner.lock().expect("locking should not fail");

        if let Some(inner) = inner.as_mut() {
            inner.boxed.set_root(cx, root);
        }
    }
}

impl<P> Element for SplitWidget<P>
where
    P: Platform,
{
    type Mut<'a>
        = WidgetMut<'a, P, Self>
    where
        Self: 'a;
}

struct SplitWidgetParent<'a, P>
where
    P: Platform,
{
    group:  &'a mut P::Group,
    layout: LayoutNode,
}

impl<'a, P> Parent<P> for SplitWidgetParent<'a, P>
where
    P: Platform,
{
    fn replace_child(&mut self, cx: &mut Context<P>, widget: P::WidgetRef, layout: LayoutNode) {
        self.group.replace_child(&mut cx.platform, 0, widget);
        cx.layout.replace_node(self.layout, layout);
    }
}
