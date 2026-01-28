use std::borrow::Cow;

use ori::{Action, Message, Mut, View, ViewMarker};

use crate::{
    Color, Context, Font, Pod, Stretch, TextSpan, Weight, native::HasText, shadows::TextShadow,
};

pub fn text(text: impl Into<String>) -> Text {
    Text::new(text)
}

pub struct Text {
    font: Font,
    text: String,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            font: Default::default(),
            text: text.into(),
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.font.size = size;
        self
    }

    pub fn family(mut self, family: impl Into<Cow<'static, str>>) -> Self {
        self.font.family = Some(family.into());
        self
    }

    pub fn weight(mut self, weight: Weight) -> Self {
        self.font.weight = weight;
        self
    }

    pub fn stretch(mut self, stretch: Stretch) -> Self {
        self.font.stretch = stretch;
        self
    }

    pub fn italic(mut self, italic: bool) -> Self {
        self.font.italic = italic;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.font.color = color;
        self
    }
}

impl ViewMarker for Text {}
impl<P, T> View<Context<P>, T> for Text
where
    P: HasText,
{
    type Element = Pod<TextShadow<P>>;
    type State = (Font, String);

    fn build(self, cx: &mut Context<P>, _data: &mut T) -> (Self::Element, Self::State) {
        let spans = [TextSpan {
            attributes: self.font.clone(),
            range:      0..self.text.len(),
        }];

        let (shadow, leaf) = TextShadow::new(cx, spans.into(), self.text.clone());
        let node = cx.new_layout_leaf(Default::default(), leaf);

        let pod = Pod { node, shadow };

        (pod, (self.font, self.text))
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        (font, text): &mut Self::State,
        cx: &mut Context<P>,
        _data: &mut T,
    ) {
        if self.font == *font && self.text == *text {
            return;
        }

        *font = self.font.clone();
        *text = self.text.clone();

        let spans = [TextSpan {
            attributes: self.font,
            range:      0..self.text.len(),
        }];

        let leaf = element.shadow.set_text(spans.into(), self.text);
        let _ = cx.set_layout_leaf(*element.node, leaf);
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
        element.shadow.teardown(cx);
        let _ = cx.remove_layout_node(element.node);
    }
}
