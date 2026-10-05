use crate::Color;

/// Fractional length.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Fract(pub f32);

/// Length that cannot be `auto`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    /// Length in pixels.
    Length(f32),

    /// Length in fraction of parent size.
    Fract(f32),
}

impl Default for Length {
    fn default() -> Self {
        Self::Length(0.0)
    }
}

impl From<f32> for Length {
    fn from(x: f32) -> Self {
        Length::Length(x)
    }
}

impl From<Fract> for Length {
    fn from(Fract(x): Fract) -> Self {
        Length::Fract(x)
    }
}

/// A direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    /// Horizontal or row.
    Horizontal,

    /// Vertical or column.
    Vertical,
}

/// Alignment of contents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Align {
    /// Contents is aligned towards the start.
    Start,

    /// Contents is aligned towards the center.
    Center,

    /// Contents is aligned towards the end.
    End,

    /// Contents is aligned towards the baseline.
    Baseline,

    /// Contents stretched to fill the container.
    Stretch,
}

/// Justification of contents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Justify {
    /// Contents is justified towards the start.
    Start,

    /// Contents is justified towards the center.
    Center,

    /// Contents is justified towards the end.
    End,

    /// Contents is stretched to fill the container.
    Stretch,

    /// Contents is justified with even space inbetween.
    SpaceBetween,

    /// Contents is justified with evenly.
    SpaceEvenly,

    /// Contents is justified with even space around.
    SpaceAround,
}

/// Positioning within a container.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Position {
    /// Offset is computed relative to the final position.
    Relative,

    /// Offset is computed relative to the container and no space is created for the item.
    Absolute,
}

/// Values for each size of a rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sides<T> {
    /// The top side.
    pub top: T,

    /// The right side.
    pub right: T,

    /// The bottom side.
    pub bottom: T,

    /// The left side.
    pub left: T,
}

impl<T> Sides<T> {
    /// Create new [`Sides`] with the same value for all sides.
    pub const fn all(value: T) -> Self
    where
        T: Copy,
    {
        Self {
            top:    value,
            right:  value,
            bottom: value,
            left:   value,
        }
    }
}

impl<T> From<T> for Sides<Length>
where
    T: Into<Length>,
{
    fn from(value: T) -> Self {
        Self::all(value.into())
    }
}

impl<T> From<T> for Sides<Option<Length>>
where
    T: Into<Length>,
{
    fn from(value: T) -> Self {
        Self::all(Some(value.into()))
    }
}

impl From<Option<Length>> for Sides<Option<Length>> {
    fn from(value: Option<Length>) -> Self {
        Self::all(value)
    }
}

impl<T, U, V, W> From<(T, U, V, W)> for Sides<Length>
where
    T: Into<Length>,
    U: Into<Length>,
    V: Into<Length>,
    W: Into<Length>,
{
    fn from((top, right, bottom, left): (T, U, V, W)) -> Self {
        Self {
            top:    top.into(),
            right:  right.into(),
            bottom: bottom.into(),
            left:   left.into(),
        }
    }
}

impl<T, U, V, W> From<(T, U, V, W)> for Sides<Option<Length>>
where
    T: Into<Length>,
    U: Into<Length>,
    V: Into<Length>,
    W: Into<Length>,
{
    fn from((top, right, bottom, left): (T, U, V, W)) -> Self {
        Self {
            top:    Some(top.into()),
            right:  Some(right.into()),
            bottom: Some(bottom.into()),
            left:   Some(left.into()),
        }
    }
}

impl<T> From<Sides<T>> for [T; 4] {
    fn from(sides: Sides<T>) -> Self {
        [sides.top, sides.right, sides.bottom, sides.left]
    }
}

/// A two dimensional point.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point<T> {
    /// The x value.
    pub x: T,

    /// The y value.
    pub y: T,
}

impl<T> Point<T> {
    /// Create new [`Point`] with the same value for `x` and `y`.
    pub const fn all(value: T) -> Self
    where
        T: Copy,
    {
        Self { x: value, y: value }
    }
}

/// A two dimensional size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Size<T> {
    /// The width value.
    pub width: T,

    /// The height value.
    pub height: T,
}

impl<T> Size<T> {
    /// Create new [`Size`] with the same value for width and height.
    pub const fn all(value: T) -> Self
    where
        T: Copy,
    {
        Self {
            width:  value,
            height: value,
        }
    }
}

/// The style of a layout node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutStyle {
    /// The positioning strategy.
    pub position: Position,

    /// The alignment within the container.
    pub align_self: Option<Align>,

    /// The factor by which the view will shrink.
    pub flex_shrink: f32,

    /// The factor by which the view will grow.
    pub flex_grow: f32,

    /// The default size before remaining space is distributed.
    pub flex_basis: Option<Length>,

    /// The margin around the view.
    pub margin: Sides<Option<Length>>,

    /// The insets from the parent.
    pub inset: Sides<Option<Length>>,

    /// The size of the view.
    pub size: Size<Option<Length>>,

    /// The minimum size of the view.
    pub min_size: Size<Option<Length>>,

    /// The maximum size of the view.
    pub max_size: Size<Option<Length>>,

    /// The preferred aspect ratio of the view.
    pub aspect_ratio: Option<f32>,
}

impl Default for LayoutStyle {
    fn default() -> Self {
        Self {
            position:     Position::Relative,
            align_self:   None,
            flex_shrink:  0.0,
            flex_grow:    0.0,
            flex_basis:   None,
            margin:       Sides::all(Some(Length::Length(0.0))),
            inset:        Sides::all(None),
            size:         Size::all(None),
            min_size:     Size::all(None),
            max_size:     Size::all(None),
            aspect_ratio: None,
        }
    }
}

/// The style of a border.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderStyle {
    /// The color.
    pub color: Color,

    /// The widths.
    pub width: Sides<Length>,
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self {
            color: Color::TRANSPARENT,
            width: Sides::all(Length::Length(0.0)),
        }
    }
}

/// The style of a flex container.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexStyle {
    /// The direction children are layed out.
    pub direction: Direction,

    /// Whether items should be layed out in reverse order.
    pub reverse: bool,

    /// Whether items should wrap when available space is exceeded.
    pub wrap: bool,

    /// The justification strategy.
    pub justify_content: Option<Justify>,

    /// The alignment strategy of iems.
    pub align_items: Option<Align>,

    /// The gap between items.
    pub gap: Size<Length>,
}

impl Default for FlexStyle {
    fn default() -> Self {
        Self {
            direction:       Direction::Horizontal,
            reverse:         false,
            wrap:            false,
            justify_content: None,
            align_items:     None,
            gap:             Size::all(Length::Length(0.0)),
        }
    }
}

/// A trait for views that can style its layout.
pub trait StyleLayout: Sized {
    /// Get a mutable reference to the layout style.
    fn get_layout_style_mut(&mut self) -> &mut LayoutStyle;

    /// Override the layout style.
    fn set_layout(mut self, style: LayoutStyle) -> Self {
        *self.get_layout_style_mut() = style;
        self
    }

    /// Set the positioning strategy.
    fn position(mut self, position: Position) -> Self {
        self.get_layout_style_mut().position = position;
        self
    }

    /// Set how the view should be aligned in the container.
    fn align_self(mut self, align_self: impl Into<Option<Align>>) -> Self {
        self.get_layout_style_mut().align_self = align_self.into();
        self
    }

    /// Set the inset from all sides.
    fn inset(mut self, inset: impl Into<Sides<Option<Length>>>) -> Self {
        self.get_layout_style_mut().inset = inset.into();
        self
    }

    /// Set the inset from the top.
    fn top(mut self, inset: impl Into<Length>) -> Self {
        self.get_layout_style_mut().inset.top = Some(inset.into());
        self
    }

    /// Set the inset from the right.
    fn right(mut self, inset: impl Into<Length>) -> Self {
        self.get_layout_style_mut().inset.right = Some(inset.into());
        self
    }

    /// Set the inset from the bottom.
    fn bottom(mut self, inset: impl Into<Length>) -> Self {
        self.get_layout_style_mut().inset.bottom = Some(inset.into());
        self
    }

    /// Set the inset from the left.
    fn left(mut self, inset: impl Into<Length>) -> Self {
        self.get_layout_style_mut().inset.left = Some(inset.into());
        self
    }

    /// Set the `width` and `height`.
    fn size(self, width: impl Into<Length>, height: impl Into<Length>) -> Self {
        self.width(width).height(height)
    }

    /// Set the `width`.
    fn width(mut self, width: impl Into<Length>) -> Self {
        self.get_layout_style_mut().size.width = Some(width.into());
        self
    }

    /// Set the `height`.
    fn height(mut self, height: impl Into<Length>) -> Self {
        self.get_layout_style_mut().size.height = Some(height.into());
        self
    }

    /// Set the minimum `width` and `height`.
    fn min_size(self, min_width: impl Into<Length>, min_height: impl Into<Length>) -> Self {
        self.min_width(min_width).min_height(min_height)
    }

    /// Set the minimum `width`.
    fn min_width(mut self, min_width: impl Into<Length>) -> Self {
        self.get_layout_style_mut().min_size.width = Some(min_width.into());
        self
    }

    /// Set the minimum `height`.
    fn min_height(mut self, min_height: impl Into<Length>) -> Self {
        self.get_layout_style_mut().min_size.height = Some(min_height.into());
        self
    }

    /// Set the maximum `width` and `height`.
    fn max_size(self, max_width: impl Into<Length>, max_height: impl Into<Length>) -> Self {
        self.max_width(max_width).max_height(max_height)
    }

    /// Set the maximum `width`.
    fn max_width(mut self, max_width: impl Into<Length>) -> Self {
        self.get_layout_style_mut().max_size.width = Some(max_width.into());
        self
    }

    /// Set the maximum `height`.
    fn max_height(mut self, max_height: impl Into<Length>) -> Self {
        self.get_layout_style_mut().max_size.height = Some(max_height.into());
        self
    }

    /// Set the preferred aspect ratio.
    fn aspect_ratio(mut self, aspect_ratio: impl Into<Option<f32>>) -> Self {
        self.get_layout_style_mut().aspect_ratio = aspect_ratio.into();
        self
    }

    /// Set the margin on all sides.
    fn margin(mut self, margin: impl Into<Sides<Option<Length>>>) -> Self {
        self.get_layout_style_mut().margin = margin.into();
        self
    }

    /// Set the margin on the top.
    fn margin_top(mut self, width: impl Into<Length>) -> Self {
        self.get_layout_style_mut().margin.top = Some(width.into());
        self
    }

    /// Set the margin on the right.
    fn margin_right(mut self, width: impl Into<Length>) -> Self {
        self.get_layout_style_mut().margin.right = Some(width.into());
        self
    }

    /// Set the margin on the bottom.
    fn margin_bottom(mut self, width: impl Into<Length>) -> Self {
        self.get_layout_style_mut().margin.bottom = Some(width.into());
        self
    }

    /// Set the margin on the left.
    fn margin_left(mut self, width: impl Into<Length>) -> Self {
        self.get_layout_style_mut().margin.left = Some(width.into());
        self
    }

    /// Set the flex factor.
    fn flex(self, amount: f32) -> Self {
        self.flex_grow(amount).flex_shrink(amount)
    }

    /// Set the flex growth factor.
    fn flex_grow(mut self, amount: f32) -> Self {
        self.get_layout_style_mut().flex_grow = amount;
        self
    }

    /// Set the flex shrinkage factor.
    fn flex_shrink(mut self, amount: f32) -> Self {
        self.get_layout_style_mut().flex_shrink = amount;
        self
    }

    /// Set the flex basis.
    fn flex_basis(mut self, basis: impl Into<Length>) -> Self {
        self.get_layout_style_mut().flex_basis = Some(basis.into());
        self
    }
}

/// A trait for views with borders.
pub trait StyleBorder: Sized {
    /// Get a mutable reference to the border style.
    fn get_border_style_mut(&mut self) -> &mut BorderStyle;

    /// Override the border style.
    fn set_border(mut self, style: BorderStyle) -> Self {
        *self.get_border_style_mut() = style;
        self
    }

    /// Set the border width and color.
    fn border(self, width: impl Into<Sides<Length>>, color: Color) -> Self {
        self.border_width(width).border_color(color)
    }

    /// Set the border color.
    fn border_color(mut self, color: Color) -> Self {
        self.get_border_style_mut().color = color;
        self
    }

    /// Set the border width on all sides.
    fn border_width(mut self, width: impl Into<Sides<Length>>) -> Self {
        self.get_border_style_mut().width = width.into();
        self
    }

    /// Set the border width on the top, and color.
    fn border_top(self, width: impl Into<Length>, color: Color) -> Self {
        self.border_top_width(width).border_color(color)
    }

    /// Set the border width on the right, and color.
    fn border_right(self, width: impl Into<Length>, color: Color) -> Self {
        self.border_right_width(width).border_color(color)
    }

    /// Set the border width on the bottom, and color.
    fn border_bottom(self, width: impl Into<Length>, color: Color) -> Self {
        self.border_bottom_width(width).border_color(color)
    }

    /// Set the border width on the left, and color.
    fn border_left(self, width: impl Into<Length>, color: Color) -> Self {
        self.border_left_width(width).border_color(color)
    }

    /// Set the border width on the top.
    fn border_top_width(mut self, width: impl Into<Length>) -> Self {
        self.get_border_style_mut().width.top = width.into();
        self
    }

    /// Set the border width on the right.
    fn border_right_width(mut self, width: impl Into<Length>) -> Self {
        self.get_border_style_mut().width.right = width.into();
        self
    }

    /// Set the border width on the bottom.
    fn border_bottom_width(mut self, width: impl Into<Length>) -> Self {
        self.get_border_style_mut().width.bottom = width.into();
        self
    }

    /// Set the border width on the left.
    fn border_left_width(mut self, width: impl Into<Length>) -> Self {
        self.get_border_style_mut().width.left = width.into();
        self
    }
}

/// A trait for container views.
pub trait StylePadding: Sized {
    /// Get a mutable reference to the padding.
    fn get_padding_mut(&mut self) -> &mut Sides<Length>;

    /// Override the padding style.
    fn set_padding(mut self, padding: Sides<Length>) -> Self {
        *self.get_padding_mut() = padding;
        self
    }

    /// Set the padding on all sides.
    fn padding(mut self, padding: impl Into<Sides<Length>>) -> Self {
        *self.get_padding_mut() = padding.into();
        self
    }

    /// Set the padding on the top.
    fn padding_top(mut self, width: impl Into<Length>) -> Self {
        self.get_padding_mut().top = width.into();
        self
    }

    /// Set the padding on the right.
    fn padding_right(mut self, width: impl Into<Length>) -> Self {
        self.get_padding_mut().right = width.into();
        self
    }

    /// Set the padding on the bottom.
    fn padding_bottom(mut self, width: impl Into<Length>) -> Self {
        self.get_padding_mut().bottom = width.into();
        self
    }

    /// Set the padding on the left.
    fn padding_left(mut self, width: impl Into<Length>) -> Self {
        self.get_padding_mut().left = width.into();
        self
    }
}

/// A trait for flex containers.
pub trait StyleFlexContainer: Sized {
    /// Get a mutable reference to the flex style.
    fn get_flex_style_mut(&mut self) -> &mut FlexStyle;

    /// Override the flex style.
    fn set_flex(mut self, style: FlexStyle) -> Self {
        *self.get_flex_style_mut() = style;
        self
    }

    /// Set the flex direction.
    fn direction(mut self, direction: Direction) -> Self {
        self.get_flex_style_mut().direction = direction;
        self
    }

    /// Reverse the direction.
    fn reverse(mut self, reverse: bool) -> Self {
        self.get_flex_style_mut().reverse = reverse;
        self
    }

    /// Set whether wrapping is enabled.
    fn wrap(mut self, wrap: bool) -> Self {
        self.get_flex_style_mut().wrap = wrap;
        self
    }

    /// Set how contents are justified within the container.
    fn justify_content(mut self, justify: impl Into<Option<Justify>>) -> Self {
        self.get_flex_style_mut().justify_content = justify.into();
        self
    }

    /// Set how items are aligned within the container.
    fn align_items(mut self, align: impl Into<Option<Align>>) -> Self {
        self.get_flex_style_mut().align_items = align.into();
        self
    }

    /// Set the gap between items within the container.
    fn gap(mut self, gap: impl Into<Length>) -> Self {
        let gap = gap.into();

        self.get_flex_style_mut().gap.width = gap;
        self.get_flex_style_mut().gap.height = gap;

        self
    }
}
