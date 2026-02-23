use glib::object::Cast;
use gtk4::prelude::{GtkWindowExt, WidgetExt};
use gtk4_layer_shell::{Edge, KeyboardMode, LayerShell as _};
use ori::{Action, Message, Mut, View, ViewId, ViewMarker};
use ori_native_core::{Context, NativeWidget, Sizing, WidgetView, views::WindowState};

use crate::{Platform, widgets::Window};

pub fn layer_shell<V>(contents: V) -> LayerShell<V> {
    LayerShell::new(contents)
}

#[derive(Debug)]
pub struct LayerShell<V> {
    contents:       V,
    sizing:         Sizing,
    layer:          Layer,
    exclusive_zone: ExclusiveZone,
    monitor:        Option<gdk4::Monitor>,
    keyboard:       KeyboardInput,
    margin_top:     i32,
    margin_right:   i32,
    margin_bottom:  i32,
    margin_left:    i32,
    anchor_top:     bool,
    anchor_right:   bool,
    anchor_bottom:  bool,
    anchor_left:    bool,
}

impl<V> LayerShell<V> {
    pub fn new(contents: V) -> Self {
        Self {
            contents,
            sizing: Sizing::User,
            layer: Layer::Top,
            exclusive_zone: ExclusiveZone::Auto,
            monitor: None,
            keyboard: KeyboardInput::Never,
            margin_top: 0,
            margin_right: 0,
            margin_bottom: 0,
            margin_left: 0,
            anchor_top: false,
            anchor_right: false,
            anchor_bottom: false,
            anchor_left: false,
        }
    }

    pub fn sizing(mut self, sizing: Sizing) -> Self {
        self.sizing = sizing;
        self
    }

    pub fn layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    pub fn exclusive_zone(mut self, zone: ExclusiveZone) -> Self {
        self.exclusive_zone = zone;
        self
    }

    pub fn monitor(mut self, monitor: Option<gdk4::Monitor>) -> Self {
        self.monitor = monitor;
        self
    }

    pub fn keyboard(mut self, keyboard: KeyboardInput) -> Self {
        self.keyboard = keyboard;
        self
    }

    pub fn margin_top(mut self, margin: i32) -> Self {
        self.margin_top = margin;
        self
    }

    pub fn margin_right(mut self, margin: i32) -> Self {
        self.margin_right = margin;
        self
    }

    pub fn margin_bottom(mut self, margin: i32) -> Self {
        self.margin_bottom = margin;
        self
    }

    pub fn margin_left(mut self, margin: i32) -> Self {
        self.margin_left = margin;
        self
    }

    pub fn anchor_top(mut self, anchor: bool) -> Self {
        self.anchor_top = anchor;
        self
    }

    pub fn anchor_right(mut self, anchor: bool) -> Self {
        self.anchor_right = anchor;
        self
    }

    pub fn anchor_bottom(mut self, anchor: bool) -> Self {
        self.anchor_bottom = anchor;
        self
    }

    pub fn anchor_left(mut self, anchor: bool) -> Self {
        self.anchor_left = anchor;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyboardInput {
    Never,
    Exclusive,
    OnDemand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExclusiveZone {
    Auto,
    Fixed(i32),
}

impl<V> ViewMarker for LayerShell<V> {}
impl<V, T> View<Context<Platform>, T> for LayerShell<V>
where
    V: WidgetView<Platform, T>,
{
    type Element = ();
    type State = WindowState<Platform, T, V>;

    fn build(self, cx: &mut Context<Platform>, data: &mut T) -> (Self::Element, Self::State) {
        let window = Window::new(&cx.platform.application);
        window.init_layer_shell();
        window.set_size_request(1, 1);

        if let Some(monitor) = self.monitor {
            window.set_monitor(monitor.downcast_ref());
        }

        window.set_keyboard_mode(match self.keyboard {
            KeyboardInput::Never => KeyboardMode::None,
            KeyboardInput::Exclusive => KeyboardMode::Exclusive,
            KeyboardInput::OnDemand => KeyboardMode::OnDemand,
        });

        window.set_layer(match self.layer {
            Layer::Background => gtk4_layer_shell::Layer::Background,
            Layer::Bottom => gtk4_layer_shell::Layer::Bottom,
            Layer::Top => gtk4_layer_shell::Layer::Top,
            Layer::Overlay => gtk4_layer_shell::Layer::Overlay,
        });

        window.set_margin(Edge::Top, self.margin_top);
        window.set_margin(Edge::Right, self.margin_right);
        window.set_margin(Edge::Bottom, self.margin_bottom);
        window.set_margin(Edge::Left, self.margin_left);

        window.set_anchor(Edge::Top, self.anchor_top);
        window.set_anchor(Edge::Right, self.anchor_right);
        window.set_anchor(Edge::Bottom, self.anchor_bottom);
        window.set_anchor(Edge::Left, self.anchor_left);

        match self.exclusive_zone {
            ExclusiveZone::Auto => {
                window.auto_exclusive_zone_enable();
            }

            ExclusiveZone::Fixed(size) => {
                window.set_exclusive_zone(size);
            }
        }

        let view_id = ViewId::next();

        let (contents, state) = cx.with_window(view_id, |cx| {
            self.contents.build(cx, data)
        });

        window.set_child(Some(contents.widget.widget()));
        window.show();

        let state = WindowState::new(
            cx,
            window,
            view_id,
            self.sizing,
            contents,
            state,
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
        state.rebuild(cx, data, self.contents, self.sizing);

        if let Some(monitor) = self.monitor {
            state.window.set_monitor(monitor.downcast_ref());
        }

        state.window.set_keyboard_mode(match self.keyboard {
            KeyboardInput::Never => KeyboardMode::None,
            KeyboardInput::Exclusive => KeyboardMode::Exclusive,
            KeyboardInput::OnDemand => KeyboardMode::OnDemand,
        });

        state.window.set_layer(match self.layer {
            Layer::Background => gtk4_layer_shell::Layer::Background,
            Layer::Bottom => gtk4_layer_shell::Layer::Bottom,
            Layer::Top => gtk4_layer_shell::Layer::Top,
            Layer::Overlay => gtk4_layer_shell::Layer::Overlay,
        });

        state.window.set_margin(Edge::Top, self.margin_top);
        state.window.set_margin(Edge::Right, self.margin_right);
        state.window.set_margin(Edge::Bottom, self.margin_bottom);
        state.window.set_margin(Edge::Left, self.margin_left);

        state.window.set_anchor(Edge::Top, self.anchor_top);
        state.window.set_anchor(Edge::Right, self.anchor_right);
        state.window.set_anchor(Edge::Bottom, self.anchor_bottom);
        state.window.set_anchor(Edge::Left, self.anchor_left);

        match self.exclusive_zone {
            ExclusiveZone::Auto => {
                state.window.auto_exclusive_zone_enable();
            }

            ExclusiveZone::Fixed(size) => {
                state.window.set_exclusive_zone(size);
            }
        }
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
