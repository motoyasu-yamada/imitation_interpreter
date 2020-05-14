mod lexer;
mod repl;
mod token;

use std::io;

fn main() {
    println!("Hello, world!");
    println!("Hello! This is the Monkey programming language!");
    println!("Feel free to type in commands");
    println!("");
    repl::start(io::stdin(), io::stdout());
}
