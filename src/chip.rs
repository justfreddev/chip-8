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
}

impl Chip8 {
    pub fn new() -> Self {
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

        return Self {
            opcode: 0,
            ar: u12::MIN,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            registers: [0; 16],
            mem: [0; 4096],
            delay: 0,
            sound: 0,
            fontset,
        }
    }
}