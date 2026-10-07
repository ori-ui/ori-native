//! Builtin views.

pub(crate) mod animate;
pub(crate) mod button;
pub(crate) mod flex;
pub(crate) mod image;
pub(crate) mod layout;
pub(crate) mod list;
pub(crate) mod measure;
pub(crate) mod popup;
pub(crate) mod pressable;
pub(crate) mod safearea;
pub(crate) mod scroll;
pub(crate) mod spring;
pub(crate) mod text;
pub(crate) mod textinput;
pub(crate) mod transform;
pub(crate) mod transition;
pub(crate) mod window;

pub use animate::{Animate, Animation, animate};
pub use button::{Button, button};
pub use flex::{Flex, column, flex, row};
pub use image::{Image, image};
pub use layout::{Layout, on_layout};
pub use list::{List, list};
pub use measure::{Measure, measure};
pub use popup::{Popup, popup};
pub use pressable::{PressState, Pressable, pressable};
pub use safearea::{SafeArea, safe_area};
pub use scroll::{Scroll, hscroll, vscroll};
pub use spring::{Spring, spring};
pub use text::{Text, text};
pub use textinput::{TextInput, textinput};
pub use transform::{Transform, transform};
pub use transition::{
    Back, BackIn, BackInOut, Ease, Elastic, ElasticIn, Lerp, Linear, Transition, transition,
};
pub use window::{Window, WindowAttributes, WindowState, window};
