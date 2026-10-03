use crate::Color;

/// Box shadow.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Shadow {
    /// Color of the shadow.
    pub color: Color,

    /// Offset in the `x` direction.
    pub offset_x: f32,

    /// Offset in the `y` direction.
    pub offset_y: f32,

    /// Blur radius.
    pub radius: f32,

    /// Spread radius.
    pub spread: f32,
}

/// A trait for styling shadows.
pub trait StyleShadow: Sized {
    /// Get a mutable reference to the shadow.
    fn get_shadow_mut(&mut self) -> &mut Shadow;

    /// Override the shadow style.
    fn set_shadow(mut self, shadow: Shadow) -> Self {
        *self.get_shadow_mut() = shadow;
        self
    }

    /// Set the shadow properties.
    fn shadow(self, dx: f32, dy: f32, radius: f32, color: Color) -> Self {
        self.shadow_offset(dx, dy)
            .shadow_radius(radius)
            .shadow_color(color)
    }

    /// Set the shadow color.
    fn shadow_color(mut self, color: Color) -> Self {
        self.get_shadow_mut().color = color;
        self
    }

    /// Set the shadow offset.
    fn shadow_offset(mut self, dx: f32, dy: f32) -> Self {
        self.get_shadow_mut().offset_x = dx;
        self.get_shadow_mut().offset_y = dy;
        self
    }

    /// Set the shadow radius.
    fn shadow_radius(mut self, radius: f32) -> Self {
        self.get_shadow_mut().radius = radius;
        self
    }

    /// Set the shadow spread.
    fn shadow_spread(mut self, spread: f32) -> Self {
        self.get_shadow_mut().spread = spread;
        self
    }
}
