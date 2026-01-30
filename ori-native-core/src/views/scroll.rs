use ori::{Action, Message, Mut, View, ViewMarker};

use crate::{
    Context, Direction, Layout, Lifecycle, Pod, ShadowView,
    native::{HasScroll, NativeScroll},
    shadows::ScrollShadow,
};

pub fn hscroll<V>(contents: V) -> Scroll<V> {
    Scroll::new(contents, Direction::Horizontal)
}

pub fn vscroll<V>(contents: V) -> Scroll<V> {
    Scroll::new(contents, Direction::Vertical)
}

pub struct Scroll<V> {
    contents:  V,
    style:     taffy::Style,
    direction: Direction,
}

impl<V> Scroll<V> {
    pub fn new(contents: V, direction: Direction) -> Self {
        let flex_direction = match direction {
            Direction::Horizontal => taffy::FlexDirection::Row,
            Direction::Vertical => taffy::FlexDirection::Column,
        };

        let overflow_x = match direction {
            Direction::Horizontal => taffy::Overflow::Scroll,
            Direction::Vertical => taffy::Overflow::Hidden,
        };

        let overflow_y = match direction {
            Direction::Horizontal => taffy::Overflow::Hidden,
            Direction::Vertical => taffy::Overflow::Scroll,
        };

        Self {
            contents,
            style: taffy::Style {
                display: taffy::Display::Flex,
                overflow: taffy::Point {
                    x: overflow_x,
                    y: overflow_y,
                },
                flex_direction,
                ..Default::default()
            },
            direction,
        }
    }
}

impl<V> Layout for Scroll<V> {
    fn style_mut(&mut self) -> &mut taffy::Style {
        &mut self.style
    }
}

impl<V> ViewMarker for Scroll<V> {}
impl<P, T, V> View<Context<P>, T> for Scroll<V>
where
    P: HasScroll,
    V: ShadowView<P, T>,
{
    type Element = Pod<ScrollShadow<P, V::Shadow>>;
    type State = V::State;

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);
        let node = cx.new_layout_node(self.style, &[contents.node]);

        let mut shadow = ScrollShadow::new(cx, contents);
        shadow.set_direction(self.direction);

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
        element.shadow.set_direction(self.direction);

        self.contents.rebuild(
            element.shadow.element(*element.node),
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
        if let Some(Lifecycle::Layout) = message.get()
            && let Ok(layout) = cx.get_computed_layout(*element.node)
        {
            (element.shadow).set_size(layout.size.width, layout.size.height);
        }

        V::message(
            element.shadow.element(*element.node),
            state,
            cx,
            data,
            message,
        )
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        V::teardown(element.shadow.contents, state, cx);
        element.shadow.scroll.teardown(&mut cx.platform);
    }
}
