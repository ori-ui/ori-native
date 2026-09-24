use ori::{Action, Message, Mut, Proxied, Proxy, Tracker, View, ViewId, ViewMarker};

use crate::{
    Context, Direction, Layout, LayoutStyle, Platform, WidgetView, widget::WidgetMut,
    widgets::ScrollWidget,
};

/// [`View`] of a horizontal scroll area.
pub fn hscroll<T, V>(contents: V) -> Scroll<T, V> {
    Scroll::new(contents, Direction::Horizontal)
}

/// [`View`] of a vertical scroll area.
pub fn vscroll<T, V>(contents: V) -> Scroll<T, V> {
    Scroll::new(contents, Direction::Vertical)
}

/// [`View`] of a scroll area.
#[allow(clippy::type_complexity)]
pub struct Scroll<T, V> {
    contents:  V,
    direction: Direction,
    layout:    LayoutStyle,
    on_scroll: Box<dyn FnMut(&mut T, f32) -> Action>,
}

impl<T, V> Scroll<T, V> {
    /// Create new [`Scroll`].
    pub fn new(contents: V, direction: Direction) -> Self {
        Self {
            contents,
            direction,
            layout: LayoutStyle {
                flex_shrink: 1.0,
                ..Default::default()
            },
            on_scroll: Box::new(|_, _| Action::new()),
        }
    }

    /// Set callback when the view is scrolled.
    pub fn on_scroll<A>(mut self, mut on_scroll: impl FnMut(&mut T, f32) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_scroll = Box::new(move |data, offset| on_scroll(data, offset).into());
        self
    }
}

impl<T, V> Layout for Scroll<T, V> {
    fn get_layout_style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.layout
    }
}

struct ScrollMessage(f32, f32);

impl<T, V> ViewMarker for Scroll<T, V> {}
impl<P, T, V> View<Context<P>, T> for Scroll<T, V>
where
    P: Platform,
    V: WidgetView<P, T>,
{
    type Element = ScrollWidget<P, V::Element>;
    type State = ScrollState<P, T, V>;

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let view_id = ViewId::next();
        cx.register(view_id);

        let on_scroll = {
            let proxy = cx.proxy();

            move |x, y| {
                proxy.message(Message::new(
                    ScrollMessage(x, y),
                    view_id,
                ));
            }
        };

        let mut widget = ScrollWidget::new(cx, contents, on_scroll);
        widget.set_layout(cx, self.layout);
        widget.set_direction(cx, self.direction);

        let state = ScrollState {
            view_id,
            state,
            direction: self.direction,
            layout: self.layout,
            on_scroll: self.on_scroll,
        };

        (widget, state)
    }

    fn rebuild(
        self,
        mut element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        if state.layout != self.layout {
            state.layout = self.layout;
            element.set_layout(cx, self.layout);
        }

        if state.direction != self.direction {
            state.direction = self.direction;
            element.set_direction(cx, self.direction);
        }

        state.on_scroll = self.on_scroll;

        let (mut parent, contents) = element.contents_mut();
        let widget = WidgetMut::new(&mut parent, contents);

        self.contents.rebuild(widget, &mut state.state, cx, data);
    }

    fn message(
        mut element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(ScrollMessage(x, y)) = message.take(state.view_id) {
            let scroll = match state.direction {
                Direction::Horizontal => x,
                Direction::Vertical => y,
            };

            return (state.on_scroll)(data, scroll);
        }

        let (mut parent, contents) = element.contents_mut();
        let widget = WidgetMut::new(&mut parent, contents);

        V::message(
            widget,
            &mut state.state,
            cx,
            data,
            message,
        )
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        let contents = element.teardown(cx);
        V::teardown(contents, state.state, cx);
        cx.unregister(state.view_id);
    }
}

#[allow(clippy::type_complexity)]
pub struct ScrollState<P, T, V>
where
    P: Platform,
    V: WidgetView<P, T>,
{
    view_id:   ViewId,
    state:     V::State,
    direction: Direction,
    layout:    LayoutStyle,
    on_scroll: Box<dyn FnMut(&mut T, f32) -> Action>,
}
