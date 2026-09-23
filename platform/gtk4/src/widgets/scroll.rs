use std::rc::Rc;

use glib::object::Cast;
use gtk4::prelude::{AdjustmentExt, FixedExt, WidgetExt};
use ori_native_core::{Direction, native::NativeScroll};

use crate::Platform;

pub struct Scroll {
    scroll: gtk4::ScrolledWindow,
    fixed:  gtk4::Fixed,
}

impl NativeScroll<Platform> for Scroll {
    fn build(
        _platform: &mut Platform,
        contents: gtk4::Widget,
        on_scroll: impl Fn(f32, f32) + 'static,
    ) -> Self {
        let fixed = gtk4::Fixed::new();
        fixed.put(&contents, 0.0, 0.0);
        fixed.set_overflow(gtk4::Overflow::Visible);

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_child(Some(&fixed));

        let on_scroll = Rc::new(on_scroll);

        scroll.vadjustment().connect_value_changed({
            let on_scroll = on_scroll.clone();
            move |adjustment| on_scroll(0.0, adjustment.value() as f32)
        });

        scroll.hadjustment().connect_value_changed({
            let on_scroll = on_scroll.clone();
            move |adjustment| on_scroll(adjustment.value() as f32, 0.0)
        });

        Self { scroll, fixed }
    }

    fn teardown(self, _platform: &mut Platform) {}

    fn widget_ref(&self) -> gtk4::Widget {
        self.scroll.clone().upcast()
    }

    fn replace_contents(&mut self, _platform: &mut Platform, child: gtk4::Widget) {
        if let Some(child) = self.fixed.first_child() {
            self.fixed.remove(&child);
        }

        self.fixed.put(&child, 0.0, 0.0);
    }

    fn set_content_size(&mut self, _platform: &mut Platform, width: f32, height: f32) {
        self.fixed.set_size_request(
            width.round() as i32,
            height.round() as i32,
        );
    }

    fn set_content_layout(
        &mut self,
        _platform: &mut Platform,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        if let Some(child) = self.fixed.first_child() {
            self.fixed.move_(&child, x as f64, y as f64);

            child.set_size_request(
                width.round() as i32,
                height.round() as i32,
            );
        }
    }

    fn set_direction(&mut self, _platform: &mut Platform, direction: Direction) {
        self.scroll.set_hscrollbar_policy(match direction {
            Direction::Horizontal => gtk4::PolicyType::Automatic,
            Direction::Vertical => gtk4::PolicyType::Never,
        });

        self.scroll.set_vscrollbar_policy(match direction {
            Direction::Horizontal => gtk4::PolicyType::Never,
            Direction::Vertical => gtk4::PolicyType::Automatic,
        });
    }
}
