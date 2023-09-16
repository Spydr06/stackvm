use std::{fs::File, io::{BufWriter, Write, Read}};

use crate::instruction::{Instruction, Value};

use colored::Colorize;

pub type SaveResult<T> = Result<T, std::io::Error>;
pub type LoadResult<T> = Result<T, LoadError>;

#[derive(Debug)]
pub enum LoadError {
    Load(String),
    Io(std::io::Error)
}

impl From<std::io::Error> for LoadError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(err) => write!(f, "{} {}", "Load Error:".bold().red(), err),
            Self::Io(err) => write!(f, "{} {}", "Io:".bold().red(), err)
        }  
    }
}

#[derive(Default)]
#[repr(C)]
struct Header {
    num_instructions: usize,
}

impl Header {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self as *const Self as *const u8, core::mem::size_of::<Header>())
        }
    }

    fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::mem::transmute((self, core::mem::size_of::<Header>()))
        }
    }
}

#[derive(Default)]
pub struct Binary {
    header: Header,
    instructions: Vec<Instruction>
}

const MAGIC: [u8; 5] = [
    b'.',
    b'S',
    b'P',
    b'V',
    b'M'
];

impl Binary {
    pub fn from_instructions(instructions: Vec<Instruction>) -> Binary {
        Binary { 
            header: Header {
                num_instructions: instructions.len()
            },
            instructions
        } 
    }

    pub fn instructions(self) -> Vec<Instruction> {
        self.instructions
    }

    pub fn load_from(filepath: String) -> LoadResult<Binary> {
        let mut binary = Binary::default();
        let mut file = File::open(filepath)?;

        let mut magic = [0; MAGIC.len()];
        file.read_exact(&mut magic)?;

        if magic != MAGIC {
            return Err(LoadError::Load("wrong file format".to_string()))
        }

        file.read_exact(binary.header.as_bytes_mut())?;
        binary.instructions.reserve_exact(binary.header.num_instructions);

        while binary.instructions.len() < binary.header.num_instructions {
            binary.instructions.push(read_instruction(&mut file)?);
        }

        Ok(binary)
    }

    pub fn save_to(self, filepath: String) -> SaveResult<()> {
        let file = File::create(filepath)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(&MAGIC)?;
        writer.write_all(self.header.as_bytes())?;

        for instruction in self.instructions {
            writer.write_all(&instruction.as_bytes())?;
        }

        writer.flush()?;

        Ok(())
    }
}

fn read_instruction(file: &mut File) -> LoadResult<Instruction> {
    fn read_arg(file: &mut File) -> LoadResult<Value> {
        let mut arg_bytes = [0; std::mem::size_of::<Value>()];
        file.read_exact(&mut arg_bytes)?;
        Ok(Value::from_le_bytes(arg_bytes))
    }
    
    let mut id_bytes = [0u8, 0];
    file.read_exact(&mut id_bytes)?;
    
    use Instruction as I;
    let mnemonic = u16::from_le_bytes(id_bytes);
    match mnemonic {
        0 => Ok(I::Push(read_arg(file)?)),
        1 => Ok(I::Pop),
        2 => Ok(I::Dup),
        3 => Ok(I::Swap),
        4 => Ok(I::Jz),
        5 => Ok(I::Jnz),
        6 => Ok(I::Jmp),
        7 => Ok(I::Add),
        8 => Ok(I::Sub),
        9 => Ok(I::Mul),
        10 => Ok(I::Div),
        11 => Ok(I::Exit),
        12 => Ok(I::Printout),
        13 => Ok(I::Call),
        14 => Ok(I::Printstr),
        _ => Err(LoadError::Load(format!("no such mnemonic `{}`", mnemonic)))
    }
}
