/// Values for each corner of a rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Corners<T> {
    /// The top left corner.
    pub top_left: T,

    /// The top right corner.
    pub top_right: T,

    /// The bottom right corner.
    pub bottom_right: T,

    /// The bottom left corner.
    pub bottom_left: T,
}

impl<T> Corners<T> {
    /// Create new [`Corners`] with the same value for all corners.
    pub const fn all(value: T) -> Self
    where
        T: Copy,
    {
        Self {
            top_left:     value,
            top_right:    value,
            bottom_right: value,
            bottom_left:  value,
        }
    }
}

impl From<f32> for Corners<f32> {
    fn from(value: f32) -> Self {
        Self::all(value)
    }
}

impl<I, T, U, V, W> From<(T, U, V, W)> for Corners<I>
where
    T: Into<I>,
    U: Into<I>,
    V: Into<I>,
    W: Into<I>,
{
    fn from((top_left, top_right, bottom_right, bottom_left): (T, U, V, W)) -> Self {
        Self {
            top_left:     top_left.into(),
            top_right:    top_right.into(),
            bottom_right: bottom_right.into(),
            bottom_left:  bottom_left.into(),
        }
    }
}

impl<T, U> From<[T; 4]> for Corners<U>
where
    T: Into<U>,
{
    fn from([top_left, top_right, bottom_right, bottom_left]: [T; 4]) -> Self {
        Self {
            top_left:     top_left.into(),
            top_right:    top_right.into(),
            bottom_right: bottom_right.into(),
            bottom_left:  bottom_left.into(),
        }
    }
}

impl<T> From<Corners<T>> for [T; 4] {
    fn from(corners: Corners<T>) -> Self {
        [
            corners.top_left,
            corners.top_right,
            corners.bottom_right,
            corners.bottom_left,
        ]
    }
}

/// A trait for styling corner radii.
pub trait StyleCorners: Sized {
    /// Get a mutable reference to the corner style.
    fn get_corners_mut(&mut self) -> &mut Corners<f32>;

    /// Set the radius of all corners.
    fn corner(mut self, radii: impl Into<Corners<f32>>) -> Self {
        *self.get_corners_mut() = radii.into();
        self
    }

    /// Set the radius of the top left corner.
    fn corner_top_left(mut self, radius: f32) -> Self {
        self.get_corners_mut().top_left = radius;
        self
    }

    /// Set the radius of the top right corner.
    fn corner_top_right(mut self, radius: f32) -> Self {
        self.get_corners_mut().top_right = radius;
        self
    }

    /// Set the radius of the bottom right corner.
    fn corner_bottom_right(mut self, radius: f32) -> Self {
        self.get_corners_mut().bottom_right = radius;
        self
    }

    /// Set the radius of the bottom left corner.
    fn corner_bottom_left(mut self, radius: f32) -> Self {
        self.get_corners_mut().bottom_left = radius;
        self
    }
}
