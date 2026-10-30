use ori::{Action, Message, Mut, View, ViewMarker};

use crate::{Canvas, Context, LayoutStyle, Platform, StyleLayout, widgets::PainterWidget};

/// [`View`] that draws a [`Canvas`].
pub fn paint(canvas: Canvas) -> Painter {
    Painter::new(canvas)
}

/// [`View`] that draws a [`Canvas`].
pub struct Painter {
    canvas: Canvas,
    layout: LayoutStyle,
}

impl Painter {
    /// Create new [`Painter`].
    pub fn new(canvas: Canvas) -> Self {
        Self {
            canvas,
            layout: Default::default(),
        }
    }
}

impl StyleLayout for Painter {
    fn get_layout_style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.layout
    }
}

pub struct PainterState {
    canvas: Canvas,
    layout: LayoutStyle,
}

impl ViewMarker for Painter {}
impl<P, T> View<Context<P>, T> for Painter
where
    P: Platform,
{
    type Element = PainterWidget<P>;
    type State = PainterState;

    fn build(self, cx: &mut Context<P>, _data: &mut T) -> (Self::Element, Self::State) {
        let mut widget = PainterWidget::new(cx);

        widget.set_layout(cx, self.layout);
        widget.set_canvas(cx, self.canvas.clone());

        let state = PainterState {
            canvas: self.canvas,
            layout: self.layout,
        };

        (widget, state)
    }

    fn rebuild(
        self,
        mut element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        _data: &mut T,
    ) {
        if state.layout != self.layout {
            state.layout = self.layout;
            element.set_layout(cx, self.layout);
        }

        if state.canvas != self.canvas {
            state.canvas = self.canvas.clone();
            element.set_canvas(cx, self.canvas);
        }
    }

    fn message(
        _element: Mut<'_, Self::Element>,
        _state: &mut Self::State,
        _cx: &mut Context<P>,
        _data: &mut T,
        _message: &mut Message,
    ) -> Action {
        Action::new()
    }

    fn teardown(element: Self::Element, _state: Self::State, cx: &mut Context<P>) {
        element.teardown(cx);
    }
}
