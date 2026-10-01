use ori_native::prelude::*;

#[ori_native::main]
pub fn main() {
    App::init_log();

    let mut data = Data {};

    App::new().run(&mut data, ui).unwrap();
}

struct Data {}

const MODAL: ViewId = ViewId::new("modal");

fn ui(_data: &Data) -> impl Effect<Data> + use<> {
    window(
        column((
            safe_area(
                column((
                    text("Hello mobile!").size(20.0),
                    modal_button(),
                ))
                .justify_content(Justify::Center)
                .align_items(Align::Center)
                .background(Color::WHITE)
                .gap(20.0)
                .flex(1.0),
            )
            .flex(1.0),
            portal(MODAL),
        ))
        .flex(1.0),
    )
    .status_bar(StatusBar {
        color:   Some(Color::hex("#dd92a4")),
        visible: true,
        light:   true,
    })
    .navigation_bar(NavigationBar {
        color: Some(Color::hex("#251684")),
        light: false,
    })
}

fn modal_button() -> impl View<Data> + use<> {
    fn modal() -> impl View<(bool, Data)> + use<> {
        column(
            pressable(|_, _| {
                column(text("This is a modal!").size(20.0))
                    .background(Color::WHITE)
                    .top(-50.0)
                    .padding(20.0)
                    .corner(12.0)
                    .shadow(16.0, 16.0, 20.0, Color::BLACK.fade(0.4))
            })
            .on_blur(|(open, _): &mut (bool, _)| *open = false),
        )
        .justify_content(Justify::Center)
        .align_items(Align::Center)
        .position(Position::Absolute)
        .inset(0.0)
    }

    with_default(|_, _| {
        pressable(|(open, _): &(bool, _), state| {
            let color = if state.pressed {
                Color::BLACK.fade(0.2)
            } else {
                Color::BLACK.fade(0.1)
            };

            effect(
                transition(color, Ease(0.1), |_, color| {
                    column(text("Open modal").size(20.0))
                        .background(color)
                        .padding(12.0)
                        .border(1.0, Color::BLACK)
                        .corner(12.0)
                }),
                open.then(|| teleport(MODAL, modal())),
            )
        })
        .on_press(|(open, _): &mut (bool, _), _| *open = !*open)
    })
}
