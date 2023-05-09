use maybe_static::maybe_static;
use rusty_jsc::{
    callback, JSClass, JSContext, JSObject, JSObjectGenericClass, JSProtected, JSValue,
};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use tokio::{fs::File, io::AsyncWriteExt, sync::Mutex};

use crate::event_loop::{self, get_hold, Action};

/// A WriteStream file can be a File, when the event loop has resolved the
/// file creation. Or Waiting, when the object is waiting for the file
/// to open.
pub enum WSFile {
    File(File),
    Waiting,
    Closed,
}

pub struct FsWriteStream {
    /// Protected pointer on a WSFile.
    ///
    /// Note: The pointer to WSFile::File is set asynchronously.
    file: Arc<Mutex<WSFile>>,
    callbacks: Arc<std::sync::Mutex<FsWriteStreamCallbacks>>,
    pending: Arc<AtomicU32>,
}

// If I add the following line, I can see the output:
//
// > drop waiting!
// > drop file!
// > drop closed!
//
// impl Drop for WSFile {
//     fn drop(&mut self) {
//         match self {
//             WSFile::File(_) => println!("drop file!"),
//             WSFile::Waiting => println!("drop waiting!"),
//             WSFile::Closed => println!("drop closed!"),
//         }
//     }
// }

#[derive(Default)]
pub struct FsWriteStreamCallbacks {
    /// On close callback.
    on_close: Option<JSObject<JSProtected>>,
    /// On finish callback.
    on_finish: Option<JSObject<JSProtected>>,
}

/// Get WriteStreamClass
pub fn get_fs_write_stream_class() -> &'static JSClass {
    maybe_static!(JSClass, || JSClass::create(
        "WriteStream",
        None,
        Some(destructor)
    ))
}

// Asynchronous functions called by the event loop.
// * CreateFile => exec_create_file
// * WriteInFile => exec_write_str
// * CloseFile => exec_close

pub async fn exec_create_file(path: String, ws_file: Arc<Mutex<WSFile>>) {
    let file = File::create(path).await.unwrap(); // TODO: signal an error (keep a callback)
    *ws_file.lock().await = WSFile::File(file);
}

pub async fn exec_write_str(ws_file: Arc<Mutex<WSFile>>, value: String, pending: Arc<AtomicU32>) {
    {
        let wsf = &mut *ws_file.lock().await;
        match wsf {
            WSFile::File(file) => {
                file.write_all(value.as_bytes()).await.unwrap();
                pending.fetch_sub(1, Ordering::Release);
                return;
            }
            WSFile::Waiting => { /* Nothing to do */ }
            WSFile::Closed => {
                panic!("cannot be closed with pending write requests")
            }
        }
    }
    // No file found, retry later
    event_loop::append(Action::WriteInWSFile(ws_file, value, pending));
}

async fn call_close_callbacks(
    callbacks: Arc<std::sync::Mutex<FsWriteStreamCallbacks>>,
    context: JSContext,
) {
    let _ = get_hold().lock().await;
    let cbs = callbacks.lock().unwrap();
    if let Some(finish) = &cbs.on_finish {
        finish.call_as_function(&context, None, &[]).unwrap();
    }
    if let Some(close) = &cbs.on_close {
        close.call_as_function(&context, None, &[]).unwrap();
    }
}

pub async fn exec_close(
    ws_file: Arc<Mutex<WSFile>>,
    callbacks: Arc<std::sync::Mutex<FsWriteStreamCallbacks>>,
    context: JSContext,
    pending: Arc<AtomicU32>,
) {
    if pending.load(Ordering::Acquire) == 0 {
        let wsf = &mut *ws_file.lock().await;
        match wsf {
            WSFile::File(_) => {
                *wsf = WSFile::Closed;
                call_close_callbacks(callbacks, context).await;
                return;
            }
            WSFile::Waiting => { /* Nothing to do */ }
            WSFile::Closed => {
                return; /* Already closed ??? warning */
            }
        }
    }
    // No file found or pending action, retry later
    event_loop::append(Action::CloseWSFile(ws_file, callbacks, context, pending));
}

impl FsWriteStream {
    /// Create a new `WriteStream` JS object. Open file for writing. The file is
    /// created (if it does not exist) or truncated (if it exists).
    pub fn make(context: &JSContext, path: String) -> JSObject<JSObjectGenericClass> {
        let mut object = get_fs_write_stream_class().make_object(context);
        let file = Arc::new(Mutex::new(WSFile::Waiting));
        event_loop::append(event_loop::Action::CreateWSFile(path, file.clone()));
        object
            .set_property(context, "on", JSValue::callback(context, Some(on)))
            .unwrap();
        object
            .set_property(context, "close", JSValue::callback(context, Some(close)))
            .unwrap();
        object
            .set_property(context, "write", JSValue::callback(context, Some(write)))
            .unwrap();
        if object
            .set_private_data(FsWriteStream {
                file,
                callbacks: Default::default(),
                pending: Default::default(),
            })
            .is_err()
        {
            panic!("cannot set private data to writestream");
        }
        object
    }

    pub fn try_from_object<'a>(
        context: &JSContext,
        object: &mut JSObject,
    ) -> Result<&'a mut FsWriteStream, JSValue> {
        let object = object.try_as_mut_object_class(context, get_fs_write_stream_class())?;
        let ws: &mut FsWriteStream = unsafe { &mut *object.get_private_data().unwrap() };
        Ok(ws)
    }

    pub fn try_take_from_object(object: &mut JSObject) -> Result<Box<FsWriteStream>, JSValue> {
        let object = unsafe { object.as_mut_object_class_unchecked() };
        let ws: Box<FsWriteStream> = unsafe { Box::from_raw(object.get_private_data().unwrap()) };
        object
            .set_private_data(std::ptr::null_mut() as *mut ())
            .unwrap();
        Ok(ws)
    }

    fn on(&mut self, event: String, object: JSObject<JSProtected>) {
        match event.as_str() {
            "close" => {
                if let Ok(mut cbs) = self.callbacks.try_lock() {
                    (*cbs).on_close = Some(object)
                }
            }
            "finish" => {
                if let Ok(mut cbs) = self.callbacks.try_lock() {
                    (*cbs).on_finish = Some(object)
                }
            }
            _ => {}
        };
    }

    fn close(&mut self, context: JSContext) {
        event_loop::append(Action::CloseWSFile(
            self.file.clone(),
            self.callbacks.clone(),
            context,
            self.pending.clone(),
        ))
    }

    fn write(&mut self, value: String) {
        self.pending.fetch_add(1, Ordering::Release);
        event_loop::append(Action::WriteInWSFile(
            self.file.clone(),
            value,
            self.pending.clone(),
        ))
    }
}

pub unsafe extern "C" fn destructor(this: rusty_jsc::private::JSObjectRef) {
    FsWriteStream::try_take_from_object(&mut JSObject::from(this)).unwrap();
}

#[callback]
pub fn create_write_stream(
    context: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> Result<JSValue, JSValue> {
    Ok(FsWriteStream::make(
        &context,
        arguments[0].to_js_string(&context).unwrap().to_string(),
    )
    .into())
}

#[callback]
fn on(context: JSContext, _function: JSObject, mut this: JSObject, arguments: &[JSValue]) {
    let ws = FsWriteStream::try_from_object(&context, &mut this).unwrap();
    ws.on(
        arguments[0].to_js_string(&context).unwrap().to_string(),
        arguments[1].to_owned().into_protected_object(&context),
    );
}

#[callback]
fn close(context: JSContext, _function: JSObject, mut this: JSObject, _arguments: &[JSValue]) {
    let ws = FsWriteStream::try_from_object(&context, &mut this).unwrap();
    ws.close(context);
}

#[callback]
/// Javascript call of fsWriteStream.write(). Returns false if the stream wishes
/// for the calling code to wait for the 'drain' event to be emitted before
/// continuing to write additional data; otherwise true.
///
/// # Parameters
///
/// Signature: `writable.write(chunk[, encoding][, callback])`.
///
/// * chunk <string> | <Buffer> | <Uint8Array> | <any> Optional data to write.
///   For streams not operating in object mode, chunk must be a string, Buffer
///   or Uint8Array. For object mode streams, chunk may be any JavaScript value
///   other than null.
/// * encoding <string> The encoding, if chunk is a string
/// * callback <Function> Callback for when this chunk of data is flushed
///
/// # Nodejs API documentation
///
/// The writable.write() method writes some data to the stream, and calls the
/// supplied callback once the data has been fully handled. If an error occurs,
/// the callback may or may not be called with the error as its first argument.
/// To reliably detect write errors, add a listener for the 'error' event.
fn write(
    context: JSContext,
    _function: JSObject,
    mut this: JSObject,
    arguments: &[JSValue],
) -> Result<JSValue, JSValue> {
    match arguments.get(0) {
        Some(value) if value.is_string(&context) => {
            let ws = FsWriteStream::try_from_object(&context, &mut this).unwrap();
            ws.write(value.to_js_string(&context).unwrap().to_string())
        }
        Some(_) => todo!("No implementation for other types than string"),
        _ => return Err(JSValue::string(&context, "Missing arguments")),
    };
    Ok(JSValue::boolean(&context, true))
}

/*

The return value is true if the internal buffer is less than the highWaterMark configured when the stream was created after admitting chunk. If false is returned, further attempts to write data to the stream should stop until the 'drain' event is emitted.

While a stream is not draining, calls to write() will buffer chunk, and return false. Once all currently buffered chunks are drained (accepted for delivery by the operating system), the 'drain' event will be emitted. It is recommended that once write() returns false, no more chunks be written until the 'drain' event is emitted. While calling write() on a stream that is not draining is allowed, Node.js will buffer all written chunks until maximum memory usage occurs, at which point it will abort unconditionally. Even before it aborts, high memory usage will cause poor garbage collector performance and high RSS (which is not typically released back to the system, even after the memory is no longer required). Since TCP sockets may never drain if the remote peer does not read the data, writing a socket that is not draining may lead to a remotely exploitable vulnerability.

Writing data while the stream is not draining is particularly problematic for a Transform, because the Transform streams are paused by default until they are piped or a 'data' or 'readable' event handler is added.

If the data to be written can be generated or fetched on demand, it is recommended to encapsulate the logic into a Readable and use stream.pipe(). However, if calling write() is preferred, it is possible to respect backpressure and avoid memory issues using the 'drain' event:

function write(data, cb) {
  if (!stream.write(data)) {
    stream.once('drain', cb);
  } else {
    process.nextTick(cb);
  }
}

// Wait for cb to be called before doing any other write.
write('hello', () => {
  console.log('Write completed, do more writes now.');
});

A Writable stream in object mode will always ignore the encoding argument.

 */
