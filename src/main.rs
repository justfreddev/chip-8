mod chip;

use chip::Chip8;

fn main() {
    let mut chip = Chip8::new(true);
    chip.clear_display();

    if let Err(e) = chip.load_rom("BRIX") {
        eprintln!("An error occured when loading the rom: {e}");
    }

    loop {
        chip.get_next_instruction();

        chip.execute();
    }
}
