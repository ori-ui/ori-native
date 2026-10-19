use jni::{jni_sig, jni_str};
use ori_native_core::{Color, Corners, OutlineStyle, Overflow, Shadow, Sides, native::NativeGroup};

use crate::{Platform, platform::WidgetId};

pub struct Group {
    id: WidgetId,
}

impl NativeGroup<Platform> for Group {
    fn build(platform: &mut Platform) -> Self {
        let id = platform.next_id();

        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("createGroup"),
                jni_sig!((long)),
                &[id.into()],
            )?
            .v()
        });

        Self { id }
    }

    fn teardown(self, platform: &mut Platform) {
        platform.remove_widget(self.id);
    }

    fn widget_ref(&self) -> WidgetId {
        self.id
    }

    fn insert_child(&mut self, platform: &mut Platform, index: usize, child: WidgetId) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("groupInsert"),
                jni_sig!((long, int, long)),
                &[self.id.into(), (index as i32).into(), child.into()],
            )?
            .v()
        });
    }

    fn remove_child(&mut self, platform: &mut Platform, index: usize) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("groupRemove"),
                jni_sig!((long, int)),
                &[self.id.into(), (index as i32).into()],
            )?
            .v()
        });
    }

    fn replace_child(&mut self, platform: &mut Platform, index: usize, child: WidgetId) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("groupRemove"),
                jni_sig!((long, int)),
                &[self.id.into(), (index as i32).into()],
            )?
            .v()?;

            env.call_method(
                activity,
                jni_str!("groupInsert"),
                jni_sig!((long, int, long)),
                &[self.id.into(), (index as i32).into(), child.into()],
            )?
            .v()
        });
    }

    fn swap_children(&mut self, platform: &mut Platform, index_a: usize, index_b: usize) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("groupSwap"),
                jni_sig!((long, int, int)),
                &[
                    self.id.into(),
                    (index_a as i32).into(),
                    (index_b as i32).into(),
                ],
            )?
            .v()
        });
    }

    fn set_child_layout(
        &mut self,
        platform: &mut Platform,
        index: usize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("groupSetChildLayout"),
                jni_sig!((long, int, float, float, float, float)),
                &[
                    self.id.into(),
                    (index as i32).into(),
                    x.into(),
                    y.into(),
                    width.into(),
                    height.into(),
                ],
            )?
            .v()
        });
    }

    fn set_background_color(&mut self, platform: &mut Platform, color: Color) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("groupSetBackgroundColor"),
                jni_sig!((long, float, float, float, float)),
                &[
                    self.id.into(),
                    color.r.into(),
                    color.g.into(),
                    color.b.into(),
                    color.a.into(),
                ],
            )?
            .v()
        });
    }

    fn set_border_color(&mut self, platform: &mut Platform, color: Color) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("groupSetBorderColor"),
                jni_sig!((long, float, float, float, float)),
                &[
                    self.id.into(),
                    color.r.into(),
                    color.g.into(),
                    color.b.into(),
                    color.a.into(),
                ],
            )?
            .v()
        });
    }

    fn set_border_width(&mut self, platform: &mut Platform, width: Sides<f32>) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("groupSetBorderWidth"),
                jni_sig!((long, float, float, float, float)),
                &[
                    self.id.into(),
                    width.top.into(),
                    width.right.into(),
                    width.bottom.into(),
                    width.left.into(),
                ],
            )?
            .v()
        });
    }

    fn set_outline(&mut self, platform: &mut Platform, outline: OutlineStyle) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("groupSetOutline"),
                jni_sig!((
                    long,
                    float,
                    float,
                    float,
                    float,
                    float,
                    float,
                    float,
                    float,
                    float,
                    float,
                    float,
                    float,
                ) -> void),
                &[
                    self.id.into(),
                    outline.color.r.into(),
                    outline.color.g.into(),
                    outline.color.b.into(),
                    outline.color.a.into(),
                    outline.width.top.into(),
                    outline.width.right.into(),
                    outline.width.bottom.into(),
                    outline.width.left.into(),
                    outline.offset.top.into(),
                    outline.offset.right.into(),
                    outline.offset.bottom.into(),
                    outline.offset.left.into(),
                ],
            )?
            .v()
        });
    }

    fn set_corners(&mut self, platform: &mut Platform, corners: Corners<f32>) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("groupSetCornerRadii"),
                jni_sig!((long, float, float, float, float)),
                &[
                    self.id.into(),
                    corners.top_left.into(),
                    corners.top_right.into(),
                    corners.bottom_right.into(),
                    corners.bottom_left.into(),
                ],
            )?
            .v()
        });
    }

    fn set_overflow(&mut self, platform: &mut Platform, overflow: Overflow) {
        let visible = matches!(overflow, Overflow::Visible);

        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("groupSetOverflow"),
                jni_sig!((long, boolean)),
                &[self.id.into(), visible.into()],
            )?
            .v()
        });
    }

    fn set_shadow(&mut self, platform: &mut Platform, shadow: Shadow) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("groupSetShadow"),
                jni_sig!((
                    long, float, float, float, float, float, float, float, float
                )),
                &[
                    self.id.into(),
                    shadow.color.r.into(),
                    shadow.color.g.into(),
                    shadow.color.b.into(),
                    shadow.color.a.into(),
                    shadow.offset_x.into(),
                    shadow.offset_y.into(),
                    shadow.radius.into(),
                    shadow.spread.into(),
                ],
            )?
            .v()
        });
    }

    fn set_hardware_layer(&mut self, platform: &mut Platform, enabled: bool) {
        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("viewSetHardwareLayer"),
                jni_sig!((long, boolean)),
                &[self.id.into(), enabled.into()],
            )?
            .v()
        });
    }
}
