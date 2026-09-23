use std::collections::VecDeque;

use ori::{Action, Message, Mut, Proxied, Proxy, Tracker, View, ViewId, ViewMarker};

use crate::{
    Context, Direction, Layout, LayoutStyle, Length, Padding, Platform, Sides, WidgetView,
    widget::WidgetMut, widgets::ListWidget,
};

/// [`View`] that can display a large scrollable list.
pub fn list<T, V>(count: usize, build: impl Fn(&T, usize) -> V) -> List<impl Fn(&T, usize) -> V> {
    List::new(count, build)
}

/// [`View`] that can display a large scrollable list.
pub struct List<F> {
    layout:    LayoutStyle,
    direction: Direction,
    padding:   Sides<Length>,
    gap:       f32,
    min_views: usize,
    buffer:    usize,
    count:     usize,
    build:     F,
}

impl<F> List<F> {
    /// Create new [`List`].
    pub fn new(count: usize, build: F) -> Self {
        Self {
            layout: LayoutStyle::default(),
            direction: Direction::Vertical,
            padding: Sides::all(Length::Length(0.0)),
            gap: 0.0,
            min_views: 16,
            buffer: 8,
            count,
            build,
        }
    }

    /// Set the direction.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Set the gap between items.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set the minimum number of active views.
    pub fn min_views(mut self, min_views: usize) -> Self {
        self.min_views = min_views;
        self
    }

    /// Set the number of buffer views at the start and end.
    pub fn buffer(mut self, buffer: usize) -> Self {
        self.buffer = buffer;
        self
    }
}

impl<F> Layout for List<F> {
    fn get_layout_style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.layout
    }
}

impl<F> Padding for List<F> {
    fn get_padding_mut(&mut self) -> &mut Sides<Length> {
        &mut self.padding
    }
}

enum ListMessage {
    Layout,
    Scrolled(f32, f32),
}

impl<F> ViewMarker for List<F> {}
impl<P, T, F, V> View<Context<P>, T> for List<F>
where
    P: Platform,
    F: Fn(&T, usize) -> V,
    V: WidgetView<P, T>,
{
    type Element = ListWidget<P, V::Element>;
    type State = ListState<P, T, F, V>;

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        // register on scroll callback
        let view_id = ViewId::next();
        cx.register(view_id);

        let on_scroll = {
            let proxy = cx.proxy();

            move |x, y| {
                proxy.message(Message::new(
                    ListMessage::Scrolled(x, y),
                    view_id,
                ));
            }
        };

        let on_layout = {
            let proxy = cx.proxy();

            move || {
                proxy.message(Message::new(
                    ListMessage::Layout,
                    view_id,
                ));
            }
        };

        let mut widget = ListWidget::new(
            cx,
            self.count,
            self.min_views,
            self.buffer,
            on_scroll,
            on_layout,
        );

        widget.set_layout(cx, self.layout);
        widget.set_direction(cx, self.direction);
        widget.set_padding(cx, self.padding);
        widget.set_gap(self.gap);

        let mut state = ListState {
            view_id,

            direction: self.direction,
            layout: self.layout,
            padding: self.padding,
            gap: self.gap,

            states: VecDeque::new(),
            build: self.build,
        };

        // initialize active views
        let count = widget.compute_active_view_count(0);
        state.build_active_back(&mut widget, cx, data, count);

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

        if state.padding != self.padding {
            state.padding = self.padding;
            element.set_padding(cx, self.padding);
        }

        if state.gap != self.gap {
            state.gap = self.gap;
            element.set_gap(self.gap);
        }

        state.build = self.build;

        element.resize(self.count);
        element.set_min_views(self.min_views);
        element.set_buffer(self.buffer);

        state.update_active_views(&mut element, cx, data);
        element.update_content_size(cx);
        state.rebuild_active_views(&mut element, cx, data);
        element.layout_active_views(cx);
    }

    fn message(
        mut element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        match message.take(state.view_id) {
            Some(ListMessage::Layout) => {
                state.update_active_views(&mut element, cx, data);
                element.layout_active_views(cx);

                return Action::new();
            }

            Some(ListMessage::Scrolled(x, y)) => {
                let offset = match state.direction {
                    Direction::Horizontal => x,
                    Direction::Vertical => y,
                };

                element.set_offset(offset);
                state.update_active_views(&mut element, cx, data);
                element.layout_active_views(cx);

                return Action::new();
            }

            None => {}
        }

        let mut action = Action::new();

        for (i, state) in state.states.iter_mut().enumerate() {
            if let Some((mut parent, child)) = element.get_active(i) {
                let widget = WidgetMut::new(&mut parent, child);
                action |= V::message(widget, state, cx, data, message);
            }
        }

        action
    }

    fn teardown(mut element: Self::Element, mut state: Self::State, cx: &mut Context<P>) {
        while let Some(child) = element.remove_back(cx)
            && let Some(state) = state.states.pop_back()
        {
            V::teardown(child, state, cx);
        }

        element.teardown(cx);
        cx.unregister(state.view_id);
    }
}

pub struct ListState<P, T, F, V>
where
    P: Platform,
    F: Fn(&T, usize) -> V,
    V: WidgetView<P, T>,
{
    view_id: ViewId,

    direction: Direction,
    layout:    LayoutStyle,
    padding:   Sides<Length>,
    gap:       f32,

    states: VecDeque<V::State>,
    build:  F,
}

impl<P, T, F, V> ListState<P, T, F, V>
where
    P: Platform,
    F: Fn(&T, usize) -> V,
    V: WidgetView<P, T>,
{
    fn update_active_views(
        &mut self,
        widget: &mut ListWidget<P, V::Element>,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        let start = widget.compute_start_index();
        let count = widget.compute_active_view_count(start);

        if widget.start() == start && widget.active_count() == count {
            return;
        }

        // if the difference is too big, don't bother reusing views
        if widget.start().abs_diff(start) >= count {
            widget.set_start(start);
            self.truncate_active_back(widget, cx, count);
            self.rebuild_active_views(widget, cx, data);
            self.build_active_back(widget, cx, data, count);
            return;
        }

        while widget.active_count() > count {
            if widget.start() < start {
                if let Some(child) = widget.remove_front(cx)
                    && let Some(state) = self.states.pop_front()
                {
                    V::teardown(child, state, cx);
                }
            } else if let Some(child) = widget.remove_back(cx)
                && let Some(state) = self.states.pop_back()
            {
                V::teardown(child, state, cx);
            }
        }

        while widget.active_count() < count {
            if widget.start() > start {
                self.build_front(widget, cx, data);
            } else {
                self.build_back(widget, cx, data);
            }
        }

        while widget.start() > start {
            self.rotate_backward(widget, cx, data);
        }

        while widget.start() < start {
            self.rotate_forward(widget, cx, data);
        }
    }

    fn rotate_backward(
        &mut self,
        widget: &mut ListWidget<P, V::Element>,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        if let Some(child) = widget.remove_back(cx) {
            widget.insert_front(cx, child);
        }

        let index = widget.start();

        if let Some((mut parent, child)) = widget.get_active(0)
            && let Some(mut state) = self.states.pop_back()
        {
            let view = (self.build)(data, index);
            let widget = WidgetMut::new(&mut parent, child);
            view.rebuild(widget, &mut state, cx, data);
            self.states.push_front(state);
        }
    }

    fn rotate_forward(
        &mut self,
        widget: &mut ListWidget<P, V::Element>,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        let index = widget.start() + widget.active_count();

        if let Some(child) = widget.remove_front(cx) {
            widget.insert_back(cx, child);
        }

        if let Some((mut parent, child)) = widget.get_active(widget.active_count() - 1)
            && let Some(mut state) = self.states.pop_front()
        {
            let view = (self.build)(data, index);
            let widget = WidgetMut::new(&mut parent, child);
            view.rebuild(widget, &mut state, cx, data);
            self.states.push_back(state);
        }
    }

    fn rebuild_active_views(
        &mut self,
        widget: &mut ListWidget<P, V::Element>,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        for (i, state) in self.states.iter_mut().enumerate() {
            let view = (self.build)(data, widget.start() + i);

            if let Some((mut parent, child)) = widget.get_active(i) {
                let widget = WidgetMut::new(&mut parent, child);
                view.rebuild(widget, state, cx, data);
            }
        }
    }

    fn build_active_back(
        &mut self,
        widget: &mut ListWidget<P, V::Element>,
        cx: &mut Context<P>,
        data: &mut T,
        count: usize,
    ) {
        while widget.active_count() < count {
            self.build_back(widget, cx, data);
        }
    }

    fn truncate_active_back(
        &mut self,
        widget: &mut ListWidget<P, V::Element>,
        cx: &mut Context<P>,
        count: usize,
    ) {
        while widget.active_count() > count {
            if let Some(child) = widget.remove_back(cx)
                && let Some(state) = self.states.pop_back()
            {
                V::teardown(child, state, cx);
            }
        }
    }

    fn build_front(
        &mut self,
        widget: &mut ListWidget<P, V::Element>,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        let index = widget.start() - 1;
        let view = (self.build)(data, index);
        let (element, state) = view.build(cx, data);
        widget.insert_front(cx, element);
        self.states.push_front(state);
    }

    fn build_back(
        &mut self,
        widget: &mut ListWidget<P, V::Element>,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        let index = widget.start() + widget.active_count();
        let view = (self.build)(data, index);
        let (element, state) = view.build(cx, data);
        widget.insert_back(cx, element);
        self.states.push_back(state);
    }
}
