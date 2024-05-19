use twelve_bit::u12;


// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
// +---------------+= 0xFFF (4095) End of Chip-8 RAM
// |               |
// |               |
// | 0x200 to 0xFFF|
// |     Chip-8    |           
// | Program / Data|
// |     Space     |
// |               |
// |               |
// +---------------+= 0x200 (512) Start of most Chip-8 programs
// | 0x000 to 0x1FF|
// | Reserved for  |
// |  interpreter  |
// +---------------+= 0x000 (0) Start of Chip-8 RAM


/// The main struct for the interpreter:
/// opcode: stores the opcode of the current instruction
/// ar: The address register (I) is used to read and write to memory
/// pc: The program counter stores the address currently being executed
/// sp: The stack pointer points to the topmost level of the stack
/// stack: Used to store the address that the interpreter should return to when finished with a subroutine
/// registers: 16 general purpose 8-bit registers, Vx, x being hex
/// mem: 4 whole KB of RAM, in the layout shown above
/// delay: Used for timings of events in games, can be written and read
/// sound: Used for sound effects, When != 0, beeping is made. Ticks down at 60Hz and can only be set
/// fontset: The ways to represent
/// graphics: The array of 1s and 0s that make up whether pixels of the 64x32 screen is black or white
pub struct Chip8 {
    opcode: u16,
    ar: u12::U12,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    registers: [u8; 16],
    mem: [u8; 4096],
    delay: u8,
    sound: u8,
    fontset: [u8; 80],
    graphics: [u8; 2048],
    debug: bool,
}

impl Chip8 {
    pub fn new(debug: bool) -> Self {
        let fontset = [
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

        let mut mem: [u8; 4096] = [0; 4096];

        for i in 0..fontset.len() {
            mem[i] = fontset[i];
        }

        return Self {
            opcode: 0,
            ar: u12::MIN,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            registers: [0; 16],
            mem,
            delay: 0,
            sound: 0,
            fontset,
            graphics: [0; 2048],
            debug,
        }
    }

    /// Loads the rom with with the name given in the parameter
    /// It reads the binary file and converts it to a Vec<u8>
    /// Then loops over the file and stores it in memory starting at 0x200
    pub fn load_rom(&mut self, name: &str) -> Result<(), std::io::Error> {
        let file = std::fs::read(format!("./roms/{name}").as_str())?;

        for i in 0..file.len() {
            self.mem[0x200 + i] = file[i];
        }

        return Ok(());
    }

    /// Executes the next instruction
    pub fn execute(&mut self) {
        self.get_next_instruction();

        if self.debug {
            println!(
                "OPCODE: 0x{} {}, PC: {}, I: {:?}",
                self.mem[self.pc as usize],
                self.mem[(self.pc as usize) + 1],
                self.pc,
                self.ar
            );
        }

        match (self.opcode >> 12) & 0xF {
            0x0 => {
                match self.opcode {
                    0x00E0 => self.clear_display(),
                    0x00EE => {
                        // Sets the PC to the address at the top of the stack
                        for i in 0..self.stack.len() {
                            if i == 0 {
                                self.pc = self.stack[i - 1];
                                self.sp -= 1;
                                break;
                            }
                            if i == self.stack.len() - 1 {
                                eprintln!("Stack overflow");
                            }
                        }
                        eprintln!("Stack is empty")
                    },
                    _ => eprint!("Unknown instruction")
                }
            },
            0x1 => self.pc = self.opcode & 0xF,
            0x2 => {
                // Call address nnn
                self.sp += 1;
                // Put the PC on top of the stack
                for i in 0..self.stack.len() {
                    if self.stack[i] == 0 {
                        self.stack[i] = self.pc;
                        break;
                    }
                }
                // Set the pc to the call address
                self.pc = self.opcode >> 4;
            },
            0x3 => {
                let x = ((self.opcode >> 8) & 0x0F) as u8;
                let kk = (self.opcode & 0xFF) as u8;
                if self.registers[x as usize] == kk {
                    self.pc += 2;
                }
            },
            0x4 => {
                let x = ((self.opcode >> 8) & 0x0F) as u8;
                let kk = (self.opcode & 0xFF) as u8;
                if self.registers[x as usize] != kk {
                    self.pc += 2;
                }
            },
            0x5 => {
                let vx = self.registers[((self.opcode >> 8) & 0x0F) as usize];
                let vy = self.registers[((self.opcode >> 4) & 0x0F) as usize];
                if vx == vy {
                    self.pc += 2;
                }
            },
            0x6 => {
                println!("{}", (self.opcode >> 8) & 0x0F);
                self.registers[((self.opcode >> 8) & 0x0F) as usize] = (self.opcode & 0xFF) as u8
            },
            0x7 => self.registers[((self.opcode >> 8) & 0x0F) as usize] += (self.opcode & 0xFF) as u8,
            0x8 => {
                let vx = self.registers[((self.opcode >> 8) & 0x0F) as usize];
                let vy = self.registers[((self.opcode >> 4) & 0x0F) as usize];
                match self.opcode >> 12 {
                    0x0 => self.registers[((self.opcode >> 8) & 0x0F) as usize] = vy,
                    0x1 => self.registers[((self.opcode >> 8) & 0x0F) as usize] = vx | vy,
                    0x2 => self.registers[((self.opcode >> 8) & 0x0F) as usize] = vx & vy,
                    0x3 => self.registers[((self.opcode >> 8) & 0x0F) as usize] = vx ^ vy,
                    0x4 => {
                        let mut result = vx as u16 + vy as u16;
                        if result > 255 {
                            self.registers[0xF] = 1;
                            result %= 255;
                        } else {
                            self.registers[0xF] = 0;
                        }
                        self.registers[((self.opcode >> 8) & 0x0F) as usize] = result as u8;
                    },
                    0x5 => {
                        if vx > vy {
                            self.registers[0xF] = 1;
                        } else {
                            self.registers[0xF] = 0;
                        }
                        self.registers[((self.opcode >> 8) & 0x0F) as usize] = vx - vy;
                    },
                    0x6 => {
                        self.registers[0xF] = vx & 1;
                        self.registers[((self.opcode >> 8) & 0x0F) as usize] = vx >> 1;
                    },
                    0x7 => {
                        if vy > vx {
                            self.registers[0xF] = 1;
                        } else {
                            self.registers[0xF] = 0;
                        }
                        self.registers[((self.opcode >> 8) & 0x0F) as usize] = vy - vx;
                    },
                    0xE => {
                        self.registers[0xF] = (vx >> 7) & 1;
                        self.registers[((self.opcode >> 8) & 0x0F) as usize] = vx << 1;
                    }
                    _ => eprintln!("Unknown instruction")
                }
            },
            0x9 => {
                let vx = self.registers[((self.opcode >> 8) & 0x0F) as usize];
                let vy = self.registers[((self.opcode >> 4) & 0x0F) as usize];

                if vx != vy {
                    self.pc += 2;
                }
            },
            0xA => self.registers[usize::from(self.ar)] = (self.opcode & 0xF) as u8,
            0xB => {
                let v0 = self.registers[0x0];
                self.pc = self.opcode & 0xF + v0 as u16;
            },
            0xC => {
                let rand_byte = rand::random::<u8>();
                let kk = (self.opcode & 0xFF) as u8;
                self.registers[((self.opcode >> 8) & 0x0F) as usize] = rand_byte & kk;
            },
            0xD => self.draw_sprite(),
            0xE => {
                match self.opcode & 0xFF {
                    0x9E => todo!(),
                    0xA1 => todo!(),
                    _ => eprintln!("Unknown instruction")
                }
            },
            0xF => {
                let vx = self.registers[((self.opcode >> 8) & 0x0F) as usize];
                match self.opcode & 0xFF {
                    0x07 => self.registers[((self.opcode >> 8) & 0x0F) as usize] = self.delay,
                    0x0A => todo!(),
                    0x15 => self.sound = vx,
                    0x18 => self.delay = vx,
                    0x1E => self.ar = self.ar + u12::U12::from(vx),
                    0x29 => self.ar = u12::U12::from(vx * 0x5),
                    0x33 => {
                        let value: Vec<u8> = vx
                            .to_string()
                            .chars()
                            .map(|c| c.to_digit(10).unwrap() as u8)
                            .collect::<Vec<u8>>()
                            .into_iter()
                            .rev()
                            .collect();
                        for i in 0..value.len() {
                            self.mem[usize::from(self.ar) + i] = value[i];
                        }
                    },
                    0x55 => {
                        for i in 0..=((self.opcode >> 8) & 0x0F) as usize {
                            self.mem[usize::from(self.ar) + i] = self.registers[i];
                        }
                    },
                    0x65 => {
                        for i in 0..=((self.opcode >> 8) & 0x0F) as usize {
                            self.registers[i] = self.mem[usize::from(self.ar) + i];
                        }
                    },
                    _ => eprintln!("Unknown instruction")
                }
            }
            _ => {}
        }
    }

    pub fn get_next_instruction(&mut self) {
        // Get the index of the current instruction
        let i = self.pc as usize;

        // Gets the first 8-bit instruction, left shift it to the first 8 bits, and then
        // bitwise OR the next instruction so it takes up the second 8 bits
        self.opcode = (self.mem[i] as u16) << 8 | self.mem[i + 1] as u16;

        // Increment the PC twice
        self.pc += 2;
    }

    pub fn clear_display(&mut self) {
        // Resets the graphics array to all 0s
        self.graphics.fill(0);
    }

    fn draw_sprite(&mut self) {
        let x = ((self.opcode >> 8) & 0x0F) as u8;
        let y = ((self.opcode >> 4) & 0x0F) as u8;
        let n = (self.opcode >> 12) as u8;

        let x_coord = self.registers[x as usize] % 64;
        let y_coord = self.registers[y as usize] % 32;

        self.registers[0xF] = 0;

        for row in 0..n {
            let sprite = self.mem[usize::from(self.ar) + row as usize];
            let mut bits = [0u8; 8];
            for i in 0..8 {
                bits[7 - i] = ((sprite >> i) & 1) as u8;
            }
            for col in 0..8 {
                let x_cor = x_coord + col;
                let y_cor = y_coord + row;
                if bits[col as usize] == 1 && self.graphics[(x_cor * 64 + y_cor) as usize] == 1 {
                    self.registers[0xF] = 1;
                } else if bits[col as usize] == 1 {
                    self.graphics[(x_cor * 64 + y_cor) as usize] = 1;
                }
                if y_cor > 63 {
                    break;
                }
            }
            if x_coord + row > 31 {
                break;
            }
        }
    }
}