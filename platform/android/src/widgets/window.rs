use std::time::Duration;

use jni::{EnvUnowned, jni_sig, jni_str, objects::JObject};
use ori_native_core::{Key, Modifiers, NavigationBar, Sides, StatusBar, native::NativeWindow};

use crate::{
    Platform,
    application::{Event, GLOBAL_STATE},
    platform::WidgetId,
};

pub struct Window {}

impl NativeWindow<Platform> for Window {
    fn build(platform: &mut Platform, contents: WidgetId) -> Self {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("windowSetContents"),
                jni_sig!((long)),
                &[contents.into()],
            )?
            .v()
        });

        Self {}
    }

    fn teardown(self, _platform: &mut Platform) {}

    fn replace_contents(&mut self, platform: &mut Platform, contents: WidgetId) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("windowSetContents"),
                jni_sig!((long)),
                &[contents.into()],
            )?
            .v()
        });
    }

    fn get_size(&self, platform: &mut Platform) -> (f32, f32) {
        let width = platform
            .jni(|env, activity| {
                env.call_method(
                    activity,
                    jni_str!("windowGetWidth"),
                    jni_sig!(() -> float),
                    &[],
                )?
                .f()
            })
            .unwrap_or(0.0);

        let height = platform
            .jni(|env, activity| {
                env.call_method(
                    activity,
                    jni_str!("windowGetHeight"),
                    jni_sig!(() -> float),
                    &[],
                )?
                .f()
            })
            .unwrap_or(0.0);

        (width, height)
    }

    fn get_scale(&self, platform: &mut Platform) -> f32 {
        platform
            .jni(|env, activity| {
                env.call_method(
                    activity,
                    jni_str!("windowGetScale"),
                    jni_sig!(() -> float),
                    &[],
                )?
                .f()
            })
            .unwrap_or(1.0)
    }

    fn get_preferred_size(&self, platform: &mut Platform) -> (Option<f32>, Option<f32>) {
        let (width, height) = self.get_size(platform);

        (Some(width), Some(height))
    }

    fn is_decorated(&self, _platform: &mut Platform) -> bool {
        true
    }

    fn set_on_animation_frame(
        &mut self,
        platform: &mut Platform,
        on_frame: impl Fn(Duration) + 'static,
    ) {
        platform.set_on_animation_frame(on_frame);
    }

    fn set_on_resize(&mut self, _platform: &mut Platform, _on_resize: impl Fn() + 'static) {}

    fn set_on_close_requested(
        &mut self,
        _platform: &mut Platform,
        _on_close_requested: impl Fn() + 'static,
    ) {
    }

    fn set_on_key(
        &mut self,
        _platform: &mut Platform,
        _on_key: impl Fn(Key, Modifiers, bool) -> bool + 'static,
    ) {
    }

    fn start_animating(&mut self, platform: &mut Platform) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("windowStartAnimating"),
                jni_sig!(()),
                &[],
            )?
            .v()
        });
    }

    fn stop_animating(&mut self, platform: &mut Platform) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("windowStopAnimating"),
                jni_sig!(()),
                &[],
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
                jni_str!("windowSetContentLayout"),
                jni_sig!((float, float, float, float)),
                &[x.into(), y.into(), width.into(), height.into()],
            )?
            .v()
        });
    }

    fn set_title(&mut self, _platform: &mut Platform, _title: String) {}
    fn set_min_size(&mut self, _platform: &mut Platform, _width: f32, _height: f32) {}
    fn set_size(&mut self, _platform: &mut Platform, _width: f32, _height: f32) {}
    fn set_decorated(&mut self, _platform: &mut Platform, _decorated: bool) {}
    fn set_resizable(&mut self, _platform: &mut Platform, _resizable: bool) {}

    fn set_status_bar(&mut self, platform: &mut Platform, bar: StatusBar) {
        let _ = platform.jni(|env, activity| {
            let set_color = bar.color.is_some();
            let color = bar.color.unwrap_or_default();

            env.call_method(
                activity,
                jni_str!("windowSetStatusBar"),
                jni_sig!((
                    boolean, boolean, float, float, float, float
                )),
                &[
                    bar.light.into(),
                    set_color.into(),
                    color.r.into(),
                    color.g.into(),
                    color.b.into(),
                    color.a.into(),
                ],
            )?
            .v()
        });
    }

    fn set_navigation_bar(&mut self, platform: &mut Platform, bar: NavigationBar) {
        let _ = platform.jni(|env, activity| {
            let set_color = bar.color.is_some();
            let color = bar.color.unwrap_or_default();

            env.call_method(
                activity,
                jni_str!("windowSetNavigationBar"),
                jni_sig!((
                    boolean, boolean, float, float, float, float
                )),
                &[
                    bar.light.into(),
                    set_color.into(),
                    color.r.into(),
                    color.g.into(),
                    color.b.into(),
                    color.a.into(),
                ],
            )?
            .v()
        });
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_ori_OriActivity_onAnimationFrame<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    duration_nanos: i64,
) {
    if let Some(activity) = GLOBAL_STATE.get() {
        let duration = Duration::from_nanos(duration_nanos as u64);
        let _ = activity.sender.send(Event::Frame(duration));
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_ori_OriActivity_onInsetsChanged<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
) {
    if let Some(activity) = GLOBAL_STATE.get() {
        let insets = Sides {
            top,
            right,
            bottom,
            left,
        };

        let _ = activity.sender.send(Event::Insets(insets));
    }
}
