#![warn(missing_docs, unused_crate_dependencies, clippy::unwrap_used)]
#![allow(refining_impl_trait)]

//! Core implementation of `ori-native`.

mod context;
mod event;
mod input;
mod layout;
mod lifecycle;
mod message;
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
pub use event::{Button, MoveEvent, PressEvent, PressableEvent, ScrollEvent, TextInputEvent};
pub use input::{Input, InputFilter, InputHandler, InputMessage, MatchKey};
pub use layout::{
    Allocation, AvailableSpace, CachedMeasurable, LayoutNode, LayoutTree, Measurable,
};
pub use lifecycle::{AnimateRequest, LayoutRequest};
pub use message::RequestFocus;
pub use platform::{Platform, Unsupported};
pub use safearea::SafeAreaInsets;
pub use style::{
    Affine, Align, BorderStyle, Color, Corners, Direction, FlexStyle, Fract, Justify, LayoutStyle,
    Length, NavigationBar, Newline, Overflow, Point, PopupPosition, Position, Shadow, Side, Sides,
    Size, Sizing, StatusBar, StyleBorder, StyleCorners, StyleFlexContainer, StyleLayout,
    StylePadding, StyleShadow, Submit,
};
pub use text::{Font, Stretch, TextAlign, TextSpan, TextWrap, Weight};
pub use widget::{
    BoxedWidget, BoxedWidgetView, Parent, Widget, WidgetMut, WidgetView, WidgetViewSeq,
};

pub use keyboard_types::{Key, Modifiers, NamedKey};
