use crate::{Color, Sides};

/// The style of an outline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutlineStyle {
    /// The color.
    pub color: Color,

    /// The widths.
    pub width: Sides<f32>,

    /// The amount of space between the border and the outline.
    pub offset: Sides<f32>,
}

impl Default for OutlineStyle {
    fn default() -> Self {
        Self {
            color:  Color::TRANSPARENT,
            width:  Sides::all(0.0),
            offset: Sides::all(0.0),
        }
    }
}

/// A trait for views with outlines.
pub trait StyleOutline: Sized {
    /// Get a mutable reference to the outline style.
    fn get_outline_style_mut(&mut self) -> &mut OutlineStyle;

    /// Override the outline style.
    fn set_outline(mut self, style: OutlineStyle) -> Self {
        *self.get_outline_style_mut() = style;
        self
    }

    /// Set the outline width and color.
    fn outline(self, width: impl Into<Sides<f32>>, color: Color) -> Self {
        self.outline_width(width).outline_color(color)
    }

    /// Set the outline width on the top, and color.
    fn outline_top(self, width: f32, color: Color) -> Self {
        self.outline_top_width(width).outline_color(color)
    }

    /// Set the outline width on the right, and color.
    fn outline_right(self, width: f32, color: Color) -> Self {
        self.outline_right_width(width).outline_color(color)
    }

    /// Set the outline width on the bottom, and color.
    fn outline_bottom(self, width: f32, color: Color) -> Self {
        self.outline_bottom_width(width).outline_color(color)
    }

    /// Set the outline width on the left, and color.
    fn outline_left(self, width: f32, color: Color) -> Self {
        self.outline_left_width(width).outline_color(color)
    }

    /// Set the outline color.
    fn outline_color(mut self, color: Color) -> Self {
        self.get_outline_style_mut().color = color;
        self
    }

    /// Set the outline width.
    fn outline_width(mut self, width: impl Into<Sides<f32>>) -> Self {
        self.get_outline_style_mut().width = width.into();
        self
    }

    /// Set the outline width on the top.
    fn outline_top_width(mut self, width: f32) -> Self {
        self.get_outline_style_mut().width.top = width;
        self
    }

    /// Set the outline width on the right.
    fn outline_right_width(mut self, width: f32) -> Self {
        self.get_outline_style_mut().width.right = width;
        self
    }

    /// Set the outline width on the bottom.
    fn outline_bottom_width(mut self, width: f32) -> Self {
        self.get_outline_style_mut().width.bottom = width;
        self
    }

    /// Set the outline width on the left.
    fn outline_left_width(mut self, width: f32) -> Self {
        self.get_outline_style_mut().width.left = width;
        self
    }

    /// Set the outline offset.
    fn outline_offset(mut self, offset: impl Into<Sides<f32>>) -> Self {
        self.get_outline_style_mut().offset = offset.into();
        self
    }

    /// Set the outline offset on the top.
    fn outline_top_offset(mut self, offset: f32) -> Self {
        self.get_outline_style_mut().offset.top = offset;
        self
    }

    /// Set the outline offset on the right.
    fn outline_right_offset(mut self, offset: f32) -> Self {
        self.get_outline_style_mut().offset.right = offset;
        self
    }

    /// Set the outline offset on the bottom.
    fn outline_bottom_offset(mut self, offset: f32) -> Self {
        self.get_outline_style_mut().offset.bottom = offset;
        self
    }

    /// Set the outline offset on the left.
    fn outline_left_offset(mut self, offset: f32) -> Self {
        self.get_outline_style_mut().offset.left = offset;
        self
    }
}
