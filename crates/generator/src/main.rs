// Temporary until main.rs drives the generator (Task 5); the parser is only
// reached from tests until then.
#![allow(dead_code)]

mod classify;
mod emit;
mod spec;

fn main() {
    eprintln!("usage: cargo xtask codegen [--check]");
    std::process::exit(2);
}
