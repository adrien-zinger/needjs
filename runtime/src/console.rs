use maybe_static::maybe_static;
use rusty_jsc::{JSClass, JSContext, JSObject, JSValue};
use rusty_jsc_macros::callback;

#[callback]
fn log(context: JSContext, _function: JSObject, _this: JSObject, arguments: Vec<JSValue>) {
    arguments.iter().for_each(|value| {
        if value.is_string(&context) {
            println!("{}", value.to_string(&context));
        }
        if value.is_boolean(&context) {
            println!("{}", value.to_boolean(&context));
        }
        if value.is_date(&context) {
            println!("date inthere!")
        }
    });
}

pub fn init(context: &mut JSContext) {
    let global = &mut context.get_global_object();
    // Define classes
    let console_class = maybe_static!(JSClass, || JSClass::create("console", None));
    let mut console = console_class.make_object(context);
    console.set_property(context, "log", JSValue::callback(context, Some(log)));
    global.set_property(context, "console", (&console).into());
}
