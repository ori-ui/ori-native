use std::{cell::Cell, rc::Rc, time::Duration};

use glib::subclass::types::ObjectSubclassIsExt;
use gtk4::prelude::{GtkWindowExt, WidgetExt};
use ori_native_core::native::{HasWindow, NativeWindow};

use crate::Platform;

pub struct Window {
    window:         ApplicationWindow,
    previous_frame: Rc<Cell<Option<i64>>>,
}

impl NativeWindow<Platform> for Window {
    fn build(platform: &mut Platform, contents: &gtk4::Widget) -> Self {
        let window = ApplicationWindow::new(&platform.application);
        window.set_child(Some(contents));
        window.show();

        Self {
            window,
            previous_frame: Default::default(),
        }
    }

    #[cfg(feature = "layer-shell")]
    fn build_layer_shell(
        platform: &mut Platform,
        contents: &gtk4::Widget,
        layer_shell: ori_native_core::views::LayerShell,
    ) -> Self
    where
        Self: Sized,
    {
        use gdk4::prelude::DisplayExt;
        use glib::object::Cast;
        use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
        use ori_native_core::views;

        let window = ApplicationWindow::new(&platform.application);
        window.init_layer_shell();
        window.set_size_request(1, 1);

        if let Some(index) = layer_shell.monitor
            && let Some(Ok(monitor)) = platform.display.monitors().into_iter().nth(index as usize)
        {
            window.set_monitor(monitor.downcast_ref());
        }

        window.set_keyboard_mode(match layer_shell.keyboard {
            views::KeyboardInput::Never => KeyboardMode::None,
            views::KeyboardInput::Exclusive => KeyboardMode::Exclusive,
            views::KeyboardInput::OnDemand => KeyboardMode::OnDemand,
        });

        window.set_layer(match layer_shell.layer {
            views::Layer::Background => Layer::Background,
            views::Layer::Bottom => Layer::Bottom,
            views::Layer::Top => Layer::Top,
            views::Layer::Overlay => Layer::Overlay,
        });

        window.set_margin(Edge::Top, layer_shell.margin_top);
        window.set_margin(Edge::Right, layer_shell.margin_right);
        window.set_margin(Edge::Bottom, layer_shell.margin_bottom);
        window.set_margin(Edge::Left, layer_shell.margin_left);

        window.set_anchor(Edge::Top, layer_shell.anchor_top);
        window.set_anchor(Edge::Right, layer_shell.anchor_right);
        window.set_anchor(Edge::Bottom, layer_shell.anchor_bottom);
        window.set_anchor(Edge::Left, layer_shell.anchor_left);

        match layer_shell.exclusive_zone {
            views::ExclusiveZone::Auto => {
                window.auto_exclusive_zone_enable();
            }

            views::ExclusiveZone::Fixed(size) => {
                window.set_exclusive_zone(size);
            }
        }

        window.set_child(Some(contents));
        window.show();

        Self {
            window,
            previous_frame: Default::default(),
        }
    }

    fn teardown(self, _platform: &mut Platform) {}

    fn get_size(&self) -> (u32, u32) {
        (
            self.window.width() as u32,
            self.window.height() as u32,
        )
    }

    fn set_on_animation_frame(&mut self, on_frame: impl Fn(Duration) + 'static) {
        if let Some(frame_clock) = self.window.frame_clock() {
            let previous = self.previous_frame.clone();

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
        self.window.connect_close_request(move |_| {
            on_close_requested();
            gtk4::glib::Propagation::Stop
        });
    }

    fn set_on_resize(&mut self, on_resize: impl Fn() + 'static) {
        self.window.set_on_size_allocate(on_resize);
    }

    fn set_min_size(&mut self, width: u32, height: u32) {
        #[cfg(feature = "layer-shell")]
        {
            use gtk4_layer_shell::LayerShell;

            if self.window.is_layer_window() {
                return;
            }
        }

        self.window.set_size_request(width as i32, height as i32);
    }

    fn set_size(&mut self, width: u32, height: u32) {
        self.window.set_size_request(
            width.max(1) as i32,
            height.max(1) as i32,
        );
    }

    fn set_resizable(&mut self, resizable: bool) {
        self.window.set_resizable(resizable);
    }

    fn start_animating(&mut self) {
        if let Some(frame_clock) = self.window.frame_clock() {
            frame_clock.begin_updating();
            self.previous_frame.set(None);
        }
    }

    fn stop_animating(&mut self) {
        if let Some(frame_clock) = self.window.frame_clock() {
            frame_clock.end_updating();
        }
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

    use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
    use gtk4::subclass::{
        prelude::ApplicationWindowImpl,
        widget::{WidgetImpl, WidgetImplExt},
        window::WindowImpl,
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
