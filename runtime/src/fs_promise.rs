use std::os::unix::prelude::MetadataExt;

use maybe_static::maybe_static;
use rusty_jsc::{JSClass, JSContext, JSObject, JSPromise, JSValue};
use rusty_jsc_macros::callback;

use crate::{
    event_loop::{self, get_hold, Action},
    fs::constants_object,
};

#[callback]
fn open(
    context: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> Result<JSValue, JSValue> {
    let mut context = context; // TODO: fix macro context mut
    let filename = arguments.first().unwrap().to_js_string(&context).unwrap();
    let promise = JSObject::<JSPromise>::promise(&mut context);
    event_loop::append(Action::OpenFile((filename.into(), promise.clone())));
    Ok(promise.into())
}

pub async fn exec_open((filename, promise): (String, JSObject<JSPromise>)) {
    let value = tokio::fs::read(filename).await.expect("file not found");
    let _ = get_hold().lock().await;
    let context = promise.context();
    promise.resolve(&[JSValue::string(&context, String::from_utf8(value).unwrap())]);
}

#[callback]
fn access(
    context: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> Result<JSValue, JSValue> {
    let mut context = context;
    // The script call fsPromise.access(). Send a future to the event loop. Two
    // action are possible, with `mode` parameter or without. Both are executed
    // by `exec_access` and `exec_access_with_mode` implemented below.
    let filename = arguments.first().unwrap().to_js_string(&context).unwrap();
    let promise = JSObject::<JSPromise>::promise(&mut context);
    let with_mode = arguments.first().and_then(|mode| {
        if mode.is_number(&context) {
            let value = mode.to_number(&context).unwrap();
            if value.is_normal() && value < 256.0 {
                return Some(unsafe { mode.to_number(&context).unwrap().to_int_unchecked::<u8>() });
            }
        }
        None
    });
    if let Some(mode) = with_mode {
        event_loop::append(Action::AccessFileWithMode((
            filename.to_string_utf8().unwrap(),
            promise.clone(),
            mode,
        )));
    } else {
        event_loop::append(Action::AccessFile((
            filename.to_string_utf8().unwrap(),
            promise.clone(),
        )));
    }
    Ok(promise.into())
}

/// Handle and execute asynchronously the access method of fsPromise. Just check
/// if we can open it.
pub async fn exec_access((filename, promise): (String, JSObject<JSPromise>)) {
    let visible = tokio::fs::File::open(filename.clone()).await.is_ok();
    let _ = get_hold().lock().await;
    if visible {
        promise.resolve(&[]);
    } else {
        promise.reject(&[]);
    }
}

/// Handle and execute asynchronously the access method of fsPromise with mode
/// parameter.
pub async fn exec_access_with_mode((filename, promise, mode): (String, JSObject<JSPromise>, u8)) {
    let file = match tokio::fs::File::open(filename.clone()).await {
        Ok(file) => file,
        _ => {
            let _ = get_hold().lock().await;
            promise.reject(&[]);
            return;
        }
    };

    let res = match file.metadata().await {
        Ok(metadata) => {
            // Check if file mode correspond to user/group/other access.
            let fmode = metadata.mode();
            let uid = unsafe { libc::getuid() };
            let gid = unsafe { libc::getuid() };
            let fuid = metadata.uid();
            let fgid = metadata.gid();
            let mut res = true;
            if crate::fs::constants::R_OK & mode > 0 {
                res |= fmode & 0o004 > 0
                    || fuid == uid && fmode & 0o400 > 0
                    || fgid == gid && fmode & 0o040 > 0;
            }
            if crate::fs::constants::W_OK & mode > 0 {
                res |= fmode & 0o002 > 0
                    || fuid == uid && fmode & 0o200 > 0
                    || fgid == gid && fmode & 0o020 > 0;
            }
            if crate::fs::constants::X_OK & mode > 0 {
                res |= fmode & 0o001 > 0
                    || fuid == uid && fmode & 0o100 > 0
                    || fgid == gid && fmode & 0o010 > 0;
            }
            res
        }
        _ => {
            let _ = get_hold().lock().await;
            promise.reject(&[]);
            return;
        }
    };
    let _ = get_hold().lock().await;
    if res {
        promise.resolve(&[]);
    } else {
        promise.reject(&[]);
    }
}

pub fn fs_promise(context: &JSContext) -> JSObject {
    let fs_promise_class = maybe_static!(JSClass, || JSClass::create("FsPromise", None));
    let mut fp = fs_promise_class.make_object(context);

    fp.set_property(context, "open", JSValue::callback(context, Some(open)))
        .unwrap();
    fp.set_property(context, "access", JSValue::callback(context, Some(access)))
        .unwrap();
    fp.set_property(context, "constants", constants_object(context).into())
        .unwrap();
    fp.into()
}
