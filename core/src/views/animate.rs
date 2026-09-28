use std::time::Duration;

use ori::{Action, Message, Mut, Proxied, Proxy, Tracked, View, ViewId, ViewMarker};

use crate::{Context, Platform, Widget, WidgetView, widget::WidgetMut, widgets::AnimateWidget};

/// [`View`] that animates its contents.
pub fn animate<P, T, A>(animation: A) -> impl WidgetView<P, T>
where
    P: Platform,
    A: Animation<T>,
    A::View: WidgetView<P, T>,
{
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
pub struct Animate<A> {
    animation: A,
}

impl<A> Animate<A> {
    /// Create new [`Animate`].
    pub fn new(animation: A) -> Self {
        Self { animation }
    }
}

struct AnimateMessage(Duration);

type Element<A, P, T> = <<A as Animation<T>>::View as View<Context<P>, T>>::Element;
type State<A, P, T> = <<A as Animation<T>>::View as View<Context<P>, T>>::State;

impl<A> ViewMarker for Animate<A> {}
impl<P, T, A> View<Context<P>, T> for Animate<A>
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

        if is_animating {
            let layout = element.layout_node();
            cx.request_start_animating(layout);
        }

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

        let widget = AnimateWidget::new(element, on_animate);

        let state = AnimateState {
            view_id,
            anim,
            state,
            is_animating,
        };

        (widget, state)
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        let should_animate = self.animation.rebuild(&mut state.anim, data);

        let widget = WidgetMut::new(
            element.parent,
            element.widget.contents(),
        );

        let view = A::view(&mut state.anim, data);
        view.rebuild(widget, &mut state.state, cx, data);

        if state.is_animating != should_animate {
            let layout = element.layout_node();

            match should_animate {
                true => cx.request_start_animating(layout),
                false => cx.request_stop_animating(layout),
            }

            state.is_animating = should_animate;
        }
    }

    fn message(
        element: Mut<'_, Self::Element>,
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

            if state.is_animating != should_animate {
                let layout = element.layout_node();

                match should_animate {
                    true => cx.request_start_animating(layout),
                    false => cx.request_stop_animating(layout),
                }

                state.is_animating = should_animate;
            }

            return Action::new();
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

    fn teardown(element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        let layout = element.layout_node();

        if state.is_animating {
            cx.request_stop_animating(layout);
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
}
