use ori::{Action, Event, Mut, View, ViewMarker};

use crate::{NativeContext, NativeText};

pub fn text(text: impl ToString) -> Text {
    Text::new(text.to_string())
}

pub struct Text {
    text: String,
}

impl Text {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl ViewMarker for Text {}
impl<C, T> View<C, T> for Text
where
    C: NativeContext,
{
    type Element = <C::Text as NativeText<C>>::Element;
    type State = C::Text;

    fn build(self, cx: &mut C, _data: &mut T) -> (Self::Element, Self::State) {
        C::Text::build(cx, self.text)
    }

    fn rebuild(
        self,
        element: Mut<C, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        todo!()
    }

    fn event(
        element: Mut<C, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        todo!()
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut C) {
        state.teardown(element, cx);
    }
}
