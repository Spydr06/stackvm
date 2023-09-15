use std::{collections::HashMap, fs::File, error::Error};

use colored::Colorize;

use crate::instruction::Instruction;

use crate::instruction::*;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub enum ParseError {
    Parse {
        err: String,
        file: String,
        lineno: usize,
    },
    Io(std::io::Error)
}

impl From<std::io::Error> for ParseError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse { err, file, lineno } => write!(f, 
                "{} {}:{}: {}", 
                "Parse Error:".bold().red(), 
                file.bold(),
                lineno.to_string().bold(),
                err
            ),
            Self::Io(err) => write!(f, "{} {}", "Io Error:".bold().red(), err),
        }
    }
}

impl Error for ParseError {}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct AsmParser {
    filepath: String,
    lineno: usize,

    labels: HashMap<String, i64>,
    relocs: HashMap<String, Vec<i64>>,
}

impl AsmParser {
    pub fn new(filepath: String) -> Self {
        Self {
            filepath,
            lineno: 0,
            labels: HashMap::new(),
            relocs: HashMap::new()
        }
    }

    pub fn assemble(&mut self) -> ParseResult<Vec<Instruction>> {
        let file = File::open(self.filepath.clone())?;
        let mut lines = BufReader::new(file).lines().enumerate();
        let mut instructions = vec![];

        while let Some((lineno, line)) = lines.next() {
            self.lineno = lineno + 1;
            let line = line?;
            if let Some(instruction) = self.parse_line(&line, &mut instructions)? {
                instructions.push(instruction);
            }
        }

        if !self.relocs.is_empty() {
            Err(self.parse_error(format!("could not resolve labels {:?}", self.relocs)))
        }
        else {
            Ok(instructions)
        }
    }

    fn parse_error(&self, err: String) -> ParseError {
        ParseError::Parse {
            err,
            file: self.filepath.clone(),
            lineno: self.lineno
        }
    }

    fn label_addr(&mut self, label: String, instruction_addr: i64) -> i64 {
        *self.labels.get(&label).unwrap_or_else(|| {
            if let Some(rel) = self.relocs.get_mut(&label) {
                rel.push(instruction_addr)
            }
            else {
                self.relocs.insert(label, vec![instruction_addr]);
            }
            &0
        })
    }

    fn parse_instruction(&mut self, mnemonic: &str, arg: Option<&str>, instruction_addr: i64) -> ParseResult<Instruction> {
        use Instruction as I;

        let arg = arg.map(|arg| arg.parse::<Value>().unwrap_or_else(|_| {
            self.label_addr(arg.to_string(), instruction_addr)
        }));
        match mnemonic {
            "PUSH" => arg.map(I::Push).ok_or(self.parse_error(format!("`PUSH` expects one argument"))),
            "POP" => Ok(I::Pop),
            "DUP" => Ok(I::Dup),
            "SWAP" => Ok(I::Swap),
            "JZ" => arg.map(I::Jz).ok_or(self.parse_error(format!("`JMP` expects one argument"))),
            "JNZ" => arg.map(I::Jnz).ok_or(self.parse_error(format!("`JMP` expects one argument"))),
            "JMP" => arg.map(I::Jmp).ok_or(self.parse_error(format!("`JMP` expects one argument"))),
            "ADD" => Ok(I::Add),
            "SUB" => Ok(I::Sub),
            "MUL" => Ok(I::Mul),
            "DIV" => Ok(I::Div),
            "EXIT" => Ok(I::Exit),
            "PRINTOUT" => Ok(I::Printout),
            _ => Err(self.parse_error(format!("no such mnemonic `{}`", mnemonic)))
        }
    }

    fn parse_line(&mut self, line: &String, instructions: &mut Vec<Instruction>) -> ParseResult<Option<Instruction>> {
        let line = line.trim();
        if line.starts_with(";") || line.is_empty() {
            return Ok(None)
        }

        let mut attribs = line
            .split_whitespace()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        let mnemonic = attribs.next().unwrap();

        let mut arg = attribs.next();
        if let Some(comment) = arg && comment.starts_with(";") {
            arg = None;
        }
        else if let Some(next) = attribs.next() && !next.starts_with(";") {
            return Err(self.parse_error(format!("too many arguments: `{}`", next)))
        }

        let instruction_addr = instructions.len() as i64;

        // label detected
        if mnemonic.ends_with(":") {
            let label = mnemonic[..mnemonic.len()-1].to_string();
            self.relocs.remove(&label).map(|rel|
                rel.iter().for_each(|ri| 
                    instructions.get_mut(*ri as usize).map(|i| 
                        i.set_arg(instruction_addr)
                    ).unwrap_or_default()
                )
            );

            self.labels.insert(label, instruction_addr);


            Ok(None)
        }
        else {
            self.parse_instruction(mnemonic, arg, instruction_addr).map(Some)
        }
    }
}