use ori::{Action, Message, Mut, View, ViewMarker, ViewSeq};
use taffy::FlexDirection;

use crate::{
    AnyShadow, Color, Context, FlexContainer, FlexItem, Layout, LayoutContainer, Pod,
    native::HasGroup, shadows::GroupShadow,
};

pub fn row<V>(contents: V) -> Flex<V> {
    Flex::new(contents, FlexDirection::Row)
}

pub fn column<V>(contents: V) -> Flex<V> {
    Flex::new(contents, FlexDirection::Column)
}

pub struct Flex<V> {
    contents:         V,
    style:            taffy::Style,
    background_color: Color,
    border_color:     Color,
    corner_radii:     [f32; 4],
}

impl<V> Flex<V> {
    pub fn new(contents: V, direction: FlexDirection) -> Self {
        Self {
            contents,
            style: taffy::Style {
                display: taffy::Display::Flex,
                flex_direction: direction,
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

    pub fn corners(
        mut self,
        top_left: f32,
        top_right: f32,
        bottom_right: f32,
        bottom_left: f32,
    ) -> Self {
        self.corner_radii = [top_left, top_right, bottom_right, bottom_left];
        self
    }
}

impl<V> Layout for Flex<V> {
    fn style_mut(&mut self) -> &mut taffy::Style {
        &mut self.style
    }
}

impl<V> LayoutContainer for Flex<V> {}
impl<V> FlexItem for Flex<V> {}
impl<V> FlexContainer for Flex<V> {}

impl<V> ViewMarker for Flex<V> {}
impl<P, T, V> View<Context<P>, T> for Flex<V>
where
    P: HasGroup,
    V: ViewSeq<Context<P>, T, AnyShadow<P>>,
{
    type Element = Pod<GroupShadow<P>>;
    type State = V::State;

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let node = cx.new_layout_node(self.style, &[]);

        let mut shadow = GroupShadow::new(cx);
        shadow.set_background_color(cx, self.background_color);
        shadow.set_border_color(cx, self.border_color);
        shadow.set_corner_radii(cx, self.corner_radii);

        let state = self
            .contents
            .seq_build(&mut shadow.elements(node), cx, data);

        let pod = Pod { node, shadow };

        (pod, state)
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        let _ = cx.set_layout_style(*element.node, self.style);
        (element.shadow).set_background_color(cx, self.background_color);
        (element.shadow).set_border_color(cx, self.border_color);
        (element.shadow).set_corner_radii(cx, self.corner_radii);

        self.contents.seq_rebuild(
            &mut element.shadow.elements(*element.node),
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
        V::seq_message(
            &mut element.shadow.elements(*element.node),
            state,
            cx,
            data,
            message,
        )
    }

    fn teardown(mut element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        V::seq_teardown(
            &mut element.shadow.elements(element.node),
            state,
            cx,
        );

        element.shadow.teardown(cx);
        let _ = cx.remove_layout_node(element.node);
    }
}
