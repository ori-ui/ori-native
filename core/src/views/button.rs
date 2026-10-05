use std::rc::Rc;

use keyboard_types::{Modifiers, NamedKey};
use ori::{
    Action, Builder, BuilderMarker, ViewId,
    views::{maybe, with, without},
};

use crate::{
    BorderStyle, BoxedWidget, Color, Context, Corners, FlexStyle, LayoutStyle, Length, Platform,
    Sides, StyleBorder, StyleCorners, StyleFlexContainer, StyleLayout, StylePadding, WidgetView,
    views::{Ease, Transition, pressable, row, transition},
};

/// A simple button [`View`](ori::View).
pub fn button<T, V, A>(
    contents: V,
    on_click: impl FnMut(&mut T) -> A,
) -> Button<V, impl FnMut(&mut T) -> A> {
    Button::new(contents, on_click)
}

/// A simple button [`View`](ori::View).
pub struct Button<V, F> {
    contents:   V,
    on_click:   F,
    color:      Color,
    layout:     LayoutStyle,
    flex:       FlexStyle,
    padding:    Sides<Length>,
    corners:    Corners<f32>,
    transition: Rc<dyn Transition>,

    view_id: Option<ViewId>,

    color_hovered: Option<Color>,
    color_pressed: Option<Color>,

    focusable:  bool,
    auto_focus: bool,

    border:         BorderStyle,
    border_focused: Color,
}

impl<V, F> Button<V, F> {
    /// Create new [`Button`].
    pub fn new(contents: V, on_click: F) -> Self {
        Self {
            contents,
            on_click,
            color: Color::hex("#2196F3"),
            layout: LayoutStyle::default(),
            flex: FlexStyle::default(),
            padding: Sides::all(Length::Length(8.0)),
            corners: Corners::all(0.0),
            transition: Rc::new(Ease(0.05)),

            view_id: None,

            color_hovered: None,
            color_pressed: None,

            focusable: true,
            auto_focus: false,

            border: BorderStyle {
                color: Color::TRANSPARENT,
                width: Sides::all(Length::Length(2.0)),
            },
            border_focused: Color::hex("#1a45bc"),
        }
    }

    /// Set the color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the color when hovered.
    pub fn color_hovered(mut self, color: Color) -> Self {
        self.color_hovered = Some(color);
        self
    }

    /// Set the color when pressed.
    pub fn color_pressed(mut self, color: Color) -> Self {
        self.color_pressed = Some(color);
        self
    }

    /// Set whether the button should be focusable.
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    /// Set whether the button should automatically be focused when built.
    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }

    /// Set the [`ViewId`] of the button.
    pub fn view_id(mut self, view_id: impl Into<Option<ViewId>>) -> Self {
        self.view_id = view_id.into();
        self
    }

    /// Set the transition between colors.
    pub fn transition(mut self, transition: impl Transition + 'static) -> Self {
        self.transition = Rc::new(transition);
        self
    }
}

impl<V, F> StyleLayout for Button<V, F> {
    fn get_layout_style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.layout
    }
}

impl<V, F> StyleFlexContainer for Button<V, F> {
    fn get_flex_style_mut(&mut self) -> &mut FlexStyle {
        &mut self.flex
    }
}

impl<V, F> StylePadding for Button<V, F> {
    fn get_padding_mut(&mut self) -> &mut Sides<Length> {
        &mut self.padding
    }
}

impl<V, F> StyleCorners for Button<V, F> {
    fn get_corners_mut(&mut self) -> &mut Corners<f32> {
        &mut self.corners
    }
}

impl<V, F> StyleBorder for Button<V, F> {
    fn get_border_style_mut(&mut self) -> &mut BorderStyle {
        &mut self.border
    }
}

impl<V, F> BuilderMarker for Button<V, F> {}
impl<P, T, V, F, A> Builder<Context<P>, T> for Button<V, F>
where
    P: Platform,
    V: WidgetView<P, T>,
    F: FnMut(&mut T) -> A,
    A: Into<Action>,
{
    type Element = BoxedWidget<P>;

    fn build(self) -> impl WidgetView<P, T> {
        struct State<F> {
            key_pressed: bool,
            on_click:    Option<F>,
        }

        let mut contents = Some(self.contents);

        with(
            |_| State {
                key_pressed: false,
                on_click:    None,
            },
            move |_, _| {
                let mut contents = contents.take();
                let trans = self.transition.clone();

                pressable(
                    move |(state, _): &(State<F>, _), press_state| {
                        let color = if press_state.pressed || state.key_pressed {
                            self.color_pressed.unwrap_or_else(|| self.color.darken(0.1))
                        } else if press_state.hovered {
                            self.color_hovered
                                .unwrap_or_else(|| self.color.darken(0.05))
                        } else {
                            self.color
                        };

                        let border_color = if press_state.focused {
                            self.border_focused
                        } else {
                            self.border.color
                        };

                        let mut contents = contents.take();

                        transition(
                            (color, border_color),
                            trans.clone(),
                            move |_, (color, border_color)| {
                                let mut row = row(without(maybe(contents.take())))
                                    .set_layout(self.layout)
                                    .padding(self.padding)
                                    .corner(self.corners)
                                    .background(color)
                                    .border(self.border.width, border_color);

                                *row.get_flex_style_mut() = self.flex;
                                row
                            },
                        )
                    },
                )
                .focusable(self.focusable)
                .auto_focus(self.auto_focus)
                .view_id(self.view_id)
                .on_press(
                    move |(state, data), _| match state.on_click {
                        Some(ref mut on_click) => on_click(data).into(),
                        None => Action::new(),
                    },
                )
                .on_key(
                    NamedKey::Enter,
                    Modifiers::empty(),
                    move |(state, _)| state.key_pressed = true,
                )
                .on_key_up(
                    NamedKey::Enter,
                    Modifiers::empty(),
                    move |(state, data)| {
                        state.key_pressed = false;

                        match state.on_click {
                            Some(ref mut on_click) => on_click(data).into(),
                            None => Action::new(),
                        }
                    },
                )
            },
        )
        .update(move |state, _| state.on_click = Some(self.on_click))
    }
}
