use gtk4::prelude::WidgetExt;
use gtk4_session_lock::Instance;
use ori::{Action, Message, Mut, View, ViewMarker};
use ori_native_core::{
    Context, WidgetView,
    views::{WindowAttributes, WindowState},
};

use crate::{Platform, widgets::Window};

pub fn session_lock<V>(contents: V, instance: Instance, monitor: gdk4::Monitor) -> SessionLock<V> {
    SessionLock {
        contents,
        instance,
        monitor,
    }
}

pub struct SessionLock<V> {
    contents: V,
    instance: Instance,
    monitor:  gdk4::Monitor,
}

impl<V> SessionLock<V> {
    pub fn new(contents: V, instance: Instance, monitor: gdk4::Monitor) -> Self {
        Self {
            contents,
            instance,
            monitor,
        }
    }
}

impl<V> ViewMarker for SessionLock<V> {}
impl<T, V> View<Context<Platform>, T> for SessionLock<V>
where
    V: WidgetView<Platform, T>,
{
    type Element = ();
    type State = WindowState<Platform, T, V>;

    fn build(self, cx: &mut Context<Platform>, data: &mut T) -> (Self::Element, Self::State) {
        let state = WindowState::new(
            cx,
            data,
            WindowAttributes::default(),
            self.contents,
            |platform, contents| {
                let window = Window::new(&platform.application);

                (self.instance).assign_window_to_monitor(&window, &self.monitor);
                window.set_size_request(1, 1);

                window.set_child(&contents);
                window.show();
                window
            },
        );

        ((), state)
    }

    fn rebuild(
        self,
        _element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<Platform>,
        data: &mut T,
    ) {
        state.rebuild(
            cx,
            data,
            self.contents,
            WindowAttributes::default(),
        );
    }

    fn message(
        _element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<Platform>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        state.message(cx, data, message)
    }

    fn teardown(_element: Self::Element, state: Self::State, cx: &mut Context<Platform>) {
        state.teardown(cx);
    }
}
