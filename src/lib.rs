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
}

pub mod prelude {
    pub use crate::{
        Action, Align, App, AutoLength, BorderLayout, BuildMarker, BuildView, Color,
        ContainerLayout, Context, Effect, Element, FlexLayout, Justify, Layout, Length, Message,
        Percent, Position, Proxy, View, views::*,
    };
}
