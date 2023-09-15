#![feature(let_chains)]

mod stack_machine;
mod assembler;
mod instruction;

use stack_machine::*;
use assembler::*;

fn die(err: impl std::fmt::Display) -> ! {
    eprintln!("{}", err);
    std::process::exit(1);
}

fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        die(format!("Usage: {} <source file>", args.next().unwrap_or("stasm".to_string())))
    }

    let mut parser = AsmParser::new(args.nth(1).unwrap());
    let instructions = parser.assemble();
    if let Err(err) = instructions {
        die(err);
    }

    let mut machine = StackMachine::new();
    match machine.run(&instructions.unwrap()) {
        Ok(exit_code) => println!("[simulation exited with code {}]", exit_code),
        Err(err) => die(err)
    }
}
