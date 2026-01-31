use std::time::Duration;

use ori::{Action, Message, Mut, Proxied, Proxy, View, ViewId, ViewMarker};

use crate::{
    Context, Lifecycle, NativeWidget, Pod, WidgetView,
    native::{HasWindow, NativeWindow},
    views::AnimationFrame,
};

pub fn window<V>(contents: V) -> Window<V> {
    Window::new(contents)
}

pub struct Window<V> {
    contents: V,
    sizing:   WindowSizing,

    #[cfg(feature = "layer-shell")]
    layer_shell: Option<LayerShell>,
}

impl<V> Window<V> {
    pub fn new(contents: V) -> Self {
        Window {
            contents,
            sizing: WindowSizing::User,

            #[cfg(feature = "layer-shell")]
            layer_shell: None,
        }
    }

    pub fn sizing(mut self, sizing: WindowSizing) -> Self {
        self.sizing = sizing;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn layer_shell(mut self, layer_shell: impl Into<Option<LayerShell>>) -> Self {
        self.layer_shell = layer_shell.into();
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WindowSizing {
    User,
    Content,
}

#[derive(Debug)]
pub enum WindowMessage {
    AnimationFrame(Duration),
    StartAnimating,
    StopAnimating,
    CloseRequested,
    Relayout,
    Resized,
}

impl<V> ViewMarker for Window<V> {}
impl<P, T, V> View<Context<P>, T> for Window<V>
where
    P: HasWindow + Proxied,
    V: WidgetView<P, T>,
{
    type Element = ();
    type State = WindowState<P, T, V>;

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let view_id = ViewId::next();

        let (contents, state) = cx.with_window(view_id, |cx| {
            self.contents.build(cx, data)
        });

        #[cfg(not(feature = "layer-shell"))]
        let mut window = P::Window::build(
            &mut cx.platform,
            contents.widget.widget(),
        );

        #[cfg(feature = "layer-shell")]
        let mut window = if let Some(layer_shell) = self.layer_shell {
            P::Window::build_layer_shell(
                &mut cx.platform,
                contents.widget.widget(),
                layer_shell,
            )
        } else {
            P::Window::build(
                &mut cx.platform,
                contents.widget.widget(),
            )
        };

        window.set_resizable(matches!(
            self.sizing,
            WindowSizing::User
        ));

        window.set_on_resize({
            let proxy = cx.proxy();

            move || {
                proxy.message(Message::new(
                    WindowMessage::Resized,
                    view_id,
                ));
            }
        });

        window.set_on_close_requested({
            let proxy = cx.proxy();

            move || {
                proxy.message(Message::new(
                    WindowMessage::CloseRequested,
                    view_id,
                ));
            }
        });

        window.set_on_animation_frame({
            let proxy = cx.proxy();

            move |delta| {
                proxy.message(Message::new(
                    WindowMessage::AnimationFrame(delta),
                    view_id,
                ));
            }
        });

        let node = cx.new_layout_node(Default::default(), &[contents.node]);

        let (width, height) = window.get_size();

        let state = WindowState {
            node,
            view_id,
            window,
            sizing: self.sizing,

            width,
            height,

            animating: 0,

            contents,
            state,
        };

        ((), state)
    }

    fn rebuild(
        self,
        _element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        cx.with_window(state.view_id, |cx| {
            self.contents.rebuild(
                state.contents.as_mut(state.contents.node),
                &mut state.state,
                cx,
                data,
            );
        });

        state.window.set_resizable(matches!(
            self.sizing,
            WindowSizing::User
        ));

        state.sizing = self.sizing;
    }

    fn message(
        _element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(message) = message.take_targeted(state.view_id) {
            match message {
                WindowMessage::AnimationFrame(delta) => {
                    if state.animating == 0 {
                        return Action::new();
                    }

                    let mut message = Message::new(AnimationFrame(delta), None);

                    cx.with_window(state.view_id, |cx| {
                        V::message(
                            state.contents.as_mut(state.node),
                            &mut state.state,
                            cx,
                            data,
                            &mut message,
                        )
                    })
                }

                WindowMessage::StartAnimating => {
                    if state.animating == 0 {
                        state.window.start_animating();
                    }

                    state.animating += 1;

                    Action::new()
                }

                WindowMessage::StopAnimating => {
                    state.animating -= 1;

                    if state.animating == 0 {
                        state.window.stop_animating();
                    }

                    Action::new()
                }

                WindowMessage::CloseRequested => {
                    cx.platform.quit();

                    Action::new()
                }

                WindowMessage::Relayout => state.layout(cx, data),

                WindowMessage::Resized => {
                    let (width, height) = state.window.get_size();

                    if state.width != width || state.height != height {
                        state.layout(cx, data)
                    } else {
                        Action::new()
                    }
                }
            }
        } else {
            cx.with_window(state.view_id, |cx| {
                V::message(
                    state.contents.as_mut(state.node),
                    &mut state.state,
                    cx,
                    data,
                    message,
                )
            })
        }
    }

    fn teardown(_element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        cx.with_window(state.view_id, |cx| {
            V::teardown(state.contents, state.state, cx);
        });

        state.window.teardown(&mut cx.platform);
        let _ = cx.remove_layout_node(state.node);
    }
}

#[doc(hidden)]
pub struct WindowState<P, T, V>
where
    P: HasWindow,
    V: WidgetView<P, T>,
{
    node:    taffy::NodeId,
    view_id: ViewId,
    window:  P::Window,
    sizing:  WindowSizing,

    width:  u32,
    height: u32,

    animating: u32,

    contents: Pod<V::Widget>,
    state:    V::State,
}

impl<P, T, V> WindowState<P, T, V>
where
    P: HasWindow,
    V: WidgetView<P, T>,
{
    fn layout(&mut self, cx: &mut Context<P>, data: &mut T) -> Action {
        let (width, height) = self.window.get_size();

        self.width = width;
        self.height = height;

        if let WindowSizing::User = self.sizing {
            let style = taffy::Style {
                max_size: taffy::Size::from_lengths(0.0, 0.0),
                ..Default::default()
            };

            let size = taffy::Size {
                width:  taffy::AvailableSpace::MinContent,
                height: taffy::AvailableSpace::MinContent,
            };

            let _ = cx.set_layout_style(self.node, style);
            let _ = cx.compute_layout(self.node, size);

            if let Ok(layout) = cx.get_computed_layout(self.node) {
                self.window.set_min_size(
                    layout.content_size.width as u32,
                    layout.content_size.height as u32,
                );
            }
        }

        let style = match self.sizing {
            WindowSizing::User => taffy::Style {
                size: taffy::Size::from_lengths(width as f32, height as f32),
                ..Default::default()
            },

            WindowSizing::Content => taffy::Style {
                size: taffy::Size::auto(),
                ..Default::default()
            },
        };

        let size = match self.sizing {
            WindowSizing::User => taffy::Size {
                width:  taffy::AvailableSpace::Definite(width as f32),
                height: taffy::AvailableSpace::Definite(height as f32),
            },

            WindowSizing::Content => taffy::Size::max_content(),
        };

        let _ = cx.set_layout_style(self.node, style);
        let _ = cx.compute_layout(self.node, size);

        if let WindowSizing::Content = self.sizing
            && let Ok(layout) = cx.get_computed_layout(self.node)
        {
            self.window.set_size(
                layout.size.width as u32,
                layout.size.height as u32,
            );
        }

        cx.with_window(self.view_id, |cx| {
            V::message(
                self.contents.as_mut(self.node),
                &mut self.state,
                cx,
                data,
                &mut Message::new(Lifecycle::Layout, None),
            )
        })
    }
}

#[cfg(feature = "layer-shell")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LayerShell {
    pub layer:          Layer,
    pub exclusive_zone: ExclusiveZone,
    pub monitor:        Option<u32>,
    pub keyboard:       KeyboardInput,
    pub margin_top:     i32,
    pub margin_right:   i32,
    pub margin_bottom:  i32,
    pub margin_left:    i32,
    pub anchor_top:     bool,
    pub anchor_right:   bool,
    pub anchor_bottom:  bool,
    pub anchor_left:    bool,
}

#[cfg(feature = "layer-shell")]
impl Default for LayerShell {
    fn default() -> Self {
        Self {
            layer:          Layer::Top,
            exclusive_zone: ExclusiveZone::Auto,
            monitor:        None,
            keyboard:       KeyboardInput::Never,
            margin_top:     0,
            margin_right:   0,
            margin_bottom:  0,
            margin_left:    0,
            anchor_top:     false,
            anchor_right:   false,
            anchor_bottom:  false,
            anchor_left:    false,
        }
    }
}

#[cfg(feature = "layer-shell")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyboardInput {
    Never,
    Exclusive,
    OnDemand,
}

#[cfg(feature = "layer-shell")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

#[cfg(feature = "layer-shell")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExclusiveZone {
    Auto,
    Fixed(i32),
}
