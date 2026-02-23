use ori::{Action, Message, Mut, View, ViewMarker, ViewSeq};

use crate::{
    BorderLayout, BoxedWidget, Color, ContainerLayout, Context, Direction, FlexLayout, Layout,
    Lifecycle, Pod,
    native::{Group, HasGroup},
};

pub fn row<V>(contents: V) -> Flex<V> {
    Flex::new(contents, Direction::Horizontal)
}

pub fn column<V>(contents: V) -> Flex<V> {
    Flex::new(contents, Direction::Vertical)
}

pub struct Flex<V> {
    contents:         V,
    layout:           taffy::Style,
    background_color: Color,
    border_color:     Color,
    corner_radii:     [f32; 4],
}

impl<V> Flex<V> {
    pub fn new(contents: V, direction: Direction) -> Self {
        let flex_direction = match direction {
            Direction::Horizontal => taffy::FlexDirection::Row,
            Direction::Vertical => taffy::FlexDirection::Column,
        };

        Self {
            contents,
            layout: taffy::Style {
                display: taffy::Display::Flex,
                flex_direction,
                ..Default::default()
            },
            background_color: Color::TRANSPARENT,
            border_color: Color::TRANSPARENT,
            corner_radii: [0.0; 4],
        }
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn corner(self, radius: f32) -> Self {
        self.corners(radius, radius, radius, radius)
    }

    pub fn corner_top_left(mut self, radius: f32) -> Self {
        self.corner_radii[0] = radius;
        self
    }

    pub fn corner_top_right(mut self, radius: f32) -> Self {
        self.corner_radii[1] = radius;
        self
    }

    pub fn corner_bottom_right(mut self, radius: f32) -> Self {
        self.corner_radii[2] = radius;
        self
    }

    pub fn corner_bottom_left(mut self, radius: f32) -> Self {
        self.corner_radii[3] = radius;
        self
    }

    pub fn corners(
        self,
        top_left: f32,
        top_right: f32,
        bottom_right: f32,
        bottom_left: f32,
    ) -> Self {
        self.corner_top_left(top_left)
            .corner_top_right(top_right)
            .corner_bottom_right(bottom_right)
            .corner_bottom_left(bottom_left)
    }
}

impl<V> Layout for Flex<V> {
    fn style_mut(&mut self) -> &mut taffy::Style {
        &mut self.layout
    }
}

impl<V> ContainerLayout for Flex<V> {}
impl<V> FlexLayout for Flex<V> {}
impl<V> BorderLayout for Flex<V> {}

impl<V> ViewMarker for Flex<V> {}
impl<P, T, V> View<Context<P>, T> for Flex<V>
where
    P: HasGroup,
    V: ViewSeq<Context<P>, T, BoxedWidget<P>>,
{
    type Element = Pod<P, Group<P>>;
    type State = V::State;

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let node = cx.new_layout_node(self.layout, &[]);

        let mut group = Group::new(cx);
        group.set_background_color(cx, self.background_color);
        group.set_border_color(cx, self.border_color);
        group.set_corner_radii(cx, self.corner_radii);

        let state = self.contents.seq_build(&mut group.elements(node), cx, data);

        let pod = Pod::new(node, group);

        (pod, state)
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        let _ = cx.set_layout_style(*element.node, self.layout);
        (element.widget).set_background_color(cx, self.background_color);
        (element.widget).set_border_color(cx, self.border_color);
        (element.widget).set_corner_radii(cx, self.corner_radii);

        self.contents.seq_rebuild(
            &mut element.widget.elements(*element.node),
            state,
            cx,
            data,
        );
    }

    fn message(
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(Lifecycle::Layout) = message.get() {
            element.widget.layout(cx, *element.node);
        }

        V::seq_message(
            &mut element.widget.elements(*element.node),
            state,
            cx,
            data,
            message,
        )
    }

    fn teardown(mut element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        V::seq_teardown(
            &mut element.widget.elements(element.node),
            state,
            cx,
        );

        element.widget.teardown(cx);
        let _ = cx.remove_layout_node(element.node);
    }
}
