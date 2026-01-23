use ori::Effect;
use ori_native_core::NativeApp;

use crate::platform;

pub struct App {
    native: platform::App,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            native: platform::App::new(),
        }
    }

    pub fn run<T, V>(self, data: &mut T, mut ui: impl FnMut(&T) -> V)
    where
        V: Effect<platform::Context, T> + 'static,
        V::State: 'static,
    {
        self.native.run(data, move |data| Box::new(ui(data)));
    }
}
