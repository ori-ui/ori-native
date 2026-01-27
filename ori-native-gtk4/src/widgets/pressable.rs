use std::sync::Arc;

use gtk4::prelude::WidgetExt;
use ori_native_core::native::{HasPressable, NativePressable};

use crate::{Platform, widgets::group::GroupWidget};

pub struct Pressable {
    widget:  GroupWidget,
    gesture: Option<gtk4::GestureClick>,
    focus:   Option<gtk4::EventControllerFocus>,
}

impl NativePressable<Platform> for Pressable {
    fn widget(&self) -> &gtk4::Widget {
        self.widget.as_ref()
    }

    fn build(_plaform: &mut Platform, contents: &gtk4::Widget) -> Self {
        let widget = GroupWidget::new();
        widget.insert_child(0, contents);
        widget.set_focusable(true);

        Self {
            widget,
            gesture: None,
            focus: None,
        }
    }

    fn teardown(self, _plaform: &mut Platform) {}

    fn set_size(&mut self, width: f32, height: f32) {
        self.widget.set_size(width as i32, height as i32);
        (self.widget).set_child_layout(0, 0, 0, width as i32, height as i32);
    }

    fn set_on_click(&mut self, on_click: impl Fn(bool) + 'static) {
        if let Some(controller) = self.gesture.take() {
            self.widget.remove_controller(&controller);
        }

        let on_click = Arc::new(on_click);

        let controller = gtk4::GestureClick::new();
        controller.connect_pressed({
            let on_click = on_click.clone();
            move |_, _, _, _| on_click(true)
        });

        controller.connect_released({
            let on_click = on_click.clone();
            move |_, _, _, _| on_click(false)
        });

        self.gesture = Some(controller.clone());
        self.widget.add_controller(controller);
    }

    fn set_on_focus(&mut self, on_focus: impl Fn(bool) + 'static) {
        if let Some(controller) = self.focus.take() {
            self.widget.remove_controller(&controller);
        }

        let on_focus = Arc::new(on_focus);

        let controller = gtk4::EventControllerFocus::new();
        controller.connect_enter({
            let on_focus = on_focus.clone();
            move |_| on_focus(true)
        });

        controller.connect_leave({
            let on_focus = on_focus.clone();
            move |_| on_focus(false)
        });

        self.focus = Some(controller.clone());
        self.widget.add_controller(controller);
    }
}

impl HasPressable for Platform {
    type Pressable = Pressable;
}
