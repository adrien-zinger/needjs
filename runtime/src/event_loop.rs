use std::{
    sync::atomic::{AtomicU32, AtomicUsize, Ordering},
    time::Duration,
};

use atomic_wait::{wait, wake_one};
use maybe_static::maybe_static;
use rusty_jsc::{JSObject, JSPromise, JSProtected};
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    oneshot::Sender,
    Mutex,
};

use crate::{fs_promise::*, timeout_api::exec_timeout};

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
    AccessFileWithMode((String, u8, JSObject<JSPromise>)),
    /// Contains a setTimeout call callback.
    SetTimeout((JSObject<JSProtected>, Duration)),

    /// Stop the loop
    Stop(Sender<()>),
}

async fn running_loop(mut receiver: UnboundedReceiver<Action>) {
    // the following mutext is supposed to constain the execution
    // of the actions to be synchrone at the resolution.
    let hold = Mutex::new(());

    // 0: going to stop, 1..N: number of pending actions in background + 1
    let pending_counter = AtomicUsize::new(1);
    // 0: running, 1: stop requested, 2: going to stop
    let status = AtomicU32::new(0);

    while let Some(action) = receiver.recv().await {
        // resolution.
        match action {
            Action::OpenFile(a) => {
                let _ = hold.lock().await;
                exec_open(a).await;
            }
            Action::AccessFile(a) => {
                let _ = hold.lock().await;
                exec_access(a).await;
            }
            Action::SetTimeout(a) => loop {
                let count = pending_counter.load(Ordering::SeqCst);
                if count > 0
                    && pending_counter
                        .compare_exchange(count, count + 1, Ordering::SeqCst, Ordering::SeqCst)
                        .is_ok()
                {
                    exec_timeout(a, &hold).await;
                    let prev = pending_counter.fetch_sub(1, Ordering::SeqCst);
                    if prev == 2
                        && status.load(Ordering::SeqCst) == 1
                        && pending_counter
                            .compare_exchange(1, 0, Ordering::SeqCst, Ordering::Acquire)
                            .is_ok()
                    {
                        status.swap(2, Ordering::Acquire);
                        wake_one(&status);
                    }
                    break;
                } else if count == 0 {
                    // count == 0 signify that the event loop will shutdown very soon.
                    // We can suspect that the global contexts instanciated will also
                    // being cleared soon. So, we can have a race condition between
                    // that cleaning and the drop of the protected value. Leaking the
                    // value solve a part of the problem. (rusty_jsc could give an unsafe
                    // unprotect method later.)
                    std::mem::forget(a.0);
                    break;
                } // else: count has been incremented in another thread --> retry.
            },
            Action::Stop(sender) => {
                let _ = hold.lock().await; // wait the end of synchronized actions
                status.swap(1, Ordering::SeqCst);
                loop {
                    if pending_counter
                        .compare_exchange(1, 0, Ordering::SeqCst, Ordering::Acquire)
                        .is_err()
                    {
                        wait(&status, 1);
                    }
                    if pending_counter.load(Ordering::SeqCst) == 0 {
                        sender.send(()).unwrap();
                        return;
                    }
                }
            }
            _ => todo!(),
        }
    }
}

pub fn append(action: Action) {
    let sender = maybe_static!(UnboundedSender::<Action>, || {
        let (sender, receiver) = mpsc::unbounded_channel::<Action>();
        tokio::spawn(running_loop(receiver));
        sender
    });
    let _ = sender.send(action);
}
