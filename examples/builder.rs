use ori_native::prelude::*;

fn main() {
    App::init_log();

    let mut data = Data {};

    App::new().run(&mut data, ui).unwrap();
}

struct Data {}

fn ui(_data: &Data) -> impl Effect<Data> + use<> {
    window(
        row(my_button(text("hello"))
            .on_click(|_| info!("clicked"))
            .corner(8.0))
        .background(Color::WHITE)
        .justify_content(Justify::Center)
        .align_items(Align::Center)
        .flex(1.0),
    )
}

/// A button.
#[builder]
pub fn my_button<T>(
    /// Contents of the button.
    contents: impl View<T>,

    /// Callback for when the button is clicked.
    #[default = |_| Action::new()]
    mut on_click: impl (FnMut(&mut T) -> impl Into<Action>) + 'static,

    /// The color of the button.
    #[default = Color::RED]
    color: Color,

    #[default = Sides::from(12.0)]
    #[padding]
    padding: Sides<Length>,

    #[default = BorderStyle {
        color: Color::TRANSPARENT,
        width: Sides::from(2.0),
    }]
    #[border]
    border: BorderStyle,

    #[layout] layout: LayoutStyle,
    #[corners] corners: Corners<f32>,
    #[shadow] shadow: Shadow,
    #[flex] flex: FlexStyle,
) -> impl View<T> {
    let mut contents = Some(contents);

    pressable(move |_, state| {
        let color = if state.pressed {
            color.darken(0.1)
        } else if state.hovered {
            color.darken(0.05)
        } else {
            color
        };

        let border_color = if state.focused {
            Color::BLUE
        } else {
            border.color
        };

        row(maybe(contents.take()))
            .background(color)
            .border(border.width, border_color)
            .set_layout(layout)
            .set_padding(padding)
            .set_corners(corners)
            .set_shadow(shadow)
            .set_flex(flex)
    })
    .focusable(true)
    .on_press(move |data, _| on_click(data))
}
