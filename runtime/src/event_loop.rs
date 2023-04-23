use maybe_static::maybe_static;
use rusty_jsc::{JSObject, JSPromise, JSValue};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub enum Action {
    OpenFile((String, JSObject<JSPromise>)),
}

async fn running_loop(mut receiver: UnboundedReceiver<Action>) {
    loop {
        let action = match receiver.recv().await {
            Some(action) => action,
            None => break,
        };
        match action {
            Action::OpenFile((filename, promise)) => {
                let value = tokio::fs::read(filename).await.expect("file not found");
                let context = promise.context();
                println!("call resolve");
                promise.resolve(&[
                    JSValue::string(&context, String::from_utf8(value).unwrap()).unwrap()
                ]);
            }
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
