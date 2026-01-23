use ori::{AnyState, ViewId};
use ori_native_core::{BoxedView, NativeApp};
use tao::{
    dpi::LogicalPosition,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder, WindowId},
};
use wry::{Rect, WebView, WebViewBuilder};

use crate::{Context, context::Signal};

pub struct App {}

impl NativeApp for App {
    type Context = Context;

    fn new() -> Self {
        Self {}
    }

    fn run<T>(self, data: &mut T, mut ui: impl FnMut(&T) -> BoxedView<Self::Context, T>) {
        let mut event_loop = EventLoop::new();
        let mut context = Context::new();

        let view = ui(data);
        let (_, state) = view.build(&mut context, data);

        let mut handler = Handler {
            data,
            state,
            context,
            windows: Vec::new(),
        };

        event_loop.run_return(move |event, event_loop, control_flow| {
            handler.handle_signals(event_loop);
            handler.send_commands();

            *control_flow = ControlFlow::Wait;

            if let Event::WindowEvent {
                window_id, event, ..
            } = event
            {
                let Some(window) = handler
                    .windows
                    .iter_mut()
                    .find(|x| x.window_id == window_id)
                else {
                    return;
                };

                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }

                    WindowEvent::Resized(size) => {
                        window
                            .webview
                            .set_bounds(Rect {
                                position: LogicalPosition::new(0, 0).into(),
                                size: size.into(),
                            })
                            .unwrap();
                    }

                    _ => {}
                }
            }
        });
    }
}

pub(crate) struct Handler<'a, T> {
    data: &'a mut T,
    state: AnyState<Context, T, ()>,
    context: Context,
    windows: Vec<WindowState>,
}

impl<T> Handler<'_, T> {
    fn handle_signals(&mut self, event_loop: &EventLoopWindowTarget<()>) {
        while let Some(signal) = self.context.next_signal() {
            self.handle_signal(event_loop, signal);
        }
    }

    fn handle_signal(&mut self, event_loop: &EventLoopWindowTarget<()>, signal: Signal) {
        match signal {
            Signal::CreateWindow { view_id } => {
                self.create_window(event_loop, view_id);
            }
        }
    }

    fn send_commands(&mut self) {
        let commands = self.context.take_commands();

        if !commands.is_empty() {
            let json = serde_json::to_string(&commands).unwrap();
            let script = format!("__applyCommands({json})");

            for window in &self.windows {
                window.webview.evaluate_script(&script).unwrap();
            }
        }
    }

    fn create_window(&mut self, event_loop: &EventLoopWindowTarget<()>, view_id: ViewId) {
        let window = WindowBuilder::new()
            .with_decorations(false)
            .build(event_loop)
            .unwrap();

        #[cfg(target_os = "linux")]
        let webview = {
            use tao::platform::unix::WindowExtUnix;
            use wry::WebViewBuilderExtUnix;
            let vbox = window.default_vbox().unwrap();
            WebViewBuilder::new()
                .with_initialization_script(include_str!("index.js"))
                .with_html("")
                .build_gtk(vbox)
                .unwrap()
        };

        webview.open_devtools();

        let state = WindowState {
            view_id,
            window_id: window.id(),
            window,
            webview,
        };

        self.windows.push(state);
    }
}

struct WindowState {
    view_id: ViewId,
    window_id: WindowId,

    window: Window,
    webview: WebView,
}
