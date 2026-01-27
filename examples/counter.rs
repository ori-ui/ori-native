use ori_native::{App, Effect};
use ori_native_core::{
    Align, FlexContainer, FlexItem, Justify, LayoutContainer,
    views::{column, pressable, text, window},
};

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
        .align_items(Align::Center),
    )
}
