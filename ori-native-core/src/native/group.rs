use ori::{Elements, Mut};

use crate::{
    BoxedWidget, Color, Context, NativeWidget, Overflow, Platform, PodMut, element::NativeParent,
};

pub trait HasGroup: Platform {
    type Group: NativeGroup<Self>;
}

pub trait NativeGroup<P>: NativeWidget<P> + NativeParent<P>
where
    P: Platform,
{
    fn build(platform: &mut P) -> Self;
    fn teardown(self, platform: &mut P);

    fn insert_child(&mut self, index: usize, child: &P::Widget);

    fn remove_child(&mut self, index: usize);

    fn swap_children(&mut self, index_a: usize, index_b: usize);

    fn set_size(&mut self, width: f32, height: f32);

    fn set_child_layout(&mut self, index: usize, x: f32, y: f32, width: f32, height: f32);

    fn set_background_color(&mut self, platform: &mut P, color: Color);
    fn set_border_color(&mut self, platform: &mut P, color: Color);
    fn set_border_width(&mut self, platform: &mut P, width: [f32; 4]);
    fn set_corner_radii(&mut self, platform: &mut P, radii: [f32; 4]);
    fn set_overflow(&mut self, platform: &mut P, overflow: Overflow);
}

pub struct Group<P>
where
    P: HasGroup,
{
    group:    P::Group,
    children: Vec<BoxedWidget<P>>,
}

impl<P> Group<P>
where
    P: HasGroup,
{
    pub fn new(cx: &mut Context<P>) -> Self {
        Self {
            group:    P::Group::build(&mut cx.platform),
            children: Vec::new(),
        }
    }

    pub fn teardown(self, cx: &mut Context<P>) {
        self.group.teardown(&mut cx.platform);
    }

    pub fn elements(&mut self, node: taffy::NodeId) -> impl Elements<Context<P>, BoxedWidget<P>> {
        GroupElements {
            node,
            index: 0,
            group: &mut self.group,
            children: &mut self.children,
        }
    }

    pub fn set_background_color(&mut self, cx: &mut Context<P>, color: Color) {
        self.group.set_background_color(&mut cx.platform, color);
    }

    pub fn set_border_color(&mut self, cx: &mut Context<P>, color: Color) {
        self.group.set_border_color(&mut cx.platform, color);
    }

    pub fn set_corner_radii(&mut self, cx: &mut Context<P>, radii: [f32; 4]) {
        self.group.set_corner_radii(&mut cx.platform, radii);
    }

    pub fn set_overflow(&mut self, cx: &mut Context<P>, overflow: Overflow) {
        self.group.set_overflow(&mut cx.platform, overflow);
    }

    pub fn layout(&mut self, cx: &mut Context<P>, node: taffy::NodeId) {
        if let Ok(layout) = cx.get_computed_layout(node).cloned() {
            self.group.set_size(layout.size.width, layout.size.height);

            self.group.set_border_width(
                &mut cx.platform,
                [
                    layout.border.top,
                    layout.border.right,
                    layout.border.bottom,
                    layout.border.left,
                ],
            );
        }

        for (index, child) in self.children.iter_mut().enumerate() {
            if let Ok(layout) = cx.get_computed_layout(child.node) {
                self.group.set_child_layout(
                    index,
                    layout.location.x,
                    layout.location.y,
                    layout.size.width,
                    layout.size.height,
                );
            }
        }
    }
}

impl<P> NativeWidget<P> for Group<P>
where
    P: HasGroup,
{
    fn widget(&self) -> &P::Widget {
        self.group.widget()
    }
}

struct GroupElements<'a, P>
where
    P: HasGroup,
{
    node:     taffy::NodeId,
    index:    usize,
    group:    &'a mut P::Group,
    children: &'a mut Vec<BoxedWidget<P>>,
}

impl<P> Elements<Context<P>, BoxedWidget<P>> for GroupElements<'_, P>
where
    P: HasGroup,
{
    fn next(&mut self, _cx: &mut Context<P>) -> Option<Mut<'_, BoxedWidget<P>>> {
        let child = self.children.get_mut(self.index)?;
        let pod = PodMut {
            parent_node:   self.node,
            parent_widget: self.group,

            index:  self.index,
            node:   &mut child.node,
            widget: &mut child.widget,
        };

        self.index += 1;
        Some(pod)
    }

    fn insert(&mut self, cx: &mut Context<P>, element: BoxedWidget<P>) {
        let _ = cx.insert_layout_child(self.node, self.index, element.node);

        self.group.insert_child(self.index, element.widget.widget());
        self.children.insert(self.index, element);
        self.index += 1;
    }

    fn remove(&mut self, cx: &mut Context<P>) -> Option<BoxedWidget<P>> {
        self.group.remove_child(self.index);
        let child = self.children.remove(self.index);
        let _ = cx.remove_layout_child(self.node, self.index);

        Some(child)
    }

    fn swap(&mut self, cx: &mut Context<P>, offset: usize) {
        let _ = cx.replace_layout_child(
            self.node,
            self.index,
            self.children[self.index + offset].node,
        );

        let _ = cx.replace_layout_child(
            self.node,
            self.index + offset,
            self.children[self.index].node,
        );

        self.group.swap_children(self.index, self.index + offset);
        self.children.swap(self.index, self.index + offset);
    }
}
