use std::{collections::HashMap, fs::File, error::Error};

use colored::Colorize;

use crate::{instruction::Instruction, debug_info::DebugInfo};

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

    debug_info: DebugInfo
}

impl AsmParser {
    pub fn new(filepath: String) -> Self {
        Self {
            filepath,
            lineno: 0,
            labels: HashMap::new(),
            relocs: HashMap::new(),
            debug_info: DebugInfo::default()
        }
    }

    pub fn assemble(&mut self) -> ParseResult<Vec<Instruction>> {
        let file = File::open(self.filepath.clone())?;
        let lines = BufReader::new(file).lines().enumerate();
        let mut instructions = vec![];

        for (lineno, line) in lines {
            self.lineno = lineno + 1;
            if let Some(instruction) = self.parse_line(&line?, &mut instructions)? {
                instructions.extend(instruction);
            }
        }

        self.relocs.is_empty()
            .then_some(instructions)
            .ok_or_else(|| self.parse_error(format!("could not resolve labels {:?}", self.relocs)))
    }

    pub fn debug_info(self) -> DebugInfo {
        self.debug_info
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
            if self.relocs.get_mut(&label).map(|rel| rel.push(instruction_addr)).is_none() {
                self.relocs.insert(label, vec![instruction_addr]);
            }
            &0
        })
    }

    fn parse_instruction(&mut self, mnemonic: &str, arg: Option<String>, instruction_addr: i64) -> ParseResult<Instruction> {
        use Instruction as I;

        let arg = arg.map(|arg| arg.parse::<Value>().unwrap_or_else(|_| self.label_addr(arg.to_string(), instruction_addr)));
        match mnemonic {
            "PUSH" => arg.map(I::Push).ok_or(self.parse_error("`PUSH` expects one argument".to_string())),
            "POP" => Ok(I::Pop),
            "DUP" => Ok(I::Dup),
            "SWAP" => Ok(I::Swap),
            "JZ" => Ok(I::Jz),
            "JNZ" => Ok(I::Jnz),
            "JMP" => Ok(I::Jmp),
            "CALL" => Ok(I::Call),
            "ADD" => Ok(I::Add),
            "SUB" => Ok(I::Sub),
            "MUL" => Ok(I::Mul),
            "DIV" => Ok(I::Div),
            "EXIT" => Ok(I::Exit),
            "PRINTOUT" => Ok(I::Printout),
            "PRINTSTR" => Ok(I::Printstr),
            _ => Err(self.parse_error(format!("no such mnemonic `{}`", mnemonic)))
        }
    }

    fn escape_code(&self, code: char) -> ParseResult<char> {
        match code {
            'n' => Ok('\n'),
            't' => Ok('\t'),
            '0' => Ok('\0'),
            '\\' => Ok('\\'),
            _ => Err(self.parse_error(format!("unknown escape character `\\{}` in string literal", code)))
        }
    }

    fn parse_str_lit(&self, arg: String) -> ParseResult<Vec<char>> {
        if !arg.starts_with('"') || !arg.ends_with('"') {
            return Err(self.parse_error(format!("expect argument `{}` to be a string literal", arg)))
        }

        let mut char_vec = Vec::with_capacity(arg.len() - 2);
        let mut chars = arg.chars();
        chars.next();
        chars.next_back();
        while let Some(c) = chars.next() {
            char_vec.push(if c == '\\' { self.escape_code(chars.next().unwrap())? } else { c })
        }
        char_vec.push('\0');

        Ok(char_vec)
    }

    fn parse_metainstruction(&mut self, mnemonic: &str, arg: Option<String>, instruction_addr: i64) -> ParseResult<Vec<Instruction>> {
        use Instruction as I;

        match mnemonic {
            "PushStr" if arg.is_some() => Ok(
                self.parse_str_lit(arg.unwrap())?
                    .into_iter()
                    .rev()
                    .map(|c| I::Push(c as i64))
                    .collect()
                ),
            "Break" if arg.is_none() => {
                self.debug_info.add_breakpoint(instruction_addr);
                Ok(vec![])
            }
            _ => Err(self.parse_error(format!("no such metainstruction `{}`", mnemonic)))
        }
    }

    fn parse_line(&mut self, line: &str, instructions: &mut Vec<Instruction>) -> ParseResult<Option<Vec<Instruction>>> {
        let line = line.trim();
        if line.starts_with(';') || line.is_empty() {
            return Ok(None)
        }

        let mut attribs = line
            .split_whitespace()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        let mnemonic = attribs.next().unwrap();

        let mut arg = attribs.next().map(String::from);
        if let Some(comment) = &arg && comment.starts_with(';') {
            arg = None;
        }
        else if let Some(str_lit) = &mut arg && str_lit.starts_with('"') && !str_lit.ends_with('"') {
            for next in attribs {
                str_lit.push(' ');
                str_lit.push_str(next);
                if next.ends_with('"') {
                    break;
                }
            }
        }
        else if let Some(next) = attribs.next() && !next.starts_with(';') {
            return Err(self.parse_error(format!("too many arguments: `{}`", next)))
        }

        let instruction_addr = instructions.len() as i64;

        // label detected
        if let Some(label) = mnemonic.strip_suffix(':') {
            if let Some(rel) = self.relocs.remove(&label.to_string()) { 
                rel.iter().for_each(|ri| 
                    if let Some(i) = instructions.get_mut(*ri as usize) {
                        i.set_arg(instruction_addr)
                    }
                )
            }

            self.labels.insert(label.to_string(), instruction_addr);
            self.debug_info.add_label(instruction_addr, label.to_string());
            Ok(None)
        }
        else if let Some(meta) = mnemonic.strip_prefix('@'){
            self.parse_metainstruction(meta, arg, instruction_addr).map(Some)
        }
        else {
            self.parse_instruction(mnemonic, arg, instruction_addr).map(|i| Some(vec![i]))
        }
    }
}
