const RAM_SIZE: usize = 4096;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
const VARIABLE_REGISTER_SIZE: usize = 16;
const FLAG_REGISTER: usize = 15;
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
    fn get_instruction(&self, pos: usize) -> Result<u16, String> {
        let mut data = match self.inner.get(pos) {
            Some(d) => *d as u16,
            None => {
                return Err(format!("index {pos} is out of bounds, memory size is {}", self.inner.len()));
            }
        };
        let mut instruction: u16 = data << 4;

        let pos = pos + 1;
        data = match self.inner.get(pos) {
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
    inner: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
}

impl Display {
    fn draw(&mut self, x: usize, y: usize, flip: bool) -> Result<bool, String> {
        let pos = x * y;
        if pos > DISPLAY_WIDTH * DISPLAY_HEIGHT {
            return Err(format!("{x}:{y} is out of bounds for the display of size {}", DISPLAY_WIDTH * DISPLAY_HEIGHT));
        }
        let old = self.inner[pos];
        self.inner[pos] = self.inner[pos] != flip;
        Ok(old == true && self.inner[pos] == false)
    }

    fn clear(&mut self) {
        self.inner = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT]
    }
}

struct Stack {
    inner: Vec<u16>,
}

struct Timer {
    inner: u8,
}

#[derive(Default)]
struct InputBuffer {
    inner: Vec<char>,
}

pub trait Screen {
    fn draw(&mut self, x: usize, y: usize, draw: bool);
    fn clear(&mut self);
}

pub struct Chip8 {
    memory: Memory,
    display: Display,
    input: InputBuffer,
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
            display: Display { inner: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT] },
            input: InputBuffer::default(),
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
        let instruction = self.memory.get_instruction(self.program_counter as usize)?;
        self.program_counter += 2;
        Ok(instruction)
    }

    fn decode(instruction: u16) -> Result<Instruction, String> {
        let first = 0b1111 & (instruction >> 12) as u8;
        let second = 0b1111 & (instruction >> 8) as u8;
        let third = 0b1111 & (instruction >> 4) as u8;
        let fourth = 0b1111 & instruction as u8;
        let number = 0b1111_1111 & instruction as u8;
        let address = 0b1111_1111_1111 & instruction;
        match first {
            0x0 => {
                if second == 0x0 && third == 0xE && fourth == 0x0 {
                    return Ok(Instruction::ClearScreen);
                }
            }
            0x1 => {
                return Ok(Instruction::Jump(address));
            }
            0x6 => {
                if second > 0xF {
                    return Err(format!("instruction contains invalid register {second}"));
                }
                return Ok(Instruction::SetRegister { register: second as usize, value: number });
            }
            0x7 => {
                return Ok(Instruction::AddRegister { register: second as usize, value: number });
            }
            0xA => {
                return Ok(Instruction::SetIndex(address));
            }
            0xD => {
                return Ok(Instruction::Draw { x_register: second as usize, y_register: third as usize, count: fourth });
            }
            _ => {}
        }
        return Err(format!("unknown instruction: {first}{second}{third}{fourth}"));
    }

    fn execute(&mut self, instruction: Instruction) -> Result<(), String> {
        match instruction {
            Instruction::ClearScreen => { self.display.clear() }
            Instruction::Jump(address) => {
                self.program_counter = address;
            }
            Instruction::SetRegister { register, value } => { self.variable_registers[register] = value }
            Instruction::AddRegister { register, value } => { self.variable_registers[register] += value }
            Instruction::SetIndex(address) => { self.index_register = address }
            Instruction::Draw { x_register, y_register, count } => {
                let mut x = (self.variable_registers[x_register] & ((DISPLAY_WIDTH - 1) as u8)) as usize;
                let mut y = (self.variable_registers[y_register] & ((DISPLAY_HEIGHT - 1) as u8)) as usize;
                self.variable_registers[FLAG_REGISTER] = 0;

                let begin = self.index_register as usize;
                let end = (self.index_register + count as u16) as usize;
                for i in begin..end {
                    let sprite_row = self.memory.inner[i];
                    let bits = get_bits(sprite_row);

                    for bit in bits {
                        let turned_off = self.display.draw(x, y, bit)?;
                        if turned_off {
                            self.variable_registers[FLAG_REGISTER] = 1;
                        }
                        x += 1;

                        if x == DISPLAY_WIDTH {
                            break;
                        }
                    }

                    y += 1;

                    if x == DISPLAY_WIDTH && y == DISPLAY_HEIGHT {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

fn get_bits(byte: u8) -> [bool; 8] {
    let mut bits = [false; 8];
    for i in 0..8 {
        let bit = byte >> i & 1;
        bits[i] = bit == 1;
    }

    bits
}

fn nth_nibble(instruction: u16, nth: u8) -> Result<u8, String> {
    match nth {
        1 => Ok(0b1111 & (instruction >> 12) as u8),
        2 => Ok(0b1111 & (instruction >> 8) as u8),
        3 => Ok(0b1111 & (instruction >> 4) as u8),
        4 => Ok(0b1111 & instruction as u8),
        _ => {
            return Err(format!("valid range for nibbles are 1-4 but got {nth}"));
        }
    }
}

enum Instruction {
    ClearScreen,
    Jump(u16),
    SetRegister {
        register: usize,
        value: u8,
    },
    AddRegister {
        register: usize,
        value: u8,
    },
    SetIndex(u16),
    Draw {
        x_register: usize,
        y_register: usize,
        count: u8,
    },
}