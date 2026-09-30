use std::borrow::Cow;

use ori::{Action, Message, Mut, View, ViewMarker};

use crate::{Color, Context, LayoutStyle, Platform, StyleLayout, widgets::ImageWidget};

/// [`View`] of an image.
pub fn image(data: impl Into<Cow<'static, [u8]>>) -> Image {
    Image::new(data.into())
}

/// [`View`] of an image.
pub struct Image {
    layout: LayoutStyle,
    data:   Cow<'static, [u8]>,
    tint:   Option<Color>,
}

impl Image {
    /// Create new [`Image`].
    pub fn new(data: Cow<'static, [u8]>) -> Self {
        Self {
            layout: Default::default(),
            data,
            tint: None,
        }
    }

    /// Set the tint.
    ///
    /// Setting this will use the image as a mask.
    pub fn tint(mut self, tint: impl Into<Option<Color>>) -> Self {
        self.tint = tint.into();
        self
    }
}

impl StyleLayout for Image {
    fn get_layout_style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.layout
    }
}

impl ViewMarker for Image {}
impl<P, T> View<Context<P>, T> for Image
where
    P: Platform,
{
    type Element = ImageWidget<P>;
    type State = ImageState;

    fn build(self, cx: &mut Context<P>, _data: &mut T) -> (Self::Element, Self::State) {
        let mut widget = ImageWidget::new(cx);
        widget.set_layout(cx, self.layout);
        widget.set_tint(cx, self.tint);

        let hash = seahash::hash(&self.data);
        if let Err(error) = widget.load_data(cx, self.data) {
            tracing::error!(?error, "loading image failed");
        }

        let state = ImageState {
            hash,
            tint: self.tint,
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

        let hash = seahash::hash(&self.data);

        if state.hash != hash {
            state.hash = hash;

            if let Err(error) = element.load_data(cx, self.data) {
                tracing::error!(?error, "loading image failed");
            }
        }

        if state.tint != self.tint {
            state.tint = self.tint;
            element.set_tint(cx, self.tint);
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

pub struct ImageState {
    hash:   u64,
    tint:   Option<Color>,
    layout: LayoutStyle,
}
