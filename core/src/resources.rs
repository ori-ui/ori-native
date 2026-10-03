use std::{
    any::{self, Any, TypeId},
    fmt,
};

use ori::Provider;

/// An implementation of [`Provider`].
#[derive(Debug, Default)]
pub struct Resources {
    resources: Vec<Resource>,
}

#[allow(dead_code)]
struct Resource {
    type_id:   TypeId,
    type_name: &'static str,
    value:     Box<dyn Any>,
}

impl Resources {
    /// Create new [`Resources`].
    pub const fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }
}

impl Provider for Resources {
    fn push<T: Any>(&mut self, resource: Box<T>) {
        self.resources.push(Resource {
            type_id:   TypeId::of::<T>(),
            type_name: any::type_name::<T>(),
            value:     resource,
        });
    }

    fn pop<T: Any>(&mut self) -> Option<Box<T>> {
        let i = self.resources.iter().rposition(|r| r.is::<T>())?;

        let resource = self.resources.remove(i);
        let resource = unsafe { resource.downcast_unchecked::<T>() };
        Some(resource)
    }

    fn get<T: Any>(&self) -> Option<&T> {
        let resource = self.resources.iter().rev().find(|r| r.is::<T>())?;
        let resource = unsafe { resource.downcast_ref_unchecked::<T>() };
        Some(resource)
    }

    fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let resource = self.resources.iter_mut().rev().find(|r| r.is::<T>())?;
        let resource = unsafe { resource.downcast_mut_unchecked::<T>() };
        Some(resource)
    }
}

impl fmt::Debug for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.type_name)
    }
}

impl Resource {
    fn is<T: Any>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }

    unsafe fn downcast_unchecked<T: Any>(self) -> Box<T> {
        let ptr: *mut T = Box::into_raw(self.value).cast();
        unsafe { Box::from_raw(ptr) }
    }

    unsafe fn downcast_ref_unchecked<T: Any>(&self) -> &T {
        let ptr = self.value.as_ref() as *const _ as *const T;
        unsafe { &*ptr }
    }

    unsafe fn downcast_mut_unchecked<T: Any>(&mut self) -> &mut T {
        let ptr = self.value.as_mut() as *mut _ as *mut T;
        unsafe { &mut *ptr }
    }
}
