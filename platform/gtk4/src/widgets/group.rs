use glib::{object::Cast, subclass::types::ObjectSubclassIsExt};
use gtk4::prelude::WidgetExt;
use ori_native_core::{Color, Corners, Overflow, Shadow, Sides, native::NativeGroup};

use crate::Platform;

impl NativeGroup<Platform> for Group {
    fn build(_platform: &mut Platform) -> Self {
        Self::new()
    }

    fn teardown(self, _platform: &mut Platform) {}

    fn widget_ref(&self) -> gtk4::Widget {
        self.clone().upcast()
    }

    fn insert_child(&mut self, _platform: &mut Platform, index: usize, child: gtk4::Widget) {
        let mut children = self.imp().children.borrow_mut();

        if let Some(current) = children.get(index) {
            child.insert_before(self, Some(&current.widget));
        } else {
            child.set_parent(self);
        }

        children.insert(
            index,
            imp::Child {
                widget: child.clone(),
                x:      0,
                y:      0,
            },
        );
    }

    fn remove_child(&mut self, _platform: &mut Platform, index: usize) {
        Group::remove(self, index);
    }

    fn replace_child(&mut self, _platform: &mut Platform, index: usize, child: gtk4::Widget) {
        let mut children = self.imp().children.borrow_mut();

        if let Some(current) = children.get_mut(index) {
            child.insert_after(self, Some(&current.widget));
            current.widget.unparent();
            current.widget = child.clone();
        }
    }

    fn swap_children(&mut self, _platform: &mut Platform, index_a: usize, index_b: usize) {
        let mut children = self.imp().children.borrow_mut();

        let first = usize::min(index_a, index_b);
        let last = usize::max(index_a, index_b);

        // get the child after the last one
        let after = children.get(last + 1).map(|child| &child.widget);

        // move the last child after the first
        children[last].widget.insert_after(
            self,
            children.get(first).map(|child| &child.widget),
        );

        // move the first child, before the child after the last
        children[first].widget.insert_before(self, after);

        // swap in the array
        children.swap(index_a, index_b);
    }

    fn set_child_layout(
        &mut self,
        _platform: &mut Platform,
        index: usize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.layout(index, x, y, width, height);
    }

    fn set_background_color(&mut self, _platform: &mut Platform, color: Color) {
        let color = gdk4::RGBA::new(color.r, color.g, color.b, color.a);

        self.imp().background_color.set(color);
        self.queue_draw();
    }

    fn set_border_color(&mut self, _platform: &mut Platform, color: Color) {
        let color = gdk4::RGBA::new(color.r, color.g, color.b, color.a);

        self.imp().border_color.set(color);
        self.queue_draw();
    }

    fn set_border_width(&mut self, _platform: &mut Platform, width: Sides<f32>) {
        self.imp().border_width.set(width.into());
        self.queue_draw();
    }

    fn set_corners(&mut self, _platform: &mut Platform, corners: Corners<f32>) {
        self.imp().corner_radii.set(corners.into());
        self.queue_draw();
    }

    fn set_overflow(&mut self, _platform: &mut Platform, overflow: Overflow) {
        self.imp().overflow.set(overflow);
        self.queue_draw();
    }

    fn set_shadow(&mut self, _platform: &mut Platform, shadow: Shadow) {
        self.imp().shadow.set(shadow);
        self.queue_draw();
    }
}

glib::wrapper! {
    pub struct Group(
        ObjectSubclass<imp::Group>)
        @extends
            gtk4::Widget,
        @implements
            gtk4::Buildable,
            gtk4::ConstraintTarget;
}

impl Group {
    pub fn new() -> Self {
        gtk4::glib::Object::builder().build()
    }

    pub fn get(&self, index: usize) -> Option<gtk4::Widget> {
        self.imp()
            .children
            .borrow()
            .get(index)
            .map(|child| child.widget.clone())
    }

    pub fn add(&self, child: gtk4::Widget) {
        child.set_parent(self);

        self.imp().children.borrow_mut().push(imp::Child {
            widget: child,
            x:      0,
            y:      0,
        });
    }

    pub fn remove(&self, index: usize) {
        let child = self.imp().children.borrow_mut().remove(index);
        child.widget.unparent();
    }

    pub fn layout(&self, index: usize, x: f32, y: f32, width: f32, height: f32) {
        let x = x.round() as i32;
        let y = y.round() as i32;
        let width = width.round() as i32;
        let height = height.round() as i32;

        if let Some(child) = self.imp().children.borrow_mut().get_mut(index) {
            child.x = x;
            child.y = y;
            child.widget.set_size_request(width, height);
            child.widget.queue_allocate();
            child.widget.queue_resize();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.imp().children.borrow().is_empty()
    }

    pub fn len(&self) -> usize {
        self.imp().children.borrow().len()
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use glib::subclass::{
        object::ObjectImpl,
        types::{ObjectSubclass, ObjectSubclassExt},
    };
    use gtk4::{
        prelude::{SnapshotExt, SnapshotExtManual, WidgetExt},
        subclass::widget::{WidgetImpl, WidgetImplExt},
    };
    use ori_native_core::{Overflow, Shadow};

    pub struct Group {
        pub(super) children: RefCell<Vec<Child>>,

        pub(super) background_color: Cell<gdk4::RGBA>,
        pub(super) border_color:     Cell<gdk4::RGBA>,
        pub(super) corner_radii:     Cell<[f32; 4]>,
        pub(super) border_width:     Cell<[f32; 4]>,
        pub(super) overflow:         Cell<Overflow>,
        pub(super) shadow:           Cell<Shadow>,
    }

    pub(super) struct Child {
        pub(super) widget: gtk4::Widget,
        pub(super) x:      i32,
        pub(super) y:      i32,
    }

    impl Default for Group {
        fn default() -> Self {
            Self {
                children: RefCell::default(),

                background_color: Cell::new(gdk4::RGBA::TRANSPARENT),
                border_color:     Cell::new(gdk4::RGBA::TRANSPARENT),
                corner_radii:     Cell::new([0.0; 4]),
                border_width:     Cell::new([0.0; 4]),
                overflow:         Cell::new(Overflow::Visible),
                shadow:           Cell::new(Shadow::default()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Group {
        const NAME: &'static str = "OriGroup";
        type Type = super::Group;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for Group {
        fn dispose(&self) {
            for child in self.children.borrow().iter() {
                child.widget.unparent();
            }
        }
    }

    impl WidgetImpl for Group {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let [tl, tr, br, bl] = self.corner_radii.get();

            let rect = gsk4::RoundedRect::new(
                graphene::Rect::new(
                    0.0,
                    0.0,
                    self.obj().width() as f32,
                    self.obj().height() as f32,
                ),
                graphene::Size::new(tl, tl),
                graphene::Size::new(tr, tr),
                graphene::Size::new(br, br),
                graphene::Size::new(bl, bl),
            );

            let shadow = self.shadow.get();

            if shadow.color.a > 0.0 {
                snapshot.append_outset_shadow(
                    &rect,
                    &gdk4::RGBA::new(
                        shadow.color.r,
                        shadow.color.g,
                        shadow.color.b,
                        shadow.color.a,
                    ),
                    shadow.offset_x,
                    shadow.offset_y,
                    shadow.spread,
                    shadow.radius,
                );
            }

            snapshot.push_rounded_clip(&rect);

            snapshot.append_color(
                &self.background_color.get(),
                rect.bounds(),
            );

            snapshot.append_border(
                &rect,
                &self.border_width.get(),
                &[self.border_color.get(); 4],
            );

            if let Overflow::Visible = self.overflow.get() {
                snapshot.pop();
            }

            self.parent_snapshot(snapshot);

            if let Overflow::Hidden = self.overflow.get() {
                snapshot.pop();
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);

            for child in self.children.borrow().iter() {
                child.widget.size_allocate(
                    &gtk4::Allocation::new(
                        child.x,
                        child.y,
                        child.widget.width_request(),
                        child.widget.height_request(),
                    ),
                    -1,
                );
            }
        }

        fn measure(&self, orientation: gtk4::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            for child in self.children.borrow().iter() {
                child.widget.measure(orientation, for_size);
            }

            match orientation {
                gtk4::Orientation::Horizontal => {
                    let width = self.obj().width_request();
                    (width, width, -1, -1)
                }

                gtk4::Orientation::Vertical => {
                    let height = self.obj().height_request();
                    (height, height, -1, -1)
                }

                _ => (-1, -1, -1, -1),
            }
        }
    }
}
