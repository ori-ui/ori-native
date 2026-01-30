use gtk4::prelude::ApplicationExt;
use ori::{Effect, Message, Proxied};
use ori_native_core::Context;

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

    pub fn run<T, V>(self, data: &mut T, ui: impl FnMut(&T) -> V)
    where
        V: Effect<Context<Platform>, T>,
    {
        Self::init_log();
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

                // handle all events before giving control back to gtk
                while let Ok(event) = receiver.try_recv() {
                    state.handle_event(event);
                }
            }

            state.teardown();
        });
    }

    fn init_log() {
        glib::log_set_writer_func(|level, fields| {
            let mut message = None;

            for field in fields {
                if field.key() == "MESSAGE" {
                    message = field.value_str()
                }
            }

            let message = message.unwrap_or("<no message>");

            match level {
                glib::LogLevel::Error | glib::LogLevel::Critical => {
                    tracing::error!(target: "glib", "{message}")
                }

                glib::LogLevel::Message | glib::LogLevel::Info => {
                    tracing::info!(target: "glib", "{message}")
                }

                glib::LogLevel::Warning => tracing::warn!(target: "glib", "{message}"),
                glib::LogLevel::Debug => tracing::debug!(target: "glib", "{message}"),
            }

            glib::LogWriterOutput::Handled
        });
    }
}

#[derive(Debug)]
pub(crate) enum Event {
    Activate,
    Quit,

    Rebuild,
    Message(Message),
}

struct State<'a, T, V, B>
where
    V: Effect<Context<Platform>, T>,
{
    data:    &'a mut T,
    build:   B,
    state:   Option<V::State>,
    context: Context<Platform>,
    running: bool,
}

impl<T, V, B> State<'_, T, V, B>
where
    V: Effect<Context<Platform>, T>,
    B: FnMut(&T) -> V,
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
                    let mut action = V::message(
                        (),
                        state,
                        &mut self.context,
                        self.data,
                        &mut event,
                    );

                    if action.take_rebuild() {
                        let view = (self.build)(self.data);
                        view.rebuild((), state, &mut self.context, self.data);
                    }

                    action.rebuild = false;
                    self.context.send_action(action);
                }
            }
        }
    }

    fn teardown(mut self) {
        if let Some(state) = self.state {
            V::teardown((), state, &mut self.context);
        }
    }
}
