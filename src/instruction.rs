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

    pub fn id(&self) -> u16 {
        match self {
            Self::Push(_) => 0,
            Self::Pop => 1,
            Self::Dup => 2,
            Self::Swap => 3,
            Self::Jz(_) => 4,
            Self::Jnz(_) => 5,
            Self::Jmp(_) => 6,
            Self::Add => 7,
            Self::Sub => 8,
            Self::Mul => 9,
            Self::Div => 10,
            Self::Exit => 11,
            Self::Printout => 12
        }
    }
    
    pub fn set_arg(&mut self, arg: Value) {
        match self {
            Self::Push(a) | Self::Jmp(a) => *a = arg,
            _ => ()
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Push(arg) | 
            Self::Jz(arg) | 
            Self::Jnz(arg) | 
            Self::Jmp(arg) => {
                [&self.id().to_le_bytes().as_slice(), arg.to_le_bytes().as_slice()].concat()
            }
            _ => self.id().to_le_bytes().to_vec()
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mnemonic = self.mnemonic().bold().magenta();
        match self {
            Self::Push(arg) | Self::Jmp(arg) => write!(f, "{:<10}{}", mnemonic, arg),
            _ => write!(f, "{}", mnemonic)
        }
    }
}
