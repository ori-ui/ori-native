use ori_native_core::NativeText;

use crate::{Context, NodeKind, context::NodeId};

pub struct Text {}

impl NativeText<Context> for Text {
    type Element = NodeId;

    fn build(cx: &mut Context, text: String) -> (NodeId, Self) {
        let node = cx.create_node(NodeKind::Text);
        cx.set_text(node, text);
        (node, Self {})
    }

    fn teardown(self, element: Self::Element, cx: &mut Context) {
        cx.delete_node(element);
    }

    fn set_text(&mut self, _element: Self::Element, _cx: &mut Context, _text: String) {}
}
