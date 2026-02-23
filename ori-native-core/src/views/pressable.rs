use ori::{Action, Message, Mut, Proxied, Proxy, View, ViewId, ViewMarker};

use crate::{
    Context, Lifecycle, NativeWidget, Pod, PodMut, WidgetView,
    native::{HasPressable, NativePressable, Press},
};

pub fn pressable<V, T>(build: impl FnMut(&T, PressState) -> V + 'static) -> Pressable<V, T> {
    Pressable::new(build)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PressState {
    pub pressed: bool,
    pub hovered: bool,
    pub focused: bool,
}

#[allow(clippy::type_complexity)]
pub struct Pressable<V, T> {
    build:    Box<dyn FnMut(&T, PressState) -> V>,
    on_press: Box<dyn FnMut(&mut T) -> Action>,
    on_hover: Box<dyn FnMut(&mut T, bool) -> Action>,
    on_focus: Box<dyn FnMut(&mut T, bool) -> Action>,
}

impl<V, T> Pressable<V, T> {
    pub fn new(build: impl FnMut(&T, PressState) -> V + 'static) -> Self {
        Self {
            build:    Box::new(build),
            on_press: Box::new(|_| Action::new()),
            on_hover: Box::new(|_, _| Action::new()),
            on_focus: Box::new(|_, _| Action::new()),
        }
    }

    pub fn on_press<A>(mut self, mut on_press: impl FnMut(&mut T) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_press = Box::new(move |data| on_press(data).into());
        self
    }

    pub fn on_hover<A>(mut self, mut on_hover: impl FnMut(&mut T, bool) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_hover = Box::new(move |data, hovered| on_hover(data, hovered).into());
        self
    }

    pub fn on_focus<A>(mut self, mut on_focus: impl FnMut(&mut T, bool) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_focus = Box::new(move |data, focused| on_focus(data, focused).into());
        self
    }
}

enum PressableMessage {
    Pressed(Press),
    Hovered(bool),
    Focused(bool),
}

impl<T, V> ViewMarker for Pressable<V, T> {}
impl<P, T, V> View<Context<P>, T> for Pressable<V, T>
where
    P: HasPressable + Proxied,
    V: WidgetView<P, T>,
{
    type Element = Pod<P::Pressable>;
    type State = (V::Widget, PressableState<P, T, V>);

    fn build(mut self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let press = PressState {
            pressed: false,
            hovered: false,
            focused: false,
        };

        let view = (self.build)(data, press);
        let (contents, state) = view.build(cx, data);

        let mut widget = P::Pressable::build(
            &mut cx.platform,
            contents.widget.widget(),
        );

        let view_id = ViewId::next();

        widget.set_on_press({
            let proxy = cx.proxy();

            move |pressed| {
                proxy.message(Message::new(
                    PressableMessage::Pressed(pressed),
                    view_id,
                ));
            }
        });

        widget.set_on_hover({
            let proxy = cx.proxy();

            move |hovered| {
                proxy.message(Message::new(
                    PressableMessage::Hovered(hovered),
                    view_id,
                ));
            }
        });

        widget.set_on_focus({
            let proxy = cx.proxy();

            move |focused| {
                proxy.message(Message::new(
                    PressableMessage::Focused(focused),
                    view_id,
                ));
            }
        });

        let pod = Pod {
            node: contents.node,
            widget,
        };

        let state = PressableState {
            press,
            view_id,
            build: self.build,
            on_press: self.on_press,
            on_hover: self.on_hover,
            on_focus: self.on_focus,
            state,
        };

        (pod, (contents.widget, state))
    }

    fn rebuild(
        mut self,
        element: Mut<'_, Self::Element>,
        (contents, state): &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        let view = (self.build)(data, state.press);
        let pod = PodMut {
            parent: element.parent,
            index:  element.index,
            node:   element.node,
            widget: contents,
        };

        view.rebuild(pod, &mut state.state, cx, data);
        state.build = self.build;
        state.on_press = self.on_press;
    }

    fn message(
        element: Mut<'_, Self::Element>,
        (contents, state): &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(Lifecycle::Layout) = message.get()
            && let Ok(layout) = cx.get_computed_layout(*element.node)
        {
            (element.widget).set_size(layout.size.width, layout.size.height);
        }

        let pod = PodMut {
            parent: element.parent,
            index:  element.index,
            node:   element.node,
            widget: contents,
        };

        if let Some(message) = message.take_targeted(state.view_id) {
            let mut action = Action::new();

            match message {
                PressableMessage::Pressed(pressed) => {
                    state.press.pressed = matches!(pressed, Press::Pressed);

                    if let Press::Released = pressed {
                        action |= (state.on_press)(data);
                    }
                }

                PressableMessage::Hovered(hovered) => {
                    state.press.hovered = hovered;
                    action |= (state.on_hover)(data, hovered);
                }

                PressableMessage::Focused(focused) => {
                    state.press.focused = focused;
                    action |= (state.on_focus)(data, focused);
                }
            }

            let view = (state.build)(data, state.press);
            view.rebuild(pod, &mut state.state, cx, data);

            action
        } else {
            V::message(pod, &mut state.state, cx, data, message)
        }
    }

    fn teardown(element: Self::Element, (contents, state): Self::State, cx: &mut Context<P>) {
        let pod = Pod {
            node:   element.node,
            widget: contents,
        };

        V::teardown(pod, state.state, cx);
        element.widget.teardown(&mut cx.platform);
    }
}

#[doc(hidden)]
#[allow(clippy::type_complexity)]
pub struct PressableState<P, T, V>
where
    P: HasPressable,
    V: WidgetView<P, T>,
{
    press:    PressState,
    view_id:  ViewId,
    build:    Box<dyn FnMut(&T, PressState) -> V>,
    on_press: Box<dyn FnMut(&mut T) -> Action>,
    on_hover: Box<dyn FnMut(&mut T, bool) -> Action>,
    on_focus: Box<dyn FnMut(&mut T, bool) -> Action>,
    state:    V::State,
}
