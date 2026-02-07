mod app;
mod view;

pub use app::App;
pub use view::{Effect, View};

pub use ori::*;
pub use ori_native_core::*;

use ori_native_gtk4 as platform;

pub type Platform = platform::Platform;
pub type Context = ori_native_core::Context<Platform>;
pub type Element = <Context as ori::Base>::Element;

pub mod views {
    pub use ori::views::*;
    pub use ori_native_core::views::*;

    #[cfg(feature = "layer-shell")]
    pub use ori_native_gtk4::{ExclusiveZone, KeyboardInput, Layer, LayerShell, layer_shell};

    #[cfg(feature = "session-lock")]
    pub use ori_native_gtk4::{SessionLock, session_lock};
}

pub mod prelude {
    pub use crate::{
        Action, Align, App, AutoLength, BorderLayout, BuildMarker, BuildView, Color,
        ContainerLayout, Context, Effect, Element, FlexLayout, Fraction, Justify, Layout, Length,
        Message, Position, Proxy, View, views::*,
    };

    #[allow(unused_imports)]
    pub use crate::platform::views::*;
}
