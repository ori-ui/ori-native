use std::time::Duration;

use ori::{Action, Message, Mut, Proxied, Proxy, Tracked, View, ViewId, ViewMarker};

use crate::{Context, Platform, WidgetView, widget::WidgetMut, widgets::AnimateWidget};

/// [`View`] that animates its contents.
pub fn animate<T, A>(animation: A) -> Animate<T, A> {
    Animate::new(animation)
}

/// An animated [`View`].
pub trait Animation<T> {
    /// The retained state of the animation.
    type State;

    /// The view produced by the animation.
    type View;

    /// Build the animation state, and return whether the animation should start.
    fn build(self, data: &mut T) -> (Self::State, bool);

    /// Rebuild the animation state, and return whether the animation should be running.
    fn rebuild(self, state: &mut Self::State, data: &mut T) -> bool;

    /// Update the state in response to an animation frame, and return whether the animation should
    /// continue.
    fn animate(state: &mut Self::State, data: &mut T, duration: Duration) -> bool;

    /// Build the animated [`View`].
    fn view(state: &mut Self::State, data: &T) -> Self::View;
}

/// [`View`] that animates its contents.
pub struct Animate<T, A> {
    animation: A,
    on_start:  Box<dyn FnMut(&mut T) -> Action>,
    on_end:    Box<dyn FnMut(&mut T) -> Action>,
}

impl<T, A> Animate<T, A> {
    /// Create new [`Animate`].
    pub fn new(animation: A) -> Self {
        Self {
            animation,
            on_start: Box::new(|_| Action::new()),
            on_end: Box::new(|_| Action::new()),
        }
    }

    /// Set the callback for when the animation starts.
    pub fn on_start<U>(mut self, mut on_start: impl FnMut(&mut T) -> U + 'static) -> Self
    where
        U: Into<Action>,
    {
        self.on_start = Box::new(move |data| on_start(data).into());
        self
    }

    /// Set the callback for when the animation ends.
    pub fn on_end<U>(mut self, mut on_end: impl FnMut(&mut T) -> U + 'static) -> Self
    where
        U: Into<Action>,
    {
        self.on_end = Box::new(move |data| on_end(data).into());
        self
    }
}

pub struct AnimateMessage(Duration);

type Element<A, P, T> = <<A as Animation<T>>::View as View<Context<P>, T>>::Element;
type State<A, P, T> = <<A as Animation<T>>::View as View<Context<P>, T>>::State;

impl<T, A> ViewMarker for Animate<T, A> {}
impl<P, T, A> View<Context<P>, T> for Animate<T, A>
where
    P: Platform,
    A: Animation<T>,
    A::View: WidgetView<P, T>,
{
    type Element = AnimateWidget<P, Element<A, P, T>>;
    type State = AnimateState<P, T, A>;

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let (mut anim, is_animating) = self.animation.build(data);

        let view = A::view(&mut anim, data);
        let (element, state) = view.build(cx, data);

        let view_id = ViewId::next();
        cx.register(view_id);

        let on_animate = {
            let proxy = cx.proxy();

            move |delta| {
                proxy.message(Message::new(
                    AnimateMessage(delta),
                    view_id,
                ));
            }
        };

        let mut widget = AnimateWidget::new(element, on_animate);
        widget.set_animating(cx, is_animating);

        let state = AnimateState {
            view_id,
            anim,
            state,
            is_animating,
            on_start: self.on_start,
            on_end: self.on_end,
        };

        (widget, state)
    }

    fn rebuild(
        self,
        mut element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        state.on_end = self.on_end;

        let should_animate = self.animation.rebuild(&mut state.anim, data);

        let widget = WidgetMut::new(
            element.parent,
            element.widget.contents(),
        );

        let view = A::view(&mut state.anim, data);
        view.rebuild(widget, &mut state.state, cx, data);

        if state.is_animating != should_animate {
            element.set_animating(cx, should_animate);

            match should_animate {
                true => {
                    let action = (state.on_start)(data);
                    cx.send_action(action);
                }

                false => {
                    let action = (state.on_end)(data);
                    cx.send_action(action);
                }
            }

            state.is_animating = should_animate;
        }
    }

    fn message(
        mut element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(AnimateMessage(delta)) = message.take(state.view_id)
            && state.is_animating
        {
            let should_animate = A::animate(&mut state.anim, data, delta);
            let view = A::view(&mut state.anim, data);

            let widget = WidgetMut::new(
                element.parent,
                element.widget.contents(),
            );

            view.rebuild(widget, &mut state.state, cx, data);

            let mut action = Action::new();

            if state.is_animating != should_animate {
                element.set_animating(cx, should_animate);

                match should_animate {
                    true => action = (state.on_start)(data),
                    false => action = (state.on_end)(data),
                }

                state.is_animating = should_animate;
            }

            return action;
        }

        let widget = WidgetMut::new(
            element.parent,
            element.widget.contents(),
        );

        A::View::message(
            widget,
            &mut state.state,
            cx,
            data,
            message,
        )
    }

    fn teardown(mut element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        if state.is_animating {
            element.set_animating(cx, false);
        }

        let contents = element.teardown();
        A::View::teardown(contents, state.state, cx);
        cx.unregister(state.view_id);
    }
}

pub struct AnimateState<P, T, A>
where
    P: Platform,
    A: Animation<T>,
    A::View: WidgetView<P, T>,
{
    view_id: ViewId,
    anim:    A::State,
    state:   State<A, P, T>,

    is_animating: bool,

    on_start: Box<dyn FnMut(&mut T) -> Action>,
    on_end:   Box<dyn FnMut(&mut T) -> Action>,
}
