use ori::{Action, Message, Mut, View, ViewMarker};

use crate::{
    Context, Direction, Layout, Lifecycle, NativeWidget, Pod, WidgetView,
    native::{HasScroll, NativeScroll},
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
    V: WidgetView<P, T>,
{
    type Element = Pod<P, P::Scroll>;
    type State = (V::Element, V::State);

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);
        let node = cx.new_layout_node(self.style, &[contents.node]);

        let mut widget = P::Scroll::build(
            &mut cx.platform,
            contents.widget.widget(),
        );

        widget.set_direction(self.direction);

        let pod = Pod::new(node, widget);

        (pod, (contents, state))
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        (contents, state): &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        let _ = cx.set_layout_style(*element.node, self.style);
        element.widget.set_direction(self.direction);

        self.contents.rebuild(
            contents.as_mut(*element.node, element.widget, 0),
            state,
            cx,
            data,
        );
    }

    fn message(
        element: Mut<'_, Self::Element>,
        (contents, state): &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(Lifecycle::Layout) = message.get()
            && let Ok(layout) = cx.get_computed_layout(*element.node)
        {
            (element.widget).set_size(layout.size.width, layout.size.height);
        }

        V::message(
            contents.as_mut(*element.node, element.widget, 0),
            state,
            cx,
            data,
            message,
        )
    }

    fn teardown(element: Self::Element, (contents, state): Self::State, cx: &mut Context<P>) {
        V::teardown(contents, state, cx);
        element.widget.teardown(&mut cx.platform);
    }
}
