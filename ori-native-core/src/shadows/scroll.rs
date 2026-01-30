use crate::{
    Context, Direction, Pod, PodMut, Shadow,
    native::{HasScroll, NativeScroll},
};

pub struct ScrollShadow<P, S>
where
    P: HasScroll,
{
    pub scroll:   P::Scroll,
    pub contents: Pod<S>,
}

impl<P, S> ScrollShadow<P, S>
where
    P: HasScroll,
    S: Shadow<P>,
{
    pub fn new(cx: &mut Context<P>, contents: Pod<S>) -> Self {
        Self {
            scroll: P::Scroll::build(
                &mut cx.platform,
                contents.shadow.widget(),
            ),
            contents,
        }
    }

    pub fn element(&mut self, parent: taffy::NodeId) -> PodMut<'_, S> {
        self.contents.as_mut(parent)
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.scroll.set_size(width, height);
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.scroll.set_direction(direction);
    }
}

impl<P, S> Shadow<P> for ScrollShadow<P, S>
where
    P: HasScroll,
    S: Shadow<P>,
{
    fn widget(&self) -> &P::Widget {
        self.scroll.widget()
    }
}
