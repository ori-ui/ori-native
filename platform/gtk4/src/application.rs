use std::{
    error, fmt, io,
    sync::atomic::{AtomicBool, Ordering},
};

use gtk4::prelude::ApplicationExt;
use ori::{Effect, Message, Proxied};
use ori_native_core::{Context, PressableEvent, TextInputEvent};
use tracing_subscriber::layer::SubscriberExt;

use crate::Platform;

#[derive(Debug)]
pub enum Error {
    GtkInit,
    GdkNoDisplay,
    GdkApplicationRegister,
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GtkInit => write!(f, "gtk4 initialization failed"),
            Self::GdkNoDisplay => write!(f, "gdk4 display could not be retrieved"),
            Self::GdkApplicationRegister => write!(
                f,
                "gtk4 application could not be registered",
            ),
            Self::Io(error) => write!(f, "io error: {error}"),
        }
    }
}

impl error::Error for Error {}

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

    pub fn run<T, V>(self, data: &mut T, ui: impl FnMut(&T) -> V) -> Result<(), Error>
    where
        V: Effect<Context<Platform>, T>,
    {
        Self::set_writer_func();
        gtk4::init().map_err(|_| Error::GtkInit)?;

        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

        let app = gtk4::Application::default();
        let display = gdk4::Display::default().ok_or(Error::GdkNoDisplay)?;
        let platform = Platform::new(sender.clone(), display, app.clone()).map_err(Error::Io)?;
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

        app.register(None::<&gio::Cancellable>)
            .map_err(|_| Error::GdkApplicationRegister)?;

        app.activate();

        glib::MainContext::default().block_on(async {
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

        Ok(())
    }

    pub fn init_log() {
        let mut filter = tracing_subscriber::EnvFilter::default();

        if cfg!(debug_assertions) {
            filter = filter.add_directive(tracing::Level::DEBUG.into());
        } else {
            filter = filter.add_directive(tracing::Level::WARN.into());
        }

        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer());

        let _ = tracing::subscriber::set_global_default(subscriber);
    }

    fn set_writer_func() {
        static WRITER_SET: AtomicBool = AtomicBool::new(false);

        if WRITER_SET.swap(true, Ordering::SeqCst) {
            return;
        }

        glib::log_set_writer_func(|level, fields| {
            macro_rules! event {
                ($target:expr, $level:expr, $message:expr) => {
                    match $level {
                        glib::LogLevel::Error | glib::LogLevel::Critical => {
                            tracing::error!(target: $target, "{}", $message);
                        }

                        glib::LogLevel::Message | glib::LogLevel::Info => {
                            tracing::info!(target: $target, "{}", $message);
                        }

                        glib::LogLevel::Warning => {
                            tracing::warn!(target: $target, "{}", $message);
                        }

                        glib::LogLevel::Debug => {
                            tracing::debug!(target: $target, "{}", $message);
                        }
                    }
                };
            }

            let mut message = None;
            let mut domain = None;

            for field in fields {
                if field.key() == "MESSAGE" {
                    message = field.value_str()
                }

                if field.key() == "GLIB_DOMAIN" {
                    domain = field.value_str();
                }

                if field.key() == "CODE_FUNC"
                    && (field.value_str() == Some("gtk_widget_measure")
                        || field.value_str() == Some("gtk_widget_allocate"))
                {
                    return glib::LogWriterOutput::Handled;
                }
            }

            let message = message.unwrap_or("<no message>");

            match domain {
                Some("Gdk") => event!("gdk", level, message),
                Some("Gtk") => event!("gtk", level, message),
                Some("GLib") => event!("glib", level, message),
                Some("GObject") => event!("gobject", level, message),
                Some("GLib-GIO") => event!("gio", level, message),
                Some("cairo") => event!("cairo", level, message),
                Some("Gsk") => event!("gsk", level, message),
                _ => event!("unknown", level, message),
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

            Event::Message(mut message) => {
                if let Some(ref mut state) = self.state {
                    let mut action = V::message(
                        (),
                        state,
                        &mut self.context,
                        self.data,
                        &mut message,
                    );

                    if let Some(target) = message.target()
                        && !message.is::<PressableEvent>()
                        && !message.is::<TextInputEvent>()
                        && !message.is_taken()
                    {
                        tracing::warn!(
                            target=?target,
                            type=message.type_name(),
                            "message sent but not received"
                        );
                    }

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
