use std::time::Duration;

use glib::subclass::types::ObjectSubclassIsExt;
use gtk4::prelude::{GtkWindowExt, WidgetExt};
use ori_native_core::{
    NativeParent,
    native::{HasWindow, NativeWindow},
};

use crate::Platform;

impl HasWindow for Platform {
    type Window = Window;
}

impl NativeParent<Platform> for Window {
    fn replace_child(&mut self, _platform: &mut Platform, index: usize, child: &gtk4::Widget) {
        debug_assert_eq!(index, 0);

        self.set_child(Some(child));
    }
}

impl NativeWindow<Platform> for Window {
    fn build(platform: &mut Platform, contents: &gtk4::Widget) -> Self {
        let window = Self::new(&platform.application);
        window.set_child(Some(contents));
        window.show();

        window
    }

    fn teardown(self, _platform: &mut Platform) {
        self.destroy();
    }

    fn get_size(&self) -> (u32, u32) {
        (
            self.width() as u32,
            self.height() as u32,
        )
    }

    fn set_on_animation_frame(&mut self, on_frame: impl Fn(Duration) + 'static) {
        if let Some(frame_clock) = self.frame_clock() {
            let previous = self.imp().previous_frame.clone();

            frame_clock.connect_before_paint(move |frame_clock| {
                let frame_time = frame_clock.frame_time();

                if let Some(previous) = previous.replace(Some(frame_time)) {
                    let delta = frame_time - previous;

                    if delta > 100 {
                        on_frame(Duration::from_micros(delta as u64));
                    }
                }
            });
        }
    }

    fn set_on_close_requested(&mut self, on_close_requested: impl Fn() + 'static) {
        self.connect_close_request(move |_| {
            on_close_requested();
            gtk4::glib::Propagation::Stop
        });
    }

    fn set_on_resize(&mut self, on_resize: impl Fn() + 'static) {
        self.set_on_size_allocate(on_resize);
    }

    fn set_min_size(&mut self, width: u32, height: u32) {
        #[cfg(feature = "layer-shell")]
        {
            use gtk4_layer_shell::LayerShell;

            if self.is_layer_window() {
                return;
            }
        }

        self.set_size_request(width as i32, height as i32);
    }

    fn set_size(&mut self, width: u32, height: u32) {
        self.set_size_request(
            width.max(1) as i32,
            height.max(1) as i32,
        );
    }

    fn set_resizable(&mut self, resizable: bool) {
        gtk4::Window::set_resizable(self.as_ref(), resizable);
    }

    fn start_animating(&mut self) {
        if let Some(frame_clock) = self.frame_clock() {
            frame_clock.begin_updating();
            self.imp().previous_frame.set(None);
        }
    }

    fn stop_animating(&mut self) {
        if let Some(frame_clock) = self.frame_clock() {
            frame_clock.end_updating();
        }
    }
}

gtk4::glib::wrapper! {
    pub struct Window(
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

impl Window {
    pub fn new(application: &gtk4::Application) -> Self {
        let window: Window = gtk4::glib::Object::builder().build();
        window.set_application(Some(application));
        window
    }

    pub fn set_on_size_allocate(&self, on_size_allocate: impl Fn() + 'static) {
        let _ = self
            .imp()
            .on_size_allocate
            .replace(Box::new(on_size_allocate));
    }
}

mod imp {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
    use gtk4::subclass::{
        prelude::ApplicationWindowImpl,
        widget::{WidgetImpl, WidgetImplExt},
        window::WindowImpl,
    };

    pub struct ApplicationWindow {
        pub on_size_allocate: RefCell<Box<dyn Fn()>>,
        pub previous_frame:   Rc<Cell<Option<i64>>>,
    }

    impl Default for ApplicationWindow {
        fn default() -> Self {
            Self {
                on_size_allocate: RefCell::new(Box::new(|| {})),
                previous_frame:   Rc::new(Cell::new(None)),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ApplicationWindow {
        const NAME: &'static str = "OriNativeWindow";
        type Type = super::Window;
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
