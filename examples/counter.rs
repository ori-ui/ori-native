use ori_native::{App, Effect, View};
use ori_native_core::views::{text, window};

fn main() {
    let mut data = Data { count: 0 };

    App::new().run(&mut data, ui);
}

struct Data {
    count: u32,
}

fn ui(_data: &Data) -> impl Effect<Data> + use<> {
    window(view())
}

fn view() -> impl View<Data> + use<> {
    text("hello")
}
