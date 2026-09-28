use std::{
    error, fmt, io,
    sync::{
        Arc, Mutex, OnceLock,
        mpsc::{Receiver, Sender},
    },
    time::Duration,
};

use jni::{objects::JObject, refs::Global, vm::JavaVM};
use ori::{Action, Effect, Message, Provider, Proxied};
use ori_native_core::{Context, PressableEvent, SafeAreaInsets, Sides, TextInputEvent};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt};

use crate::{Platform, log::MakeAndroidWriter, platform::WidgetId};

#[derive(Debug)]
pub enum Error {
    Uninitialized,
    MultipleApplications,
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Uninitialized => f.write_str("the ori runtime has not been initialized"),

            Error::MultipleApplications => f.write_str(
                "running multiple applications at the same time is not supported on android",
            ),

            Error::Io(err) => err.fmt(f),
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

    pub fn run<T, V>(self, data: &mut T, mut build: impl FnMut(&T) -> V) -> Result<(), Error>
    where
        V: Effect<Context<Platform>, T>,
    {
        let state = GLOBAL_STATE.get().ok_or(Error::Uninitialized)?;

        let mut receiver = state
            .receiver
            .try_lock()
            .map_err(|_| Error::MultipleApplications)?;

        let platform = Platform::new(state).map_err(Error::Io)?;
        let mut context = Context::new(platform);

        let view = build(data);

        let (_, state) = view.build(&mut context, data);

        let mut state = State {
            data,
            build,
            state: Some(state),
            context,
            running: true,
            receiver: &mut receiver,
        };

        while state.running
            && let Ok(event) = state.receiver.recv()
        {
            state.handle_event(event);

            // we want to handle every task in the queue before updating the ui.
            while let Ok(event) = state.receiver.try_recv() {
                state.handle_event(event);
            }

            // after all events have been handled, and we expect to be idle,
            // execute batched ui updates.
            state.context.platform.run_ui_tasks();
        }

        state.teardown();

        Ok(())
    }

    pub fn init_log() {
        let mut filter = EnvFilter::default();

        if cfg!(debug_assertions) {
            filter = filter.add_directive(tracing::Level::DEBUG.into());
        } else {
            filter = filter.add_directive(tracing::Level::WARN.into());
        }

        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().with_writer(MakeAndroidWriter));

        let _ = tracing::subscriber::set_global_default(subscriber);
    }
}

pub static GLOBAL_STATE: OnceLock<GlobalState> = OnceLock::new();

pub struct GlobalState {
    pub sender:   Sender<Event>,
    pub receiver: Mutex<Receiver<Event>>,
    pub jvm:      JavaVM,
    pub activity: Mutex<Arc<Global<JObject<'static>>>>,
}

impl GlobalState {
    pub fn event(widget: WidgetId, event: WidgetEvent) {
        if let Some(this) = GLOBAL_STATE.get() {
            let _ = this.sender.send(Event::Widget(widget, event));
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Recreate,
    Rebuild,
    Message(Message),
    Frame(Duration),
    Widget(WidgetId, WidgetEvent),
    Insets(Sides<f32>),
}

#[derive(Debug)]
pub enum WidgetEvent {
    Pressable(PressableEvent),
    TextInput(TextInputEvent),
    Scroll(f32, f32),
    Position(f32, f32),
}

struct State<'a, T, V, B>
where
    V: Effect<Context<Platform>, T>,
{
    data:     &'a mut T,
    build:    B,
    state:    Option<V::State>,
    context:  Context<Platform>,
    running:  bool,
    receiver: &'a mut Receiver<Event>,
}

impl<T, V, B> State<'_, T, V, B>
where
    V: Effect<Context<Platform>, T>,
    B: FnMut(&T) -> V,
{
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Recreate => {
                if let Some(state) = self.state.take() {
                    V::teardown((), state, &mut self.context);
                    let view = (self.build)(self.data);

                    if let Some(state) = GLOBAL_STATE.get() {
                        self.context.platform.recreate(state);
                    }

                    let ((), state) = view.build(&mut self.context, self.data);
                    self.state = Some(state);
                }
            }

            Event::Rebuild => {
                if let Some(ref mut state) = self.state {
                    let view = (self.build)(self.data);

                    view.rebuild((), state, &mut self.context, self.data);
                }
            }

            Event::Message(mut message) => {
                let t = std::time::Instant::now();

                if let Some(ref mut state) = self.state {
                    let action = V::message(
                        (),
                        state,
                        &mut self.context,
                        self.data,
                        &mut message,
                    );

                    self.context.send_action(action);
                }

                tracing::trace!(
                    type = message.type_name(),
                    time = ?t.elapsed(),
                    "message",
                );
            }

            Event::Frame(duration) => {
                self.context.platform.on_animation_frame(duration);
            }

            Event::Widget(id, event) => {
                self.context.platform.handle_event(id, event);
            }

            Event::Insets(insets) => {
                let insets = SafeAreaInsets(insets);

                match self.context.get_mut() {
                    Some(current) => {
                        if *current != insets {
                            *current = insets;
                            self.context.send_action(Action::rebuild());
                        }
                    }

                    None => {
                        self.context.push(Box::new(insets));
                        self.context.send_action(Action::rebuild());
                    }
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
