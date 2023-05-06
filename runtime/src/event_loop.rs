use std::sync::{
    atomic::{AtomicU32, AtomicUsize, Ordering},
    Arc,
};

use maybe_static::maybe_static;
use rusty_jsc::{JSContext, JSObject, JSPromise};
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    oneshot::Sender,
    Mutex, Notify,
};

use crate::{
    fs_promise::*,
    fs_write_stream::{exec_close, exec_create_file, exec_write_str, WSFile, WriteStreamCallbacks},
    timeout_api::{exec_timeout, TimeoutAction},
};

pub enum Action {
    /// Check file accessibility (Filename/path, Promise Object).
    ///
    /// Binded with fsPromise.access(path[,mode]) in javascript.
    AccessFile((String, JSObject<JSPromise>)),
    /// Check file accessibility (Filename/path, Promise Object). This is the
    /// same as AccessFile but with a mode parameter.
    ///
    /// Binded with fsPromise.access(path[,mode]) in javascript.
    AccessFileWithMode((String, JSObject<JSPromise>, u8)),
    /// Commands the file creation in write only mode like Path::create does.
    /// This action is currently used when JS calls a `fs.createWriteStream`.
    ///
    /// Note: Look at fs_write_stream file for further documentation.
    CreateWSFile(String, Arc<Mutex<WSFile>>),
    /// Open a file (Filename/path, Promise Object)
    OpenFile((String, JSObject<JSPromise>)),
    /// Contains a setTimeout call callback. (Callback, Duration to sleep,
    /// Cancel trigger Receiver)
    SetTimeout(TimeoutAction),
    /// Write a String in a WriteStream file
    WriteInWSFile(Arc<Mutex<WSFile>>, String, Arc<AtomicU32>),
    /// Close a WriteStream file. Result of the javascript call of
    /// `writer.close()`
    CloseWSFile(
        Arc<Mutex<WSFile>>,
        Arc<std::sync::Mutex<WriteStreamCallbacks>>,
        JSContext,
        Arc<AtomicU32>,
    ),
    /// Stop the loop
    Stop(Sender<()>),
}

// The event loop of needjs differ in sens that we don't manage tics. But we let
// the tokio library define which is going to be resolved or not.

// The running loop handle each action that will grow quickly and we have to put
// a serious effort on making it readable. Let's make some development rules.

// 1. An action sended by a file should contains the way to solve it. Example:
// SetTimeout is sended from timeout_api.rs, which contains exec_timeout that is
// the function that resolve the timeout calling the callback, etc.
// 2. Use some macro to reduce the code repetition. It should be
// the `hold` variable in any case.

// The macro to use is `deff!` (shortcut of deffered). The role of the deffered
// action is to run something in background, adding a pending action to wait
// before stopping the executable, and synchronize just before the execution.

// There can't a instant where the event loop is running out and
// something is running in background. That's why, if the action is managed
// after the stop status has been declared, the action can be dismissed.
//
// Lets look at a dismiss call scenario:
// 1. The event loop has received the signal to stop, but also, there is pending
// actions like a timeout. The event loop will continue to receive action during
// the pending resolution.
// 2. For a instant, in a very small amount of time called epsilon, we entred
// into a configuration where there is no more pending actions, and the latest
// executor of the latest action have the time to signal that there is nothing
// more to do.
// 3. But, in fact, there is something more to do. However, it's too late, the
// action need to dismiss.

macro_rules! deff {
    ($e: expr, $d: expr) => {{
        loop {
            let count = PENDING_COUNTER.load(Ordering::SeqCst);
            if count > 0
                && PENDING_COUNTER
                    .compare_exchange(count, count + 1, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
            {
                $e.await;
                let prev = PENDING_COUNTER.fetch_sub(1, Ordering::SeqCst);
                if prev == 2
                    && STATUS.load(Ordering::SeqCst) == 1
                    && SYNC_ASYNC_BALANCE.load(Ordering::SeqCst) == 1
                    && PENDING_COUNTER
                        .compare_exchange(1, 0, Ordering::SeqCst, Ordering::Acquire)
                        .is_ok()
                {
                    STATUS.swap(2, Ordering::Acquire);
                    STATUS_NOTIFIER.notify_one();
                }

                break;
            } else if count == 0 {
                $d;
                break;
            }
            // else: count has been incremented in another thread
            // --> retry.
        }
        SYNC_ASYNC_BALANCE.fetch_sub(1, Ordering::SeqCst);
        STATUS_NOTIFIER.notify_one();
    }};

    ($e: expr) => {
        deff!($e, ())
    };
}
/// Number of pending actions handled by the event loop.
/// 0: going to stop, 1..N: number of pending actions in background + 1
static PENDING_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Event loop status.
/// 0: running, 1: stop requested, 2: going to stop
static STATUS: AtomicU32 = AtomicU32::new(0);

/// Notify that the status might have been updated.
static STATUS_NOTIFIER: Notify = Notify::const_new();

/// Balance between synchronous notification and asynchronous handling by the
/// event loop.
static SYNC_ASYNC_BALANCE: AtomicU32 = AtomicU32::new(0);

/// The following mutext is supposed to constain the execution
/// of the actions to be synchrone at the resolution.
pub fn get_hold() -> &'static Mutex<()> {
    maybe_static!(Mutex::<()>)
}

async fn running_loop(mut receiver: UnboundedReceiver<Action>) {
    while let Some(action) = receiver.recv().await {
        tokio::spawn(async move {
            // resolution.
            match action {
                Action::AccessFile(a) => deff!(exec_access(a)),
                Action::AccessFileWithMode(a) => deff!(exec_access_with_mode(a)),
                Action::CloseWSFile(file, callbacks, context, pending) => {
                    deff!(exec_close(file, callbacks, context, pending))
                }
                Action::CreateWSFile(path, ws_file) => deff!(exec_create_file(path, ws_file)),
                Action::OpenFile(a) => deff!(exec_open(a)),
                Action::SetTimeout(a) => {
                    deff!(
                        exec_timeout(a),
                        // count == 0 signify that the event loop will
                        // shutdown very soon. We can suspect that the
                        // global contexts instanciated will also being
                        // cleared soon. So, we can have a race condition
                        // between that cleaning and the drop of the
                        // protected value. Leaking the value solve a part
                        // of the problem. (rusty_jsc could give an unsafe
                        // unprotect method later.)
                        std::mem::forget(a.callback)
                    )
                }
                Action::WriteInWSFile(ws_file, value, pending) => {
                    deff!(exec_write_str(ws_file, value, pending))
                }
                Action::Stop(sender) => exec_stop(sender),
            }
        });
    }
}

/// Require the event loop to stop. It is called for the first time after the
/// evaluation of all the given Javascript (including required files). In
/// classical nodejs implementation, I would say that it is called after the
/// first `tick`.
pub fn exec_stop(sender: Sender<()>) {
    STATUS.swap(1, Ordering::SeqCst);
    SYNC_ASYNC_BALANCE.fetch_sub(1, Ordering::SeqCst);
    tokio::spawn(async move {
        // Note: there is maybe an issue here if tokio spaw the
        // end before all the other futures in a small script.
        if SYNC_ASYNC_BALANCE.load(Ordering::SeqCst) > 0 {
            STATUS_NOTIFIER.notified().await;
        } else {
            if PENDING_COUNTER
                .compare_exchange(1, 0, Ordering::SeqCst, Ordering::Acquire)
                .is_err()
            {
                STATUS_NOTIFIER.notified().await;
            }
            if PENDING_COUNTER.load(Ordering::SeqCst) == 0 {
                sender.send(()).unwrap();
                return;
            }
        }
        // Retry, it's like using a loop but a loop would give
        // an higher priority to the current thread. Sending a
        // new action increase the probability to retry only
        // when all other pending actions have been finished.
        append(Action::Stop(sender))
    });
}

pub fn append(action: Action) {
    SYNC_ASYNC_BALANCE.fetch_add(1, Ordering::SeqCst);
    let sender = maybe_static!(UnboundedSender::<Action>, || {
        let (sender, receiver) = mpsc::unbounded_channel::<Action>();
        tokio::spawn(running_loop(receiver));
        sender
    });
    let _ = sender.send(action);
}
