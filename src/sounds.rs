use core::sync::atomic::{AtomicU32, Ordering};
use x86_64::instructions::port::Port;

static BEEP_TICKS: AtomicU32 = AtomicU32::new(0);

pub fn play_sound(frequency: u32) {
    let div = 1193180 / frequency;
    let mut command_port = Port::new(0x43);
    let mut channel2_port = Port::new(0x42);
    let mut speaker_port = Port::new(0x61);

    // Set the PIT to the desired frequency
    unsafe {
        command_port.write(0b10110110u8);
        channel2_port.write((div & 0xFF) as u8);
        channel2_port.write((div >> 8) as u8);
    }

    // Activate the speaker
    unsafe {
        let speaker: u8 = speaker_port.read();
        speaker_port.write(speaker | 0x03);
    }
}

pub fn stop_sound() {
    let mut speaker_port = Port::new(0x61);

    // Disable the speaker
    unsafe {
        let speaker: u8 = speaker_port.read();
        speaker_port.write(speaker & 0xFC);
    }
}

pub fn play_beep_for(ticks: u32, freq: u32) {
    play_sound(freq);
    BEEP_TICKS.store(ticks, Ordering::SeqCst);
}

pub fn beep_tick() {
    let current_ticks = BEEP_TICKS.load(Ordering::SeqCst);
    if current_ticks > 0 {
        if BEEP_TICKS.fetch_sub(1, Ordering::SeqCst) == 1 {
            stop_sound();
        }
    }
}

// Helper function to check if a timed beep is active
pub fn is_beep_active() -> bool {
    BEEP_TICKS.load(Ordering::SeqCst) > 0
}

// Function to cancel any active timed beep
pub fn cancel_beep() {
    BEEP_TICKS.store(0, Ordering::SeqCst);
    stop_sound();
}