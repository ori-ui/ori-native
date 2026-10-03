use crate::{Effect, Result, platform};

/// Builder of an application.
pub struct App {
    native: platform::Application,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create new [`App`].
    pub fn new() -> Self {
        Self {
            native: platform::Application::new(),
        }
    }

    /// Run the application with `data` and `ui` function.
    pub fn run<T, V>(self, data: &mut T, ui: impl FnMut(&T) -> V) -> Result<()>
    where
        V: Effect<T>,
    {
        if tokio::runtime::Handle::try_current().is_ok() {
            panic!("`App::run` cannot be called from within an async runtime.");
        }

        Self::init_log();
        self.native.run(data, ui)
    }

    /// Initialize the default log for the selected platform.
    pub fn init_log() {
        platform::Application::init_log();
    }
}
