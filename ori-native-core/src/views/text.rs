use std::borrow::Cow;

use ori::{Action, Message, Mut, View, ViewMarker};

use crate::{
    Color, Context, Font, Layout, Pod, Stretch, TextSpan, Weight, Wrap,
    native::{HasText, NativeText},
};

pub fn text(text: impl Into<String>) -> Text {
    Text::new(text)
}

pub struct Text {
    layout: taffy::Style,
    font:   Font,
    text:   String,
    wrap:   Wrap,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            layout: taffy::Style {
                overflow: taffy::Point {
                    x: taffy::Overflow::Hidden,
                    y: taffy::Overflow::Hidden,
                },
                ..Default::default()
            },
            font:   Default::default(),
            text:   text.into(),
            wrap:   Wrap::None,
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

    pub fn strikethrough(mut self, strikethrough: bool) -> Self {
        self.font.striketrough = strikethrough;
        self
    }

    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.font.color = color;
        self
    }
}

impl Layout for Text {
    fn style_mut(&mut self) -> &mut taffy::Style {
        &mut self.layout
    }
}

impl ViewMarker for Text {}
impl<P, T> View<Context<P>, T> for Text
where
    P: HasText,
{
    type Element = Pod<P, P::Text>;
    type State = (Font, String);

    fn build(self, cx: &mut Context<P>, _data: &mut T) -> (Self::Element, Self::State) {
        let spans = [TextSpan {
            font:  self.font.clone(),
            range: 0..self.text.len(),
        }];

        let (widget, leaf) = P::Text::build(
            &mut cx.platform,
            spans.into(),
            self.text.clone(),
            self.wrap,
        );

        let node = cx.new_layout_leaf(self.layout, leaf);

        let pod = Pod::new(node, widget);

        (pod, (self.font, self.text))
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        (font, text): &mut Self::State,
        cx: &mut Context<P>,
        _data: &mut T,
    ) {
        let _ = cx.set_layout_style(*element.node, self.layout);

        if self.font == *font && self.text == *text {
            return;
        }

        *font = self.font.clone();
        *text = self.text.clone();

        let spans = [TextSpan {
            font:  self.font,
            range: 0..self.text.len(),
        }];

        let leaf = element.widget.set_text(spans.into(), self.text, self.wrap);
        let _ = cx.set_leaf_layout(*element.node, leaf);
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
        let _ = cx.remove_layout_node(element.node);
    }
}
