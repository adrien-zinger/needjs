use event_loop::get_hold;
use rusty_jsc::JSContext;
use std::fs::read_to_string;
use tokio::sync::oneshot::channel;

mod console;
mod event_loop;
mod fs;
mod fs_promise;
mod fs_write_stream;
mod modules;
mod timeout_api;
mod util;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let mut context = JSContext::default();

    modules::init(&mut context);

    let default_index = String::from("./index.js");
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).unwrap_or(&default_index);
    // TODO: improve all file not found errors
    let script = read_to_string(filename).expect("input file not found");

    {
        // block any asynchronous calls from event loop during main evaluation.
        let _ = get_hold().lock().await;
        if let Err(err) = context.evaluate_script(&script, 1) {
            println!("{}", err.to_js_string(&context).unwrap());
        }
    }

    let (sender, receiver) = channel();
    event_loop::append(event_loop::Action::Stop(sender));
    receiver.await.unwrap();
}
