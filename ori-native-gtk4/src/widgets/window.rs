use gtk4::{
    glib::subclass::types::ObjectSubclassIsExt,
    prelude::{GtkWindowExt, WidgetExt},
};
use ori_native_core::native::{HasWindow, NativeWindow};

use crate::Platform;

pub struct Window {
    window: ApplicationWindow,
}

impl NativeWindow<Platform> for Window {
    fn build(platform: &mut Platform, contents: &gtk4::Widget) -> Self {
        let window = ApplicationWindow::new(&platform.application);
        window.set_child(Some(contents));
        window.show();

        Self { window }
    }

    fn teardown(self, _platform: &mut Platform) {}

    fn get_size(&self) -> (u32, u32) {
        (
            self.window.width() as u32,
            self.window.height() as u32,
        )
    }

    fn set_on_close_requested(&mut self, on_close_requested: impl Fn() + 'static) {
        self.window.connect_close_request(move |_| {
            on_close_requested();
            gtk4::glib::Propagation::Stop
        });
    }

    fn set_on_resize(&mut self, on_resize: impl Fn() + 'static) {
        self.window.set_on_size_allocate(on_resize);
    }

    fn set_min_size(&mut self, width: u32, height: u32) {
        self.window.set_size_request(width as i32, height as i32);
    }
}

impl HasWindow for Platform {
    type Window = Window;
}

gtk4::glib::wrapper! {
    struct ApplicationWindow(
        ObjectSubclass<imp::ApplicationWindow>)
        @extends
            gtk4::ApplicationWindow,
            gtk4::Window,
            gtk4::Widget,
        @implements
            gtk4::Buildable,
            gtk4::Accessible,
            gtk4::ConstraintTarget,
            gtk4::Root,
            gtk4::Native,
            gtk4::ShortcutManager,
            gtk4::gio::ActionGroup,
            gtk4::gio::ActionMap;
}

impl ApplicationWindow {
    fn new(application: &gtk4::Application) -> Self {
        let window: ApplicationWindow = gtk4::glib::Object::builder().build();
        window.set_application(Some(application));
        window
    }

    fn set_on_size_allocate(&self, on_size_allocate: impl Fn() + 'static) {
        let _ = self
            .imp()
            .on_size_allocate
            .replace(Box::new(on_size_allocate));
    }
}

mod imp {
    use std::cell::RefCell;

    use gtk4::{
        glib::{
            self,
            subclass::{object::ObjectImpl, types::ObjectSubclass},
        },
        subclass::{
            prelude::ApplicationWindowImpl,
            widget::{WidgetImpl, WidgetImplExt},
            window::WindowImpl,
        },
    };

    pub(super) struct ApplicationWindow {
        pub(super) on_size_allocate: RefCell<Box<dyn Fn()>>,
    }

    impl Default for ApplicationWindow {
        fn default() -> Self {
            Self {
                on_size_allocate: RefCell::new(Box::new(|| {})),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ApplicationWindow {
        const NAME: &'static str = "OriNativeWindow";
        type Type = super::ApplicationWindow;
        type ParentType = gtk4::ApplicationWindow;
    }

    impl ObjectImpl for ApplicationWindow {}

    impl WidgetImpl for ApplicationWindow {
        fn size_allocate(&self, _width: i32, _height: i32, baseline: i32) {
            self.parent_size_allocate(i32::MAX >> 1, i32::MAX >> 1, baseline);
            let on_size_allocate = self.on_size_allocate.borrow();
            on_size_allocate();
        }

        fn measure(&self, orientation: gtk4::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.parent_measure(orientation, for_size);
            (-1, -1, -1, -1)
        }
    }

    impl WindowImpl for ApplicationWindow {}

    impl ApplicationWindowImpl for ApplicationWindow {}
}
