use std::env;

mod client;
mod server;

fn parsechar(s: &str) -> char {
    s.parse().unwrap_or_else(|_| info())
}

fn info() -> ! {
    println!("Start client: cargro run c\nStart server: cargo run s");
    std::process::exit(1)
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 1 {
        let input: char = parsechar(&args[0]);
        if input == 'c' {
            client::start();
        } else if input == 's' {
            server::start();
        } else {
            info();
        }
    } else {
        info();
    }
}
