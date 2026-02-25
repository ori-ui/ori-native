use std::time::Duration;

use ori::{Action, Message, Mut, Proxied, Proxy, View, ViewId, ViewMarker};

use crate::{
    Context, Lifecycle, NativeWidget, Pod, Sizing, WidgetView,
    native::{HasWindow, NativeWindow},
};

pub fn window<V>(contents: V) -> Window<V> {
    Window::new(contents)
}

pub struct Window<V> {
    contents: V,
    sizing:   Sizing,
}

impl<V> Window<V> {
    pub fn new(contents: V) -> Self {
        Window {
            contents,
            sizing: Sizing::User,
        }
    }

    pub fn sizing(mut self, sizing: Sizing) -> Self {
        self.sizing = sizing;
        self
    }
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

        let window = P::Window::build(
            &mut cx.platform,
            contents.widget.widget(),
        );

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
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        state.rebuild(cx, data, self.contents, self.sizing);
    }

    fn message(
        _element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        state.message(cx, data, message)
    }

    fn teardown(_element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        state.teardown(cx);
    }
}

#[doc(hidden)]
pub struct WindowState<P, T, V>
where
    P: HasWindow,
    V: WidgetView<P, T>,
{
    pub window:  P::Window,
    pub view_id: ViewId,

    node:   taffy::NodeId,
    sizing: Sizing,

    width:  u32,
    height: u32,

    animating: u32,

    contents: Pod<P, V::Widget>,
    state:    V::State,
}

impl<P, T, V> WindowState<P, T, V>
where
    P: HasWindow,
    V: WidgetView<P, T>,
{
    pub fn new(
        cx: &mut Context<P>,
        mut window: P::Window,
        view_id: ViewId,
        sizing: Sizing,
        contents: Pod<P, V::Widget>,
        state: V::State,
    ) -> Self {
        window.set_resizable(matches!(sizing, Sizing::User));

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

        Self {
            window,
            view_id,
            node,
            sizing,
            width,
            height,
            animating: 0,
            contents,
            state,
        }
    }

    pub fn rebuild(&mut self, cx: &mut Context<P>, data: &mut T, contents: V, sizing: Sizing) {
        cx.with_window(self.view_id, |cx| {
            contents.rebuild(
                (self.contents).as_mut(self.contents.node, &mut self.window, 0),
                &mut self.state,
                cx,
                data,
            );
        });

        (self.window).set_resizable(matches!(sizing, Sizing::User));

        self.sizing = sizing;
    }

    pub fn layout(&mut self, cx: &mut Context<P>, data: &mut T) -> Action {
        let (width, height) = self.window.get_size();

        self.width = width;
        self.height = height;

        if let Sizing::User = self.sizing {
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
            Sizing::User => taffy::Style {
                size: taffy::Size::from_lengths(width as f32, height as f32),
                ..Default::default()
            },

            Sizing::Content => {
                let mut min_size = taffy::Size::auto();

                let (min_width, min_height) = self.window.get_min_size();

                if let Some(min_width) = min_width {
                    min_size.width = taffy::Dimension::length(min_width as f32);
                }

                if let Some(min_height) = min_height {
                    min_size.height = taffy::Dimension::length(min_height as f32);
                }

                taffy::Style {
                    size: taffy::Size::auto(),
                    min_size,
                    ..Default::default()
                }
            }
        };

        let size = match self.sizing {
            Sizing::User => taffy::Size {
                width:  taffy::AvailableSpace::Definite(width as f32),
                height: taffy::AvailableSpace::Definite(height as f32),
            },

            Sizing::Content => taffy::Size::max_content(),
        };

        let _ = cx.set_layout_style(self.node, style);
        let _ = cx.compute_layout(self.node, size);

        if let Sizing::Content = self.sizing
            && let Ok(layout) = cx.get_computed_layout(self.node)
        {
            self.window.set_size(
                layout.size.width as u32,
                layout.size.height as u32,
            );
        }

        cx.with_window(self.view_id, |cx| {
            V::message(
                self.contents.as_mut(self.node, &mut self.window, 0),
                &mut self.state,
                cx,
                data,
                &mut Message::new(Lifecycle::Layout, None),
            )
        })
    }

    pub fn message(&mut self, cx: &mut Context<P>, data: &mut T, message: &mut Message) -> Action {
        if let Some(message) = message.take_targeted(self.view_id) {
            return match message {
                WindowMessage::AnimationFrame(delta) => {
                    if self.animating == 0 {
                        return Action::new();
                    }

                    let mut message = Message::new(Lifecycle::Animate(delta), None);

                    cx.with_window(self.view_id, |cx| {
                        V::message(
                            self.contents.as_mut(self.node, &mut self.window, 0),
                            &mut self.state,
                            cx,
                            data,
                            &mut message,
                        )
                    })
                }

                WindowMessage::StartAnimating => {
                    if self.animating == 0 {
                        self.window.start_animating();
                    }

                    self.animating += 1;

                    Action::new()
                }

                WindowMessage::StopAnimating => {
                    self.animating -= 1;

                    if self.animating == 0 {
                        self.window.stop_animating();
                    }

                    Action::new()
                }

                WindowMessage::CloseRequested => {
                    cx.platform.quit();

                    Action::new()
                }

                WindowMessage::Relayout => self.layout(cx, data),

                WindowMessage::Resized => {
                    let (width, height) = self.window.get_size();

                    if self.width != width || self.height != height {
                        self.layout(cx, data)
                    } else {
                        Action::new()
                    }
                }
            };
        }

        cx.with_window(self.view_id, |cx| {
            V::message(
                self.contents.as_mut(self.node, &mut self.window, 0),
                &mut self.state,
                cx,
                data,
                message,
            )
        })
    }

    pub fn teardown(self, cx: &mut Context<P>) {
        cx.with_window(self.view_id, |cx| {
            V::teardown(self.contents, self.state, cx);
        });

        self.window.teardown(&mut cx.platform);
        let _ = cx.remove_layout_node(self.node);
    }
}
