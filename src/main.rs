#![feature(let_chains)]

mod assembler;
mod binary;
mod debug_info;
mod instruction;
mod stack_machine;

use assembler::*;
use stack_machine::*;

use crate::{binary::Binary, debug_info::DebugInfo};

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    filepath: String,

    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    run: bool,
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    assemble: bool,
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    verbose: bool,

    #[arg(short)]
    output_filepath: Option<String>
}

fn main() {
    use clap::Parser;
    let args = Cli::parse();

    let instructions;
    let mut debug_info;
    if args.assemble {
        let mut parser = AsmParser::new(args.filepath);
        match parser.assemble() {
            Err(err) => die(err),
            Ok(inst) => instructions = inst
        }
        debug_info = parser.debug_info();
    }
    else {
        match Binary::load_from(args.filepath) {
            Err(err) => die(err),
            Ok(binary) => instructions = binary.instructions()
        }
        debug_info = DebugInfo::default();
    }

    debug_info.set_verbose(args.verbose);

    if args.run {
        let mut machine = StackMachine::new(debug_info);
        match machine.run(&instructions) {
            Ok(exit_code) => println!("[simulation exited with code {}]", exit_code),
            Err(err) => die(err)
        }
    }
    else if let Some(filepath) = args.output_filepath &&
            let Err(err) = Binary::from_instructions(instructions).save_to(filepath) {
        die(err);
    }
}

fn die(err: impl std::fmt::Display) -> ! {
    eprintln!("{}", err);
    std::process::exit(1);
}
