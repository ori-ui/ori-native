use ori::{Effect, Message, Proxied};

use crate::{Context, Platform, PressableEvent, Quit, TextInputEvent};

/// Rebuild the view tree.
pub fn rebuild<P, T, V, B>(build: &mut B, state: &mut V::State, cx: &mut Context<P>, data: &mut T)
where
    P: Platform,
    V: Effect<Context<P>, T>,
    B: FnMut(&T) -> V,
{
    let view = build(data);
    view.rebuild((), state, cx, data);
}

/// Handle a message.
pub fn message<P, T, V, B>(
    build: &mut B,
    state: &mut V::State,
    cx: &mut Context<P>,
    data: &mut T,
    mut message: Message,
) where
    P: Platform,
    V: Effect<Context<P>, T>,
    B: FnMut(&T) -> V,
{
    if message.is::<Quit>() {
        cx.platform.quit();
        return;
    }

    let mut action = V::message((), state, cx, data, &mut message);

    if let Some(target) = message.target()
        && !message.is::<PressableEvent>()
        && !message.is::<TextInputEvent>()
        && !message.is_taken()
    {
        tracing::warn!(
            target=?target,
            type=message.type_name(),
            "message sent but not received"
        );
    }

    if action.take_rebuild() {
        let view = build(data);
        view.rebuild((), state, cx, data);
    }

    action.rebuild = false;
    cx.send_action(action);
}
