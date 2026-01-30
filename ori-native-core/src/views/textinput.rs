use std::borrow::Cow;

use ori::{Action, Message, Mut, Proxied, Proxy, View, ViewId, ViewMarker};

use crate::{
    Color, Context, Font, Layout, Pod, Stretch, Weight, native::HasTextInput,
    shadows::TextInputShadow,
};

pub fn textinput<T>() -> TextInput<T> {
    TextInput::new()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Newline {
    None,
    Enter,
    ShiftEnter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Submit {
    None,
}

#[allow(clippy::type_complexity)]
pub struct TextInput<T> {
    layout: taffy::Style,
    font:   Font,
    text:   Option<String>,

    placeholder_font: Font,
    placeholder_text: String,

    newline:    Newline,
    accept_tab: bool,
    on_change:  Box<dyn FnMut(&mut T, String) -> Action>,
    on_submit:  Box<dyn FnMut(&mut T, String) -> Action>,
}

impl<T> Default for TextInput<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TextInput<T> {
    pub fn new() -> Self {
        Self {
            layout: taffy::Style {
                overflow: taffy::Point {
                    x: taffy::Overflow::Hidden,
                    y: taffy::Overflow::Hidden,
                },
                ..Default::default()
            },
            font:   Default::default(),
            text:   None,

            placeholder_font: Font {
                color: Color::rgb(0.3, 0.3, 0.3),
                ..Default::default()
            },
            placeholder_text: String::new(),

            newline:    Newline::Enter,
            accept_tab: true,
            on_change:  Box::new(|_, _| Action::new()),
            on_submit:  Box::new(|_, _| Action::new()),
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder_text = placeholder.into();
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.font.size = size;
        self.placeholder_font.size = size;
        self
    }

    pub fn family(mut self, family: impl Into<Cow<'static, str>>) -> Self {
        self.font.family = Some(family.into());
        self.placeholder_font.family = self.font.family.clone();
        self
    }

    pub fn weight(mut self, weight: Weight) -> Self {
        self.font.weight = weight;
        self.placeholder_font.weight = weight;
        self
    }

    pub fn stretch(mut self, stretch: Stretch) -> Self {
        self.font.stretch = stretch;
        self.placeholder_font.stretch = stretch;
        self
    }

    pub fn italic(mut self, italic: bool) -> Self {
        self.font.italic = italic;
        self.placeholder_font.italic = italic;
        self
    }

    pub fn strikethrough(mut self, strikethrough: bool) -> Self {
        self.font.striketrough = strikethrough;
        self.placeholder_font.striketrough = strikethrough;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.font.color = color;
        self
    }

    pub fn placeholder_size(mut self, size: f32) -> Self {
        self.placeholder_font.size = size;
        self
    }

    pub fn placeholder_family(mut self, family: impl Into<Cow<'static, str>>) -> Self {
        self.placeholder_font.family = Some(family.into());
        self
    }

    pub fn placeholder_weight(mut self, weight: Weight) -> Self {
        self.placeholder_font.weight = weight;
        self
    }

    pub fn placeholder_stretch(mut self, stretch: Stretch) -> Self {
        self.placeholder_font.stretch = stretch;
        self
    }

    pub fn placeholder_italic(mut self, italic: bool) -> Self {
        self.placeholder_font.italic = italic;
        self
    }

    pub fn placeholder_strikethrough(mut self, strikethrough: bool) -> Self {
        self.placeholder_font.striketrough = strikethrough;
        self
    }

    pub fn placeholder_color(mut self, color: Color) -> Self {
        self.placeholder_font.color = color;
        self
    }

    pub fn newline(mut self, newline: Newline) -> Self {
        self.newline = newline;
        self
    }

    pub fn accept_tab(mut self, accept_tab: bool) -> Self {
        self.accept_tab = accept_tab;
        self
    }

    pub fn on_change<A>(mut self, mut on_change: impl FnMut(&mut T, String) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_change = Box::new(move |data, text| on_change(data, text).into());
        self
    }

    pub fn on_submit<A>(mut self, mut on_submit: impl FnMut(&mut T, String) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_submit = Box::new(move |data, text| on_submit(data, text).into());
        self
    }
}

impl<T> Layout for TextInput<T> {
    fn style_mut(&mut self) -> &mut taffy::Style {
        &mut self.layout
    }
}

enum TextInputMessage {
    Change(String),
    Submit(String),
}

impl<T> ViewMarker for TextInput<T> {}
impl<P, T> View<Context<P>, T> for TextInput<T>
where
    P: HasTextInput + Proxied,
    T: 'static,
{
    type Element = Pod<TextInputShadow<P>>;
    type State = TextInputState<T>;

    fn build(self, cx: &mut Context<P>, _data: &mut T) -> (Self::Element, Self::State) {
        let mut shadow = TextInputShadow::new(cx);

        shadow.set_font(cx, self.font.clone());

        if let Some(text) = self.text.clone() {
            shadow.set_text(cx, text);
        }

        shadow.set_placeholder_font(cx, self.placeholder_font.clone());
        shadow.set_placeholder_text(cx, self.placeholder_text.clone());

        shadow.set_newline(cx, self.newline);
        shadow.set_accept_tab(cx, self.accept_tab);

        let layout = shadow.layout(cx);
        let node = cx.new_layout_leaf(self.layout, layout);

        let view_id = ViewId::next();

        let proxy = cx.proxy();
        shadow.set_on_change(cx, move |text| {
            proxy.message(Message::new(
                TextInputMessage::Change(text),
                view_id,
            ));
        });

        let proxy = cx.proxy();
        shadow.set_on_submit(cx, move |text| {
            proxy.message(Message::new(
                TextInputMessage::Submit(text),
                view_id,
            ));
        });

        let pod = Pod { node, shadow };
        let state = TextInputState {
            font: self.font,
            text: self.text.unwrap_or_default(),

            placeholder_font: self.placeholder_font,
            placeholder_text: self.placeholder_text,

            newline: self.newline,
            accept_tab: self.accept_tab,

            view_id,
            on_change: self.on_change,
            on_submit: self.on_submit,
        };

        (pod, state)
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        _data: &mut T,
    ) {
        let _ = cx.set_layout_style(*element.node, self.layout);

        let mut changed = false;

        if self.font != state.font {
            state.font = self.font.clone();
            element.shadow.set_font(cx, self.font);
            changed |= true;
        }

        if let Some(text) = self.text
            && text != state.text
        {
            state.text = text.clone();
            element.shadow.set_text(cx, text);
            changed |= true;
        }

        if self.placeholder_font != state.placeholder_font {
            state.placeholder_font = self.placeholder_font.clone();
            element.shadow.set_font(cx, self.placeholder_font);
            changed |= true;
        }

        if self.placeholder_text != state.placeholder_text {
            state.placeholder_text = self.placeholder_text.clone();
            (element.shadow).set_placeholder_text(cx, self.placeholder_text);
            changed |= true;
        }

        if self.newline != state.newline {
            state.newline = self.newline;
            element.shadow.set_newline(cx, self.newline);
        }

        if self.accept_tab != state.accept_tab {
            state.accept_tab = self.accept_tab;
            element.shadow.set_accept_tab(cx, self.accept_tab);
        }

        if changed {
            let layout = element.shadow.layout(cx);
            let _ = cx.set_leaf_layout(*element.node, layout);
        }

        state.on_change = self.on_change;
        state.on_submit = self.on_submit;
    }

    fn message(
        _element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        _cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(message) = message.take_targeted(state.view_id) {
            match message {
                TextInputMessage::Change(text) => {
                    state.text = text.clone();
                    (state.on_change)(data, text)
                }

                TextInputMessage::Submit(text) => (state.on_submit)(data, text),
            }
        } else {
            Action::new()
        }
    }

    fn teardown(element: Self::Element, _state: Self::State, cx: &mut Context<P>) {
        element.shadow.teardown(cx);
        let _ = cx.remove_layout_node(element.node);
    }
}

#[doc(hidden)]
#[allow(clippy::type_complexity)]
pub struct TextInputState<T> {
    font: Font,
    text: String,

    placeholder_font: Font,
    placeholder_text: String,

    newline:    Newline,
    accept_tab: bool,

    view_id:   ViewId,
    on_change: Box<dyn FnMut(&mut T, String) -> Action>,
    on_submit: Box<dyn FnMut(&mut T, String) -> Action>,
}
