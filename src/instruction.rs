use core::fmt;

use colored::Colorize;

pub type Value = i64;

pub enum Instruction {
    Push(Value),
    Pop,
    Dup,
    Swap,
    Jz(Value), 
    Jnz(Value),
    Jmp(Value),
    Add,
    Sub,
    Mul,
    Div,
    Exit,
    Printout,
}

impl Instruction {
    pub fn mnemonic(&self) -> &str {
        match self {
            Self::Push(_) => "PUSH",
            Self::Pop => "POP",
            Self::Dup => "DUP",
            Self::Swap => "SWAP",
            Self::Jz(_) => "JZ",
            Self::Jnz(_) => "JNZ",
            Self::Jmp(_) => "JMP",
            Self::Add => "ADD",
            Self::Sub => "SUB",
            Self::Mul => "MUL",
            Self::Div => "DIV",
            Self::Exit => "EXIT",
            Self::Printout => "PRINTOUT"
        }
    }
    
    pub fn set_arg(&mut self, arg: Value) {
        match self {
            Self::Push(a) | Self::Jmp(a) => *a = arg,
            _ => ()
        }
    }
}

impl<'a> fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mnemonic = self.mnemonic().bold().magenta();
        match self {
            Self::Push(arg) | Self::Jmp(arg) => write!(f, "{:<10}{}", mnemonic, arg),
            _ => write!(f, "{}", mnemonic)
        }
    }
}
