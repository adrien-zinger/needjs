use std::sync::{
    atomic::{AtomicU32, AtomicUsize, Ordering},
    Arc,
};

use atomic_wait::{wait, wake_one};
use maybe_static::maybe_static;
use rusty_jsc::{JSObject, JSPromise};
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    oneshot::Sender,
    Mutex,
};

use crate::{
    fs_promise::*,
    timeout_api::{exec_timeout, TimeoutAction},
};

pub enum Action {
    /// Open a file (Filename/path, Promise Object)
    OpenFile((String, JSObject<JSPromise>)),
    /// Check file accessibility (Filename/path, Promise Object).
    ///
    /// Binded with fsPromise.access(path[,mode]) in javascript.
    AccessFile((String, JSObject<JSPromise>)),
    /// Check file accessibility (Filename/path, Promise Object). This is the
    /// same as AccessFile but with a mode parameter.
    ///
    /// Binded with fsPromise.access(path[,mode]) in javascript.
    AccessFileWithMode((String, JSObject<JSPromise>, u8)),
    /// Contains a setTimeout call callback. (Callback, Duration to sleep,
    /// Cancel trigger Receiver)
    SetTimeout(TimeoutAction),

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
// 2. Use some macro to reduce the code repetition. Two helpers are available,
// the first one is `sync`, that will lock the first identification given. It should be
// the `hold` variable in any case. Then, `sync` will await for the future given
// as second parameter.

#[allow(unused)]
macro_rules! sync {
    ($h: ident, $e: expr) => {{
        let _ = $h.lock().await;
        $e.await;
        SYNC_ASYNC_BALANCE.fetch_sub(1, Ordering::SeqCst);
    }};
}

// The second macro to use is deff, shortcut of deffered. The role of the
// deffered action is to run something in background, adding a pending action to
// wait before stopping the executable, and synchronize just before the
// execution.

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
    ($pending_counter: ident, $status: ident, $e: expr, $d: expr) => {{
        loop {
            let count = $pending_counter.load(Ordering::SeqCst);
            if count > 0
                && $pending_counter
                    .compare_exchange(count, count + 1, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
            {
                $e.await;
                let prev = $pending_counter.fetch_sub(1, Ordering::SeqCst);
                if prev == 2
                    && $status.load(Ordering::SeqCst) == 1
                    && SYNC_ASYNC_BALANCE.load(Ordering::SeqCst) == 1
                    && $pending_counter
                        .compare_exchange(1, 0, Ordering::SeqCst, Ordering::Acquire)
                        .is_ok()
                {
                    $status.swap(2, Ordering::Acquire);
                    wake_one(&*$status);
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
    }};

    ($pending_counter: ident, $status: ident, $e: expr) => {
        deff!($pending_counter, $status, $e, ())
    };
}

static SYNC_ASYNC_BALANCE: AtomicU32 = AtomicU32::new(0);

async fn running_loop(mut receiver: UnboundedReceiver<Action>) {
    // the following mutext is supposed to constain the execution
    // of the actions to be synchrone at the resolution.
    let hold = Arc::new(Mutex::new(()));

    // 0: going to stop, 1..N: number of pending actions in background + 1
    let pending_counter = Arc::new(AtomicUsize::new(1));
    // 0: running, 1: stop requested, 2: going to stop
    let status = Arc::new(AtomicU32::new(0));

    while let Some(action) = receiver.recv().await {
        let hold = hold.clone();
        let pending_counter = pending_counter.clone();
        let status = status.clone();
        tokio::spawn(async move {
            // resolution.
            match action {
                Action::OpenFile(a) => deff!(pending_counter, status, exec_open(a, &hold)),
                Action::AccessFile(a) => deff!(pending_counter, status, exec_access(a, &hold)),
                Action::AccessFileWithMode(a) => {
                    deff!(pending_counter, status, exec_access_with_mode(a, &hold))
                }
                Action::SetTimeout(a) => {
                    deff!(
                        pending_counter,
                        status,
                        exec_timeout(a, &hold),
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

                Action::Stop(sender) => {
                    status.swap(1, Ordering::SeqCst);
                    SYNC_ASYNC_BALANCE.fetch_sub(1, Ordering::SeqCst);
                    std::thread::spawn(move || loop {
                        // Note: there is maybe an issue here if tokio spaw the
                        // end before all the other futures in a small script.
                        if SYNC_ASYNC_BALANCE.load(Ordering::SeqCst) > 0 {
                            wait(&*status, 1);
                        } else {
                            if pending_counter
                                .compare_exchange(1, 0, Ordering::SeqCst, Ordering::Acquire)
                                .is_err()
                            {
                                wait(&*status, 1);
                            }
                            if pending_counter.load(Ordering::SeqCst) == 0 {
                                sender.send(()).unwrap();
                                return;
                            }
                        }
                    });
                }
            }
        });
    }
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
