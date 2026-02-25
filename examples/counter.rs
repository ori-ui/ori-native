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
            pressable(|_, state| {
                if state.pressed {
                    text("Pressed!")
                } else if state.hovered {
                    text("Hovered!")
                } else {
                    text("Press me!")
                }
            })
            .on_press(|data: &mut Data| data.count += 1),
            text(format!("Pressed {} times.", data.count)),
        ))
        .flex(1.0)
        .gap(20.0)
        .justify_contents(Justify::Center)
        .align_items(Align::Center)
        .background_color(Color::WHITE),
    )
}
