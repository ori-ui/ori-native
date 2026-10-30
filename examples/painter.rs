use ori_native::{Gradient, prelude::*};

fn main() {
    let mut data = Data {};

    App::new().run(&mut data, ui).unwrap();
}

struct Data {}

fn ui(_data: &Data) -> impl Effect<Data> + use<> {
    let mut canvas = Canvas::new();

    let mut curve = Curve::new();
    curve.circle(20.0, 20.0, 20.0);

    canvas.fill(curve, Paint::default());

    window(row(grid().flex(1.0)).background(Color::WHITE).flex(1.0))
}

#[builder]
fn grid<T>(
    #[default = 20.0] space: f32,
    #[default = 3.0] radius: f32,
    #[layout] layout: LayoutStyle,
) -> impl View<T> {
    freeze(move || {
        with(
            |_| (0.0f32, 0.0f32),
            move |(width, height), _| {
                let rows = f32::floor((width - space) / space) as u32;
                let cols = f32::floor((height - space) / space) as u32;

                let pad_x = (width - rows as f32 * space) / 2.0;
                let pad_y = (height - cols as f32 * space) / 2.0;

                let mut curve = Curve::new();

                for x in 0..=rows {
                    for y in 0..=cols {
                        let x = x as f32 * space + pad_x;
                        let y = y as f32 * space + pad_y;

                        curve.circle(x, y, radius);
                    }
                }

                let mut canvas = Canvas::new();
                canvas.fill(
                    curve,
                    Paint {
                        shader: Shader::Gradient(Gradient {
                            start:  Point::new(0.0, 0.0),
                            end:    Point::new(*width, *height),
                            colors: vec![Color::BLUE, Color::RED],
                        }),
                        ..Default::default()
                    },
                );

                on_layout(
                    paint(canvas).set_layout(layout),
                    |((width, height), _), w, h| {
                        *width = w;
                        *height = h;
                    },
                )
            },
        )
    })
}
