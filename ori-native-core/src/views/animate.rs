use std::time::Duration;

use ori::{Action, Message, Mut, View, ViewMarker};

use crate::{Context, Lifecycle, Platform, WidgetView};

#[allow(clippy::type_complexity)]
pub fn animate<P, T, U, V>(
    initial: impl FnOnce() -> U,
    rebuild: impl FnOnce(&mut U) -> bool,
    animate: impl FnMut(&mut U, Duration) -> bool,
    build: impl Fn(&U, &T) -> V,
) -> impl WidgetView<P, T>
where
    P: Platform,
    V: WidgetView<P, T>,
{
    Animate::new(initial, rebuild, animate, build)
}

pub struct Animate<F, G, H, I> {
    initial: F,
    rebuild: G,
    animate: H,
    build:   I,
}

impl<F, G, H, I> Animate<F, G, H, I> {
    pub fn new(initial: F, animating: G, animate: H, build: I) -> Self {
        Self {
            initial,
            rebuild: animating,
            animate,
            build,
        }
    }
}

impl<F, G, H, I> ViewMarker for Animate<F, G, H, I> {}
impl<P, T, F, G, H, I, U, V> View<Context<P>, T> for Animate<F, G, H, I>
where
    P: Platform,
    F: FnOnce() -> U,
    G: FnOnce(&mut U) -> bool,
    H: FnMut(&mut U, Duration) -> bool,
    I: Fn(&U, &T) -> V,
    V: WidgetView<P, T>,
{
    type Element = V::Element;
    type State = (H, I, U, bool, V::State);

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let mut state = (self.initial)();
        let should_animate = (self.rebuild)(&mut state);

        if should_animate {
            cx.start_animating();
        }

        let view = (self.build)(&state, data);
        let (element, contents) = view.build(cx, data);

        (
            element,
            (
                self.animate,
                self.build,
                state,
                should_animate,
                contents,
            ),
        )
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        (animate, build, state, is_animating, contents): &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        let view = (self.build)(state, data);
        view.rebuild(element, contents, cx, data);

        let should_animate = (self.rebuild)(state);

        if *is_animating != should_animate {
            match should_animate {
                true => cx.start_animating(),
                false => cx.stop_animating(),
            }

            *is_animating = should_animate;
        }

        *animate = self.animate;
        *build = self.build;
    }

    fn message(
        mut element: Mut<'_, Self::Element>,
        (animate, build, state, is_animating, contents): &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(Lifecycle::Animate(delta)) = message.get()
            && *is_animating
        {
            let should_animate = animate(state, *delta);
            let view = build(state, data);

            view.rebuild(element.reborrow(), contents, cx, data);

            if *is_animating != should_animate {
                match should_animate {
                    true => cx.start_animating(),
                    false => cx.stop_animating(),
                }

                *is_animating = should_animate;
            }
        }

        V::message(element, contents, cx, data, message)
    }

    fn teardown(
        element: Self::Element,
        (_aniamte, _build, _state, is_animating, contents): Self::State,
        cx: &mut Context<P>,
    ) {
        V::teardown(element, contents, cx);

        if is_animating {
            cx.stop_animating();
        }
    }
}
