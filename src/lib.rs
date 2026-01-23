mod app;

pub use app::App;

use ori::Sub;
use ori_native_wry as platform;

type Element = <platform::Context as ori::BaseElement>::Element;

pub trait View<T>:
    ori::View<platform::Context, T, Element: Sub<platform::Context, Element>>
{
}

impl<T, V> View<T> for V where
    V: ori::View<platform::Context, T, Element: Sub<platform::Context, Element>>
{
}

pub trait Effect<T>: ori::Effect<platform::Context, T> {}

impl<T, V> Effect<T> for V where V: ori::Effect<platform::Context, T> {}
