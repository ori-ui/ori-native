use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    mem,
};

use ori::{Element, Is, Mut, View};

use crate::{Context, Platform};

pub struct Pod<P, T> {
    pub node:   taffy::NodeId,
    pub widget: T,

    marker: PhantomData<fn(&P)>,
}

impl<P, T> Pod<P, T> {
    pub fn new(node: taffy::NodeId, widget: T) -> Self {
        Self {
            node,
            widget,
            marker: PhantomData,
        }
    }
}

impl<P, T> Pod<P, T> {
    pub fn as_mut<'a>(
        &'a mut self,
        parent_node: taffy::NodeId,
        parent_widget: &'a mut dyn NativeParent<P>,
        index: usize,
    ) -> PodMut<'a, P, T> {
        PodMut {
            parent_node,
            parent_widget,
            index,
            node: &mut self.node,
            widget: &mut self.widget,
        }
    }
}

pub struct PodMut<'a, P, T> {
    pub parent_node:   taffy::NodeId,
    pub parent_widget: &'a mut dyn NativeParent<P>,

    pub index:  usize,
    pub node:   &'a mut taffy::NodeId,
    pub widget: &'a mut T,
}

impl<P, T> PodMut<'_, P, T> {
    pub fn reborrow(&mut self) -> PodMut<'_, P, T> {
        PodMut {
            parent_node:   self.parent_node,
            parent_widget: self.parent_widget,
            index:         self.index,
            node:          self.node,
            widget:        self.widget,
        }
    }
}

impl<P, T> Element for Pod<P, T> {
    type Mut<'a>
        = PodMut<'a, P, T>
    where
        Self: 'a;
}

pub type BoxedWidget<P> = Pod<P, Box<dyn NativeWidget<P>>>;

pub trait WidgetView<P, T>: View<Context<P>, T, Element = Pod<P, Self::Widget>>
where
    P: Platform,
{
    type Widget: NativeWidget<P>;
}

impl<P, T, V, W> WidgetView<P, T> for V
where
    P: Platform,
    V: View<Context<P>, T, Element = Pod<P, W>>,
    W: NativeWidget<P>,
{
    type Widget = W;
}

pub trait NativeParent<P>
where
    P: Platform,
{
    fn replace_child(&mut self, platform: &mut P, index: usize, child: &P::Widget);
}

pub trait NativeWidget<P>: Any
where
    P: Platform,
{
    fn widget(&self) -> &P::Widget;
}

impl<P> NativeWidget<P> for Box<dyn NativeWidget<P>>
where
    P: Platform,
{
    fn widget(&self) -> &P::Widget {
        self.as_ref().widget()
    }
}

impl<P, T> Is<Context<P>, BoxedWidget<P>> for Pod<P, T>
where
    P: Platform,
    T: NativeWidget<P>,
{
    fn replace(cx: &mut Context<P>, other: Mut<'_, BoxedWidget<P>>, this: Self) -> BoxedWidget<P> {
        let _ = cx.replace_layout_child(
            other.parent_node,
            other.index,
            this.node,
        );

        other.parent_widget.replace_child(
            &mut cx.platform,
            other.index,
            this.widget.widget(),
        );

        let widget = mem::replace(other.widget, Box::new(this.widget));
        let node = mem::replace(other.node, this.node);

        Pod {
            widget,
            node,
            marker: PhantomData,
        }
    }

    fn upcast(_cx: &mut Context<P>, this: Self) -> BoxedWidget<P> {
        Pod {
            node:   this.node,
            widget: Box::new(this.widget),
            marker: PhantomData,
        }
    }

    fn downcast(this: BoxedWidget<P>) -> Result<Self, BoxedWidget<P>> {
        if this.widget.as_ref().type_id() == TypeId::of::<T>() {
            let shadow = *Box::<dyn Any>::downcast(this.widget)
                .expect("type should be correct, as it was just checked");

            Ok(Pod {
                node:   this.node,
                widget: shadow,
                marker: PhantomData,
            })
        } else {
            Err(this)
        }
    }

    fn downcast_mut(
        this: Mut<'_, BoxedWidget<P>>,
    ) -> Result<Self::Mut<'_>, Mut<'_, BoxedWidget<P>>> {
        if this.widget.as_ref().type_id() == TypeId::of::<T>() {
            let shadow = <dyn Any>::downcast_mut(this.widget.as_mut())
                .expect("type should be correct, as it was just checked");

            Ok(PodMut {
                parent_node:   this.parent_node,
                parent_widget: this.parent_widget,

                index:  this.index,
                node:   this.node,
                widget: shadow,
            })
        } else {
            Err(this)
        }
    }
}
