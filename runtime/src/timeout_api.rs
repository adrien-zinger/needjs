use std::time::Duration;

use rusty_jsc::{JSContext, JSObject, JSProtected, JSValue};
use rusty_jsc_macros::callback;
use tokio::sync::Mutex;

use crate::event_loop::{self, Action};

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
    event_loop::append(Action::SetTimeout((
        callback,
        Duration::from_millis(millis),
    )));
    JSValue::number(&context, 0f64)
}

pub async fn exec_timeout(
    (callback, duration): (JSObject<JSProtected>, Duration),
    hold: &Mutex<()>,
) {
    tokio::time::sleep(duration).await;
    let _ = hold.lock();
    callback.call_as_function()
}

pub fn init(context: &mut JSContext) {
    let mut global = context.get_global_object();
    global.set_property(
        context,
        "setTimeout",
        JSValue::callback(context, Some(set_timeout)),
    )
}
