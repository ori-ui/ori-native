use std::time::Duration;

use keyboard_types::Modifiers;
use ori::{Action, Message, Mut, Proxied, Proxy, Tracker, View, ViewId, ViewMarker};

use crate::{
    Allocation, AnimateRequest, AvailableSpace, Context, Input, InputHandler, LayoutNode,
    LayoutRequest, LayoutStyle, Length, MatchKey, NavigationBar, Parent, Platform, Size, Sizing,
    StatusBar, Widget, WidgetView, native::NativeWindow, widget::WidgetMut,
};

/// [`View`] of a window.
pub fn window<T, V>(contents: V) -> Window<T, V> {
    Window::new(contents)
}

/// A window [`View`].
pub struct Window<T, V> {
    contents:   V,
    attributes: WindowAttributes<T>,
}

impl<T, V> Window<T, V> {
    /// Create new [`Window`].
    pub fn new(contents: V) -> Self {
        Window {
            contents,
            attributes: WindowAttributes::default(),
        }
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.attributes.title = title.into();
        self
    }

    /// Set the [`Sizing`].
    pub fn sizing(mut self, sizing: Sizing) -> Self {
        self.attributes.sizing = sizing;
        self
    }

    /// Set Whether the window should be decorated.
    pub fn decorated(mut self, decorated: bool) -> Self {
        self.attributes.decorated = decorated;
        self
    }

    /// Set the style of the [`StatusBar`].
    pub fn status_bar(mut self, status_bar: StatusBar) -> Self {
        self.attributes.status_bar = status_bar;
        self
    }

    /// Set the style of the [`NavigationBar`].
    pub fn navigation_bar(mut self, navigation_bar: NavigationBar) -> Self {
        self.attributes.navigation_bar = navigation_bar;
        self
    }

    /// Add an callback for when a `key` is pressed.
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

#[derive(Debug)]
pub enum WindowMessage {
    AnimationFrame(Duration),
    CloseRequested,
    Resized,
}

impl<T, V> ViewMarker for Window<T, V> {}
impl<P, T, V> View<Context<P>, T> for Window<T, V>
where
    P: Platform,
    V: WidgetView<P, T>,
{
    type Element = ();
    type State = WindowState<P, T, V>;

    fn build(self, cx: &mut Context<P>, data: &mut T) -> (Self::Element, Self::State) {
        let state = WindowState::new(
            cx,
            data,
            self.attributes,
            self.contents,
            P::Window::build,
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
        state.rebuild(cx, data, self.contents, self.attributes);
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

/// Common attributes of a [`Window`].
pub struct WindowAttributes<T> {
    /// The title of the window.
    pub title: String,

    /// The sizing mode of the window.
    pub sizing: Sizing,

    /// The input handlers of the window.
    pub input: Input<T>,

    /// Whether the window is decorated.
    pub decorated: bool,

    /// The style of the status bar.
    pub status_bar: StatusBar,

    /// The style of the navigation bar.
    pub navigation_bar: NavigationBar,
}

impl<T> Default for WindowAttributes<T> {
    fn default() -> Self {
        Self {
            title:     String::new(),
            sizing:    Sizing::User,
            input:     Default::default(),
            decorated: true,

            status_bar:     Default::default(),
            navigation_bar: Default::default(),
        }
    }
}

/// Common state of a [`Window`].
pub struct WindowState<P, T, V>
where
    P: Platform,
    V: WidgetView<P, T>,
{
    /// The native window.
    pub window: P::Window,

    /// The id of the view.
    pub view_id: ViewId,

    layout:             LayoutNode,
    allocation:         Option<Allocation>,
    content_allocation: Option<Allocation>,

    status_bar:     StatusBar,
    navigation_bar: NavigationBar,

    title:     String,
    sizing:    Sizing,
    handler:   InputHandler<T>,
    decorated: bool,

    width:  f32,
    height: f32,

    animating: u32,

    contents: V::Element,
    state:    V::State,
}

impl<P, T, V> WindowState<P, T, V>
where
    P: Platform,
    V: WidgetView<P, T>,
{
    /// Create new [`WindowState`].
    pub fn new(
        cx: &mut Context<P>,
        data: &mut T,
        attributes: WindowAttributes<T>,
        contents: V,
        build: impl FnOnce(&mut P, P::WidgetRef) -> P::Window,
    ) -> Self {
        let view_id = ViewId::next();
        cx.register(view_id);

        let node = cx.layout.add_node(&[]);
        cx.layout.set_layout(
            node,
            LayoutStyle {
                size: Size::all(Some(Length::Fract(1.0))),
                min_size: Size::all(Some(Length::Length(0.0))),
                max_size: Size::all(Some(Length::Fract(1.0))),
                ..Default::default()
            },
        );
        cx.layout.insert_root(node, view_id);

        let (contents, state) = contents.build(cx, data);

        let mut window = build(&mut cx.platform, contents.widget_ref());

        window.set_title(
            &mut cx.platform,
            attributes.title.clone(),
        );
        window.set_resizable(
            &mut cx.platform,
            matches!(attributes.sizing, Sizing::User),
        );

        let proxy = cx.proxy();

        window.set_on_resize(&mut cx.platform, {
            let proxy = proxy.cloned();

            move || {
                proxy.message(Message::new(
                    WindowMessage::Resized,
                    view_id,
                ));
            }
        });

        window.set_on_close_requested(&mut cx.platform, {
            let proxy = proxy.cloned();

            move || {
                proxy.message(Message::new(
                    WindowMessage::CloseRequested,
                    view_id,
                ));
            }
        });

        window.set_on_animation_frame(&mut cx.platform, {
            let proxy = proxy.cloned();

            move |delta| {
                proxy.message(Message::new(
                    WindowMessage::AnimationFrame(delta),
                    view_id,
                ));
            }
        });

        let (filter, handler) = attributes.input.split();

        window.set_on_key(&mut cx.platform, {
            let proxy = proxy.cloned();

            move |key, modifiers, pressed| {
                if let Some(message) = filter.filter_key(key, modifiers, pressed) {
                    proxy.message(Message::new(message, view_id));
                    true
                } else {
                    false
                }
            }
        });

        window.set_decorated(&mut cx.platform, attributes.decorated);

        window.set_status_bar(&mut cx.platform, attributes.status_bar);

        window.set_navigation_bar(
            &mut cx.platform,
            attributes.navigation_bar,
        );

        cx.layout.insert_child(node, 0, contents.layout_node());

        let (width, height) = window.get_size(&mut cx.platform);

        Self {
            window,
            view_id,
            layout: node,
            allocation: None,
            content_allocation: None,
            title: attributes.title,
            sizing: attributes.sizing,
            handler,
            decorated: attributes.decorated,
            status_bar: attributes.status_bar,
            navigation_bar: attributes.navigation_bar,
            width,
            height,
            animating: 0,
            contents,
            state,
        }
    }

    /// Rebuild `self`.
    pub fn rebuild(
        &mut self,
        cx: &mut Context<P>,
        data: &mut T,
        contents: V,
        attributes: WindowAttributes<T>,
    ) {
        let mut parent = WindowParent {
            native: &mut self.window,
            layout: self.layout,
        };

        let widget = WidgetMut::new(&mut parent, &mut self.contents);
        contents.rebuild(widget, &mut self.state, cx, data);

        let (filter, handler) = attributes.input.split();

        let proxy = cx.proxy();
        self.window.set_on_key(&mut cx.platform, {
            let view_id = self.view_id;

            move |key, modifiers, pressed| {
                if let Some(message) = filter.filter_key(key, modifiers, pressed) {
                    proxy.message(Message::new(message, view_id));
                    true
                } else {
                    false
                }
            }
        });

        self.handler = handler;

        if self.title != attributes.title {
            self.title = attributes.title.clone();
            self.window.set_title(
                &mut cx.platform,
                attributes.title.clone(),
            );
        }

        if self.sizing != attributes.sizing {
            self.sizing = attributes.sizing;
            self.window.set_resizable(
                &mut cx.platform,
                matches!(attributes.sizing, Sizing::User),
            );
        }

        if self.decorated != attributes.decorated {
            self.decorated = attributes.decorated;
            (self.window).set_decorated(&mut cx.platform, attributes.decorated);
        }

        if self.status_bar != attributes.status_bar {
            self.status_bar = attributes.status_bar;
            (self.window).set_status_bar(&mut cx.platform, self.status_bar);
        }

        if self.navigation_bar != attributes.navigation_bar {
            self.navigation_bar = attributes.navigation_bar;
            (self.window).set_navigation_bar(&mut cx.platform, self.navigation_bar);
        }
    }

    /// Compute layout and potentially resize the window.
    pub fn layout(&mut self, cx: &mut Context<P>) {
        let (width, height) = self.window.get_size(&mut cx.platform);

        self.width = width;
        self.height = height;

        if let Sizing::User = self.sizing {
            let size = Size {
                width:  AvailableSpace::Definite(0.0),
                height: AvailableSpace::Definite(0.0),
            };

            (cx.layout).compute_layout(&mut cx.platform, self.layout, size);

            if let Some(layout) = cx.layout.get_allocation(self.layout) {
                self.window.set_min_size(
                    &mut cx.platform,
                    layout.content_size.width,
                    layout.content_size.height,
                );
            }
        }

        let size = match self.sizing {
            Sizing::User => Size {
                width:  AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },

            Sizing::Content => Size {
                width:  AvailableSpace::MaxContent,
                height: AvailableSpace::MaxContent,
            },
        };

        (cx.layout).compute_layout(&mut cx.platform, self.layout, size);

        if let Some(allocation) = cx.layout.get_allocation(self.layout)
            && self.allocation != Some(allocation)
        {
            self.allocation = Some(allocation);

            if let Sizing::Content = self.sizing {
                self.window.set_size(
                    &mut cx.platform,
                    allocation.size.width,
                    allocation.size.height,
                );
            }
        }

        if let Some(allocation) = cx.layout.get_allocation(self.contents.layout_node())
            && self.content_allocation != Some(allocation)
        {
            self.content_allocation = Some(allocation);

            self.window.set_content_layout(
                &mut cx.platform,
                allocation.x,
                allocation.y,
                allocation.size.width,
                allocation.size.height,
            );
        }

        self.contents.layout(cx);
    }

    /// Handle a [`Message`].
    pub fn message(&mut self, cx: &mut Context<P>, data: &mut T, message: &mut Message) -> Action {
        if let Some(message) = message.take(self.view_id) {
            return self.handler.handle(data, message);
        }

        if let Some(message) = message.take(self.view_id) {
            return match message {
                LayoutRequest::Layout => {
                    self.layout(cx);

                    Action::new()
                }
            };
        }

        if let Some(message) = message.take(self.view_id) {
            return match message {
                AnimateRequest::Start => {
                    if self.animating == 0 {
                        self.window.start_animating(&mut cx.platform);
                    }

                    self.animating += 1;

                    Action::new()
                }

                AnimateRequest::Stop => {
                    self.animating -= 1;

                    if self.animating == 0 {
                        self.window.stop_animating(&mut cx.platform);
                    }

                    Action::new()
                }
            };
        }

        if let Some(message) = message.take(self.view_id) {
            return match message {
                WindowMessage::AnimationFrame(delta) => {
                    if self.animating == 0 {
                        return Action::new();
                    }

                    self.contents.animate(cx, delta);

                    Action::new()
                }

                WindowMessage::CloseRequested => {
                    cx.platform.quit();

                    Action::new()
                }

                WindowMessage::Resized => {
                    let (width, height) = self.window.get_size(&mut cx.platform);

                    if self.width != width || self.height != height {
                        self.layout(cx);
                    }

                    Action::new()
                }
            };
        }

        let mut parent = WindowParent {
            native: &mut self.window,
            layout: self.layout,
        };

        let widget = WidgetMut::new(&mut parent, &mut self.contents);

        V::message(
            widget,
            &mut self.state,
            cx,
            data,
            message,
        )
    }

    /// Teardown the window state.
    pub fn teardown(self, cx: &mut Context<P>) {
        V::teardown(self.contents, self.state, cx);

        self.window.teardown(&mut cx.platform);
        cx.layout.remove_node(self.layout);
        cx.unregister(self.view_id);
    }
}

struct WindowParent<'a, P>
where
    P: Platform,
{
    native: &'a mut P::Window,
    layout: LayoutNode,
}

impl<P> Parent<P> for WindowParent<'_, P>
where
    P: Platform,
{
    fn replace_child(&mut self, cx: &mut Context<P>, widgets: P::WidgetRef, layout: LayoutNode) {
        self.native.replace_contents(&mut cx.platform, widgets);
        cx.layout.replace_child(self.layout, 0, layout);
    }
}
