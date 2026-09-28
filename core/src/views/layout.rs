use ori::{Action, Message, Mut, Proxied, Proxy, Tracked, View, ViewId, ViewMarker};

use crate::{Context, Platform, Size, WidgetMut, WidgetView, widgets::LayoutWidget};

/// [`View`] with a callback when layout changes.
pub fn on_layout<T, V, A>(
    contents: V,
    on_layout: impl FnMut(&mut T, f32, f32) -> A,
) -> Layout<V, impl FnMut(&mut T, f32, f32) -> A> {
    Layout::new(contents, on_layout)
}

/// [`View`] with a callback when layout changes.
pub struct Layout<V, F> {
    contents:  V,
    on_layout: F,
}

impl<V, F> Layout<V, F> {
    /// Create new [`Layout`].
    pub fn new(contents: V, on_layout: F) -> Self {
        Self {
            contents,
            on_layout,
        }
    }
}

struct LayoutMessage(Size<f32>);

impl<V, F> ViewMarker for Layout<V, F> {}
impl<P, T, V, F, A> View<Context<P>, T> for Layout<V, F>
where
    P: Platform,
    V: WidgetView<P, T>,
    F: FnMut(&mut T, f32, f32) -> A,
    A: Into<Action>,
{
    type Element = LayoutWidget<P, V::Element>;
    type State = LayoutState<P, T, V, F>;

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let view_id = ViewId::next();
        cx.register(view_id);

        let widget = LayoutWidget::new(contents, {
            let proxy = cx.proxy();
            move |size| {
                proxy.message(Message::new(
                    LayoutMessage(size),
                    view_id,
                ));
            }
        });

        let state = LayoutState {
            view_id,
            state,
            on_layout: self.on_layout,
        };

        (widget, state)
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        let widget = WidgetMut::new(
            element.parent,
            element.widget.contents(),
        );

        self.contents.rebuild(widget, &mut state.state, cx, data);
        state.on_layout = self.on_layout;
    }

    fn message(
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(LayoutMessage(size)) = message.take(state.view_id) {
            return (state.on_layout)(data, size.width, size.height).into();
        }

        let widget = WidgetMut::new(
            element.parent,
            element.widget.contents(),
        );

        V::message(
            widget,
            &mut state.state,
            cx,
            data,
            message,
        )
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        let contents = element.teardown();
        V::teardown(contents, state.state, cx);
        cx.unregister(state.view_id);
    }
}

pub struct LayoutState<P, T, V, F>
where
    P: Platform,
    V: WidgetView<P, T>,
{
    view_id:   ViewId,
    state:     V::State,
    on_layout: F,
}
