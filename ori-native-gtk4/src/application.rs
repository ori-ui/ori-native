use gtk4::{
    gio::{self, prelude::ApplicationExt},
    glib,
};
use ori::{AnyState, AnyView, Message, Proxy, View};
use ori_native_core::{BoxedEffect, Context};

use crate::Platform;

pub struct Application {}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run<T>(self, data: &mut T, ui: impl FnMut(&T) -> BoxedEffect<Platform, T>) {
        gtk4::init().unwrap();

        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

        let app = gtk4::Application::default();
        let display = gdk4::Display::default().unwrap();
        let platform = Platform::new(sender.clone(), display, app.clone());
        let mut state = State {
            data,
            build: ui,
            state: None,
            context: Context::new(platform),
            running: true,
        };

        app.connect_activate(move |_| {
            let _ = sender.send(Event::Activate);
        });

        let main_context = glib::MainContext::default();

        app.register(None::<&gio::Cancellable>).unwrap();
        app.activate();

        main_context.block_on(async {
            while state.running
                && let Some(event) = receiver.recv().await
            {
                state.handle_event(event);
            }
        });
    }
}

pub(crate) enum Event {
    Activate,
    Quit,

    Rebuild,
    Message(Message),
}

struct State<'a, T, B> {
    data:    &'a mut T,
    build:   B,
    state:   Option<AnyState<Context<Platform>, T, ()>>,
    context: Context<Platform>,
    running: bool,
}

impl<T, B> State<'_, T, B>
where
    B: FnMut(&T) -> BoxedEffect<Platform, T>,
{
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Activate => {
                let view = (self.build)(self.data);
                let (_, state) = view.build(&mut self.context, self.data);
                self.state = Some(state);
            }

            Event::Quit => {
                self.running = false;
            }

            Event::Rebuild => {
                if let Some(ref mut state) = self.state {
                    let view = (self.build)(self.data);
                    view.rebuild((), state, &mut self.context, self.data);
                }
            }

            Event::Message(mut event) => {
                if let Some(ref mut state) = self.state {
                    let action = Box::<dyn AnyView<_, _, _>>::message(
                        (),
                        state,
                        &mut self.context,
                        self.data,
                        &mut event,
                    );

                    self.context.platform.proxy.action(action);
                }
            }
        }
    }
}
