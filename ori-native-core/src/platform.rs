use ori::Proxied;

pub trait Platform: Proxied + Sized + 'static {
    type Widget;

    fn replace(&mut self, widget: &Self::Widget, other: &Self::Widget);

    fn quit(&mut self);
}
