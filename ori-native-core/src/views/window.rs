use ori::{Action, Message, Mut, Proxied, Proxy, View, ViewId, ViewMarker};

use crate::{
    Context, Pod, Shadow, ShadowView,
    native::{HasWindow, NativeWindow},
};

pub fn window<V>(contents: V) -> Window<V> {
    Window { contents }
}

pub struct Window<V> {
    contents: V,
}

pub enum WindowMessage {
    CloseRequested,
    Relayout,
    Resized,
}

impl<V> ViewMarker for Window<V> {}
impl<P, T, V> View<Context<P>, T> for Window<V>
where
    P: HasWindow + Proxied,
    V: ShadowView<P, T>,
{
    type Element = ();
    type State = WindowState<P, T, V>;

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let view_id = ViewId::next();

        cx.with_layout_controller(view_id, |cx| {
            let (contents, state) = self.contents.build(cx, data);
            let mut window = P::Window::build(
                &mut cx.platform,
                contents.shadow.widget(),
            );

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

            let node = cx.new_layout_node(Default::default(), &[contents.node]);

            let (width, height) = window.get_size();

            let mut state = WindowState {
                node,
                view_id,
                window,

                width,
                height,

                contents,
                state,
            };

            state.layout(cx);

            ((), state)
        })
    }

    fn rebuild(
        self,
        _element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
    ) {
        cx.with_layout_controller(state.view_id, |cx| {
            self.contents.rebuild(
                state.contents.as_mut(state.contents.node),
                &mut state.state,
                cx,
                data,
            );

            state.layout(cx);
        })
    }

    fn message(
        _element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut Context<P>,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        cx.with_layout_controller(state.view_id, |cx| {
            match message.take_targeted(state.view_id) {
                Some(WindowMessage::CloseRequested) => {
                    cx.platform.quit();

                    Action::new()
                }

                Some(WindowMessage::Relayout) => {
                    state.layout(cx);

                    Action::new()
                }

                Some(WindowMessage::Resized) => {
                    let (width, height) = state.window.get_size();

                    if state.width != width || state.height != height {
                        state.layout(cx);
                    }

                    Action::new()
                }

                None => V::message(
                    state.contents.as_mut(state.node),
                    &mut state.state,
                    cx,
                    data,
                    message,
                ),
            }
        })
    }

    fn teardown(_element: Self::Element, state: Self::State, cx: &mut Context<P>) {
        cx.with_layout_controller(state.view_id, |cx| {
            V::teardown(state.contents, state.state, cx);
            state.window.teardown(&mut cx.platform);
            let _ = cx.remove_layout_node(state.node);
        })
    }
}

#[doc(hidden)]
pub struct WindowState<P, T, V>
where
    P: HasWindow,
    V: ShadowView<P, T>,
{
    node:    taffy::NodeId,
    view_id: ViewId,
    window:  P::Window,

    width:  u32,
    height: u32,

    contents: Pod<V::Shadow>,
    state:    V::State,
}

impl<P, T, V> WindowState<P, T, V>
where
    P: HasWindow,
    V: ShadowView<P, T>,
{
    fn layout(&mut self, cx: &mut Context<P>) {
        let (width, height) = self.window.get_size();

        self.width = width;
        self.height = height;

        let style = taffy::Style {
            max_size: taffy::Size::from_lengths(0.0, 0.0),
            ..Default::default()
        };

        let size = taffy::Size {
            width:  taffy::AvailableSpace::MinContent,
            height: taffy::AvailableSpace::MinContent,
        };

        cx.set_layout_style(self.node, style).unwrap();
        cx.compute_layout(self.node, size).unwrap();

        if let Ok(layout) = cx.get_computed_layout(self.node) {
            self.window.set_min_size(
                layout.content_size.width as u32,
                layout.content_size.height as u32,
            );
        }

        let style = taffy::Style {
            size: taffy::Size::from_lengths(width as f32, height as f32),
            ..Default::default()
        };

        let size = taffy::Size {
            width:  taffy::AvailableSpace::Definite(width as f32),
            height: taffy::AvailableSpace::Definite(height as f32),
        };

        cx.set_layout_style(self.node, style).unwrap();
        cx.compute_layout(self.node, size).unwrap();

        self.contents.shadow.layout(cx, self.contents.node);
    }
}
