use std::{cell::Cell, rc::Rc};

use glib::object::{Cast, ObjectExt};
use gtk4::prelude::{FixedExt, GestureExt, GestureSingleExt, WidgetExt};
use ori_native_core::{
    Button, Key, Modifiers, MoveEvent, Point, PressEvent, PressableEvent, ScrollEvent,
    native::NativePressable,
};

use crate::{Platform, key};

pub struct Pressable {
    fixed:  gtk4::Fixed,
    scroll: Rc<Cell<bool>>,
    key:    Option<gtk4::EventControllerKey>,
}

impl NativePressable<Platform> for Pressable {
    fn build(
        _platform: &mut Platform,
        contents: gtk4::Widget,
        on_event: impl Fn(PressableEvent) + 'static,
    ) -> Self {
        let fixed = gtk4::Fixed::new();
        fixed.put(&contents, 0.0, 0.0);
        fixed.set_focusable(true);
        fixed.set_overflow(gtk4::Overflow::Visible);

        let scroll = Rc::new(Cell::new(false));

        let on_event = Rc::new(on_event);
        let controller = gtk4::GestureClick::new();
        controller.set_button(0);
        controller.connect_pressed({
            let on_event = on_event.clone();

            move |click, _, x, y| {
                click.set_state(gtk4::EventSequenceState::Claimed);

                let event = PressEvent {
                    button:   to_button(click.current_button()),
                    position: Point {
                        x: x as f32,
                        y: y as f32,
                    },
                };

                on_event(PressableEvent::Pressed(event));
            }
        });

        controller.connect_released({
            let on_event = on_event.clone();

            move |click, _, x, y| {
                let event = PressEvent {
                    button:   to_button(click.current_button()),
                    position: Point {
                        x: x as f32,
                        y: y as f32,
                    },
                };

                on_event(PressableEvent::Released(event));
            }
        });

        controller.connect_unpaired_release({
            let on_event = on_event.clone();

            move |_, x, y, button, _| {
                let event = PressEvent {
                    button:   to_button(button),
                    position: Point {
                        x: x as f32,
                        y: y as f32,
                    },
                };

                on_event(PressableEvent::Cancelled(event));
            }
        });

        fixed.add_controller(controller);
        let hovered = Rc::new(Cell::new(false));

        let controller = gtk4::EventControllerMotion::new();
        controller.connect_motion({
            let fixed = fixed.downgrade();
            let on_event = on_event.clone();
            let hovered = hovered.clone();

            move |_, x, y| {
                if let Some(fixed) = fixed.upgrade()
                    && x >= 0.0
                    && y >= 0.0
                    && x <= fixed.width() as f64
                    && y <= fixed.height() as f64
                {
                    if !hovered.get() {
                        on_event(PressableEvent::Hovered(true));
                        hovered.set(true);
                    }
                } else if hovered.get() {
                    on_event(PressableEvent::Hovered(false));
                    hovered.set(false);
                }

                let event = MoveEvent {
                    position: Point {
                        x: x as f32,
                        y: y as f32,
                    },
                };

                on_event(PressableEvent::Moved(event));
            }
        });

        controller.connect_leave({
            let on_event = on_event.clone();
            let hovered = hovered.clone();

            move |_| {
                if hovered.get() {
                    on_event(PressableEvent::Hovered(false));
                    hovered.set(false);
                }
            }
        });

        fixed.add_controller(controller);

        let controller =
            gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::BOTH_AXES);

        controller.connect_scroll({
            let on_event = on_event.clone();
            let scroll = scroll.clone();

            move |_, dx, dy| {
                on_event(PressableEvent::Scrolled(ScrollEvent {
                    dx: dx as f32,
                    dy: dy as f32,
                }));

                if scroll.get() {
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        });

        fixed.add_controller(controller);

        let controller = gtk4::EventControllerFocus::new();
        controller.connect_enter({
            let on_event = on_event.clone();
            move |_| on_event(PressableEvent::Focused(true))
        });

        controller.connect_leave({
            let on_event = on_event.clone();
            move |_| on_event(PressableEvent::Focused(false))
        });

        fixed.add_controller(controller);

        Self {
            fixed,
            scroll,
            key: None,
        }
    }

    fn teardown(self, _platform: &mut Platform) {}

    fn widget_ref(&self) -> gtk4::Widget {
        self.fixed.clone().upcast()
    }

    fn replace_contents(&mut self, _platform: &mut Platform, child: gtk4::Widget) {
        if let Some(child) = self.fixed.first_child() {
            self.fixed.remove(&child);
        }

        self.fixed.put(&child, 0.0, 0.0);
    }

    fn set_content_size(&mut self, _platform: &mut Platform, width: f32, height: f32) {
        if let Some(child) = self.fixed.first_child() {
            child.set_size_request(
                width.round() as i32,
                height.round() as i32,
            );
        }
    }

    fn set_scrollable(&mut self, _platform: &mut Platform, scrollable: bool) {
        self.scroll.set(scrollable);
    }

    fn set_focusable(&mut self, _platform: &mut Platform, focusable: bool) {
        self.fixed.set_focusable(focusable);
    }

    fn set_on_key(
        &mut self,
        _platform: &mut Platform,
        on_key: impl Fn(Key, Modifiers, bool) -> bool + 'static,
    ) {
        if let Some(key) = self.key.take() {
            self.fixed.remove_controller(&key);
        }

        let on_key = Rc::new(on_key);

        let controller = gtk4::EventControllerKey::new();
        controller.connect_key_pressed({
            let on_key = on_key.clone();

            move |_, key, _code, modifiers| {
                let key = key::convert_key(key);
                let modifiers = key::convert_modifiers(modifiers);

                if on_key(key, modifiers, true) {
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        });

        controller.connect_key_released({
            let on_key = on_key.clone();

            move |_, key, _code, modifiers| {
                let key = key::convert_key(key);
                let modifiers = key::convert_modifiers(modifiers);
                on_key(key, modifiers, false);
            }
        });

        self.key = Some(controller.clone());
        self.fixed.add_controller(controller);
    }
}

fn to_button(button: u32) -> Button {
    match button {
        gdk4::BUTTON_PRIMARY => Button::Primary,
        gdk4::BUTTON_SECONDARY => Button::Secondary,
        gdk4::BUTTON_MIDDLE => Button::Tertiary,
        8 => Button::Backward,
        9 => Button::Forward,

        button => Button::Unidentified(button as u16),
    }
}
