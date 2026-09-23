use jni::{EnvUnowned, jni_sig, jni_str, objects::JObject};
use ori_native_core::{Direction, native::NativeScroll};

use crate::{
    Platform,
    application::{GlobalState, WidgetEvent},
    platform::WidgetId,
};

pub struct Scroll {
    id: WidgetId,
}

impl NativeScroll<Platform> for Scroll {
    fn build(
        platform: &mut Platform,
        contents: WidgetId,
        on_scroll: impl Fn(f32, f32) + 'static,
    ) -> Self {
        let id = platform.next_id();

        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("createScroll"),
                jni_sig!((long)),
                &[id.into()],
            )?
            .v()?;

            env.call_method(
                activity,
                jni_str!("scrollSetContents"),
                jni_sig!((long, long)),
                &[id.into(), contents.into()],
            )?
            .v()
        });

        platform.add_handler(id, move |event| match event {
            WidgetEvent::Scroll(x, y) => on_scroll(*x, *y),
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
                jni_str!("scrollSetContents"),
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
                jni_str!("scrollSetContentSize"),
                jni_sig!((long, float, float)),
                &[self.id.into(), width.into(), height.into()],
            )?
            .v()
        });
    }

    fn set_content_layout(
        &mut self,
        platform: &mut Platform,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("scrollSetContentLayout"),
                jni_sig!((long, float, float, float, float)),
                &[
                    self.id.into(),
                    x.into(),
                    y.into(),
                    width.into(),
                    height.into(),
                ],
            )?
            .v()
        });
    }

    fn set_direction(&mut self, platform: &mut Platform, direction: Direction) {
        let is_vertical = matches!(direction, Direction::Vertical);

        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("scrollSetVertical"),
                jni_sig!((long, boolean)),
                &[self.id.into(), is_vertical.into()],
            )?
            .v()
        });
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_ori_OriActivity_onScrolled<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    id: i64,
    x: f32,
    y: f32,
) {
    GlobalState::event(
        WidgetId::new(id as u64),
        WidgetEvent::Scroll(x, y),
    );
}
