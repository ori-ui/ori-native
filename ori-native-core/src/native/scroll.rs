use crate::{Direction, NativeWidget, Platform, element::NativeParent};

pub trait HasScroll: Platform {
    type Scroll: NativeScroll<Self>;
}

pub trait NativeScroll<P>: NativeWidget<P> + NativeParent<P>
where
    P: Platform,
{
    fn build(platform: &mut P, contents: &P::Widget) -> Self;
    fn teardown(self, platform: &mut P);

    fn set_size(&mut self, width: f32, height: f32);

    fn set_direction(&mut self, direction: Direction);
}
