use rusty_jsc::JSContext;
use std::fs::read_to_string;

mod console;
mod event_loop;
mod fs;
mod modules;

#[tokio::main]
async fn main() {
    let mut context = JSContext::default();

    modules::init(&mut context);

    let default_index = String::from("./index.js");
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).unwrap_or(&default_index);
    println!("open {filename}");
    let script = read_to_string(filename).expect("input file not found");
    match context.evaluate_script(&script, 1) {
        Err(err) => println!("{}", err.to_string(&context)),
        _ => println!("success"),
    }
}
