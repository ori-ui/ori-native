#![warn(missing_docs, unused_crate_dependencies, clippy::unwrap_used)]

//! Core implementation of `ori-native`.

mod context;
mod event;
mod input;
mod layout;
mod lifecycle;
mod platform;
mod safearea;
mod style;
mod teleport;
mod text;
mod widget;

pub mod native;
pub mod views;
pub mod widgets;

pub use context::{BoxedEffect, Context};
pub use event::{Button, MoveEvent, PressEvent, PressableEvent};
pub use input::{Input, InputFilter, InputHandler, InputMessage, MatchKey};
pub use layout::{
    Allocation, AvailableSpace, CachedMeasurable, LayoutNode, LayoutTree, Measurable,
};
pub use lifecycle::{AnimateRequest, LayoutRequest};
pub use platform::{Platform, Unsupported};
pub use safearea::SafeAreaInsets;
pub use style::{
    Affine, Align, Border, BorderStyle, Color, Corners, Direction, FlexContainer, FlexStyle, Fract,
    Justify, Layout, LayoutStyle, Length, NavigationBar, Newline, Overflow, Padding, Point,
    PopupPosition, Position, Shadow, Side, Sides, Size, Sizing, StatusBar,
};
pub use text::{Font, Stretch, TextAlign, TextSpan, TextWrap, Weight};
pub use widget::{BoxedWidget, Parent, Widget, WidgetMut, WidgetView, WidgetViewSeq};

pub use keyboard_types::{Key, Modifiers, NamedKey};
