use std::{
    mem,
    sync::mpsc::{Receiver, Sender, channel},
};

use ori::{BaseElement, Element, Mut, Sub, ViewId};
use ori_native_core::NativeContext;
use serde::Serialize;

use crate::widgets;

pub struct Context {
    sender: Sender<Signal>,
    receiver: Receiver<Signal>,
    commands: Vec<Command>,
    next_node: u64,
}

impl Context {
    pub(crate) fn new() -> Self {
        let (sender, receiver) = channel();

        Self {
            sender,
            receiver,
            commands: Vec::new(),
            next_node: 0,
        }
    }

    pub(crate) fn next_signal(&mut self) -> Option<Signal> {
        self.receiver.try_recv().ok()
    }

    pub(crate) fn take_commands(&mut self) -> Vec<Command> {
        mem::take(&mut self.commands)
    }

    pub fn create_window(&mut self, view_id: ViewId) {
        let _ = self.sender.send(Signal::CreateWindow { view_id });
    }

    pub fn create_node(&mut self, kind: NodeKind) -> NodeId {
        let node = NodeId(self.next_node);
        self.next_node += 1;
        self.commands.push(Command::CreateNode { node, kind });
        node
    }

    pub fn delete_node(&mut self, node: NodeId) {
        self.commands.push(Command::DeleteNode { node });
    }

    pub fn set_text(&mut self, node: NodeId, text: String) {
        self.commands.push(Command::SetText { node, text });
    }

    pub fn set_style(&mut self, node: NodeId, key: String, value: String) {
        self.commands.push(Command::SetStyle { node, key, value });
    }
}

impl BaseElement for Context {
    type Element = NodeId;
}

impl NativeContext for Context {
    type Window = widgets::Window;
    type Text = widgets::Text;
}

impl Element<Context> for NodeId {
    type Mut<'a> = &'a mut NodeId;
}

impl Sub<Context, NodeId> for NodeId {
    fn replace(cx: &mut Context, this: Mut<Context, Self>, other: NodeId) -> Self {
        todo!()
    }

    fn upcast(cx: &mut Context, sub: NodeId) -> Self {
        todo!()
    }

    fn downcast(cx: &mut Context, this: NodeId) -> Option<Self> {
        todo!()
    }

    fn downcast_mut<T>(
        cx: &mut Context,
        this: Mut<Context, NodeId>,
        f: impl FnOnce(&mut Context, Mut<Context, Self>) -> T,
    ) -> Option<T> {
        todo!()
    }
}

pub enum Signal {
    CreateWindow { view_id: ViewId },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Command {
    CreateNode {
        node: NodeId,
        kind: NodeKind,
    },

    DeleteNode {
        node: NodeId,
    },

    SetLayout {
        node: NodeId,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },

    SetText {
        node: NodeId,
        text: String,
    },

    SetStyle {
        node: NodeId,
        key: String,
        value: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct NodeId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeKind {
    Text,
}
