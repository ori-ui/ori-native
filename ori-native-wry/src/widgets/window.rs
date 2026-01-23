use ori::ViewId;
use ori_native_core::NativeWindow;

use crate::Context;

pub struct Window {
    view_id: ViewId,
}

impl NativeWindow<Context> for Window {
    fn build(cx: &mut Context) -> Self {
        let view_id = ViewId::next();

        cx.create_window(view_id);

        Self { view_id }
    }

    fn teardown(self, cx: &mut Context) {}
}
