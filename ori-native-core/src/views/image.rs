use std::borrow::Cow;

use ori::{Action, Message, Mut, View, ViewMarker};

use crate::{
    Color, Context, Layout, Pod,
    native::{HasImage, NativeImage},
};

pub fn image(data: impl Into<Cow<'static, [u8]>>) -> Image {
    Image::new(data.into())
}

pub struct Image {
    style: taffy::Style,
    data:  Cow<'static, [u8]>,
    tint:  Option<Color>,
}

impl Image {
    pub fn new(data: Cow<'static, [u8]>) -> Self {
        Self {
            style: Default::default(),
            data,
            tint: None,
        }
    }

    pub fn tint(mut self, tint: impl Into<Option<Color>>) -> Self {
        self.tint = tint.into();
        self
    }
}

impl Layout for Image {
    fn style_mut(&mut self) -> &mut taffy::Style {
        &mut self.style
    }
}

impl ViewMarker for Image {}
impl<P, T> View<Context<P>, T> for Image
where
    P: HasImage,
{
    type Element = Pod<P, P::Image>;
    type State = ();

    fn build(self, cx: &mut Context<P>, _data: &mut T) -> (Self::Element, Self::State) {
        let mut widget = P::Image::build(&mut cx.platform);
        widget.set_tint(self.tint);

        let layout = widget.load_data(&mut cx.platform, self.data).unwrap();

        let node = cx.new_layout_leaf(self.style, layout);
        let pod = Pod::new(node, widget);

        (pod, ())
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        _state: &mut Self::State,
        cx: &mut Context<P>,
        _data: &mut T,
    ) {
        let _ = cx.set_layout_style(*element.node, self.style);

        let layout = element
            .widget
            .load_data(&mut cx.platform, self.data)
            .unwrap();

        cx.set_leaf_layout(*element.node, layout).unwrap();
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
        element.widget.teardown(&mut cx.platform);
    }
}
