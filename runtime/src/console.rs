use maybe_static::maybe_static;
use rusty_jsc::{JSClass, JSContext, JSObject, JSObjectGenericClass, JSValue};
use rusty_jsc_macros::{callback, constructor};

use crate::{fs_write_stream::get_fs_write_stream_class, util::format_parser};

/*
enum Output {
    FsWriteStream(FsWriteStream),
}

struct Console {
    stdout: Output,
    stderr: Output,
}

impl Console {
    fn log(&mut self, str: String) {}
}
*/

#[callback]
fn log(context: JSContext, _function: JSObject, _this: JSObject, arguments: &[JSValue]) {
    println!("{}", format_parser(&context, arguments).unwrap().join(""));
}

fn make(context: &JSContext) -> JSObject<JSObjectGenericClass> {
    let console_class = maybe_static!(JSClass, || JSClass::create(
        "console",
        Some(new_console),
        None
    ));
    let mut console = console_class.make_object(context);
    console
        .set_property(context, "log", JSValue::callback(context, Some(log)))
        .unwrap();
    console
}

#[constructor]
fn new_console(context: JSContext, _constructor: JSObject, arguments: Vec<JSValue>) -> JSValue {
    if arguments[0].is_object_of_class(&context, get_fs_write_stream_class()) {}
    todo!()
}

pub fn init(context: &mut JSContext) {
    let global = &mut context.get_global_object();
    // Define classes
    let console = make(context);
    global
        .set_property(context, "console", console.into())
        .unwrap();
}
