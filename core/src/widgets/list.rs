use std::{collections::VecDeque, time::Duration};

use ori::Element;

use crate::{
    Allocation, Context, Direction, FlexStyle, LayoutNode, LayoutStyle, Length, Overflow, Parent,
    Platform, Position, Sides, Size, Widget, WidgetMut,
    native::{NativeGroup, NativeScroll},
};

/// A [`Widget`] for virtualized lists.
pub struct ListWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    group:         P::Group,
    scroll:        P::Scroll,
    group_layout:  LayoutNode,
    scroll_layout: LayoutNode,

    direction: Direction,
    gap:       f32,
    min_views: usize,
    buffer:    usize,
    on_layout: Box<dyn Fn()>,

    sizes:        Vec<Option<f32>>,
    window_size:  f32,
    average_size: f32,
    content_size: f32,

    offset:   f32,
    start:    usize,
    children: VecDeque<ListChild<W>>,

    scroll_allocation: Option<Allocation>,
    group_allocation:  Option<Allocation>,
}

struct ListChild<W> {
    layout: LayoutNode,
    offset: f32,
    widget: W,

    allocation: Option<Allocation>,
}

impl<P, W> ListWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    /// Create new [`ListWidget`].
    pub fn new(
        cx: &mut Context<P>,
        count: usize,
        min_views: usize,
        buffer: usize,
        on_scroll: impl Fn(f32, f32) + 'static,
        on_layout: impl Fn() + 'static,
    ) -> Self {
        let group = P::Group::build(&mut cx.platform);
        let scroll = P::Scroll::build(
            &mut cx.platform,
            group.widget_ref(),
            on_scroll,
        );

        let group_layout = cx.layout.add_node(&[]);
        let scroll_layout = cx.layout.add_node(&[group_layout]);

        Self {
            group,
            scroll,
            group_layout,
            scroll_layout,

            direction: Direction::Vertical,
            gap: 0.0,
            min_views,
            buffer,
            on_layout: Box::new(on_layout),

            sizes: vec![None; count],
            window_size: 0.0,
            average_size: 0.0,
            content_size: 0.0,

            offset: 0.0,
            start: 0,
            children: VecDeque::new(),

            scroll_allocation: None,
            group_allocation: None,
        }
    }

    /// Teardown the widget.
    pub fn teardown(self, cx: &mut Context<P>) {
        self.group.teardown(&mut cx.platform);
        self.scroll.teardown(&mut cx.platform);
        cx.layout.remove_node(self.group_layout);
        cx.layout.remove_node(self.scroll_layout);
    }

    /// Set the [`LayoutStyle`].
    pub fn set_layout(&mut self, cx: &mut Context<P>, layout: LayoutStyle) {
        cx.layout.set_layout(self.scroll_layout, layout);
    }

    /// Set the `padding`.
    pub fn set_padding(&mut self, cx: &mut Context<P>, padding: Sides<Length>) {
        cx.layout.set_padding(self.scroll_layout, padding);
    }

    /// Set the [`Direction`].
    pub fn set_direction(&mut self, cx: &mut Context<P>, direction: Direction) {
        self.direction = direction;

        // set direction of scroll
        self.scroll.set_direction(&mut cx.platform, direction);

        // set layout direction of scroll
        cx.layout.set_flex(
            self.scroll_layout,
            Self::scroll_flex(direction),
        );

        // set layout overflow of scroll
        cx.layout.set_overflow(
            self.scroll_layout,
            Self::scroll_overflow(direction),
        );

        // set content layout
        cx.layout.set_layout(
            self.group_layout,
            Self::content_layout(direction, self.content_size),
        );

        // set content flex
        cx.layout.set_flex(
            self.group_layout,
            Self::content_flex(direction),
        );

        // set the layout of each child
        for child in self.children.iter() {
            cx.layout.set_flex(
                child.layout,
                Self::child_flex(direction),
            );

            cx.layout.set_layout(
                child.layout,
                Self::child_layout(direction),
            );
        }
    }

    /// Get the total number of elements in the list.
    pub fn count(&self) -> usize {
        self.sizes.len()
    }

    /// Get the `start` index of window of active widgets.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Get the number `active` widgets.
    pub fn active_count(&self) -> usize {
        self.children.len()
    }

    /// Set the start index of the `active` window.
    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }

    /// Set the scroll `offset`.
    pub fn set_offset(&mut self, offset: f32) {
        self.offset = offset;
    }

    /// Se the `gap` between items.
    pub fn set_gap(&mut self, gap: f32) {
        self.gap = gap;
    }

    /// Set the minimum number of views in the `active` window.
    pub fn set_min_views(&mut self, min_views: usize) {
        self.min_views = min_views;
    }

    /// Set the number of `buffer` items before and after the visible window.
    pub fn set_buffer(&mut self, buffer: usize) {
        self.buffer = buffer;
    }

    /// Set the total number of elements.
    pub fn resize(&mut self, count: usize) {
        self.sizes.resize(count, None);
    }

    /// Get an `active` widget.
    pub fn get_active(&mut self, index: usize) -> Option<(impl Parent<P>, &mut W)> {
        let child = self.children.get_mut(index)?;

        let parent = ListParent {
            index,
            group: &mut self.group,
            layout: child.layout,
        };

        Some((parent, &mut child.widget))
    }

    /// Insert a widget at the front of the `active` window.
    pub fn insert_front(&mut self, cx: &mut Context<P>, child: W) {
        (self.group).insert_child(&mut cx.platform, 0, child.widget_ref());

        let layout = cx.layout.add_node(&[child.layout_node()]);
        cx.layout.set_layout(
            layout,
            Self::child_layout(self.direction),
        );

        cx.layout.set_flex(layout, Self::child_flex(self.direction));
        cx.layout.insert_child(self.group_layout, 0, layout);

        let child = ListChild {
            layout,
            offset: 0.0,
            allocation: None,
            widget: child,
        };

        self.children.push_front(child);
        self.start -= 1;
    }

    /// Insert a widget at the back of the `active` window.
    pub fn insert_back(&mut self, cx: &mut Context<P>, child: W) {
        let index = self.children.len();
        self.group.insert_child(
            &mut cx.platform,
            index,
            child.widget_ref(),
        );

        let layout = cx.layout.add_node(&[child.layout_node()]);
        cx.layout.set_layout(
            layout,
            Self::child_layout(self.direction),
        );

        cx.layout.set_flex(layout, Self::child_flex(self.direction));
        cx.layout.insert_child(self.group_layout, index, layout);

        let child = ListChild {
            layout,
            offset: 0.0,
            allocation: None,
            widget: child,
        };

        self.children.push_back(child);
    }

    /// Remove a widget at the front of the `active` window.
    pub fn remove_front(&mut self, cx: &mut Context<P>) -> Option<W> {
        let child = self.children.pop_front()?;

        self.group.remove_child(&mut cx.platform, 0);
        cx.layout.remove_child(self.group_layout, 0);
        cx.layout.remove_node(child.layout);

        self.start += 1;

        Some(child.widget)
    }

    /// Remove a widget at the back of the `active` window.
    pub fn remove_back(&mut self, cx: &mut Context<P>) -> Option<W> {
        let child = self.children.pop_back()?;

        (self.group).remove_child(&mut cx.platform, self.children.len());
        (cx.layout).remove_child(self.group_layout, self.children.len());
        cx.layout.remove_node(child.layout);

        Some(child.widget)
    }

    fn compute_average_size(&self) -> f32 {
        let size_sum = self.sizes.iter().flatten().copied().sum::<f32>();
        let size_count = self.sizes.iter().flatten().count() as f32;

        size_sum / size_count
    }

    fn compute_active_view_offset(&self) -> f32 {
        let gap_sum = self.gap * self.start as f32;
        let size_sum = (0..self.start)
            .map(|i| self.get_size_estimate(i))
            .sum::<f32>();

        size_sum + gap_sum
    }

    fn compute_content_size(&self) -> f32 {
        let gap_sum = self.gap * self.count().saturating_sub(1) as f32;
        let size_sum = (0..self.count())
            .map(|i| self.get_size_estimate(i))
            .sum::<f32>();

        size_sum + gap_sum
    }

    fn get_size_estimate(&self, index: usize) -> f32 {
        self.sizes[index].unwrap_or(self.average_size)
    }

    /// Update the average size of elements.
    pub fn update_average_size(&mut self) {
        self.average_size = self.compute_average_size();
    }

    /// Update the total size of the list.
    pub fn update_content_size(&mut self, cx: &mut Context<P>) {
        let content_size = self.compute_content_size();

        if self.content_size != content_size {
            self.content_size = content_size;
            self.set_content_size(cx, content_size);
        }
    }

    fn set_content_size(&mut self, cx: &mut Context<P>, content_size: f32) {
        self.content_size = content_size;

        cx.layout.set_layout(
            self.group_layout,
            Self::content_layout(self.direction, self.content_size),
        );
    }

    /// Compute the number of widgets in the `active` window.
    pub fn compute_active_view_count(&self, start: usize) -> usize {
        let mut offset = self.compute_active_view_offset();
        let remaining = self.count().saturating_sub(start);

        for i in start..self.count() {
            if offset >= self.offset + self.window_size {
                let size = (i - start).max(self.min_views + self.buffer);
                return remaining.min(size + self.buffer);
            }

            offset += self.get_size_estimate(i) + self.gap;
        }

        remaining
    }

    /// Compute the start index of the `active` window.
    pub fn compute_start_index(&mut self) -> usize {
        let mut offset = 0.0;

        for i in 0..self.count() {
            offset += self.get_size_estimate(i) + self.gap;

            if offset >= self.offset {
                return i.saturating_sub(self.buffer);
            }
        }

        self.count().saturating_sub(self.buffer)
    }

    /// Layout the `active` widgets.
    pub fn layout_active_views(&mut self, cx: &mut Context<P>) {
        let mut offset = self.compute_active_view_offset();

        for (i, child) in self.children.iter_mut().enumerate() {
            if let Some(allocation) = cx.layout.get_allocation(child.widget.layout_node()) {
                let size = Self::allocation_size(self.direction, allocation);
                self.sizes[self.start + i] = Some(size);

                if child.allocation != Some(allocation) || child.offset != offset {
                    child.allocation = Some(allocation);
                    child.offset = offset;

                    let x_offset = match self.direction {
                        Direction::Horizontal => offset,
                        Direction::Vertical => 0.0,
                    };

                    let y_offset = match self.direction {
                        Direction::Horizontal => 0.0,
                        Direction::Vertical => offset,
                    };

                    self.group.set_child_layout(
                        &mut cx.platform,
                        i,
                        allocation.x + x_offset,
                        allocation.y + y_offset,
                        allocation.size.width,
                        allocation.size.height,
                    );
                }

                offset += size + self.gap;
            }
        }
    }

    fn layout_changed(&mut self, cx: &mut Context<P>) -> bool {
        let mut changed = false;

        for (i, child) in self.children.iter_mut().enumerate() {
            if let Some(allocation) = cx.layout.get_allocation(child.widget.layout_node()) {
                let size = Self::allocation_size(self.direction, allocation);

                if self.sizes[self.start + i] != Some(size) {
                    self.sizes[self.start + i] = Some(size);
                    changed = true;
                }
            }
        }

        changed
    }

    fn allocation_size(direction: Direction, allocation: Allocation) -> f32 {
        match direction {
            Direction::Horizontal => {
                allocation.size.width + allocation.margin.left + allocation.margin.right
            }

            Direction::Vertical => {
                allocation.size.height + allocation.margin.top + allocation.margin.bottom
            }
        }
    }

    fn content_layout(direction: Direction, size: f32) -> LayoutStyle {
        let size = match direction {
            Direction::Horizontal => Size {
                width:  Some(Length::Length(size)),
                height: Some(Length::Fract(1.0)),
            },

            Direction::Vertical => Size {
                width:  Some(Length::Fract(1.0)),
                height: Some(Length::Length(size)),
            },
        };

        LayoutStyle {
            min_size: size,
            max_size: size,
            ..Default::default()
        }
    }

    fn content_flex(direction: Direction) -> FlexStyle {
        FlexStyle {
            direction,
            ..Default::default()
        }
    }

    fn scroll_overflow(direction: Direction) -> Size<Overflow> {
        match direction {
            Direction::Horizontal => Size {
                width:  Overflow::Hidden,
                height: Overflow::Visible,
            },

            Direction::Vertical => Size {
                width:  Overflow::Visible,
                height: Overflow::Hidden,
            },
        }
    }

    fn scroll_flex(direction: Direction) -> FlexStyle {
        FlexStyle {
            direction,
            ..Default::default()
        }
    }

    fn child_layout(direction: Direction) -> LayoutStyle {
        let inset = match direction {
            Direction::Horizontal => Sides {
                left:   None,
                right:  None,
                top:    Some(Length::Length(0.0)),
                bottom: Some(Length::Length(0.0)),
            },

            Direction::Vertical => Sides {
                left:   Some(Length::Length(0.0)),
                right:  Some(Length::Length(0.0)),
                top:    None,
                bottom: None,
            },
        };

        LayoutStyle {
            position: Position::Absolute,
            inset,
            ..Default::default()
        }
    }

    fn child_flex(direction: Direction) -> FlexStyle {
        FlexStyle {
            direction,
            ..Default::default()
        }
    }
}

impl<P, W> Element for ListWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    type Mut<'a>
        = WidgetMut<'a, P, Self>
    where
        Self: 'a;
}

impl<P, W> Widget<P> for ListWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    fn widget_ref(&self) -> P::WidgetRef {
        self.scroll.widget_ref()
    }

    fn layout_node(&self) -> LayoutNode {
        self.scroll_layout
    }

    fn layout(&mut self, cx: &mut Context<P>) {
        (self.on_layout)();

        // check for layout changes
        if self.layout_changed(cx) {
            self.update_average_size();
            self.update_content_size(cx);
        }

        // update layout of scroll contents
        if let Some(allocation) = cx.layout.get_allocation(self.group_layout)
            && self.group_allocation != Some(allocation)
        {
            self.scroll.set_content_layout(
                &mut cx.platform,
                allocation.x,
                allocation.y,
                allocation.size.width,
                allocation.size.height,
            );
        }

        if let Some(allocation) = cx.layout.get_allocation(self.scroll_layout)
            && self.scroll_allocation != Some(allocation)
        {
            self.window_size = match self.direction {
                Direction::Horizontal => allocation.size.width,
                Direction::Vertical => allocation.size.height,
            };

            self.scroll.set_content_size(
                &mut cx.platform,
                allocation.content_size.width,
                allocation.content_size.height,
            );
        }

        self.scroll_allocation = cx.layout.get_allocation(self.scroll_layout);
        self.group_allocation = cx.layout.get_allocation(self.group_layout);

        self.layout_active_views(cx);

        for child in &mut self.children {
            child.widget.layout(cx);
        }
    }

    fn animate(&mut self, cx: &mut Context<P>, dt: Duration) {
        for view in &mut self.children {
            view.widget.animate(cx, dt);
        }
    }
}

struct ListParent<'a, P>
where
    P: Platform,
{
    index:  usize,
    group:  &'a mut P::Group,
    layout: LayoutNode,
}

impl<'a, P> Parent<P> for ListParent<'a, P>
where
    P: Platform,
{
    fn replace_child(&mut self, cx: &mut Context<P>, widget: P::WidgetRef, layout: LayoutNode) {
        (self.group).replace_child(&mut cx.platform, self.index, widget);
        cx.layout.replace_child(self.layout, 0, layout);
    }
}
