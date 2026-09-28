use ori::{Action, Message, Mut, Proxied, Proxy, Tracked, View, ViewId, ViewMarker};

use crate::{Context, Platform, Widget, WidgetView, widget::WidgetMut, widgets::MeasureWidget};

/// [`View`] that measures the position and size of its contents.
pub fn measure<T, V, A>(
    contents: V,
    on_measure: impl FnMut(&mut T, f32, f32, f32, f32) -> A + 'static,
) -> Measure<T, V>
where
    A: Into<Action>,
{
    Measure::new(contents, on_measure)
}

/// [`View`] that measures the position and size of its contents.
#[allow(clippy::type_complexity)]
pub struct Measure<T, V> {
    contents:   V,
    on_measure: Box<dyn FnMut(&mut T, f32, f32, f32, f32) -> Action>,
}

impl<T, V> Measure<T, V> {
    /// Create new [`Measure`].
    pub fn new<A>(
        contents: V,
        mut on_measure: impl FnMut(&mut T, f32, f32, f32, f32) -> A + 'static,
    ) -> Self
    where
        A: Into<Action>,
    {
        Self {
            contents,
            on_measure: Box::new(move |data, x, y, width, height| {
                on_measure(data, x, y, width, height).into()
            }),
        }
    }
}

enum MeasureMessage {
    PositionChanged(f32, f32),
}

impl<T, V> ViewMarker for Measure<T, V> {}
impl<P, T, V> View<Context<P>, T> for Measure<T, V>
where
    P: Platform,
    V: WidgetView<P, T>,
{
    type Element = MeasureWidget<P, V::Element>;
    type State = MeasureState<P, T, V>;

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let view_id = ViewId::next();
        cx.register(view_id);

        let on_position_changed = {
            let proxy = cx.proxy();

            move |x, y| {
                proxy.message(Message::new(
                    MeasureMessage::PositionChanged(x, y),
                    Some(view_id),
                ));
            }
        };

        let widget = MeasureWidget::new(cx, contents, on_position_changed);
        let state = MeasureState {
            state,
            view_id,
            on_measure: self.on_measure,
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
        state.on_measure = self.on_measure;

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
        let mut action = Action::new();

        if let Some(MeasureMessage::PositionChanged(x, y)) = message.take(state.view_id)
            && let Some(allocation) = cx.layout.get_allocation(element.layout_node())
        {
            action |= (state.on_measure)(
                data,
                x,
                y,
                allocation.size.width,
                allocation.size.height,
            );
        }

        let (mut parent, contents) = element.contents_mut();
        let widget = WidgetMut::new(&mut parent, contents);

        action
            | V::message(
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
pub struct MeasureState<P, T, V>
where
    P: Platform,
    V: WidgetView<P, T>,
{
    view_id:    ViewId,
    state:      V::State,
    on_measure: Box<dyn FnMut(&mut T, f32, f32, f32, f32) -> Action>,
}
