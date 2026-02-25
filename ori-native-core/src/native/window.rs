use std::time::Duration;

use crate::{Platform, element::NativeParent};

pub trait HasWindow: Platform {
    type Window: NativeWindow<Self>;
}

pub trait NativeWindow<P>: NativeParent<P>
where
    P: Platform,
{
    fn build(platform: &mut P, contents: &P::Widget) -> Self;

    fn teardown(self, platform: &mut P);

    fn get_size(&self) -> (u32, u32);
    fn get_min_size(&self) -> (Option<u32>, Option<u32>);

    fn set_on_animation_frame(&mut self, on_frame: impl Fn(Duration) + 'static);
    fn set_on_resize(&mut self, on_resize: impl Fn() + 'static);
    fn set_on_close_requested(&mut self, on_close_requested: impl Fn() + 'static);

    fn start_animating(&mut self);
    fn stop_animating(&mut self);

    fn set_min_size(&mut self, width: u32, height: u32);
    fn set_size(&mut self, width: u32, height: u32);
    fn set_resizable(&mut self, resizable: bool);
}
