/// When newlines should be inserted in a [`textinput`](crate::views::textinput).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Newline {
    /// Newlines are never inserted.
    Never,

    /// Newlines are inserted when `enter` is pressed.
    Enter,

    /// Newlines are inserted when `enter` is pressed while `shift` is held.
    ShiftEnter,
}

/// What to do when the submit action is input in a [`textinput`](crate::views::textinput).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Submit {
    /// Do nothing.
    Nothing,

    /// Blur the [`textinput`](crate::views::textinput).
    Blur,
}
