use ori::{Action, Message, Mut, Proxied, Proxy, Tracked, View, ViewId, ViewMarker};

use crate::{
    Context, LayoutRequest, Platform, Point, PopupPosition, Side, Widget, WidgetMut, WidgetView,
    widgets::PopupWidget,
};

/// A [`View`] that shows a popup relative to an anchor.
pub fn popup<T, V, W>(anchor: V, contents: Option<W>) -> Popup<T, V, W> {
    Popup::new(anchor, contents)
}

/// A [`View`] that shows a popup relative to an anchor.
pub struct Popup<T, V, W> {
    anchor:     V,
    contents:   Option<W>,
    position:   PopupPosition,
    on_dismiss: Option<OnDismiss<T>>,
}

impl<T, V, W> Popup<T, V, W> {
    /// Create new [`Popup`].
    pub fn new(anchor: V, contents: Option<W>) -> Self {
        Popup {
            anchor,
            contents,
            position: PopupPosition::Relative(Side::Bottom),
            on_dismiss: None,
        }
    }

    /// Set which side the popup is anchored to.
    pub fn side(mut self, side: Side) -> Self {
        self.position = PopupPosition::Relative(side);
        self
    }

    /// Place the popup at a specific position in the coordinate space of the anchor.
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position = PopupPosition::Absolute(Point { x, y });
        self
    }

    /// Set the callback for when the popup is dismissed.
    ///
    /// This will also make the popup modal, if not set the popup will not close automatically.
    pub fn on_dismiss<A>(mut self, mut on_dismiss: impl FnMut(&mut T) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_dismiss = Some(Box::new(move |data| {
            on_dismiss(data).into()
        }));
        self
    }
}

type OnDismiss<T> = Box<dyn FnMut(&mut T) -> Action>;

enum PopupMessage {
    Dismissed,
}

pub struct PopupState<P, T, V, W>
where
    P: Platform,
    V: WidgetView<P, T>,
    W: WidgetView<P, T>,
{
    view_id:        ViewId,
    position:       PopupPosition,
    on_dismiss:     Option<OnDismiss<T>>,
    anchor_state:   V::State,
    contents_state: Option<W::State>,
}

impl<T, V, W> ViewMarker for Popup<T, V, W> {}
impl<P, T, V, W> View<Context<P>, T> for Popup<T, V, W>
where
    P: Platform,
    V: WidgetView<P, T>,
    W: WidgetView<P, T>,
{
    type Element = PopupWidget<P, V::Element, W::Element>;
    type State = PopupState<P, T, V, W>;

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let view_id = ViewId::next();
        cx.register(view_id);

        let (anchor, anchor_state) = self.anchor.build(cx, data);

        let on_dismiss = {
            let proxy = cx.proxy();

            move || {
                proxy.message(Message::new(
                    PopupMessage::Dismissed,
                    view_id,
                ));
            }
        };

        let mut widget = PopupWidget::new(cx, anchor, on_dismiss);
        widget.set_position(cx, self.position);
        widget.set_modal(cx, self.on_dismiss.is_some());

        let mut state = PopupState {
            view_id,
            position: self.position,
            on_dismiss: self.on_dismiss,
            anchor_state,
            contents_state: None,
        };

        if let Some(contents) = self.contents {
            let (contents, contents_state) = contents.build(cx, data);
            state.contents_state = Some(contents_state);
            widget.open(cx, contents);
        }

        (widget, state)
    }

    fn rebuild(
        self,
        mut element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        {
            let (mut parent, anchor) = element.anchor_mut();
            let widget = WidgetMut::new(&mut parent, anchor);

            self.anchor.rebuild(
                widget,
                &mut state.anchor_state,
                cx,
                data,
            );
        }

        if let Some(contents) = self.contents {
            if let Some((mut parent, element)) = element.contents_mut()
                && let Some(ref mut state) = state.contents_state
            {
                let widget = WidgetMut::new(&mut parent, element);
                contents.rebuild(widget, state, cx, data);
            } else {
                let (contents, contents_state) = contents.build(cx, data);
                state.contents_state = Some(contents_state);

                cx.layout.insert_root(contents.layout_node(), state.view_id);

                element.open(cx, contents);
                element.layout(cx);
            }
        } else if let Some(state) = state.contents_state.take()
            && let Some(contents) = element.close(cx)
        {
            W::teardown(contents, state, cx);
        }

        if state.position != self.position {
            state.position = self.position;
            element.set_position(cx, self.position);
        }

        if state.on_dismiss.is_some() != self.on_dismiss.is_some() {
            element.set_modal(cx, self.on_dismiss.is_some());
        }

        state.on_dismiss = self.on_dismiss;
    }

    fn message(
        mut element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(LayoutRequest::Layout) = message.take(state.view_id) {
            element.layout(cx);
            return Action::new();
        }

        if let Some(PopupMessage::Dismissed) = message.take(state.view_id)
            && let Some(ref mut on_dismiss) = state.on_dismiss
        {
            return on_dismiss(data);
        }

        let mut action = Action::new();

        {
            let (mut parent, anchor) = element.anchor_mut();
            let widget = WidgetMut::new(&mut parent, anchor);
            let state = &mut state.anchor_state;
            action |= V::message(widget, state, cx, data, message);
        }

        if let Some((mut parent, contents)) = element.contents_mut()
            && let Some(ref mut state) = state.contents_state
        {
            let widget = WidgetMut::new(&mut parent, contents);
            action |= W::message(widget, state, cx, data, message);
        }

        action
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        let (anchor, contents) = element.teardown(cx);
        V::teardown(anchor, state.anchor_state, cx);

        if let Some(contents) = contents
            && let Some(state) = state.contents_state
        {
            W::teardown(contents, state, cx);
        }

        cx.unregister(state.view_id);
    }
}
