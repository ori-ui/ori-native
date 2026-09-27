use crate::Point;

/// An event regarding pointer presses.
#[derive(Clone, Debug, PartialEq)]
pub struct PressEvent {
    /// The [`Button`] that was pressed.
    pub button: Button,

    /// The position where the pointer was pressed.
    pub position: Point<f32>,
}

/// An event emitted then a pointer is moved.
#[derive(Clone, Debug, PartialEq)]
pub struct MoveEvent {
    /// The new position of the pointer.
    pub position: Point<f32>,
}

/// A pointer button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Button {
    /// The primary button, usually left click.
    Primary,

    /// The secondary button, usually right click.
    Secondary,

    /// The tertiary button, usually middle click.
    Tertiary,

    /// The back button.
    Backward,

    /// The forward button.
    Forward,

    /// A button identified by its raw code.
    Unidentified(u16),
}

/// An event that can happen to a [`pressable`](crate::views::pressable).
#[derive(Clone, Debug, PartialEq)]
pub enum PressableEvent {
    /// The pointer was pressed.
    Pressed(PressEvent),

    /// The pointer was released.
    Released(PressEvent),

    /// The press was cancelled.
    Cancelled(PressEvent),

    /// The pointer moved.
    Moved(MoveEvent),

    /// The view changed hovered state.
    Hovered(bool),

    /// The view changed focused state.
    Focused(bool),
}

/// An event that can happend to a [`textinput`](crate::views::textinput).
#[derive(Clone, Debug, PartialEq)]
pub enum TextInputEvent {
    /// The text has changed.
    Changed(String),

    /// The user has submitted the input.
    Submitted,

    /// The focus has changed.
    Focused(bool),
}
