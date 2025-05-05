use x86_64::instructions::port::Port;

pub fn play_sound(frequency: u32) {
    use x86_64::instructions::interrupts;

    let div = 1193180 / frequency;
    let mut command_port = Port::new(0x43);
    let mut channel2_port = Port::new(0x42);
    let mut speaker_port = Port::new(0x61);

    // Set the PIT to the desired frequency
    unsafe {
        command_port.write(0b10110110u8); // Cast to u8
        channel2_port.write((div & 0xFF) as u8); // Low byte
        channel2_port.write((div >> 8) as u8);   // High byte
    }

    interrupts::without_interrupts(|| { 
        unsafe {
            let speaker: u8 = speaker_port.read();
            if speaker & 0x03 != 0x03 {
                speaker_port.write(speaker | 0x03);
            }
        }
    });
}

pub fn stop_sound() {
    use x86_64::instructions::interrupts;

    let mut speaker_port = Port::new(0x61);

    interrupts::without_interrupts(|| {
            // Disable the PC speaker
    unsafe {
        let speaker: u8 = speaker_port.read();
        speaker_port.write(speaker & 0xFC);
    }
    });
}