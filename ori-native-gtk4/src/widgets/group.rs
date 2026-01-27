use gdk4::glib::subclass::types::ObjectSubclassIsExt;
use gtk4::prelude::WidgetExt;
use ori_native_core::{
    Color,
    native::{HasGroup, NativeGroup},
};

use crate::Platform;

pub struct Group {
    group:    GroupWidget,
    children: Vec<gtk4::Widget>,
}

impl NativeGroup<Platform> for Group {
    fn widget(&self) -> &gtk4::Widget {
        self.group.as_ref()
    }

    fn build(_platform: &mut Platform) -> Self {
        let group = GroupWidget::new();

        Self {
            group,
            children: Vec::new(),
        }
    }

    fn teardown(self, _platform: &mut Platform) {}

    fn insert_child(&mut self, index: usize, child: &gtk4::Widget) {
        self.children.insert(index, child.clone());
        self.group.insert_child(index, child);
    }

    fn remove_child(&mut self, index: usize) {
        self.children.remove(index);
        self.group.remove_child(index);
    }

    fn swap_children(&mut self, index_a: usize, index_b: usize) {
        self.children.swap(index_a, index_b);
        self.group.swap_children(index_a, index_b);
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.group.set_size(width as i32, height as i32);
    }

    fn set_child_layout(&mut self, index: usize, x: f32, y: f32, width: f32, height: f32) {
        self.group.set_child_layout(
            index,
            x as i32,
            y as i32,
            width as i32,
            height as i32,
        );
    }

    fn set_background_color(&mut self, _platform: &mut Platform, color: Color) {
        self.group.set_background_color(gdk4::RGBA::new(
            color.r, color.g, color.b, color.a,
        ));
    }

    fn set_border_color(&mut self, _platform: &mut Platform, color: Color) {
        self.group.set_border_color(gdk4::RGBA::new(
            color.r, color.g, color.b, color.a,
        ));
    }

    fn set_border_width(&mut self, _platform: &mut Platform, width: [f32; 4]) {
        self.group.set_border_width(width);
    }

    fn set_corner_radii(&mut self, _platform: &mut Platform, radii: [f32; 4]) {
        self.group.set_corner_radii(radii);
    }
}

impl HasGroup for Platform {
    type Group = Group;
}

gtk4::glib::wrapper! {
    pub struct GroupWidget(
        ObjectSubclass<imp::GroupWidget>)
        @extends
            gtk4::Widget,
        @implements
            gtk4::Buildable,
            gtk4::Accessible,
            gtk4::ConstraintTarget;
}

impl GroupWidget {
    pub fn new() -> Self {
        gtk4::glib::Object::builder().build()
    }

    pub fn set_size(&self, width: i32, height: i32) {
        self.imp().width.set(width);
        self.imp().height.set(height);
        self.queue_resize();
    }

    pub fn set_background_color(&self, background_color: gdk4::RGBA) {
        self.imp().background_color.set(background_color);
        self.queue_draw();
    }

    pub fn set_border_color(&self, border_color: gdk4::RGBA) {
        self.imp().border_color.set(border_color);
        self.queue_draw();
    }

    pub fn set_corner_radii(&self, corner_radii: [f32; 4]) {
        self.imp().corner_radii.set(corner_radii);
        self.queue_draw();
    }

    pub fn set_border_width(&self, border_width: [f32; 4]) {
        self.imp().border_width.set(border_width);
        self.queue_draw();
    }

    pub fn insert_child(&self, index: usize, child: &gtk4::Widget) {
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
                width:  0,
                height: 0,
            },
        );
    }

    pub fn remove_child(&self, index: usize) {
        let child = self.imp().children.borrow_mut().remove(index);
        child.widget.unparent();
    }

    pub fn swap_children(&self, _a: usize, _b: usize) {
        todo!()
    }

    pub fn set_child_layout(&self, index: usize, x: i32, y: i32, width: i32, height: i32) {
        if let Some(child) = self.imp().children.borrow_mut().get_mut(index) {
            child.x = x;
            child.y = y;
            child.width = width;
            child.height = height;
        }
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gdk4::glib::subclass::types::ObjectSubclassExt;
    use gtk4::{
        glib::{
            self,
            subclass::{object::ObjectImpl, types::ObjectSubclass},
        },
        graphene, gsk,
        prelude::{SnapshotExt, SnapshotExtManual, WidgetExt},
        subclass::widget::{WidgetClassExt, WidgetImpl, WidgetImplExt},
    };

    pub struct GroupWidget {
        pub(super) children: RefCell<Vec<Child>>,

        pub(super) width:  Cell<i32>,
        pub(super) height: Cell<i32>,

        pub(super) background_color: Cell<gdk4::RGBA>,
        pub(super) border_color:     Cell<gdk4::RGBA>,
        pub(super) corner_radii:     Cell<[f32; 4]>,
        pub(super) border_width:     Cell<[f32; 4]>,
    }

    pub(super) struct Child {
        pub(super) widget: gtk4::Widget,
        pub(super) x:      i32,
        pub(super) y:      i32,
        pub(super) width:  i32,
        pub(super) height: i32,
    }

    impl Default for GroupWidget {
        fn default() -> Self {
            Self {
                children: RefCell::default(),

                width:  Cell::new(0),
                height: Cell::new(0),

                background_color: Cell::new(gdk4::RGBA::TRANSPARENT),
                border_color:     Cell::new(gdk4::RGBA::TRANSPARENT),
                corner_radii:     Cell::new([0.0; 4]),
                border_width:     Cell::new([0.0; 4]),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GroupWidget {
        const NAME: &'static str = "Group";
        type Type = super::GroupWidget;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("group");
        }
    }

    impl ObjectImpl for GroupWidget {}

    impl WidgetImpl for GroupWidget {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let alloc = self.obj().allocation();
            let rect = graphene::Rect::new(
                0.0,
                0.0,
                alloc.width() as f32,
                alloc.height() as f32,
            );

            let [tl, tr, br, bl] = self.corner_radii.get();

            snapshot.append_border(
                &gsk::RoundedRect::new(
                    rect,
                    graphene::Size::new(tl, tl),
                    graphene::Size::new(tr, tr),
                    graphene::Size::new(br, br),
                    graphene::Size::new(bl, bl),
                ),
                &self.border_width.get(),
                &[self.border_color.get(); 4],
            );

            snapshot.append_color(&self.background_color.get(), &rect);

            self.parent_snapshot(snapshot);
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);

            for child in self.children.borrow().iter() {
                child.widget.size_allocate(
                    &gtk4::Allocation::new(
                        child.x,
                        child.y,
                        child.width,
                        child.height,
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
                gtk4::Orientation::Horizontal => (
                    self.width.get(),
                    self.width.get(),
                    -1,
                    -1,
                ),

                gtk4::Orientation::Vertical => (
                    self.height.get(),
                    self.height.get(),
                    -1,
                    -1,
                ),

                _ => (-1, -1, -1, -1),
            }
        }
    }
}
