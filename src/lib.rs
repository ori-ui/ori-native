mod app;

pub use app::App;
pub use ori::*;

use ori_native_gtk4 as platform;

pub type Platform = platform::Platform;
pub type Context = ori_native_core::Context<Platform>;
pub type Element = <Context as ori::Base>::Element;

pub trait Effect<T>: ori::Effect<Context, T> {}

impl<T, V> Effect<T> for V where V: ori::Effect<Context, T> {}
