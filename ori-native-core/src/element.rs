use std::{
    any::{Any, TypeId},
    mem,
};

use ori::{Element, Is, Mut, View};

use crate::{Context, Platform};

pub struct Pod<T> {
    pub node:   taffy::NodeId,
    pub widget: T,
}

impl<T> Pod<T> {
    pub fn as_mut(&mut self, parent: taffy::NodeId, index: usize) -> PodMut<'_, T> {
        PodMut {
            parent,
            index,
            node: &mut self.node,
            widget: &mut self.widget,
        }
    }
}

pub struct PodMut<'a, T> {
    pub parent: taffy::NodeId,
    pub index:  usize,
    pub node:   &'a mut taffy::NodeId,
    pub widget: &'a mut T,
}

impl<T> PodMut<'_, T> {
    pub fn reborrow(&mut self) -> PodMut<'_, T> {
        PodMut {
            parent: self.parent,
            index:  self.index,
            node:   self.node,
            widget: self.widget,
        }
    }
}

impl<T> Element for Pod<T> {
    type Mut<'a>
        = PodMut<'a, T>
    where
        Self: 'a;
}

pub type BoxedWidget<P> = Pod<Box<dyn NativeWidget<P>>>;

pub trait WidgetView<P, T>: View<Context<P>, T, Element = Pod<Self::Widget>>
where
    P: Platform,
{
    type Widget: NativeWidget<P>;
}

impl<P, T, V, W> WidgetView<P, T> for V
where
    P: Platform,
    V: View<Context<P>, T, Element = Pod<W>>,
    W: NativeWidget<P>,
{
    type Widget = W;
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

impl<P, T> Is<Context<P>, BoxedWidget<P>> for Pod<T>
where
    P: Platform,
    T: NativeWidget<P>,
{
    fn replace(cx: &mut Context<P>, other: Mut<'_, BoxedWidget<P>>, this: Self) -> BoxedWidget<P> {
        let _ = cx.replace_layout_child(other.parent, other.index, this.node);

        cx.platform.replace(
            other.widget.widget(),
            this.widget.widget(),
        );

        let widget = mem::replace(other.widget, Box::new(this.widget));
        let node = mem::replace(other.node, this.node);

        Pod { widget, node }
    }

    fn upcast(_cx: &mut Context<P>, this: Self) -> BoxedWidget<P> {
        Pod {
            node:   this.node,
            widget: Box::new(this.widget),
        }
    }

    fn downcast(this: BoxedWidget<P>) -> Result<Self, BoxedWidget<P>> {
        if this.widget.as_ref().type_id() == TypeId::of::<T>() {
            let shadow = *Box::<dyn Any>::downcast(this.widget)
                .expect("type should be correct, as it was just checked");

            Ok(Pod {
                node:   this.node,
                widget: shadow,
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
                parent: this.parent,
                index:  this.index,
                node:   this.node,
                widget: shadow,
            })
        } else {
            Err(this)
        }
    }
}
