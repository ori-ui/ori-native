use ori::{Elements, Mut};

use crate::{
    AnyShadow, Color, Context, PodMut, Shadow,
    native::{HasGroup, NativeGroup},
};

pub struct GroupShadow<P>
where
    P: HasGroup,
{
    group:    P::Group,
    children: Vec<AnyShadow<P>>,
}

impl<P> GroupShadow<P>
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

    pub fn elements(&mut self, node: taffy::NodeId) -> impl Elements<Context<P>, AnyShadow<P>> {
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
}

impl<P> Shadow<P> for GroupShadow<P>
where
    P: HasGroup,
{
    fn widget(&self) -> &P::Widget {
        self.group.widget()
    }

    fn layout(&mut self, cx: &mut Context<P>, node: taffy::NodeId) {
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

            child.shadow.layout(cx, child.node);
        }
    }
}

struct GroupElements<'a, P>
where
    P: HasGroup,
{
    node:     taffy::NodeId,
    index:    usize,
    group:    &'a mut P::Group,
    children: &'a mut Vec<AnyShadow<P>>,
}

impl<P> Elements<Context<P>, AnyShadow<P>> for GroupElements<'_, P>
where
    P: HasGroup,
{
    fn next(&mut self, _cx: &mut Context<P>) -> Option<Mut<'_, AnyShadow<P>>> {
        let child = self.children.get_mut(self.index)?;
        self.index += 1;

        let pod = PodMut {
            parent: self.node,
            node:   &mut child.node,
            shadow: &mut child.shadow,
        };

        Some(pod)
    }

    fn insert(&mut self, cx: &mut Context<P>, element: AnyShadow<P>) {
        let _ = cx.insert_layout_child(self.node, self.index, element.node);

        self.group.insert_child(self.index, element.shadow.widget());
        self.children.insert(self.index, element);
        self.index += 1;
    }

    fn remove(&mut self, cx: &mut Context<P>) -> Option<AnyShadow<P>> {
        self.group.remove_child(self.index);
        let child = self.children.remove(self.index);
        let _ = cx.remove_layout_node(child.node);

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
