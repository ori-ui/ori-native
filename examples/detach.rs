use ori_native::prelude::*;

fn main() {
    let mut data = Data {
        is_detached: false,
        count:       0,
    };

    App::new().run(&mut data, ui).unwrap();
}

struct Data {
    is_detached: bool,
    count:       usize,
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    let numbers = (0..data.count)
        .map(|n| text(format!("click: {n}")))
        .collect::<Vec<_>>();

    window(
        column(
            column((
                row((
                    button(
                        text(format!("detach: {}", data.is_detached)),
                        |data: &mut Data| {
                            data.is_detached = !data.is_detached;
                        },
                    ),
                    button(text("click me"), |data: &mut Data| {
                        data.count += 1;
                    }),
                ))
                .gap(10.0),
                detach(
                    data.is_detached,
                    (
                        button(
                            text("try click me"),
                            |data: &mut Data| {
                                data.count += 1;
                            },
                        ),
                        numbers,
                    ),
                ),
            ))
            .align_items(Align::Center)
            .gap(20.0),
        )
        .background(Color::WHITE)
        .justify_content(Justify::Center)
        .align_items(Align::Center)
        .flex(1.0),
    )
    .title("Detach (examples/detach.rs)")
}
