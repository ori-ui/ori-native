use ori_native::{LayoutStyle, prelude::*};

fn main() {
    let mut data = Data {};

    App::new().run(&mut data, ui).unwrap();
}

struct Data {}

fn ui(_data: &Data) -> impl Effect<Data> + use<> {
    window(
        row((
            button(text("Click me!"), |_| {}).align_self(Align::Center),
            button(text("Also click me!"), |_| {}).align_self(Align::Center),
        ))
        .background(Color::WHITE)
        .justify_content(Justify::Center)
        .gap(20.0)
        .flex(1.0),
    )
}

// a configurable `button` view
fn button<T, V, A>(
    contents: V,
    on_press: impl FnMut(&mut T) -> A,
) -> Button<V, impl FnMut(&mut T) -> A> {
    Button::new(contents, on_press)
}

// builder struct with all the desired properties of a `button`
struct Button<V, F> {
    contents: V,
    on_press: F,
    color:    Color,
    layout:   LayoutStyle,
}

impl<V, F> Button<V, F> {
    fn new(contents: V, on_press: F) -> Self {
        Self {
            contents,
            on_press,
            color: Color::hex("#eeeeee"),
            layout: Default::default(),
        }
    }
}

// to be able to configure layout `Layout` is implemented for `Button`
impl<V, F> StyleLayout for Button<V, F> {
    fn get_layout_style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.layout
    }
}

// implementing `Builder` allows `Button` to be used as a `View`
impl<V, F> BuilderMarker for Button<V, F> {}
impl<T, V, F, A> Builder<Context, T> for Button<V, F>
where
    V: View<T>,
    F: FnMut(&mut T) -> A + 'static,
    A: Into<Action>,
{
    #[allow(refining_impl_trait)]
    fn build(mut self) -> impl View<T> {
        let mut contents = Some(self.contents);

        pressable(move |_, state| {
            // compute the color of the button
            let mut color = self.color;

            if state.pressed {
                color = color.darken(0.1);
            } else if state.hovered {
                color = color.darken(0.05);
            }

            // compute border color
            let border_color = match state.focused {
                true => Color::BLUE,
                false => Color::TRANSPARENT,
            };

            // compute amount of floating
            let float = match state.pressed {
                true => 0.0,
                false => 6.0,
            };

            let mut contents = contents.take();
            transition(
                (color, border_color, float),
                Ease(0.1),
                move |_, (color, border_color, float)| {
                    // put the `contents` in a `maybe` since it shouldn't rebuild based on press
                    // state or transition.
                    row(maybe(contents.take()))
                        .layout(self.layout)
                        .border(2.0, border_color)
                        .background(color)
                        .top(-float)
                        .padding(12.0)
                        .corner(10.0)
                        .shadow(
                            4.0 + float * 0.4,
                            6.0 + float * 1.5,
                            6.0 + float,
                            Color::BLACK.fade(0.4),
                        )
                },
            )
        })
        .on_press(move |data, _| (self.on_press)(data))
    }
}
