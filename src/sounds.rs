use core::sync::atomic::{AtomicU32, Ordering};
use x86_64::instructions::port::Port;
use crate::println;

static BEEP_TICKS: AtomicU32 = AtomicU32::new(0);

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

/// Play a beep for a given number of timer ticks (e.g., 10-20)
pub fn play_beep_for(ticks: u32, freq: u32) {
    play_sound(freq);
    BEEP_TICKS.store(ticks, Ordering::SeqCst);
}

/// Call this from your timer interrupt handler
pub fn beep_tick() {
    if BEEP_TICKS.load(Ordering::SeqCst) > 0 {
        println!("beep_tick: {}", BEEP_TICKS.load(Ordering::SeqCst));
        if BEEP_TICKS.fetch_sub(1, Ordering::SeqCst) == 1 {
            stop_sound();
        }
    }
}