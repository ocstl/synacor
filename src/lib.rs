use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, Read, Write};

const MODULO: u16 = 32768_u16;
const MEMORY_SIZE: usize = 32768;
const NBR_REGISTERS: usize = 8;

/// Architecture specs (15-bit address space, 16-bit values, 8 registers, unlimited stack).
/// Programs are loaded starting at memory 0. Initial address of the instruction pointer.
///
/// Numbers are stored as a 16-bit little-endian pair (low byte, high byte).
///  * 0 to 32767 are literal values
///  * 32768 to 32775 are registers (0 to 7)
///  * 32776 and above are invalid.
pub struct VirtualMachine {
    memory: [u16; MEMORY_SIZE],
    registers: [u16; NBR_REGISTERS],
    stack: Vec<u16>,
    instruction_pointer: u16,
}

#[derive(Debug)]
pub enum VirtualMachineError {
    InstructionValueError,
    RegisterValueError,
    LiteralValueError,
    InvalidOpCode,
    HaltExecution,
    ReadFromEmptyStack,
    ReadError
}

enum Instruction {
    Register(u16),
    Value(u16),
}

impl Instruction {
    fn value(&self) -> u16 {
        match *self {
            Instruction::Register(x) => x,
            Instruction::Value(x) => x,
        }
    }

    fn register_value(&self) -> Result<u16, VirtualMachineError> {
        match *self {
            Instruction::Register(x) => Ok(x),
            Instruction::Value(_) => Err(VirtualMachineError::RegisterValueError),
        }
    }
}

impl TryFrom<u16> for Instruction {
    type Error = VirtualMachineError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0...32767 => Ok(Instruction::Value(value)),
            32768 ... 32775 => Ok(Instruction::Register(value % MODULO)),
            _ => Err(VirtualMachineError::InstructionValueError),
        }
    }
}

impl VirtualMachine {
    pub fn new() -> VirtualMachine {
        VirtualMachine{
            memory: [0; 32768],
            registers: [0; 8],
            stack: Vec::new(),
            instruction_pointer: 0_u16,
        }
    }
    
    pub fn load_program(&mut self, filename: &str) {
        let mut f = File::open(filename).unwrap();
        let mut buffer = Vec::new();

        f.read_to_end(&mut buffer).unwrap();
        // Load into memory one chunk at a time (little endian).
        for (i, v) in buffer.chunks(2).enumerate() {
            self.memory[i] = u16::from_le_bytes([v[0], v[1]]);
        }
    }

    pub fn execute_program(&mut self) -> Result<(), VirtualMachineError> {
        let mut result = Ok(());

        while let Ok(()) = result {
            result = match self.read_instruction()?.value() {
                0   => self.halt(),
                1   => self.set(),
                2   => self.push(),
                3   => self.pop(),
                4   => self.eq(),
                5   => self.gt(),
                6   => self.jmp(),
                7   => self.jt(),
                8   => self.jf(),
                9   => self.add(),
                10  => self.mult(),
                11  => self.rem(),
                12  => self.and(),
                13  => self.or(),
                14  => self.not(),
                15  => self.rmem(),
                16  => self.wmem(),
                17  => self.call(),
                18  => self.ret(),
                19  => self.write(),
                20  => self.read(),
                21  => Ok(()),
                _ => Err(VirtualMachineError::InvalidOpCode),
            };
        }

        result
    }

    fn read_instruction(&mut self) -> Result<Instruction, VirtualMachineError> {
        let instruction = Instruction::try_from(self.memory[self.instruction_pointer as usize]);
        self.instruction_pointer += 1;
        instruction
    }

    fn get_value(&self, i: Instruction) -> u16 {
        match i {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        }
    }

    // Halt execution (not really an error, but simpler this way.)
    fn halt(&mut self) -> Result<(), VirtualMachineError> {
        Err(VirtualMachineError::HaltExecution)
    }

    // Set register 'a' to the value 'b' (or value in register 'b').
    fn set(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction()?.register_value()?;
        let b = self.read_instruction().map(|i| self.get_value(i))?;

        self.registers[a as usize] = b;
        Ok(())
    }

    // Push value of register 'a' onto the stack.
    fn push(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction().map(|i| self.get_value(i))?;

        self.stack.push(a);
        Ok(())
    }

    // Pop the stack and write to register 'a'. Error out if the stack is empty.
    fn pop(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction()?.register_value()?;
        let b = self.stack.pop().ok_or(VirtualMachineError::ReadFromEmptyStack)?;

        self.registers[a as usize] = b;
        Ok(())
    }

    // Set register 'a' to 1 if 'b' == 'c'. 0 otherwise.
    fn eq(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction()?.register_value()?;
        let b = self.read_instruction().map(|i| self.get_value(i))?;
        let c = self.read_instruction().map(|i| self.get_value(i))?;

        self.registers[a as usize] = u16::from(b == c);
        Ok(())
    }

    // Set register 'a' to 1 if 'b' > 'c'. 0 otherwise.
    fn gt(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction()?.register_value()?;
        let b = self.read_instruction().map(|i| self.get_value(i))?;
        let c = self.read_instruction().map(|i| self.get_value(i))?;

        self.registers[a as usize] = u16::from(b > c);
        Ok(())
    }

    // Jump to instruction 'a' (value or register).
    fn jmp(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction().map(|i| self.get_value(i))?;

        self.instruction_pointer = a;
        Ok(())
    }

    // If 'a' != 0, jump to instruction 'b' (value or register).
    fn jt(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction().map(|i| self.get_value(i))?;
        let b = self.read_instruction().map(|i| self.get_value(i))?;

        if a != 0 { self.instruction_pointer = b }
        Ok(())
    }

    // If 'a' == 0, jump to instruction 'b' (value or register).
    fn jf(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction().map(|i| self.get_value(i))?;
        let b = self.read_instruction().map(|i| self.get_value(i))?;

        if a == 0 { self.instruction_pointer = b }
        Ok(())
    }

    // Assign into register 'a' the sum of 'b' and 'c' (modulo 32768).
    fn add(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction()?.register_value()?;
        let b = self.read_instruction().map(|i| self.get_value(i))?;
        let c = self.read_instruction().map(|i| self.get_value(i))?;

        self.registers[a as usize] = b.wrapping_add(c) % MODULO;
        Ok(())
    }

    // Assign into register 'a' the product of 'b' and 'c' (modulo 32768).
    fn mult(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction()?.register_value()?;
        let b = self.read_instruction().map(|i| self.get_value(i))?;
        let c = self.read_instruction().map(|i| self.get_value(i))?;

        self.registers[a as usize] = b.wrapping_mul(c) % MODULO;
        Ok(())
    }

    // Assign into register 'a' the remainder of 'b' divided by 'c' (modulo 32768).
    // Wrapping is unnecessary, as overflow is not possible, but maintains the
    // homogeneity of the code.
    fn rem(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction()?.register_value()?;
        let b = self.read_instruction().map(|i| self.get_value(i))?;
        let c = self.read_instruction().map(|i| self.get_value(i))?;

        self.registers[a as usize] = b.wrapping_rem(c);
        Ok(())
    }

    // Assign into register 'a' the bitwise and of 'b' and 'c'.
    fn and(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction()?.register_value()?;
        let b = self.read_instruction().map(|i| self.get_value(i))?;
        let c = self.read_instruction().map(|i| self.get_value(i))?;

        self.registers[a as usize] = b & c;
        Ok(())
    }

    // Assign into register 'a' the bitwise or of 'b' and 'c'.
    fn or(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction()?.register_value()?;
        let b = self.read_instruction().map(|i| self.get_value(i))?;
        let c = self.read_instruction().map(|i| self.get_value(i))?;

        self.registers[a as usize] = b | c;
        Ok(())
    }

    // Assign into register 'a' the 15-bit bitwise inverse of 'b'.
    fn not(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction()?.register_value()?;
        let b = self.read_instruction().map(|i| self.get_value(i))?;

        // Only operate on 15 bits, not the 16th.
        self.registers[a as usize] = b ^ 32767_u16;
        Ok(())
    }

    // Read memory at address 'b' and write to register 'a'.
    fn rmem(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction()?.register_value()?;
        let b = self.read_instruction().map(|i| self.get_value(i))?;

        self.registers[a as usize] = self.memory[b as usize];
        Ok(())
    }

    // Write the value from 'b' to memory at address 'a'.
    fn wmem(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction().map(|i| self.get_value(i))?;
        let b = self.read_instruction().map(|i| self.get_value(i))?;

        self.memory[a as usize] = b;
        Ok(())
    }

    // Push the next instruction's address to the stack and jump to 'a'.
    fn call(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction().map(|i| self.get_value(i))?;

        self.stack.push(self.instruction_pointer);
        self.instruction_pointer = a;
        Ok(())
    }

    // Remove the top element of the stack and jump to it. Error out if stack is
    // empty.
    fn ret(&mut self) -> Result<(), VirtualMachineError> {
        self.instruction_pointer = self.stack.pop().ok_or(VirtualMachineError::ReadFromEmptyStack)?;
        Ok(())
    }

    // Write the character with the value 'a' (or register 'a') to stdout.
    fn write(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction().map(|i| self.get_value(i))? as u8;

        let _ = io::stdout().write(&[a]);
        Ok(())
    }

    // Read a character from stdin and write its ascii code to register 'a'.
    fn read(&mut self) -> Result<(), VirtualMachineError> {
        let a = self.read_instruction()?.register_value()?;

        let mut buffer = [0; 1];
        match io::stdin().read_exact(&mut buffer) {
            Ok(_)   => {self.registers[a as usize] = buffer[0] as u16; Ok(())},
            Err(_)  => Err(VirtualMachineError::ReadError),
        }
    }
}
