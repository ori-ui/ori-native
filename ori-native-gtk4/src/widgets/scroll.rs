use gtk4::prelude::WidgetExt;
use ori_native_core::{
    Direction, NativeParent, NativeWidget,
    native::{HasScroll, NativeScroll},
};

use crate::Platform;

impl HasScroll for Platform {
    type Scroll = Scroll;
}

pub struct Scroll {
    scroll: gtk4::ScrolledWindow,
}

impl NativeWidget<Platform> for Scroll {
    fn widget(&self) -> &gtk4::Widget {
        self.scroll.as_ref()
    }
}

impl NativeParent<Platform> for Scroll {
    fn replace_child(&mut self, _platform: &mut Platform, index: usize, child: &gtk4::Widget) {
        debug_assert_eq!(index, 0);

        self.scroll.set_child(Some(child));
    }
}

impl NativeScroll<Platform> for Scroll {
    fn build(_platform: &mut Platform, contents: &gtk4::Widget) -> Self {
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_child(Some(contents));

        Self { scroll }
    }

    fn teardown(self, _platform: &mut Platform) {}

    fn set_size(&mut self, width: f32, height: f32) {
        self.scroll.set_size_request(
            width.round() as i32,
            height.round() as i32,
        );
    }

    fn set_direction(&mut self, direction: Direction) {
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
