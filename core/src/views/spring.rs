use std::{f32::consts::TAU, time::Duration};

use ori::{Builder, BuilderMarker};

use crate::{
    BoxedWidget, Context, Platform, WidgetView,
    views::{Animate, Animation},
};

/// A spring [`Animation`].
pub fn spring<T, V>(
    target: f32,
    build: impl FnMut(&T, f32) -> V,
) -> Spring<impl FnMut(&T, f32) -> V> {
    Spring::new(target, build)
}

/// A spring [`Animation`].
pub struct Spring<F> {
    build: F,

    target: f32,

    threshold: f32,
    stiffness: f32,
    damping:   f32,
    mass:      f32,
}

impl<F> Spring<F> {
    /// Create new [`Spring`].
    pub fn new(target: f32, build: F) -> Self {
        Self {
            build,
            target,
            threshold: 0.1,
            stiffness: 170.0,
            damping: 26.0,
            mass: 1.0,
        }
    }

    /// Set the threshold for when animation should stop.
    pub fn threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    /// Set the stiffness of the spring.
    pub fn stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;
        self
    }

    /// Set the damping of the spring.
    pub fn damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    /// Set the mass of the spring.
    pub fn mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    /// Derive the parameters from perceptual duration and bounciness.
    pub fn duration_bounce(mut self, duration: f32, bounce: f32) -> Self {
        self.mass = 1.0;
        self.stiffness = (TAU / duration).powi(2);

        if bounce >= 0.0 {
            self.damping = ((1.0 - bounce) * TAU * 2.0) / duration;
        } else {
            self.damping = TAU * 2.0 / (duration * (1.0 + bounce));
        }

        self
    }
}

impl<F> BuilderMarker for Spring<F> {}
impl<P, T, F, V> Builder<Context<P>, T> for Spring<F>
where
    P: Platform,
    F: FnMut(&T, f32) -> V,
    V: WidgetView<P, T>,
{
    type Element = BoxedWidget<P>;

    fn build(self) -> impl WidgetView<P, T> {
        Animate::new(self)
    }
}

pub struct SpringState<F> {
    build: F,

    target:   f64,
    position: f64,
    velocity: f64,

    threshold: f64,
    stiffness: f64,
    damping:   f64,
    mass:      f64,
}

impl<T, F, V> Animation<T> for Spring<F>
where
    F: FnMut(&T, f32) -> V,
{
    type State = SpringState<F>;
    type View = V;

    fn build(self, _data: &mut T) -> (Self::State, bool) {
        let state = SpringState {
            build: self.build,

            target:   self.target as f64,
            position: self.target as f64,
            velocity: 0.0,

            threshold: self.threshold as f64,
            stiffness: self.stiffness as f64,
            damping:   self.damping as f64,
            mass:      self.mass as f64,
        };

        (state, false)
    }

    fn rebuild(self, state: &mut Self::State, _data: &mut T) -> bool {
        state.build = self.build;
        state.target = self.target as f64;
        state.threshold = self.threshold as f64;
        state.stiffness = self.stiffness as f64;
        state.damping = self.damping as f64;
        state.mass = self.mass as f64;

        state.position != state.target
    }

    fn animate(state: &mut Self::State, _data: &mut T, duration: Duration) -> bool {
        let delta = duration.as_secs_f64() / 8.0;

        for _ in 0..8 {
            let acceleration = (state.stiffness * (state.target - state.position)
                - state.damping * state.velocity)
                / state.mass;

            state.velocity += acceleration * delta;
            state.position += state.velocity * delta;
        }

        if (state.position - state.target).abs() <= state.threshold
            && state.velocity.abs() <= state.threshold
        {
            state.velocity = 0.0;
            state.position = state.target;

            false
        } else {
            true
        }
    }

    fn view(state: &mut Self::State, data: &T) -> Self::View {
        (state.build)(data, state.position as f32)
    }
}
