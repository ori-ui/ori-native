use gtk4::prelude::DrawingAreaExtManual;
use ori_native_core::{
    Blend, Canvas, Cap, Curve, CurveSegment, Fill, Join, Paint, Primitive, Shader,
    native::NativePainter,
};

use crate::Platform;

pub struct Painter {
    area: gtk4::DrawingArea,
}

impl NativePainter<Platform> for Painter {
    fn build(_platform: &mut Platform) -> Self {
        let area = gtk4::DrawingArea::new();

        Self { area }
    }

    fn teardown(self, _platform: &mut Platform) {}

    fn widget_ref(&self) -> gtk4::Widget {
        self.area.clone().into()
    }

    fn set_canvas(&mut self, _platform: &mut Platform, canvas: Canvas) {
        self.area.set_draw_func(move |_, cx, _, _| {
            for primitive in canvas.primitives() {
                draw_primitive(cx, primitive);
            }
        });
    }
}

fn draw_primitive(cx: &gtk4::cairo::Context, primitive: &Primitive) {
    cx.set_antialias(gtk4::cairo::Antialias::Best);

    match primitive {
        Primitive::Stroke(curve, paint) => {
            draw_curve(cx, curve);
            draw_paint(cx, paint);

            let _ = cx.stroke();
        }

        Primitive::Fill(curve, paint) => {
            draw_curve(cx, curve);
            draw_paint(cx, paint);

            let _ = cx.fill();
        }
    }
}

fn draw_curve(cx: &gtk4::cairo::Context, curve: &Curve) {
    for segment in curve.segments() {
        match segment {
            CurveSegment::Move(p0) => {
                cx.move_to(p0.x as f64, p0.y as f64);
            }

            CurveSegment::Line(p0) => {
                cx.line_to(p0.x as f64, p0.y as f64);
            }

            CurveSegment::Quad(p0, p1) => {
                if let Ok((x0, y0)) = cx.current_point() {
                    cx.curve_to(
                        2.0 / 3.0 * p0.x as f64 + 1.0 / 3.0 * x0,
                        2.0 / 3.0 * p0.y as f64 + 1.0 / 3.0 * y0,
                        2.0 / 3.0 * p0.x as f64 + 1.0 / 3.0 * p1.x as f64,
                        2.0 / 3.0 * p0.y as f64 + 1.0 / 3.0 * p1.y as f64,
                        p1.x as f64,
                        p1.y as f64,
                    );
                }
            }

            CurveSegment::Cubic(p0, p1, p2) => {
                cx.curve_to(
                    p0.x as f64,
                    p0.y as f64,
                    p1.x as f64,
                    p1.y as f64,
                    p2.x as f64,
                    p2.y as f64,
                );
            }

            CurveSegment::Close => {
                cx.close_path();
            }
        }
    }
}

fn draw_paint(cx: &gtk4::cairo::Context, paint: &Paint) {
    match paint.shader {
        Shader::Color(color) => {
            cx.set_source_rgba(
                color.r as f64,
                color.g as f64,
                color.b as f64,
                color.a as f64,
            );
        }

        Shader::Gradient(ref gradient) => {
            let pattern = gtk4::cairo::LinearGradient::new(
                gradient.start.x as f64,
                gradient.start.y as f64,
                gradient.end.x as f64,
                gradient.end.y as f64,
            );

            for (i, color) in gradient.colors.iter().enumerate() {
                let offset = i as f64 / (gradient.colors.len() as f64 - 1.0);

                pattern.add_color_stop_rgba(
                    offset,
                    color.r as f64,
                    color.g as f64,
                    color.b as f64,
                    color.a as f64,
                );
            }

            let _ = cx.set_source(pattern);
        }
    }

    cx.set_line_join(match paint.stroke.join {
        Join::Miter => gtk4::cairo::LineJoin::Miter,
        Join::Round => gtk4::cairo::LineJoin::Round,
        Join::Bevel => gtk4::cairo::LineJoin::Bevel,
    });

    cx.set_line_cap(match paint.stroke.cap {
        Cap::Butt => gtk4::cairo::LineCap::Butt,
        Cap::Round => gtk4::cairo::LineCap::Round,
        Cap::Square => gtk4::cairo::LineCap::Square,
    });

    cx.set_fill_rule(match paint.fill {
        Fill::Winding => gtk4::cairo::FillRule::Winding,
        Fill::EvenOdd => gtk4::cairo::FillRule::EvenOdd,
    });

    cx.set_operator(match paint.blend {
        Blend::Clear => gtk4::cairo::Operator::Clear,
        Blend::Src => gtk4::cairo::Operator::Source,
        Blend::Dst => gtk4::cairo::Operator::Dest,
        Blend::SrcOver => gtk4::cairo::Operator::Over,
        Blend::DstOver => gtk4::cairo::Operator::DestOver,
        Blend::SrcIn => gtk4::cairo::Operator::In,
        Blend::DstIn => gtk4::cairo::Operator::DestIn,
        Blend::SrcATop => gtk4::cairo::Operator::Atop,
        Blend::DstATop => gtk4::cairo::Operator::DestAtop,
    });

    cx.set_line_width(paint.stroke.width as f64);
    cx.set_miter_limit(paint.stroke.miter as f64);
}
