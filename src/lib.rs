#![warn(missing_docs, unused_crate_dependencies, clippy::unwrap_used)]

//! A declarative UI framework for building native applications.
//!
//! # Examples
//!
//! ```rust
#![doc = include_str!("../examples/readme.rs")]
//! ```

mod app;
mod view;

pub use app::App;
pub use view::{BoxedView, Effect, View, ViewSeq};

pub use ori::*;
pub use ori_native_core::*;

pub use ori_native_macro::main;

#[cfg(platform = "gtk4")]
pub use ori_native_gtk4 as platform;

#[cfg(platform = "android")]
pub use ori_native_android as platform;

/// The selected [`Platform`](ori_native_core::Platform).
pub type Platform = platform::Platform;

/// The [`Context`](ori_native_core::Context) of the selected [`Platform`].
pub type Context = ori_native_core::Context<Platform>;

/// The [`Element`](ori::Element) of the selected [`Platform`].
pub type Element = <Context as ori::Base>::Element;

/// The error type of the selected [`Platform`].
pub type Error = platform::Error;

#[allow(missing_docs)]
pub type Result<T> = std::result::Result<T, Error>;

/// All builtin [`View`]s.
pub mod views {
    pub use ori::views::*;
    pub use ori_native_core::views::*;

    #[cfg(feature = "layer-shell")]
    pub use ori_native_gtk4::{ExclusiveZone, KeyboardInput, Layer, LayerShell, layer_shell};

    #[cfg(feature = "session-lock")]
    pub use ori_native_gtk4::{SessionLock, session_lock};
}

/// Commonly used imports.
pub mod prelude {
    pub use crate::{
        Action, Align, App, Border, BoxedView, BuildMarker, BuildView, Button, Color, Context,
        Corners, Direction, Effect, Element, FlexContainer, Font, Fract, Justify, Key, Layout,
        Length, Message, Modifiers, NamedKey, NavigationBar, Newline, Overflow, Padding, Position,
        PressableEvent, Proxy, SafeAreaInsets, Shadow, Side, Sides, Size, Sizing, StatusBar,
        Stretch, View, ViewId, ViewSeq, Weight, Wrap, views::*,
    };

    #[allow(unused_imports)]
    #[cfg(platform = "gtk4")]
    pub use crate::platform::views as gtk4;

    pub use tracing::{debug, error, info, trace, warn};
}
