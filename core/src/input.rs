use keyboard_types::{Key, Modifiers, NamedKey};
use ori::Action;

/// An input handler.
///
/// Input handling is split into two stages, filtering [`InputFilter`] and handling
/// [`InputHandler`]. Filtering decides which events should be handled by which handler, and thus
/// which events propagate.
pub struct Input<T> {
    filter:  InputFilter,
    handler: InputHandler<T>,
}

/// A filter for [`Input`]s.
#[allow(clippy::type_complexity)]
pub struct InputFilter {
    keys: Vec<Box<dyn Fn(&Key, Modifiers, bool) -> bool>>,
}

/// A handler of [`Input`]s.
#[allow(clippy::type_complexity)]
pub struct InputHandler<T> {
    keys: Vec<Box<dyn FnMut(&mut T, Key, Modifiers, bool) -> Action>>,
}

impl<T> Default for Input<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Input<T> {
    /// Create new [`Input`].
    pub fn new() -> Self {
        Self {
            filter:  InputFilter::new(),
            handler: InputHandler::new(),
        }
    }

    /// Split `self` into an [`InputFilter`] and an [`InputHandler`].
    pub fn split(self) -> (InputFilter, InputHandler<T>) {
        (self.filter, self.handler)
    }

    /// Add a handler to a specific key.
    pub fn add_key_down<A>(
        &mut self,
        key: impl MatchKey + 'static,
        mods: Modifiers,
        mut on_key: impl FnMut(&mut T) -> A + 'static,
    ) where
        A: Into<Action>,
    {
        self.add_any_key(
            move |k, m, p| key.matches(k) && m == mods && p,
            move |data, _, _, _| on_key(data),
        );
    }

    /// Add a handler to a specific key.
    pub fn add_key_up<A>(
        &mut self,
        key: impl MatchKey + 'static,
        mods: Modifiers,
        mut on_key_up: impl FnMut(&mut T) -> A + 'static,
    ) where
        A: Into<Action>,
    {
        self.add_any_key(
            move |k, m, p| key.matches(k) && m == mods && !p,
            move |data, _, _, _| on_key_up(data),
        );
    }

    /// Add a handler any key.
    pub fn add_any_key<A>(
        &mut self,
        filter: impl Fn(&Key, Modifiers, bool) -> bool + 'static,
        mut handler: impl FnMut(&mut T, Key, Modifiers, bool) -> A + 'static,
    ) where
        A: Into<Action>,
    {
        self.filter.keys.push(Box::new(filter));
        self.handler.keys.push(Box::new(
            move |data, key, modifiers, is_press| handler(data, key, modifiers, is_press).into(),
        ));
    }
}

impl InputFilter {
    fn new() -> Self {
        Self { keys: Vec::new() }
    }

    /// Check if `self` matches a given key event.
    ///
    /// If the event matches, an [`InputMessage`] is returned that should be handled by the
    /// corresponding [`InputHandler`].
    pub fn filter_key(
        &self,
        key: Key,
        modifiers: Modifiers,
        pressed: bool,
    ) -> Option<InputMessage> {
        for (index, filter) in self.keys.iter().enumerate() {
            if filter(&key, modifiers, pressed) {
                return Some(InputMessage {
                    internal: InputMessageInternal::Key {
                        key,
                        modifiers,
                        pressed,
                        index,
                    },
                });
            }
        }

        None
    }
}

impl<T> InputHandler<T> {
    fn new() -> Self {
        Self { keys: Vec::new() }
    }

    /// Handle a given [`InputMessage`].
    pub fn handle(&mut self, data: &mut T, message: InputMessage) -> Action {
        match message.internal {
            InputMessageInternal::Key {
                key,
                modifiers,
                pressed,
                index,
            } => (self.keys[index])(data, key, modifiers, pressed),
        }
    }
}

/// Trait for types that can match a [`Key`].
pub trait MatchKey {
    /// Check if the [`Key`] matches.
    fn matches(&self, key: &Key) -> bool;
}

impl MatchKey for Key {
    fn matches(&self, key: &Key) -> bool {
        self == key
    }
}

impl MatchKey for NamedKey {
    fn matches(&self, key: &Key) -> bool {
        matches!(key, Key::Named(named) if named == self)
    }
}

impl MatchKey for &str {
    fn matches(&self, key: &Key) -> bool {
        matches!(key, Key::Character(c) if c == self)
    }
}

impl MatchKey for char {
    fn matches(&self, key: &Key) -> bool {
        if let Key::Character(c) = key
            && c.len() == self.len_utf8()
            && c.starts_with(*self)
        {
            true
        } else {
            false
        }
    }
}

/// A message produced by an [`InputFilter`] and handled by an [`InputHandler`].
#[derive(Clone, Debug)]
pub struct InputMessage {
    internal: InputMessageInternal,
}

#[derive(Clone, Debug)]
enum InputMessageInternal {
    Key {
        key:       Key,
        modifiers: Modifiers,
        pressed:   bool,
        index:     usize,
    },
}
