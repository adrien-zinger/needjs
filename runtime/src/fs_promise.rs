use maybe_static::maybe_static;
use rusty_jsc::{JSClass, JSContext, JSObject, JSPromise, JSValue};
use rusty_jsc_macros::callback;

use crate::event_loop::{self, Action};

#[callback]
fn open(
    mut context: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: Vec<JSValue>,
) -> JSValue {
    let filename = arguments.first().unwrap().to_string(&context);
    let promise = JSObject::<JSPromise>::promise(&mut context);
    event_loop::append(Action::OpenFile((filename, promise.clone())));
    promise.into()
}

pub async fn exec_open((filename, promise): (String, JSObject<JSPromise>)) {
    let value = tokio::fs::read(filename).await.expect("file not found");
    let context = promise.context();
    promise.resolve(&[JSValue::string(&context, String::from_utf8(value).unwrap()).unwrap()]);
}

#[callback]
fn access(
    mut context: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: Vec<JSValue>,
) -> JSValue {
    let filename = arguments.first().unwrap().to_string(&context);
    let promise = JSObject::<JSPromise>::promise(&mut context);
    let with_mode = arguments.first().and_then(|mode| {
        if mode.is_number(&context) {
            let value = mode.to_number(&context);
            if value.is_normal() && value < 256.0 {
                return Some(unsafe { mode.to_number(&context).to_int_unchecked::<u8>() });
            }
        }
        None
    });
    if let Some(mode) = with_mode {
        event_loop::append(Action::AccessFileWithMode((
            filename,
            mode,
            promise.clone(),
        )));
    } else {
        event_loop::append(Action::AccessFile((filename, promise.clone())));
    }
    promise.into()
}

pub async fn exec_access((filename, promise): (String, JSObject<JSPromise>)) {
    //let file = tokio::fs::File::open(filename.clone()).await.unwrap();
    //let metadata = file.metadata().await.unwrap();

    let value = tokio::fs::read(filename).await.expect("file not found");
    let context = promise.context();
    promise.resolve(&[JSValue::string(&context, String::from_utf8(value).unwrap()).unwrap()]);
}

pub fn fs_promise(context: &JSContext) -> JSObject {
    let fs_promise_class = maybe_static!(JSClass, || JSClass::create("FsPromise", None));
    let mut fp = fs_promise_class.make_object(context);

    fp.set_property(context, "open", JSValue::callback(context, Some(open)));
    fp.set_property(context, "access", JSValue::callback(context, Some(access)));
    fp
}
