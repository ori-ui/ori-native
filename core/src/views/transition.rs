use std::{f32::consts::PI, rc::Rc, sync::Arc, time::Duration};

use crate::{
    Color, Platform, WidgetView,
    views::{Animation, animate},
};

/// A transition curve.
pub trait Transition {
    /// The duration in seconds.
    fn duration(&self) -> f32;

    /// The curve function that maps `t = 0.0..=1.0`.
    fn curve(&self, t: f32) -> f32;
}

/// [`View`](ori::View) of a transition.
pub fn transition<P, T, U, V>(
    value: U,
    transition: impl Transition,
    build: impl FnMut(&T, U) -> V,
) -> impl WidgetView<P, T>
where
    P: Platform,
    U: Transitionable,
    V: WidgetView<P, T>,
{
    animate(TransitionAnimation {
        value,
        transition,
        build,
    })
}

impl Transition for Box<dyn Transition> {
    fn duration(&self) -> f32 {
        self.as_ref().duration()
    }

    fn curve(&self, t: f32) -> f32 {
        self.as_ref().curve(t)
    }
}

impl Transition for Rc<dyn Transition> {
    fn duration(&self) -> f32 {
        self.as_ref().duration()
    }

    fn curve(&self, t: f32) -> f32 {
        self.as_ref().curve(t)
    }
}

impl Transition for Arc<dyn Transition> {
    fn duration(&self) -> f32 {
        self.as_ref().duration()
    }

    fn curve(&self, t: f32) -> f32 {
        self.as_ref().curve(t)
    }
}

struct TransitionAnimation<U, X, F> {
    value:      U,
    transition: X,
    build:      F,
}

impl<T, U, X, F, V> Animation<T> for TransitionAnimation<U, X, F>
where
    U: Transitionable,
    X: Transition,
    F: FnMut(&T, U) -> V,
{
    type State = State<U, X, F>;
    type View = V;

    fn build(self, _data: &mut T) -> (Self::State, bool) {
        let state = State {
            state:      self.value.build(),
            transition: self.transition,
            build:      self.build,
        };

        (state, false)
    }

    fn rebuild(self, state: &mut Self::State, _data: &mut T) -> bool {
        state.transition = self.transition;
        state.build = self.build;
        U::start(
            &mut state.state,
            &state.transition,
            self.value,
        )
    }

    fn animate(state: &mut Self::State, _data: &mut T, duration: Duration) -> bool {
        let delta = duration.as_secs_f32() / state.transition.duration();
        U::update(&mut state.state, delta)
    }

    fn view(state: &mut Self::State, data: &T) -> Self::View {
        let value = U::value(&state.state, &state.transition);
        (state.build)(data, value)
    }
}

struct State<U, X, F>
where
    U: Transitionable,
{
    state:      U::State,
    transition: X,
    build:      F,
}

pub trait Transitionable {
    type State;

    fn build(self) -> Self::State;

    fn start(state: &mut Self::State, transition: &impl Transition, target: Self) -> bool;

    fn update(state: &mut Self::State, delta: f32) -> bool;

    fn value(state: &Self::State, transition: &impl Transition) -> Self;
}

/// Type that can be linearly interpolated.
pub trait Lerp {
    /// Interpolate linearly between `a` and `b` by `t`.
    fn lerp(a: &Self, b: &Self, t: f32) -> Self;
}

/// Linear [`Transition`] curve.
pub struct Linear(pub f32);

/// Ease out [`Transition`] curve.
pub struct Ease(pub f32);

/// Elastic out [`Transition`] curve.
pub struct Elastic(pub f32);

/// Elastic in [`Transition`] curve.
pub struct ElasticIn(pub f32);

/// Back out [`Transition`] curve.
pub struct Back(pub f32);

/// Back in [`Transition`] curve.
pub struct BackIn(pub f32);

/// Back in out [`Transition`] curve.
pub struct BackInOut(pub f32);

impl Transition for Linear {
    fn duration(&self) -> f32 {
        self.0
    }

    fn curve(&self, t: f32) -> f32 {
        t
    }
}

impl Transition for Ease {
    fn duration(&self) -> f32 {
        self.0
    }

    fn curve(&self, t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
    }
}

impl Transition for Elastic {
    fn duration(&self) -> f32 {
        self.0
    }

    fn curve(&self, t: f32) -> f32 {
        1.0 + f32::powf(2.0, -10.0 * t) * f32::sin((10.0 * t - 0.75) * PI * 2.0 / 3.0)
    }
}

impl Transition for ElasticIn {
    fn duration(&self) -> f32 {
        self.0
    }

    fn curve(&self, t: f32) -> f32 {
        -f32::powf(2.0, 10.0 * t - 10.0) * f32::sin((10.0 * t - 10.75) * PI * 2.0 / 3.0)
    }
}

const C1: f32 = 1.70158;
const C2: f32 = C1 * 1.525;

impl Transition for Back {
    fn duration(&self) -> f32 {
        self.0
    }

    fn curve(&self, t: f32) -> f32 {
        1.0 + (C1 + 1.0) * f32::powi(t - 1.0, 3) + C1 * f32::powi(t - 1.0, 2)
    }
}

impl Transition for BackIn {
    fn duration(&self) -> f32 {
        self.0
    }

    fn curve(&self, t: f32) -> f32 {
        (C1 + 1.0) * f32::powi(t, 3) - C1 * f32::powi(t, 2)
    }
}

impl Transition for BackInOut {
    fn duration(&self) -> f32 {
        self.0
    }

    fn curve(&self, t: f32) -> f32 {
        if t < 0.5 {
            (f32::powi(2.0 * t, 2) * ((C2 + 1.0) * 2.0 * t - C2)) / 2.0
        } else {
            (f32::powi(2.0 * t - 2.0, 2) * ((C2 + 1.0) * (2.0 * t - 2.0) + C2) + 2.0) / 2.0
        }
    }
}

impl Lerp for f32 {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        *a * (1.0 - t) + *b * t
    }
}

impl Lerp for Color {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        a.mix(*b, t)
    }
}

impl<T> Transitionable for T
where
    T: Lerp + Clone + PartialEq,
{
    type State = TransitionState<Self>;

    fn build(self) -> Self::State {
        TransitionState {
            current: None,
            target:  self,
            elapsed: 0.0,
        }
    }

    fn start(state: &mut Self::State, transition: &impl Transition, target: Self) -> bool {
        if state.target != target {
            let current = Self::value(state, transition);
            state.current = Some(current);
            state.target = target;
            state.elapsed = 0.0;
        }

        state.elapsed < 1.0
    }

    fn update(state: &mut Self::State, delta: f32) -> bool {
        state.elapsed += delta;
        state.elapsed = state.elapsed.clamp(0.0, 1.0);
        state.elapsed < 1.0
    }

    fn value(state: &Self::State, transition: &impl Transition) -> Self {
        if let Some(ref current) = state.current {
            let t = transition.curve(state.elapsed);
            Self::lerp(current, &state.target, t)
        } else {
            state.target.clone()
        }
    }
}

pub struct TransitionState<T> {
    current: Option<T>,
    target:  T,
    elapsed: f32,
}

macro_rules! impl_tuple {
    () => {
        impl_tuple!(@impl);
    };

    ($first_name:ident:$first_arg:ident $(, $rest_name:ident:$rest_arg:ident)*) => {
        impl_tuple!(@impl $first_name:$first_arg $(,$rest_name:$rest_arg)*);
        impl_tuple!($($rest_name:$rest_arg),*);
    };

    (@impl $($name:ident:$arg:ident),*) => {
        impl<$($name),*> Transitionable for ($($name,)*)
        where
            $($name: Transitionable),*
        {
            type State = ($($name::State,)*);

            #[allow(
                non_snake_case,
                unused,
                clippy::unused_unit,
            )]
            fn build(self) -> Self::State {
                let ($($name,)*) = self;
                ($($name.build(),)*)
            }

            #[allow(
                non_snake_case,
                unused,
                clippy::unused_unit,
            )]
            fn start(
                ($($name,)*): &mut Self::State,
                transition: &impl Transition,
                ($($arg,)*): Self,
            ) -> bool {
                false $(| $name::start($name, transition, $arg))*
            }

            #[allow(
                non_snake_case,
                unused,
                clippy::unused_unit,
            )]
            fn update(($($name,)*): &mut Self::State, delta: f32) -> bool {
                false $(| $name::update($name, delta))*
            }

            #[allow(
                non_snake_case,
                unused,
                clippy::unused_unit,
            )]
            fn value(($($name,)*): &Self::State, transition: &impl Transition) -> Self {
                ($($name::value($name, transition),)*)
            }
        }
    };
}

impl_tuple!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i, J:j, K:k, L:l);
