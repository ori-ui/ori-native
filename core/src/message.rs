use ori::{Action, ViewId};

/// Message that tells a [`View`](ori::View) to request focus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestFocus;

/// Message that tells the application to quit.
pub struct Quit;

/// An extension trait for [`Action`].
pub trait NativeAction: Sized {
    #[doc(hidden)]
    fn as_action_mut(&mut self) -> &mut Action;

    /// Request that a focusable `view` become focusable.
    fn focus(view: ViewId) -> Action {
        Action::message(RequestFocus, view)
    }

    /// Request that a focusable `view` become focusable.
    fn add_focus(&mut self, view: ViewId) {
        self.as_action_mut().merge(Self::focus(view));
    }

    /// Request that a focusable `view` become focusable.
    fn with_focus(mut self, view: ViewId) -> Self {
        self.add_focus(view);
        self
    }

    /// Quit the application.
    fn quit() -> Action {
        Action::message(Quit, None)
    }

    /// Quit the application.
    fn add_quit(&mut self) {
        self.as_action_mut().merge(Self::quit());
    }

    /// Quit the application.
    fn with_quit(mut self) -> Self {
        self.add_quit();
        self
    }
}

impl NativeAction for Action {
    fn as_action_mut(&mut self) -> &mut Action {
        self
    }
}
