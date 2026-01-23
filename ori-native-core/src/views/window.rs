use ori::{Action, Event, Mut, Sub, View, ViewMarker};

use crate::{NativeContext, NativeWindow};

pub fn window<V>(contents: V) -> Window<V> {
    Window::new(contents)
}

pub struct Window<V> {
    contents: V,
}

impl<V> Window<V> {
    pub fn new(contents: V) -> Self {
        Self { contents }
    }
}

impl<V> ViewMarker for Window<V> {}
impl<C, T, V> View<C, T> for Window<V>
where
    C: NativeContext,
    V: View<C, T>,
    V::Element: Sub<C, C::Element>,
{
    type Element = ();
    type State = (C::Window, V::Element, V::State);

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (element, state) = self.contents.build(cx, data);

        ((), (C::Window::build(cx), element, state))
    }

    fn rebuild(
        self,
        _element: Mut<C, Self::Element>,
        (window, contents, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        //self.contents.rebuild(&mut contents, state, cx, data);
    }

    fn event(
        _element: Mut<C, Self::Element>,
        (_window, contents, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        //V::event(contents, state, cx, data, event)
        Action::new()
    }

    fn teardown(_element: Self::Element, (window, contents, state): Self::State, cx: &mut C) {
        V::teardown(contents, state, cx);
        window.teardown(cx);
    }
}
