use core::fmt;

use colored::Colorize;

pub type Value = i64;

pub enum Instruction {
    Push(Value),
    Pop,
    Dup,
    Swap,
    Jz, 
    Jnz,
    Jmp,
    Call,
    Add,
    Sub,
    Mul,
    Div,
    Exit,
    Printout,
    Printstr,
}

impl Instruction {
    pub fn mnemonic(&self) -> &str {
        match self {
            Self::Push(_) => "PUSH",
            Self::Pop => "POP",
            Self::Dup => "DUP",
            Self::Swap => "SWAP",
            Self::Jz => "JZ",
            Self::Jnz => "JNZ",
            Self::Jmp => "JMP",
            Self::Call => "CALL",
            Self::Add => "ADD",
            Self::Sub => "SUB",
            Self::Mul => "MUL",
            Self::Div => "DIV",
            Self::Exit => "EXIT",
            Self::Printout => "PRINTOUT",
            Self::Printstr => "PRINTSTR",
        }
    }

    pub fn id(&self) -> u16 {
        match self {
            Self::Push(_) => 0,
            Self::Pop => 1,
            Self::Dup => 2,
            Self::Swap => 3,
            Self::Jz => 4,
            Self::Jnz => 5,
            Self::Jmp => 6,
            Self::Call => 13,
            Self::Add => 7,
            Self::Sub => 8,
            Self::Mul => 9,
            Self::Div => 10,
            Self::Exit => 11,
            Self::Printout => 12,
            Self::Printstr => 14,
        }
    }
    
    pub fn set_arg(&mut self, arg: Value) {
        if let Self::Push(a) = self {
            *a = arg
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Push(arg) => [(self.id().to_le_bytes().as_slice()), arg.to_le_bytes().as_slice()].concat(),
            _ => self.id().to_le_bytes().to_vec()
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mnemonic = self.mnemonic().bold().magenta();
        match self {
            Self::Push(arg) => write!(f, "{:<10}{}", mnemonic, arg),
            _ => write!(f, "{}", mnemonic)
        }
    }
}
