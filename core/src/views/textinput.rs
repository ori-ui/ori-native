use std::borrow::Cow;

use ori::{Action, Message, Mut, Proxied, Proxy, Tracker, View, ViewId, ViewMarker};

use crate::{
    Color, Context, Font, Layout, LayoutStyle, Newline, Platform, Stretch, TextAlign, TextWrap,
    Weight, widgets::TextInputWidget,
};

/// [`View`] of a text input.
pub fn textinput<T>() -> TextInput<T> {
    TextInput::new()
}

/// [`View`] of a text input.
#[allow(clippy::type_complexity)]
pub struct TextInput<T> {
    layout: LayoutStyle,
    font:   Font,
    text:   Option<String>,

    placeholder_font: Font,
    placeholder_text: String,

    align: TextAlign,
    wrap:  TextWrap,

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
    /// Create new [`TextInput`].
    pub fn new() -> Self {
        Self {
            layout: LayoutStyle::default(),
            font:   Default::default(),
            text:   None,

            placeholder_font: Font {
                color: Color::rgb(0.3, 0.3, 0.3),
                ..Default::default()
            },
            placeholder_text: String::new(),

            align: TextAlign::Start,
            wrap:  TextWrap::Word,

            newline:    Newline::Enter,
            accept_tab: true,
            on_change:  Box::new(|_, _| Action::new()),
            on_submit:  Box::new(|_, _| Action::new()),
        }
    }

    /// Set the text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Set the placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder_text = placeholder.into();
        self
    }

    /// Set the font size.
    pub fn size(mut self, size: f32) -> Self {
        self.font.size = size;
        self.placeholder_font.size = size;
        self
    }

    /// Set the font family.
    pub fn family(mut self, family: impl Into<Cow<'static, str>>) -> Self {
        self.font.family = Some(family.into());
        self.placeholder_font.family = self.font.family.clone();
        self
    }

    /// Set the font weight.
    pub fn weight(mut self, weight: Weight) -> Self {
        self.font.weight = weight;
        self.placeholder_font.weight = weight;
        self
    }

    /// Set the font stretch.
    pub fn stretch(mut self, stretch: Stretch) -> Self {
        self.font.stretch = stretch;
        self.placeholder_font.stretch = stretch;
        self
    }

    /// Set whether the font is italic.
    pub fn italic(mut self, italic: bool) -> Self {
        self.font.italic = italic;
        self.placeholder_font.italic = italic;
        self
    }

    /// Set whether the font is strikethrough.
    pub fn strikethrough(mut self, strikethrough: bool) -> Self {
        self.font.striketrough = strikethrough;
        self.placeholder_font.striketrough = strikethrough;
        self
    }

    /// Set the text color.
    pub fn color(mut self, color: Color) -> Self {
        self.font.color = color;
        self
    }

    /// Set the placeholder font size.
    pub fn placeholder_size(mut self, size: f32) -> Self {
        self.placeholder_font.size = size;
        self
    }

    /// Set the placeholder font family.
    pub fn placeholder_family(mut self, family: impl Into<Cow<'static, str>>) -> Self {
        self.placeholder_font.family = Some(family.into());
        self
    }

    /// Set the placeholder font weight.
    pub fn placeholder_weight(mut self, weight: Weight) -> Self {
        self.placeholder_font.weight = weight;
        self
    }

    /// Set the placeholder font stretch.
    pub fn placeholder_stretch(mut self, stretch: Stretch) -> Self {
        self.placeholder_font.stretch = stretch;
        self
    }

    /// Set whether the placeholder font is italic.
    pub fn placeholder_italic(mut self, italic: bool) -> Self {
        self.placeholder_font.italic = italic;
        self
    }

    /// Set whether the placeholder font is strikethrough.
    pub fn placeholder_strikethrough(mut self, strikethrough: bool) -> Self {
        self.placeholder_font.striketrough = strikethrough;
        self
    }

    /// Set the placeholder text color.
    pub fn placeholder_color(mut self, color: Color) -> Self {
        self.placeholder_font.color = color;
        self
    }

    /// Set the text alignment.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Set the text wrapping.
    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = wrap;
        self
    }

    /// Set the newline behaviour.
    pub fn newline(mut self, newline: Newline) -> Self {
        self.newline = newline;
        self
    }

    /// Set whether to accept `tab` inputs.
    pub fn accept_tab(mut self, accept_tab: bool) -> Self {
        self.accept_tab = accept_tab;
        self
    }

    /// Set the callback for when the text changes.
    pub fn on_change<A>(mut self, mut on_change: impl FnMut(&mut T, String) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_change = Box::new(move |data, text| on_change(data, text).into());
        self
    }

    /// Set the callback for when text is submitted.
    pub fn on_submit<A>(mut self, mut on_submit: impl FnMut(&mut T, String) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_submit = Box::new(move |data, text| on_submit(data, text).into());
        self
    }
}

impl<T> Layout for TextInput<T> {
    fn get_layout_style_mut(&mut self) -> &mut LayoutStyle {
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
    P: Platform + Proxied,
{
    type Element = TextInputWidget<P>;
    type State = TextInputState<T>;

    fn build(self, cx: &mut Context<P>, _data: &mut T) -> (Self::Element, Self::State) {
        let view_id = ViewId::next();
        cx.register(view_id);

        let on_change = {
            let proxy = cx.proxy();

            move |text| {
                proxy.message(Message::new(
                    TextInputMessage::Change(text),
                    view_id,
                ));
            }
        };

        let on_submit = {
            let proxy = cx.proxy();

            move |text| {
                proxy.message(Message::new(
                    TextInputMessage::Submit(text),
                    view_id,
                ));
            }
        };

        let mut widget = TextInputWidget::new(cx, on_change, on_submit);
        widget.set_layout(cx, self.layout);
        widget.set_font(
            cx,
            self.font.clone(),
            self.align,
            self.wrap,
        );

        if let Some(text) = self.text.clone() {
            widget.set_text(cx, text);
        }

        widget.set_placeholder_font(
            cx,
            self.placeholder_font.clone(),
            self.align,
            self.wrap,
        );

        widget.set_placeholder_text(cx, self.placeholder_text.clone());
        widget.update_layout(cx);

        widget.set_newline(cx, self.newline);
        widget.set_accept_tab(cx, self.accept_tab);

        let state = TextInputState {
            layout: self.layout,

            font: self.font,
            text: self.text.unwrap_or_default(),

            placeholder_font: self.placeholder_font,
            placeholder_text: self.placeholder_text,

            align: self.align,
            wrap: self.wrap,

            newline: self.newline,
            accept_tab: self.accept_tab,

            view_id,
            on_change: self.on_change,
            on_submit: self.on_submit,
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

        let mut changed = false;

        if state.font != self.font || state.align != self.align || state.wrap != self.wrap {
            state.font = self.font.clone();
            state.align = self.align;
            state.wrap = self.wrap;
            element.set_font(cx, self.font, self.align, self.wrap);
            changed |= true;
        }

        if let Some(text) = self.text
            && state.text != text
        {
            state.text = text.clone();
            element.set_text(cx, text);
            changed |= true;
        }

        if state.placeholder_font != self.placeholder_font
            || state.align != self.align
            || state.wrap != self.wrap
        {
            state.placeholder_font = self.placeholder_font.clone();
            state.align = self.align;
            state.wrap = self.wrap;
            element.set_font(
                cx,
                self.placeholder_font,
                self.align,
                self.wrap,
            );
            changed |= true;
        }

        if state.placeholder_text != self.placeholder_text {
            state.placeholder_text = self.placeholder_text.clone();
            element.set_placeholder_text(cx, self.placeholder_text);
            changed |= true;
        }

        if state.newline != self.newline {
            state.newline = self.newline;
            element.set_newline(cx, self.newline);
        }

        if state.accept_tab != self.accept_tab {
            state.accept_tab = self.accept_tab;
            element.set_accept_tab(cx, self.accept_tab);
        }

        if changed {
            element.update_layout(cx);
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
        if let Some(message) = message.take(state.view_id) {
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

    fn teardown(element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        element.teardown(cx);
        cx.unregister(state.view_id);
    }
}

#[allow(clippy::type_complexity)]
pub struct TextInputState<T> {
    layout: LayoutStyle,

    font: Font,
    text: String,

    placeholder_font: Font,
    placeholder_text: String,

    align: TextAlign,
    wrap:  TextWrap,

    newline:    Newline,
    accept_tab: bool,

    view_id:   ViewId,
    on_change: Box<dyn FnMut(&mut T, String) -> Action>,
    on_submit: Box<dyn FnMut(&mut T, String) -> Action>,
}
