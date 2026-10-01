use ori_native::prelude::*;

fn main() {
    App::init_log();

    let mut data = Data {};

    App::new().run(&mut data, ui).unwrap();
}

struct Data {}

fn ui(_data: &Data) -> impl Effect<Data> + use<> {
    struct Menu {
        x: f32,
        y: f32,
    }

    window(with(
        |_| None::<Menu>,
        |_, _| {
            pressable(|(menu, _): &(Option<Menu>, _), _| {
                popup(
                    column(()).background(Color::WHITE).flex(1.0),
                    menu.as_ref().map(|_| {
                        column(menu_button(text("Quit"), |_| -> () {
                            std::process::exit(1);
                        }))
                        .background(Color::WHITE)
                        .border(1.0, Color::BLACK.fade(0.2))
                        .corner(8.0)
                        .shadow(6.0, 6.0, 6.0, Color::BLACK.fade(0.4))
                        .margin(12.0)
                        .size(200.0, 300.0)
                        .overflow(Overflow::Hidden)
                    }),
                )
                .position(
                    menu.as_ref().map_or(0.0, |menu| menu.x - 12.0),
                    menu.as_ref().map_or(0.0, |menu| menu.y - 12.0),
                )
                .on_dismiss(|(menu, _)| *menu = None)
            })
            .on_press(
                |(menu, _): &mut (Option<Menu>, _), event| {
                    if event.button == Button::Secondary {
                        *menu = Some(Menu {
                            x: event.position.x,
                            y: event.position.y,
                        });
                    }
                },
            )
        },
    ))
}

fn menu_button<T, A>(
    contents: impl View<T>,
    mut on_select: impl FnMut(&mut T) -> A + 'static,
) -> impl View<T>
where
    A: Into<Action>,
{
    let mut contents = Some(contents);

    pressable(move |_, state| {
        let mut color = Color::TRANSPARENT;

        if state.pressed {
            color = Color::BLACK.fade(0.2);
        } else if state.hovered {
            color = Color::BLACK.fade(0.1);
        }

        row(maybe(contents.take()))
            .background(color)
            .justify_content(Justify::Center)
            .align_items(Align::Center)
    })
    .on_press(move |data, event| {
        if event.button == Button::Primary {
            on_select(data).into()
        } else {
            Action::new()
        }
    })
}
