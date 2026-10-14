use std::time::Duration;

use ori_native::prelude::*;

fn main() {
    let mut data = Data { count: 0 };

    App::new().run(&mut data, ui).unwrap();
}

struct Data {
    count: usize,
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    let count = data.count;

    window(
        column((
            button(
                text("click me to reset"),
                |data: &mut Data| data.count += 1,
            ),
            memo(count, move |_| {
                suspense(text("wait 5 seconds"), async move {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    text(format!("time is up ({count})"))
                })
            }),
        ))
        .flex(1.0)
        .justify_content(Justify::Center)
        .align_items(Align::Center)
        .background(Color::WHITE),
    )
    .title("Suspense (examples/suspense.rs)")
}
