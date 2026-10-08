use std::{
    any::Any,
    mem,
    ops::{Deref, DerefMut},
    time::Duration,
};

use ori::{AnyView, Element, Sub, View, ViewSeq, views::Detachable};

use crate::{Context, LayoutNode, Platform};

/// A node in the `widget` tree.
pub trait Widget<P>: Any
where
    P: Platform,
{
    /// Get a reference to the underlying native widget.
    fn widget_ref(&self) -> P::WidgetRef;

    /// Get the layout node of this widget.
    fn layout_node(&self) -> LayoutNode;

    /// Adjust layout properties after layout has been computed.
    fn layout(&mut self, cx: &mut Context<P>);

    /// Animate the widget after an animation frame as passed.
    fn animate(&mut self, cx: &mut Context<P>, dt: Duration);
}

/// A handle for replacing a specific child of a node.
pub trait Parent<P>
where
    P: Platform,
{
    /// Replace the child with `widget` and `layout`.
    fn replace_child(&mut self, cx: &mut Context<P>, widget: P::WidgetRef, layout: LayoutNode);
}

/// A mutable handle to a widget that allows for replacement.
pub struct WidgetMut<'a, P, W>
where
    P: Platform,
{
    /// The [`Parent`] of this widget.
    pub parent: &'a mut dyn Parent<P>,

    /// The [`Widget`].
    pub widget: &'a mut W,
}

impl<'a, P, W> WidgetMut<'a, P, W>
where
    P: Platform,
{
    /// Create new [`WidgetMut`].
    pub fn new(parent: &'a mut dyn Parent<P>, widget: &'a mut W) -> Self {
        Self { parent, widget }
    }

    /// Reborrow `self`.
    pub fn reborrow(&mut self) -> WidgetMut<'_, P, W> {
        WidgetMut {
            parent: &mut *self.parent,
            widget: &mut *self.widget,
        }
    }
}

impl<'a, P, W> Deref for WidgetMut<'a, P, W>
where
    P: Platform,
{
    type Target = W;

    fn deref(&self) -> &Self::Target {
        self.widget
    }
}

impl<'a, P, W> DerefMut for WidgetMut<'a, P, W>
where
    P: Platform,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.widget
    }
}

/// Type erased [`Widget`].
pub type BoxedWidget<P> = Box<dyn Widget<P>>;

/// Type erased [`WidgetView`].
pub type BoxedWidgetView<P, T> = Box<dyn AnyView<Context<P>, T, BoxedWidget<P>>>;

impl<P> Widget<P> for BoxedWidget<P>
where
    P: Platform,
{
    fn widget_ref(&self) -> P::WidgetRef {
        self.as_ref().widget_ref()
    }

    fn layout_node(&self) -> LayoutNode {
        self.as_ref().layout_node()
    }

    fn layout(&mut self, cx: &mut Context<P>) {
        self.as_mut().layout(cx);
    }

    fn animate(&mut self, cx: &mut Context<P>, dt: Duration) {
        self.as_mut().animate(cx, dt);
    }
}

impl<P> Element for BoxedWidget<P>
where
    P: Platform,
{
    type Mut<'a> = WidgetMut<'a, P, Self>;
}

impl<P> Detachable<Context<P>> for BoxedWidget<P>
where
    P: Platform,
{
    fn as_detached<'a>(&'a mut self, _cx: &mut Context<P>) -> Self::Mut<'a> {
        struct DetachParent;

        impl<P> Parent<P> for DetachParent
        where
            P: Platform,
        {
            fn replace_child(
                &mut self,
                _cx: &mut Context<P>,
                _widget: P::WidgetRef,
                _layout: LayoutNode,
            ) {
            }
        }

        WidgetMut {
            parent: Box::leak(Box::new(DetachParent)),
            widget: self,
        }
    }
}

impl<P, T> Sub<Context<P>, T> for BoxedWidget<P>
where
    P: Platform,
    T: Widget<P> + WidgetElement<P>,
{
    fn replace(cx: &mut Context<P>, this: Self::Mut<'_>, sub: T) -> Self {
        let widget = sub.widget_ref();
        let layout = sub.layout_node();
        this.parent.replace_child(cx, widget, layout);
        mem::replace(this.widget, Box::new(sub))
    }

    fn upcast(_cx: &mut Context<P>, sub: T) -> Self {
        Box::new(sub) as _
    }

    fn downcast(this: Self) -> Result<T, Self> {
        if <dyn Any>::is::<T>(this.as_ref()) {
            let widget = Box::<dyn Any>::downcast(this)
                .expect("downcast should succeed since type was checked");

            Ok(*widget)
        } else {
            Err(this)
        }
    }

    fn downcast_mut(this: Self::Mut<'_>) -> Result<T::Mut<'_>, Self::Mut<'_>> {
        match <dyn Any>::is::<T>(this.widget.as_ref()) {
            true => {
                let widget = <dyn Any>::downcast_mut(this.widget.as_mut())
                    .expect("downcast should succeed since type was checked");

                Ok(WidgetMut::new(this.parent, widget))
            }

            false => Err(this),
        }
    }
}

pub trait WidgetElement<P>:
    Widget<P> + for<'a> Element<Mut<'a> = WidgetMut<'a, P, Self>> + Sized
where
    P: Platform,
{
}

impl<P, T> WidgetElement<P> for T
where
    P: Platform,
    T: Widget<P>,
    T: for<'a> Element<Mut<'a> = WidgetMut<'a, P, Self>>,
{
}

/// A [`View`] with a [`Widget`] as its element.
pub trait WidgetView<P, T>: View<Context<P>, T, Element: WidgetElement<P>>
where
    P: Platform,
{
}

/// A [`ViewSeq`] with [`BoxedWidget`]s as elements.
pub trait WidgetViewSeq<P, T>: ViewSeq<Context<P>, T, BoxedWidget<P>>
where
    P: Platform,
{
}

impl<P, T, V> WidgetView<P, T> for V
where
    P: Platform,
    V: View<Context<P>, T>,
    V::Element: WidgetElement<P>,
{
}

impl<P, T, V> WidgetViewSeq<P, T> for V
where
    P: Platform,
    V: ViewSeq<Context<P>, T, BoxedWidget<P>>,
{
}
