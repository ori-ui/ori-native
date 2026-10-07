use std::time::Duration;

use glib::{
    object::{Cast, IsA},
    subclass::types::ObjectSubclassIsExt,
};
use gtk4::prelude::{FixedExt, PopoverExt, WidgetExt};
use ori_native_core::{PopupPosition, Side, native::NativePopup};

use crate::Platform;

impl NativePopup<Platform> for Popup {
    fn build(
        _platform: &mut Platform,
        anchor: gtk4::Widget,
        on_dismiss: impl Fn() + 'static,
    ) -> Self {
        let popup = Self::new();
        popup.set_anchor(Some(&anchor));
        popup.imp().popover.connect_closed(move |_| on_dismiss());
        popup
    }

    fn teardown(self, _platform: &mut Platform) {}

    fn widget_ref(&self) -> gtk4::Widget {
        self.clone().upcast()
    }

    fn replace_anchor(&mut self, _platform: &mut Platform, anchor: gtk4::Widget) {
        self.set_anchor(Some(&anchor));
    }

    fn get_scale(&mut self, _platform: &mut Platform) -> f32 {
        1.0
    }

    fn open(&mut self, _platform: &mut Platform, contents: gtk4::Widget) {
        self.set_contents(Some(&contents));

        if let Some(anchor) = self.imp().anchor.borrow().clone() {
            if anchor.is_realized() {
                self.imp().popover.popup();
            } else {
                let popover = self.imp().popover.clone();
                let delay = Duration::from_millis(200);

                glib::timeout_add_local_once(delay, move || {
                    if anchor.is_realized() {
                        popover.popup();
                    } else {
                        tracing::warn!("popup couldn't be opened");
                    }
                });
            }
        }
    }

    fn close(&mut self, _platform: &mut Platform) {
        self.set_contents(None::<&gtk4::Widget>);
        self.imp().popover.popdown();
    }

    fn set_position(&mut self, _platform: &mut Platform, position: PopupPosition) {
        match position {
            PopupPosition::Absolute(position) => {
                let width = self.imp().popover.width_request();
                let x = position.x.round() as i32 - 2;
                let y = position.y.round() as i32;

                let rect = gdk4::Rectangle::new(x, y, 0, 0);

                self.imp().popover.set_position(gtk4::PositionType::Bottom);
                self.imp().popover.set_pointing_to(Some(&rect));
                self.imp().popover.set_offset(width / 2, 0);
                self.imp().position.set(Some(position));
            }

            PopupPosition::Relative(side) => {
                let position = match side {
                    Side::Top => gtk4::PositionType::Top,
                    Side::Right => gtk4::PositionType::Right,
                    Side::Bottom => gtk4::PositionType::Bottom,
                    Side::Left => gtk4::PositionType::Left,
                };

                self.imp().popover.set_position(position);
                self.imp().popover.set_pointing_to(None);
                self.imp().position.set(None);
            }
        }
    }

    fn set_modal(&mut self, _platform: &mut Platform, is_modal: bool) {
        self.imp().popover.set_autohide(is_modal);
    }

    fn set_anchor_size(&mut self, _platform: &mut Platform, _width: f32, _height: f32) {}

    fn set_popup_size(&mut self, _platform: &mut Platform, width: f32, height: f32) {
        self.imp().popover.set_size_request(
            width.round() as i32,
            height.round() as i32,
        );

        if let Some(position) = self.imp().position.get() {
            let width = self.imp().popover.width_request();
            let x = position.x.round() as i32 - 2;
            let y = position.y.round() as i32;

            let rect = gdk4::Rectangle::new(x, y, 0, 0);
            self.imp().popover.set_pointing_to(Some(&rect));
            self.imp().popover.set_offset(width / 2, 0);
        }
    }

    fn set_content_layout(
        &mut self,
        _platform: &mut Platform,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        if let Some(child) = self.imp().fixed.first_child() {
            self.imp().fixed.move_(&child, x as f64, y as f64);
            child.set_size_request(
                width.round() as i32,
                height.round() as i32,
            );
        }
    }
}

impl Popup {
    fn new() -> Self {
        let this: Self = glib::Object::new();
        this.imp().popover.set_parent(&this);
        this
    }

    fn set_anchor(&self, anchor: Option<&impl IsA<gtk4::Widget>>) {
        if let Some(ref child) = *self.imp().anchor.borrow() {
            child.unparent();
        }

        if let Some(child) = anchor {
            child.set_parent(self);
            self.imp().anchor.replace(Some(child.as_ref().clone()));
        }
    }

    fn set_contents(&self, contents: Option<&impl IsA<gtk4::Widget>>) {
        if let Some(child) = self.imp().fixed.first_child() {
            self.imp().fixed.remove(&child);
        }

        if let Some(popover) = contents {
            self.imp().fixed.put(popover, 0.0, 0.0);
        }
    }
}

glib::wrapper! {
    pub struct Popup(
        ObjectSubclass<imp::Popup>)
        @extends
            gtk4::Widget,
        @implements
            gtk4::Buildable,
            gtk4::ConstraintTarget;
}

mod imp {
    use std::cell::{Cell, RefCell};

    use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
    use gtk4::{
        prelude::{PopoverExt, WidgetExt},
        subclass::widget::WidgetImpl,
    };
    use ori_native_core::Point;

    pub struct Popup {
        pub(super) anchor:   RefCell<Option<gtk4::Widget>>,
        pub(super) position: Cell<Option<Point<f32>>>,
        pub(super) popover:  gtk4::Popover,
        pub(super) fixed:    gtk4::Fixed,
    }

    impl Default for Popup {
        fn default() -> Self {
            let fixed = gtk4::Fixed::new();
            let popover = gtk4::Popover::new();
            popover.set_has_arrow(false);
            popover.set_child(Some(&fixed));

            // this helps with the first popup, since wayland
            // doesn't like surfaces with zero area
            popover.set_size_request(1, 1);

            Self {
                anchor: Default::default(),
                position: Cell::new(None),
                popover,
                fixed,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Popup {
        const NAME: &'static str = "OriPopup";
        type Type = super::Popup;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for Popup {
        fn dispose(&self) {
            if let Some(ref child) = *self.anchor.borrow() {
                child.unparent();
            }

            self.popover.unparent();
        }
    }

    impl WidgetImpl for Popup {
        fn measure(&self, orientation: gtk4::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            match *self.anchor.borrow() {
                Some(ref child) => child.measure(orientation, for_size),
                None => (0, 0, -1, -1),
            }
        }

        fn size_allocate(&self, width: i32, height: i32, _baseline: i32) {
            if let Some(ref child) = *self.anchor.borrow() {
                let allocation = gtk4::Allocation::new(0, 0, width, height);
                child.size_allocate(&allocation, -1);
            }

            self.popover.present();
        }
    }
}
