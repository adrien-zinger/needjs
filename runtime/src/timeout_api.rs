use std::{collections::HashMap, time::Duration};

use maybe_static::maybe_static_unsafe;
use rusty_jsc::{JSContext, JSObject, JSProtected, JSValue};
use rusty_jsc_macros::callback;
use tokio::sync::{
    oneshot::{Receiver, Sender},
    Mutex,
};

use crate::event_loop::{self, Action};

#[derive(Default)]
struct TimeoutCancelers {
    cancel_senders: HashMap<u32, Sender<()>>,
    index: u32,
}

impl TimeoutCancelers {
    fn append(&mut self) -> (u32, Receiver<()>) {
        self.index = if self.index == u32::MAX {
            0
        } else {
            self.index + 1
        };
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.cancel_senders.insert(self.index, sender);
        (self.index, receiver)
    }

    fn cancel(&mut self, index: u32) {
        if let Some(cancel_sender) = self.cancel_senders.remove(&index) {
            let _ = cancel_sender.send(());
        }
    }

    fn remove(&mut self, index: u32) {
        self.cancel_senders.remove(&index);
    }
}

/// Should be only used in a single threaded context. Fortunatelly, Javascript
/// is one of these contexts.
fn get_timeout_cancelers() -> &'static mut TimeoutCancelers {
    let timeouts = maybe_static_unsafe!(TimeoutCancelers);
    timeouts
}

/// User call of `setTimeout`
pub struct TimeoutAction {
    pub index: u32,
    pub callback: JSObject<JSProtected>,
    pub time: Duration,
    pub cancel_receiver: Receiver<()>,
}

#[callback]
fn set_timeout(
    context: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: Vec<JSValue>,
) -> JSValue {
    let callback = arguments[0].clone().to_protected_object(&context);
    let time = arguments[1].to_number(&context);
    let millis = unsafe { time.to_int_unchecked() };

    // manage cancellation
    let (index, cancel_receiver) = get_timeout_cancelers().append();
    event_loop::append(Action::SetTimeout(TimeoutAction {
        index,
        callback,
        time: Duration::from_millis(millis),
        cancel_receiver,
    }));
    JSValue::number(&context, index.into())
}

#[callback]
fn clear_timeout(
    context: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: Vec<JSValue>,
) {
    let index = arguments[0].to_number(&context);
    let index = unsafe { index.to_int_unchecked() };
    get_timeout_cancelers().cancel(index);
}

pub async fn exec_timeout(action: TimeoutAction, hold: &Mutex<()>) {
    tokio::select! {
        _ = tokio::time::sleep(action.time) => {},
        _ = action.cancel_receiver => return,
    }
    let _ = hold.lock();
    get_timeout_cancelers().remove(action.index);
    action.callback.call_as_function();
}

pub fn init(context: &mut JSContext) {
    let mut global = context.get_global_object();
    global.set_property(
        context,
        "setTimeout",
        JSValue::callback(context, Some(set_timeout)),
    );
    global.set_property(
        context,
        "clearTimeout",
        JSValue::callback(context, Some(clear_timeout)),
    );
}
