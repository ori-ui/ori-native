use ori::AnyView;

use crate::NativeContext;

pub type BoxedView<C, T> = Box<dyn AnyView<C, T, ()>>;

pub trait NativeApp {
    type Context: NativeContext;

    fn new() -> Self;

    fn run<T>(self, data: &mut T, ui: impl FnMut(&T) -> BoxedView<Self::Context, T>);
}
