use std::borrow::Cow;

use ori::{Action, Message, Mut, Proxied, Proxy, Tracked, View, ViewId, ViewMarker};

use crate::{
    Color, Context, Font, LayoutStyle, Newline, Platform, RequestFocus, Stretch, StyleLayout,
    Submit, TextAlign, TextWrap, Weight, event::TextInputEvent, widgets::TextInputWidget,
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

    placeholder:       String,
    placeholder_color: Color,

    align: TextAlign,
    wrap:  TextWrap,

    newline:    Newline,
    submit:     Submit,
    accept_tab: bool,
    auto_focus: bool,
    view_id:    Option<ViewId>,
    on_event:   Vec<BoxedCallback<T>>,
}

type BoxedCallback<T> = Box<dyn FnMut(&mut T, TextInputEvent) -> Action>;

impl<T> Default for TextInput<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TextInput<T> {
    /// Create new [`TextInput`].
    pub fn new() -> Self {
        Self {
            view_id: None,

            layout: LayoutStyle::default(),
            font:   Default::default(),
            text:   None,

            placeholder:       String::new(),
            placeholder_color: Color::rgb(0.3, 0.3, 0.3),

            align: TextAlign::Start,
            wrap:  TextWrap::Word,

            newline:    Newline::Enter,
            submit:     Submit::Blur,
            accept_tab: true,
            auto_focus: false,
            on_event:   Vec::new(),
        }
    }

    /// Set the text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Set the placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the font size.
    pub fn size(mut self, size: f32) -> Self {
        self.font.size = size;
        self
    }

    /// Set the font family.
    pub fn family(mut self, family: impl Into<Cow<'static, str>>) -> Self {
        self.font.family = Some(family.into());
        self
    }

    /// Set the font weight.
    pub fn weight(mut self, weight: Weight) -> Self {
        self.font.weight = weight;
        self
    }

    /// Set the font stretch.
    pub fn stretch(mut self, stretch: Stretch) -> Self {
        self.font.stretch = stretch;
        self
    }

    /// Set whether the font is italic.
    pub fn italic(mut self, italic: bool) -> Self {
        self.font.italic = italic;
        self
    }

    /// Set whether the font is strikethrough.
    pub fn strikethrough(mut self, strikethrough: bool) -> Self {
        self.font.striketrough = strikethrough;
        self
    }

    /// Set the text color.
    pub fn color(mut self, color: Color) -> Self {
        self.font.color = color;
        self
    }

    /// Set the placeholder text color.
    pub fn placeholder_color(mut self, color: Color) -> Self {
        self.placeholder_color = color;
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

    /// Set the submit behaviour.
    pub fn submit(mut self, submit: Submit) -> Self {
        self.submit = submit;
        self
    }

    /// Set whether to accept `tab` inputs.
    pub fn accept_tab(mut self, accept_tab: bool) -> Self {
        self.accept_tab = accept_tab;
        self
    }

    /// Set whether the view should be automatically focused when built.
    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }

    /// Set the [`ViewId`], used to receive [`RequestFocus`](crate::RequestFocus) messages.
    pub fn view_id(mut self, view_id: impl Into<Option<ViewId>>) -> Self {
        self.view_id = view_id.into();
        self
    }

    /// Set the callback for all events.
    pub fn on_event(
        mut self,
        on_event: impl FnMut(&mut T, TextInputEvent) -> Action + 'static,
    ) -> Self {
        self.on_event.push(Box::new(on_event));
        self
    }

    /// Set the callback for when the text changes.
    pub fn on_change<A>(self, mut on_change: impl FnMut(&mut T, String) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_event(move |data, event| match event {
            TextInputEvent::Changed(text) => on_change(data, text).into(),
            _ => Action::new(),
        })
    }

    /// Set the callback for when text is submitted.
    pub fn on_submit<A>(self, mut on_submit: impl FnMut(&mut T, String) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        let mut text = self.text.clone().unwrap_or_default();

        self.on_event(move |data, event| match event {
            TextInputEvent::Changed(new_text) => {
                text = new_text;
                Action::new()
            }

            TextInputEvent::Submitted => on_submit(data, text.clone()).into(),

            _ => Action::new(),
        })
    }

    /// Set the callback for when the textinput is focused.
    pub fn on_focus<A>(self, mut on_focus: impl FnMut(&mut T) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_event(move |data, event| match event {
            TextInputEvent::Focused(true) => on_focus(data).into(),
            _ => Action::new(),
        })
    }

    /// Set the callback for when the textinput is unfocused.
    pub fn on_blur<A>(self, mut on_blur: impl FnMut(&mut T) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_event(move |data, event| match event {
            TextInputEvent::Focused(false) => on_blur(data).into(),
            _ => Action::new(),
        })
    }

    /// Set the callback for when text has been edited.
    pub fn on_edited<A>(self, mut on_edited: impl FnMut(&mut T, String) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        let mut text = self.text.clone().unwrap_or_default();

        self.on_event(move |data, event| match event {
            TextInputEvent::Changed(new_text) => {
                text = new_text;
                Action::new()
            }

            TextInputEvent::Focused(false) | TextInputEvent::Submitted => {
                on_edited(data, text.clone()).into()
            }

            _ => Action::new(),
        })
    }
}

impl<T> StyleLayout for TextInput<T> {
    fn get_layout_style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.layout
    }
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

        let on_event = {
            let proxy = cx.proxy();
            move |event| proxy.message(Message::new(event, view_id))
        };

        let mut widget = TextInputWidget::new(cx, on_event);
        widget.set_layout(cx, self.layout);
        widget.set_font(
            cx,
            self.font.clone(),
            self.align,
            self.wrap,
        );

        if let Some(ref text) = self.text {
            widget.set_text(cx, text);
        }

        widget.set_placeholder_text(cx, &self.placeholder);
        widget.set_placeholder_color(cx, self.placeholder_color);

        widget.update_layout(cx);

        widget.set_newline(cx, self.newline);
        widget.set_submit(cx, self.submit);
        widget.set_accept_tab(cx, self.accept_tab);

        if self.auto_focus {
            widget.request_focus(cx);
        }

        if let Some(view_id) = self.view_id {
            cx.register(view_id);
        }

        let state = TextInputState {
            layout: self.layout,

            font: self.font,
            text: self.text.unwrap_or_default(),

            placeholder: self.placeholder,
            placeholder_color: self.placeholder_color,

            align: self.align,
            wrap: self.wrap,

            newline: self.newline,
            submit: self.submit,
            accept_tab: self.accept_tab,

            user_view_id: self.view_id,

            view_id,
            on_event: self.on_event,
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

        let mut layout_changed = false;

        if state.font != self.font || state.align != self.align || state.wrap != self.wrap {
            state.font = self.font.clone();
            state.align = self.align;
            state.wrap = self.wrap;
            element.set_font(cx, self.font, self.align, self.wrap);
            layout_changed = true;
        }

        if let Some(text) = self.text
            && state.text != text
        {
            element.set_text(cx, &text);
            state.text = text;
            layout_changed = true;
        }

        if state.user_view_id != self.view_id {
            if let Some(view_id) = state.user_view_id {
                cx.unregister(view_id);
            }

            if let Some(view_id) = self.view_id {
                cx.register(view_id);
            }

            state.user_view_id = self.view_id;
        }

        if state.placeholder_color != self.placeholder_color {
            state.placeholder_color = self.placeholder_color;
            element.set_placeholder_color(cx, self.placeholder_color);
            layout_changed = true;
        }

        if state.placeholder != self.placeholder {
            element.set_placeholder_text(cx, &self.placeholder);
            state.placeholder = self.placeholder;
            layout_changed = true;
        }

        if state.newline != self.newline {
            state.newline = self.newline;
            element.set_newline(cx, self.newline);
        }

        if state.submit != self.submit {
            state.submit = self.submit;
            element.set_submit(cx, self.submit);
        }

        if state.accept_tab != self.accept_tab {
            state.accept_tab = self.accept_tab;
            element.set_accept_tab(cx, self.accept_tab);
        }

        if layout_changed {
            element.update_layout(cx);
        }

        state.on_event = self.on_event;
    }

    fn message(
        mut element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(event) = message.take::<TextInputEvent>(state.view_id) {
            if let TextInputEvent::Changed(ref text) = event {
                state.text = text.clone();
            }

            let mut action = Action::new();

            for on_event in state.on_event.iter_mut() {
                action |= on_event(data, event.clone());
            }

            action
        } else if let Some(view_id) = state.user_view_id
            && let Some(RequestFocus) = message.take(view_id)
        {
            element.request_focus(cx);
            Action::new()
        } else {
            Action::new()
        }
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        element.teardown(cx);
        cx.unregister(state.view_id);

        if let Some(view_id) = state.user_view_id {
            cx.unregister(view_id);
        }
    }
}

#[allow(clippy::type_complexity)]
pub struct TextInputState<T> {
    layout: LayoutStyle,

    font: Font,
    text: String,

    placeholder:       String,
    placeholder_color: Color,

    align: TextAlign,
    wrap:  TextWrap,

    newline:    Newline,
    submit:     Submit,
    accept_tab: bool,

    user_view_id: Option<ViewId>,

    view_id:  ViewId,
    on_event: Vec<BoxedCallback<T>>,
}
