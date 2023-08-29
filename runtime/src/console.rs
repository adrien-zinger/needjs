use maybe_static::maybe_static;
use rusty_jsc::{JSClass, JSContext, JSObject, JSObjectGenericClass, JSValue};
use rusty_jsc_macros::{callback, constructor};

use crate::{
    fs_write_stream::{get_fs_write_stream_class, FsWriteStream},
    util::format_parser,
};

enum Output {
    FsWriteStream(FsWriteStream),
    Default,
}

struct Console {
    stdout: Output,
    stderr: Output,
}

impl Console {
    fn make(context: &mut JSContext, stdout: Output, stderr: Output) -> JSObject {
        let is_default_log = matches!(stdout, Output::Default);
        let is_default_err = matches!(stderr, Output::Default);
        let has_private_data = !(is_default_log && is_default_err);
        let mut obj = make(context);
        if has_private_data && obj.set_private_data(Console { stdout, stderr }).is_err() {
            panic!("[Console constructor] Cannot set private data");
        }
        if is_default_log {
            obj.set_property(
                context,
                "log",
                JSValue::callback(context, Some(default_log)),
            )
            .unwrap();
        } else {
            obj.set_property(context, "log", JSValue::callback(context, Some(log)))
                .unwrap();
        }
        if is_default_err {
            obj.set_property(
                context,
                "error",
                JSValue::callback(context, Some(default_error)),
            )
            .unwrap();
        } else {
            obj.set_property(context, "error", JSValue::callback(context, Some(error)))
                .unwrap();
        }
        obj.into()
    }

    fn log(&mut self, value: String) {
        match &mut self.stdout {
            Output::FsWriteStream(ws) => ws.write(value),
            Output::Default => unreachable!(),
            // If reached, it means that the object hasn't
            // been initilized correctly.
        }
    }

    fn error(&mut self, value: String) {
        match &mut self.stderr {
            Output::FsWriteStream(ws) => ws.write(value),
            Output::Default => unreachable!(),
            // If reached, it means that the object hasn't
            // been initilized correctly.
        }
    }
}

#[callback]
fn default_log(context: JSContext, _function: JSObject, _this: JSObject, arguments: &[JSValue]) {
    println!("{}", format_parser(&context, arguments).unwrap().join(""));
}

#[callback]
fn default_error(context: JSContext, _function: JSObject, _this: JSObject, arguments: &[JSValue]) {
    eprintln!("{}", format_parser(&context, arguments).unwrap().join(""));
}

#[callback]
fn log(context: JSContext, _function: JSObject, mut this: JSObject, arguments: &[JSValue]) {
    let console_class = unsafe { this.as_mut_object_class_unchecked() };
    let console = unsafe { &mut *console_class.get_private_data::<Console>().unwrap() };
    console.log(format_parser(&context, arguments).unwrap().join(""));
}

#[callback]
fn error(context: JSContext, _function: JSObject, mut this: JSObject, arguments: &[JSValue]) {
    let console_class = unsafe { this.as_mut_object_class_unchecked() };
    let console = unsafe { &mut *console_class.get_private_data::<Console>().unwrap() };
    console.error(format_parser(&context, arguments).unwrap().join(""));
}

fn make(context: &JSContext) -> JSObject<JSObjectGenericClass> {
    let console_class = maybe_static!(JSClass, || JSClass::create(
        "Console",
        Some(new_console),
        None
    ));
    console_class.make_object(context)
}

#[constructor]
fn new_console(
    mut context: JSContext,
    _constructor: JSObject,
    arguments: Vec<JSValue>,
) -> JSObject {
    fn get_output(arg: &JSValue, context: &JSContext) -> Result<Output, &'static str> {
        if let Ok(mut fs) = arg.to_object_class(context, get_fs_write_stream_class()) {
            // Make a clone of the content, will increment the ARC and protect from
            // an unexpected cleanup.
            let out = unsafe { &*fs.get_private_data::<FsWriteStream>().unwrap() }.to_owned();
            Ok(Output::FsWriteStream(out))
        } else {
            Err("oups")
        }
    }

    match arguments.len() {
        1 =>
        // TODO: argument can be an object and it's the "option" case
        {
            if let Ok(output) = get_output(&arguments[0], &context) {
                Console::make(&mut context, output, Output::Default)
            } else {
                panic!("unexpected argument"); // TODO: check error message
            }
        }
        2 => match (
            get_output(&arguments[0], &context),
            get_output(&arguments[1], &context),
        ) {
            (Ok(out), Ok(err)) => Console::make(&mut context, out, err),
            _ => panic!("unexpected argument"), // TODO: check error message
        },
        3 => match (
            get_output(&arguments[0], &context),
            get_output(&arguments[1], &context),
        ) {
            (Ok(out), Ok(err)) => Console::make(&mut context, out, err),
            _ => panic!("unexpected argument"), // TODO: check error message
        }, // TODO get the third argument that is used to catch underlying errors
        _ => Console::make(&mut context, Output::Default, Output::Default),
    }
}

pub fn init(context: &mut JSContext) {
    let global = &mut context.get_global_object();
    // Define classes
    let mut console = make(context);
    console
        .set_property(
            context,
            "log",
            JSValue::callback(context, Some(default_log)),
        )
        .unwrap();
    console
        .set_property(
            context,
            "error",
            JSValue::callback(context, Some(default_error)),
        )
        .unwrap();
    console
        .set_property(context, "Console", make(context).into())
        .unwrap();
    global
        .set_property(context, "console", console.into())
        .unwrap();
}
