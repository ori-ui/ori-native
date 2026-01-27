use crate::{Color, Platform};

pub trait HasGroup: Platform {
    type Group: NativeGroup<Self>;
}

pub trait NativeGroup<P>
where
    P: Platform,
{
    fn widget(&self) -> &P::Widget;

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
}
