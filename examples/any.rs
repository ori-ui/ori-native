use ori_native::prelude::*;

fn main() {
    let mut data = Data { toggle: false };

    App::new().run(&mut data, ui);
}

struct Data {
    toggle: bool,
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    let toggle = pressable(|_, state| {
        row(text("Toggle"))
            .background_color(Color::RED.fade(if !state.hovered { 0.6 } else { 0.7 }))
            .padding(8.0)
            .corners(8.0)
    })
    .on_press(|data: &mut Data| data.toggle = !data.toggle);

    window(
        row(column((toggle, self::toggle(data))))
            .flex(1.0)
            .justify_contents(Justify::Center)
            .align_items(Align::Center),
    )
}

fn toggle(data: &Data) -> impl View<Data> + use<> {
    match data.toggle {
        true => any(text("thing")),
        false => any(column((text("fdad"), text("asdf")))),
    }
}
