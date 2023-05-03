use maybe_static::maybe_static;
use rusty_jsc::{
    callback, JSClass, JSContext, JSObject, JSObjectGenericClass, JSProtected, JSValue,
};
use std::fs::File;

pub struct WriteStream {
    #[allow(unused)]
    file: File,
    /// On close callback.
    on_close: Option<JSObject<JSProtected>>,
    /// On finish.
    on_finish: Option<JSObject<JSProtected>>,
    this: JSObject,
}

/// Get WriteStreamClass
fn get_write_stream_class() -> &'static JSClass {
    maybe_static!(JSClass, || JSClass::create("WriteStream", None))
}

impl WriteStream {
    /// Create a new `WriteStream` JS object. Open file for writing. The file is
    /// created (if it does not exist) or truncated (if it exists).
    pub fn make(context: &JSContext, path: String) -> JSObject<JSObjectGenericClass> {
        let mut object = get_write_stream_class().make_object(context);
        // keep a reference of this object inside WriteStream. We shouldn't have
        // to protect it from garbage collection since it is mandatory for later
        // calls.
        let this = object.clone().into();
        let file = File::create(path).unwrap();
        object
            .set_property(context, "on", JSValue::callback(context, Some(on)))
            .unwrap();
        object
            .set_property(context, "close", JSValue::callback(context, Some(close)))
            .unwrap();
        if object
            .set_private_data(WriteStream {
                file,
                on_close: None,
                on_finish: None,
                this,
            })
            .is_err()
        {
            panic!("cannot set private data to writestream");
        }
        object
    }

    pub fn try_from_object(
        context: &JSContext,
        object: JSObject,
    ) -> Result<&mut WriteStream, JSValue> {
        let mut object = object.try_into_object_class(context, get_write_stream_class())?;
        let ws: &mut WriteStream = unsafe { &mut *object.get_private_data().unwrap() };
        Ok(ws)
    }

    pub fn on(&mut self, event: String, object: JSObject<JSProtected>) {
        match event.as_str() {
            "close" => self.on_close = Some(object),
            "finish" => self.on_finish = Some(object),
            _ => {}
        };
    }

    pub fn close(&self, context: &JSContext) -> Result<(), JSValue> {
        if let Some(finish) = &self.on_finish {
            finish.call_as_function(context, Some(&self.this), &[])?;
        }
        if let Some(close) = &self.on_close {
            close.call_as_function(context, Some(&self.this), &[])?;
        }
        Ok(())
    }
}

#[callback]
pub fn create_write_stream(
    context: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> Result<JSValue, JSValue> {
    Ok(WriteStream::make(
        &context,
        arguments[0].to_js_string(&context).unwrap().to_string(),
    )
    .into())
}

#[callback]
fn on(context: JSContext, _function: JSObject, this: JSObject, arguments: &[JSValue]) {
    let ws = WriteStream::try_from_object(&context, this).unwrap();
    ws.on(
        arguments[0].to_js_string(&context).unwrap().to_string(),
        arguments[1].to_owned().into_protected_object(&context),
    );
}

#[callback]
fn close(context: JSContext, _function: JSObject, this: JSObject, _arguments: &[JSValue]) {
    let ws = WriteStream::try_from_object(&context, this).unwrap();
    ws.close(&context).unwrap();
}
