use std::{collections::HashMap, pin::Pin, sync::Arc};

use ori::{Message, Proxied, Proxy};
use tokio::sync::mpsc::UnboundedSender;

use crate::application::Event;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StyleNode(u64);

impl StyleNode {
    pub fn class(&self) -> String {
        format!("w{}", self.0)
    }
}

pub struct Platform {
    pub(crate) proxy:         Gtk4Proxy,
    pub(crate) display:       gdk4::Display,
    pub(crate) application:   gtk4::Application,
    pub(crate) css_providers: HashMap<StyleNode, gtk4::CssProvider>,
    pub(crate) next_css_node: u64,
}

impl Platform {
    pub(crate) fn new(
        sender: UnboundedSender<Event>,
        display: gdk4::Display,
        application: gtk4::Application,
    ) -> Self {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());

        let provider = gtk4::CssProvider::new();
        provider.load_from_data(include_str!("default.css"));

        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        Self {
            proxy: Gtk4Proxy { sender, runtime },
            display,
            application,
            css_providers: HashMap::new(),
            next_css_node: 0,
        }
    }

    pub fn add_style(&mut self, styles: &str) -> StyleNode {
        let node = StyleNode(self.next_css_node);
        self.next_css_node += 1;

        let provider = gtk4::CssProvider::new();
        provider.load_from_data(&format!(
            ".{} {{ {} }}",
            node.class(),
            styles,
        ));

        gtk4::style_context_add_provider_for_display(
            &self.display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        self.css_providers.insert(node, provider);

        node
    }

    pub fn set_style(&mut self, node: StyleNode, styles: &str) {
        if let Some(provider) = self.css_providers.get(&node) {
            provider.load_from_data(&format!(
                ".{} {{ {} }}",
                node.class(),
                styles,
            ));
        }
    }

    pub fn remove_style(&mut self, node: StyleNode) {
        if let Some(provider) = self.css_providers.remove(&node) {
            gtk4::style_context_remove_provider_for_display(&self.display, &provider);
        }
    }
}

impl ori_native_core::Platform for Platform {
    type Widget = gtk4::Widget;

    fn quit(&mut self) {
        let _ = self.proxy.sender.send(Event::Quit);
    }
}

impl Proxied for Platform {
    type Proxy = Gtk4Proxy;

    fn proxy(&mut self) -> Self::Proxy {
        self.proxy.clone()
    }
}

#[derive(Clone)]
pub struct Gtk4Proxy {
    sender:  UnboundedSender<Event>,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl Proxy for Gtk4Proxy {
    fn cloned(&self) -> Arc<dyn Proxy> {
        Arc::new(self.clone())
    }

    fn rebuild(&self) {
        let _ = self.sender.send(Event::Rebuild);
    }

    fn message(&self, message: Message) {
        let _ = self.sender.send(Event::Message(message));
    }

    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self.runtime.spawn(future);
    }
}
