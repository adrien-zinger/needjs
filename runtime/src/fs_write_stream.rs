use maybe_static::maybe_static;
use rusty_jsc::{
    callback, JSClass, JSContext, JSObject, JSObjectGenericClass, JSProtected, JSValue,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use tokio::{fs::File, io::AsyncWriteExt, sync::Mutex};

use crate::event_loop::{self, get_hold, Action};

/// A WriteStream file can be a File, when the event loop has resolved the
/// file creation. Or an u32 ID, when the object is waiting for the file
/// to open.
///
/// When this WriteStream is created, we append to the event loop the CreateFile
/// action. This will, of course, asynchronously cause the creation of the file,
/// but we don't modify the JSObject because we don't want to store it. Instead,
/// we use a static Hashmap to store the file with the given ID as a key. The ID
/// is just a value incremented each time we require a WriteStream. However, we
/// don't want to look in the Hashmap each time we need the file, since the
/// first time allow us to just move it from the Hashmap to that enum.
pub enum WSFile {
    File(File),
    ID(u32),
    Closed,
}

pub struct WriteStream {
    /// Protected pointer on a WSFile.
    ///
    /// Note: The pointer to WSFile::File is set asynchronously.
    file: Arc<Mutex<WSFile>>,
    callbacks: Arc<std::sync::Mutex<WriteStreamCallbacks>>,
    pending: Arc<AtomicU32>,
}

#[derive(Default)]
pub struct WriteStreamCallbacks {
    /// On close callback.
    on_close: Option<JSObject<JSProtected>>,
    /// On finish callback.
    on_finish: Option<JSObject<JSProtected>>,
}

/// Get WriteStreamClass
fn get_write_stream_class() -> &'static JSClass {
    maybe_static!(JSClass, || JSClass::create("WriteStream", None))
}

/// Get asynchronously opened file hashmap
fn get_async_opened_files() -> &'static Mutex<HashMap<u32, File>> {
    maybe_static!(Mutex::<HashMap::<u32, File>>)
}

// Asynchronous functions called by the event loop.
// * CreateFile => exec_create_file
// * WriteInFile => exec_write_str
// * CloseFile => exec_close

pub async fn exec_create_file(path: String, id: u32) {
    let file = File::create(path).await.unwrap(); // TODO: signal an error (keep a callback)
    get_async_opened_files().lock().await.insert(id, file);
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
            WSFile::ID(id) => {
                if let Some(mut file) = get_async_opened_files().lock().await.remove(id) {
                    file.write_all(value.as_bytes()).await.unwrap();
                    *wsf = WSFile::File(file);
                    pending.fetch_sub(1, Ordering::Release);
                    return;
                }
            }
            WSFile::Closed => {
                panic!("cannot be closed with pending write requests")
            }
        }
    }
    // No file found, retry
    event_loop::append(Action::WriteInWSFile(ws_file, value, pending));
}

async fn call_close_callbacks(
    callbacks: Arc<std::sync::Mutex<WriteStreamCallbacks>>,
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
    callbacks: Arc<std::sync::Mutex<WriteStreamCallbacks>>,
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
            WSFile::ID(id) => {
                if get_async_opened_files().lock().await.remove(id).is_some() {
                    *wsf = WSFile::Closed;
                    call_close_callbacks(callbacks, context).await;
                    return;
                } else {
                }
            }
            WSFile::Closed => {
                return; /* Nothing to do */
            }
        }
    }
    // No file found, retry
    event_loop::append(Action::CloseWSFile(ws_file, callbacks, context, pending));
}

fn next() -> u32 {
    use std::sync::atomic::Ordering::*;
    static COUNTER: AtomicU32 = AtomicU32::new(0);

    let mut curr = COUNTER.load(Acquire);
    let mut next = if curr == u32::MAX {
        // ??? is a modulo better here?
        0
    } else {
        curr + 1
    };
    while let Err(c) = COUNTER.compare_exchange(curr, next, Acquire, Acquire) {
        curr = c;
        next = if curr == u32::MAX {
            // ??? is a modulo better here?
            0
        } else {
            curr + 1
        };
    }
    curr
}

impl WriteStream {
    /// Create a new `WriteStream` JS object. Open file for writing. The file is
    /// created (if it does not exist) or truncated (if it exists).
    pub fn make(context: &JSContext, path: String) -> JSObject<JSObjectGenericClass> {
        let mut object = get_write_stream_class().make_object(context);
        let id = next();
        event_loop::append(event_loop::Action::CreateWSFile(path, id));
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
            .set_private_data(WriteStream {
                file: Arc::new(Mutex::new(WSFile::ID(id))),
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
    ) -> Result<&'a mut WriteStream, JSValue> {
        let object = object.try_as_mut_object_class(context, get_write_stream_class())?;
        let ws: &mut WriteStream = unsafe { &mut *object.get_private_data().unwrap() };
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
fn on(context: JSContext, _function: JSObject, mut this: JSObject, arguments: &[JSValue]) {
    let ws = WriteStream::try_from_object(&context, &mut this).unwrap();
    ws.on(
        arguments[0].to_js_string(&context).unwrap().to_string(),
        arguments[1].to_owned().into_protected_object(&context),
    );
}

#[callback]
fn close(context: JSContext, _function: JSObject, mut this: JSObject, _arguments: &[JSValue]) {
    let ws = WriteStream::try_from_object(&context, &mut this).unwrap();
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
///
fn write(
    context: JSContext,
    _function: JSObject,
    mut this: JSObject,
    arguments: &[JSValue],
) -> Result<JSValue, JSValue> {
    match arguments.get(0) {
        Some(value) if value.is_string(&context) => {
            let ws = WriteStream::try_from_object(&context, &mut this).unwrap();
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
