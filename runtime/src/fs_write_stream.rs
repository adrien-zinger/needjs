use maybe_static::maybe_static;
use rusty_jsc::{JSClass, JSContext, JSObject, JSObjectGenericClass};
use rusty_jsc_sys::JSObjectSetPrivate;

struct WriteStream;

/// Get WriteStreamClass
fn get_class() -> &'static JSClass {
    maybe_static!(JSClass, || JSClass::create("WriteStream", None))
}

impl WriteStream {
    fn make(context: &JSContext) -> JSObject<JSObjectGenericClass> {
        let stream = get_class().make_object(context);
        stream.
        // todo
        stream
    }
}
