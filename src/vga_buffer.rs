use core::fmt;
use futures_util::future::select;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

use crate::{serial, serial_println};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,      // Full square/block cursor
    Underline,  // Underline cursor
    Invert,     // Current implementation (inverted colors)
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        cursor_visible: true,
        cursor_color: ColorCode::new(Color::Black, Color::LightGray), // Inverted colors for cursor
        cursor_style: CursorStyle::Block, // Default to block cursor
    });
}

/// The standard color palette in VGA text mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Purple = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// A combination of a foreground and a background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    /// Create a new `ColorCode` with the given foreground and background colors.
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// A screen character in the VGA text buffer, consisting of an ASCII character and a `ColorCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// The height of the text buffer (normally 25 lines).
const BUFFER_HEIGHT: usize = 25;
/// The width of the text buffer (normally 80 columns).
const BUFFER_WIDTH: usize = 80;

/// A structure representing the VGA text buffer.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A writer type that allows writing ASCII bytes and strings to an underlying `Buffer`.
///
/// Wraps lines at `BUFFER_WIDTH`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
    cursor_visible: bool,
    cursor_color: ColorCode,
    cursor_style: CursorStyle,
}

impl Writer {
    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }
    
    pub fn write_byte(&mut self, byte: u8) {
        // Erase cursor before writing
        self.erase_cursor();
        
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
    
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
    
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
        
        // Draw cursor after writing
        if self.cursor_visible {
            self.draw_cursor();
        }
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Shifts all lines one line up and clears the last row.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Clears a row by overwriting it with blank characters.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.column_position > 0 {
            // Erase cursor before modifying
            self.erase_cursor();
            
            // Move cursor back one position
            self.column_position -= 1;
            
            // Create a blank ScreenChar with the current color code
            let blank = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };
            
            // Write the blank character to erase the previous character
            self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position].write(blank);
            
            // Redraw cursor at new position
            if self.cursor_visible {
                self.draw_cursor();
            }
        }
    }

    pub fn draw_cursor(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        
        // Get the current character at cursor position
        let current_char = self.buffer.chars[row][col].read();
        
        let cursor_char = match self.cursor_style {
            CursorStyle::Block => {
                // For block cursor, use space character with inverted colors
                ScreenChar {
                    ascii_character: b' ',
                    color_code: ColorCode::new(Color::Black, Color::Yellow), // Fully inverted block
                }
            },
            CursorStyle::Underline => {
                // For underline, use underscore character
                ScreenChar {
                    ascii_character: b'_',
                    color_code: self.cursor_color,
                }
            },
            CursorStyle::Invert => {
                // Current implementation (inverted colors)
                ScreenChar {
                    ascii_character: current_char.ascii_character,
                    color_code: self.cursor_color,
                }
            },
        };
        
        // Draw the cursor
        self.buffer.chars[row][col].write(cursor_char);
    }
    
    /// Erases the cursor by restoring the original character
    pub fn erase_cursor(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        
        if col < BUFFER_WIDTH {
            // Get the current character at cursor position
            let current_char = self.buffer.chars[row][col].read();
            
            // Restore the character with normal colors
            let normal_char = ScreenChar {
                ascii_character: if self.cursor_style == CursorStyle::Block || self.cursor_style == CursorStyle::Underline {
                    b' ' // For block or underline cursor, restore with space
                } else {
                    current_char.ascii_character // For inverted cursor, keep the character
                },
                color_code: self.color_code,
            };
            
            // Draw the normal character
            self.buffer.chars[row][col].write(normal_char);
        }
    }
    
    /// Toggles cursor visibility (for blinking)
    pub fn toggle_cursor(&mut self) {
        if self.cursor_visible {
            self.erase_cursor();
        } else {
            self.draw_cursor();
        }
        self.cursor_visible = !self.cursor_visible;
    }
    
    /// Sets the cursor visibility
    pub fn set_cursor_visibility(&mut self, visible: bool) {
        if visible != self.cursor_visible {
            if visible {
                self.draw_cursor();
            } else {
                self.erase_cursor();
            }
            self.cursor_visible = visible;
        }
    }
    
    /// Moves the cursor to a new position
    pub fn move_cursor(&mut self, new_col: usize) {
        // Erase cursor at current position
        self.erase_cursor();
        
        // Update position
        self.column_position = new_col;
        
        // Draw cursor at new position
        if self.cursor_visible {
            self.draw_cursor();
        }
    }
    
    /// Sets the cursor style
    pub fn set_cursor_style(&mut self, style: CursorStyle) {
        self.erase_cursor();
        self.cursor_style = style;
        if self.cursor_visible {
            self.draw_cursor();
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub fn set_color(foreground: Color, background: Color) {
    use x86_64::instructions::interrupts;
    
    interrupts::without_interrupts(|| {
        WRITER.lock().set_color(foreground, background);
    });
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Prints the given formatted string to the VGA text buffer
/// through the global `WRITER` instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

pub fn backspace() {
    use x86_64::instructions::interrupts;
    
    interrupts::without_interrupts(|| {
        WRITER.lock().handle_backspace();
    });
}

/// Sets the cursor style globally
pub fn set_cursor_style(style: CursorStyle) {
    use x86_64::instructions::interrupts;
    
    interrupts::without_interrupts(|| {
        WRITER.lock().set_cursor_style(style);
    });
}

/// Sets the cursor visibility globally
pub fn set_cursor_visibility(visible: bool) {
    use x86_64::instructions::interrupts;
    
    interrupts::without_interrupts(|| {
        WRITER.lock().set_cursor_visibility(visible);
    });
}

/// Force redraw of the cursor
pub fn redraw_cursor() {
    use x86_64::instructions::interrupts;
    
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writer.erase_cursor();
        if writer.cursor_visible {
            writer.draw_cursor();
        }
    });
}

pub fn clear_screen() {
    use x86_64::instructions::interrupts;
    
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        for row in 0..BUFFER_HEIGHT {
            writer.clear_row(row);
        }
        writer.column_position = 0;
        
        // Redraw cursor if needed
        if writer.cursor_visible {
            writer.draw_cursor();
        }
    });
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}

pub fn test_vga() {
    let colors = [
        Color::Black, Color::Blue, Color::Green, Color::Cyan,
        Color::Red, Color::Purple, Color::Brown, Color::LightGray,
        Color::DarkGray, Color::LightBlue, Color::LightGreen, Color::LightCyan,
        Color::LightRed, Color::Pink, Color::Yellow, Color::White
    ];

    for fg_color in colors.iter() {
        set_color(*fg_color, Color::Black);
        println!("#");
    }

    for bg_color in colors.iter() {
        set_color(Color::White, *bg_color);
        print!(" ");
        set_color(Color::White, Color::Black);
        print!("\n")
    }
    
    set_color(Color::White, Color::Black);

    print!("\nvga_buffer [");
    set_color(Color::Green, Color::Black);
    print!("OK");
    set_color(Color::White, Color::Black);
    print!("]\n");
}
