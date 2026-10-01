use ori_native::prelude::*;

fn main() {
    App::new().run(&mut (), ui).unwrap();
}

fn ui(_: &()) -> impl Effect<()> + use<> {
    window(
        row(row((
            row(text("hello")).background(Color::RED.fade(0.2)),
            row(text("world")).background(Color::RED.fade(0.2)),
            row(textinput()
                .text("how are you doing?")
                .placeholder("placeholder")
                .width(200.0))
            .background(Color::RED.fade(0.2)),
        ))
        .align_items(Align::Baseline)
        .gap(6.0))
        .flex(1.0)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(Color::WHITE),
    )
    .title("Baseline (examples/baseline.rs)")
}
