use ori::{Action, Message, Mut, View, ViewMarker};

use crate::{Affine, Context, Platform, WidgetView, widget::WidgetMut, widgets::TransformWidget};

/// [`View`] that transforms its contents.
pub fn transform<V>(contents: V) -> Transform<V> {
    Transform::new(contents)
}

/// [`View`] that transforms its contents.
pub struct Transform<V> {
    contents: V,
    affine:   Affine,
}

impl<V> Transform<V> {
    /// Create new [`Transform`].
    pub fn new(contents: V) -> Self {
        Self {
            contents,
            affine: Affine::default(),
        }
    }

    /// Translate the contents by `x` and `y`.
    pub fn translate(mut self, x: f32, y: f32) -> Self {
        self.affine.offset_x = x;
        self.affine.offset_y = y;
        self
    }

    /// Rotate the contents by `degrees`.
    pub fn rotate(mut self, degrees: f32) -> Self {
        self.affine.rotation = degrees;
        self
    }

    /// Scale the contents by `sx` and `sy`.
    pub fn scale(mut self, sx: f32, sy: f32) -> Self {
        self.affine.scale_x = sx;
        self.affine.scale_y = sy;
        self
    }
}

impl<V> ViewMarker for Transform<V> {}
impl<P, V, T> View<Context<P>, T> for Transform<V>
where
    P: Platform,
    V: WidgetView<P, T>,
{
    type Element = TransformWidget<P, V::Element>;
    type State = TransformState<P, T, V>;

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);
        let mut widget = TransformWidget::new(cx, contents);
        widget.set_transform(cx, self.affine);

        let state = TransformState {
            state,
            affine: self.affine,
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
        {
            let (mut parent, contents) = element.contents_mut();
            let widget = WidgetMut::new(&mut parent, contents);
            self.contents.rebuild(widget, &mut state.state, cx, data);
        }

        if state.affine != self.affine {
            state.affine = self.affine;
            element.set_transform(cx, state.affine);
        }
    }

    fn message(
        mut element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
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
        let element = element.teardown(cx);
        V::teardown(element, state.state, cx);
    }
}

pub struct TransformState<P, T, V>
where
    P: Platform,
    V: WidgetView<P, T>,
{
    state:  V::State,
    affine: Affine,
}
