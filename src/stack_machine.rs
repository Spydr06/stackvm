use std::{error::Error, io::Write};

use colored::Colorize;
use crate::{instruction::*, debug_info::DebugInfo};

pub type ExecResult<T> = Result<T, ExecError>;

#[derive(Debug)]
pub struct ExecError {
    addr: usize,
    err: String
}

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (@{}): {}", "Panic:".bold().red(), format!("{:04x}", self.addr).blue(), self.err)
    }
}

impl Error for ExecError {}

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
    debug_info: DebugInfo,
}

impl StackMachine {
    pub fn new(debug_info: DebugInfo) -> Self {
        let term_size = termsize::get().unwrap_or(termsize::Size { rows: 25, cols: 80 });
        Self {
            instruction_ptr: 0usize,
            stack: vec![],
            exited: None,
            term_width: term_size.cols,
            debug_info
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
            print!(
                "{:<5}{} {}",
                format!("{:04x}", addr).blue(),
                ip_marker,
                instruction
            );

            if let Some(label) = self.debug_info.label_at(addr as i64) {
                print!("\t{}", format!("; {}", label).bright_black())
            }

            println!()
        }

        print_header("Stack", self.term_width as usize);

        if self.stack.is_empty() {
            println!("{}\n", "<no entries>".bright_black())
        }
        for (addr, value) in self.stack.iter().enumerate() {
            print!(
                "{:<6}{}",
                format!("{:04x}", addr).blue(),
                format!("{}", value).red()
            );

            let ch = *value as u8 as char;
            if *value >= 0 && *value <= u8::MAX as i64 && is_printable(ch) {
                print!("\t{:?}", ch)
            }

            println!()
        }
    }

    pub fn run(&mut self, instructions: &[Instruction]) -> ExecResult<i32> {
        if self.debug_info.verbose() {
            self.disassembly(instructions);
        }

        while self.exited.is_none() && self.instruction_ptr < instructions.len() {
            self.eval(&instructions[self.instruction_ptr], instructions)?;
        }

        self.exited.ok_or_else(|| self.panic("no instruction left".to_string()))
    }

    fn panic(&mut self, err: String) -> ExecError {
        self.exited = Some(255);
        ExecError {
            addr: self.instruction_ptr,
            err
        }
    }

    fn pop_stack(&mut self, mnemonic: &str) -> ExecResult<Value> {
        self.stack.pop().ok_or_else(|| self.panic(format!("not enough values on stack for `{}`", mnemonic)))
    }

    fn bin_op(&mut self, op: &Instruction) -> ExecResult<()> {
        let a = self.pop_stack(op.mnemonic())?;
        let b = self.pop_stack(op.mnemonic())?;

        use Instruction as I;
        let result = match op {
            I::Add => a + b,
            I::Sub => a - b,
            I::Mul => a * b,
            I::Div => a / b,
            _ => return Err(self.panic("unreachable".to_string())),
        };

        self.stack.push(result);
        self.instruction_ptr += 1;

        Ok(())
    }

    pub fn handle_breakpoint(&mut self, instructions: &[Instruction]) -> ExecResult<()> {
        self.disassembly(instructions);
        println!();

        loop {
            print!("{} continue? [Y/n] ", "Breakpoint:".bold().cyan());
            let _ = std::io::stdout().flush();

            let mut buffer = String::new();
            let _ = std::io::stdin().read_line(&mut buffer);
            match buffer.trim().to_uppercase().as_str() {
                "Y" | "" => {
                    return Ok(())
                }
                "N" => {
                    return Err(self.panic("execution aborted at breakpoint".to_string()))
                }
                _ => {}
            }
        }
    }

    pub fn eval(&mut self, instruction: &Instruction, instructions: &[Instruction]) -> ExecResult<()> {
        // println!("{}: {:?}", instruction.mnemonic(), self.stack);

        if self.debug_info.breakpoint_at(self.instruction_ptr as i64) {
            self.handle_breakpoint(instructions)?
        }

        use Instruction as I;
        match instruction {
            I::Push(arg) => {
                self.stack.push(*arg);
                self.instruction_ptr += 1;
            }
            I::Pop => {
                let _ = self.pop_stack("POP")?;
                self.instruction_ptr += 1;
            }
            I::Add | I::Sub | I::Mul | I::Div => self.bin_op(instruction)?,
            I::Dup => {
                let value = self.pop_stack("DUP")?;
                self.stack.push(value);
                self.stack.push(value);
                self.instruction_ptr += 1;
            }
            I::Swap => {
                let a = self.pop_stack("SWAP")?;
                let b = self.pop_stack("SWAP")?;
                self.stack.push(a);
                self.stack.push(b);
                self.instruction_ptr += 1;
            }
            I::Jz => {
                let addr = self.pop_stack("JZ")?;
                let value = self.pop_stack("JZ")?;
                if value == 0 {
                    self.instruction_ptr = addr as usize;
                }
                else {
                    self.instruction_ptr += 1;
                }
            }
            I::Jnz => {
                let addr = self.pop_stack("JNZ")?;
                let value = self.pop_stack("JZ")?;
                if value != 0 {
                    self.instruction_ptr = addr as usize;
                }
                else {
                    self.instruction_ptr += 1;
                }
            }
            I::Jmp => self.instruction_ptr = self.pop_stack("JMP")? as usize,
            I::Call => {
                let addr = self.pop_stack("CALL")?;
                self.stack.push(self.instruction_ptr as Value + 1);
                self.instruction_ptr = addr as usize;
            }
            I::Printout => {
                println!("{}", self.pop_stack("PRINTOUT")?);
                self.instruction_ptr += 1;
            }
            I::Printstr => {
                while let ch = self.pop_stack("PRINTSTR")? && ch != 0 {
                    print!("{}", ch as u8 as char);
                }
                self.instruction_ptr += 1;
            }
            I::Exit => {
                let exit_code = self.stack.pop();
                self.exited = Some(exit_code.unwrap_or(0) as i32);
                self.instruction_ptr += 1;
            }
        }

        Ok(())
    }
}

fn is_printable(ch: char) -> bool {
    ch.is_ascii_digit() || ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch == '!' || ch == '\"' || ch == '#' || ch == '$' || ch == '%' || ch == '&' || ch == '\'' || ch == '(' || ch == ')' || ch == '*' || ch == '+' || ch == ',' || ch == '-' || ch == '.' || ch == '/' || ch == ':' || ch == ';' || ch == '<' || ch == '=' || ch == '>' || ch == '?' || ch == '@' || ch == '[' || ch == '\\' || ch == ']' || ch == '^' || ch == '`' || ch == '{' || ch == '|' || ch == '}'
}