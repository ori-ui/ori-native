use ori::{Action, Message, Mut, View, ViewMarker};

use crate::{
    Border, BorderStyle, Color, Context, Corners, Direction, FlexContainer, FlexStyle, Layout,
    LayoutStyle, Length, Overflow, Padding, Platform, Shadow, Sides, WidgetViewSeq,
    widgets::GroupWidget,
};

/// Container [`View`] with flexbox layout.
pub fn flex<V>(contents: V) -> Flex<V> {
    Flex::new(contents)
}

/// [`View`] of a flex row.
pub fn row<V>(contents: V) -> Flex<V> {
    Flex::new(contents).direction(Direction::Horizontal)
}

/// [`View`] of a flex column.
pub fn column<V>(contents: V) -> Flex<V> {
    Flex::new(contents).direction(Direction::Vertical)
}

/// [`View`] of a flex container.
pub struct Flex<V> {
    contents:       V,
    layout:         LayoutStyle,
    border:         BorderStyle,
    padding:        Sides<Length>,
    flex:           FlexStyle,
    background:     Color,
    corners:        Corners<f32>,
    overflow:       Overflow,
    shadow:         Shadow,
    hardware_layer: bool,
}

impl<V> Flex<V> {
    /// Create new [`Flex`].
    pub fn new(contents: V) -> Self {
        Self {
            contents,
            layout: LayoutStyle::default(),
            border: BorderStyle::default(),
            padding: Sides::all(Length::Length(0.0)),
            flex: FlexStyle::default(),
            background: Color::TRANSPARENT,
            corners: Corners::all(0.0),
            overflow: Overflow::Visible,
            shadow: Shadow::default(),
            hardware_layer: false,
        }
    }

    /// Set the background color.
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set the overflow behaviour.
    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.overflow = overflow;
        self
    }

    /// Set the shadow properties.
    pub fn shadow(self, dx: f32, dy: f32, radius: f32, color: Color) -> Self {
        self.shadow_offset(dx, dy)
            .shadow_radius(radius)
            .shadow_color(color)
    }

    /// Set the shadow color.
    pub fn shadow_color(mut self, color: Color) -> Self {
        self.shadow.color = color;
        self
    }

    /// Set the shadow offset.
    pub fn shadow_offset(mut self, dx: f32, dy: f32) -> Self {
        self.shadow.offset_x = dx;
        self.shadow.offset_y = dy;
        self
    }

    /// Set the shadow radius.
    pub fn shadow_radius(mut self, radius: f32) -> Self {
        self.shadow.radius = radius;
        self
    }

    /// Set the shadow spread.
    pub fn shadow_spread(mut self, spread: f32) -> Self {
        self.shadow.spread = spread;
        self
    }

    /// Set the radius of all corners.
    pub fn corner(mut self, radii: impl Into<Corners<f32>>) -> Self {
        self.corners = radii.into();
        self
    }

    /// Set the radius of the top left corner.
    pub fn corner_top_left(mut self, radius: f32) -> Self {
        self.corners.top_left = radius;
        self
    }

    /// Set the radius of the top right corner.
    pub fn corner_top_right(mut self, radius: f32) -> Self {
        self.corners.top_right = radius;
        self
    }

    /// Set the radius of the bottom right corner.
    pub fn corner_bottom_right(mut self, radius: f32) -> Self {
        self.corners.bottom_right = radius;
        self
    }

    /// Set the radius of the bottom left corner.
    pub fn corner_bottom_left(mut self, radius: f32) -> Self {
        self.corners.bottom_left = radius;
        self
    }

    /// Set whether to use a hardware layer.
    ///
    /// # Platform
    ///  - `android` set the layer type of the underlying view to hardware.
    ///  - `other` not supported.
    pub fn hardware_layer(mut self, enabled: bool) -> Self {
        self.hardware_layer = enabled;
        self
    }
}

impl<V> Layout for Flex<V> {
    fn get_layout_style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.layout
    }
}

impl<V> Border for Flex<V> {
    fn get_border_style_mut(&mut self) -> &mut BorderStyle {
        &mut self.border
    }
}

impl<V> Padding for Flex<V> {
    fn get_padding_mut(&mut self) -> &mut Sides<Length> {
        &mut self.padding
    }
}

impl<V> FlexContainer for Flex<V> {
    fn get_flex_style_mut(&mut self) -> &mut FlexStyle {
        &mut self.flex
    }
}

impl<V> ViewMarker for Flex<V> {}
impl<P, T, V> View<Context<P>, T> for Flex<V>
where
    P: Platform,
    V: WidgetViewSeq<P, T>,
{
    type Element = GroupWidget<P>;
    type State = FlexState<P, T, V>;

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let mut group = GroupWidget::new(cx);
        group.set_layout(cx, self.layout);
        group.set_padding(cx, self.padding);
        group.set_border(cx, self.border);
        group.set_flex(cx, self.flex);
        group.set_background(cx, self.background);
        group.set_corners(cx, self.corners);
        group.set_overflow(cx, self.overflow);
        group.set_shadow(cx, self.shadow);
        group.set_hardware_layer(cx, self.hardware_layer);

        let state = self.contents.seq_build(&mut group.elements(), cx, data);

        let state = FlexState {
            state,
            layout: self.layout,
            border: self.border,
            padding: self.padding,
            overflow: self.overflow,
            flex: self.flex,
            background: self.background,
            corners: self.corners,
            shadow: self.shadow,
            hardware_layer: self.hardware_layer,
        };

        (group, state)
    }

    fn rebuild(
        self,
        mut element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        self.contents.seq_rebuild(
            &mut element.elements(),
            &mut state.state,
            cx,
            data,
        );

        if state.layout != self.layout {
            state.layout = self.layout;
            element.set_layout(cx, self.layout);
        }

        if state.padding != self.padding {
            state.padding = self.padding;
            element.set_padding(cx, self.padding);
        }

        if state.overflow != self.overflow {
            state.overflow = self.overflow;
            element.set_overflow(cx, self.overflow);
        }

        if state.flex != self.flex {
            state.flex = self.flex;
            element.set_flex(cx, self.flex);
        }

        if state.border != self.border {
            state.border = self.border;
            element.set_border(cx, self.border);
        }

        if state.background != self.background {
            state.background = self.background;
            element.set_background(cx, self.background);
        }

        if state.corners != self.corners {
            state.corners = self.corners;
            element.set_corners(cx, self.corners);
        }

        if state.overflow != self.overflow {
            state.overflow = self.overflow;
            element.set_overflow(cx, self.overflow);
        }

        if state.shadow != self.shadow {
            state.shadow = self.shadow;
            element.set_shadow(cx, self.shadow);
        }

        if state.hardware_layer != self.hardware_layer {
            state.hardware_layer = self.hardware_layer;
            element.set_hardware_layer(cx, self.hardware_layer);
        }
    }

    fn message(
        mut element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        V::seq_message(
            &mut element.elements(),
            &mut state.state,
            cx,
            data,
            message,
        )
    }

    fn teardown(mut element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        V::seq_teardown(&mut element.elements(), state.state, cx);
        element.teardown(cx);
    }
}

pub struct FlexState<P, T, V>
where
    P: Platform,
    V: WidgetViewSeq<P, T>,
{
    state:          V::State,
    layout:         LayoutStyle,
    border:         BorderStyle,
    flex:           FlexStyle,
    padding:        Sides<Length>,
    overflow:       Overflow,
    background:     Color,
    corners:        Corners<f32>,
    shadow:         Shadow,
    hardware_layer: bool,
}
