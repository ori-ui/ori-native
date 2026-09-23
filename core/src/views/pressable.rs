use keyboard_types::Modifiers;
use ori::{Action, Message, Mut, Proxied, Proxy, Tracker, View, ViewId, ViewMarker};

use crate::{
    Context, Input, InputHandler, MatchKey, Platform, PressEvent, PressableEvent, WidgetView,
    event::MoveEvent, widget::WidgetMut, widgets::PressableWidget,
};

/// [`View`] that reacts to presses and focus.
pub fn pressable<V, T>(
    build: impl FnMut(&T, PressState) -> V,
) -> Pressable<T, impl FnMut(&T, PressState) -> V> {
    Pressable::new(build)
}

/// State of a [`Pressable`] [`View`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PressState {
    /// The view is pressed.
    pub pressed: bool,

    /// The view is hovered.
    pub hovered: bool,

    /// The view is focused.
    pub focused: bool,
}

/// [`View`] that reacts to presses and focus.
pub struct Pressable<T, F> {
    build:    F,
    on_event: Vec<BoxedCallback<T>>,
    input:    Input<T>,
}

type BoxedCallback<T> = Box<dyn FnMut(&mut T, PressableEvent) -> Action>;

impl<T, F> Pressable<T, F> {
    /// Create new [`Pressable`].
    pub fn new(build: F) -> Self {
        Self {
            build,
            on_event: Vec::new(),
            input: Input::new(),
        }
    }

    /// Set the callback for all events.
    pub fn on_event<A>(
        mut self,
        mut on_event: impl FnMut(&mut T, PressableEvent) -> A + 'static,
    ) -> Self
    where
        A: Into<Action>,
    {
        self.on_event.push(Box::new(move |data, event| {
            on_event(data, event).into()
        }));

        self
    }

    /// Set the callback for when the [`View`] is pressed.
    pub fn on_press<A>(self, mut on_press: impl FnMut(&mut T, PressEvent) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_event(move |data, event| match event {
            PressableEvent::Released(event) => on_press(data, event).into(),
            _ => Action::new(),
        })
    }

    /// Set the callback for when the [`View`] is hovered.
    pub fn on_hover<A>(self, mut on_hover: impl FnMut(&mut T, bool) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_event(move |data, event| match event {
            PressableEvent::Hovered(hovered) => on_hover(data, hovered).into(),
            _ => Action::new(),
        })
    }

    /// Set the callback for when the [`View`] is focused.
    pub fn on_focus<A>(self, mut on_focus: impl FnMut(&mut T) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_event(move |data, event| match event {
            PressableEvent::Focused(focused) if focused => on_focus(data).into(),
            _ => Action::new(),
        })
    }

    /// Set the callback for when the [`View`] is blurred (unfocused).
    pub fn on_blur<A>(self, mut on_blur: impl FnMut(&mut T) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_event(move |data, event| match event {
            PressableEvent::Focused(focused) if !focused => on_blur(data).into(),
            _ => Action::new(),
        })
    }

    /// Set the callback for when the pointer is pressed down over the [`View`].
    pub fn on_down<A>(self, mut on_down: impl FnMut(&mut T, PressEvent) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_event(move |data, event| match event {
            PressableEvent::Pressed(event) => on_down(data, event).into(),
            _ => Action::new(),
        })
    }

    /// Set the callback for when the pointer is moved over the [`View`].
    pub fn on_move<A>(self, mut on_move: impl FnMut(&mut T, MoveEvent) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_event(move |data, event| match event {
            PressableEvent::Moved(event) => on_move(data, event).into(),
            _ => Action::new(),
        })
    }

    /// Set the callback for when the pointer is released from the [`View`].
    pub fn on_up<A>(self, mut on_up: impl FnMut(&mut T, PressEvent) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_event(move |data, event| match event {
            PressableEvent::Released(event) => on_up(data, event).into(),
            _ => Action::new(),
        })
    }

    /// Set a callback for when `key` is pressed.
    pub fn on_key<A>(
        mut self,
        key: impl MatchKey + 'static,
        mods: Modifiers,
        on_key: impl FnMut(&mut T) -> A + 'static,
    ) -> Self
    where
        A: Into<Action>,
    {
        self.input.add_key(key, mods, on_key);
        self
    }
}

impl<T, F> ViewMarker for Pressable<T, F> {}
impl<P, T, V, F> View<Context<P>, T> for Pressable<T, F>
where
    P: Platform,
    V: WidgetView<P, T>,
    F: FnMut(&T, PressState) -> V,
{
    type Element = PressableWidget<P, V::Element>;
    type State = PressableState<P, T, V, F>;

    fn build(mut self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let press = PressState {
            pressed: false,
            hovered: false,
            focused: false,
        };

        let view = (self.build)(data, press);
        let (contents, state) = view.build(cx, data);

        let view_id = ViewId::next();
        cx.register(view_id);

        let on_event = {
            let proxy = cx.proxy();

            move |event| {
                proxy.message(Message::new(event, view_id));
            }
        };

        let mut widget = PressableWidget::new(cx, contents, on_event);

        let (filter, handler) = self.input.split();

        let proxy = cx.proxy();

        widget.set_on_key(cx, move |key, modifiers, pressed| {
            if let Some(message) = filter.filter_key(key, modifiers, pressed) {
                proxy.message(Message::new(message, view_id));
                true
            } else {
                false
            }
        });

        let state = PressableState {
            view_id,
            state,
            press,
            build: self.build,
            on_event: self.on_event,
            handler,
        };

        (widget, state)
    }

    fn rebuild(
        mut self,
        mut element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        {
            let view = (self.build)(data, state.press);
            let (mut parent, contents) = element.contents_mut();
            let widget = WidgetMut::new(&mut parent, contents);
            view.rebuild(widget, &mut state.state, cx, data);
        }

        let (filter, handler) = self.input.split();
        let proxy = cx.proxy();

        element.set_on_key(cx, {
            let view_id = state.view_id;

            move |key, modifiers, pressed| {
                if let Some(message) = filter.filter_key(key, modifiers, pressed) {
                    proxy.message(Message::new(message, view_id));
                    true
                } else {
                    false
                }
            }
        });

        state.build = self.build;
        state.on_event = self.on_event;
        state.handler = handler;
    }

    fn message(
        mut element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(message) = message.take(state.view_id) {
            return state.handler.handle(data, message);
        }

        if let Some(event) = message.take(state.view_id) {
            let mut press = state.press;
            let mut action = Action::new();

            match event {
                PressableEvent::Pressed(_) => {
                    press.pressed = true;
                }

                PressableEvent::Released(_) | PressableEvent::Cancelled(_) => {
                    press.pressed = false;
                }

                PressableEvent::Moved(_) => {}

                PressableEvent::Hovered(hovered) => {
                    press.hovered = hovered;
                }

                PressableEvent::Focused(focused) => {
                    press.focused = focused;
                }
            }

            for on_event in &mut state.on_event {
                action |= on_event(data, event.clone());
            }

            if state.press != press {
                state.press = press;

                let (mut parent, contents) = element.contents_mut();
                let widget = WidgetMut::new(&mut parent, contents);

                let view = (state.build)(data, state.press);
                view.rebuild(widget, &mut state.state, cx, data);
            }

            return action;
        }

        let (mut parent, contents) = element.contents_mut();
        let widget = WidgetMut::new(&mut parent, contents);

        V::message(
            widget,
            &mut state.state,
            cx,
            data,
            message,
        )
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        let element = element.teardown(cx);
        V::teardown(element, state.state, cx);
        cx.unregister(state.view_id);
    }
}

#[doc(hidden)]
pub struct PressableState<P, T, V, F>
where
    P: Platform,
    V: WidgetView<P, T>,
{
    view_id:  ViewId,
    state:    V::State,
    press:    PressState,
    build:    F,
    on_event: Vec<BoxedCallback<T>>,
    handler:  InputHandler<T>,
}
