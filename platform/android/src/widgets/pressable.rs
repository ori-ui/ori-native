use jni::{EnvUnowned, jni_sig, jni_str, objects::JObject};
use ori_native_core::{
    Button, Key, Modifiers, MoveEvent, Point, PressEvent, PressableEvent, native::NativePressable,
};

use crate::{
    Platform,
    application::{GlobalState, WidgetEvent},
    platform::WidgetId,
};

pub struct Pressable {
    id: WidgetId,
}

impl NativePressable<Platform> for Pressable {
    fn build(
        platform: &mut Platform,
        contents: WidgetId,
        on_event: impl Fn(PressableEvent) + 'static,
    ) -> Self {
        let id = platform.next_id();

        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("createPressable"),
                jni_sig!((long)),
                &[id.into()],
            )?
            .v()?;

            env.call_method(
                activity,
                jni_str!("pressableSetContents"),
                jni_sig!((long, long)),
                &[id.into(), contents.into()],
            )?
            .v()
        });

        platform.add_handler(id, move |event| match event {
            WidgetEvent::Pressable(evnet) => on_event(evnet.clone()),
            _ => unreachable!(),
        });

        Self { id }
    }

    fn teardown(self, platform: &mut Platform) {
        platform.remove_widget(self.id);
    }

    fn widget_ref(&self) -> WidgetId {
        self.id
    }

    fn replace_contents(&mut self, platform: &mut Platform, contents: WidgetId) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("pressableSetContents"),
                jni_sig!((long, long)),
                &[self.id.into(), contents.into()],
            )?
            .v()
        });
    }

    fn set_content_size(&mut self, platform: &mut Platform, width: f32, height: f32) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("pressableSetContentSize"),
                jni_sig!((long, float, float)),
                &[self.id.into(), width.into(), height.into()],
            )?
            .v()
        });
    }

    fn set_scrollable(&mut self, _platform: &mut Platform, _scrollable: bool) {}
    fn set_focusable(&mut self, _platform: &mut Platform, _focusable: bool) {}

    fn set_on_key(
        &mut self,
        _platform: &mut Platform,
        _on_key: impl Fn(Key, Modifiers, bool) -> bool + 'static,
    ) {
    }

    fn request_focus(&mut self, _platform: &mut Platform) {}
}

#[unsafe(no_mangle)]
extern "system" fn Java_ori_OriPressable_onPress<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    id: i64,
    state: i32,
    x: f32,
    y: f32,
) -> bool {
    let event = PressEvent {
        button:   Button::Primary,
        position: Point { x, y },
    };

    let event = match state {
        0 => PressableEvent::Pressed(event),
        1 => PressableEvent::Released(event),
        2 => PressableEvent::Cancelled(event),
        _ => return false,
    };

    GlobalState::event(
        WidgetId::new(id as u64),
        WidgetEvent::Pressable(event),
    );

    true
}

#[unsafe(no_mangle)]
extern "system" fn Java_ori_OriPressable_onMove<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    id: i64,
    x: f32,
    y: f32,
) -> bool {
    let event = MoveEvent {
        position: Point { x, y },
    };

    let event = PressableEvent::Moved(event);

    GlobalState::event(
        WidgetId::new(id as u64),
        WidgetEvent::Pressable(event),
    );

    true
}
