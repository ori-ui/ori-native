use ori_native::prelude::*;

fn main() {
    let mut data = Data { count: 0 };

    App::new().run(&mut data, ui);
}

struct Data {
    count: u32,
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    window(
        column((
            button(),
            text(format!("Pressed {} times.", data.count)),
        ))
        .flex(1.0)
        .gap(20.0)
        .justify_contents(Justify::Center)
        .align_items(Align::Center),
    )
}

fn button() -> impl View<Data> + use<> {
    pressable(|_, state| {
        row(if state.pressed {
            text("Pressed!")
        } else {
            text("Press me!")
        })
        .background_color(if state.hovered {
            Color::CYAN.darken(0.1)
        } else {
            Color::CYAN
        })
        .padding_all(8.0)
        .corner_all(8.0)
    })
    .on_press(|data: &mut Data| data.count += 1)
}
