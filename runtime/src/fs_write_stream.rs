#![allow(unused)] // TODO: work in progress

use maybe_static::maybe_static;
use rusty_jsc::{JSClass, JSContext, JSObject, JSObjectGenericClass};

struct WriteStream {
    object: JSObject<JSObjectGenericClass>,
}

/// Get WriteStreamClass
fn get_write_stream_class() -> &'static JSClass {
    maybe_static!(JSClass, || JSClass::create("WriteStream", None))
}

impl WriteStream {
    fn new(context: &JSContext, path: String) -> Self {
        let object = get_write_stream_class().make_object(context);
        WriteStream { object }
    }
}
