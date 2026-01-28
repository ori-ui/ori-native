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
        Align, App, AutoLength, Color, Effect, FlexContainer, FlexItem, Justify, Layout,
        LayoutContainer, Length, Percent, View, views::*,
    };
}
