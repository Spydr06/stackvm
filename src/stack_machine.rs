use colored::Colorize;
use core::fmt;

pub type Value = i64;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Mnemonic {
    PUSH,
    POP,
    DUP,
    ADD,
    SUB,
    MUL,
    DIV,
    EXIT,
    PRINTOUT,
}

impl Into<String> for Mnemonic {
    fn into(self) -> String {
        format!("{:?}", self)
    }
}

impl Mnemonic {
    pub fn to_string(self) -> String {
        self.into()
    }
}

pub struct Instruction<'a> {
    mnemonic: Mnemonic,
    args: &'a [Value],
}

impl<'a> Instruction<'a> {
    pub fn new(mnemonic: Mnemonic, args: &'a [Value]) -> Self {
        Self { mnemonic, args }
    }
}

impl<'a> fmt::Display for Instruction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<10}", self.mnemonic.to_string().bold().magenta())?;
        for arg in self.args {
            write!(f, "{}\t", arg)?;
        }
        Ok(())
    }
}

fn print_header(header: &str, width: usize) {
    println!();
    let padding_size = width / 2 - header.len() / 2 - 1;
    let padding = ":".repeat(padding_size);
    println!(
        "{}",
        format!("{} {} {}", padding, header, padding).bold().white()
    );
    println!();
}

pub struct StackMachine {
    instruction_ptr: usize,
    stack: Vec<Value>,

    exited: Option<i32>,

    // debugging
    term_width: u16,
}

impl StackMachine {
    pub fn new() -> Self {
        let term_size = termsize::get().unwrap_or(termsize::Size { rows: 25, cols: 80 });
        Self {
            instruction_ptr: 0usize,
            stack: vec![],
            exited: None,
            term_width: term_size.cols,
        }
    }

    fn disassembly(&self, instructions: &[Instruction]) {
        print_header("Instructions", self.term_width as usize);
        for (addr, instruction) in instructions.iter().enumerate() {
            let ip_marker = if addr == self.instruction_ptr {
                ">>"
            } else {
                "  "
            }
            .green();
            println!(
                "{:<5}{} {}",
                format!("{:04x}", addr).blue(),
                ip_marker,
                instruction
            );
        }

        print_header("Stack", self.term_width as usize);

        if self.stack.is_empty() {
            println!("{}\n", "<no entries>".bright_black())
        }
        for (addr, value) in self.stack.iter().enumerate() {
            println!(
                "{:<6}{}",
                format!("{:04x}", addr).blue(),
                format!("{}", value).red()
            )
        }
    }

    pub fn run(&mut self, instructions: &[Instruction]) -> i32 {
        self.disassembly(instructions);

        while self.exited.is_none() && self.instruction_ptr < instructions.len() {
            self.eval(&instructions[self.instruction_ptr]);
        }

        if let Some(exit_code) = self.exited {
            exit_code
        } else {
            println!(
                "{} no instruction at {}",
                "Panic:".bold().red(),
                format!("{:04x}", self.instruction_ptr).blue()
            );
            255
        }
    }

    fn panic(&mut self, reason: &str) {
        println!(
            "{} (@{:04x}) {}",
            "Panic:".bold().red(),
            self.instruction_ptr,
            reason
        );
        self.exited = Some(255)
    }

    fn binop(&mut self, op: Mnemonic) {
        if let Some(a) = self.stack.pop() && let Some(b) = self.stack.pop() {
            use Mnemonic as M;
            let result = match op {
                M::ADD => a + b,
                M::SUB => a - b,
                M::MUL => a * b,
                M::DIV => a / b,
                _ => {
                    self.panic("unreachable");
                    0
                }
            };

            self.stack.push(result);
        }
        else {
            self.panic(&format!("not enough values on stack for `{}`", op.to_string()));
        }
    }

    pub fn eval(&mut self, instruction: &Instruction) {
        use Mnemonic as M;
        match instruction.mnemonic {
            M::PUSH => {
                self.stack.push(*instruction.args.get(0).unwrap_or(&0));
            }
            M::POP => {
                let _ = self.stack.pop();
            }
            M::ADD | M::SUB | M::MUL | M::DIV => self.binop(instruction.mnemonic),
            M::DUP => {
                if let Some(value) = self.stack.pop() {
                    self.stack.push(value);
                    self.stack.push(value);
                } else {
                    self.panic("not enough values on stack for `DUP`");
                }
            }
            M::PRINTOUT => {
                let value = self.stack.pop();
                if value.is_none() {
                    self.panic("not enough values on stack for `PRINTOUT`");
                } else {
                    println!("{}", value.unwrap())
                }
            }
            M::EXIT => {
                let exit_code = self.stack.pop();
                self.exited = Some(exit_code.unwrap_or(0) as i32)
            }
        }

        self.instruction_ptr += 1;
    }
}
