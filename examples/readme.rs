use ori_native::prelude::*;

struct Data {
    count: u32,
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    let button = pressable(|_, state| {
        if state.pressed {
            text("Pressed!")
        } else if state.hovered {
            text("Hovered!")
        } else {
            text("Press me!")
        }
    })
    .on_press(|data: &mut Data, _| data.count += 1);

    let label = text(format!("Pressed {} times.", data.count));

    window(
        column((button, label))
            .justify_content(Justify::Center)
            .align_items(Align::Center)
            .background(Color::WHITE)
            .flex(1.0)
            .gap(20.0),
    )
    .title("Readme (examples/readme.rs)")
}

fn main() {
    let mut data = Data { count: 0 };

    App::new().run(&mut data, ui).unwrap();
}
