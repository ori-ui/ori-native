//! Builtin [`Widget`](crate::Widget)s

mod animate;
mod group;
mod image;
mod layout;
mod list;
mod measure;
mod painter;
mod popup;
mod pressable;
mod scroll;
mod text;
mod textinput;
mod transform;

pub use animate::AnimateWidget;
pub use group::GroupWidget;
pub use image::ImageWidget;
pub use layout::LayoutWidget;
pub use list::ListWidget;
pub use measure::MeasureWidget;
pub use painter::PainterWidget;
pub use popup::PopupWidget;
pub use pressable::PressableWidget;
pub use scroll::ScrollWidget;
pub use text::TextWidget;
pub use textinput::TextInputWidget;
pub use transform::TransformWidget;
