use std::fs::File;
use std::io::{self, Read, Write};

const MODULO: u16 = 32768_u16;

/*
 * Architecture specs (15-bit address space, 16-bit values, 8 registers,
 * unlimited stack). Numbers are stored as little-endian pairs (low-high).
 */
pub struct VirtualMachine {
    memory: [u16; 32768],
    registers: [u16; 8],
    stack: Vec<u16>,
    instruction_pointer: u16,
}

enum Instruction {
    Register(u16),
    Value(u16),
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
        buffer.chunks(2)
            .enumerate()
            .for_each(|(i, v)|
                self.memory[i] = (v[0] as u16) ^ ((v[1] as u16) << 8));
    }

    pub fn execute_program(&mut self) -> Result<(), String> {
        let mut result: Result<(), String> = Ok(());

        while let Ok(()) = result {
            let instruction = match self.read_instruction() {
                Instruction::Register(x) => x,
                Instruction::Value(x) => x,
            };
            
            result = match instruction {
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
                _ => Err(String::from("Invalid opcode.")),
            };
        }

        result
    }

    fn read_instruction(&mut self) -> Instruction {
        let instruction = self.memory[self.instruction_pointer as usize];
        self.instruction_pointer += 1;

        match instruction {
            x @ 0 ... 32767 => Instruction::Value(x),
            x @ 32768 ... 32775 => Instruction::Register(x % 32768),
            _ => panic!("Invalid instruction.")
        }
    }

    // Halt execution (not really an error, but simpler this way.)
    fn halt(&mut self) -> Result<(), String> {
        Err(String::from("Halting execution."))
    }    

    // Set register 'a' to the value 'b' (or value in register 'b').
    fn set(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => x,
            Instruction::Value(_) => return Err(String::from("Trying to write to a value!")),
        };

        let b = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        self.registers[a as usize] = b;
        Ok(())
    }

    // Push value of register 'a' onto the stack.
    fn push(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        self.stack.push(a);
        Ok(())
    }

    // Pop the stack and write to register 'a'. Error out if the stack is empty.
    fn pop(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => x,
            Instruction::Value(_) => return Err(String::from("Trying to write to a value!")),
        };

        let b = match self.stack.pop() {
            Some(x) => x,
            None    => return Err(String::from("Reading from an empty stack!")),
        };

        self.registers[a as usize] = b;
        Ok(())
    }

    // Set register 'a' to 1 if 'b' == 'c'. 0 otherwise.
    fn eq(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => x,
            Instruction::Value(_) => return Err(String::from("Trying to write to a value!")),
        };

        let b = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        let c = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        self.registers[a as usize] = u16::from(b == c);
        Ok(())
    }

    // Set register 'a' to 1 if 'b' > 'c'. 0 otherwise.
    fn gt(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => x,
            Instruction::Value(_) => return Err(String::from("Trying to write to a value!")),
        };

        let b = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        let c = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        self.registers[a as usize] = u16::from(b > c);
        Ok(())
    }

    // Jump to instruction 'a' (value or register).
    fn jmp(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        self.instruction_pointer = a;
        Ok(())
    }

    // If 'a' != 0, jump to instruction 'b' (value or register).
    fn jt(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        let b = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        match a {
            0   => (),
            _   => self.instruction_pointer = b,
        }

        Ok(())
    }

    // If 'a' == 0, jump to instruction 'b' (value or register).
    fn jf(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        let b = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        match a {
            0   => self.instruction_pointer = b,
            _   => (),
        };

        Ok(())
    }

    // Assign into register 'a' the sum of 'b' and 'c' (modulo 32768).
    fn add(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => x,
            Instruction::Value(_) => return Err(String::from("Trying to write to a value!")),
        };

        let b = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        let c = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        self.registers[a as usize] = b.wrapping_add(c) % MODULO;
        Ok(())
    }

    // Assign into register 'a' the product of 'b' and 'c' (modulo 32768).
    fn mult(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => x,
            Instruction::Value(_) => return Err(String::from("Trying to write to a value!")),
        };

        let b = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        let c = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        self.registers[a as usize] = b.wrapping_mul(c) % MODULO;
        Ok(())
    }

    // Assign into register 'a' the remainder of 'b' divided by 'c' (modulo 32768).
    // Wrapping is unneccessary, as overflow is not possible, but maintains the
    // homogeneity of the code.
    fn rem(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => x,
            Instruction::Value(_) => return Err(String::from("Trying to write to a value!")),
        };

        let b = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        let c = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        self.registers[a as usize] = b.wrapping_rem(c);
        Ok(())
    }

    // Assign into register 'a' the bitwise and of 'b' and 'c'.
    fn and(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => x,
            Instruction::Value(_) => return Err(String::from("Trying to write to a value!")),
        };

        let b = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        let c = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        self.registers[a as usize] = b & c;
        Ok(())
    }

    // Assign into register 'a' the bitwise or of 'b' and 'c'.
    fn or(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => x,
            Instruction::Value(_) => return Err(String::from("Trying to write to a value!")),
        };

        let b = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        let c = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        self.registers[a as usize] = b | c;
        Ok(())
    }

    // Assign into register 'a' the 15-bit bitwise inverse of 'b'.
    fn not(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => x,
            Instruction::Value(_) => return Err(String::from("Trying to write to a value!")),
        };

        let b = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        // Only operate on 15 bits, not the 16th.
        self.registers[a as usize] = b ^ 32767_u16;
        Ok(())
    }

    // Read memory at address 'b' and write to register 'a'.
    fn rmem(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => x,
            Instruction::Value(_) => return Err(String::from("Trying to write to a value!")),
        };

        let b = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        self.registers[a as usize] = self.memory[b as usize];
        Ok(())
    }

    // Write the value from 'b' to memory at address 'a'.
    fn wmem(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        let b = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        self.memory[a as usize] = b;
        Ok(())
    }

    // Push the next instruction's address to the stack and jump to 'a'.
    fn call(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        };

        self.stack.push(self.instruction_pointer);
        self.instruction_pointer = a;
        Ok(())
    }

    // Remove the top element of the stack and jump to it. Error out if stack is
    // empty.
    fn ret(&mut self) -> Result<(), String> {
        match self.stack.pop() {
            Some(x) => self.instruction_pointer = x,
            None    => return Err(String::from("Reading from an empty stack!")),
        }
        
        Ok(())
    }

    // Write the character with the value 'a' (or register 'a') to stdout.
    fn write(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => self.registers[x as usize],
            Instruction::Value(x) => x,
        } as u8;

        let _ = io::stdout().write(&[a]);
        Ok(())
    }

    // Read a character from stdin and write its ascii code to register 'a'.
    fn read(&mut self) -> Result<(), String> {
        let a = match self.read_instruction() {
            Instruction::Register(x) => x,
            Instruction::Value(_) => return Err(String::from("Trying to write to a value!")),
        };

        let mut buffer = [0; 1];
        match io::stdin().read_exact(&mut buffer) {
            Ok(_)   => {self.registers[a as usize] = buffer[0] as u16; Ok(())},
            Err(x)  => Err(x.to_string()),
        }
    }
}
