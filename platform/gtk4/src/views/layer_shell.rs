use glib::object::Cast;
use gtk4::prelude::{GtkWindowExt, WidgetExt};
use gtk4_layer_shell::{Edge, KeyboardMode, LayerShell as _};
use ori::{Action, Message, Mut, View, ViewMarker};
use ori_native_core::{
    Context, MatchKey, Modifiers, Sizing, WidgetView,
    views::{WindowAttributes, WindowState},
};

use crate::{Platform, widgets::Window};

pub fn layer_shell<T, V>(contents: V) -> LayerShell<T, V> {
    LayerShell::new(contents)
}

pub struct LayerShell<T, V> {
    contents:       V,
    attributes:     WindowAttributes<T>,
    namespace:      String,
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

impl<T, V> LayerShell<T, V> {
    pub fn new(contents: V) -> Self {
        Self {
            contents,
            attributes: WindowAttributes::default(),
            namespace: String::from("ori-native"),
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

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.attributes.title = title.into();
        self
    }

    pub fn sizing(mut self, sizing: Sizing) -> Self {
        self.attributes.sizing = sizing;
        self
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
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

    pub fn on_key<A>(
        mut self,
        key: impl MatchKey + 'static,
        mods: Modifiers,
        on_key: impl FnMut(&mut T) -> A + 'static,
    ) -> Self
    where
        A: Into<Action>,
    {
        self.attributes.input.add_key(key, mods, on_key);
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

impl<T, V> ViewMarker for LayerShell<T, V> {}
impl<T, V> View<Context<Platform>, T> for LayerShell<T, V>
where
    V: WidgetView<Platform, T>,
{
    type Element = ();
    type State = WindowState<Platform, T, V>;

    fn build(self, cx: &mut Context<Platform>, data: &mut T) -> (Self::Element, Self::State) {
        let state = WindowState::new(
            cx,
            data,
            self.attributes,
            self.contents,
            |platform, contents| {
                let window = Window::new(&platform.application);
                window.init_layer_shell();
                window.set_default_size(1, 1);
                window.set_namespace(Some(&self.namespace));

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
        state.rebuild(cx, data, self.contents, self.attributes);

        state.window.set_namespace(Some(&self.namespace));

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
