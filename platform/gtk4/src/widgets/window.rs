use std::{rc::Rc, time::Duration};

use glib::{
    object::{Cast, IsA, ObjectExt},
    subclass::types::ObjectSubclassIsExt,
};
use gtk4::prelude::{EventControllerExt, GtkWindowExt, WidgetExt};
use ori_native_core::{Key, Modifiers, NavigationBar, StatusBar, native::NativeWindow};

use crate::{Platform, key};

impl NativeWindow<Platform> for Window {
    fn build(platform: &mut Platform, contents: gtk4::Widget) -> Self {
        let window = Self::new(&platform.application);
        window.set_focusable(true);
        window.set_focus(Some(&window));
        window.set_child(&contents);
        window.show();

        let controller = gtk4::GestureClick::new();
        controller.set_propagation_phase(gtk4::PropagationPhase::Capture);

        controller.connect_pressed({
            let window = window.downgrade();

            move |_, _, _, _| {
                if let Some(window) = window.upgrade() {
                    window.set_focus(Some(&window));
                }
            }
        });

        window.add_controller(controller);

        // call on_snapshot callbacks on window snapshot
        window.set_on_snapshot({
            let on_snapshot = platform.on_snapshot.clone();

            move || {
                for on_snapshot in on_snapshot.borrow_mut().values_mut() {
                    on_snapshot();
                }
            }
        });

        window
    }

    fn teardown(self, _platform: &mut Platform) {
        self.destroy();
    }

    fn replace_contents(&mut self, _platform: &mut Platform, child: gtk4::Widget) {
        self.set_child(&child);
    }

    fn get_size(&self, _platform: &mut Platform) -> (f32, f32) {
        (
            self.width() as f32,
            self.height() as f32,
        )
    }

    fn get_preferred_size(&self, _platform: &mut Platform) -> (Option<f32>, Option<f32>) {
        #[allow(unused_mut)]
        let mut min_width = None;
        #[allow(unused_mut)]
        let mut min_height = None;

        #[cfg(feature = "layer-shell")]
        {
            use gtk4_layer_shell::{Edge, LayerShell};

            if let Some(monitor) = self.monitor() {
                use gdk4::prelude::MonitorExt;

                let geometry = monitor.geometry();

                if self.is_anchor(Edge::Left) && self.is_anchor(Edge::Right) {
                    min_width = Some(geometry.width() as f32);
                }

                if self.is_anchor(Edge::Top) && self.is_anchor(Edge::Bottom) {
                    min_height = Some(geometry.height() as f32);
                }
            }
        }

        (min_width, min_height)
    }

    fn is_decorated(&self, _platform: &mut Platform) -> bool {
        gtk4::prelude::GtkWindowExt::is_decorated(self)
    }

    fn set_on_animation_frame(
        &mut self,
        _platform: &mut Platform,
        on_frame: impl Fn(Duration) + 'static,
    ) {
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

    fn set_on_close_requested(
        &mut self,
        _platform: &mut Platform,
        on_close_requested: impl Fn() + 'static,
    ) {
        self.connect_close_request(move |_| {
            on_close_requested();
            gtk4::glib::Propagation::Stop
        });
    }

    fn set_on_resize(&mut self, _platform: &mut Platform, on_resize: impl Fn() + 'static) {
        self.set_on_size_allocate(on_resize);
    }

    fn set_on_key(
        &mut self,
        _platform: &mut Platform,
        on_key: impl Fn(Key, Modifiers, bool) -> bool + 'static,
    ) {
        for controller in self.observe_controllers().into_iter() {
            if let Ok(controller) = controller
                && let Ok(controller) = controller.dynamic_cast::<gtk4::EventControllerKey>()
            {
                self.remove_controller(&controller);
            }
        }

        let controller = gtk4::EventControllerKey::new();
        let on_key = Rc::new(on_key);

        controller.connect_key_pressed({
            let window = self.downgrade();
            let on_key = on_key.clone();

            move |_, key, _code, modifiers| {
                let ori_key = key::convert_key(key);
                let modifiers = key::convert_modifiers(modifiers);

                if on_key(ori_key, modifiers, true) {
                    glib::Propagation::Stop
                } else if key == gdk4::Key::Escape
                    && let Some(window) = window.upgrade()
                    && let Some(focus) = window.focus()
                    && focus != window
                {
                    window.set_focus(Some(&window));
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        });

        controller.connect_key_released({
            let on_key = on_key.clone();

            move |_, key, _code, modifiers| {
                let key = key::convert_key(key);
                let modifiers = key::convert_modifiers(modifiers);
                on_key(key, modifiers, false);
            }
        });

        self.add_controller(controller);
    }

    fn set_title(&mut self, _platform: &mut Platform, title: String) {
        gtk4::ApplicationWindow::set_title(self.as_ref(), Some(&title));
    }

    fn set_content_layout(
        &mut self,
        _platform: &mut Platform,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.imp().group.layout(0, x, y, width, height);
    }

    fn set_min_size(&mut self, _platform: &mut Platform, width: f32, height: f32) {
        #[cfg(feature = "layer-shell")]
        {
            use gtk4_layer_shell::LayerShell;

            if self.is_layer_window() {
                return;
            }
        }

        self.set_size_request(
            width.round() as i32,
            height.round() as i32,
        );
    }

    fn set_size(&mut self, _platform: &mut Platform, width: f32, height: f32) {
        self.set_default_size(
            width.max(1.0).round() as i32,
            height.max(1.0).round() as i32,
        );
    }

    fn set_resizable(&mut self, _platform: &mut Platform, resizable: bool) {
        gtk4::Window::set_resizable(self.as_ref(), resizable);
    }

    fn start_animating(&mut self, _platform: &mut Platform) {
        if let Some(frame_clock) = self.frame_clock() {
            frame_clock.begin_updating();
            self.imp().previous_frame.set(None);
        }
    }

    fn stop_animating(&mut self, _platform: &mut Platform) {
        if let Some(frame_clock) = self.frame_clock() {
            frame_clock.end_updating();
        }
    }

    fn set_status_bar(&mut self, _platform: &mut Platform, _bar: StatusBar) {}

    fn set_navigation_bar(&mut self, _platform: &mut Platform, _bar: NavigationBar) {}

    fn set_decorated(&mut self, _platform: &mut Platform, decorated: bool) {
        gtk4::prelude::GtkWindowExt::set_decorated(self, decorated);
    }
}

glib::wrapper! {
    pub struct Window(
        ObjectSubclass<imp::ApplicationWindow>)
        @extends
            gtk4::ApplicationWindow,
            gtk4::Window,
            gtk4::Widget,
        @implements
            gtk4::Buildable,
            gtk4::ConstraintTarget,
            gtk4::Root,
            gtk4::Native,
            gtk4::ShortcutManager,
            gtk4::gio::ActionGroup,
            gtk4::gio::ActionMap;
}

impl Window {
    pub fn new(application: &gtk4::Application) -> Self {
        let window: Window = glib::Object::new();
        window.set_application(Some(application));
        gtk4::Window::set_child(
            window.as_ref(),
            Some(&window.imp().group),
        );

        window
    }

    pub fn set_on_size_allocate(&self, on_size_allocate: impl Fn() + 'static) {
        let _ = self
            .imp()
            .on_size_allocate
            .replace(Box::new(on_size_allocate));
    }

    pub fn set_on_snapshot(&self, on_snapshot: impl Fn() + 'static) {
        let _ = self.imp().on_snapshot.replace(Box::new(on_snapshot));
    }

    pub fn set_child(&self, child: &impl IsA<gtk4::Widget>) {
        if !self.imp().group.is_empty() {
            self.imp().group.remove(0);
        }

        self.imp().group.add(child.as_ref().clone());
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

    use crate::widgets::Group;

    pub struct ApplicationWindow {
        pub group:            Group,
        pub on_size_allocate: RefCell<Box<dyn Fn()>>,
        pub on_snapshot:      RefCell<Box<dyn Fn()>>,
        pub previous_frame:   Rc<Cell<Option<i64>>>,
    }

    impl Default for ApplicationWindow {
        fn default() -> Self {
            Self {
                group:            Group::new(),
                on_size_allocate: RefCell::new(Box::new(|| {})),
                on_snapshot:      RefCell::new(Box::new(|| {})),
                previous_frame:   Rc::new(Cell::new(None)),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ApplicationWindow {
        const NAME: &'static str = "OriWindow";
        type Type = super::Window;
        type ParentType = gtk4::ApplicationWindow;
    }

    impl ObjectImpl for ApplicationWindow {
        fn dispose(&self) {}
    }

    impl WidgetImpl for ApplicationWindow {
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);

            let on_size_allocate = self.on_size_allocate.borrow();
            on_size_allocate();
        }

        fn measure(&self, orientation: gtk4::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.parent_measure(orientation, for_size);

            (0, 0, -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            self.parent_snapshot(snapshot);

            let on_snapshot = self.on_snapshot.borrow();
            on_snapshot();
        }
    }

    impl WindowImpl for ApplicationWindow {}

    impl ApplicationWindowImpl for ApplicationWindow {}
}
