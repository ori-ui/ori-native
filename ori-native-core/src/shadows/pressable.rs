use crate::{
    Context, Shadow,
    native::{HasPressable, NativePressable},
};

pub struct PressableShadow<P, S>
where
    P: HasPressable,
{
    pub pressable: P::Pressable,
    pub contents:  S,
}

impl<P, S> PressableShadow<P, S>
where
    P: HasPressable,
    S: Shadow<P>,
{
    pub fn new(cx: &mut Context<P>, contents: S) -> Self {
        Self {
            pressable: P::Pressable::build(&mut cx.platform, contents.widget()),
            contents,
        }
    }

    pub fn set_on_press(&mut self, on_press: impl Fn(bool) + 'static) {
        self.pressable.set_on_click(on_press);
    }

    pub fn set_on_focus(&mut self, on_focus: impl Fn(bool) + 'static) {
        self.pressable.set_on_focus(on_focus);
    }
}

impl<P, S> Shadow<P> for PressableShadow<P, S>
where
    P: HasPressable,
    S: Shadow<P>,
{
    fn widget(&self) -> &P::Widget {
        self.pressable.widget()
    }

    fn layout(&mut self, cx: &mut Context<P>, node: taffy::NodeId) {
        if let Ok(layout) = cx.get_computed_layout(node) {
            (self.pressable).set_size(layout.size.width, layout.size.height);
        }

        self.contents.layout(cx, node);
    }
}
