mod core;
mod greeting;

use core::{dispatch, RequestContext};
use greeting::{Greet, GreetingModule};

fn main() {
    let context = RequestContext {
        correlation_id: "demo-request".into(),
    };
    match dispatch(
        &GreetingModule,
        Greet {
            name: "World".into(),
        },
        &context,
    ) {
        Ok(result) => println!("{} [{}]", result.message, result.correlation_id),
        Err(error) => {
            eprintln!("{}: {}", error.code, error.message);
            std::process::exit(1);
        }
    }
}
