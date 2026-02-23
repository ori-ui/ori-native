use ori::Proxied;

pub trait Platform: Proxied + Sized + 'static {
    type Widget;

    fn quit(&mut self);
}
