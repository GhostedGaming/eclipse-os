use alloc::{vec, vec::Vec};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;
use x86_64::instructions::port::Port;

// PIT (Programmable Interval Timer) constants
const PIT_FREQUENCY: u32 = 1193182; // More precise base frequency
const PIT_COMMAND_PORT: u16 = 0x43;
const PIT_CHANNEL2_PORT: u16 = 0x42;
const SPEAKER_CONTROL_PORT: u16 = 0x61;

// Speaker control bits
const SPEAKER_ENABLE: u8 = 0x03;
const SPEAKER_DISABLE: u8 = 0xFC;

// Global state for the driver
static BEEP_TICKS: AtomicU32 = AtomicU32::new(0);
static IS_PLAYING: AtomicBool = AtomicBool::new(false);
static CURRENT_FREQUENCY: AtomicU32 = AtomicU32::new(0);

/// Musical note frequencies (in Hz)
#[derive(Debug, Clone, Copy)]
pub enum Note {
    C4 = 262,
    CS4 = 277, // C#/Db
    D4 = 294,
    DS4 = 311, // D#/Eb
    E4 = 330,
    F4 = 349,
    FS4 = 370, // F#/Gb
    G4 = 392,
    GS4 = 415, // G#/Ab
    A4 = 440,  // Concert pitch
    AS4 = 466, // A#/Bb
    B4 = 494,
    C5 = 523,
    CS5 = 554,
    D5 = 587,
    DS5 = 622,
    E5 = 659,
    F5 = 698,
    FS5 = 740,
    G5 = 784,
    GS5 = 831,
    A5 = 880,
    AS5 = 932,
    B5 = 988,
    C6 = 1047,
    Rest = 0, // Silence
}

impl Note {
    /// Get frequency for a note in any octave (0-8)
    pub fn frequency_in_octave(self, octave: u8) -> u32 {
        if matches!(self, Note::Rest) {
            return 0;
        }

        let base_freq = self as u32;
        let octave_multiplier = match octave {
            0 => 0.0625, // /16
            1 => 0.125,  // /8
            2 => 0.25,   // /4
            3 => 0.5,    // /2
            4 => 1.0,    // base
            5 => 2.0,    // *2
            6 => 4.0,    // *4
            7 => 8.0,    // *8
            8 => 16.0,   // *16
            _ => 1.0,
        };

        (base_freq as f32 * octave_multiplier) as u32
    }
}

/// Waveform types for tone generation
#[derive(Debug, Clone, Copy)]
pub enum Waveform {
    Square,  // Standard beep
    Pulse25, // 25% duty cycle
    Pulse75, // 75% duty cycle
}

/// A musical sequence entry
#[derive(Debug, Clone, Copy)]
pub struct MusicNote {
    pub note: Note,
    pub octave: u8,
    pub duration_ms: u32,
    pub waveform: Waveform,
}

impl MusicNote {
    pub fn new(note: Note, octave: u8, duration_ms: u32) -> Self {
        Self {
            note,
            octave,
            duration_ms,
            waveform: Waveform::Square,
        }
    }

    pub fn with_waveform(mut self, waveform: Waveform) -> Self {
        self.waveform = waveform;
        self
    }
}

/// Advanced PC Speaker Driver
pub struct PCSpeakerDriver {
    command_port: Port<u8>,
    channel2_port: Port<u8>,
    speaker_port: Port<u8>,
    current_sequence: Vec<MusicNote>,
    sequence_index: usize,
    note_timer: u32,
}

impl PCSpeakerDriver {
    /// Create a new PC Speaker driver instance
    pub fn new() -> Self {
        Self {
            command_port: Port::new(PIT_COMMAND_PORT),
            channel2_port: Port::new(PIT_CHANNEL2_PORT),
            speaker_port: Port::new(SPEAKER_CONTROL_PORT),
            current_sequence: Vec::new(),
            sequence_index: 0,
            note_timer: 0,
        }
    }

    /// Initialize the PC speaker system
    pub fn init(&mut self) {
        self.stop_sound();
        IS_PLAYING.store(false, Ordering::SeqCst);
        CURRENT_FREQUENCY.store(0, Ordering::SeqCst);
    }

    /// Set PIT to generate a specific frequency
    fn set_pit_frequency(&mut self, frequency: u32) {
        if frequency == 0 {
            self.disable_speaker();
            return;
        }

        // Clamp frequency to reasonable bounds
        let freq = frequency.max(20).min(20000);
        let divisor = PIT_FREQUENCY / freq;

        unsafe {
            // Configure PIT channel 2 for square wave generation
            self.command_port.write(0b10110110u8);

            // Send divisor (low byte first, then high byte)
            self.channel2_port.write((divisor & 0xFF) as u8);
            self.channel2_port.write((divisor >> 8) as u8);
        }

        CURRENT_FREQUENCY.store(freq, Ordering::SeqCst);
    }

    /// Enable the PC speaker
    fn enable_speaker(&mut self) {
        unsafe {
            let current = self.speaker_port.read();
            self.speaker_port.write(current | SPEAKER_ENABLE);
        }
        IS_PLAYING.store(true, Ordering::SeqCst);
    }

    /// Disable the PC speaker
    fn disable_speaker(&mut self) {
        unsafe {
            let current = self.speaker_port.read();
            self.speaker_port.write(current & SPEAKER_DISABLE);
        }
        IS_PLAYING.store(false, Ordering::SeqCst);
    }

    /// Play a continuous tone at the specified frequency
    pub fn play_tone(&mut self, frequency: u32) {
        self.set_pit_frequency(frequency);
        if frequency > 0 {
            self.enable_speaker();
        }
    }

    /// Play a musical note
    pub fn play_note(&mut self, note: Note, octave: u8) {
        let freq = note.frequency_in_octave(octave);
        self.play_tone(freq);
    }

    /// Stop all sound output
    pub fn stop_sound(&mut self) {
        self.disable_speaker();
        BEEP_TICKS.store(0, Ordering::SeqCst);
        CURRENT_FREQUENCY.store(0, Ordering::SeqCst);
    }

    /// Play a tone for a specific duration (in timer ticks)
    pub fn play_timed_tone(&mut self, frequency: u32, duration_ticks: u32) {
        self.play_tone(frequency);
        BEEP_TICKS.store(duration_ticks, Ordering::SeqCst);
    }

    /// Play a beep with specified frequency and duration in milliseconds
    pub fn beep(&mut self, frequency: u32, duration_ms: u32) {
        // Assuming 1000 ticks per second (adjust based on your timer frequency)
        let ticks = duration_ms;
        self.play_timed_tone(frequency, ticks);
    }

    /// Play a musical note for a specific duration
    pub fn play_note_timed(&mut self, note: Note, octave: u8, duration_ms: u32) {
        let freq = note.frequency_in_octave(octave);
        self.beep(freq, duration_ms);
    }

    /// Load a sequence of musical notes
    pub fn load_sequence(&mut self, sequence: Vec<MusicNote>) {
        self.current_sequence = sequence;
        self.sequence_index = 0;
        self.note_timer = 0;
    }

    /// Play a predefined melody
    pub fn play_melody(&mut self, melody: Melody) {
        let sequence = match melody {
            Melody::Startup => vec![
                MusicNote::new(Note::C4, 2, 1000),
                MusicNote::new(Note::C5, 2, 500),
            ],
            Melody::Error => vec![
                MusicNote::new(Note::C4, 5, 10),
                MusicNote::new(Note::C4, 5, 10),
                MusicNote::new(Note::C4, 5, 10),
            ],
            Melody::Success => vec![
                MusicNote::new(Note::C4, 4, 150),
                MusicNote::new(Note::E4, 4, 150),
                MusicNote::new(Note::G4, 4, 150),
                MusicNote::new(Note::C5, 4, 300),
                MusicNote::new(Note::Rest, 0, 100),
                MusicNote::new(Note::G4, 4, 150),
                MusicNote::new(Note::C5, 4, 300),
            ],
            Melody::Warning => vec![
                MusicNote::new(Note::A4, 4, 300),
                MusicNote::new(Note::Rest, 0, 100),
                MusicNote::new(Note::A4, 4, 300),
            ],
            Melody::PowerOn => vec![
                MusicNote::new(Note::C4, 3, 100),
                MusicNote::new(Note::C4, 4, 50),
                MusicNote::new(Note::C4, 6, 25),
            ],
            Melody::TetrisTheme => vec![
                // First phrase
                MusicNote::new(Note::E5, 3, 200),
                MusicNote::new(Note::B4, 2, 150),
                MusicNote::new(Note::C5, 3, 150),
                MusicNote::new(Note::D5, 3, 200),
                MusicNote::new(Note::C5, 3, 150),
                MusicNote::new(Note::B4, 2, 150),
                MusicNote::new(Note::A4, 2, 200),
                MusicNote::new(Note::A4, 2, 200),
                MusicNote::new(Note::Rest, 0, 50),
                // Second phrase
                MusicNote::new(Note::C5, 3, 150),
                MusicNote::new(Note::E5, 3, 200),
                MusicNote::new(Note::D5, 3, 200),
                MusicNote::new(Note::C5, 3, 150),
                MusicNote::new(Note::B4, 2, 150),
                MusicNote::new(Note::C5, 3, 150),
                MusicNote::new(Note::D5, 3, 200),
                MusicNote::new(Note::E5, 3, 200),
                // Third phrase (slightly higher section)
                MusicNote::new(Note::B4, 2, 150),
                MusicNote::new(Note::C5, 3, 150),
                MusicNote::new(Note::D5, 3, 200),
                MusicNote::new(Note::E5, 3, 200),
                MusicNote::new(Note::C5, 3, 150),
                MusicNote::new(Note::A4, 2, 150),
                MusicNote::new(Note::A4, 2, 200),
                MusicNote::new(Note::Rest, 0, 50),
                // Fourth phrase (ascending part)
                MusicNote::new(Note::D5, 3, 200),
                MusicNote::new(Note::F5, 3, 200),
                MusicNote::new(Note::A5, 3, 200),
                MusicNote::new(Note::G5, 3, 200),
                MusicNote::new(Note::F5, 3, 150),
                MusicNote::new(Note::E5, 3, 150),
                MusicNote::new(Note::C5, 3, 200),
                MusicNote::new(Note::E5, 3, 200),
                // Final phrase (wraps up the main loop)
                MusicNote::new(Note::D5, 3, 150),
                MusicNote::new(Note::C5, 3, 150),
                MusicNote::new(Note::B4, 2, 150),
                MusicNote::new(Note::C5, 3, 150),
                MusicNote::new(Note::D5, 3, 200),
                MusicNote::new(Note::E5, 3, 200),
                MusicNote::new(Note::Rest, 0, 100),
            ],
        };

        self.load_sequence(sequence);
    }

    /// Generate sound effects
    pub fn play_effect(&mut self, effect: SoundEffect) {
        match effect {
            SoundEffect::Click => self.beep(1000, 50),
            SoundEffect::Pop => self.beep(800, 100),
            SoundEffect::Chirp => {
                // Rising frequency chirp
                for freq in (400..=800).step_by(50) {
                    self.play_tone(freq);
                    // Small delay would go here in real implementation
                }
                self.stop_sound();
            }
            SoundEffect::Sweep => {
                // Frequency sweep
                for freq in (200..=2000).step_by(100) {
                    self.play_tone(freq);
                    use crate::time::delay_ms;
                    delay_ms(50);
                }
                self.stop_sound();
            }
            SoundEffect::Laser => {
                // Descending laser sound
                for freq in (100..=1000).rev().step_by(50) {
                    self.play_tone(freq);
                    // Small delay would go here
                }
                self.stop_sound();
            }
        }
    }

    /// Timer tick handler - call this from your timer interrupt
    pub fn timer_tick(&mut self) {
        // Handle timed beeps
        let current_ticks = BEEP_TICKS.load(Ordering::SeqCst);
        if current_ticks > 0 {
            if BEEP_TICKS.fetch_sub(1, Ordering::SeqCst) == 1 {
                self.stop_sound();
            }
        }

        // Handle music sequence playback
        if !self.current_sequence.is_empty() && self.sequence_index < self.current_sequence.len() {
            if self.note_timer == 0 {
                // Start playing the current note
                let current_note = self.current_sequence[self.sequence_index];
                let freq = current_note.note.frequency_in_octave(current_note.octave);

                if freq > 0 {
                    self.play_tone(freq);
                } else {
                    self.stop_sound();
                }

                self.note_timer = current_note.duration_ms;
            }

            self.note_timer -= 1;

            if self.note_timer == 0 {
                self.sequence_index += 1;
                if self.sequence_index >= self.current_sequence.len() {
                    // Sequence finished
                    self.stop_sound();
                    self.current_sequence.clear();
                    self.sequence_index = 0;
                }
            }
        }
    }

    /// Check if any sound is currently playing
    pub fn is_playing(&self) -> bool {
        IS_PLAYING.load(Ordering::SeqCst)
            || BEEP_TICKS.load(Ordering::SeqCst) > 0
            || !self.current_sequence.is_empty()
    }

    /// Get the current playing frequency
    pub fn current_frequency(&self) -> u32 {
        CURRENT_FREQUENCY.load(Ordering::SeqCst)
    }

    /// Emergency stop - immediately silence all audio
    pub fn emergency_stop(&mut self) {
        self.stop_sound();
        self.current_sequence.clear();
        self.sequence_index = 0;
        self.note_timer = 0;
        BEEP_TICKS.store(0, Ordering::SeqCst);
    }
}

/// Predefined melodies
#[derive(Debug, Clone, Copy)]
pub enum Melody {
    Startup,
    Error,
    Success,
    Warning,
    PowerOn,
    TetrisTheme,
}

/// Sound effects
#[derive(Debug, Clone, Copy)]
pub enum SoundEffect {
    Click,
    Pop,
    Chirp,
    Sweep,
    Laser,
}

// Safe global driver using Mutex
static DRIVER: Mutex<Option<PCSpeakerDriver>> = Mutex::new(None);

/// Initialize the global PC speaker driver
pub fn init_pc_speaker() {
    let mut driver = PCSpeakerDriver::new();
    driver.init();
    *DRIVER.lock() = Some(driver);
}

/// Execute a function with the global driver
fn with_driver<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut PCSpeakerDriver) -> R,
{
    DRIVER.lock().as_mut().map(f)
}

// Global convenience functions
pub fn play_tone(frequency: u32) {
    with_driver(|d| d.play_tone(frequency));
}

pub fn play_note(note: Note, octave: u8) {
    with_driver(|d| d.play_note(note, octave));
}

pub fn stop_sound() {
    with_driver(|d| d.stop_sound());
}

pub fn beep(frequency: u32, duration_ms: u32) {
    with_driver(|d| d.beep(frequency, duration_ms));
}

pub fn play_melody(melody: Melody) {
    with_driver(|d| d.play_melody(melody));
}

pub fn play_effect(effect: SoundEffect) {
    with_driver(|d| d.play_effect(effect));
}

pub fn timer_tick() {
    with_driver(|d| d.timer_tick());
}

pub fn is_playing() -> bool {
    with_driver(|d| d.is_playing()).unwrap_or(false)
}

pub fn emergency_stop() {
    with_driver(|d| d.emergency_stop());
}

// Utility macros for common operations
#[macro_export]
macro_rules! play_notes {
    ($($note:expr, $octave:expr, $duration:expr);* $(;)?) => {
        {
            let sequence = vec![
                $(MusicNote::new($note, $octave, $duration),)*
            ];
            with_driver(|d| d.load_sequence(sequence));
        }
    };
}

#[macro_export]
macro_rules! quick_beep {
    () => {
        beep(1000, 100)
    };
    ($freq:expr) => {
        beep($freq, 100)
    };
    ($freq:expr, $duration:expr) => {
        beep($freq, $duration)
    };
}
