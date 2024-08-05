use std::ops::Div;
use std::time::Duration;

const RAM_SIZE: usize = 4096;
const DISPLAY_SIZE: usize = 64 * 32;
const VARIABLE_REGISTER_SIZE: usize = 16;
const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

struct Memory {
    inner: [u8; RAM_SIZE],
}

impl Memory {
    fn get_instruction(&self, pos: u8) -> Result<u16, String> {
        let mut data = match self.inner.get(pos as usize) {
            Some(d) => *d as u16,
            None => {
                return Err(format!("index {pos} is out of bounds, memory size is {}", self.inner.len()));
            }
        };
        let mut instruction: u16 = data << 4;

        let pos = pos + 1;
        data = match self.inner.get(pos as usize) {
            Some(d) => *d as u16,
            None => {
                return Err(format!("index {pos} is out of bounds, memory size is {}", self.inner.len()));
            }
        };

        instruction |= data;
        Ok(instruction)
    }

    fn load(&mut self, pos: u16, data: &[u8]) -> Result<(), String> {
        if pos + data.len() as u16 > self.inner.len() as u16 {
            return Err(format!("data {} does not fit into memory {} at {}", data.len(), self.inner.len(), pos));
        }

        let range = (pos as usize)..(pos as usize + data.len());
        self.inner[range].copy_from_slice(&data);

        Ok(())
    }
}

struct Display {
    inner: [bool; DISPLAY_SIZE],
}

impl Display {
    fn draw(&mut self, x: usize, y: usize, flip: bool) -> Result<(), String> {
        let pos = x * y;
        if pos > DISPLAY_SIZE {
            return Err(format!("{x}:{y} is out of bounds for the display of size {DISPLAY_SIZE}"));
        }
        self.inner[pos] = self.inner[pos] != flip;
        Ok(())
    }

    fn clear(&mut self) {
        self.inner = [false; DISPLAY_SIZE]
    }
}

struct Stack {
    inner: Vec<u16>,
}

struct Timer {
    inner: u8,
}

struct Input {}

pub struct Chip8 {
    memory: Memory,
    display: Display,
    program_counter: u16,
    index_register: u16,
    stack: Stack,
    delay_timer: Timer,
    sound_timer: Timer,
    variable_registers: [u8; VARIABLE_REGISTER_SIZE],
    ticks: u64,
}

impl Chip8 {
    pub fn new(ticks: u64) -> Result<Self, String> {
        let mut chip = Self {
            memory: Memory { inner: [0u8; RAM_SIZE] },
            display: Display { inner: [false; DISPLAY_SIZE] },
            program_counter: 0,
            index_register: 0,
            stack: Stack { inner: Vec::new() },
            delay_timer: Timer { inner: 0 },
            sound_timer: Timer { inner: 0 },
            variable_registers: [0u8; VARIABLE_REGISTER_SIZE],
            ticks,
        };

        match chip.memory.load(0x050, &FONT) {
            Ok(_) => Ok(chip),
            Err(err) => Err(format!("could not load font into memory: {err}"))
        }
    }

    pub fn load_program(&mut self, data: &[u8]) -> Result<(), String> {
        if let Err(err) = self.memory.load(0x200, data) {
            return Err(format!("could not load program: {err}"));
        }

        self.program_counter = 0x200;
        Ok(())
    }


    fn fetch(&mut self) -> Result<u16, String> {
        let instruction = self.memory.get_instruction(self.program_counter as u8)?;
        self.program_counter += 2;
        Ok(instruction)
    }

    fn decode(instruction: u16) -> Result<Instruction, String> {
        let first = 0b1111 & (instruction >> 12) as u8;
        let second = 0b1111 & (instruction >> 8) as u8;
        let third = 0b1111 & (instruction >> 4) as u8;
        let fourth = 0b1111 & instruction as u8;
        match first {
            0x0 => {
                if second == 0x0 && third == 0xE && fourth == 0x0 {
                    return Ok(Instruction::ClearScreen);
                }
            }
            0x1 => {
                return Ok(Instruction::Jump((second, third, fourth)));
            }
            0x6 => {
                if second > 0xF {
                    return Err(format!("instruction contains invalid register {second}"));
                }
                return Ok(Instruction::SetRegister { register: second, value: (third, fourth) });
            }
            0x7 => {
                return Ok(Instruction::AddRegister { register: second, value: (third, fourth) });
            }
            0xA => {
                return Ok(Instruction::SetIndex((second, third, fourth)));
            }
            0xD => {
                return Ok(Instruction::Draw { x: second, y: third, sprite: fourth });
            }
            _ => {}
        }
        return Err(format!("unknown instruction: {first}{second}{third}{fourth}"));
    }

    fn execute(&mut self, instruction: Instruction) -> Result<(), String> {
        match instruction {
            Instruction::ClearScreen => { self.display.clear() }
            Instruction::Jump(_) => {}
            Instruction::SetRegister { register, value } => {}
            Instruction::AddRegister { .. } => {}
            Instruction::SetIndex(_) => {}
            Instruction::Draw { .. } => {}
        }
        Ok(())
    }
}

enum Instruction {
    ClearScreen,
    Jump((u8, u8, u8)),
    SetRegister {
        register: u8,
        value: (u8, u8),
    },
    AddRegister {
        register: u8,
        value: (u8, u8),
    },
    SetIndex((u8, u8, u8)),
    Draw {
        x: u8,
        y: u8,
        sprite: u8,
    },
}