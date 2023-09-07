#![feature(let_chains)]

mod stack_machine;

use stack_machine::*;

fn main() {
    let mut machine = StackMachine::new();

    let instructions = [
        Instruction::new(Mnemonic::PUSH, &[4]),
        Instruction::new(Mnemonic::PUSH, &[5]),
        Instruction::new(Mnemonic::ADD, &[]),
        Instruction::new(Mnemonic::DUP, &[]),
        Instruction::new(Mnemonic::MUL, &[]),
        Instruction::new(Mnemonic::PRINTOUT, &[]),
        Instruction::new(Mnemonic::EXIT, &[]),
    ];

    let exit_code = machine.run(&instructions);
    println!("[simulation exited with code {}]", exit_code);
}
