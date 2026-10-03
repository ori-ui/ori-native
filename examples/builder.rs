use ori_native::prelude::*;

fn main() {
    App::init_log();

    let mut data = Data {};

    App::new().run(&mut data, ui).unwrap();
}

struct Data {}

fn ui(_data: &Data) -> impl Effect<Data> + use<> {
    window(
        row(button(text("click me"), |_| {
            tracing::info!("clicked");
        })
        .color(Color::GREEN)
        .padding(10.0)
        .corner(6.0))
        .background(Color::WHITE)
        .justify_content(Justify::Center)
        .align_items(Align::Center)
        .flex(1.0),
    )
}

/// A button.
#[builder]
pub fn button<T, A>(
    contents: impl View<T>,
    mut on_click: impl FnMut(&mut T) -> A + 'static,
    #[default = Color::RED] color: Color,
    #[layout] layout: LayoutStyle,
    #[padding] padding: Sides<Length>,
    #[corners] corners: Corners<f32>,
    #[shadow] shadow: Shadow,
    #[border] border: BorderStyle,
    #[flex] flex: FlexStyle,
) -> impl View<T> + use<>
where
    A: Into<Action>,
{
    let mut contents = Some(contents);

    pressable(move |_, state| {
        let color = if state.pressed {
            color.darken(0.1)
        } else if state.hovered {
            color.darken(0.05)
        } else {
            color
        };

        row(maybe(contents.take()))
            .background(color)
            .set_layout(layout)
            .set_padding(padding)
            .set_corners(corners)
            .set_shadow(shadow)
            .set_border(border)
            .set_flex(flex)
    })
    .on_press(move |data, _| on_click(data))
}
