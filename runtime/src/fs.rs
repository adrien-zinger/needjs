use maybe_static::maybe_static;
use rusty_jsc::{JSClass, JSContext, JSObject, JSPromise, JSValue};
use rusty_jsc_macros::callback;

use crate::event_loop;

#[callback]
fn open_promise(
    mut context: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: Vec<JSValue>,
) -> JSValue {
    let filename = arguments.first().unwrap().to_string(&context);
    let promise = JSObject::<JSPromise>::promise(&mut context);
    event_loop::append(event_loop::Action::OpenFile((filename, promise.clone())));
    promise.into()
}

pub fn fs_promise(context: &JSContext) -> JSObject {
    let fs_promise_class = maybe_static!(JSClass, || JSClass::create("FsPromise", None));
    let mut fp = fs_promise_class.make_object(context);

    fp.set_property(
        context,
        "open",
        JSValue::callback(context, Some(open_promise)),
    );
    fp
}
